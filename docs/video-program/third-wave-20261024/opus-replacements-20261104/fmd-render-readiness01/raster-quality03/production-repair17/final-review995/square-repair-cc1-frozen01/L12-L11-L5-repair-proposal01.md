# 08_5 repair proposal for L12, L11 and L5 (scratch only, CC1)

Date: 2026-11-10. This is a source-only proposal for root to integrate and guard-test. Nothing in production was edited, and nothing was built, compiled, tested with cargo or rendered. No Beads were touched.

## Source binding

| Role | File | sha256 |
|---|---|---|
| Base loop_sums.rs (build268) | `base-build268-loop_sums.rs` | `8a13e8a190cd502e7b58efbcb7c40872cdb0e6e702feb69e1b789e451a7a42b5` |
| Base mod.rs (build268) | `base-build268-mod.rs` | `e1f4d1b87f4b36f979e7800f7b2048117b9a4809b2372f385350698f5c3e7cbd` |
| Candidate loop_sums.rs | `candidate01-loop_sums.rs` | `9a91ed2a205451c5ce5f46a5d97c3a2c94829c46d45cfa72f8c3922bf77db874` |
| Candidate mod.rs | `candidate01-mod.rs` | `4de1e4a56fde5c11ac3d6a65e772d09ec33db4f5757d817aff370799d3423578` |
| Patch for loop_sums.rs | `loop_sums-L12-L11-L5-candidate01.patch` | `b34edad397ab83a5a63450dec2a9c23923b2d7cae9a811d6a1eab5404c6aa96b` |
| Patch for mod.rs | `mod-supplements-candidate01.patch` | `c1fb51678b7e82da025ca399e3b38b8bfd75afb60c09f47af1c7faea5bec8bc8` |

**Where the base hash comes from.** Hash 8a13e8a1 is the file that build268 used:
- `review-repair-recipe268.json` maps loop_sums.rs to `loop-sums-review-repair238.rs` with this sha;
- `review-build268-terminal272/source-copy.json` binds the same sha;
- the copies in root-boundary552/760, mini-source492 and mini-max-source506 hash the same.

**mod.rs.** The base is recipe256's `mod-review-repair256.rs`, which equals the root-boundary760 copy.

**Preserved214 is not only a plan rebind.** `production-source-preserved214/.../loop_sums.rs` is `69042b14…` (the same as `loop_sums-repair19.rs`). It differs from 8a13e8a1 in four code hunks:
- the `Timed` import;
- the c label at `+0.28, +0.27` instead of `+0.95, +0.12`;
- the heading "Selected square Q_n, magnified" instead of "Selected square, magnified";
- no label fades during the walk.

**What the final995 stills show is build268.** Three things on the sheets match 8a13e8a1:
- the c label sits about 64–65 px from its dot (L5);
- the heading reads "Selected square, magnified";
- the labels leave during the walk (sheets 0415–0437).

So the patch is written against 8a13e8a1. Root should confirm which file the canonical tree carries before applying.

**Round trip.** `patch -o` applies each patch to the base copy. The output is byte-identical (`cmp`) to the candidate.

## What changes, and why

### L12: draw a true bisection quarter (sheets 0444–0453, from L10794)

The concentric half-square (`centre + (z − centre)·0.5`) is replaced by the cut itself:
- **Two dim cut lines.** `bisection_cuts` draws a line from each edge midpoint to the opposite edge midpoint. Their four halves are exactly the four shared edges of the `proof_children` quarters; a test proves this.
- **One kept quarter,** `proof_children(&enlarged)[KEPT_QUARTER]`, drawn with:
  - the selection vocabulary the nest already uses (a 16% gold fill and a gold edge of width 5);
  - its own teal diagonal (width 4);
  - a small `Q_{n+1}` name in ink colour.
- **c redrawn on top,** so the new corner does not cover it.

**The mathematics:**
- **Corner, not centre.** Every quarter of Q_n shares one corner with Q_n. None is concentric: each quarter's centre is 1.414 units from c in the magnified view.
- **Halving.** The kept quarter's diameter and perimeter are exactly half of Q_n's, so D·P falls by a factor of 4. This is what "Two halves make one quarter" and `D_nP_n = D_0P_0/4^n` say.
- **Parallel diagonal.** The kept quarter's diagonal is parallel to the D_n diagonal, half as long and 1.41 units away from it. The halving can therefore be read off the picture directly.
- **c stays contained.** In build268, c is defined as the centre of the last drawn nested square, and the magnified view is centred on it. So c is a corner of every quarter, and the kept quarter contains c (a closed square contains its corners) whichever quarter the rule keeps.
  - c cannot lie in the interior of any quarter without moving c. That would change the nest and zoom pictures, which were already reviewed, so the patch does not do it.
- **No claim about the largest integral:**
  - The four quarters are visible through the cut lines.
  - `KEPT_QUARTER = 3` (top-left) is chosen for layout only: it is clear of the D_n and P_n names, and the c name falls inside it. The source says so.
  - Nothing on screen says "largest". The column still states only `|I_{n+1}| ≥ |I_n|/4`, which is exactly the rule.
  - This matches the existing note at `goursat_proof` ("The highlighted chain is illustrative … no nonzero holomorphic integral is fabricated to choose a child").

### L11: show D_n as the full diagonal and say what P_n is (sheets 0411–0418 and 0432–0437)

**The walk now starts at the bottom-right corner** (`WALK_START = 1`, `walk_corners`). It runs the same tween, still counter-clockwise (oriented area +16).
- At rest, the green |z − c| segment is half the *other* diagonal. It is no longer drawn over the D_n line.
- The whole teal diagonal is therefore visible corner to corner beside its unchanged "D_n" name.
- |z − c| = D_n/2 is visibly smaller than D_n, so the picture shows "at most" rather than equality.

**"P_n" becomes "perimeter P_n":**
- The word is drawn in dim text at 20 pt (`PERIMETER_WORD`), and the TeX name is unchanged.
- The pair is centred under the square at y = −2.5, using the widths `stage.get_width` measures at render time, with a gap of 0.14. The word and the name therefore cannot overlap, whatever the glyph widths are.
- The narration already says "the steps add up to the perimeter" and "times its perimeter", so the on-screen word matches what is spoken.

### L5: tie the nest's c label to its dot (sheets 0344–0359 onward; root confirmed)

**Old position.** The name sat 0.96 units (65 px) from the dot. The straight path from dot to name met four nest edge segments: the shared x = −2.1 edge of Q_3, Q_4 and Q_5, plus Q_2's right edge.

**New position.** The name moves to `c + (0.33, −0.46)`, 0.566 units (38 px) from the dot, in the open lower right of Q_2.
- **Clearances** (estimated 'c' glyph about 0.12 × 0.12 units):
  - every nest edge by about 11 px;
  - the δ-circle (r = 0.318) by about 11 px. The circle appears at "Close enough to c" while the name is still on screen.
- **Leader** (`c_name_and_leader`, dim grey, width 1.5):
  - it starts 0.18 from c, past the dot's 0.07 rim, and ends 0.12 short of the glyph;
  - it meets no nest edge; its closest approach is 0.042 units (2.9 px);
  - it is grey, not green, so it cannot be mistaken for a δ radius.

**Fades.** The leader fades in and out in the same calls as the dot and name.

## Timing and narration stay unchanged

- **Timing skeleton** (`timing-compare03.py`). This lists, in order, every call in goursat_proof and the 08_5 block of construct that can take scene time or bind narration: `line.wait_for` phrases, `fade_in_all` and `fade_out_all` run times, `play` and `play_prepared`, `tween_linear`, `.rt`, `wait_until`, `hold`, `reading_window` and `music_breath`.
  - Base and candidate both have 111 events, and the sequences are **identical**: 34 phrases and 77 timed calls.
  - Positive control: a variant with one fade changed from 0.5 to 0.6 s is reported at event 67 (`control04-timing-variant.rs.txt`).
- **Phrases.** No `line.wait_for` is added, removed or changed.
- **Scripts.** script.tsv, captions and narration are not touched.
- **Objects.** Every new object is added to an existing fade call, which plays its objects together at that call's run time, so the cue timing is unchanged.
- **Existing tests.**
  - `goursat_phrase_waits_match_the_installed_recording_text` still sees the same 34 phrases in order.
  - `each_card_outlasts_its_reading_window` scans only the 08_5 block of construct, which is unchanged.

## Guard tests added (inside `loop_sums.rs` `mod tests`)

1. `the_kept_square_is_a_bisection_quarter_that_holds_c` checks that:
   - c is a corner of every quarter, and no quarter is concentric;
   - the kept quarter shares a corner with Q_n and has half the diameter and half the perimeter;
   - its diagonal is parallel to the D_n diagonal and apart from it;
   - the cut halves are each shared by two quarters;
   - the c and Q_{n+1} names sit inside the kept quarter and D_n outside, all clear of strokes;
   - the source binds `proof_children(&enlarged)` and `KEPT_QUARTER`, and the concentric map is gone.
2. `the_walk_rests_off_the_diagonal_and_stays_counterclockwise` checks that:
   - the walk is counter-clockwise;
   - the start corner is off the diagonal, at |z − c| = D/2;
   - the D_n name is clear of the resting segment;
   - the source binds `walk_corners`.
3. `the_name_of_c_is_near_its_dot_and_clear_of_the_nest` checks that:
   - the name is 0.4–0.7 from its dot;
   - it clears the nest edges and the δ-circle;
   - the leader starts past the rim and meets no edge;
   - the source binds the leader in both fades.
4. `the_repaired_square_picture_leaves_before_the_triangle` checks that the "perimeter" word, the P_n name and every quarter object are in the single fade-out before the triangle appears.

**mod.rs:** `loop_sums::PERIMETER_WORD` is added to `TEXT` and `loop_sums::KEPT_TEX` (`Q_{n+1}`) to `TEX`, so the glyph probe sees them without a render. There are no duplicates.

## Checks I ran (no compile)

- **rustfmt.** `rustfmt --edition 2024 --check --config-path rustfmt.toml` (max_width 120) passes on the candidate, which also shows the whole file parses. The base passes the same check.
- **Mirror on the candidate.** `test-mirror02.py`, a Python copy of the four new tests assertion for assertion with the same thresholds, gives **42/42 PASS** (`test-mirror02.out`).
- **Mirror on the base.** The base fails 15 checks: all the source-binding checks (`test-mirror02-base-control.out`).
  - The geometric checks can only fail on the base by not compiling, because the constants do not exist there.
- **Geometry.** `geometry-check01.py` holds the derivation (nest, c, δ, quarters, candidate searches and clearances), with its output in `geometry-check01.out`.

## Not done here (root's guards)

- `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test` (the 4 new tests plus all existing ones) have not been run. The Rust is uncompiled.
  - APIs used that already appear in this film: `stage.get_width` (draw.rs, counting.rs) and `stage.move_to` (kit.rs; returns `&mut Self` and is not `#[must_use]`).
  - Array pattern `for [a, b] in` and array `.map`: the crate is edition 2024.
- The glyph probe for "perimeter" and `Q_{n+1}`, the layout lint and native renders have not been run.
- I have not looked at any frame of the new pictures. The stills to read after a render are:
  - L5: about L6996–L9772, the nest name and leader, both with and without the δ-circle;
  - L11: about L10031–L10380, the walk start and resting segment, and L10339 onward, "perimeter P_n";
  - L12: about L10794–L13064, the quarter picture through the cross-fade to the triangle.

## Limits and risks

- **Glyph sizes are estimates,** calibrated on crops s0357 and s0395. The L5 and Q_{n+1} clearances (about 11 px and 20 px) assume those sizes. Only the "perimeter P_n" pair is measured at runtime.
- **The leader crosses the δ-circle** from "Close enough to c" until the zoom, about L9144–L9772. It is dim grey over the green ring. If a reviewer finds the crossing busy, removing `point_leader` (3 lines) keeps the closer 38 px name alone, which is still unambiguous by proximity.
- **The kept quarter's 16% gold fill lies over the c name** in the magnified view, a slight tint, the same as the nest's earlier selection fill. c sits on the kept quarter's corner, which is mathematically exact, and is redrawn on top.
- **Frames that were reviewed every frame will change.** Starting the walk at e1 changes L10031–L10380 (sheets 0411–0437), which my final995 review read every frame. Those frames need a fresh look.
- **Downstream bindings are invalidated.** Any frame-, hash- or map-bound evidence keyed on loop_sums.rs 8a13e8a1 or mod.rs e1f4d1b8 no longer applies to the patched files: previews, picture maps, recipes and source manifests. Every file that names those hashes needs rebinding.
- **Out of scope.** L10 ("epsilon" spelled out) and the other chapter 08 findings (L2, L3, L4, L6–L9) are not touched.
- **This is not AV acceptance.** I listened to nothing, and fine sync is not checked. ElevenLabs Frederick Surrey narration and all cue timing stay as they are.
