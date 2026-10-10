# Alibaba Wan and HappyHorse

Models: `alibaba/wan-3.0-prime`, `alibaba/wan-3.0`, `alibaba/wan-2.7`, `alibaba/wan-2.6`, `alibaba/happyhorse-1.1`, `alibaba/happyhorse-1.0`.

## When to use it

- Wan 3.0 makes clips of 2 to 30 seconds with sound, so it is good for a long clip with several shots.
- Wan 2.6 and 2.7 make clips of up to 15 seconds.
- Wan takes a first frame. It takes no last frame through `ormedia`.
- HappyHorse makes clips of 3 to 15 seconds from text or a first frame, and has no prompt guide.

## Prompt structure

- Basic order: subject, scene, motion.
- Full order: subject with description, scene with description, motion with description, then look (light, colour, composition), then style.
- You can open with look tags: "Low angle shot, wide shot, soft lighting, warm colors, center composition."

For Wan 3.0, use this order:

```
[Overall description]
Shot 1 (0-4 s): [subject, scene, motion, look]
Shot 2 (4-8 s): ...
Dialogue: ...
Sound effects and music: ...
Style and mood: ...
Negative prompt list: No subtitles, No watermark.
```

Keep the negative list short.

## Camera words

- Plain words work: "camera pushes in", "camera moves left", "fixed camera", "orbit", "handheld follow", "85mm telephoto, shallow depth of field".
- Keep an orbit under 45 degrees. A wider orbit bends the space.

## Sound

- Speech: `He says: "Study hard and make progress every day."`
  Then add the emotion, the tone, the speed, the voice, and the accent.
- Several speakers: give each one a fixed label, put an action before the line, and link the lines with a word.

```
[Black-suited Agent, raspy deep voice]: "Don't move."
[Agent]: "Why?" Immediately, [Assistant]: "Because it's time."
```

- Sound effect: the source, the action, and the background sound.
- Music: the kind of music and its style.
- To stop them, write "No dialogue." or "No background music." If you do not, the model adds them freely.
- On Wan 3.0, add "Lip sync" when a person speaks to the camera.

## With a first frame

The image sets the subject, the scene, and the style.
Write only the motion and the camera move.

## Several shots in one clip

```
Shot 1 [0-3 s] Wide shot, the boy stands alone on the platform.
Shot 2 [4-6 s] Hard cut transition, fixed camera position, focusing on the boy's eyes, which are glistening with tears.
```

- On Wan 3.0, shots run end to end with no gaps, 2 to 5 seconds each.
- Write "hard cut" or "dissolve" between shots.
- For one take, write "Generate single shot."

## Avoid

- Real people by name.
- Text that must be readable.
- A very long or complex chain of actions in one shot.

## HappyHorse

- There is no prompt guide.
- Use a short, clear prompt: subject, action, scene.
- With a first frame, describe the motion only.
- It takes no last frame.

```
A miniature city built from cardboard and bottle caps comes to life at night.
```

## Examples

```
A man is doing a stand-up comedy routine. He says, "Study hard and make progress every day," in a relaxed tone, at a moderate speed, with a clear voice, in American English.
```

## Sources

- <https://www.alibabacloud.com/help/en/model-studio/text-to-video-prompt>
- <https://www.alibabacloud.com/help/en/model-studio/wan3-video-generation-prompt-guide>
- <https://www.alibabacloud.com/help/en/model-studio/happyhorse-text-to-video-api-reference>
