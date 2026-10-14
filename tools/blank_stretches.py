#!/usr/bin/env python3
"""tools/blank_stretches.py: the stretches of a chapter MP4 that show no picture.

    uv run --quiet --no-project --with numpy python3 -I tools/blank_stretches.py FILE.mp4|DIR... [options]

A frame shows no picture when it is flat: all but a handful of its pixels sit within a few levels of
the frame's own most common value (so the background and a digital-black frame both count, and a
frame with one small label does not). A stretch is a run of such frames; every stretch longer than
`--min-s` (0.5 s) is listed with where it falls in the chapter:

  opening   starts at the first frame. A chapter opens on the silence before its first line (the
            kit's breath, or under LineGaps::Thought the gap the script's thought break asks for),
            and its first picture comes up with that line. The opening is excused until the
            chapter's first line starts, read from the `.timeline.tsv` beside the MP4, and
            `--slack-s` (0.15 s) after it for the first mark to rise; with no timeline it is excused
            to `--lead-s` (0.75 s). The bound comes from the seven scored drafts of 2026-10-14
            (99 chapter files, constant gaps): 31 open on a picture at once, 60 have a lead of 0.60
            to 0.70 s, which is where the first line starts, and seven have a first picture 1.2 to
            4.0 s after it. Past the line is a chapter that says its first sentence over an empty
            frame (G's chapter 15 did: the first icon came with its name, 2.9 s into the line).
  inside    anywhere else: J's 05 had 0.6 s of background mid-chapter and K had bare stages.
  closing   ends at the last frame.
  whole     the chapter never shows a picture.

It is an advisory report: it prints PASS / FLAG lines and exits 0 (`--gate` exits 1 on a FLAG). Each
flag says where to look; whether a pause is a defect is read in the frames around it.

Frames are read through ffmpeg at `--fps` (30) and 320x180, area-averaged so a thin line or a small
label survives. A pixel counts as picture when it is more than 8 levels from the frame's most common
value, and a frame is flat when 6 or fewer pixels are (DEV, FEW and SIZE at the top of the file).
"""
import argparse
import subprocess
import sys
from pathlib import Path

import numpy as np

sys.dont_write_bytecode = True

MIN_S = 0.5
LEAD_S = 0.75
SLACK_S = 0.15
FPS = 30
SIZE = (320, 180)
DEV = 8
FEW = 6


def flat_frames(path, fps=FPS, size=SIZE, dev=DEV, few=FEW):
    """One bool per sampled frame of `path`: True when the frame shows no picture."""
    w, h = size
    cmd = [
        "ffmpeg", "-v", "error", "-i", str(path), "-an",
        "-vf", f"fps={fps},scale={w}:{h}:flags=area,format=gray",
        "-f", "rawvideo", "-",
    ]
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE)
    n = w * h
    flat = []
    while True:
        buf = proc.stdout.read(n)
        if len(buf) < n:
            break
        px = np.frombuffer(buf, dtype=np.uint8)
        mode = int(np.bincount(px, minlength=256).argmax())
        picture = int(np.count_nonzero(np.abs(px.astype(np.int16) - mode) > dev))
        flat.append(picture <= few)
    proc.stdout.close()
    if proc.wait() != 0:
        raise RuntimeError(f"ffmpeg could not read {path}")
    return flat


def stretches(flat, fps=FPS):
    """The runs of True in `flat` as (start_s, end_s, kind); kind is opening, inside, closing or whole."""
    out = []
    i, n = 0, len(flat)
    while i < n:
        if not flat[i]:
            i += 1
            continue
        j = i
        while j < n and flat[j]:
            j += 1
        if i == 0 and j == n:
            kind = "whole"
        elif i == 0:
            kind = "opening"
        elif j == n:
            kind = "closing"
        else:
            kind = "inside"
        out.append((i / fps, j / fps, kind))
        i = j
    return out


def first_line_s(path):
    """When the chapter's first line starts (seconds), from the `.timeline.tsv` beside `path`; None without one."""
    tl = Path(path).with_suffix(".timeline.tsv")
    if not tl.is_file():
        return None
    fps, frames = None, []
    for row in tl.read_text(encoding="utf-8", errors="replace").splitlines():
        cells = row.split("\t")
        if cells[0] == "fps" and len(cells) > 1:
            fps = float(cells[1])
        elif cells[0] == "line" and len(cells) > 2:
            frames.append(float(cells[2]))
    return min(frames) / fps if fps and frames else None


def flagged(found, min_s=MIN_S, lead_s=LEAD_S, first_line=None, slack_s=SLACK_S):
    """The stretches worth a flag: longer than `min_s`; an opening is excused until the first line starts
    (`first_line`, plus `slack_s`), or up to `lead_s` when that is not known."""
    keep = []
    for start, end, kind in found:
        length = end - start
        if kind == "opening":
            excused = lead_s if first_line is None else first_line + slack_s
            if end > excused + 1e-9:
                keep.append((start, end, kind))
        elif length > min_s + 1e-9:
            keep.append((start, end, kind))
    return keep


def report(path, fps=FPS, min_s=MIN_S, lead_s=LEAD_S, verbose=False, slack_s=SLACK_S):
    """(lines, number of flags) for one chapter file."""
    flat = flat_frames(path, fps)
    found = stretches(flat, fps)
    name = Path(path).name
    first = first_line_s(path)
    lines = []
    flags = flagged(found, min_s, lead_s, first, slack_s)
    for start, end, kind in flags:
        note = f"; its first line starts at {first:.2f} s" if kind == "opening" and first is not None else ""
        lines.append(f"FLAG {name}: no picture {start:.2f}-{end:.2f} s ({end - start:.2f} s, {kind}{note})")
    if not flags:
        lines.append(f"PASS {name}: no stretch over {min_s} s without a picture (opening excused until the first line)")
    if verbose:
        lead = next((e - s for s, e, k in found if k == "opening"), 0.0)
        tail = next((e - s for s, e, k in found if k == "closing"), 0.0)
        said = "no timeline" if first is None else f"first line at {first:.2f} s"
        lines.append(f"INFO {name}: {len(flat) / fps:.1f} s, opening lead {lead:.2f} s ({said}), closing {tail:.2f} s")
    return lines, len(flags)


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
    ap.add_argument("--min-s", type=float, default=MIN_S, help="the shortest stretch to list (0.5)")
    ap.add_argument("--lead-s", type=float, default=LEAD_S, help="the opening excused when there is no timeline (0.75)")
    ap.add_argument("--slack-s", type=float, default=SLACK_S, help="after the first line starts, for the first mark to rise (0.15)")
    ap.add_argument("--fps", type=int, default=FPS, help="frames read a second (30)")
    ap.add_argument("--gate", action="store_true", help="exit 1 when anything is flagged")
    ap.add_argument("-v", "--verbose", action="store_true", help="also print each file's lead and tail")
    args = ap.parse_args(argv)
    files = files_of(args.inputs)
    if not files:
        print("blank_stretches: no MP4 found", file=sys.stderr)
        return 2
    total = 0
    for f in files:
        lines, n = report(f, args.fps, args.min_s, args.lead_s, args.verbose, args.slack_s)
        total += n
        print("\n".join(lines))
    print(f"ADVISORY blank stretches: {total} flagged in {len(files)} chapter file(s)")
    return 1 if args.gate and total else 0


if __name__ == "__main__":
    sys.exit(main())
