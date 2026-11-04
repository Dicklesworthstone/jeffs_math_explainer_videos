#!/usr/bin/env python3
"""The staged storyboard against canonical's, in isolated roots: picture cells only may differ.

tools/storyboard.py --check already fails on canonical (three chapters with no prediction pause,
and a stale script report), so its exit code proves nothing here. This asserts instead that the
staged storyboard reports exactly the same problems, that every words column is identical, and
that only the rows named below changed.

    python3 -I -B storyboard_compare.py EMPTY_SCRATCH      (from the repo root)
"""
import os
import re
import shutil
import subprocess
import sys

S = sys.argv[1]
HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-picture-repair-06-10/"
CANON = "explainer_corpus/complex_analysis_visual/"
CHANGED = {"06_3", "06_7", "06_8", "08_2", "09_7", "09_8", "09_9", "10_4", "10_6"}
results = {}
for tag, sb in (("canonical", CANON + "docs/storyboard.md"), ("staged", HERE + CANON + "docs/storyboard.md")):
    iso = f"{S}/{tag}"
    os.makedirs(iso + "/tools", exist_ok=True)
    os.makedirs(iso + "/" + CANON + "docs", exist_ok=True)
    shutil.copyfile("tools/storyboard.py", iso + "/tools/storyboard.py")
    shutil.copyfile(sb, iso + "/" + CANON + "docs/storyboard.md")
    shutil.copyfile(CANON + "script.tsv", iso + "/" + CANON + "script.tsv")
    r = subprocess.run(
        ["python3", "-I", "-B", iso + "/tools/storyboard.py", "--check", "complex_analysis_visual"],
        capture_output=True,
        text=True,
    )
    results[tag] = [line.strip() for line in (r.stdout + r.stderr).splitlines() if "PROBLEM" in line]
    print(tag, "exit", r.returncode, "problems", len(results[tag]))
for line in results["canonical"]:
    print("  already on canonical:", line)
ok = results["canonical"] == results["staged"]
print("same gate problems as canonical:", ok)


def rows(path):
    out = {}
    for line in open(path, encoding="utf-8"):
        m = re.match(r"\| (\d\d_\d+) \| (.*?) \| (.*)$", line.rstrip("\n"))
        if m:
            out[m.group(1)] = (m.group(2), m.group(3))
    return out


a = rows(CANON + "docs/storyboard.md")
b = rows(HERE + CANON + "docs/storyboard.md")
words_equal = {k: v[0] for k, v in a.items()} == {k: v[0] for k, v in b.items()}
print("rows", len(a), "words columns identical:", words_equal)
changed = {k for k in a if a[k][1] != b[k][1]}
print("picture cells changed:", sorted(changed))
ok = ok and words_equal and changed == CHANGED
sys.exit(0 if ok else 1)
