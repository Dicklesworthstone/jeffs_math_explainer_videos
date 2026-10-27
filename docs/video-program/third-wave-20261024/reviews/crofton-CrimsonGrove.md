# Crofton: independent review (CrimsonGrove)

Bead `mx-video-program-program-87do.22.2`, reviewing the lane of
`mx-video-program-program-87do.22.1` (owner GentleLantern). Reviewed:
the root intake snapshot `narration_lab/third-wave-20261024/root/tend-0610/intake/crofton`.
All 19 files match `tend-0610/intake.json` (0 mismatches, 0 unlisted;
rechecked 2026-10-24 before I wrote this). The live `explainer_corpus/crofton`
matches the intake file for file. Key hashes (sha256, first 16 hex):
`oracle.py` 8f80b3829bd545ab, `test_oracle.py` 01c05c1167ff1cba,
`docs/storyboard-draft.md` 711cc2d78ae98170, `docs/script-draft.tsv`
71d4d865e506bfd5, `src/model.rs` 78eb4aea0a126114, `src/facts.rs`
0b3094aa33864c66, `src/chapters/the_question.rs` 9321b557ea62daba,
`src/main.rs` f3cdfa6dc204e589.

Read in full: the brief, WORK_STATUS, both storyboards (the 28-chapter draft,
718 lines, and the production chapter 01), both script TSVs, `oracle.py`,
`test_oracle.py`, `model.rs`, `facts.rs`, `main.rs`, `the_question.rs`,
`draw.rs`, `glyphs.rs`, `mod.rs`, `palette.rs`, `score.rs`, `lexicon.json`,
`Cargo.toml`. Status: review delivered, `batch_pending`. I close nothing.
Root decides.

## What I ran

All from the repository root.

1. **My own recomputation**, `reviews/crofton-CrimsonGrove/independent_check.py`
   (standard library only, sha256 ebccfe9c2343b9d4…). It takes only the film's
   data (the eleven pieces, the card, the sticks, the gap) and imports and
   copies nothing from `oracle.py`. Its methods differ from the oracle's:
   - It finds crossings as meeting points by intersecting each line with each
     piece directly. A tangent counts once and a joint once.
   - Slide averages are exact. For each direction, every interval between
     change heights is weighted by its length.
   - One drop's count distribution is exact over slide × 3,600 directions.
     The ten-drop distribution is an exact convolution.
   - The dividers walk by scanning and bisection.
   - Pins use Python's own generator.

   `python3 -I -B docs/video-program/third-wave-20261024/reviews/crofton-CrimsonGrove/independent_check.py`
   → exit 1 (it exits 1 on any DIFF), 60 `ok`, 6 `DIFF`, 18 `note`, about 25 s.
   The full output is saved beside it as `independent_check.out`.
2. **The lane's oracle and tests**, re-executed (not independent), from a
   byte-identical scratch copy, because the oracle has no output-path option
   and must not write into the snapshot:
   `uv run --no-project --quiet --with numpy python3 -I -B <copy>/oracle.py`
   → exit 0, stderr ending `# 182 facts, 0 failed cross-checks`. A second
   run printed the same facts byte for byte.
   `… <copy>/test_oracle.py` → `Ran 33 tests in 0.435s`, `OK`, exit 0.
3. **Root's evidence, kept as it was:** `tend-0539/crofton-spacing01`
   ran the six `Spacing` tests, exit 0, against an **older** `oracle.py`
   (baf5532435e3056f…) with the same `test_oracle.py` (01c05c1167ff1cba…). The
   six tests are among the 33 that pass above against the current oracle. I
   did not rerun the class by itself.

Not done: no Cargo, rustc or clippy-driver, no TTS, no render, no listening.
I ran no rustc for crofton at all, so I have no direct rustc evidence for this
lane. My earlier direct rustc runs were on Shamir and say nothing about crofton.

## Findings, most severe first

### F1. 19_1 is false on the chart it draws (fix)

"Most lines miss it entirely" is false on the chart as drawn. The legend
"deepening blue for 1 to 12 crossings" also mixes one line's count with a
whole sheet's.

- Chart area by **a single line's** count, over 3,600 directions, in cm × share
  of the half turn: 1: 7.4134, 2: 0.7101, 3: 1.8603, 4: 0.2470, 5: 0.1040,
  6: 0.0850. No positive-area patch has more than **6**. The widest
  6-window (0.47 cm at 7.775°) recounts as 6 on a 20,000-per-cm polyline.
  Twelve is the largest count of a whole sheet at 90° (`turn.90` 7..12),
  not of one line, so shades 7 to 12 would never appear.
- Lines meeting the coast fill 32.7350 cm·rad of the chart. That equals the
  perimeter of the coast's convex hull, computed separately (32.7350), as
  Cauchy says it must. That is **65.5%** of the lines meeting the card (the
  17_6 region, area 50) and 56.6% of the whole 18.4 cm × half-turn chart. Most
  lines **meet** the coast.
- Consistency check: Σ count × area = 16.4330 = the mean travel, so 19_4's
  51.63 = π × 16.433 is right.
- Suggested words: "Many lines miss it. Most of the lines that meet it cross
  just once (71%), and a few cross as many as six times." Legend: 1 to 6. The
  oracle should pin `chart.line_max` and the miss share so the words are
  facts.

### F2. Said and shown products don't match their rounded operands (fix)

AGENTS: "What is said equals what is shown, rounded the same way."

| line | said / shown | arithmetic at the shown precision | exact |
|---|---|---|---|
| 11_4, also 23_2 and 28_3 | "8.13 × π = 25.53" | 8.13π = 25.541 | 8.12729π = 25.5326 |
| 11_5 (voice) | "eight point two two … times pi … twenty-five point eight one" | 8.22π = 25.824 | 8.21649π = 25.8129 |
| 11_5 (picture column) | "16.43 ÷ 2 = 8.22 crossings, × π = 25.81 cm" | the note says the card shows 8.2165 in small print, but the row still reads 8.22 | |
| 19_4 | "51.63 = 2 × 25.81" | 2 × 25.81 = 51.62 | 2L = 51.6257 |
| 11_1 table | 45°: travel 17.49 → average 8.74 | 17.49/2 = 8.745 | 17.4852/2 = 8.7426 |
| 28_3 | "twenty-five point eight one, exactly" | a rounded figure called exact | |

Suggested repairs: carry three places where a product is formed (8.127 × π =
25.53, 8.216 × π = 25.81). Or say "about" and drop the product. For 19_4, say
"51.63, twice the coast's length" without "2 × 25.81". For 28_3, say "gives
the length itself". A facts test that recomputes each displayed product from
its displayed operands would catch the whole class.

### F3. Sampled spreads are presented as properties (fix)

The spreads are the seeded 1,000-drop sample's standard deviations, but they
are said as properties. Exact values, quadrature over slide and direction, then
checked against 20,000 drops from Python's generator:

| claim | film | exact | 20,000 random drops |
|---|---|---|---|
| one drop's spread, 2 cm gap (21_4, 25_3) | 4.3 | **4.4319** | 4.423 |
| 4 cm gap (25_3) | 8.2 | **8.1200** | |
| 8 cm gap (25_3) | 15.9 | **14.9313** | 14.872 |
| drops for a 0.1 cm spread (22_3) | 1,868, "about nineteen hundred" | **1,965**, about two thousand | |

- 21_4 scopes its number to the sample ("In a thousand drops"). That's fine,
  apart from the derived uses below.
- 25_3 states a property ("One drop's typical miss grows from … to fifteen
  point nine"), and 15.9 is 6.5% above the exact 14.93.
- 22_3 answers a prediction with a sample-derived figure. The exact answer
  rounds to two thousand, not nineteen hundred.
- These agree exactly: the ten-drop spread 1.401 (film 1.4), the chance a
  ten-drop estimate misses by over 1 cm, **43.31%** by exact convolution (film
  43% from 1,000 repeats), and the thousand-drop spread 0.1401 (film 0.14).
- Also exact: a single drop can meet the sheet **13** times (chance 0.036%;
  window 62.475°, slides 0.198–0.240; the 20,000-per-cm polyline recount also
  gives 13). 21_4's "anywhere from five to twelve" is true of the sample only.
  Don't generalise it.
- Repair: compute the exact count distribution in `model.rs`. Per direction
  the count is piecewise constant in the slide, so this is cheap. Say the exact
  spreads, or round honestly ("about four and a half", "about fifteen", "about
  two thousand"). Keep the seeded draws for the pictures.

### F4. 01_7 "A smaller step cuts less off" (wording)

Step by step on an arc, this is true. As a claim about the total walk, it fails
on this coast:

- 1.90 cm → 21.630, 2.00 → 22.641, 2.05 → 23.213, 2.10 → 19.435 cm.
- Over steps 0.25 to 4.00 in 0.05 increments, 46 of the 75 increments
  **raise** the total.
- Negative control: on a square outline of side 2, a 1.9 cm step walks 7.70,
  against 8.00 for a 2 cm step.
- `main.rs` tests only 1 cm against 2 cm.
- 25_6's values (4: 18.66, 2: 22.64, 1: 25.14, 0.5: 25.64; halving +1.99%)
  are right. Suggested 01_7: "Each smaller step cuts less off its bend, but it
  needs more steps…"

### F5. The six-direction bound: −2.26% cited, −2.30% computed

The lane flagged this and left it open. My investigation:

- For m fixed families of parallel lines, each averaged over its slides, a
  straight stick at angle α is estimated as (π/2m) Σₖ |sin(α − kπ/m)| × its
  length. The band runs from t·cot t (stick along a family) to t / sin t
  (midway), with t = π/2m.
- For m = 6 in closed form: low (2 + √3)π/12 − 1 = **−2.2951%**; high
  (π/12)/sin(π/12) − 1 = **+1.1515%**.
- Any curve's estimate is a length-weighted average of its pieces', so the
  band holds for every curve. The coast's 25.7269 (−0.333%) lies inside.
- The bands for m = 1, 2, 4 match the draft (−100/+57, −21.5/+11.1, −5.2/+2.6).
- Steinerberger (arXiv 2603.27807, §1.4) gives "between −2.26% and 1.15%" and
  doesn't define the percentage. I read it through the HTML text extraction
  only, not the PDF. His +1.15% is exactly t/sin t − 1, so he uses the same
  normalisation. Under that normalisation −2.26% isn't reachable:
  - π as 22/7 gives −2.2558% but +1.19%;
  - error relative to the estimate gives −2.35%;
  - rounding the 3.82 mm spacing gives −2.303%/+1.144%;
  - the reciprocal convention gives −1.14%/+2.35%.
- The low end is a V-shaped minimum with slope π/12 per radian. Reading
  −2.26% needs the stick 0.077° off a family, as a sampled angle grid would give.
- Conclusion: −2.2951% is exact. The paper's −2.26% is unexplained (a slip
  or an unstated convention). The draft is right to say "two point three
  percent low and one point two high" from the computation without quoting
  the paper (24_1).
- Recommendation: record the closed form in `brief.md` and keep the paper's
  figure out of the film. If the paper is cited for the longimeter, cite it
  for the construction, not the band.

### F6. Smaller points (info)

- **28_5 "never anything else".** A square turned 45° with a corner exactly
  on a line meets the sheet **3** times (my direct count). A side along a line
  meets it infinitely often. Both have chance zero. The note names only the
  side case. Spoken words are fine because the drop is random. Name both in
  the note.
- **18_4 "a stick near the edge is always shortchanged".** It holds in every
  case I tried against the same stick in the middle in the same orientation:
  - upright: middle 22.01%, the film's corner stick 12.44%, bottom edge
    12.53%, left edge 11.26%;
  - lying: middle 12.45%, bottom edge 6.49%, left edge 5.67%;
  - 300,000 pin lines, standard error about 0.08%. The lane's exact middle
    figure is 21.891%.

  The film's corner stick still gets more pin lines than the fair 8%, so
  "shortchanged" means against the middle, not against fairness. "Always" is
  untested beyond these cases.
- **Runtime.** The header says 28.2 min (word count). The 28 chapter headings
  sum to 2,017 s = 33.6 min. These are two different estimates, and neither is
  measured. No compression or ceiling is proposed anywhere, and every chapter
  heading is ≤ 85 s (largest 84).
- **`docs/script-draft.tsv`** carries the tool's header "generated from
  docs/storyboard.md", although it is the draft snapshot (the draft's opening
  paragraph explains this). Words: all 155 draft lines equal the draft's words
  column, and all 7 production lines equal `storyboard.md` (my parse,
  ⟨predict⟩ markers stripped). Total 3,974 words.
- **Hygiene.** The live `explainer_corpus/crofton/__pycache__/oracle.cpython-314.pyc`
  is not in the intake. It isn't mine: I ran Python 3.15 with `-B` from a
  scratch copy. It should not be committed.
- **07_3.** "Midway between two" lines. The oracle checks the stick at
  y = 0.7 with lines at 0 and 2 (`oracle.py:582`), where midway would be 1.0.
  The count is 0 either way.

## Verified, no finding

All by my own recomputation (see `independent_check.out`).

- **The coast.**
  - Length 15/2 + 1399π/240 = 25.812867 cm, and the placed pieces' arc
    lengths sum to it.
  - Joins have no gaps or corners (max 4.4e-16).
  - Reach |x| 7.302, |y| 2.016, inside 16 × 9 by more than ¼ cm.
  - No self-crossing (0 among 7,326 chords).
  - Ends 14.7305 apart.
  - The beach is level at −1.982 and 3.5 cm long.
  - The headland's middle tangent has the whole coast on one side, at a
    7.5000° tilt (01_3's single touching point).
- **Shown placements.**
  - 0° slide 0.45: 7 crossings, clearance 0.397 cm, 4 on the lower line and
    3 on the upper, walking order cove, climb, rise, fall, bay, bay, end (02_4,
    02_6).
  - 45° at 1.20: 8, clearance 0.277.
  - 90° at 0.30: 7, clearance 0.161.
  - 135° at 1.35: 8, clearance 0.152.
  - Each slide is the clearest point on its 0.05 cm grid.
- **The 04_1 slide.** Counts 7, 9, 8, 7, shortest run 0.056 cm. Exact runs
  over the whole gap: the only sub-0.02 cm run is 9 crossings from 1.5676 to
  1.5724 cm past the start, where the beach lies along a line. That is past
  the 1.5 cm shown, as the draft says.
- **Travel and slide means.**
  - Per direction: 0°: 15.2603 / 7.6301; 45°: 17.4852 / 8.7426; 90°:
    16.4516 / 8.2258; 135°: 15.8213 / 7.9106. Exact slide means equal travel
    ÷ gap.
  - Count ranges per direction match `slide_counts`.
  - All directions: mean travel 16.4330, × π/2 = 25.8129.
  - Four directions: 8.12729, −1.086%.
- **The gap.**
  - The mean count is 2L/(π·gap) at gaps 2, 4 and 8, so mean × worth =
    25.8129 every time (gap invariance, separate from root's six Spacing
    tests).
  - Worth π·gap/2 (π cm on the 2 cm sheet).
- **Negative controls, all behaving.**
  - A quarter turn of directions → 26.65 cm.
  - Directions weighted 2 sin² → 26.13 cm.
  - Forgetting the half in the worth → 51.63 = 2L.
  - Headland radius 4/5 → length 25.98.
  - The square outline of side 2 for the dividers (F4).
- **Sticks, square, coin.**
  - The 6 cm stick at 30° gives 1 or 2, half each.
  - Bent stick 1..4 against straight 0..3.
  - The 2 cm square gives 2 or 4, with four-share 0.273240 = 4/π − 1. The
    28_5/28_6 transfer puzzle is answerable from the film: mean count 8/π
    with counts 2 or 4.
  - The grazing coin gives 2.
- **The longimeter.** 12/π = 3.8197 mm. Six slid directions on the coast give
  25.7269.
- **Dividers.**
  - 4 cm: 4 steps, 18.659.
  - 2 cm: 11 steps + 0.6414 = 22.641. The step leaving the headland is step 6
    and spans 3.0772 cm of coast.
  - 1 cm: 25 steps, 25.137.
  - 0.5 cm: 51 steps, 25.636.
- **Ripples.**
  - The rippled drawing is 27.8107 long, mean count 8.8524.
  - 2 cm dividers on it walk 22.627.
  - Richardson's 2^¼ − 1 = 18.92%.
- **Pins.** Middle 22.01% (the lane's 21.9%), corner 12.44% (12.4%). The even
  way gives 8%.

## Pedagogy

- **One coast.** The same coast from 01 to 28. 26's rippled beach is named
  "the new drawing", with its ends fixed.
- **A calibrated worth.** The coin comes before the bet (05 → 06), and the bet
  is tested on sticks (07–11) before Barbier's reason (12). The worth is π·gap/2,
  calibrated per gap in the oracle's Spacing controls.
- **Intuition before the formula.** The integral appears at 20, after the
  stick's band and the coast's chart (18, 19) with a prediction.
- **Finite estimates.** They are called estimates, with their spreads (but see
  F3 for which spreads).
- **The final transfer (28_5–28_6).** A fresh shape, answerable from the
  film's own facts.
- **No compression or runtime ceiling.**

These all hold, subject to F1 to F4.

## Native source and API binding (read, not compiled)

- **fmn at pin a399d626c43e (checkout verified with `git rev-parse`).** These
  signatures match the calls: `Stage::add_updater`, `tracker_value`,
  `add_value_tracker`, `clear_updaters`, `remove_from_scene`, `set_points`
  (`Result`), `move_to` (not `Result`), and `AnimBuilder::scale_about`,
  `set_stroke`, `rotate`, `set_opacity` and `set_value`.
  - `DashedLine::build` returns `Result<_, DashError>`.
  - `Rectangle::build` returns `Result`.
  - `SceneError::InvalidConfig` exists.
  - The same call patterns appear in crates that are committed and compiled.
- **explainer_kit and explainer_score.**
  - The `Explainer` literal's 12 fields equal the struct's in both HEAD and
    the working tree.
  - The `Identity` literal's 24 fields (plus the 4 `Palette` fields) equal the
    struct's in both.
  - `LineGaps::Thought`, `narration::GAIN_DB`, `Script::{parse, text, lines}`,
    `storyboard::check_video`, `cli::check_script_owners(&Script, &[&str],
    Option<…>)`, `cli::run`, `Chord::{new, six_nine, dom7sus4, sus4}`,
    `Motif::new`, `Key::{new, degree}` and `IONIAN` all exist.
  - The working tree has uncommitted edits to `narration.rs` (+392 lines) and
    `score.rs` (+193). Crofton's literals match both versions, but root's
    compile should name the tree it uses.
- **`main.rs`.** The spoken-number guard is phrase-scoped and has its own
  negative test. Its SPOKEN rows cover the four number phrases in chapter 01.
  `the_dividers_answer_is_the_facts_answer` checks only 1 cm against 2 cm (F4).
- **`facts.rs`.** Rounding goes through f64 (`(x*scale).round()`), and
  values on a half are not specially handled. I did not check every fact for
  this. The chapter 01 values I recomputed (14.7305, 22.6414, 0.6414, 3.0772)
  are not near a rounding half.

## Opening (chapter 01) frame risks, with no rendered frame claimed

1. **The walk outlasts 01_4.** After "walk them" the scene spends about
   0.3 + 0.2 + 10 × (0.42 + 0.12) + 0.3 ≈ 6.2 s (`the_question.rs:259–280`).
   About ten words remain in the line, so expect roughly 2–3 s of silent
   walking before 01_5. Check at first render whether it reads as watching or
   as dead air.
2. **The live step count pops in.** `Kit::live_value` adds itself to the
   scene, so it appears without a fade while "steps" fades in beside it. Its
   semantics are right: 1 at the opening, 11 at the last landing.
3. **Estimated clearances.** The 01_6 zoom (2.2×) words are checked against
   the zoomed coast with an estimated half-width of 2.0, not measured glyph
   extents. The same holds for the label margins (the lane's own "hook" label
   is 0.087 cm inside its margin).
4. **The draft's 04_1 flicker.** The 0.056 cm run of 8 lasts about 0.2 s and
   needs a frame check, as the lane notes.

## Source-reading limits

- Steinerberger: read through the fetch tool's HTML extraction only. Quotes
  are as extracted, not checked against the PDF.
- Not read by me: Crofton 1868, Barbier 1860, Cauchy, Steinhaus's longimeter
  paper and patent, Mandelbrot 1967, Richardson 1961, Santaló.
- No competitor videos watched. No listening of any kind.
