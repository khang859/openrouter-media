# Contributing

Thanks for your help.

## Set up

You need the latest stable [Rust](https://rustup.rs).

```bash
git clone https://github.com/khang859/openrouter-media
cd openrouter-media
cargo test
cargo build --release
claude --plugin-dir .
```

## Rules

- Keep the dependencies few. Add one only when the standard library cannot do the job well.
- Add or update a test for every behavior change. Unit tests sit in the source file they test, and `tests/cli.rs` runs the real program.
- Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `claude plugin validate .` before you open a pull request.
- If you change an API call, run the changed command against the real OpenRouter API once.
- Never commit an API key, and never print one in a log or an issue.
- Use [conventional commits](https://www.conventionalcommits.org) with a scope, for example `fix(video): ...`.

## Pull requests

Every change to `main` goes through a pull request, and CI must pass.
Pull requests are squashed when they merge.
Open an issue first for a large change, so we agree on the direction before you write code.

## Security

Report a vulnerability in private.
See [SECURITY.md](SECURITY.md).
