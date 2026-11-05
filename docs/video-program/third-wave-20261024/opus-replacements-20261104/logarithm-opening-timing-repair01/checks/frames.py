"""Compute the new beats on the scratch timeline and test the new strings against ink on existing frames. Read only.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-opening-timing-repair01/checks/frames.py

The frames are root's stage06 preview at 720p with the scratch voice. They show the OLD picture. This says where the new
beats and strings would fall against what exists, not what a new render shows. Times inside a sentence are character
shares, as the kit's scratch voice and the candidates' helpers compute them; none of them is Oliver.
"""
import json
import sys

from PIL import Image

B = "narration_lab/third-wave-20261024/root/five-film-release-20261027/"
FR = B + "logarithm-stage06-frames01/"
RENDER = B + "logarithm-stage06-preview01/render/"
SCRIPT = "explainer_corpus/logarithm_mysteries/script.tsv"
fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


script = dict(line.rstrip("\n").split("\t", 1) for line in open(SCRIPT, encoding="utf-8") if "\t" in line and line[0].isdigit())


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start : i + 1].strip())
            start = i + 1
    return out


def cue(name, chapter):
    for line in open(f"{RENDER}{chapter}.cues.tsv", encoding="utf-8"):
        c, start, length = line.split("\t")
        if c == name:
            return float(start), float(length)
    raise KeyError(name)


def starts(name, chapter):
    start, length = cue(name, chapter)
    parts = sentences(script[name])
    total = sum(len(p) for p in parts)
    out, before = [], 0
    for p in parts:
        out.append(start + length * before / total)
        before += len(p)
    return out, start + length


def frame(chapter, n):
    im = Image.open(f"{FR}{chapter}/frame-{n:04d}.png").convert("RGB")
    return im.size[0], im.tobytes()


def ink(fr, x0, x1, r0, r1, test=lambda r, g, b: max(r, g, b) > 70):
    w, d = fr
    xs, rs = [], []
    for r in range(r0, r1 + 1):
        for x in range(x0, x1 + 1):
            i = 3 * (r * w + x)
            if test(d[i], d[i + 1], d[i + 2]):
                xs.append(x)
                rs.append(r)
    return (min(xs), max(xs), min(rs), max(rs)) if xs else None


def peak(fr, x0, x1, r0, r1):
    w, d = fr
    return max(max(d[3 * (r * w + x) : 3 * (r * w + x) + 3]) for r in range(r0, r1 + 1) for x in range(x0, x1 + 1))


times = {c: json.load(open(f"{FR}{c}/sampling.json", encoding="utf-8"))["times"] for c in ("00_ratio_ruler", "02_slide_rule", "03_folded_decade")}


def nearest(chapter, t):
    n = min(range(len(times[chapter])), key=lambda k: abs(times[chapter][k] - t))
    return f"frame {n} ({times[chapter][n]} s), sheet {n // 4}"


print("== F1: 00_1 and 00_2 on the scratch timeline")
ch = "00_ratio_ruler"
(s0, s1), end = starts("00_1", ch)
print(f"00_1 sentences at {s0:.3f} {s1:.3f}, voice ends {end:.3f}")
PHRASES = ["their frequencies", "four hundred and forty", "eight hundred and eighty", "seventeen sixty", "thirty-five twenty", "vibrations per second"]
WHAT = ["55, 110, 220", "440", "880", "1760", "3520", "vibrations per second"]
sentence = sentences(script["00_1"])[1]
arrivals = []
for phrase, what in zip(PHRASES, WHAT):
    share = sentence.index(phrase) / len(sentence)
    at = s1 + (end - s1) * share
    arrivals.append(at)
    print(f"  {what}: on '{phrase}', share {share:.3f}, {at:.2f} s, whole at {at + 0.35:.2f} s; {nearest(ch, at + 0.35)} on the old preview")
check(all(a + 0.35 < b for a, b in zip(arrivals, arrivals[1:])), "each number's fade ends before the next one starts")
check(arrivals[-1] + 0.35 < end, f"the unit is whole {end - arrivals[-1] - 0.35:.2f} s before the voice ends, so the picture does not push 00_2")
(p0, p1, p2), _ = starts("00_2", ch)
print(f"00_2 sentences at {p0:.3f} {p1:.3f} {p2:.3f}")
axis_end, dots_end, lead_end = p0 + 1.0, p0 + 1.0 + 7 * 0.18, p0 + 1.0 + 7 * 0.18 + 0.2
print(f"  axis and ticks {p0:.2f} to {axis_end:.2f}; dots to {dots_end:.2f}; lead to {lead_end:.2f}; next hook {p1:.2f}")
check(lead_end < p1, f"the ruler is whole {p1 - lead_end:.2f} s before sentence 1 of 00_2")
title_end = s0 + 0.6
beats = [title_end] + [x for a in arrivals for x in (a, a + 0.35)] + [p0, lead_end, p1]
stills = [(b - a, a, b) for a, b in zip(beats, beats[1:])]
longest = max(stills)
print(f"  longest stretch with no change in 00_1 and sentence 0 of 00_2: {longest[0]:.2f} s, {longest[1]:.2f} to {longest[2]:.2f}. Old: 15.0 s, 7.5 to 22.5, from the frames")
print(f"  title alone until {arrivals[0]:.2f} s")
check(longest[0] < 7.0, "the longest still stretch here is under 7 s")

print("\n== F2: 03_3 sentence 0 on the scratch timeline")
ch = "03_folded_decade"
parts, _ = starts("03_3", ch)
q0, q1 = parts[0], parts[1]
print(f"03_3 sentences at {q0:.3f} {q1:.3f}")
first = sentences(script["03_3"])[0]
named = "The dial remembers the digits,"
share = len(named) / len(first)
print(f"'{named}' is {len(named)} of {len(first)} characters, share {share:.3f}; the word ends near {q0 + (q1 - q0) * share:.2f} s")
check(abs(share - 0.58) < 0.01, "DIGITS_SHARE 0.58 is the script's")
now = q0 + 0.5
laps = max(q0 + (q1 - q0) * 0.58 - now, 0.6)
lay = max(q1 - now - laps - 0.3, 1.0)
print(f"title swap {q0:.2f} to {now:.2f}; laps fade {now:.2f} to {now + laps:.2f} ({laps:.2f} s); digits fade {now + laps:.2f} to {now + laps + 0.3:.2f}; lay {now + laps + 0.3:.2f} to {now + laps + 0.3 + lay:.2f} ({lay:.2f} s)")
check(abs(now + laps + 0.3 + lay - q1) < 1e-6, "the lay ends on sentence 1, so nothing after that hook moves")
old_gone = 43.0
print(f"digits stay until {now + laps:.2f} s; on the old preview they are gone by frame 101 ({old_gone} s)")
check(now + laps >= q0 + (q1 - q0) * share, "the digits start to leave only after the word is said, by share")
f100, f101 = frame(ch, 100), frame(ch, 101)
a, b = peak(f100, 152, 171, 336, 357), peak(f101, 152, 171, 336, 357)
c, d = peak(f100, 550, 583, 566, 589), peak(f101, 550, 583, 566, 589)
print(f"control on the old frames: the dial's 6 peak brightness {a} on frame 100, {b} on frame 101; the gold 2.4 {c} and {d}")
check(a > 150 and b < 40 and c > 150 and d < 40, "control: the old digits are up on frame 100 and gone on frame 101")
for label, t in (("laps half faded, digits whole", now + laps / 2), ("digits start to leave", now + laps), ("lay half way", now + laps + 0.3 + lay / 2), ("sentence 1, flat dial", q1)):
    print(f"  anchor {t:.2f} s, {label}: {nearest(ch, t)} on the old preview")
for q in (2.6, 3.0, 3.2, 3.6):
    n = 0.5
    l = max(q * 0.58 - n, 0.6)
    ly = max(q - n - l - 0.3, 1.0)
    print(f"  a first sentence of {q} s: laps {l:.2f} s, lay {ly:.2f} s, lay ends {n + l + 0.3 + ly - q:+.2f} s from sentence 1")

print("\n== F3: the two labels")
ch = "02_slide_rule"
EMS = {"0": 0.5, "1": 0.5, "2": 0.5, "3": 0.5, "4": 0.5, "5": 0.5, "6": 0.5, "7": 0.5, "8": 0.5, "9": 0.5, ".": 0.278, ":": 0.278, " ": 0.333, "o": 0.5, "f": 0.306, "a": 0.5, "u": 0.556, "l": 0.278, "e": 0.444, "n": 0.556, "g": 0.5, "t": 0.389, "h": 0.556}


def ems(text):
    return sum(EMS[c] for c in text)


def px(text):
    return ems(text) * 22 * 0.01 * 90


blue = lambda r, g, b: b > 150 and b > r + 40
f40, f65, f103 = frame(ch, 40), frame(ch, 65), frame(ch, 103)
bar = ink(f40, 80, 500, 415, 435, blue)
word = ink(f40, 80, 500, 440, 468, blue)
print(f"frame 40: bar columns {bar[0]}-{bar[1]}, old label columns {word[0]}-{word[1]}, rows {word[2]}-{word[3]}")
old_a, new_a = "0.398 of a full length", "2.5: 0.398 of a full length"
print(f"'{old_a}' {ems(old_a):.2f} ems, {px(old_a):.1f} px by ems, {word[1] - word[0] + 1} px measured")
check(abs(px(old_a) - (word[1] - word[0] + 1)) < 4, "control: the ems estimate is within 4 px of the old label's measured ink")
check(abs(ems(new_a) - 10.95) < 0.01, f"'{new_a}' is {ems(new_a):.2f} ems, the figure in the test")
centre = (word[0] + word[1]) / 2
left, right = centre - px(new_a) / 2, centre + px(new_a) / 2
print(f"new label about columns {left:.0f}-{right:.0f}: {left - bar[0]:.0f} px inside the bar's left end, {bar[1] - right:.0f} px inside its right end")
check(left > bar[0] + 10 and right < bar[1] - 10, "the longer label stays inside its bar, by estimate")
others = ink(f40, 80, int(left) - 2, word[2] - 3, word[3] + 3)
check(others is None, f"frame 40: nothing left of it on those rows: {others}")
others = ink(f40, int(right) + 2, 700, word[2] - 3, word[3] + 3)
check(others is None, f"frame 40: nothing right of it on those rows: {others}")
top = ink(f65, 380, 760, 170, 200, blue)
cbar = ink(f65, 380, 760, 205, 222, blue)
print(f"frame 65: C's bar columns {cbar[0]}-{cbar[1]}, old label columns {top[0]}-{top[1]}, rows {top[2]}-{top[3]}")
old_b, new_b = "0.505", "3.2: 0.505"
print(f"'{old_b}' {ems(old_b):.2f} ems, {px(old_b):.1f} px by ems, {top[1] - top[0] + 1} px measured")
check(abs(px(old_b) - (top[1] - top[0] + 1)) < 4, "control: the ems estimate is within 4 px of the second label's measured ink")
centre = (top[0] + top[1]) / 2
left, right = centre - px(new_b) / 2, centre + px(new_b) / 2
print(f"'{new_b}' {ems(new_b):.2f} ems, about columns {left:.0f}-{right:.0f}")
# 3 px off the old ink on each side: its antialiased edge is dim and is not what is being asked about
check(ink(f65, int(left) - 2, top[0] - 3, top[2] - 3, top[3] + 3) is None and ink(f65, top[1] + 3, int(right) + 2, top[2] - 3, top[3] + 3) is None, "frame 65: the extra width above C's bar lands on empty rows")
low = ink(f103, 500, 650, 440, 468, blue)
print(f"frame 103: second label on the D row, columns {low[0]}-{low[1]}, rows {low[2]}-{low[3]}")
centre = (low[0] + low[1]) / 2
left, right = centre - px(new_b) / 2, centre + px(new_b) / 2
first_word = ink(f103, 100, 400, 440, 468, blue)
print(f"new second label about columns {left:.0f}-{right:.0f}; the first label's new right end is near {(first_word[0] + first_word[1]) / 2 + px(new_a) / 2:.0f}")
check(ink(f103, int(left) - 2, low[0] - 3, low[2] - 3, low[3] + 3) is None and ink(f103, low[1] + 3, int(right) + 2, low[2] - 3, low[3] + 3) is None, "frame 103: the extra width on the D row lands on empty rows")
check((first_word[0] + first_word[1]) / 2 + px(new_a) / 2 + 40 < left, "frame 103: the two labels stay more than 40 px apart")
sum_row = ink(f103, 700, 1150, 480, 530, blue)
check(sum_row[2] > low[3] + 5, f"frame 103: the sum row starts at row {sum_row[2]}, below the label's {low[3]}")
for t in (15.567, 28.0, 41.5):
    print(f"  anchor {t} s: {nearest(ch, t)}")
print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
