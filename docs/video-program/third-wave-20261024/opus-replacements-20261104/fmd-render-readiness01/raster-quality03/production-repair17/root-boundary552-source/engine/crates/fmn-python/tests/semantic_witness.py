"""Semantic sanity oracles on the portal's frame routes (fm-5wq.46).

A bit-locked golden proves a frame unchanged, not right: the v0.4.0 wheel
rendered text upside down for 47 days while its goldens agreed with it
(fm-sq8.9). This suite draws the semantic witness through the portal and
asks questions a mirrored, misplaced or recoloured frame cannot answer. The
witness is the portal twin of `fmn::builtins::witness` (crates/fmn/src/lib.rs),
and the oracles are the twins of `fmn_conformance::semantic`:

- orientation: the red right triangle in the upper-left quadrant, full-width
  edge on top, right angle on the left; the F's bar on top, stem on the left;
- placement: the dot at UP*3 in the top band at its mapped pixel, the dot at
  LEFT*5 in the left band;
- colour: the triangle's interior is #FF0000, and the corners are the
  background;
- reading order: Text("AB") puts A left of B, and Tex("x^2") puts the
  superscript above and right of its base.

Each route's frame must pass, and its planted vertical mirror must fail
every orientation-sensitive oracle. Runs embedded (semantic_witness.rs's
Rust test) and against the installed wheel (scripts/check_portal_runtime.sh).
"""
import pathlib
import struct
import tempfile
import zlib

import numpy as np
import manimlib as m
from fmn_python import render_scene

WIDTH, HEIGHT = 320, 180

TRIANGLE = (255, 0, 0)
TRIANGLE_VERTICES = ((-6.6, 3.6, 0.0), (-4.2, 3.6, 0.0), (-6.6, 1.4, 0.0))
F_SHAPE = (255, 255, 255)
F_VERTICES = (
    (3.2, 0.8, 0.0), (3.7, 0.8, 0.0), (3.7, 2.0, 0.0), (5.0, 2.0, 0.0), (5.0, 2.5, 0.0),
    (3.7, 2.5, 0.0), (3.7, 3.1, 0.0), (5.6, 3.1, 0.0), (5.6, 3.6, 0.0), (3.2, 3.6, 0.0),
)
UP_DOT, UP_DOT_CENTER = (0, 255, 0), (0.0, 3.0, 0.0)
LEFT_DOT, LEFT_DOT_CENTER = (0, 0, 255), (-5.0, 0.0, 0.0)
DOT_RADIUS = 0.25
TEXT_A, TEXT_B, TEXT_CENTER = (255, 255, 0), (0, 255, 255), (3.6, -2.2, 0.0)
TEX_BASE, TEX_SUPERSCRIPT, TEX_CENTER = (255, 0, 255), (255, 128, 0), (-2.6, -2.4, 0.0)
FONT_SIZE = 96

CLASSIFY_TOLERANCE = 40
EXACT_TOLERANCE = 3


def hex_color(rgb):
    return "#{:02X}{:02X}{:02X}".format(*rgb)


def solid(mobject, rgb):
    return mobject.set_fill(hex_color(rgb), opacity=1).set_stroke(width=0)


def paint_glyphs(mobject, colors):
    glyphs = [part for part in mobject.family_members_with_points()]
    assert len(glyphs) == len(colors), (len(glyphs), len(colors))
    for glyph, rgb in zip(glyphs, colors):
        solid(glyph, rgb)
    return mobject


def build_witness():
    triangle = solid(m.Polygon(*TRIANGLE_VERTICES), TRIANGLE)
    f_shape = solid(m.Polygon(*F_VERTICES), F_SHAPE)
    up = solid(m.Dot(UP_DOT_CENTER, radius=DOT_RADIUS), UP_DOT)
    left = solid(m.Dot(LEFT_DOT_CENTER, radius=DOT_RADIUS), LEFT_DOT)
    text = paint_glyphs(m.Text("AB", font_size=FONT_SIZE).move_to(TEXT_CENTER), (TEXT_A, TEXT_B))
    tex = paint_glyphs(m.Tex("x^2", font_size=FONT_SIZE).move_to(TEX_CENTER), (TEX_BASE, TEX_SUPERSCRIPT))
    return [triangle, f_shape, up, left, text, tex]


def rgb8(rgba):
    return tuple(int(round(255 * float(c))) for c in list(rgba)[:3])


_scene_background = []


class SemanticWitness(m.Scene):
    def construct(self):
        # The configured background this scene renders over.
        _scene_background[:] = rgb8(self.camera.background_rgba)
        self.add(*build_witness())


def decode_png(path):
    """The portal's RGBA8 non-interlaced PNGs, stdlib zlib plus numpy."""
    payload = pathlib.Path(path).read_bytes()
    assert payload.startswith(b"\x89PNG\r\n\x1a\n")
    offset, packed, size = 8, bytearray(), None
    while offset < len(payload):
        length = struct.unpack(">I", payload[offset:offset + 4])[0]
        kind, body = payload[offset + 4:offset + 8], payload[offset + 8:offset + 8 + length]
        offset += 12 + length
        if kind == b"IHDR":
            width, height, depth, color, _, _, interlace = struct.unpack(">IIBBBBB", body)
            assert (depth, color, interlace) == (8, 6, 0)
            size = (width, height)
        elif kind == b"IDAT":
            packed.extend(body)
    width, height = size
    raw = np.frombuffer(zlib.decompress(bytes(packed)), dtype=np.uint8).reshape(height, width * 4 + 1)
    out = np.zeros((height, width * 4), dtype=np.int32)
    for y in range(height):
        mode, row = raw[y, 0], raw[y, 1:].astype(np.int32)
        up = out[y - 1] if y else np.zeros(width * 4, dtype=np.int32)
        if mode == 0:
            out[y] = row
        elif mode == 2:
            out[y] = (row + up) & 255
        else:
            line = np.zeros(width * 4 + 4, dtype=np.int32)
            upl = np.concatenate([np.zeros(4, dtype=np.int32), up])
            for x in range(width * 4):
                left, corner = line[x], upl[x]
                if mode == 1:
                    predictor = left
                elif mode == 3:
                    predictor = (left + up[x]) // 2
                else:
                    estimate = left + up[x] - corner
                    predictor = min((left, up[x], corner), key=lambda c: abs(estimate - c))
                line[x + 4] = (row[x] + predictor) & 255
            out[y] = line[4:]
    return out.astype(np.uint8).reshape(height, width, 4)


def pixels_of(image, rgb):
    diff = np.abs(image[:, :, :3].astype(np.int16) - np.array(rgb, dtype=np.int16))
    ys, xs = np.nonzero((diff <= CLASSIFY_TOLERANCE).all(axis=2))
    return xs, ys


def thirds(coords):
    if coords.size == 0:
        return 0, 0
    lo, span = coords.min(), coords.max() - coords.min() + 1
    rel = (coords - lo) * 3
    return int((rel < span).sum()), int((rel >= 2 * span).sum())


def heavier(a, b):
    return a > 0 and a >= 2 * b


def to_pixel(image, point):
    height, width = image.shape[:2]
    width_units = 8.0 * width / height
    return (point[0] / width_units + 0.5) * width, (0.5 - point[1] / 8.0) * height


def read_witness(image, background, reading_order=True):
    height, width = image.shape[:2]
    readings = {}
    xs, ys = pixels_of(image, TRIANGLE)
    share = float(((2 * xs < width) & (2 * ys < height)).mean()) if xs.size else 0.0
    top, bottom = thirds(ys)
    left, right = thirds(xs)
    readings["orientation.triangle"] = (
        share >= 0.95 and heavier(top, bottom) and heavier(left, right),
        f"mass {xs.size} upper_left_share {share:.3f} top {top} bottom {bottom} left {left} right {right}",
    )
    xs, ys = pixels_of(image, F_SHAPE)
    top, bottom = thirds(ys)
    left, right = thirds(xs)
    upper_right = int(((2 * xs >= width) & (2 * ys < height)).sum())
    readings["orientation.f_shape"] = (
        xs.size > 0 and upper_right == xs.size and heavier(top, bottom) and heavier(left, right),
        f"mass {xs.size} upper_right {upper_right} top {top} bottom {bottom} left {left} right {right}",
    )
    for name, rgb, center, band in (
        ("placement.up_dot", UP_DOT, UP_DOT_CENTER, lambda cx, cy: cy < height / 4),
        ("placement.left_dot", LEFT_DOT, LEFT_DOT_CENTER, lambda cx, cy: cx < width / 4),
    ):
        xs, ys = pixels_of(image, rgb)
        ex, ey = to_pixel(image, center)
        if xs.size:
            cx, cy = float(xs.mean()), float(ys.mean())
            ok = abs(cx - ex) <= 0.02 * width and abs(cy - ey) <= 0.02 * height and band(cx, cy)
            readings[name] = (ok, f"mass {xs.size} centroid ({cx:.1f}, {cy:.1f}) expected ({ex:.1f}, {ey:.1f})")
        else:
            readings[name] = (False, "mass 0")
    inside = tuple(sum(v[i] for v in TRIANGLE_VERTICES) / 3 for i in range(3))
    x, y = to_pixel(image, inside)
    sample = image[int(y), int(x)] if 0 <= x < width and 0 <= y < height else None
    readings["colour.fill"] = (
        sample is not None and all(abs(int(c) - t) <= EXACT_TOLERANCE for c, t in zip(sample[:3], TRIANGLE)),
        f"pixel ({x:.0f}, {y:.0f}) = {None if sample is None else tuple(int(c) for c in sample)}",
    )
    corners = [image[0, 0], image[0, -1], image[-1, 0], image[-1, -1]]
    readings["colour.background"] = (
        all(all(abs(int(c) - t) <= EXACT_TOLERANCE for c, t in zip(p[:3], background)) for p in corners),
        f"corners {[tuple(int(c) for c in p) for p in corners]}",
    )
    if reading_order:
        def centroid(rgb):
            xs, ys = pixels_of(image, rgb)
            return (float(xs.mean()), float(ys.mean())) if xs.size else None
        a, b = centroid(TEXT_A), centroid(TEXT_B)
        readings["reading_order.text"] = (
            a is not None and b is not None and a[0] < b[0] and abs(a[1] - b[1]) <= 0.05 * height,
            f"A {a} B {b}",
        )
        base, sup = centroid(TEX_BASE), centroid(TEX_SUPERSCRIPT)
        readings["reading_order.tex"] = (
            base is not None and sup is not None and sup[0] > base[0] and sup[1] < base[1],
            f"base {base} superscript {sup}",
        )
    return readings


MUST_REJECT_FLIP = ("orientation.triangle", "orientation.f_shape", "placement.up_dot",
                    "colour.fill", "reading_order.tex")


_semantic_readings = []


def check_route(route, image, background):
    assert image.shape == (HEIGHT, WIDTH, 4), (route, image.shape)
    for verdict, frame in (("rendered", image), ("planted_vertical_flip", np.flipud(image))):
        readings = read_witness(frame, background)
        for oracle, (ok, measured) in readings.items():
            _semantic_readings.append((route, verdict, oracle, measured, bool(ok)))
            print(f"semantic-oracle route={route} frame={verdict} oracle={oracle} "
                  f"measured={measured!r} pass={ok}")
        failed = [oracle for oracle, (ok, _) in readings.items() if not ok]
        if verdict == "rendered":
            assert not failed, (route, failed, readings)
        else:
            missed = [oracle for oracle in MUST_REJECT_FLIP if oracle not in failed]
            assert not missed, (route, "a vertical mirror passed", missed)


with tempfile.TemporaryDirectory() as _semantic_root:
    _still = pathlib.Path(_semantic_root) / "still.png"
    render_scene(SemanticWitness, _still, resolution=(WIDTH, HEIGHT), threads=1)
    assert len(_scene_background) == 3, "the witness scene ran"
    check_route("portal_scene", decode_png(_still), tuple(_scene_background))

_semantic_camera = m.Camera(resolution=(WIDTH, HEIGHT))
_semantic_camera.capture(*build_witness())
check_route("portal_camera_readback", _semantic_camera.get_pixel_array(),
            rgb8(_semantic_camera.background_rgba))
semantic_witness_report = {"routes": 2, "readings": _semantic_readings}
