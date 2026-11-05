"""Read-only analysis of attempt 03: added tests by name, default pixels, inputs and stage sources.

    python3 -I -B attempt03.py RUNNER ROOT_STAGE STAGE TEST_LOG

Uses the runner's own functions (tests_added, scan, pixel_faults, hash_faults, changed_inputs) on
files attempt 03 left behind. Writes nothing, compiles nothing, queues nothing.
"""
import collections
import hashlib
import importlib.util
import io
import json
import sys
import time
from pathlib import Path

runner_path, source, stage, log = map(Path, sys.argv[1:5])
spec = importlib.util.spec_from_file_location("runner", runner_path)
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)
runner.SOURCE = source.resolve()

# 1. Tests the stack adds, against the `... ok` lines of engine-test03 (the runner's own scan).
seen = runner.scan(io.StringIO(log.read_text()), io.StringIO())
added = runner.tests_added()
print(f"engine-test03 log: {seen['passed']} passed in all")
for file, name, ignored in added:
    state = "ignored, runs in engine-ignored" if ignored else ("ok" if name in seen["ok"] else "NOT ok")
    print(f"  {state:32} {Path(file).name}: {name}")
ran = [n for _, n, skip in added if not skip]
print(f"added, not ignored: {sum(n in seen['ok'] for n in ran)} of {len(ran)} ok")

# 2. Default frames: pixels/base from base-run01 (unpatched), pixels/combined from engine-test03.
base, combined = stage / "pixels/base", stage / "pixels/combined"
count = lambda root: collections.Counter(p.relative_to(root).parts[0] for p in root.rglob("*.png"))
print(f"frames per route, base {dict(sorted(count(base).items()))}")
print(f"frames per route, combined {dict(sorted(count(combined).items()))}")
faults = runner.pixel_faults(base, combined)
print(f"default pixels (runner.pixel_faults): {'identical' if not faults else faults}")
digest = hashlib.sha256()
for path in sorted(base.rglob("*.png")):
    digest.update(f"{path.relative_to(base)}\0{runner.sha(path)}\n".encode())
print(f"digest of the {sum(count(base).values())} base frames: {digest.hexdigest()[:16]}")
distinct = {runner.sha(p) for p in base.rglob("*.png")}
print(f"distinct frame contents among them: {len(distinct)}")

# 3. The stage's patched sources after the run, and the frozen dependencies.
print(f"stage against the extended final hashes: {runner.hash_faults(stage / 'engine', runner.STACK[-1][1]) or 'all match'}")
manifest = json.loads((stage / "manifests/engine-test03.json").read_text())["files"]
moved = [row["path"] for row in manifest if Path(row["path"]).is_file() and runner.sha(row["path"]) != row["sha256"]]
print(f"engine-test03 input manifest, {len(manifest)} files re-hashed now: {moved or 'all unchanged'}")
started = time.monotonic()
faults = runner.changed_inputs(runner_path.parent / "inputs04.tsv", runner.not_source)
print(f"non-source inputs against inputs04 after attempt 03: {faults or 'all match'} ({time.monotonic() - started:.0f} s)")
