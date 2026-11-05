# logarithm-stage09 render preparation 03

Answers BlueSalmon Mail 4410, plus LavenderPike Mail 4452 on the schema (bead 87do.63). RubyRaven, 2026-11-05.

**What I did not do:**
- I launched, compiled, rendered and scored nothing.
- I wrote nothing in W, WS, the lane, EL, `target/`, canonical source, Beads or the TODO.
- Root's running `logarithm-stage09-timeline01` was not touched. Its job script and inputs file were read; its lane folder was only listed for progress.
- Version 2 (`352a8107`), version 1 (`fa654b83`) and the recovery bundle (`6bf686a1`) are unchanged.
- The kit's source is unchanged.

## The omission

The kit's `master` null-tests each chapter against the soundtrack that the chapter render muxed. The render bound none of those soundtracks, and neither did version 2.

**How the kit makes and picks the capture:**
- **Capture:** `render <CH> --crf …` sets the capture folder `WORK/soundtrack/<CH>` (W `repo-view/crates/explainer_kit/src/cli.rs:666–681`). It first moves an earlier folder of that name to `WORK/soundtrack/previous/<CH>.vN`. Before each ffmpeg job that takes `-i …/soundtrack.wav`, it copies that WAV in as `soundtrack_<n>.wav`, where n is a per-process counter (`encode.rs:187–204`).
- **Choice:** the master null-tests the last of `soundtrack_0.wav`, `_1`, … before the first gap (`cli.rs:987–1000`). The replay must equal it byte for byte (`master.rs` `null_test`), and its length must equal `frame_to_sample(frames, fps)`.
- **What the master writes:** it writes nothing into `soundtrack/`.

**What version 2 bound:**
- Its render summary held the chapter, mp4, timeline and layout.
- Its master's frozen inputs held each render's mp4, `timeline.tsv`, `cuelog.tsv`, `cues.tsv` and `layout.tsv`.
- Neither held the capture. A capture changed, replaced or added between render and master went unbound.
- Check G0 shows this on the same render job: version 2's 8 rows bind no capture, and version 3's 9 rows are those 8 plus the capture.

## What changed from version 2

The full change is `docs/v2-to-v3.diff`: +106/−8 lines in 11 hunks. The helper is now 882 lines, up from 784.

1. **Render job.**
   - After the kit's render and the schedule check, the job refuses if the kit's capture rule finds no capture.
   - `render-summary.json` gains `soundtrack`: `{path, sha256, captures}`.
2. **Render verify, and the rows a later step freezes.**
   - The recorded capture must still be the kit's choice: no new `soundtrack_<N+1>`, and no gap filled in.
   - Its bytes must not have changed.
   - A render summary without `soundtrack` (version 2's shape) is refused.
   - `output_rows` adds the capture to the input rows, so the master's frozen inputs bind all 13.
3. **Master step and job.**
   - CONFIG gains `soundtracks`: `{CH: {path, sha256}}`, taken from each chapter's latest render summary.
   - Before running the kit, the job refuses if, for any chapter, the kit would choose another file or the bytes differ. After the kit and captions, it checks again.
   - `master-summary.json` gains `soundtracks` (the same dict).
   - `verify-step` requires the summary's `soundtracks` to equal CONFIG and each render summary, and re-checks each capture.
4. **Writer of a predecessor job.**
   - Every verified job's script header must name its helper.
   - That helper must be this one, or version 2 (`b867aaa2`, pinned by path and hash).
   - The guard's input rows must bind that helper's bytes.
   - So root's version 2 `timeline01` is consumed as it stands, without a rerun. A job written by any other helper is refused.
5. **Binding schema.** The schema is now `logarithm-stage09-render-narration-binding-v3`, and `helper_sha256` is version 3's hash.

**Unchanged:**
- `bind` and its audio and build checks, apart from the schema name;
- job names, the work folder, every argv, `write_step`, `predecessors` and `latest_render`;
- the success lines and `SCOPE`;
- the job template's `timeline`, `preview`, `score` and `main`.

Check S1 compares these sources with version 2's.

## Commands for root

Run from the repository root, after timeline01 has finished. Its receipt must show exit 0 and its log the timeline success line.

```
H=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-render-preparation03/prepare_stage09_render.py
WS="$W/logarithm-stage09-workspace01"
O="$PWD/narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/production-audio-stage09-01/prepared01"
python3 -I -B $H bind        --w "$W" --ws "$WS" --build 02 --audio "$O" --attempt 02       # writes WS/narration-binding02.json
python3 -I -B $H verify-step --w "$W" --ws "$WS" --binding 02 --series 01 --step timeline   # accepts version 2's timeline01
python3 -I -B $H step        --w "$W" --ws "$WS" --binding 02 --series 01 --step score      # prints the guard line
```

Then, in order, verifying each step before writing the next:
1. `verify-step … --step score`;
2. `step`/`verify-step … --step render --chapter CH` for the 13 chapters (`--retry N` after a failure, with the earlier `CH.mp4` moved aside);
3. `step`/`verify-step … --step master`.

**Notes:**
- `narration-binding01.json` stays as it is. Version 3 reads only its own `binding02`, which binds the same 181 files and the same binary.
- If a score job was written with version 2 meanwhile, version 3 accepts it.
- A render written with version 2 records no capture, so the master step refuses it. Rerender with version 3 (`--retry`).

## Schema for LavenderPike's final verifier

Bronze's six points (Mail 4380) all still hold. Version 3 only adds to them:

1. **Job names:** unchanged.
2. **Job files:** unchanged. There is still exactly one `CONFIG = json.loads(…)` line, and row 0 is the script.
3. **Receipt, command and success lines:** unchanged.
4. **Master CONFIG:** every listed key is unchanged, including `argv` and `captions_argv`. Added: `soundtracks`, `{CH: {"path", "sha256"}}` in `CHAPTERS` order. Each path is `L/logarithm-stage09-render-work01/soundtrack/<CH>/soundtrack_<n>.wav`, and each entry equals the `{path, sha256}` of `render_jobs[CH]`'s render summary.
5. **Summaries:**
   - `master-summary.json` keeps `master`, `srt` and `vtt` as `{path, sha256}`, and adds `soundtracks`, equal to CONFIG's.
   - `render-summary.json` keeps `chapter`, `mp4_sha256`, `timeline_sha256` and `layout_findings`, and adds `soundtrack: {path, sha256, captures}`. `captures` is n + 1.
   - `timeline-summary.json` is unchanged; timeline01 is version 2's.
6. **Master inputs:** everything they bound before, plus one `{path, sha256}` row per chapter for its capture. Separately, the row for `WS/narration-binding01.json` becomes `WS/narration-binding02.json`.

**Header:** timeline01's script header names `b867aaa2`. Scripts for the later steps name version 3's hash, which is in `SHA256SUMS` here.

**Optional checks for her version 03.** If she adds capture checks, the kit's rule is:
- the last contiguous `soundtrack_<n>.wav` from 0 is the one consumed;
- it is 48 kHz, 16-bit stereo, with exactly `frame_to_sample(frames, 60)` sample frames (`explainer_score/src/units.rs:41`).

A1 checks both on the Nov 2 render.

## Checks: 24 PASS, 0 FAIL

`checks/run_checks.py SCRATCH --w W` was run twice into fresh scratch folders (runs 02 and 03). The two transcripts are byte-equal, and one is `checks/outputs/run_checks.out`. Each run takes about 30 s. Run 01, before G0 and two label fixes, passed 23/23.

| Group | What it shows |
|---|---|
| A1 | **Real old output.** In the Nov 2 render work folder (`L/logarithm-machine-render-work01`), both the helper's and the job's capture rule pick `soundtrack_0.wav` for all 7 chapters, not the kit's set-aside `previous/04_principal_product.v1` copy. Each is 48 kHz S16 stereo and exactly `frame_to_sample` long |
| A2–A5, real, read-only | `check-audio` passes on root's real `prepared01` (181 files). `build-verify` passes the real ws-build02 (`e88e58e0`). Version 2's seal verifies. Root's real timeline01 script names `b867aaa2`, and its 239 guard rows bind those bytes, so `written_by` accepts it |
| B0–B1 | In scratch, as root ran it: ws-build02, version 2's binding01 and timeline01, with the real coherent verify. Version 3 then binds binding02 beside binding01 |
| T1–T3 | Version 3 verifies version 2's timeline01 without rerunning it. Another writer is refused, and so is a version 2 job whose rows lack version 2's bytes. The score step runs and verifies |
| R1–R3 | **Missing capture:** a render that leaves none fails in its own job. The retry records the kit's choice (`soundtrack_1`, 2 captures). The other 12 chapters render and verify |
| G0–G3 | **G0:** version 2's rows omit the capture. **G1, changed capture:** one byte changed after the render, and both `verify-step` and the master step refuse. **G2:** a stray file after a gap is ignored, as the kit ignores it; fill the gap and the kit's choice changes, so verify refuses. **G3:** a version 2-shaped summary is refused |
| M1–M4 | **M1:** the master CONFIG names the 13 captures and its rows bind them; chapter 00's is the retry's. **M2:** the job itself refuses a new capture and a changed capture before the kit runs. **M3:** the master runs, its summary records what the stand-in kit null-tested, and verify passes. **M4:** a capture changed afterwards is refused |
| S1–S2, U1 | **S1:** the unchanged code is version 2's. **S2:** the scratch timeline01 is byte-identical after version 3 used it. **U1:** the real inputs are untouched (bytes and modification times): `prepared01` and its timings, the Nov 2 render, version 2's bundle, the installed helper and its receipt. No new binding or job appeared in root's WS, and no narrated job in the lane |

**Emulated:**
- The guard binds the inputs before and after. It does no admission, lane lock or sampling.
- cargo is a stub.
- The binary is a stand-in. It writes the files the kit writes for `chapters`, timeline-only, `score`, `render <CH>` (two captures, the kit's set-aside), `master` and `captions`. It renders, scores and mixes nothing, and its null test is only the choice of file.
- The real kit's capture, null test and master gates ran only on Nov 2, not here. A1 reads their real output.

## Root's timeline01 at 06:49Z (not reviewed)

- Six of 13 chapters have a timeline in `timeline-output/`: `00_ratio_ruler` through `05_complex_turns`, the last written at 06:46Z.
- The job was still running, with no exit in its receipt.
- I read no timeline content, and no review, listening or viewing of it is claimed.

## Not claimed

I claim no listening, viewing, picture review, render, fine sync, independent review or acceptance. I wrote the helper and these checks, so the checks are not independent. No stage09 render, capture or master exists yet, so nothing here is evidence on final production.

## Files

| File | Purpose |
|---|---|
| `prepare_stage09_render.py` | version 3: `check`, `check-audio`, `build-verify`, `bind`, `step`, `verify-step` |
| `docs/v2-to-v3.diff` | every line changed from version 2 (`b867aaa2`) |
| `checks/run_checks.py` | the checks above |
| `checks/outputs/run_checks.out` | transcript, 24 PASS |
| `SHA256SUMS` | these files |
