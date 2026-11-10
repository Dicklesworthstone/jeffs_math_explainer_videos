# Handoff 719: apportionment chapter 06 repair candidate

Author: CC3 (WhiteSwan), from review 715 as accepted by root. Status: staged for root's independent review. Uncompiled and unrendered. Canonical source, script and takes are untouched. The current 700 master remains the comparison baseline.

## Files

| File in this directory | sha256 | Bytes |
|---|---|---|
| `before-not_every_time.rs` (read-only copy) | `d3e88f7bcad146b9078ddc7d84588f6569339cd59cd3b2c225aa963bfadf7c94` | 8493 |
| `not_every_time.rs` (candidate) | `d6c0235a0df000603f30322dade099b240e2f3cc100813c235d3efac5315d64b` | 10328 |

The candidate is a one-file replacement for `explainer_corpus/apportionment/src/chapters/not_every_time.rs`. It needs no change to `draw.rs`, `dials.rs`, `facts.rs`, `script.tsv` or any take.

## Binding of the before copy

Root-provided live evidence, which I did not reproduce and for which I used no SSH: root read the actual build-host final672-06 inputs manifest, sha256 `a1a61ca8882d4cc1017dfb89cc40f411715bc4f75eac604b2f1bec248e0cf0a2`, and hashed the actual assembly664 and build656 copies of `not_every_time.rs`. Both equal `d3e88f7bcad146b9078ddc7d84588f6569339cd59cd3b2c225aa963bfadf7c94`.

My local checks: the before copy and the canonical file both hash to that value and `cmp` reports them identical. `apportionment-export652.json` and `apportionment-local-finals-export682.json` each list the source at that hash and `script.tsv` at `b9344e1123170e7e3898d8aea2123e0b659bcd571085008fc97894e7d69da624`, which is also the current `script.tsv` hash. `git status` showed neither file modified.

## Changes

| 715 finding | Cue | Candidate lines | Change | Carried by existing play |
|---|---|---|---|---|
| F1 | 06_1 | 55-84 | Three `step_arc` wedges and "+4/39 turn", "+9/39 turn", "+26/39 turn" labels, using the same calls and positions as `dials.rs:130-141`, each wedge starting at that town's 16-seat tail. Top line "16 seats → 17: does Ash get its seat back?", size 30, ink, y 3.3. | the dials' 0.6 s fade in |
| F1 | 06_2 | 89-91 | The wedges, labels and top line leave with the "?". | the "?"'s 0.2 s fade out |
| F5 | 06_2 | 99-104 | The result reads "Ash 2, Birch 4, Cedar 11", from `model::NAMES` and `model::hamilton`. | the result's 0.5 s fade in |
| F4 | 06_3 | 150-167 | New line "5 of 99 additions take a seat away, always from Ash", size 24, red, y -0.95. Counts come from the pinned facts `alabama.count` and `alabama.additions`. The existing guard at line 170 covers "always", because `alabama.loser` is the deduplicated set of every event's losers. | last element of the tags' 1.5 s group |
| F2 | 06_4 | 178 | The range line now begins "1880 census, C. W. Seaton:". | the line's existing 0.6 s fade in |

The five-loss line leaves with the strip in the existing `fade_all` at 06_4. The range line stays to the end of the chapter, as it does now. F3, F6, F7, F8 and F9 are not implemented.

## Timing

Each file has 41 timing calls. A script compared them in order, and all phrases, run times and lags are identical. Three planted changes (an added pause, a changed run time, a reworded hook) were each reported, so the comparison can fail. Two calls differ only in the name of their list argument: `&[q]` became `&cleared` at line 91, and `&tags` became `&arrivals` at line 167. The list `first` is also longer at line 84. The 6 `say` calls are unchanged. The expected result is the same 4947 frames and cue spans, which only a build can confirm.

One change inside a group: `fade_in_lagged` fixes the group's total at 1.5 s (`AnimationGroup::with_lag_ratio(..).with_run_time(rt)`), so a sixth element shortens each tag's own fade slightly. The group still ends at the same time.

## Decisions for root

1. The year is a literal. `facts.rs` has six `history.seaton.*` keys and none is the year. 1880 appears only in `docs/description_numbers.tsv:5`, sourced to Young p. 13. Pinning it means changing `facts.rs` and the oracle, which is outside this one-file candidate. The 16 and 17 in the question are also literals, like the existing `ranking(16)` and `hamilton(&POP, 17)` calls.
2. I kept the 06_4 trigger at "worked out Hamilton" to preserve timing, so the strip still holds over "After the eighteen eighty census" and the year and name appear just after they are spoken. Review 715 suggested the earlier trigger. It would add no play time but would move the fade, so I left it out.
3. Each wedge ends exactly where its hand lands at 17 seats (exact fractions: .7436, .9231, and Cedar 1.3333, wrapping to .3333). That previews the hand positions. The viewer must still see that Cedar's whole part rises, that two seats are left, and that Ash ranks second. If root finds that too revealing, removing the wedges and keeping the labels is a small change.
4. `docs/storyboard.md` describes the old picture, including the readout "2, 4, 11", and I did not edit it. `check_video` binds only the storyboard, script and chapter keys, so it cannot see this file's on-screen text.

## Checks run

- `rustfmt 1.10.0-nightly --check`, edition 2024, max width 120: exit 0 on both files. This is a parse and format check only.
- Every new call was read against its definition: `step_arc`, `CENTRES`, `share(..).tail().to_f64()`, `model::total`, `hamilton` returning `Vec<u64>`, and the `fact` closure. The wedge loop matches `dials.rs:128-144`, and it indexes two arrays as that loop does, which should keep `needless_range_loop` quiet.
- `arc_points` advances continuously from `from`, so Cedar's wedge runs past the top, and its 0.667 turn is inside the 1.0 clamp.
- The 06_1 layout repeats 05_3 (same dial centres, label y -1.05, ranking y -2.75, think ring), plus the top line in the slot the result uses later.

Estimated line widths, scaled from the range line measured at 60 characters and size 24 on sheet 4:

| Text | Size | y | Share of frame width |
|---|---|---|---|
| question | 30 | 3.3 | 40.9% |
| named result | 36 | 3.3 | 28.1% |
| five-loss line | 24 | -0.95 | 39.8% |
| extended range line | 24 | 2.6 | 67.8% |

## Not established

No compile, Clippy, unit test, facts check, layout lint, render or picture of this candidate exists. Nothing here is listening, motion, AV or release evidence.

Please look at these first once frames exist:

- Cedar's wedge at the top tick. The tick ends at radius 0.94 and the wedge band starts at 0.955, a gap of 0.015 before any line cap. This is the first step arc in the film to cross the top.
- The top line against the dial names during 06_1.
- The prediction hold, to confirm that no wedge or label remains once 06_2 begins.
- The five-loss line against the "adding seat" end labels.
- The extended range line above the 06_6 note.
