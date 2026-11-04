"""Run the crate's own oracle on baseline/ and candidate/ and show what changed.

    python3 -I -B checks/oracle_diff.py

The oracle is the crate's second computation (numpy and scipy, no Rust), run as
its own docstring says, from its own folder. The Rust side, src/facts.rs, is
uncompiled here: only `./x facts-check` compares the two. Exit 1 unless the one
difference is the new fact, in facts.rs's order, with the value facts.rs computes.
"""
import difflib
import pathlib
import re
import subprocess
import sys

HERE = pathlib.Path(__file__).resolve().parent
STAGE = HERE.parent
CRATE = "explainer_corpus/complex_analysis_visual"
out = {}
for side in ("baseline", "candidate"):
    r = subprocess.run([sys.executable, "-I", "-B", "oracle.py"], cwd=STAGE / side / CRATE, capture_output=True, text=True)
    last = r.stderr.strip().splitlines()[-1]
    print(f"{side}: exit {r.returncode}; {last}")
    if r.returncode:
        sys.exit(1)
    out[side] = r.stdout.splitlines()
changes = [l for l in difflib.ndiff(out["baseline"], out["candidate"]) if l[:2] in ("+ ", "- ")]
for l in changes:
    print(" ", l)
ok = changes == ["+ square.circle.in_radius\t0.5"]
rows = dict(l.split("\t", 1) for l in out["candidate"])
print("counters' finals:", rows["identity.circle.turns"], rows["square.circle.turns"], "so the cards read", rows["identity.circle.turns"] + ".00", rows["square.circle.turns"] + ".00")
ok = ok and (rows["identity.circle.turns"], rows["square.circle.turns"]) == ("1", "2")
# the new key directly precedes its neighbour in both files, so the order still matches
facts = (STAGE / "candidate" / CRATE / "src" / "facts.rs").read_text()
keys = re.findall(r'\.add\("([^"]+)"', facts)
names = [l.split("\t", 1)[0] for l in out["candidate"]]
pair = ("square.circle.in_radius", "square.circle.out_radius")
adjacent = keys.index(pair[1]) - keys.index(pair[0]) == 1 and names.index(pair[1]) - names.index(pair[0]) == 1
print("the new key directly precedes its neighbour in facts.rs and in the oracle:", adjacent)
print("PASS" if ok and adjacent else "FAIL")
sys.exit(0 if ok and adjacent else 1)
