#!/usr/bin/env python3
"""Verify one installed-wheel parity receipt against checkout authorities."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections.abc import Mapping
from pathlib import Path
from typing import Any

AUDIT_SCHEMA = "fmn.portal.runtime-audit"
AUDIT_VERSION = 1
RECEIPT_SCHEMA = "fmn.portal.runtime-receipt"
RECEIPT_VERSION = 1
MAX_REPORT_BYTES = 4 * 1024 * 1024
MAX_AUTHORITY_BYTES = 8 * 1024 * 1024
MAX_OUTPUT_BYTES = 64 * 1024
MAX_JSON_DEPTH = 32
MAX_JSON_NODES = 100_000
STATUS_NAMES = ("excluded", "improved", "same", "tiered", "unreviewed")
REPORT_KEYS = frozenset(
    {
        "schema",
        "version",
        "ok",
        "counts",
        "status_counts",
        "contradictions",
        "overlay_sha256",
        "api_schema_sha256",
        "schema_provenance",
    }
)
AUDIT_COUNT_KEYS = frozenset(
    {
        "status_rows",
        "reviewed_implemented",
        "runtime_placeholders",
        "trivial_bodies",
        "missing_reviewed",
        "contradictions",
    }
)
PROVENANCE_COUNT_KEYS = frozenset(
    {"schema_rows", "modules", "classes", "constructors", "functions", "methods"}
)


class ReceiptError(ValueError):
    def __init__(self, message: str, *, exit_code: int, identity: str) -> None:
        self.exit_code = exit_code
        self.identity = identity
        super().__init__(message)


def _invalid(message: str) -> ReceiptError:
    return ReceiptError(message, exit_code=2, identity="invalid-receipt")


def _stale(message: str) -> ReceiptError:
    return ReceiptError(message, exit_code=1, identity="stale-artifact")


def _failed(message: str) -> ReceiptError:
    return ReceiptError(message, exit_code=1, identity="audit-failed")


def _read_bounded(path: Path, *, label: str, limit: int) -> bytes:
    try:
        with path.open("rb") as handle:
            payload = handle.read(limit + 1)
    except OSError as exc:
        raise _invalid(f"cannot read {label} {path}: {exc}") from exc
    if len(payload) > limit:
        raise _invalid(f"{label} exceeds {limit}-byte limit")
    return payload


def _decode_utf8(payload: bytes, *, label: str) -> str:
    try:
        return payload.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise _invalid(f"{label} is not valid UTF-8: {exc}") from exc


def _reject_constant(value: str) -> Any:
    raise _invalid(f"report contains forbidden non-finite constant {value}")


def _object_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise _invalid(f"report contains duplicate JSON key {key!r}")
        result[key] = value
    return result


def _validate_structure(value: Any) -> None:
    stack: list[tuple[Any, int]] = [(value, 0)]
    nodes = 0
    while stack:
        current, depth = stack.pop()
        nodes += 1
        if nodes > MAX_JSON_NODES:
            raise _invalid(f"report exceeds {MAX_JSON_NODES}-node limit")
        if depth > MAX_JSON_DEPTH:
            raise _invalid(f"report exceeds {MAX_JSON_DEPTH}-level depth limit")
        if isinstance(current, dict):
            stack.extend((item, depth + 1) for item in current.values())
        elif isinstance(current, list):
            stack.extend((item, depth + 1) for item in current)


def _load_report(path: Path) -> dict[str, Any]:
    payload = _read_bounded(path, label="audit report", limit=MAX_REPORT_BYTES)
    text = _decode_utf8(payload, label="audit report")
    try:
        value = json.loads(
            text,
            object_pairs_hook=_object_pairs,
            parse_constant=_reject_constant,
        )
    except ReceiptError:
        raise
    except (json.JSONDecodeError, RecursionError, ValueError, OverflowError) as exc:
        raise _invalid(f"audit report is not valid bounded JSON: {exc}") from exc
    _validate_structure(value)
    if not isinstance(value, dict):
        raise _invalid("audit report must be a JSON object")
    return value


def _require_exact_keys(value: Mapping[str, Any], expected: frozenset[str], label: str) -> None:
    actual = set(value)
    missing = sorted(expected - actual)
    unknown = sorted(actual - expected)
    if missing or unknown:
        details = []
        if missing:
            details.append(f"missing {', '.join(missing)}")
        if unknown:
            details.append(f"unknown {', '.join(unknown)}")
        raise _invalid(f"{label} has {'; '.join(details)}")


def _mapping(value: Any, *, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise _invalid(f"{label} must be an object")
    return value


def _nonnegative_int(value: Any, *, label: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise _invalid(f"{label} must be a nonnegative integer")
    return value


def _digest(value: Any, *, label: str) -> str:
    if not isinstance(value, str) or len(value) != 64:
        raise _invalid(f"{label} must be a lowercase SHA-256 hex digest")
    if any(character not in "0123456789abcdef" for character in value):
        raise _invalid(f"{label} must be a lowercase SHA-256 hex digest")
    return value


def _parse_schema(text: str) -> tuple[int, int, dict[str, int]]:
    section: str | None = None
    rows = 0
    modules: set[str] = set()
    kinds: dict[str, int] = {}
    for line_number, raw_line in enumerate(text.splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1]
            continue
        if section != "symbols":
            continue
        columns = raw_line.split("\t")
        if len(columns) != 6:
            raise _invalid(
                f"API schema [symbols] line {line_number} must contain exactly 6 columns"
            )
        module_name, qualified, kind = (column.strip() for column in columns[:3])
        if not module_name or not qualified or not kind:
            raise _invalid(f"API schema [symbols] line {line_number} has an empty identity")
        rows += 1
        modules.add(module_name)
        kinds[kind] = kinds.get(kind, 0) + 1
    if rows == 0:
        raise _invalid("API schema contains no [symbols] rows")
    return rows, len(modules), kinds


def _parse_overlay_status_counts(text: str) -> dict[str, int]:
    section: str | None = None
    counts = {status: 0 for status in STATUS_NAMES}
    seen: set[str] = set()
    for line_number, raw_line in enumerate(text.splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1]
            continue
        if section != "status":
            continue
        columns = raw_line.split("\t")
        if len(columns) != 5:
            raise _invalid(
                f"API overlay [status] line {line_number} must contain exactly 5 columns"
            )
        symbol, status = (column.strip() for column in columns[:2])
        if ":" not in symbol or not all(symbol.split(":", 1)):
            raise _invalid(f"API overlay [status] line {line_number} has an invalid symbol")
        if status not in counts:
            raise _invalid(
                f"API overlay [status] line {line_number} has unknown status {status!r}"
            )
        if symbol in seen:
            raise _invalid(f"API overlay repeats [status] symbol {symbol}")
        seen.add(symbol)
        counts[status] += 1
    if not seen:
        raise _invalid("API overlay contains no [status] rows")
    return counts


def _validate_report(
    report: dict[str, Any],
    *,
    schema_rows: int,
    schema_modules: int,
    schema_kinds: Mapping[str, int],
    overlay_counts: Mapping[str, int],
) -> tuple[dict[str, int], dict[str, int]]:
    _require_exact_keys(report, REPORT_KEYS, "audit report")
    if report["schema"] != AUDIT_SCHEMA or report["version"] != AUDIT_VERSION:
        raise _invalid("audit report has an unknown schema contract")
    if report["ok"] is not True:
        raise _failed("audit report does not carry a successful verdict")

    counts = _mapping(report["counts"], label="audit report counts")
    _require_exact_keys(counts, AUDIT_COUNT_KEYS, "audit report counts")
    normalized_counts = {
        key: _nonnegative_int(value, label=f"audit report counts.{key}")
        for key, value in counts.items()
    }
    if normalized_counts["runtime_placeholders"] != 0:
        raise _failed("successful audit reports runtime placeholders")
    if normalized_counts["trivial_bodies"] != 0:
        raise _failed("successful audit reports trivial stub bodies")
    if normalized_counts["missing_reviewed"] != 0:
        raise _failed("successful audit reports missing reviewed symbols")
    if normalized_counts["contradictions"] != 0:
        raise _failed("successful audit reports contradictions")
    if report["contradictions"] != []:
        raise _failed("successful audit carries contradiction records")

    status_counts = _mapping(report["status_counts"], label="audit status counts")
    _require_exact_keys(status_counts, frozenset(STATUS_NAMES), "audit status counts")
    normalized_status = {
        key: _nonnegative_int(value, label=f"audit status counts.{key}")
        for key, value in status_counts.items()
    }
    if normalized_status != dict(overlay_counts):
        raise _stale("audit status counts disagree with checkout API_OVERLAY.tsv")
    status_rows = sum(normalized_status.values())
    reviewed = normalized_status["same"] + normalized_status["improved"]
    if normalized_counts["status_rows"] != status_rows:
        raise _invalid("audit report status_rows disagrees with status_counts")
    if normalized_counts["reviewed_implemented"] != reviewed:
        raise _invalid("audit report reviewed_implemented disagrees with status_counts")

    provenance = _mapping(report["schema_provenance"], label="schema provenance")
    _require_exact_keys(provenance, frozenset({"version", "counts"}), "schema provenance")
    if provenance["version"] != 1:
        raise _invalid("schema provenance has an unknown version")
    provenance_counts = _mapping(provenance["counts"], label="schema provenance counts")
    _require_exact_keys(
        provenance_counts,
        PROVENANCE_COUNT_KEYS,
        "schema provenance counts",
    )
    normalized_provenance = {
        key: _nonnegative_int(value, label=f"schema provenance counts.{key}")
        for key, value in provenance_counts.items()
    }
    if normalized_provenance["schema_rows"] != schema_rows:
        raise _stale("schema provenance row count disagrees with checkout API_SCHEMA.tsv")
    if normalized_provenance["modules"] != schema_modules + 1:
        raise _stale("schema provenance module count disagrees with assembled API schema")
    class_rows = schema_kinds.get("class", 0)
    function_rows = schema_kinds.get("function", 0) + schema_kinds.get("leaked_import", 0)
    method_rows = schema_kinds.get("method", 0)
    if normalized_provenance["classes"] > class_rows:
        raise _invalid("schema provenance reports more generated classes than class rows")
    if normalized_provenance["constructors"] > class_rows:
        raise _invalid("schema provenance reports more constructors than class rows")
    if normalized_provenance["functions"] > function_rows:
        raise _invalid("schema provenance reports more functions than eligible schema rows")
    if normalized_provenance["methods"] > method_rows:
        raise _invalid("schema provenance reports more methods than method rows")
    return normalized_counts, normalized_provenance


def verify_receipt(report_path: Path, schema_path: Path, overlay_path: Path) -> dict[str, Any]:
    report = _load_report(report_path)
    schema_payload = _read_bounded(
        schema_path,
        label="API schema",
        limit=MAX_AUTHORITY_BYTES,
    )
    overlay_payload = _read_bounded(
        overlay_path,
        label="API overlay",
        limit=MAX_AUTHORITY_BYTES,
    )
    schema_text = _decode_utf8(schema_payload, label="API schema")
    overlay_text = _decode_utf8(overlay_payload, label="API overlay")
    schema_rows, schema_modules, schema_kinds = _parse_schema(schema_text)
    overlay_counts = _parse_overlay_status_counts(overlay_text)
    counts, provenance = _validate_report(
        report,
        schema_rows=schema_rows,
        schema_modules=schema_modules,
        schema_kinds=schema_kinds,
        overlay_counts=overlay_counts,
    )

    embedded_schema = _digest(report["api_schema_sha256"], label="api_schema_sha256")
    embedded_overlay = _digest(report["overlay_sha256"], label="overlay_sha256")
    checkout_schema = hashlib.sha256(schema_payload).hexdigest()
    checkout_overlay = hashlib.sha256(overlay_payload).hexdigest()
    if embedded_schema != checkout_schema:
        raise _stale(
            f"installed wheel embeds stale API_SCHEMA.tsv: wheel={embedded_schema} "
            f"checkout={checkout_schema}"
        )
    if embedded_overlay != checkout_overlay:
        raise _stale(
            f"installed wheel embeds stale API_OVERLAY.tsv: wheel={embedded_overlay} "
            f"checkout={checkout_overlay}"
        )
    return {
        "schema": RECEIPT_SCHEMA,
        "version": RECEIPT_VERSION,
        "ok": True,
        "api_schema_sha256": checkout_schema,
        "api_overlay_sha256": checkout_overlay,
        "reviewed_implemented": counts["reviewed_implemented"],
        "schema_provenance": {
            "version": 1,
            "counts": provenance,
        },
    }


def _render_json(receipt: Mapping[str, Any]) -> str:
    text = json.dumps(receipt, sort_keys=True, separators=(",", ":")) + "\n"
    if len(text.encode("utf-8")) > MAX_OUTPUT_BYTES:
        raise _invalid(f"receipt output exceeds {MAX_OUTPUT_BYTES}-byte limit")
    return text


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    parser.add_argument("--schema", type=Path, default=Path("API_SCHEMA.tsv"))
    parser.add_argument("--overlay", type=Path, default=Path("API_OVERLAY.tsv"))
    parser.add_argument("--robot", action="store_true")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = make_parser().parse_args(argv)
    try:
        receipt = verify_receipt(args.report, args.schema, args.overlay)
        if args.robot:
            sys.stdout.write(_render_json(receipt))
        else:
            provenance = receipt["schema_provenance"]["counts"]
            generated = sum(
                provenance[key]
                for key in ("classes", "constructors", "functions", "methods")
            )
            print(
                "portal runtime receipt: PASS; "
                f"{receipt['reviewed_implemented']} reviewed rows, "
                f"{generated} generated fallback identities"
            )
            print(f"schema identity: {receipt['api_schema_sha256']}")
            print(f"overlay identity: {receipt['api_overlay_sha256']}")
    except ReceiptError as exc:
        print(
            f"portal runtime receipt: {exc.identity}: {exc}",
            file=sys.stderr,
        )
        return exc.exit_code
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
