#!/usr/bin/env python3
"""Audit every fmn-library helper-to-manimlib constructor authority path."""

from __future__ import annotations

import argparse
import ast
import json
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
PORTAL_PYTHON = ROOT / "crates" / "fmn-python" / "python"
if str(PORTAL_PYTHON) not in sys.path:
    sys.path.insert(0, str(PORTAL_PYTHON))

from fmn_python.library_constructor_authority import (  # noqa: E402
    LIBRARY_CONSTRUCTOR_AUTHORITIES,
    REFERENCE_SYMBOL_BY_RUST_HELPER,
)

SCHEMA = "fmn.library-constructor-authority-audit"
SCHEMA_VERSION = 2
MAX_SOURCE_BYTES = 8 * 1024 * 1024
MAX_OUTPUT_BYTES = 256 * 1024
BOOTSTRAP_PATH = Path("crates/fmn-python/python/manimlib_bootstrap.py")
BRIDGE_PATH = Path("crates/fmn-python/src/lib.rs")
WHEEL_SMOKE_PATH = Path("crates/fmn-python/tests/wheel_smoke.py")
WHEEL_MAPPING_NAME = "REFERENCE_CONSTRUCTOR_ALIASES"


class AuthorityAuditError(ValueError):
    def __init__(
        self,
        code: str,
        detail: str,
        *,
        helper: str | None = None,
        path: Path | None = None,
    ) -> None:
        self.code = code
        self.detail = detail
        self.helper = helper
        self.path = path
        super().__init__(detail)


def _error(
    code: str,
    detail: str,
    *,
    helper: str | None = None,
    path: Path | None = None,
) -> AuthorityAuditError:
    return AuthorityAuditError(code, detail, helper=helper, path=path)


def _read_bounded(path: Path) -> str:
    try:
        with path.open("rb") as handle:
            payload = handle.read(MAX_SOURCE_BYTES + 1)
    except OSError as exc:
        raise _error(
            "source-read-failed",
            f"cannot read {path}: {exc}",
            path=path,
        ) from exc
    if len(payload) > MAX_SOURCE_BYTES:
        raise _error(
            "source-too-large",
            f"{path} exceeds the {MAX_SOURCE_BYTES}-byte limit",
            path=path,
        )
    try:
        return payload.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise _error(
            "source-not-utf8",
            f"{path} is not valid UTF-8: {exc}",
            path=path,
        ) from exc


def _parse_python(text: str, path: Path) -> ast.Module:
    try:
        return ast.parse(text, filename=str(path))
    except SyntaxError as exc:
        raise _error(
            "python-parse-failed",
            f"{path} is not valid Python: {exc}",
            path=path,
        ) from exc


def _top_level_class(
    tree: ast.Module,
    *,
    name: str,
    path: Path,
    helper: str,
) -> ast.ClassDef:
    matches = [
        node
        for node in tree.body
        if isinstance(node, ast.ClassDef) and node.name == name
    ]
    if len(matches) != 1:
        raise _error(
            "python-class-count",
            f"{path}: expected one top-level class {name}, found {len(matches)}",
            helper=helper,
            path=path,
        )
    return matches[0]


def _direct_method(
    class_node: ast.ClassDef,
    *,
    name: str,
    path: Path,
    helper: str,
) -> ast.FunctionDef | ast.AsyncFunctionDef:
    matches = [
        node
        for node in class_node.body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        and node.name == name
    ]
    if len(matches) != 1:
        raise _error(
            "python-method-count",
            f"{path}: expected one direct {class_node.name}.{name}, "
            f"found {len(matches)}",
            helper=helper,
            path=path,
        )
    return matches[0]


def _base_spelling(node: ast.ClassDef) -> str:
    return ", ".join(ast.unparse(base) for base in node.bases)


class _ExecutableCallCollector(ast.NodeVisitor):
    """Collect calls while refusing to treat nested code objects as evidence."""

    def __init__(self) -> None:
        self.calls: list[ast.Call] = []

    def visit_Call(self, node: ast.Call) -> None:
        self.calls.append(node)
        self.generic_visit(node)

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        del node

    def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
        del node

    def visit_ClassDef(self, node: ast.ClassDef) -> None:
        del node

    def visit_Lambda(self, node: ast.Lambda) -> None:
        del node


def _executable_calls(
    function: ast.FunctionDef | ast.AsyncFunctionDef,
) -> list[ast.Call]:
    collector = _ExecutableCallCollector()
    for statement in function.body:
        collector.visit(statement)
    return collector.calls


def _call_target(call: ast.Call) -> str:
    return ast.unparse(call.func)


def _expected_call(
    token: str,
    *,
    path: Path,
    helper: str,
) -> tuple[str, str | ast.Call]:
    stripped = token.strip()
    if stripped.endswith("("):
        target = stripped[:-1].strip()
        if not target:
            raise _error(
                "python-authority-contract",
                f"{path}: empty callable target in {token!r}",
                helper=helper,
                path=path,
            )
        return "target", target
    try:
        expression = ast.parse(stripped, mode="eval").body
    except SyntaxError as exc:
        raise _error(
            "python-authority-contract",
            f"{path}: invalid authority call {token!r}: {exc}",
            helper=helper,
            path=path,
        ) from exc
    if not isinstance(expression, ast.Call):
        raise _error(
            "python-authority-contract",
            f"{path}: authority token is not a call: {token!r}",
            helper=helper,
            path=path,
        )
    return "exact", expression


def _has_expected_call(
    function: ast.FunctionDef | ast.AsyncFunctionDef,
    token: str,
    *,
    path: Path,
    helper: str,
) -> bool:
    mode, expected = _expected_call(token, path=path, helper=helper)
    calls = _executable_calls(function)
    if mode == "target":
        assert isinstance(expected, str)
        return any(_call_target(call) == expected for call in calls)
    assert isinstance(expected, ast.Call)
    expected_dump = ast.dump(expected, include_attributes=False)
    return any(
        ast.dump(call, include_attributes=False) == expected_dump
        for call in calls
    )


def _native_lifecycle_init_points(
    class_node: ast.ClassDef,
    constructor: ast.FunctionDef | ast.AsyncFunctionDef,
) -> ast.FunctionDef | ast.AsyncFunctionDef | None:
    """The class's own init_points when its __init__ takes the native lifecycle.

    _init_native_vmobject runs the Reference's init_points hook, so a class
    whose __init__ calls it builds its geometry there, not in __init__.
    """
    if not any(
        _call_target(call) == "_init_native_vmobject"
        for call in _executable_calls(constructor)
    ):
        return None
    matches = [
        node
        for node in class_node.body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        and node.name == "init_points"
    ]
    return matches[0] if len(matches) == 1 else None


def _routes_native_builder(
    function: ast.FunctionDef | ast.AsyncFunctionDef,
    builder: str,
) -> bool:
    """Whether function publishes builder through _set_native_vmobject_points."""
    return any(
        _call_target(call) == "_set_native_vmobject_points"
        and len(call.args) >= 2
        and isinstance(call.args[1], ast.Constant)
        and call.args[1].value == builder
        for call in _executable_calls(function)
    )


def _require_expected_call(
    function: ast.FunctionDef | ast.AsyncFunctionDef,
    token: str,
    *,
    path: Path,
    helper: str,
    owner: str,
    code: str,
) -> None:
    if not _has_expected_call(function, token, path=path, helper=helper):
        raise _error(
            code,
            f"{path}: {owner}.__init__ lacks executable call {token!r}",
            helper=helper,
            path=path,
        )


def _blank_non_newlines(buffer: list[str], text: str, start: int, end: int) -> None:
    for index in range(start, end):
        if text[index] not in "\r\n":
            buffer[index] = " "


def _rust_raw_string_end(text: str, start: int) -> int | None:
    for prefix in ("br", "cr", "r"):
        if not text.startswith(prefix, start):
            continue
        cursor = start + len(prefix)
        hashes = 0
        while cursor < len(text) and text[cursor] == "#":
            hashes += 1
            cursor += 1
        if cursor >= len(text) or text[cursor] != '"':
            continue
        delimiter = '"' + ("#" * hashes)
        closing = text.find(delimiter, cursor + 1)
        return None if closing < 0 else closing + len(delimiter)
    return -1


def _rust_quoted_end(text: str, quote: int) -> int | None:
    cursor = quote + 1
    while cursor < len(text):
        if text[cursor] == "\\":
            cursor += 2
            continue
        if text[cursor] == '"':
            return cursor + 1
        cursor += 1
    return None


def _rust_char_end(text: str, quote: int) -> int | None:
    cursor = quote + 1
    if cursor >= len(text):
        return None
    if text[cursor] == "\\":
        cursor += 2
        if cursor < len(text) and text[cursor - 1] in {"u", "x"}:
            while cursor < len(text) and text[cursor] != "'":
                cursor += 1
    else:
        cursor += 1
    if cursor < len(text) and text[cursor] == "'":
        return cursor + 1
    return None


def _rust_code_mask(
    text: str,
    *,
    path: Path,
    helper: str | None = None,
) -> str:
    """Blank comments and literals while preserving byte positions/newlines."""

    masked = list(text)
    cursor = 0
    while cursor < len(text):
        if text.startswith("//", cursor):
            end = text.find("\n", cursor + 2)
            end = len(text) if end < 0 else end
            _blank_non_newlines(masked, text, cursor, end)
            cursor = end
            continue
        if text.startswith("/*", cursor):
            start = cursor
            depth = 1
            cursor += 2
            while cursor < len(text) and depth:
                if text.startswith("/*", cursor):
                    depth += 1
                    cursor += 2
                elif text.startswith("*/", cursor):
                    depth -= 1
                    cursor += 2
                else:
                    cursor += 1
            if depth:
                raise _error(
                    "rust-lex-failed",
                    f"{path}: unterminated block comment",
                    helper=helper,
                    path=path,
                )
            _blank_non_newlines(masked, text, start, cursor)
            continue

        raw_end = _rust_raw_string_end(text, cursor)
        if raw_end is None:
            raise _error(
                "rust-lex-failed",
                f"{path}: unterminated raw string",
                helper=helper,
                path=path,
            )
        if raw_end >= 0:
            _blank_non_newlines(masked, text, cursor, raw_end)
            cursor = raw_end
            continue

        quote = cursor
        if text[cursor] in {"b", "c"} and cursor + 1 < len(text):
            if text[cursor + 1] == '"':
                quote = cursor + 1
        if text[quote] == '"':
            end = _rust_quoted_end(text, quote)
            if end is None:
                raise _error(
                    "rust-lex-failed",
                    f"{path}: unterminated string literal",
                    helper=helper,
                    path=path,
                )
            _blank_non_newlines(masked, text, cursor, end)
            cursor = end
            continue

        char_quote = cursor
        if text[cursor] == "b" and cursor + 1 < len(text) and text[cursor + 1] == "'":
            char_quote = cursor + 1
        if text[char_quote] == "'":
            end = _rust_char_end(text, char_quote)
            if end is not None:
                _blank_non_newlines(masked, text, cursor, end)
                cursor = end
                continue
        cursor += 1
    return "".join(masked)


def _rust_function_block(
    text: str,
    *,
    name: str,
    path: Path,
    helper: str,
    top_level_public: bool,
) -> str:
    code = _rust_code_mask(text, path=path, helper=helper)
    if top_level_public:
        pattern = re.compile(
            rf"(?m)^pub\s+fn\s+{re.escape(name)}(?:<[^>\n]+>)?\s*\("
        )
    else:
        pattern = re.compile(
            rf"(?m)^[ \t]+fn\s+{re.escape(name)}(?:<[^>\n]+>)?\s*\("
        )
    matches = list(pattern.finditer(code))
    if len(matches) != 1:
        kind = "top-level public" if top_level_public else "bridge"
        raise _error(
            "rust-function-count",
            f"{path}: expected one {kind} function {name}, found {len(matches)}",
            helper=helper,
            path=path,
        )
    match = matches[0]
    opening = code.find("{", match.end())
    if opening < 0:
        raise _error(
            "rust-function-unclosed",
            f"{path}: function {name} has no body opening brace",
            helper=helper,
            path=path,
        )
    depth = 0
    for cursor in range(opening, len(code)):
        if code[cursor] == "{":
            depth += 1
        elif code[cursor] == "}":
            depth -= 1
            if depth == 0:
                return code[match.start() : cursor + 1]
    raise _error(
        "rust-function-unclosed",
        f"{path}: function {name} has no scoped closing brace",
        helper=helper,
        path=path,
    )


def _literal_assignment(
    tree: ast.Module,
    *,
    name: str,
    path: Path,
) -> Any:
    values: list[ast.expr] = []
    for node in tree.body:
        if isinstance(node, ast.Assign):
            targets = node.targets
            value = node.value
        elif isinstance(node, ast.AnnAssign):
            targets = [node.target]
            value = node.value
        else:
            continue
        if value is None:
            continue
        if any(
            isinstance(target, ast.Name) and target.id == name
            for target in targets
        ):
            values.append(value)
    if len(values) != 1:
        raise _error(
            "wheel-mapping-count",
            f"{path}: expected one {name} assignment, found {len(values)}",
            path=path,
        )
    try:
        return ast.literal_eval(values[0])
    except (TypeError, ValueError, SyntaxError) as exc:
        raise _error(
            "wheel-mapping-not-literal",
            f"{path}: {name} must be a literal mapping",
            path=path,
        ) from exc


def _wheel_mapping(text: str, path: Path) -> dict[str, tuple[str, str]]:
    raw = _literal_assignment(
        _parse_python(text, path),
        name=WHEEL_MAPPING_NAME,
        path=path,
    )
    if not isinstance(raw, dict):
        raise _error(
            "wheel-mapping-type",
            f"{path}: {WHEEL_MAPPING_NAME} must be a dict",
            path=path,
        )
    result: dict[str, tuple[str, str]] = {}
    for helper, symbol in raw.items():
        if (
            not isinstance(helper, str)
            or not isinstance(symbol, tuple)
            or len(symbol) != 2
            or not all(isinstance(part, str) and part for part in symbol)
        ):
            raise _error(
                "wheel-mapping-entry",
                f"{path}: malformed constructor alias entry {helper!r}: {symbol!r}",
                path=path,
            )
        result[helper] = symbol
    return result


def audit(root: Path = ROOT) -> dict[str, Any]:
    root = root.resolve()
    bootstrap_path = root / BOOTSTRAP_PATH
    bridge_path = root / BRIDGE_PATH
    wheel_path = root / WHEEL_SMOKE_PATH
    bootstrap_text = _read_bounded(bootstrap_path)
    bridge_text = _read_bounded(bridge_path)
    wheel_text = _read_bounded(wheel_path)
    bootstrap_tree = _parse_python(bootstrap_text, bootstrap_path)

    rust_sources = {
        record.rust_source: _read_bounded(root / record.rust_source)
        for record in LIBRARY_CONSTRUCTOR_AUTHORITIES
    }

    records = []
    for record in LIBRARY_CONSTRUCTOR_AUTHORITIES:
        helper = record.rust_helper
        rust_path = root / record.rust_source
        rust_block = _rust_function_block(
            rust_sources[record.rust_source],
            name=record.rust_function,
            path=rust_path,
            helper=helper,
            top_level_public=True,
        )
        if record.rust_authority_token not in rust_block:
            raise _error(
                "rust-authority-missing",
                f"{rust_path}: {record.rust_function} lacks executable "
                f"{record.rust_authority_token!r}",
                helper=helper,
                path=rust_path,
            )

        class_node = _top_level_class(
            bootstrap_tree,
            name=record.reference_class,
            path=bootstrap_path,
            helper=helper,
        )
        observed_base = _base_spelling(class_node)
        if observed_base != record.python_base:
            raise _error(
                "python-base-mismatch",
                f"{bootstrap_path}: {record.reference_class} bases are "
                f"{observed_base!r}, expected {record.python_base!r}",
                helper=helper,
                path=bootstrap_path,
            )
        constructor = _direct_method(
            class_node,
            name="__init__",
            path=bootstrap_path,
            helper=helper,
        )
        lifecycle = _native_lifecycle_init_points(class_node, constructor)
        if lifecycle is None or not _has_expected_call(
            lifecycle,
            record.python_authority_token,
            path=bootstrap_path,
            helper=helper,
        ):
            _require_expected_call(
                constructor,
                record.python_authority_token,
                path=bootstrap_path,
                helper=helper,
                owner=record.reference_class,
                code="python-authority-missing",
            )

        if (
            record.native_builder is not None
            and record.python_authority_class != record.reference_class
        ):
            authority_node = _top_level_class(
                bootstrap_tree,
                name=record.python_authority_class,
                path=bootstrap_path,
                helper=helper,
            )
            authority_constructor = _direct_method(
                authority_node,
                name="__init__",
                path=bootstrap_path,
                helper=helper,
            )
            authority_lifecycle = _native_lifecycle_init_points(
                authority_node, authority_constructor
            )
            if authority_lifecycle is None or not _routes_native_builder(
                authority_lifecycle, record.native_builder
            ):
                _require_expected_call(
                    authority_constructor,
                    f"self.{record.native_builder}(",
                    path=bootstrap_path,
                    helper=helper,
                    owner=record.python_authority_class,
                    code="python-native-builder-missing",
                )

        if record.bridge_function is not None:
            bridge_block = _rust_function_block(
                bridge_text,
                name=record.bridge_function,
                path=bridge_path,
                helper=helper,
                top_level_public=False,
            )
            if record.bridge_authority_token not in bridge_block:
                raise _error(
                    "bridge-authority-missing",
                    f"{bridge_path}: {record.bridge_function} lacks executable "
                    f"{record.bridge_authority_token!r}",
                    helper=helper,
                    path=bridge_path,
                )

        records.append(
            {
                "helper": helper,
                "reference": (
                    f"{record.reference_module}:{record.reference_class}"
                ),
                "binding_kind": record.binding_kind,
            }
        )

    expected_wheel = dict(REFERENCE_SYMBOL_BY_RUST_HELPER)
    observed_wheel = _wheel_mapping(wheel_text, wheel_path)
    if observed_wheel != expected_wheel:
        missing = sorted(expected_wheel.keys() - observed_wheel.keys())
        extra = sorted(observed_wheel.keys() - expected_wheel.keys())
        changed = sorted(
            helper
            for helper in expected_wheel.keys() & observed_wheel.keys()
            if expected_wheel[helper] != observed_wheel[helper]
        )
        raise _error(
            "wheel-mapping-drift",
            f"{wheel_path}: missing={missing}, extra={extra}, changed={changed}",
            path=wheel_path,
        )

    kind_counts = Counter(
        record.binding_kind for record in LIBRARY_CONSTRUCTOR_AUTHORITIES
    )
    return {
        "schema": SCHEMA,
        "version": SCHEMA_VERSION,
        "ok": True,
        "proof_model": "executable-constructor-routing",
        "counts": {
            "authorities": len(records),
            "rust_sources": len(rust_sources),
            "native_bridges": sum(
                record.bridge_function is not None
                for record in LIBRARY_CONSTRUCTOR_AUTHORITIES
            ),
            "python_owned": sum(
                record.bridge_function is None
                for record in LIBRARY_CONSTRUCTOR_AUTHORITIES
            ),
        },
        "binding_kinds": {
            name: kind_counts[name] for name in sorted(kind_counts)
        },
        "records": records,
    }


def render_json(report: dict[str, Any]) -> str:
    text = json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n"
    if len(text.encode("utf-8")) > MAX_OUTPUT_BYTES:
        raise _error(
            "output-too-large",
            f"audit output exceeds the {MAX_OUTPUT_BYTES}-byte limit",
        )
    return text


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--robot", action="store_true")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = make_parser().parse_args(argv)
    try:
        report = audit(args.root)
    except AuthorityAuditError as exc:
        if args.robot:
            payload = {
                "schema": SCHEMA,
                "version": SCHEMA_VERSION,
                "ok": False,
                "error": {
                    "code": exc.code,
                    "detail": exc.detail[:4096],
                },
            }
            if exc.helper is not None:
                payload["error"]["helper"] = exc.helper
            if exc.path is not None:
                payload["error"]["path"] = str(exc.path)
            sys.stdout.write(render_json(payload))
        else:
            prefix = f"{exc.helper}: " if exc.helper else ""
            print(
                f"library constructor authority audit: FAIL: "
                f"{exc.code}: {prefix}{exc.detail}",
                file=sys.stderr,
            )
        return 2
    if args.robot:
        sys.stdout.write(render_json(report))
    else:
        counts = report["counts"]
        print(
            "library constructor authority audit: PASS; "
            f"{counts['authorities']} helpers, "
            f"{counts['native_bridges']} native bridge routes, "
            f"{counts['python_owned']} Python-owned identity routes"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
