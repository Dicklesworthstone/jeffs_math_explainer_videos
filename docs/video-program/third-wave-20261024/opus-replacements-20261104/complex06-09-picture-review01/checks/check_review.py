"""Check REVIEW.md against its inputs. Standard library only.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex06-09-picture-review01/checks/check_review.py

It tests what the review says, not what the pictures show. A pass does not mean
any frame was looked at correctly. The two planted controls at the end must fail.
"""
import hashlib
import json
import os
import re
from collections import Counter

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex06-09-picture-review01/"
REL = "narration_lab/third-wave-20261024/root/five-film-release-20261027/"
FRAMES = REL + "complex-combined-picture-frames02/"
RENDER = REL + "complex-combined-picture-preview02/render/"
SRC = "explainer_corpus/complex_analysis_visual/src/chapters/"
CHAPTERS = {"06": "06_one_derivative", "07": "07_one_point", "08": "08_loop_sums", "09": "09_boundary"}
SHEETS = {"06": 117, "07": 66, "08": 58, "09": 140}
TOL = 0.06

fails = []


def check(ok, what):
    if not ok:
        fails.append(what)
        print("FAIL", what)


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


text = open(HERE + "REVIEW.md", encoding="utf-8").read()
times = {k: json.load(open(FRAMES + v + "/sampling.json", encoding="utf-8"))["times"] for k, v in CHAPTERS.items()}

script = {}
for line in open("explainer_corpus/complex_analysis_visual/script.tsv", encoding="utf-8"):
    parts = line.rstrip("\n").split("\t")
    if re.match(r"0[6-9]_\d+$", parts[0]):
        script[parts[0]] = parts[-1]
cues = {}
for name in CHAPTERS.values():
    for line in open(RENDER + name + ".cues.tsv", encoding="utf-8"):
        parts = line.rstrip("\n").split("\t")
        if re.match(r"0[6-9]_\d+$", parts[0]):
            cues[parts[0]] = (float(parts[1]), float(parts[2]))

sections = {}
for chapter in CHAPTERS:
    m = re.search(rf"^## Chapter {chapter}:.*?(?=^## )", text, re.M | re.S)
    check(bool(m), f"section for chapter {chapter}")
    sections[chapter] = m.group(0) if m else ""


def pair_ok(chapter, frame, stated):
    t = times[chapter]
    return frame < len(t) and abs(t[frame] - float(stated)) <= TOL


def frame_pairs(section):
    """Return (frame, time) pairs written as fN (T) or fA to fB (S to E)."""
    found = re.findall(r"\bf(\d+) \(([\d.]+)\)", section)
    for a, b, s, e in re.findall(r"\bf(\d+) to f(\d+) \(([\d.]+) to ([\d.]+)\)", section):
        found += [(a, s), (b, e)]
    return [(int(f), s) for f, s in found]


counted = Counter()
for chapter, section in sections.items():
    for frame, stated in frame_pairs(section):
        counted[chapter] += 1
        check(pair_ok(chapter, frame, stated), f"{chapter} f{frame} ({stated}), sampling says {times[chapter][frame] if frame < len(times[chapter]) else 'out of range'}")
    for frame in re.findall(r"\bf(\d+)\b", section):
        check(int(frame) < len(times[chapter]), f"{chapter} f{frame} out of range")
outside = re.sub(r"^## Chapter 0[6-9]:.*?(?=^## )", "", text, flags=re.M | re.S)
cross = re.findall(r"\b(0[6-9]) f(\d+) \(([\d.]+)\)", outside)
for chapter, frame, stated in cross:
    check(pair_ok(chapter, int(frame), stated), f"{chapter} f{frame} ({stated}), outside the chapter sections")
for frame, stated, chapter in re.findall(r"\| f(\d+) \(([\d.]+)\) to f\d+ \([\d.]+\)[^|]*, (0[6-9]) \|", outside):
    check(pair_ok(chapter, int(frame), stated), f"{chapter} f{frame} ({stated}), Medium table")
for chapter, frame, stated in re.findall(r"^\| (0[6-9]) \| f(\d+) \| ([\d.]+) \|", text, re.M):
    check(pair_ok(chapter, int(frame), stated), f"{chapter} f{frame} ({stated}), originals table")
print("frame and time pairs:", dict(counted), "plus", len(cross), "cross-chapter")

quotes = 0
for chapter, section in sections.items():
    voice = " ".join(v for k, v in script.items() if k.startswith(chapter)).lower()
    for q in re.findall(r'"([^"]+)"', section):
        quotes += 1
        check(q.lower() in voice, f'{chapter} voice quote not in script: "{q}"')
print("voice quotes:", quotes)

shares = re.findall(r'^\| (0[6-9]_\d) \| "([^"]+)" \| ([\d.]+) \|', text, re.M)
for cue, words, stated in shares:
    if words not in script[cue]:
        check(False, f'{cue} share words not in cue: "{words}"')
        continue
    before = len(script[cue][: script[cue].index(words)].split())
    start, length = cues[cue]
    estimate = start + length * before / len(script[cue].split())
    check(abs(estimate - float(stated)) <= 0.06, f"{cue} share {stated}, computed {estimate:.2f}")
print("word shares:", len(shares))

ANCHORS = {
    "one_derivative.rs": [(26, "z_0"), (44, "z_0"), (47, "z_0"), (211, "z_0"), (217, "z_0"), (67, "Plane"), (68, "Plane"), (544, "UP_TEX"), (680, "fade_out_all"), (691, "v(4.2, -1.3)"), (715, "cross2"), (725, "r1"), (726, "r2"), (732, "all.extend(cross1)")],
    "one_point.rs": [(42, "PRODUCT"), (51, "LONE"), (147, "squeeze_sample"), (237, "map_arrow"), (238, "v(-0.6, -0.15)"), (387, "names_now"), (389, "MULTIPLIED"), (390, "fade_out_all"), (396, "reading_window"), (698, "disks"), (714, "disks")],
    "loop_sums.rs": [(213, "CIRCULATION.1 - 0.35"), (293, "ring_head"), (311, "at_mid"), (312, "mid_name"), (443, "product_diagram"), (449, "gone"), (451, "fade_out_all"), (516, "BIGGER[0]"), (520, "wait_until"), (578, "SURVIVES"), (757, "a_circulation_mark_points_the_way_it_turns")],
    "boundary.rs": [(76, "foot"), (281, "CENTRE_ZERO"), (284, "C::new(0.85, -0.72)"), (287, "a_is"), (300, "v(right, -0.75)"), (506, "NEXT"), (577, "axis_end[0] + 0.18"), (633, "RADIUS")],
    "draw.rs": [(468, "right()"), (479, "arrow_head_points"), (554, "CELL")],
}
for name, rows in ANCHORS.items():
    lines = open(SRC + name, encoding="utf-8").read().split("\n")
    for number, needle in rows:
        check(needle in lines[number - 1], f"{name}:{number} lacks {needle!r}: {lines[number - 1].strip()[:70]}")
print("source anchors:", sum(len(r) for r in ANCHORS.values()))

sev = Counter(s for _, s in re.findall(r"^\| ((?:0[6-9]-N\d+)|S\d|G\d) \| (High|Medium|Low) \|", text, re.M))
prior = re.findall(r"^\| (0[6-9]-[A-L]) \| (High|Medium|Low) \| ([^|]+) \|", text, re.M)
residual = sum("(Low)" in now for _, _, now in prior)
low = sev["Low"] + residual
now = Counter(n.split(",")[0].split(" (")[0].strip() for _, _, n in prior)
print("new rows by severity:", dict(sev), "prior rows", len(prior), "prior now", dict(now), "prior residual Low", residual)
check(len(prior) == 38, f"prior rows {len(prior)}, expected 38 plus S1 and S2")
stated_medium = re.search(r"^\| Medium \| (\d+) \|", text, re.M)
stated_low = re.search(r"^\| Low \| (\S+) \|", text, re.M)
check(int(stated_medium.group(1)) == sev["Medium"], f"Medium count {stated_medium.group(1)} vs {sev['Medium']}")
check(stated_low.group(1) == str(low), f"Low count {stated_low.group(1)} vs {low}")
check(not sev["High"], "High rows present")
fixed_with_s1 = sum(n.startswith("Fixed") for _, _, n in prior) + 1
stated_fixed = re.search(r"40 findings for these chapters, (\S+) are fixed", text)
check(stated_fixed and stated_fixed.group(1) == str(fixed_with_s1), f"fixed count {stated_fixed.group(1) if stated_fixed else None} vs {fixed_with_s1}")

for chapter, name in CHAPTERS.items():
    folder = FRAMES + name + "/"
    sheets = sorted(n for n in os.listdir(folder) if n.startswith("sheet-") and n.endswith(".png"))
    check(len(sheets) == SHEETS[chapter], f"{chapter} sheets {len(sheets)}")
    check(len(times[chapter]) == (sum(n.startswith("frame-") for n in os.listdir(folder))), f"{chapter} frames against times")
    joined = hashlib.sha256()
    for n in sheets:
        joined.update(sha(folder + n).encode())
    check(joined.hexdigest() in text, f"{chapter} hash of sheet hashes not in review")
    check(f"({len(times[chapter])})" in text, f"{chapter} frame count not in review")

rows = re.findall(r"^\| `([^`]+)` \| `([0-9a-f]{64})` \|", text, re.M)
for path, digest in rows:
    check(os.path.exists(path) and sha(path) == digest, f"binding row differs: {path}")
print("binding rows:", len(rows))
check(len(rows) == 10, f"binding rows {len(rows)}")

# Planted controls: the same tests must reject wrong input.
control_pair = pair_ok("09", 223, "81.0")
control_quote = "the viewer never hears this" in " ".join(script.values()).lower()
check(not control_pair, "control: a wrong time passed")
check(not control_quote, "control: a wrong quote passed")
print("controls rejected:", (not control_pair) and (not control_quote))

print(f"RESULT {'FAIL ' + str(len(fails)) if fails else 'PASS'}")
