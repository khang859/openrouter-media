# Runway

Models: `runway/gen-4.5`, `runway/aleph-2`.

## When to use it

- `runway/gen-4.5` is strong at camera control and a film look.
- Clips are 2 to 10 seconds at 720p, in 16:9 or 9:16.
- It makes no sound. Do not write speech or sound lines.
- It takes a first frame but not a last frame.
- `runway/aleph-2` edits an existing video. `ormedia` cannot send a video as input, so do not use it here.

## Prompt structure

- From text: `[Camera] shot of [subject] [action] in [environment]. [Supporting details]`
- From a first frame: `The camera [motion] as the subject [action]. [Extra details]`
- Write natural sentences. Keywords are weaker, and JSON is ignored.
- The order of the parts does not matter.
- There is no ideal length, but a very long prompt creates conflicts.
- Give each action real time. A walk across a room needs about 3 seconds.

## Camera words

- Move: pan, tilt up or down, dolly, push in, pull back, truck, tracking, pedestal, crane or jib, orbit, arc, zoom, crash zoom, whip pan, handheld, Steadicam, gimbal, static.
- Frame: macro, close-up, medium, wide, extreme wide, establishing, aerial, high angle, low angle, bird's-eye, worm's-eye, over the shoulder, POV.
- Focus: deep, soft, shallow, rack focus.
- You can combine terms. For a big move, say what comes into view at each stage.

## With a first frame

- Write almost only about motion.
- Add a visual description only for a new element, a big change, or an interaction.
- Use general names such as "the subject" or "the person".
- An image with motion blur or a pose in the middle of an action fights a prompt that says the opposite.
- To stop an unwanted cut, add `Continuous, seamless shot.` or use a longer `--duration`.
- For a still camera, add `The locked-off camera remains perfectly still.`

## Timing

```
[00:01] The door opens. [00:03] A cat walks in.
```

Or in plain words: "X occurs, then Y occurs. Finally, Z occurs."
The times are approximate.

## Avoid

- Negative wording. Write "sharp focus, high detail", not "not blurry".
- Abstract words such as "beautiful".
- Instructions that conflict.
- A fully still wide shot. It is hard for this model.

## Examples

```
A handheld low angle tracking shot, with low contrast and fast-paced motion, follows a skilled astronaut skateboarder on a moon landscape. Their movements blur against the soft glow of the dark lunar environment. Film grain, low contrast, black and white.
```

With a first frame:

```
The camera slowly pushes in as the person scales the giant soda.
```

## Sources

- <https://help.runwayml.com/hc/en-us/articles/47313737321107-Text-to-Video-Prompting-Guide>
- <https://help.runwayml.com/hc/en-us/articles/48324313115155-Image-to-Video-Prompting-Guide>
- <https://help.runwayml.com/hc/en-us/articles/47313504791059-Camera-Terms-Prompts-Examples>
- <https://help.runwayml.com/hc/en-us/articles/52150503729171-Aleph-2-0-Prompting-Guide>
