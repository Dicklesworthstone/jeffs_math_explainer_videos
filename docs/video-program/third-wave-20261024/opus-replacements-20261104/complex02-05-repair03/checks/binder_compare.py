"""Run the existing Oliver binder's `build` on canonical and on canonical + this repair.

Reads only. Nothing is written by the binder: `build` is called directly and its `main`, which
writes, is not. The binder already refuses on canonical for problems this repair did not make,
so the question asked here is narrower: does the repair add a problem, and what hold rows does
the binder itself derive from the repaired source?

Usage: python3 -I -B binder_compare.py REPO BINDER_PY STAGED_CHAPTERS OVERLAY_DIR OUT_JSON
OVERLAY_DIR is a scratch copy of the crate into which STAGED_CHAPTERS are copied.
"""

import importlib.util
import json
import re
import shutil
import sys
from pathlib import Path

repo, binder, staged, overlay, out = (Path(a) for a in sys.argv[1:6])
crate = repo / "explainer_corpus/complex_analysis_visual"
names = ("turning", "two_planes", "reading_color", "gallery")
for name in names:
    shutil.copyfile(staged / f"{name}.rs", overlay / "src/chapters" / f"{name}.rs")

spec = importlib.util.spec_from_file_location("bind_oliver_plan", binder)
module = importlib.util.module_from_spec(spec)
sys.dont_write_bytecode = True
spec.loader.exec_module(module)

VERSION = "repair03-compare"


def run(root):
    plan, outputs, _audit, _canonical_map, map_text = module.build(str(repo), str(root), VERSION)
    return plan.problems, json.loads(map_text)


def mask(text):
    return re.sub(r"\.rs:\d+", ".rs:N", text)


before, map_before = run(crate)
after, map_after = run(overlay)
new = sorted(set(map(mask, after)) - set(map(mask, before)))
gone = sorted(set(map(mask, before)) - set(map(mask, after)))
print(f"canonical problems: {len(before)}; with this repair: {len(after)}")
print(f"added by the repair: {len(new)}; gone with the repair: {len(gone)}")
for text in new:
    print("  ADDED", text)
for text in gone:
    print("  GONE ", text)
print("problems on canonical (not this repair's):")
for text in before:
    print("  ", text)


def rows(m):
    return {r["cue"]: r for r in m["visual_holds"]}


a, b = rows(map_before), rows(map_after)
changed = {}
for cue in sorted(set(a) | set(b)):
    if a.get(cue) != b.get(cue):
        changed[cue] = {"before": a.get(cue), "after": b.get(cue)}
print(f"hold rows the binder derives differently: {sorted(changed)}")
for cue, pair in changed.items():
    print(f"  {cue}: line {pair['before']['line']} -> {pair['after']['line']}; "
          f"marker {'same' if pair['before']['marker'] == pair['after']['marker'] else 'CHANGED'}")
out.write_text(json.dumps(changed, indent=1, ensure_ascii=False) + "\n", encoding="utf8")
same = {k: map_before.get(k) == map_after.get(k) for k in map_after if k != "visual_holds"}
print("other sections unchanged:", {k: v for k, v in same.items()})
