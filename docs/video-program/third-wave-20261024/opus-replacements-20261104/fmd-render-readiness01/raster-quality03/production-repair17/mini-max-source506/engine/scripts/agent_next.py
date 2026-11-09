#!/usr/bin/env python3
"""Fail-closed claim planner layered over :mod:`agent_brief`.

``agent_brief`` owns bounded ledger parsing and blocking-graph analysis. This
module adds the narrower question an autonomous worker actually needs answered:
which *scoped, unassigned, dependency-ready leaf* is the best collision-free
next claim? A non-epic parent with live ``parent-child`` descendants is a
container, not a leaf. Unscoped work stays visible for human repair but can
never be selected autonomously. Exact ``agent:claim:*`` labels separate normal
autonomous work from manual decisions and externally gated evidence. Among
otherwise equal candidates, work that immediately releases more blocked issues
wins deterministically.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import sys
from collections import defaultdict
from pathlib import Path
from typing import Any

import agent_brief
import agent_task_semantics

agent_task_semantics.install_loader()

import agent_claim_policy

SCHEMA = "fmn.agent.next"
SCHEMA_VERSION = 4
MAX_PLAN_OUTPUT_BYTES = 4 * 1024 * 1024
UTC = dt.timezone.utc
UNSCOPED = agent_brief.UNSCOPED
governed_workstream = agent_brief.governed_workstream


class PlanError(ValueError):
    pass


def live_children(issues: dict[str, agent_brief.Issue]) -> dict[str, tuple[str, ...]]:
    """Return live child IDs keyed by their existing parent ID."""

    children: dict[str, list[str]] = defaultdict(list)
    for child in issues.values():
        if child.status not in agent_brief.OPEN_STATUSES:
            continue
        for dependency in child.dependencies:
            if dependency.kind == "parent-child" and dependency.depends_on_id in issues:
                children[dependency.depends_on_id].append(child.id)
    return {parent: tuple(sorted(ids)) for parent, ids in children.items()}


def strongly_connected_components(
    adjacency: dict[str, tuple[str, ...]],
) -> tuple[tuple[str, ...], ...]:
    """Return deterministic nontrivial SCCs without Python recursion."""

    reverse_lists = {issue_id: [] for issue_id in adjacency}
    for issue_id, targets in adjacency.items():
        for target in targets:
            reverse_lists[target].append(issue_id)
    reverse = {
        issue_id: tuple(sorted(targets))
        for issue_id, targets in reverse_lists.items()
    }

    visited: set[str] = set()
    finish_order: list[str] = []
    for root in sorted(adjacency):
        if root in visited:
            continue
        visited.add(root)
        stack: list[tuple[str, int]] = [(root, 0)]
        while stack:
            issue_id, next_index = stack[-1]
            targets = adjacency[issue_id]
            if next_index < len(targets):
                target = targets[next_index]
                stack[-1] = (issue_id, next_index + 1)
                if target not in visited:
                    visited.add(target)
                    stack.append((target, 0))
            else:
                stack.pop()
                finish_order.append(issue_id)

    assigned: set[str] = set()
    components: list[tuple[str, ...]] = []
    for root in reversed(finish_order):
        if root in assigned:
            continue
        assigned.add(root)
        component: list[str] = []
        stack = [root]
        while stack:
            issue_id = stack.pop()
            component.append(issue_id)
            for target in reversed(reverse[issue_id]):
                if target not in assigned:
                    assigned.add(target)
                    stack.append(target)
        if len(component) > 1:
            components.append(tuple(sorted(component)))
    return tuple(sorted(components))


def containment_cycles(
    issues: dict[str, agent_brief.Issue],
    children: dict[str, tuple[str, ...]],
) -> tuple[tuple[str, ...], ...]:
    """Return live ``parent-child`` cycles as deterministic SCCs."""

    live_ids = {
        issue.id
        for issue in issues.values()
        if issue.status in agent_brief.OPEN_STATUSES
    }
    adjacency = {
        issue_id: tuple(child for child in children.get(issue_id, ()) if child in live_ids)
        for issue_id in live_ids
    }
    return strongly_connected_components(adjacency)


def blocker_pressure(
    issues: dict[str, agent_brief.Issue],
) -> tuple[dict[str, int], dict[str, int]]:
    direct: dict[str, int] = defaultdict(int)
    immediate: dict[str, int] = defaultdict(int)
    for issue in issues.values():
        if issue.status not in agent_brief.OPEN_STATUSES:
            continue
        blockers = agent_brief.unresolved_blockers(issue, issues)
        for blocker in blockers:
            direct[blocker] += 1
        if len(blockers) == 1:
            immediate[blockers[0]] += 1
    return dict(direct), dict(immediate)


def rank_key(
    issue: agent_brief.Issue,
    *,
    immediate: dict[str, int],
    direct: dict[str, int],
) -> tuple[int, int, int, int, float, str]:
    return (
        issue.priority,
        -immediate.get(issue.id, 0),
        -direct.get(issue.id, 0),
        1 if governed_workstream(issue) == UNSCOPED else 0,
        -issue.updated_at.timestamp(),
        issue.id,
    )


def build_plan(
    issues: dict[str, agent_brief.Issue],
    *,
    as_of: dt.datetime,
    stale_days: int,
    activation_cap: int,
    limit: int,
) -> dict[str, Any]:
    base = agent_brief.build_snapshot(
        issues,
        as_of=as_of,
        stale_days=stale_days,
        activation_cap=activation_cap,
        limit=limit,
    )
    try:
        policies, all_policy_violations = agent_claim_policy.classify_issues(issues)
    except agent_claim_policy.ClaimPolicyError as exc:
        raise PlanError(str(exc)) from exc

    children = live_children(issues)
    hierarchy_cycles = containment_cycles(issues, children)
    direct, immediate = blocker_pressure(issues)
    dependency_ready = [
        issue
        for issue in issues.values()
        if issue.status == "open" and not agent_brief.unresolved_blockers(issue, issues)
    ]
    assigned = [issue for issue in dependency_ready if issue.assignee is not None]
    containers = [
        issue
        for issue in dependency_ready
        if issue.assignee is None
        and (
            issue.issue_type in agent_brief.NON_CLAIMABLE_TYPES
            or children.get(issue.id)
        )
    ]
    leaf_candidates = [
        issue
        for issue in dependency_ready
        if issue.assignee is None
        and issue.issue_type not in agent_brief.NON_CLAIMABLE_TYPES
        and not children.get(issue.id)
    ]
    scoped_leaves = [
        issue
        for issue in leaf_candidates
        if governed_workstream(issue) != UNSCOPED
    ]
    leaves = [issue for issue in scoped_leaves if policies[issue.id].autonomous]
    non_autonomous_ready = [
        issue
        for issue in scoped_leaves
        if not policies[issue.id].autonomous and policies[issue.id].mode != "invalid"
    ]
    invalid_policy_ready = [
        issue for issue in scoped_leaves if policies[issue.id].mode == "invalid"
    ]
    unscoped_leaves = [
        issue for issue in leaf_candidates if governed_workstream(issue) == UNSCOPED
    ]
    unscoped_active = sorted(
        (
            issue
            for issue in issues.values()
            if issue.status == agent_brief.ACTIVE_STATUS
            and governed_workstream(issue) == UNSCOPED
        ),
        key=agent_brief.issue_sort_key,
    )
    live_ids = {
        issue.id
        for issue in issues.values()
        if issue.status in agent_brief.OPEN_STATUSES
    }
    policy_violations = [
        violation
        for violation in all_policy_violations
        if violation["issue_id"] in live_ids
    ]
    active_workstreams = {
        governed_workstream(issue)
        for issue in issues.values()
        if issue.status == agent_brief.ACTIVE_STATUS
        and governed_workstream(issue) != UNSCOPED
    }
    activation = {
        "cap": activation_cap,
        "active_workstreams": sorted(active_workstreams),
        "count": len(active_workstreams),
        "within_cap": len(active_workstreams) <= activation_cap,
    }
    base_integrity = base["integrity"]
    blocking_ok = bool(base_integrity["within_contract"])
    integrity = {
        "ok": (
            blocking_ok
            and not hierarchy_cycles
            and not unscoped_active
            and not policy_violations
        ),
        "blocking_within_contract": blocking_ok,
        "blocking_cycles": base_integrity["blocking_cycles"],
        "containment_cycles": [list(component) for component in hierarchy_cycles],
        "missing_blockers": base_integrity["missing_blockers"],
        "missing_links": base_integrity["missing_links"],
        "unscoped_active": [issue.id for issue in unscoped_active],
        "claim_policy_violations": policy_violations,
    }

    recommendation: agent_brief.Issue | None = None
    reason: str
    if not blocking_ok:
        reason = "blocking-graph integrity must be repaired before claiming work"
    elif hierarchy_cycles:
        reason = "parent-child containment cycles must be repaired before claiming work"
    elif unscoped_active:
        reason = "unscoped active claims must be scoped or released before claiming work"
    elif policy_violations:
        reason = "claim-policy labels must be repaired before claiming work"
    else:
        active_leaves = [
            issue for issue in leaves if governed_workstream(issue) in active_workstreams
        ]
        candidates = active_leaves or leaves
        if not candidates:
            if non_autonomous_ready and unscoped_leaves:
                reason = (
                    "no autonomous leaf exists; ready leaves require manual/external "
                    "handling or governed workstream scope"
                )
            elif non_autonomous_ready:
                reason = "only manual/external dependency-ready leaves remain"
            elif unscoped_leaves:
                reason = (
                    "only unscoped leaves remain; add a governed workstream prefix "
                    "before claiming"
                )
            else:
                reason = "no dependency-ready unassigned leaf exists"
        elif not active_leaves and len(active_workstreams) >= activation_cap:
            reason = (
                "activation cap is full and no claimable leaf belongs to an active "
                "workstream"
            )
        else:
            recommendation = min(
                candidates,
                key=lambda issue: rank_key(
                    issue,
                    immediate=immediate,
                    direct=direct,
                ),
            )
            if active_leaves:
                reason = (
                    f"best leaf in already-active {governed_workstream(recommendation)}"
                )
            else:
                reason = (
                    f"best leaf; claiming it activates "
                    f"{governed_workstream(recommendation)}"
                )
            if immediate.get(recommendation.id, 0):
                reason += (
                    f"; completion immediately unblocks "
                    f"{immediate[recommendation.id]} live issue(s)"
                )
            elif direct.get(recommendation.id, 0):
                reason += (
                    f"; it participates in blocking "
                    f"{direct[recommendation.id]} live issue(s)"
                )

    def row(issue: agent_brief.Issue) -> dict[str, Any]:
        return {
            "id": issue.id,
            "priority": issue.priority,
            "workstream": governed_workstream(issue),
            "issue_type": issue.issue_type,
            "assignee": issue.assignee,
            "updated_at": issue.updated_at.isoformat().replace("+00:00", "Z"),
            "title": agent_brief.compact_title(issue.title),
            "live_children": list(children.get(issue.id, ())),
            "direct_unblocks": direct.get(issue.id, 0),
            "immediate_unblocks": immediate.get(issue.id, 0),
            "claim_policy": policies[issue.id].as_dict(),
        }

    activates = bool(
        recommendation is not None
        and governed_workstream(recommendation) not in active_workstreams
    )
    sort = lambda issue: rank_key(issue, immediate=immediate, direct=direct)
    return {
        "schema": SCHEMA,
        "version": SCHEMA_VERSION,
        "as_of": as_of.astimezone(UTC).isoformat().replace("+00:00", "Z"),
        "integrity": integrity,
        "activation": activation,
        "claim_policy": agent_claim_policy.contract(),
        "recommendation": {
            "issue": row(recommendation) if recommendation is not None else None,
            "reason": reason,
            "activates_workstream": activates,
        },
        "counts": {
            "claimable_leaves": len(leaves),
            "non_autonomous_ready": len(non_autonomous_ready),
            "manual_ready": sum(
                policies[issue.id].mode == "manual"
                for issue in non_autonomous_ready
            ),
            "external_ready": sum(
                policies[issue.id].mode == "external"
                for issue in non_autonomous_ready
            ),
            "invalid_policy_ready": len(invalid_policy_ready),
            "unscoped_leaves": len(unscoped_leaves),
            "unscoped_active": len(unscoped_active),
            "assigned_ready": len(assigned),
            "ready_containers": len(containers),
            "non_epic_containers": sum(
                issue.issue_type not in agent_brief.NON_CLAIMABLE_TYPES
                for issue in containers
            ),
            "containment_cycles": len(hierarchy_cycles),
            "claim_policy_violations": len(policy_violations),
        },
        "claimable_leaves": [row(issue) for issue in sorted(leaves, key=sort)[:limit]],
        "non_autonomous_ready": [
            row(issue) for issue in sorted(non_autonomous_ready, key=sort)[:limit]
        ],
        "invalid_policy_ready": [
            row(issue) for issue in sorted(invalid_policy_ready, key=sort)[:limit]
        ],
        "unscoped_leaves": [
            row(issue) for issue in sorted(unscoped_leaves, key=sort)[:limit]
        ],
        "ready_containers": [row(issue) for issue in sorted(containers, key=sort)[:limit]],
        "assigned_ready": [row(issue) for issue in sorted(assigned, key=sort)[:limit]],
    }


def _bounded_output(text: str) -> str:
    size = len(text.encode("utf-8"))
    if size > MAX_PLAN_OUTPUT_BYTES:
        raise PlanError(
            f"claim plan exceeds the {MAX_PLAN_OUTPUT_BYTES}-byte output limit ({size} bytes)"
        )
    return text


def render_json(plan: dict[str, Any]) -> str:
    return _bounded_output(json.dumps(plan, sort_keys=True, separators=(",", ":")) + "\n")


def render_markdown(plan: dict[str, Any]) -> str:
    recommendation = plan["recommendation"]
    issue = recommendation["issue"]
    selected = (
        "none"
        if issue is None
        else f"P{issue['priority']} `{issue['id']}` [{issue['workstream']}]"
    )
    lines = [
        "# FrankenManim claim plan",
        "",
        f"As of `{plan['as_of']}`; Beads remains authoritative.",
        "",
        f"- Graph integrity: **{'valid' if plan['integrity']['ok'] else 'INVALID'}**.",
        (
            f"- Active workstreams: **{plan['activation']['count']}/"
            f"{plan['activation']['cap']}**; "
            + (
                ", ".join(
                    f"`{item}`" for item in plan["activation"]["active_workstreams"]
                )
                or "none"
            )
            + "."
        ),
        f"- Recommended next: **{selected}** — {recommendation['reason']}.",
        (
            f"- Queues: {plan['counts']['claimable_leaves']} autonomous leaves, "
            f"{plan['counts']['non_autonomous_ready']} manual/external leaves, "
            f"{plan['counts']['unscoped_leaves']} unscoped leaves, "
            f"{plan['counts']['ready_containers']} ready containers, "
            f"{plan['counts']['assigned_ready']} assigned-ready, "
            f"{plan['counts']['containment_cycles']} containment cycles."
        ),
        "",
        (
            "A task with any live `parent-child` descendant is a container even "
            "when its issue type is not `epic`."
        ),
        "Unscoped issues are diagnostic state only and are never autonomous claim candidates.",
        (
            "Exact `agent:claim:manual` and `agent:claim:external` labels keep "
            "ready work visible while excluding it from autonomous claims."
        ),
        "",
    ]
    for component in plan["integrity"]["containment_cycles"]:
        lines.append(
            "- Parent-child cycle: "
            + " → ".join(f"`{issue_id}`" for issue_id in component)
        )
    for issue_id in plan["integrity"]["unscoped_active"]:
        lines.append(f"- Unscoped active claim: `{issue_id}`")
    for violation in plan["integrity"]["claim_policy_violations"]:
        labels = ", ".join(f"`{label}`" for label in violation["labels"]) or "none"
        lines.append(
            f"- Claim-policy violation `{violation['code']}` on "
            f"`{violation['issue_id']}`; labels: {labels}."
        )
    if (
        plan["integrity"]["containment_cycles"]
        or plan["integrity"]["unscoped_active"]
        or plan["integrity"]["claim_policy_violations"]
    ):
        lines.append("")
    return _bounded_output("\n".join(lines))


def parse_as_of(
    value: str | None, issues: dict[str, agent_brief.Issue]
) -> dt.datetime:
    if value is not None:
        return agent_brief.parse_timestamp(
            value,
            field="--as-of",
            issue_id="command line",
        )
    return max(
        (issue.updated_at for issue in issues.values()),
        default=dt.datetime(1970, 1, 1, tzinfo=UTC),
    )


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ledger", type=Path, default=Path(".beads/issues.jsonl"))
    parser.add_argument("--format", choices=("id", "json", "markdown"), default="id")
    parser.add_argument("--as-of")
    parser.add_argument("--stale-days", type=int, default=2)
    parser.add_argument("--activation-cap", type=int, default=4)
    parser.add_argument("--limit", type=int, default=20)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--require", action="store_true")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = make_parser().parse_args(argv)
    if args.stale_days < 0 or args.activation_cap < 1 or not 1 <= args.limit <= 1000:
        print("error: invalid planner bounds", file=sys.stderr)
        return 2
    try:
        issues = agent_brief.load_issues(args.ledger)
        plan = build_plan(
            issues,
            as_of=parse_as_of(args.as_of, issues),
            stale_days=args.stale_days,
            activation_cap=args.activation_cap,
            limit=args.limit,
        )
    except (agent_brief.BriefError, PlanError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    enforce = args.check or args.require
    if enforce and not plan["integrity"]["ok"]:
        print("error: Beads task-graph integrity failed", file=sys.stderr)
        return 1
    if enforce and not plan["activation"]["within_cap"]:
        print("error: active workstream cap breached", file=sys.stderr)
        return 1
    if args.require and plan["recommendation"]["issue"] is None:
        print("error: no claimable recommendation exists", file=sys.stderr)
        return 3

    try:
        if args.format == "json":
            output = render_json(plan)
        elif args.format == "markdown":
            output = render_markdown(plan)
        else:
            issue = plan["recommendation"]["issue"]
            output = (issue["id"] if issue is not None else "none") + "\n"
            output = _bounded_output(output)
    except PlanError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
