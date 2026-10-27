# german_tank — work status

Lane: GentleLynx (Claude Code, claude-opus-5-5), Bead
`mx-video-program-program-87do.12.1`, session
`jeffs_math_explainer_videos--third-wave-20261024`, pane %13. Root and Beads
owner: BlueSalmon. Status: **batch_pending** (first handoff, 2026-10-24;
source corrections after root's review, the strict-Clippy repair, the
box-trajectory, test-only and tick-anchor repairs after root's opening
previews, chapters 02 and 03 as unregistered native source with one
integration patch, which root applied and qualified, chapters 04 and 05
the same way, and chapters 06 and 07, then 08 and 09, then 10 and 11, then
12 and 13 the same way under native task
`mx-video-program-program-87do.12.3`, all the same day). Not self-closed:
root qualifies and decides the Beads' state.

## Acknowledgment (2026-10-24)

Assignment acknowledged. Intake read: AGENTS.md including the 2026-10-24
amendment; docs/video-program/README.md; plan §0, §1, §3, §4 (and §20's
german_tank row); third-wave README, MOR_CASE, SELECTION, TODO; this topic's
catalog entry; the four skills (end-to-end explainer, franken-manim mega
skill, natural-sounding narration, de-slopify); resistor_random_walk as the
crate-layout example. Agent Mail reservation 7519 held
`explainer_corpus/german_tank/**` until root narrowed it (Mail 6871). Since
then I hold four exclusive leases, 7902 to 7905, on the new 04 and 05
chapter files, `docs/integration-04-05.patch` and this file. No other
writers' proposals were read.

- Workspace hazard (Mail 6763): the `explainer_corpus/*` member glob fails on
  a directory without `Cargo.toml`. Root added this directory to the root
  manifest's `exclude` (Mail 6769) before any file here was written.
- Owner direction (Mail 6777, acknowledged): more setup and visual intuition,
  no first-minute deadline. Applied as a new three-chapter opening act
  (01 five boxes and the lower bound, 02 computed fair handfuls and spacing as
  a clue, 03 the gaps counted one at a time and the recipe with its
  condition on screen); the later chapters renumbered 04 to 16. The previous
  storyboard, draft, pauses, brief, protected list, script and oracle were
  kept outside the repository before the rewrite.
- Scope correction (Mail 6790): only chapters with a native scene are in the
  production storyboard, `script.tsv` and the registry, so those now hold
  chapter 01 alone; chapters 02 and 03 moved, word for word, to the top of
  the draft (checked: the two files' non-blank lines are the same set before
  and after the move). Nothing was discarded, no placeholder chapter is
  registered, and no preview machinery was built. The pre-move files were
  kept outside the repository too.

## Source corrections after root's review (2026-10-24)

Root found that the opening's "1" and "2" (01_3) and its zoom marks 1, 250,
500, 750 (01_9) were literal labels outside the picture's facts binding. They
were correct, but nothing tied them to the line's geometry. The pre-change
files were kept outside the repository.

- `src/model.rs`: `ZOOM_PARTS` (4) and `marks(first, end, parts)`: the first
  number, then the extent's equal parts short of its end. It refuses an
  extent the parts do not divide, zero parts, and a first step at or below
  the first number. Test `the_marks_divide_the_line_evenly` covers 1,000 in
  4, 4 in 4, 1 part, refusals at 1,002, 0 parts, step 1 and first 250, and
  parts 1 to 10 against the definition.
- `src/facts.rs`: new fact `G-F1.zoom_marks` = `marks(G-F1.first, BIG,
  ZOOM_PARTS)`; `G-F1.second` is `first + 1`; count test 152.
- `oracle.py`: `G-F1.zoom_marks` computed independently (`first` plus
  `range(1000 // 4, 1000, 1000 // 4)`); `G-F1.second` as `first + 1`.
- `src/chapters/five_boxes.rs`: the `MARKS` table is gone. `Picture` reads
  `G-F1.first`, `G-F1.second` and `G-F1.zoom_marks`. The labels, the dots
  (`first..=LINE_DOTS`), the certain bracket's start, the lit range and the
  marks all come from those facts. `checked()` refuses a first number other
  than 1, a second that is not first + 1, a box at or left of the second
  label, and marks that differ from `marks(first, thousand, ZOOM_PARTS)`.
  New negative controls: first 0; first 2 with second 3; second 3; a box at
  number 2; extent 2,000 with the old marks; a mark moved to 760; extent
  1,002. Positive control: extent 2,000 with marks 1, 500, 1,000, 1,500.
- `src/chapters/draw.rs` (comments only): `Spoken` and `Spoken::at` now say
  that a phrase's moment is interpolated by character position inside its
  sentence. That is an approximate draft cue, not measured word timing. The
  narrated version must bind the number flips and reveals (01_4 first) to
  the qualified FrankenWhisper word alignment before claiming synchrony. No
  timing framework was built, and the approximation proves nothing about
  natural pacing.
- `docs/storyboard.md`: the notes for 01 name `G-F1.second` and
  `G-F1.zoom_marks`; the chapter's words are unchanged, so `script.tsv` is
  unchanged.
- Beat 16_1's provenance (`docs/storyboard-draft.md`, notes for 16;
  `docs/brief.md`, "Sources"): root obtained, through a web search, the
  publisher's indexed abstract of Ruggles and Brodie (1947),
  https://www.tandfonline.com/doi/abs/10.1080/01621459.1947.10501915. It
  states that in early 1943 the Economic Warfare Division of the American
  Embassy in London used markings and serial numbers on captured German
  equipment to estimate production and strength, first tyres and later tanks
  and other weapons. That supports the narrow sentence. The earlier
  observation stands: opening the page directly returns HTTP 403, and the
  full paper has not been read. No historical figure is imported. The
  abstract names an embassy division, not a profession; at root's choice
  16_1 now says "Allied analysts" (was "Allied statisticians"), with the
  draft's notes for 16, the brief and the lexicon's risk list to match.

## Strict-Clippy repair and UBS disposition (2026-10-24)

Root's serial pass stopped at `cargo clippy ... --all-targets -- -D warnings`
with 9 errors (`narration_lab/third-wave-20261024/root/native-qualification/german-qualify01/command.log`).
Repairs, in the three affected files only:

- `src/model.rs`: `Q::f64` was dead code with no caller anywhere, tests
  included, so it is removed rather than scoped; nothing is suppressed.
- `src/chapters/draw.rs` (2) and `src/chapters/five_boxes.rs` (5): each
  layout assertion Clippy found constant is now `const { assert!(..) }`.
  The checks are kept (a failure now stops the test build), with their
  messages.
- `src/chapters/five_boxes.rs`: `n % 10 == 0` is now `n.is_multiple_of(10)`
  in the spelling helper.

Root's bounded UBS scan of the nine accepted files
(`narration_lab/third-wave-20261024/root/tend-0610/german-ubs.log`): exit 0,
0 critical, 331 warnings, 149 info. It is **not clean**. Below is my triage by
category; the counts are UBS's, and the production-site reasoning is mine
from reading the current bytes.

- **unwrap/expect (39):** 11 `expect` calls outside tests, all on documented
  invariants.
  - Five are `Q`'s overflow checks (`gcd`, `times`, `plus`, `negated`,
    `count`). The largest values are C(1000, 5) ≈ 8.3·10¹² and pairwise
    products near 6.8·10²⁵, far below i128's 1.7·10³⁸. An overflow would be a
    model bug and should stop `tank facts` loudly.
  - Three take `last()` of a handful: in `gap_recipe` and `gap_list` the
    handful is never empty (k is 5 or 2), and `FRIENDS` is a constant array.
  - One is `(1..).find(..)` for `G-F1.at_least`, which stops at the largest
    friend's number.
  - One is `worst_coverage`'s `min()` over counts 5..=150, never empty.
  - The rest are in `#[cfg(test)]` code, as test failure paths.
- **panic! (5):** all test-only, as UBS says.
- **assert inventory (170):** outside tests there are four preconditions: a
  non-zero denominator, no division by a zero fraction, `below(n)` with
  n > 0, and the new `drawn_handfuls` guard below. The rest are test
  assertions.
- **direct indexing (111):** each production site, bounded as follows.
  - `dots[n − first]`: `checked()` fixes 1 ≤ n ≤ at_least < 84.
  - `dots[..12]`: within the 84 dots.
  - `dashes[..shown]`: shown ≤ the zoomed count, because a dashed span
    starting further left (x_at(61, 1000) < x_at(61, 84)) to the same right
    end has at least as many dashes. The draw.rs test checks these values.
  - Box, body and label `[i]`: i < 5, since `checked()` fixes five numbers in
    the five band slots.
  - `sentences[j]`: offset 0 always exists.
  - `text[..byte]`: at a `find` boundary.
  - The model's tables: `ways[j][s − b]` with s ≥ b and j ≥ 1; `g[m − k]`
    with m in k..n; `tops[m]` with m ≤ last.
  - `big[0][4]`, `small[0][4]`, `known[4]` in `facts.rs`: rows of K = 5.
    The literal 4 means "the largest" only while K is 5. It is not wrong
    now, so I left it (facts.rs was not in this repair). It is a cleanup for
    when facts.rs opens for chapters 02 and 03.
- **parse().unwrap() (3):** all in main.rs tests.
- **string allocation in loops (3):** a few G-F9 keys and the five box
  labels; negligible.
- **float equality (3):** two are false positives (a string comparison at
  main.rs:179, a u64 comparison at five_boxes.rs:164). The third, draw.rs:643,
  compares a loop variable taken from the literal list [0.0, 1.0], so it is
  exact.
- **division (34) and modulo (3) by a variable:**
  - `x_at` divides by end − 1, and end stays in [84, 1000].
  - Fraction division asserts a non-zero divisor.
  - `Spoken::at` divides by `max(1)`.
  - `gcd` takes `a % b` only while b ≠ 0.
  - `below(n)` asserts n > 0.
  - Fraction denominators are K, K + 1 and k − 1, with k = 5.
- **wrapping arithmetic (3):** SplitMix64's own recurrence, the same one the
  oracle runs; intentional.
- **clone (12), collect then for (4), wildcard imports (9):** the clones are
  the Rc-backed `Kit` and vectors moved into the `'static` updater, plus one
  leaf clone in the combinatorial recursion. The collects are small joins.
  The wildcards are `fmn::prelude::*` and `use super::*` in tests.
- **`as` casts (77):** none truncates for this video's values.
  - u64 → f64 for screen positions (at most 1000, exact).
  - Widening u64/usize → i128 into fractions; usize → u64 lengths.
  - `(n / 10) as usize` with n < 100.
  - `(96·f).ceil() as usize` with f clamped to [0.002, 1].
  - Table indexes at most n·k + 1.
- **not evaluated by UBS:** fmt, clippy, build and dependency hygiene (no
  Cargo.toml at the scan root). Root's own runs are the evidence there.

A release-build note: the release profile leaves overflow checks off, so a
misused u64 precondition would wrap there rather than panic. `gaps()`
needs sorted, distinct numbers of at least 1. Every current caller meets
that, and the oracle diff would expose a wrong fact.

Actual defects: none of the scanned sites is wrong for this video's inputs.
UBS did not name one latent defect, which is now fixed in `src/model.rs`:
`drawn_handfuls` redrew forever when asked for more different boxes than
exist (k > n). It now refuses with a message. Two tests cover it: the edge
k = n gives every box, and k = n + 1 must panic with that message. No
current caller is affected: the draws are (1000, 5), (70, 5), (84, 5) and
(71, 5).

## Box-trajectory repair after root's opening preview (2026-10-24)

Root's silent scratch preview (Mail 6837;
`narration_lab/third-wave-20261024/root/native-qualification/german-opening02/output/01_five_boxes.mp4`)
showed the numbered boxes overlapping at 74.5 and 75.0 s. That is 01_8,
where the line's end and the boxes' spread were tweened together. The
layout tests checked only where moves end, so they missed it.

A scratch Python mirror of the geometry measured every move in chapter 01.
It takes the clearance between box outlines over 400 samples a move, using
the same rectangle test as `draw::boxes_clear`. It found a second collision
that root's sampled frames did not include. In 01_5 the boxes went onto
the line left to right, and box 51, sliding down, met box 60 still in the
band (worst clearance −0.138). A crop of root's preview at 42.267 s shows
51's outline touching the bottom of 60's.

Repairs, in `src/chapters/draw.rs` and `src/chapters/five_boxes.rs` only:

- **01_8** (`draw::zoom_legs`, `leg_views`, `play_legs`): the line's end
  moves only while the boxes are in their row. The boxes spread first, at
  the opening end, on "or run on" (0.7 s). The end then moves on
  "hundreds more" (2.0 s, as before). Worst clearance: −0.262 before
  (27 and 33), +0.099 now (the row's own spacing).
- **01_5** (`five_boxes::place_order`): the boxes go onto the line right to
  left. Each one slides away from the boxes still in the band. Worst
  clearance: +0.279 (27 and 33).
- Unchanged: ticks, labels, box identities, the zoom's endpoints, and the
  5.2 s prediction hold before 01_7. By the draft cue times, the end leg
  fills the same 74.17 to 76.17 s window the old zoom did, so 01_9 should
  start where it did. These are character-share estimates, not
  measurements.

Regression tests (3 new, so 38 expected; none compiled yet):

- `draw::boxes_are_clear_only_with_room_between_them`: the clearance test
  at its edges.
- `draw::the_line_moves_only_under_spread_boxes`: the legs in each
  direction. Each sequence starts at `from` and ends exactly at `to`, and
  the fan is 1 wherever the end is strictly between its endpoints.
- `five_boxes::the_boxes_never_meet_on_the_way`: every sampled state of
  01_5 (in the scene's order) and of 01_8 (in the scene's legs) is clear.
  The negative controls are the two trajectories the preview showed
  colliding, left to right and end-with-fan. The same check must refuse
  both.

The view `(1000, 0)`, boxes back on their numbers at a thousand, is itself
crowded (−0.432). No leg can make it safe, and no scene uses it.

## Test-only helpers and the tick anchor (2026-10-24)

- Root's repair03 run (`native-qualification/german-requalify03/command.log`)
  passed fmt and all 38 release tests, including both collision negative
  controls. Strict Clippy then stopped on four symbols only tests use, in
  `draw.rs`: `BOX_CLEAR`, `boxes_clear`, `all_clear` and `leg_views`. Each
  is now `#[cfg(test)]`; nothing is suppressed. Root's repair04 run passed
  fmt, 38 tests, strict Clippy, build and 152/152 facts, and its sampled
  frames showed the two box collisions gone.
- Root's repair04 frames at 73.5 and 73.8 s
  (`root/tend-0640/german-visual04/frame-73.500.png`, `frame-73.800.png`,
  which I viewed) showed the gold ticks running diagonally through the
  lower bodies of neighbouring boxes as they fanned left. The old tick met
  its box where a line toward the dot leaves the box's side. Now
  `draw::tick_ends` ends each tick at the bottom centre of its own box. The
  boxes share a row bottom and the dots are below it, so every tick stays
  under every box. Dot and box identities, the prediction, the zoom order
  and the timings are unchanged.
- Regression tests (2 new, so 40 expected; Mail 6855):
  `a_segment_enters_a_box_only_through_its_inside` tests the clipping check
  at its edges. `ticks_pass_under_every_box` covers every sampled state of
  both zoom legs and the ticks' growth. In each state, no tick enters any
  box and no two ticks cross. Its negative control is the old side-exit
  tick, which must cross a neighbour. The frozen `draw.rs` is
  `a28b04a8792e…`.

## Chapters 02 and 03 (2026-10-24)

Root assigned real native chapters `02_a_thousand_boxes` and
`03_counting_the_gaps`. They are written as new, unregistered files, and the
registered files stayed frozen during root's qualification. None of the new
code has been compiled, and none of its tests has run: `mod.rs` does not name
these modules yet, so no build includes them.

- `src/chapters/friends_line.rs` (sha256 `89a3ed0d4242…`, 11 tests): the
  line both chapters share. It holds the bin and the pour, rows of computed
  handfuls as rings, spacing arrows, gap arcs, the badge, panel layout and
  reading time. Everything on the line is bound to trackers through
  `draw::x_at`, and every zoom plays `draw::zoom_legs`.
- `src/chapters/a_thousand_boxes.rs` (`d70d7dd1ba88…`, 8 tests): chapter 02.
  - The bin defines the fair handful (card B).
  - The prediction with the line shown to a thousand.
  - Ten computed rows from a thousand, all past 600. The teal outline of 1
    to 60 that none of them fits.
  - Ten rows from seventy, three of which fit.
  - The spacing arrows with a live readout of one row's arrows over five,
    and the panel's exact averages labelled "all handfuls from".
  - `checked()` refuses pictures the words contradict, for example none or
    all of the rows fitting, a count different from "Three of these ten", a
    row from a thousand that stops short of 600, or a readout that ends away
    from the printed fact. Every negative control asserts its reason.
  - The intermediate states of every zoom keep the boxes apart.
- `src/chapters/counting_the_gaps.rs` (`3333166da85f…`, 9 tests): chapter 03.
  - Each gap counted dot by dot. The count under the line and the lit blue
    dots round the same tracker, with a floor-rounding negative control.
  - The counts slide into the panel's sum, then 55 ÷ 5 = 11.
  - The top gap's dashed bracket, bound to one tracker, shortens to 71.
  - The inset of chapter 02's first row from seventy, its six gaps visible.
  - The recipe, with the badge beside "right on average".
  - Card A, and the estimate dimming while the recipe stays bright.
  - Negative controls include the doubling recipe's 73, a known handful
    that does not fill seventy, and a card text too long to read.
- Defects found and fixed before handoff, by reading and the scratch checks
  below:
  - `friends_line.rs` used `.rt()` without importing `Timed`.
  - Three dead items: `FriendsLine::friends_dots`, the `Rows.hands` field
    and the `FriendsLine.layout` field.
  - The 02 stretch was a teal fill; the certain role allows strokes only.
  - Card A at 201 characters would truncate at the narrower lower-third
    widths, so it is now 178.
  - The inset's rings sat too close together.
  - Constant-only asserts are now `const { assert!(..) }`.
  - `% 2 == 0` is now `is_multiple_of`.
  - An unused `let` binding was removed.

### The integration patch

`docs/integration-02-03.patch` (sha256 `843c0cf85b7f057d…`, 738 lines, 12
files) registers the chapters in one coordinated change. The patch holds
these changes:

- `oracle.py` and `src/facts.rs` gain the same 9 facts (152 → 161):
  - `G-F2.over` 600 and `G-F2.spacing_over` 150, which the existing
    `…_over_600` and `…_over_150` flags now use.
  - `G-F2.small_fit_shown` 3.
  - `G-F3.first_seen` 14, `second_seen` 27, `first_gap` 13, `second_gap` 12,
    `last_gaps` 5,17,8.
  - `G-F3.top_gaps` 1, counted on the known handful, where the top gap can
    be seen.
  - The oracle gains one cross-check, so 27 in all.
- `src/palette.rs`: `unseen()` (blue) and `estimate()` (green), both in the
  roles test.
- `src/chapters/mod.rs`: the three modules, and registry rows for 02 (0.45)
  and 03 (0.5).
- `src/main.rs`: 40 new SPOKEN rows (55 in all), 02 and 03 in the
  prediction test, and the module doc.
- `docs/storyboard.md` gets chapters 02 and 03 with pictures matched to the
  source, plus the 01_5 and 01_8 picture cells matched to the repaired
  moves.
- `docs/storyboard-draft.md` keeps chapters 04 to 16. The block removed was
  checked to equal the old draft's lines 25 to 115 exactly before the 02
  and 03 text was revised.
- `script.tsv` is regenerated with the tool's `script_text`: 28 lines.
- `docs/pauses.tsv`: the 02_1 row follows its new words.
- `docs/protected.tsv`: its 7 rows for 02 and 03 are now active.
- `docs/brief.md` and `lexicon.json`: one pronunciation risk added (compound
  numbers).
- Spoken rewordings, so that every number word has a fact and chapter 02
  stays within the 85 s estimate:
  - 02_1 "in one huge bin" became "in a huge bin".
  - 02_5 "Every one of them" became "Each of them".
  - 02_6 was rephrased around "Five numbers".
  - 02_7 now ends "Nearly half of its possible handfuls stay at sixty or
    below. Three of these ten do."
  - 03_6 now ends "and it happens to be out of sight."

The files the patch touches are byte-identical to the frozen, root-qualified
ones (checked by `diff -r` against a pristine copy). Root applies it with
`git apply explainer_corpus/german_tank/docs/integration-02-03.patch` at the
repository root. Then the 68 tests (40 + 28), fmt, strict Clippy, build,
`tank facts` against the oracle (161), the glyph probe and the frames need
qualifying. Keep the patch file after it is applied, as the record of the
change (root, Mail 6871).

Root's result (Mail 6877): root applied the patch and qualified chapters 02
and 03. It reported 68 release tests, strict all-target Clippy and build,
161 native and oracle facts, 35 independent display checks and 66 glyph
sources, with six pages viewed. Chapter 03's frames showed `dim_all`
filling the open estimate bracket at 03_10, because the pinned engine's
`set_opacity` also sets fill alpha. Root repaired `counting_the_gaps.rs`
in the live tree: strokes now dim by stroke alpha only, with the same
0.6 s. That file is now root's bytes (`04647539d4b7…`), not the
`3333166d…` I delivered, and I have not changed it. The patch file stays
in `docs/`.

### Checks actually run on the patched copy (2026-10-24, to 08:08 UTC)

All of these ran on a scratch copy outside the repository: the pristine
topic with the patch applied, which `diff -r` showed equal to the working
copy the edits were made in.

- `git apply --check` on the live tree: exit 0, all 12 files. Applying it
  to a pristine copy reproduces the tested copy exactly.
- `python3 -I oracle.py`: exit 0, 161 facts, 27 cross-checks, 0 failed;
  stderr kept.
- The storyboard tool's `check`: 01 74.8 s, 02 84.6 s, 03 67.3 s, no
  problems. The draft: 04 to 16, no problems.
- `rustfmt --edition 2024 --check --config max_width=120` on all 12 Rust
  files: pass. The differences rustfmt reported along the way were applied
  by hand.
- `protected_check.py` on the patched protected list and script: 10 listed,
  0 missing, PASS.
- A scratch Python mirror of the text tests. It covers main.rs's SPOKEN
  values against the oracle output and the coverage of every number word,
  the predictions, the pause rows, the draft continuing the registry and
  `script.tsv` not stale. It also covers every phrase both chapter tests
  list and every `wait_for` in the two scenes (once, in order). Result: 55
  SPOKEN rows, 53 pause rows, 0 failed.
- Scratch heuristics, which are not rustc or Clippy, so their passes prove
  nothing:
  - Unused imports: clean except a known false positive (`Timed`, used
    through `.rt`).
  - Dead items and fields: only parameters, struct literals and trait
    methods, which are false positives.
  - Constant-only asserts: none in the new files. The hits in the frozen
    files are on bytes root's strict Clippy has already passed.

## Chapters 04 and 05 (2026-10-24)

Root's assignment (Mail 6871): new, unregistered `04_what_we_assume` and
`05_every_handful` from the reviewed draft. They keep the same five boxes
and line through the definition of a recipe and a fair handful, then shrink
to the same ten visible pairs to earn the average. Each exact estimate comes
from one computed model, with the prediction and inspection time kept.
Mail 6877 added one rule: unfilled paths dim by stroke alpha only. Neither
file has been compiled, and none of their tests has run: `mod.rs` names
them only once the patch below is applied.

- `src/chapters/what_we_assume.rs` (sha256 `f836ac7f2a71…`, 1,198 lines, 9
  tests): chapter 04, on chapter 03's line (1 to 84, the five boxes on their
  ticks, the badge).
  - 04_1: "count = ?", then 60, 71 and 500, each on its word.
  - 04_2: halos round the five and round the badge; then six rows of rings,
    the fair handfuls from the same computed draw as chapter 02's bin, the
    bin's own handful first (`G-F4.ghost_handfuls`).
  - 04_3: handful → recipe → estimate as a small picture, with "estimator"
    under it.
  - 04_4 and 04_5: the recipe in words; the certain bracket 1 to 60; a halo
    round the five; the arcs and "gaps: 60 − 5 = 55"; the same rule for any
    five; the recipe in the largest number alone; 60 + 55 ÷ 5 = 71 piece by
    piece; the green tick; a 2.5 s inspection hold.
  - 04_6 asks about the handful 1 to 5; 04_7 answers 5 + 0 ÷ 5 = 5 and adds
    "never below the largest"; in 04_8 the rows come back.
  - Builders 05 reuses: `halo`/`glow`, `replace`, `words_left`,
    `sum_texts`, `gaps_line`, `formula`, `instance`, `roles` and
    `unseen_below`.
  - `checked()` refuses each of these:
    - friends' numbers not in order or not on a line from 1;
    - a gaps' total other than the largest minus five;
    - an estimate that is not exact;
    - a possible count below the largest, or out of order;
    - rows that are not six fair handfuls of the bin with the bin's first;
    - an edge handful other than 1 to 5 with empty gaps.

    Every negative control asserts its reason.
  - Tests:
    - The picture is the facts, and numbers are coloured only where
      unambiguous.
    - The refusals above.
    - The recipe never guesses below the largest number, for every handful
      with k ≤ 6 and N ≤ 12, with equality exactly at 1 to k (57 cases); the
      doubling recipe on `SILLY` is the negative control.
    - Every waited-for phrase is once in its line, in order.
    - The halos keep clear of what they mark and stay on the frame; the
      panel fits.
    - Ring rows keep apart; the negative control is 43 and 44 on a line to
      1,000.
- `src/chapters/every_handful.rs` (`3152edb9e8b1…`, 1,164 lines, 8 tests):
  chapter 05.
  - 05_1: the 71-box company's handfuls as a live count up to 13,019,909;
    five small boxes, with labels 2 and 5 replaced by gold ones.
  - 05_2: the ten pairs as a table, halos on the first and last rows, "each
    pair: 1 in 10", then a 2.5 s hold.
  - 05_3 and 05_4: for the pair 2 and 5, the unseen 1, 3 and 4 are replaced
    in blue, then come its two gaps' arcs and counts, "gaps: 5 − 2 = 3", the
    recipe for a pair and 5 + 3 ÷ 2 = 6.5 into its row. The pair 1 and 2
    gives 2.
  - 05_5: the other eight estimates cascade in; a count line at 5 gets each
    row's dot; then the prediction.
  - 05_6: the ten dots fly onto an axis and stack, the fulcrum reading
    "average = 5". The larger numbers, as gold dots, stack on a second axis
    at "average = 4".
  - 05_7: "unbiased: average estimate = count", then "count 5: average 5",
    then "every other count: ?" with a halo.
  - Every estimate is the model's exact fraction, shown exactly. `checked()`
    recomputes each from its pair's gaps and refuses each of these:
    - handfuls not equal to C(71, 5), or outside the millions the words say;
    - pairs not each once, in order;
    - an estimate other than its largest plus the average gap, or shown
      inexactly;
    - averages other than exactly 5 and 4;
    - a worked pair whose unseen boxes, gaps or estimate disagree;
    - a low pair that is not the gapless first row.

    The doubling recipe also averages 5 here, and it is refused as the
    table's column.
  - Tests:
    - The picture is the facts.
    - The refusals above.
    - Fractions are read and shown exactly.
    - The waited-for phrases.
    - The balance settles at the exact means; the negative control is 4 for
      the estimates.
    - The table, the worked pair and the balance fit the frame.
- No typeset text is indicated, scaled or recoloured in 04 or 05.
  - A highlight is a separate rounded outline that fades in, holds 0.5 s and
    fades out. A change of colour or wording replaces the line.
  - The one `indicate` is 04_5's ripple over the five gap arcs, which are
    open strokes with no text. At the pin, `indicate` rewrites RGB only
    (`fmn-anim`'s `set_family_rgb`), so the arcs stay hollow.
  - Nothing dims by opacity: each dim colour is set when the object is made.
    Root's stroke-only rule therefore has no site here.
- Defects found and fixed before handoff:
  - `FlowAt.right` was read only by a test, which makes it a dead field
    outside test builds. It is removed, and the test computes the right end.
  - Three constant-only asserts in 05's tests are now
    `const { assert!(..) }`.
  - Earlier, while writing: an unused `Timed` import (05); `&str == &String`
    in `roles`; an eight-argument builder, now bundled into `Stacks`;
    `instance` takes the estimate as text so that 6.5 can show; a float loop
    for the axis ticks.

### The 04/05 integration patch

`docs/integration-04-05.patch` (sha256 `d28589efae32e229…`, 693 lines, 11
files) is made against the live tree as root left it after applying 02/03,
root's `counting_the_gaps.rs` included. It touches no chapter source; the
two new files are already in place.

- `oracle.py` and `src/facts.rs` gain the same 15 facts (161 → 176):
  - `G-F4.far_count` 500 and `G-F4.ghost_handfuls` (six rows);
  - `G-F5.company` 71, `handfuls` 13,019,909, `millions` 13 and
    `handfuls_over_millions` true;
  - `G-F5.first_pair` 1,2, `last_pair` 4,5 and `estimates_shown`
    2,3.5,5,6.5,3.5,5,6.5,5,6.5,6.5;
  - `G-F5.example_low_pair` 1,2, `example_high_pair` 2,5,
    `example_high_largest` 5, `example_high_unseen` 1,3,4,
    `example_high_unseen_count` 3 and `example_high_gaps` 1,2.

  `facts.rs` adds `FAR` (500) and `GHOSTS` (6), names the worked pairs once
  as `low` and `high`, and its count test becomes 176. The oracle gains 8
  cross-checks (35 in all):
  - its own count of the 71-box handfuls, as ordered draws over orders,
    checked against chapter 08's brute force;
  - 04_5's rule for every handful, k ≤ 5 and N ≤ 14;
  - a negative control: dropping the per-gap share, or dividing by three
    gaps, misses the average of 5.
- `src/chapters/mod.rs`: the two modules, registry rows 04 (0.5) and 05
  (0.55), and the module doc.
- `src/main.rs`: 50 new SPOKEN rows (105 in all), 04 and 05 in the
  prediction test (answered by "Five." and "Exactly five"), and the crate
  doc.
- `docs/storyboard.md` gets chapters 04 (84.7 s, 8 beats) and 05 (77.4 s, 7
  beats), with pictures matched to the source, and the status rows.
- `docs/storyboard-draft.md` now holds 06 to 16. Its 52-line 04/05 block
  was cut after asserting that both headings occur once.
- `script.tsv` is regenerated with the tool's `script_text`: 43 lines.
- `docs/pauses.tsv` gets 4 rows (04_2, 05_1, 05_5, 05_7), 57 in all, and
  the header's list of scene holds.
- `docs/protected.tsv` gets 7 rows for 04 and 05, 17 in all.
- `lexicon.json` and `docs/brief.md`: the new risks. The brief's Length,
  Scope and Protected rows still described chapter 01 alone and are brought
  up to date.
- Spoken changes from the draft:
  - Every number word needs a fact, so "one handful" became "a single
    handful" (04_2, 04_8, 05_7).
  - 04_4 now counts the gaps on the friends' own line ("Of the sixty boxes
    from one to sixty, five are the friends', so the gaps hold sixty minus
    five, fifty-five") before 04_5 gives the rule for any five.
  - 04_7 now says "never guesses below the largest number it saw".
  - 05_1 names the 71-box company's "over thirteen million handfuls of
    five" (it said "seventy-odd … millions").
  - 05_3 works the pair two and five before the rule for a pair; 05_4 gives
    the high pair first.
  - 05_5 now says "Some are too low and some too high."

### Checks actually run for 04/05 (2026-10-24, to 09:12 UTC)

These ran on scratch copies outside the repository: `d`, the live topic
tree, and `e`, `d` with the edits. Both new chapter files were copied in
again after their last change, and `d` was re-checked equal to the live
tree, apart from the hook's `.ee`.

- `python3 -I -B oracle.py` on `e`: exit 0, 176 facts, 35 cross-checks, 0
  failed, stderr kept.
- The storyboard tool's `check`: 01 74.8 s, 02 84.6 s, 03 67.3 s, 04
  84.7 s, 05 77.4 s, 6.5 min, no problems. The draft is 06 to 16, 12.4 min,
  no problems. `script.tsv` equals the tool's `script_text` output.
- `rustfmt --edition 2024 --check --config max_width=120` on all 14 Rust
  files of `e`: pass. Differences it reported along the way were applied by
  hand.
- `protected_check.py` on `e`'s list and script: 17 listed, 0 missing,
  PASS.
- The scratch text-test mirror, extended to 04 and 05, gave 105 SPOKEN rows
  against the oracle's values, 57 pause rows, 5 chapters and 0 failed. It
  checks number-word coverage, the predictions, the registry against the
  storyboard, the draft continuing at 06, and every listed and `wait_for`
  phrase of 02 to 05 (once, in order).
- The patch round trip: `git apply --check` and `git apply` on a fresh
  copy of `d` exited 0, and `diff -r` against `e` exited 0. `git apply
  --check` on the live tree also exited 0. Rebuilding the patch after the
  last chapter-file fixes gave the same bytes.
- Scratch heuristics, which are not rustc or Clippy:
  - Unused imports: only the known `Timed` false positive (04 uses
    `.rt`).
  - Dead items and fields: after removing `FlowAt.right`, only parameters
    misread as fields.
  - Constant-only asserts: in the new files, only asserts that go through
    locals, a kind root's Clippy has passed before.
- Frozen bytes are unchanged:
  - `friends_line.rs` `89a3ed0d…`;
  - `a_thousand_boxes.rs` `d70d7dd1…`;
  - `draw.rs` `a28b04a8…`;
  - `docs/integration-02-03.patch` `843c0cf8…`;
  - root's `counting_the_gaps.rs` `04647539…`.

## Chapters 06 and 07 (2026-10-24)

Root's assignment (reply to Mail 6886, and Mail 6888): new, unregistered
`06_gaps_trade_places` and `07_right_on_average`.

- 05's ten-pair table stays in place.
- The pair-to-gap-list correspondence and its equal chances are computed
  and shown before any two gaps trade.
- The friends' five boxes come back for the hidden sixth gap.
- 07 builds the general proof from that correspondence and keeps one
  handful's average apart from the average over handfuls.
- The one-box prediction stays protected until its answer, and the formula
  for any handful size comes last.
- Emphasis uses outlines only.

Neither file has been compiled, and none of their tests has run. `mod.rs`
names them only after both patches are applied. 04/05's frozen files and
patch were not touched.

- `src/chapters/gaps_trade_places.rs` (sha256 `acb757aaff7b…`, 1,643
  lines, 9 tests): chapter 06, on 05's table.
  - The table's geometry constants are copied from `every_handful.rs`,
    which is frozen and keeps them private, so the rows sit where 05 left
    them.
  - 06_1: each row's three unseen squares become gap squares, and the heads
    "below", "between" and "above" come in.
  - 06_2: halos on rows 1,2 and 2,5. Their lists 0 0 3 and 1 2 0 are written
    on their words, then the other eight lists cascade in. Then "below +
    between + above = 3".
  - 06_3: the list 1 2 0 is read back onto five empty squares, one step per
    phrase ("skip one box", "take the next, skip", "skip two", "take the
    next, and stop", "and stop"), with a halo on the gap being read. Halos
    then compare the built row with the table's row 2,5.
  - 06_4: "pair ↔ gap list" (the arrow is a stroke, not a glyph), then
    "every split of 3 into 3 gaps, once each" with a halo round all ten
    lists, then "each pair: 1 in 10" and "each list: 1 in 10". A 2.5 s hold
    follows.
  - 06_5, the one worked trade:
    - Row 1,2's list is written again at the right as a separate copy;
      nothing is laid over the table.
    - Its below and above entries trade places (lift, slide, settle), and
      the copy slides to the height of row 4,5, whose own list is 3 0 0.
    - Halos on the copy and on row 4,5 stand through a 2.5 s hold.
  - 06_6: partner halos sweep the table, one pair of rows at a time:
    1,2↔4,5, 1,3↔3,5, 1,4↔2,5 and 2,3↔3,4. Then rows 1,5 and 2,4 each get a
    halo as their own partners. Then "below: 0, 0, 0, 0, 1, 1, 1, 2, 2, 3"
    and the same for above.
  - 06_7: the same for between, then a halo round all three.
  - 06_8: the column sums (10 each) and "average = sum ÷ 10" with the
    averages (1 each; the top gap's in the hero colour). Then "3 unseen ÷ 3
    gaps = 1 each" and a halo on the top gap's average, with a 2.5 s hold.
  - 06_9 and 06_10, back to the friends' boxes:
    - 06_9: everything fades. The friends' line comes back as chapter 04
      drew it, with the badge and the five arcs with their counts, then the
      countdown. Nothing above 60 is shown yet.
    - 06_10: the dashed bracket from 61 and its "?", then "small company:
      each gap averaged 1", and a halo round the five arcs.
  - `GapLine` (the friends' line with arcs, counts, dashes and "?"),
    `glow_each`, `matched` and `list_of` are the builders 07 reuses.
  - `checked()` refuses each of these, and every negative control asserts
    its reason:
    - pairs that are not every pair once, in order;
    - lists that are not each pair's gaps adding up to 3;
    - lists that are not every split once each, at 1 in 10;
    - worked rows other than 1,2 and 2,5, or a read-back that does not
      rebuild 2,5;
    - a trade that does not land on its partner row, or partners that are
      not each other's;
    - columns without the same values, sums or even share;
    - friends' gaps or positions that disagree with the line.
  - Tests:
    - the picture is the facts;
    - a list read backwards rebuilds its pair;
    - every row has a partner, with a non-involution as the negative
      control;
    - trading gaps keeps the lists for every k ≤ 4 and N ≤ 9. Weighting
      pairs by their larger number breaks it: the top gap averages 3/4 and
      the gap below 9/8;
    - the refusals;
    - every waited-for phrase is once in its line, in order;
    - the table, the right side and the friends' line fit the frame.
- `src/chapters/right_on_average.rs` (`98f27cf8f107…`, 1,254 lines, 8
  tests): chapter 07. It opens on 06's line with every gap drawn and the
  badge.
  - 07_1: "count = N", then "13 + 12 + 5 + 17 + 8 + top gap = N − 5" piece
    by piece, with halos on the five counts and the top gap.
  - 07_2: "handful ↔ 6 gaps adding to N − 5", then a halo on the boxes.
  - 07_3: a halo on the badge, then on N − 5. Then "each gap averages (N −
    5) ÷ 6", with "small company: 3 ÷ 3 = 1" beside it in dim, and a 2.5 s
    hold.
  - 07_4: the top gap gets a halo, then "guess = ( 13 + 12 + 5 + 17 + 8 ) ÷
    5 = 11" piece by piece. The guess's half-strength arc lands over 61 to
    71.
  - 07_5, one handful against all of them:
    - A halo on the 11.
    - The small company's inset comes in one row per phrase: pairs, guesses
      (0, .5, 1, 1.5, .5, 1, 1.5, 1, 1.5, 1.5) and top gaps (3, 2, 1, 0, 2,
      1, 0, 1, 0, 0).
    - Halos on the three pairs where guess and top gap agree (1,4, 2,4 and
      3,4) stand through a 2.5 s hold.
  - 07_6: "average 1" after each row, with a halo. Then "guess: averages (N
    − 5) ÷ 6 too", halos on the top gap and the guess's arc, and a 2.5 s
    hold.
  - 07_7: the inset and the panel go, and the derivation comes in line by
    line: "count = largest + top gap", "estimate = largest + guess",
    "estimate − count = guess − top gap", and "on average: (N − 5) ÷ 6 − (N −
    5) ÷ 6 = 0". Then the definition at the upper left, with a halo and a
    2.5 s hold.
  - 07_8: everything but the badge goes. A fresh line shows box 37, its arc
    with no count, and the top gap's dashes and "?". The countdown follows.
    No count, guess, formula or 73 is on screen.
  - 07_9: the one-box recipe and "= twice the largest − 1" come in on the
    answer. Then halos on the two gaps, the guess's arc over 38 to 73, the
    count 36 on "thirty-six boxes below it", and "37 + 36 ÷ 1 = 73" piece by
    piece. The green tick at 73 comes last.
  - 07_10: "k numbers: k + 1 gaps", "each gap averages (N − k) ÷ (k + 1)",
    "estimate = largest + (largest − k) ÷ k" and, in dim, "k = 5: 60 + 55 ÷
    5 = 71". Card D comes in on "it's right on average". A halo goes on the
    badge on "as long as the handful is fair", and the card stays until it
    can be read (15 characters a second).
  - `checked()` refuses each of these:
    - the friends' gaps, total or positions disagreeing with the line;
    - an average or estimate other than the one recipe;
    - a gap average other than (N − 5) ÷ 6;
    - pairs not every pair once;
    - guesses or top gaps not each pair's own;
    - an agreement count or averages other than the computed ones;
    - a one-box case not 37, 36 below, 2 gaps and 73.
  - Tests:
    - the picture is the facts;
    - the refusals;
    - the guess and the top gap average the same for every k ≤ 4 and N ≤
      10. Using the widest seen gap as the guess is the negative control:
      it averages above N;
    - one box doubles and subtracts one;
    - every waited-for phrase is once in its line, in order, and 07_8 says
      no answer word;
    - the panel and the inset fit;
    - the guesses and ticks fit the line;
    - card D fits in three lines at every lower-third width and is read in
      under 15 s, with one more clause as the negative control.
- No typeset text is indicated, scaled, recoloured or laid over a copy in
  06 or 07. Every emphasis is a separate rounded outline (`halo`,
  `glow_each`). A change of words is a replacement, and the trade moves a
  separate copy.
  - Nothing dims by opacity. Dim text is made dim.
  - There is no `indicate`, `flash_all` or `flash_lagged` in either file.
  - Root's stroke-only rule therefore has no site here.
- Defects found and fixed before handoff, all by hand:
  - 11 rustfmt differences in 06 and 9 in 07;
  - constant-only asserts rewritten as `const { assert!(..) }`;
  - always-true asserts replaced by real bounds;
  - a wrong negative-control expectation for `partner_groups` is now an
    exact `assert_eq`;
  - 07's `(k + 1)` substitution, which left brackets;
  - a typed 0.85, now `MATCH_GAP` and `MATCH_ARROW`, imported from 06;
  - a typed 71 in a test's padding clause, now words;
  - the 07 storyboard note still named a pause row after 07_5, where the
    standing halos' hold already gives the time; the note now says so.

### The 06/07 integration patch

`docs/integration-06-07.patch` (sha256 `1168c161a164…`, 841 lines, 11
files) applies after `docs/integration-04-05.patch`, against the live tree's
current bytes. It touches no chapter source; the two new files are already
in place.

- `oracle.py` and `src/facts.rs` gain the same 22 facts (176 → 198):
  - `G-F6.low_gaps`, `high_gaps`, `low_traded` and `low_partner`;
  - `G-F6.partners`, `column_sums` and the `below_values`,
    `between_values` and `above_values` columns;
  - `G-F6.each_chance` and `top_average`;
  - `G-F7.tiny_guesses`, `tiny_guesses_shown`, `tiny_tops`, `tiny_agree`,
    `tiny_guess_average` and `tiny_top_average`;
  - `G-F7.one_box_handful`, `one_box` (37), `one_box_below`,
    `one_box_gaps` and `one_box_estimate`.

  `facts.rs` adds `ONE_BOX`, and its count test becomes 198. The oracle
  gains 7 cross-checks (42 in all):
  - partners are an involution, and the three columns hold the same values;
  - estimate − count = guess − top gap, row by row;
  - one box gives 2m − 1 for every m to 40;
  - trading keeps the lists, and the guess averages (N − k) ÷ (k + 1), for
    k ≤ 4 and N ≤ 11;
  - two negative controls: weighted pairs (3/4 and 9/8), and the widest
    seen gap averaging above N.
- `src/chapters/mod.rs`: the two modules, registry rows 06 (0.55) and 07
  (0.6), and the module doc.
- `src/main.rs`: 69 new SPOKEN rows (174 in all), the crate doc, and 06 and
  07 in the prediction test. 06 asks "Where is the sixth gap?" and is
  answered by "Above sixty". 07 asks about "a handful of just one box" and
  is answered by "It doubles the number".
- `docs/storyboard.md` gets 06 (136.4 s estimate, 10 beats) and 07 (166.0
  s, 10 beats), with pictures matched to the source. Both carry the tool's
  advisory for a long chapter; the estimate is not a cap.
- `docs/storyboard-draft.md` now holds 08 to 16.
- `script.tsv` is regenerated with the tool's `script_text`: 63 lines.
- `docs/pauses.tsv`: the draft's four 06/07 rows, written for the shorter
  draft wording, give way to 9 semantic rows (62 in all): 06_1, 06_3,
  06_4, 06_5, 06_8, 06_10, 07_4, 07_7 and 07_9. Its header now lists 06's
  and 07's inspection holds as scene holds.
- `docs/protected.tsv` gets 10 rows, 27 in all. They cover the matching's
  chance, the trade landing on a real row, the columns, the even share,
  (N − 5) ÷ 6, "one handful's average", "agree only three times in ten",
  the unbiased conclusion, the spoken condition, and the one-box answer.
- `lexicon.json` and `docs/brief.md` get the new pronunciation risks, and
  the brief's Length, Scope and Protected rows are brought up to date.
- Protected visual holds, proposed for root's director map in its own
  shape (`video`, `cue`, `next_cue`, `reason`, and `evidence` naming the
  chapter source and its sha256):
  - `06_4 → 06_5`: read the matching and both chances;
  - `06_5 → 06_6`: compare the traded copy with row 4,5;
  - `06_8 → 06_9`: read the sums, averages and even share;
  - `07_3 → 07_4`: read (N − 5) ÷ 6 beside the small company;
  - `07_5 → 07_6`: inspect the three agreeing pairs;
  - `07_6 → 07_7`: read both averages and the guess's line;
  - `07_7 → 07_8`: read the four derivation lines and the definition.

  `07_10`'s card hold has no next cue in `script.tsv` (08 is not
  registered), so it cannot be a map row yet. `06_9 → 06_10` and `07_8 →
  07_9` are the prediction countdowns.

### Checks actually run for 06/07 (2026-10-24, to 10:20 UTC)

These ran on scratch copies outside the repository:
- `f`: the live topic tree with the 04/05 patch and the two new files;
- `g`: `f` with the 06/07 edits.

The new chapter files were copied into both after their last change and
compared byte for byte with the live files.

- `python3 -I -B oracle.py` on `g`: exit 0, 198 facts, 42 cross-checks, 0
  failed, stderr kept.
- The storyboard tool's `check`, with no problems:
  - 01 74.8 s, 02 84.6 s, 03 67.3 s, 04 84.7 s, 05 77.4 s, 06 136.4 s and
    07 166.0 s: 11.5 min, with the long-chapter advisory on 06 and 07;
  - the draft, 08 to 16: 10.0 min.
- `rustfmt --edition 2024 --check --config max_width=120` on all 16 Rust
  files of `g`: pass.
- `protected_check.py` on `g`'s list and script: 27 listed, 0 missing,
  PASS.
- The scratch text-test mirror, extended to 06 and 07: 174 SPOKEN rows
  against the oracle's values, 62 pause rows, 7 chapters and 0 failed. It
  checks:
  - number-word coverage and the two predictions;
  - the registry against the storyboard, and the draft continuing at 08;
  - every `wait_for` phrase of 02 to 07, once and in order.

  A separate script found 0 of the 69 new SPOKEN values differing from the
  oracle's output.
- The patch round trip:
  - on a fresh copy of the live topic tree, `git apply` of the live 04/05
    patch, then of the 06/07 patch, each exited 0 after `--check`;
  - `diff -r` against `f` after the first, and against `g` after the
    second, showed only `WORK_STATUS.md`, the delivered 04/05 patch and the
    hook's git-ignored `.ee` directories.
- Scratch heuristics, which are not rustc or Clippy:
  - Unused imports: none in the two new files, `facts.rs` or `main.rs`.
  - Dead items and fields: none in 07; in 06, only function parameters
    misread as fields.
  - Constant-only asserts: one `str::contains` assert in 07's card test,
    which is not a const expression, and the through-locals kind in frozen
    files that root's Clippy has passed.
- Frozen bytes are unchanged:
  - `five_boxes.rs` `619f84c6…`, `a_thousand_boxes.rs` `d70d7dd1…` and
    `counting_the_gaps.rs` `04647539…`;
  - `draw.rs` `a28b04a8…` and `friends_line.rs` `89a3ed0d…`;
  - `what_we_assume.rs` `f836ac7f…`, `every_handful.rs` `3152edb9…` and
    `glyphs.rs` `f64f9ee7…`;
  - live `mod.rs` `f301298f…`;
  - the patches `843c0cf8…` (02/03) and `d28589ef…` (04/05).

## Chapters 08 and 09 (2026-10-24)

Root's assignment (Mail 6901): new `08_one_estimate` and `09_how_far_off`,
on the same boxes and under the same fair-handful condition, with
`docs/integration-08-09.patch` composing after 04/05 and 06/07.

- 08 keeps apart:
  - the 20 computed draws;
  - the exact distribution over all 13,019,909 handfuls;
  - the average miss;
  - the standard deviation.

  It counts by the largest number, so it needs no per-frame or full
  enumeration. The exact-71 answer stays hidden through the prediction.
- 09 keeps the count fixed and unknown, and gives the 95% its
  repeated-sampling meaning.
  - It shows the actual 107/108 cutoff and the 79 for ten boxes, from exact
    chances.
  - It never calls the 95% a chance about the count.
  - Viewers inspect the chances and the coverage experiment before any
    formula.

Neither file has been compiled, and none of their tests has run. Frozen
files 01 to 07 and the three earlier patches were not touched.

- `src/chapters/one_estimate.rs` (sha256 `b271d6dbd62b…`, 1,578 lines, 8
  tests): chapter 08. It opens on the friends' line with the boxes, the green
  tick at 71 and the badge.
  - 08_1: a company of exactly 71 boxes, an upright ink mark at 71, and
    "count = 71".
  - 08_2: the first twenty handfuls of chapter 02's stream 3 drop in one row
    at a time. Each row is five gold rings at half strength, and each row's
    estimate drops as a green dot onto a stack over the line, no stack more
    than three high. The rows keep landing after the words end.
  - 08_3: the lowest and highest (44.6, 84.2), the three exact 71s and the
    draws' average.
  - 08_4: the draws go and every handful comes in as 67 green bars, one per
    largest number m.
    - Each bar holds C(m − 1, 4) handfuls (`model::with_largest`) at that
      stack's estimate, so no handful is listed.
    - The table's rows are m = 5 (1 handful), 6 (5) and 71 (916,895).
    - A 2.5 s hold follows.
  - 08_5, the prediction: the bar at 71 is missing, with a "?" over its
    place. Nothing shows that stack's size or 1 in 29.
  - 08_6: the bar rises, then "exactly 71: 1 in 29" and "20 draws: 3
    exactly 71".
  - 08_7: the balance point at 71 and the two sides: 58% over, by about 8;
    under, by 12. A 2.5 s hold follows.
  - 08_8: the average miss 9.2 and the standard deviation 11.7, defined as
    "√(average squared miss)", with a 2.5 s hold.
  - 08_9: halos on the balance and the widths, "our 71" as one estimate, and
    card E. Card E's formula comes after the bars, and it stays until it can
    be read.
  - `checked()` refuses each of these:
    - a company count other than the friends' estimate;
    - a handful total other than C(71, 5);
    - stacks not running from 5 to 71 with the counted sizes, with the last
      not the tallest, or without exactly one stack at exactly 71;
    - an average other than 71, or exact and over shares other than the
      computed ones;
    - overshoot or undershoot, average miss or variance other than every
      handful's;
    - draws that are not stream 3's handfuls, with their estimates, range,
      average and exact count;
    - a stack of draws over three high.
  - Tests:
    - the picture is the facts;
    - fractions and decimals are read and shown exactly;
    - the refusals;
    - counting by the largest number equals listing every handful for k ≤ 4
      and N ≤ 12, the average miss included;
    - every waited-for phrase is once in its line, in order;
    - the panel, the table and the bars fit the frame;
    - the words under the line keep apart;
    - card E fits in three lines and is read.
  - Builders 09 reuses: `count_text`, `count_mark`, `plain_dot`, `bar`,
    `BAR_O`, `grow_bars`, `span_points`, `SPAN_W`, `right_at`, `fraction`
    and `approx`.
- `src/chapters/how_far_off.rs` (`7a47a777adca…`, 1,351 lines, 9 tests):
  chapter 09. It opens on the friends' line with the boxes and the badge.
  - 09_1: "count: fixed, unknown", and a halo round box 60. The line's end
    slides out to 130, a layout value that is never written or said. Then
    one bar per count from 60, each the exact chance C(60, 5) ÷ C(N, 5) that
    a fair handful is all at most 60.
  - 09_2: "71: 42%" and "100: 7%" under the line, with halos on their bars.
  - 09_3: the dashed 5% line, "107 boxes: 5.1%" and "108 boxes: 4.9%". The
    bars from 108 on are replaced by faint ones.
  - 09_4: the range 60 to 107 under the line, "range: 60 to 107", and a
    2.5 s hold.
  - 09_5: everything but the friends' dots goes, and those dots turn plain.
    Then the mark at 71 and "count = 71". The twenty draws of 08 land as
    rows, each with its range in dim. The rows keep landing after the words
    end.
  - 09_6: the 19 ranges that cover 71 become ink. The miss is marked
    "largest 38". Then "every handful: 96% cover 71" and "any count: at
    least 95%", with a 2.5 s hold.
  - 09_7: "95% confidence", with halos on the fixed mark and on the moving
    ranges.
  - 09_8: the 09_4 picture comes back, then the division in three panel
    lines ending "N = 107: 5,461,512 ÷ 106,308,566 ≈ 5.1%", with a halo on
    bar 107 and a 2.5 s hold.
  - 09_9, the prediction: "10 boxes, largest still 60" and the countdown.
  - 09_10: the bars for ten, the range 60 to 79 and card F. Card F says the
    95% is about the method, "not a chance about N", and it stays until it
    can be read.
  - `checked()` refuses each of these:
    - a handful not topped by the count's lower bound;
    - a level other than 1 in 20 with 95% confidence;
    - a printed chance other than C(m, k) ÷ C(N, k);
    - a range top other than `upper_bound`;
    - bars that do not fall all along the shown line;
    - a division other than the handfuls of 5 from 60 and from 107;
    - a ten-box range other than 60 to 79, or not narrower;
    - draws that are not 08's handfuls, or ranges not built from each
      largest number, with exactly one miss;
    - coverage at 71 other than every handful's, or a worst coverage up to
      130 under 95%.
  - Tests:
    - the picture is the facts;
    - ranges are read as printed;
    - the refusals;
    - the range's top is where the chance crosses the level, by brute force
      for k ≤ 4 and m ≤ 12. The (60/N)⁵ guess, 109, is not the top;
    - ranges built this way cover every count up to 30 at least 95% of the
      time for k ≤ 4. Each handful is listed and counted by its largest
      number. The negative control: at a level of 1 in 10 some cases fall
      short;
    - the confidence is neither a posterior nor the estimate. Flat priors
      to 200 and to 1,000 give 91.8% and 91.2%, and ranges topped at the
      estimate cover 71 under two times in three;
    - every waited-for phrase is once in its line, in order, and 09_9 says
      no answer word;
    - the bars, the rows and the words fit the frame;
    - card F fits in three lines and is read.
- No typeset text is indicated, scaled, recoloured or laid over a copy.
  - Every emphasis is a separate rounded outline (`halo`, `glow`).
  - A change of words or of strength is a replacement.
  - Bars and dots are fills, and the rings are `friends_line::ring`.
  - Nothing stroked is dimmed with `set_opacity`.
- Defects found and fixed before handoff, all by hand:
  - a wrong import name;
  - every rustfmt difference;
  - two index loops, now `zip`/`enumerate`;
  - a manual range check, now `contains`;
  - 09_2's labels colliding with taller bars, now under the line;
  - adjacent rows' rings overlapping, now radius 0.038 at a 0.11 step;
  - dots past 84 left faded after the zoom, now refilled;
  - card F at four lines, now three;
  - the coverage test finding a range top for every handful, now once per
    largest number.

### The 08/09 integration patch

`docs/integration-08-09.patch` (sha256 `75479840c6e7…`, 1,067 lines, 12
files) touches no chapter source but `mod.rs`. The two new files are
already in place.

Its exact base is the live topic tree as of 11:37 UTC, the moment root's
application of the 06/07 patch landed. Root had applied 04/05 earlier and
reworded the brief's Length row. Before that landed, the frozen 06/07 patch
failed `--check` at `docs/brief.md:18`, because its Length row still carried
the replaced sentence. I built the base in scratch as live plus that patch,
with only root's sentence put in on both sides of that row. Root's result
has the same bytes in all 12 files. Before → after, as sha256 prefixes:

| File | Base | After |
|---|---|---|
| `docs/brief.md` | `ed2029e5` | `00075755` |
| `docs/pauses.tsv` | `65ab806d` | `b2661ffc` |
| `docs/protected.tsv` | `da26b659` | `cf432842` |
| `docs/storyboard-draft.md` | `f057daf3` | `d15d9127` |
| `docs/storyboard.md` | `5969ea58` | `6377e52e` |
| `lexicon.json` | `1843653a` | `7a94cd48` |
| `oracle.py` | `37a1b356` | `c6a7b65d` |
| `script.tsv` | `a2c96092` | `4a7d52fc` |
| `src/chapters/mod.rs` | `36cf4b8d` | `f160d8ec` |
| `src/facts.rs` | `4b5d7153` | `aa733889` |
| `src/main.rs` | `576e9956` | `3521f5c9` |
| `src/model.rs` | `13075d70` | `171dcb3d` |

- `oracle.py` and `src/facts.rs` gain the same 31 facts (198 → 229).
  - 13 in `G-F8`: the stacks and their sizes, the exact three, the
    overshoot and undershoot, the variance, and the rounded widths.
  - 18 in `G-F9`: the second count (100), the level and confidence
    percents, the chances at 107 and 108, the cutoff, the division, ten
    boxes, the twenty ranges with their 19 covers and the miss at 38, and
    the coverage of 71.

  `facts.rs` adds three named constants, and its count test becomes 229.
  The oracle gains 15 cross-checks (57 in all), 5 of them negative
  controls:
  - the median estimate 74.6 is not the balance point;
  - the average miss squared is below the variance;
  - ranges topped at the estimate cover 71 only about 62% of the time;
  - the (60/N)⁵ guess gives 109;
  - flat-prior posteriors of 91.8% and 91.2%, never 95%.
- `src/model.rs`: `Q::sqrt_about1`, the square root rounded half up to one
  decimal, found exactly from the integer square root of 400 × the value.
  Edge asserts are added to the existing rounding test.
- `src/chapters/mod.rs`: the two modules, registry rows 08 (0.55) and 09
  (0.6), and the module doc.
- `src/main.rs`: 65 new SPOKEN rows (239 in all), the crate doc, and 08 and
  09 in the prediction test.
  - 08 asks "…the estimate is exactly seventy-one, the true count?" and is
    answered by "About one handful in twenty-nine".
  - 09 asks "…would the range get wider or narrower?" and is answered by
    "Narrower".
- `docs/storyboard.md` gets 08 (135.7 s estimate, 9 beats) and 09 (142.3 s,
  10 beats), with pictures matched to the source. Both carry the tool's
  advisory for a long chapter; the estimate is not a cap.
- `docs/storyboard-draft.md` now holds 10 to 16.
- `script.tsv` is regenerated with the tool's `script_text`: 82 lines.
- `docs/pauses.tsv`: the draft's 7 rows for 08 and 09 give way to 12
  semantic rows (67 in all).
  - 08: 08_1, 08_4 twice, 08_6, 08_7 and 08_8 twice;
  - 09: 09_1, 09_3, 09_6, 09_7 and 09_10.

  Root's `docs/pause-overrides.json` (11:38 UTC) holds 06/07's intentions in
  the narration director's override shape. It is root's file and is not in
  the patch. The same 12 rows in that shape went to root in the handoff
  mail. Their role is the director's punctuation default at each boundary,
  as in root's entries: ten `thought` and two `question`.
- `docs/protected.tsv` gets 9 rows, 36 in all. They cover:
  - unbiased as the recipe's property;
  - right on average but rarely exact;
  - one realised estimate;
  - the fixed count;
  - the exact cutoff at 108;
  - the range as what varies;
  - "not a ninety-five percent chance";
  - "never fewer than ninety-five";
  - the ten-box range.
- `lexicon.json` and `docs/brief.md` get the new pronunciation risks:
  - the decimals of 08_3 and 08_8;
  - the large numbers;
  - "standard deviation" and "confidence".

  The brief's Length, Scope and Protected rows are brought up to date.
  Root's Length sentence is kept.
- Protected visual holds, proposed for root's director map in its own shape
  (`video`, `cue`, `next_cue`, `reason`, and `evidence` naming the chapter
  source and its sha256):
  - `08_2 → 08_3`: watch the twenty rows land and their estimates stack;
  - `08_4 → 08_5`: read the 67 bars and the table;
  - `08_7 → 08_8`: read the balance point and both sides;
  - `08_8 → 08_9`: read the two widths and the definition;
  - `08_9 → 09_1`: read card E;
  - `09_4 → 09_5`: read the range and the level;
  - `09_5 → 09_6`: watch the twenty ranges build;
  - `09_6 → 09_7`: inspect the covering ranges and the miss;
  - `09_8 → 09_9`: read the division at 107.

  `09_10`'s card hold has no next cue in `script.tsv` (10 is not
  registered). `08_5 → 08_6` and `09_9 → 09_10` are the prediction
  countdowns.

### Checks actually run for 08/09 (2026-10-24, to 11:47 UTC)

These ran on scratch copies outside the repository:
- `j`: the base;
- `k`: `j` with the 08/09 edits.

The new chapter files were copied into both after their last change and
compared byte for byte with the live files.

- `python3 -I -B oracle.py` on `k`: exit 0, 229 facts, 57 cross-checks, 0
  failed, stderr kept.
- The storyboard tool's `check`, with no problems:
  - 01 to 07 as before, 08 135.7 s and 09 142.3 s: 16.2 min, with the
    long-chapter advisory on 06 to 09;
  - the draft, 10 to 16: 7.8 min.
- `rustfmt --edition 2024 --check --config max_width=120` on all 18 Rust
  files of `k`: pass.
- `protected_check.py` on `k`'s list and script: 36 listed, 0 missing,
  PASS.
- The scratch text-test mirror, extended to 08 and 09: 239 SPOKEN rows
  against the oracle's values, 67 pause rows, 9 chapters and 0 failed. It
  checks:
  - number-word coverage and the two predictions;
  - the registry against the storyboard, and the draft continuing at 10;
  - every `wait_for` phrase, once and in order.
- A Python mirror of 08 and 09's frame and numeric test assertions, with the
  constants copied from the sources: 809 checks, 0 failed.
- The coverage test, mirrored in exact fractions with the same per-largest
  cache: worst coverage 248/261 (k = 4, N = 30), and 86 short cases at 1 in
  10.
- The patch round trip:
  - Before root's 06/07 application, on a fresh copy of the live tree, the
    frozen 06/07 patch failed `--check` (brief.md:18). The same patch with
    only root's Length sentence substituted passed and applied, giving
    `j`. The 08/09 patch then passed `--check` and applied, giving `k`.
  - After root's application, on a fresh copy of the live tree, the 08/09
    patch passed `--check`, applied, and gave `k`. Reverse `--check` then
    passed.
  - Each `diff -r` showed only `WORK_STATUS.md` and the delivered patch
    files. After root's application it also showed root's later files: the
    edit to `right_on_average.rs` and the new `docs/pause-overrides.json`.
- The 12 pause rows in root's override shape, through the narration
  director's own `directions()` (read-only, on the target's `script.tsv`)
  together with root's 10 entries: 22 authored boundaries, each an
  unambiguous phrase. The negative control, "seventy-one" in 08_1, which
  occurs twice, was refused.
- Scratch heuristics, which are not rustc or Clippy:
  - Unused imports: none in the two new files, `mod.rs`, `facts.rs`,
    `model.rs` or `main.rs`. The flag on `Timed` in 08 is a trait used by
    `.rt(rt)`.
  - Dead items and fields: none in either new file.
  - Unused `let` bindings and parameters: none in the two new files or in
    the patched `main.rs`, `facts.rs` and `model.rs`. The check was written
    after root's 07 repair (below). It finds that repair's binding in 07's
    pre-repair bytes, and a parameter planted in a scratch copy.
  - Constant-only asserts: two method-call asserts in `model.rs`, which are
    not const expressions, and one through-locals assert in 08's frame test,
    the kind root's Clippy has passed in frozen files.
- Frozen bytes are unchanged:
  - `five_boxes.rs` `619f84c6…`, `a_thousand_boxes.rs` `d70d7dd1…` and
    `counting_the_gaps.rs` `04647539…`;
  - `draw.rs` `a28b04a8…`, `friends_line.rs` `89a3ed0d…` and `glyphs.rs`
    `f64f9ee7…`;
  - `what_we_assume.rs` `f836ac7f…`, `every_handful.rs` `3152edb9…` and
    `gaps_trade_places.rs` `acb757aa…`;
  - the patches `843c0cf8…` (02/03), `d28589ef…` (04/05) and `1168c161…`
    (06/07).

  `right_on_average.rs` is now `88f02a26…`. That is root's edit at 11:37
  UTC, which drops the unused binding of 07_8's `say`; I did not touch the
  file.

## Chapters 10 and 11 (2026-10-24)

Root's assignment (Mail 6917): new `10_a_rival_recipe` and
`11_the_largest_says_it_all` in new disjoint modules, on the friends' 60 and
the same five boxes, under the same fair-handful condition.
`docs/integration-10-11.patch` composes after the frozen 08/09 patch.

- 10 earns the doubling recipe, 2 × average − 1, from the middle number
  (N + 1) ÷ 2.
  - On the strange handful 2, 3, 4, 5, 60 it says 28.6, which that
    handful's own box 60 rules out.
  - It keeps the average miss (ignoring direction) apart from the standard
    deviation (from squared misses), with exact values for both recipes.
    This replaces draft 10_5's ambiguous "misses … on average".
  - Viewers compare the two recipes over every handful before the chapter
    asks what the other four are worth.
- 11 shows the conditional distribution of the other four, given largest
  60. Each group of four below 60 has chance 1/455,126 at every count from
  60 up. Only the chance that 60 is the largest changes: 1 in 12 at the edge
  N = 60.
  - Viewers predict before "sufficient" is named.
  - 11_8 keeps the fair-sampling assumption explicit.

Neither file has been compiled, and none of their tests has run. These were
not touched:
- the frozen files 01 to 09;
- the four earlier patches;
- root's `docs/pause-overrides.json`.

- `src/chapters/a_rival_recipe.rs` (sha256 `e72331a3978b…`, 1,527 lines, 8
  tests): chapter 10. It opens on the friends' line as 08 did, with the
  boxes on their gold ticks, the green tick at 71 and the badge.
  - 10_1: "middle = (count + 1) ÷ 2", an ink span from 1 to 71 with a
    fulcrum at 36, then "count = 2 × middle − 1".
  - 10_2: the span goes. The rule gives way to "doubling recipe = 2 ×
    average − 1", with a purple halo.
  - 10_3: "(14 + 27 + 33 + 51 + 60) ÷ 5 = 37" and a purple mark at 37 under
    the line. Then "2 × 37 − 1 = 73" and a purple tick at 73, beside the
    green 71 over the line.
  - 10_4: halos on the badge, the middle's line and the recipe, for
    "unbiased too".
  - 10_5, the prediction: the friends' working goes. A gold halo goes round
    box 60, the four smaller boxes fade, and gold rings at half strength
    land on 2, 3, 4 and 5. Nothing shows the new average.
  - 10_6:
    - "(2 + 3 + 4 + 5 + 60) ÷ 5 = 14.8" and "2 × 14.8 − 1 = 28.6";
    - a purple tick at 28.6;
    - a teal halo round box 60 and a green halo round 71.
  - 10_7: "71 boxes: 20.9% of handfuls / doubling below their own largest",
    then a green halo for "The gap recipe never does".
  - 10_8: everything on the line goes and the dots even out as the line's
    end slides to 140, a layout value that is never written or said.
    - Then "count = 71" and its mark, and 08's 67 green bars.
    - Over them, the doubling recipe's 111 steps rise as a stroke-only
      purple outline, with "5" and "137" at its ends.
    - The steps come from the sum-of-subsets table (handfuls counted by
      total). The tallest is 339,265 handfuls, at 71. No handful is listed.
  - 10_9: the fulcrum at 71 with "balance point: 71", and halos round both
    sets of bars together. Then the table:
    - "average miss" 9.2 (gap) against 14.3 (doubling), on "Ignoring
      direction";
    - "standard deviation" 11.7 against 17.8, on "The standard deviation".

    A 2.5 s hold follows.
  - 10_10: halos round the green bars and then the purple outline. The
    table stays until it can be read, counted from when it came up.
  - `checked()` refuses each of these:
    - a company count other than the friends' estimate;
    - a middle rule that does not give back every count from 1 to 200;
    - a recipe, total, average or doubling not from the friends' five, or
      not above 71;
    - either recipe not balancing at the count, over every handful;
    - a strange handful that does not keep box 60, or whose 28.6 is not
      under 60 while the gap recipe says 71;
    - a share below the largest other than the computed one, or any gap
      estimate below its largest;
    - bars that do not hold every handful once, a range other than 5 to
      137, or a tallest step not unique at 71;
    - widths other than the exact average misses and standard deviations.
  - Tests:
    - the picture is the facts;
    - the rules are read as the facts print them;
    - the refusals;
    - counting by total equals listing every handful, in small cases;
    - the doubling recipe at its edges;
    - the outline's steps join up;
    - every waited-for phrase is once in its line, in order;
    - the panel, the marks and the bars fit the frame.
- `src/chapters/the_largest_says_it_all.rs` (`16b1c070a5d7…`, 1,254 lines, 7
  tests): chapter 11. It opens on the friends' line with the boxes on their
  ticks and the badge. The gap arcs and counts are built with it, by 06's
  `GapLine` (as 07 does), and first shown in 11_7.
  - 11_1: the four smaller boxes go and their dots turn plain. "largest: 60"
    goes on the panel. On "any four different numbers below sixty", a span
    under the line over 1 to 59 comes in, with "any 4 of 1 to 59".
  - 11_2: the tiny company's ten pairs as a triangle, each pair in its
    larger number's column at its smaller number's height, with "5 boxes:
    10 pairs".
    - The column topped by 5 turns gold, with "5" and "topped by 5: 4 of
      10".
    - Row labels come in for "the smaller one is one…", and "each 1 in 4"
      on "so each is one time in four".
  - 11_3: triangles for 6 and 8 boxes beside it ("15 pairs", "28 pairs"),
    then their shares (4 of 15, 4 of 28) on "rarer". Their columns turn
    gold, a halo goes round each gold column, and "each 1 in 4" comes again.
  - 11_4: the triangles go and "455,126 groups of 4 below 60" goes on the
    panel, while the voice says "over four hundred and fifty thousand". The
    table follows: heads "count", "largest is 60" and "each group of 4".
    - Its rows come in on "With exactly sixty boxes", "with seventy-one" and
      "with a thousand": 1 in 12, ≈ 1 in 29 and ≈ 1 in 18 million. The last
      column is 1 in 455,126 on every row.
    - A 2.5 s hold follows, and the table stays until it can be read (8.1 s
      on the kit's estimate).
  - 11_5, the prediction: the question with the table standing, then the
    countdown. Nothing says "nothing" or "sufficient".
  - 11_6: a halo round the last column, then "the largest number:
    sufficient" and card G. Card G stays until it can be read: 186
    characters, 12.4 s.
  - 11_7: the table, the card and the panel's lower lines go. The four boxes
    come back with their arcs and counts. Then "13 + 12 + 5 + 17 + 8 = 55 =
    60 − 5", with 55 in the unseen colour, and the strange handful's "2, 3,
    4, 5: 1 + 0 + 0 + 0 + 54 = 55".
  - 11_8: a halo round the badge on "needs the handful to be fair", and
    another round the four smaller boxes on "they can still warn you". A
    0.6 s end beat follows.
  - Card G: "fair handful from any N ≥ 60, given largest 60: each group of
    4 below 60 is the others with chance 1/455,126. Only the chance of
    largest 60 depends on N: the largest is sufficient for N." It fits three
    lines at every lower-third width, and no formula is split.
  - `checked()` refuses 19 contradicted pictures, each for its own reason:
    - the friends' handful;
    - the pairs topped by 5, their shares and their equal chances;
    - the groups of four and their rounding (`facts::GROUPS_ROUND`);
    - the chances: 5/60 at the edge, strictly falling with the count, each
      group's chance equal to 1/455,126 at every row, and "1 in" exact only
      at the edge;
    - the gaps below 60: "fairly evenly" (2 × the largest gap < 55) and
      "nearly all" (10 × the largest gap > 9 × 55).
  - Tests:
    - the picture is the facts;
    - the 19 refusals;
    - given the largest, the others never depend on the count:
      - handfuls of 4 topped by 9 give the same others for every N from 9
        to 16, and their gaps total 9 − 4;
      - at 60, each group's chance is 1/C(59, 4) for every N from 60 to 300
        while the largest's chance strictly falls;
      - the edge N = 60 gives 5/60, and N = 59 gives 0;
    - unfair draws let the others carry a clue: each box is drawn in turn
      with weight ∝ its number, for (k = 2, largest 4) and (k = 3, largest
      6) and N from the largest to 3 more. The others' chances are unequal
      even at the edge and change with N. The same code with weight 1 gives
      equal chances that never change;
    - every waited-for phrase is once in its line, in order, and 11_5 says
      no answer word;
    - the panel, the pairs and the table fit the frame;
    - card G fits in three lines and is read.
- No typeset text is indicated, scaled, recoloured or laid over a copy.
  - Every emphasis is a separate rounded outline (`halo`, `glow`,
    `glow_each`).
  - A change of words is a replacement.
  - The purple outline is stroke-only and is cut into short pieces, so no
    long stroke sits under moving objects.
  - Dots are recoloured with `refill`, which applies to fills only.
- Defects found and fixed before handoff, all by hand:
  - every rustfmt difference: 6 in chapter 11, 4 in `main.rs`;
  - a closure bound `mut` that did not need it;
  - a possible `Q::new(1, 0)` panic on an empty column, now guarded;
  - constant-only asserts in both files, now `const { assert!(…) }`;
  - card G at four lines and with a split formula, now three lines with no
    split.

### The 10/11 integration patch

`docs/integration-10-11.patch` (sha256 `4581fb056574…`, 1,231 lines, 13
files) touches no chapter source but `mod.rs`. The two new files are
already in place.

Its base is the live tree with the frozen 08/09 patch applied. That base
keeps root's `right_on_average.rs` repair and `docs/pause-overrides.json`,
which neither patch touches. Before → after, as sha256 prefixes:

| File | Base | After |
|---|---|---|
| `docs/brief.md` | `00075755` | `bb69d429` |
| `docs/pauses.tsv` | `b2661ffc` | `25af3179` |
| `docs/protected.tsv` | `cf432842` | `e23d34d0` |
| `docs/storyboard-draft.md` | `d15d9127` | `6babe9f1` |
| `docs/storyboard.md` | `6377e52e` | `fd168dce` |
| `lexicon.json` | `7a94cd48` | `463e4df6` |
| `oracle.py` | `c6a7b65d` | `6cc6e101` |
| `script.tsv` | `4a7d52fc` | `043d7dac` |
| `src/chapters/mod.rs` | `f160d8ec` | `f636ca39` |
| `src/facts.rs` | `aa733889` | `86511400` |
| `src/main.rs` | `3521f5c9` | `50580f53` |
| `src/model.rs` | `171dcb3d` | `ca20b69e` |
| `src/palette.rs` | `5b3e7f6d` | `662d447e` |

The base hashes are the 08/09 table's "after" column, plus `palette.rs`.

- `oracle.py` and `src/facts.rs` go from 229 facts to 271. `G-F10` grows
  from the draft's 11 to 31 and `G-F11` from 2 to 24; `facts.rs`'s count
  test becomes 271.
  - The new facts cover the middle and the recipe, the friends' and the
    strange handful's working, and the share below the largest (2,721,678
    of 13,019,909).
  - They also cover the 111 purple steps, the range 5 to 137, the tallest
    step, and the exact widths:
    - average miss 934,167,456/65,099,545 ≈ 14.35 against 08's
      17,164,752/1,859,987 ≈ 9.23;
    - variance 1,584/5 = 316.8 against 4,752/35 ≈ 135.8;
    - ratio 7/3.
  - For 11: the pairs, the groups, and the chances at 60, 71 and 1,000
    (1/12, 65,018/1,859,987 and 227,563/4,125,145,625,100), each group's
    chance 1/455,126 at all three, and the gaps.

  `GROUPS_ROUND` becomes `pub`, because 11's `checked()` uses it. The
  oracle gains 14 cross-checks (71 in all), 3 of them negative controls:
  - the two widths of doubling differ as shown (14.3 and 17.8);
  - weighted draws (k = 2, largest 4, and k = 3, largest 6): the others'
    chances change with the count and are unequal; fair draws' never do.

  The other new checks:
  - every count to 200 is twice its middle minus one;
  - the purple bars hold every handful once;
  - both widths are larger for doubling, and each average miss squared is
    under its variance;
  - doubling's variance at 71 is (N + 1)(N − 5)/15, 7/3 of the gap
    recipe's;
  - in general it is (k + 2)/3 times the gap recipe's, for k ≤ 5 and
    N ≤ 13;
  - doubling's edges:
    - never below the largest for one box or a pair;
    - below it for some handful of 3 to 6 from every larger count;
    - exactly N at N = k;
  - 5/60 at the edge N = 60;
  - each group's chance is 1/C(59, 4) for every count from 60 to 300;
  - the gaps below 60 total 55 for every group.

  No histogram is built by listing handfuls. The bars come from the
  sum-of-subsets table and the share below the largest from that table
  under each largest number. Listing is used only in the small brute-force
  controls.
- `src/model.rs`: `share_by_largest_and_sum`, which counts the handfuls
  topped by each m by their sum with the same table. The existing
  brute-force test gains a comparison against listing every handful.
- `src/palette.rs`: `doubling()`, the colour table's purple, with its row
  in the palette test.
- `src/chapters/mod.rs`: the two modules, registry rows 10 (0.55) and 11
  (0.6), and the module doc.
- `src/main.rs`: 69 new SPOKEN rows (308 in all), the crate doc, and 10 and
  11 in the prediction test.
  - 10 asks "What does doubling say for this handful?" and is answered by
    "Twenty-eight point six".
  - 11 asks "…how much can the other four tell you about the count?" and is
    answered by "Nothing".
- `docs/storyboard.md` gets 10 (149.2 s estimate, 10 beats) and 11 (142.8
  s, 8 beats), with pictures matched to the source. Both carry the tool's
  advisory for a long chapter; the estimate is not a cap. 10's notes say
  that 10_8's zoom runs inside its first sentence.
- `docs/storyboard-draft.md` now holds 12 to 16.
- `script.tsv` is regenerated with the tool's `script_text`: 100 lines.
- `docs/pauses.tsv`: the draft's rows for 10 and 11 give way to 19
  semantic rows (84 in all).
  - 10: 10_2, 10_3, 10_4, 10_5, 10_6, 10_7, 10_8, 10_9 twice and 10_10;
  - 11: 11_1, 11_2, 11_3, 11_4 twice, 11_6 twice, 11_7 and 11_8.

  The header notes 10_8's zoom. The same 19 rows in root's override shape
  went to root in the handoff mail. Their role is the director's
  punctuation default at each boundary, as in root's entries: eighteen
  `thought` and one `question`.
- `docs/protected.tsv` gets 11 rows, 47 in all. They cover:
  - the recipe earned from the middle;
  - the 28.6 below a box someone holds;
  - the gap recipe depending on the largest alone;
  - one handful in five;
  - the average miss as distance ignoring direction;
  - the standard deviation named apart;
  - every group equally likely for every count;
  - only the largest's chance changing;
  - the term of art, said once after the prediction;
  - sufficiency stated with its condition;
  - the fair-sampling assumption.
- `lexicon.json` and `docs/brief.md` get the new pronunciation risks:
  - the decimals of 10_6 and 10_9;
  - the large numbers of 10 and 11;
  - "sufficient" and "Statisticians" at 11_6.

  The brief's Length, Scope and Protected rows are brought up to date, and
  root's Length sentence is kept.
- Protected visual holds, proposed for root's director map in its own shape
  (`video`, `cue`, `next_cue`, `reason`, and `evidence` naming the chapter
  source and its sha256):
  - `10_9 → 10_10`: read the two widths;
  - `10_10 → 11_1`: read the table, to its reading deadline;
  - `11_4 → 11_5`: read the chance table, to its reading deadline;
  - `11_6 → 11_7`: read card G.

  `11_8`'s end beat has no next cue in `script.tsv`, because 12 is not
  registered. `10_5 → 10_6` and `11_5 → 11_6` are the prediction
  countdowns.

### Checks actually run for 10/11 (2026-10-24, last at 13:13 UTC)

These ran on scratch copies outside the repository:
- `l`: the base, live plus the frozen 08/09 patch;
- `m`: `l` with the two new files and the 10/11 edits.

The live chapter files were compared byte for byte with `m`'s after their
last change.

- `python3 -I -B oracle.py` on `m`: exit 0, 271 facts, 71 cross-checks, 0
  failed, stderr kept.
- The storyboard tool's `check`, with no problems:
  - 01 to 09 as before, 10 149.2 s and 11 142.8 s: 21.0 min, with the
    long-chapter advisory on 06 to 11;
  - the draft, 12 to 16.

  `script_text` equals `script.tsv`.
- `rustfmt --edition 2024 --check --config max_width=120` on all 20 Rust
  files of `m`: pass. Through `main.rs`, which follows every module: exit
  0.
- `protected_check.py` on `m`'s list and script: 47 listed, 0 missing,
  PASS.
- The scratch text-test mirror, extended to 10 and 11, found 0 failures in
  308 SPOKEN rows against the oracle's values, 100 lines, 84 pause rows and
  11 chapters. It checks:
  - number-word coverage and the two predictions;
  - the registry against the storyboard, and the draft continuing at 12;
  - every `wait_for` phrase, once and in order.

  11_4's three row cues are waited for through a loop variable, which the
  mirror does not follow. A separate check found each once in 11_4's line,
  in order.
- A Python mirror of the facts: 55 `G-F10`/`G-F11` keys equal the oracle's,
  with 0 differences. `share_by_largest_and_sum` matched brute force in 6
  small cases.
- A Python mirror of 11's checks:
  - the real picture is accepted, and each of the 19 refusals fails for its
    own reason;
  - the closed form agrees from 60 to 300, with 5/60 at the edge and 0
    below it;
  - the drawn-in-turn control, in exact fractions: weighted draws are
    unequal at the edge and shift with N, and fair draws are uniform. The
    largest denominator is 1.76 × 10¹¹, inside `i128`.
- A layout mirror of 11's frame test: no failures. The table's bottom is at
  −2.26 and card G's top at −2.42.
- The patch round trip:
  - On a fresh copy of the live tree, the frozen 08/09 patch passed
    `--check`, applied, and gave `l` with no differences.
  - With the two new files added, the 10/11 patch passed `--check`,
    applied, and gave `m` with no differences.
  - Re-run from the delivered live bytes, the only difference from `m` is
    the new patch file itself.
- The 19 pause rows in root's override shape, through the narration
  director's own `directions()` (read-only, on `m`'s `script.tsv`),
  together with root's 10 entries and the 08/09 handoff's 12: 41 authored
  boundaries, each an unambiguous phrase. The negative control, "sixty" in
  11_4, which occurs more than once, was refused.
- Scratch heuristics, which are not rustc or Clippy:
  - Unused imports: none in the two new files, `mod.rs`, `palette.rs`,
    `facts.rs`, `model.rs` or `main.rs`.
  - Unused `let` bindings and parameters: none in the two new files,
    `facts.rs` or `model.rs`.
  - Dead items and fields: none in chapter 10. In chapter 11, three flags
    (`stage`, `g`, `words`) are parameters of `triangle`, not fields.
  - Constant-only asserts: none in either file except three that go
    through locals (one in 10's frame test, two in 11's). That is the kind
    root's Clippy has passed in 08's frame test.
- Privacy scan of the patch and both new files (home paths, user names,
  scratch paths, hosts): nothing found.
- Frozen bytes are unchanged:
  - `one_estimate.rs` `b271d6dbd62b…` and `how_far_off.rs` `7a47a777adca…`;
  - `integration-08-09.patch` `75479840c6e7…`;
  - `right_on_average.rs` (root's), `draw.rs`, `friends_line.rs` and
    `glyphs.rs`.

## Chapters 12 and 13 (2026-10-24)

Root's assignment (GERMAN1301-G12): new `12_averaging_the_wobble` and
`13_nothing_does_better` in the new modules `averaging_the_wobble.rs` and
`nothing_does_better.rs`, on the tiny company's same ten pairs, with
`docs/integration-12-13.patch` on live plus 08/09 plus 10/11.

The base, found before the patch was built. Root has already applied the
08/09 patch to the live tree: its `git apply --check` fails there, and the
frozen 10/11 patch applies cleanly. Live also carries three root repairs that
my earlier scratch base did not have:
- card F's storyboard note ("matching the ten-box experiment now shown");
- card F in `how_far_off.rs`, built from `pic.ten`;
- `docs/pause-overrides.json` with the 08/09 entries (22 in all).

So the 12/13 patch is built on live plus the frozen 10/11 patch (root's
receipt, sha256 `4581fb056574…`), and it touches none of those three.

Chapter 12 turns the doubling recipe into the gap recipe on screen:
- 12_1 and 12_2: the ten pairs come back with doubling's estimates, 2, 3,
  4, 5, 4, 5, 6, 6, 7 and 8. They average five, but wobble from 2 to 8.
- 12_3 names the cause, from 11: given the larger number, the smaller
  carries no news about the count, yet doubling listens to it.
- 12_4 sorts the pairs by their larger number. Each row moves whole, its
  squares and its estimate together, so the viewer sees the same ten pairs
  regrouped, not new ones. The count line and the arrows come only later.
- 12_5, the prediction: the biggest group, whose estimates are 5, 6, 7 and
  8. "What's their average?" The countdown shows nothing of it. 12_6
  answers six and a half, the gap recipe's value, then gives the other
  groups' 3.5, 5 and 2 in the order spoken.
- 12_8 carries forward 11's fair-sampling sufficiency. Given the larger
  number, every smaller one is equally likely whatever the count. So the
  group averages are the same in companies of 6 and 8, and the averaged
  recipe needs no count.
- 12_9 and 12_10 weight each group by its true chance, 1, 2, 3 and 4 in
  ten: 0.2 + 0.7 + 1.5 + 2.6 = 5. Equal weights for the four unequal groups
  give 17/4, spoken as the contrast.
- 12_11 to 12_13 show the misses as arrows from the count line at 5:
  - In the group topped by 4 the arrows are −1, 0 and +1, and they cancel.
  - In the group topped by 5 they are 0, 1, 2 and 3, all on one side. The
    average's miss, 3/2, equals the arrows' average length.
  - Over the ten pairs: average miss 1.4 against 1.2 (mean absolute error).
- 12_14 states the rule only after the arrows have shown it. An average's
  arrow is never longer than the arrows' average length: shorter when they
  point both ways, just as long when they all point one way.
- 12_15 keeps the second measure apart. Squared misses average 3 against
  9/4. The standard deviation is ≈ 1.7 against exactly 1.5. The drop, 3/4,
  is exactly the spread inside the groups. 1.4 against 1.2 is never called a
  standard deviation, and 3 against 9/4 is never called an average miss.
- 12_16: card H, the general statement, after the picture.

Chapter 13 earns uniqueness before it states it:
- 13_2: unbiased must hold at every count N ≥ k, because the count is
  unknown. "Always 71" is right on average at 71 and wrong at every other
  count: the counterexample.
- 13_3 to 13_6 build the staircase for pairs, one company at a time, before
  any general argument. Each company line shows its one unknown:
  - 2 boxes force 2;
  - 3 boxes force the new pairs to 3.5;
  - 4 boxes, the prediction, force 5;
  - 5 boxes force 6.5.
- 13_7, the general step. Going up a box, the only new handfuls are those
  containing the new top box, and it is always their largest. Their share,
  k ÷ N, is never zero, so each step has one unknown with a positive
  coefficient. With five seen from exactly 60 boxes the share is 1 in 12,
  as in 11.
- 13_8: so only one recipe of the largest is unbiased at every count, the
  gap recipe.
- 13_9 to 13_11: a rival that uses every number, 3 × smaller − 1. Its
  estimates run from 2 to 11 and average 5.
  - Averaged inside the groups, it is a recipe only because of 11's
    sufficiency (13_10 says so), and it comes back to the staircase 2, 3.5,
    5 and 6.5.
  - It misses by 2.4 against 1.2.
  - So no unbiased recipe misses by less.
- 13_12: best is not "every rival does worse". At N = k every unbiased
  recipe is exactly right, and a recipe of the largest alone does not change
  when it is averaged.
- 13_13 and 13_14: 7/6 × largest − 1.
  - It says 69 for the friends and averages 69 at 71 (its bias is
    −(N + 1)/36).
  - At 71 it misses by less: plain ≈ 8.7 against ≈ 9.2, squared ≈ 132.3
    against ≈ 135.8.
  - At N = 5 it says 29/6 ≈ 4.8 where the gap recipe is exact.
  - Its squared misses are bigger at every count from 5 to 10.

  Then card I, and the condition, "as long as the handful is fair".

The proof, written here and not resting on any enumeration. Fix k, and let
g be a recipe of the largest number M alone.
- For N ≥ k, P_N(M = m) = C(m − 1, k − 1) ÷ C(N, k). So unbiased at N means
  Σ_{m = k..N} C(m − 1, k − 1)·g(m) = N·C(N, k).
- At N = k the only handful is 1 to k, so g(k) = k.
- For N > k, subtract the equation at N − 1: C(N − 1, k − 1)·g(N) =
  N·C(N, k) − (N − 1)·C(N − 1, k). The coefficient C(N − 1, k − 1) is at
  least 1, so g(N) is fixed by the values below it. By induction from N = k
  there is at most one such g.
- With k·C(N, k) = N·C(N − 1, k − 1) and k·C(N − 1, k) = (N − k)·C(N − 1,
  k − 1), this is k·g(N) = N² − (N − 1)(N − k) = (k + 1)N − k. So g(N) =
  N + (N − k) ÷ k, the gap recipe, which 06 and 07 prove unbiased: exactly
  one exists. The new handfuls' share, C(N − 1, k − 1) ÷ C(N, k), is k ÷ N.
- Now let T be any recipe of the whole handful, unbiased at every N ≥ k.
  - Given M = m, a fair draw makes the other k − 1 a uniform choice from
    1 to m − 1 at every N ≥ m (11). So φ(m) = E[T | M = m] does not depend
    on N, and φ is a recipe.
  - E_N[φ(M)] = Σ_m P_N(M = m)·φ(m) = E_N[T] = N, each group weighted by
    its chance as in 12_10. So φ is unbiased at every N, and it is the gap
    recipe.
- For a convex loss L, conditional Jensen gives E[L(T − N) | M] ≥
  L(φ(M) − N). Averaging with the same weights gives E_N[L(T − N)] ≥
  E_N[L(gap − N)] at every N, for L = |·| and L = (·)².
- Equality holds:
  - always at N = k;
  - for |·| at any count where T's misses inside each group are one-sided;
  - for (·)² only where T already equals φ(M).

  Nothing in 13 claims a strict improvement for every recipe, or a biased
  competitor that is better everywhere.

Where it is checked. `oracle.py`'s `recurrence_proof()` and `facts.rs`'s
`staircase_algebra_holds()` check the algebra as polynomial identities,
giving `G-F13.recurrence_proved`:
- the two binomial identities for k = 1 to 12, as polynomials in N;
- the last step in N and k together.

A negative control refuses a wrong step, N² − (N − 1)(N + k). The
enumerations are controls, not the proof:
- the forced staircase for k = 1 to 6 and N ≤ 40 in the Rust test, and for
  k = 1 to 4 and N ≤ 25 in the oracle;
- the staircase off at one value;
- the rivals (k + 1) × smaller − 1 for k = 2 to 5 and N ≤ 13;
- min(largest, 4), which has no new value to choose at five boxes and so
  averages 22/5 there: the step with a zero coefficient.

The biased recipe, checked exactly, which corrected one of my own notes.
`G-F13.shrunk_loses_plain_at` "5,11" lists the counts from 5 to 300 where
its plain miss is not smaller (≥). It is strictly bigger only at 5. At 11
the two tie exactly, 12/11 each. So 13_14 says nothing about plain losses,
and the storyboard note now says so.

The Rust test checks three lists:
- the ≥ list against the fact;
- the strictly-bigger list against [5];
- the squared ≥ list against `G-F13.shrunk_loses_squared_at`, 5 to 10.

The scratch mirror found no squared tie from 5 to 300.

- `src/chapters/averaging_the_wobble.rs` (sha256 `45b065d825f1…`, 1,812
  lines, 7 tests): chapter 12. The table of ten rows, each with its squares,
  estimate, plot and arrow, moves whole through `move_all`.
  - `checked()` refuses five contradicted pictures:
    - doubling's estimates, their average or their range;
    - a group not holding its pairs, not averaging the gap recipe's value,
      or not the same in companies 6 and 8;
    - chances that do not weight the averages back to 5, or an
      equal-weight average that is not the different 17/4;
    - misses of the group topped by 4 that do not cancel, or of the group
      topped by 5 that are not one-sided;
    - widths other than 1.4 and 1.2 plain and 3 and 9/4 squared, or a drop
      that is not the spread inside the groups.
  - Tests:
    - the picture is the facts;
    - a picture the words contradict is refused;
    - the regroup moves each pair whole;
    - averaging keeps the average and never widens, for doubling and for
      3 × smaller − 1 in companies 2 to 10;
    - every waited-for phrase is once in its line, in order;
    - the table, the plots and the panel fit the frame;
    - card H fits and is read.
  - Card H: "fair handful, any recipe: averaged over each largest M, it
    becomes a recipe of M alone, free of N. At every N its average stays,
    and its average miss and squared miss never grow." It is 178
    characters, three lower-third lines at every width, and an 11.9 s
    reading deadline.
- `src/chapters/nothing_does_better.rs` (`dab9ec4db304…`, 1,823 lines, 9
  tests): chapter 13. It reuses 12's `GAP_HEAD`, `AVERAGED_IS`, `linear`,
  `rule_text`, `shown` and `mean`. The grid of pairs has a bar under each
  column for the staircase.
  - `checked()` refuses seven contradicted pictures:
    - an "always" recipe right on average at any count but its own (5 to
      100);
    - companies with the wrong pairs or new pairs;
    - a staircase that does not solve every company line, or is not the gap
      recipe's;
    - a new-handful share at 60 other than 5 ÷ 60, or a step not proved;
    - a rival with the wrong values or average, or that does not average to
      the staircase, or does not miss by more;
    - a biased recipe whose 71-box average is not 69, or that does not miss
      by less there, plain and squared;
    - a biased recipe not beaten at the edge, or whose squared losses over
      5 to 100 (strict) are not the fact's list.
  - Tests:
    - the picture is the facts;
    - a picture the words contradict is refused;
    - each count fixes one value (k = 1 to 6, N ≤ 40);
    - a staircase off at one value is biased from there up, and not below;
    - the rival averages back to the staircase;
    - the biased recipe wins somewhere and loses somewhere (the three lists
      above);
    - every waited-for phrase is once in its line, in order;
    - the grid, the bars, the panel and the table fit the frame;
    - card I fits and is read.
  - Card I: "fair handful: a recipe unbiased at every N ≥ k, averaged over
    each largest M, is the only unbiased recipe of M, M + (M − k) ÷ k. So
    none has a smaller average miss or squared miss at any N." It is 189
    characters, three lines, a 12.6 s deadline, and no split formula.
- Dense statements come only after the concrete picture: card H after 12's
  arrows and table, card I after 13's three recipes. Each holds 2.5 s and
  then until its reading deadline, counted from when it came up. 12_15's
  table also waits for its own deadline, and 13_14 waits for the later of
  card I's and the table's.
- No typeset text is indicated, scaled, recoloured or laid over a copy.
  Every emphasis is a separate halo, and the arrows and the count line are
  short strokes.
- Defects found and fixed before handoff, all by hand:
  - a 13_4 to 13_6 loop with a dead branch, now straight-line beats;
  - an undefined test constant (`TINY_PAIR`);
  - frame-test asserts that could not fail, now real head, label, badge
    and countdown checks;
  - possible `needless_range_loop` sites, now `zip` and `enumerate`;
  - shadowed locals;
  - an unused import;
  - constant-only asserts, now `const { assert!(…) }`;
  - the plain-tie semantics above;
  - an ambiguous storyboard note on the min(largest, 4) control;
  - every rustfmt difference (6 hunks in `main.rs`; both chapter files
    were written to rustfmt's layout).

### The 12/13 integration patch

`docs/integration-12-13.patch` (sha256 `39f983690a99…`, 1,876 lines, 12
files) touches no chapter source but `mod.rs`. The two new files are already
in place. Before → after, as sha256 prefixes; the base is live plus the
frozen 10/11 patch:

| File | Base | After |
|---|---|---|
| `docs/brief.md` | `bb69d429` | `ef4a09c4` |
| `docs/pauses.tsv` | `25af3179` | `9f33b789` |
| `docs/protected.tsv` | `e23d34d0` | `081fe670` |
| `docs/storyboard-draft.md` | `6babe9f1` | `db6efbf1` |
| `docs/storyboard.md` | `aee2ff9a` | `4b0f06c7` |
| `lexicon.json` | `463e4df6` | `3d861368` |
| `oracle.py` | `6cc6e101` | `1b67831d` |
| `script.tsv` | `043d7dac` | `bfb7e708` |
| `src/chapters/mod.rs` | `f636ca39` | `2662f261` |
| `src/facts.rs` | `86511400` | `02eb9844` |
| `src/main.rs` | `50580f53` | `50087c99` |
| `src/model.rs` | `ca20b69e` | `305e2a61` |

`docs/storyboard.md`'s base, `aee2ff9a`, is root's repaired file, not the
10/11 table's `fd168dce`.

- `oracle.py` and `src/facts.rs` go from 271 facts to 338: `G-F12` grows
  from the draft's 11 to 37 and `G-F13` from 9 to 50. The oracle gains 20
  cross-checks (91 in all), 5 of them negative controls:
  - averaging changes nothing for the gap recipe itself, for one box, or at
    N = k;
  - weighted draws (k = 2, largest 4, and k = 3, largest 6): the doubling
    average given the largest changes with the count;
  - the polynomial check refuses a wrong step;
  - min(largest, 4) averages 22/5 at five boxes.

  `facts.rs` gains `staircase_algebra_holds` and 5 tests:
  - the averaging numbers are the storyboard's;
  - averaging inside a group never misses more;
  - the staircase is forced and needs a new value;
  - the staircase algebra check can fail;
  - the 7/6 recipe is biased at every count.
- `src/model.rs`: `Q::decimal2` (exact only) and `Q::exact_sqrt`.
- `src/chapters/mod.rs`: the two modules, registry rows 12 and 13, and the
  module doc.
- `src/main.rs`: 97 new SPOKEN rows (405 in all), the crate doc, and 12
  and 13 in the prediction test.
  - 12 asks "What's their average?" and is answered by "Six and a half".
  - 13 asks "What must the recipe say for the new three?" and is answered
    by "Five".
- `docs/storyboard.md` gets 12 (244.6 s estimate, 16 beats) and 13 (256.6
  s, 14 beats), with pictures matched to the source. Both carry the tool's
  long-chapter advisory. Motivation and reading time were not shortened to
  meet the old draft's ~58 s and ~67 s.
- `docs/storyboard-draft.md` now holds 14 to 16. The old draft 12 and 13
  (seven beats each) are replaced, not edited in place. Their words stay
  readable as the removed lines of this patch, which is kept. No 14 to 16
  row changes.
- `script.tsv` is regenerated with the tool's `script_text`: 130 lines. No
  line of 01 to 11 changes.
- `docs/pauses.tsv`: the draft's 12_5 and 13_5 rows give way to 27 semantic
  rows (109 in all), 13 for 12 and 14 for 13. The header lists 12_15, 12_16
  and 13_14 as scene holds and notes that 12_4's regroup runs inside its
  first sentences. The same 27 rows in root's override shape go to root in
  the handoff mail:
  - roles are the director's punctuation default at each boundary: 16
    `thought`, 6 `clause`, 3 `question` and 2 `cue_transition`;
  - 13_6's "Five." carries `word_index` 0, because 13_6 also says "five
    boxes".
- `docs/protected.tsv` gets 17 rows, 64 in all, 8 for 12 and 9 for 13. They
  cover the regroup keeping pairs whole, 11's sufficiency, the chance
  weights and the equal-weight contrast, the average miss kept apart from
  squared misses, the arrow rule, the drop as the wobble inside the groups,
  every count, "always 71", the never-zero share, uniqueness by the step,
  the averaged rival as a recipe, "none misses by less", the ties, and the
  biased recipe's win and its loss.
- `lexicon.json` and `docs/brief.md` get the new pronunciation risks:
  - close decimals side by side (1.4 and 1.2; 2.4; ≈ 1.7 and exactly 1.5;
    ≈ 8.7 and 9.2);
  - "sixty-nine" two short of "seventy-one";
  - the fractions "four and a quarter", "two and a quarter", "three
    quarters", "seven sixths" and "four and five sixths", where "sixths"
    blurred into "six" changes the recipe;
  - the signed misses "minus one, zero and plus one";
  - the staircase list, where "three and a half, five" must not merge;
  - the groupings "three times the smaller number, minus one" and "seven
    sixths of the largest number, minus one";
  - the terms' new lines.

  Brief cue references that had drifted were corrected: "sufficient" and
  "Statisticians" are at 11_6, and the 10/11 decimals at 10_6 and 10_9. The
  brief's Length (29.4 min for 01 to 13, 3.7 min drafted, about 33.1 min),
  Scope and Protected rows are brought up to date.
- Protected visual holds, proposed for root's director map in its own shape
  (`video`, `cue`, `next_cue`, `reason`, and `evidence` naming the chapter
  source and its sha256):
  - `12_15 → 12_16`: read the widths table and the drop line, 2.5 s, then
    to the table's deadline (≈ 9.0 s from its rise, on my reading of the
    text the code builds);
  - `12_16 → 13_1`: read card H, 2.5 s, then to its 11.9 s deadline;
  - `13_14` (end beat): read card I and the comparison table, 2.5 s, then
    to the later of 12.6 s and the table's ≈ 14.0 s.

  `13_14` has no next cue in `script.tsv`, because 14 is not registered.
  `12_5 → 12_6` and `13_5 → 13_6` are the prediction countdowns.

### Checks actually run for 12/13 (2026-10-24, last at 15:19 UTC)

These ran on scratch copies outside the repository:
- `n`: the 12/13 work on the old base `m`;
- `o`: the true base (live plus the frozen 10/11 patch) with the two new
  files and the 12/13 delta applied.

`o` differs from `n` only by root's three repairs and this file.

- The frozen 10/11 files match root's receipt: the patch `4581fb05…`,
  `a_rival_recipe.rs` `e72331a3…`, `the_largest_says_it_all.rs` `16b1c070…`
  and this file `594de67a…` at intake.
- `python3 -I -B oracle.py` on `o`: exit 0, 338 facts, 91 cross-checks, 0
  failed, stderr kept. Its output equals `n`'s byte for byte. The base's
  oracle gives 271 and 71.
- The storyboard tool's `check`, with no problems:
  - 01 to 11 as before, 12 244.6 s and 13 256.6 s: 29.4 min, with the
    long-chapter advisory on 06 to 13;
  - the draft, 14 to 16: 3.7 min.

  `script_text` equals `script.tsv`.
- `rustfmt --check --config-path` (the repository's `rustfmt.toml`, edition
  2024, width 120) through `o`'s `main.rs`, which follows every module:
  exit 0.
- The scratch text-test mirror on `o`: 405 SPOKEN rows against the
  oracle's values, 130 lines, 109 pause rows, 13 chapters, the draft from
  14 to 16, "all text checks pass". It checks number-word coverage, the
  predictions, the registry against the storyboard, every pause phrase,
  and the protected rows.
- `wait_for` mirrors of both chapters: every phrase once in its line, in
  order, with 0 failures. 12_6's four group answers are waited for through
  an array in spoken order, and 12_6 is checked in that order.
- A Python mirror of 13's `checked()` and four of its tests, independent of
  the Rust (fractions, `math.comb`): 0 failures. At 71 it gives plain
  8.7124 against 9.2284 and squared 132.33 against 135.77; the squared
  losses are 5 to 10 and the plain ≥ list is 5 and 11.
- A layout mirror of 12's frame test and of its averaging test for
  companies 2 to 10: 0 failures (the badge's right edge 6.791 against the
  safe 6.861).
- Card wrap mirror for H and I: three lines at every lower-third width from
  92 to 64. The negative control, one clause more, takes four lines at 64.
  A separate check reads the cards from the source and finds them equal to
  the mirror's.
- The pause rows in root's override shape, through the narration
  director's own `directions()` (read-only, on `n`'s `script.tsv`, which
  equals `o`'s), together with root's 22 live entries and the 10/11
  handoff's 19: 68 authored boundaries. Two negative controls were refused:
  an ambiguous "five" in 12_6, and 13_6's row without its `word_index`.
- Scratch heuristics, which are not rustc or Clippy:
  - Unused imports: none in either new file.
  - Unused `let` bindings: none.
  - Dead items: only function parameters flagged, which are false
    positives.
  - Constant-only asserts: only the card-text asserts (`CARD_H.contains`,
    `CARD_I.starts_with` and `contains`), the kind root's strict Clippy
    passed in chapter 07.
- Privacy scan of the patch and both new files (home paths, user names,
  scratch paths, hosts, e-mail): nothing found. No `todo!`,
  `unimplemented!` or `#[allow`.
- The patch round trip:
  - On a fresh copy of live, the frozen 10/11 patch applied and gave the
    base with no differences.
  - With the two new files added, the 12/13 patch passed `--check`,
    applied, and gave `o` with no differences.
  - Re-run from the delivered live bytes on another fresh copy, the result
    again equals `o`.
- Untouched by the patch: every chapter file of 01 to 11, `draw.rs`,
  `friends_line.rs`, `glyphs.rs`, `palette.rs`, root's
  `docs/pause-overrides.json`, this file, and all five earlier patches.

## Files (all under `explainer_corpus/german_tank/`)

| File | What it is |
|---|---|
| `docs/brief.md` | takeaway, promise, model, three ideas kept apart, counterexamples, sources read and not read, bounded competition check (search results only, no video watched), pronunciation risks, assumptions |
| `docs/storyboard.md` | the production storyboard: the film's arc, vocabulary and color roles, and chapters 01 to 07 (to 09 with the 08/09 patch, to 11 with the 10/11 patch after it, to 13 with the 12/13 patch after that), pictures matched to the scene source |
| `docs/storyboard-draft.md` | chapters 08 to 16 in full (10 to 16 with the 08/09 patch, 12 to 16 with the 10/11 patch, 14 to 16 with the 12/13 patch), same format, words and pictures |
| `script.tsv` | generated from `docs/storyboard.md` by `tools/storyboard.py`: 63 lines for 01 to 07 (82 for 01 to 09, 100 for 01 to 11, 130 for 01 to 13) |
| `docs/pauses.tsv` | 62 authored pause intentions for chapters 01 to 16 (67 with the 08/09 patch, 84 with the 10/11 patch, 109 with the 12/13 patch); editorial hypotheses, not measured |
| `docs/protected.tsv` | 27 active phrases for chapters 01 to 07 (36 for 01 to 09, 47 for 01 to 11, 64 for 01 to 13) |
| `oracle.py` | the independent oracle: 198 facts, 42 cross-checks including negative controls (229 and 57 with the 08/09 patch, 271 and 71 with the 10/11 patch, 338 and 91 with the 12/13 patch) |
| `lexicon.json` | empty respellings (no proper name is voiced) and the recorded risks |
| `Cargo.toml` | package `german_tank`, binary `tank`, workspace-inherited fields |
| `src/model.rs` | exact fractions, the recipes, closed-form distributions, the SplitMix64 draws, tests against brute force and negative controls |
| `src/facts.rs` | all 198 facts by closed forms (229 with the 08/09 patch, 271 with the 10/11 patch, 338 with the 12/13 patch), the CGM formula pin, the staircase algebra as polynomial identities, tests |
| `src/chapters/five_boxes.rs` | chapter 01's native scene and its tests |
| `src/chapters/draw.rs` | the line's one map (`x_at`), box and tick geometry, box clearance and zoom legs, the accepted playback, line-timing and countdown helpers, geometry tests |
| `src/chapters/friends_line.rs` | the line chapters 02 to 04 share (the bin, the pour, rows of handfuls, spacing arrows, gap arcs, the badge), 11 tests; registered by root's application of the 02/03 patch |
| `src/chapters/a_thousand_boxes.rs`, `counting_the_gaps.rs` | chapters 02 and 03's native scenes, 8 and 9 tests, qualified by root (Mail 6877); `counting_the_gaps.rs` carries root's stroke-only dimming repair |
| `src/chapters/what_we_assume.rs`, `every_handful.rs` | chapters 04 and 05's native scenes, 9 and 8 tests, and the halo, replacement and panel builders they share; registered by root's application of the 04/05 patch |
| `src/chapters/gaps_trade_places.rs`, `right_on_average.rs` | chapters 06 and 07's native scenes, 9 and 8 tests; 06 also holds the six-gap line and halo builders 07 reuses. Registered by root's application of the 06/07 patch; `right_on_average.rs` carries root's unused-binding repair |
| `src/chapters/one_estimate.rs`, `how_far_off.rs` | chapters 08 and 09's native scenes, 8 and 9 tests; 08 also holds the bars, count mark and spans 09 reuses. Unregistered until the 08/09 patch is applied |
| `docs/integration-08-09.patch` | the coordinated change that registers 08 and 09, applied after the 06/07 patch; keep it after it is applied |
| `src/chapters/a_rival_recipe.rs`, `the_largest_says_it_all.rs` | chapters 10 and 11's native scenes, 8 and 7 tests. Unregistered until the 10/11 patch is applied |
| `docs/integration-10-11.patch` | the coordinated change that registers 10 and 11, applied after the 08/09 patch; keep it after it is applied |
| `src/chapters/averaging_the_wobble.rs`, `nothing_does_better.rs` | chapters 12 and 13's native scenes, 7 and 9 tests; 12 also holds the gap-recipe head, the averaged line and the rule and number formatters 13 reuses. Unregistered until the 12/13 patch is applied |
| `docs/integration-12-13.patch` | the coordinated change that registers 12 and 13, applied after the 10/11 patch on the live tree (08/09 already applied by root); keep it after it is applied, since its removed lines are the record of the old draft 12 and 13 |
| `docs/integration-06-07.patch` | the coordinated change that registered 06 and 07; applied by root, with root's Length sentence kept in the brief, and kept as the record |
| `docs/integration-02-03.patch` | the coordinated change that registered 02 and 03; applied by root and kept as the record |
| `docs/integration-04-05.patch` | the coordinated change that registered 04 and 05 (facts, oracle, registry, SPOKEN rows, storyboard, draft, script, pauses, protected rows, lexicon and brief); applied by root and kept as the record |
| `src/chapters/mod.rs`, `glyphs.rs` | the registry (01 to 07; to 09 with the 08/09 patch, to 11 with the 10/11 patch, to 13 with the 12/13 patch) and the glyph-probe extras |
| `src/palette.rs`, `src/score.rs` | the color roles the implemented chapters use; the musical identity (D major, unheard) |
| `src/main.rs` | the `Explainer`, and tests: every spoken number against its fact (plus a coverage scan with its own edge test), storyboard/registry/script agreement, the draft continuing the registry, the prediction asked once and answered next, every pause row naming a real boundary |

Chapter 01's scene binds the whole line (84 dots, the line past them, the five
small boxes and their numbers, the ticks, the certain bracket and the dashed
tail) to three trackers through `x_at`; one updater reshapes those existing
mobjects only on frames where a tracker changed, so the zoom to a thousand is
the same picture moving, not a redraw every frame (the kit's engine rule).

## Checks actually run (2026-10-24, about 05:40 to 05:55 UTC)

- `python3 -I tools/storyboard.py german_tank`: exit 0; 01_five_boxes 74.8 s,
  9 beats; wrote `script.tsv` (9 lines). `--check`: exit 0 (not stale).
- The draft, with the storyboard tool's own functions (command in the
  draft's header): exit 0; 15 chapters, 17.2 min, every chapter at most 85 s.
- `lint_script.py` (the skill's): `script.tsv` 9 lines, 0 errors, 0 warnings,
  PASS; the draft's generated script 112 lines, 0 errors, 0 warnings, PASS.
- `protected_check.py docs/protected.tsv script.tsv`: 3 listed, 0 missing,
  PASS (run directly, not through `./x`, so no gate mark was written).
- `python3 -I -B oracle.py`: exit 0, 151 facts, 26 cross-checks, 0 failed,
  about 8 s; stderr kept, not silenced.
- A Python mirror of the new Rust tests' data checks (reading the tables
  from the Rust source text; kept outside the repository): 15 spoken rows, 3
  vague phrases, 22 scene waits, 53 pause rows, 0 failed. Its negative
  control, the oracle output with `G-F1.at_least` changed to 61 and one
  number changed: 6 failures, exit 1, as expected.
- `rustfmt --edition 2024 --check src/main.rs` (read-only, follows the
  modules): no syntax errors; formatting differences, which `rustfmt` then
  applied to these files only; the re-check passed.
- Scans: no `todo!`, `unimplemented!` or `unreachable!()`; no absolute home
  paths, user names or scratch paths in this directory.

### After the source corrections (2026-10-24)

Only the checks the label, fact and note changes affect were re-run:

- `python3 -I -B oracle.py`: exit 0, 152 facts, 26 cross-checks, 0 failed;
  `G-F1.first` 1, `G-F1.second` 2, `G-F1.zoom_marks` 1,250,500,750. Stderr
  was kept, not silenced.
- The G-F1 keys that `src/facts.rs` adds (read from its source text) and the
  oracle's G-F1 keys are the same 8.
- The Python mirror, extended for this defect: no digit-only string reaches
  `kit.text` in the scene's `construct`, no `MARKS` table, the labels and
  marks read from `Picture`, and the oracle's `G-F1.zoom_marks` equals the
  first number plus the quarters of `G-F1.thousand`. On the current source:
  0 failed, exit 0. Negative control, the same mirror on the pre-change
  `five_boxes.rs`: 4 failures (digit literals '1' and '2', a `MARKS` table,
  labels not from `Picture`, marks not from `Picture`), exit 1, as expected.
- `tools/storyboard.py --check german_tank`: exit 0 (`script.tsv` not stale;
  01_five_boxes 74.8 s, 9 beats). The draft through the tool's functions:
  exit 0, no problems, 15 chapters, 17.2 min.
- `rustfmt --edition 2024 --check src/main.rs`: exit 0.
- The placeholder and privacy scans again: nothing found.

## Not run, and so not known

- This lane has run no `cargo`, `rustc`, render or TTS command. Root's runs
  are the native evidence. In Mail 6837 root reported, for the bytes frozen
  in Mail 6829: 35 release tests, fmt, strict Clippy, release build, and
  `tank facts` against the oracle at 152/152 all pass. The glyph probe gave
  21 sources and 0 failures. There was a 1080p15 scratch preview with an
  empty layout lint. Root's repair04 run passed for the bytes before the
  tick anchor. For the tick-anchor bytes and chapters 02 and 03 together,
  root reported 68 release tests, strict Clippy, build and 161 facts
  passing (Mail 6877). Chapters 04 and 05 are uncompiled (below).
- All 23 of chapter 01's mid-line moments (18 `wait_for` phrases and the 5
  number flips of 01_4) go through `Spoken::at`, so each is a
  character-share estimate. Chapters 02 and 03's `wait_for` moments are the
  same kind. Nothing has been aligned to real narration.
- Root has rendered chapters 02 and 03 silently and viewed frames of them
  (Mail 6877). This lane has seen none of those frames.
- This lane has viewed only the crops it took from root's preview around
  01_5 (40.6 to 42.333 s), no continuous playback. Root's sampled frames are
  root's evidence. No narration units, synthesis, TTS, music or listening.
  Whether the repaired moves read clearly is unknown until root renders
  them. Whether the pacing feels right and whether any voice sounds right
  are also unknown: written plans and passing arithmetic prove none of
  them, and no claim about English-accent consistency is made.
- The all-state checks cover box against box only. They do not cover a
  sliding box against the text or strokes it passes, which only frames can
  show.
- `narration_units.json` does not exist; the resistor crate's test for it is
  not copied until root builds the units.
- Beat 16_1 (the one historical sentence) rests on the publisher's abstract
  that root found, not on the paper, which nobody here has read.
- Chapters 04 and 05: no `cargo`, `rustc`, Clippy, render or TTS command
  has run on them.
  - Their 17 tests, and the changed facts and main tests (176 facts, 105
    SPOKEN rows), have never run; 85 tests are expected after the patch.
  - The scratch checks above are not compiler evidence. A wrong fmn or kit
    signature, a borrow or type error, or a lint the heuristics miss would
    surface only in root's build.
  - No frame of 04 or 05 exists. All layout checks are arithmetic on
    constants and on the kit's width estimate (`est_width`), not on real
    glyph bounds. That includes halo clearance, the panel lines beside the
    badge, the six ring rows over the line, the table beside its plot, the
    two balances under the right-hand lines, and the live count's width at
    13,019,909.
  - The halo is built from each object's bounding box at run time. Its test
    checks the frame function on boxes, not on rendered text.
  - The live count in 05_1 shows 0 from the line's start until "over
    thirteen million". Whether that reads as a counter about to run, or as
    a wrong number, is a question for the frames.
  - Every mid-line moment in 04 and 05 is a `wait_for` character-share
    estimate, as in 01 to 03. The 2.5 s holds after 04_5 and 05_2 and the
    countdowns are scene time, not measured reading time.
  - New glyphs for the probe: "−" (U+2212), which no earlier chapter
    typesets, appears in 04 and 05's panel lines and formulas, along with
    "÷", "=", "+", ":", "(", ")", "?", the thousands commas of the live
    count and the decimal point of 3.5 and 6.5.
  - No narration exists, and nothing was synthesized or heard. The
    pronunciation risks in `docs/brief.md` and `lexicon.json` are written,
    not heard.
- Chapters 06 and 07: no `cargo`, `rustc`, Clippy, render or TTS command
  has run on them.
  - Their 17 tests, and the changed facts and main tests (198 facts, 174
    SPOKEN rows), have never run. 102 tests are expected after both
    patches.
  - The scratch checks are not compiler evidence. A wrong fmn or kit
    signature, a borrow or type error, or a lint the heuristics miss would
    surface only in root's build.
  - No frame of 06 or 07 exists. Every layout check is arithmetic on
    constants and on `est_width`, not on real glyph bounds.
  - Every mid-line moment is a `wait_for` character-share estimate. Every
    hold is scene time, not measured reading time.
- Timing risks in 06 and 07 that only narration and frames can settle:
  - Length: the estimates are 136.4 s (06) and 166.0 s (07) before
    authored pauses and holds. Each has five or more 2.5 s holds. No
    runtime cap was set, but the inspection time is a hypothesis.
  - 06_6's partner sweep is six halo cycles of 0.85 s, 5.1 s in all, from
    "Every row has a partner".
    - At the storyboard's 2.68 words a second, "So the below column" comes
      about 7.2 s in, so the sweep fits.
    - The two own-partner halos (3.4 to 5.1 s) come during "a row with
      equal gaps below and above", just before "is its own partner" (about
      5.7 s).
    - 0.35 s per pair is a glance, not an inspection. The worked trade
      (06_5) and the sorted columns carry the inspection.
    - `PARTNER_HOLD_S` is the knob; at about 0.65 s the sweep reaches the
      next phrase.
  - 06_3's read-back: five 0.4 s replacements on five phrases. "and stop"
    sits inside "take the next, and stop", so the last two steps come about
    a second apart. A run-together reading would leave the squares behind
    the words.
  - 06_9 fades everything and then rebuilds the friends' line. The same
    numbers, positions and colours come back as new objects, about 1.4 s
    later, over a moment of empty frame. It is meant to read as "back to the
    friends' boxes", but it may read as a cut.
  - 07_10: card D comes in on "it's right on average", about 4.9 s before
    the line ends at the storyboard's rate. It stays until 12.1 s after it
    came in, so about 7 s of held picture with no voice ends the chapter.
    Bringing it in at "So the recipe always adds" would overlap more of the
    reading with the voice; that is root's choice.
- Frame risks in 06 and 07:
  - 07's inset (07_5 to 07_6) is the tightest layout:
    - Its columns are 0.47 apart, and the widest values ("1,2", "1.5") are
      about 0.325 wide on `est_width` at size 20. That leaves a 0.145 gap
      between neighbours, 0.045 over the test's bound.
    - Its top row sits 0.12 under the panel's fourth line, on assumed
      half-heights of 0.12 and 0.16.

    Real glyph bounds may close either gap.
  - 07_4's guess is pieced with the kit's sum spacing (0.14), so "( 13" and
    "8 )" may look loose.
  - 07_9's guess arc ends where the green tick at 73 stands. They are meant
    to meet, and a frame should confirm it reads as a landing, not an
    overlap.
  - 06's table constants duplicate 05's private ones. If 05's table moves,
    06 must move with it. Root could make 05's constants `pub(crate)`,
    which would also keep the "every pub item is used" rule.
  - Glyphs: nothing beyond 04/05's set ("−", "÷", "=", "+", ":", "(",
    ")", "?", ","), plus the letters N and k on screen. ".5" and "1.5" are
    in 07's inset.
- Chapters 08 and 09: no `cargo`, `rustc`, Clippy, render or TTS command
  has run on them.
  - Their 17 tests, and the changed facts and main tests (229 facts, 239
    SPOKEN rows), have never run. 119 tests are expected after the patch
    (102 + 17).
  - The scratch checks are not compiler evidence. A wrong fmn or kit
    signature, a borrow or type error, or a lint the heuristics miss would
    surface only in root's build. Root's 07 repair shows that kind of miss.
  - No frame of 08 or 09 exists. Every layout check is arithmetic on
    constants and on `est_width`, not on real glyph bounds.
  - Every mid-line moment is a `wait_for` character-share estimate. Every
    hold is scene time, not measured reading time.
  - `Q::sqrt_about1` uses `u128::isqrt`, stable since Rust 1.84.
- Timing risks in 08 and 09 that only narration and frames can settle:
  - Length: the estimates are 135.7 s (08) and 142.3 s (09) before authored
    pauses and holds. Each has three 2.5 s holds and a card's reading wait.
    No runtime cap was set, but the inspection time is a hypothesis.
  - 08_2's twenty rows take 21 steps of 0.3 s, 6.3 s from "Here are twenty
    such handfuls", so about 3 s of rows land after the line. 09_5's take
    20 × 0.25 s, 5 s from "each build their own range", so about 2 s land
    after it. Both are the proposed protected visual holds, not pause rows.
  - 09_1's slide (a 0.4 s refill, then legs of 0.7 s and 2.0 s) starts at
    "How large could it be" and runs through "Flip the question around." At
    the storyboard's rate it ends before "For each possible count", where
    the bars begin. If the narration is quicker, the bars come late.
- Frame risks in 08 and 09:
  - The bars at 107 (5.1%) and 108 (4.9%) differ by about 0.009 in height,
    which no eye can see. The dashed line, the two panel lines and the
    faint bars from 108 carry the cutoff; the storyboard notes say so.
  - In 09's twenty rows, same-number rings in adjacent rows are 0.11 apart
    with radius 0.038. That leaves 0.034 between edges before stroke width.
  - Cards E and F fill three lower-third lines. Each fits at every lower-
    third width on `est_width`; real glyph widths may differ.
  - Glyphs: "√" (U+221A) is new for this topic, in 08's definition and card
    E. "÷", "≈", "−" and "%" appeared before. The probe should confirm "√"
    in the font in use.
- Chapters 10 and 11: no `cargo`, `rustc`, Clippy, render or TTS command
  has run on them.
  - Their 15 tests, and the changed facts, model, palette and main tests
    (271 facts, 308 SPOKEN rows), have never run. 134 test functions are
    expected after both patches (119 + 15).
  - The scratch checks are not compiler evidence. A wrong fmn or kit
    signature, a borrow or type error, or a lint the heuristics miss would
    surface only in root's build. The Python mirrors of 11's checks and
    layout are copies of the logic, not the Rust.
  - No frame of 10 or 11 exists. Every layout check is arithmetic on
    constants and on `est_width`, not on real glyph bounds.
  - Every mid-line moment is a `wait_for` character-share estimate. Every
    hold and reading deadline is scene time from the kit's 15 characters a
    second, not measured reading time.
- Timing risks in 10 and 11 that only narration and frames can settle:
  - Length: the estimates are 149.2 s (10) and 142.8 s (11) before authored
    pauses and holds. No runtime cap was set, but the inspection time is a
    hypothesis. The waits after each line:
    - 10: two 2.5 s holds, and the table's reading deadline;
    - 11: two 2.5 s holds, the chance table's deadline (8.1 s) and card
      G's (12.4 s).
  - 10_8, the first sentence: from "put both recipes side by side", a 0.5 s
    fade, the 0.4 s evening-out and the 2.0 s zoom take 2.9 s. The words
    before "exactly seventy-one boxes" take about 2.6 to 3.0 s (15
    characters or 2.68 words a second).
    - "count = 71" can come a few tenths late.
    - The storyboard's 2.4 s counts the evening-out and the zoom only.
    - The cue is a draft cue until the line is measured.
  - 10_8, the outline: its 1.6 s rise starts at "counted by total". About
    1.8 s of words come before "from five all the way to", where the ends
    "5" and "137" come in.
  - 10_9's "Ignoring direction" and "The standard deviation" bring the
    table's rows in with the words. The two widths are on screen together
    only from the second phrase, through the hold.
  - 11_4's rows each come in 0.4 s from their count's name ("With exactly
    sixty boxes", "with seventy-one", "with a thousand"). A quick reading
    could put the voice ahead of the rows.
- Frame risks in 10 and 11:
  - 10's purple outline is stroke-only over filled green bars at the 140
    scale. Whether the two read apart is a question for the frames.
  - 11's three triangles use dots of radius 0.045 on a 0.22 grid, with
    three caption lines under each. Only the frame test and its layout
    mirror check their spacing, on `est_width`.
  - 11's table bottom (−2.26) is 0.16 above card G's top (−2.42), and both
    are on screen in 11_6.
  - Card G fills three lower-third lines. It fits at every width on
    `est_width`; real glyph widths may differ.
  - Glyphs: "≥" (U+2265) is new for this topic, in card G. "×", "÷", "−"
    and "≈" appeared before. The probe should confirm "≥" in the font in
    use.
- Chapters 12 and 13: no `cargo`, `rustc`, Clippy, render or TTS command
  has run on them.
  - Their 16 tests, and the changed facts and main tests (338 facts, 405
    SPOKEN rows, 5 new `facts.rs` tests), have never run. 155 test
    functions are expected after the 10/11 and 12/13 patches (134 + 21).
  - The scratch checks are not compiler evidence. A wrong fmn or kit
    signature, a borrow or type error, or a lint the heuristics miss would
    surface only in root's build. The Python mirrors of 12's layout and 13's
    checks are copies of the logic, not the Rust.
  - No frame of 12 or 13 exists. Every layout check is arithmetic on
    constants and on `est_width`, not on real glyph bounds.
  - Every mid-line moment is a `wait_for` character-share estimate. Every
    hold and reading deadline is scene time from the kit's 15 characters a
    second, not measured reading time. The two table deadlines (≈ 9.0 s and
    ≈ 14.0 s) are my reading of the text the code builds, not a run.
  - Nothing was narrated, synthesized or heard. The pronunciation risks are
    written, not heard.
- Timing risks in 12 and 13 that only narration and frames can settle:
  - Length: the estimates are 244.6 s (12) and 256.6 s (13) before authored
    pauses and holds. No runtime cap was set, and the assignment says not to
    shorten motivation or reading time; the inspection time is still a
    hypothesis. The waits after each line:
    - 12: the countdown, 12_15's 2.5 s hold and table deadline, 12_16's
      2.5 s hold and card H's 11.9 s;
    - 13: the countdown, and 13_14's 2.5 s hold, then the later of card I's
      12.6 s and the table's ≈ 14.0 s.
  - 12_4's regroup, a 0.4 s fade and a 1.6 s move from "So sort the pairs",
    runs inside the first sentences. The group labels wait for "That makes
    four groups", about 9 s of words later at the storyboard's rate.
  - 12_11's count line (0.4 s) and the eight purple arrows (a 1.2 s lagged
    fade) start at "an arrow from five". "the group topped by four" comes
    about 1.5 s of words later, so its halo can come a few tenths after the
    voice.
  - 13_8's four halos in turn take 4.4 s from "Each count fixes the next
    value". The formula waits for "it's the gap recipe", about 10 s of words
    later.
  - 13_9's rival line and its ten values take 2.0 s from "three times the
    smaller number, minus one". The halos for "from two all the way to
    eleven" may come while the last values land.
- Frame risks in 12 and 13:
  - 12's table is the densest frame of the film so far. Each of the ten rows
    has its squares, estimate, plot, and later an arrow or two, with the
    group labels, the right-hand chance lines and the panel beside it.
    Only the frame test and its layout mirror check the spacing, on
    `est_width`.
  - 12's arrows sit 0.09 off each plot's line, purple (doubling) above and
    green (the gap recipe) below. Whether the two rows of arrows
    read apart at the plot's 0.28 per box is a question for the frames.
  - 13's comparison table runs across the frame, with its losses line at
    −0.8 above card I in 13_14.
  - Cards H and I fill three lower-third lines. Each fits at every width on
    `est_width`; real glyph widths may differ.
  - Glyphs: nothing new. "−", "×", "÷", "≈" and "≥" have all appeared
    before ("≥" in card G).

## For root

- Integration: remove this directory from the root manifest's `exclude`,
  then build, test, `tank facts` against the oracle, glyph probe and lint.
- In the later narrated version (the `.4` requirement): bind 01_4's number
  flips and the other reveals to the qualified FrankenWhisper word alignment
  in place of `Spoken::at`'s character share before claiming synchrony.
- `src/.ee/hook-state/session_start_orient.last.json` appeared at 05:32 UTC,
  written by a session-start hook while a shell was in `src/`; it is
  git-ignored (`**/.ee/`) and was left untouched.
- Chapters 02 and 03: the three source files are complete and frozen at the
  hashes above. `docs/integration-02-03.patch` registers them when root
  chooses; `git apply --check` passes on the current tree. After applying
  it, the native checks to run are 68 tests, fmt, strict Clippy, build,
  facts 161/161, the glyph probe (new strings include ≈, ÷ and %), and
  frames of 02_4 and 02_8 (zooms with rows and arrows), 03_4 (the slide into
  the sum) and 03_6 (the inset beside the panel). Root has since done this
  (Mail 6877).
- Chapters 04 and 05: the two source files are complete at the hashes
  above, and I will not change them unless root names a defect.
  - Apply `git apply explainer_corpus/german_tank/docs/integration-04-05.patch`
    at the repository root; `--check` passes on the current tree. Keep the
    patch file afterwards.
  - Then run 85 tests, fmt, strict Clippy, build, facts 176/176, and the
    glyph probe ("−" is new).
  - Frames to look at:
    - 04_2: six ring rows over the line, with the badge;
    - 04_3: the recipe picture in the panel;
    - 04_5: three panel lines and the green tick;
    - 04_7: the edge handful's lines;
    - 05_1: the live count;
    - 05_2: the table;
    - 05_5: the count line and the dots beside the estimates;
    - 05_6: the two balances and their fulcrum labels;
    - 05_7: the right-hand lines over the upper balance.
- Under Mail 6871's rule (no indicate or recolour on typeset text), the
  frozen and qualified chapters still call `indicate` (`draw::flash_all`
  and `flash_lagged`) on objects that include typeset text. I have not seen
  a duplicated glyph in any of them; root's viewed frames are the evidence
  either way. The sites, by line in the current files:
  - `a_thousand_boxes.rs:588`, the badge;
  - `counting_the_gaps.rs` 618 (the "?"), 626 (the badge), 632 (the dashed
    tail with its "?"), 658 and 705 (box 60 with its label), 707
    (the guess's count), 714 (the claim and the badge) and 725 (the
    recipe);
  - `five_boxes.rs:484`, box 60 with its label.

  I made no change: those files are frozen, and the decision is root's.
  Root has recorded these as review risks, not confirmed defects (reply to
  Mail 6886).
- Chapters 06 and 07: the two source files are complete at the hashes
  above, and I will not change them unless root names a defect.
  - Apply the 04/05 patch first, then `git apply
    explainer_corpus/german_tank/docs/integration-06-07.patch` at the
    repository root. Keep both patch files afterwards.
  - The 06/07 patch does not apply to the current tree alone: it needs the
    04/05 patch's lines.
  - Then run 102 tests, fmt, strict Clippy, build, facts 198/198 and the
    glyph probe.
  - Frames to look at:
    - 06_2: the lists beside the table;
    - 06_3: the read-back squares;
    - 06_5: the trade, and the copy beside row 4,5;
    - 06_6: the partner halos and the sorted columns;
    - 06_8: the sums and averages under the table;
    - 06_10: the dashed bracket;
    - 07_4: the pieced guess;
    - 07_5: the inset under the panel;
    - 07_7: the derivation lines and the definition;
    - 07_8: the one-box line with nothing revealed;
    - 07_9: the arc and the tick at 73;
    - 07_10: card D over the line.
  - The proposed visual-hold rows above, for the director map.
  - Root has since applied the 04/05 and 06/07 patches. The tree had both
    at 11:37 UTC, with root's Length sentence in the brief.
- Chapters 08 and 09: the two source files are complete at the hashes
  above, and I will not change them unless root names a defect.
  - Apply `git apply explainer_corpus/german_tank/docs/integration-08-09.patch`
    at the repository root. `--check` passed on a fresh copy of the tree as
    of 11:38 UTC. Keep the patch file afterwards.
  - The patch's base is the 12 hashes in its table above. If root edits one
    of those files first, the patch needs a rebase. `right_on_average.rs`
    is not among them.
  - Then run 119 tests, fmt, strict Clippy, build, facts 229/229 and the
    glyph probe ("√" is new).
  - Frames to look at:
    - 08_2: the rows landing and the dots stacking;
    - 08_4: the 67 bars, the table and the tallest bar;
    - 08_5: the empty place and "?" at 71;
    - 08_7: the fulcrum and the two sides' words;
    - 08_8 and 08_9: the widths, the definition and card E;
    - 09_1: the slide to 130 and the bars rising;
    - 09_2 and 09_3: the chances under the line, the dashed level and
      "5%" past the line's end;
    - 09_6: the twenty rows, the ink ranges and "largest 38";
    - 09_8: the division's three panel lines;
    - 09_10: the ten-box bars, the range 60 to 79 and card F.
  - The proposed visual-hold rows above, for the director map.
- Chapters 10 and 11: the two source files are complete at the hashes
  above, and I will not change them unless root names a defect.
  - Apply the 08/09 patch first, then `git apply
    explainer_corpus/german_tank/docs/integration-10-11.patch` at the
    repository root. Keep both patch files afterwards.
  - The 10/11 patch does not apply to the current tree alone: it needs the
    08/09 patch's lines. Its base is the 13 hashes in its table above. If
    root edits one of those files first, the patch needs a rebase. Root's
    `right_on_average.rs` and `docs/pause-overrides.json` are not among
    them.
  - Then run 134 tests, fmt, strict Clippy, build, facts 271/271 and the
    glyph probe ("≥" is new).
  - Frames to look at:
    - 10_1 to 10_3: the span and fulcrum under the line, the panel's four
      lines, and the purple marks under the line beside the green tick;
    - 10_5 and 10_6: the rings on 2 to 5, the tick at 28.6 and the two
      halos;
    - 10_8: the zoom to 140, the green bars and the purple outline with its
      ends;
    - 10_9 and 10_10: the fulcrum, both halos and the widths table;
    - 11_2 and 11_3: the three triangles, their gold columns, halos and
      captions;
    - 11_4 to 11_6: the panel line, the chance table and card G under it;
    - 11_7: the arcs, the counts and the two sums on the panel.
  - The 19 pause rows in the director's override shape (in the handoff
    mail), and the proposed visual-hold rows above, for the director map.
- Chapters 12 and 13: the two source files are complete at the hashes
  above, and I will not change them unless root names a defect.
  - The live tree already has the 08/09 patch. Apply the frozen 10/11 patch
    first, then `git apply
    explainer_corpus/german_tank/docs/integration-12-13.patch` at the
    repository root. Keep both patch files afterwards.
  - The 12/13 patch does not apply to the live tree alone: it needs the
    10/11 patch's lines. Its base is the 12 hashes in its table above. If
    root edits one of those files first, the patch needs a rebase. Root's
    `how_far_off.rs` and `docs/pause-overrides.json` are not among them;
    root's card F note in `docs/storyboard.md` is in the base and kept.
  - Then run 155 tests, fmt, strict Clippy, build, facts 338/338 and the
    glyph probe (no new glyph).
  - Frames to look at:
    - 12_1 to 12_3: the ten rows with their squares, purple estimates and
      plots, and the halos round 2 and 8 and the four rows topped by 5;
    - 12_4: the regroup mid-move and settled, with the group labels;
    - 12_5 and 12_6: the countdown under the panel's recipe line, right of
      the group labels, then the green 6.5s and the panel's average line;
    - 12_9 and 12_10: the chance lines on the right and the two sums under
      them;
    - 12_11 to 12_13: the count line, the purple and green arrows, and the
      miss lines beside "topped by 4" and "topped by 5";
    - 12_15 and 12_16: the widths table, the drop line and card H;
    - 13_3 to 13_6: the grid's first columns, the bars, the company lines
      and the countdown;
    - 13_7 and 13_8: the general lines under the company lines, and the
      formula;
    - 13_9 to 13_11: the rival's values beside the dots, the averaging lines
      and the misses;
    - 13_13 and 13_14: the comparison table, the losses line and card I.
  - The 27 pause rows in the director's override shape (in the handoff
    mail, 13_6 with `word_index` 0), and the proposed visual-hold rows
    above, for the director map.
- The hook's `.ee/hook-state` directories also sit at the topic root (from
  02:07 local), in `src/` and in `src/chapters/` (from 04:23 local, 08:23
  UTC). They are git-ignored and were left untouched. The patch and the
  scratch copies exclude them.
