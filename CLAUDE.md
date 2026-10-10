# openrouter-media

A Claude Code plugin that generates and edits images, and generates videos, through the OpenRouter API.
It ships the `ormedia` CLI and the `generate-media` skill.

## Stack

Rust, on the latest stable toolchain, edition 2024.
The program is one binary named `ormedia`.
The dependencies are few on purpose: `clap`, `ureq`, `serde_json`, `base64`, `sha2`, and `getrandom`.

A plugin is installed by copying the repo, and the repo holds no compiled program.
So `bin/ormedia` is a POSIX shell launcher.
It runs `target/release/ormedia` when a checkout has one.
Otherwise it downloads the build for the plugin version from the GitHub release, checks its SHA-256, and keeps it in `~/.cache/openrouter-media/`.

## Layout

- `.claude-plugin/plugin.json` - the plugin manifest
- `.claude-plugin/marketplace.json` - makes this repo its own marketplace, with the plugin at `./`
- `bin/ormedia` - the launcher that Claude Code puts on the Bash PATH. It finds or downloads the program, then runs it.
- `Cargo.toml`, `Cargo.lock` - the package and its locked dependencies
- `src/main.rs` - the module list and the `Result` type
- `src/cli.rs` - the `clap` argument types, one function per command, and all printing
- `src/api.rs` - every OpenRouter HTTP call, plus the pure request-body builders
- `src/auth.rs` - key storage and the PKCE browser sign-in
- `src/config.rs` - the settings table, the config file, and how a setting is resolved
- `src/env.rs` - the environment variables and the config folder, as a value that tests can fake
- `src/files.rs` - reference images, output names, saving, and opening in Fleet
- `src/models.rs` - turns a model record into text for `ormedia models`
- `src/update.rs` - the once-a-day check for a newer release
- `skills/generate-media/SKILL.md` - what Claude reads to learn the CLI
- `tests/cli.rs` - runs the real program, the way a user does
- `.github/workflows/release.yml` - builds the program for each platform when a `vX.Y.Z` tag is pushed

Unit tests live at the bottom of the source file they test, in a `tests` module.

## Verification commands

- **Test:** `cargo test`
- **Lint:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `shellcheck bin/ormedia`
- **Build:** `cargo build --release`
- **Validate the plugin:** `claude plugin validate .`
- **Try it in Claude Code:** `claude --plugin-dir .`
- **Run the CLI directly:** `bin/ormedia --help`

CI (`.github/workflows/ci.yml`) runs the lint commands on Linux, and the tests on Linux, macOS, and Windows.

## Rules

- Add a dependency only when the standard library cannot do the job well. Use the latest version of a well maintained crate.
- No `unsafe` code. `Cargo.toml` forbids it.
- An error is a sentence for the user. `main` prints it after `ormedia: ` and exits with 1.
- Never print, log, or echo an API key. `ormedia status` shows only the label that OpenRouter returns.
- Never overwrite a file the user already has. `save_outputs` picks a free name instead.
- Results go to stdout, progress goes to stderr, so `--json` output stays clean.
- Send an option to the API only when the user set it. An unset option means "whatever the model does".
- Do not validate option values in the CLI. The API rejects a bad value and lists the valid ones, and the model lists change often.
- The network code takes a `Transport` as a parameter, and the storage code takes an `Env`, so tests never call the real API or touch the real home folder.
- Add or update a test for every behavior change.

## When you change the CLI

A new or changed flag must be updated in four places: the `clap` types in `src/cli.rs`, the `help()` text in the same file, `skills/generate-media/SKILL.md`, and `README.md`.

## Real API checks

Unit tests use a fake `Transport`, so they cannot catch a change in the OpenRouter API.
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
For a release, set the same version in `Cargo.toml` and `.claude-plugin/plugin.json`, run `cargo build` to update `Cargo.lock`, merge, then push a `vX.Y.Z` tag.
A test fails when the two versions differ.
The tag starts `release.yml`, which builds the program for each platform and publishes the GitHub release.
Push the tag right after the merge: until the release is published, a newly installed plugin has no program to download.
Release tags cannot be moved or deleted after the push.
