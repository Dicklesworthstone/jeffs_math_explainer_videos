#!/usr/bin/env python3
"""Recovery for logarithm-stage09-ws-build01 (BlueSalmon Mail 4120, bead 87do.63). Root runs every step.

What happened (diagnose, read-only). Cargo names and fingerprints a path package relative to its workspace root.
W/logarithm-stage09 and WS/explainer_corpus/logarithm_mysteries are each their own workspace root with the same
48 bytes, the same dependencies (absolute W paths) and the same profile, flags and config, so ws-build01 resolved
to build01's own units in target/geometry-kit01 (test c1b3bb6b1a138a1e, bin 2e30d21d5db228a1). Freshness is by
content (checksum-freshness) over package-relative paths, and the bytes are equal; cargo's freshness record keeps
no CARGO_MANIFEST_DIR, although rustc's dep-info lists it (env-dep). Cargo therefore reported every unit fresh,
compiled nothing, rewrote only the uplifted .d from the new root, and left build01's 04:57 binary (187e119d,
topic W/logarithm-stage09) in place. Not RCH: the job sets RCH_SHIM_LOCAL_IDE=1 and the shim then runs local cargo.

The recovery (build-script). One guarded build whose CARGO_TARGET_DIR is a new folder of its own,
target/logarithm-stage09-ws-buildNN, so no unit compiled from another root can be found. The cargo lines are
build01's and ws-build01's, line for line. The job itself runs `artifacts` after Clippy, so a guard exit 0 now
means the compiled topic was proved. target/geometry-kit01 is not written, cleaned or moved: its logarithm units,
the shared binary and every ws-build01 file are bound inputs, so the guard proves they did not change.

    python3 -I -B recover_ws_build.py diagnose     --w W --ws WS                 (attempt 01, read-only)
    python3 -I -B recover_ws_build.py build-script --w W --ws WS --attempt 02    (then root's guard line)
    python3 -I -B recover_ws_build.py build-verify --w W --ws WS --attempt 02
    python3 -I -B recover_ws_build.py artifacts    --w W --ws WS --attempt 02    (run by the job; read-only)

The attempt keeps the sealed helper's names (WS/logarithm-stage09-ws-buildNN.sh, -inputs.json, -binary,
-binaries.sha256; lane job logarithm-stage09-ws-buildNN), so the sealed prepare_stage09_render.py build-verify
and bind accept the same attempt. build-verify here adds the compile and output-path proofs. Run from the
repository root. --receipts and --target-root exist for the checks' scratch copies; root leaves them at their
defaults. Exit 0 pass, 1 refused, 2 usage.
"""
import argparse
import datetime
import hashlib
import importlib.util
import json
import os
import re
import shlex
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
REPO = HERE.parents[5]
P_REL = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
RENDER_REL = f"{P_REL}/logarithm-stage09-render-preparation01/prepare_stage09_render.py"
RENDER_SHA = "9ddc2003aa666844214aace166343bb37c1d5baeb6b62abb252f517c634b5388"
RENDER_SUMS_SHA = "fa654b83b72ffb570fdaad851125c7bec2dcd7b17533300b5c984f2657cae5d3"
SHARED = "geometry-kit01"  # the target folder build01 and ws-build01 used
VERSION = "0.1.0"
DEP_INFO = re.compile(r"logarithm_mysteries(-[0-9a-f]{16})?\.d")  # rustc's dep-info for a logarithm_mysteries unit


def load_render():
    path = REPO / RENDER_REL
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if digest != RENDER_SHA:
        raise SystemExit(f"REFUSED: the sealed render helper is {digest[:12]}, expected {RENDER_SHA[:12]}")
    spec = importlib.util.spec_from_file_location("prepare_stage09_render", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


R = load_render()
N = R.N
Refused, need, sha, tree, mentions = R.Refused, R.need, R.sha, R.tree, R.mentions
STAGE, CRATE, CHAPTERS = R.STAGE, R.CRATE, R.CHAPTERS


def utc(ts):
    return datetime.datetime.fromtimestamp(ts, datetime.timezone.utc)


def when(text):
    return datetime.datetime.fromisoformat(text)


def target_dir(c, nn):
    return c.target / f"{STAGE}-ws-build{nn}"


def unit_line(c, verb):
    return f"{verb} {R.TOPIC} v{VERSION} ({c.crate})"


def rustc_dep_infos(root):
    """rustc's dep-info files of every logarithm_mysteries unit under root/debug, not cargo's uplifted .d."""
    found = []
    for parent, dirs, files in os.walk(root / "debug"):
        dirs[:] = sorted(d for d in dirs if d != "incremental")
        if Path(parent) == root / "debug":
            continue
        found += [Path(parent) / f for f in sorted(files) if DEP_INFO.fullmatch(f)]
    return found


def manifest_dirs(path):
    return re.findall(r"^# env-dep:CARGO_MANIFEST_DIR=(.*)$", path.read_text(errors="replace"), re.M)


def running_paths(log):
    return re.findall(r"^\s+Running unittests src/main\.rs \((.+)\)$", log, re.M)


def at_repo(path):
    path = Path(path)
    return path if path.is_absolute() else REPO / path


# ---------------------------------------------------------------- diagnose (attempt 01)

def diagnose(c):
    """Every fact the cache-reuse diagnosis rests on, read from root's ws-build01 and the shared target."""
    R.verify_ws(c)
    j = R.build_paths(c, "01")
    r = json.loads((j["receipt"] / "resource-receipt.json").read_text())
    need(R.receipt_ok(r) and r.get("command") == ["sh", str(j["script"])], "ws-build01's receipt is not a clean exit 0")
    started, finished = when(r["started"]), when(r["finished"])
    try:
        R.build_verify(c, "01")
        verdict = None
    except Refused as refusal:
        verdict = str(refusal)
    need(verdict is not None, "the sealed build-verify now accepts ws-build01: this diagnosis no longer applies")
    need(verdict == "the new binary does not name the workspace topic", f"build-verify refuses for another reason: {verdict}")
    log = (j["receipt"] / "command.log").read_text(errors="replace")
    compiled = re.findall(rf"^\s+(?:Compiling|Checking) {R.TOPIC} .*$", log, re.M)
    need(not compiled, f"ws-build01 compiled the crate: {compiled}")
    finished_lines = re.findall(r"^\s+Finished .*$", log, re.M)
    old_log = (c.lane / f"{STAGE}-build01" / "command.log").read_text(errors="replace")
    need(running_paths(log) == running_paths(old_log) and len(running_paths(log)) == 1,
         "ws-build01 did not run build01's test executable")
    shared = c.target / SHARED / "debug"
    sums = dict(reversed(line.split("  ", 1)) for line in j["sums"].read_text().splitlines())
    before, after = sums[j["before"].name], sums[j["after"].name]
    need(before == after == R.BUILD01_BINARY == sha(c.w / f"{STAGE}-build01-binary"),
         "the preserved and post-build binaries are not both build01's")
    data = j["after"].read_bytes()
    old_topic, ws_topic = mentions(data, c.native.stage), mentions(data, c.crate)
    need(old_topic > 0 and ws_topic == 0, "the post-build binary does not name only W/logarithm-stage09")
    units = {}
    for info in rustc_dep_infos(c.target / SHARED):
        unit = info.parent.parent
        outputs = [p for p in (unit / "out").iterdir() if p.is_file()]
        newest = max(utc(p.stat().st_mtime) for p in outputs)
        cargo_record = [p.read_bytes() for p in (unit / "fingerprint").glob("dep-*")]
        units[unit.name] = {"rustc_env_dep_CARGO_MANIFEST_DIR": manifest_dirs(info),
                            "newest_output_utc": newest.isoformat(timespec="seconds"),
                            "cargo_record_names_a_root": any(str(c.w).encode() in b or b"CARGO_MANIFEST_DIR" in b
                                                             for b in cargo_record)}
        need(manifest_dirs(info) == [str(c.native.stage)], f"unit {unit.name} was not compiled from W/logarithm-stage09")
        need(newest < started, f"unit {unit.name} wrote output during ws-build01")
        need(cargo_record and not units[unit.name]["cargo_record_names_a_root"],
             f"cargo's freshness record of {unit.name} names a root")
    need(len(units) == 4, f"expected build01's 4 logarithm units, found {sorted(units)}")
    need(running_paths(log)[0].split("/")[-1].rsplit("-", 1)[1] in units, "the test executable is not one of those units")
    binary_mtime = utc((shared / R.TOPIC).stat().st_mtime)
    uplift = shared / f"{R.TOPIC}.d"
    uplift_mtime = utc(uplift.stat().st_mtime)
    need(sha(shared / R.TOPIC) == R.BUILD01_BINARY and binary_mtime < started, "the shared binary is not build01's")
    need(started <= uplift_mtime <= finished, "the uplifted dep-info was not rewritten during ws-build01")
    need(str(c.crate / "src/main.rs") in uplift.read_text(errors="replace"), "the uplifted dep-info does not name the workspace")
    old_manifest, ws_manifest = (c.native.stage / "Cargo.toml").read_bytes(), (c.crate / "Cargo.toml").read_bytes()
    need(old_manifest == ws_manifest and b"\n[workspace]\n" in ws_manifest, "the two manifests are not identical workspace roots")
    return {
        "attempt": "01", "receipt": {"exit": r["exit"], "started": r["started"], "finished": r["finished"]},
        "sealed_build_verify": f"REFUSED: {verdict}",
        "log": {"compiling_or_checking_logarithm_mysteries": 0, "finished": [s.strip() for s in finished_lines],
                "test_executable": running_paths(log)[0], "same_as_build01": True},
        "binary": {"before": before, "after": after, "names_W_stage": old_topic, "names_workspace": ws_topic},
        "shared_units": units,
        "shared_binary_utc": binary_mtime.isoformat(timespec="seconds"),
        "uplifted_dep_info_utc": uplift_mtime.isoformat(timespec="seconds"),
        "identical_workspace_roots": True,
        "diagnosis": "cargo reused build01's units: same package-relative identity and content, and its freshness "
                     "record keeps no CARGO_MANIFEST_DIR; nothing was compiled",
    }


# ---------------------------------------------------------------- recovery build

def attempt_paths(c, nn):
    need(nn != "01", "attempt 01 ran (Mail 4120); the recovery is attempt 02 or later")
    j = R.build_paths(c, nn)
    j["target"] = target_dir(c, nn)
    return j


def preserved_rows(c):
    """Everything the recovery must leave alone, bound so the guard proves it did not change."""
    paths = [HERE, REPO / RENDER_REL.replace("prepare_stage09_render.py", "SHA256SUMS")]
    old = R.build_paths(c, "01")
    paths += [old[k] for k in ("script", "inputs", "before", "after", "sums") if old[k].is_file()]
    shared = c.target / SHARED / "debug"
    for name in (R.TOPIC, f"{R.TOPIC}.d"):
        if (shared / name).is_file():
            paths.append(shared / name)
    units = shared / "build" / R.TOPIC
    if units.is_dir():
        for parent, dirs, files in os.walk(units):
            dirs.sort()
            paths += [Path(parent) / f for f in sorted(files)]
    return paths


def build_text(c, j, nn):
    q = shlex.quote
    lines = [
        f"# {j['job']}: recovery of logarithm-stage09-ws-build01 (Mail 4120), whose cargo reused build01's units in",
        f"# target/{SHARED} and left the old binary. This attempt builds in its own new target, so every unit is",
        f"# compiled from WS/{CRATE}. Written by recover_ws_build.py (sha256 {sha(HERE)}).",
        "# Same engine (W/engine-budget.toml) and cargo lines as build01 and ws-build01; launch only through the guard.",
        "set -eu",
        'refuse() { echo "REFUSED: $1" >&2; exit 1; }',
        f"W={q(str(c.w))}",
        f"WS={q(str(c.ws))}",
        f"R={q(str(REPO))}",
        f"T={q(str(c.target))}",
        '[ "${CARGO_TARGET_DIR:-}" = "$T" ] || refuse "CARGO_TARGET_DIR is not the guard\'s target"',
        "export CARGO_INCREMENTAL=0",
        "export RCH_SHIM_LOCAL_IDE=1",
        f'export CARGO_TARGET_DIR="$CARGO_TARGET_DIR/{j["target"].name}"',
        '[ ! -e "$CARGO_TARGET_DIR" ] || refuse "$CARGO_TARGET_DIR exists: this attempt needs a new target"',
        'cd "$R"',
        f'M="$WS/{CRATE}/Cargo.toml"',
        'B="$CARGO_TARGET_DIR/debug/logarithm_mysteries"',
        f'AFTER="$WS/{j["after"].name}"',
        f'SUMS="$WS/{j["sums"].name}"',
        f'for f in "$WS/{j["before"].name}" "$AFTER" "$SUMS"; do [ ! -e "$f" ] || refuse "$f exists"; done',
        f'python3 -I -B "$R/{N.LOCK_CHECK_REL}" "$W/{STAGE}-original-Cargo.lock" "$WS/{CRATE}/Cargo.lock"',
        'cargo test --locked --offline --config "$W/engine-budget.toml" --manifest-path "$M" --all-targets',
        'cargo build --locked --offline --config "$W/engine-budget.toml" --manifest-path "$M" --bin logarithm_mysteries',
        'cp -p "$B" "$AFTER"',
        'cargo clippy --locked --offline --config "$W/engine-budget.toml" --manifest-path "$M" --all-targets --no-deps -- -D warnings',
        'cmp "$B" "$AFTER" || refuse "Clippy changed the new binary"',
        f'(cd "$WS" && shasum -a 256 {q(j["after"].name)}) > "$SUMS"',
        f'python3 -I -B "$R/{HERE.relative_to(REPO)}" artifacts --w "$W" --ws "$WS" --attempt {nn} --target-root "$T"',
        '"$AFTER" chapters',
    ]
    return "\n".join(lines) + "\n"


def build_script(c, nn):
    j = attempt_paths(c, nn)
    R.check(c)
    R.verify_ws(c)
    pinned_sums = REPO / RENDER_REL.replace("prepare_stage09_render.py", "SHA256SUMS")
    need(sha(pinned_sums) == RENDER_SUMS_SHA, "the render preparation seal changed")
    for key in ("script", "inputs", "before", "after", "sums", "receipt", "target"):
        need(not j[key].exists() and not j[key].is_symlink(), f"{j[key]} exists: use a new attempt number")
    R.write_new(j["script"], build_text(c, j, nn))
    rows = R.build_rows(c, j)
    extra = preserved_rows(c)
    listed = {Path(r["path"]) for r in rows}
    need(not listed.intersection(extra) and len(set(extra)) == len(extra), "an input row repeats")
    rows += [{"path": str(p), "sha256": sha(p)} for p in extra]
    R.write_new(j["inputs"], json.dumps({"files": rows}, indent=2) + "\n")
    shared = c.target / SHARED / "debug"
    return {"job": j["job"], "script_sha256": sha(j["script"]), "inputs_sha256": sha(j["inputs"]), "input_rows": len(rows),
            "new_target": str(j["target"]), "preserved_rows": len(extra),
            "shared_binary": sha(shared / R.TOPIC) if (shared / R.TOPIC).is_file() else "absent",
            "launch_from_repo_root": f"python3 -I -B {N.GUARD_REL} --phase native --job {j['job']} "
                                     f"--inputs {j['inputs']} -- sh {j['script']}"}


def artifacts(c, nn):
    """The compiled topic, from the new target's own files: binary, rustc dep-info and the unit the binary came from."""
    R.verify_ws(c)
    j = attempt_paths(c, nn)
    td = j["target"]
    need(td.is_dir() and not td.is_symlink(), f"{td} is not this attempt's target")
    binary = td / "debug" / R.TOPIC
    need(binary.is_file() and not binary.is_symlink(), f"{binary} is not a file")
    data = binary.read_bytes()
    digest = hashlib.sha256(data).hexdigest()
    need(digest != R.BUILD01_BINARY, "the new target's binary is build01's")
    topic, old = mentions(data, c.crate), mentions(data, c.native.stage)
    need(topic > 0, "the new binary does not name the workspace topic")
    need(old == 0, f"the new binary still names W/logarithm-stage09 ({old} times)")
    infos = rustc_dep_infos(td)
    need(infos, "no rustc dep-info for a logarithm_mysteries unit in the new target")
    for info in infos:
        need(manifest_dirs(info) == [str(c.crate)],
             f"{info.relative_to(td)} was compiled with CARGO_MANIFEST_DIR {manifest_dirs(info)}")
    executables = [i.with_suffix("") for i in infos if i.with_suffix("").is_file()]
    source = [e for e in executables if sha(e) == digest]
    need(source, "the binary is not the output of a unit compiled from the workspace")
    uplift = (td / "debug" / f"{R.TOPIC}.d").read_text(errors="replace")
    need(str(c.crate / "src/main.rs") in uplift and mentions(uplift.encode(), c.native.stage) == 0,
         "cargo's uplifted dep-info does not name only the workspace sources")
    if j["after"].exists():
        need(sha(j["after"]) == digest, "the workspace copy is not the new target's binary")
    return {"attempt": nn, "target": str(td), "binary": str(binary), "binary_sha256": digest,
            "workspace_topic_mentions": topic, "W_stage_mentions": 0, "units": len(infos),
            "binary_unit": str(source[0].parent.parent.relative_to(td)),
            "compiled_with": f"CARGO_MANIFEST_DIR={c.crate} in every unit"}


def build_verify(c, nn):
    sealed = R.build_verify(c, nn)  # receipt, inputs, lock, 241 tests, chapters, sums, topic: the sealed checks
    j = attempt_paths(c, nn)
    rows = json.loads(j["inputs"].read_text())["files"]
    need({"path": str(HERE), "sha256": sha(HERE)} in rows, "the job ran another version of this helper")
    log = (j["receipt"] / "command.log").read_text(errors="replace")
    lines = [line.strip() for line in log.splitlines()]
    finished = [i for i, line in enumerate(lines) if line.startswith("Finished `")]
    profiles = [re.match(r"Finished `(\w+)`", lines[i]).group(1) for i in finished]
    need(profiles == ["test", "dev", "dev"], f"expected test, build and Clippy to finish once each: {profiles}")
    phases = {"test": lines[:finished[0]], "build": lines[finished[0] + 1:finished[1]],
              "clippy": lines[finished[1] + 1:finished[2]]}
    compiled = {name: sum(line == unit_line(c, "Checking" if name == "clippy" else "Compiling") for line in seg)
                for name, seg in phases.items()}
    need(compiled == {"test": 1, "build": 1, "clippy": 1}, f"the workspace crate was not compiled in every phase: {compiled}")
    other = [line for line in lines if re.search(rf"\b{R.TOPIC} v\d", line)
             and line not in (unit_line(c, "Compiling"), unit_line(c, "Checking"))]
    need(not other, f"cargo names another logarithm_mysteries: {other[:2]}")
    running = running_paths(log)
    need(len(running) == 1, f"expected one test executable, found {len(running)}")
    tests = at_repo(running[0])
    need(tests.resolve().is_relative_to((j["target"] / "debug").resolve()), "the tests did not run from the new target")
    exe = tests.read_bytes()
    need(mentions(exe, c.crate) > 0 and mentions(exe, c.native.stage) == 0, "the test executable does not name only the workspace")
    need('"artifacts": "PASS"' in log, "the job did not prove the compiled topic before listing the chapters")
    proof = artifacts(c, nn)
    need(proof["binary_sha256"] == sealed["binary_sha256"], "the target binary is not the verified workspace copy")
    return {**sealed, "compiled": {k: f"{v} line {unit_line(c, 'Checking' if k == 'clippy' else 'Compiling')}"
                                   for k, v in compiled.items()},
            "test_executable": str(tests), "artifacts": proof}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=("diagnose", "build-script", "build-verify", "artifacts"))
    parser.add_argument("--w", required=True)
    parser.add_argument("--ws", required=True)
    parser.add_argument("--attempt")
    for hidden in ("--receipts", "--target-root"):
        parser.add_argument(hidden, help="checks only: a scratch copy in place of the default")
    a = parser.parse_args(argv)
    if a.command != "diagnose" and not re.fullmatch(r"\d\d", a.attempt or ""):
        parser.error(f"{a.command} needs --attempt NN")
    try:
        c = R.Ctx(a)
        if a.command == "diagnose":
            result = {"diagnose": "PASS", **diagnose(c)}
        elif a.command == "build-script":
            result = {"build_script": "WRITTEN", **build_script(c, a.attempt)}
        elif a.command == "artifacts":
            result = {"artifacts": "PASS", **artifacts(c, a.attempt)}
        else:
            result = {"build_verify": "PASS", **build_verify(c, a.attempt)}
    except Refused as refusal:
        print(f"REFUSED: {refusal}")
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
