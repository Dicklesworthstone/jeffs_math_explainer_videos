#!/usr/bin/env python3
"""Glyph coverage and text widths from the pinned bundled fonts, by reading their tables.

Reads cmap and hmtx from franken_markdown 126a530 in the Cargo git checkout. Width follows
fmn's own layout: advance in ems x font size x 0.01 scene units. TeX widths are not
estimated here. A check of tables, not of drawn glyphs.
"""
import os
import struct

BASE = os.path.expanduser("~/.cargo/git/checkouts/franken_markdown-2f7478214ec48ef9/126a530/fmd-font/fonts/")
CM = BASE + "computer-modern/cmunrm.ttf"
MATH = BASE + "noto-sans-math/NotoSansMathSymbols.ttf"
PLEX = BASE + "ibm-plex-sans/IBMPlexSans-Regular.ttf"
HALF_WIDTH = 7.11


def tables(data):
    count = struct.unpack(">H", data[4:6])[0]
    out = {}
    for i in range(count):
        tag, _, offset, length = struct.unpack(">4sIII", data[12 + 16 * i : 28 + 16 * i])
        out[tag.decode()] = (offset, length)
    return out


def cmap(data, table):
    offset = table["cmap"][0]
    version, count = struct.unpack(">HH", data[offset : offset + 4])
    best = None
    for i in range(count):
        platform, encoding, sub = struct.unpack(">HHI", data[offset + 4 + 8 * i : offset + 12 + 8 * i])
        fmt = struct.unpack(">H", data[offset + sub : offset + sub + 2])[0]
        if fmt == 12:
            best = (12, offset + sub)
            break
        if fmt == 4 and best is None:
            best = (4, offset + sub)
    fmt, at = best
    mapping = {}
    if fmt == 12:
        groups = struct.unpack(">I", data[at + 12 : at + 16])[0]
        for g in range(groups):
            start, end, gid = struct.unpack(">III", data[at + 16 + 12 * g : at + 28 + 12 * g])
            for c in range(start, end + 1):
                mapping[c] = gid + c - start
    else:
        seg = struct.unpack(">H", data[at + 6 : at + 8])[0] // 2
        ends = struct.unpack(f">{seg}H", data[at + 14 : at + 14 + 2 * seg])
        starts = struct.unpack(f">{seg}H", data[at + 16 + 2 * seg : at + 16 + 4 * seg])
        deltas = struct.unpack(f">{seg}h", data[at + 16 + 4 * seg : at + 16 + 6 * seg])
        ranges_at = at + 16 + 6 * seg
        ranges = struct.unpack(f">{seg}H", data[ranges_at : ranges_at + 2 * seg])
        for i in range(seg):
            for c in range(starts[i], ends[i] + 1):
                if c == 0xFFFF:
                    continue
                if ranges[i] == 0:
                    gid = (c + deltas[i]) & 0xFFFF
                else:
                    where = ranges_at + 2 * i + ranges[i] + 2 * (c - starts[i])
                    gid = struct.unpack(">H", data[where : where + 2])[0]
                    gid = (gid + deltas[i]) & 0xFFFF if gid else 0
                if gid:
                    mapping[c] = gid
    return mapping


def font(path):
    data = open(path, "rb").read()
    table = tables(data)
    upem = struct.unpack(">H", data[table["head"][0] + 18 : table["head"][0] + 20])[0]
    metrics = struct.unpack(">H", data[table["hhea"][0] + 34 : table["hhea"][0] + 36])[0]
    at = table["hmtx"][0]
    advances = [struct.unpack(">H", data[at + 4 * i : at + 4 * i + 2])[0] for i in range(metrics)]
    return cmap(data, table), advances, upem


chars, advances, upem = font(CM)
math_chars, _, _ = font(MATH)
plex_chars, _, _ = font(PLEX)
print("coverage of characters these scenes send to Text (fallback chain: CM, then Noto Sans Math, then IBM Plex Sans):")
for ch in "²³⁴≤|=/?:,.":
    faces = [("CM", chars), ("math", math_chars), ("Plex", plex_chars)]
    found = next((name for name, table in faces if ord(ch) in table), "NONE: Text build fails")
    print(f"  {ch!r} U+{ord(ch):04X}: served by {found}")


def width(text, size):
    total = 0
    for ch in text:
        gid = chars.get(ord(ch))
        if gid is None:
            total += upem * 0.6  # outside CM; a plain estimate
        else:
            total += advances[min(gid, len(advances) - 1)]
    return total / upem * size * 0.01


ITEMS = [
    # scene, text, size, centre x, note
    ("factorials", "Where the factorials come from", 35.0, 0.0),
    ("factorials", "Which coefficient makes the constant column agree?", 24.0, 0.0),
    ("factorials", "A finite polynomial loses its highest power", 25.0, 0.0),
    ("factorials", "The infinite tail needs a proof", 25.0, 0.0),
    ("factorials", "Uniform convergence on every bounded interval", 25.0, 0.0),
    ("factorials", "derivative", 23.0, 2.6),
    ("factorials", "partial sums on fixed axes", 23.0, 0.0),
    ("factorials", "|x| ≤ 1.5, degree 4: tail ≤ 0.08438", 25.0, 0.0),
    ("product_grid", "Why addition becomes multiplication", 34.0, 0.0),
    ("product_grid", "A corner of an infinite grid", 22.0, -2.1),
    ("product_grid", "What is the degree-two diagonal?", 23.0, 2.9),
    ("product_grid", "Absolute convergence permits regrouping", 22.0, 0.0),
    ("product_grid", "Complete diagonals: degrees 0 through 3", 22.0, 3.0),
]
print("text extents (scene units; the frame is +/-7.11):")
for scene, text, size, centre in ITEMS:
    w = width(text, size)
    left, right = centre - w / 2, centre + w / 2
    flag = "" if -HALF_WIDTH < left and right < HALF_WIDTH else "  OFF FRAME"
    print(f"  {scene}: {w:5.2f} wide, {left:6.2f} to {right:5.2f}  {text!r}{flag}")
question = width("What is the degree-two diagonal?", 23.0)
print(f"product_grid: question's left edge {2.9 - question / 2:.2f}; the grid's rules end at 0.50")
