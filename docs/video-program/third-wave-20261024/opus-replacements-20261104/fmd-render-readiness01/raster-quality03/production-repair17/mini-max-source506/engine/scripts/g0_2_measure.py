#!/usr/bin/env python3
"""G0-2 look-study measurements (fm-k77) — the empirical half of the evidence.

Re-derives every measured number in `docs/g0/G0-2-look-study-ratification.md`
from the one-time Reference captures and from the Reference's own Python. The
analytic half of the evidence is the Reference's GLSL, read as source and quoted
in the note; this script is what makes the other half checkable.

Two of these measurements exist because a first attempt got them wrong (note
L2): the glow profile must avoid the neighbouring dots, and the edge profile
must be reconstructed from a CURVED boundary — an axis-aligned edge lands at one
sub-pixel phase, so per-scanline ramps measure the phase, not the profile.

Usage (needs the Reference import closure and a GL context for -3/-6; see
scripts/capture_reference_imagery.py for the recipe):

    xvfb-run -a refenv/bin/python scripts/g0_2_measure.py
"""

import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
REF = os.path.join(HERE, "manim_ref")
CAP = os.path.join(HERE, "..", "gallery", "reference_captures")

# camera.py:176-177 — pixel_size = FRAME_WIDTH / pixel_width at 1920x1080.
PX_PER_UNIT = 1920.0 / (8.0 * 16 / 9)  # = 135.0


def load(name):
    path = os.path.join(CAP, f"{name}.png")
    if not os.path.exists(path):
        raise SystemExit(
            f"missing capture {path}\nRun scripts/capture_reference_imagery.py first."
        )
    return np.asarray(Image.open(path)).astype(np.float64) / 255.0


def srgb_to_linear(c):
    c = np.asarray(c, float)
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def linear_to_srgb(c):
    c = np.asarray(c, float)
    return np.where(c <= 0.0031308, c * 12.92, 1.055 * c ** (1 / 2.4) - 0.055)


def smoothstep(x):
    s = np.clip(x, 0, 1)
    return s * s * (3 - 2 * s)


# ---------------------------------------------------------------- L1: the AA band
def measure_aa_profile():
    """Reconstruct the continuous edge profile from the captured circle."""
    print("=" * 74)
    print("L1  AA PROFILE — reconstructed from a curved boundary")
    print("=" * 74)
    img = load("gradient_fills")
    lum = img[..., :3].sum(axis=2)
    bg = lum[5, 5]
    obj = np.abs(lum - bg) > 0.03
    cols = np.where(obj.any(axis=0))[0]
    gaps = np.where(np.diff(cols) > 5)[0]
    c_start = cols[gaps[0] + 1]
    ys, xs = np.where(obj[:, c_start:])
    cy0 = (ys.min() + ys.max()) / 2
    cx0 = c_start + (xs.min() + xs.max()) / 2
    R0 = ((ys.max() - ys.min()) + (xs.max() - xs.min())) / 4

    def bilinear(y, x):
        yi, xi = int(np.floor(y)), int(np.floor(x))
        fy, fx = y - yi, x - xi
        return (
            lum[yi, xi] * (1 - fy) * (1 - fx)
            + lum[yi + 1, xi] * fy * (1 - fx)
            + lum[yi, xi + 1] * (1 - fy) * fx
            + lum[yi + 1, xi + 1] * fy * fx
        )

    # half-coverage contour -> least-squares circle
    pts = []
    for th in np.linspace(0, 2 * np.pi, 720, endpoint=False):
        rr = np.arange(R0 - 6, R0 + 6, 0.02)
        vals = np.array(
            [bilinear(cy0 + r * np.sin(th), cx0 + r * np.cos(th)) for r in rr]
        )
        if abs(vals[0] - vals[-1]) < 0.1:
            continue
        half = 0.5 * (vals[0] + vals[-1])
        idx = np.where(np.diff(np.sign(vals - half)) != 0)[0]
        if not len(idx):
            continue
        i = idx[-1]
        t = (half - vals[i]) / (vals[i + 1] - vals[i] + 1e-12)
        r_c = rr[i] + t * (rr[i + 1] - rr[i])
        pts.append((cy0 + r_c * np.sin(th), cx0 + r_c * np.cos(th)))
    pts = np.array(pts)
    A = np.c_[2 * pts[:, 1], 2 * pts[:, 0], np.ones(len(pts))]
    sol, *_ = np.linalg.lstsq(A, pts[:, 1] ** 2 + pts[:, 0] ** 2, rcond=None)
    cx, cy = sol[0], sol[1]
    R = np.sqrt(sol[2] + cx**2 + cy**2)
    resid = np.abs(np.hypot(pts[:, 1] - cx, pts[:, 0] - cy) - R)
    print(
        f"  fitted circle R={R:.3f} px, contour residual median {np.median(resid):.4f} px"
    )

    yy, xx = np.mgrid[0 : lum.shape[0], 0 : lum.shape[1]]
    d = np.hypot(yy - cy, xx - cx) - R
    inner = np.median(lum[(d > -4.5) & (d < -3.0)])
    outer = np.median(lum[(d > 3.0) & (d < 4.5)])
    band = np.abs(d) < 5.0
    cov = (lum[band] - outer) / (inner - outer)
    dd = d[band]

    edges = np.arange(-1.0, 1.01, 0.25)
    pd, pc = [], []
    for i in range(len(edges) - 1):
        m = (dd >= edges[i]) & (dd < edges[i + 1])
        if m.sum() > 30:
            pd.append(0.5 * (edges[i] + edges[i + 1]))
            pc.append(cov[m].mean())
    pd, pc = np.array(pd), np.array(pc)
    print("  coverage vs signed distance:")
    for a, b in zip(pd, pc):
        print(f"    d={a:+.3f}  {b:.4f}")

    best = min(
        (
            (W, off, np.sqrt(((smoothstep(0.5 - (pd - off) / W) - pc) ** 2).mean()))
            for W in np.arange(0.6, 3.01, 0.01)
            for off in np.arange(-0.8, 0.81, 0.01)
        ),
        key=lambda t: t[2],
    )
    print(
        f"  best smoothstep: band {best[0]:.3f} px, offset {best[1]:+.3f} px, RMS {best[2]:.5f}"
    )
    print("  declared anti_alias_width = 1.5 px (vectorized_mobject.py:96)")


# ------------------------------------------------------------- L3: fill AA levels
def measure_fill_levels():
    """The Reference's fill AA is a 2x2 box downsample: five coverage levels."""
    print()
    print("=" * 74)
    print("L3  FILL AA — distinct coverage levels on a BARE fill edge")
    print("=" * 74)
    sys.path.insert(0, REF)
    import manimlib as m

    s = m.Scene()
    c = m.Circle(radius=2.5)
    c.set_fill(m.BLUE_D, opacity=1.0)
    c.set_stroke(width=0)  # nothing covers the fill boundary
    s.add(c.center())
    s.update_frame(force_draw=True)
    im = np.asarray(s.get_image()).astype(np.float64) / 255.0
    lum = im[..., :3].sum(axis=2)
    bg = lum[5, 5]
    ys, xs = np.where(np.abs(lum - bg) > 0.02)
    cy = (ys.min() + ys.max()) / 2
    cx = (xs.min() + xs.max()) / 2
    R = ((ys.max() - ys.min()) + (xs.max() - xs.min())) / 4
    yy, xx = np.mgrid[0 : lum.shape[0], 0 : lum.shape[1]]
    d = np.hypot(yy - cy, xx - cx) - R
    inner = np.median(lum[(d > -3.5) & (d < -2.5)])
    outer = np.median(lum[(d > 2.5) & (d < 3.5)])
    cov = ((lum - outer) / (inner - outer))[np.abs(d) < 4]
    u = np.unique(np.round(cov, 4))
    mid = np.sort(u[(u > 0.02) & (u < 0.98)])
    print(f"  circle R={R:.1f} px")
    print(
        f"  distinct intermediate levels: {len(mid)}  ->  {np.array2string(mid[:8], precision=4)}"
    )
    print("  2x2 box supersampling predicts {0, 1/4, 1/2, 3/4, 1}")


# -------------------------------------------------------------- L4: the colour model
def measure_colour():
    print()
    print("=" * 74)
    print("L4  COLOUR — interpolate_color / average_color, and the linear-light gap")
    print("=" * 74)
    sys.path.insert(0, REF)
    from manimlib.utils.color import average_color, color_to_rgb, interpolate_color

    c1, c2 = "#1C758A", "#FFFF00"  # BLUE_E, YELLOW
    r1, r2 = np.array(color_to_rgb(c1)), np.array(color_to_rgb(c2))
    worst_naive = worst_sq = 0.0
    for a in (0.0, 0.25, 0.5, 0.75, 1.0):
        ref = np.array(color_to_rgb(interpolate_color(c1, c2, a)))
        worst_naive = max(worst_naive, np.abs(ref - ((1 - a) * r1 + a * r2)).max())
        worst_sq = max(
            worst_sq, np.abs(ref - np.sqrt((1 - a) * r1**2 + a * r2**2)).max()
        )
    print(f"  interpolate_color vs sqrt-of-squares : max err {worst_sq:.6f}")
    print(f"  interpolate_color vs naive lerp      : max err {worst_naive:.6f}")
    avg = np.array(color_to_rgb(average_color(c1, c2)))
    print(
        f"  average_color vs per-channel RMS     : max err "
        f"{np.abs(avg - np.sqrt((r1**2 + r2**2) / 2)).max():.6f}"
    )

    print("  kept gamma-2 form vs a true linear-light lerp:")
    worst = 0.0
    for a in np.linspace(0, 1, 11):
        sq = np.sqrt((1 - a) * r1**2 + a * r2**2)
        ll = linear_to_srgb((1 - a) * srgb_to_linear(r1) + a * srgb_to_linear(r2))
        e = np.abs(sq - ll).max()
        worst = max(worst, e)
        if a in (0.1, 0.2, 0.5, 0.9):
            print(f"    a={a:.1f}  max channel err {e:.4f}")
    print(f"  worst case across the ramp: {worst:.4f} = {worst * 255:.1f}/255")


# ------------------------------------------------------------------- L7: the glow
def measure_glow():
    print()
    print("=" * 74)
    print("L7  GLOW — radial falloff of the CENTRE dot (neighbours excluded)")
    print("=" * 74)
    g = load("glow")
    h, w = g.shape[:2]
    cy, cx = h // 2, w // 2
    bg = g[5, 5, :3].copy()
    maxr = 185  # the neighbouring dots sit at +-270 px
    prof = np.array(
        [
            np.mean(
                [
                    g[int(round(cy + r * np.sin(t))), int(round(cx + r * np.cos(t))), 0]
                    - bg[0]
                    for t in np.linspace(0, 2 * np.pi, 16, endpoint=False)
                ]
            )
            for r in range(maxr)
        ]
    )
    norm = prof / prof[0]
    R = 1.5 * PX_PER_UNIT
    rr = np.arange(maxr, dtype=float)
    m = (rr > 3) & (norm > 0.02) & (rr < R * 0.98)
    x = 1.0 - rr[m] / R
    k, b = np.polyfit(np.log(x), np.log(norm[m]), 1)
    print(f"  GlowDot radius 1.5 u = {R:.1f} px")
    print(f"  fit I/I0 = (1-r/R)^k  ->  k = {k:.4f}")
    for name, model in (
        ("(1-r/R)^2", lambda z: np.clip(1 - z / R, 0, None) ** 2),
        ("(1-r/R)^1.5", lambda z: np.clip(1 - z / R, 0, None) ** 1.5),
        ("exp(-3(r/R)^2)", lambda z: np.exp(-3 * (z / R) ** 2)),
    ):
        print(f"    {name:<16} max|err| = {np.abs(model(rr) - norm).max():.4f}")
    print("  source: true_dot/frag.glsl:26, glow_factor = 2.0 (dot_cloud.py:171)")


# ---------------------------------------------------------- L8: cubic -> quadratic
def measure_cubic_to_quad():
    print()
    print("=" * 74)
    print("L8  CUBIC->QUAD — the Reference's fixed two-quadratic approximation")
    print("=" * 74)
    sys.path.insert(0, REF)
    from manimlib.utils.bezier import get_quadratic_approximation_of_cubic as approx

    def cubic(p, t):
        t = t[:, None]
        a, b, c, d = p
        return (
            (1 - t) ** 3 * a
            + 3 * (1 - t) ** 2 * t * b
            + 3 * (1 - t) * t**2 * c
            + t**3 * d
        )

    def quad(p, t):
        t = t[:, None]
        a, hh, b = p
        return (1 - t) ** 2 * a + 2 * (1 - t) * t * hh + t**2 * b

    cases = {
        "quarter-circle-ish": [[1, 0, 0], [1, 0.5523, 0], [0.5523, 1, 0], [0, 1, 0]],
        "gentle S": [[-1, 0, 0], [-0.4, 0.8, 0], [0.4, -0.8, 0], [1, 0, 0]],
        "half-circle-ish": [[1, 0, 0], [1, 1.3333, 0], [-1, 1.3333, 0], [-1, 0, 0]],
        "near-cusp": [[0, 0, 0], [1.4, 0, 0], [-1.4, 0, 0], [0, 0.2, 0]],
        "strong C": [[-1, -1, 0], [-1, 1.6, 0], [1, 1.6, 0], [1, -1, 0]],
    }
    ts = np.linspace(0, 1, 4001)
    print(f"  {'case':<20}{'n quads':>9}{'max dev (units)':>18}{'max dev (px)':>15}")
    for name, pts in cases.items():
        p = [np.array(x, float) for x in pts]
        out = np.array(approx(*p)).reshape(-1, 3)
        n = (len(out) - 1) // 2
        Q = np.vstack([quad(out[2 * i : 2 * i + 3], ts) for i in range(n)])
        C = cubic(p, ts)
        dev = np.sqrt(((C[:, None, :] - Q[None, :, :]) ** 2).sum(-1)).min(axis=1).max()
        print(f"  {name:<20}{n:>9}{dev:>18.6f}{dev * PX_PER_UNIT:>15.2f}")
    print("  our default tolerance: 0.1 px (see the note, decision (f))")


if __name__ == "__main__":
    measure_aa_profile()
    measure_colour()
    measure_glow()
    measure_cubic_to_quad()
    measure_fill_levels()  # last: needs a GL context
