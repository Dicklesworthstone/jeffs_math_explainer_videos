#!/usr/bin/env python3
"""Checks for prepare_stage09_render.py version 3. Writes only inside SCRATCH (new, outside the repository).

    python3 -I -B checks/run_checks.py SCRATCH --w W        exit 0 when every check passes

Real inputs are read, never written: the Nov 2 logarithm render work folder (its real soundtrack captures and
timelines), root's production audio, root's ws-build02 and the sealed version 2 bundle. Of root's running timeline01,
only its job script and inputs file are read; its lane folder is not. The real cargo, guard, binary and kit never run.

- A: read-only checks on real files. A1 is the positive check on actual old kit output: in the Nov 2 render, the
  capture rule (the helper's and the job template's) picks each chapter's capture, and every capture is as long
  as the kit's master requires of the file it null-tests (frame_to_sample of the chapter timeline's frames).
- The rest runs in a scratch lane and workspace: version 1 installs the workspace, the recovery helper writes
  ws-build02, and version 2 binds and writes timeline01, as root did. Version 3 then binds and carries on from
  that timeline01 without rerunning it: score, 13 chapter renders (one refused and retried) and the master.
  Jobs run under an emulated guard (inputs bound before and after; no admission, lane lock or sampling) with a
  stub cargo and a stand-in binary. The stand-in writes the files the kit writes, including soundtrack_0.wav and
  soundtrack_1.wav per render (one per ffmpeg job given soundtrack.wav) and the kit's set-aside of an earlier
  capture folder. The timeline job's coherent verify is real.

Printed paths are shortened to SCRATCH, W, REPO and ~.
"""
import argparse
import contextlib
import hashlib
import importlib.util
import inspect
import io
import json
import os
import pathlib
import re
import shutil
import subprocess
import sys
import wave
from pathlib import Path

HERE = Path(__file__).resolve()
HELPER = HERE.parents[1] / "prepare_stage09_render.py"
REPO = HERE.parents[6]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


V3 = load("prepare_stage09_render_v3", HELPER)
V2 = load("prepare_stage09_render_v2", REPO / V3.RENDER_V2_REL)
X = V3.load_recovery()
V1, N = X.R, V3.N
STAGE, CRATE, CHAPTERS = V3.STAGE, V3.CRATE, V3.CHAPTERS

parser = argparse.ArgumentParser()
parser.add_argument("scratch")
parser.add_argument("--w", required=True)
ARGS = parser.parse_args()
SCR, W = Path(ARGS.scratch), Path(ARGS.w)
if not SCR.is_absolute() or (SCR.exists() and any(SCR.iterdir())) or SCR.resolve().is_relative_to(REPO.resolve()):
    sys.exit("SCRATCH must be a new or empty absolute folder outside the repository")
SCR.mkdir(parents=True, exist_ok=True)
WS, LANE, TARGET, STUB, ASIDE = SCR / "ws", SCR / "lane", SCR / "target", SCR / "stub", SCR / "moved-aside"
REAL_O = REPO / N.EL_REL / "production-audio-stage09-01/prepared01"
REAL_WS = W / f"{STAGE}-workspace01"
REAL_LANE = REPO / N.RECEIPTS_REL
OLD_WORK = REAL_LANE / "logarithm-machine-render-work01"
V2_DIR = (REPO / V3.RENDER_V2_REL).parent
V2_SUMS_SHA = "352a81071f0b44134b163f91ecd9ed810022a8508630d09cea663f1d197ff2ed"
WORK = LANE / f"{STAGE}-render-work01"
RESULTS = []


def clean(text):
    text = str(text)
    for path, name in ((SCR.resolve(), "SCRATCH"), (SCR, "SCRATCH"), (W, "W"), (REPO, "REPO"), (Path.home(), "~")):
        text = text.replace(str(path), name)
    return text


def record(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'} {name}{': ' + clean(detail) if detail else ''}", flush=True)


def run(helper, *args, ws=WS, scratch=True):
    extra = ["--ws", str(ws)] + (["--receipts", str(LANE), "--target-root", str(TARGET)] if scratch else [])
    p = subprocess.run([sys.executable, "-I", "-B", str(helper), *map(str, args), "--w", str(W), *extra], cwd=REPO,
                       capture_output=True, text=True, timeout=900)
    return p.returncode, clean(p.stdout + p.stderr)  # cleaned before any caller cuts it


def result(out):
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        return {}


def refused(out, fragment):
    return out.startswith("REFUSED: ") and fragment in out


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def ctx():
    return V3.Ctx(argparse.Namespace(w=str(W), ws=str(WS), receipts=str(LANE), target_root=str(TARGET)))


def attempt(fn, *a):
    try:
        return True, fn(*a)
    except V3.Refused as refusal:
        return False, str(refusal)


@contextlib.contextmanager
def edited(*changes):
    """Mutate scratch files for one check, then put their exact bytes back."""
    saved = {Path(p): Path(p).read_bytes() for p, _ in changes}
    try:
        for path, change in changes:
            change(Path(path))
        yield
    finally:
        for path, data in saved.items():
            path.write_bytes(data)


def flip(path):
    data = bytearray(path.read_bytes())
    data[-1] ^= 1
    path.write_bytes(bytes(data))


@contextlib.contextmanager
def planted(*paths):
    """Plant scratch files for one check, then move them aside (nothing is deleted)."""
    try:
        for p in paths:
            Path(p).write_bytes(b"planted capture\n")
        yield
    finally:
        for k, p in enumerate(paths):
            ASIDE.mkdir(exist_ok=True)
            Path(p).rename(ASIDE / f"planted-{len(list(ASIDE.iterdir()))}-{k}-{Path(p).name}")


def tree_state(root):
    return {str(p.relative_to(root)): (sha(p), p.stat().st_mtime_ns) for p in sorted(Path(root).rglob("*")) if p.is_file()}


def real_snapshot():
    """Every real input this run reads or could touch, with modification times."""
    snap = {"prepared01": tree_state(REAL_O), "prepared01-timings": tree_state(f"{REAL_O}-timings"),
            "Nov 2 render work": tree_state(OLD_WORK), "render-preparation02": tree_state(V2_DIR)}
    for rel in (V3.INSTALLED_REL, V3.INTEGRATION_REL):
        snap[rel] = (sha(REPO / rel), (REPO / rel).stat().st_mtime_ns)
    snap["WS bindings"] = sorted(p.name for p in REAL_WS.iterdir() if p.name.startswith("narration-binding"))
    snap["WS jobs"] = sorted(p.name for p in (REAL_WS / "jobs").iterdir()) if (REAL_WS / "jobs").is_dir() else []
    snap["lane narrated jobs"] = sorted(p.name for p in REAL_LANE.iterdir()
                                        if p.name.startswith(tuple(f"{STAGE}-{s}" for s in ("timeline", "preview", "score", "render", "master"))))
    return snap


def template_function(name):
    m = re.search(rf"^def {name}\(.*?(?=^def |^if __name__)", V3.JOB_TEMPLATE, re.S | re.M)
    return m.group(0)


def template_function_v2(name):
    m = re.search(rf"^def {name}\(.*?(?=^def |^if __name__)", V2.JOB_TEMPLATE, re.S | re.M)
    return m.group(0)


# ------------------------------------------------------------------ stand-ins

STUB_CARGO = r'''#!/usr/bin/env python3
# Stub cargo for the version 3 checks: writes cargo's files for logarithm_mysteries; the binary is a stand-in.
import os, pathlib, shutil, sys
argv = sys.argv[1:]
td = pathlib.Path(os.environ["CARGO_TARGET_DIR"])
crate = str(pathlib.Path(argv[argv.index("--manifest-path") + 1]).parent)
units = td / "debug/build/logarithm_mysteries"
STAND_IN = open(os.environ["STUB_STAND_IN"]).read().replace("@@CRATE@@", crate)
def unit(h, name, kind):
    u = units / h
    (u / "fingerprint").mkdir(parents=True, exist_ok=True)
    (u / "out").mkdir(parents=True, exist_ok=True)
    (u / "fingerprint" / ("dep-" + kind + "-logarithm_mysteries")).write_bytes(b"stand-in record src/main.rs")
    (u / "out" / (name + ".d")).write_text("%s: src/main.rs\n\n# env-dep:CARGO_MANIFEST_DIR=%s\n" % (u / "out" / name, crate))
    return u / "out" / name
def executable(path):
    path.write_text(STAND_IN)
    path.chmod(0o755)
print("   %s logarithm_mysteries v0.1.0 (%s)" % ("Checking" if argv[0] == "clippy" else "Compiling", crate), flush=True)
if argv[0] == "test":
    exe = unit("aaaa000000000001", "logarithm_mysteries-aaaa000000000001", "test-bin")
    executable(exe)
    print("    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s")
    print("     Running unittests src/main.rs (%s)" % exe)
    print("test result: ok. 241 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s")
elif argv[0] == "build":
    exe = unit("bbbb000000000002", "logarithm_mysteries", "bin")
    executable(exe)
    shutil.copy2(exe, td / "debug/logarithm_mysteries")
    (td / "debug/logarithm_mysteries.d").write_text("%s: %s/src/main.rs\n" % (td / "debug/logarithm_mysteries", crate))
    print("    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s")
else:
    for h, kind in (("cccc000000000003", "bin"), ("dddd000000000004", "test-bin")):
        unit(h, "logarithm_mysteries-" + h, kind)
    print("    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s")
'''

STAND_IN = r'''#!/usr/bin/env python3
# Stand-in for the stage09 binary in the version 3 checks, CARGO_MANIFEST_DIR=@@CRATE@@
# It writes the files the kit writes for each command; nothing is rendered, scored or mixed.
import json, os, pathlib, sys
CUES = json.loads(@@CUES@@)
argv = sys.argv[1:]
mode = os.environ.get("STUB_MODE", "honest")
def opt(name):
    return argv[argv.index(name) + 1]
def timeline_text(chapter, fps, narration):
    k = list(CUES).index(chapter)
    rows = ["# explainer timeline v1", "chapter\t" + chapter, "fps\t%d" % fps, "frames\t%d" % ((10 + 2 * k) * fps)]
    rows += ["line\t%s\t%d\t0\t%s/%s.wav" % (c, j * fps // 2, narration, c) for j, c in enumerate(CUES[chapter])]
    return "\n".join(rows) + "\n"
def set_aside(path):  # the kit's rule (cli.rs set_aside): <parent>/previous/<stem>.vN<ext>
    if not path.exists():
        return
    folder = path.parent / "previous"
    folder.mkdir(parents=True, exist_ok=True)
    stem, dot, ext = path.name.partition(".")
    n = 1
    while (folder / ("%s.v%d%s" % (stem, n, dot + ext))).exists():
        n += 1
    path.rename(folder / ("%s.v%d%s" % (stem, n, dot + ext)))
def kit_choice(work, chapter):
    folder, n = work / "soundtrack" / chapter, 0
    while (folder / ("soundtrack_%d.wav" % n)).exists():
        n += 1
    return n - 1
if argv == ["chapters"]:
    print("\n".join(CUES))
    sys.exit(0)
command = argv[0]
if command == "render" and "--timeline-only" in argv:
    out, fps, narration = pathlib.Path(opt("--out")), int(opt("--fps")), opt("--narration")
    out.mkdir(parents=True)
    for chapter in CUES:
        (out / (chapter + ".timeline.tsv")).write_text(timeline_text(chapter, fps, narration))
    sys.exit(0)
if command == "score":
    stems = pathlib.Path(opt("--out")) / "stems"
    (stems / "qa").mkdir(parents=True)
    for chapter in CUES:
        (stems / (chapter + ".music.f32.wav")).write_bytes(("stand-in stem " + chapter).encode())
        (stems / "qa" / (chapter + ".json")).write_text(json.dumps({"chapter": chapter, "passed": True}))
    sys.exit(0)
if command == "render":
    chapter, work = argv[1], pathlib.Path(opt("--out"))
    fps, narration = int(opt("--fps")), opt("--narration")
    work.mkdir(parents=True, exist_ok=True)
    tracks = work / "soundtrack" / chapter
    set_aside(tracks)
    if mode != "no-capture":
        tracks.mkdir(parents=True)
        for n in range(2):  # one copy per ffmpeg job given soundtrack.wav
            (tracks / ("soundtrack_%d.wav" % n)).write_bytes(("stand-in soundtrack %s %d %s" % (chapter, n, mode)).encode())
    (work / (chapter + ".mp4")).write_bytes(("stand-in 4K chapter %s %s" % (chapter, mode)).encode())
    (work / (chapter + ".timeline.tsv")).write_text(timeline_text(chapter, fps, narration))
    for ext in ("cuelog.tsv", "cues.tsv", "layout.tsv"):
        (work / (chapter + "." + ext)).write_text("")
    sys.exit(0)
if command == "master":
    work = pathlib.Path(opt("--out"))
    used = ["%s soundtrack_%d.wav" % (chapter, kit_choice(work, chapter)) for chapter in CUES]
    if any(u.endswith("soundtrack_-1.wav") for u in used):
        print("no captured soundtrack", file=sys.stderr)
        sys.exit(1)
    (work / "logarithm_mysteries_master.mp4").write_text("stand-in master; null-tested " + ", ".join(used) + "\n")
    sys.exit(0)
if command == "captions":
    work = pathlib.Path(opt("--out"))
    for ext in ("en.srt", "en.vtt"):
        (work / ("logarithm_mysteries." + ext)).write_text("stand-in captions\n")
    sys.exit(0)
sys.exit(2)
'''


def stub_env(**extra):
    env = dict(os.environ, PATH=f"{STUB / 'bin'}:{os.environ['PATH']}", CARGO_TARGET_DIR=str(TARGET),
               STUB_STAND_IN=str(STUB / "stand-in.py"))
    env.update(extra)
    if shutil.which("cargo", path=env["PATH"]) != str(STUB / "bin/cargo"):
        sys.exit("the stub cargo is not first on PATH; refusing to run a job script")
    return env


def emulated_guard(job, inputs, command, **extra):
    """Root's guard as far as the inputs go: rows checked before and after, receipt and log in LANE/job."""
    folder = LANE / job
    folder.mkdir(parents=True)
    rows = json.loads(Path(inputs).read_text())["files"]

    def bound():
        return [r["path"] for r in rows if sha(r["path"]) != r["sha256"]], sha(inputs)
    receipt = {"started": "emulated", "phase": "native", "command": command, "preflight_only": False, "samples": [],
               "admitted": True, "emulated_guard": "checks only: no admission, lane lock or resource sampling"}
    moved, receipt["inputs_sha256"] = bound()
    assert not moved, ("inputs moved before launch", moved[:3])
    save = lambda: (folder / "resource-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    save()
    with (folder / "command.log").open("x") as log:
        p = subprocess.run(command, cwd=REPO, env=stub_env(**extra), stdout=log, stderr=subprocess.STDOUT, timeout=900)
    receipt["exit"] = p.returncode
    moved, receipt["final_inputs_sha256"] = bound()
    if moved:
        receipt["error"] = f"ValueError('reviewed input changed: {moved[0]}')"
    receipt["finished"] = "emulated"
    save()
    return p.returncode, (folder / "command.log").read_text()


def launch(job, **extra):
    """Run a written job as root's guard line does (helper output paths are shortened, so they are rebuilt here)."""
    return emulated_guard(job, WS / f"jobs/{job}-inputs.json", ["python3", "-I", "-B", str(WS / f"jobs/{job}.py")], **extra)


def last_line(log):
    lines = [line for line in log.splitlines() if line.strip()]
    return lines[-1] if lines else ""


def main():
    before = real_snapshot()
    (STUB / "bin").mkdir(parents=True)
    (STUB / "bin/cargo").write_text(STUB_CARGO)
    (STUB / "bin/cargo").chmod(0o755)

    # ---------------------------------------------------------------- A: real files, read-only
    namespace = {"pathlib": pathlib}
    exec(template_function("capture"), namespace)
    rows, ok = [], True
    for t in sorted(OLD_WORK.glob("*.timeline.tsv")):
        chapter = t.name.split(".")[0]
        table = dict(line.split("\t")[:2] for line in t.read_text().splitlines()
                     if line and not line.startswith("#") and line.split("\t")[0] in ("fps", "frames"))
        fps, frames = int(table["fps"]), int(table["frames"])
        chosen, by_job = V3.kit_capture(OLD_WORK, chapter), namespace["capture"](OLD_WORK, chapter)
        with wave.open(str(chosen)) as w:
            fmt = (w.getframerate(), w.getnchannels(), 8 * w.getsampwidth(), w.getnframes())
        want = (2 * frames * 48000 + fps) // (2 * fps)  # explainer_score::units::frame_to_sample
        ok &= (chosen == by_job == OLD_WORK / "soundtrack" / chapter / "soundtrack_0.wav" and fmt == (48000, 2, 16, want))
        rows.append(f"{chapter} {chosen.name} {fmt[3]}")
    moved_aside = sorted(p.relative_to(OLD_WORK).as_posix() for p in (OLD_WORK / "soundtrack/previous").rglob("*.wav"))
    record("A1 actual old output (Nov 2 render work): the helper's and the job's capture rule both pick each chapter's "
           "capture, not the kit's set-aside copy; each is 48 kHz S16 stereo and exactly frame_to_sample(frames) long, "
           "the length the master's null test requires",
           ok and len(rows) == 7 and moved_aside == ["soundtrack/previous/04_principal_product.v1/soundtrack_0.wav"],
           f"{'; '.join(rows)}; set aside, not chosen: {moved_aside}")
    code, out = run(HELPER, "check-audio", "--audio", REAL_O, ws=REAL_WS, scratch=False)
    a = result(out)
    record("A2 check-audio on root's real prepared01 passes (version 2's audio checks, unchanged)",
           code == 0 and a.get("cues") == 80 and a.get("bound_files") == 181, out[-200:] if code else f"{a['bound_files']} files")
    code, out = run(HELPER, "build-verify", "--attempt", "02", ws=REAL_WS, scratch=False)
    v = result(out)
    record("A3 build-verify passes root's real ws-build02 (read-only)", code == 0 and v.get("artifacts", {}).get("units") == 4,
           out[-200:] if code else f"binary {v['binary_sha256'][:12]}")
    sums = (V2_DIR / "SHA256SUMS").read_text().splitlines()
    v2_ok = sha(V2_DIR / "SHA256SUMS") == V2_SUMS_SHA and all(sha(V2_DIR / line.split("  ", 1)[1]) == line.split("  ", 1)[0] for line in sums)
    record("A4 version 2 is preserved: its seal 352a8107 verifies and its helper is b867aaa2",
           v2_ok and sha(REPO / V3.RENDER_V2_REL) == V3.RENDER_V2_SHA, f"{len(sums)} files")
    real_script = REAL_WS / f"jobs/{STAGE}-timeline01.py"
    real_rows = json.loads((REAL_WS / f"jobs/{STAGE}-timeline01-inputs.json").read_text())["files"]
    ok, why = attempt(V3.written_by, ctx(), real_script, real_rows)
    record("A5 root's real timeline01 (its job script and inputs file only): row 0 is the script, written by version 2 "
           "b867aaa2, whose bytes the guard's rows bind, so version 3 accepts its writer",
           ok and why == V3.RENDER_V2_SHA and real_rows[0]["sha256"] == sha(real_script), why if not ok else f"{len(real_rows)} rows")

    # ---------------------------------------------------------------- root's chain so far, in scratch
    cues = {ch: [] for ch in CHAPTERS}
    for cue in V3.script_cues(REAL_WS / CRATE / "script.tsv"):
        cues[next(ch for ch in CHAPTERS if ch.split("_", 1)[0] == cue.split("_", 1)[0])].append(cue)
    (STUB / "stand-in.py").write_text(STAND_IN.replace("@@CUES@@", repr(json.dumps(cues))))
    V1.install(V1.Ctx(argparse.Namespace(w=str(W), ws=str(WS), receipts=str(LANE), target_root=str(TARGET))))
    code, out = run(X.HERE, "build-script", "--attempt", "02")
    job = result(out)["job"]
    code_b, _ = emulated_guard(job, WS / f"{job}-inputs.json", ["sh", str(WS / f"{job}.sh")])
    code_v2b, _ = run(V2.HERE, "bind", "--build", "02", "--audio", REAL_O, "--attempt", "01")
    code_v2s, out = run(V2.HERE, "step", "--binding", "01", "--series", "01", "--step", "timeline")
    code_t, log = launch(f"{STAGE}-timeline01")
    code_v2v, _ = run(V2.HERE, "verify-step", "--binding", "01", "--series", "01", "--step", "timeline")
    record("B0 setup as root ran it: ws-build02, version 2's binding01 and timeline01 (real coherent verify), which "
           "version 2 verifies", (code_b, code_v2b, code_v2s, code_t, code_v2v) == (0, 0, 0, 0, 0) and
           last_line(log) == V3.SUCCESS["timeline"], f"exits {(code_b, code_v2b, code_v2s, code_t, code_v2v)}")

    # ---------------------------------------------------------------- B: version 3 binds and takes over timeline01
    code, out = run(HELPER, "bind", "--build", "02", "--audio", REAL_O, "--attempt", "02")
    b = json.loads((WS / "narration-binding02.json").read_text()) if code == 0 else {}
    record("B1 version 3 binds the real audio to build02 as binding02, beside version 2's binding01",
           code == 0 and b["schema"] == "logarithm-stage09-render-narration-binding-v3" and b["helper_sha256"] == sha(HELPER)
           and (WS / "narration-binding01.json").is_file(), out[-200:] if code else f"{len(b['files'])} bound files")
    tl_script = WS / f"jobs/{STAGE}-timeline01.py"
    tl_before = tree_state(LANE / f"{STAGE}-timeline01")
    code, out = run(HELPER, "verify-step", "--binding", "02", "--series", "01", "--step", "timeline")
    record("T1 version 3 accepts version 2's timeline01 as it stands (no rerun): writer b867aaa2, pinned and bound "
           "by the guard", code == 0 and result(out).get("cues") == 80, out.strip()[:200] if code else "13 chapters, 80 cues")
    tl_rows = json.loads((WS / f"jobs/{STAGE}-timeline01-inputs.json").read_text())["files"]
    probe = SCR / "probe" / tl_script.name
    probe.parent.mkdir()
    probe.write_text(tl_script.read_text().replace(V3.RENDER_V2_SHA, "0" * 64, 1))
    ok1, why1 = attempt(V3.written_by, ctx(), probe, tl_rows)
    ok2, why2 = attempt(V3.written_by, ctx(), tl_script, [r for r in tl_rows if r["sha256"] != V3.RENDER_V2_SHA])
    record("T2 a job written by any other helper is refused, and so is a version 2 job whose rows lack version 2's bytes",
           not ok1 and "not version 2 or 3" in why1 and not ok2 and "did not bind the helper" in why2, f"{why1} | {why2}")

    code, out = run(HELPER, "step", "--binding", "02", "--series", "01", "--step", "score")
    code_s, log = launch(f"{STAGE}-score01")
    code_v, out_v = run(HELPER, "verify-step", "--binding", "02", "--series", "01", "--step", "score")
    rows = json.loads((WS / f"jobs/{STAGE}-score01-inputs.json").read_text())["files"]
    record("T3 the score step consumes timeline01: written by version 3, its rows bind timeline01's outputs; it runs "
           "and verifies", code == 0 and code_s == 0 and code_v == 0
           and any(r["path"].endswith(f"{STAGE}-timeline01/timeline-summary.json") for r in rows)
           and {"path": str(HELPER), "sha256": sha(HELPER)} in rows, f"{len(rows)} input rows, {last_line(log)}")

    # ---------------------------------------------------------------- R: renders
    first = CHAPTERS[0]
    code, out = run(HELPER, "step", "--binding", "02", "--series", "01", "--step", "render", "--chapter", first)
    code_r, log = launch(f"{STAGE}-render-{first}-01", STUB_MODE="no-capture")
    record("R1 regression, missing capture: a render that leaves no soundtrack capture fails in its own job",
           code == 0 and code_r == 1 and "the render left no soundtrack capture" in log
           and not (LANE / f"{STAGE}-render-{first}-01/render-summary.json").exists(), last_line(log))
    ASIDE.mkdir(exist_ok=True)
    (WORK / f"{first}.mp4").rename(ASIDE / f"{first}.mp4")  # what root would do before a retry: move it aside
    code, out = run(HELPER, "step", "--binding", "02", "--series", "01", "--step", "render", "--chapter", first, "--retry", "1")
    code_r, log = launch(f"{STAGE}-render-{first}-01-r1")
    code_v, out_v = run(HELPER, "verify-step", "--binding", "02", "--series", "01", "--step", "render", "--chapter", first,
                        "--retry", "1")
    track, chosen = result(out_v).get("soundtrack", {}), WORK / f"soundtrack/{first}/soundtrack_1.wav"
    record("R2 the retry records the capture the kit will null-test: the last of soundtrack_0 and _1, its hash and count",
           code == 0 and code_r == 0 and code_v == 0 and track.get("path") == clean(chosen)
           and track.get("sha256") == sha(chosen) and track.get("captures") == 2,
           out_v[-200:] if code_v else f"{track['path']}, captures {track['captures']}")
    c = ctx()
    results = []
    for chapter in CHAPTERS[1:]:
        w = V3.write_step(c, "02", "01", "render", chapter)
        code_r, _ = launch(w["job"])
        ok, facts = attempt(V3.verify_step, c, "02", "01", "render", chapter)
        results.append(code_r == 0 and ok and facts["soundtrack"]["path"].endswith(f"soundtrack/{chapter}/soundtrack_1.wav"))
    record("R3 the other 12 chapters render and verify, each with its own capture recorded", all(results) and len(results) == 12,
           f"{sum(results)}/12")

    probe_ch = CHAPTERS[5]
    capture = WORK / f"soundtrack/{probe_ch}/soundtrack_1.wav"
    probe_job = f"{STAGE}-render-{probe_ch}-01"
    v2_rows = V2.output_rows(V2.Ctx(argparse.Namespace(w=str(W), ws=str(WS), receipts=str(LANE), target_root=str(TARGET))),
                             probe_job, "01")
    v3_rows = V3.output_rows(ctx(), probe_job, "01")
    record("G0 the omission repaired here: for the same render job, version 2's rows (what its master freezes) bind no "
           "capture; version 3's are version 2's plus the kit's choice",
           not any("soundtrack" in str(p) for p in v2_rows) and v3_rows == v2_rows + [capture],
           f"version 2 {len(v2_rows)} rows, version 3 {len(v3_rows)}; added {clean(v3_rows[-1])}")
    render_args =("verify-step", "--binding", "02", "--series", "01", "--step", "render", "--chapter", probe_ch)
    with edited((capture, flip)):
        code, out = run(HELPER, *render_args)
        code_m, out_m = run(HELPER, "step", "--binding", "02", "--series", "01", "--step", "master")
    record("G1 regression, changed capture: one byte changed after the render; verify-step and the master step both refuse",
           refused(out, "soundtrack capture soundtrack_1.wav moved") and refused(out_m, "moved")
           and not (WS / f"jobs/{STAGE}-master01.py").exists(), f"{out.strip()} | {out_m.strip()[:120]}")
    with planted(WORK / f"soundtrack/{probe_ch}/soundtrack_3.wav"):
        code_gap, out_gap = run(HELPER, *render_args)
        with planted(WORK / f"soundtrack/{probe_ch}/soundtrack_2.wav"):
            code, out = run(HELPER, *render_args)
    record("G2 the kit's gap rule: a stray soundtrack_3 after a gap is not the kit's choice (still passes); with "
           "soundtrack_2 the kit would null-test soundtrack_3, so verify refuses",
           code_gap == 0 and refused(out, "would null-test") and "soundtrack_3.wav" in out, out.strip()[:200])
    summary = LANE / f"{STAGE}-render-{probe_ch}-01/render-summary.json"

    def drop(path):
        data = json.loads(path.read_text())
        data.pop("soundtrack")
        path.write_text(json.dumps(data, indent=2) + "\n")
    with edited((summary, drop)):
        code, out = run(HELPER, *render_args)
    record("G3 a render summary without a capture (version 2's shape) is refused", refused(out, "written before version 3"),
           out.strip())

    # ---------------------------------------------------------------- M: master
    code, out = run(HELPER, "step", "--binding", "02", "--series", "01", "--step", "master")
    script = WS / f"jobs/{STAGE}-master01.py"
    cfg = V3.read_config(script) if code == 0 else {}
    rows = json.loads((WS / f"jobs/{STAGE}-master01-inputs.json").read_text())["files"] if code == 0 else []
    want = {}
    for chapter, job in cfg.get("render_jobs", {}).items():
        t = json.loads((LANE / job / "render-summary.json").read_text())["soundtrack"]
        want[chapter] = {"path": t["path"], "sha256": t["sha256"]}
    record("M1 the master step names each chapter's recorded capture in CONFIG and binds all 13 in its frozen inputs; "
           "chapter 00 is the retry's", code == 0 and cfg["soundtracks"] == want and len(want) == 13
           and all({"path": t["path"], "sha256": t["sha256"]} in rows for t in want.values())
           and cfg["render_jobs"][first] == f"{STAGE}-render-{first}-01-r1",
           out.strip()[:200] if code else f"{len(rows)} input rows, 13 capture rows")
    job_module = load("stage09_master_job", script)
    probe_ch = CHAPTERS[7]
    capture = WORK / f"soundtrack/{probe_ch}/soundtrack_1.wav"

    def master_refusal():
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            try:
                job_module.master()
                return None
            except SystemExit:
                return buffer.getvalue().strip()
            except Exception as error:  # it got past the capture check: not a refusal
                return f"no refusal ({type(error).__name__})"
    with planted(WORK / f"soundtrack/{probe_ch}/soundtrack_2.wav"):
        extra = master_refusal()
    with edited((capture, flip)):
        changed = master_refusal()
    untouched = not any((WORK / name).exists() for name in ("logarithm_mysteries_master.mp4", "logarithm_mysteries.en.srt"))
    record("M2 regression in the master job itself (its master() called directly, no guard): a new capture the kit would "
           "choose instead, and one changed byte of the recorded capture, each refused before the kit runs, nothing written",
           extra is not None and "another soundtrack capture" in extra and changed is not None and "another soundtrack capture" in changed
           and untouched and not (LANE / f"{STAGE}-master01").exists(), f"{extra} | {changed}")
    code_j, log = launch(f"{STAGE}-master01")
    s = json.loads((LANE / f"{STAGE}-master01/master-summary.json").read_text()) if code_j == 0 else {}
    master_text = (WORK / "logarithm_mysteries_master.mp4").read_text() if code_j == 0 else ""
    code_v, out_v = run(HELPER, "verify-step", "--binding", "02", "--series", "01", "--step", "master")
    record("M3 the master runs: its summary records the 13 captures it consumed, the stand-in kit null-tested the same "
           "files, and verify-step passes",
           code_j == 0 and last_line(log) == V3.SUCCESS["master"] and s.get("soundtracks") == want and code_v == 0
           and all(f"{ch} {Path(t['path']).name}" in master_text for ch, t in want.items()),
           out_v[-200:] if code_v else f"{result(out_v).get('soundtracks')} captures")
    with edited((capture, flip)):
        code, out = run(HELPER, "verify-step", "--binding", "02", "--series", "01", "--step", "master")
    record("M4 regression after the master: a changed capture is refused", refused(out, "moved"), out.strip())

    # ---------------------------------------------------------------- S: what did not change
    same_helper = [name for name in ("job_name", "job_files", "latest_render", "predecessors", "write_step", "read_config",
                                     "verify_step", "load_binding", "audio_facts", "check_audio", "build_verify", "check")
                   if inspect.getsource(getattr(V3, name)) == inspect.getsource(getattr(V2, name))]
    same_template = [name for name in ("refuse", "sha", "finished_ok", "rows", "first", "schedule", "findings", "run", "summary",
                                       "timeline", "preview", "score", "main") if template_function(name) == template_function_v2(name)]
    bind_same = inspect.getsource(V3.bind) == inspect.getsource(V2.bind).replace("binding-v2", "binding-v3")
    record("S1 everything else is version 2's: 12 helper functions, bind except its schema name, the success lines, "
           "SCOPE, and 13 job-template functions (timeline among them)",
           len(same_helper) == 12 and len(same_template) == 13 and bind_same and V3.SUCCESS == V2.SUCCESS and V3.SCOPE == V2.SCOPE,
           f"{len(same_helper)}/12 helper, {len(same_template)}/13 template")
    record("S2 timeline01 was only read: its folder is byte-for-byte unchanged by version 3's checks",
           tree_state(LANE / f"{STAGE}-timeline01") == tl_before)

    after = real_snapshot()
    changed = sorted(k for k in set(before) | set(after) if before.get(k) != after.get(k))
    record("U1 real inputs untouched: prepared01 and its timings, the Nov 2 render work (bytes and times), version 2's "
           "bundle, the installed helper and its receipt; no new binding or job in root's workspace, no new narrated "
           "job in the lane", not changed, ", ".join(changed[:5]))
    print(f"{sum(RESULTS)} PASS, {len(RESULTS) - sum(RESULTS)} FAIL")
    return 0 if all(RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main())
