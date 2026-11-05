# Logarithm final master: picture review of sheets 113–225

Reviewer: BronzeJaguar (pane 168), 2026-11-05. Assignment: review contact sheets 113 through 225
of `logarithm-stage09-final-verify02`.

This is a sampled picture review. It is not independent final AV acceptance, and it accepts
nothing. Nothing here was heard or watched in motion.

## 1. Result

All 113 sheets were opened and read, each sheet hash matches `verification.json`, and the master
hash is bound.

- **Mathematics:** no mathematical error was observed in the sampled frames.
- **Overlaps:** no overlap between text and text was observed. Text clearances that looked tight
  on a sheet were checked on 4K frames and cleared. One non-text crossing was observed: the
  countdown bar crosses an axis tail (O3).
- **Prediction answers:** no explicit answer was visible on any sampled hold frame of the seven
  cards.
- **Observed items:** one standards deviation (O1), one notation slip (O2), four cosmetic items
  (O3–O6) and one transition fact for root or the owner to judge (J1–J3).
- **Largest open judgment:** U1, whether the 06_2 hold telegraphs its answer.

## 2. Binding

Values are in `binding.json`, recomputed when this folder was written.

| Item | Value |
|---|---|
| Master | `narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage09-render-work02/logarithm_mysteries_master.mp4` |
| Master sha256 | `00236d1a6915ff968235a30ab59769b02125ece0fee482e4fa3154c265ac7c24`, equal to `verification.json` |
| Master size and job | 200,701,388 bytes, 116,245 frames at 60 fps, job `logarithm-stage09-master02` |
| `verification.json` sha256 | `22fb77188d6e9d86a0c9499e7a81db782d4b29830d1ddb4be40fe4dd3003fd62` |
| `command.log` sha256 | `a236e77e25993b51d9e2beccd8a85bf115aae6156aeb6c5361611df618cf622f` |
| `script.tsv` sha256 | `559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a` |
| Sheet hashes | 113 of 113 equal `sheet_sha256` in `verification.json` |

The span runs from 05_1 (646.42 s, frame 38785) to 08_5 (1292.43 s, frame 77546), pictures 0449
through 0900.

## 3. What was and was not examined

| Evidence | Count | Where recorded |
|---|---|---|
| Sheets opened, each holding 4 sampled frames at 960×540 | 113 of 113, so 452 pictures seen at sheet scale | `viewed-sheets.tsv` |
| Recorded 960×540 frames opened individually | 10 | `frames-inspected.tsv` |
| Recorded frames hashed or measured, not opened | 7 | `frames-inspected.tsv` |
| 4K frames pulled from the master into scratch | 21: 1 opened whole, 9 viewed only as crops, 11 measured only | `frames-inspected.tsv` |

- **Proof of viewing.** I parsed the session transcript for Read calls that returned an image.
  Every sheet in the span has one. Its call id is in `viewed-sheets.tsv`.
- **Sampling.** The sheets show 452 of about 38,760 frames. Anything brief between samples is
  unseen.
- **Not done.** No hearing, no motion review and no fine sync check. Frames that were not sampled
  were not inspected, apart from the 4K frames pulled for this review. Sheets outside 113–225 are
  outside this review.
- **Why 4K frames.** The recorded `frame-NNNN.png` files are 960×540, the same size as the sheet
  cells, so they cannot settle a close clearance. I pulled 21 single frames with niced, two-thread
  ffmpeg seeks into scratch. There was no full decode. Scratch PNG hashes depend on the ffmpeg
  build, so re-extraction may not reproduce them. Downscaled, the frames I compared differed from
  the recorded ones by a mean of 1.73–1.98 levels, and the content matches. That is not a
  frame-identity proof.

## 4. Observed defects and deviations

Seen on sampled frames. Fixes are suggestions, every one needs a render, and all of them are
root's decision. I changed nothing.

| ID | Severity | Where | Observed | Suggested fix |
|---|---|---|---|---|
| O1 | Low to medium, standards deviation | All 7 cards: 05_1, 05_5, 06_2, 06_4, 07_3, 08_1, 08_4 | None of the sampled cards carries "pause for more time". `draw.rs:352 predict` draws only the bar. `REVIEW-AND-CONTINUITY.md` lines 74–77 ask for those words on the card. | Add one small line to `predict`, or record a decision to omit it. |
| O2 | Low, notation | Sheet 140, picture 0559, frame 47005, 783.42 s, cue 05_7 | "real part ln 2 = 0.693" uses `=` for a rounded value. The film's own source writes `ln(2) ≈ 0.69` in `bookends.rs:39`. | `complex_turns.rs:718`: change ` = ` to ` ≈ `. The glyph is in the film's character list (`mod.rs:29`), and the spoken 05_7 line does not say the number. |
| O3 | Cosmetic | Sheet 135, picture 0538, frame 44878, 747.97 s, cue 05_5 | The countdown bar crosses the tail of the output vertical axis at about x = 662, y = 455 in cell pixels. It clears the outer circle. | `complex_turns.rs:636`: move the bar under "which inputs?". |
| O4 | Cosmetic | Sheets 145 and 150, pictures 0578 and 0598 | The live readout leaves about 25 cell pixels between the number and "πi" ("1.86    πi"). | Close the gap in the readout layout. |
| O5 | Cosmetic | Sheets 156 and 159, pictures 0623 and 0634 | The teal side circle reaches x ≈ 414, past the end of the horizontal axis at x ≈ 393. | Extend the axis or leave it. |
| O6 | Cosmetic | See the list below | Small alignment differences. | Root's choice; none affects reading. |

O6 items:

- Sheet 186, picture 0743 (07_3): the bar spans 554–757 under text at 540–732, neither centred nor
  left-aligned. The 08_1 bar is centred.
- Sheet 217, picture 0866 (08_4): the `=` column sits at about 551 on some lines and 572 on others.
- Sheet 220, picture 0880 (08_5): the red box spans 513–825 around content at 572–696, so it is
  lopsided.
- Sheet 195, picture 0779 (07_6): the title "Two cautions" sits about 2 cell pixels lower than
  other titles.

### J1–J3: chapter joins are one-frame hard cuts to an empty background

Observed, not judged as a defect. Root or the owner decides whether this is intended.

| Join | Last full frame | First empty frame | Time | Sheets |
|---|---|---|---|---|
| J1, chapter 05 to 06 | 48391 | 48392 | 806.53 s | 143, 144 |
| J2, chapter 06 to 07 | 58962 | 58963 | 982.72 s | 175 |
| J3, chapter 07 to 08 | 69144 | 69145 | 1152.42 s | 199, 200 |

- Frame means fall from about 19.3–19.7 to 16.33 on a single frame. All three empty frames are
  byte-identical PNGs with a maximum of 20 and no pixel above 100.
- The final frames were measured, not opened.
- The empty picture lasts at least 0.6 s, and the title is up by about 0.5 s after the first cue.
- The audio side of these joins was not heard.

## 5. Unverified concerns

These are judgments or gaps in the evidence, not observed defects.

| ID | Where | Concern | What would settle it |
|---|---|---|---|
| U1 | Sheet 150, picture 0598, frame 50777, 846.28 s, cue 06_2 | The card "back at 1: logarithm = ?" holds with the readout at 1.86 and the blue dot just under the marked purple dot at 2πi. The word "2πi" appears as a tick label, not as an answer, and the source stops at 0.93 of a lap on purpose. It may still give the answer away against "no hint". | A viewer's judgment, or stop the climb earlier, near 1.5. |
| U2 | Sheets 139 and 169, "k whole", "whole rungs only" | "Whole" with negative k matches the spoken "whole number", but "integer" is stricter. | An editorial choice. |
| U3 | Dim labels and ticks at sizes 18–20 | Measured contrast is fine (section 9). Size at phone scale was not tested. The film's fine print elsewhere uses 22. | A check at the target viewing size. |
| U4 | Sheet 190, 1082.32 to 1082.72 s; sheet 199, 1147.82 to 1148.32 s | Labels, dashes and the right column clear within 0.4 s, and polar turns to strip within 0.5 s. These may read as abrupt. | Motion review. |
| U5 | Sheet 168, picture 0670, frame 56012, 933.53 s | The rewind shows no gap dot. It looks transitional. | Motion review. |
| U6 | Sheet 181, picture 0722, 1022.53 s, cue 07_2 | Log i and Log(−i) are already up 0.5 s into the sentence that speaks them, which may run ahead of the voice. | AV review. |
| U7 | Sheet 196, cue 07_6 | Only +0.850, +0.851 and −0.850 were sampled. The readout near ±1 is unseen. | Frames across that passage. |
| U8 | Whole span | Sync between picture and voice was not judged. Silence during holds is known from cue data only. | AV review. |

## 6. Candidates checked and dismissed

These looked like possible problems on a sheet. Closer evidence cleared them.

| Candidate | Evidence | Outcome |
|---|---|---|
| "−1" label touching the circle | f44781, 4K crop | About 8 4K pixels clear |
| "k whole" against the axis | f46800, 4K crop | 28 pixels clear to the right, 40 to the left; placement is deliberate in the source |
| Lap count against the trace | f43469, 4K crop | "laps: 3" matches a 3.25-lap trace |
| Dot and axis tail against "L(z)" | f57585, 4K crop | 16–23 4K pixels clear |
| Fractions in "cut upward" | f65563, 4K crop | Clear |
| Spiral start and self-crossing | f66445, 4K crop | Starts at 0, no self-crossing seen |
| ½(2πi) under the axis end | f77178, 4K crop | About 24 pixels clear |
| Crowded row under the strip | f64363, 4K crop | No overlap, about 76 pixels between items |
| Purple stack, 08_3 | f74244, 4K crop | Clear |
| ½ inside exponents | f77178, 4K crop and 1080p downscale crop | Legible at both |
| Bars of 06_2 and 06_4 | Frames 0598 and 0634 opened | Clear of axes and text |
| 07_1 trace showing only the lower half | `branch_cut.rs:288–291` with sheets 177–178 | Explained: the walker runs to +0.47 of a lap, turns back and runs to −0.47. Motion not viewed. |

## 7. Mathematics checked on sampled frames

No error was observed in any of these:

- **Chapter 05:** 1.2 × 1.5 = 1.8; 30° + 50° = 80°; e^{ln 2 + iπ/2} = 2i; grid radii against e^x;
  the 45° ray; the column 2.6π, 0.6π, −1.4π for the 0.6π dot.
- **Chapter 06:** stem height equals angle; the side-circle logarithm loop returns; the gap flips
  after −1 with sign −1; k goes from 0 to −1 at z = 1.
- **Chapter 07:**
  - ray count of the polar grid;
  - strip spacings consistent with ln of the radii, read from sheets to about two figures;
  - yellow and purple edges just inside ±π;
  - the upward-cut window (−3π/2, π/2);
  - the 0.85π dot at about 153°;
  - Log(e^w) = w ⟺ −π < Im w ≤ π;
  - Log(e^{2πi}) = 0;
  - top edge included, bottom edge left out;
  - Arg(−1) = π stated as a convention, consistent with that half-open window.
- **Chapter 08:**
  - z at 135°;
  - window-line positions ¾π, 3/2π, −π/2;
  - the arrow of length 2π, 209 of 210 pixels;
  - k ∈ {−1, 0, 1} and k = −1 for z²;
  - the steps of the √−1 puzzle, the box on the correct step, and the half-turn arcs.

## 8. Prediction timing

Full table in `prediction-timing.tsv`. Times are from picture data and cue placements. Silence was
not heard.

| Cue | Question up by | Cue end | Bar start | Last hold frame | Next cue | Answer first seen |
|---|---|---|---|---|---|---|
| 05_1 | 646.92 | 648.95 | 649.55 | 653.60 | 653.80 | Arc 654.30, text 656.35 |
| 05_5 | 744.67 | 746.87 | 747.47 | 751.52 | 751.72 | 752.22 |
| 06_2 | 844.97 | 845.17 | 845.78 | 849.83 | 850.03 | 852.45 |
| 06_4 | 880.75 | 882.96 | 883.57 | 887.62 | 887.82 | 892.38 |
| 07_3 | 1058.20 | 1059.66 | 1060.27 | 1064.32 | 1064.52 | Morph by 1067.57, labels 1069.98 |
| 08_1 | 1178.63 | 1180.94 | 1181.55 | 1185.60 | 1185.80 | "outside" 1186.30, answer 1190.57 |
| 08_4 | 1256.43 | 1260.20 | 1260.80 | 1264.85 | 1265.05 | Box 1265.55 |

- Cue end to next cue is 4.85–4.86 s on all seven. The bar runs about 4.05 s. That meets the 4–6 s
  standard at its low end, and `THINK` in `draw.rs` is 4.0, marked there as a proposal until heard.
- Every frame at a next-cue start shows no answer. Answers appear after the countdown ends.
- For 08_4, "Which step broke?" exists in the voice only. The card reads "so −1 = 1 ?".

## 9. Contrast

Full table in `contrast-2160p.tsv`: WCAG ratio of the brightest 0.1 percent against the box median,
on 4K frames, 22 rows. Use the 2160p values. Lanczos overshoot inflates downscaled figures.

| Group | Measured |
|---|---|
| Dim text and ticks | 6.24–6.43 |
| Purple text | 4.98–5.11 |
| Red "cut upward" | 6.45 |
| Title, as a control | 16.99 |

No item falls below the 4.5:1 text threshold on this peak-against-median measure. Thin glyphs may
read lower than a peak suggests, which is one reason U3 stays open.

## 10. Disclosure

- **Source authorship.** This is not independent of the source. The shared memory of this workspace
  records that sessions on it staged logarithm repairs in these chapters: repair 05–07 (Mails 1484
  and 1531), WhiteSwan's repair 08–10 (Mail 1512) and RubyRaven's rebuild. BronzeJaguar also did
  stage09 audio and timing reviews (Mails 4378, 4380, 4518, 4613, 4700). I treat those ties as
  real.
- **Reading source.** I read candidate source to explain frames and to locate fixes. That shaped
  some judgments, for example the 07_1 dismissal.
- **Actions taken.** 21 single-frame extractions and hash reads of the master, all read-only.
  Nothing was rendered, synthesized, decoded in full, published or committed. No Beads, TODO or
  production file was touched.
- **Not claimed.** Hearing, motion review, viewing of frames not listed, or acceptance of any kind.
