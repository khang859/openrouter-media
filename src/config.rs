use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use serde_json::{Map, Value, json};

use crate::Result;
use crate::env::Env;

pub const IMAGE_MODEL: &str = "image-model";
pub const VIDEO_MODEL: &str = "video-model";
pub const OUT: &str = "out";
pub const OPEN: &str = "open";
pub const TIMEOUT: &str = "timeout";
pub const UPDATE_CHECK: &str = "update-check";

pub const DEFAULT_IMAGE_MODEL: &str = "google/gemini-nano-banana-2.1";
pub const DEFAULT_VIDEO_MODEL: &str = "x-ai/grok-imagine-video-1.5-lite";
pub const DEFAULT_TIMEOUT_MINUTES: u64 = 20;

/// The `source` of a setting that nothing has set.
pub const DEFAULT_SOURCE: &str = "default";

/// How a setting is read from the command line, and what a stored one must be.
#[derive(Clone, Copy)]
enum Kind {
    Text,
    Boolean,
    Count,
}

impl Kind {
    fn hint(self) -> &'static str {
        match self {
            Kind::Text => "some text",
            Kind::Boolean => "true or false",
            Kind::Count => "a whole number",
        }
    }

    fn parse(self, text: &str) -> Option<Value> {
        match self {
            Kind::Text => (!text.is_empty()).then(|| json!(text)),
            Kind::Boolean => text.parse::<bool>().ok().map(Value::Bool),
            Kind::Count => {
                let digits = !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
                text.parse::<u64>().ok().filter(|_| digits).map(Value::from)
            }
        }
    }

    fn valid(self, value: &Value) -> bool {
        match self {
            Kind::Text => value.as_str().is_some_and(|text| !text.is_empty()),
            Kind::Boolean => value.is_boolean(),
            Kind::Count => value.is_u64(),
        }
    }
}

pub struct Setting {
    pub name: &'static str,
    kind: Kind,
    /// A variable that wins over the stored value.
    pub env: Option<&'static str>,
    /// What applies when nothing is set.
    fallback: fn() -> Option<Value>,
    pub about: &'static str,
}

/// Every setting `ormedia config` knows.
pub const SETTINGS: [Setting; 6] = [
    Setting {
        name: IMAGE_MODEL,
        kind: Kind::Text,
        env: Some("ORMEDIA_IMAGE_MODEL"),
        fallback: || Some(json!(DEFAULT_IMAGE_MODEL)),
        about: "Model for `ormedia image`",
    },
    Setting {
        name: VIDEO_MODEL,
        kind: Kind::Text,
        env: Some("ORMEDIA_VIDEO_MODEL"),
        fallback: || Some(json!(DEFAULT_VIDEO_MODEL)),
        about: "Model for `ormedia video`",
    },
    Setting {
        name: OUT,
        kind: Kind::Text,
        env: None,
        fallback: || None,
        about: "Folder that results are saved to",
    },
    Setting {
        name: OPEN,
        kind: Kind::Boolean,
        env: None,
        fallback: || Some(json!(true)),
        about: "Open a new image in Fleet",
    },
    Setting {
        name: TIMEOUT,
        kind: Kind::Count,
        env: None,
        fallback: || Some(json!(DEFAULT_TIMEOUT_MINUTES)),
        about: "Minutes to wait for a video",
    },
    Setting {
        name: UPDATE_CHECK,
        kind: Kind::Boolean,
        env: Some("ORMEDIA_UPDATE_CHECK"),
        fallback: || Some(json!(true)),
        about: "Look for a new version, one time per day",
    },
];

pub fn setting(name: &str) -> Result<&'static Setting> {
    SETTINGS
        .iter()
        .find(|setting| setting.name == name)
        .ok_or_else(|| {
            let names: Vec<_> = SETTINGS.iter().map(|setting| setting.name).collect();
            format!(
                "Unknown setting \"{name}\". The settings are: {}",
                names.join(", ")
            )
        })
}

pub fn config_path(env: &Env) -> Result<PathBuf> {
    Ok(env.config_dir()?.join("config.json"))
}

/// What the file holds. No file is the same as an empty one, but a file that
/// cannot be used is an error: ignoring it would silently drop the user's settings.
pub fn read_config(env: &Env) -> Result<Map<String, Value>> {
    let path = config_path(env)?;
    let shown = path.display();
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Map::new()),
        Err(error) => return Err(format!("Cannot read {shown}: {error}")),
    };
    let stored: Value = serde_json::from_str(&text)
        .map_err(|error| format!("{shown} is not valid JSON ({error}). Fix it or delete it."))?;
    let Value::Object(stored) = stored else {
        return Err(format!(
            "{shown} must hold a JSON object. Fix it or delete it."
        ));
    };
    for Setting { name, kind, .. } in &SETTINGS {
        if stored.get(*name).is_some_and(|value| !kind.valid(value)) {
            let hint = kind.hint();
            return Err(format!(
                "\"{name}\" in {shown} must be {hint}. Fix it or delete it."
            ));
        }
    }
    Ok(stored)
}

/// Through a temp file, so a crash mid-write cannot leave half a config.
fn write_config(stored: &Map<String, Value>, env: &Env) -> Result<PathBuf> {
    let path = config_path(env)?;
    let failed = |error: std::io::Error| format!("Cannot write {}: {error}", path.display());
    crate::files::create_private_dir(&env.config_dir()?).map_err(failed)?;
    let temp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    let text = format!("{:#}\n", Value::Object(stored.clone()));
    fs::write(&temp, text)
        .and_then(|()| fs::rename(&temp, &path))
        .map_err(|error| {
            let _ = fs::remove_file(&temp);
            failed(error)
        })?;
    Ok(path)
}

/// Stores `text` as given on the command line. Returns the file it went to.
pub fn set_setting(name: &str, text: &str, env: &Env) -> Result<PathBuf> {
    let Setting { kind, .. } = setting(name)?;
    let value = kind
        .parse(text)
        .ok_or_else(|| format!("{name} must be {}, got \"{text}\"", kind.hint()))?;
    let mut stored = read_config(env)?;
    stored.insert(name.to_string(), value);
    write_config(&stored, env)
}

pub fn unset_setting(name: &str, env: &Env) -> Result<PathBuf> {
    setting(name)?;
    let mut stored = read_config(env)?;
    stored.shift_remove(name);
    write_config(&stored, env)
}

pub struct Resolved {
    pub name: &'static str,
    pub value: Option<Value>,
    /// The variable, the config file, or `default`.
    pub source: String,
}

pub struct Settings(Vec<Resolved>);

/// Each setting's value and where it came from: the environment variable, then
/// the config file, then the built-in default. A flag on the command line wins
/// over all of these, and that is the caller's job.
pub fn resolve_settings(env: &Env) -> Result<Settings> {
    let stored = read_config(env)?;
    let path = config_path(env)?.display().to_string();
    let resolve = |setting: &'static Setting| {
        let from_env = setting
            .env
            .and_then(|variable| Some((variable, env.get(variable)?)));
        let (value, source) = match (from_env, stored.get(setting.name)) {
            (Some((variable, text)), _) => {
                let hint = setting.kind.hint();
                let value = setting.kind.parse(text);
                let value =
                    value.ok_or_else(|| format!("${variable} must be {hint}, got \"{text}\""))?;
                (Some(value), variable.to_string())
            }
            (None, Some(value)) => (Some(value.clone()), path.clone()),
            (None, None) => ((setting.fallback)(), DEFAULT_SOURCE.to_string()),
        };
        Ok(Resolved {
            name: setting.name,
            value,
            source,
        })
    };
    Ok(Settings(
        SETTINGS.iter().map(resolve).collect::<Result<_>>()?,
    ))
}

impl Settings {
    pub fn iter(&self) -> impl Iterator<Item = &Resolved> {
        self.0.iter()
    }

    fn value(&self, name: &str) -> Option<&Value> {
        self.iter()
            .find(|resolved| resolved.name == name)?
            .value
            .as_ref()
    }

    pub fn text(&self, name: &str) -> Option<String> {
        self.value(name)?.as_str().map(String::from)
    }

    pub fn flag(&self, name: &str) -> Option<bool> {
        self.value(name)?.as_bool()
    }

    pub fn count(&self, name: &str) -> Option<u64> {
        self.value(name)?.as_u64()
    }

    /// For `--json`. A setting with no value says so with `null`.
    pub fn to_json(&self) -> Value {
        let entry = |resolved: &Resolved| {
            let facts = json!({ "value": resolved.value, "source": resolved.source });
            (resolved.name.to_string(), facts)
        };
        Value::Object(self.iter().map(entry).collect())
    }
}

#[cfg(test)]
impl Settings {
    pub fn of(values: &[(&'static str, Value)]) -> Self {
        let resolved = |(name, value): &(&'static str, Value)| Resolved {
            name,
            value: (!value.is_null()).then(|| value.clone()),
            source: "test".into(),
        };
        Self(values.iter().map(resolved).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_with_file(text: &str) -> (Env, tempfile::TempDir) {
        let (env, dir) = Env::temp();
        fs::create_dir_all(env.config_dir().unwrap()).unwrap();
        fs::write(config_path(&env).unwrap(), text).unwrap();
        (env, dir)
    }

    fn resolved(settings: &Settings, name: &str) -> (Option<Value>, String) {
        let found = settings
            .iter()
            .find(|resolved| resolved.name == name)
            .unwrap();
        (found.value.clone(), found.source.clone())
    }

    fn error<T>(result: Result<T>) -> String {
        result.err().expect("an error")
    }

    #[test]
    fn with_no_file_every_setting_is_its_default() {
        let (env, _dir) = Env::temp();
        let settings = resolve_settings(&env).unwrap();
        let names: Vec<_> = settings.iter().map(|resolved| resolved.name).collect();
        assert_eq!(
            names,
            [IMAGE_MODEL, VIDEO_MODEL, OUT, OPEN, TIMEOUT, UPDATE_CHECK]
        );
        assert!(
            settings
                .iter()
                .all(|resolved| resolved.source == DEFAULT_SOURCE)
        );
        assert_eq!(settings.text(IMAGE_MODEL).unwrap(), DEFAULT_IMAGE_MODEL);
        assert_eq!(settings.text(VIDEO_MODEL).unwrap(), DEFAULT_VIDEO_MODEL);
        assert_eq!(settings.text(OUT), None);
        assert_eq!(settings.flag(OPEN), Some(true));
        assert_eq!(settings.count(TIMEOUT), Some(20));
        assert_eq!(settings.flag(UPDATE_CHECK), Some(true));
    }

    #[test]
    fn an_environment_variable_is_read_as_the_type_of_its_setting() {
        let (env, _dir) = Env::temp();
        set_setting(UPDATE_CHECK, "true", &env).unwrap();
        let env = env.with("ORMEDIA_UPDATE_CHECK", "false");
        let settings = resolve_settings(&env).unwrap();
        let found = (Some(json!(false)), "ORMEDIA_UPDATE_CHECK".to_string());
        assert_eq!(resolved(&settings, UPDATE_CHECK), found);

        let env = env.with("ORMEDIA_UPDATE_CHECK", "0");
        let wrong = error(resolve_settings(&env).map(|_| ()));
        assert_eq!(
            wrong,
            "$ORMEDIA_UPDATE_CHECK must be true or false, got \"0\""
        );
    }

    #[test]
    fn a_saved_setting_round_trips_with_its_type_and_can_be_removed() {
        let (env, _dir) = Env::temp();
        let path = set_setting(VIDEO_MODEL, "acme/video-1", &env).unwrap();
        set_setting(OPEN, "false", &env).unwrap();
        set_setting(TIMEOUT, "45", &env).unwrap();
        assert_eq!(path, config_path(&env).unwrap());
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "{\n  \"video-model\": \"acme/video-1\",\n  \"open\": false,\n  \"timeout\": 45\n}\n"
        );

        let settings = resolve_settings(&env).unwrap();
        let file = path.display().to_string();
        assert_eq!(
            resolved(&settings, VIDEO_MODEL),
            (Some(json!("acme/video-1")), file.clone())
        );
        assert_eq!(
            resolved(&settings, OPEN),
            (Some(json!(false)), file.clone())
        );
        assert_eq!(resolved(&settings, TIMEOUT), (Some(json!(45)), file));

        unset_setting(OPEN, &env).unwrap();
        unset_setting(OUT, &env).unwrap();
        let left = json!({ "video-model": "acme/video-1", "timeout": 45 });
        assert_eq!(Value::Object(read_config(&env).unwrap()), left);
        let files: Vec<_> = fs::read_dir(path.parent().unwrap()).unwrap().collect();
        assert_eq!(files.len(), 1, "no temp file is left behind");
    }

    #[test]
    fn the_environment_model_wins_over_the_saved_one_but_an_empty_variable_does_not() {
        let (env, _dir) = Env::temp();
        let file = set_setting(IMAGE_MODEL, "acme/saved", &env)
            .unwrap()
            .display()
            .to_string();
        let settings = resolve_settings(&env.with("ORMEDIA_IMAGE_MODEL", "acme/env")).unwrap();
        assert_eq!(
            resolved(&settings, IMAGE_MODEL),
            (Some(json!("acme/env")), "ORMEDIA_IMAGE_MODEL".to_string())
        );

        let (env, _dir) = Env::temp();
        let file_2 = set_setting(IMAGE_MODEL, "acme/saved", &env)
            .unwrap()
            .display()
            .to_string();
        assert_ne!(file, file_2);
        let settings = resolve_settings(&env.with("ORMEDIA_IMAGE_MODEL", "")).unwrap();
        assert_eq!(
            resolved(&settings, IMAGE_MODEL),
            (Some(json!("acme/saved")), file_2)
        );
    }

    #[test]
    fn an_unknown_setting_or_a_value_of_the_wrong_type_is_refused_and_nothing_is_written() {
        let (env, _dir) = Env::temp();
        let unknown = error(set_setting("colour", "red", &env));
        assert!(
            unknown.starts_with("Unknown setting \"colour\"") && unknown.contains("image-model")
        );
        assert!(error(unset_setting("colour", &env)).starts_with("Unknown setting"));
        assert_eq!(
            error(set_setting(OPEN, "yes", &env)),
            "open must be true or false, got \"yes\""
        );
        for bad in ["1.5", "-1", "+1", "", "99999999999999999999999"] {
            assert!(
                error(set_setting(TIMEOUT, bad, &env))
                    .starts_with("timeout must be a whole number")
            );
        }
        assert!(error(set_setting(OUT, "", &env)).starts_with("out must be some text"));
        assert!(!config_path(&env).unwrap().exists());
    }

    #[test]
    fn a_file_that_cannot_be_used_is_an_error_not_an_empty_config() {
        let broken = |text| error(resolve_settings(&env_with_file(text).0).map(|_| ()));
        assert!(broken("{ \"open\": ").contains("is not valid JSON"));
        assert!(broken("[]").contains("must hold a JSON object"));
        assert!(broken("{ \"timeout\": \"soon\" }").contains("must be a whole number"));
        assert!(broken("{ \"timeout\": 1.5 }").contains("\"timeout\" in "));

        let (env, _dir) = env_with_file("not json");
        assert!(error(set_setting(OPEN, "false", &env)).contains("is not valid JSON"));
        assert_eq!(
            fs::read_to_string(config_path(&env).unwrap()).unwrap(),
            "not json"
        );
    }

    #[test]
    fn a_setting_this_version_does_not_know_survives_a_save() {
        let (env, _dir) = env_with_file("{ \"from-the-future\": 1 }");
        set_setting(OUT, "media", &env).unwrap();
        let kept = json!({ "from-the-future": 1, "out": "media" });
        assert_eq!(Value::Object(read_config(&env).unwrap()), kept);
    }

    #[test]
    fn json_output_says_null_for_a_setting_with_no_value() {
        let (env, _dir) = Env::temp();
        let shown = resolve_settings(&env).unwrap().to_json();
        assert_eq!(shown[OUT], json!({ "value": null, "source": "default" }));
        assert_eq!(shown[OPEN], json!({ "value": true, "source": "default" }));
    }
}
