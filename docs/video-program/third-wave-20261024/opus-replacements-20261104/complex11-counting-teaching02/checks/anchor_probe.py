"""Test frames of a rerendered chapter 11 against the acted division, the plated 0 and the moved reach label.

Run from the repo root. For sampled frames at 1280 x 720 (other sizes are resized):
  python3 -I -B <this file> <frames-dir> REF=<n> TURNED=<n> LANDED=<n> BEFORE=<n> SWEEP=<n>,<n>,... [ZERO_SWEEP=<n>,<n>,...]

<frames-dir> holds frame-NNNN.png. Under Oliver pick the numbers from these moments:
  REF     in 11_7, after the clear, with the claim up and before "complex polynomial"
  TURNED  in 11_7, the turn finished, at the end of the first sentence, before "Divide"
  LANDED  in 11_7, after "which leaves" with the new row up, before the wide plane
  BEFORE  in 11_8, the reach label up and "0" named, before "cannot move", the disk at its start
  SWEEP   in 11_8, several frames between "cannot move" and "Then no turn can hide", the last third of the sweep among them
  ZERO_SWEEP  optional, the frames for the 0 alone: include those with the line and the rim at the 0. Defaults to SWEEP.
With no arguments the script runs its own controls. On preview02's real pixels the marks must pass and the
other three groups fail, and the field and curve metrics must separate the film's t = 0 from the model's
t = 1. The desk composites must pass every group, though the comparison with the model's t = 1 is trivially
true on the composite made from it, which is why the real-pixel row exists. Four planted defects must each
fail their own group.

A pass means the right pixels are in the right places on those frames: the field and the curve left their
11_6 picture and came back to it, the five marks stayed, the 0 stayed whole while the line and the rim went
by, and the label is whole in its slot. It is not picture acceptance, and it says nothing about sound, sync,
the moments between the frames, or whether the transition reads. Requires Pillow. Writes nothing.
"""
import math
import sys

from PIL import Image

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex11-counting-teaching02/checks/desk-composites/"
PREVIEW02 = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/11_counting"
TURNED_STUDY = HERE + "a-end-of-sentence-0-t1.png"
SIZE = (1280, 720)
# the reach label, as the frozen probe had them
SLOT = (355, 160, 466, 182)
OLD = (415, 376, 526, 396)
WINDOW = (288, 188, 534, 434)
# the 0's digit, every pixel of it inside the plate centred (394.75, 329.4) with radius 9.45
DIGIT = (389, 322, 401, 337)
# the input field and what may legitimately lie on it
FIELD = (78, 139, 582, 643)
CENTRE, RING = (329.5, 391.5), 126.0
MARKS = [(372, 392), (483, 392), (319, 224), (319, 559), (155, 392)]
CARD = (236, 584, 424, 633)
# the output plane, and the row over the field
PLANE = (707, 166, 1158, 617)
ROW = (60, 80, 600, 130)


def load(path):
    im = Image.open(path).convert("RGB")
    return im if im.size == SIZE else im.resize(SIZE, Image.LANCZOS)


def teal(p):
    return p[1] > p[0] + 40 and p[2] > p[0] + 30


def red(p):
    return p[0] > p[1] + 50


def ink(p):
    return min(p) > 190


def gold(p):
    return p[0] > 200 and p[1] > 160 and p[2] < 150


def green(p):
    return p[1] > 130 and p[1] > p[0] + 30 and p[1] > p[2] + 40


def pixels(im, box, test):
    px = im.load()
    return [(x, y) for y in range(box[1], box[3]) for x in range(box[0], box[2]) if test(px[x, y])]


def area(box):
    return (box[2] - box[0]) * (box[3] - box[1])


def moved(a, b, box):
    pa, pb = a.load(), b.load()
    return sum(1 for y in range(box[1], box[3]) for x in range(box[0], box[2]) if max(abs(pa[x, y][i] - pb[x, y][i]) for i in range(3)) > 40)


def field_cells():
    """The field's pixels, without the ring, the marks and the card's place."""
    cells = []
    for y in range(FIELD[1], FIELD[3]):
        for x in range(FIELD[0], FIELD[2]):
            if abs(math.dist((x, y), CENTRE) - RING) < 9:
                continue
            if any(math.dist((x, y), m) < 14 for m in MARKS):
                continue
            if CARD[0] <= x < CARD[2] and CARD[1] <= y < CARD[3]:
                continue
            cells.append((x, y))
    return cells


CELLS = field_cells()


def same_share(a, b):
    pa, pb = a.load(), b.load()
    return sum(1 for x, y in CELLS if max(abs(pa[x, y][i] - pb[x, y][i]) for i in range(3)) <= 24) / len(CELLS)


def near_share(points, others, reach=5):
    grown = {(x + dx, y + dy) for x, y in others for dx in range(-reach, reach + 1) for dy in range(-reach, reach + 1)}
    return sum(1 for p in points if p in grown) / len(points) if points else 0.0


def mark_place(im, mark):
    box = (mark[0] - 12, mark[1] - 12, mark[0] + 13, mark[1] + 13)
    points = pixels(im, box, ink)
    if len(points) < 40:
        return None
    return sum(p[0] for p in points) / len(points), sum(p[1] for p in points) / len(points)


def centre(im):
    """Where the disk is: the mean of its teal pixels, enough to show it travelling."""
    points = [p for p in pixels(im, WINDOW, teal) if not (OLD[0] <= p[0] < OLD[2] and OLD[1] <= p[1] < OLD[3])]
    if not points:
        return None
    return sum(p[0] for p in points) / len(points), sum(p[1] for p in points) / len(points)


def turn_group(ref, turned, landed, study):
    results = []
    share = same_share(ref, turned)
    results.append((f"the turn changes the field: {1 - share:.0%} of its pixels differ by more than 24 levels from REF", share < 0.5))
    share = same_share(turned, study)
    results.append((f"TURNED's field is 2i times the quintic's: {share:.1%} within 24 levels of the model's picture", share > 0.9))
    share = same_share(ref, landed)
    results.append((f"the division lands on the picture it left: {share:.1%} of LANDED's field within 24 levels of REF", share > 0.98))
    curve_ref, curve_turned, curve_landed, curve_study = (pixels(im, PLANE, green) for im in (ref, turned, landed, study))
    near_study, near_ref = near_share(curve_turned, curve_study), near_share(curve_turned, curve_ref)
    results.append((f"TURNED's curve lies on the model's: {near_study:.0%} of its {len(curve_turned)} green pixels within 5 px, and {near_ref:.0%} on REF's", near_study > 0.9 and near_ref < 0.7))
    near_back = near_share(curve_landed, curve_ref)
    results.append((f"LANDED's curve is REF's again: {near_back:.0%} of its {len(curve_landed)} green pixels within 5 px", near_back > 0.9 and len(curve_landed) > 100))
    row = pixels(turned, ROW, gold)
    results.append((f"the row over the field has its gold leading coefficient: {len(row)} gold pixels", len(row) > 20))
    return results


def marks_group(ref, turned, landed):
    results = []
    for mark in MARKS:
        places = [mark_place(im, mark) for im in (ref, turned, landed)]
        here = [p for p in places if p]
        spread = max((math.dist(a, b) for a in here for b in here), default=0.0)
        results.append((f"mark near {mark}: on all three frames, moved {spread:.1f} px", len(here) == 3 and spread < 1.5 and math.dist(here[0], mark) < 4))
    return results


def zero_group(before, sweep):
    results = []
    digit = pixels(before, DIGIT, ink)
    results.append((f"the 0 is there before the sweep: {len(digit)} ink pixels in its box", len(digit) > 12))
    for k, im in enumerate(sweep):
        change = moved(before, im, DIGIT)
        results.append((f"0 sweep {k}: {change} of {area(DIGIT)} pixels differ", change <= 2))
    return results


def reach_group(before, sweep):
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


def show(name, results):
    ok = all(good for _, good in results)
    print(f"  {'PASS' if ok else 'FAIL'} {name}")
    for what, good in results:
        print(f"       {'ok ' if good else 'BAD'} {what}")
    return ok


def groups(frames):
    study = load(TURNED_STUDY)
    ref, turned, landed, before, sweep, zero = frames
    return {
        "turn": show("the division acts", turn_group(ref, turned, landed, study)),
        "marks": show("the marks stay", marks_group(ref, turned, landed)),
        "zero": show("the 0 stays whole", zero_group(before, zero)),
        "reach": show("the reach label", reach_group(before, sweep)),
    }


def frames_from(numbers, path):
    sweep = numbers["SWEEP"]
    zero = numbers.get("ZERO_SWEEP", sweep)
    return (load(path(numbers["REF"])), load(path(numbers["TURNED"])), load(path(numbers["LANDED"])), load(path(numbers["BEFORE"])), [load(path(n)) for n in sweep], [load(path(n)) for n in zero])


def moved_marks(im):
    """A planted defect: every mark lifted, its place covered with a piece of field from beside it."""
    out = im.copy()
    for x, y in MARKS:
        patch = im.crop((x - 13, y - 13, x + 14, y + 14))
        out.paste(im.crop((x - 53, y - 13, x - 26, y + 14)), (x - 13, y - 13))
        out.paste(patch, (x - 13 + 16, y - 13))
    return out


def covered_zero(im):
    """A planted defect: the plate drawn over the name instead of under it."""
    out = im.copy()
    px = out.load()
    for y in range(DIGIT[1], DIGIT[3]):
        for x in range(DIGIT[0], DIGIT[2]):
            px[x, y] = (12, 17, 20)
    return out


def main():
    if len(sys.argv) > 1:
        root = sys.argv[1].rstrip("/")
        numbers = dict(arg.split("=") for arg in sys.argv[2:])
        for key in ("SWEEP", "ZERO_SWEEP"):
            if key in numbers:
                numbers[key] = [int(n) for n in numbers[key].split(",")]
        for key in ("REF", "TURNED", "LANDED", "BEFORE"):
            numbers[key] = int(numbers[key])
        print(root)
        results = groups(frames_from(numbers, lambda n: f"{root}/frame-{n:04d}.png"))
        print("RESULT " + ("PASS" if all(results.values()) else "FAIL"))
        return
    bad = 0

    def expect(what, got, want):
        nonlocal bad
        good = got == want
        bad += 0 if good else 1
        print(f"{'ok  ' if good else 'BROKEN'} control: {what}: {got}")

    print("== preview02's real pixels, in 11_6 and 11_8: its marks hold still, and nothing else is done")
    preview = frames_from({"REF": 340, "TURNED": 346, "LANDED": 352, "BEFORE": 535, "SWEEP": [544, 547, 549, 550, 551], "ZERO_SWEEP": [544, 546, 547, 548, 549, 550, 551]}, lambda n: f"{PREVIEW02}/frame-{n:04d}.png")
    got = groups(preview)
    expect("preview02 passes the marks and fails the turn, the 0 and the label", (got["marks"], got["turn"], got["zero"], got["reach"]), (True, False, False, False))
    print("== the metrics on real pixels: the film's own t = 0 against the model's two pictures")
    plain = load(f"{PREVIEW02}/frame-0356.png")
    t0, t1 = load(HERE + "c-which-leaves-t0.png"), load(TURNED_STUDY)
    share0, share1 = same_share(plain, t0), same_share(plain, t1)
    print(f"     frame 356's field: {share0:.1%} within 24 levels of the model at t = 0, {share1:.1%} of the model at t = 1")
    expect("the field metric takes the film's t = 0 for t = 0 and not for t = 1", (share0 > 0.9, share1 > 0.5), (True, False))
    curve = pixels(preview[0], PLANE, green)
    near0, near1 = near_share(curve, pixels(t0, PLANE, green)), near_share(curve, pixels(t1, PLANE, green))
    print(f"     frame 340's {len(curve)} green pixels: {near0:.0%} within 5 px of the model's curve at t = 0, {near1:.0%} of the curve at t = 1")
    expect("the curve metric does the same", (near0 > 0.9, near1 > 0.7), (True, False))
    print("== the desk composites: every group must pass")
    composite = (
        load(HERE + "c-which-leaves-t0.png"),
        load(TURNED_STUDY),
        load(HERE + "d-sentence-2-t0.png"),
        load(HERE + "plate-from-frame-0535.png"),
        [load(f"{HERE}plate-from-frame-{n:04d}.png") for n in (544, 547, 549, 550, 551)],
        [load(f"{HERE}plate-from-frame-{n:04d}.png") for n in (544, 546, 547, 548, 549, 550, 551)],
    )
    got = groups(composite)
    expect("the composites pass all four groups", tuple(got.values()), (True, True, True, True))
    print("== planted defects, on the composites: each must fail its own group")
    ref, turned, landed, before, sweep, zero = composite
    got = groups((ref, turned, turned, before, sweep, zero))
    expect("a division that never turns back fails the turn", got["turn"], False)
    got = groups((ref, turned, moved_marks(landed), before, sweep, zero))
    expect("marks that move fail the marks", got["marks"], False)
    got = groups((ref, ref, landed, before, sweep, zero))
    expect("a turn that never happens fails the turn", got["turn"], False)
    got = groups((ref, turned, landed, covered_zero(before), [covered_zero(im) for im in sweep], [covered_zero(im) for im in zero]))
    expect("the plate over the name fails the 0", got["zero"], False)
    print("RESULT " + ("PASS" if bad == 0 else f"FAIL {bad}"))


main()
