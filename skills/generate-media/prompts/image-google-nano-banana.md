# Google Nano Banana (Gemini image)

Models: `google/gemini-nano-banana-2.1`, `google/gemini-3.1-flash-image`, `google/gemini-3.1-flash-lite-image`, `google/gemini-3-pro-image`, `google/gemini-2.5-flash-image`.

## When to use it

- The best general choice for edits, for mixing several reference images, and for readable text in a picture.
- Use `google/gemini-3-pro-image` for professional assets with exact text, and when you need a style reference image.
- Use `google/gemini-3.1-flash-lite-image` for fast, cheap, single-step pictures. It is weak at edits over many steps and takes no character references.

## Prompt structure

Describe the scene in full sentences.
Do not send a list of keywords.

- Order for a new picture: subject, action, place, composition, style.
- Order with reference images: the reference images, how they relate, the new scene.
- Start with a strong verb that names the job, for example "Create", "Replace", or "Combine".
- Say what the picture is for and who will see it.
- Be exact about materials and textures.
- Split a complex scene into numbered steps.

## Recipes

- **Photograph:** write like a photographer.
  "A photorealistic [shot type] of [subject] in [setting]. [The light]. Shot from [camera angle] with a [lens]."
  A lens, an f-stop, a film stock, or a camera body (GoPro, Fujifilm, disposable camera) each sets a look.
- **Text in the picture:** put the exact words in quotes and name or describe the font.
  Decide the wording first, then ask for the picture.
  You can write the prompt in one language and ask for the text in another.
- **Logo:** "Create a [image type] for [brand] with the text "[words]" in a [font style]. The design is [style], with a [colour scheme]."
- **Illustration or sticker:** "A [style] of [subject] doing [activity]. The design has [bold outlines, cel shading]. The background is white."
  Ask for the white background in words.
- **Product shot:** "A high-resolution, studio-lit product photograph of [product] on [surface]."
  Then give the lighting setup, the camera angle, and one detail that must be in sharp focus.
- **Room for a headline:** put one subject in a named corner of "a vast, empty [colour] canvas".

## Edits and reference images

- Say what changes and what stays.
- Change one thing: "Using the provided image, change only the [element] to [new element]. Keep everything else in the image exactly the same, preserving the original style, lighting, and composition."
- Add or remove: "Using the provided image of [subject], [add/remove] [element]. Ensure the change is [how it must fit in]."
- Protect a detail: "Ensure that the features of [element from image 1] remain completely unchanged."
- Several images: say which element comes from which image, for example "the dress from image 1" and "the model from image 2".
  There is no special index syntax. "Image 1" or a short description both work.
- Same character across pictures: pass the earlier result back with `--image`.
  If the face drifts after many edits, start again with a full description of the character.
- An edit usually keeps the shape of the input image.
  Pass `--aspect-ratio` when you need a different shape.

Reference limits: up to 14 images in one prompt.
The Pro model follows up to 6 objects, 5 characters, and 3 style references.
The 2.1 and 3.1 Flash models follow up to 10 objects and 4 characters.

## Avoid

- Lists of things you do not want, such as "no cars".
  Describe the wanted result instead: "an empty, deserted street".
- Keyword lists.
- A request for an exact number of pictures in one prompt. The model can return a different count.
- Long or small text on the first try. Expect a second pass.

## Examples

```
A photo of a glossy magazine cover. The minimal blue cover has the large bold words "Nano Banana".
```

```
Using the provided image of a living room, change only the blue sofa to a vintage, brown leather chesterfield sofa. Keep the rest of the room, including the pillows on the sofa and the lighting, unchanged.
```

```
Put this logo on a high-end ad for a banana scented perfume. The logo is perfectly integrated into the bottle.
```

## Sources

- <https://ai.google.dev/gemini-api/docs/image-generation>
- <https://cloud.google.com/blog/products/ai-machine-learning/ultimate-prompting-guide-for-nano-banana>
- <https://developers.googleblog.com/en/how-to-prompt-gemini-2-5-flash-image-generation-for-the-best-results/>
