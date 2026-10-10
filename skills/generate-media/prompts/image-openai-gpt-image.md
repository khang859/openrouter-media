# OpenAI GPT Image

Models: `openai/gpt-image-2.5-sunburst`, `openai/gpt-image-2.5-flare`, `openai/gpt-image-2`, `openai/gpt-image-1`, `openai/gpt-image-1-mini`, and the `openai/gpt-5*-image*` models.

## When to use it

- Strong at exact text, layouts, user interface mockups, diagrams, and transparent backgrounds.
- Use `openai/gpt-image-2.5-sunburst` when quality or edit precision matters most.
- Use `openai/gpt-image-2.5-flare` for speed.
- Use `openai/gpt-image-1-mini` for many low-stakes pictures.
- A picture can take up to two minutes.

## Prompt structure

- Name the result first: the subject and what the picture is for, such as a product photo, an ad, or a diagram.
- Then give the order: background or scene, subject, key details, limits.
- For a complex request, use labelled sections: `Scene:`, `Subject:`, `Details:`, `Constraints:`.
- Short prompts, paragraphs, and labelled sections all work. Pick one that is easy to scan.
- Describe what the eye sees: materials, light, colours, medium.
- For people, state the framing, where they look, and what the hands do, for example "full body visible, feet included".
- Keep settings out of the prompt.
  Use `--aspect-ratio`, `--size`, `--quality`, `--background`, and `--n` instead.

## Recipes

- **Photograph:** say "photorealistic", "real photograph", or "iPhone photo".
  Ask for real texture such as pores and worn materials.
  Leave out studio words when you want a candid look.
- **Text in the picture:** put the exact words in quotes or in capitals, and give the position and the type style.
  Spell an unusual word letter by letter.
  Add "Show the text exactly once. No extra text."
  Use `--quality medium` or higher for small or dense text.
- **Logo:** give the brand character and the use.
  Ask for "clean, vector-like shapes, a strong silhouette, balanced negative space, flat design".
  Pass `--background transparent` with `--format png`, and `--n 4` for options.
- **Diagram or infographic:** write every real part, label, and number in the prompt.
  Use `--quality high`, avoid tiny text, and check the labels in the result.
- **Product cut-out:** "Isolate the product on a fully transparent background. Preserve product geometry and label legibility exactly. Do not add a backdrop, a checkerboard, scenery, or a shadow."
- **User interface mockup:** describe the product as if it exists. Do not use concept-art words.

## Edits and reference images

- Pattern: "Change only X." Then a list of what to keep: identity, geometry, layout, lighting, labels, camera angle. Then "Do not change anything else."
- Repeat the keep list on every new edit.
- Several images: number each one and give it a role, such as subject, style, clothing, or background.
  Then say what moves where: "Place the dog from the second image into the setting of image 1."
- Make one change for each step, and pass the last result back with `--image`.
- Same character: write one reusable description, pass the first result back, and add "Do not redesign the character."
- The model cannot promise that untouched areas stay identical to the pixel.

## Avoid

- Mood words alone for wide, dark, or neon scenes. Give the scale, the atmosphere, and the colours.
- A checkerboard pattern as a sign of transparency. Use `--background transparent`.
- The belief that a higher `--quality` always gives a better picture.

Unlike Google models, a short list of exclusions works well here: "No extra text, no watermarks, no unrelated logos."

## Examples

```
In this room photo, replace ONLY the white chairs with chairs made of wood. Preserve camera angle, room lighting, floor shadows, and surrounding objects. Keep all other aspects of the image unchanged.
```

```
Create an original logo for a company called "Field & Flour", a local bakery. The logo should feel warm, simple, and timeless. Use clean, vector-like shapes, a strong silhouette, and balanced negative space.
```

```
Place the dog from the second image into the setting of image 1, right next to the woman. Use the same style of lighting, composition, and background. Do not change anything else.
```

## Sources

- <https://developers.openai.com/api/docs/guides/image-prompting>
- <https://developers.openai.com/api/docs/guides/image-generation>
- <https://developers.openai.com/cookbook/examples/multimodal/image-gen-models-prompting-guide>
