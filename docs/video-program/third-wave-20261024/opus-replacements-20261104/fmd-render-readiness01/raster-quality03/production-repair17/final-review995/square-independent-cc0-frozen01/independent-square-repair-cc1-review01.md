# Independent review: CC1 square-proof repair (08_5 L5, L11, L12)

- Reviewer: LavenderPike (CC0). Written 2026-11-10T21:23Z.
- Inputs: `final-review995/square-repair-cc1-frozen01` (receipt eac1fb7d…) and `final-review995/square-repair-cc1-support02` (receipt 02c029de…).
- Method: I read the full source and all tests of both candidate files, the two patches, the proposal, the mirror and both timing scripts with their outputs, and all four support02 files. I ran only cheap checks on copies: patch round trips, rustfmt `--check`, CC1's timing comparator, and my own geometry script (`square-review-cc0/geometry-cc0.py`, output `geometry-cc0.out`).
- Not done: cargo build, test or clippy; no render, no glyph probe, no layout lint.
- No production file, Bead, TODO or commit was touched.
- No audio heard or checked. Narration stays ElevenLabs Frederick Surrey, not Michael. The patch touches no audio, script.tsv or cue words, and every `wait_for` phrase is unchanged (see P10).
- Root asked for actionable findings. There is no visual or listening acceptance here, because this is a source-only review.

## Verdict

**CONDITIONAL PASS for guarded integration, on the exact build268 lineage only.**

- The mathematics, the quarter construction, c containment, diameter and perimeter halving, and the updater and fade lifetimes all pass.
- Reading the source, I found no compile or API defect. It is still uncompiled.
- One integration blocker (B1, about lineage, not content). It must be honoured.
- One Medium visual risk (F1, the L5 leader start) and one test weakness (F2). Fix both, or accept them on fresh stills before the 4K render.
- The rest are Low or notes.

| Area root asked about | Result |
|---|---|
| Mathematics | PASS |
| True quarter construction | PASS |
| c containment and association | PASS (Low F4: the L12 name is tinted) |
| Diameter and perimeter | PASS (Low F5: estimated baseline offset) |
| Updater and fade lifetimes | PASS |
| Compile and API (by reading only) | PASS, unverified (uncompiled) |
| Concrete visual risks | F1 Medium, F4 and F5 Low, N1 note |
| Integration path | B1: apply only onto 8a13e8a1 / e1f4d1b8 |

## Lineage and hashes (all verified at 21:23Z)

| Role | File | sha256 |
|---|---|---|
| build268 base loop_sums (= loop-sums-review-repair238.rs per review-repair-recipe268.json) | frozen01/base-build268-loop_sums.rs | 8a13e8a190cd502e7b58efbcb7c40872cdb0e6e702feb69e1b789e451a7a42b5 |
| build268 base mod (= recipe256 mod-review-repair256.rs) | frozen01/base-build268-mod.rs | e1f4d1b87f4b36f979e7800f7b2048117b9a4809b2372f385350698f5c3e7cbd |
| candidate loop_sums | frozen01/candidate01-loop_sums.rs | 9a91ed2a205451c5ce5f46a5d97c3a2c94829c46d45cfa72f8c3922bf77db874 |
| candidate mod | frozen01/candidate01-mod.rs | 4de1e4a56fde5c11ac3d6a65e772d09ec33db4f5757d817aff370799d3423578 |
| loop_sums patch | frozen01/loop_sums-L12-L11-L5-candidate01.patch | b34edad397ab83a5a63450dec2a9c23923b2d7cae9a811d6a1eab5404c6aa96b |
| mod patch | frozen01/mod-supplements-candidate01.patch | c1fb51678b7e82da025ca399e3b38b8bfd75afb60c09f47af1c7faea5bec8bc8 |
| proposal | frozen01/L12-L11-L5-repair-proposal01.md | 5e600e13bbafcbc4a4d237c9c907edfed7658bdec695622c9fa24432ec1bf872 |
| mirror and output (42/42 PASS) | frozen01/test-mirror02.py / .out | 918490d5… / e903796d0627c8196eeb1c8e6c9714c2cff707e20185090a6250334e322d8e2d |
| timing comparator and output | frozen01/timing-compare03.py / .out | 0d9513b7… / 422d364af8a6dbb9cc840a0c2c2de522e35f50977b5c915c4e80495472928847 |
| timing control and output | frozen01/timing-control04.py / .out | 83ba8d43… / 3f8f72c7fa5346eb26554c34319aff5a79cd4d7c573a5fc7b5752127c8cff119 |
| geometry (CC1) | support02/geometry-check01.py / .out | a4e4f4fa… / 7e8fb4e45f0a550a0450b61cb3175d1bc37e72979e81ff523fa3df5e0b526941 |
| mirror on base, control (15 FAIL) | support02/test-mirror02-base-control.out | a0ff56420015e2ded4f94ae15d098708efa9a204f2b29d3167bf84b1bf43d9c8 |
| timing control variant | support02/control04-timing-variant.rs.txt | 541a5b7ede2fe73911044d9ed156373552bcb7e273d2a3a0090b7b7cb1277cdb |
| committed tree, NOT the base | explainer_corpus/…/chapters/loop_sums.rs | cf383504cdc80652acf4dacf13ba6b99cc6c679cb859749618dd559ab95f695d |
| committed tree, NOT the base | explainer_corpus/…/chapters/mod.rs | 57c46d67161490cbadc96e9ccad6ae6eb821c54d4bd257f3233735792a529b5e |
| this review's geometry | square-review-cc0/geometry-cc0.py / .out | fa9bd581… / a0f7666be91ba474b19a705cdff95c44b1f4a14d9f51b69b10145e604d5a4646 |

- Every frozen01 and support02 file matches its receipt.
- On copies, each patch applied to its base gives the candidate byte for byte (cmp equal) for both files.
- preserved214 (69042b14) is a different file from the base and must not be used as the base.

## Blocker

### B1 (integration blocker): apply only onto the build268 lineage

- **What:** The committed `explainer_corpus/complex_analysis_visual/src/chapters/loop_sums.rs` (cf383504) and `mod.rs` (57c46d67) come from an older tree with no `goursat_proof`. A dry run of the loop_sums patch on the committed file fails 8 of 9 hunks.
- **Pass condition for root:** In the guarded tree, confirm before applying that loop_sums.rs is 8a13e8a1… and mod.rs is e1f4d1b8…. Then either:
  - apply the two frozen patches, or
  - copy candidate01 in place and confirm 9a91ed2a… / 4de1e4a5….
- **Fail:** any other base, including preserved214 or the committed tree. Do not apply with fuzz.

## Findings

### F1 (Medium, visual): the L5 leader starts beside a nest edge, not visibly at the dot

**Source:** `c_name_and_leader` (candidate01-loop_sums.rs:342–346). It is used at 542–545.

**Geometry:**
- c = (−2.1625, 0.5875). Unit direction u = (0.583, −0.813).
- The dot rim is at s = 0.07. The leader runs from s = 0.18 to s = 0.446.
- The ray from c crosses the shared x = −2.1 edge (the right edges of Q_3, Q_4 and Q_5, stroke 3) at s = 0.107. That crossing falls inside the bare 0.11-unit stretch between the dot rim and the leader start: 7.4 px at 960, 29.7 px at 4K.
- The leader start sits 0.042 units from that edge's centre line and 0.059 from the Q_3/Q_4 corner at (−2.1, 0.4).
- Ink gap, after subtracting half-widths 0.015 + 0.0075: 0.020 units = 1.3 px at 960 = 5.4 px at 4K.

**Risk:** On screen the order is dot, then nest edge, then a near-touching leader. The thin dim leader can read as growing out of the nest edge or the Q_3/Q_4 corner rather than from c. That would weaken the very association L5 repairs. The source comment's "meets no nest edge" (line 540) holds only in centre-line terms.

**Geometry is tight here:** CC1's own search (support02 geometry-check01.out rows 7–14) found no start with more than 0.031 of leader-edge clearance. Any direction from c toward open space crosses that edge band near the dot.

**Fix options (root chooses on fresh stills):**
- (a) Start the leader at the rim, s ≈ 0.075. It then visibly leaves the dot and crosses the x = −2.1 edge as a deliberate crossing: dim 1.5 over a stroke-3 edge.
- (b) Drop the leader. This is CC1's own fallback. The name alone sits 0.566 from c (38 px at 960), with 9.7 px of ink clearance to the nest and 10.2 px to the δ-circle, by glyph-box estimate.
- (c) Keep it as is, and accept only after a 4K still of the L5 window shows the leader reads as attached to the dot.

**Pass/fail:** FAIL as a source-only guarantee of association. It can pass on a 4K still.

**Not a defect:** the leader crosses the δ-circle at s = 0.318 once that circle is drawn. This is acknowledged in the proposal, and the circle arrives later.

### F2 (Low-Medium, test): the leader guard cannot prove ink separation

**Source:** `the_name_of_c_is_near_its_dot_and_clear_of_the_nest` (candidate01-loop_sums.rs:1764–1782).

- `segment_gap(from, to, a, b) > 0.02` (line 1777) compares centre lines. The half-width sum is 0.015 + 0.0075 = 0.0225, so a leader whose ink touches an edge would still pass.
- No assertion checks that the stretch between the dot rim and the leader start is free of edges, which is exactly the F1 situation.

**Suggested fix:** assert `> 0.0225 + margin` in ink terms; with option (a), assert a rim start instead. Also check that no nest edge crosses the segment `[c + u·0.07, from]`, or deliberately allow one crossing with a comment.

**Pass/fail:** FAIL as a guard. The current value passes (0.042 > 0.02), but for the wrong reason.

### F3 (Low, evidence provenance): the support02 geometry does not bind the frozen leader start

- geometry-check01.py searched leader starts s0 ∈ {0.12, 0.14, 0.16} only (lines 128–134).
- The chosen offset (+0.330, −0.460) is reported at s0 0.16 with leader-edge 0.031 (row 12).
- The frozen source uses 0.18, which gives 0.042 (geometry-cc0.out).
- The direction is benign: the frozen value is larger. Still, root should cite 0.042 / 1.3 px, not 0.031, as the frozen clearance.
- support02's receipt says these files were preserved, not re-run; this agrees.

### F4 (Low, visual): the L12 kept fill tints the c name

- `proof_fill` is `flat(hero, 0.16)` (line 405–411). It is added at L12 above `centre_name`, which sits at c + (−0.22, 0.28) = (−2.82, 0.18). That point is inside the kept quarter; the test at line 1729 asserts `inside(c_name)`.
- Only the dot is redrawn on top: line 762, radius 0.07, the same colour and radius as centre_dot at line 648.
- So the c glyph takes a 16% hero tint while the dot does not.

**Fix:** add a second `kit.tex(stage, "c", 26.0, output(), &[], v(centre.re - 0.22, centre.im + 0.28))?` to `quarter_picture` after the fill. Or accept the tint on a still.

**Pass/fail:** PASS for legibility: the glyph box has 9.3 px of ink clearance at 960 to every L12 stroke. The colour association is weakened slightly.

### F5 (Low, estimate): the "perimeter P_n" pair may sit off a shared baseline

- Lines 705–715 centre both boxes at y = −2.5 with `move_to(…, ORIGIN)`.
- From CM metric estimates (descender of 'p' against subscript 'n'), the P_n baseline sits about 0.022 units below the word's: 1.5 px at 960, 5.9 px at 4K.
- The horizontal placement from measured widths is sound: estimated x −3.225 to −1.975, with a 0.14 gap, and 19.5 px below the square's bottom-edge ink.

**Fix if a still shows it:** shift P_n by the measured baseline difference, or align the bottoms of the two boxes' non-descending parts.

**Pass/fail:** cannot be decided from source.

### N1 (note, unchanged from base): D_n name clearance

The D_n name at (−1.7, 1.2) is about 5.7 px at 960 from its own diagonal's ink, by glyph-box corner. CC1 got 0.092 (6 px). The patch does not change it; the L11 walk now leaves the whole diagonal visible beside it.

### N2 (note): earlier picture evidence is superseded for these windows

- The patch changes the 08_5 L5, L11 and L12 pictures. The source comment at line 538 cites root's L5 sheets 344–359.
- Every earlier still of those windows, and every downstream hash binding of this chapter (final995 extraction, master993 bindings), no longer describes the candidate. Fresh stills are needed after root's guarded render.

## PASS items with evidence

1. **True quarters.** `proof_children` gives [a, ab, ce, da], [ab, b, bc, ce], [ce, bc, c, cd] and [da, ce, cd, d]. All four are squares of half the side, and each shares one corner with Q_n. `bisection_cuts` (334–338) draws exactly the two midlines. KEPT_QUARTER = 3 is the top-left quarter, sharing corner (−4.6, 1.9) with Q_n.
2. **c containment.** In the magnified picture, c = SQUARE_CENTRE = (−2.6, −0.1), the centre of the enlarged Q_n. It is the common corner `ce` of all four quarters, so the kept quarter contains c whichever one the rule keeps (closed squares). The comment at 745–751 says honestly that KEPT_QUARTER is a layout choice and that nothing on screen calls it the largest.
   - Drawing c at the centre of Q_n is a special but valid nest configuration, inherited from the base.
3. **Diameter and perimeter.** The kept quarter's side is 2 against 4, so its diameter is D_n/2 and its perimeter P_n/2. D·P therefore falls to a quarter, which matches "this bound shrinks to a quarter every time". The kept diagonal (−4.6, −0.1) to (−2.6, 1.9) is parallel to D_n, half as long, and 1.414 away.
4. **The L11 walk.** It visits enlarged[(1 + k) % 4], that is, bottom-right, top-right, top-left, bottom-left: counter-clockwise, oriented area +16. The updater interpolation ends at t = 4 at moving[0], so the walk starts and ends at the bottom-right corner. The resting |z − c| segment is c to (−0.6, −2.1), of length 2.8284 = D_n/2. It lies on the other diagonal, so the whole D_n diagonal stays visible, and the picture shows |z − c| ≤ D_n without showing equality.
5. **Updater lifetime.**
   - `walk_corners` (a Vec<C>) is read at 667–668 before it moves into the closure at 673. This matches the base's `enlarged.clone()` pattern.
   - The anchor is added, the updater set, then `clear_updaters(anchor, true)` and `remove_from_scene(anchor)` (699–700).
   - c_aside and d_aside are unchanged (0.3 s inside the 2 s walk).
6. **Fade lifetimes.**
   - point_leader fades in with point and name (0.35) and out with neighbourhood and small (0.25, line 642).
   - perimeter_word joins P_n's existing 0.3 s fade and is listed in square_picture.
   - All seven quarter_picture mobs fade in at 0.5 and are extended into square_picture (837), which fades at 0.4 before `proof_triangle()`. This is asserted by the test at 1785.
   - Nothing new outlives the square picture.
7. **APIs (by reading, at pin a399d626 usage in the base):**
   - `stage.get_width` is #[must_use] and its value is used.
   - `stage.move_to` returns &mut Self and is not must_use, so ignoring it is fine.
   - The `kit.text` and `kit.tex` argument orders match existing calls.
   - `proof_path`, `proof_fill`, `dot`, `fade_in_all`, `fade_out_all` and `Palette::dim/ink` are all existing helpers.
   - The array destructure is valid in edition 2024. C is Copy and Debug.
   - The test helpers (`drawn_nest`, `point_to_segment`, `segment_gap`, `sides`, `goursat_body`) are all used. I spotted no clippy (default, `-D warnings`) trigger.
8. **mod.rs supplements.** `loop_sums::PERIMETER_WORD` goes into TEXT and `loop_sums::KEPT_TEX` into TEX. Both are `pub const &str`, and `loop_sums` is in scope in mod.rs. Neither is a duplicate under `no_supplement_is_empty_or_duplicated`.
9. **Formatting.** rustfmt `--check` (edition 2024, max_width 120) passes on both candidate files and both bases.
10. **Timing.**
    - timing-compare03 on base against candidate: 111 events, 34 `wait_for` phrases, 77 fades, plays and tweens; skeleton identical.
    - timing-control04 shows the comparator detects a single 0.5 → 0.6 change (first difference at event 67, IDENTICAL False), so the comparator is live.
    - No time is added, so narration alignment is unaffected.
11. **Mirror.** test-mirror02 passes 42/42 on the candidate and fails 15 on the base, all of them source bindings. So the new source-binding assertions do discriminate. It is a Python mirror, not `cargo test`.
12. **L12 label clearances (glyph-box estimates, ink terms, 960):** c is 9.3 px from the nearest stroke, Q_{n+1} 18.6 px, and D_n 5.7 px (N1). The kept-fill c dot is redrawn at the same radius and colour.
13. **L5 name.** It is 0.566 from c (38 px at 960), with 9.7 px of ink clearance to the nearest nest edge and 10.2 px to the δ-circle. The old name sat 0.958 away and crossed four edges.

## Limitations

- Uncompiled: no `cargo build`, `cargo test` or clippy was run. Every API and lint statement above comes from reading the source against the base's existing calls.
- No render, no stills of the candidate, no glyph probe for `perimeter` or `Q_{n+1}`, and no layout lint.
- Glyph sizes and baselines are CM metric estimates (F1, F4, F5, N1 and P12). The scene measures real widths at run time.
- Other build268 chapter files were not checked for supplement duplicates across chapters; only the mod.rs lists were.
- No picture, motion, AV, fine-sync, listening or release acceptance is given or implied. No audio heard or checked.
- Narration is unchanged: ElevenLabs Frederick Surrey, not Michael.
- Preserving the support02 files is not acceptance, and neither receipt claims acceptance.

## Actionable summary for guarded integration

1. **B1:** confirm the guarded tree's loop_sums.rs is 8a13e8a1… and mod.rs is e1f4d1b8… before applying. Afterwards they must read 9a91ed2a… / 4de1e4a5….
2. Decide F1 before the render: (a) a rim start with a deliberate crossing, (b) no leader, or (c) keep and judge on a 4K still. If the source changes, update the F2 guard to ink terms in the same edit and re-run the timing comparator.
3. Optional Low fixes: F4 (redraw the c name in quarter_picture) and F5 (baseline, only if a still shows it).
4. Then build, test and run clippy `-D warnings` and the glyph probe in root's guarded flow. Render, then take fresh 4K stills of the L5, L11 and L12 windows (N2).
