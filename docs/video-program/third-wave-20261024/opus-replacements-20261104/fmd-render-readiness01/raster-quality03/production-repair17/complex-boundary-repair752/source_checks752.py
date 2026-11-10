"""Bounded source checks for repair 752 of chapter 09 (boundary.rs), against the final553 base.

Run from anywhere: python3 -I -B source_checks752.py. It reads the base in root-boundary552-source/, the repaired
boundary.rs beside this file and the base's script.tsv. It compiles nothing and renders nothing. It checks:
  1. both files are at their sealed hashes;
  2. construct() is unchanged outside the three head calls: the same timing calls with the same arguments, the same
     wait phrases in script order, the same text from 09_2 on, and RHO_AT and the shrink updater unchanged;
  3. the only base lines removed are the twelve this repair replaces, and the base's tests are kept verbatim;
  4. a Python mirror of the two new tests passes on the repair's constants and fails on the base's.
"""
import difflib
import hashlib
import math
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CRATE = HERE.parent / 'root-boundary552-source' / 'workspace' / 'explainer_corpus' / 'complex_analysis_visual'
BASE, REPAIR, SCRIPT = CRATE / 'src' / 'chapters' / 'boundary.rs', HERE / 'boundary.rs', CRATE / 'script.tsv'
BASE_SHA = 'bda7d931498b23bd3203cd0c6106de69dc0a8c5e160e738d698376322ff39e53'
REPAIR_SHA = sys.argv[1] if len(sys.argv) > 1 else None
TEST_MARK = '\n#[cfg(test)]\nmod tests {'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def construct(text):
    return text[text.index('fn construct(&mut self'):text.index(TEST_MARK)]


def calls(text, names):
    """Every call of `names` in `text`, whitespace collapsed, with its balanced argument list."""
    flat = re.sub(r'\s+', ' ', text)
    out = []
    for m in re.finditer(r'\b(%s)\(' % '|'.join(map(re.escape, names)), flat):
        depth, i = 1, m.end()
        while depth:
            depth += {'(': 1, ')': -1}.get(flat[i], 0)
            i += 1
        out.append(flat[m.start():i].replace('( ', '(').replace(', )', ')').replace(',)', ')'))
    return out


base_text, repair_text = BASE.read_text(), REPAIR.read_text()
assert sha(BASE) == BASE_SHA, 'base changed'
print('1. base %s; repair %s%s' % (sha(BASE), sha(REPAIR), '' if REPAIR_SHA is None else ' (pin given)'))
if REPAIR_SHA is not None:
    assert sha(REPAIR) == REPAIR_SHA, 'repair is not at the given hash'

TIMING = ['line.wait_for', 'line.wait_sentence', 'line.until', 'stage.play', 'stage.play_prepared', 'fade_in_all',
          'fade_out_all', 'pause', 'wait_until', 'kit.hold', 'tween_linear', 'stage.wait', 'say']
cb, cr = construct(base_text), construct(repair_text)
tb, tr = calls(cb, TIMING), calls(cr, TIMING)
assert tb == tr, [x for x in difflib.unified_diff(tb, tr, lineterm='')][:20]
heads_b, heads_r = calls(cb, ['side_heads']), calls(cr, ['side_heads'])
assert len(heads_b) == len(heads_r) == 3
assert cb.split('// 09_2: on a circle', 1)[1] == cr.split('// 09_2: on a circle', 1)[1]
spoken = next(line.split('\t', 1)[1] for line in SCRIPT.read_text().splitlines() if line.startswith('09_1\t'))
body = cr.split('// 09_1: divide by', 1)[1].split('// 09_2: on a circle', 1)[0]
phrases = [c.split('"')[1] for c in body.split('line.wait_for(')[1:]]
positions = [spoken.index(p) for p in phrases]
assert positions == sorted(positions) and len(phrases) >= 25
for fixed in ('pub const RHO_AT: C = C::new(0.62, -0.25);', 'a.move_to(rho, cp.p(C::new(r + 0.06, -0.12)), ORIGIN);',
              'let rho = kit.tex(stage, RHO_TEX, 22.0, output(), &[], cp.p(RHO_AT))?;'):
    assert base_text.count(fixed) == repair_text.count(fixed) == 1, fixed
print('2. construct: %d timing calls identical with their arguments; %d wait phrases in script order; 09_2 on'
      ' identical; rho unchanged; head calls %s -> %s' % (len(tb), len(phrases), heads_b, heads_r))

old, new = base_text.splitlines(), repair_text.splitlines()
removed = ['- ' + l for tag, i1, i2, _, _ in difflib.SequenceMatcher(None, old, new, autojunk=False).get_opcodes()
           if tag in ('replace', 'delete') for l in old[i1:i2]]
EXPECTED_REMOVED = [
    '    faded_line, flat, group, hero, input, line_style, marks, names_now, output, pause, pole, precisely_now, predict,',
    '    reading_window, say, tween_linear, value, wait_until, warn,',
    '/// Labels in the close-up: a, ρ, the polygons P and p.',
    'pub const POLYGON_NAMES_AT: [C; 2] = [C::new(0.445, 0.445), C::new(-0.445, -0.445)];',
    '/// A head on the middle of every side of the polygon `corners`, pointing the way it is walked.',
    'fn side_heads(stage: &mut Stage<\'_>, plane: &Plane, corners: &[C], counterclockwise: bool, color: Srgb) -> Vec<Mob> {',
    '            stage',
    '                .arena_mut()',
    '                .add(Polygon::new(mid_head(a, b, 0.12)).style(flat(color, 1.0)).build())',
    '        let outer_arrows = side_heads(stage, &cp, &polygon(LOOP_R, SIDES), true, hero());',
    '        let inner_cw = side_heads(stage, &cp, &polygon(SMALL_R, SIDES), false, output());',
    '        let inner_ccw = side_heads(stage, &cp, &polygon(SMALL_R, SIDES), true, output());',
]
assert [l[2:] for l in removed] == EXPECTED_REMOVED, [l[2:] for l in removed]
base_tests = base_text[base_text.index(TEST_MARK):].rstrip()
assert base_tests.endswith('}') and repair_text[repair_text.index(TEST_MARK):].startswith(base_tests[:-1])
print('3. removed base lines: exactly the %d expected; the base tests are kept verbatim' % len(removed))


# 4. Mirror of the two new tests. Label half sizes and pixels per scene unit are the test's own constants.
def const(text, name):
    return re.search(r'pub const %s: [^=]+= ([^;]+);' % name, text).group(1)


def number(text, name):
    expr = const(text, name)
    shift = re.fullmatch(r'1 << (\d+)', expr)       # SIDES = 1 << 3
    return float(1 << int(shift.group(1))) if shift else float(expr)


def point(text, name, index=None):
    raw = const(text, name)
    pairs = re.findall(r'C::new\(([-0-9.]+), ([-0-9.]+)\)', raw)
    x, y = pairs[0 if index is None else index]
    return float(x), float(y)


def srgb(h):
    return tuple(int(h[i:i + 2], 16) / 255 for i in (1, 3, 5))


HERO, GREEN, GROUND = srgb('#FFD866'), srgb('#83C167'), srgb('#0E1116')
palette = (HERE.parent / 'root-boundary552-source' / 'repo-view' / 'crates' / 'explainer_kit' / 'src' / 'palette.rs').read_text()
draw = (CRATE / 'src' / 'chapters' / 'draw.rs').read_text()
for needle in ('pub fn hero() -> Srgb {\n        hex("#FFD866")', 'pub fn green() -> Srgb {\n        hex("#83C167")',
               'pub const BACKGROUND: &str = "#0E1116";'):
    assert needle in palette, needle
for needle in ('pub fn output() -> Srgb {\n    Palette::green()', 'pub fn hero() -> Srgb {\n    Palette::hero()',
               'pub fn ground() -> Srgb {\n    hex(explainer_kit::BACKGROUND)'):
    assert needle in draw, needle


def luminance(c):
    lin = [u / 12.92 if u <= 0.04045 else ((u + 0.055) / 1.055) ** 2.4 for u in c]
    return 0.2126 * lin[0] + 0.7152 * lin[1] + 0.0722 * lin[2]


def contrast(a, b):
    x, y = luminance(a), luminance(b)
    return (max(x, y) + 0.05) / (min(x, y) + 0.05)


def cross(o, a, b):
    return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])


def apart(a, b):
    return math.hypot(a[0] - b[0], a[1] - b[1])


def to_segment(q, a, b):
    dx, dy = b[0] - a[0], b[1] - a[1]
    t = min(1.0, max(0.0, ((q[0] - a[0]) * dx + (q[1] - a[1]) * dy) / (dx * dx + dy * dy)))
    return math.hypot(q[0] - a[0] - t * dx, q[1] - a[1] - t * dy)


def edges(pts):
    return [(pts[k], pts[(k + 1) % len(pts)]) for k in range(len(pts))]


def holds(es, q):
    return all(cross(u, w, q) >= 0 for u, w in es) or all(cross(u, w, q) <= 0 for u, w in es)


def gap(a, b):
    ea, eb = edges(a), edges(b)
    if any(cross(p, q, r) * cross(p, q, s) < 0 and cross(r, s, p) * cross(r, s, q) < 0 for p, q in ea for r, s in eb) \
            or any(holds(eb, q) for q in a) or any(holds(ea, q) for q in b):
        return 0.0
    return min(min(to_segment(q, u, w) for q in a for u, w in eb), min(to_segment(q, u, w) for q in b for u, w in ea))


def mirror(text, fill, rim_colour, rim_width, inner, outer):
    px, clear = 270.0, 0.035
    a_half, p_half = (12.0 / px, 12.5 / px), (13.5 / px, 17.5 / px)
    unit = 4.0 / 1.8                       # close_up(): 4.0 wide over (-0.9, 0.9)
    centre = (4.4, -0.95)
    small_r, loop_r, bound_r, hole = (number(text, n) for n in ('SMALL_R', 'LOOP_R', 'BOUND_R', 'HOLE'))
    sides = int(number(text, 'SIDES'))
    p = lambda z: (centre[0] + z[0] * unit, centre[1] + z[1] * unit)
    poly = lambda r: [(r * math.cos(2 * math.pi * k / sides), r * math.sin(2 * math.pi * k / sides)) for k in range(sides)]

    def heads(ccw, size):
        out = []
        cs = poly(small_r)
        for k in range(sides):
            a, b = p(cs[k]), p(cs[(k + 1) % sides])
            a, b = (a, b) if ccw else (b, a)
            ln = apart(a, b)
            ux, uy = (b[0] - a[0]) / ln, (b[1] - a[1]) / ln
            tip = ((a[0] + b[0]) / 2 + ux * size / 2, (a[1] + b[1]) / 2 + uy * size / 2)
            back = (tip[0] - ux * size, tip[1] - uy * size)
            out.append([tip, (back[0] - uy * size / 2, back[1] + ux * size / 2), (back[0] + uy * size / 2, back[1] - ux * size / 2)])
        return out

    box = lambda at, h: [(at[0] - h[0], at[1] - h[1]), (at[0] + h[0], at[1] - h[1]), (at[0] + h[0], at[1] + h[1]), (at[0] - h[0], at[1] + h[1])]
    fails, worst = [], {}
    note = lambda ok, what: ok or fails.append(what)
    note(fill == HERO and rim_colour == GROUND, 'colours')
    note(contrast(rim_colour, GREEN) > 4.5 and contrast(fill, rim_colour) > 4.5, 'contrast')
    note(inner > outer and inner * px > 40.0, 'size')
    rim = rim_width / 100.0 / 2.0
    note(rim * px > 3.0, 'rim')
    c0, name_a = p((0.0, 0.0)), box(p(point(text, 'A_AT')), a_half)
    cs = poly(small_r)
    for ccw in (False, True):
        for k, h in enumerate(heads(ccw, inner)):
            a, b = p(cs[k]), p(cs[(k + 1) % sides])
            middle = ((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
            note((cross(c0, middle, h[0]) > 0) == ccw, 'turn')
            worst['a'] = min(worst.get('a', 9), gap(h, name_a) - rim)
            inmost = min(to_segment(c0, h[j], h[(j + 1) % 3]) for j in range(3))
            outmost = max(apart(q, c0) for q in h)
            note(inmost - rim > hole + 0.015 + clear and outmost + rim < bound_r * unit - 0.015 - clear, 'hole/dashes')
    note(worst['a'] > clear, 'a')
    at = p(point(text, 'POLYGON_NAMES_AT', 1))
    name = box(at, p_half)
    inmost = min(to_segment(c0, name[j], name[(j + 1) % 4]) for j in range(4))
    outmost = max(apart(q, c0) for q in name)
    note(inmost > small_r * unit + 0.02 + clear, 'p vs circle')
    note(outmost < bound_r * unit - 0.015 - clear, 'p vs dashes')
    worst['p heads'] = min(gap(h, name) - rim for ccw in (False, True) for h in heads(ccw, inner))
    note(worst['p heads'] > clear, 'p vs heads')

    def to_polygon(q, r):
        cs2 = [p(z) for z in poly(r)]
        return min(to_segment(q, cs2[k], cs2[(k + 1) % sides]) for k in range(sides))
    big = p(point(text, 'POLYGON_NAMES_AT', 0))
    note(3 * to_polygon(at, small_r) < to_polygon(at, loop_r), 'p nearer the inner polygon')
    note(3 * to_polygon(big, loop_r) < to_polygon(big, small_r), 'P nearer the outer polygon')
    return fails, {k: round(v * px, 1) for k, v in worst.items()}


rim_w = number(repair_text, 'INNER_HEAD_RIM')
assert 'Style::default().fill(hero(), 1.0).stroke(ground(), INNER_HEAD_RIM, 1.0)' in repair_text
fails_r, worst_r = mirror(repair_text, HERO, GROUND, rim_w, number(repair_text, 'INNER_HEAD'), number(repair_text, 'OUTER_HEAD'))
fails_b, worst_b = mirror(base_text, GREEN, GREEN, 0.0, 0.12, 0.12)    # base: flat(output()), mid_head(.., 0.12)
assert not fails_r, fails_r
assert {'colours', 'contrast', 'size', 'rim', 'p nearer the inner polygon'} <= set(fails_b), fails_b
print('4. mirror: repair passes (least gaps, native px: %s); base fails as it should (%s)' % (worst_r, ', '.join(sorted(set(fails_b)))))
