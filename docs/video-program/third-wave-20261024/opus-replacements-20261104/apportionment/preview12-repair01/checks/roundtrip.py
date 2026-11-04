#!/usr/bin/env python3
"""Build and prove the patch. Run from the repository root:

  python3 -I -B <stage>/checks/roundtrip.py <new empty scratch folder> [--write]

--write regenerates preview12-repair01.patch and MANIFEST.json from base and
candidate. Every run then proves, without writing to the canonical crate:
  1. the patch applied to a copy of base gives candidate, byte for byte
  2. the same patch reversed on a copy of candidate gives base
  3. base equals the canonical crate's current bytes
  4. `git apply --check` accepts the patch against the canonical crate
"""

import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path

STAGE = Path(__file__).resolve().parent.parent
REPO = STAGE.parents[5]
BASE = STAGE / "base"
CAND = STAGE / "candidate"
PATCH = STAGE / "preview12-repair01.patch"
MANIFEST = STAGE / "MANIFEST.json"
# read by the candidate but not changed by it: drift here also voids the patch
UNCHANGED = [
    "explainer_corpus/apportionment/script.tsv",
    "explainer_corpus/apportionment/src/chapters/draw.rs",
    "explainer_corpus/apportionment/docs/description.md",
]

scratch = Path(sys.argv[1])
write = "--write" in sys.argv[2:]
scratch.mkdir(parents=True, exist_ok=False)
failures = []


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check(ok, what):
    print(("  ok   " if ok else "  FAIL ") + what)
    if not ok:
        failures.append(what)


def files(root):
    return sorted(p.relative_to(root).as_posix() for p in root.rglob("*") if p.is_file())


names = files(BASE)
check(names == files(CAND), f"base and candidate hold the same {len(names)} files")

if write:
    out = subprocess.run(
        ["git", "diff", "--no-index", "--no-color", "--no-ext-diff", "--no-renames", "--",
         str(BASE.relative_to(REPO)), str(CAND.relative_to(REPO))],
        cwd=REPO, capture_output=True, text=True,
    )
    if out.returncode != 1:
        sys.exit(f"git diff --no-index exited {out.returncode}: {out.stderr}")
    b_rel, c_rel = BASE.relative_to(REPO).as_posix(), CAND.relative_to(REPO).as_posix()
    text = out.stdout.replace(f"a/{b_rel}/", "a/").replace(f"b/{c_rel}/", "b/")
    if b_rel in text or c_rel in text:
        sys.exit("a stage path survived in the patch")
    PATCH.write_text(text)
    print(f"  wrote {PATCH.name}")

# 1. forward, on a copy of base outside any repository
forward = scratch / "forward"
shutil.copytree(BASE, forward)
done = subprocess.run(["git", "apply", str(PATCH.resolve())], cwd=forward, capture_output=True, text=True)
check(done.returncode == 0, f"the patch applies to a copy of base {done.stderr.strip()}")
same = [n for n in names if (forward / n).read_bytes() == (CAND / n).read_bytes()]
check(len(same) == len(names), f"and gives candidate byte for byte ({len(same)} of {len(names)})")

# 2. reverse, on a copy of candidate
reverse = scratch / "reverse"
shutil.copytree(CAND, reverse)
done = subprocess.run(["git", "apply", "-R", str(PATCH.resolve())], cwd=reverse, capture_output=True, text=True)
check(done.returncode == 0, f"reversed, it applies to a copy of candidate {done.stderr.strip()}")
same = [n for n in names if (reverse / n).read_bytes() == (BASE / n).read_bytes()]
check(len(same) == len(names), f"and gives base byte for byte ({len(same)} of {len(names)})")

# 3. base is canonical, now
drift = [n for n in names if sha(REPO / n) != sha(BASE / n)]
check(not drift, f"base equals the canonical crate's current bytes (drift: {drift})")

# 4. against canonical, read-only
done = subprocess.run(["git", "apply", "--check", str(PATCH.resolve())], cwd=REPO, capture_output=True, text=True)
check(done.returncode == 0, f"git apply --check accepts it from the repository root {done.stderr.strip()}")

changed = [n for n in names if sha(BASE / n) != sha(CAND / n)]
if write:
    MANIFEST.write_text(json.dumps({
        "stage": "preview12-repair01",
        "bead": "mx-video-program-program-87do.30",
        "grant": "Agent Mail 1269",
        "patch": {"path": PATCH.name, "sha256": sha(PATCH), "strip": 1, "apply_from": "repository root"},
        "files": [
            {"path": n, "base_sha256": sha(BASE / n), "candidate_sha256": sha(CAND / n), "changed": n in changed}
            for n in names
        ],
        "read_but_unchanged": [{"path": n, "sha256": sha(REPO / n)} for n in UNCHANGED],
        "not_done": "no compile, Clippy, Cargo test, render, listening, fine sync or AV acceptance",
    }, indent=2) + "\n")
    print(f"  wrote {MANIFEST.name}")

print("patch sha256", sha(PATCH))
print("changed files", len(changed), "of", len(names))
for n in UNCHANGED:
    print("unchanged", sha(REPO / n)[:16], n)
print()
if failures:
    print(f"FAILED {len(failures)}")
    sys.exit(1)
print("PASS")
