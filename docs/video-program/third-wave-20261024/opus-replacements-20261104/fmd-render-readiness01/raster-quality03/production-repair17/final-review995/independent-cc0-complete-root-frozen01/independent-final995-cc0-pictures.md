# Independent final995 still review, sheets 0001-0261 (CC0)

Written 2026-11-10T21:10Z by CC0 (Agent Mail LavenderPike). Independent of root's review, which root runs separately.

## Scope and limits

- Input: `docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/final-review995/overview/sheet-0001.jpg` to `sheet-0261.jpg`. Each sheet has 8 tiles of 960x540 in a 2x4 grid.
- Sampling: `sampling.json` sha256 c211e7d571a9ddd67f655c9b5070ff568555723b4e30f92891ee224b4b64626f. Every assigned sheet's sha256 matches that sampling (index `final995-cc0/index-0001-0261.json`, `matches_sampling` true for all 261).
- Method: I opened and read every sheet as an image, in bounded batches of 3 to 6 sheets. Where a judgement needed numbers, I used crops (`final995-cc0/crops/`) and small pixel-measurement helpers. Hashes were used only to identify sheets, never as visual evidence.
- Stills only. No audio heard or checked. Nothing here reviews motion, fine sync, or whole-film audio and video, and nothing here accepts the film or a release. Transient findings (L2 and the timing notes) are described only as they appear in still frames.
- Narration: ElevenLabs Frederick Surrey is the current voice. Narration was not checked, and nothing here asks for it to change.
- No production edits, builds, renders, synthesis, SSH, Beads or publication.

## Coverage

| Chapter | Sheets | Chapter time sampled | Master time | Master frames | Tiles |
|---|---|---|---|---|---|
| 01_the_bump | 0001-0029 | 0.000-105.167 s | 0.000-105.167 s | 0-6310 | 229 |
| 02_turning_numbers | 0029-0041 | 0.000-89.517 s | 105.183-194.700 s | 6311-11682 | 99 |
| 03_two_planes | 0042-0057 | 0.000-120.800 s | 194.717-315.517 s | 11683-18931 | 128 |
| 04_reading_color | 0058-0090 | 0.000-112.883 s | 315.533-428.417 s | 18932-25705 | 263 |
| 05_gallery | 0090-0226 | 0.000-164.633 s | 428.433-593.067 s | 25706-35584 | 1083 |
| 06_one_derivative | 0226-0261 | 0.000-166.917 s | 593.083-760.000 s | 35585-45600 | 286 |

- Viewed: 261 of 261 sheets (2088 tiles). Remaining: none.
- Final progress file: `final995-cc0/progress/progress-v082.json`, sha256 3b2c038132e534d9f2d1f9ab4902768c6f3418b7c28b59486fc9326135515ca8. It holds each sheet's sha256, my note for it, and the batch that recorded it. Versions v001 to v081 are kept unchanged.
- Batches: `final995-cc0/batches/batch-001.json` to `batch-083.json`. All are kept unchanged, and none repeats a sheet (progress.py asserts this).
- End of range: sheet 0261 ends at 06 frame 45600 (166.917 s), in the middle of the chapter's final fade. The end of that fade and the 06 to 07 seam fall on sheet 0262 and later, outside this range.

## Verdict

- No blocker. No High or Medium finding.
- One Medium-low finding (M1) and nine Low families (L1-L9), across 33 finding records. Several families recur through the film.
- The mathematics on screen held wherever I checked it against exact values (examples under "Checked and correct").

## Findings, most severe first

Sheet/slot/frame give the first or clearest sampled instance. Frames are master frame numbers, and times are chapter times.

### M1 Medium-low: the 01_6 preview arrow is only a detached tip

- Where: sheet 0023 slot 3, f5591, about 93.2 s to the 01_6 fade-out (sheets 0023-0025). Crop `crops/s0023-slot4-preview3x.png`.
- What: the middle "coming up" preview should be a loop with an arrow turning around zero (`the_bump.rs` previews(): Arrow from 0 to polar(0.8, 0.9)). On screen there is only a white arrow tip about 8 px wide at 960x540. It floats inside the circle at about 0.4 of the radius, with no shaft, and touches neither the circle nor the centre. It does not read as an arrow from zero.
- Likely cause: at franken_manim a399d626, Arrow::new trims buff = MED_SMALL_BUFF (0.25) at both ends (fmn-library `line.rs:494`, `619-626`). This arrow is about 0.5 units long, so the trim leaves no shaft.
- Suggested fix: `buff(0.0)` or a Vector-style arrow. This is a source reading; the fix has not been rendered.

### L1 Low, film-wide: the minus is a short heavy bar

The minus sign renders shorter and heavier than the plus and equals signs beside it. TeX's minus is as wide as the plus. This happens in prose and in TeX titles and readouts alike. It is legible, but where minus and plus sit side by side the mismatch is visible.

| Sheet / slot / frame | Where | Measure |
|---|---|---|
| 0005 / 5 / f1078 | 01 series 1 - x^2 + x^4 - ... (from 17.967 s) | minus about 10 px against plus about 15 px |
| 0077 / 8 / f22892 | 04_4 "precisely:" note, (z - a) | short heavy hyphen |
| 0079 / 8 / f23481 | 04_5 "colour cycles around 0: -1" | differs from the full TeX minus in the "-1.0" counter beside it (0080 slot 4, f23714) |
| 0083 / 8 / f25113 | 04_7 "at i: -1 turn, at -i: -1 turn" | three short minuses |
| 0099 / 6 / f26066 | 05 TeX title f(z) = z^3 - 1 | minus 12 px wide and 2-3 px thick; "=" bars 16 px and 1 px |
| 0129 / 4 / f28933 | 05_4 "-1 <= sin x <= 1" | short heavy hyphen |
| 0137 / 1 / f30188 | 05_5 tan(0.4 + 12i) over tan(0.4 - 12i) | plus 11-12 px, minus 8 px wide and 2 px thick |
| 0173 / 7 / f32836 | 05_7 "zeros at -2, -4, -6, ..." | minus 8 px, digits 7 px |
| 0216 / 1 / f35075 | 05_9 "one tile of z^3 - 1" | differs from the thumbnail label above it |
| 0229 / 1 / f36869 | 06 quotient (f(z_0 + h) - f(z_0))/h, then "v_y - i u_y" and "v_x = -u_y" | minus 12-14 px against plus 16 px; in "v_x = -u_y", 14 px and 3 rows against "=" 18 px and 1-2 rows (0237 slot 5) |
| 0240 / 6 / f43664 | 06_8 "x + i y -> x - i y"; 06_7 matrix entry -2 | shorter than the plus in the same line |
| 0242 / 3 / f44628 | 06_9 "up: -1" | 9 px wide and 2 rows thick against a 12-13 px digit height; a TeX minus would be about 15 px |

Suggestion: find out which glyph the text and TeX paths map "-" to at the pinned engine (the text hyphen, or a minus drawn too short). Fix it in one place rather than per scene.

### L2 Low, transient: the 01_3 degree counter leads the curve

- Where: sheet 0008 slot 3, f1848 (30.8 s), and slot 6, f1884 (31.4 s).
- What: the "terms up to degree" counter switches at the start of each curve morph. Mid-morph stills show 8 with the degree-6 shape, then 10 with the degree-8 shape. Each step settles within a second. Whether this reads as wrong in motion needs a motion review.

### L3 Low: a gap after one-digit angles in "turned: N deg"

- Where: sheet 0033 slot 1, f7571 (02_2). Also f7582 and f7743. Crop `crops/s0033-degree-readouts4x.png`.
- What: the degree sign is fixed while the number is centred. One-digit angles show a visible space before the sign; two-digit angles sit tight.

### L4 Low, film-wide: variables in prose captions are upright

Variables in prose captions are set upright, while the same symbols in formulas are italic. It is readable, but the same symbol looks different in two places.

| Sheet / slot / frame | Text |
|---|---|
| 0035 / 5 / f8711 | 02_4 "multiplying by w: stretch by \|w\|, turn by the angle of w"; f9040 "modulus \|w\| ... argument arg w"; 02_2 "every point times i" (f7571) |
| 0037 / 7 / f9911 | 02_5 quiz "(1 + i)^2 = ?" in the text face, against its TeX answer "(1 + i)^2 = 2i" at 74.5 s (f10839) |
| 0077 / 4 / f22790 | 04_4 "precisely:" note: z, a, m, c, f(z) upright, and the power in words ("to the power m") |
| 0083 / 8 / f25113 | 04_7 "at i", "at -i" |
| 0129 / 4 / f28933 | 05_4 "sin x" |
| 0169 / 4 / f32340 | 05_7 "series converges: Re s > 1", with the title on the same frame setting s italic |
| 0181 / 3 / f33070 | 05_7 "more zeros on Re s = 1/2", "Im s = 32" |
| 0216 / 1 / f35075 | 05_9 "as at z = 0", "one tile of z^3 - 1" |
| 0229 / 6 / f37108 | 06_2 ratio readouts "2.43 + 2.00 i", "-> 2.000 + 2.000 i" |
| 0235 / 5 / f40445 | 06_5 "u_x is how fast u changes as x changes" |
| 0255 / 6 / f45315 | 06_9 "image drawn moved down by i" |

### L5 Low: the 03_2 turn counters are rounded separately

- Where: sheet 0045 slot 7, f13243 (26.0 s), "turns: 0.28" beside "turns: 0.57". Also 0046: 0.85 with 1.71 (f13483), and 1.00 with 1.99 (f13543). They settle at 1.00 and 2.00 by f13570.
- What: the picture's point is that the output turns exactly twice as fast. The dot positions agree with an exact input of about 0.285.
- This is the F1 pattern from the earlier final03 review, still present.
- Suggested fix: compute the output readout from the displayed input value.

### L6 Low: the 05_7 critical-strip inset uses unequal scales

- Where: sheet 0181 slot 5, f33122; measured on 0183 slot 1.
- What: the inset is 44.3 px per unit across and 12.5 px per unit up (3.5:1). Its level-set rings come out as flat ovals, while the rings around -2, -4 and -6 are round. Only "gold box = gold band" is on screen; nothing says the strip is squashed.

### L7 Low: the 05_9 recap grid squeezes the second-row labels

- Where: sheet 0206 slot 4, f34622 (148.6 s to at least 152.7 s).
- What: "sin z", "Gamma(z)" and "zeta(s)" sit 4 px below the picture above and 5 px above their own. First-row labels have 13 px. A glance can read "sin z" as the caption of the z^3 - 1 picture above it.
- Note: sheet field is the integer 206 in the JSON record (see "Data notes").

### L8 Low: tight clearances next to lines

| Sheet / slot / frame | Text |
|---|---|
| 0228 / 2 / f36545 | 06_1 green "change in the output": 2-3 px from the panel border (about 8-12 px at 4K); at 16.0 s the shrinking arrowhead passes about 3 px from the "c" |
| 0232 / 5 / f38499 | 06_3/06_4 teal "a stretch and a turn": 4-5 px right of the output panel's Im axis |
| 0227-0228 (note only) | "z_0 = 1 + i": about 4 px right of its Im axis |

Nothing touches on the sampled frames.

### L9 Low: the live 06_2 ratio readout is spaced unevenly

- Where: sheet 0229 slot 6, f37108 (25.4-31.5 s).
- What: "2.43 + 2.00 i", "2.00 + 2.45 i" and "1.64 + 2.18 i" have gaps about a digit wide around the plus and before the i. The settled "-> 2.000 + 2.000 i" (34.1 s) is spaced normally. Same family as L5.

## Notes, not findings

- **06_9 x + 2iy stretch (sheets 0243-0255).** The green rectangle grows about the cyan square's centre, not about the real axis. This is honest: when it settles (161.85 s) the note "image drawn moved down by i" appears. I measured the square's centre at 1 + i at 114 px per unit, and the drawn rectangle at 1.99x the height. The true image is centred at 1 + 2i, so the drawn one is exactly that image moved down by i. While the rectangle is still growing (160.7-161.85 s) the note is not yet there.
- **"no complex derivative" carried over (0242, 159.033 s).** The label stays from conjugation into the x + 2iy title before that map is demonstrated. It is true for both maps.
- **06_8 think-time ring (0241).** A grey ring with a purple dot appears under "a derivative anywhere?" at 146.0 s and is empty at 146.1 s. It was not on screen at 141.0 s, so its run between 141.0 and 146.0 s is not sampled. The answer begins after it (148.0 s).
- **06_7 matrix [[2, -2], [2, 2]] = x(2 + 2i)** is correct for z^2 at z_0 = 1 + i. The point is not restated in that scene.
- **Edge margins.** The 06_2 readout and ratio panel end about 25 px and 20 px from the right edge at 960 px. The 06_6 "names to search" box ends about 14 px from the bottom edge. Both are inside the frame but outside a 5 percent title-safe margin.

## Checked and correct (examples)

- **05_9 recap:**
  - All six thumbnails carry correct markers: Gamma's poles at 0 to -4; zeta's trivial zeros at -2, -4, -6 and its pole at 1.
  - Batch 065 corrects batch 064's names for the fifth and sixth thumbnails (see "Corrections").
  - The zoom of a zero-free square is true: z = 0 is the centroid and the critical point, and the traced tile of z^3 - 1 sits on the contour steps.
  - The 05 to 06 seam is a clean black seam with an even start (0226).
- **06_1 to 06_4:**
  - The output arrow equals 2 z_0 h + h^2 at the same scale, within about 4 px.
  - The ratio readouts equal (2 + 2i) + h for the measured h.
  - The settled fan shows stretch 2.87 and turn 46 degrees at 51.0 s, against |2 + 2i| = 2.83 and 45 degrees.
  - The image square's corners match exact z^2.
  - The "same scale" note is removed exactly when the zoom starts.
  - "largest gap / image side: 0.088 -> 0.0088" is true.
- **06_5 to 06_8:**
  - The Cauchy-Riemann steps and both boxed equations are correct.
  - The colour pairing (teal u_x and v_y, gold v_x and u_y) is consistent.
  - The conjugation square is mirrored with reversed orientation.
  - "right: 1, up: -1" and "right: 1, up: 2" are true, and the red X appears only after both ratios are shown.
- **Holds and fades (0256-0261).** Every-frame holds are still: 0 pixels changed by more than 24 levels between same-position tiles across sheets. The final fade dims every element together with no pop.

## Corrections to my own earlier batches

The earlier batches are kept unchanged. Each correction is in a later batch.

- **batch-064 (sheets 0202-0204), corrected in batch-065.** I named the fifth and sixth recap thumbnails "tan z" and "Gamma". They are Gamma(z) and zeta(s); tan z is not in the recap.
- **batch-072 (sheet 0227), corrected in batch-075.** I called the quotient's numerator minus "a proper math minus". Measured, it is the L1 short minus: 12-14 px against plus 16 px.
- **batch-075 (sheet 0235), corrected in batch-076.** I wrote "u terms teal, v terms gold". The colours actually pair the Cauchy-Riemann partners: teal u_x and v_y, gold v_x and u_y.

## Timestamp caveat

- Batches 001-022 carry hand-typed `viewed_utc` values that run ahead of the files' modification times. The gap grows from about 9 minutes (batch-001) to about 2.5 hours (batch-022).
- Batches 025 and 026 run about 1 minute ahead.
- From batch-023 on, `viewed_utc` comes from `date -u` run just before each write, and agrees with the file time to the minute (or is up to a minute earlier).
- The file times show when each batch file was written, which was after its sheets were read. The table below lists both.

## Data notes

- Finding records use the sheet field inconsistently: integers for sheets 5, 8, 23, 33, 35, 37, 45, 46 and 206, and four-digit strings for the rest.
- Ids repeat on purpose, one record per instance (L1 12 records, L4 11, L5 2, L8 2, the others 1 each).
- batch-023 has an extra `viewed_note` key, and batch-069 a `note` key. Both are informational.

## Appendix A: batch files

| File | sha256 | viewed_utc (as written) | file modified (UTC, os.stat) | Sheets |
|---|---|---|---|---|
| batch-001.json | fc7d098d3d35e698905d8bca5b3e1f2a12af96a9f25be6ba24aaeb2bca1148a2 | 2026-11-10T20:05Z | 2026-11-10T19:56:19Z | 0001-0006 (6) |
| batch-002.json | a7dd2c2c158586ccdb66a83b9b1ef317597dc299b60a12d9b38b9d567f48a87f | 2026-11-10T20:12Z | 2026-11-10T19:57:17Z | 0007-0012 (6) |
| batch-003.json | f00ad0c3c5fd2d7d72b84a53c72755c8680216f25b00bdf8d707e6fe519d7304 | 2026-11-10T20:18Z | 2026-11-10T19:57:51Z | 0013-0018 (6) |
| batch-004.json | 0bcb775ceef58cc158263350911f348c3b7bab1575db92b3f54d7da313390bb7 | 2026-11-10T20:24Z | 2026-11-10T19:59:30Z | 0019-0021 (3) |
| batch-005.json | 593efa386837fd995fd5b1e07d2cf108bcce3d6579f9c6eda40fe60c668cb67a | 2026-11-10T20:31Z | 2026-11-10T20:00:42Z | 0022-0024 (3) |
| batch-006.json | 755b0a4ee202a10d45c466b857dc0a39231cc4ab306085dba2fdea8a0e86ee03 | 2026-11-10T20:37Z | 2026-11-10T20:01:12Z | 0025-0030 (6) |
| batch-007.json | 603dad1d9fae42a49f297990aeba8e0142fc427e33424c81f01cbdbdabcd44da | 2026-11-10T20:46Z | 2026-11-10T20:02:06Z | 0031-0033 (3) |
| batch-008.json | c47f9f90312d189c810a577bfa608ea8de7e3cb37eb3b7ac38bf6495950858fa | 2026-11-10T20:53Z | 2026-11-10T20:02:41Z | 0034-0036 (3) |
| batch-009.json | c4050ca2dbede5aa88a9ae7e7ee6a4b7728ed14e3d770f0ccf3842c4662bf227 | 2026-11-10T21:00Z | 2026-11-10T20:03:16Z | 0037-0039 (3) |
| batch-010.json | 781e6bc62845ad11134012782228c78b9d1f9d5c49ed50df66722d9af68eb7f1 | 2026-11-10T21:08Z | 2026-11-10T20:04:01Z | 0040-0042 (3) |
| batch-011.json | 55a23573e57db0d57b5749a85a17cf4ba0e966083afe9425bfb76066b9f2b2c5 | 2026-11-10T21:16Z | 2026-11-10T20:04:39Z | 0043-0045 (3) |
| batch-012.json | 2f900530656538660f2cdd1aa47c0305208256d2f2cc313a0216569d81e81dd6 | 2026-11-10T21:24Z | 2026-11-10T20:05:16Z | 0046-0048 (3) |
| batch-013.json | 566b784ae9c4fd0917a6cc41e968e8a67b5f4b4f1c850590bf3c42be5ec15d81 | 2026-11-10T21:31Z | 2026-11-10T20:05:45Z | 0049-0051 (3) |
| batch-014.json | d4f79a8a5809d1221bafaeba3fd697c6b74d44d58e3b81a7d6d3d5d360bdb93d | 2026-11-10T21:39Z | 2026-11-10T20:06:22Z | 0052-0054 (3) |
| batch-015.json | 1b856b3e90ac31d8a14f875f3e1ac01aa98232b8f99d9c7de7b5682c3ad33084 | 2026-11-10T21:46Z | 2026-11-10T20:06:51Z | 0055-0057 (3) |
| batch-016.json | b3d095ddb63084b90cb24124d884c50f773f859a5c77e4f3beae57cb7899d218 | 2026-11-10T21:53Z | 2026-11-10T20:07:26Z | 0058-0060 (3) |
| batch-017.json | 51a48c5015ed7cc25a017d332ab606a576dde14458245bccccedacc66d3b7c40 | 2026-11-10T22:00Z | 2026-11-10T20:08:05Z | 0061-0063 (3) |
| batch-018.json | 3c08f91ed3ea2870fd12f0e87c50b1d11f51eb5e5ce63aed08d87468c6267b1a | 2026-11-10T22:06Z | 2026-11-10T20:08:19Z | 0064-0066 (3) |
| batch-019.json | 1a0d7c4710c39f3d43bd452b0e7839bd031dea8d7577c070716e14f5cb76df2d | 2026-11-10T22:14Z | 2026-11-10T20:09:40Z | 0067-0069 (3) |
| batch-020.json | a8946c84d742fc21cd74c0f8819cfa5bde59ce71ee2818f5e9e20f5c75636bc2 | 2026-11-10T22:24Z | 2026-11-10T20:10:43Z | 0070-0072 (3) |
| batch-021.json | 271a6f8162ee829e2dfd577cf0665cc7fd35a80e7119f96950c6440207cd297b | 2026-11-10T22:33Z | 2026-11-10T20:11:17Z | 0073-0075 (3) |
| batch-022.json | 8132e66618bbfcf246072b2a851f645800b9f0d7d7f210caf73ea7da96ac264d | 2026-11-10T22:42Z | 2026-11-10T20:11:56Z | 0076-0078 (3) |
| batch-023.json | 00a0a5408e37af7ffcf6c802274ca32ee8f81bca12eb0fcd337d65f098a11994 | 2026-11-10T20:14Z | 2026-11-10T20:13:42Z | 0079-0081 (3) |
| batch-024.json | b83213fef1ca38fb1ec3f3d5fea9e233a347599327d9441e12c927770ad1b394 | 2026-11-10T20:14Z | 2026-11-10T20:14:25Z | 0082-0084 (3) |
| batch-025.json | 75c1c8911674865e8834956b97b7ec4bd4057418a119cf586c4ccb3bf898a212 | 2026-11-10T20:15Z | 2026-11-10T20:14:46Z | 0085-0087 (3) |
| batch-026.json | 7628660328aacb218de75501ae5f4d429b48a0fe7b08ab6f3a60c29847f846d4 | 2026-11-10T20:16Z | 2026-11-10T20:15:14Z | 0088-0090 (3) |
| batch-027.json | 7f99b2bb80556b6c762f8836738f9eba70d0a251402f663373e738fdd0d95272 | 2026-11-10T20:15Z | 2026-11-10T20:15:40Z | 0091-0093 (3) |
| batch-028.json | 9446702747df185a6955f13eb8923ee3e015395fec24089d08166f5caec52584 | 2026-11-10T20:15Z | 2026-11-10T20:15:57Z | 0094-0096 (3) |
| batch-029.json | 4fa3a38192d76da2105f9631625f21e74eb13a2f47855b0036ada20a7eb7c879 | 2026-11-10T20:16Z | 2026-11-10T20:16:48Z | 0097-0099 (3) |
| batch-030.json | 8da34d6a25a5e96bcd87b7fe72693a07b34edf784908a74e2f7017fc2fb04335 | 2026-11-10T20:17Z | 2026-11-10T20:17:25Z | 0100-0102 (3) |
| batch-031.json | 1fbd6732dd4c5daaea2a395c05f40109025735eed5854f92b261d1e7c614bf94 | 2026-11-10T20:17Z | 2026-11-10T20:18:04Z | 0103-0105 (3) |
| batch-032.json | 95b876ebd9963092708f5da91d6c0ec0443af9467540762c2f45462f63f96df6 | 2026-11-10T20:18Z | 2026-11-10T20:18:23Z | 0106-0108 (3) |
| batch-033.json | c8f3c00290cebeebcd79aacd1111d4addb6ed7934de4fc84795e5e5760090558 | 2026-11-10T20:18Z | 2026-11-10T20:18:45Z | 0109-0111 (3) |
| batch-034.json | 9cac04516cc97198b3379f66a3e6ecc047b0f20d077823fb827a2e7218214aa1 | 2026-11-10T20:20Z | 2026-11-10T20:20:09Z | 0112-0114 (3) |
| batch-035.json | da8b68998cf605e5448d3c92ca6aeda283dad841aa0db69edd783f2d79349c30 | 2026-11-10T20:21Z | 2026-11-10T20:22:06Z | 0115-0117 (3) |
| batch-036.json | b1d6cf97a3eed8ef13614dd4421d75dd9f37e87159dc6dd7abc2a4b30a784aa6 | 2026-11-10T20:22Z | 2026-11-10T20:22:25Z | 0118-0120 (3) |
| batch-037.json | b04ef82c756e691469f631a4f2ee83eb4f65c7004280e5cef2622adcb12f6886 | 2026-11-10T20:22Z | 2026-11-10T20:23:05Z | 0121-0123 (3) |
| batch-038.json | d566ea07e48c0885a38bedd998c68edd26dd30f3c2b36d48a13a4e701870a0ea | 2026-11-10T20:23Z | 2026-11-10T20:23:38Z | 0124-0126 (3) |
| batch-039.json | 44ea5d1d08ed2169c5e3138a0f9ce5842079d87ecbbe504f1cb51f38adaa941f | 2026-11-10T20:24Z | 2026-11-10T20:24:17Z | 0127-0129 (3) |
| batch-040.json | 2741e0ace9437e2b66a5b45ee0fa4e0ca276af0c0dfeeae2df2d1cd06c033e57 | 2026-11-10T20:24Z | 2026-11-10T20:24:54Z | 0130-0132 (3) |
| batch-041.json | 5a9451536ed3763203f263c18573e74d42fc7ad99c9de924393fac4567c658e4 | 2026-11-10T20:25Z | 2026-11-10T20:25:26Z | 0133-0135 (3) |
| batch-042.json | 85691e2d8745f0e5a9959e4e1a1f56f2b338322d4867e81f9d312500634b3c17 | 2026-11-10T20:26Z | 2026-11-10T20:26:16Z | 0136-0138 (3) |
| batch-043.json | 94c24b7215d06e83457065054e2b1dd330637635ab42a6d4b631f569c10b8d31 | 2026-11-10T20:26Z | 2026-11-10T20:26:33Z | 0139-0141 (3) |
| batch-044.json | 9c6257cd04f338b1c7fc18f967017cedfdce529ce1c214c16920d01045b44507 | 2026-11-10T20:26Z | 2026-11-10T20:27:05Z | 0142-0144 (3) |
| batch-045.json | 31dcb2cefd82bdab87a0c9e749850461d75215ee97ef3167c18b6fd2c964cdd2 | 2026-11-10T20:27Z | 2026-11-10T20:27:22Z | 0145-0147 (3) |
| batch-046.json | 0f64cbf6b2e5d0dc3ef746a8051c872cd493aedb09b77304baf33600e89bf4f8 | 2026-11-10T20:27Z | 2026-11-10T20:28:04Z | 0148-0150 (3) |
| batch-047.json | 7b1d88c5c68a0cff1caa9c3325e490603c25122aa227cd2ae76785bd952c6212 | 2026-11-10T20:28Z | 2026-11-10T20:28:26Z | 0151-0153 (3) |
| batch-048.json | 8b9dc802f5079c91e801bf990e48948297d0f98598902081dee28de3fed743c2 | 2026-11-10T20:28Z | 2026-11-10T20:28:48Z | 0154-0156 (3) |
| batch-049.json | c74d972759d7879af7a859cc6bae126b8465995f5888bc9f249fc77c2fd1d957 | 2026-11-10T20:29Z | 2026-11-10T20:29:06Z | 0157-0159 (3) |
| batch-050.json | dbca6fff6298675e7e88413a19f68e7338be2a99903318565609ec6eb9db6631 | 2026-11-10T20:30Z | 2026-11-10T20:30:50Z | 0160-0162 (3) |
| batch-051.json | 203497f626607bf311aa678bc148fd83005f5baea360ad25df52859bfe4f43ec | 2026-11-10T20:31Z | 2026-11-10T20:31:09Z | 0163-0165 (3) |
| batch-052.json | 49e8d8f0d44740a88a8536f56ed17cf1d64f370b64af5ef36046ca3660193d9c | 2026-11-10T20:32Z | 2026-11-10T20:32:27Z | 0166-0168 (3) |
| batch-053.json | 19b280a3030a45e6f6327c57bb9a0e68e8be135ec6424ab80409d9ad61c078f1 | 2026-11-10T20:33Z | 2026-11-10T20:33:12Z | 0169-0171 (3) |
| batch-054.json | 9f129c71e7aeb0931797ee01da76d18cbe9d6375618d4a7e2977e01c89b16147 | 2026-11-10T20:34Z | 2026-11-10T20:34:22Z | 0172-0174 (3) |
| batch-055.json | 96f965be62805c65f1bb4a014cec81bc291ee2a22b355f1ea93ce3e531e39994 | 2026-11-10T20:34Z | 2026-11-10T20:34:49Z | 0175-0177 (3) |
| batch-056.json | da67e26cc4c9958c4bfe592616781588f3152e87ed917ea3684313d64bb94cb4 | 2026-11-10T20:35Z | 2026-11-10T20:35:48Z | 0178-0180 (3) |
| batch-057.json | ba5012901b3b6f0d343e4607974ed96efb025e61968fa90649db119531a8dc1f | 2026-11-10T20:37Z | 2026-11-10T20:37:27Z | 0181-0183 (3) |
| batch-058.json | 47a57475059abcd7c0f912903aa0b24e0cbc317e6e932bb26060c9c969aecf2f | 2026-11-10T20:42Z | 2026-11-10T20:42:27Z | 0184-0186 (3) |
| batch-059.json | 561fed1d3ce1d29603b8fed0171438bb386da1e004dedecc46b81c287f35c5fb | 2026-11-10T20:42Z | 2026-11-10T20:43:05Z | 0187-0189 (3) |
| batch-060.json | e135eb3543b09f556dbdc079a0b0879e6ddb552d37740ef4b01e2991b2856c0a | 2026-11-10T20:43Z | 2026-11-10T20:43:43Z | 0190-0192 (3) |
| batch-061.json | ab683616f66f8a9748007f3176c10b9d4e3f211769d755b2d47d164d6e7054ba | 2026-11-10T20:44Z | 2026-11-10T20:44:14Z | 0193-0195 (3) |
| batch-062.json | e5aa140a130f811770a70dd7d2dad7dffc213e5ff803181bdec702bbc1df36cc | 2026-11-10T20:44Z | 2026-11-10T20:44:38Z | 0196-0198 (3) |
| batch-063.json | 66bae07b753e6f395574bfb24d5d9f4249cd74b7a80464044266f20932e592e4 | 2026-11-10T20:44Z | 2026-11-10T20:45:02Z | 0199-0201 (3) |
| batch-064.json | c1221031565646a7acb585ee492cd9cc810c1b72ee988289192498bee0166224 | 2026-11-10T20:46Z | 2026-11-10T20:46:42Z | 0202-0204 (3) |
| batch-065.json | d6a51fee8d0ff431398da281890ca3746cdb65d3fcd0cdc22aa218c763201f53 | 2026-11-10T20:48Z | 2026-11-10T20:48:48Z | 0205-0207 (3) |
| batch-066.json | 06559ff5c0a994fd43df2f254ce46f81d2ae955cc3a21627511891ddb03da391 | 2026-11-10T20:49Z | 2026-11-10T20:49:14Z | 0208-0210 (3) |
| batch-067.json | afdaba8a361fa93113aa84d329994a24dda3eb4d9098dee2e35d1e1363127353 | 2026-11-10T20:49Z | 2026-11-10T20:50:05Z | 0211-0213 (3) |
| batch-068.json | 88e45f2eb7f44656653eadbffa0ce4c4f29bfe67e52e66640949ed39186edaa3 | 2026-11-10T20:51Z | 2026-11-10T20:51:26Z | 0214-0216 (3) |
| batch-069.json | 21783d31e9aacf62d3839a653f03924d6a02394cf9aff3efe43929dde09f54b6 | 2026-11-10T20:52Z | 2026-11-10T20:52:35Z | 0217-0219 (3) |
| batch-070.json | a7d76ff44aba43ba4e1a215d68f2e3181cd9daa063ad07716fe687a06d0ee4e5 | 2026-11-10T20:53Z | 2026-11-10T20:53:13Z | 0220-0222 (3) |
| batch-071.json | e20b32ceaae53fbacfd8c5050a543068c91db3752380757278a0c0bd7beec55d | 2026-11-10T20:53Z | 2026-11-10T20:53:35Z | 0223-0225 (3) |
| batch-072.json | cb3d58795d62d4908c6d09d849bc18bc5f0e3ce99208c0df6da1a5cd7bbad8a8 | 2026-11-10T20:55Z | 2026-11-10T20:56:06Z | 0226-0228 (3) |
| batch-073.json | 97c384316ceef448d59d4261ccdc7f3061831bf33082376d7431960f8736f3a5 | 2026-11-10T20:57Z | 2026-11-10T20:57:23Z | 0229-0231 (3) |
| batch-074.json | 251c28ca8331e8fc7a48c76d7dfa62234324de9843b55cad6ca71ce0019e9c18 | 2026-11-10T20:59Z | 2026-11-10T20:59:46Z | 0232-0234 (3) |
| batch-075.json | 1995cf2369e8ec851a355f67efc281ce9d593e19e85dbde5501ac1e360a1fb6f | 2026-11-10T21:00Z | 2026-11-10T21:01:06Z | 0235-0237 (3) |
| batch-076.json | ad73075d1a4e876ac553447ee42733e883dffda11ab8e9c9cf7b8adafb4a47a9 | 2026-11-10T21:01Z | 2026-11-10T21:01:57Z | 0238-0240 (3) |
| batch-077.json | d3964c903e7489a1b9c93d89dc5be0fafdb28dbeb1bdaba3cac51e554be6a3bd | 2026-11-10T21:04Z | 2026-11-10T21:04:59Z | 0241-0243 (3) |
| batch-078.json | 313909c1e70b2afd9c5de7e30f7bcd534304d48dea1bcbe43e58734e114247dd | 2026-11-10T21:05Z | 2026-11-10T21:06:20Z | 0244-0246 (3) |
| batch-079.json | 935e5be46833a4eb7dacf88fb7e1da317c8498a6e621c2fd0ad3814f39c4f4f2 | 2026-11-10T21:06Z | 2026-11-10T21:06:42Z | 0247-0249 (3) |
| batch-080.json | cc15a7622154c85ab3a8fc69f5e448a23e5a4e817cb446420a44440397557080 | 2026-11-10T21:06Z | 2026-11-10T21:07:09Z | 0250-0252 (3) |
| batch-081.json | a41729537d237872cd48ab987417ad30d63dc41fc2f2c042d8b92c5dfeb9cab9 | 2026-11-10T21:07Z | 2026-11-10T21:07:52Z | 0253-0255 (3) |
| batch-082.json | 03912decd36f7514ccb64b758dcccfa6b92db2b30d2dbc55bbb0cd4b5382e10e | 2026-11-10T21:08Z | 2026-11-10T21:08:43Z | 0256-0258 (3) |
| batch-083.json | bff3b7e892657dc430695cb14c2a99eabffe29d472d86e459efb71f5e154a21e | 2026-11-10T21:09Z | 2026-11-10T21:09:27Z | 0259-0261 (3) |

Final progress: progress-v082.json sha256 3b2c038132e534d9f2d1f9ab4902768c6f3418b7c28b59486fc9326135515ca8 (verified when this report was made).

## Appendix B: viewed sheets

| Sheet | sha256 | Batch |
|---|---|---|
| 0001 | ee2f5602ee8eaf20bfc41be6c2f792402028d66c8cad1b713b342ff354b6e33b | 001 |
| 0002 | 0297ce6adca4b8280adba96d86518af356531adb2ef6b2569b241c3fc96d5b0a | 001 |
| 0003 | 7fdc34451602b2ebadf6ae1dffa8717aecba61d911d3df2ee56733683322bdc0 | 001 |
| 0004 | 2414a1cbbfa0019631cb66323f1ecf223f32bd88ddf770566164a945f218fec8 | 001 |
| 0005 | 9b0bd03168e2c798b109c4d5ab6cc062b49909be13d87e13c9ffed46436f2c9d | 001 |
| 0006 | d1f2225b62b7248509a5b58cd18cb28ee6230210f2e002217a26f7db63313785 | 001 |
| 0007 | b086b3d2ac704343388614e259583e56d90a209eff62271cbbccaa447bf6d768 | 002 |
| 0008 | a9ce3c7b4c40ba5c3e32fc1f486a43e1bfee1df8b8d250e771c16f3215178743 | 002 |
| 0009 | 37d6e759a3a9505a5ca5d73286ee320adb0fe8f859cded45ad3a9a97f81824b7 | 002 |
| 0010 | 056372ae6ffa8f1de097bff47b9cb464f3e19483e6e4165e85a0d6164526f59c | 002 |
| 0011 | 243998b0388ead85ccd677803221a95a6e61ff229906abb694059359af8ad68c | 002 |
| 0012 | 11a140626fa9796288b81c708ab102f109325f6ecba26ab986aaa334fdf54d55 | 002 |
| 0013 | b94a44b872feceebbfdd781d91af480a0b00bf2672a800e54ecd6cfdda89e0d2 | 003 |
| 0014 | 1c447037733f176122e40ed1d93bef2adeac53fe94b1fba74f1ae065951aed56 | 003 |
| 0015 | 7c9d824d8535f3bc22da7223913d2ee91130516a65de1cd974238d9b69abfef1 | 003 |
| 0016 | afd51a5560cad27af313b329de94cf2712fda90c394e572779ec4796f155dc75 | 003 |
| 0017 | ef37d0292b5c966c3f45042a49a6b559dee2b27bcbbc061816143a0afa925b69 | 003 |
| 0018 | d261dced379b167d9d5dca57ade8fc4d43b12a4985642f3e9a3a45a696d15d2d | 003 |
| 0019 | df3732905e68f0ffc63c91ca08a530bb3f8fbafa8f3cd4af4cfcb379f1e35a9e | 004 |
| 0020 | 4ec7eb8221b749c1735224a3f87f600b4794f701a57ddc9aabb2d9501a4c0da7 | 004 |
| 0021 | eaceb3109e5fe87c99b22c01c697203291f78bdd19506671bb2a4d9acd0554d7 | 004 |
| 0022 | a06d29e0e461ac7c30cef517ec3f1aec888f355a6ce0c10c918790e38fd52096 | 005 |
| 0023 | e27219e66525dcbfed699d2811d453d79e57ecd0e6cc048c392e15bb4b7c8558 | 005 |
| 0024 | e03a33b3927f0b1c884962a41549c39caa1aec3e9b16b1b62f6dd21f25324f1a | 005 |
| 0025 | cbd2dd67bcbf5876ec44f54ea50b761074a580ded75c91e33d3d4ddaa934e433 | 006 |
| 0026 | 4507d16f87482e861256296392d26f3e0e33dc4f636b68d0b63298af535d6e47 | 006 |
| 0027 | f3d7ba7089490a44d7e302fe3cfbdcc1ebedf926a8b9c7f3fb3567a688062665 | 006 |
| 0028 | 1e7417d63f9e92c62d98e5688f53b6571fcff51c9bdf028121e1d2b90a9bfb39 | 006 |
| 0029 | 78c6cd78cdddbd6948c93025b7eda32fc7409e6b78a3b00b211e7229040feb98 | 006 |
| 0030 | c054cd8d153de503e31805215a9199cff200e7b78c4d205265b645a0ab4817fa | 006 |
| 0031 | 0868dd2831fb25aaafc1e69f1f24061018f7ad9f34ffd04ea08f0fb80b8a9595 | 007 |
| 0032 | 4959abee23b4c1871aa7a7cac374da80044773a394e12cc9cc89891b6117b6d4 | 007 |
| 0033 | 88497144c86941446aa891795ed0b2870a1ab8cc84b2aed5e8b432d10eb23779 | 007 |
| 0034 | 1e5d149b8b6f7307800ed9906210b5515d5dbf52ba15c5d328d4dbba1059f23c | 008 |
| 0035 | dea858c3a787b9d15634c09e3dd6119d5605e4f0afbb5bd686fde7cdabab4de9 | 008 |
| 0036 | fca92f6de182a6858f0e060445e91f4cd97203ab23a39a3fd3d282eb6cedf7fc | 008 |
| 0037 | 43dcdc113ef105651409187db5e5ba4c81e33335972159b7f1388423e43db54c | 009 |
| 0038 | 8168b68338bcae3c4d8369faca51af19bea6c6b50f8542c6d2a20672a1645f9a | 009 |
| 0039 | 5bbc3521f45a258e15c719f325c8a13b54b5008be921a7a3a6a3b0f5e8de7d82 | 009 |
| 0040 | b0662a09ec72182c69aa9a5e059673bdcfec37f7765d9fad41a2483adeac2a3d | 010 |
| 0041 | 384746b34c2728a67b63f401ac41362bf0b394148113ded750294aa8ac09bbab | 010 |
| 0042 | 745f187e66a6a3086903646c2df01fe05def25a60d4c12596ce352de4a110828 | 010 |
| 0043 | 2c226bd81b1acec23feb01b69698f210afd329cf9490cd8c14f0579f2bf375e0 | 011 |
| 0044 | df52e3ccd3bbcef295fcebff6338eb51e611c7dcd2d30e515ed39acd5d730118 | 011 |
| 0045 | 81f7eed8a1759719406397b739825c6facc14a23f49951a5b9589e625030f602 | 011 |
| 0046 | 83d700225ea2e313901a28f7de65da72f0380eb2b190be839ca69493f146901e | 012 |
| 0047 | 036f905e18ad89a4abc1929dd2269e703ceb71421ae3c6e8d5107dc209258506 | 012 |
| 0048 | 4d17a9dd7704211dd1dbe442a03c21433c3113e73387e3f24c172aabca92ad20 | 012 |
| 0049 | 7245003573bc84c5e113f6846eafc5baa4a799099b70b786981a5f992c47b91f | 013 |
| 0050 | df23dfaebfdbd332a95bedba5b0419366ef732ae999d218caf6612e3e5045528 | 013 |
| 0051 | 146c9832f8ad97dbad94f2c1f8bd80a700107b2bcf312f561460ee01558978ca | 013 |
| 0052 | 90d45a467d4dd56b163eb40258e8357577af7cc104e09561ed9c8d2724b46518 | 014 |
| 0053 | 6b415f369bd1aca974b01c135a80e9afd911a2337b8ba4f967c50053013880c1 | 014 |
| 0054 | f46c35c7dfda54d9d53b8968a01bbfcd3169089b1ae8433f29458425afc070cc | 014 |
| 0055 | ac9ae13c3c0d3bddb0c1d6aee6a669601b505f4a8900f6e7824de956b01b0e94 | 015 |
| 0056 | fe4e2dde09bce51c9da39721e6d92cd72b6b825947601607af41c4f53b83e6b4 | 015 |
| 0057 | d5cc3816f664db374d45fde7eea4396eddaa2fde5b75ca2b1f3dad24a3459bcf | 015 |
| 0058 | 92dab5ceef785b1950f052392d68c9b3e4db97f1c837c03d6d1536022fd5598b | 016 |
| 0059 | 93d7fdde3672c70ab39b0bebb320978433ad7620dc7801ad3eb62f39ac33f520 | 016 |
| 0060 | 9b7862418d1938556b3673691550815185135632e2798b7a34f7faefee360294 | 016 |
| 0061 | e88c9b5299504514fda5483c4a46de5629ccc5af61708ec2365fcaa0239c6b55 | 017 |
| 0062 | 46f4552211832043340b93f304f57d2308eaf744fbc5cfbe82b7f5db6bacb391 | 017 |
| 0063 | bd801ec59742412d5e87ba837fd0de85e2f8be0796750031647aa736376c610b | 017 |
| 0064 | 509e19a2d0de4cb3de6372c97a73c4473e07bca216e04879f69da7d01e16e02d | 018 |
| 0065 | eb8a5e6b7e444444935919699267bc5af347f99648c9d56bd5dcf39fabffb960 | 018 |
| 0066 | eb08c99276010b9931e0aebbfe8cc716da97a4d89b39d3249adc2a56157e700d | 018 |
| 0067 | dc18aa8d518170db587f6d49a7572fca24bdd91717b47e1693b964f50eb6273a | 019 |
| 0068 | 99ace2aca7768bd5da81f87c6d87095c6165e3602187d270f97104b8ee3bc2cd | 019 |
| 0069 | f174cd201b5f4c715f0abd2ee66187b2f4fd08e2e71c72d32ead8ee45662e6bd | 019 |
| 0070 | 5b4869f99101778f9eb19b0a80b9fc8c00a303f824850b51085c54068015db9e | 020 |
| 0071 | cd772a2686c7743039182cfa28e01dd15094ae0ba15418cec3bc64d257986d32 | 020 |
| 0072 | 22279e637de13a0cbc79176d69657006478decb2b6fd0e303a0e13f73edc5eb8 | 020 |
| 0073 | df8867290ebe1f1fccaf19a31bc48a404e16374c1d0c41d293fd49e69d445336 | 021 |
| 0074 | 733164b3bce4ae46e50ab305b992d587a0bee982e8f8cc400a4ee21c0f1b768e | 021 |
| 0075 | ce479716bdfe57e294bf98a1313dbbcc9fd6422eaf49aebce9df1f17a711c1ca | 021 |
| 0076 | 4b5dae70c079b395dbbd1512755d211acb0b7d47746123ae2af8c57de27567b1 | 022 |
| 0077 | 59d6d36bfe695d2524fe54c1d9671930005d3b421889e8432809ebb0e8c91a93 | 022 |
| 0078 | 4f951408d5897e310a4a00893047c5dc7210e72e29b3c143a6c485b02935dc20 | 022 |
| 0079 | 075f4a31751de2575cc77bc61914a29aaf6ffa4cfd35480d918441859f61ee0e | 023 |
| 0080 | 87cf5e8d7ec6815f587773f7f8c7a63f0728267983acde0065d51d917aa99847 | 023 |
| 0081 | 2fab0732f18088e2a0c4f1076b0023c8145c3fd120d0d540e48a2f95393771ee | 023 |
| 0082 | 473a3f652b9f3a7a9b2d9b2634c13967a74abafc105b9606836c5b267f463f5b | 024 |
| 0083 | 6db33139b9f44b40bf354b8a4d7f91ce7873b76b072ab02b92160459480cfa6c | 024 |
| 0084 | dae7e8aadf22a278baa026d4f44a167eee8d80a33f8f869c2f8b8b84677f3193 | 024 |
| 0085 | 47de4c0f78340ad9cd68c0bb7842a0f88aa62ccd717a0dfe9ce8be73dc2c9434 | 025 |
| 0086 | 932e34bc6e336b6519e50ea2afba0d3be95785453a4f73ec9344eb6d0f4bef11 | 025 |
| 0087 | cbf2d6fcb5b18c001cf7706a3509f4b4f5a01013752625e9e1ea626bf3be5bac | 025 |
| 0088 | cd0e8a21ea97edbf95a0f3ae753e4da37b75be547bd195b1b0ea7ee5be9af5d1 | 026 |
| 0089 | 129d66cb50eea9e01927357dbe43f78740facea9089296fcf58026607b5f9ebb | 026 |
| 0090 | bae4889803a55d5466658be49c4bbe1c0bc08bb1459f551dee5490476d351ebe | 026 |
| 0091 | fba544ecdcbb8c36b1c576d593f1934fc788d8c65bdbd8ec013541a72805fce8 | 027 |
| 0092 | 8e0e5a935324f95e244fd1e7eec3241b36a25200d7ebf57d71e815ce97712197 | 027 |
| 0093 | f7a99d53f968be396ab73e2502886203e4cbbd1611286ccb19d0bb3a34f84aac | 027 |
| 0094 | 01aba559e1e3b90d52e9ea36c428d0688b292502ee3ddf4b2e6cc15598d01d17 | 028 |
| 0095 | cf164bcad380184ff4eb514f89290f666386dad9a908d2de3c0c8cfb08e8330a | 028 |
| 0096 | 5e89ac0102b36e64ab0666bca5ee418062395e7d68e5fe1a7a872f617ed09202 | 028 |
| 0097 | 9eede2888ab679d92a8e3b29c45b736e5eaa05b468d2002fddafe8a1f1912060 | 029 |
| 0098 | 405892301fad1382aabf6439764543ce5c593e67d7d4c621ae06e039080e2bf9 | 029 |
| 0099 | e2fc1178051aabc47418d4fb85da2b30869f3eece1d988c253fb2ce34b8e1d4e | 029 |
| 0100 | c0f1adde95e05cbec543d93b7579c65abb2ea4f08fe51f794c972cdf02abc085 | 030 |
| 0101 | 5d62d0a31e341186f4253a1d61832c3098f51cc9dcfbb078c87ba0e2c656b52e | 030 |
| 0102 | c0b8db6b71c33ed7816646426c07b274bc54f67f195e820ce5f5ae7e07ab3ea7 | 030 |
| 0103 | 20fa91844c8f16c168d4c68a7ae8229ba61aa616f4d95f1edf6f1063c6eb2a75 | 031 |
| 0104 | 8f10b0ff97d471f5cf5738a591f064e2ba84d9289b2933bca262d8f182ecb6f6 | 031 |
| 0105 | b7df3349f1bf1c18c4257376beb6c492432260ea5db028c516c57bf4fc23bdff | 031 |
| 0106 | 44904626e922a95a59e081f424895a72a2477bdd8622eec3f7dc80d1ace8224e | 032 |
| 0107 | 9e9fa564674ae5f296cdf9fa038caafa851703330c80c6dbf774e4ed9627e4a0 | 032 |
| 0108 | dc07bbba7796b3482c6430cdda39d42ba6ba119aba7b930f1bf4789a3c5c5546 | 032 |
| 0109 | 3146a3c719e578a312cc22d17ab5b470799d92e5b7367a1af36cdf8075ffd78a | 033 |
| 0110 | fcec088300051bd99d1d0963a4379e410ab3c25fd6fd340518b0b3ae7686833d | 033 |
| 0111 | 4a990cfe503d0e0a9f42cb44ef92e46b478678f739454b53f6f1a2baecc8ce58 | 033 |
| 0112 | 30feb2af9de2c51828f5c5b5140c22669192f84556f5c54d8b75242592865f31 | 034 |
| 0113 | bd66d749f9eea630425e47ba232ac68aabcbb0eb28b2565cbb5731d23d6d0744 | 034 |
| 0114 | 66e64cf11fba7f3809f42e1571baeddee23a1c4a3a038264049dd7c38f2da94a | 034 |
| 0115 | 19205459167f7b69b22bd22dd40b8c77ff745383340da7c96339c508f37390ba | 035 |
| 0116 | 2c1e8c05b0c29145015d0eea08fcc1e787fc23c5f24b2518798a4ae24f4b87d4 | 035 |
| 0117 | 3427e261ba671c7febbe34d851bfd7b32f0d9bb8ef803adc252b0563d33a5392 | 035 |
| 0118 | 28f6286aae3974d4c81dd39f157030f49ea1feaf0a687697d26d71f4c1a6c13c | 036 |
| 0119 | edd8fa03dbf2fecb6f66140d592c81391da7fad4082f7ba3eef68fb3f7246e7d | 036 |
| 0120 | 95550ddae0147476e1ea9ba446c8e2a3bf94930c61ef31ec0cc4bee366fa58ae | 036 |
| 0121 | 9043d0cef0d24d02472b9586bb495c46ea815fa71ad24db7f2fd5750fecbc4bb | 037 |
| 0122 | f9a296f0d954eff6f499990c074b4b250a4dfcf06b687d5db137adb9da100eb3 | 037 |
| 0123 | 408c973df4f1aaa46c56e99cf43047029b8d628d7439a17a889ee7925eba3892 | 037 |
| 0124 | d43386ef566e1c39cd8a07a69cafcfcc48f1ed894fc5611005ddc3c0496ea0c0 | 038 |
| 0125 | 237c2fc19ae0a94313fcca47e7f8c61f3e54a1dbd97c4d0939525595043c5c84 | 038 |
| 0126 | 0797ddef4b1ae05a41a3446ff4c8fd27998af0935a58e9f802fa6ec81aca2d91 | 038 |
| 0127 | ef69f3f7e2a6a26be1fd1b476429d674e553773959e2b2578b895b6aef448e9e | 039 |
| 0128 | f0b1809477809149494e708ba90525d0e8a4c66de2d0f3e7e1a8d850f73c572a | 039 |
| 0129 | 87dacb153ede329bd51700226e4d4cf6ae5a9b65f046abefc1b8829081e2c06c | 039 |
| 0130 | c45aa8b62b5bc017c893c2f8c0370436f82e3ec53cef2007b2c4632407faa049 | 040 |
| 0131 | 5e5863b318e3f24f01e1a6ec8983e328e4a647ea118c3e10589d5fbbf2dea7f3 | 040 |
| 0132 | 1c543dfe73434f95daa33a9ef549a8d3f7ad87d16630ba0a2c5e89e19b7e3d7b | 040 |
| 0133 | c513e99f86b58f25155a7e74c1872b398c7050ed7a46e24e974a020c1d214e5d | 041 |
| 0134 | 14e4bf8706f02a063f4af8978234be52dd5a7dc7ba52e96d4d0b13344f850719 | 041 |
| 0135 | 63f395042a96f52eb3b8890f68c9ca29abc58fb821f33fa7e3facfbcc3a41b6c | 041 |
| 0136 | f67f439982f39be13a53e83128641db1938bc4d7d8f7c19bcad9a8e46e8fbf49 | 042 |
| 0137 | aef4142b8652cf93e9576ad559f31f7872383a117e1160beb8102854387e14c5 | 042 |
| 0138 | 332e04dfe2a010f4873afca266c08c4fb262fe49b5fcb3135584369ab4dfc33e | 042 |
| 0139 | d3bab2bca4cfb063458e3777e959fd9ab472af3a23c22095fe981cf94a27b012 | 043 |
| 0140 | 291d093138dc52f96242f81c393d513a98e3b059b7d48b6a749abdfaf211d6ab | 043 |
| 0141 | 0cca13beb39ae4fb8fdcc913efd05ae1799168ab4ac7c1f497aabcc8195796bf | 043 |
| 0142 | 0116f9036b22b5ddfad6119d237495e9ac18e9b5ccce51828a8fd2635746f114 | 044 |
| 0143 | b4798a21d8d87e4e1392379b80817b097942b7294e906bd88c6dac0d8c5ceb2a | 044 |
| 0144 | 33b5841d9c142e4ec71b20940931edc3694177f5e7801a6897a14fff5792745f | 044 |
| 0145 | d6f8c25f0d90b7338c63c3310efc2dd9ad3bd095f8ab1427a9316dfbb710193d | 045 |
| 0146 | 48065b62afeb7dd16139ccb03c44a8ed81d73d55d07607b5aaf469fe9cfbc97e | 045 |
| 0147 | 324d0219be0605354bd306c69138c20073365fef23613d2bc316089406058406 | 045 |
| 0148 | 68e6ce60056a43d302d883441a8598d54158eab3a9379baf74b93f29041fb47b | 046 |
| 0149 | d1684427163195d75bc0fd32a2ab4588b3c91569c6e6ca8acfa8f8858b2fff22 | 046 |
| 0150 | 65782df108e4dfc4916a46decfb3361a404ec1a6a26ed3dd04dbe93fae832d02 | 046 |
| 0151 | 3beda3913968740bcc74073e1db0fa9d959084aeaaae8e330c91df539a2fd982 | 047 |
| 0152 | c16acd3357e159b46c2980fd41c625ea1410d655fbecbed9a16dfe5a38961d6d | 047 |
| 0153 | 8a058f768f11e6a2c2979a8943195b129281c8ca59407e7374f465d9340429ae | 047 |
| 0154 | c2750f326574964fb8a712aa4a40277356f699f8d060a509c708568e9a498f0c | 048 |
| 0155 | fb0d0f54f32ec547cb06c0c4c7210ff041620f40ea0d2bb747dffc4d1a0c973f | 048 |
| 0156 | eefda0ec1ef17b8db65e0f5c312f113d37667f3e300e7e3608bc77b6039560c6 | 048 |
| 0157 | 5d7eef8bd4d6d72be4e3f2403589096b3d87a3c281d8d6510492e48f33b68095 | 049 |
| 0158 | a799bb9571c65994ce155699b0169778f8d0e9502cc08891017fec09b02ebe31 | 049 |
| 0159 | 974150b27e7ee0fdd5b9ce42675169992e91f91ff3660b55c02a6fb87f1d9cbf | 049 |
| 0160 | 261ae77d7234be64eda7198dca45adc69de541ea2577ff85c9461511fcc6bda9 | 050 |
| 0161 | 46ff42bd2cd64d72e9c512e10c20dfb1e72080fb56970961e37b2c8644cc58e6 | 050 |
| 0162 | 0e6dac2be6674f690e0b11483b32890b185509277a638656c8f40700619944dc | 050 |
| 0163 | fd9a095520b2212fbefbcdf1dea185ba99c0b6721cf67b215f7e2a75ade0eafd | 051 |
| 0164 | 9585587314bea08107b35a436a9a1d7568bd4febd16e5617e99efc882c04d0f5 | 051 |
| 0165 | 319d111cddf972dfd6953402f4c78dbc9dfacc284862d55daf43e7ee432d93e6 | 051 |
| 0166 | d09390a27cc7854d07350d03208e224c20cd6c3bc869f4484d94ed953836a542 | 052 |
| 0167 | 1ccc4694b6c03cb2129f02ad97d22fd39deff6a78d3ba15ab5284ce78ff01132 | 052 |
| 0168 | 397e464b29ec555aebae17d13381233a1274f3d4945600b121f4e9fe328a7d20 | 052 |
| 0169 | 60653040b4e70044729e8170cf592ddacc4f3796b6a34c9d55b074ddfdf5dd60 | 053 |
| 0170 | 9fd13b457e03f38e23aad5de762a548b822a98ed3f2ee46ad9ffc0cfcf6f0a4e | 053 |
| 0171 | de3292368ddb7b6fc8a8403c0d622f1651d200c819fee4bd7b0be35fad09f530 | 053 |
| 0172 | adc69e18e0ecbfb535971b57541ae681346db74fdcb0b84851b895ee6164eb61 | 054 |
| 0173 | 71e8f54625af00d070e0f06927d0e0696797af6416fe80a8af6bb38c22f68831 | 054 |
| 0174 | cf5b1d163479d88920782c4b7f24bc452d865bbfc3646049e6626150925ccc99 | 054 |
| 0175 | 9da96e6efd035615c0ba983f5b91cdd28fbecb6126b071b29eb994bb09ba2f99 | 055 |
| 0176 | 0f146ca7ae9bf9d2852692576df45e27aac02b20c20a8ad3dc6d6f1597419d40 | 055 |
| 0177 | 740dcf421a73c42a82831579a9bb368e62e8866d52d5d9815abec38885cf3e42 | 055 |
| 0178 | 00e94fe9976c01f32ba26c5d4dfd6e488d494af3a3572de09140768209e741f3 | 056 |
| 0179 | dea8ac5429aa063f20549125a7ce0229cafbb9fff2f5669664ec6c896430f1a9 | 056 |
| 0180 | 74baae025f0aa10766df948a9b2e8b8a097677608efcf2427ac60b50982a5521 | 056 |
| 0181 | 6a6e769a35aaa1379d00743b3a4bfdcbd5c39456f8370591c5495fa0977c89a6 | 057 |
| 0182 | 613e35a0ccacf81095fa9c3a07a1b9912d134093c72c752685bd9666b2566b56 | 057 |
| 0183 | ab41df2b6d387e7842ae034bf902ea73c7f3f3197a4bc7c6d6d8f43717ed9dbb | 057 |
| 0184 | 781b85a078fa257321c8949b8cfec76ab1670c8a3ef2cd2d168c5ecf4df61d6c | 058 |
| 0185 | 61dcef4e6761bcd559cda111ed6cf4f5e961d2aed94b89d5f38be0246474ec8f | 058 |
| 0186 | 6499ce6ced98c2dcf14fb53c8fb139ac404749577b262c4798c19ce6a751599b | 058 |
| 0187 | 8215a455f8d2369a5bcb11d474e53fbe74bc213f7024d158fbde78c4eb318bb2 | 059 |
| 0188 | eb1dabb483fc9830844e127b29850fcc9f19fe91480784979d0db3dc7ab084af | 059 |
| 0189 | 9b07f4b737c9907243986a107925cee8b5a3187f8a412e745e54941bf74069e3 | 059 |
| 0190 | 6f8965c3eb2e26689309aac7f2f573c0279eaa66025c03114a87367be1db6643 | 060 |
| 0191 | 6c69da65cf11eea8346cb34ec83e0dd770eb997b09634f841528715bfe9c3884 | 060 |
| 0192 | 88d6c9df051a4a903aad6ebab06c3c59c4921d398cbf1e9d1b744c5eedae8007 | 060 |
| 0193 | 6377cd1dcc1eb5d4662a470c442a6b1410f169080e31c892c20583a64b8ea942 | 061 |
| 0194 | 5554e0a05d3c981806318fbc672eb42663f66262d5d17d7fc2ed628d6c28be99 | 061 |
| 0195 | d96221da0edb69f5946cecdf11d96ef593630d93ccca510de67fd32a412b2896 | 061 |
| 0196 | 204807e3ef45072cc5e92dcb9e138eacf5a50e7ccf1fb60282c205735499327b | 062 |
| 0197 | dbde74d739039ccce62a9eeabff888c62b104fc9b87e0642e0cf486dd2d12b93 | 062 |
| 0198 | 94d0208c13e2ed48fd00c4b7eb5a33aeef5fe3940839df3a4b3da0d6e62f770b | 062 |
| 0199 | 2ccba0f75856462b50e8ee31a9253cfec5799dfdb2a471e1101d35cfc342c052 | 063 |
| 0200 | 037b29f5fbb8b4329df71ffd976200fa46cbb2787b225ba386f7f1167b3d8ea4 | 063 |
| 0201 | a9dc39bf90c3b43c628a2004e0705bb28ca61566198ccec72137f03b3f717ec9 | 063 |
| 0202 | 744c45817cdeca1a90a294c8350efff3212fc69f814120ae88fdfb52dc9d953e | 064 |
| 0203 | 29a5975b31d329f7ff471c6684ee05fca291c9ae297856ec6725c99cd18bade3 | 064 |
| 0204 | b7e6f5d69e401e5f3fb6e670cf85d882a9720877e6f07e797bd5f362e2ff872c | 064 |
| 0205 | 79d5983d69c4988754676d930fddba9a6ef73d8ca4c76e37d91a313de369773d | 065 |
| 0206 | a9b3f46269dc415c1e723f3e732bbb57cec6d976cd4b1a780733c4ec1b35dc3c | 065 |
| 0207 | 48d50e0ff699576db941147f96c85a4d44ca284686716a017b68af713ab98752 | 065 |
| 0208 | 813ccc3b8279f1eacbcae35edc4ac886853497d04b3c050dd007e01393c5010d | 066 |
| 0209 | db24770f865bc139397ba89d983229cf04de0848fac38bb8c17eec1ad31dff69 | 066 |
| 0210 | a76df28d67169e6730bfc0292bfc872b2d18b46d17d895876e50648481a8ec02 | 066 |
| 0211 | 5e3c7a7723d6e167da03dfd3a84986c43e822d8e24c62c31e7653e7e7a91218d | 067 |
| 0212 | 278466b5ed35d885ccb86cd692314174682eb09f314e3f0dd0ae09ecb8e4b22b | 067 |
| 0213 | 31cf8e856feef0c5071ce715e7f9f015a77a3b761542d3bca64dac97aff7f8db | 067 |
| 0214 | 889ab07a272f41bbcbff4e0e6dece687557c363c9f6f9f9e71667870ffd86453 | 068 |
| 0215 | 272a9f20add093c56bf48e656ce81e74a074b448a1bf6198f0ba2819283907cb | 068 |
| 0216 | de969b9095150dc5ed4672b22375e3ec8eddf612dd20999195f7dfa2a1439625 | 068 |
| 0217 | f2c19849fa5194535304cc1596d6554303d8a21235256be726b223637c3c69ef | 069 |
| 0218 | 23ea51ee08847c7df4f73114ef1707176750c9a6dfbaff616f7d3185147915ef | 069 |
| 0219 | e907bc32eb995f162ef0d7783848fa85546e2842b0f87d21a24f44bb12e522d8 | 069 |
| 0220 | f8e6363ba3edaa328e864969febcdb7366de83a634bc5afa0f120a2cd514f167 | 070 |
| 0221 | b35585fae2af60eb9a1ab841303aa335028bb11bcdd83fca0f9c10326a017c58 | 070 |
| 0222 | ab9713b709b24343b23bf60ff453884daf3a97baa82988c887844dd7b1aabd3c | 070 |
| 0223 | e26d134e3e2266a682f543cf79f0a3af3c9ad8cfaacbc6e607edbe9cfb510365 | 071 |
| 0224 | 4986270c36b64a77cedd8de1b9a5c84b2ca29d51d48e5b4090ba6a0abc5bf64b | 071 |
| 0225 | 71c2753c3c9406b1682f737191f70ac1400b5586ad22e61ee0075d89bac5ff10 | 071 |
| 0226 | b0b4c557b422312b9e68aae8234b9f77f7a7e6d791962cb5d6e2c046653d453d | 072 |
| 0227 | a9ed1cba8736e6101b6d45fb2cd7ef2828bd22a39b22263fe5f6e630bf727d31 | 072 |
| 0228 | cc43eb6e2af28f2e7c5aeb7376f0e3184d2de5c3ca6f72effbd9299cedc23267 | 072 |
| 0229 | 8ad2cd879bb5f6466beb7dfe10d5fcdf326dc7d84b750ba67dd4984e5fa1764d | 073 |
| 0230 | 485b8ae40965587d457fe3cecf4721e77f2b778dd8e82b611dd2728b04ad1c30 | 073 |
| 0231 | 8a0ede1d3b970117cdb5b06198b62db08f83c069dd55148e510d46b87b3172f5 | 073 |
| 0232 | 63321510cd43e9a03f2ad87291d5b28c7ef8ac51f8c1eec21ee75b8b3b16bf1a | 074 |
| 0233 | 5eefdc09c45e13d2995c664f5ffe1a31c370ff64131c068cd2c4321929c91d4f | 074 |
| 0234 | cf1d8d5da29913827f7ff07c78f437386daeb0660b9199fcc8b0a32161754eea | 074 |
| 0235 | c3b6e8acd988ce20694c84185b6b87958e8eed975e383cee622f6197dbf13943 | 075 |
| 0236 | 8d32910ddbf511b3080695b693a6fa320006269645cc9e2519664d54b7d80f73 | 075 |
| 0237 | 8b944ed99e1847724488a82dd47b00572321629b7d3846c4f9d30f50eead67a3 | 075 |
| 0238 | 490ba2fa04e3409240ace4b5b9fc6aeea67ab29d6c54368ccaead75ef5797dd3 | 076 |
| 0239 | 8847ab546f4addb7402f110bf2a01b45964386ebc6ca67e3850e80b0fb5e4d17 | 076 |
| 0240 | 0fb509d87ddfdb5aa0ec9097ec97dab2d741ef564b97a0e23ce28dec1f25cb2d | 076 |
| 0241 | d868049599d150cbab5db940193af26fe08039cc0d72d28cce2750f3f08c45c0 | 077 |
| 0242 | 19610311745acabf52ed8976330454332c81ce88a1806a04f88211a80e2fe249 | 077 |
| 0243 | cc8bfef91dca70081476a971a07a6baff7b806831dc4056884e72dc4c4c9c947 | 077 |
| 0244 | fb68b9f40b1e82a4795deecb8254d7250f0e554d26560eaccf7bd8a32b978df8 | 078 |
| 0245 | 2409ca96cba2f80f15f36066e3226510ebd30a6efd2f7e5dd9e4a7a78d3de2f7 | 078 |
| 0246 | 8738b41f39ecf000e980815b16b7b7b8c309c771d7518cfe53d62dea28a5936f | 078 |
| 0247 | fe4e2a75af49466bd79a413754ce971f820c91ab6c7d60ca507c6bd6dee20477 | 079 |
| 0248 | bbb58c4b747abf06ddb1f7860ee8158933e70937063ffae4717632b8541a3b28 | 079 |
| 0249 | 3a1194087cba9018071a9157f35638b4feb1cfbdfc9672e9c4e5544fd68f4949 | 079 |
| 0250 | c1f49a05f6027e44212e1e28a1ccd4779ed39f517fd7f7d4d3b0b457abbf8f9d | 080 |
| 0251 | ad7eaa9176b951c2190a02c4b5a18c333567dfb4d8c4d19ad32af5371955b439 | 080 |
| 0252 | 9eb791bb77db0cf9c5e9ea17c74f1a5db1e2c3755437f15e2395eb5602101fbc | 080 |
| 0253 | f82e19293e47c9ce580c45851959bd316f8f641dce7750eab721da4533b843ff | 081 |
| 0254 | 9c760f52c21a8fb135734e8906fdac47221679282f8a824d9c7c6521a2f6d1e9 | 081 |
| 0255 | aed41e87080fab3464af618810dbfedff67deffccf05f4750051ecd82445e39c | 081 |
| 0256 | 7d4a8c08e7cda30fa1cc7576de93ab9c6f9a9189d05d0874902150d377b2003f | 082 |
| 0257 | b1301ce76e0edbdb91574c583af897571014463a7ba15a7905f276aa08564fde | 082 |
| 0258 | 58904389c3a87c47d72e9363d7d0f3f3a8790e692b41cdd5b0220353f88f89a2 | 082 |
| 0259 | 36a488c314d3be7e756de1b707c28445adc79dd0ed802edfc9038969e12daf9a | 083 |
| 0260 | 9387c6d585f545152416d812092467fc936be1d26e852797d9e31b6259acb010 | 083 |
| 0261 | 641b0d2b206d34bd41f91324eeda1e8d835923d883d2f8c5d5afc6662403c1da | 083 |
