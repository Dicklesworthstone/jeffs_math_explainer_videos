# Final 14 picture review 221

RubyRaven, for root, under `mx-video-program-program-87do.39`. This is an independent review of the selected final 14 (`14_continuation`) stills.

This review covers stills and source/technical evidence only. I viewed every contact sheet as an image. I then viewed full frames at half native size and native zooms for every priority detail, and ran read-only Python measurements at full native resolution on geometry, margins, bar heights and the colour fields.

I did no listening and make no claim about motion between samples, sync, the master join or the final audiovisual result. The soundtrack's voice is ElevenLabs Frederick Surrey according to root; I did not hear it or check it. The one audio measurement below is a 50 ms RMS energy profile, used only to bound when formulas appear relative to quiet gaps. I ran no job, extraction, render, synthesis or build. I changed no file except this one.

## Job

**Folder:** `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-final14-pictures217`.
- Its `sampling.json` names movie `c00d0c837b81…`, the sha256 of `final14-terminal217/14_continuation.mp4`.
- Its receipt binds `final14-pictures217-inputs.json`. I started no other extraction.

**Receipt:**
- The helper is `final-chapter-pictures115.py` (`062413519aa08f4ee6792fce9bb92cfa870e4eff66fe795035db77dc648ca3f8`). That matches the inputs manifest and the file in this folder. The command is `--source final14-terminal217 --job complex-final14-pictures217 --chapter 14_continuation`, with absolute paths.
- The guard admitted the job at 04:02:26 UTC on 2026-11-08. It finished at 04:04:24 UTC with exit 0. The receipt records no stop.
- The inputs hash was `9f18537968b82e64f6b0a1cb7bfd653202d235d4909b3161069f1b148ff49a64` at both the start and the end. This equals the sha256 of `final14-pictures217-inputs.json`.

| Job file | sha256 |
|---|---|
| `resource-receipt.json` | `e194ff7842c80f766f8cd08ac24c399afb795577003142bbfa10de50153f2783` |
| `sampling.json` | `ede15fc40e7f97e94f701b0e5c2b14173104db601e0243736a236e5a18aa2115` |
| `select.filter` (3,003 bytes) | `a78a9ee5ab37851a91c4b5225d3071b22a54d63fd6b1fa76cfefd98cba00d9cc` |
| `command.log` (0 bytes) | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

## Inputs (`final14-terminal217`)

| File | sha256 |
|---|---|
| `14_continuation.mp4` (11,756 frames, 195.933 s at 60 fps) | `c00d0c837b81174ce43ee283e2366f7bc450d159310596626d3260d708e19bf4` |
| `14_continuation.timeline.tsv` | `24e8f8c6c76eaafea17f560712239c981bf3368f15dd66091a1e0dd7b5213330` |
| `14_continuation.cues.tsv` | `ed6ffa4c6fb34de7af8b217b61c1b6e2fec133132f2b4bd06145078d607223ad` |
| `14_continuation.cuelog.tsv` | `d4d02a811861127d2a9b79e3fa19e937d11794d982f64bfc26be1cf2d082e6e7` |
| `14_continuation.layout.tsv` (empty: clean lint) | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `soundtrack_0.wav` (48 kHz stereo, 195.933 s) | `7d7b558721567b2492bb8dd5d5d2285992eb71b6ede8990085d0fbe6de27e3e4` |

- These six files match `render-summary.json` (`ca4da94a9ab05027…`): binary `f4836eeeb4f2…`, permit `10838cbf5066…`, x264 CRF 16.
- The folder also holds the render's `resource-receipt.json` (`d96667c3ea83…`). It holds no pictures-reviewed file.
- I read root's `final14-pictures-viewed217.json` (`d87aa1833fe8…`) only after my own pass, and nothing below depends on it.
  - Its observations agree with mine on clipping and on the subjects reviewed. It records no contact; L1 and L2 below are contacts.
  - It calls the 155 s formula "close to top but fully visible". I measured its top at y 391, which is 18% of the frame height below the top edge (see What holds).
- I did not read `bead-final14-review217.json`. It is 351,065 bytes and not a single JSON document.

**Timeline:**

| Row | Frame | Time | Duration |
|---|---|---|---|
| line 14_1 | 36 | 0.600 s | 18.800 s |
| line 14_2 | 1200 | 20.000 s | 21.440 s |
| line 14_3 | 2523 | 42.050 s | 35.120 s |
| line 14_4 | 4997 | 83.283 s | 5.920 s |
| mark predict | 5389 | 89.817 s | — |
| line 14_5 | 5704 | 95.067 s | 31.680 s |
| line 14_6 | 7641 | 127.350 s | 19.600 s |
| line 14_7 | 8853 | 147.550 s | 21.520 s |
| line 14_8 | 10181 | 169.683 s | 22.640 s |
| breath | 11576 | 192.933 s | 2 s |

The timeline also has `listen 5389 5`, so the think window runs to 94.817 s.

## Coverage

- **Samples:** 230 native JPEGs at 3840×2160, frames 0 to 11755, all unique, with no gap longer than 1.0 s:
  - 196 one-second samples;
  - 32 cue samples (8 lines × 4, at +0, +30, +60 and +120 frames). Frames 1200, 1260 and 1320 are also one-second samples.
  - 4 samples at `mark predict` (frames 5389, 5419, 5449 and 5509);
  - the last frame.
- **Hashes:** I re-hashed every picture and every sheet against `sampling.json`. There were 0 mismatches.
- **Sheets:** I viewed all 29 sheets as images, 28 × 8 tiles plus 6 on the last sheet, which is all 230 samples.
- **Full frames at half native size (1920×1080), viewed as images:** frames 1080, 2400, 2640, 3600, 4320, 4440, 5100, 6000, 7200, 9300, 9900, 10500 and 11460.
- **Native zooms at 1:1, viewed as images:**
  - the 14_2 title (frame 2400);
  - the ζ(−1) line (frame 7200);
  - the Gamma integral (frame 11460).
- **Measurements at full native resolution, on 43 frames:**
  - 900, 960, 1020, 1080, 1140, 1200, 1230, 1260 and 1320;
  - 2400, 2640 and 3600;
  - 3900 to 4440, every 60 frames;
  - 7200;
  - 7740, 7920, 8100, 8160, 8220, 8280 and 8460;
  - 9300, 9840, 9900, 9960, 10020, 10080, 10140, 10181 and 10200;
  - 10980, 11460, 11700 and 11755.
  - The methods were circle fits to stroke pixels, axis and frame-line scans, bar-top scans, tick clustering by angle, and hue against the argument of the drawn function.
- **Soundtrack:** a 50 ms RMS profile of `soundtrack_0.wav` over 147.3–156.0 s and 169.4–181.0 s. This is energy only, not listening.

| Sheet | Tiles | Frames | Seconds | sha256 |
|---|---|---|---|---|
| sheet-0001.jpg | 8 | 0–180 | 0.000–3.000 | `0e64d5e1cbbab0454932eb6d6b38a8b1bc3b7c846255ea6894f056188e5cc99c` |
| sheet-0002.jpg | 8 | 240–660 | 4.000–11.000 | `03bc3140c78c841ef956a63dd154da0bf6e51d051b87838f7a4c67b9c6fcb966` |
| sheet-0003.jpg | 8 | 720–1140 | 12.000–19.000 | `2d383fa30a190c0c1a8d7df9c8126081dae2c29322ca8342a2e2717a79da553c` |
| sheet-0004.jpg | 8 | 1200–1560 | 20.000–26.000 | `3986e00e79f13089f2ea4efe784fa0175c33c69ff25a7a5e86c1f293d1599a3e` |
| sheet-0005.jpg | 8 | 1620–2040 | 27.000–34.000 | `1a4580a5f37f67e1095b36c4382314869d26be40e3e89db1e703ce11212305c3` |
| sheet-0006.jpg | 8 | 2100–2520 | 35.000–42.000 | `ac24e5a6f1bcda8b636067de5d1db52b71e96a276e96fbd9655bab99e65aaba5` |
| sheet-0007.jpg | 8 | 2523–2760 | 42.050–46.000 | `997f74f2498808c659f3e5f98dc7e97abf3eb63136f1c9de292e60325a357f45` |
| sheet-0008.jpg | 8 | 2820–3240 | 47.000–54.000 | `0bb8917aa84d85a588215e8b4cd0d70961c05ff0eb7dde98432622c899227d92` |
| sheet-0009.jpg | 8 | 3300–3720 | 55.000–62.000 | `a8c7a40b9f58e4c2040bedd5379e1511a3629349308a05bef1d789501094a6cf` |
| sheet-0010.jpg | 8 | 3780–4200 | 63.000–70.000 | `641f348aadda9425100a81b3e751bf0a6c8caa1b3796cf92e80e88bc6ba81c26` |
| sheet-0011.jpg | 8 | 4260–4680 | 71.000–78.000 | `56874452e416d0c0d0411fc16fc89d91a51b371b91062bb37bcc0d245881677d` |
| sheet-0012.jpg | 8 | 4740–5040 | 79.000–84.000 | `f481b7aa7a0f7d39a9e9d8be75d8db451b5abd8d8afb37f5ba83820f26730f90` |
| sheet-0013.jpg | 8 | 5057–5389 | 84.283–89.817 | `7718494366cf3145e3f98d3bfda026d0fff8bcf1086d621f12ab23fab830c816` |
| sheet-0014.jpg | 8 | 5400–5640 | 90.000–94.000 | `447603bf375253d0beefc1aade9031ebfe52872e0ce123e12a2c7ca81040a61d` |
| sheet-0015.jpg | 8 | 5700–5880 | 95.000–98.000 | `afb73d112f43d37c637caf0f2feb7204a14d15d76ab761d59aca2a0855f8ca2b` |
| sheet-0016.jpg | 8 | 5940–6360 | 99.000–106.000 | `e66a7623d4ee1826ae2a7e8370c6cc1cb190bd0346bc6379dff46d62381d05f5` |
| sheet-0017.jpg | 8 | 6420–6840 | 107.000–114.000 | `de67e0bd22227465edda16eac5e1dcb032cdb7b0a6a4e6fc4581b6d136e1ffaa` |
| sheet-0018.jpg | 8 | 6900–7320 | 115.000–122.000 | `d269565902bfe3a4d68890f7972869e2d452d520ca20505a92d7c5736c923e34` |
| sheet-0019.jpg | 8 | 7380–7680 | 123.000–128.000 | `dcf480b447283360701ce1cf9d323da7f2603e476b1d6775cf44f0b987936399` |
| sheet-0020.jpg | 8 | 7701–8040 | 128.350–134.000 | `52806caf942b1c2f5b6d01a384ee2ba65a348156eeaa3546313bde3db5894018` |
| sheet-0021.jpg | 8 | 8100–8520 | 135.000–142.000 | `d98e0ab3ef0813c2eed6833bc2f7263fe36e27e24fff2539d846071d00994893` |
| sheet-0022.jpg | 8 | 8580–8883 | 143.000–148.050 | `9281fb4f355ca742218fac7b0c79d0c5d35b95892da036b582ddc33f914b15a8` |
| sheet-0023.jpg | 8 | 8913–9240 | 148.550–154.000 | `fcc8e5a21dc899f67ec2419a1a87f737eabe53bb6baa3841717be7bcdd8c4340` |
| sheet-0024.jpg | 8 | 9300–9720 | 155.000–162.000 | `dea55e54468b6639c140be6a54af7f9ca902f7085d10d989a31b0b4d4759709f` |
| sheet-0025.jpg | 8 | 9780–10181 | 163.000–169.683 | `e12a2a1f35c4cf5fc4a4c2d3abfc43d028010a1c3a6fba87eb3eccc022e5cde5` |
| sheet-0026.jpg | 8 | 10200–10440 | 170.000–174.000 | `d71545d1d588c09410b3194ff96ebc0d6fad7651b32e4b4cf48580a0672947b1` |
| sheet-0027.jpg | 8 | 10500–10920 | 175.000–182.000 | `74df85df50338197fa21314cbb71bacd05179e034c61e97ad3547ecfbafbdc18` |
| sheet-0028.jpg | 8 | 10980–11400 | 183.000–190.000 | `01862aa9e7a73b0565a76ffab675ff2e919ad5096cb1917593fff6c2fa66555f` |
| sheet-0029.jpg | 6 | 11460–11755 | 191.000–195.917 | `d2fc5c0ab85778d29c419987008272ec47f3814b9dbfca7b8a039890a4de565a` |

## Verdict

I found two Low contacts and one carried Low type-style class. Nothing is High or Medium. None changes what the chapter teaches, and none justifies a re-render on its own; that call is root's.

On the four priorities, all checked at full native resolution:
- **Continuation overlap:** every chain disk's centre lies inside the previous disk.
- **Taylor-disc validity:** every disk's radius equals its distance to the pole at 1, to within 0.8 px.
- **Boundary narrative:** the lacunary picture matches the 12-term sum, the 64 roots of unity are drawn exactly, and the natural boundary is the unit circle.
- **Formula visibility, including 155 s:** every formula is fully visible, with at least 154 px of top margin.

## Findings (all Low)

**L1. The four cardinal root-of-unity ticks run through the plot frame.** Seen from 164.000 s to 169.683 s; samples 9840, 9900, 9960, 10020, 10080, 10140 and 10181, sheet 25.
- These are the ticks at 1, i, −1 and −i. They are absent at 163.000 s and faded by 170.000 s (frame 10200), so they are on screen for at least 5.7 s and under 7 s. The contact is held, not transient.
- **Unit circle:** the red natural-boundary circle (frame 10020) is centred at (1109.8, 1174.0) with a radius of 694.8 px. The ticks run radially from 690.5 to 732.7 px, which is out to 1.055 units.
- **Frame:** the square frame's lines are at rows 443–447 (top) and 1901–1905 (bottom), and columns 378–383 (left) and 1836–1841 (right). That puts each line centre 729 px, or 1.049 units, from the circle centre.
- **Contact:** at x = 1110 the top tick is red from y 441 to 469. It is drawn over the whole frame line and ends 2 px beyond the line's outer edge. The right tick does the same (red at x 1820–1842 on row 1174). So do the bottom tick (to y 1906) and the left tick (to x 378).
- At sheet scale the four ticks read as fused with the frame (sheet 25, 164 s tile, top edge).
- The other 60 ticks end well inside the square, and the circle, disk and labels are clear of the frame.
- Readable, and the meaning is unchanged. Ticks shorter than 1.04 units, or a frame a little larger, would clear it.

**L2. The "names to search" panel sits flush on the bump plot and hides its bottom frame line.** Seen in samples 1080, 1140 and 1200 (18.000–20.000 s), sheets 3–4. The panel is absent at 17.000 s and at 20.500 s, so it is on screen for at least 2.0 s and under 3.5 s. It overlaps the start of line 14_2 (20.000 s) by under 0.5 s.
- **Without the panel (frame 1020):** the frame lines are at rows 416–420 (top), columns 270–275 (left) and 1783–1787 (right), and rows 1928–1932 (bottom). The square therefore closes at a line centre of 1930.5. The colour field ends at row 1927.
- **With the panel:** the panel's top border is at rows 1926–1929, with its fill (24, 32, 43) from row 1930. The field ends at row 1925.
- So the panel covers the plot's whole bottom frame line and the last 2 rows of the field. The panel and the field touch with no gap.
- The text "names to search: analytic continuation" is fully legible. The −i marker (centre y 1471), its chip and the gold circle are all well clear.
- This is a much smaller overlap than final04 L1/L2 (181 px and 84 px). It may well be intended. I report it because the plot loses its bottom edge for those seconds.

**L3. Type-style differences.** Same class as final13 L4 and final04 L4, which root accepted in disposition 206.
- **Minus sign:** it is short and heavy against full-width plus signs, here inside typeset formulas as well as in text (1:1 zooms):
  - the 14_2 title's denominator "1 − z" (frame 2400);
  - "ζ(−1) = −1/12 ≈ −0.083333" (frame 7200; the `≈` is also heavier than the `=`);
  - the superscripts in t^{z−1} e^{−t} (frame 11460);
  - text and labels: "1/(1−1.2) = −5", "1/(1−2) = −1", "here every path gives 1/(1 − z); not always", "√ after one loop: −1", "a new pole at each step: 0, −1, −2, ..." and the "−i" chip.
- **Ellipsis:** the quiz box "1 + 2 + 4 + 8 + ... = ?" (frame 5100) and the pole caption use baseline "...", while the titles use math "⋯".
- **Upright z:** `z` is upright in "Re z = 0", "(for Re z > 0)", "drawn: 12 terms, inside |z| < 0.98", "z = 1/2: 11 terms give 1.999023" and "every z except 1". The formulas set z in math italic.
- **Bare √:** "√ after one loop" uses a bare radical with no radicand.
- All are readable, and none changes meaning.

## Carried open item (not checked here)

The same open item as final13 L3 and final04 applies here:
- Several bottom elements sit in the lowest 15% of the frame, where uploaded captions and player controls appear:
  - the "names to search" panel (from y 1926) and the "precisely" panel (from about y 1928);
  - the "z = 1/2 …" and "z = 1.2 …" captions, at about y 1890–1940;
  - "the series diverges", at about y 1905–1955;
  - "a new pole at each step: 0, −1, −2, ...", at about y 1860–1915.
- These approximate positions are read from the half-size frames.
- Whether captions or controls cover them on the platform is not visible in source stills. It stays with root's open platform caption-overlay check.

## Near misses and observations

- **Opening:** frame 0 is blank background. The bump field is the first content, from 0.6 s (the first cue frame). This was not seen as a defect.
- **Bump circle against the poles (frame 1080):** the gold "recipe at 0" circle has centre (1028.75, 1173.9) and radius 302.25 px. That comes from the gold runs at x 720–733 and 1325–1337 on row 1174; along rays the stroke spans radii 296–309.
  - The ±i markers' dark centres are 297.5 px from that centre. So the circle's centre-line runs 4.75 px (1.6%) outside the poles, and the poles sit on the stroke's inner edge.
  - This is invisible at viewing size; final04's equivalent was 2 px.
- **"1.2" label in the 14_2 inset (frame 2400):** its nearest ink is 11.4 px from the teal circle's stroke. It is clear, but tight.
- **"the region here: every z except 1"** stays on screen from 51 s until the scene clears after 83 s. The region shading behind it is switched off at 65 s. The statement stays true.
- **Line starts:** the samples at the starts of 14_2, 14_3, 14_4, 14_6, 14_7 and 14_8 still show the previous scene. Each clears within 0.5 s, the same hold-to-cue pattern as final04.
- **Chain build:** the green-stroke pixel count grows from 0 at 65 s (frame 3900) to 101,880 at 72 s (frame 4320). It holds between 68 s and 69 s, then stays at 101,950 through 74 s.
  - The walker dot's measured centroid is 1.7–3.5 px from 0 at 72–74 s. Part of that is bias from the white path lines meeting at that vertex.
- **Gamma pole markers** appear about one sample after the field reaches them. For example, at 185 s the −1 pole is visible in the field but unmarked; it is marked at 186 s.
- **14_8 fade:** the lacunary disk stays alone for one sample (170.183 s) before the Gamma field appears. The first Gamma strip shows without the frame at 171.000 s, and the frame arrives by 171.683 s.
- **Last frame (195.917 s):** it shows the bare Gamma field, at the same mean level as at 195 s (127.3 against 126.9). The frame line, formulas, labels and markers are all removed. This matters only for the 14→15 join in the master, which I did not check.
- **z = 1/2 bars:** bars 7–10 merge with the dashed 2-line, as expected for values within 0.008 of 2.
- **z = 2 chart:** bars 0–2 are under 3.1 px tall, which is why the first bars look empty.

## What holds

- **Taylor-disc validity and continuation overlap (14_3).**
  - The axes cross at (920.5, 1133.5). The pole marker's centre is at (1190.69, 1133.48), which gives a unit of 270.19 px.
  - The teal "small disk" (frame 2640) has centre 0.0002 and radius 0.9988.
  - The yellow re-centred disk (frame 3600) has centre −0.4989 and radius 1.4982, against |1 − c| = 1.4989. That is −0.20 px off, so it passes through the pole. Its centre dot (−0.4988) lies inside the small disk, and the disk pokes out to −2, as "its new disk pokes outside the old one" requires.
  - The eight green disks (frame 4320) are fitted from about 14,000 stroke pixels each, with an rms of 3.4 px:

    | Centre vertex (about the pole) | Fitted centre | Radius | \|1 − c\| | Radius − \|1 − c\| |
    |---|---|---|---|---|
    | 0° | 1.9970 | 0.9998 | 0.9970 | +0.76 px |
    | 45° | 1.7049 + 0.7053i | 0.9998 | 0.9972 | +0.70 px |
    | 90° | 0.9995 + 0.9976i | 0.9998 | 0.9976 | +0.58 px |
    | 135° | 0.2941 + 0.7055i | 0.9997 | 0.9980 | +0.45 px |
    | 180° | 0.0024 | 0.9994 | 0.9976 | +0.49 px |
    | 225° | 0.2941 − 0.7055i | 0.9997 | 0.9980 | +0.46 px |
    | 270° | 0.9995 − 0.9976i | 0.9998 | 0.9976 | +0.58 px |
    | 315° | 1.7049 − 0.7053i | 0.9997 | 0.9972 | +0.69 px |

  - Each disk's circle passes through the pole within its stroke, and no disk contains the pole. This is the radius of convergence of 1/(1 − z) about c, as "each disk stops at the pole at 1" says.
  - Successive centres are 0.7634–0.7636 apart, under every radius (0.9994–0.9998). So each step's centre lies inside the previous disk, all the way round, and the path closes at 0.
  - The "precisely" panel (from 74 s) is 148 px below the plot frame (frame lines at rows 1778–1784, panel border at 1925). Its sentence is mathematically correct.
- **Series numbers (14_2).**
  - "1.999023" = 2 − 2⁻¹⁰ and "32.15" = (1.2¹¹ − 1)/0.2 = 32.1504. "1/(1 − 1.2) = −5" is correct.
  - The z = 1/2 bars, measured from the base, are 0.9994, 1.4978, 1.7495, 1.8729, 1.9346, 1.9667 and 1.9815, against 1, 1.5, 1.75, 1.875, 1.9375, 1.96875 and 1.984. The dashed 2-line is at rows 995–1002.
  - The z = 1.2 bars, scaled to the last, are all within 0.04 of the partial sums. The cyan dashed 2-line in that chart implies 2.001.
  - The "−5" ends 261 px from the right edge, and the title's first ink is at row 154.
- **Quiz (14_4).**
  - "1 + 2 + 4 + 8 + ⋯" shows from 83.78 s, and the box from 84.28 s.
  - The countdown ring appears at 89.817 s and drains through the think window (to 94.817 s). It is nearly gone at 95 s.
  - No answer, bar or value appears before line 14_5. The box clears by 95.567 s, and the bars rise from 98 s.
- **Divergence (14_5).**
  - "z = 2: 11 terms give 2047" = 2¹¹ − 1.
  - The bars for n = 3–10, measured from the base, give 6.8, 22.7, 54.5, 120.4, 247.6, 504.4, 1017.8 and 2047, against 15, 31, 63, 127, 255, 511, 1023 and 2047. Each is short by a constant 3–4 px, where the axis line covers the bar foot.
  - "1/(1 − 2) = −1", "continuation extends the function, not the sum" and "ζ(−1) = −1/12 ≈ −0.083333" are correct. "the series diverges" appears under both series.
- **Path dependence (14_6).** I took the dot centres from filled-disc detection and fitted each panel's circle (z: r 405.0 px; √: r 397–405 px).

  | Sample | z angle travelled | √ dot angle | Half of travelled |
  |---|---|---|---|
  | 132 s | 0.07° | 359.48° (−0.52°) | 0.03° |
  | 135 s | 27.33° | 13.20° | 13.67° |
  | 136 s | 153.63° | 77.12° | 76.82° |
  | 137 s | 279.95° | 140.69° | 139.98° |
  | 138 s | 360.01° | 180.50° | 180.00° |

  - The √ value is half the travelled angle within 0.71°. It ends at −1 after one loop, tracing the upper semicircle.
  - "√ after one loop: −1" shows from 138 s and "log gains 2πi each loop" from 141 s.
- **Boundary narrative (14_7).**
  - The coloured disk (frame 9300) is centred at (1109.8, 1174.0) with an area radius of 680.6 px. The red natural-boundary circle (frame 10020) has the same centre and radius 694.8 px. Their ratio, 0.9796, matches "inside |z| < 0.98".
  - The hue at 1,004 grid points inside |z| < 0.9 matches arg of z + z² + z⁴ + … (12 terms), with a median deviation of 6.2° and a 90th percentile of 14.2°. On the same pixels, f(z) = z gives 18.4° and 54.0°. The offset is 358°, so red is positive real.
  - The 12-term sum's real zero at −0.65863 falls at pixel x 652, where the left-hand zero is drawn.
  - At 165 s (frame 9900) there are 64 ticks at 5.625° steps, each within 0.020° of k·5.625°: "drawn: through order 64". At 164 s, 8 ticks at 45° steps show partway through the build.
  - Ticks, "it blows up at the 2nd, 4th, 8th, 16th, ... roots of 1", "all these roots together are dense" and "natural boundary" follow in that order, as line 14_7 says. The red circle comes in at 167 s.
- **Formula visibility.**
  - **At 155 s (frame 9300):** "z + z² + z⁴ + z⁸ + ⋯" occupies rows 391–473 and columns 2642–3411, with peak ink (243, 248, 251) on background (12, 17, 20).
    - Its top is 390 px (18%) below the frame top and 52 px above the plot frame's top line. Its right end is 428 px from the right edge.
    - The grey note below it (to row 622) peaks at (151, 156, 162) and is legible.
    - Nothing is clipped at the top in any sample from 153 s to 169.683 s.
  - **The other formulas:**
    - the 14_2 title, from row 154;
    - "1 + 2 + 4 + 8 + ⋯";
    - the ζ(−1) line;
    - the Gamma integral, its top at about row 366 from the half-size frame;
    - "Γ(z) = Γ(z+1)/z".
    - All are fully inside the frame and clear of the plots.
  - **Onset bounds** (energy only, by script order; which word lands with each formula is not established):
    - The 14_7 soundtrack has quiet spans at 148.30–149.00 s and 151.30–152.00 s. Its formula fades in between the 152 s and 153 s samples, within about 1 s of the start of the sentence that names it.
    - In 14_8, the Gamma integral appears between 174 s and 175 s, after the quiet span at 173.50–174.20 s. The recurrence appears between 178 s and 179 s, after 178.15–178.80 s.
- **Gamma continuation (14_8).**
  - The five pole markers' dark centres (frame 11460) are 259.31, 259.79, 259.99 and 259.25 px apart, on rows 1213.5–1215.5.
  - "Re z = 0" is centred within 0.2 px of the pole at 0.
  - The finished field spans Re −4.785 to 2.989 and Im −1.987 to 1.993. So it holds exactly the poles 0 to −4, as "a new pole at each step: 0, −1, −2, ..." says.
  - The hue at 2,955 grid points (away from the poles) matches arg Γ, computed by Lanczos, with a median deviation of 6.5° and a 90th percentile of 13.2°. On the same pixels f(z) = z gives 57.3°. The Re z > 0 strip alone (frame 10980) gives 5.8°.
  - On the real axis the colour is teal (hue 182–184) on (−1, 0), (−3, −2) and (−5, −4), and red (350–352) on (−2, −1), (−4, −3) and x > 0. That matches the sign of Γ.
- **Bump field (14_1).** The hue at 2,093 points matches arg 1/(1 + z²), with a median of 6.8° and an offset of 356°. "the bump: defined everywhere but ±i" sits beside markers at ±i.
- **Layout lint:** the layout log is empty. No text touches a frame edge in any sample.

## Not done

- No listening, of either the ElevenLabs Frederick Surrey narration or the soundtrack mix. The RMS profile is not a listening check.
- No claim about motion between samples or fine sync.
- No check of the master or chapter joins, captions, or platform overlay.
- No independent final audiovisual acceptance and no publication acceptance.
- No job, extraction, render, synthesis or build.
- No edit to source, helpers, shared files, TODO, memory or Beads. No commit.
