#!/usr/bin/env python3
"""Final gate on the assembled narration: every edited line WAV, transcribed,
against its script line (same normalization and name rules as the chooser).

    python3 qa_lines.py SCRIPT.tsv ASR.json
"""
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import build_v2 as b  # noqa: E402

script = dict(b.read_script(sys.argv[1]))
asr = json.loads(Path(sys.argv[2]).read_text())
worst = []
for line_id, text in script.items():
    want = [t for t, _ in b.tokens(text)]
    heard = [t for t, _ in b.tokens(asr[line_id]["text"])]
    ratio, _, hit = b.align(want, heard)
    names = sum(1 for i, w in enumerate(want) if w.rstrip("s") in b.NAMES and i not in hit)
    worst.append((ratio, names, line_id, asr[line_id]["text"]))
worst.sort()
bad = [w for w in worst if w[0] < 0.9 or w[1]]
print(f"{len(script)} lines; mean match {sum(w[0] for w in worst) / len(worst):.3f}; "
      f"{len(bad)} below 0.9 or with a misheard name")
for ratio, names, line_id, heard in worst[:8]:
    print(f"  {line_id:4} {ratio:.3f} names_missed={names} | {heard}")
