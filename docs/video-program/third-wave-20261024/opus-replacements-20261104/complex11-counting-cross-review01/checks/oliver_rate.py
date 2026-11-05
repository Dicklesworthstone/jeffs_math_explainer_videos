"""Oliver's characters a second, from an existing narration of another film (Parrondo, narration02).

Each cue's character count over its WAV's length. That gives a rate to reason with, and nothing about
any take of 11_7, which does not exist yet: the sentences and pauses of the real take will differ.

Usage: python3 -I -B oliver_rate.py REPO
Prints: overall slowest fastest, on its last line.
"""

import glob
import os
import statistics
import sys
import wave

repo = sys.argv[1]
narration = os.path.join(repo, "narration_lab/third-wave-20261024/root/elevenlabs-parrondo-20261028/narration02")
script = {}
for path in glob.glob(os.path.join(repo, "explainer_corpus/parrondo*/script.tsv")):
    for row in open(path, encoding="utf8"):
        if "\t" in row:
            cue, text = row.rstrip("\n").split("\t", 1)
            script[cue] = text
rows = []
for cue, text in sorted(script.items()):
    path = os.path.join(narration, f"{cue}.wav")
    if os.path.exists(path):
        with wave.open(path) as f:
            rows.append((len(text) / (f.getnframes() / f.getframerate()), len(text), f.getnframes() / f.getframerate()))
if not rows:
    raise SystemExit("no takes found")
overall = sum(r[1] for r in rows) / sum(r[2] for r in rows)
rates = [r[0] for r in rows]
print(f"{len(rows)} cues of {len(script)}; median {statistics.median(rates):.2f} characters a second")
print(f"{overall:.2f} {min(rates):.2f} {max(rates):.2f}")
