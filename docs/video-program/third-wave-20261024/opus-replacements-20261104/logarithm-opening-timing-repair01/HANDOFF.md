# logarithm-opening-timing-repair01: F1, F2 and F3 from the 00–03 review

Status: staged and frozen. Not applied, not built by Cargo, no native test run, not rendered, not heard. Canonical is untouched.

Narration: ElevenLabs Oliver. Narration wording delta: none. `script.tsv` stays `f201d2dc57fa9cca`, so every take's text stands. No synthesis is needed.

Repairs F1, F2 and F3 of `logarithm-stage06-review00-03/REVIEW.md`. F4–F8 and the Lows are out of scope and untouched. This patch and `logarithm04-probe-repair01` touch different files, so they can go in either order.

## What changes

### F1, `ratio_ruler.rs`: the ruler waits for the words that ask for it

| Beat | Old, scratch | New, scratch |
|---|---|---|
| Title | 0.6–1.2 s | same, then alone until 6.63 s |
| "55, 110, 220" | 7.0 s with all the labels | 6.63 s, on "their frequencies" |
| 440, 880, 1760, 3520 | 7.0 s | 11.73, 13.32, 15.04, 16.36 s, each on its spoken words |
| "vibrations per second" | with the axis, about 1.5 s | 17.62 s, on those words |
| Axis and ticks | about 1.5 s, 18.2 s before "Mark them" | 19.70–20.70 s, on `say 00_2` |
| Seven dots | to 6.5 s | 20.70–21.96 s, same 0.18 s pace |
| Lead line to the low label | 7.0 s | 21.96–22.16 s |
| Next hook, half the ruler | 22.73 s | unchanged |

The longest stretch with no visible change here falls from 15.0 s (7.5–22.5 s, from the frames) to 5.43 s (1.20–6.63 s, computed). The composition from 22.16 s on is the old one. Every object keeps its position and wording, and the chapter's later beats do not move.

- **New in the file:** `read_out`, `read_shares`, `READ_PHRASES`, `READ_FADE` and `SCRIPT`, placed after `preview()` so no later line moves.
- **Timing basis:** arrival times are character shares of the sentence. They are estimates, and Oliver's real rhythm may differ inside the sentence.
- **Not `until_phrase`:** that would have added 6 phrase anchors for `phrase-timings.json`. Say if you would rather have measured marks.

### F2, `folded_decade.rs`: the digits stay while the voice names them

One fade list becomes two, in order, on the first sentence of 03_3:

| Beat | Old, scratch | New, scratch |
|---|---|---|
| Title and caption swap | 41.87–42.37 s | same |
| Lap objects fade | 42.37–42.97 s | 42.37–44.19 s, slowly, under "remembers the digits" |
| Digits, gold 2.4 and its word fade | with the lap objects, gone by 43.0 s | 44.19–44.49 s, after "the digits," |
| Dial lies flat | 42.97–44.97 s, 2.0 s | 44.49–45.87 s, 1.38 s |
| Sentence 1 | 45.87 s, hook | unchanged, and the lay now ends exactly on it |

The new `lay_beats` computes both run times and handles a quicker voice:

| First sentence | Lap fade | Lay | Lay ends |
|---|---|---|---|
| 3.6 s | 1.59 s | 1.21 s | on sentence 1 |
| 3.2 s | 1.36 s | 1.04 s | on sentence 1 |
| 3.0 s | 1.24 s | 1.00 s | 0.04 s after |
| 2.6 s | 1.01 s | 1.00 s | 0.21 s after |

The fade run time does the waiting, so **no wait is added to this file** and the binder sees no new site. Without a narrator it falls back to 0.6 s and the old 2.0 s lay.

**Change from the review's sketch:** the sketch waited until sentence 1 minus 2.0 s and laid the dial in 1.6 s. That left the digits starting to go at about 43.9 s, before "digits" ends near 44.18 s. Reading the share of the sentence keeps them through the word.

### F3, `slide_rule.rs`: 2.5 and 3.2 are named at their marks

| Line | Cue | Old | New |
|---|---|---|---|
| 158 | 02_2 | `0.398 of a full length` | `2.5: 0.398 of a full length` |
| 192 | 02_2 | `0.505` | `3.2: 0.505` |
| 223 | 02_3 | `0.505` | `3.2: 0.505` |

Line 223 goes beyond the review, so the same length does not change its label when it drops onto D's row.

The numbers come from facts `slide.a` and `slide.b`. Neither label states the answer, so the 02_2 question stays open until 02_3. No line moves.

## Bindings

| File | Baseline sha256 | Candidate sha256 | Lines |
|---|---|---|---|
| `ratio_ruler.rs` | `92cf028b124f8a3732218cbe31505e5351f86fea5bad180c0d3c97240e077bfd` | `198d4a0586369258d845a21ee857eeb51e34357b61fe40398e5789379a6b5fd1` | 550 → `650` |
| `folded_decade.rs` | `acfc7f9c198b3d3d0f99e1b160ec6c2d4e22a4ce28aeb4263deb29df9efa670d` | `c4f023ce5284dd768767b28c9d68a4da1e05c40385eb750dc9d4fbe583b6347e` | 595 → `658` |
| `slide_rule.rs` | `eea4d01d96fd4853ff541e0f55264178a09428079200dd9397b148a3976ab7e4` | `68d4b011fb4d90033ef1ae69aedef1639e12c11ce3624dbb5c378152d39b1ecd` | 518 → `525` |

Patch: `opening-timing.patch`, baseline to candidate, all three files. It applies and reverses byte for byte on a scratch copy. Its hash is in `SHA256SUMS`.

### Map13 hold rows

| Field | 00_4 → 00_5, `ratio_ruler.rs` | 02_2 → 02_3, `slide_rule.rs` |
|---|---|---|
| `sha256`, `evidence.sha256` | `92cf028b…7bfd` → `198d4a0586369258d845a21ee857eeb51e34357b61fe40398e5789379a6b5fd1` | `eea4d01d…76ab7e4` → `68d4b011fb4d90033ef1ae69aedef1639e12c11ce3624dbb5c378152d39b1ecd` |
| `line` | 371, unchanged | 204, unchanged |
| `marker` | unchanged | unchanged |
| `interval_sha256` | `86bf44bb70227da9e2f416862ba78a056faaaa05b3b7955e8bc009d769001515`, **unchanged** | `e45bdd98eeb62a82c547c28c645a4416ef1cc35c19bcfaff38420281e2a38c07` → `20094536ae2191f558b6ca76bc94b089e10f8fbe2bf78647b08036b306a75215`, **changed** |

- No hold binds `folded_decade.rs`.
- In `ratio_ruler.rs`, `say 00_2` moves from line 212 to 206, and lines 213–509 are byte-identical and keep their numbers. That is why 00_4 keeps its line and interval hash.
- My interval hashes reproduce map13's stored values on both baselines.
- The 02_2 interval changes because lines 158 and 192 lie inside it. The picture changes before that hold and the take does not. Line 223 is in 02_3, outside it.

### Binder `SITES`, `bind_final80-stage06.py`: **this file needs a table edit or the binder stops**

| Key today | Key after | Class | Note |
|---|---|---|---|
| `("ratio_ruler.rs", 181)` | `("ratio_ruler.rs", 210)` | `picture_pacing` | Unchanged: paces the note dots, 0.18 s each. The census now lists it under 00_2, not 00_1. |
| none | `("ratio_ruler.rs", 555)` | `word_sync_helper` | New: inside `read_out`, waits to a share of a sentence of 00_1, inside the voice. The census lists it under 00_6 because the helper follows `preview()`. |
| `("ratio_ruler.rs", 509)` | same | `helper_tail` | Unchanged. |

Without that edit the binder stops with "a wait this audit has not classified" and "classified waits that are not in the source". I read the stage06 binder and did not run it. Its stage may have moved on.

### Other binder effects, regenerated when it runs

- `phrase_marks` uses unchanged, so no phrase anchor is added or moved. `slide_rule.rs` phrase hooks stay on the same lines.
- Literal sentence hooks go up by 3: `sentence_at("00_1", 1)` in `read_out` and `sentence_at("03_3", 0)` and `("03_3", 1)` in 03_3.
- `cue_differs_from_current` gains one row: `ratio_ruler.rs` line `541`, 00_1, read while the census says 00_6. The preflight compares this list to the audit, so a regenerated audit should agree.
- The stale pair 00_1 → 00_2 now lists no waits in its final interval.
- The old hashes of `ratio_ruler.rs` and `slide_rule.rs` each appear twice in map13 and twice in `narration-direction-map-oliver12.json`, and nowhere else in the crate's docs. I did not touch them. Map13 is the map stage06 binds, so only it is listed above.

## Native re-review anchors

Times are the scratch-voice preview and should hold on a scratch rebuild. Under Oliver, count from each sentence start. Frame and sheet numbers are from the current preview, so a new sampler may pick different frames: go by time.

### F1

| Time | Current frame, sheet | Look for |
|---|---|---|
| 1.2–6.6 s | through frame 16, sheets 0–4 | Title alone. **Worth your eye:** 5.4 s of bare title at the film's opening. |
| 7.0 s | 17, sheet 4 | "55, 110, 220" alone. Does it read without its lead line and ruler? |
| 12.0–18.0 s | 27, 30, 34, 36, 39, sheets 6–9 | Each number arrives with its words; no ruler yet. |
| 19.7–22.2 s | sheets 11–13 | Axis under the numbers, dots landing under them, lead line. Nothing touches. |
| 22.73 s on | from sheet 13 | Same picture as now. |

**Under Oliver:** check the arrivals against the voice. These are shares, not marks. Also check that the ruler is whole before "The last step alone".

### F2

| Time | Current frame, sheet | Look for |
|---|---|---|
| 42.4–44.2 s | 99–103, sheets 24–25 | Numerals and gold 2.4 at full brightness while the arcs, pointer and panel fade. |
| 44.2–44.5 s | 103–104, sheets 25–26 | Digits leave after "digits" is said. |
| 44.5–45.9 s | 104–107, sheet 26 | The dial lies flat in 1.38 s, faster than the old 2.0 s. Does it still read as lying down? |
| 45.9 s on | from 107, sheet 26 | Helix and everything after, as now. |

**Worth your eye:** the caption "the dial shows only the mark" stays up while the digits are still showing. I kept it, because this edit changes order and not words.

### F3

| Time | Current frame, sheet | Look for |
|---|---|---|
| 15.6 s on | 40, sheet 10 | `2.5: 0.398 of a full length` inside its bar, about columns 135–351 of 98–388 (estimate). |
| 28.0 s | 65, sheet 16 | `3.2: 0.505` above C's bar, about columns 526–609 (estimate). |
| 41.5 s | 103, sheet 25 | `3.2: 0.505` under D's row. **Worth your eye:** the "3.2" now sits under D's marks near 5. Does it read as a length from C? |

**Glyphs:** the colon and all these characters already render in this film, for example in "slope = height > 0: always climbing". These exact strings are unseen natively.

## What was run

Every result is in `checks/*.out.txt`.

| Check | Result | Proves | Does not prove |
|---|---|---|---|
| `checks/frames.py` | Pass | New beats on the scratch timeline; new strings against ink on old frames | What a new render shows |
| `checks/source_checks.py` | Pass | Line pins, hold rows, `SITES` delta, patch round trip, `rustfmt --check` | That the binder itself accepts it |
| `checks/stand_in_run.py` | Base 84 tests, candidate 89, clippy clean, 12 of 12 controls | Types, borrows, the object ledger and the new tests' teeth | Real signatures, TeX, glyphs, frames |

- **Stand-ins:** bare rustc on the repo's pinned toolchain against hand-written crates, with draw.rs's own tests dropped and chapter 00's calls added for this run. This is not native evidence.
- **Comparison, base against candidate at 2.3, 2.68 and 3.1 words per second:** every cue starts at the same time in all 15 chapter runs and every chapter ends at the same time. The only per-cue row that differs is 00_1, whose picture finishes closer to its voice (−11.1 s to −1.0 s at 2.68). There are no object-ledger problems and no new estimated overlaps at rest. The stand-ins' text boxes are estimates.
- **Controls:** one unmutated build passes, and 11 mutations each fail the named test or tests.
- **Clippy caught one real problem** on the way: a literal-only `const { assert!(…) }` in my first F1 test. I replaced it.
- **Ems:** the estimate is within 4 px of the old labels' measured ink (179.3 against 179 px, 45.1 against 44 px).

**Not done:** Cargo build, native tests, render, listening, fine sync.

## Limits and independence

Not independent. I wrote the review findings being repaired, R3/R4 and the `slide_rule.rs` strings, and I missed F2 at source in Mail 1518. `folded_decade.rs` is LavenderPike's candidate and `building_ruler.rs` is root's binding, which is not changed here. I reviewed chapter 00 first in Mail 3162. No second reader has seen this repair.

All pixel figures come from root's 720p scratch frames. All times are scratch, and word times inside a sentence are character shares, not Oliver. Continuous pacing and how the new F1 opening feels remain unreviewed.

## Files

- `opening-timing.patch`
- `explainer_corpus/logarithm_mysteries/src/chapters/ratio_ruler.rs`, `folded_decade.rs`, `slide_rule.rs` (candidates)
- `source-markers.json`
- `checks/frames.py`, `checks/source_checks.py`, `checks/stand_in_run.py`, each with `.out.txt`
- `SHA256SUMS`
