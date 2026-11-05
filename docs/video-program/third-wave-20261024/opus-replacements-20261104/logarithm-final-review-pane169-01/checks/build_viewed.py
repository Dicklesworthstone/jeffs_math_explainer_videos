#!/usr/bin/env python3
"""Merge the hand notes with verification.json into VIEWED.tsv. Reads only, apart from its output.

A sheet appears here because a note was written after its image was read. This script
cannot show that a sheet was read: it only fails if a sheet of 226-339 has no note,
has two notes, or no longer matches the hash in verification.json.

Run from the repo root. Takes the output path as its only argument.
"""

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-stage09-final-verify02")
HERE = Path(__file__).resolve().parent.parent
FIRST, LAST = 226, 339
PER_SHEET = 4


def main() -> int:
    data = json.loads((ROOT / "verification.json").read_text())
    notes: dict[int, str] = {}
    for batch in sorted((HERE / "notes").glob("batch*.tsv")):
        for line in batch.read_text().splitlines():
            if not line.strip():
                continue
            number, note = line.split("\t", 1)
            key = int(number)
            if key in notes:
                print(f"FAIL duplicate note for sheet {key}")
                return 1
            notes[key] = note
    wanted = set(range(FIRST, LAST + 1))
    if set(notes) != wanted:
        print(f"FAIL missing {sorted(wanted - set(notes))} extra {sorted(set(notes) - wanted)}")
        return 1
    rows = ["sheet\tsheet_sha256\tframe1\tframe2\tframe3\tframe4\tviewed\tnote"]
    for number in range(FIRST, LAST + 1):
        name = f"sheet-{number:03d}.jpg"
        digest = hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
        if digest != data["sheet_sha256"][number - 1]:
            print(f"FAIL hash {name}")
            return 1
        frames = [str(p["frame"]) for p in data["pictures"][(number - 1) * PER_SHEET : number * PER_SHEET]]
        rows.append("\t".join([name, digest, *frames, "yes", notes[number]]))
    Path(sys.argv[1]).write_text("\n".join(rows) + "\n")
    print(f"PASS {len(rows) - 1} sheets, 226-339, each with one note and a matching hash")
    return 0


if __name__ == "__main__":
    sys.exit(main())
