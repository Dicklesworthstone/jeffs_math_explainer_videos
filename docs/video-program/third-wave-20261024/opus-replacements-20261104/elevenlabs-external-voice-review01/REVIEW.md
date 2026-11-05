# elevenlabs-external-voice-review01: independent review of the ElevenLabs external-voice repair

WhiteSwan, 2026-11-05, for root (BlueSalmon) and BronzeJaguar. This answers Mail 3827 (and Mail 3872). The bundle
reviewed is BronzeJaguar's sealed `P/elevenlabs-external-voice-repair01`: `SHA256SUMS` `3df152e4…`, Mail 3840.
This review is frozen for root and Bronze. `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`.

**Nothing here was compiled, rendered, synthesized, transcribed or heard. No production audio was opened.**
- The retained scan reads JSON only, and every WAV in the scenarios is synthetic integer PCM.
- I did not run the bundle's `frederick_rehearsal.py` or `retained_compat.py`, because they decode or reconstruct
  production audio.
- Nothing was installed. I edited no canonical file, Bead, TODO or file of root's or Bronze's.

## Verdict

**The two diffs do what `CONTRACT.md` says.** I found no defect in the checks they add, and no weakened copy of
them escapes this review's scenarios or the bundle's own suite.

Two Medium findings remain at the edges of the repair, plus one Low:
- **F1, the producer.** Frederick narration cannot reach the repaired assembler through the existing stage08
  helper.
- **F2, the legacy label.** It overrides a voice the receipt itself states.
- **F3 (Low).** A wording error in `CONTRACT.md` §6.

Recommendations:
- Apply both diffs together.
- Add the 9-line fix for F2 (`proposed/legacy-voice-contradiction.diff`, tested below).
- Before connecting Frederick production, give the stage08 helper the v2 fields (F1).

## What holds, against Mail 3827's list

All checks are in `checks/review_checks.py`. The transcript is `checks/review_checks.out`, and every scenario
outcome is in `checks/outputs/outcomes-{base,applied,fixed}.json`. My scenario code shares nothing with the bundle.
Its voice strings are copied from the real request files, not from the patch.

| Asked for | Result | Evidence (scenario names) |
|---|---|---|
| Both tools | Both diffs apply with `git apply`, a different route from the bundle's `patch(1)`. Base + diff equals `patched/tools/` byte for byte. The diffs remove 2 and 1 lines. The canonical suites give 44 tests on base, applied and fixed | transcript: "apply with git apply", "canonical tools/test_*.py suites" |
| Exact provider binding | A wrong or missing `provider` is refused | `provider_wrong`, `provider_missing` |
| Exact voice binding | Refused: the short name, the Oliver voice on the Frederick label, a record voice in lower case, a missing record voice | `receipt_voice_short`, `receipt_voice_oliver_on_fred_label`, `record_voice_case`, `record_voice_missing` |
| Exact request binding | Refused: a request with another voice, a short voice, a neighbour's text, one changed word, a neighbour's cue, the neighbour's request file, a missing voice, a non-object, changed bytes | `request_*` (9 cases) |
| Exact text, hash and sample binding | Refused: the record text of the neighbour, a wrong WAV hash, a sample count one too high, rate 48000, a wrong or changed source, an identical WAV at another path, a duplicated record, a record only for the other cue, a context crop | 10 cases |
| Oliver legacy compatibility | The legacy assembly's outputs are identical on base, applied and fixed, apart from paths. 264 retained span manifests name 908 distinct receipts (89 Oliver UI, 819 native). All 908 are classified as the canonical tool reads them, and none of the 264 mixes kinds. All 91 retained receipts carrying the Oliver label read as Oliver | `oliver_legacy`; transcript: "retained receipts and span manifests" |
| Mixed-voice refusal | Refused: Frederick + Oliver, and external + native (the canonical tool assembled that). After a refusal no output folder exists | `mix_fred_oliver`, `mix_external_native` |
| Mix introduced after assembly | Assembly is all Frederick. One cue's receipt is then swapped for a self-consistent Oliver v2 receipt. `unit_spans` accepts it on its own, so only the guard in `verify` can see the mix, and it does. The delivery CLI refuses it too | `swap_after_assembly`, `delivery_swapped_mix` |
| Malformed or unknown labels | Refused: a near-miss label, a trailing space, lower case with v2, lower case unversioned, a label given as a list, an unknown schema (also under the Oliver label, which the canonical tool accepted), unversioned Frederick, a receipt that is a list (the canonical tool crashed with `AttributeError`), missing keys | `label_*`, `schema_*`, `unversioned_frederick`, `receipt_is_list`, `record_missing_*` |
| Real Frederick request schema | All 105 `frederick-stage05-01/*-request01.json` files carry `cue`, `text` and `voice`, with `voice` exactly `Frederick Surrey - Smooth and Velvety`. All have `submitted: false`; the submission receipts are separate. The positive scenarios use that layout | `fred_shared`, `fred_per_cue` (PCM rebuilt independently, sample for sample) |
| Downstream consumer | The real `narration_delivery.py` CLI, render consumer: <br>• Frederick external plan: exit 0, `external-ui` (the canonical tool fails at assembly with `KeyError 'wav'`) <br>• legacy Oliver: exit 0 on both <br>• native receipts under an external plan: refused <br>• swapped mix: refused | `delivery_*` |
| Negative controls reproduced | My 11 weakened copies (table below) are each caught by my scenarios. The bundle's own suite also catches all 11, so there is no coverage gap to report. Replays: `run_tests.py` ALL PASS; `mutation_check.py` ALL MUTANTS KILLED | transcript: "weakened copies of the repair", "the bundle's own synthetic checks, replayed" |

| Weakened copy | Caught by (mine) |
|---|---|
| X1: no guard in `verify` | `swap_after_assembly`, `delivery_swapped_mix` |
| X2: no guard in `assemble` | `mix_fred_oliver`, `mix_external_native` (the output folder would exist) |
| X3: request digest | `request_bytes_changed` |
| X4: request text | `request_text_neighbour`, `request_text_one_word` |
| X5: request cue | `request_cue_neighbour` |
| X6: provider | `provider_wrong`, `provider_missing` |
| X7: request voice | `request_voice_*` |
| X8: native-key check | `label_lowercase_unversioned` |
| X9: delivery accepts anything | `delivery_native_under_external_plan` |
| X10: delivery diff not applied | `delivery_frederick`, `delivery_swapped_mix` |
| X11: only the delivery diff | 50 scenarios |

## Findings

### F1 (Medium): nothing writes a v2 receipt, and the stage08 helper's receipt is refused for Frederick

**Where.** The decode receipt is written by `P/logarithm-stage08-audio-preparation01/prepare_audio.py:790–797`
(`cmd_decode`, sealed `7e7a1502…`). The patched `coherent_narration.py:46–73` (`external_ui_voice`) reads it.

**What.** The helper writes `{"source": receipt_source, "voice": binding name, "provider": …, "records": [...]}`.
It writes no `schema`, and its records carry no `voice`, `request` or `request_sha256`.
- With the Oliver label, that is a legacy receipt, and it is accepted (`decode_shape_oliver`).
- With the Frederick label, the patched assembler refuses it with "unrecognised external UI receipt; a non-legacy
  label needs schema elevenlabs-ui-receipt-v2" (`decode_shape_frederick`).

The repair defers the producer to "a later root step" (CONTRACT §6). The stage08 handoff says the helper is
voice-agnostic and needs no edit, which is true for logarithm stage09, since it stays Oliver. For complex analysis
and every other Frederick film, though, the helper and the repaired assembler do not connect:
- The failure is safe: the helper's dry run refuses and writes `REFUSED.txt`.
- But Frederick narration cannot be assembled until a producer writes v2.

**Narrow fix.** This is Bronze's file, and the change is root's call. In `cmd_decode`, when `receipt_source` is
not `ElevenLabs UI, Oliver`, also write:
- `"schema": "elevenlabs-ui-receipt-v2"`;
- the receipt `voice` as the exact UI voice, `Frederick Surrey - Smooth and Velvety`. The binding's `name` is a
  short name today (`Oliver` in the stage08 binding), so either the binding names the exact string or the helper
  takes it from `download_source_contains`;
- on each record, `voice` (the same string), and `request` as a repository-relative path: the downloads receipt's
  folder joined to that receipt's `request` name (see F3);
- `request_sha256`, the hash of that file.

`decode_shape_frederick_plus_v2_fields` shows that these additions to the helper's exact shape are enough: the
patched `unit_spans` accepts the result. I did not write the change. The file is sealed, and testing `cmd_decode`
needs ffmpeg on production audio.

### F2 (Medium): the legacy Oliver label wins over a voice the receipt itself states

**Where.** The patched `coherent_narration.py:68–69`, the legacy branch of `external_ui_voice`.

**What.** Any receipt with the label `ElevenLabs UI, Oliver` and no `schema` is classified as Oliver. Its own
`voice` and its records' `voice` and `request` are ignored.

**A concrete failure.** Take a Frederick binding for the stage08 helper whose `receipt_source` was copied from the
Oliver binding. The voice direction has changed twice, so this is a realistic mistake.
- The helper writes `{"source": "ElevenLabs UI, Oliver", "voice": "Frederick Surrey", …}`.
- The patched tool reads it as Oliver (`decode_shape_misbound`).
- Assembled next to a genuine Oliver cue, it passes the one-voice guard and reconstructs
  (`misbound_mixed_with_oliver`: ok).
- A Frederick v2 receipt with its `schema` dropped and the Oliver label put on it also passes
  (`v2_schema_stripped_under_oliver_label`).

The canonical tool behaves the same way, so this is not a regression. But the repair exists to enforce one voice,
and here the receipt contradicts its own label.

**Narrow fix: `proposed/legacy-voice-contradiction.diff`** (`148af01b0cacfaa38f56f87c4db4f537299bb4b6159cc7d8135093efc35dd485`).
- It applies on top of the bundle's patched `coherent_narration.py` and only adds 9 lines.
- It refuses a legacy receipt when the receipt or any record states a voice other than `Oliver` or
  `Oliver - Clean, British and Steady`, or when a record carries `request` or `request_sha256`. The error message
  points to v2.

Evidence that it is safe:
- **Retained data:** none of the 91 retained receipts with the Oliver label states a voice at either level or
  carries a request. More widely, none of the 937 records in all 225 retained ElevenLabs receipts has a record
  `voice` or `request`.
- **Stage08 helper:** it writes `voice: "Oliver"`, which the fix allows.
- **The fixed variant passes:**
  - the 44 canonical tests and the bundle's 17 synthetic tests;
  - every other scenario, with the same outcome;
  - all 908 retained receipts, classified as before;
  - all 91 receipts carrying the Oliver label, which still read as Oliver.
- **The three F2 scenarios** are now refused.

**Limit.** A deliberate relabel that also strips the voice fields still passes. Only v2 binds the voice.

### F3 (Low): CONTRACT §6's "copy `request` from plan.json" gives a bare file name

**Where.** `CONTRACT.md` §6, last paragraph. Lavender's intake (`complex-frederick-audio-preparation01/frederick_intake.py:378–383`)
records `receipts.request.name` from the downloads receipt's `request` field, and that field is a bare name
(`01_1-request01.json` in `frederick-stage05-01/01_1-downloads01.json`).

**What.** v2 resolves `request` against the working directory, which is the repository root. A producer that
follows §6 literally therefore fails on every take with `FileNotFoundError` (`request_bare_name`). The failure is
safe, but it is certain.

**Fix.** Reword §6: `request` is the downloads receipt's folder joined to that name, as a repository-relative path,
and `request_sha256` comes from `plan.json`.

## Notes for root (not defects)

1. **Apply both diffs, coherent first or together.**
   - With only the delivery diff, every external plan stops with an uncaught traceback: X11 gives exit 1,
     "AttributeError: module 'coherent' has no attribute 'external_ui_voice'". The delivery CLI's `except` tuple
     does not include `AttributeError`.
   - With only the coherent diff, Frederick external plans are refused (X10).
2. **The 04_4 and 08_5 Oliver replacements.**
   - In the legacy form nothing binds their request or voice (F2). Written as v2 Oliver, the assembler checks their
     request's cue, text and voice.
   - The `ui-takes01` request layout already has `cue`, `text` and `voice`. The voice must be exactly
     `Oliver - Clean, British and Steady`: 31 of the 80 current requests use that string, and 49 use `Oliver`.
   - v2 and legacy Oliver count as one voice (`oliver_v2_with_legacy`).
   - Separately, `logarithm-clarity-integration01 --validate-receipt` checks these requests. It accepts both
     spellings.
3. **The guard is per assembly and does not tie a voice to a topic.** An all-Oliver complex assembly passes, and so
   does an all-Frederick logarithm assembly. That belongs to the binding and the intake.
4. **v2 does not pair a take with its request.** It binds the request to cue, text and voice, and the source to its
   hash. The pairing of take and request goes through the downloads and submission receipts, which Lavender's
   intake checks (`frederick_intake.py:363–420`). A producer should copy that pairing from `plan.json`, not pair
   files by hand.
5. **The canonical tests are not extended** (bundle decision 5). Once the diffs are applied, nothing in
   `tools/test_*.py` exercises Frederick or the guard. Porting Frederick assemble and verify, a mix refusal and the
   swap after assembly would keep them under the canonical suite.

## Replay (repository root)

```
R=docs/video-program/third-wave-20261024/opus-replacements-20261104/elevenlabs-external-voice-review01
(cd $R && shasum -a 256 -c SHA256SUMS)
python3 -I -B $R/checks/review_checks.py NEW_SCRATCH_DIR     # RESULT PASS, 0 failed; prints checks/review_checks.out
```

It writes only into `NEW_SCRATCH_DIR`, and exits 3 if the bundle's seal or the five canonical base hashes differ.

## Limits

- These are synthetic and metadata checks only. A pass says nothing about whose voice is in a recording. The
  assembler cannot hear, and neither did I.
- **Independence.** Before writing my scenarios and mutants I read the bundle's `HANDOFF.md`, `CONTRACT.md`, both
  diffs, `run_tests.py` and `mutation_check.py`. I did not read `test_external_voice.py`. The native receipt in my
  fixture copies the data shape of the canonical `tools/test_coherent_narration.py`; no code is shared with it.
- **F1 and F2** concern the stage08 helper, which I read but did not run.
- **Note 2** refers to my own `logarithm-clarity-integration01`, so that part is not independent.
