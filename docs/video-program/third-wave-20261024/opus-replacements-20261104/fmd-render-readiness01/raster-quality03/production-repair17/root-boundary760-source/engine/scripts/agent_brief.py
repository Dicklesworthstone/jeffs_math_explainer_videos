#!/usr/bin/env python3
"""Deterministic, bounded operational brief for the Beads work graph.

This is a read-only projection of .beads/issues.jsonl. It never edits Beads and
never becomes a second source of truth. The output is intentionally compact so
an agent can reconstruct current work, blockers, graph integrity, and
activation pressure. Autonomous claim selection belongs only to agent_next.py.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import stat
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

MAX_LEDGER_BYTES = 32 * 1024 * 1024
MAX_LINE_BYTES = 2 * 1024 * 1024
MAX_ISSUES = 100_000
MAX_DEPENDENCIES_PER_ISSUE = 10_000
MAX_DEPENDENCIES = 500_000
MAX_COMMENTS_PER_ISSUE = 10_000
SNAPSHOT_SCHEMA_VERSION = 5
ACTIVE_STATUS = "in_progress"
KNOWN_STATUSES = frozenset({"open", ACTIVE_STATUS, "closed", "tombstone"})
OPEN_STATUSES = frozenset({"open", ACTIVE_STATUS})
CLOSED_STATUSES = frozenset({"closed", "tombstone"})
NON_CLAIMABLE_TYPES = frozenset({"epic"})
UNSCOPED = "UNSCOPED"
GOVERNED_WORKSTREAM_RE = re.compile(
    r"^(?:(?P<g0>G0)|W(?P<number>[1-9]|1[01]))(?:\b|:)"
)
UTC = dt.timezone.utc


class BriefError(ValueError):
    pass


class _DuplicateJsonKey(ValueError):
    def __init__(self, key: str):
        super().__init__(key)
        self.key = key


class _NonFiniteJsonConstant(ValueError):
    def __init__(self, spelling: str):
        super().__init__(spelling)
        self.spelling = spelling


@dataclass(frozen=True)
class Dependency:
    issue_id: str
    depends_on_id: str
    kind: str


@dataclass(frozen=True)
class Issue:
    id: str
    title: str
    status: str
    priority: int
    issue_type: str
    assignee: str | None
    updated_at: dt.datetime
    dependencies: tuple[Dependency, ...]
    comments: tuple[dict[str, Any], ...]

    @property
    def workstream(self) -> str:
        match = GOVERNED_WORKSTREAM_RE.match(self.title)
        if match is None:
            return UNSCOPED
        if match.group("g0") is not None:
            return "G0"
        return f"W{match.group('number')}"


def governed_workstream(issue: Issue) -> str:
    """Return the exact governance workstream encoded by an issue title."""

    return issue.workstream


def parse_timestamp(value: str, *, field: str, issue_id: str) -> dt.datetime:
    if not isinstance(value, str) or not value:
        raise BriefError(f"{issue_id}: {field} must be a non-empty timestamp")
    normalized = value[:-1] + "+00:00" if value.endswith("Z") else value
    try:
        parsed = dt.datetime.fromisoformat(normalized)
    except ValueError as exc:
        raise BriefError(f"{issue_id}: invalid {field} timestamp {value!r}") from exc
    if parsed.tzinfo is None:
        raise BriefError(f"{issue_id}: {field} must include a timezone")
    return parsed.astimezone(UTC)


def _text(value: Any, *, field: str, issue_id: str, optional: bool = False) -> str | None:
    if optional and value is None:
        return None
    if not isinstance(value, str) or not value:
        raise BriefError(f"{issue_id}: {field} must be a non-empty string")
    return value


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise _DuplicateJsonKey(key)
        result[key] = value
    return result


def _reject_nonfinite_constant(spelling: str) -> None:
    raise _NonFiniteJsonConstant(spelling)


def _optional_array(raw: dict[str, Any], field: str, issue_id: str) -> list[Any]:
    value = raw.get(field, [])
    if value is None:
        return []
    if not isinstance(value, list):
        raise BriefError(f"{issue_id}: {field} must be an array or null")
    return value


def parse_issue(raw: Any, *, line: int) -> Issue:
    if not isinstance(raw, dict):
        raise BriefError(f"line {line}: issue record must be an object")
    issue_id = _text(raw.get("id"), field="id", issue_id=f"line {line}")
    assert issue_id is not None
    title = _text(raw.get("title"), field="title", issue_id=issue_id)
    status = _text(raw.get("status"), field="status", issue_id=issue_id)
    if status not in KNOWN_STATUSES:
        raise BriefError(
            f"{issue_id}: unknown status {status!r}; expected one of "
            + ", ".join(sorted(KNOWN_STATUSES))
        )
    issue_type = _text(raw.get("issue_type"), field="issue_type", issue_id=issue_id)
    priority = raw.get("priority")
    if isinstance(priority, bool) or not isinstance(priority, int) or priority < 0:
        raise BriefError(f"{issue_id}: priority must be a nonnegative integer")
    assignee = _text(raw.get("assignee"), field="assignee", issue_id=issue_id, optional=True)
    if "updated_at" in raw:
        updated_raw = raw["updated_at"]
        updated_field = "updated_at"
    else:
        updated_raw = raw.get("created_at")
        updated_field = "created_at"
    updated_at = parse_timestamp(updated_raw, field=updated_field, issue_id=issue_id)

    dependencies_raw = _optional_array(raw, "dependencies", issue_id)
    if len(dependencies_raw) > MAX_DEPENDENCIES_PER_ISSUE:
        raise BriefError(
            f"{issue_id}: dependencies exceed the {MAX_DEPENDENCIES_PER_ISSUE}-edge limit"
        )
    dependencies: list[Dependency] = []
    seen_dependencies: set[tuple[str, str]] = set()
    for index, item in enumerate(dependencies_raw):
        if not isinstance(item, dict):
            raise BriefError(f"{issue_id}: dependency {index} must be an object")
        owner = _text(item.get("issue_id"), field="dependency.issue_id", issue_id=issue_id)
        target = _text(item.get("depends_on_id"), field="dependency.depends_on_id", issue_id=issue_id)
        kind = _text(item.get("type"), field="dependency.type", issue_id=issue_id)
        if owner != issue_id:
            raise BriefError(
                f"{issue_id}: dependency {index} is owned by {owner!r}, not this issue"
            )
        if target == issue_id:
            raise BriefError(f"{issue_id}: dependency {index} is self-referential")
        edge = (target or "", kind or "")
        if edge in seen_dependencies:
            raise BriefError(
                f"{issue_id}: duplicate dependency on {target!r} with type {kind!r}"
            )
        seen_dependencies.add(edge)
        dependencies.append(Dependency(owner or "", target or "", kind or ""))

    comments_raw = _optional_array(raw, "comments", issue_id)
    if len(comments_raw) > MAX_COMMENTS_PER_ISSUE:
        raise BriefError(
            f"{issue_id}: comments exceed the {MAX_COMMENTS_PER_ISSUE}-comment limit"
        )
    comments: list[dict[str, Any]] = []
    for index, item in enumerate(comments_raw):
        if not isinstance(item, dict):
            raise BriefError(f"{issue_id}: comment {index} must be an object")
        text = item.get("text")
        if text is not None and not isinstance(text, str):
            raise BriefError(f"{issue_id}: comment {index} text must be a string or null")
        comments.append(item)
    return Issue(
        id=issue_id,
        title=title or "",
        status=status or "",
        priority=priority,
        issue_type=issue_type or "",
        assignee=assignee,
        updated_at=updated_at,
        dependencies=tuple(dependencies),
        comments=tuple(comments),
    )


def _open_ledger(path: Path):
    flags = os.O_RDONLY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags)
    except OSError as exc:
        raise BriefError(f"cannot open {path}: {exc}") from exc
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode):
            raise BriefError(f"{path} is not a regular file")
        if metadata.st_size > MAX_LEDGER_BYTES:
            raise BriefError(f"{path} exceeds the {MAX_LEDGER_BYTES}-byte ledger limit")
        return os.fdopen(descriptor, "rb")
    except Exception:
        os.close(descriptor)
        raise


def load_issues(path: Path) -> dict[str, Issue]:
    issues: dict[str, Issue] = {}
    dependency_count = 0
    try:
        with _open_ledger(path) as handle:
            for line_number, raw_line in enumerate(handle, 1):
                if len(raw_line) > MAX_LINE_BYTES:
                    raise BriefError(
                        f"{path}:{line_number} exceeds the {MAX_LINE_BYTES}-byte line limit"
                    )
                if not raw_line.endswith(b"\n"):
                    raise BriefError(f"{path}:{line_number} is missing its final LF")
                if not raw_line.strip():
                    raise BriefError(f"{path}:{line_number} is blank")
                try:
                    record = json.loads(
                        raw_line,
                        object_pairs_hook=_unique_object,
                        parse_constant=_reject_nonfinite_constant,
                    )
                except _DuplicateJsonKey as exc:
                    raise BriefError(
                        f"{path}:{line_number}: duplicate JSON object key {exc.key!r}"
                    ) from exc
                except _NonFiniteJsonConstant as exc:
                    raise BriefError(
                        f"{path}:{line_number}: non-finite JSON constant "
                        f"{exc.spelling!r} is forbidden"
                    ) from exc
                except (UnicodeDecodeError, json.JSONDecodeError) as exc:
                    raise BriefError(f"{path}:{line_number}: invalid UTF-8 JSON: {exc}") from exc
                issue = parse_issue(record, line=line_number)
                if issue.id in issues:
                    raise BriefError(f"{path}:{line_number}: duplicate issue id {issue.id}")
                issues[issue.id] = issue
                dependency_count += len(issue.dependencies)
                if len(issues) > MAX_ISSUES:
                    raise BriefError(f"{path} exceeds the {MAX_ISSUES}-issue limit")
                if dependency_count > MAX_DEPENDENCIES:
                    raise BriefError(
                        f"{path} exceeds the {MAX_DEPENDENCIES}-dependency limit"
                    )
    except BriefError:
        raise
    except OSError as exc:
        raise BriefError(f"cannot read {path}: {exc}") from exc
    return issues


def blocking_dependencies(issue: Issue) -> tuple[str, ...]:
    return tuple(
        dep.depends_on_id
        for dep in issue.dependencies
        if dep.issue_id == issue.id and dep.kind == "blocks"
    )


def unresolved_blockers(issue: Issue, issues: dict[str, Issue]) -> tuple[str, ...]:
    unresolved: list[str] = []
    for blocker_id in blocking_dependencies(issue):
        blocker = issues.get(blocker_id)
        if blocker is None or blocker.status not in CLOSED_STATUSES:
            unresolved.append(blocker_id)
    return tuple(sorted(set(unresolved)))


def missing_dependency_targets(
    issues: dict[str, Issue],
) -> tuple[tuple[str, str, str], ...]:
    return tuple(
        sorted(
            (issue.id, dependency.depends_on_id, dependency.kind)
            for issue in issues.values()
            for dependency in issue.dependencies
            if dependency.depends_on_id not in issues
        )
    )


def blocking_cycles(issues: dict[str, Issue]) -> tuple[tuple[str, ...], ...]:
    """Return deterministic live blocking SCCs without Python recursion."""

    live_ids = {issue.id for issue in issues.values() if issue.status in OPEN_STATUSES}
    adjacency = {
        issue_id: tuple(
            sorted(
                target
                for target in blocking_dependencies(issues[issue_id])
                if target in live_ids
            )
        )
        for issue_id in live_ids
    }
    reverse_lists = {issue_id: [] for issue_id in live_ids}
    for issue_id, targets in adjacency.items():
        for target in targets:
            reverse_lists[target].append(issue_id)
    reverse = {
        issue_id: tuple(sorted(targets))
        for issue_id, targets in reverse_lists.items()
    }

    visited: set[str] = set()
    finish_order: list[str] = []
    for root in sorted(live_ids):
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


def issue_sort_key(issue: Issue) -> tuple[int, int, float, str]:
    status_rank = 0 if issue.status == ACTIVE_STATUS else 1
    return (status_rank, issue.priority, -issue.updated_at.timestamp(), issue.id)


def situational_priority_sort_key(issue: Issue) -> tuple[int, int, float, str]:
    scope_rank = 1 if issue.workstream == UNSCOPED else 0
    return (issue.priority, scope_rank, -issue.updated_at.timestamp(), issue.id)


def compact_title(title: str, limit: int = 100) -> str:
    normalized = " ".join(title.split())
    return normalized if len(normalized) <= limit else normalized[: limit - 1] + "…"


def latest_comment(issue: Issue) -> str | None:
    for item in reversed(issue.comments):
        text = item.get("text")
        if isinstance(text, str) and text.strip():
            return compact_title(text, 140)
    return None


def select_situational_priority(
    ready: list[Issue], active_workstreams: set[str], activation_cap: int
) -> tuple[Issue | None, str]:
    in_active = [issue for issue in ready if issue.workstream in active_workstreams]
    if in_active:
        issue = min(in_active, key=situational_priority_sort_key)
        return issue, f"highest-priority broad-ready candidate in already-active {issue.workstream}"
    if not ready:
        return None, "no dependency-ready, unassigned non-epic candidates"
    if len(active_workstreams) >= activation_cap:
        return None, "activation cap is full and no broad-ready candidate belongs to an active workstream"
    issue = min(ready, key=situational_priority_sort_key)
    if issue.workstream == UNSCOPED:
        return issue, "highest-priority broad-ready candidate is unscoped"
    return issue, (
        "highest-priority broad-ready candidate; autonomous selection still requires "
        "scripts/agent_next.py"
    )


def build_snapshot(
    issues: dict[str, Issue],
    *,
    as_of: dt.datetime,
    stale_days: int,
    activation_cap: int,
    limit: int,
) -> dict[str, Any]:
    if as_of.tzinfo is None:
        raise BriefError("as_of must include a timezone")
    as_of = as_of.astimezone(UTC)
    active = sorted(
        (issue for issue in issues.values() if issue.status == ACTIVE_STATUS),
        key=issue_sort_key,
    )
    open_issues = sorted(
        (issue for issue in issues.values() if issue.status in OPEN_STATUSES),
        key=issue_sort_key,
    )
    active_workstream_set = {
        issue.workstream for issue in active if issue.workstream != UNSCOPED
    }
    active_workstreams = sorted(active_workstream_set)
    dependency_ready = [
        issue
        for issue in open_issues
        if issue.status == "open" and not unresolved_blockers(issue, issues)
    ]
    assigned_ready = [issue for issue in dependency_ready if issue.assignee is not None]
    container_ready = [
        issue
        for issue in dependency_ready
        if issue.assignee is None and issue.issue_type in NON_CLAIMABLE_TYPES
    ]
    ready = [
        issue
        for issue in dependency_ready
        if issue.assignee is None and issue.issue_type not in NON_CLAIMABLE_TYPES
    ]
    blocked = [
        (issue, unresolved_blockers(issue, issues))
        for issue in open_issues
        if unresolved_blockers(issue, issues)
    ]
    stale_before = as_of - dt.timedelta(days=stale_days)
    stale = [
        issue
        for issue in active
        if issue.assignee is not None and issue.updated_at < stale_before
    ]
    unowned = [issue for issue in active if issue.assignee is None]
    unscoped = [issue for issue in active if issue.workstream == UNSCOPED]
    missing_targets = missing_dependency_targets(issues)
    missing_blockers = tuple(
        (owner, target) for owner, target, kind in missing_targets if kind == "blocks"
    )
    cycles = blocking_cycles(issues)
    integrity_ok = not missing_blockers and not cycles
    if integrity_ok:
        priority_issue, priority_reason = select_situational_priority(
            ready, active_workstream_set, activation_cap
        )
    else:
        priority_issue = None
        priority_reason = "ledger integrity failures suppress situational prioritization"

    def row(issue: Issue) -> dict[str, Any]:
        blockers = unresolved_blockers(issue, issues)
        return {
            "id": issue.id,
            "priority": issue.priority,
            "status": issue.status,
            "issue_type": issue.issue_type,
            "workstream": issue.workstream,
            "assignee": issue.assignee,
            "updated_at": issue.updated_at.isoformat().replace("+00:00", "Z"),
            "title": compact_title(issue.title),
            "blockers": list(blockers),
            "latest_comment": latest_comment(issue),
        }

    priority_row = row(priority_issue) if priority_issue is not None else None
    activates_workstream = bool(
        priority_issue is not None
        and priority_issue.workstream != UNSCOPED
        and priority_issue.workstream not in active_workstream_set
    )
    return {
        "schema_version": SNAPSHOT_SCHEMA_VERSION,
        "as_of": as_of.isoformat().replace("+00:00", "Z"),
        "integrity": {
            "within_contract": integrity_ok,
            "blocking_cycles": [list(component) for component in cycles],
            "missing_blockers": [
                {"issue_id": owner, "depends_on_id": target}
                for owner, target in missing_blockers
            ],
            "missing_links": [
                {"issue_id": owner, "depends_on_id": target, "type": kind}
                for owner, target, kind in missing_targets
                if kind != "blocks"
            ],
        },
        "activation": {
            "cap": activation_cap,
            "active_workstreams": active_workstreams,
            "count": len(active_workstreams),
            "within_cap": len(active_workstreams) <= activation_cap,
        },
        "counts": {
            "total": len(issues),
            "open": sum(issue.status == "open" for issue in issues.values()),
            "in_progress": len(active),
            "closed": sum(issue.status in CLOSED_STATUSES for issue in issues.values()),
            "dependency_ready": len(dependency_ready),
            "ready": len(ready),
            "assigned_ready": len(assigned_ready),
            "container_ready": len(container_ready),
            "blocked": len(blocked),
            "blocking_cycles": len(cycles),
            "missing_blockers": len(missing_blockers),
            "missing_links": len(missing_targets) - len(missing_blockers),
            "stale_claims": len(stale),
            "unowned_active": len(unowned),
            "unscoped_active": len(unscoped),
        },
        "situational_priority": {
            "issue": priority_row,
            "reason": priority_reason,
            "activates_workstream": activates_workstream,
            "claim_safe": False,
            "claim_surface": "scripts/agent_next.py",
        },
        "active": [row(issue) for issue in active[:limit]],
        "ready": [row(issue) for issue in ready[:limit]],
        "assigned_ready": [row(issue) for issue in assigned_ready[:limit]],
        "container_ready": [row(issue) for issue in container_ready[:limit]],
        "blocked": [
            {**row(issue), "blockers": list(blockers)}
            for issue, blockers in blocked[:limit]
        ],
        "stale_claims": [row(issue) for issue in stale[:limit]],
        "unowned_active": [row(issue) for issue in unowned[:limit]],
        "unscoped_active": [row(issue) for issue in unscoped[:limit]],
    }


def render_markdown(snapshot: dict[str, Any]) -> str:
    integrity = snapshot["integrity"]
    activation = snapshot["activation"]
    counts = snapshot["counts"]
    situational = snapshot["situational_priority"]
    integrity_label = "clean" if integrity["within_contract"] else "FAILED"
    lines = [
        "# FrankenManim agent brief",
        "",
        f"As of `{snapshot['as_of']}`. Read-only projection of `.beads/issues.jsonl`.",
        "",
        "## Control plane",
        "",
        (
            f"- Ledger integrity: **{integrity_label}**; "
            f"**{counts['blocking_cycles']}** blocking cycles, "
            f"**{counts['missing_blockers']}** missing blockers, "
            f"**{counts['missing_links']}** missing non-blocking links."
        ),
        (
            f"- Workstreams: **{activation['count']}/{activation['cap']}** active "
            f"({'within cap' if activation['within_cap'] else 'CAP BREACHED'}): "
            + (", ".join(f"`{item}`" for item in activation["active_workstreams"]) or "none")
        ),
        (
            f"- Issues: **{counts['in_progress']}** in progress, **{counts['open']}** open, "
            f"**{counts['ready']}** broad-ready, **{counts['assigned_ready']}** assigned-ready, "
            f"**{counts['container_ready']}** ready epic containers, **{counts['blocked']}** blocked, "
            f"**{counts['stale_claims']}** stale, **{counts['unowned_active']}** unowned active."
        ),
    ]
    priority_issue = situational["issue"]
    if priority_issue is None:
        lines.append(f"- Broad priority: **none** — {situational['reason']}.")
    else:
        activation_note = "; would activate a workstream" if situational["activates_workstream"] else ""
        lines.append(
            f"- Broad priority: **P{priority_issue['priority']} `{priority_issue['id']}`** "
            f"[{priority_issue['workstream']}] — {situational['reason']}{activation_note}."
        )
    lines.append(
        "- Claim contract: **none in this projection**; use `python3 scripts/agent_next.py`."
    )

    def section(title: str, rows: Iterable[dict[str, Any]], empty: str) -> None:
        lines.extend(["", f"## {title}", ""])
        materialized = list(rows)
        if not materialized:
            lines.append(empty)
            return
        for row in materialized:
            owner = f" @{row['assignee']}" if row["assignee"] else ""
            blockers = (
                " blocked by " + ", ".join(f"`{item}`" for item in row["blockers"])
                if row["blockers"]
                else ""
            )
            lines.append(
                f"- **P{row['priority']} `{row['id']}`** [{row['workstream']}]"
                f" ({row['issue_type']}){owner}{blockers}: {row['title']}"
            )

    lines.extend(["", "## Integrity failures", ""])
    if integrity["within_contract"] and not integrity["missing_links"]:
        lines.append("No dependency-integrity failures or orphan links.")
    else:
        for component in integrity["blocking_cycles"]:
            lines.append(
                "- Blocking cycle: " + " → ".join(f"`{issue_id}`" for issue_id in component)
            )
        for edge in integrity["missing_blockers"]:
            lines.append(
                f"- Missing blocker: `{edge['issue_id']}` depends on absent "
                f"`{edge['depends_on_id']}`."
            )
        for edge in integrity["missing_links"]:
            lines.append(
                f"- Orphan {edge['type']} link: `{edge['issue_id']}` references absent "
                f"`{edge['depends_on_id']}`."
            )

    section("Active claims", snapshot["active"], "No active claims.")
    section(
        "Broad dependency-ready queue",
        snapshot["ready"],
        "No broad dependency-ready open issues.",
    )
    section(
        "Assigned dependency-ready work",
        snapshot["assigned_ready"],
        "No assigned dependency-ready work.",
    )
    section(
        "Dependency-ready epic containers",
        snapshot["container_ready"],
        "No dependency-ready epic containers.",
    )
    section("Blocked queue", snapshot["blocked"], "No blocked open issues.")
    section("Stale claims", snapshot["stale_claims"], "No stale assigned claims.")
    section("Unowned active claims", snapshot["unowned_active"], "No unowned active claims.")
    section("Unscoped active claims", snapshot["unscoped_active"], "No unscoped active claims.")
    lines.extend(
        [
            "",
            "## Agent protocol",
            "",
            "1. Refresh this brief immediately before claiming work.",
            "2. Treat Beads as authoritative; this projection never edits status.",
            "3. Repair blocking cycles or missing blocker targets before any new claim.",
            "4. Never claim directly from this projection; use `scripts/agent_next.py`.",
            "5. Recheck reservations and current HEAD after receiving a leaf-safe recommendation.",
            "6. Do not activate a fifth workstream; an unscoped recommendation needs a governance check.",
            "7. Re-run after every claim, dependency change, close, or handoff.",
            "",
        ]
    )
    return "\n".join(lines)


def parse_as_of(value: str | None) -> dt.datetime:
    if value is None:
        return dt.datetime.now(UTC)
    return parse_timestamp(value, field="--as-of", issue_id="command line")


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ledger", type=Path, default=Path(".beads/issues.jsonl"))
    parser.add_argument(
        "--format",
        choices=("markdown", "json", "next"),
        default="markdown",
        help="render situational Markdown/JSON; legacy 'next' is a fail-closed refusal",
    )
    parser.add_argument("--as-of", help="ISO-8601 timestamp; defaults to the current UTC time")
    parser.add_argument("--stale-days", type=int, default=2)
    parser.add_argument("--activation-cap", type=int, default=4)
    parser.add_argument("--limit", type=int, default=20)
    parser.add_argument(
        "--check",
        action="store_true",
        help="exit nonzero without stdout when governance or dependency integrity is invalid",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    args = make_parser().parse_args(argv)
    if args.stale_days < 0:
        print("error: --stale-days must be nonnegative", file=sys.stderr)
        return 2
    if args.activation_cap < 1:
        print("error: --activation-cap must be positive", file=sys.stderr)
        return 2
    if args.limit < 1 or args.limit > 1000:
        print("error: --limit must be between 1 and 1000", file=sys.stderr)
        return 2
    if args.format == "next":
        print(
            "error: agent_brief --format next was removed because the broad projection "
            "is not leaf-safe; use `python3 scripts/agent_next.py`",
            file=sys.stderr,
        )
        return 2
    try:
        issues = load_issues(args.ledger)
        snapshot = build_snapshot(
            issues,
            as_of=parse_as_of(args.as_of),
            stale_days=args.stale_days,
            activation_cap=args.activation_cap,
            limit=args.limit,
        )
    except BriefError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    if args.check:
        failed = False
        if not snapshot["integrity"]["within_contract"]:
            print("error: Beads dependency integrity is invalid", file=sys.stderr)
            failed = True
        if not snapshot["activation"]["within_cap"]:
            print(
                f"error: active workstream cap breached "
                f"({snapshot['activation']['count']}/{snapshot['activation']['cap']})",
                file=sys.stderr,
            )
            failed = True
        if failed:
            return 1
    if args.format == "json":
        print(json.dumps(snapshot, indent=2, sort_keys=True))
    else:
        print(render_markdown(snapshot), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
