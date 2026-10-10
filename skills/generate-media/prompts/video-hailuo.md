# MiniMax Hailuo

Models: `minimax/hailuo-3` (H3), `minimax/hailuo-3-max` (H3 Max), `minimax/hailuo-2.3`.

## When to use it

- `minimax/hailuo-3` makes 2K clips of 5 to 15 seconds with stereo sound in the same pass.
- `minimax/hailuo-3-max` makes 480p or 768p clips of 5 to 15 seconds.
- Both H3 models take a first frame, a last frame, and reference images.
- `minimax/hailuo-2.3` makes short silent clips, and it follows camera commands in brackets very exactly.

## Hailuo 3 (H3 and H3 Max)

### Prompt structure

Write the clip as a timeline, in plain English sentences.

- Start with the style and the first composition: "Live-action, cinematic, a medium-wide shot frames ...".
- Style words that the model knows: cinematic, live-action, 2D-animated, 3D CG, claymation, watercolor, vintage film.
- Describe only what the viewer can see or hear.
- Then give the action in order: the start, the development, the result or reaction.
- Finish with one sentence for the background sound and one sentence for the music.

### Camera

Write the camera move as a sentence inside the shot: type, then size, then speed.
Do not stack labels at the end.

```
The camera pushes in with small amplitude at slow speed toward the folded letter in her hands.
The camera pans right with large amplitude at fast speed, revealing the open doorway.
The camera holds a static shot as the runner exits the frame.
```

- Types: zoom in or out, push in, pull out, pan left or right, truck left or right, tilt up or down, pedestal up or down, arc shot, tracking shot, static shot, shake slightly or strongly, POV, roll clockwise or counterclockwise.
- Size: `with small amplitude`, `with large amplitude`. Leave it out for a medium move.
- Speed: `at slow speed`, `at fast speed`. Leave it out for normal speed.

### Sound

- Speech: describe the speaker and the voice, then the line.
  "The young woman with a quiet, breathy voice says: I get off at the next station."
- For a voice from off screen, write "says in an off-screen voiceover", and add "while his lips remain completely closed".
- Put text that must show on screen in double quotes.
- Background sound: one sentence for the whole clip, such as rain, traffic, or footsteps.
- Music: describe the style, or write that there is no music.

### With a first frame

- First name the anchors from the image: the style, the subject, the composition.
- Then describe the next action, and keep the person, the clothes, the colours, and the main objects the same.
- With a last frame too, describe the path between the two images: how the subject moves, how the pose changes, how the light changes.
- Use one continuous shot between a first and a last frame.

### Several shots in one clip

Give no time to the first shot.
Start each later shot with its cut time:

```
[Shot 1] Live-action, cinematic, a wide shot frames a lighthouse on a cliff at dusk.
[Shot 2] At 00:03.500, the camera cuts to a close-up of the keeper's hands on the lamp.
```

A cut must show something new.
If only the distance or the angle changes a little, use a camera move instead.

### The native format

The model is trained on a format with three labelled fields.
MiniMax does not say if a hosted user must write it, so use plain prose first.
Try this format when plain prose misses the sound or the cuts:

```
integrated_multimodal_description: [Shot 1] Live-action, cinematic, a medium shot frames ... The young woman with a quiet, breathy voice (S1) says: <d>[English] I get off at the next station.</d>

overall_soundscape: Steady train rumble, a soft door chime.

non_diegetic_music: N/A
```

## Hailuo 2.3

- The clip is silent.
- Put a camera command in square brackets after the action it belongs to.
- Commands: `[Truck left]`, `[Truck right]`, `[Pan left]`, `[Pan right]`, `[Push in]`, `[Pull out]`, `[Pedestal up]`, `[Pedestal down]`, `[Tilt up]`, `[Tilt down]`, `[Zoom in]`, `[Zoom out]`, `[Shake]`, `[Tracking shot]`, `[Static shot]`.
- Up to 3 commands in one bracket run at the same time: `[Pan left,Pedestal up]`.
- Commands in a row run one after the other.
- Plain words also work, but the commands are more exact.

```
A man picks up a book [Pedestal up], then reads [Static shot].
```

## Avoid

- A cut that shows nothing new.
- Camera labels in a pile at the end of a sentence on H3.
- A negative prompt. These models have none.

## Sources

- <https://openrouter.ai/minimax/hailuo-3>
- <https://openrouter.ai/minimax/hailuo-3-max>
- <https://huggingface.co/MiniMaxAI/MiniMax-H3/blob/main/docs/VIDEO_PROMPT_WRITING_GUIDE_base_en.md>
- <https://platform.minimax.io/docs/guides/video-generation>
- <https://platform.minimax.io/docs/api-reference/video-generation-i2v>
