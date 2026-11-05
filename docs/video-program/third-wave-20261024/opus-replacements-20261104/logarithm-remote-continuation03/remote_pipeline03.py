#!/usr/bin/env python3
"""Logarithm binding, timeline and score on mac-mini-max, after rebind02 and assemble01 passed (bead 87do.63).

bind refused at the sealed check that searches the binary for the crate path. That search is not sound: Rust string
literals have no terminator, so the path can abut the next literal, and the check's lookahead then rejects a correct
binary. The path itself does matter. main.rs hands env!("CARGO_MANIFEST_DIR") to the kit as the topic, and the kit finds
tools/narration_delivery.py and script.tsv from it, so a wrong one would refuse every narrated job.

This file loads sealed remote_pipeline01.py by hash. Its gates() runs unchanged except for that one call, which now does
two things instead of searching bytes:
- ties the exact pinned binary to the stable build, its frozen script, sources and target output;
- runs that binary up to its first narrated call, with python3 a stub that records the call and refuses.

The binding gains the proof under checks, plus this helper's hash and two file rows. Its schema does not change. Run from
anywhere; the controller runs it. Commands are the sealed helper's. Technical gates only: nothing here was heard, viewed
or accepted, and no job is rebuilt or rerun.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve()
BUNDLE = HERE.parent
REPO = HERE.parents[5]
SEALED = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-remote-mac-mini-max01"
SEALED_SUMS_SHA = "26c0b341f877e6b492a4d1798e37ce11698c9d8282d759248cd9914dab2890d8"
SEALED_HELPER_SHA = "72a9dbed0da6380441a71795b76fe700d5199ac2cd7f66a63f6adbd524212941"
BINARY_SHA = "ae54bc176e67c9580b5110a6af3af3e0796974f357844544d0c10d8345246136"
BUILD_SCRIPT_SHA = "d95199232ea6f0f0217ac0caefb9119e6dc99d0c891ac3e2bb98d2cd7855a53b"
REBIND, ASSEMBLE = "logarithm-remote-rebind02", "logarithm-remote-assemble01"
# Lines of the frozen build script that make the proof: stop on failure, start clean, build this manifest, copy, compare.
BUILD_LINES = (
    "set -euo pipefail",
    'test "${CARGO_TARGET_DIR:-}" = "$O/target"',
    'test ! -e "$O/target"',
    'test ! -e "$O/logarithm_mysteries-release01"',
    'cargo build --release --locked --offline --config "$W/engine-budget.toml" --manifest-path "$C/Cargo.toml" --bin logarithm_mysteries',
    'cp -p "$CARGO_TARGET_DIR/release/logarithm_mysteries" "$O/logarithm_mysteries-release01"',
    'cmp "$CARGO_TARGET_DIR/release/logarithm_mysteries" "$O/logarithm_mysteries-release01"',
)


def load_sealed():
    sums, helper = SEALED / "SHA256SUMS", SEALED / "remote_pipeline01.py"
    if hashlib.sha256(sums.read_bytes()).hexdigest() != SEALED_SUMS_SHA:
        raise SystemExit("REFUSED: the sealed bundle's SHA256SUMS moved")
    if f"{SEALED_HELPER_SHA}  {helper.name}" not in sums.read_text().splitlines():
        raise SystemExit("REFUSED: the sealed bundle does not list this helper")
    if hashlib.sha256(helper.read_bytes()).hexdigest() != SEALED_HELPER_SHA:
        raise SystemExit("REFUSED: the sealed helper moved")
    spec = importlib.util.spec_from_file_location("remote_pipeline01", helper)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


P1 = load_sealed()
M, H = P1.M, P1.H
need, sha = M.need, M.sha
TARGET = P1.O / "target/release/logarithm_mysteries"
PROOF = {}


def rows_of(manifest):
    return {row["path"]: row["sha256"] for row in json.loads(manifest.read_text())["files"]}


def script_paths(text):
    """The script's own path assignments, resolved. Each may use only the ones above it."""
    values = {}
    for line in text.splitlines():
        match = re.fullmatch(r'([A-Z]+)="?([^"]+)"?', line)
        if match:
            values[match[1]] = re.sub(r"\$([A-Z]+)", lambda name: values[name[1]], match[2])
    return values


def compiled_topic():
    """The first narrated call of the real binary. It reads script.tsv from its compile-time topic and must match it
    byte for byte, then calls python3. The stub records that call and refuses, so nothing renders or is written."""
    want = ["-I", "-B", str(M.TOOLS / "narration_delivery.py"), "--topic", str(M.CRATE), "--narration", str(M.NARRATION),
            "--consumer", "render", "--gain-db", H.GAIN_DB]
    for key in M.CHAPTERS:
        want += ["--chapter-key", key]
    with tempfile.TemporaryDirectory() as folder:
        base = Path(folder).resolve()
        log, stub, unused = base / "argv.txt", base / "python3", base / "unused"
        stub.write_text(f'#!/bin/bash\nprintf "%s\\n" "$@" > "{log}"\nexit 9\n')
        stub.chmod(0o755)
        done = subprocess.run([str(M.BINARY), "render", "all", "--timeline-only", "--fps", "60", "--narration", str(M.NARRATION),
                               "--out", str(unused)], cwd=REPO, capture_output=True, text=True, timeout=300,
                              env={**os.environ, "PATH": f"{base}:{os.environ['PATH']}"})
        need(log.is_file(), f"the binary stopped before its delivery call: {done.stderr.strip()[-300:]}")
        found = log.read_text().splitlines()
        need(found == want, f"the binary's delivery call differs: {found[:6]}")
        need(done.returncode != 0 and not unused.exists(), "the binary went on past a refused delivery contract")
    return found


def build_proof(data, crate):
    """Stands where the sealed gates() calls H.mentions, whose `> 0` stays. Each gap refuses with its own reason.

    gates() has already required the build receipt and its frozen manifest, with every source row unmoved."""
    need(crate == M.CRATE, f"the proof was asked about {crate}")
    need(hashlib.sha256(data).hexdigest() == BINARY_SHA, f"{M.BINARY.name} is not the binary the build proved")
    need(TARGET.is_file() and sha(TARGET) == BINARY_SHA, "the build's target output is not this binary")
    receipt = M.finished(M.BUILD, P1.BUILD_SCRIPT)
    need(receipt["command"] == ["/bin/bash", str(P1.BUILD_SCRIPT)], f"the build ran {receipt['command']}")
    need(sha(P1.BUILD_SCRIPT) == BUILD_SCRIPT_SHA, "build01.sh is not the frozen build script")
    text = P1.BUILD_SCRIPT.read_text()
    lines = text.splitlines()
    gaps = [wanted for wanted in BUILD_LINES if not any(line.endswith(wanted) for line in lines)]
    need(not gaps, f"build01.sh lacks {gaps[:2]}")
    paths = script_paths(text)
    need(paths.get("C") == str(M.CRATE) and paths.get("O") == str(P1.O), f"build01.sh built {paths.get('C')}, not {M.CRATE}")
    frozen = rows_of(P1.BUILD_INPUTS)
    required = [P1.BUILD_SCRIPT, M.CRATE / "Cargo.toml", M.CRATE / "Cargo.lock", Path(paths["W"]) / "engine-budget.toml"]
    absent = [str(p) for p in required if frozen.get(str(p)) != sha(p)]
    need(not absent, f"the build's manifest does not bind {absent[:2]}")
    need(rows_of(P1.ASSEMBLE_INPUTS).get(str(M.BINARY)) == BINARY_SHA, "the assembly did not freeze this binary")
    argv = compiled_topic()
    PROOF.update({"binary_sha256": BINARY_SHA, "target": str(TARGET), "target_sha256": BINARY_SHA,
                  "build_job": M.BUILD, "build_receipt_sha256": sha(M.LANE / M.BUILD / "resource-receipt.json"),
                  "build_script_sha256": BUILD_SCRIPT_SHA, "build_inputs_sha256": receipt["inputs_sha256"],
                  "manifest_path": str(M.CRATE / "Cargo.toml"), "first_narrated_call": argv})
    return 1


def gates(a):
    body = SEALED_GATES(a)
    body["remote_helper_sha256"] = sha(HERE)
    body["checks"]["compiled_build"] = dict(PROOF)
    body["files"].update({str(p): sha(p) for p in (HERE, BUNDLE / "SHA256SUMS")})
    return body


def seal(digest):
    """This bundle against the hash root passes in, then sealed01's own seal, then the two jobs this run starts after."""
    sums = BUNDLE / "SHA256SUMS"
    need(sums.is_file() and sha(sums) == digest, f"this bundle's SHA256SUMS is not the seal {digest[:12]}")
    for line in sums.read_text().splitlines():
        expected, name = line.split(maxsplit=1)
        need(sha(BUNDLE / name.lstrip("*")) == expected, f"{name} differs from its seal")
    P1.seal(SEALED_SUMS_SHA)
    for job in (REBIND, ASSEMBLE):
        path = M.LANE / job / "resource-receipt.json"
        need(path.is_file() and H.receipt_ok(json.loads(path.read_text())), f"{job} has not passed; this run never starts it")
    return {"seal": "PASS"}


SEALED_GATES = P1.gates
H.mentions = build_proof
M.gates, M.seal = gates, seal

if __name__ == "__main__":
    sys.exit(M.main())
