# inspection_paradox: independent review (WildBass)

Bead `mx-video-program-program-87do.13.2`, review of GoldGate's writing
handoff for `.13.1`. Reviewer WildBass (pane %16), 2026-10-24. This is a
writing-stage review. It finds nothing about the Rust build, frames, voice or
the full film, because none of those has run.

**Verdict.** The mathematics of the draft is right everywhere I checked, the
continuing example is well chosen and the film earns its formula. I found
one source defect worth fixing before root's native batch (S1), one
mathematical claim that rests on an unstated assumption (M1), and a handful
of small wording and source points. Nothing here calls for a restructure or
for cutting length.

## Snapshot reviewed

`narration_lab/third-wave-20261024/root/intake-0550/inspection_paradox/`,
captured `2026-10-24T05:50:51.885799+00:00` per `inputs.json` in the same
intake folder. All 18 listed files match their sha256 and byte counts
(`checks.py`, below). Leading 16 hex digits:

| file | sha256 | bytes |
|---|---|---|
| `Cargo.toml` | `7433db1ff9ce72c6` | 805 |
| `lexicon.json` | `cae1bd9d92a36257` | 1718 |
| `script.tsv` | `e598c79ca194c19f` | 1214 |
| `oracle.py` | `54cdce31f5863ec6` | 26240 |
| `docs/storyboard.md` | `d6c5ba48f6ab9a61` | 9764 |
| `docs/pause-overrides.json` | `fc209b4347dcb18a` | 1610 |
| `docs/WORK_STATUS.md` | `10437ada55fc0612` | 8387 |
| `docs/brief.md` | `0e042d47dab10b63` | 15111 |
| `docs/storyboard-draft.md` | `2f0369f889b2d238` | 53771 |
| `src/palette.rs` | `aeb18f555f303cd4` | 1544 |
| `src/facts.rs` | `0db9cf5351b4fd6f` | 18617 |
| `src/score.rs` | `8bb4cb277d6a9ae0` | 4050 |
| `src/main.rs` | `4e0d727894d5883a` | 8005 |
| `src/model.rs` | `e34548a424f986c7` | 20652 |
| `src/chapters/the_stop.rs` | `b676639da4047a82` | 19331 |
| `src/chapters/draw.rs` | `d33cb70fff33d3d4` | 17994 |
| `src/chapters/mod.rs` | `95fb7acfe896ceb6` | 1232 |
| `src/chapters/glyphs.rs` | `892b8e2b81a55ba6` | 893 |

The full hashes are in `inputs.json`; the check compares all 64 digits.

**One unlisted file has appeared in the snapshot since capture:**
`.ee/hook-state/session_start_orient.last.json` (181 bytes, modified
2026-10-24 05:58:47 UTC, content
`{"outcome":"command_error","surface":"session_start_orient",...}`). It is
hook state from a session-start hook, written after capture, most likely
when my own session's working directory was the snapshot folder. It is not
review input and none of the 18 listed files changed. I did not delete it
(no permission to delete); root may want to remove it or exclude `.ee/`.

Read in full: `docs/storyboard-draft.md` (all twelve chapters, 86 beats),
`docs/brief.md`, `docs/storyboard.md`, `script.tsv`, `oracle.py`,
`src/model.rs`, `src/facts.rs`, `src/main.rs`, `src/chapters/the_stop.rs`,
`src/chapters/draw.rs`, `src/chapters/mod.rs`, `src/chapters/glyphs.rs`,
`src/palette.rs`, `Cargo.toml`, `docs/pause-overrides.json`, `lexicon.json`,
`docs/WORK_STATUS.md`. Read the head only: `src/score.rs`. For the kit
behaviour behind S1 and S2 I read `crates/explainer_kit/src/kit.rs`
(`live_bar`, `finish`, `hold`) and `crates/explainer_kit/src/cli.rs` in the
working tree.

## Commands actually run

From the repo root:

```text
python3 -I -B docs/video-program/third-wave-20261024/reviews/inspection-paradox-WildBass/checks.py \
    narration_lab/third-wave-20261024/root/intake-0550
  exit 1: "1 failure(s)". The one FAIL is the unlisted .ee/ hook file above.
  Every arithmetic check printed "ok"; six "finding" lines, quoted below.
  It also runs the snapshot's oracle.py as a separate process:
  "ok   oracle (exit 0, 155 facts) agrees on 17 reviewed keys".
python3 -I -B oracle.py            (inside the snapshot; read-only)
  exit 0, 155 facts
curl -A "<project UA>" https://export.arxiv.org/api/query?id_list=2004.14081
curl -A "<project UA>" https://api.crossref.org/works/10.1007/BF00991287
WebFetch https://web.mit.edu/urban_or_book/www/book/chapter2/2.13.html
```

`checks.py` imports nothing from the author's code. It rebuilds the
timetable from blue `0, 12, …` and red `d` later, and computes every number
with `fractions.Fraction`. Its finding lines, verbatim:

```text
finding 07_5 slider: of the 121 offsets d = 0.0, 0.1, ..., 12.0, only 13 give a whole-second wait (e.g. d = 2.5 gives 241.25 s); model::offset_wait takes i64 and clock() refuses fractions, so a live sweep needs a rational offset and an explicit rounding path, or integer steps
finding 08_8 card wording: 2,375 counts multisets (repeats allowed, order ignored, not all zero); true sets would be 1091 and ordered lists 30936
finding K07 finite-moment control: Pareto gaps (alpha 1.5, mean 6, infinite mean square): the running wait estimate at n = 1e3..1e6 is 10.6, 38.2, 52.3, 230.6 min; it has no limit, so K07's finite-mean-square condition is needed (correctly stated)
finding K07 vs K05/K09 wording: turning up at :02:30 every day is independent of a fixed timetable, yet waits 9:30 every day; uniform turning-up waits 4:20. K05 and K09 say 'uniformly / spread evenly in time'; K07 says only 'independently of the buses'
finding 11_6 even split: under the ready-time model the share standing in long gaps stays 5/6 (not 1/2); the even split needs at least 20 of 60 passengers to pass up a bus they could make, which the draft does not state
finding 01_4 bar rates: bars are to scale (0.20 units/min: 0.10 and 1.90 wide) but drain at 0.50 and 3.65 wait-minutes per second, 7.3x apart; the storyboard says 'shrinks the same way'
```

**Not run, and not claimed:** any Cargo build, `cargo test`, clippy or
rustfmt; `facts-check`; the glyph probe; the layout lint; any render or
contact sheet; any narration, TTS or listening; any English-accent judgment.
The Rust findings below come from reading the source.

## What I checked and found correct

Each line is a claim in the draft and the value `checks.py` computed for
it independently.

| claim (cue) | independent result |
|---|---|
| tags 10, 2, 10, 2, … average 6 (02_2) | tags from the previous bus, :00 after :50; mean 6; half say ten |
| five in six (02_4, 02_5, 02_6) | 50 of 60 minutes in long gaps; 50 of 60 midpoint dots |
| 9½ … ½ = 50, ½ + 1½ = 2, 52 ÷ 12 = 4:20 (04_3–04_7) | exact |
| midpoint passengers are exact (04, K02) | 260 passenger-min = triangle area 260 min²; also exact for 1/2, 1/3, 1/7-minute midpoints; a 1/1000-min Riemann sum gives 260.03 |
| start-of-minute grid is not a random moment (oracle negative control) | start gives 290 (4:50), end gives 230 (3:50) |
| units (08_2, 11_3) | 260 passenger-min ÷ 60 passengers = 52 min² ÷ 12 min = 13/3 min |
| bars hold exactly the triangle (05_4) | each bar equals its minute's trapezoid; the two corners are ⅛ min² each |
| 15 upright + 10 inverted = 25 (05_7) | n(n+1)/2 + n(n−1)/2 = n² for n = 1..12 |
| passengers' gap 8:40, since-last 4:20 (06_4, 06_7) | Σg²/Σg = 26/3; mean age 13/3 |
| stretch shares 6..10 of 12, buses ½ (03_2–03_4) | (12 − d)/12 and 5 of 10 enders, d = 6..1 |
| d = 0 edge: all passengers in 12-minute gaps, sign still 6 (03_6, 07_6) | zero gaps counted: mean gap 6, wait 6; merged: mean gap 12, wait 6 = half |
| U from 3:00 to 6:00 (07_5–07_7) | (d² + (12 − d)²)/24 for d = 0..12 by midpoints and by gaps: 6:00, 5:05, 4:20, 3:45, 3:20, 3:05, 3:00, …, 6:00 |
| 52 = 36 + 16, 3:00 + 1:20 (08_4, 08_5) | spread 16, 16/12 = 1:20 |
| never below half, equal only for even gaps (08_8, F-X1) | 2,375 cases, 0 violations, 48 equalities, equality iff all equal |
| random buses wait 6, your gap 12 (09_4–09_7) | quadrature E[G] = 6.000000, E[G²] = 72.0000; seeded simulation 5.982 and 11.983; P(wait > 6 \| last bus > 6 ago) 0.3661 vs P(wait > 6) 0.3651 |
| clumped 0, 0, 18 waits 9 (09_8) | exact |
| 104 ÷ 12 = 8⅔ (10_4); profiler ½ / ⅙ and ½ / ⅚ (10_6) | exact |
| timetable reader waits 1 minute (11_4, 11_5) | exact |
| 4 and 8: 8 + 32 = 40, 3:20, long share ⅔ (12_2–12_4) | exact |
| oracle agreement | 17 reviewed keys equal, including `F-O1.wait_clocks`, `F-X1.*`, `F-K2.*` |

Bus versus time sampling is the film's core distinction, and the draft
keeps it exact. Tags are one per bus and dots are one per minute, both on
the same timetable. The two counts agree on even gaps (03_1, 03_2) and
part on uneven ones.

The scope cards are right: K01, K05 and K09 all say that arrival is uniform
in time. K06 labels Poisson as a model with long-run averages. K07's
finite-mean-square condition is necessary, as the Pareto control shows. The
d = 0 and d = 12 endpoints are handled, and the zero-gap counting choice is
said out loud.

## Mathematical findings

**M1 (medium): the planners' "six to a bus, half in a short gap" depends
on an unstated even split.**
- Where:
  - `docs/storyboard-draft.md:506–507` (11_6, 11_7) and notes `:514–517`;
  - `src/model.rs:385–394` (`planned_arrivals`, "shared evenly among the
    buses") and its test `:600–608`;
  - `src/facts.rs:367–384` (F-K2) and `oracle.py:446–455`;
  - `docs/brief.md:70–72`.
- What: the voice says "All sixty would crowd in a minute before a bus, six
  to a bus, and half of them would be standing in a short gap". The bars
  then go from 5/6 to 1/2. That follows only if planners share themselves
  evenly over the buses. The oracle builds the same split, so its
  "negative control" agrees with the model by construction.
- The natural reading of the film's own dots: each dot is when that
  passenger is ready, and they plan to stand at the stop one minute before
  the first bus they can still make. Then 10 stand before each blue bus and
  2 before each red. Every wait is still one minute, but 5/6 still stand in
  long gaps. Reaching six per bus needs at least 20 of the 60 to pass up a
  bus they could make.
- Consequence: the viewer can leave thinking that planning moves people out
  of long gaps, and the notes call keeping 5/6 the "attractive but wrong
  step". Under the natural model 5/6 is right. What fails is the triangles,
  because nobody is spread evenly *within* a gap any more.
- Smallest correction: keep "Every wait would be one minute" (true under
  both models). In 11_6, slide each dot forward to one minute before the
  first bus it can catch, giving stacks of 10 and 2, the same objects
  moving only forward. The share bar stays at 5/6 while every wait becomes
  1 minute. Then the line is that the triangles stop describing passengers
  because nobody arrives spread out inside a gap any more. If the even
  split is kept instead, say it ("if the planners spread themselves evenly
  over the buses"). Either way, align `planned_arrivals`, F-K2, the oracle
  and brief.md:70–72 with the chosen model.

**M2 (low): K07 drops "spread evenly in time".**
`docs/storyboard-draft.md:452–454` (09_8, K07): "the formula holds for
passengers who turn up independently of the buses, when the gaps have a
long-run average and spread". Against a fixed timetable, turning up at
:02:30 every day is independent of the buses, yet that passenger waits 9:30
every day, not 4:20. K05 and K09 already carry the condition. It matters
more here because the film's anchor is one commuter, and a commuter who is
ready at the same minute every morning is exactly this counterexample. Add
"and turn up spread evenly in time" to K07, matching K05.

**M3 (low): the 08_8 card calls multisets "sets".**
`docs/storyboard-draft.md:408`: "every set of up to 4 gaps, each 0 to 12
min: 2,375 sets". 2,375 counts multisets: repeats allowed, order ignored,
not all zero. True sets would be 1,091 and ordered lists 30,936. The
oracle's own comment (`oracle.py:38`) already says "multiset". Smallest
fix: "every choice of up to 4 gaps (repeats allowed), each 0 to 12 min:
2,375".

**M4 (low, root's packaging call): the title is literally false at the
film's own numbers.** `src/chapters/glyphs.rs:11` and 01_7
(`docs/storyboard-draft.md:184`): "Why You Wait Longer Than the Average".
Your average wait, 4:20, is shorter than the sign's average gap of 6:00.
Poisson waits equal it, and only spreads past exponential exceed it. What
is longer than the average is the gap you land in (8:40) and the wait
compared with half the gap. The brief records that root chose this title
(Mail 6766, `docs/brief.md:7`), so I flag only the mismatch. The thumbnail
text "WHOSE AVERAGE?" in `packaging-drafts.md` frames the same question
accurately.

## Teaching findings

**T1 (low to medium): the commuter's stated need is never paid off.**
`docs/storyboard-draft.md:178, 181` / `docs/storyboard.md:101, 104` (01_1,
01_4): "Your train won't wait for you, so you'd like to know how long
you'll stand here." A train deadline is answered by the worst case, which
the timetable shows directly: just after a red bus, 10 minutes. The average
doesn't answer it. The train never returns after 01_4 (I searched the
draft). The setup is otherwise strong, so the fix is small. Either:
- aim the need at what the film answers ("…so you'd like to know how long
  you usually stand here", or the time lost over a year of mornings); or
- pay it off once in 11, e.g. "For the train, allow the full ten minutes.
  The four twenty is what the stop costs you on an average morning."

**T2 (low): 04_8 changes where the guess came from.**
`docs/storyboard-draft.md:285`: "The guess of three came from averaging one
and five". In 01_5 the guess came from halving six. The two are equal (the
bus-weighted mean of the half-gaps is half the bus-weighted mean gap), but
a viewer who halved six won't recognise their reasoning. Smallest fix:
"Half of six is the same as averaging one and five, as if both gaps had
the same number of passengers."

**What already works (no change asked).**
- One concrete, persistent world: you, the two ghosts, the 60 dots, the
  tags, the 2/10 brackets and the sign.
- The two mornings return as the first and last gold passengers in 04_4.
- Every chapter has a computed experiment that changes one cause (03 moves
  only the red bus, 07 sweeps the same slide for the wait, 12 gives a
  transfer puzzle).
- Predictions with nothing previewed: 01_5, 02_4, 03_5, 04_5, 05_6, 06_3,
  07_3, 08_7, 09_3, 10_3, 11_4, 12_2.
- Numbers come before letters in 08, and the formula is earned from
  triangles the viewer has already seen tiled.
- The negative controls are real: even gaps agree (03_1, 07_4), and the
  start-of-minute grid is in the oracle.
- The commuter's need and the two mornings (01_4) do earn the guess and
  the reveal before any formula.
- The film's length is used for intuition, not padding. I would not
  compress it.

## Source findings

**S1 (medium): chapter 01 never calls `kit.finish`.**
`src/chapters/the_stop.rs:421–422` ends
`fade_out_all(stage, &title, 0.5)?; Ok(())`. `Kit::finish`
(`crates/explainer_kit/src/kit.rs:201–231`) does three things:
- runs the closing `hold` and its lint checkpoint;
- lays and length-checks the music stem;
- records `kit.finished`, the chapter timeline.

Without it the CLI writes no `timeline.tsv` and prints `note …: no timeline
(the chapter does not call kit.finish)` (`cli.rs:607`). The scored music
bed has nothing to align to. The headless test
`the_chapter_runs_and_says_its_lines_in_order` (`the_stop.rs:439–451`)
checks only the order of the lines, so it would pass anyway. 224
`kit.finish(stage)` calls exist across the corpus chapters.

Smallest fix: make `kit.finish(stage)` the last expression in place of
`Ok(())`, and add `assert!(scene.kit.finished.borrow().is_some())` to the
test.

**S2 (low): 01_4 bars against the storyboard.**
`src/chapters/the_stop.rs:71–72, 192–207, 310–316` against
`docs/storyboard.md:104` / draft `:181`. The behaviour matches:
- the ghost appears, and a gold bar hangs left from the :12 bus (a
  negative `live_bar` tracker) and grows back to you;
- "wait 0:30" / "wait 9:30" sits under the ghost;
- the bar drains linearly (`steady`);
- both bars and labels leave together, and the ghosts dim to 0.45.

Two mismatches:
- "shrinks the same way": the bars are to scale (0.10 and 1.90 units), but
  they drain at 0.50 and 3.65 wait-minutes per second (`LUCKY_RT` 1.0 s,
  `UNLUCKY_RT` 2.6 s). Matching the rates is impractical: at the unlucky
  rate the lucky bar lasts 0.14 s, and at the lucky rate the unlucky bar
  takes 19 s. Fix the words: "shrinks steadily to the bus".
- The lucky label stays on screen through the second morning. It sits at
  x = −3.70, under the right end of the unlucky bar (x = −5.50 … −3.60), so
  for a moment "wait 0:30" can read as a label on the long bar. Either fade
  the lucky sliver and label before the second morning, or have the
  storyboard say they stay. Actual frames decide.

**S3 (low, future scene): the 07_5 live slider can't be driven by the
model as written.** `docs/storyboard-draft.md:373, 386–388` (a continuous
sweep with a live "m:ss" readout that "rounds to whole seconds"), against
`src/model.rs:185–194` (`clock` refuses non-whole seconds, "no rounding
anywhere") and `:354–357` (`offset_wait(d: i64)`). Only 13 of the 121
tenth-minute offsets give whole seconds. Before 07 is implemented, either
step the slider in whole minutes (as 03_3 already does) or add a
rational-offset wait with one explicit, tested rounding function for the
readout.

**S4 (cosmetic): the oracle's section comments carry draft-1 chapter
numbers.**
- `oracle.py:43` "(chapter 07)" is 09; `:48` "Chapter 08" is 10;
- `:253` (one person a minute) is 04; `:269` "# 03: the triangles" is 05;
- `:354` "# 05" is 07; `:372` and `:387` "# 06" are 08;
- `:401` and `:414` "# 07" are 09; `:422` "# 08" is 10.

Renumbering helps whoever traces a fact to its chapter.

## Primary references

| source | what I checked | result |
|---|---|---|
| Larson & Odoni, *Urban Operations Research* §2.13 (MIT online edition) | WebFetch extraction of the quoted sentences; I did not view the equation images | The brief's six quotes appear as quoted ("favors gaps of longer duration in direct proportion to their duration"; "depends only on the mean and variance"; "can in no way depend on the past history"; "need not be mutually independent…"; "E[V] = T0/2"; "a 50 percent increase over the case of perfect headways"). The section assumes F_Y(0) = 0 (no zero gaps). The film's d = 0 edge goes beyond that, harmlessly, because it counts the zero gaps explicitly |
| Feld & Grofman 1977 | Crossref metadata for DOI 10.1007/BF00991287 | "Variation in class size, the class size paradox, and some consequences for students", Scott L. Feld and Bernard Grofman, *Research in Higher Education* 6(3), 215–222, 1977. This matches 10_5's card. I did not read the article; the page-216 quote rests on the author's reading |
| Masuda & Hiraoka, arXiv:2004.14081 | full abstract via the arXiv export API | "We present an English translation and discussion of an essay that a Japanese physicist, Torahiko Terada, wrote in 1922. In the essay, he described the waiting-time paradox, also called the bus paradox… He also observed and analyzed data on Tokyo City trams…". Journal ref *Northeast Journal of Complex Systems* 2(1), 1 (2020). This supports the 10_7 card ("English translation and discussion") and the voice, including "noticed it over a century ago" |
| Feller Vol. II §I.4 | not checked | Still unverified, as the brief says; read it before the 09_7 search card is locked |

## Gates still open

Rust build and tests, `facts-check`, the glyph probe (title and digits), the
layout lint at the K01 lower-third and the sign card, rendered frames for
01_4's bars and labels, narration timing for the four authored pauses,
listening, and pronunciation and accent qualification (Terada, Grofman,
"millisecond" in `lexicon.json`). None of these is evidenced by this
review.
