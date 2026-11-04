#!/usr/bin/env python3
"""Run the real tools/narration_delivery.py prediction_end on realistic placements.

Run from the repo root with: python3 -I -B <this file>
The placements mirror the native order in draw::predict: Narrator::hold waits to
voice end + BREATH, the mark lands there, then music_breath(THINK) and a 0.25 s
fade come before the next say. Synthetic arithmetic only: no native timeline,
caption file, score, picture or audio is involved.
"""

from collections import Counter
import importlib.util
import json
import math
from pathlib import Path
import re

spec = importlib.util.spec_from_file_location("narration_delivery", "tools/narration_delivery.py")
delivery = importlib.util.module_from_spec(spec)
spec.loader.exec_module(delivery)

BREATH, THINK, FADE = 0.6, 4.0, 0.25
SAY = re.compile(r'kit\.say\(stage, "([^"]+)"\)')
STAGE = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage02/explainer_corpus/logarithm_mysteries/"


def covering(seconds, fps):
    """explainer_kit::narration::frames_covering."""
    return max(0, math.ceil(seconds * fps - 1e-6))


def placed(fps, start_frame, samples, rate):
    voice_end = start_frame / fps + samples / rate
    mark = covering(voice_end + BREATH, fps)
    next_frame = mark + covering(THINK, fps) + covering(FADE, fps)
    line = ["line", "04_6", str(start_frame), "0", "x.wav"]
    following = ["line", "04_7", str(next_frame), "0", "y.wav"]
    outputs = {"04_6": {"frames": samples, "sample_rate": rate}}
    return line, following, outputs, mark, next_frame, voice_end


print("case | fps | result | expected")
for fps in (30, 60):
    for samples in (401_234, 441_000, 370_001):
        line, following, outputs, mark, next_frame, voice_end = placed(fps, 600, samples, 44_100)
        predicts = [mark / fps]
        once = Counter({("04_6", "04_7"): 1})
        offset = 12.5
        got = delivery.prediction_end(line, following, fps, predicts, outputs, once, offset)
        want = offset + next_frame / fps
        print(f"declared | {fps} | {got} | {want}")
        assert got == want
        margin = mark / fps - voice_end
        assert 0.6 - 1e-6 <= margin < 0.6 + 1.0 / fps + 1e-6, margin
        print(f"  mark after voice end {margin:.4f} s, next cue after mark {(next_frame - mark) / fps:.4f} s")

line, following, outputs, mark, next_frame, _ = placed(60, 600, 401_234, 44_100)
predicts = [mark / 60]
for name, declared, follow, marks in (
    ("undeclared pair", Counter(), following, predicts),
    ("declared twice", Counter({("04_6", "04_7"): 2}), following, predicts),
    ("last line of a chapter", Counter({("04_6", "04_7"): 1}), None, predicts),
    ("declared, no mark", Counter({("04_6", "04_7"): 1}), following, []),
):
    got = delivery.prediction_end(line, follow, 60, marks, outputs, declared, 0.0)
    print(f"{name} | 60 | {got} | None")
    assert got is None

order = ["ratio_ruler", "building_ruler", "slide_rule", "folded_decade", "undoing_growth", "complex_turns",
         "lifting_loops", "branch_cut", "product_rule", "matrix_logs", "doing_nothing", "unreachable",
         "what_it_forgets"]
pairs = []
for chapter in order:
    source = Path(STAGE + "src/chapters/" + chapter + ".rs").read_text(encoding="utf-8").split("#[cfg(test)]")[0]
    lines = source.splitlines()
    for i, text in enumerate(lines):
        if re.search(r"\bpredict\(&kit", text):
            before = next(SAY.search(lines[k]).group(1) for k in range(i - 1, -1, -1) if SAY.search(lines[k]))
            after = next(SAY.search(lines[k]).group(1) for k in range(i + 1, len(lines)) if SAY.search(lines[k]))
            pairs.append((before, after))
print("source pairs:", len(pairs), pairs)

maps = sorted(Path(STAGE + "docs").glob("narration-direction-map-oliver*.json"))
newest = json.loads(maps[-1].read_text(encoding="utf-8"))
declared = {(h["cue"], h["next_cue"]) for h in newest.get("visual_holds", []) if h.get("kind") == "prediction"}
print("newest stage map:", maps[-1].name, "declares", sorted(declared))
print("source pairs declared by it:", len(set(pairs) & declared), "of", len(pairs))
