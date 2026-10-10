import assert from 'node:assert/strict';
import { mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import {
  extensionFor,
  freePath,
  outputPath,
  saveOutputs,
  slug,
  timestamp,
  toImageUrl
} from '../src/files.mjs';

test('toImageUrl passes URLs through and inlines local files', async () => {
  assert.equal(await toImageUrl('https://x/a.png'), 'https://x/a.png');
  assert.equal(await toImageUrl('data:image/png;base64,AA'), 'data:image/png;base64,AA');
  const file = join(await mkdtemp(join(tmpdir(), 'ormedia-')), 'ref.JPG');
  await writeFile(file, 'pixels');
  assert.equal(
    await toImageUrl(file),
    `data:image/jpeg;base64,${Buffer.from('pixels').toString('base64')}`
  );
});

test('toImageUrl rejects a file that is not an image', async () => {
  await assert.rejects(toImageUrl('notes.txt'), /Unsupported image type/);
});

test('extensionFor reads the media type, with or without parameters', () => {
  assert.equal(extensionFor('image/jpeg'), '.jpg');
  assert.equal(extensionFor('video/mp4; codecs=avc1'), '.mp4');
  assert.equal(extensionFor('application/x-unknown'), '.bin');
});

test('slug makes a short safe file name', () => {
  assert.equal(slug('A Cat, in SPACE!'), 'a-cat-in-space');
  assert.equal(slug('!!!'), 'media');
  assert.ok(slug('word '.repeat(40)).length <= 40);
});

test('outputPath treats a name with an extension as a file, anything else as a folder', () => {
  const at = (out, index = 0) => outputPath({ out, stem: 's', index, ext: '.jpg', cwd: '/w' });
  assert.equal(at(undefined), '/w/s.jpg');
  assert.equal(at('art'), '/w/art/s.jpg');
  assert.equal(at('art', 1), '/w/art/s-2.jpg');
  // The real extension wins over the one that was asked for.
  assert.equal(at('art/hero.png'), '/w/art/hero.jpg');
  assert.equal(at('/abs/hero.png', 1), '/abs/hero-2.jpg');
});

test('freePath steps past names that are taken', () => {
  const taken = new Set(['/w/a.png', '/w/a-2.png']);
  assert.equal(freePath('/w/b.png', (path) => taken.has(path)), '/w/b.png');
  assert.equal(freePath('/w/a.png', (path) => taken.has(path)), '/w/a-3.png');
});

test('saveOutputs writes into a new folder and never overwrites', async () => {
  const dir = join(await mkdtemp(join(tmpdir(), 'ormedia-')), 'nested');
  const output = (text) => [{ data: Buffer.from(text), mimeType: 'image/png' }];
  const first = await saveOutputs(output('first'), { out: dir, stem: 'pic' });
  const second = await saveOutputs(output('second'), { out: dir, stem: 'pic' });
  assert.deepEqual(first, [join(dir, 'pic.png')]);
  assert.deepEqual(second, [join(dir, 'pic-2.png')]);
  assert.equal(await readFile(first[0], 'utf8'), 'first');
});

test('timestamp is sortable and file-name safe', () => {
  assert.equal(timestamp(new Date('2026-10-10T13:05:09Z')), '20261010-130509');
});
