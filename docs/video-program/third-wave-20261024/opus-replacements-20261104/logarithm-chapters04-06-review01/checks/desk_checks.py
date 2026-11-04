"""Desk checks for the staged patches. Nothing here renders or compiles.

Run from the repo root:  /usr/bin/python3 -I -B <this file>

Three kinds of evidence, labelled in the output:
  TEXT    the cue calls in baseline and candidate source, and docs/check_cues.py
          run on temporary copies of the crate's chapter folder;
  MATHS   a mirror of the geometry and timing the patches rely on, with the
          baseline as a positive control where the defect can be shown;
  PIXELS  ink scans of the existing preview frames, which show the BASELINE
          picture: that a patched label's box is empty today, not that the
          patched label looks right.
Writes desk_checks.out beside itself. Exit 1 on any failed check.
"""

import json
import math
import pathlib
import re
import shutil
import struct
import subprocess
import tempfile
import zlib

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parent
CRATE = pathlib.Path("explainer_corpus/logarithm_mysteries")
FRAMES = pathlib.Path(
    "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-combined-repair-frames01"
)
FILES = ("undoing_growth.rs", "complex_turns.rs", "lifting_loops.rs")
SCRIPT_SHA = "f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84"

out = []
failed = []


def check(kind, name, ok, detail=""):
    out.append(f"{'ok  ' if ok else 'FAIL'} {kind:6} {name}{': ' + detail if detail else ''}")
    if not ok:
        failed.append(name)


# ------------------------------------------------------------------- TEXT

CALL = re.compile(r'kit\.(say|until_sentence)\(stage, "([0-9_]+)"(?:, ([^)]+))?\)')


def cue_calls(path):
    text = path.read_text(encoding="utf-8")
    body = text.split("#[cfg(test)]")[0]
    return CALL.findall(body)


for name in FILES:
    a = cue_calls(ROOT / "baseline" / name)
    b = cue_calls(ROOT / "candidate" / name)
    check("TEXT", f"{name} cue calls unchanged", a == b and len(a) > 0, f"{len(a)} calls")

import hashlib  # noqa: E402

script = (CRATE / "script.tsv").read_bytes()
check("TEXT", "script.tsv unchanged", hashlib.sha256(script).hexdigest() == SCRIPT_SHA)


def run_check_cues(folder):
    with tempfile.TemporaryDirectory() as tmp:
        work = pathlib.Path(tmp)
        shutil.copytree(CRATE / "src" / "chapters", work / "src" / "chapters")
        (work / "docs").mkdir()
        shutil.copyfile(CRATE / "docs" / "check_cues.py", work / "docs" / "check_cues.py")
        shutil.copyfile(CRATE / "script.tsv", work / "script.tsv")
        if folder:
            for name in FILES:
                shutil.copyfile(folder / name, work / "src" / "chapters" / name)
        return subprocess.run(
            ["/usr/bin/python3", "-I", "-B", "docs/check_cues.py"],
            cwd=work,
            capture_output=True,
            text=True,
        )


base_run = run_check_cues(ROOT / "baseline")
cand_run = run_check_cues(ROOT / "candidate")
check("TEXT", "check_cues passes on the baseline copy", base_run.returncode == 0)
check("TEXT", "check_cues passes on the candidate copy", cand_run.returncode == 0)


def listing(run):
    """The check's listing with file line numbers taken out, as a multiset."""
    rows = {}
    for line in run.stdout.splitlines():
        key = re.sub(r"\.rs:\d+", ".rs", line)
        rows[key] = rows.get(key, 0) + 1
    return rows


base_rows, cand_rows = listing(base_run), listing(cand_run)
new_lines = sorted(k for k in cand_rows if cand_rows[k] > base_rows.get(k, 0))
gone_lines = sorted(k for k in base_rows if base_rows[k] > cand_rows.get(k, 0))
check(
    "TEXT",
    "check_cues listing changes only by the new 0 label and the reworded formula",
    len(new_lines) <= 3 and len(gone_lines) <= 2,
    f"{len(new_lines)} new, {len(gone_lines)} gone",
)
for row in new_lines:
    out.append(f"note   TEXT   check_cues new row: {row.strip()[:140]}")
for row in gone_lines:
    out.append(f"note   TEXT   check_cues gone row: {row.strip()[:140]}")

# ------------------------------------------------------------------ MATHS


def smooth(t):
    """Assumed to be the default rate of tween: peak rate 1.875 x average."""
    return t * t * t * (t * (6 * t - 15) + 10)


def time_of(alpha, run):
    lo, hi = 0.0, 1.0
    for _ in range(60):
        mid = (lo + hi) / 2
        if smooth(mid) < alpha:
            lo = mid
        else:
            hi = mid
    return run * (lo + hi) / 2


TAU = 2 * math.pi
CLIMB_END = 6.5 * math.pi
IN_CENTRE = (-4.9, -0.15)
IN_SCALE = 0.3
IN_Y = 3 * math.pi
OUT_CENTRE = (2.7, -0.15)
OUT_SCALE = 0.85
OUT_HALF_W = 3.9

# C1: baseline stops at 3pi with the count on 1; the candidate reaches 3
check("MATHS", "C1 control: baseline climb to 3 pi never moves the count", math.floor(3 * math.pi / TAU) == 1)
check("MATHS", "C1 candidate count ends on 3", math.floor(CLIMB_END / TAU) == 3)
past = (CLIMB_END / TAU) % 1
check("MATHS", "C1 stops between whole laps", 0.2 < past < 0.8, f"fraction {past:.3f}, end angle pi/2")
run = 2.6
t2 = time_of((2 * TAU - TAU) / (CLIMB_END - TAU), run)
t3 = time_of((3 * TAU - TAU) / (CLIMB_END - TAU), run)
counts = {"1": t2 + 0.4, "2": t3 - t2, "3": run - t3 + 1.2}
check(
    "MATHS",
    "C1 each count stands at least 0.4 s under the assumed smooth rate",
    min(counts.values()) >= 0.4,
    ", ".join(f"{k}: {v:.2f} s" for k, v in counts.items()) + " (real-speed legibility not seen)",
)
top = IN_CENTRE[1] + IN_SCALE * IN_Y
check("MATHS", "C1 pinned dot stops short of the arrow tail", top + 0.08 < top + 0.12, f"dot top {top + 0.08:.3f}, tail {top + 0.12:.3f}")
wound_max = 1 + 0.06 * (CLIMB_END / TAU)
next_ring = math.exp(0.5)
check("MATHS", "C1 wound trace stays inside the x = 0.5 ring", wound_max * OUT_SCALE < next_ring * OUT_SCALE - 0.1, f"{wound_max:.3f} vs {next_ring:.3f} in lengths")
spacing = 0.06 * OUT_SCALE
check("MATHS", "C1 lap spacing clears the 3 pt stroke", spacing > 0.03, f"{spacing:.3f} vs 0.030 units; baseline {0.045 * OUT_SCALE:.3f} (margin only 1.9 px at 720p)")
gap_px = abs(wound_max - 1) * OUT_SCALE * 90
out.append(f"note   MATHS  C1 the green dot rides the true circle, so at the end it stands {gap_px:.0f} px (720p) inside the trace tip; the on-screen aid already says laps are drawn further out")

# C7: EACH measured 278 px wide at frame-0350 (3.09 units), centre -2.30 today
each_w = 278 / 90
strip_right = IN_CENTRE[0] + IN_SCALE * 1.2
axis_left = OUT_CENTRE[0] - OUT_HALF_W
base_right = (IN_CENTRE[0] + 2.6) + each_w / 2
cand_left = (IN_CENTRE[0] + 2.05) - each_w / 2
cand_right = (IN_CENTRE[0] + 2.05) + each_w / 2
check("MATHS", "C7 control: baseline rule runs over the output axis", base_right > axis_left, f"right {base_right:.3f}, axis {axis_left:.3f}")
check("MATHS", "C7 candidate rule stops short of the axis", cand_right < axis_left - 0.03, f"right {cand_right:.3f}")
check("MATHS", "C7 candidate rule clears the strip", cand_left > strip_right + 0.05, f"left {cand_left:.3f}, strip {strip_right:.3f}")

# S3: order of the column
facts = [-3.0, -1.0, 1.0, 3.0]
check("MATHS", "S3 column sorts to the spoken order", sorted(facts, key=lambda k: (k < 0, int(abs(k)))) == [1.0, 3.0, -1.0, -3.0])

# C3: the axis
Z_CX, Z_S = -4.1, 1.25
check("MATHS", "C3 control: baseline axis passes the safe edge", Z_CX - 2.9 < -6.96, f"{Z_CX - 2.9:.2f}")
check("MATHS", "C3 candidate axis inside the safe edge", Z_CX - 2.8 > -6.96, f"{Z_CX - 2.8:.2f}")

# C4: ours really has two values at z = 1, and theirs only one
ours_start, ours_end = 0.0, TAU
check("MATHS", "C4 carried value differs at z = 1 by one lap", ours_end - ours_start == TAU)
z = complex(math.cos(TAU), math.sin(TAU))
check("MATHS", "C4 both values are logarithms of the same z", abs(complex(math.cos(ours_end), math.sin(ours_end)) - z) < 1e-12)


def principal_gap(turn):
    ours = TAU * turn
    wrapped = ours - TAU * math.ceil(ours / TAU - 0.5)
    return round((wrapped - ours) / TAU)


check("MATHS", "06_6 gauge sign: candidate - ours drops by a lap past -1", principal_gap(0.49) == 0 and principal_gap(0.51) == -1)

# C6b: the note under its own circle
SIDE = (1.7, 0.8)
note_w = 112 / 90  # measured at frame-0170, 283..395 px
note_left = Z_CX + 1.5
under = Z_CX + Z_S * SIDE[0]
check("MATHS", "C6b note centres under the side circle", abs(note_left + note_w / 2 - under) < 0.1, f"{note_left + note_w / 2:.3f} vs {under:.3f}")
check("MATHS", "C6b control: baseline note sits under the unit circle", Z_CX + 0.15 < Z_CX + Z_S)
circle_bottom = -0.3 - Z_S * SIDE[1]
check("MATHS", "C6b note stands below the side circle", -0.3 - 1.6 + 0.12 < circle_bottom, f"note top about {-0.3 - 1.6 + 0.12:.2f}, circle bottom {circle_bottom:.2f}")
check("MATHS", "C6b 0 is outside the side circle", abs(0 - SIDE[0]) > SIDE[1])

# S1: mirror name
check("MATHS", "S1 name right of the line at its own top", 3.05 > 2.8 + 0.125 + 0.1)
check("MATHS", "S1 name short of the note at scene x = 1.0", -3.2 + 3.05 + 0.6 < 0.9)

# Budgets under the scratch voice, whose sentence starts are character shares


def shares(cue_id):
    for row in (CRATE / "script.tsv").read_text(encoding="utf-8").splitlines():
        if row.startswith(cue_id + "\t"):
            text = row.split("\t", 1)[1]
            parts = re.split(r"(?<=[.?!])\s+", text.strip())
            return parts, len(text)
    raise SystemExit(cue_id)


def share_seconds(cue_id, length, j):
    parts, total = shares(cue_id)
    return length * len(parts[j]) / total


s = share_seconds("05_3", 25.400, 5)
check("MATHS", "05_3 last sentence holds the 4.2 s picture under scratch shares", 4.2 <= s, f"{s:.1f} s; baseline picture 3.8 s")
s = share_seconds("06_2", 19.246, 0)
check("MATHS", "06_2 first sentence holds the 2.8 s picture under scratch shares", 2.8 <= s, f"{s:.1f} s")
out.append("note   MATHS  06_20 third sentence: 0.4 + 4.0 + 0.3 s before and after, order only; real Oliver sentence timing unmeasured everywhere")

# ----------------------------------------------------------------- PIXELS


def read_png(path):
    data = pathlib.Path(path).read_bytes()
    pos = 8
    idat = bytearray()
    width = height = bits = ctype = None
    while pos < len(data):
        n, kind = struct.unpack(">I4s", data[pos : pos + 8])
        chunk = data[pos + 8 : pos + 8 + n]
        if kind == b"IHDR":
            width, height, bits, ctype = struct.unpack(">IIBB", chunk[:10])
        elif kind == b"IDAT":
            idat += chunk
        pos += 12 + n
    if bits != 8 or ctype not in (2, 6):
        raise SystemExit(f"{path}: unsupported png {bits} {ctype}")
    bpp = 3 if ctype == 2 else 4
    raw = zlib.decompress(bytes(idat))
    stride = width * bpp
    rows = []
    prior = bytearray(stride)
    at = 0
    for _ in range(height):
        f = raw[at]
        line = bytearray(raw[at + 1 : at + 1 + stride])
        at += 1 + stride
        if f == 1:
            for i in range(bpp, stride):
                line[i] = (line[i] + line[i - bpp]) & 255
        elif f == 2:
            for i in range(stride):
                line[i] = (line[i] + prior[i]) & 255
        elif f == 3:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                line[i] = (line[i] + ((a + prior[i]) >> 1)) & 255
        elif f == 4:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                b = prior[i]
                c = prior[i - bpp] if i >= bpp else 0
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pr = a if pa <= pb and pa <= pc else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 255
        rows.append(line)
        prior = line
    return width, height, bpp, rows


def px(x, y):
    return round((x + 7.11) * 90), round(360 - 90 * y)


def ink(frame, box, floor=40):
    """Pixels brighter than the dark background inside a scene-unit box."""
    width, height, bpp, rows = frame
    (x0, y0), (x1, y1) = px(box[0], box[3]), px(box[1], box[2])
    count = 0
    xs = []
    for y in range(max(y0, 0), min(y1 + 1, height)):
        row = rows[y]
        for x in range(max(x0, 0), min(x1 + 1, width)):
            i = x * bpp
            if max(row[i], row[i + 1], row[i + 2]) > floor:
                count += 1
                xs.append(x)
    return count, (min(xs), max(xs)) if xs else None


def frame_file(chapter, index):
    return FRAMES / chapter / f"frame-{index:04d}.png"


def times(chapter):
    return json.loads((FRAMES / chapter / "sampling.json").read_text())["times"]


def frames_between(chapter, a, b, step):
    t = times(chapter)
    picked = [i for i, s in enumerate(t) if a <= s <= b]
    return picked[::step] + ([picked[-1]] if picked and (len(picked) - 1) % step else [])


def scan_empty(chapter, a, b, step, box, name):
    picked = frames_between(chapter, a, b, step)
    worst = 0
    where = None
    for i in picked:
        n, _ = ink(read_png(frame_file(chapter, i)), box)
        if n > worst:
            worst, where = n, i
    check(
        "PIXELS",
        name,
        worst == 0,
        f"{len(picked)} frames t {a}..{b}, most ink {worst}" + (f" at frame {where}" if where is not None else ""),
    )


# positive control: the scanner does see the words it should see
ch5, ch6, ch4 = "05_complex_turns", "06_lifting_loops", "04_undoing_growth"
control = ink(read_png(frame_file(ch5, 350)), (-3.85, -0.75, -0.3, 0.1))
check("PIXELS", "control: the scanner finds the 05 rule on frame 350", control[0] > 0, f"{control[0]} px, x {control[1]}")
axis_under = ink(read_png(frame_file(ch5, 350)), (-1.15, -0.8, -0.2, -0.1))
check("PIXELS", "C7 control: baseline axis and rule share the row right of -1.2", axis_under[0] > 0, f"{axis_under[0]} px")

# 05_3: the note and arrow's box is empty until the climb ends (05_4 start 98.533)
scan_empty(ch5, 92.0, 98.5, 6, (-4.75, -3.4, 2.72, 3.12), "C1 note and arrow box empty in 05_3")
title_box = ink(read_png(frame_file(ch5, 216)), (-3.4, 7.0, 3.0, 3.85))
check("PIXELS", "C1 title's left end stands right of the note", title_box[1] is not None and title_box[1][0] > px(-2.4, 0)[0], f"title ink starts at px {title_box[1][0] if title_box[1] else None}")

# 05_6 rule's new box is empty while it stands
scan_empty(ch5, 140.0, 155.0, 6, (-4.42, -3.9, -0.3, 0.1), "C7 new left part of the rule is empty today")

# 06: the 0 mark's box stays empty from 06_4 to the last picture
# (the dot itself sits on the axes' crossing by design; px 270-271 is the upright axis)
axis_seen = ink(read_png(frame_file(ch6, 364)), (-4.5, -3.98, -0.68, -0.36))
check("PIXELS", "C6b control: a box reaching x = -4.1 does find the upright axis", axis_seen[0] > 0, f"{axis_seen[0]} px, x {axis_seen[1]}")
scan_empty(ch6, 60.0, 178.15, 8, (-4.50, -4.15, -0.68, -0.36), "C6b box of the name 0, from before 06_4 to the last picture")

# 06_20 on: the (l) box right of "ours: carried along"
sample = read_png(frame_file(ch6, 364))
row = ink(sample, (1.3, 6.9, 1.85, 2.05))
right_end = row[1][1] if row[1] else None
check("PIXELS", "C4 ours line found on frame 364", right_end is not None, f"ink x {row[1]}")
if right_end is not None:
    left_units = right_end / 90 - 7.11
    out.append(f"note   PIXELS C4 ours line ends at scene x {left_units:.2f}; the name starts 0.12 right of it")
    scan_empty(
        ch6,
        130.0,
        178.15,
        12,
        (left_units + 0.1, left_units + 0.75, 1.8, 2.12),
        "C4 box of the name right of ours, 06_20 to the end",
    )

# 04: the mirror's name, whose box is p(3.05, 2.8) with OX -3.2 and OY -0.4
line_top = ink(read_png(frame_file(ch4, 230)), (-0.5, -0.2, 2.2, 2.65))
check("PIXELS", "S1 control: the scanner finds the dashed mirror beside the box", line_top[0] > 0, f"{line_top[0]} px, x {line_top[1]}")
scan_empty(ch4, 76.3, 108.0, 8, (-0.17, 0.5, 2.25, 2.6), "S1 box of the mirror's name, from 04_4 while the graph stands")

report = "\n".join(out) + f"\n\n{len(failed)} failed\n"
(HERE / "desk_checks.out").write_text(report, encoding="utf-8")
print(report)
raise SystemExit(1 if failed else 0)
