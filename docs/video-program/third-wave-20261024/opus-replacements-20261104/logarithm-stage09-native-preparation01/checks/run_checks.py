#!/usr/bin/env python3
"""Checks for prepare_stage09_native.py. Writes only inside SCRATCH (new, outside the repository).

    python3 -I -B checks/run_checks.py SCRATCH --w W        exit 0 when every check passes

Real inputs are read, never written: the EL stage09 candidate, White's seal, root's W stage, the
shared target binaries. Negative controls use copies in SCRATCH. A refusal that a pin would hide is
reached by consistent forgery: the copy is mutated, White's seal and root's receipt are rewritten to
match it, and the helper's pins are pointed at the forgery, so only the guard under test can refuse;
each forgery family first passes unmutated (its positive control).

The generated job script is exercised with a stub `cargo` placed first on PATH (it logs its argv and
copies a stand-in binary); the real cargo is never run, nothing compiles and nothing renders.
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
HELPER = HERE.parents[1] / "prepare_stage09_native.py"
REPO = HERE.parents[6]
spec = importlib.util.spec_from_file_location("prepare_stage09_native", HELPER)
H = importlib.util.module_from_spec(spec)
spec.loader.exec_module(H)

parser = argparse.ArgumentParser()
parser.add_argument("scratch")
parser.add_argument("--w", required=True)
ARGS = parser.parse_args()
SCR, W = Path(ARGS.scratch), Path(ARGS.w)
if not SCR.is_absolute() or (SCR.exists() and any(SCR.iterdir())) or SCR.resolve().is_relative_to(REPO.resolve()):
    sys.exit("SCRATCH must be a new or empty absolute folder outside the repository")
SCR.mkdir(parents=True, exist_ok=True)
EL = REPO / H.EL_REL
S9 = EL / "combined-repair-stage09"
WHITE = REPO / H.WHITE_REL
SHARED = REPO / "target/geometry-kit01"
RESULTS = []


def clean(text):
    text = str(text)
    for path, name in ((SCR.resolve(), "SCRATCH"), (SCR, "SCRATCH"), (W, "W"), (REPO, "REPO"), (Path.home(), "~")):
        text = text.replace(str(path), name)
    return text


def record(name, ok, detail=""):
    RESULTS.append(ok)
    print(f"{'PASS' if ok else 'FAIL'} {name}{': ' + clean(detail) if detail else ''}", flush=True)


def helper(*args):
    run = subprocess.run([sys.executable, "-I", "-B", str(HELPER), *map(str, args)], cwd=REPO,
                         capture_output=True, text=True)
    return run.returncode, run.stdout + run.stderr


def expect_refusal(name, call, fragment):
    try:
        call()
    except H.Refused as refusal:
        record(name, fragment in str(refusal), f"refused: {refusal}")
        return
    record(name, False, "not refused")


@contextlib.contextmanager
def patched(table, **values):
    saved = {k: table[k] for k in values}
    table.update(values)
    try:
        yield
    finally:
        table.update(saved)


@contextlib.contextmanager
def global_value(name, value):
    saved = getattr(H, name)
    setattr(H, name, value)
    try:
        yield
    finally:
        setattr(H, name, saved)


def paths(w=W, el=EL, white=WHITE, into=None, receipts=None, target=None):
    return H.Paths(argparse.Namespace(w=str(w), el=str(el), white=str(white), into=into and str(into),
                                      receipts=receipts and str(receipts), target_root=target and str(target)))


def snapshot():
    """Every real input this run must leave alone."""
    snap = {f"S9/{k}": v for k, v in H.tree(S9 / "candidate").items()}
    snap.update({f"S9/bound-plan/{k}": v for k, v in H.tree(S9 / "bound-plan").items()})
    snap["S9/receipt"] = H.sha(S9 / "root-integration-receipt01.json")
    snap["white"] = H.sha(WHITE / "SHA256SUMS")
    for sub in ("engine", "repo-view", "logarithm-stage08"):
        snap.update({f"W/{sub}/{k}": v for k, v in H.tree(W / sub).items()})
    # Root may add its own files to W meanwhile; this run must add none named for stage09.
    snap["W/stage09 names"] = sorted(p.name for p in W.iterdir() if "stage09" in p.name)
    snap.update({f"W/{k}": H.sha(W / k) for k in H.W_PIN if "/" not in k})
    binary = SHARED / "debug/logarithm_mysteries"
    snap["target/debug/logarithm_mysteries"] = H.sha(binary) if binary.is_file() else None
    return snap


# ------------------------------------------------------------------ forgeries

def copy_sources(tag):
    base = SCR / f"src-{tag}"
    s9 = base / "el/combined-repair-stage09"
    s9.mkdir(parents=True)
    shutil.copytree(S9 / "candidate", s9 / "candidate", symlinks=True)
    shutil.copytree(S9 / "bound-plan", s9 / "bound-plan", symlinks=True)
    shutil.copy2(S9 / "root-integration-receipt01.json", s9)
    (base / "white").mkdir()
    shutil.copy2(WHITE / "SHA256SUMS", base / "white")
    return base, s9, s9 / "candidate" / H.CRATE


def reseal(base, s9, receipt_edits=None):
    """White's seal and root's receipt rewritten to match the copy; returns the pins that bind them."""
    sums = base / "white/SHA256SUMS"
    keep = [line for line in sums.read_text().splitlines() if not line.partition("  ")[2].startswith("stage09/")]
    rows = [f"{h}  stage09/{sub}/{rel}" for sub in ("candidate", "bound-plan")
            for rel, h in sorted(H.tree(s9 / sub).items())]
    sums.write_text("\n".join(rows + keep) + "\n")
    path = s9 / "root-integration-receipt01.json"
    receipt = json.loads(path.read_text())
    receipt["sealed_manifest_sha256"] = H.sha(sums)
    receipt.update(receipt_edits or {})
    path.write_text(json.dumps(receipt, indent=2) + "\n")
    return {"white_sums": H.sha(sums), "root_receipt": H.sha(path)}


def source_case(name, tag, mutate, fragment, seal=True, extra_pins=None, receipt_edits=None, engine=False):
    base, s9, crate = copy_sources(tag)
    more = mutate(crate, s9)
    more = more if isinstance(more, dict) else {}
    pins = reseal(base, s9, {**(receipt_edits or {}), **more.get("receipt", {})}) if seal else {}
    pins.update(extra_pins or {})
    pins.update(more.get("pins", {}))
    p = paths(el=base / "el", white=base / "white")
    with patched(H.PIN, **pins):
        if fragment is None:
            try:
                facts = H.check_sources(p)
                H.check_engine(p, facts["crate"])
                H.derive(p)
                record(name, True, "the unmutated forgery passes check_sources, check_engine and derive")
            except H.Refused as refusal:
                record(name, False, f"refused: {refusal}")
        elif engine:
            facts = H.check_sources(p)
            expect_refusal(name, lambda: H.check_engine(p, facts["crate"]), fragment)
        else:
            expect_refusal(name, lambda: H.check_sources(p), fragment)


def replace_text(path, old, new, count=1):
    text = path.read_text(encoding="utf-8")
    assert text.count(old) == count, (path, old)
    path.write_text(text.replace(old, new), encoding="utf-8")


def fake_w(tag):
    fw = SCR / f"w-{tag}"
    fw.mkdir()
    for sub in ("engine", "repo-view", "logarithm-stage08"):
        shutil.copytree(W / sub, fw / sub, symlinks=True)
    for rel in H.W_PIN:
        if "/" not in rel:
            shutil.copy2(W / rel, fw / rel)
    for rel in ("engine-budget.toml", "logarithm-stage08/Cargo.toml"):
        (fw / rel).write_text((W / rel).read_text().replace(str(W), str(fw)))
    rows = json.loads((W / "logarithm-stage08-build01-inputs.json").read_text())["files"]
    for row in rows:
        if row["path"].startswith(f"{W}/"):
            row["path"] = f"{fw}{row['path'][len(str(W)):]}"
    write_rows(fw, rows)
    binding = json.loads((W / "logarithm-stage08-source-binding01.json").read_text())
    for row in binding["files"]:
        row["stage_sha256"] = H.sha(fw / "logarithm-stage08" / row["path"])
    (fw / "logarithm-stage08-source-binding01.json").write_text(json.dumps(binding, indent=2) + "\n")
    return fw, rows


def write_rows(fw, rows):
    (fw / "logarithm-stage08-build01-inputs.json").write_text(json.dumps({"files": rows}, indent=2) + "\n")


def repin(fw):
    return {rel: H.sha(fw / rel) for rel in H.W_PIN}


def engine_case(name, tag, mutate, fragment, engine_files=None):
    fw, rows = fake_w(tag)
    mutate(fw, rows)
    p = paths(w=fw)
    facts = H.check_sources(p)
    with patched(H.W_PIN, **repin(fw)), global_value("ENGINE_FILES", engine_files or H.ENGINE_FILES):
        if fragment is None:
            try:
                H.check_engine(p, facts["crate"])
                H.derive(p)
                record(name, True, "the unmutated copy of W passes check_engine and derive with its own pins")
            except H.Refused as refusal:
                record(name, False, f"refused: {refusal}")
        else:
            expect_refusal(name, lambda: (H.check_engine(p, facts["crate"]), H.derive(p)), fragment)
    return fw


def append(path, text):
    with open(path, "a") as handle:
        handle.write(text)


def row_update(fw, rows, rel, add=False):
    path = f"{fw}/engine/{rel}"
    if add:
        rows.append({"path": path, "sha256": H.sha(path)})
    else:
        next(r for r in rows if r["path"] == path)["sha256"] = H.sha(path)
    write_rows(fw, rows)


# ------------------------------------------------------------------ stub harness

STUB_CARGO = """#!/bin/sh
printf '%s\\n' "$*" >> "$STUB_LOG"
case "$1" in
  test) echo "test result: ok. ${STUB_TESTS:-241} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s" ;;
  build) mkdir -p "$CARGO_TARGET_DIR/debug" && cp "$STUB_BINARY" "$CARGO_TARGET_DIR/debug/logarithm_mysteries" &&
         chmod 755 "$CARGO_TARGET_DIR/debug/logarithm_mysteries" ;;  # cargo links a new executable; cp keeps an old mode
  clippy) [ -z "${STUB_CLIPPY_FAIL:-}" ] || { echo "error: stub Clippy failure" >&2; exit 101; } ;;
  *) echo "stub cargo: unexpected $1" >&2; exit 2 ;;
esac
"""
STUB_BINARY = "#!/bin/sh\n# stand-in for the stage09 binary\n[ \"$1\" = chapters ] || exit 2\nprintf '%s\\n' " + " ".join(H.CHAPTERS) + "\n"


def stub_env(stub, **extra):
    env = dict(os.environ, PATH=f"{stub / 'bin'}:{os.environ['PATH']}", STUB_LOG=str(stub / "cargo.log"),
               STUB_BINARY=str(stub / "stand-in-binary"), CARGO_TARGET_DIR=str(stub / "target"))
    env.update(extra)
    if shutil.which("cargo", path=env["PATH"]) != str(stub / "bin/cargo"):
        sys.exit("the stub cargo is not first on PATH; refusing to run the job script")
    return env


def run_job(stub, script, **extra):
    env = stub_env(stub, **extra)
    for key in [k for k, v in extra.items() if v is None]:
        env.pop(key)
    before = (stub / "cargo.log").read_text().splitlines() if (stub / "cargo.log").exists() else []
    run = subprocess.run(["sh", str(script)], cwd=REPO, env=env, capture_output=True, text=True)
    after = (stub / "cargo.log").read_text().splitlines() if (stub / "cargo.log").exists() else []
    return run.returncode, run.stdout + run.stderr, after[len(before):]


def fake_receipt(root, j, code, log, edit=None):
    folder = root / j["job"]
    folder.mkdir(parents=True)
    receipt = {"phase": "native", "admitted": True, "command": ["sh", str(j["script"])], "exit": code,
               "inputs_sha256": H.sha(j["inputs"]), "final_inputs_sha256": H.sha(j["inputs"])}
    receipt.update(edit or {})
    (folder / "resource-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    (folder / "command.log").write_text(log)


def main():
    print(f"helper sha256 {H.sha(HELPER)}; checks sha256 {H.sha(HERE)}")
    before = snapshot()

    # --- real inputs, read only
    code, out = helper("check", "--w", W)
    record("R1 check on the real inputs", code == 0 and '"check": "PASS"' in out, out.splitlines()[-1] if code else "")
    facts = json.loads(out) if code == 0 else {}
    record("R2 the derived Cargo files are root's stage08 bytes",
           facts.get("stage_cargo") == {"Cargo.toml": H.W_PIN["logarithm-stage08/Cargo.toml"],
                                        "Cargo.lock": H.W_PIN["logarithm-stage08/Cargo.lock"]})
    record("R3 stage09 differs from stage08's sources only in the reviewed files",
           facts.get("engine", {}).get("stage09_vs_stage08") == sorted(H.CHANGED | H.ADDED))
    record("R4 the receipt's open gates are carried, not cleared",
           facts.get("receipt_open_gates") == {"native_test": "pending", "new_recordings": "not yet generated",
                                               "listening": False, "fine_sync": False, "independent_final_av": False})

    # --- install
    into = SCR / "real-into"
    into.mkdir()
    code, out = helper("install", "--w", W, "--into", into)
    record("I1 install into a scratch folder", code == 0, out.splitlines()[-1] if code else "")
    stage = H.tree(into / H.STAGE)
    crate = H.tree(S9 / "candidate" / H.CRATE)
    same = all(stage[k] == v for k, v in crate.items() if k not in ("Cargo.toml", "Cargo.lock"))
    record("I2 48 files: candidate bytes, Cargo files as root's stage08 copy",
           set(stage) == set(crate) and len(stage) == 48 and same
           and stage["Cargo.toml"] == H.W_PIN["logarithm-stage08/Cargo.toml"]
           and stage["Cargo.lock"] == H.W_PIN["logarithm-stage08/Cargo.lock"]
           and stage["script.tsv"] == H.PIN["script"] and stage["narration_units.json"] == H.PIN["units"])
    binding = json.loads((into / f"{H.STAGE}-source-binding01.json").read_text())
    record("I3 the binding names the helper, stage09 script, units, oliver16 map and W",
           binding["helper_sha256"] == H.sha(HELPER) and binding["script_sha256"] == H.PIN["script"]
           and binding["direction_map"]["sha256"] == H.PIN["map16"] and binding["engine"]["w"] == str(W)
           and {r["path"]: r["stage_sha256"] for r in binding["files"]} == stage)
    run = subprocess.run([sys.executable, "-I", "-B", str(REPO / H.LOCK_CHECK_REL),
                          str(into / f"{H.STAGE}-original-Cargo.lock"), str(into / H.STAGE / "Cargo.lock")],
                         capture_output=True, text=True)
    record("I4 root's verify_patched_lock passes the installed lock", run.returncode == 0, run.stdout.strip())
    code, out = helper("install", "--w", W, "--into", into)
    record("I5 a second install refuses to overwrite", code == 1 and "refusing to overwrite" in out, out.strip())
    listing = sorted(p.name for p in HELPER.parent.iterdir())
    code, out = helper("install", "--w", W, "--into", HELPER.parent)
    record("I6 an --into inside the repository is refused, nothing written",
           code == 1 and "inside the repository" in out and sorted(p.name for p in HELPER.parent.iterdir()) == listing,
           out.strip())

    # --- job script on the real target (written, not run)
    code, out = helper("script", "--w", W, "--into", into, "--attempt", "01")
    record("J1 script --attempt 01 written", code == 0, out.splitlines()[-1] if code else "")
    info = json.loads(out) if code == 0 else {}
    j = H.job_paths(paths(into=into), "01")
    text = j["script"].read_text()
    root08 = (W / "logarithm-stage08-build01.sh").read_text().splitlines()
    cargo08 = [line for line in root08 if line.startswith("cargo ")]
    record("J2 the three cargo lines are root's stage08 lines verbatim (same flags, config, \"$M\")",
           len(cargo08) == 3 and all(line in text.splitlines() for line in cargo08))
    record("J3 the manifest is the installed stage09 copy, not stage08",
           f'M="$D/{H.STAGE}/Cargo.toml"' in text and "logarithm-stage08" not in text.replace("logarithm-stage08-build01", ""))
    shared_now = H.sha(SHARED / "debug/logarithm_mysteries")
    record("J4 the script preserves the shared binary it saw", f"= {shared_now} ]" in text
           and info.get("preserves") == shared_now, f"stage08 binary: {info.get('preserves_stage08_binary')}")
    rows = json.loads(j["inputs"].read_text())["files"]
    names = [r["path"] for r in rows]
    record("J5 inputs: absolute, unique, current, script first, helper and engine included, shared binary excluded",
           all(Path(n).is_absolute() for n in names) and len(set(names)) == len(names)
           and all(H.sha(r["path"]) == r["sha256"] for r in rows) and names[0] == str(j["script"])
           and str(HELPER) in names and sum(n.startswith(f"{W}/engine/") for n in names) == H.ENGINE_FILES
           and str(SHARED / "debug/logarithm_mysteries") not in names, f"{len(rows)} rows")
    record("J6 the launch line is root's 56g/20g guard, phase native, this job",
           info.get("launch_from_repo_root", "").startswith(f"python3 -I -B {H.GUARD_REL} --phase native "
                                                             f"--job {H.STAGE}-build01 --inputs "))
    code, out = helper("script", "--w", W, "--into", into, "--attempt", "01")
    record("J7 the same attempt again is refused", code == 1 and "use a new attempt number" in out, out.strip())
    code, out = helper("script", "--w", W, "--into", into, "--attempt", "1")
    record("J8 a malformed attempt is a usage error", code == 2)

    # --- stub harness: the job script's preservation logic
    stub = SCR / "stub"
    (stub / "bin").mkdir(parents=True)
    (stub / "bin/cargo").write_text(STUB_CARGO)
    (stub / "bin/cargo").chmod(0o755)
    (stub / "stand-in-binary").write_text(STUB_BINARY)
    (stub / "stand-in-binary").chmod(0o755)
    target = stub / "target"
    (target / "geometry-kit01/debug").mkdir(parents=True)
    planted = b"planted: the binary the build must preserve\n"
    (target / "geometry-kit01/debug/logarithm_mysteries").write_bytes(planted)
    sinto, receipts = stub / "into", stub / "receipts"
    sinto.mkdir()
    helper("install", "--w", W, "--into", sinto)
    common = ["--w", W, "--into", sinto, "--receipts", receipts, "--target-root", target]
    sp = paths(into=sinto, receipts=receipts, target=target)

    code, out = helper("script", *common, "--attempt", "01")
    j1 = H.job_paths(sp, "01")
    code, log, calls = run_job(stub, j1["script"], CARGO_TARGET_DIR=str(target))
    manifest = f"{sinto}/{H.STAGE}/Cargo.toml"
    flags = f"--locked --offline --config {W}/engine-budget.toml --manifest-path {manifest}"
    ok = code == 0 and calls == [f"test {flags} --all-targets", f"build {flags} --bin logarithm_mysteries",
                                 f"clippy {flags} --all-targets --no-deps -- -D warnings"]
    record("H1 stub run: exit 0, cargo called exactly as test, build, strict Clippy", ok, "" if ok else log.strip()[-300:])
    record("H2 the shared binary was preserved before cargo, the new one copied after",
           j1["before"].read_bytes() == planted and j1["after"].read_text() == STUB_BINARY
           and "PASS: the 19 packages are path copies" in log)
    sums = dict(reversed(line.split("  ", 1)) for line in j1["sums"].read_text().splitlines())
    record("H3 the sums file binds both copies", sums == {j1["before"].name: H.sha(j1["before"]),
                                                         j1["after"].name: H.sha(j1["after"])})
    record("H4 the chapters listing ran from the copy", log.strip().splitlines()[-13:] == H.CHAPTERS)

    fake_receipt(receipts, j1, 0, log)
    code, out = helper("verify", *common, "--attempt", "01")
    ok = code == 0 and '"verify": "PASS"' in out
    record("V1 verify passes the stub run's receipt", ok, "" if ok else out.strip()[-300:])
    for tag, edit, new_log, fragment in (
            ("V2 verify refuses 240 tests", None, log.replace("241 passed", "240 passed"), "test results"),
            ("V3 verify refuses a failed exit", {"exit": 101}, log, "did not pass"),
            ("V4 verify refuses other inputs", {"final_inputs_sha256": "0" * 64}, log, "input hashes"),
            ("V5 verify refuses a short chapters listing", None, log.replace("12_what_it_forgets\n", ""), "chapters listing"),
            ("V6 verify refuses a failed lock check", None, log.replace("PASS: the 19", "FAULT: the 19"), "lock check")):
        root = stub / f"receipts-{tag[:2].lower()}"
        fake_receipt(root, j1, edit.get("exit", 0) if edit else 0, new_log, edit)
        code, out = helper("verify", "--w", W, "--into", sinto, "--receipts", root, "--target-root", target, "--attempt", "01")
        record(tag, code == 1 and fragment in out, out.strip())

    helper("script", *common, "--attempt", "02")
    code, log, calls = run_job(stub, H.job_paths(sp, "02")["script"], CARGO_TARGET_DIR=None)
    record("H5 without the guard's CARGO_TARGET_DIR the script refuses before cargo",
           code == 1 and "REFUSED" in log and not calls, log.strip())
    helper("script", *common, "--attempt", "03")
    (target / "geometry-kit01/debug/logarithm_mysteries").write_bytes(b"relinked by someone else\n")
    j3 = H.job_paths(sp, "03")
    code, log, calls = run_job(stub, j3["script"])
    record("H6 a shared binary changed after the script was written: refused, nothing copied, no cargo",
           code == 1 and "not the one this script preserves" in log and not calls and not j3["before"].exists(), log.strip())
    helper("script", *common, "--attempt", "04")
    j4 = H.job_paths(sp, "04")
    j4["before"].write_bytes(b"an earlier copy\n")
    code, log, calls = run_job(stub, j4["script"])
    record("H7 an existing preserved copy is never overwritten",
           code == 1 and "exists" in log and not calls and j4["before"].read_bytes() == b"an earlier copy\n", log.strip())
    helper("script", *common, "--attempt", "05")
    j5 = H.job_paths(sp, "05")
    code, log, calls = run_job(stub, j5["script"], STUB_CLIPPY_FAIL="1")
    record("H8 a Clippy failure fails the job and keeps both copies",
           code == 101 and len(calls) == 3 and j5["before"].read_bytes() == b"relinked by someone else\n"
           and j5["after"].read_text() == STUB_BINARY and not j5["sums"].exists(), f"exit {code}")
    (receipts / f"{H.STAGE}-build06").mkdir(parents=True)
    code, out = helper("script", *common, "--attempt", "06")
    record("H9 an attempt whose guard receipt exists is refused", code == 1 and "new attempt number" in out, out.strip())

    # --- sources: consistent forgeries
    stage08 = EL / "combined-repair-stage08/candidate" / H.CRATE
    source_case("F0 positive control: an unmutated resealed copy passes", "f0", lambda c, s: None, None)
    source_case("F1 the stage08 script is refused", "f1",
                lambda c, s: shutil.copy2(stage08 / "script.tsv", c / "script.tsv"), "superseded stage08 script")

    def revert_04_4(c, s):
        replace_text(c / "script.tsv", H.WORDING["04_4"][1], H.WORDING["04_4"][2])
        return {"receipt": {"script_sha256": H.sha(c / "script.tsv")}, "pins": {"script": H.sha(c / "script.tsv")}}
    source_case("F2 a script whose 04_4 lost the reviewed words is refused (script pin moved with it)", "f2",
                revert_04_4, "04_4 text is not the reviewed wording")

    def old_08_5_words(c, s):
        append(c / "script.tsv", "# Halving the lost lap\n")
        return {"receipt": {"script_sha256": H.sha(c / "script.tsv")}, "pins": {"script": H.sha(c / "script.tsv")}}
    source_case("F3 the old 08_5 words anywhere in the script are refused", "f3", old_08_5_words,
                "stage08 wording 'Halving the lost lap'")
    source_case("F4 the stage08 units are refused", "f4",
                lambda c, s: shutil.copy2(stage08 / "narration_units.json", c / "narration_units.json"),
                "superseded stage08 units")

    def units_map(path, digest):
        def mutate(c, s):
            units = json.loads((c / "narration_units.json").read_text())
            units["direction_contract"]["map"] = {"path": f"{H.CRATE}/docs/{path}", "sha256": digest}
            (c / "narration_units.json").write_text(json.dumps(units, indent=2) + "\n")
            return {"pins": {"units": H.sha(c / "narration_units.json")}}
        return mutate
    source_case("F5 units bound to the oliver15 map are refused (units pin moved with them)", "f5",
                units_map("narration-direction-map-oliver15.json", H.STAGE08["map15"]), "expected oliver16")
    source_case("F6 units naming oliver16 with oliver15's bytes are refused", "f6",
                units_map("narration-direction-map-oliver16.json", H.STAGE08["map15"]), "expected oliver16")
    source_case("F7 the stage08 product_rule.rs is refused", "f7",
                lambda c, s: shutil.copy2(stage08 / "src/chapters/product_rule.rs", c / "src/chapters/product_rule.rs"),
                "product_rule.rs is not the stage09 bytes")
    source_case("F8 a stray unsealed file is refused", "f8",
                lambda c, s: ((c / ".ee").mkdir(), (c / ".ee/hook-state").write_text("{}\n")),
                "differ from White's seal", seal=False)
    source_case("F9 an unsealed byte change in the bound plan is refused", "f9",
                lambda c, s: append(s / "bound-plan/hold-audit.json", " "), "differ from White's seal", seal=False)
    source_case("F10 a changed Cargo.lock is refused", "f10",
                lambda c, s: append(c / "Cargo.lock", "\n"), "not the stage08 originals")
    source_case("F11 a receipt that changed other cues is refused", "f11", lambda c, s: None,
                "changed_cues", receipt_edits={"changed_cues": ["04_4"]})
    source_case("F12 a changed neighbour chapter passes the seal but is refused against stage08", "f12",
                lambda c, s: append(c / "src/chapters/slide_rule.rs", "// drift\n"),
                "src/chapters/slide_rule.rs", engine=True)

    def tamper_seal(c, s):
        sums = s.parents[1] / "white/SHA256SUMS"
        sums.write_text(sums.read_text().replace("  HANDOFF.md", "  HANDOFF.txt"))
    source_case("F13 a tampered White seal is refused", "f13", tamper_seal, "White's sealed bundle", seal=False)

    # --- engine: copies of W with their own pins
    engine_case("E0 positive control: a copy of W passes with its own pins", "e0", lambda fw, rows: None, None)
    engine_case("E1 the secondary stack's frame_table_limits.rs in W/engine is refused", "e1",
                lambda fw, rows: ((fw / "engine/crates/fmn-render/tests/frame_table_limits.rs").write_text("// S\n"),
                                  row_update(fw, rows, "crates/fmn-render/tests/frame_table_limits.rs", add=True)),
                "carries the secondary stack", engine_files=H.ENGINE_FILES + 1)
    engine_case("E2 a patched fmn rendering/pipeline.rs is refused", "e2",
                lambda fw, rows: (append(fw / "engine/crates/fmn/src/rendering/pipeline.rs", "// S\n"),
                                  row_update(fw, rows, "crates/fmn/src/rendering/pipeline.rs")),
                "is not the unpatched file")
    engine_case("E3 an engine file changed since stage08 is refused", "e3",
                lambda fw, rows: append(fw / "engine/crates/fmn-core/Cargo.toml", "\n"), "changed since the stage08 build")
    engine_case("E4 an engine file added since stage08 is refused", "e4",
                lambda fw, rows: (fw / "engine/crates/fmn/tests/slow_sink_budget_refusal.rs").write_text("// S\n"),
                "added ['crates/fmn/tests/slow_sink_budget_refusal.rs']")

    def budget_elsewhere(fw, rows):
        replace_text(fw / "engine-budget.toml", f'{fw}/engine/crates/fmn-render"', '/elsewhere/engine/crates/fmn-render"')
    engine_case("E5 a budget that sends one crate elsewhere is refused", "e5", budget_elsewhere,
                "fmn-render is not W's engine copy")
    engine_case("E6 a changed W/logarithm-stage08 is refused", "e6",
                lambda fw, rows: append(fw / "logarithm-stage08/src/main.rs", "\n"),
                "no longer matches root's stage08 binding")
    engine_case("E7 a changed repo-view file is refused", "e7",
                lambda fw, rows: append(next((fw / "repo-view").rglob("*.rs")), "\n"), "W/repo-view changed")
    fw8, _ = fake_w("e8")
    append(fw8 / "Cargo.lock.pinned", "\n")
    p8 = paths(w=fw8)
    with patched(H.W_PIN, **{k: v for k, v in repin(fw8).items() if k != "Cargo.lock.pinned"}):
        expect_refusal("E8 an unpinned change to W/Cargo.lock.pinned is refused",
                       lambda: H.check_engine(p8, H.check_sources(p8)["crate"]), "W/Cargo.lock.pinned")
    code, out = helper("install", "--w", fw8, "--into", fw8 / "engine")
    record("E9 an --into inside W/engine is refused", code == 1 and "is inside" in out, out.strip())

    after = snapshot()
    moved = sorted(k for k in set(before) | set(after) if before.get(k) != after.get(k))
    record("U1 the real EL stage09, White's seal, W's engine, repo-view, stage08 and pins, and the shared "
           "logarithm binary are untouched; no stage09 file in W", not moved, str(moved[:3]))
    passed = sum(RESULTS)
    print(f"RESULT: {passed} PASS, {len(RESULTS) - passed} FAIL")
    return 0 if passed == len(RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main())
