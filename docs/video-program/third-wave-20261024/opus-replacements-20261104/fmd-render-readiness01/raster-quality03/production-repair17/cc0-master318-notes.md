# CC0 master 318: mixed-provenance combined master

cc0, 2026-11-08. Prepared only. Nothing here ran on CSD, CSS, TRJ or HZ2: no launch, job, import, render, master, caption export, permit, deploy, canonical, TODO or Beads change. Paths are relative to D (CSD root `/data/projects/jmev-stroke-probe-20261106`) and Q (`D/qualification01/production-repair17`). `:N` is a line of `cc0-combined-master318.py` unless another file is named.

**Not claimed.** No picture, motion, 4K, fine-sync, caption, AV or publication acceptance. No audio heard or checked. The narration stays the retained ElevenLabs Frederick Surrey takes, bound by bytes only.

| File | sha256 | State |
|---|---|---|
| `cc0-combined-master318.py` (950 lines) | `478b44a5787e12fd495ed6312624a66783f90863cb88774775d1bf0c31a55f57` | new |
| `cc0-master318-notes.md` | this file | new |
| `cc0-combined-master293.py` | `8745c179d42a3ae05f5c47236e673c29dbb020d78da1de04802737a07daa06e5` | base, unchanged |
| `csd-reviewed-boundary-final317.py` (CC2's, not mine) | `19abeee93dbdb98a0217c69b005f05fb18daff716c10406c67ed36fb7210b036` | read for its interface; it was `198a3ab5…` at my first read, and CC2 revised it at 14:00 (now `EARLIER` also has '303'). 318 does not pin it. |
| the nine 313 files | as in `repair313-stage305.json` | unchanged and re-hashed; none edited during 314 |

## 1. What 318 assembles

| Chapters | Row kind | Source |
|---|---|---|
| 09_boundary only | `boundary313` (new) | final317 job `csd-reviewed-final-09-boundary317`, worker `Q/csd-reviewed-boundary-final317.py`, binary 313 from build313, narration rebind313, tables timeline313 |
| 01, 04, 05, 06, 07, 08, 10, 11, 12, 14 | `combined` (as 293) | binary 268 finals: `csd-reviewed-final*` (303, or 287) on CSD, or CSS `css-final-repair*` (309) through a CSD import |
| 02, 03, 13, 15 | `retained` (as 293) | production19 finals pinned to plan183, through import311's CSD imports (`rows293` fields) |

Binary 268 stays the master and caption consumer: `master --narration rebind269 --score stems`, then `captions all` (the 293 body, cwd `D/work/production-repo07`). No chapter is re-rendered. `gates()` (:694) refuses a 09 row of any kind other than `boundary313`, and any other changed chapter that is not `combined`. It also requires exactly one `boundary313` row.

**The 09 row** has the 293 row keys plus `worker_review`:
```json
{"chapter": "09_boundary", "kind": "boundary313",
 "worker": "<Q>/csd-reviewed-boundary-final317.py", "worker_sha256": "<actual>",
 "job": "csd-reviewed-final-09-boundary317", "inputs": "<the 317 launcher's inputs>",
 "permit": "<root's 317 permit>",
 "worker_review": {"path": "<Q>/<independent review of worker 317>", "sha256": "<actual>"}}
```
Hashes come from the files as they are. Nothing in 318 pins binary 313, worker 317 or any final317 output.

## 2. Is binary 268 a valid consumer of a binary 313 chapter? (inspection)

Yes, provided the WAV bytes, the tables and the mixer are all equal. 318 requires each of these. Evidence:

- **Delivery contract.** This is the workspace268 `tools/narration_delivery.py` (`961f17cc…`), which the kit runs before `master` and `captions` (`cli.rs:390–471`). It is path-agnostic for tables:
  - `timeline()` compares each line row's WAV hash with rebind269's `assembly.json` outputs (`:117`);
  - `cue_log()` checks each logged WAV against its logged hash and matches `(sha, start, gain)` entries (`:162`, `:169–181`).
  09's rebind313 paths therefore pass exactly when their bytes equal rebind269's. Prediction holds are counted by `(cue, next_cue)` pairs (`:318–320`), and 09's marks equal timeline269's, so map313's moved hold lines (938→942 and so on) are not read and change nothing here.
- **Kit master.** `master::replay` reads each cue at the path in the cue log and refuses a hash mismatch (`master.rs:88+`). `master_video` null-tests the replay against the last contiguous `soundtrack_N.wav` and prints the replay line only after that passes (`cli.rs:943–984`).
- **Kit captions.** These read rebind269's `report.json` (sentence rows equal to rebind313's) and each timeline's WAV by path (`captions.rs:553`). For 09 those are rebind313 WAVs, which `schedule()` freezes.
- **Why replay should be bit-identical.**
  - Build313's source copy differs from build268's in `boundary.rs` alone, so no kit or score source differs.
  - The direct proofs are the capture comparison before the master (§3) and the master's own null test.
- **What is not a valid check.** A preview capture cannot be compared with a final capture. Previews run without `--score` (`review-repair-preview313.py:970`, `cc3-preview-fade274.py:814`), and their cue logs have no stem rows. The local copies confirm it: preview274's 09 cue log has 10 narration rows, while final08's has three score parts.

## 3. Gates added to 293 (all 293 gates kept)

| Requirement | Where | What refuses |
|---|---|---|
| Exact 313 source delta | `provenance313()` :434 | The two source copies must have the same 70 file names, and only `boundary.rs` may differ, `dc80c541` → `bda7d931` (= `Q/boundary-guide-repair310.rs`). `changed` must be `[boundary.rs]` in both build313 records, and `base_source_copy_sha256` must equal build268's copy. Binary 313's hash comes from `qualification.json` and must equal the binary's bytes, and differ from 268 and 19. Tests, Clippy, fmt, probe, chapters and facts must equal build268's. CONTRACT and `boundary.rs` in workspace313 must be at the copied hashes. |
| 105 WAVs equal | same | rebind313, rebind269 and assembly19 must hold the same file names, and all 105 WAVs must be byte-identical across the three. Only `external-units.json`, `report.json` and `assembly.json` may differ. Report `lines`, assembly `outputs` and manifest `units` must equal rebind269's. |
| Complete 09 tables | same, and `chapter_gate` :659 | timeline313's three tables must equal timeline269's (rebind269→rebind313) and timeline21's (assembly19→rebind313), and comparison.json rows and mappings must match. final317's timeline and cues must equal timeline313's, with timeline313's cue log as the prefix of final317's. 293's `schedule()` (timeline269 mapped) and 317's `verify_tables()` (score rows) also run. |
| Completed 313 jobs | `completed313()` :418 | Each of build313, rebind313, timeline313 and preview313 must have a successful, admitted receipt whose manifest is at the bound hash with every row still at its hash. All rows are frozen into 318's inputs, so the closure comes through build313's manifest. |
| 09 replay before the master | `capture_check09()` :503 | Preview313's 09 capture (binary 313) must be byte-identical to preview274's (binary 268). Their cue logs must be equal under the one mapping, equal to timeline313's, and narration-only. Preview274's movie `dba964d2…` and capture `f7f8aa60…` must be at the recorded hashes (`preview274-transfer305.json`). Any difference is unexplained and refuses before anything is spent. |
| 09 replay in the master | `replays_of()` :819 | The 15 replay lines must come in registry order, each with its staged cue-log row count and frames × 800 samples. For 09 that is binary 268 replaying final317's cue log, null-tested against binary 313's capture. `master-summary.json` records `boundary_replay` and the `chapter_binaries` map. |
| Independent 317 qualification | `independent317()` :527 | The permit's `independent_reviewer` must be non-empty and not its `reviewer`, with `independent_review` files directly in Q at their hashes. The row's `worker_review` must be in Q, at its hash, and contain worker 317's exact sha256. 317's own `check()` is re-run, and its 3-tuple must equal the permit, its hash and the binding. |
| 317 interface and summary | `chapter_gate` :557–628 | The row must use worker and job `JOB317`. The module must have `VERSION` '317', `CHAPTERS` (09,), `HOST` csd, and `BIN`, `NARRATION` and `TIMELINES` equal to the 313 paths. The summary's `host`, `preview_job`, `boundary_sha256` and `build313_qualification_sha256` must equal the 313 records. The permit table is timeline313's. |
| 307 N7 | `contract_reads()` :278, called by `film()` | See §4. |
| 307 L1 | `preflight()` :738, in `launch` | See §5. |
| 307 L2 | `launch` | After the preflight and before `combined-master318-inputs.json` is written, `D/jobs` must have at least 12 GiB free. That is the guard's 10 GiB native admission (`cc3-import-review316.md:108`) plus 307 N6's 1.2–1.7 GB of growth. |

`CONTRACT` gains `tools/narration_cache.py`, which `coherent_narration.py:25` loads beside itself. It must be at build268's copied hash `47331efd…`.

## 4. 307 N7: integrated, because the real consumer reads these files

The evidence that the consumer reads them is root's own preflight. `master-caption-contract305.json` (`dc24242b…`) refused with the wrong cwd on the relative `…/decoded01/decode-receipt.json`. `305b` (`1bc05010…`) passed from `D/work/production-repo07`.

`contract_reads()` mirrors the reads that `narration_delivery.py:254–347` and `coherent_narration.py:84–94, 347–395, 489–564` make with `--narration rebind269`. It resolves relative paths against `REPO07`, as the kit's cwd does. Each file must be at the hash its own record gives:
- the workspace268 `narration_units.json`, which must equal rebind269's `canonical_sha256` (rebind269 rewrote it in place, `review-repair-rebind269.py:68–83`), with the same direction contract as rebind269's external plan;
- the review269 map, inside the topic tree, at both the report's hash and the summary's `map_sha256`;
- every disposition evidence file;
- for each of the 105 units: the record's WAV; its generation receipt (schema `elevenlabs-ui-receipt-v2`, else refuse); exactly one receipt row for the cue, whose `wav` must be the same file; and that row's `source` and `request` at their hashes;
- every report row's `source` WAV.

A structure other than the 105 external UI v2 cues refuses here, so nothing is skipped.

On a mock built from the real assembly19 metadata and the real local receipts and files, 334 distinct paths were frozen. The two receipts were `decoded01/decode-receipt.json` (relative, 103 units) and `audio17/mapped-receipt.json` (absolute, 2 units). The other paths were 105 request files, 14 evidence files, the map, the units file, the summary, and the decoded, relocated and downloaded WAVs. Rebind269 kept these records unchanged from assembly19 (`review-repair-rebind269.py:107`), so the real set should match.

## 5. 307 L1 and L2 in `launch`

Order:
1. wait;
2. pins;
3. `gates()`;
4. `preflight()`;
5. free space;
6. `'x'` inputs;
7. guard.

**The preflight.**
- It makes a new folder, `D/work/tmp/combined-master318-preflight-aN`, using the first free N, so earlier attempts are kept. `D/work/tmp` must already exist; preflight never creates it.
- It puts symbolic links in that folder, under the kit's names, to each row's staged timeline, cues and cue log.
- It runs the workspace268 contract twice, as the kit would:
  - registry order, `--gain-db 0`, cwd REPO07, PATH `/usr/bin:/bin`;
  - `master` with `--score stems`, then `captions` without it.
- It writes `preflight.json` with `'x'`.
- It requires exit 0, the consumer named, 105/105 cues and plan mode `external-ui`. That file is frozen into the inputs.
- This goes further than root's 305b, because it includes the stem rows and 09's rebind313 tables. No native binary runs.

**What a refusal costs.** A refusal at steps 3–5 writes no inputs file, so 318 is not spent.

**What still spends 318.**
- The guard's own admission refusals: another native job, or a guard condition I cannot read. Root should confirm no native job is running.
- A null-test failure inside the master. The kit has no replay-only command (`cli.rs:360–376`). But `master_video` replays all 15 chapters before it writes anything (`cli.rs:943–984`), so that failure spends only the job name.

`check` stays write-free and does not run the preflight.

## 6. Local mocks (scratch, outside the repo; not host runs)

**`mock_contract318.py` (N7, real data, 9 cases, 9 as expected).** This is the good case above, plus 8 refusals:
- units drift;
- receipt schema;
- request hash;
- missing cue row;
- evidence hash;
- summary map hash;
- report source hash;
- unit order.

**`mock_suite318.py` (45 cases, 45 as expected).** It runs synthetic CSD roots, with real `boundary-guide-repair310.rs` and the real preview274 09 movie and capture. A fake 317 module stands in under 317's file name.
- **`provenance313`.** The good case, plus 10 refusals:
  - a second source delta;
  - boundary not repair310;
  - a rebind313 WAV differs;
  - report lines differ;
  - an extra file;
  - timeline21 differs;
  - a 313 manifest row mutated;
  - binary 313 bytes;
  - a receipt not admitted;
  - binary 313 equal to 268.
- **`capture_check09`.** The good case, plus 4 refusals:
  - one byte differs;
  - stem rows present;
  - cue logs differ;
  - non-contiguous captures.
- **`chapter_gate` boundary313.** The good case, plus 9 refusals:
  - kind `combined`;
  - wrong chapter;
  - no `worker_review`;
  - a review that does not cite the worker;
  - independent reviewer equal to the reviewer;
  - module version 303;
  - summary boundary hash;
  - tables from timeline269 instead of timeline313;
  - another job name.
- **`gates` kinds.** The good case, plus 4 refusals:
  - 09 as combined;
  - 01 as boundary313;
  - 09 retained;
  - a second boundary313.
- **`replays_of`.** The good case, plus 5 refusals:
  - 09 cue count;
  - 09 samples;
  - 09 missing;
  - order;
  - an unparsed line.
- **`preflight`.**
  - Missing `D/work/tmp` refused.
  - In the good case, the argv, cwd, `--score` for master only, registry order and the six links are as intended.
  - A refused contract fails the preflight, and both attempt folders are kept.
- **`launch`, with a fake guard and stubbed `gates()`.**
  - An L2 refusal writes no inputs and no job.
  - An L1 refusal writes no inputs.
  - In the good case, the inputs freeze `preflight.json`.
  - A second launch is refused.
- **The real 317 worker's interface.** As 318 reads it, this holds at `19abeee9…`, rechecked after CC2's revision.

**Limits.**
- The mocks run no binary, guard, ffprobe or 317 `check()`.
- The 293 parts unchanged in 318 (`film()` beyond N7, `schedule()`, `imported()`, `retention()`, the retained and combined branches, the master body's ffprobe gates) were not re-mocked. 307 reviewed them in 293.

**Static checks.** `compile()` passes under `python3 -I -B`, and the hygiene scan is clean.

## 7. Prerequisites and decisions for root

1. **Root's own steps.** Root runs build313 → rebind313 → timeline313 → preview313 under 313's launchers (recipe313 is now in Q), then root's actual picture and motion review, then the 317 permit with an independent review, then the final317 job.
2. **Worker review.** An independent review file of worker 317 must sit in Q and contain its exact sha256, for the row's `worker_review`. Root's `final303-root-review305.json` style (its `files` map) satisfies this.
3. **Other rows.** The ten other changed chapters need finished 268 finals: 303 or 287 on CSD, or 309 on CSS through a CSD import. The four retained rows need import311's three CSD imports (`rows293` fields; review 316 accepted).
4. **Before `launch`.** Confirm that no native job holds the lane. `launch` itself checks the 12 GiB floor and the preflight.
5. **317 R1.** An existing 303 09 job stops 317's `check()`, and with it 318's 09 gate. That is root's decision, as CC2 notes.

## 8. Still open after a successful 318

- full 4K decode;
- exact captions;
- the master's pictures and motion;
- joins;
- loudness and true peak;
- fine sync;
- independent AV acceptance;
- publication306's final-master binding.

No audio heard or checked, here or by any check above.
