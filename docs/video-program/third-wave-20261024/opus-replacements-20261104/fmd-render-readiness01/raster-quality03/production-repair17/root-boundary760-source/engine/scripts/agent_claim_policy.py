#!/usr/bin/env python3
"""Deterministic autonomous-claim policy derived from authoritative Beads labels.

The Beads ledger remains the only authority. This module interprets one small,
closed label namespace so an autonomous planner can distinguish ordinary work
from tasks that require a human decision or external evidence. Unknown,
duplicate, or conflicting reserved labels fail closed instead of becoming
best-effort prose heuristics.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

import agent_task_semantics

SCHEMA = "fmn.agent.claim-policy"
SCHEMA_VERSION = 1
LABEL_PREFIX = "agent:claim:"
AUTO_LABEL = f"{LABEL_PREFIX}auto"
MANUAL_LABEL = f"{LABEL_PREFIX}manual"
EXTERNAL_LABEL = f"{LABEL_PREFIX}external"
LABEL_TO_MODE = {
    AUTO_LABEL: "auto",
    MANUAL_LABEL: "manual",
    EXTERNAL_LABEL: "external",
}
AUTONOMOUS_MODES = frozenset({"auto"})


class ClaimPolicyError(ValueError):
    pass


@dataclass(frozen=True)
class ClaimPolicy:
    mode: str
    source: str
    label: str | None
    autonomous: bool

    def as_dict(self) -> dict[str, Any]:
        return {
            "schema": SCHEMA,
            "version": SCHEMA_VERSION,
            "mode": self.mode,
            "source": self.source,
            "label": self.label,
            "autonomous": self.autonomous,
        }


def _invalid_policy() -> ClaimPolicy:
    return ClaimPolicy(
        mode="invalid",
        source="invalid",
        label=None,
        autonomous=False,
    )


def _violation(
    issue_id: str,
    code: str,
    *,
    labels: list[str] | None = None,
) -> dict[str, Any]:
    return {
        "issue_id": issue_id,
        "code": code,
        "labels": [] if labels is None else sorted(labels),
    }


def classify_semantic(
    issue_id: str,
    semantic: dict[str, Any],
) -> tuple[ClaimPolicy, tuple[dict[str, Any], ...]]:
    fields = semantic.get("fields")
    if not isinstance(fields, dict):
        return _invalid_policy(), (_violation(issue_id, "invalid-semantic-fields"),)

    labels = fields.get("labels", [])
    if labels is None:
        labels = []
    if not isinstance(labels, list) or not all(isinstance(label, str) for label in labels):
        return _invalid_policy(), (_violation(issue_id, "invalid-labels"),)

    reserved = sorted(label for label in labels if label.startswith(LABEL_PREFIX))
    if not reserved:
        return (
            ClaimPolicy(
                mode="auto",
                source="default",
                label=None,
                autonomous=True,
            ),
            (),
        )

    unknown = sorted(label for label in reserved if label not in LABEL_TO_MODE)
    if unknown:
        return _invalid_policy(), (
            _violation(issue_id, "unknown-claim-label", labels=unknown),
        )
    if len(reserved) != len(set(reserved)):
        duplicates = sorted({label for label in reserved if reserved.count(label) > 1})
        return _invalid_policy(), (
            _violation(issue_id, "duplicate-claim-label", labels=duplicates),
        )
    if len(reserved) != 1:
        return _invalid_policy(), (
            _violation(issue_id, "conflicting-claim-labels", labels=reserved),
        )

    label = reserved[0]
    mode = LABEL_TO_MODE[label]
    return (
        ClaimPolicy(
            mode=mode,
            source="label",
            label=label,
            autonomous=mode in AUTONOMOUS_MODES,
        ),
        (),
    )


def contract() -> dict[str, Any]:
    return {
        "schema": SCHEMA,
        "version": SCHEMA_VERSION,
        "label_prefix": LABEL_PREFIX,
        "default_mode": "auto",
        "labels": {
            "auto": AUTO_LABEL,
            "manual": MANUAL_LABEL,
            "external": EXTERNAL_LABEL,
        },
    }


def classify_issues(
    issues: dict[str, Any],
) -> tuple[dict[str, ClaimPolicy], tuple[dict[str, Any], ...]]:
    try:
        semantics = agent_task_semantics.semantics_for(issues)
    except agent_task_semantics.SemanticError as exc:
        raise ClaimPolicyError(str(exc)) from exc

    policies: dict[str, ClaimPolicy] = {}
    violations: list[dict[str, Any]] = []
    for issue_id in sorted(issues):
        semantic = semantics.get(issue_id)
        if not isinstance(semantic, dict):
            policies[issue_id] = _invalid_policy()
            violations.append(_violation(issue_id, "missing-task-semantics"))
            continue
        policy, issue_violations = classify_semantic(issue_id, semantic)
        policies[issue_id] = policy
        violations.extend(issue_violations)
    return policies, tuple(violations)
