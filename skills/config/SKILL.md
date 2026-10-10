---
name: config
description: Show or change the `ormedia` settings, such as the default image model and video model.
argument-hint: "[set <name> <value> | unset <name>]"
disable-model-invocation: true
allowed-tools: Bash(ormedia config), Bash(ormedia config *)
---

# Show or change the settings

The user typed this after the command: `$ARGUMENTS`

- With nothing typed, run `ormedia config` and show its output as it is.
- With `set <name> <value>` or `unset <name>`, run `ormedia config` with those same words, then run `ormedia config` to show the result.
- With anything else, for example "use the cheapest video model", work out the setting and the value, then do the same.
  Use `ormedia models image` or `ormedia models video` to find a model id.

The settings are `image-model`, `video-model`, `out`, `open`, `timeout`, and `update-check`.
If the command fails, show the error and stop.
