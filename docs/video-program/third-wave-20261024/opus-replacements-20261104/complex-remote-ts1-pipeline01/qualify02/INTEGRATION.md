# Stage07 build on ts1

WhiteSwan, for root (Mails 5612, 5610, 5721, 5731). `O` is `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/remote-ts1-01`. I launched nothing and used no SSH.

`ts1_qualify02.py` is one guarded job, `complex-remote-build02`. It runs the steps of root's `build01.sh` on the stage07 crate, in the same order, and adds what that job never reached: the oracle under its isolated env, both facts comparisons and the chapter registry. A clean run needs no separate qualifier job. `build01`'s failed evidence is left alone and isn't read.

## Start

Transfer this folder and the stage07 tree, then run by the `/Users` path:

```
Q=<this folder, by its logical path>
PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/bin:/bin" python3 -I -B $Q/ts1_qualify02.py --check
bash $Q/start-build02.sh
```

`--check` is read-only and spends no name, and it can run at any time. It prints `chain02_ended`, so true or false are both fine here. Run it first: `start-build02.sh` creates its log before anything else, so a start that fails its first check can't be repeated under the same log name.

`start-build02.sh` runs the same checks, waits for `O/chain-pictures-controller02.exit`, however chain02 ended (Mail 5610), repeats the checks, freezes `O/build-inputs02.json` and starts the guard. It never touches a running chain02 job.

## What it needs and leaves

| Name | Role |
|---|---|
| `EC/native-stage07-root01` | the tree `frederick_intake_stage07.py tree` writes. The five sealed repair files are pinned here, and the tree's file set must equal `tree-receipt.json`. |
| `O/complex_analysis_visual-release01` | old binary, kept. Every stage re-hashes it. |
| `O/target` | **overwritten** by the build, which is why the job waits for chain02 |
| `O/complex_analysis_visual-release02` | the new binary, copied before anything judges it. It's a separate file from `target`, and clippy must not change `target`. |
| `O/jobs/complex-remote-build02` | `command.log`, facts, `chapters.txt`, `binary.sha256`, `build-summary.json` |
| `O/build-inputs02.json`, `O/build-controller02.{log,exit}` | manifest and controller files |

## Passing means

207 tests pass with each one listed `ok`; fmt, build and clippy `-D warnings` run in order; the lock check passes; the binary differs from the old one, which it must because the script compiles into it; native facts equal the oracle and equal macOS's `release-facts.tsv`; and the registry lists all 15 chapters. `binary.sha256` and `build-summary.json` are written last, so a failed step leaves neither.

207 is the count from the old build. Ruby's count of `#[test]` attributes is the same in both trees, 260 in total and 7 in `boundary.rs`. This is an expectation until the job runs.

## Changes from Ruby's review (Mail 5721) and root's rulings (Mail 5731)

- **C1.** The chain's pictures stage reads this job under its own python. `check_outputs` compares the command tail exactly, and the receipt's interpreter must be a frozen row of the build manifest, by path and hash. So a reader's interpreter doesn't matter, and the build's still does.
- **C3.** The child's first receipt read retries the half-written race, as the chain does.
- **C4.** `--check` no longer needs chain02 to end. `--launch` checks before the wait and hashes the exit file after it.

## Disk

Ruby can't measure `work_bytes` on ts1, and neither can I. chain02's and chain03's outputs share `O/jobs` against the 56 GiB work cap. A guard refusal spends the job name, so please check this before the build.

## What the rehearsal shows

`checks/rehearse_build02.py` runs a tiny tree whose repository path is a symlink, as on ts1. `checks/outputs/rehearse-build02.txt` is its output: 24 checks, 0 failed, the same bytes on a second run.

- **Real:** the helper, its start script, ts1's `guard01.py` logic and the preserved guard, numpy and scipy, and the five stage07 files.
- **Fake:** rustup and cargo, the lock tool, the binary, sbh and chain02's exit file. Nothing compiled.

The stops it proves: fmt, tests, clippy, a missing test, the old binary again, facts differing, the target moving during clippy, a changed tree, an extra tree file, a changed old binary, another interpreter or command in the receipt, and a second start.

## Not claimed

No compile has happened. No timeline, picture, listening or baseline equivalence claim comes from this job.
