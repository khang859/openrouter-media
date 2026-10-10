import { createHash, randomBytes } from 'node:crypto';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { homedir } from 'node:os';
import { dirname, join } from 'node:path';

const page = (text) =>
  `<!doctype html><meta charset="utf-8"><title>OpenRouter Media</title><body style="font-family:system-ui;padding:3rem"><h1>${text}</h1>`;

export function credentialsPath(env = process.env) {
  const configHome = env.XDG_CONFIG_HOME || join(homedir(), '.config');
  return join(configHome, 'openrouter-media', 'credentials.json');
}

/** The key and where it came from. The environment wins over the stored key. */
export async function readKey(env = process.env) {
  if (env.OPENROUTER_API_KEY) return { key: env.OPENROUTER_API_KEY, source: 'OPENROUTER_API_KEY' };
  const path = credentialsPath(env);
  try {
    const { key } = JSON.parse(await readFile(path, 'utf8'));
    if (typeof key === 'string' && key !== '') return { key, source: path };
  } catch {
    // No file, or one that is not ours to read: the same as not signed in.
  }
  return null;
}

export async function saveKey(key, env = process.env) {
  const path = credentialsPath(env);
  await mkdir(dirname(path), { recursive: true, mode: 0o700 });
  await writeFile(path, `${JSON.stringify({ key }, null, 2)}\n`, { mode: 0o600 });
  return path;
}

export async function deleteKey(env = process.env) {
  await rm(credentialsPath(env), { force: true });
}

/**
 * OpenRouter's PKCE sign-in. The browser sends the code back to a server that
 * lives only for this call, so the key is never typed, pasted, or printed.
 * Resolves to what the code exchange needs.
 */
export async function authorize({ openBrowser, log, timeoutMs = 5 * 60_000 }) {
  const verifier = randomBytes(32).toString('base64url');
  const challenge = createHash('sha256').update(verifier).digest('base64url');
  const state = randomBytes(16).toString('base64url');

  const server = createServer();
  await new Promise((resolve, reject) => {
    server.once('error', reject).listen(0, 'localhost', resolve);
  });
  const callbackUrl = `http://localhost:${server.address().port}/callback`;
  const authUrl = `https://openrouter.ai/auth?${new URLSearchParams({
    callback_url: callbackUrl,
    code_challenge: challenge,
    code_challenge_method: 'S256',
    state,
    key_label: 'openrouter-media'
  })}`;

  const timer = { id: undefined };
  try {
    const code = await new Promise((resolve, reject) => {
      timer.id = setTimeout(() => reject(new Error('Sign-in timed out')), timeoutMs);
      server.on('request', (req, res) => {
        const url = new URL(req.url, callbackUrl);
        if (url.pathname !== '/callback') {
          res.writeHead(404).end();
          return;
        }
        const code = url.searchParams.get('code');
        const ok = code !== null && url.searchParams.get('state') === state;
        res
          .writeHead(ok ? 200 : 400, { 'Content-Type': 'text/html' })
          .end(page(ok ? 'Signed in. You can close this tab.' : 'Sign-in failed. Try again.'));
        // A request that is not ours is answered and ignored, not fatal.
        if (ok) resolve(code);
      });
      log(`Open this link to sign in to OpenRouter:\n${authUrl}`);
      openBrowser(authUrl);
    });
    return { code, verifier };
  } finally {
    clearTimeout(timer.id);
    server.close();
    server.closeAllConnections();
  }
}
