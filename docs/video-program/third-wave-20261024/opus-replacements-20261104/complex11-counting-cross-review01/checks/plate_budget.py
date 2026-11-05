"""What the turn asks of the geometry budget under each plate rung. Arithmetic only: no engine ran.

Formulas are LavenderPike's and RubyRaven's, checked first against the numbers they published. The
strip formula for flat cells, 4n + 9, is inferred here from "about 13%", so `now`'s row is the
weakest. The factor of two for a repaint is RubyRaven's reading of compaction, unobserved. The factor
of three is this script's worst case, if the painter's rows are still retained at the first repaint:
also unobserved.

Usage: python3 -I -B plate_budget.py
"""

import sys

LIMIT = 1 << 20
CELL = 0.035
RUNGS = {
    "now": (CELL, CELL, False),
    "tile": (CELL, CELL, False),
    "blend": (CELL, CELL, True),
    "rows2": (CELL / 2, CELL, True),
    "cells2": (CELL / 2, CELL / 2, True),
    "cells4": (CELL / 4, CELL / 4, True),
}


def plate(width, rung):
    row, col, blended = RUNGS[rung]
    rows, cols = round(width / row), round(width / col)
    per_strip = 2 * cols + 9 if blended else 4 * cols + 9
    return rows, cols, rows * per_strip


failed = 0
for label, got, want, source in (
    ("6.4 plate at cells4", plate(6.4, "cells4")[2], 1_075_301, "Mail 2921 / comparison REVIEW"),
    ("6.4 plate at cells2", plate(6.4, "cells2")[2], 271_206, "comparison REVIEW"),
    ("5.6 plate at cells4", plate(5.6, "cells4")[2], 824_960, "Mail 2916, sin(1/z)"),
):
    ok = got == want
    failed += 0 if ok else 1
    print(f"{'PASS' if ok else 'FAIL'} control {label}: {got:,}, published {want:,} ({source})")
for label, got, share in (("6.4 plate at blend", plate(6.4, "blend")[2], 6.5), ("6.4 plate at now", plate(6.4, "now")[2], 13.0)):
    ok = abs(100 * got / LIMIT - share) < 0.5
    failed += 0 if ok else 1
    print(f"{'PASS' if ok else 'FAIL'} control {label}: {100 * got / LIMIT:.1f}% of the limit, published about {share}%")

print()
print("the turn's plate is 5.6 wide. Knots as a share of 1<<20, then of 1<<22:")
print("rung\trows x cols\tcells a repaint\tmodel evals for 96 repaints\tone plate\ttakeover (painter + field)\trepaint (x2)\tworst (x3)\tworst of 1<<22")
for rung in RUNGS:
    rows, cols, knots = plate(5.6, rung)
    cells = rows * cols
    print(
        f"{rung}\t{rows} x {cols}\t{cells:,}\t{96 * cells:,}\t{knots:,} ({100 * knots / LIMIT:.1f}%)"
        f"\t{2 * knots:,} ({200 * knots / LIMIT:.1f}%)\t{2 * knots:,} ({200 * knots / LIMIT:.1f}%)\t{3 * knots:,} ({300 * knots / LIMIT:.1f}%)\t{300 * knots / (4 * LIMIT):.1f}%"
    )
print()
print("The repaint count is an upper bound: 48 steps out and 48 back, each repainted at most once.")
print("Against the painter's crossfade: 12 steps, each of which paints two functions.")
sys.exit(1 if failed else 0)
