"""Positive control for timing-compare03.py: write a NEW scratch variant of the candidate with one
fade time changed (0.5 -> 0.6 on the quarter picture), refusing to overwrite.

usage: python3 -I -B timing-control04.py CANDIDATE.rs OUT_VARIANT.rs
"""
import sys

src = open(sys.argv[1], encoding="utf-8").read()
old = "fade_in_all(stage, &quarter_picture, 0.5)?;"
assert src.count(old) == 1
with open(sys.argv[2], "x", encoding="utf-8") as f:
    f.write(src.replace(old, "fade_in_all(stage, &quarter_picture, 0.6)?;"))
print("wrote", sys.argv[2])
