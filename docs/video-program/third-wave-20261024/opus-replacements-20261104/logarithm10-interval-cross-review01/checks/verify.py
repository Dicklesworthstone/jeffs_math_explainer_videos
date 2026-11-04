"""Checks for root's logarithm10-interval-repair01. Nothing here builds or renders.

Run from the repo root:  /usr/bin/python3 -I -B <this file>

TEXT    the patch, its effect on source, the cue checker on a temporary copy;
MODEL   a mirror of the kit's scratch timing and until_phrase's frame arithmetic;
PIXELS  the BASELINE render, a few dozen frames decoded with ffmpeg into a temp
        folder. No candidate picture exists, so these show when today's swap
        runs, not what the candidate looks like.
Writes verify.out and ../coverage/frame-series.tsv. Exit 1 on any failed check.
"""

import hashlib
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
O = pathlib.Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
C = O / "logarithm10-interval-repair01"
CRATE = pathlib.Path("explainer_corpus/logarithm_mysteries")
REL = "explainer_corpus/logarithm_mysteries/src/chapters/doing_nothing.rs"
CANON = pathlib.Path(REL)
BASE = C / "before-doing_nothing.rs"
CAND = C / "doing_nothing.rs"
MP4 = pathlib.Path(
    "narration_lab/third-wave-20261024/root/five-film-release-20261027/"
    "logarithm-combined-repair-preview01/render/10_doing_nothing.mp4"
)
FRAMES = pathlib.Path(
    "narration_lab/third-wave-20261024/root/five-film-release-20261027/"
    "logarithm-combined-repair-frames01/10_doing_nothing"
)
BASE_SHA = "535e985522f976fe83ed17a36f5ddd9a2fb29467225162aeb9f119ef13398a43"
CAND_SHA = "eeee764f0fe5fd3cd45263ada9a87f22b3e05a48b120ca2ba4f862cdf7a27143"
CUE_START = 67.1
FPS = 30

out = []
failed = []


def check(kind, name, ok, detail=""):
    out.append(f"{'ok  ' if ok else 'FAIL'} {kind:6} {name}{': ' + str(detail) if detail != '' else ''}")
    if not ok:
        failed.append(name)


def note(kind, text):
    out.append(f"note {kind:6} {text}")


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


# ------------------------------------------------------------------- TEXT
check("TEXT", "canonical is still the patch's base", sha(CANON) == BASE_SHA)
check("TEXT", "root's before copy is canonical", sha(BASE) == BASE_SHA)
check("TEXT", "root's candidate is the hash in its receipt", sha(CAND) == CAND_SHA)

with tempfile.TemporaryDirectory() as tmp:
    work = pathlib.Path(tmp)
    target = work / REL
    target.parent.mkdir(parents=True)
    shutil.copyfile(BASE, target)
    done = subprocess.run(
        ["patch", "-p1", "-s", "-i", str((C / "interval.patch").resolve())], cwd=work, capture_output=True, text=True
    )
    same = target.read_bytes() == CAND.read_bytes()
check("TEXT", "interval.patch applied to the base gives the candidate", done.returncode == 0 and same)

fmt = subprocess.run(
    ["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", str(CAND)],
    capture_output=True,
    text=True,
)
check("TEXT", "rustfmt --check accepts the candidate", fmt.returncode == 0)

base_lines = BASE.read_text(encoding="utf-8").splitlines()
cand_lines = CAND.read_text(encoding="utf-8").splitlines()
check("TEXT", "line count unchanged", len(base_lines) == len(cand_lines), len(cand_lines))
differ = [i + 1 for i, (a, b) in enumerate(zip(base_lines, cand_lines)) if a != b]
check("TEXT", "only lines 556, 557 and 560 differ", differ == [556, 557, 560], differ)
for line_no, marker in (
    (558, 'until_phrase(&kit, stage, "10_4", "strictly between")'),
    (598, "kit.hold(stage)?;"),
    (604, "kit.finish(stage)"),
):
    check("TEXT", f"hook site {line_no} still holds {marker}", marker in cand_lines[line_no - 1])

CALL = re.compile(r'(kit\.say|kit\.until_sentence|until_phrase|kit\.hold)\([^)]*\)')


def hooks(lines):
    body = "\n".join(lines).split("#[cfg(test)]")[0]
    return CALL.findall(body), [m.group(0) for m in CALL.finditer(body)]


check("TEXT", "cue, sentence and phrase hooks identical in order", hooks(base_lines)[1] == hooks(cand_lines)[1], len(hooks(cand_lines)[1]))

scene = "\n".join(cand_lines[537:565])
at_hook = scene.index('until_phrase(&kit, stage, "10_4", "strictly between")')
at_swap = scene.index("swap(stage, &[closed_r], &[open_r, rule], 0.3)?;")
check("TEXT", "rule and open end are in one swap, after the phrase hook", at_hook < at_swap)
check("TEXT", "no other fade of rule remains in the scene", "\n".join(cand_lines[:640]).count("&[rule]") == 0)
uses = [i + 1 for i, l in enumerate(cand_lines[:640]) if re.search(r"\brule\b", l) and "//" not in l]
check("TEXT", "rule is made once and used once, never removed", uses == [538, 560], uses)
strict = [i + 1 for i, l in enumerate(cand_lines[:640]) if "−π <" in l and "//" not in l]
note("TEXT", f"strict inequality strings outside comments, lines {strict}: RULE_WORD and the closing SOURCE fine print")


PLAN = pathlib.Path(
    "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage05/bound-plan"
)
LINE_REF = re.compile(r'"file":\s*"doing_nothing\.rs",\s*"line":\s*(\d+)')
refs = []
bindings = {}
for path in sorted(PLAN.rglob("*")):
    if path.is_file() and path.suffix in (".json", ".sha256"):
        body = path.read_text(encoding="utf-8")
        refs += [int(x) for x in LINE_REF.findall(body)]
        if BASE_SHA in body:
            bindings[str(path.relative_to(PLAN))] = body.count(BASE_SHA)
check("TEXT", "control: root's plan does carry doing_nothing.rs line references", len(refs) > 0, f"{len(refs)} references")
check("TEXT", "none of them points at a changed line", not {556, 557, 560} & set(refs), f"nearest to the change: {sorted(x for x in set(refs) if 540 <= x <= 575)}")
note("TEXT", f"source hash bindings to rebind if root integrates into that plan: {bindings}")


def run_cues(chapter_file):
    with tempfile.TemporaryDirectory() as tmp:
        work = pathlib.Path(tmp)
        shutil.copytree(CRATE / "src" / "chapters", work / "src" / "chapters")
        (work / "docs").mkdir()
        shutil.copyfile(CRATE / "docs" / "check_cues.py", work / "docs" / "check_cues.py")
        shutil.copyfile(CRATE / "script.tsv", work / "script.tsv")
        shutil.copyfile(chapter_file, work / "src" / "chapters" / "doing_nothing.rs")
        return subprocess.run(["/usr/bin/python3", "-I", "-B", "docs/check_cues.py"], cwd=work, capture_output=True, text=True)


rb, rc = run_cues(BASE), run_cues(CAND)
check("TEXT", "check_cues passes on base and candidate copies", rb.returncode == 0 and rc.returncode == 0)
check("TEXT", "check_cues listing identical", rb.stdout == rc.stdout)

# ------------------------------------------------------------------ MODEL
script_row = next(
    r for r in (CRATE / "script.tsv").read_text(encoding="utf-8").splitlines() if r.startswith("10_4\t")
)
text = script_row.split("\t", 1)[1]
words = len(text.split())
speech = words / 2.6
parts = [p.strip() for p in re.split(r"(?<=[.?!]) ", text) if p.strip()]
total = sum(len(p) for p in parts)
starts = []
before = 0
for p in parts:
    starts.append(speech * before / total)
    before += len(p)
offsets = [0] + [i + 2 for i in range(len(text) - 1) if text[i] in ".?!" and text[i + 1] == " "]
byte = text.encode("utf-8").find(b"strictly between")
check("MODEL", "phrase is in sentence 0, at byte 39, as root's anchor file says", byte == 39 and offsets[1] > byte, f"sentence 1 at byte {offsets[1]}")
frac = byte / offsets[1]
offset = starts[1] * frac
anchor = CUE_START + offset
note("MODEL", f"scratch: phrase {offset:.3f} s after the cue starts, scene time {anchor:.3f}; sentence 1 at {CUE_START + starts[1]:.3f}")


def frames_covering(seconds, fps):
    return max(math.ceil(seconds * fps - 1e-6), 0)


def landing(cue_frame, fade_s, offset_s, fps):
    """The frame the swap starts on: cue start, then an optional fade, then until_phrase."""
    now = cue_frame + round(fade_s * fps)
    at = cue_frame / fps + offset_s
    if at > now / fps:
        now += frames_covering(at - now / fps, fps)
    return now


cue_frame = round(CUE_START * FPS)
predicted = landing(cue_frame, 0.0, offset, FPS)
check("MODEL", "scratch: swap starts on the same frame with or without the 0.4 s fade", landing(cue_frame, 0.4, offset, FPS) == predicted, f"frame {predicted}, {predicted / FPS:.3f} s")
mismatches = 0
tried = 0
for fps in (30, 60):
    start = round(CUE_START * fps)
    for rate in [x / 100 for x in range(40, 301, 5)]:
        tried += 1
        if landing(start, 0.4, offset / rate, fps) != landing(start, 0.0, offset / rate, fps):
            mismatches += 1
check("MODEL", "same landing frame for voices 0.4x to 3x the scratch rate, at 30 and 60 fps", mismatches == 0, f"{tried} cases")
check("MODEL", "control: a phrase inside the old 0.4 s fade would land earlier now", landing(cue_frame, 0.0, 0.2, FPS) < landing(cue_frame, 0.4, 0.2, FPS))
after = 0.3 + 0.4
headroom = (starts[1] - offset) - after
note("MODEL", f"unchanged by the candidate: swap 0.3 s + crosses 0.4 s after the phrase; {headroom:.2f} s spare before sentence 1 at scratch rate, gone only at a voice {(starts[1] - offset) / after:.1f}x faster")
note("MODEL", "real Oliver timing for this phrase is unmeasured (root's phrase-timings-unmeasured-template.json)")
note("MODEL", "without a narrator until_phrase waits for nothing, so a silent run is 0.4 s shorter; the previews and final use a narrator")

# ----------------------------------------------------------------- PIXELS


def read_png(path):
    data = pathlib.Path(path).read_bytes()
    pos = 8
    idat = bytearray()
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


def mean(frame, x0, y0, x1, y1, channel):
    _, _, bpp, rows = frame
    values = [rows[y][x * bpp + channel] for y in range(y0, y1 + 1) for x in range(x0, x1 + 1)]
    return sum(values) / len(values)


def smooth(t):
    return t * t * t * (t * (6 * t - 15) + 10)


# π endpoint at scene (4.95, 1.55): px (1085.4, 220.5). Above the 6 pt window line,
# inside the dot: 216. The ring's stroke crosses the column at about 212.
DOT = (1084, 215, 1086, 217)
RING = (1084, 212, 1086, 213)
# "keep −π < turning < π": px 896..1078 on row 283
RULE = (896, 276, 1078, 291)


def decode(start_s, count, tmp, tag):
    subprocess.run(
        ["ffmpeg", "-nostdin", "-v", "error", "-ss", str(start_s), "-i", str(MP4), "-frames:v", str(count), "-pix_fmt", "rgb24", f"{tmp}/{tag}-%02d.png"],
        check=True,
    )
    first = round(start_s * FPS)
    return {first + k: pathlib.Path(tmp) / f"{tag}-{k + 1:02d}.png" for k in range(count)}


rows_tsv = ["frame\ttime_s\tsha256\tdot_green\tring_red\trule_green"]
with tempfile.TemporaryDirectory() as tmp:
    series = {}
    series.update(decode(67.0, 20, tmp, "a"))
    series.update(decode(69.5, 24, tmp, "b"))
    metrics = {}
    for frame_no, path in sorted(series.items()):
        f = read_png(path)
        metrics[frame_no] = (mean(f, *DOT, 1), mean(f, *RING, 0), mean(f, *RULE, 1))
        rows_tsv.append(
            f"{frame_no}\t{frame_no / FPS:.3f}\t{sha(path)}\t"
            + "\t".join(f"{v:.1f}" for v in metrics[frame_no])
        )
    # alignment: decoded frames against root's sampled frames at the same times
    for frame_no, sampled in ((2085, "frame-0159.png"), (2100, "frame-0160.png")):
        a = read_png(series[frame_no])
        b = read_png(FRAMES / sampled)
        diff = 0
        for y in range(205, 236):
            for x in range(1060, 1110):
                diff += abs(a[3][y][3 * x + 1] - b[3][y][b[2] * x + 1])
        diff /= 31 * 50
        check("PIXELS", f"decoded frame {frame_no} matches root's {sampled} at the π end", diff < 2.0, f"mean abs green difference {diff:.2f}")

(ROOT / "coverage").mkdir(exist_ok=True)
(ROOT / "coverage" / "frame-series.tsv").write_text("\n".join(rows_tsv) + "\n", encoding="utf-8")

closed = metrics[2085][0]
opened = metrics[2108][0]
check("PIXELS", "control: the dot's inside is green while closed and dark once open", closed > opened + 60, f"{closed:.0f} then {opened:.0f}")
share = {n: (metrics[n][0] - opened) / (closed - opened) for n in range(2085, 2109)}
first_down = next(n for n in sorted(share) if share[n] < 0.97)
last_down = next(n for n in sorted(share) if share[n] < 0.03)
check("PIXELS", "today's swap starts within a frame or two of the modelled phrase", abs(first_down - predicted) <= 2, f"model frame {predicted}, first visible change {first_down}")
check("PIXELS", "today's swap runs about 9 frames", 6 <= last_down - first_down <= 10, f"{first_down}..{last_down}")
note("PIXELS", "closed dot's visible share by frame: " + ", ".join(f"{n}:{share[n]:.2f}" for n in range(predicted - 1, predicted + 12)))
note("PIXELS", "smooth over 9 frames, for comparison: " + ", ".join(f"{1 - smooth(k / 9):.2f}" for k in range(0, 10)))
rule_bg = metrics[2010][2]
rule_full = metrics[2029][2]
check("PIXELS", "control: the rule's box is empty before the cue and full by 0.6 s in", rule_full > rule_bg + 3, f"{rule_bg:.1f} then {rule_full:.1f}")
rule_share = {n: (metrics[n][2] - rule_bg) / (rule_full - rule_bg) for n in range(2010, 2030)}
first_rule = next(n for n in sorted(rule_share) if rule_share[n] > 0.03)
full_rule = next(n for n in sorted(rule_share) if rule_share[n] > 0.97)
note("PIXELS", "baseline rule's visible share by frame: " + ", ".join(f"{n}:{rule_share[n]:.2f}" for n in range(2011, 2028)))
span = (first_down - first_rule) / FPS
check("PIXELS", "baseline contradiction measured: strict rule visible while π is still closed", 2.0 < span < 3.2, f"rule first visible frame {first_rule} ({first_rule / FPS:.2f} s), full {full_rule}, π starts to open frame {first_down} ({first_down / FPS:.2f} s): {span:.2f} s")

report = "\n".join(out) + f"\n\n{len(failed)} failed\n"
(HERE / "verify.out").write_text(report, encoding="utf-8")
print(report)
raise SystemExit(1 if failed else 0)
