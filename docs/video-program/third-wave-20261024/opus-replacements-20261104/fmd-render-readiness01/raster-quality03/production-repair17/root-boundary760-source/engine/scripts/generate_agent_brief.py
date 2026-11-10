#!/usr/bin/env python3
"""Generate or verify the deterministic committed FrankenManim agent brief.

The Beads JSONL ledger remains authoritative. This command derives
``docs/AGENT_BRIEF.md`` from that ledger using the newest issue timestamp as
``as_of``; wall-clock time never enters the artifact, so identical ledger bytes
produce identical Markdown. Broad situational context and the leaf planner are
computed from the same parsed graph and governed-scope classifier, but only the
leaf section is a claim contract.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import stat
import sys
from pathlib import Path
from typing import Any

import agent_brief
import agent_next

DEFAULT_LEDGER = Path(".beads/issues.jsonl")
DEFAULT_OUTPUT = Path("docs/AGENT_BRIEF.md")
MAX_OUTPUT_BYTES = 4 * 1024 * 1024


class GenerateError(ValueError):
    pass


def ledger_as_of(issues: dict[str, agent_brief.Issue]):
    if not issues:
        raise GenerateError("the Beads ledger contains no issues")
    return max(issue.updated_at for issue in issues.values())


def validate_snapshot_scope(snapshot: dict[str, Any], plan: dict[str, Any]) -> None:
    """Fail closed if broad and claim projections disagree on governed scope."""

    if snapshot["activation"] != plan["activation"]:
        raise GenerateError("broad snapshot and claim planner disagree on activation scope")
    broad_unscoped = sorted(row["id"] for row in snapshot["unscoped_active"])
    plan_unscoped = sorted(plan["integrity"]["unscoped_active"])
    if broad_unscoped != plan_unscoped:
        raise GenerateError("broad snapshot and claim planner disagree on unscoped active claims")

    situational = snapshot["situational_priority"]
    broad_issue = situational["issue"]
    if broad_issue is not None and broad_issue["workstream"] == agent_brief.UNSCOPED:
        situational.update(
            {
                "issue": None,
                "reason": (
                    "the highest-priority broad-ready candidate is unscoped; "
                    "scope it before prioritization"
                ),
                "activates_workstream": False,
            }
        )


def validate_plan_for_publication(plan: dict[str, Any]) -> None:
    integrity = plan["integrity"]
    if not integrity["ok"]:
        raise GenerateError(
            "Beads task-graph integrity is invalid "
            f"({len(integrity['blocking_cycles'])} blocking cycles, "
            f"{len(integrity['containment_cycles'])} containment cycles, "
            f"{len(integrity['missing_blockers'])} missing blockers, "
            f"{len(integrity['unscoped_active'])} unscoped active claims)"
        )
    activation = plan["activation"]
    if not activation["within_cap"]:
        raise GenerateError(
            "active workstream cap breached "
            f"({activation['count']}/{activation['cap']})"
        )


def render_leaf_section(plan: dict[str, Any]) -> str:
    recommendation = plan["recommendation"]
    issue = recommendation["issue"]
    if issue is None:
        selected = "**none**"
    else:
        selected = (
            f"**P{issue['priority']} `{issue['id']}`** "
            f"[{issue['workstream']}]"
        )
    lines = [
        "## Leaf-safe claim plan",
        "",
        f"- Recommended next: {selected} — {recommendation['reason']}.",
        (
            f"- Queues: **{plan['counts']['claimable_leaves']}** scoped claimable leaves, "
            f"**{plan['counts']['unscoped_leaves']}** unscoped leaves, "
            f"**{plan['counts']['ready_containers']}** ready containers "
            f"(**{plan['counts']['non_epic_containers']}** non-epic), "
            f"**{plan['counts']['assigned_ready']}** assigned-ready."
        ),
        "- A task with any live `parent-child` descendant is a container, regardless of its issue type.",
        "- Unscoped work is visible for repair but is never an autonomous claim candidate.",
        "- This section is the claim contract; the broader queue below is situational context only.",
        "",
    ]
    return "\n".join(lines)


def build_document(
    ledger: Path,
    *,
    stale_days: int,
    activation_cap: int,
    limit: int,
) -> tuple[str, dict[str, Any]]:
    issues = agent_brief.load_issues(ledger)
    as_of = ledger_as_of(issues)
    snapshot = agent_brief.build_snapshot(
        issues,
        as_of=as_of,
        stale_days=stale_days,
        activation_cap=activation_cap,
        limit=limit,
    )
    plan = agent_next.build_plan(
        issues,
        as_of=as_of,
        stale_days=stale_days,
        activation_cap=activation_cap,
        limit=limit,
    )
    validate_plan_for_publication(plan)
    validate_snapshot_scope(snapshot, plan)
    snapshot["claim_plan"] = plan
    broad = agent_brief.render_markdown(snapshot).rstrip("\n")
    marker = "\n## Active claims\n"
    if marker not in broad:
        raise GenerateError("agent brief renderer omitted the Active claims section")
    document = broad.replace(
        marker,
        "\n" + render_leaf_section(plan) + "## Active claims\n",
        1,
    ).rstrip("\n") + "\n"
    encoded = document.encode("utf-8")
    if len(encoded) > MAX_OUTPUT_BYTES:
        raise GenerateError(
            f"generated agent brief exceeds the {MAX_OUTPUT_BYTES}-byte output limit"
        )
    return document, snapshot


def digest(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


FileIdentity = tuple[int, int, int, int, int]


def _file_identity(metadata: os.stat_result) -> FileIdentity:
    return (
        metadata.st_dev,
        metadata.st_ino,
        metadata.st_size,
        metadata.st_mtime_ns,
        metadata.st_ctime_ns,
    )


def _open_existing_regular(path: Path):
    try:
        before = os.lstat(path)
    except FileNotFoundError as exc:
        raise GenerateError(f"generated agent brief is missing: {path}") from exc
    except OSError as exc:
        raise GenerateError(f"cannot inspect generated agent brief {path}: {exc}") from exc
    if stat.S_ISLNK(before.st_mode):
        raise GenerateError(f"refusing symlink output path {path}")
    if not stat.S_ISREG(before.st_mode):
        raise GenerateError(f"generated agent brief is not a regular file: {path}")

    flags = os.O_RDONLY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags)
    except OSError as exc:
        raise GenerateError(f"cannot open generated agent brief {path}: {exc}") from exc
    try:
        opened = os.fstat(descriptor)
        after = os.lstat(path)
        identity = _file_identity(opened)
        if not stat.S_ISREG(opened.st_mode):
            raise GenerateError(f"generated agent brief is not a regular file: {path}")
        if _file_identity(before) != identity or _file_identity(after) != identity:
            raise GenerateError(f"generated agent brief changed while opening: {path}")
        if opened.st_size > MAX_OUTPUT_BYTES:
            raise GenerateError(
                f"{path} exceeds the {MAX_OUTPUT_BYTES}-byte generated-output limit"
            )
        return os.fdopen(descriptor, "r", encoding="utf-8", newline="")
    except Exception:
        os.close(descriptor)
        raise


def read_existing(path: Path) -> str:
    try:
        with _open_existing_regular(path) as handle:
            return handle.read()
    except GenerateError:
        raise
    except (OSError, UnicodeDecodeError) as exc:
        raise GenerateError(f"cannot read generated agent brief {path}: {exc}") from exc


def _remove_owned_temporary(path: Path, identity: FileIdentity) -> str | None:
    try:
        current = os.lstat(path)
    except FileNotFoundError:
        return None
    except OSError as exc:
        return f"cannot inspect temporary output {path}: {exc}"
    if not stat.S_ISREG(current.st_mode) or _file_identity(current) != identity:
        return f"temporary output identity changed before cleanup: {path}"
    try:
        os.unlink(path)
    except OSError as exc:
        return f"cannot remove temporary output {path}: {exc}"
    return None


def write_atomic(path: Path, document: str) -> None:
    if path.is_symlink():
        raise GenerateError(f"refusing symlink output path {path}")
    if path.exists() and not path.is_file():
        raise GenerateError(f"generated agent brief path is not a regular file: {path}")
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
    except OSError as exc:
        raise GenerateError(f"cannot create output directory {path.parent}: {exc}") from exc

    temporary = path.with_name(f".{path.name}.tmp")
    if temporary.is_symlink():
        raise GenerateError(f"refusing symlink temporary path {temporary}")
    if temporary.exists():
        raise GenerateError(
            f"refusing pre-existing temporary path {temporary}; "
            "inspect it before retrying"
        )
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    descriptor: int | None = None
    identity: FileIdentity | None = None
    try:
        descriptor = os.open(temporary, flags, 0o644)
        identity = _file_identity(os.fstat(descriptor))
        with os.fdopen(descriptor, "w", encoding="utf-8", newline="\n") as handle:
            descriptor = None
            try:
                handle.write(document)
                handle.flush()
                os.fsync(handle.fileno())
            finally:
                try:
                    identity = _file_identity(os.fstat(handle.fileno()))
                except OSError:
                    pass
        os.replace(temporary, path)
    except OSError as exc:
        if descriptor is not None:
            try:
                os.close(descriptor)
            except OSError:
                pass
        cleanup = _remove_owned_temporary(temporary, identity) if identity is not None else None
        detail = f"cannot publish generated agent brief {path}: {exc}"
        if cleanup is not None:
            detail += f"; {cleanup}"
        raise GenerateError(detail) from exc


def check_current(path: Path, expected: str) -> None:
    observed = read_existing(path)
    if observed != expected:
        raise GenerateError(
            f"generated agent brief is stale: {path}; "
            f"expected_sha256={digest(expected)} observed_sha256={digest(observed)}; "
            "run `python3 scripts/generate_agent_brief.py`"
        )


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ledger", type=Path, default=DEFAULT_LEDGER)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--stale-days", type=int, default=2)
    parser.add_argument("--activation-cap", type=int, default=4)
    parser.add_argument("--limit", type=int, default=20)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument(
        "--check",
        action="store_true",
        help="verify committed bytes and governance state without writing",
    )
    mode.add_argument(
        "--stdout",
        action="store_true",
        help="render exact Markdown to stdout without reading or writing the output path",
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
    try:
        document, snapshot = build_document(
            args.ledger,
            stale_days=args.stale_days,
            activation_cap=args.activation_cap,
            limit=args.limit,
        )
    except (agent_brief.BriefError, GenerateError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    try:
        if args.stdout:
            sys.stdout.write(document)
            return 0
        if args.check:
            check_current(args.output, document)
        else:
            write_atomic(args.output, document)
    except GenerateError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1

    action = "current" if args.check else "generated"
    print(
        f"agent brief {action}: {args.output} "
        f"sha256={digest(document)} as_of={snapshot['as_of']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
