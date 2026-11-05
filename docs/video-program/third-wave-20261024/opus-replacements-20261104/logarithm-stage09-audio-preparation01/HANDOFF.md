# Logarithm stage09 audio preparation (bead mx-video-program-program-87do.63)

BronzeJaguar, 2026-11-05, for root (BlueSalmon). Answers Mail 3845, and re-pins the assembler after Mail 3920.

This bundle adapts my sealed stage08 helper (`logarithm-stage08-audio-preparation01`, SHA256SUMS `7e7a1502…`, left untouched) at a new path. It holds:
- the adapted helper;
- a production binding for the stage09 candidate, the 162 retained Oliver recordings, the four clarity replacements and their recognition;
- the read-only report the helper produced;
- checks, a mutation check, and both deltas against stage08.

**What I did not do:**
- select or accept a take, or listen to anything;
- decode, assemble or time any production audio;
- synthesize, render, build or compile anything;
- edit the candidate, canonical, receipts, reviews or any sealed bundle;
- touch Beads, the TODO or any commit.

**Recognition is not listening.** Every time and transcript here is a whisper-cpp estimate. Where this file says a transcript "matches", it means the recognizer's words, not what was spoken.

## Inputs read

| Input | sha256 | Note |
|---|---|---|
| `S/root-integration-receipt01.json` | `e16430ecc6728cde8325d761ee6d55b533dd1232025c15d6d4603f13e9e69282` | see below |
| `S/candidate/…/script.tsv` | `559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a` | 80 cues; only 04_4 and 08_5 differ from stage08 (check C0) |
| `S/candidate/…/narration_units.json` | `1132a5a452b339dd4d6fb340668c525b2e10dfe892d27035ce11957e80cafce9` | |
| `S/candidate/…/docs/narration-direction-map-oliver16.json` | `14e87893ea389ef2cdf827f0a7e59a8e2b65ac537b726d796ffceff5117e4787` | |
| `EL/full80-retained-inventory01.json` | `d4ee2762f067ca7e1a78ec3d950dae5fce9ba7b8d39be6425a61127ae8eaf44c` | 162 recordings; `receipt_sha256` hashes each downloads receipt |
| `S/clarity-retained-inventory01.json` | `0f409e93c34ce41d3c70437f05231cde0f27a65075c50738f1e2847d02b7517c` | 4 recordings; `selection: null`; `receipt_sha256` hashes each **request** |
| `S/oliver-clarity-takes01/04_4-downloads01.json` | `20707786124aa21d4b32b613105120063bbafcbb525b0b7f83224587fac83751` | pinned (the inventory does not hash it) |
| `S/oliver-clarity-takes01/08_5-downloads01.json` | `35c951df31735b310073db33f46054908e2e5e41b7f61213bcee0b01cdea715c` | pinned |
| `…/04_4-request01.json`, `…/08_5-request01.json` | `83f0aa2d…`, `bce14b16…` | hashed by the clarity inventory |
| `…/04_4-submission01.json`, `…/08_5-submission01.json` | `d6493a9d…`, `3fbe2057…` | read by check C3b; not bound by the helper |
| `B/logarithm-clarity-recognition01/command.log` | `5145e630ed8be7f0aec89b2f5afe1262f15c8f598ed3fa3f735c129c66a24d75` | |
| `B/logarithm-clarity-recognition01/resource-receipt.json` | `84b9e37242d2ae9b480fc2e9b85ca2a99953f6805767ee01051377dc81b2e895` | exit 0, admitted, not preflight, inputs stable |
| `S/clarity-recognition01/frozen-inputs.json` | `52f0441d737547a7763b176e2883e74567fc6bf4fb938282b4e40f2f3176f115` | the four WAVs at their inventoried hashes |
| `S/bound-plan/phrase-anchors.json` | `557d74a065260303689df29fedefafa54b084a691e912844b3f8e9171b1b7c84` | 33 anchors; 08_5 at bytes 267 and 306; none in 04_4 |
| `S/bound-plan/phrase-timings-unmeasured-template.json` | `94f6e564e2178188c3f6867edef7fb26f7ce5dec110348e67b0fefee48fccd2a` | |
| `S/bound-plan/direction-dispositions-proposed.json` | `8f95fb565b9a11c53889d1bb3701cc62a1221609deca74c7ebf7e7390f21cd94` | evidence `hold-audit.json` `f6f315d6…` |
| `tools/coherent_narration.py` (canonical) | `95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5` | as root installed it in Mail 3920 |
| `tools/narration_cache.py`, `crates/explainer_kit/src/narration.rs` | `47331efd…`, `8072720d…` | unchanged since stage08 |

Abbreviations: `EL` = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`, `S` = `EL/combined-repair-stage09`, `B` = `narration_lab/third-wave-20261024/root/five-film-release-20261027`.

**Root's integration receipt** says:
- White's sealed manifest `5fcb5ea6…` was verified: 118 replay checks, exit 0; 58 files installed.
- Lavender's acceptance: 44 pass, 1 fail ("units scope prose changed"), accepted by root.
- Voice "ElevenLabs Oliver", `native_test: pending`, `listening: false`, `fine_sync: false`.

Its `new_recordings: "not yet generated"` predates the clarity requests: the receipt is stamped 03:59:01Z and the requests 03:59:37Z. The clarity inventory and the recognition job are the later record. I did not edit the receipt.

## The binding (`bindings/logarithm-stage09-oliver.json`, `2d7f4731d373b5089686f5d345834faeb1a7238dc15007abfc7b4fab420d6803`)

- `use: production`; one voice, Oliver: `receipt_source` "ElevenLabs UI, Oliver" and `download_source_contains` "Oliver - Clean, British and Steady".
- `tree` is `S/candidate`.
- Two inventories:
  - full80, whose receipt is the downloads receipt;
  - clarity, with `receipt_hash_of` = the request.
  - Both name `request` = `{takes_dir}/{cue}-request01.json`.
- Six recognition runs: the stage08 five, unchanged, plus `clarity` (large-v3-turbo, anchors).
- Anchors, template and dispositions come from `S/bound-plan`.
- Assembler, cache and kit are canonical. The assembler is pinned at root's Mail 3920 install (`95615cfa…`); see "Assembler pin".
- 36 pins.
- The two stage08 content questions are kept as history. They are bound to the stage08 texts, so the report lists them under `content_questions_for_other_text` and raises no question from them (check C10).

## Helper changes (`docs/helper-delta.diff`, stage08 `3083d5cd…` → stage09)

1. **Which receipt an inventory hashes.**
   - An inventory may say `receipt_hash_of`. Without it, the downloads receipt is the hashed receipt, as in stage08.
   - A downloads receipt the inventory does not hash must be pinned by the binding (checks G15, G17).
   - This was needed because the clarity inventory hashes the requests: the stage08 helper refused it (G18 is that refusal).
2. **Requests.** When an inventory names `request`, each recording's request must name its cue and exactly its inventoried text. Both the `text` field's hash and the `text_sha256` field are compared.
   - This is what stops an inventory from relabelling an old take as the new text, or a new take as the old (checks G9–G12).
   - All 166 pass (C3c).
3. **File names shared with superseded takes.** The clarity takes reuse the old file names: `04_4-download01.wav` exists in `ui-takes01` (old wording) and in `oliver-clarity-takes01` (new).
   - A selection may name a recording by file name or by repository-relative path.
   - A bare name shared with a superseded recording is refused with the path to use (G8, G8b, and G1 on a stage08 selection).
   - A superseded recording named by its path is refused as superseded wording (G6, G7).
4. **Ids carry the path for those four recordings.** Question and point ids normally end in the file name. For the four clarity recordings they end in the path, for example `onset:08_5:s1:narration_lab/…/oliver-clarity-takes01/08_5-download01.wav`.
   - A stage08 answer such as `onset:08_5:s4:08_5-download01.wav`, measured on the old recording, is therefore an unknown id and refuses the answers file (check D13, "a stage08 answer id carried over").
   - All other ids are unchanged (G22).
5. **Report and receipt fields.**
   - `anchors.json` recordings carry `path` and `label`.
   - `coverage.json` lists `superseded_text_recordings` as paths.
   - `prepare-receipt.json` cues carry `recording_path`.
6. **One hint reworded.** When the pinned assembler does not name the binding's receipt source, `REFUSED.txt` used to say the receipt was "read as a native FrankenTTS receipt". The installed assembler refuses such a receipt instead, so the hint now says it "cannot read the decode receipt as an external take".

The helper still names no voice (F1). `docs/checks-delta.diff` is the same comparison for `checks/run_checks.py`.

## Assembler pin (Mail 3920)

Root installed the external-voice adapter, with White's F2 fix, at 04:34Z. Canonical `tools/coherent_narration.py` changed from `e0aa81f3…` to `95615cfa…`.
- My first stage09 binding pinned `e0aa81f3…`. A check run that started before the install and finished after it went stale (exit 3), as it should.
- I moved that binding and its report to my scratch; they were never delivered. The binding here was rewritten with the same content, apart from the new assembler pin and one sentence in `why`. The report was regenerated from it and equals the first report apart from the binding hash.
- **Qualification against the installed assembler:**
  - D5, D9 and D12 assemble with byte copies of it, and D6 verifies the result.
  - D6b imports the pinned copy. It reads the helper's decode receipt (`source` "ElevenLabs UI, Oliver", `voice` "Oliver", no schema) as exactly "Oliver - Clean, British and Steady". It refuses the same receipt when it states "Frederick Surrey" (the F2 guard).
  - The relabel plant now ends in the installed tool's own refusal ("unrecognised external UI receipt; a non-legacy label needs schema elevenlabs-ui-receipt-v2").
- `tools/narration_delivery.py` (`961f17cc…`) is not read by this helper and is not pinned.
- **White's F1 (no producer writes v2)** still holds for this helper. That matters for a Frederick binding, not for this one. This binding uses the legacy Oliver label, which binds no request.
  - The helper's request check (change 2) ties each recording to its cue and exact text before decode.
  - v2 could not cover the whole film in any case: 49 of the 80 `ui-takes01` requests name the voice "Oliver", not the exact UI name (White's note 2).
  - Adding v2 output for Frederick bindings is root's call; I have not done it.

## The four new takes against the exact stage09 texts

Changed words: 04_4 "which is why this base is called natural" → "which is why **base e** is called natural"; 08_5 "**Halving** the lost lap leaves a half turn" → "**Taking half of** the lost lap leaves a half turn".

| Take | Frames at 48 kHz | Recognizer text at the change | Word differences from the stage09 text |
|---|---|---|---|
| `04_4-download01.wav` | 1451520 (30.24 s) | "…which is why base E is called natural." | none apart from numbers written as digits ("1", "1.01", "0.01") |
| `04_4-download02.wav` | 1440000 (30.00 s) | same | same |
| `08_5-download01.wav` | 1555200 (32.40 s) | "Taking half of the lost lap leaves a half turn" | none apart from digits ("minus 1", "0", "2 pi"); it also writes "Here, a square root" with a comma |
| `08_5-download02.wav` | 1551360 (32.32 s) | same | none; numbers written as words |

- All four records have status `ok`. The clarity run recognized exactly these four files, and no other run recognized them (C2b).
- Against the old texts, the same transcripts differ exactly at the change: 04_4 drops "this" and adds "e"; 08_5 replaces "halving" with "taking half of".
- The helper's normalized comparison lists no difference for any of the four (`asr-differences.json`; C7b checks this independently with a second tokenizer).
- The superseded takes' transcripts are unchanged and kept apart: "space is called natural" and "having the lost lap" four times each (C7a). Those were the stage08 disputes, and they belong to the old recordings.

**Residual disputes, without any listening claim:**
1. **No text dispute remains in the recognizer output** for the four new takes. The old base/space and halving/having questions concern the old recordings and old texts only.
2. **One recognizer.** Only large-v3-turbo ran on the new takes. In stage08, a small.en run corroborated the old 04_4/08_5 transcripts; none exists for these four. This is a gap in corroboration, not a disagreement.
3. **"base E"** is the recognizer's spelling of the letter. It shows the recognizer matched the word, not how "e" was pronounced or stressed.
4. **08_5-download01, sentence 1 ("Here a square root…")** starts at 1.64 s in a 1.26 s word segment that abuts "roots." So the onset may lie up to about 1.26 s before the word. download02's "Here" segment is 0.24 s. If root picks download01, a measured `sample` for `onset:08_5:s1:<path>` is worth considering. Without one, the recognizer onset is used as for every other cue.

Onsets the report places for the new takes (24 kHz samples; sentence 0 is the file start):

| Take | s1 | s2 | s3 | s4 | s5 | marks 267 / 306 |
|---|---|---|---|---|---|---|
| 04_4 download01 | 67200 | 156960 | 320160 | 404160 | 563040 | none in 04_4 |
| 04_4 download02 | 67200 | 149760 | 310080 | 392640 | 552000 | none in 04_4 |
| 08_5 download01 | 39360 | 160800 | 353280 | 488640 | 660480 | 538080 / 609120 |
| 08_5 download02 | 51840 | 156000 | 355680 | 487680 | 649440 | 535680 / 607200 |

All of these are supported. 08_5 sentence 4 ("Taking…") agrees with an independent trigram search in both takes (C5); in stage08 that onset had no supported value.

## Report (real inputs, `report/`)

- **Coverage.** 162 selectable recordings of 80 cues, all 48 kHz mono PCM16. 166 are inventoried; the four original 04_4/08_5 takes in `ui-takes01` are listed under `superseded_text_recordings` and cannot be selected.
  - 04_4 and 08_5 are selectable only as the four clarity recordings, timed from the clarity run.
  - Each of the 78 unchanged cues keeps exactly its full80 recordings (G20). Their bytes, recognition, onsets and marks are identical to the stage08 report: 158 recordings compared (G21).
  - The old takes' records in middle (2), remaining (2) and targeted-small (4) count as superseded and are not used (C2c).
- **Onsets 570/570 and marks 66/66 supported.**
- **Abutting caveat.** 431 supported onsets abut the previous word (median segment 0.24 s, max 1.64 s at 04_7 s1). 46 segments are at least 0.6 s and 5 at least 1.0 s, including 08_5-download01 s1 above. `coverage.json` lists all 46.
- **Questions: 4, of which 2 block.**

| Id | Kind | Blocks | Decisions |
|---|---|---|---|
| `provenance:00_1:00_1-download03.wav`, `…download04.wav` | twin recognition (samples equal download02's) | yes, only if selected | `use_pcm_identity` |
| `voice:04_3:04_3-download01.wav`, `…download02.wav` | voice (download names lack the voice; the receipt records "Two matching Oliver provider history entries…") | no (advisory, Mail 3725) | `confirmed_voice`, `reject_take` |

  No 04_4 or 08_5 question remains (C8).
- **`asr-differences.json`** lists 36 transcripts, all from the four earlier runs on unchanged cues: numbers and notation, "and" omissions, possessives and spellings, as in stage08. None is from the clarity run.

## Root's commands

Run every command from the repository root with `python3 -I -B`. Each writes only into a new folder named by `--out`. The OUT names below are suggestions; any new folder outside the bound inputs works.

```bash
H=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-audio-preparation01
B=$H/bindings/logarithm-stage09-oliver.json
O=narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage09/audio-preparation01
python3 -I -B $H/prepare_audio.py --binding $B report --check $H/report        # in step: 5 files, 4 questions
```

**1. Selection (root's decision).** Name every cue in script order, or one chapter's cues with `--chapter NN`. `by` and `basis` are required. `binding_sha256` is `2d7f4731…` (the full hash above); `script_sha256` is `559afa36…`. 04_4 and 08_5 must be given as paths:

```json
{"schema": "audio-preparation-selection-v1", "binding_sha256": "2d7f4731d373b5089686f5d345834faeb1a7238dc15007abfc7b4fab420d6803",
 "script_sha256": "559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a", "by": "…", "basis": "…",
 "takes": {"00_1": "00_1-download01.wav", "…": "…",
           "04_4": "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage09/oliver-clarity-takes01/04_4-download0N.wav",
           "08_5": "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage09/oliver-clarity-takes01/08_5-download0N.wav", "…": "…"}}
```

A selection or answers file written for stage08 cannot be reused: the binding and script hashes differ (G1–G3), and the stage08 ids for 04_4/08_5 no longer exist.

**2. Answers.** Start from `report/answers-template.json`.
- Answer the provenance questions only if 00_1 download03 or download04 is selected. The 04_3 voice questions are advisory.
- Any onset or mark id in `anchors.json` may also be given `sample` (`sample_24k`), and a mark `omit_mark`. Every answer needs `by` and `basis`.

**3–6. Decode, prepare, assemble, timings.**

```bash
python3 -I -B $H/prepare_audio.py --binding $B decode --selection SEL.json --out $O/decoded01
python3 -I -B $H/prepare_audio.py --binding $B prepare --selection SEL.json --answers ANS.json \
  --decoded $O/decoded01 --reviewer BlueSalmon --out $O/prepared01
# then, from the repository root: the argv in $O/prepared01/assemble-command.json (workspace coherent_narration.py assemble
# --require-directions), then its "then" line (timings --prepared … --narration … --out …-timings); install with the cp -n
# line in timings-receipt.json
```

Decode uses root's ffmpeg arguments (`-nostdin -n -v error -i SRC -ac 1 -ar 24000 -c:a pcm_s16le DST`) and requires exactly ⌈n/2⌉ frames. Its receipt uses the external label "ElevenLabs UI, Oliver". Prepare dry-runs the spans through the workspace copy of the pinned assembler before it reports success. These steps are the stage08 workflow; that HANDOFF's "Root's workflow" section describes every output.

## Checks

From the repository root:

```bash
python3 -I -B $H/checks/run_checks.py FRESH_SCRATCH > out.txt        # 79 PASS, 0 FAIL, about 40 s
python3 -I -B $H/checks/mutation_check.py FRESH_SCRATCH > out.txt    # M0 79/0; 8 of 8 mutants killed; about 7 min
```

`checks/outputs/run_checks-01.txt` and `checks/outputs/mutation_check-01.txt` are the transcripts. Two fresh run_checks runs against the final bytes gave byte-identical transcripts.

- **A, B.** The bundled report is in step with the real inputs. Two fresh mirror reports equal it byte for byte.
- **C** recomputes independently, with a second tokenizer and a unique-trigram search rather than the helper's alignment:
  - C0: the stage08→stage09 change is exactly 04_4 and 08_5;
  - C1, C2: inventories, superseded and current recordings, recognition coverage, the clarity run's exact records;
  - C3: download names (160 + the 04_3 pair in ui-takes01; all four clarity downloads name the voice); C3b: requests and submissions (each submitted once, bound to its request by hash, recording the voice "visibly selected"); C3c: requests against the inventoried text; C3d: which receipt each inventory hashes;
  - C4: 571 onsets and marks agree, none disagree;
  - C5–C7b: the transcripts above;
  - C8–C10: the exact four questions, the advisory pair, the stage08 content questions reported as not applying.
- **D.** End to end in a scratch mirror where every input is a read-only symlink:
  - the full film through a test copy of the binding, with 04_4/08_5 named by path and root-style samples for 01_1 s1 and 08_5 s1;
  - chapter 01 through the production binding;
  - chapter 04 with a fixture replacement, a superseded-wording fixture and a content question bound to the current 04_4 text;
  - 13 planted faults, including a carried-over stage08 answer id.
  Each run decodes with a labelled stand-in, prepares, assembles with the real assembler copy and writes timings. D6b checks the pinned assembler's reading of the decode receipt.
- **G.** Stale and forged inputs:
  - G1–G4: a stage08 selection, the stage08 script hash, stage08 answers, and a decode receipt from another binding;
  - G5: stage08 pins carried into the stage09 binding, exit 3 (six files);
  - G6–G8b: the old takes by path and the shared names bare;
  - G9–G12: re-pinned inventories and requests forged against each other;
  - G13–G16: the clarity inventory, a request, a downloads receipt and a recording each changed in a fresh mirror under the real binding (exit 3 for pinned files, exit 1 for hashed ones);
  - G17–G19: missing pins, a missing `receipt_hash_of`, a missing clarity run;
  - G20–G22: full coverage, the 78 unchanged cues against the stage08 report, and the path ids.
- **E.** Root's ffmpeg arguments on synthetic tones only. **F.** The helper names no voice; no bundle file holds an absolute home or scratch path.
- **Mutation check.** Eight mutants each remove one stage09 guard from a scratch copy of the helper; run_checks must report a FAIL for every one. **All 8 are killed**, and the unmutated helper passes 79/0 in the same mutation root:

| Mutant | Guard removed | Caught by |
|---|---|---|
| M1 | bare shared file name refused | G8, G8b |
| M2 | path in ids for shared names | A1, B1, D4, G3 |
| M3 | `receipt_hash_of` | A1, then the harness raises (no report) |
| M4 | unhashed downloads receipt must be pinned | G17 |
| M5 | request cue/text check | G9, G10, G11, G12 |
| M6 | superseded-by-path message | G6, G7 |
| M7 | selectable only for the current text | A1, then the harness raises |
| M8 | superseded recording by path refused | G6, G7 |

Test fixtures: the selections and answers in D and G are fixtures. Nobody chose or heard anything. Production audio was never decoded; real ffmpeg ran only on synthetic tones.

## Gaps

- **No listening, fine sync, selection, render, AV or native-kit run.** The kit checks are a Python mirror of `narration.rs`, and I compiled nothing. Root's receipt also records `native_test: pending`.
- **One recognizer on the new takes** (large-v3-turbo). Nothing corroborates their transcripts or onsets.
- **Pronunciation of "e"** in 04_4 is not established by the transcript.
- **08_5-download01 sentence 1** onset caution, above.
- **00_1 twins:** blocking only if selected.
- **Assembler pin.** The binding pins the installed `95615cfa…`. Any later change to `tools/coherent_narration.py`, `tools/narration_cache.py` or `narration.rs` makes it stale (exit 3). That needs a new binding, and selection and answers files bound to it.
- **No v2 receipts** (White's F1): fine for this Oliver binding, open for Frederick bindings (see "Assembler pin").
- **Requests in `ui-takes01` are content-checked, not hash-bound.** full80 hashes the downloads receipts. A request edited without changing its cue or text would pass. Clarity requests are hash-bound through their inventory.
- **Submission receipts** are read by check C3b only; the helper does not bind them. The voice guard checks receipts and download names, not the provider. Root verifies provider identity and settings (requests: Eleven v4, stability 0.5, similarity 0.75, WAV48k).
- Root owns selection, answers, decode, assembly, installation, the TODO and Beads.
