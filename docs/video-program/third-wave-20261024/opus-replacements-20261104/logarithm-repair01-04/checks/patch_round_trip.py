#!/usr/bin/env python3
"""Write the patch from base/ to candidate/, then prove it applies and reverses.

The patch names the four chapter files with a/ and b/ prefixes relative to the repo
root. With --fuzz=0 it must take base/ to candidate/ exactly, and back. The same forward
apply is then tried on a scratch copy of the current canonical files. If canonical has
moved, that step reports which files differ rather than failing the base round trip:
root edits canonical between syncs, and `git merge-file` can then do a three-way merge
from these base/ and candidate/ copies.

Usage: python3 -I -B checks/patch_round_trip.py SCRATCH_DIR [--write]
--write refreshes logarithm-repair01-04.patch. Without it the file on disk must match
what this script would write. Writes nothing else outside SCRATCH_DIR.
"""

import difflib
import hashlib
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPO = ROOT.parents[4]
PATCH = ROOT / "logarithm-repair01-04.patch"
PREFIX = "explainer_corpus/logarithm_mysteries"
CHAPTERS = [f"src/chapters/{c}.rs" for c in ("building_ruler", "slide_rule", "folded_decade", "undoing_growth")]


def patch_text():
    pieces = []
    for relative in CHAPTERS:
        name = f"{PREFIX}/{relative}"
        before = (ROOT / "base" / name).read_text().splitlines(keepends=True)
        after = (ROOT / "candidate" / name).read_text().splitlines(keepends=True)
        pieces.extend(difflib.unified_diff(before, after, f"a/{name}", f"b/{name}", n=3))
    return "".join(pieces)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def seed(tree, source):
    for relative in CHAPTERS:
        target = tree / PREFIX / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source / PREFIX / relative, target)


def apply(tree, *flags):
    return subprocess.run(
        ["patch", "-p1", "--fuzz=0", "--no-backup-if-mismatch", *flags, "-i", str(PATCH)],
        cwd=tree, capture_output=True, text=True,
    )


def same(tree, source):
    return [r for r in CHAPTERS if sha(tree / PREFIX / r) != sha(source / PREFIX / r)]


def main():
    out = Path(sys.argv[1]).resolve()
    text = patch_text()
    if "--write" in sys.argv:
        PATCH.write_text(text)
    elif PATCH.read_text() != text:
        sys.exit("the patch on disk is not the diff of base/ and candidate/")
    print(f"patch: {len(text.splitlines())} lines, sha256 {sha(PATCH)}")
    trouble = 0

    tree = out / "round_trip"
    if tree.exists():
        shutil.rmtree(tree)
    seed(tree, ROOT / "base")
    forward = apply(tree)
    wrong = same(tree, ROOT / "candidate")
    print(f"base + patch = candidate: {'yes' if forward.returncode == 0 and not wrong else 'NO ' + str(wrong)}")
    trouble += bool(forward.returncode or wrong)
    backward = apply(tree, "-R")
    wrong = same(tree, ROOT / "base")
    print(f"candidate - patch = base: {'yes' if backward.returncode == 0 and not wrong else 'NO ' + str(wrong)}")
    trouble += bool(backward.returncode or wrong)

    moved = same(REPO, ROOT / "base")
    support = ["script.tsv", "oracle.py", "src/facts.rs", "src/model.rs", "src/chapters/draw.rs"]
    moved_support = [r for r in support if sha(REPO / PREFIX / r) != sha(ROOT / "base" / PREFIX / r)]
    print(f"canonical chapters equal to base/: {'all four' if not moved else 'NO, moved: ' + str(moved)}")
    print(f"canonical support files equal to base/: {'all five' if not moved_support else 'NO, moved: ' + str(moved_support)}")
    tree = out / "on_canonical"
    if tree.exists():
        shutil.rmtree(tree)
    seed(tree, REPO)
    onto = apply(tree)
    wrong = same(tree, ROOT / "candidate")
    print(f"canonical + patch = candidate: {'yes' if onto.returncode == 0 and not wrong else 'NO ' + str(wrong)}")
    trouble += bool(onto.returncode or wrong)
    for relative in CHAPTERS:
        print(f"  {sha(ROOT / 'base' / PREFIX / relative)}  base {relative}")
        print(f"  {sha(ROOT / 'candidate' / PREFIX / relative)}  candidate {relative}")
    sys.exit(1 if trouble else 0)


main()
