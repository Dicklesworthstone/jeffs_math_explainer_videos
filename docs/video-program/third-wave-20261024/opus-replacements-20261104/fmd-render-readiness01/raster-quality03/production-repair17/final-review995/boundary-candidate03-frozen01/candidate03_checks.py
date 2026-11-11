"""candidate03_checks.py BASE_RS CANDIDATE_RS SCRIPT_TSV: source-bound checks for CC2's recommended chapter 09 boundary
candidate03. Self-contained: every input is a path argument, nothing is read relative to this file, nothing is
written, nothing is compiled or rendered. Exit 0 only if every check passes and every planted fault is caught.

Pinned inputs:
  BASE_RS       boundary.rs cd1900aa... (complex-boundary-repair752; native844 files[3]; final969 source_sha256)
  CANDIDATE_RS  boundary.rs 4b689791... (candidate03 = candidate02 + the leader for a)
  SCRIPT_TSV    script.tsv 234f7257... (root-boundary760-source), for 09_1's spoken order
Checks:
  T. timing: every timed or narration-binding call in the film part (the text before the test module), in order,
     with its timing arguments; fades' mob lists ignored. Two denominators are reported: CC2's 214 (12 kinds) and
     CC1's 228 (the same plus kit.music_breath, names_now, stage.add_updater, stage.clear_updaters). Only the heads
     fade and the bar fade may carry more mobs. Every 09_1 wait phrase in script order.
  S. source: exactly the 11 expected base lines removed; base tests verbatim; the repair's helpers, the shrink's
     names and leader, both growths' trackers, centres and ends, and 09_8's slide verbatim; SHRUNK_R 0.06;
     thin spaces; a faded by radius in both growths; the "1" card rooted above the 09_9 disk; the 7 new tests.
  G. geometry from the candidate's own constants (box model; 270 native px per scene unit): the shrink's names
     clear of the circle, dashes, hole ring and each other at every radius; the shrunk circle 7+ px off the hole
     ring; a's rest within 12.5 px of A_AT; a hidden wherever either growth's line is within CLEAR of it; the
     leader short of the hole ring and of a, across the circle once, clear of rho.
  F. planted faults in in-memory copies, each of which must fail T, S or G.
"""
import difflib
import hashlib
import math
import re
import sys

PINS = {'base': 'cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e',
        'candidate': '4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932',
        'script': '234f725781bde0bdf35afbc21fd2d6ec0f73f99a95723294259cc1b75a813d3c'}
TEST_MARK = '\n#[cfg(test)]\nmod tests {'
KINDS_214 = ['say', 'line.wait_for', 'line.wait_sentence', 'line.until', 'wait_until', 'kit.hold', 'pause',
             'stage.wait', 'fade_in_all', 'fade_out_all', 'stage.play_prepared', 'stage.play', 'tween_linear', 'predict']
KINDS_228 = KINDS_214 + ['kit.music_breath', 'names_now', 'stage.add_updater', 'stage.clear_updaters']
GROWN_FADES = {('fade_out_all(stage, &small_arrows, 0.2)', 'fade_out_all(stage, &leaving, 0.2)'),
               ('fade_in_all(stage, &[bar, bar_name], 0.3)',
                'fade_in_all(stage, &[bar, bar_name, ca_out, rho_out, leader], 0.3)')}
EXPECTED_REMOVED = [
    r'pub const ARC_TEX: &str = r"\Big|\int_{\mathrm{arc}} g\,dz - \int_{\mathrm{chord}} g\,dz\Big| \le w \cdot L";',
    r'pub const SUM_TEX: &str = r"\Big|\oint_{\mathrm{circle}} g\,dz - \oint_{\mathrm{polygon}} g\,dz\Big| \le \max w'
    r' \cdot \mathrm{total}\;L \to 0";',
    r'pub const SHRINK_TEX: &str = r"\Big|\oint_{|z-a|=\rho} g\,dz\Big| \le 2\pi\rho \cdot M \to 0";',
    'pub const SHRUNK_R: f64 = 0.045;',
    '        let a_name = kit.tex(stage, "a", 28.0, Palette::ink(), &[], p.p(C::new(0.18, -0.2)))?;',
    '        fade_out_all(stage, &small_arrows, 0.2)?;',
    '        fade_in_all(stage, &[bar, bar_name], 0.3)?;',
    '                a.move_to(rho, cp.p(C::new(r + 0.06, -0.12)), ORIGIN);',
    '            outer, small, hole, ca, rho, g_name, zero, dashes, bound, inside, bar, bar_name,',
    '        // the largest disk: it grows from the centre until it hits the poles',
    '                a.move_to(a_name, p.p(c + C::new(0.18, -0.2)), ORIGIN);',
]
VERBATIM = [
    'pub fn shrink_names(r: f64) -> [C; 2] {\n    let unit = close_up().unit();\n'
    '    [C::new(0.0, r + A_RIDE / unit), C::new(r + RHO_RIDE / unit, 0.0)]\n}',
    'pub fn a_name_shown(r: f64) -> f64 {\n    let [lo, hi] = A_NAME_HIDDEN;\n'
    '    ((lo - r).max(r - hi) / A_NAME_EASE).clamp(0.0, 1.0)\n}',
    'pub fn a_leader(r: f64) -> [C; 2] {\n    let unit = close_up().unit();\n    let foot = A_RIDE - 12.5 / 270.0;\n'
    '    [\n        C::new(0.0, (HOLE + 0.015 + 0.02) / unit),\n        C::new(0.0, r + (foot - 0.02) / unit),\n    ]\n}',
    '        let [a_from, rho_from] = shrink_names(SMALL_R);\n'
    '        let ca_out = kit.tex(stage, "a", 20.0, Palette::ink(), &[], cp.p(a_from))?;\n'
    '        let rho_out = kit.tex(stage, RHO_TEX, 22.0, output(), &[], cp.p(rho_from))?;\n'
    '        let [low, high] = a_leader(SMALL_R);\n        let leader = stage.arena_mut().add(\n'
    '            polyline(vec![cp.p(low), cp.p(high)])\n                .style(faded_line(Palette::ink(), 1.5, 0.6))\n'
    '                .build(),\n        );\n        let mut leaving = small_arrows;\n        leaving.extend([ca, rho]);\n',
    '                let [a_at, rho_at] = shrink_names(r);\n'
    '                a.move_to(ca_out, cp.p(a_at), ORIGIN);\n'
    '                a.move_to(rho_out, cp.p(rho_at), ORIGIN);\n'
    '                let [low, high] = a_leader(r);\n'
    '                let _ = a.set_points(leader, polyline(vec![cp.p(low), cp.p(high)]).build().points());\n',
    '            outer, small, hole, ca_out, rho_out, leader, g_name, zero, dashes, bound, inside, bar, bar_name,',
    'pub const A_NAME_HIDDEN: [f64; 2] = [0.11, 0.43];',
    'pub const A_NAME_FROM_CENTRE: C = C::new(0.18, -0.2);',
    'pub const SHRUNK_R: f64 = 0.06;',
    'let disk = stage.add_value_tracker(1e-3);',
    '.map(|k| p.p(C::polar(r, TAU * f64::from(k) / 120.0)))',
    'stage.play(tween_linear(disk, fact_f64("bump.radius0")?, rt)?)?;',
    'a.move_to(a_name, p.p(c + A_NAME_FROM_CENTRE), ORIGIN);',
    'stage.play(tween_linear(slide, 1.0, 1.8)?)?;',
    'let unit = p.circle(stage, C::real(1.0), 1e-3, hero(), 5.0)?;',
    'let grow = stage.add_value_tracker(1e-3);',
    '.map(|k| p.p(C::real(1.0) + C::polar(r, TAU * f64::from(k) / 160.0)))',
    'let radius = fact_f64("recentre1.radius")?;',
    'stage.play(tween_linear(grow, radius, rt)?)?;',
]
NEW_TESTS = ['the_closing_bars_keep_clear_of_dz', 'the_shrinking_circle_never_crosses_a_or_rho',
             'the_shrunk_circle_keeps_apart_from_the_hole_ring', 'the_shrinking_names_start_where_the_shrink_puts_them',
             'the_growing_disks_never_draw_through_a', 'the_growing_disks_pass_under_the_cards_and_round_a',
             'the_leader_runs_from_the_hole_up_to_a']
MODEL = {'A_RIDE': 0.02 + 0.045 + 12.5 / 270.0, 'RHO_RIDE': 0.02 + 0.045 + 15.0 / 270.0, 'A_NAME_EASE': 0.1}


def sha(b):
    return hashlib.sha256(b).hexdigest()


def film_of(text):
    return text.split(TEST_MARK)[0]


def calls(text, kinds):
    flat = re.sub(r'\s+', ' ', re.sub(r'//[^\n]*', '', text))
    out = []
    for m in re.finditer(r'(?<![\w.])(%s)\(' % '|'.join(map(re.escape, kinds)), flat):
        depth, i = 1, m.end()
        while depth:
            depth += {'(': 1, ')': -1}.get(flat[i], 0)
            i += 1
        call = flat[m.start():i].replace('( ', '(').replace(', )', ')').replace(',)', ')')
        f = re.match(r'(fade_in_all|fade_out_all)\(stage, (.*), ([0-9.]+)\)$', call)
        u = re.match(r'(stage\.add_updater|stage\.clear_updaters)\(\s*(\w+)', call)
        key = ('%s(.., %s)' % (f.group(1), f.group(3)) if f else '%s(%s, ..)' % (u.group(1), u.group(2)) if u else call)
        out.append((key, call))
    return out


def timing(base, cand, spoken):
    fails, counts = [], {}
    for label, kinds in (('214', KINDS_214), ('228', KINDS_228)):
        tb, tc = calls(film_of(base), kinds), calls(film_of(cand), kinds)
        counts[label] = (len(tb), len(tc))
        if [k for k, _ in tb] != [k for k, _ in tc]:
            fails.append('T%s: timing skeleton differs: %s' % (label, list(difflib.unified_diff(
                [k for k, _ in tb], [k for k, _ in tc], lineterm='', n=0))[2:5]))
        elif label == '214':
            grown = {(b, c) for (_, b), (_, c) in zip(tb, tc) if b != c and b.startswith('fade_')}
            if grown != GROWN_FADES:
                fails.append('T: fades changed other than the two expected: %s' % sorted(grown ^ GROWN_FADES)[:2])
    body = film_of(cand).split('// 09_1: divide by', 1)[1].split('// 09_2: on a circle', 1)[0]
    phrases = [c.split('"')[1] for c in body.split('line.wait_for(')[1:]]
    try:
        pos = [spoken.index(p) for p in phrases]
        if pos != sorted(pos) or len(phrases) < 25:
            fails.append('T: 09_1 wait phrases out of script order')
    except ValueError as e:
        fails.append('T: a 09_1 wait phrase is not in the script: %s' % e)
    return fails, counts


def source(base, cand):
    fails = []
    old, new = base.splitlines(), cand.splitlines()
    ops = difflib.SequenceMatcher(None, old, new, autojunk=False).get_opcodes()
    removed = [l for tag, i1, i2, _, _ in ops if tag in ('replace', 'delete') for l in old[i1:i2]]
    if removed != EXPECTED_REMOVED:
        fails.append('S: removed base lines differ: extra %s missing %s' % (
            [l for l in removed if l not in EXPECTED_REMOVED][:2], [l for l in EXPECTED_REMOVED if l not in removed][:2]))
    if not cand[cand.index(TEST_MARK):].startswith(base[base.index(TEST_MARK):].rstrip()[:-1]):
        fails.append('S: base tests not kept verbatim')
    film = film_of(cand)
    for want in VERBATIM:
        if film.count(want) != 1:
            fails.append('S: not exactly once in the film code: %s' % want.splitlines()[0])
    for name, want in MODEL.items():
        m = re.search(r'pub const %s: f64 = ([0-9. +/*-]+);' % name, cand)
        if not m or abs(eval(m.group(1), {'__builtins__': {}}) - want) > 1e-15:
            fails.append('S: %s is not the modelled value' % name)
    for tex in ('ARC_TEX', 'SUM_TEX', 'SHRINK_TEX'):
        m = re.search(r'pub const %s: &str = r"(.*)";' % tex, cand)
        if not m or not m.group(1).rsplit(r'\Big|', 1)[0].endswith(r'g\,dz\,'):
            fails.append('S: %s lacks the thin space before its closing bar' % tex)
    if film.count('a.set_opacity(a_name, a_name_shown(r), true);') != 2:
        fails.append('S: a is not faded by radius in both growths')
    i = film.find('fade_in_all(stage, &[unit, reach1, reach_name[0], reach_name[1]], 0.3)?;')
    j = film.find('for m in one_name {\n            stage.add_to_scene(m)?;')
    k = film.find('let grow = stage.add_value_tracker(1e-3);')
    if not 0 <= i < j < k:
        fails.append('S: the "1" card is not rooted above the 09_9 disk before it grows')
    tests = cand[cand.index(TEST_MARK):]
    for t in NEW_TESTS:
        if tests.count('    #[test]\n    fn %s() {' % t) != 1:
            fails.append('S: test %s missing' % t)
    return fails


PX, AA, CLEAR = 270.0, 1.5 / 270.0, 0.035
CU, PU = 4.0 / 1.8, 5.6 / 5.0
A20, RHO22, A28 = (12.0 / PX, 12.5 / PX), (15.0 / PX, 21.0 / PX), (17.5 / PX, 19.0 / PX)
DASH_IN = 0.2906 * CU                                      # handoff753's measured dash ink
FACTS = {'bump.radius0': 1.0, 'recentre1.radius': 1.414}   # facts.rs prints fixed(1, 0) and fixed(sqrt 2, 3)


def radial(b):
    x0, x1, y0, y1 = b
    nx = 0.0 if x0 <= 0.0 <= x1 else min(abs(x0), abs(x1))
    ny = 0.0 if y0 <= 0.0 <= y1 else min(abs(y0), abs(y1))
    return math.hypot(nx, ny), math.hypot(max(abs(x0), abs(x1)), max(abs(y0), abs(y1)))


def ring_gap(b, radius, half):
    lo, hi = radial(b)
    return max(lo - (radius + half + AA), (radius - half - AA) - hi)


def geometry(cand):
    fails, report = [], {}
    num = lambda n: eval(re.search(r'pub const %s: f64 = ([0-9. +/*-]+);' % n, cand).group(1), {'__builtins__': {}})
    try:
        SMALL, MID, SHRUNK, BOUND, HOLE = (num(n) for n in ('SMALL_R', 'MID_R', 'SHRUNK_R', 'BOUND_R', 'HOLE'))
        A_RIDE, RHO_RIDE, EASE = num('A_RIDE'), num('RHO_RIDE'), num('A_NAME_EASE')
        lo_h, hi_h = (float(x) for x in re.search(r'A_NAME_HIDDEN: \[f64; 2\] = \[([0-9.]+), ([0-9.]+)\]', cand).groups())
        off = tuple(float(x) for x in re.search(r'A_NAME_FROM_CENTRE: C = C::new\(([-0-9.]+), ([-0-9.]+)\)', cand).groups())
        lead = re.search(r'C::new\(0\.0, \(HOLE \+ 0\.015 \+ ([0-9.]+)\) / unit\),\n\s+C::new\(0\.0, r \+ \(foot - ([0-9.]+)\)'
                         r' / unit\),', cand)
        lead_lo, lead_hi = float(lead.group(1)), float(lead.group(2))
    except (AttributeError, TypeError) as e:
        return ['G: a modelled constant is missing (%s)' % e], report
    hole_out = HOLE + 0.015
    radii = sorted({SHRUNK + (SMALL - SHRUNK) * k / 4000 for k in range(4001)}
                   | {SMALL + (MID - SMALL) * k / 42 for k in range(43)} | {MID + (SHRUNK - MID) * k / 60 for k in range(61)})
    least = {}
    for r in radii:
        R = r * CU
        a = (-A20[0], A20[0], R + A_RIDE - A20[1], R + A_RIDE + A20[1])
        rho = (R + RHO_RIDE - RHO22[0], R + RHO_RIDE + RHO22[0], -RHO22[1], RHO22[1])
        gaps = {'a/circle': ring_gap(a, R, 0.02), 'rho/circle': ring_gap(rho, R, 0.02),
                'a/dashes': DASH_IN - radial(a)[1], 'rho/dashes': DASH_IN - radial(rho)[1],
                'a/hole': ring_gap(a, HOLE, 0.015), 'rho/hole': ring_gap(rho, HOLE, 0.015),
                'a/rho': math.hypot(max(rho[0] - a[1], 0.0), max(a[2] - rho[3], 0.0))}
        low, high = hole_out + lead_lo, R + (A_RIDE - A20[1]) - lead_hi
        gaps.update({'leader/hole': low - 0.0075 - hole_out, 'leader/a': (R + A_RIDE - A20[1]) - (high + 0.0075),
                     'leader/rho': rho[0] - 0.0075})
        if not (low < R - 0.02 and high > R + 0.02):
            fails.append('G: the leader does not cross the circle at r %.4f' % r)
        for key, g in gaps.items():
            least[key] = min(least.get(key, 1e9), g)
    for key, g in least.items():
        need = 0.0 if key in ('leader/hole', 'leader/a') else CLEAR
        if g <= need:
            fails.append('G: shrink %s least gap %.1f px' % (key, g * PX))
    report['shrink least px'] = {k: round(v * PX, 1) for k, v in least.items()}
    hole_gap = (SHRUNK * CU - 0.02 - hole_out) * PX
    a_end = abs(SHRUNK * CU + A_RIDE - 0.09 * CU) * PX
    report['hole gap px'], report['a rest from A_AT px'] = round(hole_gap, 1), round(a_end, 2)
    if not hole_gap > 7.0:
        fails.append('G: shrunk circle %.1f px from the hole ring' % hole_gap)
    if not a_end < 12.5:
        fails.append('G: a rests %.1f px from A_AT' % a_end)
    shown = lambda r: min(max(max(lo_h - r, r - hi_h) / EASE, 0.0), 1.0)
    name = (off[0] * PU - A28[0], off[0] * PU + A28[0], off[1] * PU - A28[1], off[1] * PU + A28[1])
    for label, end in (('09_7', FACTS['bump.radius0']), ('09_9', FACTS['recentre1.radius'])):
        near = [r for r in (1e-3 + (end - 1e-3) * k / 20000 for k in range(20001))
                if ring_gap(name, r * PU, 0.025) < CLEAR]
        bad = [r for r in near if shown(r) > 0.0]
        if not near or bad or shown(1e-3) < 1.0 or shown(end) < 1.0:
            fails.append('G: %s shows a near the line (r %s) or not at the ends' % (label, bad[:1]))
        report['%s line near a' % label] = (round(near[0], 3), round(near[-1], 3)) if near else None
    return fails, report


def run_all(base, cand, spoken):
    t, counts = timing(base, cand, spoken)
    g, report = geometry(cand)
    return t + source(base, cand) + g, counts, report


if len(sys.argv) != 4:
    sys.exit('usage: candidate03_checks.py BASE_RS CANDIDATE_RS SCRIPT_TSV')
raw = [open(p, 'rb').read() for p in sys.argv[1:4]]
for (key, pin), b in zip(PINS.items(), raw):
    assert sha(b) == pin, '%s is not at its pinned hash %s (got %s)' % (key, pin, sha(b))
base, cand, script = (b.decode() for b in raw)
spoken = next(l.split('\t', 1)[1] for l in script.splitlines() if l.startswith('09_1\t'))
fails, counts, report = run_all(base, cand, spoken)
print('pins: base %s, candidate %s, script %s' % (PINS['base'][:8], PINS['candidate'][:8], PINS['script'][:8]))
print('T: timing events base/candidate: CC2 denominator %d/%d, CC1 denominator %d/%d; identical skeletons, only the'
      ' heads and bar fades carry more mobs; 09_1 phrases in script order' % (counts['214'] + counts['228']))
print('S: 11 expected base lines removed, base tests verbatim, %d verbatim lines, %d new tests present'
      % (len(VERBATIM), len(NEW_TESTS)))
print('G: %s' % report)
if fails:
    print('FAIL: ' + '; '.join(fails))
    sys.exit(1)

FAULTS = [
    ('a fade time', 'fade_in_all(stage, &[bar, bar_name, ca_out, rho_out, leader], 0.3)?;',
     'fade_in_all(stage, &[bar, bar_name, ca_out, rho_out, leader], 0.35)?;'),
    ('an added pause', '        let radius = stage.add_value_tracker(SMALL_R);\n',
     '        stage.wait(0.2)?;\n        let radius = stage.add_value_tracker(SMALL_R);\n'),
    ('a tween time', 'tween_linear(radius, SHRUNK_R, 1.0)', 'tween_linear(radius, SHRUNK_R, 1.2)'),
    ('a reworded wait', 'line.wait_for(stage, "and that goes to zero")?;', 'line.wait_for(stage, "and that goes to nil")?;'),
    ('an extra updater', '        let radius = stage.add_value_tracker(SMALL_R);\n',
     '        stage.add_updater(radius, move |_, _| {}, true)?;\n        let radius = stage.add_value_tracker(SMALL_R);\n'),
    ('a music breath moved', 'kit.music_breath(', 'kit.music_breath_later('),
    ('a thin space dropped', r'g\,dz\,\Big| \le 2\pi', r'g\,dz\Big| \le 2\pi'),
    ('SHRUNK_R back at 0.045', 'pub const SHRUNK_R: f64 = 0.06;', 'pub const SHRUNK_R: f64 = 0.045;'),
    ('SHRUNK_R at 0.05', 'pub const SHRUNK_R: f64 = 0.06;', 'pub const SHRUNK_R: f64 = 0.05;'),
    ('rho made at RHO_AT', 'cp.p(rho_from))?;', 'cp.p(RHO_AT))?;'),
    ('a smaller ride gap', 'pub const A_RIDE: f64 = 0.02 + 0.045 + 12.5 / 270.0;',
     'pub const A_RIDE: f64 = 0.02 + 0.03 + 12.5 / 270.0;'),
    ('a narrower hidden span', '[0.11, 0.43]', '[0.15, 0.43]'),
    ('09_9 fade dropped', '                a.set_opacity(a_name, a_name_shown(r), true);\n            },\n            true,\n'
     '        )?;\n        let rt = line.until(stage, "So the new recipe", 1.6)?;',
     '            },\n            true,\n        )?;\n        let rt = line.until(stage, "So the new recipe", 1.6)?;'),
    ('card rooted after the growth', '        for m in one_name {\n            stage.add_to_scene(m)?;\n        }\n'
     '        let grow = stage.add_value_tracker(1e-3);\n',
     '        let grow = stage.add_value_tracker(1e-3);\n        for m in one_name {\n            stage.add_to_scene(m)?;\n'
     '        }\n'),
    ('09_9 tracker from 1', 'let grow = stage.add_value_tracker(1e-3);', 'let grow = stage.add_value_tracker(1.0);'),
    ('09_9 drawn about 0', '.map(|k| p.p(C::real(1.0) + C::polar(r, TAU * f64::from(k) / 160.0)))',
     '.map(|k| p.p(C::polar(r, TAU * f64::from(k) / 160.0)))'),
    ('leader not faded in', 'fade_in_all(stage, &[bar, bar_name, ca_out, rho_out, leader], 0.3)?;',
     'fade_in_all(stage, &[bar, bar_name, ca_out, rho_out], 0.3)?;'),
    ('leader left after the proof', 'ca_out, rho_out, leader, g_name,', 'ca_out, rho_out, g_name,'),
    ('leader not moved', '                let _ = a.set_points(leader, polyline(vec![cp.p(low), cp.p(high)]).build().points());\n', ''),
    ('leader onto the hole ring', 'C::new(0.0, (HOLE + 0.015 + 0.02) / unit),', 'C::new(0.0, (HOLE + 0.015 + 0.0) / unit),'),
    ('leader into a', 'C::new(0.0, r + (foot - 0.02) / unit),', 'C::new(0.0, r + (foot - 0.0) / unit),'),
    ('an edited base test', 'assert!(count >= 25);', 'assert!(count >= 20);'),
    ('a dropped new test', '    #[test]\n    fn the_leader_runs_from_the_hole_up_to_a() {', '    fn the_leader_runs_from_the_hole_up_to_a() {'),
]
caught = 0
film, tests = cand.split(TEST_MARK, 1)
for name, old, new in FAULTS:
    part = tests if name in ('an edited base test', 'a dropped new test') else film
    assert part.count(old) == 1, name
    mutant = (film + TEST_MARK + tests.replace(old, new)) if part is tests else (film.replace(old, new) + TEST_MARK + tests)
    got, _, _ = run_all(base, mutant, spoken)
    print('  fault %-30s -> %s' % (name, got[0][:96] if got else 'NOT CAUGHT'))
    caught += bool(got)
print('F: planted faults caught %d of %d' % (caught, len(FAULTS)))
print('PASS' if caught == len(FAULTS) else 'FAIL')
sys.exit(0 if caught == len(FAULTS) else 1)
