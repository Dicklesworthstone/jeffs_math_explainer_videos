"""Local, read-only on inputs: write a downscaled whole view and a full-resolution crop of each given 3840x2160 JPEG
into OUTDIR (created exclusively). Crops are lossless PNG so no further JPEG loss is added.
usage: python3 -I -B crop-frames01.py OUTDIR X0 Y0 X1 Y1 JPEG [JPEG ...]"""
import sys
from pathlib import Path

from PIL import Image

out = Path(sys.argv[1])
box = tuple(int(v) for v in sys.argv[2:6])
out.mkdir()
for src in map(Path, sys.argv[6:]):
    with Image.open(src) as im:
        assert im.size == (3840, 2160), (src, im.size)
        im.resize((1920, 1080), Image.Resampling.LANCZOS).save(out / f'{src.stem}-whole1920.png')
        im.crop(box).save(out / f'{src.stem}-crop-{box[0]}-{box[1]}-{box[2]}-{box[3]}.png')
        print(src.name, 'whole and crop', box, 'written')
