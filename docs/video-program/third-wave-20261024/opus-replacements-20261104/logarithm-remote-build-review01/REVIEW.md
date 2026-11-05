# Review of root's remote logarithm build

WhiteSwan, for root (Mails 5272, 5275, 5276, 5278, 5287). Independent read-only review of `guard01.py`, `build01.sh`, `start-build01.sh`, `guard-pins01.json` and `build-inputs01.json` in `O`. I wrote none of these, used no SSH, and changed nothing.

`O` is `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01`. `S` is the sibling `final-picture-repair01`, and `C` is the logarithm crate in `S/workspace`.

## Verdict

**There is no safety or correctness blocker in the guard or the build. Leave the remote run as it is.**

One real conflict sits in the frozen manifest. It can't harm the running build unless the audio step starts early, and it will stop the next step if the local code is copied unchanged. The fix belongs in Ruby's continuation, not in any frozen file.

## Findings

### F1. Medium, blocks the next step: the rebind's output is frozen as a build input

`C/narration_units.json` is a row in `O/build-inputs01.json`. The local freeze left it out on purpose, because the audio rebind rewrites it (`rebind_audio.py:636`, `os.replace`). `build-freeze01.json` records that choice as `new_map_and_units_excluded`.

- **If `--apply` runs before the build job ends:** the guard's final input check raises `ValueError`, and a good build is recorded as failed. Waiting for the build's receipt avoids this.
- **If the continuation starts its rows from `O/build-inputs01.json` unchanged:** `audio-after-build01.py:82` refuses with `Rebind output cannot be a frozen input`.
- **If that assert is removed:** the rebind job's own final input check fails after `--apply` changes the file.

**Fix in the continuation:** start from `O/build-inputs01.json` without that one row, and add the new hash after `--verify`, as the local code does at line 87. No other rebind output is a frozen row. The old hash stays bound in the build manifest, and the rebind saves the old bytes as `narration_units-before-rebind01.json`.

### F2. Medium, in effect now: the binary exists before the job has passed

`build01.sh` copies the binary into `O` before clippy, the oracle comparison and the input recheck. Root reports the build is at clippy, so a binary is probably there already, and it would stay if a later step failed. The local script has the same order. No job file records the binary's hash.

**Fix in the continuation:** wait for the terminal guard receipt of `logarithm-remote-build01`, never for the binary file. Then hash it once and pin that hash for every later job.

### F3. Low: the local continuations name the original guard

`audio-after-build01.py:14` and `render-master01.py:18` hard-code `guard_owner56g_target20g.py`. A copy that keeps that path would run outside the remote lane, with a different lock, `five-film-release-20261027` as work and `R/target` as target. Every remote job must call `O/guard01.py`.

### F4. Low, already found by root: the cargo config is unfrozen

`.cargo/config.toml` is in the local freeze, but it isn't in the remote manifest or any transfer list. Cargo reads it from the working directory. The local bytes hash to `79fb8766…`, the same value root read on the remote host (Mail 5287). Its settings are `-Z threads=4`, content fingerprints and checksum freshness, which shouldn't change the output on a fresh target. Record it as a qualification gap and add it to the next freeze.

## Checked and clean

- **Guard overrides:** `main` and `sample` read `ROOT`, `WORK` and `TARGET` when called, so the overrides fully apply. The lock, job folders and baseline all land in `O`. The baseline starts fresh, and the guard never reads `local-baseline-provenance01.json`.
- **Thresholds:** native 10, ASR 12, stop 8, per job 4, work 56 and target growth 20 GiB. Both pinned hashes match.
- **Accounting:** `O/jobs` and `O/target` are siblings and count apart, with no double counting. A missing target counts as 0. The binary sits outside both, as Mail 5276 disclosed, so only the free-space stop covers it.
- **Per-job cap:** the closest real builds grew 0.72 GiB (release test and build, 233 crates) and 1.34 GiB (debug with clippy). Together that's about 2.1 GiB as a rough ceiling, under the 4 GiB cap. That figure is my estimate; the remote receipt will give the real one.
- **Source closure:** all 4308 rows equal the current local bytes. No row shared with the local freeze changed. Of 67 local-only rows, 59 are old target artifacts and 8 are files `build01.sh` doesn't name, one of which is F4's config. Every row is in a transfer list, and every file `build01.sh` names is frozen.
- **Repairs:** the six repaired chapters equal their repair files byte for byte.
- **Toolchain:** pinned nightly, `--locked --offline`, and a target that must start absent. A fresh target also rules out the stale-unit failure of `logarithm-stage09-ws-build01`.
- **Facts comparison:** no earlier logarithm job ran this byte-exact `cmp`. `facts.rs`, `model.rs` and `oracle.py` are unchanged since stage09, and that stage's binary matches the oracle byte for byte over 102 rows.
- **No local duplicate:** no local final-picture job, binary or exit file exists, and I found no matching local process at review time.
- **Next-step files:** all 276 inputs of the rebind and assembly are in the transfer lists.

## Limits

- **Remote state is root's report.** This covers the 4308 remote hashes, `verify_patched_lock` PASS 19, compile progress, the clippy stage and both remote config hashes. I verified none of it directly.
- **The guard's full path didn't run here.** `guard01.py` refused on this host, as it should. I tested the overrides through `sample()` and read `main()`.
- **Not reviewed:** Ruby's continuation, and whether the final source compiles or passes clippy. This source has never finished a build.

A failed build needs new job, target and binary names, because `build01.sh` refuses existing ones. That's by design and keeps the old attempt.

## Replay

```
P=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-remote-build-review01
(cd $P && shasum -a 256 -c SHA256SUMS)
python3 -I -B $P/checks/check_remote_build.py
```

Run these from the repository root. The script passes all 21 checks, and a `found-` line reproduces a finding. It's read-only, apart from one print-only `facts` run of the existing stage09 binary. `local-no-duplicate` stops being valid if root later starts local logarithm work.
