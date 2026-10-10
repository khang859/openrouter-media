---
name: generate-media
description: Generate or edit images, and generate videos, with any OpenRouter model through the `ormedia` CLI. Use when the user asks to create, draw, render, design, edit, or restyle an image, logo, icon, illustration, or mockup, or to make, animate, or generate a video or clip.
---

# Generate media with OpenRouter

The `ormedia` command is on your PATH.
It saves each result to a file and prints the path.
Run `ormedia --help` for every option.

## Sign in

Run `ormedia status` when a command says there is no API key.
If the user is not signed in, run `ormedia login` with a Bash timeout of 300000 ms.
It opens the browser, the user approves, and the key is stored on disk.
Never ask the user to paste an API key into the chat.

## Images

```bash
ormedia image --prompt "A weathered lighthouse at dusk, oil painting, thick brushwork" --aspect-ratio 16:9 --out assets/
ormedia image --prompt "Change only the sky to a sunset. Keep the building the same." --image ./photo.png
```

- `--image` takes a local path or a URL, and repeats for several references.
- `--out` takes a file name or a folder. An existing file is never overwritten.
- After the command returns, read the saved file to check the result before you report it.
- Inside Fleet the image opens in a Fleet tab on its own. Pass `--no-open` to stop that.

## Videos

Video costs real money per second, so do these steps in order:

1. Run `ormedia models video --search <name>` to see the durations, resolutions, and prices the model takes.
2. Tell the user the model, the duration, and the estimated price, and get a yes before you start.
3. Run the command in the background, because a video takes from 30 seconds to several minutes.

```bash
ormedia video --prompt "Slow dolly shot through a neon-lit alley in the rain" --duration 4 --resolution 720p --out clips/
ormedia video --prompt "The boat sails away" --first-frame ./boat.png --duration 6
```

The command prints `Video job <id> started` first.
If the command stops before the video is saved, the job still runs on OpenRouter.
Get it with `ormedia job <id> --wait`.
Do not submit the same video again, because that pays twice.

## Choosing a model

Use the default model unless the user names one or the task needs something the default lacks.
`ormedia models image` and `ormedia models video` list every model with the options it accepts.
Send only options the chosen model lists.
The API rejects an unsupported value and names the valid ones, so read the error and retry once.

## Writing the prompt

- Write plain descriptive sentences in this order: scene, subject, key details, style, lighting, camera.
- Name the finished artifact, for example "an editorial product photograph" or "a flat vector app icon".
- Put literal text in quotes and state its font, weight, colour, and place.
- For an edit, say what changes and list what must stay the same.
- For a video, describe one continuous shot: the subject, its motion, and the camera move.
- Skip quality tags such as "4k, masterpiece". They do nothing on current models.
