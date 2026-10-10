export const BASE_URL = 'https://openrouter.ai/api/v1';

/** Who is asking, on every request. OpenRouter uses it for app attribution. */
const APP_HEADERS = {
  'HTTP-Referer': 'https://github.com/khang859/openrouter-media',
  'X-Title': 'OpenRouter Media'
};

/** Job states that will never turn into a video. */
const DEAD_STATUSES = new Set(['failed', 'cancelled', 'expired']);

/** Drops unset fields, so an option the user left alone means "whatever the model does". */
function optional(fields) {
  return Object.fromEntries(Object.entries(fields).filter(([, value]) => value !== undefined));
}

function asImageUrl(url) {
  return { type: 'image_url', image_url: { url } };
}

export function imageBody(o) {
  return {
    model: o.model,
    prompt: o.prompt,
    // Buffered on purpose: a CLI has nowhere to show a half-drawn render.
    stream: false,
    ...optional({
      n: o.n,
      aspect_ratio: o.aspectRatio,
      resolution: o.resolution,
      size: o.size,
      quality: o.quality,
      seed: o.seed,
      output_format: o.format,
      background: o.background
    }),
    ...(o.references.length === 0 ? {} : { input_references: o.references.map(asImageUrl) })
  };
}

export function videoBody(o) {
  const frames = [
    [o.firstFrame, 'first_frame'],
    [o.lastFrame, 'last_frame']
  ]
    .filter(([url]) => url !== undefined)
    .map(([url, frameType]) => ({ ...asImageUrl(url), frame_type: frameType }));
  return {
    model: o.model,
    ...optional({
      prompt: o.prompt,
      duration: o.duration,
      aspect_ratio: o.aspectRatio,
      resolution: o.resolution,
      generate_audio: o.audio,
      seed: o.seed
    }),
    ...(frames.length === 0 ? {} : { frame_images: frames }),
    ...(o.references.length === 0 ? {} : { input_references: o.references.map(asImageUrl) })
  };
}

async function errorMessage(res) {
  const hint = res.status === 401 ? ' Run `ormedia login` to sign in.' : '';
  try {
    const message = (await res.json())?.error?.message;
    if (typeof message === 'string') return `${message} (HTTP ${res.status}).${hint}`;
  } catch {
    // Fall through to the status line.
  }
  return `OpenRouter responded ${res.status}.${hint}`;
}

/** What a failed job says about itself, in whichever shape the provider used. */
function jobError(job) {
  const error = job.error;
  if (typeof error === 'string') return error;
  if (typeof error?.message === 'string') return error.message;
  return `The video job ${job.status}`;
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

export function createClient({ apiKey = null, fetchImpl = fetch } = {}) {
  async function call(path, body) {
    const res = await fetchImpl(`${BASE_URL}${path}`, {
      method: body === undefined ? 'GET' : 'POST',
      headers: {
        ...APP_HEADERS,
        ...(apiKey === null ? {} : { Authorization: `Bearer ${apiKey}` }),
        ...(body === undefined ? {} : { 'Content-Type': 'application/json' })
      },
      body: body === undefined ? undefined : JSON.stringify(body)
    });
    if (!res.ok) throw new Error(await errorMessage(res));
    return res;
  }

  const client = {
    /** `kind` is `image` or `video`. Needs no key. */
    async listModels(kind) {
      return (await (await call(`/${kind}s/models`)).json()).data;
    },

    async generateImages(body) {
      const json = await (await call('/images', body)).json();
      if (!Array.isArray(json.data) || json.data.length === 0) {
        throw new Error('OpenRouter returned no image');
      }
      return {
        images: json.data.map((image) => ({
          data: Buffer.from(image.b64_json, 'base64'),
          mimeType: image.media_type ?? 'image/png'
        })),
        costUsd: json.usage?.cost ?? null
      };
    },

    async submitVideo(body) {
      return (await call('/videos', body)).json();
    },

    async getVideoJob(id) {
      return (await call(`/videos/${encodeURIComponent(id)}`)).json();
    },

    async downloadVideo(id, index) {
      const res = await call(`/videos/${encodeURIComponent(id)}/content?index=${index}`);
      return {
        data: Buffer.from(await res.arrayBuffer()),
        mimeType: res.headers.get('content-type') ?? 'video/mp4'
      };
    },

    /**
     * Polls until the job has a video. A job that outlives `timeoutMs` is not
     * lost: it keeps running on OpenRouter and `ormedia job <id>` picks it up.
     */
    async waitForVideo(id, { timeoutMs, intervalMs = 10_000, onStatus = () => {} }) {
      const deadline = Date.now() + timeoutMs;
      for (;;) {
        const job = await client.getVideoJob(id);
        onStatus(job.status);
        if (job.status === 'completed') return job;
        if (DEAD_STATUSES.has(job.status)) throw new Error(jobError(job));
        if (Date.now() + intervalMs > deadline) {
          throw new Error(
            `The video is still ${job.status}. It keeps running. Get it later with: ormedia job ${id} --wait`
          );
        }
        await sleep(intervalMs);
      }
    },

    async exchangeCode(code, verifier) {
      const json = await (
        await call('/auth/keys', {
          code,
          code_verifier: verifier,
          code_challenge_method: 'S256'
        })
      ).json();
      if (typeof json.key !== 'string') throw new Error('OpenRouter returned no API key');
      return json.key;
    },

    async keyInfo() {
      return (await (await call('/key')).json()).data;
    }
  };
  return client;
}
