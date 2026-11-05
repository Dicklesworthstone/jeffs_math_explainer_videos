"""The tables REVIEW.md quotes: cost from the receipts, stairs from the saved measure, the centre cell, the knot budget, and exact cells on zeros.

    python3 -B .../checks/tables.py VIEW     (VIEW from consolidate.py, from the repo root)

Seconds are one debug-build run of each scene at 720p, so they rank and they bound nothing at 4K.
"""
import csv
import math
import pathlib
import re
import statistics
import sys

import numpy as np
from PIL import Image

HERE = pathlib.Path(__file__).resolve().parent
LIMIT = 1 << 20
FAILED_AT = 1048823
CELL = 0.035
# Every plate a Painter builds, from the source. Each function's exact zeros and poles a cell centre could equal in f64.
AT = {
    "Identity": [0], "Square": [0], "Recip": [0], "Bump": [1j, -1j], "Cubic": [1], "ZeroDoublePole": [1, -1], "Exp": [],
    "Sin": [0], "Tan": [0], "Gamma": [0, -1, -2, -3, -4, -5, -6, -7, -8], "Zeta": [1, -2, -4, -6, -8, -10, -12, -14, -16],
    "Moebius": [0.45, 1.35], "Quintic": [], "AbsSquare": [0], "SinRecip": [0], "Lacunary": [0],
}
ZOOM = ((-0.03617800000000004, 1.263822), (0.41317100000000007, 1.713171))  # plates.tsv, zoom
PAINTED = (
    ("reading_color plane", 6.4, (-2.0, 2.0), (-2.0, 2.0), "Identity Square Recip Bump"),
    ("counting planes.0", 5.6, (-2.0, 2.0), (-2.0, 2.0), "Identity ZeroDoublePole Moebius Quintic"),
    ("gallery square_plane", 5.6, (-2.0, 2.0), (-2.0, 2.0), "Cubic ZeroDoublePole"),
    ("gallery exp_plane", 3.2, (-3.0, 3.0), (-4.0, 8.0), "Exp"),
    ("gallery trig_plane", 8.4, (-6.6, 6.6), (-3.3, 3.3), "Sin Tan"),
    ("gallery gamma_plane", 8.4, (-4.5, 4.5), (-2.5, 2.5), "Gamma"),
    ("gallery zeta_plane", 6.4, (-7.0, 3.0), (-3.0, 3.0), "Zeta"),
    ("gallery zoom_plane", 5.0, ZOOM[0], ZOOM[1], "Cubic"),
    ("mosaic Cubic", 4.2, (-2.7, 2.7), (0.3 - 5.4 * 2.6 / 4.2 / 2.0, 0.3 + 5.4 * 2.6 / 4.2 / 2.0), "Cubic"),
    ("mosaic ZeroDoublePole", 4.2, (-2.7, 2.7), (-(5.4 * 2.6 / 4.2 / 2.0), 5.4 * 2.6 / 4.2 / 2.0), "ZeroDoublePole"),
    ("mosaic Exp", 4.2, (-3.5, 3.5), (1.2 - 7.0 * 2.6 / 4.2 / 2.0, 1.2 + 7.0 * 2.6 / 4.2 / 2.0), "Exp"),
    ("mosaic Sin", 4.2, (-6.6, 6.6), (-(13.2 * 2.6 / 4.2 / 2.0), 13.2 * 2.6 / 4.2 / 2.0), "Sin"),
    ("mosaic Gamma", 4.2, (-4.5, 3.5), (-(8.0 * 2.6 / 4.2 / 2.0), 8.0 * 2.6 / 4.2 / 2.0), "Gamma"),
    ("mosaic Zeta", 4.2, (-7.0, 3.0), (-(10.0 * 2.6 / 4.2 / 2.0), 10.0 * 2.6 / 4.2 / 2.0), "Zeta"),
    ("continuing bump_plane", 5.6, (-2.5, 2.5), (-2.5, 2.5), "Bump"),
    ("continuing lacunary_plane", 5.4, (-1.05, 1.05), (-1.05, 1.05), "Lacunary"),
    ("continuing gamma_plane", 7.5, (-4.8, 3.0), (-2.0, 2.0), "Gamma"),
    ("boundary picture", 5.6, (-2.5, 2.5), (-2.5, 2.5), "Bump"),
    ("residues small_planes", 2.8, (-1.2, 1.2), (-1.2, 1.2), "Identity Square Recip"),
    ("residues bump_plane", 5.0, (-2.0, 2.0), (-2.0, 2.0), "Bump"),
    ("rigidity sin_recip_plane", 5.6, (-0.42, 0.42), (-0.42, 0.42), "SinRecip"),
    ("rigidity sine_plane", 6.0, (-4.0, 4.0), (-3.5, 3.5), "Sin"),
    ("one_point painted", 5.4, (-2.0, 2.0), (-2.0, 2.0), "AbsSquare"),
)


def luma(a):
    return a[..., 0] * 0.2126 + a[..., 1] * 0.7152 + a[..., 2] * 0.0722


def knots(rows, cols):
    """A blended strip's knots: its 2n + 11 records less the middle and one anchor of each of its two null curves."""
    return rows * (2 * cols + 9)


def main(view):
    view = pathlib.Path(view)
    plates = {r["subject"]: r for r in csv.DictReader(open(view / "plates.tsv"), delimiter="\t")}
    rows = list(csv.DictReader(open(view / "receipt.tsv"), delimiter="\t"))
    by = {(r["kind"], r["subject"], r["candidate"]): float(r["s_per_frame"]) for r in rows}

    print("1. Seconds a frame against the same scene on `now`. Median, lowest, highest, over the subjects that have both. One debug run each, 720p.")
    for kind, names in (("stills", ("overlap06", "tile", "tile_p3", "blend", "rows2", "cells2")), ("fades", ("tile", "blend")), ("repaints", ("blend", "rows2"))):
        for name in names:
            ratios = [v / by[(kind, s, "now")] for (k, s, c), v in by.items() if k == kind and c == name and (kind, s, "now") in by]
            print(f"   {kind:9s}{name:10s} {statistics.median(ratios):5.2f}  {min(ratios):5.2f}  {max(ratios):5.2f}   n={len(ratios)}")

    print("\n2. Against truth, rms after a 2x2 box, from checks/measure_lab.out section C. Each candidate as a share of the same subject's tile.")
    text = (HERE / "measure_lab.out").read_text()
    section = text[text.index("C. Against truth") : text.index("D. Fades")]
    table = {}
    subject = None
    for line in section.splitlines():
        m = re.match(r"  (\w+), truth's own largest jump", line)
        if m:
            subject = m.group(1)
            table[subject] = {}
            continue
        m = re.match(r"    (\w+)\s+([\d.]+)\s+([\d.]+)\s+(\d+)", line)
        if m and subject:
            table[subject][m.group(1)] = float(m.group(3))
    names = ("now", "tile", "tile_p3", "blend", "rows2", "cells2")
    print("   subject    " + "".join(f"{n:>9s}" for n in names))
    shares = {n: [] for n in names}
    for s, row in table.items():
        cells = ""
        for n in names:
            if n in row:
                share = row[n] / row["tile"]
                if s != "z":  # z has no contours and its zero cell dominates: read it apart
                    shares[n].append(share)
                cells += f"{row[n]:9.2f}"
            else:
                cells += f"{'-':>9s}"
        print(f"   {s:10s} {cells}")
    print("   median share of tile, leaving out z: " + ", ".join(f"{n} {statistics.median(v):.2f}" for n, v in shares.items() if v))

    print("\n3. The centre cell of z and z2: darkest luma within 12 px of the plate's centre, and how many pixels wide the dark run is on that row (luma under half of the row's median there).")
    for s in ("z", "z2"):
        p = plates[s]
        cx, cy = (int(p["x0"]) + int(p["x1"])) // 2, (int(p["y0"]) + int(p["y1"])) // 2
        for c in ("now", "tile", "tile_p3", "blend", "rows2", "cells2"):
            a = np.asarray(Image.open(view / "stills" / f"{s}__{c}" / "frame_000000.png").convert("RGB"), dtype=float)
            w = luma(a[cy - 12 : cy + 13, cx - 12 : cx + 13])
            y, x = np.unravel_index(np.argmin(w), w.shape)
            run = int(np.sum(w[y] < np.median(w[y]) * 0.5))
            print(f"   {s:3s} {c:8s} darkest {w.min():6.1f} at ({x - 12:+d}, {y - 12:+d}), dark run {run} px")

    print("\n4. Knots. A blended strip of n cells holds 2n + 9, and 713 rows of 2 x 731 + 9 is the failure's number:")
    print(f"   713 x {2 * 731 + 9} = {713 * (2 * 731 + 9)}, failure said {FAILED_AT}, {'EQUAL' if 713 * (2 * 731 + 9) == FAILED_AT else 'DIFFERENT'}")
    print(f"   so cells4 on the 6.4 plate needs {knots(731, 731)} of {LIMIT}, {100 * knots(731, 731) / LIMIT:.1f}%, and stopped at row 713 of 731.")
    print("   A plate's live knots, % of the limit, by subject and rung (row and column counts from each candidate's receipt):")
    print("   subject    blend  rows2  cells2")
    for s in plates:
        out = []
        for c in ("blend", "rows2", "cells2"):
            r = next(x for x in rows if x["subject"] == s and x["candidate"] == c)
            out.append(100.0 * knots(int(r["rows"]), int(r["cols"])) / LIMIT)
        print(f"   {s:9s} {out[0]:5.1f}  {out[1]:5.1f}  {out[2]:6.1f}")

    def plate(width, height, cell):
        return knots(int(round(height / cell)), int(round(width / cell)))

    half = CELL / 2.0
    print("\n   Film scenes at cells2, from the planes in the source:")
    print(f"   lacunary 5.4 x 5.4 (continuing.rs:675)            {plate(5.4, 5.4, half):8d}  {100 * plate(5.4, 5.4, half) / LIMIT:5.1f}%")
    mosaic = 6 * plate(4.2, 2.6, half)
    print(f"   mosaic, 6 panels 4.2 x 2.6 (gallery.rs:857)       {mosaic:8d}  {100 * mosaic / LIMIT:5.1f}%")
    zoom = mosaic + plate(5.0, 5.0, half)
    print(f"   mosaic and the 5.0 zoom together (gallery.rs:895) {zoom:8d}  {100 * zoom / LIMIT:5.1f}%, if both are live at once")
    whole = plate(64.0 / 4.5, 8.0, half)
    print(f"   the whole 16:9 frame in plate, no scene does this {whole:8d}  {100 * whole / LIMIT:5.1f}%")
    print("\n   How many frames of a fade between compactions, the plate's knots being new at each opacity, on the 6.4 plate:")
    now = 2 * knots(183, 183)  # REPAIR.md: blend's colour records are 0.51 of now's, so about twice
    for name, k in (("now, about", now), ("blend", knots(183, 183)), ("rows2", knots(366, 183)), ("cells2", knots(366, 366))):
        print(f"   {name:11s} {k:8d} knots, every {math.floor(LIMIT / k)} frames")

    print("\n5. Which painted plates put a cell's centre exactly on a zero or pole of what they paint? shade() has no colour there, so draw.rs paints the background: the black cell.")
    print("   model::cell_input in f64, every cell tested for exact equality. Rows x cols, then the point hit and the function hit, by rung. The strip plane is left out: its only exact point, 1, is on its lower edge, where no cell centre can land.")
    print(f"   {'plate':28s}{'now':>30s}{'rows2':>30s}{'cells2':>30s}")
    for name, width, x, y, funcs in PAINTED:
        height = width * (y[1] - y[0]) / (x[1] - x[0])
        cells = []
        for row, col in ((CELL, CELL), (half, CELL), (half, half)):
            cols = max(math.floor(width / col + 0.5), 8)
            rows_n = max(math.floor(height / row + 0.5), 8)
            re_axis = x[0] + (x[1] - x[0]) * ((np.arange(cols) + 0.5) / cols)
            im_axis = y[1] - (y[1] - y[0]) * ((np.arange(rows_n) + 0.5) / rows_n)
            hits = []
            for f in funcs.split():
                for pt in AT[f]:
                    z = complex(pt)
                    if np.any(re_axis == z.real) and np.any(im_axis == z.imag):
                        hits.append(f"{f}@{pt}")
            cells.append(f"{rows_n}x{cols} {' '.join(hits) or '-'}")
        print(f"   {name:28s}" + "".join(f"{c:>30s}" for c in cells))


if __name__ == "__main__":
    main(sys.argv[1])
