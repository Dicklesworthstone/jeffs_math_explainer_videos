"""Python mirrors of the repair's numbers: the spiral, the input dot, the half turn, the phrase times.

    python3 -I -B mirrors.py <crate-script.tsv>

Each mirror runs on the baseline formula as well, as a positive control: a check that the baseline
also passes proves nothing. Standard library. Exit 1 on any failure. Nothing is compiled or rendered.

The phrase times are ESTIMATES. Lengths are the stage06 preview's cue lengths. Sentence starts are a
character share of the cue, because no word-aligned transcript of an Oliver take is in this lane.
The one Oliver figure used is the length of the two 05_20 takes in ui-takes01, 25.28 and 25.12 s
against 24.631 s here, which is why the scales below run around 1.0.
"""
import math
import sys

PI, TAU = math.pi, math.tau
failures = []


def check(name, ok, detail=""):
    print(("PASS " if ok else "FAIL ") + name + (f": {detail}" if detail else ""))
    if not ok:
        failures.append(name)


# ------------------------------------------------------------------ B8, the spiral
CENTRE, HALF_H, STROKE, ZERO_RING, BRANCH_RING = (-3.4, -0.3), 2.7, 4.0 * 0.01, 0.13, 0.2


def old_spiral():
    k_, start = 0.18, -5.0 * PI
    end = math.log(HALF_H / 0.5) / k_
    out = []
    for k in range(301):
        th = start + (end - start) * k / 300
        out.append((0.5 * math.exp(k_ * th), th))
    return out


def new_spiral(turns=3.0, last=3.0 * PI):
    out = []
    for k in range(301):
        share = k / 300
        out.append((HALF_H * share, last - TAU * turns * (1 - share)))
    return out


def spiral_measures(path):
    """Least distance between a point and the point one full turn on; turns of strand inside each ring."""
    def at(r, th):
        return (CENTRE[0] + r * math.cos(th), CENTRE[1] + r * math.sin(th))
    points = [at(r, th) for r, th in path]
    least = math.inf
    for i, (r, th) in enumerate(path):
        j = next((j for j in range(i + 1, len(path)) if path[j][1] >= th + TAU), None)
        if j is None:
            break
        least = min(least, math.dist(points[i], points[j]))
    def turns_inside(radius):
        angles = [th for r, th in path if r < radius]
        return (angles[-1] - angles[0]) / TAU if angles else 0.0
    return {
        "start_r": path[0][0],
        "end_r": path[-1][0],
        "least_turn_gap": least,
        "turns_in_zero_ring": turns_inside(ZERO_RING),
        "turns_in_branch_ring": turns_inside(BRANCH_RING),
        "radius_only_grows": all(b[0] > a[0] for a, b in zip(path, path[1:])),
        "in_frame": all(x > -7.1 and -3.2 < y < 3.0 for x, y in points),
    }


def spiral_passes(m):
    return (
        m["least_turn_gap"] > 10 * STROKE
        and m["turns_in_zero_ring"] < 0.25
        and m["turns_in_branch_ring"] < 0.25
        and m["start_r"] < 1e-12
        and abs(m["end_r"] - HALF_H) < 1e-9
        and m["radius_only_grows"]
        and m["in_frame"]
    )


print("== B8, the spiral")
old = spiral_measures(old_spiral())
new = spiral_measures(new_spiral())
for label, m in (("baseline", old), ("staged", new)):
    print(f"  {label}: " + ", ".join(f"{k} {v:.4f}" if isinstance(v, float) else f"{k} {v}" for k, v in m.items()))
check("the baseline spiral fails, so the check can fail", not spiral_passes(old))
check("the baseline's inner turns stood barely a stroke apart", old["least_turn_gap"] < 2 * STROKE, f"{old['least_turn_gap']:.3f} against stroke {STROKE}")
check("the staged spiral passes", spiral_passes(new))
check("plant: twelve turns fails", not spiral_passes(spiral_measures(new_spiral(turns=12.0))))
check("the outer end stays where it was, near the negative axis", abs(new_spiral()[-1][1] - 3 * PI) < 1e-12 and abs((math.log(HALF_H / 0.5) / 0.18) - 3 * PI) < 0.06)

# ------------------------------------------------------------------ C2, the input dot
IN_Y, IN_CENTRE_Y, IN_SCALE, TIP, CLIMB_END = 3 * PI, -0.15, 0.3, 0.48, 6.5 * PI
PAST = TIP / IN_SCALE


def old_dot(y):
    return (min(y, IN_Y), 1.0)


def new_dot(y):
    past = max(y - IN_Y, 0.0)
    return (min(y, IN_Y + PAST), min(max(1.0 - past / PAST, 0.0), 1.0))


def dot_passes(f):
    rows = [f(IN_Y + PAST * k / 16) for k in range(17)]
    climbs = all(b[0] > a[0] and b[1] < a[1] for a, b in zip(rows, rows[1:]))
    gone = rows[-1][1] < 1e-9 and f(CLIMB_END)[1] == 0.0
    on_strip = all(f(y) == (y, 1.0) for y in (0.0, PI, TAU, IN_Y))
    return climbs and gone and on_strip


print("\n== C2, the input dot")
check("the baseline dot fails: it waits at the strip's top, whole", not dot_passes(old_dot), f"{old_dot(CLIMB_END)}")
check("the staged dot passes", dot_passes(new_dot), f"gone by y = {IN_Y + PAST:.3f}, climb ends at {CLIMB_END:.3f}")
tip_scene = IN_CENTRE_Y + IN_SCALE * (IN_Y + PAST)
check("the dot's last place is the arrow's tip, inside the safe area", abs(tip_scene - (IN_CENTRE_Y + IN_SCALE * IN_Y + TIP)) < 1e-12 and tip_scene < 3.85, f"y {tip_scene:.3f}")

# ------------------------------------------------------------------ R10, the half turn
Z_C, Z_S, Z_H, DOT_R, W, DEF_H = (-4.2, 0.1), 1.55, 2.2, 0.1, 5.0, 1.66
GAP, HEAD = (DOT_R + 0.07) / Z_S, 0.16
CAP = W * 0.01 / 2


def end_of(t):
    return GAP + (PI - 2 * GAP) * min(max(t, 0.0), 1.0)


def arc(t, n=60):
    a, b = GAP, end_of(t)
    return [(Z_C[0] + Z_S * math.cos(a + (b - a) * k / n), Z_C[1] + Z_S * math.sin(a + (b - a) * k / n)) for k in range(n + 1)]


def head(t):
    e = end_of(t)
    reach = min(HEAD, Z_S * (e - GAP))
    tip = (Z_C[0] + Z_S * math.cos(e), Z_C[1] + Z_S * math.sin(e))
    back, out = (math.sin(e), -math.cos(e)), (math.cos(e), math.sin(e))
    wing = lambda s: (tip[0] + reach * (0.87 * back[0] + 0.5 * s * out[0]), tip[1] + reach * (0.87 * back[1] + 0.5 * s * out[1]))  # noqa: E731
    return [wing(1.0), tip, wing(-1.0)]


def clearances(gap=GAP, reach_back=True):
    plus, minus = (Z_C[0] + Z_S, Z_C[1]), (Z_C[0] - Z_S, Z_C[1])
    least_dot, least_green, lowest, highest = math.inf, math.inf, math.inf, -math.inf
    for k in range(51):
        t = k / 50
        for p in arc(t) + head(t):
            least_dot = min(least_dot, math.dist(p, plus) - CAP - DOT_R, math.dist(p, minus) - CAP - DOT_R)
            least_green = min(least_green, math.dist(p, Z_C) - 0.68)
            lowest, highest = min(lowest, p[1]), max(highest, p[1])
    return least_dot, least_green, lowest, highest


print("\n== R10, the half turn")
least_dot, least_green, lowest, highest = clearances()
print(f"  least clear of a dot {least_dot:.3f}, of the green arcs {least_green:.3f}; y {lowest:.3f} to {highest:.3f}")
check("it never reaches a dot, cap included, by more than the test's margin", least_dot > 0.02)
check("it never reaches the green arcs", least_green > 0.2)
check("it stays above the cut and under the axis's top end", lowest > Z_C[1] + 0.05 and highest < Z_C[1] + Z_H)
check("it runs counter-clockwise through the top, as e^{πi} does", arc(1.0)[30][1] > arc(1.0)[0][1] and abs(arc(1.0)[30][0] - Z_C[0]) < 1e-9)
check("start and end sit one gap from +1 and −1", abs(end_of(0.0) - GAP) < 1e-12 and abs(end_of(1.0) - (PI - GAP)) < 1e-12)
one_turned = complex(1, 0) * complex(math.cos(PI), math.sin(PI))
check("1 · e^{πi} = −1", abs(one_turned + 1) < 1e-12)
# the baseline drew nothing here: the same geometry with no gap is the positive control for the dot check
GAP_SAVED, GAP = GAP, 0.0
check("plant: with no gap the arc reaches both dots", clearances()[0] < 0.0)
GAP = GAP_SAVED
# estimated word box: CMR10 ems × size × 0.01, cap height and x-height from the font's tables
word_w = (0.556 + 0.5 + 0.278 + 0.306 + 0.333 + 0.389 + 0.556 + 0.392 + 0.556) * 20 * 0.01
word_left, word_y = -2.95, 1.45
word_box = (word_left, word_left + word_w, word_y - 0.07, word_y + 0.07)
near = math.hypot(word_box[0] - Z_C[0], word_box[2] - Z_C[1])
print(f"  word box, estimated: x {word_box[0]:.2f} to {word_box[1]:.2f}, y {word_box[2]:.2f} to {word_box[3]:.2f}; corner {near:.3f} from the centre")
check("the word clears the ring and the half turn's stroke", near > Z_S + 0.1)
check("the word clears the right-hand column and the definition", word_box[1] < 0.5 and word_box[3] < Z_C[1] + Z_S * DEF_H - 0.3)

# ------------------------------------------------------------------ phrase times
script = {}
for line in open(sys.argv[1], encoding="utf-8"):
    if "\t" in line and not line.startswith("#"):
        cue, text = line.rstrip("\n").split("\t", 1)
        script[cue] = text


def opens_of(text):
    raw = text.encode("utf-8")
    opens = [0]
    for i in range(len(raw) - 1):
        if raw[i] in b".?!" and raw[i + 1] == 0x20:
            opens.append(i + 2)
    return raw, opens


def phrase_seconds(text, phrase, marks, starts, length):
    raw, opens = opens_of(text)
    byte = raw.find(phrase.encode("utf-8"))
    if byte < 0:
        raise ValueError("no phrase")
    if byte in marks:
        return marks[byte]
    if len(opens) != len(starts):
        raise ValueError("sentence count")
    j = max(i for i, o in enumerate(opens) if o <= byte)
    close = opens[j + 1] if j + 1 < len(opens) else len(raw)
    t0, t1 = starts[j], starts[j + 1] if j + 1 < len(starts) else length
    before = len(raw[opens[j]:byte].decode("utf-8"))
    whole = max(len(raw[opens[j]:close].decode("utf-8").rstrip()), 1)
    return t0 + (t1 - t0) * before / whole


def share_starts(text, length):
    raw, opens = opens_of(text)
    total = len(text)
    return [length * len(raw[:o].decode("utf-8")) / total for o in opens]


print("\n== the phrase rule, the Rust test's own numbers")
sample = "One two. Three four five six. Seven."
check("eleven of twenty characters", abs(phrase_seconds(sample, "five", {}, [0.2, 2.0, 6.0], 8.0) - (2.0 + 4.0 * 11 / 20)) < 1e-12)
check("a measured mark wins", phrase_seconds(sample, "five", {sample.find("five"): 3.25}, [0.2, 2.0, 6.0], 8.0) == 3.25)


def sigmoid(x):
    return 1 / (1 + math.exp(-x))


def smooth(t, inflection=10.0):
    error = sigmoid(-inflection / 2)
    return min(max((sigmoid(inflection * (t - 0.5)) - error) / (1 - 2 * error), 0.0), 1.0)


def crossing_share(frm, to, at):
    want = (at - frm) / (to - frm)
    lo, hi = 0.0, 1.0
    for _ in range(60):
        mid = (lo + hi) / 2
        lo, hi = (mid, hi) if smooth(mid) < want else (lo, mid)
    return lo


CROSS = crossing_share(0.45, 0.6, 0.5)
LENGTHS = {"05_20": 24.631, "06_6": 28.477, "08_5": 31.554}
SCHEDULES = {
    "05_20": [("S", 4), ("play", 0.4), ("wait", "the logarithm of two"), ("play", 0.4), ("wait", "a half pi"), ("play", 0.3), ("wait", "the length is two"), ("play", 0.4), ("wait", "land on two i"), ("play", 0.3), ("play", 0.5)],
    "06_6": [("S", 3), ("wait", "passes minus one"), ("play", 0.3), ("play", 1.6), ("wait", "leaps from pi to minus pi"), ("play", 0.4), ("play", 1.2), ("S", 4), ("play", 0.4)],
    "08_5": [("S", 3), ("play", 0.5), ("S", 4), ("play", 0.5), ("wait", "leaves a half turn"), ("play", 1.6), ("play", 0.3), ("wait", "a factor of minus one"), ("play", 0.3), ("S", 5), ("play", 0.5)],
}


def walk(cue, scale):
    text, length = script[cue], LENGTHS[cue] * scale
    starts = share_starts(text, length)
    now, rows = None, []
    for kind, value in SCHEDULES[cue]:
        if kind == "S":
            at = starts[value]
            rows.append((f"sentence {value}", at, (now or at) - at))
            now = max(now or at, at)
        elif kind == "wait":
            at = phrase_seconds(text, value, {}, starts, length)
            rows.append((value, at, now - at))
            now = max(now, at)
        else:
            now += value
    return rows, now, length


print(f"\n== timing fit, estimates: every hook, its time, and how late the picture reaches it (>0 is late)")
print(f"  the crossing of −1 falls {CROSS:.3f} of the way through the 1.6 s tween, under the engine's smooth easing, so {0.3 + 1.6 * CROSS:.2f} s after \"passes minus one\" starts")
worst = {}
for scale in (0.85, 1.0, 1.02, 1.15):
    for cue in SCHEDULES:
        rows, end, length = walk(cue, scale)
        if scale == 1.0:
            print(f"  {cue} scale 1.0, length {length:.3f}")
            for name, at, late in rows:
                print(f"    {at:7.3f}  {late:+.3f}  {name}")
            print(f"    {end:7.3f}  last picture settled; voice ends {length:.3f}")
        latest = max(late for _, _, late in rows)
        worst[(cue, scale)] = (latest, length - end)
for key, (latest, room) in sorted(worst.items()):
    print(f"  {key[0]} scale {key[1]:.2f}: latest arrival {latest:+.3f} s; {room:.3f} s of voice left after the last picture")
check("at scale 0.85 to 1.15 no picture arrives more than 0.6 s late at its hook", all(v[0] <= 0.6 for v in worst.values()))
check("and no last picture outlasts its voice, so no later cue moves", all(v[1] > 0 for v in worst.values()))

# ------------------------------------------------------------------ the positive control: the old lead
print("\n== positive control: the same model on the baseline's order")
text, length = script["05_20"], LENGTHS["05_20"]
starts = share_starts(text, length)
land = phrase_seconds(text, "land on two i", {}, starts, length)
check("C1: the baseline's answer led its words by 3 s or more", land - starts[4] >= 3.0, f"{land - starts[4]:.2f} s")
text, length = script["06_6"], LENGTHS["06_6"]
starts = share_starts(text, length)
passes, leaps = phrase_seconds(text, "passes minus one", {}, starts, length), phrase_seconds(text, "leaps from pi to minus pi", {}, starts, length)
old_cross, old_inset = starts[3] + 1.6 * CROSS, starts[3] + 1.6
check("L1: the baseline's drop led \"passes minus one\" by over 1 s", passes - old_cross > 1.0, f"{passes - old_cross:.2f} s")
check("L1: the baseline's inset led \"leaps\" by 3 s or more", leaps - old_inset >= 3.0, f"{leaps - old_inset:.2f} s")
text, length = script["08_5"], LENGTHS["08_5"]
starts = share_starts(text, length)
factor = phrase_seconds(text, "a factor of minus one", {}, starts, length)
check("R10: the baseline's sign led its words by 3 s or more", factor - starts[4] >= 3.0, f"{factor - starts[4]:.2f} s")

print("\nFAILURES: " + (", ".join(failures) if failures else "none"))
sys.exit(1 if failures else 0)
