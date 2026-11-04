# Complex chapters 11–15: native picture repair

WhiteSwan for BlueSalmon, Mail 1655, bead `.39`. Date: 2026-11-04. Narration stays ElevenLabs Oliver.

**Status: a proposal. Nothing here is installed, compiled, rendered, voiced or independently accepted.** I reviewed these chapters and then repaired my own findings, so this needs someone else's read as well as a build. No repaired picture has been seen by anyone.

## What this repairs

The review is `../complex-preview01-review-11-15/REVIEW.md` (sha256 `76872154e4fd9915…`), which stays frozen.

| Severity | Found | Changed | Left |
|---|---|---|---|
| High | 5 | 5 | 0 |
| Medium | 30 | 30 | 0 |
| Low | 31 | 21, two of them partly | 10 |

One further finding came up during the repair, 11-Z below. **No cue text changed**: `script.tsv` stays `2968b0fd76540cbc…`, and I found no script correction to propose. No depth, label, mapping or motivating example was removed.

## How to apply

The patches measure from canonical and already contain everything in the review's patches. **Apply these instead of the review's, not on top of them.** `baseline/` equalled canonical in all 11 files when the replay last ran.

1. `shared-facts.patch`
2. `11-counting.patch`, `12-residues.patch`, `13-real-integral.patch`, `14-continuing.patch`, `15-closing.patch`, `16-storyboard.patch`, in any order
3. `shared-registry.patch`
4. **One** `chain_arrows` fix, see below

| Patch | Changes | sha256 |
|---|---|---|
| `shared-facts.patch` | +49 −2, `oracle.py`, `facts.rs`, `model.rs` | `eca530d87d2cb16b6974c6ff10f3fd9bc33cf12f18340f7fee3bfa549e165b6c` |
| `11-counting.patch` | +605 −37 | `d0d16b019ea9862cf9c705b3d5d2642b28c779bf9ecaa502bf0c0a1e98d69450` |
| `12-residues.patch` | +75 −12 | `77d0331a3e67840befb5e6909fa284775b82cc3c95395575a57fb1b8b4caace8` |
| `13-real-integral.patch` | +204 −30 | `62b6199dd4dda7b881076bc0fe6df74bddd7b700afd707f70cb8cf8dd8e11156` |
| `14-continuing.patch` | +248 −27 | `91c63b2180ede81a2e3897ed99704ee13c60a7d0af54b9750803683eb5ce57a6` |
| `15-closing.patch` | +69 −15 | `66704997aee3a734d4bbcd2fe9e37ab7add2711b00f9811630de855a7cdd79ae` |
| `16-storyboard.patch` | +19 −19, picture column only | `4baab8f806bfd8b8fcefc8317aa81e074a971a21e62d757fca6e8e34894dcd15` |
| `shared-registry.patch` | +26, `mod.rs` | `8d3e3d22af4a4c30f3318064377d85d20df721d683c25af684dd7420977daf83` |
| `shared-chain-arrows-fallback.patch` | +3 −2, `draw.rs` | `2ecb308676b0d1f1983a79a04bee2cac35ef62480937a1e726ef23bd07cb09e1` |

`SHA256SUMS` lists every file here.

## Shared changes

**`chain_arrows`: apply once.** LavenderPike confirmed (reply to Mail 1747) that `complex-picture-repair-06-10/patches/00-draw-helpers.patch` owns the `.buff(0.0)` line. Use that one if chapters 06–10 go in. My `shared-chain-arrows-fallback.patch` is only for applying 11–15 without it, and it is byte-identical to the review's `shared-chain-arrows.patch`. Applying both fails or does nothing.

- Why it matters: `Arrow::new` trims 0.25 scene units from each end by default (fmn pin a399d626, `crates/fmn-library/src/line.rs`), so 12_1's chains had 38 backward arrows.
- Callers, at canonical line numbers: `residues.rs:137` and `loop_sums.rs:315, 368, 466, 481`.
- Chapter 12 depends on it. Chapter 08's chains change too, and I have not viewed those. LavenderPike has not viewed chapter 12.

**Facts.** The oracle goes from 120 rows to 130 and no existing row changes. Six rows come from the review, four are new: `argument.rational.zero` 0.45, `.pole` 1.35, `.radii` 0.8,1.7, and `argument.several.turns` −1. `derivative_bound` also changes for `ZeroDoublePole`: with −1 inside the loop the gap went negative and the bound was refused, so it now takes the absolute value. The bound is 21.296, which the true maximum of |f′| equals at z = −1.6. One step's reach is 0.0523 against a least |f| of 0.0888. The other `ZeroDoublePole` calls give the same answer as before.

**Registry.** Three new TEX and nine new TEXT entries beyond the review's. `counting.rs` reuses `the_bump::RADIUS` because a second "radius " would have failed `no_supplement_is_empty_or_duplicated`.

**Storyboard.** 19 picture cells now say what is drawn; ids and words were asserted unchanged. `check_video` reads the words only.

## For root: a build failure between the other two lanes

Stacking all three lanes applies cleanly as text, but **`no_supplement_is_empty_or_duplicated` will fail** on `\sin z`, registered twice in TEX:

- `complex-picture-repair-02-05/patches/chapters-02-05.patch`, through `gallery::PANEL_NAMES_TEX`
- `complex-picture-repair-06-10/patches/12-mod-lists-10.patch` line 20, `rigidity::SIN_Z_TEX`

Dropping the `rigidity::SIN_Z_TEX` registry line should clear it, since the value stays registered and the kit needs it only once. Neither side is mine, so I changed nothing. No duplicate involves this lane.

## What changed, by finding

The review's own patches carried A1, 11-B, E, F, H, K, L, M; 12-B, F, G, J; 13-D, F, J, L, O, P, Q; 14-C, D, G, J, L, M; 15-A, D, E, F, G. That includes Mail 1655's named case: in 11_4 the true output runs up the window and leaves it, the readout climbs to a finite 30.0 at 0.03 from the pole, and no value is ever drawn at the pole. These are new in this round:

| Finding | Now |
|---|---|
| 11-A | "Several of them" walks \|z\| = 1.6 on (z − 1)/(z + 1)², zero at 1 and double pole at −1, counting to −1. "+1" and "−2" land on the marks and "+1 − 2 = −1" on "each counted". Names arrive on their sentence. Conditions sit on the right, with the loop and marks left to check against |
| 11-C | Each 11_1 walk takes the room its sentence leaves, 1.8–2.6 s |
| 11-D | True minus in "turns: −1" |
| 11-G | The question's ring sits inside the output plane, clear of the border and axes |
| 11-I | −3z named beside its circle |
| 11-N | "zero at 0.45", "pole at 1.35" and the loop's radius in rows above the picture, from facts; red "radius 1.35" in 11_4 |
| 11-O | The same loop grows through the pole on "Grow the circle"; it no longer blinks and redraws |
| 11-P | An arrow on each inset circle, tested against the path's own direction |
| 11-Q | `step` defined in the code's first line |
| 11-S | "reach covers 0" above the window; "reach L × step" off the axis (lint `11_8 +22.4`) |
| 11-V | On "cannot move" the reach disk is carried once round the output circle with its distance from 0 drawn. 0 is never inside it, and the next sample always is |
| 11-Z (new) | The three condition cards land on the words naming them. They all used to arrive within about 2.3 s of a sentence about 10 s long |
| 12-E | `near` at the sentence, `simple` on "the residue is" |
| 12-K | Loops, arrows and corridors laid down piece by piece, with direction arrows on every loop; the big loop's arrow dims with it |
| 13-B | The calculus line dims on "a second way" |
| 13-C | A walker draws the segment, then the half circle, as the voice walks them |
| 13-E | "the bump's size along the half circle" |
| 13-G | R reads to 2 places during the sweep, so a paused frame shows a true pair; it settles on the fact |
| 13-H | The plane rescales while R grows: the loop keeps its size, i and −i close in at the true ratio, and a note says so |
| 13-I | Degree labels 24 pt, ink |
| 13-K | `h = 1 + z⁴: Res = 1/h′(p) = 1/(4p³)`, in 12_2's letters. I did not use the `(1+z^4)'\Big\|_p` form I had first planned: the prime on a bracket has no precedent here |
| 13-M | `Im p = √2/2` on "each has imaginary part" |
| 13-N | The sum's value set as maths, so its i matches |
| 14-A (part) | Poles named i and −i. Axes and a "1" label not added; the row "the recipe at 0 works here" names the disk |
| 14-B | A small plane on "Inside the unit circle", with dots at 1/2 inside and 1.2 outside as each chart starts. Baselines wait for their sentence |
| 14-E | "small disk" and "1" on the plane; "each disk stops at the pole at 1" for the chain |
| 14-H | The path drawn through the chain's centres |
| 14-K | 1 and −1 named on the √z plane |
| 14-N | "Re z = 0" on the cover's edge |
| 15-C | The quotient carries its limit |
| 15-H (part) | Holds of 1.0 s after 15_1 and 1.2 s after 15_2, so chapter 15 grows by 2.2 s. "though not always everywhere" still is not on the card |
| Lint `15_1 +0.8` | Formula moved down 0.04 |

12 new test functions: 6 in chapter 11, 1 in 12, 3 in 13 and 2 in 14. The coarse-loop test in chapter 11 is also extended, and `facts.rs` asserts the new rows.

## Not changed

| Finding | Why |
|---|---|
| 11-J | Legible and on the cue. Card spacing is better judged at 4K |
| 11-R | Code size: judge at 4K, as the review said |
| 11-U | "8 samples say 0 turns" stays beside the 64-sample picture. It is true of the coarse loop. Dimming it on "take steps" would be a one-liner if root wants it |
| 11-W | The marker on the card edge needs a native look before moving anything |
| 11-X | Row banding: `complex-picture-repair-02-05/patches/shared-P1-crisp-strips.patch` addresses it. Not duplicated |
| 11-Y | Resolved from source: `fade_out_all(&last, 0.5)` runs to the chapter's end and the samples simply stop 0.166 s early. Still check the join |
| 12-H | Roman f′/f in a text row. Cosmetic, and setting it as maths adds a glyph risk |
| 14-F | "agree everywhere" has no picture. That needs a decision about what to draw |
| 14-I | "..." against "⋯". Cosmetic |
| 15-B | The bump and plane's two scales need a change of composition, not a label |

## What to check first at the build

- `\lim` is new to this crate's registry. `epsilon_delta/old_objection.rs` uses `\lim_{h \to 0}` with the same engine.
- The direction arrows are short: 0.246, 0.273 and 0.386 scene units. `key_dimensions` caps the head to the length, so they should build, but they may read as mostly head. Judge at 4K.
- 13_2's path starts as two equal points, as `trail` and `ztrail` already do.
- Text widths are estimates: `RES_TEX` about 5.1 wide on the column at x = 3.9, `ZOOM_NOTE` about 6.0 from x = −6.0 to 0.
- Risks carried from the review: `VALUE_RESULT_TEX` starting with "=", `VALUE_TEX` ending in "=", Painter from `None` to Gamma, the glyph probes, Clippy.

Pictures I would look at first: 11_2's walk, 11_3's growth, 11_4's approach, 11_8's carried disk, 12_2's arrows, 13_2's walk, 13_3's labels at R = 10, where i sits 0.21 above the segment, and 14_2's small plane.

## Checks

| Check | Result |
|---|---|
| `checks/replay_patches.py` | Passes. Patches rebuild `candidate/` from `baseline/`, and also apply on top of every 02–05 and 06–10 patch. Oracle exits 0 with 130 rows and no canonical row changed. `rustfmt --check` exits 0 on all 9 Rust files, which proves parsing and format only |
| `checks/registry_check.py` | Passes: no empty or duplicate values in this lane. Caught the "radius " duplicate before delivery. New characters: none |
| `checks/cue_phrases.py` | Passes: 111 spoken anchors resolve against the unchanged script; 3 through variables read by eye |
| `checks/dead_code.py` | Passes after one real fix: `contour()` had become test-only, which `-D warnings` would refuse. Name-based, so **not a Clippy pass** |
| `checks/geometry.py` | Passes. Numeric mirror of the new geometry and of the new tests' tolerances. **A mirror, not fmn** |
| `checks/beat_times.py` | Runs. Scratch timing only. Worst cases: 11_1 sentence 4 is 0.75 s late, 11_2 sentence 1 is 0.71 s, 14_3 sentence 6 is 0.49 s, 13_2 sentence 1 is 0.40 s. All need rechecking on Oliver |

## Not done, and still open

- **Nothing is compiled.** No `cargo` build, test, Clippy, `--lint` or `facts-check` ran, so the Rust rows have not been compared with the oracle by any tool. Root integrates and runs guarded native validation.
- **Nothing is rendered**, and no repaired picture has been viewed.
- **No synthesis, listening or fine sync.** Timing above is scratch, and every phrase moment is an estimate until the Oliver takes are aligned.
- **No independent review or final AV review.** These patches are author-checked only.
