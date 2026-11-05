#!/usr/bin/env python3
"""Draw the repair's computed new geometry over frames of the existing preview. Run from the repo root:

    python3 -I -B overlays.py OUT_DIR

These are EXISTING frames of complex-combined-picture-preview02 (canonical source, scratch voice) with
positions computed here by hand from the staged constants. Nothing here is a render of the repair:
a label is a box of estimated size, and the tile is a position check only.
"""
import cmath
import json
import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02")
W, H = 1280, 720
SX, SY = 7.111, 4.0
GOLD, BLUE, GREEN, INK, PINK, PURPLE, WARN = "#f2c14e", "#58a6ff", "#7ee081", "#ffffff", "#ff7fb6", "#c9a2ff", "#ff6b6b"


def px(p):
    return ((p[0] + SX) / (2 * SX) * W, (SY - p[1]) / (2 * SY) * H)


def frame_at(chapter, t):
    times = json.loads((ROOT / chapter / "sampling.json").read_text())["times"]
    n = min(range(len(times)), key=lambda k: abs(times[k] - t))
    path = ROOT / chapter / f"frame-{n:04d}.png"
    image = Image.open(path).convert("RGB")
    if image.size != (W, H):
        raise SystemExit(f"{path}: {image.size}")
    return image, n, times[n]


def plane(center, unit):
    return lambda z: (center[0] + unit * z.real, center[1] + unit * z.imag)


def arrow(d, a, b, color, width=3):
    d.line([px(a), px(b)], fill=color, width=width)
    (x0, y0), (x1, y1) = px(a), px(b)
    th = math.atan2(y1 - y0, x1 - x0)
    for sign in (-1, 1):
        d.line([(x1, y1), (x1 - 12 * math.cos(th + sign * 0.45), y1 - 12 * math.sin(th + sign * 0.45))], fill=color, width=width)


def label(d, at, half_w, half_h, text, color):
    (x, y) = px(at)
    rx, ry = half_w / (2 * SX) * W, half_h / (2 * SY) * H
    d.rectangle([x - rx, y - ry, x + rx, y + ry], outline=color)
    d.text((x, y), text, fill=color, anchor="mm", font=FONT)


def path(d, points, color, width=3):
    d.line([px(p) for p in points], fill=color, width=width)


def tag(image, name):
    d = ImageDraw.Draw(image)
    d.rectangle([0, 0, 760, 18], fill="#000000")
    d.text((4, 3), f"EXISTING FRAME + COMPUTED OVERLAY, NOT A RENDER: {name}", fill="#ffffff", font=FONT)


FONT = ImageFont.load_default(13)


def save(image, out, name):
    image.save(out / name)
    print(name)


def main():
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    unit = 2.3 / 1.959  # 1.1739, two_planes::planes
    inp, outp = plane((-3.5, -0.35), unit), plane((3.5, -0.35), unit)

    # 03_2: the rule's point, length and angle, over the frame of "squares each length"
    image, n, t = frame_at("03_two_planes", 20.5)
    d = ImageDraw.Draw(image)
    z = cmath.rect(1.4, math.pi / 6)
    w = z * z
    arrow(d, inp(0j), inp(z), BLUE)
    arrow(d, outp(0j), outp(w), GREEN)
    label(d, inp(complex(1.5, 0.85)), 0.16, 0.14, "|z|", BLUE)
    label(d, outp(complex(1.35, 1.75)), 0.24, 0.16, "|z|^2", GREEN)
    path(d, [inp(cmath.rect(0.55, z.imag and cmath.phase(z) * k / 32)) for k in range(33)], GOLD)
    path(d, [outp(cmath.rect(0.55, cmath.phase(w) * k / 32)) for k in range(33)], GOLD)
    label(d, inp(complex(0.92, 0.25)), 0.22, 0.13, "arg z", GOLD)
    label(d, outp(complex(0.82, 0.48)), 0.29, 0.13, "2 arg z", GOLD)
    tag(image, f"03_2 rule, frame {n}, t={t}")
    save(image, out, "overlay-03_2-rule.png")

    # 03_3: the predict card and its ring moved to the free corner of the output plane
    image, n, t = frame_at("03_two_planes", 34.5)
    d = ImageDraw.Draw(image)
    at = outp(complex(0.55, -0.65))
    h = 28 / 48 * 0.62 + 0.3
    half = 1.45
    (x0, y0), (x1, y1) = px((at[0] - half, at[1] + h / 2)), px((at[0] + half, at[1] - h / 2))
    d.rectangle([x0, y0, x1, y1], outline=PURPLE, width=2)
    d.text(px(at), "straight after z^2?", fill=PURPLE, anchor="mm", font=FONT)
    ring = (at[0], at[1] - h / 2 - 0.45)
    (cx, cy) = px(ring)
    r = 0.26 / (2 * SX) * W
    d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=PURPLE, width=2)
    tag(image, f"03_3 card, frame {n}, t={t}")
    save(image, out, "overlay-03_3-card.png")

    # 03_5: the flips, ring at e^{i th}, arrow to e^{-i th}, over the ticks
    image, n, t = frame_at("03_two_planes", 79.5)
    d = ImageDraw.Draw(image)
    for th, color in zip((0.6, 2.0, 3.6, 5.0), (GOLD, PURPLE, PINK, INK)):
        a, b = cmath.rect(1, th), cmath.rect(1, -th)
        (cx, cy) = px(outp(a))
        r = 0.1 / (2 * SX) * W
        d.ellipse([cx - r, cy - r, cx + r, cy + r], outline=color, width=2)
        sign = 1 if b.imag > a.imag else -1
        arrow(d, outp(complex(a.real, a.imag + sign * 0.1 / unit)), outp(complex(b.real, b.imag - sign * 0.1 / unit)), color)
    tag(image, f"03_5 flips, frame {n}, t={t}")
    save(image, out, "overlay-03_5-flips.png")

    # 04_3: walker on the loop, needle in the wheel, live count under the wheel
    p04 = plane((-2.6, -0.25), 1.6)
    image, n, t = frame_at("04_reading_color", 48.0)
    d = ImageDraw.Draw(image)
    (cx, cy) = px(p04(cmath.rect(0.9, 0.9)))
    d.ellipse([cx - 7, cy - 7, cx + 7, cy + 7], fill=INK)
    wheel = (4.55, 2.2)
    arrow(d, wheel, (wheel[0] + 0.75 * math.cos(1.8), wheel[1] + 0.75 * math.sin(1.8)), INK)
    label(d, (wheel[0], wheel[1] - 0.75 - 0.35), 0.32, 0.15, "+1.7", GOLD)
    tag(image, f"04_3 walk, frame {n}, t={t}")
    save(image, out, "overlay-04_3-walk.png")

    # 04_4's end: the count is gone by then, the card up: nothing to overlay but the box of 115 chars
    image, n, t = frame_at("04_reading_color", 69.0)
    tag(image, f"04_4 card as it stands in canonical, frame {n}, t={t}")
    save(image, out, "existing-04_4-card.png")

    # 05_3: the tile, on the true picture, with a 4x zoom beside it
    pe = plane((-2.6, -0.25 - 2 * 0.5333), 0.5333)
    s = math.tau / 12
    x, y = s * 0.5, s * 2.5
    corners = [complex(x, y), complex(x + s, y), complex(x + s, y + s), complex(x, y + s), complex(x, y)]
    image, n, t = frame_at("05_gallery", 44.0)
    clean = image.copy()
    d = ImageDraw.Draw(image)
    path(d, [pe(c) for c in corners], GOLD, 2)
    tag(image, f"05_3 tile, frame {n}, t={t}")
    save(image, out, "overlay-05_3-tile.png")
    (ax, ay) = px(pe(complex(x, y + s)))
    (bx, by) = px(pe(complex(x + s, y)))
    margin = 30
    box = (int(ax - margin), int(ay - margin), int(bx + margin), int(by + margin))
    zoom = 6
    raw = clean.crop(box).resize(((box[2] - box[0]) * zoom, (box[3] - box[1]) * zoom), Image.NEAREST)
    drawn = raw.copy()
    d = ImageDraw.Draw(drawn)
    pts = [px(pe(c)) for c in corners]
    d.line([((u - box[0]) * zoom, (v - box[1]) * zoom) for u, v in pts], fill=GOLD, width=1)
    both = Image.new("RGB", (raw.width * 2 + 10, raw.height), "#000000")
    both.paste(raw, (0, 0))
    both.paste(drawn, (raw.width + 10, 0))
    save(both, out, "zoom-05_3-tile.png")
    print("tile pixels", box, "pixel side", bx - ax, by - ay)

    # plain existing frames the handoff names
    for chapter, when, name in (
        ("02_turning_numbers", 92.0, "existing-02_6-end.png"),
        ("05_gallery", 140.0, "existing-05_8-card.png"),
        ("05_gallery", 158.0, "existing-05_9-tile.png"),
        ("05_gallery", 27.0, "existing-05_2-pole.png"),
    ):
        image, n, t = frame_at(chapter, when)
        tag(image, f"{name} frame {n}, t={t}, not the repair")
        save(image, out, name)


if __name__ == "__main__":
    main()
