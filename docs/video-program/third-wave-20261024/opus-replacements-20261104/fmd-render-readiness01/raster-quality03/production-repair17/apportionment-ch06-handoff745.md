# Handoff 745: rebind the 06_1 hold, then compare all 15 schedules (after build 748)

cc0 (LavenderPike), 2026-11-10, 02:04 UTC. Source preparation only. I used no SSH, Cargo, render or synthesis, made no Beads change or commit, and edited no canonical file. Helper 744 has not run on any host. Helper 731, handoff 732, the failed job `csd-apportionment-ch06-build731`, helper 748 and handoff 749 are all unchanged.

## Result

- **Ready for root's audit, to run only after build 748 has passed.** Helper 744 runs two guarded jobs on the CSD, one after the other: `prepare` (job `csd-apportionment-ch06-prepare744`) and `timeline` (job `csd-apportionment-ch06-timeline744`). They use 664's, 666's and 667's existing files and guard; there is no new framework.
- **Prerequisite, pinned:** a finished `csd-apportionment-ch06-build748` (or `-aN`): exit 0, the same inputs at its end as at its start, run by `apportionment-ch06-build748.py` at `20ebb008…`, with `qualification748.json` showing 33 tests, Clippy, 443 facts, facts and oracle equal to 656, the 15 chapters and the 93 cues. Its workspace must be the exact 722 source: the four candidates at their hashes, the workspace at `WORKSPACE731` (95 files) and the crate at `CRATE731` (49).
- **Why a rebind:** 722 moved the 06_1 → 06_2 prediction hold's marker `let q = think_here(&kit, stage)?;` from line 58 of `not_every_time.rs` (`d3e88f7b…`) to line 102 of the candidate (`04499441…`). Of the map's 18 records (15 holds, 3 `draw.rs` helper anchors), only `visual_holds[5]` names a file 722 changed. The other 17 still match 748's files exactly as recorded, and the job checks that.
- **What prepare changes:** two fields of that one record, so a new map; the unit plan's `direction_contract.map`, which names the new map; and in a copy of 664's narration, three report fields and two assembly fields. It changes no recording, no phrase mark and no timing. It installs 667's four runtime tools beside 748's topic, which handoff 749's fourth `required_before_reuse` item asks for. Nothing is deleted or overwritten: the original unit plan is moved aside under a new name.
- **What timeline shows:** whether `apportionment-native748` with the rebound narration gives 666's schedule for all 15 chapters. That is 45 tables (`timeline.tsv`, `cues.tsv`, `cuelog.tsv`) after the narration path substitution, and 100,886 frames.

| File | sha256 |
|---|---|
| `apportionment-ch06-prepare744.py` | `1fe9efc589fe575ece6a1fe90a31cf6a5156fd52ad2c8db931ad30466a6b0867` |
| `apportionment-ch06-build748.py` (prerequisite, unchanged) | `20ebb00854c16b05fb3a0941ebbc94db6870586d2d8f62b60d1811539439339d` |
| `apportionment-ch06-handoff749.md` (build 748 handoff, unchanged) | `c0752760b6ae7dc31aeb6db5f4d191f0b7d8f54703fe3866e7163523db70283c` |

## Pins the helper holds

| What | Value |
|---|---|
| Old map `docs/narration-direction-map-frederick01.json` (656's copy and 748's) | `d6d21eb2…`, kept in place |
| New map `docs/narration-direction-map-frederick-rebind744.json` | `38209347ef411d0c343342a1b264f3b714cfb4bb059805250a15888c8b89e587` |
| Original unit plan `narration_units.json`, moved to `narration_units.before-rebind744.json` | `d18c1f93…` |
| New unit plan `narration_units.json` | `9bf31d880c03b7362f34d2f21ddaa4546cc32bd86372c58130932a37b511d8a3` |
| 748 workspace after the install (101 files) | `68380f56c5f7fdeeb2583010dcaefe28eb2721378e681f33434c67a62257ae26` |
| 748 crate after the install (51 files) | `037f9166bb6329c7f77e519028040bc14c2604c491a80ea0843aea13b4ead7f2` |
| Tools under `workspace/tools/` (667's record) | `coherent_narration.py` `95615cfa…`, `narration_cache.py` `47331efd…`, `narration_delivery.py` `961f17cc…`, `storyboard.py` `3faed8c1…` |
| Phrase marks, copied byte for byte from 664 | `fc3c86fa…` |
| Script | `b9344e11…` |

I computed both new hashes with the helper's own transforms from the canonical map and unit plan, whose bytes are 656's. Each rebound file must round-trip exactly (`json.dumps(indent=2)` with the original's final newline or none), and each transform asserts the exact set of leaves it changed:

| File | Changed leaves, and nothing else |
|---|---|
| map | `visual_holds/5/line` (58 → 102), `visual_holds/5/sha256` (`d3e88f7b…` → `04499441…`) |
| unit plan | `direction_contract/map/path`, `direction_contract/map/sha256` |
| report | `meta/direction_contract/binding/contract/map/path`, `…/map/sha256`, `meta/units_sha256` |
| assembly | `manifest/units_sha256`, `report_sha256` |

`spans_sha256` still names 566's span file. The report and assembly hashes depend on the CSD paths 664 wrote, so I do not predict them. `rebind744.json` records both.

## What each phase reads and does

Each phase's `inputs` checks everything it can without the guard and writes only the guard's manifest. At its start, `run` requires that manifest to bind exactly what the phase reads.

**Prepare.** It binds:
- 748's receipt, record, manifest, helper and binary;
- every 748 workspace file except `narration_units.json`, which this job moves aside, so the runner's end-of-run re-hash cannot fail on it. 656's identical copy (`d18c1f93…`) is bound in its place, and `run` checks 748's against the same hash.
- 656's receipt and record, and its copies of the old map, the unit plan and `not_every_time.rs` (line 58 checked);
- 667's record, and its tools beside 656's workspace and in 664's repo;
- 666's receipt and manifest;
- 664's narration folder. Every file in it, and every file under 664's repo that its report and assembly name at any depth (receipts, source WAVs), must be at the hash 666 bound.
- the guard, its two pinned files, owned-process-stop02 and `/usr/bin/python3`.

Steps:
1. **sources:** the 17 other records against 748's files, and the marker once at line 102.
2. **rebind:** new files in the job folder.
3. **verify-copy:** `coherent_narration.py verify --require-directions` on the job's copy. It rebuilds every cue's PCM from its retained source span and compares it sample for sample.
4. **install:** into 748's workspace, recorded at once in `install744.json`.
5. **verify-installed** on the installed files.
6. **delivery:** `narration_delivery.py --consumer render --gain-db 0` for all 15 chapters, the call the binary makes before each narrated render. `GAIN_DB` is 0.0 in the kit (`narration.rs:46`); `cli.rs:428–440` passes it and runs `python3 -I -B`, so nothing is written beside the tools.
7. The workspace is checked at `68380f56…` and the crate at `037f9166…`, then `rebind744.json` is written.

**Timeline.** It binds everything prepare bound, but takes 748's installed workspace (all 101 files) and adds:
- prepare's receipt, records, manifest and narration files;
- 666's 45 tables at the hashes `comparison666.json` records, and that record;
- the 15 score QA files;
- `/usr/bin/ffmpeg` and `/usr/bin/ffprobe`.

It runs 666's command for each chapter: `render CH --res 64x36 --fps 60 --lint --narration <prepare744>/narration --out <job>/timeline --crf 16 --preset slow --threads 4`, on CPUs 0–3, with `RAYON_NUM_THREADS=4`, `PATH=/usr/bin:/bin`, cwd 748's crate and one 14,400 s deadline for all 15.
- A render failure stops the job. A table or frame mismatch does not: every chapter runs.
- `comparison744.json` is written only when all 45 tables and all 15 frame counts equal 666's and 748's workspace is unchanged. Otherwise the job writes `mismatch744.json` and exits 1.
- `layout.tsv` is recorded against 666's hash and never stops the job.

## Minimum transfer to the CSD

One new file, under `Q = /data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17`, as a regular file:

| CSD path | sha256 |
|---|---|
| `Q/apportionment-ch06-prepare744.py` | `1fe9efc589fe575ece6a1fe90a31cf6a5156fd52ad2c8db931ad30466a6b0867` |

Everything else is already on the CSD: build 748's job and helper (from handoff 749), jobs 656, 664 and 666, `Q/apportionment-runtime667.json`, `Q/csd-apportionment-timeline666-inputs.json`, the guard and owned-process-stop02.

## Exact invocation

Run each job once, in this order, and only after build 748 has passed. Never run two at once. If build 748 passed as `-aN`, give that name to `--build`.

```
D=/data/projects/jmev-stroke-probe-20261106; Q=$D/qualification01/production-repair17
sha256sum $Q/apportionment-ch06-prepare744.py $Q/apportionment-ch06-build748.py $D/guard_csd02.py $D/qualification01/owned-process-stop02.py
/usr/bin/python3 -I -B $Q/apportionment-ch06-prepare744.py inputs prepare --job csd-apportionment-ch06-prepare744 --build csd-apportionment-ch06-build748
/usr/bin/python3 -I -B $D/guard_csd02.py --phase native --job csd-apportionment-ch06-prepare744 --inputs $Q/csd-apportionment-ch06-prepare744-inputs.json -- /usr/bin/python3 -I -B $Q/apportionment-ch06-prepare744.py run prepare --job csd-apportionment-ch06-prepare744 --build csd-apportionment-ch06-build748
```

Then, after prepare has finished with exit 0:

```
/usr/bin/python3 -I -B $Q/apportionment-ch06-prepare744.py inputs timeline --job csd-apportionment-ch06-timeline744 --build csd-apportionment-ch06-build748 --prepare csd-apportionment-ch06-prepare744
/usr/bin/python3 -I -B $D/guard_csd02.py --phase native --job csd-apportionment-ch06-timeline744 --inputs $Q/csd-apportionment-ch06-timeline744-inputs.json -- /usr/bin/python3 -I -B $Q/apportionment-ch06-prepare744.py run timeline --job csd-apportionment-ch06-timeline744 --build csd-apportionment-ch06-build748 --prepare csd-apportionment-ch06-prepare744
```

Expected `sha256sum` values: `1fe9efc5…`, `20ebb008…`, `f1cae1db…` for guard_csd02 (the copy I read) and `a2fe7985…` for owned-process-stop02. If the CSD's guard differs, please tell me before the launch. `--phase native` is the phase 667 gave 666. The helper refuses any other phase or policy, and refuses a command that is not exactly as above. Each `inputs` prints the manifest path, its sha256 and the row count. I do not predict the row counts; my stand-in prepare manifest had 443 rows.

## If it stops

| Event | Meaning | Action |
|---|---|---|
| `inputs` refuses | Nothing created | Read the message. A missing transfer is fixed by transferring again. Anything else comes to me. |
| Prepare stops before `install744.json` exists (resource stop, guard 75, `verify-copy` failure) | 748's workspace is untouched: no `tools/`, no new map, the original unit plan in place | Preserve the job. After a resource stop, the same helper may run as `csd-apportionment-ch06-prepare744-a2`, with its own `inputs` first. After a `verify-copy` failure, send me its stderr; `-a2` is not the fix. |
| Prepare stops after `workspace/tools/` exists | The install stays in place, complete or partial. `inputs` for any further prepare refuses, and timeline needs `rebind744.json`. | Preserve everything and tell me. The next step is a successor that checks the install against `install744.json` and reruns the recheck. |
| `verify-installed` or `delivery` fails | The installed files do not pass what the binary will run | As in the row above, with the step's stderr |
| Timeline: a render fails | A runtime finding in that chapter | Preserve the job and send me the render's stderr |
| Timeline: `mismatch744.json`, exit 1 | Every chapter ran; at least one table, frame count or the workspace differs | Do not render any final from 748. Send me the record. |
| Timeline: resource stop or guard 75 | 748's workspace and prepare's files are unchanged; the renders write only in the job folder and the system temp folder | Preserve it. The same helper may run as `csd-apportionment-ch06-timeline744-a2`, with its own `inputs` first. |

Stops are 748's, and so 731's: the helper's own group, a cleanup record, then SIGKILL. The timeline renders start ffmpeg in its own process group, which the helper's group stop does not reach. Root has read owned-process-stop02, which guard_csd02 pins: its subreaper custody and pidfds cover every owned descendant, detached ffmpeg included.

## After it passes

- `comparison744.json` is a precondition for root's decisions, not a decision. It does not say that the new 06's frames are the old 06's (review 725 F2).
- A new 06 final must come from `apportionment-native748` with `--narration` set to prepare744's `narration/` folder. 664's narration binds the original unit plan. In the scratch tree, the installed delivery check refused it ("assembly does not bind the supplied script and unit plan") and accepted prepare's.
- The old 06 final from `apportionment-native656` stays preserved.
- Handoff 732's "Later, before any retained render is reused" list still applies.
- **For root, later:** if 722 lands in the canonical tree, the canonical map's 06_1 record needs the same two-field change. Helper 744 makes it only in 748's workspace and its own job folder.

| Chapters | Render | Binary |
|---|---|---|
| 01, 02 | local final632 renders (export 682) | Mac build, not 656 |
| 03–05, 07–15 | CSD final672 renders | `apportionment-native656` |
| old 06 | CSD final672, 4,947 frames | `apportionment-native656`, preserved, superseded only if root accepts the new 06 |
| new 06 | not rendered | `apportionment-native748` with prepare744's narration only |

## What I checked

I ran a scratch proof (`python3 -I -B`, scratch files only; the repo was only read). The scratch tree held:
- 656's real 95 workspace files at the predicted hashes, and 748's workspace as 748 builds it, with the real candidates;
- 664's narration relocated as 664's code relocates it: every referenced file hard-linked unchanged, receipts relocated, the report and assembly in 664's format, `.before-path664` copies;
- 666's 45 tables made from the 567 reference by 666's own path substitution;
- the real `coherent_narration.py` and `narration_delivery.py`.

The binary was a stand-in. It made the binary's script check and its `python3 -I -B` delivery call, then wrote the 567 reference tables with the narration path replaced. Both phases ran through the real guard wrapper and runner (hash-checked against guard_csd02's `PINS`) with guard_csd02's four overrides, as in handoff 749. Only the host-name check and the owned-process-stop02 install were left out.

- **The 664 stand-in** passed `coherent_narration.py verify` as 664 ran it, with 220 hard-linked files.
- **Prepare passed:** guard and receipt exit 0, stable inputs.
  - The manifest left out 748's unit plan, bound 656's copy, all 220 closure files and the relocated receipt.
  - The workspace and crate ended at their pins: the new map, the new plan, the moved-aside original at `d18c1f93…`, the old map untouched, the four tools.
  - All 93 WAVs and the phrase marks were byte-equal to 664's.
  - The report, assembly and map changed exactly the leaves listed above.
  - Both verifies passed, and delivery gave 93 of 93 cues, canonical plan, no chapter scope.
  - All 18 records matched after the rebind.
- **Timeline passed:** 45 of 45 tables equal, 100,886 frames, no layout difference.
  - 15 renders, each with 666's arguments plus `--narration` set to prepare's folder, cwd 748's crate, `PATH=/usr/bin:/bin`, `RAYON_NUM_THREADS=4`.
  - The delivery check passed inside every render.
- **Timeline stand-in defects:**
  - a predict mark moved in 06: all 15 rendered, `mismatch744.json` named `06_not_every_time timeline.tsv`, exit 1;
  - a layout finding in 06 only: `comparison744.json`, exit 0, with 06 listed under `layout_differs`.
- **Refused at `inputs`, nothing created:**
  - timeline before any prepare; a second prepare after the install;
  - the 731 job name as `--build`; prepare given `--prepare`;
  - 748: receipt exit 1 (as 731 ended), inputs that moved, a run by another helper, 32 tests recorded, a changed binary, the old `not_every_time.rs` in its workspace, a changed unit plan;
  - 664: one changed sample in `06_1.wav`, changed phrase marks, a report other than the one 666 bound, one changed sample in a source WAV that the report names;
  - a changed 667 tool beside 656, a changed 656 `not_every_time.rs`, a failed 666 receipt;
  - transforms: a map whose 06_1 record names another marker or another line, a unit plan bound to another map, a file that does not round-trip.
- **The proof caught two faults in my draft, both fixed before sealing.** The map and plan paths were joined to the crate instead of the workspace, so the job would have stopped at its first read. And closure files were bound as found instead of being required at 666's hashes. The last run is of the sealed bytes, `1fe9efc5…`.
- **Shared code:** `run_step`, `end_group`, `on_stop`, `own_group`, `read_receipt` and `check_files` are 748's after renumbering. `guard_binding` is 748's without the Cargo environment checks, because neither phase runs Cargo.
- **Render writes, from the source:** `render` creates only `--out` and the system temp folder (`explainer_kit_sounds`). Build 748's own run asserts that its workspace listing is unchanged after `render all` and `glyphs`, with cwd the crate.
- **compile350:** passed, with no banned text.
- The repo's 566 narration and every hard-linked file were unchanged after the proof.

## Not established

Nothing has run on the CSD, and build 748 has not run. 744 depends on build 748, which has not compiled anywhere. The real manifest row counts and run times are unknown. 666's run time is not in its record, so I give no estimate for the 15 renders.

I assume 664's report names files only under 664's repo, as its relocation record says. Any file it names elsewhere under `D` is bound as found, not compared with 666. I have not seen the CSD's own `guard_csd02.py`, and owned-process-stop02 did not run in my proof.

No audio heard or checked. No picture, motion, fine-sync, AV or release acceptance, and no reuse decision for any retained render.
