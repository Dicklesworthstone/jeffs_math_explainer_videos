"""Check logarithm chapter 12's staged repair: mirror every new test's arithmetic,
the width table against measured strings, literals, anchors, order of the F7 beats."""
import math, re, sys
sys.path.insert(0, sys.argv[1])
from mirror import *
ROOT = "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm/preview12-repair01/explainer_corpus/logarithm_mysteries/src/chapters/"
CANON = "explainer_corpus/logarithm_mysteries/src/chapters/"
new = open(ROOT + "what_it_forgets.rs", encoding="utf-8").read()
old = open(CANON + "what_it_forgets.rs", encoding="utf-8").read()
PIN = (0.0, 2.0, 0.0); n = OTHER; REACH = 3.0


def line2(a, reach):
    m = math.ceil(reach * 40)
    return [tuple(a[i] * reach * (2 * k / m - 1) for i in range(3)) for k in range(m + 1)]


assert line2(ROD, 2.0) == [tuple(ROD[i] * 2.0 * (2 * k / 80 - 1) for i in range(3)) for k in range(81)]


def pin_pts(pose):
    a, b = turn(*pose, TOP), turn(*pose, PIN)
    return [tuple(a[i] + (b[i] - a[i]) * k / 24 for i in range(3)) for k in range(25)]


fails = 0


def chk(ok, msg):
    global fails
    if not ok:
        fails += 1
        print("FAIL", msg)


mins = dict(pin=9, arcdiff=0, top=0)
for j in range(1, 41):
    t = j / 40
    p = (n, TAU * .5 * t)
    d = length(runs(path(TOP, n, .5, t), p))
    mins['top'] = max(mins['top'], d)
    chk(d < 1e-12, f"top {t} {d}")
    arc = path(PIN, n, .5, t)
    whole = length([[project(q) for q in arc]])
    shown = length(runs(arc, p))
    mins['arcdiff'] = max(mins['arcdiff'], abs(shown - whole))
    chk(abs(shown - whole) < 1e-9, f"arc {t}")
    chk(not hidden(turn(*p, PIN), p), f"tip {t}")
    pd = length(runs(pin_pts(p), p))
    mins['pin'] = min(mins['pin'], pd)
    chk(pd > .8, f"pin {t} {pd}")
    chk(not hidden(CORNER, p) and not hidden(turn(ROD, PI, CORNER), p), f"rings {t}")
print("over 40 poses: top-centre drawn max", mins['top'], "| arc shown-vs-whole max diff", mins['arcdiff'], "| pin drawn min %.3f (test > 0.8)" % mins['pin'])
for t in [.25, .5, .75, 1.0]:
    p = (n, TAU * .5 * t)
    c = length(runs(path(CORNER, n, .5, t), p))
    a = length(runs(path(PIN, n, .5, t), p))
    print(f"t {t}: corner drawn {c:.3f} (test > 0.9); pin tip arc drawn {a:.3f} (test > 0.85)")
    chk(c > .9 and a > .85, f"thresholds {t}")
end = (n, PI)
chk(all(abs(x - y) < 1e-12 for x, y in zip(turn(n, PI, CORNER), (-1, -1, -1))), "corner end")
chk(not hidden(turn(n, PI, CORNER), end), "corner visible")
for c in CORNERS:
    a, b = turn(n, PI, c), turn(ROD, PI, c)
    chk(abs(math.dist(a, b) - 2) < 1e-9, f"gap {c}")
    chk(any(all(abs(x - y) < 1e-12 for x, y in zip(a, q)) for q in CORNERS), f"on corner {c}")
for t in (0., 1.):
    p = (n, TAU * .5 * t)
    d = length(runs(line2(n, REACH), p))
    print(f"other axis at t {t}: drawn {d:.3f} (test > 2.0)")
    chk(d > 2, "axis")
# existing bounds, with the longer red line and the pin
for axis, rate, reach in [(ROD, 2.5, 2.0), (n, .5, 3.0)]:
    for j in range(41):
        ang = TAU * rate * j / 40
        for c in CORNERS:
            q = project(turn(axis, ang, c))
            chk(q[0] < -.6 and -3 < q[1] < 2.9, f"corner bound {q}")
    for k in (-reach, reach):
        q = project(tuple(axis[i] * k for i in range(3)))
        chk(q[0] < -.6 and -3 < q[1] < 2.9, f"axis bound {q}")
rod_word_y = project((0, 2.3, 0))[1]
sc = [project(q) for q in path(PIN, n, .5, 1.0)]
print(f"pin sweep x {min(q[0] for q in sc):.3f}..{max(q[0] for q in sc):.3f}, y {min(q[1] for q in sc):.3f}..{max(q[1] for q in sc):.3f}; rod word y {rod_word_y:.3f}, test y < {rod_word_y-.18:.3f}")
for q in sc:
    chk(-6.8 < q[0] < -.6 and -3 < q[1] < rod_word_y - .18, f"sweep {q}")


# ---- panel D
def mmul(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(2)) for j in range(2)] for i in range(2)]


def mscale(a, k):
    return [[a[i][j] * k for j in range(2)] for i in range(2)]


def madd(a, b):
    return [[a[i][j] + b[i][j] for j in range(2)] for i in range(2)]


def inv(a):
    d = a[0][0] * a[1][1] - a[0][1] * a[1][0]
    return [[a[1][1] / d, -a[0][1] / d], [-a[1][0] / d, a[0][0] / d]]


def app(a, p):
    return [a[0][0] * p[0] + a[0][1] * p[1], a[1][0] * p[0] + a[1][1] * p[1]]


def rot(th):
    c, s = math.cos(th), math.sin(th)
    return [[c, -s], [s, c]]


I2 = [[1., 0.], [0., 1.]]
J = [[0., -1.], [1., 0.]]


def mexp(x):
    norm = max(abs(v) for r in x for v in r)
    sq = 0
    while norm / 2 ** sq > .25:
        sq += 1
    small = mscale(x, 1 / 2 ** sq)
    s = I2
    term = I2
    for k in range(1, 31):
        term = mscale(mmul(term, small), 1 / k)
        s = madd(s, term)
    for _ in range(sq):
        s = mmul(s, s)
    return s


def gen(a, b, tilt):
    p = mmul(rot(tilt), [[a, 0.], [0., b]])
    return mscale(mmul(mmul(p, J), inv(p)), TAU)


S = [1.6, 0.]


def orbit(a, b, tilt, t):
    return app(mexp(mscale(gen(a, b, tilt), t)), S)


fams = [(1., 1., 0.), (2., 1., 0.), (1.6, .7, math.radians(30))]
worst = 0
parted = []
for a, b, tilt in fams:
    for t, want in [(0., S), (1., S), (.5, [-S[0], -S[1]])]:
        q = orbit(a, b, tilt, t)
        worst = max(worst, abs(q[0] - want[0]), abs(q[1] - want[1]))
    p = mmul(rot(tilt), [[a, 0.], [0., b]])
    turned = mmul(mmul(p, rot(TAU * .3)), inv(p))
    m = orbit(a, b, tilt, .3)
    on = app(turned, S)
    worst = max(worst, abs(m[0] - on[0]), abs(m[1] - on[1]))
    parted.append(m)
print("orbit identities: worst error %.2e (test < 1e-9)" % worst)
chk(worst < 1e-9, "orbit")
gaps = [math.hypot(parted[i][0] - parted[j][0], parted[i][1] - parted[j][1]) for i in range(3) for j in range(i + 1, 3)]
print("dots at t 0.3 apart by", [round(g, 3) for g in gaps], "(test > 0.3)")
chk(min(gaps) > .3, "parted")
pts = []
for a, b, tilt in fams:
    for j in range(32):
        q = orbit(a, b, tilt, j / 32)
        pts.append((2.0 + .32 * q[0], -1.55 + .32 * q[1]))
xs, ys = [q[0] for q in pts], [q[1] for q in pts]
print(f"orbits on screen x {min(xs):.3f}..{max(xs):.3f}, y {min(ys):.3f}..{max(ys):.3f} (test x < 3.2, y > -2.4; panel D x .35..6.75, y -2.9..-.4)")
chk(max(xs) < 3.2 and min(ys) > -2.4 and min(xs) > .35 and max(ys) < -.4, "orbit bounds")


# ---- widths
def em(c):
    if c in "il,.:;![]":
        return .278
    if c in "jf":
        return .306
    if c in " -":
        return .333
    if c == "I":
        return .361
    if c in "rst()":
        return .39
    if c in "acez":
        return .444
    if c in "go^/0123456789":
        return .5
    if c in "kqvxy":
        return .528
    if c in "bdhnpu":
        return .556
    if c == "L":
        return .625
    if c == "w":
        return .722
    if c in "=+−×±<>":
        return .778
    if c == "m":
        return .833
    return .75


def width(s, size):
    return sum(em(c) for c in s) * size * .01


RIGHT = 7.11 - .25
ROOM = 1.03
rule = "Log zw = Log z + Log w + 2πik"
print(f"rule width table {width(rule,18):.3f}, measured 2.61 (frame-0100, WhiteSwan): {100*(width(rule,18)/2.61-1):+.1f}%; end at -3.1: table×room {-3.1+width(rule,18)*ROOM:.3f}, measured {-3.1+2.61:.3f}, border -0.35")
chk(-3.1 + width(rule, 18) * ROOM < -.35, "rule fits")
chk(-3.1 + width("a cut chooses the laps", 18) * ROOM < -.35, "cut fits")
measured = {
    "axis: only the rod; another axis drags the top off it": (22, (853 - 520) / 67.5),
    "so: infinitely many spins, all about the rod": (24, (825 - 520) / 67.5),
    "how far each of these spins turns in one unit of time:": (20, (889 - 574) / 67.5),
    "the slowest two, ±π, tie on the window's open edge": (20, (880 - 574) / 67.5),
}
for s, (size, w) in measured.items():
    print(f"  table {width(s,size):.3f} vs sheet read {w:.3f} ({100*(width(s,size)/w-1):+.1f}%)  {s[:44]}")
table = [
    ("each corner keeps to a circle about the rod", 20, .6),
    ("spin: ½ lap the other way per unit of time", 22, .6),
    ("every corner halfway round: a half turn about the rod", 22, .6),
    ("half turn after one unit of time?", 26, .6),
    ("axis: only the rod; another axis drags the top off it", 22, .6),
    ("so: infinitely many spins, all about the rod", 24, .6),
    ("how far each of these spins turns in one unit of time:", 20, 1.4),
    ("the slowest two, ±π, tie on the window's open edge", 20, 1.4),
]
for s, size, x in table:
    e = x + width(s, size) * ROOM
    print(f"  {e:.3f} < {RIGHT:.2f}  {s[:50]}")
    chk(e < RIGHT, s)
fine = "precisely: spins ending here: ±½, ±1½, … laps per unit time about the rod; eigenvalue −1, so no principal log (Higham §4 Fact 1)"
print(f"fine print width×room {width(fine,22)*ROOM:.3f} < {2*RIGHT:.2f}; sheet read {(907-54)/67.5:.2f}")
chk(width(fine, 22) * ROOM < 2 * RIGHT, "fine")
# labels beside panel C's new words
for k in (-2, -1, 0, 1):
    odd = 2 * k + 1
    y = -1.55 + .28 * odd
    lab = ("−" if odd < 0 else "") + (str(abs(odd)) if abs(odd) > 1 else "") + "πi"
    print(f"  column label {lab} y {y:.2f} ends x {-3.7+width(lab,16):.3f}; gap to words at -3.1: {-3.1-(-3.7+width(lab,16)):.3f}")
# ---- literals, anchors, order
lit = re.compile(r'"((?:[^"\\]|\\.)*)"')


def scene(t):
    return t.split("#[cfg(test)]")[0]


a, b = lit.findall(scene(old)), lit.findall(scene(new))
print("literals removed:", sorted(set(a) - set(b)) or "none", "| added:", sorted(set(b) - set(a)) or "none")
print("same multiset of literals:", sorted(a) == sorted(b))
chars = set("".join(b)) - set("".join(a))
print("characters new to scene code:", chars or "none")
for needle in ["until_sentence", "kit.say"]:
    x = re.findall(needle + r'\(stage, "[^"]+"(?:, \d+)?\)', scene(old))
    y = re.findall(needle + r'\(stage, "[^"]+"(?:, \d+)?\)', scene(new))
    print(f"{needle}: {len(x)} -> {len(y)}, same order: {x == y}")
    chk(x == y, needle)
sc2 = scene(new)
order = ['until_sentence(stage, "12_6", 5)', 'fade_in_all(stage, &marks', 'fade_in_all(stage, &ends', 'until_share(&kit, stage, "12_6", 5, tie_began, TIE_AT)', 'fade_in_all(stage, &[tie]', 'until_sentence(stage, "12_6", 6)', 'let source = precisely(', 'fade_in_all(stage, &[source, convention]', 'kit.finish(stage)']
pos = [sc2.find(o) for o in order]
print("F7 order holds:", pos == sorted(pos) and -1 not in pos)
chk(pos == sorted(pos) and -1 not in pos, "F7 order")
order = ['until_sentence(stage, "12_6", 3)', 'steady(pz.time, 1.0, red_spin)', 'until_share(&kit, stage, "12_6", 3, axis_began, AXIS_RESET)', 'set_tracker_value(pz.other, 0.0)', 'until_share(&kit, stage, "12_6", 3, axis_began, AXIS_CHIP)', 'fade_in_all(stage, &[axis_chip]', 'until_sentence(stage, "12_6", 4)']
pos = [sc2.find(o) for o in order]
print("axis-sentence order holds:", pos == sorted(pos) and -1 not in pos)
chk(pos == sorted(pos) and -1 not in pos, "axis order")
print("tween left in file:", len(re.findall(r'\btween\(', new)), "| M2 left:", len(re.findall(r'\bM2\b', new)), "| steady calls:", len(re.findall(r'steady\(', scene(new))) - 1, "| TOP_CENTRE path in draw:", 'self.path(s, TOP_CENTRE' in new)
# every new identifier is used in the scene code outside its definition
for name in "SHARED_START SHARED_RING_R PLANE_WORDS_X PIN_TIP OTHER_REACH AXIS_SPIN_END AXIS_RESET AXIS_CHIP TIE_AT MIN_RUN spin_path axis_points pin_points steady sentence_len room until_share frames_covering wait_frames".split():
    k = len(re.findall(r'\b' + name + r'\b', scene(new)))
    chk(k >= 2, f"unused outside tests? {name} {k}")
print("FAILS:", fails)
sys.exit(1 if fails else 0)
