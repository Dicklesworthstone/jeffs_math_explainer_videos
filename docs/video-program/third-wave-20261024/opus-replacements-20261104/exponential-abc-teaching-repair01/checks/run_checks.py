"""Run every check in this folder and store each output beside its script.

Usage from the repo root:  python3 -I -B <this folder>/run_checks.py SCRATCH_DIR
SCRATCH_DIR receives every generated harness and rmeta. Only the .out files here change. No Cargo, no codegen for
the scenes, no render. scene_typecheck needs the existing target/debug/deps metadata at the pinned fmn.
"""
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
ORDER = (
    ("bytes_check", []),
    ("native_facts", ["native"]),
    ("scene_typecheck", ["tc"]),
    ("scene_controls", ["tc"]),
    ("math_exact", []),
    ("timing_replay", ["timing"]),
    ("source_checks", ["source"]),
    ("text_fit", []),
)
worst = 0
for name, args in ORDER:
    r = subprocess.run(
        ["python3", "-I", "-B", os.path.join(HERE, name + ".py"), *(os.path.join(S, a) for a in args)],
        capture_output=True,
        text=True,
    )
    with open(os.path.join(HERE, name + ".out"), "w", encoding="utf-8") as out:
        out.write(r.stdout + r.stderr)
    last = (r.stdout.strip().splitlines() or [""])[-1]
    print(f"{name}: exit {r.returncode}; {last[:110]}")
    worst = max(worst, r.returncode)
sys.exit(worst)
