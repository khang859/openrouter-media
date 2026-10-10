import assert from 'node:assert/strict';
import { stat, mkdtemp } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { authorize, deleteKey, readKey, saveKey } from '../src/auth.mjs';

const tempEnv = async () => ({ XDG_CONFIG_HOME: await mkdtemp(join(tmpdir(), 'ormedia-')) });

test('a stored key round-trips, is private, and can be deleted', async () => {
  const env = await tempEnv();
  assert.equal(await readKey(env), null);
  const path = await saveKey('sk-stored', env);
  assert.deepEqual(await readKey(env), { key: 'sk-stored', source: path });
  if (process.platform !== 'win32') assert.equal((await stat(path)).mode & 0o777, 0o600);
  await deleteKey(env);
  assert.equal(await readKey(env), null);
});

test('the environment key wins over the stored one', async () => {
  const env = await tempEnv();
  await saveKey('sk-stored', env);
  const found = await readKey({ ...env, OPENROUTER_API_KEY: 'sk-env' });
  assert.deepEqual(found, { key: 'sk-env', source: 'OPENROUTER_API_KEY' });
});

test('authorize returns the code from the callback and ignores a forged one', async () => {
  const result = await authorize({
    log: () => {},
    openBrowser: async (authUrl) => {
      const params = new URL(authUrl).searchParams;
      assert.equal(params.get('code_challenge_method'), 'S256');
      const callback = params.get('callback_url');
      const forged = await fetch(`${callback}?code=evil&state=wrong`);
      assert.equal(forged.status, 400);
      await fetch(`${callback}?code=real&state=${params.get('state')}`);
    }
  });
  assert.equal(result.code, 'real');
  assert.ok(result.verifier.length >= 43);
});

test('authorize times out when nobody signs in', async () => {
  await assert.rejects(
    authorize({ log: () => {}, openBrowser: () => {}, timeoutMs: 20 }),
    /timed out/
  );
});
