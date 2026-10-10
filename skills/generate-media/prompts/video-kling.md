# Kling

Models: `kwaivgi/kling-v3.0-pro`, `kwaivgi/kling-v3.0-std`, `kwaivgi/kling-video-o1`.

## When to use it

- Strong at people, natural body motion, and speech from several named speakers.
- Version 3.0 makes clips of 3 to 15 seconds, with sound.
- Use Pro for the final clip and Standard for drafts.

## Prompt structure

Order: subject with a short description, movement, scene, then camera, light, and mood.

- Use simple words and short sentences.
- For version 3.0 with sound, this order works well: scene and background sound, camera, the character's action, then the labelled speech.
- Give an action that can finish in the clip. Plan about one action for each 4 seconds.

## Camera words

Low-angle rear wide shot, side close-up, first-person POV, frontal medium shot tracking backward, eye-level tracking, high-angle wide with downward tilt, handheld, orbit, zoom in, aerial, telephoto.

## Sound

- Speech: `Speaker (tone, language or accent): line`

```
Mom (softly, in a surprised tone): Wow, I didn't expect this plot at all.
Dad (in a low voice, agreeing, in a calm tone): Yeah, it's totally unexpected.
```

- Name the speaker next to each line, most of all with three or more people.
- Speech works in Chinese, English, Japanese, Korean, and Spanish. Other languages become English.
- Write the accent in the label, for example American, British, or Indian.
- Background sound and music are plain description: "faint hum of the living room air conditioner in the background".

## With a first frame

- Order: subject and its movement, then background and its movement.
- Name the subject: "Mona Lisa puts on sunglasses with her hand", not "Put on sunglasses".
- Keep the prompt close to the image. A prompt that is far from the image causes a hard cut.
- With a last frame too, use two similar images of the same theme. Two very different images give a cut, not a smooth change.

## Several shots in one clip

```
Shot 1, low-angle rear wide shot, tracking behind the rider as she enters the canyon.
Shot 2, side close-up of her face, wind in her hair.
```

For one continuous take, use time cues: "At the 4th second, ..." and "In the final 3 seconds, ...".
It is not confirmed that the shot labels work through OpenRouter, so check the first result.

## Avoid

- Exact counts, such as "10 puppies on the beach". The count comes out wrong.
- Complex physics, such as a bouncing ball or the path of a thrown object.
- The `@Image1` and `@Video` tags from the Kling app. Use `--first-frame` and `--last-frame`.

## Examples

```
A medieval sailing ship sailing on the sea, a foggy night, bright moonlight, and an eerie atmosphere.
```

With a first frame:

```
The model is smiling with her hair blown by the wind.
```

## Sources

- <https://kling.ai/quickstart/klingai-video-3-model-user-guide>
- <https://kling.ai/quickstart/klingai-video-o1-user-guide>
- <https://kling.ai/quickstart/text-to-video-prompt-guide>
- <https://kling.ai/quickstart/image-to-video-guide>
- <https://kling.ai/quickstart/ai-video-start-end-frames>
