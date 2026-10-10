#!/usr/bin/env python3
"""Where are a line's outlier peaks, and are they speech or glitches?

For each named line WAV, list the top peaks (time, level above the line's
loudness) with the local energy around them. A click is a narrow spike over
quiet surroundings; a plosive is a ~5-20 ms burst inside speech. Prints the
ASR words nearest each peak from the line transcript, if given.

    uv run --with numpy python3 peak_probe.py DIR LINE... [--asr lines_asr.json]
"""
import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).parent))
import build_v2 as b  # noqa: E402

args = sys.argv[1:]
asr = {}
if "--asr" in args:
    k = args.index("--asr")
    asr = json.loads(Path(args[k + 1]).read_text())
    args = args[:k] + args[k + 2:]
d, names = Path(args[0]), args[1:]
for name in names:
    x, rate = b.load(d / f"{name}.wav")
    lufs = b.integrated_lufs(x)
    a = np.abs(x)
    order = np.argsort(a)[::-1]
    picked = []
    for i in order:
        if all(abs(i - j) > rate * 0.05 for j in picked):
            picked.append(i)
        if len(picked) == 4:
            break
    print(f"{name}: {lufs} LUFS")
    for i in picked:
        over = 20 * np.log10(a[i]) - lufs
        w5 = a[max(0, i - int(0.0025 * rate)):i + int(0.0025 * rate)]
        w50 = x[max(0, i - int(0.025 * rate)):i + int(0.025 * rate)]
        narrow = 20 * np.log10(np.sqrt(np.mean(w5 ** 2)) / (np.sqrt(np.mean(w50 ** 2)) + 1e-12))
        words = [w["w"] for w in asr.get(name, {}).get("words", []) if w["s"] - 0.15 <= i / rate <= w["e"] + 0.15]
        print(f"   t={i / rate:6.3f}s  peak {over:+5.1f} dB over loudness  "
              f"5ms-vs-50ms energy {narrow:+5.1f} dB  near {words}")
