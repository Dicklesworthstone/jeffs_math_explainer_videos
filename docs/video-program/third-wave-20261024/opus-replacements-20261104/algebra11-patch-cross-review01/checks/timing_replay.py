#!/usr/bin/env python3
"""Run from the repo root. Sentence windows, picture blocks and phrase sync for the patched cues.

The kit's scratch rule (crates/explainer_kit/src/narration.rs): speech = words / 2.6, cue length = speech + 0.4,
sentence starts by character share of the speech. Chapter 11 has no local Oliver takes, so the Oliver column is
only this rule scaled by the ratio measured on retained takes from other chapters. Nothing here is a measurement of
chapter 11's voice. Block durations are read from the candidate's source by hand and written out term by term.
"""
import json
import re
import statistics

SCRIPT = "explainer_corpus/abstract_algebra_expanded/script.tsv"
CUES = "narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-combined-repair-preview01/render/11_permutations.cues.tsv"
INVENTORY = "narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/combined-repair-stage01/retained-recordings-inventory01.json"
script = {}
for row in open(SCRIPT, encoding="utf-8").read().splitlines():
    if "\t" in row:
        cue, text = row.split("\t", 1)
        script[cue] = text.strip()


def sentences(text):
    return [s.strip() for s in re.split(r"(?<=[.?!])(?= |$)", text) if s.strip()]


def speech(text):
    return len(text.split()) / 2.6


def starts(text, scale=1.0):
    parts = sentences(text)
    total = sum(len(p) for p in parts)
    before, out = 0, []
    for p in parts:
        out.append(speech(text) * scale * before / total)
        before += len(p)
    return out


def window(cue, a, b=None, scale=1.0):
    """Seconds from sentence a to sentence b, or to the end of speech."""
    s = starts(script[cue], scale)
    end = s[b] if b is not None and b < len(s) else speech(script[cue]) * scale
    return end - s[a]


print("== model against the baseline render's cue table")
rendered = {r.split("\t")[0]: r.split("\t") for r in open(CUES, encoding="utf-8").read().splitlines() if r.startswith("11_")}
errors = [abs(float(rendered[c][2]) - (speech(script[c]) + 0.4)) for c in rendered]
print(f"{len(rendered)} cues; largest difference between cue length and words/2.6 + 0.4: {max(errors):.3f} s")
s3 = float(rendered["11_24"][1]) + starts(script["11_24"])[3]
print(f"11_24 sentence 3 starts {s3:.3f}; plus 0.4 + 8 x (0.5 + 0.3) = {s3 + 6.8:.3f}; the render places 11_25 at {rendered['11_25'][1]}. Baseline is picture-driven here.")

print("\n== Oliver against the scratch rule, from retained takes in other chapters (local inventory, take files only)")
inv = json.load(open(INVENTORY, encoding="utf-8"))
ratios = []
for path, row in inv["verified_takes"].items():
    m = re.search(r"/(\d\d_\d+)-take\d+\.wav$", path)
    if m and m.group(1) in script and path.count("/") and row.get("sample_rate"):
        ratios.append((row["frames"] / row["sample_rate"]) / (speech(script[m.group(1)]) + 0.4))
ratios.sort()
q = statistics.quantiles(ratios, n=10)
print(f"{len(ratios)} takes whose cue text may or may not be current; duration / scratch length: min {ratios[0]:.2f}, p10 {q[0]:.2f}, median {statistics.median(ratios):.2f}, p90 {q[8]:.2f}, max {ratios[-1]:.2f}")
print("chapter 11 takes in that inventory:", sum("/11_" in p for p in inv["verified_takes"]), "| missing_by_chapter['11']:", inv["missing_by_chapter"].get("11"))
SLOW, FAST = q[8], q[0]

BLOCKS = [
    ("I 11_5 s0-1", "11_5", 0, 2, [("overlap out", 0.4), ("name and cards in", 0.4), ("the swap", 0.9)], 1.70, 5.08),
    ("I 11_5 s2", "11_5", 2, 3, [("cards out", 0.4), ("factor in", 0.4), ("board in", 0.4)], 1.20, 8.99),
    ("T 11_20 s0", "11_20", 0, 1, [("title", 0.4), ("row", 0.4), ("4 trades", 2.4), ("rest", 0.3)], 3.50, 7.00),
    ("V 11_21 s3", "11_21", 3, 4, [("recolour left", 0.3), ("recolour right", 0.3), ("keys", 0.4), ("pulse", 0.6), ("pulse", 0.6)], 2.20, 6.23),
    ("AA 11_22 s2", "11_22", 2, 3, [("reading", 0.4), ("ring in", 0.4), ("6 hops", 6.9), ("ring out", 0.4)], 8.10, 9.76),
    ("AA 11_22 s3", "11_22", 3, 4, [("reading out", 0.4), ("ring in", 0.4), ("2 hops", 2.3), ("ring out", 0.4)], 3.50, 5.08),
    ("AE 11_25 s0", "11_25", 0, 1, [("out", 0.4), ("in", 0.4), ("one shuffle", 0.8), ("count", 0.3)], 1.90, 5.09),
    ("AE 11_25 s2", "11_25", 2, 3, [("7 shuffles and counts", 7 * 0.58), ("out", 0.4), ("in", 0.4)], 4.86, 6.66),
    ("AE 11_26 s1", "11_26", 1, 2, [("out", 0.4), ("in", 0.4)], 0.80, 5.47),
    ("AI 11_26 s3", "11_26", 3, 4, [("3 stages", 2.1), ("2 rests", 0.5), ("caption", 0.4)], 3.00, 5.87),
    ("AM 11_27 s1", "11_27", 1, 2, [("first shuffle", 0.45), ("p^-1", 0.4), ("rest", 0.3), ("7 shuffles", 7 * 0.22)], 2.69, 2.72),
    ("G 11_14 s3", "11_14", 3, 4, [("out", 0.4), ("in", 0.4)], 0.80, 8.99),
    ("J 11_17 s4", "11_17", 4, 5, [("out", 0.4), ("in", 0.4)], 0.80, 6.64),
]
print("\n== picture blocks against their sentence windows; README's figures in brackets")
print(f"{'item':14} {'block':>6} {'[README]':>9} {'window':>7} {'[README]':>9} {'spare':>6} | spare at p10 x{FAST:.2f}, at p90 x{SLOW:.2f}")
for name, cue, a, b, terms, readme_block, readme_window in BLOCKS:
    block = sum(t for _, t in terms)
    w = window(cue, a, b)
    print(f"{name:14} {block:6.2f} [{readme_block:7.2f}] {w:7.2f} [{readme_window:7.2f}] {w - block:6.2f} | {window(cue, a, b, FAST) - block:6.2f}, {window(cue, a, b, SLOW) - block:6.2f}")

print("\n== 11_24 to 11_25: the payoff")
text = script["11_24"]
start3 = starts(text)[3]
tail3 = speech(text) + 0.4 - start3
print(f"sentence 3 voice: {tail3:.2f} s to cue end. Picture from its start: baseline 0.4 + 8 x 0.8 = 6.80; candidate + 0.6 + 0.6 = 8.00")
print(f"silence between 11_24's end and 11_25: baseline {6.8 - tail3:.2f}, candidate {8.0 - tail3:.2f}. README's push: 1.20. Direction policy for a cue transition: 0.45-1.4 s")
print(f"last bar lands at 6.80 - 0.30 = 6.50; 'Shuffles: 8' fully up at 6.80; the '8' starts to leave at 8.00, gone at 8.40. Staircase still until the first in-shuffle at 8.80. Baseline: count up 6.80, and 11_25's crossfade starts at 6.80")
print(f"the same silence at p10 and p90 voice: {8.0 - tail3 * FAST:.2f}, {8.0 - tail3 * SLOW:.2f}. It remains picture-driven.")

print("\n== 11_20 sentence 0: words against the trades")
t0 = script["11_20"]
sent = sentences(t0)[0]
w0 = window("11_20", 0, 1)
for phrase in ("since they compose", "and each one can be undone"):
    print(f"  '{phrase}' starts {w0 * sent.index(phrase) / len(sent):.2f} s into the sentence")
print(f"  candidate: trades 0.80-2.00 and 2.30-3.50; all done before 'since' starts")
at = w0 * sent.index("since they compose") / len(sent)
print(f"  with until_phrase on 'since they compose': trades {at:.2f}-{at + 1.2:.2f} and {at + 1.5:.2f}-{at + 2.7:.2f}; window ends {w0:.2f}; spare {w0 - at - 2.7:.2f}")
for label, scale in (("p10", FAST), ("p90", SLOW)):
    print(f"    at {label}: phrase {at * scale:.2f}, trades end {at * scale + 2.7:.2f}, window {w0 * scale:.2f}, spare {w0 * scale - at * scale - 2.7:.2f}")

print("\n== 11_22 sentence 2: the ring's hops against the words, scratch rate")
s2 = sentences(script["11_22"])[2]
w2 = window("11_22", 2, 3)
marks = ["Two goes to three,", "three to five,", "five to nine,", "nine to eight,", "eight to six,", "and six back to two,", "which closes"]
ends = [w2 * (s2.index(m) + (len(m) if i < 6 else 0)) / len(s2) for i, m in enumerate(marks)]
base_t, cand_t = 0.4, 0.8
for i, m in enumerate(marks[:6]):
    a, b = base_t + 1.15 * (i + 1), cand_t + 1.15 * (i + 1)
    print(f"  hop {i + 1} words end {ends[i]:5.2f} | ring lands: baseline {a:5.2f} ({a - ends[i]:+.2f}), candidate {b:5.2f} ({b - ends[i]:+.2f})")

print("\n== 11_9 sentence 3: the rest against the anchors (X)")
s3t = sentences(script["11_9"])[3]
w3 = window("11_9", 3, 4)
rs, sr = s3t.index("r s is the swap"), s3t.index("and s r is the swap")
gap = w3 * (sr - rs) / len(s3t)
print(f"  from 'r s' to 'and s r': {gap:.2f} s. Left picture: ring 0.4, moves 0.6 + 0.35 + 0.6, tex 0.4 = 2.35 (baseline 2.00). Delay past 'and s r': {max(0, 2.35 - gap):.2f} (baseline {max(0, 2.00 - gap):.2f})")
after_sr = w3 - w3 * sr / len(s3t)
print(f"  from 'and s r' to sentence 4: {after_sr:.2f} s. Right picture needs 2.35. Spare when it starts on time: {after_sr - 2.35:.2f}")
print(f"  sentence 4 is {window('11_9', 4):.2f} s long. How late its picture lands, candidate (baseline):")
for label, scale in (("scratch", 1.0), ("p10", FAST), ("median", statistics.median(ratios)), ("p90", SLOW)):
    g, a = gap * scale, after_sr * scale
    def late(block):
        delay = max(0, block - g)
        return max(0, delay + block - a)
    print(f"    {label} x{scale:.2f}: {late(2.35):.2f} ({late(2.00):.2f})")

print("\n== Why README's windows differ: it takes each sentence's share of WORDS of the whole cue length; the kit takes its share of CHARACTERS of the speech")
for name, cue, a, b, readme in (("AA s2", "11_22", 2, 3, 9.76), ("AA s3", "11_22", 3, 4, 5.08), ("I s0-1", "11_5", 0, 2, 5.08), ("AM s1", "11_27", 1, 2, 2.72), ("T s0", "11_20", 0, 1, 7.00)):
    parts = sentences(script[cue])[a:b]
    words = sum(len(p.split()) for p in parts)
    share = words / len(script[cue].split()) * (speech(script[cue]) + 0.4)
    print(f"  {name}: word share of cue length {share:.2f} (README {readme:.2f}); kit rule {window(cue, a, b):.2f}")

print("\n== With the follow-up patch")
print(f"  T: see above; picture spare {window('11_20', 0, 1) - at - 2.7:.2f}")
w5 = window("11_21", 5)
print(f"  V sentence 5: keys out 0.4, recolour 0.3 + 0.3, kinds 0.4 = 1.40 in {w5:.2f}; spare {w5 - 1.4:.2f}")
print(f"  AA sentence 2: caption and ring up together, 0.4, 6 hops 6.9, ring out 0.4 = 7.70 in {w2:.2f}; spare {w2 - 7.7:.2f}, at p10 {window('11_22', 2, 3, FAST) - 7.7:.2f}. Hops land as in baseline, +0.22 to +0.60")
for name, cue, j, block in (("11_8 s4", "11_8", 4, 0.8), ("11_12 s3, then rest 0.6, rings out 0.4", "11_12", 3, 1.8), ("11_14 s5", "11_14", 5, 0.8), ("11_27 s5", "11_27", 5, 0.8)):
    end = j + 1 if j + 1 < len(sentences(script[cue])) else None
    print(f"  swap_caption {name}: {block:.2f} in {window(cue, j, end):.2f}; spare {window(cue, j, end) - block:.2f}, at p10 {window(cue, j, end, FAST) - block:.2f}")

print("\n== Reading time of the new words at 15 characters a second")
for name, text, on in [
    ("AA reading", "Card 2 went from slot 2 to slot 3", window("11_22", 2, 3) - 0.4),
    ("I name", "Transposition: swap two labels", window("11_5", 0, 2) - 0.8),
    ("V keys", "Gold: even Blue: odd", window("11_21", 3, 5) - 1.0),
    ("AC", "By size: 2 fixed, 1 swap, 6 cycles of length 8", window("11_24", 1) + 0),
    ("AH", "After 3 stages: all data back on their starting lines", window("11_26", 3) - 3.0),
]:
    print(f"  {name}: {len(text)} chars need {len(text) / 15:.2f} s; on screen at least {on:.2f} s at scratch rate")
