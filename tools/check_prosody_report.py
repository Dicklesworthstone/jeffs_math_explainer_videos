#!/usr/bin/env python3
"""Validate legacy prosody execution, not speaker identity or audible quality."""
import argparse
from pathlib import Path
import re


def valid_report(path, exit_code):
    if exit_code != 0:
        return False
    try:
        verdicts = re.findall(r"^(PASS|FLAG|FAIL|ADVISORY)\s", path.read_text(), re.MULTILINE)
    except (OSError, UnicodeError):
        return False
    return bool(verdicts) and "FAIL" not in verdicts


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", type=Path)
    parser.add_argument("exit_code", type=int)
    args = parser.parse_args()
    raise SystemExit(0 if valid_report(args.report, args.exit_code) else 1)
