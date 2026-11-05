#!/usr/bin/env python3
"""Independent review checks of logarithm-stage09-render-preparation03 (RubyRaven, sealed aa645803), bead 87do.63,
root Mail 4467. LavenderPike, 2026-11-05.

    python3 -I -B P/logarithm-stage09-render03-review01/checks/review_checks.py SCRATCH --w W

SCRATCH must be a new or empty absolute folder outside the repository. W is root's geometry stage folder.

What this reads, and never writes:
- Ruby's sealed bundle;
- the kit sources that ws-build02 compiled (W/repo-view) and the repository's copies of them;
- the logarithm crate's main.rs;
- the Nov 2 logarithm render work, its chapter 04 render log and its master log;
- root's timeline01 job script and inputs file, but not its lane folder;
- WS/narration-binding01.json.

It writes only under SCRATCH, including what Ruby's own checks write under SCRATCH/render03-checks. It runs no Cargo,
kit, guard or native job. Printed paths are shortened to SCRATCH, W, REPO and ~.
"""
import argparse
import hashlib
import importlib.util
import json
import os
import re
import subprocess
import sys
import wave
from pathlib import Path

HERE = Path(__file__).resolve()
REPO = HERE.parents[6]
P = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104"
R3 = P / "logarithm-stage09-render-preparation03"
H3 = R3 / "prepare_stage09_render.py"
V02 = P / "logarithm-stage09-final-verification02/verify_stage09_final.py"
H3_SHA = "6e5a602dc338b18bb417a0a5345558a46d3f0cf9766b6d6a773ce4ae05e2bebb"
V02_SHA = "f266a2315e4a9af247c648da416881f3a2999e2682e9c611ff82ca3727b70b94"
SEALS = {"logarithm-stage09-render-preparation03": "aa645803a04d749855e9c7596731eb12f03d7a112c231eb2540c553eb8ee0096",
         "logarithm-stage09-render-preparation02": "352a81071f0b44134b163f91ecd9ed810022a8508630d09cea663f1d197ff2ed",
         "logarithm-stage09-final-verification02": "e8e19275a325a41861a2abfec0338cc0cd74fb4ee6f13616f28c5bd862180700"}

parser = argparse.ArgumentParser()
parser.add_argument("scratch")
parser.add_argument("--w", required=True)
ARGS = parser.parse_args()
SCR, W = Path(ARGS.scratch), Path(ARGS.w)
if not SCR.is_absolute() or (SCR.exists() and any(SCR.iterdir())) or SCR.resolve().is_relative_to(REPO.resolve()):
    sys.exit("SCRATCH must be a new or empty absolute folder outside the repository")
SCR.mkdir(parents=True, exist_ok=True)
SHORT = sorted({str(SCR.resolve()): "SCRATCH", str(SCR): "SCRATCH", str(W): "W", str(REPO): "REPO",
                str(Path.home()): "~"}.items(), key=lambda kv: -len(kv[0]))
RESULTS = []


def sha(path):
    with open(path, "rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def clean(text):
    for full, short in SHORT:
        text = text.replace(full, short)
    return text


def record(name, ok, detail):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'} {name}: {clean(detail)}", flush=True)


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def block(text, start, end):
    i = text.index(start)
    return text[i:text.index(end, i + len(start))]


def frame_to_sample(frames, fps, rate=48_000):
    """explainer_score/src/units.rs:41, transcribed: round half away from zero of frames * rate / fps."""
    return (2 * frames * rate + fps) // (2 * fps)


def complete(path, frames):
    """The rule proposed in L1: the capture is the s16 stereo 48 kHz WAV the master's null test needs, whole."""
    want = frame_to_sample(frames, 60)
    try:
        with wave.open(str(path)) as w:
            return ((w.getnchannels(), w.getsampwidth(), w.getframerate(), w.getnframes()) == (2, 2, 48_000, want)
                    and len(w.readframes(want)) == 4 * want)
    except (wave.Error, EOFError):
        return False


def run_h3(*args, ws, lane, target):
    p = subprocess.run([sys.executable, "-I", "-B", str(H3), *map(str, args), "--w", str(W), "--ws", str(ws),
                        "--receipts", str(lane), "--target-root", str(target)],
                       cwd=REPO, capture_output=True, text=True, timeout=900)
    return p.returncode, clean(p.stdout + p.stderr)


def tail(text, n=160):
    text = " ".join(text.split())
    return text if len(text) <= n else "..." + text[-n:]


if sha(H3) != H3_SHA or sha(V02) != V02_SHA:
    sys.exit("a pinned helper moved: render03 or final-verification02")
print(f"render03 helper sha256 {H3_SHA}; review checks sha256 {sha(HERE)}", flush=True)
V3 = load("prepare_stage09_render_v3", H3)
F = load("verify_stage09_final_v02", V02)
N = V3.N
REAL_LANE = REPO / N.RECEIPTS_REL
OLD = REAL_LANE / "logarithm-machine-render-work01"
REAL_WS = W / "logarithm-stage09-workspace01"
VIEW = W / "repo-view/crates/explainer_kit/src"
KIT = REPO / "crates/explainer_kit/src"
MAIN_RS = REAL_WS / "explainer_corpus/logarithm_mysteries/src/main.rs"
TL_SCRIPT = REAL_WS / "jobs/logarithm-stage09-timeline01.py"
TL_INPUTS = REAL_WS / "jobs/logarithm-stage09-timeline01-inputs.json"
BINDING01 = REAL_WS / "narration-binding01.json"
REAL_O = REPO / N.EL_REL / "production-audio-stage09-01/prepared01"
TEMPLATE_NS = {"pathlib": __import__("pathlib")}
exec(block(V3.JOB_TEMPLATE, "def capture(work, chapter):", "\n\n\ndef "), TEMPLATE_NS)
template_capture = TEMPLATE_NS["capture"]


def read_only_state():
    paths = [MAIN_RS, TL_SCRIPT, TL_INPUTS, BINDING01, REAL_WS / "logarithm-stage09-ws-build02-inputs.json",
             REAL_LANE / "logarithm-machine-master01/command.log",
             REAL_LANE / "logarithm-machine-render-04_principal_product-01/command.log"]
    paths += [VIEW / f for f in ("cli.rs", "encode.rs", "master.rs")] + [KIT / f for f in ("cli.rs", "encode.rs", "master.rs")]
    paths += sorted(p for p in (OLD / "soundtrack").rglob("*") if p.is_file())
    paths += sorted(p for p in OLD.iterdir() if p.is_file())
    for folder in SEALS:
        paths += sorted(p for p in (P / folder).rglob("*") if p.is_file() and "__pycache__" not in p.parts)
    return {str(p): (sha(p), os.stat(p).st_mtime_ns) for p in paths}


BEFORE = read_only_state()

# ---------------------------------------------------------------- R0 seals
seals = []
for folder, digest in SEALS.items():
    v = subprocess.run(["shasum", "-a", "256", "-c", "SHA256SUMS"], cwd=P / folder, capture_output=True, text=True)
    lines = v.stdout.strip().splitlines()
    seals.append((folder, sha(P / folder / "SHA256SUMS") == digest and v.returncode == 0, len(lines)))
record("R0 the reviewed bundle and its neighbours verify at their sealed hashes", all(ok for _, ok, _ in seals),
       "; ".join(f"{f.split('logarithm-stage09-')[1]} {n}/{n}" if ok else f"{f} BROKEN" for f, ok, n in seals))

# ---------------------------------------------------------------- K1 the kit's capture and master code
build_rows = {r["path"]: r["sha256"] for r in json.loads((REAL_WS / "logarithm-stage09-ws-build02-inputs.json").read_text())["files"]}
compiled = all(build_rows.get(str(p)) == sha(p) for p in [VIEW / "cli.rs", VIEW / "encode.rs", VIEW / "master.rs", MAIN_RS])
cli_v, cli_r, enc = (VIEW / "cli.rs").read_text(), (KIT / "cli.rs").read_text(), (VIEW / "encode.rs").read_text()
same_files = sha(VIEW / "encode.rs") == sha(KIT / "encode.rs") and sha(VIEW / "master.rs") == sha(KIT / "master.rs")


def capture_block(text):
    return block(text, "if let Some(crf) = args.crf {", "let rendered = match camera.camera_shot()")


def master_block(text):
    return block(text, "fn master_video(", "\n}\n")


same_blocks = capture_block(cli_v) == capture_block(cli_r) and master_block(cli_v) == master_block(cli_r)
attempt_loop = block(cli_v, "let (report, timeline) = loop {", "let secs = started.elapsed()")
per_attempt = all(s in attempt_loop for s in ("n.reset();", "set_aside(&capture)?;", "encode::delivery_options(",
                                              "capture_dir: (!profiling).then_some(capture),"))
cuelog_after = cli_v.index('("cuelog.tsv", n.cue_log())') > cli_v.index("let secs = started.elapsed()")
runner = block(enc, "impl ProcessRunner for QualityRunner {", "\n}\n")
before_job = (runner.count("self.capture(spec);") == 2
              and runner.index("self.capture(spec);") < runner.index("self.inner.start(")
              and runner.rindex("self.capture(spec);") < runner.index("self.inner.run("))
fresh_counter = "captured: AtomicUsize::new(0)," in block(enc, "fn new(quality: Quality) -> Self {", "\n    }\n")
warning_only = ('if let Err(e) = std::fs::copy(&wav, dir.join(format!("soundtrack_{n}.wav"))) {\n'
                '            eprintln!("soundtrack capture of {} failed: {e}", wav.display());\n        }') in enc
master_rule = ('.map(|n| dir.join(format!("soundtrack_{n}.wav")))\n            .take_while(|p| p.exists())\n'
               '            .last()') in master_block(cli_v)
main_text = MAIN_RS.read_text()
default_options = "explainer_kit::cli::run_with_contract(" in main_text and "run_with_contract_and_options" not in main_text
record("K1 the compiled kit captures and chooses as Ruby describes, and the logarithm crate keeps the capture folder",
       all((compiled, same_files, same_blocks, per_attempt, cuelog_after, before_job, fresh_counter, warning_only,
            master_rule, default_options)),
       f"ws-build02 bound these bytes {compiled}; encode.rs and master.rs equal the repository's {same_files}; "
       f"cli.rs capture block and master_video equal the repository's {same_blocks}; each EPERM attempt sets the "
       f"capture folder aside, builds a fresh runner and resets the narrator {per_attempt}; cuelog written after the "
       f"loop {cuelog_after}; copy before each ffmpeg job {before_job}; counter from 0 per runner {fresh_counter}; "
       f"a failed copy is only a warning {warning_only}; master takes the last contiguous soundtrack_<n> {master_rule}; "
       f"main.rs uses run_with_contract (no options hook) {default_options}")

# ---------------------------------------------------------------- K2 the rule on planted layouts
CH = "05_complex_turns"
cases = [("no folder", [], None), ("empty folder", ["mkdir"], None), ("0", ["0"], 0), ("0 1 2", ["0", "1", "2"], 2),
         ("0 1 3, a gap", ["0", "1", "3"], 1), ("1 2, no 0", ["1", "2"], None), ("0 beside a set-aside copy", ["0", "aside"], 0),
         ("0 and a dangling link as 1", ["0", "dangling1"], 0), ("0 and a link to a file as 1", ["0", "link1"], 1),
         ("0 and a directory as 1", ["0", "dir1"], 1), ("padded 00", ["00"], None)]
k2 = []
for number, (label, parts, want) in enumerate(cases):
    work = SCR / "k2" / f"case{number:02d}"
    folder = work / "soundtrack" / CH
    work.mkdir(parents=True)
    if parts:
        folder.mkdir(parents=True)
    for part in parts:
        if part.isdigit():
            (folder / f"soundtrack_{part}.wav").write_bytes(b"RIFF" + part.encode())
        elif part == "aside":
            aside = work / "soundtrack/previous" / f"{CH}.v1"
            aside.mkdir(parents=True)
            (aside / "soundtrack_0.wav").write_bytes(b"RIFF aside")
            (aside / "soundtrack_1.wav").write_bytes(b"RIFF aside")
        elif part == "dangling1":
            os.symlink(work / "nowhere.wav", folder / "soundtrack_1.wav")
        elif part == "link1":
            (work / "target.wav").write_bytes(b"RIFF target")
            os.symlink(work / "target.wav", folder / "soundtrack_1.wav")
        elif part == "dir1":
            (folder / "soundtrack_1.wav").mkdir()
    expected = None if want is None else folder / f"soundtrack_{want}.wav"
    got = (V3.kit_capture(work, CH), template_capture(work, CH))
    ok = got[0] == expected and got[1] == expected
    if label.startswith("0 and a directory"):
        try:
            V3.check_capture("planted", work, CH, {"path": str(expected), "sha256": "0" * 64})
            ok = False
        except V3.Refused:
            pass
        ok = ok and not got[1].is_file()  # the render job's `not track.is_file()` refuses it too
    k2.append((label, ok, "none" if expected is None else expected.name))
record("K2 the helper's and the job's rule give the kit's choice on 11 planted layouts (expected answers read from "
       "cli.rs:988-991: n from 0 while soundtrack_<n>.wav exists, links followed)", all(ok for _, ok, _ in k2),
       "; ".join(f"{label} -> {name}{'' if ok else ' WRONG'}" for label, ok, name in k2)
       + "; the directory case is then refused by check_capture and by the job")

# ---------------------------------------------------------------- K3 the real Nov 2 render and master
log04 = (REAL_LANE / "logarithm-machine-render-04_principal_product-01/command.log").read_text(errors="replace")
master_log = (REAL_LANE / "logarithm-machine-master01/command.log").read_text(errors="replace")
null_tested = {m.group(1): int(m.group(3)) for m in
               re.finditer(r"^(\S+)\s+replay = fmn's soundtrack, bit for bit \((\d+) cues, (\d+) samples\)$", master_log, re.M)}
old_chapters = sorted(p.name for p in (OLD / "soundtrack").iterdir() if p.is_dir() and p.name != "previous")
rows3 = []
for ch in old_chapters:
    chosen = OLD / "soundtrack" / ch / "soundtrack_0.wav"
    frames = int(V3.first(V3.timeline_rows(OLD / f"{ch}.timeline.tsv"), "frames"))
    rule_ok = V3.kit_capture(OLD, ch) == chosen == template_capture(OLD, ch)
    rows3.append((ch, rule_ok and complete(chosen, frames) and null_tested.get(ch) == frame_to_sample(frames, 60),
                  frame_to_sample(frames, 60)))
aside = OLD / "soundtrack/previous/04_principal_product.v1/soundtrack_0.wav"
chosen04 = OLD / "soundtrack/04_principal_product/soundtrack_0.wav"
t = {name: os.stat(p).st_mtime_ns for name, p in (("aside", aside), ("chosen", chosen04),
                                                   ("mp4", OLD / "04_principal_product.mp4"),
                                                   ("master", OLD / "logarithm_mysteries_master.mp4"))}
retry = ("04_principal_product: transient Darwin EPERM after ffmpeg exit (attempt 1); rendering again" in log04
         and "soundtrack/04_principal_product to " in log04 and "soundtrack/previous/04_principal_product.v1" in log04
         and "(attempt 2)" in log04)
order = t["aside"] < t["chosen"] <= t["mp4"] < t["master"]
record("K3 real Nov 2 output: one render job set its own first capture aside after a transient EPERM; the master then "
       "null-tested every chapter's soundtrack_0, the file both rules choose, each whole and frame_to_sample long",
       retry and order and len(null_tested) == len(old_chapters) == 7 and all(ok for _, ok, _ in rows3),
       f"chapter 04 log: EPERM on attempt 1, capture folder set aside to previous/04_principal_product.v1, attempt 2 "
       f"rendered {retry}; set-aside before the chosen capture, capture before the mp4, mp4 before the master {order}; "
       f"set-aside bytes equal the chosen {sha(aside) == sha(chosen04)}; master log null-tested {len(null_tested)} "
       f"chapters: " + "; ".join(f"{ch} {n}{'' if ok else ' WRONG'}" for ch, ok, n in rows3))

# ---------------------------------------------------------------- K4 Ruby's checks, then final-verification02 on her v3 master
RC = SCR / "render03-checks"
p = subprocess.run([sys.executable, "-I", "-B", str(R3 / "checks/run_checks.py"), str(RC), "--w", str(W)], cwd=REPO,
                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=1800)
sealed = (R3 / "checks/outputs/run_checks.out").read_bytes()
record("K4a Ruby's checks replayed into a fresh scratch folder: exit 0 and a transcript byte-equal to her sealed one",
       p.returncode == 0 and p.stdout == sealed, f"exit {p.returncode}, {p.stdout.decode().strip().splitlines()[-1]}")
LANE, WS, TARGET = RC / "lane", RC / "ws", RC / "target"
narration = json.loads((WS / "narration-binding02.json").read_text())["audio"]["narration"]
c = F.Ctx(argparse.Namespace(lane=str(LANE), jobs=str(WS / "jobs"), crate=str(WS / "explainer_corpus/logarithm_mysteries"),
                             narration=narration, series="01", verify=None, job_dir=None))
cues = list(F.read_script(c.crate / "script.tsv"))


def bind():
    try:
        m = F.bind_master(c, cues)
        return True, f"{len(m['bound'])} bound rows, {len(m['renders'])} renders"
    except F.Refused as refusal:
        return False, str(refusal)


ok_before, said_before = bind()
code_before, out_before = run_h3("verify-step", "--binding", "02", "--series", "01", "--step", "master", ws=WS, lane=LANE, target=TARGET)
mcfg = V3.read_config(WS / "jobs/logarithm-stage09-master01.py")
capture = Path(next(iter(mcfg["soundtracks"].values()))["path"])
bound_captures = sum(1 for r in json.loads((WS / "jobs/logarithm-stage09-master01-inputs.json").read_text())["files"]
                     if "/soundtrack/" in r["path"])
saved = capture.read_bytes()
capture.write_bytes(saved[:-1] + bytes([saved[-1] ^ 1]))
try:
    ok_flip, said_flip = bind()
    code_flip, out_flip = run_h3("verify-step", "--binding", "02", "--series", "01", "--step", "master", ws=WS, lane=LANE, target=TARGET)
finally:
    capture.write_bytes(saved)
ok_after, _ = bind()
record("K4b final-verification02's master gate on the v3 master: it accepts it, and refuses one changed capture because "
       "the master's bound rows now include the 13 captures",
       ok_before and code_before == 0 and bound_captures == 13 and not ok_flip and "bound inputs moved" in said_flip
       and code_flip != 0 and "moved" in out_flip and ok_after,
       f"before: {said_before}; v3 verify-step exit {code_before}; capture rows bound by the master {bound_captures}; "
       f"one byte of {capture.name} ({capture.parent.name}) changed: final-verification02 refused ({tail(said_flip, 90)}), "
       f"v3 verify-step refused ({tail(out_flip, 90)}); restored: accepted again {ok_after}")

# ---------------------------------------------------------------- K5 L1: the render job records any bytes as the capture
k5 = SCR / "k5/work"
folder = k5 / "soundtrack" / CH
folder.mkdir(parents=True)
frames = 120
whole = b"\x01\x00\xff\xff" * frame_to_sample(frames, 60)
cases5 = {}
for label, nframes, data in (("whole", frame_to_sample(frames, 60), whole), ("header only", 0, b""),
                             ("cut in half", frame_to_sample(frames, 60), whole[:len(whole) // 2])):
    path = folder / "soundtrack_0.wav"
    with wave.open(str(path), "wb") as w:
        w.setnchannels(2)
        w.setsampwidth(2)
        w.setframerate(48_000)
        w.writeframes(whole[:4 * nframes])
    if label == "cut in half":
        path.write_bytes(path.read_bytes()[:44 + len(data)])
    track = template_capture(k5, CH)
    accepted = not (track is None or not track.is_file())  # the job's only test, prepare_stage09_render.py:772
    cases5[label] = (accepted, complete(track, frames))
    path.rename(folder / f"kept-{label.replace(' ', '-')}.wav")
record("K5 L1: the render job's only test of the capture is that a file exists, so a header-only or half-written WAV "
       "is recorded; the length rule Ruby gives (48 kHz s16 stereo, frame_to_sample(frames, 60) frames) refuses both",
       cases5["whole"] == (True, True) and cases5["header only"] == (True, False) and cases5["cut in half"] == (True, False),
       "; ".join(f"{k}: job accepts {a}, length rule accepts {b}" for k, (a, b) in cases5.items())
       + "; the 7 real Nov 2 captures pass the length rule (K3)")

# ---------------------------------------------------------------- K6 the live timeline01 handover, from its job files only
b1 = json.loads(BINDING01.read_text())
tcfg = V3.read_config(TL_SCRIPT)
trows = json.loads(TL_INPUTS.read_text())["files"]
header = V3.WRITTEN_BY.search(TL_SCRIPT.read_text())
k6 = {"narration = binding01 = what v3 bind derives from root's --audio O": tcfg["narration"] == b1["audio"]["narration"] == str(V3.audio_paths(REAL_O)["narration"]),
      "binary = binding01's ws-build02 binary": tcfg["binary_sha256"] == b1["build"]["binary_sha256"] and b1["build"]["attempt"] == "02",
      "header names version 2 and the rows bind its bytes": header is not None and header.group(1) == V3.RENDER_V2_SHA
      and {"path": str(REPO / V3.RENDER_V2_REL), "sha256": V3.RENDER_V2_SHA} in trows,
      "row 0 is the script": trows[0]["sha256"] == sha(TL_SCRIPT),
      "job and lane": tcfg["job"] == "logarithm-stage09-timeline01" and tcfg["lane"] == str(REAL_LANE)}
record("K6 root's live timeline01 meets what v3's verify-step checks before its receipt (its lane folder not read)",
       all(k6.values()), "; ".join(f"{k} {v}" for k, v in k6.items()) + f"; {len(trows)} rows")

# ---------------------------------------------------------------- K7 N1: a master CONFIG without soundtracks
mjob = "logarithm-stage09-master01"
script, inputs, receipt = WS / "jobs" / f"{mjob}.py", WS / "jobs" / f"{mjob}-inputs.json", LANE / mjob / "resource-receipt.json"
saved7 = {q: q.read_bytes() for q in (script, inputs, receipt)}
try:
    cfg = V3.read_config(script)
    cfg.pop("soundtracks")
    text = script.read_text()
    line = re.search(r"^CONFIG = json\.loads\((.*)\)$", text, re.M)
    script.write_text(text[:line.start(1)] + repr(json.dumps(cfg, indent=1)) + text[line.end(1):])
    data = json.loads(inputs.read_text())
    data["files"][0]["sha256"] = sha(script)
    inputs.write_text(json.dumps(data, indent=2) + "\n")
    r = json.loads(receipt.read_text())
    r["inputs_sha256"] = r["final_inputs_sha256"] = sha(inputs)
    receipt.write_text(json.dumps(r, indent=2) + "\n")
    code7, out7 = run_h3("verify-step", "--binding", "02", "--series", "01", "--step", "master", ws=WS, lane=LANE, target=TARGET)
finally:
    for q, b in saved7.items():
        q.write_bytes(b)
code7b, _ = run_h3("verify-step", "--binding", "02", "--series", "01", "--step", "master", ws=WS, lane=LANE, target=TARGET)
record("K7 N1: a master whose CONFIG lacks soundtracks (version 2's shape), bound consistently, stops verify-step with a "
       "KeyError traceback instead of a REFUSED line", code7 != 0 and "KeyError: 'soundtracks'" in out7
       and "REFUSED" not in out7 and code7b == 0,
       f"exit {code7}, last line {out7.strip().splitlines()[-1]}; restored: exit {code7b}")

# ---------------------------------------------------------------- U1
after = read_only_state()
record("U1 everything read outside SCRATCH is unchanged (bytes and modification times)", after == BEFORE,
       f"{len(BEFORE)} files" + ("" if after == BEFORE else f"; changed {[k for k in BEFORE if after.get(k) != BEFORE[k]][:3]}"))
print(f"RENDER03 REVIEW CHECKS: {'PASS' if all(RESULTS) else 'FAIL'} ({sum(RESULTS)}/{len(RESULTS)})", flush=True)
sys.exit(0 if all(RESULTS) else 1)
