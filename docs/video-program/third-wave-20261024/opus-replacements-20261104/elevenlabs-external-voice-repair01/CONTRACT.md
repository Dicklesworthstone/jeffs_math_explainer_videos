# ElevenLabs external receipt contract (coherent_narration, patched)

This is the receipt that a spans unit names as `generation_receipt`, when its audio came from the ElevenLabs web UI and not from FrankenTTS. It is the input contract of `patch/coherent_narration.diff`, applied to canonical `tools/coherent_narration.py` `e0aa81f3…`.

Every comparison below is exact equality of whole strings or integers. No substring, prefix, case-folded or "contains" test ever accepts a receipt. The single prefix test in the patch is used only to *refuse*: an `ElevenLabs…` label, or an `elevenlabs…` schema, that is not recognised raises an error instead of falling through to the native branch.

## 1. Identity table (in the code)

| `source` label | provider voice (exact) | accepted form |
|---|---|---|
| `ElevenLabs UI, Oliver` | `Oliver - Clean, British and Steady` | legacy (no `schema`), or v2 |
| `ElevenLabs UI, Frederick Surrey` | `Frederick Surrey - Smooth and Velvety` | v2 only |

The voice strings are the ones the UI shows. Frederick's string is byte-identical to the `voice` field of all 105 retained `frederick-stage05-01/*-request01.json` files, and to `VOICE` in Lavender's `complex-frederick-audio-preparation01/frederick_intake.py`. Adding a voice means adding one line to `EXTERNAL_UI_VOICES` and getting it reviewed. That is deliberate.

## 2. Legacy receipt (unchanged)

```json
{"source": "ElevenLabs UI, Oliver",
 "records": [{"cue", "text", "wav", "wav_sha256", "samples", "rate", "source", "source_sha256", "...any extra keys"}],
 "...any extra keys": "e.g. provenance, voice: \"Oliver\", provider, claim"}
```

- It is recognised as legacy only when it has no `schema` key.
- Extra keys are ignored, exactly as before.
- Every one of the 115 retained Oliver assemblies uses this form: none of their receipts has a `schema` key. So do root's `external-receipt.json` files and the decode receipt from `logarithm-stage08-audio-preparation01`. 24 of those assemblies were re-verified with both tools and gave the same outcome (`checks/retained_compat.py`).
- Oliver takes whose requests spell the voice differently should keep this form. The retained Oliver requests use three spellings: exact ×81, `Oliver` ×50, `ElevenLabs Oliver - …` ×3.

## 3. v2 receipt (required for Frederick)

```json
{"schema": "elevenlabs-ui-receipt-v2",
 "source": "ElevenLabs UI, Frederick Surrey",
 "provider": "ElevenLabs",
 "voice": "Frederick Surrey - Smooth and Velvety",
 "records": [{
   "cue": "01_1",
   "text": "<the unit text, which equals the script row>",
   "voice": "Frederick Surrey - Smooth and Velvety",
   "request": "<path of the request JSON named by this take's downloads receipt>",
   "request_sha256": "<sha256 of that file>",
   "source": "<path of the retained take, e.g. .../frederick-stage05-01/01_1-download01.wav>",
   "source_sha256": "<sha256 of that file>",
   "wav": "<path of the decoded 24 kHz mono PCM16 WAV>",
   "wav_sha256": "<sha256 of the decoded WAV file>",
   "samples": 397440,
   "rate": 24000}],
 "...any extra keys": "ignored"}
```

**Paths** are resolved as written, as in the legacy branch: relative to the cwd, which is the repository root for every tool here. The `wav` path must resolve to the same file as the spans record's `wav`.

**Decoding.** Root's arguments, `ffmpeg -nostdin -n -v error -i SRC -ac 1 -ar 24000 -c:a pcm_s16le DST`, give ⌈n/2⌉ frames from a 48 kHz take. `samples` is the decoded frame count, not n/2.

## 4. What the patched assembler checks

On every external receipt (legacy or v2), per unit, in `unit_spans`:

1. The unit is one complete cue with no `context_after`. This is unchanged.
2. `records` is a list of objects, each with a `cue`, and exactly one names this unit's cue.
3. The matching record has every key in §2, and also `voice`, `request` and `request_sha256` for v2. A missing key is a `ValueError` (previously a `KeyError`).
4. These checks are unchanged:
   - `text == unit text`;
   - `wav` resolves to the spans WAV;
   - `wav_sha256 ==` the spans `sha256` `==` the decoded identity;
   - `samples` and `rate` equal the decoded WAV;
   - the digest of `source` equals `source_sha256`.

Additionally, for v2 only:

5. `provider == "ElevenLabs"`.
6. The receipt's `voice` is the table voice for `source`, and every record's `voice` equals it.
7. The digest of `request` equals `request_sha256`. The request JSON's `cue`, `text` and `voice` equal the record's `cue`, `text` and the table voice.

Per assembly, in `assemble` and in `verify` (the branch given a script and plan):

8. **One narrator voice.** Every unit's receipt is classified as one exact voice, or as `native FrankenTTS receipt`. More than one class is refused.
   - Legacy and v2 Oliver are the same voice, so logarithm with targeted v2 replacements stays legal.
   - Frederick with Oliver, or any external voice with a native receipt, is refused. None of the 223 retained assemblies mixes classes.
   - Native voices are not told apart from each other; that is outside this patch.

`narration_delivery.external_plan` now accepts any receipt that `external_ui_voice` recognises, instead of the literal Oliver label. `coherent.verify`, which it calls next, applies rule 8.

## 5. Refusals (message → cause)

| message (prefix) | cause |
|---|---|
| `generation receipt must be a JSON object` | receipt is not an object |
| `external UI receipt names no exact known source label` | v2 with a label not in the table (near misses, trailing space, case, a list) |
| `external UI receipt provider or voice differs from its exact source label` | v2 provider not `ElevenLabs`, or receipt voice not the table voice (`Frederick Surrey`, Oliver's voice on the Frederick label, …) |
| `every external UI record must state the receipt voice exactly` | v2 `records` missing or empty, or any record's voice differs or is absent |
| `unrecognised external UI receipt; a non-legacy label needs schema …` | `ElevenLabs…` label without v2 (including unversioned Frederick), or an unknown `elevenlabs…` schema (even on the Oliver label) |
| `external receipt records must each name a cue` | malformed records |
| `external receipt must identify exactly one source per cue` | zero or duplicate records for the cue |
| `external receipt record lacks …` | missing record key |
| `external source, text or decoded audio changed` | any check in step 4 |
| `external request file changed` | request digest differs |
| `external request does not name this cue, text and voice exactly` | request cue, text or voice differs, or the request is not an object |
| `generation receipt is neither native FrankenTTS nor an identified external UI receipt` | no external label and no native keys (previously a `KeyError`) |
| `one assembly must use one narrator voice, not: …` | rule 8 |

## 6. Deliberately not checked by the assembler

These came out of coordination with Lavender (Mail 3830, answering 3806).

- **Request `submitted` flag.** Every Frederick `request01.json` says `submitted: false` (prepared only). The flag is never read.
- **Submission receipts.** There are two layouts: A, by request name; B, `request_sha256` + `submitted_once`. They are not bound here. The intake binds them.
- **Request `script_sha256`.** Requiring the current script hash would refuse valid takes of *unchanged* cues after any script revision. Logarithm stage09 is that case: 78 takes bind an older script. Exact cue-text equality is the binding that matters.
- **Request `text_sha256`.** This is redundant, since `text` is compared exactly. The intake checks that it hashes its text.
- **"Not an Oliver recording" by content.** This needs an inventory of retained recordings. Lavender's intake already refuses any take whose file or PCM hash matches one of 182 retained Oliver WAVs. Root may still want it in the assembler; it is not in this patch.

The producer of a v2 receipt (a later root step) can copy `request`, `request_sha256`, `source` and `source_sha256` from the intake's `plan.json`. That file records, per take: `path`, `sha256`, and `receipts.request.{name,sha256}`, each hash computed over the bytes the intake validated.

## 7. What this contract does not establish

- A request file records the UI setting the operator chose. It does not prove the speaker of the audio. The assembler cannot hear.
- These are owned elsewhere:
  - that a take is Frederick-only, and that its audio does not match a retained Oliver recording: Lavender's intake;
  - provider-history observation: root;
  - human listening.
- A pass is technical reconstruction only. It is not listening acceptance, voice approval, fine sync or a selection.
