# elevenlabs-external-voice-repair01: handoff

Answers root Mail 3785, bead .39. BronzeJaguar, 2026-11-05.

**Canonical is not modified.** The bundle holds a patch, patched copies, tests and checks. Nothing was synthesised, transcribed, built, rendered or heard. No Beads, TODO or commits.

## The defect

`tools/coherent_narration.py:275` (`e0aa81f3…`) treats a receipt as an external UI take only when `source == 'ElevenLabs UI, Oliver'`. A Frederick receipt therefore falls into the native FrankenTTS branch and fails with `KeyError: 'wav'`. The same literal also appears at `tools/narration_delivery.py:268` (`c91914b8…`, in `external_plan`), so the four consumers would refuse a Frederick external plan even after the first fix.

Reproduced on synthetic input (`test_canonical_base_fails_on_frederick_v2`) and on a real Frederick take (`checks/outputs/frederick_rehearsal-01.txt`: `KeyError 'wav'`).

## The repair

Two diffs against the pinned base. The receipt contract is in **`CONTRACT.md`**.

- `patch/coherent_narration.diff` adds:
  - an exact table, `EXTERNAL_UI_VOICES` (Oliver and Frederick Surrey label → exact UI voice);
  - the v2 schema `elevenlabs-ui-receipt-v2`. It requires `provider`, the receipt voice, a voice on every record, and a bound request whose cue, text and voice must match;
  - `external_ui_voice()`, which classifies a receipt, refuses unknown `ElevenLabs…` labels and schemas, and never matches substrings;
  - `assembly_voice()`, the one-voice guard, called in `assemble` and in the script-and-plan branch of `verify`;
  - clear `ValueError`s where malformed external records and keyless native receipts used to raise `KeyError`.

  Every existing exact check is kept: text, WAV path, three-way sha, samples, rate, source digest, one complete cue, no context crop. The legacy unversioned Oliver receipt keeps its old behaviour. Only 2 lines are removed; the rest are additions.
- `patch/narration_delivery.diff` (one condition): `external_plan` accepts what `coherent.external_ui_voice` recognises, instead of the Oliver literal.

| file | sha256 |
|---|---|
| base `tools/coherent_narration.py` | `e0aa81f352cb2c847f53dacf1d37b7d1b6a44526102df7f79329278c278c48d0` |
| base `tools/narration_delivery.py` | `c91914b89bed9e7aca71a649d403fa591bf40ec135f9c3d30f9d5e45c88422ba` |
| `patch/coherent_narration.diff` | `c017af999add12ebe9bb587ca21ed510f049c9a30fd7af8b7af7142a531d3adb` |
| `patch/narration_delivery.diff` | `1600be752c9f31a1661ebea04fbf5d5017dabba61662fe2ba4e323595ae29130` |
| `patched/tools/coherent_narration.py` (= base + diff) | `18c4e75e740943463a7ccecd22f7aaf01e08e19b45c30d0960d6a5e12490ba7d` |
| `patched/tools/narration_delivery.py` (= base + diff) | `961f17cc85dd30eadca73f950ec0bab6e52eb47ad43a7384c3710c97962b1878` |

`narration_cache.py` (`47331efd…`) and both canonical test files are unchanged.

**To apply** (root only), from the repo root:

```
patch -p1 --dry-run < P/elevenlabs-external-voice-repair01/patch/coherent_narration.diff
patch -p1 --dry-run < P/elevenlabs-external-voice-repair01/patch/narration_delivery.diff
```

Then run them without `--dry-run`. `git apply` also works.

## Checks (all from the repo root; P = `docs/video-program/third-wave-20261024/opus-replacements-20261104`; WORK = a new scratch directory)

```
(cd P/elevenlabs-external-voice-repair01 && shasum -a 256 -c SHA256SUMS)
python3 -I -B P/elevenlabs-external-voice-repair01/tests/run_tests.py --work WORK1
python3 -I -B P/elevenlabs-external-voice-repair01/checks/mutation_check.py --work WORK2
python3 -I -B P/elevenlabs-external-voice-repair01/checks/frederick_rehearsal.py --work WORK3
python3 -I -B P/elevenlabs-external-voice-repair01/checks/retained_compat.py --work WORK4
```

Each one writes only into its WORK directory and refuses a WORK that already exists. `run_tests.py` exits 3 if the canonical base no longer has the pinned hashes. Transcripts are in `checks/outputs/`. `run_tests` and the rehearsal transcripts were byte-identical across two runs.

1. **`tests/run_tests.py` → ALL PASS** (`run_tests-01.txt`).
   - Both diffs apply to the pinned base with `patch(1)`, and base + diff equals `patched/tools/` byte for byte.
   - The canonical `tools/test_coherent_narration.py` and `tools/test_narration_delivery.py` pass with **44 tests on the base and 44 on the patched tools**. This includes the existing Oliver UI test, unchanged.
   - **`tests/test_external_voice.py`: 17 synthetic tests**, all passing. The tests generate their own integer PCM.
     - *Positive:*
       - legacy Oliver: assembly output is **byte-identical** to the canonical tool's (report.json, assembly.json, WAVs), and each tool verifies the other's;
       - legacy Oliver with the stage08 helper's extra fields;
       - **Frederick v2 full assemble + verify, with sample-for-sample reconstruction checked independently with `wave`**, using one shared receipt and also per-cue receipts;
       - Oliver v2 together with legacy Oliver in one assembly;
       - the Frederick external plan through the real `narration_delivery.py` render CLI (`plan_mode: external-ui`).
     - *Negative:*
       - malformed: non-object receipt, records missing or empty, record without voice, request or cue, duplicate cue, legacy record missing `samples`, request not an object;
       - wrong source: near-miss labels, trailing space, case, a list, unversioned Frederick, unknown schema (including on the Oliver label), bare voice name with no native keys, wrong or missing provider;
       - wrong voice: `Frederick Surrey`, `Frederick`, Oliver's voice, trailing space, case, all on the receipt, the record and the request;
       - **mixed voice**: Frederick + Oliver, external + native (which the canonical tool still assembles), a receipt swapped to another voice after assembly refused by `verify` and by the render CLI;
       - **incorrect text** in the record, the request text or cue, and the unit;
       - **hash, sample and rate**: `wav_sha256`, spans sha, `source_sha256`, changed download, request sha, changed request, changed decoded PCM, identical copy at another path, samples −1, rate 48000;
       - cue scope: context crop, two-cue unit.
2. **`checks/mutation_check.py` → ALL MUTANTS KILLED** (`mutation_check-01.txt`).
   - Six weakened patches are each caught: no mix guard, no request check, prefix label acceptance, no per-record voice, voice substring, legacy accepting any schema.
   - The unpatched canonical tool, run in the patched slot, fails 13 of the 17 tests.
3. **`checks/frederick_rehearsal.py` → ALL PASS** (`frederick_rehearsal-01.txt`). This is real input, read-only.
   - All 105 `frederick-stage05-01/*-request01.json` files name their cue, the exact voice, and text equal to the combined-repair-stage05 script row and unit text.
   - The stage05 plan is 105 one-cue units.
   - 36 retained takes in 18 downloads receipts still match their hashes.
   - **One take (01_1 download01) was decoded with root's ffmpeg arguments into WORK** (794880 → 397440 frames). It passed the patched v2 branch, and its spans cover every decoded sample.
   - The canonical tool fails on it with `KeyError 'wav'`. Oliver label, short voice name and unversioned label are refused.
   - The sentence ends are **placeholders** (an even split). This is a schema-fit check, not intake.
4. **`checks/retained_compat.py` → 36 agree, 0 differ** (`retained_compat-01.txt`). This is real retained assemblies, read-only.
   - Canonical and patched `verify` give the same outcome on the 24 smallest Oliver UI assemblies and 12 native assemblies whose bound script and plan were found by sha256: 15 Oliver passes, 9 identical Oliver refusals (stale evidence), 11 native passes, 1 identical native refusal.
   - Across all 223 retained assemblies, none mixes receipt classes: 115 are Oliver-only and 108 native-only.

## Coordination with Lavender (Mails 3806 → 3830)

- She confirmed the v2 fields, one request per record, and path resolution as written against the repo root.
- Her intake (`complex-frederick-audio-preparation01`, still unsealed) pins the same voice string.
- It neither decodes nor writes `generation_receipt` files. A later root step writes v2 receipts and can copy request, source and hashes from her `plan.json`.
- Her suggestions not taken, with reasons, are listed in `CONTRACT.md` §6: request `script_sha256` and `text_sha256`, and an Oliver-content check in the assembler.

## Decisions for root

1. **Apply both diffs.** With only the coherent diff applied, `narration_delivery.py` still refuses Frederick external plans for render, captions, score and master.
2. **One-voice guard.** It also refuses native FrankenTTS mixed with external UI. No retained assembly does that (0 of 223), but it is a new refusal the canonical tool did not have. Native voices are not told apart from each other.
3. **Oliver v2 needs the exact request voice** `Oliver - Clean, British and Steady`. Older Oliver requests spell it three ways, so existing Oliver takes should keep the legacy label. The two targeted 04_4/08_5 replacements can use either form. If they use v2, their requests must carry the exact string.
4. **Error types.** Inputs that were already refused with `KeyError` (malformed external records, native receipts missing keys) now get a `ValueError` with a message. `narration_delivery.main` catches both.
5. **Tests are standalone.** I did not edit the canonical test files. If you want them in `tools/test_coherent_narration.py`, the cases port directly.
6. **The table lives in code.** A new voice is a reviewed one-line change, not configuration.

## Limits

- The synthetic tests prove the receipt contract only. A request records the UI setting the operator chose; it does not prove who is speaking. The assembler cannot hear.
- Decoding one real take in scratch is the only audio processing done. No listening, ASR, synthesis, build, render or production write.
- WhiteSwan is assigned the independent review (Mail 3827). I wrote the patch and every test, so nothing here is independent of the patch.
