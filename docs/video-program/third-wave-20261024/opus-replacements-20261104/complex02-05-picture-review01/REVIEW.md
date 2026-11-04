# Picture review: complex analysis 02 to 05, preview02

BronzeJaguar, 2026-11-04. Assignment: Mail 2134, bead .39. This is the final report. `interim-02-03.md` and `interim-04.md` stay as the sent record, and this file carries their corrected locators.

**Result: all 348 sheets read, 1,384 sampled frames. These chapters are not picture-clean yet. One High remains: thin dark rows across every colour picture in 04 and 05. Four Medium and six Medium-low defects remain, plus Lows. I found no wrong mathematics at any place I checked by hand. Most of the preview01 defects are gone on these pictures.**

**Not independent.** I wrote the native scenes of all four chapters, the preview01 review of them (Mail 1673) and the repair rendered here (Mail 1744). These pictures are the first real look at that repair. Someone else should read at least chapter 05.

**Evidence.** Sheets opened as images, plus the full frames and crops listed under Coverage. Nothing was watched or heard. There is no ASR or listening equivalence here. Preview02 is 720p30 with the scratch voice, not ElevenLabs Oliver, so every time below is a scratch time. Fine sync comes from per-cue word shares only and must be rechecked on Oliver. Sampling is about 0.5 s, so a shorter defect can be missed.

No edit below has been compiled or rendered. I changed no code, canonical file, Bead or render.

## Do first

| ID | Severity | Where (scratch) | Defect | Edit |
|---|---|---|---|---|
| 04-A | **High**, persists, halved by P1 | 04 frames 12–222, 273–316. 05 frames 3–490. Worst on `05_gallery/frame-0217` | Thin dark rows every 3.15 px across every colour picture. On tan they fill both flat regions while the voice says "the colours there go flat" (05 frames 188–227, sheets 47–56) | See "Stripes" below. Measure one 4K still first |
| 04-A2 | Medium, persists | 04 frames 61–222, 273–315. All 05 pictures, worst on the zoom, frames 460–489, sheets 115–122 | Band edges are stair-stepped at the cell size. On `05_gallery/frame-0468` the stepped edges sit beside the smooth gold outline captioned "traced exactly". By scale only, the cell is 9.45 px at 4K | Patch P3 from `complex-picture-repair-02-05` (`patches/shared-P3-optional-four-samples.patch`), left out because its render cost is unmeasured. Measure it on one 4K still, or take a smaller `CELL` within `a_full_picture_stays_within_the_strip_budget` |
| 04-still | Medium, new | 04 frames 97–163, sheets 24–40, 41.5–67.5 s. Same for 1/z, frames 199–217, sheets 49–54 | The chapter's central claim, that colours cycle in the wheel's order and twice for a double zero, is never acted out. One picture holds for 26 s | On "cycle in this order", walk a dot once around the gold loop while a wheel needle turns twice. Repeat on 1/z with the needle reversed. Chapter 05 proves the device: at 8.0–13.5 s, frames 19–30, walker, output box and readout carry the same kind of claim well |
| 03-empty | Medium | 03 frames 39–53, sheets 9–13, 15.5–22.0 s | Both planes stay empty for 6.5 s under "squares each length and doubles each angle". Only the formula row carries it | On "doubles each angle", draw one radius in the input at about 30° and its image at 60° with a small arc. Keep them until the walk starts at 22.5 |
| 05-pole | Medium, new | 05 frames 366–433, sheets 91–108, 126.5–146.6 s. Plain on `frame-0420` | The gold box's lower edge lies on the real axis, so it runs straight across the pole marker at s = 1 and through its black centre, for about 20 s. This is the marker the voice has just called "a single pole". Frame 363 is the clean control | After `gallery.rs:782`, raise the marker: `stage.bring_to_front(&pole1)?`. The pin has `bring_to_front(&[Mob])` at `fmn-scene/src/runtime.rs:966`. No chapter uses it yet, and I did not confirm that this `Stage` exposes it |
| 05-tile | Medium-low, new | 05 frames 454–465, sheets 113–116, 153.2–157.5 s | The tile that proves "nearly square" finishes about 1.5 s after those words, under "That is not a coincidence". From "in little tiles" to the finished outline is 3.9 s of fixed animation (0.4 + 0.4 + 1.0 + 0.8 + 0.3 + 0.3 + 0.5), and the words need about 2.75 s by share | `gallery.rs:892` leaves five painters one after another for 1.0 s. Use `leave(stage, 0.05)`, `zoom.show` 0.5 s and border plus links 0.2 s. That gives 2.25 s, inside "nearly square" by share. Recheck on Oliver |
| 05-crowd | Medium-low, new | 05 frames 397–432, sheets 99–108, 135.0–146.5 s. Plain on `frame-0420` | Eleven text items plus the formula are on screen by 140.5 s: four rows, four dot labels, a caption, a card, a bar. The caption sits 15 sheet px above the bar, against the one-slot rule | When the strip arrives (`gallery.rs:782`), fade `converges` and `pole_says`. When the bar arrives (`gallery.rs:823`), fade `squashed`. Take them out of the later exit list near `gallery.rs:836` so nothing fades twice |
| 04-a | Medium-low, new | 04 frames 164–185, sheets 41–46, 67.7–76.0 s | "near a zero a of order m" sets the article and the variable as the same upright letter, so it does not parse on one read. About 122 characters in about 8 s, right at the 15 c/s floor | Italic maths for the variables, or name the point z₀. Hold 0.5 s longer if the text stays this long |
| 02-freeze | Medium-low | 02 frames 151–177, sheets 37–44, 57.0–65.5 s | Nothing moves for 8.5 s while the voice recalls the eighth turn and asks the question | On "an eighth of a turn", pulse the angle arc or redraw the 1+i arrow |
| 02-G | Medium-low, persists | 02 frames 228–259, sheets 57–64, 84.0–96.3 s | Nothing moves for 12.3 s, of which 9.6 s is under the voice. The right panel holds 7 rows | On "Lengths multiply", pulse the two readouts. On "Every complex derivative", fade the F and the grid so the two arrows stand alone. Drop the 41.5 s caption at 74.4 |
| 03-flip | Medium-low, new | 03 frames 197–207, sheets 49–51, 78.0–82.5 s | "Each point flips across the horizontal axis" is said and the picture is still for 4.5 s. The flip is never performed | On "flips", draw a short vertical chord from one output dot to its reflection, or pulse each matching pair in turn |

## Stripes: what is measured and what is not

| Picture | Frame | Rows rms | Period | Columns rms |
|---|---|---|---|---|
| tan, top flat | 05 `frame-0217` | 1.83% | 3.23 px | 0.06% |
| tan, bottom flat | 05 `frame-0217` | 1.94% | 3.25 px | 0.05% |
| z³ − 1 | 05 `frame-0008` | 1.71% | 3.11 px | 0.57% |
| zoom | 05 `frame-0473` | 1.68% | 3.13 px | 0.30% |
| f(z) = z, three patches | 04 `frame-0020` | 1.6–1.8% | 3.14 px | 0.03–0.06% |
| frame background | 05 `frame-0217` | 0.00% | | 0.00% |

The exp, sin and zeta patches are not stripe measures, because their own bands dominate (3.81%, 9.38%, 2.65%). They are listed in `checks/outputs/gallery_measures.txt`.

**Tied to the plane's row geometry, on two independent tests.**
- On 04, row residuals across four functions correlate +0.92 to +0.97, and −0.28 to −0.37 when shifted one row.
- On 05, the same pair test failed to say anything, because sin's bands swamp it (+0.14, +0.17). I replaced it with a fold test on tan's two flat regions, about 320 rows apart. At the predicted 3.15 px (`CELL` 0.035 at 720p) both give coherence 0.56 with a phase gap of 1°. No other period tried from 3.00 to 3.30 px has both: 3.25 px gives 0.50 with a 33° gap, 3.00 px gives a 15° gap at only 0.14 and 0.17, and 3.05, 3.10, 3.20 and 3.30 px have gaps of 110° to 157°.

**The cause is still inferred.** `RowStrips::build` (`props.rs:289`) strokes each row at exactly one cell height, and `crisp` (`draw.rs:556–563`) sets a 0.5 anti-alias width, so two soft edges meet at each row boundary and let the dark background through.

**Before repairing, measure one 4K still.** A 1-pixel soft seam every 9.45 px should show less than one every 3.15 px. That is an expectation from scale, not a measurement. 04-A could fall below High at 4K, and everything measured here is 720p at CRF 26.

**If it stays visible, overlap the rows, with two cautions. Both are new since Mail 2167.**
- `RowStrips` is shared with `p_values_medical`, `eigen_engineering` and `raptorq_gf256`. An edit at `props.rs:289` changes those films too. Prefer an opt-in, or set the stroke in the complex crate's `crisp`.
- Overlapping rows double-cover while a picture fades, so a fixed overlap could draw brighter or darker rows during every fade. Mail 2167's 0.012 units is about 1 px at 720p but about 3.2 px at 4K. Test a fade frame as well as a still, and size the overlap in pixels.

## Low

**05**
- 05_3, frames 106–119, sheets 26–29: "So its tiles are exact squares" has nothing to point at, and the `2π ≈ 6.28` bracket arrives about 2.4 s before "repeating every two pi". Edit: gold outline on one tile on "exact squares", bracket on "repeating".
- 05_8, frames 396–408, sheets 99–102: "Riemann hypothesis: open" arrives on "Whether", about 3.6 s before the name by share (`gallery.rs:819`). Edit: `wait_for(stage, "is the Riemann hypothesis")`.
- 05_2, frames 55–66, sheets 13–16: pole marker and card arrive at 23.0–23.5, under "simple zero at one". Edit: move them to the existing `wait_for` on "and at minus one".
- 05_7, frames 312–329, sheets 78–82, 107.0–113.0 s: picture and formula hold for 6 s while the voice reads the series, with the right half of the frame empty. This is what remains of 05-I. Edit: underline each term as it is spoken. No new numbers needed.
- 05_5, frames 212–227, sheets 53–56: still under "the colours there go flat". Candidate: brighten both flat regions on those words, after 04-A is settled.
- 05_1, frames 19–30, sheets 4–7: laps 2 and 3 overdraw one circle, so only the readout says "three". Same as 03's lap two. Candidate: a brighter tone per lap.
- 05_2, frame 65, sheet 16: readout says "+0.0" when the needle has already gone backwards. Edit: no sign on a rounded zero.
- 05_4, frames 165–186, sheets 41–46: the vertical arrow's head merges with the sin(3i) dot. Edit: stop the arrow at 2.8.
- `FOUR` (`gallery.rs:64`), frames 412–432: "this plot shows four zeros" with two plots and seven zero markers on screen. Four is right for the strip, since the next zero is at 32.93, above its top. Edit: "this strip", and check any test that pins the text.
- 05-L, improved: leaders tie each label to its dot, but the label column and right column still read a little like one table.
- Mosaic names, frames 445–457: the lower names sit in the 21 sheet px between rows, about 3 px clear on each side, and read as belonging to either row.
- "a single pole" in pink has less contrast than its white neighbours. The dim unit circle in 05_1 and the output box's small label are also faint.
- Upright maths: `sin x`, `Re s`, `Im s`, `s = 1`, i in the zero labels, f(z) in the output box label.
- Glyph pointers, already in `complex-glyphs02-review01`: Unicode `z³` beside the TeX title, frames 460–489; heavy title minus beside the thin readout minus, frames 64–75.
- Faint residue after fades: frames 308–311 (19.7 against background 16.2 luma, gone by frame 312), frames 436–439 and frame 490. I read these as encoder residue but cannot separate them at CRF 26. Recheck on a better encode.

**04**
- Title ahead of the picture three times: 39.3–40.0 s (sheet 23), 76.2–77.5 s (sheets 46–48), 85.233–86.0 s (sheet 55). Edit: delay each title to the crossfade.
- "zero of order 2" arrives at 56.8–57.3 s, about 2.5 s before its words, so "Once for a simple zero" is spoken over the order-2 zero. Edit: dim row "once: simple zero" on "Once", card on "twice".
- `dark lines: every twelfth of a turn` is small and dim while it is spoken, 33.5 s on.
- Picture still 28.5–33.0 s under the band ratio. Candidate: pulse one band edge on "a new band begins".
- "simple pole" reads on its card but has less contrast than "zero of order 2" (`frame-0205`).
- Gold crosses the pole markers at the sampled transitions only, 108.0 s (frame 284) and 118.0 s (frame 314). Same remedy as 05-pole.
- Sampled gap at 107.5 s (frame 283): the dim circle is gone and the gold is not yet in.
- "negative", "positive" beside the wheel are never spoken. Upright i in "at i: −1 turn, at −i: −1 turn".
- Dark specks on the gold loop's inner edge in a 6× crop of `frame-0143`, unresolved at CRF 26.

**03**
- One arrow in an otherwise empty frame, 1.5–7.0 s. Still again 10.5–15.0 s.
- Upright "input z", "output f(z)" beside italic dot labels. Quiz card "straight after z²?" has an upright z and a Unicode square.
- "radius 0.5", "radius 0.25" are spoken labels at 20 pt, under the skill's 24 pt.
- Output trace is complete after lap one (27.0 s), so only the readout says "twice".
- Predict ring's top edge touches the real axis, 37.5–41.5 s. Edit: move it to clear space in a quadrant.
- Foot labels say "real axis", "imaginary axis" while the voice says horizontal, vertical.
- White fold stub at the output origin (51.0–69.3 s) stays vertical while both folded axes are horizontal.
- 18 pt note "new windows: the ticks still mark whole numbers", 90.5–106.9 s, is not spoken.
- The r = 1 circle runs through the 8 sheet px gap between "0.368" and "1.000", 101.5–106.9 s, frames 257–276. Edit: put 1.000 above the axis, or drop it once the circles close.
- Rays and circle pass behind the top corners of the `e^{2πi}` card from 110.8 s. Text is clear.

**02**
- Still 10.5–16.2 s and 35.5–40.3 s. Judge both again on Oliver.
- Grid and F at home with only a caption, 40.8–45.0 s.
- 02-D half: the grid's top edge strikes through "1 + i", 77.8–78.5 s, frames 213–215.
- 02-F: quiz card `(1 + i)² = ?` has an upright i and a Unicode square, 65.5–74.9 s, frames 177–205.
- Upright w and |w| in "multiplying by w" (41.5 s on) and "names" (49.5–57.9 s). Upright i in "every point times i" (23.0 s on).
- The 1+i arrow and readouts arrive at 45.0–45.5 s, one sentence before the voice names 1+i. Deliberate, but the viewer sees the value first.

## Prior 05 IDs on these pictures

| ID | State | What shows it |
|---|---|---|
| 05-A comb at x = 0 | Fixed | Row-to-row change at column 406: 1.25 here, 7.18 on preview01 `frame-0092` (positive control) |
| 05-F tan bars | Fixed | Bar max/min 1.01–1.10 here, 2.14 and 1.94 on preview01 `frame-0200` (positive control). Stripes remain, see 04-A |
| 05-B unlabelled box | Fixed | "output f(z), sizes squashed" from 7.0 s |
| 05-C empty box | Fixed | No trace for about a second at most: 7.0–8.0 s and 24.5–25.2 s |
| 05-D late turns | Fixed | "turns: −2 (backwards twice)" from 29.0 s (frame 73), words about 29.3–30.1 s |
| 05-E sin(3i) card | Fixed | Card x 371–521, y 171–219 on the sheet, inside the plane by 6 px. The arrow is a separate Low |
| 05-G gold line | Half | Line runs behind the dots. Upright `sin x` remains |
| 05-I formula alone | Reduced | Picture shares the frame from 106.6 s. The 6 s hold remains as a Low |
| 05-J false "enlarged" | Fixed | "gold box = gold band. The strip runs on up to Im s = 32". Box and band geometry match by hand |
| 05-K labels | Fixed | "1/2 + 14.13i" and so on. Marker heights match 475 − Im × 12.44 |
| 05-L table rows | Improved | See Low |
| 05-M mosaic names | Fixed | Named 150.0–154.0 s |
| Edge zeros of sin | Fixed | 14 sheet px inside the frame |

**The repair's "please look" list.** The links cross only plain red tiles at the plot's right, touching no marker. The band's height reads at the strip's foot. The RH card stays about 31 frame px inside the right edge. The moved bands look right: rings at about 141, 84 and 48 sheet px on z², and exp tiles of pi/6 both ways.

**Fixed in 02–04, by what the frames show.** 02-A, B, C, E and half of 02-D. 03-open, 03-A to 03-K. 04-B, C, E, F, G, H, I (minus only), K. 04-D and 04-J improved.

## Passed, with what was checked

- **Predict windows.** 02, 68.5–74.4 s. 03, 36.85–41.73 s. 04, voice ends 97.056 s, restarts 102.933 s, frames 224–272 show no colour and no marker. No answer appears on any frame before its window ends. 05 has no predict cue.
- **Lint rows, by eye.** 02, 03: none. 04: all five rows, no defect. 05: all five are picture strips under a card that belongs on the picture (`simple zero at 1`, `pole of order 2`, `sin(i)`, `sin(3i)`, `integral works to the right`). Each is caught mid-fade and each settled card clears its marker. Most defects above were not on the lint list.
- **Hues by hand against f.** z, z², 1/z, 1/(1+z²), z³ − 1, the rational function, exp, sin, tan, and the real axes of Γ and ζ.
- **Geometry by hand.** Both `(1+i)` landings; parabola ends and fold; right-angle marks; 1/z and e^z dots; z³ zeros at 1 and e^(±2πi/3); exp's 2π bracket; sin's zeros; tan's zeros and poles; Γ's poles; ζ's pole and zeros on both plots.
- **Readouts against the walkers.** z³ at 9.0 s (probe 74.7°, 0.62 turn, readout +0.6). The rational function at 26.0 s (needle about 52°, computed 53.8°, readout −0.4).
- **Printed values.** tan(0.4 ± 12i) ≈ ±1.0000 i, sin(i) ≈ 1.18 i, sin(3i) ≈ 10.0 i, Γ(4) = 6, Γ(5) = 24.

## Coverage

| Chapter | Sheets read | Frames | Duration |
|---|---|---|---|
| 02_turning_numbers | 66 of 66 | 261 | 96.866 s |
| 03_two_planes | 79 of 79 | 315 | 120.7 s |
| 04_reading_color | 80 of 80 | 317 | 118.866 s |
| 05_gallery | 123 of 123 | 491 | 165.2 s |
| **Total** | **348 of 348** | **1,384** | |

`coverage/sheets.tsv` is the manifest, with hash, frames and times per sheet. `coverage/viewed.tsv` is the reading log. Its 348 rows match the manifest's hashes, and the sheet bytes had not changed at the end.

**A check on my own log.** While writing this report I saw that the images for 05 sheets 010–112 had been removed from my working context, and `viewed.tsv` only proves that a Read returned. I tested it two ways, and both held:
- The session transcript records an image result for all 348 sheets (`checks/transcript_images.py`, `checks/outputs/transcript_images.txt`).
- I wrote 16 claims from my notes to `PROGRESS.md` before re-opening anything, then re-opened 21 sheets: 02 `040`; 03 `011`; 04 `030`; 05 `014`, `016`, `018`, `025`, `026`, `034`, `040`, `041`, `050`, `052`, `061`, `066`, `067`, `077`, `083`, `087`, `088`, `089`. Fifteen claims matched directly, four of them on the sheet the manifest gave rather than the one I had guessed. The other, 05's 113.2–113.5 s onset, is only consistent, because `sheet-083` starts at 113.6 s. Card positions matched to the pixel.

This is the transcript's record plus a sample. It is not a second full read.

**Full frames read.** 02 `0046`. 03 `0060`, `0135`, `0164`, `0255`. 04 `0020`, `0070`, `0143`, `0205`, `0226`. 05 `0020`, `0100`, `0217`, `0420`, `0468`.

**Crops read.** 04 `frame-0020` and `frame-0143`; 05 `frame-0020` and `frame-0066` titles; 05 frames 363, 368 and 397 at the zeta pole.

**Skills.** Both the FrankenManim and the video skills were loaded this session. I re-read their review sections for this task: the storyboard and visuals checklist, review and continuity §7–8, and the TeX note on the Noto minus.

## Bindings

- **Frames:** `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/`, extraction 17:21:10Z to 17:30:02Z, exit 0, inputs `239a05204c61a179ac77067908869e902e17a201d9d39c6f9c1f08c9bef64584` equal to final.
- **Preview02:** 15:50:12Z to 17:20:29Z, exit 0, inputs `e0e8beafee7f85e80dc594ed1c64a0de7c3a8d14ee447d6878a6787861bbb33d` equal to final. `render all --res 1280x720 --fps 30 --scratch-voice --lint --crf 26 --preset ultrafast`. The four mp4 hashes are in `coverage/preview02-mp4.sha256`.
- **Source, equal to preview inputs and to canonical when this report was written:** turning.rs `2b4dd9a1e41e`, two_planes.rs `d8f547c95cef`, reading_color.rs `dc98bcee3fc0`, gallery.rs `6f310eb739ec`. draw.rs is `4eba4bb18b98`, changed by others after my candidate. props.rs is `73606e8921ec`. Line numbers above apply to these bytes. Compare before applying any edit.
- **Script:** `2968b0fd76540cbc…`, 105 cues, listed with times in `cues.txt`.
- **P3:** not in this render. sha256 `96bc7bf3501e25af25be2c834167e6b82d22701f1323f198ff5a0b352045ab62`.

## Not checked

- **Anything between samples.** A defect under about 0.5 s can be missed, and motion, easing and flicker cannot be judged from stills.
- **Sound and Oliver timing.** Nothing was heard. Beats anchored with `wait_for` follow the words on Oliver, but fixed animation durations do not. All sync findings need rechecking there.
- **1080p and 4K.** Stripes, stair-steps and residue are measured at 720p, CRF 26 only.
- **Hues away from the points named.** I did not re-run a whole-picture colour check.
- **Joins into chapters 01 and 06**, and everything outside 02–05. Those are root's.
- **Every edit proposed here.** None is compiled or rendered, and `bring_to_front` on this `Stage` is unconfirmed.

## Corrections to what I sent

- **Mail 2166 locators:** corrected in Mail 2194 and repeated in the table above. 02-freeze is frames 151–177, 02-G is frames 228–259, 03-empty is frames 39–53.
- **Mail 2167, same-still control:** weak, because inter-frame coding can copy a still's noise. Use the cross-function and fold tests.
- **Mail 2167, overlap edit:** it did not mention the shared kit or the fade effect. See "Stripes".
- **`PROGRESS.md`, order-2 card clearance:** the "about 9 sheet px" and "about 45 px" figures are wrong. By the `frame-0143` geometry, the card's corners clear the gold loop by about 14 frame px and its side by about 34.

## Reproduce

From the repo root. PIL scripts need plain `python3 -B`, not `-I`.

```
python3 -B  docs/video-program/third-wave-20261024/opus-replacements-20261104/complex02-05-picture-review01/checks/stripes02.py
python3 -B  docs/video-program/third-wave-20261024/opus-replacements-20261104/complex02-05-picture-review01/checks/gallery_measures.py
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex02-05-picture-review01/checks/locate.py
```

Saved outputs are in `checks/outputs/`. `SHA256SUMS` covers every file in this folder.
