# Codex technical investigation: production identity, evidence and Beads

Author: SageBasin. Date: 2026-10-16. Consumer: Jeff and BronzeRidge deciding an implementation sequence. Independent source investigation and design proposal; not product implementation, a release qualification, or a listening verdict. No source repositories, Beads, production agents, or models were mutated; no SSH, synthesis, build, or render ran.

## Evidence boundaries and source names

For `file:line` citations below:

- `P` = `<VIDEO_REPO>` (read-only mounted production evidence).
- `L` = `<VIDEO_REPO>` (older local checkout).
- `T` = `<FTTS_REPO>` (local source, not the installed or production engine).
- `B` = `<BEADS_REPO>` (local documentation mirror; installed binary contracts checked separately).
- `W` = this planning directory.

Governing workspace AGENTS/README and relevant repository instructions were read before focused source inspection; requested skills were read for the assigned work. User boundaries override skill defaults that would create Beads, run an orchestrator, modify repositories, render, or demand extra process artifacts. No other wizard output or PLAN.md was read.

Observed fences:

```text
P HEAD: 63674d8075522f95987cc73a8024cbb397ecda3f
L HEAD: 31b515660b9a10b6f9edaa391d466ba601f7a071
T HEAD: 7718a5f6a14f0d7f42b72933af73c9e66aec0dde
P tracked dirt: .beads/issues.jsonl;
                explainer_corpus/visual_proofs/src/chapters/four_triangles.rs
installed br: <LOCAL_BIN>/br, 0.7.4,
  build commit 3956ebb8417414eba883e2446a4cefafa07d0b75,
  aarch64-apple-darwin, self_update feature, no mcp feature
installed bv: /opt/homebrew/bin/bv, 0.12.0
installed ftts: /opt/homebrew/bin/ftts, 0.1.7
additional installed ftts: <LOCAL_BIN>/ftts, 0.1.11
```

These were read live with `git --no-optional-locks ... rev-parse/status`, `command -v`, `br --no-db --json version`, `bv --version`, and both explicit-path `ftts --version` calls. Parent identified the second FTTS location; I independently confirmed it during the assigned plan review. This is PATH ambiguity, not an automatic upgrade prerequisite. Dirty work was preserved. A mounted file may change after this snapshot; cited source findings require rechecking before implementation.

Probe commands were limited to help/version/schema/capabilities and inline, standard-library mathematical calculations. No command opened or changed a live Beads database. Mounted `.beads/issues.jsonl` was parsed directly as read-only text. No `br init`, `create`, `update`, `gate report`, `sync`, `doctor`, or bare `bv` ran. The installed schema command itself says it is not a stable API.

## Material findings

### F01 — Most of the proposed tracking primitives already exist natively

**Demonstrated capability: installed command/schema discovery.** `br create --help`, `br update --help`, `br --no-db --json schema issue`, and `... schema issue-details` expose:

| Requirement | Native field or operation | Boundary |
|---|---|---|
| Stable issue identity and readable name | `id`; `create --slug`; mutable `title` | A slug contributes to issue creation; it is not a native video-identity type. |
| Work assignment | `assignee`, `owner`; `update --claim` | `owner` is documented as owner/email, not approval proof. |
| Workflow | `status`; built-ins include open, in_progress, blocked, deferred, draft, closed | Schema also allows other strings; custom lifecycle semantics require policy and consumer compatibility. |
| Classification | `issue_type`, `labels` | Use labels for video, revision, phase and work-kind conventions. These conventions are application additions. |
| Self-contained work | `description`, `design`, `acceptance_criteria`, `prerequisites`, `notes`, comments | No arbitrary top-level video metadata map was exposed. |
| Dependency graph | parent-child, blocks and other typed dependencies | Containment and prerequisite order are distinct relations. |
| External identities | `external_ref`, source_repo, source_repo_path | One string external_ref is not an unbounded asset registry. |
| Review protection | native `gate report/list`, acceptance checklists, transition comments | Grammar exists; no policy enforcement execution was performed here. |
| Concurrent edits | `update --if-unchanged <updated_at>`; append-notes; checklist updates | Installed help documents compare-and-swap refusal on moved records. |
| Scheduling and capacity | ready, scheduler, policy capacity, estimate, due/defer | These rank work; they cannot select a compelling topic or certify a video. |

Local supporting documentation: `B/README.md:744-767` (configurable ready states), `B/README.md:771-814` (capacity and atomic admission), `B/README.md:816-844` (checklists/comments/gates), `B/README.md:846-870` (hierarchy-aware capacity and rollup). This is documentation support, not proof that every policy described in the source mirror is configured on production.

**Observation:** mounted production `.beads/config.yaml` contains only commented prefix/default examples and `.beads/policy.yaml` is absent. Native gate availability therefore does not imply this production tracker currently enforces the proposed gates.

**Design implication:** do not start by changing the br SQLite schema or inventing a second issue tracker. Compose native issues, labels, dependency edges, comments and local gates with a small artifact-validation adapter.

### F02 — Native gates are useful, but their revision and durability scopes matter

**Demonstrated capability: installed help.**

```text
br gate report <ID> --gate <NAME> --provider <NAME> --status pass|fail
               [--to <STATUS>] [--note <TEXT>] [--json]
br gate list <ID> [--json]
```

`--to` may be omitted only when policy gives one matching target. `br close --help` exposes close policy, request-scoped transition comments, and an audited `--bypass-policy --bypass-reason` path; this is cooperative workflow enforcement, not a cryptographic authorization system.

**Documented semantics:** gate results are scoped to `(issue, source status, target status, status revision)` and append to history. Leaving and reentering the source invalidates earlier passes. They are **project-local metadata and do not sync through JSONL**. Sources: `B/docs/CLI_REFERENCE.md:1459-1502`; `B/README.md:839-844`.

**Inference:** a status revision is not a digest of the current script, speaker vector, model, selected takes, rendered master or metadata package. A same-status content change must not leave an earlier pass effective in the application. Gate results also cannot be the only portable evidence if work moves between clones.

**Proposal:** retain portable receipts/evidence pointers in normal comments and their referenced artifacts. Reuse native gates as the *local transition authorization* computed from those receipts. On another clone, validate the receipt against locally available inputs and report a new local gate result; a missing artifact means `unknown`, not an automatic pass. Never replay a pass solely because another agent said it passed.

### F03 — `agent_context` is inherited governance, not the video catalogue

The installed Issue schema describes canonical JSON governing instructions inherited by descendant issues when `inherited_context.enabled=true`; show/claim emits ancestor context, while list/search do not display it. It mentions a deliberately open JSON shape, but explicitly characterizes the field as per-bead governance. Installed `update --help` confirms emission on descendant show/claim.

**Recommended use:** inherited constraints, named skills, source references and workflow rules, such as “Michael required; no time stretching; owner decides publication.”

**Rejected use:** hide `video_id`, current master hashes, publication status, audience metrics and asset inventories inside agent_context just because it accepts JSON. That turns a governance channel into an invisible metadata database and makes normal triage incomplete.

### F04 — `./x status` and scaffolding are already implemented

`P/x:151-195` contains `new`; `P/x:426-564` contains status including porcelain output and next-command selection. `P/README.md:260` still says both are future work. `P/README.md:23-27,263` also retains older publication summaries despite newer per-video rows and the publishing manifest.

**Inference:** README prose cannot be the lifecycle authority. Reuse and strengthen existing status rather than proposing to invent it from zero. Separate a recorded historical publication from a current build checkout's readiness.

### F05 — Media paths are voice-scoped, but gate marks are not

`P/x:356-375` resolves native voices, `VOICE`, variant base, narration, finals and stems. Fish defaults to matt and Hoeffding to robert; their Michael cuts live under variants. `P/AGENTS.md:43` states the owner chose Michael for published cuts.

`mark()` always writes under `crate/.x` (`P/x:419-423`). Status receives the crate, slug and binary, not the resolved base, and uses `crate/.x` (`P/x:430-441`). Thus the *source implementation* does not distinguish Michael versus an alternate voice in those marks. `P/tools/final_check.py:78-79,110-115` likewise selects the native final directory and reads crate-level marks; absent marks are printed informationally.

**Hypothesis about impact:** a pass from one voice/cut could be misread as another voice/cut's pass. I did not generate two variants or reproduce an incorrect green status. This is a concrete source-grounded scoping defect to test, not evidence that a particular public video is bad.

### F06 — Current freshness tracking is incomplete and mainly based on mtimes

`newer()` ignores absent reference paths and compares modification times (`P/x:443-457`). The stored marker text contains a commit, but status does not parse it. The G6 narration references are script, lexicon and alts (`P/x:517-520`), not the complete set of picks, extra reads, exact FTTS executable, voice bytes, sampler/route, model and selected audio bytes. The status row ordering is useful operational guidance, not a content-bound proof of completed production.

**Positive existing capability:** `P/x:387-413,740-748` hashes narration WAV bytes and ensures they do not change between score timeline and score generation. Preserve this check. Its shown use is within that score invocation; do not relabel it as a durable complete render receipt.

**Proposal:** replace mtime-only freshness for pivotal gates with exact input/artifact binding. Keep mtimes as an inexpensive hint. Missing inputs on a planning mount must produce unknown/unavailable evidence, not a claim of failed production or a ready-to-build command.

### F07 — Some important checks are advisory or can fail without failing the branch

In narrated production, the prosody command is followed by `|| true`; the branch sets a failing status only when a line starts with `FAIL`. If the report never ran or crashed, it prints a warning but can still mark narrated.ok (`P/x:719-735`). Source therefore permits missing prosody execution without that failure becoming the branch exit status. No runtime crash was induced here.

`P/tools/prosody_qa.py:14-35` carefully distinguishes FAIL, FLAG, ADVISORY and PASS. The always-on inner-sentence-silence gate and optional gap gates are different from pitch-reset comparisons; the latter explicitly have no human-calibrated universal cutoff. **Do not promote every flag into a hard defect or claim pitch-reset metrics prove identity/accent.**

The final branch checks pinned engine mode, narration/stem directories, builds and renders, but does not directly inspect all the earlier review/owner gate marks before rendering (`P/x:791-801`). This is a driver-branch observation, not a claim that underlying binaries have no input validation. The plan requires review and owner approval before 4K (`P/docs/planning/COMPREHENSIVE_PLAN_FOR_JEFFS_MATH_EXPLAINER_VIDEOS.md:632-665`). A lifecycle adapter should make the plan's prerequisite contract executable without weakening the underlying checks.

### F08 — “publish” assembles a bundle; publication venues have separate proof

`P/tools/publish.py:244-257` guards GitHub release upload with ALLOW_PUBLISH_UPLOAD. Even the default dry-run creates a bundle, copies media, encodes a proxy and writes checksums (`P/tools/publish.py:285-344`), so it is not a read-only probe. It was not run here. Its upload branch creates a **GitHub release** (`P/tools/publish.py:345-352`), not a YouTube publication.

`P/tools/release.py:321-415` checks the final package and prepares a Private-upload sheet; it does not establish a saved public YouTube state. `manifest_mismatch()` rejects a conflicting recorded hash but explicitly permits no matching row/no hash (`P/tools/release.py:306-318`). Keep that useful mismatch check, but do not describe it as requiring every asset to have a registered, approved cut.

`P/docs/publish/MANIFEST.md:3-14,91-105` records historical Public receipts and separates YouTube from GitHub approval. Its loudness qualification says Michael masters used an accepted quiet-peak exception outside the literal −16 LUFS ±1 doctrine (`P/docs/publish/MANIFEST.md:83-89`). **A future receipt must carry an explicit measured exception/owner disposition, not silently call this an unqualified literal gate pass.**

### F09 — A mount cannot supply missing host-only production evidence

Direct read-only inspection found almost no `.x` markers on the mount: only a matrix protected.ok text marker. These are gitignored by design (plan `P/docs/planning/COMPREHENSIVE_PLAN_FOR_JEFFS_MATH_EXPLAINER_VIDEOS.md:588-601`). The parsed mounted JSONL contained 233 issues: 217 closed, 10 in_progress and 6 open. Examples include a still-in-progress GitHub-release item and a caption-replacement item. Counts/titles alone do not establish their true completion or staleness.

**Current supported statement:** the mounted manifest records eleven selected cuts as Public as of October 15; this run has not independently reloaded Studio or rehashed their masters. Parent browser evidence is the appropriate independent surface. A rendered master, closed production task, native gate mark, upload and Public receipt are distinct facts. Do not reopen old work solely because the planning mount lacks the host's ignored files.

### F10 — Old FTTS doctrine and installed/source identities disagree

The local narration skill still says new seeds give identical reads and requires one sentence per call. Mounted production instructions explicitly correct seed behavior (`P/AGENTS.md:72`) and `P/x:666-673` exposes opt-in seeded candidates and joint choice. The production driver requires FTTS_VERSION default 0.1.11 (`P/x:644-653`); this pane's PATH resolves Homebrew 0.1.7, while the explicit `<LOCAL_BIN>/ftts` reports 0.1.11. Pin and hash the selected executable; do not prescribe an upgrade merely from the PATH result.

Local source likewise demonstrates the danger of treating comments as execution: `T/crates/ftts-cli/src/lib.rs:2197-2200` retains a greedy-oriented comment, whereas `T/crates/ftts-cli/src/synth.rs:2260-2271` selects Production and passes the seed. `T/crates/ftts-model-qwen/src/sampler.rs:104-141,151-164` implements distinct greedy and sampled branches, with determinism scoped to build/ISA/sampler/seed/artifact.

**Rejected claim:** “FTTS is greedy-deterministic, so seed experiments cannot help.” Static source contradicts that blanket statement. **Not established:** which production binary/hash, source revision or chunking route made each complained-about take. Parent owns the deeper FTTS chunk/reset trace after the other investigator hit quota. This memo does not infer that a paragraph submitted to one CLI command becomes one acoustic take.

## Proposed lifecycle system

This section is a design option for the orchestrator's plan and later source implementation. No part of it is installed in this run.

### Separate four kinds of identity

| Object | Proposed identity | What may change independently |
|---|---|---|
| Video concept | Stable `V012` plus a root issue ID, e.g. external_ref `urn:jeffs-math:video:V012` | Working title, public title, folder slug and slate rank. A title is not identity. |
| Cut/revision | `(V012, R01)` with a revision child epic | Script/content revision, voice rendition, production inputs; a later selected cut is another revision. |
| Work item | Native br issue ID | open/in_progress/blocked/deferred/closed, assignee, priority and estimated effort. |
| Venue publication | `(V012, R01, venue, external publication ID)` child task | Private/public state, metadata/captions revisions, post-publication fixes. |

Root and revision issues use native types/parent links; labels such as `video:V012`, `revision:R01`, `phase:narration`, `kind:gate`, `venue:youtube` are conventions. Native description/design fields carry a small documented record of video_id, slug and receipt references; no new arbitrary br field is assumed. Keep the stable root external_ref rather than overwriting it with the latest YouTube URL; publication children can use that URL in external_ref. Metadata/caption-only corrections receive an explicit package revision so a changed description does not pretend the audio changed.

Root status reports whether scoped work remains, not a content lifecycle state. Close it when its explicitly scoped children are delivered or intentionally retired; a new revision reopens scoped work. Preserve earlier revisions and their published evidence. Candidate C01 is not automatically V012: mint a stable production identity only after the owner selects it.

### Reuse native work status; derive production stage from validated evidence

Recommended first version keeps familiar native statuses so bv and existing tooling keep working. Custom statuses are supported in br's schema/policy, but **bv 0.12.0 custom-state semantics were not executed or certified here**. Avoid encoding draft/script/rendered/published as 20 new issue statuses before that integration is proven.

The derived view should show a stage frontier and independent facets, because narration and visual implementation can proceed concurrently. For example:

```text
video_id: V012
selected_revision: R01
math: verified
script: owner-reviewed
visuals: draft-review-pending
narration: identity-listening-pending
delivery: not-attempted
youtube: not-uploaded
github_release: not-approved
frontier: [draft review, voice listening]
```

This is a proposed output shape, not a measured current video status. Each fact needs a status such as pass/fail/unknown/stale/waived with scope and evidence. A closed task alone cannot turn its facet green. The next recommended command comes from the earliest unmet *mandatory prerequisites in the DAG*, not from the newest timestamp or the highest-numbered phase label.

### Gate graph and exact acceptance obligations

| Gate task | Prerequisites | Required evidence before closing / reporting native pass |
|---|---|---|
| Select topic and assign stable identity | Independent briefs and competition check | Owner selection; concrete hook/theorem; sources; differentiation; kill criterion. Likes alone do not pass. |
| Mathematical brief and oracle | Topic selection | Exact theorem and assumptions; independent calculation; edge cases and counterexamples; sources pinned to claims. |
| Storyboard review | Mathematical brief | Pictures earn the abstractions; complete proof route; owner/persona findings have dispositions. Agent prose is review input, not a real viewer trial. |
| Script and claim audit | Storyboard | Script digest; vocabulary/pronunciation decisions; computed quantities agree with visuals; mathematician review. |
| Visual implementation and draft review | Script/brief; can overlap narration preparation | Relevant source digest; build/tests/oracle; glyph/layout/cue/contact-sheet evidence; actual reviewer observation and disposition. |
| Narration | Script audit and exact Michael production profile | Complete lexical/edge/no-stretch gates; take provenance; boundary/identity/accent listening acceptance; no unverified improvement claim. |
| Scored narrated draft | Visual and narration gates | Exact narration/timeline/stem binding; score QA; audible balance/listen-window review; comprehension findings resolved. |
| Owner approval to render final | Reviewed selected draft | Owner decision tied to this revision, voice and draft/package digest, including accepted exceptions. |
| Final and delivery verification | Owner final approval plus all mandatory input gates | Exact master digest, source/toolchain/engine/voice manifest; 4K60/color/loudness/peak/A-V checks; actual final review evidence. |
| YouTube private upload/review | Delivery, canonical package, applicable owner authority | Exact upload/master binding; URL; saved metadata/captions/chapters; private review result. |
| YouTube public verification | Selected private cut, owner publication authority | Fresh reload and public playback; saved title/description, visibility, caption/chapters/source checks, exact selected cut binding. |
| GitHub release | Its own owner authority, delivery and bundle validation | Release URL/tag/assets/checksums and clean retrieval proof; not inferred from YouTube. |
| Post-publication observation | Verified publication | Dated qualitative or analytics observations with exposure/sample context; corrections become new scoped work. Not a prerequisite for declaring that publication happened. |

Containment uses parent-child. Ordering uses explicit blocks edges: a visual draft and narration preparation can be ready together after the audited script; scored-draft work depends on both; public verification depends on upload, review and authority. Do not serialize independent math research behind an FTTS kernel improvement. Do not create a prerequisite cycle by making “measure audience response” block publishing the video that supplies that response.

### A small evidence adapter, using native gates

The missing addition is application-specific evidence binding, not another status database. A receipt stored with production evidence should minimally include:

```yaml
schema_version: 1
video_id: V012
revision_id: R01
gate: narration_identity
verdict: pass  # fail, unknown, stale or waived are application states too
subject_sha256: <selected passage/assembled-audio digest>
inputs:
  script_sha256: <digest>
  voice_sha256: <resolved bytes, not only michael>
  model_manifest_sha256: <digest>
  engine_binary_sha256: <digest>
  route_and_sampler: <recorded settings>
  takes_and_slices_sha256: <digest>
producer: <command/reviewer identity>
observed_at: <timestamp>
evidence: <artifact paths or immutable external references>
exceptions: []
```

This is a **custom receipt schema proposal**, not fields br already understands. Gate-specific validators consume only relevant inputs: changing a thumbnail need not invalidate a mathematical proof, while changing speaker bytes must invalidate voice/mix approval. Record relevant shared-kit/score/engine sources and dirty content hashes too; HEAD alone cannot distinguish uncommitted changes and an unrelated documentation commit should not force an unrelated full render. Exact path and durability policy belong in the implementation brief.

Sequence, performed later by the authorized orchestrator:

1. Validate receipt schema, video/revision/voice, required artifact existence and hashes, and gate-specific obligations.
2. Add a portable normal issue comment with the receipt digest/path, revision scope, command/results, reviewer and accepted exceptions.
3. Report the corresponding native gate with `--to` and a note referencing that exact receipt.
4. Transition/close the matching gate task under policy, supplying acceptance and a new transition comment. No bare policy bypass counts as acceptance.
5. On any relevant input change, make the facet stale and re-enter review/new revision; on another clone, revalidate and re-report because native verdicts do not travel in JSONL.

Provider names and comments are self-asserted in a cooperative local tool. The owner-approval provider should be written only by the orchestrator from an actual owner decision. Do not mistake a named provider for authentication. If strict enforcement is wanted, specify a single writer and access controls separately; no security promise is made here.

### Minimal implementation sequence and high-value tests

The following is a backlog specification for BronzeRidge, not created Beads:

| Work package | Depends on | Deliverable and acceptance |
|---|---|---|
| L1: Native contract/identity pilot | None | Freeze exact production br/bv versions; choose one existing video and preserve all earlier IDs/assets. Map stable identity, revision and venue separately. Native fields only; no schema migration. |
| L2: Evidence receipt and variant-aware resolver | L1 | Resolve selected Michael cut explicitly, validate gate-specific digests, distinguish mount-missing from actual failure. Fixture proof: a matt marker cannot authorize Michael. |
| L3: Native gate adapter and derived status | L2 | Reuse gate report/list and native work statuses; add portable receipt comments. Same-status script/audio change invalidates approval. Reenter-review invalidates old native pass; a second clone cannot inherit gate authorization without revalidation. |
| L4: Closure/owner policy pilot | L3 | One disposable isolated *future* tracker demonstrates missing evidence cannot close; waived literal loudness gate displays exact exception; owner-publication authority bound to selected cut; no bypass reported as pass. No production rollout until this works. |
| L5: Backfill the eleven published identities | L4 plus actual old artifacts/browser receipts | Source/manifest/browser reconciled without changing public videos or reopening accepted cuts on missing mount data. YouTube and GitHub states stay independent. Old unavailable checks marked historical/unknown. |
| L6: Next-video template and recovery | L4 and selected topic | Scaffold identity/gate tasks with explicit dependencies; ready/blocked/bv views agree on runnable work; interrupted gate reporting resumes idempotently; no duplicate publication action. |

Use native `br --json` for execution and exact-path source commits only when later authorized. The orchestrator alone edits Beads in this research run. Appropriate later tests focus on actual failure modes: stale evidence despite preserved mtime, wrong voice, missing prerequisite, missing report, metadata-only revision, dirty source, native-gate portability, and separate release venues. Do not write hundreds of tests that simply restate a status enum.

## Voice investigation: competing mechanisms, not a predetermined fix

The direct complaint is between-line voice change (`W/evidence/audience.md:42-48`), while accent drift, flat prosody, lexical normalization and comprehension have separate observations. There are no timestamped listening judgments by this author and no inferred prevalence. Improving the open-source FTTS/Michael pipeline is the owner preference (`W/evidence/audience.md:9-16,55`).

| Hypothesis | Causal mechanism and distinct prediction | Cheapest discriminator and falsifier | Current status |
|---|---|---|---|
| H1: context resets / independent take choice | Separate sentence conditioning and take selection produce discontinuous register/accent/prosody at joins. Defects concentrate at boundaries, while interiors sound consistent. | Review approved old passages with boundary/interior markers hidden from listeners; inspect actual synthesis segmentation/reset trace. A genuinely continuous matched take should reduce boundary defects. If frozen-profile continuous passages drift just as much internally, context resets alone do not explain it. | Hypothesis. Existing joint chooser means the baseline must include current best sequence choice. |
| H2: weak/ambiguous speaker and accent conditioning | An x-vector may retain broad timbre without tightly preserving accent/phoneme realization. A verified reference-plus-transcript ICL path could help even for isolated sentences. | First identify the exact resolved Michael bytes/mode and authorized reference availability. Compare matched text/seeds using the same reference in x-vector and ICL modes after route qualification. If reference conditioning does not improve blind identity/accent ratings, reject it as the solution for this panel. | Hypothesis. `T/README.md:215-222` documents ICL conditioning and says listening comparison remains open. |
| H3: mixed provenance/cache/variant selection (**third alternative**) | Different engine builds, stale takes or wrong resolved voice assets create apparent speaker switches even without a fundamental model failure. Defects cluster by origin/cache rather than linguistic content. | Audit existing take manifests, actual binary and speaker hashes, picks and variant paths before generating anything. `P/x:644-646` records an earlier mixed-build incident; version-string checking alone is weaker than a binary hash. If all suspect audio is shown to share a frozen provenance profile and still drifts, this mechanism is insufficient for those clips. | Plausible source-grounded mechanism; no claim that the complained-about public cuts used mixed builds. |
| H4: editing and masking create a perceived identity switch | Trimming, gain discontinuity, digital silence, missing consonant edges or music masking changes the perceived voice at a join. Raw interiors may be stable while edited/mixed boundaries sound unstable. | Compare the same recorded samples as raw take, assembled voice-only and final mix; use fixed loudness matching and retain edges. If listeners report the same switch in unedited interior samples, joining/mixing alone cannot explain it. | Hypothesis. Existing edge/spectral gates and masking history make this a concrete alternative, not an invented diagnosis. |

Additional design axis, not a fifth asserted cause: int8/reference numerical route. `T/README.md:135,255` acknowledges open listening evaluation of the full sampled optimized path. It is legitimate to test routes after provenance and segmentation are controlled; it is not legitimate to blame quantization from a generic accent complaint. Identical seed across different routes does not imply identical tokens or an identical acoustic performance.

### Cheapest sequence before any model change

1. Obtain timestamped examples and owner-approved Michael reference clips from existing assets. Record actual offending interval, adjacent lines, raw takes/picks and selected-public-cut identity. A general complaint remains general until localized.
2. Audit provenance and the exact production chunk/reset implementation; distinguish one CLI invocation, one normalized text unit, one generation context and one stitched WAV. Parent supplies the FTTS trace. Do not increase input length until this audit can tell whether reset count actually falls.
3. Evaluate existing audio in raw/voice-only/mix and boundary/interior conditions. This can distinguish editing and boundary hypotheses without synthesizing or rendering anything.
4. Only then run small controlled synthesis experiments on a qualified production binary in new isolated output paths, once authorized. An FTTS 0.1.7 laptop test is not production 0.1.11 validation.

### Proposed controlled listening experiment

**No part of this experiment ran. The thresholds below are proposed pre-registration, not measured success.** Consumer is the owner deciding a default pipeline, not a publication claim that the voice has been fixed.

**Material.** Four 25–45-second math passages from the next selected script: ordinary explanation; proper names/numbers; strongly connected argument; and a passage containing a known troublesome join. Keep exact text/respellings fixed where possible. Use an owner-approved stable Michael anchor. Freeze binary/model/voice/sampler/route, normalization, cleanup, gain target and seed policy; record every resolved hash. Include both boundary-focused 8–12-second excerpts and the full passages. Longer material is necessary in a later qualification to test long-form drift.

**Initial arms.**

- A: best current sentence pipeline, including its current joint selection and thought-based gaps.
- B: one genuinely continuous passage context, if the source trace and run receipt prove that; sample-exact slices afterward, with no time stretching.
- C: the same paragraph submission using normal automatic chunking, recording every boundary/reset. If it is identical to B by implementation, do not pretend they are independent arms; replace C with deliberately matched reset segmentation.
- D, only after available reference/mode qualification: isolated sentence/paragraph in ICL using the same authorized Michael reference and verified transcript. Keep the x-vector comparison; do not introduce another speaker as the treatment.

Start with seeds 1,2,3 per passage/arm, all retained, with a fixed candidate budget and selection rule. Do not give B more audition effort than A or choose only the luckiest B seed. Evaluate the same textual sentence boundaries in all arms, including B's continuous audio; the denominator is fixed target boundaries rather than the number of joins an arm happens to introduce. Randomize presentation order and blind the generation arm. A screen contains anonymous clip IDs, the Michael anchor and five separate questions: same reader, stable English accent, naturalness, lexical completeness, argument comprehension. No model-generated “listener” votes.

**Pilot and main comparison.** Jeff plus 2–3 listeners can screen out broken conditions; treat that small pilot as triage. Then use 10–12 fluent English listeners, including listeners familiar with the intended English accent. Balance passages/arms across listeners and sessions to limit fatigue. The primary statistical unit is the passage/seed performance; repeated listener ratings of the same audio do not create independent synthesis samples. Analyze paired differences and confidence intervals clustered by passage/seed and listener, with the method fixed before unblinding. Four passages provide limited generalization; an interval too wide for a decision is inconclusive.

**Proposed acceptance gate for replacing the default.** Relative to A, require at least a 30% reduction in rated speaker/accent boundary switches per eligible join, with a paired 95% interval excluding no improvement; no more than a 0.25-point loss in mean naturalness on a five-point scale; zero missing/wrong mathematical names or numbers; existing ASR/edge/no-stretch gates pass; comprehension does not decrease by more than 10 percentage points. Jeff must also accept the identity/accent against the frozen anchor. These smallest worthwhile changes are design choices to agree before the trial, not externally validated universal thresholds.

**Kill/stop gates.** Drop an arm immediately for omitted mathematical content, repeatedly clipped endings, unbounded synthesis, or worse identity in both boundary and whole-passage comparisons. Retire “larger takes solve this” if a trace-confirmed continuous arm fails the boundary improvement criterion while provenance and text are controlled. If B improves joins but drifts late within the passage, it is not a default-quality win: test a bounded 2–4-sentence thought unit or different conditioning, not a whole-video take. If intervals remain wide, report inconclusive and commission a larger, specifically powered trial rather than lowering the threshold.

**Long-form confirmation before a new production default.** Test at least two 2–3-minute passages, early/middle/late excerpts, and several topics/seeds. Retain alignment maps and raw takes. Require the short-panel gates plus no new late-passage identity/accent drift. Passage-level success must survive sliced/assembled narration and the final music mix; an isolated pleasing take is not completed video qualification.

## Claim handback for independent audit

| Provisional claim | Type | Decisive anchor / residual unknown |
|---|---|---|
| Native installed br has gate report/list and governance-scoped agent_context, but no arbitrary video-metadata field in the exposed Issue schema. | Executed discovery observation | Help/schema commands above; no mutation/enforcement test. |
| Native gate history is status-transition scoped and local-only, not portable JSONL evidence. | Document-supported capability contract | `B/docs/CLI_REFERENCE.md:1459-1502`; confirm exact production binary integration in pilot. |
| Production status marks are shared across voice variants and freshness omits several content identities. | Static source finding | `P/x:370-375,419-457,517-520`; runtime consequence not reproduced. |
| A missing/crashed prosody report need not fail narrated branch status. | Static source finding | `P/x:719-735`; no failure injected. |
| Old seed doctrine is unreliable for current source, and local PATH can select either 0.1.7 or 0.1.11; the production driver defaults to requiring 0.1.11. | Source plus executed version observation | `T/.../synth.rs:2260-2271`, sampler; `P/x:650-652`; both explicit paths independently probed; no production audio run. |
| Use native gates plus portable revision-bound receipts, standard statuses and explicit venue tasks. | Design recommendation | Must survive fresh independent review and disposable-tracker pilot before source rollout. |
| Longer coherent takes improve Michael consistency. | **Unresolved hypothesis** | Need source-confirmed segmentation and the preregistered listening experiment. |

This author does not assign final audit status to their own claims. Ready for BronzeRidge's independent claim normalization/cross-review. Archive this memo when its implementation plan is superseded; no audio or shipped-capability credit is claimed.
