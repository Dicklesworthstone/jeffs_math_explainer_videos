"""Test frames against the moved reach label, by pixel rectangles.

Run from the repo root. For a sampled rerender of chapter 11 at 1280 x 720 (other sizes are resized):
  python3 -I -B <this file> <frames-dir> BEFORE=<n> SWEEP=<n>,<n>,...

<frames-dir> holds frame-NNNN.png. BEFORE is one frame after the label has come and before
"cannot move", with the disk still at its start. SWEEP is several frames between "cannot move"
and the sentence "Then no turn can hide", the last third of the sweep among them. Under Oliver
pick the numbers from those moments. With no arguments the script runs its own controls:
preview02's frames must fail, and the desk composites must pass.

A pass means the right pixels are in the right rectangles on those frames: the label is whole
over the window, its old place is empty, and nothing touched the label while the disk went round.
It is not picture acceptance, and it says nothing about sound or sync. Requires Pillow. Writes nothing.
"""
import math
import sys

from PIL import Image

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-review-repair01/checks/desk-composites/"
PREVIEW02 = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/11_counting"
BEFORE, SWEEP = 535, [544, 547, 549, 550, 551]
SIZE = (1280, 720)
SLOT = (355, 160, 466, 182)
OLD = (415, 376, 526, 396)
WINDOW = (288, 188, 534, 434)


def load(path):
    im = Image.open(path).convert("RGB")
    return im if im.size == SIZE else im.resize(SIZE, Image.LANCZOS)


def teal(p):
    return p[1] > p[0] + 40 and p[2] > p[0] + 30


def red(p):
    return p[0] > p[1] + 50


def pixels(im, box, test):
    px = im.load()
    return [(x, y) for y in range(box[1], box[3]) for x in range(box[0], box[2]) if test(px[x, y])]


def area(box):
    return (box[2] - box[0]) * (box[3] - box[1])


def moved(a, b, box):
    pa, pb = a.load(), b.load()
    return sum(1 for y in range(box[1], box[3]) for x in range(box[0], box[2]) if max(abs(pa[x, y][i] - pb[x, y][i]) for i in range(3)) > 40)


def centre(im):
    """Where the disk is: the mean of its teal pixels, enough to show it travelling."""
    points = [p for p in pixels(im, WINDOW, teal) if not (OLD[0] <= p[0] < OLD[2] and OLD[1] <= p[1] < OLD[3])]
    if not points:
        return None
    return sum(p[0] for p in points) / len(points), sum(p[1] for p in points) / len(points)


def probe(before, sweep):
    results = []
    name = pixels(before, SLOT, teal)
    results.append((f"label ink over the window, {len(name)} teal pixels", len(name) > 150))
    if name:
        xs, ys = [p[0] for p in name], [p[1] for p in name]
        results.append((f"label ink x {min(xs)} to {max(xs)}, y {min(ys)} to {max(ys)}: 95 to 112 wide, whole inside the slot", 95 <= max(xs) - min(xs) + 1 <= 112 and min(xs) > SLOT[0] and max(xs) < SLOT[2] - 1 and min(ys) > SLOT[1] and max(ys) < SLOT[3] - 1))
        results.append(("label centred over the window at x 410.5, within 4", abs((min(xs) + max(xs)) / 2 - 410.5) < 4))
    results.append(("old place empty before the sweep", len(pixels(before, OLD, teal)) < 5))
    results.append(("no red label left in the slot", len(pixels(before, SLOT, red)) < 5))
    for k, im in enumerate(sweep):
        change = moved(before, im, SLOT)
        results.append((f"sweep {k}: slot unchanged, {change} of {area(SLOT)} pixels differ", change <= 4))
    places = [centre(im) for im in sweep]
    known = [p for p in places if p]
    spread = max((math.dist(a, b) for a in known for b in known), default=0.0)
    results.append((f"the disk travels across these frames, by {spread:.0f} px", len(known) == len(sweep) and spread > 60))
    crossing = max((len(pixels(im, OLD, teal)) for im in sweep), default=0)
    results.append((f"one of them is the crossing moment: {crossing} teal pixels pass through the old place", crossing > 20))
    return results


def report(label, before, sweep):
    results = probe(before, sweep)
    ok = all(good for _, good in results)
    print(f"{'PASS' if ok else 'FAIL'} {label}")
    for what, good in results:
        print(f"     {'ok ' if good else 'BAD'} {what}")
    return ok


def main():
    if len(sys.argv) > 1:
        root = sys.argv[1].rstrip("/")
        numbers = dict(arg.split("=") for arg in sys.argv[2:])
        before = int(numbers["BEFORE"])
        sweep = [int(n) for n in numbers["SWEEP"].split(",")]
        ok = report(root, load(f"{root}/frame-{before:04d}.png"), [load(f"{root}/frame-{n:04d}.png") for n in sweep])
        print("RESULT " + ("PASS" if ok else "FAIL"))
        return
    bad = 0
    print("== preview02: the anchor must fail")
    if report("preview02", load(f"{PREVIEW02}/frame-{BEFORE:04d}.png"), [load(f"{PREVIEW02}/frame-{n:04d}.png") for n in SWEEP]):
        print("     CONTROL BROKEN: preview02 passed")
        bad += 1
    print("== desk composites: the anchor must pass")
    if not report("composites", load(f"{HERE}reach-from-frame-{BEFORE:04d}.png"), [load(f"{HERE}reach-from-frame-{n:04d}.png") for n in SWEEP]):
        print("     CONTROL BROKEN: the composites failed")
        bad += 1
    print("RESULT " + ("PASS" if bad == 0 else f"FAIL {bad}"))


main()
