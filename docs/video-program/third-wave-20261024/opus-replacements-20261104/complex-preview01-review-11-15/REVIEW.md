# Complex preview01: picture review of chapters 11–15

Reviewer: WhiteSwan. Assignment: Agent Mail 1623 from BlueSalmon. Date: 2026-11-04.

**Status: sampled frames read as images, plus source.** I opened every sheet of these five chapters with the Read tool and re-read 15 native frames. Nothing was watched or heard. Preview01 is 720p30 with the scratch voice, not Oliver, so every time below is a scratch time. The sampling is half-second, cue-boundary and layout samples, so it does not prove continuous motion.

**Author and reviewer overlap: this is not a fully independent review.**

- BronzeJaguar wrote the scenes of these chapters (`complex/HANDOFF.md`), and I did not write them.
- I did touch these files once. My `complex/clippy-repair05/clippy-repair05.patch` (Mail 1385, integrated by root) changed `counting.rs`, `draw.rs`, `real_integral.rs`, `residues.rs` and `model.rs`. All of it was Clippy repair: test-only attributes, `const` asserts, doc comments, moved imports, dead code removal (`Walker::leave`, `Func::Quartic`), `Div` routed through `Mul::mul`, and one float literal respelled. The one logic edit is `winding_certified`, rewritten so that a tie or a NaN fails, and I added tests for it. I read every hunk again for this review. None of them draws a picture in 11–15, but `winding_certified` does sit behind facts these chapters rely on (`facts.rs:86, 300, 323, 328`), so my earlier edit is part of the number path I'm reporting on here.
- This session's shared memory holds a complex lane note written under BronzeJaguar's name, so I can't claim an untouched view of the lane.
- **The candidate patches below are my own work. Nobody has reviewed them.**

## Result

**The mathematics holds on everything I checked, but the pictures are not ready.** Of the four predict windows, three hide their answers and 11_5 does not. There are 5 high findings, 30 medium and 31 low. The high findings are not cosmetic: in four places the picture doesn't show what the voice is arguing, or shows the answer early, and one shared arrow default draws wrong directions.

| ID | Chapter, cue | Finding in one line |
|---|---|---|
| A1 | 12, 13, 15; shared | Arrows are 0.5 scene units shorter than asked, and an arrow asked between 0.25 and 0.5 points backwards. 12_1's chains have 38 backward arrows |
| 11-E | 11_4 | "The output shoots off to infinity" plays over an empty output plane |
| 11-F | 11_5 | The answer to the quiz is on screen before the question and through the think time |
| 11-L | 11_7 | "The leash is shorter than the walker's distance from zero" has no visible leash |
| 14-J | 14_5 | "The partial sums grow without limit" plays over bars that hit a flat ceiling |

Eight candidate patches cover all five high findings and 21 of the others. They change no cue text. They are uncompiled and unrendered, so none of these repairs is proved until root builds and renders them.

## What I read

Frames are in `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-opus-rebuild-frames01/`. Digests of the sheets, frames, sampling files, MP4s, cue tables and layout tables are in `checks/bindings.out.txt` (exit 0).

| Chapter | Sheets read | Frames on them | Sampled span of chapter |
|---|---|---|---|
| 11_counting | 000–130, 131 of 131 | 0–523, 524 | 0.000–207.600 of 207.766 s |
| 12_residues | 000–041, 42 of 42 | 0–164, 165 | 0.000–71.216 of 71.466 s |
| 13_real_integral | 000–118, 119 of 119 | 0–473, 474 | 0.000–194.883 of 195.133 s |
| 14_continuation | 000–113, 114 of 114 | 0–454, 455 | 0.000–203.783 of 204.033 s |
| 15_closing | 000–055, 56 of 56 | 0–221, 222 | 0.000–76.916 of 77.166 s |
| **Total** | **462** | **1840** | |

Chapter 11 sheets 028–048 and 049–076 count on their second reading only, because the first images dropped out of my context. `PROGRESS.md` records that, and sheet-by-sheet notes for every batch.

An (n) marks one of the 15 native 1280×720 frames I re-read: 11 frames 33, 200, 380, 455; 12 frames 40, 150; 13 frames 93, 420; 14 frames 170, 228, 370, 398; 15 frames 34, 94, 200. Source lines refer to canonical files at the hashes in `checks/bindings.out.txt`. I re-checked that `base/` still equals canonical after making the patches.

## High

### A1. Arrows stop short, and short ones point backwards

**Where:** every `Arrow::new` in these chapters: `chain_arrows` `draw.rs:967` (called at `residues.rs:137`), `residues.rs:122`, `real_integral.rs:203, 207, 529, 530, 536`, `continuing.rs:602`, `closing.rs:95, 117`. None of them sets a buffer.

**Cause:** at fmn pin a399d626, `Arrow::new` (`crates/fmn-library/src/line.rs:490`) defaults to a 0.25 scene unit buffer at each end (`crates/fmn-core/src/constants.rs:33`). Asked for length L, it draws L − 0.5. Under 0.5 the trimmed ends cross and the arrow points backwards. Under 0.25 the head's own dimensions also go negative, so the head flips forward again.

**Numbers**, from a Python mirror of `Arrow::build` (`checks/patch_numbers.out.txt`), not from a run of fmn:

| Arrow | Asked | Drawn | Head |
|---|---|---|---|
| 12_1 chain of 1/z², 24 steps | 0.261 | | **Backward, 24 of 24** |
| 12_1 chain of h(z), 24 steps | 0.145–0.378 | | **Backward 14**, forward 10 |
| 12_1 chain of 1/z, 24 steps | 0.100 | | Forward 24, by the double flip |
| 12_1 loop direction | 0.451 | 0.049 | **Backward** |
| 15 box 3 direction | 0.351 | 0.149 | **Backward** |
| 15 box 1 carry | 0.575 | 0.075 | Forward, no shaft |
| 13 contour, along and back | 0.625, 0.623 | 0.125, 0.123 | Forward |
| 13_8 p1, p2 | 1.042 | 0.542 | Forward, gaps at both ends |
| 13_8 p1 + p2 | 1.473 | 0.973 | Forward, gaps at both ends |
| 14_8 hop | 1.154 | 0.654 | Forward |

**On the picture:**

- 12 frame 40 (n), with enlarged crops: h(z) arrows on the right half and top run clockwise, against the true walk, while the left half runs the true way. The chain seems to flow down both sides. The 1/z² path is a hairline with no readable heads (12-A), the h(z) chain has gaps (12-C), and the 1/z stack's top stretch shows no heads (12-D). The loop's direction mark is a nub (12-B).
- 13 frame 420 (n): the pink pair and the yellow total stop well short of their points, so the tip-to-tail sum never touches (13-Q).
- 15 frame 94 (n): the box 1 "arrow" is a head with no shaft (15-D), and box 3's direction tick is a speck (15-F).

**Repair:** `.buff(0.0)` on all of these except the 14_8 hop, which still reads correctly and which I left for root to decide. The numbers show direction will be right. Whether a chain of very short arrows reads well can only be judged on a render.

**Outside this review:** `chain_arrows` is also called at `loop_sums.rs:315, 368, 466, 481` (chapter 08), and bare `Arrow::new` sites exist in `loop_sums.rs`, `gallery.rs`, `one_point.rs`, `one_derivative.rs`, `the_bump.rs`, `turning.rs`, `reading_color.rs` and `two_planes.rs`. I didn't view those pictures. BronzeJaguar's 02-B reports the same short arrows in chapter 02.

### 11-E. No blow-up under "shoots off to infinity"

**Frames 180–215, 78.5–93.5 s, cue 11_4; (n) frame 200. `counting.rs:250-255`.** The voice says "the loop runs through the pole, and the output shoots off to infinity". The output axes hold only "0" throughout: no walker, trace, arrow or number. Nothing moves at all from 86.0 to 92.333 s except the red card landing at 86.5.

**Repair (11-counting):** `approach_pole`. A walker travels the circle |z| = 1.35 towards the pole and stops 0.03 short. The output is drawn at true scale, on its true line Re w = 2/3, and clipped at the top of the window, where an arrow and "off the window" appear. A live |f| readout climbs 0.67, 0.73, 0.95, 1.49, 2.60, 4.72, 8.72, 16.17, 30.01 and settles on the fact `30.0`, labelled "0.03 from the pole". No value is ever drawn at the pole.

### 11-F. The quiz shows its answer

**Frames 217–251, 93.5–108.5 s, cue 11_5. `counting.rs:263`.** The chapter has just taught that a pinwheel marks a zero. The quintic's colouring arrives at 93.5 s with five pinwheels, and the unit circle (94.0–94.5) holds exactly one. The question card lands at 98.0, the voice ends at 101.579, and the ring drains from 102.5 to 108.0. So the answer is readable before the question and through the full think time. The hold's timing is right, and this is what spoils it.

**Repair (11-counting):** during 11_5 the plot goes to plain axes with the unit circle (`painter.to(stage, None, 1.2)`). The quintic is painted at the start of 11_6, after the think time. This also removes the loop drop in 11-H.

### 11-L. No visible leash

**Frames 356–399, cue 11_7; (n) frame 380. `counting.rs:679-683`.** The voice's whole argument is "The leash is shorter than the walker's distance from zero". The leash is drawn, but the `squash` compression places a walker at |w| = 243 about 111 px from 0, and shrinks a leash of length 8–10 to at most 2.0 px radially and 4.6 px sideways. Both ends sit inside the dots, whose radii are 8.1 and 7.2 px. On screen, two dots overlap.

**Repair (11-counting):** keep the compressed walk, since its direction is exact, and add printed rows "walker's distance from 0: 243" and "longest leash: 10" with two bars on one scale (3.000 and 0.123 scene units). A note row says "drawn: n = 5, on the circle |z| = 3", which also answers 11-M. Both numbers go through `facts.rs` and the oracle.

### 14-J. Bars that plateau under "grow without limit"

**Frames 226–228 and on, 100.5 s on, cue 14_5; (n) frame 228. `continuing.rs:402`, `:145`.** Bars for 1, 3, 7, 15, 31 rise, then 63 to 2047 all clip to one height with no mark. The voice says "The partial sums grow without limit", and the picture shows levelling off. A test confirms the clip was intended; what's missing is any sign that the bars leave the chart.

**Repair (14-continuing):** chart top 2200 with no clip, so 2047 stands at 3.35 of 3.6 scene units and the doubling is visible at true scale. A caption "z = 2: 11 terms give 2047" comes from the oracle. The cost is that the early bars become slivers (0.002 to 0.1 scene units for the first six).

## Medium

"P" names the patch that addresses it. Rows without one are report-only. Some rows are rated medium/low or medium/high, as noted.

| ID | Frames, scratch time | Defect | Patch |
|---|---|---|---|
| 14-L, medium/high | 352–380, 157.5–171.5; (n) 370 | Colouring frozen for 14 s under "near a dense set of points on the edge it blows up". No rim marks, and the right rim near z = 1 looks calm | P 14: rows and 64 rim ticks by level on "near a dense set" |
| 11-K | 323–343, 133.0–139.7 | Empty output axes alone for 6.7 s under "every polynomial has roots" and "degree n" | P 11: claim, then p(z) = zⁿ + ⋯ |
| 11-M | 344–399 | "Huge" has no scale. The picture uses n = 5 and nothing says so while the voice says n | P 11, with 11-L |
| 11-A | 57–110, 23.0–46.167 | Output plane stays empty through "the turns add". Left panel blank 59–64. Names arrive about 4.4 s after spoken. "Each counted by its order" is not on the card | |
| 11-B | 25–28, 39–41, 55–57 | Results flash: "turns: 1" 1.5 s, "turns: 2" under 0.5 s, "turns: -1" about 0.27 s | P 11: 0.8 s hold after each walk |
| 11-C | 16.0–22.733 | The pole walk plays almost wholly after its sentence ended (20.231). Recheck on Oliver | |
| 11-N | 113–175, cue 11_3 | Markers unlabelled while the voice says 0.45, 1.35, radius 0.8 | |
| 11-O | 136–138, 59.0–60.0 | "Grow the circle" blinks off and redraws larger | |
| 11-V | 463–521, 187.0–207.0 | 20 s static, last 10.5 s after the voice. The reach circle is drawn at one sample only, so "cannot move as far as its distance" has no second sample | |
| 11-H, medium/low | 252–256, 108.667–110.0; `counting.rs:271` | The loop drops out for about a second on the answer cue | P 11 |
| 11-I, medium/low | 255–318 | Cyan −3z circle never labelled | |
| 12-F | 97–103, 41.5–44.0 | Blank frame for 2.5 s under "The turns we counted are…" | P 12: rows arrive on S0 |
| 12-J | 131–164; (n) 150 | Bump plot has no i, −i or axes. The voice says "At i" | P 12: i and −i named |
| 12-E, medium/low | 68–95, 29.5–41.067 | Static 11.6 s while the voice continues | |
| 12-K, medium/low | 56–58, 23.5–24.5 | Deformation carried by text alone over a static picture, with no direction marks on any loop | |
| 13-C | 45–47 and on, 18.3–19.0 | The half-disc loop fades in whole while the voice walks it | |
| 13-D | 45–71 | No −R, R, i, −i on the plane. The voice names all four | P 13 |
| 13-E | 89–96 | "its size along the half circle" sits under \|z²\| alone for about 4 s, so "its" reads as z² | |
| 13-L | 118.5–132.9; (n) 420 | Yellow arrow points up, unlabelled, while the answer −0.35i points down. Easy to misread as the residue sum | P 13: "p₁ + p₂" named, and a downward "sum" arrow −(p₁+p₂)/4 |
| 13-O | 134.5, frame 320 | The whole equation "2πi·(−i√2/4) = π√2/2 = π/√2" arrives about 7–9 s before the voice reaches the result | P 13: two stages |
| 13-P | 409–431+, 173.2 on; (n) 420; lint real | "precisely:" card halves the two lower pole dots and covers "225°", "315°" | P 13 |
| 13-H, medium/low | 123–139 | Right panel says R = 10 while the plane's loop stays at R = 2 | |
| 14-M | 398–399, 177.5–178.0; (n) 398; `continuing.rs:578-579` | The punchline flashes: the whole gamma colouring, poles included, shows at partial strength for up to about 0.5 s, 15 s before the reveal, because the cover is added after `gamma.show` | P 14: cover first |
| 14-B | 48–64, 19.5–27.5 | Formula and two blank baselines for 8 s. "Inside the unit circle" has no disc | |
| 14-C | 70–95, 30.5–41.467 | Red and blue panels have different scales and the red has none, so the bar for 32.15 stands level with the blue 2 | P 14: blue "2" drawn on the red chart |
| 14-D, medium/low | 77–78, 34.0–34.5 | "= −5" leads "It equals minus five" (39.14) by about 5 s | P 14 |
| 14-E, medium/low | 100–119 | Nothing on the plane is labelled: no 1, small disk or scale | |
| 15-A | 32–40, (n) 34; again in 15_3, (n) 200; `closing.rs:238-239`, `:303-304`; lint real | The radius-1 circle cuts "i" and crowds "−i". At sheet size "−i" reads as "−1", though it reads better at native size. The lint named only "i" | P 15 |
| 15-E | (n) 94; `closing.rs:129` | Box 4's arc touches the pole rings, so the poles read as on the contour, which chapter 11 said is forbidden | P 15: radius 1.22 |
| 15-G | 193–210, 66.5–72.2; (n) 200; `closing.rs:376` | The film's closing numbers: the circle runs through the decimal points of "−0.16" and "6.16", which read "-0\|16" and "6\|16", and the axis runs through "−0.16" too. The lint missed it. My inference, not run: the strokes pass the small decimal point and so fall outside the glyph cores `check_crossed` tests | P 15 |

## Low

| ID | Frames, scratch time | Defect | Patch |
|---|---|---|---|
| 11-D | 55–56 | "turns: -1" has an ASCII hyphen; the counter has a true minus | |
| 11-G | 238–249 | The predict ring sits on the plane's top border and the axis shows through it | |
| 11-J | 294–318 | "precisely:" card abuts "one root inside" and covers the plot's bottom edge. Legible, and on the cue | |
| 11-P | 156–175 | Inset has no arrowheads, so "circle once and then unwind" is read only from the sum. Low/medium | |
| 11-Q | 412–419 | Left half blank beside the code. `step` is not defined on the card | |
| 11-R | 412 on | Code line height about 17 px at 720p. Judge at 4K | |
| 11-S | 452–459; (n) 455 | "reach covers 0" is red on red with the axis through it. Also the real lint entry `11_8 +22.4` ("reach L × step" over the axis). Low/medium | |
| 11-U | 461–479 | "8 samples say 0 turns" stays beside a 64-sample picture | |
| 11-W | 268 on | Lowest root marker rests on the card edge | |
| 11-X | (n) 33, 200; `draw.rs:534` | Row banding and stair-stepped colouring cells. Cells are fixed in scene units, so this may survive to 4K. **Judge on a 4K still** | |
| 11-Y | 522–523 | The last sampled frame (207.600 s) still shows everything at about 40 percent, 0.166 s before the chapter ends. **The samples don't show whether the fade finishes.** Check the join | |
| 12-G | 116–127; `residues.rs:42` | "the turns of chapter 11:" is a production number on screen. Low/medium | P 12 |
| 12-H | 116–127 | "f'/f" in roman beside math-set f′/f | |
| 13-B | 22–41, 9.5–17.5 | Static 8 s through "a second way". Low/medium | |
| 13-F | 90–93; (n) 93 | The moving "R" lands on the pole ring at i. Low/medium | P 13 |
| 13-G | 123–126 | R label and bound disagree mid-sweep: "R = 2 / 1.294", "R = 4 / 0.672", "R = 7 / 0.400", against true 2.094, 0.838, 0.458. The end state "R = 10 / 0.317" is correct. A paused frame shows a false pair. Low/medium | |
| 13-I | 179–191 | Degree labels small and dim. Not re-read natively | |
| 13-J | 225–239 on; (n) 420 | "135°" touches "p₂", "p₁" abuts "45°". Low/medium | P 13 |
| 13-K | 239–254 | g′ never written. Low/medium | |
| 13-M | 120–127 | No number for "imaginary part √2/2". Low/medium | |
| 13-N | 320 area | Roman i beside italic i | |
| 14-A | 0–23 on | No axes or i, −i, radius 1 labels on the plot. Cell edges as in 11-X, not re-read natively. Low/medium | |
| 14-F | 116–132 | "agree everywhere" has no picture. Low/medium | |
| 14-G | 152–167; (n) 170 | Pole ring under the discs reads as a knot. Downgraded after the native read. Low/medium | P 14 |
| 14-H | 148–164 | "Chain disks along a path" has no path drawn. Low/medium | |
| 14-I | 193 on | Card repeats the series, with "..." against the title's "⋯" | |
| 14-K | 312 area | 1 and −1 not marked on the √z panel. Low/medium | |
| 14-N | 406 area | Mask edge has no "Re z = 0" label. Low/medium | |
| 15-B | 21–43 | Bump and plane share an axis at two unlabelled scales. Low/medium | |
| 15-C | 54 on | "the same from every direction" sits beside a quotient with no limit sign. Only the limit is the same. Low/medium | |
| 15-H | 40, 102–129 | Holds of about 0.6 s after the voice at the 15_1 pay-off picture and the 15_2 recap. "though not always everywhere" is spoken but not on the card. Low/medium | |

**Withdrawn after a native read:** 11-T (the highlight band reaches neighbouring lines but clips no glyph at 720p), 13-A (the lower half does fill).

## Layout lint verdicts

Chapters 12 and 14 have no warnings. I checked 13 entries by eye (`layout.tsv` wording is in `PROGRESS.md`):

| Entry | Verdict |
|---|---|
| `11_1 +14.0`, `11_6 +6.1`, `hold @124.7` (two) | False positive for legibility, (n) 33. The shape is a row strip of the painter under an opaque card |
| `11_8 +22.4` | Real, low/medium, in 11-S |
| `13_11 +1.4`, `+1.6` (four) | Real, 13-P |
| `15_1 +0.8` | Formula 0.22 from the top against 0.25. Real but small, not patched |
| `hold @12.2`, `15_3 +1.9` | Real, 15-A |

15-G is a real defect the lint didn't report, so a clean lint isn't enough here.

## What holds

The evidence is the frames read and source, and where stated the oracle.

- **Maths on screen:** walk directions and turn counts in 11_1, including order 2 at +2 and the pole at −1. The 11_6 counter agrees with the pointer angle measured on four sheets. |z⁵ + 1| ≤ 2 < 3 = |−3z| on |z| = 1. The z⁸ demo and the 11_8 code logic. Residues 1, 2, −1 and 1/(2i) = −0.5i. 13's antiderivative, πR rectangle, p⁴ = −1 ⇒ 1/p³ = −p, Res = −p/4, and the sum −i√2/4 = −(p₁+p₂)/4. The eight tangent discs in 14 (each drawn radius is the true distance to the pole, centres 0.765 apart). √z's half angle, ζ(−1), and the gamma poles at 0, −1, −2, −3, −4.
- **Printed numbers:** 0.335, 6.28 and 0.79, 3.14159, 2.094, 0.654, 0.317, 2.2214, 2.22, 0.4189, 0.0031, 1.999023, 32.15, −0.083333, 3.162, −0.16 and 6.16 each match a row of the canonical `oracle.py`, which I ran. The mid-sweep values in 13-G are the exception.
- **Predict windows:** 13_5, 14_4 and 15_3 hide their answers until the think time ends. 11_5's timing is right, but 11-F spoils it.
- **Strong beats to keep:** the three-term residue device, the 14_1 dim-and-return on "Of course not", the re-centred disc chain, the natural-boundary framing, the gamma strip reveal apart from 14-M, the √10 circle in 15, and the title ending.

## Candidate patches

All are in `patches/`, relative to the repo root and against the canonical hashes in `checks/bindings.out.txt`. Result trees are `base/` and `candidate/`. `checks/replay_patches.out.txt` records each patch applying to a fresh copy of `base/` and giving bytes equal to `candidate/`.

| Patch | SHA-256 | Fixes |
|---|---|---|
| `11-counting.patch` | `4d397fa0064c2ffcb18465d3bad80af5cf27495afe5ba3c7f02c4c542e8da1ca` | 11-B, 11-E, 11-F, 11-H, 11-K, 11-L, 11-M |
| `12-residues.patch` | `d003142012bb26a5effa9f030110306720ba07f1c1e58ea996287045dfecc02b` | 12-B, 12-F, 12-G, 12-J |
| `13-real-integral.patch` | `eb9af851436c6025f90275f24172b4f3ebe582aa99edaabb2e75177c186d6118` | 13-D, 13-F, 13-J, 13-L, 13-O, 13-P, 13-Q |
| `14-continuing.patch` | `1c247ad41514f3d6782d8a53a78c4dcb7738a6389eb67d28097eb2e2e15f34e5` | 14-C, 14-D, 14-G, 14-J, 14-L, 14-M |
| `15-closing.patch` | `fb5cefa4009c3954fbdc9866a4ab3a8a6ef66077bb122fc5edc9d2d3f2891ae5` | 15-A, 15-D, 15-E, 15-F, 15-G |
| `shared-chain-arrows.patch` | `2ecb308676b0d1f1983a79a04bee2cac35ef62480937a1e726ef23bd07cb09e1` | A1 chains: 12-A, 12-C, 12-D |
| `shared-facts.patch` | `a18733320c501e745444e93acdac63edfa05ec2dad4921f8056071a2ef8a3daf` | Six new facts for 11 and 14, in `oracle.py`, `facts.rs`, `model.rs` |
| `shared-registry.patch` | `47ae8c5c5705c1a15f922cc8a96e1d22336708731381a1c2a5fccfd2364256ff` | Glyph supplements for new strings in 11, 13, 14 |

**Order:**

- `11-counting` and `14-continuing` need `shared-facts`.
- `shared-registry` names constants from 11, 13 and 14, so apply it after them.
- `12`, `13`, `15` and `shared-chain-arrows` stand alone.

**Shared changes and who they affect:**

- `shared-chain-arrows.patch` changes one line of a shared helper. Callers: `residues.rs:137` (reviewed) and `loop_sums.rs:315, 368, 466, 481` (chapter 08, **not viewed by me**).
- **Don't apply this one twice.** LavenderPike's in-progress `complex-picture-repair-06-10/` `draw.rs` already has the same `.buff(0.0)` at its line 1000. If that lands first, drop my patch.
- `complex-picture-repair-02-05/` and `-06-10/` also stage `draw.rs` with different bytes. I didn't check whether they also touch `mod.rs`, `facts.rs` or `oracle.py`, so these patches may need a merge rather than a clean apply.
- `shared-facts.patch` adds six rows and changes no existing fact. Candidate `oracle.py` prints 126 rows against 120, with none changed or gone: `argument.rational.near_pole.distance` 0.03, `.size` 30.0, `argument.quintic.walker` 243, `.leash` 10, `series.doubling.at` 2, `.n10` 2047. It also adds the constant `MOEBIUS_NEAR_POLE`.

**Cue text changes: none.** `script.tsv` stays at `2968b0fd7654…`. These patches add on-screen text only, so the voice is unaffected, but picture timing moves. 11_1 can run up to about 2.4 s longer, from the three holds.

**New tests in the patches, not run:** `the_rows_print_the_lengths_the_walk_really_has`, `the_walk_to_the_pole_is_finite_and_its_output_runs_up_a_straight_line`, `the_rim_ticks_mark_where_the_series_blows_up`. Existing tests in 14 and 15 were updated to match.

**Where a build is most likely to complain:**

- `VALUE_RESULT_TEX` starts with "=" and the shortened `VALUE_TEX` ends with "=". The typesetter may treat a lone relation differently from the original whole line.
- `Painter::build(stage, None, …)` followed by `to(Some(Func::Gamma))` is a new path in 14-M.
- The glyph probe hasn't seen the new strings.
- Clippy, and the source-reading test `each_card_outlasts_its_reading_window`, which I reasoned about but didn't run.

**Repairs to re-judge once rendered:**

- 11-E, whether 0.5 of the sentence is enough for each leg of the walk.
- 11-L, whether a 0.123 scene unit bar reads beside a 3.000 one, which is the point of it but is thin.
- 14-J, the slivers.
- 15-G, because "−0.16" now sits 0.56 to the right of its tick, inside the circle. It is clear of strokes by the numbers, but a viewer must still connect it to the tick.

## Not patched, for root to weigh

None of the report-only rows is a mathematical error except 13-G's mid-sweep pairs. The ones I'd do next are 11-N, 11-V, 13-C and 14-B: each leaves a spoken statement with no matching picture. They need new staging decisions, so I left them out of a review's candidates.

## Checks run

| Check | Result |
|---|---|
| `checks/bindings.py` | Exit 0. Counts and digests above |
| `checks/replay_patches.py` | Exit 0. All 8 patches apply, and 10 files equal `candidate/` while `base/` equals canonical |
| `rustfmt --check` on the 9 Rust files | Exit 0. Parse and format only (`mod.rs` by stdin). Checks no types |
| Canonical and candidate `oracle.py` | Exit 0, rows above |
| `checks/patch_numbers.py` | Runs clean. Numeric mirror of `Arrow::build`, the pole walk, leash, bars, rim, label clearances and 13_8. **A mirror, not fmn** |

## Not done, and still open

- **Nothing is compiled.** No `cargo` build, test, Clippy or `--lint` ran on the patches, and none of them is rendered, so no repaired picture has been seen by anyone.
- **Scratch voice only.** No Oliver audio is in preview01, so all timing findings need rechecking against Oliver. 11-C especially may change.
- **No listening, fine sync or independent AV review.** No synthesis. All of this stays open.
- **Sampled frames, not motion.** Fades, walks and sweeps are judged from half-second samples.
- **720p.** Thin strokes, cell edges (11-X, 14-A) and code size (11-R) need a 4K still.
- **Native re-read not done** for 13-I and 14-A's cell edges. Every other finding without (n) rests on sheet panels at 960×540.
- **Chapters 1–10 pictures were not viewed by me,** including the chapter 08 chains that `shared-chain-arrows.patch` changes.
- **Patches are unreviewed.** They need an independent read as well as a build.

## Corrections to my working log

`PROGRESS.md` is kept as written. Two entries are superseded here:

- 14-M's anchor there is `continuing.rs:579-580`. The correct lines are 578–579.
- Its note that chapter 11's fades "are cut off by the chapter end" went beyond the evidence. The samples stop 0.166 s before the chapter ends, so it's an open question (11-Y).
