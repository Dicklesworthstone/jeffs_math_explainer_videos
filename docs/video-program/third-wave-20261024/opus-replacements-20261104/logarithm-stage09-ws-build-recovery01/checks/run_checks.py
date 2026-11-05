#!/usr/bin/env python3
"""Checks for recover_ws_build.py. Writes only inside SCRATCH (new, outside the repository).

    python3 -I -B checks/run_checks.py SCRATCH --w W        exit 0 when every check passes

Real inputs are read, never written: root's ws-build01 (receipt, log, workspace files), build01, the W stage and
target/geometry-kit01. The real cargo, guard and binaries never run; no real target folder is created.

- diagnose runs on root's real ws-build01.
- Recovery jobs run in a scratch workspace (installed by the sealed render helper) under an emulated guard that
  binds the inputs file before and after, as root's guard does, without admission, lane lock or sampling.
- `cargo` is a stub first on PATH. It writes the files cargo writes for this crate (unit folders with rustc
  dep-info, executables, the uplifted binary and .d) and prints cargo's Compiling/Finished/Running lines. Its
  modes plant each failure: `reuse` repeats ws-build01 (nothing compiled, old topic everywhere), the others
  break one proof at a time while the rest stays consistent.

Printed paths are shortened to SCRATCH, W, REPO and ~.
"""
import argparse
import contextlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
HELPER = HERE.parents[1] / "recover_ws_build.py"
REPO = HERE.parents[6]
spec = importlib.util.spec_from_file_location("recover_ws_build", HELPER)
X = importlib.util.module_from_spec(spec)
spec.loader.exec_module(X)
R, N = X.R, X.N
STAGE, CRATE = X.STAGE, X.CRATE

parser = argparse.ArgumentParser()
parser.add_argument("scratch")
parser.add_argument("--w", required=True)
ARGS = parser.parse_args()
SCR, W = Path(ARGS.scratch), Path(ARGS.w)
if not SCR.is_absolute() or (SCR.exists() and any(SCR.iterdir())) or SCR.resolve().is_relative_to(REPO.resolve()):
    sys.exit("SCRATCH must be a new or empty absolute folder outside the repository")
SCR.mkdir(parents=True, exist_ok=True)
WS, WS2 = SCR / "ws", SCR / "ws-real-target"
LANE, LANE2 = SCR / "lane", SCR / "lane-real-target"
TARGET = SCR / "target"
STUB = SCR / "stub"
REAL_WS = W / f"{STAGE}-workspace01"
REAL_TARGET = REPO / "target"
REAL_LANE = REPO / N.RECEIPTS_REL
RESULTS = []


def clean(text):
    text = str(text)
    for path, name in ((SCR.resolve(), "SCRATCH"), (SCR, "SCRATCH"), (W, "W"), (REPO, "REPO"), (Path.home(), "~")):
        text = text.replace(str(path), name)
    return text


def record(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'} {name}{': ' + clean(detail) if detail else ''}", flush=True)


def helper(*args, ws=WS, lane=LANE, target=TARGET):
    extra = (["--ws", str(ws)] if ws else []) + (["--receipts", str(lane)] if lane else []) + \
            (["--target-root", str(target)] if target else [])
    run = subprocess.run([sys.executable, "-I", "-B", str(HELPER), *map(str, args), "--w", str(W), *extra], cwd=REPO,
                         capture_output=True, text=True, timeout=900)
    return run.returncode, run.stdout + run.stderr


def result(out):
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        return {}


def refused(out, fragment):
    return out.startswith("REFUSED: ") and fragment in out


def ctx(ws=WS, lane=LANE, target=TARGET):
    return R.Ctx(argparse.Namespace(w=str(W), ws=str(ws), receipts=str(lane), target_root=str(target)))


@contextlib.contextmanager
def edited(path, change):
    """Mutate one file in place for one check, then put its exact bytes back."""
    path = Path(path)
    saved = path.read_bytes()
    try:
        change(path)
        yield
    finally:
        path.write_bytes(saved)


def snapshot():
    """Every real input this run must leave alone, with modification times where cargo would leave a trace."""
    snap = {f"W/{STAGE}/{k}": v for k, v in N.tree(W / STAGE).items()}
    snap["W names"] = sorted(p.name for p in W.iterdir())
    # Only ws-build job names: root's other stage09 jobs (audio decode, ...) legitimately appear while this runs.
    snap["lane ws-build names"] = sorted(p.name for p in REAL_LANE.iterdir() if p.name.startswith(f"{STAGE}-ws-build"))
    snap["target names"] = sorted(p.name for p in REAL_TARGET.iterdir())
    for p in sorted(REAL_WS.iterdir()):
        if p.is_file():
            snap[f"WS/{p.name}"] = (N.sha(p), p.stat().st_mtime_ns)
    snap.update({f"WS/{CRATE}/{k}": v for k, v in N.tree(REAL_WS / CRATE).items()})
    shared = REAL_TARGET / X.SHARED / "debug"
    for p in [shared / "logarithm_mysteries", shared / "logarithm_mysteries.d"] + \
             sorted(q for q in (shared / "build/logarithm_mysteries").rglob("*") if q.is_file()):
        snap[str(p.relative_to(REAL_TARGET))] = (N.sha(p), p.stat().st_mtime_ns)
    return snap


# ------------------------------------------------------------------ stand-ins

STUB_CARGO = r'''#!/usr/bin/env python3
# Stub cargo for the recovery checks: writes what cargo writes for logarithm_mysteries, in STUB_MODE.
import os, pathlib, shutil, sys
argv = sys.argv[1:]
mode = os.environ.get("STUB_MODE", "compile")
with open(os.environ["STUB_LOG"], "a") as log:
    log.write(" ".join(argv) + "\n")
td = pathlib.Path(os.environ["CARGO_TARGET_DIR"])
crate = str(pathlib.Path(argv[argv.index("--manifest-path") + 1]).parent)
old = os.environ["STUB_OLD"]
topic = old if mode == "reuse" else crate
units = td / "debug/build/logarithm_mysteries"
chapters = os.environ["STUB_CHAPTERS"].split(",")
def say(text):
    print(text, flush=True)
def unit(h, name, kind, topic_dir):
    u = units / h
    (u / "fingerprint").mkdir(parents=True, exist_ok=True)
    (u / "out").mkdir(parents=True, exist_ok=True)
    (u / "fingerprint" / ("dep-" + kind + "-logarithm_mysteries")).write_bytes(b"stand-in record src/main.rs")
    (u / "out" / (name + ".d")).write_text("%s: src/main.rs\n\nsrc/main.rs:\n\n# env-dep:CARGO_MANIFEST_DIR=%s\n"
                                           % (u / "out" / name, topic_dir))
    return u / "out" / name
def executable(path, topic_dir, kind):
    path.write_text("#!/bin/sh\n# stand-in %s, CARGO_MANIFEST_DIR=%s\n[ \"$1\" = chapters ] && printf '%%s\\n' %s && exit 0\nexit 2\n"
                    % (kind, topic_dir, " ".join(chapters)))
    path.chmod(0o755)
def compiling(verb):
    if mode not in ("reuse", "silent"):
        say("   %s logarithm_mysteries v0.1.0 (%s)" % (verb, crate))
    if mode == "other-name":
        say("   %s logarithm_mysteries v0.1.0 (%s)" % (verb, old))
if argv[0] == "test":
    compiling("Compiling")
    exe = unit("aaaa000000000001", "logarithm_mysteries-aaaa000000000001", "test-bin", topic)
    executable(exe, topic, "tests")
    say("    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s")
    shown = exe if mode != "running-elsewhere" else td.parent / "geometry-kit01/debug/build/logarithm_mysteries/aaaa000000000001/out/logarithm_mysteries-aaaa000000000001"
    say("     Running unittests src/main.rs (%s)" % shown)
    say("test result: ok. 241 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s")
elif argv[0] == "build":
    compiling("Compiling")
    exe = unit("bbbb000000000002", "logarithm_mysteries", "bin", topic)
    executable(exe, topic, "binary")
    up = td / "debug/logarithm_mysteries"
    if mode == "uplift-mismatch":
        executable(up, topic, "some other binary")
    else:
        shutil.copy2(exe, up)
    (td / "debug/logarithm_mysteries.d").write_text("%s: %s/src/main.rs\n" % (up, crate))
    say("    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s")
elif argv[0] == "clippy":
    compiling("Checking")
    for h, kind in (("cccc000000000003", "bin"), ("dddd000000000004", "test-bin")):
        stale = mode == "check-unit-old" and h.startswith("cccc")
        unit(h, "logarithm_mysteries-" + h, kind, old if stale else topic)
        (units / h / "out" / ("liblogarithm_mysteries-" + h + ".rmeta")).write_bytes(b"")
    say("    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s")
else:
    sys.exit("stub cargo: unexpected " + argv[0])
'''


def stub_env(**extra):
    env = dict(os.environ, PATH=f"{STUB / 'bin'}:{os.environ['PATH']}", STUB_LOG=str(STUB / "calls.log"),
               CARGO_TARGET_DIR=str(TARGET), STUB_OLD=str(W / STAGE), STUB_CHAPTERS=",".join(X.CHAPTERS))
    env.update({k: v for k, v in extra.items() if v is not None})
    for key in [k for k, v in extra.items() if v is None]:
        env.pop(key, None)
    if shutil.which("cargo", path=env["PATH"]) != str(STUB / "bin/cargo"):
        sys.exit("the stub cargo is not first on PATH; refusing to run a job script")
    return env


def calls():
    log = STUB / "calls.log"
    return log.read_text().splitlines() if log.exists() else []


def emulated_guard(job, inputs, command, **extra):
    """Root's guard as far as the inputs go: rows checked before and after, receipt and log in LANE/job."""
    folder = LANE / job
    folder.mkdir(parents=True)
    rows = json.loads(Path(inputs).read_text())["files"]

    def bound():
        moved = [r["path"] for r in rows if N.sha(r["path"]) != r["sha256"]]
        return moved, N.sha(inputs)
    receipt = {"started": "emulated", "phase": "native", "command": command, "preflight_only": False, "samples": [],
               "admitted": True, "emulated_guard": "checks only: no admission, lane lock or resource sampling"}
    moved, receipt["inputs_sha256"] = bound()
    assert not moved, ("inputs moved before launch", moved[:3])
    save = lambda: (folder / "resource-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    save()
    with (folder / "command.log").open("x") as log:
        run = subprocess.run(command, cwd=REPO, env=stub_env(**extra), stdout=log, stderr=subprocess.STDOUT, timeout=900)
    receipt["exit"] = run.returncode
    moved, receipt["final_inputs_sha256"] = bound()
    if moved:
        receipt["error"] = f"ValueError('reviewed input changed: {moved[0]}')"
    receipt["finished"] = "emulated"
    save()
    return run.returncode, (folder / "command.log").read_text()


def write(nn):
    code, out = helper("build-script", "--attempt", nn)
    assert code == 0, clean(out)
    return result(out)


def launch(nn, **extra):
    data = write(nn)
    return emulated_guard(data["job"], WS / f"{STAGE}-ws-build{nn}-inputs.json", ["sh", str(WS / f"{STAGE}-ws-build{nn}.sh")],
                          **extra)


def last_line(log):
    lines = [line for line in log.splitlines() if line.strip()]
    refusals = [line for line in lines if line.startswith("REFUSED")]
    return refusals[-1] if refusals else lines[-1]


def main():
    before = snapshot()
    (STUB / "bin").mkdir(parents=True)
    (STUB / "bin/cargo").write_text(STUB_CARGO)
    (STUB / "bin/cargo").chmod(0o755)

    # ---------------------------------------------------------------- diagnose, on root's real ws-build01
    code, out = helper("diagnose", ws=REAL_WS, lane=None, target=None)
    d = result(out)
    units = d.get("shared_units", {})
    record("D1 diagnose on root's ws-build01: exit 0 run, the sealed verifier's refusal, no crate compiled",
           code == 0 and d.get("receipt", {}).get("exit") == 0
           and d.get("sealed_build_verify") == "REFUSED: the new binary does not name the workspace topic"
           and d.get("log", {}).get("compiling_or_checking_logarithm_mysteries") == 0,
           f"exit {code}; {d.get('sealed_build_verify')}; finished {d.get('log', {}).get('finished')}")
    record("D2 ws-build01 ran build01's own test executable (unit c1b3bb6b1a138a1e) and kept build01's binary",
           d.get("log", {}).get("same_as_build01") is True
           and d.get("log", {}).get("test_executable", "").endswith("c1b3bb6b1a138a1e")
           and d.get("binary", {}).get("before") == d.get("binary", {}).get("after") == R.BUILD01_BINARY
           and d.get("binary", {}).get("names_W_stage") == 17 and d.get("binary", {}).get("names_workspace") == 0,
           f"{d.get('log', {}).get('test_executable')}; binary {d.get('binary')}")
    record("D3 all 4 shared logarithm units were compiled from W/logarithm-stage09 before the job; cargo's freshness "
           "record names no root",
           len(units) == 4 and all(u["rustc_env_dep_CARGO_MANIFEST_DIR"] == [str(W / STAGE)]
                                   and u["newest_output_utc"] < d["receipt"]["started"]
                                   and not u["cargo_record_names_a_root"] for u in units.values()),
           "; ".join(f"{k} {v['newest_output_utc'][11:19]}" for k, v in units.items()))
    record("D4 cargo only rewrote the uplifted .d from the new root during the job; the binary is from 04:57",
           d.get("shared_binary_utc", "") < d.get("receipt", {}).get("started", "")
           <= d.get("uplifted_dep_info_utc", "") <= d.get("receipt", {}).get("finished", "~"),
           f"binary {d.get('shared_binary_utc')}, .d {d.get('uplifted_dep_info_utc')}")

    # ---------------------------------------------------------------- the recovery script
    code, out = helper("build-script", "--attempt", "01")
    record("B1 attempt 01 is refused: it ran", code == 1 and refused(out, "attempt 01 ran"), out.strip())
    R.install(ctx())
    written = write("02")
    script = WS / f"{STAGE}-ws-build02.sh"
    text = script.read_text()
    rows = json.loads((WS / f"{STAGE}-ws-build02-inputs.json").read_text())["files"]
    cargo = [line for line in text.splitlines() if line.startswith("cargo ")]
    build01 = [line for line in (W / f"{STAGE}-build01.sh").read_text().splitlines() if line.startswith("cargo ")]
    ws01_text = (REAL_WS / f"{STAGE}-ws-build01.sh").read_text()
    ws01 = [line for line in ws01_text.splitlines() if line.startswith("cargo ")]
    record("B2 the recovery's three cargo lines equal build01's and ws-build01's, line for line",
           len(cargo) == 3 and cargo == build01 == ws01)
    sealed_text = R.build_text(ctx(), R.build_paths(ctx(), "02"), R.BUILD01_BINARY)  # what the sealed helper writes
    new_lines = [line for line in text.splitlines() if not line.startswith("#") and line not in sealed_text.splitlines()]
    gone = [line for line in sealed_text.splitlines() if not line.startswith("#") and line not in text.splitlines()]
    record("B3 against the sealed generator's attempt-02 script, only the target, the shared-binary preservation and "
           "the in-job proof differ",
           len(new_lines) == 6 and len(gone) == 9
           and 'export CARGO_TARGET_DIR="$CARGO_TARGET_DIR/logarithm-stage09-ws-build02"' in new_lines
           and 'export CARGO_TARGET_DIR="$CARGO_TARGET_DIR/geometry-kit01"' in gone
           and any(" artifacts --w " in line for line in new_lines),
           f"{len(new_lines)} new: " + " | ".join(line[:64] for line in new_lines)
           + f" || {len(gone)} gone: " + " | ".join(line[:40] for line in gone))
    record("B4 the inputs: the script first, then the sealed rows, this helper and the render seal",
           rows[0] == {"path": str(script), "sha256": N.sha(script)}
           and {"path": str(HELPER), "sha256": N.sha(HELPER)} in rows
           and written["new_target"] == str(TARGET / f"{STAGE}-ws-build02") and not Path(written["new_target"]).exists(),
           f"{len(rows)} rows, new target {written['new_target']}")
    real = X.preserved_rows(ctx(ws=REAL_WS, lane=REAL_LANE, target=REAL_TARGET))
    names = [str(p) for p in real]
    shared_units = [p for p in names if "/geometry-kit01/debug/build/logarithm_mysteries/" in p]
    record("B5 on the real paths the preserved rows bind ws-build01's 5 files, the shared binary and .d, and all of "
           "build01's unit files",
           sum(f"{STAGE}-ws-build01" in Path(p).name for p in names) == 5
           and str(REAL_TARGET / "geometry-kit01/debug/logarithm_mysteries") in names
           and str(REAL_TARGET / "geometry-kit01/debug/logarithm_mysteries.d") in names
           and len(shared_units) == sum(1 for p in (REAL_TARGET / "geometry-kit01/debug/build/logarithm_mysteries").rglob("*")
                                        if p.is_file()),
           f"{len(names)} preserved rows, {len(shared_units)} unit files")
    code, out = helper("build-script", "--attempt", "02", ws=WS2, lane=LANE2, target=REAL_TARGET)
    record("B6 build-script refuses a workspace that is not installed (scratch ws-real-target)",
           code == 1 and refused(out, "no workspace binding"), out.strip())
    R.install(ctx(ws=WS2, lane=LANE2, target=REAL_TARGET))
    code, out = helper("build-script", "--attempt", "02", ws=WS2, lane=LANE2, target=REAL_TARGET)
    r6 = result(out)
    record("B7 with the real target root: script written in scratch, new target target/logarithm-stage09-ws-build02 "
           "absent, shared binary build01's",
           code == 0 and r6.get("new_target") == str(REAL_TARGET / f"{STAGE}-ws-build02")
           and not (REAL_TARGET / f"{STAGE}-ws-build02").exists() and r6.get("shared_binary") == R.BUILD01_BINARY,
           f"{r6.get('input_rows')} rows ({r6.get('preserved_rows')} preserved), launch: {r6.get('launch_from_repo_root', out)[:160]}")
    code, out = helper("build-script", "--attempt", "02")
    record("B8 attempt 02 cannot be written twice", code == 1 and refused(out, "exists: use a new attempt number"), out.strip())
    (TARGET / f"{STAGE}-ws-build03").mkdir(parents=True)
    code, out = helper("build-script", "--attempt", "03")
    record("B9 a target folder that already exists is refused before anything is written",
           code == 1 and refused(out, f"{STAGE}-ws-build03 exists") and not (WS / f"{STAGE}-ws-build03.sh").exists(), out.strip())
    write("11")
    n = len(calls())
    run = subprocess.run(["sh", str(WS / f"{STAGE}-ws-build11.sh")], cwd=REPO, env=stub_env(CARGO_TARGET_DIR=None),
                         capture_output=True, text=True)
    record("B10 the job refuses outside the guard, before any cargo",
           run.returncode == 1 and "CARGO_TARGET_DIR is not the guard's target" in run.stderr and len(calls()) == n)
    data = write("04")
    (TARGET / f"{STAGE}-ws-build04").mkdir()
    n = len(calls())
    code4, log4 = emulated_guard(data["job"], WS / f"{STAGE}-ws-build04-inputs.json", ["sh", str(WS / f"{STAGE}-ws-build04.sh")])
    record("B11 a target that appears after writing is refused by the job, before any cargo",
           code4 == 1 and "this attempt needs a new target" in log4 and len(calls()) == n, last_line(log4))

    # ---------------------------------------------------------------- honest build
    n = len(calls())
    code2, log2 = emulated_guard(written["job"], WS / f"{STAGE}-ws-build02-inputs.json", ["sh", str(script)])
    ran = calls()[n:]
    td = TARGET / f"{STAGE}-ws-build02"
    record("S1 under the emulated guard the stub compiles in the new target; the job proves the topic and lists chapters",
           code2 == 0 and [c.split()[0] for c in ran] == ["test", "build", "clippy"] and '"artifacts": "PASS"' in log2
           and (td / "debug/logarithm_mysteries").is_file()
           and [line for line in log2.splitlines() if line.strip()][-len(X.CHAPTERS):] == X.CHAPTERS,
           f"exit {code2}, cargo {[c.split()[0] for c in ran]}")
    code, out = helper("build-verify", "--attempt", "02")
    v = result(out)
    record("S2 build-verify passes: compiled once per phase from the workspace, tests in the new target, every unit "
           "CARGO_MANIFEST_DIR=WS crate",
           code == 0 and v.get("artifacts", {}).get("units") == 4
           and v.get("artifacts", {}).get("binary_sha256") == v.get("binary_sha256") != R.BUILD01_BINARY
           and v.get("test_executable", "").startswith(str(td)),
           out if code else f"{v.get('compiled')}; unit {v.get('artifacts', {}).get('binary_unit')}")
    try:
        sealed = R.build_verify(ctx(), "02")
        record("S3 the sealed render helper's build-verify (used by its bind) accepts the same attempt",
               sealed["binary_sha256"] == v.get("binary_sha256"), sealed["binary"])
    except R.Refused as refusal:
        record("S3 the sealed render helper's build-verify (used by its bind) accepts the same attempt", False, str(refusal))
    with edited(td / "debug/logarithm_mysteries", lambda p: p.write_bytes((W / f"{STAGE}-build01-binary").read_bytes())):
        code, out = helper("build-verify", "--attempt", "02")
    record("S4 refused when the target's binary is build01's", code == 1 and refused(out, "binary is build01's"), out.strip())
    unit_d = td / "debug/build/logarithm_mysteries/dddd000000000004/out/logarithm_mysteries-dddd000000000004.d"
    with edited(unit_d, lambda p: p.write_text(p.read_text().replace(str(WS / CRATE), str(W / STAGE)))):
        code, out = helper("build-verify", "--attempt", "02")
    record("S5 refused when one unit's rustc dep-info names W/logarithm-stage09",
           code == 1 and refused(out, "was compiled with CARGO_MANIFEST_DIR"), out.strip())
    with edited(td / "debug/logarithm_mysteries.d", lambda p: p.write_text(p.read_text().replace(str(WS / CRATE), str(W / STAGE)))):
        code, out = helper("build-verify", "--attempt", "02")
    record("S6 refused when cargo's uplifted .d names the old sources", code == 1 and refused(out, "uplifted dep-info"), out.strip())
    log_path = LANE / f"{STAGE}-ws-build02/command.log"
    with edited(log_path, lambda p: p.write_text(p.read_text().replace('"artifacts": "PASS"', '"artifacts": "SKIPPED"'))):
        code, out = helper("build-verify", "--attempt", "02")
    record("S7 refused when the log lacks the job's own proof", code == 1 and refused(out, "did not prove the compiled topic"), out.strip())

    # ---------------------------------------------------------------- planted failures, each a full job
    cases = [
        ("05", "reuse", 1, "does not name the workspace topic", "the job fails",
         "P1 ws-build01 replayed (nothing compiled, old topic everywhere): the job itself fails, no exit 0"),
        ("06", "silent", 0, "not compiled in every phase", "build-verify",
         "P2 right files but no Compiling line: build-verify refuses"),
        ("07", "check-unit-old", 1, "was compiled with CARGO_MANIFEST_DIR", "the job fails",
         "P3 one Clippy unit from the old root: the job fails"),
        ("08", "running-elsewhere", 0, "the tests did not run from the new target", "build-verify",
         "P4 tests run from another target: build-verify refuses"),
        ("09", "uplift-mismatch", 1, "not the output of a unit compiled from the workspace", "the job fails",
         "P5 the uplifted binary is not the unit's output: the job fails"),
        ("10", "other-name", 0, "cargo names another logarithm_mysteries", "build-verify",
         "P6 cargo also names W/logarithm-stage09: build-verify refuses"),
    ]
    for nn, mode, exit_want, fragment, where, label in cases:
        code_j, log = launch(nn, STUB_MODE=mode)
        code, out = helper("build-verify", "--attempt", nn)
        if where == "the job fails":
            ok = code_j == exit_want and fragment in log and code == 1 and refused(out, "the job did not pass")
            detail = f"job exit {code_j}: {last_line(log)}; verify {out.strip()[:90]}"
        else:
            ok = code_j == exit_want and code == 1 and refused(out, fragment)
            detail = f"job exit {code_j}; verify {out.strip()}"
        record(label, ok, detail)

    after = snapshot()
    changed = sorted(k for k in set(before) | set(after) if before.get(k) != after.get(k))
    record("U1 real inputs untouched: W stage, W names, ws-build lane jobs, target names, real workspace files and "
           "build01's shared units (bytes and times)", not changed, ", ".join(changed[:5]))
    print(f"{sum(RESULTS)} PASS, {len(RESULTS) - sum(RESULTS)} FAIL")
    return 0 if all(RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main())
