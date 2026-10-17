#!/usr/bin/env python3
"""Exact-cut status and guarded native Beads closure; never synthesizes or renders."""
from __future__ import annotations

import argparse
import contextlib
import fcntl
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import time

_spec = importlib.util.spec_from_file_location("video_evidence", Path(__file__).with_name("video_evidence.py"))
evidence = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(evidence)
catalog = evidence.catalog
GATE, PROVIDER = "exact_cut_receipts", "video-artifacts"
LABEL = "video:exact-cut"
POLICY_PATH = "docs/video-program/gate-policy.json"
LOCK_ENV = "VIDEO_PRODUCER_LOCK_FD"
STATES = ("open", "in_progress", "blocked", "deferred")
NATIVE_POLICY = {"workflow": {"strict": True, "gates": {
    f"{state} -> closed": {"require_if": [{"label": LABEL, "gate": GATE}]} for state in STATES
}}}
SUBJECT = {"type": "string", "minLength": 1}
POLICY_SCHEMA = {
    "type": "object", "required": ["schema_version", "profiles", "bindings"],
    "additionalProperties": False, "properties": {
        "schema_version": {"const": 1},
        "profiles": {"type": "object", "additionalProperties": {
            "type": "object", "required": ["gate", "checker_sha256", "scope", "qualification_ref"],
            "additionalProperties": False, "properties": {
                "gate": {"enum": list(evidence.POLICY)},
                "checker_sha256": {"type": "string", "pattern": "^[a-f0-9]{64}$"},
                "scope": SUBJECT, "qualification_ref": SUBJECT}}},
        "bindings": {"type": "array", "items": {
            "type": "object", "required": ["issue", "video_id", "revision_id", "voice", "checks"],
            "additionalProperties": False, "properties": {
                "issue": SUBJECT, "video_id": SUBJECT, "revision_id": SUBJECT, "voice": SUBJECT,
                "checks": {"type": "array", "minItems": 1, "items": {
                    "type": "object", "required": ["profile", "receipt"], "additionalProperties": False,
                    "properties": {"profile": SUBJECT, "receipt": SUBJECT}}}}}}
    }
}


def br(root, *args):
    tracker = (root / ".beads").resolve(strict=True)
    if not tracker.is_relative_to(root.resolve()):
        raise ValueError("refusing tracker outside the selected local repository")
    env = os.environ.copy()
    for key in ("BEADS_DB", "BEADS_JSONL"):
        env.pop(key, None)
    env["BEADS_DIR"] = str(tracker)
    result = subprocess.run(["br", *args, "--json", "--actor", PROVIDER], cwd=root,
                            env=env, capture_output=True, text=True, timeout=45)
    if result.returncode:
        raise ValueError(f"br {' '.join(args[:2])}: {result.stdout.strip()} {result.stderr.strip()}")
    return json.loads(result.stdout)


def issue_state(root, issue):
    result = br(root, "show", issue)
    return result[0] if isinstance(result, list) else result


def read_policy(root):
    policy = catalog.read_json(evidence.local_file(root, POLICY_PATH))
    errors = evidence._schema_errors(policy, POLICY_SCHEMA)
    if errors:
        raise ValueError("invalid gate policy: " + "; ".join(errors))
    seen = set()
    for binding in policy["bindings"]:
        if binding["issue"] in seen:
            raise ValueError("duplicate issue binding: " + binding["issue"])
        seen.add(binding["issue"])
        profiles = [check["profile"] for check in binding["checks"]]
        if len(profiles) != len(set(profiles)):
            raise ValueError("duplicate required profile: " + binding["issue"])
        for check in binding["checks"]:
            if check["profile"] not in policy["profiles"] or not catalog.safe_path(check["receipt"]):
                raise ValueError("unknown checker profile or unsafe receipt path")
    return policy


def assess(root, data, policy, binding):
    reasons, checks = [], []
    subject = {k: binding[k] for k in ("video_id", "revision_id", "voice")}
    for check in binding["checks"]:
        profile = policy["profiles"][check["profile"]]
        row = {**check, "gate": profile["gate"], "scope": profile["scope"], "reasons": []}
        try:
            receipt_path = evidence.local_file(root, check["receipt"])
            receipt = catalog.read_json(receipt_path)
            if not isinstance(receipt, dict):
                raise ValueError("receipt must be a JSON object")
            result = evidence.evaluate(receipt, data, root, subject["video_id"], subject["revision_id"], profile["gate"])
            row["reasons"].extend(result["reasons"])
            if receipt.get("subject") != subject:
                row["reasons"].append("receipt voice/subject differs from task binding")
            if catalog.digest(receipt.get("checker")) != profile["checker_sha256"]:
                row["reasons"].append("checker recipe is not the qualified pinned profile")
            qualification = evidence.local_file(root, profile["qualification_ref"])
            row["qualification_sha256"] = evidence.file_digest(qualification)
            row["receipt_sha256"] = evidence.file_digest(receipt_path)
        except (OSError, ValueError) as error:
            row["reasons"].append(str(error))
        row["state"] = "blocked" if row["reasons"] else "current_pass"
        reasons.extend(f"{check['profile']}: {reason}" for reason in row["reasons"])
        checks.append(row)
    return {"issue": binding["issue"], "subject": subject, "checks": checks,
            "state": "blocked" if reasons else "ready_for_task_transition", "reasons": reasons}


def load(root):
    data = catalog.read_json(evidence.local_file(root, "docs/video-program/catalog.json"))
    errors = catalog.validate(data)
    if errors:
        raise ValueError("invalid catalog: " + "; ".join(errors))
    return data, read_policy(root)


def status(root, video_id, revision_id=None, voice=None):
    data, policy = load(root)
    video = next((v for v in data["videos"] if v["video_id"] == video_id), None)
    if video is None:
        raise ValueError("unknown catalog video: " + video_id)
    revisions = [r for r in video["revisions"] if (not revision_id or r["revision_id"] == revision_id)
                 and (not voice or r["voice"] == voice)]
    if (revision_id or voice) and not revisions:
        raise ValueError("no cut matches the explicit revision/voice; no voice fallback")
    rows = []
    for revision in revisions:
        rid = revision["revision_id"]
        bindings = [b for b in policy["bindings"] if b["video_id"] == video_id and b["revision_id"] == rid]
        tasks = [assess(root, data, policy, b) for b in bindings]
        for task in tasks:
            try:
                issue = issue_state(root, task["issue"])
                task["task_status"] = issue["status"]
                if LABEL not in issue.get("labels", []):
                    task["reasons"].append("native exact-cut label missing")
                    task["state"] = "blocked"
                if issue["status"] in STATES:
                    native_contract(root, task["issue"])
            except (OSError, ValueError, subprocess.TimeoutExpired) as error:
                task["task_status"] = "unknown"
                task["reasons"].append(str(error))
                task["state"] = "blocked"
        blocker = next((reason for t in tasks for reason in t["reasons"]), None)
        if not tasks:
            blocker = "No qualified checker/receipt binding recorded for this cut; historical markers cannot qualify it."
        rows.append({"revision_id": rid, "voice": revision["voice"], "tasks": tasks,
                     "state": "blocked" if blocker else "receipts_current",
                     "next_blocker": blocker,
                     "publications": [{"venue": p["venue"], "external_id": p["external_id"]}
                                      for p in video["publications"] if p["revision_id"] == rid]})
    return {"video_id": video_id, "revisions": rows,
            "next_blocker": None if rows else "No cut recorded; select and record a revision before producing evidence.",
            "authority": "Current exact-cut receipts only; historical publication and task closure are not new-cut approval."}


@contextlib.contextmanager
def reservation(root):
    lock_path = root / ".x/producer.lock"
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    inherited = os.environ.get(LOCK_ENV)
    if inherited is not None:
        fd = int(inherited)
        if os.fstat(fd).st_ino != lock_path.stat().st_ino or os.fstat(fd).st_dev != lock_path.stat().st_dev:
            raise ValueError("invalid inherited producer reservation")
        # A valid inherited descriptor must also hold the lock. flock is per
        # open-file description, so checking this never releases the parent's.
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield fd
        return
    with lock_path.open("a+") as lock:
        deadline = time.monotonic() + 30
        while True:
            try:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                if time.monotonic() >= deadline:
                    raise ValueError("another producer holds .x/producer.lock; retry after it finishes")
                time.sleep(0.1)
        yield lock.fileno()


def native_contract(root, issue):
    # JSON is valid YAML. Requiring this small unambiguous subset avoids a
    # second parser/dependency and catches strict:false (gate list alone does not).
    policy = catalog.read_json(evidence.local_file(root, ".beads/policy.yaml"))
    if policy.get("workflow", {}).get("strict") is not True:
        raise ValueError("native workflow.strict must be true; listed gates alone are advisory")
    current = issue_state(root, issue)
    if current["status"] not in STATES or LABEL not in current.get("labels", []):
        raise ValueError("task must be open/in_progress/blocked/deferred and carry " + LABEL)
    transitions = br(root, "gate", "list", issue)["gated_transitions"]
    edge = next((t for t in transitions if t["from"] == current["status"] and t["to"] == "closed"), None)
    if not edge or GATE not in [g["gate"] for g in edge["gates"]]:
        raise ValueError("native policy does not require exact-cut receipts on this transition")
    return current["status"]


def checked_binding(root, video_id, revision_id, voice, issue):
    data, policy = load(root)
    binding = next((b for b in policy["bindings"] if b["issue"] == issue), None)
    if binding is None:
        raise ValueError("no qualified receipt binding for task " + issue)
    if binding["video_id"] != video_id or binding["revision_id"] != revision_id or (voice and binding["voice"] != voice):
        raise ValueError("requested video/revision/voice differs from task binding")
    result = assess(root, data, policy, binding)
    if result["reasons"]:
        raise ValueError("; ".join(result["reasons"]))
    # Bind policy, receipt and selected inputs. Unrelated catalog edits do not
    # invalidate scoped receipts, but any policy edit during closure does.
    result["policy_sha256"] = catalog.digest(policy)
    result["native_policy_sha256"] = evidence.file_digest(root / ".beads/policy.yaml")
    result["adapter_sha256"] = evidence.file_digest(Path(__file__))
    return result


def report(root, issue, passed, note):
    return br(root, "gate", "report", issue, "--gate", GATE, "--provider", PROVIDER,
              "--status", "pass" if passed else "fail", "--to", "closed", "--note", note)


def close_task(root, video_id, revision_id, voice, issue):
    with reservation(root):
        initial_status = native_contract(root, issue)
        # Revoke this provider's old verdict BEFORE attempting revalidation.
        report(root, issue, False, "Exact-cut validation started; old provider pass revoked.")
        try:
            result = checked_binding(root, video_id, revision_id, voice, issue)
            note = "Exact-cut receipt validation (not publication authority): " + json.dumps(result, sort_keys=True)
            br(root, "comments", "add", issue, note)
            if native_contract(root, issue) != initial_status or checked_binding(root, video_id, revision_id, voice, issue) != result:
                raise ValueError("task or exact-cut evidence changed before native report")
            report(root, issue, True, note)
            if native_contract(root, issue) != initial_status or checked_binding(root, video_id, revision_id, voice, issue) != result:
                raise ValueError("task or exact-cut evidence changed before transition")
            br(root, "close", issue, "--reason", "Exact-cut receipts freshly validated; see portable evidence comment.")
            if issue_state(root, issue)["status"] != "closed":
                raise ValueError("native closure was not confirmed")
            return {"issue": issue, "status": "closed", "evidence": result}
        except BaseException:
            # Native CLI has no transaction spanning media hashing and closure.
            # Fail closed on handled failure; SIGKILL/power loss remains a stated
            # limitation, and subsequent attempts always revoke/revalidate.
            if issue_state(root, issue)["status"] == initial_status:
                report(root, issue, False, "Exact-cut closure aborted; revalidation required.")
            raise


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=catalog.ROOT)
    sub = parser.add_subparsers(dest="action", required=True)
    for action in ("status", "close"):
        p = sub.add_parser(action)
        p.add_argument("--video", required=True)
        p.add_argument("--revision", required=action == "close")
        p.add_argument("--voice")
        if action == "close": p.add_argument("--issue", required=True)
        else: p.add_argument("--porcelain", action="store_true")
    reserve = sub.add_parser("reserve")
    reserve.add_argument("command", nargs=argparse.REMAINDER)
    sub.add_parser("verify-reservation")
    args = parser.parse_args(argv)
    try:
        root = args.root.resolve(strict=True)
        if args.action == "verify-reservation":
            if LOCK_ENV not in os.environ: raise ValueError("no inherited producer reservation")
            with reservation(root):
                return 0
        if args.action == "reserve":
            command = args.command[1:] if args.command[:1] == ["--"] else args.command
            if not command: raise ValueError("reserve requires a producer command")
            with reservation(root) as fd:
                env = {**os.environ, LOCK_ENV: str(fd)}
                return subprocess.run(command, env=env, pass_fds=(fd,)).returncode
        if args.action == "close":
            result = close_task(root, args.video, args.revision, args.voice, args.issue)
        else:
            result = status(root, args.video, args.revision, args.voice)
            if not args.porcelain:
                print(f"{args.video}: exact-cut evidence (legacy .x marks are not authority)")
                for row in result["revisions"]:
                    print(f"  {row['revision_id']} / {row['voice']}: {row['state']}")
                    if row["next_blocker"]: print("    next: " + row["next_blocker"])
                    for task in row["tasks"]:
                        print(f"    {task['issue']} [{task['task_status']}]: {task['state']}")
                if result["next_blocker"]: print("  next: " + result["next_blocker"])
                return 0
        print(json.dumps(result, indent=2))
        return 0
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        print(json.dumps({"state": "blocked", "reasons": [str(error)]}, indent=2))
        return 1


if __name__ == "__main__":
    sys.exit(main())
