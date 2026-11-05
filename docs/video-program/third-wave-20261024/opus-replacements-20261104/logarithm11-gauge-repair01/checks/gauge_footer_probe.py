"""Chapter 11's gauge notes and footers on a frame: where they sit and how they read.

Needs PIL, so run without -I.

  python3 -B gauge_footer_probe.py report FRAMES_DIR OUT_PNG   old stage06 frames: measurements, controls, a sheet
  python3 -B gauge_footer_probe.py footer FRAME_PNG            one new frame, any width: exit 0 if the footer shows the repair
  python3 -B gauge_footer_probe.py notes FRAME_PNG             one new frame, any width: exit 0 if both gauge notes do

The sheet's repaired pictures are NOT RENDERS: an old frame with its rows cut out, recoloured by coverage, and pasted
at their new places; the notes also scaled by 22/18 with a real approximately sign borrowed from frame 598. They show
the layout on the page, not what the engine will draw.
"""

import sys
from pathlib import Path

from PIL import Image, ImageDraw

INK, DIM, BLUE = (0xEC, 0xEF, 0xF4), (0x8A, 0x93, 0xA3), (0x58, 0xC4, 0xDD)
UNIT = 90.0  # pixels per scene unit at 720p
GROUND_AT = (20, 700)

# Pass marks, in scene units unless named. All are my own.
PASS_FOOTER_P90 = 11.0  # brightest tenth of inked pixels, halfway between dim's and ink's nominal contrast
PASS_NOTE_P90 = 7.0  # WCAG's higher level for text, which dim's nominal 6.13 cannot reach
PASS_EDGE = 0.40  # below the footer: 5% of the frame's height
PASS_NOTE_WIDTH = 1.25  # the t = 3 note: 0.94 as staged, about 1.39 at 22 pt with the sign
BELOW = -2.85  # the footer is the only ink below this
NOTE_LEFT = 1.15
NOTE_RIGHT = 3.2
NOTE_YS = {"t = 1": -0.9 + 1.35 * 1.221, "t = 3": -0.9 + 1.35 * 1.822}
NOTE_HALF = 0.17


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
    per_unit = UNIT * w / 1280.0
    return {
        "box": (min(xs), min(ys), max(xs) + 1, max(ys) + 1),
        "left": (min(xs) - w / 2.0) / per_unit,
        "right": (max(xs) + 1 - w / 2.0) / per_unit,
        "top": (h / 2.0 - min(ys)) / per_unit,
        "bottom": (h / 2.0 - max(ys) - 1) / per_unit,
        "peak": ratio(inked[-1], ground),
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
        f"{name}: scene x {s['left']:.3f}..{s['right']:.3f} (width {s['right'] - s['left']:.3f}), y {s['bottom']:.3f}..{s['top']:.3f}; "
        f"contrast on ground, peak {s['peak']:.2f}, brightest tenth {s['p90']:.2f}, median {s['median']:.2f}"
    )


def footer_row(frame):
    return stats(frame, to_box(frame, -7.11, BELOW, 7.11, -4.0), ground_of(frame))


def footer_faults(s):
    if s is None:
        return ["no ink below the film"]
    bad = []
    edge = 4.0 + s["bottom"]
    if edge < PASS_EDGE:
        bad.append(f"{edge:.3f} from the frame's edge, wanted {PASS_EDGE}")
    if s["p90"] < PASS_FOOTER_P90:
        bad.append(f"brightest tenth {s['p90']:.2f}, wanted {PASS_FOOTER_P90}")
    return bad


def note_row(frame, name):
    y = NOTE_YS[name]
    return stats(frame, to_box(frame, NOTE_LEFT, y + NOTE_HALF, NOTE_RIGHT, y - NOTE_HALF), ground_of(frame))


def note_faults(s, name):
    if s is None:
        return [f"{name}: no ink"]
    bad = []
    if s["p90"] < PASS_NOTE_P90:
        bad.append(f"{name}: brightest tenth {s['p90']:.2f}, wanted {PASS_NOTE_P90}")
    if s["right"] - s["left"] < PASS_NOTE_WIDTH:
        bad.append(f"{name}: width {s['right'] - s['left']:.3f}, wanted {PASS_NOTE_WIDTH}")
    return bad


def cover_of(frame, box, ground, colour=DIM):
    data = frame.load()
    out = Image.new("L", (box[2] - box[0], box[3] - box[1]), 0)
    px = out.load()
    for y in range(box[1], box[3]):
        for x in range(box[0], box[2]):
            p = data[x, y]
            c = sum((p[i] - ground[i]) / (colour[i] - ground[i]) for i in range(3)) / 3.0
            px[x - box[0], y - box[1]] = int(round(255 * max(0.0, min(1.0, c))))
    return out


def paste(frame, cover, x0, y0, ground, colour):
    data = frame.load()
    cov = cover.load()
    for y in range(cover.size[1]):
        for x in range(cover.size[0]):
            a = cov[x, y] / 255.0
            if a > 0:
                data[x0 + x, y0 + y] = tuple(int(round(ground[i] + a * (colour[i] - ground[i]))) for i in range(3))


def runs(cover, min_gap):
    """Column spans of a coverage picture, split wherever at least min_gap columns are blank."""
    w, h = cover.size
    px = cover.load()
    on = [any(px[x, y] > 40 for y in range(h)) for x in range(w)]
    spans, begin, blank = [], None, 0
    for x, value in enumerate(on + [False] * (min_gap + 1)):
        if value:
            if begin is None:
                begin = x
            blank = 0
            last = x
        elif begin is not None:
            blank += 1
            if blank >= min_gap:
                spans.append((begin, last + 1))
                begin = None
    return spans


def mock_footer(frame, ground):
    s = footer_row(frame)
    cover = cover_of(frame, s["box"], ground)
    ImageDraw.Draw(frame).rectangle((0, s["box"][1] - 1, frame.size[0], s["box"][3] + 1), fill=ground)
    x0 = int(round(frame.size[0] / 2 - cover.size[0] / 2))
    y0 = int(round(frame.size[1] / 2 + 3.4 * UNIT - cover.size[1] / 2))
    paste(frame, cover, x0, y0, ground, INK)


def mock_note(frame, name, sign, ground):
    s = note_row(frame, name)
    cover = cover_of(frame, s["box"], ground)
    spans = runs(cover, 3)
    number = spans[-1]
    h = cover.size[1]
    left = cover.crop((0, 0, spans[-2][1], h))
    right = cover.crop((number[0], 0, number[1], h))
    gap = 5
    built = Image.new("L", (left.size[0] + gap + sign.size[0] + gap + right.size[0], max(h, sign.size[1])), 0)
    built.paste(left, (0, 0))
    built.paste(sign, (left.size[0] + gap, (built.size[1] - sign.size[1]) // 2))
    built.paste(right, (left.size[0] + gap + sign.size[0] + gap, 0))
    built = built.resize((int(round(built.size[0] * 22 / 18)), int(round(built.size[1] * 22 / 18))), Image.LANCZOS)
    ImageDraw.Draw(frame).rectangle((s["box"][0] - 1, s["box"][1] - 1, s["box"][2] + 1, s["box"][3] + 1), fill=ground)
    y0 = int(round((s["box"][1] + s["box"][3]) / 2 - built.size[1] / 2))
    paste(frame, built, s["box"][0], y0, ground, BLUE)


def report(frames_dir, out_png):
    d = Path(frames_dir)
    gauge_frame = Image.open(d / "frame-0115.png").convert("RGB")
    culver = Image.open(d / "frame-0550.png").convert("RGB")
    gallier = Image.open(d / "frame-0598.png").convert("RGB")
    ground = ground_of(gauge_frame)
    print(f"frames 115, 550 and 598, {gauge_frame.size[0]} x {gauge_frame.size[1]}, ground {ground}")
    print(f"nominal contrast: dim {ratio(DIM, ground):.2f}, blue {ratio(BLUE, ground):.2f}, ink {ratio(INK, ground):.2f}")
    failed = []

    print("\nas staged")
    rows = {"Liouville, frame 115": gauge_frame, "Culver, frame 550": culver, "Gallier, frame 598": gallier}
    for name, frame in rows.items():
        s = footer_row(frame)
        print(show(name, s), f"; {4.0 + s['bottom']:.3f} from the frame's edge")
        bad = footer_faults(s)
        print("  the probe fails it on: " + ("; ".join(bad) if bad else "NOTHING, CONTROL FAILED"))
        if not bad:
            failed.append(name)
    for name in NOTE_YS:
        s = note_row(gauge_frame, name)
        print(show(name + " note, frame 115", s))
        bad = note_faults(s, name)
        print("  the probe fails it on: " + ("; ".join(bad) if bad else "NOTHING, CONTROL FAILED"))
        if not bad:
            failed.append(name)
    word = stats(gauge_frame, to_box(gauge_frame, -0.6, 2.9, 2.4, 2.55), ground)
    print(show("for comparison, 'signed area of the square:', blue at 22 pt", word))
    live = stats(gauge_frame, to_box(gauge_frame, 0.45, 2.55, 1.4, 2.2), ground)
    print(show("for comparison, the live readout, blue at 24 pt", live))
    print(f"  the note's mark {PASS_NOTE_P90} against that blue word's brightest tenth {word['p90']:.2f}")
    if word["p90"] < PASS_NOTE_P90:
        failed.append("the blue word should clear the note's mark")

    # the sign, borrowed from "ln 2 ≈ 0.693", dim at 18 pt on frame 598
    source = stats(gallier, to_box(gallier, -1.0, -0.02, 1.0, -0.2), ground)
    cover = cover_of(gallier, source["box"], ground)
    parts = runs(cover, 3)
    print(f"\nthe borrowed row has {len(parts)} runs of ink: {parts}")
    sign_span = next(s for s in parts if 8 <= s[1] - s[0] <= 14 and parts.index(s) >= 2)
    sign = cover.crop((sign_span[0], 0, sign_span[1], cover.size[1]))

    # the mock-ups
    mock_g, mock_c, mock_l = gauge_frame.copy(), culver.copy(), gallier.copy()
    for name in NOTE_YS:
        mock_note(mock_g, name, sign, ground)
    for frame in (mock_g, mock_c, mock_l):
        mock_footer(frame, ground)
    print("\nmock-ups, NOT RENDERS")
    for name, frame in (("Liouville", mock_g), ("Culver", mock_c), ("Gallier", mock_l)):
        s = footer_row(frame)
        print(show(name, s), f"; {4.0 + s['bottom']:.3f} from the frame's edge")
        bad = footer_faults(s)
        print("  the probe: " + ("; ".join(bad) if bad else "passes"))
        if bad:
            failed.append("mock " + name)
    for name in NOTE_YS:
        s = note_row(mock_g, name)
        print(show(name + " note", s))
        bad = note_faults(s, name)
        print("  the probe: " + ("; ".join(bad) if bad else "passes"))
        if bad:
            failed.append("mock " + name)

    # is the room the wider notes need empty, on every sampled frame while they show?
    print("\nthe wider notes' new room, x 2.16 to 2.70, on every 0.5 s frame from 38.5 s to 69 s")
    import json

    times = json.loads((d / "sampling.json").read_text(encoding="utf8"))["times"]
    picked = [i for i, t in enumerate(times) if 38.5 <= t <= 69.0]
    hits = []
    for i in picked:
        frame = Image.open(d / f"frame-{i:04d}.png").convert("RGB")
        g = ground_of(frame)
        for name, y in NOTE_YS.items():
            if stats(frame, to_box(frame, 2.16, y + 0.13, 2.70, y - 0.13), g) is not None:
                hits.append((i, name))
    print(f"  {len(picked)} frames, ink in that room on: {hits or 'none'}")
    if hits:
        failed.append("the notes' room is not empty")

    w, h = gauge_frame.size
    strip = 26
    pictures = [
        ("As staged, root's stage06 frame 115", gauge_frame, (255, 235, 90)),
        ("NOT A RENDER: frame 115, notes and footer recoloured, scaled and moved by hand. The sign is borrowed from frame 598. Judge the layout, not the pixels.", mock_g, (255, 150, 150)),
        ("NOT A RENDER: frame 550, footer recoloured and moved by hand", mock_c, (255, 150, 150)),
        ("NOT A RENDER: frame 598, footer recoloured and moved by hand", mock_l, (255, 150, 150)),
    ]
    sheet = Image.new("RGB", (w, len(pictures) * (h + strip)), (0, 0, 0))
    draw = ImageDraw.Draw(sheet)
    for k, (title, pic, colour) in enumerate(pictures):
        draw.text((6, k * (h + strip) + 7), title, fill=colour)
        sheet.paste(pic, (0, k * (h + strip) + strip))
    sheet.save(out_png)
    print("\nwrote the sheet:", sheet.size)
    if failed:
        print("CONTROL FAILED: " + "; ".join(failed))
    return 1 if failed else 0


def probe_footer(path):
    frame = Image.open(path).convert("RGB")
    s = footer_row(frame)
    if s:
        print(show(Path(path).name, s))
    bad = footer_faults(s)
    for reason in bad:
        print("FAIL:", reason)
    print(("FAIL" if bad else "PASS") + ". Pick a frame while one footer is fully faded in. A probe, not a reading of the picture.")
    return 1 if bad else 0


def probe_notes(path):
    frame = Image.open(path).convert("RGB")
    bad = []
    for name in NOTE_YS:
        s = note_row(frame, name)
        if s:
            print(show(name, s))
        bad += note_faults(s, name)
    for reason in bad:
        print("FAIL:", reason)
    print(("FAIL" if bad else "PASS") + ". Pick a frame after the t = 3 note's fade, before 11_3's fade. A probe, not a reading of the picture.")
    return 1 if bad else 0


if __name__ == "__main__":
    if len(sys.argv) >= 4 and sys.argv[1] == "report":
        sys.exit(report(sys.argv[2], sys.argv[3]))
    if len(sys.argv) >= 3 and sys.argv[1] == "footer":
        sys.exit(probe_footer(sys.argv[2]))
    if len(sys.argv) >= 3 and sys.argv[1] == "notes":
        sys.exit(probe_notes(sys.argv[2]))
    raise SystemExit(__doc__)
