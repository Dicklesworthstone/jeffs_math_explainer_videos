"""Run every desk check of this review and save the outputs. Usage (repo root): python3 -I -B <this file>

Builds candidate-ABC, writes the correction patch with diff, applies each patch to a scratch copy with `patch`,
compares bytes, runs rustfmt --check on the changed leaf file, and records the canonical hashes.
Nothing here compiles, renders, or edits canonical files. Scratch copies go in a temporary directory.
"""
import difflib
import hashlib
import pathlib
import shutil
import subprocess
import tempfile

O = pathlib.Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
R = O / "algebra-picture-review-13_arithmetic"
C = O / "algebra13-patch-cross-review01"
CRATE = "explainer_corpus/abstract_algebra_expanded"
RS = f"{CRATE}/src/arithmetic.rs"
PY = ["/usr/bin/python3", "-I", "-B"]
OUT = C / "checks/outputs"
OUT.mkdir(parents=True, exist_ok=True)


def run(args):
    done = subprocess.run([str(a) for a in args], capture_output=True, text=True)
    return done.returncode, done.stdout + done.stderr


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


code, text = run(PY + [C / "checks/build_correction.py"])
(OUT / "build_correction.out").write_text(text, encoding="utf-8")
assert code == 0, text

old = (R / "candidate-AB" / RS).read_text(encoding="utf-8").splitlines(keepends=True)
new = (C / "candidate-ABC" / RS).read_text(encoding="utf-8").splitlines(keepends=True)
patch = "".join(difflib.unified_diff(old, new, f"a/{RS}", f"b/{RS}", n=3))
(C / "patches").mkdir(exist_ok=True)
(C / "patches/C-cross-review-corrections.patch").write_text(patch, encoding="utf-8")

for name, path in (("baseline", R / "baseline" / RS), ("A", R / "candidate-A" / RS), ("AB", R / "candidate-AB" / RS), ("ABC", C / "candidate-ABC" / RS)):
    code, text = run(PY + [C / "checks/replay.py", path, name])
    (OUT / f"replay-{name}.out").write_text(text, encoding="utf-8")
    assert code == 0, text
for name, root in (("AB", R / "candidate-AB"), ("ABC", C / "candidate-ABC")):
    code, text = run(PY + [C / "checks/group_and_boxes.py", root, R / "baseline"])
    (OUT / f"group_and_boxes-{name}.out").write_text(text, encoding="utf-8")

rows = []
with tempfile.TemporaryDirectory() as tmp:
    work = pathlib.Path(tmp)
    for rel in (RS, f"{CRATE}/oracle.py"):
        (work / rel).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(R / "baseline" / rel, work / rel)
    for label, p, want in (
        ("A on baseline", R / "patches/A-wrong-or-early.patch", {RS: R / "candidate-A" / RS}),
        ("B on A", R / "patches/B-teaching-and-sync.patch", {RS: R / "candidate-AB" / RS, f"{CRATE}/oracle.py": R / "candidate-AB" / CRATE / "oracle.py"}),
        ("C on AB", C / "patches/C-cross-review-corrections.patch", {RS: C / "candidate-ABC" / RS, f"{CRATE}/oracle.py": C / "candidate-ABC" / CRATE / "oracle.py"}),
    ):
        done = subprocess.run(["patch", "-p1", "-s", "-i", str(p.resolve())], cwd=work, capture_output=True, text=True)
        same = all(sha(work / rel) == sha(path) for rel, path in want.items())
        rows.append(f"{label}: patch exit {done.returncode}, bytes equal to candidate: {same} {done.stdout}{done.stderr}".strip())
    for rel, path in (("baseline arithmetic.rs", R / "baseline" / RS), ("baseline oracle.py", R / "baseline" / CRATE / "oracle.py")):
        rows.append(f"{rel} {sha(path)[:16]}")
(OUT / "patch_check.out").write_text("\n".join(rows) + "\n", encoding="utf-8")

rows = []
for label, path in (("candidate-A", R / "candidate-A" / RS), ("candidate-AB", R / "candidate-AB" / RS), ("candidate-ABC", C / "candidate-ABC" / RS)):
    code, text = run(["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", path])
    rows.append(f"{label}: rustfmt --check exit {code}")
    if text:
        rows.append(text)
for rel in (RS, f"{CRATE}/oracle.py", f"{CRATE}/script.tsv", f"{CRATE}/src/main.rs"):
    rows.append(f"canonical {sha(rel)}  {rel}")
(OUT / "rustfmt_and_canonical.out").write_text("\n".join(rows) + "\n", encoding="utf-8")

added = sum(1 for line in patch.splitlines() if line.startswith("+") and not line.startswith("+++"))
removed = sum(1 for line in patch.splitlines() if line.startswith("-") and not line.startswith("---"))
print(f"patch C: +{added} -{removed}")
for name in sorted(p.name for p in OUT.iterdir()):
    print(name)
