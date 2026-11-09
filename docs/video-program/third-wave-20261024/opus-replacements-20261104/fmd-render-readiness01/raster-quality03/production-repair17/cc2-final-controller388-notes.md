# 388: final-verification launch controller (CC2)

cc2, 2026-11-08, for root's review. Q is this folder locally, or `D/qualification01/production-repair17` on CSD, where D is the CSD job root in 388's constant `D` (:35), the same as 377's. In the commands below, `<Q>` stands for that CSD path, spelled out in full.

Nothing has run or been launched on any host. The only checks were source and syntax checks, made locally (§5).

| File | sha256 | State |
|---|---|---|
| `csd-final-controller388.py` (this task) | `5599e007e6b1debf37fe98a22ad1db322896e446a07d3f837fcddcd9e33fb59e` | new; 188 lines; compiles |
| `csd-final-verify377.py` | `827be351861d0fd4ebc083aa24d594111b059dd9f78c7ca43072b0b6f12f22fe` | frozen, used unchanged. CC0 found it in CSD Q at this hash (review 378 §5) |
| `cc0-verifier377-review378.md` | `5354f41f551907e214daa4242b6194dfce7dbcecfbaf644a7f56e9eabba10c3b` | CC0's: no High or Medium; L1 wording (§4) |
| `cc0-combined-master350.py` | `60575944736159139b05733c299b984ab6627ada3d88a5aff65ecde1bbde28d6` | read only; launched by root with root's final 384 manifest |

## 1. What 388 does

Root's direction, as relayed by the user: one controller that uses frozen 377; waits for the actual 350 master's successful, stable receipt; then runs 377's guarded verify, the overview pictures, the external sampling ledger and the native pictures, in that order. It keeps every guard and never-duplicate check, fails on a terminal prerequisite failure, and writes no acceptance booleans.

0. **Refusals before waiting.** 388 refuses before waiting:
   - if 377 is not at `827be351` (checked before its code is loaded, as `csd-launch-final352.py` checks its worker);
   - if 377's names differ from those 388 expects;
   - if any of `csd-final-verify377`, `csd-final-pictures377-overview`, `csd-final-pictures377-native`, their three inputs files, or the ledger already exists.

   It pins its own hash, 377's and `guard_csd02.py`'s at start, and re-checks them on every poll and before every step.
1. **Waiting for the master.** It waits for `D/jobs/csd-combined-master350/resource-receipt.json` to be written `finished`, polling every 30 s, as 350's own `launch()` waits on its rows.
   - A missing receipt is pending, and so is one read while the guard rewrites it in place.
   - A finished failure stops 388 at once, before any 377 job: a refused admission, a non-zero exit, changed inputs, an error or a resource stop (377 `success()`).
   - The finished receipt must read the same 30 s later.
   - CC0 confirms the master job is always `csd-combined-master350`. A failed master cannot be relaunched under 350, so stopping is the right end state.
2. **`check`** (read-only). Its first gate is 377 `stable()` on the master job and every row it bound. 388 keeps the `plan_sha256` it prints.
3. **`launch-verify`.** This is 377's own guarded launch of `csd-final-verify377`; it must exit 0, and its receipt must be a success.
4. **`launch-pictures`, overview.** It must name `csd-final-pictures377-overview` and the plan from step 2, then succeed.
5. **The ledger** (review 339 L1/N1; 363 notes §5 step 3):
   - 388 reads the `sampling.json sha256` from the last line the overview job wrote to its `command.log`;
   - it hashes `sampling.json` read-only and requires a match, with the line's sample and sheet counts and the plan;
   - it writes `Q/final-pictures377-overview-ledger388.json` once (`open('x')`), outside the job folder;
   - it prints the ledger's own sha256.

   The ledger holds hashes and names only: 377's hash; the master, verify and overview receipts; the last line; `sampling.json`; the plan; and the time it was written. It has no flags.
6. **Just before the native launch**, `sampling.json` and the ledger are re-hashed against the printed values.
7. **`launch-pictures`, native.** It must name `csd-final-pictures377-native`, the plan, and the ledger's hash as its printed `overview_sampling_sha256`, then succeed. 388 then requires the ledger hash in the native inputs row for the overview `sampling.json`, and in `union.overview_sampling_sha256`.
8. **Before viewing**, 377's own `part_done()` runs on both parts. Every picture and sheet must be at its recorded sha256, with none missing or added. The last line 388 prints gives the counts and hashes, and says every sheet and native frame awaits root's viewing.

Every launch is 377's own: its gates, its free-space and CPU refusals, its bound inputs and its guard command line are unchanged. Any non-zero exit stops the chain, and nothing is retried. 388 itself writes only the ledger.

**Patterns reused:**
- the wait loop and pins from 350 `launch()`;
- the hash-before-load check from `csd-launch-final352.py`;
- the verify-then-pictures chain from `trj-pictures-after-verify182.py`;
- the ledger steps from 363 notes §5.

## 2. Exact launch, for root

1. **Copy 388 to CSD Q** as a real file, as root staged 379 and 383, and read the two files back:

   ```
   ssh -o BatchMode=yes csd 'cd <Q> && printf "%s  %s\n" 5599e007e6b1debf37fe98a22ad1db322896e446a07d3f837fcddcd9e33fb59e csd-final-controller388.py 827be351861d0fd4ebc083aa24d594111b059dd9f78c7ca43072b0b6f12f22fe csd-final-verify377.py | sha256sum -c -'
   ```

2. **Start 388** detached from the ssh session, with its log protected by noclobber. It may start before or after root's 350 launch, and it waits on the master's receipt either way:

   ```
   ssh -o BatchMode=yes csd 'cd <Q> && printf "%s  %s\n" 5599e007e6b1debf37fe98a22ad1db322896e446a07d3f837fcddcd9e33fb59e csd-final-controller388.py 827be351861d0fd4ebc083aa24d594111b059dd9f78c7ca43072b0b6f12f22fe csd-final-verify377.py | sha256sum -c - && set -C && { setsid /usr/bin/python3 -I -B csd-final-controller388.py --wait-hours 48 < /dev/null > csd-final-controller388.log 2>&1 & }'
   ```

   `--wait-hours` is 0 to 48, the same bound 350 uses. The run needs at least 16 CPUs in its inherited affinity, which 377 checks.
3. **Watch it, read-only:** `ssh -o BatchMode=yes csd 'tail -n 20 <Q>/csd-final-controller388.log'`. Each job's own output is in `D/jobs/<job>/command.log`.
4. **When it ends,** copy the ledger sha256 and 388's last line into root's own records. Then view every sheet and every native frame.

## 3. If 388 stops

- **Before its first 377 launch** (prerequisite pending, a pin changed, the master failed, or `check` refused): nothing has been spent. A wait timeout can be run again. Use a new log name such as `csd-final-controller388-a2.log`, because noclobber refuses the old one.
- **After a 377 job exists:** 388 refuses to start again, on purpose. Root continues by hand with 377's remaining modes (377 notes §4) and the ledger steps in 363 notes §5.
  - Each mode still refuses duplicates.
  - The overview's last line and `sampling.json` stay in its folder for the ledger.

## 4. For root

- **377 wording (CC0 378 L1).** 377's docstring (:8–9), the comment at :661 and 377 notes §2.1 call the 09 rule "07's rule" with a native at the centre.
  - In fact 07's five instants get ±2 samples only, in both 363 and 377. The 09 code does what disposition 377 asks for, with a native at each centre.
  - 377 and its notes are frozen, so this sentence is the correction: "07's sampling rule, plus a native at each centre".
  - Natives at 07's instants would be root's call (CC0: +5 natives, about 0.12 GiB).
- **Facts 388 relies on.** The guard writes the child's output to `command.log` in the job folder and rewrites `resource-receipt.json` in place. That is what the owner guard in this repo does, and `qualify-timeline22.py` reads a CSD job's `command.log`; I have not read `guard_csd02.py` itself. If the overview's last line is not the 377 line, 388 stops before the ledger and before the native job.
- **Manifest name.** CC0 says the final 384 manifest's name is root's choice and not chosen yet. 388 doesn't need it, because 377 reads the manifest path from the master receipt.

## 5. Checks (source and syntax only)

- **Compile.** 388 compiles in memory (`compile()`, `python3 -I -B`).
- **Names.** All 15 names 388 uses from 377 exist at 377's top level: `D`, `Q`, `PYTHON`, `VERSION`, `MASTER`, `MASTER_INPUTS`, `VERIFY`, `VERIFY_INPUTS`, `PICTURE_PARTS`, `GUARD`, `TOTAL_FRAMES`, `read`, `success`, `part_paths` and `part_done`.
- **Loading 377.** Loading 377 runs no I/O. Its top-level calls are `Path`, `dict`, `range`, `set`, `list` and `tuple` only (AST).
- **Parsed output.** The keys 388 parses exist in 377's printed output: `check`, `picture_jobs`, `plan_sha256`, `part`, `job` and `overview_sampling_sha256`. The last-line regex matches 377's format string, rendered.
- **Hygiene.** No local user, temp or scratch paths appear in either file.
- **Not done:** no run, harness or fixture, as root asked.

## 6. Not claimed

No launch, no picture, motion, listening, fine-sync, caption-playback or independent AV acceptance, and no publication authorization. Every sheet and native frame still needs root's viewing.
