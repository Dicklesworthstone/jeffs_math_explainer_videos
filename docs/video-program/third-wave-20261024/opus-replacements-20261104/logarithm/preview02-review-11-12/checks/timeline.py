"""Scratch sentence times for logarithm chapters 11 and 12 by the kit's rule. Read-only."""
from pathlib import Path

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


for cue, (start, length) in CUES.items():
    text = rows[cue]
    ps = parts(text)
    speech = length - 0.4
    whole = sum(len(s) for s in ps)
    before = 0
    print(f"{cue} {start:.3f}..{start + length:.3f}")
    for k, s in enumerate(ps):
        t = start + speech * before / whole
        print(f"   s{k} {t:8.3f}  {s[:90]}")
        before += len(s)
