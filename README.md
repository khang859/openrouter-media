# openrouter-media

A Claude Code plugin that generates and edits images, generates videos, turns text into speech, and transcribes audio, with any model on [OpenRouter](https://openrouter.ai).

It adds three things:

- `ormedia`, a small command-line tool written in Rust.
- A `generate-media` skill that tells Claude when and how to use it.
- Slash commands, such as `/openrouter-media:login`, for the things you do yourself.

## Install

```
/plugin marketplace add khang859/openrouter-media
/plugin install openrouter-media@openrouter-media
```

Then ask Claude for an image, a video, or a voice-over.
The first time `ormedia` runs, it downloads the program for your computer from the GitHub release, one time.
It needs `curl` for that, and nothing else.
To sign in, type `/openrouter-media:login`, your browser opens, and you approve.
If you skip that, Claude runs the sign-in the first time it needs a key.
The API key is stored in `~/.config/openrouter-media/credentials.json`, readable only by you.
You never paste a key into the chat.

To use a key you already have, set `OPENROUTER_API_KEY` instead.
It wins over the stored key.

## Slash commands

Type these in Claude Code.

| Command                                                              | What it does                                |
| -------------------------------------------------------------------- | ------------------------------------------- |
| `/openrouter-media:login`                                            | Sign in with the browser                    |
| `/openrouter-media:logout`                                           | Delete the stored API key                   |
| `/openrouter-media:status`                                           | Show which key is in use and its spend      |
| `/openrouter-media:models <image\|video\|speech\|transcribe> [text]` | List models, and keep those that match text |
| `/openrouter-media:config`                                           | Show the settings                           |
| `/openrouter-media:config set <name> <value>`                        | Save a setting                              |
| `/openrouter-media:config unset <name>`                              | Remove a saved setting                      |

Each one runs the `ormedia` command of the same name.
Claude never starts these on its own.
For an image, a video, or speech, just ask Claude.

## Commands

| Command                          | What it does                                          |
| -------------------------------- | ----------------------------------------------------- |
| `ormedia login`                  | Sign in with the browser and store an API key         |
| `ormedia logout`                 | Delete the stored API key                             |
| `ormedia status`                 | Show which key is in use and its spend                |
| `ormedia models <kind>`          | List the image, video, speech, or transcribe models   |
| `ormedia image --prompt <text>`  | Generate or edit an image                             |
| `ormedia video --prompt <text>`  | Generate a video                                      |
| `ormedia speech --text <text>`   | Turn text into speech                                 |
| `ormedia transcribe <path\|url>` | Write down the words in an audio file                 |
| `ormedia job <job-id>`           | Check a video job, and download it when it is done    |
| `ormedia config`                 | Show the settings, or save one with `set` and `unset` |

Run `ormedia --help` for every option.

```bash
ormedia image --prompt "A flat vector fox logo, orange on white" --aspect-ratio 1:1 --out assets/
ormedia image --prompt "Make it night. Keep everything else." --image ./street.png
ormedia video --prompt "Waves roll onto a black sand beach, slow push in" --duration 4
ormedia speech --text "[calm] The tide comes in, and the day ends." --voice <id>
```

Results are saved to the current folder, or to `--out`, or to the `out` setting.
A file that already exists is never overwritten.
Add `--json` to get the result as JSON.

## Models

The defaults are `google/gemini-nano-banana-2.1` for images, `x-ai/grok-imagine-video-1.5-lite` for videos, and `fish-audio/s2.1-pro` for speech, and `fish-audio/transcribe-1-pro` for transcripts.
Change them per call with `--model`, or for good with `ormedia config`.

## Speech and dubbing

`ormedia speech` turns text into an MP3 file with any speech model on OpenRouter.
The default is [Fish Audio S2.1 Pro](https://openrouter.ai/fish-audio/s2.1-pro).

```bash
ormedia speech --text "[excited] We are live!" --voice <id>
ormedia speech --text-file script.txt --voice-sample ./me.wav --voice-sample-text "The exact words in me.wav."
```

| Option                       | What it does                                                                                     |
| ---------------------------- | ------------------------------------------------------------------------------------------------ |
| `--text <text>`              | The words to say. Loose words after `speech` work too.                                           |
| `--text-file <path>`         | Read the words from a file                                                                       |
| `--voice <id>`               | A voice of the model. `ormedia models speech` lists the voices of a model that has a fixed list. |
| `--voice-sample <path\|url>` | Audio of a voice to copy. Repeat for more.                                                       |
| `--voice-sample-text <text>` | The words said in the sample. Repeat in the same order.                                          |
| `--instructions <text>`      | How to say it, for models that take it                                                           |
| `--speed <x>`                | For example `1.25`, for models that take it                                                      |
| `--format <f>`               | `mp3` or `pcm`. The default is `mp3`.                                                            |

Copy a voice only when it is yours, or when you have the right to use it.

`ormedia transcribe` writes down the words in an audio file, a video file, or a URL.

```bash
ormedia transcribe ./interview.mp3
ormedia transcribe ./clip.mp4 --timestamps segment --language en
```

| Option                 | What it does                                                      |
| ---------------------- | ----------------------------------------------------------------- |
| `--language <code>`    | For example `en`. Without it, the model detects the language.     |
| `--timestamps <level>` | `segment` or `word`: add the start and end time. Repeat for both. |
| `--speakers`           | Label who says each word, for models that can                     |
| `--format <f>`         | What the file holds, when its name does not say                   |
| `--json`               | Print the full reply                                              |

The words go to stdout and the cost goes to stderr, so `ormedia transcribe a.mp3 > a.txt` saves the words alone.

To dub a video, ask Claude, for example "add this narration to clip.mp4" or "dub clip.mp4 in Spanish".
Claude transcribes the video, translates the lines, makes the voice with `ormedia speech`, and joins it to the video with `ffmpeg`, which you must have installed.

## Prompt guides

Each model family wants a different kind of prompt.
The skill has one guide for each family in `skills/generate-media/prompts/`, taken from the maker's own documentation.
Claude reads the guide for the model in use before it writes the prompt.
`video-directing.md` covers the craft for any video model: the shot list, the still-first method, consistency between shots, sound, and how to spend less on retries.
`speech-fish-audio.md` covers the emotion cues and the voices of Fish Audio, and `speech-dubbing.md` covers how to put a voice on a video, and how to translate one.

## Settings

`ormedia config` shows every setting and where its value comes from.
Add `--json` to get that as JSON.

```bash
ormedia config set image-model openai/gpt-image-2
ormedia config set out ~/Pictures/generated
ormedia config unset image-model
```

| Setting            | What it does                                 | Default                            |
| ------------------ | -------------------------------------------- | ---------------------------------- |
| `image-model`      | Model for `ormedia image`                    | `google/gemini-nano-banana-2.1`    |
| `video-model`      | Model for `ormedia video`                    | `x-ai/grok-imagine-video-1.5-lite` |
| `speech-model`     | Model for `ormedia speech`                   | `fish-audio/s2.1-pro`              |
| `transcribe-model` | Model for `ormedia transcribe`               | `fish-audio/transcribe-1-pro`      |
| `out`              | Folder that results are saved to             | The current folder                 |
| `open`             | `true` or `false`: open a new image in Fleet | `true`                             |
| `timeout`          | Minutes to wait for a video                  | `20`                               |
| `update-check`     | `true` or `false`: look for a new version    | `true`                             |

Settings are stored in `~/.config/openrouter-media/config.json`.
A flag on the command line always wins over a saved setting.
`ORMEDIA_IMAGE_MODEL`, `ORMEDIA_VIDEO_MODEL`, `ORMEDIA_SPEECH_MODEL`, `ORMEDIA_TRANSCRIBE_MODEL`, and `ORMEDIA_UPDATE_CHECK` also win over the saved settings.
A relative `out` folder is relative to the folder you run the command in.

## Updates

One time per day, `ormedia` asks GitHub for the newest release.
When a newer one exists, it prints one line that says so, with the command that updates the plugin:

```
claude plugin update openrouter-media@openrouter-media
```

The check sends nothing about you, and never your API key.
It waits 3 seconds at most, and a check that fails says nothing.
Turn it off with `ormedia config set update-check false`.

Claude Code can also update the plugin for you.
Run `/plugin`, open **Marketplaces**, select `openrouter-media`, and select **Enable auto-update**.

## Videos take time

A video takes from 30 seconds to several minutes.
`ormedia video` prints the job id first, then waits up to 20 minutes, or the `timeout` setting.
If the wait ends early, the job still runs on OpenRouter.
Get the result later with `ormedia job <job-id> --wait`.

## Fleet

Inside a [Fleet](https://github.com/khang859/fleet) pane, a new image opens in a Fleet tab on its own.
Pass `--no-open` to stop that once, or run `ormedia config set open false` to stop it for good.
With `open` set to `false`, pass `--open` to open one result.

## Development

You need the latest stable [Rust](https://rustup.rs).

```bash
cargo test
cargo build --release
claude plugin validate .
claude --plugin-dir .
```

`bin/ormedia` runs `target/release/ormedia` when it exists, so a checkout uses your own build.

## License

MIT
