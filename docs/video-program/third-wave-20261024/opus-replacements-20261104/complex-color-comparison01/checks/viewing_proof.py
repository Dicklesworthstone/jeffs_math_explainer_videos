"""Did each image in this folder reach the reviewer, and at its exact pixels? Reads this session's own transcript.

    python3 -B .../checks/viewing_proof.py SESSION_ID     (from the repo root)

For every Read of a PNG under sheets/, evidence/ or evidence/lossless/, finds its tool result and compares what arrived with the file.
EXACT means the bytes arrived unchanged. The harness turned the largest sheets into JPEG and shrank the widest images, so a sheet
that did not arrive exact counts only if every part cut from it in evidence/lossless/ did. A result holding only text is NOT SEEN.
This proves delivery. What was noticed is the reviewer's claim, in coverage/evidence.tsv. These are lab frames, not film frames.
"""
import base64
import hashlib
import json
import re
import sys
from pathlib import Path

from PIL import Image

BUNDLE = Path(__file__).resolve().parent.parent
TRANSCRIPTS = Path.home() / ".claude/projects/-Users-jemanuel-projects-jeffs-math-explainer-videos"
PATTERN = re.compile(r"complex-color-comparison01/((?:sheets|evidence|evidence/lossless)/[\w.-]+\.png)$")


def walk(content):
    if isinstance(content, list):
        yield from content


def main():
    pending, seen = {}, {}
    with (TRANSCRIPTS / f"{sys.argv[1]}.jsonl").open(encoding="utf-8") as f:
        for line in f:
            if "complex-color-comparison01" not in line and '"tool_result"' not in line:
                continue
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            for item in walk((entry.get("message") or {}).get("content")):
                if not isinstance(item, dict):
                    continue
                if item.get("type") == "tool_use" and item.get("name") == "Read":
                    m = PATTERN.search(str((item.get("input") or {}).get("file_path", "")))
                    if m:
                        pending[item["id"]] = m.group(1)
                elif item.get("type") == "tool_result" and item.get("tool_use_id") in pending:
                    images = [c for c in walk(item.get("content")) if isinstance(c, dict) and c.get("type") == "image"]
                    seen.setdefault(pending[item["tool_use_id"]], []).append(images)

    wanted = sorted(p.relative_to(BUNDLE).as_posix() for d in ("sheets", "evidence", "evidence/lossless") for p in (BUNDLE / d).glob("*.png"))
    state = {}
    lines = {}
    for rel in wanted:
        results = [r for r in seen.get(rel, []) if r]
        if not results:
            state[rel], lines[rel] = "NOT SEEN", "no image in the transcript"
            continue
        # any delivery of the file's exact bytes counts, so a later re-read cannot hide an earlier one or the reverse
        exact = False
        kinds = set()
        for images in results:
            src = images[0].get("source") or {}
            kinds.add(src.get("media_type", src.get("type")))
            if src.get("type") == "base64" and hashlib.sha256(base64.b64decode(src["data"])).hexdigest() == hashlib.sha256((BUNDLE / rel).read_bytes()).hexdigest():
                exact = True
        state[rel] = "EXACT" if exact else "CHANGED"
        size = Image.open(BUNDLE / rel).size
        lines[rel] = f"{size[0]}x{size[1]}, delivered as {sorted(kinds)}"

    ok = 0
    for rel in wanted:
        verdict = state[rel]
        detail = lines[rel]
        if verdict == "CHANGED" and rel.startswith("evidence/") and "/lossless/" not in rel:
            parts = [w for w in wanted if w.startswith(f"evidence/lossless/{pathlib_stem(rel)}-")]
            if parts and all(state[w] == "EXACT" for w in parts):
                verdict = f"LOSSY, READ EXACT IN {len(parts)} PARTS"
        ok += verdict == "EXACT" or verdict.startswith("LOSSY, READ EXACT")
        print(f"{rel} {verdict}: {detail}")
    print(f"\nRESULT {ok} of {len(wanted)} images read at exact pixels, directly or through their parts")
    sys.exit(0 if ok == len(wanted) else 1)


def pathlib_stem(rel):
    return Path(rel).stem


if __name__ == "__main__":
    main()
