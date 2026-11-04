"""Print the rows a page should show, to read beside the page. python3 -I -B show_rows.py FIRST LAST"""
import csv
import sys
from pathlib import Path

here = Path(sys.argv[0]).resolve().parent
csv.field_size_limit(10**7)
first, last = int(sys.argv[1]), int(sys.argv[2])
with (here / "page_rows.tsv").open(encoding="utf-8", newline="") as f:
    r = csv.reader(f, delimiter="\t", quoting=csv.QUOTE_NONE, escapechar="\\")
    next(r)
    for page, top, bottom, left, right, row, kind, chapters, text in r:
        n = int(page[6:12])
        if first <= n <= last:
            chapter = chapters.replace(",supplement", "+").replace("supplement", "sup")
            print(f"p{n:02d} {top:>4}-{bottom:<4} R{right:>4} #{row:>3} {kind[:3]} {chapter[:22]:22s} {text}")
