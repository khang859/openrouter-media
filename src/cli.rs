use std::io::Write;
use std::process::ExitCode;
use std::time::{Duration, SystemTime};

use clap::{Args, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use serde_json::{Map, Value, json};

use crate::Result;
use crate::api::{Client, Https, ImageOptions, VideoOptions, image_body, status_of, video_body};
use crate::auth::{authorize, delete_key, read_key, save_key};
use crate::config::{
    self, DEFAULT_SOURCE, SETTINGS, Settings, resolve_settings, set_setting, unset_setting,
};
use crate::env::Env;
use crate::files::{Output, open_in_fleet, open_url, save_outputs, slug, timestamp, to_image_url};
use crate::models::{describe_model, plain};
use crate::update;

const SIGN_IN_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const POLL_INTERVAL: Duration = Duration::from_secs(10);

fn help() -> String {
    let settings: Vec<String> = SETTINGS
        .iter()
        .map(|setting| format!("  {:<24}{}", setting.name, setting.about))
        .collect();
    format!(
        "ormedia - generate images and videos with OpenRouter

Usage
  ormedia login                      Sign in with the browser and store an API key
  ormedia logout                     Delete the stored API key
  ormedia status                     Show which key is in use and its spend
  ormedia models <image|video>       List models and the options each one takes
  ormedia image --prompt <text>      Generate or edit an image
  ormedia video --prompt <text>      Generate a video
  ormedia job <job-id>               Check a video job, and download it when done
  ormedia config                     Show the settings and where each one comes from
  ormedia config set <name> <value>  Save a setting
  ormedia config unset <name>        Remove a saved setting

Image options
  --model <id>            Default: {image_model}
  --image <path|url>      Reference image to edit or follow. Repeat for more.
  --aspect-ratio <r>      For example 1:1, 16:9, 9:16
  --resolution <r>        For example 1K, 2K, 4K
  --size <WxH>            Exact pixels, for models that take them
  --quality <q>           auto, low, medium, high
  --format <f>            png, jpeg, webp, svg
  --background <b>        auto, transparent, opaque
  --seed <n>              Same seed, same picture, where the model allows it
  --n <count>             How many images

Video options
  --model <id>            Default: {video_model}
  --first-frame <path|url>  Image the video starts on
  --last-frame <path|url>   Image the video ends on
  --image <path|url>      Reference image. Repeat for more.
  --duration <seconds>
  --aspect-ratio <r>      For example 16:9, 9:16
  --resolution <r>        For example 720p, 1080p
  --audio | --no-audio    Ask for sound, or for none
  --seed <n>
  --no-wait               Print the job id and return at once
  --timeout <minutes>     How long to wait for the video. Default: {timeout}

Shared options
  --out <path>            A file name, or a folder. Default: the current folder.
  --search <text>         models: keep only models that match
  --wait                  job: wait until the video is done
  --open | --no-open      Open the result in Fleet, or do not
  --json                  Print the result as JSON

Settings
{settings}
  A flag wins over $ORMEDIA_IMAGE_MODEL, $ORMEDIA_VIDEO_MODEL, and
  $ORMEDIA_UPDATE_CHECK, which win over a saved setting, which wins over the default.

The API key comes from $OPENROUTER_API_KEY, or from `ormedia login`.
",
        image_model = config::DEFAULT_IMAGE_MODEL,
        video_model = config::DEFAULT_VIDEO_MODEL,
        timeout = config::DEFAULT_TIMEOUT_MINUTES,
        settings = settings.join("\n"),
    )
}

#[derive(Parser)]
#[command(
    name = "ormedia",
    version,
    about = "Generate images and videos with OpenRouter"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Sign in with the browser and store an API key
    Login,
    /// Delete the stored API key
    Logout,
    /// Show which key is in use and its spend
    Status {
        /// Print the result as JSON
        #[arg(long)]
        json: bool,
    },
    /// List models and the options each one takes
    Models {
        kind: Kind,
        /// Keep only models that match
        #[arg(long, value_name = "text")]
        search: Option<String>,
        /// Print the result as JSON
        #[arg(long)]
        json: bool,
    },
    /// Generate or edit an image
    Image(ImageArgs),
    /// Generate a video
    Video(VideoArgs),
    /// Check a video job, and download it when done
    Job(JobArgs),
    /// Show the settings, or save one with `set` and `unset`
    Config(ConfigArgs),
}

#[derive(Clone, Copy, ValueEnum)]
enum Kind {
    Image,
    Video,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Image => "image",
            Kind::Video => "video",
        }
    }
}

#[derive(Args, Default)]
struct OutputArgs {
    /// A file name, or a folder. Default: the current folder, or the `out` setting.
    #[arg(long, value_name = "path")]
    out: Option<String>,
    /// Open the result in Fleet
    #[arg(long)]
    open: bool,
    /// Do not open the result in Fleet
    #[arg(long)]
    no_open: bool,
    /// Print the result as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct ImageArgs {
    /// The prompt, when it is not given with --prompt
    words: Vec<String>,
    #[arg(long, value_name = "text")]
    prompt: Option<String>,
    /// Default: the `image-model` setting
    #[arg(long, value_name = "id")]
    model: Option<String>,
    /// Reference image to edit or follow. Repeat for more.
    #[arg(long, value_name = "path|url")]
    image: Vec<String>,
    /// For example 1:1, 16:9, 9:16
    #[arg(long, value_name = "r")]
    aspect_ratio: Option<String>,
    /// For example 1K, 2K, 4K
    #[arg(long, value_name = "r")]
    resolution: Option<String>,
    /// Exact pixels, for models that take them
    #[arg(long, value_name = "WxH")]
    size: Option<String>,
    /// auto, low, medium, high
    #[arg(long, value_name = "q")]
    quality: Option<String>,
    /// png, jpeg, webp, svg
    #[arg(long, value_name = "f")]
    format: Option<String>,
    /// auto, transparent, opaque
    #[arg(long, value_name = "b")]
    background: Option<String>,
    /// Same seed, same picture, where the model allows it
    #[arg(long, value_name = "n")]
    seed: Option<u64>,
    /// How many images
    #[arg(long, value_name = "count")]
    n: Option<u64>,
    #[command(flatten)]
    output: OutputArgs,
}

#[derive(Args)]
struct VideoArgs {
    /// The prompt, when it is not given with --prompt
    words: Vec<String>,
    #[arg(long, value_name = "text")]
    prompt: Option<String>,
    /// Default: the `video-model` setting
    #[arg(long, value_name = "id")]
    model: Option<String>,
    /// Image the video starts on
    #[arg(long, value_name = "path|url")]
    first_frame: Option<String>,
    /// Image the video ends on
    #[arg(long, value_name = "path|url")]
    last_frame: Option<String>,
    /// Reference image. Repeat for more.
    #[arg(long, value_name = "path|url")]
    image: Vec<String>,
    #[arg(long, value_name = "seconds")]
    duration: Option<u64>,
    /// For example 16:9, 9:16
    #[arg(long, value_name = "r")]
    aspect_ratio: Option<String>,
    /// For example 720p, 1080p
    #[arg(long, value_name = "r")]
    resolution: Option<String>,
    /// Ask for sound
    #[arg(long)]
    audio: bool,
    /// Ask for no sound
    #[arg(long)]
    no_audio: bool,
    #[arg(long, value_name = "n")]
    seed: Option<u64>,
    /// Print the job id and return at once
    #[arg(long)]
    no_wait: bool,
    /// How long to wait for the video. Default: the `timeout` setting
    #[arg(long, value_name = "minutes")]
    timeout: Option<u64>,
    #[command(flatten)]
    output: OutputArgs,
}

#[derive(Args)]
struct JobArgs {
    id: String,
    /// Wait until the video is done
    #[arg(long)]
    wait: bool,
    /// How long to wait for the video. Default: the `timeout` setting
    #[arg(long, value_name = "minutes")]
    timeout: Option<u64>,
    #[command(flatten)]
    output: OutputArgs,
}

#[derive(Args)]
struct ConfigArgs {
    #[command(subcommand)]
    action: Option<ConfigAction>,
    /// Print the settings as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Save a setting
    Set { name: String, value: String },
    /// Remove a saved setting
    Unset { name: String },
}

/// A result line, to stdout. A reader that has gone away, as in `| head`, ends the command quietly.
fn say(text: impl std::fmt::Display) {
    if writeln!(std::io::stdout(), "{text}").is_err() {
        std::process::exit(0);
    }
}

/// Progress goes to stderr, so `--json` output stays clean.
fn progress(text: impl std::fmt::Display) {
    eprintln!("{text}");
}

fn pretty(value: &Value) -> String {
    format!("{value:#}")
}

fn signed_in_client(env: &Env) -> Result<Client<Https>> {
    let found =
        read_key(env).ok_or("No API key. Run `ormedia login`, or set OPENROUTER_API_KEY.")?;
    Ok(Client::https(Some(found.key)))
}

/// What a command runs with: a flag on the command line wins over its setting.
struct Chosen {
    out: Option<String>,
    open: bool,
    timeout: Duration,
}

fn choose(output: &OutputArgs, timeout: Option<u64>, settings: &Settings) -> Chosen {
    let minutes = timeout
        .or(settings.count(config::TIMEOUT))
        .unwrap_or(config::DEFAULT_TIMEOUT_MINUTES);
    Chosen {
        out: output.out.clone().or_else(|| settings.text(config::OUT)),
        open: output.open || (!output.no_open && settings.flag(config::OPEN).unwrap_or(true)),
        timeout: Duration::from_secs(minutes.saturating_mul(60)),
    }
}

fn choose_model(flag: Option<String>, kind: Kind, settings: &Settings) -> String {
    let (name, fallback) = match kind {
        Kind::Image => (config::IMAGE_MODEL, config::DEFAULT_IMAGE_MODEL),
        Kind::Video => (config::VIDEO_MODEL, config::DEFAULT_VIDEO_MODEL),
    };
    flag.or_else(|| settings.text(name))
        .unwrap_or_else(|| fallback.to_string())
}

/// Saves the outputs, shows them, and reports. `facts` are the result fields after `files`.
fn finish(
    outputs: &[Output],
    stem: &str,
    chosen: &Chosen,
    json: bool,
    facts: Vec<(&str, Value)>,
    env: &Env,
) -> Result<()> {
    let cwd = std::env::current_dir()
        .map_err(|error| format!("Cannot find the current folder: {error}"))?;
    let files = save_outputs(outputs, chosen.out.as_deref(), stem, &cwd)?;
    if chosen.open {
        open_in_fleet(&files, env);
    }
    let files: Vec<String> = files
        .iter()
        .map(|file| file.display().to_string())
        .collect();
    if json {
        let mut result = Map::new();
        result.insert("files".to_string(), json!(files));
        result.extend(
            facts
                .into_iter()
                .map(|(name, value)| (name.to_string(), value)),
        );
        say(pretty(&Value::Object(result)));
        return Ok(());
    }
    for file in files {
        say(format!("Saved {file}"));
    }
    if let Some((_, cost)) = facts
        .iter()
        .find(|(name, cost)| *name == "costUsd" && !cost.is_null())
    {
        say(format!("Cost ${}", plain(cost)));
    }
    Ok(())
}

fn login(env: &Env) -> Result<()> {
    let authorized = authorize(open_url, |text| progress(text), SIGN_IN_TIMEOUT)?;
    let key = Client::https(None).exchange_code(&authorized.code, &authorized.verifier)?;
    say(format!(
        "Signed in. Key stored in {}",
        save_key(&key, env)?.display()
    ));
    Ok(())
}

fn logout(env: &Env) -> Result<()> {
    delete_key(env)?;
    say("Stored key deleted.");
    Ok(())
}

fn status(json: bool, env: &Env) -> Result<ExitCode> {
    let Some(found) = read_key(env) else {
        say(if json {
            json!({ "signedIn": false }).to_string()
        } else {
            "Not signed in. Run `ormedia login`.".into()
        });
        return Ok(ExitCode::FAILURE);
    };
    let info = Client::https(Some(found.key)).key_info()?;
    if json {
        let mut result = Map::new();
        result.insert("signedIn".to_string(), json!(true));
        result.insert("source".to_string(), json!(found.source));
        result.extend(info.as_object().cloned().unwrap_or_default());
        say(pretty(&Value::Object(result)));
        return Ok(ExitCode::SUCCESS);
    }
    say(format!("Signed in. Key from {}", found.source));
    if info["label"]
        .as_str()
        .is_some_and(|label| !label.is_empty())
    {
        say(format!("Label: {}", plain(&info["label"])));
    }
    for (field, title) in [("usage", "Spent"), ("limit_remaining", "Left on this key")] {
        if !info[field].is_null() {
            say(format!("{title}: ${}", plain(&info[field])));
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn models(kind: Kind, search: Option<String>, json: bool) -> Result<()> {
    let term = search.map(|term| term.to_lowercase());
    let matches = |model: &Value| {
        let text = format!("{} {}", plain(&model["id"]), plain(&model["name"])).to_lowercase();
        term.as_ref().is_none_or(|term| text.contains(term))
    };
    let list: Vec<Value> = Client::https(None)
        .list_models(kind.name())?
        .into_iter()
        .filter(matches)
        .collect();
    if json {
        say(pretty(&Value::Array(list)));
    } else {
        let video = matches!(kind, Kind::Video);
        say(list
            .iter()
            .map(|model| describe_model(video, model))
            .collect::<Vec<_>>()
            .join("\n\n"));
    }
    Ok(())
}

fn image_urls(references: &[String]) -> Result<Vec<String>> {
    references
        .iter()
        .map(|reference| to_image_url(reference))
        .collect()
}

fn image(args: ImageArgs, env: &Env) -> Result<()> {
    let prompt = args.prompt.unwrap_or_else(|| args.words.join(" "));
    if prompt.is_empty() {
        return Err("An image needs a --prompt".into());
    }
    let settings = resolve_settings(env)?;
    let chosen = choose(&args.output, None, &settings);
    let model = choose_model(args.model, Kind::Image, &settings);
    let body = image_body(ImageOptions {
        model: model.clone(),
        prompt: prompt.clone(),
        references: image_urls(&args.image)?,
        aspect_ratio: args.aspect_ratio,
        resolution: args.resolution,
        size: args.size,
        quality: args.quality,
        format: args.format,
        background: args.background,
        seed: args.seed,
        n: args.n,
    });

    let client = signed_in_client(env)?;
    progress(format!("Generating with {model}..."));
    let result = client.generate_images(&body)?;
    let stem = format!("{}-{}", slug(&prompt), timestamp(SystemTime::now()));
    let facts = vec![("model", json!(model)), ("costUsd", result.cost_usd)];
    finish(&result.images, &stem, &chosen, args.output.json, facts, env)
}

fn download(client: &Client<Https>, job: &Value, id: &str) -> Result<Vec<Output>> {
    let count = job["unsigned_urls"].as_array().map_or(0, Vec::len).max(1);
    (0..count)
        .map(|index| client.download_video(id, index))
        .collect()
}

/// Prints a status once, when it changes, so a long wait is not a wall of lines.
fn status_printer() -> impl FnMut(&str) {
    let mut seen = String::new();
    move |status| {
        if status != seen {
            seen = status.to_string();
            progress(format!("Video is {status}..."));
        }
    }
}

fn video(args: VideoArgs, env: &Env) -> Result<()> {
    let prompt = args.prompt.unwrap_or_else(|| args.words.join(" "));
    let first_frame = args.first_frame.as_deref().map(to_image_url).transpose()?;
    let last_frame = args.last_frame.as_deref().map(to_image_url).transpose()?;
    if prompt.is_empty() && first_frame.is_none() && last_frame.is_none() {
        return Err("A video needs a --prompt or a frame image".into());
    }
    let settings = resolve_settings(env)?;
    let chosen = choose(&args.output, args.timeout, &settings);
    let model = choose_model(args.model, Kind::Video, &settings);
    let body = video_body(VideoOptions {
        model: model.clone(),
        prompt: (!prompt.is_empty()).then(|| prompt.clone()),
        first_frame,
        last_frame,
        references: image_urls(&args.image)?,
        duration: args.duration,
        aspect_ratio: args.aspect_ratio,
        resolution: args.resolution,
        audio: if args.audio {
            Some(true)
        } else {
            args.no_audio.then_some(false)
        },
        seed: args.seed,
    });

    let client = signed_in_client(env)?;
    let submitted = client.submit_video(&body)?;
    let id = submitted["id"]
        .as_str()
        .ok_or("OpenRouter returned no job id")?;
    // Said before the wait, so the id survives a killed or timed-out command.
    progress(format!("Video job {id} started with {model}."));
    if args.no_wait {
        say(if args.output.json {
            json!({ "jobId": id, "status": submitted["status"] }).to_string()
        } else {
            format!("Check it with: ormedia job {id} --wait")
        });
        return Ok(());
    }
    let job = client.wait_for_video(id, chosen.timeout, POLL_INTERVAL, status_printer())?;
    let stem = format!(
        "{}-{}",
        slug(if prompt.is_empty() { "video" } else { &prompt }),
        timestamp(SystemTime::now())
    );
    let facts = vec![
        ("model", json!(model)),
        ("jobId", json!(id)),
        ("costUsd", job["usage"]["cost"].clone()),
    ];
    finish(
        &download(&client, &job, id)?,
        &stem,
        &chosen,
        args.output.json,
        facts,
        env,
    )
}

fn job(args: JobArgs, env: &Env) -> Result<()> {
    let id = args.id.as_str();
    let chosen = choose(&args.output, args.timeout, &resolve_settings(env)?);
    let client = signed_in_client(env)?;
    let found = if args.wait {
        client.wait_for_video(id, chosen.timeout, POLL_INTERVAL, status_printer())?
    } else {
        client.get_video_job(id)?
    };
    let status = status_of(&found);
    if status != "completed" {
        say(if args.output.json {
            json!({ "jobId": id, "status": status }).to_string()
        } else {
            format!("Video job {id} is {status}.")
        });
        return Ok(());
    }
    let facts = vec![
        ("jobId", json!(id)),
        ("costUsd", found["usage"]["cost"].clone()),
    ];
    finish(
        &download(&client, &found, id)?,
        &format!("video-{id}"),
        &chosen,
        args.output.json,
        facts,
        env,
    )
}

fn show_config(json: bool, env: &Env) -> Result<()> {
    let settings = resolve_settings(env)?;
    let path = config::config_path(env)?.display().to_string();
    if json {
        say(pretty(
            &json!({ "path": path, "settings": settings.to_json() }),
        ));
        return Ok(());
    }
    for resolved in settings.iter() {
        let value = resolved.value.as_ref().map_or_else(
            || "not set".to_string(),
            |value| {
                let source = resolved.source.as_str();
                let from = if source == path {
                    "saved".to_string()
                } else if source == DEFAULT_SOURCE {
                    source.to_string()
                } else {
                    format!("${source}")
                };
                format!("{}  ({from})", plain(value))
            },
        );
        say(format!("{:<13}{value}", resolved.name));
    }
    say(format!("\nSaved settings are in {path}"));
    Ok(())
}

fn configure(args: ConfigArgs, env: &Env) -> Result<()> {
    match args.action {
        None => show_config(args.json, env),
        Some(ConfigAction::Set { name, value }) => {
            say(format!(
                "Saved {name} in {}",
                set_setting(&name, &value, env)?.display()
            ));
            if let Some(variable) = config::setting(&name)?
                .env
                .filter(|variable| env.get(variable).is_some())
            {
                progress(format!(
                    "${variable} is set, and it wins over the saved {name}."
                ));
            }
            Ok(())
        }
        Some(ConfigAction::Unset { name }) => {
            say(format!(
                "Removed {name} from {}",
                unset_setting(&name, env)?.display()
            ));
            Ok(())
        }
    }
}

fn run(command: Command, env: &Env) -> Result<ExitCode> {
    match command {
        Command::Login => login(env)?,
        Command::Logout => logout(env)?,
        Command::Status { json } => return status(json, env),
        Command::Models { kind, search, json } => models(kind, search, json)?,
        Command::Image(args) => image(args, env)?,
        Command::Video(args) => video(args, env)?,
        Command::Job(args) => job(args, env)?,
        Command::Config(args) => configure(args, env)?,
    }
    Ok(ExitCode::SUCCESS)
}

pub fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() <= 1 {
        say(help().trim_end());
        return ExitCode::SUCCESS;
    }
    // One page that lists every option. `ormedia <command> --help` has the detail.
    let matches = Cli::command().override_help(help()).get_matches_from(args);
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    let env = Env::real();
    let code = run(cli.command, &env).unwrap_or_else(|message| {
        eprintln!("ormedia: {message}");
        ExitCode::FAILURE
    });
    announce_update(&env);
    code
}

/// After the command, so its own output comes first. Settings that cannot be read mean no check.
fn announce_update(env: &Env) {
    let wanted = resolve_settings(env)
        .is_ok_and(|settings| settings.flag(config::UPDATE_CHECK) != Some(false));
    if wanted && let Some(notice) = update::notice(env, SystemTime::now(), update::latest_release) {
        progress(notice);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(out: Value, open: bool, timeout: u64) -> Settings {
        Settings::of(&[
            (config::IMAGE_MODEL, json!("saved/image")),
            (config::VIDEO_MODEL, json!("saved/video")),
            (config::OUT, out),
            (config::OPEN, json!(open)),
            (config::TIMEOUT, json!(timeout)),
        ])
    }

    #[test]
    fn the_command_line_is_well_formed() {
        Cli::command().debug_assert();
    }

    #[test]
    fn with_no_flags_a_command_runs_with_the_settings() {
        let saved = settings(json!("media"), false, 5);
        let chosen = choose(&OutputArgs::default(), None, &saved);
        assert_eq!(chosen.out.as_deref(), Some("media"));
        assert!(!chosen.open);
        assert_eq!(chosen.timeout, Duration::from_secs(300));
        assert_eq!(choose_model(None, Kind::Image, &saved), "saved/image");
        assert_eq!(choose_model(None, Kind::Video, &saved), "saved/video");

        let unset = choose(
            &OutputArgs::default(),
            None,
            &settings(Value::Null, true, 20),
        );
        assert_eq!(unset.out, None);
        assert!(unset.open);
    }

    #[test]
    fn a_flag_wins_over_its_setting() {
        let flags = OutputArgs {
            out: Some("here".into()),
            ..Default::default()
        };
        let chosen = choose(&flags, Some(0), &settings(json!("media"), true, 5));
        assert_eq!(chosen.out.as_deref(), Some("here"));
        assert_eq!(chosen.timeout, Duration::ZERO);
        assert_eq!(
            choose_model(
                Some("flag/model".into()),
                Kind::Video,
                &settings(Value::Null, true, 5)
            ),
            "flag/model"
        );

        let open = OutputArgs {
            open: true,
            ..Default::default()
        };
        assert!(choose(&open, None, &settings(Value::Null, false, 5)).open);
        let no_open = OutputArgs {
            no_open: true,
            ..Default::default()
        };
        assert!(!choose(&no_open, None, &settings(Value::Null, true, 5)).open);
    }

    #[test]
    fn a_prompt_is_a_flag_or_the_loose_words() {
        let parse = |args: &[&str]| match Cli::try_parse_from(args).map(|cli| cli.command) {
            Ok(Command::Image(args)) => args,
            _ => panic!("not an image command"),
        };
        let words = parse(&[
            "ormedia", "image", "a", "red", "boat", "--n", "2", "--image", "a.png", "--image",
            "b.png",
        ]);
        assert_eq!(
            (words.words.join(" "), words.n, words.image.len()),
            ("a red boat".to_string(), Some(2), 2)
        );
        let flag = parse(&[
            "ormedia",
            "image",
            "--prompt",
            "a boat",
            "--no-open",
            "--aspect-ratio",
            "1:1",
        ]);
        assert_eq!(
            (flag.prompt.as_deref(), flag.output.no_open),
            (Some("a boat"), true)
        );
        assert_eq!(flag.aspect_ratio.as_deref(), Some("1:1"));
        assert!(Cli::try_parse_from(["ormedia", "image", "--seed", "soon"]).is_err());
    }

    #[test]
    fn the_help_page_lists_every_command_and_setting() {
        let help = help();
        for command in Cli::command().get_subcommands() {
            assert!(
                help.contains(&format!("  ormedia {}", command.get_name())),
                "{}",
                command.get_name()
            );
        }
        for setting in &SETTINGS {
            assert!(help.contains(&format!("\n  {:<24}{}", setting.name, setting.about)));
        }
    }
}
