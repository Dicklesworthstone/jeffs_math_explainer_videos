#!/usr/bin/env python3
"""Audit reviewed portal parity claims against an actually imported wheel.

The semantic contract lives in ``fmn_python.parity_audit`` and ships in the
wheel. This checkout wrapper contributes only bounded overlay I/O and command
exit behavior so source gates and installed-wheel gates cannot drift apart.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PORTAL_PYTHON = ROOT / "crates" / "fmn-python" / "python"
if str(PORTAL_PYTHON) not in sys.path:
    sys.path.insert(0, str(PORTAL_PYTHON))

from fmn_python.parity_audit import (  # noqa: E402
    MAX_OVERLAY_BYTES,
    ParityAuditError as AuditError,
    StatusRow,
    audit_overlay,
    audit_rows,
    parse_status_rows,
    render_json,
)


def _read_bounded(path: Path) -> str:
    try:
        with path.open("rb") as handle:
            payload = handle.read(MAX_OVERLAY_BYTES + 1)
    except OSError as exc:
        raise AuditError(f"cannot read overlay {path}: {exc}") from exc
    if len(payload) > MAX_OVERLAY_BYTES:
        raise AuditError(f"overlay exceeds {MAX_OVERLAY_BYTES}-byte limit")
    try:
        return payload.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise AuditError(f"overlay is not valid UTF-8: {exc}") from exc


def make_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--overlay", type=Path, default=Path("API_OVERLAY.tsv"))
    parser.add_argument("--check", action="store_true", help="exit 1 on runtime contradictions")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = make_parser().parse_args(argv)
    try:
        report = audit_overlay(_read_bounded(args.overlay))
        output = render_json(report)
    except AuditError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    sys.stdout.write(output)
    if args.check and not report["ok"]:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
