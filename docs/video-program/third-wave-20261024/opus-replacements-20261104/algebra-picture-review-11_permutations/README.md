# Chapter 11 picture review and staged repair

LavenderPike, Mail 1788. Frozen for root. Findings are in `REVIEW.md`. **This is an author's review of my own code**, and the patch is mine too.

**Status of the patch: not built, not rendered, no full test run.** Root owns integration and guarded tests.

## Contents

| Path | What it is |
|---|---|
| `REVIEW.md` | Coverage, hashes, findings with anchors, severity, repairs, quiz holds, what passed |
| `PROGRESS.md` | Running notes per sheet batch |
| `evidence/` | Input hashes and per-sheet index |
| `baseline/permutations.rs` | Canonical at delivery, sha `de3c7b10` |
| `candidate/permutations.rs` | Baseline with the repair |
| `patches/permutations.patch` | Baseline to candidate: 32 hunks, 153 lines added, 32 removed |
| `checks/make_candidate.py` | The 34 exact edits, each asserted to match once (rustfmt then applied) |
| `checks/desk_checks.py`, `.out` | Geometry, truth, widths, sentence order, budgets |
| `HASHES.sha256` | Every file here, relative to this folder |

## Applying

Only `explainer_corpus/abstract_algebra_expanded/src/permutations.rs` changes, and only if its sha is still `de3c7b10d96e856ab218d07463a33aa57f97acd41188b1dc267556cfd28e3c68`:

```
patch -p1 --fuzz=0 < docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra-picture-review-11_permutations/patches/permutations.patch
```

If canonical has moved, use `git merge-file` with `baseline/` as the base.

## What it repairs

AD, L, I, AB, M, AA, V, T, X, AG, AH, AI, AE, G, J in full, AC and AM in part. Everything else in `REVIEW.md` is a recommendation only.

**No cue words change.** `script.tsv` is not touched.

Display strings root may want to read:

| Finding | Text |
|---|---|
| I, new | `Transposition: swap two labels` |
| V, new | `Gold: even`, `Blue: odd` |
| AA, new | `Card 2 went from slot 2 to slot 3` (built from the first trail) |
| AC, changed | `By size: 2 fixed, 1 swap, 6 cycles of length 8` |
| AH, changed | `After 3 stages: all data back on their starting lines` (was `every line back where it began`) |
| AG, new | `Stage 4` (from `order + 1`) |

My frozen `algebra/permutation-followup02/storyboard-chapter11.md` and `checks/script_and_captions.py` still print the two old strings. I left them alone. No canonical file outside `permutations.rs` carries them.

New code: helpers `swap_caption`, `shifted`, `link_offsets`, constant `LINK_SEP`, test `long_cycles_part_head_from_tail_and_a_swap_keeps_one_arc`. Shared `exchange` and `pulse` are unchanged.

## Timing

- **AD is the one deliberate push:** 1.2 s, moving 11_25 and all that follows. No word changes.
- X can delay the `sr` picture by up to 0.35 s past its anchor.
- Everything else fits inside its sentence on the scratch voice's word rate:

| Finding | Cue, sentences | Window | Block, before and after |
|---|---|---|---|
| I | 11_5, 0-1 | 5.08 s | 0.40 → 1.70 s |
| I | 11_5, 2 | 8.99 s | 0.80 → 1.20 s |
| T | 11_20, 0 | 7.00 s | 0.40 → 3.50 s |
| V | 11_21, 3 | 6.23 s | 1.20 → 2.20 s |
| AA | 11_22, 2 | 9.76 s | 7.70 → 8.10 s |
| AA | 11_22, 3 | 5.08 s | 3.10 → 3.50 s |
| AE | 11_25, 0 | 5.09 s | 1.40 → 1.90 s |
| AE | 11_25, 2 | 6.66 s | 4.36 → 4.86 s |
| AE | 11_26, 1 | 5.47 s | 0.30 → 0.80 s |
| AI | 11_26, 3 | 5.87 s | 2.50 → 3.00 s |
| AM | 11_27, 1 | 2.72 s | 2.39 → 2.69 s |
| G | 11_14, 3 | 8.99 s | 0.30 → 0.80 s |
| J | 11_17, 4 | 6.64 s | 0.30 → 0.80 s |

**These windows are scratch timing.** Oliver's will differ. Tightest are AM (0.03 s spare) and AA's six-cycle (1.66 s spare). An overrun only makes the next beat start late, but recheck both on Oliver's alignment.

## What was checked

| Check | Result |
|---|---|
| Baseline against canonical, by `cmp` | Equal |
| Patch forward, fuzz 0, against candidate | Equal |
| Patch reversed, fuzz 0, against baseline | Equal |
| `rustfmt --check` through `main.rs` on a scratch copy of `src/` with the candidate | Passes, so every module parses |
| `link_offsets` and its test, extracted verbatim, `rustc --test` on their own | Compiled, 1 passed |
| `checks/desk_checks.py` | No failures |

The desk checks found: no shared head and tail point on any long cycle (was 3, 4 and 6 touching pairs); crossings unchanged at 0, 0 and 1, the one being the six-cycle's unavoidable 5→9 and 6→2; arc tops clear the words above; moved numbers clear links and the written inverse; widths fit; every sentence wait in film order. They caught two errors of their own while I wrote them, both in the check and neither in the patch: 11_7's sentence 2 sits in `witness_proof`, above `construct` in the file, and "reading" matched an old comment.

## What was not checked

- **Type-checking of everything except `link_offsets`.** Signatures were read by hand against the source. Compile errors remain possible.
- The test suite, Clippy, oracle, layout lint and any render.
- Whether the new text clears everything at render. Widths use an estimate of 0.50 em a character. The lint should confirm this, and specifically that "4" is off the layout report.
- How `link_offsets` arcs look. Separation of 0.32 between head and tail was chosen from geometry, not pixels.

## Frames to read on the next preview

| Finding | Cue and moment |
|---|---|
| L | 11_4 sentence 0: "4" and "2" under their rings; lint rows gone |
| M | 11_4 from sentence 2; 11_12 hold; the red arc's head shared with 1→3, two heads visibly at one point |
| AB, AA | 11_22 sentences 2-3: legs single-headed, 4↔7 double-headed, caption clear |
| AD | 11_24 end: clean staircase with "Shuffles: 8" for about 1.2 s |
| I | 11_5 sentences 0-2: the swap, then heading and board |
| T | 11_20 sentence 0: row home by sentence 1, "Swaps: 0" |
| V | 11_21: one colour until sentence 3; keys clear of both blocks |
| AG, AH, AI | 11_26 sentences 3-4 |
| Others | Unpatched `exchange` sites listed in `REVIEW.md` |

Narrated sync, listening, fine sync and final AV stay open. ElevenLabs Oliver remains the narrator; no synthesis or take work was done here.
