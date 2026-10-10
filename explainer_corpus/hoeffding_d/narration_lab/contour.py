#!/usr/bin/env python3
"""Print a read's pitch contour (semitones re: its median, per 50 ms) with
the ASR words underneath, to tell genuine uptalk from tracker octave errors.

    uv run --with numpy python3 contour.py WORK_DIR TAG...
"""
import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).parent))
import build_v2 as b  # noqa: E402

work = Path(sys.argv[1])
asr = json.loads((work / "asr.json").read_text())
for tag in sys.argv[2:]:
    x, rate = b.load(work / "takes" / f"{tag}.wav")
    f0 = b.f0_track(x, rate)
    med = np.nanmedian(f0)
    print(f"{tag}: median {med:.0f} Hz  | {asr[tag]['text']}")
    row = []
    for i in range(0, len(f0), 5):
        chunk = f0[i:i + 5]
        voiced = chunk[~np.isnan(chunk)]
        row.append("   ." if len(voiced) == 0 else f"{12 * np.log2(np.median(voiced) / med):+4.0f}")
    words = {int(w["s"] / 0.05): w["w"] for w in asr[tag]["words"]}
    for start in range(0, len(row), 20):
        print("   ", " ".join(row[start:start + 20]))
        print("   ", " ".join(f"{words.get(k, '')[:4]:>4}" for k in range(start, min(start + 20, len(row)))))
