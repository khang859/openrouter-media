# openrouter-media

A Claude Code plugin that generates and edits images, and generates videos, with any model on [OpenRouter](https://openrouter.ai).

It adds three things:

- `ormedia`, a small command-line tool written in Rust.
- A `generate-media` skill that tells Claude when and how to use it.
- Slash commands, such as `/openrouter-media:login`, for the things you do yourself.

## Install

```
/plugin marketplace add khang859/openrouter-media
/plugin install openrouter-media@openrouter-media
```

Then ask Claude for an image or a video.
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

| Command                                         | What it does                                |
| ----------------------------------------------- | ------------------------------------------- |
| `/openrouter-media:login`                       | Sign in with the browser                    |
| `/openrouter-media:logout`                      | Delete the stored API key                   |
| `/openrouter-media:status`                      | Show which key is in use and its spend      |
| `/openrouter-media:models <image\|video> [text]` | List models, and keep those that match text |
| `/openrouter-media:config`                      | Show the settings                           |
| `/openrouter-media:config set <name> <value>`   | Save a setting                              |
| `/openrouter-media:config unset <name>`         | Remove a saved setting                      |

Each one runs the `ormedia` command of the same name.
Claude never starts these on its own.
For an image or a video, just ask Claude.

## Commands

| Command                         | What it does                                          |
| ------------------------------- | ----------------------------------------------------- |
| `ormedia login`                 | Sign in with the browser and store an API key         |
| `ormedia logout`                | Delete the stored API key                             |
| `ormedia status`                | Show which key is in use and its spend                |
| `ormedia models <image\|video>` | List models and the options each one takes            |
| `ormedia image --prompt <text>` | Generate or edit an image                             |
| `ormedia video --prompt <text>` | Generate a video                                      |
| `ormedia job <job-id>`          | Check a video job, and download it when it is done    |
| `ormedia config`                | Show the settings, or save one with `set` and `unset` |

Run `ormedia --help` for every option.

```bash
ormedia image --prompt "A flat vector fox logo, orange on white" --aspect-ratio 1:1 --out assets/
ormedia image --prompt "Make it night. Keep everything else." --image ./street.png
ormedia video --prompt "Waves roll onto a black sand beach, slow push in" --duration 4
```

Results are saved to the current folder, or to `--out`, or to the `out` setting.
A file that already exists is never overwritten.
Add `--json` to get the result as JSON.

## Models

The defaults are `google/gemini-nano-banana-2.1` for images and `x-ai/grok-imagine-video-1.5-lite` for videos.
Change them per call with `--model`, or for good with `ormedia config`.

## Settings

`ormedia config` shows every setting and where its value comes from.
Add `--json` to get that as JSON.

```bash
ormedia config set image-model openai/gpt-image-2
ormedia config set out ~/Pictures/generated
ormedia config unset image-model
```

| Setting        | What it does                                 | Default                            |
| -------------- | -------------------------------------------- | ---------------------------------- |
| `image-model`  | Model for `ormedia image`                    | `google/gemini-nano-banana-2.1`    |
| `video-model`  | Model for `ormedia video`                    | `x-ai/grok-imagine-video-1.5-lite` |
| `out`          | Folder that results are saved to             | The current folder                 |
| `open`         | `true` or `false`: open a new image in Fleet | `true`                             |
| `timeout`      | Minutes to wait for a video                  | `20`                               |
| `update-check` | `true` or `false`: look for a new version    | `true`                             |

Settings are stored in `~/.config/openrouter-media/config.json`.
A flag on the command line always wins over a saved setting.
`ORMEDIA_IMAGE_MODEL`, `ORMEDIA_VIDEO_MODEL`, and `ORMEDIA_UPDATE_CHECK` also win over the saved settings.
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
