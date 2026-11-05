# ts1 qualifier

WhiteSwan, for root (Mails 5300, 5339, 5348). This qualifies the retained binary from `complex-remote-build01` without compiling. I launched nothing and used no SSH. `O` is `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/remote-ts1-01`.

## Start

Transfer `qualify01/` to the same path on ts1. Then run these with the `/Users` path, not the `/data` one:

```
Q=<this folder, by its logical path>
PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/bin:/bin" python3 -I -B $Q/ts1_qualify01.py --check
bash $Q/start-qualify01.sh
```

`--check` is read-only and spends no job name. If it refuses, send me its last line. The start script runs once, and a second start is refused before it touches the first log. The job is `complex-remote-qualify01`, in `O/jobs`. Its controller files are `O/qualify-controller01.log` and `.exit`, and its manifest is `O/qualify-inputs01.json`.

## What it requires before launching

**The original build, exactly as it ended:**
- The receipt is finished and admitted, with exit 1, no resource stop and no guard error, under the unchanged policy.
- It ran `/bin/bash O/build01.sh`, and `build-controller01.exit` holds 1.
- Its input hashes match each other, the live `inputs01.json` and the pin `29c03e8e…`.
- The log's last line is `ModuleNotFoundError: No module named 'scipy'`, at `oracle.py` line 39, on `import scipy.special as sp`, with one traceback.
- The log shows the lock check, then 207 `ok` tests, then the release build, then clippy, in that order. `build01.sh` runs under `set -e`, so reaching the oracle line means each of those exited 0.
- The only `error` lines are the known `unclosed table` noise, which also appears in healthy local logs.
- `facts-native.tsv` has content, `facts-oracle.tsv` is empty, and `chapters.txt` doesn't exist.

**The binary:** `O/complex_analysis_visual-release01` and `O/target/release/complex_analysis_visual` both hash to `3c90fb54…c168`, the value in Mail 5348.

**The rest:** every row of `inputs01.json` matches live bytes. The oracle env job succeeded and reports numpy 2.5.4 and scipy 1.18.1. The macOS facts hash matches its pin.

The new manifest freezes all of that: the 4323 source rows, the original job's files, both binaries, the controller files and the env, leaving out only its `uv-cache`.

## What the job does

It runs `facts` again and the oracle under the env's Python, keeping both outputs before comparing. Native facts must equal the oracle, the original build's facts and the macOS `release-facts.tsv`, byte for byte. `chapters` must list the exact 15 chapters.

`qualification-summary.json` records `original_build_exit: 1`, `original_build_relabelled: false`, and the original receipt and log hashes. **Downstream stages need this success and the original receipt together. The original job stays a failed job.**

## Paths on ts1

`R` comes from the `guard01.py` row in `inputs01.json`, and `samefile` proves it is this repository. Everything else is built from `R`, so strings stay equal to the manifest rows and the binary's paths. The rehearsal runs with its repository behind a symlink, through the real guard.

## What was tested

`checks/rehearse_qualify.py` passes all 44 checks in a tiny temporary tree. Real parts are the helper, the start script, `guard01.py` logic, the preserved guard, real numpy and scipy imports, and the macOS build log as the fixture log's base. Fake parts are the binary, the receipts and `sbh`. In the fixture copy only, the hostname and hash pins change.

One run went end to end through the real guard. All 31 damaged inputs were refused read-only, and 4 failures stopped inside the job with no summary written.

## Limits

- **The real ts1 log hasn't been read by this helper.** The log rules come from root's report and the macOS log. A difference from them refuses at `--check`, which is safe but would need a repair in a new bundle.
- **A facts difference from macOS stops the job,** with both outputs kept. That's a real signal to inspect, and the binary may still be sound.
- **This proves the binary, not the film.** It isn't timeline, picture, listening or baseline equivalence.
- **Not independently reviewed.** I wrote this helper.

## Replay

```
P=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-remote-ts1-pipeline01/qualify01
(cd $P && shasum -a 256 -c SHA256SUMS)
python3 -I -B $P/checks/rehearse_qualify.py
```

Run these from the repository root on the local host. The rehearsal needs 10 GiB free, because it uses the real guard's admission.
