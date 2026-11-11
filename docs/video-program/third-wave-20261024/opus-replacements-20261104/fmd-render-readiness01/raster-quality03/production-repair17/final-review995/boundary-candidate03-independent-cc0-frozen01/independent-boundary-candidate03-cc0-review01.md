# Chapter 09 boundary candidate03: independent CC0 review (review01)

CC0 (LavenderPike), 2026-11-10T21:58Z. This review covers CC2's recommended candidate03, inspected only after CC2's receipt was complete (written 21:47:27Z).

**Subject:**
- candidate03 boundary.rs `4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932`
- handoff.md `e43a7153…`
- receipt.json `505159e6…`

Every hash in CC2's receipt matches the files.

**Evidence:**
- Source reads of candidate03, the base, draw.rs (bound copy `9bb07b3f`), explainer_kit (`kit.rs`, `palette.rs`) and fmn at a399d626.
- Cheap Python checks of my own, each with a positive control.

**Not done:**
- The candidate is not compiled, tested or Clippy-checked.
- It is not rendered, and no frame has been seen.
- No audio heard or checked.

The narration is ElevenLabs Frederick Surrey, unchanged by candidate03, and it was not assessed. This review gives no native, motion, fine-sync, AV or release acceptance.

## Verdict

**No blocker.** candidate03 is fit to go to root's guarded native build. That build is root's step 1 in CC2's handoff: cargo fmt --check, check, Clippy -D warnings, the chapter's tests at fmn a399d626, then the 4K stills.

| Scope item | Result |
|---|---|
| Exact base lineage | PASS. The base is `cd1900aa`. Forward, delta and reverse patches give identical bytes. The live repo file does not take the patch |
| 228-event film timing | PASS. CC1's frozen skeleton gives 228/228 identical against the base and against candidate01. Two positive controls are caught |
| Source correctness | PASS from source. No compile/API problem found; the one risk I chased (`set_opacity` on a's TeX) is closed below |
| Centre association (CC1 Medium) | PASS from source. The leader's foot is fixed for every radius, it crosses the circle line at every radius, and it ends short of the hole ring and of a. Whether it reads as a pointer or as a radius is left to the stills (P1) |
| True radius/label geometry | PASS. 17/17 source-read checks hold. All five mutation controls fail exactly the checks predicted |
| Tests | PASS from source, with one Low (L1). The string assertions of the new source-reading tests hold on candidate03 and fail on the controls. The numeric tests agree with my independent geometry |

## 1. Lineage

| Role | sha256 | How checked |
|---|---|---|
| Base | `cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e` | This is the boundary.rs in production-repair17/root-boundary760-source/workspace (hashed here). CC2 and root's map also bind it as complex-boundary-repair752/boundary.rs |
| script.tsv | `234f7257…` | From the same root-boundary760-source workspace |
| candidate01 (what CC1 reviewed) | `511f4492…` | |
| leader variant | `e33be1e7…` | Differs from candidate03 only at line 2074, a test doc comment (diff) |
| candidate03 | `4b689791…` | |
| candidate03.patch | `9cf0d3d8…` | 14 hunks |
| candidate01-to-candidate03.patch | `168a0434…` | |

Results, in `lineage-roundtrip-fmt01.out`:
- base + candidate03.patch is byte-identical to candidate03;
- candidate01 + the delta patch is byte-identical to candidate03;
- candidate03 − candidate03.patch is byte-identical to the base.

**Lineage control:** a dry run of the patch on the live repo boundary.rs (`328d2acb…`, not the base) fails 8 of 14 hunks. Root must apply the patch to the cd1900aa tree, not to the live file.

**rustfmt** (edition 2024, max_width 120, via stdin, compared with the file): OK for both candidate03 and the base.

## 2. Timing (whole film part of boundary.rs, 09_1 to 09_9)

CC1's frozen `timing-skeleton01.py` (`8fcb78d0…`, unchanged) reads the film part from source.

**Results** (`timing01.out`):
- Base cd1900aa vs candidate03: 228 = 228 events, **identical**.
- candidate01 vs candidate03: 228 = 228, **identical**.

Kinds: fade_in_all 84, fade_out_all 42, wait_for 31, wait_sentence 16, until 3, say 9, predict 2, kit.hold 5, pause 2, wait_until 4, play 8, tween_linear 8, music_breath 1, names_now 1, add_updater 6, clear_updaters 6.

**Positive controls.** These are new variant files, with one edit each in the film part only.
- **T1:** the shrink's bar fade, `fade_in_all(stage, &[bar, bar_name, ca_out, rho_out, leader], 0.3)`, changed to 0.35. The skeleton first differs at event 101.
- **T2:** "Only now do we shrink the small circle" changed to "Only now we shrink the small circle". The skeleton first differs at event 98.

**CC2's own check, reproduced** (`cc2-checks-reproduced01.out`; this is CC2's script, not mine):
- `candidate03_checks.py` (`c9df85a4…`) was run from `/` with the base, candidate03 and script.tsv as arguments. It passes: 214/214 and 228/228, and 23 of 23 planted faults caught.
- The 14-event difference between CC2's 214 and CC1's 228 is only the kinds counted (music_breath, names_now, add/clear_updaters), as CC2's handoff states. Both counts cover the same code.

No updater, fade, play or wait is added. The leader and the two names ride the shrink's existing updater, the existing 0.2 s heads fade and the existing 0.3 s bar fade. All 09_1 phrase waits are unchanged.

## 3. Source correctness

Line numbers below are candidate03's.

**Closing bars (M1).** `g\,dz\,\Big|` is used in ARC_TEX (82), SUM_TEX (84) and SHRINK_TEX (87). The source change is right. The typeset gap is not measured (CC1 L4, carried).

**Shrink names (M2, M3).**
- `ca_out` (a, 20 pt) and `rho_out` (ρ, 22 pt) are made at `shrink_names(SMALL_R)` (874–875).
- The old a and ρ leave in the existing heads fade, `fade_out_all(stage, &leaving, 0.2)` (884). They come back in the bar fade (890).
- The updater moves both names by `shrink_names(r)` and sets the leader's points from `a_leader(r)` (902–903). It is cleared right after `tween_linear(radius, SHRUNK_R, 1.0)` (913–914).
- `proof_gone` (922–923) fades ca_out, rho_out and the leader out after their updater is cleared. No fade fights a running updater.

**The leader.**
- It is `faded_line(Palette::ink(), 1.5, 0.6)`. In the bound draw.rs, `faded_line` is `fill(c, 0.0).stroke(c, w, o)`.
- fmn's `fade_in` (fmn-anim fading.rs) transforms from an opacity-zero copy onto the mob as it stands. The leader therefore ends at its own 0.6, as glow0's faded line does.
- No z_index is set anywhere. The leader is added after `small` and `hole`, so it draws above both.

**Growths (M4).**
- Both growth updaters call `a.set_opacity(a_name, a_name_shown(r), true)` (1319, 1429). They are cleared right after their tweens (1325, 1435), before any later fade touches a_name.
- `VStyle::set_opacity` (fmn-library style.rs:276) is not must_use. It is in scope via the prelude, and the same pattern already runs on a TeX label in an updater at real_integral.rs:430.
- **Risk chased and closed.** set_opacity writes stroke alpha as well as fill alpha. a's style is explainer_kit's `ink_style`: fill alpha 1, stroke width 0, stroke alpha 1. fmn's default TeX style also has stroke width 0 (fmn-library tex.rs:17). So the stroke write draws no outline, and at shown = 1 the style is exactly the original.

**09_8's slide.** It now uses `c + A_NAME_FROM_CENTRE`, with `A_NAME_FROM_CENTRE = C::new(0.18, -0.2)` (184). The base used the literal `C::new(0.18, -0.2)` at both of its sites, so a's places are value-identical.

**09_9 "1" card.** `one_name` comes from `carded`, which returns [back, t]. It is re-rooted (1415) after the disk's fade-in and before `let grow = stage.add_value_tracker(1e-3)`. `add_to_scene` re-appends each mob, back first, so the text stays above its card and both lie above the disk's line. The card's extent is still estimated (CC1 L3, carried).

**Planes and radii.**
- draw.rs `Plane::new` sets height from width, so both planes are isotropic and r × unit is the true drawn radius (close_up unit 2.2222, picture unit 1.12).
- HOLE is a scene-unit radius (0.07). The tests and my geometry both treat it that way.

## 4. Geometry (my own, from source)

`geometry-cand03.py` reads all of its inputs from candidate03 itself:
- the constants and both planes;
- the line widths at their call sites (small 4, hole 3, dashes 3, leader 1.5, disks 5);
- both shrink tween times (0.7 s, 1.0 s);
- the test module's PX, CLEAR and label boxes.

It binds the bodies of `shrink_names`, `a_leader` and `a_name_shown` by exact text. It samples a 4001-radius grid plus every 60 fps frame of both tweens (4086 radii), and 20001 radii per growth.

Output: `geometry-cand03.out`, 17 checks, 0 failed. Least drawn gaps, in native 4K px, box model (CLEAR is 9.45 px):

| Pair | Least gap |
|---|---|
| a / circle line | 12.15 |
| ρ / circle line | 12.15 |
| a / dashes | 24.92 |
| ρ / dashes | 18.99 |
| a / hole ring | 30.60 |
| ρ / hole ring | 30.60 |
| a / ρ | 52.78 |
| leader foot / hole ring (cap counted) | 3.37 |
| leader top / a's ink (cap counted) | 3.37 |
| leader / ρ | 51.52 |
| leader crossing: below the line's inner edge | 2.25 |
| leader crossing: above the line's outer edge | 6.75 |

- The leader's foot is the same at every radius.
- a is always nearer the circle's line than the hole, by at least 18.45 px.
- The shrunk circle's line sits 7.65 px off the hole ring (L14 fixed; it was −1.4).
- CC2's figures are these less its 1.5 px allowance (for example 12.15 − 1.5 ≈ 10.7, and 30.60 − 1.5 = 29.1), so the two computations agree.

**The leader at the held radii** (px):

| Radius | Inside the circle | Across the line | Outside it | Total | a's centre above point a |
|---|---|---|---|---|---|
| 0.18 | 76.3 | 10.8 | 8.8 | 95.8 | 138 |
| 0.12 | 40.3 | 10.8 | 8.8 | 59.8 | 102 |
| 0.06 | 4.3 | 10.8 | 8.8 | 23.8 | 66 |

CC2 quotes 2.3 px inside at rest. That figure leaves out the leader's 2.0 px round cap.

**Growths (09_7, 09_9).**
- a@28's box comes within CLEAR of a growing line only for r in [0.1301, 0.4081]. That lies inside the hidden range [0.11, 0.43].
- While a shows at all, the least gap is 15.53 px.
- a is fully shown at 0.001, at 1 and at 1.414.

**Mutation controls** (`geometry-controls01.out`). Each is a new variant file with one edit in the film part. Every control failed exactly the checks predicted before the run:

| Control | Edit | Checks that failed |
|---|---|---|
| G1 | A_RIDE without the 0.045 gap | circle-line clearance, leader crossing |
| G2 | a_leader's text changed (12.5 → 12.0 in the foot) | only the formula binding |
| G3 | SHRUNK_R back to 0.045 | shrunk-circle gap, leader crossing |
| G4 | A_NAME_HIDDEN [0.15, 0.43] | only the hidden-band check (265 radii shown; least gap 3.45 px) |
| G6 | RHO_RIDE without the gap | circle-line clearance |

## 5. Tests

I read all seven new tests.

**String assertions.** `string-tests01.py` mirrors the include_str! assertions of `the_shrinking_names_start_where_the_shrink_puts_them` and `the_growing_disks_pass_under_the_cards_and_round_a`. On candidate03 all of them hold:
- every slice marker exists;
- `// Second limit:` (867) and `// A finite picture` (916) both lie in the film part;
- each wanted line occurs once in its slice;
- the 09_9 order holds.

The base fails 8 of these assertions (it predates the tests). The T1 variant fails exactly the bar-fade line.

**Numeric assertions.** `the_leader_runs_from_the_hole_up_to_a` asserts these margins, and my geometry gives each one positive:

| Assertion | Margin |
|---|---|
| `low − 0.0075 > HOLE + 0.015` | 3.37 px |
| `high + 0.0075 <` a's lower edge | 3.37 px |
| `low < r·unit − 0.02` | 2.25 px |
| `high > r·unit + 0.02` | 6.75 px |
| ρ's left edge − cap > CLEAR | 51.5 px against 9.45 |

The growth test's hidden-band condition is the same as my `max(gap_out, gap_in) ≤ CLEAR`. None of this is a test run.

**L1 (Low, test binding).** The Rust tests do not bind two of the leader's lifetimes:
- that the leader is in `proof_gone` (922–923);
- that the shrink updater moves it (`set_points(leader, …)`, 903).

The new test's slice ends at `// A finite picture` (916), before proof_gone, and it has no wanted line for the leader's set_points. CC2's Python faults "leader left after the proof" and "leader not moved" catch both, but a compiled test run would not.

The leader test also takes the widths 1.5, 4 and 3 as literals. They agree with the call sites today.

This is not a blocker, because candidate03 is correct as written. If root wants the tests to carry it, the fix is two film-wide `matches(...).count() == 1` lines. Otherwise the native stills settle it: no leader in 09_2, and a leader that follows the shrink.

## 6. For root's stills (perceptual, not blockers)

**P1, leader as radius.**
- At the start of the shrink (r 0.18), 76 px of the 96 px leader lies inside the circle, from 26 px off the centre (cap counted) to the line. ρ has no drawn radius.
- The leader is thinner (1.5) and dimmer (0.6) than every other close-up line, and it runs on 8.8 px past the line to a.
- Only the stills can show whether it reads as a pointer to the fixed hole or as a radius.
- At rest only 4.3 px shows inside the circle.

**P2, small leader gaps.** The leader stops 3.4 px short of the hole ring and 3.4 px short of a, at 4K. That is about 1.7 px at 1080p. The stills should confirm that the gaps read as gaps and that the foot does not touch the ring.

**Carried from CC1:**
- L3: the "1" card's extent is estimated;
- L4: the closing-bar test checks the string, not the typeset gap;
- L5: SUM_TEX may rescale about 1% under its width cap.

## 7. Limits

- **Box model.** The label ink boxes are the test module's, partly widened from tile readings. They are not glyph bounds measured at fmn a399d626. Ink bounds of ρ@22, a@20, a@28 and the "1" card are for the native check.
- **Fact binding.** The facts 1 and 1.414 are read as CC2 and the tests take them. I did not prove which facts.rs is bound to native844. It does not change the M4 result: the hidden band, 0.11 to 0.43, lies far from either end value.
- **Native work.** Uncompiled, untested, unrendered, no frame seen.
- **Audio.** No audio heard or checked. Narration is ElevenLabs Frederick Surrey, unchanged, and not assessed.
- **No acceptance.** No native, motion, fine-sync, AV or release acceptance is given or implied.

## Files (this folder)

**Inputs:** copies of the base, candidate01, the leader variant, the live file, candidate03, both patches, CC2's checks, handoff, receipt and script.tsv.

**My scripts and outputs:**
- `geometry-cand03.py` and its `.out`
- `geometry-controls01.py` and its `.out`
- `timing-controls01.py`, with `timing01.out`
- `string-tests01.py` and its `.out`
- `lineage-roundtrip-fmt01.out`
- `cc2-checks-reproduced01.out`

**Variant files:** `controls/` (T1, T2, G1–G4, G6) and `roundtrip/`.

The hashes are in `receipt.json`. Output paths were stripped of their folder prefix.

The controls/ and inputs/ copies of boundary.rs carry the film's own source comments verbatim. Two inputs are verbatim copies that contain words outside this review's own vocabulary:
- `inputs/script-234f7257.tsv` has narration text at line 72;
- `inputs/handoff.md` has CC2's limits line 149.

Neither is my wording.
