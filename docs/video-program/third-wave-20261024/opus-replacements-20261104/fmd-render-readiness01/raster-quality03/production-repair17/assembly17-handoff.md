# Production17 complete-take assembly helper

Status: implementation and read-only validation complete. No assembly, deployment, build, rendering, synthesis or source mutation was run by this worker. Root owns guarded execution.

`prepare-assembly17.py` SHA-256: `525035f6dd665fd893f8f0d8afe45dea858846465b105cc27dcb4b436b92af89`.

The helper uses the actual `coherent_narration.py assemble` and `verify` commands, followed by the actual `narration_delivery.py --consumer render` check for all 15 chapters. It does not manufacture an assembly receipt or bypass the external UI provider checks. The selected full decoded 08_5/09_1 WAVs replace those two cues; the other 103 WAVs and sentence/phrase timing records must remain identical.

## Root execution contract

Under `/data/projects/jmev-stroke-probe-20261106`:

- New workspace: `work/production-repair17/workspace`.
- Build prerequisite: terminal stable `jobs/csd-production-repair-build17/resource-receipt.json`, its `source-copy.json`, `qualification.json`, and hash-matching `work/bin/production17`.
- Prior assembly: stable `jobs/csd-continuous-rebind16/narration`; original relative generation-receipt paths resolve from `work/production-repo07`, exactly as in rebind16.
- Selected inputs: `qualification01/production-repair17/audio17/mapped-receipt.json` with the two mapped full decoded WAVs, original retained raw sources and requests; `qualification01/production-repair17/timing-selected17.json`.
- Provider-aware adapter: install the exact staged `qualification01/production-repair17/coherent_narration.py` into new workspace `tools/coherent_narration.py`. Preserve the original at **`tools/coherent_narration.before-production17.py`**. The helper checks its original build hash and refuses missing or different backups. Sibling tools remain the build's copies.
- Guard job: **`jobs/csd-production-assembly17`**. The helper requires its active admitted resource receipt. It creates **`jobs/csd-production-assembly17/narration`** with `exist_ok=False`.

Guard child command, after root's guarded adapter installation:

```text
python3 -I -B qualification01/production-repair17/prepare-assembly17.py run --adapter-sha256 <exact staged coherent_narration.py SHA-256>
```

Root's guard input manifest should freeze the helper, Q adapter, timing/selected receipt and every referenced raw/request/decoded file; the stable prior assembly and all its generation/source assets; original16 metadata/tool counterparts; build17 terminal qualification/source-copy receipts and binary; and all unchanged build17 source files. **Do not classify the new workspace canonical plan or adapter as immutable preinputs:** those are explicitly intended outputs. Their original hashes are enforced against `source-copy.json` immediately before mutation. Freeze the original counterparts and the receipt that binds them instead. The copied Rust files and script must remain unchanged.

## Intended workspace changes

Exactly two existing files may differ from the native build snapshot:

1. `tools/coherent_narration.py`: root's exact provider-aware adapter installation.
2. `explainer_corpus/complex_analysis_visual/narration_units.json`: exact new script hash/text for 08_5/09_1, their `external_ui_provider: elevenlabs` flags, and the new map contract. All old IDs and cue order remain.

Exactly three new workspace files are permitted:

- `tools/coherent_narration.before-production17.py`.
- `explainer_corpus/complex_analysis_visual/narration_units.before-production17.json`.
- `explainer_corpus/complex_analysis_visual/docs/narration-direction-map-frederick-production17.json`.

The map rebind resolves each of 23 native hold calls within its exact cue block and each of five helper markers against actual source lines, with current hashes. The 08_5 `after` text becomes the complete approved replacement text. No hold duration, chapter registry, old16 source, or old assembly is changed. Final workspace hashes are written to `workspace-after.json`; unexpected source additions or changes fail closed.

## Results for preview consumers

The success receipt is **`jobs/csd-production-assembly17/assembly17-summary.json`**, schema `production-assembly17-v1`. It must be paired with the guard's successful terminal receipt; the helper cannot assert terminal guard stability while it is running.

- `narration`: absolute new narration directory.
- `script_sha256`: new workspace `script.tsv`.
- `units_sha256`: `narration/external-units.json`.
- `canonical_sha256`: new workspace `narration_units.json`.
- `adapter_sha256`: new workspace `tools/coherent_narration.py`, **not** the assembly JSON.
- `assembly_sha256`, `report_sha256`, `phrase_timings_sha256`: corresponding files in the new narration folder.
- `direction_map_sha256`, `build_binary_sha256`, `build_source_copy_sha256`.
- `retained_wav_cues_byte_identical: 103`, `retained_sentence_and_phrase_cues: 103`, `sample_verified_cues: 105`, and `replaced_cues`.
- `phrase_hook_audit`, all source-marker relocations, actual delivery output, and hashes of closed helper outputs. Active guard logs are deliberately excluded from these hashes; the terminal guard receipt owns those.

Every new cue mark is rebound to the actual assembled WAV hash, because decoded-source and assembled-WAV headers can differ while PCM is identical. The old 103 cue timing records remain exactly equal. New timing is explicitly provisional ASR timing, not heard onset evidence. The adapter must independently validate all generation receipts and reconstruct every cue sample-for-sample.

## Checks actually performed

- Python AST syntax passes.
- Actual approved script and selected timing were checked: **35/35 chapter08 phrase hooks** and **29/29 chapter09 phrase hooks** have exact provisional onset marks. This includes both tuple-driven affine phrases in 08 and the multiline wait in 09. No unsupported phrase hooks were found.
- Removing the constant-integral onset mark produces an explicit unsupported-hook result; changing that phrase in the script raises a missing-phrase error. No estimated fallback is supplied.
- 08 partitions contain 33 sentence spans and 487 word marks; 09 partitions contain 25 sentence spans and 357 word marks. Sentence starts match the complete-take partition, and mark bytes/seconds match their words/sample positions in the approved text. These checks do not establish acoustic correctness.
- All 22 unchanged visual-hold `after` texts equal their complete script cues; the 23rd is the intended 08_5 replacement.
- The inspected original rebind16 helper is byte-identical to the actual CSD helper: SHA-256 `210f684405636de173f739eebc271f5fd01ef160a92854228f480c8cb25870f7`.

Remote execution, sample reconstruction, native delivery acceptance and postassembly source audit remain unrun until root launches the guarded child. No listening, picture, synchronization or publication acceptance is implied.
