#!/usr/bin/env python3
"""Bind every task-semantic Beads field omitted from ``agent_brief.Issue``.

The broad brief intentionally models only the fields needed for planning. Claim
safety needs a stronger boundary: descriptions, acceptance criteria, labels,
estimates, dependency metadata, and future extension fields must not change
without invalidating the guard. This module reads the exact JSONL authority,
canonicalizes those additional fields, and couples them to the parsed issue map
without turning the projection into a second source of truth.
"""

from __future__ import annotations

import contextvars
import hashlib
import json
import math
import os
import stat
from pathlib import Path
from typing import Any, Callable

import agent_brief

SCHEMA = "fmn.agent.task-semantics"
SCHEMA_VERSION = 1
MAX_JSON_DEPTH = 64
MAX_JSON_NODES = 100_000
CORE_FIELDS = frozenset(
    {
        "id",
        "title",
        "status",
        "priority",
        "issue_type",
        "assignee",
        "updated_at",
        "dependencies",
        "comments",
    }
)
DEPENDENCY_CORE_FIELDS = frozenset({"issue_id", "depends_on_id", "type"})


class SemanticError(ValueError):
    pass


class _DuplicateJsonKey(ValueError):
    def __init__(self, key: str):
        super().__init__(key)
        self.key = key


class _NonFiniteJsonConstant(ValueError):
    def __init__(self, spelling: str):
        super().__init__(spelling)
        self.spelling = spelling


class SemanticIssues(dict[str, agent_brief.Issue]):
    """A normal issue map carrying exact canonical extension-field evidence."""

    def __init__(
        self,
        issues: dict[str, agent_brief.Issue],
        *,
        task_semantics: dict[str, dict[str, Any]],
        ledger_sha256: str,
        ledger_path: Path,
    ) -> None:
        super().__init__(issues)
        self.task_semantics = task_semantics
        self.ledger_sha256 = ledger_sha256
        self.ledger_path = ledger_path


_BaseLoader = Callable[[Path], dict[str, agent_brief.Issue]]
_expected_semantics: contextvars.ContextVar[tuple[str, dict[str, str]] | None] = (
    contextvars.ContextVar("fmn_expected_task_semantics", default=None)
)


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise _DuplicateJsonKey(key)
        result[key] = value
    return result


def _reject_nonfinite_constant(spelling: str) -> None:
    raise _NonFiniteJsonConstant(spelling)


def _validate_json_tree(value: Any, *, issue_id: str, line: int) -> None:
    stack: list[tuple[Any, int]] = [(value, 1)]
    nodes = 0
    while stack:
        current, depth = stack.pop()
        nodes += 1
        if nodes > MAX_JSON_NODES:
            raise SemanticError(
                f"{issue_id}: line {line} exceeds the {MAX_JSON_NODES}-node JSON limit"
            )
        if depth > MAX_JSON_DEPTH:
            raise SemanticError(
                f"{issue_id}: line {line} exceeds the {MAX_JSON_DEPTH}-level JSON depth limit"
            )
        if isinstance(current, dict):
            for key, item in current.items():
                stack.append((item, depth + 1))
                stack.append((key, depth + 1))
        elif isinstance(current, list):
            stack.extend((item, depth + 1) for item in current)
        elif isinstance(current, str):
            if any(0xD800 <= ord(character) <= 0xDFFF for character in current):
                raise SemanticError(
                    f"{issue_id}: line {line} contains an unpaired Unicode surrogate"
                )
        elif isinstance(current, float) and not math.isfinite(current):
            raise SemanticError(f"{issue_id}: line {line} contains a non-finite number")


def _read_ledger(path: Path) -> bytes:
    flags = os.O_RDONLY
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags)
    except OSError as exc:
        raise SemanticError(f"cannot open {path}: {exc}") from exc
    try:
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode):
            raise SemanticError(f"{path} is not a regular file")
        if before.st_size > agent_brief.MAX_LEDGER_BYTES:
            raise SemanticError(
                f"{path} exceeds the {agent_brief.MAX_LEDGER_BYTES}-byte ledger limit"
            )
        remaining = agent_brief.MAX_LEDGER_BYTES + 1
        chunks: list[bytes] = []
        while remaining > 0:
            chunk = os.read(descriptor, min(64 * 1024, remaining))
            if not chunk:
                break
            chunks.append(chunk)
            remaining -= len(chunk)
        data = b"".join(chunks)
        if len(data) > agent_brief.MAX_LEDGER_BYTES:
            raise SemanticError(
                f"{path} exceeds the {agent_brief.MAX_LEDGER_BYTES}-byte ledger limit"
            )
        after = os.fstat(descriptor)
        identity_before = (
            before.st_dev,
            before.st_ino,
            before.st_size,
            before.st_mtime_ns,
            before.st_ctime_ns,
        )
        identity_after = (
            after.st_dev,
            after.st_ino,
            after.st_size,
            after.st_mtime_ns,
            after.st_ctime_ns,
        )
        if identity_before != identity_after or len(data) != after.st_size:
            raise SemanticError(f"{path} changed while task semantics were being read")
        return data
    except OSError as exc:
        raise SemanticError(f"cannot read {path}: {exc}") from exc
    finally:
        os.close(descriptor)


def _normalize_labels(value: Any) -> Any:
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        return value
    return sorted(value)


def _dependency_semantics(raw: Any, *, issue_id: str) -> list[dict[str, Any]]:
    if raw is None:
        return []
    if not isinstance(raw, list):
        raise SemanticError(f"{issue_id}: dependencies must be an array or null")
    rows: list[dict[str, Any]] = []
    for index, dependency in enumerate(raw):
        if not isinstance(dependency, dict):
            raise SemanticError(f"{issue_id}: dependency {index} must be an object")
        extensions = {
            key: value
            for key, value in dependency.items()
            if key not in DEPENDENCY_CORE_FIELDS
        }
        if extensions:
            rows.append(
                {
                    "issue_id": dependency.get("issue_id"),
                    "depends_on_id": dependency.get("depends_on_id"),
                    "type": dependency.get("type"),
                    "fields": extensions,
                }
            )
    rows.sort(
        key=lambda row: (
            str(row["depends_on_id"]),
            str(row["type"]),
            str(row["issue_id"]),
            _canonical_json(row["fields"]),
        )
    )
    return rows


def _canonical_json(value: Any) -> str:
    try:
        return json.dumps(
            value,
            ensure_ascii=True,
            allow_nan=False,
            sort_keys=True,
            separators=(",", ":"),
        )
    except (TypeError, ValueError, RecursionError) as exc:
        raise SemanticError(f"task semantics cannot be canonically encoded: {exc}") from exc


def _record_semantics(raw: dict[str, Any], *, issue_id: str) -> dict[str, Any]:
    fields = {key: value for key, value in raw.items() if key not in CORE_FIELDS}
    if "labels" in fields:
        fields["labels"] = _normalize_labels(fields["labels"])
    dependency_fields = _dependency_semantics(raw.get("dependencies"), issue_id=issue_id)
    return {
        "schema": SCHEMA,
        "version": SCHEMA_VERSION,
        "fields": fields,
        "dependency_fields": dependency_fields,
    }


def load_task_semantics(
    path: Path,
) -> tuple[dict[str, dict[str, Any]], dict[str, agent_brief.Issue], str]:
    data = _read_ledger(path)
    records: dict[str, dict[str, Any]] = {}
    core_issues: dict[str, agent_brief.Issue] = {}
    dependency_count = 0
    for line_number, raw_line in enumerate(data.splitlines(keepends=True), 1):
        if len(raw_line) > agent_brief.MAX_LINE_BYTES:
            raise SemanticError(
                f"{path}:{line_number} exceeds the {agent_brief.MAX_LINE_BYTES}-byte line limit"
            )
        if not raw_line.endswith(b"\n"):
            raise SemanticError(f"{path}:{line_number} is missing its final LF")
        if not raw_line.strip():
            raise SemanticError(f"{path}:{line_number} is blank")
        try:
            raw = json.loads(
                raw_line,
                object_pairs_hook=_unique_object,
                parse_constant=_reject_nonfinite_constant,
            )
        except _DuplicateJsonKey as exc:
            raise SemanticError(
                f"{path}:{line_number}: duplicate JSON object key {exc.key!r}"
            ) from exc
        except _NonFiniteJsonConstant as exc:
            raise SemanticError(
                f"{path}:{line_number}: non-finite JSON constant {exc.spelling!r} is forbidden"
            ) from exc
        except (UnicodeDecodeError, json.JSONDecodeError, RecursionError, ValueError) as exc:
            raise SemanticError(f"{path}:{line_number}: invalid UTF-8 JSON: {exc}") from exc
        if not isinstance(raw, dict):
            raise SemanticError(f"{path}:{line_number}: issue record must be an object")
        issue_id = raw.get("id")
        if not isinstance(issue_id, str) or not issue_id:
            raise SemanticError(f"{path}:{line_number}: id must be a non-empty string")
        if issue_id in records:
            raise SemanticError(f"{path}:{line_number}: duplicate issue id {issue_id}")
        _validate_json_tree(raw, issue_id=issue_id, line=line_number)
        try:
            issue = agent_brief.parse_issue(raw, line=line_number)
        except agent_brief.BriefError as exc:
            raise SemanticError(str(exc)) from exc
        core_issues[issue_id] = issue
        dependency_count += len(issue.dependencies)
        if dependency_count > agent_brief.MAX_DEPENDENCIES:
            raise SemanticError(
                f"{path} exceeds the {agent_brief.MAX_DEPENDENCIES}-dependency limit"
            )
        records[issue_id] = _record_semantics(raw, issue_id=issue_id)
        if len(records) > agent_brief.MAX_ISSUES:
            raise SemanticError(f"{path} exceeds the {agent_brief.MAX_ISSUES}-issue limit")
    return records, core_issues, hashlib.sha256(data).hexdigest()


def load_semantic_issues(path: Path, base_loader: _BaseLoader) -> SemanticIssues:
    before, expected_core, before_sha256 = load_task_semantics(path)
    issues = base_loader(path)
    after, after_core, after_sha256 = load_task_semantics(path)
    if (
        before_sha256 != after_sha256
        or before != after
        or expected_core != after_core
    ):
        raise SemanticError(f"{path} changed while the planning graph was being loaded")
    if issues != expected_core:
        raise SemanticError(
            f"{path} core projection disagrees with the stable task-semantic read"
        )
    return SemanticIssues(
        issues,
        task_semantics=before,
        ledger_sha256=before_sha256,
        ledger_path=path,
    )


def install_loader() -> None:
    current = agent_brief.load_issues
    if getattr(current, "_fmn_task_semantics", False):
        return

    def semantic_loader(path: Path) -> SemanticIssues:
        try:
            return load_semantic_issues(path, current)
        except SemanticError as exc:
            raise agent_brief.BriefError(str(exc)) from exc

    semantic_loader._fmn_task_semantics = True  # type: ignore[attr-defined]
    semantic_loader._fmn_base_loader = current  # type: ignore[attr-defined]
    agent_brief.load_issues = semantic_loader


def semantics_for(issues: dict[str, agent_brief.Issue]) -> dict[str, dict[str, Any]]:
    if isinstance(issues, SemanticIssues):
        if set(issues.task_semantics) != set(issues):
            raise SemanticError("task-semantic issue membership disagrees with the planning graph")
        return issues.task_semantics
    return {
        issue_id: {
            "schema": SCHEMA,
            "version": SCHEMA_VERSION,
            "fields": {},
            "dependency_fields": [],
        }
        for issue_id in issues
    }


def _ledger_identity(path: Path) -> str:
    return os.path.abspath(os.fspath(path))


def remember_semantics(issues: dict[str, agent_brief.Issue]) -> None:
    if isinstance(issues, SemanticIssues):
        canonical = {
            issue_id: _canonical_json(semantic)
            for issue_id, semantic in semantics_for(issues).items()
        }
        _expected_semantics.set((_ledger_identity(issues.ledger_path), canonical))
    else:
        _expected_semantics.set(None)


def verify_remembered_semantics(issues: dict[str, agent_brief.Issue]) -> None:
    expected_state = _expected_semantics.get()
    if expected_state is None or not isinstance(issues, SemanticIssues):
        return
    expected_path, expected = expected_state
    if _ledger_identity(issues.ledger_path) != expected_path:
        return
    actual = {
        issue_id: _canonical_json(semantic)
        for issue_id, semantic in semantics_for(issues).items()
    }
    if actual != expected:
        changed = sorted(
            issue_id
            for issue_id in set(expected) | set(actual)
            if expected.get(issue_id) != actual.get(issue_id)
        )
        display = changed[:20]
        suffix = "" if len(changed) <= 20 else f" (+{len(changed) - 20} more)"
        raise SemanticError(
            "claim postcondition observed task-semantic field drift in "
            f"{display!r}{suffix}"
        )
    _expected_semantics.set(None)
