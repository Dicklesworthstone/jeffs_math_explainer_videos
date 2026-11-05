#!/usr/bin/env python3
"""Guarded native validation of the frame-table and camera budget patches on root's stack.

    python3 -I -B run_validation.py --self-test                     Python only: no Cargo, lane or stage
    python3 -I -B run_validation.py plan --stage NEW                live headroom, every command; runs nothing
    python3 -I -B run_validation.py bind --source ROOT_STAGE --out FILE      hash the inputs again
    python3 -I -B run_validation.py stage --source ROOT_STAGE --stage NEW    new verified copy; compiles nothing
    python3 -I -B run_validation.py run --source ROOT_STAGE --stage NEW [--from JOB] [--attempt 02]

ROOT_STAGE holds root's engine, repo-view and crate as bound in inputs.tsv. `stage` refuses any other
bytes, so a different film needs its own reviewed bind and its own job.

`run` queues each job on root's exclusive lane through guard_owner56g_target20g.py and stops at the
first failure or resource stop. It never retries: after a stop, --from with a new --attempt is root's
decision. Copies only; root's staged tree and the sealed bundles are read, never written. The camera
route, the frame tables and the two ignored controls are unmeasured until this runs.
"""
import argparse
from contextlib import contextmanager
from dataclasses import dataclass, field
import glob
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
import tomllib

HERE = Path(__file__).resolve().parent.parent / "engine-secondary-validation01"
O = HERE.parent
REPO = HERE.parents[4]
TARGET = REPO / "target"
SOURCE = None  # Root's staged folder, from --source. Machine paths stay out of this public file.
GUARD = HERE / "guard_owner56g_target20g.py"
PROBE = HERE / "secondary_default_pixels.rs"
INPUTS = HERE / "inputs.tsv"
T, S, V = O / "engine-secondary-budget01", O / "engine-secondary-camera-budget01", O / "engine-geometry-validation02"
STACK = (
    (T / "engine-frame-table-limits.patch", T / "expected-engine.sha256"),
    (S / "engine-camera-limits.patch", S / "expected-engine.sha256"),
)
TOOLS = ("verify_patched_lock.py", "instantiate_config.py", "repoint_lab.py", "engine-budget.config.toml.in")
TREES = ("engine", "repo-view", "crate")
PINNED_LOCKS = ("Cargo.lock.pinned", "Cargo.lock.view-pinned")
ROUTES = {"affine-serial", "affine-worker", "camera-serial", "camera-worker", "camera-rig"}
TOOLCHAIN = Path.home() / ".rustup/toolchains/nightly-2026-08-31-aarch64-apple-darwin"
REGISTRY = Path.home() / ".cargo/registry/src/index.crates.io-1949cf8c6b5b557f"
CHECKOUTS = Path.home() / ".cargo/git/checkouts"
FILM_BINARIES = (
    TARGET / "geometry-kit01/debug/complex_analysis_visual",
    TARGET / "geometry-kit01/debug/examples/plate_lab",
)
# Refuse to start below this: 8 GiB stop + the 7.43 GiB of upper bounds below, rounded up.
RUN_FLOOR_GIB = 16
HOUR = 3600


def sha(path):
    with open(path, "rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def files_under(root):
    """Every file and symlink below root, except session state and git metadata."""
    for parent, dirs, names in os.walk(root, followlinks=False):
        dirs.sort()
        links = [d for d in dirs if (Path(parent) / d).is_symlink()]
        dirs[:] = [d for d in dirs if d not in (".ee", ".git", "__pycache__") and d not in links]
        yield from (Path(parent) / name for name in sorted(names + links) if name != ".DS_Store")


def tree_row(root):
    digest, count, size = hashlib.sha256(), 0, 0
    for path in files_under(root):
        if path.is_symlink():
            content = "link:" + os.readlink(path)
        else:
            content, size = sha(path), size + path.stat().st_size
        digest.update(f"{path.relative_to(root)}\0{content}\n".encode())
        count += 1
    return count, size, digest.hexdigest()


def locked_sources():
    """Registry and git sources of every package in root's five locks. The checkouts are what compile."""
    found = {}
    for name in ("engine/Cargo.lock", "repo-view/Cargo.lock", "crate/Cargo.lock", *PINNED_LOCKS):
        for package in tomllib.loads((SOURCE / name).read_text())["package"]:
            source = package.get("source", "")
            if source.startswith("registry+"):
                label = f"{package['name']}-{package['version']}"
                found[f"registry/{label}"] = REGISTRY / label
            elif source.startswith("git+"):
                repo, full = re.fullmatch(r"git\+https://[^/]+/[^/]+/([^?#]+)(?:\?[^#]*)?#([0-9a-f]+)", source).groups()
                hits = glob.glob(f"{CHECKOUTS}/{repo}-*/{full[:7]}")
                if len(hits) != 1:
                    raise SystemExit(f"expected one checkout of {repo} {full[:7]}, found {len(hits)}")
                found[f"git/{repo}/{full[:7]}"] = Path(hits[0])
    return found


def frozen_paths():
    """Name to path for everything the run depends on. Names hold no machine-specific path."""
    paths = {f"source/{name}": SOURCE / name for name in (*TREES, *PINNED_LOCKS)}
    paths |= locked_sources()
    paths |= {"toolchain/bin": TOOLCHAIN / "bin", "toolchain/lib": TOOLCHAIN / "lib"}
    paths["tool/ffmpeg"] = Path(os.path.realpath(shutil.which("ffmpeg")))
    guard = spec_load(GUARD, "guard_wrapper")
    repo_files = [guard.SOURCE, GUARD, PROBE, *(p for pair in STACK for p in pair), *(V / tool for tool in TOOLS)]
    paths |= {f"repo/{p.relative_to(REPO)}": p for p in repo_files}
    return paths


def spec_load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def row_for(name, path):
    if path.is_dir():
        return (name, *tree_row(path))
    return (name, 1, path.stat().st_size, sha(path))


def bind(out):
    rows = [row_for(name, path) for name, path in sorted(frozen_paths().items())]
    with open(out, "x") as file:
        file.write("name\tfiles\tbytes\tsha256\n")
        file.writelines("\t".join(map(str, row)) + "\n" for row in rows)
    print(f"{out}: {len(rows)} rows, {sum(r[1] for r in rows)} files, {sum(r[2] for r in rows) >> 20} MiB")


def changed_inputs(inputs, keep=lambda name: True):
    """The frozen rows that no longer match. Root's staged tree can move on once the stage is copied."""
    lines = Path(inputs).read_text().splitlines()[1:]
    frozen = {line.split("\t")[0]: line.split("\t")[3] for line in lines}
    paths = frozen_paths()
    faults = [f"not frozen: {name}" for name in paths if name not in frozen]
    faults += [f"gone: {name}" for name in frozen if name not in paths]
    for name, digest in frozen.items():
        if name in paths and keep(name) and row_for(name, paths[name])[3] != digest:
            faults.append(f"changed: {name}")
    return faults


def not_source(name):
    return not name.startswith("source/")


def make_stage(stage, inputs):
    probe = stage.parent
    while not probe.exists():
        probe = probe.parent
    inside = subprocess.run(["git", "-C", str(probe), "rev-parse", "--is-inside-work-tree"], capture_output=True, text=True)
    if inside.returncode == 0:
        raise SystemExit("the stage must sit outside any git work tree: git apply there resolves paths from its top")
    faults = changed_inputs(inputs)
    if faults:
        raise SystemExit("inputs changed since binding; review, then bind again:\n  " + "\n  ".join(faults))
    stage.mkdir(parents=True)  # Refuses an existing stage.
    for name in TREES:
        shutil.copytree(SOURCE / name, stage / name, symlinks=True, ignore=shutil.ignore_patterns(".ee"))
        if tree_row(stage / name) != tree_row(SOURCE / name):
            raise SystemExit(f"the copy of {name} differs from its source")
    for name in PINNED_LOCKS:
        shutil.copy2(SOURCE / name, stage / name)
    shutil.copy2(PROBE, stage / "engine/crates/fmn/tests" / PROBE.name)
    py = ["python3", "-I", "-B"]
    subprocess.run([*py, V / "instantiate_config.py", stage / "engine", stage / "budget.toml"], check=True)
    subprocess.run([*py, V / "repoint_lab.py", stage / "crate", stage / "repo-view"], check=True)
    for name in ("manifests", "results", "pixels"):
        (stage / name).mkdir()
    engine = stage / "engine"
    record = {rel: sha(engine / rel) if (engine / rel).is_file() else None for rel in patched_files()}
    (stage / "manifests/unpatched.json").write_text(json.dumps(record, indent=2) + "\n")
    print(f"stage ready; compiled nothing: {stage}")


def expected(file):
    return dict(reversed(line.split(maxsplit=1)) for line in Path(file).read_text().splitlines())


def hash_faults(engine, expected_file):
    """Files away from a patch's sealed result. A file the patch adds is a fault until it exists."""
    return [rel for rel, digest in expected(expected_file).items()
            if not (engine / rel).is_file() or sha(engine / rel) != digest]


def patched_files():
    return sorted({rel for _, hashes in STACK for rel in expected(hashes)})


def unpatched_faults(stage):
    """The base jobs must see root's bytes, not a stage a later attempt already patched."""
    record = json.loads((stage / "manifests/unpatched.json").read_text())
    engine = stage / "engine"
    now = {rel: sha(engine / rel) if (engine / rel).is_file() else None for rel in record}
    return [rel for rel in record if now[rel] != record[rel]]


def apply_stack(stage):
    """Apply both patches once, in order. Each one's result must match its sealed hashes."""
    engine = stage / "engine"
    if not hash_faults(engine, STACK[-1][1]):
        return
    for patch, hashes in STACK:
        if hash_faults(engine, hashes):
            subprocess.run(["git", "apply", str(patch)], cwd=engine, check=True)
            faults = hash_faults(engine, hashes)
            if faults:
                raise SystemExit(f"{patch.name} left other hashes: {faults}")


def tests_added():
    """Every test the two patches add: (file, name, ignored), read from the patches themselves."""
    found = []
    for patch, _ in STACK:
        current, pending, ignored = None, False, False
        for line in patch.read_text().splitlines():
            if line.startswith("+++ b/"):
                current = line[6:]
            elif line.startswith("+"):
                text = line[1:].strip()
                if text == "#[test]":
                    pending, ignored = True, False
                elif pending and text.startswith("#[ignore"):
                    ignored = True
                elif pending and (match := re.match(r"fn (\w+)\(", text)):
                    found.append((current, match[1], ignored))
                    pending = False
    return found


def pixel_faults(before, after):
    """Byte-for-byte comparison of the frames a build without the patches and one with them wrote."""
    names = lambda root: {str(p.relative_to(root)) for p in Path(root).rglob("*.png")}
    left, right = names(before), names(after)
    faults = [f"only one side has {name}" for name in sorted(left ^ right)]
    faults += [f"differs: {name}" for name in sorted(left & right) if sha(Path(before) / name) != sha(Path(after) / name)]
    routes = {Path(name).parts[0] for name in left | right}
    if routes != ROUTES:
        faults.append(f"routes written: {sorted(routes)}")
    return faults


def cargo(sub, *rest):
    return ("cargo", sub, "--locked", "--offline", *rest)


@dataclass
class Job:
    bound_gib: float  # Upper estimate of target growth from root's receipts, not a measurement.
    steps: list = field(default_factory=list)


def jobs(stage):
    engine, kit, lab = stage / "engine", stage / "repo-view", stage / "crate/Cargo.toml"
    budget = ("--config", str(stage / "budget.toml"))
    added = tests_added()
    fmn = ("-p", "fmn", "--test", "native_render", "--test", "native_bundle_pipeline",
           "--test", "native_camera_pipeline", "--test", "secondary_default_pixels")
    base = ("-p", "fmn", "--test", "secondary_default_pixels", "--test", "native_camera_pipeline")
    kit_tests = ("-p", "explainer_kit", "--lib", "--test", "options_hook_routes", "--test", "camera_chapter",
                 "--test", "glyph_probe", "--test", "cli_pipeline")
    probe_name = "every_native_route_writes_its_default_frames"

    def e(*argv, **kw):
        return dict(cwd=engine, target="geometry-native01", argv=argv, **kw)

    def k(*argv, **kw):
        return dict(cwd=kit, target="geometry-kit01", argv=argv, **kw)

    def f(*argv, **kw):  # The repository root's .cargo/config.toml applies, as in lab-build01.
        return dict(cwd=REPO, target="geometry-kit01", argv=argv, **kw)

    def names(prefix, ignored=False):
        return tuple(name for file, name, skip in added if file.startswith(prefix) and skip == ignored)

    def lock(*flags_and_paths):
        return ("python3", "-I", "-B", str(V / "verify_patched_lock.py"), *flags_and_paths)

    def ignored(test, name):
        return e(*cargo("test", "-p", "fmn-render", "--test", test), "--", "--ignored", "--exact", name,
                 must=(name,), timed=True, limit=HOUR)

    changed = [*patched_files(), f"crates/fmn/tests/{PROBE.name}"]
    return {
        # Resolves all three workspaces and proves the locks; compiles nothing.
        "deps": Job(0.0, [
            e("rustc", "-vV"), e("cargo", "-V"),
            e(*cargo("metadata", "--format-version", "1"), out="engine-metadata.json"),
            k(*cargo("metadata", *budget, "--format-version", "1"), out="kit-metadata.json"),
            k(*lock("--subset", str(stage / "Cargo.lock.view-pinned"), str(kit / "Cargo.lock"))),
            f(*cargo("metadata", *budget, "--manifest-path", str(lab), "--format-version", "1"), out="film-metadata.json"),
            f(*lock(str(stage / "Cargo.lock.pinned"), str(stage / "crate/Cargo.lock"))),
        ]),
        # Bound: all of target/geometry-native01 today, 1.68 GiB.
        "base-build": Job(1.68, [e(*cargo("test", *base, "--no-run"))]),
        "base-run": Job(0.0, [
            e(*cargo("test", *base, "--no-fail-fast"), env={"FMN_DEFAULT_PIXELS_OUT": str(stage / "pixels/base")},
              must=(probe_name,), limit=HOUR),
        ]),
        # The two patches go on here, and their hashes are checked, before this job's manifest.
        # Bound: complex-geometry-native-check03 grew 0.35 GiB.
        "engine-lint": Job(0.35, [
            e("rustfmt", "--check", "--edition", "2024", "--config", "skip_children=true", *changed),
            e(*cargo("clippy", "-p", "fmn-render", "-p", "fmn", "--all-targets"), "--", "-D", "warnings"),
        ]),
        # Same stage path as base-build, so Cargo replaces those artifacts. Same bound.
        "engine-build": Job(1.68, [
            e(*cargo("test", "-p", "fmn-render", "--no-run")),
            e(*cargo("test", *fmn, "--no-run")),
        ]),
        # Ordinary budget tests: serial and worker, camera and affine, lowered and raised.
        "engine-test": Job(0.0, [
            e(*cargo("test", "-p", "fmn-render", "--no-fail-fast"), must=names("crates/fmn-render/"), limit=HOUR),
            e(*cargo("test", *fmn, "--no-fail-fast"), env={"FMN_DEFAULT_PIXELS_OUT": str(stage / "pixels/combined")},
              must=(*names("crates/fmn/"), probe_name), limit=HOUR),
            e(call=lambda: pixel_faults(stage / "pixels/base", stage / "pixels/combined")),
        ]),
        # The two controls over the real defaults. Their memory is printed, not assumed.
        "engine-ignored": Job(0.0, [
            ignored(Path(file).stem, name) for file, name, skip in added if skip
        ]),
        # The only literal of a changed struct outside the patched files is perf_pg6_peak.rs:719, in
        # fmn-conformance's library. --lib leaves out its dev-dependencies (pyo3, studio, wasm), which
        # were never built on this stack. Bound: a guess, as fmn-conformance itself was never built.
        "engine-fronts": Job(0.5, [
            e(*cargo("check", "-p", "fmn-cli", "--all-targets")),
            e(*cargo("check", "-p", "fmn-conformance", "--lib")),
        ]),
        # Bound: all of target/geometry-kit01 today, 2.22 GiB.
        "kit-build": Job(2.22, [
            k("rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml",
              "crates/explainer_kit/tests/options_hook_routes.rs"),
            k(*cargo("clippy", *budget, "-p", "explainer_kit", "--all-targets", "--no-deps"), "--", "-D", "warnings"),
            k(*cargo("test", *budget, *kit_tests, "--no-run")),
        ]),
        # 100 + 1 + 6 + 3 + 1 passed in complex-geometry-kit-check03.
        "kit-test": Job(0.0, [
            k(*cargo("test", *budget, *kit_tests, "--no-fail-fast"), must=("the_hook_governs_every_render_route",),
              min_passed=111, limit=HOUR),
        ]),
        # Bound: complex-geometry-lab-build01 grew 1.00 GiB. Clippy links nothing, so root's film binaries stay.
        "film-check": Job(1.0, [
            f(*cargo("clippy", *budget, "--manifest-path", str(lab), "--bin", "complex_analysis_visual",
                     "--example", "plate_lab", "--no-deps"), "--", "-D", "warnings"),
        ]),
    }


def manifest(stage, job, attempt):
    """What root's guard hashes before and after one job."""
    paths = [path for name in TREES for path in files_under(stage / name) if path.is_file()]
    paths += [stage / "budget.toml", *(stage / name for name in PINNED_LOCKS)]
    paths += [Path(__file__).resolve(), GUARD, spec_load(GUARD, "guard_wrapper").SOURCE, *(V / tool for tool in TOOLS)]
    paths += [path for path in FILM_BINARIES if path.exists()]
    out = stage / "manifests" / f"{job}{attempt}.json"
    with out.open("x") as file:
        json.dump({"files": [{"path": str(path.resolve()), "sha256": sha(path)} for path in paths]}, file)
    return out


def run_step(step, stage, active):
    """One step: its output goes to root's command log and is scanned for test results."""
    if "call" in step:
        faults = step["call"]()
        print(f"default pixels: {'identical' if not faults else faults}", flush=True)
        return {"check": "default pixels", "faults": faults, "exit": 1 if faults else 0}
    env = dict(os.environ, CARGO_INCREMENTAL="0", CARGO_TARGET_DIR=str(TARGET / step["target"]), **step.get("env", {}))
    argv = (["/usr/bin/time", "-l"] if step.get("timed") else []) + list(step["argv"])
    print("$", " ".join(argv), flush=True)
    sink = open(stage / "results" / step["out"], "w") if step.get("out") else subprocess.PIPE
    started = time.monotonic()
    child = subprocess.Popen(argv, cwd=step["cwd"], env=env, text=True, start_new_session=True,
                             stdout=sink, stderr=subprocess.PIPE if step.get("out") else subprocess.STDOUT)
    active.append(child)
    timer = threading.Timer(step.get("limit", 2 * HOUR), os.killpg, (child.pid, signal.SIGTERM))
    timer.start()
    seen = scan(child.stderr if step.get("out") else child.stdout, sys.stdout)
    code = child.wait()
    timer.cancel()
    active.pop()
    if step.get("out"):
        sink.close()
    missing = [name for name in step.get("must", ()) if name not in seen["ok"]]
    short = seen["passed"] < step.get("min_passed", 0)
    return {"argv": argv, "exit": code, "seconds": round(time.monotonic() - started, 1), "passed": seen["passed"],
            "max_rss": seen["rss"], "peak_footprint": seen["footprint"], "missing": missing, "short": short}


def scan(stream, echo):
    seen = {"ok": set(), "passed": 0, "rss": None, "footprint": None}
    for line in stream:
        echo.write(line)
        echo.flush()
        if match := re.fullmatch(r"test (\S+) \.\.\. ok\n", line):
            seen["ok"].add(match[1].rsplit("::", 1)[-1])
        elif match := re.match(r"test result: \w+\. (\d+) passed", line):
            seen["passed"] += int(match[1])
        elif match := re.fullmatch(r"\s*(\d+)  maximum resident set size\n", line):
            seen["rss"] = int(match[1])
        elif match := re.fullmatch(r"\s*(\d+)  peak memory footprint\n", line):
            seen["footprint"] = int(match[1])
    return seen


def run_job(name, stage, attempt):
    """Runs as the command root's guard launches, in its process group, with its environment."""
    if os.environ.get("CARGO_TARGET_DIR") != str(TARGET) or os.environ.get("RCH_SHIM_LOCAL_IDE") != "1":
        raise SystemExit("job runs only under the guard")
    active = []

    def stop(*_):
        for child in active:
            os.killpg(child.pid, signal.SIGTERM)
        raise SystemExit(143)

    signal.signal(signal.SIGTERM, stop)
    results = []
    code = 0
    for step in jobs(stage)[name].steps:
        results.append(run_step(step, stage, active))
        last = results[-1]
        if last["exit"] or last.get("missing") or last.get("short"):
            print("FAILED STEP", json.dumps(last), flush=True)
            code = last["exit"] or 1
            break
    with open(stage / "results" / f"{name}{attempt}.json", "x") as file:
        json.dump(results, file, indent=2)
    return code


def headroom():
    guard = spec_load(GUARD, "guard_wrapper").load()
    now = guard.sample()
    baseline = json.loads((guard.WORK / "allocation-baseline.json").read_text())
    gib = 1024 ** 3
    print(f"free {now['free_bytes'] / gib:.2f} GiB; admission {guard.ADMISSION_GIB['native']}, stop {guard.STOP_FREE_GIB}")
    print(f"work {now['work_bytes'] / gib:.2f} of {guard.WORK_CAP_GIB} GiB")
    grown = now["target_bytes"] - baseline["target_bytes"]
    print(f"target growth {grown / gib:.2f} of {guard.TARGET_GROWTH_CAP_GIB} GiB, over the baseline of {baseline['utc']}")
    print(f"growth per native job {guard.CAPS['native']} GiB")
    return now["free_bytes"] / gib


def plan(stage):
    headroom()
    table = jobs(stage)
    print(f"upper bounds, estimates from root's receipts: {sum(job.bound_gib for job in table.values()):.2f} GiB in all")
    for name, job in table.items():
        print(f"\n[{name}] bound {job.bound_gib} GiB")
        if name == "engine-lint":
            print("  before this job: git apply both patches and check their sealed hashes")
        for step in job.steps:
            print("  ", "compare pixels/base with pixels/combined" if "call" in step else " ".join(step["argv"]))


def run_all(stage, start, attempt, inputs):
    if not stage.is_dir():
        raise SystemExit("run `stage` first")
    table = list(jobs(stage))
    if start not in table:
        raise SystemExit(f"--from must be one of {table}")
    faults = changed_inputs(inputs, not_source)
    if faults:
        raise SystemExit("dependencies changed:\n  " + "\n  ".join(faults))
    if headroom() < RUN_FLOOR_GIB:
        raise SystemExit(f"under {RUN_FLOOR_GIB} GiB free: the later jobs would hit the 8 GiB stop")
    py = ["python3", "-I", "-B"]
    for name in table[table.index(start):]:
        if table.index(name) >= table.index("engine-lint"):
            apply_stack(stage)
        elif unpatched_faults(stage):
            raise SystemExit(f"{name} needs root's unpatched bytes, and this stage is patched: stage a new copy")
        command = [*py, str(GUARD), "--phase", "native", "--job", f"engine-secondary-{name}{attempt}",
                   "--inputs", str(manifest(stage, name, attempt)), "--",
                   *py, str(Path(__file__).resolve()), "job", name, "--stage", str(stage), "--attempt", attempt]
        code = subprocess.run(command, cwd=REPO, check=False).returncode
        if code:
            print(f"STOPPED at {name}: exit {code}" + (" (resource stop)" if code == 75 else ""))
            return code
    faults = changed_inputs(inputs, not_source)
    print("dependencies unchanged" if not faults else f"DEPENDENCIES CHANGED DURING THE RUN: {faults}")
    print("ALL JOBS PASSED" if not faults else "NOT A PASS")
    return 1 if faults else 0


@contextmanager
def preserved_test_directory():
    """Retain every test fixture, including failed controls, for inspection."""
    path = Path(tempfile.mkdtemp(prefix="geometry-secondary-root-selftest-"))
    try:
        yield path
    finally:
        print(f"preserved self-test fixtures: {path}")


def self_test():
    # Test names come from the patches, so a typo cannot pass a run that never ran the test.
    added = tests_added()
    assert len(added) == 20 and sum(skip for _, _, skip in added) == 2, added
    assert {name for _, name, skip in added if skip} == {
        "a_monotone_budget_above_the_default_is_admitted_and_the_default_still_refuses",
        "a_working_byte_budget_above_the_default_is_admitted_and_the_default_still_refuses",
    }
    print(f"pass: {len(added)} tests from the two patches, 2 ignored")

    seen = scan(io.StringIO("test a::b::c ... ok\ntest d ... FAILED\ntest e ... ignored, slow\n"
                            "test result: ok. 3 passed; 0 failed\ntest result: FAILED. 4 passed; 1 failed\n"
                            "  123  maximum resident set size\n  45  peak memory footprint\n"), io.StringIO())
    assert seen == {"ok": {"c"}, "passed": 7, "rss": 123, "footprint": 45}, seen
    print("pass: only `ok` lines count as run, and memory lines are read")

    with preserved_test_directory() as tmp:
        root = Path(tmp).resolve()
        for side in ("a", "b"):
            for route in ROUTES:
                (root / side / route).mkdir(parents=True)
                (root / side / route / "frame_000000.png").write_bytes(b"same")
        assert not pixel_faults(root / "a", root / "b")
        (root / "b/camera-rig/frame_000000.png").write_bytes(b"other")
        assert pixel_faults(root / "a", root / "b") == ["differs: camera-rig/frame_000000.png"]
        (root / "b/camera-rig/frame_000000.png").write_bytes(b"same")
        (root / "b/camera-rig/frame_000001.png").write_bytes(b"same")
        assert pixel_faults(root / "a", root / "b") == ["only one side has camera-rig/frame_000001.png"]
        (root / "c/affine-serial").mkdir(parents=True)
        (root / "c/affine-serial/frame_000000.png").write_bytes(b"same")
        assert pixel_faults(root / "c", root / "c") and "routes written" in pixel_faults(root / "c", root / "c")[0]
        print("pass: pixel comparison catches a changed frame, an extra frame and a missing route")

        first = tree_row(root / "a")
        (root / "a/camera-rig/frame_000000.png").write_bytes(b"Same")
        assert tree_row(root / "a") != first
        (root / "a/camera-rig/frame_000000.png").write_bytes(b"same")
        assert tree_row(root / "a") == first
        (root / "a/camera-rig/frame_000000.png").rename(root / "a/camera-rig/moved.png")
        assert tree_row(root / "a") != first
        print("pass: tree digests catch an edit and a rename")
        guard_cases(root)
    print("self-test passed: Python only; nothing was compiled, rendered or queued")


def guard_cases(root):
    """The caps of the loaded guard, driven with fake samples in a temporary lane. No real job runs."""
    gib = 1024 ** 3
    real = spec_load(GUARD, "guard_wrapper")
    baseline = json.loads((real.WORK / "allocation-baseline.json").read_text())
    base = baseline["target_bytes"]

    class Shim:  # Everything from subprocess but the sbh gate.
        Popen, TimeoutExpired, STDOUT = subprocess.Popen, subprocess.TimeoutExpired, subprocess.STDOUT

        @staticmethod
        def run(*_, **__):
            return subprocess.CompletedProcess([], 0, "{}", "")

    def state(free, work, grown):
        return {"utc": "t", "free_bytes": int(free * gib), "work_bytes": int(work * gib),
                "target_bytes": int(base + grown * gib)}

    calm = (40, 44, 10)
    cases = [
        ("admitted just inside the caps", (10, 55.9, 19.9), (10, 55.9, 19.9), 0, False),
        ("refused: free under 10 GiB", (9.9, 44, 10), None, 75, None),
        ("refused: work at 56 GiB", (40, 56, 10), None, 75, None),
        ("refused: target growth over 20 GiB", (40, 44, 20.1), None, 75, None),
        ("stopped: free under 8 GiB", calm, (7.9, 44, 10), 75, True),
        ("stopped: job grew over 4 GiB", calm, (40, 44, 14.1), 75, True),
        ("stopped: work over 56 GiB", (40, 55.5, 10), (40, 56.1, 10), 75, True),
        ("stopped: target growth over 20 GiB", (40, 44, 18), (40, 44, 20.1), 75, True),
    ]
    sys_argv = sys.argv
    out = sys.stdout
    try:
        for index, (label, first, later, want, stopped) in enumerate(cases):
            guard = real.load()
            assert guard.ROOT == real.SOURCE.parent
            lane = root / f"lane{index}"
            (lane / "work").mkdir(parents=True)
            (lane / "work/allocation-baseline.json").write_text(json.dumps(baseline))
            guard.ROOT, guard.REPO, guard.WORK, guard.subprocess = lane, lane, lane / "work", Shim
            samples = iter([state(*first)] + [state(*later)] * 3 if later else [state(*first)])
            guard.sample = lambda: next(samples)
            inputs = lane / "inputs.json"
            inputs.write_text(json.dumps({"files": [{"path": str(inputs.parent / "x"), "sha256": hashlib.sha256(b"").hexdigest()}]}))
            (lane / "x").write_bytes(b"")
            sys.argv = ["guard", "--phase", "native", "--job", "case", "--inputs", str(inputs), "--",
                        "sleep", "0" if want == 0 else "30"]
            sys.stdout = io.StringIO()
            code = guard.main()
            sys.stdout = out
            assert code == want, (label, code)
            receipt = json.loads((lane / "work/case/resource-receipt.json").read_text())
            assert receipt["policy"] == {"admission_gib": 10, "stop_free_gib": 8, "phase_growth_gib": 4, "work_cap_gib": 56,
                                         "target_growth_cap_gib": 20, "sample_seconds": 10}, receipt["policy"]
            if stopped is not None:
                assert receipt.get("stopped_for_resource_limit", False) == stopped, label
            assert json.loads((lane / "work/allocation-baseline.json").read_text()) == baseline
            print(f"pass: {label}")
    finally:
        sys.argv, sys.stdout = sys_argv, out


def main():
    if sys.argv[1:] == ["--self-test"]:
        return self_test()
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("action", choices=("plan", "bind", "stage", "run", "job"))
    parser.add_argument("name", nargs="?", help="job name, for the guard's own call")
    parser.add_argument("--source", type=Path)
    parser.add_argument("--stage", type=Path)
    parser.add_argument("--inputs", type=Path, default=INPUTS)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--from", dest="start", default="deps")
    parser.add_argument("--attempt", default="01")
    args = parser.parse_args()
    if args.action in ("bind", "stage", "run") and not args.source:
        parser.error(f"{args.action} needs --source")
    if args.action != "bind" and not args.stage:
        parser.error(f"{args.action} needs --stage")
    global SOURCE
    SOURCE = args.source.resolve() if args.source else None
    if args.action == "plan":
        return plan(args.stage)
    if args.action == "bind":
        return bind(args.out)
    if args.action == "stage":
        return make_stage(args.stage, args.inputs)
    if args.action == "job":
        return run_job(args.name, args.stage, args.attempt)
    return run_all(args.stage, args.start, args.attempt, args.inputs)


if __name__ == "__main__":
    raise SystemExit(main())
