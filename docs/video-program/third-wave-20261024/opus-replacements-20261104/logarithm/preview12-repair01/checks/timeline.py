"""The new beats' timing under preview01's scratch-voice sentence starts (the kit's
estimated_starts rule), checked against what the old source does on the sampled frames."""
import re

rows = {}
for line in open("explainer_corpus/logarithm_mysteries/script.tsv", encoding="utf-8"):
    p = line.rstrip("\n").split("\t")
    if len(p) >= 2:
        rows[p[0]] = p[1]


def sentences(t):
    out, start = [], 0
    b = t.encode()
    for i, c in enumerate(b):
        if c in b".?!" and (i + 1 == len(b) or b[i + 1] == 32):
            s = b[start : i + 1].decode().strip()
            if s:
                out.append(s)
            start = i + 1
    r = b[start:].decode().strip()
    if r:
        out.append(r)
    return out


def starts(cue, length):
    parts = sentences(rows[cue])
    total = sum(len(s) for s in parts)
    before, out = 0, []
    for s in parts:
        out.append(length * before / total)
        before += len(s)
    return parts, out


CUES = {"12_3": (31.867, 17.708), "12_4": (50.200, 36.554), "12_5": (87.367, 12.323), "12_6": (104.567, 58.092)}
abs_s = {}
for cue, (at, ln) in CUES.items():
    parts, st = starts(cue, ln)
    abs_s[cue] = [at + x for x in st] + [at + ln]
    print(cue, "sentences", len(parts), "starts", [round(at + x, 2) for x in st], "ends", round(at + ln, 2))


def seg(cue, j):
    return abs_s[cue][j], abs_s[cue][j + 1] - abs_s[cue][j]


print("\nvalidation of the model against the old source on the sheets:")
a, l = seg("12_6", 5)
print(f"  12_6 s5 opens {a:.2f}: old build line 0.4, window 0.4, ends 0.5, tie 0.4, source 0.4 -> tie {a+1.3:.2f}, source {a+1.7:.2f}; sheets: tie at the 143.5 frame, precisely at the 144.0 frame")
a, l = seg("12_6", 3)
print(f"  12_6 s3 opens {a:.2f}: old red spin {a+.3:.2f}..{a+3.1:.2f}; sheets: readout runs 124.0..127.0 (F1)")
a, l = seg("12_4", 3)
print(f"  12_4 s3 opens {a:.2f}: old spin {a+.6:.2f}..{a+3.6:.2f}; review: 71.5..74.5 (F4)")

print("\nnew beats under the same scratch timing (code reads the real starts at render):")
a, l = seg("12_4", 2)
run = max(l - .4 - .6, 1)
print(f"  12_4 s2 {a:.2f}..{a+l:.2f} ({l:.2f} s): one lap {a+.4:.2f}..{a+.4+run:.2f} ({run:.2f} s); {l-.4-run:.2f} s to spare")
a, l = seg("12_4", 3)
run = max(l - .6 - .8, 1)
print(f"  12_4 s3 {a:.2f}..{a+l:.2f} ({l:.2f} s): half lap {a+.6:.2f}..{a+.6+run:.2f} ({run:.2f} s); {l-.6-run:.2f} s to spare")
a, l = seg("12_6", 3)
run = max(.45 * l - .3, 1)
print(f"  12_6 s3 {a:.2f}..{a+l:.2f} ({l:.2f} s): red spin {a+.3:.2f}..{a+.3+run:.2f} ({run:.2f} s), holds to {a+.6*l:.2f} ({a+.6*l-(a+.3+run):.2f} s), back, chip at {a+.79*l:.2f}, sentence 4 at {a+l:.2f}; {l-.79*l-.4:.2f} s after the chip's fade")
a, l = seg("12_6", 5)
print(f"  12_6 s5 {a:.2f}..{a+l:.2f} ({l:.2f} s): line, window, dots done {a+1.3:.2f}; tie at {a+.45*l:.2f}; source and convention at {a+l:.2f}")

w3 = sentences(rows["12_6"])[3].split()
print("\n12_6 s3 words", len(w3))
for k in ("where", "and", "only", "so"):
    i = w3.index(k)
    print(f"  '{k}' word {i+1}: {100*i/len(w3):.0f}%")
print("  'it was,' ends:", f"{100*(w3.index('and'))/len(w3):.0f}%")
w5 = sentences(rows["12_6"])[5].split()
print("12_6 s5 words", len(w5), "| 'the very edge' at", f"{100*w5.index('very')/len(w5):.0f}%", "| 'no principal logarithm' from", f"{100*w5.index('no')/len(w5):.0f}%")

line_end = 87.367 + 12.323
print(f"\nF6 think bar: line ends {line_end:.2f}; old: bar from the call at about 97.4 s; new: bar opens {line_end+.6:.2f}, empties {line_end+.6+4:.2f}, fade ends {line_end+.6+4+.25:.2f}; next cue 104.567 unchanged")

# literals with comments stripped
ROOT = "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm/preview12-repair01/explainer_corpus/logarithm_mysteries/src/chapters/"
CANON = "explainer_corpus/logarithm_mysteries/src/chapters/"


def code(path):
    t = open(path, encoding="utf-8").read().split("#[cfg(test)]")[0]
    return "\n".join(ln.split("//")[0] for ln in t.split("\n"))


lit = re.compile(r'"((?:[^"\\]|\\.)*)"')
a, b = lit.findall(code(CANON + "what_it_forgets.rs")), lit.findall(code(ROOT + "what_it_forgets.rs"))
from collections import Counter

print("\nscene-code literals, comments stripped: removed", sorted((Counter(a) - Counter(b)).elements()) or "none", "| added", sorted((Counter(b) - Counter(a)).elements()) or "none")
