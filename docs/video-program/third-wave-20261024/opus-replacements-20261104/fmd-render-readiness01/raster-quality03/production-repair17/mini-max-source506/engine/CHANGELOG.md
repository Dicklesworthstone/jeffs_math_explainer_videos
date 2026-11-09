# Changelog

This is the evidence-bounded project changelog for **franken_manim**, a sovereign deterministic rewrite of 3Blue1Brown's `manim` in pure Rust with a separately installed `manimlib`-compatible Python portal.

The repository remains **pre-1.0**. Tagged releases `v0.1.0` through `v0.5.0` are prereleases. Source behavior, compatibility rulings, task state, and release evidence remain distinct:

- source code says what behavior exists;
- `API_SCHEMA.tsv` plus `API_OVERLAY.tsv` say what compatibility status is claimed;
- `.beads/issues.jsonl` says what work is open, active, blocked, or closed;
- gates and retained artifacts say what was actually exercised.

The latest repair checkpoint covered here is **`225b589d6281800723990479c991bc6e13e113fd`** on 2026-09-04. Its production animation changes are `ff49cd4b` and `dd2bdce0`; its executable runtime-audit test restoration is `225b589d`. The earlier **`7aeb3f40a763998d07b43b74613f3c6becc49207`** checkpoint remains documented below as historical runtime-audit work. None of these entries implies a clean-wheel parity pass that was not executed.

---

## v0.5.0 — 2026-10-04 (prerelease)

Pre-1.0 preview cut from `main` (about 2,200 commits after `v0.4.0`). The
lockstep workspace, allowlist rows, standalone locks and installer fallback move
to 0.5.0.

### Highlights since v0.4.0

- Native CLI composition outputs: `--format wav` mixes a scene's sound cues into
  a deterministic S16 soundtrack with FMNP provenance, and `--format svg`
  publishes the final scene state as a deterministic SVG still.
- Bounded multi-team CPU rendering for ordinary, camera-bearing and compiled
  (FMTL) scenes. Native scenes can be exported as standalone, replayable FMTL
  bundles, and camera-bearing bundles open in native Studio.
- A large widening of the `manimlib` portal. Constructors, authoring hooks and
  live updates now go through native-backed lifecycles for axes, planes, number
  lines, text/Tex, braces, surfaces, 3D solids, SVG assets, images, trackers and
  controls. There is also live editing/rebuild at IPython cell boundaries.
- New native library surfaces: markdown documents with mathematics, tables and
  CSV ingestion, graphs and networks, implicit contours, OBJ meshes with
  materials, rasters and material transitions, and sound fades and trimming.
- Conformance tooling: structural-facts differential harness, class sweep,
  corpus speed-ratio producer and certified-arithmetic guard.

### Fixed (release-review blockers)

- **Concurrent WAV/SVG renders could corrupt provenance (#4).** The new
  `--format wav` and `--format svg` CLI paths checked that the destination was
  absent, then published through a *replacing* atomic rename. A competing render
  that won after that check could have its artifact overwritten, leaving its
  `.manifest` sidecar describing different bytes. Both paths now publish with the
  filesystem's create-only primitive (`publish_wav_new` / `publish_svg_new`: a
  synced unique sibling temp file, then an atomic no-clobber hard link), like the
  PNG, GIF, Y4M and video outputs. A losing render is refused by name, leaves the
  winner's bytes and sidecar untouched, and publishes no provenance. Regression
  tests cover a deterministic "competitor wins after preflight" fault injection
  for both formats, and six real concurrent SVG renders into one destination.
  Both tests fail against the previous publishers.
- **Windows `fmn doctor` could load DLLs from the working directory (#3).**
  An unreleased change had enabled the private ffmpeg workdir on Windows, so the
  default `doctor` probe would launch the user's installed ffmpeg. The child
  inherited the caller's working directory, which the Windows loader searches
  for non-KnownDLL imports, and hashing/relocating the EXE does not bind its DLL
  closure. Windows (and every non-Unix target) is back to the v0.4.0 fail-closed
  behavior: the private workdir is refused before any probe is spawned, and
  `doctor` reports ffmpeg as unavailable. The native outputs (PNG, PNG sequence,
  GIF, Y4M, WAV, SVG) need no ffmpeg.

### Known limitations

- Single-file artifacts are committed with a hard link, so an output directory
  on a filesystem without hard links (for example FAT32/exFAT) refuses
  publication after rendering. This is deliberately fail-closed rather than a
  replacing fallback. Render to a native filesystem.
- ffmpeg-backed video export and the ffmpeg doctor probe are unavailable on
  Windows (as in v0.4.0).
- `scripts/check.sh`'s library constructor-authority audit is stale after the
  native `init_points` migrations (#6), and the ADR-0015 unsafe inventory has not
  yet ratified the CPython GC traversal wrappers (#5). Both are tracked
  governance follow-ups; neither is a known runtime defect.
- Same exclusions as v0.4.0: no Linux arm64 artifact, no artifact signatures,
  and no npm publication.

---

## Unreleased — 2026-09-04 Python animation dispatch repairs

### Fixed

- `AnimationGroup`, `LaggedStart`, `Succession`, and `LaggedStartMap` can again construct their native specifications after the wheel installs its semantic continuation. The validator now admits the deferred `None` root only for actual `AnimationGroup` instances, not for an arbitrary class advertising a group-like `_native_kind`. Ordinary animation targets remain strictly validated; Choreo's deferred timing values remain intact.
- `CyclicReplace` and `Swap` retain their targetless native contract. The installed `Transform._native_target` no longer invents a copied target for a class whose `_target_attr` is `None`.
- `Transform.interpolate_mobject` now reaches the installed family dispatcher instead of shadowing it with the old straight-only bootstrap implementation. This restores per-member lag, time-span remapping, path dispatch, style/uniform interpolation, and subclass `interpolate_submobject` hooks. A subclass's whole-mobject override remains authoritative.
- Point-free mobjects skip nonexistent constant data rows while continuing to interpolate uniforms. Locked record fields and locked uniforms remain untouched.
- The new Transform dispatch method participates in the package's existing public method-name, qualified-name, and module normalization.
- Restored the runtime-audit regression source corrupted in `91cd47a7`, preserving its intended post-class staticmethod/classmethod binding fix and every original audit assertion. The corrupt blob failed byte compilation before any test could run; the restored suite passes all 15 tests.

### Regression coverage and evidence

`scripts/test_animation_semantics_installer.py` is wired into `scripts/check.sh` alongside byte compilation of the shipped installer. Its 13 tests compile the actual bootstrap animation class definitions and apply the actual shipped installer, with only Rust-backed mobject storage replaced by a small NumPy fixture. Constructor and interpolation failures were reproduced before their respective fixes and all 13 tests pass afterward.

Also passed: schema provenance (3), runtime audit (15), parity CLI (6), runtime receipt verification (8), Python helper aliases (26), and library constructor authority (18): **89 focused Python tests total**, using CPython 3.13.5 and NumPy 2.3.5. Changed Python files byte-compile and `bash -n scripts/check.sh` passes. Uploaded source blobs were matched to the Git blob hashes of the tested local bytes.

`crates/fmn-python/tests/animation_semantics.py` gained native-facing constructor and subclass-hook cases, but those cases were **not executed** here. The focused fixture tests do not certify Rust storage, native alignment, renderer output, an installed wheel, or cross-platform behavior. No complete Cargo gate, Rust formatting/lint gate, UBS run, or clean-wheel audit is claimed.

### Remaining scope

These are repairs to the already-shipped package continuation, not a completed composite-animation implementation or a native-bootstrap cutover. Direct-extension activation, full native acceptance integration, and clean-wheel execution remain separate work. Existing one-shot workflows and historical source files were left in place; no new workflow, branch, dependency, or file deletion was needed.

The unchanged Beads export still records `fm-5wq.4` and `fm-ai1` as in progress. Tracker-native `br` was unavailable, so this pass did not claim, close, or reconstruct any Bead. The earlier continuation narratives must not be used as evidence that those parent tasks or the missing native integration are complete.

---

## Unreleased — W10 runtime parity truth gate

### Added

- `crates/fmn-python/python/fmn_python/parity_audit.py`, the one runtime interpreter for the authored `[status]` section of `API_OVERLAY.tsv`.
- Machine schema `fmn.portal.runtime-audit` version 1 with deterministic counts, sorted contradictions, status counts, and the SHA-256 of the exact overlay bytes audited.
- Installed-product self-audit entry points:

  ```text
  fmn-python --audit-parity
  fmn-python --audit-parity --robot
  python3 -m fmn_python --audit-parity --robot
  ```

- `scripts/audit_portal_runtime.py`, a checkout wrapper that deliberately reuses the wheel's shared parser/resolver contract.
- `scripts/check_portal_runtime.sh`, an explicit clean-wheel gate that invokes the installed product's own robot audit and verifies the embedded overlay SHA-256 against the checkout.
- `scripts/test_audit_portal_runtime.py` and `scripts/test_portal_parity_cli.py`, both wired into `scripts/check.sh`.

### Changed

- `same` and `improved` overlay rows are now mechanically treated as runtime implementation claims rather than passive reviewed metadata.
- `tiered`, `excluded`, and `unreviewed` rows remain visible classifications but are not falsely interpreted as callable-equivalence promises.
- A wheel self-audits the `API_OVERLAY.tsv` bytes embedded into its native extension, not an arbitrary checkout file.
- Checkout-side and wheel-side auditing share one Python implementation so agents, source gates, and release hosts do not develop different meanings of “implemented.”
- The mandatory source gate runs hermetic parser/resolver and wheel-CLI contract tests but does not pretend that those tests prove a freshly installed wheel.

### Fixed

- A dynamic schema fallback carrying `_fmn_schema_placeholder=True` can no longer silently coexist with a `same` or `improved` compatibility claim without producing a runtime contradiction.
- A missing reviewed symbol or a module import failure is now a fail-closed contradiction rather than an implicit success because the ledger row had already been reviewed.
- A stale wheel can no longer self-audit an old embedded overlay and be mistaken for evidence about the current checkout: the clean-wheel gate compares exact SHA-256 identities after the runtime audit passes.
- Audit mode refuses unrelated scene/CLI arguments instead of accidentally mixing a compatibility proof with normal rendering behavior.

### Typed audit exits

- exit `0`: every reviewed SAME/IMPROVED row resolved to a real non-placeholder runtime value;
- exit `1`: at least one runtime contradiction;
- exit `2`: malformed audit contract or invalid audit arguments;
- namespace collision remains the existing capability exit `4` before native use.

### Focused regression inventory

The committed tests cover:

- reviewed real functions and methods;
- schema-placeholder contradictions;
- missing reviewed symbols and failed module imports;
- tiered/excluded exemptions;
- duplicate, unknown, malformed, and empty status sections;
- deterministic contradiction ordering;
- human and robot CLI success/failure behavior;
- strict audit argument refusal;
- typed malformed-embedded-overlay errors;
- exact `overlay_sha256` binding in the shared report and installed-product robot output.

### Representative commits

- `813dffd` — add the runtime parity contradiction auditor.
- `5df21a6` — lock the initial resolver/parser regressions.
- `f7bf091` — add an installed-wheel parity gate.
- `d835b72` — wire the runtime-audit contract into the mandatory source gate.
- `45cd080` — ship the canonical audit logic in `fmn_python`.
- `cdb1181` — expose `fmn-python --audit-parity [--robot]`.
- `9e9eb18` — remove duplicate checkout/wheel audit semantics.
- `b7406a0` — make the clean-wheel gate invoke the installed product's self-audit.
- `0cc8776` — cover the installed audit CLI hermetically.
- `fd41d02` — execute both runtime-audit suites in the source gate.
- `531de60` / `797a745` — bind wheel and checkout reports to exact overlay bytes.
- `90177c5` — reject stale clean-wheel overlay evidence.
- `e580088` / `7aeb3f4` — regression-lock the overlay identity proof at both layers.

### Evidence boundary

This tranche implements the runtime truth mechanism, the packaged self-audit surface, the clean-wheel freshness gate, and committed hermetic regressions. The connector editing environment did **not** execute those Python tests, build/install the current wheel, run `scripts/check_portal_runtime.sh`, run the complete Cargo/Rust gate, execute UBS, mutate Beads with native `br`, or produce platform/hardware/browser receipts. Hosted Actions runs were pending or cancelled by newer main pushes and are not acceptance evidence.

Accordingly, this changelog does **not** claim that the current portal has zero SAME/IMPROVED contradictions. `fm-5wq.4` remains open; a fresh installed wheel must actually pass the new gate, and any contradiction must be resolved by real implementation or an evidence-backed overlay correction.

---

## Unreleased — exact autonomous claim kinds

### Added

- `scripts/agent_claim_policy.py`, the single interpreter for a closed Beads label namespace:
  - `agent:claim:auto`;
  - `agent:claim:manual`;
  - `agent:claim:external`.
- Nested machine contract `fmn.agent.claim-policy` version 1.
- `non_autonomous_ready`, `invalid_policy_ready`, and per-mode counts in the claim planner.
- Per-candidate policy evidence showing mode, source, exact label, and autonomous eligibility.
- `scripts/test_agent_claim_policy.py`, wired into `scripts/check.sh`.
- ADR-0023 and matching governance rules for work-kind changes, stop conditions, and tracker-native handoff.

### Changed

- Unlabelled issues remain autonomous by default, preserving the existing Beads corpus without a bulk migration.
- `agent:claim:manual` leaves remain visible but cannot be selected by an autonomous worker.
- `agent:claim:external` leaves remain visible but cannot be selected until the required real-world or credential-bound evidence lane is available.
- `agent_next.py` installs the full semantic loader directly and consumes canonical labels already derived from the same bounded ledger bytes.
- The outer `fmn.agent.next` envelope remains version 4; the additive policy semantics carry their own nested version. The claim guard already binds the complete planner document and schema contract.
- Governance now treats malformed live claim labels as a hard stop and requires ready human/external obligations to be classified through `br`, never through hand-edited JSONL or invented dependencies.

### Fixed

- Dependency-ready evidence-only tasks can no longer outrank executable implementation work merely because their numeric priority is higher.
- Authenticated task meaning now participates in recommendation behavior rather than serving only as stale-token invalidation evidence.
- Unknown, duplicate, conflicting, or malformed live `agent:claim:*` labels fail closed before any usable recommendation or token is published.
- Closed historical records with an obsolete reserved spelling remain digest-bound but do not poison current live selection.
- `docs/IMPLEMENTATION_STATUS.md` no longer lists shared governed-scope classification as unfinished; ADR-0022 and its parity regressions already completed that work.

### Focused regression inventory

The new suite covers:

- default-unlabelled and explicit-auto selection;
- manual/external visibility without autonomous ranking;
- valid no-recommendation behavior when only non-autonomous leaves remain;
- unknown, duplicate, conflicting, and malformed reserved labels;
- closed-history tolerance;
- the exact machine-readable policy contract.

### Representative commits

- `2a4071f` — govern autonomous claim policy in code and the mandatory repository gate.
- `35252a0` — accept ADR-0023.
- `19e1e8b` — operationalize the policy in governance.
- `2870405` — replace the stale status narrative with a present-tense evidence map.

### Evidence boundary

The changed Python files passed bytecode compilation, `scripts/check.sh` passed shell syntax validation, and seven focused policy cases passed against a faithful minimal semantic-loader harness before publication. The published blobs matched locally computed Git object hashes and the commit fast-forwarded from the audited `main` head without force.

The editing environment did not contain a complete native checkout, Cargo/Rust execution context, tracker-native `br`, Agent Mail, UBS, release credentials, hardware matrices, or browser/package infrastructure. Hosted Actions runs were queued or pending without completed jobs and were not treated as acceptance evidence. No `.beads/` file was reconstructed or replaced.

---

## Unreleased — full task-semantic guarded claims

### Added

- `scripts/agent_task_semantics.py`, a bounded semantic projection over the exported Beads JSONL authority.
- Claim binding for every top-level issue field outside `agent_brief.Issue`, including:
  - description, design, acceptance criteria, and notes;
  - owner, estimate, creation/source metadata, due/defer values, and labels;
  - unknown future extension fields.
- Claim binding for non-core dependency-record fields, including metadata, thread identity, creation metadata, and unknown future extensions.
- A stable-source proof that derives both full semantics and the broad core model from the same bytes, brackets the established loader with identical before/after reads, and requires all projections to agree.
- Per-record depth and node limits for unknown nested metadata.
- A context-local, exact-ledger-path semantic postcondition used by the existing post-export graph digest.
- `scripts/test_agent_task_semantics.py`, wired into `scripts/check.sh`.

### Changed

- `fmn.agent.claim-graph` advanced from version 2 to version 3.
- `fmn.agent.claim-input` advanced from version 2 to version 3.
- The public guard JSON envelope remains `fmn.agent.claim-guard` version 2.
- The public token spelling remains:

  ```text
  v2:<claim-sha256>:<issue-id-or-none>
  ```

- The executor receipt remains `fmn.agent.claim` version 6; its embedded graph/input schema contract now identifies the stronger version-3 semantic proof.
- Issue-row order, dependency-array order, JSON object-key order, and label order are normalized as harmless representation ordering. Comment order and ordinary array order remain significant.
- `after_graph_sha256` now requires every exported task-semantic field on every issue to remain unchanged before calculating the post-export graph digest.

### Fixed

- Changing a task's description, design, acceptance criteria, notes, owner, estimate, dates, labels, source metadata, dependency metadata, or unknown extension fields now invalidates a previously issued claim token.
- A claim can no longer receive a verified receipt after changing an unmodeled field on the selected issue.
- A claim can no longer receive a verified receipt after changing an unmodeled field on an unrelated issue.
- The semantic projection and broad planning projection can no longer silently come from different ledger states.
- An ABA-style read window in which the ledger changes while being projected is rejected whenever the broad/core or semantic views disagree.
- Remembered postcondition state is scoped to the exact absolute ledger path, preventing one fixture, worktree, or repository from contaminating another.

### Focused regression inventory

The new suite covers:

- token invalidation for every major task-semantic field and unknown nested metadata;
- dependency metadata binding;
- harmless row, dependency, label, and object-key order normalization;
- the permitted `open` → `in_progress` core transition;
- semantic drift on selected and unrelated issues;
- unknown-metadata depth refusal;
- mutation between projections;
- disagreement between the established core loader and the core projection derived from the same stable bytes.

### Representative commits

- `5811d5c` — canonicalize complete task-semantic Beads fields.
- `43feb35` — bind complete semantics into the guarded claim graph.
- `2df9d98` — scope postcondition state to one ledger path.
- `ce33db7` — preserve the v2 guard/token envelope while versioning graph semantics.
- `fb964b1` — add full-semantic guard regressions.
- `3a8ff75` — ratchet graph/input grammar to version 3.
- `d35685f` — execute the semantic suite in the mandatory repository gate.
- `6ec8f5a` — prove semantic and broad projections share one stable source state.
- `c2f0453` — refuse a core projection not derived from the same stable bytes.

### Evidence boundary

The authored semantic module and guard passed Python bytecode compilation, and a focused stub-backed seven-case run passed before the final projection-disagreement regression was added. The committed suite now contains eight tests and is part of `scripts/check.sh`.

This editing environment could not resolve `github.com` for an exact container checkout. GitHub Actions runs triggered by the incremental commits remained pending or queued without jobs. Therefore the complete repository gate, exact current Python suite, live-ledger projection, `br` mutation, Rust axes, UBS, and platform matrices are not claimed as executed for this tranche.

---

## Unreleased — SVG UTF-8 parser hardening

### Fixed

- The preserved 17-byte `svg_document` fuzz reproducer no longer reaches a Rust string-slice panic. The original tokenizer used a fixed nine-byte DOCTYPE slice that could end inside a valid two-byte UTF-8 character.
- The public SVG path now validates bytes and fixed-width markup probes at a UTF-8-safe admission boundary before invoking the private parser.
- Case-insensitive DOCTYPE refusal remains intact.
- Markup-like multibyte text inside a quoted attribute is not misclassified as a tokenizer-level declaration.
- `emit_svg_document` no longer clones the complete retained shape tree merely to call the established emitter.

### Added

- A permanent regression using the exact 17-byte fuzz input.
- Mixed-case DOCTYPE regressions.
- A quoted-attribute multibyte regression.
- A finding README that records root cause, resolution, and the remaining sanitizer-replay boundary.

### Evidence boundary

The source-level panic path is fixed and the regression is committed. A sanitizer replay of the original fuzz target has not been run in this editing environment. The public admission facade currently performs one extra bounded scan before the private parser; collapsing it into a byte-safe tokenizer probe is a later simplification, not a current public safety gap.

---

## Unreleased — scoped autonomous planning

### Added

- An exact autonomous-workstream grammar shared by `scripts/agent_brief.py` and `scripts/agent_next.py`:
  - `G0`;
  - `W1` through `W11`;
  - beginning at the first title character and followed by a word boundary or `:`.
- Explicit `unscoped_leaves` and `unscoped_active` planner evidence.
- Human-brief normalization that uses the planner's governed activation state and workstream labels.
- Cross-layer regressions proving the same scope policy reaches the broad snapshot, planner, claim token, generated human brief, and activation cap.
- ADR-0022, which establishes one shared governed-scope classifier without moving claimability into the broad projection.

### Changed

- `fmn.agent.next` advanced to schema version `4`.
- Open unscoped leaves remain visible for repair but never enter autonomous ranking.
- A valid governed leaf wins even when an unscoped leaf has a numerically higher priority.
- Any active unscoped issue invalidates the planner before token, executor, or brief publication.
- `G0` now counts as one real active workstream. `G0` plus four W-streams is a five-stream cap breach.
- Broad and planner projections now call the same exact classifier and expose parity-tested activation and unscoped-active state.
- Claim tokens bind planner version 4 through the schema contract; older planner tokens cannot revalidate.

### Fixed

- An issue without a governed workstream prefix can no longer be selected or atomically claimed by autonomous tooling.
- Titles such as `W12: ...`, `W999: ...`, lower-case `w10: ...`, or embedded `prefix W10: ...` can no longer masquerade as governed workstreams.
- `G0` is no longer omitted from the activation count.
- The deterministic human brief can no longer accept a `G0` plus four-W-stream cap breach because a projection classified `G0` differently.
- A broad unscoped priority can no longer appear beside a different leaf-safe recommendation as though both were actionable.
- Direct broad JSON and planner JSON can no longer disagree merely because they parsed title scope independently.

### Representative commits

- `88354b6` — refuse unscoped autonomous claims.
- `57d8c3c` — lock scoped-only leaf planning.
- `29c61d1` — enforce `G0` and `W1`–`W11` as the governed vocabulary.
- `4ba640a` — lock the exact workstream grammar.
- `73bf062` — align deterministic human rendering with the planner scope.
- `b66f06e` — preserve blocker truth while normalizing rows.
- `1abc050` — cover G0, unscoped priorities, unscoped active claims, and strict cap refusal.
- `fffc469` — propagate scoped semantics through claim-guard tokens.
- `2a49744` — share the governed classifier with the broad projection.
- `15920a5` — accept ADR-0022.
- `0253820` — execute cross-projection scope parity in the repository gate.

---

## Unreleased — atomic guarded Beads claims

### Added and changed

- `scripts/agent_claim.py` uses Beads' storage-level atomic claim primitive:

  ```text
  br update ISSUE --claim --actor ASSIGNEE --json [--transition-comment TEXT]
  ```

- `fmn.agent.claim` is schema version `6`.
- Atomic response JSON is bounded by byte, structural-depth, and node-count limits and rejects duplicate keys, malformed UTF-8, non-finite numbers, malformed envelopes, multiple updated rows, identity drift, and success-path stderr.
- The response timestamp must agree with the explicitly exported JSONL row.
- Successful mutation receipts include normalized atomic-response evidence and a core-field post-export claim delta.
- The clone-local lock lives in Git's shared `commondir`, so the primary checkout and linked worktrees contend on one persistent inode.
- Claim-input/claim-graph version 3 adds a separate full task-semantic invariant around that core delta.

### Resource policy

Each `br update --claim` and `br sync --flush-only` child has:

- a default 60-second wall-clock deadline, configurable up to 3,600 seconds;
- a default 16 MiB combined stdout/stderr production budget, configurable up to 1 GiB;
- a 1 MiB retained diagnostic ceiling per stream;
- bounded process-tree termination and reader cleanup.

A stalled, continuously producing, or inherited-pipe child cannot hold the claim lock indefinitely. Exit `5` means no verified receipt, not proof that native tracker state is unchanged.

### Canonical authority boundary

The guard and executor now bind all task semantics exported in `.beads/issues.jsonl`, not merely the fields modeled by `agent_brief.Issue`.

This is still a semantic projection rather than a raw-byte or whole-database identity. Harmless JSON/row ordering is normalized, and fields Beads does not export cannot be proven. `br show` and external coordination remain mandatory immediately before invocation.

---

## Unreleased — graph- and policy-bound recommendations

- `agent_claim_guard.py` emits tokens of the form `v2:<claim-sha256>:<issue-id-or-none>`.
- The claim digest binds the canonical full-semantic graph, complete planner output, policy values, and parser/planner/guard schema contract.
- `graph_sha256` remains graph-only evidence; `claim_sha256` is the complete token digest.
- Issue-row, dependency-array, object-key order, and label ordering are canonicalized, while semantic graph, policy, schema, or planner-output changes invalidate the token.
- The literal issue ID `none` is reserved for the no-recommendation sentinel.
- The unsafe broad `agent_brief.py --format next` spelling has been retired.

---

## Unreleased — deterministic agent control plane

- The Beads reader uses bounded regular-file input and strict JSONL framing.
- Duplicate JSON keys and IDs, invalid UTF-8, blank records, missing final LF, non-finite constants, malformed optional arrays, invalid statuses/timestamps, ownership errors, self-edges, duplicate edges, and finite resource-budget violations fail closed.
- Full task-semantic projection adds bounded nested metadata, stable cross-projection reads, and post-export semantic invariants.
- Blocking and containment cycles suppress claims.
- Planner output and generated Markdown derive time from the newest ledger record rather than wall clock.
- Generated brief publication uses descriptor-bound reads, exclusive temporary creation, flush, fsync, and atomic replacement.
- Hosted GitHub Actions availability is not part of the correctness authority chain.

---

## Unreleased — CLI and portal truthfulness

- The shipped `fmn` process publishes stdout and stderr independently and preserves typed exits when downstream pipes close.
- Human `doctor --quiet` behavior is owned by the reusable dispatcher; robot records and typed failures remain visible.
- Batch robot mode emits explicit terminal per-job success records.
- A shipped-binary smoke renders a tiny FMTL scene and verifies native PNG and machine/human manifest publication.
- Rust-only snake_case geometry helpers cannot leak into the pinned Reference wildcard namespace.
- Explicit Python portal refusals are parsed into a canonical bounded inventory; anonymous placeholders fail the mandatory gate.
- Reviewed SAME/IMPROVED portal rows can now be checked against actual runtime values with `fmn-python --audit-parity`, while the clean-wheel wrapper also proves overlay freshness.
- `fm-5wq.4` remains open: reviewed parity rows, inventoried refusals, and an audit mechanism are not universal callable implementation.

---

## Release timeline

| Version | Date | Evidence-bounded summary |
|---|---:|---|
| Project inception | 2026-07-20 | Revision-4 comprehensive plan, workspace governance, Beads graph, and architecture doctrine. |
| `v0.1.0` prerelease | 2026-08-15 | Early native preview: standalone CLI, scene/runtime foundations, renderer, and output stack. |
| `v0.2.0` prerelease | 2026-08-16 | Production Python-render preview: portal scenes produce retained-renderer PNG sequences. |
| `v0.3.0` prerelease | 2026-08-16; published 2026-08-17 | Source-unedited final-still preview for the locked seed corpus. |
| `v0.4.0` prerelease | 2026-08-18 | Native Studio/Metal and threaded-browser preview; installer and WASM package foundations. |
| Current unreleased line | 2026-08-19 onward | Portal convergence, Studio/runtime hardening, distribution gates, executable smoke, parser hardening, and fail-closed agent operations. |

## Major implementation phases

1. **Substrate and semantics:** constants, color, rates, RNG, numeric canonicalization, rational frame time, QuadPath geometry, Marionette state, Choreo timelines, native text, and math.
2. **Rendering and media:** analytic fills/strokes, retained tiles, camera/depth/lighting/textures, native codecs, ffmpeg boundary, FMTL, and WASM foundations.
3. **Front doors and Gauntlet:** Rust API, `fmn`, optional CPython portal, Studio supervisor/worker, API schema, Parity Ledger, self-goldens, differential tests, determinism, fuzzing, performance, and packaging gates.
4. **Portal convergence:** authored `manimlib` compatibility, precise capability refusals, runtime parity self-auditing, executable wheel/bridge tests, and namespace/refusal ratchets.
5. **Agent operations:** strict graph parsing, governed leaf planning, exact claim-kind classification, full-semantic stale-plan tokens, atomic claims, shared-local locking, structured response proof, semantic/core deltas, and bounded subprocess resources.

## Current evidence boundaries

The following are not inferred from a focused source probe or from documentation:

- the complete local repository gate on the latest commit;
- an actual passing `scripts/check_portal_runtime.sh` result for the current wheel;
- sanitizer replay of the repaired SVG fuzz input;
- cross-platform release artifacts;
- real aarch64 topology evidence;
- pinned-host performance-gate receipts;
- platform-native SIMD and certified bit-identity matrices;
- real-browser npm/WASM publication;
- clean-wheel behavior on every supported Python/platform pair;
- ffmpeg/video-container equivalence receipts;
- closure of `fm-5wq.4` or the W10 epic.

Hosted GitHub Actions availability is not a correctness dependency. The authoritative source gate is `scripts/check.sh` run on an exact local or owned-host checkout; installed-wheel parity additionally requires the explicit runtime gate, with source commit and artifact/overlay identity recorded alongside the result.

## Agent notes

- Start with `docs/IMPLEMENTATION_STATUS.md` for present-tense evidence.
- Use `agent_next.py` for governed selection, `agent_claim_guard.py` for the bound token, and `agent_claim.py` for atomic mutation.
- Treat `agent_brief.py` as broad situational context only.
- For W10, build/install a fresh wheel and run `scripts/check_portal_runtime.sh`; a reported contradiction is a direct implementation-or-ledger defect, not something to waive informally.
- Treat Beads as authoritative; commit the `.beads/` export explicitly after every tracker-native mutation.
- Use exact ADR-0023 labels to distinguish autonomous, manual, and externally gated ready work; do not infer work kind from prose.
- On executor exit `5`, inspect Beads and the working tree before doing anything else; never replay the old token blindly.
- Never replace `.beads/issues.jsonl` from truncated connector output.
- Do not turn “reviewed,” “compiled,” “inventoried,” “auditable,” “queued,” or “historically green” into a stronger implementation or release claim.
