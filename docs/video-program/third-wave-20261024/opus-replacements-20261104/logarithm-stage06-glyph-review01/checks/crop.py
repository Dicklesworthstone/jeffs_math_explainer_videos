"""Enlarge regions of probe pages. python3 -B crop.py OUT.png SCALE page:x0,y0,x1,y1 ...

Run from the repo root. Several regions are stacked top to bottom with a thin separator.
Crops are for looking only; the pages are not changed.
"""
import sys
from pathlib import Path

from PIL import Image

PAGES = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-glyph01/glyphs/pages")


def main():
    out, scale = Path(sys.argv[1]), int(sys.argv[2])
    parts = []
    for spec in sys.argv[3:]:
        page, box = spec.split(":")
        x0, y0, x1, y1 = (int(v) for v in box.split(","))
        im = Image.open(PAGES / f"frame_{int(page):06d}.png").convert("RGB").crop((x0, y0, x1, y1))
        parts.append(im.resize((im.width * scale, im.height * scale), Image.LANCZOS))
    width = max(p.width for p in parts)
    height = sum(p.height for p in parts) + 6 * (len(parts) - 1)
    sheet = Image.new("RGB", (width, height), (60, 60, 70))
    y = 0
    for p in parts:
        sheet.paste(p, (0, y))
        y += p.height + 6
    sheet.save(out)
    print(out.name, sheet.size)


if __name__ == "__main__":
    main()
