"""Independent arithmetic for the logarithm 08-10 source review. Reads files, runs nothing.

    python3 -I -B <this file>        (from the repo root)

Widths use width = CMR10 ems x size x 0.01, which held within 3% on 11 measured
preview02 widths in logarithm-repair05-07/checks/predicted_margins.py. It is an
estimate, not a measured box. The timing simulation walks the candidate's beats
in source order under the scratch rule (kit: SCRATCH_WPS, estimated_starts) and
under illustrative speed factors. These are arithmetic, not Oliver.
"""
import math
import pathlib
import re

CRATE = pathlib.Path("explainer_corpus/logarithm_mysteries")
STAGE = pathlib.Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-repair08-10/candidate")
script = dict(l.split("\t", 1) for l in (CRATE / "script.tsv").read_text().splitlines() if l and not l.startswith("#"))

EM = {" ": 0.333, ",": 0.278, ":": 0.278, ".": 0.278, ";": 0.278, "=": 0.778, "(": 0.389, ")": 0.389, "[": 0.278,
      "]": 0.278, "π": 0.57, "→": 1.0, "−": 0.778, "×": 0.778, "+": 0.778, "°": 0.5, "?": 0.472, "'": 0.278,
      "±": 0.778, "<": 0.778, ">": 0.778, "↔": 1.0, "⇒": 1.0, "∞": 1.0, "½": 0.75, "-": 0.333}
for c in "0123456789":
    EM[c] = 0.5
EM.update(a=.5, b=.556, c=.444, d=.556, e=.444, f=.306, g=.5, h=.556, i=.278, j=.306, k=.528, l=.278, m=.833, n=.556,
          o=.5, p=.556, q=.528, r=.392, s=.394, t=.389, u=.556, v=.528, w=.722, x=.528, y=.528, z=.444,
          A=.75, B=.708, G=.785, H=.75, I=.361, J=.514, L=.625, T=.722, X=.75)


def w(text, size):
    return sum(EM[c] for c in text) * size * 0.01


print("== spans, centred unless noted; lint limit 6.86, safe 6.96")
rows = [
    ("F2 dictionary", "turn-and-scale matrix ↔ complex number", 20, 4.95, "c"),
    ("F3 rule", "keep −π < turning < π", 20, 3.85, "c"),
    ("F3 lap word", "ellipse flows: ±2π", 20, 5.95, "c"),
    ("F8 spiral word", "spiral flow: turns 0.872 radians per unit time, about 0.28π", 22, 1.0, "l"),
    ("10 stretch word", "stretch flow: does not turn round at all", 22, 1.0, "l"),
    ("F16 cond_a", "A sends no direction to 0 or to a negative multiple of itself", 22, 1.0, "l"),
    ("F16 cond_b", "⇒ exactly one logarithm turns by less than a half turn", 22, 1.0, "l"),
    ("F16 name", "the principal logarithm, log A: real when A is real", 22, 1.0, "l"),
    ("F11 target word", "J: quarter turn", 22, 5.4, "c"),
]
src = (STAGE / CRATE / "src/chapters/doing_nothing.rs").read_text()
TEXTBOOK = re.search(r'const TEXTBOOK: &str = "(.*?)";', src).group(1)
SOURCE = re.search(r'const SOURCE: &str = "(.*?)";', src).group(1)
rows.append(("F15 textbook", TEXTBOOK, 18, 0.0, "c"))
rows.append(("F15 precisely", "precisely: " + SOURCE, 22, 0.0, "c"))
for name, text, size, x, kind in rows:
    wid = w(text, size)
    a, b = (x - wid / 2, x + wid / 2) if kind == "c" else (x, x + wid)
    flag = "" if a > -6.86 and b < 6.86 else "  <-- past 6.86"
    print(f"  {name:18s} {a:6.2f} .. {b:5.2f}  width {wid:5.2f}{flag}")
base = (pathlib.Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-repair08-10/baseline") / CRATE / "src/chapters/doing_nothing.rs").read_text()
old = re.search(r'"(no eigenvalue of A in[^"]*)"', base).group(1)
print(f"  baseline precisely width {w('precisely: ' + old.replace('^', ' '), 22):5.2f} ({len('precisely: ' + old)} chars)")
chars = len(TEXTBOOK) + len("precisely: ") + len(SOURCE)
print(f"  fine print {chars} characters; {chars / 13.2:.2f} per second over 13.2 s")

print("== matrix cards through the candidate's card_widths")


def card(size, widest):
    dx = max(size * 0.011, (widest + size * 0.25 / 30) / 2)
    return dx, dx + max(size * 0.0095, widest / 2 + size * 0.10 / 30)


for name, entries, size, x in [
    ("rotation", ["0.5", "−0.866", "0.866", "0.5"], 34, 4.4),
    ("stretched", ["1.6", "0", "0", "0.6"], 34, 4.4),
    ("x_card", ["0.3", "−1", "0.8", "−0.1"], 30, 4.6),
    ("e_card", ["0.905", "−0.97", "0.776", "0.517"], 30, 4.6),
    ("target", ["0", "−1", "1", "0"], 30, 5.4),
    ("pair", ["a", "−b", "b", "a"], 26, 4.9),
    ("i_card", ["1", "0", "0", "1"], 32, 4.6),
    ("ch11 a_card -2,-0.5", ["−2", "0", "0", "−0.5"], 30, 3.65),
    ("ch11 a_card -2,-1/2", ["−2", "0", "0", "−½"], 30, 3.65),
    ("ch11 minus_i", ["−1", "0", "0", "−1"], 30, 3.65),
    ("ch11 m_card", ["−1", "0", "0", "1"], 32, 5.4),
]:
    widest = max(w(e, size) for e in entries)
    dx, half = card(size, widest)
    print(f"  {name:20s} widest {widest:5.3f} dx {dx:5.3f} half {half:5.3f} spans {x - half:5.2f} .. {x + half:5.2f}"
          f"  col gap {2 * dx - widest:5.3f} bracket gap {half - dx - widest / 2:5.3f}")

print("== mirror of the candidate's phrase timing, and the beats of 10_3 / 10_4")


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start:i + 1].strip())
            start = i + 1
    return out


def offsets(text):
    at = [0]
    for i in range(len(text) - 1):
        if text[i] in ".?!" and text[i + 1] == " ":
            at.append(i + 2)
    return at


def starts_for(text, speech):
    parts = sentences(text)
    total = sum(len(p) for p in parts)
    s, before = [], 0
    for p in parts:
        s.append(speech * before / total)
        before += len(p)
    return s


def phrase(text, starts, length, ph):
    byte = text.index(ph)
    of = offsets(text)
    assert len(of) == len(starts), (len(of), len(starts))
    j = max(k for k, a in enumerate(of) if a <= byte)
    frm, to = of[j], of[j + 1] if j + 1 < len(of) else len(text)
    t0, t1 = starts[j], starts[j + 1] if j + 1 < len(starts) else length
    return t0 + (t1 - t0) * (byte - frm) / max(to - frm, 1)


def sim(f):
    t3 = script["10_3"]
    speech = len(t3.split()) / 2.6 * f
    s3, l3 = starts_for(t3, speech), speech + 0.4 * f
    now = 0.0
    log = []
    now = max(now, 0) + 0.5 + 0.4  # clear, heading swap
    now = max(now, s3[1]) + 0.4 + 0.5  # the number line
    now = max(now, s3[2]) + 0.4  # swap to the flow words and resting grid
    turns, nott = phrase(t3, s3, l3, "turns round at a steady"), phrase(t3, s3, l3, "or does not turn")
    first = min(max(turns - now - 0.9, 1.0), 2.6)
    now += first
    log.append(("first run", first))
    now = max(now, s3[3]) + 0.4 + 0.5
    log.append(("sweep start minus phrase", max(now, turns) - turns))
    now = max(now, turns)
    sweep = max(nott - now - 0.5, 1.0)
    now += sweep + 0.5
    log.append(("sweep seconds", sweep))
    now = max(now, nott)
    log.append(("stretch word minus phrase", now - nott))
    now += 0.3
    run = max(s3[4] - now - 0.2, 1.0)
    log.append(("stretch run", run))
    now += run
    now = max(now, s3[4]) + 0.3 + 0.5 + 0.4
    now = max(now, s3[5]) + 0.5
    log.append(("10_3 beats end", now))
    log.append(("10_3 line length", l3))
    # 10_4 from its own zero
    t4 = script["10_4"]
    speech4 = len(t4.split()) / 2.6 * f
    s4, l4 = starts_for(t4, speech4), speech4 + 0.4 * f
    n = 0.4
    strict = phrase(t4, s4, l4, "strictly between")
    log.append(("10_4 strictly at", strict))
    n = max(n, strict) + 0.3 + 0.4
    log.append(("10_4 strict beats end vs s1", n - s4[1]))
    n = max(n, s4[1]) + 0.3 + 0.5
    n = max(n, s4[2]) + 0.5
    n = max(n, s4[3]) + 0.5
    n = max(n, s4[4]) + 0.5
    shown = n
    n = max(n, l4) + 0.6
    read = n - shown
    extra = max(13.2 - read, 0.0)
    log.append(("fine print voice+breath read", read))
    log.append(("added window", extra))
    log.append(("10_4 s4 span", l4 - s4[4]))
    return log


for f in (0.8, 1.0, 1.25):
    print(f"  factor {f}: " + "; ".join(f"{k} {v:.2f}" for k, v in sim(f)))

print("== 10_2 first sentence: candidate beats 0.4 + 1.1 + 0.3 = 1.8 s before sentence 1")
t2 = script["10_2"]
for f in (0.8, 1.0, 1.25):
    speech = len(t2.split()) / 2.6 * f
    print(f"  factor {f}: sentence 1 at {starts_for(t2, speech)[1]:.2f}")

print("== literal checks")
print("  repr of the test's rounding literal:", repr(3.0000000000000018))
th = 2 * math.pi
print("  e^X row 1 to 3 places:", round(0.90522, 3), round(-0.97041, 3))
print("  0.872/pi:", 0.8718 / math.pi)
