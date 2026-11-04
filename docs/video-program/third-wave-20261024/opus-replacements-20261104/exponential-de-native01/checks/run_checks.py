"""Run every check in this folder and store each output beside its script.

Usage from the repo root:  python3 -I -B <this folder>/run_checks.py SCRATCH_DIR
make_integration.py runs first, so the patch and the integrated crate are always from the current candidate files.
SCRATCH_DIR receives every generated harness and rmeta. Only the .out files and the patch here change. No Cargo, no
codegen for the scenes, no render. scene_typecheck needs the existing target/debug/deps metadata at the pinned fmn.
"""
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
ORDER = (
    "make_integration",
    "bytes_check",
    "native_facts",
    "scene_typecheck",
    "scene_controls",
    "math_exact",
    "timing_replay",
    "source_checks",
    "text_fit",
)
NEEDS_SCRATCH = {"make_integration", "bytes_check", "native_facts", "scene_typecheck", "scene_controls", "timing_replay", "source_checks"}
worst = 0
for name in ORDER:
    args = [S] if name in NEEDS_SCRATCH else []
    r = subprocess.run(["python3", "-I", "-B", os.path.join(HERE, name + ".py"), *args], capture_output=True, text=True)
    text = (r.stdout + r.stderr).replace(S + "/", "<scratch>/").replace(S, "<scratch>")
    with open(os.path.join(HERE, name + ".out"), "w", encoding="utf-8") as out:
        out.write(text)
    last = (r.stdout.strip().splitlines() or [""])[-1]
    print(f"{name}: exit {r.returncode}; {last[:110]}")
    worst = max(worst, r.returncode)
sys.exit(worst)
