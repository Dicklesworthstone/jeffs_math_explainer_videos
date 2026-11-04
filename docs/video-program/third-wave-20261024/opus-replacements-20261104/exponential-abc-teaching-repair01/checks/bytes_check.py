"""Staged bytes: base equals its sources, patches replay exactly, the frozen review and script originals stand.

Usage from the repo root:  python3 -I -B bytes_check.py
Reads only; replays patches in temporary copies.
"""
import hashlib
import os
import re
import shutil
import subprocess
import tempfile

R = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
D = R + "exponential-abc-teaching-repair01/"
FROZEN = R + "exponential-abc-cross-review02/"
fails = 0


def sha(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest()


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


print("--- base/ equals where it was copied from")
SOURCES = {
    "rate02/rate_law.rs": "exponential-expanded-rate02/rate_law.rs",
    "rate02/script-slice.tsv": "exponential-expanded-rate02/script-slice.tsv",
    "series01/factorials.rs": "exponential-expanded-review01/candidate/series01/factorials.rs",
    "series01/series_model.rs": "exponential-expanded-review01/candidate/series01/series_model.rs",
    "series01/oracle.py": "exponential-expanded-review01/candidate/series01/oracle.py",
    "series01/script-slice.tsv": "exponential-expanded-series01/script-slice.tsv",
    "product01/product_grid.rs": "exponential-expanded-review01/candidate/product01/product_grid.rs",
    "product01/product_model.rs": "exponential-expanded-review01/candidate/product01/product_model.rs",
    "product01/oracle.py": "exponential-expanded-review01/candidate/product01/oracle.py",
    "product01/script-slice.tsv": "exponential-expanded-product01/script-slice.tsv",
}
for mine, source in SOURCES.items():
    ck(sha(D + "base/" + mine) == sha(R + source), f"base/{mine} == {source}", sha(R + source)[:12])

print("--- patches, forward and reverse, fuzz 0, no offset")
for name, folder in (("rate", "rate02"), ("series", "series01"), ("product", "product01")):
    patch = os.path.abspath(f"{D}patches/{name}.patch")
    text = open(patch, encoding="utf-8").read()
    touched = sorted(set(re.findall(r"^\+\+\+ b/(\S+)", text, re.M)))
    print(f"  {name}.patch touches {touched}")
    files = sorted(os.listdir(f"{D}base/{folder}"))
    with tempfile.TemporaryDirectory() as td:
        for f in files:
            shutil.copy(f"{D}base/{folder}/{f}", td)
        r = subprocess.run(["patch", "-p1", "--fuzz=0", "-i", patch], cwd=td, capture_output=True, text=True)
        ck(r.returncode == 0 and "offset" not in r.stdout, f"{name}.patch forward")
        for f in files:
            ck(sha(f"{td}/{f}") == sha(f"{D}candidate/{folder}/{f}"), f"  forward {f} == candidate")
        r = subprocess.run(["patch", "-R", "-p1", "--fuzz=0", "-i", patch], cwd=td, capture_output=True, text=True)
        ck(r.returncode == 0, f"{name}.patch reverse")
        for f in files:
            ck(sha(f"{td}/{f}") == sha(f"{D}base/{folder}/{f}"), f"  reverse {f} == base")

print("--- the frozen review stands")
want = "027075b445ac11a67e2430064d5d3cdcfd51b2821c5f8515c57ad3632090f518"
ck(sha(FROZEN + "HASHES.sha256") == want, "cross-review02/HASHES.sha256 is the delivered one", want[:16])
bad = listed = 0
seen = set()
for line in open(FROZEN + "HASHES.sha256", encoding="utf-8"):
    h, rel = line.rstrip("\n").split("  ", 1)
    listed += 1
    seen.add(rel)
    bad += sha(FROZEN + rel) != h
ck(bad == 0, f"every listed file still matches: {listed} listed")
extra = [
    os.path.relpath(os.path.join(d, f), FROZEN)
    for d, _, fs in os.walk(FROZEN)
    for f in fs
    if os.path.relpath(os.path.join(d, f), FROZEN) not in seen | {"HASHES.sha256"}
]
ck(not extra, "no unlisted file was added to it", str(extra))

print("--- nothing in this folder names a machine path")
home = re.compile(r"/Users/|/private/tmp|/home/\w")
hits = []
for d, _, fs in os.walk(D):
    for f in fs:
        p = os.path.join(d, f)
        if home.search(open(p, encoding="utf-8", errors="replace").read()):
            hits.append(os.path.relpath(p, D))
# This script's own pattern is the one expected hit.
ck(hits in ([], ["checks/bytes_check.py"]), "no absolute home or scratch path", str(hits))

print("--- REPAIR.md's bindings, when it exists")
repair = D + "REPAIR.md"
if os.path.exists(repair):
    text = open(repair, encoding="utf-8").read()
    rows = re.findall(r"^\| `([^`]+)` \| `([0-9a-f]{64})` \|", text, re.M)
    for rel, h in rows:
        ck(sha(D + rel) == h, f"binding {rel}", h[:12])
    print("  bindings checked:", len(rows))
else:
    print("  not written yet")

print("--- canonical, read only, for root's base check")
canon = "explainer_corpus/exponential_magic/"
for f in ("oracle.py", "src/facts.rs", "src/main.rs", "script.tsv"):
    print("  canonical now", f, sha(canon + f)[:16])
print("  canonical chapters:", sorted(os.listdir(canon + "src/chapters")) if os.path.isdir(canon + "src/chapters") else "none")
print("RESULT", "PASS" if not fails else f"{fails} FAIL")
raise SystemExit(1 if fails else 0)
