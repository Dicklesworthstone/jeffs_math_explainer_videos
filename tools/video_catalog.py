#!/usr/bin/env python3
"""Read-only catalog validation and exact-cut lineage inspection.

uv run --no-project --with jsonschema python3 -I tools/video_catalog.py validate
uv run --no-project --with jsonschema python3 -I tools/video_catalog.py show hoeffding_d
uv run --no-project --with jsonschema python3 -I tools/video_catalog.py board

Structural validity and recorded hashes are not fresh file verification or
publication authority. This tool never writes the catalog, Beads or gate state.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from collections import deque
from pathlib import Path, PurePosixPath

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "docs/video-program/catalog.schema.json"
CATALOG = ROOT / "docs/video-program/catalog.json"
ROLES = ("script", "lexicon", "voice", "take", "cue", "scene", "music", "master", "package")


def _unique_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _invalid_constant(value):
    raise ValueError(f"non-JSON numeric constant: {value}")


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=_unique_keys,
                      parse_constant=_invalid_constant)


def digest(value) -> str:
    """Deterministic data digest; deliberately preserves array order."""
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":"),
                         ensure_ascii=False, allow_nan=False).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def safe_path(value: str) -> bool:
    # Portable, normalized repository-relative spelling. No URI/drive, hidden
    # path traversal or backslash interpretation. Existence is a separate check.
    return (bool(value) and not PurePosixPath(value).is_absolute()
            and "\\" not in value and ":" not in value
            and not any(ord(c) < 32 for c in value)
            and all(part not in ("", ".", "..") for part in value.split("/")))


def validate(data, schema=None) -> list[str]:
    schema = read_json(SCHEMA) if schema is None else schema
    Draft202012Validator.check_schema(schema)
    errors = []
    for error in Draft202012Validator(schema).iter_errors(data):
        location = "/" + "/".join(str(p) for p in error.absolute_path)
        errors.append(f"{location}: {error.message}")
    if errors:
        return sorted(errors)  # Semantic checks require structurally valid data.

    def unique(items, key, label):
        found = {}
        for item in items:
            identity = item[key]
            if identity in found:
                errors.append(f"{label}: duplicate {key} {identity!r}")
            found[identity] = item
        return found

    videos = unique(data["videos"], "video_id", "videos")
    artifacts = unique(data.get("artifacts", []), "artifact_id", "artifacts")
    for identity, artifact in artifacts.items():
        if not safe_path(artifact["path"]):
            errors.append(f"artifact {identity}: path must be normalized repository-relative")
        for parent in artifact["inputs"]:
            if parent not in artifacts:
                errors.append(f"artifact {identity}: dangling input {parent!r}")

    # Topological check, including isolated or currently unbound artifacts.
    degrees = {key: 0 for key in artifacts}
    children = {key: [] for key in artifacts}
    for key, artifact in artifacts.items():
        for parent in artifact["inputs"]:
            if parent in artifacts:
                degrees[key] += 1
                children[parent].append(key)
    ready = deque(key for key, degree in degrees.items() if degree == 0)
    while ready:
        for child in children[ready.popleft()]:
            degrees[child] -= 1
            if degrees[child] == 0:
                ready.append(child)
    blocked = sorted(key for key, degree in degrees.items() if degree)
    if blocked:
        errors.append("artifact dependency cycle (or dependent on cycle): " + ", ".join(blocked))

    publication_ids = set()
    for video_id, video in videos.items():
        revisions = unique(video["revisions"], "revision_id", f"video {video_id}")
        for related in video["related_video_ids"]:
            if related not in videos:
                errors.append(f"video {video_id}: dangling related_video_id {related!r}")
        for revision_id, revision in revisions.items():
            label = f"cut {video_id}/{revision_id}"
            master = revision.get("master")
            if master:
                if not safe_path(master["path"]):
                    errors.append(f"{label}: master path must be normalized repository-relative")
                old_digest = master["digest"]
                if old_digest:
                    expected = {"md5": 32, "sha256": 64}[old_digest["algorithm"]]
                    if len(old_digest["value"]) != expected:
                        errors.append(f"{label}: {old_digest['algorithm']} digest requires {expected} hex digits")
            for role, refs in revision.get("artifact_refs", {}).items():
                for ref in refs:
                    artifact = artifacts.get(ref)
                    if artifact is None:
                        errors.append(f"{label}: dangling {role} artifact {ref!r}")
                    elif artifact["kind"] != role:
                        errors.append(f"{label}: {ref!r} has kind {artifact['kind']!r}, expected {role!r}")
                if role == "master" and master:
                    matches = [artifacts[r] for r in refs if r in artifacts
                               and artifacts[r]["path"] == master["path"]]
                    if len(matches) != 1:
                        errors.append(f"{label}: legacy master must match exactly one bound master path")
                    elif master["digest"] and master["digest"]["algorithm"] == "sha256":
                        if master["digest"]["value"] != matches[0]["sha256"]:
                            errors.append(f"{label}: legacy and bound master SHA256 disagree")
        for publication in video["publications"]:
            if publication["revision_id"] not in revisions:
                errors.append(f"video {video_id}: publication references missing revision {publication['revision_id']!r}")
            publication_id = (publication["venue"], publication["external_id"])
            if publication_id in publication_ids:
                errors.append(f"duplicate publication identity {publication_id!r}; retain observations in evidence_refs")
            publication_ids.add(publication_id)
    return sorted(errors)


def cut_lineage(data, video_id: str, revision_id: str) -> dict:
    """Requires validated data. Walk inputs, retaining their exact recorded data."""
    video = next(v for v in data["videos"] if v["video_id"] == video_id)
    revision = next(r for r in video["revisions"] if r["revision_id"] == revision_id)
    artifacts = {a["artifact_id"]: a for a in data.get("artifacts", [])}
    pending = [ref for refs in revision.get("artifact_refs", {}).values() for ref in refs]
    seen = set()
    while pending:
        ref = pending.pop()
        if ref not in seen:
            seen.add(ref)
            pending.extend(artifacts[ref]["inputs"])
    payload = {"video_id": video_id, "revision": revision,
               "artifacts": [artifacts[key] for key in sorted(seen)]}
    return {
        **payload,
        "manifest_sha256": digest(payload),
        "unrecorded_roles": [r for r in ROLES if r not in revision.get("artifact_refs", {})],
        "publications": [p for p in video["publications"] if p["revision_id"] == revision_id],
        "claim_scope": "Recorded lineage only; no fresh byte, listening, gate or publication qualification.",
    }


def board_rows(data, issues, blocked, children, root=ROOT):
    """Derive a read-only view; dated records remain observations, not live proof."""
    def reference(ref):
        if ref in by_id:
            return {"ref": ref, "state": "tracker_issue_exists"}
        if ref.startswith("mx-") and "/" not in ref:
            return {"ref": ref, "state": "tracker_issue_missing"}
        if ref.startswith(("https://", "http://")):
            return {"ref": ref, "state": "external_not_checked"}
        path, separator, _anchor = ref.partition("#")
        if not safe_path(path):
            return {"ref": ref, "state": "unresolved_reference"}
        state = "local_missing"
        if (root / path).exists():
            state = "local_exists_anchor_unchecked" if separator else "local_exists"
        return {"ref": ref, "state": state}

    by_id = {issue["id"]: issue for issue in issues}
    blocked_by = {issue["id"]: issue["blocked_by"] for issue in blocked}
    artifacts = {item["artifact_id"]: item for item in data.get("artifacts", [])}
    rows = []
    for video in data["videos"]:
        anchor = video.get("work_epic")
        seen, pending = set(), [anchor] if anchor else []
        while pending:
            issue_id = pending.pop()
            if issue_id in seen:
                continue
            seen.add(issue_id)
            pending.extend(children.get(issue_id, []))
        work = []
        for issue_id in sorted(seen):
            issue = by_id.get(issue_id)
            if issue is None:
                work.append({"id": issue_id, "status": "unknown", "title": "Tracker entry missing"})
            elif issue["status"] not in ("closed", "tombstone"):
                work.append({"id": issue_id, "title": issue["title"], "status": issue["status"],
                             "labels": issue.get("labels", []),
                             "blocked_by": blocked_by.get(issue_id, [])})
        revisions = []
        for cut in video["revisions"]:
            master = cut.get("master")
            refs = cut.get("artifact_refs", {})
            revisions.append({"revision_id": cut["revision_id"], "label": cut["label"],
                              "voice": cut["voice"], "generation_provenance": cut["generation_provenance"],
                              "master": master,
                              "master_path_exists": (root / master["path"]).is_file() if master else None,
                              "artifacts": [{**artifacts[ref], "path_exists": (root / artifacts[ref]["path"]).is_file(),
                                             "evidence_refs_health": [reference(link) for link in artifacts[ref]["evidence_refs"]]}
                                            for ids in refs.values() for ref in ids],
                              "unrecorded_artifact_roles": [role for role in ROLES if role not in refs],
                              "evidence_refs": [reference(ref) for ref in cut.get("evidence_refs", [])],
                              "milestones": [{**m, "evidence_refs_health": [reference(ref) for ref in m["evidence_refs"]]}
                                             for m in cut.get("milestones", [])],
                              "publications": [{**p, "evidence_refs_health": [reference(ref) for ref in p["evidence_refs"]]}
                                               for p in video["publications"] if p["revision_id"] == cut["revision_id"]]})
        decision = video["decisions"][-1] if video["decisions"] else None
        rows.append({"video_id": video["video_id"], "title": video["working_title"],
                     "approval": video["production_approval"], "editorial_decision": decision,
                     "source_refs": [reference(ref) for ref in video.get("source_refs", [])],
                     "recorded_public_revisions": [p["revision_id"] for p in video["publications"]
                                                   if p["visibility"] == "public"],
                     "work_epic": anchor, "tracker_anchor_state": "recorded" if anchor in by_id else "unknown",
                     "work": work, "revisions": revisions})
    return rows


def tracker_snapshot(data):
    """Read supported br envelopes, including the actual parent-child graph."""
    warnings = []
    def call(*args):
        result = subprocess.run(["br", *args, "--json", "--no-auto-import", "--no-auto-flush"], cwd=ROOT, text=True,
                                capture_output=True, check=False)
        if result.returncode:
            raise ValueError(f"br {' '.join(args)} failed ({result.returncode}): {result.stderr.strip()}")
        if result.stderr.strip():
            warnings.append(result.stderr.strip())
        return json.loads(result.stdout)
    warnings.append("Automatic tracker import and export are disabled; the board reads the local database as-is.")
    try:
        sync = call("sync", "--status")
        if sync.get("workspace_health") != "healthy" or sync.get("coverage_drift"):
            warnings.append("Tracker sync health: " + str(sync.get("workspace_health", "unknown")) +
                            "; DB and JSONL may differ (dirty count: " + str(sync.get("dirty_count", "unknown")) + ").")
    except (ValueError, json.JSONDecodeError) as exc:
        warnings.append("Tracker sync status unavailable: " + str(exc))
    issues = call("list", "--all", "--limit", "0")["issues"]
    blocked = call("blocked", "--limit", "0")["issues"]
    by_id = {issue["id"]: issue for issue in issues}
    children, seen = {}, set()
    pending = [v["work_epic"] for v in data["videos"] if v.get("work_epic")]
    while pending:
        issue_id = pending.pop()
        if issue_id in seen:
            continue
        seen.add(issue_id)
        issue = by_id.get(issue_id)
        if not issue or not issue.get("dependent_count"):
            continue
        details = call("show", issue_id)[0]
        child_ids = [d["id"] for d in details.get("dependents", []) if d["dependency_type"] == "parent-child"]
        children[issue_id] = child_ids
        pending.extend(child_ids)
    return issues, blocked, children, warnings


def print_board(result):
    """Render a compact human-readable view without converting observations into gates."""
    attention = {"local_missing", "tracker_issue_missing", "unresolved_reference",
                 "local_exists_anchor_unchecked", "external_not_checked"}
    print(f"Catalog updated {result['catalog_updated_on']}; {len(result['videos'])} video(s) in view")
    print(result["claim_scope"])
    if result["tracker_warning"]:
        print("Tracker warning: " + result["tracker_warning"])
    for video in result["videos"]:
        decision = video["editorial_decision"] or {}
        print(f"\n{video['video_id']} | {video['title']} | approval: {video['approval']} | "
              f"decision: {decision.get('decision', 'unknown')} ({decision.get('on', 'date unknown')})")
        for ref in video["source_refs"]:
            if ref["state"] in attention:
                print(f"  {ref['state']}: {ref['ref']}")
        if not video["revisions"]:
            print("  Cuts: none recorded")
        for cut in video["revisions"]:
            master = "no master recorded" if cut["master_path_exists"] is None else (
                "master path exists" if cut["master_path_exists"] else "master path missing")
            master_evidence = (cut["master"] or {}).get("evidence_state", "unknown")
            print(f"  {cut['revision_id']} | {cut['label']} | voice: {cut['voice']} | {master} | "
                  f"master evidence: {master_evidence} | "
                  f"unrecorded artifact roles: {', '.join(cut['unrecorded_artifact_roles']) or 'none'}")
            for artifact in cut["artifacts"]:
                existence = "exists" if artifact["path_exists"] else "missing"
                print(f"    recorded {artifact['kind']}: {artifact['evidence_state']} | "
                      f"path {existence}: {artifact['path']}")
                for ref in artifact["evidence_refs_health"]:
                    if ref["state"] in attention:
                        print(f"      {ref['state']}: {ref['ref']}")
            for milestone in cut["milestones"]:
                print(f"    recorded {milestone['stage']}: {milestone['evidence_state']} "
                      f"as of {milestone['recorded_as_of']}")
                for ref in milestone["evidence_refs_health"]:
                    if ref["state"] in attention:
                        print(f"      {ref['state']}: {ref['ref']}")
            for publication in cut["publications"]:
                print(f"    {publication['visibility']} as of {publication.get('recorded_as_of', 'unknown')}: "
                      f"{publication['url']} | evidence: {publication.get('evidence_state', 'unknown')}")
                for ref in publication["evidence_refs_health"]:
                    if ref["state"] in attention:
                        print(f"      {ref['state']}: {ref['ref']}")
            for ref in cut["evidence_refs"]:
                if ref["state"] in attention:
                    print(f"    {ref['state']}: {ref['ref']}")
        if video["tracker_anchor_state"] != "recorded":
            print("  Tracker anchor: unknown")
        for issue in video["work"]:
            blockers = ", ".join(issue.get("blocked_by", [])) or "none recorded"
            print(f"  {issue['id']} | {issue['status']} | {issue['title']} | blocked by: {blockers}")
    if result["unmapped_lane_work"]:
        print("\nMatching lane work outside mapped video epics:")
        for issue in result["unmapped_lane_work"]:
            print(f"  {issue['id']} | {issue['status']} | {issue['title']} | "
                  f"blocked by: {', '.join(issue['blocked_by']) or 'none recorded'}")
    print("\n" + result["lane_scope"])


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog", type=Path, default=CATALOG)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("validate")
    show = commands.add_parser("show")
    show.add_argument("video_id")
    show.add_argument("--revision")
    board = commands.add_parser("board", help="derive a read-only view from catalog and local Beads")
    board.add_argument("--lane", help="filter by a recorded Beads lane label, for example narration")
    board.add_argument("--decision", help="filter by the latest recorded editorial decision")
    board.add_argument("--blocker", help="filter by recorded blocking issue ID")
    board.add_argument("--format", choices=("json", "text"), default="json")
    args = parser.parse_args(argv)
    try:
        data = read_json(args.catalog)
        errors = validate(data)
        if errors:
            print(json.dumps({"valid": False, "errors": errors}, indent=2))
            return 1
        if args.command == "validate":
            result = {"valid": True, "catalog_sha256": digest(data),
                      "videos": len(data["videos"]),
                      "revisions": sum(len(v["revisions"]) for v in data["videos"]),
                      "artifacts": len(data.get("artifacts", [])),
                      "claim_scope": "Structure and references only; recorded evidence is not requalified."}
        elif args.command == "board":
            issues, blocked, children, warnings = tracker_snapshot(data)
            rows = board_rows(data, issues, blocked, children)
            unmapped_lane_work = []
            if args.lane:
                lane = args.lane if args.lane.startswith("lane:") else "lane:" + args.lane
                mapped = {work["id"] for row in rows for work in row["work"]}
                blocked_by = {item["id"]: item["blocked_by"] for item in blocked}
                unmapped_lane_work = [{"id": item["id"], "title": item["title"], "status": item["status"],
                                       "blocked_by": blocked_by.get(item["id"], [])}
                                      for item in issues if item["id"] not in mapped
                                      and item["status"] not in ("closed", "tombstone")
                                      and lane in item.get("labels", [])]
                rows = [row for row in rows if any(lane in work.get("labels", []) for work in row["work"])]
            if args.decision:
                rows = [row for row in rows if (row["editorial_decision"] or {}).get("decision") == args.decision]
            if args.blocker:
                rows = [row for row in rows if any(args.blocker in work.get("blocked_by", []) for work in row["work"])]
                unmapped_lane_work = [work for work in unmapped_lane_work if args.blocker in work["blocked_by"]]
            result = {"catalog_updated_on": data["updated_on"], "videos": rows,
                      "tracker_warning": " | ".join(dict.fromkeys(warnings)) if warnings else None,
                      "unmapped_lane_work": unmapped_lane_work,
                      "lane_scope": "Video rows use explicitly labelled unfinished work under that video epic; other matching work is listed separately, without an editorial decision, and unlabelled work is not guessed.",
                      "claim_scope": "Recorded dates and tracker state are observations; no live publication, byte, listening or release qualification. Tracker reads are not atomic."}
            if args.format == "text":
                print_board(result)
                return 0
        else:
            videos = [v for v in data["videos"] if v["video_id"] == args.video_id]
            if not videos:
                raise ValueError(f"unknown video_id {args.video_id!r}")
            revisions = videos[0]["revisions"]
            if args.revision:
                revisions = [r for r in revisions if r["revision_id"] == args.revision]
                if not revisions:
                    raise ValueError(f"unknown revision_id {args.revision!r} in {args.video_id!r}")
            result = {"video_id": args.video_id, "working_title": videos[0]["working_title"],
                      "cuts": [cut_lineage(data, args.video_id, r["revision_id"]) for r in revisions]}
        print(json.dumps(result, indent=2, ensure_ascii=False))
        return 0
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(json.dumps({"valid": False, "errors": [str(error)]}, indent=2))
        return 1


if __name__ == "__main__":
    sys.exit(main())
