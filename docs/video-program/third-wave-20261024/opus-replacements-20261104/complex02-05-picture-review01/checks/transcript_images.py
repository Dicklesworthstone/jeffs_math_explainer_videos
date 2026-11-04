"""Which Read calls of sheet and frame PNGs have an image recorded in this session's transcript?

python3 -I -B transcript_images.py TRANSCRIPT.jsonl

Read-only. Classifies each tool result as image, removed-media text, or other. Prints counts per chapter, the ranges in
each class, and the set of sheets with at least one image, so a later re-read counts.
"""
import json
import re
import sys
from collections import defaultdict

PAT = re.compile(r"complex-combined-picture-frames02/([0-9a-z_]+)/((?:sheet|frame)-\d+\.png)$")


def kinds(content):
    if isinstance(content, str):
        return {"removed" if "media removed" in content else "text"}
    out = set()
    for block in content or []:
        if not isinstance(block, dict):
            continue
        if block.get("type") == "image":
            out.add("image")
        elif block.get("type") == "text":
            out.add("removed" if "media removed" in block.get("text", "") else "text")
        else:
            out.add(block.get("type", "unknown"))
    return out


calls = {}
results = []
lines = 0
for raw in open(sys.argv[1], encoding="utf-8"):
    lines += 1
    try:
        row = json.loads(raw)
    except ValueError:
        continue
    message = row.get("message")
    if not isinstance(message, dict) or not isinstance(message.get("content"), list):
        continue
    for block in message["content"]:
        if not isinstance(block, dict):
            continue
        if block.get("type") == "tool_use" and block.get("name") == "Read":
            match = PAT.search(str(block.get("input", {}).get("file_path", "")))
            if match:
                calls[block["id"]] = match.groups()
        elif block.get("type") == "tool_result" and block.get("tool_use_id") in calls:
            results.append((calls[block["tool_use_id"]], kinds(block.get("content")), row.get("timestamp", "")))

print("transcript lines", lines, "read calls", len(calls), "results", len(results))
with_image = defaultdict(set)
without = defaultdict(set)
tally = defaultdict(lambda: defaultdict(int))
for (chapter, name), found, _ in results:
    key = "image" if "image" in found else ("removed" if "removed" in found else "other")
    tally[(chapter, name.split("-")[0])][key] += 1
    (with_image if key == "image" else without)[(chapter, name.split("-")[0])].add(int(name.split("-")[1].split(".")[0]))
for key in sorted(tally):
    print(key, dict(tally[key]), "distinct with image", len(with_image[key]))


def ranges(values):
    values = sorted(values)
    out, start = [], None
    for i, v in enumerate(values):
        if start is None:
            start = v
        if i + 1 == len(values) or values[i + 1] != v + 1:
            out.append(f"{start}-{v}" if v != start else f"{v}")
            start = None
    return ", ".join(out)


for key in sorted(tally):
    if key[1] == "sheet":
        print(key[0], "sheets with an image recorded:", ranges(with_image[key]) or "none")
        print(key[0], "sheets never with an image:", ranges(without[key] - with_image[key]) or "none")
