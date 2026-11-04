"""Does the engine's coverage rule, on the film's real row geometry, predict the dark rows measured on preview02?

    python3 -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-color-repair03/checks/seam_cause.py

Plain python3 -B: PIL and numpy are needed. Reads extracted preview02 frames, writes nothing. The review calls the cause
inferred; this is a test that could have failed. The mirror knows nothing about the frames: it predicts which pixel rows
go dark from the plate's position, cell count and anti-alias width. The frames were decoded from H.264 at CRF 26, so the
test is whether the pattern lands on the same rows, and roughly how deep. Two negative controls: the same prediction one
row off, and a prediction with a wrong plate position.
"""
import pathlib
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import engine_mirror as em  # noqa: E402

BASE = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02")
H = 720
READING = em.Plate((-2.6, -0.25), 6.4, (-2.0, 2.0), (-2.0, 2.0))
TRIG = em.Plate((-2.4, -0.1), 8.4, (-6.6, 6.6), (-3.3, 3.3))
# Mail 2542: chapter 10's two plates, rigidity.rs sin_recip_plane and sine_plane.
SIN_RECIP = em.Plate((-2.8, 0.0), 5.6, (-0.42, 0.42), (-0.42, 0.42))
SINE = em.Plate((-2.8, -0.3), 6.0, (-4.0, 4.0), (-3.5, 3.5))
# Mail 2591: chapter 12's small plates and its bump plate, residues.rs small_planes and bump_plane.
SMALL = {x: em.Plate((x, -1.0), 2.8, (-1.2, 1.2), (-1.2, 1.2)) for x in (-4.9, -1.7, 1.5)}
BUMP = em.Plate((-3.4, -0.3), 5.0, (-2.0, 2.0), (-2.0, 2.0))
# Mail 2614: chapter 14's opening plate, continuing.rs bump_plane.
OPENING = em.Plate((-3.3, -0.35), 5.6, (-2.5, 2.5), (-2.5, 2.5))

# label, frame, plate, function, enhance, x0, x1, y0, y1. Patches are the review's, clear of loops, markers and cards.
CASES = [
    ("f=z, hue only, upper left", "04_reading_color/frame-0020.png", READING, "z", 0.0, 130, 390, 120, 360),
    ("f=z, hue only, lower left", "04_reading_color/frame-0020.png", READING, "z", 0.0, 130, 390, 400, 650),
    ("f=z, hue only, lower right", "04_reading_color/frame-0020.png", READING, "z", 0.0, 420, 680, 400, 650),
    ("z^2, enhanced, upper left", "04_reading_color/frame-0143.png", READING, "z^2", 1.0, 125, 265, 100, 240),
    ("z^2, enhanced, lower left", "04_reading_color/frame-0143.png", READING, "z^2", 1.0, 125, 265, 520, 660),
    ("tan, enhanced, top flat", "05_gallery/frame-0217.png", TRIG, "tan", 1.0, 60, 480, 185, 235),
    ("tan, enhanced, bottom flat", "05_gallery/frame-0217.png", TRIG, "tan", 1.0, 60, 480, 505, 552),
    ("sin(1/z), enhanced, upper left", "10_rigidity/frame-0170.png", SIN_RECIP, "sin(1/z)", 1.0, 145, 375, 116, 340),
    ("sin(1/z), enhanced, lower left", "10_rigidity/frame-0170.png", SIN_RECIP, "sin(1/z)", 1.0, 145, 375, 380, 605),
    ("sin(1/z), enhanced, upper right", "10_rigidity/frame-0170.png", SIN_RECIP, "sin(1/z)", 1.0, 400, 632, 116, 340),
    ("sin z, enhanced, upper left", "10_rigidity/frame-0280.png", SINE, "sin", 1.0, 128, 378, 158, 370),
    ("sin z, enhanced, lower left", "10_rigidity/frame-0280.png", SINE, "sin", 1.0, 128, 378, 460, 615),
    ("sin z, enhanced, upper right", "10_rigidity/frame-0280.png", SINE, "sin", 1.0, 400, 650, 158, 370),
    ("residues z, small plate", "12_residues/frame-0120.png", SMALL[-4.9], "z", 1.0, 80, 190, 330, 435),
    ("residues z^2, small plate", "12_residues/frame-0120.png", SMALL[-1.7], "z^2", 1.0, 368, 475, 330, 435),
    ("residues 1/z, small plate", "12_residues/frame-0120.png", SMALL[1.5], "1/z", 1.0, 655, 765, 330, 435),
    ("residues bump, upper left", "12_residues/frame-0140.png", BUMP, "bump", 1.0, 115, 290, 170, 255),
    ("residues bump, lower left", "12_residues/frame-0140.png", BUMP, "bump", 1.0, 115, 300, 525, 605),
    ("continuation bump, upper left", "14_continuation/frame-0020.png", OPENING, "bump", 1.0, 100, 280, 148, 270),
    ("continuation bump, lower left", "14_continuation/frame-0020.png", OPENING, "bump", 1.0, 100, 280, 520, 635),
]


def measured(rel, x0, x1, y0, y1):
    a = np.asarray(Image.open(BASE / rel).convert("RGB"), dtype=float)
    return em.luma(a[y0:y1, x0:x1]).mean(axis=1)


def predicted(plate, f, enhance, x0, x1, y0, y1, aa=0.5, overlap=0.0, policy="adaptive", shift_top=0.0):
    stack = em.Stack(H, plate.top + shift_top, plate.height, plate.rows, aa, overlap)
    colours = plate.colours(em.FUNCS[f], enhance)
    out, _ = em.composite(plate, stack, colours, x0, x1, y0, y1, H, policy=policy)
    return em.luma(out * 255.0).mean(axis=1)


print("mirror against preview02, 720p, CRF 26. Residual after a 9-row running mean, luma 0..255.")
print("r = mirror vs frame on the same rows. off1 = one row off. wrong = the plate's top moved 1.3 px, a wrong lattice")
print("seam r takes the seam-free mirror (aa 0 + guard) away from both first, so what is left is the seams alone. It matters")
print("on sin z, whose real horizontal bands dominate r and leave off1 and wrong high.")
print("case | frame rms %, period | mirror rms %, period | r | off1 | wrong | seam r | seam off1")
results = []
for label, rel, plate, f, enhance, x0, x1, y0, y1 in CASES:
    m = measured(rel, x0, x1, y0, y1)
    p = predicted(plate, f, enhance, x0, x1, y0, y1)
    q = predicted(plate, f, enhance, x0, x1, y0, y1, shift_top=1.3 / 90.0)
    rm, rp, rq = em.residual(m), em.residual(p), em.residual(q)
    r = em.correlation(rm, rp)
    off = em.correlation(rm[:-1], rp[1:])
    wrong = em.correlation(rm, rq)
    clean = predicted(plate, f, enhance, x0, x1, y0, y1, aa=0.0, overlap=1e-4)
    seam_m, seam_p = em.residual(m - clean), em.residual(p - clean)
    seam_r = em.correlation(seam_m, seam_p)
    seam_off = em.correlation(seam_m[:-1], seam_p[1:])
    sm, sp = em.stats(m), em.stats(p)
    results.append((label, r, off, wrong, seam_r, seam_off))
    print(f"{label} | {sm[2]:.2f}%, {sm[3]:.2f} | {sp[2]:.2f}%, {sp[3]:.2f} | {r:+.3f} | {off:+.3f} | {wrong:+.3f} | {seam_r:+.3f} | {seam_off:+.3f}")

print("\nfirst rows of f=z upper left, frame then mirror (the review printed the frame's line):")
label, rel, plate, f, enhance, x0, x1, y0, y1 = CASES[0]
print("frame ", " ".join(f"{v:+.1f}" for v in em.residual(measured(rel, x0, x1, y0, y1))[:24]))
print("mirror", " ".join(f"{v:+.1f}" for v in em.residual(predicted(plate, f, enhance, x0, x1, y0, y1))[:24]))

print("\nwhat removes the mirror's dark rows on the same patch (rms % of row residual, frame shown above):")
for name, kw in [
    ("aa 0.5, width = cell (the film now)", dict(aa=0.5)),
    ("aa 1.5, width = cell (the engine's default)", dict(aa=1.5)),
    ("aa 0.0, width = cell + 0.0001", dict(aa=0.0, overlap=1e-4)),
    ("aa 0.5, width = cell + 0.012 (Mail 2167)", dict(aa=0.5, overlap=0.012)),
]:
    s = em.stats(predicted(plate, f, enhance, x0, x1, y0, y1, **kw))
    print(f"  {name}: {s[2]:.2f}%, period {s[3]:.2f}")

weak = [x for x in results if not (x[1] > 0.5 and x[1] > x[2] + 0.3 and x[1] > x[3] + 0.3 and x[4] > 0.5 and x[4] > x[5] + 0.3)]
print("\nRESULT", "PASS: the rule predicts the dark rows on every case" if not weak else f"NOT SHOWN on {[w[0] for w in weak]}")

# Real fades (Mail 2591, 2614): the film's own frames with a plate caught part way through its fade, nothing over it.
# Two questions of the same pixels. Does each cell still hold its own colour, scaled towards the background like the rest?
# Does the engine's rule still predict the rows at that opacity? The fitted opacity is from the frame, not assumed. The
# control flattens each row to its first colour in the mirror, which the colour test must fail. Lossy 720p, as above: at
# this opacity and CRF the row detail is close to the encoder's floor, so read the seam-only r as weak evidence.
fades_kept = []
for label, rel, plate, f, x0, x1, y0, y1 in [
    ("1/z, residues", "12_residues/frame-0130.png", SMALL[1.5], "1/z", 655, 895, 330, 570),
    ("bump, continuation", "14_continuation/frame-0003.png", OPENING, "bump", 100, 330, 148, 270),
]:
    print(f"\nA real fade: {rel}, {label}, window {x0}..{x1} x {y0}..{y1}.")
    frame = np.asarray(Image.open(BASE / rel).convert("RGB"), dtype=float)[y0:y1, x0:x1] / 255.0
    bg = em.decode(np.array(em.BACKGROUND))
    colours = plate.colours(em.FUNCS[f], 1.0)
    stack = em.Stack(H, plate.top, plate.height, plate.rows, 0.5)
    full, _ = em.composite(plate, stack, colours, x0, x1, y0, y1, H)
    a = em.decode(full) - bg
    b = em.decode(frame) - bg
    alpha = float(np.sum(a * b) / np.sum(a * a))
    scaled = em.encode(bg + a * alpha)
    direct, _ = em.composite(plate, stack, colours, x0, x1, y0, y1, H, opacity=alpha)
    flat_colours = np.repeat(colours[:, :1], colours.shape[1], axis=1)
    flat, _ = em.composite(plate, stack, flat_colours, x0, x1, y0, y1, H, opacity=alpha)
    rms = lambda u, v: float(np.sqrt(np.mean(((u - v) * 255.0) ** 2)))
    series = em.luma(frame * 255.0).mean(axis=1)
    predicted_rows = em.luma(direct * 255.0).mean(axis=1)
    seam_clean, _ = em.composite(plate, em.Stack(H, plate.top, plate.height, plate.rows, 0.0, 1e-4), colours, x0, x1, y0, y1, H, opacity=alpha)
    clean_rows = em.luma(seam_clean * 255.0).mean(axis=1)
    fade_seam_r = em.correlation(em.residual(series - clean_rows), em.residual(predicted_rows - clean_rows))
    print(f"fitted opacity {alpha:.3f}")
    print(f"each pixel against the mirror at that opacity: rms {rms(frame, direct):.2f} of 255. Against the same colours scaled from the full picture: {rms(frame, scaled):.2f}")
    print(f"control, each row flattened to its first colour: rms {rms(frame, flat):.2f}")
    print(f"row residual, frame rms {em.stats(series)[1]:.3f} luma, mirror {em.stats(predicted_rows)[1]:.3f}; seam-only r {fade_seam_r:+.3f}")
    fades_kept.append(rms(frame, direct) * 3.0 < rms(frame, flat))
print("\nRESULT", "PASS: these frames keep their cell colours through the fade" if all(fades_kept) else "NOT SHOWN")
