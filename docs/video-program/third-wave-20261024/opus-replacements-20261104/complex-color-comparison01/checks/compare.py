"""Labelled side-by-side panels from real lab frames, nearest-neighbour enlarged, for looking only. Measures nothing, changes no pixel value.

    python3 -B .../checks/compare.py VIEW NEW_OUT_DIR     (VIEW from consolidate.py, from the repo root)

Unlike measure_lab's sheets, a window that runs past a plate's box still shows the truth portrait in its right place, because truth is
pasted on a background canvas first. (The measure's sheet leaves truth blank for `recip`: a negative slice in its crop, a sheet bug only.)
"""
import pathlib
import sys

import numpy as np
from PIL import Image, ImageDraw

BG = (20, 20, 20)
GAP = 8
LABEL = 22


def load(path):
    return np.asarray(Image.open(path).convert("RGB"))


class View:
    def __init__(self, root):
        self.root = pathlib.Path(root)
        rows = (self.root / "plates.tsv").read_text().splitlines()
        names = rows[0].split("\t")
        self.plates = {r.split("\t")[0]: dict(zip(names, r.split("\t"))) for r in rows[1:]}

    def box(self, subject):
        p = self.plates[subject]
        return int(p["x0"]), int(p["y0"]), int(p["x1"]), int(p["y1"])

    def frame(self, kind, subject, candidate, index=0):
        files = sorted((self.root / kind / f"{subject}__{candidate}").glob("*.png"))
        return load(files[index])

    def truth(self, subject):
        t = load(self.root / "truth" / f"{subject}.ppm")
        canvas = self.frame("stills", subject, "now").copy()
        x0, y0, x1, y1 = self.box(subject)
        canvas[y0:y0 + t.shape[0], x0:x0 + t.shape[1]] = t
        return canvas


def sheet(cells, scale, per_row, path):
    h, w = cells[0][1].shape[:2]
    w *= scale
    h *= scale
    rows = (len(cells) + per_row - 1) // per_row
    out = Image.new("RGB", (per_row * (w + GAP) - GAP, rows * (h + LABEL + GAP)), BG)
    draw = ImageDraw.Draw(out)
    for i, (name, a) in enumerate(cells):
        x, y = (i % per_row) * (w + GAP), (i // per_row) * (h + LABEL + GAP)
        draw.text((x + 4, y + 5), name, fill=(235, 235, 235))
        out.paste(Image.fromarray(a).resize((w, h), Image.NEAREST), (x, y + LABEL))
    out.save(path)
    print(f"{path.name}: {len(cells)} panels at {scale}x, {out.size[0]}x{out.size[1]}")


def main(view_dir, new_dir):
    view = View(view_dir)
    out = pathlib.Path(new_dir)
    out.mkdir(parents=True, exist_ok=False)

    def centre(subject, half):
        x0, y0, x1, y1 = view.box(subject)
        cx, cy = (x0 + x1) // 2, (y0 + y1) // 2
        return cx - half, cy - half, cx + half, cy + half

    def still(subject, window, candidates, scale, per_row, name):
        x0, y0, x1, y1 = window
        cells = []
        for c in candidates:
            a = view.truth(subject) if c == "truth" else view.frame("stills", subject, c)
            cells.append((f"{subject} {c}" + (", not a render" if c == "truth" else ""), a[y0:y1, x0:x1]))
        sheet(cells, scale, per_row, out / f"{name}.png")

    def fades(subject, window, candidates, frames, scale, name):
        x0, y0, x1, y1 = window
        cells = []
        for c in candidates:
            for k in frames:
                cells.append((f"{subject} {c} frame {k}", view.frame("fades", subject, c, k)[y0:y1, x0:x1]))
        sheet(cells, scale, len(frames), out / f"{name}.png")

    ladder = ("truth", "now", "tile", "tile_p3", "blend", "rows2", "cells2")
    short = ("truth", "now", "tile", "blend", "rows2", "cells2")

    # stripes, whole plate at 100%
    z = view.box("z")
    still("z", z, ("now", "overlap", "overlap06", "tile", "tile_raw", "flat_control"), 1, 3, "stripes-z-whole")
    still("z", (z[0] + 120, z[1] + 120, z[0] + 320, z[1] + 270), ("now", "overlap", "overlap06", "tile", "tile_raw", "blend"), 3, 3, "stripes-z-3x")

    # stairs: whole plates
    still("z2", view.box("z2"), short, 1, 3, "stairs-z2-whole")
    still("sinrecip", view.box("sinrecip"), short, 1, 3, "stairs-sinrecip-whole")
    still("sin", view.box("sin"), short, 1, 3, "stairs-sin-whole")
    still("zoom", view.box("zoom"), short, 1, 3, "stairs-zoom-whole")
    still("bump", view.box("bump"), short, 1, 3, "stairs-bump-whole")
    still("openbump", view.box("openbump"), short, 1, 3, "stairs-openbump-whole")
    still("recip", view.box("recip"), short, 2, 3, "stairs-recip-2x")
    still("tan", view.box("tan"), short, 1, 2, "stairs-tan-whole")

    # enlargements
    still("sinrecip", centre("sinrecip", 80), ladder, 3, 3, "dense-sinrecip-centre-3x")
    still("z2", centre("z2", 70), ladder, 3, 3, "dense-z2-centre-3x")
    still("z", centre("z", 16), ("truth", "now", "tile", "blend", "tile_p3", "rows2", "cells2"), 12, 4, "zero-z-centre-12x")
    x0, y0, x1, y1 = view.box("sin")
    still("sin", (x0 + 20, y0 + 60, x0 + 190, y0 + 230), ladder, 3, 3, "bands-sin-3x")
    x0, y0, x1, y1 = view.box("z")
    still("z", (x0 - 12, y0 - 12, x0 + 44, y0 + 44), ("now", "overlap06", "tile", "blend", "rows2", "cells2"), 8, 3, "edge-z-corner-8x")
    x0, y0, x1, y1 = view.box("tan")
    still("tan", ((x0 + x1) // 2 - 90, (y0 + y1) // 2 - 20, (x0 + x1) // 2 + 90, (y0 + y1) // 2 + 100), short, 3, 2, "dense-tan-3x")

    # whole frames, one each, so nothing outside the plate goes unseen
    for subject, candidate in (("z", "tile"), ("tan", "blend"), ("sinrecip", "cells2")):
        sheet([(f"{subject} {candidate}, whole frame", view.frame("stills", subject, candidate))], 1, 1, out / f"frame-{subject}-{candidate}.png")

    # fades: frames 0, 1, 2 (5, 4, 3 mirror them, 6 is empty)
    z = view.box("z")
    window = (z[0] + 120, z[1] + 120, z[0] + 320, z[1] + 270)
    fades("z", window, ("now", "overlap", "overlap06", "tile", "blend"), (0, 1, 2), 2, "fade-z-2x")
    t = view.box("tan")
    fades("tan", (t[0] + 40, t[1] + 20, t[0] + 300, t[1] + 200), ("now", "tile", "blend"), (0, 1, 2), 2, "fade-tan-2x")
    b = view.box("bump")
    fades("bump", (b[0] + 60, b[1] + 60, b[0] + 320, b[1] + 240), ("now", "tile", "blend"), (0, 1, 2), 2, "fade-bump-2x")

    # repaints, whole plate
    x0, y0, x1, y1 = view.box("z2")
    cells = []
    for c in ("now", "blend", "rows2"):
        for k in range(3):
            cells.append((f"z2 repaint {c} frame {k}", view.frame("repaints", "z2", c, k)[y0:y1, x0:x1]))
    sheet(cells, 1, 3, out / "repaint-z2.png")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
