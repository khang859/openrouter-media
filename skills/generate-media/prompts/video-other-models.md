# Other video models

These makers publish little or no prompt guide.
Use the general rules in `SKILL.md` and in `video-directing.md`, plus the notes here.
Run `ormedia models video --search <name>` to see the options and the price of each model.

## xAI Grok Imagine Video

Models: `x-ai/grok-imagine-video-1.5`, `x-ai/grok-imagine-video-1.5-lite`, `x-ai/grok-imagine-video`.

- Clips are 1 to 15 seconds, so this is the cheapest way to test an idea with a clip of 1 or 2 seconds.
- `grok-imagine-video-1.5-lite` is the default model of `ormedia` and the cheapest.
- `grok-imagine-video-1.5` makes real 1080p and speech that matches the lips.
- xAI gives one rule: describe the shot, with the camera move, the light, and the sound.
- There is no special camera syntax. Use plain words: "slow cinematic push-in", "drone shot", "the camera dollies from the doorway to the window".
- The model makes sound effects, background sound, and speech. No speech syntax is documented, so write the line in plain words with a named speaker.
- With a first frame, the clip takes the shape of the image, and the prompt describes what happens after the still.

```
Slow cinematic push-in as embers drift across the battlefield and the helmet's crest stirs in the wind.
```

```
Epic cinematic drone shot flying through mountain peaks.
```

Source: <https://docs.x.ai/developers/model-capabilities/video/generation>

## HeyGen

Models: `heygen/heygen-video-1`, `heygen/avatar-iv`.

- `heygen/heygen-video-1` makes clips of 5 to 15 seconds from text or a first frame. It has no prompt guide.
- `heygen/avatar-iv` makes a person talk from a script and a voice. `ormedia` cannot send a script, so do not use it here.

Source: <https://developers.heygen.com/avatar-iv>

## Models that edit a video

`runway/aleph-2`, `black-forest-labs/flux-video-edit`, and `black-forest-labs/flux-video-upscale` need a video as input.
`ormedia` cannot send one, so tell the user that these models are not available here.
