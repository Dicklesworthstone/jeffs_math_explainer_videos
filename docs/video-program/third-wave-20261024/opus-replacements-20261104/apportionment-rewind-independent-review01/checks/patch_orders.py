"""Apply the two patches to root's candidate ae70d701 in both orders, on scratch copies only, and hash.

    python3 -I -B patch_orders.py <scratch dir>     (from the repo root)

Also: rustfmt --check on root's staged file (check mode writes nothing), and the root stage's diff against
the candidate must be exactly what the two patches change.
"""
import hashlib
import pathlib
import shutil
import subprocess
import sys
import tempfile

SCRATCH = pathlib.Path(sys.argv[1])
REPO = pathlib.Path.cwd()
OPUS = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104"
REL = "explainer_corpus/apportionment/src/chapters/both_promises.rs"
CANDIDATE = OPUS / "root-apportionment-proof-repair02/candidate" / REL
PATCHES = OPUS / "apportionment-proof-repair02-source-review"
STAGED = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-apportionment-rebuild-20261104/combined-repair-stage03" / REL


def sha(p):
    return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()


print("candidate", sha(CANDIDATE)[:16], "| staged", sha(STAGED)[:16])
for name in ["n3-rewind.patch", "grey-key.patch"]:
    print(" ", name, sha(PATCHES / name)[:16])
for order in (["n3-rewind", "grey-key"], ["grey-key", "n3-rewind"], ["n3-rewind"], ["grey-key"]):
    work = pathlib.Path(tempfile.mkdtemp(prefix="ap", dir=SCRATCH))
    dest = work / REL
    dest.parent.mkdir(parents=True)
    shutil.copy2(CANDIDATE, dest)
    notes = []
    for p in order:
        r = subprocess.run(["patch", "-p1", "--no-backup-if-mismatch", "-i", str(PATCHES / f"{p}.patch")], cwd=work, capture_output=True, text=True)
        notes.append((p, r.returncode, [l for l in r.stdout.splitlines() if "FAILED" in l or "fuzz" in l]))
    print(" -> ".join(order), notes, "result", sha(dest)[:16], "| equals staged:", dest.read_bytes() == STAGED.read_bytes())
    f = subprocess.run(["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", str(dest)], capture_output=True, text=True)
    print("   rustfmt --check exit", f.returncode, "| parse errors:", [l for l in f.stderr.splitlines() if l.startswith("error")])
d = subprocess.run(["diff", str(CANDIDATE), str(STAGED)], capture_output=True, text=True)
added = sum(1 for l in d.stdout.splitlines() if l.startswith(">"))
removed = sum(1 for l in d.stdout.splitlines() if l.startswith("<"))
print(f"candidate to staged: +{added} -{removed} lines")
