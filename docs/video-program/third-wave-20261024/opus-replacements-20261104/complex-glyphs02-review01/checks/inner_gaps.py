"""List blank vertical gaps inside a row's ink band, largest first.

python3 -B inner_gaps.py MIN_PX   (from the repo root; PIL needs site packages; reads page_rows.tsv beside it)

A symbol dropped from the middle of a stacked expression can leave a hole, so the rows with the
widest holes get read again. A gap is normal in a fraction or a limit; this only picks what to look at.
"""
import csv
import sys
from pathlib import Path

from PIL import Image

here = Path(sys.argv[0]).resolve().parent
PAGES = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-glyphs02/glyphs/pages")
MIN_PX = int(sys.argv[1])
csv.field_size_limit(10**7)
found = []
with (here / "page_rows.tsv").open(encoding="utf-8", newline="") as f:
    r = csv.reader(f, delimiter="\t", quoting=csv.QUOTE_NONE, escapechar="\\")
    next(r)
    cache = {}
    for page, top, bottom, left, right, row, kind, chapters, text in r:
        if page not in cache:
            cache[page] = Image.open(PAGES / page).convert("L")
        im = cache[page]
        px = im.load()
        w, h = im.size
        bg = px[w - 2, h - 2]
        top, bottom = int(top), int(bottom)
        run = 0
        for y in range(top, bottom + 1):
            if any(abs(px[x, y] - bg) > 40 for x in range(w)):
                if run >= MIN_PX:
                    found.append((run, page[6:12], int(row), y - run, y, text[:70]))
                run = 0
            else:
                run += 1
for run, page, row, y0, y1, text in sorted(found, reverse=True):
    print(f"{run:3d} px  page {int(page):2d}  row {row:3d}  y{y0}-{y1}  {text}")
print(f"{len(found)} gaps of at least {MIN_PX} px")
