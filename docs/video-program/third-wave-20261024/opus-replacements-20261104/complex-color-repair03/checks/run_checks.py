"""Run every desk check in this bundle and save its output beside it. No Cargo build, no link of the engine, no render.

    python3 -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-color-repair03/checks/run_checks.py SCRATCH_DIR [NAME ...]

From the repo root. SCRATCH_DIR must be outside the bundle and is only added to. Plain python3 -B, because the picture
checks need numpy and PIL. With NAME, runs only those checks. Saved outputs have SCRATCH_DIR's path replaced, so they
carry no home path.
"""
import pathlib
import subprocess
import sys
import time

CHECKS = pathlib.Path(__file__).resolve().parent
BUNDLE = CHECKS.parent
scratch = pathlib.Path(sys.argv[1]).resolve()
only = set(sys.argv[2:])
scratch.mkdir(parents=True, exist_ok=True)
stamp = time.strftime("%H%M%S")
tests_bin = scratch / f"plate_layout_tests-{stamp}"
LAYOUT = BUNDLE / "candidate/explainer_corpus/complex_analysis_visual/src/plate_layout.rs"
PY = [sys.executable, "-B"]
RUNS = [
    ("seam_cause", [PY + [str(CHECKS / "seam_cause.py")]]),
    ("seam_predict", [PY + [str(CHECKS / "seam_predict.py")]]),
    ("stair_model", [PY + [str(CHECKS / "stair_model.py")]]),
    ("measure_lab_selftest", [PY + [str(CHECKS / "measure_lab.py"), "--self-test", str(scratch / f"selftest-{stamp}")]]),
    ("plate_layout_tests", [["rustc", "--edition", "2024", "--test", str(LAYOUT), "-o", str(tests_bin)], [str(tests_bin)]]),
    ("pure_helpers", [PY + ["-I", str(CHECKS / "pure_helpers.py"), str(scratch / f"pure-{stamp}")]]),
    ("candidate_typecheck", [PY + ["-I", str(CHECKS / "candidate_typecheck.py"), str(scratch / "tc")]]),
    ("patch_roundtrip", [PY + ["-I", str(CHECKS / "patch_roundtrip.py"), str(scratch / "rt")]]),
    ("bindings", [PY + ["-I", str(CHECKS / "bindings.py")]]),
]
failed = []
for name, commands in RUNS:
    if only and name not in only:
        continue
    text = []
    code = 0
    for command in commands:
        r = subprocess.run(command, capture_output=True, text=True)
        text.append(r.stdout + r.stderr)
        code = code or r.returncode
    body = "".join(text).replace(str(scratch), "<scratch>").replace(str(pathlib.Path.home()), "<home>")
    (CHECKS / f"{name}.out").write_text(body)
    print(f"{'ok  ' if code == 0 else 'FAIL'} {name} (exit {code})")
    if code:
        failed.append(name)
print("ALL OK" if not failed else f"FAILED: {failed}")
sys.exit(1 if failed else 0)
