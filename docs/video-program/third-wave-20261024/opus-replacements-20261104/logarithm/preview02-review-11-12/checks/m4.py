"""Fourth pass: last clearances, phrase times by character share, gold coverage of the x2 arrow."""
import sys
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).parent))
import measure as M

rows = {}
for line in Path("explainer_corpus/logarithm_mysteries/script.tsv").read_text(encoding="utf-8").splitlines():
    p = line.split("\t")
    if len(p) >= 2:
        rows[p[0]] = p[1]
CUES = {}
ROOT = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-opus-rebuild-preview02/render")
for ch in ("11_unreachable", "12_what_it_forgets"):
    for line in (ROOT / f"{ch}.cues.tsv").read_text().splitlines():
        c, s, l = line.split("\t")
        CUES[c] = (float(s), float(l))


def parts(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start : i + 1].strip())
            start = i + 1
    return out


def phrase(cue, needle):
    start, length = CUES[cue]
    ps = parts(rows[cue])
    speech = length - 0.4
    whole = sum(len(s) for s in ps)
    before = 0
    for j, s in enumerate(ps):
        k = s.find(needle)
        if k >= 0:
            a = start + speech * (before + k) / whole
            b = start + speech * (before + k + len(needle)) / whole
            sa = start + speech * before / whole
            sb = start + speech * (before + len(s)) / whole
            print(f"  {cue} s{j} '{needle}': {a:.2f}..{b:.2f}  (sentence {sa:.2f}..{sb:.2f}; share of sentence at phrase end {(b - sa) / (sb - sa):.3f})")
            return
        before += len(s)
    print(f"  {cue}: NOT FOUND {needle}")


print("phrase times, scratch, character-share estimate inside the sentence")
for cue, needle in [
    ("11_1", "the letter F"),
    ("11_1", "squashes the whole plane flat onto a line"),
    ("11_2", "run backwards"),
    ("11_3", "The area is multiplied by plus one"),
    ("11_5", "send it along both routes"),
    ("11_5", "end in different places"),
    ("11_6", "running one after the other"),
    ("11_7", "come in matching pairs"),
    ("11_7", "blocks for each negative stretch"),
    ("12_2", "forgets the decade"),
    ("12_6", "one and a half laps per unit of time also ends here"),
    ("12_6", "also ends here"),
    ("12_6", "and so does two and a half"),
    ("12_6", "any odd number of half laps"),
    ("12_6", "so any other axis is ruled out"),
]:
    phrase(cue, needle)

print("x2 arrow row at frame y 387, f170 and f160")
for i in (160, 170):
    a = M.load("11", i)
    row = a[387, 190:316]
    gold = (row[:, 0] > 190) & (row[:, 1] > 150) & (row[:, 2] < 150)
    green = (row[:, 1] > 150) & (row[:, 0] < 170) & (row[:, 2] < 150)
    print(f"  f{i} t={M.TIMES['11'][i]:.3f}: x 190..315 gold px {int(gold.sum())}, green px {int(green.sum())}")
    for off in (-1, 0, 1):
        row = a[387 + off, 190:316]
        print(f"    y {387 + off}: mean colour by 21 px bins", [tuple(int(v) for v in row[k:k + 21].mean(axis=0)) for k in range(0, 126, 21)])

print("zero line and its label, f69")
a = M.load("11", 69)
sub = a[425:445, 670:780]
red = (sub[:, :, 0] > 150) & (sub[:, :, 1] < 140)
ys, xs = np.nonzero(red)
print(f"  red x {xs.min() + 670}..{xs.max() + 670}, y {ys.min() + 425}..{ys.max() + 425}")
M.bbox("11", 69, (670, 425, 690, 445), '"0" label')

print("end-ring box, sampled frame f287, red and gold")
print("first and last frames of each red bar sample around f284..f286 (frame px)")
for i in (283, 284, 285, 286):
    a = M.load("11", i)
    sub = a[150:620, 140:560]
    red = (sub[:, :, 0] > 200) & (sub[:, :, 1] < 130) & (sub[:, :, 2] < 120)
    print(f"  f{i} t={M.TIMES['11'][i]:.3f} red px {int(red.sum())}")

print("kept crops, default OUT")
for args in [
    ("11", 24, (140, 150, 560, 560), "11_f024_before.png", 2),
    ("11", 33, (140, 150, 560, 560), "11_f033_dashes.png", 2),
    ("11", 43, (140, 150, 560, 620), "11_f043_red.png", 2),
    ("11", 69, (660, 120, 1000, 560), "11_f069_readout.png", 2),
    ("11", 170, (140, 280, 560, 620), "11_f170_arrows.png", 2),
    ("11", 285, (140, 150, 560, 620), "11_f285_routes.png", 2),
    ("11", 365, (140, 150, 560, 620), "11_f365_axis.png", 2),
    ("11", 375, (880, 380, 1200, 440), "11_f375_key.png", 4),
    ("11", 524, (540, 110, 800, 240), "11_f524_purple.png", 3),
    ("12", 41, (730, 110, 960, 350), "12_f041_dial.png", 3),
    ("12", 62, (40, 500, 640, 640), "12_f062_lower_left.png", 2),
    ("12", 80, (720, 410, 1000, 590), "12_f080_flows.png", 3),
    ("12", 81, (720, 410, 1000, 590), "12_f081_flows.png", 3),
    ("12", 287, (200, 200, 540, 620), "12_f287_red.png", 2),
    ("12", 293, (200, 200, 540, 620), "12_f293_red.png", 2),
]:
    M.crop(*args)
