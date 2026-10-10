import assert from 'node:assert/strict';
import { test } from 'node:test';
import { createClient, imageBody, videoBody } from '../src/api.mjs';

const json = (body, status = 200) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' }
  });

/** A fetch that answers from a queue and remembers what it was asked. */
function fakeFetch(...responses) {
  const calls = [];
  const fetchImpl = async (url, init) => {
    calls.push({ url, init });
    return responses.shift();
  };
  return { fetchImpl, calls };
}

test('imageBody sends only what was set', () => {
  assert.deepEqual(imageBody({ model: 'm', prompt: 'a cat', references: [] }), {
    model: 'm',
    prompt: 'a cat',
    stream: false
  });
});

test('imageBody maps options and references to API names', () => {
  const body = imageBody({
    model: 'm',
    prompt: 'p',
    references: ['https://x/a.png'],
    aspectRatio: '16:9',
    format: 'webp',
    seed: 0,
    n: 2
  });
  assert.equal(body.aspect_ratio, '16:9');
  assert.equal(body.output_format, 'webp');
  assert.equal(body.seed, 0);
  assert.equal(body.n, 2);
  assert.deepEqual(body.input_references, [
    { type: 'image_url', image_url: { url: 'https://x/a.png' } }
  ]);
});

test('videoBody tags frames and keeps an explicit audio off', () => {
  const body = videoBody({
    model: 'm',
    prompt: 'p',
    firstFrame: 'data:a',
    lastFrame: 'data:b',
    references: [],
    audio: false
  });
  assert.equal(body.generate_audio, false);
  assert.deepEqual(
    body.frame_images.map((frame) => [frame.frame_type, frame.image_url.url]),
    [
      ['first_frame', 'data:a'],
      ['last_frame', 'data:b']
    ]
  );
  assert.equal('input_references' in body, false);
});

test('generateImages decodes every image and reports the cost', async () => {
  const { fetchImpl, calls } = fakeFetch(
    json({
      data: [{ b64_json: Buffer.from('one').toString('base64'), media_type: 'image/webp' }],
      usage: { cost: 0.04 }
    })
  );
  const result = await createClient({ apiKey: 'k', fetchImpl }).generateImages({ model: 'm' });
  assert.equal(result.images[0].data.toString(), 'one');
  assert.equal(result.images[0].mimeType, 'image/webp');
  assert.equal(result.costUsd, 0.04);
  assert.equal(calls[0].url, 'https://openrouter.ai/api/v1/images');
  assert.equal(calls[0].init.headers.Authorization, 'Bearer k');
});

test('an API error surfaces its message, and a 401 says how to sign in', async () => {
  const { fetchImpl } = fakeFetch(json({ error: { message: 'No auth' } }, 401));
  await assert.rejects(
    createClient({ fetchImpl }).keyInfo(),
    /No auth \(HTTP 401\)\. Run `ormedia login`/
  );
});

test('waitForVideo polls until the job completes', async () => {
  const { fetchImpl, calls } = fakeFetch(
    json({ id: 'j', status: 'pending' }),
    json({ id: 'j', status: 'in_progress' }),
    json({ id: 'j', status: 'completed', unsigned_urls: ['u'] })
  );
  const seen = [];
  const job = await createClient({ apiKey: 'k', fetchImpl }).waitForVideo('j', {
    timeoutMs: 1000,
    intervalMs: 1,
    onStatus: (status) => seen.push(status)
  });
  assert.equal(job.status, 'completed');
  assert.deepEqual(seen, ['pending', 'in_progress', 'completed']);
  assert.equal(calls.length, 3);
});

test('waitForVideo fails with the reason the job gives', async () => {
  const { fetchImpl } = fakeFetch(json({ id: 'j', status: 'failed', error: 'blocked prompt' }));
  await assert.rejects(
    createClient({ fetchImpl }).waitForVideo('j', { timeoutMs: 1000, intervalMs: 1 }),
    /blocked prompt/
  );
});

test('waitForVideo gives up at the deadline and says how to resume', async () => {
  const { fetchImpl } = fakeFetch(json({ id: 'j', status: 'pending' }));
  await assert.rejects(
    createClient({ fetchImpl }).waitForVideo('j', { timeoutMs: 5, intervalMs: 10 }),
    /ormedia job j --wait/
  );
});
