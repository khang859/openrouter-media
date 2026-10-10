# openrouter-media

A Claude Code plugin that generates and edits images, and generates videos, with any model on [OpenRouter](https://openrouter.ai).

It adds two things:

- `ormedia`, a small command-line tool with no dependencies.
- A `generate-media` skill that tells Claude when and how to use it.

## Install

You need Node.js 22 or newer.

```
/plugin marketplace add khang859/openrouter-media
/plugin install openrouter-media@openrouter-media
```

Then ask Claude for an image or a video.
The first time, Claude runs `ormedia login`, your browser opens, and you approve.
The API key is stored in `~/.config/openrouter-media/credentials.json`, readable only by you.
You never paste a key into the chat.

To use a key you already have, set `OPENROUTER_API_KEY` instead.
It wins over the stored key.

## Commands

| Command                         | What it does                                         |
| ------------------------------- | ---------------------------------------------------- |
| `ormedia login`                 | Sign in with the browser and store an API key        |
| `ormedia logout`                | Delete the stored API key                            |
| `ormedia status`                | Show which key is in use and its spend               |
| `ormedia models <image\|video>` | List models and the options each one takes           |
| `ormedia image --prompt <text>` | Generate or edit an image                            |
| `ormedia video --prompt <text>` | Generate a video                                     |
| `ormedia job <job-id>`          | Check a video job, and download it when it is done   |

Run `ormedia --help` for every option.

```bash
ormedia image --prompt "A flat vector fox logo, orange on white" --aspect-ratio 1:1 --out assets/
ormedia image --prompt "Make it night. Keep everything else." --image ./street.png
ormedia video --prompt "Waves roll onto a black sand beach, slow push in" --duration 4
```

Results are saved to the current folder, or to `--out`.
A file that already exists is never overwritten.
Add `--json` to get the result as JSON.

## Models

The defaults are `google/gemini-nano-banana-2.1` for images and `x-ai/grok-imagine-video-1.5-lite` for videos.
Change them per call with `--model`, or for good with `ORMEDIA_IMAGE_MODEL` and `ORMEDIA_VIDEO_MODEL`.

## Videos take time

A video takes from 30 seconds to several minutes.
`ormedia video` prints the job id first, then waits up to 20 minutes.
If the wait ends early, the job still runs on OpenRouter.
Get the result later with `ormedia job <job-id> --wait`.

## Fleet

Inside a [Fleet](https://github.com/khang859/fleet) pane, a new image opens in a Fleet tab on its own.
Pass `--no-open` to stop that.

## Development

```bash
npm test
claude plugin validate .
claude --plugin-dir .
```

## License

MIT
