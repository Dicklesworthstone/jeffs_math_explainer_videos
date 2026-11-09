# CC1 gallery transition review 319

Reviewer: CC1, independent of root and CC0. Date: 2026-11-08.
Companion data: `cc1-gallery-transition-review319.json`, sha256 `58a7381dd8506497e10bd147d54837a6472e72651e3724e9dd99747f83e6e651`. It holds every sheet's hash and frames, every site's record and the measured opacity curves.

## Verdict

**No transition defect in the 715 frames: 0 High, 0 Medium, 0 Low.**

- In each of C1–C17, the authored object enters or leaves on its own.
- The vector fade before each site is complete before the plate moves.
- The titles and labels that follow start after their plate.
- I measured 22 opacity curves: one per site, with C13 split into its six panels. Every curve moves only inside its authored window and never reverses by more than 0.01.

This is a review of still frames only. It makes no claim about real-time motion, listening, narration timing, fine A/V sync, final AV acceptance or 4K (see Limits).

## What was viewed

All 60 sheets were read as images, in order, every tile matched to its frame label: 715 frames in all, 12 per sheet, with 7 on sheet 060.

| Window (video frames) | Frames | Sheets | Sites |
|---|---|---|---|
| 34–98 | 65 | 001–006 | C1 |
| 1989–2072 | 84 | 006–013 | C2, C3 |
| 2894–2977 | 84 | 013–020 | C4, C5 |
| 4772–4855 | 84 | 020–027 | C6, C7 |
| 6181–6264 | 84 | 027–034 | C8, C9 |
| 7312–7365 | 54 | 034–038 | C10 |
| 8715–8859 | 145 | 038–050 | C11, C12, C13 |
| 9168–9220 | 53 | 051–055 | C14, C15 |
| 9817–9878 | 62 | 055–060 | C16, C17 |
| **Total** | **715** | **60** | **17** |

673 of the 715 frames lie inside the authored windows of C1–C17. The rest are context before and after each window.

**Inputs, hash-checked:**
- `gallery-transition-frames305/manifest.json`: sha256 `5a98dd205a39f9b596f61e9c3dcc048b75c3aa9f1aac286f1872ab26acfe7d49`, 108,367 bytes.
- Its `source_sha256` is `f5123300084eba264276377f36518d263324bb9e28dd5df0f77ecf5f20cf5e48`. This equals the hash of `preview274-review305/preview/05_gallery.mp4` (960×540, 60 fps, 9,879 frames).
- All 715 PNG hashes match the manifest.
- The manifest names the sheets but gives no hashes for them, so the JSON records the hash of each sheet I viewed.

**Context read:**

| File | sha256 (prefix) | Used for |
|---|---|---|
| `gallery-review-repair249.rs` | `d57e003a` | expected content of each window |
| `cc3-preview-fade274.py` | `d80cf7b6` | the C1–C17 site list |
| `preview274-review305/05_gallery/sampling.json` | `b8011d90` | fade placement |
| `cc3-preview-results308.md` | `4b73ecfd` | the C10 and C14 notes |

## Per site

All measured frames below are inside the authored window, and none of the curves reverses.

| Site | Authored | Event | Measured movement | Seen on the sheets |
|---|---|---|---|---|
| C1 | 36–95 | z³−1 plate in | 39–85 | Plate alone. The title starts at 96, after the plate. |
| C2 | 1991–2020 | zdp plate out | 1996–2018 | Plate at full with no marks or labels. Background only from 2020. |
| C3 | 2021–2069 | e^z plate in | 2023–2060 | See the note below the table. |
| C4 | 2896–2925 | e^z plate out | 2900–2923 | No text from the start. Background at 2925–2926. |
| C5 | 2926–2974 | sine plate in | 2928–2964 | No border or title during the fade. Title from 2975. |
| C6 | 4774–4803 | tan plate out | 4779–4801 | Tan plate alone, no text. |
| C7 | 4804–4852 | Gamma plate in | 4807–4843 | Title from 4853. |
| C8 | 6183–6212 | Gamma plate out | 6187–6210 | No title, border or marks. |
| C9 | 6213–6261 | zeta plate in | 6215–6252 | Title from 6262. |
| C10 | 7314–7362 | critical strip in | 7316–7352 | See the note below the table. |
| C11 | 8717–8741 | zeta plate out | 8721–8738 | The strip stays at full. Title, marks and labels are already gone at 8715. |
| C12 | 8742–8766 | strip out | 8746–8763 | Strip alone. Same curve shape as C11, 25 frames later (R1). |
| C13 | 8767–8856 | six panels in | each panel inside its own 15-frame slot | Row-major order: z³−1, zdp, e^z, sin z, Gamma, zeta. No overlap between panels. No borders or names until after 8859. |
| C14 | 9170–9187 | five panels out | 9173–9185 | The five fade together with their names and marks. The cubic panel, its name, its marks and the gold box are untouched. |
| C15 | 9188–9217 | zoom plate in | 9189–9212 | The links and the zoom border begin at 9218–9220 (R2). |
| C16 | 9819–9848 | cubic panel out | 9824–9846 | 9817 holds the last remnants of the 0.5 s text fade. 9818 is clean. |
| C17 | 9849–9878 | zoom out | 9854–9876 | Background only at 9877–9878, the file's last frame. |

- **C3:** The title signal is measurable from 2066, but only at 0.25 code levels or less, which cannot be seen. It reaches 0.58 at 2069 and 5.85 at 2072. The plate already measures 0.98 or more from 2061, so there is no visible overlap.
- **C10:** At 7312 there are faint remnants of the 0.3 s label fade. The title, plate, marks and "zeros at −2, −4, −6, …" stay static throughout. The strip region is empty until 7314. The text fade-in that follows begins at 7364–7365. This window holds the actual `strip.show(0.8)`; the 274 chroma fit at 6875 was a different event.

## Candidates checked and refuted

- **R1 (C12): the strip looked late to leave.**
  - On sheets 039–041, the strip looks fully bright up to about 8751.
  - Measured opacity: 1.00 at 8742, 0.97 at 8746, 0.80 at 8751, 0.51 at 8756, 0.00 at 8765.
  - At each frame offset this is within 0.03 of the C11 zeta curve. That matches the source: `zeta.leave(0.4)`, then `strip.leave(0.4)`.
  - Not a defect.
- **R2 (C15): the gold links seemed to start at the panel edge.**
  - At 9218–9220 the links can't be seen between the gold box and the panel edge, so they seem to start at the edge.
  - To check, I decoded two later frames from the same hash-checked preview. This was decode only, no render.
  - As a control, frames 9168 and 9220 decoded identically to the manifest PNGs (max abs diff 0).
  - At 9240 and 9300 both links are drawn from the box corners across the picture.
  - Those two frames are outside the 715. Their hashes are in the JSON. A third decoded frame, 9500, was not viewed or used.
  - Not a defect.
- **R3 (C3): the title's brightness looked uneven from 2070 to 2071.**
  - Measured column excess is 0.58, 1.38, 2.00, 5.85 over 2069–2072, which is monotone.
  - Not a defect.

## Notes (not defects)

- **N1: blocky first and last frames.**
  - The first and last one to three frames of nearly every fade show coarse, blocky, low-amplitude texture, not a dimmed copy of the picture.
  - This is consistent with lossy encoding of the 960×540 preview at very low signal.
  - It can't be attributed to authored content, and these stills can't show whether a final master has it.
- **N2: a faint static column in C2.**
  - A column at x = 396, y 110–210 (at 640×360) sits about 0.76 code levels above the background in every frame from 1989 to 2000.
  - It is invisible at normal gain and does not change, so it is not animation.
- **N3: sin z marks touch the panel edges.**
  - The mosaic's sin z panel spans x −6.6 to 6.6, so the marks at ±2π sit 0.32 units (about 4.5 px at 640 wide) inside the panel. Their discs reach the panel's left and right edges but are not clipped.
  - This is a static composition point for the owner, outside transition timing.

## Method

- Every sheet was read as an image.
- Suspicious frames were cropped, enlarged and gain-lifted from the hash-checked PNGs.
- For the opacity curves:
  - For each site I masked the pixels that differ between a full frame and a blank frame, boxed where other objects share the frame.
  - Opacity per frame is a least-squares fit over that mask.
  - Each PNG's hash was asserted as it was read.
- The gallery source was read for the expected content of each window: 05_1–05_4, 05_6 and 05_7–05_9.
- The helpers are reviewer scratch and are not delivered.

## Limits

- **Stills only.** No real-time playback, and no judgement of motion, cadence or smoothness beyond the sampled frames.
- **No audio.** No listening, no narration timing, no fine A/V sync and no final AV acceptance. These gaps remain open.
- **No 4K or master claim,** and no pixel equivalence with any other build. The frames are a 960×540 preview scaled to 640×360, and the sheets are JPEG.
- **Authored windows are estimates.** They come from the 274 sampling and the cc3 site list. The measured curves are reported beside them.

## Not done

- No render, synthesis, build, launch or master.
- No edit to 314 or to any source, helper, TODO or Beads.
- No writes besides this file and its JSON.
