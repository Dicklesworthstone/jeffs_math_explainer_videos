#!/usr/bin/env python3
"""Apply the follow-up patch to scratch trees, in every order root might use.

Reads canonical files and the frozen patches, writes only under SCRATCH_DIR.
Every apply uses --fuzz=0. Offsets are printed, because they are not errors.

Run from the repository root: python3 -I -B <this file> SCRATCH_DIR
"""

import re
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parents[1]
ALGEBRA = HERE.parent
REPO = HERE.parents[5]
CANONICAL = REPO / "explainer_corpus/abstract_algebra_expanded"
PREFIX = "explainer_corpus/abstract_algebra_expanded"
FOLLOWUP = HERE / "chapter11-followup.patch"
REPAIR = ALGEBRA / "permutation-repair01/chapter11-teaching-repair.patch"
P17 = ALGEBRA / "patches/abstract_algebra_expanded-17-followup.patch"
P13 = ALGEBRA / "patches/abstract_algebra_expanded-13-followup.patch"
P10 = ALGEBRA / "patches/abstract_algebra_expanded-10-cyclic.patch"
STAGED = ["src/permutations.rs", "src/main.rs", "oracle.py", "script.tsv"]


def new_tree(root, name):
    tree = root / name
    if tree.exists():
        sys.exit(f"{tree} exists; pass an empty scratch directory")
    target = tree / PREFIX
    (target / "src").mkdir(parents=True)
    for path in (CANONICAL / "src").glob("*"):
        shutil.copy2(path, target / "src")
    for name_of_file in ("oracle.py", "script.tsv"):
        shutil.copy2(CANONICAL / name_of_file, target)
    return tree


def apply(tree, patch, *flags):
    done = subprocess.run(
        ["patch", "-d", str(tree), "-p1", "--fuzz=0", "--no-backup-if-mismatch", "-i", str(patch), *flags],
        capture_output=True,
        text=True,
    )
    if done.returncode:
        sys.exit(f"{patch.name} failed on {tree.name}:\n{done.stdout}{done.stderr}")
    offsets = re.findall(r"Hunk #\d+ succeeded at \d+ (?:with fuzz \d+ )?\(offset -?\d+ lines?\)", done.stdout)
    if "fuzz" in done.stdout:
        sys.exit(f"{patch.name} needed fuzz on {tree.name}")
    return len(offsets)


def snapshot(tree):
    root = tree / PREFIX
    return {str(path.relative_to(root)): path.read_bytes() for path in sorted(root.rglob("*")) if path.is_file()}


def oracle_lines(tree):
    done = subprocess.run(
        [sys.executable, "-I", "-B", str(tree / PREFIX / "oracle.py")], capture_output=True, text=True, check=True
    )
    lines = done.stdout.splitlines()
    if len({line.split("\t")[0] for line in lines}) != len(lines):
        sys.exit(f"{tree.name}: the oracle prints a key twice")
    return len(lines)


def main():
    root = Path(sys.argv[1])
    root.mkdir(parents=True, exist_ok=True)
    candidate = {name: (HERE / "candidate" / PREFIX / name).read_bytes() for name in STAGED}
    base = {
        "src/permutations.rs": HERE / "base/permutations.rs.base",
        "src/main.rs": HERE / "base/main.rs.base",
        "oracle.py": HERE / "base/oracle.py.base",
        "script.tsv": HERE / "base/script.tsv.base",
    }

    # 1. Base to candidate and back.
    tree = root / "base-only"
    (tree / PREFIX / "src").mkdir(parents=True)
    for name, path in base.items():
        shutil.copy2(path, tree / PREFIX / name)
    apply(tree, FOLLOWUP)
    same = all((tree / PREFIX / name).read_bytes() == candidate[name] for name in STAGED)
    print(f"base + follow-up equals the candidate, all four files: {same}")
    apply(tree, FOLLOWUP, "-R")
    back = all((tree / PREFIX / name).read_bytes() == path.read_bytes() for name, path in base.items())
    print(f"reversed, equals the base: {back}")

    # 2. Canonical, repair01, follow-up.
    plain = new_tree(root, "repair-then-followup")
    apply(plain, REPAIR)
    offset = apply(plain, FOLLOWUP)
    equal = all((plain / PREFIX / name).read_bytes() == candidate[name] for name in STAGED)
    print(f"canonical + repair01 + follow-up equals the candidate: {equal}; {offset} offset hunks")

    # 3. The three frozen patches, around it, in three orders.
    late = new_tree(root, "chain-then-followup")
    for patch in (REPAIR, P17, P13, P10):
        apply(late, patch)
    offset_late = apply(late, FOLLOWUP)
    early = new_tree(root, "followup-then-chain")
    for patch in (REPAIR, FOLLOWUP, P17, P13, P10):
        apply(early, patch)
    first = new_tree(root, "chain-then-repair")
    for patch in (P17, P13, P10, REPAIR, FOLLOWUP):
        apply(first, patch)
    late_bytes, early_bytes, first_bytes = snapshot(late), snapshot(early), snapshot(first)
    print(f"repair01, 17, 13, 10, follow-up applies at fuzz 0; {offset_late} offset hunks")
    print(f"repair01, follow-up, 17, 13, 10 gives identical bytes: {late_bytes == early_bytes}")
    print(f"17, 13, 10, repair01, follow-up gives identical bytes: {late_bytes == first_bytes}")
    print(f"oracle on the full chain: {oracle_lines(late)} lines, no key twice")
    print(f"oracle on canonical + repair01 + follow-up: {oracle_lines(plain)} lines, no key twice")
    if not (same and back and equal and late_bytes == early_bytes == first_bytes):
        sys.exit(1)


main()
