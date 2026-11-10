# Handoff 753: complex chapter 09 repair 752 for review 747 B1 (source only)

WhiteSwan (CC3), 2026-11-10. This is a narrow source repair of the 09_1 close-up in `boundary.rs`, made for B1 of
`cc2-boundary-final-review747.md`. Nothing was compiled, rendered, synthesised or launched. There was no SSH, Cargo,
Beads change, canonical edit, gate change, commit or deletion. **The fix is not visually validated: no picture of the
repaired source exists yet.**

## Files

| File | sha256 |
|---|---|
| Base: `root-boundary552-source/workspace/explainer_corpus/complex_analysis_visual/src/chapters/boundary.rs` (unchanged; byte-equal to root's independent copy of final553) | `bda7d931498b23bd3203cd0c6106de69dc0a8c5e160e738d698376322ff39e53` |
| `complex-boundary-repair752/boundary.rs` (the repair, 1859 lines) | `cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e` |
| `complex-boundary-repair752/boundary-553-to-752.diff` (`git diff --no-index`, base to repair, 221 lines added, 12 removed) | `3633598fcefa861d2051386148367917cd50cefe5a0dcf28a4baee1d5cf51bcb` |
| `complex-boundary-repair752/source_checks752.py` (bounded source checks, below) | `8571ec14ba0059881d163d519b465f44114fc2a76d0e1b8568b7606b989b8cda` |

The folder `complex-boundary-repair752/` holds exactly these 3 files. Its tree digest is
`7654ba46ec7313cb81e0d89f9d647734eea44a3309848cb4eda727c06e0b48fa`. Method: inside the folder, write one line
`<sha256>  <name>\n` per file, sorted by name (what `shasum -a 256` prints), and take the sha256 of that text.

The only file changed against base553 is `boundary.rs`. No other source file, `script.tsv`, narration unit, take,
fact, oracle, gate or other chapter was copied or edited. The canonical `explainer_corpus` was not used.

## What changed in `boundary.rs`

1. **Inner heads.** These are the clockwise heads at "What survives is the outer polygon" and the counterclockwise heads
   at "So the two polygons, both walked counterclockwise".
   - Before: `output()` green, 0.12 scene long, on the green polygon and circle.
   - Now: hero yellow (the colour of the boundary walks they stand for), filled, with a rim of the background
     (`ground()`, #0E1116) of width 2.5. Size `INNER_HEAD` = 0.16 scene, about 43 native 4K px long and wide (old: 32).
   - They sit on the same side midpoints, and the `counterclockwise` flag sets the direction as before.
   - Contrast ratios (WCAG relative luminance):

     | Pair | Ratio |
     |---|---|
     | rim against the green line | 8.82 |
     | fill against rim | 13.75 |
     | fill against green | 1.56 |

     The rim supplies the luminance edge and the yellow supplies the hue. Ink fill instead of yellow would be a one-word
     change in `inner_head_style` (ink against green is 1.86).
2. **`p`.** `POLYGON_NAMES_AT[1]` moves from (-0.445, -0.445) to (-0.245, 0.0). That is just outside the inner
   polygon's corner at 180°, in the ring between the small circle's line and the dashes.
   - It stays ink, like the `p` in the formula.
   - P is unchanged.
3. **Outer heads unchanged:** `OUTER_HEAD` = 0.12, `flat(hero(), 1.0)`, as before.
4. **Helpers.**
   - `side_head_points` is the head geometry as a pure function. The drawing and the tests both use it.
   - `inner_head_style` is the style above.
   - `side_heads` now takes a size and a style instead of a colour.
   - `ground` is imported from `draw`.
5. **Two new tests,** appended after the base's tests, which are kept verbatim:
   - `the_inner_heads_stand_out_and_point_the_way_each_side_is_walked` checks five things:
     - the style's colours, opacities and rim width;
     - contrast above 4.5 for rim against green and for fill against rim;
     - the size against the outer heads and above 40 native px;
     - for both directions and all 8 sides, that each head is centred on its own side's midpoint, is `INNER_HEAD` long
       and wide, and points counterclockwise exactly when it should;
     - with the rim included, each head is at least 0.035 scene (about 9.5 native px) from `a`'s ink box, the hole's
       ring and the dashes.
   - `each_polygon_name_stands_by_its_own_polygon` checks three things:
     - `p`'s ink box clears the small circle's line and the dashes by at least 0.035;
     - it clears every inner head of both directions, rim included;
     - each name's centre is more than 3 times nearer its own polygon than the other.

   Both tests map points with the scene's own `close_up()` plane, as the drawing does. The label half sizes are measured
   constants, as in the base's existing card test.

## What did not change

- `construct()` keeps all 212 timing calls with identical arguments: `wait_for`, `wait_sentence`, `until`, `play`,
  fades, `pause`, `wait_until`, `tween_linear`, `say` and `hold`.
  - The 29 `wait_for` phrases of 09_1 are found in script order.
  - The text from 09_2 onward is byte-identical.
  - The only base lines removed are 12: the import line pair, one doc line, `POLYGON_NAMES_AT`, the old `side_heads`
    head and body, and the three head calls.
- Only mob styles and one label position change, inside fades that already exist. No animation is added or removed,
  and no run time changes. So 09 should keep 16504 frames and every phrase time. This is unproven until root's build
  and exact schedule and audio comparison.
- ϱ is not moved; see below.

## ϱ: not relocated, because it is not safe

The task asked for this only if safe. I checked the shrink updater's start, (SMALL_R + 0.06, -0.12) = (0.24, -0.12),
using ϱ's ink as measured in the native frame.

1. **The triangle walks cross it.** The inset walk on T1_7's diagonal, drawn from (0.448, -0.048) to (0.187, -0.137),
   passes 0.0008 z from that anchor and runs through ϱ's ink. T2_7's diagonal walk passes 4.2 native px from the ink,
   and walk lines are 2.7 px half-width. Review 747 samples the walks from 58 s to 70.0 s. No pocket beside the small
   circle is free of walks in that window. The inside of the inner polygon is free, but it holds the hole and `a`, and
   later the shrinking circle passes through it.
2. **It touches the dashes.** At that anchor, ϱ's ink would reach 0.2914 z from a, and the dash ink starts at 0.2906 z.
   That is a contact, which would last for all of 43 to 119 s.
   - In the final, ϱ shows at that spot only at the start of the 0.7 s shrink tween.
   - Pictures 208 to 210 (119 to 121 s) already show it at the MID_R spot, ink box centred at (0.182, -0.122).

A start consistent with the updater would therefore need a new updater offset as well (a 119 to 128 s picture change),
and it still could not stand beside the circle during the walks. RHO_AT, ϱ's creation and the updater are unchanged.

If root wants ϱ by the small circle where it first enters a formula, one option (not done) adds no time:
- At "So in the limit, the big circle's integral equals the small circle's", about 110 s, fade the old ϱ out in the
  existing `[summed, outer_poly, inner_poly, piece]` fade (0.3 s).
- Fade a new ϱ in with `small_arrows` (0.3 s), at an offset clear of the dashes.
- Have the updater use the same offset function.

## When the repaired marks are on screen, and what is near them

The inner heads and both names fade in after "What survives is the outer polygon", once the boundary walks (0.2 s) and
the triangle rule (0.2 s) have faded. The tiles, inside walks and the red pair went earlier. They all fade out at "Now
keep the small circle fixed". In review 747's samples that is about 70.7 to 85 s (pictures 146 to 161). The clockwise
set fades out before the counterclockwise set fades in, so the two are never on screen together.

In that window, the tests check `p` and the heads against everything near them:
- the small circle and 8-sided inner polygon lines;
- the dashes;
- the hole's ring;
- `a`;
- `p` against both head sets.

ϱ (at RHO_AT, at least 0.4 z away), the outer polygon, its heads and P are not near them.

## Evidence, by kind

1. **Native measurement.** I used the 4K frames root extracted for review 746 (final 587):
   - picture 148 (72 s) and picture 155 (79 s), for the labels;
   - pictures 207 to 210, for where ϱ is;
   - and I measured three things:
     - ink boxes: `a` 24 x 25 px, `p` 27 x 35 px, ϱ 25 x 42 px, P 36 x 37 px, each centred within 1.5 px of its anchor;
     - dash ink at 0.2906 to 0.3090 z;
     - green ink of the circle, polygon and old heads to 0.1942 z.

   These are measurements of the old picture, not of the repair.
2. **Synthetic geometry.** I modelled the repair in Python, using those boxes and the scene's mapping (270 px per scene
   unit, 2.222 scene units per z). Least gaps, rim included, in native px:

   | Between | Gap (px) |
   |---|---|
   | heads and `a` | 10.6 |
   | heads and hole ring | 54.8 |
   | heads and the dashes' drawn edge | 49.3 |
   | `p` and the circle's line | 20.1 |
   | `p` and the measured dash ink | 12.9 |
   | `p` and the inner heads | 12.5 |

   `p`'s centre is 0.065 z from the inner polygon and 0.236 z from the outer, a ratio of 3.6. The old place was 0.449 z
   from the inner polygon and 0.129 z from the outer.
3. **Source checks.** Run `python3 -I -B complex-boundary-repair752/source_checks752.py
   cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e`. It exited 0 and printed four lines:
   - (1) both hashes;
   - (2) 212 timing calls identical, 29 phrases in order, 09_2 onward identical, ϱ unchanged;
   - (3) exactly the 12 expected base lines removed, base tests verbatim;
   - (4) a Python mirror of the two new tests passes on the repair's constants. On the base's constants it fails, as it
     should, on colours, contrast, rim, size, `p` against the dashes, and `p` being nearer the outer polygon.

   It also binds hero, green and the background to `palette.rs` and `draw.rs` in the base folder.
4. **Planted faults.** In a scratch copy of the layout, 10 of 10 faults made the checks fail, and the clean copy passed.
   The faults were: a fade time, an added pause, ϱ moved, `p` moved back, `p` on the heads, a green fill, oversized
   heads, a reordered phrase, an edited old test, and a 09_2 change.
5. **Format and parse.** `rustfmt --check` passed (exit 0), on a scratch copy of the crate with the repair in place. I
   used the pinned toolchain nightly-2026-08-31 (rustfmt 1.10.0) and the repo's `rustfmt.toml` (max_width 120, edition
   2024). Run on `main.rs`, it parses every module. The base copy also passed.
6. **Engine facts.** I read these at the pinned fmn rev a399d626, without a checkout:
   - `Style` is `Copy` with public fields;
   - `Vec3` is `[f64; 3]`;
   - `Srgb` is unit-range.

**Not established:**
- compiling, Clippy, and every Rust test, old and new (all unrun);
- any render or actual picture of the repair;
- legibility at 1:1, at 1080p or on a phone;
- whether the rim draws over or behind the fill (so how much yellow shows inside the dark outline);
- motion, audio, sync and the frame count.

## For root

Root's sequence:
1. Independent review (CC2 is ready as 759).
2. Replace only the bound `boundary.rs` with the repair. Root rebinds any gate or pin that names bda7d931; I changed none.
3. Guarded build, Clippy and tests.
4. Preview, then the actual pictures at 71 to 85 s.
5. Exact schedule and audio comparison.
6. One new final09; the other 14 finals are untouched.

Picture questions to answer:
- Do the inner heads read as clockwise at 72 s and as counterclockwise at 79 s, at 1:1 and at 1080p?
- Does the rim show?
- Is `p` read as the inner polygon's name?
- Are `a` and `p` clear of the heads?
