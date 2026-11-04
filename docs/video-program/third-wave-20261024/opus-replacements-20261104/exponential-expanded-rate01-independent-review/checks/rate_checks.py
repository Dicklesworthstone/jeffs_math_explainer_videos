#!/usr/bin/env python3
"""Arithmetic and source checks for the rate-law review. Run from the repo root.

Nothing here runs Rust, renders a frame or plays audio. Sentence onsets use the
Kit's scratch rule (words/2.6 + 0.4 s, onsets by character share) at illustrative
factors. No Oliver take exists for this script, so these are not measured times.
Text widths use CMR10 ems × size × 0.01, an estimate until the layout lint runs.
"""
import json
import math
from fractions import Fraction as Q

STAGE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
ROOT = STAGE + "exponential-expanded-rate01/"
REVIEW = STAGE + "exponential-expanded-rate01-independent-review/"
COUNTS = (4, 8, 16)
LEFT, RIGHT, WIDTH, BOTTOM, HEIGHT, TOP = -5.4, 0.5, 4.5, -1.8, 3.1, 3.0
failures = []


def check(ok, label):
    print(("ok   " if ok else "FAIL ") + label)
    if not ok:
        failures.append(label)


def euler(n):
    y, out = Q(1), [Q(1)]
    for _ in range(n):
        y += y * Q(1, n)
        out.append(y)
    return out


def drawn(n, t):
    """The stepped drawing's height at time t, exactly."""
    pts = euler(n)
    k = min(int(t * n), n - 1)
    return pts[k] + (t * n - k) * (pts[k + 1] - pts[k])


print("== 1. Exact arithmetic against root's oracle file")
oracle = json.load(open(ROOT + "rational-oracle01.json", encoding="utf-8"))
for row in oracle["checks"]:
    n, pts = row["n"], euler(row["n"])
    closed = [Q(n + 1, n) ** k for k in range(n + 1)]
    check(pts == closed, f"n={n}: recurrence equals (1+1/n)^k at all {n + 1} points")
    check([str(x) for x in pts] == row["all_endpoints"], f"n={n}: all_endpoints match the file")
    check(
        pts[-1] == Q(row["endpoint_numerator"], row["endpoint_denominator"]),
        f"n={n}: endpoint fraction matches",
    )
    check(f"{float(pts[-1]):.3f}" == row["display_3dp"], f"n={n}: display {row['display_3dp']}")
    float_y = 1.0
    for _ in range(n):
        float_y += float_y / n
    check(f"{float_y:.3f}" == row["display_3dp"], f"n={n}: f64 recurrence prints the same")
    check(pts[-1] < Q(2718281828, 1000000000), f"n={n}: endpoint below e")

print("\n== 2. What 01_4 claims about the drawings")
times = [Q(k, 16) for k in range(17)]
for coarse, fine in ((4, 8), (8, 16)):
    gaps = [drawn(fine, t) - drawn(coarse, t) for t in times]
    first = next(t for t, g in zip(times, gaps) if g > 0)
    check(all(g >= 0 for g in gaps), f"{fine}-step is never below {coarse}-step on the 1/16 grid")
    check(gaps[-1] == max(gaps), f"{fine} vs {coarse}: the widest gap is at the endpoint")
    print(f"     {fine} vs {coarse}: identical until t={first - Q(1, 16)}, end gap {float(gaps[-1]):.4f}")
end = {n: euler(n)[-1] for n in COUNTS}
check(end[16] - end[8] < end[8] - end[4], "the second refinement's end gap is smaller")
scale, stroke = HEIGHT / TOP, 3.5 * 0.01
print(f"     blue stroke is {stroke:.3f} scene units wide")
print("     t     4v8 units  8v16 units  (stroke width is the merge limit)")
for t in (Q(1, 4), Q(1, 2), Q(3, 4), Q(1)):
    a = float(drawn(8, t) - drawn(4, t)) * scale
    b = float(drawn(16, t) - drawn(8, t)) * scale
    print(f"     {float(t):.2f}  {a:9.3f}  {b:10.3f}")

print("\n== 3. Drawn slopes (anisotropic axes, consistent in data coordinates)")
y_per, x_per = HEIGHT / TOP, WIDTH
for label, rate in (("1.00", 1.0), ("1.25", 1.25), ("1.56", 1.5625), ("1.95", 1.953125), ("2.44", 2.44140625)):
    print(f"     slope {label} draws at {math.degrees(math.atan(rate * y_per / x_per)):.1f} degrees")
tip_y = 2.44140625 + 0.12 * 2.44140625
print(f"     last right tick ends at x={RIGHT + WIDTH * 1.12:.2f}, y={BOTTOM + y_per * tip_y:.3f}")
print(f"     last left tick ends at x={LEFT + WIDTH * 1.12:.2f}, y={BOTTOM + y_per * 2.12:.3f}")
check(tip_y < TOP, "the last tick stays under the top of the axis")

print("\n== 4. Text layout (estimated)")
EM = dict(a=.5, b=.556, c=.444, d=.556, e=.444, f=.306, g=.5, h=.556, i=.278, j=.306, k=.528, l=.278,
          m=.833, n=.556, o=.5, p=.556, q=.528, r=.392, s=.394, t=.389, u=.556, v=.528, w=.722, x=.528,
          y=.528, z=.444, S=.556, W=1.028, N=.75)
EM.update({".": .278, ",": .278, ":": .278, "?": .472, "=": .778, " ": .333})
EM.update({str(d): .5 for d in range(10)})


def box(text, size, x, y):
    w = sum(EM[c] for c in text) * size * 0.01
    h = 0.888 * size * 0.01
    return (x - w / 2, x + w / 2, y - h / 2, y + h / 2, w)


ALWAYS = [("title", "Same start, two different rules", 35, 0.0, 3.1),
          ("rule L", "constant rate = 1", 27, -3.15, 2.2),
          ("rule R", "rate = current height", 27, 2.75, 2.2),
          ("promise", "Next: construct an exact solution", 28, 0.0, 2.65),
          ("time L", "time: 0 to 1", 21, LEFT + WIDTH / 2, -2.25),
          ("time R", "time: 0 to 1", 21, RIGHT + WIDTH / 2, -2.25)]
READS_L = ["height 1.00: slope 1.00", "height 2.00: slope 1.00"]
READS_R = ["height 1.25: slope 1.25", "last step: slope 1.95, height now 2.44", "4 steps: 2.441",
           "4, 8 steps: 2.441, 2.566", "4, 8, 16 steps: 2.441, 2.566, 2.638"]
items = list(ALWAYS)
items += [("read L", s, 23, -3.15, 1.7) for s in READS_L]
items += [("read R", s, 23, 2.75, 1.7) for s in READS_R]
items += [("question", "Will the two curves stay together?", 27, 0.0, -2.65),
          ("approx", "straight steps: an approximation", 25, 2.75, -2.65)]
boxes = [(tag, s, box(s, size, x, y)) for tag, s, size, x, y in items]
for tag, s, b in boxes:
    print(f"     {tag:9} {b[4]:5.2f} wide  x {b[0]:6.2f}..{b[1]:5.2f}  y {b[2]:6.2f}..{b[3]:5.2f}  {s}")
check(all(abs(b[0]) < 6.86 and abs(b[1]) < 6.86 for _, _, b in boxes), "every string is inside the lint's x limit")
widest = max(b[4] for t, _, b in boxes if t == "read R")
check(widest < WIDTH, f"the widest right read-out ({widest:.2f}) fits the {WIDTH} wide panel")
# The two bottom-row strings never share the screen: the question leaves before 01_2.
apart = {("question", "approx"), ("read R", "read R"), ("read L", "read L")}
hits = []
for i, (ta, sa, a) in enumerate(boxes):
    for tb, sb, b in boxes[i + 1:]:
        if (ta, tb) in apart or (tb, ta) in apart:
            continue
        if a[0] < b[1] and b[0] < a[1] and a[2] < b[3] and b[2] < a[3]:
            hits.append((sa, sb))
check(not hits, f"no two coexisting strings overlap {hits if hits else ''}")
print(f"     read-out bottom {box('x', 23, 0, 1.7)[2]:.2f}, axis top {BOTTOM + HEIGHT:.2f}, last tick top "
      f"{BOTTOM + y_per * tip_y:.2f}")
print(f"     start dot right of digit: digit ends {RIGHT - 0.25 + 0.0475:.3f}, dot begins {RIGHT - 0.09:.3f}")

print("\n== 5. Timing on the scratch rule (illustrative factors, not Oliver)")
script = dict(line.rstrip("\n").split("\t") for line in open(REVIEW + "candidate/script-slice.tsv", encoding="utf-8"))
root_script = dict(line.rstrip("\n").split("\t") for line in open(ROOT + "script-slice.tsv", encoding="utf-8"))


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start:i + 1].strip())
            start = i + 1
    return out


def timing(text, f):
    length = (len(text.split()) / 2.6 + 0.4) * f
    parts = sentences(text)
    total = sum(len(p) for p in parts)
    starts, before = [], 0
    for p in parts:
        starts.append(length * before / total)
        before += len(p)
    return length, starts


for factor in (0.8, 1.0, 1.25):
    print(f"-- factor {factor}")
    rs = {c: timing(root_script[c], factor) for c in root_script}
    cs = {c: timing(script[c], factor) for c in script}
    (l1, s1), (l2, s2), (l3, s3), (l4, s4), (l5, s5) = (rs[c] for c in ("01_1", "01_2", "01_3", "01_4", "01_5"))
    start_3 = l1 + 0.6 + 4.0 + 0.3 + l2 + 0.6
    print(f"   root: rule R visible at 0.60, voice reaches it at {s1[3]:.2f}; question at 1.00, voice at {s1[5]:.2f}"
          f" (early {s1[5] - 1.0:.2f})")
    print(f"   root: footer visible at 0.60, voice says it at {start_3 + s3[4]:.2f} (early {start_3 + s3[4] - 0.6:.2f})")
    print(f"   root: 01_3 last change ends 4.80, voice runs to {l3:.2f}: {l3 - 4.8:.2f} s with no picture change")
    print(f"   root: 'slope 2.44' stays until 01_4 S3 at {s4[3]:.2f} + 0.30, with its tick gone at 0.30")
    print(f"   root: promise at 0.40, voice reaches it at {s5[2]:.2f} (early {s5[2] - 0.4:.2f})")
    (l1, s1), (l2, s2), (l3, s3), (l4, s4), (l5, s5) = (cs[c] for c in ("01_1", "01_2", "01_3", "01_4", "01_5"))
    check(s1[2] >= 0.6 and all(s1[j] + 0.4 <= s1[j + 1] for j in (2, 3, 4)), "cand 01_1: each reveal ends before the next sentence")
    print(f"   cand: question lands {s1[5]:.2f}, its voice ends {l1:.2f}, think gap {0.6 + 4.0:.1f} s + 0.3 fade")
    print(f"   both: 01_2 read-outs change at S2 {s2[2]:.2f}; the left sentence starts {s2[3]:.2f}, the right {s2[4]:.2f}")
    check(s3[1] + 1.2 <= s3[2], f"cand 01_3: step 2 read-out done before S2 ({s3[1] + 1.2:.2f} vs {s3[2]:.2f})")
    check(s3[2] + 1.8 <= s3[3], f"cand 01_3: step 3 done before S3 ({s3[2] + 1.8:.2f} vs {s3[3]:.2f})")
    check(s3[3] + 1.8 <= s3[4], f"cand 01_3: step 4 done before S4 ({s3[3] + 1.8:.2f} vs {s3[4]:.2f})")
    print(f"   cand: 01_3 last change at S5 {s3[5] + 0.4:.2f}, voice runs to {l3:.2f}: {l3 - s3[5] - 0.4:.2f} s still")
    check(0.6 <= s4[2], f"cand 01_4: read-out replaced at 0.60, before grey at S2 {s4[2]:.2f}")
    draw8 = s4[3] + 0.3 + 1.2
    draw16 = s4[4] + 0.4 + 0.3 + 2.4
    check(s4[2] + 0.4 <= s4[3], "01_4: grey ends before the 8-step label")
    check(draw8 <= s4[4], f"01_4: 8 steps done before S4 ({draw8:.2f} vs {s4[4]:.2f})")
    print(f"   01_4: 16 steps done {draw16:.2f}, S5 at {s4[5]:.2f}, past S5 by {draw16 - s4[5]:.2f}")
    total = sum(cs[c][0] for c in cs) + 4 * 0.6 + 4.0 + 0.3 + 0.6
    print(f"   cand: 01_5 promise {s5[2]:.2f}; chapter about {total:.1f} s")

print("\n== 6. Python mirrors of the new tests (the Rust tests have never run)")
law = open(REVIEW + "candidate/rate_law.rs", encoding="utf-8").read().split("#[cfg(test)]")[0]


def in_order(needles):
    at = 0
    for needle in needles:
        found = law.find(needle, at)
        if found < 0:
            return False
        at = found + len(needle)
    return True


check(in_order(['until_sentence(stage, "01_1", 2)', '"constant rate = 1"', 'until_sentence(stage, "01_1", 3)',
                '"rate = current height"', 'until_sentence(stage, "01_1", 4)', "reading_label(c[0].1, 1.0)",
                'until_sentence(stage, "01_1", 5)', '"Will the two curves stay together?"',
                'until_sentence(stage, "01_3", 4)', '"straight steps: an approximation"',
                'until_sentence(stage, "01_3", 5)', "last_step_label(p[last - 1].1, p[last].1)",
                'until_sentence(stage, "01_5", 2)', '"Next: construct an exact solution"']),
      "each_reveal_waits_for_the_sentence_that_names_it")
check(in_order(["show(stage, &[question], 0.4)?;", "kit.hold(stage)?;", 'kit.mark(stage, "predict");',
                "kit.music_breath(stage, 4.0)?;", "fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?;",
                'kit.say(stage, "01_2")?;']),
      "the_think_gap_is_authored_and_nothing_new_appears_during_it")
check(in_order(['kit.say(stage, "01_4")?;', "fade_out(stage.arena_mut(), right_slope, ORIGIN, 1.0)?;",
                "endpoints_label(&STEP_COUNTS[..1])", "morph(stage, &[(right_read, coarse)], 0.3)?;",
                "proportional_steps(n)"]),
      "no_refinement_runs_under_the_old_slope_or_its_read_out")
p = [float(x) for x in euler(4)]
c = [1 + k / 4 for k in range(5)]
rows = [f"height {y:.2f}: slope {y:.2f}" for y in p]
check(rows == ["height 1.00: slope 1.00", "height 1.25: slope 1.25", "height 1.56: slope 1.56",
               "height 1.95: slope 1.95", "height 2.44: slope 2.44"], "reading_label rows")
check(f"height {c[4]:.2f}: slope {1.0:.2f}" == "height 2.00: slope 1.00", "left reading at the end")
check(f"last step: slope {p[3]:.2f}, height now {p[4]:.2f}" == "last step: slope 1.95, height now 2.44", "last_step_label")
ends = [f"{float(end[n]):.3f}" for n in COUNTS]
check(", ".join(ends) == "2.441, 2.566, 2.638", "endpoints_label numbers")
for name in ("01_1", "01_2", "01_3", "01_4", "01_5"):
    check(len(script[name].split()) < 100, f"{name}: {len(script[name].split())} words")
check(sum(len(v.split()) for v in root_script.values()) == 244, "root's slice is the 244 words its README states")
check(sum(len(v.split()) for v in script.values()) == 244 + 4, "the candidate slice adds 4 words")
check(all(len(sentences(script[c])) == len(sentences(root_script[c])) for c in script), "sentence counts are unchanged")

print("\n== 7. Rows for facts.rs and oracle.py (root integrates; not added here)")
for k in range(1, 5):
    print(f"     rate.height.{k}\t{float(euler(4)[k]):.2f}")
for k in range(1, 5):
    print(f"     rate.line_height.{k}\t{float(1 + Q(k, 4)):.2f}")
for n in COUNTS:
    print(f"     rate.end.{n}\t{float(end[n]):.3f}")
check(f"{float(end[4]):.5f}" == "2.44141" and f"{float(end[16]):.5f}" == "2.63793", "matches the crate's limit.4 and limit.16 facts when rounded to 5 places")

print(f"\nFAILURES: {len(failures)}")
raise SystemExit(1 if failures else 0)
