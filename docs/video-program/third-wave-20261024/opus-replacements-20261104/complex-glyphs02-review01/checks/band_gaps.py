"""Find the blank gap that separates rows, and where 18 px split a row in two.

Run from the repo root with python3 -B. Reads only.
"""
import collections
import sys
from pathlib import Path

sys.path.insert(0, str(Path(sys.argv[0]).resolve().parent))
import page_map as pm

expected = [r for r in pm.probe_rows(pm.registry()) if r[4] is not None]
print("expected ink rows", len(expected))
files = sorted(pm.PAGES.glob("frame_*.png"))
for gap in (18, 22, 25, 27, 29, 31, 34):
    pm.GAP = gap
    total = 0
    for path in files:
        found, _ = pm.bands(path)
        total += len([b for b in found if b["bottom"] >= pm.HEADING_BOTTOM])
    print(f"gap {gap}: {total} bands")

pm.GAP = 1
hist = collections.Counter()
for path in files:
    found, _ = pm.bands(path)
    tops = [b for b in found if b["bottom"] >= pm.HEADING_BOTTOM]
    for a, b in zip(tops, tops[1:]):
        hist[b["top"] - a["bottom"]] += 1
print("blank gaps between ink runs, px: count")
for g in sorted(hist):
    print(f"  {g:3d}: {hist[g]}")
