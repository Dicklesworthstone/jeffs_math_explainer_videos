# Measurements: preview02 chapters 11 and 12

Read-only measurements on the sampled frames, plus decoded frames where noted. Frames are 1280×720 at 90 px per scene unit: scene x = (px − 640) / 90, scene y = (360 − py) / 90. Boxes are frame pixels (x0, y0, x1, y1). All times are on the scratch voice.

## Methods

- **Contrast:** WCAG ratio from frame pixels. Text regions compare the modal colour with the most distant 1.5% of pixels. Thin type reads lower this way, so treat these as lower bounds. The F fill uses the mean of a box inside the fill.
- **Extents:** bounding box of pixels more than 60 summed RGB from the field colour [12, 17, 20].
- **Column runs:** the same test by column, for gaps along a text line.
- **Still spans:** consecutive samples with fewer than 30 px changing by more than 36 summed RGB. A change shorter than a sample can't be seen.
- **Scratch sentence starts:** the kit's rule, `Narrator::scratch`: cue length = words / 2.6 + 0.4, with sentence starts as character shares of length − 0.4.
- **Phrase times:** my estimate, continuing that character share inside the sentence. The kit doesn't compute these.

## Contrast

| Item | Frame, time | Box | Foreground | Ratio |
|---|---|---|---|---|
| Faint before-F fill, stem | ch 11 f24, 10.5 | (327, 395, 340, 420) | [53, 61, 75] | 1.74 |
| Same, with gold F present | ch 11 f33, 15.0 | same | [53, 61, 75] | 1.74 |
| "before" word, dim | ch 11 f24, 10.5 | (322, 300, 375, 318) | | 6.08 |
| Gold after-F fill | ch 11 f33, 15.0 | (258, 343, 308, 427) | | 12.73 |
| "where A puts the F", red 20 pt | ch 11 f375, 171.0 | (915, 390, 1072, 412) | | 5.63 |
| e^iπ = −1, purple | ch 11 f524, 240.5 | (560, 130, 720, 200) | [131, 106, 146] | about 4.0 |
| "complex exponential forgets the laps", purple 20 pt | ch 12 f96, 40.5 | (50, 590, 372, 610) | | 4.70 |
| "−3πi", purple 16 pt | ch 12 f62, 26.5 | (307, 565, 335, 585) | [136, 111, 147] | 4.31 |
| Turning-line tick labels, dim 16 pt | ch 12 f320, 143.5 | (805, 558, 1170, 576) | [128, 132, 138] | 5.06 |
| "time" word, dim | ch 12 f300, 133.5 | (690, 183, 735, 203) | [140, 146, 160] | 6.09 |
| Laps value, purple | ch 12 f300, 133.5 | (972, 183, 1012, 203) | [153, 114, 171] | 4.84 |
| `precisely` line | ch 12 f350, 158.5 | (72, 672, 1200, 696) | | 6.09 |

Blended prediction for the R7 proposal, `Palette::dim()` over the field: alpha 0.55 gives 2.66, 0.65 gives 3.25, 0.75 gives 3.94.

## Extents and clearances

Lint text margin: right edge under x 6.86.

| Item | Frame | Measured |
|---|---|---|
| "signed area" word | ch 11 f69 | px y 106–124, x 690–831 |
| Live value | ch 11 f69 | px y 140–155, x 695–741 |
| "det" word | ch 11 f137 | px y 106–123, x 690–828 |
| Live value | ch 11 f137 | px y 140–155, x 695–741 |
| "0" label and zero line | ch 11 f69 | Label px x 677–683, y 435–444; red line px x 697–753, y 439–442 |
| Gap caption | ch 11 f313 | x 2.200–6.000 |
| Key and word | ch 11 f375, f386 | Key's right edge px 897–899; word starts px 914 |
| "equal stretches" | ch 11 f432 | Right edge x 5.756 |
| "2×2" lines | ch 11 f450 | Right edge x 6.022 |
| Rule | ch 11 f503 | Right edge x 6.022, y −3.011 to −1.656 |
| "same end point" | ch 11 f230 | x 5.400–6.856 |
| X card | ch 11 f283 | Right edge x 6.778 |
| Log line and border | ch 12 f62 | Text ends px 594; border px 607–609 |
| "−3πi" and sentence | ch 12 f62 | Label px y 570–580; sentence px y 590–606 |
| Axis stub | ch 12 f62 | Ends px y 585, x 135–136 |
| `precisely` line | ch 12 f350 | x −6.311 to 6.300, y −3.722 to −3.500 |

## Gold over the ×2 arrow

Row y 387, x 190–315 (126 px), counting pixels with R > 190, G > 150, B < 150.

| Frames | Gold pixels |
|---|---|
| f160–f167 (72.5–76.0) | 0 |
| f168 (76.5) | 75 |
| f169–f221 | 76 at every sample |
| f222 (100.5) | 0 |

## Red pixels at the start of 11_5

Box (140, 150, 560, 620), R > 200, G < 130, B < 120: f283 0, f284 0, f285 387, f286 388.

## Still spans of 5 s or more

Chapter 11, start–end (length): 13.0–19.5 (6.50); 20.5–28.533 (8.03); 51.5–60.7 (9.20); 83.5–89.5 (6.00); 95–100 (5.00); 101–108.5 (7.50); 109.5–116 (6.50); 117–126.633 (9.63); 129.5–137 (7.50); 137.5–142.5 (5.00); 143.5–152 (8.50); 170.5–175.5 (5.00); 198–206 (8.00); **206.5–221.5 (15.00, f450–f480)**; **222–232.5 (10.50, f481–f502)**; 248.5–255.05 (6.55).

Chapter 12, start–end (length): 8–14.7 (6.70); 26–31.5 (5.50); 54–60.5 (6.50); 81–88.5 (7.50); 89.5–95 (5.50); 148–154 (6.00); 155–163.016 (8.02).

These are scratch lengths; Oliver's lengths will differ.

The chapter 11 plane region alone, box (100, 170, 530, 600) against f425: 0 changed pixels from f424 193.5 to f512 235.833; 16,080 at f513 236.0, as it fades.

## Think bars

| Cue | Evidence |
|---|---|
| 12_5, line ends 99.69 | Widths at f231–f238 in sheet px of a full 229.5: 218, 190, 161, 132, 103.5, 75, 46, 17. Linear fit 100.30–104.30. No bar at f230 100.0; gone at f239 104.5. Next cue 104.567. |
| 11_3, line ends 89.177 | f198 90.0 about 193 of 202.5 sheet px, so it starts about 89.81. Gone at f206 94.0. Next cue 94.067. |

## Decoded frames

Decoded from `logarithm-opus-rebuild-preview02/render/12_what_it_forgets.mp4` with ffmpeg into scratch space, with absolute times confirmed by `showinfo` under `-copyts`. These aren't sampled frames and they aren't a motion review. The file is CRF 26, ultrafast.

| Run | Frames | What they show |
|---|---|---|
| 110.700 + k/30 | 10 | time/laps: k0 .96/1.43, k1 .97/1.45, k2 .98/1.47, k3 .99/1.48, **k4 110.833 1.00/1.50**, **k5 110.867 0.00/0.00**, k6–k9 0.00/0.00 |
| 126.600 + k/30 | 10 | Pin tip dot beside and touching the end ring, pin shaft through it, red line at the rim |
| 32.400 + k/30 | 20 | k0–k3 no movers; **k4 32.533 all three in the ring**; k5 on they separate from there |
| 114.400 + k/30 | 24 | Laps chip absent through k4; starts at k5 114.567, so the 2½ run ends there; full brightness by k14 114.867 |

## Phrase times (estimates)

| Cue | Phrase | Time |
|---|---|---|
| 11_1 s2 | "the letter F" | 10.15–11.01 |
| 11_1 s4 | "squashes the whole plane flat onto a line" | 24.51–27.45 |
| 11_2 s3 | "run backwards" | 41.97–42.93 |
| 11_3 s6 | "The area is multiplied by plus one" | 80.38–82.88 |
| 11_5 s1 | "send it along both routes" | 131.92–133.78 |
| 11_5 s2 | "end in different places" | 140.86–142.57 |
| 11_6 s3 | "running one after the other" | 180.86–182.91 |
| 11_7 s5 | "blocks for each negative stretch" | 228.66–230.87 |
| 11_7 s5 | "come in matching pairs" | 230.94–232.46 |
| 12_2 s0 | "forgets the decade" | 17.28–18.61 |
| 12_6 s1 | "also ends here" | 111.33–112.40 (share .474 at its end) |
| 12_6 s1 | "and so does two and a half" | 112.56–114.55 |
| 12_6 s1 | "any odd number of half laps" | 115.63–117.70 |
| 12_6 s3 | "so any other axis is ruled out" | 134.37–136.68 |

## Discarded measurements

My first pass (`checks/m2.py`) had some mis-aimed boxes. These outputs aren't used anywhere in the review:

- **Contrast:** "faint before-F fill" 3.18 (caught a grid line); "tick labels" and "laps value" 1.00 (empty boxes); "convention line" 4.97 (caught the `precisely` line); "time word" 3.84 (caught purple text).
- **Clearances:** the "signed area word" and "live value" boxes clipped or merged the two items. The "zero line" left edge and "key F" and "key word" were cut by box edges. "Log line" maximum 609 is the panel border. "Inside plate right" was a text pixel.

The re-measured values above come from `checks/m3.py` and `checks/m4.py`. Also ignore `m3.py`'s "mean of end-ring box" lines, which were aimed at empty field, and `m4.py`'s closing "end-ring box" print header, whose numbers are the red-pixel table above.

`checks/m3.py` and `checks/m5.py` expect decoded frames in a `dec/` folder beside them. That folder is not kept here. The 114.400 run has no crop.

## Crop index

Native frame pixels enlarged by NEAREST at the scale given. Hashes are in `crops.sha256`.

| Crop | Source | Box | Scale |
|---|---|---|---|
| `11_f024_before.png` | ch 11 f24, 10.5 | (140, 150, 560, 560) | 2 |
| `11_f033_dashes.png` | ch 11 f33, 15.0 | (140, 150, 560, 560) | 2 |
| `11_f043_red.png` | ch 11 f43, 20.0 | (140, 150, 560, 620) | 2 |
| `11_f069_readout.png` | ch 11 f69, 30.0 | (660, 120, 1000, 560) | 2 |
| `11_f170_arrows.png` | ch 11 f170, 77.5 | (140, 280, 560, 620) | 2 |
| `11_f285_routes.png` | ch 11 f285, 129.0 | (140, 150, 560, 620) | 2 |
| `11_f365_axis.png` | ch 11 f365, 166.0 | (140, 150, 560, 620) | 2 |
| `11_f375_key.png` | ch 11 f375, 171.0 | (880, 380, 1200, 440) | 4 |
| `11_f524_purple.png` | ch 11 f524, 240.5 | (540, 110, 800, 240) | 3 |
| `12_f041_dial.png` | ch 12 f41, 16.0 | (730, 110, 960, 350) | 3 |
| `12_f062_lower_left.png` | ch 12 f62, 26.5 | (40, 500, 640, 640) | 2 |
| `12_f080_flows.png` | ch 12 f80, 32.5 | (720, 410, 1000, 590) | 3 |
| `12_f081_flows.png` | ch 12 f81, 33.0 | (720, 410, 1000, 590) | 3 |
| `12_f287_red.png` | ch 12 f287, 127.0 | (200, 200, 540, 620) | 2 |
| `12_f293_red.png` | ch 12 f293, 130.0 | (200, 200, 540, 620) | 2 |
| `12_decoded_110p700_readouts.png` | decoded 110.700–111.000, reading across then down, two per row | (680, 170, 1100, 220) | 1 |
| `12_decoded_110p800_cube.png` | decoded 110.800, 110.833, 110.867, 110.900, reading across then down | (170, 190, 530, 490) | 1 |
| `12_decoded_126p667_ring.png` | decoded 126.667, 126.767, 126.867, left to right | (190, 250, 390, 450) | 2 |
| `12_decoded_32p400_flows.png` | decoded 32.400–33.033, reading across then down, four per row | (720, 410, 1000, 590) | 1 |
