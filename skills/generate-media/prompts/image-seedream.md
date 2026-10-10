# ByteDance Seedream

Models: `bytedance-seed/seedream-5-0-pro`, `bytedance-seed/seedream-5-0-flash`, `bytedance-seed/seedream-5-0-lite`, `bytedance-seed/seedream-4.5`.

## When to use it

- Strong at dense pictures full of information: infographics, formulas, and a user interface made from a wireframe.
- Strong at text in many languages. The 5.0 Pro and Flash models also write Japanese, Korean, Arabic, Thai, and more.
- Use Pro for quality and Flash for speed and price.
- Use `seedream-5-0-lite` or `seedream-4.5` for a set of pictures with the same character. Pro and Flash make one picture at a time.

## Prompt structure

Write connected, natural sentences.

- Order: subject, action, environment. Then style, colour, light, and composition when the look matters.
- State the purpose and the type: "Design a logo for a gaming company".
- Short and exact is better than a pile of fancy words.
- Keep the prompt under 600 English words. A longer prompt loses elements.
- For a diagram or a formula, use exact technical terms and name the chart type, the layout, and the style.

## Special syntax

- **Text:** put the words in double quotes.
- **A set of pictures:** use the words "a series" or "a set", or give the number of pictures. This works on Lite and 4.5 only.
- **Exact area (5.0 Pro and Flash only):** `<point>x y</point>` and `<bbox>x1 y1 x2 y2</bbox>`.
  The numbers run from 0 to 999, with 0 0 at the top left.

## Edits and reference images

- There are four kinds of edit: add, delete, replace, modify. Give a short instruction for one of them.
- Do not use vague pronouns. Name the thing.
- State what stays: "keeping its pose unchanged", "Keep the composition unchanged".
- Several images: say "Image 1" and "Image 2" in the order of the `--image` flags, and say what to take from each.
- You can draw on the input picture. Mark an area with an arrow, a box, or a scribble, then write "where the red area is marked" and "Remove all sketch lines".
- Pro and Flash take up to 10 reference images.

## Avoid

- Keyword lists, such as "Girl, umbrella, tree-lined street, oil painting texture".
- Text with no quotes.
- Vague instructions, such as "Put that one in pink clothes".

There is no negative prompt.
Write an exclusion as a plain sentence: "Do not include any text."

## Examples

```
A girl in a lavish dress walking under a parasol along a tree-lined path, in the style of a Monet oil painting.
```

```
Replace the largest bread man with a croissant man, keeping the action and expression unchanged.
```

```
Replace the character in Image 2 with the character from Image 1, and generate the result in the style of Image 3.
```

## Sources

- <https://docs.byteplus.com/en/docs/ModelArk/seedream-4-0-5-0-prompt-guide>
- <https://docs.byteplus.com/en/docs/ModelArk/seedream-5-0-pro>
- <https://docs.byteplus.com/en/docs/ModelArk/seedream-5-0-pro-editing-guide>
