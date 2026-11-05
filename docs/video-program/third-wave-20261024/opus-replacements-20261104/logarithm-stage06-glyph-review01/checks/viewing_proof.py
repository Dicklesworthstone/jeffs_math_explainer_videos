"""Did each probe page actually reach the reviewer as an image? Reads this session's own transcript.

    python3 -B .../checks/viewing_proof.py SESSION_ID     (from the repo root; PIL needs site packages)

For every Read of a page in the job, finds its tool result and reports whether it holds an image, then compares that
image's pixels with the page on disk. A result that holds only text (a removed image, an error) counts as NOT seen.
This proves the image was delivered; what was noticed in it is the reviewer's claim, in coverage/page-evidence.tsv.
"""
import base64
import hashlib
import io
import json
import re
import sys
from pathlib import Path

from PIL import Image, ImageChops

PAGES = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage06-glyph01/glyphs/pages")
TRANSCRIPTS = Path.home() / ".claude/projects/-Users-jemanuel-projects-jeffs-math-explainer-videos"
PATTERN = re.compile(r"logarithm-stage06-glyph01/glyphs/pages/frame_(\d{6})\.png$")


def walk(content):
    if isinstance(content, list):
        for item in content:
            yield item


def main():
    path = TRANSCRIPTS / f"{sys.argv[1]}.jsonl"
    pending = {}
    seen = {}
    with path.open(encoding="utf-8") as f:
        for line in f:
            if '"tool_use"' not in line and '"tool_result"' not in line:
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
                        pending[item["id"]] = int(m.group(1))
                elif item.get("type") == "tool_result" and item.get("tool_use_id") in pending:
                    page = pending[item["tool_use_id"]]
                    images = [c for c in walk(item.get("content")) if isinstance(c, dict) and c.get("type") == "image"]
                    seen.setdefault(page, []).append(images)

    wanted = sorted(int(p.stem.split("_")[1]) for p in PAGES.glob("frame_*.png"))
    ok = 0
    for page in wanted:
        results = seen.get(page, [])
        with_image = [r for r in results if r]
        disk = Image.open(PAGES / f"frame_{page:06d}.png").convert("RGB")
        verdict = "NOT SEEN"
        detail = f"{len(results)} transcript entries"
        if with_image:
            src = with_image[-1][0].get("source") or {}
            if src.get("type") == "base64":
                raw = base64.b64decode(src["data"])
                shown = Image.open(io.BytesIO(raw)).convert("RGB")
                same_bytes = hashlib.sha256(raw).hexdigest() == hashlib.sha256((PAGES / f"frame_{page:06d}.png").read_bytes()).hexdigest()
                if shown.size != disk.size:
                    shown = shown.resize(disk.size)
                peak = max(hi for _, hi in ImageChops.difference(shown, disk).getextrema())
                verdict = "SEEN" if same_bytes or peak <= 2 else f"IMAGE DIFFERS (peak {peak})"
                detail += f", {shown.size[0]}x{shown.size[1]}, bytes equal {same_bytes}, pixel peak diff {peak}"
            else:
                verdict = "SEEN (image stored without data)"
                detail += f", source type {src.get('type')}"
        if verdict.startswith("SEEN"):
            ok += 1
        print(f"page {page:02d} {verdict}: {detail}")
    print(f"\nRESULT {ok} of {len(wanted)} pages delivered as images")
    sys.exit(0 if ok == len(wanted) else 1)


if __name__ == "__main__":
    main()
