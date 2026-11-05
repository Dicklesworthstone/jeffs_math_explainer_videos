"""Every frame of the readout and its notes around t = 1 and t = 3, from root's stage06 preview.

Needs PIL and ffmpeg, so run without -I, from the repo root:
  python3 -B dense_readout.py OUT_DIR

Writes a contact sheet per window into OUT_DIR and its crops into OUT_DIR's work folder. The preview is 30 fps on the
scratch voice. Each label is the seek time plus the frame's index over 30.
"""

import subprocess
import sys
from pathlib import Path

from PIL import Image, ImageDraw

MP4 = "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-preview01/render/11_unreachable.mp4"
OUT = Path(sys.argv[1])
OUT.mkdir(parents=True, exist_ok=True)
CROP = "240:190:640:125"  # w:h:x:y at 720p: the readout, the top of the gauge and the notes
FPS = 30

for name, start, count in (("readout-around-t1", 35.2, 30), ("readout-around-t3", 37.6, 30)):
    work = OUT / "work" / name
    work.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["ffmpeg", "-v", "error", "-y", "-ss", str(start), "-i", MP4, "-frames:v", str(count), "-vf", f"crop={CROP}", "-start_number", "0", str(work / "c-%03d.png")],
        check=True,
    )
    cols = 6
    w, h = 240, 190 + 18
    files = sorted(work.glob("c-*.png"))
    sheet = Image.new("RGB", (cols * w, ((len(files) + cols - 1) // cols) * h), (0, 0, 0))
    draw = ImageDraw.Draw(sheet)
    for i, f in enumerate(files):
        x, y = (i % cols) * w, (i // cols) * h
        draw.text((x + 3, y + 3), f"{start} + {i}/{FPS} = {start + i / FPS:.3f} s", fill=(255, 235, 90))
        sheet.paste(Image.open(f).convert("RGB"), (x, y + 18))
    sheet.save(OUT / f"{name}.png")
    print(name, len(files), sheet.size)
