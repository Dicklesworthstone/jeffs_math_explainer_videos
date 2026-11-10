#!/usr/bin/env python3
"""Sweep real 3b1b scenes through an installed fmn-python portal (fm-5wq.14).

The README promises that existing manim scenes run source-unedited. Until
this sweep, the only measurement was eight allowlisted seed scenes. This
tool enumerates every Scene subclass in the pinned 3b1b/videos checkout for
the chosen years, renders each one's final frame through the portal with a
timeout, and classifies the outcome with a closed vocabulary. A portal-side
Python exception is a bug, never a refusal.

The corpus is licensing-restricted (CC BY-NC-SA): records carry scene
identifiers and outcome codes only, never scene source.

Usage:
    corpus_scene_sweep.py --portal PATH/TO/fmn-python --videos scripts/videos_ref \
        --years 2020-2026 --out DIR [--jobs 16] [--timeout 180] [--limit N]

Writes DIR/records.ndjson (one JSON record per scene, sorted) and prints a
summary; `--summary-tsv` also writes the per-outcome counts. Each record
carries its era (year), wall time, the child's own peak RSS (KiB, from
os.wait4), and `portal_digest`, a sha256 over the imported manimlib and
fmn_python trees. `--construct-check` re-runs every scene that did not
render with the portal's `--construct-only` and records
`construct_outcome`: a scene that constructs but does not render failed in
the render layer.

The portal CLI reports one structured error line without a traceback, so an
ordinary exception is first recorded as `runtime_error` (origin unknown).
`--attribute --python PATH/TO/portal/python` re-runs each of those scenes
in-process and records which code raised it (`raised_in`: portal or scene),
splitting them into `portal_exception` and `scene_code_error`. Raised-in is
where the exception surfaced, not proof of fault: an AttributeError raised in
scene code can still come from the portal returning the wrong type. A
TypeError the pinned Reference raises identically counts as scene code even
when a portal entry point raises it: a signature-binding error (portal
signatures follow the Reference's API schema) or prepare_animation's "cannot
be converted to an animation". `--report` re-derives that split from each
record's evidence, so a rule fix applies to existing records.
"""
import argparse
import ast
import concurrent.futures
import hashlib
import json
import os
import pathlib
import re
import signal
import subprocess
import sys
import threading
import time

SCENE_ROOTS = {"Scene", "ThreeDScene", "InteractiveScene"}

# Closed outcome vocabulary, in precedence order.
OUTCOMES = (
    "ok",
    "capability_refusal",
    "portal_unbound",
    "tex_unsupported",
    "missing_dependency",
    "missing_asset",
    "scene_code_error",
    "portal_exception",
    "runtime_error",
    "timeout",
    "crash",
)


def enumerate_scenes(videos: pathlib.Path, years: range):
    """Scene subclasses by static AST analysis.

    A class is a scene when a base is a scene root or a scene class known in
    the same module; imported custom bases ending in "Scene" also count.
    Classes whose names start with "_" and abstract helpers without a
    construct() anywhere in their module lineage are still attempted: the
    portal's own verdict is the measurement.
    """
    scenes = []
    for year in years:
        root = videos / f"_{year}"
        if not root.is_dir():
            continue
        for path in sorted(root.rglob("*.py")):
            try:
                tree = ast.parse(path.read_text(encoding="utf-8"))
            except (SyntaxError, UnicodeDecodeError):
                continue
            bases = {}
            for node in tree.body:
                if isinstance(node, ast.ClassDef):
                    names = []
                    for base in node.bases:
                        if isinstance(base, ast.Name):
                            names.append(base.id)
                        elif isinstance(base, ast.Attribute):
                            names.append(base.attr)
                    bases[node.name] = names
            known = set()
            changed = True
            while changed:
                changed = False
                for name, parents in bases.items():
                    if name in known:
                        continue
                    if any(p in SCENE_ROOTS or p in known or p.endswith("Scene") for p in parents):
                        known.add(name)
                        changed = True
            rel = path.relative_to(videos).as_posix()
            scenes.extend((rel, name) for name in sorted(known))
    return scenes


def classify(returncode, stderr, timed_out):
    if timed_out:
        return "timeout", "exceeded timeout"
    if returncode == 0:
        return "ok", ""
    tail = stderr.strip().splitlines()[-1] if stderr.strip() else ""
    missing = re.search(r"No module named '([^']+)'", stderr)
    if missing and "manimlib" not in missing.group(1) and "fmn_python" not in missing.group(1):
        return "missing_dependency", missing.group(1)
    # fmn-python: scene/<kind>: <ExceptionType>: <message>
    parsed = re.search(r"scene/[a-z-]+: ([A-Za-z_.]+): (.*)", tail)
    exc_type = parsed.group(1).rsplit(".", 1)[-1] if parsed else ""
    if "semantic binding has not landed" in tail:
        return "portal_unbound", tail[:300]
    if (
        returncode == 4
        or "capability/" in tail
        or exc_type in ("NotImplementedError", "CapabilityError")
        # Only the bundled faces ship; a system font name is a named refusal.
        or re.search(r"font family '[^']+' is not available", tail)
    ):
        return "capability_refusal", tail[:300]
    if re.search(r"UnsupportedCommand|TexError|fmd-math|is not yet supported", stderr):
        return "tex_unsupported", tail[:300]
    # "<name> not Found" is ImageMobject's Reference-worded missing-file error.
    if re.search(r"FileNotFoundError|No such file|asset| not Found$", tail):
        return "missing_asset", tail[:300]
    if returncode < 0 or returncode > 100:
        return "crash", f"exit {returncode}"
    return "runtime_error", tail[:300]


# Re-run one scene in-process and report where its exception surfaced.
ATTRIBUTION_RUNNER = r"""
import importlib, json, os, sys, traceback
videos, rel, scene, dest = sys.argv[1:5]
sys.path.insert(0, videos)
module = rel[:-3].replace("/", ".")
try:
    cls = getattr(importlib.import_module(module), scene)
    cls().render(dest, format="png", resolution=(320, 180))
    print(json.dumps({"raised_in": "none"}))
except BaseException as error:
    origin, frames = "other", []
    for frame in reversed(traceback.extract_tb(error.__traceback__)):
        name = frame.filename
        if name.startswith(videos):
            where = "scene"
        elif "manimlib" in name or "fmn_python" in name:
            where = "portal"
        else:
            continue
        frames.append([where, os.path.basename(name), frame.lineno, frame.name])
        if origin == "other":
            origin = where
    print(json.dumps({"raised_in": origin, "type": type(error).__name__, "frames": frames[:4]}))
"""


def scene_env(args):
    # The corpus root goes first; an inherited PYTHONPATH (e.g. a portal
    # build under test) stays importable after it.
    inherited = os.environ.get("PYTHONPATH")
    path = str(args.videos) + (os.pathsep + inherited if inherited else "")
    return dict(os.environ, PYTHONPATH=path)


# TypeErrors the pinned Reference raises identically: portal signatures come
# from its API schema, and prepare_animation's refusal (animation.py:216) is
# reproduced verbatim. Raised at a portal entry point, they are scene code.
REFERENCE_REJECTION = re.compile(
    r"TypeError: .*(unexpected keyword argument|missing \d+ required|takes \d+ positional|"
    r"got multiple values for argument|cannot be converted to an animation)"
)


def attributed_outcome(record):
    """portal_exception or scene_code_error, from a record's recorded
    attribution evidence alone, so --report re-derives it under the same rule."""
    if record["raised_in"] == "portal" and not REFERENCE_REJECTION.search(
        record.get("detail", "")
    ):
        return "portal_exception"
    return "scene_code_error"


def attribute(args, record, out_dir):
    target = out_dir / "attribution" / f"{record['module'].replace('/', '__')}__{record['scene']}.png"
    target.parent.mkdir(parents=True, exist_ok=True)
    try:
        proc = subprocess.run(
            [args.python, "-c", ATTRIBUTION_RUNNER, str(args.videos), record["module"],
             record["scene"], str(target)],
            cwd=args.workdir, env=scene_env(args), capture_output=True, text=True,
            timeout=args.timeout,
        )
        found = json.loads(proc.stdout.strip().splitlines()[-1])
    except (subprocess.TimeoutExpired, ValueError, IndexError):
        return record
    record = dict(record, raised_in=found["raised_in"], frames=found.get("frames", []))
    if found["raised_in"] in ("scene", "portal"):
        record["outcome"] = attributed_outcome(record)
    return record


def run_child(argv, cwd, env, timeout):
    """Run one portal child to completion or to its timeout, killing its
    whole process group on expiry. Returns (exit code or None, stderr,
    peak RSS in KiB): the peak is the child's own ru_maxrss from os.wait4,
    so concurrent children never share one figure."""
    proc = subprocess.Popen(argv, cwd=cwd, env=env, stdout=subprocess.DEVNULL,
                            stderr=subprocess.PIPE, text=True, start_new_session=True)
    chunks = []
    reader = threading.Thread(target=lambda: chunks.append(proc.stderr.read()), daemon=True)
    reader.start()
    deadline = time.monotonic() + timeout
    timed_out = False
    while True:
        pid, status, usage = os.wait4(proc.pid, os.WNOHANG)
        if pid:
            break
        if time.monotonic() >= deadline:
            timed_out = True
            os.killpg(proc.pid, signal.SIGKILL)
            pid, status, usage = os.wait4(proc.pid, 0)
            break
        time.sleep(0.05)
    proc.returncode = os.waitstatus_to_exitcode(status)
    reader.join()
    proc.stderr.close()
    return (None if timed_out else proc.returncode), "".join(chunks), usage.ru_maxrss


def run_one(args, rel, scene, out_dir):
    target = out_dir / "frames" / f"{rel.replace('/', '__')}__{scene}.png"
    target.parent.mkdir(parents=True, exist_ok=True)
    start = time.monotonic()
    code, stderr, peak = run_child(
        [args.portal, str(args.videos / rel), scene, "--format", "png",
         "--resolution", "320x180", "--video_dir", str(target)],
        args.workdir, scene_env(args), args.timeout,
    )
    if code is None:
        outcome, detail = "timeout", f"exceeded {args.timeout}s"
    else:
        outcome, detail = classify(code, stderr, False)
    return {
        "module": rel,
        "scene": scene,
        "era": rel.split("/", 1)[0].lstrip("_"),
        "outcome": outcome,
        "detail": detail,
        "exit": code,
        "seconds": round(time.monotonic() - start, 2),
        "peak_rss_kib": peak,
        "portal_digest": args.portal_digest,
    }


def construct_check(args, record):
    """Re-run a scene that did not render with --construct-only, which runs
    its whole lifecycle without rasterizing. A scene that constructs but
    does not render failed in the render layer."""
    start = time.monotonic()
    code, stderr, peak = run_child(
        [args.portal, str(args.videos / record["module"]), record["scene"], "--construct-only"],
        args.workdir, scene_env(args), args.timeout,
    )
    outcome, _ = ("timeout", "") if code is None else classify(code, stderr, False)
    return dict(record, construct_outcome=outcome,
                construct_seconds=round(time.monotonic() - start, 2),
                construct_peak_rss_kib=peak)


def portal_digest(python, env):
    """sha256 over every file (bytecode caches aside) of the manimlib and
    fmn_python package trees the portal interpreter imports: the exact
    portal code under test, whether from a wheel or a build tree."""
    code = ("import os, manimlib, fmn_python; "
            "print(os.path.dirname(manimlib.__file__)); "
            "print(os.path.dirname(fmn_python.__file__))")
    roots = subprocess.run([python, "-c", code], env=env, capture_output=True, text=True,
                           timeout=300, check=True).stdout.split("\n")
    digest = hashlib.sha256()
    for index, root in enumerate(filter(None, roots)):
        root = pathlib.Path(root)
        for path in sorted(root.rglob("*")):
            if path.is_file() and "__pycache__" not in path.parts:
                digest.update(f"{index}/{path.relative_to(root).as_posix()}".encode() + b"\0")
                digest.update(path.read_bytes())
    return digest.hexdigest()


def portal_python(args):
    """The portal environment's interpreter: --python, else the console
    script's shebang."""
    if args.python:
        return args.python
    with open(args.portal, "rb") as handle:
        first = handle.readline().decode("utf-8", "replace").strip()
    if not first.startswith("#!"):
        raise SystemExit("--python is required when --portal has no shebang")
    return first[2:].split()[0]


def cluster_label(record):
    """A failure's cluster: exception type plus its message with every
    quoted literal that is not a bare identifier elided, so scene-authored
    text (TeX, labels) never reaches a committed artifact. Absolute paths
    are elided too: they name the sweep host's directories or the author's."""
    detail = record.get("detail", "")
    detail = re.sub(r"^fmn-python: scene/[a-z-]+: ", "", detail)
    detail = re.sub(r"\bat 0x[0-9a-f]+", "", detail)
    detail = re.sub(r"(?<![\w.])/(?:[\w.@+-]+/)+[^\s'\"]*", "…", detail)
    detail = re.sub(r"'([^']*)'", lambda m: m.group(0) if re.fullmatch(r"[\w.()]{1,40}", m.group(1)) else "'…'", detail)
    detail = re.sub(r'"[^"]*"', '"…"', detail)
    detail = re.sub(r"\(bytes \d+\.\.\d+\)", "", detail)
    return detail[:140]


def write_report(records, dashboard, tsv, title, note=None):
    counts = {name: 0 for name in OUTCOMES}
    for record in records:
        counts[record["outcome"]] += 1
    total = len(records)
    by_year = {}
    modules = {}
    for record in records:
        year = record["module"].split("/", 1)[0]
        by_year.setdefault(year, {name: 0 for name in OUTCOMES})[record["outcome"]] += 1
        modules.setdefault(record["module"], []).append(record["outcome"] == "ok")
    module_ok = sum(1 for outcomes in modules.values() if all(outcomes))
    with open(tsv, "w", encoding="utf-8") as handle:
        handle.write("module\tscene\toutcome\texception\n")
        for record in sorted(records, key=lambda r: (r["module"], r["scene"])):
            exception = ""
            match = re.search(r"scene/[a-z-]+: ([A-Za-z_.]+):", record.get("detail", ""))
            if match:
                exception = match.group(1).rsplit(".", 1)[-1]
            handle.write(f"{record['module']}\t{record['scene']}\t{record['outcome']}\t{exception}\n")
    lines = [f"# {title}", ""]
    if note:
        lines += [note.strip(), ""]
    lines.append(
        f"{total} scene classes; **{counts['ok']} ok ({100 * counts['ok'] / total:.1f}%)**. "
        f"Module-weighted: {module_ok} of {len(modules)} modules render every scene "
        f"({100 * module_ok / len(modules):.1f}%)."
    )
    lines += ["", "| outcome | scenes | share |", "|---|---:|---:|"]
    for name in OUTCOMES:
        lines.append(f"| {name} | {counts[name]} | {100 * counts[name] / total:.1f}% |")
    lines += ["", "| year | scenes | ok | ok % |", "|---|---:|---:|---:|"]
    for year in sorted(by_year):
        year_counts = by_year[year]
        year_total = sum(year_counts.values())
        lines.append(
            f"| {year} | {year_total} | {year_counts['ok']} | {100 * year_counts['ok'] / year_total:.1f}% |"
        )
    for outcome in ("portal_exception", "portal_unbound", "scene_code_error", "capability_refusal",
                    "tex_unsupported", "missing_dependency", "missing_asset"):
        clusters = {}
        for record in records:
            if record["outcome"] == outcome:
                label = cluster_label(record)
                clusters[label] = clusters.get(label, 0) + 1
        if not clusters:
            continue
        lines += ["", f"## {outcome}: top clusters", "", "| scenes | cluster |", "|---:|---|"]
        for label, count in sorted(clusters.items(), key=lambda item: (-item[1], item[0]))[:10]:
            lines.append(f"| {count} | {label.replace('|', '/')} |")
    with open(dashboard, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--portal")
    parser.add_argument("--videos", type=pathlib.Path)
    parser.add_argument("--report", type=pathlib.Path,
                        help="regenerate --dashboard/--tsv from an existing records.ndjson")
    parser.add_argument("--dashboard", type=pathlib.Path)
    parser.add_argument("--tsv", type=pathlib.Path)
    parser.add_argument("--title", default="Corpus scene sweep")
    parser.add_argument("--note", type=pathlib.Path,
                        help="a Markdown paragraph describing the run's environment")
    parser.add_argument("--years", default="2020-2026")
    parser.add_argument("--out", type=pathlib.Path)
    parser.add_argument("--jobs", type=int, default=16)
    parser.add_argument("--timeout", type=int, default=180)
    parser.add_argument("--limit", type=int, default=0)
    parser.add_argument("--summary-tsv", type=pathlib.Path)
    parser.add_argument("--attribute", action="store_true",
                        help="re-run runtime_error scenes in-process to find where they raised")
    parser.add_argument("--python", help="the portal environment's python (for --attribute)")
    parser.add_argument("--construct-check", action="store_true",
                        help="re-run every scene that did not render with --construct-only")
    parser.add_argument(
        "--config", type=pathlib.Path,
        help="a custom_config.yml to run under, as a user with their own directories "
             "would; default: the corpus's own (its base is the author's machine)",
    )
    args = parser.parse_args()
    if args.report:
        if not (args.dashboard and args.tsv):
            parser.error("--report requires --dashboard and --tsv")
        records = [json.loads(line) for line in args.report.read_text(encoding="utf-8").splitlines()]
        for record in records:
            if record.get("raised_in") in ("scene", "portal"):
                record["outcome"] = attributed_outcome(record)
        note = args.note.read_text(encoding="utf-8") if args.note else None
        write_report(records, args.dashboard, args.tsv, args.title, note)
        return 0
    if not (args.portal and args.videos and args.out):
        parser.error("a sweep requires --portal, --videos and --out")
    if args.attribute and not args.python:
        parser.error("--attribute requires --python")
    args.videos = args.videos.resolve()
    first, _, last = args.years.partition("-")
    years = range(int(first), int(last or first) + 1)

    scenes = enumerate_scenes(args.videos, years)
    if args.limit:
        scenes = scenes[: args.limit]
    args.out.mkdir(parents=True, exist_ok=True)
    # Scenes run from a working directory whose custom_config.yml the portal
    # reads, exactly as the Reference reads the cwd's.
    args.workdir = args.videos
    if args.config:
        args.workdir = (args.out / "workdir").resolve()
        args.workdir.mkdir(exist_ok=True)
        (args.workdir / "custom_config.yml").write_text(
            args.config.read_text(encoding="utf-8"), encoding="utf-8"
        )
    print(f"enumerated {len(scenes)} scene classes in {args.years}", file=sys.stderr)
    args.portal_digest = portal_digest(portal_python(args), scene_env(args))
    print(f"portal digest {args.portal_digest}", file=sys.stderr)

    records = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
        futures = [pool.submit(run_one, args, rel, scene, args.out) for rel, scene in scenes]
        for done, future in enumerate(concurrent.futures.as_completed(futures), 1):
            records.append(future.result())
            if done % 50 == 0:
                print(f"{done}/{len(scenes)}", file=sys.stderr)

    if args.attribute:
        pending = [r for r in records if r["outcome"] == "runtime_error"]
        print(f"attributing {len(pending)} runtime errors", file=sys.stderr)
        keep = [r for r in records if r["outcome"] != "runtime_error"]
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
            keep.extend(pool.map(lambda r: attribute(args, r, args.out), pending))
        records = keep

    if args.construct_check:
        failed = [r for r in records if r["outcome"] != "ok"]
        print(f"construct-checking {len(failed)} scenes that did not render", file=sys.stderr)
        rendered = [r for r in records if r["outcome"] == "ok"]
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
            rendered.extend(pool.map(lambda r: construct_check(args, r), failed))
        records = rendered

    records.sort(key=lambda r: (r["module"], r["scene"]))
    with open(args.out / "records.ndjson", "w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record, sort_keys=True) + "\n")

    counts = {name: 0 for name in OUTCOMES}
    for record in records:
        counts[record["outcome"]] += 1
    total = len(records)
    for name in OUTCOMES:
        share = 100.0 * counts[name] / total if total else 0.0
        print(f"{name}\t{counts[name]}\t{share:.1f}%")
    if args.summary_tsv:
        with open(args.summary_tsv, "w", encoding="utf-8") as handle:
            handle.write("outcome\tscenes\n")
            for name in OUTCOMES:
                handle.write(f"{name}\t{counts[name]}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
