#!/usr/bin/env python3
"""Classify the exported Beads state after an unverified claim attempt.

Exit 5 from ``agent_claim.py`` means no verified receipt, not necessarily no
mutation. This read-only command authenticates a pre-claim ledger against the
old guard token, compares it with the current exported ledger, and distinguishes
an unchanged graph, the exact intended claim delta, a conflicting assignee, or
additional ambiguous drift. It never authorizes replay of the old token.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any, Sequence

import agent_brief
import agent_claim
import agent_claim_guard
import agent_next
import agent_task_semantics

SCHEMA = "fmn.agent.claim-reconcile"
SCHEMA_VERSION = 1
MAX_OUTPUT_BYTES = 1024 * 1024
MAX_BASELINE_REF_BYTES = 512
MAX_GIT_PROGRAM_BYTES = 4096
GIT_TIMEOUT_SECONDS = 30.0
GIT_DIAGNOSTIC_BYTES = 64 * 1024
LEDGER_REPOSITORY_PATH = ".beads/issues.jsonl"
CLASS_UNCHANGED = "unchanged"
CLASS_EXACT = "exact-claim-committed"
CLASS_CONFLICT = "conflicting-claim"
CLASS_AMBIGUOUS = "ambiguous-drift"


class ReconcileError(ValueError):
    def __init__(self, message: str, exit_code: int = 2):
        super().__init__(message)
        self.exit_code = exit_code


def _bounded_text(
    label: str,
    value: str | None,
    limit: int,
    *,
    required: bool,
) -> str | None:
    if value is None:
        if required:
            raise ReconcileError(f"{label} is required")
        return None
    if not isinstance(value, str):
        raise ReconcileError(f"{label} must be text")
    try:
        size = len(value.encode("utf-8"))
    except UnicodeEncodeError as exc:
        raise ReconcileError(f"{label} contains an unpaired Unicode surrogate") from exc
    if size > limit:
        raise ReconcileError(f"{label} exceeds the {limit}-byte limit")
    if required and not value:
        raise ReconcileError(f"{label} must not be empty")
    if "\x00" in value:
        raise ReconcileError(f"{label} must not contain NUL")
    return value


def _policy_values(
    *,
    stale_days: int,
    activation_cap: int,
    limit: int,
) -> None:
    if isinstance(stale_days, bool) or stale_days < 0:
        raise ReconcileError("--stale-days must be a nonnegative integer")
    if isinstance(activation_cap, bool) or activation_cap < 1:
        raise ReconcileError("--activation-cap must be a positive integer")
    if isinstance(limit, bool) or not 1 <= limit <= 1000:
        raise ReconcileError("--limit must be between 1 and 1000")


def _git_environment() -> dict[str, str]:
    env = os.environ.copy()
    env.update(
        {
            "GIT_OPTIONAL_LOCKS": "0",
            "LC_ALL": "C",
            "LANG": "C",
        }
    )
    return env


def _git_capture(
    git_program: str,
    repo_root: Path,
    arguments: Sequence[str],
    label: str,
) -> bytes:
    try:
        result = subprocess.run(
            [git_program, "-C", str(repo_root), *arguments],
            check=False,
            capture_output=True,
            timeout=GIT_TIMEOUT_SECONDS,
            env=_git_environment(),
        )
    except (OSError, subprocess.SubprocessError) as exc:
        raise ReconcileError(f"{label} failed: {exc}", 5) from exc
    if len(result.stdout) > GIT_DIAGNOSTIC_BYTES or len(result.stderr) > GIT_DIAGNOSTIC_BYTES:
        raise ReconcileError(f"{label} exceeded the diagnostic output limit", 5)
    if result.returncode != 0:
        detail = (result.stderr or result.stdout).decode("utf-8", errors="replace").strip()
        raise ReconcileError(
            f"{label} exited {result.returncode}: {detail[:4096]}",
            5,
        )
    return result.stdout


def _materialize_baseline(
    repo_root: Path,
    *,
    git_program: str,
    baseline_ref: str,
    destination: Path,
) -> None:
    if baseline_ref.startswith("-") or any(character.isspace() for character in baseline_ref):
        raise ReconcileError("--baseline-ref must be one bounded Git object spelling")
    object_name = f"{baseline_ref}:{LEDGER_REPOSITORY_PATH}"
    size_text = _git_capture(
        git_program,
        repo_root,
        ("cat-file", "-s", object_name),
        "baseline size lookup",
    )
    try:
        size = int(size_text.strip())
    except ValueError as exc:
        raise ReconcileError("baseline size lookup returned a non-integer", 5) from exc
    if size < 0 or size > agent_brief.MAX_LEDGER_BYTES:
        raise ReconcileError(
            f"baseline ledger size {size} exceeds the {agent_brief.MAX_LEDGER_BYTES}-byte limit",
            5,
        )
    try:
        with destination.open("wb") as stdout, tempfile.TemporaryFile() as stderr:
            result = subprocess.run(
                [
                    git_program,
                    "-C",
                    str(repo_root),
                    "cat-file",
                    "blob",
                    object_name,
                ],
                check=False,
                stdout=stdout,
                stderr=stderr,
                timeout=GIT_TIMEOUT_SECONDS,
                env=_git_environment(),
            )
            stderr.seek(0, os.SEEK_END)
            stderr_size = stderr.tell()
            if stderr_size > GIT_DIAGNOSTIC_BYTES:
                raise ReconcileError(
                    "baseline materialization exceeded the diagnostic output limit",
                    5,
                )
            stderr.seek(0)
            detail = stderr.read(GIT_DIAGNOSTIC_BYTES).decode(
                "utf-8", errors="replace"
            )
    except ReconcileError:
        raise
    except (OSError, subprocess.SubprocessError) as exc:
        raise ReconcileError(f"baseline materialization failed: {exc}", 5) from exc
    if result.returncode != 0:
        raise ReconcileError(
            f"baseline materialization exited {result.returncode}: {detail.strip()[:4096]}",
            5,
        )
    actual = destination.stat().st_size
    if actual != size:
        raise ReconcileError(
            f"baseline materialization size mismatch: expected {size}, got {actual}",
            5,
        )


def _load(path: Path, label: str) -> agent_task_semantics.SemanticIssues:
    try:
        issues = agent_brief.load_issues(path)
    except agent_brief.BriefError as exc:
        raise ReconcileError(f"{label} ledger is invalid: {exc}", 5) from exc
    if not isinstance(issues, agent_task_semantics.SemanticIssues):
        raise ReconcileError(f"{label} ledger lacks full task-semantic evidence", 5)
    return issues


def _graph(issues: agent_task_semantics.SemanticIssues) -> dict[str, Any]:
    try:
        return agent_claim_guard.canonical_graph(issues)
    except agent_claim_guard.GuardError as exc:
        raise ReconcileError(str(exc), 5) from exc


def _target_state(
    issues: dict[str, agent_brief.Issue],
    issue_id: str,
) -> dict[str, Any] | None:
    issue = issues.get(issue_id)
    if issue is None:
        return None
    return {
        "status": issue.status,
        "assignee": issue.assignee,
        "updated_at": issue.updated_at.isoformat().replace("+00:00", "Z"),
        "comment_count": len(issue.comments),
    }


def _classify(
    before: agent_task_semantics.SemanticIssues,
    current: agent_task_semantics.SemanticIssues,
    *,
    issue_id: str,
    assignee: str,
    transition_comment: str | None,
) -> tuple[str, str, str, dict[str, object] | None]:
    if _graph(before) == _graph(current):
        return (
            CLASS_UNCHANGED,
            "no-semantic-delta",
            "refresh-coordination-and-issue-new-token",
            None,
        )

    semantic_equal = (
        agent_task_semantics.semantics_for(before)
        == agent_task_semantics.semantics_for(current)
    )
    delta_error: str | None = None
    if semantic_equal:
        try:
            delta = agent_claim._verify_claim_only_delta(
                before,
                current,
                issue_id,
                assignee,
                transition_comment,
            )
        except agent_claim.ClaimError as exc:
            delta_error = str(exc)
        else:
            return (
                CLASS_EXACT,
                "exact-exported-claim-delta",
                "continue-with-claimed-work",
                delta,
            )
    else:
        delta_error = "task-semantic fields differ from the authenticated baseline"

    target = current.get(issue_id)
    if target is not None and target.assignee is not None and target.assignee != assignee:
        return (
            CLASS_CONFLICT,
            "target-assigned-to-different-actor",
            "stop-and-coordinate-with-current-assignee",
            None,
        )
    if (
        target is not None
        and target.status == agent_brief.ACTIVE_STATUS
        and target.assignee == assignee
    ):
        reason = "expected-target-state-with-additional-drift"
    elif target is not None and target.status == "open" and target.assignee is None:
        reason = "baseline-drift-without-target-claim"
    elif target is None:
        reason = "target-missing-from-current-ledger"
    else:
        reason = "target-state-does-not-match-before-or-intended-claim"
    if delta_error:
        reason = f"{reason}: {delta_error[:2048]}"
    return (
        CLASS_AMBIGUOUS,
        reason,
        "inspect-ledger-and-beads-before-any-mutation",
        None,
    )


def reconcile(
    repo: Path,
    expect_token: str,
    assignee: str,
    *,
    transition_comment: str | None = None,
    before_ledger: Path | None = None,
    baseline_ref: str = "HEAD",
    git_program: str = "git",
    as_of: str | None = None,
    stale_days: int = 2,
    activation_cap: int = 4,
    limit: int = 20,
) -> dict[str, Any]:
    expect_token = _bounded_text(
        "--expect-token",
        expect_token,
        agent_claim.MAX_TOKEN_BYTES,
        required=True,
    ) or ""
    assignee = _bounded_text(
        "--assignee",
        assignee,
        agent_claim.MAX_ASSIGNEE_BYTES,
        required=True,
    ) or ""
    transition_comment = _bounded_text(
        "--transition-comment",
        transition_comment,
        agent_claim.MAX_TRANSITION_COMMENT_BYTES,
        required=False,
    )
    baseline_ref = _bounded_text(
        "--baseline-ref",
        baseline_ref,
        MAX_BASELINE_REF_BYTES,
        required=True,
    ) or "HEAD"
    git_program = _bounded_text(
        "--git",
        git_program,
        MAX_GIT_PROGRAM_BYTES,
        required=True,
    ) or "git"
    _policy_values(
        stale_days=stale_days,
        activation_cap=activation_cap,
        limit=limit,
    )
    try:
        _expected_digest, issue_id = agent_claim_guard.parse_token(expect_token)
    except agent_claim_guard.GuardError as exc:
        raise ReconcileError(str(exc)) from exc
    if issue_id is None:
        raise ReconcileError("a no-recommendation token has no claim to reconcile")

    try:
        repo_root = agent_claim._repo_root(repo)
    except agent_claim.ClaimError as exc:
        raise ReconcileError(str(exc), exc.exit_code) from exc
    current_path = repo_root / LEDGER_REPOSITORY_PATH

    with tempfile.TemporaryDirectory(prefix="fmn-claim-reconcile-") as directory:
        if before_ledger is None:
            baseline_path = Path(directory) / "issues.before.jsonl"
            _materialize_baseline(
                repo_root,
                git_program=git_program,
                baseline_ref=baseline_ref,
                destination=baseline_path,
            )
            baseline_source = {
                "kind": "git-ref",
                "value": baseline_ref,
            }
        else:
            baseline_path = before_ledger.resolve()
            baseline_source = {
                "kind": "file",
                "value": str(baseline_path),
            }
        before = _load(baseline_path, "baseline")
        try:
            guard = agent_claim_guard.build_guard(
                before,
                as_of=agent_next.parse_as_of(as_of, before),
                stale_days=stale_days,
                activation_cap=activation_cap,
                limit=limit,
            )
        except (
            agent_brief.BriefError,
            agent_claim_guard.GuardError,
            agent_next.PlanError,
        ) as exc:
            raise ReconcileError(f"baseline guard reconstruction failed: {exc}", 5) from exc
        if guard["token"] != expect_token:
            raise ReconcileError(
                "authenticated baseline does not reproduce the supplied claim token; "
                "select the exact pre-claim ledger and policy",
                4,
            )

        current = _load(current_path, "current")
        classification, reason, next_action, exact_delta = _classify(
            before,
            current,
            issue_id=issue_id,
            assignee=assignee,
            transition_comment=transition_comment,
        )
        baseline_graph = _graph(before)
        current_graph = _graph(current)
        report: dict[str, Any] = {
            "schema": SCHEMA,
            "version": SCHEMA_VERSION,
            "ok": True,
            "classification": classification,
            "reason": reason,
            "next_action": next_action,
            "retry_old_token": False,
            "issue_id": issue_id,
            "expected_assignee": assignee,
            "transition_comment_expected": transition_comment is not None,
            "baseline": {
                "source": baseline_source,
                "claim_sha256": guard["claim_sha256"],
                "graph_sha256": agent_claim_guard.digest(baseline_graph),
                "ledger_sha256": before.ledger_sha256,
                "target": _target_state(before, issue_id),
            },
            "current": {
                "graph_sha256": agent_claim_guard.digest(current_graph),
                "ledger_sha256": current.ledger_sha256,
                "target": _target_state(current, issue_id),
            },
        }
        if exact_delta is not None:
            report["claim_delta"] = exact_delta
        return report


def render_json(report: dict[str, Any]) -> str:
    try:
        text = json.dumps(
            report,
            ensure_ascii=False,
            allow_nan=False,
            sort_keys=True,
            separators=(",", ":"),
        ) + "\n"
    except (TypeError, ValueError, RecursionError) as exc:
        raise ReconcileError(f"reconciliation report cannot be encoded: {exc}", 5) from exc
    size = len(text.encode("utf-8"))
    if size > MAX_OUTPUT_BYTES:
        raise ReconcileError(
            f"reconciliation report exceeds the {MAX_OUTPUT_BYTES}-byte limit",
            5,
        )
    return text


def render_human(report: dict[str, Any]) -> str:
    classification = report["classification"]
    issue_id = report["issue_id"]
    assignee = report["expected_assignee"]
    if classification == CLASS_UNCHANGED:
        detail = "the authenticated graph is semantically unchanged"
    elif classification == CLASS_EXACT:
        detail = f"the exact intended claim is exported for {assignee}"
    elif classification == CLASS_CONFLICT:
        current = report["current"]["target"] or {}
        detail = f"the issue is assigned to {current.get('assignee')!r}"
    else:
        detail = report["reason"]
    return (
        f"claim reconciliation: {classification.upper()}: {issue_id}: {detail}; "
        f"next={report['next_action']}; retry_old_token=false\n"
    )


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path("."))
    parser.add_argument("--expect-token", required=True)
    parser.add_argument("--assignee", default=os.environ.get("FMN_AGENT_ID"))
    parser.add_argument("--transition-comment")
    parser.add_argument("--before-ledger", type=Path)
    parser.add_argument("--baseline-ref", default="HEAD")
    parser.add_argument("--git", default="git")
    parser.add_argument("--as-of")
    parser.add_argument("--stale-days", type=int, default=2)
    parser.add_argument("--activation-cap", type=int, default=4)
    parser.add_argument("--limit", type=int, default=20)
    parser.add_argument("--format", choices=("human", "json"), default="human")
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = make_parser().parse_args(argv)
    try:
        report = reconcile(
            args.repo,
            args.expect_token,
            args.assignee,
            transition_comment=args.transition_comment,
            before_ledger=args.before_ledger,
            baseline_ref=args.baseline_ref,
            git_program=args.git,
            as_of=args.as_of,
            stale_days=args.stale_days,
            activation_cap=args.activation_cap,
            limit=args.limit,
        )
        output = render_json(report) if args.format == "json" else render_human(report)
    except ReconcileError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return exc.exit_code
    sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
