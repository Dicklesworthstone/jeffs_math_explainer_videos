"""Cut the sheets that the viewer received as JPEG into parts small enough to arrive as PNG, panels unchanged and in order.

    python3 -B .../checks/split_lossless.py     (from the repo root)

The harness turned the seven largest sheets into JPEG on delivery (checks/viewing_proof.out). The panel pixels here are copied from
those sheets. A part is kept under LIMIT bytes (400,794 is the largest seen to arrive as PNG here) and under WIDEST pixels.
"""
import pathlib

from PIL import Image

BUNDLE = pathlib.Path(__file__).resolve().parent.parent
EVIDENCE = BUNDLE / "evidence"
OUT = EVIDENCE / "lossless"
GAP, LABEL, LIMIT, WIDEST = 8, 22, 390_000, 1900
# sheet: (panels across, panels down, panel width, panel height)
SHEETS = {
    "stairs-z2-whole": (3, 2, 577, 577),
    "stairs-sinrecip-whole": (3, 2, 504, 504),
    "stairs-sin-whole": (3, 2, 540, 474),
    "stairs-bump-whole": (3, 2, 450, 451),
    "stairs-openbump-whole": (3, 2, 504, 505),
    "stairs-tan-whole": (2, 3, 756, 378),
    "repaint-z2": (3, 3, 577, 577),
}


def main():
    OUT.mkdir(exist_ok=False)
    for name, (across, down, w, h) in SHEETS.items():
        sheet = Image.open(EVIDENCE / f"{name}.png").convert("RGB")
        if sheet.size != (across * (w + GAP) - GAP, down * (h + LABEL + GAP)):
            raise SystemExit(f"{name}: sheet is {sheet.size}, geometry says otherwise")
        panels = [
            sheet.crop((c * (w + GAP), r * (h + LABEL + GAP), c * (w + GAP) + w, r * (h + LABEL + GAP) + h + LABEL))
            for r in range(down)
            for c in range(across)
        ]

        def write(group, path):
            part = Image.new("RGB", (len(group) * (w + GAP) - GAP, h + LABEL), (20, 20, 20))
            for k, p in enumerate(group):
                part.paste(p, (k * (w + GAP), 0))
            part.save(path, optimize=True)
            return path.stat().st_size

        k, index = 0, 1
        while k < len(panels):
            path = OUT / f"{name}-{index}.png"
            # past about 2000 px the viewer shrinks an image, so a part stays narrower
            count = min(3, len(panels) - k, (WIDEST + GAP) // (w + GAP))
            while True:
                size = write(panels[k : k + count], path)
                if size <= LIMIT or count == 1:
                    break
                count -= 1
            print(f"{path.name}: {count} panels, {size} bytes")
            k += count
            index += 1


if __name__ == "__main__":
    main()
