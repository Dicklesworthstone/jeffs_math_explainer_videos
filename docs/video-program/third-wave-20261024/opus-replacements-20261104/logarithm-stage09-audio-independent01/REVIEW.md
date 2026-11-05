# Independent review: logarithm stage09 audio preparation (87do.63, Mail 3974)

LavenderPike, 2026-11-05.

**Subject:** BronzeJaguar's sealed `P/logarithm-stage09-audio-preparation01`, SHA256SUMS `2bb8e2800fd4d403ebd7bc73f8f6515b53a227125dc6a418fdd74e5e4c40bb13` (14 files). `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`.

**Scope, from root's Mail 3974:**
- the actual selection, decode and spans contract;
- preserving all authored pauses;
- the changed 04_4/08_5 wording;
- the current canonical assembler.

This review covers source and technical checks. Listening and fine sync are listed separately at the end and were not assessed.

**Verdict:** no High or Medium finding. The contract holds as written for this binding. There is one Low finding (L1) and two observations for root (O1, O2).

## What "independent" means here

- I wrote `checks/independent_checks.py`. I did not run or copy Bronze's `run_checks.py` or `mutation_check.py`.
- **K07** parses the same recognizer output the report used. It corroborates the placement, not the speech.
- **K12 and K13** call Bronze's own `load_selection`, `check_answers` and `resolved_points` in process on the real binding, with fixture files I wrote. They test Bronze's code; they do not reimplement it.
- **K09 and K10** take the decode receipt and span shapes from Bronze's source: the receipt keys come from the AST of `cmd_decode`, and the span fields from `cmd_prepare`. The canonical assembler then runs on my own synthetic world.

## Results

`checks/outputs/independent_checks.txt`: 14/14, byte-equal over two runs.

| Check | What it shows |
|---|---|
| K01 | Bronze's SHA256SUMS is `2bb8e280…`, and all 14 files verify |
| K02 | All 36 binding pins re-hash. Assembler, cache and kit are pinned at canonical `95615cfa…` / `47331efd…` / `8072720d…`, the bytes root installed in Mail 3920 |
| K03 | Stage08 → stage09 script: only 04_4 and 08_5 differ, each by exactly the reviewed phrase. 04_4 deletes "this" and inserts "e"; 08_5 replaces "Halving" with "Taking half of" (+7 bytes). Both keep 6 sentences. My splitter equals the canonical `sentences()` on all 80 cues |
| K04 | The non-test chapter code names 04_4/08_5 in 14 calls. `until_sentence` 1–5 for each, which fits 6 sentences. `until_phrase` `HALF_TURN_PHRASE` 'leaves a half turn' sits at byte 267 and `FACTOR_PHRASE` 'a factor of minus one' at byte 306, each found once in the stage09 text. Root's anchor plan and Bronze's report give the same bytes for the 4 clarity recordings, and 04_4 has no mark |
| K05 | Map oliver16 has 15 visual directions. Each sits at its cue's last word, and none is in 04_4/08_5. The 15 proposed dispositions are `native_visual_hold`, each with a current evidence hash. The canonical `review_directions` accepts them for all 80 units |
| K06 | The 4 clarity WAVs (48 kHz) match their inventoried file hashes, PCM hashes and frame counts. Each request names its cue, the exact stage09 text and 'Oliver - Clean, British and Steady', and hashes to the inventory's `receipt_sha256`. The 4 ui-takes01 04_4/08_5 recordings are kept as superseded |
| K07 | The clarity job's `command.log`, parsed with my own tokenizer and word alignment: each new take holds the new phrase and not the old one. All 20 sentence onsets equal the report's samples. 08_5-download01's 'Here' segment is [1.64, 2.90] = 1.26 s; download02's is 0.24 s |
| K08 | Real ffmpeg 8.1.2 with root's arguments, on synthetic 48 kHz clicks: frames = ⌈n/2⌉ = the helper's `expected_frames` at all four clarity lengths and at an odd length. A click at sample c lands at exactly c/2 |
| K09 | `cmd_decode` writes every canonical `EXTERNAL_RECORD_KEY` and no request key. Source 'ElevenLabs UI, Oliver' with voice 'Oliver' reads as 'Oliver - Clean, British and Steady'. The same receipt with a request on a record is refused by F2, so a request binding can only arrive as v2 |
| K10 | 80 synthetic takes, in Bronze's receipt and span shapes, with the 15 proposed holds, through a byte copy of canonical `assemble --require-directions` and `verify`. Every assembled cue equals its decoded take sample for sample (80/80), and the report onsets equal the span ends |
| K11 | Canonical assembler negatives. A one-sample trim is refused ("cover every selected target sample"). A hold relabelled `authored_silence` with zero gaps is refused. 0.1 s of silence inserted at a boundary without a direction is **accepted** (O1) |
| K12 | Selection, through Bronze's `load_selection` in process. 04_4/08_5 named by clarity path are chosen from oliver-clarity-takes01. A bare `04_4-download01.wav` is refused with the path to use. The ui-takes01 path is refused as superseded wording. A stage08 binding hash is refused |
| K13 | Answers: a stage08 answer id for 08_5 is refused as unknown. **L1 is reproduced** (below) |
| K14 | L1's exposure on the real report (below) |

## L1 (Low): onset answers are not checked against phrase marks

**Where the check is missing.** In `prepare_audio.py`, `resolved_points` (lines 766–795) resolves sentence onsets from root's `sample` answers or the recognizer, and marks from their own answers or the recognizer. Only the onsets are checked for order. Two later stages check only that the marks are in order and inside the WAV:
- `cmd_timings` (lines 1058–1062);
- the kit loader (`narration.rs` 937–945).

**Effect.** A `sample` answer for sentence k's onset can land after a recognizer mark inside sentence k, and nothing refuses it. The same holds for a next-sentence onset moved before a mark. In the film, `product_rule.rs` calls `until_sentence(08_5, 4)` and then `until_phrase(HALF_TURN_PHRASE)`. Such a mark would fire at the sentence start, not at the phrase.

**Reproducer (K13).** On 08_5-download01, sentence 4's onset was moved to 22.520 s, 0.1 s after the 'leaves a half turn' mark at 22.42 s. `check_answers` and `resolved_points` accept it, and the marks still pass the kit's order and range rules.

**Exposure (K14).** Bronze's report has 54 supported marks inside sentences 1..n.
- The smallest lead of a mark over its sentence's onset is 1.76 s (11_7, byte 174).
- The largest onset word segment among those sentences is 0.86 s.
- So only a correction far beyond the recognizer's stated uncertainty would trigger L1.
- Bronze's suggested 08_5-download01 s1 sample moves no mark, because 08_5's marks are both in sentence 4.

**Suggested fix (for Bronze or root; not applied).** After resolving in `resolved_points`, require start(k) ≤ mark < start(k+1) for the sentence k that holds each mark's byte. Refuse with the mark's id, so root answers the mark as well.

## O1: the assembler alone does not forbid invented silence

The canonical `tools/coherent_narration.py` (`95615cfa…`) accepts silence inserted at a sentence boundary that has no direction (K11). `check_dispositions` examines only directed boundaries, and `unit_spans` allows gaps of 0–5 s.

In this pipeline, other steps guard against invented silence:
- `prepare` writes `gap_after_samples` as all zeros;
- `timings` refuses if `spans.json` changed since prepare;
- `timings` refuses if the assembly's manifest is not that `spans.json`;
- `timings` refuses if any assembled cue's PCM is not its decoded take sample for sample.

So "nothing removed, no silence invented, every recorded pause kept" is established by `timings`, not by `assemble`. **An assembly should count as unproven until `timings` passes on it.** This is not a defect in Bronze's helper.

## O2: the decoder qualification is per ffmpeg build

On this host's ffmpeg 8.1.2, root's arguments do two things (K08):
- give ⌈n/2⌉ frames;
- put 48 kHz sample c at exactly c/2.

Together these mean the recognizer seconds, measured on the 48 kHz files, and the sentence samples, on the 24 kHz decode, share one timeline.

Bronze's checks rely on that without needing it themselves:
- E checks frame counts only;
- D uses a stand-in decimator that keeps every other sample. That gives the same positions this ffmpeg build produces.

The decode receipt records the ffmpeg version, but `prepare` does not compare it. **If root decodes with another build, rerun K08 first.**

## The contract as checked

**Selection.**
- Root names every cue in script order.
- 04_4 and 08_5 must be given as clarity paths. Bare names shared with superseded recordings are refused, and so are superseded paths.
- Selections and answers from stage08 cannot carry over (K12, K13).

**Decode.**
- ffmpeg uses root's arguments, with no cut, gain, time or pitch change, and exactly ⌈n/2⌉ frames (K08).
- The legacy Oliver receipt names no request, so it passes F2 (K09).

**Spans and authored pauses.**
- Each unit is one whole decoded take, every gap is 0, and the last end is the take's last sample. The canonical `unit_spans` refuses a trim (K11).
- The only authored directions are the 15 end-of-cue visual holds. They become `native_visual_hold`, so the picture holds and the take stays as recorded (K05).
- K10 shows the assembled cue is the decoded take for 80/80 cues.
- O1 says which step proves it on real audio.

**Changed wording.**
- Only 04_4 and 08_5 changed, by the reviewed phrase (K03).
- The chapter hooks still fit, and 08_5's marks are at the moved bytes 267/306 (K04).
- The four clarity requests name the exact new text (K06).
- The recognizer output holds the new phrase and not the old (K07).

**Canonical assembler.** Pinned at root's installed bytes (K02), and exercised end to end through a byte copy (K10, K11).

## Not assessed: listening and fine sync, for root

- **The new wording as spoken.** Whether the four clarity takes say the new words as written. "base E" is the recognizer's spelling of the letter; it does not show how "e" was pronounced or stressed.
- **08_5-download01, sentence 1.** The 'Here' word segment is [1.64, 2.90], 1.26 s long, and abuts "roots.", so the true onset may lie anywhere in that window. If root selects download01, a measured `sample` for its s1 is worth having. Download02's segment is 0.24 s.
- **One recognizer.** Only large-v3-turbo ran on the four new takes; no second model corroborates them.
- **Bronze's open questions.** The advisory 04_3 voice-provenance questions and the 00_1 twins (blocking only if selected) were not re-examined here.
- **Picture against voice.** Picture-to-voice sync, reading windows on real lengths, and AV acceptance.

## Not checked

- Bronze's tamper mirrors (G13–G16), its mutation check, and the equality of the 78 unchanged cues with stage08 (G21) were not re-run.
- The ui-takes01 requests are content-checked but not hash-bound. Bronze lists this as a gap, and I did not re-examine it.
- No Rust compiled, no production audio decoded, assembled or timed, no recognition run, no take selected.

## Files

| File | What it is |
|---|---|
| `REVIEW.md` | This review |
| `checks/independent_checks.py` | The 14 checks; read-only on the repository, writing only under a fresh scratch folder |
| `checks/outputs/independent_checks.txt` | Their output at seal |

**Replay**, from the repository root, with ffmpeg on PATH:

```
python3 -I -B P/logarithm-stage09-audio-independent01/checks/independent_checks.py NEW_SCRATCH
```
