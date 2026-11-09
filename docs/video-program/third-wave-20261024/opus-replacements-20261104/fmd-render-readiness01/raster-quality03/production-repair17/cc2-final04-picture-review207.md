# Final 04 picture review 207

RubyRaven, for root, under `mx-video-program-program-87do.39`. This is an independent review of the selected final 04 (`04_reading_color`) stills.

This review covers stills and source/technical evidence only. I viewed every contact sheet as an image and then cropped and viewed native 3840×2160 frames for every suspect detail. I ran read-only Python checks on hashes, geometry and the colour field.

I did no listening and make no claim about motion between samples, sync, the master join or the final audiovisual result. The soundtrack's voice is ElevenLabs Frederick Surrey according to root; I did not hear it or check it. I ran no job, extraction, render, synthesis or build. I changed no file except this one.

## Job

**Folder:** `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-final04-pictures198`.
- Root's `final04-terminal198/pictures-reviewed198.json` does not name a folder. It names movie `6ba1ebd0…` and review 198.
- This folder is the only final04 picture folder in `five-film-release-20261027`. Its `sampling.json` names the same movie hash, `6ba1ebd07371…`, and its receipt binds `final04-pictures198-inputs.json`. So it is the extraction root reviewed. I started no other extraction.

**Receipt:**
- The helper is `final-chapter-pictures142.py` (`a0430c030b48e9defdbaa50b22494cb7f96a763ce9ec1419233dff752edd8dc5`). The command uses absolute paths, `--source final04-terminal198` and `--chapter 04_reading_color`.
- The guard admitted the job at 03:17:58 UTC on 2026-11-08. It finished at 03:18:54 UTC with exit 0, and there was no resource stop.
- The inputs hash was `3d80ca907e526f74d7931b4356fb1d8b811c55cc01f7b6bda10f26f5ea413aea` at both the start and the end. This equals the sha256 of `final04-pictures198-inputs.json`.

| Job file | sha256 |
|---|---|
| `resource-receipt.json` | `7b8203d381fe6b5ba6e06105dd318a49eb373c5d5a6ad7903c52469d17074831` |
| `sampling.json` | `b71f58f1f94c0e0df7bdffa312f1d4d49b6a7f4e9d5b5237622af06d63bc100b` |
| `select.filter` (1,877 bytes) | `db3c273b5bc94894377e7f1304a1d3aabd9a06c2dfdf4c98a0c163db00ce5929` |
| `command.log` (0 bytes) | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

## Inputs (`final04-terminal198`)

| File | sha256 |
|---|---|
| `04_reading_color.mp4` (6,774 frames, 112.9 s at 60 fps) | `6ba1ebd07371e583052a884f6f1e4050c5037b3a50ff71857e4b255ecd7cf0d6` |
| `04_reading_color.timeline.tsv` | `9839aadb5beba3ceaf99d406393f81b73d0113feeaec51cf11cc1ad6169bf095` |
| `04_reading_color.cues.tsv` | `7581709f8c6972f6bf3c099cf05e25fc37d28dcccfd327e7bd6e24580b360a4c` |
| `04_reading_color.cuelog.tsv` | `819bafeb26e6e0ebca578a7113b0b345cb249d6ba2c8d74142145181fd3d07a3` |
| `04_reading_color.layout.tsv` (empty: clean lint) | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `soundtrack_0.wav` | `c2a35bbce9cd8797d78c0bf83b0a060881b6314258fef66e5d8d8e0556589369` |

- These six files match `render-summary.json` (`4084934052df…`): binary `f4836eeeb4f2…`, permit `ae1433d8c0ac…`, x264 CRF 16.
- The folder also holds the render's `resource-receipt.json` (`b5e844b4…`) and root's `pictures-reviewed198.json` (`f707dbd4…`).
- I read that file's summary fields only after my own pass through the sheets, and nothing below depends on it. Its observations agree with mine, except that it records no overlap. L1 and L2 below are overlaps that may well be intended.

**Timeline:**

| Row | Frame | Time |
|---|---|---|
| line 04_1 | 36 | 0.600 s |
| line 04_2 | 1311 | 21.850 s |
| line 04_3 | 2264 | 37.733 s |
| line 04_4 | 2958 | 49.300 s |
| line 04_5 | 4332 | 72.200 s |
| line 04_6 | 4824 | 80.400 s |
| mark predict | 5489 | 91.483 s |
| line 04_7 | 5804 | 96.733 s |
| breath | 6594 | 109.9 s (2 s) |

The timeline also has `listen 5489 5`.

## Coverage

- **Samples:** 146 native JPEGs at 3840×2160, frames 0 to 6773, all unique, with no gap longer than 1.0 s:
  - 113 one-second samples;
  - 28 cue samples (7 lines × 4, at +0, +30, +60 and +120 frames);
  - 4 samples at `mark predict` (frames 5489, 5519, 5549 and 5609);
  - the last frame.
- **Hashes:** I re-hashed every picture and every sheet against `sampling.json`. There were 0 mismatches.
- **Sheets:** I viewed all 19 sheets as images, 18 × 8 tiles plus 2 on the last sheet, which is all 146 samples.
- **Native crops viewed:** 28 crops from 23 frames:
  - frames 36, 1740, 2160, 2460, 2700, 3018, 3120, 3420, 3900 (two crops), 4320, 4332, 4362, 4560 (title), 4740, 4800 (two crops), 5100 (two crops), 5924 (two crops), 5940, 6180 (two crops), 6420, 6480, 6720 and 6773;
  - the zoomed crops are ×3 on 5100 and 4800.
- **Measurements:**
  - plot and panel bounds on frames 3840, 3900, 4320, 4332, 4362, 6360, 6420, 6480, 6720 and 6773;
  - panel opacity from matched pixels, 3840 against 3900 and 6360 against 6480;
  - phase-line steps on rings and band edges on rays, on frames 2160 (z), 3600 (z²) and 4560 (1/z).

| Sheet | Tiles | Frames | Seconds | sha256 |
|---|---|---|---|---|
| sheet-0001.jpg | 8 | 0–180 | 0.000–3.000 | `6a369b28ba79e44f08e607e652448ade090bf7a43ad736b102b48c2655ae4d83` |
| sheet-0002.jpg | 8 | 240–660 | 4.000–11.000 | `9412d33c76c543010b6a59709f326a6afe04aa3da4487030556218288dfc4147` |
| sheet-0003.jpg | 8 | 720–1140 | 12.000–19.000 | `e181707ca41700eddb56a65b8c91faa42968cafc88e7023a78e6a8eada2f05dd` |
| sheet-0004.jpg | 8 | 1200–1431 | 20.000–23.850 | `bc4b34374bc9b5243971720b8e4f6c023d4c16d23d27df7e2fd37d4a233be0e3` |
| sheet-0005.jpg | 8 | 1440–1860 | 24.000–31.000 | `f6b473fcd14b48fda430a4251ca1c40364ba809a4558f179493459815cf671c2` |
| sheet-0006.jpg | 8 | 1920–2280 | 32.000–38.000 | `b937fb9f69dad8facb9788f54855048a9efb274c785e8d2328aa1b0c68ba0831` |
| sheet-0007.jpg | 8 | 2294–2580 | 38.233–43.000 | `31a39315051a91f5dc422420ba8911666fcae2d1369ea3b6206475a5e7daa61d` |
| sheet-0008.jpg | 8 | 2640–2988 | 44.000–49.800 | `e194ac8d150dc43f89f794a1a32388bbaefea59dd2736a01dfd653e5f930d1ce` |
| sheet-0009.jpg | 8 | 3000–3300 | 50.000–55.000 | `f4b45d5a7965fca1c57679ead4434346c4889f9480c02d96846544ee1cde6c65` |
| sheet-0010.jpg | 8 | 3360–3780 | 56.000–63.000 | `e90a1107ce45ef4390305f54d92d1ffb407384d8320f3bfde302d0f02c7cd30a` |
| sheet-0011.jpg | 8 | 3840–4260 | 64.000–71.000 | `c37db356bf703ef4503daa08f08f3cd76a9930ec7894e3e395963f39d7bf0fe8` |
| sheet-0012.jpg | 8 | 4320–4500 | 72.000–75.000 | `9600fd3b1c6c7237eb239fe505f2cb8804f5349eda61b40c5f847abbbe706e8a` |
| sheet-0013.jpg | 8 | 4560–4860 | 76.000–81.000 | `ff8c93bfdc6c61e0f19480d9d293a30f62b9cf9c2a710773c642f4f27209fdb4` |
| sheet-0014.jpg | 8 | 4884–5220 | 81.400–87.000 | `ead65ab877e4365e79fd479e386d2248ff7bcd1e63207ed5b4bddd459c179054` |
| sheet-0015.jpg | 8 | 5280–5549 | 88.000–92.483 | `a681c5cb635ca437f8b7b7bdb6770d3ab05a773a5f5ac3fc01985ff04a5f1a79` |
| sheet-0016.jpg | 8 | 5580–5834 | 93.000–97.233 | `c32d21d57148507b7838edcf3a70c327bc34436e10df7706c68164dba343a243` |
| sheet-0017.jpg | 8 | 5864–6180 | 97.733–103.000 | `6423581295c2e190727aeeb0cc436410a3dcc7ba8a934bf2cccd58edc9234c42` |
| sheet-0018.jpg | 8 | 6240–6660 | 104.000–111.000 | `19ea0d0ad78b88ff2a7d392a1fff48fbc602f76346b1e61877742fed61b573db` |
| sheet-0019.jpg | 2 | 6720–6773 | 112.000–112.883 | `8b87a90ac7f97740886af0de43a2011212ebd96cbe8c9cb7a2358f6f12ebbc99` |

## Verdict

On the selected stills, I found no wrong mathematics, wrong label or wrong colour. I found no clipped text and no frame-edge overflow, and the quiz answer is not shown early.

I found four Low findings. None changes what the chapter teaches. None of them justifies a re-render on its own; that call is root's.

## Findings (all Low)

**L1. The opaque "precisely" panel covers the bottom 181 px of the plot.** Seen from 65.0 s to at least 72.2 s; samples 3900 to 4332, sheets 11–12.
- Panel bounds: x 105–3734, y 1828–2108. It is opaque: matched pixels give alpha 1.00.
- Without the panel, the plot's field ends at y 2008 (frames 3840 and 4362). With it, the visible field ends at y 1827. So rows 1828–2008 are hidden: 181 px, about 10.5% of the plot's height. This includes its bottom border.
- Only the outer z² field is hidden. The loop (bottom near y 1540), the zero marker and the "zero of order 2" label are all clear.
- The sentence is fully legible and correct: "near a zero z = a of order m, f(z) is about c times (z − a) to the power m, so the hue turns m times as z circles a".
- The panel is absent at 64.0 s and gone by 72.7 s. That gives at least 7.2 s and under 8.7 s on screen.
- Likely intended as a footnote band. I report it only because the plot box is visibly cut.

**L2. The translucent "names to search" panel overlaps the bottom 84 px of the plot.** Seen from 107.0 s to 112.0 s; samples 6420 to 6720, sheets 18–19.
- Panel bounds: x 105–3734, y 1925–2108.
- Over the plot, matched pixels against frame 6360 give alpha 0.81–0.87, so the field shows through at about 16%. Rows 1925–2008 of the field are dimmed, not hidden.
- The text "names to search: domain colouring · enhanced phase portrait" begins about 30 px above the plot's bottom edge and stays fully legible.
- The −i pole marker (y ≈ 1578) and the gold circle are clear of it.

**L3. The tour dot merges with the loop's arrowheads in 3 samples.**
- The tracker dot and the three direction arrowheads are the same near-white.
- Affected samples:
  - 50.3 s (frame 3018, counter +0.3, z²): the dot is fully under the upper-right arrowhead (about 54°);
  - 52.0 s (frame 3120, counter +1.0): only a small bump shows at the left arrowhead;
  - 79.0 s (frame 4740, counter −0.8, 1/z): only a small bump shows at the lower-right arrowhead.
- Each placement agrees with its counter. For z², the dot has gone half as many turns as the counter shows. For 1/z, it has gone as many turns as the counter shows, with the sign reversed.
- The counter and the wheel needle still report the tour, so nothing is lost. This is the same class as the transient contacts root accepted in disposition 206.

**L4. Type-style differences.** Same class as final13 L4.
- The text-mode minus in "colour cycles around 0: −1" and in "at i: −1 turn, at −i: −1 turn" is shorter and heavier than the counter's math minus in "−1.0" (×3 crop of frame 4800). The `×` in "size × 1.69" and the minus in "(z − a)" are similarly heavy.
- The apostrophe in "where are the bump's poles?" is a straight typewriter tick, not a curly one (×3 crop of frame 5100).
- `z`, `a`, `m`, `i` and `−i` in the text labels are upright, while the titles set `f(z)` in math italic.
- All are readable, and none changes meaning.

## Carried open item (not checked here)

The same open item as final13 L3 applies here:
- Both bottom panels sit in the bottom 15% (L1, from y 1828) and 11% (L2, from y 1925) of the frame, where uploaded captions and player controls appear.
- Whether captions or controls cover them on the platform is not visible in source stills.
- It stays with root's open platform caption-overlay check.

## Near misses and observations

- **Opening:** frames 0 and 36 (0.6 s, the first cue frame) are blank background. The colour wheel is the first content, at 1.0 s. This was not seen as a defect; whether it reads as dead air needs the cut.
- **Last frame (112.883 s):** it shows the bare bump field. The title, wheel, labels, markers, circle, axes, panel and plot border are all removed, and the field bottom is at y 2011. This matters only for the 04→05 join in the master, which I did not check.
- **Line 04_5 at 72.2 s:** frame 4332 still shows the whole z² scene (title, "zero of order 2", counter label, note panel). By 72.7 s (frame 4362), all of these are cleared, leaving the field, loop and legend. The 1/z crossfade follows at 73.2 s.
- **The z→z² crossfade at 38.733 s:** it shows both fields superimposed, which is an expected dissolve. The title changes at 39.733 s.
- **Counter "+0.0" at 76 s (1/z):** the dot is about 12° into the tour, a true value of about −0.03. The sign-free rounding is acceptable.
- **The phase-line step detector:** it found steps on only part of each ring, about 6 of 12 at r = 250–650 px on f(z) = z and 22 of 24 on z². The misses fall where the hue's own brightness masks the step. Every detected spacing is a multiple of the expected angle (below).
- **Wording:** the corpus copy `explainer_corpus/complex_analysis_visual/src/chapters/reading_color.rs` (sha `dc98bcee…`, last commit 2fb1c0c7) reads "near a zero a of order m". The rendered frames read "near a zero z = a of order m". So that corpus copy is not the render source. I compared the pictures with the narration lines of `explainer_corpus/complex_analysis_visual/script.tsv` (sha `2968b0fd…`) and with the mathematics, not with chapter source.

## What holds

- **Hue convention.** The wheel, the legend ("right: red", "up: yellow green", "left: teal", "down: violet", from 14–20 s), the grey "negative" and "positive" labels and the f(z) = z field all agree with line 04_1.
- **Twelve phase lines per output turn.**
  - On f(z) = z (frame 2160), ring steps fall every 30.0° (±0.7°) at radii 250, 450 and 650 px.
  - On z² (frame 3600), every 15.0° (±0.3° at r = 550 and 700 px): 24 lines in the input, each output twelfth twice.
  - On 1/z (frame 4560, r = 700), every 30.0° (±0.4°).
  - The step direction reverses for 1/z (rising, not falling, counter-clockwise), as the backwards cycle requires.
- **Band ratio "size × 1.69" = exp(τ/12) = 1.6881.** The ratios are between successive band edges on two rays per frame, excluding the innermost edges and the loop stroke.

  | Field | Ratios | Mean | Expected |
  |---|---|---|---|
  | z | 1.662, 1.675, 1.685, 1.710, 1.692, 1.699 | 1.687 | 1.688 |
  | z² | 1.291, 1.293, 1.294, 1.297, 1.295, 1.305, 1.294, 1.297, 1.297, 1.295, 1.304 | 1.297 | √1.6881 = 1.299 |
  | 1/z | 1.671, 1.680, 1.671, 1.690, 1.679, 1.695 | 1.681 | 1.688 |

  - The band step falls outward for z and rises outward for 1/z, as |1/z| = 1/r requires.
  - Edges used: steps in the band direction only.
    - z and 1/z: from about 65 px out, on rays at 15° and 195° (z) and at 15° and 105° (1/z).
    - z²: from 100 px out, on rays at 7.5° and 97.5°.
  - Excluded edges: the loop stroke's jumps. Across the loop, one edge is hidden and the pair spans two bands: 487/288.5 = 1.688.
  - On z, the next edge (about 550 px) is below the detector threshold.
  - The ratios have about 1% pixel quantisation.
- **Squaring (04_3, 04_4).**
  - The z² field has red right and left, teal top and bottom, in wheel order.
  - "colour cycles around 0: 2" is shown from 41 s.
  - The counter runs up to +2.0 on each of two tours (45–49.3 s and 49.8–55 s).
  - "zero of order 2" sits inside the loop, clear of the marker and the loop (frame 3420).
- **1/z (04_5).**
  - Violet top, yellow-green bottom, teal left, red right.
  - The counter runs to "−1.0" with the needle back on red.
  - "colour cycles around 0: −1" and "simple pole" are shown.
- **Quiz (04_6, 04_7).**
  - The box reads "where are the bump's poles? how many turns at each?" from 85 s.
  - The plot is dark, with only the faint circle and axes, from 82 s through 96.733 s. No pole, marker or turn count appears before line 04_7.
  - The think window runs from `mark predict` (91.483 s) for 5 s, to 96.483 s.
  - The colours fade in at 97.233 s, the pole markers appear at 98.733 s and "at i: −1 turn, at −i: −1 turn" at 99 s.
- **Bump field 1/(1+z²).** I checked it analytically against crop 5924:
  - red on the real axis and on the imaginary axis for |y| < 1, teal beyond ±i;
  - around i: teal above, yellow-green left, red below, violet right, which is −1 turn;
  - around −i: teal below, yellow-green right, violet left, which is −1 turn;
  - the top-edge colours at ±2 + 2i are yellow-green and violet, as computed.
- **Poles on the unit circle.** The pole centres are 432.5 px from the circle centre, against a circle radius of 434.5 px, so within 2 px. The circle of radius one touches both poles, as 04_7 says.
- **Gold segment.** The gold diameter runs from −1 to 1 and ends on the gold circle (frame 6180). "gold segment: where the recipe works" is legible.
- **Titles.** I viewed `f(z) = z`, `f(z) = z²`, `f(z) = 1/z` and `f(z) = 1/(1+z²)` at native size. All are correctly typeset and clear of the plot.
- **No layout lint issues.** The layout log is empty, and no text touches a frame edge in any sample.

## Not done

- No listening, of either the ElevenLabs Frederick Surrey narration or the soundtrack mix.
- No claim about motion between samples or fine sync.
- No check of the master or chapter joins, captions, or platform overlay.
- No independent final audiovisual acceptance and no publication acceptance.
- No job, extraction, render, synthesis or build.
- No edit to source, helpers, shared files, TODO or Beads. No commit.
