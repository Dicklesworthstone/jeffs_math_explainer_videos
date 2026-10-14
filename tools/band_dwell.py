#!/usr/bin/env python3
"""tools/band_dwell.py: how long each lower-third band of a chapter MP4 stands whole.

    uv run --quiet --no-project --with numpy python3 -I tools/band_dwell.py FILE.mp4|DIR... [options]

A lower third (the band at the foot of the frame that names a term, quotes a source or states a caveat)
has to stand long enough to be read: 2.6 s from fully up (the floor K's chapters keep, `draw::BAND_UP`).
The scene tests measure it on the scratch voice, where the lines are shorter and their gaps are not the
finished voice's, so a band that the finished read leaves 1.0 s or 0.3 s on screen passes them. This reads
the render instead.

The bottom 10% of each frame (30 frames a second) is compared with the chapter's background. A band is on
while at least 12% of that strip differs by more than 6 levels; it stands whole while the strip's summed
difference, which follows the band's opacity, is at least 90% of that band's peak. A band that is on for
under `--min-on-s` (0.2 s) is a cross-fade of one band into the next, not a band. Every band that stands
whole for under `--floor-s` is listed with where it falls and the line then speaking; a band that is still
on when the chapter ends says so (its closing is the chapter's, and the chapter should wait for it).

It is an advisory report (FLAG / PASS lines, exit 0; `--gate` exits 1 on a FLAG). A flag is a band to look
at: the cure is a pause of the floor after its fade-in before the next line takes it, or an earlier entry.
On 2026-10-14 it found eight such bands in G's narrated draft and five in H's (1.00 to 2.47 s), none of
them new to the thought gaps: the 13 October scored drafts had them too.
"""
import argparse
import os
import subprocess
import sys
from pathlib import Path

import numpy as np

sys.dont_write_bytecode = True

FPS = 30
FLOOR_S = 2.6
MIN_ON_S = 0.2
STRIP = 0.10  # the bottom tenth of the frame
SIZE = (960, 540)  # decoded at this size (a unit is 67.5 px at 960)
DEV = 6
SHARE = 0.12


def runs(mask):
    """The (start, end) index pairs of the runs of True in `mask`."""
    out, i = [], 0
    while i < len(mask):
        if mask[i]:
            j = i
            while j < len(mask) and mask[j]:
                j += 1
            out.append((i, j))
            i = j
        else:
            i += 1
    return out


def strip_frames(path, fps=FPS):
    """The bottom strip of every sampled frame of `path`, one row of uint8 pixels per frame."""
    w, h = SIZE
    rows = round(h * STRIP)
    env = {**os.environ, "PATH": "/opt/homebrew/bin:" + os.environ.get("PATH", "")}
    cmd = [
        "ffmpeg", "-v", "error", "-i", str(path), "-an",
        "-vf", f"fps={fps},scale={w}:{h},crop={w}:{rows}:0:{h - rows},format=gray",
        "-f", "rawvideo", "-",
    ]
    proc = subprocess.run(cmd, capture_output=True, env=env, check=False)
    if proc.returncode != 0:
        raise RuntimeError(f"ffmpeg could not read {path}: {proc.stderr.decode(errors='replace')[:200]}")
    n = w * rows
    raw = proc.stdout
    return np.frombuffer(raw[: len(raw) // n * n], dtype=np.uint8).reshape(-1, n)


def bands(frames, fps=FPS, min_on_s=MIN_ON_S):
    """Each band as (start_s, on_s, whole_s, still_on_at_the_end) from the strips of consecutive frames."""
    if len(frames) == 0:
        return []
    background = int(np.bincount(frames[::10].ravel(), minlength=256).argmax())
    dev = np.abs(frames.astype(np.int16) - background)
    present = (dev > DEV).sum(axis=1) >= SHARE * frames.shape[1]
    level = dev.sum(axis=1)
    out = []
    for a, b in runs(present):
        if (b - a) / fps < min_on_s:
            continue
        peak = level[a:b].max()
        full = runs(level[a:b] >= 0.9 * peak)
        whole = max((y - x for x, y in full), default=0) / fps
        out.append((a / fps, (b - a) / fps, whole, b == len(present)))
    return out


def first_lines(path):
    """[(start_s, id)] of the lines in the `.timeline.tsv` beside `path`, or []."""
    tl = Path(path).with_suffix(".timeline.tsv")
    if not tl.is_file():
        return []
    fps, lines = None, []
    for row in tl.read_text(encoding="utf-8", errors="replace").splitlines():
        c = row.split("\t")
        if c[0] == "fps" and len(c) > 1:
            fps = float(c[1])
        elif c[0] == "line" and len(c) > 2:
            lines.append((float(c[2]), c[1]))
    return sorted((f / fps, i) for f, i in lines) if fps else []


def report(path, floor_s=FLOOR_S, min_on_s=MIN_ON_S, fps=FPS):
    """(lines, number of flags) for one chapter file."""
    name = Path(path).name
    found = bands(strip_frames(path, fps), fps, min_on_s)
    lines_at = first_lines(path)
    out, flags = [], 0
    for start, on, whole, closing in found:
        if whole >= floor_s - 1e-9:
            continue
        flags += 1
        line = next((i for t, i in reversed(lines_at) if t <= start + 0.01), "-")
        end = " (still on when the chapter ends)" if closing else ""
        out.append(f"FLAG {name}: band from {start:.2f} s (line {line}) stands whole {whole:.2f} s, on {on:.2f} s{end}")
    if not flags:
        out.append(f"PASS {name}: {len(found)} bands, none whole for under {floor_s} s")
    return out, flags


def files_of(args):
    out = []
    for a in args:
        p = Path(a)
        if p.is_dir():
            out += sorted(f for f in p.glob("*.mp4") if "_watch" not in f.name and "master" not in f.name)
        else:
            out.append(p)
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("inputs", nargs="+", help="chapter MP4s, or directories of them")
    ap.add_argument("--floor-s", type=float, default=FLOOR_S, help="the least a band may stand whole (2.6)")
    ap.add_argument("--min-on-s", type=float, default=MIN_ON_S, help="shorter than this is a cross-fade, not a band (0.2)")
    ap.add_argument("--gate", action="store_true", help="exit 1 when anything is flagged")
    args = ap.parse_args(argv)
    files = files_of(args.inputs)
    if not files:
        print("band_dwell: no MP4 found", file=sys.stderr)
        return 2
    total = 0
    for f in files:
        lines, n = report(f, args.floor_s, args.min_on_s)
        total += n
        print("\n".join(lines))
    print(f"ADVISORY band dwell: {total} bands under {args.floor_s} s in {len(files)} chapter file(s)")
    return 1 if args.gate and total else 0


if __name__ == "__main__":
    sys.exit(main())
