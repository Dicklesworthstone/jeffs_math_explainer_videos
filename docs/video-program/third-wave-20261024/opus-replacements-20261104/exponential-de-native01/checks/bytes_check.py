"""Bytes: root's stage still matches what this bundle was built on, the frozen bundles stand, the patch replays exactly.

Usage from the repo root:  python3 -I -B bytes_check.py SCRATCH_DIR
Reads only; replays the patch in a temporary copy of the stage. SCRATCH_DIR/integrated comes from make_integration.py.
"""
import filecmp
import hashlib
import os
import re
import shutil
import subprocess
import sys
import tempfile

S = os.path.abspath(sys.argv[1])
R = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
D = R + "exponential-de-native01/"
STAGE = "narration_lab/third-wave-20261024/root/elevenlabs-exponential-20261101/abc-teaching-stage01/native-crate/"
fails = 0


def sha(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest()


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


print("--- root's stage, the bytes this patch was built on")
BUILT_ON = {
    "src/phrase.rs": "7ab27a09372cc42e",
    "src/chapters/factorials.rs": "ca48d460151a418a",
    "src/chapters/product_grid.rs": "d06c45e6a7d9387d",
    "src/chapters/rate_law.rs": "0d472d7098fb1534",
    "src/series_model.rs": "f5115801e279a5c6",
    "src/product_model.rs": "b978fcd3ff5eafe3",
    "src/rate_model.rs": "8d896cf9d23d6a6f",
    "oracle.py": "650cb5b3a06efa09",
    "script.tsv": "adf9543c50555dbc",
}
for rel, want in BUILT_ON.items():
    ck(sha(STAGE + rel)[:16] == want, f"stage {rel}", want)
for rel in ("src/main.rs", "src/facts.rs", "src/chapters/mod.rs", "Cargo.toml"):
    print(f"  stage {rel}: {sha(STAGE + rel)[:16]}")

print("--- the patch, forward and reverse, fuzz 0, no offset")
patch = os.path.abspath(D + "patches/integration.patch")
text = open(patch, encoding="utf-8").read()
touched = sorted(set(re.findall(r"^\+\+\+ b/(\S+)", text, re.M)))
print("  touches", touched)
ck(
    touched
    == sorted(
        [
            "oracle.py",
            "script.tsv",
            "src/chapters/mod.rs",
            "src/chapters/only_solution.rs",
            "src/chapters/tiny_multipliers.rs",
            "src/facts.rs",
            "src/limit_model.rs",
            "src/main.rs",
            "src/solution_model.rs",
        ]
    ),
    "exactly the nine expected files",
)
ck(not re.search(r"/Users/|/private/tmp|/home/\w", text), "no machine path in the patch")
with tempfile.TemporaryDirectory() as td:
    copy = os.path.join(td, "native-crate")
    shutil.copytree(STAGE, copy, ignore=shutil.ignore_patterns("target", "renders", "logs", ".git"))
    r = subprocess.run(["patch", "-p1", "--fuzz=0", "-i", patch], cwd=copy, capture_output=True, text=True)
    ck(r.returncode == 0 and "offset" not in r.stdout and "fuzz" not in r.stdout, "forward")
    if r.returncode:
        print(r.stdout[-1500:], r.stderr[-500:])
    for rel in touched:
        ck(sha(os.path.join(copy, rel)) == sha(os.path.join(S, "integrated", rel)), f"  forward {rel} == integrated")
    for rel, src in (
        ("src/limit_model.rs", "candidate/limit01/limit_model.rs"),
        ("src/solution_model.rs", "candidate/only01/solution_model.rs"),
        ("src/chapters/tiny_multipliers.rs", "candidate/limit01/tiny_multipliers.rs"),
        ("src/chapters/only_solution.rs", "candidate/only01/only_solution.rs"),
    ):
        ck(sha(os.path.join(copy, rel)) == sha(D + src), f"  {rel} == {src}")
    r = subprocess.run(["patch", "-R", "-p1", "--fuzz=0", "-i", patch], cwd=copy, capture_output=True, text=True)
    ck(r.returncode == 0, "reverse")
    for rel in ("oracle.py", "script.tsv", "src/main.rs", "src/facts.rs", "src/chapters/mod.rs"):
        ck(filecmp.cmp(os.path.join(copy, rel), STAGE + rel, shallow=False), f"  reverse {rel} == stage")
    # patch may leave an emptied file behind; an empty file is what it removes.
    for rel in ("src/limit_model.rs", "src/solution_model.rs", "src/chapters/tiny_multipliers.rs", "src/chapters/only_solution.rs"):
        p = os.path.join(copy, rel)
        ck(not os.path.exists(p) or os.path.getsize(p) == 0, f"  reverse removes {rel}")

with tempfile.TemporaryDirectory() as td:
    copy = os.path.join(td, "native-crate")
    shutil.copytree(STAGE, copy, ignore=shutil.ignore_patterns("target", "renders", "logs", ".git"))
    r = subprocess.run(["git", "apply", "--check", patch], cwd=copy, capture_output=True, text=True)
    ck(r.returncode == 0, "git apply --check from inside a copy of native-crate")
    if r.returncode:
        print(r.stderr[-800:])

print("--- the frozen bundles stand")
for name, want in (
    ("exponential-abc-teaching-repair01/", "0b7717acf0b9408e1fa1c5092226a73a786f416943e5a2025b2a9684dba351eb"),
    ("exponential-abc-cross-review02/", "027075b445ac11a67e2430064d5d3cdcfd51b2821c5f8515c57ad3632090f518"),
):
    f = R + name
    ck(sha(f + "HASHES.sha256") == want, f"{name}HASHES.sha256 is the delivered one", want[:16])
    bad = listed = 0
    for line in open(f + "HASHES.sha256", encoding="utf-8"):
        h, rel = line.rstrip("\n").split("  ", 1)
        listed += 1
        bad += sha(f + rel) != h
    ck(bad == 0, f"every listed file still matches: {listed} listed")

print("--- nothing in this folder names a machine path")
home = re.compile(r"/Users/|/private/tmp|/home/\w")
hits = []
for d, _, fs in os.walk(D):
    for f in fs:
        p = os.path.join(d, f)
        if f.endswith(".out") or f == "HASHES.sha256":
            continue
        if home.search(open(p, encoding="utf-8", errors="replace").read()):
            hits.append(os.path.relpath(p, D))
ck(hits in ([], ["checks/bytes_check.py"]), "no absolute home or scratch path outside this script's own pattern", str(hits))

print("--- HANDOFF.md's bindings, when it exists")
handoff = D + "HANDOFF.md"
if os.path.exists(handoff):
    rows = re.findall(r"^\| `([^`]+)` \| `([0-9a-f]{64})` \|", open(handoff, encoding="utf-8").read(), re.M)
    for rel, h in rows:
        ck(sha(D + rel) == h, f"binding {rel}", h[:12])
    print("  bindings checked:", len(rows))
else:
    print("  not written yet")

print("--- canonical, read only, for root's base check")
canon = "explainer_corpus/exponential_magic/"
for f in ("oracle.py", "src/facts.rs", "src/main.rs", "script.tsv"):
    print("  canonical now", f, sha(canon + f)[:16])
print("RESULT", "PASS" if not fails else f"{fails} FAIL")
raise SystemExit(1 if fails else 0)
