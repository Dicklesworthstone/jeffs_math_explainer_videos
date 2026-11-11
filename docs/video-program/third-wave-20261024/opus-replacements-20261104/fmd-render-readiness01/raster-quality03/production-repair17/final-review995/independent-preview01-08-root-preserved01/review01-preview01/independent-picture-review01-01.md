# Independent picture review of the 01_the_bump preview extraction

Reviewer: RubyRaven (CC2), 2026-11-11 01:20Z. Root reviewer: BlueSalmon.

## What was reviewed

The existing scope02 extraction on CSD, with no extra extraction:

| Item | Value |
|---|---|
| movie | `jobs/csd-native-combined-preview01/preview/01_the_bump.mp4`, sha256 `005271e97d32246645b8c36340258d9be83b590962be2c97af1cf922c95e6beb`, 6311 frames |
| extraction | `jobs/csd-native-combined-pictures01/01_the_bump/pictures.json`, sha256 `82e3fce62972ae4c9330e9dbe2466c43e0297d902a706b4bf0c4e46e69c70852` |
| job receipt | `resource-receipt.json`, exit 0, finished 2026-11-11T01:00:03.257494Z, inputs_sha256 = final_inputs_sha256 = `19eb221b...4650e29b50` |
| source | the_bump.rs at sha256 `90b51c2b30877911bbb5f5e14ee810268e14efb24e7c4e2a55f7f7e128c06ee7` |

I copied the chapter folder read-only and checked every picture and sheet hash against pictures.json: 120 of 120 pictures and 15 of 15 sheets match, and the sheet ranges are 8 per sheet. The repo copy of pictures.json under final-review995/combined-pictures-root-review01 has the same sha256.

## Exact frames viewed

All 120 selected frames, as their 640x360 cells on all 15 contact sheets (1280x1536):

- singles 36, 48, 60, 72, 84, 96, 108, 120, 132, 144, 156, 168, 348, 648, 948, 1066, 1248, 1548, 1848, 2148, 2448, 2748, 3048, 3348, 3440, 3648, 3948, 4248, 4548, 4848, 5148, 5448, 5497, 5527
- the run 5562 to 5597 (36 frames)
- singles 5600, 5659, 5696, 5748, 5760, 5911, 6000, 6048, 6119, 6155, 6215, 6269
- the run 6273 to 6310 (38 frames, through the last frame)

Enlarged crops from the 960x540 pictures: 48, 120, 1248, 6269 (header, 3x); 6305 to 6308 (ending, 1x); 1548, 1848, 2148 (series, 1x); 3440, 3648 (predict box, 2x and 3x); 5497 to 5696 (14 frames of the thumbnail column, 1x). Pixel measurements: header margins on 14 frames from 48 to 6290, bottom margin on 8 frames, curve heights on 1548 and 1848, and the ending fade on every selected frame from 6155 to 6310.

## Result

No concrete defect. Safe to advance to final 4K on picture grounds. I authored no repair.

### O1. The fraction title-safe flag (Low)

The extraction's layout anchor says the fraction 1/(1 + x^2) stands 0.22 from the top edge where the kit wants 0.25. I measured it on the pictures:

- The fraction's top ink is row 14 of 540 (row 15 at frame 48, while it fades in). That is 0.207 scene units, or 2.6% of the height, and it is the topmost ink in every measured frame.
- The kit's 0.25 units is 16.9 px at 540p and 67.5 px at 4K, so the fraction is short by about 0.04 units: about 3 px at 540p, about 12 px at 4K.
- It also lies outside a 5% title-safe band (27 px) and a 3.5% action-safe band (19 px).
- The header row does not move from 0.8 s to 105.1 s (frames 48 to 6305). The title's top ink stays at row 31.
- The fraction is wholly inside the frame, unclipped and legible in every viewed frame.

I rate this a Low framing note, not a concrete defect: nothing is cut, and on a 16:9 web player the top 5% is not lost to overscan. If root wants the kit's 0.25 kept, moving the header row down by about 0.05 units would clear it. That is a source change, and I have not drafted it.

The other edges in the measured frames:

- The left ink starts at column 110 or later (11%).
- The thumbnail column ends at column 909 (5.2%).
- The bottom ink ends at row 508 (5.7%).
- The predict box border at 3440 and 3648 reaches column 917, a 4.4% right margin. That is outside 5% title-safe and inside 3.5% action-safe. The box text ends near column 900 (6.1%).

### O2. The ending fade

| Frame | Mean distance from background | Note |
|---|---|---|
| through 6275 | 7.07 | full contrast, p99.9 239 |
| 6290 | 5.54 | |
| 6300 | 2.58 | |
| 6305 | 0.88 | |
| 6307 | 0.31 | region maxima 22 / 19 / 18 |
| 6308 | | region maxima 13 / 8 / 8 |
| 6309 | 0.03 | |
| 6310 | 0.00 | every sampled pixel equals the background |

The fall from 6276 to 6310 (104.6 s to 105.17 s) is monotonic and lasts about 0.6 s, matching the source's `fade_out_all(0.6)`. The header, the curve with its ring, and the thumbnail column fall together, so no element leaves early or lingers. The broken outlines visible at 6307 and 6308 are the faded content at under 10% of full contrast in the preview JPEGs. They are not stray ink.

### O3. Frame 1848: the readout leads the curve, by design

At 30.8 s the degree readout shows 8 while the yellow curve still looks like degree 6. The source computes the readout as `2 * ceil(k)`, so it steps to 8 as soon as k leaves 3 in the tween to 5. I measured the yellow curve's heights at x = 0.8, 0.9, 0.95 and 1.0, calibrated by the dashed lines at x = -1 and +1. They fit d6 + 0.07 (d8 - d6), which is k about 3.07.

The settled frames agree. At 1548 the curve is degree 6, with its roots at x = -1 and +1. At 2148 it is degree 40. Whether the lead reads well in motion is a motion question, and I make no claim about it.

### O4. Numbers checked

At x = 1.1 the box shows recipe ≈ 25.2 and bump ≈ 0.452. The sum to degree 40 is (1 + 1.21^21) / 2.21 = 25.24, and 1 / 2.21 = 0.4525. The ring has radius 1, through the poles at i and -i.

### O5. Two vertical scales share the axis (teaching note)

From 65.8 s (frame 3948) the ring of radius 1 is drawn in the x scale, at 92 px per unit, while the dimmed bump keeps its own y scale, about 141 px per unit. So the poles at i and -i sit below the bump's peak, at about 0.65 on the bump's scale. The peak label 1 is gone before this, so no two labels disagree. This note is for root and does not describe a fault.

### Also seen

The "coming up" column builds in order:

1. The color wheel and the ring are in by 5527.
2. The arrow and the half disk fade in from 5566 to about 5586.
3. The dot is in by 5659.

I saw no tile or block artifact in this build. The radius label clears the circle and the graph.

## Comparison with root's receipt

I read root's root-picture-review01.json (sha256 `e4a9c8f8...1140af2a`) only after forming the view above. It lists the same 120 frames and 15 sheets, has no concrete defects, and records three findings: a legible fraction despite the flag, separate thumbnails, and an orderly fade from 6273 to 6310. I agree with all three, and I disagree with nothing. This review adds the margin measurements (O1), the fade measurements (O2), the account of the 1848 readout (O3) and the scale note (O5). Root's scope line leaves the job's terminal receipt to recheck. I checked it: exit 0, finished 01:00:03Z, and the inputs and final inputs hashes are equal.

## Not claimed

These are 960x540 preview stills. This review makes no claim about motion, continuous playback, listening, audio, fine sync, AV, final 4K pictures, or acceptance. The narration was not assessed.

## Files

- `independent-picture-review01-01.json`: the evidence in the gate's pictures() schema. It lists all 120 frames in viewed_frame_indices, all 15 sheets, the 48 runs, the crop frames and the observations, with concrete_defects empty, safe_to_advance_to_final4k true and authored_repair false.
- `make_evidence01.py`: writes that JSON from the copy after re-checking every hash. It refuses an existing output.
- `fraction_margin01.py`, `fade_profile01.py` and `curve_heights01.py`: the read-only measurements.
- `crops01/`: the enlarged crops.
