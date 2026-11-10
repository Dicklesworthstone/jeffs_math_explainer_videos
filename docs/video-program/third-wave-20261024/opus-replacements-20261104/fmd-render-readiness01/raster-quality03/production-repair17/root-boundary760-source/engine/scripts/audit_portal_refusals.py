#!/usr/bin/env python3
"""Inventory and validate explicit refusal sites in the Python portal.

The Python skin is allowed to refuse capabilities that the native engine does
not yet expose, but those refusals must stay precise and mechanically visible.
This audit treats the bootstrap as untrusted source, inventories direct
``NotImplementedError`` raises and ``_refuse_unrouted`` calls, and rejects
anonymous failures outside abstract methods. File identity, source size, AST
size/depth, site count, and output size are all bounded before publication.
"""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import os
import stat
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

DEFAULT_SOURCE = Path("crates/fmn-python/python/manimlib_bootstrap.py")
MAX_SOURCE_BYTES = 8 * 1024 * 1024
MAX_AST_NODES = 1_000_000
MAX_AST_DEPTH = 4_096
MAX_SITES = 4_096
MAX_OUTPUT_BYTES = 4 * 1024 * 1024
SCHEMA = "fmn.portal.refusals"
SCHEMA_VERSION = 2


class AuditError(ValueError):
    pass


@dataclass(frozen=True)
class Context:
    scope: tuple[str, ...]
    abstract: bool

    @property
    def qualname(self) -> str:
        return ".".join(self.scope) if self.scope else "<module>"


def final_name(node: ast.AST | None) -> str | None:
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        return node.attr
    return None


def static_string(node: ast.AST | None) -> str | None:
    if isinstance(node, ast.Constant) and isinstance(node.value, str):
        return node.value
    if isinstance(node, ast.JoinedStr):
        parts: list[str] = []
        for value in node.values:
            if not isinstance(value, ast.Constant) or not isinstance(value.value, str):
                return None
            parts.append(value.value)
        return "".join(parts)
    return None


def static_blank_string(node: ast.AST | None) -> bool:
    value = static_string(node)
    return value is not None and not value.strip()


def static_empty_collection(node: ast.AST | None) -> bool:
    if isinstance(node, (ast.List, ast.Tuple, ast.Set)):
        return not node.elts
    if isinstance(node, ast.Dict):
        return not node.keys
    return bool(
        isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name)
        and node.func.id in {"dict", "frozenset", "list", "set", "tuple"}
        and not node.args
        and not node.keywords
    )


def is_abstract(decorators: list[ast.expr]) -> bool:
    return any(final_name(decorator) == "abstractmethod" for decorator in decorators)


def file_identity(metadata: os.stat_result) -> tuple[int, int]:
    return metadata.st_dev, metadata.st_ino


def read_source(path: Path) -> tuple[bytes, str]:
    try:
        before = os.lstat(path)
    except OSError as exc:
        raise AuditError(f"cannot inspect portal source {path}: {exc}") from exc
    if stat.S_ISLNK(before.st_mode):
        raise AuditError(f"refusing symlink portal source {path}")
    if not stat.S_ISREG(before.st_mode):
        raise AuditError(f"portal source is not a regular file: {path}")
    if before.st_size > MAX_SOURCE_BYTES:
        raise AuditError(
            f"portal source exceeds the {MAX_SOURCE_BYTES}-byte input limit: {path}"
        )

    flags = os.O_RDONLY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    descriptor: int | None = None
    try:
        descriptor = os.open(path, flags)
        opened = os.fstat(descriptor)
        after = os.lstat(path)
        identity = file_identity(opened)
        if not stat.S_ISREG(opened.st_mode):
            raise AuditError(f"portal source is not a regular file: {path}")
        if file_identity(before) != identity or file_identity(after) != identity:
            raise AuditError(f"portal source changed while opening: {path}")
        if opened.st_size > MAX_SOURCE_BYTES:
            raise AuditError(
                f"portal source exceeds the {MAX_SOURCE_BYTES}-byte input limit: {path}"
            )
        with os.fdopen(descriptor, "rb") as handle:
            descriptor = None
            data = handle.read(MAX_SOURCE_BYTES + 1)
    except AuditError:
        if descriptor is not None:
            os.close(descriptor)
        raise
    except OSError as exc:
        if descriptor is not None:
            os.close(descriptor)
        raise AuditError(f"cannot read portal source {path}: {exc}") from exc

    if len(data) > MAX_SOURCE_BYTES:
        raise AuditError(
            f"portal source exceeds the {MAX_SOURCE_BYTES}-byte input limit: {path}"
        )
    try:
        return data, data.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise AuditError(f"portal source is not valid UTF-8: {path}: {exc}") from exc


class RefusalScanner:
    """Scope-aware iterative AST scanner with finite work and stack bounds."""

    def __init__(self, source: str) -> None:
        self.source = source
        self.sites: list[dict[str, Any]] = []

    def expression(self, node: ast.AST | None, limit: int = 180) -> str | None:
        if node is None:
            return None
        try:
            text = ast.get_source_segment(self.source, node)
        except (IndexError, TypeError, ValueError):
            text = None
        if text is None:
            text = type(node).__name__
        text = " ".join(text.split())
        return text if len(text) <= limit else text[: limit - 1] + "…"

    def add_site(
        self,
        node: ast.AST,
        context: Context,
        *,
        kind: str,
        subject: ast.AST | None,
        detail: ast.AST | None,
        violations: list[str],
        bare: bool = False,
    ) -> None:
        if len(self.sites) >= MAX_SITES:
            raise AuditError(f"portal refusal inventory exceeds the {MAX_SITES}-site limit")
        self.sites.append(
            {
                "line": getattr(node, "lineno", 0),
                "column": getattr(node, "col_offset", 0),
                "qualname": context.qualname,
                "kind": kind,
                "subject": self.expression(subject),
                "detail": self.expression(detail),
                "abstract": context.abstract,
                "bare": bare,
                "violations": violations,
            }
        )

    def scan_raise(self, node: ast.Raise, context: Context) -> None:
        exception = node.exc
        call = exception if isinstance(exception, ast.Call) else None
        target = call.func if call is not None else exception
        if final_name(target) != "NotImplementedError":
            return
        violations: list[str] = []
        message = call.args[0] if call is not None and call.args else None
        if call is None:
            if not context.abstract:
                violations.append("bare NotImplementedError outside an abstract method")
        elif message is None:
            violations.append("NotImplementedError call has no message")
        elif isinstance(message, ast.Starred):
            violations.append("NotImplementedError message uses unverifiable *args expansion")
        elif static_blank_string(message):
            violations.append("NotImplementedError message is blank")
        self.add_site(
            node,
            context,
            kind="not_implemented",
            subject=target,
            detail=message,
            violations=violations,
            bare=call is None,
        )

    def scan_refuse_call(self, node: ast.Call, context: Context) -> None:
        if final_name(node.func) != "_refuse_unrouted":
            return
        violations: list[str] = []
        positional = list(node.args)
        if len(positional) > 2:
            violations.append("_refuse_unrouted call has more than two positional arguments")
        if any(isinstance(argument, ast.Starred) for argument in positional):
            violations.append("_refuse_unrouted call uses unverifiable *args expansion")

        keyword_values: dict[str, ast.AST] = {}
        for keyword in node.keywords:
            if keyword.arg is None:
                violations.append("_refuse_unrouted call uses unverifiable **kwargs expansion")
                continue
            if keyword.arg in keyword_values:
                violations.append(f"_refuse_unrouted repeats keyword {keyword.arg!r}")
            keyword_values[keyword.arg] = keyword.value

        subject_candidates: list[ast.AST] = []
        if positional and not isinstance(positional[0], ast.Starred):
            subject_candidates.append(positional[0])
        for name in ("class_name", "subject"):
            if name in keyword_values:
                subject_candidates.append(keyword_values[name])
        if len(subject_candidates) > 1:
            violations.append("_refuse_unrouted call supplies the subject more than once")
        subject = subject_candidates[0] if subject_candidates else None

        entry_candidates: list[ast.AST] = []
        if len(positional) > 1 and not isinstance(positional[1], ast.Starred):
            entry_candidates.append(positional[1])
        if "entries" in keyword_values:
            entry_candidates.append(keyword_values["entries"])
        if len(entry_candidates) > 1:
            violations.append("_refuse_unrouted call supplies entries more than once")
        entries = entry_candidates[0] if entry_candidates else None

        if subject is None:
            violations.append("_refuse_unrouted call has no subject")
        elif static_blank_string(subject):
            violations.append("_refuse_unrouted subject is blank")
        if entries is None:
            violations.append("_refuse_unrouted call has no entries argument")
        elif static_empty_collection(entries):
            violations.append("_refuse_unrouted entries are statically empty")
        self.add_site(
            node,
            context,
            kind="refuse_unrouted",
            subject=subject,
            detail=entries,
            violations=violations,
        )

    @staticmethod
    def scoped_children(
        node: ast.AST,
        context: Context,
    ) -> list[tuple[ast.AST, Context]]:
        children = list(ast.iter_child_nodes(node))
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            body_ids = {id(child) for child in node.body}
            body_context = Context(
                scope=(*context.scope, node.name),
                abstract=is_abstract(node.decorator_list),
            )
            return [
                (child, body_context if id(child) in body_ids else context)
                for child in children
            ]
        if isinstance(node, ast.ClassDef):
            body_ids = {id(child) for child in node.body}
            body_context = Context(scope=(*context.scope, node.name), abstract=False)
            return [
                (child, body_context if id(child) in body_ids else context)
                for child in children
            ]
        if isinstance(node, ast.Lambda):
            return [
                (
                    child,
                    Context(scope=(*context.scope, "<lambda>"), abstract=False)
                    if child is node.body
                    else context,
                )
                for child in children
            ]
        return [(child, context) for child in children]

    def scan(self, tree: ast.AST) -> tuple[int, int]:
        stack: list[tuple[ast.AST, Context, int]] = [
            (tree, Context(scope=(), abstract=False), 0)
        ]
        node_count = 0
        max_depth = 0
        while stack:
            node, context, depth = stack.pop()
            node_count += 1
            if node_count > MAX_AST_NODES:
                raise AuditError(
                    f"portal source exceeds the {MAX_AST_NODES}-node AST limit"
                )
            if depth > MAX_AST_DEPTH:
                raise AuditError(
                    f"portal source exceeds the {MAX_AST_DEPTH}-level AST depth limit"
                )
            max_depth = max(max_depth, depth)
            if isinstance(node, ast.Raise):
                self.scan_raise(node, context)
            if isinstance(node, ast.Call):
                self.scan_refuse_call(node, context)
            for child, child_context in reversed(self.scoped_children(node, context)):
                stack.append((child, child_context, depth + 1))
        return node_count, max_depth


def build_inventory(path: Path) -> dict[str, Any]:
    source_bytes, source = read_source(path)
    try:
        tree = ast.parse(source, filename=str(path))
    except (SyntaxError, ValueError) as exc:
        line = getattr(exc, "lineno", None)
        offset = getattr(exc, "offset", None)
        location = f"{line}:{offset}" if line is not None else "unknown"
        message = getattr(exc, "msg", str(exc))
        raise AuditError(f"portal source is not valid Python at {location}: {message}") from exc
    except (MemoryError, RecursionError) as exc:
        raise AuditError(f"portal source exhausted parser resources: {type(exc).__name__}") from exc

    scanner = RefusalScanner(source)
    ast_nodes, ast_depth = scanner.scan(tree)
    sites = sorted(
        scanner.sites,
        key=lambda site: (site["line"], site["column"], site["kind"], site["qualname"]),
    )
    violations = [
        {
            "line": site["line"],
            "column": site["column"],
            "qualname": site["qualname"],
            "kind": site["kind"],
            "messages": site["violations"],
        }
        for site in sites
        if site["violations"]
    ]
    direct = [site for site in sites if site["kind"] == "not_implemented"]
    helpers = [site for site in sites if site["kind"] == "refuse_unrouted"]
    return {
        "schema": SCHEMA,
        "version": SCHEMA_VERSION,
        "source": path.as_posix(),
        "source_sha256": hashlib.sha256(source_bytes).hexdigest(),
        "source_bytes": len(source_bytes),
        "ast_nodes": ast_nodes,
        "ast_depth": ast_depth,
        "counts": {
            "sites": len(sites),
            "not_implemented": len(direct),
            "abstract_bare": sum(site["abstract"] and site["bare"] for site in direct),
            "refuse_unrouted": len(helpers),
            "violations": len(violations),
        },
        "violations": violations,
        "sites": sites,
    }


def bounded(text: str) -> str:
    size = len(text.encode("utf-8"))
    if size > MAX_OUTPUT_BYTES:
        raise AuditError(
            f"portal refusal report exceeds the {MAX_OUTPUT_BYTES}-byte output limit ({size} bytes)"
        )
    return text


def render_json(inventory: dict[str, Any]) -> str:
    return bounded(json.dumps(inventory, sort_keys=True, separators=(",", ":")) + "\n")


def render_markdown(inventory: dict[str, Any]) -> str:
    counts = inventory["counts"]
    lines = [
        "# Python portal refusal inventory",
        "",
        f"- Source: `{inventory['source']}`",
        f"- SHA-256: `{inventory['source_sha256']}`",
        (
            f"- Source bytes / AST nodes / AST depth: **{inventory['source_bytes']}** / "
            f"**{inventory['ast_nodes']}** / **{inventory['ast_depth']}**"
        ),
        (
            f"- Sites: **{counts['sites']}** total; **{counts['not_implemented']}** direct "
            f"`NotImplementedError`; **{counts['refuse_unrouted']}** `_refuse_unrouted`; "
            f"**{counts['abstract_bare']}** abstract bare; **{counts['violations']}** violations."
        ),
        "",
    ]
    for site in inventory["sites"]:
        subject = f" — `{site['subject']}`" if site["subject"] else ""
        lines.append(
            f"- `{site['line']}:{site['column']}` **{site['kind']}** "
            f"in `{site['qualname']}`{subject}"
        )
        for message in site["violations"]:
            lines.append(f"  - **VIOLATION:** {message}")
    return bounded("\n".join(lines).rstrip() + "\n")


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=DEFAULT_SOURCE)
    parser.add_argument("--format", choices=("json", "markdown"), default="json")
    parser.add_argument(
        "--check",
        action="store_true",
        help="exit nonzero and emit no report when anonymous refusal sites exist",
    )
    return parser


def main(argv: list[str] | None = None) -> int:
    args = make_parser().parse_args(argv)
    try:
        inventory = build_inventory(args.source)
    except AuditError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    if args.check and inventory["violations"]:
        print(
            f"error: Python portal has {len(inventory['violations'])} anonymous refusal site(s)",
            file=sys.stderr,
        )
        for violation in inventory["violations"]:
            print(
                f"error: {violation['line']}:{violation['column']} "
                f"{violation['qualname']}: {'; '.join(violation['messages'])}",
                file=sys.stderr,
            )
        return 1
    try:
        output = render_json(inventory) if args.format == "json" else render_markdown(inventory)
    except AuditError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
