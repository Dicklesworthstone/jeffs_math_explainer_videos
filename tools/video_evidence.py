#!/usr/bin/env python3
"""Content-bound checker receipts. No Beads or production gate writes.

All paths in catalog inputs, checker source pins and report output_refs must
resolve inside the repository. Checker stdout must be a single JSON object:
{"verdict":"pass|fail|unknown", "output_refs":["relative/file"], "summary":"..."}
An eligible receipt is one ingredient for A4's trusted gate policy, not upload
authority or a claim that an arbitrary checker measures audible quality.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time

from jsonschema import Draft202012Validator, FormatChecker

_spec = importlib.util.spec_from_file_location("video_catalog", Path(__file__).with_name("video_catalog.py"))
catalog = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(catalog)
RECEIPT_SCHEMA = catalog.ROOT / "docs/video-program/evidence-receipt.schema.json"

# Required roles are the positive-capability floor. Optional roles are still
# bound when present; adding/removing one invalidates that gate's receipt.
POLICY = {
    "script": (("script",), ("lexicon",)),
    "dry_voice": (("script", "voice", "take"), ("lexicon",)),
    "timing": (("script", "voice", "take", "cue"), ("lexicon",)),
    "visuals": (("script", "cue", "scene"), ("lexicon", "voice", "take")),
    "mix": (("script", "voice", "take", "cue", "scene", "master"), ("lexicon", "music")),
    "master": (("script", "voice", "take", "cue", "scene", "master"), ("lexicon", "music")),
    "package": (("script", "voice", "take", "cue", "scene", "master", "package"), ("lexicon", "music")),
    "public_metadata": (("script", "voice", "take", "cue", "scene", "master", "package"), ("lexicon", "music")),
}
REPORT_SCHEMA = {
    "type": "object", "properties": {
        "verdict": {"enum": ["pass", "fail", "unknown"]},
        "output_refs": {"type": "array", "uniqueItems": True, "items": {"type": "string", "minLength": 1}},
        "summary": {"type": "string", "minLength": 1},
    }, "required": ["verdict", "output_refs", "summary"], "additionalProperties": False,
}


def file_digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def local_file(root: Path, value: str) -> Path:
    if not catalog.safe_path(value):
        raise ValueError(f"unsafe repository-relative path: {value!r}")
    path = (root / value).resolve(strict=True)
    if not path.is_relative_to(root.resolve()) or not path.is_file():
        raise ValueError(f"not a regular file inside repository: {value!r}")
    return path


def source_pins(root: Path, files) -> dict:
    if not files:
        raise ValueError("at least one checker source file is required")
    return {p: file_digest(local_file(root, p)) for p in sorted(set(files))}


def executable_path(command, root: Path) -> Path:
    if os.sep in command[0]:
        candidate = Path(command[0])
        return (candidate if candidate.is_absolute() else root / candidate).resolve(strict=True)
    executable = shutil.which(command[0])
    if executable is None:
        candidate = Path(command[0])
        executable = str(candidate if candidate.is_absolute() else root / candidate)
    return Path(executable).resolve(strict=True)


def executable_digest(command, root: Path) -> str:
    return file_digest(executable_path(command, root))


def snapshot(data, root: Path, video_id: str, revision_id: str, gate: str) -> dict:
    errors = catalog.validate(data)
    if errors:
        raise ValueError("invalid catalog: " + "; ".join(errors))
    video = next((v for v in data["videos"] if v["video_id"] == video_id), None)
    revision = next((r for r in video["revisions"] if r["revision_id"] == revision_id), None) if video else None
    if revision is None:
        raise ValueError(f"unknown cut: {video_id}/{revision_id}")
    if gate not in POLICY:
        raise ValueError(f"unknown gate: {gate}")
    required, optional = POLICY[gate]
    refs = revision.get("artifact_refs", {})
    missing = [role for role in required if role not in refs]
    if missing:
        raise ValueError("unrecorded required roles: " + ", ".join(missing))
    bindings = {role: refs.get(role, []) for role in sorted(required + optional)}
    artifacts = {a["artifact_id"]: a for a in data.get("artifacts", [])}
    pending = [ref for ids in bindings.values() for ref in ids]
    records = {}
    while pending:
        ref = pending.pop()
        if ref in records:
            continue
        artifact = artifacts[ref]
        actual = file_digest(local_file(root, artifact["path"]))
        if actual != artifact["sha256"]:
            raise ValueError(f"artifact {ref}: current bytes differ from catalog SHA256")
        records[ref] = {k: artifact[k] for k in ("artifact_id", "kind", "path", "sha256", "inputs")}
        pending.extend(artifact["inputs"])
    return {"policy_version": 1, "gate": gate,
            "producer_sha256": file_digest(Path(__file__)),
            "subject": {"video_id": video_id, "revision_id": revision_id, "voice": revision["voice"]},
            "bindings": bindings, "artifacts": [records[key] for key in sorted(records)]}


def _schema_errors(value, schema):
    return [f"{'/'.join(map(str, e.absolute_path))}: {e.message}"
            for e in Draft202012Validator(schema, format_checker=FormatChecker()).iter_errors(value)]


def run_checker(data, root: Path, video_id: str, revision_id: str, gate: str,
                command: list[str], version: str, checker_files: list[str],
                reviewer: str, receipt_path: Path, *, advisory=False, timeout=60.0) -> dict:
    if not (0 < timeout <= 600) or not command or not version.strip() or not reviewer.strip():
        raise ValueError("require command, version, reviewer and a timeout in (0, 600]")
    inputs = snapshot(data, root, video_id, revision_id, gate)
    pins = source_pins(root, checker_files)
    executable = executable_digest(command, root)
    receipt_path.parent.mkdir(parents=True, exist_ok=True)
    # Reserve the receipt exclusively before execution. Interruption leaves an
    # invalid/empty record, never an old pass. Logs are retained beside it.
    with receipt_path.open("x", encoding="utf-8") as destination:
        stdout_path = receipt_path.with_name(receipt_path.name + ".stdout")
        stderr_path = receipt_path.with_name(receipt_path.name + ".stderr")
        problems, report, outputs, returncode = [], None, {}, None
        started = time.monotonic()
        with stdout_path.open("xb") as out, stderr_path.open("xb") as err:
            try:
                process = subprocess.Popen(command, executable=executable_path(command, root), cwd=root, stdin=subprocess.DEVNULL,
                                           stdout=out, stderr=err, start_new_session=True)
                while process.poll() is None:
                    if time.monotonic() - started > timeout or max(out.tell(), err.tell()) > 1_048_576:
                        problems.append("checker exceeded time or output budget")
                        os.killpg(process.pid, signal.SIGKILL)
                        break
                    time.sleep(0.02)
                returncode = process.wait()
            except OSError as error:
                problems.append(f"checker launch failed: {error}")
        stdout = stdout_path.read_bytes()[:1_048_577].decode("utf-8", errors="replace")
        stderr = stderr_path.read_bytes()[:1_048_577].decode("utf-8", errors="replace")
        if stdout_path.stat().st_size > 1_048_576 or stderr_path.stat().st_size > 1_048_576:
            problems.append("checker output exceeded 1 MiB; receipt contains bounded prefix")
        if returncode != 0:
            problems.append(f"checker exited {returncode}")
        try:
            report = json.loads(stdout, object_pairs_hook=catalog._unique_keys,
                                parse_constant=catalog._invalid_constant)
            report_errors = _schema_errors(report, REPORT_SCHEMA)
            if report_errors:
                problems.append("invalid checker report: " + "; ".join(report_errors))
                report = None
            else:
                outputs = {p: file_digest(local_file(root, p)) for p in report["output_refs"]}
        except (OSError, ValueError) as error:
            problems.append(f"missing/corrupt checker report or output: {error}")
            report = None
        try:
            if snapshot(data, root, video_id, revision_id, gate) != inputs:
                problems.append("input binding changed during checker")
            if source_pins(root, checker_files) != pins or executable_digest(command, root) != executable:
                problems.append("checker changed during execution")
        except (OSError, ValueError) as error:
            problems.append(f"post-check input verification failed: {error}")
        receipt = {
            "schema_version": 1, "gate": gate, "subject": inputs["subject"], "inputs": inputs,
            "checker": {"command": command, "version": version, "source_sha256": pins, "executable_sha256": executable},
            "recorded_at_utc": datetime.now(timezone.utc).isoformat(), "reviewer": reviewer,
            "classification": "advisory" if advisory else "required",
            "verdict": "fail" if problems else report["verdict"], "returncode": returncode,
            "duration_seconds": time.monotonic() - started, "report": report,
            "stdout": stdout, "stderr": stderr, "outputs": outputs, "problems": problems,
        }
        json.dump(receipt, destination, indent=2)
        destination.write("\n")
    return receipt


def evaluate(receipt, data, root: Path, video_id: str, revision_id: str, gate: str) -> dict:
    problems = _schema_errors(receipt, catalog.read_json(RECEIPT_SCHEMA))
    if problems:
        return {"eligible_for_required_gate": False, "state": "invalid", "reasons": problems}
    if receipt["classification"] != "required":
        problems.append("advisory evidence cannot satisfy a required gate")
    if receipt["verdict"] != "pass" or receipt["returncode"] != 0 or receipt["problems"]:
        problems.append("checker did not produce a clean passing result")
    report = receipt["report"]
    if _schema_errors(report, REPORT_SCHEMA) or report["verdict"] != "pass":
        problems.append("missing, invalid or nonpassing checker report")
    elif set(report["output_refs"]) != set(receipt["outputs"]):
        problems.append("checker output references disagree with receipt")
    try:
        original_report = json.loads(receipt["stdout"], object_pairs_hook=catalog._unique_keys,
                                     parse_constant=catalog._invalid_constant)
        if original_report != report:
            problems.append("embedded checker report disagrees with captured stdout")
    except ValueError:
        problems.append("captured checker stdout is not a structured report")
    try:
        current = snapshot(data, root, video_id, revision_id, gate)
        if receipt["subject"] != current["subject"] or receipt["gate"] != gate:
            problems.append("receipt subject, voice or gate does not match requested cut")
        if receipt["inputs"] != current:
            problems.append("scoped inputs or bindings changed")
        checker = receipt["checker"]
        if source_pins(root, checker["source_sha256"]) != checker["source_sha256"]:
            problems.append("dirty checker source bytes changed")
        if executable_digest(checker["command"], root) != checker["executable_sha256"]:
            problems.append("checker executable changed")
        for path, expected in receipt["outputs"].items():
            if file_digest(local_file(root, path)) != expected:
                problems.append(f"checker output changed: {path}")
    except (OSError, ValueError) as error:
        problems.append(str(error))
    return {"eligible_for_required_gate": not problems,
            "state": "current_pass" if not problems else "blocked", "reasons": problems,
            "authority": "Fresh receipt only. Trusted-checker policy and transition authority belong to A4."}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog", type=Path, default=catalog.CATALOG)
    parser.add_argument("--root", type=Path, default=catalog.ROOT)
    parser.add_argument("--video", required=True)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--gate", choices=POLICY, required=True)
    commands = parser.add_subparsers(dest="action", required=True)
    check = commands.add_parser("check")
    check.add_argument("receipt", type=Path)
    run = commands.add_parser("run")
    run.add_argument("--receipt", type=Path, required=True)
    run.add_argument("--checker-version", required=True)
    run.add_argument("--checker-file", action="append", required=True)
    run.add_argument("--reviewer", required=True)
    run.add_argument("--advisory", action="store_true")
    run.add_argument("--timeout", type=float, default=60)
    run.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    try:
        data = catalog.read_json(args.catalog)
        if args.action == "check":
            result = evaluate(catalog.read_json(args.receipt), data, args.root, args.video, args.revision, args.gate)
            success = result["eligible_for_required_gate"]
        else:
            command = args.command[1:] if args.command[:1] == ["--"] else args.command
            receipt = run_checker(data, args.root, args.video, args.revision, args.gate,
                                  command, args.checker_version, args.checker_file, args.reviewer,
                                  args.receipt, advisory=args.advisory, timeout=args.timeout)
            result = {"receipt": str(args.receipt), "verdict": receipt["verdict"], "problems": receipt["problems"]}
            success = receipt["verdict"] == "pass"
        print(json.dumps(result, indent=2))
        return 0 if success else 1
    except (OSError, ValueError) as error:
        print(json.dumps({"eligible_for_required_gate": False, "state": "blocked", "reasons": [str(error)]}, indent=2))
        return 1


if __name__ == "__main__":
    sys.exit(main())
