"""Exact scratch times of the frames this review cites, and the frame nearest a named time.

    python3 -I -B frame_times.py      (from the repo root)

Reads each chapter's sampling.json only. Frame n of a chapter is times[n].
"""
import json
import pathlib

ROOT = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-opus-rebuild-frames01")
CITED = {
    "02_turning_numbers": [24, 47, 55, 74, 79, 83, 84, 108, 122, 124, 135, 156, 177, 190, 204, 212, 218, 220, 223, 260],
    "03_two_planes": [0, 17, 18, 19, 44, 53, 54, 74, 73, 77, 78, 87, 100, 109, 115, 133, 140, 176, 194, 213, 215, 226, 228, 234, 253, 255, 263, 276, 294, 300, 312, 314],
    "04_reading_color": [12, 13, 20, 31, 32, 37, 62, 65, 66, 95, 135, 140, 185, 196, 197, 200, 219, 221, 223, 224, 231, 240, 257, 263, 268, 271, 272, 273, 277, 284, 300, 314, 315, 316],
    "05_gallery": [17, 30, 41, 69, 79, 80, 81, 82, 86, 92, 126, 131, 139, 165, 170, 183, 188, 200, 227, 244, 262, 271, 275, 279, 308, 320, 328, 333, 347, 366, 367, 368, 371, 411, 412, 444, 446, 448, 450, 464, 467, 490],
}
NEAR = {"04_reading_color": [56.8, 68.0, 79.4, 111.8], "05_gallery": [23.2, 58.6, 62.6, 86.6, 75.0]}
for ch, frames in CITED.items():
    s = json.loads((ROOT / ch / "sampling.json").read_text())
    t = s["times"]
    print(f"{ch}: {len(t)} frames, duration {s['duration']}")
    print("  " + "  ".join(f"{n}:{t[n]:.3f}" for n in sorted(set(frames))))
    for want in NEAR.get(ch, []):
        n = min(range(len(t)), key=lambda i: abs(t[i] - want))
        print(f"  nearest {want}: frame {n} at {t[n]:.3f}")
