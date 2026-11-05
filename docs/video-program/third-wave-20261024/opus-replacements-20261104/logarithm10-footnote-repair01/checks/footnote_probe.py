"""The chapter 10 closing footnotes on a frame: where they sit and how they read.

Needs PIL, so run without -I.

  python3 -B footnote_probe.py report FRAME_PNG OUT_PNG   an old stage06 frame: measurements, controls and a sheet
  python3 -B footnote_probe.py probe FRAME_PNG            one new frame, any width: exit 0 if the repair shows

The sheet's lower picture is NOT A RENDER: the old frame with the two rows cut out, recoloured from dim to ink
keeping each pixel's coverage, the upper row scaled by 22/18, and both pasted at their new centres. It shows the
layout on the page, not what the engine will draw.
"""

import sys
from pathlib import Path

from PIL import Image, ImageDraw

INK, DIM = (0xEC, 0xEF, 0xF4), (0x8A, 0x93, 0xA3)
GROUND_AT = (20, 700)  # at 1280 x 720; scaled by the frame's width
GRID_BOTTOM_ROW = 591  # the grid's lowest inked row on the old frames, at 720p
SIBLING = (720, 322, 1260, 352)  # "A sends no direction ...", ink at 22 pt, the control
UNIT = 90.0  # pixels per scene unit at 720p

# Pass marks for `probe`, in scene units unless named.
PASS_P90 = 11.0  # brightest tenth of inked pixels, halfway between dim's and ink's nominal contrast
PASS_EDGE = 0.40  # below the lower row: 5% of the frame's height
PASS_GAP = 0.15  # between the grid and the upper row, and between the rows
PASS_UPPER_WIDTH = 6.4  # the upper row was 5.44 wide at 18 pt and should be about 6.65 at 22 pt
PASS_LOWER_WIDTH = 12.4  # the lower row is 12.67 wide at 22 pt and should stay so
PASS_SIDE = 6.86  # the crate's own limit for right-hand text


def linear(c):
    c /= 255.0
    return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4


def lum(px):
    return 0.2126 * linear(px[0]) + 0.7152 * linear(px[1]) + 0.0722 * linear(px[2])


def ratio(a, b):
    la, lb = lum(a), lum(b)
    return (max(la, lb) + 0.05) / (min(la, lb) + 0.05)


def box_stats(frame, box, ground):
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
    return {
        "x0": min(xs),
        "x1": max(xs),
        "y0": min(ys),
        "y1": max(ys),
        "inked": len(inked),
        "peak": ratio(inked[-1], ground),
        "p90": ratio(inked[int(0.9 * (len(inked) - 1))], ground),
        "median": ratio(inked[len(inked) // 2], ground),
    }


def rows_below_grid(frame, ground):
    """Clusters of inked rows under the grid. Text rows have no blank pixel row inside them."""
    w, h = frame.size
    k = w / 1280.0
    data = frame.load()
    start = int(round((GRID_BOTTOM_ROW + 3) * k))
    # coarse first: a row counts if any of its pixels is inked
    marked = []
    for y in range(start, h):
        marked.append(any(ratio(data[x, y], ground) > 1.3 for x in range(0, w, 1)))
    clusters, begin = [], None
    for i, on in enumerate(marked + [False]):
        if on and begin is None:
            begin = i
        elif not on and begin is not None:
            clusters.append((start + begin, start + i))
            begin = None
    return clusters


def measure(frame):
    w, h = frame.size
    k = w / 1280.0
    ground = frame.getpixel((int(GROUND_AT[0] * k), int(GROUND_AT[1] * k)))
    per_unit = UNIT * k
    out = []
    for y0, y1 in rows_below_grid(frame, ground):
        s = box_stats(frame, (0, y0, w, y1), ground)
        s["left"] = (s["x0"] - w / 2.0) / per_unit
        s["right"] = (s["x1"] + 1 - w / 2.0) / per_unit
        s["top"] = (h / 2.0 - s["y0"]) / per_unit
        s["bottom"] = (h / 2.0 - s["y1"] - 1) / per_unit
        out.append(s)
    return out, ground, per_unit


def show(name, s):
    return (
        f"{name}: scene x {s['left']:.3f}..{s['right']:.3f} (width {s['right'] - s['left']:.3f}), "
        f"y {s['bottom']:.3f}..{s['top']:.3f}; contrast on ground, peak {s['peak']:.2f}, "
        f"brightest tenth {s['p90']:.2f}, median {s['median']:.2f}"
    )


def verdict(rows, per_unit, frame_h):
    """Reasons the frame does not show the repair; empty if it does."""
    if len(rows) != 2:
        return [f"expected two rows under the grid, found {len(rows)}"]
    upper, lower = rows
    bad = []
    edge = 4.0 + lower["bottom"]
    grid_bottom = (frame_h / 2.0 - (GRID_BOTTOM_ROW + 1) * per_unit / UNIT) / per_unit
    to_grid = grid_bottom - upper["top"]
    between = upper["bottom"] - lower["top"]
    if edge < PASS_EDGE:
        bad.append(f"lower row {edge:.3f} from the frame's edge, wanted {PASS_EDGE}")
    if to_grid < PASS_GAP:
        bad.append(f"upper row {to_grid:.3f} from the grid, wanted {PASS_GAP}")
    if between < PASS_GAP:
        bad.append(f"rows {between:.3f} apart, wanted {PASS_GAP}")
    for name, s, width in (("upper", upper, PASS_UPPER_WIDTH), ("lower", lower, PASS_LOWER_WIDTH)):
        if s["p90"] < PASS_P90:
            bad.append(f"{name} row's brightest tenth {s['p90']:.2f}, wanted {PASS_P90}")
        if s["right"] - s["left"] < width:
            bad.append(f"{name} row {s['right'] - s['left']:.3f} wide, wanted {width}")
        if max(abs(s["left"]), abs(s["right"])) > PASS_SIDE:
            bad.append(f"{name} row reaches x = {max(abs(s['left']), abs(s['right'])):.3f}, limit {PASS_SIDE}")
    return bad


def cover_of(frame, box, ground):
    """A dim row's coverage of each pixel, 0 to 255, as an 8-bit picture."""
    data = frame.load()
    out = Image.new("L", (box[2] - box[0], box[3] - box[1]), 0)
    px = out.load()
    for y in range(box[1], box[3]):
        for x in range(box[0], box[2]):
            p = data[x, y]
            c = sum((p[i] - ground[i]) / (DIM[i] - ground[i]) for i in range(3)) / 3.0
            px[x - box[0], y - box[1]] = int(round(255 * max(0.0, min(1.0, c))))
    return out


def paste_ink(frame, cover, centre_y, ground):
    w = frame.size[0]
    x0 = int(round(w / 2.0 - cover.size[0] / 2.0))
    y0 = int(round(centre_y - cover.size[1] / 2.0))
    data = frame.load()
    cov = cover.load()
    for y in range(cover.size[1]):
        for x in range(cover.size[0]):
            a = cov[x, y] / 255.0
            data[x0 + x, y0 + y] = tuple(int(round(ground[i] + a * (INK[i] - ground[i]))) for i in range(3))


def report(path, out_png):
    frame = Image.open(path).convert("RGB")
    rows, ground, per_unit = measure(frame)
    w, h = frame.size
    print(f"{Path(path).name}, {w} x {h}, ground {ground}")
    print(f"nominal contrast: dim {ratio(DIM, ground):.2f}, ink {ratio(INK, ground):.2f}; WCAG asks 4.5 for small text, 7 at its higher level")
    if len(rows) != 2:
        print(f"CONTROL FAILED: {len(rows)} rows")
        return 1
    print(show("textbook row as staged, dim at 18 pt", rows[0]))
    print(show("source row as staged, dim at 22 pt", rows[1]))
    sib = box_stats(frame, SIBLING, ground)
    print(f"sibling, ink at 22 pt: contrast on ground, peak {sib['peak']:.2f}, brightest tenth {sib['p90']:.2f}, median {sib['median']:.2f}")
    print(f"lower row to the frame's edge: {4.0 + rows[1]['bottom']:.3f} units, {(4.0 + rows[1]['bottom']) * per_unit:.0f} px at this size")
    print(f"upper row to the grid: {(h / 2.0 - GRID_BOTTOM_ROW - 1) / per_unit - rows[0]['top']:.3f} units")

    # the mock-up
    mock = frame.copy()
    draw = ImageDraw.Draw(mock)
    upper_box = (rows[0]["x0"], rows[0]["y0"], rows[0]["x1"] + 1, rows[0]["y1"] + 1)
    lower_box = (rows[1]["x0"], rows[1]["y0"], rows[1]["x1"] + 1, rows[1]["y1"] + 1)
    upper_cover = cover_of(frame, upper_box, ground)
    lower_cover = cover_of(frame, lower_box, ground)
    draw.rectangle((0, GRID_BOTTOM_ROW + 3, w, h), fill=ground)
    scale = 22.0 / 18.0
    upper_cover = upper_cover.resize((int(round(upper_cover.size[0] * scale)), int(round(upper_cover.size[1] * scale))), Image.LANCZOS)
    paste_ink(mock, upper_cover, h / 2.0 + 2.93 * per_unit, ground)
    paste_ink(mock, lower_cover, h / 2.0 + 3.4 * per_unit, ground)

    new_rows, _, _ = measure(mock)
    if len(new_rows) == 2:
        print(show("textbook row, mock-up, not a render", new_rows[0]))
        print(show("source row, mock-up, not a render", new_rows[1]))
        print(f"lower row to the frame's edge in the mock-up: {4.0 + new_rows[1]['bottom']:.3f} units, {(4.0 + new_rows[1]['bottom']) * per_unit:.0f} px at this size")

    old_bad = verdict(rows, per_unit, h)
    new_bad = verdict(new_rows, per_unit, h)
    print("controls: the old frame should fail the probe, the mock-up should pass it")
    print(f"  old frame fails on: {'; '.join(old_bad) if old_bad else 'NOTHING, CONTROL FAILED'}")
    print(f"  mock-up: {'; '.join(new_bad) if new_bad else 'passes'}")

    strip = 26
    sheet = Image.new("RGB", (w, 2 * (h + strip)), (0, 0, 0))
    d = ImageDraw.Draw(sheet)
    d.text((6, 7), "As staged, root's stage06 frame: dim, 18 and 22 pt", fill=(255, 235, 90))
    sheet.paste(frame, (0, strip))
    d.text((6, h + strip + 7), "NOT A RENDER: the same frame with the two rows recoloured, scaled and moved by hand. Judge the layout, not the pixels.", fill=(255, 150, 150))
    sheet.paste(mock, (0, h + 2 * strip))
    sheet.save(out_png)
    print("wrote the sheet:", sheet.size)
    return 0 if old_bad and not new_bad else 1


def probe(path):
    frame = Image.open(path).convert("RGB")
    rows, _, per_unit = measure(frame)
    for name, s in zip(("upper", "lower", "extra"), rows):
        print(show(f"{name} row", s))
    bad = verdict(rows, per_unit, frame.size[1])
    for reason in bad:
        print("FAIL:", reason)
    print(("FAIL" if bad else "PASS") + ". Pick a frame after the rows' fade, before the chapter ends. A probe, not a reading of the picture.")
    return 1 if bad else 0


if __name__ == "__main__":
    if len(sys.argv) >= 4 and sys.argv[1] == "report":
        sys.exit(report(sys.argv[2], sys.argv[3]))
    if len(sys.argv) >= 3 and sys.argv[1] == "probe":
        sys.exit(probe(sys.argv[2]))
    raise SystemExit(__doc__)
