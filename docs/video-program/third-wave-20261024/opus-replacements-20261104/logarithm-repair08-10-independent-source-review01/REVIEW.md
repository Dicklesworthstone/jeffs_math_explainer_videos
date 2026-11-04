# Logarithm 08–10 staged repairs: independent source review

Reviewer: BronzeJaguar. Assignment: Agent Mail 1533 from BlueSalmon, under root-owned bead `mx-video-program-program-87do.63`. Date: 2026-11-04.

Scope: WhiteSwan's `logarithm-repair08-10` patches, read against the pinned engine and Kit, for API use, maths, object lifecycle and timing.

**Status: source reading and arithmetic only.** Nothing was compiled and no test or Clippy was run. Nothing was rendered, heard or looked at as a picture. No canonical, combined-stage or shared file was edited. Oliver has no timing for these chapters yet, so every time below comes from the scratch rule.

## 1. Result

**The repairs hold up as written.** I found no wrong maths, no call the pin does not provide, no object faded twice, and no resting copy left behind by a hand-over. I expect the code to compile, though nothing here has compiled it. Four things need attention, one of which should block the build:

| ID | Severity | Where | Defect | Fix |
|---|---|---|---|---|
| **R1** | **Blocks strict Clippy (expected; native Clippy decides)** | `matrix_logs.rs:442–443` | Two `assert!`s on constants only | `patches/r1-const-asserts.patch` |
| **R2** | Low | `doing_nothing.rs:369` | The three gold family ellipses still pop on at 10_2 sentence 3, though F17 says "No pops" | `patches/r2-family-fade.patch` |
| **R3** | Low (test strength) | `draw.rs:695`, `:685`, `matrix_logs.rs:449`, `doing_nothing.rs:738` | The tests for F1, F2, F3 and F5 never touch the typeset text or a real flow | `patches/r3-real-box-tests.patch` |
| **R4** | Nit | `doing_nothing.rs:398` | A comment still gives the window F4 replaced | `patches/r4-window-comment.patch` |

There is also one small correction to `REPAIR.md`: chapter 10 grows by about 4.5 s, not 5.1 s (section 5).

Only R1 needs action before the build. R2–R4 are optional, and each patch applies alone. None of them changes a cue, an anchor or the chapter's length.

## 2. Bindings

| File, under `explainer_corpus/logarithm_mysteries/` | Canonical and baseline sha256 | Candidate and `combined-repair-stage01` sha256 |
|---|---|---|
| `src/chapters/draw.rs` | `59ebd09d2fa5c793` | `82c7d086462eafd3` |
| `src/chapters/product_rule.rs` | `a8b4e31d98b23bd7` | `e9019c828710207b` |
| `src/chapters/matrix_logs.rs` | `4250a5f0294f58a3` | `ebc6c0f4ce10fce6` |
| `src/chapters/doing_nothing.rs` | `bf0008113e804186` | `b37ed55c69345f6a` |
| `docs/storyboard.md` | `c5bc6e7e9fbd0797` | `617897027ddfc55f` |
| `script.tsv` | `3e4e34af7090e448` | unchanged |

| Item | sha256 prefix |
|---|---|
| `logarithm-repair08-10/REPAIR.md` | `612f1b02ee087251` |
| `patches/shared-draw-helpers.patch` | `3871aa3b88a300a1` |
| `patches/chapters-08-10.patch` | `c71d4bfa9ca0a9f6` |

The engine is `franken_manim` at pin `a399d626`, the rev the crate's `Cargo.toml` names. Line numbers below are the candidate's unless a file says otherwise.

Root's newer stages were read but not edited. `combined-repair-stage02` differs from the candidate only in `draw.rs` (`93c2f95108d11af5`), by the arrow patch. `narration-binding-stage01` splits 05_2 and 06_6 and adds a marker in `predict`. Neither touches anything reviewed here.

## 3. What I ran

Output is in `checks/`.

| Check | Result | What it does not show |
|---|---|---|
| `patch -p1`, both repair patches, scratch copy of canonical | Exit 0, no fuzz, byte-equal to the candidate | |
| `rustfmt --check`, whole crate | No parse errors. The only file with a diff is `score.rs`, which these patches do not touch | That it compiles |
| `docs/check_cues.py` | Fails, with a failure list identical to canonical's (15 lines). Not caused by this repair | |
| Anchors, source order | `product_rule` unchanged at 26. `matrix_logs` 18→19 (adds `until_sentence 09_2 1`). `doing_nothing` 19→20 (adds `hold`). All `say` order the same | |
| `predict` calls | 2→2, 0→0, 0→0 | |
| 77 distinct added float literals, against Clippy's `approx_constant` table | None close | That no other lint fires |
| `checks/checks.py` | Text spans, card spans and a timing simulation | **Estimates.** CMR10 widths and the scratch rule, not measured boxes or Oliver |
| `checks/prototype.py` | A Python port of `model::mexp` and `inside_clip` | The Rust itself |
| `checks/build_fixes.py` | Each proposed patch applies alone. All four apply in either order. With all four applied, `rustfmt` finds no parse error and only `score.rs` differs. They also apply, with line offsets only, to a scratch copy of `combined-repair-stage02` plus `prediction-marker.patch` | **That they compile.** They are as uncompiled as the candidate |

I read every API claim below in the pinned source, and I read the candidate's code itself. I did not rely on `checks/repair_checks.py`, which mirrors the logic in Python.

## 4. Findings

### R1. Constant-only asserts should fail strict Clippy

```rust
// matrix_logs.rs:442–443
assert!(CENTRE[0] - UNIT * CLIP > -6.86);
assert!(CENTRE[0] + UNIT * CLIP < 1.0);
```

All three names are `const` (`:35`, `:36`, `:43`), so the condition is constant. Clippy's `assertions_on_constants` should reject this under `-D warnings`. The crate's convention for the same check is `const { assert!(…) }`, for example canonical `unreachable.rs:1177`. These are the only such lines among the 36 added asserts.

I did not find a log of this lint firing, so this is an expectation from the crate's convention. Native Clippy decides.

```rust
const { assert!(CENTRE[0] - UNIT * CLIP > -6.86) };
const { assert!(CENTRE[0] + UNIT * CLIP < 1.0) };
```

Line 445 (`18f64.sqrt() < CLIP && 8f64.sqrt() < CLIP`) calls `.sqrt()`, which I expect Clippy's evaluator to leave alone. If it does fire there, write `const { assert!(18.0 < CLIP * CLIP && 8.0 < CLIP * CLIP) };`.

### R2. The family ellipses still pop

`REPAIR.md` says of F17: "No pops or grid jumps". At 10_2 sentence 3, `family` is created by `always_redraw` and added at `doing_nothing.rs:369` with no fade, while `family_word` beside it fades in over 0.4 s at `:378`. So three gold ellipses appear in one frame.

This was not on the frozen review's list, so it is a remainder of F17's class rather than a miss against a named finding. The fix uses the hand-over the candidate already uses at `:444` and in `matrix_logs.rs:348`, and adds no time:

```rust
let resting_family = family_orbits(stage.arena_mut(), 1.0, gold);
// … family_word as now …
fade_in_all(stage, &[resting_family, family_word], 0.4)?;
stage.scene_mut().remove(&[resting_family]);
let family = stage.always_redraw(move |s: &mut Arena| {
    let a = value(s, stretch);
    family_orbits(s, a, gold)
});
stage.add_to_scene(family)?;
```

`family_orbits` is the closure's old body as a free function. `stretch` starts at 1.0, so the resting copy is the live one at that moment. `family` stays in `clear` at `:392`, and the resting copy has left the scene by then. Moving `family` above `family_word` in z-order changes nothing, because the two do not overlap.

Two smaller pops of the same kind are left to root's eye, since either might read as intended: the `clock` number at `matrix_logs.rs:251` beside `clock_word`'s fade, and the six journey dots at t = 0 (`doing_nothing.rs:325`, `:341`).

### R3. Four tests do not touch what they name

| Finding | Test | What it actually checks |
|---|---|---|
| F1 | `card_columns_and_brackets_clear_the_widest_entry` (`draw.rs:695`) | `card_widths` against literal widths, which holds by construction. `matrix_card` never runs |
| F2 | `the_dictionary_line_stays_in_the_frame_and_clear_of_the_third_panel` (`matrix_logs.rs:449`) | Preview02's width as a literal. Rewording the line would not fail it |
| F3 | `the_labels_under_the_turning_line_keep_apart` (`doing_nothing.rs:738`) | The same |
| F5 | `a_line_on_the_clip_box_survives_a_flows_rounding` (`draw.rs:685`) | A literal `3.0000000000000018`, not a flow |

The literals are accurate. My port of `mexp` gives exactly `3.0000000000000018`. It also keeps 292 of 292 border samples for each of the 2πJ flow and both ellipse flows, and 194 of 194 on the stretch flow's x = ±2 lines, all with the 1e-9 slack. Without the slack it keeps 0 of 292 for 2πJ and 0 of 194 for the stretch lines. So F5's fix is right, and the weakness is only that nothing in the crate shows it. This matters because the frozen review says the layout lint flags none of F3, F6 or F7, so a test is the only automatic guard.

`patches/r3-real-box-tests.patch` uses the pattern from `crates/explainer_kit/src/props.rs:552`:

- A `#[cfg(test)]` helper, `draw::probe_scene`, runs a headless scene.
- `a_real_card_sets_its_brackets_from_its_own_typeset_entries` runs the rotation and `e_card` entries through `matrix_card` and compares the group's width with `card_widths` on the widths it measures.
- `real_flows_keep_the_box_and_the_stretch_lines_when_they_come_home` uses `model::flow`.
- The F2 and F3 tests read real boxes, and the three label strings become constants that both scene and test use.

The test count rises by two. The F6 and F7 labels are TeX and stay with their literal test (`product_rule.rs:516`). `REPAIR.md`'s item 7 already sends them to frames.

### R4. A stale comment

`doing_nothing.rs:398` says "with the window −π < angle < π". Since F4, a number's window is closed at +π, as `:413–414` says. The patch changes it to "a number's window −π < angle ≤ π". This affects the comment only.

## 5. Correction to `REPAIR.md`

The "Added end hold" column (6.8, 5.1, 3.1 s) and "about 5 s longer … (a breath, then the hold)" count one breath the baseline already had:

- **Baseline:** `kit.finish` calls `hold`, so the chapter ends at voice + breath.
- **Candidate:** `kit.hold` waits for voice + breath. `music_breath` then adds the window. `window` sets `busy_until = now + seconds − BREATH`, so `finish`'s own `hold` adds nothing (`narration.rs:1302`).

The growth is therefore the window alone: 6.2, 4.5 and 2.5 s on `REPAIR.md`'s own numbers. My simulation, which also scales the line's 0.4 s tail, gives 6.25, 4.53 and 2.39 s. No code changes, and the existing stem must still be re-scored.

The fine print gets at least 13.2 s, which is 14.92 characters per second over 197 characters. It gets exactly 13.2 s when `hold`'s gap is a full breath, and slightly more under `LineGaps::Thought` when the gap is shorter.

## 6. Checked and sound

**APIs at the pin.**
- `fmn::core::rate::linear` is `fn(f64) -> f64` and fits `AnimateArgs.rate_func`. The engine writes the same expression at `fmn/src/lib.rs:211`.
- `AnimBuilder` and `AnimateArgs` are in the prelude.
- `Scene::remove` has no `must_use`.
- `bring_to_front` goes through `add_many_to_scene`, which removes and re-extends the roots, so `arrows_field` really is promoted.
- `AnimateError` and `SceneError` both convert to `fmn::Error`.
- `get_width` and `move_to` work on the scene `Stage`.
- `frames_covering`, `wait_frames`, `sentence_starts`, `phrase_marks`, `sentence_at` and `length` are public on the Kit side.
- `include_str!("../../script.tsv")` resolves correctly.
- `grid_under` is still used by `ghost`, so no import is left unused. There are no dead constants and no item after a test module.
- `3.0000000000000018` round-trips, so `excessive_precision` should not fire. `manual_range_contains` does not match the added code.

**Object lifecycle.**
- F12 is fixed: `window_word` fades once.
- No mobject in 08, 09 or 10 is in two fade lists, and the removed resting copies are never faded later.
- `resting`, `circles`, `resting_grid` and `resting_panels` all leave before their live copies arrive.
- `flow(x, 0)` is exactly I, since `mexp` does no squaring at norm 0.
- `squashed_rings` at u = 0 is the same function as the live copy.
- `clipped_grid` matches `grid_under` in style, widths, order and reach, and its identity box is the same.
- The clip of 4.4 exceeds √18, so no turned grid is cut, and the grid spans −6.63..0.23.

**Timing hooks and arithmetic.** Times are for line-length factors ×0.8, ×1.0 and ×1.25.
- 09_2 sentence 1 is "Here is the picture that matters."
- 10_3 has six sentences, and both phrases fall in sentence 3.
- "strictly between" falls in 10_4 sentence 0.
- The phrases are unique and the script is ASCII, so bytes equal characters.
- `offsets.len() != starts.len()` cannot occur under a loaded narrator, because `check_report_script` requires matching sentences.
- The sweep starts on its phrase and the stretch word lands on its phrase at all three factors.
- The sweep runs 4.80, 6.13 and 7.78 s.
- 10_3's beats end at 22.06 of 24.32, 27.45 of 30.40 and 34.19 of 38.00.
- 10_4's strict beats end 1.19, 1.66 and 2.25 s before sentence 1.
- 10_2's 1.8 s fits before sentence 1 at 1.89, 2.36 and 2.95 s.

So no cue moves, and each cue is its own request in `narration-request-plan01.json`, so no seam can overrun.

**Layout (CMR10 estimates, limit 6.86).**
- F2: 3.11..6.79.
- F3: rule 2.82..4.88 and lap 5.18..6.72, a gap of about 0.30.
- F8 ends at 6.60 and `cond_a` at 6.68.
- The fine print: textbook ±2.71, precisely ±6.37.
- The dictionary's left end at 3.11 only clears the third panel because the line arrives after the spin ends. The panel's 2.96 reach occurs mid-spin, and its rest extent is 2.56.
- Cards: rotation 3.11..5.69, `e_card` 3.61..5.59 (`e_word` ends ≈3.1), `x_card` 3.76..5.44 (`x_word` ends ≈3.27), target 4.78..6.02, pair 4.35..5.45 (↔ starts ≈5.70). The brackets' lips clear the text.
- Chapter 11, read in canonical `unreachable.rs`: `shown_num(-0.5)` really prints "−½". `a_card` is 2.97..4.33 between "A =" ending at 2.72 and "X =" starting at 4.90. `minus_i` is 3.03..4.27 and `m_card` 4.74..6.06. No line lies on its 3.2 clip box. Only `doing_nothing`, `matrix_logs` and `unreachable` call these helpers.
- `z²` spans −3.99..−3.69 and clears both the axis and the −2π tick.
- √z's `def` is 2.48..2.86, above the axis's end at 2.30.
- F13's TeX is rougher: step 4's bottom is about −0.85, the product −1.05..−1.36, and root1's top about −1.53.

**Maths.**
- `squashed_point` at u = 1 equals the baseline ring and the true orbit of 2πPJP⁻¹.
- 0.8718/π = 0.2775, which rounds to the card's 0.28.
- F4 agrees with chapter 08, and −1's principal angle is +π.
- F15's wording is right, and `target_card` is `model::J`.
- By reading, all 15 new tests should pass, including the exact `assert_eq!`s on floats, whose operands are identical.
- No answer leaks past `predict`, which is untouched. There is no new on-screen number outside the facts, and `script.tsv` is untouched.
- The fact-binding follow-up touches only `building_ruler`, `undoing_growth`, `facts.rs` and `oracle.py`. The matrix facts are unchanged in `oracle01.tsv`.

**Performance (no defect).** F14 moves chapter 09's live `picture` from 49 to 73 points per line at n = 3, and the panels from 33 to 49, under `always_redraw`. Chapter 10 already uses the same helper at 97. `GROUP_TRANSFORM_LIMIT` is not approached. Root's render will show the real cost.

## 7. For root's eyes and ears under Oliver

These are not defects, only questions source reading cannot answer:

- The 0.3 s and 0.5 s rewinds of the grid: do they read as a rewind or a flicker?
- The stretch flow's run is 1.25–2.2 s. Is that long enough to see?
- The chapter ends with about 4.5 s of music alone under the fine print. Does it feel like a held reading window or a stall?
- `phrase-timings.json` is still unbound, so these beats use the proportional estimate until it exists.

## 8. Not checked

- Compile, Clippy, tests, render, frames and listening.
- Any Oliver timing, since no `report.json` or phrase timings exist yet for 08–10.
- Real TeX boxes.
- `logarithm-repair11-12`, whose chapters I read only as they call these helpers in canonical.
- Fine sync and full independent AV, which remain open with root's native verification.

## 9. Files

- `REVIEW.md`
- `patches/`: `r1-const-asserts.patch`, `r2-family-fade.patch`, `r3-real-box-tests.patch`, `r4-window-comment.patch`. Each is against the candidate bytes, so from the repo root use `patch -p1 -i <file>`.
- `checks/`: `roundtrip.py`, `checks.py`, `prototype.py`, `build_fixes.py`, and their `.out.txt` outputs.
- `SHA256SUMS`
