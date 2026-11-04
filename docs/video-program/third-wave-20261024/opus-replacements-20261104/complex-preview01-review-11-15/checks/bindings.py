#!/usr/bin/env python3
"""Count and hash the reviewed pictures. Run from the repo root. Reads only."""
import hashlib
import json
import os

BASE = "narration_lab/third-wave-20261024/root/five-film-release-20261027/"
FRAMES = BASE + "complex-opus-rebuild-frames01/"
RENDER = BASE + "complex-opus-rebuild-preview01/render/"
SOURCE = "explainer_corpus/complex_analysis_visual/"
CHAPTERS = ["11_counting", "12_residues", "13_real_integral", "14_continuation", "15_closing"]
FILES = ["script.tsv", "src/facts.rs", "src/model.rs", "src/chapters/draw.rs", "src/chapters/mod.rs",
         "src/chapters/counting.rs", "src/chapters/residues.rs", "src/chapters/real_integral.rs",
         "src/chapters/continuing.rs", "src/chapters/closing.rs"]


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def digest(folder, names):
    whole = hashlib.sha256()
    for name in names:
        whole.update(name.encode() + b" " + sha(folder + name).encode() + b"\n")
    return whole.hexdigest()


total_sheets = total_frames = 0
for chapter in CHAPTERS:
    folder = FRAMES + chapter + "/"
    names = sorted(os.listdir(folder))
    sheets = [n for n in names if n.startswith("sheet-")]
    frames = [n for n in names if n.startswith("frame-")]
    sampling = json.load(open(folder + "sampling.json", encoding="utf-8"))
    times = sampling["times"]
    assert len(times) == len(frames), chapter
    assert len(sheets) == (len(frames) + 3) // 4, chapter
    assert sheets == [f"sheet-{i:03}.png" for i in range(len(sheets))], chapter
    assert frames == [f"frame-{i:04}.png" for i in range(len(frames))], chapter
    total_sheets += len(sheets)
    total_frames += len(frames)
    print(f"{chapter}: sheets 000-{len(sheets) - 1:03} ({len(sheets)}), frames 0-{len(frames) - 1} ({len(frames)}), "
          f"{times[0]:.3f}-{times[-1]:.3f} s of {sampling['duration']:.3f} s")
    print(f"  sheets digest {digest(folder, sheets)}")
    print(f"  frames digest {digest(folder, frames)}")
    print(f"  sampling.json {sha(folder + 'sampling.json')}")
    for suffix in (".mp4", ".cues.tsv", ".layout.tsv"):
        print(f"  {chapter}{suffix} {sha(RENDER + chapter + suffix)}")
print(f"total: {total_sheets} sheets, {total_frames} frames")
print("source at review time:")
for name in FILES:
    print(f"  {sha(SOURCE + name)}  {name}")
