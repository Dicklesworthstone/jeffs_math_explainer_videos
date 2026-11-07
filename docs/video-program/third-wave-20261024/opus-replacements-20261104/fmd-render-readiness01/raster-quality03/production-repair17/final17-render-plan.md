# Production17 preview and final rendering handoff

Root is the consumer and owns launch, actual picture/motion review and publication. These helpers are prepared locally only; no remote job was launched. Retire this plan when the production17 package is qualified or superseded.

## Prepared executable path

The two new scripts preserve the existing CSD resource policy: admission 10 GiB, stop below 8 GiB, phase growth 4 GiB, work cap 56 GiB, target growth 20 GiB, ten-second sampling. They enter the existing `guard_csd02.py`, refuse existing job/output paths, retain stdout/stderr and freeze dependencies. They neither create review permits nor import the old all15 acceptance gate.

Once the production17 build and assembly have terminal exit0, invariant-input receipts, stage these two reviewed helper files under `/data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17/`. Root runs on CSD:

```sh
/usr/bin/python3 -I -B /data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17/launch-preview17.py --phase timeline
```

This creates job `csd-production-timeline17`, generates all15 native timelines at 60fps and produces `timeline17-summary.json`: per-chapter frame counts, cumulative start/end frames, new total frames and duration, and all45 table hashes. The old total117697 is never used. It compares the old/new tables with only the explicitly recorded old narration directory to new narration directory substitution. That comparison is schedule evidence, not a media-reuse permit.

Assembly contract, coordinated with complete_native08:

- Job: `jobs/csd-production-assembly17`; narration: its `narration/`; summary: `assembly17-summary.json`.
- `script_sha256` binds the new topic script; `units_sha256` binds `narration/external-units.json`; `assembly_sha256` and `report_sha256` bind those narration files; `adapter_sha256` binds `TREE/tools/coherent_narration.py`; `build_binary_sha256` binds the guarded build output.
- Default assembly input manifest is `Q/assembly17-inputs.json`. If root names it differently, pass its exact path with `--assembly-inputs`; the launcher requires its hash to equal the terminal assembly receipt's input hash.
- Both phases rerun native `narration_delivery.py` for all15 and require105 verified/selected cues. Canonical plan metadata must already be rebound by the authorized assembly workflow; these helpers do not alter it.

After the timeline receipt is terminal, root runs:

```sh
/usr/bin/python3 -I -B /data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17/launch-preview17.py --phase preview
```

Default fresh previews are exactly03,05,08,09,11,13,14. Explicit repeatable `--chapter <registry-key>` options can select a different set if an unchanged candidate fails equivalence. The new job is `csd-production-preview17`; outputs are native960x540p60 H.264 CRF18/veryfast with the assembled narration and no reused music stem. Every new timeline, cue and cuelog table must match the new timeline pass exactly; each movie must have the derived frame count and pass software decode. The helper extracts five-second progression contact sheets and full-size cue/phrase pictures directly from each native movie. It also requires the terminal production16 preview/picture receipts, exact old layout hashes and bound extraction record; samples every current, production16 and prior12 layout anchor at nearby offsets; preserves all production16 selected timestamps and adds cue-relative remaps for shifted chapters. Repeated-layout findings trigger five-second recurrence samples through chapter end. The aggregate later-look count is not converted into a duration or assumed consecutive. Historical/remapped anchors are labeled as historical evidence requiring new viewing, especially inside replaced08_5/09_1 cues. It writes motion-review targets referring to the full native movies; all reviewed/accepted flags remain false.

## Fast lane: eight conditional unchanged finals

Follow `unchanged-final17-reuse-review.md`: candidates01,02,04,06,07,10,12,15 can render provisionally with frozen production16 without waiting for the repaired proof previews. Actual applicable prior selected-frame/motion observations must exist for each selected chapter, with no outstanding defect for it. A new partial permit names only those chapters and those exact observations. Do not change or forge the existing all15 gate.

Root can begin those useful candidates in parallel with CSD production17 timeline/preview work on the separate qualified TRJ host. Promotion later requires, individually: exact source/dependency closure including cross-chapter imports; per-cue script/audio/PCM/report/phrase/sentence/direction identity; all three schedule tables equal after only validated filename mapping; identical score stems/parts/sample coverage; production17 master replay exactly matching the retained native PCM; matching video encoding parameters/extradata/timebase and verified joins. Preserve provenance as render_binary16 and master_binary17. If a condition fails, retain the candidate and rerender that chapter with17.

Chapter13 remains in the new-preview set because its review issue is unresolved, regardless of whether its source file changed. Chapter14 source changed and cannot be called unchanged. Global glyph registry changes must be checked for runtime effects; a registry-only change is not by itself proof of a scene change or proof of equivalence.

## TRJ parallel 4K route

The local copy of `trj-worker01/guard_trj01.py` preserves the owner guard and descendant cleanup. Its underlying guard takes one `expanded-native-lane.lock`. Separate concurrent invocations will queue, so do not create extra guard roots or bypass the lock to obtain parallelism.

The safe implementation path for root is one guard-admitted aggregate child, launching a bounded set of disjoint chapter renders beneath that same monitored process tree, each with fresh output/capture directories and frozen inputs. Start with two chapters and separate CPU subsets; use the existing tested16-thread native path per process if measured host headroom supports it. Keep every child/ffmpeg descendant under the existing stop mechanism, test interruption of the aggregate tree, and increase concurrency only from measured headroom. The existing aggregate disk-growth/free-space limits remain binding; queue remaining chapters rather than weakening them. No aggregate launcher or changed guard is supplied here.

The native CLI supports chapter selection, not an arbitrary start/end-frame interval. Use complete chapters as parallel chunks. `--max-frames` is a profiling stop that publishes nothing, not a chunk exporter. Internal chapter splitting would require separately reviewed scene/driver work and is not hidden inside this plan.

Each eventual final native command uses the existing pattern, with exact validated paths and chapter key:

```text
production16-or-17 render CHAPTER --res 3840x2160 --fps 60 --threads 16 --narration FROZEN_NARRATION --score QUALIFIED_STEMS --out FRESH_CHAPTER_DIR --lint --crf 16 --preset veryfast
```

Keep one encoder/configuration across all mixed-provenance chapters. TRJ's existing111-frame silent4K/1fps parity test measured one chapter04 diagnostic and production16 only; it does not qualify narration,60fps motion or all chapters. Bind the actual final encoder/tool identity, perform actual narrated/native60fps checks and preserve the current worker stop evidence. Frozen production16 candidates do not establish production17 host parity automatically.

## Changed chapters and final assembly

Recompute music from the new15 timelines in a separately guarded scoring phase; do not lay old08/09 stems against lengthened narration. Use the existing `production17 score --narration N --timelines NEW_TIMELINES --out FRESH_SCORE_DIR` command and require every per-chapter QA result. For the eight reuse candidates compare regenerated stem bytes and exact part/sample order against the retained originals before promotion.

Before4K, read the actual encoded-output raw-byte caps from the frozen render source and compare them with new frame counts and negotiated encoder input format. `timeline17-summary.json` gives YUV420p 4K byte estimates per chapter. The local shared kit declares256GiB raw/32GiB artifact caps; this is a source observation, not a reason to raise any guard or assume long08/09 fit. If a new chapter exceeds its actual cap, root must use an explicitly reviewed solution; arbitrary CLI frame-range chunking is unavailable.

Actual producer picture/motion review of the seven new movies plus exact applicable prior per-chapter review/reuse proofs for the other eight supplies a complete coverage record. Contact-sheet extraction alone supplies none. Final4K chapter outputs still need native picture inspection, complete decode, terminal invariant-input receipts and exact soundtrack capture coverage (48kHz stereo PCM16, frames*800 samples).

Use a new mixed-provenance importer/master handoff; final-master16.py hardcodes production16/all15/CSD and117697. Compute every chapter offset, total frame count, duration and caption offset from the new timeline summaries. Require production17's bit-identical native soundtrack replay, explicit encoder/color/audio verification, stream-copy join validation and final audiovisual review. No all15 permit or public-release claim is produced by these prepared scripts.

## Local checks performed

Python AST/compile syntax passed for both scripts. Pure helper checks passed for fresh frame-count parsing, rejection of invalid/duplicate fps/frame rows, bounded ordered cue-picture selection, and exact disjoint seven/eight partition. Imports were checked without remote IO or launch. These are source/helper checks only; actual receipts and native media do not yet exist from this helper run.

Final helper revision after layout sampling repair: preview17.py `4cf1ae250464270022a47201a211233be101df02f5587561ef70f364df4a9b21`; launch-preview17.py `a063f386b4e408412f35cfa21081d4e848f229fa861e86dfcfe57d9b8714e663`. Read-only SSH confirmed the actual production16 preview directory and extraction path and inspected its chapter09 layout format. Pure checks cover old/new cue-relative anchors, historical samples, nonconsecutive recurrence semantics and unknown checkpoint refusal. A four-frame16x16 synthetic ffmpeg stdin check selected exactly frames1and3 with the emitted select-expression grammar; no media file/native job was created. The helper writes its identical filter expression to a new per-chapter filter file to avoid command-line length limits.
