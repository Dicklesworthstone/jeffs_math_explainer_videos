# Logarithm final master: join review of the encoded ASR (87do.63)

LavenderPike, 2026-11-05, for BlueSalmon. Paths:
- `L` = `narration_lab/third-wave-20261024/root/five-film-release-20261027` (the assignment's `B`)
- `EL` = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`

## Result

**I read 80 of 80 cues: both edge windows of each, 160 of 160.**

- **Clear:** 78/80 cues at both edges.
- **Open:** 2/80 cues, each with one closing-edge item already in your triage: 02_3 `turned`/`turn`, and the 04_4 ending.
- **Missing content at a join:** none supported, 0/80.
- **Repeated content at a join:** none supported, 0/80. `a small step` is the only repeat-like text, and the evidence runs against it.
- **Misleading boundaries:** the recognizer's word times mislead in several places. All of these are in the transcript. The master's audio sits at its placements in 160/160 windows.

**Recommendation:** no master change on this evidence. If a person does a hearing pass, send them to J1, J2, J3 and J4 below.

This is recognizer and signal evidence. Nothing was heard, and this is not AV acceptance. It is also not independent (see Disclosure).

## Candidates

| ID | Where | Evidence | Status |
|---|---|---|---|
| J1 | 02_3 closing, 307.72–308.10 s | Script says `turned`. Full pass: `turn`. Large unsplit: `turned`. Small unsplit: `turn`. | **Open.** Meaning is the same either way. Word cannot be settled from these passes. |
| J2 | 04_4 closing, 537.48–537.50 s | `a small step` appears between `is` and `called` in the full pass only. Three tokens share 0.02 s, and both unsplit passes omit them. If they were spoken, `which … natural` would be 11 words in 2.94 s (3.7 words/s) against 2.35 words/s for the whole cue. They echo `same small step`, stamped 2.4 s earlier. | **Open, not supported.** Probably a recognizer artifact. No heard verdict. |
| J3 | 04_4 end, 538.63–538.86 s | `Wow` is in the large unsplit pass only. Your −26.3 dBFS peak is at 30.008 s of the take, inside the decay of `natural`. Take RMS by span from 29.90 s: −30.1, −39.4, −56.6, −70.4, −85.0 dBFS. The master reaches the −44 dBFS music bed by 30.05–30.10 s and stays there. | **Open, not supported.** No separate sound shows in the take or above the bed. No heard verdict. |
| J4 | 04_3 opening, 487.15 s | Loudest file start of 80: first 5 ms peak −35.5 dBFS (next is −51.7). Voicing starts at 25 ms. The first sample is −22 of 32768, the file is the complete recording, and the word is recognized as `That`. | **Weak lead, mine.** Not a defect on this evidence. Worth a moment if someone is already at that spot. |

Your decision stands: preserve all takes and the master, and do not resynthesize on recognizer disagreement.

## Misleading boundaries

**M1, first-word times.** The first word of a cue is often stamped inside the silence before it: median −0.445 s, and 69/80 cues are 0.2 s or more early. Taken at face value, these words would be spoken before their placement:

| Cue | Word | Stamped | Placement | Early by |
|---|---|---|---|---|
| 11_8 | Allow | 1733.84 | 1751.417 | 17.58 s |
| 11_1 | Can | 1491.40 | 1498.233 | 6.83 s |
| 05_2 | Two | 648.98 | 653.800 | 4.82 s |
| 01_4 | Use | 178.98 | 183.417 | 4.44 s |
| 06_3 | Two | 845.76 | 850.033 | 4.27 s |
| 04_3 | That | 485.52 | 487.150 | 1.63 s |
| 02_1 | Now | 259.38 | 260.633 | 1.25 s |
| 08_1 | On | 1151.78 | 1153.017 | 1.24 s |
| 04_4 | The | 507.52 | 508.633 | 1.11 s |

The audio does not support this reading. Each take's level envelope matches the master at lag 0, and no gap holds speech-level energy. **Do not use these stamps for sync.** Four of the 13 thinking holds have their next word stamped at least 0.5 s early (01_4, 05_2, 05_6, 06_3), but no hold is shortened: the loudest 10 ms in any of the 13 is −38.4 dBFS.

**M2, neighbour words.** 63 windows hold 64 tokens from the adjacent cue. Examples: `second.` ends at 20.98 s, after 00_2 starts at 20.73 s, and `Two` closes the 05_1 window. This comes from the windows' 0.5 s margins and from stretched word times. No window holds a token from a non-adjacent cue.

**M3, the `confidence` field.** It sums to 1.0000 across a file, so it is a share, not a probability. The 0.03 on `Wow` says nothing. Your triage does not rely on it.

**M4, 05_5.** The cue lasts 4.40 s, so its two windows overlap and show the same words.

**Long silences are authored, not missing content.** 76 of 79 gaps fall in the usual classes: 52 near 0.6 s, 11 near 1.2 s, 13 near 4.85 s. The other three come from the source:

| After | Gap | Source |
|---|---|---|
| 04_2 | 2.607 s | The probe sweep finishes before 04_3 is said (`undoing_growth.rs`). |
| 10_4 | 7.450 s | Fine-print reading hold (`doing_nothing.rs`). |
| 11_20 | 18.347 s | Rule and row-by-row reading hold, the film's longest (`unreachable.rs`). |

## What was checked

`checks/join_review.py` asserts or counts each row. Two runs were byte-equal.

| Check | Denominator | Result |
|---|---|---|
| Placements equal `verification.json` and script order | 80 | 80 |
| Windows equal the overlapping `full-large.json` words | 160 | 160 |
| Windows read by eye against the script (`windows-read.txt`) | 160 | 160 |
| Script's first and last 12 tokens at each cue | 1,916 (05_5 has 10 tokens) | 1,808 exact, 104 numeric, 2 hyphenation, 1 homophone, 1 open |
| Tokens in a window from a non-adjacent cue | 160 windows | 0 |
| Inserted non-hyphen tokens in a window | 160 windows | 3, all J2 |
| Comparison rows re-derived | 157 | 157 |
| Rows by class | 157 | 141 numeric, 9 hyphenation, 3 homophone, 2 spelling, 2 open |
| Numeric rows with digits equal to the script's value | 141 | 141 |
| Rows touching a window | 157 | 71 |
| Non-first words starting outside their own placement | whole film | 0 |
| Last words starting after their cue's end | 80 | 0 |
| Narration PCM equal to the complete recording | 80 | 80 |
| Narration length equal to placement | 80 | 80 |
| Narration ends quietly (last 5 ms peak at most −78.3 dBFS) | 80 | 80 |
| Master envelope matches the take at lag 0, searched to ±300 ms | 160 | 160 (r minimum 0.989, median 0.996) |
| Negative control, neighbour take at the same window | 158 | r maximum 0.626 |
| Gaps with speech-level energy (10 ms above −36 dBFS) | 79 | 0 (loudest −37.8) |

**Other readings**
- **Film edges:** the first word is stamped 0.64 s (cue at 0.60 s). The last word ends at 1936.32 s, the cue at 1936.82 s, the video at 1937.42 s.
- **Long words:** 11 unpunctuated words last over 1.2 s in the windows. Nine are the first-word stretches of M1 (including `Now`, `On`, `That` and `The`), and two are `0.05t`, a whole spoken phrase. None suggests unrecognized speech.
- **Repeats in the script across a join:** two runs, both intended. `zero point zero five t` is carried from 04_6 into 04_7's working, and `i times i` is the 05_1 question answered in 05_2.

## Limits

- **Text is not speech.** A recognizer can fill in the expected words, so matching text is not proof they were spoken.
- **The envelope tests placement, not words.** It shows the retained take sits at its placement. It says nothing about what the take says.
- **Coverage:** windows reach at most 5 s into each cue. Words further in are covered only by the whole-film comparison.
- **Beyond transcripts:** I also read the retained narration and the existing 16 kHz WAV, read-only, to separate recognizer artifacts from signal. The thresholds are mine. The WAV is bound by `comparison.json`; I did not decode the master again.
- **No claims:** nothing was heard. Fine sync and final AV sign-off stay open.
- **No mutations:** no new ASR, synthesis, rendering, production edits, TODO, Beads or memory writes.

## Disclosure

This review is not independent of the audio work.
- I wrote the stage09 audio preparation and timing inspection, the ElevenLabs external voice repair, `render-preparation04`, `final-verification02` and the four-take review.
- You ran the ASR and the triage, and I read your triage before running my own checks.
- I did not read any other reviewer's join work.

## Files and replay

| File | What it is |
|---|---|
| `REVIEW.md` | This note |
| `join-ledger.tsv` | 80 rows, one per cue: edges, tokens, recognizer times, envelope, levels, verdict, note |
| `checks/join_review.py` | Read-only check, standard library only |
| `checks/outputs/summary.json` | Every count above, plus input hashes |
| `checks/outputs/windows-read.txt` | The 160 windows as read, with `\|\|` at each cue edge |

From the repository root, where OUT must not exist:

```
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-final-join-review01/checks/join_review.py OUT
```
