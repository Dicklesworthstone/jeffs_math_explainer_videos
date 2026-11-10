#!/usr/bin/env python3
"""Generate the PNG conformance fixture matrix (fm-17m).

Pure CPython (zlib + struct only) — an implementation independent of
fmn-codec. For every variant this writes <name>.png and <name>.rgba
(the expected canonical-RGBA8 bytes), derived from the same abstract
pattern definition, NOT by decoding the PNG.

Deterministic: rerunning produces identical bytes (zlib level fixed).

Cross-validation note (2026-07-22): every 8-bit/palette/tRNS/filter/
Adam7 fixture's expected RGBA was verified byte-exact against
ImageMagick (libpng). The 16-bit fixtures differ from ImageMagick by
at most 1/255 on some samples because libpng-based decoders CHOP
(v >> 8) while fmn-codec quantizes by exact rounding
((v*255 + 32767)/65535) — the documented §14.2 policy. The expected
bytes here encode the rounding policy on purpose.
"""

import os
import struct
import zlib

HERE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "png")
os.makedirs(HERE, exist_ok=True)

W, H = 13, 7  # deliberately odd, exercises sub-byte row padding
PASSES = [(0, 0, 8, 8), (4, 0, 8, 8), (0, 4, 4, 8), (2, 0, 4, 4),
          (0, 2, 2, 4), (1, 0, 2, 2), (0, 1, 1, 2)]


def chunk(name: bytes, body: bytes) -> bytes:
    return (struct.pack(">I", len(body)) + name + body
            + struct.pack(">I", zlib.crc32(name + body) & 0xFFFFFFFF))


def pack_row(samples, depth):
    """Pack one scanline's samples (already channel-interleaved)."""
    if depth == 16:
        return b"".join(struct.pack(">H", s) for s in samples)
    if depth == 8:
        return bytes(samples)
    out = bytearray()
    per = 8 // depth
    for i in range(0, len(samples), per):
        byte = 0
        group = samples[i:i + per]
        for j, s in enumerate(group):
            byte |= (s & ((1 << depth) - 1)) << (8 - (j + 1) * depth)
        out.append(byte)
    return bytes(out)


def sub_filter(row: bytes, prev: bytes, bpp: int) -> bytes:
    """PNG filter type 1 (Sub)."""
    out = bytearray()
    for i, b in enumerate(row):
        left = row[i - bpp] if i >= bpp else 0
        out.append((b - left) & 0xFF)
    return bytes(out)


def up_filter(row: bytes, prev: bytes, bpp: int) -> bytes:
    out = bytearray()
    for i, b in enumerate(row):
        up = prev[i] if prev else 0
        out.append((b - up) & 0xFF)
    return bytes(out)


def avg_filter(row: bytes, prev: bytes, bpp: int) -> bytes:
    out = bytearray()
    for i, b in enumerate(row):
        left = row[i - bpp] if i >= bpp else 0
        up = prev[i] if prev else 0
        out.append((b - (left + up) // 2) & 0xFF)
    return bytes(out)


def paeth_filter(row: bytes, prev: bytes, bpp: int) -> bytes:
    out = bytearray()
    for i, b in enumerate(row):
        a = row[i - bpp] if i >= bpp else 0
        u = prev[i] if prev else 0
        c = prev[i - bpp] if (i >= bpp and prev) else 0
        p = a + u - c
        pa, pb, pc = abs(p - a), abs(p - u), abs(p - c)
        pred = a if (pa <= pb and pa <= pc) else (u if pb <= pc else c)
        out.append((b - pred) & 0xFF)
    return bytes(out)


FILTERS = [None, sub_filter, up_filter, avg_filter, paeth_filter]


def serialize(passes, depth, channels, filter_cycle):
    """Per-pass scanlines -> filtered byte stream.

    The filter 'previous scanline' resets at each pass boundary, per
    the PNG spec. The filter id cycles per row within a pass.
    """
    bpp = max(1, (depth * channels) // 8)
    stream = bytearray()
    for rows in passes:
        prev = b""
        for y, samples in enumerate(rows):
            raw = pack_row(samples, depth)
            f = filter_cycle[y % len(filter_cycle)]
            if f == 0:
                stream.append(0)
                stream.extend(raw)
            else:
                stream.append(f)
                stream.extend(FILTERS[f](raw, prev, bpp))
            prev = raw
    return bytes(stream)


def interlaced_passes(pixel_rows):
    """Adam7 split: list of passes, each a list of scanlines."""
    h = len(pixel_rows)
    w = len(pixel_rows[0])
    out = []
    for x0, y0, xs, ys in PASSES:
        pw = max(0, (w - x0 + xs - 1) // xs)
        ph = max(0, (h - y0 + ys - 1) // ys)
        if pw == 0 or ph == 0:
            continue
        rows = []
        for py in range(ph):
            row = []
            for px in range(pw):
                row.extend(pixel_rows[y0 + py * ys][x0 + px * xs])
            rows.append(row)
        out.append(rows)
    return out


def write_png(name, color_type, depth, pixel_rows, expected_rgba,
              palette=None, trns=None, interlace=0, filter_cycle=(0,)):
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[color_type]
    if interlace:
        passes = interlaced_passes(pixel_rows)
    else:
        passes = [[[s for px in row for s in px] for row in pixel_rows]]
    stream = serialize(passes, depth, channels, filter_cycle)
    png = bytearray(b"\x89PNG\r\n\x1a\n")
    png += chunk(b"IHDR", struct.pack(">IIBBBBB", W, H, depth, color_type,
                                      0, 0, interlace))
    if palette is not None:
        png += chunk(b"PLTE", b"".join(bytes(e) for e in palette))
    if trns is not None:
        png += chunk(b"tRNS", trns)
    png += chunk(b"IDAT", zlib.compress(stream, 6))
    png += chunk(b"IEND", b"")
    open(os.path.join(HERE, name + ".png"), "wb").write(png)
    flat = bytearray()
    for row in expected_rgba:
        for px in row:
            flat.extend(px)
    assert len(flat) == W * H * 4, name
    open(os.path.join(HERE, name + ".rgba"), "wb").write(flat)
    print(f"{name}: {len(png)} bytes")


def scale(v, depth):
    if depth == 16:
        return (v * 255 + 32767) // 65535
    return v * (255 // ((1 << depth) - 1))


def maxv(depth):
    return (1 << depth) - 1


# --- grayscale, every depth, filters cycling on the 8-bit one --------
for depth in (1, 2, 4, 8, 16):
    m = maxv(depth)
    pix = [[( (x * y + x + 3 * y) % (m + 1), ) for x in range(W)]
           for y in range(H)]
    exp = [[(lambda g: (g, g, g, 255))(scale(p[0], depth)) for p in row]
           for row in pix]
    cycle = (0, 1, 2, 3, 4) if depth in (8, 16) else (0,)
    write_png(f"gray{depth}", 0, depth, pix, exp, filter_cycle=cycle)

# --- gray + tRNS: sample value 5 (8-bit) / 300 (16-bit) is transparent
for depth, key in ((8, 5), (16, 300)):
    m = maxv(depth)
    pix = [[((x * 37 + y * 111 + key * (x == y)) % (m + 1),)
            for x in range(W)] for y in range(H)]
    # Force a few exact hits.
    for i in range(min(W, H)):
        pix[i][i] = (key,)
    exp = [[(lambda v: (scale(v, depth),) * 3 + (0 if v == key else 255,))(p[0])
            for p in row] for row in pix]
    write_png(f"gray{depth}_trns", 0, depth, pix, exp,
              trns=struct.pack(">H", key))

# --- grayscale + alpha ------------------------------------------------
for depth in (8, 16):
    m = maxv(depth)
    pix = [[((x * 19 + y) % (m + 1), (y * 43 + x) % (m + 1))
            for x in range(W)] for y in range(H)]
    exp = [[(scale(p[0], depth),) * 3 + (scale(p[1], depth),)
            for p in row] for row in pix]
    write_png(f"graya{depth}", 4, depth, pix, exp,
              filter_cycle=(0, 2, 4))

# --- truecolor --------------------------------------------------------
for depth in (8, 16):
    m = maxv(depth)
    pix = [[((x * 20) % (m + 1), (y * 36) % (m + 1), (x * y * 7) % (m + 1))
            for x in range(W)] for y in range(H)]
    exp = [[tuple(scale(s, depth) for s in p) + (255,) for p in row]
           for row in pix]
    write_png(f"rgb{depth}", 2, depth, pix, exp,
              filter_cycle=(1, 3, 0, 4, 2))

# --- rgb + tRNS: one exact color is transparent ----------------------
key = (10, 20, 30)
pix = [[(x * 5 % 256, y * 11 % 256, (x + y) % 256) for x in range(W)]
       for y in range(H)]
pix[2][2] = key
pix[5][9] = key
exp = [[p + ((0 if p == key else 255),) for p in row] for row in pix]
write_png("rgb8_trns", 2, 8, pix, exp, trns=struct.pack(">HHH", *key))

# --- rgba -------------------------------------------------------------
for depth in (8, 16):
    m = maxv(depth)
    pix = [[((x * 20) % (m + 1), (y * 36) % (m + 1),
             (x * y * 7) % (m + 1), (x + y * 16) % (m + 1))
            for x in range(W)] for y in range(H)]
    exp = [[tuple(scale(s, depth) for s in p) for p in row] for row in pix]
    write_png(f"rgba{depth}", 6, depth, pix, exp,
              filter_cycle=(4, 3, 2, 1, 0))

# --- indexed, every legal depth, with and without tRNS ---------------
for depth in (1, 2, 4, 8):
    n = min(1 << depth, 16) if depth < 8 else 200
    palette = [((i * 53) % 256, (i * 97) % 256, (i * 13) % 256)
               for i in range(n)]
    pix = [[((x + y * W) % n,) for x in range(W)] for y in range(H)]
    exp = [[palette[p[0]] + (255,) for p in row] for row in pix]
    write_png(f"pal{depth}", 3, depth, pix, exp, palette=palette)

# pal8 with partial tRNS (first 3 entries translucent).
n = 40
palette = [((i * 11) % 256, (i * 29) % 256, (i * 71) % 256) for i in range(n)]
trns = bytes([0, 128, 200])
pix = [[((x * 3 + y) % n,) for x in range(W)] for y in range(H)]
exp = [[palette[p[0]] + (trns[p[0]] if p[0] < 3 else 255,) for p in row]
       for row in pix]
write_png("pal8_trns", 3, 8, pix, exp, palette=palette, trns=trns)

# --- Adam7 interlaced variants ---------------------------------------
pix = [[((x * 20) % 256, (y * 36) % 256, (x * y * 7) % 256,
         (x + y * 16) % 256) for x in range(W)] for y in range(H)]
exp = [[p for p in row] for row in pix]
write_png("rgba8_adam7", 6, 8, pix, exp, interlace=1,
          filter_cycle=(0, 1, 2, 3, 4))

m = 65535
pix = [[((x * 4999 + y * 331) % (m + 1),) for x in range(W)]
       for y in range(H)]
exp = [[(scale(p[0], 16),) * 3 + (255,) for p in row] for row in pix]
write_png("gray16_adam7", 0, 16, pix, exp, interlace=1,
          filter_cycle=(0, 2))

pal = [((i * 7) % 256, (i * 3) % 256, (255 - i) % 256) for i in range(16)]
pix = [[((x ^ y) % 16,) for x in range(W)] for y in range(H)]
exp = [[pal[p[0]] + (255,) for p in row] for row in pix]
write_png("pal4_adam7", 3, 4, pix, exp, palette=pal, interlace=1)

# --- color-intent chunks ---------------------------------------------
png = bytearray(b"\x89PNG\r\n\x1a\n")
png += chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 0, 0, 0, 0))
png += chunk(b"gAMA", struct.pack(">I", 45455))
png += chunk(b"sRGB", b"\x01")
png += chunk(b"IDAT", zlib.compress(b"\x00\x7f", 6))
png += chunk(b"IEND", b"")
open(os.path.join(HERE, "intent_srgb_wins.png"), "wb").write(png)

png = bytearray(b"\x89PNG\r\n\x1a\n")
png += chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 0, 0, 0, 0))
png += chunk(b"gAMA", struct.pack(">I", 100000))
png += chunk(b"IDAT", zlib.compress(b"\x00\x7f", 6))
png += chunk(b"IEND", b"")
open(os.path.join(HERE, "intent_gamma.png"), "wb").write(png)

print("done")
