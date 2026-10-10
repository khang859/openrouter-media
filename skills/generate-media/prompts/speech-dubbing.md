# Dubbing a video

Use this guide to put a voice on a video: a narration over a silent clip, or a new voice track in place of the old one.
`ormedia transcribe` writes down the words that the video has now.
`ormedia speech` makes the new voice.
`ffmpeg` joins the voice and the picture, so check that `ffmpeg` and `ffprobe` are installed first.

For a narration over a silent clip, skip the "Translate a video that has speech" section.

## 1. Measure the video

```bash
ffprobe -v error -show_entries format=duration -of csv=p=0 clip.mp4
```

The script must fit in that time.
English speech runs at about 150 words per minute, so a 10 second clip takes about 25 words.

## 2. Write the script

Write the words for the time you have, and leave one second free at the end.
For a video with more than one part, write one line per part and note the second each line starts at.
For a translation, make each line about as long as the line it replaces.

## Translate a video that has speech

Do this in place of step 2 when the video already has a voice.

```bash
ormedia transcribe clip.mp4 --timestamps segment --model openai/whisper-large-v3-turbo
```

```text
[0.00 - 0.68] Hello.
[0.87 - 5.67] This is a short test of the speech command.
```

- The command takes the video file as it is. When the API says the file is too large, take the sound out first: `ffmpeg -i clip.mp4 -vn -ac 1 -b:a 64k sound.mp3`.
- Each line has its start and end time in seconds.
- `openai/whisper-large-v3-turbo` gives about one line per sentence, which is a good shape for a dub. It is also one of the cheapest models.
- The default model, `fish-audio/transcribe-1-pro`, marks each speaker in the text with `<|speaker:0|>`. Use it when more than one person speaks.
- `--speakers` adds a speaker to each word in the `--json` output, on the models that can, such as `elevenlabs/scribe-v2`.
- `--timestamps word` adds a time for each word to the `--json` output.

Then translate each line yourself.
Keep the translation about as long, when spoken, as the time of its line.
Show the user the translated lines before you generate the voice, because a name or a term can be wrong.
Generate one audio file per line, and place each one at the start time of its line with the last command in step 5.
A line that runs longer than its time slot will cover the next line, so check each length in step 4.

## 3. Generate the voice

```bash
ormedia speech --text-file script.txt --voice <id> --out voice.mp3
```

Use the same `--voice`, or the same `--voice-sample`, for every line of one speaker.
To keep the voice of the person in the video, cut a clean sample from it and pass it with `--voice-sample`:

```bash
ffmpeg -i clip.mp4 -ss 3 -t 15 -vn -ac 1 sample.wav
```

Do this only when the user owns that voice or has the right to use it.

## 4. Check the length

```bash
ffprobe -v error -show_entries format=duration -of csv=p=0 voice.mp3
```

- Too long: cut words and generate again. This sounds better than a faster voice.
- A little too long, up to 10 percent: speed it up with `ffmpeg -i voice.mp3 -filter:a "atempo=1.08" voice-fit.mp3`.
- Too short: that is fine. The video keeps its length.

## 5. Join the voice and the video

The video stream is copied, so the picture does not lose quality.
Never write over the source video: give the result a new name.

A silent video, or a full replacement of the old sound:

```bash
ffmpeg -i clip.mp4 -i voice.mp3 -map 0:v -map 1:a -c:v copy -c:a aac -b:a 192k dubbed.mp4
```

Keep the old sound, such as music, quiet under the voice:

```bash
ffmpeg -i clip.mp4 -i voice.mp3 -filter_complex "[0:a]volume=0.25[bed];[bed][1:a]amix=inputs=2:duration=first:normalize=0[a]" -map 0:v -map "[a]" -c:v copy -c:a aac -b:a 192k dubbed.mp4
```

Start the voice later, here after 1.5 seconds:

```bash
ffmpeg -i clip.mp4 -itsoffset 1.5 -i voice.mp3 -map 0:v -map 1:a -c:v copy -c:a aac -b:a 192k dubbed.mp4
```

Several lines, each at its own start time in milliseconds:

```bash
ffmpeg -i clip.mp4 -i line1.mp3 -i line2.mp3 -filter_complex "[1:a]adelay=0:all=1[a1];[2:a]adelay=6500:all=1[a2];[a1][a2]amix=inputs=2:normalize=0[a]" -map 0:v -map "[a]" -c:v copy -c:a aac -b:a 192k dubbed.mp4
```

## 6. Check the result

Run `ffprobe dubbed.mp4` and confirm one video stream and one audio stream, with the length of the source video.
Tell the user the file name and ask them to listen, because you cannot hear the result.
