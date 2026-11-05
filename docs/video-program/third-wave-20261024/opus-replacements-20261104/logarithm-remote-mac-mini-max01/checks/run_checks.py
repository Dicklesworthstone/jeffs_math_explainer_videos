#!/usr/bin/env python3
"""Small checks for the remote continuation. Real folders are only read; run with the remote's Python 3.14 where possible.

    python3 -I -B checks/run_checks.py SCRATCH        SCRATCH must not exist; it needs under 1 MB

Root barred large fixture writes while the local disk is short, so this has no end-to-end run. A reads real inputs,
including the real freeze and Lavender's --check. B runs the real gate and wait functions on tiny files. C runs the real
controller with a logging stand-in for python3. No guard, rebind, assembly, kit, timeline or score ran here.
"""
import ast
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
BUNDLE = HERE.parents[1]
REAL = HERE.parents[6]
SCRATCH = Path(sys.argv[1]).absolute()
SCRATCH.mkdir(parents=True)
SCRATCH = SCRATCH.resolve()
HOME = str(Path.home())
RESULTS = []


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


P = load("remote_pipeline", BUNDLE / "remote_pipeline01.py")
A = load("remote_audio", BUNDLE / "remote_audio_after_build01.py")
M, H = P.M, P.H
EL = M.EL
O = P.O


def listing(name):
    return {"/" + line for line in (O / name).read_text().splitlines() if line}


def refused(call, fragment):
    try:
        call()
    except H.Refused as refusal:
        return fragment in str(refusal)
    return False


def static():
    print("A. sources and overrides", flush=True)
    for path in sorted(BUNDLE.glob("*.py")) + [HERE]:
        compile(path.read_text(), str(path), "exec")
    record("every Python file parses", True, f"Python {sys.version.split()[0]}")
    done = subprocess.run(["/bin/bash", "-n", str(BUNDLE / "remote_continuation01.sh")], capture_output=True, text=True)
    record("the controller parses under macOS /bin/bash", done.returncode == 0, done.stderr)
    record("job names follow root's", H.job_name("timeline", "01") == "logarithm-remote-timeline01"
           and H.job_name("score", "01") == "logarithm-remote-score01")
    files = H.job_files(M.C, "logarithm-remote-timeline01")
    record("jobs run in O/jobs, scripts sit in O/job-scripts, target is O/target",
           files["folder"] == O / "jobs/logarithm-remote-timeline01" and files["script"] == O / "job-scripts/logarithm-remote-timeline01.py"
           and M.C.target == O / "target" and M.C.lane == O / "jobs")
    record("the binary is root's and the narration is inside the assemble job",
           M.BINARY == O / "logarithm_mysteries-release01" and M.NARRATION == O / "jobs/logarithm-remote-assemble01/narration01")
    record("the sealed default narration argument was replaced", M.full_tables.__defaults__ == (M.NARRATION,))
    record("the loaded sealed helper uses these gates and this seal", M.gates is P.gates and M.seal is P.seal)
    record("stage09's jobs resolve to the old lane", M.LANE / M.PRIOR["timeline"] == P.OLD_LANE / "logarithm-stage09-timeline02"
           and M.LANE / M.PRIOR["score"] == P.OLD_LANE / "logarithm-stage09-score02")
    record("the binding is the name agreed with Bronze", M.BINDING == O / "remote-binding01.json")
    text = (BUNDLE / "remote_audio_after_build01.py").read_text()
    record("no continuation names the old lane or the local guard",
           "five-film-release" not in text + (BUNDLE / "remote_assemble01.py").read_text() + (BUNDLE / "remote_continuation01.sh").read_text()
           and "guard_owner56g" not in text)


def names_in(function, module, label):
    """Every label.attribute read in a function that has not run here must exist on the loaded module."""
    tree = ast.parse(inspect.getsource(function))
    names = {n.attr for n in ast.walk(tree) if isinstance(n, ast.Attribute) and isinstance(n.value, ast.Name) and n.value.id == label}
    return sorted(n for n in names if not hasattr(module, n)), len(names)


def cli(*args):
    return subprocess.run([sys.executable, "-I", "-B", str(BUNDLE / "remote_pipeline01.py"), *args], cwd=REAL, capture_output=True, text=True, timeout=120)


def entry_points():
    print("A. entry points, on the real tree, before any job exists", flush=True)
    for label, module in (("M", M), ("H", H), ("N", P.N)):
        absent, count = names_in(P.gates, module, label)
        record(f"gates() reads only names that exist on {label}", not absent, f"{count} names; {absent}")
    digest = sha(BUNDLE / "SHA256SUMS")
    done = cli("seal", "--sums", digest)
    record("the seal command passes both bundles and sees stage09's jobs", done.returncode == 0 and '"PASS"' in done.stdout, done.stdout[-160:])
    done = cli("seal", "--sums", "0" * 64)
    record("the seal command refuses another hash", done.returncode == 1 and "not the seal" in done.stdout)
    kept = dict(M.PRIOR)
    M.PRIOR["score"] = str(SCRATCH / "not-transferred")
    record("refuses: stage09's job not transferred", refused(lambda: P.seal(digest), "has not been transferred"))
    M.PRIOR.update(kept)
    script = P.BUILD_SCRIPT
    P.BUILD_SCRIPT = SCRATCH / "build01.sh"
    record("refuses: a tree whose paths are not spelled as in root's manifest", refused(lambda: P.seal(digest), "path differs from root's"))
    P.BUILD_SCRIPT = script
    done = cli("poll", "--job", "logarithm-remote-build01")
    record("poll answers through the sealed command line", done.returncode == 0 and done.stdout.strip() == "waiting", done.stdout)
    done = cli("ready", "--integration-sha256", sha(M.S / "integration.json"), "--holds-sha256", "f" * 64)
    record("ready reaches the first receipt gate here: the pins hold and no remote job exists", done.returncode == 1
           and "no guard receipt for logarithm-remote-build01" in done.stdout, done.stdout[-160:])
    done = cli("step", "--step", "timeline")
    record("a step before the binding refuses", done.returncode == 1 and "no binding" in done.stdout, done.stdout[-160:])
    record("none of that wrote into O", not (O / "jobs").exists() and not (O / "job-scripts").exists() and not M.BINDING.exists())


def real_inputs():
    print("A. real inputs, read only", flush=True)
    transferred = listing("source-files01.txt") | listing("audio-files01.txt") | listing("support-files01.txt") | listing("launch-files01.txt")
    root = json.loads((O / "build-inputs01.json").read_text())["files"]
    record("root's manifest binds the unit plan, so the plain local freeze would refuse", str(A.UNITS) in {r["path"] for r in root})
    rows = A.inputs()
    mine = {str(p) for p in BUNDLE.iterdir() if p.is_file()}
    other = sorted(p for p in rows if p not in transferred and p not in mine)
    record("the real rebind freeze: no target row, no unit plan, every row transferred or in this bundle",
           not [p for p in rows if "/target/" in p] and str(A.UNITS) not in rows and not other, f"{len(rows)} rows; {other[:3]}")
    record("the rows the rebind needs are in it", {str(A.H), str(A.GUARD), str(A.ASSEMBLE), str(A.S / "initial-copy.json")} <= set(rows))
    wavs = sum(p.endswith(".wav") and "/prepared02/narration/" in p for p in rows)
    record("it binds the 80 retained Oliver WAVs", wavs == 80, f"{wavs}")
    done = subprocess.run([sys.executable, "-I", "-B", str(A.H), "--check"], cwd=REAL, capture_output=True, text=True, timeout=300)
    record("Lavender's real --check runs under this Python and predicts root's holds hash",
           done.returncode == 0 and "ff1ece5a" in done.stdout, (done.stdout + done.stderr)[-160:] if done.returncode else "")
    verify = [sys.executable, "-I", "-B", str(M.TOOLS / "coherent_narration.py"), "verify", str(M.OLD_AUDIO), "--script",
              str(M.CRATE / "script.tsv"), "--units", str(M.CRATE / "narration_units.json"), "--require-directions"]
    done = subprocess.run(verify, cwd=REAL, capture_output=True, text=True, timeout=300)
    record("the real coherent verify runs under this Python on the 80 retained WAVs", done.returncode == 0, done.stderr[-160:])
    prior = P.OLD_LANE / "logarithm-stage09-timeline02/timeline-output"
    frames = M.full_tables(prior, M.cues_of(), M.OLD_AUDIO)
    record("the sealed table gate holds on stage09's 13 chapters under this Python", len(frames) == 13 and sum(frames.values()) == 116245)
    lines = [line for line in (BUNDLE / "transfer-files02.txt").read_text().splitlines() if line]
    missing = [line for line in lines if not (REAL / line).is_file()]
    needed = [M.HERE.parent / "SHA256SUMS", M.V4_DIR / "prepare_stage09_render.py", M.V4_DIR / "SHA256SUMS", REAL / H.NATIVE_REL,
              P.OLD_LANE / "logarithm-stage09-timeline02/resource-receipt.json", P.OLD_LANE / "logarithm-stage09-score02/scored/stems/score.fingerprint"]
    listed = set(lines)
    record("the transfer list exists, lists real files, and holds what this needs",
           not missing and all(str(p.relative_to(REAL)) in listed for p in needed), f"{len(lines)} files")


def tiny_gates():
    print("B. the real gates on tiny files", flush=True)
    base = SCRATCH / "gates"
    base.mkdir()
    keep = (P.BUILD_INPUTS, P.UNITS, M.S)
    a, units, before = base / "a.txt", base / "narration_units.json", base / "narration_units-before-rebind01.json"
    script = base / "build01.sh"
    for path, text in ((a, "a"), (units, "approved"), (before, "approved"), (script, "x")):
        path.write_text(text)
    manifest = base / "build-inputs01.json"
    write = lambda rows: manifest.write_text(json.dumps({"files": [{"path": str(p), "sha256": h} for p, h in rows.items()]}))
    rows = {a: sha(a), units: sha(units), script: sha(script)}
    write(rows)
    P.BUILD_INPUTS, P.UNITS, M.S = manifest, units, base
    try:
        run = lambda required=(script,): P.frozen_build({"inputs_sha256": sha(manifest)}, list(required))
        run()
        units.write_text("rewritten by the rebind")
        run()
        record("a rewritten unit plan passes when the preserved file is what the build froze", True)
        record("refuses: a manifest that is not the frozen one", refused(lambda: P.frozen_build({"inputs_sha256": "0" * 64}, []), "not the manifest"))
        before.write_text("other")
        record("refuses: a preserved plan that is not the frozen one", refused(run, "is not the one the rebind preserved"))
        before.write_text("approved")
        a.write_text("moved")
        record("refuses: any other frozen row that moved", refused(run, "frozen inputs moved"))
        a.write_text("a")
        record("refuses: a manifest that does not bind what the gate requires", refused(lambda: run([base / "absent.rs"]), "does not bind"))
    finally:
        P.BUILD_INPUTS, P.UNITS, M.S = keep
    jobs = SCRATCH / "jobs"
    (jobs / "j").mkdir(parents=True)
    receipt = jobs / "j/resource-receipt.json"
    kept = A.JOBS
    A.JOBS = jobs
    good = {"finished": "t", "admitted": True, "exit": 0, "inputs_sha256": "x", "final_inputs_sha256": "x"}
    try:
        record("the wait: no receipt is not success", not A.success("absent"))
        receipt.write_text('{"fin')
        record("the wait: a half-written receipt is not success", not A.success("j"))
        receipt.write_text(json.dumps({**good, "finished": None}))
        record("the wait: an unfinished job is not success", not A.success("j"))
        receipt.write_text(json.dumps(good))
        record("the wait: a good receipt is success", A.success("j"))
        for name, change in (("exit 1", {"exit": 1}), ("not admitted", {"admitted": False}), ("a resource stop", {"stopped_for_resource_limit": True}),
                             ("an error", {"error": "x"}), ("moved inputs", {"final_inputs_sha256": "y"})):
            receipt.write_text(json.dumps({**good, **change}))
            with contextlib.suppress(AssertionError):
                A.success("j")
                record(f"the wait stops on {name}", False)
                continue
            record(f"the wait stops on {name}", True)
    finally:
        A.JOBS = kept


STAND_IN = r'''#!/bin/bash
# Logs each call, as the controller made it, and exits as FAIL_AT says.
printf '%s\n' "$*" | sed "s#@@BUNDLE@@/##g; s#@@REMOTE@@/##g" >> "@@LOG@@"
case "$*" in
  *"${FAIL_AT:-no such call}"*) exit 7 ;;
esac
'''


def controller():
    print("C. the real controller with a logging stand-in for python3", flush=True)
    base = SCRATCH / "controller"
    bundle = base / BUNDLE.relative_to(REAL)
    bundle.mkdir(parents=True)
    shutil.copy2(BUNDLE / "remote_continuation01.sh", bundle)
    remote = base / O.relative_to(REAL)
    bin_dir = base / "home/.local/bin"
    bin_dir.mkdir(parents=True)
    log = base / "calls.log"
    stand_in = bin_dir / "python3"
    stand_in.write_text(STAND_IN.replace("@@LOG@@", str(log)).replace("@@BUNDLE@@", str(bundle)).replace("@@REMOTE@@", str(remote)))
    stand_in.chmod(0o755)

    def run(*args, **env):
        log.write_text("")
        done = subprocess.run(["/bin/bash", str(bundle / "remote_continuation01.sh"), *args], capture_output=True, text=True, timeout=60,
                              env={**os.environ, "HOME": str(base / "home"), **env})
        return done, log.read_text().splitlines()

    done, calls = run("SEAL", "INTEGRATION", "HOLDS")
    want = ["-I -B remote_pipeline01.py seal --sums SEAL", "-I -B remote_audio_after_build01.py",
            "-I -B remote_pipeline01.py bind --integration-sha256 INTEGRATION --holds-sha256 HOLDS"]
    for step in ("timeline", "score"):
        job = f"logarithm-remote-{step}01"
        want += [f"-I -B remote_pipeline01.py step --step {step}",
                 f"-I -B guard01.py --phase native --job {job} --inputs job-scripts/{job}-inputs.json -- python3 -I -B job-scripts/{job}.py",
                 f"-I -B remote_pipeline01.py verify-step --step {step}", f"-I -B remote_pipeline01.py compare --step {step}"]
    record("the controller makes exactly these calls, in order", done.returncode == 0 and calls == want, "\n".join(calls[:2]) if calls != want else f"{len(calls)} calls")
    record("its last line says render was not launched", done.stdout.strip().endswith("render not launched"))
    for fragment, last, name in (("seal --sums", 1, "a refused seal"), ("remote_audio_after_build01", 2, "a failed audio step"),
                                 ("bind --integration", 3, "a refused binding"), ("guard01.py --phase native --job logarithm-remote-timeline01", 5, "a refused timeline guard"),
                                 ("compare --step timeline", 7, "a timeline hard failure")):
        done, calls = run("SEAL", "INTEGRATION", "HOLDS", FAIL_AT=fragment)
        record(f"{name} stops everything after it", done.returncode == 7 and len(calls) == last, f"{len(calls)} calls")
    done, calls = run("SEAL", "INTEGRATION")
    record("a missing approval launches nothing", done.returncode != 0 and not calls)


def main():
    static()
    entry_points()
    real_inputs()
    tiny_gates()
    controller()
    print(f"{'ALL ' if all(RESULTS) else 'FAILED: '}{sum(RESULTS)}/{len(RESULTS)} PASS. No guard, rebind, assembly, kit, timeline or score ran.")
    return 0 if all(RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main())
