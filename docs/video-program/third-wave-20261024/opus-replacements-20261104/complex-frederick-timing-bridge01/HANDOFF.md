# Complex Frederick timing bridge (bead .39, answers Mail 3846)

LavenderPike, 2026-11-05. This bundle turns root's ingested Frederick recognition plus root's selected complete takes into the timing the native kit reads. It also writes the elevenlabs-ui-receipt-v2 receipt that the canonical assembler reads. That assembler is Bronze's patch plus WhiteSwan's F2 fix, which root installed in Mail 3920.

This bundle also covers:

- **Mail 3910:** the full request path, the request hash and the voice in each record.
- **Mail 3920:** the assembler pin is rebound to the canonical bytes, and check r05 qualifies the rebind.
- **Mail 3923:** both completed ingests are read. No recognition was repeated.

It produces:

- **Sentence timing:** span `sentence_ends`, which become the sentence onsets in `report.json`.
- **Phrase timing:** `phrase-timings.json` marks at the exact byte offsets the stage05 chapter code waits on.

Mail 3846's subject says "independently qualify Frederick intake". Its body asks for this timing bridge, and I followed the body.

Nothing here decoded or assembled production audio, ran recognition, synthesized, built or rendered, and nothing here is a listening check. Decode, prepare, assemble and timings ran only on seeded-noise fixtures in scratch. The real-input report reads root's lane, jobs and takes, and writes nothing outside scratch.

## Paths

- `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`
- `EC` = `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104`
- `WORK` = `narration_lab/third-wave-20261024/root/five-film-release-20261027`

## Files

| File | What it is |
|---|---|
| `frederick_timing_bridge.py` | The helper: `report`, `decode`, `prepare`, `timings` |
| `checks/bridge_selftest.py` | 37 checks on a synthetic chapter-01 world, plus read-only checks of the real stage05 sources and the assembler rebind |
| `checks/real_input_report.py` | The report on root's real lane, run twice; prints every number and question |
| `checks/outputs/*.txt` | The two checks' output at seal |
| `report/` | `source.json`, `takes.json`, `questions.json`, `answers-template.json` and `coverage.json` from the real run |
| `inputs.sha256` | Every input read, by hash |

## Pinned inputs

A pin that moves exits 3.

- **Stage05 binding:** script `12442480…`, units `e776c860…`, map `70888624…`. The checks come from my sealed intake `frederick_intake.py` `edc154c6…`.
- **Stage05 chapter sources:** the 25 `src/chapters/*.rs` files, as one listing hash `a8e92ca2…`. The `.orig` files are not in it.
- **Log preparation:** Bronze's `logarithm-stage08-audio-preparation01/prepare_audio.py` `3083d5cd…`. I import its `tokens`, `kit_sentences` and `kit_normalize`, so word normalization and the kit's sentence rule are the same code. `align` and `place` are mirrored with the same rules plus the recognizer window. A test re-places every point with Bronze's own `points_for`.
- **Assembler:** canonical `tools/coherent_narration.py` `95615cfa…`, which root installed in Mail 3920 (Bronze's patch plus WhiteSwan's F2 fix).
  - Check r05 qualifies the rebind: these bytes are Bronze's sealed patched copy `18c4e75e…` with 9 inserted lines in 2 places, and nothing removed or changed. The insertions are the `LEGACY_OLIVER_VOICES` constant and a check inside the legacy branch, which a v2 receipt never reaches because the v2 branch returns first.
  - Every synthetic decode, prepare, assemble and timings test runs these canonical bytes.
  - The stage05 oliver12 map is not in the canonical tree, and the assembler resolves the units' map path against its own tree. So prepare copies the tool, `tools/narration_cache.py` `47331efd…` and the map into a workspace.
  - `tools/narration_delivery.py` is not used.
- **Kit contract mirrored:** `crates/explainer_kit/src/narration.rs` `8072720d…`.

## What each step does

### `report` (read-only on its inputs)

**Phrase anchors.** A small Rust lexer scans the non-test part of every chapter file and finds every `.wait_for(stage, …)`, `.until(stage, …)` and `.at(…)` call.

- 80 hooks take a string literal.
- 5 take their phrase indirectly. Each of these is bound to its exact source line in `INDIRECT`:
  - `counting.rs` `NEEDS_SAID`
  - `reading_color.rs` `NAMED`
  - `gallery.rs` `named`
  - `the_bump.rs`, the tuple array
  - `gallery.rs` `walk_and_count`, argument 3
- Any other variable hook exits 3.

Each phrase is bound to a cue in one of two ways:

1. by a `say(&kit, stage, "XX_Y")` in the same function (87 uses);
2. when the receiver is a helper's `line: &Spoken` parameter, by the only script cue containing the phrase (7 uses: 05_1, 05_2, 11_4 and four in 11_8).

A phrase missing from its cue, or ambiguous, is reported as a problem. There are none.

The result is 93 anchors in 51 cues. Each has:

- `byte` = the first occurrence, as Rust `str::find` gives it, which is what `Spoken::at` looks up;
- `byte_end` and `char`;
- the sentence it falls in;
- whether it starts that sentence;
- every call site.

**Dispositions.** All 23 map overrides are end-of-cue visual holds. Each becomes `native_visual_hold`, with evidence = the hold's chapter file under `EC/combined-repair-stage05/`, and its sha and marker line are checked. The pinned assembler's own `direction_contract` resolves the same end words, and its `review_directions` accepts all 23.

**Recognition.**

- For every ingest in root's lane, the helper re-reads the guard job's `command.log` from `WORK` (bound by `command_log_sha256` and `resource_receipt_sha256`).
- It re-derives the intake's alignment and requires it to equal `transcripts.json`, and re-hashes every WAV.
- It places each sentence onset (sentences 1…n; sentence 0 is the take's start) and each anchor with Bronze's support rule: an agreeing run of at least 3 words, first in its segment, in range and in order.
- An anchor that starts sentence k ≥ 1 reuses that sentence's onset, so the two can never disagree.

**Supplement** (`--supplement JOB MANIFEST`, opt-in, repeatable). This takes another model's guarded job, such as root's small.en run on 02_5:

- The job's receipt must have hashed exactly that manifest.
- Its command must be word-split whisper-cpp fw with the pinned fw binary.
- Each record must be ok and carry the manifest model's tag, and its input must be an unchanged ingested lane take.

Its placements appear beside the primary ones as cross-checks, with `delta_s`. They are offered as `use_supplementary:<job>` and never applied unless root answers with that decision.

**Questions.**

- `sentence_onset` and `phrase_anchor`: no supported placement. Decisions: `use_candidate` when the recognizer has one different word between agreeing runs, `use_supplementary:<job>`, `sample`; anchors also take `omit_mark`.
- `content`: any normalized recognizer disagreement, from either model. Decisions: `proceed` or `reject_take`. `proceed` is root's decision that the difference does not stop preparation. It states nothing heard.

All of these block the take until answered. Any supported point can also be overridden with `sample`, or for anchors with `omit_mark`.

### `decode` (root's run)

- Needs root's `complex-frederick-selection-v1` (format below).
- Converts each selected take with root's arguments: `ffmpeg -nostdin -n -v error -i SRC -ac 1 -ar 24000 -c:a pcm_s16le DST`.
- Requires mono PCM16 24 kHz with exactly ⌈n/2⌉ frames.
- Writes `decode-receipt.json` in CONTRACT §3 form:
  - `schema` `elevenlabs-ui-receipt-v2`, `source` `ElevenLabs UI, Frederick Surrey`, `provider` `ElevenLabs`, `voice` `Frederick Surrey - Smooth and Velvety`;
  - per record: `cue`, `text`, `voice`, `request` (the downloads receipt's folder joined to the request's bare name, repository-relative, as WhiteSwan's F3 and root's Mail 3910 ask), `request_sha256`, `source`, `source_sha256`, `wav`, `wav_sha256`, `samples`, `rate`;
  - extra keys the assembler ignores: the source PCM hash and frames, the downloads receipt, and the recognition job and record index.

### `prepare` (root's run)

- Needs the selection, root's answers and the decode receipt.
- Each unit is one whole decoded take:
  - `sentence_ends` = resolved onsets, then the last sample;
  - `gap_after_samples` all 0;
  - nothing cut or inserted.
- Dispositions are the map's holds for the selected cues. `boundary_review.listening` is `unreviewed`.
- Writes `workspace/` (byte copies of the pinned canonical assembler, the cache module and the map), `spans.json`, `assemble-command.json`, `phrase-marks-plan.json` and `prepare-receipt.json`.
- Dry run: the spans go through the workspace assembler's `read_units`, `check_boundary_review`, `assembly_voice`, `review_directions`, `unit_spans` (every v2 receipt check) and `check_dispositions`.
- If the assembler refuses, `REFUSED.txt` is written and the folder is kept as evidence.

### `timings` (root's run, after root's assemble)

- Calls the workspace assembler's `verify` with `--require-directions`.
- Requires `assembly.json`'s manifest to equal `spans.json`.
- Requires each assembled cue's PCM to equal its decoded take sample for sample.
- Mirrors the kit's checks:
  - sentence count and text;
  - total within 0.05 s;
  - marks byte-increasing, time-non-decreasing and inside the WAV.
- Requires the report's sentence onsets to equal the prepared starts exactly.
- Writes `phrase-timings.json` when the assembly holds every script cue. Otherwise it writes `phrase-timings-fragment-NN.json`, which the kit cannot load alone, because its loader needs a complete cue map.

Each mark carries:

- `byte`;
- `seconds` (= `sample_24k` / 24000);
- `sample_24k`;
- `phrase`;
- `basis` (recognizer onset, candidate, supplement or root's sample, each with who and why);
- `window_s`;
- `abuts_previous_word`;
- `cross_checks`;
- `film_without_mark_s`: what `Spoken::at` would compute from sentence starts without the mark.

## Uncertainty, stated in every output

- `window_s` is the recognizer's word segment [start, end]. The spoken onset was not measured.
- When `abuts_previous_word` is true, the segment can begin in the pause before the word, so the onset can lie anywhere up to the segment end.
- A recognizer can also place a word late. The window is not a bound. In 02_5 download01, small.en starts "what" 0.10 s and "which" 0.14 s before turbo's windows.
- On the real takes:
  - 146 of the supported points abut the previous word, with a window median of 0.24 s and a maximum of 0.88 s;
  - the 6 small.en cross-checks on 02_5 differ from turbo by a median of 0.06 s and a maximum of 0.14 s.

## Real input (root's lane at seal)

The full numbers are in `checks/outputs/real_input_report.txt` and `report/`.

**Source side.** sha256 `7188f725b110e4bfb2faa87e74d447d008f737143c8cb3bb2e645d728271705f`. Selections and answers must bind this value. It covers the pins, so it changed when the assembler pin moved.

**Recognition.**

- ingest-01 (plan-01, job `complex-frederick-stage05-recognition01-plan-01-a1`): 36 takes, 0 failed.
- ingest-02 (plan-02, job `complex-frederick-stage05-recognition01-plan-02-a1`): 6 takes (03_7, 04_1, 04_2), 0 failed.
- Total: 42 takes of 21 cues.
- Supplement `complex-frederick-targeted-small01`: small.en `c6138d6d…`, 2 records.

**Placements.**

- Sentence onsets: 126/126 supported.
- Anchor marks: 26/58 supported, and the other 32 sit at sentence onsets.
- Complete chapters: 01, 02 and 03.
- All 184 points equal Bronze's `points_for` placements.
- The 6 ingest-02 takes have no normalized differences. Root saw "2pi" in 03_7 and "1.7" in 04_2. The log preparation's `tokens()` reads those as "two pi" and "one point seven", the script's words.

**Questions.** 8, all blocking. Root's answers are needed before any of these takes can be prepared:

- `content:01_3:01_3-download01.wav`, `content:01_3:01_3-download02.wav`: "to the fortieth" was recognized as "forty".
- `content:01_6:01_6-download01.wav`, `content:01_6:01_6-download02.wav`: roots → routes.
- `content:02_4:02_4-download02.wav`: turn → term.
- `content:02_5:02_5-download01.wav`, `content:02_5:02_5-download02.wav`: turbo deletes "one plus i" twice and gives turn → term. small.en gives turn → term only. Root said not to accept 02_5 (Mail 3877); these questions hold both takes until root answers.
- `content:03_2:03_2-download01.wav`: half → over two, quarter → over four.

**Take folder at seal.** 2 WAVs of 04_3 are eligible for the next intake plan and not yet recognized. They are not in this report.

## Root's run sequence

All commands run from the repository root, and every `--out` must be a new folder.

```
B=P/complex-frederick-timing-bridge01/frederick_timing_bridge.py
L=EC/frederick-stage05-recognition01
S="--supplement WORK/complex-frederick-targeted-small01 EC/frederick-targeted-small01/inputs.json"   # optional
python3 -I -B $B report  --lane $L $S --out NEW_REPORT
#   root writes SELECTION and ANSWERS, binding NEW_REPORT/source.json "sha256"
python3 -I -B $B decode  --lane $L $S --chapter 01 --selection SELECTION --out NEW_DECODED
python3 -I -B $B prepare --lane $L $S --chapter 01 --selection SELECTION --answers ANSWERS \
    --decoded NEW_DECODED --reviewer BlueSalmon --out NEW_PREPARED
#   then the exact argv in NEW_PREPARED/assemble-command.json (the workspace assembler's assemble)
python3 -I -B $B timings --prepared NEW_PREPARED --narration NEW_PREPARED-narration --out NEW_TIMINGS
```

**Selection format:**

```json
{"schema": "complex-frederick-selection-v1", "source_sha256": "<source.json sha256>", "script_sha256": "12442480…",
 "by": "BlueSalmon", "basis": "<why these takes>", "chapter": "01",
 "takes": {"01_1": "01_1-download0N.wav", "…": "…"}}
```

- `takes` must list every cue of the chapter, or every script cue when `chapter` is null, in script order.
- Each take must be an ingested take of current wording.

**Answers format:**

```json
{"schema": "complex-frederick-answers-v1", "source_sha256": "…",
 "answers": {"<question or point id>": {"decision": "…", "by": "…", "basis": "…", "sample_24k": 123}}}
```

`sample_24k` is only for `sample`. `answers-template.json` lists every open question.

## Tests

### `checks/bridge_selftest.py`: 37/37, byte-equal over two runs

**Real stage05 sources, read-only:**

| Check | What it shows |
|---|---|
| r01 | The hook census matches an independent line grep: 80 literal + 5 indirect = 85 |
| r02 | Every anchor's byte, byte_end and char follow Rust `str::find` semantics |
| r03 | The dispositions agree with the pinned assembler's `direction_contract` and `review_directions` |
| r04 | coherent `sentences` equals `kit_sentences`, and draw.rs `sentence_offsets` counts equal them on all 105 cues, so the film uses measured starts everywhere |
| r05 | The canonical assembler is the sealed patched copy `18c4e75e…` plus insertions only (9 lines in 2 places, all for legacy Oliver), none of them on the v2 path |

**Synthetic world** (12 seeded-noise takes of chapter 01, through the sealed intake's plan and ingest):

| Check | What it shows |
|---|---|
| w01 | The sealed intake's plan and ingest accept the fixture world |
| w02 | `report` is byte-deterministic |
| w03 | Every supported point equals round(start × 24000) of its synthetic segment, with the exact window, and `abuts_previous_word` is true exactly for the abutting takes |
| w04 | The questions: a misheard anchor word offers its candidate; a merged segment leaves an onset inside_a_segment and offers the supplement; content differences block |
| w05 | Supplement placement and delta |
| w06 | Agreement with Bronze's `points_for` |
| w07 | `decode` with ffmpeg on the fixtures: ⌈n/2⌉ frames including an odd count; full request paths, hashes and the exact voice; the pinned assembler's `external_ui_voice` accepts the receipt |
| w08 | `prepare`: ends follow root's sample, the accepted candidate and the accepted supplement; 01_4's hold with the_bump.rs evidence; zero gaps; `unreviewed` |
| w09 | The prepared assemble argv runs |
| w10 | `timings`: report onsets equal the prepared ends; marks and fragment are correct |

**Negatives**, each with its cause in the message:

| Check | What is refused |
|---|---|
| n01–n04 | A selection without basis, an unknown take, the wrong cue set or another source side |
| n05–n12 | An unanswered blocking question, an unknown id, a non-integer sample, `reject_take`, an answer without basis, sentence ends out of order, a supplement job that was not offered, and `use_supplementary` when that supplement was not given to the run |
| n13 | A decoded file changed after decode |
| n14 | A receipt naming Oliver's voice |
| n15 | Narration changed after assembly (the assembler's own verify) |
| n16 | `transcripts.json` changed |
| n17 | The job's `command.log` changed |
| n18 | A supplement with another manifest |
| n19 | An existing `--out` |
| n20 | Script drift (exit 3) |
| n21 | A variable hook outside `INDIRECT` (exit 3); ambiguous and missing literals reported; comments skipped |
| n22 | A request file changed after decode: the pinned assembler's own `external request file changed` |

### `checks/real_input_report.py`: 5/5

- the report twice, byte-equal;
- both recognitions bound;
- all 184 points agree with `points_for`;
- coverage;
- the intake's read-only status.

**Replay:** `python3 -I -B P/complex-frederick-timing-bridge01/checks/bridge_selftest.py NEW_SCRATCH` (about 10 s), and the same for `real_input_report.py`.

## Limits and open points for root

1. **Rust is not compiled.** The kit's report and phrase-timing loader is mirrored, not run. A complete `phrase-timings.json` needs one assembly holding all 105 cues; I did not write a fragment merger. Q: do you want one?
2. **Assembler pin.** The bridge now pins canonical `tools/coherent_narration.py` `95615cfa…`, per Mail 3920, and r05 qualifies that rebind. Any later change to the tool exits 3 until the pin is reviewed again.

   The sealed intake bundle `complex-frederick-audio-preparation01` is unchanged, and this bundle does not edit it.
   - Its code pins no assembler hash.
   - Its `inputs.sha256` lists the pre-3920 `tools/coherent_narration.py`. That line is the one input of its 100 that no longer verifies; the other 99 are OK.
   - Its self-test compares mirrors of `read_script`, `sentences` and `direction_words`, which F2 does not touch. Replayed against the canonical tool in scratch, it passes 54/54 with output byte-identical to its sealed `checks/outputs/intake_selftest.txt`.
3. **Dispositions** are derived from the map's visual holds, and root may replace them. A `native_visual_hold` claims no silence in the take: the hold happens on the picture timeline, in the chapter code its evidence names.
4. **Marks** are quantized to the 24 kHz sample grid. A mark at a sentence start therefore equals that sentence's onset exactly.
5. **No take is selected here.** The 8 content questions need root's `proceed` or `reject_take` with basis, and 02_5 stays blocked until then.
6. **Source changes.** If the chapter sources change, the listing pin exits 3. The `INDIRECT` table and the source pin then need review.
7. **Recognizer limits.** The recognizer window is not a bound, and a transcript is not listening. Every value here is an estimate until someone with ears checks it.
