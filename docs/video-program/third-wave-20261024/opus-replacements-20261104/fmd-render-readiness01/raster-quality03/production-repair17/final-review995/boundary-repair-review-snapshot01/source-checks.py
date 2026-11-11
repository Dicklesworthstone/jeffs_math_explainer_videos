"""final995_cc2_source_checks.py [REPAIR_SHA]: bounded source checks for CC2's scratch repair of chapter 09.

Base: production-repair17/complex-boundary-repair752/boundary.rs (sha cd1900aa..., the source bound to native844 and
final969). Repair: final995-cc2-boundary-repair/boundary.rs beside this script. Compiles and renders nothing.
  1. both files at their hashes;
  2. construct(): the same timing calls in the same order with the same timing arguments (wait phrases, until margins,
     tween targets and times, fade times, holds); a fade's mob list may grow, nothing else; every wait phrase in
     09_1's script order (script.tsv from root-boundary760-source);
  3. the only base lines removed are the expected ones, and the base's tests are kept verbatim;
  4. the repair's constants are the ones the clearance mirror models, and the mirror passes;
  5. planted faults in an in-memory copy each make checks 2 to 4 fail, and the clean copy passes.
"""
import difflib
import hashlib
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
P17 = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos/docs/video-program/third-wave-20261024/'
           'opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17')
BASE = P17 / 'complex-boundary-repair752' / 'boundary.rs'
SCRIPT = P17 / 'root-boundary760-source' / 'workspace' / 'explainer_corpus' / 'complex_analysis_visual' / 'script.tsv'
REPAIR = HERE / 'final995-cc2-boundary-repair' / 'boundary.rs'
MIRROR = HERE / 'final995_cc2_repair_mirror.py'
BASE_SHA = 'cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e'
TEST_MARK = '\n#[cfg(test)]\nmod tests {'
TIMING = ['line.wait_for', 'line.wait_sentence', 'line.until', 'stage.play', 'stage.play_prepared', 'fade_in_all',
          'fade_out_all', 'pause', 'wait_until', 'kit.hold', 'tween_linear', 'stage.wait', 'say', 'predict']
EXPECTED_REMOVED = [
    r'pub const ARC_TEX: &str = r"\Big|\int_{\mathrm{arc}} g\,dz - \int_{\mathrm{chord}} g\,dz\Big| \le w \cdot L";',
    r'pub const SUM_TEX: &str = r"\Big|\oint_{\mathrm{circle}} g\,dz - \oint_{\mathrm{polygon}} g\,dz\Big| \le \max w'
    r' \cdot \mathrm{total}\;L \to 0";',
    r'pub const SHRINK_TEX: &str = r"\Big|\oint_{|z-a|=\rho} g\,dz\Big| \le 2\pi\rho \cdot M \to 0";',
    '        let a_name = kit.tex(stage, "a", 28.0, Palette::ink(), &[], p.p(C::new(0.18, -0.2)))?;',
    '        fade_out_all(stage, &small_arrows, 0.2)?;',
    '        fade_in_all(stage, &[bar, bar_name], 0.3)?;',
    '                a.move_to(rho, cp.p(C::new(r + 0.06, -0.12)), ORIGIN);',
    '            outer, small, hole, ca, rho, g_name, zero, dashes, bound, inside, bar, bar_name,',
    '        // the largest disk: it grows from the centre until it hits the poles',
    '                a.move_to(a_name, p.p(c + C::new(0.18, -0.2)), ORIGIN);',
]
# what the mirror models; check 4 reads these from the repair and evaluates them
MODEL = {'A_RIDE': 0.02 + 0.045 + 12.5 / 270.0, 'RHO_RIDE': 0.02 + 0.045 + 15.0 / 270.0, 'A_NAME_EASE': 0.1}
MODEL_HIDDEN = [0.11, 0.43]
MODEL_FROM_CENTRE = (0.18, -0.2)
# the formulas the mirror models, and the shrink's use of them, verbatim
RIDING = [
    'pub fn shrink_names(r: f64) -> [C; 2] {\n    let unit = close_up().unit();\n'
    '    [C::new(0.0, r + A_RIDE / unit), C::new(r + RHO_RIDE / unit, 0.0)]\n}',
    'pub fn a_name_shown(r: f64) -> f64 {\n    let [lo, hi] = A_NAME_HIDDEN;\n'
    '    ((lo - r).max(r - hi) / A_NAME_EASE).clamp(0.0, 1.0)\n}',
    '        let [a_from, rho_from] = shrink_names(SMALL_R);\n'
    '        let ca_out = kit.tex(stage, "a", 20.0, Palette::ink(), &[], cp.p(a_from))?;\n'
    '        let rho_out = kit.tex(stage, RHO_TEX, 22.0, output(), &[], cp.p(rho_from))?;\n',
    '                let [a_at, rho_at] = shrink_names(r);\n'
    '                a.move_to(ca_out, cp.p(a_at), ORIGIN);\n'
    '                a.move_to(rho_out, cp.p(rho_at), ORIGIN);\n',
]


def sha(data):
    return hashlib.sha256(data.encode() if isinstance(data, str) else data).hexdigest()


def construct(text):
    return text[text.index('fn construct(&mut self'):text.index(TEST_MARK)]


def calls(text, names):
    flat = re.sub(r'\s+', ' ', text)
    out = []
    for m in re.finditer(r'\b(%s)\(' % '|'.join(map(re.escape, names)), flat):
        depth, i = 1, m.end()
        while depth:
            depth += {'(': 1, ')': -1}.get(flat[i], 0)
            i += 1
        out.append(flat[m.start():i].replace('( ', '(').replace(', )', ')').replace(',)', ')'))
    return out


def skeleton(call):
    """A fade keeps only its name and time (its mob list may grow); every other timing call is kept whole."""
    m = re.match(r'(fade_in_all|fade_out_all)\(stage, (.*), ([0-9.]+)\)$', call)
    if m:
        return '%s(.., %s)' % (m.group(1), m.group(3))
    return call


def checks(base_text, repair_text, run_mirror=True):
    fails = []
    cb, cr = construct(base_text), construct(repair_text)
    tb, tr = calls(cb, TIMING), calls(cr, TIMING)
    sb, sr = [skeleton(c) for c in tb], [skeleton(c) for c in tr]
    if sb != sr:
        fails.append('2: timing skeleton differs: %s' % list(difflib.unified_diff(sb, sr, lineterm='', n=0))[2:6])
    grown = [(b, r) for b, r in zip(tb, tr) if b != r]
    allowed = {('fade_out_all(stage, &small_arrows, 0.2)', 'fade_out_all(stage, &leaving, 0.2)'),
               ('fade_in_all(stage, &[bar, bar_name], 0.3)', 'fade_in_all(stage, &[bar, bar_name, ca_out, rho_out], 0.3)')}
    if sb == sr and set(grown) != allowed:
        fails.append('2: fades changed other than the two expected: %s' % [g for g in grown if g not in allowed][:3])
    spoken = next(line.split('\t', 1)[1] for line in SCRIPT.read_text().splitlines() if line.startswith('09_1\t'))
    body = cr.split('// 09_1: divide by', 1)[1].split('// 09_2: on a circle', 1)[0]
    phrases = [c.split('"')[1] for c in body.split('line.wait_for(')[1:]]
    try:
        positions = [spoken.index(p) for p in phrases]
    except ValueError as e:
        positions = None
        fails.append('2: a wait phrase is not in the 09_1 script: %s' % e)
    if positions is not None and (positions != sorted(positions) or len(phrases) < 25):
        fails.append('2: wait phrases out of script order')
    old, new = base_text.splitlines(), repair_text.splitlines()
    ops = difflib.SequenceMatcher(None, old, new, autojunk=False).get_opcodes()
    removed = [l for tag, i1, i2, _, _ in ops if tag in ('replace', 'delete') for l in old[i1:i2]]
    if removed != EXPECTED_REMOVED:
        fails.append('3: removed base lines differ: %s' % [l for l in removed if l not in EXPECTED_REMOVED][:3]
                     + ' missing %s' % [l for l in EXPECTED_REMOVED if l not in removed][:3])
    tests_b = base_text[base_text.index(TEST_MARK):].rstrip()
    tests_r = repair_text[repair_text.index(TEST_MARK):]
    if not tests_r.startswith(tests_b[:-1]):
        fails.append('3: base tests not kept verbatim')
    for name, want in MODEL.items():
        m = re.search(r'pub const %s: f64 = ([0-9. +/*-]+);' % name, repair_text)
        if not m or abs(eval(m.group(1), {'__builtins__': {}}) - want) > 1e-15:
            fails.append('4: %s is not the modelled value' % name)
    m = re.search(r'pub const A_NAME_HIDDEN: \[f64; 2\] = \[([0-9.]+), ([0-9.]+)\];', repair_text)
    if not m or [float(m.group(1)), float(m.group(2))] != MODEL_HIDDEN:
        fails.append('4: A_NAME_HIDDEN is not the modelled range')
    m = re.search(r'pub const A_NAME_FROM_CENTRE: C = C::new\(([-0-9.]+), ([-0-9.]+)\);', repair_text)
    if not m or (float(m.group(1)), float(m.group(2))) != MODEL_FROM_CENTRE:
        fails.append('4: A_NAME_FROM_CENTRE is not the modelled offset')
    for tex in ('ARC_TEX', 'SUM_TEX', 'SHRINK_TEX'):
        m = re.search(r'pub const %s: &str = r"(.*)";' % tex, repair_text)
        if not m or not m.group(1).rsplit(r'\Big|', 1)[0].endswith(r'g\,dz\,'):
            fails.append('4: %s lacks the thin space before its closing bar' % tex)
    film = repair_text.split(TEST_MARK)[0]
    for wanted in RIDING:
        if film.count(wanted) != 1:
            fails.append('4: not exactly once in the film code: %s' % wanted.splitlines()[0])
    if film.count('a.set_opacity(a_name, a_name_shown(r), true);') != 2:
        fails.append('4: a_name is not faded by radius in both growths')
    i_in = film.find('fade_in_all(stage, &[unit, reach1, reach_name[0], reach_name[1]], 0.3)?;')
    i_up = film.find('for m in one_name {\n            stage.add_to_scene(m)?;')
    i_grow = film.find('let grow = stage.add_value_tracker(1e-3);')
    if not (0 <= i_in < i_up < i_grow):
        fails.append('4: the "1" card is not rooted above the 09_9 disk before it grows')
    if run_mirror:
        r = subprocess.run([sys.executable, '-I', '-B', str(MIRROR)], capture_output=True, text=True)
        if r.returncode != 0:
            fails.append('4: the clearance mirror fails: %s' % r.stdout.strip().splitlines()[-1:])
    return fails


base_text, repair_text = BASE.read_text(), REPAIR.read_text()
assert sha(BASE.read_bytes()) == BASE_SHA, 'base changed'
print('1. base %s; repair %s%s; mirror %s' % (sha(BASE.read_bytes()), sha(REPAIR.read_bytes()),
      '' if len(sys.argv) < 2 else ' (pin given)', sha(MIRROR.read_bytes())))
if len(sys.argv) > 1:
    assert sha(REPAIR.read_bytes()) == sys.argv[1], 'repair is not at the given hash'
clean = checks(base_text, repair_text)
if clean:
    print('FAIL clean copy: ' + '; '.join(clean))
    sys.exit(1)
nb = len(calls(construct(base_text), TIMING))
print('2. construct: %d timing calls, same order and timing arguments; two fades gain mobs (the heads fade takes a and'
      ' rho, the bar fade takes their riding copies); wait phrases in 09_1 script order' % nb)
print('3. exactly the %d expected base lines removed; base tests kept verbatim' % len(EXPECTED_REMOVED))
print('4. constants match the mirror; thin spaces in ARC/SUM/SHRINK; a fades by radius in 09_7 and 09_9; "1" card'
      ' rooted above the 09_9 disk; mirror exits 0')

FAULTS = {
    'a fade time': ('fade_in_all(stage, &[bar, bar_name, ca_out, rho_out], 0.3)?;',
                    'fade_in_all(stage, &[bar, bar_name, ca_out, rho_out], 0.35)?;'),
    'an added pause': ('        let radius = stage.add_value_tracker(SMALL_R);\n',
                       '        stage.wait(0.2)?;\n        let radius = stage.add_value_tracker(SMALL_R);\n'),
    'a tween time': ('stage.play(tween_linear(radius, MID_R, 0.7)?)?;', 'stage.play(tween_linear(radius, MID_R, 0.8)?)?;'),
    'a reworded wait': ('line.wait_for(stage, "and that goes to zero")?;', 'line.wait_for(stage, "and that goes to nil")?;'),
    'a thin space dropped': (r'g\,dz\,\Big| \le 2\pi', r'g\,dz\Big| \le 2\pi'),
    'rho made at RHO_AT': ('cp.p(rho_from))?;', 'cp.p(RHO_AT))?;'),
    'a smaller ride gap': ('pub const A_RIDE: f64 = 0.02 + 0.045 + 12.5 / 270.0;', 'pub const A_RIDE: f64 = 0.02 + 0.03 + 12.5 / 270.0;'),
    'a narrower hidden span': ('[0.11, 0.43]', '[0.15, 0.43]'),
    '09_9 fade dropped': ('                let _ = a.set_points(unit, polyline(pts).build().points());\n'
                          '                a.set_opacity(a_name, a_name_shown(r), true);\n            },\n            true,\n'
                          '        )?;\n        let rt = line.until(stage, "So the new recipe", 1.6)?;',
                          '                let _ = a.set_points(unit, polyline(pts).build().points());\n'
                          '            },\n            true,\n'
                          '        )?;\n        let rt = line.until(stage, "So the new recipe", 1.6)?;'),
    'card rooted after the growth': ('        for m in one_name {\n            stage.add_to_scene(m)?;\n        }\n'
                                     '        let grow = stage.add_value_tracker(1e-3);\n',
                                     '        let grow = stage.add_value_tracker(1e-3);\n'
                                     '        for m in one_name {\n            stage.add_to_scene(m)?;\n        }\n'),
    'an edited base test': ('assert!(count >= 25);', 'assert!(count >= 20);'),
    'an offset in the updater': ('a.move_to(rho_out, cp.p(rho_at), ORIGIN);',
                                 'a.move_to(rho_out, cp.p(rho_at + C::new(0.0, -0.05)), ORIGIN);'),
    'a changed ride formula': ('C::new(r + RHO_RIDE / unit, 0.0)', 'C::new(r + RHO_RIDE / unit, -0.05)'),
}
caught = 0
film_r, tests_r = repair_text.split(TEST_MARK, 1)
for name, (old, new) in FAULTS.items():
    if name == 'an edited base test':
        assert tests_r.count(old) == 1, name
        mutant = film_r + TEST_MARK + tests_r.replace(old, new)
    else:
        assert film_r.count(old) == 1, name
        mutant = film_r.replace(old, new) + TEST_MARK + tests_r
    # a mirror-modelled constant changed only in the repair must fail check 4 without rerunning the mirror
    got = checks(base_text, mutant, run_mirror=False)
    print('   fault %-28s -> %s' % (name, got[0][:110] if got else 'NOT CAUGHT'))
    caught += bool(got)
print('5. planted faults caught: %d of %d; clean copy passes' % (caught, len(FAULTS)))
sys.exit(0 if caught == len(FAULTS) else 1)
