# Contributing

Thanks for your help.

## Set up

You need Node.js 22 or newer.
There is nothing to install, because the plugin has no dependencies.

```bash
git clone https://github.com/khang859/openrouter-media
cd openrouter-media
npm test
claude --plugin-dir .
```

## Rules

- Keep the plugin free of runtime dependencies.
- Add or update a test in `test/` for every behavior change.
- Run `npm test` and `claude plugin validate .` before you open a pull request.
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
