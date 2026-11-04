"""Append sheets just opened with the Read tool to coverage/viewed.tsv, with each sheet's hash at this moment.

python3 -I -B viewed.py CHAPTER FIRST LAST   (from the repo root; inclusive sheet indices)

Run only after the Read calls have returned. A sheet logged twice, or whose hash no longer equals the manifest, stops it.
"""
import datetime
import hashlib
import sys
from pathlib import Path

F = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/complex02-05-picture-review01")
FRAMES = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02")


def main():
    chapter, first, last = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
    manifest = {}
    for line in (F / "coverage/sheets.tsv").read_text(encoding="utf-8").splitlines()[1:]:
        cells = line.split("\t")
        manifest[(cells[0], cells[1])] = cells[2]
    log = F / "coverage/viewed.tsv"
    done = set()
    if log.exists():
        for line in log.read_text(encoding="utf-8").splitlines()[1:]:
            cells = line.split("\t")
            done.add((cells[0], cells[1]))
    else:
        log.write_text("chapter\tsheet\tsha256\tviewed_utc\n", encoding="utf-8")
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    rows = []
    for index in range(first, last + 1):
        name = f"sheet-{index:03}.png"
        key = (chapter, name)
        if key in done:
            raise SystemExit(f"{key} already logged")
        digest = hashlib.sha256((FRAMES / chapter / name).read_bytes()).hexdigest()
        if digest != manifest[key]:
            raise SystemExit(f"{key} no longer equals the manifest")
        rows.append(f"{chapter}\t{name}\t{digest}\t{stamp}\n")
    with log.open("a", encoding="utf-8") as stream:
        stream.writelines(rows)
    total = len(done) + len(rows)
    print(f"logged {chapter} {first}-{last}; {total} of 348 sheets viewed")


if __name__ == "__main__":
    main()
