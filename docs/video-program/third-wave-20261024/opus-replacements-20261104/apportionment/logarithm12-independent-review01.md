# Logarithm chapter 12: independent source and selected-picture review

Reviewer: WhiteSwan. Assignment: Mail 1119, thread mx-video-program-program-87do.63, 2026-11-04.
Nothing was edited except this file. No Cargo, render, TTS, Beads or commit.

## Verdict

The mathematics holds: every claim I tested in lines 12_1–12_6 survived, including the axis
exclusion and the principal-log step.

The pictures do not yet earn acceptance. Root's note found transient crossfades and no settled
defect. I agree the crossfades are not defects, but there are settled defects elsewhere:

- **F1:** the other-axis spin never draws the red trace it is built to show, and it drops the corner
  that would expose the miss.
- **F2:** the product-rule text crosses panel C's border on every sampled frame from 26.0 to 47.0 s.
- **F3:** panel D says "many flows, one place: all home" over three dots at three places, none of
  them at the grey dot in the middle.

Every fix proposed here changes pictures only. None touches `script.tsv`, so none needs a new take.
All proposed code is uncompiled.

**This is not motion, listening, fine-sync or independent AV acceptance.** I read source and stills.
I heard nothing and watched nothing move.

## Evidence

**Sheets viewed: all 91, `sheet-000.png` through `sheet-090.png`** in
`narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-opus-rebuild-frames01/12_what_it_forgets/`.
Together they carry all 363 frames.

- Sheets 000–067 were read earlier in this session, before a context reset, and those observations
  come from my notes of that read.
- Sheets 015, 020, 027, 037, 042, 055 and 058 were opened again for the anchors below. Sheets 068–090
  were read after the reset.
- Frames opened singly at 1280×720: `frame-0100.png`, `frame-0111.png`, `frame-0284.png`.

**Source read:**

| File | sha256 prefix |
|---|---|
| `explainer_corpus/logarithm_mysteries/src/chapters/what_it_forgets.rs` | `8354492ba9d6a8cb` |
| `explainer_corpus/logarithm_mysteries/src/chapters/draw.rs` | `f9d2511334565991` |
| `explainer_corpus/logarithm_mysteries/script.tsv` (lines 12_1–12_6) | `3e4e34af7090e448` |

I also read `docs/sources.md`, `docs/storyboard.md`, `chapters/doing_nothing.rs` around line 293,
and `crates/explainer_kit/src/{play,kit,narration}.rs` where cited, and located the `rate_func` field in
`crates/fmn-mobject/src/animate.rs` at the pinned franken_manim revision.

**Link between source and frames:** the three source files were last written at 06:44:50Z and the
frames at 07:19:06Z. The folder holds no source hash, so the link rests on those times and on the
frames matching the source wherever I compared them.

**Limits on the times below:**

- Frame times come from the sheet captions.
- Cue boundaries are inferred from `sampling.json`, not from alignments: 12_1 ends 13.977, 12_2 is
  14.7–31.154, 12_3 is 31.867–49.475, 12_4 is 50.2–86.654, 12_5 is 87.367–99.59, and 12_6 is
  104.567–162.559.
- Word positions inside a sentence are estimates from even word spacing, and are marked as such.

**Independent computation:** I wrote a Python mirror of `turn`, `project`, `hidden` and
`unhidden_runs`, then compared it with the frames. "Drawn length" means the screen length, in scene
units, of the parts of a path the film's own depth test keeps. The mirror lives outside the repo
because this assignment allows one file; the numbers can be reproduced from the source lines cited.

## The five questions asked

| Question | Result | Basis |
|---|---|---|
| Mathematical correctness | **Holds** | See "What held up" |
| Visual motivation | **Fails in three places** | F1 and F3 show something other than their words; F5 and F7 put picture and words apart |
| Solid cube rotations, point correspondence | **Holds about the rod; not shown about the other axis** | Face colours and the gold corner match the mirror on every rod spin; F1 drops the corner in the one spin where it matters |
| Axis exclusion | **Argument valid, picture weak** | F1 |
| Principal-log distinction | **Holds, but on screen too early** | F7; the open-edge marks agree with chapter 10 |

## Findings, most severe first

| # | Severity | Anchor | Defect |
|---|---|---|---|
| F1 | High | Frames 281–288, 124.0–127.5 s; `what_it_forgets.rs:364–375` | Other-axis spin: no trace, no corner, axis reads as a scratch |
| F2 | High | Frames 61–109, 26.0–47.0 s; `:566` | `Log zw` text crosses panel C's border |
| F3 | Medium-high | Frames 81–109, 33.0–47.0 s; `:396–400`, `:580`, `:598` | "One place" over three places |
| F4 | Medium | 71.5–74.5, 108.0–110.5, 111.0–114.5 s; `:708`, `:759` | "Steady" spins are eased |
| F5 | Medium | Frames 149–163, 170–181, 289–306 | Motion ends long before its words |
| F6 | Medium | Frames 222–239, 97.5–104.5 s; `draw.rs:351–385` | Think bar drains under the voice, then sits empty |
| F7 | Medium | Frames 320–322, 143.5–144.5 s; `:869–889` | "No principal log" on screen about 9 s before it is spoken |

### F1. The other-axis spin does not show what its words claim

**Seen.** Frames 281–288 (sheets 070–072), readout t = 0.00 to 1.00:

- No red trace on any frame. The only red is two stubs of the axis.
- Frame 284 (t = 0.52, opened at full size) shows two red stubs, two rod stubs and the solid: no
  trace and no gold dot at all.
- The two circles, the start ring, the end ring and the gold corner all vanish at frame 281, because
  lines 344 and 369 are guarded by `on_rod`.
- At t = 0 the near stub (frame 281) lies inside the top face's outline, so it reads as a scratch on
  the grey face, not as an axis leaving the cube.

**Why, from the source.** Line 367 draws the dragged path through `unhidden_runs`. The path of a
surface point stays inside the solid's current position, so the depth test removes all of it.

| t | Path length | Drawn length |
|---|---|---|
| 0.25 | 0.442 | 0.000 |
| 0.50 | 1.008 | 0.000 |
| 0.75 | 1.531 | 0.000 |
| 1.00 | 1.918 | 0.000 |

The storyboard promises this trace ("drags the top face's centre off the rod (red trace)",
`docs/storyboard.md:165`). The test comment says "so the drag shows", but lines 968–974 check only
the end position of the dot and its visibility at t = 1.

**Why it matters.** `OTHER_AXIS` is an edge axis, so its half turn is a symmetry of the cube. The
outline at t = 1 equals the start, which frames 281 and 287 confirm. A viewer comparing outlines sees
the same cube as after the true half turn. Only the face colours differ (frame 287: green top, grey
left, teal right; true half turn: grey top, purple left, teal right), plus a gold dot that blinks out
mid-spin (hidden on frame 284) and reappears on the left face. That is the one moment the film has to
show that the place reached is different, and the corner that proves it has been removed.

I would keep this axis. Same outline with a different map is a good lesson, once the corner shows it.

**Proposed native fix**, all in `Puzzle::draw`:

1. Remove `on_rod &&` from the `follow` branch at line 369, and colour the moving corner and its trace
   red off the rod. The rings stay where they are: `turn(ROD, PI, CORNER)` does not depend on the pose.
   Under the other axis the corner ends at (−1,−1,−1), screen (−4.46, −1.45), directly below its gold
   end ring at (−4.46, 0.53). This uses the existing depth test:

   | t | Corner path | Drawn length |
   |---|---|---|
   | 0.25 | 0.924 | 0.924 |
   | 0.50 | 2.088 | 1.760 |
   | 0.75 | 3.493 | 1.555 |
   | 1.00 | 4.698 | 2.010 |

   All eight corners miss their half-turn landing place, each by exactly one cube edge.

2. Show the drag in one of two ways. I recommend (a), since it keeps the point the words name.
   - (a) Draw the top centre's path without the depth test, dimmed, as an x-ray:
     `add_runs(s, g, vec![dragged.iter().map(|&p| project(p)).collect()], red, 3.0, 0.6);`
   - (b) Trace a point on the rod's stub, rigid with the cube, such as (0, 1.5, 0). With the depth test
     it draws 2.163 of 2.877 at t = 1 and ends at screen (−4.29, −0.86). The film would need a short
     stem so that the viewer reads the point as part of the cube.

3. Extend only the red line to reach 3.0. Its visible length at t = 0 rises from 0.493 of 3.285 to
   2.094 of 4.928, both ends leave the silhouette, and the ends fall at screen (−4.82, 1.11) and
   (−1.18, −2.21). Neither end is near the text column at x ≥ 0.6.

4. Add a test on drawn length, which is the guard that was missing: for t in {0.25, 0.5, 0.75, 1.0},
   the summed length of `unhidden_runs` on the corner path must exceed 0.9.

### F2. `Log zw = Log z + Log w + 2πik` crosses panel C's right border

**Seen.** On `frame-0100.png` the text runs from x = 383 to x = 618 px and the border is at x = 608 px.
The "k" sits past the line. Sheet 015 shows it from frame 61 (26.0 s), and it stays until the panels
clear at about 47.0 s.

**Why the test missed it.** The width estimator at line 1087 gives 1.99 units for this string; it
renders 2.61 units, 24% low, and the string is not in the test's list anyway.

**Proposed native fix:**

- Move `cut_word` and `rule` (lines 565–566) from x −2.85 to x −3.1. The text then ends at about −0.49
  against the border at −0.35, and the two lines stay aligned. Nothing occupies the extra room; the lap
  labels end at about x −3.4.
- If root prefers to keep x, size 16 gives a width of about 2.32 and an end at about −0.53.
- Add the string to `the_panels_hold_their_pictures`, and use about `0.005 * size` per character at
  line 1087. That figure comes from this one measured string, so check it against the others.

### F3. Panel D's caption contradicts its picture

**Seen.** Frames 80–109. The orbits and a grey dot appear at 32.5 s. Three green dots start at 33.0 s
from three different points, with no start marks, and end at those same three points. The caption
"many flows, one place: all home at t = 1" arrives at about 35.5 s. On `frame-0100.png` the three dots
sit apart on the ellipses and the grey dot, the only thing that looks like a home, is empty.

**Why.** `orbit_point` (lines 396–400) starts each flow at P·(1, 0), which differs per flow, and `home`
(line 580) is the origin, a point no dot ever visits. The mathematics is right: each flow is the
identity at t = 1. The picture just never gives the viewer one place to look at.

**Proposed native fix.** Run all three flows from one shared start:

```rust
const SHARED_START: [f64; 2] = [1.6, 0.0];
fn orbit_point(a: f64, b: f64, tilt: f64, t: f64) -> [f64; 2] {
    model::apply(model::flow(model::ellipse_generator(a, b, tilt), t), SHARED_START)
}
```

- Replace `home` with a ring at `flows_screen(SHARED_START)`.
- The orbits then span screen x 1.34–2.66, y −2.06 to −1.04. That clears the text column at x 3.4 and
  the caption at y −2.65.
- The dots leave together, part, and return together, which is the sentence "whole families of flows
  end in the same place".
- They also meet once at t = 0.5, at −s, because each half-time flow is −I. That is true and harmless;
  the note is here so that nobody reads it as a bug.
- The tests at lines 1042–1045 and 1063 call `orbit_point` and need rechecking.

### F4. The "steady" spins are not steady

**Seen.** Readouts at half-second steps:

| Spin | Time | Fraction of its run, every 0.5 s |
|---|---|---|
| ½ lap | 71.5–74.5 s | 0.00, 0.01, 0.13, 0.40, 0.70, 0.92, 1.00 |
| 1½ laps | 108.0–110.5 s | 0.00, 0.08, 0.30, 0.60, 0.87, 0.99 |
| 2½ laps | 111.0–114.5 s | 0.00, 0.01, 0.11, 0.32, 0.59, 0.83, 0.97, 1.00 |

**Why.** `tween` (`crates/explainer_kit/src/play.rs:49`) sets only `run_time`, so it takes the default
easing.

The mathematics still holds, because the time readout eases along with the cube. But the chapter's key
word is "steady", and the spoken "at half a lap per unit of time" sits over a visibly accelerating spin.

**Proposed native fix.** A chapter-local helper; `play.rs` stays unchanged. The field exists at the pin
(`crates/fmn-mobject/src/animate.rs:61`), and `explainer_corpus/golay_echoes/src/chapters/draw.rs:362`
uses it the same way.

```rust
fn steady(tracker: Mob, value: f64, run_time: f64) -> fmn::Result<AnimBuilder> {
    Ok(tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(run_time),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(value)?)
}
```

Use it at lines 608 (panel D's clock), 708, 759, 781 and 804.

### F5. Motion ends long before its words

Judged from stills against the source's sentence anchors. Fine sync itself remains unreviewed.

- **"A steady spin about that rod … stays put" (12_4 sentence 2, about 61.0–71.5 s).** A quarter lap
  out and back runs 61.0–64.0, then frames 149–163 hold still. The reversal is also not a steady spin.
  Sheet 037 shows the stop at 64.0. *Fix:* replace lines 678–679 with one steady full lap,
  `steady(pz.time, 2.0, window)`, then `set_tracker_value(pz.time, 0.0)`, which is seamless because a
  full lap returns the same pose. Every corner then sweeps its circle for the whole sentence.
- **"after one unit it has gone exactly halfway round its circle" (12_4 sentence 3, about 71.5–80.3 s).**
  The spin finishes at 74.5 (sheet 042) and frames 170–181 hold still, so the arrival comes well before
  the words describing it. *Fix:* run line 708 over the sentence's length minus about 0.8 s.
- **"Last, the axis …" (12_6 sentence 3, about 124.0–137.0 s).** The red spin takes 124.0–127.0, the
  state resets at 128.0, and frames 289–306 hold still for the rest of a 13 s sentence, which by word
  count is where "our half turn leaves only the rod's points in place" falls. *Fix:* run the red spin
  over about 45% of the sentence, hold it, and reset only for the second clause, leaving the top-centre
  dot in plain view for "only the rod's points".

Window helper: `kit.sentence_starts(id)` returns an empty list without a narrator
(`crates/explainer_kit/src/kit.rs:315`), so use `.get(j)` with the current run time as a fallback. Keep
every run time shorter than its window, or the next `until_sentence` returns at once and the picture
trails the voice.

### F6. The think bar drains under the voice, then sits empty

**Seen.**

| Time | State | Evidence |
|---|---|---|
| 97.45 s | Bar appears | Frame 222 at 97.5, nearly full (sheet 055) |
| 99.59 s | Voice ends | Inferred cue boundary |
| 101.45 s | Bar empties | Frame 232 at 101.0 is a short bar (sheet 058) |
| 101.5–104.5 s | A 1 px stub remains | Frames 233–239 |
| 104.567 s | Next cue starts | Inferred cue boundary |

Of the 4.98 s of silence, the bar moves for only 1.86 s. The last 3.1 s shows a stub.

**Why.** `predict` starts its updater when called, at `what_it_forgets.rs:744`, but
`kit.music_breath` first waits for the line to finish (`narration.rs:1302–1303`). The doc comment at
`draw.rs:348–350` says the line finishes first, which the code does not do. The stub is
`.max(1e-3)` at `draw.rs:357`.

**Proposed native fix,** in `predict`:

- Call `kit.hold(stage)?;` before creating the bar (`crates/explainer_kit/src/kit.rs:304`). By my
  reading this moves no cue: `hold` waits at most `BREATH`, and the window's own wait then tops up to
  `BREATH`, so the window opens at the same frame.
- Draw nothing when `f <= 0.0` instead of a stub.

The helper has 13 call sites, 12 of them in other chapters. I did not review those pictures. Whichever
of them is still speaking when `predict` is called has this same defect, and this one change covers
them all, so they need a look at render.

**Passed:** the answer does not leak. Nothing is revealed before 104.567, and the first new spin starts
at about 108.0.

### F7. The principal-log conclusion is on screen before it is spoken

**Seen.** 12_6 sentence 5 ("The two slowest … no principal logarithm") runs about 141.7–154.3 s. The
whole picture builds in under 3 s and then holds still until frame 342:

| Frame | Time | Appears |
|---|---|---|
| 317 | 142.0 | Turning line |
| 318 | 142.5 | Window |
| 319 | 143.0 | Dots and rings |
| 320 | 143.5 | `tie`, "the slowest two, ±π, tie on the window's open edge" |
| 321–322 | 144.0–144.5 | `precisely`, ending "so no principal log" |

Assuming even word spacing, "the very edge" falls near 147.8 s, so `tie` leads it by about 4 s, and "no
principal logarithm" falls near 153.5 s, so the on-screen conclusion leads it by about 9 s.

**Proposed native fix.** At line 827 show the line, window and dots. Show `tie` at about 45% of the
sentence and `precisely` at sentence 6, just before `convention`. No word changes.

## What held up

**Mathematics, each checked independently of the film's facts:**

- All 8 corners lie at radius √2 about the rod.
- A scan at 1/48-lap steps over ±4 laps finds only odd half laps landing on the half turn H. Rates 0.5,
  1.5, 2.5 and −0.5 all send corner (1,1,1) to (−1,1,−1).
- **Axis exclusion:** (H−I)x = 0 gives x = (0,y,0), so H fixes exactly the rod. A spin fixes its own axis
  pointwise, so an axis n ≠ ±rod would need H(n) = n, which fails. At the edges: a whole-lap spin about
  another axis ends at the identity, not at H, so the argument has no gap there.
- Log(−1) = iπ, consistent with 07_6.
- diag(−2, −½) has no real logarithm, since its negative eigenvalues are distinct and each has one
  Jordan block. The spoken "real" is needed and present.
- **Principal log:** if A has eigenvalue −1, every logarithm has an eigenvalue with imaginary part an odd
  multiple of π, so none lies in the open strip. This is the step in `docs/sources.md:51–54`. I checked
  the citation "Higham §4 Fact 1" only against `docs/sources.md`; I did not open Higham.

**Pictures:**

- The dial and the log ruler are drawn to scale on `frame-0100.png`: the needle measures 137.4° clockwise
  against 136.9° for 2.4, and ruler marks 2 and 3 sit where log spacing puts them.
- Face colours through every rod spin match the mirror: quarter turn teal left and green right; half turn
  purple left and teal right.
- The gold corner takes the back arc for positive rates and the near arc for the other way (frames 270–275,
  then 289+). The readout "−0.30, −0.44, −0.50" is correct.
- The final R pose in F1 is right: `R(x,y,z) = (−y,−x,−z)` sends grey to the left and green to the top
  (frame 287). The rod stays fixed through the solid during the red spin.
- The turning line's marks agree with chapter 10, where red rings mark the window's open ends
  (`doing_nothing.rs:293–294`). In chapter 12 the purple dot inside the ring is the spin and the ring is
  the open end, so this is not a filled-endpoint error. Ticks at 0 and ±2π correctly carry no dot.
- The `precisely` line fits (54–907 of 960 px, frames 321+). `tie`, `convention` and `precisely` do not
  overlap. The `so` line sits clear of the chips.
- Crossfades at frames 120–121, 269, 281 and 289 are 0.3–0.4 s swaps, as root said. Not defects.

## Lower-priority notes

None of these blocks acceptance.

- **Hard resets:** `set_tracker_value` jumps the pose at 107.5→108.0, 110.5→111.0, 117.5→118.0, 123.5→124.0
  and 127.5→128.0. They read as cuts on stills; judge in motion.
- **"Its own circle" (spoken, 12_4):** eight corners share two circles, four on each. The on-screen wording
  at line 671, "each corner keeps to a circle", is exact. Fixing the voice would need a new take: root's
  call. I would leave it.
- **"The whole family" (spoken, 12_6):** this has to mean the family of spins, as the on-screen "spins ending
  here" says. Two edges worth one sentence in the description, with no script change:
  - Non-rigid real logarithms of H exist. The block [[0, −2π], [π/2, 0]] in the x–z plane exponentiates to −I
    there, and its flow is not a rotation.
  - The number's convention applied per eigenvalue gives iπP with P = diag(1,0,1), a complex logarithm of H and
    not a real spin. That is why the convention does not break the tie here.
- **"Fold the ruler" (12_2):** the dial fades in at 14.7–15.5 with no fold on the sampled frames.
- **"The keyboard" (12_1):** no picture for it during the 8.5–14.5 hold.
- **Small crowding:** "9 10" on panel A's ruler nearly touch on `frame-0100.png`. The cut is added after the −1
  target and crosses it. The turning line's dim 16 pt tick labels should be checked at delivery size.
- **Title-only hold:** 50.7–53.0.
- **Frame 111 (48.0 s):** faint ghosts of the lower panels, gone by frame 112. Every panel is in the fade list,
  so this is probably residue from the preview encode. Check it on a final encode; I do not claim a source
  defect.

## Suggested order

1. F2: two numbers and a test line.
2. F1: the branch change and the drawn-length test.
3. F3: `orbit_point` and the ring.
4. F4 and F5 together, since both change the same tween calls.
5. F7, then F6, which reaches 12 other chapters.

## Not reviewed

- Motion between samples, listening, voice and music balance, fine sync, independent AV. Half-second samples
  cannot show them.
- Any chapter other than 12, beyond the chapter 10 marks cited.
- Whether the proposed code compiles. Nothing was built.
- Higham, except through `docs/sources.md`.
- 4K rendering of any of this. The frames are 1280×720 samples.
