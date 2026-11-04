"""Test frames against the five repaired pictures, by pixel rectangles.

Run from the repo root. For a sampled rerender laid out like complex-combined-picture-frames02:
  python3 -I -B <this file> <frames-dir> [A06=450] [A07=10] [A08N1=92] [A08N2=186] [A09=240]

The numbers are frame indices. They default to preview02's, which still fit a rerender on the
scratch voice, because the repair moves no beat. Under Oliver the indices move: pick each frame
from the moment named in HANDOFF.md and pass it. With no arguments the script runs its own
controls: preview02's frames must fail every anchor, and the desk composites must pass.

A pass means the right pixels are in the right rectangles on that one frame. It is not picture
acceptance, and it says nothing about motion, sound or sync. Requires Pillow. Writes nothing.
"""
import math
import sys

from PIL import Image

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex06-09-repair03/checks/desk-composites/"
PREVIEW02 = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/"
CHAPTER = {"A06": "06_one_derivative", "A07": "07_one_point", "A08N1": "08_loop_sums", "A08N2": "08_loop_sums", "A09": "09_boundary"}
DEFAULT = {"A06": 450, "A07": 10, "A08N1": 92, "A08N2": 186, "A09": 240}
SIZE = (1280, 720)


def load(path):
    im = Image.open(path).convert("RGB")
    return im if im.size == SIZE else im.resize(SIZE, Image.LANCZOS)


def share(im, box, test):
    px = im.load()
    x0, y0, x1, y1 = box
    hits = sum(1 for y in range(y0, y1) for x in range(x0, x1) if test(px[x, y]))
    return hits / ((x1 - x0) * (y1 - y0))


def ink(p):
    return max(p) > 70


def red(p):
    return p[0] > 150 and p[1] < 130 and p[2] < 130


def green(p):
    return p[1] > 110 and p[1] > p[0] + 25 and p[1] > p[2] + 25


def blue(p):
    return p[2] > 110 and p[2] > p[0] + 40


def chroma(im, box):
    px = im.load()
    x0, y0, x1, y1 = box
    total = sum(max(px[x, y]) - min(px[x, y]) for y in range(y0, y1) for x in range(x0, x1))
    return total / ((x1 - x0) * (y1 - y0))


def a06(im):
    """Reflection's rates gone; doubling's rates and cross above the verdict."""
    return [
        ("old right: band empty", share(im, (930, 202, 1110, 229), ink) < 0.005),
        ("old up: band empty", share(im, (930, 257, 1110, 283), ink) < 0.005),
        ("old readout band empty", share(im, (930, 462, 1110, 490), ink) < 0.005),
        ("old lower cross empty", share(im, (990, 540, 1046, 594), ink) < 0.005),
        ("new readout present", share(im, (930, 230, 1110, 256), ink) > 0.05),
        ("cross present", share(im, (990, 306, 1046, 360), red) > 0.1),
        ("verdict present", share(im, (915, 368, 1120, 395), red) > 0.05),
    ]


def a07(im):
    """A clear gap between Re and the arrow's tail; the name over the arrow's middle."""
    px = im.load()
    xs = [x for y in range(312, 345) for x in range(590, 720) if ink(px[x, y])]
    centre = (min(xs) + max(xs)) / 2 if xs else -1
    return [
        ("gap after Re is empty", share(im, (594, 368, 601, 379), ink) < 0.005),
        ("Re present", share(im, (570, 365, 593, 381), ink) > 0.1),
        ("arrow present", share(im, (606, 371, 668, 376), ink) > 0.8),
        (f"name centred near x 648, measured {centre:.1f}", abs(centre - 648) < 4),
    ]


def a08n1(im):
    """The z_k dot and its name on the ring."""
    return [
        ("dot present", share(im, (367, 321, 375, 329), green) > 0.8),
        ("name present", share(im, (378, 288, 404, 312), green) > 0.03),
    ]


def a08n2(im):
    """Each corner mark's head stays near its arc: no corner standing far out past the ring."""
    px = im.load()
    out = []
    for col, row in [(0, 3), (3, 3), (0, 0), (3, 0)]:
        cx, cy = 640 + 90 * (-2.6 - 1.5 + col), 360 - 90 * (-0.1 - 1.5 + row)
        reach = 0.0
        for y in range(round(cy) - 40, round(cy) + 40):
            for x in range(round(cx) - 40, round(cx) + 40):
                if blue(px[x, y]):
                    d = math.hypot(x - cx, y - cy)
                    if d < 40:
                        reach = max(reach, d)
        out.append((f"mark ({col},{row}) reach {reach:.1f} px, under 29.0", 0 < reach < 29.0))
    return out


def a09(im):
    """The a = 0 card whole at its new place, and the plate back where it was."""
    return [
        ("card fill at the new place", chroma(im, (409, 400, 488, 410)) < 28),
        ("card text at the new place", share(im, (424, 408, 476, 430), lambda p: min(p) > 150) > 0.03),
        ("plate where the card was", chroma(im, (386, 430, 404, 452)) > 40),
    ]


PROBES = {"A06": a06, "A07": a07, "A08N1": a08n1, "A08N2": a08n2, "A09": a09}


def run(name, path, verbose=True):
    results = PROBES[name](load(path))
    ok = all(good for _, good in results)
    if verbose:
        print(f"{'PASS' if ok else 'FAIL'} {name}")
        for what, good in results:
            print(f"     {'ok ' if good else 'BAD'} {what}")
    return ok


def frame_path(root, name, n):
    return f"{root}/{CHAPTER[name]}/frame-{n:04d}.png"


def main():
    if len(sys.argv) > 1:
        root = sys.argv[1].rstrip("/")
        frames = dict(DEFAULT)
        for arg in sys.argv[2:]:
            key, value = arg.split("=")
            frames[key] = int(value)
        results = [run(name, frame_path(root, name, n)) for name, n in frames.items()]
        print("RESULT " + ("PASS" if all(results) else "FAIL"))
        return
    bad = 0
    print("== preview02: every anchor must fail")
    for name, n in DEFAULT.items():
        if run(name, frame_path(PREVIEW02.rstrip("/"), name, n)):
            print(f"     CONTROL BROKEN: {name} passed on preview02")
            bad += 1
    print("== desk composites: every anchor must pass")
    for name, n in DEFAULT.items():
        if not run(name, f"{HERE}{name}-from-frame-{n:04d}.png"):
            print(f"     CONTROL BROKEN: {name} failed on its composite")
            bad += 1
    print("RESULT " + ("PASS" if bad == 0 else f"FAIL {bad}"))


main()
