#!/usr/bin/env python3
"""Read-only catalog validation and exact-cut lineage inspection.

uv run --no-project --with jsonschema python3 -I tools/video_catalog.py validate
uv run --no-project --with jsonschema python3 -I tools/video_catalog.py show hoeffding_d

Structural validity and recorded hashes are not fresh file verification or
publication authority. This tool never writes the catalog, Beads or gate state.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from collections import deque
from pathlib import Path, PurePosixPath
import sys

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


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog", type=Path, default=CATALOG)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("validate")
    show = commands.add_parser("show")
    show.add_argument("video_id")
    show.add_argument("--revision")
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
    except (OSError, ValueError) as error:
        print(json.dumps({"valid": False, "errors": [str(error)]}, indent=2))
        return 1


if __name__ == "__main__":
    sys.exit(main())
