#!/usr/bin/env python3
"""Issue a graph-and-policy-bound token for one autonomous Beads claim.

``agent_next`` answers which leaf is the best next claim. This command binds
that answer to the complete parsed task graph, the planner output, every
persisted task-semantic field, and every policy/schema input that can affect the
recommendation. An agent can therefore revalidate the token immediately before
mutating Beads. The token is a compare-before-set guard, not a lease: current
file reservations, assignees, and coordinating messages still need inspection.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

import agent_brief
import agent_next
import agent_task_semantics

SCHEMA = "fmn.agent.claim-guard"
SCHEMA_VERSION = 2
TOKEN_VERSION = "v2"
CLAIM_INPUT_SCHEMA = "fmn.agent.claim-input"
CLAIM_INPUT_VERSION = 3
CLAIM_GRAPH_SCHEMA = "fmn.agent.claim-graph"
CLAIM_GRAPH_VERSION = 3
RESERVED_ISSUE_ID = "none"
MAX_OUTPUT_BYTES = 1024 * 1024
TOKEN_RE = re.compile(r"^v2:(?P<digest>[0-9a-f]{64}):(?P<issue>[^:\s]+)$")


class GuardError(ValueError):
    pass


agent_task_semantics.install_loader()


def canonical_json(value: Any) -> bytes:
    try:
        rendered = json.dumps(
            value,
            ensure_ascii=False,
            allow_nan=False,
            sort_keys=True,
            separators=(",", ":"),
        )
    except (TypeError, ValueError, RecursionError) as exc:
        raise GuardError(f"claim input cannot be canonically encoded: {exc}") from exc
    try:
        return rendered.encode("utf-8")
    except UnicodeEncodeError as exc:
        raise GuardError(f"claim input contains an unpaired Unicode surrogate: {exc}") from exc


def canonical_graph(issues: dict[str, agent_brief.Issue]) -> dict[str, Any]:
    if RESERVED_ISSUE_ID in issues:
        raise GuardError(
            f"issue id {RESERVED_ISSUE_ID!r} is reserved for the no-recommendation token sentinel"
        )
    if not isinstance(issues, agent_task_semantics.SemanticIssues):
        raise GuardError(
            "claim graph version 3 requires issues loaded through the full task-semantic loader"
        )
    try:
        task_semantics = agent_task_semantics.semantics_for(issues)
    except agent_task_semantics.SemanticError as exc:
        raise GuardError(str(exc)) from exc
    rows: list[dict[str, Any]] = []
    for issue_id in sorted(issues):
        issue = issues[issue_id]
        dependencies = sorted(
            (
                {
                    "issue_id": dependency.issue_id,
                    "depends_on_id": dependency.depends_on_id,
                    "type": dependency.kind,
                }
                for dependency in issue.dependencies
            ),
            key=lambda row: (row["depends_on_id"], row["type"], row["issue_id"]),
        )
        rows.append(
            {
                "id": issue.id,
                "title": issue.title,
                "status": issue.status,
                "priority": issue.priority,
                "issue_type": issue.issue_type,
                "assignee": issue.assignee,
                "updated_at": issue.updated_at.isoformat().replace("+00:00", "Z"),
                "dependencies": dependencies,
                "comments": list(issue.comments),
                "task_semantics": task_semantics[issue_id],
            }
        )
    return {
        "schema": CLAIM_GRAPH_SCHEMA,
        "version": CLAIM_GRAPH_VERSION,
        "issues": rows,
    }


def digest(value: Any) -> str:
    return hashlib.sha256(canonical_json(value)).hexdigest()


def graph_digest(issues: dict[str, agent_brief.Issue]) -> str:
    try:
        agent_task_semantics.verify_remembered_semantics(issues)
    except agent_task_semantics.SemanticError as exc:
        raise GuardError(str(exc)) from exc
    return digest(canonical_graph(issues))


def recommendation_id(plan: dict[str, Any]) -> str | None:
    issue = plan["recommendation"]["issue"]
    return None if issue is None else str(issue["id"])


def schema_contract() -> dict[str, Any]:
    return {
        "agent_brief_snapshot_version": agent_brief.SNAPSHOT_SCHEMA_VERSION,
        "agent_next": {
            "schema": agent_next.SCHEMA,
            "version": agent_next.SCHEMA_VERSION,
        },
        "claim_graph": {
            "schema": CLAIM_GRAPH_SCHEMA,
            "version": CLAIM_GRAPH_VERSION,
        },
        "claim_guard": {
            "schema": SCHEMA,
            "version": SCHEMA_VERSION,
        },
        "claim_input": {
            "schema": CLAIM_INPUT_SCHEMA,
            "version": CLAIM_INPUT_VERSION,
        },
        "token_version": TOKEN_VERSION,
    }


def claim_input(
    graph: dict[str, Any],
    plan: dict[str, Any],
    *,
    stale_days: int,
    activation_cap: int,
    limit: int,
) -> dict[str, Any]:
    return {
        "schema": CLAIM_INPUT_SCHEMA,
        "version": CLAIM_INPUT_VERSION,
        "schemas": schema_contract(),
        "policy": {
            "as_of": plan["as_of"],
            "stale_days": stale_days,
            "activation_cap": activation_cap,
            "limit": limit,
        },
        "graph": graph,
        "plan": plan,
    }


def make_token(claim_sha256: str, issue_id: str | None) -> str:
    if not re.fullmatch(r"[0-9a-f]{64}", claim_sha256):
        raise GuardError("claim digest must be 64 lowercase hexadecimal characters")
    if issue_id == RESERVED_ISSUE_ID:
        raise GuardError(
            f"issue id {RESERVED_ISSUE_ID!r} is reserved for the no-recommendation token sentinel"
        )
    subject = RESERVED_ISSUE_ID if issue_id is None else issue_id
    if not subject or ":" in subject or any(character.isspace() for character in subject):
        raise GuardError(f"recommendation id cannot be represented in a claim token: {subject!r}")
    return f"{TOKEN_VERSION}:{claim_sha256}:{subject}"


def parse_token(value: str) -> tuple[str, str | None]:
    match = TOKEN_RE.fullmatch(value)
    if match is None:
        raise GuardError(
            "--expect-token must have the form v2:<64 lowercase hex characters>:<issue-id|none>"
        )
    issue_id = match.group("issue")
    return match.group("digest"), None if issue_id == RESERVED_ISSUE_ID else issue_id


def build_guard(
    issues: dict[str, agent_brief.Issue],
    *,
    as_of,
    stale_days: int,
    activation_cap: int,
    limit: int,
) -> dict[str, Any]:
    plan = agent_next.build_plan(
        issues,
        as_of=as_of,
        stale_days=stale_days,
        activation_cap=activation_cap,
        limit=limit,
    )
    graph = canonical_graph(issues)
    graph_sha256 = digest(graph)
    input_document = claim_input(
        graph,
        plan,
        stale_days=stale_days,
        activation_cap=activation_cap,
        limit=limit,
    )
    claim_sha256 = digest(input_document)
    selected = recommendation_id(plan)
    token = make_token(claim_sha256, selected)
    try:
        agent_task_semantics.remember_semantics(issues)
    except agent_task_semantics.SemanticError as exc:
        raise GuardError(str(exc)) from exc
    return {
        "schema": SCHEMA,
        "version": SCHEMA_VERSION,
        "as_of": plan["as_of"],
        "graph_sha256": graph_sha256,
        "claim_sha256": claim_sha256,
        "policy": input_document["policy"],
        "schemas": input_document["schemas"],
        "recommendation_id": selected,
        "token": token,
        "integrity": plan["integrity"],
        "activation": plan["activation"],
        "recommendation": plan["recommendation"],
    }


def bounded(text: str) -> str:
    size = len(text.encode("utf-8"))
    if size > MAX_OUTPUT_BYTES:
        raise GuardError(
            f"claim-guard output exceeds the {MAX_OUTPUT_BYTES}-byte limit ({size} bytes)"
        )
    return text


def render_json(guard: dict[str, Any]) -> str:
    return bounded(
        json.dumps(
            guard,
            ensure_ascii=False,
            allow_nan=False,
            sort_keys=True,
            separators=(",", ":"),
        )
        + "\n"
    )


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ledger", type=Path, default=Path(".beads/issues.jsonl"))
    parser.add_argument("--format", choices=("token", "id", "json"), default="token")
    parser.add_argument(
        "--expect-token",
        help="return exit 4 unless the current graph, policy, schemas, and recommendation match",
    )
    parser.add_argument("--as-of")
    parser.add_argument("--stale-days", type=int, default=2)
    parser.add_argument("--activation-cap", type=int, default=4)
    parser.add_argument("--limit", type=int, default=20)
    parser.add_argument(
        "--require",
        action="store_true",
        help="return exit 3 when the valid graph has no claimable recommendation",
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
    if not 1 <= args.limit <= 1000:
        print("error: --limit must be between 1 and 1000", file=sys.stderr)
        return 2

    try:
        issues = agent_brief.load_issues(args.ledger)
        as_of = agent_next.parse_as_of(args.as_of, issues)
        guard = build_guard(
            issues,
            as_of=as_of,
            stale_days=args.stale_days,
            activation_cap=args.activation_cap,
            limit=args.limit,
        )
    except (agent_brief.BriefError, GuardError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    if not guard["integrity"]["ok"]:
        print("error: Beads task-graph integrity failed", file=sys.stderr)
        return 1
    if not guard["activation"]["within_cap"]:
        print("error: active workstream cap breached", file=sys.stderr)
        return 1

    if args.expect_token is not None:
        try:
            expected_digest, expected_issue = parse_token(args.expect_token)
        except GuardError as exc:
            print(f"error: {exc}", file=sys.stderr)
            return 2
        if (
            expected_digest != guard["claim_sha256"]
            or expected_issue != guard["recommendation_id"]
        ):
            print(
                "error: claim token is stale; refresh the plan and re-check reservations before claiming",
                file=sys.stderr,
            )
            return 4

    if args.require and guard["recommendation_id"] is None:
        print("error: no claimable recommendation exists", file=sys.stderr)
        return 3

    try:
        if args.format == "json":
            output = render_json(guard)
        elif args.format == "id":
            output = bounded((guard["recommendation_id"] or RESERVED_ISSUE_ID) + "\n")
        else:
            output = bounded(guard["token"] + "\n")
    except (GuardError, TypeError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
