"""Find what the faint speck near the centre of frames 546-549 is. Read-only. Needs PIL (python3 -B)."""
import pathlib

from PIL import Image, ImageChops

D = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-combined-repair-frames01/13_arithmetic")
REF = Image.open(D / "frame-0552.png").convert("RGB")
BOX = (600, 340, 680, 420)
for i in list(range(540, 553)):
    f = Image.open(D / f"frame-{i:04}.png").convert("RGB")
    d = ImageChops.difference(f.crop(BOX), REF.crop(BOX))
    bb = d.getbbox()
    peak = max(max(p) for p in d.getdata())
    print(i, "diff bbox in window", bb, "peak", peak)
for i in (547, 548, 549):
    f = Image.open(D / f"frame-{i:04}.png").convert("RGB")
    f.crop(BOX).resize((480, 480), Image.NEAREST).save(pathlib.Path(__file__).parent / f"speck-{i}.png")
