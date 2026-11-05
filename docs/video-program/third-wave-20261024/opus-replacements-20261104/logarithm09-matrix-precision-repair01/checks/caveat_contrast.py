"""The 09_1 caveat's contrast on a frame, against what it should read after the repair.

Needs PIL, so run without -I.

  python3 -B caveat_contrast.py report FRAMES_DIR OUT_PNG   old stage06 frames: measurements, controls and the sheet
  python3 -B caveat_contrast.py probe FRAME_PNG             one new frame, any width: exit 0 if the caveat reads in ink

The sheet is NOT A RENDER: the old frame with the caveat's pixels remapped from dim to ink, keeping each
pixel's coverage. It shows the hierarchy on the page, not what the engine will draw.
"""

import sys
from pathlib import Path

from PIL import Image, ImageDraw

INK, DIM = (0xEC, 0xEF, 0xF4), (0x8A, 0x93, 0xA3)
# at 1280 x 720; scaled by the frame's width
NOTE = (90, 636, 615, 660)
SIBLING = (760, 138, 1240, 162)  # "the matrix's columns ...", ink at 22 pt, the control
GROUND_AT = (20, 700)
# pass mark for `probe`: the brightest tenth of inked pixels, halfway between dim's and ink's nominal contrast
PASS_P90 = 11.0


def linear(c):
    c /= 255.0
    return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4


def lum(px):
    return 0.2126 * linear(px[0]) + 0.7152 * linear(px[1]) + 0.0722 * linear(px[2])


def ratio(a, b):
    la, lb = lum(a), lum(b)
    return (max(la, lb) + 0.05) / (min(la, lb) + 0.05)


def scaled(box, frame):
    k = frame.size[0] / 1280.0
    return tuple(int(round(v * k)) for v in box)


def pixels(frame, box):
    crop = frame.crop(box)
    data = crop.load()
    return [data[x, y] for y in range(crop.size[1]) for x in range(crop.size[0])]


def stats(frame, box, ground):
    px = sorted(pixels(frame, box), key=lum)
    inked = [p for p in px if ratio(p, ground) > 1.3]
    if not inked:
        return {"inked": 0, "peak": 1.0, "p90": 1.0, "median": 1.0}
    return {
        "inked": len(inked),
        "peak": ratio(px[-1], ground),
        "p90": ratio(inked[int(0.9 * (len(inked) - 1))], ground),
        "median": ratio(inked[len(inked) // 2], ground),
    }


def show(name, s):
    return f"{name}: inked {s['inked']} px; contrast on ground, peak {s['peak']:.2f}, brightest tenth {s['p90']:.2f}, median {s['median']:.2f}"


def remapped(frame, box, ground):
    """Each pixel's coverage of dim over the ground, redrawn as the same coverage of ink."""
    out = frame.copy()
    data = out.load()
    for y in range(box[1], box[3]):
        for x in range(box[0], box[2]):
            p = data[x, y]
            cover = sum((p[c] - ground[c]) / (DIM[c] - ground[c]) for c in range(3)) / 3.0
            cover = max(0.0, min(1.0, cover))
            data[x, y] = tuple(int(round(ground[c] + cover * (INK[c] - ground[c]))) for c in range(3))
    return out


def report(frames_dir, out_png):
    frames_dir = Path(frames_dir)
    frame = Image.open(frames_dir / "frame-0048.png").convert("RGB")
    ground = frame.getpixel(GROUND_AT)
    print(f"frame-0048.png, {frame.size[0]} x {frame.size[1]}, ground {ground}")
    print(f"nominal contrast: dim {ratio(DIM, ground):.2f}, ink {ratio(INK, ground):.2f}; WCAG asks 4.5 for small text, 7 at its higher level")
    old = stats(frame, NOTE, ground)
    sib = stats(frame, SIBLING, ground)
    print(show("the caveat as staged, dim at 20 pt", old))
    print(show("the line above the card, ink at 22 pt", sib))
    new_frame = remapped(frame, NOTE, ground)
    new = stats(new_frame, NOTE, ground)
    print(show("the caveat remapped to ink, same coverage, not a render", new))
    em = 20 * 0.01
    for name, per_unit in (("720p", 90.0), ("1080p", 135.0), ("4K", 270.0)):
        print(f"  20 pt at {name}: em {em * per_unit:.1f} px, x-height about {0.43 * em * per_unit:.1f} px (CM's 0.43 em, arithmetic)")
    failed = []
    if not old["p90"] < PASS_P90:
        failed.append("the old frame should fail the probe")
    if not new["p90"] >= PASS_P90:
        failed.append("the remapped frame should pass the probe")
    if not sib["p90"] >= PASS_P90:
        failed.append("the ink sibling should pass the probe")
    print(f"probe's pass mark {PASS_P90} on the brightest tenth: old {old['p90']:.2f} fails, remapped {new['p90']:.2f} passes, sibling {sib['p90']:.2f} passes" if not failed else "CONTROL FAILED: " + "; ".join(failed))

    # the sheet: the two lower halves at actual size, then the two lines enlarged
    crop_box = (60, 590, 660, 680)
    width = crop_box[2] - crop_box[0]
    zoom_box = (90, 632, 400, 664)
    z = 2
    zw, zh = (zoom_box[2] - zoom_box[0]) * z, (zoom_box[3] - zoom_box[1]) * z
    h = 2 * (crop_box[3] - crop_box[1]) + zh + 90
    sheet = Image.new("RGB", (max(width, zw) * 2 + 30, h), (0, 0, 0))
    draw = ImageDraw.Draw(sheet)
    for k, (title, pic) in enumerate((("as staged: dim", frame), ("remapped: ink, same coverage", new_frame))):
        x = k * (max(width, zw) + 30)
        draw.text((x + 4, 4), title + " (actual size, 720p)", fill=(255, 235, 90))
        sheet.paste(pic.crop(crop_box), (x, 20))
        draw.text((x + 4, 116), "enlarged 2x", fill=(255, 235, 90))
        sheet.paste(pic.crop(zoom_box).resize((zw, zh), Image.LANCZOS), (x, 132))
    draw.text((6, h - 18), "NOT A RENDER: an old preview frame on the scratch voice, recoloured by hand. Judge the hierarchy, not the pixels.", fill=(255, 150, 150))
    sheet.save(out_png)
    print("wrote the sheet:", sheet.size)
    return 1 if failed else 0


def probe(path):
    frame = Image.open(path).convert("RGB")
    ground = frame.getpixel(scaled(GROUND_AT + GROUND_AT, frame)[:2])
    s = stats(frame, scaled(NOTE, frame), ground)
    print(show(Path(path).name, s))
    good = s["p90"] >= PASS_P90
    print(("PASS" if good else "FAIL") + f": pass mark {PASS_P90}. Pick a frame after the caveat's fade, before 09_2's. A probe, not a reading of the picture.")
    return 0 if good else 1


if __name__ == "__main__":
    if sys.argv[1] == "report":
        sys.exit(report(sys.argv[2], sys.argv[3]))
    if sys.argv[1] == "probe":
        sys.exit(probe(sys.argv[2]))
    raise SystemExit(__doc__)
