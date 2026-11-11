# 08_loop_sums: independent preview picture review

CC2 (RubyRaven), 2026-11-11, 01:09Z. This review is independent of root's 08 review. I formed it from the extraction alone and did not read root's 08 evidence first. CSD was read-only, and I started no jobs.

**Verdict: no concrete defect in the 484 selected pictures. Safe to advance to the final 4K render.** This covers still pictures from the preview movie only. Final 4K pictures, motion, audio and listening, fine sync and AV all stay open.

## What I reviewed

| Item | Value |
|---|---|
| Movie | preview01 `08_loop_sums.mp4`, `63984b44...` |
| Extraction | `csd-native-combined-pictures01/08_loop_sums/pictures.json`, `57dba91d...` |
| Extraction job | `resource-receipt.json`: exit 0, finished 01:00:03Z, inputs `19eb221b...` unchanged |
| Source | `loop_sums.rs`, `a8d62f56...` |
| Selection | 484 of 14959 frames, on 61 sheets |
| Evidence | `independent-picture-review08-01.json`, `7f33ead5...` |

**Checking the copy.** I copied the extraction read-only. Every picture and sheet hash matches the record. The selected frames equal my own reproduction of root's scope02 rule (`sheet_map02`). The required run 10200 to 10380 is complete: 181 frames.

**How I viewed it.** I saw every selected picture as its 640x360 cell in all 61 contact sheets. For the frames below, I also cut regions from the 960x540 pictures and enlarged them:

| Crop | Frames | Scale |
|---|---|---|
| c on the quarter | 10785, 10800, 10836, 10846 | 5x |
| 4^n strokes | 12336, 12420, 12444, 12456 | 3x |
| 4^n strokes | 12336, 12420 | 4x |
| c leader in the nested squares | 6990, 7000, 7026 | 5x |
| delta disk around c | 9150, 9168, 9195, 9740 | 5x |
| triangle arrows | 13536, 13788 | 3x |
| footer box (pixel measurement) | 14688, 14736 | none |

`viewed_frame_indices` in the JSON is all 484 selected frames. `viewed_runs` lists them as 75 runs of consecutive frames. The longest are:

- 10200 to 10380;
- 9740 to 9800;
- 13052 to 13105;
- 10785 to 10818;
- 10024 to 10052;
- 9139 to 9168;
- 6986 to 7012.

## The checks root asked for

1. **The c at 179.8 s, frame 10788.** There is one c glyph, with no doubled or offset strokes at 5x. Its weight matches the c at 10785, before the quarter fills in. It sits on the amber fill at lower contrast and stays legible.

2. **The 4^n strokes at 205.6 s, frame 12336.** Each stroke is a single diagonal through its own 4^n. The base and the exponent stay readable. The strokes touch neither the fraction bar, nor εD_0P_0, nor |I_0| ≤. The fade has started by 12456.

3. **The L12 fade-out, frames 13052 to 13081.** Everything fades together. The c does not linger:
   - at 13076 only faint traces remain;
   - 13078 and 13079 are black;
   - "The same proof, now on a triangle" fades in from 13080.

4. **The required run, 10200 to 10380 (L11 perimeter).** The green leg from c follows the yellow point counterclockwise round the square. It starts at the lower-right corner, goes up the right side, along the top, down the left side and along the bottom, and closes at the corner at 10337.
   - The point stays on the boundary in every frame.
   - The leg always meets c.
   - The c and D_n labels fade out from 10227 while the leg crosses their places. They fade back in with "perimeter P_n" from 10339, and are full by 10347.

## Mathematics

- **The opening (3748).** ∮ z^k dz is 0 for k = −3, −2, 0, 1, 2. For k = −1 it is 2πi, and its Riemann-sum steps line up along the imaginary direction.
- **The nested squares (6360 to 7100).** The proof supposes |I_0| > 0. Then I_0 is the sum of the four quarters, and the triangle inequality gives |I_{n+1}| ≥ |I_n|/4. The squares nest as Q_0 ⊃ Q_1 ⊃ Q_2 ⊃ ⋯.
  - The c leader lands on the limit point.
  - The c label sits clear of the square edges.
- **The expansion (7566 to 10483).** f(z) = f(c) + f′(c)(z − c) + r(z), and the constant term integrates to zero. With |r(z)| ≤ ε|z − c| on |z − c| < δ, |I_n| ≤ εD_nP_n.
  - The delta disk is drawn around c.
  - The c label stays outside the disk.
- **The quarter (10846 to 12776).** D_nP_n = D_0P_0/4^n leads to |I_0| ≤ 4^n|I_n| ≤ εD_0P_0. That holds for every ε > 0, so I_0 = 0.
- **The triangle (13188 to 14736).** Midpoint subdivision gives four triangles of half the size, so D_n = D_0/2^n and P_n = P_0/2^n.
  - At 13536 each small triangle has its own counterclockwise arrows, including the inverted middle one. I checked this at 3x: each inner edge carries two opposite arrows.
  - At 13788 the circular arrow in the middle triangle turns counterclockwise, matching "The middle walk is also counterclockwise".
  - D_n marks the longest side.
  - The footer reads: "Goursat: differentiability at the nested limit point bounds the remainder; no continuity of f′ is assumed". This is correct.

## Observations (not defects)

- **O1.** From 10788 to 13073, the c sits on the amber quarter at lower contrast. It is still legible.
- **O2. The footer box at 14688 and 14736.**
  - The box's ink spans x 26 to 933 of 960, and down to row 527 of 540. That leaves margins of 2.7% at each side and 2.2% at the bottom.
  - Its text spans x 43 to 822 and rows 471 to 510: 4.5% from the left edge and 5.4% from the bottom.
  - So the border, and the first 0.5% of the text, fall outside a 5% title-safe area. Everything is inside the frame and legible.
  - I record this measurement in case root applies a title-safe rule to 08. The 01 review checks one for a fraction. I do not treat it as a defect.
- **O3.** The c and D_n labels are absent during the perimeter walk, from 10235 to 10338. This keeps the leg from crossing them. It is a design choice.
- **O4.** At 6360 and 6506, "I_0 = I_1 + I_2 + I_3 + I_4" numbers the four quarters 1 to 4. Elsewhere, I_n means the n-th nested integral, so the two notations overlap. This is in the script, not a picture fault, and I ask for no change.

## Not claimed

I make no claim about final 4K pictures, motion, listening, fine sync, AV or speed. The Frederick narration is not assessed. I authored no repair.

## Files

All in `review08-preview01`:

| File | Contents |
|---|---|
| `independent-picture-review08-01.md` | this report |
| `independent-picture-review08-01.json` | the evidence |
| `make_evidence08.py` | the script that wrote the evidence |
| `notes08-progress01.md`, `notes08-progress02.md` | viewing notes |
| `crops01/` | the enlarged crops |
| `verify_actual01.py` and its `.out.json` | the hash check of the copy |
| `sheet_map02.py` and its `.out.json` | my reproduction of the selection |
