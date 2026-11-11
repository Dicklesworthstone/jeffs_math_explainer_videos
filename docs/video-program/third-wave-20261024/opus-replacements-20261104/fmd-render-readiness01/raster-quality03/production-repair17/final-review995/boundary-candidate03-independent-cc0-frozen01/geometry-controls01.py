"""geometry-controls01.py: mutation controls for geometry-cand03.py.

Each control writes a NEW variant of candidate03 (mode 'x') with one change in the film part
(the text before "\\n#[cfg(test)]\\nmod tests {"), runs geometry-cand03.py on it, and compares
the checks that fail with the prediction written here before the run.

usage: python3 -I -B geometry-controls01.py CANDIDATE03_RS GEOMETRY_PY OUT_DIR PYTHON
exit: 0 when every control fails exactly the predicted checks
"""
import os
import subprocess
import sys

cand, geo, out, py = sys.argv[1:5]
src = open(cand, encoding="utf-8").read()
film, mark, tests = src.partition("\n#[cfg(test)]\nmod tests {")
assert mark

CONTROLS = [
    ("G1-a-ride-no-gap",
     "pub const A_RIDE: f64 = 0.02 + 0.045 + 12.5 / 270.0;", "pub const A_RIDE: f64 = 0.02 + 12.5 / 270.0;",
     {"a and rho clear the circle line by CLEAR", "leader always crosses the circle line"}),
    ("G2-leader-formula-text",
     "    let foot = A_RIDE - 12.5 / 270.0;", "    let foot = A_RIDE - 12.0 / 270.0;",
     {"formula a_leader is the one evaluated here"}),
    ("G3-shrunk-r-0045",
     "pub const SHRUNK_R: f64 = 0.06;", "pub const SHRUNK_R: f64 = 0.045;",
     {"shrunk circle keeps a drawn gap from the hole ring", "leader always crosses the circle line"}),
    ("G4-hidden-from-015",
     "pub const A_NAME_HIDDEN: [f64; 2] = [0.11, 0.43];", "pub const A_NAME_HIDDEN: [f64; 2] = [0.15, 0.43];",
     {"a hidden wherever a growing line is within CLEAR"}),
    ("G6-rho-ride-no-gap",
     "pub const RHO_RIDE: f64 = 0.02 + 0.045 + 15.0 / 270.0;", "pub const RHO_RIDE: f64 = 0.02 + 15.0 / 270.0;",
     {"a and rho clear the circle line by CLEAR"}),
]

base = subprocess.run([py, "-I", "-B", geo, cand], capture_output=True, text=True)
print("candidate03 itself: exit %d, %s" % (base.returncode, base.stdout.strip().splitlines()[-1]))
all_ok = base.returncode == 0
for name, old, new, predicted in CONTROLS:
    assert film.count(old) == 1, name
    path = os.path.join(out, name + "-boundary.rs")
    with open(path, "x", encoding="utf-8", newline="") as f:
        f.write(film.replace(old, new) + mark + tests)
    run = subprocess.run([py, "-I", "-B", geo, path], capture_output=True, text=True)
    failed = {ln[5:].split("  [")[0] for ln in run.stdout.splitlines() if ln.startswith("FAIL ")}
    ok = run.returncode == 1 and failed == predicted
    all_ok &= ok
    print("\n## %s: %s -> %s" % (name, old.strip(), new.strip()))
    print("   exit %d; predicted %s" % (run.returncode, sorted(predicted)))
    print("   failed    %s" % sorted(failed))
    print("   " + ("AS PREDICTED" if ok else "NOT AS PREDICTED"))
    for ln in run.stdout.splitlines():
        if ln.startswith("FAIL ") or "least gap while a shows" in ln or "line within CLEAR" in ln:
            print("     | " + ln)
    if run.stderr.strip():
        print("   stderr: " + run.stderr.strip().splitlines()[-1])
print("\nall controls as predicted:", all_ok)
sys.exit(0 if all_ok else 1)
