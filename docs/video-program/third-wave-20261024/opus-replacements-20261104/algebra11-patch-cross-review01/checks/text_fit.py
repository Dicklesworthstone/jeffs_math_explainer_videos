#!/usr/bin/env python3
"""Run anywhere. Glyph coverage and text widths of the patch's words, from the pinned bundled font's tables.

Reads cmap and hmtx of cmunrm from franken_markdown 126a530 in the Cargo git checkout. Width follows fmn's own
layout: advance in ems x font size x 0.01 scene units. A check of tables, not of drawn glyphs.
"""
import os
import struct

FONT = os.path.expanduser("~/.cargo/git/checkouts/franken_markdown-2f7478214ec48ef9/126a530/fmd-font/fonts/computer-modern/cmunrm.ttf")
HALF = 7.11


def tables(data):
    count = struct.unpack(">H", data[4:6])[0]
    out = {}
    for i in range(count):
        tag, _, offset, length = struct.unpack(">4sIII", data[12 + 16 * i : 28 + 16 * i])
        out[tag.decode()] = (offset, length)
    return out


def cmap(data, table):
    offset = table["cmap"][0]
    _, count = struct.unpack(">HH", data[offset : offset + 4])
    best = None
    for i in range(count):
        _, _, sub = struct.unpack(">HHI", data[offset + 4 + 8 * i : offset + 12 + 8 * i])
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


data = open(FONT, "rb").read()
t = tables(data)
upem = struct.unpack(">H", data[t["head"][0] + 18 : t["head"][0] + 20])[0]
hhea = struct.unpack(">H", data[t["hhea"][0] + 34 : t["hhea"][0] + 36])[0]
mapping = cmap(data, t)


def advance(ch):
    gid = mapping[ord(ch)]
    at = t["hmtx"][0] + 4 * min(gid, hhea - 1)
    return struct.unpack(">H", data[at : at + 2])[0] / upem


def width(text, size):
    return sum(advance(ch) for ch in text) * size * 0.01


ROWS = [
    # who, words, size, centre x, left limit, right limit
    ("I", "Transposition: swap two labels", 32, 0.0, -HALF, HALF),
    ("V", "Gold: even", 26, -0.15, -2.25, 1.95),
    ("V", "Blue: odd", 26, -0.15, -2.25, 1.95),
    ("AA", "Card 2 went from slot 2 to slot 3", 27, 0.0, -HALF, HALF),
    ("AC", "By size: 2 fixed, 1 swap, 6 cycles of length 8", 27, 0.0, -HALF, HALF),
    ("AH", "After 3 stages: all data back on their starting lines", 27, 0.0, -HALF, HALF),
    ("AG", "Stage 4", 22, 3.9, -HALF, HALF),
    ("follow-up V", "Even", 26, -0.15, -2.25, 1.95),
    ("follow-up V", "Odd", 26, -0.15, -2.25, 1.95),
    ("follow-up AA", "Card 2 went from position 2 to position 3", 27, 0.0, -HALF, HALF),
    ("follow-up AH", "After 3 stages: all data back on their starting wires", 27, 0.0, -HALF, HALF),
    ("follow-up AH", "The same shuffle, on eight wires", 32, 0.0, -HALF, HALF),
]
print("left and right limits for the keys are the two cell blocks' inner edges, cell width taken from '1234' at size 24")
print(f"'1234' at 24: {width('1234', 24):.2f} wide; right cell ends {-2.5 + width('1234', 24) / 2:.2f}; next block's left cell starts {2.2 - width('1234', 24) / 2:.2f}")
for who, text, size, x, lo, hi in ROWS:
    missing = sorted({ch for ch in text if ord(ch) not in mapping})
    w = width(text, size)
    print(f"{who:13} {w:5.2f} wide, x {x - w / 2:+.2f} to {x + w / 2:+.2f}; inside {lo:+.2f} to {hi:+.2f}: {lo <= x - w / 2 and x + w / 2 <= hi}; missing glyphs {missing}; {text!r}")
