---
name: models
description: List the OpenRouter image, video, speech, or transcribe models, with the options and the price of each one.
argument-hint: "<image|video|speech|transcribe> [search text]"
disable-model-invocation: true
allowed-tools: Bash(ormedia models *)
---

# List the models

The user typed this after the command: `$ARGUMENTS`

The first word is `image`, `video`, `speech`, or `transcribe`.
If it is missing, ask the user which one they want.
The other words are a search text.

Run `ormedia models <image|video|speech|transcribe>`, and add `--search "<text>"` when there is a search text.
Show the output as it is.
To make one of them the default, tell the user to run `/openrouter-media:config set image-model <id>`, with `video-model`, `speech-model`, or `transcribe-model` for the other kinds.
