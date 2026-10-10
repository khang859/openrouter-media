use std::io::Write;
use std::process::ExitCode;
use std::time::{Duration, SystemTime};

use clap::{Args, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use serde_json::{Map, Value, json};

use crate::Result;
use crate::api::{
    Client, Https, ImageOptions, SpeechOptions, TranscribeOptions, VideoOptions, VoiceSample,
    image_body, speech_body, status_of, transcribe_body, video_body,
};
use crate::auth::{authorize, delete_key, read_key, save_key};
use crate::config::{
    self, DEFAULT_SOURCE, SETTINGS, Settings, resolve_settings, set_setting, unset_setting,
};
use crate::env::Env;
use crate::files::{
    Output, audio_format, is_web_url, open_in_fleet, open_url, save_outputs, slug, timestamp,
    to_audio_reference, to_image_url,
};
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
        "ormedia - generate images, videos, and speech with OpenRouter

Usage
  ormedia login                      Sign in with the browser and store an API key
  ormedia logout                     Delete the stored API key
  ormedia status                     Show which key is in use and its spend
  ormedia models <kind>              List the image, video, speech, or transcribe models
  ormedia image --prompt <text>      Generate or edit an image
  ormedia video --prompt <text>      Generate a video
  ormedia speech --text <text>       Turn text into speech
  ormedia transcribe <path|url>      Write down the words in an audio file
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

Speech options
  --model <id>            Default: {speech_model}
  --text-file <path>      Read the words from a file, in place of --text
  --voice <id>            A voice of the model. `ormedia models speech` lists them.
  --voice-sample <path|url>   Audio of a voice to copy. Repeat for more.
  --voice-sample-text <text>  The words said in the sample. Repeat in the same order.
  --instructions <text>   How to say it, for models that take it
  --speed <x>             For example 1.25, for models that take it
  --format <f>            mp3 or pcm. Default: mp3

Transcribe options
  --model <id>            Default: {transcribe_model}
  --language <code>       For example en, ja. Without it, the model detects the language.
  --timestamps <level>    segment or word: add the start and end time. Repeat for both.
  --speakers              Label who says each word, for models that can
  --format <f>            What the file holds, when its name does not say. For example mp3.

Shared options
  --out <path>            A file name, or a folder. Default: the current folder.
  --search <text>         models: keep only models that match
  --wait                  job: wait until the video is done
  --open | --no-open      Open the result in Fleet, or do not
  --json                  Print the result as JSON

Settings
{settings}
  A flag wins over $ORMEDIA_IMAGE_MODEL, $ORMEDIA_VIDEO_MODEL, $ORMEDIA_SPEECH_MODEL,
  $ORMEDIA_TRANSCRIBE_MODEL, and $ORMEDIA_UPDATE_CHECK, which win over a saved setting,
  which wins over the default.

The API key comes from $OPENROUTER_API_KEY, or from `ormedia login`.
",
        image_model = config::DEFAULT_IMAGE_MODEL,
        video_model = config::DEFAULT_VIDEO_MODEL,
        speech_model = config::DEFAULT_SPEECH_MODEL,
        transcribe_model = config::DEFAULT_TRANSCRIBE_MODEL,
        timeout = config::DEFAULT_TIMEOUT_MINUTES,
        settings = settings.join("\n"),
    )
}

#[derive(Parser)]
#[command(
    name = "ormedia",
    version,
    about = "Generate images, videos, and speech with OpenRouter"
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
    /// List the image, video, speech, or transcribe models
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
    /// Turn text into speech
    Speech(SpeechArgs),
    /// Write down the words in an audio file
    Transcribe(TranscribeArgs),
    /// Check a video job, and download it when done
    Job(JobArgs),
    /// Show the settings, or save one with `set` and `unset`
    Config(ConfigArgs),
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Kind {
    Image,
    Video,
    Speech,
    Transcribe,
}

impl Kind {
    /// Where OpenRouter lists the models of this kind.
    fn models_path(self) -> &'static str {
        match self {
            Kind::Image => "/images/models",
            Kind::Video => "/videos/models",
            Kind::Speech => "/models?output_modalities=speech",
            Kind::Transcribe => "/models?output_modalities=transcription",
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
struct SpeechArgs {
    /// The words to say, when they are not given with --text
    words: Vec<String>,
    #[arg(long, value_name = "text")]
    text: Option<String>,
    /// Read the words from a file, in place of --text
    #[arg(long, value_name = "path", conflicts_with = "text")]
    text_file: Option<String>,
    /// Default: the `speech-model` setting
    #[arg(long, value_name = "id")]
    model: Option<String>,
    /// A voice of the model. `ormedia models speech` lists them.
    #[arg(long, value_name = "id")]
    voice: Option<String>,
    /// Audio of a voice to copy. Repeat for more.
    #[arg(long, value_name = "path|url")]
    voice_sample: Vec<String>,
    /// The words said in the sample. Repeat in the same order.
    #[arg(long, value_name = "text")]
    voice_sample_text: Vec<String>,
    /// How to say it, for models that take it
    #[arg(long, value_name = "text")]
    instructions: Option<String>,
    /// For example 1.25, for models that take it
    #[arg(long, value_name = "x")]
    speed: Option<f64>,
    /// mp3 or pcm. Default: mp3
    #[arg(long, value_name = "f")]
    format: Option<String>,
    #[command(flatten)]
    output: OutputArgs,
}

#[derive(Args)]
struct TranscribeArgs {
    /// The audio file, or its URL
    #[arg(value_name = "path|url")]
    audio: String,
    /// Default: the `transcribe-model` setting
    #[arg(long, value_name = "id")]
    model: Option<String>,
    /// For example en, ja. Without it, the model detects the language.
    #[arg(long, value_name = "code")]
    language: Option<String>,
    /// segment or word: add the start and end time. Repeat for both.
    #[arg(long, value_name = "level")]
    timestamps: Vec<String>,
    /// Label who says each word, for models that can
    #[arg(long)]
    speakers: bool,
    /// What the file holds, when its name does not say. For example mp3.
    #[arg(long, value_name = "f")]
    format: Option<String>,
    /// Print the result as JSON
    #[arg(long)]
    json: bool,
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
        Kind::Speech => (config::SPEECH_MODEL, config::DEFAULT_SPEECH_MODEL),
        Kind::Transcribe => (config::TRANSCRIBE_MODEL, config::DEFAULT_TRANSCRIBE_MODEL),
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
        .list_models(kind.models_path())?
        .into_iter()
        .filter(matches)
        .collect();
    if json {
        say(pretty(&Value::Array(list)));
    } else {
        say(list
            .iter()
            .map(|model| describe_model(kind, model))
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

fn speech(args: SpeechArgs, env: &Env) -> Result<()> {
    let text = match (args.text, args.text_file) {
        (Some(text), _) => text,
        (None, Some(path)) => std::fs::read_to_string(&path)
            .map_err(|error| format!("Cannot read {path}: {error}"))?,
        (None, None) => args.words.join(" "),
    };
    if text.trim().is_empty() {
        return Err("Speech needs a --text or a --text-file".into());
    }
    if args.voice_sample_text.len() > args.voice_sample.len() {
        return Err("Each --voice-sample-text needs a --voice-sample".into());
    }
    let mut transcripts = args.voice_sample_text.into_iter();
    let samples = args
        .voice_sample
        .iter()
        .map(|reference| {
            Ok(VoiceSample {
                audio: to_audio_reference(reference)?,
                transcript: transcripts.next(),
            })
        })
        .collect::<Result<_>>()?;
    let settings = resolve_settings(env)?;
    let chosen = choose(&args.output, None, &settings);
    let model = choose_model(args.model, Kind::Speech, &settings);
    let body = speech_body(SpeechOptions {
        model: model.clone(),
        text: text.clone(),
        voice: args.voice,
        samples,
        instructions: args.instructions,
        speed: args.speed,
        // Always sent: the API default is raw PCM, which no player opens.
        format: Some(args.format.unwrap_or_else(|| "mp3".to_string())),
    });

    let client = signed_in_client(env)?;
    progress(format!("Generating speech with {model}..."));
    let audio = client.generate_speech(&body)?;
    let stem = format!("{}-{}", slug(&text), timestamp(SystemTime::now()));
    let facts = vec![("model", json!(model))];
    finish(&[audio], &stem, &chosen, args.output.json, facts, env)
}

/// A transcript as lines of text: one line per timed segment, or the text alone.
fn describe_transcript(reply: &Value) -> String {
    let segments = reply["segments"].as_array();
    let Some(segments) = segments.filter(|segments| !segments.is_empty()) else {
        return plain(&reply["text"]).trim().to_string();
    };
    let line = |segment: &Value| {
        let seconds = |field: &str| segment[field].as_f64().unwrap_or_default();
        format!(
            "[{:.2} - {:.2}] {}",
            seconds("start"),
            seconds("end"),
            plain(&segment["text"]).trim()
        )
    };
    segments.iter().map(line).collect::<Vec<_>>().join("\n")
}

fn transcribe(args: TranscribeArgs, env: &Env) -> Result<()> {
    let web = is_web_url(&args.audio);
    let format = args.format.or_else(|| audio_format(&args.audio));
    if !web && format.is_none() {
        return Err(format!(
            "Cannot tell what {} holds. Add --format, for example --format mp3",
            args.audio
        ));
    }
    let model = choose_model(args.model, Kind::Transcribe, &resolve_settings(env)?);
    let body = transcribe_body(TranscribeOptions {
        model: model.clone(),
        audio: to_audio_reference(&args.audio)?,
        // A URL says its own format, so only an explicit one is sent with it.
        format: if web { None } else { format },
        language: args.language,
        timestamps: args.timestamps,
        speakers: args.speakers,
    });

    let client = signed_in_client(env)?;
    progress(format!("Transcribing with {model}..."));
    let mut reply = client.transcribe(&body)?;
    if args.json {
        reply["model"] = json!(model);
        say(pretty(&reply));
        return Ok(());
    }
    say(describe_transcript(&reply));
    // To stderr, so the transcript alone can go to a file or a pipe.
    let cost = &reply["usage"]["cost"];
    if !cost.is_null() {
        progress(format!("Cost ${}", plain(cost)));
    }
    Ok(())
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
    // Two spaces after the longest name.
    let width = SETTINGS
        .iter()
        .map(|setting| setting.name.len())
        .max()
        .unwrap_or(0)
        + 2;
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
        say(format!("{:<width$}{value}", resolved.name));
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
        Command::Speech(args) => speech(args, env)?,
        Command::Transcribe(args) => transcribe(args, env)?,
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
            (config::SPEECH_MODEL, json!("saved/speech")),
            (config::TRANSCRIBE_MODEL, json!("saved/transcribe")),
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
        assert_eq!(choose_model(None, Kind::Speech, &saved), "saved/speech");
        assert_eq!(
            choose_model(None, Kind::Transcribe, &saved),
            "saved/transcribe"
        );

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
    fn speech_takes_its_words_three_ways_and_pairs_samples_with_their_text() {
        let parse = |args: &[&str]| match Cli::try_parse_from(args).map(|cli| cli.command) {
            Ok(Command::Speech(args)) => args,
            _ => panic!("not a speech command"),
        };
        assert_eq!(parse(&["ormedia", "speech", "Hi", "there."]).words.len(), 2);
        let flags = parse(&[
            "ormedia",
            "speech",
            "--text",
            "Hi.",
            "--voice-sample",
            "a.wav",
            "--voice-sample-text",
            "First.",
            "--voice-sample",
            "b.wav",
            "--speed",
            "1.25",
        ]);
        assert_eq!(flags.text.as_deref(), Some("Hi."));
        assert_eq!(flags.voice_sample, ["a.wav", "b.wav"]);
        assert_eq!(flags.voice_sample_text, ["First."]);
        assert_eq!(flags.speed, Some(1.25));
        assert_eq!(
            parse(&["ormedia", "speech", "--text-file", "script.txt"])
                .text_file
                .as_deref(),
            Some("script.txt")
        );
        let both = ["ormedia", "speech", "--text", "Hi.", "--text-file", "s.txt"];
        assert!(Cli::try_parse_from(both).is_err());
    }

    #[test]
    fn a_transcript_is_one_line_per_timed_segment_or_the_text_alone() {
        let timed = json!({
            "text": " Hello. Goodbye.",
            "segments": [
                { "start": 0, "end": 0.68, "text": " Hello." },
                { "start": 0.875, "end": 12.5, "text": " Goodbye." }
            ]
        });
        assert_eq!(
            describe_transcript(&timed),
            "[0.00 - 0.68] Hello.\n[0.88 - 12.50] Goodbye."
        );
        for bare in [
            json!({ "text": " Hello. Goodbye.\n" }),
            json!({ "text": "Hello. Goodbye.", "segments": [] }),
        ] {
            assert_eq!(describe_transcript(&bare), "Hello. Goodbye.");
        }
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
