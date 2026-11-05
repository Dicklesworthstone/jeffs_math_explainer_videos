"""Read-only analysis of attempt 04: added tests by name, the failure's own words, default pixels,
inputs and stage sources.

    python3 -I -B attempt04-analysis.py RUNNER ROOT_STAGE STAGE TEST_LOG

Uses the runner's own functions (tests_added, scan, pixel_faults, hash_faults, changed_inputs) on
files attempt 04 left behind. Writes nothing, compiles nothing, queues nothing.
"""
import collections
import hashlib
import io
import importlib.util
import json
import re
import sys
import time
from pathlib import Path

runner_path, source, stage, log = map(Path, sys.argv[1:5])
spec = importlib.util.spec_from_file_location("runner", runner_path)
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)
runner.SOURCE = source.resolve()
text = log.read_text()

# 1. Tests the stack adds, against the `... ok` lines of engine-test04 (the runner's own scan).
seen = runner.scan(io.StringIO(text), io.StringIO())
added = runner.tests_added()
failed = set(re.findall(r"^test (\S+) \.\.\. FAILED$", text, re.M))
print(f"engine-test04 log: {seen['passed']} passed in all; failed: {sorted(failed) or 'none'}")
for file, name, ignored in added:
    if ignored:
        state = "ignored, runs in engine-ignored"
    elif name in seen["ok"]:
        state = "ok"
    elif name in failed:
        state = "FAILED"
    else:
        state = "not run"
    print(f"  {state:32} {Path(file).name}: {name}")
ran = [n for _, n, skip in added if not skip]
print(f"added, not ignored: {sum(n in seen['ok'] for n in ran)} of {len(ran)} ok, "
      f"{sum(n in failed for n in ran)} failed, {sum(n not in seen['ok'] and n not in failed for n in ran)} not run")

# 2. The failure in its own words: which frame's emit failed, whose slot was abandoned, how far it got.
for line in text.splitlines():
    if line.startswith("round ") and "Pipeline(" in line:
        head = line.split(": ", 2)
        stage_error = re.search(r"error: (Stage \{ sequence: Some\((\d+)\), stage: (\w+), source: (.*?) \}), stats", line)
        stats = {key: int(value) for key, value in re.findall(r"\b(submitted|prepared|rasterized|converted|emitted|max_in_flight|outstanding_slots|backpressure_waits): (\d+)", line)}
        abandoned = re.search(r"ReservationAbandoned \{ sequence: (\d+) \}", line)
        print(f"failure: {head[0]}, {head[1]}")
        print(f"  recorded first: stage {stage_error[3]} of frame {stage_error[2]}, source {stage_error[4]} }}")
        print(f"  abandoned slot: frame {abandoned[1]}; named refusal in the words: {'macrotile references' in line}")
        print(f"  stats: {stats}")

# 3. Default frames: pixels/base from base-run01 (unpatched), pixels/combined from engine-test04.
base, combined = stage / "pixels/base", stage / "pixels/combined"
count = lambda root: collections.Counter(p.relative_to(root).parts[0] for p in root.rglob("*.png"))
print(f"frames per route, base {dict(sorted(count(base).items()))}")
print(f"frames per route, combined (attempt 04) {dict(sorted(count(combined).items()))}")
faults = runner.pixel_faults(base, combined)
print(f"default pixels, base vs attempt 04 (runner.pixel_faults): {'identical' if not faults else faults}")
for other in ("combined-attempt03-written", "combined-attempt03", "combined-attempt04"):
    faults = runner.pixel_faults(combined, stage / "pixels" / other)
    print(f"  attempt 04 vs pixels/{other}: {'identical' if not faults else faults}")
digest = hashlib.sha256()
for path in sorted(combined.rglob("*.png")):
    digest.update(f"{path.relative_to(combined)}\0{runner.sha(path)}\n".encode())
print(f"digest of the {sum(count(combined).values())} attempt 04 frames: {digest.hexdigest()[:16]}")
print(f"distinct frame contents among them: {len({runner.sha(p) for p in combined.rglob('*.png')})}")

# 4. The stage's patched sources after the run, and the frozen dependencies.
print(f"stage against the F1 entry's sealed hashes: {runner.hash_faults(stage / 'engine', runner.STACK[-1][1]) or 'all match'}")
manifest = json.loads((stage / "manifests/engine-test04.json").read_text())["files"]
moved = [row["path"] for row in manifest if Path(row["path"]).is_file() and runner.sha(row["path"]) != row["sha256"]]
print(f"engine-test04 input manifest, {len(manifest)} files re-hashed now: {moved or 'all unchanged'}")
started = time.monotonic()
faults = runner.changed_inputs(runner_path.parent / "inputs05.tsv", runner.not_source)
print(f"non-source inputs against inputs05 after attempt 04: {faults or 'all match'} ({time.monotonic() - started:.0f} s)")
