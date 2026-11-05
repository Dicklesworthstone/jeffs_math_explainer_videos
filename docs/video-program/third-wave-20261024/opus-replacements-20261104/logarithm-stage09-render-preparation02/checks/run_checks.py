#!/usr/bin/env python3
"""Checks for prepare_stage09_render.py version 2. Writes only inside SCRATCH (new, outside the repository).

    python3 -I -B checks/run_checks.py SCRATCH --w W        exit 0 when every check passes

Real inputs are read, never written: root's production audio (EL/production-audio-stage09-01: prepared01,
prepared01-timings and the decoded takes its receipts name), the installed audio helper and its integration
receipt, the W stage and root's workspace. The real cargo, guard, binary and kit never run.

- check-audio runs on the real prepared folder. Its coherent verify and delivery contract are real runs of the
  workspace's tool copies on the real production narration.
- Planted faults use a scratch copy of prepared01 and prepared01-timings. The copy's receipts still name the real
  decoded takes, so only the planted file differs; where a fault breaks a hash chain, the chain is re-forged so
  only the check under test can refuse.
- bind and the timeline step run on an emulated build: a scratch workspace installed by version 1, ws-build02
  written by the recovery helper and run under an emulated guard (inputs bound before and after; no admission,
  lane lock or sampling) with a stub cargo. The stub's binary is a stand-in that lists the chapters and writes
  60 fps timelines for the bound narration. The timeline job's coherent verify is real.

Printed paths are shortened to SCRATCH, W, REPO and ~.
"""
import argparse
import contextlib
import hashlib
import importlib.util
import inspect
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
HELPER = HERE.parents[1] / "prepare_stage09_render.py"
REPO = HERE.parents[6]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


V2 = load("prepare_stage09_render_v2", HELPER)
X = V2.load_recovery()
V1, N = X.R, V2.N
STAGE, CRATE = V2.STAGE, V2.CRATE

parser = argparse.ArgumentParser()
parser.add_argument("scratch")
parser.add_argument("--w", required=True)
ARGS = parser.parse_args()
SCR, W = Path(ARGS.scratch), Path(ARGS.w)
if not SCR.is_absolute() or (SCR.exists() and any(SCR.iterdir())) or SCR.resolve().is_relative_to(REPO.resolve()):
    sys.exit("SCRATCH must be a new or empty absolute folder outside the repository")
SCR.mkdir(parents=True, exist_ok=True)
WS, LANE, TARGET, STUB = SCR / "ws", SCR / "lane", SCR / "target", SCR / "stub"
REAL_O = REPO / V2.N.EL_REL / "production-audio-stage09-01/prepared01"
REAL_WS = W / f"{STAGE}-workspace01"
REAL_LANE = REPO / N.RECEIPTS_REL
COPY = SCR / "audio/prepared01"
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
    return p.returncode, p.stdout + p.stderr


def result(out):
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        return {}


def refused(out, fragment):
    return out.startswith("REFUSED: ") and fragment in out


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


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


def json_change(fn):
    def apply(path):
        data = json.loads(path.read_text())
        fn(data)
        path.write_text(json.dumps(data, indent=2) + "\n")
    return apply


def real_snapshot():
    """Every real input this run must leave alone, with modification times."""
    snap = {}
    for root in (REAL_O, Path(f"{REAL_O}-timings")):
        for rel, digest in N.tree(root).items():
            snap[f"{root.name}/{rel}"] = (digest, (root / rel).stat().st_mtime_ns)
    for rel in (V2.INSTALLED_REL, V2.INTEGRATION_REL):
        snap[rel] = (sha(REPO / rel), (REPO / rel).stat().st_mtime_ns)
    snap["WS names"] = sorted(p.name for p in REAL_WS.iterdir() if p.name.startswith("narration-binding") or p.name == "jobs")
    snap["lane narrated jobs"] = sorted(p.name for p in REAL_LANE.iterdir()
                                        if p.name.startswith(tuple(f"{STAGE}-{s}" for s in ("timeline", "preview", "score", "render", "master"))))
    return snap


# ------------------------------------------------------------------ stand-ins

STUB_CARGO = r'''#!/usr/bin/env python3
# Stub cargo for the version 2 checks: writes cargo's files for logarithm_mysteries; the binary is a stand-in.
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
# Stand-in for the stage09 binary in the version 2 checks, CARGO_MANIFEST_DIR=@@CRATE@@
import json, pathlib, sys
CUES = json.loads(@@CUES@@)
argv = sys.argv[1:]
if argv == ["chapters"]:
    print("\n".join(CUES))
    sys.exit(0)
def opt(name):
    return argv[argv.index(name) + 1]
if argv[:2] == ["render", "all"] and "--timeline-only" in argv:
    out, fps, narration = pathlib.Path(opt("--out")), int(opt("--fps")), opt("--narration")
    out.mkdir(parents=True)
    for k, (chapter, cues) in enumerate(CUES.items()):
        rows = ["# explainer timeline v1", "chapter\t" + chapter, "fps\t%d" % fps, "frames\t%d" % ((10 + 2 * k) * fps)]
        rows += ["line\t%s\t%d\t0\t%s/%s.wav" % (c, j * fps // 2, narration, c) for j, c in enumerate(cues)]
        (out / (chapter + ".timeline.tsv")).write_text("\n".join(rows) + "\n")
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


def emulated_guard(job, inputs, command):
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
        p = subprocess.run(command, cwd=REPO, env=stub_env(), stdout=log, stderr=subprocess.STDOUT, timeout=900)
    receipt["exit"] = p.returncode
    moved, receipt["final_inputs_sha256"] = bound()
    if moved:
        receipt["error"] = f"ValueError('reviewed input changed: {moved[0]}')"
    receipt["finished"] = "emulated"
    save()
    return p.returncode, (folder / "command.log").read_text()


def main():
    before = real_snapshot()
    (STUB / "bin").mkdir(parents=True)
    (STUB / "bin/cargo").write_text(STUB_CARGO)
    (STUB / "bin/cargo").chmod(0o755)

    # ---------------------------------------------------------------- the real production audio, read-only
    code, out = run(HELPER, "check-audio", "--audio", REAL_O, ws=REAL_WS, scratch=False)
    a = result(out)
    record("A1 check-audio on root's real prepared01: installed helper 54df274a wrote both receipts, plan hash bound, "
           "real coherent verify and 13-chapter delivery contract pass",
           code == 0 and a.get("cues") == 80 and a.get("plan_mode") == "canonical"
           and a.get("audio", {}).get("helper_sha256") == V2.INSTALLED_SHA and a.get("voice") == V2.VOICE,
           out if code else f"{a['bound_files']} files, {a['audio']['marks']} marks, plan {a['audio']['phrase_marks_plan_sha256'][:12]}, "
           f"prepare receipt {a['audio']['prepare_receipt_sha256'][:12]}, selection by {a['audio']['selection']['by']}")
    code, out = run(HELPER, "build-verify", "--attempt", "01", ws=REAL_WS, scratch=False)
    record("A2 build-verify refuses ws-build01", code == 1 and refused(out, "reused build01's units"), out.strip())

    # ---------------------------------------------------------------- planted audio faults on a scratch copy
    shutil.copytree(REAL_O, COPY)
    shutil.copytree(f"{REAL_O}-timings", f"{COPY}-timings")
    code, out = run(HELPER, "check-audio", "--audio", COPY, ws=REAL_WS, scratch=False)
    record("A3 the scratch copy passes as the original does (positive control)", code == 0, out.strip()[-200:] if code else "")
    pr, tr = COPY / "prepare-receipt.json", Path(f"{COPY}-timings/timings-receipt.json")
    pt, installed = Path(f"{COPY}-timings/phrase-timings.json"), COPY / "narration/phrase-timings.json"
    old_tool = f"{V2.AUDIO_REL}/prepare_audio.py prepare"
    cases = [
        ("P1 one byte of phrase-marks-plan.json changed after prepare", [(COPY / "phrase-marks-plan.json", lambda p: p.write_bytes(p.read_bytes() + b"\n"))],
         "not the plan the prepare receipt bound"),
        ("P2 a prepare receipt without phrase_marks_plan_sha256 (version 1 shape)",
         [(pr, json_change(lambda d: d.pop("phrase_marks_plan_sha256")))], "not the plan the prepare receipt bound"),
        ("P3 a prepare receipt written by Bronze's original helper", [(pr, json_change(lambda d: d.update(tool=old_tool)))],
         "the prepare receipt was written by"),
        ("P4 a timings receipt written by the staged (uninstalled) helper",
         [(tr, json_change(lambda d: d.update(tool=d["tool"].replace("logarithm-stage09-audio-preparation02", "logarithm-stage09-plan-binding-fix01/staged"))))],
         "the timings receipt was written by"),
        ("P5 the installed phrase-timings.json differs from the timings output",
         [(installed, json_change(lambda d: d["cues"]["00_1"]["marks"].append({"id": "planted"})))], "is not installed in the narration"),
        ("P6 a Frederick prepare receipt", [(pr, json_change(lambda d: d["binding"]["voice"].update(name="Frederick")))],
         "prepare receipt binding"),
    ]
    for label, changes, fragment in cases:
        with edited(*changes):
            code, out = run(HELPER, "check-audio", "--audio", COPY, ws=REAL_WS, scratch=False)
        record(label, code == 1 and refused(out, fragment), out.strip()[:200])
    wav = COPY / "narration/05_1.wav"

    def flip(path):
        data = bytearray(path.read_bytes())
        data[-2] ^= 1
        path.write_bytes(bytes(data))

    def reforge(path):
        d = json.loads(pt.read_text())
        d["cues"]["05_1"]["wav_sha256"] = sha(wav)
        text = json.dumps(d, indent=2) + "\n"
        pt.write_text(text)
        installed.write_text(text)
    with edited((wav, flip), (pt, lambda p: None), (installed, reforge)):
        code, out = run(HELPER, "check-audio", "--audio", COPY, ws=REAL_WS, scratch=False)
    record("P7 one sample of 05_1.wav changed, phrase timings re-forged to match: the real coherent verify refuses",
           code == 1 and refused(out, "coherent_narration verify refused") and "05_1" in out, out.strip()[:200])
    integration = SCR / "integration-receipt01.json"
    shutil.copy2(REPO / V2.INTEGRATION_REL, integration)
    for label, change, fragment in (
            ("P8 an integration receipt for other installed bytes", lambda d: d.update(installed_sha256="0" * 64), "names another helper"),
            ("P9 an integration receipt that claims listening", lambda d: d.update(listening=True), "claims more")):
        with edited((integration, json_change(change))):
            try:
                V2.installed_helper(integration)
                record(label, False, "accepted")
            except V2.Refused as refusal:
                record(label, fragment in str(refusal), str(refusal))

    # ---------------------------------------------------------------- bind and the timeline step on an emulated build02
    cues = {}
    for cue in V2.script_cues(REAL_WS / CRATE / "script.tsv"):
        chapter = next(ch for ch in V2.CHAPTERS if ch.split("_", 1)[0] == cue.split("_", 1)[0])
        cues.setdefault(chapter, []).append(cue)
    (STUB / "stand-in.py").write_text(STAND_IN.replace("@@CUES@@", repr(json.dumps(cues))))
    V1.install(V1.Ctx(argparse.Namespace(w=str(W), ws=str(WS), receipts=str(LANE), target_root=str(TARGET))))
    code, out = run(X.HERE, "build-script", "--attempt", "02")
    job = result(out)["job"]
    code_b, log = emulated_guard(job, WS / f"{job}-inputs.json", ["sh", str(WS / f"{job}.sh")])
    code, out = run(HELPER, "build-verify", "--attempt", "02")
    v = result(out)
    record("B1 version 2's build-verify on the emulated ws-build02 delegates to the recovery helper and passes",
           code_b == 0 and code == 0 and v.get("verified_by", "").startswith("recover_ws_build.py 00a18c996f0b")
           and v.get("artifacts", {}).get("units") == 4, out[-300:] if code else v["verified_by"])
    code, out = run(V1.HERE, "bind", "--build", "02", "--audio", REAL_O, "--attempt", "09")
    v1 = json.loads((WS / "narration-binding09.json").read_text()) if code == 0 else {}
    record("B2 the sealed version 1 bind would accept these receipts but records Bronze's c6c854ce and no plan hash",
           code == 0 and v1["audio"]["helper_sha256"] == V2.AUDIO_SUMS_SHA[:0] + "c6c854ce62f86ecb4025a9999e883ff04e28a29492788fec05c9060238cdb1e3"
           and "phrase_marks_plan_sha256" not in v1["audio"], out.strip()[:200] if code else "version 1 binding09 in scratch")
    code, out = run(HELPER, "bind", "--build", "02", "--audio", REAL_O, "--attempt", "01")
    b = result(out)
    binding = json.loads((WS / "narration-binding01.json").read_text()) if code == 0 else {}
    files = binding.get("files", {})
    record("B3 version 2 binds the real production audio to the emulated build02",
           code == 0 and binding["schema"] == "logarithm-stage09-render-narration-binding-v2"
           and binding["helper_sha256"] == sha(HELPER) and binding["audio"]["helper_sha256"] == V2.INSTALLED_SHA
           and str(REAL_O / "phrase-marks-plan.json") in files and str(REPO / V2.INTEGRATION_REL) in files
           and str(REPO / V2.INSTALLED_REL) in files and binding["build"]["verified_by"].startswith("recover_ws_build.py"),
           out.strip()[:300] if code else f"{b['bound_files']} files, plan mode {b['plan_mode']}")
    code, out = run(HELPER, "bind", "--build", "02", "--audio", REAL_O, "--attempt", "01")
    record("B4 a binding is written once", code == 1 and refused(out, "exists: use a new attempt number"), out.strip())
    code, out = run(HELPER, "bind", "--build", "01", "--audio", REAL_O, "--attempt", "02")
    record("B5 bind refuses build 01", code == 1 and refused(out, "reused build01's units"), out.strip())
    code, out = run(HELPER, "step", "--binding", "09", "--series", "01", "--step", "timeline")
    record("B6 version 2 steps refuse version 1's binding", code == 1 and refused(out, "another helper's or WS's"), out.strip())

    code, out = run(HELPER, "step", "--binding", "01", "--series", "01", "--step", "timeline")
    s = result(out)
    script = WS / f"jobs/{STAGE}-timeline01.py"
    text = script.read_text() if code == 0 else ""
    record("S1 the timeline step is written: header names version 2, CONFIG keeps version 1's keys, real coherent verify argv",
           code == 0 and f"Written by prepare_stage09_render.py (sha256 {sha(HELPER)})" in text
           and V2.read_config(script)["verify_argv"] == binding["checks"]["coherent_verify"],
           out.strip()[:200] if code else f"{s['input_rows']} input rows")
    code_t, log = emulated_guard(f"{STAGE}-timeline01", WS / f"jobs/{STAGE}-timeline01-inputs.json", ["python3", "-I", "-B", str(script)])
    lines = [line for line in log.splitlines() if line.strip()]
    record("S2 under the emulated guard the job runs the real coherent verify on the production narration, then the "
           "stand-in's timelines; it ends with the success line",
           code_t == 0 and lines[-1] == V2.SUCCESS["timeline"], lines[-1] if lines else "no log")
    code, out = run(HELPER, "verify-step", "--binding", "01", "--series", "01", "--step", "timeline")
    record("S3 verify-step passes: 13 chapters at 60 fps, 80 cues in script order", code == 0 and result(out).get("cues") == 80,
           out.strip()[:200])
    code, out = run(HELPER, "step", "--binding", "01", "--series", "01", "--step", "score")
    rows = json.loads((WS / f"jobs/{STAGE}-score01-inputs.json").read_text())["files"] if code == 0 else []
    record("S4 the score step is written after the timeline verifies; its inputs bind the timeline outputs",
           code == 0 and any(r["path"].endswith("timeline-summary.json") for r in rows), out.strip()[:200])
    same = [name for name in ("job_name", "job_files", "render_work", "latest_render", "predecessors", "output_rows",
                              "step_config", "write_step", "verify_job", "read_config", "verify_step", "load_binding")
            if inspect.getsource(getattr(V2, name)) == inspect.getsource(getattr(V1, name))]
    record("S5 the narrated route is version 1's: JOB_TEMPLATE, SUCCESS, SCOPE and every step function unchanged",
           V2.JOB_TEMPLATE == V1.JOB_TEMPLATE and V2.SUCCESS == V1.SUCCESS and V2.SCOPE == V1.SCOPE and len(same) == 12,
           f"{len(same)}/12 functions identical")

    after = real_snapshot()
    changed = sorted(k for k in set(before) | set(after) if before.get(k) != after.get(k))
    record("U1 real inputs untouched: prepared01 and its timings (bytes and times), installed helper, integration receipt; "
           "no binding or job in root's workspace, no narrated job in the lane", not changed, ", ".join(changed[:5]))
    print(f"{sum(RESULTS)} PASS, {len(RESULTS) - sum(RESULTS)} FAIL")
    return 0 if all(RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main())
