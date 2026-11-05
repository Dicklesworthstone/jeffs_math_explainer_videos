# logarithm-stage09 render preparation 02

Answers BlueSalmon Mails 4108, 4245 and 4279, and LavenderPike Mails 4141 and 4229 (bead 87do.63). RubyRaven, 2026-11-05.

I launched, compiled and rendered nothing. I wrote nothing in W, WS, the lane, EL, `target/`, canonical source, Beads or the TODO. Version 1 (`fa654b83`) and the recovery bundle (`6bf686a1`) are unchanged.

## What changed from version 1

The full change is `docs/v1-to-v2.diff`: +109/−183 lines in 12 hunks. The helper is now 784 lines, down from 858.

1. **Installed audio helper.**
   - Version 1 pinned Bronze's helper `c6c854ce` in `audio-preparation01`.
   - Version 2 pins root's installed helper `P/logarithm-stage09-audio-preparation02/prepare_audio.py` (`54df274a`) and root's receipt `EL/stage09-audio-integration02/integration-receipt01.json`.
   - The receipt must name that path and hash, with source `08a26927` (LavenderPike's staged L1 + O3).
   - The receipt must not claim listening, fine sync or production assembly.
   - The installed folder has no SHA256SUMS, so these bytes and that receipt are the pins.
   - The binding `2d7f4731` and its sealed bundle `2bb8e280` are pinned as before.
2. **Tool fields.** Each receipt must name the installed helper as its `tool`: `<installed> prepare` and `<installed> timings`. Version 1 read neither field.
3. **Plan hash (O3).** The prepare receipt's `phrase_marks_plan_sha256` must equal the hash of `O/phrase-marks-plan.json`, and the plan becomes a bound file.
4. **Build.**
   - `build-verify` refuses attempt 01 and hands attempt 02 or later to the recovery helper (`00a18c99`, loaded by hash).
   - The recovery helper runs version 1's build-verify, then its compile and output-path proofs.
   - `bind --build 02` uses it, and so does every step, through `load_binding`.
5. **Workspace.** Version 2 accepts only a workspace that version 1 installed (its binding names `9ddc2003`). `install` and `build-script` are removed, because version 1 and the recovery helper own them.
6. **`check-audio`.** A new read-only command. It runs every audio check of `bind`, including the real coherent verify and the kit's delivery contract, and writes nothing.
7. **`--audio` must be absolute.** The binding, the kit's argv and the job scripts all record it exactly as given.
8. **Binding schema.** The schema is now `logarithm-stage09-render-narration-binding-v2`.
   - Every version 1 key is kept. `helper_sha256` is version 2's hash, and `audio.helper_sha256` is now `54df274a`.
   - Added keys:
     - `build.verified_by`;
     - in `audio`: `helper`, `integration_receipt` and its hash, `prepare_receipt_sha256`, `phrase_marks_plan_sha256`, `timings_receipt_sha256`, `phrase_timings_sha256`, `marks`, `selection.{sha256,by}` and `answers_sha256`.
   - Three files join the bound set: the plan, the integration receipt and the installed helper.

**Unchanged:** everything from `job_name` to the end of the job template, namely:
- job names, the lane and the work folder;
- each step's argv and `write_step`'s input rows;
- `verify_job`, the success lines and `SCOPE`.

Check S5 compares these sources with version 1's. A job script that version 2 writes differs from version 1's only in the helper hash its header and template name.

## Commands for root

Run from the repository root:

```
H=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-render-preparation02/prepare_stage09_render.py
WS="$W/logarithm-stage09-workspace01"
O="$PWD/narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/production-audio-stage09-01/prepared01"
python3 -I -B $H check-audio  --w "$W" --ws "$WS" --audio "$O"                             # read-only
python3 -I -B $H build-verify --w "$W" --ws "$WS" --attempt 02                             # read-only
python3 -I -B $H bind         --w "$W" --ws "$WS" --build 02 --audio "$O" --attempt 01      # writes WS/narration-binding01.json
python3 -I -B $H step         --w "$W" --ws "$WS" --binding 01 --series 01 --step timeline  # writes WS/jobs/..., prints the guard line
python3 -I -B $H verify-step  --w "$W" --ws "$WS" --binding 01 --series 01 --step timeline
```

The remaining steps follow version 1, one at a time, and each must pass `verify-step` before the next is written: preview (optional), score, render for each of the 13 chapters (`--chapter CH`, or `--retry N`), then master. The narration that LavenderPike's verifier takes as `--narration` is `audio.narration` in `WS/narration-binding01.json`, which is `O/narration`.

## What bind checks

`bind` runs `check`, then `build-verify`, then the audio checks below. Steps 3, 4 and 6 are new in version 2; the rest are version 1's, unchanged.

1. The audio bundle's SHA256SUMS (`2bb8e280`) and the binding (`2d7f4731`) are pinned.
2. The installed helper's bytes and root's integration receipt match.
3. The prepare receipt is right:
   - binding: Oliver, production;
   - whole film, written by the installed helper (`tool`);
   - the plan hash matches;
   - it lists the script's 80 cues in order;
   - `spans.json` (units `1132a5a4`) is the one it names.
4. The decode receipt is the production Oliver receipt, and its hash matches. The assembly workspace's tools are canonical, and `assembly.json` was made from `spans.json`.
5. The timings receipt is right:
   - binding: Oliver, production;
   - written by the installed helper (`tool`);
   - 80 cues;
   - it names this narration's `assembly.json` and `report.json`.
6. `phrase-timings.json` is right:
   - it is this preparation's (voice, binding, script and prepare-receipt hashes);
   - each cue's WAV and text hash matches;
   - the copy in `O/narration` is byte-identical.
7. Two real runs of the workspace's tool copies must pass: `coherent_narration.py verify --script --units --require-directions`, and the kit's delivery contract for the 13 chapters (80 of 80 cues verified, whole film, no promotion).
8. Every bound file is hashed into the binding: receipts, plan, narration tree, decoded takes and the bound plan.

## Read-only runs on root's real files

- **`check-audio` on the real O passes:**
  - 80 cues and 33 marks, plan mode canonical, 181 bound files;
  - plan `1302b36a`, prepare receipt `0b30ba1f`;
  - selection `a5c4dfa1` by BlueSalmon.
  - The coherent verify and the delivery contract ran on the real production narration.
- **`build-verify --attempt 02` on root's real ws-build02 passes:**
  - binary `e88e58e03b69b18f9e7c77dbeb4d810bc1b660637074b86746a6b98e497cda08`;
  - 4 units compiled from the workspace, 241 tests.
  - This agrees with root's result (Mail 4279).
- **Not run on real paths:** `bind`, because it writes into root's W, and so neither `step` nor any job.

## Checks: 26 PASS, 0 FAIL

`checks/run_checks.py SCRATCH --w W` was run twice into fresh scratch folders. The two transcripts are byte-equal, and one is `checks/outputs/run_checks.out`. Each run takes about 20 s.

| Group | What it shows |
|---|---|
| A1–A4, real | `check-audio` passes on root's real O. `build-verify` refuses ws-build01 and passes the real ws-build02. A relative `--audio` is refused |
| P0–P9, planted | Faults are planted in a scratch copy of `prepared01` and its timings. P0 shows the unchanged copy passes. Each of these is refused: a changed plan byte; a receipt without the plan hash; a prepare receipt from Bronze's original helper; a timings receipt from the staged helper; an installed timings file that differs; a Frederick receipt; one changed WAV sample with the timings re-forged to match (the real coherent verify refuses it); an integration receipt for other bytes; one that claims listening |
| B1–B6, emulated build | The scratch workspace is installed by version 1, and ws-build02 is written by the recovery helper and run under the emulated guard with a stub cargo. Version 2's `build-verify` passes. Version 2 binds the real audio once, refuses build 01, and its steps refuse version 1's binding |
| S1–S5, steps | The timeline job runs under the emulated guard: the real coherent verify on the production narration, then the stand-in's timelines. It ends with the success line, and `verify-step` passes. The score step is then written. JOB_TEMPLATE, SUCCESS, SCOPE and 12 step functions are identical to version 1's |
| U1 | Unchanged after the run: `prepared01` and its timings (bytes and modification times), the installed helper and the integration receipt. No binding or job appeared in root's workspace, and no narrated job appeared in the lane |

**B2 finding.** The sealed version 1 `bind`, run in scratch on the same real receipts, accepts them. But it records `c6c854ce` as the audio helper and binds no plan. So binding with version 1 would have passed while naming the wrong helper.

**Emulated:**
- The guard binds the inputs before and after the job. It does no admission, lane lock or sampling.
- cargo is a stub.
- The binary is a stand-in that lists the chapters and writes 60 fps timelines naming the bound WAVs. Its frame counts (17160) are not the film's.
- The kit's real timeline output was not produced. The real binary, the real guard and real cargo never ran.

## For LavenderPike (Mails 4141 and 4229)

All five schema points hold under version 2, because the step code is version 1's:

1. **Lane and names:** as you listed. There is also the optional `logarithm-stage09-preview<SS>`. `logarithm-stage09-final-verify<NN>` collides with no name of mine.
2. **Master job:** the receipt command, the success line and `master-summary.json` are as you listed. The summary's `master`, `srt` and `vtt` each hold a `path` and a `sha256`, and the summary also has a `scope`.
3. **Master script:** CONFIG has your keys, plus `step`, `script`, `target`, `topic`, `chapter`, `success`, `scope` and `after`. Row 0 of the inputs is the script. The rows bind the timeline job's, the score job's and the 13 render jobs' outputs.
4. **Work folder:**
   - For each chapter, the render rows bind `<CH>.mp4`, `.timeline.tsv`, `.cuelog.tsv`, `.cues.tsv` and `.layout.tsv`, each when present.
   - The kit writes `<CH>.layout.tsv` whenever `--lint` is on and it is not profiling, even with no findings (W `repo-view/crates/explainer_kit/src/cli.rs:765–766`). Render steps pass `--lint`.
   - I have not checked the kit's `soundtrack/` and `master/` folders.
5. **Timeline summary:** as you listed.

Your verifier reads no field of my binding and pins no hash of mine; `9ddc2003` appears only in its comments. Jobs that version 2 writes name version 2's hash in their header, and nothing else differs.

## Sweeps and a stray bytecode file

- **Sweep commits.** Two commits I did not make swept this folder while I worked: `f925fb6` (06:14:27Z) and `aeac056` (06:18:06Z). They hold an earlier helper (`ad30856e`) and earlier checks. BlueSalmon says root made neither (Mails 4223 and 4245). This seal is authoritative.
- **Bytecode.** A `__pycache__/prepare_stage09_render.cpython-315.pyc`, compiled from the 06:12:17Z helper, appeared at 06:14:22Z, 5 s before `f925fb6`. In the recovery bundle, the same thing happened 6 s before `fd81490`. So whatever makes those commits imports Python without `-B`. Every Python call of mine uses `-I -B`. I renamed the file in place (`.moved-20261105`); it is gitignored and not in SHA256SUMS.
- **Correction.** My recovery HANDOFF and Mail 4227 called `fd81490` "root's sweep commit". Root did not make it (Mail 4245). The recovery seal `6bf686a1` stays as it is, and the correction is recorded here.

## Not claimed

I claim no listening, viewing, picture review, render, fine sync, independent review or acceptance. I wrote the helper and its checks, so these checks are not independent.

## Files

| File | Purpose |
|---|---|
| `prepare_stage09_render.py` | version 2: `check`, `check-audio`, `build-verify`, `bind`, `step`, `verify-step` |
| `docs/v1-to-v2.diff` | every line changed from version 1 (`9ddc2003`) |
| `checks/run_checks.py` | the checks above |
| `checks/outputs/run_checks.out` | transcript, 26 PASS |
| `SHA256SUMS` | these files |
