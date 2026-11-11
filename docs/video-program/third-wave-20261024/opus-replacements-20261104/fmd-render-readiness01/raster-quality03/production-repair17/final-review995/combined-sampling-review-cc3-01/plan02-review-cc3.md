# Plan02 sampling review (CC3, 2026-11-10)

**Scope.** This is an independent review of CC1's native-sampling01 plan02. I checked it against the frozen combined-repair bundle02 sources, CC1's own simulated frames, master993's chapter rows and the preserved boundary775 frame selection. It is a source and frame-list review only. I did not build, render, decode or view any frame, did no listening or AV check, and accept nothing. I changed no bound plan and no Beads, and launched nothing. Narration stays ElevenLabs Frederick Surrey, unchanged.

## Answer to root

| Transition (local frames; master-equivalent) | Frames in the union | Verdict |
|---|---|---|
| 09: SUM_TEX, polygons and piece fade out, 6488–6505 (73553–73570) | none inside; 6480 and 6487 before, 6510 after | **True gap, not visually relevant to the repair.** Required: nothing. Optional: one frame, **6496 (73561)**. |
| 09: closing (check text, breath 16324–16443, fade-out 16444–16473, painter leave 16474–16503, join to 10_rigidity at 83569) | plan02 stops at 16123; prior775 adds 16140 … 16440, 16470, 16473, 16474, 16488, 16500–16503 | Not a gap once the union is applied |
| 08: L11 walker and \|z−c\| fade out, 10748–10765 (62854–62871) | none inside; 10483 before, 10785 after | Bracketed; no supplement |

The union for 09 is prior775 `selected_frames` (814) plus plan02 `frames-09_boundary-all.txt` (735), which gives 1506 frames. That matches what root read: 6480, 6487 and 6510 are its only frames in 6470–6520.

**Why 6488–6505 is not visually relevant.**
- `summed = proof_tex(...SUM_TEX...)` is one Mob (`boundary.rs:451–457` caps its width at 5.1). It leaves in one `fade_out_all(stage, &[summed, outer_poly, inner_poly, piece], 0.3)` (`boundary.rs:854`), so the closing bar and the rest of the formula share one alpha ramp.
- candidate03 changes only the TeX string (`dz\,\Big|`). Nothing about it moves or changes during the fade.
- The repaired gap and any width-cap rescale show in full at 6487. A bar left behind would show at 6510, while the inset fades.
- It is still the only event that CC1 tagged with a finding (M1) and left outside every window. If root wants an in-transition frame for it, 6496 (k = 9 of 18, about half alpha) is enough. That is the `optional` entry in `supplement-cc3.json`.

**09 closing.** The repaired states that last to the chapter end are:
- a's name, whose opacity is driven through `set_opacity` in 09_7 and 09_9. `a_name_shown` returns 1 at r = 1 and at r = √2.
- the "1" card, re-rooted above the 09_9 disk (`boundary.rs:1415–1417`).

Both appear in every union frame from 16140 to 16503. Both are in the closing `all` list (`boundary.rs:1495–1513`). Frame 16440 shows the held state before the fade. Frames 16470 and 16473 fall inside the fade, at about 10% and 0% alpha: an a's name stuck at full opacity would be obvious there. Frame 16503 is the last frame before the join. The selected timeline 5a91b8f9's breath row (16324, 2 s) and the 30 + 30 fade frames give exactly 16504 frames, which matches master993's 09 row.

**08 handover.** The L11 objects leave in the base's own fade list (`loop_sums.rs:743`: expansion, remainder, epsilon, error, local, distance, walker). The bundle changes only where the walker rests (WALK_START corner). That resting state is sampled from 10340 to 10483. Frame 10785 shows both objects gone before the quarter picture fades in.

## Checks I ran (`check-plan02-cc3-02.py`, log `check-plan02-cc3-02.out`)

- **Bindings.** All eight bundle02 files hash to the plan's `sources`: staged 90b51c2b / a8d62f56 / 4de1e4a5 / 4b689791 and bases 8e93012e / 8a13e8a1 / e1f4d1b8 / cd1900aa. The manifest is 9c29672e and the receipt 2d9fbc03.
- **Chapters.** The plan's starts and lengths equal master993's rows (summary 38b52105): 01 at 0 with 6311 frames, 08 at 52106 with 14959, 09 at 67065 with 16504. So the joins are 6311 (02), 67065 (09) and 83569 (10_rigidity).
- **Windows.** Every window's global range equals start + local. Every still lies inside its window and chapter, and no still is in both tiers.
- **Samples.** There are no duplicates. Each sample's frame equals start + local, and its `seconds` is master-equivalent (frame / 60). Each sample is listed in the windows it names. The six `frames-*.txt` files equal the sample tiers. Counts are 72/89, 259/407 and 419/735, for 750 required (94 sheets of 8) and 1231 in all (154 sheets).
- **Tagged events.** Every simulated event that CC1 tagged with a finding lies inside some window, except the SUM_TEX fade-out above.
- **Lifetimes.** Every on-screen interval of a changed object has required stills at its start and end, except the three transitions in the table.
- My first checker (`check-plan02-cc3.py`) flagged every 08 and 09 sample's `seconds` as wrong. That was my error: I assumed local seconds. Version 02 fixes it. I kept both files.

## Completeness against the bundle02 film diff

| Change (staged source) | First drawn (local) | In and out transitions covered by |
|---|---|---|
| the_bump `preview_arrow()` with `.buff(0.0)` | 01: 5564 | 01-arrow-fade-in, held, fade-out (through 6310, the last frame) |
| mod.rs: `PERIMETER_WORD`, `KEPT_TEX` in probe lists | not drawn in the film | n/a |
| loop_sums L5: c name, leader, `C_NAME_OFFSET` | 08: 6988 | L5-fade-in, held, delta-circle, fade-out (9782–9796) |
| loop_sums inset `c_name_at` (rename only) | 08: 9797 | 08-inset |
| loop_sums L11: `WALK_START`, whole D_n, c and D_n set aside | 08: 10026 | diagonal-in, walk, perimeter-F5; out bracketed (above) |
| loop_sums "perimeter P_n" pair | 08: 10338 | perimeter-F5; out in L12-fade-out |
| loop_sums L12: cuts, kept quarter, redrawn c, `KEPT_TEX` | 08: 10787 | quarter-in, held, fade-out (13054–13078) |
| boundary ARC / SUM / SHRINK closing `\,` | 09: 5680 / 6177 / 7229 | M1-arc, M1-sum, M1-shrink, rest-zero; SUM out: the gap above |
| boundary riding a, ρ, leader; old a, ρ leave | 09: 7066–7096 | M2, shrink1, hold-mid, shrink2-rest, rest-zero, proof-gone |
| boundary `SHRUNK_R` 0.06 | 09: 7601–7660 | shrink2-rest, rest-zero, proof-gone |
| boundary `a_name_shown` in 09_7 and 09_9 | 09: 13657, 15613 | 09_7-growth, after-control, 09_8, 09_9; closing via the union |
| boundary "1" card re-rooted | 09: 15613 | 09_9-growth-card; closing via the union |

Source facts behind the table:
- `previews()` in the_bump is private, and its only caller is 01_6 (`the_bump.rs:328`).
- No other chapter imports a changed item. counting, closing, continuing and reading_color import only unchanged names.
- `A_NAME_FROM_CENTRE` has the base value (0.18, −0.2), so a's name is unchanged before 09_7.
- At fmn a399d626, `add_to_scene` removes the family and appends it again, then stable-sorts by z_index (`fmn-mobject/src/stage.rs:811–830`). The re-rooted "1" card is therefore drawn once, above the disk, and is not doubled. That is unlike L12's redrawn c, which is a second copy.

One note outside plan02's scope: P1 hashes only these four files. If root's counting candidate rides in a later native build, chapter 11 would need its own window, and P1 would not notice the change.

## Files (my scratch `plan02-review-cc3/`)

| File | sha256 |
|---|---|
| `check-plan02-cc3.py` (superseded) | f0208a452af03e2af57be08b78d360c1c9e38d58ea411051d557cda2c8cc2f8a |
| `check-plan02-cc3-02.py` | 7be3df5097907f3efda79cf5cce02c22b8816564e5da87df82b6e6f219aac9f3 |
| `check-plan02-cc3-02.out` | 7694f91849ed44f37e9983d0d1f1b5b800e588195a9d70d5bbc8fb848a75769c |
| `union-gaps-cc3.py` | c93e5a1a7ad8b235ffe329908a7ce9a8a2f2bbf01cccb52e65245a0087555582 |
| `union-gaps-cc3.out` | 5c9a276fe93010853d90e15192694b4ed7d0633ff255692109377dfc3a184af8 |
| `supplement-cc3.json` | listed in the handoff message |

Inputs read, with the hashes I computed:

| Input | sha256 |
|---|---|
| plan02 `sampling-plan02.json` | bd59ffc6d9347c37e5badf76434b91562d4c0a8a00f3ffed49779559231a64f4 |
| plan02 `rationale02.md` | 6aa628c03c23d1180fe807f30a9190b8424fee8db7ad5b290a9dccbe53045922 |
| `frame-windows01.json` | f41c3d35c0ec3757e347ad6e6ff9d58ddaa22f5a61b87355182ecfe8b892c06b |
| `csd-native855-prior775.json` | 2a1864771d82f9edb436823e100a3c6b97f8421e272a87bd7c2a8c95c1b4cf99 |
| `completed-master993-summary.json` | 38b52105520640e495ada17aa00da4ddbc16316a5a77c4568e102e1ca3adf185 |
| 09 selected timeline | 5a91b8f9e2e5b5d6223a49285c06039b1b43bd16c11cecdd163a8e2e545baa9e |

Plan02's `frames-*.txt` hashes (all / required):
- 01: 89dfb46a / 376f3c2d
- 08: 6b70dab9 / f2093918
- 09: 019e0d9d / aa13f888

## Not done

No build, Cargo, render, decode, still extraction or frame viewing. No listening, fine sync or AV check. No Beads, plan, production or memory edits. Nothing deleted or overwritten.
