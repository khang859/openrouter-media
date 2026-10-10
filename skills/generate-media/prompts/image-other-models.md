# Other image models

These makers publish little or no prompt guide.
Use the general rules in `SKILL.md`, plus the notes here.
Run `ormedia models image --search <name>` to see the options each model takes.

## xAI Grok Imagine

Models: `x-ai/grok-imagine-image-2.0`, `x-ai/grok-imagine-image-quality`.

- xAI has no prompt guide. Its examples are one short sentence each.
- Edit with a plain instruction, and pass each result back with `--image` for the next step.
- With several images, refer to them in the order you send them. The model takes up to 3.
- `x-ai/grok-imagine-image-quality` stops on 2026-11-02. Use `grok-imagine-image-2.0`.

```
A collage of London landmarks in a stenciled street-art style.
```

```
Render this as a pencil sketch with detailed shading.
```

Source: <https://docs.x.ai/developers/model-capabilities/images/generation>

## Microsoft MAI-Image

Models: `microsoft/mai-image-2.6`, `microsoft/mai-image-2.6-flash`, `microsoft/mai-image-2.5`, `microsoft/mai-image-2.5-pro`.

- Microsoft has no prompt guide.
- Its stated strengths are text on labels, posters, and packaging, portraits, and product pictures.
- It takes up to 5 reference images.

```
A photorealistic concept art poster of a university at sunset, cinematic lighting.
```

Source: <https://learn.microsoft.com/en-us/azure/foundry/foundry-models/how-to/use-foundry-models-mai-image>

## Tencent Hunyuan Image

Model: `tencent/hy-image-v3.5-preview`.

- No prompt guide was found for this version.
- Write a long, full description. The earlier open model does not rewrite a short prompt for you.
- It takes up to 20 reference images. Refer to them by number.

Source: <https://github.com/Tencent-Hunyuan/HunyuanImage-3.0>

## Krea

Models: `krea/krea-2-large`, `krea/krea-2-medium`, `krea/krea-2-medium-turbo`.

- Krea has no guide of its own. These notes come from a partner guide.
- Use exact words for material, light, and camera.
- Use Medium for illustration, and Large for photographs and gritty looks.

Source: <https://fal.ai/learn/tools/krea-2-prompting-guide>

## inclusionAI Ming Image

Models: `inclusionai/ming-image-0.1-design`, `inclusionai/ming-image-0.1-design-layer`.

- Made for user interfaces, infographics, posters, and designs with a lot of text.
- `ming-image-0.1-design` takes no reference image.
- `ming-image-0.1-design-layer` needs exactly one reference image.
- There is no prompt guide.

Source: <https://huggingface.co/inclusionAI/Ming-Image-0.1-Design>

## Sourceful Riverflow and Meta Muse Image

Models: `sourceful/riverflow-v2.5-pro`, `sourceful/riverflow-v2.5-fast`, `sourceful/riverflow-v2-pro`, `sourceful/riverflow-v2-fast`, `meta/muse-image`.

- No official prompt guide was found for these models.
- For Muse Image, state counts, positions, and labels in plain words.
