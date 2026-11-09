# cc2 preview08 review 271: chapter 08, loop sums (native 540p preview243)

## Scope

This review covers sampled 540p stills only. I looked at sheets, crops and script measurements of extracted pictures.

It is not:
- a motion review;
- a listening, sync or fine-sync review;
- a 4K review;
- final AV acceptance.

The voice is ElevenLabs Frederick Surrey, according to root. I did not hear it.

I ran no build, render, extraction, synthesis or other heavy job. I changed no source or test file and no Beads state. The only file I wrote is this one. The frozen files (source/tests 264, 260, 246) are untouched. Paths are relative to this folder.

## Subject and gate

- **Gate.** `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-preview243-copy265/resource-receipt.json` shows:
  - finished 2026-11-08T05:36:17.721810Z, exit 0, admitted;
  - `inputs_sha256` = `final_inputs_sha256` = `aa5d96a7b0e05c69d47624fdedf9a877d5cd670c69c9fa2d0cc491d1b3bf1ac9`;
  - no stop recorded.
- **Film.** `preview243-review265/preview/08_loop_sums.mp4`:
  - sha `1ac1ee716f6aa383a9154d6305ffa571bd555df818565d28a07084eb4b5364d6`, 9,384,754 bytes. This equals `movie_sha256` in the sampling file.
  - 960×540 at 60 fps. Frames 0–14958, so 14,959 frames, 249.3 s, by the sampling. That matches final08 in review219.
- **Tables.**
  - The cue table `1beae064…` and the layout table `7be1c4b1…` are byte-identical to final08's, as listed in review219.
  - Cuelog `6f0c1208…` has the same two lint rows at 205.6 s.
  - Timeline `9d6425d6…`.
- **Extraction.** `preview243-review265/08_loop_sums/sampling.json` (`951eab4a4c64517a5cff9b8ff824f79c83cca4d5b1f614a6157498383d48a7da`), with `select.filter` (`445d7121903c1d36950242fa167ebf98c44630222ad8eba2705e439d26c09bb6`):
  - 552 pictures. I re-hashed them all: 0 mismatches.
  - The largest gap between samples is 60 frames. There are 69 sheets.
  - **Window.** Frames 10200–10380 (170.000–173.000 s) are all present: 181 consecutive frames, samples 277–457.
- **Source.** `loop-sums-review-repair238.rs` (`8a13e8a190cd502e7b58efbcb7c40872cdb0e6e702feb69e1b789e451a7a42b5`). The pictures match it at every point I compared:
  - the caption text (`:610`);
  - the walk and its two 0.3 s fade-outs (`:645–650`);
  - the return with P_n (`:653–654`);
  - the cancellation strokes (`:715–718`);
  - the triangle fades (`:784–796`).

  I did not re-derive the build binding or the narration binding.
- **Independence.**
  - This review is independent of root's build, render and extraction.
  - I did not open root's preview243 review records (`preview243-root-review-progress272.json` and `-complete277.json`).
  - I read review219 only after finishing the sheet pass, to map its items.

## What I viewed

- **Sheets.** All 69 sheets, `sheet-0001.jpg` to `sheet-0069.jpg`, each read as an image. Together they hold all 552 sampled pictures; sheet k holds samples 8k−7 to 8k.
- **The 181-frame window.** Every frame from 10200 to 10380 was read on the sheets:
  - sheet 35 (10200–10203);
  - sheets 36–57 (10204–10379, 8 frames each);
  - sheet 58 (10380).
- **Enlarged crops** (nearest neighbour, 2–6×) of 10 frames, read as images:
  - 10200 (glyph weights);
  - 10740, 10800, 10860, 11220 (Dₙ against the half-size square);
  - 11280 (≥);
  - 12324, 12336, 12420 (cancellation strokes);
  - 14700 (footnote).
- **Script measurements**, without reading the frames as images:
  - **Window levels.** Ink levels over all 181 window frames for four labels. Each label uses a fixed mask: pixels lit in the reference frame, taken as 10200, or 10380 for Pₙ. The masks are:
    - caption, 525 px;
    - c, 28 px;
    - Dₙ, 85 px;
    - Pₙ, 88 px.
  - **Colours.** Mean RGB of the lit pixels for c, Dₙ, Pₙ, the dot and the caption at 13 frames between 9840 and 10620.
  - **Ink counts.** c's ink count on the holds before and after the walk.
  - **Where the scripts went.** The crops and scripts were temporary files outside the repo.

## Result

**0 High, 0 Medium, 4 Low, 6 notes.**
- **Two named checks.** Both pass on stills: the plain caption, and the c/Dₙ fades at 170–173 s.
- **Review219 M1** (literal `Q_n`) is fixed. Review219 L3 is cleared on all 181 window frames.
- **One new Low, R1:** a stale middle-walk caption on the triangle.
- **Three Lows carried from review219** are still present: L1, L2 and L4.
- **Lint.** Both lint rows are the intended cancellation strokes, so 0 lint defects.

| Item | Severity | Status in preview243 |
|---|---|---|
| review219 M1, literal `Q_n` in the zoom caption | was Medium | **fixed**: reads "Selected square, magnified", steady (below) |
| review219 L3, segment predicted to cross the zoom c label near 171.7 s | was Low (prediction) | **cleared on stills**: c and Dₙ are gone before the segment reaches their spots (below) |
| R1, "The middle walk is also counterclockwise" outlives the middle walk, 231.0–249.0 s | Low | **new** |
| review219 L1, heavier face for ≤, ≥, −, ∂ | Low | present |
| review219 L2, nested c label far from its point, 117–163 s | Low | present |
| review219 L4, right-edge midpoint dot half under the cyan Dₙ edge, 233–249 s | Low | present |
| N1, small-label level and colour steps on static holds | note (preview encode) | new; not a render defect |
| review219 O1, cancellation built from pieces; layout lint at 205.6 s | note | present; plus a timing remark (below) |
| review219 O3, text-only hold 63.5–74 s | note | present |
| review219 O4, Dₙ above the half-size square's corner, 180–204 s | note | present |
| review219 O5, "precisely:" lower third, 242–249 s | note | present; plus `f'` remark (below) |
| review219 O2, overlapping tip-to-tail arrows, 140–147 s | note | not re-examined at native size; the sheets show the walk closing |

## The two named checks

### Plain "Selected square, magnified" caption: pass on stills

- **Appearance.**
  - The caption fades in with the inset at 9810 (163.5 s) and is full from 9828.
  - It is on every sampled picture through 13032 (217.2 s) and is gone at 13080 (218.0 s).
  - It is plain text: no underscore and no `Q_n` (source `:610`).
- **Window.** Over all 181 frames its mask level stays at 1.000–1.001.
- **Colour.** Its mean colour is (85, 90, 95) at every frame measured from 10020 to 10620.
- **Placement.**
  - It sits above the inset's top edge, clear of the formula column.
  - Nothing crosses it in any sampled picture, including the cancellation (205–207.6 s) and the half-size square (from 180 s).

### c and Dₙ during the walk, 170.0–173.0 s: pass on stills

Measured mask levels, with 1 = the label as held at 170.000 s:

| Frames (s) | What the sheets show | c | Dₙ | Pₙ |
|---|---|---|---|---|
| 10200–10217 (170.000–170.283) | static: segment from the bottom-left corner to c, walker at the corner | 1.000 | 1.000 | 0 |
| 10218 (170.300) | walker leaves the corner along the bottom edge | 1.000 | 1.000 | 0 |
| 10220–10235 (170.333–170.583) | both labels fade while the walker is on the bottom edge | 0.995 → 0.000 | 1.008 → 0.003 | 0 |
| 10236–10337 (170.600–172.283) | labels absent; walker at the corners about every 30 frames (bottom-right ~10247, top-right 10277, top-left ~10307, bottom-left 10337) | ≤ 0.004 outside the crossings below | ≤ 0.014 outside the crossings below | 0 |
| 10279–10284 | segment sweeps the empty Dₙ spot (sheet 45) | 0 | up to 0.442 at 10283 (segment ink) | 0 |
| 10293–10319 | segment sweeps the empty c spot (sheets 47–50) | up to 0.757 at 10306 (segment ink) | 0.004 | 0 |
| 10338–10354 (172.300–172.567) | segment at rest on the lower half of the diagonal; c, Dₙ and Pₙ return together | 0.003 → 0.910 | 0.016 → 1.015 | 0.010 → 1.000 |
| 10355–10380 (172.583–173.000) | steady | 0.912–0.918 | 1.014–1.015 | 1.000 |

- **Leaving.**
  - c falls every frame from 10223 to 10235; Dₙ falls every frame from 10222 to 10235. The 1.002–1.008 values at 10220–10221 are within noise.
  - The fade takes about 0.25 s, inside the source's `rt(0.3)`.
  - At that time the walker is still on the bottom edge. The segment first reaches a label's spot 44 frames later (Dₙ's, at 10279).
  - The largest one-frame drop is 0.155.
- **Absence.**
  - The only non-zero readings while the labels are away are the green segment passing over their masks.
  - The sheets show these frames with the segment over empty background and no glyph.
- **Return.**
  - All three labels rise every frame from 10338 to 10354.
  - No label appears at full brightness before its ramp, and none jumps. The largest one-frame rise is 0.106.
  - The return starts the frame after the walker is back at the corner (10337). The segment is at rest throughout.
- **Caption.** Steady over the whole window (above).
- **The c end level of 0.91 is the preview encode, not opacity** (N1). On holds with no animation on c, its ink count changes from one hold to the next:
  - 4274 at 9840–9990;
  - 4926–4944 at 10020–10220;
  - 4507 at 10356–10440;
  - 4926 again from 10500.

  Its mean colour alternates between two states, (99, 109–110, 92) and (97–98, 122, 92), on the same holds. Pₙ (red 139 → 149) and Dₙ (blue 127 → 131) step at 10500 with them. c is a glyph of about 7×9 px at 540p.

**What the stills cannot show.** I have no evidence about smoothness in motion, about how the fades sit against the voice, or about 4K. At 60 fps every frame of the window was sampled, so the stills rule out a one-frame flash or overlap in this encode only.

## Findings

### R1 Low (new): "The middle walk is also counterclockwise" stays after the middle walk is gone, 231.0–249.0 s

- **On screen.**
  - The four child walks fade in from 225.6 s (13536).
  - The green caption and the middle triangle's fill appear at 13680 (228.0 s). The circulation mark follows at 229.0 s.
  - The child walks and the mark are gone at 13836 (230.6 s), and the child outlines by 13860 (231.0 s). Only the outer walk is left.
  - The caption stays at the top of the right column until everything fades at 14940 (249.0 s), as does the fill.
  - It stays through "The longest side plays the diagonal's part", the scale formulas, ∮∂T f dz = 0 and the "precisely:" box. That is about 18 s, the chapter's last stretch.
- **Source.** `:794–796` fade `triangle_walks`, `middle_turn` and `triangle_parts`. `middle` and `middle_name` (faded in at `:791`) are never faded before the chapter ends.
- **Why Low.** The sentence is still true and nothing overlaps it. But it describes arrows that are no longer drawn, and it heads the column above the newer result.
- **Smallest repair (root's choice).** Add `middle_name` to the fade at `:794–796`, and `middle` too if the fill should go. This adds no time. Keeping it as a deliberate recap is also defensible.
- **Not found in review219,** so this is new to this review.

### Carried Lows, confirmed on these stills

- **L1, type faces.** At 10200 (3×) and 11280 (3×), the following are visibly heavier than the hairline CM `+`, `=` and `<` in the same expressions:
  - ≤ in |r(z)| ≤ ε|z − c|;
  - ≥ in |Iₙ₊₁| ≥ |Iₙ|/4;
  - the minus in (z − c) and |z − c|;
  - ∂ in ∂Qₙ.

  The same ≤ (from 233 s) and ∂ (∂T, from 238 s) appear on the triangle. This is the same class as review221 L3, which records root's acceptance in disposition 206 for chapter 14. I did not check disposition 206 for chapter 08.
- **L2.** The nested c label still sits about 65 px at 540p (about 258 px at 4K) from its point at 116–119 s (sheet 20). It is unambiguous, because nothing else is near it.
- **L4.** On sheets 67–69 (233–249 s), the right-edge midpoint dot is half covered by the width-7 cyan Dₙ edge (`:806`). It is still readable.

## Notes

- **N1, preview encode.** See the window section. These are level and colour steps on small static labels between holds, with no animation. This is about the 540p preview encode only; I did not look at 4K.
- **O1, cancellation (205–207.6 s).**
  - **The lint.** The cuelog's two crossings at 205.6 s have exactly the source's stroke boxes: (2.1, 0.75)–(2.8, 1.25) and (4.15, 0.25)–(4.85, 0.75) at `:715–716`. They are the intended strokes, so the lint finds no defect.
  - **Readability.** At 3× (12324, 12336, 12420), the struck 4ⁿ stays readable, although the stroke is the same green as the glyph.
  - **Layout.** The fraction bar is about 182 px against a numerator of about 58 px at 540p, about 3.1×, with wide gaps. This matches review219's 4K figures.
  - **Timing.** The strokes start fading in right after the pieces' 0.4 s fade-in (`:717–718`). So the unstruck 4ⁿ is at full strength for only about a frame before the strike begins. Judge this in motion with the voice.
- **O3.** From 63.5 to 74 s only text is on screen (sheets 11–12). Whether the voice fills it is a listening question.
- **O4.** At 6× (10740–11220), Dₙ clears the half-size square's top edge by about 12 px and the diagonal by about 5 px at 540p. It sits directly above the inner square's corner, so it could be read as naming either diagonal. The right-hand formulas disambiguate.
- **O5.**
  - The "precisely:" box is on screen from 242 to 249 s. It wraps after "no continuity of", so the second line begins `f' is assumed`.
  - At 2× (14700) the prime is a straight text apostrophe, while the formulas use a typeset ′. Readable; cosmetic.
- **Other sheets.**
  - 08_1–08_4 (sheets 1–10) and the rest of 08_5 show no clipping, cut-off text, collision or wrong maths in any sampled picture.
  - The triangle's tinted lower edges from 221 s are the corner children's own colours (`:765–767`).
  - The last frame (14958) is black after the closing fade.

## Limits

- **Sampled stills only.**
  - Outside 170–173 s the samples are at most 60 frames (1 s) apart, so brief events between samples are unseen.
  - Nothing here accepts motion, timing against the voice, listening, fine sync, 4K or the final AV.
- **Pixels only for type faces.** I did not inspect fonts.
- **Mask method.** Mask levels measure ink under fixed masks. They cannot tell a label from other ink passing over its spot; I used the sheets for that.
