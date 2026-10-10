---
name: login
description: Sign in to OpenRouter with the browser, so `ormedia` can generate images and videos.
disable-model-invocation: true
allowed-tools: Bash(ormedia login), Bash(ormedia status)
---

# Sign in to OpenRouter

Run `ormedia status` first.
If it says the user is signed in, show that output and stop.

Otherwise run `ormedia login` with a Bash timeout of 300000 ms.
It opens the browser, the user approves, and the key is stored on disk.
Tell the user to look at the browser while the command waits.

When it ends, show its output.
Never ask the user to paste an API key into the chat, and never print one.
