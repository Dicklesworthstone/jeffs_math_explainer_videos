#!/usr/bin/env python3
"""Write the two patches from stage04's bytes to the candidates in this bundle.

    python3 -I -B <this file>

Run from the repo root. Writes only into this bundle's patches/ folder.
"""

import difflib
import re
from pathlib import Path

REL = "explainer_corpus/logarithm_mysteries"
STAGE = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage04") / REL
HERE = Path(__file__).resolve().parent.parent
OUT = HERE / "patches"

PATCHES = {
    "01-check-cues.patch": ["docs/check_cues.py", "docs/test_check_cues.py"],
    "02-storyboard.patch": ["docs/storyboard.md"],
}


def keep_ends(text):
    return re.findall(r".*\n|.+\Z", text)


def main():
    OUT.mkdir(exist_ok=True)
    for name, files in PATCHES.items():
        out = []
        for rel in files:
            base = STAGE / rel
            old = base.read_text(encoding="utf-8") if base.is_file() else ""
            new = (HERE / REL / rel).read_text(encoding="utf-8")
            if not new.endswith("\n") or (old and not old.endswith("\n")):
                raise SystemExit(f"{rel}: a file without a final newline needs a marker this writer doesn't add")
            label = f"{REL}/{rel}"
            out.extend(difflib.unified_diff(keep_ends(old), keep_ends(new), f"a/{label}" if old else "/dev/null", f"b/{label}"))
        (OUT / name).write_text("".join(out), encoding="utf-8")
        print(name, sum(line.startswith("+") for line in out), "added lines", sum(line.startswith("-") for line in out), "removed lines")


if __name__ == "__main__":
    main()
