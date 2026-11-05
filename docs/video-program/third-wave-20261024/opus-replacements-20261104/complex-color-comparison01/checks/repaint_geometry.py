"""Is each repaint's last frame the right picture in the right place? Compares it with z^3-1 computed per pixel, with no cell grid.

    python3 -B .../checks/repaint_geometry.py VIEW     (VIEW from consolidate.py, from the repo root)

The measure's section F compares the last frame with the mirror's picture on the `now` grid, which marks a finer grid down just for
being finer. This one has no grid to favour: each pixel centre's own input through the film's colour formula (engine_mirror.portrait).
Then the same comparison with the ideal stretched about the plate's centre, to see whether any candidate lands better when moved.
Also each still of z2 against z^2, the same way, as a positive control on the method.
"""
import csv
import importlib.util
import pathlib
import sys

import numpy as np
from PIL import Image

HERE = pathlib.Path(__file__).resolve()
MIRROR = HERE.parents[2] / "complex-color-repair03/checks/engine_mirror.py"
spec = importlib.util.spec_from_file_location("engine_mirror", MIRROR)
em = importlib.util.module_from_spec(spec)
sys.dont_write_bytecode = True
spec.loader.exec_module(em)


def load(path):
    return np.asarray(Image.open(path).convert("RGB"), dtype=np.float64) / 255.0


def ideal(plate_row, f, h, stretch_y=1.0, shift_y=0.0):
    """Per-pixel colours inside the plate's box."""
    x0, y0, x1, y1 = (int(plate_row[k]) for k in ("x0", "y0", "x1", "y1"))
    left, top, width, height = (float(plate_row[k]) for k in ("left", "top", "width", "height"))
    re0, re1, im0, im1 = (float(plate_row[k]) for k in ("re0", "re1", "im0", "im1"))
    scale = h / em.FRAME_HEIGHT
    w = int(round(h * 16 / 9))
    xs = (np.arange(x0, x1) + 0.5 - w / 2.0) / scale
    ys = (h / 2.0 - (np.arange(y0, y1) + 0.5)) / scale
    u = (xs - left) / width
    t = (top - ys) / height
    re = re0 + (re1 - re0) * u
    im = im1 + (im0 - im1) * t
    centre = (im0 + im1) / 2.0
    im = centre + (im - centre) / stretch_y + shift_y
    z = re[None, :] + 1j * im[:, None]
    return em.portrait(f, z, float(plate_row["enhance"]))


def main(view):
    view = pathlib.Path(view)
    rows = {r["subject"]: r for r in csv.DictReader(open(view / "plates.tsv"), delimiter="\t")}
    p = rows["z2"]
    h = int(p["res"].split("x")[1])
    x0, y0, x1, y1 = (int(p[k]) for k in ("x0", "y0", "x1", "y1"))
    m = 5

    def compare(frame, target):
        a = frame[y0 + m : y1 - m, x0 + m : x1 - m]
        b = target[m : (y1 - y0) - m, m : (x1 - x0) - m]
        r = em.correlation(em.luma(a).ravel(), em.luma(b).ravel())
        return 255.0 * float(np.sqrt(np.mean((a - b) ** 2))), r

    print("Control: z2 stills against z^2, per pixel. rms of 255, r on luma.")
    target = ideal(p, em.FUNCS["z^2"], h)
    for c in ("now", "tile", "blend", "rows2", "cells2"):
        frame = load(sorted((view / "stills" / f"z2__{c}").glob("*.png"))[0])
        rms, r = compare(frame, target)
        print(f"  {c:7s} rms {rms:6.2f}  r {r:+.4f}")

    print("\nRepaints: each frame against z^3-1, per pixel. Frames before the last still hold some of z^2.")
    target = ideal(p, em.FUNCS["z^3-1"], h)
    for c in ("now", "blend", "rows2"):
        files = sorted((view / "repaints" / f"z2__{c}").glob("*.png"))
        for k, path in enumerate(files):
            rms, r = compare(load(path), target)
            print(f"  {c:7s} frame {k}  rms {rms:6.2f}  r {r:+.4f}")

    print("\nThe last frame against z^3-1 stretched vertically about the centre, then moved. A correct picture is best at 1.000 and 0.")
    for c in ("now", "blend", "rows2"):
        frame = load(sorted((view / "repaints" / f"z2__{c}").glob("*.png"))[-1])
        best = (1e9, None)
        for s in np.arange(0.90, 1.1001, 0.005):
            for d in (-0.04, -0.02, 0.0, 0.02, 0.04):
                rms, _ = compare(frame, ideal(p, em.FUNCS["z^3-1"], h, s, d))
                if rms < best[0]:
                    best = (rms, (float(s), d))
        print(f"  {c:7s} best rms {best[0]:6.2f} at stretch {best[1][0]:.3f}, shift {best[1][1]:+.2f}")

    print("\nHow far the crossfade had got at each frame, fitted per pixel: frame = (1-b) z^2 + b z^3-1 in encoded values, as Painter::to mixes.")
    a = ideal(p, em.FUNCS["z^2"], h)
    c_ = ideal(p, em.FUNCS["z^3-1"], h)
    for c in ("now", "blend", "rows2"):
        for k, path in enumerate(sorted((view / "repaints" / f"z2__{c}").glob("*.png"))):
            f = load(path)[y0:y1, x0:x1]
            d = (c_ - a).ravel()
            b = float(np.sum((f - a).ravel() * d) / np.sum(d * d))
            print(f"  {c:7s} frame {k}  b {b:.3f}")


if __name__ == "__main__":
    main(sys.argv[1])
