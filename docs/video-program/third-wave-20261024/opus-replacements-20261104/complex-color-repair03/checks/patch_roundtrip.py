"""Do this bundle's patches apply exactly, come back off exactly, and give the candidate's bytes? Touches only SCRATCH_DIR.

    python3 -I -B .../complex-color-repair03/checks/patch_roundtrip.py SCRATCH_DIR

From the repo root. Copies the crate's changed files into SCRATCH_DIR/tree, then runs patch -p1 --fuzz=0 forward and in
reverse there, comparing bytes at each stage. git apply --check runs against the real working tree, which it reads and
does not change. Each select patch is applied on top of A and taken off again. It also lists the bound source hashes
and whether they still hold: root edits canonical between syncs, and this bundle's patches are against these bytes.
"""
import hashlib
import pathlib
import shutil
import subprocess
import sys

ROOT = pathlib.Path.cwd()
BUNDLE = pathlib.Path(__file__).resolve().parent.parent
PATCHES = BUNDLE / "patches"
CANDIDATE = BUNDLE / "candidate"
CRATE = "explainer_corpus/complex_analysis_visual"
FILES = ["src/chapters/draw.rs", "src/main.rs", "src/plate_layout.rs", "examples/plate_lab.rs"]
BOUND = {
    f"{CRATE}/src/chapters/draw.rs": "4eba4bb18b989ac7075519031913ad4974e471a495a56a80fde4f796d9159545",
    f"{CRATE}/src/main.rs": "dcc3e6281e42f7bbc6f0b068122d9086547a0f75221b1652eebe3278946327c1",
    f"{CRATE}/script.tsv": "2968b0fd76540cbcd4f50290925e8acfec1aa995bea76a04934dac12f959929e",
    "crates/explainer_kit/src/props.rs": "73606e8921ec18c8492ec3a7355a755e269acabd174cafed1fabfca14379d436",
}
tree = pathlib.Path(sys.argv[1]).resolve() / "tree"
fails = 0


def ck(ok, label):
    global fails
    print(("PASS " if ok else "FAIL ") + label)
    fails += 0 if ok else 1


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def patch(name, *flags):
    r = subprocess.run(["patch", "-p1", "--fuzz=0", "--no-backup-if-mismatch", *flags, "-i", str(PATCHES / name)], cwd=tree, capture_output=True, text=True)
    if r.returncode:
        print(r.stdout, r.stderr)
    return r.returncode == 0


def state():
    return {f: (tree / CRATE / f).read_bytes() if (tree / CRATE / f).exists() else None for f in FILES}


for path, want in BOUND.items():
    ck(sha(path) == want, f"bound source unchanged: {path} {want[:12]}")

if tree.exists():
    tree.rename(tree.with_name(f"tree.{next(i for i in range(10000) if not tree.with_name(f'tree.{i}').exists())}"))
(tree / CRATE / "src/chapters").mkdir(parents=True)
(tree / CRATE / "examples").mkdir()
for f in FILES:
    src = ROOT / CRATE / f
    if src.exists():
        shutil.copy2(src, tree / CRATE / f)
before = state()

ck(patch("A-plate-core.patch"), "A applies, fuzz 0")
ck(patch("B-plate-lab.patch"), "B applies, fuzz 0")
after = state()
for f in FILES:
    ck(after[f] == (CANDIDATE / CRATE / f).read_bytes(), f"gives the candidate's bytes: {f}")
for name, rung in [("select-tile.patch", "PLATE_TILE"), ("select-blend.patch", "PLATE_BLEND"), ("select-rows2.patch", "PLATE_ROWS2"), ("select-cells2.patch", "PLATE_CELLS2")]:
    ck(patch(name), f"{name} applies on A, fuzz 0")
    text = (tree / CRATE / "src/chapters/draw.rs").read_text()
    ck(f"pub const PLATE: PlateStyle = {rung};" in text, f"{name} chooses {rung}")
    ck(patch(name, "-R"), f"{name} reverses, fuzz 0")
    ck(state() == after, f"{name} leaves A's bytes")
ck(patch("B-plate-lab.patch", "-R"), "B reverses, fuzz 0")
ck(patch("A-plate-core.patch", "-R"), "A reverses, fuzz 0")
back = state()
for f in FILES:
    ck(back[f] == before[f] or (before[f] is None and back[f] in (None, b"")), f"back to the source's bytes: {f}")

for name in ("A-plate-core.patch", "B-plate-lab.patch"):
    r = subprocess.run(["git", "apply", "--check", str(PATCHES / name)], cwd=ROOT, capture_output=True, text=True)
    ck(r.returncode == 0, f"git apply --check {name} against the working tree")
    if r.returncode:
        print(r.stderr)
print("PATCH ROUND TRIP:", "OK" if fails == 0 else f"{fails} FAIL")
sys.exit(1 if fails else 0)
