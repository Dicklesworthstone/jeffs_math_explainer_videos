#!/usr/bin/env python3
"""Run from the repo root: python3 -I -B <this file>

Which sheet holds which frame, at what time, under which sentence of the scratch
voice; the exact bytes of every sheet viewed; and, from the frame pixels, every
stretch in which nothing on screen changed.

Reads only. No render, no listening. Times are the scratch-voice preview's, not
Oliver's. A "still" stretch is between half-second samples: a change that starts
and ends between two samples cannot show here.
"""
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageChops

ROOT = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
FRAMES = ROOT / "algebra-chapter10-repair03-frames01/10_cyclic"
CUES = ROOT / "algebra-chapter10-repair03-preview01/render/10_cyclic.cues.tsv"
SCRIPT = Path("explainer_corpus/abstract_algebra_expanded/script.tsv")

sampling = json.loads((FRAMES / "sampling.json").read_text(encoding="utf-8"))
times = sampling["times"]
text = {
    row.split("\t", 1)[0]: row.split("\t", 1)[1]
    for row in SCRIPT.read_text(encoding="utf-8").splitlines()
    if row.startswith("10_")
}
cues = [(c, float(a), float(b)) for c, a, b in (row.split("\t") for row in CUES.read_text(encoding="utf-8").splitlines())]


def sentences(line):
    """The kit's rule: a sentence ends at . ? or ! followed by a space or the end."""
    out, start = [], 0
    for i, c in enumerate(line):
        if c in ".?!" and (i + 1 == len(line) or line[i + 1] == " "):
            out.append(line[start : i + 1].strip())
            start = i + 1
    return out


def scratch_starts(cue, length):
    """Scratch speech is the cue length less 0.4 s; a sentence's share is by characters."""
    parts = sentences(text[cue])
    total = sum(len(p) for p in parts)
    speech = length - 0.4
    out, at = [], 0.0
    for part in parts:
        out.append(at)
        at += speech * len(part) / total
    return out


def where(t):
    last = None
    for row in cues:
        if row[1] <= t:
            last = row
    if last is None:
        return "before"
    cue, start, length = last
    rel = t - start
    if rel >= length:
        return f"{cue}+gap"
    starts = scratch_starts(cue, length)
    return f"{cue}s{max(j for j, s in enumerate(starts) if s <= rel)}"


print("== sampling.json and the folder")
print("duration", sampling.get("duration"), "| samples", len(times), "| scope:", sampling.get("scope"))
sheet_paths = sorted(FRAMES.glob("sheet-*.png"))
frame_paths = sorted(FRAMES.glob("frame-*.png"))
print("sheet files", len(sheet_paths), "| frame files", len(frame_paths))
print("sheets expected at 4 frames each:", (len(times) + 3) // 4)
print("times strictly increasing:", all(a < b for a, b in zip(times, times[1:])))
print("largest step between samples:", round(max(b - a for a, b in zip(times, times[1:])), 3))
print("sampling.json sha256", hashlib.sha256((FRAMES / "sampling.json").read_bytes()).hexdigest())
with Image.open(sheet_paths[0]) as im:
    print("sheet size", im.size)
with Image.open(frame_paths[0]) as im:
    print("frame size", im.size)

print("\n== cues, from the preview's cue table, with scratch sentence starts")
for cue, start, length in cues:
    print(cue, f"start {start:.3f} len {length:.3f} end {start + length:.3f}", [round(start + s, 2) for s in scratch_starts(cue, length)])

print("\n== every sheet viewed, in order: index, sha256, frame@time=cue+sentence")
all_digest = hashlib.sha256()
for index, path in enumerate(sheet_paths):
    assert path.name == f"sheet-{index:03}.png", path.name
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    all_digest.update(f"{digest}  {path.name}\n".encode())
    span = range(index * 4, min(index * 4 + 4, len(times)))
    print(f"{index:03} {digest} " + " ".join(f"{i}@{times[i]:.2f}={where(times[i])}" for i in span))
print("sha256 of the list of sheet hashes:", all_digest.hexdigest())

print("\n== picture changes between neighbouring samples, from the frame pixels")
# Reduced greyscale removes single-pixel encoder noise. A block counts as changed
# at 14 levels of 255. The threshold is checked against known moments below.
reduced = []
for path in frame_paths:
    with Image.open(path) as im:
        reduced.append(im.convert("L").resize((320, 180), Image.BILINEAR))
changed = []
for a, b in zip(reduced, reduced[1:]):
    histogram = ImageChops.difference(a, b).histogram()
    changed.append(sum(histogram[14:]))

print("\n== stretches with no change for 5 s or more (samples, times, cue+sentence at each end)")
run_start = 0
for i in range(1, len(times) + 1):
    if i == len(times) or changed[i - 1] > 0:
        if times[i - 1] - times[run_start] >= 5.0:
            print(
                f"frames {run_start}-{i - 1}  {times[run_start]:.2f}-{times[i - 1]:.2f}  "
                f"{times[i - 1] - times[run_start]:.2f} s  {where(times[run_start])} to {where(times[i - 1])}"
            )
        run_start = i

print("\n== positive control: moments the sheets show as changing, and as not changing")
for frame, want in ((536, True), (551, True), (505, True), (66, True), (598, True), (603, True), (520, False), (100, False), (30, False), (615, False)):
    print(f"frames {frame - 1} to {frame}: {'change' if changed[frame - 1] > 0 else 'still'} (expected {'change' if want else 'still'}), pixels over threshold {changed[frame - 1]}")

print("\nsentences per cue", {c: len(sentences(text[c])) for c, _, _ in cues})
