# Alibaba Qwen Image

Models: `qwen/qwen-image-3-pro`, `qwen/qwen-image-3`.

## When to use it

- Strong at text in Chinese and English: long paragraphs, complex layouts, posters, slides, and user interfaces.
- The Pro model is good at text that sits on a real material, such as paint on wood or print on fabric.
- Only Chinese and English are reliable.

Alibaba has no prompt guide for this model.
The notes below come from the product pages and their examples.

## Prompt structure

- Order: subject, setting, style. Then camera, mood, and detail when you need them.
- The official examples are long paragraphs: format and setting, then subject, then light, then style and mood.
- Write every line of text that must show, in the order a reader sees it.

## Special syntax

- **Text:** put the words in quotes, with the style and the place: "A dark blue sign reads 'Il Messaggero' in white Gothic lettering."

## Edits and reference images

- The model takes up to 4 images.
- Say "image 1" and "image 2" in the order of the `--image` flags.
- Change text: `Replace 'OLD' on the sign with 'NEW'.`
- Keep things with plain words: "Perfectly preserve the facial features."

## Examples

```
A dark blue sign reads 'Il Messaggero' in white Gothic lettering, partially obscured by foliage.
```

```
Replace 'HEALTH INSURANCE' on the Scrabble tiles with 'Tomorrow Will Be Better'.
```

```
Remove the hair from the plate.
```

## Sources

- <https://www.alibabacloud.com/help/en/model-studio/text-to-image>
- <https://www.alibabacloud.com/help/en/model-studio/qwen-image-edit-guide>
- <https://github.com/QwenLM/Qwen-Image>
