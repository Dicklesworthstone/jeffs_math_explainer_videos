"""Tie every probe page to the registry rows it should show, from ink alone.

Run from the repo root with python3 -B (PIL needs site packages). Reads only; writes the
named output paths. The probe sets rows in registry order, 0.22 units apart (29.7 px at
1920x1080), and repeats a heading on every page. So rows on a page are ink bands split by
blank gaps, and the counts must add up to the rows the probe's own chunking predicts.

This cannot say that a row's ink is the right glyphs. That part is done by eye.
"""
import csv
import hashlib
import re
import sys
from pathlib import Path

from PIL import Image

RUN = Path(
    "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-glyph01"
)
PAGES = RUN / "glyphs/pages"
GAP = 26  # px of blank that separates rows: gaps inside a row reach 24, gaps between rows start at 29.
HEADING_BOTTOM = 95  # px: the heading sits above this on every page.
BACKGROUND_DELTA = 40


def decode(literal):
    """Undo Rust's {:?} on a string."""
    body = literal[1:-1]
    out, i = [], 0
    while i < len(body):
        c = body[i]
        if c != "\\":
            out.append(c)
            i += 1
            continue
        n = body[i + 1]
        if n == "u":
            end = body.index("}", i)
            out.append(chr(int(body[i + 3 : end], 16)))
            i = end + 1
        else:
            out.append({"n": "\n", "t": "\t", "r": "\r", "0": "\0"}.get(n, n))
            i += 2
    return "".join(out)


def registry():
    rows = []
    for number, line in enumerate((RUN / "glyphs/registry.tsv").read_text(encoding="utf-8").splitlines(), 1):
        kind, chapters, literal = line.split("\t", 2)
        rows.append((number, kind, chapters, decode(literal)))
    return rows


def probe_rows(rows):
    """What the probe lays out: TeX whole; other text split at newlines and every 64 chars."""
    out = []
    for number, kind, chapters, source in rows:
        if kind == "Tex":
            out.append((number, kind, chapters, source, source))
            continue
        for line in source.split("\n"):
            if not line:
                out.append((number, kind, chapters, source, None))  # a blank step, no ink
                continue
            chars = list(line)
            for start in range(0, len(chars), 64):
                out.append((number, kind, chapters, source, "".join(chars[start : start + 64])))
    return out


def bands(path):
    im = Image.open(path).convert("L")
    w, h = im.size
    data = im.load()
    bg = data[w - 2, h - 2]
    ink_rows = []
    extents = {}
    for y in range(h):
        xs = [x for x in range(w) if abs(data[x, y] - bg) > BACKGROUND_DELTA]
        if xs:
            ink_rows.append(y)
            extents[y] = (xs[0], xs[-1])
    groups = []
    for y in ink_rows:
        if groups and y - groups[-1][-1] <= GAP:
            groups[-1].append(y)
        else:
            groups.append([y])
    result = []
    for g in groups:
        left = min(extents[y][0] for y in g)
        right = max(extents[y][1] for y in g)
        result.append({"top": g[0], "bottom": g[-1], "left": left, "right": right})
    return result, (w, h)


def main():
    out_map, out_rows = Path(sys.argv[1]), Path(sys.argv[2])
    rows = registry()
    expected = [r for r in probe_rows(rows) if r[4] is not None]
    print(f"registry sources {len(rows)}; ink rows the probe should lay out {len(expected)}")
    files = sorted(PAGES.glob("frame_*.png"))
    print(f"pages {len(files)}: {files[0].name} to {files[-1].name}")
    at = 0
    page_rows = []
    with out_map.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, delimiter="\t", lineterminator="\n")
        w.writerow(["page", "sha256", "bands", "first_registry_row", "last_registry_row", "first_ink_top", "last_ink_bottom", "max_right_px"])
        for path in files:
            found, size = bands(path)
            heading = [b for b in found if b["bottom"] < HEADING_BOTTOM]
            body = [b for b in found if b["bottom"] >= HEADING_BOTTOM]
            if len(heading) != 1:
                print(f"  {path.name}: {len(heading)} heading bands")
            first = at
            for b in body:
                if at < len(expected):
                    number, kind, chapters, source, text = expected[at]
                else:
                    number, kind, chapters, source, text = (0, "?", "?", "?", "?")
                page_rows.append((path.name, b, number, kind, chapters, text))
                at += 1
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            last_n = expected[at - 1][0] if 0 < at <= len(expected) else "?"
            first_n = expected[first][0] if first < len(expected) else "?"
            w.writerow([
                path.name, digest, len(body), first_n, last_n,
                body[0]["top"] if body else "", body[-1]["bottom"] if body else "",
                max(b["right"] for b in body) if body else "",
            ])
    print(f"bands found {at}; expected {len(expected)}; {'MATCH' if at == len(expected) else 'MISMATCH'}")
    with out_rows.open("w", newline="", encoding="utf-8") as f:
        w = csv.writer(f, delimiter="\t", lineterminator="\n", quoting=csv.QUOTE_NONE, quotechar=None, escapechar="\\")
        w.writerow(["page", "top", "bottom", "left", "right", "registry_row", "kind", "chapters", "text"])
        for name, b, number, kind, chapters, text in page_rows:
            w.writerow([name, b["top"], b["bottom"], b["left"], b["right"], number, kind, chapters, text])
    wide = [(n, b["right"]) for n, b, *_ in page_rows if b["right"] >= 1920 - 60]
    low = [(n, b["bottom"]) for n, b, *_ in page_rows if b["bottom"] >= 1080 - 40]
    left = [(n, b["left"]) for n, b, *_ in page_rows if b["left"] < 40]
    print(f"rows reaching within 60 px of the right edge: {wide}")
    print(f"rows reaching within 40 px of the bottom edge: {low}")
    print(f"rows starting within 40 px of the left edge: {left}")


if __name__ == "__main__":
    main()
