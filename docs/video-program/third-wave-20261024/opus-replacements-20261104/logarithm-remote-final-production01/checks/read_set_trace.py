#!/usr/bin/env python3
"""Does a render's frozen read set hold every file the real delivery contract opens? Read-only; runs no job.

The kit starts tools/narration_delivery.py before every render, master and caption run. This runs that tool for real,
under an open() audit hook, and compares what it opened with the rows render_master_remote.py freezes for the narration
and the source. The narration is stage09's, standing in for the new byte-identical assembly, so this covers the file
set and not the new bytes. Rust kit reads cannot be traced this way: a file only the kit opens would first show at runtime.

Usage, from the repository root:  python3.14 -I -B checks/read_set_trace.py SCRATCH RESULTS
SCRATCH and RESULTS must not exist.
"""
import hashlib
import importlib.util
import json
import os
import runpy
import subprocess
import sys
from pathlib import Path

GAIN_DB = "0"  # the kit's narration_gain_db: explainer_kit::narration::GAIN_DB = 0.0, as Rust's f64 Display prints it

if sys.argv[1:2] == ["--trace"]:
    out, script, *arguments = sys.argv[2:]
    seen = set()

    def hook(event, args):
        if event == "open" and isinstance(args[0], (str, bytes, os.PathLike)):
            seen.add(os.path.abspath(os.fsdecode(args[0])))

    sys.addaudithook(hook)
    sys.argv = [script, *arguments]
    stdout, sys.stdout = sys.stdout, open(os.devnull, "w")
    code = 0
    try:
        runpy.run_path(script, run_name="__main__")
    except SystemExit as stop:
        code = stop.code or 0
    sys.stdout = stdout
    Path(out).write_text(json.dumps(sorted(seen)))
    sys.exit(code)

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
REAL = HERE.parents[6]


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    scratch, results = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    if scratch.exists() or results.exists():
        raise SystemExit("SCRATCH and RESULTS must not exist")
    scratch.mkdir(parents=True)
    os.chdir(REAL)
    spec = importlib.util.spec_from_file_location("driver", BUNDLE / "render_master_remote.py")
    M = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(M)
    M.N = M.EL / "production-audio-stage09-01/prepared02/narration"
    command = [str(M.WORKSPACE / "tools/narration_delivery.py"), "--topic", str(M.CRATE), "--narration", str(M.N), "--consumer", "render",
               "--gain-db", GAIN_DB] + [x for chapter in M.CHAPTERS for x in ("--chapter-key", chapter)]
    out = scratch / "opened.json"
    done = subprocess.run([sys.executable, "-I", "-B", str(HERE), "--trace", str(out), *command], cwd=REAL, capture_output=True, text=True, timeout=500)
    opened = {p for p in json.loads(out.read_text()) if Path(p).is_file() and not p.startswith(sys.base_prefix)
              and "/opt/homebrew/" not in p and not p.startswith(str(scratch))}
    # binding["crate"] is every file under the crate, and the assembly's folder holds the narration
    base = {*map(str, M.tree(M.CRATE)), *map(str, M.tree(M.N)), *map(str, M.KIT_TOOLS)}
    reads = set(map(str, M.narration_reads()))
    outside = sorted(opened - base - reads)
    needed = sorted(opened - base)
    data = {"driver_sha256": hashlib.sha256((BUNDLE / "render_master_remote.py").read_bytes()).hexdigest(), "python": sys.version.split()[0],
            "tool_exit": done.returncode, "files_opened": len(opened), "outside_the_frozen_rows": [p.replace(str(REAL), "") for p in outside],
            "positive_control_files_only_narration_reads_covers": len(needed),
            "scope": "The Python delivery contract on stage09's narration. Rust kit reads are not traced. Nothing was rendered, heard or viewed."}
    with results.open("x") as handle:
        handle.write(json.dumps(data, indent=2) + "\n")
    print(data["tool_exit"], data["files_opened"], len(outside), len(needed), done.stderr[-200:])
    return 0 if done.returncode == 0 and not outside and needed else 1


if __name__ == "__main__":
    sys.exit(main())
