# Final 13 picture review 200

RubyRaven, for root, under `mx-video-program-program-87do.39`. This is an independent review of the final 13 (`13_real_integral`) stills.

This review covers stills and source/technical evidence only. I viewed every contact sheet as an image and then cropped and viewed native 3840×2160 frames for every suspect detail. I ran read-only Python checks on hashes, geometry and on-screen numbers.

I did no listening and make no claim about motion between samples, sync, or the final audiovisual result. I ran no job, extraction, render or build. I changed no file except this one.

## Jobs

**Failed job 200, kept as it is.** Folder: `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-final13-pictures200`.
- The guard admitted it at 03:26:11 UTC. It finished at 03:27:03 UTC with exit 2.
- Its command passed relative paths (`final-chapter-pictures142.py`, `--source final13-terminal200`). The guard runs the job in another working directory, so Python reported "can't open file …/final-chapter-pictures142.py: [Errno 2] No such file or directory". That error is in `command.log` (263 bytes, `ec31d7d6…`).
- Nothing was extracted. The folder holds only `command.log` and `resource-receipt.json` (`710e7f54937fa6b8ea79da32d7b4b7bf91411f0a38db23e1839703c672b7763e`), with no `pictures/` folder and no sheets.
- Its bound inputs hash `8408e8bf…` was the same at the start and at the end.

**Job 201, the one reviewed here.** Folder: `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-final13-pictures201`.
- It used absolute paths and ran `final-chapter-pictures142.py` (`a0430c030b48e9defdbaa50b22494cb7f96a763ce9ec1419233dff752edd8dc5`).
- The guard admitted it at 03:27:26 UTC. It finished at 03:29:22 UTC with exit 0.
- Inputs hash `8408e8bffd835e563c7fa9577a2c0a7462a6b542b3ad5fb9b95dbf1e9de1c580` at both the start and the end.
- No resource stop.

| Job 201 file | sha256 |
|---|---|
| `resource-receipt.json` | `550d38d12d534fa1058a4f14a867b63e16db1ec042524501be5b3492bafa073e` |
| `sampling.json` | `e042208ac856aa2f3f6b4bd611596a9240412fa29aed78792c9aaa44028ad65f` |

## Inputs (`final13-terminal200`, bound by `final13-pictures200-inputs.json` = `8408e8bf…`)

| File | sha256 |
|---|---|
| `13_real_integral.mp4` (10,641 frames, 177.35 s) | `49934034d4611b81155925e667dab463072b21ba0240467447b9ddfb52d32f51` |
| `13_real_integral.timeline.tsv` | `819ffec50efd82df351a6f900eb1d70cc3a5a50cfda6e2767bb7f819bee0a847` |
| `13_real_integral.cues.tsv` | `c19f91943c0a206cf0a0b0adfd31c3d1f8b0214af69106d8df566d91d4940255` |
| `13_real_integral.cuelog.tsv` | `a9d5cd053501b69ff1130951cb43e9f675d8c94a29882953be6db352a14fc4f7` |
| `13_real_integral.layout.tsv` (empty: clean lint) | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `soundtrack_0.wav` | `630e8f38b150364b66bbeaa27d6af8f3f6e1b2ae36f2380514e271394e07058c` |

- The six files match `render-summary.json` (`392ebd1c…`). `sampling.json` names the same movie hash, `49934034…`.
- The folder also holds `render-summary.json`, the render's `resource-receipt.json` (`d67279d1…`) and, since 03:32 UTC, root's own `pictures-reviewed201.json` (`4fb89f1b…`).
- I read only that file's summary fields, after my own pass through the sheets, and nothing below depends on it. Its observations agree with mine, except that it records no transient overlaps. L1 and L2 below are such overlaps.

## Coverage

- **Samples:** 227 native JPEGs at 3840×2160, frames 0 to 10640, with no gap longer than 1.0 s:
  - 178 one-second samples;
  - 44 cue samples (11 lines × 4, at +0, +30, +60 and +120 frames);
  - 4 samples at `mark predict` (frame 4927);
  - the last frame.
- **Hashes:** I re-hashed every picture and every sheet against `sampling.json`. There were 0 mismatches.
- **Sheets:** I viewed all 29 sheets as images, 28 × 8 tiles plus 3 on the last sheet, which is all 227 samples. The table lists each one.
- **Native crops:** I cropped and viewed 11 native frames: 2220, 3120, 3180, 3600, 6720, 7080, 8820, 9466, 9600, 10140 and 10620.
- **Ink extents:** I measured them on frames 8820, 9600, 10140 and 10560.

| Sheet | Tiles | Frames | Seconds | sha256 |
|---|---|---|---|---|
| sheet-0001.jpg | 8 | 0–180 | 0.000–3.000 | `c658c8d95e6ffa74f1176bfc433245f2ed38e599e38091a185de254c2deb7451` |
| sheet-0002.jpg | 8 | 240–660 | 4.000–11.000 | `f42c54634c877e3d192c66bc3ccf248ec93226e56152c188429c163cef10f409` |
| sheet-0003.jpg | 8 | 720–1062 | 12.000–17.700 | `f2fe000a109cc1a9022fdee8573c4f16d02c9b82fb13d29622299616a936501a` |
| sheet-0004.jpg | 8 | 1080–1380 | 18.000–23.000 | `e402fc2b272611666d937b54499ce25753989f0b1650beab535f77c9645bcdca` |
| sheet-0005.jpg | 8 | 1440–1860 | 24.000–31.000 | `16b2521ca3ffdc64ff5105de61ed3a4e7250a88e74aca4733bb6ef040e00772d` |
| sheet-0006.jpg | 8 | 1920–2160 | 32.000–36.000 | `60172488daaad74b1a048b57b129e4efe26a1cf62b18fc827dece7e3f8798309` |
| sheet-0007.jpg | 8 | 2187–2580 | 36.450–43.000 | `8f4f4cc7253c48f9e488b2eed5a35b75ce08e81080e8894bb7ec776067ee7660` |
| sheet-0008.jpg | 8 | 2640–3060 | 44.000–51.000 | `744dd8fc629e3d14c1f3310502d3fd3e694c9f64569a9f8203fbee37092da5cf` |
| sheet-0009.jpg | 8 | 3120–3360 | 52.000–56.000 | `4f010d92eb81e71ebd6c43bbc907eae5a00019b4823ca56fd5ea803be6f0a73f` |
| sheet-0010.jpg | 8 | 3395–3780 | 56.583–63.000 | `be1a33fc0b82aed69a591c397f89d28b88a5c5bb29691a3fef8478a1c84687d7` |
| sheet-0011.jpg | 8 | 3840–4080 | 64.000–68.000 | `1a76048495d9e42092d3cda3c31c8199b53cfc20e2091dd89b1fe2f7dc45179f` |
| sheet-0012.jpg | 8 | 4094–4500 | 68.233–75.000 | `c3c18055af2e7ce94020b35eddb9d7effd68b4d34bcb80f4ad18eede4d3f3d8e` |
| sheet-0013.jpg | 8 | 4560–4927 | 76.000–82.117 | `d63357263d134d69e2e254a99dcfd692649871b9893d7eb4e4741e4b6d1df608` |
| sheet-0014.jpg | 8 | 4957–5220 | 82.617–87.000 | `c6a041c503753227f84d5898043b26e17f9c80cd31b8ec356269ea9414813234` |
| sheet-0015.jpg | 8 | 5242–5460 | 87.367–91.000 | `139053bae6cc7dd76b6ba72bf23533a5668bfe868cf61697437da3cca7fb845f` |
| sheet-0016.jpg | 8 | 5520–5700 | 92.000–95.000 | `448e367b395cac4e7162869067d11301da58afca16efad533381d5fa5226aac7` |
| sheet-0017.jpg | 8 | 5760–6180 | 96.000–103.000 | `efcb2decbb44f816a2341334845fc6a04aa38878915e7229ebf1a362fb5c7be9` |
| sheet-0018.jpg | 8 | 6240–6584 | 104.000–109.733 | `901edda084cce7e57cd356b69b338774f22fbdc74a689ef5e0e24f5a31b55a02` |
| sheet-0019.jpg | 8 | 6600–6900 | 110.000–115.000 | `f7d7e9f076c16ad4dcc74e3dccecfb25d837f6951a9c9f321d0934d1bce6d017` |
| sheet-0020.jpg | 8 | 6960–7380 | 116.000–123.000 | `c923fe1f75a050d7b27ca332843b6b2db2d71cbc69e01df30c7b47b0bfb65906` |
| sheet-0021.jpg | 8 | 7392–7620 | 123.200–127.000 | `f3e710ffa61b8ce52b30cd03dfd9f526d71bd329e9391736b2c525a8524dab05` |
| sheet-0022.jpg | 8 | 7680–8100 | 128.000–135.000 | `24321b25a2e0bdf248fcd270b0217a3296e468d877688e1bccb831bbfc5cab93` |
| sheet-0023.jpg | 8 | 8129–8340 | 135.483–139.000 | `3cddccc7089fe0b1312ec37d0c1d9be8d8b238971fe64d629946f5ee65aeb2e6` |
| sheet-0024.jpg | 8 | 8400–8820 | 140.000–147.000 | `56e6c3c476e28de4b08203289baf7f5e26cc50247acc8fd7c3aad078151e526a` |
| sheet-0025.jpg | 8 | 8880–9300 | 148.000–155.000 | `3f21f8fea24e8f8d252a249cc8daf265a2b9c08e80c653c2ac17c04bbcdfb572` |
| sheet-0026.jpg | 8 | 9346–9540 | 155.767–159.000 | `f79b443e68e41c61e59b8ee8737fba68d2ebcea193338cd27c2ba038a6641f0e` |
| sheet-0027.jpg | 8 | 9600–10020 | 160.000–167.000 | `ee1509bde6cd3533f78332f561d7d27e353a000ab58fd02305f0ab8dd5244b96` |
| sheet-0028.jpg | 8 | 10080–10500 | 168.000–175.000 | `59ac17332a6dab41cbe518e1286659792706bc65801357c6e3d7c42ac74864ae` |
| sheet-0029.jpg | 3 | 10560–10640 | 176.000–177.333 | `a0345c51d684b37d451670f2c9272ee948716e303c57844afc72695cb42d9eb0` |

## Verdict

- **No release-blocking defect in the sampled stills.**
- **Labels and residues are right.** Every mathematical label, residue and on-screen number I checked is correct.
- **Clean layout.** Nothing is clipped, and no text overlap persists.
- **Quiz holds.** The answer stays hidden through the think time in every sampled frame.
- **Four Low findings.** Three are cosmetic and one is advisory. None needs a re-render for correctness.

## Findings (all Low, none blocking)

**L1. A transient overlap during the radius sweep, at 37.000 s (frame 2220; native crop viewed).**
- The moving `R` label on the radius crosses the imaginary axis, and its dark backing cuts a short gap in the axis.
- The radius line passes within a few pixels of the pole label `i`, and the label's backing interrupts the line.
- At 36.450 s and 38.000 s the label is clear of the axis. Only one sampled frame shows the contact, so how long it lasts is unmeasured.
- Cosmetic. The fix would be to offset the radius label to the side away from the axis or to dim the pole label during the sweep.

**L2. During fades, the axis and unit circle show through the "precisely:" note.**
- This happens in the fade-in at 157.767 s (frame 9466) and the closing fade at 177.000 s (frame 10620). Native crops viewed.
- While the note box is partly transparent, the imaginary axis and the grey unit circle run through the words "|f| over".
- In every fully opaque sample, from 158.000 s to 176.000 s, the box hides them cleanly.
- Transient. The fade durations are not measurable from stills.

**L3. Advisory: the note box sits on the bottom edge, where YouTube draws captions and controls.**
- From 157.767 s to the end, the box spans x 106–3733 and roughly y 1731–2107 of 2160. Ink reaches 53 px from the bottom edge. The sample at 169.000 s already shows three lines of text.
- Nothing is clipped in the frame.
- On YouTube, captions and the control bar draw in this band, so this text may be covered whenever they show. I did not measure player geometry.
- Root's call. Possible fixes are to raise the box, or to accept it because the note repeats the narration.

**L4. Cosmetic: typography is inconsistent.**
- The pole labels `i` and `−i` are set upright in the text face, as are the readouts `R = 2.00`, `R = 10` and `R = 2:  0.4189` and the note's `R` and `f`.
- The formulas and axis labels use italic math `i`, `R` and `f`.
- The meaning is unaffected.

**Near misses I checked and accept.**
- **`p₁ + p₂` label (frames 6720 and 7080, native).** It sits flush against the pink arrowhead of the `p₂` step but does not overlap it. It is legible.
- **`i` and `−i` at R = 10 (frames 3120 and 3180, native).** They sit close to the real axis but are clear of it and of the arrowhead.

## What holds (stills and arithmetic only)

**First example: 1/(1+x²).**
- **Bump and formula.** The bump appears at 2.6 s. "∫ dx/(1+x²) = arctan x |₋∞^∞ = π" is legible from about 9 s and dims at about 14 s.
- **Contour.** The contour is the segment −R→R plus the counter-clockwise upper semicircle, with an arrow on each piece.
- **Poles.** The pole at i is inside and highlighted. The pole at −i sits outside and is dimmed.
- **Residue.** "2πi · 1/(2i) = π ≈ 3.14159" is correct.
- **Arc bound.**
  - "|z²| = R² ⇒ |1/(1+z²)| ≤ 1/(R²−1)" and "|∫_arc| ≤ πR·1/(R²−1) = πR/(R²−1)" are correct.
  - The plotted size along the arc peaks at θ = π/2 and touches the 1/(R²−1) line, which is right for R = 2.
- **Readouts.** All recomputed:

  | R | πR/(R²−1) |
  |---|---|
  | 2.00 | 2.094 |
  | 4.07 | 0.821 |
  | 6.52 | 0.493 |
  | 10 | 0.317 |

- **Rescaling.** The caption "rescaled as R grows … i and −i close in" discloses the change of scale.

**Second example: 1/(1+x⁴).**
- **Antiderivative.** The antiderivative at 60 s is legible in the native crop. I checked it numerically: its derivative matches f, and F(200) − F(−200) = 2.2214414.
- **Roots.** The roots sit at 45°, 135°, 225° and 315° on the unit circle.
- **Quiz.**
  - "which poles are inside?" is up from 79 s, and the ring runs from 82.117 s to about 87 s.
  - All four roots stay the same grey through 87.867 s.
  - p₁ and p₂ are highlighted at 90–91 s, after the think time.
- **Residue chain.** These lines are correct:
  - "h = 1+z⁴ : Res_p = 1/h′(p) = 1/(4p³)";
  - "p⁴ = −1 ⇒ 1/p³ = −p";
  - "Res_p = −p/4";
  - "Im p = √2/2".
- **Vector picture.** The vectors are to scale: p₁, then p₂ tip to tail, and p₁+p₂ = i√2. In the native crop the unit is 280 px and the yellow vector measures about 395 px. The "sum" arrow points down and is about 0.354 units long.
- **Sum and value.**
  - "Res_{p₁}+Res_{p₂} = −(p₁+p₂)/4 = −i√2/4 ≈ −0.35355 i" is correct.
  - So is "2πi·(−i√2/4) = π√2/2 = π/√2 ≈ 2.22", with π/√2 = 2.2214415.
- **Arc bound.** "|1/(1+z⁴)| ≤ 1/(R⁴−1)" and "|∫_arc| ≤ πR/(R⁴−1)" are correct. R = 2 gives 0.4189. R = 10 gives 0.0031, which is below 1/300 = 0.00333.
- **Numerical check.** "a numerical sum on [−200, 200]: 2.2214" is correct. Simpson's rule gives 2.2214414.
- **Closing note.** The note is mathematically sound. It says absolute integrability makes the improper integral the limit over [−R, R], and that for slowly decaying f the symmetric limit can exist when the integral does not.

**Layout.**
- **Right margin.** The rightmost ink at 147 s is at x = 3689 of 3840, from "R = 10:  0.0031 < 1/300". The note box's right edge is at x = 3733. Nothing is clipped.

**Transitions at every cue start (stills only).** Each one has clean before and after states:
- **13_3:** the arctan line leaves, and the result moves up.
- **13_4:** a full wipe to the bare loop and baseline from 55.0 to 56.6 s, then the new bump at 57 s.
- **13_5:** the antiderivative dims and then leaves.
- **13_6:** the ring and the question leave by 87.867 s.
- **13_7:** the residue rule appears.
- **13_8:** the derivation collapses to Res_p = −p/4.
- **13_9:** the sum chain is replaced by 2πi(…).
- **13_10:** the result is boxed and moves up, and the arc is highlighted.
- **13_11:** the note.
- **End:** the fade begins after 176 s, and the last frame is black.

**Observations for the listening reviewer.** These are not defects.
- The picture is blank from 0 to 2.0 s, although 13_1 starts at 0.6 s.
- The closing fade starts after 176 s.
- Both look authored. Their timing against the voice needs listening.

## Not done

- No listening, fine sync or motion review between samples.
- No captions or player-overlay measurement.
- No independent final audiovisual acceptance, and no master or publication step.
- These stills say nothing about the joined master.
