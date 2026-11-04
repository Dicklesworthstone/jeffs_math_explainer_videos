"""What the validated mirror predicts for seam candidates at 720p, 1080p and 4K, on stills and during fades. Not a render.

    python3 -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-color-repair03/checks/seam_predict.py

seam_cause.py shows the mirror lands on the measured rows at 720p. Everything here is that same arithmetic at other sizes
and opacities, with a flat colour so that the row structure is all there is to see. These numbers choose what to measure;
they are not evidence of how a 4K frame looks.

Deviation is from the ideal: one flat colour composited at the fade's opacity. A negative number is a row darker than it
should be (background showing through), a positive one brighter (two rows covering one pixel during a fade).
"""
import math
import pathlib
import sys

import numpy as np

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import engine_mirror as em  # noqa: E402

FLAT = np.array([0.72, 0.45, 0.38])
BG = em.decode(np.array(em.BACKGROUND))
READING = ("reading plate, 6.4 wide", (-2.6, -0.25), 6.4, (-2.0, 2.0), (-2.0, 2.0))
SIZES = (720, 1080, 2160)
CANDIDATES = [
    ("C0 now: aa 0.5, width = cell", 0.5, lambda s: 0.0),
    ("C1 tile: aa 0, width = cell + 0.0001", 0.0, lambda s: 1e-4),
    ("    the same, no guard: aa 0, width = cell", 0.0, lambda s: 0.0),
    ("C2 small overlap: aa 0.5, width = cell + 0.6 px", 0.5, lambda s: 0.6 / s),
    ("C3 Mail 2167: aa 0.5, width = cell + 0.012", 0.5, lambda s: 0.012),
]


def ideal(opacity):
    return em.encode(BG + (em.decode(FLAT) - BG) * opacity)


def run(plate, height_px, aa, overlap, opacity, policy="adaptive"):
    stack = plate.stack(height_px, aa, overlap)
    linear = em.decode(FLAT)
    want = em.luma(ideal(opacity) * 255.0)
    # Stay away from the plate's top and bottom edges.
    y0 = int(stack.top_px + 12 * stack.cell * stack.scale)
    y1 = int(stack.top_px + (plate.rows - 12) * stack.cell * stack.scale)
    devs = []
    ss = 0
    for py in range(y0, y1):
        weights, g, s = stack.pixel_row(py, opacity, policy)
        ss += s > 1
        total = sum(weights.values())
        rgb = em.encode(BG * g + linear * total)
        devs.append(float(em.luma(rgb * 255.0)) - float(want))
    return np.array(devs), ss, y1 - y0


def fmt(d):
    return f"{d.min():+6.1f} {d.max():+6.1f} {math.sqrt(float(np.mean(d * d))):5.2f} {int(np.sum(np.abs(d) > 1.0)):4d}"


plate = em.Plate(*READING[1:])
print(f"{READING[0]}: {plate.rows} rows. Flat colour, luma {float(em.luma(FLAT * 255.0)):.0f} of 255.")
print("Each cell: lowest, highest deviation from ideal (luma), rms, pixel rows off by more than 1 luma.")
for height_px in SIZES:
    scale = height_px / 8.0
    print(f"\n{height_px}p: {scale:.1f} px a unit, a row is {plate.height / plate.rows * scale:.2f} px")
    print(f"{'candidate':52s} {'opacity 1':>25s} {'opacity 0.75':>25s} {'opacity 0.5':>25s} {'opacity 0.25':>25s}  2x2 rows")
    for name, aa, overlap in CANDIDATES:
        cells = []
        ss_full = total = 0
        for opacity in (1.0, 0.75, 0.5, 0.25):
            d, ss, total = run(plate, height_px, aa, overlap(scale), opacity)
            if opacity == 1.0:
                ss_full = ss
            cells.append(f"{fmt(d):>25s}")
        print(f"{name:52s} " + " ".join(cells) + f"  {ss_full}/{total}")

# Row geometry on every plate with literal numbers in source, for the guard: is there a real gap between two rows, and
# does the guard close it? In pixels, f32 rounding of y and width included.
PLATES = {
    "boundary picture": ((-3.3, -0.2), 5.6, (-2.5, 2.5), (-2.5, 2.5)),
    "closing": ((-1.6, -0.2), 7.6, (-1.2, 6.8), (-3.4, 3.4)),
    "counting input": ((-3.45, -0.35), 5.6, (-2.0, 2.0), (-2.0, 2.0)),
    "counting output": ((3.25, -0.35), 5.0, (-1.5, 1.5), (-1.5, 1.5)),
    "counting inset": ((5.75, 2.35), 1.5, (-0.6, 0.6), (-0.6, 0.6)),
    "continuing bump": ((-3.3, -0.35), 5.6, (-2.5, 2.5), (-2.5, 2.5)),
    "continuing gamma": ((-2.6, -0.5), 7.5, (-4.8, 3.0), (-2.0, 2.0)),
    "lacunary": ((-3.0, -0.35), 5.4, (-1.05, 1.05), (-1.05, 1.05)),
    "gallery square": ((-3.4, -0.2), 5.6, (-2.0, 2.0), (-2.0, 2.0)),
    "gallery exp": ((-2.6, -0.25), 3.2, (-3.0, 3.0), (-4.0, 8.0)),
    "gallery trig": ((-2.4, -0.1), 8.4, (-6.6, 6.6), (-3.3, 3.3)),
    "gallery gamma": ((-2.4, -0.3), 8.4, (-4.5, 4.5), (-2.5, 2.5)),
    "gallery zeta": ((-3.4, 0.2), 6.4, (-7.0, 3.0), (-3.0, 3.0)),
    "gallery zoom": ((2.6, -0.3), 5.0, (-0.65, 0.65), (-0.65, 0.65)),
    "reading": ((-2.6, -0.25), 6.4, (-2.0, 2.0), (-2.0, 2.0)),
    "one point painted": ((-2.8, -0.25), 5.4, (-2.0, 2.0), (-2.0, 2.0)),
    "one point inset": ((4.4, -0.9), 2.6, (-0.6, 0.6), (-0.6, 0.6)),
    "bump small": ((-5.0, 2.0), 1.5, (-1.2, 1.2), (-1.2, 1.2)),
    "rigidity trig": ((-3.0, -0.3), 5.4, (-3.6, 3.6), (-3.6, 3.6)),
    "rigidity sin(1/z)": ((-2.8, 0.0), 5.6, (-0.42, 0.42), (-0.42, 0.42)),
    "rigidity sin z": ((-2.8, -0.3), 6.0, (-4.0, 4.0), (-3.5, 3.5)),
}


SAMPLE_OFFSETS = (0.25, 0.5, 0.75)


def lattice(plate, height_px, overlap):
    """Largest and smallest gap between neighbouring rows, in px, and how many row boundaries have a real hole or a real
    overlap with one of the classifier's sample heights strictly inside it. Asked directly of each interval."""
    stack = plate.stack(height_px, 0.0, overlap)
    lower_edge = stack.centres[:-1] + stack.half
    upper_edge = stack.centres[1:] - stack.half
    gaps = upper_edge - lower_edge
    holes = doubles = 0
    for low, high in zip(lower_edge, upper_edge):
        a, b = min(low, high), max(low, high)
        base = np.floor(a)
        inside = any(a < base + k + off < b for k in (0.0, 1.0) for off in SAMPLE_OFFSETS)
        if inside and high > low:
            holes += 1
        elif inside:
            doubles += 1
    return float(gaps.max()), float(gaps.min()), holes, doubles


print("\nRow lattice, aa 0. gap > 0 is a real hole between two rows, in px. Each size lists: largest gap with no guard, and")
print("boundaries whose hole holds a sample height; largest gap with the guard, and boundaries whose overlap holds one.")
print(f"{'plate':20s} {'rows':>4s} | " + " | ".join(f"{h}p" for h in SIZES))
worst_unguarded = -1.0
worst_guarded = -1.0
open_holes = 0
for name, args in PLATES.items():
    p = em.Plate(*args)
    cells = []
    for h in SIZES:
        g0, _, holes0, _ = lattice(p, h, 0.0)
        g1, _, _, doubles1 = lattice(p, h, 1e-4)
        worst_unguarded = max(worst_unguarded, g0)
        worst_guarded = max(worst_guarded, g1)
        open_holes += holes0
        cells.append(f"{g0:+.1e}, {holes0}; {g1:+.1e}, {doubles1}")
    print(f"{name:20s} {p.rows:4d} | " + " | ".join(cells))
print(f"\nlargest unguarded gap {worst_unguarded:+.2e} px; largest guarded gap {worst_guarded:+.2e} px (negative: rows overlap)")
print(f"unguarded boundaries, all plates and sizes, with a sample height inside a real hole: {open_holes}")

# Controls: the arithmetic must be able to fail. Wider AA with no overlap must deepen the rows. A planted hole must show:
# 0.3 px under adaptive, where a probe lands in it on many rows, and 0.02 px under canonical, where only the pixel centre
# can land in it. A 0.02 px hole under adaptive stays dark-free, and that is reported beside them rather than hidden.
d0, _, _ = run(plate, 2160, 0.5, 0.0, 1.0)
d1, _, _ = run(plate, 2160, 1.5, 0.0, 1.0)
d2, _, _ = run(plate, 2160, 0.0, -0.3 / 270.0, 1.0)
d3, _, _ = run(plate, 2160, 0.0, -0.02 / 270.0, 1.0, "canonical")
d4, _, _ = run(plate, 2160, 0.0, -0.02 / 270.0, 1.0)
print(f"\ncontrols, 2160p: aa 0.5 rms {math.sqrt(float(np.mean(d0 * d0))):.2f}; aa 1.5 rms {math.sqrt(float(np.mean(d1 * d1))):.2f}")
print(f"aa 0, planted 0.3 px hole, adaptive: lowest {d2.min():+.1f}, {int(np.sum(np.abs(d2) > 1.0))} rows off")
print(f"aa 0, planted 0.02 px hole, canonical: lowest {d3.min():+.1f}, {int(np.sum(np.abs(d3) > 1.0))} rows off")
print(f"aa 0, planted 0.02 px hole, adaptive: lowest {d4.min():+.1f}, {int(np.sum(np.abs(d4) > 1.0))} rows off (the classifier only ever samples it at 2x)")
if not (d1.std() > d0.std() and d2.min() < -1.0):
    print("CONTROL FAILED: the arithmetic did not show a planted defect")


def sweep(height_px, aa, overlap, opacity, policy):
    lo, hi, off = 0.0, 0.0, 0
    for args in PLATES.values():
        p = em.Plate(*args)
        stack = p.stack(height_px, aa, overlap)
        linear = em.decode(FLAT)
        want = float(em.luma(ideal(opacity) * 255.0))
        y0 = int(stack.top_px + 3 * stack.cell * stack.scale)
        y1 = int(stack.top_px + (p.rows - 3) * stack.cell * stack.scale)
        for py in range(y0, y1):
            weights, g, _ = stack.pixel_row(py, opacity, policy)
            rgb = em.encode(BG * g + linear * sum(weights.values()))
            d = float(em.luma(rgb * 255.0)) - want
            lo, hi = min(lo, d), max(hi, d)
            off += abs(d) > 1.0
    return lo, hi, off


print("\nEvery plate above, every pixel row: lowest, highest deviation (luma), rows off by more than 1. aa 0.")
print("adaptive is the policy the film's standard renders use. canonical has no classifier: one sample at the pixel centre.")
print(f"{'':28s} {'adaptive, opacity 1':>26s} {'adaptive, opacity 0.5':>26s} {'canonical, opacity 1':>26s} {'canonical, opacity 0.5':>26s}")
for label, ov in (("no guard", 0.0), ("guard 0.0001", 1e-4)):
    for h in SIZES:
        cells = []
        for policy in ("adaptive", "canonical"):
            for opacity in (1.0, 0.5):
                lo, hi, off = sweep(h, 0.0, ov, opacity, policy)
                cells.append(f"{lo:+7.1f} {hi:+7.1f} {off:6d}")
        print(f"{label:16s} {h:5d}p      " + "  ".join(f"{c:>26s}" for c in cells))
