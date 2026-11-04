"""Run the staged text-only checkers on scratch overlays of baseline and a candidate.

Run from the repo root with python3 -I -B. Everything runs in scratch copies; the first
argument names the candidate applications.rs, later ones add name=path overlays.
"""
import difflib
import glob
import os
import pathlib
import shutil
import subprocess
import sys
import time

SP = pathlib.Path(sys.argv[0]).resolve().parent
REPO = pathlib.Path.cwd()
CRATE = REPO / "explainer_corpus/abstract_algebra_expanded"
K = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex/checks"
KIT = REPO / "crates/explainer_kit/src"
FMN = glob.glob(os.path.expanduser("~/.cargo/git/checkouts/franken_manim-*/a399d62/crates"))[0]
work = SP / f"desk-{time.strftime('%H%M%S')}"
work.mkdir()


def overlay(name, source):
    d = work / name
    shutil.copytree(CRATE / "src", d / "src")
    (d / "docs").mkdir()
    shutil.copyfile(CRATE / "script.tsv", d / "script.tsv")
    shutil.copyfile(CRATE / "oracle.py", d / "oracle.py")
    if source:
        shutil.copyfile(source, d / "src/applications.rs")
    return d


trees = {"baseline": overlay("baseline", None)}
for arg in sys.argv[1:]:
    name, path = arg.split("=", 1)
    trees[name] = overlay(name, path)
py = [sys.executable, "-I", "-B"]
tools = [
    ("arity_check", [str(K / "arity_check.py"), str(KIT), FMN]),
    ("scope_check", [str(K / "scope_check.py"), str(KIT), FMN]),
    ("shape_check", [str(K / "shape_check.py"), str(KIT), FMN]),
    ("unused_check", [str(K / "unused_check.py")]),
    ("desk_check", [str(K / "desk_check.py")]),
]
results = {}
for tree, d in trees.items():
    for tool, args in tools:
        r = subprocess.run(py + args, cwd=d, capture_output=True, text=True)
        results[tree, tool] = (r.returncode, (r.stdout + r.stderr).replace(str(d), "<crate>").splitlines())
for tool, _ in tools:
    code0, base = results["baseline", tool]
    print(f"--- {tool}: baseline exit {code0}, {len(base)} lines; last: {base[-1][:150] if base else ''}")
    for tree in trees:
        if tree == "baseline":
            continue
        code1, cand = results[tree, tool]
        new = [l for l in difflib.ndiff(base, cand) if l[:2] in ("+ ", "- ")]
        print(f"    {tree}: exit {code1}; {len(cand)} lines; {len(new)} changed")
        for l in new[:40]:
            print("       ", l[:200])
