# Logarithm remote continuation, mac-mini-max

For root (BlueSalmon), bead 87do.63. This runs rebind, retained-audio assembly, binding, timeline and score after root's remote build, then stops. Render, master, verify and ASR aren't here.

O = `EL/remote-mac-mini-max01`, S = `EL/final-picture-repair01`, B = `narration_lab/third-wave-20261024/root/five-film-release-20261027`.

## Run it

1. Transfer this folder and every path in `transfer-files02.txt` to the same paths under the remote repository. The list is 105 files, 778 MB, repository-relative: the sealed pipeline bundle, the two stage09 helpers, and stage09's timeline and score jobs for the comparisons. `--ignore-existing` is safe here, because every helper is checked by hash or seal.
2. Launch once, in tmux, from anywhere. No wait is needed, because root's build has already finished:

```sh
/bin/bash P/logarithm-remote-mac-mini-max01/remote_continuation01.sh <SHA256SUMS hash> 8f7c4a9f45c4dd256536d8d5992bd14ebef44ac4fc80e5563f84c9de2bce5952 ff1ece5acfa8fe5f2fff52efb683ed76ac1b010330c526d46b64cdc2bfaa808c
```

The arguments are this bundle's `SHA256SUMS` hash, root's approved `integration.json` hash, and root's approved holds hash. If the actual holds hash differs, `bind` refuses.

Exit 0 means every step passed. Any other exit means stop, and no later step ran. The script has no restart logic. A used job name refuses, so a rerun needs a repair first, and earlier outputs must be preserved.

## What it does

| Step | Job in `O/jobs` | Writes |
|---|---|---|
| Rebind | `logarithm-remote-rebind01` | in S: `spans01.json`, `holds-bound01.json`, `narration_units-before-rebind01.json`, `rebind-report01.json`; in the crate: map17 and the unit plan |
| Assembly | `logarithm-remote-assemble01` | `narration01/` and `audio-comparison.json` in the job folder |
| Binding | none | `O/remote-binding01.json`, schema `logarithm-remote-binding-v1` |
| Timeline | `logarithm-remote-timeline01` | as stage09, script in `O/job-scripts`; then `O/schedule-comparison01.json` |
| Score | `logarithm-remote-score01` | as stage09, script in `O/job-scripts`; then `O/score-comparison01.json` |

All guarded jobs go through root's `O/guard01.py`, so limits, lane, target and baseline are root's. ASR's 12 GiB is untouched.

## What differs from the local continuation

- **Freeze:** starts from `O/build-inputs01.json`, which has no `target` rows. The real freeze here has 4,483 rows, all transferred or in this bundle.
- **Unit plan:** root's manifest binds `narration_units.json`, which the rebind must rewrite. A plain copy of the local code would refuse after the rebind. It's dropped from the freeze, and the build gate instead requires the frozen hash to equal the preserved `S/narration_units-before-rebind01.json`.
- **Sealed code:** `remote_pipeline01.py` loads `prepare_final_picture_pipeline.py` by hash and overrides paths in memory, plus `gates()` and `seal()`. The 80-WAV gate, the 15-hold gate, the full 13-chapter tables, the 13 stems and QA reports, and the comparisons all run unchanged.
- **Assembly:** `remote_assemble01.py` is root's `assemble01.py` with the remote job, manifest and output paths.
- **`seal`:** also refuses, before anything launches, if root's manifest spells this tree's paths differently or stage09's jobs haven't arrived.

`superseded-guard-draft01/` is my stopped guard draft, kept as root asked. Nothing uses it.

## Evidence and limits

`checks/run_checks.py` passes under Python 3.14.6, the remote's version; see `checks/outputs/run_checks.out`. It covers parsing, the overrides, the real freeze, Lavender's `--check` (predicting `ff1ece5a`), coherent verify on the 80 retained WAVs, the sealed table gate on stage09, the real gates and waits on tiny files, and the real controller's call order and stops, using a logging stand-in for `python3`.

**There was no end-to-end run.** Root barred large fixtures on the short local disk, so these have never run together, even with stand-ins: guard, rebind `--apply`, assembly, the rest of `gates()`, job writing and the kit. The first remote run is the first test of them. The sealed pipeline's own fixture covered the same sealed gates at their old paths.

This is technical only: no listening, picture review, fine sync or independent AV acceptance. Comparison deltas are reported, not accepted. A move other than the predicted +24 and +420 frames appears in `not_as_predicted` and doesn't stop the score, as in the local design.
