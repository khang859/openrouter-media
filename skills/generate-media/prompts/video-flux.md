# Black Forest Labs FLUX video

Models: `black-forest-labs/flux-3-video`, `black-forest-labs/flux-video-edit`, `black-forest-labs/flux-video-upscale`.

## When to use it

- `black-forest-labs/flux-3-video` makes clips of 5 to 20 seconds at 720p or 1080p, with sound, from text, a first frame, or a first and last frame.
- It is one of the most expensive models, so draft on a cheaper model first.
- `flux-video-edit` and `flux-video-upscale` need a video as input. `ormedia` cannot send one, so do not use them here.

## Prompt structure

Order: camera, subject and action, environment, supporting motion, style and light.

- A short prompt gives the model freedom. A stuffed prompt makes the motion less coherent.
- Use one framing term and one movement term.
  `low tracking shot` works. `low aerial handheld orbit push-in` does not.
- Plan 2 or 3 actions at most for 5 seconds.
- Use exact words, not vague adjectives.

## Camera words

- Move: pan, tilt, dolly in, tracking, orbit, crane or boom, handheld, whip pan, dolly zoom, Steadicam follow, push through, arc, pedestal, trucking, camera roll.
- Rig: drone FPV, bodycam, dashcam.
- Time: slow motion, speed ramp, timelapse.

## Sound

Sound is on by default.
Use this layout:

```
[Shot and action].
Dialogue: The mechanic says, "Try it now."
Ambience: a busy garage, an air compressor in the next bay.
Effects: the engine turns over and catches.
Music: none.
```

- Put the exact words in quotes and name a speaker that the viewer can see.
- For a voice from off screen, write `voiceover` or `narration`.
- A quoted line with no speaker can show as text on screen. Add `No on-screen text or subtitles.`
- Describe a voice with facts: age, accent, pitch, and how it was recorded. Add `no announcer delivery` for a natural voice.
- Do not write `quiet room tone`. It becomes static noise. Name a sound source, or write `No music`.
- Keep lines short. If the last word is cut off, shorten the line or use a longer `--duration`.

## With a first frame

- The image sets the first frame. The prompt drives motion and sound.
- A last frame pins the end, and the model makes one continuous shot between the two.

## Several shots in one clip

```
[0.0s-2.5s] Wide shot, a red fox trots across fresh snow.
HARD CUT
[2.5s-5.0s] Close-up, the red fox stops and looks at the camera.
```

- Repeat the same subject description in every shot.
- Use one piece of music across all shots.
- The times are targets, not exact controls.

## Avoid

- Stacked camera terms.
- Lines for several speakers who interrupt each other. Check the result if you need this.
- The belief that the same voice description gives the same voice in the next clip.

Short exclusions inside the prompt do work on this model: "No on-screen text", "no logos, no subtitles, no credits", "No people anywhere".

## Examples

```
A red fox leaping through fresh snow, telephoto.
```

```
Slow pan across pine trees to reveal a lake at dawn.
```

## Sources

- <https://docs.bfl.ai/guides/prompting_video_overview>
- <https://docs.bfl.ai/guides/prompting_video_text_to_video>
- <https://docs.bfl.ai/guides/prompting_video_audio>
- <https://docs.bfl.ai/guides/prompting_video_camera_terms>
- <https://docs.bfl.ai/flux_3/flux3_video>
