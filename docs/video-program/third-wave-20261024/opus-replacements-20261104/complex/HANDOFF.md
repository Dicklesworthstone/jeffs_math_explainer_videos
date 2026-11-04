# complex_analysis_visual: Opus replacement candidate, handoff

BronzeJaguar · bead `mx-video-program-program-87do.39` (root-owned) ·
increment 06, 2026-11-04. Everything here is a staged candidate:
**uncompiled, unrendered, unnarrated**. No Cargo, render, TTS, ASR, commit or
Beads edit was run by me. Earlier texts are kept: handoff 01 in
`previous/handoff01-mail750/HANDOFF.md`, later ones beside their crate mirrors
in `previous/` (`increment02-ch09-10-12-14/`, `increment03-ch02-03-05-08-15/`,
`increment04-ch01-07-11-13/`, `increment05-typed-numbers/`,
`increment06-desk-compile/`). **Build from increment 06:** increments 03, 04
and 05 contain the compile error below.

**Narrator: ElevenLabs Oliver** (owner instruction; Mails 595, 611, 698, 726,
753, 831). Root synthesizes serially through the guarded ElevenLabs lane; I
launched no synthesis, made no takes and substituted no voice.

## State

**All 15 chapters are native FrankenManim scenes** (Rust, `fmn` + the kit);
no chapter plays as an animatic any more. The 105 cue texts are unchanged
since handoff 01 (`script.tsv` sha256 prefix `2968b0fd7654`), so every cue
is still new against canonical and nothing has to be re-synthesized because
of this increment.

| chapter | file | since |
|---|---|---|
| 01 the bump | `src/chapters/the_bump.rs` | handoff 01; strings moved in increment 04 |
| 02 turning numbers | `src/chapters/turning.rs` | increment 03 |
| 03 two planes | `src/chapters/two_planes.rs` | increment 03 |
| 04 reading colour | `src/chapters/reading_color.rs` | handoff 01 |
| 05 gallery | `src/chapters/gallery.rs` | increment 03 |
| 06 one derivative | `src/chapters/one_derivative.rs` | handoff 01 |
| 07 one point | `src/chapters/one_point.rs` | handoff 01; 07_5 rewritten in increment 04 |
| 08 loop sums | `src/chapters/loop_sums.rs` | increment 03 |
| 09 boundary | `src/chapters/boundary.rs` | increment 02 (Mail 877) |
| 10 rigidity | `src/chapters/rigidity.rs` | increment 02 |
| 11 counting | `src/chapters/counting.rs` | handoff 01; 11_8 rewritten in increment 04 |
| 12 residues | `src/chapters/residues.rs` | increment 02 |
| 13 real integral | `src/chapters/real_integral.rs` | handoff 01; 13_3 and 13_10 rewritten in increment 04 |
| 14 continuation | `src/chapters/continuing.rs` | increment 02 |
| 15 closing | `src/chapters/closing.rs` | increment 03 |

Increment 05 changes no picture and no words on screen: every caption reads
as it did, but its number now comes from a fact. See "What increment 05
routed" below.

Shared files changed in increment 04:

- `src/model.rs`: `arc_max`, `arc_size`, `ARC_RADII`, `QUARTIC_ARC_RADII`,
  `NUMERIC_REACH`, `SQUEEZE_RADII`, `COARSE_POWER`, `COARSE_STEPS`,
  `BUMP_READ_AT`. `arc_bound` is now literally πR × `arc_max`.
- `src/facts.rs` and `oracle.py`: 14 new keys, 107 facts in all, same order on
  both sides: `bump.read_at`, `onepoint.squeeze.radii`, `argument.coarse.*`
  (8 keys), `integral.bump.radii`, `integral.quartic.radii`,
  `integral.quartic.under`, `integral.quartic.numeric_reach`.
- `src/chapters/mod.rs`: TEXT/TEX supplements for the new strings; `≥ <` added
  to the glyph row. `closing::CENTRE` removed: 15 now uses `the_bump::AT_X`.
- `src/chapters/draw.rs`: `rgb` is `pub`.
- `docs/storyboard.md`: the picture cells of 07_5, 11_8, 13_3 and 13_10
  describe the new pictures. The words column is untouched.

Shared files changed in increment 03:

- `src/chapters/mod.rs`: registry all native; the `Animatic` import removed;
  TEXT/TEX supplements for 02, 03, 05, 08 and 15.
- `src/main.rs`: `STORYBOARD` is `#[cfg(test)]` (only tests read it now, as
  in bayesian_search and discrete_tomography); doc comments.
- `src/chapters/draw.rs`: `dashed(stage, a, b, color)` helper.
- `src/model.rs`: `TURN_POINT`, `TURN_ADDED`, `TURN_BY` (02's numbers, used
  by facts.rs and turning.rs) and `RECENTRE_AT` (15).
- `src/facts.rs` and `oracle.py`: two new keys, `recentre3.centre` ("3") and
  `recentre3.radius_sq` ("10"), so 15 prints no typed number; 93 facts.
- `docs/storyboard.md`: 08_5's picture cell now says the loop is a square
  (words untouched; see the defects below).

Unchanged canonical copies: `src/score.rs`, `src/chapters/thumbnail.rs`,
`Cargo.toml` (`cmp` verified). The canonical chapter files that are no longer
declared (`argument.rs`, `bookends.rs`, `cauchy.rs`, `continuation.rs`,
`derivative.rs`, `domain_color.rs`, `integral.rs`) should be moved aside at
integration, never deleted. **Exclude `explainer_corpus/complex_analysis_visual/.ee/`**
(session-start hook state, not a deliverable).

## What increment 06 repaired (desk compile)

With no compiler available to me, I wrote text-only checks of the things rustc
would reject and ran them over the source. They found two defects, both now
fixed:

1. **A compile error, `loop_sums.rs:286`:** `kit.tex` had 5 arguments, missing
   the `&[]`. It was present since chapter 08 arrived in increment 03.
2. **A render defect, `real_integral.rs:256`, 13_2:** the dimmed lower pole used
   `set_opacity` on `pole_mark`'s stroke-only ring. `set_opacity` also writes
   fill alpha (`fmn-library/src/style.rs`), so by my reading the ring would have
   rendered as a tinted disk; no frame has confirmed this. The ring now dims
   through `set_stroke`. The file's other two `set_opacity` calls dim the bump's
   filled area and TeX, and stay as they were.

The checks are in `checks/`, each with its usage in its docstring:

| script | what it looks for | result |
|---|---|---|
| `arity_check.py` | argument counts for 3,475 calls to crate, kit and `stage` functions; `?` on infallible calls; dropped `Result`s. A second pass matches 3,493 other calls by name alone, and a third lists called names defined nowhere | 0 problems. One by-name hit (`readout`, a `fn` parameter in `loop_sums.rs`) and a third-pass list I read: only std methods, attributes and one local `f` |
| `scope_check.py` | names used in a file, or its test module, that are not in scope | 25 lines, all of which I read and judged false positives: attributes, `model.rs`'s own enum variants and constructors, a local `f` holding a function pointer |
| `shape_check.py` | `{}` count against arguments in 148 format macros; declared length of 81 const arrays; 7,265 arguments against parameter types (integer for `f64`, missing or extra `&`, and so on) | 0 |
| `unused_check.py` | 2,253 `let` bindings for unused names and needless `mut` | 0 |
| `desk_check.py` | unused imports; items only their own definition mentions | 0 |
| `supplements.py` | the glyph-supplement resolver, previously scratch | 0 |

To see whether a clean result means anything, I planted 9 defects in a scratch
copy (one each of: wrong arity, missing import, wrong array length, wrong
format holes, integer for `f64`, missing `&`, unused `let`, needless `mut`,
unused import). All 9 were flagged.

**This is not a compile.** These checks see text, not types. They cannot find a
wrong type behind a variable, a borrow-checker error, a trait that isn't in
scope, a wrong field name, or a closure that won't coerce. Expect rustc to
find more. No fact value and no cue changed.

## What increment 05 routed

Each number below used to be typed into a string or a literal. It now comes
from a `model.rs` constant, through `facts.rs`, and `oracle.py` restates it in
the same key order. 13 new keys, 120 facts in all.

| chapter | on screen | key | model constant |
|---|---|---|---|
| 03 | the walk's final "1" (was a runtime `format!`) | `identity.circle.turns` | none: certified count |
| 05 | "strip squashed: 0 ≤ Im s ≤ 32", and the strip's top | `gallery.zeta.strip_top` | `ZETA_STRIP_TOP` |
| 07_2 | the pair of "90°" | `critical.angle_kept` | `CONFORMAL_POINT` (`one_point::Z1` is now this) |
| 09 | "average of 8: ", "average of 16: " | `cauchy.samples` | `AVERAGE_SAMPLES` |
| 09 | "limit: 0", and the tally's "= 0" | `cauchy.r_two.limit` | none: f(0) + 2 × the pole's share |
| 09 | the value plane's "1" tick | `bump.peak` (existing) | none |
| 09 | "60 terms at 2: ", "60 terms at 2.6: " | `recentre1.check.terms`, `recentre1.check.at` | `RECENTRE_CHECK_TERMS`, `RECENTRE_CHECK_AT` |
| 14 | "z = 1/2: 11 terms give ", "z = 1.2: 11 terms give " | `series.terms`, `series.half.at`, `series.one_two.at` | `SERIES_TOP`, `SERIES_INSIDE`, `SERIES_OUTSIDE` |
| 14 | the level line at 2 and its "2" label | `series.half.function` | as above |
| 14 | "drawn: 12 terms, inside \|z\| < 0.98" | `lacunary.terms`, `lacunary.limit` | existing `LACUNARY_*` |

- `identity.circle.turns`, `critical.angle_kept`, `cauchy.r_two.limit` and
  `series.half.function` are computed on both sides. The other nine new keys
  print chosen parameters, so the oracle restates them; that catches a silent
  change, nothing more.
- Three new oracle cross-checks: the strip holds all four zeros, z₁ is not the
  critical point, and the 16-point average at radius 2 is near the limit.
- The samples, terms and points in the drawing code use the same constants:
  09's `sample_round` calls and its coefficient `n`, the recentred curve, 14's
  bars and `bars_frame`, 05's strip.
- `model::TAN_FAR` also replaces the literals `facts.rs` used for 05's tangent.
- `rigidity::SIN_3I` replaces an inline caption string, so the supplement
  list now sees it.
- `chapters/mod.rs`: supplements follow the new constants (236 / 130 / 6).
  `boundary::COLON` is shared with 14.

**Dead code removed.** All chapters are in, so `draw.rs` no longer carries its
`#![allow(dead_code)]`, whose stated reason had expired. Gone with it: `swap`,
`tween_all`, the never-read `Painter.plane`, and `line_style` from the import
in `residues.rs`. They were found by a text scan, not by the compiler, so
rustc may still report something it missed; if so, it will now show up rather
than be hidden. This is the first thing to look at if a dead-code warning
appears in `draw.rs`.

**TeX audit.** I read every `*_TEX` constant that holds a digit.

- *Now bound by a test:* 14's `\frac{1}{1 - 1.2}` against `series.one_two.at`;
  05's `\tan(0.4 ± 12i)` against `TAN_FAR`; 06's `z_0 = 1 + i` and
  `z_0^2 = 2i` against `Z0`; 11's `|z^5 + 1| \le 2 < 3 = |-3z|` by sampling the
  unit circle (2 is reached at z = 1).
- *Already bound:* 11's `COARSE_TEX` and `LIP_TEX`; 05's Γ(4) and Γ(5) cards
  (facts print 6 and 24); 13's residue sum; 12's "closes: 0 / 2πi" totals.
- *Worked by hand, no test:* 13's `LOOP_TEX` and `VALUE_TEX`; the remaining
  formulas are symbolic statements.
- *Still typed, deliberately:* see "Remaining gaps".

## What increment 04 shows (Mail 757's three mechanisms)

- **13_3, the arc bound as length × max.**
  - A point walks the half circle on its spoke R with `|z²| = R²`. The bump's
    actual size along the arc is traced in a window beside it, the arc
    unrolled. It stays under the line 1/(R² − 1) and touches it at the top,
    nearest the pole.
  - The arc lights with the unrolled base, labelled πR.
  - The rectangle on that base is length × max, shown with
    `|∫_arc| ≤ πR · 1/(R² − 1) = πR/(R² − 1)`.
  - R goes 2 → 5 → 10 on one fixed scale: the rectangle turns long and flat,
    with live R and "at most" readouts that settle on the printed facts.
  - "All of pi comes from the real line": the segment lights and the bump's
    area brightens.
- **13_10:** the half circle lights, the max 1/(R⁴ − 1) appears, then
  length × max; "R = 2: 0.4189" and "R = 10: 0.0031 < 1/300", where 300 is a
  computed fact.
- **11_8, the refusal on a real coarse loop:** z⁸ on the unit circle, left of
  the code panel.
  - Eight samples light up one by one and every image lands on 1: "8 samples
    say 0 turns".
  - A walker then shows one whole turn hiding in every step: "between the
    samples: 8 turns".
  - With L = 8, one step's reach is 6.28 against |a| = 1, so it is wider than
    the output window. The window is shaded, labelled "reach covers 0", the
    code's refuse line gets a band, and the row reads "refuse".
  - At 64 steps the reach is a true-scale disk of radius π/4 around `a` that
    misses 0, the row reads "passes", and a card says "64 steps: 8 turns,
    certain".
  - This replaces the old schematic two-sample sketch.
- **07_5, the 2-D squeeze:** a jagged g is painted on the plane.
  - It is piecewise constant on polar cells at every scale, with |g| < 1 and
    no limit at 0. Hue is its direction.
  - A whole disk of 90 steps h sits on it. In the ratio's window from 07_4,
    the 90 ratios of g·|z|² jump about discontinuously but stay inside the
    disk of radius |h|.
  - Both disks shrink together from 0.55 to 0.06, with a live "inside radius".
  - This replaces the real-line slice between ±x² and the note about it.
- **01:** "x = 1.1" is now built from `bump.read_at`, and "coming up" is a
  constant.
- **13_10 also:** the sum's range "[−200, 200]" comes from a fact.

## What the increment 03 chapters show

- **02:** 3 + 2i as how far right and how far up. Adding (1 − i) tip to tail
  with the sum arrow, from `turn.sum`. A grid sheet with an F glyph, turned by
  ×i, with a live degrees readout. Then i·i = −1 as two quarter turns, and
  1 + i's length and angle named. The prediction, then (1 + i)² = 2i: lengths
  multiply, angles add.
- **03:** a point in and a point out, then two framed planes.
  - The z² walk, with both planes on one scale so radius ¼ is drawn half as
    large as ½. Live turn counts read 1 and 2.
  - The grid prediction, then every line morphs exactly, pointwise:
    parabolas, the axes folded onto the two real rays with a fold tick, and
    right-angle marks at three crossings in both planes plus the red straight
    angle at 0.
  - 1/z: a point walking out and back, reflected unit-circle ticks, and
    "1/0 undefined".
  - e^z: e^(−1), 1 and e from `exp.circle_radii`. The x = 1 line climbs once
    around its circle, then the other circles grow, then the rays.
  - A climb of 2πi lands back on the computed 1.000000 + 0.000000i from
    `exp.after_2pi`, with e^(z + 2πi) = e^z and e^z ≠ 0.
- **05:** a walk around the cubic's three zeros, and a walk around the
  double pole of (z − 1)/(z + 1)².
  - exp's period, then sin's real line and climb.
  - tan's limits at ±i∞.
  - Γ's poles at 0, −1, −2, synced to the words.
  - ζ's pole, trivial zeros and Newton-found critical-line zeros, with the RH
    card.
  - The six-panel mosaic, ending in one traced colour tile of z³ − 1.
- **08:**
  - One step's product drawn as three arrows from one origin (value, step,
    product; lengths multiply, angles add), then the sum formula and a finer
    cut.
  - f = z: the 24 products laid tip to tail, each step lit on the circle as
    its arrow lands, with a live distance-from-start readout. The chain closes
    twice; |total| comes from `loop.z.total`.
  - 1/z: the product points straight up, and the column stacks with a live
    height readout. Then "24 steps: 6.27i" and "limit: 2πi ≈ 6.28i". The
    circle grows to radius 2 and the column does not move.
  - z⁻³ … z² at one common scale: each closed chain is marked from
    `loop.powers.closed`, and the boxed z⁻¹ column "2πi" survives.
  - Cauchy by tiling a square loop: circulation in each cell, one shared
    edge's two opposite walks, the inner edges cancelling, and only the
    boundary left lit. Goursat's "precisely" lower third.
- **15:**
  - 01's graph and degree-40 recipe; the plane rises with ±i and the radius 1
    circle.
  - The demand (06's quotient, "the same from every direction"), then five
    recap panels lit as named. The first says "on an open region: stretch
    and turn"; the others are boundary → inside, a loop counting zeros −
    poles, the half-disk residue contour, and two continuation paths ending
    on two values.
  - The prediction "radius at 3?" (from the fact), then the circle through ±i
    with "√10 ≈ 3.162", and the interval −0.16 … 6.16 on the real line.
  - A title card ("The Hidden Geometry of Complex Functions"; the
    title is root's or the owner's call).

## Checks actually run (all pass)

From `explainer_corpus/complex_analysis_visual/` unless noted:

1. `python3 -I -B oracle.py`: 120 facts, 0 failed cross-checks.
   - New in increment 04: z⁸ turn sums over 8, 64 and 4096 steps
     (0, 8, 8) computed in numpy, reach 6.28,0.79, refused true; radii
     2,5,10 and 2,10; `integral.quartic.under` 300, by a different formula
     from the Rust.
   - The radii, step counts, read point and sum range are chosen parameters,
     so the oracle restates them rather than recomputing them.
2. `python3 -I -B checks/oracle_vs_asserts.py explainer_corpus/complex_analysis_visual <oracle stdout>`,
   run from the staging directory:
   - 120 keys on both sides, none missing, in the same order (also checked
     with a multi-line parse);
   - 70 literal values asserted in the Rust tests, none disagreeing.
3. `python3 -I -B ../../checks/scene_crosscheck.py .`: all 15 scenes speak
   exactly their chapter's cues in order, which is all 105 cues.
   - Every `wait_sentence` index exists, and every `wait_for`/`until` phrase
     occurs in its cue.
   - Every fact key exists, and every `predict` sits on a `PREDICTS` cue.
   - Result: 0 problems.
4. Glyph-supplement resolver (scratch, not shipped): it resolves every
   `chapters::{TEXT, TEX, MONO}` entry, 2-D arrays included.
   - 236 / 130 / 6 entries, with no duplicates, which the
     `no_supplement_is_empty_or_duplicated` test requires.
   - No pub string constant of any chapter is left unlisted.
5. `rustfmt --edition 2024 --config max_width=120 --check src/main.rs` parses
   every module with 0 errors. My files are formatted; the only diffs are in
   the canonical copies `thumbnail.rs` and `score.rs`.
   - I format leaf files by name only: rustfmt on `mod.rs` rewrites its
     children, and that once touched `thumbnail.rs`. It was restored, and
     `cmp` now equals canonical.
6. Storyboard words column = `script.tsv`, all 105 beats: a scratch parse,
   not the Rust gate `the_storyboard_the_registry_and_the_script_agree`.
7. Python prototypes of the new tests' numerics:
   - 03's windows and the morph's intermediate points;
   - 08's label gaps (≥ 0.996), chain boxes (12-gon x 0.887…3.913 against the
     readout column at 4.0), the column at −2.8…2.0, the six row scales all
     0.45, and the radius-2 column identical;
   - 05's tile (sides 0.32–0.34, band error 9e-16);
   - 13's unrolled window: rectangles 1.06 × 1.39, 2.65 × 0.17 and 5.29 × 0.04
     scene units, with screen areas in the ratio of the bounds; the size
     profile peaks exactly at 1/(R² − 1) for all three radii;
   - 07's jagged g, using the same integer mixing as the Rust: |g| in
     [0.4, 1), spread above 1.6 at each of 61 scales down to 2⁻³⁰, every cell
     middle reporting its own indices, and every sampled ratio within |h|;
   - increment 05's new tests: the radius-2 mean over 256 samples (about
     3e-18 from the limit), the kept angle (89.9999966°, printing 90), the
     partial sums (11 terms, largest 1.999), and |z⁵ + 1| = 2 at z = 1, the
     first sample of `circle`.

8. The desk checks in "What increment 06 repaired", run from the crate. They
   are regex over text, so they can miss things and are no substitute for
   rustc.

**Not run:** `cargo build`/`test`/`clippy`, the glyph probe, any render or
contact sheet, TTS, ASR, listening, fine sync, or independent AV review.
Every layout claim is arithmetic from text-size heuristics and unit tests,
not from looking at a frame.

## Known compile and render risks (please qualify)

- **The Rust is uncompiled**, all of it. APIs were read from fmn and kit
  source:
  - `add_updater`/`set_points`/`move_to`, `set_tracker_value`,
    `kit.live_value`, `Arrow`/`DashedLine`/`polyline`;
  - `props::equation_card`, `precisely`, and `Painter.to`.

  Expect a compile-repair pass. I wrote it to avoid closure-to-fn-pointer
  coercion in tuples and `needless_range_loop`, but Clippy is unrun.
- **Glyphs** in text (not TeX), all listed in TEXT:
  - ² in 03's "straight after z²?";
  - ≈, →, √, π, − and ° in readouts and captions.
  - "f'" is ASCII in 08's Goursat lower third.
  - "zeros − poles" uses U+2212.
  - ≥ in 11_8's row (crofton's text already uses it).
- **TeX** used in new files: `\oint_{\gamma}`, `\Delta`, `\sum_k`, `\,`,
  `\frac`, `\ne`, `\times`, `\arg`, `\quad` and `\gamma`.
  - `\Delta` and `\big` appear in canonical films.
  - Increment 04 adds `\Rightarrow` and `\max_{|z|=1}`.
  - None of it has been rendered here.
- **Render cost:** 03_6 reveals 7 polylines of 121 points per frame and 08_3
  re-sets 24 chords plus a 240-point circle per frame for 1.6 s; both should
  be cheap. 05's Painter crossfades are as in increment 02.
  - 07_5 moves 180 dots and re-sets 2 rings per frame over a group of 208
    filled cells, and is the one to watch.
  - 11_8 re-sets a trail of up to 961 points per frame.
- **Unchecked API assumptions in increment 04:**
  - `set_points` on a `polyline` styled `flat` is relied on to keep its fill
    in 13_3's rectangle (precedent: the cover in `continuing.rs`);
  - `get_center`/`get_width` on a code line after the panel's slide
    (precedent: `capture_recapture`).

## Remaining gaps

- **Mail 757's three mechanisms are written, not yet seen.** They are source
  only; no frame of them exists.
- **Numbers still typed on screen.** None is a computed result, and none has
  a test binding the string to the code. I left them because they name a place
  or state a theorem, and the words say the same thing. If you read AGENTS.md
  as covering these too, tell me and I'll route them:
  - the point 3i: `sin(3i)`, `sin²(3i)`, `cos²(3i)`, "3i", with
    `C::new(0.0, 3.0)` as a literal in `gallery.rs`, `rigidity.rs` and
    `facts.rs` (the facts' key `gallery.sin.at_3i` names it);
  - the centre 1: "recipe centred at 1:", with `1.0` a literal in the
    `bump_recipe` and `bump_radius` calls;
  - 01's "at x = 1?", 09's "average = 1?", "bump at 0", 14's
    "1 + 2 + 4 + 8 + … = ?", "every z except 1", "0, −1, −2, …";
  - 05's "Re s > 1", "s = 1", "Re s = 1/2", "simple zero at 1", "−1 ≤ sin x ≤ 1";
  - axis labels such as the "0" in 11 and the "1" in 02.
- **Not done:**
  - compile, glyph probe, draft renders, contact and cue sheets;
  - Oliver synthesis (105 cues, serial, root);
  - ASR text checks (kept separate from listening);
  - human listening, fine sync, independent AV review.
- The pronunciation lexicon (Cauchy, Riemann, Liouville, Rouché, Goursat,
  Lanczos), a de-slopify pass, the thumbnail and title.

## Strongest unresolved teaching defects

1. **The new mechanisms ask a lot of a short time, and nobody has watched
   them.** All timings are estimates until Oliver's takes exist.
   - 07_5: the whole squeeze has one sentence. Clearing the old picture,
     painting g and fading in both disks takes about 2.2 s before the shrink,
     which starts at "and the product". If the sentence runs short the shrink
     has a 1.5 s floor and overruns into the hold. The painted g may also be
     loud enough to compete with the ratio window, which is the point.
   - 11_8: "But samples alone can miss a turn made quickly between them" gets
     eight full turns on a 1.8 s floor. The trail overlaps itself on one
     circle, so only the counter and the walker's speed show that there are
     eight.
   - 13_3: the right column changes six times within one line, so I expect it
     to be among the most crowded screens in the film.
2. **Three pictures stand in for what cannot be drawn at scale.**
   - 13_3: the plane's arc stays at R = 2 while the unrolled rectangle grows
     to R = 10, because the window cannot hold it. "Unrolled" is shown, not
     said.
   - 11_8: the coarse reach is a shaded window, not a disk. A test proves the
     disk contains the whole window.
   - 07_5: g is one particular jagged function standing for "any bounded
     function". The cue's "smooth or jagged" shows no smooth case, and the
     screen says M only through `|g| ≤ 1`.
3. **Length:** about 31 min of narration plus holds and listen windows. A
   split after 07 remains an owner or root decision.
4. **08_5 departs from the storyboard's disk.** The loop is a square, so the
   small squares fill it exactly. I chose this over the honesty problem of
   ignoring a disk's cut rim cells. The narration ("a loop", "tiny squares")
   fits either.
5. **08_1 labels the arc midpoint z_k** (where `model::loop_products`
   evaluates). That is consistent inside the film, but differs from texts
   that sample at step starts.
6. **15's recap icons are schematic.**
   - The stretch-and-turn panel multiplies a square by 2 + i, not a named
     function's derivative.
   - The zeros and pole in the counting panel are placed by hand. No numbers
     are shown on these icons.
7. **03_7's computed card "1.000000 + 0.000000i"** may read as fussy. A
   shorter print needs a coarser fact, which is a root or editorial choice.
8. **None of the 13 prediction pauses, and no new chapter, has been watched.**
   Text widths come from a size heuristic, so overlaps are possible. The
   tightest are 08's readout column against the f = z chain (about 0.14
   scene units), 03_4's two ray captions on the caption row, and 11_8's gap
   of about 0.3 between the output window and the panel.
