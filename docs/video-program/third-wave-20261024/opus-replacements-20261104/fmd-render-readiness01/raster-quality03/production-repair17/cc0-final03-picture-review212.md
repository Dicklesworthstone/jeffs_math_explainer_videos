# cc0 final03 picture review 212: chapter 03, two planes

Dispatch-0 (cc0), 2026-11-08T04:16Z. **Independent review of rendered stills only.** I rendered, extracted, built and synthesised nothing. This file is the only file I wrote in the repository. For viewing I made crops of the existing pictures, kept outside the repository.

## Subject

- **Film.** `final03-terminal115/03_two_planes.mp4`, sha `20353d01ee564ef101ecad80bdfa8ec1db87ac2188b2e4642969b371f2e82cf3`. I recomputed this sha at 04:15Z. It is 7,249 frames, from binary `f4836eee…` (production19).
- **Root's receipt.** `final03-terminal115/pictures-reviewed115.json`, sha `623e81de…`, reviewed at 2026-11-07T23:28:28Z.
- **Extraction.** `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-final03-pictures115/`. Its `sampling.json` lists 154 native 3840×2160 pictures and 20 sheets. I checked all 174 file hashes against `sampling.json` and found 0 mismatches; its `movie_sha256` equals the film's.
- **Script.** Lines 03_1–03_7 of the bound `script.tsv`, sha `946a9c07…`. I checked what is shown against the words of each line. I did not hear the audio.

## Result

**No blocking defect.** In every sampled picture, the maths shown agrees with the script and with direct calculation. Every label is legible at native size, and nothing is clipped. I found two Low items and one cosmetic note. None needs a re-render for release; they are worth fixing only if chapter 03 is reopened for another reason.

**Domain colouring:** chapter 03 has no domain-colouring field in any of the 154 pictures. It uses grids, curves, circles and dots only, so the smoothness check does not apply to this chapter.

## Findings

**F1, Low: the turn counters disagree with "twice" while they move (25–31 s, line 03_2).** The input and output counters are rounded separately. In 4 of the 7 sampled moving pictures, the output reading is not twice the input reading. I read all of these on native crops of pictures 33–41:

| Time | Input | Output | Twice the input |
|---|---|---|---|
| 25 s | 0.14 | 0.28 | 0.28 |
| 26 s | 0.28 | **0.57** | 0.56 |
| 27 s | 0.43 | **0.85** | 0.86 |
| 28 s | 0.57 | 1.14 | 1.14 |
| 29 s | 0.71 | 1.42 | 1.42 |
| 30 s | 0.85 | **1.71** | 1.70 |
| 31.0 s (picture 40) | 1.00 | **1.99** | 2.00 |
| 31.65 s (picture 41, settled) | 1.00 | 2.00 | 2.00 |

The underlying values are right: the output angle is double the input angle in every sampled picture. The narration says the image goes around "twice", so a viewer who pauses can see "1.00" beside "1.99". The settled picture is correct. I could not measure from samples how long each mismatched pair stays on screen. If 03 is reopened, the fix is to compute the output reading as 2 × the input reading as displayed, or to truncate both readings instead of rounding them.

**F2, Low: the "≠" glyph does not match the CM "=" beside it.** It appears in "z ≠ 0" at the end of the gold formula (16.6–31 s; native picture 28 and 40 crops) and in the red "e^z ≠ 0" caption (115–120 s; picture 148 crop). The "≠" is heavier and shorter than the Computer Modern "=" in the same line and in "e^{z+2πi} = e^z", so it looks like a different face. It is legible and its meaning is clear. I did not check the font source; a glyph probe would settle whether it is a fallback face.

**F3, cosmetic note: the axis shows through the e^{2πi} box (112–120 s).** The box's backing panel is translucent. The imaginary axis and its 2i tick run through the number between "1.000" and "000".
- In the settled pictures 146–153, the show-through is faint. Measured red levels are: axis about 40, panel about 24, text about 232.
- In picture 145 (112.0 s), during the panel's fade-in, the axis is bright through the text: about 108 against about 20.

The text reads correctly in every sampled picture. I am not counting this as a defect; root's receipt says there is no unintended overlap, and this faint show-through is the only place where I would qualify that.

## Mathematics checked

Measurements were taken from sheets (960×540) unless the row says native.

| Line | Checked | Result |
|---|---|---|
| 03_1 | z ≈ 0.78+0.58i maps to f(z) ≈ 0.27+0.93i; z² = 0.27+0.905i | agrees at sheet precision |
| 03_2 | Vectors: \|z\| ≈ 1.37 at 29° → 1.93 at 59° (\|z\|² = 1.88). The radius ½ circle maps to the radius ¼ circle, measured and labelled "radius 0.5" / "radius 0.25" (native, picture 40). The output dot's angle is double the input's. The formula "\|z²\| = \|z\|², arg z² = 2 arg z (mod 2π), z ≠ 0" is legible (native). | agrees; F1 |
| 03_3 | The question box "straight after z²?" appears at 35.0 s. The countdown ring appears at 37.933 s (mark `predict`, frame 2276) and is empty at 42.0 s, about 4.07 s, which matches the timeline's 4 s think row at frame 2276. "No." starts at 42.183 s with the answer still hidden; the straight copy appears on the output plane at 42.683 s and bends from 44 s. | quiz answer hidden until think time ends |
| 03_4 | Native picture 81, at 316.5 px per unit: gold parabola vertices at −1 and −0.25, cyan at 0.25 and 1, and the outer pair's tips at ±2i. These match u = v²/4b² − b² and u = a² − v²/4a². The right-angle marks sit at 0.5i, −0.75−i and 0.75−i, which are the exact squares of the marked input crossings 0.5+0.5i, 0.5−i and −1+0.5i. The real axis lands on [0, 1] (gold) and the imaginary axis on [−1, 0] (cyan). At 0, the 90° red corner on the input becomes a 180° red half-circle on the output (native, picture 84). The two captions and "right angles kept, except at 0" match the words. | agrees |
| 03_5 | Four sampled points: (0.38, 0.26) → (1.81, −1.23), where 1/z = (1.79, −1.23); (0.99, 0.66) → (0.70, −0.48), where 1/z = (0.70, −0.47); (1.81, 1.23) → (0.38, −0.26); (0.92, 0.62) → (0.74, −0.52), where 1/z = (0.75, −0.50). The four coloured points on the unit circle land on their conjugates. The arrows run from the hollow position of z to the filled 1/z. The pictures also show "inside → outside, outside → inside", and a red X at 0 with an opaque "1/0 undefined" callout, legible at native (picture 111). | agrees |
| 03_6 | Native: the 2πi tick is at 6.29 units. The dots sit at 0.368, 1.000 and 2.718 at 228 px per unit, with "×e" (picture 123). At 100 s, the radius-e arc ends at 4.14 rad while the x = 1 segment has climbed to 4.13 (picture 129). The circles have radii e⁻¹, 1 and e. The horizontals sit at 0.78, 2.36, 3.93 and 5.50, which are π/4, 3π/4, 5π/4 and 7π/4, and become rays at 45°, 135°, 225° and 315°. The drawn input segments span x ∈ [−1.52, 1.17], and the rays span r ∈ [0.20, 3.17], equal to eˣ within the stroke caps (pictures 136, 137 and 153). | agrees |
| 03_7 | The dot climbs: input height 0.32 / 2.13 / 2.9 / 5.76 → output angle 0.31 / 2.14 / 2.88 / 5.76 rad on the unit circle. At 2πi the output is back at 1. The box reads "e^{2πi} = 1.000000 + 0.000000i". The captions read "e^{z+2πi} = e^z" and "e^z ≠ 0", with a red ring at output 0. | agrees; F2, F3 |

## Readability and layout

- **Text.** All titles, plane headers, captions, value labels and tick labels are legible at native size and at sheet scale. The smallest text is the grey tick labels "1" and "i" and the grey caption "new windows: the ticks still mark whole numbers" (90.7–96 s). Both are legible at native size (pictures 73 and 120). I did not test phone size.
- **Clipping.** I found no clipping. The radius-e circle and the longest rays stay inside the output frame.
- **Label placement.** The "2.718" label sits to the left of its dot, while "0.368" and "1.000" sit to the right of theirs. It still reads correctly.
- **Transitions.** Mid-motion pictures show the morph in progress:
  - doubled cyan curves at 48 s, as images of ±a converge;
  - the axis fold at 50 s;
  - fades at 15, 32, 69 and 89 s.

  Each one resolves to the correct settled picture in the next sample.
- **First and last frames.** Picture 1 (frame 0) and picture 154 (frame 7248) are uniform background, RGB (12, 17, 20).

## Coverage

- **Sheets.** I read all 20 sheets, which together show all 154 pictures at 960×540. I logged them sheet by sheet as I went.
- **Native views.**
  - Whole pictures at half scale: pictures 28 and 40.
  - Crops: the counters in pictures 33–41, plus pictures 62, 69, 71, 73, 81, 84, 111, 120, 123, 129, 136, 145, 148 and 153.
  - Pixel measurements: pictures 1, 136, 137, 145–153 and 154.
- **Root's detail times.** Root's detail seconds 60 and 100 are pictures 81 and 129, and I viewed both at native size.
- **Agreement with root.** I agree with the observations in root's receipt and add F1–F3.

## Limits

- **Sampled stills only.** I checked 154 of 7,249 frames, about 2%. I make no claim about motion between samples or about how long transient states last.
- **No audio check.** I did not play or hear the soundtrack. Per root, the narration is ElevenLabs Frederick Surrey; I did not verify the audio track, its loudness or its sync.
- **No captions or fine-sync.** I did not check the captions, and no fine-sync was checked.
- **Not final acceptance.** This is not independent final AV acceptance of chapter 03 or of the master, and it is not publication acceptance.
