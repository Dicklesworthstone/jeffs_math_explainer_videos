# Interim defects: complex 04, preview02

BronzeJaguar, 2026-11-04. Interim only. Chapter 05 is still being read, so this is not a verdict on the four chapters.

**Result so far: 225 of 348 sheets read, all of 02, 03 and 04. One High, 04-A stripes, already sent as Mail 2167 and now better evidenced. Three Medium or Medium-low defects in 04, and a set of Lows.**

**Not independent.** I wrote this chapter's native scene, the preview01 review of it and the repair rendered here.

**Evidence.** Sheets opened as images, listed with hashes in `coverage/viewed.tsv`. Full frames re-read in 04: `frame-0020`, `0070`, `0143`, `0205`, `0226`. Nothing was watched or heard. Preview02 is 720p30 with the scratch voice, so every time is a scratch time and all fine sync is unmeasured. Sampling is about 0.5 s, so a shorter defect can be missed.

## Do first

| ID | Severity | Where (scratch) | Defect | Edit |
|---|---|---|---|---|
| 04-A | **High**, persists, halved | Every colour picture in 04, frames 12 to 222 and 273 to 316 | Thin dark rows across the picture. Rows 1.6–1.8% rms, period 3.14 px, columns 0.03–0.06%, background 0.00% | Overlap the strips (Mail 2167). Untested |
| 04-A2 | Medium, persists | 04 frames 61 to 222 and 273 to 315, sheets 15 onward. Plain on `frame-0070` | Band edges are stair-stepped at the cell size. The cell is 3.15 px here and, by scale only, 9.45 px at 4K | Patch P3 from `complex-picture-repair-02-05` (four samples a cell). It was left out because its render cost is unmeasured. Measure it on one 4K still, or take a smaller `CELL` within `a_full_picture_stays_within_the_strip_budget` |
| 04-still | Medium, new | 04_3 and 04_4, 41.5–67.5 s, frames 97 to 163, sheets 24 to 40. Same remedy for 04_5, 79.6–84.7 s, frames 199 to 217, sheets 49 to 54 | The chapter's central claim, that the colours cycle in the wheel's order and twice for a double zero, is never acted out. The one picture holds for 26 s, with only a marker at 50.4 and a card at 56.8 | On "cycle in this order", walk a dot once around the gold loop while a wheel needle turns twice, re-using the probe and needle of 04_1. On "cycle backwards", repeat on 1/z with the needle going the other way |
| 04-a | Medium-low, new | Band, 67.7–76.0 s, frames 164 to 185, sheets 41 to 46 | "near a zero a of order m" sets the article and the variable as the same upright letter, so "a zero a" does not parse on one read. m, c, z and f(z) are upright too. It is about 122 characters in about 8 s, right at the 15 c/s floor | Italic maths for the variables, or name the point z₀: "near a zero z₀ of order m, f(z) is about c(z − z₀)^m". Hold 0.5 s longer if the text stays this long |

## Stripes: new evidence since Mail 2167

Row residuals on four different functions in the same plane correlate **+0.92 to +0.97**, and **−0.28 to −0.37** when shifted one row. The period is 3.14 px on all four. The stripes follow the plane's row geometry, not the picture's content or encoder noise. The same-still control I quoted in Mail 2167 is weak, because inter-frame coding can copy a still's noise forward, so please rely on this one. The cause in `RowStrips::build` is still inferred, and the overlap edit is still untested. 1080p and 4K are still unmeasured. Script: `checks/stripes02.py`, plain `python3 -B`.

## Low

- Title ahead of the picture, three times. New formula over the old picture: 39.3–40.0 s (sheet 23), 76.2–77.5 s (sheets 46–48), 85.233–86.0 s (sheet 55). Delay each title to the crossfade.
- "zero of order 2" arrives at 56.8–57.3 s, about 2.5 s before its words by word share. "Once for a simple zero" is spoken over the order-2 zero. Edit: dim row "once: simple zero" on "Once", card on "twice".
- `dark lines: every twelfth of a turn` is small and dim while it is spoken, 33.5 s on.
- Picture still 28.5–33.0 s while the voice explains the band ratio. Candidate: on "a new band begins", pulse one band edge.
- "simple pole" (04-D) is readable now on its dark card with a pink border, `frame-0205`, but has less contrast than the white "zero of order 2". Raise the card's fill opacity, or use white text.
- Gold over the pole markers only at the sampled transitions, 108.0 s (frame 284) and 118.0 s (frame 314).
- Sampled gap at 107.5 s (frame 283): the dim circle is gone and the gold is not yet in.
- "negative", "positive" beside the wheel are never spoken. Upright i in "at i: −1 turn, at −i: −1 turn".
- Small dark specks on the gold loop's inner edge at row steps, in a 6× crop of `frame-0143`. I cannot separate picture from CRF 26 here. Recheck on a better encode.

## Fixed on these pictures

04-B (formula on screen), 04-C (probe and needle match the voice), 04-E (solid loop heads), 04-F and 04-I (true minus; the repeat remains as a Low above), 04-G (quiz formula shown, no answer), 04-H (rows gone during the quiz), 04-K (gold segment named). 04-D and 04-J improved, listed above. P2's moved bands look right: rings at about 141, 84 and 48 sheet px from the centre, ratio about 1.7, matching "size × 1.69".

## Passed, with what was checked

- Predict window: voice ends 97.056 s, ring from about 98.0 to 102.5 s, voice restarts 102.933 s, picture fades in 103.5–104.5 s, markers 105.0 s. Frames 224 to 272 show no colour and no marker. The unit circle stays on the plane during the question. That hints but does not place the poles.
- Hues by hand against f: z, z², 1/z, and 1/(1+z²) at both poles and in the far field. All match.
- Lint rows judged by eye: `zero of order 2`, `precisely:`, `near a zero a of order m…`, `times as z circles a`, `names to search:`. No overlap defect. Their text is legible.

## Corrections to Mail 2166

I re-derived these locators from `coverage/sheets.tsv`. The times were right; these sheet and frame numbers were estimates.

| ID | Sent as | Correct |
|---|---|---|
| 02-freeze | sheets 39–41 | frames 151–177, sheets 37–44 |
| 02-G | frames 225 onward, sheets 56–65 | frames 228–259, sheets 57–64 |
| 03-empty | frames 41–53, sheets 10–13 | frames 39–53, sheets 9–13 |

03-A, 03-B and 03-C, "to be restated" in Mail 2166, are fixed on `03_two_planes/frame-0060.png`.
