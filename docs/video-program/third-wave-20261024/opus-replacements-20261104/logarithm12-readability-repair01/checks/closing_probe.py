"""Chapter 12's closing block on a frame: where its rows sit and how they read.

Needs PIL, so run without -I.

  python3 -B closing_probe.py report FRAMES_DIR OUT_PNG   old stage06 frames: measurements, room, controls, a sheet
  python3 -B closing_probe.py closing FRAME_PNG           one new frame, any width, with all the rows faded in: exit 0 if it shows the repair

The sheet's repaired pictures are NOT RENDERS: an old frame with its rows cut out, moved, recoloured by coverage and
scaled by hand. They show the layout on the page, not what the engine will draw.
"""

import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw

INK, DIM = (0xEC, 0xEF, 0xF4), (0x8A, 0x93, 0xA3)
UNIT = 90.0  # pixels per scene unit at 720p
GROUND_AT = (20, 700)
RAISE = 0.2  # how far the block moves up
LINE_Y = -1.4
LABEL_Y = LINE_Y - 0.32
CONVENTION_Y = -2.5
FOOTER_Y = -3.4

# Pass marks, in scene units unless named. All are my own.
PASS_P90 = 11.0  # brightest tenth of inked pixels, halfway between dim's and ink's nominal contrast
PASS_EDGE = 0.40  # below the footer: 5% of the frame's height
PASS_LABEL_H = (0.125, 0.165)  # a digit's height: 0.111 at 16 pt, 0.139 at 20 pt
PASS_CONVENTION_W = 4.0  # 3.744 at 18 pt, 4.16 at 20 pt
PASS_ROW_H = (0.17, 0.23)  # one row of 20 pt text; more means the box caught a neighbour
PASS_GAP = 0.20  # the family's row to the footer
PASS_CENTRE = 0.04


def linear(c):
    c /= 255.0
    return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4


def lum(px):
    return 0.2126 * linear(px[0]) + 0.7152 * linear(px[1]) + 0.0722 * linear(px[2])


def ratio(a, b):
    la, lb = lum(a), lum(b)
    return (max(la, lb) + 0.05) / (min(la, lb) + 0.05)


def stats(frame, box, ground):
    data = frame.load()
    xs, ys, inked = [], [], []
    for y in range(box[1], box[3]):
        for x in range(box[0], box[2]):
            if ratio(data[x, y], ground) > 1.3:
                xs.append(x)
                ys.append(y)
                inked.append(data[x, y])
    if not inked:
        return None
    inked.sort(key=lum)
    w, h = frame.size
    per = UNIT * w / 1280.0
    return {
        "box": (min(xs), min(ys), max(xs) + 1, max(ys) + 1),
        "left": (min(xs) - w / 2.0) / per,
        "right": (max(xs) + 1 - w / 2.0) / per,
        "top": (h / 2.0 - min(ys)) / per,
        "bottom": (h / 2.0 - max(ys) - 1) / per,
        "p90": ratio(inked[int(0.9 * (len(inked) - 1))], ground),
        "median": ratio(inked[len(inked) // 2], ground),
    }


def ground_of(frame):
    k = frame.size[0] / 1280.0
    return frame.getpixel((int(GROUND_AT[0] * k), int(GROUND_AT[1] * k)))


def to_box(frame, x0, y1, x1, y0):
    """Scene left, top, right, bottom to pixels."""
    w, h = frame.size
    u = UNIT * w / 1280.0
    return (int(w / 2 + x0 * u), int(h / 2 - y1 * u), int(w / 2 + x1 * u), int(h / 2 - y0 * u))


def show(name, s):
    return (
        f"{name}: x {s['left']:.3f}..{s['right']:.3f} (width {s['right'] - s['left']:.3f}), y {s['bottom']:.3f}..{s['top']:.3f} "
        f"(height {s['top'] - s['bottom']:.3f}); brightest tenth {s['p90']:.2f}, median {s['median']:.2f}"
    )


def closing_rows(frame):
    g = ground_of(frame)
    return {
        "labels": stats(frame, to_box(frame, 1.75, LINE_Y - 0.2, 6.05, LINE_Y - 0.46), g),
        "dots": stats(frame, to_box(frame, 1.42, LINE_Y + 0.1, 1.74, LINE_Y - 0.1), g),
        "convention": stats(frame, to_box(frame, 1.2, CONVENTION_Y + 0.2, 6.9, CONVENTION_Y - 0.2), g),
        "family": stats(frame, to_box(frame, 1.2, -2.7, 6.9, -3.1), g),
        "footer": stats(frame, to_box(frame, -7.11, -3.2, 7.11, -4.0), g),
    }


def closing_faults(rows):
    bad = []
    r = rows["labels"]
    if r is None:
        bad.append(f"labels: no ink under the line at y {LINE_Y}")
    else:
        h = r["top"] - r["bottom"]
        centre = (r["top"] + r["bottom"]) / 2
        if not PASS_LABEL_H[0] <= h <= PASS_LABEL_H[1]:
            bad.append(f"labels: height {h:.3f}, wanted {PASS_LABEL_H}")
        if abs(centre - LABEL_Y) > PASS_CENTRE:
            bad.append(f"labels: centre {centre:.3f}, wanted {LABEL_Y}")
        if r["p90"] < PASS_P90:
            bad.append(f"labels: brightest tenth {r['p90']:.2f}, wanted {PASS_P90}")
    r = rows["dots"]
    if r is None:
        bad.append(f"dots: no ink on the line's left end at y {LINE_Y}")
    elif r["p90"] < PASS_P90:
        bad.append(f"dots: brightest tenth {r['p90']:.2f}, wanted {PASS_P90}")
    r = rows["convention"]
    if r is None:
        bad.append(f"convention: no ink at y {CONVENTION_Y}")
    else:
        centre = (r["top"] + r["bottom"]) / 2
        h = r["top"] - r["bottom"]
        if not PASS_ROW_H[0] <= h <= PASS_ROW_H[1]:
            bad.append(f"convention: height {h:.3f}, wanted {PASS_ROW_H}")
        if abs(centre - CONVENTION_Y) > PASS_CENTRE:
            bad.append(f"convention: centre {centre:.3f}, wanted {CONVENTION_Y}")
        if r["right"] - r["left"] < PASS_CONVENTION_W:
            bad.append(f"convention: width {r['right'] - r['left']:.3f}, wanted {PASS_CONVENTION_W}")
        if r["p90"] < PASS_P90:
            bad.append(f"convention: brightest tenth {r['p90']:.2f}, wanted {PASS_P90}")
    r = rows["footer"]
    if r is None:
        bad.append("footer: no ink below y -3.2")
    else:
        edge = 4.0 + r["bottom"]
        if edge < PASS_EDGE:
            bad.append(f"footer: {edge:.3f} from the frame's edge, wanted {PASS_EDGE}")
        if r["p90"] < PASS_P90:
            bad.append(f"footer: brightest tenth {r['p90']:.2f}, wanted {PASS_P90}")
    family = rows["family"]
    if family is None:
        bad.append("family: no ink between y -2.7 and -3.1")
    else:
        h = family["top"] - family["bottom"]
        if not PASS_ROW_H[0] <= h <= PASS_ROW_H[1]:
            bad.append(f"family: height {h:.3f}, wanted {PASS_ROW_H}")
        if r is not None and family["bottom"] - r["top"] < PASS_GAP:
            bad.append(f"family to footer: {family['bottom'] - r['top']:.3f}, wanted {PASS_GAP}")
    return bad


def cover_of(frame, box, ground, colour):
    data = frame.load()
    out = Image.new("L", (box[2] - box[0], box[3] - box[1]), 0)
    px = out.load()
    for y in range(box[1], box[3]):
        for x in range(box[0], box[2]):
            p = data[x, y]
            c = sum((p[i] - ground[i]) / (colour[i] - ground[i]) for i in range(3)) / 3.0
            px[x - box[0], y - box[1]] = int(round(255 * max(0.0, min(1.0, c))))
    return out


def paste_cover(frame, cover, x0, y0, ground, colour):
    data = frame.load()
    cov = cover.load()
    for y in range(cover.size[1]):
        for x in range(cover.size[0]):
            a = cov[x, y] / 255.0
            if a > 0:
                data[x0 + x, y0 + y] = tuple(int(round(ground[i] + a * (colour[i] - ground[i]))) for i in range(3))


def mock(old, ground):
    """The whole staged block, on an old 720p frame, by hand. Not what the engine will draw."""
    u = UNIT
    new = old.copy()
    w, h = old.size
    draw = ImageDraw.Draw(new)
    fill = lambda x0, y1, x1, y0: draw.rectangle(to_box(old, x0, y1, x1, y0), fill=ground)  # noqa: E731
    fill(1.2, -0.95, 7.11, -3.25)
    fill(-7.11, -3.45, 7.11, -3.8)
    shift = int(round(RAISE * u))

    def carry(x0, y1, x1, y0):
        box = to_box(old, x0, y1, x1, y0)
        new.paste(old.crop(box), (box[0], box[1] - shift))

    carry(1.2, -0.95, 6.9, -1.25)  # how far
    carry(1.76, -1.42, 6.02, -1.78)  # the line, its marks, dots and rings
    carry(1.2, -2.15, 6.9, -2.45)  # the tie
    carry(1.2, -2.95, 6.9, -3.25)  # the family
    # the two runs of dots: moved and recoloured
    for x0, x1 in ((1.42, 1.74), (6.04, 6.4)):
        s = stats(old, to_box(old, x0, -1.5, x1, -1.7), ground)
        if s:
            paste_cover(new, cover_of(old, s["box"], ground, DIM), s["box"][0], s["box"][1] - shift, ground, INK)
    # the labels: scaled 20/16, recoloured, centred on the staged row
    for k in range(-3, 4):
        cx = 3.9 + 0.62 * k
        s = stats(old, to_box(old, cx - 0.3, -1.8, cx + 0.3, -2.05), ground)
        if not s:
            continue
        c = cover_of(old, s["box"], ground, DIM)
        c = c.resize((int(round(c.size[0] * 20 / 16)), int(round(c.size[1] * 20 / 16))), Image.LANCZOS)
        mid = (s["box"][0] + s["box"][2]) / 2
        paste_cover(new, c, int(round(mid - c.size[0] / 2)), int(round(h / 2 - LABEL_Y * u - c.size[1] / 2)), ground, INK)
    # the convention: scaled 20/18, recoloured, its left edge kept
    s = stats(old, to_box(old, 1.2, -2.55, 6.9, -2.86), ground)
    if s:
        c = cover_of(old, s["box"], ground, DIM)
        c = c.resize((int(round(c.size[0] * 20 / 18)), int(round(c.size[1] * 20 / 18))), Image.LANCZOS)
        paste_cover(new, c, s["box"][0], int(round(h / 2 - CONVENTION_Y * u - c.size[1] / 2)), ground, INK)
    # the footer: same size, recoloured, moved
    s = stats(old, to_box(old, -7.11, -3.4, 7.11, -3.95), ground)
    if s:
        c = cover_of(old, s["box"], ground, DIM)
        paste_cover(new, c, s["box"][0], int(round(h / 2 - FOOTER_Y * u - c.size[1] / 2)), ground, INK)
    return new


def report(frames_dir, out_png):
    d = Path(frames_dir)
    old = Image.open(d / "frame-0378.png").convert("RGB")
    ground = ground_of(old)
    print(f"frame 378, {old.size[0]} x {old.size[1]}, ground {ground}")
    print(f"nominal contrast: dim {ratio(DIM, ground):.2f}, ink {ratio(INK, ground):.2f}")
    failed = []

    print("\nas staged, frame 378")
    named = {
        "so row, gold 24 pt": (0.5, -0.03, 6.9, -0.42),
        "how far, ink 20 pt: the convention's and labels' new spec": (1.2, -0.93, 6.9, -1.27),
        "number-line labels, dim 16 pt": (1.3, -1.8, 6.9, -2.05),
        "the tie, gold 20 pt": (1.2, -2.15, 6.9, -2.48),
        "convention, dim 18 pt": (1.2, -2.55, 6.9, -2.86),
        "the family, gold 20 pt": (1.2, -2.95, 6.9, -3.28),
        "footer, dim 22 pt": (-7.11, -3.4, 7.11, -3.95),
        "left dots, dim 20 pt": (1.42, -1.5, 1.74, -1.7),
    }
    for name, (x0, y1, x1, y0) in named.items():
        print("  " + show(name, stats(old, to_box(old, x0, y1, x1, y0), ground)))
    rod = stats(old, to_box(old, -7.11, -2.0, 0.9, -3.2), ground)
    print(f"  lowest of the cube and rod: y {rod['bottom']:.3f}")
    for k in range(-3, 4):
        cx = 3.9 + 0.62 * k
        print("  " + show(f"label k = {k:+d}", stats(old, to_box(old, cx - 0.3, -1.8, cx + 0.3, -2.05), ground)))

    bad = closing_faults(closing_rows(old))
    print("\ncontrol: the probe on old frame 378")
    for reason in bad:
        print("  fails on: " + reason)
    if len(bad) < 5:
        failed.append("the probe should fail the old frame on every row")

    new = mock(old, ground)
    print("\nmock-up of frame 378, NOT A RENDER")
    rows = closing_rows(new)
    for name, r in rows.items():
        print("  " + (show(name, r) if r else f"{name}: no ink"))
    bad = closing_faults(rows)
    print("  the probe: " + ("; ".join(bad) if bad else "passes"))
    if bad:
        failed.append("the mock-up should pass the probe")

    print("\nthe room the repair uses, on every sampled frame from 145.0 s to the chapter's end")
    times = json.loads((d / "sampling.json").read_text(encoding="utf8"))["times"]
    picked = [i for i, t in enumerate(times) if t >= 145.0]
    print(f"  frames {picked[0]}..{picked[-1]}, {len(picked)} of them, {times[picked[0]]} s to {times[picked[-1]]} s")
    rooms = {
        "above the block, for how far: x 1.2..6.9, y -0.42..-0.93": (1.2, -0.42, 6.9, -0.93, None),
        "left of the block: x 0.6..1.3, y -0.42..-2.9": (0.6, -0.42, 1.3, -2.9, None),
        "right of the rows: x 6.4..7.11, y -1.9..-3.3": (6.4, -1.9, 7.11, -3.3, None),
        "under the cube, for the footer: x -7.11..0.9, y -2.95..-3.45": (-7.11, -2.95, 0.9, -3.45, None),
    }
    for name, (x0, y1, x1, y0, _) in rooms.items():
        hits = []
        for i in picked:
            f = Image.open(d / f"frame-{i:04d}.png").convert("RGB")
            if stats(f, to_box(f, x0, y1, x1, y0), ground_of(f)) is not None:
                hits.append(i)
        print(f"  {name}: ink on {len(hits)} frames {hits[:8]}")
        if hits:
            failed.append("room not empty: " + name)

    w, h = old.size
    strip = 26
    box = to_box(old, 0.4, -0.05, 7.11, -3.95)
    crop = lambda f: f.crop(box).resize(((box[2] - box[0]) * 2, (box[3] - box[1]) * 2), Image.LANCZOS)  # noqa: E731
    pictures = [
        ("As staged, root's stage06 frame 378", old, (255, 235, 90)),
        ("NOT A RENDER: frame 378 with the rows moved, recoloured and scaled by hand. Judge the layout, not the pixels.", new, (255, 150, 150)),
        ("As staged, the closing block, 2x", crop(old), (255, 235, 90)),
        ("NOT A RENDER: the closing block, 2x", crop(new), (255, 150, 150)),
    ]
    height = sum(p.size[1] + strip for _, p, _ in pictures)
    sheet = Image.new("RGB", (w, height), (0, 0, 0))
    draw = ImageDraw.Draw(sheet)
    y = 0
    for title, pic, colour in pictures:
        draw.text((6, y + 7), title, fill=colour)
        sheet.paste(pic, (0, y + strip))
        y += pic.size[1] + strip
    sheet.save(out_png)
    print("\nwrote the sheet:", sheet.size)
    if failed:
        print("CONTROL FAILED: " + "; ".join(failed))
    return 1 if failed else 0


def probe_closing(path):
    frame = Image.open(path).convert("RGB")
    rows = closing_rows(frame)
    for name, r in rows.items():
        if r:
            print(show(name, r))
    bad = closing_faults(rows)
    for reason in bad:
        print("FAIL:", reason)
    print(("FAIL" if bad else "PASS") + ". Pick a frame after the family's fade. A probe, not a reading of the picture.")
    return 1 if bad else 0


if __name__ == "__main__":
    if len(sys.argv) >= 4 and sys.argv[1] == "report":
        sys.exit(report(sys.argv[2], sys.argv[3]))
    if len(sys.argv) >= 3 and sys.argv[1] == "closing":
        sys.exit(probe_closing(sys.argv[2]))
    raise SystemExit(__doc__)
