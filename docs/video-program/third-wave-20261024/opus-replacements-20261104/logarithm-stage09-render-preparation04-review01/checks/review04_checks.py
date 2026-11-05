#!/usr/bin/env python3
"""Independent checks of LavenderPike's logarithm-stage09-render-preparation04 (SHA256SUMS 34b04c88), Mails 4552/4615.

    python3 -I -B P/logarithm-stage09-render-preparation04-review01/checks/review04_checks.py SCRATCH --w W
        run from the repository root; SCRATCH must be a new or empty absolute folder outside the repository;
        W is root's native folder holding logarithm-stage09-workspace01

Reads only. The real files are the two bundles, the Nov 2 render work (7 captures), root's series 02 render work and
jobs (timeline02, score02, renders 00-10, all written by version 4) and their job scripts. Planted WAVs are written
only under SCRATCH. No job, Cargo, kit or guard is run; nothing is heard or viewed. Exit 0 when every check passes.
"""
import argparse
import datetime
import difflib
import hashlib
import json
import re
import struct
import sys
import wave
from fractions import Fraction
from pathlib import Path

sys.dont_write_bytecode = True
REPO = Path.cwd()
P = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104"
V3_DIR, V4_DIR = P / "logarithm-stage09-render-preparation03", P / "logarithm-stage09-render-preparation04"
PINS = {"v3 SHA256SUMS": (V3_DIR / "SHA256SUMS", "aa645803a04d749855e9c7596731eb12f03d7a112c231eb2540c553eb8ee0096"),
        "v3 helper": (V3_DIR / "prepare_stage09_render.py",
                      "6e5a602dc338b18bb417a0a5345558a46d3f0cf9766b6d6a773ce4ae05e2bebb"),
        "v4 SHA256SUMS": (V4_DIR / "SHA256SUMS", "34b04c880117b80344686197688b2591b5b78d7e79f1e1294de653247b7fd409"),
        "v4 helper": (V4_DIR / "prepare_stage09_render.py",
                      "debd6f757ba571e71ea1a4a306f81fd12cf00417dee0a9e627834c8a9ed9ee73")}
LANE = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
NOV2 = LANE / "logarithm-machine-render-work01"
WORK2 = LANE / "logarithm-stage09-render-work02"
STAGE = "logarithm-stage09"
results = []


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def check(label, ok, detail=""):
    results.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'} {label}" + (f"\n     {detail}" if detail else ""))


def show(path):
    path = Path(path).resolve()
    for root, label in ((REPO.resolve(), ""), (W.resolve(), "W/"), (SCR.resolve(), "SCRATCH/")):
        if path.is_relative_to(root):
            return label + str(path.relative_to(root))
    return path.name


def template(helper):
    """The JOB_TEMPLATE text of a helper, and its functions by name."""
    text = helper.read_text(encoding="utf-8")
    body = text.split("JOB_TEMPLATE = r'''", 1)[1].split("\n'''", 1)[0]
    functions = {m.group(1): m.group(0) for m in re.finditer(r"^def (\w+)\(.*?(?=^\S)", body, re.M | re.S)}
    return body, functions


def riff(data):
    """An independent chunk walk: RIFF size, chunk list, fmt fields and the data chunk's declared and present bytes."""
    out = {"riff_ok": data[:4] == b"RIFF" and data[8:12] == b"WAVE"
           and struct.unpack_from("<I", data, 4)[0] + 8 == len(data), "chunks": []}
    at = 12
    while at + 8 <= len(data):
        cid, size = data[at:at + 4].decode("latin-1"), struct.unpack_from("<I", data, at + 4)[0]
        out["chunks"].append(cid)
        if cid == "fmt ":
            out["tag"], out["channels"], out["rate"], _, out["block"], out["bits"] = struct.unpack_from(
                "<HHIIHH", data, at + 8)
        if cid == "data":
            out["declared"], out["present"] = size, min(size, len(data) - at - 8)
        at += 8 + size + (size & 1)
    return out


def chapter_frames(timeline):
    table = dict(line.split("\t")[:2] for line in Path(timeline).read_text().splitlines()
                 if line and not line.startswith("#") and line.split("\t")[0] in ("fps", "frames"))
    return int(table["frames"]), int(table["fps"])


PCM_GUID = b"\x01\x00\x00\x00\x00\x00\x10\x00\x80\x00\x00\xaa\x00\x38\x9b\x71"


def build(pcm, channels=2, rate=48000, bits=16, extensible=False, trailing=b""):
    block = channels * bits // 8
    if extensible:
        fmt = struct.pack("<HHIIHHHHI", 0xFFFE, channels, rate, rate * block, block, bits, 22, bits,
                          3 if channels == 2 else 4) + PCM_GUID
    else:
        fmt = struct.pack("<HHIIHH", 1, channels, rate, rate * block, block, bits)
    body = b"WAVE" + b"fmt " + struct.pack("<I", len(fmt)) + fmt + b"data" + struct.pack("<I", len(pcm)) + pcm
    body += b"\0" * (len(pcm) & 1) + trailing
    return b"RIFF" + struct.pack("<I", len(body)) + body


def snapshot():
    files = [*V3_DIR.rglob("*"), *V4_DIR.rglob("*"), *NOV2.rglob("*.wav"), *NOV2.glob("*.timeline.tsv"),
             *WORK2.rglob("*.wav"), *WORK2.glob("*.timeline.tsv"), *SERIES2_FILES]
    return {show(p): (p.stat().st_size, p.stat().st_mtime_ns, sha(p)) for p in sorted(set(files)) if p.is_file()}


parser = argparse.ArgumentParser()
parser.add_argument("scratch")
parser.add_argument("--w", required=True)
args = parser.parse_args()
SCR, W = Path(args.scratch), Path(args.w)
if not SCR.is_absolute() or (SCR.exists() and any(SCR.iterdir())) or SCR.resolve().is_relative_to(REPO.resolve()):
    sys.exit("SCRATCH must be a new or empty absolute folder outside the repository")
if not (REPO / "tools/coherent_narration.py").is_file():
    sys.exit("run from the repository root")
SCR.mkdir(parents=True, exist_ok=True)
JOBS = W / f"{STAGE}-workspace01/jobs"
SERIES2 = [f"{STAGE}-timeline02", f"{STAGE}-score02",
           *sorted(p.name for p in LANE.glob(f"{STAGE}-render-[0-9][0-9]_*-02") if p.is_dir())]
SERIES2_FILES = [f for job in SERIES2 for f in (LANE / job).iterdir() if f.is_file()]
SERIES2_FILES += [JOBS / f"{job}{end}" for job in SERIES2 for end in (".py", "-inputs.json")]
before = snapshot()

# ----------------------------------------------------------------------------- D: the delta

for label, (path, pin) in PINS.items():
    check(f"D0 {label} is {pin[:8]}", sha(path) == pin)
for folder in (V3_DIR, V4_DIR):
    rows = [line.split("  ", 1) for line in (folder / "SHA256SUMS").read_text().splitlines() if line]
    check(f"D0 {folder.name}: every SHA256SUMS row verifies",
          all(sha(folder / name) == digest for digest, name in rows), f"{len(rows)} rows")
old = (V3_DIR / "prepare_stage09_render.py").read_text(encoding="utf-8").splitlines()
new = (V4_DIR / "prepare_stage09_render.py").read_text(encoding="utf-8").splitlines()
hunks = [h for h in difflib.SequenceMatcher(None, old, new, autojunk=False).get_opcodes() if h[0] != "equal"]
mine = sorted([("-", line) for tag, a, b, c, d in hunks for line in old[a:b]]
              + [("+", line) for tag, a, b, c, d in hunks for line in new[c:d]])
theirs = sorted((line[0], line[1:]) for line in (V4_DIR / "docs/v3-to-v4.diff").read_text().splitlines()
                if line[:1] in "+-" and not line.startswith(("+++", "---")))
check("D1 my v3->v4 diff has the same changed lines as Lavender's docs/v3-to-v4.diff",
      mine == theirs, f"{len(hunks)} changed regions at v3 lines {', '.join(str(h[1] + 1) for h in hunks)}; "
      f"-{sum(1 for s, _ in mine if s == '-')}/+{sum(1 for s, _ in mine if s == '+')} lines")
body3, fn3 = template(V3_DIR / "prepare_stage09_render.py")
body4, fn4 = template(V4_DIR / "prepare_stage09_render.py")
check("D2 template functions: version 4 adds frame_to_sample and incomplete; only timeline and render change",
      set(fn4) - set(fn3) == {"frame_to_sample", "incomplete"} and not set(fn3) - set(fn4)
      and sorted(n for n in fn3 if fn3[n] != fn4[n]) == ["render", "timeline"],
      f"{len(fn4)} functions; changed: {sorted(n for n in fn3 if fn3[n] != fn4[n])}")
runs = {v: {n: re.findall(r"run\(CONFIG\[\"(\w+)\"\], (\d+)\)", f) for n, f in fns.items()}
        for v, fns in (("v3", fn3), ("v4", fn4))}
runs = {v: {n: r for n, r in found.items() if r} for v, found in runs.items()}
changed = {n: (runs["v3"].get(n), runs["v4"].get(n)) for n in set(runs["v3"]) | set(runs["v4"])
           if runs["v3"].get(n) != runs["v4"].get(n)}
check("D3 the only timeout that changes is the timeline's kit run, 2400 -> 7200 s",
      changed == {"timeline": ([("verify_argv", "900"), ("argv", "2400")], [("verify_argv", "900"), ("argv", "7200")])},
      "; ".join(f"{n} {', '.join(f'{a} {s}' for a, s in r)}" for n, r in runs["v4"].items()))

# ----------------------------------------------------------------------------- F: the length rule

space = {"pathlib": __import__("pathlib"), "wave": wave}
for name in ("capture", "frame_to_sample", "incomplete"):
    exec(fn4[name], space)
f2s, incomplete, capture = space["frame_to_sample"], space["incomplete"], space["capture"]
cases = [(f, fps) for fps in (60, 30, 24, 25, 48, 50, 59, 7) for f in range(0, 120001)]
check("F1 frame_to_sample equals exact round-half-away of frames*48000/fps (units.rs:41) on 960,008 cases",
      all(f2s(f, fps) == int(Fraction(f * 48000, fps) + Fraction(1, 2)) for f, fps in cases)
      and (f2s(1, 60), f2s(60, 60)) == (800, 48000))

# ----------------------------------------------------------------------------- C: real captures, read only

lines = []
ok = True
for work, label in ((NOV2, "Nov 2"), (WORK2, "series 02")):
    for timeline in sorted(work.glob("*.timeline.tsv")):
        chapter = timeline.name.split(".")[0]
        track = capture(work, chapter)
        frames, fps = chapter_frames(timeline)
        want = f2s(frames, fps)
        r = riff(track.read_bytes())
        raw = (r["riff_ok"], r["tag"], r["channels"], r["rate"], r["bits"], r["declared"], r["present"])
        good = raw == (True, 1, 2, 48000, 16, 4 * want, 4 * want) and r["chunks"][-1] == "data" \
            and incomplete(track, frames, fps) is None
        if label == "series 02":
            job = LANE / f"{STAGE}-render-{chapter}-02"
            s = json.loads((job / "render-summary.json").read_text())["soundtrack"]
            receipt = json.loads((job / "resource-receipt.json").read_text())
            good &= (s["path"] == str(track) and s["sha256"] == sha(track) and s["captures"] == 1
                     and receipt.get("exit") == 0 and receipt.get("inputs_sha256") == receipt.get("final_inputs_sha256"))
        ok &= good
        lines.append(f"{label} {chapter} {frames} frames -> {want} samples, {track.name}, tag {r['tag']}, "
                     f"chunks {'/'.join(r['chunks'])}{'' if good else '  <-- FAIL'}")
check("C1 every real capture (7 Nov 2, 11 series 02) is PCM tag 1, 48 kHz s16 stereo, with exactly frame_to_sample "
      "frames declared and present, and passes version 4's render-job test; each series 02 render-summary names it "
      "at today's hash", ok and len(lines) == 18, "\n     ".join(lines))

src = capture(WORK2, "02_slide_rule")
frames, fps = chapter_frames(WORK2 / "02_slide_rule.timeline.tsv")
data = src.read_bytes()
r = riff(data)
head = len(data) - r["declared"]
pcm = data[head:]
left = b"".join(pcm[k:k + 2] for k in range(0, len(pcm), 4))
plants = {"exact copy": (data, True),
          "same PCM, WAVE_FORMAT_EXTENSIBLE header": (build(pcm, extensible=True), True),
          "a LIST chunk after the data": (build(pcm, trailing=b"LIST" + struct.pack("<I", 4) + b"INFO"), True),
          "one frame cut from the end": (data[:-4], False),
          "half-written copy (header for the whole file)": (data[:len(data) // 2], False),
          "header only": (data[:head], False),
          "one frame more": (build(pcm + b"\0" * 4), False),
          "mono, same length": (build(left, channels=1), False),
          "not a WAV": (b"\0" * 64, False)}
lines, ok = [], True
for k, (label, (blob, accept)) in enumerate(plants.items()):
    path = SCR / f"plant{k:02d}.wav"
    path.write_bytes(blob)
    why = incomplete(path, frames, fps)
    ok &= (why is None) == accept
    lines.append(f"{label}: {'accepted' if why is None else 'refused: ' + why[:90]}"
                 f"{'' if (why is None) == accept else '  <-- unexpected'}")
check(f"C2 planted captures from series 02's 02_slide_rule ({frames} frames) under Python "
      f"{sys.version_info[0]}.{sys.version_info[1]}", ok, "\n     ".join(lines))

# ----------------------------------------------------------------------------- J: the real series 02 jobs

wrote = [job for job in SERIES2 if PINS["v4 helper"][1] in (JOBS / f"{job}.py").read_text(encoding="utf-8").split("\n")[1]
         and any(row.get("sha256") == PINS["v4 helper"][1]
                 for row in json.loads((JOBS / f"{job}-inputs.json").read_text())["files"])]
body_ok = all(fn4["incomplete"] in (JOBS / f"{job}.py").read_text(encoding="utf-8") for job in SERIES2) \
    and 'run(CONFIG["argv"], 7200)' in (JOBS / f"{STAGE}-timeline02.py").read_text(encoding="utf-8")
check("J1 every series 02 job script names and binds helper debd6f75 and carries version 4's capture test",
      wrote == SERIES2 and body_ok, f"{len(wrote)}/{len(SERIES2)}: {', '.join(j.replace(STAGE + '-', '') for j in SERIES2)}")
writers = {}
for script in sorted(JOBS.glob(f"{STAGE}-*.py")):
    named = re.search(r"\(sha256 ([0-9a-f]{64})\)", script.read_text(encoding="utf-8").split("\n")[1] + " ")
    writers.setdefault(named.group(1)[:8] if named else "other", []).append(script.stem.replace(STAGE + "-", ""))
check("J2 no job script in the workspace was written by version 3 (6e5a602d), so version 4 accepting only its own "
      "and version 2's jobs leaves nothing unverifiable",
      "6e5a602d" not in writers, "; ".join(f"{k}: {len(v)}" for k, v in sorted(writers.items())))


def seconds(job):
    r = json.loads((LANE / job / "resource-receipt.json").read_text())
    return (datetime.datetime.fromisoformat(r["finished"]) - datetime.datetime.fromisoformat(r["started"])).total_seconds()


log = (LANE / f"{STAGE}-timeline02/command.log").read_text()
total = re.search(r"^total\s+(\d+) frames\s+([\d.]+)s", log, re.M)
rates = [(m.group(1), float(m.group(4))) for m in re.finditer(r"^(\d\d_\w+)\s+(\d+) frames\s+([\d.]+)s\s+([\d.]+) fps",
                                                               log, re.M)]
slow = min(rates, key=lambda x: x[1])
renders = [seconds(j) for j in SERIES2 if "-render-" in j]
frames_all = int(total.group(1))
print(f"NOTE E1 timeline02: job {seconds(f'{STAGE}-timeline02'):.0f} s, kit {float(total.group(2)):.1f} s for "
      f"{frames_all} frames (margin {7200 / float(total.group(2)):.2f}x under 7200); slowest chapter {slow[0]} at "
      f"{slow[1]} fps, which over the whole film would be {frames_all / slow[1]:.0f} s")
print(f"NOTE E1 score02 job {seconds(f'{STAGE}-score02'):.0f} s (timeout 7200); renders 00-10 jobs "
      f"{min(renders):.0f}-{max(renders):.0f} s (timeout 43200 each)")

after = snapshot()
check("S1 every real file read is unchanged (size, mtime, sha256)", before == after, f"{len(after)} files")
print(f"\n{sum(results)}/{len(results)} PASS")
sys.exit(0 if all(results) else 1)
