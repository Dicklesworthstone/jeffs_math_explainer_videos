#!/usr/bin/env python3
"""How hard would the narration limiter work at each ceiling? Runs
build_v2.limit_peaks over unlimited line WAVs (sentences at SENTENCE_LUFS)
and reports, per ceiling: time with >1 dB and >3 dB of reduction, the worst
reduction, and the lines that need the most.

    uv run --with numpy python3 limiter_sweep.py UNLIMITED_DIR
"""
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).parent))
import build_v2 as b  # noqa: E402

lines = {w.stem: b.load(w)[0] for w in sorted(Path(sys.argv[1]).glob("*.wav"))}
total = sum(len(x) for x in lines.values())
for headroom in (13, 14, 15, 16, 17):
    ceiling = 10 ** ((b.SENTENCE_LUFS + headroom) / 20)
    over1 = over3 = 0
    worst = []
    for name, x in lines.items():
        y, _, max_db = b.limit_peaks(x, ceiling)
        gain_db = -20 * np.log10(np.maximum(np.abs(y), 1e-12) / np.maximum(np.abs(x), 1e-12))
        gain_db[np.abs(x) < 1e-6] = 0.0
        over1 += int(np.sum(gain_db > 1.0))
        over3 += int(np.sum(gain_db > 3.0))
        worst.append((round(max_db, 1), name))
    worst.sort(reverse=True)
    print(f"ceiling loudness+{headroom} dB: >1 dB for {100 * over1 / total:.2f}% of samples, "
          f">3 dB for {100 * over3 / total:.3f}%, worst {worst[0][0]} dB; top: {worst[:4]}")
