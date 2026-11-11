"""final995_cc2_repair_mirror.py: Python clearance mirror for CC2's scratch repair of chapter 09
(boundary.rs cd1900aa..., the latest native844/final969 source). Read-only; compiles and renders nothing.

It models the label boxes and lines in scene units (270 native 4K px per scene unit) and checks, for every
radius the moving circles pass through (a fine grid plus each 60 fps frame value of the linear tweens):
  A. 09_1 shrink (SMALL_R 0.18 -> MID_R 0.12 in 0.7 s, then -> SHRUNK_R 0.045 in 1.0 s):
     the base layout (a fixed at A_AT, rho at (r + 0.06, -0.12)) and the repair (both names ride outside
     the small circle's line: a above, rho to the right).
  B. 09_7 (disk about 0 grows to 1) and 09_9 (disk about 1 grows to sqrt 2): a's name at (0.18, -0.2)
     from the centre, base (always shown) and repair (hidden while the line is near).
  C. 09_9's other plate cards, the M1 thin space, and the shrunk circle against the hole ring.
It exits non-zero if the repair fails any check, or if the base unexpectedly passes the checks it fails.
"""
import math
import sys

PX = 270.0
AA = 1.5 / PX                       # antialiasing allowance added to every drawn line edge
CLEAR = 0.035                       # the least gap the file's tests keep (about 9.5 native px)
CU = 4.0 / 1.8                      # close_up() scene units per z
PU = 5.6 / 5.0                      # picture() scene units per z
LOOP_R, BOUND_R, SMALL_R, MID_R, SHRUNK_R, HOLE = 0.5, 0.3, 0.18, 0.12, 0.045, 0.07
A_AT = (0.0, 0.09)
SMALL_HALF, DASH_HALF, HOLE_HALF, LOOP_HALF, PIC_HALF = 0.02, 0.015, 0.015, 0.02, 0.025
DASH_INK_IN = 0.2906 * CU           # handoff753's native measurement of where the dash ink starts
# ink half sizes (scene units): a at 20 is 24 x 25 native px; rho at 22 is 25 x 42 (handoff753), widened
# to 30 x 42 for CC2's own tile reading (29 x 38); a at 28 scaled from a at 20, widened to the tile reading
A20 = (12.0 / PX, 12.5 / PX)
RHO22 = (15.0 / PX, 21.0 / PX)
A28 = (17.5 / PX, 19.0 / PX)
# the repair's constants (boundary.rs, proposed)
RIDE_GAP = 0.045
A_RIDE = SMALL_HALF + RIDE_GAP + 12.5 / PX
RHO_RIDE = SMALL_HALF + RIDE_GAP + 15.0 / PX
A_NAME_FROM_CENTRE = (0.18, -0.2)
A_NAME_HIDDEN = (0.11, 0.43)
A_NAME_EASE = 0.1

fails = []


def check(ok, what):
    if not ok:
        fails.append(what)


def box(c, half):
    return (c[0] - half[0], c[0] + half[0], c[1] - half[1], c[1] + half[1])


def radial(b):
    """Least and greatest distance from the origin (the circle's centre) of a box not holding it."""
    x0, x1, y0, y1 = b
    nx = 0.0 if x0 <= 0.0 <= x1 else min(abs(x0), abs(x1))
    ny = 0.0 if y0 <= 0.0 <= y1 else min(abs(y0), abs(y1))
    fx, fy = max(abs(x0), abs(x1)), max(abs(y0), abs(y1))
    return math.hypot(nx, ny), math.hypot(fx, fy)


def ring_gap(b, radius, half):
    """Gap between a box and a circle line of centre radius `radius` and half width `half` (+AA)."""
    lo, hi = radial(b)
    h = half + AA
    return max(lo - (radius + h), (radius - h) - hi)


def box_gap(p, q):
    dx = max(q[0] - p[1], p[0] - q[1], 0.0)
    dy = max(q[2] - p[3], p[2] - q[3], 0.0)
    return math.hypot(dx, dy) if (dx or dy) else -1.0


def px(x):
    return x * PX


def shrink_radii():
    grid = [SHRUNK_R + (SMALL_R - SHRUNK_R) * k / 4000 for k in range(4001)]
    frames = [SMALL_R + (MID_R - SMALL_R) * k / 42 for k in range(43)]
    frames += [MID_R + (SHRUNK_R - MID_R) * k / 60 for k in range(61)]
    return sorted(set(grid + frames), reverse=True)


def shrink_layout(r, repaired):
    if repaired:
        a = (0.0, r * CU + A_RIDE)
        rho = (r * CU + RHO_RIDE, 0.0)
    else:
        a = (A_AT[0] * CU, A_AT[1] * CU)
        rho = ((r + 0.06) * CU, -0.12 * CU)
    return box(a, A20), box(rho, RHO22)


def shrink_table(repaired):
    worst = {}
    rows = []
    for r in shrink_radii():
        ba, br = shrink_layout(r, repaired)
        R = r * CU
        g = {
            'a/circle': ring_gap(ba, R, SMALL_HALF),
            'rho/circle': ring_gap(br, R, SMALL_HALF),
            'a/dashes': DASH_INK_IN - radial(ba)[1],
            'rho/dashes': DASH_INK_IN - radial(br)[1],
            'a/hole': ring_gap(ba, HOLE, HOLE_HALF),
            'rho/hole': ring_gap(br, HOLE, HOLE_HALF),
            'a/loop heads': (LOOP_R * CU - 0.14) - radial(ba)[1],
            'rho/loop heads': (LOOP_R * CU - 0.14) - radial(br)[1],
            'a/rho': box_gap(ba, br),
        }
        for k, v in g.items():
            if k not in worst or v < worst[k][0]:
                worst[k] = (v, r)
        if any(abs(r - x) < 1e-12 for x in (SMALL_R, 0.16, 0.14, MID_R, 0.1, 0.08, 0.06, SHRUNK_R)):
            rows.append((r, g))
    return worst, rows


print('A. 09_1 shrink, gaps in native px (negative = overlap); radius grid 4001 + 104 frame values')
for repaired in (False, True):
    worst, rows = shrink_table(repaired)
    print('  %s' % ('repair (a above, rho right, riding the line)' if repaired else 'base (a at A_AT, rho at r + 0.06)'))
    keys = list(rows[0][1])
    print('    %-6s ' % 'r' + ' '.join('%11s' % k for k in keys))
    for r, g in rows:
        print('    %-6.3f ' % r + ' '.join('%11.1f' % px(g[k]) for k in keys))
    print('    least  ' + ' '.join('%11.1f' % px(worst[k][0]) for k in keys))
    bad = [k for k in keys if worst[k][0] <= CLEAR]
    print('    under CLEAR (%.1f px): %s' % (px(CLEAR), ', '.join('%s at r %.3f' % (k, worst[k][1]) for k in bad) or 'none'))
    if repaired:
        check(not bad, 'repair shrink: ' + ', '.join(bad))
    else:
        check('a/circle' in bad and 'rho/dashes' in bad, 'base shrink should fail a/circle and rho/dashes')

# the snap (M2): base rho is at RHO_AT before the updater and at (SMALL_R + 0.06, -0.12) on its first frame
snap = math.hypot((0.62 - 0.24) * CU, (-0.25 + 0.12) * CU)
print('  base: rho jumps %.0f native px on the updater\'s first frame (RHO_AT to (0.24, -0.12)); the repair'
      ' creates rho at the updater\'s own SMALL_R place, so it moves 0 px' % px(snap))
a_end = SHRUNK_R + A_RIDE / CU
print('  repair: a rests at (0, %.4f) z after the shrink, %.1f native px above A_AT' % (a_end, px((a_end - A_AT[1]) * CU)))


def shown(r):
    lo, hi = A_NAME_HIDDEN
    if r < lo:
        return min((lo - r) / A_NAME_EASE, 1.0)
    if r > hi:
        return min((r - hi) / A_NAME_EASE, 1.0)
    return 0.0


print()
print('B. 09_7 and 09_9: the growing circle (width 5) and a\'s name at (0.18, -0.2) from its centre')
a_name = box((A_NAME_FROM_CENTRE[0] * PU, A_NAME_FROM_CENTRE[1] * PU), A28)
lo_s, hi_s = radial(a_name)
print('  a\'s ink box spans %.4f..%.4f scene units from the centre (%.3f..%.3f z)' % (lo_s, hi_s, lo_s / PU, hi_s / PU))
for name, end, cap in (('09_7', 1.0, 'disk about 0 to 1'), ('09_9', math.sqrt(2.0), 'disk about 1 to sqrt 2')):
    near, crossing, partly = [], [], []
    for k in range(20001):
        r = 1e-3 + (end - 1e-3) * k / 20000
        g = ring_gap(a_name, r * PU, PIC_HALF)
        if g < CLEAR:
            near.append(r)
            check(shown(r) == 0.0, '%s: a shown %.3f at r %.4f, gap %.1f px' % (name, shown(r), r, px(g)))
        if g < 0:
            crossing.append(r)
        if shown(r) < 1.0:
            partly.append(r)
    check(shown(1e-3) == 1.0 and shown(end) == 1.0, '%s: a not fully shown at the start or end' % name)
    print('  %s (%s): base line within CLEAR of a for r %.3f..%.3f, through its ink for r %.3f..%.3f;'
          ' repair hides a for r %.2f..%.2f (eased to full by %.2f and from %.2f), shown 1 at r = 1e-3 and %.3f'
          % (name, cap, near[0], near[-1], crossing[0], crossing[-1], A_NAME_HIDDEN[0], A_NAME_HIDDEN[1],
             partly[0], partly[-1], end))
    # share of the tween: the growth is linear in time, so the hidden share is the radius share
    print('      hidden %.0f%% and partly shown %.0f%% of the growth (09_7 rt <= 3.0 s; 09_9 rt from the voice)'
          % (100 * (A_NAME_HIDDEN[1] - A_NAME_HIDDEN[0]) / end, 100 * (2 * A_NAME_EASE) / end))

print()
print('C. 09_9 cards on the plate while the circle about 1 grows to sqrt 2 (card = text + 0.5 by + 0.35, edge 2)')
# text ink allowances (scene units), generous: "1" at 22, "i" and "-i" at 22, "sqrt2 = 1.414" at 20
cards = {
    '"1" card (C(1.0, -0.62), line 1325)': ((0.0, -0.62), (0.10 + 0.5, 0.16 + 0.35)),
    'sqrt 2 card (C(0.95, 0.95), line 1344)': ((-0.05, 0.95), (1.10 + 0.5, 0.17 + 0.35)),
    '"i" card (i + (-0.42, 0.3))': ((-1.42, 1.3), (0.08 + 0.5, 0.16 + 0.35)),
    '"-i" card (-i + (-0.42, -0.3))': ((-1.42, -1.3), (0.16 + 0.5, 0.16 + 0.35)),
}
for label, (off, (w, h)) in cards.items():
    b = box((off[0] * PU, off[1] * PU), (w / 2 + 0.01, h / 2 + 0.01))
    lo, hi = radial(b)
    end = math.sqrt(2.0) * PU
    crossed = lo - (PIC_HALF + AA) < end
    print('  %-40s spans %.3f..%.3f z from 1: %s' % (label, lo / PU, hi / PU,
          'crossed by the growing line' if crossed else 'clear of the final line by %.0f px' % px(lo - end - PIC_HALF - AA)))
print('  z order: the circle is rooted by its fade at line 1347; the sqrt 2 card fades in after it in the same call,'
      ' so the line already passes under that card; the "1" card (faded in at 1328) is under the line until the'
      ' repair re-roots it after 1347')

print()
print('D. M1 thin space: 3 mu = 3/18 em, em = size x 0.01 scene units')
for name, size, capped in (('ARC_TEX', 24.0, False), ('SUM_TEX', 23.0, True), ('SHRINK_TEX', 26.0, False)):
    w = 3.0 / 18.0 * size * 0.01
    print('  %-10s size %.0f: +%.4f scene units = %.1f native px%s' % (
        name, size, w, px(w), ' (proof_tex caps this formula at 5.1 wide, so it rescales by about 0.8%'
        ' and its ends stay put)' if capped else ' (centred at ROW_B, each end moves %.1f px)' % (px(w) / 2)))

print()
print('E. shrunk circle against the hole ring (base and repair alike; SHRUNK_R is not changed by the repair)')
inner = SHRUNK_R * CU - SMALL_HALF
outer = HOLE + HOLE_HALF
print('  drawn edges: green inner %.4f, red outer %.4f scene units: gap %.1f native px (before antialiasing)'
      % (inner, outer, px(inner - outer)))
for s in (0.05, 0.055, 0.06):
    print('  SHRUNK_R %.3f would leave %.1f px drawn gap' % (s, px(s * CU - SMALL_HALF - outer)))

print()
if fails:
    print('FAIL: ' + '; '.join(fails[:8]))
    sys.exit(1)
print('PASS: the repair clears every check; the base fails a/circle and rho/dashes as reviewed')
