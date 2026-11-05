#!/usr/bin/env python3
"""Small checks for continuation02. Real folders are only read; run with the remote's Python 3.14 where possible.

    python3 -I -B checks/run_checks.py SCRATCH        SCRATCH must not exist; it needs under 1 MB

A reads the real freeze. B traces every file the real audio tools open and compares them with that freeze and the
transfer lists; this is the check sealed01 lacked. C runs main() on tiny files with a stand-in guard and rebind. D runs
the real controller with a logging stand-in for python3. No real guard, rebind, assembly, kit, timeline or score ran.
"""
import ast
import hashlib
import importlib.util
import json
import os
import re
import runpy
import shutil
import subprocess
import sys
from pathlib import Path

if sys.argv[1:2] == ["--trace"]:
    # Runs one tool under an open() audit hook and writes the files it opened. Children are traced by their own run.
    out, script, *arguments = sys.argv[2:]
    seen = set()

    def hook(event, args):
        if event == "open" and isinstance(args[0], (str, bytes, os.PathLike)):
            seen.add(os.path.abspath(os.fsdecode(args[0])))

    sys.addaudithook(hook)
    sys.argv = [script, *arguments]
    stdout, sys.stdout = sys.stdout, open(os.devnull, "w")
    code = 0
    try:
        runpy.run_path(script, run_name="__main__")
    except SystemExit as stop:
        code = stop.code or 0
    sys.stdout = stdout
    Path(out).write_text(json.dumps(sorted(seen)))
    sys.exit(code)

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
REAL = HERE.parents[6]
SCRATCH = Path(sys.argv[1]).absolute()
SCRATCH.mkdir(parents=True)
SCRATCH = SCRATCH.resolve()
HOME = str(Path.home())
RESULTS = []
ROW = re.compile(r".*/remote-mac-mini-max01/jobs/logarithm-remote-rebind\d\d/resource-receipt\.json")


def clean(text):
    return str(text).replace(str(SCRATCH), "SCRATCH").replace(str(REAL), "REPO").replace(HOME, "HOME")


def record(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'}  {name}" + (f"  [{clean(detail)}]" if detail else ""), flush=True)


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def refused(call, fragment=""):
    try:
        call()
    except (AssertionError, subprocess.CalledProcessError) as error:
        return fragment in str(error)
    return False


P = load("audio02", BUNDLE / "remote_audio_after_build02.py")
A = P.A
O = A.O
SEALED_PIPE = load("remote_pipeline", P.SEALED / "remote_pipeline01.py")
M, H = SEALED_PIPE.M, SEALED_PIPE.H


def listed():
    rows = set()
    for name in ("source-files01.txt", "audio-files01.txt", "support-files01.txt", "launch-files01.txt", "comparison-files01.txt",
                 "raw-source-files01.txt"):
        rows |= {"/" + line for line in (O / name).read_text().splitlines() if line}
    rows |= {str(REAL / line) for line in (P.SEALED / "transfer-files02.txt").read_text().splitlines() if line}
    for folder in (P.SEALED, BUNDLE):
        rows |= {str(Path(path) / name) for path, _, names in os.walk(folder) for name in names}
    return rows


def static():
    print("A. sources and the real freeze", flush=True)
    files = [BUNDLE / "remote_audio_after_build02.py", HERE]
    for path in files:
        ast.parse(path.read_text())
    record("every Python file parses", True, f"Python {sys.version.split()[0]}")
    done = subprocess.run(["/bin/bash", "-n", str(BUNDLE / "remote_continuation02.sh")], capture_output=True, text=True)
    record("the controller parses under macOS /bin/bash", done.returncode == 0, done.stderr[-120:])
    record("only the rebind names are new", P.REBIND == "logarithm-remote-rebind02" and P.REBIND_INPUTS == O / "rebind-inputs02.json"
           and A.ASSEMBLY == "logarithm-remote-assemble01" and P.ASSEMBLE_INPUTS == O / "assemble-inputs01.json"
           and P.LATER_JOBS == ("logarithm-remote-timeline01", "logarithm-remote-score01"))
    record("the failed job and its manifest are never reused", "rebind01" not in (BUNDLE / "remote_continuation02.sh").read_text())
    record("sealed01 loads by its seal, and its pipeline and assembly name no rebind job",
           not [name for name in ("remote_pipeline01.py", "remote_assemble01.py") if "logarithm-remote-rebind" in (P.SEALED / name).read_text()])
    pin, P.SEALED_HELPER_SHA = P.SEALED_HELPER_SHA, "0" * 64
    record("refuses: a sealed helper at another hash", refused(P.load_sealed, "does not list"))
    P.SEALED_HELPER_SHA = pin
    record("the rebind and assembly helpers are the sealed ones", sha(A.H) == A.H_SHA and sha(A.ASSEMBLE) in (P.SEALED / "SHA256SUMS").read_text())


def real_freeze():
    stand_in = SCRATCH / "binary-stand-in"
    stand_in.write_bytes(b"stand-in")
    binary, binary_sha = P.BINARY, P.BINARY_SHA
    P.BUNDLE = SCRATCH  # this bundle is sealed after these checks, so the freeze hashes an empty stand-in for its seal
    (SCRATCH / "SHA256SUMS").write_text("")
    P.BINARY, P.BINARY_SHA = stand_in, sha(stand_in)
    inputs = A.inputs()
    original = A.inputs
    A.inputs = lambda: dict(inputs)
    rows = P.freeze()
    decode = A.read(P.DECODE)
    sources = {str(REAL / r["source"]) for r in decode["records"]}
    receipts = {str(REAL / r["downloads_receipt"]) for r in decode["records"]}
    record("the freeze is the sealed rows plus 166", len(inputs) == 4483 and len(rows) == 4483 + 166, f"{len(inputs)} + {len(rows) - len(inputs)}")
    record("it holds all 80 takes, their 80 download receipts and the voice binding", len(sources) == len(receipts) == 80
           and sources | receipts <= set(rows) and str(REAL / decode["binding"]["path"]) in rows)
    record("every take hash equals the decode receipt", all(rows[str(REAL / r["source"])] == r["source_sha256"] for r in decode["records"]))
    record("it holds the raw manifest, the binary, the cargo config, this helper", {str(P.RAW), str(stand_in), str(P.CARGO_CONFIG), str(P.HERE)} <= set(rows))
    record("the unit plan stays out and no provider original got in", str(A.UNITS) not in rows and not [p for p in rows if "/Downloads/" in p])
    record("no target path got in", not [p for p in rows if "/target/" in p])
    raw = A.read(P.RAW)
    real_raw = P.RAW

    def with_raw(files):
        P.RAW = SCRATCH / "raw.json"
        P.RAW.write_text(json.dumps({"files": files}))
        return lambda: P.freeze()

    record("refuses: a raw row dropped", refused(with_raw(raw["files"][1:]), "is not what the decode receipt names"))
    bad = [dict(row) for row in raw["files"]]
    bad[1]["sha256"] = "0" * 64
    record("refuses: a raw hash that differs from the file", refused(with_raw(bad), bad[1]["path"]))
    P.RAW = real_raw
    P.BINARY_SHA = "0" * 64
    record("refuses: a binary other than the one the build proved", refused(P.freeze, str(stand_in)))
    P.BINARY_SHA = sha(stand_in)
    pin, P.CARGO_CONFIG_SHA = P.CARGO_CONFIG_SHA, "0" * 64
    record("refuses: another cargo config", refused(P.freeze, "config.toml"))
    P.CARGO_CONFIG_SHA = pin
    record("the real cargo config is the bytes root read remotely", sha(P.CARGO_CONFIG) == pin)
    A.inputs = original
    P.BINARY, P.BINARY_SHA, P.BUNDLE = binary, binary_sha, BUNDLE
    return rows, inputs


def trace(rows, inputs):
    print("B. every file the real audio tools open", flush=True)
    script, units = str(M.CRATE / "script.tsv"), str(M.CRATE / "narration_units.json")
    tool = str(M.TOOLS / "coherent_narration.py")
    runs = {
        "rebind --check": [str(A.H), "--check"],
        "coherent check": [tool, "check", "--script", script, "--units", units],
        "coherent verify": [tool, "verify", str(M.OLD_AUDIO), "--script", script, "--units", units, "--require-directions"],
        "delivery contract": [str(M.TOOLS / "narration_delivery.py"), "--topic", str(M.CRATE), "--narration", str(M.OLD_AUDIO),
                              "--consumer", "render", "--gain-db", H.GAIN_DB] + [x for k in M.CHAPTERS for x in ("--chapter-key", k)],
    }
    transfers = listed()
    takes = {str(REAL / r["source"]) for r in A.read(P.DECODE)["records"]}
    allowed = set(rows) | {str(A.UNITS)}
    opened_all = set()
    for name, command in runs.items():
        out = SCRATCH / (name.replace(" ", "-") + ".json")
        done = subprocess.run([sys.executable, "-I", "-B", str(HERE), "--trace", str(out), *command], cwd=REAL, capture_output=True, text=True, timeout=500)
        opened = {p for p in json.loads(out.read_text()) if Path(p).is_file() and not p.startswith(sys.base_prefix)
                  and "/opt/homebrew/" not in p and "/logarithm-rebind-" not in p and str(SCRATCH) not in p}
        opened_all |= opened
        missing = sorted(opened - {*allowed, str(M.OLD_AUDIO)} - {p for p in opened if p.startswith(str(M.OLD_AUDIO))})
        record(f"{name}: read {len(opened)} files, none outside the freeze", done.returncode == 0 and not missing, ", ".join(missing[:3]) or done.stderr[-160:])
        record(f"{name}: all on the transfer lists", not sorted(opened - transfers), ", ".join(sorted(opened - transfers)[:3]))
    record("positive control: the sealed01 rows alone miss exactly the 80 takes", opened_all - set(inputs) - {str(A.UNITS)} == takes,
           f"{len(opened_all - set(inputs) - {str(A.UNITS)})} files")
    report = subprocess.run([sys.executable, "-I", "-B", str(A.H), "--check"], cwd=REAL, capture_output=True, text=True, timeout=500)
    record("the real --check predicts the four pinned outputs", json.loads(report.stdout)["outputs_sha256"] == P.PREDICTED)
    fixed = [SEALED_PIPE.BUILD_SCRIPT, M.C.ws_binding, REAL / M.N.GUARD_REL, REAL / M.N.GUARD_SOURCE_REL, M.HERE, M.V4_DIR / "prepare_stage09_render.py",
             *[M.TOOLS / name for name in H.TOOLS]]
    gone = [str(p) for p in fixed if str(p) not in transfers]
    record("the files the sealed gates and job writer name are on the transfer lists", not gone, ", ".join(gone[:3]))


GUARD = '''import hashlib, json, pathlib, sys
args = sys.argv[1:]
job, manifest = args[args.index("--job") + 1], args[args.index("--inputs") + 1]
base = pathlib.Path(@@BASE@@)
with (base / "calls.log").open("a") as log:
    log.write(job + " " + args[-1].rsplit("/", 1)[-1] + "\\n")
folder = base / "S/jobs" / job
folder.mkdir(parents=True)
if "rebind" in job:
    for path, text in json.loads((base / "outputs.json").read_text()).items():
        pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
        pathlib.Path(path).write_text(text)
digest = hashlib.sha256(open(manifest, "rb").read()).hexdigest()
(folder / "resource-receipt.json").write_text(json.dumps({"finished": True, "admitted": True, "exit": 0, "inputs_sha256": digest, "final_inputs_sha256": digest}))
'''


def flow_case(name, change=None, keep=True):
    """main() on tiny files: real launch, success, unused and row code; stand-in guard, rebind helper and freeze."""
    base = SCRATCH / "flow" / name
    (base / "S/jobs/logarithm-remote-build01").mkdir(parents=True)
    saved = {key: getattr(A, key) for key in ("S", "JOBS", "GUARD", "UNITS")}
    saved_p = {key: getattr(P, key) for key in ("rebind_helper", "freeze", "check_seal", "PREDICTED", "REBIND_INPUTS", "ASSEMBLE_INPUTS")}
    A.S, A.JOBS, A.UNITS = base / "S", base / "S/jobs", base / "S/workspace/explainer_corpus/logarithm_mysteries/narration_units.json"
    A.GUARD = base / "guard.py"
    A.GUARD.write_text(GUARD.replace("@@BASE@@", repr(str(base))))
    P.REBIND_INPUTS, P.ASSEMBLE_INPUTS = base / "rebind-inputs02.json", base / "assemble-inputs01.json"
    digest = hashlib.sha256(b"x").hexdigest()
    (A.JOBS / A.BUILD / "resource-receipt.json").write_text(json.dumps({"finished": True, "admitted": True, "exit": 0, "inputs_sha256": digest, "final_inputs_sha256": digest}))
    A.UNITS.parent.mkdir(parents=True)
    A.UNITS.write_text("old")
    P.PREDICTED = {key: hashlib.sha256(key.encode()).hexdigest() for key in saved_p["PREDICTED"]}
    outputs = {str(P.output(key)): key for key in P.PREDICTED}
    (base / "outputs.json").write_text(json.dumps(outputs))
    calls = []
    reports = {"--check": json.dumps({"outputs_sha256": P.PREDICTED}), "--verify": "{}"}
    P.rebind_helper = lambda mode: (calls.append(mode), reports[mode])[1]
    P.freeze = lambda: {str(base / "frozen"): "0" * 64}
    P.check_seal = lambda digest: None
    if change:
        change(base, reports, outputs)
    try:
        P.main("SEAL")
        error = None
    except (AssertionError, subprocess.CalledProcessError) as stop:
        error = str(stop)
    log = (base / "calls.log").read_text().splitlines() if (base / "calls.log").exists() else []
    manifest = json.loads(P.ASSEMBLE_INPUTS.read_text())["files"] if P.ASSEMBLE_INPUTS.exists() else []
    rebind_rows = json.loads(P.REBIND_INPUTS.read_text())["files"] if P.REBIND_INPUTS.exists() else []
    for key, value in saved.items():
        setattr(A, key, value)
    for key, value in saved_p.items():
        setattr(P, key, value)
    return error, calls, log, manifest, rebind_rows


def flow():
    print("C. main() on tiny files, with a stand-in guard and rebind helper", flush=True)
    error, calls, log, manifest, rebind_rows = flow_case("happy")
    record("rebind02 runs, then --verify, then assemble01", error is None and calls == ["--check", "--verify"]
           and log == ["logarithm-remote-rebind02 --apply", "logarithm-remote-assemble01 remote_assemble01.py"], str(error or log))
    rows = [row["path"] for row in manifest]
    named = [p for p in rows if ROW.fullmatch(p.replace("/S/jobs/", "/remote-mac-mini-max01/jobs/"))]
    record("the assembly manifest names one rebind receipt, rebind02's (Lavender's rule)", len(named) == 1 and "/logarithm-remote-rebind02/" in named[0], str(named))
    record("the assembly manifest holds the five rebind rows, the rebind manifest none of them", len(manifest) == len(rebind_rows) + 5)

    def used(path):
        return lambda base, reports, outputs: (base / path).mkdir(parents=True)

    for name, path in (("rebind02", "S/jobs/logarithm-remote-rebind02"), ("assemble01", "S/jobs/logarithm-remote-assemble01"),
                       ("timeline01", "S/jobs/logarithm-remote-timeline01"), ("score01", "S/jobs/logarithm-remote-score01")):
        error, calls, log, *_ = flow_case(f"used-{name}", used(path))
        record(f"a used {name} launches nothing and runs no --check", error is not None and "Already used" in error and not calls and not log)
    error, calls, log, *_ = flow_case("used-output", lambda base, reports, outputs: Path(next(p for p in outputs if p.endswith("spans01.json"))).write_text("old"))
    record("an existing rebind output launches nothing", error is not None and "spans01.json" in error and not log)

    def moved_prediction(base, reports, outputs):
        reports["--check"] = json.dumps({"outputs_sha256": {}})

    error, calls, log, *_ = flow_case("moved-prediction", moved_prediction)
    record("a moved prediction launches nothing", error is not None and "no longer predicts" in error and not log)

    def moved_output(base, reports, outputs):
        outputs[next(p for p in outputs if p.endswith("holds-bound01.json"))] = "other bytes"
        (base / "outputs.json").write_text(json.dumps(outputs))

    error, calls, log, manifest, _ = flow_case("moved-output", moved_output)
    record("rebind outputs off prediction stop before the assembly", error is not None and log == ["logarithm-remote-rebind02 --apply"] and not manifest, str(error))
    (BUILD := SCRATCH / "flow/seal").mkdir(parents=True)
    (BUILD / "a.txt").write_text("a\n")
    (BUILD / "SHA256SUMS").write_text(f"{sha(BUILD / 'a.txt')}  a.txt\n")
    P.BUNDLE = BUILD
    good = sha(BUILD / "SHA256SUMS")
    record("the seal check passes on matching files", P.check_seal(good) is None)
    (BUILD / "a.txt").write_text("b\n")
    record("the seal check refuses a changed file", refused(lambda: P.check_seal(good), "a.txt"))
    record("the seal check refuses another seal", refused(lambda: P.check_seal("0" * 64), "not the seal"))
    P.BUNDLE = BUNDLE


STAND_IN = r'''#!/bin/bash
# Logs each call, as the controller made it, and exits as FAIL_AT says.
printf '%s\n' "$*" | sed "s#@@BUNDLE@@/##g; s#@@SEALED@@/##g; s#@@REMOTE@@/##g" >> "@@LOG@@"
case "$*" in
  *"${FAIL_AT:-no such call}"*) exit 7 ;;
esac
'''


def controller():
    print("D. the real controller with a logging stand-in for python3", flush=True)
    base = SCRATCH / "controller"
    bundle = base / BUNDLE.relative_to(REAL)
    bundle.mkdir(parents=True)
    shutil.copy2(BUNDLE / "remote_continuation02.sh", bundle)
    remote = base / O.relative_to(REAL)
    sealed = base / P.SEALED.relative_to(REAL)
    bin_dir = base / "home/.local/bin"
    bin_dir.mkdir(parents=True)
    log = base / "calls.log"
    stand_in = bin_dir / "python3"
    stand_in.write_text(STAND_IN.replace("@@LOG@@", str(log)).replace("@@BUNDLE@@", str(bundle)).replace("@@SEALED@@", str(sealed)).replace("@@REMOTE@@", str(remote)))
    stand_in.chmod(0o755)

    def run(*args, **env):
        log.write_text("")
        done = subprocess.run(["/bin/bash", str(bundle / "remote_continuation02.sh"), *args], capture_output=True, text=True, timeout=60,
                              env={**os.environ, "HOME": str(base / "home"), **env})
        return done, log.read_text().splitlines()

    done, calls = run("SEAL", "INTEGRATION", "HOLDS")
    want = [f"-I -B remote_pipeline01.py seal --sums {P.SEALED_SUMS_SHA}", "-I -B remote_audio_after_build02.py SEAL",
            "-I -B remote_pipeline01.py bind --integration-sha256 INTEGRATION --holds-sha256 HOLDS"]
    for step in ("timeline", "score"):
        job = f"logarithm-remote-{step}01"
        want += [f"-I -B remote_pipeline01.py step --step {step}",
                 f"-I -B guard01.py --phase native --job {job} --inputs job-scripts/{job}-inputs.json -- python3 -I -B job-scripts/{job}.py",
                 f"-I -B remote_pipeline01.py verify-step --step {step}", f"-I -B remote_pipeline01.py compare --step {step}"]
    record("the controller makes exactly these calls, in order, sealed01's pipeline at its own seal", done.returncode == 0 and calls == want, "\n".join(calls[:2]) if calls != want else f"{len(calls)} calls")
    record("its last line says render was not launched", done.stdout.strip().endswith("render not launched"))
    for fragment, last, name in (("seal --sums", 1, "a refused sealed01"), ("remote_audio_after_build02", 2, "a failed audio step"),
                                 ("bind --integration", 3, "a refused binding"), ("guard01.py --phase native --job logarithm-remote-timeline01", 5, "a refused timeline guard")):
        done, calls = run("SEAL", "INTEGRATION", "HOLDS", FAIL_AT=fragment)
        record(f"{name} stops everything after it", done.returncode == 7 and len(calls) == last, f"{len(calls)} calls")
    done, calls = run("SEAL", "INTEGRATION")
    record("a missing approval launches nothing", done.returncode != 0 and not calls)


def main():
    static()
    rows, inputs = real_freeze()
    trace(rows, inputs)
    flow()
    controller()
    print(f"{'ALL ' if all(RESULTS) else 'FAILED: '}{sum(RESULTS)}/{len(RESULTS)} PASS. No real guard, rebind, assembly, kit, timeline or score ran.")
    return 0 if all(RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main())
