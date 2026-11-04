"""Staged bytes for the exponential A-C review: bindings, sums, patch replay.

Run from the repo root with python3 -I -B. Reads only; replays patches in a temp copy.
"""
import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile

R = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
W = R + "exponential-expanded-review01/"
fails = 0


def sha(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest()


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


# 1. every backticked binding in REVIEW.md
text = open(W + "REVIEW.md", encoding="utf-8").read()
n = 0
for path, h in re.findall(r"^\| `([^`]+)` \| `([0-9a-f]{64})` \|", text, re.M):
    p = (W + path) if path.startswith("patches/") else (R + path)
    ck(sha(p) == h, f"binding {path}", h[:12])
    n += 1
print("bindings checked:", n)

# 2. SHA256SUMS over the review folder
listed = set()
bad = 0
for line in open(W + "SHA256SUMS", encoding="utf-8"):
    h, rel = line.rstrip("\n").split("  ", 1)
    rel = rel[2:] if rel.startswith("./") else rel
    listed.add(rel)
    if sha(W + rel) != h:
        bad += 1
        print("  sum differs:", rel)
ck(bad == 0, f"SHA256SUMS: {len(listed)} listed, {bad} differ")
for d, _, files in os.walk(W):
    for f in files:
        rel = os.path.relpath(os.path.join(d, f), W)
        if rel not in listed and rel != "SHA256SUMS":
            print("UNLISTED", rel)

# 3. rate02 equals the rate01 independent review's candidate
c = R + "exponential-expanded-rate01-independent-review/candidate/"
for f in ("rate_law.rs", "rate_model.rs", "script-slice.tsv"):
    ck(sha(c + f) == sha(R + "exponential-expanded-rate02/" + f), f"rate02/{f} == rate01-review candidate/{f}")

# 4. base equals originals
SETS = (
    ("series01", "exponential-expanded-series01", "series-repair.patch", ("factorials.rs", "series_model.rs", "oracle.py")),
    ("product01", "exponential-expanded-product01", "product-repair.patch", ("product_grid.rs", "product_model.rs", "oracle.py")),
)
for name, orig, _, files in SETS:
    for f in files:
        ck(sha(f"{W}base/{name}/{f}") == sha(f"{R}{orig}/{f}"), f"base/{name}/{f} == {orig}/{f}")

# 5. patch replay from the ORIGINAL folder at fuzz 0, forward and reverse
for name, orig, pf, files in SETS:
    patch_text = open(W + "patches/" + pf, encoding="utf-8").read()
    touched = sorted(set(re.findall(r"^\+\+\+ b/(\S+)", patch_text, re.M)))
    print(f"{pf}: touches {touched}")
    ck(not any("script" in t for t in touched), f"{pf} leaves the script slice alone")
    print(f"  {pf}: READMEs touched: {[t for t in touched if t.endswith('.md')]}")
    with tempfile.TemporaryDirectory() as td:
        for f in files:
            shutil.copy(f"{R}{orig}/{f}", td)
        pp = os.path.abspath(W + "patches/" + pf)
        r = subprocess.run(["patch", "-p1", "--fuzz=0", "-i", pp], cwd=td, capture_output=True, text=True)
        ck(r.returncode == 0 and "offset" not in r.stdout, f"{pf} forward, fuzz 0, no offset", r.stdout.strip().replace("\n", "; "))
        for f in files:
            ck(sha(f"{td}/{f}") == sha(f"{W}candidate/{name}/{f}"), f"forward {f} == candidate")
        r = subprocess.run(["patch", "-R", "-p1", "--fuzz=0", "-i", pp], cwd=td, capture_output=True, text=True)
        ck(r.returncode == 0, f"{pf} reverse, fuzz 0")
        for f in files:
            ck(sha(f"{td}/{f}") == sha(f"{R}{orig}/{f}"), f"reverse {f} == original")

# 6. canonical unchanged from facts01's base
b = json.load(open(R + "exponential-rate-facts01/bindings.json", encoding="utf-8"))
print("bindings.json top-level keys:", list(b)[:12])
canon = "explainer_corpus/exponential_magic/"


def walk(o, trail=""):
    if isinstance(o, dict):
        for k, v in o.items():
            yield (trail + "/" + k, v)
            yield from walk(v, trail + "/" + k)
    elif isinstance(o, list):
        for x in o:
            yield from walk(x, trail)


for k, v in walk(b):
    if isinstance(v, str) and re.fullmatch(r"[0-9a-f]{64}", v):
        print(f"  binding {k} = {v[:16]}")
for f in ("oracle.py", "src/facts.rs", "src/main.rs", "script.tsv"):
    print("canonical now", f, sha(canon + f)[:16])
print("canonical rate_model exists:", os.path.exists(canon + "src/rate_model.rs"))
print("canonical src/:", sorted(os.listdir(canon + "src")))
print("RESULT", "PASS" if not fails else f"{fails} FAIL")
