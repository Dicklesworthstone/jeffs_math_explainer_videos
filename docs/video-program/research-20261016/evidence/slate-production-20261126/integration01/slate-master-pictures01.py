"""Make exact-frame contact sheets from the joined 4K60 slate master."""

import argparse
from fractions import Fraction
from io import BytesIO
from itertools import count
import json
from math import ceil
from pathlib import Path
import re
import runpy
import subprocess

from PIL import Image, ImageDraw


HERE = Path(__file__).resolve().parent
packaging = runpy.run_path(str(HERE / "slate-publication-package01.py"))
WORK, sha, read = (packaging[name] for name in ("WORK", "sha", "read_json"))
TABLES = ("timeline.tsv", "cues.tsv", "layout.tsv")


def chapter_anchors(key, frames, tables):
    timeline = [row.split("\t") for row in tables["timeline.tsv"].read_text().splitlines()]
    headers = [row for row in timeline if row[0] in ("chapter", "fps", "frames")]
    assert len(headers) == 3 and dict(headers) == {"chapter": key, "fps": "60", "frames": str(frames)}
    points = {}

    def add(frame, reason):
        points.setdefault(max(0, min(frames - 1, frame)), []).append(reason)

    add(0, "entry")
    add(frames - 1, "exit")
    arrivals = [(row[1], int(row[2])) for row in timeline if row[0] == "line"]
    assert arrivals and all(0 <= start < frames for _, start in arrivals)
    assert all(a[1] < b[1] for a, b in zip(arrivals, arrivals[1:]))
    durations = [row.split("\t") for row in tables["cues.tsv"].read_text().splitlines()]
    assert [row[0] for row in durations] == [cue for cue, _ in arrivals]
    for (cue, start), (_, seconds, length) in zip(arrivals, durations, strict=True):
        assert abs(Fraction(seconds) * 60 - start) < Fraction(1, 25), cue
        span = Fraction(length) * 60
        assert span > 0 and start + span <= frames + Fraction(1, 25), cue
        add(start + ceil(span / 4), f"{cue} early")
        add(start + ceil(span * Fraction(4, 5)), f"{cue} late")
    for row in timeline:
        if row[0] == "listen":
            start = int(row[1])
            duration = ceil(Fraction(row[2]) * 60 - Fraction(1, 1_000_000))
            label = row[3]
            end = start + duration
            assert label and duration > 0 and 0 <= start < end <= frames, (key, row)
            add(start, f"{label} starts")
            add(start + duration // 2, f"{label} midpoint")
            add(end - 1, f"{label} last hold frame")
            add(end + 60, f"{label} after hold")
        elif row[:2] == ["mark", "predict"]:
            frame = int(row[2])
            assert 0 <= frame < frames
            after = next((start for _, start in arrivals if start > frame), frames)
            add(frame, "prediction start")
            add(frame + (after - frame) // 2, "prediction midpoint")
            add(after - 1, "before next cue")
    for index, finding in enumerate(tables["layout.tsv"].read_text().splitlines(), 1):
        label = finding.split("\t", 1)[0]
        match = re.search(r"@([0-9]+(?:\.[0-9]+)?)s$", label)
        anchor = ceil(Fraction(match.group(1) if match else label) * 60)
        for elapsed in (0, 1, 3, 8):
            add(anchor + elapsed * 60, f"layout {index} +{elapsed}s")
    return sorted(points.items())


def picture(path, frame):
    sec, within = divmod(frame, 60)
    raw = subprocess.check_output([
        "ffmpeg", "-nostdin", "-v", "error", "-xerror", "-threads", "1",
        "-ss", str(sec), "-i", path, "-map", "0:v:0", "-an",
        "-vf", f"select=eq(n\\,{within}),scale=640:360",
        "-frames:v", "1", "-fps_mode", "passthrough", "-threads", "1",
        "-c:v", "png", "-f", "image2pipe", "pipe:1",
    ], timeout=60)
    with Image.open(BytesIO(raw)) as image:
        assert image.size == (640, 360)
        return image.convert("RGB")


def sheets(master, output, key, start, frames, tables):
    points = chapter_anchors(key, frames, tables)
    results = []
    for offset in range(0, len(points), 6):
        batch = points[offset:offset + 6]
        sheet = Image.new("RGB", (1280, 384 * ((len(batch) + 1) // 2)), "#10131a")
        text = ImageDraw.Draw(sheet)
        for index, (local, reasons) in enumerate(batch):
            absolute = start + local
            x, y = (index % 2) * 640, (index // 2) * 384
            sheet.paste(picture(master, absolute), (x, y))
            text.text((x + 8, y + 364), f"{absolute} | {absolute / 60:.3f}s  " + ", ".join(reasons), fill="white")
        path = output / f"{key}-{offset // 6 + 1:02d}.png"
        with path.open("xb") as handle:
            sheet.save(handle, format="PNG")
        results.append({"file": path.name, "sha256": sha(path), "chapter_frames": batch})
    return {
        "start": start, "frames": frames, "samples": len(points), "sheets": results,
        "source_sha256": {suffix: sha(path) for suffix, path in tables.items()},
    }


def rendered_chapter(film):
    if film == "anytime":
        native = runpy.run_path(str(HERE / "flagship-native-delivery01.py"))
        return native["render_job"]
    native = runpy.run_path(str(HERE / "companion-native-delivery01.py"))
    return lambda key: native["stage_job"](native["chapter_job"](film, key))


def build(film):
    job, filename, *_ = packaging["film_inputs"](film)
    master = job / "delivery" / filename
    receipt_path, report_path = job / "resource-receipt.json", master.parent / "delivery01.json"
    receipt, report = packaging["stable_success"](receipt_path), read(report_path)
    frames = report["media"]["frames"]
    assert receipt["phase"] == "native" and report["full_av_decode"] is True
    assert report["remux_clock"]["video_packets"] == frames
    assert report["remux_clock"]["video_packet_payloads_unchanged"] is True
    assert sha(master) == report["sha256"][master.name]
    sums = {"master_sha256": report["sha256"][master.name],
            "master_guard_sha256": sha(receipt_path), "delivery_sha256": sha(report_path)}
    reviewed = read(packaging["COPY_REVIEW"])["publication_copy"]["chapter_labels"][film]
    starts = packaging["verified_starts"](master.parent, report["chapters"], reviewed, frames)
    counts = dict(report["chapters"])

    manifest = WORK / "navier-production-20261120" / f"{job.name}-inputs.json"
    assert sha(manifest) == receipt["inputs_sha256"]
    frozen = {Path(row["path"]): row["sha256"] for row in read(manifest)["files"]}
    chapter_job = rendered_chapter(film)
    sources = {}
    for key, _, _ in starts:
        rendered = WORK / chapter_job(key) / "pictures"
        tables = {suffix: master.parent / f"{key}.{suffix}" for suffix in TABLES}
        for path in tables.values():
            upstream = rendered / path.name
            assert sha(path) == frozen.get(upstream) == sha(upstream), path
        sources[key] = tables

    for attempt in count(1):
        output = job / f"master-pictures{attempt:02d}"
        plan = output / "sample-plan01.json"
        if not output.exists():
            break
        if plan.is_file():
            saved = read(plan)
            assert all(saved[name] == value for name, value in sums.items())
            assert list(saved["chapters"]) == list(reviewed)
            for key, start, _ in starts:
                row = saved["chapters"][key]
                assert row["start"] == start and row["frames"] == counts[key]
                assert row["source_sha256"] == {suffix: sha(path) for suffix, path in sources[key].items()}
                assert all(sha(output / item["file"]) == item["sha256"] for item in row["sheets"])
            return plan
    output.mkdir()
    chapters = {key: sheets(master, output, key, start, counts[key], sources[key]) for key, start, _ in starts}
    with plan.open("x") as handle:
        json.dump({**sums, "chapters": chapters,
                   "scope": "Extracted master pictures. Visual and auditory review are pending."},
                  handle, indent=2)
        handle.write("\n")
    return plan


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("film", choices=tuple(packaging["FILMS"]))
    args = parser.parse_args()
    print(build(args.film))
