"""Do two neighbouring glyphs touch? Count ink components in a page region.

python3 -B touch_test.py page:x0,y0,x1,y1 [...]   (from the repo root; PIL needs site packages)

Ink level = how far a pixel sits between the page background and the brightest pixel in the
region, so 0.25 includes the antialiased edge and 0.75 keeps only solid ink. Two glyphs that
stay one component at 0.75 share solid ink; if they only join at 0.25 they share an edge blur
and read as touching at this size without overlapping.
"""
import sys
from pathlib import Path

from PIL import Image

PAGES = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-glyph01/glyphs/pages")
LEVELS = (0.25, 0.5, 0.75)


def components(im, level):
    w, h = im.size
    px = im.load()
    bg = min(px[x, y] for x in range(w) for y in range(h))
    top = max(px[x, y] for x in range(w) for y in range(h))
    cut = bg + level * (top - bg)
    seen = set()
    out = []
    for y in range(h):
        for x in range(w):
            if (x, y) in seen or px[x, y] < cut:
                continue
            stack = [(x, y)]
            seen.add((x, y))
            xs, ys = [], []
            while stack:
                a, b = stack.pop()
                xs.append(a)
                ys.append(b)
                for da in (-1, 0, 1):
                    for db in (-1, 0, 1):
                        c, d = a + da, b + db
                        if 0 <= c < w and 0 <= d < h and (c, d) not in seen and px[c, d] >= cut:
                            seen.add((c, d))
                            stack.append((c, d))
            out.append((min(xs), min(ys), max(xs), max(ys), len(xs)))
    return sorted(out)


def main():
    for spec in sys.argv[1:]:
        page, box = spec.split(":")
        x0, y0, x1, y1 = (int(v) for v in box.split(","))
        im = Image.open(PAGES / f"frame_{int(page):06d}.png").convert("L").crop((x0, y0, x1, y1))
        print(spec)
        for level in LEVELS:
            found = [c for c in components(im, level) if c[4] >= 3]
            text = " ".join(f"[x{x0 + c[0]}-{x0 + c[2]} y{y0 + c[1]}-{y0 + c[3]}]" for c in found)
            print(f"  level {level}: {len(found)} components {text}")


if __name__ == "__main__":
    main()
