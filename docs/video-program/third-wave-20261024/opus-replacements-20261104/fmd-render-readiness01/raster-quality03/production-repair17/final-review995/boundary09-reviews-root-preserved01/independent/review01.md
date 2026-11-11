# 09_boundary repaired preview pictures: CC0 independent review

CC0 (LavenderPike), 2026-11-11T01:23Z. CC3 authored repair752, so this review is independent of the repair's author.

**Verdict.** I found no concrete defect in the 1506 selected pictures. P1 is still open as a judgement for the owner or root, and there are five Low observations and one note. On these stills, the candidate03 changes behave as their windows expect.

**Scope.**
- These are still pictures from the preview movie: 960x540 pictures, viewed in 640x360 sheet cells.
- I viewed every selected frame in its cell on all 189 sheets, and opened 26 pictures at full size.
- No motion, fine-sync, AV or final-4K claim. No audio heard or checked. The Frederick narration is unchanged and not assessed.
- I made no pixel comparison with master993 and processed no media.
- I launched nothing and ran no extraction. CC3's local copy was the only input.

## Inputs, checked before viewing

| Input | Value |
|---|---|
| Extraction record | `csd-native-combined-pictures01/09_boundary/pictures.json`, `e2f1c855…` (CC3: equal on CSD before and after the copy) |
| Movie | preview01 `09_boundary.mp4`, `da4df9de…`, 16504 frames, full decode |
| Source | candidate03 `boundary.rs`, `4b689791…` |

**`verify01.py`.** Every picture and sheet hash matches the record. The job receipt is a success with inputs unchanged, and the three stderr files are empty.

**`index01.py`.** The selection is missing none of the frames it should hold:
- plan02 required frames: 0 missing;
- plan02 extended frames: 0 missing;
- prior775 frames: 0 missing.

Both lint anchors are selected: 9408 (picture 791) and 13392 (picture 965).

Of root's four 343 instants (±2 frames), only 2129 and 2130 are in the selection. The ones near 1661, 1914 and 2347 are not, so I did not see those exact frames. They fall in the control window, where plan02 has nothing missing.

## What I viewed

- **All 189 sheets**, every cell. That is pictures 1–1506, which are exactly the record's `selected_frames`. Pictures are numbered in selection order, and picture k is `selected_frames[k-1]`.
- **Full size, 26 pictures:**

| Picture | Frame |
|---|---|
| 329 | 5310 |
| 384 | 5760 |
| 440 | 6300 |
| 512 | 7090 |
| 518 | 7096 |
| 519 | 7097 |
| 565 | 7180 |
| 606 | 7320 |
| 658 | 7640 |
| 684 | 7680 |
| 796 | 9456 |
| 889 | 11520 |
| 965 | 13392 |
| 1010 | 13672 |
| 1088 | 13750 |
| 1176 | 13860 |
| 1200 | 14430 |
| 1252 | 14954 |
| 1290 | 15581 |
| 1330 | 15621 |
| 1360 | 15651 |
| 1372 | 15663 |
| 1468 | 15780 |
| 1490 | 16260 |
| 1500 | 16473 |
| 1505 | 16502 |

  `independent-picture-review09-01.json` has one note for each.

## The candidate03 windows

**M2: names and leader in.**
- The old a and ρ leave with the small circle's heads at about 7067–7076.
- The bar, the riding a and ρ, and the leader fade in at 7077–7090.
- At 7090 the leader is drawn across the small circle's top.
- Neither a nor ρ jumps between 7096 and 7097.

**Shrinks.**
- a is never crossed in either shrink (shrink1 runs 7097–7138).
- At rest (7680) a dark gap to the hole ring is visible. The leader is a short stub, and its 4K gaps cannot be resolved at 540p.
- The zero formula's swap is at about 7728–7751.

**Proof gone (7980–7996).** The leader, the riding names, the circles and the bar fade together. No orphan leader is left.

**M4, 09_7 growth.**

| Frames | a's name |
|---|---|
| 13658–13676 | eases out |
| 13677–13735 | hidden |
| from 13736 | faint |
| about 13750 | full |

At 13672 the ring's radius is about 10 px, while a sits about 19 px from the centre. When a comes back, the ring is already outside it. The ring never draws over a visible a.

**09_7 after the growth.**
- The radius-1 ring passes through both poles from about 13837.
- The radius line to i and the "radius 1" card arrive at about 14340. Both are clear of the ring and of a.
- At 540p, a looks the same before and after the growth. I did not compare it with master993's pixels.

**09_8: slide and card.**
- The ring, the card and the interval bar fade at 14821–14833.
- During the slide (14833–14940), a rides the dot at a fixed offset in every sampled frame.
- The "1" card fades in at 14944–14954, about 10 px below a.

**Predict at 15202.** The countdown ring runs from 15202 under "recipe centred at 1: works from ? to ?". The card fades at 15583–15595.

**M4, 09_9 growth.**
- The √2 line and card fade in at 15596–15611.
- a's name fades at 15619–15622, is hidden at 15623–15657, shows faint from 15658, and is back by about 15666.
- The ring's bottom meets the top of the "1" card at 15651, and the card's opaque fill hides that arc until about 15705.
- At rest (15758 on) the ring passes through i and −i and meets the √2 card at its top-left corner.
- The interval (−0.41, 2.41) appears by 15810, and the two "60 terms" lines by 16140.

**Ending.**
- The labels go at 16470, and the plate is alone at 16473.
- The plate fades from 16474 to 16502, and 16503 is background.
- The faint outlines at 16502 are the tail of that fade. Nothing in them is legible.

## Findings

**P1 (open judgement, not a concrete defect), frame 7180.** Held at MID_R, the leader still reads as a radius from the centre through the circle's top to a, rather than as a pointer to the fixed hole. It is the owner's or root's call.

**O1 (Low), M1 bars at 5760, 6300 and 7320.**
- The closing-bar gap is visible in ARC_TEX, SUM_TEX and SHRINK_TEX, at about 2–3 px at 540p.
- By eye it is narrower than the opening bar's gap, which is about 6–8 px.
- So "a visible gap" holds, but "like the opening bar's gap" does not hold by eye. A 4K measurement would settle it if root wants one.

**O2 (Low), SUM_TEX right edge at 6300.** The final "0" sits about 17 px from the right edge at 960, which is about 0.25 unit and right at the kit's limit. A ~1% rescale cannot be resolved at this size.

**O3 (Low, pre-existing), lint anchor at 13392.**
- The bump plot's "x" label ends about 12 px (0.18 unit) from the right edge, which matches the lint.
- It stays there from 09_7 to the end; I saw it at 13860, 14430, 14954, 15581, 15780 and 16260.

**O4 (Low, pre-existing), frame 11520.** The 09_5 "precisely:" card covers the bottom ~15 px of the plate.

**O5 (Low), √2 card at 15780 and 16260.** At rest the √2 ring meets the √2 card at its top-left corner. It reads as passing behind the card, as the window expects, but the fit is tight.

**O6 (note), a's tint at 14954, 15581, 15780 and 16260.** Over the saturated red plate, the thin a reads pinkish at 540p, but white over other colours. It looks the same before (15581) and after (16260) the 09_9 growth.

**Pre-existing control items, not findings:**
- At 5310, "inside the dashes" sits about 5 px above the "same endpoints" box, with no overlap.
- At 9408 and 9456, the 09_3 plot's vertical axis ends under the opaque "opposite samples share a value" pill. It reads as intended.

## Files (this folder, frozen)

| File | What it is |
|---|---|
| `verify01.py`, `verify01.out` | hash checks of CC3's copy against the record |
| `index01.py`, `index01.out`, `index01.tsv` | every sheet cell with its frame, plan02 windows and expectation |
| `make_evidence09.py` | rehashes all 1506 pictures and 189 sheets, then writes the evidence |
| `independent-picture-review09-01.json` | the evidence, in the gate's `pictures()` schema: `concrete_defects` `[]`, `safe_to_advance_to_final4k` true, `authored_repair` false, `viewed_frame_indices` = all 1506 selected frames; the claim-word rule finds no keys |
| `progress01.md` | working notes up to sheet 106 (superseded by this file) |
| `review01.md` | this report |
