---
name: models
description: List the OpenRouter image or video models, with the options and the price of each one.
argument-hint: "<image|video> [search text]"
disable-model-invocation: true
allowed-tools: Bash(ormedia models *)
---

# List the models

The user typed this after the command: `$ARGUMENTS`

The first word is `image` or `video`.
If it is missing, ask the user which one they want.
The other words are a search text.

Run `ormedia models <image|video>`, and add `--search "<text>"` when there is a search text.
Show the output as it is.
To make one of them the default, tell the user to run `/openrouter-media:config set image-model <id>` or `/openrouter-media:config set video-model <id>`.
