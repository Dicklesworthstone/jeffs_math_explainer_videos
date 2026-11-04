# Chapter 11 (`11_permutations`) picture review

LavenderPike, for BlueSalmon, Mail 1788, Bead `mx-video-program-program-87do.64`.

## Disclosure: this is a review of my own code

`explainer_corpus/abstract_algebra_expanded/src/permutations.rs` as rendered is byte-identical to my own `algebra/permutation-followup02` candidate, built on my `permutation-repair01`. The scenes for 11_12 to 11_27 and much of the rest are mine. Within chapter 11, `script.tsv` differs from my candidate only in root's 11_26 claim correction. Treat this as an author's review, not an independent one. Several findings below are defects I wrote.

## Bottom line

- **The mathematics held everywhere I could check it.** No wrong value, cycle, card state, lamp or caption was found (list at the end).
- **No premature answer on any viewer quiz** (11_6, 11_10, 11_12, 11_15). Colour under 11_21's question gives away the split early, but that heading is not a think-time quiz (finding V).
- **Five teaching defects are worth repairing before narration work builds on these pictures:** AD, L, I, AB/M and AA.
- Nothing here is acceptance. These are scratch-voice pixels at half-second samples.

## Coverage

| What | Exactly |
|---|---|
| Sheets | All 400 of `sheet-000` to `sheet-399` read as images, once each, in order. |
| Frames | All 1,597 selected frames are covered at 960x540 through the sheets: a pixel comparison found 0 of 1,597 tiles differing from their frame resized. |
| Sheets reopened after the first pass | 044-063 (all of 11_4) and 074, 077, 078 (the 11_12 quiz hold). |
| Frames opened at 1280x720 (23) | 0030 0120 0186 0193 0399 0412 0419 0567 0600 0955 1040 1161 1170 1190 1276 1289 1375 1411 1412 1413 1414 1415 1484. |
| Source read | The scene code and helpers named below. The tests at lines 4243-4712 were searched and partly read, not read in full. |
| Hashes | `evidence/input-hashes.sha256` (sha256 `af1872553ef48203ab1e2e7acdfcee094e2b270a1aa8a9f621da45dbba0de0d9`, 2,006 inputs, all rechecked OK before delivery). Per-sheet index: `evidence/sheet-index.tsv`. |

Key inputs:

| File | sha256 |
|---|---|
| `src/permutations.rs` | `de3c7b10d96e856ab218d07463a33aa57f97acd41188b1dc267556cfd28e3c68` |
| `script.tsv` | `83619d48676b32d5d93bf7527477aa752a902dd16927c548e4b3052cc2d689c0` |

Limits of this evidence:

- Images leave my working context as I go, so apart from the items reopened above, each finding on sheets 000-399 rests on notes I wrote when I read that batch. `PROGRESS.md` holds those notes.
- Half-second samples plus cue-boundary offsets. Motion between samples was not seen. The MP4 was hashed but not decoded or played.
- Source and script date from before the render (04:56 against 07:32), so these pixels match the current source. `narration_units.json` changed later (08:57) and was not examined; the scratch-voice picture was not reviewed against it.
- Not covered: narrated sync, listening, fine sync, final AV. Timings below are the scratch voice's, not Oliver's.

## Lint disposition

`11_permutations.layout.tsv` has two rows, and both are real and the same object: slot number "4" at (-3.2, -0.48), hidden by the link from slot 4 to slot 1. It persists: absent in frames 181-186 (80.2-81.2 s) and 188-196 (82.0-85.2 s), visible only at frame 180 while fading in and frame 187 (81.5 s) with the links off, then gone with the board at 85.5 s. There are no other rows. Filed as finding L, Medium.

## Severity

- **High:** wrong mathematics on screen or a premature quiz answer. None found.
- **Medium:** a picture that can teach the wrong thing, or that hides or omits what the sentence is about.
- **Low/Medium:** a real teaching cost, but the viewer can still recover the point.
- **Low:** polish.

Line numbers refer to the baseline, sha `de3c7b10`. "Patched" means staged in `patches/permutations.patch`.

## Medium findings

### AD. The eight-shuffle payoff gets no clean frame. Medium. Patched.

- **Where:** 11_24 into 11_25, frames 1411-1415, 631.0-631.933 s. `full_deck` end :3681, `other_way` :3701.
- **Defect:** 11_24 says "Watch the staircase break apart and come back". The staircase and "Shuffles: 8" land at 631.433 s, the same instant 11_25 starts, and `other_way` begins at once. At 631.433 the next captions are already ghosting in, by 631.5 both are garbled, by 631.633 the 8 is gone, and by 631.933 the bars are spreading. No sample shows the restored staircase with a clean count.
- **Repair:** at the end of `full_deck`, pulse `lcm(1,2,8)=8` and the counter together, then hold 0.6 s. This pushes 11_25 and everything after it by 1.2 s.

### L. The lint: the slot card 3 lands in loses its number. Medium. Patched.

- **Where:** 11_4, frames 181-186 and 188-196, 80.2-85.2 s, confirmed on 0186 and 0193. `undo_and_disjoint` :1268.
- **Defect:** the sentence teaches how a cycle moves and undoes, and the hidden number is the destination of one of its moves.
- **Repair:** put the two lower-row numbers under their rings (y = -1.72), clear of every link and of the written inverse.

### I. "Transposition" is spoken but never printed or pictured. Medium. Patched.

- **Where:** 11_5, frames 591-618, 261.0-274.5 s, confirmed on 0600. `construct` :4104-4115.
- **Defect:** "A transposition swaps just two labels. Every cycle can be built from transpositions." plays over the formula `(1 3 4)=(1 4)(1 3)` and three columns of empty rings, 13.5 s before any token moves. A new word, and its formula appears a sentence ahead of its example.
- **Repair:** during sentences 0-1 print "Transposition: swap two labels" and trade two cards. Heading and board arrive at sentence 2, as the example begins.

## Medium/Low findings

### AB and M. Cycle legs read as double-headed, like a real swap. Medium/Low. Patched.

- **Where:** AB: 11_22 and 11_23's start, frames 1280-1311, 573.0-588.136 s, confirmed on 1289, `walk_cycle` :3380. M: 11_4 frames 198-255 (86.0-112.777 s) `undo_and_disjoint` :1311-1315, :1378; 11_12 to 11_17 boards at frames 296-315 and 412+, 445+, confirmed on 0399, 0412, 0419, `loop_arrows` :2216.
- **Defect:** every arc's head and tail share one anchor over a slot. In the six-cycle, legs 2→3 and 9→8 look like the blue double-headed swap 4↔7, so the picture suggests a six-cycle containing two swaps. On (1 3 5), 5→1 looks like a swap of 1 and 5, directly above the real swap (2 6).
- **Repair:** in cycles longer than a swap, a slot's head and tail stand 0.16 either side of its centre, on the sides that stop its two arcs crossing there (`link_offsets`). Swaps keep one shared anchor pair, so the true double head is the only one. In 11_4 the red impossible arc lands exactly on the head of 1→3, which keeps its point: two inputs at one place.

### AA. The ring sits on a card whose number disagrees with the position named. Medium/Low. Patched.

- **Where:** 11_22, frames 1272-1296, 569.0-581.0 s, checked on 1276. `read_the_cycles` :3404.
- **Defect:** the voice says "Two goes to three" while the ring is on card 6. The map is right and can be read, since each arrow from slot n lands on card n, but nothing says so, and reading the card numbers gives a wrong trail.
- **Repair:** a caption during the six-cycle walk, "Card 2 went from slot 2 to slot 3", built from the first trail and checked against the shuffle. My first note suggested dimming the tokens; this is better because it teaches the reading.

## Low/Medium findings

| ID | Cue, frames, time | Source | Defect | Repair | Status |
|---|---|---|---|---|---|
| V | 11_21, frames 1036+, from 460.0 s; 1040 opened | :3113-3120, :3137 | The 24 cells are already split 12 gold and 12 blue, colour unlabelled, under "How many even permutations?", before the pairing is argued (arrows 466.5, caption 477.5). The colour answers the heading early. The colour is `Palette::gold()`; my earlier note wrongly called it hero. | One ink colour until sentence 3, then recolour with a key, "Gold: even", "Blue: odd", between the blocks. | Patched |
| T | 11_20, frames 951-963, 420.5-426.5 s; 0955 opened | :2976-3000 | Heading alone on an empty frame for 6 s during "they compose and each one can be undone". 457.7-459.5 s in 11_21 is the same class, shorter. | Row appears at sentence 0, composes two swaps and undoes them, uncounted, ends home. | Patched |
| Z | 11_10, frames 1184-1203, 528.5-537.5 s; 1190 opened | `shuffle_question` :3285 | Heading alone for 9.5 s; no picture of "a machine mixes a list". | A small row repeating one fixed rearrangement twice, not reaching home, so the question stays open. A design call on spoiling, so left to root. | Recommended |
| X | 11_9, frames 1161-1162, 1165-1166, 519.5-522.5 s; 1161 opened | :2058-2059, :2073-2074 | Each composite runs its two moves in 1.2 s with no rest, so the first move cannot be told from the second. | 0.35 s rest between the moves. | Patched |
| AC | 11_24, frames 1372+, from 613.0 s; 1375 opened | `cycle_strip` :3583, :3651 | The strip sits directly under the bars, so it reads as location. True fixed positions are 1 and 52, the swap is (18 35), and the cycles interleave. | Caption gains "By size:". A visible offset would help more, but is not patched. | Patched in part |
| AG | 11_26, frames 1503+, from 672.5 s | :3847-3866 | Red fourth column has no header, while the caption names "stage 4". | Red "Stage 4" over that column. | Patched |
| AM | 11_27, frames 1532-1538, 684.0-687.0 s; 1562-1566, 699.0-701.0 s | :3897-3900, :3930 | "Seven more" is an uncounted blink, and the first shuffle barely rests. | Patch adds a 0.3 s rest only. `advance` costs 0.3 s per run, so a counter would overrun sentence 1 by about 1.5 s even at 0.1 s a shuffle, and by about 2.4 s at today's 0.22 s. A counter is root's decision. | Patched in part |

## Low findings

| ID | Cue, frames, time | Source | Defect | Repair | Status |
|---|---|---|---|---|---|
| AE | 11_25, frames 1412-1414, 631.433-631.633 s | :3701 | Same-spot crossfade garbles captions and counters. Also 645.5 s frame 1443 (:3739), 657.5 s frame 1473 (:3778). | `swap_caption`: out, then in. Shared `exchange` is left alone because changing it would move timing everywhere. | Patched |
| G | 11_14, frame 399, 177.0 s | :2568 | Same class: "Both loops home" over "lcm(3,2)=6". | `swap_caption` | Patched |
| J | 11_17, frame 567, 252.0 s | :2771 | Same class: "Overlapping cycles interfere" over the rule. | `swap_caption` | Patched |
| AH | 11_26, frames 1495+, from 668.5 s | :3839 | "every line back where it began": the lines never move, the data do. The voice says "data back on their starting wires". | "After 3 stages: all data back on their starting lines" | Patched |
| AI | 11_26, frames 1491-1495, 666.5-668.5 s | :3835 | Three 0.7 s stages with no rest; the middle state never rests on a sample. | 0.25 s between stages | Patched |
| H | 11_14, frame 419, 187.0 s; olive at 105.5 s | `pulse` :801 | `indicate(..., None)` flashes flat bright yellow, off palette, and washes out card 4's number on "Card 4 never moves". | Pass a palette colour. Shared helper, so root's call. | Recommended |
| N | 214.0, 217.0, 245.0, 383.0, 406.5, 440.0, 545.0, 590.0 s | `advance` | Counters lag the landing by one sample. | Advance with the move's last part. Changes many timings. | Recommended |
| O | Frames 646, 683-684, 743, 758-759, 931 | `morph` | One-sample scrambles during formula morphs. Normal for the transform. | None required. | Note |
| A | 11_1, frames 22-47, 9.5-21.2 s; 0030 opened | :3971-3981 | Tokens static for 11.7 s; the promise is kept 1 s into 11_2. | None required. | Note |
| B | 11_1-11_2 | `cycle_trace` :1038 | "Cards" never move until 11_4; only a ring travels. | Optional. | Note |
| C | 11_1, frames 16-20, 6.5-8.5 s | `collisions` :1151 | The harmless 3→3 arrow is red with the others. | Tint only the colliding arrows. | Recommended |
| D | 11_2, frames 94-107, 43-48 s | :4001-4009 | Caption is all gold but only (1 4 3)'s arrows remain. 30-34.5 s static as well. | Colour the caption by arrow. | Recommended |
| E | 11_3, frames 113-128, 49.5-57.0 s; 0120 opened | :4036 | Empty rings 7.5 s. Corrected: column names do tie factors to columns. | None required. | Note |
| F | 11_3 frames 113+; frames 913+ | :909 | "Start" sits 2 px low; "even" 2 px high. Cosmetic. | None required. | Note |
| Q | 11_7, frames 699-705, 309.5-312.5 s | :4173-4183 | Formula alone before its numbers. | Optional. | Note |
| R | 11_11, frames 736-771, 325.0-342.5 s | `witness_proof` | Stale +1008 unmarked for 18 s under cards already showing negatives. A deliberate before-figure. | Dim or label it "before". | Recommended |
| S | 11_8, 373-379.7 s | | No picture for "even permutations form a subgroup"; 11_20 does it later. | Optional. | Note |
| U | 11_20, frames 989-993, 439.5-441.5 s | :3034 | "Undo b" caption lands after the moves, and b's swaps commute, so nothing visibly reverses. | Caption before the moves. | Recommended |
| W | 11_21, frames 1060-1070, 472.0-477.0 s | | "doing it twice changes nothing" has no picture. | Optional: swap one cell there and back. | Recommended |
| AF | 11_25-11_26, frames 1445-1471, 646.5-656.5 s | :3739-3778 | "closed loop" words over a deck visibly away with red "0 cards home", until the 657.5 s exchange. Not showing the 52 is a sensible choice. | None required. | Note |
| AJ | 11_26, frames 1483-1502 | | Composition sits left of centre to reserve the stage-4 column. Justified by what follows. | None. | Note |
| AK | 11_27, frames 1523+ | :3883 | "p" is unnamed on screen. | "p: one out-shuffle". Display text. | Recommended |
| AL | 11_26, frames 1505-1513 | | "repeats the first" has nothing pointing back at stage 1. AG helps. | Optional mark on column 1. | Recommended |

Other same-class `exchange` sites (:1704, :1792, :2494, :2594, :2965, :3440, :3463, :3952) did not show garbling on a sample. The half-second sampling could miss a 0.3 s crossfade, and :3952 at least puts `next` on the same y as `odd`, so they deserve a look on the next preview. They are not patched.

Withdrawn: Y (mini squares). They are the film's own tile poses and agree with the big squares. P (11_6) was checked and is not a defect.

## Quiz holds

| Cue | Prompt on screen | Answer first shown | Evidence |
|---|---|---|---|
| 11_12 | "How many runs? Predict first." from 132.5 s, through the 137.06 cue end, "Runs: 0" and empty strips | After 140.667 s | Sheets 074, 077, 078 reopened: frames 298-315 clean to 139.5 s |
| 11_15 | Both wrong guesses carry "?", empty strips, "Runs" 0 | After 212.867 s | Notes, through frame 479 (212.5 s) |
| 11_6 | Question from 291.5 s to 302.233 s | Not before 302.233 s | Notes, frames 644-681 |
| 11_10 | "How many shuffles bring every card home?" from 553.0 s | Not before 562.067 s; only "Shuffles: 1" and the once-shuffled row show | Notes, frames 1216-1255 |

## Checked correct

Checked from the frames and, where noted, by hand or independent calculation:

- 11_1 maps; 11_2 trail; 11_3 both boards, (1 2 3)(1 2) = (1 3) and (1 2)(1 2 3) = (2 3).
- 11_4 every move: (1 3 4), reverse (1 4 3), rows 5 2 1 4 3 6 and 1 6 3 4 5 2 ending 5 6 1 4 3 2; red 5→3 arc with two heads and halo; (1 3 5)(2 6) = (2 6)(3 5 1). Row names and results clear of the edges.
- 11_13 slots and lamps; 11_14 lamps and frames at 6, 12, 5; 11_16 token states, lcm(4,2) = 4; 11_17 returns at 2.
- 11_5 routes; 11_6 (3 4)(1 4) = (1 3 4); 11_7 values and product 1008; 11_11 -1008; 11_8 parity trail; 11_18; 11_19 table and signs; 11_20 strings and captions.
- 11_21 all 24 strings, 1 + 3 + 8 = 12.
- 11_9 square, S_4, the eight, r = (1 4 3 2), r², rs = (2 4), sr = (1 3), mini squares at full size on 1170.
- 11_10 interleave; 11_22 cycles; 11_23 deck states by hand, lamps, "6 of 10 still away".
- 11_24 and 11_25, independent Python: out-shuffle of 52 has lengths 1, 1, 2 and six 8s, lcm 8, fixed 1 and 52, swap (18 35); in-shuffle is one 52-cycle; 0 home after 1, 2, 4, 8 or 26, all home after 52; 52! = 8.0658e67.
- 11_26 wire map (2 3 5)(4 7 6), first column 1 5 2 6 3 7 4 8, home after 3, stage 4 repeats that order.
- 11_27 the slip: cards 11 and 37 trade at slots 20 and 21 and end in each other's home slots, a single transposition, odd. End frames clean to 714.616 s.
