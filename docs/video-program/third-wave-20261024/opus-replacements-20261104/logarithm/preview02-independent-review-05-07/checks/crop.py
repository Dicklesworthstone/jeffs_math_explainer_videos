"""Scratch only: native-resolution crops, upscaled by whole factors with nearest neighbour, on one labelled sheet.
Args: OUT.png then chapter:frame:x0,y0,x1,y1:scale items."""
import pathlib
import sys
from PIL import Image, ImageDraw

ROOT = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-opus-rebuild-frames02")
CH = {"05": "05_complex_turns", "06": "06_lifting_loops", "07": "07_branch_cut"}
out, items = sys.argv[1], sys.argv[2:]
panels = []
for it in items:
    ch, frame, box, scale = it.split(":")
    b = tuple(int(x) for x in box.split(","))
    im = Image.open(ROOT / CH[ch] / f"frame-{int(frame):04d}.png").convert("RGB").crop(b)
    s = int(scale)
    panels.append((f"{ch} frame {frame} box {box} x{s}", im.resize((im.width * s, im.height * s), Image.NEAREST)))
W = max(p.width for _, p in panels) + 20
H = sum(p.height + 28 for _, p in panels) + 10
sheet = Image.new("RGB", (W, H), (40, 40, 40))
d = ImageDraw.Draw(sheet)
y = 6
for label, p in panels:
    d.text((10, y), label, fill=(255, 255, 255))
    sheet.paste(p, (10, y + 16))
    y += p.height + 28
sheet.save(out)
print(out, sheet.size)
