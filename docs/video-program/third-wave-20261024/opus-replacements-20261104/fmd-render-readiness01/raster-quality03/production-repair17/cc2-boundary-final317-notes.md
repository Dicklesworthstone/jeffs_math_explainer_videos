# CC2 boundary final 317: the 09_boundary scored 4K60 worker for build313

Prepared for root on 2026-11-08. Nothing has been launched, deployed, permitted, imported or
rendered, and no Beads were touched. The 313 jobs (build313, rebind313, timeline313, preview313)
had not run when this was written. The worker requires each of them finished and stable, and it
fails closed until they are. Paths below are relative to D (the CSD project root) and
Q (`D/qualification01/production-repair17`).

| File | Lines | Role |
|---|---|---|
| `Q/csd-reviewed-boundary-final317.py` | 252 | guard-run worker: `check()`, `run()` |
| `Q/csd-launch-boundary-final317.py` | 105 | root-only launcher: hashes, closure, inputs, one guard call |
| `Q/cc2-boundary-final317-notes.md` | this | notes |

The sha256 of the two Python files is reported in the pane, not here, because the notes cannot
name their own hash. Preserved unchanged: `csd-reviewed-final303.py` (db3ce8e4…) and
`csd-launch-final303.py` (def224f0…), 293, 284, 300, 307 and 311 (862968830a…, ff39db34…,
22f3a0ce…). CC0's 313 helpers were read only.

## What 317 is

It is 303 adapted only for `09_boundary` under the 313 chain. Root's disposition 305 L1 is that
303 cannot render the repaired 09, because its binary is 268, its tables are timeline269 and its
preview is preview274. Against 303:

| Item | 303 | 317 |
|---|---|---|
| chapters | eleven | `('09_boundary',)`, `FRAMES=16504` |
| binary | `D/work/bin/review-repair268`, hash constant | `D/work/bin/review-repair313`. No hash constant: read from completed `D/jobs/csd-review-repair-build313/qualification.json` (see below) |
| narration | rebind269 | `D/jobs/csd-review-repair-rebind313/narration` |
| tables (`TIMELINES`) | timeline269 | `D/jobs/csd-review-repair-timeline313/timeline-output` |
| preview | preview274 (inputs hash constant) | preview313: worker pinned 67d780d8… (as `preview-after-timeline313.py:19`); inputs hash = its own receipt's, bound at launch |
| refused earlier finals | 244/278/287 | 244/278/287/**303** for 09 (decision R1) |
| job / inputs | `csd-reviewed-final-09-boundary303` / `Q/final-09_boundary-inputs303.json` | `csd-reviewed-final-09-boundary317` / `Q/final-09_boundary-inputs317.json` |

`verify_tables` is unchanged. Its narration mapping is the identity, because `FMD_NARRATION` and
`NARRATION` are both rebind313, just as the timeline313 tables already name rebind313.

## Gates kept exactly from 303

I diffed these regions of 303 against 317 line for line, and they are identical:

- `verify_tables` (timeline, cues, cuelog prefix, score parts rows, sample offsets);
- admission: `admitted`, no `exit`, policy `stop_free_gib 8` and `phase_growth_gib 4`, inputs
  hash, unique rows, every row re-hashed;
- affinity to the first `--threads` CPUs (the launcher passes 16 and refuses fewer than 16), and
  `render` must not exist;
- environment `PATH=/usr/bin:/bin`, plus TMPDIR and XDG_CACHE_HOME under `D/work` as in 303;
- the render command: `render 09_boundary --res 3840x2160 --fps 60 --threads N --narration
  --score STEMS --out --lint --crf 16 --preset veryfast`, cwd `D/work/production-repo07`,
  timeout 43200;
- ffprobe: 3840×2160, `60/1`, `nb_frames == frames`, h264/yuv420p, aac/48000/2;
- x264 `rc=crf` and `crf=16.0`;
- last soundtrack capture 48000/2/16-bit/`frames*800`;
- `check()` repeated after the render, then all frozen rows re-hashed, then
  `render-summary.json` written with `'x'`;
- argparse;
- in the launcher: worker and permit hashed before the worker loads, resolved paths, the job
  must not exist, inputs written with `'x'`, and the same single guard command
  (`guard_csd02.py --phase native --job … --inputs … -- python3 -I -B worker --chapter --job
  --inputs --permit --threads 16`, no wrapper, as 303).

The permit gates of 303 are kept: binary, chapter, `host == 'csd'`, both `actually_reviewed_*`
true, `unresolved_defects == []`, reviewer and observations non-empty, `timeline_sha256`,
evidence, `motion_intervals` inside `[0, frames)`, `source_binding_evidence`, every item at
hash, `frames` equal to the timeline header, and stems qa passed. The score21 summary and all
its files, the encoder-stop13, timeline21 and score21 receipts, and build268, rebind269 and
timeline269 with all fifteen tables equal to timeline21 under assembly19→rebind269 are checked
as in 303.

## What 317 adds (all in `check()`, all fail closed)

1. **`completed(job)`**, for each of the four 313 jobs. The receipt must show success (`finished`,
   exit 0, inputs equal to final inputs, no error or resource stop). The manifest cc0's launcher
   wrote (`Q/review-repair-{build,rebind,timeline,preview}313-inputs.json`) must hash to that
   receipt's `inputs_sha256`, and **every row is re-hashed**. This is what "successful stable
   receipts" means here. Rebind313's rewrite of `narration_units.json` does not break it:
   `review-repair-after-build313.py:40` excludes that file from rebind313's rows, and timeline313
   binds the tree after the rewrite.
2. **`build313()`**: the exact 310 source.
   - Qualification: binary hash equal to `sha(BIN)`, different from 268, and base 268; tests 233,
     plate-fade probe, Clippy and fmt passed; facts and chapters equal to build268's.
   - `source-copy.json`: `changed == [boundary.rs]`, base 81d28272…, and the same 70 keys as
     build268's, differing only in boundary.rs, which goes from dc80c541… to bda7d931….
     bda7d931… is `sha(Q/boundary-guide-repair310.rs)`.
   - Workspace: boundary.rs, `script.tsv` and the two narration tools in
     `D/work/review-repair313/workspace` still equal their source-copy hashes. Binary 313 reads
     the script and delivery contract from that crate, as 293:56-61 records for 268.
3. **`narration313()`**: the retained Frederick (ElevenLabs) takes, with nothing synthesized.
   - Rebind313's summary reports 105/105 byte-identical WAVs.
   - The file names in rebind313, rebind269 and assembly19 are identical.
   - All 105 WAVs are byte-identical across the three folders, recomputed here.
   - Only `external-units.json`, `report.json` and `assembly.json` may differ from rebind269.
   - The rewritten `narration_units.json` and the review313 map equal the summary's
     `canonical_sha256` and `map_sha256`.
4. **`schedule313()`**: the full 09 schedule.
   - The three timeline313 09 tables are re-compared here, byte for byte, with timeline269's under
     rebind269→rebind313 and with production timeline21's under assembly19→rebind313.
   - `comparison.json` must state frames `{09_boundary: 16504}`, both named mappings,
     `new_sha256` and both equality flags true.
5. **`independent(review)`**: the permit's `independent_reviewer` must be non-empty and different
   from `reviewer`, and `independent_review` must be a non-empty list of files directly in Q,
   resolved, at hash.
6. **`preview_binding()`**: preview313, the run root reviews.
   - Receipt and manifest are stable (1). The worker is in the receipt command and in the
     manifest at 67d780d8…. BIN, build313's `source-copy.json` and `qualification.json` are in
     the manifest at their current hashes.
   - The three `preview/` tables are byte-equal to timeline313's.
   - `summary.json`: binary equals BIN; 09 has 16504 frames, `zoom_probe_fails == []` and an exit
     frame. `sampling.json` and `zoom-probe.json` name the preview movie's hash.
   - The permit's `evidence` must bind preview313's `09_boundary/sampling.json` and
     `09_boundary/zoom-probe.json` at hash, and some `motion_intervals` entry must contain the
     measured `zoom_probe_exit_frame`. 312 L2 predicts that exit at 2087–2099.

## Binary hash: pinned at launch, never invented

Neither file contains a 313 binary hash. At launch:
- the launcher loads the worker, whose `build313()` takes `binary_sha256` from the completed
  build313 `qualification.json` and checks it against `sha(BIN)` and the permit;
- BIN and that `qualification.json` become rows of the new inputs at their actual hashes, together
  with every 313 receipt and manifest;
- the launcher prints the job, inputs hash, binary hash and the four 313 receipts'
  `inputs_sha256` before the guard call.

Inside the guard, `run()` requires each of these to be bound at hash. `render-summary.json`
records `binary_sha256`, the build313 qualification hash, `boundary_sha256`, `preview_inputs_sha256`
and the independent review items.

## Input closure the launcher writes

| Group | Rows |
|---|---|
| launch | launcher, worker, permit, `D/guard_csd02.py`, BIN |
| each 313 job | receipt, its manifest, every row, every file in its job folder |
| timeline269 | receipt, manifest, rows, job folder (as 303) |
| permit | evidence, source-binding evidence, independent review files, the preview binding list |
| narration | rebind313, rebind269 and assembly19 folders (all files) |
| tables | timeline313 / timeline269 / timeline21 09 tables, `comparison.json` |
| workspace | script, the two tools, boundary.rs, `narration_units.json`, review313 map; `Q/boundary-guide-repair310.rs` |
| score and earlier receipts | score21 summary and files, then score21, encoder-stop13, timeline21, build268 and rebind269 receipts, plus build268 qualification |

## Permit fields root writes (after its own actual review)

`chapter: 09_boundary`, `host: csd`, `binary_sha256` (build313's), `preview_job:
csd-review-repair-preview313`, `frames: 16504`, `timeline_sha256` (the **timeline313** 09
timeline table), `actually_reviewed_pictures: true`, `actually_reviewed_motion: true`,
`unresolved_defects: []`, `reviewer`, `observations`, `evidence` (including preview313's 09
`sampling.json` and `zoom-probe.json`), `motion_intervals` (covering the exit frame),
`source_binding_evidence`, `independent_reviewer` and `independent_review`. Each item is
`{path, sha256}` with an absolute path. The worker does not write or approve any of these.

## Root's order, once ready

1. Build313 (`launch-review-repair313.py`), then `review-repair-after-build313.py` (rebind313 and
   timeline313), then `preview-after-timeline313.py` (preview313). These are CC0's helpers,
   unchanged.
2. Root's own actual picture and motion review of preview313, an independent review, and the
   permit.
3. `python3 -I -B Q/csd-launch-boundary-final317.py --chapter 09_boundary --permit <permit>
   --permit-sha256 <hash> --worker-sha256 <hash of the 317 worker>`. This runs once, in the one
   exclusive native lane on csd.

## Decisions and findings for root

- **R1: 303 09 job refused.** An existing `csd-reviewed-final-09-boundary303` stops 317. That job
  would be an old-boundary 09, so root decides whether it can stand next to a repaired final.
  Remove `'303'` from `EARLIER` in a new version if root wants both.
- **R2: master 293 cannot take a 317 09 final as written.** `cc0-combined-master293.py:312-314`
  requires `module.BINARY_SHA == BINARY_268` and binds the timeline269 table. 317 has no
  `BINARY_SHA`, by design (its binary is build313's, read at launch). Its permit binds
  timeline313's table and its 09 tables name rebind313. Assembling a master with the repaired 09
  needs a new master version that accepts this worker. Its 09 audio would come from
  WAVs byte-identical to rebind269's. Captions and full AV would need the same. I made no change
  to 293 and wrote no master helper.
- **R3: only the zoom-exit frame is mechanically tied to motion.** The other 312 L2 points
  (arrival from 2326, 69.67/90.28/104 s, persistence over 39–133 s) and L3 are root's to watch.
  317 only requires the permit to say it did.
- **R4: cost.** `check()` re-hashes every row of the four 313 manifests at least three times: in the
  launcher and twice in the worker. The preview313 manifest includes preview274's 12,669 rows.
  This is hashing only, before and after the render.

## Evidence and limits

Both files pass `ast.parse` under `python3 -I -B`. I exercised them against a synthetic fixture
tree outside the repo, which is not kept in the repo and is not a real run. The fixture used
the real worker and launcher text, with only the host root and the two build268 base hashes
changed; subprocess and CPU affinity were patched, so no binary, guard or render ran. 24 of 24
expectations held:
- one positive `check()`;
- 18 refusals: 9 permit faults, 1 WAV, 1 cues table, 1 binary, 1 candidate row, 1 probe failure,
  1 resource stop, 1 rebind row, 1 workspace boundary, and 1 existing 303 job;
- the launcher writing inputs once and issuing exactly the 303-form guard argv;
- a second launch refused on the existing inputs file;
- `run()` refusing inputs without the independent review;
- `run()` passing every pre-render gate and issuing exactly the 303 render command.

Nothing was run on csd, CSS or HZ2. No listening, picture, motion, 4K, fine-sync, caption or AV
acceptance is claimed. Those remain root's, together with launch, master, captions and publication.
Narration is the retained Frederick ElevenLabs takes, unchanged.
