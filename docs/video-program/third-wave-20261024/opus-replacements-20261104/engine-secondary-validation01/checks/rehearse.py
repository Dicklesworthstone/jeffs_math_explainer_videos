#!/usr/bin/env python3
"""Rehearse every part of the runner that needs no Cargo, and write the other files in checks/.

    python3 -I -B checks/rehearse.py ROOT_STAGE NEW_STAGE

Copies ROOT_STAGE into NEW_STAGE (outside any git work tree), applies both patches there, and builds
the manifests root's guard would hash. Launches no Cargo, no guard job and no lane lock.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import time

CHECKS = Path(__file__).resolve().parent
OUT = CHECKS.parent
REPO = OUT.parents[4]
SOURCE, STAGE = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).absolute()
PY = ["python3", "-I", "-B"]
RUNNER = OUT / "run_validation.py"


def scrub(text):
    for path, word in ((STAGE, "STAGE"), (SOURCE, "ROOT_STAGE"), (REPO, "REPO"), (Path.home(), "HOME")):
        text = text.replace(str(path), word)
    return text


def run(*argv, **kw):
    done = subprocess.run([str(a) for a in argv], capture_output=True, text=True, **kw)
    return done.returncode, scrub(done.stdout + done.stderr)


report = []


def say(line):
    report.append(line + "\n")
    print(line)


code, text = run(*PY, RUNNER, "--self-test")
assert code == 0, text
(CHECKS / "self-test.out").write_text(text)

rows = (OUT / "inputs.tsv").read_text().splitlines()[1:]
kinds = {}
for row in rows:
    name, files, size, _ = row.split("\t")
    entry = kinds.setdefault(name.split("/")[0], [0, 0, 0])
    entry[0] += 1
    entry[1] += int(files)
    entry[2] += int(size)
for kind, (count, files, size) in sorted(kinds.items()):
    say(f"inputs.tsv {kind}: {count} rows, {files} files, {size >> 20} MiB")

# What root's last real job hashed for the same trees must be what is bound here.
routes = json.loads((SOURCE / "film-cells4-routes01-inputs.json").read_text())["files"]
for tree in ("engine", "repo-view", "crate"):
    prefix = f"{SOURCE}/{tree}/"
    theirs = {r["path"][len(prefix):]: r["sha256"] for r in routes if r["path"].startswith(prefix) and "/.ee/" not in r["path"]}
    ours = {}
    for path in (SOURCE / tree).rglob("*"):
        if path.is_file() and ".ee" not in path.parts:
            ours[str(path.relative_to(SOURCE / tree))] = hashlib.sha256(path.read_bytes()).hexdigest()
    differ = sorted(k for k in set(theirs) | set(ours) if theirs.get(k) != ours.get(k))
    say(f"{tree}: {len(ours)} files; differ from root's film-cells4-routes01 manifest: {differ}")

assert not STAGE.exists(), "the new stage already exists: name another"
inside = OUT / "no-such-stage"
code, text = run(*PY, RUNNER, "stage", "--source", SOURCE, "--stage", inside)
assert code != 0 and "outside any git work tree" in text and not inside.exists(), text
say("a stage inside the repository: refused before any copy")
started = time.monotonic()
code, text = run(*PY, RUNNER, "stage", "--source", SOURCE, "--stage", STAGE)
assert code == 0, text
say(text.strip())
say(f"stage took {time.monotonic() - started:.0f} s, most of it hashing the frozen inputs")
code, text = run("grep", "-rl", str(SOURCE), STAGE / "engine", STAGE / "repo-view", STAGE / "crate", STAGE / "budget.toml")
assert code == 1, text
say("no file in the stage names root's staged folder")
assert f"{STAGE}/repo-view/crates/explainer_kit" in (STAGE / "crate/Cargo.toml").read_text()
assert (STAGE / "budget.toml").read_text().count(f"{STAGE}/engine/crates/") == 19
say("crate/Cargo.toml and budget.toml name the stage copies: 19 engine packages")

spec = importlib.util.spec_from_file_location("runner", RUNNER)
runner = importlib.util.module_from_spec(spec)
sys.modules["runner"] = runner
spec.loader.exec_module(runner)
runner.SOURCE = SOURCE
started = time.monotonic()
assert not runner.changed_inputs(OUT / "inputs.tsv", runner.not_source)
say(f"every frozen dependency still matches inputs.tsv; `run` makes this check before and after, {time.monotonic() - started:.0f} s here")

guard = runner.spec_load(runner.GUARD, "guard").load()
before = runner.manifest(STAGE, "rehearse-base", "01")
assert guard.check_inputs(before)
say(f"manifest before the patches: {len(json.loads(before.read_text())['files'])} rows; root's own check_inputs accepts it")

assert not runner.unpatched_faults(STAGE)
faults = runner.hash_faults(STAGE / "engine", runner.STACK[-1][1])
say(f"before the patches, files away from the combined hashes: {len(faults)} of 8")
runner.apply_stack(STAGE)
assert not runner.hash_faults(STAGE / "engine", runner.STACK[-1][1])
patched = runner.unpatched_faults(STAGE)
assert len(patched) == len(runner.patched_files()) == 12
say("after the patches, all 12 touched files differ from the unpatched record, so the base jobs refuse this stage")
hashes = {rel: runner.sha(STAGE / "engine" / rel) for rel in runner.patched_files()}
runner.apply_stack(STAGE)
assert hashes == {rel: runner.sha(STAGE / "engine" / rel) for rel in runner.patched_files()}
say("apply_stack: both patches applied, 8 + 8 sealed hashes checked, a second call changes nothing")
after = runner.manifest(STAGE, "rehearse-combined", "01")
assert guard.check_inputs(after)
say(f"manifest after the patches: {len(json.loads(after.read_text())['files'])} rows; accepted")
probe = STAGE / "engine/crates/fmn/tests" / runner.PROBE.name
assert probe.read_bytes() == runner.PROBE.read_bytes()

# The real format steps of engine-lint and kit-build. They need no Cargo.
table = runner.jobs(STAGE)
fmt = [step for step in table["engine-lint"].steps + table["kit-build"].steps if step["argv"][0] == "rustfmt"]
assert len(fmt) == 2
lines = []
for step in fmt:
    code, text = run(*step["argv"], cwd=step["cwd"])
    assert code == 0, text
    lines.append(f"exit {code} in {scrub(str(step['cwd']))}: rustfmt on {len(step['argv']) - 6 if step is fmt[0] else 1} files\n")
(CHECKS / "rustfmt.out").write_text("".join(lines))
say("rustfmt: the engine-lint and kit-build format steps exit 0 on the stage")

steps = sum(len(job.steps) for job in table.values())
bounds = [job.bound_gib for job in table.values()]
say(f"{len(table)} jobs, {steps} steps; upper bounds {sum(bounds):.2f} GiB in all, largest {max(bounds)} GiB")
assert max(bounds) < 4
must = {name for job in table.values() for step in job.steps for name in step.get("must", ())}
added = {name for _, name, _ in runner.tests_added()}
assert added <= must, added - must
say(f"all {len(added)} tests the patches add are required by name, the 2 ignored ones included")

code, text = run(*PY, RUNNER, "plan", "--stage", STAGE)
assert code == 0, text
(CHECKS / "plan.out").write_text(text)
(CHECKS / "rehearsal.out").write_text(scrub("".join(report)))
print("wrote checks; compiled nothing; queued nothing")
