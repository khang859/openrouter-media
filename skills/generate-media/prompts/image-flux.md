# Black Forest Labs FLUX

Models: `black-forest-labs/flux-3-image`, `black-forest-labs/flux.2-max`, `black-forest-labs/flux.2-pro`, `black-forest-labs/flux.2-flex`, `black-forest-labs/flux.2-klein-4b`.

## When to use it

- Strong at photographs with a named camera look, type on posters, exact brand colours, and pictures built from several reference images.
- Use `flux-3-image` for most work.
- Use `flux.2-klein-4b` for fast, cheap drafts.
- English prompts are the most exact.

## Prompt structure

Order: type of picture or medium, subject and action, setting, light, framing and detail.
Put the most important part first, and effects last.

- Use full sentences for relations: who holds what, where the light comes from.
- Short phrases are fine for style, colour, and light.
- `flux-3-image` expands a short prompt itself. Start short, and add detail only for the parts you must control.
- Use a long prompt, about 150 to 200 words, for a scene with several subjects, a layout, or text.
- `flux.2-klein-4b` does not expand the prompt. Write full prose, describe the light source, its direction, and its softness, and end with `Style: [style]. Mood: [mood].`

## Special syntax

- **Exact colour:** attach a hex code to a named object with the word "color" or "hex": "the mug in color #FF0000".
  For a gradient, give a start code and an end code.
- **Text:** put the exact words in quotes.
  Add the place and the type style: weight, case, colour, serif or sans.
  A font name works as a comparison.
- **JSON prompt (FLUX.2 only):** send a JSON string as the prompt for a complex scene with several subjects.
  Fields: `scene`, `subjects` (each with `description`, `position`, `action`), `style`, `color_palette`, `lighting`, `mood`, `background`, `composition`, `camera` (`angle`, `lens`, `depth_of_field`).
  Use prose for a simple picture.

## Edits and reference images

An edit prompt has three parts:

1. Name the target so that only one thing matches: its position, colour, or neighbour.
2. State the change with a clear verb: Replace, Remove, Add, Change ... to, Recolor, Turn into.
3. Name what must stay: "Change nothing else", or a list of the parts to keep.

- Several images: say "image 1" and "image 2" in the order of the `--image` flags, each with a role: "the woman in image 2", "in the style of image 4".
- Put the base or setting image first. The `auto` aspect ratio follows image 1.
- Reference limits: 10 for `flux-3-image`, 8 for FLUX.2 max, pro, and flex, 4 for klein.

## Avoid

- "No X" wording. These models have no negative prompt. Say what fills the space: "an empty promenade", not "no crowds".
- Quality tags such as "masterpiece" or "highly detailed".
- Vague style words such as "vintage" or "aesthetic".
- A hex code with no object, such as "use #FF0000 somewhere".
- Long strings of text.
- "Combine these images" with no image number, no role, and no keep list.

## Examples

```
A vintage illustration of an apple in color #0047AB with a heart-shaped cutout in the middle, on a white background.
```

```
Replace the cherries in the right-most jar with multi-colored sprinkles. Change nothing else.
```

```
The person from image 1 wears the jacket from image 2 and stands in the kitchen from image 3.
```

## Sources

- <https://docs.bfl.ml/guides/prompting_summary.md>
- <https://docs.bfl.ml/guides/prompting_unified_basics.md>
- <https://docs.bfl.ml/guides/prompting_editing_single_reference.md>
- <https://docs.bfl.ml/guides/prompting_editing_multi_reference.md>
- <https://docs.bfl.ml/guides/usecases_t2i_json_prompting.md>
- <https://docs.bfl.ml/guides/usecases_t2i_hex_color_prompting.md>
- <https://docs.bfl.ml/guides/usecases_t2i_typography_design.md>
