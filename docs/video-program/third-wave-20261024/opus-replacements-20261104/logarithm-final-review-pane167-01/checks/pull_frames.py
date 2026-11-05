"""Read-only: pull named full-size frames from the logarithm master into a scratch folder. Run from the repo root.

A frame that is also a sealed verification picture is re-extracted at 960x540 with the sealed command, and its
hash is compared with verification.json, which proves this tool reads the same frame the sheet shows.
"""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

L = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
V = L / "logarithm-stage09-final-verify02"
MASTER = L / "logarithm-stage09-render-work02/logarithm_mysteries_master.mp4"
OUT = Path(sys.argv[1])
FRAMES = [int(a) for a in sys.argv[3:]]
TSV = Path(sys.argv[2])


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


v = json.loads((V / "verification.json").read_text())
fps = v["fps"]
sealed = {p["frame"]: p for p in v["pictures"]}
rows = []
for f in FRAMES:
    full = OUT / f"f{f:05d}.png"
    if not full.exists():
        subprocess.run(["ffmpeg", "-nostdin", "-n", "-v", "error", "-xerror", "-threads", "2", "-ss", f"{f / fps:.9f}", "-i", str(MASTER),
                        "-frames:v", "1", str(full)], check=True, timeout=180)
    match = "unsealed"
    if f in sealed:
        small = OUT / f"f{f:05d}-960.png"
        if not small.exists():
            subprocess.run(["ffmpeg", "-nostdin", "-n", "-v", "error", "-xerror", "-threads", "2", "-ss", f"{f / fps:.9f}", "-i", str(MASTER),
                            "-frames:v", "1", "-vf", "scale=960:540", str(small)], check=True, timeout=180)
        match = "same-as-sealed" if sha(small) == sealed[f]["sha256"] else "DIFFERS"
    row = f"{f}\t{f / fps:.3f}\t{sha(full)}\t{match}"
    rows.append(row)
    print(row, flush=True)
TSV.write_text("\n".join(rows) + "\n")
