"""final995_cc2_rust_test_mirror.py: Python emulation of the five new Rust tests in CC2's scratch repair
(final995-cc2-boundary-repair/boundary.rs), with the repair's constants, the test module's PX/A_HALF/CLEAR and the
same string slicing as include_str!. It also runs each test on the base (cd1900aa) to show which fail there.
Read-only; this is not the Rust test run, which root performs.
"""
import math
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPAIR = HERE / 'final995-cc2-boundary-repair' / 'boundary.rs'
BASE = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/docs/video-program/third-wave-20261024/'
            'opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/'
            'complex-boundary-repair752/boundary.rs')
PX, CLEAR = 270.0, 0.035
A_HALF = (12.0 / PX, 12.5 / PX)
CU, PU = 4.0 / 1.8, 5.6 / 5.0
SMALL_R, SHRUNK_R, BOUND_R, HOLE = 0.18, 0.045, 0.3, 0.07
A_AT = (0.0, 0.09)
A_RIDE = 0.02 + 0.045 + 12.5 / 270.0
RHO_RIDE = 0.02 + 0.045 + 15.0 / 270.0
HIDDEN, EASE = (0.11, 0.43), 0.1
TEST_MARK = '\n#[cfg(test)]\nmod tests {'


def shrink_names(r):
    return (0.0, r + A_RIDE / CU), (r + RHO_RIDE / CU, 0.0)


def shown(r):
    return min(max(max(HIDDEN[0] - r, r - HIDDEN[1]) / EASE, 0.0), 1.0)


def box(c, half):
    return [(c[0] - half[0], c[1] - half[1]), (c[0] + half[0], c[1] - half[1]),
            (c[0] + half[0], c[1] + half[1]), (c[0] - half[0], c[1] + half[1])]


def to_segment(q, a, b):
    dx, dy = b[0] - a[0], b[1] - a[1]
    t = max(0.0, min(1.0, ((q[0] - a[0]) * dx + (q[1] - a[1]) * dy) / (dx * dx + dy * dy)))
    return math.hypot(q[0] - a[0] - t * dx, q[1] - a[1] - t * dy)


def reach(name):
    inmost = min(to_segment((0.0, 0.0), name[j], name[(j + 1) % 4]) for j in range(4))
    outmost = max(math.hypot(*q) for q in name)
    return inmost, outmost


def box_gap(p, q):
    xs = lambda b: (min(v[0] for v in b), max(v[0] for v in b))
    ys = lambda b: (min(v[1] for v in b), max(v[1] for v in b))
    (px0, px1), (py0, py1), (qx0, qx1), (qy0, qy1) = xs(p), ys(p), xs(q), ys(q)
    dx, dy = max(qx0 - px1, px0 - qx1, 0.0), max(qy0 - py1, py0 - qy1, 0.0)
    return math.hypot(dx, dy)


def t_closing_bars(src):
    import re
    for name in ('ARC_TEX', 'SUM_TEX', 'SHRINK_TEX'):
        tex = re.search(r'pub const %s: &str = r"(.*)";' % name, src).group(1)
        before, after = tex.rsplit(r'\Big|', 1)
        if not (before.endswith(r'g\,dz\,') and after.startswith(r' \le ')):
            return 'fails on %s' % name
    return 'passes'


def t_shrink_geometry(_src):
    rho_half = (15.0 / PX, 21.0 / PX)
    for k in range(1001):
        r = SHRUNK_R + (SMALL_R - SHRUNK_R) * k / 1000
        a_at, rho_at = shrink_names(r)
        names = [box((a_at[0] * CU, a_at[1] * CU), A_HALF), box((rho_at[0] * CU, rho_at[1] * CU), rho_half)]
        for n in names:
            inmost, outmost = reach(n)
            if not (inmost > r * CU + 0.02 + CLEAR and outmost < BOUND_R * CU - 0.015 - CLEAR
                    and inmost > HOLE + 0.015 + CLEAR):
                return 'fails at r %.4f' % r
        if not box_gap(names[0], names[1]) > CLEAR:
            return 'fails a and rho at r %.4f' % r
    a_end = shrink_names(SHRUNK_R)[0]
    d = math.hypot((a_end[0] - A_AT[0]) * CU, (a_end[1] - A_AT[1]) * CU) * PX
    return 'passes (a ends %.2f native px from A_AT, under 4)' % d if d < 4.0 else 'fails at the end (%.2f px)' % d


def t_shrink_source(src):
    parts = src.split('// Second limit:')
    if len(parts) < 2:
        return 'fails: no marker'
    shrink = parts[1].split('// A finite picture')[0]
    for wanted in ('let [a_from, rho_from] = shrink_names(SMALL_R);', 'fade_out_all(stage, &leaving, 0.2)?;',
                   'fade_in_all(stage, &[bar, bar_name, ca_out, rho_out], 0.3)?;',
                   'let [a_at, rho_at] = shrink_names(r);'):
        if shrink.count(wanted) != 1:
            return 'fails on %s' % wanted
    if 'r + 0.06' in shrink:
        return 'fails: r + 0.06 still there'
    return 'passes (slice %d chars, %d markers in the file)' % (len(shrink), len(parts) - 1)


def t_growth_geometry(src):
    if 'fn a_name_shown' not in src:
        return 'fails (no a_name_shown: a is drawn at full opacity throughout both growths)'
    half = (17.5 / PX, 19.0 / PX)
    inmost, outmost = reach(box((0.18 * PU, -0.2 * PU), half))
    ends = (1.0, math.sqrt(2.0))
    hidden = 0
    for k in range(4001):
        r = 1e-3 + (ends[1] - 1e-3) * k / 4000
        if r * PU + 0.025 + CLEAR > inmost and r * PU - 0.025 - CLEAR < outmost:
            if not shown(r) < 1e-12:
                return 'fails at r %.4f' % r
            hidden += 1
    if not hidden or any(not shown(r) > 1 - 1e-12 for r in (1e-3,) + ends):
        return 'fails at the ends'
    return 'passes (%d of 4001 radii in the hidden zone)' % hidden


def t_growth_source(src):
    film = src.split(TEST_MARK)[0]
    if film.count('a.set_opacity(a_name, a_name_shown(r), true);') != 2:
        return 'fails: opacity calls'
    i = film.find('fade_in_all(stage, &[unit, reach1, reach_name[0], reach_name[1]], 0.3)?;')
    j = film.find('for m in one_name {\n            stage.add_to_scene(m)?;')
    k = film.find('let grow = stage.add_value_tracker(1e-3);')
    return 'passes' if 0 <= i < j < k else 'fails: order %d %d %d' % (i, j, k)


tests = [('the_closing_bars_keep_clear_of_dz', t_closing_bars),
         ('the_shrinking_circle_never_crosses_a_or_rho', t_shrink_geometry),
         ('the_shrinking_names_start_where_the_shrink_puts_them', t_shrink_source),
         ('the_growing_disks_never_draw_through_a', t_growth_geometry),
         ('the_growing_disks_pass_under_the_cards_and_round_a', t_growth_source)]
repair, base = REPAIR.read_text(), BASE.read_text()
ok = True
for name, fn in tests:
    r = fn(repair)
    b = fn(base) if fn is not t_shrink_geometry else 'n/a (pure geometry of the repair constants)'
    ok &= r.startswith('passes')
    print('%-55s repair: %s\n%-55s base:   %s' % (name, r, '', b))
sys.exit(0 if ok else 1)
