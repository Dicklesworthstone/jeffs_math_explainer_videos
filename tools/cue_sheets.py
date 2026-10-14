#!/usr/bin/env python3
"""Thin wrapper that delegates to the installed math-explainer skill's cue_sheets.py (plan §3.1, D-15)."""

import os
import sys
from pathlib import Path


def main() -> int:
    skills = Path(os.environ.get("SKILLS", str(Path.home() / ".claude" / "skills")))
    target = skills / "math-explainer-videos-end-to-end-with-franken-manim-frankentts" / "scripts" / "cue_sheets.py"
    if not target.is_file():
        sys.stderr.write(f"cue_sheets.py: not found at {target} (set SKILLS in project.env)\n")
        return 2
    os.execv(sys.executable, [sys.executable, "-I", str(target), *sys.argv[1:]])


if __name__ == "__main__":
    raise SystemExit(main())
