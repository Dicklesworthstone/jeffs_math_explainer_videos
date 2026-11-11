"""Independent label-path mirror for CC2's boundary candidate 511f4492 (read-only; no compile/render).

Close-up (09_1 shrink): close_up() = Plane(centre (4.4,-0.95), width 4.0, (-0.9,0.9)^2) -> 2.2222 scene/plane unit.
Picture (09_7, 09_9): picture() = Plane(centre (-3.3,-0.2), width 5.6, (-2.5,2.5)^2) -> 1.12 scene/plane unit.
Line widths: w/100 scene units (memory: width-3 dashes measure 8 native px).
Ink half-sizes (scene units, 270 native px per unit): a@20 (12, 12.5) px measured (base test A_HALF);
rho@22 (12.5, 21) px measured (repair752 notes), CC2 widened (15, 21); a@28 (16.8, 17.5) px scaled, CC2 (17.5, 19).

Reports, over every radius each label path passes (step 1e-4 plane units):
  * shrink: least gap (scene) from a's and rho's ink to the small circle's line, the dashes, the hole ring, the loop,
    and each other, for CANDIDATE paths and BASE paths (base a fixed at A_AT, base rho at (r+0.06,-0.12));
  * shrink: where a's ink sits relative to the point it names (centre) and to the circle's line;
  * growth: for both disks, every radius where the line comes within CLEAR of a's ink, and a_name_shown there;
    the opacity while the line is near but not within CLEAR; the "1" card's radial span in 09_9.
usage: python3 -I -B label-paths01.py
"""
import math

PX, CLEAR = 270.0, 0.035
CU = 4.0 / 1.8
PU = 5.6 / 5.0
SMALL_R, MID_R, SHRUNK_R, BOUND_R, LOOP_R, HOLE = 0.18, 0.12, 0.045, 0.3, 0.5, 0.07
A_AT, RHO_AT = (0.0, 0.09), (0.62, -0.25)
A_RIDE = 0.02 + 0.045 + 12.5 / 270.0
RHO_RIDE = 0.02 + 0.045 + 15.0 / 270.0
HIDDEN, EASE = (0.11, 0.43), 0.1
A_FROM_CENTRE = (0.18, -0.2)


def box(c, half):
    (x, y), (w, h) = c, half
    return (x - w, y - h, x + w, y + h)


def radial_span(b):
    """Least and greatest distance from the origin to an axis-aligned box (scene units)."""
    x0, y0, x1, y1 = b
    nx = 0.0 if x0 <= 0.0 <= x1 else min(abs(x0), abs(x1))
    ny = 0.0 if y0 <= 0.0 <= y1 else min(abs(y0), abs(y1))
    fx, fy = max(abs(x0), abs(x1)), max(abs(y0), abs(y1))
    return math.hypot(nx, ny), math.hypot(fx, fy)


def ring_gap(b, radius, half_width):
    lo, hi = radial_span(b)
    if lo - half_width > radius:
        return lo - half_width - radius
    if hi + half_width < radius:
        return radius - hi - half_width
    return -1.0  # the line runs through the box's radial span (crosses or touches the ink box)


def box_gap(a, b):
    dx = max(b[0] - a[2], a[0] - b[2], 0.0)
    dy = max(b[1] - a[3], a[1] - b[3], 0.0)
    return math.hypot(dx, dy)


def shown(r):
    lo, hi = HIDDEN
    return min(max(max(lo - r, r - hi) / EASE, 0.0), 1.0)


A20 = (12.0 / PX, 12.5 / PX)
RHO_M, RHO_W = (12.5 / PX, 21.0 / PX), (15.0 / PX, 21.0 / PX)
A28_M, A28_W = (16.8 / PX, 17.5 / PX), (17.5 / PX, 19.0 / PX)


def shrink_paths(kind, r):
    if kind == "candidate":
        a = (0.0, r * CU + A_RIDE)
        rho = (r * CU + RHO_RIDE, 0.0)
    else:
        a = (A_AT[0] * CU, A_AT[1] * CU)
        rho = ((r + 0.06) * CU, -0.12 * CU)
    return a, rho


print("== 09_1 shrink, radius SMALL_R 0.18 -> MID_R 0.12 -> SHRUNK_R 0.045 (plane units) ==")
for kind in ("base", "candidate"):
    for rho_half, tag in ((RHO_M, "rho measured 25x42"), (RHO_W, "rho widened 30x42")):
        worst = {}
        r = SMALL_R
        while r >= SHRUNK_R - 1e-12:
            a_c, rho_c = shrink_paths(kind, r)
            ab, rb = box(a_c, A20), box(rho_c, rho_half)
            for name, b in (("a", ab), ("rho", rb)):
                for obj, rad, hw in (("small circle", r * CU, 0.02), ("dashes", BOUND_R * CU, 0.015),
                                     ("hole ring", HOLE, 0.015), ("loop", LOOP_R * CU, 0.02)):
                    g = ring_gap(b, rad, hw)
                    key = (name, obj)
                    if key not in worst or g < worst[key][0]:
                        worst[key] = (g, r)
            g = box_gap(ab, rb)
            if ("a", "rho") not in worst or g < worst[("a", "rho")][0]:
                worst[("a", "rho")] = (g, r)
            r -= 1e-4
        print(f"-- {kind}, {tag}")
        for (name, obj), (g, rr) in sorted(worst.items()):
            flag = "CROSSES" if g < 0 else ("< CLEAR" if g < CLEAR else "ok")
            print(f"   {name:3s} vs {obj:12s}: least gap {g:+.4f} scene ({g * PX:+6.1f} px 4K) at r={rr:.4f}  {flag}")

print("\n-- candidate a: distance of a's ink centre from the centre a it names, vs from the circle's line")
for r in (SMALL_R, 0.15, MID_R, 0.08, SHRUNK_R):
    a_c, _ = shrink_paths("candidate", r)
    d_centre = a_c[1]
    d_line = a_c[1] - r * CU
    print(f"   r={r:.3f}: a centre {d_centre:.3f} scene above the centre ({d_centre * PX:.0f} px 4K), "
          f"{d_line:.3f} above the circle's top ({d_line * PX:.0f} px); ratio {d_centre / d_line:.1f}")
print(f"   base a (fixed A_AT): centre {A_AT[1] * CU:.3f} scene above the centre; inside the small circle until "
      f"r*unit < {A_AT[1] * CU + A20[1] + 0.02:.3f} (r < {(A_AT[1] * CU + A20[1] + 0.02) / CU:.4f})")
a_end, _ = shrink_paths("candidate", SHRUNK_R)
print(f"   candidate a at SHRUNK_R is {(a_end[1] - A_AT[1] * CU) * PX:.1f} px (4K) above A_AT")
print(f"   candidate a at SMALL_R is {(shrink_paths('candidate', SMALL_R)[0][1] - A_AT[1] * CU) * PX:.1f} px (4K) "
      f"above where base a stood when it faded out")
print(f"   candidate rho at SMALL_R stands {math.hypot(SMALL_R * CU + RHO_RIDE - RHO_AT[0] * CU, RHO_AT[1] * CU) * PX:.0f}"
      f" px (4K) from where base rho stood at RHO_AT")

print("\n== 09_7 / 09_9 growth: a's name (size 28) at the same offset from each disk's centre ==")
for half, tag in ((A28_M, "a@28 scaled 33.6x35"), (A28_W, "a@28 widened 35x38")):
    c = (A_FROM_CENTRE[0] * PU, A_FROM_CENTRE[1] * PU)
    b = box(c, half)
    lo, hi = radial_span(b)
    need_lo, need_hi = (lo - 0.025 - CLEAR) / PU, (hi + 0.025 + CLEAR) / PU
    touch_lo, touch_hi = (lo - 0.025) / PU, (hi + 0.025) / PU
    print(f"-- {tag}: ink spans {lo / PU:.4f}..{hi / PU:.4f} plane from the centre; line (width 5) within CLEAR for "
          f"r in [{need_lo:.4f}, {need_hi:.4f}], touching for r in [{touch_lo:.4f}, {touch_hi:.4f}]; "
          f"HIDDEN {HIDDEN} covers: {HIDDEN[0] <= need_lo and need_hi <= HIDDEN[1]}")
    worst = max(shown(r / 1e4) for r in range(int(need_lo * 1e4), int(need_hi * 1e4) + 1))
    print(f"   max shown inside that band: {worst:.3g}")
    print(f"   shown at the band edges + margin: r={need_lo - 0.01:.3f}->{shown(need_lo - 0.01):.2f}, "
          f"r={need_hi + 0.01:.3f}->{shown(need_hi + 0.01):.2f}")
print(f"   shown at 1e-3 {shown(1e-3):.2f}, at 1.0 {shown(1.0):.2f}, at sqrt2 {shown(math.sqrt(2)):.2f}")
print(f"   fade-out runs r 0.01..0.11 ({0.10 / 1.0:.0%} of the 09_7 growth to 1; {0.10 / math.sqrt(2):.0%} of 09_9's)")

# 09_9's "1" card: carded("1", 22, p.p(1, -0.62)), card = text + (0.5, 0.35). Text "1" at 22: about 0.5 x 0.66 em,
# em = 22*0.01 scene (memory: width ~ CMR10 ems x size x 0.01): an estimate, not measured.
em = 22 * 0.01
w, h = 0.5 * em + 0.5, 0.66 * em + 0.35
cb = box((0.0, -0.62 * PU), (w / 2, h / 2))
lo, hi = radial_span(cb)
print(f"\n09_9 '1' card (estimated {w:.2f} x {h:.2f} scene): spans r {lo / PU:.3f}..{hi / PU:.3f} plane from the centre; "
      f"the disk's line passes under it for r in about [{(lo - 0.025) / PU:.3f}, {(hi + 0.025) / PU:.3f}]")
ab = box((A_FROM_CENTRE[0] * PU, A_FROM_CENTRE[1] * PU), A28_W)
print(f"   gap from a's ink (widened) to the card: {box_gap(ab, cb):.3f} scene")
