# Final08 picture review 219 (cc1)

Written 2026-11-08 at about 04:30 UTC by the cc1 pane. This is an independent, read-only picture review of the native 4K chapter `08_loop_sums` in `final08-terminal218`.

I read all 36 contact sheets, covering all 283 sampled pictures. I then took native crops and pixel measurements from the 4K picture JPEGs. I wrote only this file. Crops and measurement scripts stayed in my scratch space.

I did not render, compile, synthesize, extract frames from the MP4, edit source, or touch Beads, the TODO or git. I did no listening and no motion review.

Paths below are relative to `Q` (`docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17`) unless they start with `narration_lab/`.

## Verdict

- **The two named checks pass.**
  - The repaired c label clears the gold edges and the δ circle in every sampled picture from 117 to 163 s.
  - Both 205.6 s layout-lint crossings are the intended 4ⁿ cancellation strokes.
- **One Medium defect: M1.** The zoom caption shows a literal `Q_n` with an underscore from 164.0 to 217.0 s. I recommend fixing it before public release.
- **Four Low items: L1–L4.** One of them, L3, is a prediction about motion that the stills cannot show.
- No High findings. No clipping, cut-off text or wrong maths in any sampled picture.

Root's record `final08-pictures-viewed218.json` (04:13:47Z) also mentions the Q_n caption, calling it "readable but typographically literal" and not blocking. I logged it at sheet 24 and confirmed it in the source before I opened root's files. My severity is higher; see M1.

## Inputs (sha256)

| input | sha256 |
|---|---|
| `final08-terminal218/08_loop_sums.mp4` (14,959 frames, 249.3 s) | `5278d2daf0543c5bee035e346477b1772a6774bdd4f173a60e6c7c86572412ed` |
| `final08-terminal218/08_loop_sums.layout.tsv` | `7be1c4b12ad55f624040c3217f98587c89c3f94f07807c0f8e7f211d309aa951` |
| `final08-terminal218/08_loop_sums.cues.tsv` | `1beae064b4b1a8115a70180d92740299195fd6cc55b68cebcdbd97ace8e127d3` |
| `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-final08-pictures218/sampling.json` | `2ac153ad888584527fb064f30a9edbe2b3dae337d181d2be10393d2eb618e2d9` |
| the same folder's `select.filter` | `74d334cef23b0d96645f852e1f65d093be192c1759e17bff1e4dc5ba60b5d070` |
| `loop-sums-label-repair164.rs` (the chapter source the render used; see note) | `1cf259683b29606e51f0fd25ffb35e26247f7adff1f5d5aee8334b57aa6dedec` |

All 283 pictures and all 36 sheets hash to their `sampling.json` values; I rechecked them just before writing this. The movie hash matches `sampling.json`'s `movie_sha256`. Every crop below was taken from a picture whose hash I rechecked.

**Which source rendered.** The render's c label sits at offset (+0.95, +0.12). Only `loop-sums-label-repair164.rs:502` has that offset. `cc3-source-preservation-notes220.md` maps `loop_sums.rs` to this file for build164.

`production-source-preserved214/.../loop_sums.rs` differs from repair164 only at line 502, where it keeps the pre-repair offset (+0.28, +0.27). A chapter 08 build from that snapshot would bring back the old c-label position. Line numbers below are from repair164.

## Findings

### M1 Medium: literal `Q_n` in the zoom caption, 164.0–217.0 s

- **What is on screen.** The dim caption above the magnified square reads `Selected square Q_n, magnified`, with a visible underscore between Q and n.
- **When.** It appears in 60 consecutive sampled pictures:
  - first at 164.000 s (frame 9840, `pictures/frame-000185.jpg`);
  - last at 217.000 s (frame 13020, `frame-000244.jpg`);
  - not at 163.000 s (`000184`) or 218.000 s (`000245`).

  That is about 54 s across the zoom, "Two halves make one quarter", the 4ⁿ cancellation and I₀ = 0.
- **Evidence.**
  - Native crops at 164 s and 217 s (x 700–1750, y 330–490) show the underscore glyph.
  - The source, `loop-sums-label-repair164.rs:608–614`, passes this string to `kit.text`, which sets plain text, not TeX.
  - Every other subscript on screen in 08_5 goes through `kit.tex` and is set properly: Q₀ ⊃ Q₁, Dₙ, Pₙ, and the ∂Qₙ in Iₙ = ∮ r(z) dz, which is on screen at the same time.
- **Why Medium.** There is no maths error and it is readable. But it is a piece of source notation left on screen for nearly a minute in the film's longest proof, right beside a correctly typeset Qₙ.
- **Smallest repair (root's choice).** Either:
  - change the string to `Selected square, magnified`, or
  - set `Q_n` with `kit.tex` next to the plain text.

  Either way it needs a chapter 08 rebuild and a native re-render. I did not check whether the bundled text face has a subscript ₙ, so I would not use a Unicode subscript.

### L1 Low: five maths symbols are set in a heavier, upright face than the CM glyphs around them, throughout the chapter

- **Which symbols, and when.**

  | symbol | where | times (s) |
  |---|---|---|
  | ≈ | header (crop at 29 s) | through 62 |
  | minus sign | superscripts in z⁻³, z⁻², z⁻¹ | 51–62 |
  | minus sign | (z − c) in the Taylor line | 126–217 |
  | minus sign | \|z − c\| | 153–217 |
  | minus sign | the green formula | 139–147 |
  | ≥ | — | 101–123 and 186–189 |
  | ≤ | — | 105–108, 153–217 and 233–249 |
  | ∂ | ∂Qₙ | 148–217 |
  | ∂ | ∂T | 238–249 |

- **Evidence.** Native crops at 29, 55, 101, 105, 126, 164 and 205 s show the same pattern every time:
  - These glyphs have strokes roughly twice as thick as the hairline CM + and = in the same expression, with rounded ends.
  - The minus sign is short and heavy next to a thin +.
  - In |r(z)| ≤ ε|z − c| (153 s on), the ≤ sits directly above a thin CM < in |z − c| < δ.
  - ∂ is upright, whereas CM's ∂ is slanted.
- **Impact.** Everything is legible, the maths is right, and the look is consistent within the chapter. This is a mismatch of type faces, not a layout problem.
- **Scope.** I judged this from pixels only: I did not inspect fonts, and I looked at no other chapter. It is probably engine- or kit-level rather than specific to chapter 08. If root has already accepted this symbol mapping, there is nothing to do for this chapter.

### L2 Low: the repaired nested c label clears everything but sits far from c, 117–163 s

- **Measured at 4K.** The point is centred at (1336, 920); the label occupies x 1580–1605, y 872–905. The label's centre is about 258 px from the point, which is the (+0.95, +0.12) offset at `repair164:502`.
- **Clearances:**
  - about 90 px to Q₂'s right edge and about 150 px to Q₁'s right edge (crops at 117, 120, 123 and 131 s);
  - about 150 px to the δ circle (crops at 153, 155, 160 and 163 s).

  No sampled picture shows a crossing, so the repair holds. I agree with root's "clears".
- **The cost.**
  - From 117 to 160 s the label sits on the far side of Q₂'s gold edge from the point.
  - At 163 s only the circle and the point remain, and the label floats outside the circle.
  - There is no leader line.

  It is the only label in the picture and the voice says "one point c", so viewers can still connect them.
- **Advisory only.** A leader line, or a position closer to the point but outside the circle, would bind it more tightly. Any move would need rechecking against Q₂'s edges and the circle in every hold from 117 to 163 s.

### L3 Low, predicted and not observed: the green distance segment probably crosses the zoom c label for a few frames near 171.7 s

- **Source.** In `repair164:616–641`:
  - the walker goes round the magnified square in 2.0 s (`tween_linear(progress, 4.0, 2.0)`);
  - the green segment from c to the walker updates every frame;
  - the segment is created after `centre_name` (`:607`, `:619`), so it should draw over the label, in the same green.
- **Geometry, from the native crops at 168 and 172 s.**
  - The label is centred 59 px left of and 77 px above c (the −0.22, +0.28 offset).
  - The segment passes through that spot when the walker is about 414 px left of c on the top edge.
  - Given a segment about 12 px wide and a glyph about 30 px wide, I estimate about 8 frames around 171.7 s. The walker covers one side every 0.5 s and reaches that spot about 0.44 s after passing the top-right corner.
- **What the stills show.** No sampled picture falls in that window: at 171.0 s the walker is on the right edge, and at 172.0 s it is on the left. Root's note that the label clears the moving segment at 168 and 172 s is true of those stills.
- **What to do.** Check it in motion. If it is visible, drawing the label above the segment would keep it legible.

### L4 Low: on the triangle, the right-edge midpoint dot is half covered by the cyan Dₙ edge, 233–249 s

- **Evidence.** Native crops at 233 and 240 s (x 1050–1700, y 750–1450) show:
  - the white midpoint dot at about (1353, 1065), half hidden under the thick cyan edge;
  - the gold counterclockwise arrow running about 20 px inside the edge and touching the dot.
- **Impact.** It is still readable. Cosmetic only.

## Observations (no action recommended)

- **O1, 205–207.6 s.**
  - The cancellation inequality is built from separate objects, |I₀| ≤ · 4ⁿ · and the fraction, as the source comment at `repair164:694–695` intends.
  - The fraction bar is 733 px long (x 2768–3501), against a numerator about 226 px wide. The pieces are about 260 px and 150 px apart.
  - It reads as one inequality, helped by the voice.
- **O2, 140–147 s.** In the green tip-to-tail walk:
  - two of the four arrows overlap for about 250 px;
  - the upward arrowhead sits on the return shaft (native crop at 143 s).

  This comes from the exact changes being drawn, and the walk still visibly closes.
- **O3, 62.5–74 s.** Only the title, the hypothesis and ∮γ f dz = 0 are on screen, on the right. The left two-thirds stay empty until the square appears at about 74 s. A staging choice.
- **O4, 180–204 s.** The Dₙ label sits about 40 px above the top-right corner of the half-size square, on the diagonal. Nothing overlaps, but it could be read as labelling the inner half-diagonal.
- **O5, 242–249 s.** The shared "precisely:" lower third:
  - its lowest ink is at row 2108, leaving a 51 px bottom margin at 4K;
  - its 105 characters are fully on screen for about 7.2 s, about 14.6 characters per second;
  - overlap with player controls or captions was not measured.
- **Proximity candidates checked natively and found clear:**
  - "second lap, over the first" starts 67 px from the rightmost chain arrowhead at 27–31 s;
  - the f(zₖ) label at 36–42 s is about 260 px above the bottom of the frame;
  - the zoom c label is about 46 px above the green segment and about 120 px from the diagonal at 172 s.

## The two named checks

### c-label repair: holds

| time (s) | picture | native measurement |
|---|---|---|
| 117, 120, 123, 131 | `000138`, `000141`, `000144`, `000152` | label about 90 px from Q₂'s right edge and about 150 px from Q₁'s; no gold edge crossed |
| 153, 155, 160 | `000174`, `000176`, `000181` | label about 150 px outside the δ circle; no crossing |
| 163 | `000184` | squares faded out; label about 150 px from the circle (see L2) |
| 164–217 (zoom label) | `000185`, `000189`, `000192`, `000193` crops | zoom label clears the diagonal and the green segment at 168, 171 and 172 s (see L3 for the frames between) |

### 205.6 s cancellation: intended

- **The two lint rows are the two strokes.** The layout lint's "crossed" boxes (x 2.08–2.81, y 0.73–1.26 and x 4.14–4.86, y 0.23–0.77) are exactly `cancel_top` (2.1, 0.75)→(2.8, 1.25) and `cancel_bottom` (4.15, 0.25)→(4.85, 0.75) at `repair164:706–707`, plus stroke width.
- **The pixels agree.** In picture `000227` (frame 12336), the green ink bounding boxes are x 2483–2679 and x 3036–3239. Each stroke goes diagonally through one green 4ⁿ, the top factor and the denominator.
- **Persistence matches the lint's four later looks.**
  - The strokes are still there at 205.8, 206.0, 206.6 and 207.0 s (`000228`–`000231`).
  - They are fading at 207.6 s (`000232`) and gone by 208.0 s (`000233`).
  - |I₀| ≤ εD₀P₀ then follows, with I₀ = 0 at 213 s.

## Coverage

- **Sheets.** I read all 36 sheets (`sheet-0001.jpg` to `sheet-0036.jpg`) one at a time with the image reader and logged each one. Each tile is one of the 283 sampled pictures, so all 283 were seen at sheet scale.
- **Native crops,** from the 3840×2160 JPEGs: 27–31, 29, 38, 55, 101, 105, 117, 120, 123, 126, 131, 143, 153, 155, 160, 163, 164, 168, 171, 172, 184, 205, 205.6, 205.8, 206.6, 207.6, 217, 233 and 240 s.
- **Pixel measurements** of ink extents with PIL: the Q_n window over all samples from 150 to 230 s, the c-label clearances, the stroke bounding boxes, the fraction bar and the lower-third margin.

## Limits

- **Stills only.** These are 283 of 14,959 frames, as JPEGs extracted by root; I did not decode the MP4. Nothing here reviews motion, and L3 is a prediction from the source and the geometry.
- **No audio.** No listening, no fine sync, no audio measurements and no independent final AV acceptance. All of these remain open.
- **No player checks.** Player overlays, captions and public playback were not checked.
- **L1 rests on pixels alone.** I did not inspect font tables.
- **Independence.** Root read the same sheets before I did. I opened root's two progress files only after reading all 36 sheets and taking the native crops, and before writing this file. Where we agree, we reached it separately; we differ on how severe M1 is.
