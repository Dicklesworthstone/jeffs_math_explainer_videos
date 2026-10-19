# Exact video cuts and their artifacts

The catalog answers which cut a file or publication belongs to. Beads owns work status, assignment and dependencies. This contract extends catalog v1 without changing the imported video, revision or publication records. It has one consumer: a producer deciding what evidence actually describes the requested cut.

## Inspect and validate

From the repository root:

```sh
uv run --no-project --with jsonschema python3 -I tools/video_catalog.py validate
uv run --no-project --with jsonschema python3 -I tools/video_catalog.py show hoeffding_d
uv run --no-project --with jsonschema python3 -I tools/video_catalog.py show hoeffding_d --revision legacy-C9xOzbLIARg
uv run --no-project --with jsonschema python3 -I tools/test_video_catalog.py
```

Use `--offline` when the dependency is cached. `--catalog PATH` before the subcommand inspects another catalog without writing it. Invalid input returns exit 1 with JSON errors. Both CI and `./x all test` include the validator tests, including the actual catalog. This does not run renders or playback. Current artifact-path checks are lexical; no path is opened, resolved through symlinks or hashed by validation.

## Identity and history

- `video_id` is unique across the catalog and survives title changes.
- A cut's identity is the pair `(video_id, revision_id)`. Repeated revision labels across different videos are legitimate. Duplicate revision IDs within one video fail. The imported catalog already uses that scope.
- Publication records reference a revision within their own video. A `(venue, external_id)` identifies one upload; retain observation receipts in `evidence_refs`. A future cut does not replace the public cut's record. `show` lists all requested cuts and their associated publications, with no implicit "current" cut chosen by order.
- `artifact_id` is unique across the whole catalog. Preserve old records when bytes or production inputs change; assign a new ID and bind the new revision. Validation of one snapshot cannot prove historical immutability.
- Root `artifacts` and revision `artifact_refs` are optional. A legacy cut with `generation_provenance: "unknown"` remains valid. A draft can record only its script. Missing artifacts stay absent: no placeholder file, fake hash or inferred approval.

## Per-cut records

Root `artifacts` holds records with `artifact_id`, `kind`, repository-relative `path`, full `sha256`, direct `inputs` IDs, `evidence_state` and `evidence_refs`. Paths must be normalized POSIX relative paths without `..`, empty components, drive/URI prefixes or backslashes. Each file has its own record. Direct inputs form an acyclic graph; missing inputs fail. Tool, model and settings files can be recorded as input artifacts, so relevant dirty source and exact configuration can be identified without claiming HEAD alone identifies a run.

Each revision's optional `artifact_refs` maps these roles to nonempty arrays of matching artifact IDs:

| Role | What it identifies |
|---|---|
| `script` | Exact narration text/source used by this cut |
| `lexicon` | Pronunciation conditioning used |
| `voice` | Voice preset/reference material used |
| `take` | Selected dry audio files |
| `cue` | Timing/alignment data |
| `scene` | Scene sources or rendered visual assets |
| `music` | Selected music sources/stems |
| `master` | Exact assembled master file(s) |
| `package` | Exact publication metadata/caption/package files |

`model`, `tool` and `settings` are additional artifact kinds usable through dependency inputs. Multiple source files are represented by multiple IDs, not by a directory with an ambiguous hash. A complete synthetic fixture with all roles, a draft and an unknown legacy cut is exercised in `tools/test_video_catalog.py::fixture`. "All roles recorded" is coverage, not approval; a deliberately music-free cut can omit music. The inspection output calls omitted roles `unrecorded_roles`, not missing requirements.

For example, a take may depend on script, lexicon, voice, model, engine and settings artifacts. A master may depend on dry takes, cues, rendered scenes and music. A package may depend on the master and caption sources. Record actual inputs; these examples do not supply missing runtime provenance. An input ID cannot mean "whatever bytes currently live at this path."

Existing legacy `master` records retain their MD5, SHA256, prefix or null digest. If a cut also binds a new master artifact, its legacy path must match exactly one bound master; known SHA256 values must agree. A legacy MD5 and a new SHA256 cannot be compared without checking the actual bytes, which this structural command does not do.

## Evidence and authority

New artifacts use `pass`, `fail`, `unknown`, `stale` or `waived`. These are recorded evidence claims, not computed gate authorization. Historical milestones/master/publication records may retain `historical_record`. A waiver must preserve the failed `measurement`, named `owner`, explicit `disposition`, exact `scope` and `source_ref`. The waiver record accompanies `waived`; it cannot silently appear on a `pass`. This enforces record completeness, not the authenticity of a human authorization.

The CLI rejects unsupported versions, duplicate JSON keys, invalid digest lengths, dangling/ambiguous references, role mismatches and dependency cycles. `manifest_sha256` binds the revision record and its reachable artifact records. It uses sorted object keys and compact UTF-8 JSON; array order is preserved. Changes to a reachable input record change the digest, while an unrelated artifact does not. `catalog_sha256` fingerprints the full parsed catalog, independent of indentation. Neither digest detects a file that changed while its recorded hash was left unchanged.

Content-bound receipts and fresh file hashing/invalidation are implemented separately by the [A3 helper](EVIDENCE-RECEIPTS.md). Driver/native gate enforcement remains A4. No checker in this contract promotes task status, selects a winner, grants publication authority or verifies acoustic quality. Keep the contract while it prevents cut confusion; replace it in place if a later schema supersedes it, retaining historical records.
