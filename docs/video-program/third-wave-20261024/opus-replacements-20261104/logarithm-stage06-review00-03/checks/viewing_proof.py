"""Prove from this session's transcript which pictures came back to me as images, and write coverage.tsv.

Run from the repo root, with the session transcript as the one argument:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage06-review00-03/checks/viewing_proof.py TRANSCRIPT.jsonl

A row means the Read call returned an image. That proves delivery only. What I noticed is my claim, in REVIEW.md.
The transcript is not part of this folder, so the saved .out.txt and coverage.tsv are the record.
"""
import base64
import hashlib
import json
import os
import re
import sys

O = "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage06-review00-03/"
FR = "narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-frames01/"
CH = ["00_ratio_ruler", "01_building_ruler", "02_slide_rule", "03_folded_decade"]
PATH = re.compile(r"logarithm-stage06-frames01/(\d\d_[a-z_]+)/(sheet|frame)-(\d+)\.png$")
CROP = re.compile(r"/(crops06b?)/([0-9A-Za-z_]+)\.png$")

uses, results = {}, {}
for line in open(sys.argv[1], encoding="utf-8"):
    try:
        row = json.loads(line)
    except ValueError:
        continue
    content = row.get("message", {}).get("content") if isinstance(row.get("message"), dict) else None
    if not isinstance(content, list):
        continue
    for item in content:
        if not isinstance(item, dict):
            continue
        if item.get("type") == "tool_use" and item.get("name") == "Read":
            uses[item["id"]] = item.get("input", {}).get("file_path", "")
        if item.get("type") == "tool_result":
            c = item.get("content")
            if isinstance(c, list):
                images = [x for x in c if isinstance(x, dict) and x.get("type") == "image"]
                if images:
                    results[item.get("tool_use_id")] = images[0]

opened, crops, identical = {}, [], {}
for use_id, path in uses.items():
    if use_id not in results:
        continue
    m = PATH.search(path)
    if m:
        key = (m.group(1), m.group(2), int(m.group(3)))
        opened.setdefault(key, 0)
        opened[key] += 1
        data = results[use_id].get("source", {}).get("data", "")
        if data and os.path.exists(FR + f"{m.group(1)}/{m.group(2)}-{int(m.group(3)):0{3 if m.group(2) == 'sheet' else 4}d}.png"):
            disk = open(FR + f"{m.group(1)}/{m.group(2)}-{int(m.group(3)):0{3 if m.group(2) == 'sheet' else 4}d}.png", "rb").read()
            identical[key] = base64.b64decode(data) == disk
    m = CROP.search(path)
    if m:
        crops.append(f"{m.group(1)}/{m.group(2)}")

fails = 0
rows = ["kind\tchapter\titem\tframes\tt_first_s\tt_last_s\tsha256_16\timage_returned"]
for ch in CH:
    folder = FR + ch + "/"
    times = json.load(open(folder + "sampling.json", encoding="utf-8"))["times"]
    sheets = sorted(n for n in os.listdir(folder) if n.startswith("sheet-"))
    seen = 0
    for name in sheets:
        k = int(name[6:9])
        first, last = 4 * k, min(4 * k + 3, len(times) - 1)
        got = (ch, "sheet", k) in opened
        seen += got
        digest = hashlib.sha256(open(folder + name, "rb").read()).hexdigest()[:16]
        rows.append(f"sheet\t{ch}\t{name}\t{first}-{last}\t{times[first]:.3f}\t{times[last]:.3f}\t{digest}\t{'yes' if got else 'NO'}")
    same = sum(1 for key, v in identical.items() if key[0] == ch and key[1] == "sheet" and v)
    ok = seen == len(sheets)
    fails += 0 if ok else 1
    print(f"{'PASS' if ok else 'FAIL'} {ch}: {seen} of {len(sheets)} sheets returned as images, {same} byte-identical to disk; frames 0-{len(times) - 1}")
for key in sorted(k for k in opened if k[1] == "frame"):
    ch, _, n = key
    name = f"frame-{n:04d}.png"
    times = json.load(open(FR + ch + "/sampling.json", encoding="utf-8"))["times"]
    digest = hashlib.sha256(open(FR + ch + "/" + name, "rb").read()).hexdigest()[:16]
    rows.append(f"frame\t{ch}\t{name}\t{n}\t{times[n]:.3f}\t{times[n]:.3f}\t{digest}\tyes")
for name in sorted(set(crops)):
    rows.append(f"crop\t\t{name}\t\t\t\t\tyes")
with open(O + "coverage.tsv", "w", encoding="utf-8") as out:
    out.write("\n".join(rows) + "\n")
sheets_total = sum(1 for r in rows if r.startswith("sheet\t") and r.endswith("yes"))
print(f"sheets returned {sheets_total}; full frames opened {sum(1 for r in rows if r.startswith('frame'))}; enlarged crops opened {len(set(crops))}")
check_total = sheets_total == 255
fails += 0 if check_total else 1
print(("PASS" if check_total else "FAIL") + " all 255 sheets")
print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
