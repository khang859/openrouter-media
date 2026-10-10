import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { dirname, extname, resolve } from 'node:path';

const IMAGE_MIME_BY_EXT = {
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.webp': 'image/webp',
  '.gif': 'image/gif'
};

const EXT_BY_MIME = {
  'image/png': '.png',
  'image/jpeg': '.jpg',
  'image/webp': '.webp',
  'image/gif': '.gif',
  'image/svg+xml': '.svg',
  'video/mp4': '.mp4',
  'video/webm': '.webm',
  'video/quicktime': '.mov'
};

/** A reference image as the API wants it: a URL as is, a local file as a data URL. */
export async function toImageUrl(ref) {
  if (/^(https?:|data:)/i.test(ref)) return ref;
  const mime = IMAGE_MIME_BY_EXT[extname(ref).toLowerCase()];
  if (mime === undefined) {
    throw new Error(`Unsupported image type: ${ref} (use png, jpg, webp, or gif)`);
  }
  const data = await readFile(ref);
  return `data:${mime};base64,${data.toString('base64')}`;
}

export function extensionFor(mimeType) {
  return EXT_BY_MIME[mimeType.split(';')[0].trim().toLowerCase()] ?? '.bin';
}

export function slug(text) {
  const words = text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  return words.slice(0, 40).replace(/-+$/, '') || 'media';
}

/**
 * Where output number `index` goes. `out` with an extension names the file,
 * anything else names a folder. The extension always comes from what was
 * actually returned, so a `.png` name never hides a JPEG.
 */
export function outputPath({ out, stem, index, ext, cwd = process.cwd() }) {
  const suffix = index === 0 ? '' : `-${index + 1}`;
  if (out !== undefined && extname(out) !== '') {
    const base = out.slice(0, -extname(out).length);
    return resolve(cwd, `${base}${suffix}${ext}`);
  }
  return resolve(cwd, out ?? '.', `${stem}${suffix}${ext}`);
}

/** The path itself, or the first `name-2`, `name-3`... that is not taken. */
export function freePath(path, exists = existsSync) {
  if (!exists(path)) return path;
  const ext = extname(path);
  const base = path.slice(0, -ext.length || undefined);
  for (let n = 2; ; n++) {
    const candidate = `${base}-${n}${ext}`;
    if (!exists(candidate)) return candidate;
  }
}

/** Writes each output and returns where it went. Never overwrites a file. */
export async function saveOutputs(outputs, { out, stem }) {
  const paths = [];
  for (const [index, output] of outputs.entries()) {
    const path = freePath(
      outputPath({ out, stem, index, ext: extensionFor(output.mimeType) })
    );
    await mkdir(dirname(path), { recursive: true });
    await writeFile(path, output.data, { flag: 'wx' });
    paths.push(path);
  }
  return paths;
}

/** Fire and forget: a viewer that fails to start must not fail the command. */
export function launch(command, args) {
  const child = spawn(command, args, { detached: true, stdio: 'ignore' });
  child.on('error', () => {});
  child.unref();
}

export function openUrl(url) {
  if (process.platform === 'darwin') launch('open', [url]);
  else if (process.platform === 'win32') launch('cmd', ['/c', 'start', '', url]);
  else launch('xdg-open', [url]);
}

/** Inside a Fleet pane, show images in a Fleet tab. Fleet has no video viewer. */
export function openInFleet(paths, env = process.env) {
  const images = paths.filter((path) => extname(path).toLowerCase() in IMAGE_MIME_BY_EXT);
  if (env.FLEET_SESSION !== '1' || images.length === 0) return;
  launch('fleet', ['open', ...images]);
}

export function timestamp(now = new Date()) {
  return now.toISOString().replace(/[-:]/g, '').replace('T', '-').slice(0, 15);
}
