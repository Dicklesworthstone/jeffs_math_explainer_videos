# Interim defects: complex 02 and 03, preview02

BronzeJaguar, 2026-11-04, 18:10Z. Interim only. Chapters 04 and 05 are still being read, so this is not a verdict on the four chapters.

**Result so far: 145 of 348 sheets read, all of 02 (66) and 03 (79). No High defect. Four Medium or Medium-low static stretches, and a set of Lows.**

**Not independent.** I wrote the native scenes of these chapters, the preview01 review of them and the repair rendered here. These pictures are the first real look at that repair.

**Evidence.** Sheets opened as images, listed with hashes in `coverage/viewed.tsv`. Full frames re-read: 03 `frame-0135`, `frame-0164`, `frame-0255`. Nothing was watched or heard. Preview02 is 720p30 with the scratch voice, so every time below is a scratch time and all fine sync is unmeasured. Sampling is about 0.5 s, so a defect shorter than that can be missed.

## Do first

| ID | Severity | Where (scratch) | Defect | Edit |
|---|---|---|---|---|
| 03-empty | Medium | 03_2, 15.5–22.0 s, frames 41–53, sheets 10–13 | Both planes stay empty for 6.5 s under "squares each length and doubles each angle". Only the formula row carries it | On "doubles each angle", draw one radius in the input at about 30° and its image at 60° with a small arc. Keep them until the walk starts at 22.5 |
| 02-freeze | Medium-low | 02_5, 57.0–65.5 s, sheets 39–41 | Nothing moves for 8.5 s while the voice recalls the eighth turn and asks the question | On "an eighth of a turn", pulse the angle arc or redraw the 1+i arrow |
| 02-G | Medium-low, persists | 02_6, 84.0–96.3 s, frames 225 onward, sheets 56–65 | Nothing moves for 12.3 s, of which 9.6 s is under the voice ("Lengths multiply … stretch and turn moves"). The right panel holds 7 rows | On "Lengths multiply", pulse the two readouts. On "Every complex derivative", fade the F and the grid so the two arrows stand alone. Drop the 41.5 s caption at 74.4 |
| 03-flip | Medium-low, new | 03_5, 78.0–82.5 s, frames 197–207, sheets 49–51 | "Each point flips across the horizontal axis" is said, and the picture is still for 4.5 s. The flip is never performed | On "flips", draw a short vertical chord from one output dot to its reflection, or pulse each matching pair in turn |

## Low

**02**
- 02_1: still 10.5–16.2 s. 02_3: still 35.5–40.3 s, voice ends 39.56. Judge both again on Oliver.
- 02_4: grid and F at home with only a caption, 40.8–45.0 s.
- 02-D half: the grid's top edge strikes through the label "1 + i", 77.8–78.5 s, frames 213–215. The label is clear afterwards.
- 02-F: quiz card `(1 + i)² = ?` has an upright i and a Unicode square, 65.5–74.9 s, frames 177–205.
- Upright w and |w| in "multiplying by w: …" (41.5 s on) and "names: …" (49.5–57.9 s). Upright i in "every point times i" (23.0 s on).
- The 1+i arrow and readouts arrive at 45.0–45.5 with "It stretches by that number's length" (`turning.rs` `wait_sentence(1)`), one sentence before the voice names 1+i. Deliberate, but the viewer sees the value first.

**03**
- 03_1: one arrow in an otherwise empty frame, 1.5–7.0 s. Still again 10.5–15.0 s.
- Titles "input z", "output f(z)" are upright beside italic dot labels. The quiz card "straight after z²?" has the same upright z and Unicode square as 02-F.
- Spoken labels "radius 0.5" and "radius 0.25" are 20 pt (`two_planes.rs`), under the skill's 24 pt for text the narration depends on.
- Output trace is complete after lap one (27.0 s), so lap two is a dot on a finished circle and only the readout says "twice". Candidate: brighter tone on lap two.
- Predict ring's top edge touches the real axis, 37.5–41.5 s. Move it to clear space in a quadrant.
- Foot labels say "real axis", "imaginary axis" while the voice says horizontal, vertical.
- White fold stub through the output origin (51.0–69.3 s) is named "0" but stays vertical while both folded axes are horizontal. Inside the red half circle from 63.5 s it reads a little like a dial needle.
- 18 pt dim note "new windows: the ticks still mark whole numbers", 90.5–106.9 s, is not spoken.
- The r = 1 circle runs through the 8 sheet px gap between "0.368" and "1.000", 101.5–106.9 s, frames 257–276, and reads as "0.368)1.000". Put 1.000 above the axis, or drop it once the circles close.
- Rays and circle pass behind the top corners of the `e^{2πi}` card from 110.8 s. Text clear.

## Fixed on these pictures

02-A, 02-B, 02-C, 02-E, half of 02-D (the 2i arrow is labelled). 03-open, 03-D, 03-E, 03-F, 03-G, 03-H, 03-I, 03-J, 03-K, each by what the frames show. Margins I was asked to look at: "0" at the fold clears the gold parabola by about 7 px on `frame-0135`; "2.718" clears the circle by about 5 px on `frame-0255`; the 1/0 card and leader are clear of the circle's dots.

## Passed, with what was checked

- Predict windows: 02, 68.5–74.4 s; 03, 36.85–41.73 s. No answer on any frame read before the window ends.
- Geometry checked by hand against sheet pixels: both `(1+i)` landings; all parabola ends, vertices and the fold; the three right-angle marks and their images; 1/z dots (2 on the ray, 4 on the circle); e^z dots, arc angles and ray ends.
- 02 and 03 have no layout lint rows.

## Still open for the final report

03-A, 03-B and 03-C are to be restated from sheet evidence at the final. Chapters 04 and 05, including stripes and the moved bands, are next. Coverage continues without reduced depth.
