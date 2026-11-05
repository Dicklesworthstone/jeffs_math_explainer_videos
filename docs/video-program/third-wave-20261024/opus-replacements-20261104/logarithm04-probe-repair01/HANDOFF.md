# logarithm04-probe-repair01: the probe in 04_2 and the caption in 04_4

Status: staged and frozen. Not applied, not built by Cargo, no native test run, not rendered, not heard. Canonical is untouched.

Narration: ElevenLabs Oliver. Narration wording delta: none. `script.tsv` stays `f201d2dc57fa9cca`, so every take's text stands. No synthesis is needed.

Answers Mails 3018 and 3047. Findings repaired: root's sheet 027 (50.0 s), sheet 031 (54.9 s), and the caption at 97–104 s.

## What changes

| Line | Cue | Old | New | Why |
|---|---|---|---|---|
| 360 | 04_2 | `stage.add_value_tracker(2.6)` | `stage.add_value_tracker(PROBE_START)`, 2.16 | The line used to start above the name and sweep through it. 2.16 is the top of the whole path and lies under the name. |
| 402 | 04_2 | `tween(c, 2.0, 1.6)` | `tween(c, PROBE_REST, 1.6)`, 1.5 | The still line lay across the tick label 2. 1.5 is half way between the labels 1 and 2. |
| 551 | 04_4 | `+1% in x  →  +0.01 in ln x` | `+1% in x  →  about +0.01 in ln x` | The caption stands alone and ln 1.01 is 0.00995. |
| after 860 | | | `PROBE_START`, `PROBE_REST` | Placed between the impl's closing brace and `#[cfg(test)]`, so no scene line moves. |
| tests | | `[0.15, 1.0, 2.0, 2.6]` | `[0.15, 1.0, PROBE_REST, PROBE_START]`, plus 2 new tests | The new tests guard the whole path by source order and const asserts, and pin the caption. |

The full-width line, its colour change, the moving dot, the counter, all three run times (2.4, 1.4, 1.6 s) and every later beat stay as they were. No timing changes, so the scratch cue table should not move: all 12 stand-in scene runs end at the same times in both trees.

The name is not moved or hidden. `EXP_NAME_X`, the name's `tex` call, the fade before the flip and the flip itself are byte-identical, so the placement constraints carry through the reflection sequence unchanged. The name still leaves at 04_4 sentence 1, before the mirror turns.

## Why this fix

Measured on root's frames (`checks/frames.out.txt`), in 720p pixel rows:

| Ink | Rows |
|---|---|
| Name's lowest ink | 193 |
| New start, line centre | 201.6 |
| Tick label 2 | 210–221 |
| New rest, line centre | 261.0 |
| Tick label 1 | 300–311 |

- **Start:** 6.5 px clear above and 7.3 px clear below.
- **Rest:** 37.9 px clear on both sides.
- **Path:** the legs run start → 0.15 → −1.2 → rest, and the rest is below the start, so the start is the highest the line ever goes. This guards the animated path, not only its endpoints.
- **Rows:** on frame 124 the only ink on either new row is the y axis and the curve.

The old start also stood on the green dot at (1, e) on frame 107 (49.5 s). The new start clears that as well.

### Rejected

| Option | Why not |
|---|---|
| Backing behind the name, or draw-order changes | Cuts a visible gap in the moving line and adds a dark box beside the mirror and "rise" constraints. |
| Hiding the name during the sweep | Adds a pop in a section that already overruns its cue, and takes the name away while the voice is on this curve. |
| Moving the name above the sweep, near y 2.9 | Touches the name's literals, its rise-label test and the mirror and dot constraints, for no gain over a lower start. |
| A gap or backing at the tick label | Makes the seams Mail 3018 warned about, and the rest height removes the overlap without any new object. |
| `≈` in the caption | `about` is plain Latin in a font already in use, and reads as the voice's "almost exactly". |

**Cost:** the line no longer reaches the top of the curve, and it rests at 1.5 rather than at the 2 that 04_3 uses for `across`. The probe has faded before `across` arrives, so nothing visible ties the two together. The range 0.15 to 2.16 still shows one crossing at each height.

## Left in place, and disclosed

The moving line still crosses tick labels in passing. These are modelled on a 0.01 s grid on the scratch timeline; the model reproduces the old line within 0.009 graph units on 10 frames.

| Label | Crossings |
|---|---|
| 2 | 49.82–50.03 s |
| 1 | 50.65–50.75, 54.27–54.32 s |
| −1 | 52.74–52.86, 53.50–53.59 s |
| x-axis row | 52.33–52.41, 53.83–53.88 s |

The longest is 0.21 s, and the line never stops on one. I did not protect them, because any protection brings back the seams. It is root's call if the new frames say otherwise.

**Worth your eye:** the line stands at the start for 0.4 s (49.0–49.4 s), 6.5 px under the name, while "times the curve reaches this height:" fades in. It might read briefly as an underline. If it does, the next option is a start below the tick label 2, which gives up more of the range.

## Bindings

| | Baseline | Candidate |
|---|---|---|
| `undoing_growth.rs` sha256 | `754ea3125c2f7e89440807772713fe57584c96937be3ec0ac8c52d86645cf323` | `4dc4c555122d9cd5423d7094336c1d5e6f7f9afb77d6850a38bc0148c20180ef` |
| Lines | 1097 | 1135 (scene lines 1–860 keep their numbers) |

Patch: `undoing_growth.patch`, baseline to candidate. It applies and reverses byte for byte on a scratch copy. Its hash is in `SHA256SUMS`.

### Map13 hold row, the only one bound to this file

| Field | Old | New |
|---|---|---|
| cue → next | 04_6 → 04_7 | same |
| `sha256`, `evidence.sha256` | `754ea312…f323` | `4dc4c555…80ef` |
| `line` | 749 | 749 |
| `marker` | `predict(&kit, stage, v(4.6, THINK_BAR_Y), 3.0)?;` | same |
| `interval_sha256` | `b2d1d76b8f9fbd9eaa730d0c96aba14999df1e10304ef8e99526d92ebdc91c17` | same |

My interval hash reproduces map13's stored value on the baseline, which is the control for that reading.

The old hash appears twice in canonical's crate docs, both in map13 (`sha256` and `evidence.sha256`).

### Binder, `bind_final80-stage06.py`

- `SITES` row `("undoing_growth.rs", 351)`: still `super::draw::wait_until(stage, onset)?;` on line 351.
- All 7 says and all 3 timing sites keep their lines and text.
- `phrase_marks` uses stay at 1.
- 04_2's phrase anchor sites stay on lines 115 and 351.
- No `SITES`, anchor or shared-helper change.

I read the stage06 binder and did not run it. Its stage may have moved on.

## Native re-review anchors

Times are the scratch-voice preview and should hold on a scratch rebuild. Under Oliver, count from 04_2 sentence 4 (scratch 48.7 s) and 04_4 sentence 5.

| Check | From sentence | Scratch | Frames, sheets | Look for |
|---|---|---|---|---|
| Line appears at the new start | +0.3 to +0.7 s | 49.0–49.4 | 106–107, sheet 26 | No contact with the name or the label 2. Does it read as an underline? |
| Root's finding, first leg | +0.7 to +3.1 s | 49.4–51.8 | 107–112, sheets 26–28, was 108 on sheet 27 | Name clean on every frame; line never above it. |
| Below zero | +3.1 to +4.5 s | 51.8–53.2 | 112–117, sheets 28–29 | Counter reads 0, dot gone, as before. |
| Rise and rest | +4.5 to +6.1 s | 53.2–54.8 | 118–122, sheets 29–30 | Stops between the labels 1 and 2. |
| Root's finding, rest and fade | to `say 04_3` + 0.4 s | 54.8–55.3 | 122–127, sheets 30–31, was 124 on sheet 31 | Label 2 clean. Under Oliver this rest may last longer. |
| Caption | 04_4 sentence 5 + 0.5 s | 97.5–104.8 | 223–242, sheets 55–60, frame 228 | `about +0.01` whole, on one line, ending near column 1054 (estimate), clear of "that is why…" below. |
| Reflection, regression only | 04_4 start to sentence 1 | from 76.3 | around the flip | Name and mirror as in the current preview. No change expected. |

**Lint:** the row `before 04_3 @54.9s crossed a stroke … painted over "2"` should go away, and no new row should appear for a stroke near scene y 1.10.

**Glyphs:** the caption adds only `a b o u t` and a space, in a string that already renders. Its first native appearance is still unseen.

## What was run

Every result is in `checks/*.out.txt`.

| Check | Result | Proves | Does not prove |
|---|---|---|---|
| `checks/frames.py` | Pass | The new heights against ink on existing frames | What a new render shows |
| `checks/source_checks.py` | Pass | Line pins, hold row, patch round trip, `rustfmt --check` | That the binder itself accepts it |
| `checks/stand_in_run.py` | Base 81 tests, candidate 83, clippy clean, 8 of 8 controls | Types, borrows and the new tests' teeth | Real signatures, TeX, glyphs, frames |

- **Stand-ins:** bare rustc on the repo's pinned toolchain against hand-written crates, with draw.rs's own tests dropped. This is not native evidence.
- **Controls:** one unmutated build passes. Four wrong constants are refused at compile time with E0080. The bare caption and both scene literals compile and fail the expected test. An earlier version of these controls passed vacuously, because a toolchain mismatch refused every build; the positive control exposed that, and the results above are from the corrected run.

**Not done:** Cargo build, native tests, render, listening, fine sync.

## Limits and independence

Not independent. I wrote the earlier repair of this chapter's source files in `logarithm-repair01-04`, and this repair answers root's findings without a second reader. Root's frames were cut with the scratch voice at 720p, and all pixel figures come from those frames. Times are scratch, not Oliver. Continuous pacing here remains unreviewed, as Mail 3047 says.

## Files

- `undoing_growth.patch`
- `explainer_corpus/logarithm_mysteries/src/chapters/undoing_growth.rs` (candidate)
- `source-markers.json`
- `checks/frames.py`, `checks/source_checks.py`, `checks/stand_in_run.py`, each with `.out.txt`
- `SHA256SUMS`
