# GOVERNANCE.md — program governance machinery (R9)

Risk R9—program bandwidth across eleven workstreams—is rated **High**, and the plan names governance as its mitigation, with a hard stop: breaching governance halts new activation. This document makes that mitigation checkable.

Authority: the Revision-4 comprehensive plan is the source of truth. This document operationalizes §20, §23, §2.9, and the standing stop conditions. Where they disagree, the plan wins and this file is corrected.

---

## 1. Workstream activation and claim integrity

### Governed workstreams

Autonomous work belongs to exactly one of these title-prefix scopes:

```text
G0
W1 through W11
```

The prefix must begin at the first title character and be followed by a word boundary or `:`. `G0: spike`, `W7: drawings`, and `W5/fm-4wt: SIMD` are governed. `W0`, `W12`, `W999`, lower-case `w10`, and embedded `prefix W10` are `UNSCOPED`.

An open unscoped issue remains visible for human repair but is never an autonomous claim candidate. Any active unscoped issue is a governance failure: scope it correctly or release it to `open` before another autonomous claim.

### Activation cap

At most **four governed workstreams** may be simultaneously active. `G0` counts as one workstream. Raising the cap requires an ADR.

A governed workstream is active iff at least one of its issues is `in_progress`. Historical touch does not count. A workstream with landed work and no current claim is dormant; reactivation goes through the same check as fresh activation.

### Claimable leaf

An issue is claimable only when it is:

- `open`;
- scoped to `G0` or `W1`–`W11`;
- unassigned;
- dependency-ready;
- not an epic;
- free of live `parent-child` descendants;
- authorized for autonomous handling by the exact claim-kind policy;
- part of a live blocking graph with no missing blocker or cycle;
- part of a live containment graph with no cycle;
- compatible with the four-workstream activation cap;
- evaluated while no active unscoped issue exists.

A task, bug, or feature with live children is a container regardless of its type label.

### Autonomous claim kind

ADR-0023 defines one closed namespace in the authoritative Beads `labels` field:

```text
agent:claim:auto
agent:claim:manual
agent:claim:external
```

Unlabelled issues default to `auto`. `manual` means the next step is a human judgment, approval, choice, or tracker repair. `external` means the next step requires evidence or an effect outside the current repository execution environment, such as real hardware, a pinned host, release credentials, or independent review. Manual and external leaves remain visible in `non_autonomous_ready` but never enter autonomous ranking.

Exactly zero or one recognized `agent:claim:*` label may appear. An unknown, duplicate, conflicting, or malformed reserved label on any live issue invalidates the plan. Closed historical rows remain graph-bound but do not halt current selection solely because they contain an obsolete reserved spelling.

`scripts/agent_claim_policy.py` is the one interpreter for this namespace. It consumes the canonical labels already exposed by `agent_task_semantics`; it never reparses or edits the ledger. The policy is nested schema `fmn.agent.claim-policy` version 1. Any vocabulary or semantic change requires an ADR and a policy-schema change.

Apply or change these labels only through tracker-native `br` commands. Never hand-edit or reconstruct `.beads/issues.jsonl` to classify work.

### Mandatory pre-claim sequence

```bash
python3 scripts/agent_brief.py --format json --check >/dev/null
python3 scripts/agent_next.py --format json --check

token="$(python3 scripts/agent_claim_guard.py --require)"
issue="${token##*:}"

br show "$issue"
# Check current main, Agent Mail, file reservations, and active peers.

python3 scripts/agent_claim.py \
    --expect-token "$token" \
    --issue "$issue" \
    --assignee "$FMN_AGENT_ID" \
    --command-timeout-seconds 60 \
    --command-output-budget-bytes 16777216 \
    --transition-comment "Claimed after graph, reservation, and HEAD checks" \
    --dry-run

python3 scripts/agent_claim.py \
    --expect-token "$token" \
    --issue "$issue" \
    --assignee "$FMN_AGENT_ID" \
    --command-timeout-seconds 60 \
    --command-output-budget-bytes 16777216 \
    --transition-comment "Claimed after graph, reservation, and HEAD checks"
```

`agent_next.py` version 4 is the scope, leafhood, activation, claim-kind, and recommendation authority. Its additive claim-kind projection carries nested policy schema version 1. `agent_brief.py` remains broad situational context; its historical display grouping cannot override the planner.

Exit identities are governance:

- exit `0`: valid plan/token or verified executor receipt;
- exit `1`: blocking, containment, unscoped-active, claim-policy, or activation state is unsafe;
- exit `2`: ledger, arguments, bounds, token grammar, or resource policy is malformed;
- exit `3`: the graph is valid but has no governed autonomous claimable leaf;
- exit `4`: the token or explicit issue is stale;
- exit `5`: no verified executor receipt because locking, child execution, response validation, export, or postcondition verification failed.

Every nonzero planner, guard, or executor result emits no usable success payload. Never activate work from a failed or empty plan.

A recommendation and token are not a lease. External coordination checks remain mandatory immediately before executor invocation.

### Atomic mutation and recovery

The canonical autonomous mutation uses Beads' storage-level compare-and-set:

```text
br update ISSUE --claim --actor ASSIGNEE --json
```

The executor then runs `br sync --flush-only`, validates the exported represented-field delta, and emits schema `fmn.agent.claim` version 6 only after all proofs succeed.

Exit `5` does not prove native tracker state is unchanged. Inspect:

```bash
br show "$issue"
br sync --status
git status
```

Never replay the old token blindly.

The executor's lock covers cooperating worktrees in one clone's Git common directory. It does not cover another clone, manual Beads commands, Agent Mail, reservations, or unrelated movement of `main`.

---

## 2. Gate ownership

Every gate G0–G5 has a named owner responsible for convening review and recording its evidence packet. The owner of record remains **Jeffrey Emanuel**, who may delegate a verdict under ADR-0018. A chat summary is never a pass.

| Gate | Name | Evidence packet |
|---|---|---|
| G0 | The Laws of the Machine | one ratification note per spike under `docs/g0/`; decision amendments as ADRs; `SUITE.lock` and allowlist committed |
| G1 | Core 2D | primitive self-goldens; {1,4,16} thread proof; path/kernel fixtures; Look Gallery verdicts |
| G2 | The Native Word | tier-1 published-rule verification; span-map fixtures; ratchet dashboard; Gallery verdicts; PG-1(G2) and PG-7 |
| G3 | Depth & Motion | 3D/lighting/camera fixtures; Studio baseline; OQ-12 declaration; PG-3 and PG-A when applicable |
| G4a | The Python Gallery | frozen video corpus; structural runs; Gallery review; TeX gaps; PG-8 class table; clean-wheel runtime parity report with exact overlay identity |
| G4b | Certified Reproducibility | certified-matrix bit manifests; PG-5; PG-1(G4); provenance samples |
| G5 | Distribution & Leapfrogs | release artifacts by tier; selection UX; reproducible-release proof; annex and ratchet records |

A gate passes only when the owner records a verdict against the committed packet. **The packet is the pass.**

### W10/G4 portal parity evidence

A reviewed row in `API_OVERLAY.tsv` is an authored compatibility claim, not runtime proof. For any W10 or G4 handoff that relies on `same` or `improved` rows, the evidence packet must include a report from a freshly installed wheel produced by:

```bash
scripts/check_portal_runtime.sh
```

Run it on CPython 3.13 with the wheel installed (the wheel brings `numpy==2.5.2`) and `IPython==9.14.0`: `source_autoreload` drives a real embedded IPython. A host without IPython stops in the script's preamble.

That gate proves two independent properties:

1. the installed wheel's own `fmn-python --audit-parity --robot` resolves every reviewed SAME/IMPROVED symbol to a non-placeholder runtime value; and
2. the wheel's embedded `API_OVERLAY.tsv` SHA-256 exactly matches the checkout being adjudicated.

A source-only `scripts/check.sh` result, static refusal inventory, import census, generated coverage percentage, or self-audit implementation existing in the tree cannot substitute for this clean-wheel receipt. A nonzero runtime audit is a finding to resolve by real implementation or an evidence-backed overlay correction; it is not waived by the fact that the row was previously reviewed.

---

## 3. Architecture Decision Records

Convention: `docs/adr/NNNN-kebab-title.md`, numbered monotonically and never reused. Status vocabulary: `Proposed → Accepted → Superseded by ADR-NNNN`.

An ADR is required for:

- every amendment to D-01 through D-24;
- every resolution of OQ-1 through OQ-12;
- every policy ruling under a standing rule;
- any change to the four-workstream cap, governed workstream vocabulary, or autonomous claim-kind vocabulary.

When an ADR amends the plan, the plan is edited in the same commit or the ADR records why true-up is deferred and under which Bead.

---

## 4. Review rules before handoff

A workstream may not hand off with failing gates or unwritten fixtures.

1. `scripts/check.sh` is green on one named, unchanged committed source HEAD. A partial run, uncommitted tree, different commit, or hosted-only result is not this evidence.
2. For W10/G4 portal parity claims, `scripts/check_portal_runtime.sh` is additionally green against a freshly installed wheel from that same source checkpoint, and its robot report/overlay hash are retained.
3. UBS is run over changed files; criticals are fixed or explicitly adjudicated in the handoff.
4. New or changed behavior carries tests or fixtures in the same tranche. Self-golden drift is adjudicated, never reflexively re-blessed. A re-bless follows the protocol below.

### Re-bless protocol (fm-5wq.46)

A bit-locked golden proves a frame is unchanged, never that it is right: G1 closed on goldens that had locked vertically mirrored frames for a month (fm-sq8.9). So every change to a golden lock under `crates/fmn-conformance/goldens/` carries its evidence:

1. **The semantic sanity oracles stay green.** `cargo test -p fmn-conformance --test semantic_oracles`, the portal's `semantic_witness.py`, wasm-smoke and the e2e `render_matrix.semantic_oracles.*` scenarios read the built-in semantic witness on every golden-producing route. Orientation, placement, colour and reading order must hold, and a planted vertical mirror must fail. A re-bless that turns them red is a regression, not a bless.
2. **The before/after side-by-side is committed with the lock.** For a lock whose new SHA-256 is `D`, the tree holds `crates/fmn-conformance/goldens/rebless/<lock stem>-<D[..12]>.png`. It is the panel the adjudicator reviewed, composed with `fmn_conformance::golden::side_by_side_png` (before left, after right, magenta divider). A lock that only gains new entries carries the same file, with the new frames on both sides.
3. **The commit message names the adjudication**: what moved, why it is correct, and the bead.

`tests/golden_rig.rs::every_golden_lock_carries_its_rebless_artefact` enforces item 2 for every lock. Digests from before the protocol are its launch baseline (`golden::REBLESS_LAUNCH_BASELINE`); that list is never extended. `a_rebless_without_its_side_by_side_fails_the_protocol` is the planted negative.
5. New semantic divergences from the Reference have Behavior Notes.
6. Beads is trued up: finished work closed with reasons, unfinished claims released with status comments, and follow-ups filed. Ready human/external obligations carry the exact ADR-0023 claim-kind label instead of an invented dependency or prose-only convention.
7. Run `br sync --flush-only`, then stage and commit `.beads/`.
8. Re-run `python3 scripts/agent_next.py --format json --check` against the exported post-mutation ledger.
9. If a next leaf exists, issue a fresh guard token for handoff. A pre-mutation token is stale by construction.
10. Release Agent Mail reservations and post a handoff in the Bead thread.
11. Record exact commits, commands actually run, source HEAD, remaining evidence gaps, and the current next-leaf/no-work/refusal state.

If tracker-native `br` execution is unavailable, do not reconstruct `.beads/issues.jsonl` from a truncated connector response. Leave the tracker untouched and state the limitation.

---

## 5. Stop conditions and leapfrog postponement

| Tripwire | Halt |
|---|---|
| Four governed streams already active | no new workstream activation |
| Active unscoped issue | no new autonomous claim until scoped or released |
| Unknown, duplicate, conflicting, or malformed live `agent:claim:*` label | no new autonomous claim until repaired through `br` |
| Missing blocker, blocking cycle, or containment cycle | no new claim until repaired |
| Planner, guard, or executor nonzero exit | no autonomous claim from that plan/token |
| Claim exit `5` | inspect tracker/native state before retry |
| Current-wheel SAME/IMPROVED runtime contradiction or overlay-hash mismatch | no W10/G4 compatibility handoff until implemented or correctly reclassified |
| Core PG-1 through PG-3 regression | all annex work pauses |
| Purity misclassification | effect class demotes to stateful until root-caused |
| Unadjudicated self-golden drift | introducing change reverts |
| Unlisted governed-closure package | no lands until closure is green |

Enhanced-tier and leapfrog work may be postponed for bandwidth, but postponement never weakens core interfaces and never silently reduces scope. Acceptance criteria travel unchanged to the later gate.

---

## 6. Upstream ledger ritual

Primitives belonging in a foundation crate land there and are tracked in `UPSTREAM_LEDGER.md`.

1. Propose the ledger row with primitive, target repository, owner, status, and coordination dependency.
2. Land the implementation upstream under that repository's governance.
3. Pin the new commit in `SUITE.lock` and update affected allowlist rows in a single-purpose commit.
4. Run the full Gauntlet and diff self-goldens and Look Gallery evidence.
5. Record the adjudication and advance the ledger row. Land only green.
