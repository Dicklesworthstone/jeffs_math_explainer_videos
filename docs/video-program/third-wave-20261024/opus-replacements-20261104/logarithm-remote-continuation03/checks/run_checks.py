#!/usr/bin/env python3
"""Small checks for continuation03. Real folders are only read; run with the remote's Python 3.14 where possible.

    python3 -I -B checks/run_checks.py SCRATCH        SCRATCH must not exist; it needs under 1 MB

A reads the real sources and root's real build script and manifests. B runs the topic probe on a real local Rust binary
from stage09, both accepted for its own crate and refused for this one. C runs build_proof and seal on tiny files. D runs
the real controller with a logging stand-in for python3. The remote binary, bind, timeline and score did not run here.
"""
import ast
import hashlib
import importlib.util
import json
import os
import re
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


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


spec = importlib.util.spec_from_file_location("remote_pipeline03", BUNDLE / "remote_pipeline03.py")
P = importlib.util.module_from_spec(spec)
spec.loader.exec_module(P)
P1, M, H = P.P1, P.M, P.H
O = P1.O


def refused(call, fragment=""):
    try:
        call()
    except M.Refused as refusal:
        return fragment in str(refusal)
    return False


def calls_of(path, name):
    tree = ast.parse(Path(path).read_text())
    return [n for n in ast.walk(tree) if isinstance(n, ast.Call) and isinstance(n.func, ast.Attribute) and n.func.attr == name]


def sources():
    print("A. sources, and root's real build script and manifests", flush=True)
    for path in (BUNDLE / "remote_pipeline03.py", HERE):
        ast.parse(path.read_text())
    record("every Python file parses", True, f"Python {sys.version.split()[0]}")
    done = subprocess.run(["/bin/bash", "-n", str(BUNDLE / "remote_continuation03.sh")], capture_output=True, text=True)
    record("the controller parses under macOS /bin/bash", done.returncode == 0, done.stderr[-120:])
    record("the sealed helpers load at their pins", sha(P1.__file__) == P.SEALED_HELPER_SHA and sha(P.SEALED / "SHA256SUMS") == P.SEALED_SUMS_SHA)
    record("the sealed gates() is the one in use, with its single byte search", M.gates is P.gates and P.SEALED_GATES is P1.gates
           and M.seal is P.seal and H.mentions is P.build_proof and len(calls_of(P1.__file__, "mentions")) == 1)
    pipeline = Path(M.__file__)
    record("on this path nothing else searches the binary: sealed gates() is replaced, V4 check() is never called",
           [n.func.attr for n in calls_of(pipeline, "mentions")] == ["mentions"] and "check(" not in (P.SEALED / "remote_pipeline01.py").read_text())
    text = P1.BUILD_SCRIPT.read_text()
    paths = P.script_paths(text)
    record("root's build01.sh is at the pin and in its frozen manifest", sha(P1.BUILD_SCRIPT) == P.BUILD_SCRIPT_SHA
           and P.rows_of(P1.BUILD_INPUTS)[str(P1.BUILD_SCRIPT)] == P.BUILD_SCRIPT_SHA)
    record("its variables resolve to this crate and this lane", paths["C"] == str(M.CRATE) and paths["O"] == str(O) and paths["R"] == str(REAL))
    lines = text.splitlines()
    record("it has every proof line", all(any(line.endswith(w) for line in lines) for w in P.BUILD_LINES), f"{len(P.BUILD_LINES)} lines")
    rows = P.rows_of(P1.BUILD_INPUTS)
    required = [M.CRATE / "Cargo.toml", M.CRATE / "Cargo.lock", Path(paths["W"]) / "engine-budget.toml"]
    record("its manifest binds the crate's Cargo files and the engine budget", all(rows.get(str(p)) == sha(p) for p in required))
    crate = M.tree(M.CRATE)
    sources_ = M.build_sources(crate)
    record("its manifest binds every crate source the sealed gate requires", len(sources_) > 20 and all(str(p) in rows for p in sources_ if p.name != "narration_units.json"), f"{len(sources_)} files")
    main_rs = (M.CRATE / "src/main.rs").read_text()
    record("main.rs still takes the topic from CARGO_MANIFEST_DIR", 'env!("CARGO_MANIFEST_DIR")' in main_rs and "run_with_contract" in main_rs)


def probe():
    print("B. the topic probe on a real local Rust binary, stage09's", flush=True)
    stage = Path(HOME) / "scratch/jmev-geometry-native-20261104-01/logarithm-stage09-workspace01"
    binary = stage / "logarithm-stage09-ws-build02-binary"
    data = binary.read_bytes()
    crate = stage / "explainer_corpus/logarithm_mysteries"
    literal = str(crate).encode()
    at = data.find(literal + "π".encode())
    record("in it the env! literal abuts the next literal, with no terminator", at > 0)
    search = re.escape(literal) + H.CONTINUES
    record("likely cause: the sealed search refuses that same literal when the next byte is ASCII",
           re.search(search, literal + "π".encode()) is not None and re.search(search, literal + b"loop") is None)
    saved = (M.BINARY, M.CRATE, M.TOOLS, M.NARRATION)
    M.BINARY, M.NARRATION = binary, M.OLD_AUDIO
    M.CRATE, M.TOOLS = crate, stage / "tools"
    found = P.compiled_topic()
    record("accepted for its own crate: script.tsv matched and the call is exact", found[3:5] == ["--topic", str(crate)] and found[2] == str(stage / "tools/narration_delivery.py")
           and found.count("--chapter-key") == 13, f"{len(found)} arguments")
    M.CRATE, M.TOOLS = saved[1], saved[2]
    record("refused for the repaired crate: this binary was not compiled for it", refused(P.compiled_topic, "delivery call differs"))
    M.TOOLS = stage / "tools"
    M.CRATE = stage / "explainer_corpus/other"
    record("refused for another crate too", refused(P.compiled_topic, "delivery call differs"))
    M.BINARY, M.CRATE, M.TOOLS, M.NARRATION = saved
    stub = SCRATCH / "stops-early"
    stub.write_text("#!/bin/bash\necho 'facts self-check failed' >&2\nexit 1\n")
    stub.chmod(0o755)
    M.BINARY = stub
    record("refused: a binary that stops before the call", refused(P.compiled_topic, "stopped before its delivery call"))
    stub.write_text('#!/bin/bash\nprintf "%s\\n" -I > "${PATH%%:*}/argv.txt"\nexit 0\n')
    record("refused: a wrong call, even one that exits 0", refused(P.compiled_topic, "delivery call differs"))
    M.BINARY = saved[0]


def fixture(name):
    base = SCRATCH / "proof" / name
    (base / "lane/logarithm-remote-build01").mkdir(parents=True)
    binary, target = base / "binary", base / "target"
    binary.write_bytes(b"pinned")
    target.write_bytes(b"pinned")
    digest = sha(binary)
    receipt = {"admitted": True, "exit": 0, "finished": True, "inputs_sha256": "a", "final_inputs_sha256": "a", "phase": "native",
               "command": ["/bin/bash", str(P1.BUILD_SCRIPT)]}
    assemble = base / "assemble-inputs01.json"
    assemble.write_text(json.dumps({"files": [{"path": str(binary), "sha256": digest}]}))
    return base, binary, target, digest, receipt, assemble


def proof_case(name, change=None):
    base, binary, target, digest, receipt, assemble = fixture(name)
    saved = (M.LANE, M.BINARY, P.BINARY_SHA, P.TARGET, P.compiled_topic, P1.ASSEMBLE_INPUTS, P.BUILD_LINES, P.BUILD_SCRIPT_SHA)
    M.LANE, M.BINARY, P.BINARY_SHA, P.TARGET, P1.ASSEMBLE_INPUTS = base / "lane", binary, digest, target, assemble
    P.compiled_topic = lambda: ["stand-in"]
    if change:
        change(base, binary, target, receipt, assemble)
    (base / "lane/logarithm-remote-build01/resource-receipt.json").write_text(json.dumps(receipt))
    try:
        result = P.build_proof(binary.read_bytes(), M.CRATE)
        error = None
    except M.Refused as refusal:
        result, error = None, str(refusal)
    M.LANE, M.BINARY, P.BINARY_SHA, P.TARGET, P.compiled_topic, P1.ASSEMBLE_INPUTS, P.BUILD_LINES, P.BUILD_SCRIPT_SHA = saved
    return result, error


def proof():
    print("C. build_proof and seal on tiny files; root's build script and manifest are the real ones", flush=True)
    P.PROOF.clear()
    result, error = proof_case("good")
    record("the exact binary, target, receipt, script and rows pass", result == 1 and error is None, str(error))
    record("the sealed gate's `> 0` still holds on its return", result > 0)
    record("the proof names the build, the manifest and the first narrated call", P.PROOF.get("manifest_path") == str(M.CRATE / "Cargo.toml")
           and P.PROOF.get("build_script_sha256") == P.BUILD_SCRIPT_SHA and P.PROOF.get("first_narrated_call") == ["stand-in"])

    def set_key(key, value):
        return lambda base, binary, target, receipt, assemble: receipt.__setitem__(key, value)

    def other_binary(base, binary, target, receipt, assemble):
        binary.write_bytes(b"another")

    def other_target(base, binary, target, receipt, assemble):
        target.write_bytes(b"another")

    def no_target(base, binary, target, receipt, assemble):
        P.TARGET = base / "absent"

    def other_lines(base, binary, target, receipt, assemble):
        P.BUILD_LINES = ('--manifest-path "$C/Other.toml"',)

    def other_script_pin(base, binary, target, receipt, assemble):
        P.BUILD_SCRIPT_SHA = "0" * 64

    def unfrozen_binary(base, binary, target, receipt, assemble):
        assemble.write_text(json.dumps({"files": []}))

    cases = (("another binary", other_binary, "is not the binary the build proved"), ("a different target", other_target, "target output"),
             ("no target", no_target, "target output"), ("a failed build", set_key("exit", 1), "not a successful stable run"),
             ("inputs that moved during the build", set_key("final_inputs_sha256", "b"), "not a successful stable run"),
             ("a resource stop", set_key("stopped_for_resource_limit", True), "not a successful stable run"),
             ("another command", set_key("command", ["/bin/sh", str(P1.BUILD_SCRIPT)]), "the build ran"),
             ("another build script", other_script_pin, "frozen build script"), ("a missing proof line", other_lines, "lacks"),
             ("a binary the assembly did not freeze", unfrozen_binary, "did not freeze"))
    for index, (name, change, fragment) in enumerate(cases):
        result, error = proof_case(f"case{index}", change)
        record(f"refuses: {name}", result is None and error is not None and fragment in error, str(error)[:90])
    record("refuses: a proof asked about another crate", refused(lambda: P.build_proof(b"", Path("/elsewhere")), "asked about"))

    saved_bundle, saved_seal, saved_lane = P.BUNDLE, P1.seal, M.LANE
    folder = SCRATCH / "seal"
    folder.mkdir()
    (folder / "a.txt").write_text("a\n")
    (folder / "SHA256SUMS").write_text(f"{sha(folder / 'a.txt')}  a.txt\n")
    for job in (P.REBIND, P.ASSEMBLE):
        (folder / job).mkdir()
        (folder / job / "resource-receipt.json").write_text(json.dumps({"admitted": True, "exit": 0, "finished": True, "inputs_sha256": "a", "final_inputs_sha256": "a"}))
    P.BUNDLE, M.LANE = folder, folder
    sealed_calls = []
    P1.seal = lambda digest: sealed_calls.append(digest)
    good = sha(folder / "SHA256SUMS")
    record("the seal passes, and it checks sealed01 at its pin", P.seal(good) == {"seal": "PASS"} and sealed_calls == [P.SEALED_SUMS_SHA])
    record("the seal refuses another hash", refused(lambda: P.seal("0" * 64), "is not the seal"))
    receipt = folder / P.REBIND / "resource-receipt.json"
    passed = receipt.read_text()
    receipt.write_text(json.dumps({"admitted": True, "exit": 1, "finished": True}))
    record("the seal refuses if rebind02 has not passed, and never starts it", refused(lambda: P.seal(good), "never starts it"))
    receipt.write_text(passed)
    (folder / "a.txt").write_text("b\n")
    record("the seal refuses a changed file", refused(lambda: P.seal(good), "a.txt"))
    P.BUNDLE, P1.seal, M.LANE = saved_bundle, saved_seal, saved_lane

    def gates_case():
        saved_gates, saved_bundle = P.SEALED_GATES, P.BUNDLE
        P.SEALED_GATES = lambda a: {"schema": "logarithm-remote-binding-v1", "remote_helper_sha256": "sealed", "checks": {"coherent_verify": []}, "files": {"x": "y"}}
        (SCRATCH / "SHA256SUMS").write_text("")
        P.BUNDLE = SCRATCH
        body = P.gates(None)
        P.SEALED_GATES, P.BUNDLE = saved_gates, saved_bundle
        return body

    body = gates_case()
    record("the binding keeps its schema, gains the proof, this helper and two file rows", body["schema"] == "logarithm-remote-binding-v1"
           and body["remote_helper_sha256"] == sha(BUNDLE / "remote_pipeline03.py") and "compiled_build" in body["checks"] and "coherent_verify" in body["checks"]
           and len(body["files"]) == 3)


STAND_IN = r'''#!/bin/bash
# Logs each call, as the controller made it, and exits as FAIL_AT says.
printf '%s\n' "$*" | sed "s#@@BUNDLE@@/##g; s#@@REMOTE@@/##g" >> "@@LOG@@"
case "$*" in
  *"${FAIL_AT:-no such call}"*) exit 7 ;;
esac
'''


def controller():
    print("D. the real controller with a logging stand-in for python3", flush=True)
    base = SCRATCH / "controller"
    bundle = base / BUNDLE.relative_to(REAL)
    bundle.mkdir(parents=True)
    shutil.copy2(BUNDLE / "remote_continuation03.sh", bundle)
    remote = base / O.relative_to(REAL)
    bin_dir = base / "home/.local/bin"
    bin_dir.mkdir(parents=True)
    log = base / "calls.log"
    stand_in = bin_dir / "python3"
    stand_in.write_text(STAND_IN.replace("@@LOG@@", str(log)).replace("@@BUNDLE@@", str(bundle)).replace("@@REMOTE@@", str(remote)))
    stand_in.chmod(0o755)

    def run(*args, **env):
        log.write_text("")
        done = subprocess.run(["/bin/bash", str(bundle / "remote_continuation03.sh"), *args], capture_output=True, text=True, timeout=60,
                              env={**os.environ, "HOME": str(base / "home"), **env})
        return done, log.read_text().splitlines()

    done, calls = run("SEAL", "INTEGRATION", "HOLDS")
    want = ["-I -B remote_pipeline03.py seal --sums SEAL", "-I -B remote_pipeline03.py bind --integration-sha256 INTEGRATION --holds-sha256 HOLDS"]
    for step in ("timeline", "score"):
        job = f"logarithm-remote-{step}01"
        want += [f"-I -B remote_pipeline03.py step --step {step}",
                 f"-I -B guard01.py --phase native --job {job} --inputs job-scripts/{job}-inputs.json -- python3 -I -B job-scripts/{job}.py",
                 f"-I -B remote_pipeline03.py verify-step --step {step}", f"-I -B remote_pipeline03.py compare --step {step}"]
    record("the controller makes exactly these calls, in order", done.returncode == 0 and calls == want, "\n".join(calls[:2]) if calls != want else f"{len(calls)} calls")
    record("it starts at bind: no rebind, assembly, build or audio call", not [c for c in calls if any(w in c for w in ("rebind", "assemble", "audio", "build", "cargo"))])
    record("its last line says render was not launched", done.stdout.strip().endswith("render not launched"))
    for fragment, last, name in (("seal --sums", 1, "a refused seal"), ("bind --integration", 2, "a refused binding"),
                                 ("guard01.py --phase native --job logarithm-remote-timeline01", 4, "a refused timeline guard")):
        done, calls = run("SEAL", "INTEGRATION", "HOLDS", FAIL_AT=fragment)
        record(f"{name} stops everything after it", done.returncode == 7 and len(calls) == last, f"{len(calls)} calls")
    done, calls = run("SEAL", "INTEGRATION")
    record("a missing approval launches nothing", done.returncode != 0 and not calls)


def main():
    sources()
    probe()
    proof()
    controller()
    print(f"{'ALL ' if all(RESULTS) else 'FAILED: '}{sum(RESULTS)}/{len(RESULTS)} PASS. The remote binary, bind, timeline and score did not run here.")
    return 0 if all(RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main())
