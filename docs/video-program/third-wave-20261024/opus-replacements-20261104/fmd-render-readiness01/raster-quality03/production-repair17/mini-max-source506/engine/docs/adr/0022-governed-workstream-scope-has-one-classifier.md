# ADR 0022: governed workstream scope has one classifier

- **Status:** Accepted
- **Date:** 2026-09-02
- **Task association:** `fm-5wq.4`

## Context

The agent control plane is a one-authority/many-projections system. `.beads/issues.jsonl` is the work-state authority; `agent_brief.py` is a broad diagnostic projection; `agent_next.py` is the only autonomous claim planner. Those projections previously interpreted title scope independently:

- the broad projection accepted every anchored numeric `W…` prefix and rejected `G0`;
- the claim planner accepted exactly `G0` and `W1` through `W11`;
- the generated Markdown path repaired the broad result after the fact.

That meant direct `agent_brief.py --format json` and `--check` could report different activation pressure and unscoped state from the claim planner. A compensating renderer made the common document look coherent without making the underlying model coherent.

## Decision

The bounded foundational module `scripts/agent_brief.py` owns one pure governed-scope classifier alongside the `Issue` value object:

- accepted: an anchored `G0` or `W1` through `W11` prefix followed by a word boundary or `:`;
- rejected as `UNSCOPED`: `W0`, `W12+`, lowercase forms, zero-padded forms, and prefixes appearing later in a title.

`scripts/agent_next.py` imports that exact function object and the shared `UNSCOPED` sentinel. It does not carry a second regular expression or wrapper implementation. This does **not** move claimability into the broad projection: leaf analysis, containment, activation-cap enforcement, ranking, and recommendation remain exclusively in `agent_next.py`.

The broad snapshot schema remains version 5 because this restores the already-documented v5 governance contract and aligns direct broad output with the generated projection; it does not add, remove, or rename fields.

## Consequences

- Broad JSON, broad `--check`, generated Markdown, and the claim planner cannot disagree merely because they parsed title scope differently.
- `G0` consumes one active-workstream slot everywhere.
- Out-of-range or malformed prefixes remain visible but never masquerade as governed work.
- Future vocabulary changes have one implementation point and require updating the cross-projection grammar regression.
- `agent_next.py` remains the sole source of autonomous claim recommendations.

## Verification

`scripts/test_agent_scope.py` proves:

1. planner and broad projection expose the same classifier object and sentinel;
2. every accepted and representative rejected title has the intended result;
3. broad and planner activation objects are identical for a mixed governed/unscoped ledger;
4. both projections identify the same unscoped active issue set.

Repository governance still requires the native local gate before a Beads task can be closed. Connector-only commits and hosted CI are supplemental evidence, not substitutes for `br` mutation or the local gate.
