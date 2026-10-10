# Directing a video

This file is the craft of a good video, for any model.
Read it when the user wants more than one clip, a clip with a person who speaks, or a result that must look professional.
For the prompt syntax of one model, read the file for that model.

## Plan before you spend

A video costs money for each second, and about one take in three is good enough to use.
So plan on paper first.

1. Write the idea in one sentence.
2. Choose the aspect ratio one time and keep it for every shot: `16:9` for film and web, `9:16` for phones, `1:1` for a feed.
3. Write a shot list. Show it to the user before you make anything.
4. Write the style block (see "Keep shots consistent").

One shot is one camera move and one subject action.
Short clips follow instructions better than long clips, so two clips of 4 seconds are safer than one clip of 8 seconds.

| Duration | What fits |
|---|---|
| 4 s | One action, one camera move, one short line of speech |
| 8 s | One action with a setup and a result, or 2 to 3 small beats |
| 15 s and more | Several beats, but drift and unwanted cuts become likely. Use two shorter clips when you can. |

A shot list row has: number, duration, shot size, angle, camera move, action, sound, and the first-frame image.

## Make the still first

This is the most important habit.
A still costs a few cents, and a video costs much more.

1. Make the first frame with `ormedia image`, at the same aspect ratio as the video.
2. Look at it. Fix the composition, the clothes, and the light on the still, where a fix is cheap.
3. When the still is right, animate it with `ormedia video --first-frame <file>`.

- With a first frame, the image sets the look. The prompt describes only the motion, the camera, and the sound.
- Clean the still first. A flaw in the still gets worse in the video.
- A still with motion blur, or a pose in the middle of an action, makes the model continue that motion.
- For a controlled change, make two stills and use `--first-frame` and `--last-frame`.
  Keep the same subject and the same light in both, and describe the path between them.
  Two stills that are too different give a morph.
- To continue a shot, use a late frame of clip 1 as the first frame of clip 2.
  A clip slows down in its last tenth, so take the frame from just before that, not the very last frame.

```bash
ffmpeg -sseof -0.5 -i clip1.mp4 -frames:v 1 clip2-first.png
```

## Keep shots consistent

The model has no memory of the last shot.
Each prompt must carry everything again.

- Write one style block and paste it, word for word, into every prompt:

```
STYLE: 35mm film, anamorphic lens, soft window light from the left, palette of teal, amber, and cream, fine grain.
CHARACTER: Mara, about 30, short black hair, green wool coat, silver ring on her right hand.
LOCATION: a small bakery with white tiles and brass lamps, early morning.
```

- Name 3 to 5 colours. A named palette keeps the colour stable.
- Keep the same light source and direction in every shot of a scene.
- For a person who appears many times, make a character sheet with an image model first: front, side, back, and a close view of the face.
  Pass it with `--image` on models that take reference images.
- Do not mix traits that compete, such as two hair colours in different prompts.

## Camera and light words

Choose each word for its effect.

| Choice | Effect |
|---|---|
| Extreme wide shot | Scale, a person alone |
| Wide shot | A person in a place |
| Medium shot | Neutral, good for speech |
| Close-up | Emotion |
| Extreme close-up | One detail |
| Over-the-shoulder | A conversation |
| POV | The viewer is the person |
| Eye level | Neutral |
| Low angle | Power |
| High angle | Weakness |
| Dutch angle | Unease |
| Static, locked-off | Attention on the acting |
| Push in | Tension or closeness |
| Pull out | A person left alone, or a reveal |
| Pan, tilt | A reveal, scale |
| Tracking shot | The viewer moves with the subject |
| Orbit, arc | Energy around a subject |
| Crane | Grandeur |
| Handheld | Raw, like a documentary |
| Shallow depth of field | The subject stands out |
| Rack focus | Attention moves from one thing to another |
| Golden hour | Warm, nostalgic |
| Blue hour | Calm, sad |
| High-key light | Clean, bright, happy |
| Low-key light | Drama |
| Backlight, rim light | The subject separates from the background |
| Slow motion | Weight, emotion |
| Timelapse | Time passes |

Replace a vague word with exact ones.
"Cinematic look" is weak. "Anamorphic lens, shallow depth of field, shafts of light through dust" is strong.

## Direct the action

- Describe the action in counted beats: "She takes four steps to the window, pauses, and pulls the curtain in the final second."
- "Person moves quickly" is weak. "The cyclist pedals three times, brakes, and stops at the crossing" is strong.
- Use order words: "First ... then ... finally ...".
- Show emotion as behaviour: "rubbing his temples", "a slight smile", not "he is tired".
- Say what you want, not what you do not want.
- State the screen direction in each prompt: "moves from left to right", "looks to the left of the camera".
- Models add motion by default. For a still camera, write "The locked-off camera remains perfectly still."
- Hands, text, crowds, and fast turns cause the most errors.
  Keep hands busy with one simple thing or out of the frame, and use one or two subjects.
- If a shot keeps failing, freeze the camera, reduce the action, and clear the background. Then add things back one at a time.

## Sound

- There are three layers: speech, sound effects, and background sound. Write each in its own sentence.
- Speech must be short. Plan about 2 words for each second, so 8 words for a clip of 4 seconds.
- Describe the voice: "in a tired, low voice".
- Make background sound, sound effects, and one short line of speech in the model.
- Add music, narration, and anything that runs across a cut in an editor.
  Music that is made in each clip does not match from clip to clip.
- Pass `--no-audio` for background footage that gets music later.

## Think like an editor

`ormedia` makes clips. The user joins them in an editor or with `ffmpeg`.
Plan each clip so that it cuts well.

- Cover a scene with a wide, a medium, and a close shot.
- Change the shot size or the angle clearly between two clips. A small change looks like a mistake.
- Keep the camera on one side of the action, so a person who faces right keeps facing right.
- Cut during motion. A still moment at a cut shows the seam.
- Ask for 1 or 2 seconds more than the shot needs, then use only the best part.
- Trim the start and the end of each clip. The first frames can show errors, and the end slows down.
- Put real logos and exact text on in the editor. Models bend them.

```bash
# Keep 3.5 seconds, starting half a second in
ffmpeg -ss 0.5 -i in.mp4 -t 3.5 -c:v libx264 -crf 18 out.mp4

# Join clips listed in list.txt (lines of: file 'clip1.mp4')
ffmpeg -f concat -safe 0 -i list.txt -c copy joined.mp4
```

## Spend less on retries

- Draft on a cheap model, at the lowest resolution and the shortest duration.
  Make the final take only when the motion is right.
- A take at a higher resolution is a new take, not a sharper copy of the draft. Check it again.
- Change one thing for each retry, and tell the user what you changed.
- Pass `--seed` on models that take it, to make a retry easier to compare. The same seed does not promise the same clip.
- Try again with the same prompt when the idea is right and the fault is random.
- Rewrite the prompt when the same fault shows in two takes.
- Go back to the still when the fault is in the look.

| Fault | Cause | Fix |
|---|---|---|
| The face changes | Too much motion, no anchor | Use a first frame, a slower camera, a shorter clip |
| Bad hands, extra limbs | Complex hand action, crowds | Simplify the action, hide the hands, use fewer people |
| A cut you did not ask for | The prompt reads like several shots | Add "Continuous, seamless shot." |
| The camera drifts | Models move by default | "The locked-off camera remains perfectly still." |
| Wrong motion from a still | The still shows motion blur | Fix the still first |
| Part of the prompt is ignored | Too many actions | One move and one action. Split the shot. |
| Broken text or logo | Small text is a weak point | Add it in the editor |
| A hitch where two clips join | The first frame came from the slow end of the last clip | Take the frame from earlier |
| A morph between first and last frame | The two stills are too different | Use closer stills and describe the path |
| Lips do not match | The line is too long | One short line. Split it across shots. |
| Colour jumps between clips | Each clip varies a little | Same style block, then grade in the editor |

## Recipes

- **Product ad:** start from a clean product still as the first frame.
  One slow move for 4 to 6 seconds: a turntable, a push in, or an orbit.
  State what must not change: the label, the colours, the shape.
  Add the logo and the words in the editor.
- **Person who talks to the camera:** `9:16`, medium close-up, eye level, static or handheld, light from a window.
  One line of about 8 to 12 words for each clip, with the voice described.
  Change the shot size between clips.
- **Story scene:** style block and character sheet first.
  A wide, a medium, and a close shot. One beat for each clip of 4 to 8 seconds.
  Background sound in the model, music in the editor.
- **Short for phones:** `9:16`, a strong action in the first second, shots of 2 to 4 seconds, bold motion.
  Add captions in the editor.
- **Logo reveal:** make the final logo frame as a still and pass it with `--last-frame`.
  2 to 4 seconds, with one sound effect.
  Put the real logo over the last frame in the editor.
- **Landscape or background footage:** a wide or aerial shot with one slow move.
  State the time of day and the weather. Pass `--no-audio`.
  This is the easiest kind, so a text prompt with no still is fine.

## Sources

- <https://developers.openai.com/cookbook/examples/sora/sora2_prompting_guide>
- <https://cloud.google.com/blog/products/ai-machine-learning/ultimate-prompting-guide-for-veo-3-1>
- <https://academy.runwayml.com/prompt-guide>
- <https://kling.ai/blog/ai-image-to-video-workflow-guide>
- <https://blog.google/technology/ai/flow-video-tips/>
- <https://www.studiobinder.com/blog/ultimate-guide-to-camera-shots/>
- <https://www.studiobinder.com/blog/different-types-of-camera-movements-in-film/>
- <https://hackernoon.com/how-to-chain-ai-video-clips-into-one-shot-without-the-hidden-stutter-at-every-cut>
