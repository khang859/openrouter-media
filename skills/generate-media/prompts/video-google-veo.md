# Google Veo

Models: `google/veo-3.1`, `google/veo-3.1-fast`, `google/veo-3.1-lite`.

## When to use it

- Strong at film-like shots with sound and speech in one pass.
- Clips are 4, 6, or 8 seconds, in 16:9 or 9:16.
- 1080p and 4K need 8 seconds. The Lite model has no 4K.
- English prompts work best.

## Prompt structure

Order: camera, subject, action, setting, style and mood.

- Keep one clip to one scene. A chain of events comes out muddled.
- Plan about one action for each 2 seconds.
- Put each sound in its own sentence at the end.

## Camera words

- Move: static, pan, tilt, dolly, truck, pedestal, zoom, crane, aerial or drone, handheld, whip pan, arc, tracking, POV.
- Frame: wide or establishing, medium, close-up, extreme close-up, low angle, high angle, bird's-eye, worm's-eye, Dutch angle, over-the-shoulder, two-shot.
- Lens: shallow or deep depth of field, wide-angle, telephoto, macro, rack focus, lens flare, dolly zoom.

## Sound

- Speech: `A woman says: We have to leave now.`
  Google's guides disagree here. One uses quotes around the line, one warns that quotes can make the words show as text on screen.
  Start with the colon form. Use quotes only when the lips do not match the words.
- Sound effect: `SFX: thunder cracks in the distance.`
- Background sound: `Ambient noise: the quiet hum of a starship bridge.`
- Music has no label. Describe it in a plain sentence.

## With a first frame

- The image is the first frame, so write motion only.
- Do not describe the subject, the background, or the light again.
- Use plain names: "the subject", "the woman", "she".
- Direct one or more of: the camera move, the subject's motion, the motion of the surroundings.
- With a last frame too, describe the change between the two images and the sound.

## Several shots in one clip

Use time blocks of about 2 seconds:

```
[00:00-00:02] Medium shot, the explorer pushes aside a jungle vine. SFX: leaves rustle.
[00:02-00:04] Reverse shot of her face as she sees the ruins. Ambient noise: distant birds.
```

## Avoid

- "No" lists in the prompt. Describe the wanted result: "a desolate landscape with no buildings or roads".
- Too many events in one clip.
- The belief that the same `--seed` gives the same clip.

## Examples

```
Medium shot, a tired corporate worker, rubbing his temples in exhaustion, in front of a bulky 1980s computer in a cluttered office late at night. The scene is lit by the harsh fluorescent overhead lights and the green glow of the monochrome monitor. Retro aesthetic, shot as if on 1980s color film, slightly grainy.
```

```
Crane shot starting low on a lone hiker and ascending high above, revealing they are standing on the edge of a colossal, mist-filled canyon at sunrise, epic fantasy style, awe-inspiring, soft morning light.
```

With a first frame:

```
Slow dolly in on the subject. Fog rolls in slowly across the landscape.
```

## Sources

- <https://ai.google.dev/gemini-api/docs/veo>
- <https://cloud.google.com/blog/products/ai-machine-learning/ultimate-prompting-guide-for-veo-3-1>
- <https://docs.cloud.google.com/vertex-ai/generative-ai/docs/video/video-gen-prompt-guide>
- <https://docs.cloud.google.com/vertex-ai/generative-ai/docs/video/best-practice>
