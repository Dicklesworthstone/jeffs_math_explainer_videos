# Chapters 11 and 00 picture repair, preview01 (Mails 1088, 1090, 1100, 1137)

RubyRaven, bead `mx-video-program-program-87do.63`. Staging only; root owns canonical.

**Status: uncompiled, unrendered, unlistened.** Nothing here has been built or seen on
screen. Fixes are predicted from source, frames, the lint's spans and a Python mirror.
Root's build, Clippy, preview, lint and frame look decide whether they worked.

## What to apply

Two independent patches, for `git apply` or `patch -p1` at the repo root. Paths are relative
to this directory.

| Chapter | Patch | Result file | Canonical base | sha256 starts: patch, result, base |
|---|---|---|---|---|
| 11 | `unreachable-picture-repair01.patch` (21 hunks) | `explainer_corpus/logarithm_mysteries/src/chapters/unreachable.rs` | `unreachable.rs` | `592989d16fd57f4f`, `b725f3688334b821`, `5f64624786f5ff88` |
| 00 | `ratio-ruler-repair01.patch` (5 hunks) | `explainer_corpus/logarithm_mysteries/src/chapters/ratio_ruler.rs` | `ratio_ruler.rs` | `f5e9242445255958`, `92cf028b124f8a37`, `b2428d611f66cd55` |

The base is canonical at build04, copied to `../snapshots/canonical-build04-preview01/`.
Root's build04 fixes are kept (the `const { assert!(..) }` and `for (offset, boundary)` forms
in chapter 11's fit test). `script.tsv` (`3e4e34af7090e448`), `oracle.py`, facts and
`chapters/mod.rs` do not change. ElevenLabs Oliver remains the narrator and no cue changes,
so this adds no synthesis.

If canonical has moved past those bases, check the patches again before applying.

## Chapter 00 (Mails 1100, 1137)

**The false caption, 00_3 (sheet 032, 55.0–55.5 s).** "distance between notes = ratio of
frequencies" is now **"note interval depends on frequency ratio"**, one of root's two wordings.
I chose it over "equal note intervals = equal frequency ratios" for two reasons:

- It says what cue 00_3 says at that moment: "is set by the ratio of their frequencies".
- The other wording states the rule the film is about to earn. It would sit on screen right
  before 00_4's prediction, and would pre-empt 00_5's payoff, "equal steps on the keyboard =
  equal ratios".

The other wording is a one-string swap if root prefers it, and it uses no new characters
either. The cue text is unchanged.

**The grid over "motions of the plane", 00_6.** The lint's single hold at 116.6 s understates
this. The corner enters the words while the grid turns (sheet 061, 106.0–107.0 s) and stays
there to the last frame (sheets 064, 068, 071, through 124.55 s). Computed over the whole turn,
t in [0, 1], the turn itself is unchanged:

| | y |
|---|---|
| Grid's lowest point, mid-turn at t = 0.758 | −0.501 |
| Grid at rest, end of the turn (the lint read −0.46) | −0.453 |
| Old words' top (centre −0.4) | −0.27 |
| New words' top (centre −0.8) | −0.67 |

So the row clears the true low point by 0.169, not just the rest position.

**Found by looking: the dial touches "laps forgotten"** (sheets 064, 068, 071). Neither the lint
nor the mails name this. The ring's foot is at y −0.30 against that word's top at −0.27. Moving
the whole row fixes it, with 0.37 clear, and keeps the three words aligned.

**The change.** "multiply by sliding", "laps forgotten" and "motions of the plane" move to y −0.8
(`PREVIEW_WORD_Y`). "and a transformation…" keeps its 0.6 spacing beneath them, at y −1.4. The
grid's generator, spacing and reach become constants so the new test and the scene share them.
Rulers, dial, grid and timing stay where they were. The new test fails at the old row, in the
mirror, and passes at the new one.

## Chapter 11: lint flags and the frames confirming them

Times are from the scratch-voice preview01 render, so they will move with Oliver.

| Time | Flag | Frame read | Cause | Change |
|---|---|---|---|---|
| 29.8 | crossed: zero line through the tick "0" | 0069 | Tick labels at `GAUGE_X − 0.35`, where the zero line started | Labels at −0.45, line from −0.25 (`GAUGE_LABEL_X`, `ZERO_LINE_LEFT`) |
| 30.0 | overlap: "signed area of the square:" and the live value | 0069 | Centred word and live value shared y 2.45; the value started at x 2.0, inside the word | Word at y 2.72, value beneath at y 2.36, centred on the gauge (`READOUT_*`) |
| 60.8 | overlap: live value and "det = signed area factor:" | 0136 | Same | Same constants |
| 142.8 | edge: "each gap part…" to x 7.08 | 0313 | One line too long for the column | Three lines at y −1.1, −1.5, −1.9; `only_diag` down to y −2.45 |
| 168.2 | behind: gold axis under "where A puts the F" | 0369 | Label on the panel | Label moved to a key in the column (see below) |
| 176.2 | crossed: grid over the same label | 0385 | Same | Same |
| 197.4 | outside: "equal stretches…" to x 7.23, ends "commutes with −" | 0432 | One long line, begun at x 1.0 | Two lines in the column at y 0.1, −0.3 |
| 206.2 | edge: "2×2…" to x 7.04 | 0449 | One long line, begun at x 0.3 | Two lines at y −0.8, −1.2 |
| 233.1 | outside: the rule, to x 7.13 and x 7.50 | 0503 | Two long lines, begun at x 0.3 | Four lines at y −1.75, −2.15, −2.55, −2.95; `cite` stays at y −3.62 |

**The target F key (11_6).** On the panel, the words were cut by the gold axis, and sheets
098 and 101 show the blue square sweeping over them at about 180–187 s. A plate cannot fix
that, because the plane moves through the spot. The words now stand in the column between
`positive` (y −0.05) and `stage_word` (y −0.85), beside a small red copy of the same
outline, so the colour and shape identify the target. The outline on the panel is unchanged.
The label went from 18 pt to 20 pt to match its column neighbours.

**Root's sheets from Mail 1100.** Sheet 126 (233.5–235 s), which I also opened, shows the same
clipped rule as frame 0503. Sheets 078 and 096 contain my frames 0313 and 0385, so the gap line
and the F caption there are the rows above. All three are covered by these changes.

## Chapter 11: found by looking, not flagged by the lint

The lint ignores strokes fainter than `FAINT_INK` (0.35), and the static grid is 0.5 ink, so
these pass it while still visibly touching the words.

| Where | Seen in | Change |
|---|---|---|
| "before" and "after" touch grid lines (11_1) | sheets 004, 010 | Centred in their grid cells, `[±0.5, 1.25]` |
| "×2, flipped" has a grid line through it (11_3) | frame 0198, sheets 049, 053 | `on_plate` |
| "×½, flipped" has a grid line through it (11_3) | frame 0198, sheets 049, 053 | `on_plate`, and x + 0.35 so the plate clears the faint F's stem |
| "u = (1, 1)" has a grid line through it (11_4, 11_5) | frame 0313, sheets 060, 067, 072, 087 | `on_plate` |
| "det < 0: no real logarithm" sits 0.5 right of the column | frame 0198 | Back to x `right`; the offset only dodged the old readout |

`on_plate` follows `props.rs` L137–155: measure the text, build a background-coloured
`Rectangle`, then group plate and text so the plate paints first.

## Why my fit test passed text that clipped

`the_panels_and_text_column_fit` estimated width as 0.55 em × size / 144 per character,
which is about 20% short. Measured width is about 0.01 scene units per em per point, with
CMR10 advance widths. My test passed lines that were off screen, so the fault was mine.

The test now uses a CMR10 class table and tests the new lines against the lint's real text
limit, x < 6.86 (`7.11 − TEXT_MARGIN`), with 3% room.

| Mirror of the Rust table against | Rows | Error |
|---|---|---|
| Lint spans on preview01 | 5 | −1.7% to +2.0% |
| Widths read from frames | 9 | −4.0% to +3.2% |

## Wording, timing and maths

- **Chapter 11:** words are unchanged. Comparing every string literal outside the tests,
  canonical against candidate: 6 removed, 11 added, and the added lines join back into exactly
  the removed words, in order. All other literals are identical. The joined text keeps
  `{gap_factor}` and both `shown_num` values, so facts still feed it. All 37 `until_sentence`
  and 8 `kit.say` anchors are identical and in the same order.
- **Chapter 00:** exactly one literal changes, the caption root asked for. All 15
  `until_sentence` and 6 `kit.say` anchors are identical and in the same order.
- Neither patch brings a new character to the glyph probe.
- Fade durations are unchanged, so no beat gets longer or shorter.

## Checks actually run

Scripts are in the session scratchpad and are not part of the handoff.

- `rustfmt --edition 2024 --check` with the repo config: bases and candidates all clean.
- `git apply --check` against canonical passes for both patches. Each, applied to a scratch copy
  of canonical, reproduces its candidate byte for byte.
- Chapter 00's turn and test bounds, computed in Python as above.
- Chapter 11, Python mirror of the test table: all 22 fit rows and the 3 full-width rows pass. The tightest
  are "e^(tp) > 0…" at 6.791 and "e^(tX) stretches…" at 6.773, against 6.86.
- Mirrored new tests: readout word right edges 2.147 and 2.112 (limit 2.2); tick "−1" right
  edge 0.568 (limit 0.600); key F x 2.200..2.875, y −0.596..−0.304.
- Plates against what moves: no route arrow in 11_4, or in 11_5 at 21 steps of the off-diagonal
  scale, passes under the `u` plate. The arrows clear the `×2` and `×½` plates, and the `×½`
  plate clears the faint F's stem by 0.02.
- Canonical `src` equals the snapshot (`diff -rq`), all 30 snapshot checksums pass, and the
  frozen candidate (`../explainer_corpus`, `../HANDOFF.md`, `../snapshots/handoff-mail785`)
  has no file newer than the new snapshot.
- Compile risk was only checked by reading: signatures and precedents for every new call, by
  hand. `cargo` was not run.

Mirrored means a Python copy of the Rust arithmetic and strings. It is evidence the tests
should pass, not a test run.

## What was inspected

No audio was heard and no continuous motion was viewed. Stills only, from
`logarithm-opus-rebuild-frames01/`.

**`11_unreachable`:**

- **Full frames:** 0069 (30.0 s), 0136 (61.0), 0198 (90.0), 0313 (143.0), 0369 (168.0),
  0385 (176.0), 0432 (197.5), 0449 (206.0), 0503 (233.0).
- **Sheets:** 004 (6.5–8.0 s), 010 (18.5–20), 020 (35.5–37), 024 (43.5–45), 038 (68.5–70),
  049 (89.4–90.5), 053 (95.5–97), 060 (109.5–111), 067 (123.5–125), 072 (130.5–132),
  087 (159.7–160.3), 098 (179.5–181), 101 (185.5–186.8), 104 (189.5–191), 126 (233.5–235),
  130 (238.5–240), 137 (252.5–254).

**`00_ratio_ruler`:** sheets 032 (55.0–55.5 s), 061 (105.5–107), 064 (111.5–113),
068 (119.5–121), 071 (124.39–124.55).

That is 17 of chapter 11's 140 sheets plus 9 full frames, and 5 of chapter 00's 72 sheets.
It is **not** complete picture acceptance of either chapter. Most sheets were not opened.

## Left alone, for root to decide

1. `draw::arrow` heads look hollow in 11_3 (frame 0198). It is a shared helper, so changing it
   reaches other chapters.
2. `draw.rs` L488 `matrix_card` comment repeats the wrong "one em is size / 144". I did not
   check whether its sizes are affected.
3. Both chapters' header comments still say "uncompiled, unrendered, unlistened", now stale.
4. Chapter 11's panel is empty for about 187.5–190.5 s (sheet 104), and only the title shows for
   about 236.3–239.5 s (sheet 130). Oliver's timing will change both, so judge them after
   narration.
5. The frozen older staging (`../explainer_corpus`) still holds the false 00_3 caption. It stays
   frozen by order, so it should not be used as a source for that line.
6. A hook wrote `.ee/hook-state/session_start_orient.last.json` into this directory. dcg
   refused moving it out, so it is still here. It is not part of either patch.

## Prediction for chapters not yet previewed

The width model, run over the literal text calls in all chapters, predicts one more line past
the lint's limit outside chapter 11: `matrix_logs.rs` L354–358, 09_4's "turn-and-scale matrix
↔ complex number", centred, predicted to span x 3.6..7.2. Treat it as a lead: `↔` has no measured
width in the table, nobody has looked at a frame of it, and no preview of that chapter exists.

The scan covered 185 calls and skipped 41 whose x it could not compute. It also misses every
`format!` string, which is exactly what hid "each gap part…" from it. A clean result here does
not clear a chapter.

## For root to run

Cargo, render and lint are root's. In order:

1. Apply both patches to canonical, or take the result files.
2. Run the crate's tests and strict Clippy. The width table and new tests have never compiled.
3. Re-render previews 11 and 00 with `--lint`.
4. Look at the same moments, since the flags could clear while a new collision appears:
   - **Chapter 11:** readout about 30 s and 61 s; gap caption about 143 s; F key about
     168–187 s; 11_7 captions about 197–234 s; plates about 90 s and 110–160 s.
   - **Chapter 00:** caption about 50–55 s; words under the turning grid about 105–107 s; the
     final row about 112–124 s.
