# ByteDance Seedance

Models: `bytedance/seedance-2.5`, `bytedance/seedance-2.0`, `bytedance/seedance-2.0-fast`, `bytedance/seedance-2.0-mini`, `bytedance/seedance-1-5-pro`.

## When to use it

- Strong at clips with several shots, speech, and sound in one pass.
- `seedance-2.5` makes clips of 4 to 30 seconds.
- The 2.0 models make clips of 4 to 15 seconds. Use Fast or Mini for drafts.
- All of them take a first frame and a last frame.
- Prompts work in English and in Chinese.

## Prompt structure

Order: subject, action, scene, visual style, camera, sound.

For `seedance-2.5`, use three blocks:

1. One sentence that sums up the clip.
2. The plot in detail, by time range or by shot.
3. Notes for the things that stay the same, such as the look, the clothes, and "No subtitles."

- Use one camera move for each shot.
- Prefer slow, gentle, continuous motion. A sprint, a big jump, or a hard roll comes out badly.
- Do not put too much plot in one time range. The model adds cuts or drops actions.

## Camera words

Push in, pull out, pan, track, follow, orbit, dolly in, dolly out, tilt up, handheld, dolly zoom, FPV, bullet time, speed ramp.

For an unusual term, add a short description of it, for example "rack focus: the focus moves from the cup to her face".

## Sound

The 2.0 and 2.5 models read special brackets:

- `{ }` for speech: `The girl says {Wait for me}.`
- `< >` for a sound effect: `<a door slams>`
- `( )` for music: `(soft piano)`

More rules:

- Add the emotion to the line: `Granny Zhou's line (in disbelief): {You came back}.`
- Keep all speech in one language. Mark a less common one: `says in Japanese {...}`.
- `seedance-1-5-pro` uses plain quotes with a speaker label: `Man: "Let's begin."`
  Describe the voice too: "calm emotional state, even tone, normal pace".
- To stop sound: "No BGM; generate only environmental sounds and action sounds." or "No audio."

## With a first frame

- Describe the action and the camera only: "The man turns his head and smiles at the camera."
- The clip takes the shape of the first frame.
- A last frame with a different shape is stretched, so make both images the same shape.

## Several shots in one clip

For 2.0, use shot labels, because exact times are unstable:

```
Shot 1: Side shot of a street alley; the man slowly starts running.
Shot 2: Close-up of his shoes on wet stone.
Shot 3: The man climbs over a low wall and disappears; the camera slowly pulls back and freezes on the empty street.
```

For 2.5, use whole seconds with no gaps:

```
0-3 seconds: Wide shot, the train enters the station.
3-7 seconds: Medium shot, the doors open and she steps out.
```

## Avoid

- Fast repeated actions, such as "shake your head three times per second".
- More than 4 reference characters. The faces become unstable.
- Strong emotion words on 2.5. They can cause glowing eyes.
- Text that must be spelled right on 2.5.

There is no negative prompt.
Write limits in the prompt: "No subtitles.", "Do not generate a watermark."
Subtitles are less likely in a wide clip than in a tall clip.

## Examples

```
In a calm emotional state, with an even tone and a normal speaking pace, say: "Let's begin with what matters most."
```

With a first and a last frame:

```
The girl in the image says {cheese} to the camera, with a 360-degree orbiting camera shot.
```

## Sources

- <https://docs.byteplus.com/en/docs/modelark/seedance-2-5-prompt-guide>
- <https://docs.byteplus.com/en/docs/modelark/seedance-2-0-prompt-guide>
- <https://docs.byteplus.com/docs/modelark/2168087>
