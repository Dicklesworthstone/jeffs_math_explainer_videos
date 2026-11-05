"""Scratch: spacing audit of the 13 prediction sites on recorded frames, in the real typeface.

Geometry comes from the source (`at`, width) and the kit's scale; the audit first proves that
geometry by finding each rendered bar. The pause line's box comes from CMU Serif Roman metrics,
calibrated against labels already in the master. Read-only on V. Writes only with mode "x".
Usage: lmr_audit01.py V FONT OUT_TSV MOCK_DIR
"""
import hashlib
import json
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont

V, FONT, OUT_TSV, MOCK = sys.argv[1:5]
SCALE = 67.5  # cell pixels per scene unit: 960 pixels across 14.222 units
WORDS = "pause for more time"
SIZE = 22.0
DROP = 0.27
WIDTH_TOLERANCE = 1.04  # kit text width against the font, from the calibration below
DIM = (0x8A, 0x93, 0xA3)
HERO = None  # read from the rendered bar


def cell(x, y):
    return 480.0 + SCALE * x, 270.0 - SCALE * y


# id, file:line, cue picture number at mark plus half second, at (old), width, at (new or None)
SITES = [
    ("00_4", "ratio_ruler.rs:371", 48, (3.6, -2.75), 3.0, None),
    ("01_3", "building_ruler.rs:187", 121, (3.8, -2.9), 3.0, None),
    ("02_2", "slide_rule.rs:204", 202, (4.7, -2.75), 3.0, None),
    ("04_6", "undoing_growth.rs:749", 414, (4.6, 1.1), 3.0, None),
    ("05_1", "complex_turns.rs:245", 455, (2.5, -0.62), 3.0, None),
    ("05_5", "complex_turns.rs:636", 538, (2.7, -2.75), 3.0, (-2.7, 1.85)),
    ("06_2", "lifting_loops.rs:374", 598, (-4.1, 2.55), 2.6, (2.9, 1.05)),
    ("06_4", "lifting_loops.rs:475", 632, (2.4, 1.8), 2.6, None),
    ("07_3", "branch_cut.rs:419", 745, (2.6, 1.95), 3.0, None),
    ("08_1", "product_rule.rs:221", 820, (3.6, 0.6), 3.0, None),
    ("08_4", "product_rule.rs:346", 875, (3.1, -2.75), 3.0, None),
    ("11_3", "unreachable.rs:1070", 1111, (3.8, -1.4), 3.0, None),
    ("12_5", "what_it_forgets.rs:1123", 1311, (2.6, -2.1), 3.4, None),
]
OPENED = {48, 121, 202, 414, 455, 538, 598, 632, 634, 745, 820, 875, 1111, 1311}

d = json.load(open(os.path.join(V, "verification.json")))
pics = d["pictures"]
pl = {x["cue"]: i for i, x in enumerate(d["placements"])}
em = ImageFont.truetype(FONT, 1000)
line_w = em.getlength(WORDS) / 1000.0 * SIZE * 0.01 * SCALE
l, t, r, b = em.getbbox(WORDS)
line_h = (b - t) / 1000.0 * SIZE * 0.01 * SCALE
half_w = line_w * WIDTH_TOLERANCE / 2.0
half_h = line_h / 2.0
print("pause line: %.1f x %.1f cell px at size 22 (font); %.1f wide with tolerance" % (line_w, line_h, line_w * WIDTH_TOLERANCE))


def load(n):
    p = pics[n - 1]
    path = os.path.join(V, "frame-%04d.png" % n)
    assert hashlib.sha256(open(path, "rb").read()).hexdigest() == p["sha256"], n
    return np.asarray(Image.open(path).convert("RGB")).astype(int)


# calibration: labels already in the master, measured against the font
print("calibration, ink width in cell px, master against font at em = size x 0.01 unit:")
for n, words, size, box in [
    (538, "which inputs?", 24.0, (230, 105, 365, 130)),
    (598, "back at 1: logarithm = ?", 22.0, (570, 160, 770, 182)),
    (598, "length stays 1: real part stays 0", 20.0, (570, 128, 800, 148)),
    (632, "back where it began?", 22.0, (525, 110, 700, 132)),
    (745, "on the cut plane", 20.0, (680, 345, 810, 362)),
]:
    a = load(n)
    x0, y0, x1, y1 = box
    roi = a[y0:y1, x0:x1].max(axis=2)
    xs = np.where((roi > 60).any(axis=0))[0]
    want = em.getlength(words) / 1000.0 * size * 0.01 * SCALE
    print("  %-36s size %2d  master %5.1f  font %5.1f  ratio %.3f" % (words, size, xs[-1] - xs[0] + 1, want, (xs[-1] - xs[0] + 1) / want))

rows = ["site\tsource\tmark_frame\tnext_cue_frame\tbar_at_old\tbar_at_new\tbar_width\tbar_found_left_cell\tbar_expected_left_cell\tbar_found_y_cell"
        "\tbar_expected_y_cell\tline_box_cell\tpictures_in_hold\topened\tink_inside_line_box\tnearest_ink_cell_px"
        "\tnearest_ink_direction\tchanged_pixels_first_to_last\tnew_bar_box_ink\tverdict"]
for sid, src, pic, old, w, new in SITES:
    idx = pl[sid]
    mark = [p["frame"] for p in pics if p["seconds"] > 0 and any(r.endswith("mark predict") for r in p["reasons"])
            and d["placements"][idx]["start_frame"] < p["frame"] < d["placements"][idx + 1]["start_frame"]][0]
    nxt = d["placements"][idx + 1]["start_frame"]
    hold = [i + 1 for i, p in enumerate(pics) if mark <= p["frame"] < nxt]
    at = new or old
    ox0, oy = cell(old[0] - w / 2.0, old[1])
    # where the rendered bar is: bright warm pixels on the expected row, half a second in
    a = load(pic)
    warm = (a[..., 0] > 200) & (a[..., 1] > 160) & (a[..., 2] < 150)
    band = warm[int(oy) - 6:int(oy) + 7]
    rowsum = band.sum(axis=1)
    found_y = int(oy) - 6 + float(np.average(np.arange(band.shape[0]), weights=rowsum + 1e-9))
    xs = np.where(band[int(rowsum.argmax())])[0]
    # the longest run on the row: an arrow or rod in the same gold can share the row
    runs = np.split(xs, np.where(np.diff(xs) != 1)[0] + 1) if len(xs) else []
    xs = max(runs, key=len) if runs else xs
    found_left = int(xs[0]) if len(xs) else -1
    if HERO is None and len(xs):
        HERO = tuple(int(v) for v in np.median(a[int(oy)-6+int(rowsum.argmax()), xs], axis=0))
    cx, cy = cell(at[0], at[1] - DROP)
    box = (cx - half_w, cy - half_h, cx + half_w, cy + half_h)
    inside = 0
    nearest = 1e9
    where = ""
    first = last = None
    for n in hold:
        a = load(n)
        ink = a.max(axis=2) > 40
        # the bar never counts: its old place in frames recorded before the patch, and its own place
        for bx, by, bw in {(old[0], old[1], w), (at[0], at[1], w)}:
            x0, y = cell(bx - bw / 2.0, by)
            ink[int(y) - 6:int(y) + 7, max(int(x0) - 4, 0):int(x0 + bw * SCALE) + 5] = False
        ys, xs = np.nonzero(ink)
        dx = np.maximum(np.maximum(box[0] - xs, xs - box[2]), 0)
        dy = np.maximum(np.maximum(box[1] - ys, ys - box[3]), 0)
        dist = np.hypot(dx, dy)
        inside += int((dist == 0).sum())
        k = int(dist.argmin())
        if dist[k] < nearest:
            nearest = float(dist[k])
            ux, uy = xs[k] - cx, ys[k] - cy
            where = ("right" if ux > 0 else "left") if dy[k] == 0 else ("below" if uy > 0 else "above")
        first = a if first is None else first
        last = a
    bx0, by0 = cell(old[0] - w / 2.0, old[1])
    keep = np.ones(first.shape[:2], bool)
    keep[int(by0) - 6:int(by0) + 7, max(int(bx0) - 4, 0):int(bx0 + w * SCALE) + 5] = False
    changed = int(((np.abs(first - last).max(axis=2) > 24) & keep).sum())
    new_bar = ""
    if new:
        nx0, ny = cell(new[0] - w / 2.0, new[1])
        count = 0
        for n in hold:
            a = load(n)
            roi = a[int(ny) - 6:int(ny) + 7, int(nx0) - 4:int(nx0 + w * SCALE) + 5].max(axis=2) > 40
            count += int(roi.sum())
        new_bar = str(count)
    verdict = "clear" if inside == 0 and nearest >= 6 else ("tight" if inside == 0 else "COLLISION")
    rows.append("\t".join([
        sid, src, str(mark), str(nxt), "%.2f,%.2f" % old, "%.2f,%.2f" % new if new else "unchanged", "%.1f" % w,
        str(found_left), "%.1f" % ox0, "%.1f" % found_y, "%.1f" % oy,
        "%.0f,%.0f,%.0f,%.0f" % box, ",".join("%04d" % n for n in hold),
        ",".join("%04d" % n for n in hold if n in OPENED), str(inside), "%.1f" % nearest, where,
        str(changed), new_bar, verdict]))

    # stand-in picture: not native; recorded frame, recorded bar painted out if it moves
    arr = load(pic).astype("uint8")
    if new:
        # the recorded bar goes: each of its rows takes the row just above it, axis included
        for y in range(int(oy) - 5, int(oy) + 6):
            arr[y, max(int(ox0) - 3, 0):int(ox0 + w * SCALE) + 4] = arr[int(oy) - 7, max(int(ox0) - 3, 0):int(ox0 + w * SCALE) + 4]
    im = Image.fromarray(arr, "RGB")
    k = 4
    big = im.resize((960 * k, 540 * k), Image.LANCZOS)
    dr = ImageDraw.Draw(big)
    if new:
        nx0, ny = cell(at[0] - w / 2.0, at[1])
        dr.rectangle([nx0 * k, (ny - 2.4) * k, (nx0 + w * SCALE * 0.875) * k, (ny + 2.4) * k], fill=HERO)
    font = ImageFont.truetype(FONT, int(round(SIZE * 0.01 * SCALE * k)))
    tw = font.getlength(WORDS)
    x0, y0, x1, y1 = font.getbbox(WORDS)
    dr.text((cx * k - tw / 2.0, cy * k - (y0 + y1) / 2.0), WORDS, font=font, fill=DIM)
    big.resize((960, 540), Image.LANCZOS).save(os.path.join(MOCK, "mock-%s.png" % sid))

with open(OUT_TSV, "x", encoding="utf-8") as fh:
    fh.write("\n".join(rows) + "\n")
for r in rows:
    c = r.split("\t")
    print(c[0], c[2], c[3], c[5], "bar", c[7], c[8], c[9], c[10], "| ink", c[14], "near", c[15], c[16], "| changed", c[17], "| newbar", c[18], "|", c[19])
