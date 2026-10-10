# openrouter-media

A Claude Code plugin that generates and edits images, and generates videos, through the OpenRouter API.
It ships the `ormedia` CLI and the `generate-media` skill.

## Stack

Plain Node.js ES modules (`.mjs`), Node 22 or newer, and no dependencies at all.
There is no build step: the files in the repo are the files that run.
Keep it that way, because a plugin is installed by copying the repo.

## Layout

- `.claude-plugin/plugin.json` - the plugin manifest
- `.claude-plugin/marketplace.json` - makes this repo its own marketplace, with the plugin at `./`
- `bin/ormedia` - the launcher that Claude Code puts on the Bash PATH. It is CommonJS on purpose and only imports `src/cli.mjs`.
- `src/cli.mjs` - argument parsing, one function per command, and all printing
- `src/api.mjs` - every OpenRouter HTTP call, plus the pure request-body builders
- `src/auth.mjs` - key storage and the PKCE browser sign-in
- `src/files.mjs` - reference images, output names, saving, and opening in Fleet
- `src/models.mjs` - turns a model record into text for `ormedia models`
- `skills/generate-media/SKILL.md` - what Claude reads to learn the CLI
- `test/` - `node:test` files, one per source module

## Verification commands

- **Test:** `npm test`
- **Validate the plugin:** `claude plugin validate .`
- **Try it in Claude Code:** `claude --plugin-dir .`
- **Run the CLI directly:** `bin/ormedia --help`

CI (`.github/workflows/ci.yml`) runs `npm test` on Node 22, 24, and 26.

## Rules

- Never add a runtime dependency.
- Never print, log, or echo an API key. `ormedia status` shows only the label that OpenRouter returns.
- Never overwrite a file the user already has. `saveOutputs` picks a free name instead.
- Results go to stdout, progress goes to stderr, so `--json` output stays clean.
- Send an option to the API only when the user set it. An unset option means "whatever the model does".
- Do not validate option values in the CLI. The API rejects a bad value and lists the valid ones, and the model lists change often.
- The network code takes `fetchImpl` as a parameter so tests never call the real API.
- Add or update a test for every behavior change.

## When you change the CLI

A new or changed flag must be updated in four places: `src/cli.mjs`, the `HELP` text in the same file, `skills/generate-media/SKILL.md`, and `README.md`.

## Real API checks

Unit tests use a fake `fetch`, so they cannot catch a change in the OpenRouter API.
After you change a request or a response path, run the real command once.
An image costs about $0.04.
A video costs real money per second, so use the shortest duration and the lowest resolution, and ask the user first.

Check the API shape in the OpenRouter docs before you change a call:

- Images: <https://openrouter.ai/docs/guides/overview/multimodal/image-generation>
- Videos: <https://openrouter.ai/docs/guides/overview/multimodal/video-generation>
- Sign-in: <https://openrouter.ai/docs/guides/overview/auth/oauth>

## Commits and releases

Use conventional commits with a scope, for example `fix(video): ...`.
`main` is protected: every change is a pull request, CI must pass, and merges are squashed.
For a release, set the same version in `package.json` and `.claude-plugin/plugin.json`, merge, then push a `vX.Y.Z` tag.
Release tags cannot be moved or deleted after the push.
