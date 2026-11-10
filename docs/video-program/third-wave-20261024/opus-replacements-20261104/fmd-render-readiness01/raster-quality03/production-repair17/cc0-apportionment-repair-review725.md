# CC0 review 725: apportionment chapter 06 repair722

cc0, 2026-11-10, 00:56 UTC. Independent source review of CC3's `apportionment-ch06-repair722`. Source only: no Cargo, SSH, build, render, synthesis, Beads change or commit. The candidate is uncompiled and no frame of it exists.

## Result

**No blocker, and nothing needs fixing before binding.** The arithmetic, oracle output, year binding, cue timing, `include_str!` path and answer concealment all hold in source. I found three Low items. None changes a number, a cue or the answer's concealment. Compile, tests, Clippy and pictures are still open.

| File | sha256 | Verified |
|---|---|---|
| `handoff722.md` | `0516ec6c1b32b25dc219c027d1b8246c76b46092bccfafc884cdacfdd377fa1e` | Read in full |
| `repair722.diff` | `feb84a349a20572fd2ae4e436f1b32fb1eee2fa20df17b5d3b19c102ef6da710` | `git apply --check` exit 0; patching scratch copies of the canonical files reproduced all four candidates |
| `delta-from-719.diff` | `ada461e305a8fafbaff5af7ec0929c9565db5ce5ed3828a03c32efb34ad261b4` | Patching 719's scene reproduced the candidate |
| `not_every_time.rs` | `04499441ccde5be5f827b9768da26bbd0672f21e283728b298809f827d7225e9` | Read in full against its helpers |
| `facts.rs` | `e9b86e26f7e285ddc397a8948462492c824e77a637a4904e501466a7ece78f1d` | Diff read in full |
| `oracle.py` | `8c1dd3af893059601f1c6a195a3292e7274d796edd4701adfefd6fb3d780c94f` | Diff read in full and run |
| `storyboard.md` | `17a46b19463c07dd4713527574bcb46d56e8dc4dba6b648cc1daac4817f8ffbd` | Diff read in full |

All four `before-*` files equal the canonical files. `script.tsv` (`b9344e11…`) and `docs/description_numbers.tsv` (`f8e46262…`) are at the handoff's hashes, and `git status` shows the canonical crate and `tools/` unmodified.

## Findings

### F1, Low: the narration check is not limited to cue 06_4

`oracle.py` searches all of `script.tsv` for "After the eighteen eighty census", though its error message says "06_4 does not say". The Rust test uses `crate::SCRIPT.contains` the same way.

Tested: I removed the phrase from 06_4 and appended it as cue `99_9`. The candidate oracle exited 0 and printed "the sourced row and the narration agree".

This cannot affect the current film, because `script.tsv` is frozen by hash. If the check is touched again, read only the row whose id is `06_4`, on both sides.

### F2, Low: the handoff understates how the 06_3 tags move

The handoff says the sixth item in the lagged group shortens each tag's fade. It also moves every tag after the first earlier. At the pinned engine (`a399d626`), each start is the previous start plus lag times duration, and the group's local time is scaled to the fixed total. Assuming the group's default linear rate:

| | Each fade | Tag starts within the group (s) | Last tag finishes |
|---|---|---|---|
| Canonical, 5 items | 0.682 s | 0, 0.205, 0.409, 0.614, 0.818 | 1.500 s |
| Candidate, 6 items | 0.600 s | 0, 0.180, 0.360, 0.540, 0.720 | 1.320 s |

The fifth tag arrives 0.098 s (5.9 frames at 60 fps) earlier. The summary line takes 0.90 to 1.50 s, so the group still ends at 1.5 s and no later call moves. This is not a cue change, but frames inside that 1.5 s window will differ from the 700 master for the tags as well as the new line. No source change is needed.

### F3, Low, picture check: Cedar's wedge touches the top tick

Handoff 719 gives a 0.015 gap "before any line cap". The engine's strokes have round caps (`fmn-render/src/stroke.rs` at `a399d626`), so the width-3 tick reaches 0.94 + 0.015 = 0.955. The width-5 wedge at radius 0.98 has its inner edge at 0.955. At the tick's centre line the gap is zero, so expect the two strokes to touch as Cedar's wedge crosses the top.

Chapter 05's Cedar wedge starts at the same point, because Cedar's tail at 15 seats is exactly 0. Existing chapter 05 frames should therefore show this contact. I have not looked at them. Check it at 4K; I would not change the shared `step_arc` or `dial` for it on source alone.

## What was checked

**Maths and oracle.** An exact-fraction mirror, written independently of the oracle:

| House | Whole parts | Tails | Left over | Seats |
|---|---|---|---|---|
| 15 | 1, 3, 10 | .5385, .4615, 0 | 1 | 2, 3, 10 |
| 16 | 1, 3, 10 | .6410, .6923, .6667 | 2 | 1, 4, 11 |
| 17 | 1, 3, 11 | .7436, .9231, .3333 | 2 | 2, 4, 11 |

- The common factor is 1000, giving 4/39, 9/39 and 26/39. Each wedge float equals `POP[i] / total` exactly. Wedge ends are .7436, .9231 and 1.3333.
- Across 99 additions there are five losses, all Ash: 15→16, 23→24, 54→55, 62→63, 93→94.
- Candidate and canonical oracle stdout are byte-identical: 443 lines, `2d622f71…`. `--check` exits 0. The `speed39.*`, `alabama.*`, `whole.17.*`, `hamilton.17.*` and `history.seaton.range_*` values agree with the mirror and the on-screen strings.

**Source-number binding.**

- "+n/39" comes from `POP`, and a test ties the numerators to the oracle-compared `speed39.*` facts.
- "5 of 99" comes from `alabama.count` and `alabama.additions`. "always from Ash" is covered by the existing guard on `alabama.loser`, which is the deduplicated loser set, and that guard aborts the render if it fails.
- "Ash 2, Birch 4, Cedar 11" comes from `model::hamilton` zipped with `NAMES`, in the same order.
- 1880 comes from row 5 of `docs/description_numbers.tsv`. The Rust and Python readers apply the same rules: one four-cell `year` row naming Seaton, year in the context, "census PDF p. 13", and "after the {year} census". `docs/description.md:59` carries the same statement.
- The candidate oracle fails with no TSV, no script, a changed year, or a changed page. The last two were also run without `--check`, which is the mode `facts_check.py` uses. A Seaton row of another kind is correctly ignored, and running from another directory works.
- The real `tools/description_numbers.py` confirms the handoff: canonical exit 0, candidate exit 0, canonical plus a printed `history.seaton.census_year` exit 1 with row 5 "used by nothing".
- As the handoff says, both sides read one row, so this validates the row and the narration; it is not two derivations. I did not re-read Young.

**Timing and cues.** Extracted independently from both scene files, ignoring comments and tests:

- 41 timing calls in the same order, 6 `say` calls unchanged, and every phrase, run time, lag and flag equal.
- The only three text differences are `&[q]` to `&cleared`, `&tags` to `&arrivals`, and `17.0` to `HOUSE_NEXT as f64`.
- New text and arcs are detached until their existing plays, so they add no time. `chars` in the 06_6 reading hold is unchanged.
- The frame count (4947) can only be confirmed by a build. See F2 for the frames inside the tags' group.

**`include_str!` path.** From `src/facts.rs`, `../docs/description_numbers.tsv` resolves to `explainer_corpus/apportionment/docs/description_numbers.tsv`, which exists and is tracked. Exports 652 and 682 list it at `f8e46262…`. `apportionment-build656.py` copies every imported row under `explainer_corpus/apportionment/`, keeping relative paths, and runs `oracle.py` inside that copy, so `docs/` and `script.tsv` sit where the new code expects. I did not see the CSD's actual import.

**Composition, by coordinates; not rendered.**

- 06_1 repeats 05_3's layout: step labels at y -1.05 between the whole parts at -0.54 and the ranking at -2.75, plus the question at 3.3, clear of the dial names at 1.97.
- No wedge reaches the bottom of its dial in 06_1, so none nears the whole-part number. Cedar's wedge passes the top with about 0.1 to spare below its name, by estimate.
- The summary at y -0.95 is centred, and the "adding seat" labels are at the far left and right.
- By character scale from 719's measurement, the longest new line (06_4's range) is about 68% of frame width. It clears the 06_6 note at y 3.3 and the columns at y 1.2.
- The summary and wedges leave through the existing fades. `flash_all` still uses `tags`, so the summary does not flash.

**Prediction concealment.** During the 06_1 hold the frame holds only the 16-seat dials, the 16-seat ranking, the wedges, the step labels, the question and the think ring.

- The 17-seat ranking and the result text are created detached and are not played until after the hold. The hands do not move until "The whole parts become".
- All wedges, labels and the question leave in the "?"'s 0.2 s fade at the start of 06_2.
- The wedge ends do preview the hand positions (719 decision 3). I would keep them. They are the inputs review 715 asked for and the same device as 05_3. The viewer still has to work out that Cedar's whole part rises, that two seats are left, and that Ash ranks second.
- The on-screen question reaches full strength about 2 s before the voice asks it, estimated from 06_1's sentence shares. It contains no answer.

**Likely compile errors.** I read each new call against its definition (`step_arc`, `CENTRES`, `share`, `ranking`, `hamilton`, `kit.text`, `Facts::get`, `SCRIPT`, slice pattern on `&str`, let-else, the test imports) and found none. `rustfmt --check` exits 0 on both Rust candidates, a parse and format check only.

## Decisions for root

1. **Year outside the printed facts:** keep the candidate for this repair. It changes no printed fact and no TSV. The cost is real: the allow-list becomes a compile input, and removing or rewording row 5 would make chapter 06 fail at render, or change its text, while the crate still compiles. The Rust test would catch that. I did not test the handoff's alternative of a printed pin plus deleting row 5.
2. **"/39" labels:** keep them. 05_2 says "thirty ninths" and chapter 05 shows the same labels.

**For binding:** pin `docs/description_numbers.tsv` in the next build's closure, since it now affects the binary. `facts.rs` and `oracle.py` change for every chapter's inputs, so a new binary should be claimed only for what is rendered from it.

## Exact scope

Run, in scratch or read only: every hash above, patch reproduction, the timing extraction, the exact mirror, the canonical and candidate oracle, the faults listed, the real `tools/description_numbers.py` three ways, and `rustfmt --check`.

Read: handoffs 719 and 722, review 715, `dials.rs`, the named `draw.rs` helpers, the model and kit signatures, `apportionment-build656.py`, exports 652 and 682, and group timing and stroke caps at `a399d626`.

Not done: Cargo, compile, Clippy, the Rust tests, `facts-check` as a whole, layout lint, render, or any frame. Young's text and the CSD's imported files were not checked. Line widths and overlaps are estimates. No audio heard or checked. This accepts no picture, motion, sound, fine-sync, AV or release result.

All prior files are preserved. Sealed 716 and stage08 were not touched.
