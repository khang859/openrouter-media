use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Map, Value, json};

use crate::Result;
use crate::files::{Output, is_web_url};

pub const BASE_URL: &str = "https://openrouter.ai/api/v1";

/// Who is asking, on every request. OpenRouter uses it for app attribution.
const APP_HEADERS: [(&str, &str); 2] = [
    (
        "HTTP-Referer",
        "https://github.com/khang859/openrouter-media",
    ),
    ("X-Title", "OpenRouter Media"),
];

/// Job states that will never turn into a video.
const DEAD_STATUSES: [&str; 3] = ["failed", "cancelled", "expired"];

/// A video can be large, and so can a 4K image as base64 inside JSON.
const MAX_BODY_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// A value for one part of a URL path or query.
pub fn encode_component(text: &str) -> String {
    let mut encoded = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn as_image_url(url: &str) -> Value {
    json!({ "type": "image_url", "image_url": { "url": url } })
}

/// Adds the fields that are set. One the user left alone means "whatever the model does".
fn set_optional(body: &mut Map<String, Value>, fields: Vec<(&str, Option<Value>)>) {
    for (name, value) in fields {
        if let Some(value) = value {
            body.insert(name.to_string(), value);
        }
    }
}

fn set_references(body: &mut Map<String, Value>, references: &[String]) {
    if !references.is_empty() {
        let references = references.iter().map(|url| as_image_url(url)).collect();
        body.insert("input_references".to_string(), Value::Array(references));
    }
}

#[derive(Default)]
pub struct ImageOptions {
    pub model: String,
    pub prompt: String,
    pub references: Vec<String>,
    pub aspect_ratio: Option<String>,
    pub resolution: Option<String>,
    pub size: Option<String>,
    pub quality: Option<String>,
    pub format: Option<String>,
    pub background: Option<String>,
    pub seed: Option<u64>,
    pub n: Option<u64>,
}

pub fn image_body(o: ImageOptions) -> Value {
    // Buffered on purpose: a CLI has nowhere to show a half-drawn render.
    let Value::Object(mut body) = json!({ "model": o.model, "prompt": o.prompt, "stream": false })
    else {
        unreachable!("the literal is an object");
    };
    set_optional(
        &mut body,
        vec![
            ("n", o.n.map(Value::from)),
            ("aspect_ratio", o.aspect_ratio.map(Value::from)),
            ("resolution", o.resolution.map(Value::from)),
            ("size", o.size.map(Value::from)),
            ("quality", o.quality.map(Value::from)),
            ("seed", o.seed.map(Value::from)),
            ("output_format", o.format.map(Value::from)),
            ("background", o.background.map(Value::from)),
        ],
    );
    set_references(&mut body, &o.references);
    Value::Object(body)
}

#[derive(Default)]
pub struct VideoOptions {
    pub model: String,
    pub prompt: Option<String>,
    pub first_frame: Option<String>,
    pub last_frame: Option<String>,
    pub references: Vec<String>,
    pub duration: Option<u64>,
    pub aspect_ratio: Option<String>,
    pub resolution: Option<String>,
    pub audio: Option<bool>,
    pub seed: Option<u64>,
}

pub fn video_body(o: VideoOptions) -> Value {
    let mut body = Map::new();
    body.insert("model".to_string(), Value::from(o.model));
    set_optional(
        &mut body,
        vec![
            ("prompt", o.prompt.map(Value::from)),
            ("duration", o.duration.map(Value::from)),
            ("aspect_ratio", o.aspect_ratio.map(Value::from)),
            ("resolution", o.resolution.map(Value::from)),
            ("generate_audio", o.audio.map(Value::from)),
            ("seed", o.seed.map(Value::from)),
        ],
    );
    let frames: Vec<Value> = [(o.first_frame, "first_frame"), (o.last_frame, "last_frame")]
        .into_iter()
        .filter_map(|(url, frame_type)| {
            let mut frame = as_image_url(&url?);
            frame["frame_type"] = Value::from(frame_type);
            Some(frame)
        })
        .collect();
    if !frames.is_empty() {
        body.insert("frame_images".to_string(), Value::Array(frames));
    }
    set_references(&mut body, &o.references);
    Value::Object(body)
}

/// A recording of the voice to copy, and the words said in it.
pub struct VoiceSample {
    /// A web URL, or the audio as base64.
    pub audio: String,
    pub transcript: Option<String>,
}

#[derive(Default)]
pub struct SpeechOptions {
    pub model: String,
    pub text: String,
    pub voice: Option<String>,
    pub samples: Vec<VoiceSample>,
    pub instructions: Option<String>,
    pub speed: Option<f64>,
    pub format: Option<String>,
}

pub fn speech_body(o: SpeechOptions) -> Value {
    let Value::Object(mut body) = json!({ "model": o.model, "input": o.text }) else {
        unreachable!("the literal is an object");
    };
    set_optional(
        &mut body,
        vec![
            ("voice", o.voice.map(Value::from)),
            ("instructions", o.instructions.map(Value::from)),
            ("speed", o.speed.map(Value::from)),
            ("response_format", o.format.map(Value::from)),
        ],
    );
    // A transcript comes right after the sample it belongs to.
    let references: Vec<Value> = o
        .samples
        .into_iter()
        .flat_map(|sample| {
            let field = if is_web_url(&sample.audio) {
                "url"
            } else {
                "data"
            };
            let audio = json!({ "type": "input_audio", "input_audio": { field: sample.audio } });
            let transcript = sample
                .transcript
                .map(|text| json!({ "type": "text", "text": text }));
            std::iter::once(audio).chain(transcript)
        })
        .collect();
    if !references.is_empty() {
        body.insert("input_references".to_string(), Value::Array(references));
    }
    Value::Object(body)
}

#[derive(Default)]
pub struct TranscribeOptions {
    pub model: String,
    /// A web URL, or the audio as base64.
    pub audio: String,
    /// What a local file holds, for example `mp3`. A URL says it with its own name.
    pub format: Option<String>,
    pub language: Option<String>,
    /// `segment`, `word`, or both.
    pub timestamps: Vec<String>,
    pub speakers: bool,
}

pub fn transcribe_body(o: TranscribeOptions) -> Value {
    let field = if is_web_url(&o.audio) { "url" } else { "data" };
    let Value::Object(mut audio) = json!({ field: o.audio }) else {
        unreachable!("the literal is an object");
    };
    set_optional(&mut audio, vec![("format", o.format.map(Value::from))]);
    let Value::Object(mut body) = json!({ "model": o.model, "input_audio": audio }) else {
        unreachable!("the literal is an object");
    };
    // Times and speakers come only in the long form of the reply.
    let verbose = o.speakers || !o.timestamps.is_empty();
    set_optional(
        &mut body,
        vec![
            ("language", o.language.map(Value::from)),
            ("response_format", verbose.then(|| json!("verbose_json"))),
            (
                "timestamp_granularities",
                (!o.timestamps.is_empty()).then(|| json!(o.timestamps)),
            ),
            ("diarize", o.speakers.then(|| json!(true))),
        ],
    );
    Value::Object(body)
}

/// One call to OpenRouter. A request with a body is a POST, one without is a GET.
pub struct Request<'a> {
    pub url: String,
    pub api_key: Option<&'a str>,
    pub body: Option<&'a Value>,
}

pub struct Response {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

/// The network, as a parameter, so tests never call the real API.
pub trait Transport {
    fn send(&self, request: Request) -> Result<Response>;
}

pub struct Https(ureq::Agent);

impl Https {
    pub fn new() -> Self {
        let config = ureq::Agent::config_builder()
            // The status is ours to read: an error reply carries a message for the user.
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(30)))
            // An image can take minutes to render before the first byte arrives.
            .timeout_recv_response(Some(Duration::from_secs(15 * 60)))
            .user_agent(concat!("ormedia/", env!("CARGO_PKG_VERSION")))
            .build();
        Self(config.into())
    }
}

impl Transport for Https {
    fn send(&self, request: Request) -> Result<Response> {
        let authorization = request.api_key.map(|key| format!("Bearer {key}"));
        let headers = APP_HEADERS.into_iter().chain(
            authorization
                .as_deref()
                .map(|value| ("Authorization", value)),
        );
        let sent = match request.body {
            None => headers
                .fold(self.0.get(&request.url), |call, (name, value)| {
                    call.header(name, value)
                })
                .call(),
            Some(body) => headers
                .fold(self.0.post(&request.url), |call, (name, value)| {
                    call.header(name, value)
                })
                .send_json(body),
        };
        let failed = |error: ureq::Error| format!("Cannot reach OpenRouter: {error}");
        let mut response = sent.map_err(failed)?;
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok());
        Ok(Response {
            status: response.status().as_u16(),
            content_type: content_type.map(String::from),
            body: response
                .body_mut()
                .with_config()
                .limit(MAX_BODY_BYTES)
                .read_to_vec()
                .map_err(failed)?,
        })
    }
}

fn error_message(response: &Response) -> String {
    let status = response.status;
    let hint = if status == 401 {
        " Run `ormedia login` to sign in."
    } else {
        ""
    };
    let reply: Value = serde_json::from_slice(&response.body).unwrap_or_default();
    match reply["error"]["message"].as_str() {
        Some(message) => format!("{message} (HTTP {status}).{hint}"),
        None => format!("OpenRouter responded {status}.{hint}"),
    }
}

/// What a failed job says about itself, in whichever shape the provider used.
fn job_error(job: &Value) -> String {
    let error = &job["error"];
    let message = error.as_str().or_else(|| error["message"].as_str());
    message.map_or_else(|| format!("The video job {}", status_of(job)), String::from)
}

pub fn status_of(job: &Value) -> &str {
    job["status"].as_str().unwrap_or("unknown")
}

pub struct Images {
    pub images: Vec<Output>,
    pub cost_usd: Value,
}

pub struct Client<T> {
    api_key: Option<String>,
    transport: T,
}

impl Client<Https> {
    pub fn https(api_key: Option<String>) -> Self {
        Self::new(api_key, Https::new())
    }
}

impl<T: Transport> Client<T> {
    pub fn new(api_key: Option<String>, transport: T) -> Self {
        Self { api_key, transport }
    }

    fn call(&self, path: &str, body: Option<&Value>) -> Result<Response> {
        let request = Request {
            url: format!("{BASE_URL}{path}"),
            api_key: self.api_key.as_deref(),
            body,
        };
        let response = self.transport.send(request)?;
        if (200..300).contains(&response.status) {
            Ok(response)
        } else {
            Err(error_message(&response))
        }
    }

    fn json(&self, path: &str, body: Option<&Value>) -> Result<Value> {
        serde_json::from_slice(&self.call(path, body)?.body)
            .map_err(|error| format!("OpenRouter sent a reply that is not JSON ({error})"))
    }

    /// `path` is the model list for one kind of media. Needs no key.
    pub fn list_models(&self, path: &str) -> Result<Vec<Value>> {
        match self.json(path, None)?["data"].take() {
            Value::Array(models) => Ok(models),
            _ => Err("OpenRouter returned no model list".into()),
        }
    }

    pub fn generate_images(&self, body: &Value) -> Result<Images> {
        let mut reply = self.json("/images", Some(body))?;
        let decode = |image: &Value| {
            let data = image["b64_json"]
                .as_str()
                .and_then(|text| STANDARD.decode(text).ok());
            let mime_type = image["media_type"]
                .as_str()
                .unwrap_or("image/png")
                .to_string();
            Some(Output {
                data: data?,
                mime_type,
            })
        };
        let images: Option<Vec<Output>> = reply["data"]
            .as_array()
            .into_iter()
            .flatten()
            .map(decode)
            .collect();
        match images {
            Some(images) if !images.is_empty() => Ok(Images {
                images,
                cost_usd: reply["usage"]["cost"].take(),
            }),
            _ => Err("OpenRouter returned no image".into()),
        }
    }

    /// The reply is the audio itself, not JSON, so it has no cost field.
    pub fn generate_speech(&self, body: &Value) -> Result<Output> {
        let response = self.call("/audio/speech", Some(body))?;
        if response.body.is_empty() {
            return Err("OpenRouter returned no audio".into());
        }
        let mime_type = response
            .content_type
            .unwrap_or_else(|| "application/octet-stream".to_string());
        Ok(Output {
            data: response.body,
            mime_type,
        })
    }

    /// The whole reply: the text, the cost, and the times when they were asked for.
    pub fn transcribe(&self, body: &Value) -> Result<Value> {
        let reply = self.json("/audio/transcriptions", Some(body))?;
        if reply["text"].is_string() {
            Ok(reply)
        } else {
            Err("OpenRouter returned no transcript".into())
        }
    }

    pub fn submit_video(&self, body: &Value) -> Result<Value> {
        self.json("/videos", Some(body))
    }

    pub fn get_video_job(&self, id: &str) -> Result<Value> {
        self.json(&format!("/videos/{}", encode_component(id)), None)
    }

    pub fn download_video(&self, id: &str, index: usize) -> Result<Output> {
        let response = self.call(
            &format!("/videos/{}/content?index={index}", encode_component(id)),
            None,
        )?;
        let mime_type = response
            .content_type
            .unwrap_or_else(|| "video/mp4".to_string());
        Ok(Output {
            data: response.body,
            mime_type,
        })
    }

    /// Polls until the job has a video. A job that outlives `timeout` is not
    /// lost: it keeps running on OpenRouter and `ormedia job <id>` picks it up.
    pub fn wait_for_video(
        &self,
        id: &str,
        timeout: Duration,
        interval: Duration,
        mut on_status: impl FnMut(&str),
    ) -> Result<Value> {
        let started = Instant::now();
        loop {
            let job = self.get_video_job(id)?;
            let status = status_of(&job);
            on_status(status);
            if status == "completed" {
                return Ok(job);
            }
            if DEAD_STATUSES.contains(&status) {
                return Err(job_error(&job));
            }
            if started.elapsed() + interval > timeout {
                return Err(format!(
                    "The video is still {status}. It keeps running. Get it later with: ormedia job {id} --wait"
                ));
            }
            std::thread::sleep(interval);
        }
    }

    pub fn exchange_code(&self, code: &str, verifier: &str) -> Result<String> {
        let body =
            json!({ "code": code, "code_verifier": verifier, "code_challenge_method": "S256" });
        match self.json("/auth/keys", Some(&body))?["key"].take() {
            Value::String(key) => Ok(key),
            _ => Err("OpenRouter returned no API key".into()),
        }
    }

    pub fn key_info(&self) -> Result<Value> {
        Ok(self.json("/key", None)?["data"].take())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    /// What one call asked for: the URL, the key, and the body.
    type Call = (String, Option<String>, Option<Value>);

    /// A network that answers from a queue and remembers what it was asked.
    #[derive(Default)]
    struct Fake {
        replies: RefCell<VecDeque<Response>>,
        calls: RefCell<Vec<Call>>,
    }

    impl Fake {
        fn replying(replies: Vec<(u16, Value)>) -> Self {
            let reply = |(status, body): (u16, Value)| Response {
                status,
                content_type: Some("application/json".into()),
                body: body.to_string().into_bytes(),
            };
            Self {
                replies: RefCell::new(replies.into_iter().map(reply).collect()),
                ..Self::default()
            }
        }

        fn client(&self, api_key: Option<&str>) -> Client<&Self> {
            Client::new(api_key.map(String::from), self)
        }
    }

    impl Transport for &Fake {
        fn send(&self, request: Request) -> Result<Response> {
            let call = (
                request.url,
                request.api_key.map(String::from),
                request.body.cloned(),
            );
            self.calls.borrow_mut().push(call);
            Ok(self
                .replies
                .borrow_mut()
                .pop_front()
                .expect("a reply for every call"))
        }
    }

    const SOON: Duration = Duration::from_millis(1);

    #[test]
    fn image_body_sends_only_what_was_set() {
        let body = image_body(ImageOptions {
            model: "m".into(),
            prompt: "a cat".into(),
            ..Default::default()
        });
        assert_eq!(
            body,
            json!({ "model": "m", "prompt": "a cat", "stream": false })
        );
    }

    #[test]
    fn image_body_maps_options_and_references_to_api_names() {
        let body = image_body(ImageOptions {
            model: "m".into(),
            prompt: "p".into(),
            references: vec!["https://x/a.png".into()],
            aspect_ratio: Some("16:9".into()),
            format: Some("webp".into()),
            seed: Some(0),
            n: Some(2),
            ..Default::default()
        });
        assert_eq!(
            body,
            json!({
                "model": "m",
                "prompt": "p",
                "stream": false,
                "n": 2,
                "aspect_ratio": "16:9",
                "seed": 0,
                "output_format": "webp",
                "input_references": [{ "type": "image_url", "image_url": { "url": "https://x/a.png" } }]
            })
        );
    }

    #[test]
    fn video_body_tags_frames_and_keeps_an_explicit_audio_off() {
        let body = video_body(VideoOptions {
            model: "m".into(),
            prompt: Some("p".into()),
            first_frame: Some("data:a".into()),
            last_frame: Some("data:b".into()),
            audio: Some(false),
            ..Default::default()
        });
        assert_eq!(
            body,
            json!({
                "model": "m",
                "prompt": "p",
                "generate_audio": false,
                "frame_images": [
                    { "type": "image_url", "image_url": { "url": "data:a" }, "frame_type": "first_frame" },
                    { "type": "image_url", "image_url": { "url": "data:b" }, "frame_type": "last_frame" }
                ]
            })
        );
        assert_eq!(
            video_body(VideoOptions {
                model: "m".into(),
                ..Default::default()
            }),
            json!({ "model": "m" })
        );
    }

    #[test]
    fn speech_body_sends_only_what_was_set() {
        let body = speech_body(SpeechOptions {
            model: "m".into(),
            text: "Hello.".into(),
            ..Default::default()
        });
        assert_eq!(body, json!({ "model": "m", "input": "Hello." }));
    }

    #[test]
    fn speech_body_puts_each_transcript_after_its_voice_sample() {
        let sample = |audio: &str, transcript: Option<&str>| VoiceSample {
            audio: audio.into(),
            transcript: transcript.map(String::from),
        };
        let body = speech_body(SpeechOptions {
            model: "m".into(),
            text: "Hello.".into(),
            voice: Some("v".into()),
            samples: vec![
                sample("UklGRg==", Some("First clip.")),
                sample("HTTPS://x/b.wav", None),
            ],
            instructions: Some("Slow and calm.".into()),
            speed: Some(1.25),
            format: Some("mp3".into()),
        });
        assert_eq!(
            body,
            json!({
                "model": "m",
                "input": "Hello.",
                "voice": "v",
                "instructions": "Slow and calm.",
                "speed": 1.25,
                "response_format": "mp3",
                "input_references": [
                    { "type": "input_audio", "input_audio": { "data": "UklGRg==" } },
                    { "type": "text", "text": "First clip." },
                    { "type": "input_audio", "input_audio": { "url": "HTTPS://x/b.wav" } }
                ]
            })
        );
    }

    #[test]
    fn generate_speech_returns_the_audio_with_the_type_the_reply_names() {
        let audio = |content_type: Option<&str>, body: &[u8]| Fake {
            replies: RefCell::new(VecDeque::from([Response {
                status: 200,
                content_type: content_type.map(String::from),
                body: body.to_vec(),
            }])),
            ..Fake::default()
        };
        let body = json!({ "model": "m", "input": "Hello." });

        let fake = audio(Some("audio/mpeg"), b"ID3");
        let speech = fake.client(Some("k")).generate_speech(&body).unwrap();
        assert_eq!(
            speech,
            Output {
                data: b"ID3".to_vec(),
                mime_type: "audio/mpeg".into()
            }
        );
        assert_eq!(
            fake.calls.borrow()[0],
            (
                "https://openrouter.ai/api/v1/audio/speech".into(),
                Some("k".into()),
                Some(body.clone())
            )
        );

        let unnamed = audio(None, b"raw").client(None).generate_speech(&body);
        assert_eq!(unnamed.unwrap().mime_type, "application/octet-stream");
        let empty = audio(Some("audio/mpeg"), b"")
            .client(None)
            .generate_speech(&body);
        assert_eq!(empty.unwrap_err(), "OpenRouter returned no audio");

        let fake = Fake::replying(vec![(
            400,
            json!({ "error": { "message": "No such voice" } }),
        )]);
        assert_eq!(
            fake.client(None).generate_speech(&body).unwrap_err(),
            "No such voice (HTTP 400)."
        );
    }

    #[test]
    fn transcribe_body_sends_a_file_with_its_format_and_a_url_as_it_is() {
        let file = transcribe_body(TranscribeOptions {
            model: "m".into(),
            audio: "UklGRg==".into(),
            format: Some("wav".into()),
            ..Default::default()
        });
        assert_eq!(
            file,
            json!({ "model": "m", "input_audio": { "data": "UklGRg==", "format": "wav" } })
        );
        let url = transcribe_body(TranscribeOptions {
            model: "m".into(),
            audio: "https://x/a.mp3".into(),
            ..Default::default()
        });
        assert_eq!(
            url,
            json!({ "model": "m", "input_audio": { "url": "https://x/a.mp3" } })
        );
    }

    #[test]
    fn transcribe_body_asks_for_the_long_reply_only_for_times_or_speakers() {
        let body = |timestamps: &[&str], speakers| {
            let mut body = transcribe_body(TranscribeOptions {
                model: "m".into(),
                audio: "https://x/a.mp3".into(),
                language: Some("en".into()),
                timestamps: timestamps.iter().map(|level| level.to_string()).collect(),
                speakers,
                ..Default::default()
            });
            let body = body.as_object_mut().unwrap();
            body.shift_remove("model");
            body.shift_remove("input_audio");
            Value::Object(body.clone())
        };
        assert_eq!(body(&[], false), json!({ "language": "en" }));
        assert_eq!(
            body(&["segment", "word"], false),
            json!({
                "language": "en",
                "response_format": "verbose_json",
                "timestamp_granularities": ["segment", "word"]
            })
        );
        assert_eq!(
            body(&[], true),
            json!({ "language": "en", "response_format": "verbose_json", "diarize": true })
        );
    }

    #[test]
    fn transcribe_returns_the_whole_reply_and_refuses_one_with_no_text() {
        let reply = json!({ "text": "Hello.", "usage": { "cost": 0.0006 } });
        let fake = Fake::replying(vec![(200, reply.clone()), (200, json!({ "usage": {} }))]);
        let client = fake.client(Some("k"));
        let body = json!({ "model": "m" });
        assert_eq!(client.transcribe(&body).unwrap(), reply);
        assert_eq!(
            fake.calls.borrow()[0],
            (
                "https://openrouter.ai/api/v1/audio/transcriptions".into(),
                Some("k".into()),
                Some(body.clone())
            )
        );
        assert_eq!(
            client.transcribe(&body).unwrap_err(),
            "OpenRouter returned no transcript"
        );
    }

    #[test]
    fn a_model_list_is_read_from_the_path_of_its_kind() {
        let fake = Fake::replying(vec![
            (200, json!({ "data": [{ "id": "a/b" }] })),
            (200, json!({})),
        ]);
        let client = fake.client(None);
        let models = client.list_models("/models?output_modalities=speech");
        assert_eq!(models.unwrap(), [json!({ "id": "a/b" })]);
        assert_eq!(
            fake.calls.borrow()[0].0,
            "https://openrouter.ai/api/v1/models?output_modalities=speech"
        );
        assert_eq!(
            client.list_models("/images/models").unwrap_err(),
            "OpenRouter returned no model list"
        );
    }

    #[test]
    fn generate_images_decodes_every_image_and_reports_the_cost() {
        let fake = Fake::replying(vec![(
            200,
            json!({ "data": [{ "b64_json": "b25l", "media_type": "image/webp" }], "usage": { "cost": 0.04 } }),
        )]);
        let body = json!({ "model": "m" });
        let result = fake.client(Some("k")).generate_images(&body).unwrap();
        assert_eq!(
            result.images,
            [Output {
                data: b"one".to_vec(),
                mime_type: "image/webp".into()
            }]
        );
        assert_eq!(result.cost_usd, json!(0.04));
        let calls = fake.calls.borrow();
        assert_eq!(
            calls[0],
            (
                "https://openrouter.ai/api/v1/images".into(),
                Some("k".into()),
                Some(body)
            )
        );
    }

    #[test]
    fn a_reply_with_no_usable_image_is_an_error() {
        for reply in [
            json!({ "data": [] }),
            json!({}),
            json!({ "data": [{ "b64_json": "not base64!" }] }),
        ] {
            let fake = Fake::replying(vec![(200, reply)]);
            assert_eq!(
                fake.client(None).generate_images(&json!({})).err().unwrap(),
                "OpenRouter returned no image"
            );
        }
    }

    #[test]
    fn an_api_error_surfaces_its_message_and_a_401_says_how_to_sign_in() {
        let fake = Fake::replying(vec![
            (401, json!({ "error": { "message": "No auth" } })),
            (500, json!("oops")),
        ]);
        let client = fake.client(None);
        assert_eq!(
            client.key_info().unwrap_err(),
            "No auth (HTTP 401). Run `ormedia login` to sign in."
        );
        assert_eq!(client.key_info().unwrap_err(), "OpenRouter responded 500.");
        assert_eq!(fake.calls.borrow()[0].1, None);
    }

    #[test]
    fn a_job_id_is_escaped_in_the_url() {
        let fake = Fake::replying(vec![(200, json!({ "status": "pending" }))]);
        fake.client(None).get_video_job("a/b?c").unwrap();
        assert_eq!(
            fake.calls.borrow()[0].0,
            "https://openrouter.ai/api/v1/videos/a%2Fb%3Fc"
        );
    }

    #[test]
    fn wait_for_video_polls_until_the_job_completes() {
        let fake = Fake::replying(vec![
            (200, json!({ "id": "j", "status": "pending" })),
            (200, json!({ "id": "j", "status": "in_progress" })),
            (
                200,
                json!({ "id": "j", "status": "completed", "unsigned_urls": ["u"] }),
            ),
        ]);
        let mut seen = Vec::new();
        let on_status = |status: &str| seen.push(status.to_string());
        let job = fake
            .client(Some("k"))
            .wait_for_video("j", Duration::from_secs(1), SOON, on_status)
            .unwrap();
        assert_eq!(status_of(&job), "completed");
        assert_eq!(seen, ["pending", "in_progress", "completed"]);
        assert_eq!(fake.calls.borrow().len(), 3);
    }

    #[test]
    fn wait_for_video_fails_with_the_reason_the_job_gives() {
        let failed = |job| {
            let fake = Fake::replying(vec![(200, job)]);
            fake.client(None)
                .wait_for_video("j", Duration::from_secs(1), SOON, |_| {})
                .unwrap_err()
        };
        assert_eq!(
            failed(json!({ "status": "failed", "error": "blocked prompt" })),
            "blocked prompt"
        );
        assert_eq!(
            failed(json!({ "status": "failed", "error": { "message": "too long" } })),
            "too long"
        );
        assert_eq!(
            failed(json!({ "status": "expired" })),
            "The video job expired"
        );
    }

    #[test]
    fn wait_for_video_gives_up_at_the_deadline_and_says_how_to_resume() {
        let fake = Fake::replying(vec![(200, json!({ "id": "j", "status": "pending" }))]);
        let error = fake
            .client(None)
            .wait_for_video(
                "j",
                Duration::from_millis(5),
                Duration::from_millis(10),
                |_| {},
            )
            .unwrap_err();
        assert!(
            error.starts_with("The video is still pending.")
                && error.ends_with("ormedia job j --wait")
        );
    }
}
