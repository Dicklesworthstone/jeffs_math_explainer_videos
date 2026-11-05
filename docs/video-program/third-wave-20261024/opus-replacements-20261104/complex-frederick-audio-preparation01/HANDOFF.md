# Complex Frederick intake and recognition preparation

LavenderPike, bead `.39`, answers Mail 3766. Frozen for root.

**Nothing in this bundle has been run against fw.** I ran no ASR, synthesis, build or render. Every fw log and guard receipt used in the tests is synthetic, and the WAVs I wrote are seeded-noise test fixtures, not narration. I made no claim of listening, take selection or acceptance. I edited no Beads, TODO, source, canonical file or receipt, and made no commits. Root owns the launch.

Paths below are relative to the repository root. Shorthand:
- `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`
- `H` = `P/complex-frederick-audio-preparation01` (this folder)
- `EC` = `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104`
- `EL` = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`
- `WORK` = `narration_lab/third-wave-20261024/root/five-film-release-20261027` (the guard's job folder)

## What it is

`frederick_intake.py` is one executable with four subcommands. It reads root's retained take folder `EC/frederick-stage05-01` and checks it against the pinned `combined-repair-stage05` crate.

| Subcommand | Writes | Purpose |
|---|---|---|
| `status --lane L` | nothing | Prints what `plan` would do now: eligible, excluded and refused WAVs, open plans, and the Oliver inventory. |
| `plan --lane L` | `L/plan-NN/` | Builds the frozen manifest, the inputs list and root's guarded runner for every WAV that is complete, distinct, Frederick-only, and neither processed nor in flight. If nothing is new, it writes nothing. |
| `ingest --lane L --plan plan-NN --job WORK/<job>` | `L/ingest-NN/` | Binds the guard job to the plan, re-hashes every WAV and writes provisional transcripts and alignment. This closes the plan. |
| `release --lane L --plan plan-NN --reason TEXT` | `L/release-NN.json` | Closes a plan that will not be run, so its WAVs become eligible again. |

`L` is root's lane folder; I suggest `EC/frederick-stage05-recognition01`. The first `plan` creates it.

The lane name becomes part of the guard job names, so it must be lowercase letters, digits, `-` or `_`. The tool refuses a lane inside the take folder, the stage05 crate, this bundle or `WORK`.

Exit codes:
- **0:** done, including "nothing new".
- **2:** refused, with nothing written.
- **3:** a pinned crate file or tool moved.

## Commands for root (repo root, `python3 -I -B`)

```
(cd H && shasum -a 256 -c SHA256SUMS)
python3 -I -B H/checks/intake_selftest.py NEW_SCRATCH_A        # SELFTEST: PASS (54/54)
python3 -I -B H/checks/real_intake_snapshot.py NEW_SCRATCH_B   # SNAPSHOT: PASS; counts follow the take folder
python3 -I -B H/frederick_intake.py status --lane EC/frederick-stage05-recognition01
python3 -I -B H/frederick_intake.py plan   --lane EC/frederick-stage05-recognition01
python3 -I -B EC/frederick-stage05-recognition01/plan-01/run-asr.py --attempt 1 --dry-run
python3 -I -B EC/frederick-stage05-recognition01/plan-01/run-asr.py --attempt 1        # root's guarded ASR
python3 -I -B H/frederick_intake.py ingest --lane EC/frederick-stage05-recognition01 --plan plan-01 \
    --job WORK/complex-frederick-stage05-recognition01-plan-01-a1
python3 -I -B H/frederick_intake.py plan   --lane EC/frederick-stage05-recognition01   # later: new WAVs only
```

**Guard admission.** If the guard does not admit the job (exit 75, no `command.log`), run the same plan again with `--attempt 2`. Every attempt is a new guard job folder.

**Failed runs.** If a run fails or stops on a resource limit, either retry with a new attempt, or ingest the failed job. Ingesting closes the plan and makes all of its WAVs eligible for the next plan. `release` closes a plan that will never run.

**Closed plans.** `run-asr.py` refuses a plan that an ingest or release has already closed. It also refuses to run from any directory other than the repository root.

## The plan root launches

The `plan-NN/` folder holds four files.

**`plan.json`** records, for each WAV:
- `cue`, `name`, absolute `path`;
- file `sha256` and `pcm_sha256`, `bytes`, `frames`, `seconds`;
- `sample_rate`, `channels`, `sample_bits`, `peak`;
- the RIFF chunk list (root's files are fmt, data, then a trailing C2PA chunk);
- the Downloads `source_name`;
- the `request`, `submission` and `downloads` receipts, each by name and by the sha256 of the bytes that were checked;
- the cue's `text_sha256`.

It also records the binding pins, the exact fw command, the job prefix, and the refused, excluded and pending cues.

**`inputs.txt`** has one absolute WAV path per line, in `plan.json` order. fw's record `index` is the line number; root's logarithm run has all 84 of its records in this order.

**`run-asr.py`** has the same shape as root's `EL/remaining-recognition01/run01.py`:

```
python3 -I -B P/engine-secondary-validation01/guard_owner56g_target20g.py --phase asr \
  --job complex-<lane>-plan-NN-a<attempt> --inputs <plan>/frozen-inputs.json -- \
  ~/.local/bin/fw transcribe --inputs-from <plan>/inputs.txt --backend whisper-cpp \
  --model ~/.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin \
  --language en --no-diarize --no-persist --split-on-word --max-segment-length 1 --threads 4 --timeout 180 --json
```

Both checks compare these fw arguments with root's run01.py, and they are equal except for the inputs list. The guard path is root's.

**`frozen-inputs.json`** is in the guard's `{"files": [{"path", "sha256", "role"}]}` form; the guard ignores `role`. Its rows are:
- the guard wrapper and root's guard source;
- `fw` and the model;
- this tool;
- the crate's script, units and map;
- every request, submission and downloads receipt the planned takes rest on;
- every WAV;
- `plan.json`, `inputs.txt` and `run-asr.py`.

Pinned tools and checked bytes are frozen at the hash that was checked, so a change made after planning fails the guard. Both checks load root's pinned guard through its wrapper and call only `check_inputs`, which hashes files. It accepts the manifests.

The lane files hold absolute paths for two reasons: the guard requires them, and fw echoes the inputs.txt path as each record's `input`. These are root's lane files, like run01.py. No file in this bundle holds an absolute path.

## Validation (`plan` and `status`)

**Whole-run pins (exit 3, nothing written):**

| File | sha256 |
|---|---|
| stage05 `script.tsv` | `12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3` |
| stage05 `narration_units.json` | `e776c860dcfa66c277ef0e188437cade034f0427d19efa4c975dde313d9b51c6` |
| `docs/narration-direction-map-oliver12.json` | `70888624edd6e930b383b277b50f8da5e39ac94f1a661c71b7f0e9575362f20c` |
| guard wrapper | `1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097` |
| root's guard source | `819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da` |
| `~/.local/bin/fw` | `7b570cf44d1f413ead01340c45e4070ec71009b7c2482d71a2aebf49e8fda47f` |
| `ggml-large-v3-turbo.bin` | `1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69` |

The checks also confirm that:
- the units bind the script and the map;
- every unit is one cue whose text equals its script row;
- the map binds the script and names only script cues.

The map's oliver12 file name is reviewed provenance only, as the integration01 limits say.

**Per downloads receipt.** A defect refuses that cue only; every other cue still plans. A refusal is not sticky: a WAV that is still copying is refused now and planned once it is complete.

- **Request:**
  - its `cue` matches;
  - its `text` is the stage05 row exactly, or the cue is refused as *stale cue text*;
  - `text_sha256` is the hash of that text;
  - `script_sha256` is `12442480…`, or the cue is refused as *script drift*;
  - `voice` is exactly `Frederick Surrey - Smooth and Velvety`;
  - `model` is Eleven v4, `stability` 0.5, `similarity` 0.75, `clone_strength` 0, `output` WAV 48 kHz Lossless.
- **Submission.** Exactly one submission must bind the request. Root's receipts use two layouts, and only these two are accepted:
  - layout A (01_x–02_x): `request` by name, `submitted: true`, `generation_requests`;
  - layout B (03_x): `request_sha256` equal to the hash of the request file, `submitted_once: true`.

  A receipt that carries both bindings must match both.
- **Takes:** the receipt must list exactly 2 takes per generation request. Each take:
  - is named `<cue>-downloadNN.wav` and lies in the take folder;
  - has a Downloads source name that starts `ElevenLabs_`, contains `Frederick Surrey - Smooth and Velvety_pvc_s50_sim75_v4`, ends `.wav` and does not contain "oliver";
  - exists, and its bytes hash to the receipt `sha256`.
- **WAV probe:**
  - it is RIFF/WAVE, and the declared RIFF size equals the file size;
  - every chunk lies inside the file, with its pad byte;
  - there is exactly one fmt chunk, PCM mono 48 kHz 16-bit, and exactly one data chunk, even and non-empty;
  - the `wave` module reads the same samples;
  - the samples are not all zero;
  - `pcm_sha256`, `frames` and `seconds` match the receipt.
- **Mixed voice by content.** Neither the file hash nor the PCM hash may match a retained Oliver recording. The inventory hashes every WAV in `EC/ui-takes-stage04-01`, `EC/counting-oliver02` and `EL/ui-takes01`: 182 WAVs, file and PCM. It also walks every `sha256`/`pcm_sha256` field in their receipts, which come in three layouts: `downloads`, `recordings` and `takes`. Only hashes are read; Oliver timings are never read or reused.
- **Duplicate input:** a cue is refused when:
  - the same name appears twice in one receipt or in two receipts;
  - the same file or PCM hash appears under two names, within a cue or across cues;
  - a take has the same audio as an already processed take or an in-flight take under another name.
- **Not planned:**
  - WAVs named by no receipt (reported as unreceipted);
  - cues with a submission but no downloads (reported as awaiting download).

## Incremental state

The lane is append-only. Every state is derived from its own immutable files:
- **processed:** the `processed` list of an `ingest-NN/ingest.json`, keyed by WAV sha256;
- **in flight:** the WAVs of any `plan-NN` with no ingest or release naming it;
- **eligible:** every other valid WAV.

Processed WAVs are never planned again. A WAV whose fw record failed, or whose run did not complete, becomes eligible again, and the next plan holds only those WAVs and new ones.

No file in the lane is ever rewritten. Plans, ingests and releases are all created exclusively, and their numbers only increase.

**Several take folders.** If root later uses a second take folder (for example a regeneration folder), pass `--takes` and keep the same lane. Processed and in-flight audio is matched by hash across folders. Duplicates between two folders that are both unplanned are not compared; plan one folder at a time.

## Ingest

Ingest refuses (exit 2, nothing written) when:
- the job folder is not `<job_prefix>-a<N>` for this plan;
- the plan is already closed;
- the guard receipt's `phase` or `command` differs from the plan's fw command;
- the job was not admitted, or was preflight only;
- the job has not finished;
- `inputs_sha256` is not the hash of this plan's `frozen-inputs.json`;
- a `command.log` record:
  - is not a `franken-whisper-batch-result-v1` record of this plan;
  - names an input outside the plan;
  - repeats an input;
  - has an index that does not match its inputs.txt line.

**A run that did not complete** (exit ≠ 0, a resource stop, a guard error, or a final hash that differs) closes the plan with every WAV failed.

**A record fails its WAV, which becomes eligible again, when:**
- its status is not ok;
- it has no record;
- it names a different model, or its model tag is not `sha256:1fc70f774d38`;
- it is not whisper_cpp, English, undiarized and split on words;
- the WAV changed since the plan.

Non-JSON log lines (fw stderr) are counted, not parsed.

`transcripts.json` has one row per processed take, holding:
- the transcript;
- the recognized words with fw's start, end and confidence;
- a provisional alignment of the script row to those words:
  - `direction_words` tokens matched with difflib, as root's transcript review compares them;
  - a `differences` list in root's {script, recognized} form, with the script word index;
  - the onset of every kit sentence, taken from its first word. When that word is unaligned the onset is null, and the first aligned onset is given instead.

Every row says `listening: false`, `fine_sync: false`, and that its times are provisional DTW marks. Ingest writes no kit `report.json` or `phrase-timings.json`. Turning provisional marks into kit timings is a separate gated step for root.

`read_script`, `sentences` and `direction_words` mirror `tools/coherent_narration.py`. The self-test confirms they agree on all 105 stage05 cues, so a later edit of that file is caught instead of silently changing this tool.

## Tests against Mail 3766

`checks/intake_selftest.py` passes 54 of 54. Its cases use synthetic fixtures with the real stage05 text, both submission layouts, and root's chunk order. Each planted defect leaves two good cues, so every refusal is shown to stay local to its own cue.

| Requirement | Cases |
|---|---|
| stale cue text | n01 (old wording, with a self-consistent hash), n02 (text hash wrong) |
| missing or partial WAV | n03 (missing), n04 (truncated, receipt of the full file), n05 (truncated, with a receipt written from the partial bytes: caught by the RIFF probe), n06 (all-zero samples), n07 (stereo), n16 (one take of two), n21 (frames wrong) |
| mixed voice | n08 (request voice Oliver), n09 (Oliver source name), n10 (a real retained Oliver WAV under complete Frederick labels), n10b (the same Oliver samples in a new container), n19 (an Oliver-layout receipt) |
| duplicate input | n11 (within a cue), n12 (across cues), n13 (same PCM, different container), n14 (one WAV in two receipts), i19 (same audio as a processed take, from a second take folder) |
| script drift | n15 (request binds the canonical script `2968b0fd`), n23 (a crate copy with one edited row: exit 3, nothing written), n24 (an exact crate copy is accepted, as a control) |
| incremental intake | i01–i03 (new cue only; in-flight WAVs not replanned), i13 (failed WAVs eligible again; processed WAVs excluded), i16 (release), i17 (resource stop), i18 (WAV changed after the plan), i20 (status) |
| ingest binding | i04–i10 (another plan's job, inputs hash, command, not admitted, foreign input, repeated record, index), i11 (status error and model mismatch fail only their WAVs), i12 (alignment and onsets), i14–i15 (a closed plan is refused by ingest and by the generated runner, checked without launching) |
| other | n17–n18 (no submission, or a submission bound to another request), n20 (take outside the folder), n22 (unreceipted WAVs), n25 (fw pin moved: exit 3), n26 (lane inside the take folder), n27 (nothing eligible: no lane created) |

The positive controls are p00:
- `status` writes nothing;
- 6 WAVs are planned;
- root's guard `check_inputs` accepts the manifest;
- the sample information is recorded;
- the runner command matches root's run01.py.

`checks/real_intake_snapshot.py` runs `status` and `plan` on the real take folder into scratch. It checks that:
- every planned take matches its receipt as mono 48 kHz 16-bit;
- the guard accepts the frozen manifest;
- the runner command matches root's;
- a second plan before any ingest writes nothing.

Root is still downloading, so its sealed output (`checks/outputs/real_intake_snapshot.txt`) lists only the cues present when the bundle was sealed.

## Notes for root

1. **Submission layouts.** The two layouts above are accepted exactly. A third layout would be refused until a reviewed tool revision.
2. **Stage04 Oliver receipts use a third layout (`recordings`).** That is why the inventory hashes the WAV files themselves, not only receipt fields.
3. **Units scope wording.** The stage05 units `scope` still says "Candidate full-cue source plan for ElevenLabs Oliver". This does not block intake. A later rewrite of that sentence changes the units hash and needs a new tool revision.
4. **A new script stage** (stage06 or later) exits 3 here by design. Make a new copy of the tool with new pins at a new path; do not edit this one in place.
5. **Model hashing.** Each `plan` and `status` hashes the 1.6 GB model, about 1 s.
6. **Coordination with BronzeJaguar.** They asked how their Frederick external receipt schema for `coherent_narration` (Mail 3806) fits this intake. I answered in Mail 3830: this intake does not decode or write assembly receipts. `plan.json` carries the request, submission and downloads bindings their v2 record needs.

## Limits

- This is preparation only: no fw, guard launch, synthesis, build, render, Beads, TODO or commit.
- ASR is not listening. Recognized word times are provisional DTW marks, not fine sync or phrase timings.
- I made no take selection and no acceptance claim, and Oliver timings are never reused.
- I wrote both the tool and its tests, so the tests are not independent of the tool.
- The snapshot reflects the take folder at sealing time only.
