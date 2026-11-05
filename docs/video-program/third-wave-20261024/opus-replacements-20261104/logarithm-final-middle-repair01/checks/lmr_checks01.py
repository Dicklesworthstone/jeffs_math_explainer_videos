"""Scratch: read-only checks for the staged repair. Run from the repo root.

rustfmt runs with --check, so it writes nothing. Nothing is compiled.
Usage: lmr_checks01.py BASE STAGED WS V FONT
"""
import json
import os
import re
import subprocess
import sys

import numpy as np
from PIL import Image, ImageFont

BASE, STAGED, WS, V, FONT = sys.argv[1:6]
FILES = ["draw.rs", "complex_turns.rs", "lifting_loops.rs"]

print("== rustfmt --check, 2024 edition, repo rustfmt.toml")
for label, root in (("base", BASE), ("staged", STAGED)):
    for f in FILES:
        path = os.path.join(root, "src/chapters", f)
        r = subprocess.run(["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", path],
                           capture_output=True, text=True)
        hunks = r.stdout.count("Diff in ")
        print("%-6s %-18s exit %d, %d diff hunks%s" % (label, f, r.returncode, hunks, ", stderr: " + r.stderr[:160] if r.stderr else ""))
        if r.returncode and label == "staged":
            lines = [x for x in r.stdout.splitlines() if x.startswith("Diff in ")]
            print("   ", [re.sub(r".*:(\d+):?$", r"\1", x) for x in lines][:12])

print("== name collisions in the frozen crate")
names = 0
for dirpath, _, files in os.walk(os.path.join(WS, "src")):
    for f in files:
        if not f.endswith(".rs"):
            continue
        s = open(os.path.join(dirpath, f), encoding="utf-8").read()
        for m in re.finditer(r"\b(pause_line|PAUSE_[A-Z]+)\b|\blet pause\b", s):
            names += 1
            print("  ", f, m.group(0))
print("hits:", names)

print("== rounded values printed with = in chapters 05-08")
for f in ("complex_turns.rs", "lifting_loops.rs", "branch_cut.rs", "product_rule.rs"):
    for i, line in enumerate(open(os.path.join(WS, "src/chapters", f), encoding="utf-8"), 1):
        if re.search(r'"[^"]*= \{[^"]*"', line):
            print("  %s:%d %s" % (f, i, line.strip()[:110]))

print("== hash pins of the three frozen files inside the crate")
pins = {"draw.rs": "2a4e84504bab", "complex_turns.rs": "93f663d372ab", "lifting_loops.rs": "01885b9f558c"}
found = 0
for dirpath, _, files in os.walk(WS):
    for f in files:
        p = os.path.join(dirpath, f)
        try:
            s = open(p, encoding="utf-8").read()
        except (UnicodeDecodeError, OSError):
            continue
        for name, h in pins.items():
            if h in s:
                found += 1
                print("  ", os.path.relpath(p, WS), name)
print("pins found:", found)
for n in ("15", "16"):
    s = open(os.path.join(WS, "docs/narration-direction-map-oliver%s.json" % n), encoding="utf-8").read()
    print("oliver%s mentions:" % n, [m.group(0)[:70] for m in re.finditer(r"[^\"]{0,40}(?:draw|complex_turns|lifting_loops)\.rs[^\"]{0,20}", s)][:3])

print("== glyph widths")
em = ImageFont.truetype(FONT, 1000)
print("= %.0f, ≈ %.0f; label ems %.3f against %.3f" % (
    em.getlength("="), em.getlength("≈"),
    em.getlength("real part ln 2 = 0.693") / 1000, em.getlength("real part ln 2 ≈ 0.693") / 1000))

d = json.load(open(os.path.join(V, "verification.json")))
print("== recorded pictures showing the blue 05_7 label (box 240..400, 152..172)")
shown = []
for i in range(545, 585):
    f = np.asarray(Image.open(os.path.join(V, "frame-%04d.png" % (i + 1))).convert("RGB")).astype(int)
    roi = f[152:172, 240:400]
    blue = int(((roi[..., 2] > 170) & (roi[..., 0] < 120)).sum())
    if blue:
        shown.append((i + 1, d["pictures"][i]["frame"], round(d["pictures"][i]["seconds"], 2), blue))
print(shown)

print("== recorded pictures around 06_2 and 06_3")
for n in range(590, 607):
    p = d["pictures"][n - 1]
    print("%04d f%d %.2f %s" % (n, p["frame"], p["seconds"], p["reasons"]))
for cue in ("06_2", "06_3"):
    x = [q for q in d["placements"] if q["cue"] == cue][0]
    print(cue, "start", x["start_frame"], "%.2f" % (x["start_frame"] / 60), "duration", x["duration_s"])
