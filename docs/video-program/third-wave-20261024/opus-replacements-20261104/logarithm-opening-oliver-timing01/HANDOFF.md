# logarithm-opening-oliver-timing01: F1 and F2 against Oliver's retained takes

Status: audit done; one small patch staged and frozen. Not applied, not built by Cargo, no native test run, not rendered, not heard. Canonical and the sealed `logarithm-opening-timing-repair01` are untouched.

Narration: ElevenLabs Oliver. Narration wording delta: none. No synthesis is needed.

## Result

| Beat | What the evidence shows | Proposal |
|---|---|---|
| F1, the numbers in 00_1 | The shares land within 0.08 s of the envelope reading on 10 of 12 candidates. The other two are "seventeen sixty", one per take, 0.23 s and 0.33 s late, inside a number spoken for about 1.15 s. | **No source change.** Recognizer marks would not improve this. |
| F1, the ruler in 00_2 | The ruler is whole at 2.46 s, before "frequency." ends at about 2.76 s or 2.84 s. | No change. |
| F2, the digits in 03_3 | On both takes the share puts the digits' 0.30 s fade on the word "forgets": 0.33 s and 0.44 s of overlap. The voice says "forgets" as the digits vanish, and the laps have already gone. | **Add one mark on "forgets"**, using the existing `phrase_marks` support. The share stays as the fallback. |

The sealed F2 still does what it promised: the digits stay until they are named. It holds them about 0.4 s longer than intended, and that lands them on the wrong word.

## Exact baseline

| Item | Hash | State at this check |
|---|---|---|
| `folded_decade.rs` patch baseline | `c4f023ce5284dd768767b28c9d68a4da1e05c40385eb750dc9d4fbe583b6347e` | The sealed candidate, and root's `combined-repair-stage07/candidate`. Canonical still holds `acfc7f9c…`, so **do not apply this to canonical before the sealed patch.** |
| `ratio_ruler.rs` | `198d4a05…` | Stage07 equals the sealed candidate. This patch does not touch it. |
| `slide_rule.rs` | `68d4b011…` | Same. |
| `script.tsv`, stage07 | `b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e` | Root's 01_5 rounding wording. |
| `script.tsv`, canonical | `f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84` | Differs from stage07 only in 01_5. 00_1, 00_2 and 03_3 are the same text in both. |

If `folded_decade.rs` in stage07 has moved since this check, the patch may not apply. Tell me rather than forcing it.

## The evidence and its limits

Six retained takes, two per cue, all recorded as Oliver, Eleven v4, with `listening: false` and `fine_sync: false`. Each WAV hash and text was checked against root's records. Neither take of any cue is selected. Hashes are in `checks/recognition_audit.out.txt` and `recognition-candidates.json`.

Three kinds of time appear below. **None was heard.**

- **Recognizer:** the word row in the retained whisper-cpp logs, `logarithm-opening-recognition01` and `logarithm-middle-recognition01`. Root's own limit applies: "ASR differences are diagnostic only… No listening, speech correctness, take selection or fine sync acceptance."
- **Envelope:** my mechanical reading of level and zero-crossing rate, 20 ms frames every 10 ms. Each candidate names its rule. Sounds such as /f/ are inferences from those two numbers. The fricative rule passed 12 controls on words with known first sounds (6 positive, 6 negative), but that does not make it a phoneme detector.
- **Share:** the sealed source's character share, recomputed here. The mirror reproduces the sealed scratch times.

Things that limit all of this:

- **Recognizer rows are contiguous.** Every row touches the next, so a word's start can include the silence before it. "and" gets 0.04 s and 0.06 s in 00_1, which cannot be its real length. The confidence field stays near 0.03 and is shared across runs of words, so I did not use it.
- **Sentence starts are provisional.** No `report.json` exists for these takes. In the one multi-sentence cue I read from each of two earlier Oliver reports (Golay and the five-paradox film), spans are contiguous, `gap_after_samples` is 0 and starts sit on the recognizer's 0.02 s grid, so I used the recognizer's sentence start. Root's assembly decides the real value. For F1 I also computed the case where it falls at the end of the quiet run: arrivals move by at most 0.08 s.
- **Times are take-relative.** They carry to an assembled cue WAV only if its samples keep the take's positions, with no pause cut before the word.
- **Numbers are normalized.** The recognizer prints "440," for "four hundred and forty". I bound these by exact token and reading order. That is not proof the words were spoken correctly.

## F1: the number arrivals

Seconds from the start of the take. Late is positive. Length 19.52 s on both takes. Sentence start 5.68 s and 5.88 s.

| Phrase | Token | Recognizer | Envelope, rule | Share | Share − envelope | Recognizer − envelope |
|---|---|---|---|---|---|---|
| **download01** | | | | | | |
| their frequencies | `their` | 6.90 | 6.96, quiet run | 6.95 | −0.01 | −0.06 |
| four hundred and forty | `440,` | 11.90 | 12.15, quiet run | 12.10 | −0.05 | −0.25 |
| eight hundred and eighty | `880,` | 13.66 | 13.76, quiet run | 13.70 | −0.06 | −0.10 |
| seventeen sixty | `1760,` | 15.06 | 15.21, quiet run | 15.44 | **+0.23** | −0.15 |
| thirty-five twenty | `3520` | 16.76 | 16.81, fricative | 16.78 | −0.03 | −0.05 |
| vibrations per second | `vibrations` | 17.86 | 18.10, vowel rise | 18.05 | −0.05 | −0.24 |
| **download02** | | | | | | |
| their frequencies | `their` | 7.08 | 7.13, quiet run | 7.13 | 0.00 | −0.05 |
| four hundred and forty | `440,` | 12.06 | 12.28, quiet run | 12.21 | −0.07 | −0.22 |
| eight hundred and eighty | `880,` | 13.54 | 13.77, quiet run | 13.79 | +0.02 | −0.23 |
| seventeen sixty | `1760,` | 15.00 | 15.17, quiet run | 15.50 | **+0.33** | −0.17 |
| thirty-five twenty | `3520` | 16.76 | 16.80, fricative | 16.82 | +0.02 | −0.04 |
| vibrations per second | `vibrations` | 17.82 | 18.05, vowel rise | 18.07 | +0.02 | −0.23 |

The vowel rise comes after the voiced /v/ starts, by up to 0.1 s, so the last row's share could be up to 0.1 s later against the true word start than shown.

**Why no F1 patch:**

- **Marks would not be more accurate.** The recognizer runs 0.04–0.25 s early on all 12, because its starts include the preceding pause.
- **Marks would not all exist.** The word-mark producer the stage06 preflight cites, `elevenlabs-golay-20261028/phrase_marks.py`, expands 3-digit numbers but keeps 1760 and 3520 as digits. Reimplemented on these rows, its logic gives no mark for "seventeen sixty" or "thirty-five twenty". Those two would stay on shares, while the others moved early. I did not run root's script, and this film may use another.
- **The cost is real.** It needs 6 anchors, a binder row, and a preflight row, to replace errors already within about one fade (0.35 s).

**Two things for your eye, unchanged:**

- The low label "55, 110, 220" arrives on "their frequencies", 1.12 s and 1.34 s before "fifty-five" (envelope 8.07 s and 8.47 s). This is by design.
- The title stands alone longer under Oliver: 6.35 s and 6.53 s, against 5.43 s on scratch, if 00_1 still starts at 0.60 s on the scene clock.

**The ruler:** 2.46 s after the cue starts (axis 1.00, dots 7 × 0.18, lead 0.20). It is whole 0.80 s and 0.88 s before the recognizer's "The last step alone", and before "frequency." ends.

## F2: the digits boundary

Cue text: "The dial remembers the digits, and forgets the laps."

| | download01 | download02 |
|---|---|---|
| Recognizer `digits` | 1.18–1.58 | 1.24–1.70 |
| Recognizer `and` | 1.58–2.14 | 1.70–2.16 |
| Recognizer `forgets` | **2.14**–2.68 | **2.16**–2.72 |
| Recognizer `To`, sentence 1 | 3.88 | 4.04 |
| Envelope: high-crossing run ends, likely the end of "digits," | 1.86 | 1.92 |
| Envelope: comma's quiet run | 1.87–1.95 | 1.92–2.03 |
| Envelope: fricative start, likely /f/ of "forgets" | 2.22 | 2.20 |
| Envelope: voice ends before sentence 1 | 3.35 | 3.40 |

**Why the share runs late.** Its span from sentence 0 to sentence 1 includes 0.58 s and 0.69 s of quiet. 30/52 of the speech alone would be 1.93 s and 1.96 s, inside the comma's run on both takes. The source can't compute that, because the kit does not expose where a sentence's speech ends.

**Beats, from the mirror of `lay_beats`, starting 0.5 s into the cue after the title swap:**

| Timing | Take | Lap fade | Digits fade | Lay, ending on sentence 1 | Digits fade vs "digits," end | Overlap with "forgets" |
|---|---|---|---|---|---|---|
| Share, sealed | 01 | 0.50–2.25 | 2.25–2.55 | 1.33 s | +0.39 s | **0.33 s** |
| Share, sealed | 02 | 0.50–2.34 | 2.34–2.64 | 1.40 s | +0.42 s | **0.44 s** |
| Mark on "forgets" | 01 | 0.50–1.84 | 1.84–2.14 | 1.74 s | −0.02 s | none, ends 0.08 s before |
| Mark on "forgets" | 02 | 0.50–1.86 | 1.86–2.16 | 1.88 s | −0.06 s | none, ends 0.04 s before |
| Rejected: mark on "and" | 01 | 0.50–1.58 | 1.58–1.88 | 2.00 s | **−0.28 s** | none |
| Rejected: mark on "and" | 02 | 0.50–1.70 | 1.70–2.00 | 2.04 s | **−0.22 s** | none |

**Why "forgets" and not "and".** The recognizer's `digits` row ends 0.29 s and 0.22 s before the comma's quiet run, around the /t/ closure. So a fade starting on `and` would begin while "digits" is still being said, the original F2 fault in small. `forgets` agrees across takes to 0.02 s, and the fade's 0.30 s runs back from it to almost exactly the word's end.

**Remaining error.** With the mark, the fade starts 0.02 s and 0.06 s before the likely end of "digits,". I applied no offset to correct this. A later take with a much shorter comma pause would start the fade earlier against the word.

A side benefit: the lay gets 1.74 s and 1.88 s, close to the old 2.0 s, against 1.38 s in the sealed scratch timeline. That is the "does it read as lying down?" question from the sealed handoff.

## The patch

`digits-boundary.patch`, one file, baseline `c4f023ce…` → candidate `acfd77f46c2378c98ec77d1b5e7d0f2cb9c15f2bc3d209d80ba690c8fcc26e5e`, 658 → 699 lines.

- **One scene line changes, line 345:** `lay_beats(…, spoken, forgets_mark(&kit))`. Lines above the constants keep their numbers.
- **New:** `SCRIPT`, `FORGETS_PHRASE = "forgets"`, and `forgets_mark`, which reads `phrase_marks("03_3")` at the phrase's byte, 35.
- **`lay_beats`:** takes the mark. If it lies inside the first sentence, the digits' fade ends on it. Otherwise the share stands. The lay still ends on sentence 1.
- **Without a mark, nothing changes:** scratch and share-only Oliver builds get the sealed beats, including F2 landing on "forgets" in an Oliver build with no `phrase-timings.json`.
- **One new test,** plus the existing `lay_beats` test updated for the new argument.

Same pattern as `climb_onset` in `undoing_growth.rs`: mark if present, character share if not.

**To reach the kit, the mark needs `phrase-timings.json`** in the narration folder, covering all 80 cues and binding each WAV, so it can't exist before takes are selected. The reimplemented logic gives byte 35 the recognizer's `forgets`, 2.14 s and 2.16 s. I wrote no marks file. `recognition-candidates.json` is evidence, not timing data, and the loader would not accept it.

## Binder and preflight

Read, not run, from `combined-repair-stage06`. I saw no stage07 binder yet. Root's pending rebind is the natural place for these. All rows are in `source-markers.json`.

**The binder stops until these three go in:**

1. `PHRASE_MARK_USES`: add `"folded_decade.rs": 1`. It stops today on "1 phrase_marks uses; 0 expected".
2. `FALLBACK`: add a `folded_decade.rs` row, proposed text in `source-markers.json`.
3. Add the anchor by hand, as lines 638–641 do for 04_2:
   - cue 03_3, phrase `forgets`, const `FORGETS_PHRASE`, byte 35–42, sentence 0
   - sites: line 474 `forgets_mark`, needle `phrase_marks("03_3")`; line 345 `lay_beats`, needle `spoken, forgets_mark(&kit));`

**The preflight then needs:** `PHR.direct_mark_readers` to expect `("folded_decade.rs", "03_3")` as well.

**Regenerated by the binder:** anchors 24 → 25, cues with anchors 14 → 15, `by_chapter_file` gains `folded_decade.rs: 1`, and the template gains `{"byte": 35, "seconds": null, "phrase": "forgets"}` under 03_3.

**No change:** `SITES`, sentence hooks, `HOOKS`, map13 and all 15 holds. None of the holds binds this file, and no wait is added.

## What was run

| Check | Proves | Does not prove |
|---|---|---|
| `checks/recognition_audit.py`: bindings, rule controls, share and `lay_beats` mirrors checked against sealed values, producer reimplemented | The tables above come from the retained bytes | That any word is where it was read. Nothing was heard. Root's producer was not run. |
| `checks/source_checks.py`: sealed bundle intact (12 files), one scene line changed, binder regexes on the candidate, anchor rows against bytes, `rustfmt --check` | The patch is small and these table rows are the needed ones under stage06's rules | That a stage07 binder has the same rules |
| `checks/stand_in_run.py`: stage07 sources against stand-in crates. 91 tests on base, 92 on candidate. Clippy clean. No mark: identical cue rows at three speech rates. Stand-in mark: same chapter ends, same picture at every sentence start, and a probe shows the beats change (1.40 s lap fade at each rate). Patch round trip. 7 mutations, each failing the named test. | Type, borrow and bookkeeping errors; the mark reaches the beats; no mark means no change | A real signature, the real loader, TeX, a glyph or a frame. The 2.2 s mark is a stand-in. |

`checks/stand_in_run.py` copies the sealed harness's approach and stand-ins.

## Native re-review anchors

No frame numbers: none of this has been rendered. Times are from the start of the cue.

**F2, with a mark loaded:**

| Time | Look and listen for |
|---|---|
| 0.5–1.85 s | Lap objects fade. Numerals and gold 2.4 stay at full brightness through "the digits,". |
| 1.85–2.15 s | Digits fade, in the comma's pause. They should be gone as "forgets" starts. |
| 2.15 s to sentence 1 | The dial lies flat over "forgets the laps", about 1.75–1.9 s. |

**F2, without a mark, under Oliver:** expect the digits to fade on "forgets". That is the fault this patch addresses, not a new defect.

**F2, scratch voice:** the sealed anchors still hold (44.19 s digits, 1.38 s lay). The patch changes nothing there, at least in the stand-in runs.

**F1, under Oliver:** does each number feel like it arrives with its words, especially "seventeen sixty", the late one? Is 6.4 s of title alone too long? Both need a person listening and looking.

## Limits and independence

- **Not independent.** I wrote F1 and F2, and this audit judges my own repair. It found a miss in my own F2 estimate, but someone else should still check the reading of the envelope.
- **No listening.** Nothing here replaces listening or fine sync review. Recognizer and envelope readings are diagnostics. If listening disagrees, listening wins.
- **Take dependence.** Both takes point to the same conclusions. A new take would need the tables redone.
- **Uncompiled against the real crate.** Cargo, native tests, Clippy and builds are root's.
- **Scope.** F1 and F2 only. F3 and the review's other findings are untouched.

## Files

| File | What |
|---|---|
| `digits-boundary.patch` | Sealed baseline → candidate, one file |
| `explainer_corpus/logarithm_mysteries/src/chapters/folded_decade.rs` | The candidate |
| `source-markers.json` | Hashes and every proposed table row. Nothing measured. |
| `recognition-candidates.json` | The 14 rows behind the tables. Not marks, not a `phrase-timings.json`. |
| `checks/` | Three scripts and their outputs |
| `SHA256SUMS` | Everything above |
