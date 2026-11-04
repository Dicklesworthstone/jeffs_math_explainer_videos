"""04-A: fine horizontal stripes across the colour pictures, and the same measure on approved GF256 frames.

    python3 -I -B stripes.py      (from the repo root)

For a patch, averages each row's luma, removes a 9-row running mean, and reports the size of what is left and the
strongest period by a plain DFT. Reads native PNGs. For the approved film, ffmpeg extracts 3 single frames into a new
temporary folder, which is left in place and is not part of this review.
"""
import math
import pathlib
import subprocess
import tempfile

from PIL import Image

ROOT = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-opus-rebuild-frames01")
PATCHES = [
    ("04_reading_color/frame-0020.png", 150, 650, 300, 420, "04 frame 20, hue only"),
    ("04_reading_color/frame-0140.png", 130, 260, 200, 330, "04 frame 140, z squared"),
    ("04_reading_color/frame-0300.png", 130, 250, 110, 240, "04 frame 300, bump"),
    ("05_gallery/frame-0092.png", 270, 400, 300, 420, "05 frame 92, exp"),
    ("05_gallery/frame-0200.png", 60, 780, 180, 300, "05 frame 200, tan"),
    ("05_gallery/frame-0367.png", 60, 380, 180, 330, "05 frame 367, zeta"),
    ("05_gallery/frame-0464.png", 660, 1090, 170, 300, "05 frame 464, zoom"),
]
GF = pathlib.Path("explainer_corpus/raptorq_gf256/renders/release_v1")
APPROVED = [
    (GF / "raptorq_gf256_1080p_v1.mp4", 4, 110, 270, 140, 190, "GF256 1080p 4 s, hook picture, no crisp"),
    (GF / "raptorq_gf256_4k_v1.mp4", 4, 220, 540, 280, 380, "GF256 4K 4 s, hook picture, no crisp"),
    (GF / "raptorq_gf256_1080p_v1.mp4", 794, 1409, 1425, 100, 700, "GF256 1080p 794 s, table's key, crisp"),
]


def measure(path, x0, x1, y0, y1, name):
    im = Image.open(path).convert("RGB")
    rows = []
    for y in range(y0, y1):
        vals = [0.2126 * r + 0.7152 * g + 0.0722 * b for r, g, b in (im.getpixel((x, y)) for x in range(x0, x1))]
        rows.append(sum(vals) / len(vals))
    resid = [rows[i] - sum(rows[i - 4 : i + 5]) / 9 for i in range(4, len(rows) - 4)]
    mean = sum(rows) / len(rows)
    rms = math.sqrt(sum(r * r for r in resid) / len(resid))
    n = len(resid)
    best = max(
        (abs(sum(r * complex(math.cos(2 * math.pi * k * i / n), math.sin(2 * math.pi * k * i / n)) for i, r in enumerate(resid))), k)
        for k in range(2, n // 2)
    )
    print(f"{name}: {im.size[0]}x{im.size[1]}, patch mean luma {mean:.1f}, row residual rms {rms:.2f} ({100 * rms / mean:.1f}%), peak-to-peak {max(resid) - min(resid):.1f}, strongest period {n / best[1]:.2f} px")


print("CELL 0.035 units x 90 px per unit =", round(0.035 * 90, 3), "native px at 720p;", round(0.035 * 270, 3), "px at 4K by scale (not measured)")
for p, x0, x1, y0, y1, name in PATCHES:
    measure(ROOT / p, x0, x1, y0, y1, name)
tmp = pathlib.Path(tempfile.mkdtemp())
for k, (film, t, x0, x1, y0, y1, name) in enumerate(APPROVED):
    out = tmp / f"approved{k}.png"
    subprocess.run(["ffmpeg", "-v", "error", "-y", "-ss", str(t), "-i", str(film), "-frames:v", "1", str(out)], check=True)
    measure(out, x0, x1, y0, y1, name)
print("GF256 table's key: 4.6 units / 256 rows at 135 px per unit =", round(4.6 / 256 * 135, 2), "px a row")
