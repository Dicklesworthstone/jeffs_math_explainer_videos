# Handoff 749: guarded CSD build 748, the successor of 731

cc0 (LavenderPike), 2026-11-10, 01:56 UTC. Source preparation only. I used no SSH, Cargo, render or synthesis, made no Beads change or commit, and edited no canonical file. Helper 748 has not run on any host. Helper 731, handoff 732, the 731 input manifest and the failed job `csd-apportionment-ch06-build731` are all preserved.

## Result

- **Ready for root's audit and one launch.** Helper 748 is helper 731 with one guard check corrected. It builds the same four 722 candidates on the same 656 closure, with the same pins, steps, pass conditions and stop handling.
- **What stopped 731:** root's run of `csd-apportionment-ch06-build731` finished at 01:41:09 UTC with exit 1, stable inputs and no resource stop. It stopped at `guard_binding` (line 476), before Cargo. 731 required the guard's `CARGO_TARGET_DIR` to be `D/work/target`. guard_csd02 sets it to `D/work`, so that the whole work tree counts as target allocation.
- **Whose error:** mine. 716 compared the guard's value with `work` (`guard_target=work`) and pointed Cargo at `work/target`. In 731 I compared the guard's value with Cargo's target. My 731 proof wrote its own receipt and environment with the same wrong value, so it could not catch this. The 748 proof runs the real guard code.

| File | sha256 |
|---|---|
| `apportionment-ch06-build748.py` | `20ebb00854c16b05fb3a0941ebbc94db6870586d2d8f62b60d1811539439339d` |
| `apportionment-ch06-build731.py` (base, unchanged) | `8963f280589090c524137465694fc82aabe8d6608f382c6d6e4155bb412d4b00` |
| `apportionment-ch06-handoff732.md` (base handoff, unchanged) | `c4624178996e5a923428b8043299e7b91d7a36e01b523cd011f0f0735666c605` |
| `cc2-apportionment-build-review741.md` (review of 731, no blocker) | `8c4118129a638947dd3ae0651a74b6a6cac6eeaf50b84b1a8a103531d1d2bd1f` |

Review 725's three Lows stay open (F1 narration check not limited to 06_4, F2 06_3 tag starts, F3 Cedar's wedge), and so do review 741's L1 and L4.

## The whole change from 731

| Where | 731 | 748 |
|---|---|---|
| `guard_binding` | `CARGO_TARGET_DIR == D/work/target` | `CARGO_TARGET_DIR == D/work`, with `D/work/target` directly inside it. The guard block of `preflight748.json` also records `guard_target_dir`. |
| Names | `build731`, `preflight731.json`, `apportionment-native731`, `qualification731.json`, `handoff732`, schemas and refusal messages | the same with 748 and 749 |
| Record text | `required_before_reuse` has 3 items | A fourth item: the four runtime tools 667 installed must be under this job's `workspace/tools/` before any `--narration` render (review 741 L3). |
| Docstring | | A paragraph on the successor, and a sentence on owned-process-stop02 from root's reading (review 741 L2) |

Everything else is 731's:

- the sources, and the pins `SOURCE656`, `WORKSPACE731`, `CRATE731`, `PRISTINE_ENGINE`, `CARGO_CONFIG` and `ORACLE_SHA`, which keep their 731 names;
- the tests and the toolchain checks;
- every step and its pass condition, which are handoff 732's "Steps and what each must show" with the 748 names;
- `end_group`, `on_stop` and `run_step`.

`run()` still sets `CARGO_TARGET_DIR=D/work/target` before the first Cargo step, as 731 did. Cargo therefore writes inside the tree the guard counts.

## The guard contract 748 now checks

I read the local `guard_csd02.py` (`f1cae1db…`) and the two files it pins by hash, which are at those hashes here: the wrapper `guard_owner56g_target20g.py` (`1c19a9b8…`) and the runner `run_bounded_phase_oliver_owner48g_target14g_20261102.py` (`819746fb…`).

- guard_csd02 sets the runner's `ROOT` and `REPO` to `D`, `WORK` to `D/jobs` and `TARGET` to `D/work`.
- The runner creates `D/jobs/<job>` and writes the receipt with `admitted`, `phase`, `command`, `preflight_only`, `policy` (10/8/4/56/20/10) and `inputs_sha256`. It then opens `command.log` exclusively and starts the command in a new session, with cwd `D`.
- The command's environment adds `RCH_SHIM_LOCAL_IDE=1`, `RUSTUP_TOOLCHAIN=nightly-2026-08-31`, `CARGO_TARGET_DIR=D/work`, `CARGO_BUILD_JOBS=2`, `RAYON_NUM_THREADS=4` and `FW_ENC_ATTN_OUT_I8I32=0`.
- It re-hashes every manifest row at the start and at the end.

748 checks `CARGO_TARGET_DIR=D/work`, `RUSTUP_TOOLCHAIN`, `CARGO_BUILD_JOBS=2` and `RAYON_NUM_THREADS=4`. 731 never reached the last three on the CSD. They match the runner's own environment line.

## Minimum transfer to the CSD

One new file, under `Q = /data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17`, as a regular file:

| CSD path | sha256 |
|---|---|
| `Q/apportionment-ch06-build748.py` | `20ebb00854c16b05fb3a0941ebbc94db6870586d2d8f62b60d1811539439339d` |

The four 722 candidates that 731's transfer placed under `Q/apportionment-ch06-repair722/` are read again at the same hashes (`04499441…`, `e9b86e26…`, `8c1dd3af…`, `17a46b19…`). Everything else comes from `jobs/csd-apportionment-build656/`, exactly as handoff 732 lists.

## Exact invocation

Run once, only when no other Cargo job is using `work/target`, and never duplicate it.

```
D=/data/projects/jmev-stroke-probe-20261106; Q=$D/qualification01/production-repair17
sha256sum $Q/apportionment-ch06-build748.py $Q/apportionment-ch06-repair722/not_every_time.rs $Q/apportionment-ch06-repair722/facts.rs $Q/apportionment-ch06-repair722/oracle.py $Q/apportionment-ch06-repair722/storyboard.md $D/guard_csd02.py
/usr/bin/python3 -I -B $Q/apportionment-ch06-build748.py inputs --job csd-apportionment-ch06-build748
/usr/bin/python3 -I -B $D/guard_csd02.py --phase native --job csd-apportionment-ch06-build748 --inputs $Q/csd-apportionment-ch06-build748-inputs.json -- /usr/bin/python3 -I -B $Q/apportionment-ch06-build748.py run --job csd-apportionment-ch06-build748
```

For `guard_csd02.py` I expect `f1cae1db…`, the copy I read. If the CSD copy differs, please tell me before the launch, because 748's checks follow the copy I read.

`inputs` prints the manifest path, its sha256 and the row count. Expect 4,164 rows, as 731's manifest `f36ae00a…` had, with 748 in place of 731 as the helper row. It also prints the sha256 of any binary already at `work/target/release/apportionment`. The guard command must be given exactly as above.

## If it stops

| Event | Meaning | Action |
|---|---|---|
| `inputs` refuses | Nothing created | Read the message. A transfer problem is fixed by transferring again. Anything else comes to me. |
| Guard returns 75, or a resource stop | Admission or resource limit. The job folder is kept. | Preserve it. The same sealed helper may run again as `csd-apportionment-ch06-build748-a2`, with its own `inputs` first. |
| A guard-binding refusal | A guard contract I have still misread | Preserve the job and send me the last line of `command.log`. The fix is a successor, not `-a2`. |
| Test, Clippy or build failure | A source finding for CC3. 741 L1: strict Clippy over the 30 old tests runs here for the first time. | Preserve the job. New candidate hashes need a successor helper. |
| Facts or oracle differ from 656 | Contradicts 722's "no printed value changes" | Do not proceed. Send the stdout files to CC3 and me. |
| Cue or glyph probe failure | A runtime finding in a scene | As for a test failure |

Stops are 731's, so they are 521's and 716's too: the helper's own group, a cleanup record, then SIGKILL. Root has read owned-process-stop02, which guard_csd02 pins: its subreaper custody and pidfds cover every owned descendant, the cue probe's detached ffmpeg included.

## After it passes

Handoff 732's "Later, before any retained render is reused" list still applies, with `apportionment-native748` in place of 731. `qualification748.json` records all four items, including the runtime tools. My successor task 744/745 (guarded `prepare` and `timeline` jobs) will take build748 as its prerequisite, pinned to `20ebb008…`.

## Provenance kept separate

| Chapters | Render | Binary |
|---|---|---|
| 01, 02 | local final632 renders (export 682) | Mac build, not 656 |
| 03–05, 07–15 | CSD final672 renders | `apportionment-native656` |
| old 06 | CSD final672, 4,947 frames | `apportionment-native656`, preserved, superseded only if root accepts the new 06 |
| new 06 | not rendered | `apportionment-native748` only (731 built nothing) |

## What I checked

I ran a scratch proof (`python3 -I -B`, scratch files only). The scratch tree held the real 95 workspace files at the predicted hashes and the real candidates. Cargo, the binary and the engine were stand-ins.

- **Through the real guard code:** I ran the real wrapper and runner (hash-checked against guard_csd02's `PINS`) with guard_csd02's four overrides. These were checked against its text, then applied. Only its host-name check and the Linux-only owned-process-stop02 install were left out.
  - 731 reproduced root's stop: guard exit 1, receipt exit 1, stable inputs, no resource stop. The job folder held only `command.log` and the receipt, and the log ended `AssertionError: CARGO_TARGET_DIR is not the guard value`.
  - 748 passed: guard and receipt exit 0, stable inputs, `qualification748.json` with 33 tests and 443 facts. `preflight748.json` shows `guard_target_dir` = `work` and Cargo's `CARGO_TARGET_DIR` = `work/target`. The binary was copied from `work/target/release`.
- **Direct launches, environment as the runner gives it:**
  - **Full pass:** 33 tests, 443 facts, 93 cues said in order, 17 modules, workspace and crate at their pins.
  - **Wrong target value:** `CARGO_TARGET_DIR=work/target` (731's assumption) and another value were both refused at `guard_binding`.
  - **Refused after launch, as for 731:** every mutation in handoff 732 "What I checked", with no `qualification748.json` written:
    - tests: 32 tests, a renamed original test, a replaced new test;
    - Clippy failure;
    - facts: a changed fact, facts reordered;
    - a missing chapter, a changed script;
    - cue probe: an unsaid cue, swapped cues;
    - glyphs: a glyph failure, a missing 06 string;
    - receipt: a stale policy, a job folder that is not fresh, another command.
  - **Refused at `inputs`:** a changed candidate, 656's receipt, a workspace file, its tests, its oracle output and its binary. The real file refuses on this Mac (not the CSD) and refuses the 731 job name.
  - **Stops:** SIGTERM and a timeout each ended the group, wrote `tests.cleanup.json` and left no grandchild.
- **Source comparison (AST, after renumbering):** only `guard_binding` and `run` differ from 731, `run` only in the record text above. Of the module constants, only `JOB_NAME` differs. The 716 functions 731 shared are unchanged.
- **compile350:** passed, with no banned text.

## Not established

Nothing has compiled. The tests, Clippy and both probes have not run on these sources. The CSD run time is unknown. I have not seen the CSD's own `guard_csd02.py`, only the local copy, and owned-process-stop02 did not run in my proof. Glyph pages will need reading, and the cue probe's timing is estimated. No audio heard or checked. No picture, motion, fine-sync, AV or release acceptance, and no reuse decision for any retained render.
