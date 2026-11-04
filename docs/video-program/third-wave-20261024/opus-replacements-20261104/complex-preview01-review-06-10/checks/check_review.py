#!/usr/bin/env python3
"""Check REVIEW.md against the inputs. Run from the repo root. Reads only."""
import hashlib
import json
import os
import re
import sys

B = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-preview01-review-06-10/"
F = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-opus-rebuild-frames01/"
CH = {"06": "06_one_derivative", "07": "07_one_point", "08": "08_loop_sums", "09": "09_boundary", "10": "10_rigidity"}
text = open(B + "REVIEW.md", encoding="utf-8").read()
bad = 0


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


# frame and time pairs, per chapter section
parts = re.split(r"^## Chapter (\d\d)$", text, flags=re.M)
for num, body in zip(parts[1::2], parts[2::2]):
    times = json.load(open(F + CH[num] + "/sampling.json"))["times"]
    body = body.split("\n## ")[0]
    singles = re.findall(r"\bf(\d+) \((\d+(?:\.\d+)?)\)", body)
    for frame, sec in singles:
        if abs(times[int(frame)] - float(sec)) > 0.0015:
            bad += 1
            print(f"MISMATCH {num} f{frame}: cited {sec}, sampling {times[int(frame)]}")
    ranges = re.findall(r"\bf(\d+) to f(\d+) \((\d+(?:\.\d+)?) to (\d+(?:\.\d+)?)\)", body)
    for f0, f1, s0, s1 in ranges:
        for frame, sec in ((f0, s0), (f1, s1)):
            if abs(times[int(frame)] - float(sec)) > 0.0015:
                bad += 1
                print(f"MISMATCH {num} range f{frame}: cited {sec}, sampling {times[int(frame)]}")
    named = sorted({int(x) for x in re.findall(r"\bf(\d+)\b", body)})
    assert named[-1] < len(times), (num, named[-1])
    print(f"{num}: {len(singles)} frame/time pairs, {len(ranges)} ranges, {len(named)} distinct frames named")

# severity counts in the summary table against the finding tables
rows = re.findall(r"^\| (S\d|\d\d-[A-Z]) \| (High|Medium|Low) \|", text, flags=re.M)
count = {"High": 0, "Medium": 0, "Low": 0}
for _, sev in rows:
    count[sev] += 1
# S1 is High, S2 Medium: both are headed sections, not rows
for sev in re.findall(r"^### S\d\. (High|Medium|Low)\.", text, flags=re.M):
    count[sev] += 1
for sev, n in count.items():
    m = re.search(rf"^\| {sev} \| (\d+) \|", text, flags=re.M)
    state = "ok" if m and int(m.group(1)) == n else "BAD"
    if state == "BAD":
        bad += 1
    print(f"{sev}: table says {m.group(1) if m else '?'}, findings {n} {state}")

# hash lists
for listing in ("checks/sheet-hashes.sha256", "checks/input-hashes.sha256"):
    n = 0
    for line in open(B + listing, encoding="utf-8"):
        want, path = line.rstrip("\n").split("  ", 1)
        n += 1
        if sha(path) != want:
            bad += 1
            print("HASH DIFFERS", path)
    print(listing, n, "files")

# denominators
total = 0
for key in CH.values():
    names = [n for n in os.listdir(F + key) if n.startswith("sheet-")]
    times = json.load(open(F + key + "/sampling.json"))["times"]
    assert len(names) == (len(times) + 3) // 4, key
    total += len(names)
print("sheets on disk", total)
if total != 441:
    bad += 1

if "/Users/" in text or "/home/" in text:
    bad += 1
    print("absolute home path in the report")
print("problems:", bad)
sys.exit(1 if bad else 0)
