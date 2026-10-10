import assert from 'node:assert/strict';
import { test } from 'node:test';
import { describeModel } from '../src/models.mjs';

test('an image model lists each option in its own shape', () => {
  const text = describeModel('image', {
    id: 'a/b',
    name: 'B',
    supported_parameters: {
      resolution: { type: 'enum', values: ['1K', '2K'] },
      n: { type: 'range', min: 1, max: 4 },
      seed: { type: 'boolean' }
    }
  });
  assert.equal(text, 'a/b (B)\n  resolution: 1K, 2K\n  n: 1-4\n  seed');
});

test('a video model skips what it does not support', () => {
  const text = describeModel('video', {
    id: 'v/x',
    name: 'X',
    supported_durations: [4, 8],
    supported_resolutions: null,
    generate_audio: true,
    seed: null,
    pricing_skus: { duration_seconds: '0.1' }
  });
  assert.equal(text, 'v/x (X)\n  duration: 4, 8 s\n  audio\n  price duration_seconds: 0.1');
});
