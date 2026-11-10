#!/usr/bin/env python3
"""Generate the JPEG conformance fixtures (fm-17m).

Encoders: ImageMagick (baseline/progressive/subsampling/gray/CMYK) and
ffmpeg (restart intervals) — independent implementations. References:
ImageMagick's decode to raw RGB (libjpeg-turbo), committed alongside.
EXIF orientation fixtures are built by injecting a hand-crafted APP1
segment (both endiannesses) into a baseline file; their references are
the base reference with the orientation transform applied here.

fmn-codec's decoder is compared against the references within a small
tolerance (the JPEG spec does not pin decoder output bit-exactly, but
our islow IDCT + triangle upsampling family sits within ±2).
"""

import os
import struct
import subprocess
import sys

HERE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "jpeg")
os.makedirs(HERE, exist_ok=True)
W, H = 64, 48


def sh(*args, **kw):
    r = subprocess.run(args, capture_output=True, **kw)
    if r.returncode != 0:
        sys.exit(f"FAILED: {' '.join(args)}\n{r.stderr.decode()[:500]}")
    return r.stdout


# Base pattern: smooth gradients + a sharp square (chroma edges).
ppm = bytearray(f"P6\n{W} {H}\n255\n".encode())
for y in range(H):
    for x in range(W):
        r = (x * 4) % 256
        g = (y * 5) % 256
        b = (255 - x * 2 - y) % 256
        if 20 <= x < 36 and 12 <= y < 28:
            r, g, b = 220, 40, 40
        ppm += bytes((r, g, b))
base_ppm = os.path.join(HERE, "base.ppm")
open(base_ppm, "wb").write(ppm)

variants = {
    "baseline_444": ["-sampling-factor", "1x1", "-quality", "92"],
    "baseline_420": ["-sampling-factor", "2x2", "-quality", "85"],
    "baseline_422": ["-sampling-factor", "2x1", "-quality", "85"],
    "baseline_440": ["-sampling-factor", "1x2", "-quality", "85"],
    "progressive_444": ["-sampling-factor", "1x1", "-interlace", "JPEG",
                        "-quality", "92"],
    "progressive_420": ["-sampling-factor", "2x2", "-interlace", "JPEG",
                        "-quality", "85"],
    "gray": ["-colorspace", "Gray", "-quality", "90"],
}
for name, args in variants.items():
    out = os.path.join(HERE, f"{name}.jpg")
    sh("magick", base_ppm, *args, out)

# Restart markers (DRI + RSTn every 2 MCUs), baseline 4:2:0.
restart = os.path.join(HERE, "restart_420.jpg")
sh("magick", base_ppm, "-sampling-factor", "2x2",
   "-define", "jpeg:restart-interval=2", "-quality", "85", restart)
blob = open(restart, "rb").read()
assert b"\xff\xdd" in blob, "no DRI marker in restart fixture"
assert b"\xff\xd0" in blob or b"\xff\xd1" in blob, "no RST markers"

# CMYK (policy-refusal fixture).
sh("magick", base_ppm, "-colorspace", "CMYK", "-quality", "90",
   os.path.join(HERE, "cmyk.jpg"))

# References: ImageMagick decode to raw RGB (no auto-orient).
names = list(variants) + ["restart_420"]
for name in names:
    rgb = sh("magick", os.path.join(HERE, f"{name}.jpg"), "rgb:-")
    assert len(rgb) == W * H * 3, (name, len(rgb))
    open(os.path.join(HERE, f"{name}.rgb"), "wb").write(rgb)

# EXIF orientation: inject APP1 into baseline_444.
def app1_exif(orientation, little_endian):
    if little_endian:
        tiff = b"II" + struct.pack("<H", 42) + struct.pack("<I", 8)
        ifd = struct.pack("<H", 1)
        ifd += struct.pack("<HHI", 0x0112, 3, 1) + struct.pack("<HH", orientation, 0)
        ifd += struct.pack("<I", 0)
    else:
        tiff = b"MM" + struct.pack(">H", 42) + struct.pack(">I", 8)
        ifd = struct.pack(">H", 1)
        ifd += struct.pack(">HHI", 0x0112, 3, 1) + struct.pack(">HH", orientation, 0)
        ifd += struct.pack(">I", 0)
    body = b"Exif\x00\x00" + tiff + ifd
    return b"\xff\xe1" + struct.pack(">H", len(body) + 2) + body


base_jpg = open(os.path.join(HERE, "baseline_444.jpg"), "rb").read()
assert base_jpg[:2] == b"\xff\xd8"
for name, orient, le in (("orient6_le", 6, True), ("orient3_be", 3, False)):
    patched = base_jpg[:2] + app1_exif(orient, le) + base_jpg[2:]
    open(os.path.join(HERE, f"{name}.jpg"), "wb").write(patched)

# Oriented references from the base reference.
base_rgb = open(os.path.join(HERE, "baseline_444.rgb"), "rb").read()


def orient_rgb(rgb, w, h, o):
    swapped = o >= 5
    ow, oh = (h, w) if swapped else (w, h)
    out = bytearray(len(rgb))
    for oy in range(oh):
        for ox in range(ow):
            if o == 3:
                sx, sy = w - 1 - ox, h - 1 - oy
            elif o == 6:
                sx, sy = oy, h - 1 - ox
            else:
                raise ValueError(o)
            s = (sy * w + sx) * 3
            d = (oy * ow + ox) * 3
            out[d:d + 3] = rgb[s:s + 3]
    return bytes(out), ow, oh


for name, o in (("orient6_le", 6), ("orient3_be", 3)):
    rgb, ow, oh = orient_rgb(base_rgb, W, H, o)
    open(os.path.join(HERE, f"{name}.rgb"), "wb").write(rgb)

os.remove(base_ppm)
print("jpeg fixtures done:", sorted(os.listdir(HERE)))
