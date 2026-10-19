# Evidence that belongs to the requested cut

`tools/video_evidence.py` runs a bounded checker and retains a receipt, stdout and stderr. Its `check` operation reads the current catalog and actual file bytes. Old receipts are preserved; changes are reported relative to the requested cut instead of overwriting historical results. This is A3's local mechanism. [A4's lifecycle adapter](LIFECYCLE-GATES.md) now connects qualified checker recipes and required receipts to `x` status and native Beads task closure.

## What is bound

Every receipt records the gate, stable video ID, revision ID, voice name, exact artifact bindings and their transitive inputs, actual SHA256 values, checker command/version, dirty checker source hashes, executable hash, UTC, reviewer, duration, outputs, bounded verdict and failure details. The receipt also binds the evidence helper's own source hash. Changes in its policy/implementation conservatively stale old receipts. Merely changing file modification time has no effect. Repository paths and symlinks must resolve to regular files inside the repository.

These role sets are the version-one policy. Each also walks declared dependency inputs, so a model, tool or settings artifact used by a take is included. Inputs missing from the manifest cannot be inferred; producers must record the actual dependency graph.

| Gate | Required roles | Optional roles bound when present |
|---|---|---|
| script | script | lexicon |
| dry_voice | script, voice, take | lexicon |
| timing | script, voice, take, cue | lexicon |
| visuals | script, cue, scene | lexicon, voice, take |
| mix, master | script, voice, take, cue, scene, master | lexicon, music |
| package, public_metadata | script, voice, take, cue, scene, master, package | lexicon, music |

The mix role uses the recorded integrated master artifact; a future separate mix file should be an explicit input of that artifact. A music-free cut can omit music. Adding music later invalidates integrated checks. The optional lexicon role permits scripts with no pronunciation overrides; adding or changing one invalidates relevant checks. A legacy cut without the required artifact records blocks rather than silently passing.

The scope separates metadata from master/dry voice and music from dry voice. It is intentionally conservative about other upstream changes. A fresh pass says only that the recorded checker passed on these bytes. It does not prove that all required gates exist or that the checker's measure establishes identity, accent or delivery quality.

## Checker and CLI contract

The checker runs without a shell in the repository root. Stdout must contain exactly one JSON object:

```json
{"verdict":"pass","output_refs":["relative/path/to/measurement.json"],"summary":"What was actually measured and its bounded result."}
```

`fail` and `unknown` are supported verdicts. Output files are hashed and rechecked. An empty output list is permitted when the embedded structured report is the entire result. Diagnostics belong on stderr. A clean exit without the report, a malformed report, missing output, nonzero exit, timeout, changed inputs or changed checker source cannot produce a passing receipt. A timeout kills the owned checker process group. Limits are 60 seconds by default (explicit values up to 600 seconds) and 1 MiB per output stream; bounded prefixes and failure reasons are retained after a budget failure. A receipt path is exclusively created, never overwritten. Interruption may leave an invalid partial receipt, which blocks.

Example command shape, **only after recording real per-cut artifacts and qualifying the checker**:

```sh
uv run --no-project --with jsonschema python3 -I tools/video_evidence.py \
  --video VIDEO_ID --revision REVISION_ID --gate master \
  run --receipt receipts/unique-run.json --checker-version EXACT_VERSION \
  --checker-file tools/checker.py --reviewer REVIEWER \
  -- python3 -I tools/checker.py

uv run --no-project --with jsonschema python3 -I tools/video_evidence.py \
  --video VIDEO_ID --revision REVISION_ID --gate master check receipts/unique-run.json
```

Repeat `--checker-file` for every relevant source/configuration file the checker reads. Use `--advisory` for metrics that do not qualify a required gate; a passing advisory receipt is rejected when checked for a required gate. Missing/corrupt receipts return exit 1 with explicit reasons. `run` reports the execution verdict; only `check` evaluates current eligibility. No command alters Beads or production gate state.

## Authority and validation limits

Receipts are not signed approvals. This runner does not know whether a supplied checker is trustworthy, whether a human really authorized the named review, or whether the recorded source-file list is complete. A4's fixed trusted-checker mapping therefore starts empty until actual qualification is recorded; arbitrary `echo` output or a prosody metric cannot be enrolled as voice acceptance. A label such as `checker-version` is provenance supplied by the caller; executable/source hashes are the verified pins. `public_metadata` binds local package inputs, but live publication observations also need external freshness policy. A3 does not make a historical web observation current.

`tools/test_video_evidence.py` exercises the actual CLI, subprocess runner and filesystem hashes using clearly synthetic files. The matrix covers every role, two simultaneous cuts, changed voice identity, changed/renamed bytes, mtime-only changes, dirty checker sources, changed outputs, missing/corrupt reports, crashes, timeout/output budgets, mid-check mutation, symlink escape, advisory separation and exclusive receipt retention. This is receipt-mechanism proof, not successful narration, a render, publication or production-checker qualification. Run it with `uv run --no-project --with jsonschema python3 -I tools/test_video_evidence.py`.
