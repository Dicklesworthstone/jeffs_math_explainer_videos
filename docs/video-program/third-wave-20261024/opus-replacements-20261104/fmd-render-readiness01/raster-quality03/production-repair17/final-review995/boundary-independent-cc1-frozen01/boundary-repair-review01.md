# CC2 boundary candidate: independent review (CC1, 2026-11-10)

Scope: the immutable interim snapshot `final-review995/boundary-repair-review-snapshot01`. This is a source review only. I did not compile, run tests, run Clippy, render or decode anything. **No rendered acceptance is claimed.** I wrote nothing to production; every file I wrote is in my scratch, `.../scratchpad/boundary-review/`.

**Independence:** the base (cd1900aa) is the 752 boundary repair, which an earlier session of mine wrote. This review is independent of CC2's candidate, but not of the base.

## Inputs (sha256)

| Snapshot file | sha256 |
|---|---|
| base-boundary.rs | cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e |
| candidate-boundary.rs | 511f4492cdbe7b632710f4121cee2da032baeee670f2e83531c4272f0c86b6a7 |
| candidate.patch | 984a33a438e19dedf1009840b52ec29ee730d36377e2d9210f694b174349ac70 |
| geometry-mirror.py | 51087623a1ddb863b95fe2cc80aaed566a1665b203b1da7208c974851f3e1a41 |
| rust-test-mirror.py | ef375d8dc259928b9d89ff27a159bd96d635090e15d11f2d479fec84acd357f2 |
| source-checks.py | 3edc43176b33481822e6d451f31ea943f201afcbfa2b54f272dcec0d75839be8 |
| receipt.json | 49d226267b973f535a73e5741c16629fc90a785cf911d107373a73c9ecad399a |

The base is the chapter 09 source behind final969/native844 and the master993 stills reviewed in final995. Build268's `source-copy.json` binds an older boundary.rs, dc80c541.

## 1. Patch round-trip (scratch)

- I applied `candidate.patch` to a copy of the base at `roundtrip01/src/chapters/boundary.rs`. The result is 511f4492…, byte-identical to `candidate-boundary.rs`.
- The patch also reverse-applies cleanly.
- **PASS.**

## 2. What the diff does

| Defect (final995) | Change |
|---|---|
| M1: the closing `\Big|` touches the z of dz (sheet 692) | `g\,dz\Big|` → `g\,dz\,\Big|` in `ARC_TEX`, `SUM_TEX` and `SHRINK_TEX` |
| M2: ρ jumps from `RHO_AT` onto the dashes as the shrink begins (sheet 680, f74163–74167) | `ca` and `rho` fade out with the small circle's heads (the existing 0.2 s fade). New `ca_out` and `rho_out` are made at `shrink_names(SMALL_R)` and fade in with the bar (the existing 0.3 s fade). The updater moves them by `shrink_names(r)`. This replaces `move_to(rho, (r+0.06, −0.12))`. |
| M3: the shrinking circle crosses a (sheet 692, f74689–74717) | The same `shrink_names(r)`: a rides `A_RIDE` = 0.111 scene above the circle's line centre, and ρ rides `RHO_RIDE` = 0.121 scene to its right. |
| M4: the 09_7 disk draws through a (sheet 766, f80757–80822) | Both growth updaters (09_7 and 09_9) gain `a.set_opacity(a_name, a_name_shown(r), true)`. a is hidden for r ∈ [0.11, 0.43] and eases over 0.1 either side. |
| 09_9: the disk runs through a (f82702) and over the "1" card (f82742–82761) (sheet 789) | The same opacity rule, plus `for m in one_name { stage.add_to_scene(m)?; }` after the disk fades in. That re-roots the card above the disk's line. |

The 09_8 slide's literal `C::new(0.18, -0.2)` becomes `A_NAME_FROM_CENTRE`, which has the same value, so there is no change there. `proof_gone` lists `ca_out` and `rho_out` in place of `ca` and `rho`. The patch adds five tests and one helper, `reach`.

**Use-after-change scan (film part):**
- `ca` appears only at 598 (made), 611 (faded in) and 861 (leaving).
- Outside comments, `rho` appears only at 645, 646 and 861.
- `small_arrows` is moved into `leaving` and never used again.
- `ca_out` and `rho_out` are Copy `Mob`s, captured by the `move` updater and used afterwards in `proof_gone`.
- `cp` is `close_up()` (line 589), so the `close_up().unit()` inside `shrink_names` matches the plane the updater draws in.

**Nothing else in the new label paths:**
- `g_name`, `bound`, `inside`, `every` and `bar_name` stand at `GAP_X` 1.15, outside the close-up (x 2.4–6.4).
- `bar_points` is also at `GAP_X`.
- The loop's heads are at 45° + k·90° on `LOOP_R`, more than 0.5 scene away.
- The small circle's heads have faded out before the new names appear.

## 3. TeX spacing (M1)

Source checked in fmd-math (franken_markdown 126a530):
- A plain `\Big` delimiter is `SizedDelim{class: None}` and lays out as Ord (`commands.rs:111-116`, `:695`).
- `\,` is `SpaceKind::Thin`, 3 mu (`parse.rs:361`, `node.rs:236`).

Consequences:
- Before the patch, `dz\Big|` is Ord–Ord, so it gets no automatic space. The opening `\Big|\oint` / `\Big|\int` is Ord–Op, so it gets an automatic thin space.
- The candidate's `\,` therefore makes the closing side match the opening side exactly. **Correct and symmetric.**
- At size 26, 3 mu ≈ 0.043 scene ≈ 12 native px at 4K. I computed this with CMR10 em ≈ size × 0.01; it is not measured.
- The right side `\Big| \le` is Ord–Rel and already gets a thick space, unchanged.

Uncertainty: `proof_tex` caps widths at 5.1 with `set_width`. If `SUM_TEX` was already at the cap, the added 3 mu shrinks it by about 1%. That is not a defect, but it is not measured either.

## 4. Timing invariance

- `timing-skeleton01.py` (8fcb78d0) extracts every timed or narration-binding call in the whole film part in source order. That covers `say`, `wait_for`, `wait_sentence`, `until`, `wait_until`, `hold`, `music_breath`, `pause`, `stage.wait`, fades, `play`, `tween_linear`, `predict`, `names_now`, and updater add/clear, with their run-time literals.
- Base vs candidate: **228 events each, identical** (`timing-skeleton01.out`, 70dba822).
- Positive control: in a copy (`control-timing-variant01.rs.txt`, 78ff4adb) I changed the first film `fade_in_all(…, 0.3)` in the shrink to 0.35. The script caught it at event 101 (`timing-skeleton01-control.out`, 75ebe238).
- The new names ride the existing fades. `add_to_scene` and `set_opacity` take no scene time.
- Cue words are untouched. The narration (ElevenLabs Frederick Surrey) and every schedule stay unchanged.

## 5. Native API plausibility (fmn a399d626, source only)

- `Stage::add_to_scene` (`fmn-mobject/src/stage.rs:811-836`) removes the mob, appends it, then stable-sorts by `z_index`. Nothing in boundary.rs or draw.rs sets `z_index`, so re-adding the two "1"-card mobs puts them above everything added before. That includes the 09_9 disk, `reach1` and the √2 card; neither of the latter two overlaps the "1" card. `carded` returns `[Mob; 2]` (Copy), so `one_name` is still usable at `all.extend(one_name)`.
- `set_opacity(mob, o, recurse)` (`fmn-library/src/style.rs:276`) sets fill and stroke alpha. It already appears in an updater at `real_integral.rs:430`.
  - `kit.tex` uses ink style (fill alpha 1, stroke width 0), and nothing changes a_name's opacity before 09_7.
  - So `a_name_shown` = 1 at the end of each growth (r = 1 and √2) restores the label exactly.
- `kit.tex` makes a detached mob that the fade roots. The base already relies on this: `shrink_bound` and `zero` are made before an unrelated fade-out and faded in after it. So `ca_out` and `rho_out` do not pop in during the 0.2 s fade-out.
- The updaters are installed with `true`, so they run immediately. At r = `SMALL_R`, `shrink_names(r)` equals the creation place, so nothing jumps when the updater is installed.

## 6. Test plausibility (not run)

All helpers exist in the candidate test module: `num` (1529), `PX` (1745), `A_HALF` (1746), `CLEAR` (1748), `apart` (1771), `to_segment` (1775), `gap` (1800) and `label_box` (1812, corners in cyclic order). The parent items are reached through `use super::*` (1527). I mirrored each test by hand against the source:

| Test | Reading | Expected |
|---|---|---|
| `the_closing_bars_keep_clear_of_dz` | `rsplit_once(r"\Big|")`; checks `before` ends `g\,dz\,` and `after` starts ` \le `. All three strings match (lines 82, 84, 87). | pass. Note: this is a source-string check, not a typeset gap, so the name says more than the test checks. |
| `the_shrinking_circle_never_crosses_a_or_rho` | 1001 radii from `SHRUNK_R` to `SMALL_R`: the circle, the dashes, the hole ring, a–ρ, and a ending within 4 px of `A_AT`. | pass. In my mirror the least margins over `CLEAR` are +0.010 (a vs circle, ρ widened vs circle) and a's end is 3.1 px from `A_AT`. |
| `the_shrinking_names_start_where_the_shrink_puts_them` | Slices `include_str!("boundary.rs")` between the first "// Second limit:" (851) and "// A finite picture" (892). The marker occurs twice, but the second copy is the test literal, so `nth(1)` is the film slice. Four strings once each, and no `r + 0.06`. | pass |
| `the_growing_disks_never_draw_through_a` | 4001 radii to √2; hidden whenever the width-5 line is within `CLEAR` of a's widened box; shown = 1 at 1e-3, 1 and √2. | pass. My mirror's near band is [0.130, 0.408] ⊂ [0.11, 0.43]. |
| `the_growing_disks_pass_under_the_cards_and_round_a` | Source order: disk fade-in < re-root < `grow` tracker; two `set_opacity` lines. | pass, but this is **order only, no geometry** (finding L3). |

The ink half-sizes are those the base test uses (`A_HALF` 12×12.5 px). ρ is widened to 15 px from a measured 12.5, and a@28 to 17.5×19 px. These are reasonable margins, but they come from the stills, not from native glyph bounds.

CC2's `geometry-mirror.py`, `rust-test-mirror.py` and `source-checks.py` resolve paths against CC2's own scratch (`HERE/final995-cc2-boundary-repair/…`). They **do not run from the snapshot**, and I did not run them.

## 7. Label paths: independent mirror

`label-paths01.py` (a6d2041b) produced `label-paths01.out` (5fc2a848). It steps 1e-4 in r over the whole shrink and both growths. Units are scene units; px means 4K native px at 270 per unit.

**09_1 shrink, `SMALL_R` 0.18 → `MID_R` 0.12 → `SHRUNK_R` 0.045:**

| Pair | Base | Candidate (ρ measured / widened) |
|---|---|---|
| a vs small circle | **crosses** for r ∈ [0.060, 0.122] (touch at `MID_R` hold, crossing in the second tween) | +0.045 (12.1 px) |
| a vs dashes | +0.401 | +0.092 (24.9 px) at `SMALL_R` |
| a vs hole ring | +0.069 | +0.080 |
| ρ vs dashes | **crosses** at r = 0.18 (the M2 jump) | +0.080 / +0.070 |
| ρ vs small circle | +0.102 / +0.094 | +0.054 / +0.045 |
| ρ vs hole ring | +0.18 | +0.089 / +0.080 |
| a vs ρ | +0.37 | +0.156 / +0.149 |

**09_7 and 09_9 growths** (a@28 at `A_NAME_FROM_CENTRE` from each centre):
- The line comes within `CLEAR` of a's ink for r ∈ [0.135, 0.403] (scaled box) or [0.130, 0.408] (widened box), and touches it for r ∈ [0.161, 0.377].
- `HIDDEN` [0.11, 0.43] covers both bands. Shown is 0 everywhere inside, and 1 at 1e-3, 1 and √2.
- The fade-out runs over r 0.01–0.11, which is 10% of the 09_7 growth and 7% of 09_9's.
- The "1" card (size **estimated** at 0.61 × 0.50 scene) spans r ≈ 0.40–0.88 from 1, so the 09_9 line passes under it for r ≈ 0.38–0.91. The card stands 0.152 from a's widened box.

Geometrically, the candidate clears M2, M3, M4 and the 09_9 crossings at every radius I checked.

## 8. Findings

**Medium (meaning; a judgment call for root): the shrinking 'a' rides the circle instead of naming its fixed centre.**

- In `SHRINK_TEX`/`CIRCLES_TEX`, a is the fixed centre of |z−a| = ρ. It is the point the hole ring marks, and the only thing that does not move as ρ → 0.
- In the candidate, `ca_out` sits 0.111 scene (30 px) above the circle's top at every radius. Its distance from the centre runs from 0.511 scene (138 px) at `SMALL_R` to 0.211 (57 px) at `SHRUNK_R`. The ratio of distance from the centre to distance from the line falls from 4.6 to 1.9.
- It reappears 84 px above where base a stood, in the corridor between the circle and the dashes, then descends with the circle.
- With ρ also riding the circle on the right, both names read as attributes of the circle. a could read as its top point, a + iρ, or simply as a label that moves. A viewer may take this as a moving centre during the very limit whose point is that the centre stays put.
- It also departs from the patch's own M4 rule, where a "keeps its place and makes way".
- It ends 3.1 px from `A_AT`, so the final frames are right; only the path during the limit carries the risk.

Every alternative costs something. No time is added in any of them.

- **(a) Keep a at `A_AT` and make way by radius**, as in M4. Its near band is r ∈ [0.044, 0.137], which takes in `SHRUNK_R`. So a is hidden through the hold at `MID_R` and through "and that goes to zero", the limit itself. It could fade back in with the existing 0.3 s fade of `zero` at "But the big circle's integral never changed". There it stands 0.034 scene (9 px) from the line: 0.001 under `CLEAR`. Root would either relax that band or accept it.
- **(b) Keep a fixed and above the line, with a background rim**, as the "1" card does in 09_9. The circle then passes under it. This depends on rim/stroke-order support for a TeX label, which I have not checked at fmn a399d626.
- **(c) Keep CC2's ride.** Acceptable if root's look at the stills of the candidate shows that it reads as the centre.

My preference, without rendered evidence: (b), then (c), with (a) last because it hides a during the limit. A narrow 4K stills check of the 09_1 shrink should decide.

**Low:**
- **L1:** ρ moves 276 px from `RHO_AT` to the circle's right side through the existing fades. That is acceptable, and arguably better: ρ is the circle's radius, so riding the circle suits it.
- **L2:** M4 hides a almost as soon as each growth starts (r 0.01–0.11 fade). In 09_7 this coincides with "the largest disk around the centre". The centre dot stays visible and the narration does not say "a", so the blink is acceptable. a is fully back by r = 0.53 and at both ends.
- **L3:** the 09_9 "1"-card re-root is covered only by a source-order test. The card's extent is my estimate, not a measurement. The z-order effect depends on `add_to_scene` re-append semantics and on no z_index being set, both of which hold in the code I checked.
- **L4:** `the_closing_bars_keep_clear_of_dz` checks the source string, not the typeset gap. The TeX semantics are verified above.
- **L5:** `SUM_TEX` may shrink about 1% under the 5.1 width cap. Unmeasured.
- **L6 (reproducibility):** CC2's three mirrors do not run from the snapshot because of scratch-relative paths.

**No concrete regression found** in timing, cue binding, API use, ownership/moves, test slicing, the 09_8 slide, or z-order beyond the intended card.

## 9. Not established / uncertainty

- No native frames exist for the candidate, so every gap above is a box model:
  - the a box is measured;
  - the ρ and a@28 boxes are measured and then widened or scaled;
  - the "1" card is estimated.
- Glyph overhangs and the stroke anti-aliasing at 4K are not modelled.
- The finding that 3 mu ≈ 12 px is computed, not measured.
- Whether the riding 'a' reads as the centre is a perceptual judgment that only stills of the candidate can settle.

## 10. Native checks not run (root's)

- `cargo build` and the five new tests, plus the existing chapter tests, at fmn a399d626.
- Strict Clippy (`-D warnings`) and `rustfmt --check` on the candidate.
- A native render of three windows, read as 4K stills sheets:
  - the 09_1 shrink (frames around f74160–74720);
  - the 09_7 growth (around f80750–80830);
  - the 09_9 growth (around f82700–82770).
- Measured ink bounds for ρ@22, a@28 and the "1" card.

## Evidence

- Final995 overview sheets read: 680, 692, 766 and 789 (chapter 09 is sheets 508–800, frames 67065–83568). Root's adjudication is `independent-findings03/root-adjudication01.json` (01f21c06).
- Scratch files, all under `/private/tmp/claude-501/-Users-jemanuel-projects-jeffs-math-explainer-videos/b0a72b9b-6c8d-4794-942e-44d50a9d8626/scratchpad/boundary-review/`:

| File | sha256 |
|---|---|
| roundtrip01/src/chapters/boundary.rs | 511f4492… |
| candidate.patch (copy) | 984a33a4… |
| timing-skeleton01.py | 8fcb78d0341e9e5bb0f94705c7b50c78467e07bc45631e23928ac5e57b5bd903 |
| timing-skeleton01.out | 70dba822d18cc941df45a491dcc1734e187b2e884be12b8bf088ceaaf67627b7 |
| control-timing-variant01.rs.txt | 78ff4adbfff7a2975a0a26826a56b81c207508c0cd7447694000b3c8fd1bad78 |
| timing-skeleton01-control.out | 75ebe23866dd0b3392dcbd53e1571706cfd04da184b76ed910dc02852af43799 |
| label-paths01.py | a6d2041ba562e8e822b0552b756896d586161f709d002c27da344774f41abb10 |
| label-paths01.out | 5fc2a8482a023dd2d0e684c01a68689fd30673aada0482eba525a0022445e1c4 |
