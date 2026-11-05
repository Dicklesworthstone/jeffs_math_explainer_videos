#!/usr/bin/env python3
"""Bind the logarithm final-verify02 sheets 226-339 to the master, and map each sheet to frames, cues and captions.

Reads only. Prints a binding report to stdout and writes the sheet map as TSV to the path given as the only argument.
It decides nothing about pictures: viewing is recorded by hand in VIEWED.tsv, which it never touches.
"""
import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
VERIFY = ROOT / "logarithm-stage09-final-verify02"
WORK = ROOT / "logarithm-stage09-render-work02"
FIRST, LAST = 226, 339
PER_SHEET = 4


def sha(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def stamp(seconds):
    return f"{int(seconds // 60):02d}:{seconds % 60:06.3f}"


def captions(path):
    rows = []
    for block in path.read_text().strip().split("\n\n"):
        lines = block.splitlines()
        match = re.match(r"(\d+):(\d+):(\d+),(\d+) --> (\d+):(\d+):(\d+),(\d+)", lines[1])
        h, m, s, ms, h2, m2, s2, ms2 = map(int, match.groups())
        rows.append((h * 3600 + m * 60 + s + ms / 1000, h2 * 3600 + m2 * 60 + s2 + ms2 / 1000, " ".join(lines[2:])))
    return rows


def main():
    out = Path(sys.argv[1])
    data = json.loads((VERIFY / "verification.json").read_text())
    master = WORK / "logarithm_mysteries_master.mp4"
    srt = WORK / "logarithm_mysteries.en.srt"
    vtt = WORK / "logarithm_mysteries.en.vtt"
    summary = json.loads((ROOT / "logarithm-stage09-master02/master-summary.json").read_text())

    failures = 0

    def report(name, passed, detail):
        nonlocal failures
        failures += not passed
        print(f"{'PASS' if passed else 'FAIL'} {name}: {detail}")

    master_hash = sha(master)
    report("master", master_hash == data["sha256"] == summary["master"]["sha256"], master_hash)
    report("srt", sha(srt) == data["srt_sha256"], data["srt_sha256"])
    report("vtt", sha(vtt) == data["vtt_sha256"], data["vtt_sha256"])
    print(f"     verification.json sha256 {sha(VERIFY / 'verification.json')}")
    print(f"     strict check {data['strict_video_check']}; {data['media']['video']}; frames {data['frames']}; "
          f"pictures_viewed {data['pictures_viewed']}")

    report("counts", len(data["sheets"]) == 339 == len(data["sheet_sha256"]) and len(data["pictures"]) == 1356,
           f"{len(data['sheets'])} sheets, {len(data['pictures'])} pictures, {PER_SHEET} per sheet")

    sheet_bad, frame_bad, rows = [], [], []
    caps = captions(srt)
    place = sorted(data["placements"], key=lambda p: p["start_frame"])
    chapters = data["chapters"]
    for number in range(FIRST, LAST + 1):
        name = f"sheet-{number:03d}.jpg"
        listed = Path(data["sheets"][number - 1]).name
        if listed != name or sha(VERIFY / name) != data["sheet_sha256"][number - 1]:
            sheet_bad.append(name)
        for slot, pic in enumerate(data["pictures"][(number - 1) * PER_SHEET: number * PER_SHEET], 1):
            frame_path = VERIFY / Path(pic["path"]).name
            if sha(frame_path) != pic["sha256"]:
                frame_bad.append(frame_path.name)
            frame = pic["frame"]
            chapter = next(c["chapter"] for c in chapters if c["start_frame"] <= frame < c["start_frame"] + c["frames"])
            cue = ""
            for item in place:
                if item["start_frame"] <= frame:
                    cue = item["cue"]
            seconds = frame / data["fps"]
            words = next((c[2] for c in caps if c[0] <= seconds < c[1]), "(no caption)")
            rows.append([number, slot, Path(pic["path"]).name, frame, stamp(seconds), chapter, cue,
                         "; ".join(pic["reasons"]), words])
    report("sheets 226-339", not sheet_bad, f"{LAST - FIRST + 1} hashed against verification.json; bad {sheet_bad}")
    report("frames of those sheets", not frame_bad, f"{len(rows)} PNG hashed against verification.json; bad {frame_bad}")

    lines = ["sheet\tslot\tpng\tframe\tmm:ss\tchapter\tlast_cue_started\treasons\tcaption"]
    lines += ["\t".join(str(x) for x in row) for row in rows]
    out.write_text("\n".join(lines) + "\n")
    print(f"     first {rows[0][4]} {rows[0][5]}; last {rows[-1][4]} {rows[-1][5]}; sheet map rows {len(rows)}")
    print(f"{'BIND PASS' if not failures else 'BIND FAIL'}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
