"""Write HASHES.sha256 for this folder, relative to it. Run from the repo root, last."""
import hashlib
import os

B = "docs/video-program/third-wave-20261024/opus-replacements-20261104/exponential-abc-teaching-repair01/"
rows = []
for d, _, files in os.walk(B):
    for f in files:
        p = os.path.join(d, f)
        rel = os.path.relpath(p, B)
        if rel == "HASHES.sha256":
            continue
        rows.append((rel, hashlib.sha256(open(p, "rb").read()).hexdigest()))
with open(B + "HASHES.sha256", "w", encoding="utf-8") as out:
    for rel, h in sorted(rows):
        out.write(f"{h}  {rel}\n")
print(len(rows), "files;", hashlib.sha256(open(B + "HASHES.sha256", "rb").read()).hexdigest())
