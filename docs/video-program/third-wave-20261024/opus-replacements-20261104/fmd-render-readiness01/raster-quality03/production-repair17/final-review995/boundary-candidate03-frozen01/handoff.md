# Chapter 09 boundary candidate03 (CC2, recommended)

CC2 (RubyRaven), 2026-11-10. This is the single candidate I recommend for the guarded native build. It answers final995 M1 to M4, the 09_9 "1" card and L14, and it takes up CC1's boundary-review01 Medium with the leader for a, as root chose.

**There is no native acceptance of any kind.**
- It is uncompiled: no cargo check, Clippy or test run.
- It is unrendered, and none of its frames have been seen.
- No production file, build, render or Bead was touched.
- Narration (ElevenLabs Frederick Surrey) and every cue time are unchanged. The narration was not assessed.

## Files (this folder)

| File | sha256 |
|---|---|
| boundary.rs (candidate03) | `4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932` |
| candidate03.patch (base to candidate03: the one to apply) | `9cf0d3d84f17005616296699527268ae0da615f08b38b2bf0b286072fa5c3ed3` |
| candidate01-to-candidate03.patch (delta over what CC1 reviewed) | `168a0434236b385b857de7790a4d602f2247a4f4591ac2d4e22bca8263b2b057` |
| candidate03_checks.py (self-contained source-bound checks) | `c9df85a43307bef6a819977447f1bf5e163a8a201765a9a0d786eb1f47de4282` |
| handoff.md and receipt.json | in receipt.json and the cover message |

**Base:** boundary.rs `cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e`.
- That is production-repair17/complex-boundary-repair752/boundary.rs, native844 deployment files[3], the final969 source_sha256.
- The live repo file (`328d2acb…`) and build268's `dc80c541…` are different files.

**Applying the patch:**
- Apply with `git apply -p1` at the complex_analysis_visual crate root. The paths are `a/` and `b/src/chapters/boundary.rs`.
- The patch has 14 hunks: 252 lines added, 12 removed. The removals are the 11 base lines it replaces, plus the test module's closing brace, re-added after the new tests.
- Both patches apply forward to identical bytes (4b689791) and reverse to their from-files (cd1900aa, 511f4492), on fresh copies in final995-cc2-patch-check03/.

## What candidate03 changes against the base

| Finding | Change |
|---|---|
| M1: closing bar touches dz | `g\,dz\,\Big|` in ARC_TEX, SUM_TEX and SHRINK_TEX (CC1 §3: this matches the opening side's automatic thin space) |
| M2: ρ jumps onto the dashes | ρ leaves in the existing 0.2 s heads fade and returns in the existing 0.3 s bar fade at `shrink_names(SMALL_R)`, then rides the circle's right side |
| M3: the circle crosses a | a rides `A_RIDE` above the circle's line, from 138 px above the centre down to 66 px |
| CC1 Medium: a riding the circle could read as a moving centre | **the leader:** a thin line (`faded_line(Palette::ink(), 1.5, 0.6)`) on the vertical through a. Its foot stays still, 0.02 outside the hole ring that marks a; its top follows the name down. It arrives with the bar fade, is moved by the shrink's existing updater, and leaves with `proof_gone` |
| L14: shrunk circle meets the hole ring | `SHRUNK_R` 0.045 becomes 0.06: a 7.6 px drawn gap (was −1.4) |
| M4: the 09_7 and 09_9 disks run through a | `a.set_opacity(a_name, a_name_shown(r), true)` in both growth updaters: hidden for r 0.11 to 0.43, eased over 0.1 each side |
| 09_9: the line runs over the "1" card | the card's two mobs are re-rooted (`add_to_scene`) after the disk fades in, so it covers the line as the √2 card does |

Seven new tests:
- `the_closing_bars_keep_clear_of_dz`
- `the_shrinking_circle_never_crosses_a_or_rho`
- `the_shrunk_circle_keeps_apart_from_the_hole_ring`
- `the_shrinking_names_start_where_the_shrink_puts_them`
- `the_growing_disks_never_draw_through_a` (each growth about its own centre)
- `the_growing_disks_pass_under_the_cards_and_round_a` (it also binds both trackers' 1e-3 starts, centres, ends and 09_8's slide)
- `the_leader_runs_from_the_hole_up_to_a`

## Timing: CC2's 214 and CC1's 228

Both counts cover only boundary.rs (chapter 09, 09_1 to 09_9). Neither is a count for the whole video or the master. CC1's "whole film" means the whole film part of this file, the code before the test module. All of its timed calls sit inside `construct()`, which is what CC2's 214 scans, so the two scopes cover the same code. The 14-event difference is in the kinds counted:

| Kind | CC2 (214) | CC1 (228) |
|---|---|---|
| fade_in_all / fade_out_all | 84 / 42 | 84 / 42 |
| line.wait_for / wait_sentence / until | 31 / 16 / 3 | 31 / 16 / 3 |
| say / predict / kit.hold / pause / wait_until | 9 / 2 / 5 / 2 / 4 | 9 / 2 / 5 / 2 / 4 |
| stage.play / tween_linear | 8 / 8 | 8 / 8 |
| kit.music_breath / names_now | not counted | 1 / 1 |
| stage.add_updater / clear_updaters | not counted | 6 / 6 |
| **Total** | **214** | **228** |

For candidate03, both skeletons match the base:
- candidate03_checks.py checks both denominators: 214/214 and 228/228 identical, with only the heads fade and the bar fade carrying more mobs.
- CC1's own timing-skeleton01.py (`8fcb78d0…`) also gives 228/228 identical on candidate03.
- No new updater, fade or play call is added. The leader rides the existing updater and fades.

## Checks run (all exit 0)

**candidate03_checks.py:** run as `candidate03_checks.py BASE_RS CANDIDATE_RS SCRIPT_TSV`, from `/`, so it reads nothing relative to itself (CC1's L6). It pins the base, candidate03 and script.tsv (`234f7257…`, root-boundary760-source) and checks:
- **T, timing:** as above, plus every 09_1 wait phrase in script order.
- **S, source:**
  - exactly the 11 expected base lines removed, with the base tests verbatim;
  - 19 lines kept verbatim: the helpers, the shrink names and leader, SHRUNK_R, both growths' tracker starts, centres and ends, and 09_8's slide;
  - thin spaces, both opacity calls, the card order, and all 7 tests present.
- **G, geometry:** read from candidate03's own constants, on a box model with 4001 radii plus every 60 fps frame. The least gaps in native px:

  | Pair | Least gap |
  |---|---|
  | a / circle | 10.7 |
  | ρ / circle | 10.6 |
  | a / dashes | 23.3 |
  | ρ / dashes | 17.4 |
  | a / hole | 29.1 |
  | ρ / hole | 29.1 |
  | a / ρ | 52.8 |
  | leader / hole ring (round cap) | 3.4 |
  | leader / a | 3.4 |
  | leader / ρ | 51.5 |

  - The shrunk circle sits 7.6 px off the hole ring.
  - a rests 12.05 px above A_AT.
  - In 09_7 and 09_9 the line is within CLEAR of a only for r 0.125 to 0.413, all inside the hidden range, and a is shown at both ends.
- **F, faults:** 23 of 23 planted faults are caught, including an added updater and a moved music breath (CC1's extra kinds).

Other checks:
- rustfmt --check (repo rustfmt.toml, edition 2024, max_width 120): exit 0.
- Two earlier runs used the leader file (`e33be1e7…`), which differs from candidate03 only in one test doc comment (checked with diff):
  - final995_cc2_rust_test_mirror02.py: all seven tests pass in Python, the base fails the six it can, and candidate01 fails only the L14 test;
  - final995_cc2_source_checks02.py.

## CC1 boundary-review01 (`18fa8c72…`): disposition

| CC1 | Disposition in candidate03 |
|---|---|
| Medium: a riding the circle may imply a moving centre | Leader added (root's choice). The name still moves, but a line with a fixed foot at the hole now ties it to the fixed point |
| L1: ρ moves 276 px through the existing fades | Kept, as CC1 judged acceptable (ρ names the circle's radius) |
| L2: M4 hides a soon after each growth starts | Kept, as CC1 judged acceptable |
| L3: the card re-root is order-tested only, with an estimated extent | Partly: the trackers are now source-bound, but the card's extent is still estimated, for the native check |
| L4: the closing-bar test checks the source string, not the typeset gap | Unchanged; the native stills should settle it |
| L5: SUM_TEX may rescale about 1% under its width cap | Unmeasured; native |
| L6: CC2's mirrors do not run from a snapshot | candidate03_checks.py takes every path as an argument |

## Alternatives (none of them deleted)

| Name | What it is | Where |
|---|---|---|
| candidate01 | riding names, stop 0.045, no leader (what CC1 reviewed) | final995-cc2-boundary-repair/ (`511f4492…`), frozen |
| candidate02 | candidate01 plus SHRUNK_R 0.06 and the bound tests, no leader (CC1's (c)) | final995-cc2-boundary-repair02/ (`bf2a0645…`) |
| leader variant | candidate02 plus the leader (candidate03 before its doc fix) | final995-cc2-boundary-repair02-leader/ (`e33be1e7…`) |
| CC1 (a): a at A_AT, made way by radius | not built | It hides a through the MID_R hold and "and that goes to zero", the limit itself |
| CC1 (b): a fixed above the line with a background rim | not built | Rim support for a TeX label at fmn a399d626 is unchecked. At SHRUNK_R the rest frame would also put the rim on the circle's top |

**Leader risk, for root's stills:** inside the circle the leader runs from near the centre to the line, which is how a radius is drawn, and ρ has no drawn radius.
- The leader is thinner and dimmer than every other line in the close-up, its foot stops short of the hole ring, and it runs on past the line to a. A radius would stop on the line.
- Only the stills can show whether it reads as a pointer to the centre or as a radius.
- It matters most at the start of the shrink; at rest only 2.3 px shows inside the circle.

## Native checks for the guarded build (root)

1. Apply candidate03.patch to the cd1900aa source. Run cargo fmt --check, check, Clippy with -D warnings, and the chapter's tests: the 7 new ones plus the existing ones, at fmn a399d626.
2. Render chapter 09 and read 4K stills of:
   - the shrink, about f74150 to f75060: names riding, no ρ jump, the leader's look, the 7.6 px gap at rest;
   - the 09_7 growth, about f80750 to f80830;
   - the 09_9 growth and the "1" card, about f82660 to f82770.
3. Confirm the leader fades in to its own 0.6 opacity, as glow0's faded line does, and draws above the small circle's line.
4. Measure the ink bounds of ρ at 22, a at 28 and the "1" card, which the box model only estimates.

## Limits

- No native acceptance: not compiled, not test-run, not rendered, no frame seen.
- The geometry is a box model:
  - the ρ and a@28 boxes are widened from tile readings;
  - the "1" card is estimated;
  - glyph overhang and antialiasing are approximated by a 1.5 px allowance.
- The fact values (1 and 1.414) and draw.rs's `faded_line` come from root-boundary760-source. I did not check that those files are the ones bound to native844.
- Not listening, motion, fine sync or AV. The ElevenLabs Frederick Surrey narration is unchanged and was not assessed.
