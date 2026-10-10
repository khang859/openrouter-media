import { parseArgs } from 'node:util';
import { createClient, imageBody, videoBody } from './api.mjs';
import { authorize, deleteKey, readKey, saveKey } from './auth.mjs';
import { openInFleet, openUrl, saveOutputs, slug, timestamp, toImageUrl } from './files.mjs';
import { describeModel } from './models.mjs';

const DEFAULT_IMAGE_MODEL = 'google/gemini-nano-banana-2.1';
const DEFAULT_VIDEO_MODEL = 'x-ai/grok-imagine-video-1.5-lite';

const HELP = `ormedia - generate images and videos with OpenRouter

Usage
  ormedia login                      Sign in with the browser and store an API key
  ormedia logout                     Delete the stored API key
  ormedia status                     Show which key is in use and its spend
  ormedia models <image|video>       List models and the options each one takes
  ormedia image --prompt <text>      Generate or edit an image
  ormedia video --prompt <text>      Generate a video
  ormedia job <job-id>               Check a video job, and download it when done

Image options
  --model <id>            Default: ${DEFAULT_IMAGE_MODEL} (or $ORMEDIA_IMAGE_MODEL)
  --image <path|url>      Reference image to edit or follow. Repeat for more.
  --aspect-ratio <r>      For example 1:1, 16:9, 9:16
  --resolution <r>        For example 1K, 2K, 4K
  --size <WxH>            Exact pixels, for models that take them
  --quality <q>           auto, low, medium, high
  --format <f>            png, jpeg, webp, svg
  --background <b>        auto, transparent, opaque
  --seed <n>              Same seed, same picture, where the model allows it
  --n <count>             How many images

Video options
  --model <id>            Default: ${DEFAULT_VIDEO_MODEL} (or $ORMEDIA_VIDEO_MODEL)
  --first-frame <path|url>  Image the video starts on
  --last-frame <path|url>   Image the video ends on
  --image <path|url>      Reference image. Repeat for more.
  --duration <seconds>
  --aspect-ratio <r>      For example 16:9, 9:16
  --resolution <r>        For example 720p, 1080p
  --audio | --no-audio    Ask for sound, or for none
  --seed <n>
  --no-wait               Print the job id and return at once
  --timeout <minutes>     How long to wait for the video. Default: 20

Shared options
  --out <path>            A file name, or a folder. Default: the current folder.
  --search <text>         models: keep only models that match
  --wait                  job: wait until the video is done
  --no-open               Do not open the result in Fleet
  --json                  Print the result as JSON

The API key comes from $OPENROUTER_API_KEY, or from \`ormedia login\`.
`;

const SHARED_OPTIONS = {
  out: { type: 'string' },
  'no-open': { type: 'boolean' },
  json: { type: 'boolean' }
};

function parse(args, options) {
  return parseArgs({ args, options, allowPositionals: true });
}

function integer(value, flag) {
  if (value === undefined) return undefined;
  if (!/^\d+$/.test(value)) throw new Error(`${flag} must be a whole number, got "${value}"`);
  return Number(value);
}

async function signedInClient() {
  const found = await readKey();
  if (found === null) {
    throw new Error('No API key. Run `ormedia login`, or set OPENROUTER_API_KEY.');
  }
  return createClient({ apiKey: found.key });
}

const progress = (text) => console.error(text);

function report(result, json) {
  if (json) {
    console.log(JSON.stringify(result, null, 2));
    return;
  }
  for (const file of result.files) console.log(`Saved ${file}`);
  if (result.costUsd != null) console.log(`Cost $${result.costUsd}`);
}

async function finish(outputs, { values, stem, ...facts }) {
  const files = await saveOutputs(outputs, { out: values.out, stem });
  if (!values['no-open']) openInFleet(files);
  report({ files, ...facts }, values.json);
}

async function login() {
  const { code, verifier } = await authorize({ openBrowser: openUrl, log: progress });
  const key = await createClient().exchangeCode(code, verifier);
  console.log(`Signed in. Key stored in ${await saveKey(key)}`);
}

async function logout() {
  await deleteKey();
  console.log('Stored key deleted.');
}

async function status(args) {
  const { values } = parse(args, { json: { type: 'boolean' } });
  const found = await readKey();
  if (found === null) {
    if (values.json) console.log(JSON.stringify({ signedIn: false }));
    else console.log('Not signed in. Run `ormedia login`.');
    process.exitCode = 1;
    return;
  }
  const info = await createClient({ apiKey: found.key }).keyInfo();
  if (values.json) {
    console.log(JSON.stringify({ signedIn: true, source: found.source, ...info }, null, 2));
    return;
  }
  console.log(`Signed in. Key from ${found.source}`);
  if (info.label) console.log(`Label: ${info.label}`);
  if (info.usage != null) console.log(`Spent: $${info.usage}`);
  if (info.limit_remaining != null) console.log(`Left on this key: $${info.limit_remaining}`);
}

async function models(args) {
  const { values, positionals } = parse(args, {
    search: { type: 'string' },
    json: { type: 'boolean' }
  });
  const kind = positionals[0];
  if (kind !== 'image' && kind !== 'video') {
    throw new Error('Say which models: `ormedia models image` or `ormedia models video`');
  }
  const term = values.search?.toLowerCase();
  const list = (await createClient().listModels(kind)).filter(
    (model) => term === undefined || `${model.id} ${model.name}`.toLowerCase().includes(term)
  );
  if (values.json) console.log(JSON.stringify(list, null, 2));
  else console.log(list.map((model) => describeModel(kind, model)).join('\n\n'));
}

async function image(args) {
  const { values, positionals } = parse(args, {
    ...SHARED_OPTIONS,
    prompt: { type: 'string' },
    model: { type: 'string' },
    image: { type: 'string', multiple: true },
    'aspect-ratio': { type: 'string' },
    resolution: { type: 'string' },
    size: { type: 'string' },
    quality: { type: 'string' },
    format: { type: 'string' },
    background: { type: 'string' },
    seed: { type: 'string' },
    n: { type: 'string' }
  });
  const prompt = values.prompt ?? positionals.join(' ');
  if (prompt === '') throw new Error('An image needs a --prompt');
  const model = values.model ?? process.env.ORMEDIA_IMAGE_MODEL ?? DEFAULT_IMAGE_MODEL;
  const body = imageBody({
    model,
    prompt,
    references: await Promise.all((values.image ?? []).map(toImageUrl)),
    aspectRatio: values['aspect-ratio'],
    resolution: values.resolution,
    size: values.size,
    quality: values.quality,
    format: values.format,
    background: values.background,
    seed: integer(values.seed, '--seed'),
    n: integer(values.n, '--n')
  });

  const client = await signedInClient();
  progress(`Generating with ${model}...`);
  const { images, costUsd } = await client.generateImages(body);
  await finish(images, { values, stem: `${slug(prompt)}-${timestamp()}`, model, costUsd });
}

async function download(client, job) {
  const count = job.unsigned_urls?.length || 1;
  const outputs = [];
  for (let index = 0; index < count; index++) {
    outputs.push(await client.downloadVideo(job.id, index));
  }
  return outputs;
}

/** Prints a status once, when it changes, so a long wait is not a wall of lines. */
function statusPrinter() {
  const seen = { status: null };
  return (status) => {
    if (status === seen.status) return;
    seen.status = status;
    progress(`Video is ${status}...`);
  };
}

async function video(args) {
  const { values, positionals } = parse(args, {
    ...SHARED_OPTIONS,
    prompt: { type: 'string' },
    model: { type: 'string' },
    'first-frame': { type: 'string' },
    'last-frame': { type: 'string' },
    image: { type: 'string', multiple: true },
    duration: { type: 'string' },
    'aspect-ratio': { type: 'string' },
    resolution: { type: 'string' },
    audio: { type: 'boolean' },
    'no-audio': { type: 'boolean' },
    seed: { type: 'string' },
    'no-wait': { type: 'boolean' },
    timeout: { type: 'string' }
  });
  const prompt = values.prompt ?? positionals.join(' ');
  const frame = (ref) => (ref === undefined ? undefined : toImageUrl(ref));
  const firstFrame = await frame(values['first-frame']);
  const lastFrame = await frame(values['last-frame']);
  if (prompt === '' && firstFrame === undefined && lastFrame === undefined) {
    throw new Error('A video needs a --prompt or a frame image');
  }
  const model = values.model ?? process.env.ORMEDIA_VIDEO_MODEL ?? DEFAULT_VIDEO_MODEL;
  const timeoutMs = (integer(values.timeout, '--timeout') ?? 20) * 60_000;
  const body = videoBody({
    model,
    prompt: prompt === '' ? undefined : prompt,
    firstFrame,
    lastFrame,
    references: await Promise.all((values.image ?? []).map(toImageUrl)),
    duration: integer(values.duration, '--duration'),
    aspectRatio: values['aspect-ratio'],
    resolution: values.resolution,
    audio: values.audio ? true : values['no-audio'] ? false : undefined,
    seed: integer(values.seed, '--seed')
  });

  const client = await signedInClient();
  const submitted = await client.submitVideo(body);
  // Said before the wait, so the id survives a killed or timed-out command.
  progress(`Video job ${submitted.id} started with ${model}.`);
  if (values['no-wait']) {
    if (values.json) console.log(JSON.stringify({ jobId: submitted.id, status: submitted.status }));
    else console.log(`Check it with: ormedia job ${submitted.id} --wait`);
    return;
  }
  const job = await client.waitForVideo(submitted.id, { timeoutMs, onStatus: statusPrinter() });
  await finish(await download(client, job), {
    values,
    stem: `${slug(prompt || 'video')}-${timestamp()}`,
    model,
    jobId: job.id,
    costUsd: job.usage?.cost ?? null
  });
}

async function job(args) {
  const { values, positionals } = parse(args, {
    ...SHARED_OPTIONS,
    wait: { type: 'boolean' },
    timeout: { type: 'string' }
  });
  const id = positionals[0];
  if (id === undefined) throw new Error('Say which job: `ormedia job <job-id>`');
  const timeoutMs = (integer(values.timeout, '--timeout') ?? 20) * 60_000;

  const client = await signedInClient();
  const found = values.wait
    ? await client.waitForVideo(id, { timeoutMs, onStatus: statusPrinter() })
    : await client.getVideoJob(id);
  if (found.status !== 'completed') {
    if (values.json) console.log(JSON.stringify({ jobId: id, status: found.status }));
    else console.log(`Video job ${id} is ${found.status}.`);
    return;
  }
  await finish(await download(client, found), {
    values,
    stem: `video-${id}`,
    jobId: id,
    costUsd: found.usage?.cost ?? null
  });
}

const COMMANDS = { login, logout, status, models, image, video, job };

export async function main(argv) {
  const [name, ...args] = argv;
  if (name === undefined || name === 'help' || name === '--help' || name === '-h') {
    console.log(HELP);
    return;
  }
  const command = COMMANDS[name];
  try {
    if (command === undefined) throw new Error(`Unknown command "${name}". Try: ormedia --help`);
    await command(args);
  } catch (error) {
    console.error(`ormedia: ${error.message}`);
    process.exitCode = 1;
  }
}
