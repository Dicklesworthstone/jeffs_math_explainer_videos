# Independent final995 picture review, sheets 0784-1044 (CC3, WhiteSwan)

Date: 2026-11-10. Reviewer: CC3 (WhiteSwan), independent of root's own review.

Q = `docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17`

Input: `Q/final-review995/overview/sheet-0784.jpg` to `sheet-1044.jpg` and `Q/final-review995/overview/sampling.json`.

- sampling.json sha256 `c211e7d571a9ddd67f655c9b5070ff568555723b4e30f92891ee224b4b64626f`. I recomputed this at the end of the review and it matches the task.
- `master_sha256` in sampling.json is `80244e88ca5970d0da07a5d96d6d825ae1851bae703d75bd200f8810b04e7b90`. I copied it from the sampler; I did not hash the master file myself.

## Scope and evidence

- **What was done.** I opened every assigned sheet as an image, in bounded batches across four sessions (two account swaps and one tmux loss; each resumed from the persisted progress files). On some sheets I also measured pixels by decoding the sheet JPEG to raw RGB in memory. Those measurements wrote no files.
- **Sheet geometry.** Each sheet is a 2x4 grid of 960x540 tiles, one sampled master frame per tile.
- **Not claimed.** Stills only. No listening, no motion or continuous-playback acceptance, no fine A/V sync, no whole-film AV acceptance.
- **Narration and captions.** The narration (ElevenLabs, Frederick Surrey) was not heard. Caption times come from the delivery SRT text (`Q/release-selection1003/complex-analysis-delivery.en.srt`) and the script, not from audio.
- **Between samples.** Where the sampler gives single frames, I make no claim about the frames in between.
- **Integrity checks.** Every progress record was checked against the sheet files:
  - each sheet's sha256 was recomputed from the copied JPEG and equals the sampling.json value;
  - each record's frame list equals sampling.json's samples for that sheet.

## Coverage (complete)

- **Sheets.** 261 of 261 assigned sheets were viewed: 0784 to 1044 inclusive, with none missing and none duplicated.
- **Frames.** 2087 sampled frames: 260 sheets x 8, plus 7 on sheet 1044. They run from f82264 (09_boundary) to f133661, the last frame of 15_closing and of the film.
- **Chapters.** The tail of 09_boundary, then 10_rigidity, 11_counting, 12_residues, 13_real_integral, 14_continuation and 15_closing.
- **Remaining.** None. The progress files are listed at the end with their full sha256.

## Verdict

- **No High or Medium defect** in sheets 0784-1044.
- **Severity of what was found.**
  - 20 Low items, several already known to root (tags O7/O8, O12, O13, O17, C9, and the x-label lint).
  - 13 cosmetic or observational notes.
- **What was checked and holds.**
  - Every number and statement I checked against independent arithmetic holds (list below).
  - Every quiz answer I checked appears only after its think ring empties.
  - Every chapter seam in the range is clean.
- **Blank or clipped frames.** None was seen, apart from the intended dips and fades listed under seams.

## Low findings (sheet, frames, finding, tag)

| # | Sheets | Frames | Finding | Tag |
|---|---|---|---|---|
| L1 | 0784-0788 | f82264-f82654 | x-axis label `x` sits near the right frame edge; not clipped | root known lint |
| L2 | 0789 | f82742, f82755, f82761 | the growing radius circle from a crosses the `1` label box (transient) | new |
| L3 | 0803-0804 | f85286, f85369, f85429 | the `agree here` box has a translucent backing, so the growing yellow region shows through it; text stays legible | new |
| L4 | 0818 | f88969, f89029 | the red caveat line is the smallest and dimmest text on screen while it carries the key exception; the `precisely` footer touches the plate bottom (no overlap) | new |
| L5 | 0853 | f92850-f92907 | names-to-search footer top meets the domain plate bottom (touching, not overlapping) | root O12 |
| L6 | 0854-0855 | f92986-f93258 | condition boxes `the loop winds once, counterclockwise` and `it passes through no zero and no pole` sit in the w-plane panel under `turns: -1` after the image curve is cleared, so they can read as describing the image curve, not the yellow domain loop | new |
| L7 | 0856 | f93291-f93531 | third condition chip sits across the w-plane real axis; the `0` tick label touches its lower edge | root O17 |
| L8 | 0861-0862 | f94839-f95271 | inset chip (outer ring +1, inner ring -1) overlaps the w-panel top-right border corner and pokes past the panel | root O13 |
| L9 | 0871, 0875 | f97431 vs f98007 | the leash inequality `|z^5+1| <= 2 < 3 = |-3z|` is on screen from f97431 while the drawn gap is squashed, not the literal distance 2; the squash note appears 9.6 s later at f98007 (full f98031) | new (pedagogy) |
| L10 | 0872 | f97477-f97539 | card `one root inside, at distance 0.335` touches the bottom root marker at -0.08-1.33i; marker still readable | new |
| L11 | 0875, 0897 | f98007-f98031; from f100363 | footer top meets the plate bottom (875) or the input axis end (897), touching only | same edge style as O12 |
| L12 | 0893 | f99355-f99579 | a `+0.0` turn readout stands about 3.9 s over an empty output plane before any walker, dog or label appears | new (pedagogy) |
| L13 | 0910-0911 | f102100-f102942 | the 11_8 layout is unchanged from local 184.8 s to the fade at 198.8 s; the SRT has no caption from local 188.26 s to the chapter end, a 10.60 s caption-free tail, the longest in the film. Whether that is silence, music or a deliberate pause needs listening | new (pacing, listening needed) |
| L14 | 0914 | f104175 vs f104647 | `h` names the holomorphic remainder in 12_1 and the denominator of g/h in 12_2; the two never share a frame, but the letter changes meaning within about 25 s | new (notation) |
| L15 | 0934 | f105943, f105945 | the bottom residue row sits about 23 px of 540 above the frame edge (player and caption area) | root O7/O8 |
| L16 | 0969 | f112478, f112646, f112951 | the `h = 1 + z^4` line ends at x 912 of 960 (47 px margin, 1 px past the 48 px action-safe line); not clipped | new (margin) |
| L17 | 0974-0975 | f115688; f115886, f115941, f116262 | the `R = 10` line ends at x 921 of 960 (38 px margin, 10 px past action-safe); not clipped | new (margin) |
| L18 | 0976-0977 | f116360-f117485 | `precisely` card covers the lower half of the plane, including the 225 and 315 degree roots and the lower unit-circle arc, for the rest of chapter 13. Its border is at x 26-933, rows 432-526, outside action-safe on three sides; its text runs x 44-781. Its third line appears between f116999 and f117086, after the last 13_11 caption ends (local 168.65 s), so it is unspoken text held about 7 s in the 8.70 s tail | new (layout) |
| L19 | 0987-0988 | f118576-f118739 | `names to search: analytic continuation` card top border (rows 481-482) sits directly on the colour panel bottom edge (row 480) with no gap; bottom border at row 526 | root C9 (sampler cites after-357 C9) |
| L20 | 1013-1014 | measured f127665; held f127322 to the 14_8 fade | the grey line `all these roots together are dense; drawn: through order 64` ends at x 931 of 960 (28 px, 2.9 percent, 20 px past action-safe); other lines end at x 853-908; not clipped | new (margin) |

## Cosmetic and observational notes

| Sheets | Frames | Note |
|---|---|---|
| 0833 | f90695 | `|sin z| =` readout leaves a wide gap between `=` and `0.77` |
| 0952 | f106483-f106603 | pole labels are set upright (i, -i) beside italic i in the formulas |
| 0958 | f106888-f106952 | 13 opens on empty frames until about 1.5-2 s after the first caption (0.60 s); the bump arrives at f107043 (2.62 s) |
| 0987 | f118127 | the i chip's lower-left corner is about 2 px from the yellow circle (close, not touching) |
| 0995 | f120287, f120562 | the small disk label `1` sits about 5 px inside the rim and 7 px from the vertical axis; tight but clear |
| 0997 | about f121967-f122512 | `precisely` card (uniqueness, continuation along a path) border x 26-933, rows 481-526, text x 44-813, rows 496-512; it is clear of plot content. It is unspoken and held about 9 s across the 6.1 s pause after caption 532. **Correction to my 0997 progress note:** that note says the card is "inside 5 percent action-safe"; it is not. The border is 26 px from the sides and 14 px from the bottom, and the text starts 4 px past the left action-safe line. The text stays inside the vertical action-safe line (28 px from the bottom) |
| 0998 | f122554-f123243 | the title uses centred dots (`1 + 2 + 4 + 8 + ...` with centred ellipsis) while the quiz box uses baseline dots |
| 1002 | f125816 | the label `sqrt after one loop: -1` begins with a bare radical sign with no radicand |
| 1021 | f127756-f127766 | an 11-frame (about 0.18 s) dip to empty background between 14_7 and 14_8, while caption 558 plays |
| 1035 | f129281-f129282 | the last two chapter 14 frames carry a residual within about 5 levels per channel of the background; the seam is otherwise clean |
| 1037 | f129752, f129883 | the pink `i` label dot sits about 2 px from the yellow partial-sum curve; the stem is clear by 8 px |
| 1039 | f130962-f131443 | the five recap box borders span x 43-916, 5 px past the 48 px action-safe line (borders only; all text within x 72-897). The card omits the narrated "there may be none" (summary only) |
| 1040-1043 | f132043-f133365 | the final 15_3/15_4 picture sits in a plot box at x 115-628 of 960; the right 332 px (35 percent) of the frame is empty (no pixel off background at x > 700, measured on f133203). This is a layout choice, not a defect |

## Numbers and statements checked (independent arithmetic or pixel measurement)

**09-10**

| Sheet | Check |
|---|---|
| 0795 | 60-term readouts 0.200000 at 2 and 212 at 2.6, recomputed 0.2 and 212.18; interval (1 - sqrt2, 1 + sqrt2) rounded correctly |
| 0804 | sin^2(3i) = -100.36, cos^2(3i) = 101.36, sum 1.000000 |
| 0816 | zeros of sin(1/z) at 0.318, 0.159, 0.106 = 1/pi, 1/(2pi), 1/(3pi) |
| 0833 | sinh 0.71 = 0.77; sin(3i) = 10.0 i |

**11 counting**

| Sheet | Check |
|---|---|
| 0846-0850 | winding readouts match the drawn image angles; turns 1 and -1 |
| 0858 | radius 0.8 circle measured 0.79; readouts against f = (z - 0.45)/(z - 1.35) at three angles |
| 0865 | `|f| = 30.0` at 0.03 from the pole (0.9/0.03 = 30) |
| 0871 | root markers 0.333, 1.207, -1.379, -0.08 +/- 1.32i (true 0.3347, 1.2146, -1.3888, -0.0802 +/- 1.3284i); image curve pixel fit median 0.57 px |
| 0872 | distance 0.335 against the smallest root 0.3347 |
| 0882 | rotation by 2i: loops at 6i, -4+2i, 4+2i, -2i |
| 0894-0897 | 243 = 3^5; longest leash 10 at z = -3; turns 5 |
| 0903 | counter floor(8 x walked) against pixel angles |
| 0905 | 64 samples land on the eighth roots; L x step 0.785 shown as 0.79 |

**12 residues**

| Sheet | Check |
|---|---|
| 0913 | Laurent term integrals round p: 0, 2 pi i, 0 |
| 0914 | g/h residue form for a simple zero |
| 0933 | f'/f of 1/z = -1/z, residue -1 |
| 0952 | Res at i of 1/(1+z^2) = -0.5 i |
| 0917-0949 | hue order and pole/zero orientation of the domain-colour squares |

**13 real integral**

| Sheet | Check |
|---|---|
| 0959 | integral of 1/(1+x^2) = pi |
| 0961 | 2 pi i x 1/(2i) = pi = 3.14159; display R = 2 |
| 0962 | bump end/peak ratio 0.60 = (R^2-1)/(R^2+1) at R = 2 |
| 0965 | antiderivative derivative = 1/(1+x^4) at 5 points; 2.22144 |
| 0968 | quiz answer only after the ring |
| 0969-0971 | Res_p = 1/(4p^3) = -p/4; sum -i sqrt2/4 |
| 0972-0975 | pi/sqrt2 = 2.2214; arc bounds 0.4189 = 2 pi/15 and 0.0031419 < 1/300; Simpson on [-200, 200] 2.2214414 |

**14 continuation**

| Sheet | Check |
|---|---|
| 0980-0986 | two-pole panel hue decreases counterclockwise round both poles, as for 1/(1+z^2) |
| 0994 | 11 terms at z = 1/2 give 1.999023 (2 - 2^-10); at z = 1.2 give 32.15 (32.1504); 1/(1 - 1.2) = -5 |
| 0996 | re-centred disk at -1/2 has radius 1.5 through the pole (31/93 px against a 62 px unit) |
| 0997 | the walker loop closes at the origin |
| 0998-0999 | quiz answer after the ring empties (ring near empty f123219 at 94.87 s, answer caption 534 at 95.07 s); 2047 = 2^11 - 1; bars halve |
| 1000 | zeta(-1) = -1/12 = -0.083333 |
| 1002 | sqrt z angle is half the z angle within 0.5 degrees at 4 points; after one loop sqrt z = -1 |
| 1003 | log gains 2 pi i per loop; label clearances 8 and 12 px |
| 1013 | 64 ticks at 5.6 degree spacing (360/64 = 5.625); lacunary hue against arg of the 12-term sum at 48 points: median error 6.5 degrees, max 30 near the rim; 0.98^2048 about 1e-18 |
| 1031 | Gamma pole markers at 65.0 px spacing with 0 on the Re z = 0 edge; hue against arg Gamma (Lanczos with reflection) at 32 points: median 8.1 degrees, max 20.5 |

**15 closing**

| Sheet | Check |
|---|---|
| 1037 | poles at +-i 85 px from the axis, equal to the dashed lines at +-1; the circle is round |
| 1041 | quiz `radius at 3?`: think ring empty at f132736 (57.55 s), box fades f132760 (57.95 s); answer caption 578 starts 57.75 s; no answer on screen before then |
| 1042-1043 | sqrt(9 + 1) = 3.16228; diameter ends 3 - 3.162 = -0.16 and 3 + 3.162 = 6.16 |
| 1043 | measured on f133203: circle outer x 180-589, y 79-487 (round); pole rings at x 192 on the imaginary axis; 63.5 px per unit; centreline radius about 202 px = 3.18 units including stroke; the circle sits 14 px inside the plot box top and bottom |

## Seams, fades and every-frame windows

**Chapter seams (all clean)**

| Seam | Sheets | Frames | What the frames show |
|---|---|---|---|
| 09 to 10 | 0800 | f83558-f83571 | 09 fades to black, 10 opens on black |
| 10 to 11 | 0837-0838 | f91003-f91048 | 10 fades to near black, 11 opens on black for about 0.6 s, first trace at f91048 |
| 11 to 12 | 0911-0912 | f102940-f102991 | fade at f102940-f102942, 12 empty at f102943-f102945, window fades in from f102967 |
| 12 to 13 | 0957 | f106880-f106887 | every frame |
| 13 to 14 | 0977-0978 | f117521-f117563 | fade, then empty |
| 14 to 15 | 1035 | f129281-f129285 and f129307-f129343 | every frame; residual of at most 5 levels on f129281-f129282 |
| Film end | 1044 | f133659-f133661 | pure background, (12,17,20). The title card `The Hidden Geometry of Complex Functions` holds f133433-f133607 and dims at f133643. The eighth cell of sheet 1044 is sheet filler, not a frame |

**Every-frame windows**

No flicker, jump or stray blank was seen in any window except the intended dips noted.

| Frames | Sheets | Chapter | Note |
|---|---|---|---|
| f87304-f87367 | 0807-0814 | 10 | |
| f89810-f89897 | 0820-0830 | 10 | |
| f90979-f91010 | 0834-0837 | 10 | |
| f91049-f91096 | 0839-0844 | 11 | |
| f98475-f98504 | 0877-0880 | 11 | |
| f99305-f99354 | 0887-0893 | 11 | |
| f105414-f105457 | 0916-0921 | 12 | |
| f105481-f105520 | 0923-0927 | 12 | |
| f105706-f105743 | 0928-0932 | 12 | |
| f105973-f106092 | 0935-0949 | 12 | includes a three-frame empty dip near f106043-f106045 between squares |
| f106856-f106887 | 0954-0957 | 12 | |
| f117561-f117619 | 0978-0985 | 14 | |
| f118750-f118784 | 0988-0992 | 14 | |
| f126403-f126467 | 1004-1012 | 14 | |
| f127696-f127828 | 1014-1029 | 14 | includes the 11-frame dip f127756-f127766 |
| f129251-f129285 | 1031-1035 | 14 | |

## Caption-free chapter tails (from SRT text, not audio)

| Chapter | Tail (s) |
|---|---|
| 01 | 3.20 |
| 02 | 3.20 |
| 03 | 3.21 |
| 04 | 3.61 |
| 05 | 4.10 |
| 06 | 3.11 |
| 07 | 3.11 |
| 08 | 8.21 |
| 09 | 3.61 |
| 10 | 3.60 |
| 11 | **10.60** |
| 12 | 3.60 |
| 13 | 8.70 |
| 14 | 3.61 |
| 15 | 7.55 |

The first caption of every chapter starts at 0.60 s. Tails 11 and 13 hold a static layout or an unspoken card (L13, L18). Tail 15 holds the final circle and then the title card.

## Progress files (scratch, all preserved, none overwritten)

61 files, written exclusively (mode 'x') by `progress995.py`. Full sha256 of each:

| File | sha256 |
|---|---|
| independent-final995-cc3-progress-0784-0791.json | 676a747688f481032d92a6bb9d872e33e889628224c81b6eb5d25d07c505b05d |
| independent-final995-cc3-progress-0792-0799.json | 134fad2b29180f29e0ad75d2f2d4ee3a71c20a6b55bec24b3008627c73e63f28 |
| independent-final995-cc3-progress-0800-0807.json | 1aecad86ffb460b15aa646f44b3ea69295554b639ed036d1adb392759c90fd8a |
| independent-final995-cc3-progress-0808-0815.json | 7c6186bb8366e95bb2f3ad456c9641139e232bc1bfb7f9a8459a8e507b795590 |
| independent-final995-cc3-progress-0816-0819.json | a5180be1e90674e0f908b1a252c2d76828d607e8911641a3c2645b34ec23aabc |
| independent-final995-cc3-progress-0820-0823.json | de671fb0b5bd0354f18bf78741732671de7d934e2728ba5c1d4f663844c40aa6 |
| independent-final995-cc3-progress-0824-0827.json | 563fbf02e6f17a568c3e9f85e8dcee052cd8ddf1813f5ac573408d8edc25b9c0 |
| independent-final995-cc3-progress-0828-0831.json | e45631ba651338c55fb8e8864263511e14ed3fb24d8f5a4cc0f449c3fc3b5dab |
| independent-final995-cc3-progress-0832-0835.json | b0e399919c1e1cbca51b41cef1de3c2ebf2c266a589b2f6f46054e3c7b600d7e |
| independent-final995-cc3-progress-0836-0839.json | 723c8a106be5d4b9ed59cab9cc82bb4d3535e7e662cc7ffa4fb403f629c3a7b3 |
| independent-final995-cc3-progress-0840-0843.json | 7e5d9682211ee4c32a187dabb1a384c0ca4151883914ab1a999bd46e5561fee2 |
| independent-final995-cc3-progress-0844-0847.json | ca32bac666b95a41d6dd49dd8122cb91ab0e86ffcaa6dadeaa32229e46e978f7 |
| independent-final995-cc3-progress-0848-0851.json | 18bac8c410140c974acd89c26bc0191488784dc881ed3cd9a24fe270de2db3fa |
| independent-final995-cc3-progress-0852-0855.json | c3e5b138b7a29f10fd04aa65fbd9c88b5a1ad6e65e3b90312172cfd96290f54b |
| independent-final995-cc3-progress-0856-0859.json | 698b8647a8e67c7ebb9d2372c53dd348719379923bde6933a83e6e526d4b5b0e |
| independent-final995-cc3-progress-0860-0863.json | 69d9ff2cc53cbf9c9b986c39aac5a7fc9324b14909fe1c652ff909574ba009fb |
| independent-final995-cc3-progress-0864-0867.json | ad89b884f5de2667256a8a67080fa29bfdf49a7cee3bf37d58cb9b1ac18cd5f4 |
| independent-final995-cc3-progress-0868-0871.json | fe0bbd2dad5ae64e742bd2041ebd88403b0d25a9d499a380c4f60f3c9701cc50 |
| independent-final995-cc3-progress-0872-0875.json | 76eb087fc9e963449c4132144ed87431c0efb36fb9063ec8e38acdf57dccc647 |
| independent-final995-cc3-progress-0876-0879.json | e79a31eb2438d2180425a899090492fa11911d44699414101cf97e14b0b432a1 |
| independent-final995-cc3-progress-0880-0883.json | 8fb2949679041db1a5b6c8695e06c10d01e0f0e5640c6b8dca108515129b59b0 |
| independent-final995-cc3-progress-0884-0887.json | 93c62fc3da1ec23d326dfa99ca9c3ec49a86ebb30822a5bce8531247e0ef0467 |
| independent-final995-cc3-progress-0888-0891.json | e92a76748ae23ac8ac7738504bbf6e4996ce44cdfc78b1d22b6b69b3f5fd873c |
| independent-final995-cc3-progress-0892-0895.json | ddb309386eba9a4dc335e14f92d2cb868ceb9b801cb71ded300e03c44065ef77 |
| independent-final995-cc3-progress-0896-0899.json | b4e73996815d60ce4dc56a3b9d54f5e7b91a11ba5dcd7ecbb4be3438b8569ced |
| independent-final995-cc3-progress-0900-0903.json | 7748555a37cdd42bef3bb95dea7e080757121cd399430b72b1dc81a017c0956e |
| independent-final995-cc3-progress-0904-0907.json | 062c249838b3eb21e4949e4c7c53bd81614c9e81889ed2787001cddc9b098ba5 |
| independent-final995-cc3-progress-0908-0911.json | 767feb6ce4a02418e633b1c974541bf3aa8b2040272ae8d73a29dfe11ea0fec9 |
| independent-final995-cc3-progress-0912-0915.json | b8263209c011e897c94af916c191b30642907bfe9dddcff99596a20524838e80 |
| independent-final995-cc3-progress-0916-0919.json | 595cb421759ff8b644738d11240125141374a1f51f979a4b40e054dcf9bfde8c |
| independent-final995-cc3-progress-0920-0923.json | 514cca4eba2355cb9648efb09aaec31b64e66f5033a7d91081b918b952e18fa4 |
| independent-final995-cc3-progress-0924-0927.json | fbc4071be6b38722359265e439e8ff65efa112e13d6cc5b3a46ea0b698fce6cd |
| independent-final995-cc3-progress-0928-0931.json | d84dc12d31afb17ce98d820f4bf66090e198ae78dbb858981cf422579925ed60 |
| independent-final995-cc3-progress-0932-0935.json | ab1c271aa456eabf167ef4db8e4810d6ec94cdf645b802d7412da0649917b4ad |
| independent-final995-cc3-progress-0936-0939.json | 742674852f3f39b367c2e62cdd54786bf646682cd43cf0eda100aa8e7e40b9e2 |
| independent-final995-cc3-progress-0940-0943.json | 92abf136d7cdc02caa160319b72706c31883137f57c4c6fdb699a5219928b120 |
| independent-final995-cc3-progress-0944-0947.json | 402ef2c41e219a081c27f8ad0e7a3f07d14e7691387d81eea3cc3e1ba49c9992 |
| independent-final995-cc3-progress-0948-0951.json | 41dcdd2d69df934134381531406ac09a6901b830d39ce327bf078ff5717bc7c4 |
| independent-final995-cc3-progress-0952-0955.json | febdf83fe0c909caa3e9d7bb6a8cfc1f39c4eff0d22fba36a4c5a8b1cf73bb46 |
| independent-final995-cc3-progress-0956-0959.json | 0b18c2aec16bdf3fd2e2b32b46cf4c4910befaade1198b8a21fc884415d4dc65 |
| independent-final995-cc3-progress-0960-0963.json | ab0d81bb3cdf8fc50307b4d10b0361cd95be20acab921c325c85cf97a22db588 |
| independent-final995-cc3-progress-0964-0967.json | 666833ccf318c117024600b5739280fc856a4933d9f70766409666826f7d8549 |
| independent-final995-cc3-progress-0968-0971.json | 17ccbd04817fd9bc79d85058f93629abddff7b88c93da174c2fd28f16771be13 |
| independent-final995-cc3-progress-0972-0975.json | 8eb7d0280f61297d0c51f21f76bf4386110eeb819c6013e8609def62ade6d94d |
| independent-final995-cc3-progress-0976-0979.json | 76e676399ce82790bdb31337dd6ea4cf2ef3dccc0f46cd45de35f83e97438742 |
| independent-final995-cc3-progress-0980-0983.json | 8139bd70e5c08612f19e19c397c55fc203b4003f967a6249e291b061f1c69f45 |
| independent-final995-cc3-progress-0984-0987.json | 7ba39ca9deeb9b050ba78d267ca16b96fd5eee08f4cc4bfb36cda192a8cc0f09 |
| independent-final995-cc3-progress-0988-0991.json | fcc2fbb15fbefbe87e1c34c8de4e6a77cbf5b2190e913efb53216c77fdb60c0b |
| independent-final995-cc3-progress-0992-0995.json | 5784541ae8b83f57e02480cf229736d4fed597ace002debea1eb34eb700d1a61 |
| independent-final995-cc3-progress-0996-0999.json | c2314ab55a87a8124692fdc6cb0830c412938e4ed165b66db50d4c6ad42b24c0 |
| independent-final995-cc3-progress-1000-1003.json | cb3a3b6c652686e49f9e8c50ce35ce84832ac1428ab560e81035ec37fde97eb2 |
| independent-final995-cc3-progress-1004-1007.json | ae8a44d05d4cf82e4b4a35d8600ba610b8b59f58ad1600aa3b884caac85b5c8f |
| independent-final995-cc3-progress-1008-1011.json | 4ff3058374fa32741a98f2208422ebdc210644165e9cb48291138bd8433616b6 |
| independent-final995-cc3-progress-1012-1015.json | 42000f8a06dd7f0e0de7dde6af81553962b052f6019b53fab32071d045d93ec1 |
| independent-final995-cc3-progress-1016-1019.json | 1bcd0391a55ee2817c591fabe33a2dff1a8d4ec72a9e1c96a2a046ca6828bfa0 |
| independent-final995-cc3-progress-1020-1023.json | 625d1a79f18e18a004b9464a386f086473bd09667684ddf9e0b7440e526cb75c |
| independent-final995-cc3-progress-1024-1027.json | d8e7a283c368172cbe143fd1867a338bdf141152e9fc2244ebe53aedb3b84830 |
| independent-final995-cc3-progress-1028-1031.json | 7a37f3725f214f939fa669af7acb74ea34582becca06d59a1fe12d4f207a624c |
| independent-final995-cc3-progress-1032-1035.json | 4d8bbda59f43f004461ff35ba19cc632677d9181aca4513c25f7f9dd25eb6cc6 |
| independent-final995-cc3-progress-1036-1039.json | 61ede727ce464cce3968e8dbcebfff53fa5fc1cde4821e4554f9f2307197da05 |
| independent-final995-cc3-progress-1040-1044.json | 02f4d6158bd403772a082f0eecbc09c81a6f989a6fd27c8c32be8d31d8aada02 |

## What this review does not establish

- **Audio.** No listening: narration content, pronunciation, cadence, music and silence are unchecked. The 10.60 s tail of chapter 11 (L13) needs a listener.
- **Motion.** No continuous-motion acceptance. Every-frame windows cover only the frames listed. Elsewhere, single sampled frames say nothing about the frames between them.
- **Sync.** No fine A/V sync. Cue timing was compared only against SRT caption times at sampled frames.
- **Delivery.** No whole-film AV acceptance or delivery verification. The master was not decoded or hashed by me, and no production file, build, render, synthesis, Beads or publication step was touched.
