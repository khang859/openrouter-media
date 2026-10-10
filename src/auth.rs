use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::Result;
use crate::api::encode_component;
use crate::env::Env;
use crate::files::create_private_dir;

/// The key and where it came from.
#[derive(Debug, PartialEq)]
pub struct Found {
    pub key: String,
    pub source: String,
}

pub fn credentials_path(env: &Env) -> Result<PathBuf> {
    Ok(env.config_dir()?.join("credentials.json"))
}

/// The environment wins over the stored key.
pub fn read_key(env: &Env) -> Option<Found> {
    if let Some(key) = env.get("OPENROUTER_API_KEY") {
        return Some(Found {
            key: key.to_string(),
            source: "OPENROUTER_API_KEY".to_string(),
        });
    }
    // No file, or one that is not ours to read: the same as not signed in.
    let path = credentials_path(env).ok()?;
    let stored: Value = serde_json::from_str(&fs::read_to_string(&path).ok()?).ok()?;
    let key = stored["key"].as_str().filter(|key| !key.is_empty())?;
    Some(Found {
        key: key.to_string(),
        source: path.display().to_string(),
    })
}

pub fn save_key(key: &str, env: &Env) -> Result<PathBuf> {
    let path = credentials_path(env)?;
    let failed =
        |error: std::io::Error| format!("Cannot store the key in {}: {error}", path.display());
    create_private_dir(&env.config_dir()?).map_err(failed)?;
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let text = format!("{:#}\n", json!({ "key": key }));
    options
        .open(&path)
        .and_then(|mut file| file.write_all(text.as_bytes()))
        .map_err(failed)?;
    Ok(path)
}

pub fn delete_key(env: &Env) -> Result<()> {
    let path = credentials_path(env)?;
    match fs::remove_file(&path) {
        Err(error) if error.kind() != ErrorKind::NotFound => {
            Err(format!("Cannot delete {}: {error}", path.display()))
        }
        _ => Ok(()),
    }
}

fn random_token(bytes: usize) -> Result<String> {
    let mut buffer = vec![0; bytes];
    getrandom::fill(&mut buffer).map_err(|error| format!("Cannot get random bytes: {error}"))?;
    Ok(URL_SAFE_NO_PAD.encode(buffer))
}

fn page(text: &str) -> String {
    format!(
        "<!doctype html><meta charset=\"utf-8\"><title>OpenRouter Media</title>\
         <body style=\"font-family:system-ui;padding:3rem\"><h1>{text}</h1>"
    )
}

/// One value from a URL query, decoded.
fn query_value(query: &str, name: &str) -> Option<String> {
    let (_, value) = query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(key, _)| *key == name)?;
    let mut bytes = Vec::new();
    let mut rest = value.bytes();
    while let Some(byte) = rest.next() {
        match byte {
            b'+' => bytes.push(b' '),
            b'%' => {
                let hex = [rest.next()?, rest.next()?];
                bytes.push(u8::from_str_radix(std::str::from_utf8(&hex).ok()?, 16).ok()?);
            }
            other => bytes.push(other),
        }
    }
    String::from_utf8(bytes).ok()
}

/// The target of the request line, for example `/callback?code=abc`.
fn read_target(stream: &mut TcpStream) -> Option<String> {
    stream.set_nonblocking(false).ok()?;
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok()?;
    let mut head = Vec::new();
    let mut chunk = [0; 1024];
    while !head.windows(4).any(|window| window == b"\r\n\r\n") && head.len() < 16 * 1024 {
        match stream.read(&mut chunk).ok()? {
            0 => break,
            read => head.extend_from_slice(&chunk[..read]),
        }
    }
    let head = String::from_utf8_lossy(&head);
    let mut request_line = head.lines().next()?.split(' ');
    (request_line.next()? == "GET").then(|| request_line.next().map(String::from))?
}

/// Answers one browser request. Returns the code when the request is the real callback.
fn answer(mut stream: TcpStream, state: &str) -> Option<String> {
    let target = read_target(&mut stream)?;
    let (path, query) = target.split_once('?').unwrap_or((&target, ""));
    let code = query_value(query, "code")
        .filter(|_| query_value(query, "state").as_deref() == Some(state));
    let (status, body) = match (path, &code) {
        ("/callback", Some(_)) => ("200 OK", page("Signed in. You can close this tab.")),
        ("/callback", None) => ("400 Bad Request", page("Sign-in failed. Try again.")),
        _ => ("404 Not Found", String::new()),
    };
    let reply = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(reply.as_bytes());
    // A request that is not ours is answered and ignored, not fatal.
    code.filter(|_| path == "/callback")
}

/// What the code exchange needs.
pub struct Authorized {
    pub code: String,
    pub verifier: String,
}

/// OpenRouter's PKCE sign-in. The browser sends the code back to a server that
/// lives only for this call, so the key is never typed, pasted, or printed.
pub fn authorize(
    open_browser: impl FnOnce(&str),
    log: impl Fn(&str),
    timeout: Duration,
) -> Result<Authorized> {
    let verifier = random_token(32)?;
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let state = random_token(16)?;

    let failed = |error: std::io::Error| format!("Cannot start the sign-in server: {error}");
    let server = TcpListener::bind(("127.0.0.1", 0)).map_err(failed)?;
    server.set_nonblocking(true).map_err(failed)?;
    let callback_url = format!(
        "http://localhost:{}/callback",
        server.local_addr().map_err(failed)?.port()
    );
    let auth_url = format!(
        "https://openrouter.ai/auth?callback_url={}&code_challenge={challenge}\
         &code_challenge_method=S256&state={state}&key_label=openrouter-media",
        encode_component(&callback_url)
    );
    log(&format!(
        "Open this link to sign in to OpenRouter:\n{auth_url}"
    ));
    open_browser(&auth_url);

    let started = Instant::now();
    loop {
        match server.accept() {
            Ok((stream, _)) => {
                if let Some(code) = answer(stream, &state) {
                    return Ok(Authorized { code, verifier });
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                if started.elapsed() >= timeout {
                    return Err("Sign-in timed out".into());
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => return Err(failed(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stored_key_round_trips_is_private_and_can_be_deleted() {
        let (env, _dir) = Env::temp();
        assert_eq!(read_key(&env), None);
        let path = save_key("sk-stored", &env).unwrap();
        let found = Found {
            key: "sk-stored".into(),
            source: path.display().to_string(),
        };
        assert_eq!(read_key(&env), Some(found));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "{\n  \"key\": \"sk-stored\"\n}\n"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
            let folder = fs::metadata(path.parent().unwrap()).unwrap();
            assert_eq!(folder.permissions().mode() & 0o777, 0o700);
        }
        delete_key(&env).unwrap();
        delete_key(&env).unwrap();
        assert_eq!(read_key(&env), None);
    }

    #[test]
    fn the_environment_key_wins_over_the_stored_one() {
        let (env, _dir) = Env::temp();
        save_key("sk-stored", &env).unwrap();
        let found = read_key(&env.with("OPENROUTER_API_KEY", "sk-env"));
        assert_eq!(
            found,
            Some(Found {
                key: "sk-env".into(),
                source: "OPENROUTER_API_KEY".into()
            })
        );
    }

    #[test]
    fn a_key_file_that_cannot_be_used_is_the_same_as_not_signed_in() {
        let (env, _dir) = Env::temp();
        let path = save_key("sk-stored", &env).unwrap();
        for text in ["not json", "{}", "{ \"key\": \"\" }", "{ \"key\": 1 }"] {
            fs::write(&path, text).unwrap();
            assert_eq!(read_key(&env), None);
        }
    }

    #[test]
    fn the_config_sits_next_to_the_stored_key() {
        let (env, dir) = Env::temp();
        let folder = dir.path().join("openrouter-media");
        assert_eq!(
            credentials_path(&env).unwrap(),
            folder.join("credentials.json")
        );
        assert_eq!(
            crate::config::config_path(&env).unwrap(),
            folder.join("config.json")
        );
    }

    #[test]
    fn authorize_returns_the_code_from_the_callback_and_ignores_a_forged_one() {
        let get = |url: String| {
            let agent: ureq::Agent = ureq::Agent::config_builder()
                .http_status_as_error(false)
                .build()
                .into();
            agent.get(&url).call().unwrap().status().as_u16()
        };
        let browser = |auth_url: &str| {
            let (_, query) = auth_url.split_once('?').unwrap();
            assert_eq!(query_value(query, "code_challenge_method").unwrap(), "S256");
            assert_eq!(query_value(query, "code_challenge").unwrap().len(), 43);
            let callback = query_value(query, "callback_url")
                .unwrap()
                .replace("localhost", "127.0.0.1");
            let state = query_value(query, "state").unwrap();
            std::thread::spawn(move || {
                assert_eq!(get(format!("{callback}?code=evil&state=wrong")), 400);
                assert_eq!(get(callback.replace("/callback", "/other")), 404);
                assert_eq!(
                    get(format!("{callback}?code=re%61l+code&state={state}")),
                    200
                );
            });
        };
        let result = authorize(browser, |_| {}, Duration::from_secs(10)).unwrap();
        assert_eq!(result.code, "real code");
        assert!(result.verifier.len() >= 43);
    }

    #[test]
    fn authorize_times_out_when_nobody_signs_in() {
        let result = authorize(|_| {}, |_| {}, Duration::from_millis(20));
        assert_eq!(result.err().unwrap(), "Sign-in timed out");
    }
}
