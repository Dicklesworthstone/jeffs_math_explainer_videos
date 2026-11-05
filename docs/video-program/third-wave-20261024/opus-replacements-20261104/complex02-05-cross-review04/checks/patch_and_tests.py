#!/usr/bin/env python3
"""Does rebinding-tex-storyboard.patch fit, and do existing tests still name what the repair kept?
Run from the repo root:

    python3 -I -B patch_and_tests.py

1. Zero-fuzz replay on canonical and on the candidate, after the repair, and in either order with
   the glyph-spacing patch and the chapter 11 storyboard patch. A control proves a moved context
   line refuses.
2. The storyboard's words and hold columns are unchanged, cue for cue, and every row keeps its cells.
3. No TEX supplement in mod.rs after the patch names a constant missing from the staged source.
4. Existing test literals: each string a canonical test in the four files, mod.rs, draw.rs or main.rs
   matches against source, which the staged source no longer holds.

Text only. It cannot see types, and it runs no test.
"""
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
O = HERE.parents[1]
REL = Path("explainer_corpus/complex_analysis_visual")
CANDIDATE = Path("narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/combined-picture-stage01/candidate") / REL
STAGED = O / "complex02-05-repair03" / REL / "src/chapters"
NAMES = ("turning", "two_planes", "reading_color", "gallery")
MINE = HERE.parent / "rebinding-tex-storyboard.patch"
GLYPH = O / "complex-glyph-spacing-repair01/complex-glyph-spacing.patch"
BOARD_11 = O / "complex11-counting-review-repair01/patches/02-storyboard-11_7.patch"
failures = []


def check(ok, text):
    print(("ok   " if ok else "FAIL ") + text)
    if not ok:
        failures.append(text)


def copy(base):
    tmp = Path(tempfile.mkdtemp(prefix="cross04-order-"))
    shutil.copytree(base, tmp / REL, ignore=shutil.ignore_patterns("target"))
    return tmp


def apply(tmp, patch):
    head = patch.read_text(encoding="utf-8")
    strip = 1 if "a/explainer_corpus/" in head else 0
    args = ["git", "apply", "--whitespace=nowarn", "-C3", f"-p{strip}"]
    if strip == 0 and "--- explainer_corpus/" not in head:
        args = ["git", "apply", "--whitespace=nowarn", "-p1"]
    done = subprocess.run(args + [str(patch.resolve())], cwd=tmp, capture_output=True, text=True)
    return done.returncode == 0, done.stderr[:240]


def sequence(base, patches, label):
    tmp = copy(base)
    for patch in patches:
        ok, why = apply(tmp, patch)
        if not ok:
            check(False, f"{label}: {patch.name} refused: {why}")
            return None
    check(True, label)
    return tmp


def rows(text):
    out = {}
    for line in text.split("\n"):
        m = re.match(r"\| (\d\d_\d+) \|", line)
        if m:
            cells = re.split(r"(?<!\\)\|", line)[1:-1]
            out[m.group(1)] = [c.strip() for c in cells]
    return out


def main():
    results = {}
    for label, base in (("canonical", REL), ("candidate", CANDIDATE)):
        results[label] = sequence(base, [MINE], f"{label}: this patch applies with zero fuzz")
        sequence(base, [MINE, GLYPH], f"{label}: this patch, then glyph spacing")
        sequence(base, [GLYPH, MINE], f"{label}: glyph spacing, then this patch")
        sequence(base, [MINE, BOARD_11], f"{label}: this patch, then the 11_7 storyboard patch")
        sequence(base, [BOARD_11, MINE], f"{label}: the 11_7 storyboard patch, then this patch")
    tmp = copy(REL)
    board = tmp / REL / "docs/storyboard.md"
    board.write_text(board.read_text(encoding="utf-8").replace("drawn with a fold tick at the origin", "drawn with a fold mark at the origin"), encoding="utf-8")
    ok, _ = apply(tmp, MINE)
    check(not ok, "control: a changed line refuses this patch")
    before = rows((REL / "docs/storyboard.md").read_text(encoding="utf-8"))
    after = rows((results["canonical"] / REL / "docs/storyboard.md").read_text(encoding="utf-8"))
    check(list(before) == list(after), f"storyboard: same {len(before)} rows in the same order")
    changed = []
    for cue, old in before.items():
        new = after[cue]
        if len(old) != len(new) or len(new) != 5:
            check(False, f"{cue}: {len(old)} cells became {len(new)}")
            continue
        if old[0] != new[0] or old[1] != new[1] or old[3] != new[3] or old[4] != new[4]:
            check(False, f"{cue}: a column other than the picture changed")
        if old[2] != new[2]:
            changed.append(cue)
    print("     picture cells changed:", " ".join(changed))
    check(changed == "02_5 02_6 03_2 03_4 03_5 04_3 04_4 04_5 05_2 05_3 05_7 05_8 05_9".split(), "storyboard: exactly the 13 planned picture cells changed; words, features and holds did not")
    mod = (results["canonical"] / REL / "src/chapters/mod.rs").read_text(encoding="utf-8")
    source = (STAGED / "two_planes.rs").read_text(encoding="utf-8")
    for name in ("RULE_LENGTH_TEX", "RULE_ANGLE_TEX"):
        check(f"two_planes::{name}[0]" in mod and f"two_planes::{name}[1]" in mod and f"pub const {name}: [&str; 2]" in source, f"TEX names {name}, which the staged two_planes.rs defines")
    # 4: literals in tests that look like source being read
    literal = re.compile(r'(?:contains|find|index|matches|starts_with|ends_with|split)\(\s*r?#?"((?:[^"\\]|\\.)*)"')
    staged_all = {n: (STAGED / f"{n}.rs").read_text(encoding="utf-8") for n in NAMES}
    joined = "\n".join(staged_all.values())
    src = REL / "src"
    hits = []
    for path in [src / "chapters" / f"{n}.rs" for n in NAMES] + [src / "chapters/mod.rs", src / "chapters/draw.rs", src / "main.rs"]:
        text = path.read_text(encoding="utf-8")
        at = text.find("#[cfg(test)]")
        if at < 0:
            continue
        for m in literal.finditer(text, at):
            value = m.group(1).replace('\\"', '"').replace("\\n", "\n")
            if len(value) < 6:
                continue
            canon = "\n".join((src / "chapters" / f"{n}.rs").read_text(encoding="utf-8") for n in NAMES)
            if value in canon and value not in joined:
                hits.append((path.name, text.count("\n", 0, m.start()) + 1, value))
    print(f"     test literals matched in canonical's four files but gone from the staged ones: {len(hits)}")
    for hit in hits:
        print("      ", hit)
    print(f"{len(failures)} failures")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
