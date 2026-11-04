# Picture review: algebra chapter 10, repair03 frames

**Reviewer:** WhiteSwan, for BlueSalmon's Mail 1965.
**Reviewed:** all 157 sheets (all 628 sampled frames) of `algebra-chapter10-repair03-frames01/10_cyclic`, against `sampling.json`, `cyclic.rs` `bee439fd…`, `script.tsv` `83619d48…` and the repair03 receipts.

**These pictures run on the scratch voice, at 1280×720, with no Oliver audio. I did not listen to anything, and no fine sync was checked.** Oliver figures below are estimates from root's ASR word times for take01 and take02. No take is selected. ElevenLabs Oliver remains the provider, and nothing here needs synthesis.

**Authorship:** I did not write this source, and I have no stake in repair03. Searches of the three `algebra-chapter10-*` bundles and the crate's `src` for my name, and of Mail from WhiteSwan on `cyclic` and `chapter10`, found nothing. A search for `exponential` in the same way returned my Mails 1900, 1889 and 1831, so the search works. LavenderPike staged the native rebuild (Mail 1021), root wrote repair02, and RubyRaven reviewed it and proposed the patch root adopted as repair03. **I did write the patch staged here, so it has not had an independent review.**

## Verdict

**No blocker. The on-screen maths is right, nothing leaks through the prediction hold, and every repair03 fix shows up in the pixels.** All six items RubyRaven asked the first frames to confirm pass on the scratch voice, except that chapter length on Oliver can't be confirmed from this preview.

**Five concrete defects remain, all about what the picture does while the voice talks.** `patches/cyclic-picture-followup.patch` fixes all five. It's uncompiled, changes no words and adds no facts. I'd take A and B even if the others are left out. A sixth item, F, is a design call I left unpatched.

| # | Severity | Cue | Defect | Sheets | Patch |
|---|---|---|---|---|---|
| B | Medium-high | 10_10 | The table sits unchanged for about 17.5 s on Oliver; the diagonal is never marked, and "order two" and the square's rotations have no picture | 126–143 | Yes |
| A | Medium-high | 10_2 | "The whole clock group has twelve elements. These are different meanings of order" has no picture: about 9.5 s unchanged | 023–029 | Yes |
| D | Medium | 10_1 | The question is on screen about 13 s before "add eight" and about 17 s before "predict" | 001–015 | Yes |
| E | Low-medium | 10_6 | The gcd row builds during the question, before "The formula says" | 070–072 | Yes |
| C | Low | 10_4 | `k = d k'` appears after the voice reaches "k prime" | 049–050 | Yes |
| F | Medium-low | 10_4 | An unvoiced star and `4 orbits, 3 positions each` play over the gcd sentence | 046–054 | No, a design call for root |

## What was reviewed

| Input | sha256 |
|---|---|
| `explainer_corpus/abstract_algebra_expanded/src/cyclic.rs` | `bee439fdad35eb75a22a05d32d4e9aad059feabac0456fbfd29e24746b391d46` |
| `explainer_corpus/abstract_algebra_expanded/script.tsv` | `83619d48676b32d5d93bf7527477aa752a902dd16927c548e4b3052cc2d689c0` |
| `explainer_corpus/abstract_algebra_expanded/docs/narration-direction-map-oliver24.json` | `7f05be9377a54b2c4e1a03ca83b563eb87d90734818273d59d95cea566784d4a` |

- **Sheets:** every sheet's index, full sha256, frames, times and scratch sentence are in `checks/sheet_index.out.txt`. The sha256 of that list of sheet hashes is `b2bad0c0d67f1972ef1988d3c5767f9ce2024ed011a1bb1e89226c11c950ab99`, and `sampling.json` is `e08755f5e9dc04f37a65c250143f14335f267a2b4ed1e2f75f54759ea4299424`.
- **Coverage:** 628 samples and 157 sheets of 4 frames each, with no step between samples over 0.5 s. As `sampling.json` says, this is not proof of continuous motion, so a change between two samples can't show.
- **Receipts:** `frames-inputs.json`, `preview-inputs.json` and `build-inputs.json` all bind `cyclic.rs` `bee439fd…` and `script.tsv` `83619d48…`. Build01 and frames01 receipts exit 0 (inputs `9802b7fc…`, `149a1e6f…`); preview01 exits 0 (inputs `0bc03d69…`).
- **Build log:** 110 tests passed, the Clippy step finished, and the native and oracle facts agree at 240. The log's `unclosed table` lines come from a fixture in a dependency checkout whose path says `malformed`; each step still finished. `integration.json` still says `native_validation: pending` because it predates that build.
- **Layout report:** `10_cyclic.layout.tsv` is 0 bytes.
- **Beyond the sheets:**
  - I enlarged crops of existing frames: 144, 146, 150, 158 and 170 for the 10_3 box; 417, 538 and 384 for glyph faces; 197–202 for the hop; 264–272 for an hour-label colour I then dropped as compression; 279–281 for the title morph; 605–608 for the d ring. My crop boxes for frames 456 and 470 missed their text, so those two rest on sheet scale.
  - Before writing, I looked again at frames 30, 100, 240, 320, 464, 500, 520, 540 and 560 to check my notes.
  - I decoded three single stills from the existing preview MP4 (`2640e278…`) at 88.75, 89.70 and 273.25 s, because half-second samples straddle those short pulses. That was an ffmpeg seek, not a render.
- **Checks:** `checks/sheet_index.py` also measures every unchanged stretch from the frame pixels, with a positive control of 6 known changes and 4 known still moments. `checks/oliver_timing.py` replays the source's beats. Its scratch replay lands within one sample of 10 frame anchors from the sheets, which is why I trust it on Oliver.

## Findings

Times written as `frame N (T)` are scratch-preview seconds. Oliver times are seconds into the cue, take01/take02.

### B. 10_10: the table sits still while the voice moves on

**Voice:** four elements, not automatically cyclic; consider these numbers; each relatively prime, so inverses and closure; five, seven and eleven squared equal one; every nonidentity element has order two; so not the structure of the square's four rotations.

**Picture:** the table appears at the top of the cue, then nothing changes from frame 506 (226.00) to frame 535 (240.50), with the right half of the screen empty. On Oliver the table is full at 0.80 and `squares` arrives at 18.40/18.20, so it stays unchanged for **17.6/17.4 s**.

- **The diagonal is never marked.** At frame 540 the three cells that make the point look like every other cell (`cyclic.rs:1295`, the cell loop).
- **"Every nonidentity element has order two" has no picture.** Nothing changes from frame 537 (241.50) to frame 550 (248.00).
- **"The four rotations of our square" has no picture.** Nothing changes from frame 552 (249.00) to frame 572 (257.17).
- **The whole table leads "Consider the numbers"** by 3.1–3.6 s on Oliver. RubyRaven left this alone as outside their scope.

**Fix (`cyclic.rs:1317`):** `U(12)` alone for "a group with four elements", the table at sentence 1, `Each relatively prime to 12` at sentence 2, `squares` plus a hero tint on the three diagonal cells at sentence 3, `ord(5)=ord(7)=ord(11)=2` at sentence 4, `no` at sentence 5 as now, and `Not the four rotations of a square` at "so this group is not". The longest unchanged gap drops to **7.5/7.3 s**.

All five right-column rows hide with the table at 10_11. Measured against frame 560, the slot at y −0.5 leaves about 26 and 30 px clear of its neighbours, and the table's ink ends at x −1.59 while the widest new row starts at x 0.40. A new test pins the table's closure, its squares and the powers of each unit; its Python mirror fails on two mutated inputs.

### A. 10_2: the second meaning of "order" has no picture

**Voice:** "The whole clock group has twelve elements. These are different meanings of order." This is the chapter's terminology warning.

**Picture:** nothing changes from frame 94 (42.50) to frame 119 (53.33), and no 12 appears anywhere on the right. On Oliver `ord(8)=3` is up at 19.64 and speech ends at 29.44/28.80, so it stays unchanged for **9.8/9.2 s**.

**Fix (after `cyclic.rs:498`):** `Whole clock group: 12 elements` under `ord(8)=3` at sentence 7, both pulsing at sentence 8, hidden with the question at 10_3. By the width law it spans x 0.35 to 4.25; that box is empty on frame 100.

### D. 10_1: the question comes before the move it asks about

**Voice:** defines "generated" for three sentences, reaches "add eight" at 13.36/13.28 and "predict" at 17.64/17.52.

**Picture:** `Add 8: how many positions?` is up by 0.40, during "A cyclic group begins". Frames 4–62 (1.00–26.50) are the longest unchanged stretch in the chapter. This isn't a leak, since the words give no answer, but the viewer is reading about a specific move while the voice explains the general idea.

**Fix (`cyclic.rs:446`):** hold the question to sentence 3. The hold after the voice is untouched: `k.hold`, then `stage.wait(3.0)`, then 10_2.

**What this leaves:** the clock still sits still for about 13 s. A demonstration of "repeat it, run it backwards, include doing nothing" would need a backward step (`step_round` only moves forward) and a move that doesn't give away the add-8 answer. That's a design choice for root, so I didn't patch it.

### E. 10_6: the row that answers the question builds during the question

**Voice:** the question runs 2.64–7.42 / 2.62–7.40, then "The formula says".

**Picture:** the gcd row builds from frame 284 (125.50). On take01 it's complete at 5.94, about 1.5 s before the voice answers.

The map promises a prediction hold only for 10_1, and the gcd row is not the answer itself, so this breaks no rule. It's still the row the viewer needs to answer it.

**Fix (`cyclic.rs:872`):** the `k` row alone during the question, then the gcd and order rows from sentence 2. They finish, with the rule's pulse, at 10.72 on take01, ahead of the boxes at 14.03. The rule's pulse ends 1.2 s later than today, but the boxes still wait for their phrase, so no later beat moves.

### C. 10_4: the equation arrives after its clause

On take01 the ASR hears "and k as" at 11.78 and "k prime" at 14.16; `k_split` appears at 14.60. Take02: 11.68, 13.94, 14.53. On the scratch voice it's later still, frame 202 (90.50) against sentence 3 at 89.64.

**Fix (`cyclic.rs:661`):** show `k_split` at the phrase, then hop. It's a pure reorder: the same beats, the same total, and `coprime` keeps its time in the replay. I checked the hop's label pulses on the decoded stills: at 88.75 `1` is lit with the disc at 4, at 89.70 `2` is lit with the disc at 8.

### F. 10_4: the star nobody mentions (not patched)

While the voice says "Let d be the greatest common divisor of n and k", four triangles draw and the hours tint, from frame 186 (82.50); `4 orbits, 3 positions each` follows. None of this is voiced, and the word "orbit" is first spoken in 10_11. The star is gone by frame 193 (86.00), or at about 9.8/9.9 on Oliver. The label stays until the 10_5 hide, frame 217 (95.93).

The picture does give gcd a meaning, so I wouldn't call it wrong; it just isn't what's being said. I didn't patch it because removal would leave `orbits` and `orbit_color` used only by tests, which needs a decision on those helpers. Keep it, or cut it and move the helpers under `#[cfg(test)]`. **Root's call.**

## Prediction leakage

- **10_1 to 10_2: clean.** Frames 4–62 are identical by pixel check, covering the voice's end at 22.15 and the whole hold to 25.77. The marker first reaches 8 at frame 66 (28.50), after "Add eight and land on eight" begins.
- **10_6:** finding E. No hold is promised there.
- **10_8:** `8=2·4+0` at frame 420 (187.50) shows a zero remainder before 10_9 argues it. It's the worked member, and the hypothetical `10=2·4+2` at frame 465 (207.50) does the arguing. No change needed.

## Labels and arrows

- **Clipping:** none seen on any sheet, including the pulse scale at frames 495 and 204, and block labels against the frame edge.
- **Values checked against the arithmetic:** all 16 table cells, every gcd and order column in 10_6, `8=2·4+0`, `10=2·4+2`, and the 10_3 columns (8, 16, 24 and 8, 4, 0).
- **Glyphs:** the enlarged crops of `m=qd+r`, `5²≡7²≡11²≡1`, `d=min{…}` and `G=⟨g⟩, H≤G` are clean. The 10_3 box fully encloses `3`, `24`, `0` and the tick, top and bottom.
- **Direction:** the 10_2 marker lands at 8, 4, then 0: frame 66 (28.50), frame 70 (30.50), frame 76 (33.50). The 10_11 path's first edge heads to hour 8, and its arcs draw left to right. The 10_7 arc top clears the `d=min` row.
- **`0<2<d` at frame 470 (210.00):** clears both its neighbours and the X above it. The hypothetical landing on an already crossed tick is, to my eye, the strongest beat in the chapter; keep it.

## Teaching and pacing: low, report only

1. **10_5:** ✓ at frame 245 (109.00), about 1.1 s before "The first positive choice" (scratch).
2. **10_6:** the title is garbled by the morph at frame 280 (123.50), and clear by frame 281 (124.00). It's brief, inside the 0.6 s swap; a crossfade would avoid it.
3. **10_5:** block labels turn gold from frame 232 (102.50) while their arcs stay teal, which loosens the colour tie.
4. **Listing order:** `{0,8,4}` at `cyclic.rs:482` is visit order, `{0,4,8}` at `cyclic.rs:988` is sorted. Both are defensible.
5. **10_6:** the final +1/+11 and +5/+7 dials are identical (frame 320), so "different orders" shows only during the tours.
6. **10_9:** the gold ring at 8 stays with no label from frame 492 (221.00) to frame 503 (225.00). The rings at 0 and 4 are both blue through frames 456–464.
7. **`U(12)`** is on screen but never spoken, and chapter 12 reuses the name (`isomorphism.rs:165`). Changing that would mean changing a cue, so I only note it.
8. **10_11:** no hour labels on the recap dial. The last picture change ends at 17.50/17.38 of 25.84/25.52, leaving about 8.2 s still under "rule out a remainder" and the permutations hook. That's a direct result of F4, which I agree with: the removed pulse pointed at the wrong thing.
9. **10_8:** unchanged from frame 421 (188.00) to frame 434 (194.50), while the long sentence catches up to a formula already shown.
10. **10_6 boxes:** by the phrase rule the generator boxes start at 14.03/14.17, but the ASR hears "one, five, seven" at 12.60/13.16. A phrase mark would close that gap if the native run has one; I can't tell from here.

## Repair03 against the pixels

**RubyRaven's first-frame list:**

| Item | Result | Evidence |
|---|---|---|
| 1. 10_3 box clears the tick | Pass | Frames 146–170, enlarged |
| 2. 10_7 title, then line | Pass | Title alone from frame 335 (148.00), line from frame 339 (150.00), before "Let's prove it" |
| 3. 10_7 small clock returns whole, no flash | Pass | Frame 367 (164.00) |
| 4. 10_11 table gone, then clock, then formula | Pass | Table gone by frame 575 (257.67), path from frame 578 (259.00), formula at frame 588 (264.00) |
| 5. 10_11 proof returns clean, arcs left to right | Pass | Frame 598 (269.00) has no arcs, drawn frames 603–605 |
| 6. Chapter length matches baseline | Scratch only | Cue table bytes identical to `algebra-combined-repair-preview01`; not confirmable on Oliver from a scratch preview |

**Adopted fixes:**

- **F1, recap timed to phrases:** pass; see items 4 and 5. The d-ring pulse the recap ends on shows on the decoded still at 273.25.
- **F2, results at sentences 3 and 5:** pass, frame 536 (241.00) and frame 551 (248.50), matching the replay's 240.90 and 248.33.
- **F3:** pass, item 2.
- **F4, last pulse removed:** pass, nothing changes after frame 606 (273.00).
- **F5, order label pulses alone:** pass, frame 593 (266.50).
- **F6, test coverage:** present in source, and 110 tests passed.

## The staged patch

`patches/cyclic-picture-followup.patch`, sha256 `dbb3a64aebb68076edd1be98a24e2709c807cc980a613f1f29d89e8790491d19`: 12 hunks, 92 added, 17 removed, against `bee439fd…`. It produces `bee0eb2c55f2b9ad43fcd10b4f0993947b7fd104cda963e63ed27ccfefa4f17f`. **The two hashes start alike, so compare them in full.**

| Finding | Hunks (in file order, from 0) | Result alone |
|---|---|---|
| A 10_2 | 3 | `d8b37282b641` |
| B 10_10 | 0, 1, 7, 8, 9, 10 | `05beefda26a8` |
| C 10_4 | 4, 5 | `2816a9c2e766` |
| D 10_1 | 2, 11 | `ebf784219c1e` |
| E 10_6 | 6 | `4484c5db8342` |

**What I ran (`checks/patch_fidelity.py`):**

- The patch applies at fuzz 0, forward and reverse, and an independent hunk walker gets the same bytes.
- Each finding's hunks apply alone, and each result passes `rustfmt --check`.
- `k.say` order is unchanged, all `until_sentence` indices are in range and in order within each cue, and every phrase sits in its sentence.
- New text widths come from the CMR10 law, which matched three labels measured on these frames (3.98 against 4.00, 3.75 against 3.73, 2.84 against 2.84 units).
- In the replay, every patched beat in the five touched cues ends inside the voice, on the scratch voice and both takes, so no cue should start later.

**What root must do on integration:**

- **Rebind the source.** The map's 10_1 `visual_holds` row must move from line 448 to line 453 with the new sha, and `narration_units.json` follows the new map hash. Recompute the line if you take a subset; hunks 0, 1 and 2 all shift it.
- **Expect 111 tests and still 240 facts.** The new labels add no facts. `{N}` supplies the 12s, and the literal `5`, `7`, `11` and `2` in `ord(5)=ord(7)=ord(11)=2` are pinned by the new test, not by the oracle, the same way `U(12)={1,5,7,11}` is on screen today.
- **Nothing to synthesize.** No cue text changes, so all Oliver takes stand.

**What the next frames must confirm:**

1. 10_1: no question before sentence 3, and the hold unchanged.
2. 10_2: the new label clears `ord(8)=3` and is gone before the 10_3 table.
3. 10_4: `k_split` before the hop, with `coprime` at the same time.
4. 10_6: only the `k` row during the question.
5. 10_10: `U(12)` alone, then the table; five right-column rows with clear spacing; three tinted diagonal cells, not 1×1; all of it gone before the 10_11 dial.
6. The cue table unchanged for the same voice.

## What I did not do

- **No listening or fine sync.** Oliver numbers are ASR estimates. Phrase times use the kit's fallback, and a real run may use phrase marks.
- **No build, render, TTS, Beads edit or commit.** Nothing compiled the patch, and nothing has drawn it.
- **No canonical edits.** I wrote only in this folder. Root owns integration.
- **Patch review:** the author is the only reader so far.
