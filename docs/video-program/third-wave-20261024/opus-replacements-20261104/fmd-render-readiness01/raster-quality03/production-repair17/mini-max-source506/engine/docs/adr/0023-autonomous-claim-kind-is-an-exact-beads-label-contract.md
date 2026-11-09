# ADR 0023: autonomous claim kind is an exact Beads label contract

- **Status:** Accepted
- **Date:** 2026-09-02
- **Task association:** `fm-5wq.4`

## Context

The agent control plane authenticates the complete exported Beads record before an autonomous claim. Descriptions, acceptance criteria, labels, dependency metadata, comments, and future extension fields all participate in the guarded digest. Before this decision, however, only the narrow graph projection influenced recommendation ranking.

That left a semantic gap between authenticated meaning and operational behavior. A dependency-ready leaf whose remaining obligation was a human judgment, a real-hardware receipt, a pinned-host benchmark, or another external observation could compete directly with executable implementation work. Agents could read the prose and decline such a task, but the planner could not express that distinction mechanically. Repeated prose inference wastes context, produces inconsistent decisions, and permits evidence-only work to crowd out code that can actually be completed in the present environment.

The alternative of inferring work kind from titles, descriptions, comments, or acceptance text was rejected. Those fields are unbounded natural language, change over time, and cannot support a stable fail-closed claim contract.

## Decision

Autonomous claim kind is represented by one closed namespace in the authoritative Beads `labels` field:

```text
agent:claim:auto
agent:claim:manual
agent:claim:external
```

The meanings are:

- `agent:claim:auto`: the issue may enter autonomous ranking when every other claimability condition holds;
- `agent:claim:manual`: the issue is ready for a human judgment, choice, approval, or tracker repair and must not be claimed autonomously;
- `agent:claim:external`: the issue is ready only for evidence or an effect outside the current repository execution environment, such as real hardware, a pinned host, a release credential, or an independent review receipt.

An issue with no `agent:claim:*` label defaults to `auto`. This preserves the established Beads corpus and makes the policy opt-out rather than requiring a bulk tracker migration.

Exactly zero or one recognized reserved label may appear on an issue. For every live issue:

- an unknown `agent:claim:*` label is invalid;
- a duplicate reserved label is invalid;
- multiple different reserved labels are invalid;
- a non-array label value or a label array containing non-strings is invalid.

Any such live violation invalidates the planner before it can publish a recommendation or claim token. Historical closed records remain digest-bound but do not halt current live selection merely because they contain an obsolete reserved spelling.

`scripts/agent_claim_policy.py` is the one interpreter for this namespace. It consumes the canonical labels already exposed by `agent_task_semantics`; it does not reread JSONL or create a second authority. `scripts/agent_next.py` projects the policy into every candidate row, exposes manual/external leaves in a separate `non_autonomous_ready` queue, and excludes them from recommendation ranking.

The policy is versioned as nested schema `fmn.agent.claim-policy` version 1. The outer `fmn.agent.next` envelope remains version 4 because no existing field is removed or redefined and the new semantics carry their own explicit versioned contract. Claim tokens still bind the entire planner document and complete task graph, so changing a label, policy result, nested schema version, or queue changes the token digest.

## Consequences

- Authenticated task meaning now affects autonomous selection instead of merely invalidating stale tokens.
- Implementation agents can spend their context and execution budget on work that is actually performable in the current environment.
- Evidence-only and human-decision tasks remain visible and countable; they are not hidden, silently closed, or treated as blocked by invented dependencies.
- Operators can change a task's work kind with a tracker-native label mutation and immediately receive a different guarded plan.
- New work-kind spellings require a new ADR and a claim-policy schema change. Free-form aliases are not accepted.
- The policy does not replace dependency edges, workstream scope, assignment, file reservations, Agent Mail, the activation cap, or the atomic Beads compare-and-set. It answers only whether an otherwise ready leaf may be selected autonomously.
- Existing unlabeled work remains autonomous. Known manual or external obligations should be labeled through `br` when tracker-native execution is available; `.beads/issues.jsonl` must not be reconstructed or hand-edited to apply the policy.

## Verification

`scripts/test_agent_claim_policy.py` proves:

1. unlabeled and explicitly `auto` leaves remain autonomous;
2. higher-priority `manual` and `external` leaves remain visible but cannot outrank executable work;
3. a graph containing only manual/external ready leaves returns a valid no-recommendation plan;
4. unknown, duplicate, conflicting, and malformed reserved labels fail closed with no usable payload;
5. obsolete reserved labels on closed history do not poison current live selection;
6. the machine-readable namespace contract is exact.

The suite is compiled and executed by `scripts/check.sh`. Repository governance still requires the native local gate and tracker-native reconciliation before a Beads task can be closed; connector-only commits and hosted CI are supplemental evidence.
