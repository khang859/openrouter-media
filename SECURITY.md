# Security policy

## Report a vulnerability

Do not open a public issue for a security problem.
Report it in private at <https://github.com/khang859/openrouter-media/security/advisories/new>.

Include the steps to reproduce the problem and the version you use.
You get a first answer within 7 days.

## Supported versions

Only the latest release gets security fixes.

## How the plugin handles your API key

- `ormedia login` uses the OpenRouter PKCE sign-in, so the key is never typed, pasted, or printed.
- The key is stored in `~/.config/openrouter-media/credentials.json` with mode `0600`.
- The key is sent only to `https://openrouter.ai`.
- The program has no `unsafe` code and few dependencies, and `Cargo.lock` pins each one.

## How the plugin gets its program

- `bin/ormedia` downloads the program for the plugin version from this repository's GitHub release, over HTTPS only.
- It checks the download against the SHA-256 file from the same release before it runs it.
- The release is built by `.github/workflows/release.yml` from the tagged commit.

If you think a key leaked, delete it at <https://openrouter.ai/settings/keys> and run `ormedia login` again.
