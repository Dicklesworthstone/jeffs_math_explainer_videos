#!/usr/bin/env python3
"""Install exactly stage06's seven files into the canonical logarithm crate. Root runs this.

    python3 -I -B integrate_stage06.py --check                  read only: would it install cleanly?
    python3 -I -B integrate_stage06.py --apply --out RECORD_DIR  install, keeping what it replaces

Run from the repo root. RECORD_DIR must not exist yet. Follows root's combined-integration01
pattern: explicit paths, the old bytes kept under RECORD_DIR/before, a manifest written before the
first copy, every hash asserted before and after, refusal to run twice. It deletes nothing.
Paths in its records are repo-relative. It has only ever been run on a temp copy of the repo
(rehearse_integration.py), not on the real tree.

Source of each file, on purpose:
    the five chapters       stage06's crate
    narration_units.json    bound-plan. stage06's crate holds a stale 14-unit file.
    map oliver13            bound-plan. stage06's crate has no such file.
Not installed: doing_nothing.rs.orig, stage06's stale units, anything else.
oliver12 stays in docs/, unbound, as every older map has.
"""
import argparse
import datetime
import hashlib
import json
import os
import shutil
import sys
from pathlib import Path

sys.dont_write_bytecode = True
REL = Path("explainer_corpus/logarithm_mysteries")
LANE = Path("narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104")
STAGE = LANE / "combined-repair-stage06"
CRATE = STAGE / REL
PLAN = STAGE / "bound-plan" / REL
RECEIPTS = LANE.parent / "five-film-release-20261027"
MAP = "docs/narration-direction-map-oliver13.json"

# (path in the crate, where its new bytes live, sha256 now in canonical, sha256 after)
FILES = [
    ("src/chapters/undoing_growth.rs", CRATE, "8e26f3e4ae1e12b65bce5441e260f7beaf5d6fc0ab45b847f7a994f660f5ed36", "754ea3125c2f7e89440807772713fe57584c96937be3ec0ac8c52d86645cf323"),
    ("src/chapters/complex_turns.rs", CRATE, "d5f202afb97debcef1a3f7abd872634b7cc7093108e87975167b04b065e212d5", "2cc2ba7723e2733ef06c633650b135507d5d1726161fa2a277b90e00277b5ead"),
    ("src/chapters/lifting_loops.rs", CRATE, "78af9acdd126c12035ec865280b91b303921c228cab7f79023987fb90ec228c6", "ca34aef0523ceadce22edccd7d384ac29c514653207fdf052fd5cb31cd3fa01e"),
    ("src/chapters/doing_nothing.rs", CRATE, "535e985522f976fe83ed17a36f5ddd9a2fb29467225162aeb9f119ef13398a43", "eeee764f0fe5fd3cd45263ada9a87f22b3e05a48b120ca2ba4f862cdf7a27143"),
    ("src/chapters/unreachable.rs", CRATE, "43c8dac362df7fac8d6c974b3c108a25dee677faac85d093bfdafc1564db6527", "a676dac78580ef73f0e658a21c0395cfe0505fe631bd6aa0d71b1f3ec05a86ee"),
    ("narration_units.json", PLAN, "34747511327c7c8bfee51c7a04c1fc721199d11117cb840c3df1e3205864dc7d", "d594089297ed09534bf4a291f1ddb33d10402da3d60a2aeee9f7175a4381aee2"),
    (MAP, PLAN, None, "56f6ad64344514005a5ac385eb7826fa276346fb6c449acf54bf3de191da7e27"),
]
# Everything else in the crate must already match stage06, except these known differences.
ONLY_STAGE = {"src/chapters/doing_nothing.rs.orig"}
ONLY_CANONICAL = {"docs/narration-direction-map-oliver12.json"}


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def live_jobs(repo):
    """Guarded jobs with no finish time whose pid still answers. A recycled pid can look live."""
    found = []
    for receipt in (repo / RECEIPTS).glob("*/resource-receipt.json"):
        try:
            row = json.loads(receipt.read_bytes())
        except (OSError, ValueError):
            continue
        pid = row.get("pid")
        if row.get("finished") is None and isinstance(pid, int):
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                continue
            except PermissionError:
                pass
            found.append(f"{receipt.parent.name} pid {pid}")
    return sorted(found)


def preconditions(repo):
    problems = []
    for rel, origin, before, after in FILES:
        source, dest = repo / origin / rel, repo / REL / rel
        if not source.is_file():
            problems.append(f"{rel}: new bytes missing at {origin}")
        elif sha(source) != after:
            problems.append(f"{rel}: new bytes are not the frozen candidate")
        have = sha(dest) if dest.exists() else None
        if have != before:
            state = "already the candidate; was this run before?" if have == after else f"canonical is {str(have)[:16]}, expected {str(before)[:16]}"
            problems.append(f"{rel}: {state}")
    installed = {rel for rel, *_ in FILES}
    stage = {p.relative_to(repo / CRATE).as_posix() for p in (repo / CRATE).rglob("*") if p.is_file()}
    canon = {p.relative_to(repo / REL).as_posix() for p in (repo / REL).rglob("*") if p.is_file() and "target" not in p.relative_to(repo / REL).parts}
    if stage - canon - installed != ONLY_STAGE:
        problems.append(f"only in stage06: {sorted(stage - canon - installed)}")
    if canon - stage - installed != ONLY_CANONICAL:
        problems.append(f"only in canonical: {sorted(canon - stage - installed)}")
    differ = sorted(k for k in (stage & canon) - installed if sha(repo / CRATE / k) != sha(repo / REL / k))
    if differ:
        problems.append(f"canonical moved under stage06, not in the seven files: {differ}")
    return problems


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", default=".")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--apply", action="store_true")
    parser.add_argument("--out", help="with --apply: a new folder for the backups and records")
    parser.add_argument("--open-jobs-checked", action="store_true", help="root has looked at the live jobs listed and they bind none of these files")
    args = parser.parse_args()
    repo = Path(args.repo).resolve()
    problems = preconditions(repo)
    jobs = live_jobs(repo)
    if jobs and not args.open_jobs_checked:
        problems.append(f"guarded jobs look live, and any that binds these paths would trip its input guard: {jobs}")
    for problem in problems:
        print(f"REFUSED {problem}")
    if problems:
        return 1
    if args.check:
        print(f"OK: all {len(FILES)} files can install; nothing written. Live guarded jobs: {jobs or 'none'}")
        return 0
    if not args.out:
        print("REFUSED --apply needs --out")
        return 1
    out = (repo / args.out).resolve()
    if out.exists():
        print(f"REFUSED {args.out} exists; a record is never overwritten")
        return 1
    manifest = []
    for rel, origin, before, after in FILES:
        dest = repo / REL / rel
        if dest.exists():
            backup = out / "before" / rel
            backup.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(dest, backup)
            assert sha(backup) == before, rel
        manifest.append({"path": (REL / rel).as_posix(), "source": (origin / rel).as_posix(), "before": before, "after": after})
    out.mkdir(parents=True, exist_ok=True)
    with (out / "before-manifest.json").open("x") as handle:
        json.dump(manifest, handle, indent=2)
    for row in manifest:
        dest, source = repo / row["path"], repo / row["source"]
        assert (sha(dest) if dest.exists() else None) == row["before"], row["path"]
        assert sha(source) == row["after"], row["source"]
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, dest)
        assert sha(dest) == row["after"], row["path"]
    with (out / "integration.json").open("x") as handle:
        json.dump({"utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "files": manifest, "live_jobs_seen": jobs,
                   "claim": "stage06 source and bound plan installed; every prior file kept under before/. Native build, tests, glyphs, new pictures, Oliver takes, listening and final AV all pending."}, handle, indent=2)
    print(f"Installed {len(manifest)} files; records in {args.out}. Next: build_stage06.sh, which runs preflight --mode canonical first.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
