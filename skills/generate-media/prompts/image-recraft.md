# Recraft

Models: `recraft/recraft-v4.1`, `recraft/recraft-v4.1-pro`, `recraft/recraft-v4.1-flash`, `recraft/recraft-v4.1-utility`, the `-vector` models, the `recraft-v4-styles` models, `recraft/recraft-v4`, and `recraft/recraft-v3`.

## When to use it

- The only family here that makes a real SVG file. Use a `-vector` model with `--format svg` for a logo, an icon, or a flat illustration.
- Strong design taste for posters, lettering, 3D shapes, and gradients.
- `-pro` gives 2K detail.
- `-utility` gives flat light and a front view, for mockups and simple product scenes.
- `-flash` is the fastest.
- The `recraft-v4-styles` models need 1 to 10 reference images with `--image`. They copy the style of those images, not the content.

## Prompt structure

Template:

```
[subject and action], [composition], [context], [medium], [style], [vibe], [light, colour, texture, technical details]
```

- Order matters. The model gives more weight to what comes first.
- A short prompt of 3 to 6 words is good to explore.
- A long, structured prompt gives control. Version 4 follows long prompts with many limits.
- Exact words are better than dramatic adjectives.
- **Poster order:** format, background layer, graphic layer, type hierarchy, text placement.
- **Logo or vector order:** graphic type, shape logic, colour system, line rules, layout, limits.

## Special syntax

- **Text:** put all the words in quotes, and say which line is the headline and which lines are secondary.
- **Colour:** use named colours in the prompt, such as "two-color palette (blue, green)".
- **Limits:** version 4 and 4.1 have no negative prompt. Write limits in the prompt: "no gradients, no shadows, no texture".

## Edits and reference images

- With `--image` on a normal model, describe the changed scene: "the same scene in winter, snow on the road".
- The guides give no wording for several images. For a mix of several pictures, use a Google, OpenAI, or FLUX model.

## Avoid

- Texture or material words in a logo or vector prompt.
- Text with no quotes.
- Stacks of praise words.

## Examples

```
Flat vector illustration of mountain landscape, simple geometric shapes, two-color palette (blue, green).
```

```
Fashion portrait, close up.
```

```
A humanoid robot with black body standing in a vast wildflower meadow with arms raised upward, captured from behind, shot with 35mm lens at f/2.8.
```

## Sources

- <https://www.recraft.ai/docs/prompt-engineering-guide/prompting-with-recraft-v4.md>
- <https://www.recraft.ai/docs/prompt-engineering-guide/prompt-templates/universal.md>
- <https://www.recraft.ai/docs/prompt-engineering-guide/visual-formats/vector-art.md>
- <https://www.recraft.ai/docs/recraft-models/choosing-a-model.md>
- <https://www.recraft.ai/docs/recraft-models/recraft-v4-1.md>
