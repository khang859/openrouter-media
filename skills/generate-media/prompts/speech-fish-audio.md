# Fish Audio speech models

Use this guide for a model id that starts with `fish-audio/`.
The text you send is the script.
The model says every word of it, so send only the words to speak and the cues below.

## Emotion and tone cues

`fish-audio/s2-pro` and `fish-audio/s2.1-pro` read a cue in square brackets.
A cue is plain language, so it is not limited to a fixed list.

```text
[happy] What a wonderful day!
[whisper] Do not wake the baby.
What a [warm and proud] result this is.
I did not expect that. [laugh]
```

- Put a cue for a full sentence at the start of that sentence.
- A tone cue or a sound, such as `[laugh]` or `[sigh]`, can go anywhere.
- Keep each cue short: one to four words.
- Use few cues in a short text. One cue per sentence is the limit.
- Do not put two cues that disagree on the same sentence.

`fish-audio/s1` is the older model.
It takes the same emotion names in round brackets, such as `(happy)`, and only from its fixed list.

## Voice

- `--voice <id>` takes the id of a voice from the Fish Audio voice library at <https://fish.audio>.
  The id is the long string at the end of the voice page address.
- `--voice-sample <path|url>` copies the voice in a recording, with no id.
  Use a clean clip of 10 to 30 seconds with one speaker and no music.
- Add `--voice-sample-text "<the exact words in the clip>"` when you know them.
  The copy is closer with the words.
- With no `--voice` and no `--voice-sample`, the model picks a voice, and it can change between calls.
  For more than one line by the same speaker, always pass the same voice.

Copy a voice only when the user owns it or has the right to use it.

## Script

- Write the words as a person says them: "twenty twenty-six", not "2026", when the reading can be wrong.
- Use full stops and commas for pauses. A new paragraph gives a longer pause.
- Speech runs at about 150 words per minute in English. Use that to fit a script to a time.
- Put a long script in a file and pass `--text-file`.
- The model speaks more than 80 languages. Write the script in the language you want to hear.

## Price

`ormedia models speech --search fish` shows the price.
The price is per character of text, so a retry of a short line costs little.
`fish-audio/s2.1-pro-free:free` costs nothing and is good for a first test.
