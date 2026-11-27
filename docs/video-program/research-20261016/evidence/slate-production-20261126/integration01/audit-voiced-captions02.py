"""Export and verify captions for the completed 33-cue flagship preview."""

import argparse
import json
import os
from pathlib import Path
import re
import runpy
import shutil
import subprocess
import sys


HERE = Path(__file__).resolve().parent
COMPLETE = HERE / "prepare-complete-voice01.py"
REPORT = HERE / "native-caption-report01.py"
voice = runpy.run_path(str(COMPLETE))
WORK, PRIOR, digest = (voice[k] for k in ("WORK", "PRIOR", "digest"))
REPO = voice["PREVIEW"]["REPO"]
NATIVE = HERE / "probability-voice01" / "script.tsv"
BINARY = REPO / "target/debug/probability_native"
JOB = "anytime-valid-voiced-captions02"


def frozen():
    speech, pending, wavs, lines, paths = voice["retained_opening"]()
    native = dict(line.split("\t", 1) for line in NATIVE.read_text().splitlines()
                  if line and not line.startswith("#"))
    assert list(native.items()) == list(speech.items())
    chapters = sorted((PRIOR / "clock60").glob("*.timeline.tsv"))
    selected = [p.name.removesuffix(".timeline.tsv") for p in chapters]
    actual, total = [], 0
    for chapter in chapters:
        rows = [line.split("\t") for line in chapter.read_text().splitlines() if not line.startswith("#")]
        assert next(row[1] for row in rows if row[0] == "fps") == "60"
        end = int(next(row[1] for row in rows if row[0] == "frames"))
        total += end
        last_end = 0
        for _, cue, frame, gain, path in (row for row in rows if row[0] == "line"):
            assert Path(path) == wavs[cue] and float(gain) == -3.0
            start = int(frame) * 800
            stop = start + sum(r["out_samples"] for r in lines[cue])
            assert last_end <= start < stop <= end * 800
            last_end = stop
            actual.append(cue)
    assert actual == list(wavs) and len(selected) == 11 and total == 47891 and len(pending) == 8
    failure_receipt = WORK / "anytime-valid-voiced-captions01" / "resource-receipt.json"
    voice["checked_receipt"](failure_receipt, 1)
    paths.update((Path(__file__).resolve(), COMPLETE, REPORT, failure_receipt, NATIVE, voice["ASR"]["SCRIPT"], BINARY, *chapters))
    return sorted(paths), chapters, selected, speech, lines, total


def millis(raw):
    hours, minutes, seconds, milliseconds = map(int, re.split(r"[:,.]", raw))
    return ((hours * 60 + minutes) * 60 + seconds) * 1000 + milliseconds


def captions(path, vtt):
    blocks = re.split(r"\n\s*\n", path.read_text().strip())
    if vtt:
        assert blocks.pop(0) == "WEBVTT"
    result = []
    for index, block in enumerate(blocks, 1):
        body = block.splitlines()
        if not vtt or body[0].isdigit():
            assert int(body.pop(0)) == index
        a, b = body.pop(0).split(" --> ")
        assert 1 <= len(body) <= 2 and all(row.strip() for row in body)
        result.append((millis(a), millis(b), tuple(body)))
    return result


def execute():
    _, chapters, selected, speech, lines, frames = frozen()
    out = WORK / JOB / "captions"
    out.mkdir()
    report_dir = out / "native-report"
    report_dir.mkdir()
    source = PRIOR / "narration" / "report.json"
    adapted = runpy.run_path(str(REPORT))["compatible_report"](voice["read"](source))
    assert set(adapted["lines"]) == set(lines)
    voice["write_json"](report_dir / "report.json", adapted)
    for original in chapters:
        target = out / original.name
        shutil.copyfile(original, target)
        assert digest(target) == digest(original)
    native_script = subprocess.run([BINARY, "script"], check=True, capture_output=True, text=True)
    assert [r.split("\t", 1) for r in native_script.stdout.splitlines()] == [list(pair) for pair in speech.items()]
    subprocess.run([
        BINARY, "captions", *selected, "--fps", "60",
        "--narration", report_dir, "--out", out,
    ], check=True)
    srt, vtt = out / "probability_native.en.srt", out / "probability_native.en.vtt"
    english = captions(srt, False)
    assert english and english == captions(vtt, True)
    expected = " ".join(" ".join(row["sentence"] for row in group) for group in lines.values())
    actual = " ".join(" ".join(block) for _, _, block in english)
    assert actual.split() == expected.split()
    previous = 0
    speeds = []
    for start, end, text in english:
        assert previous <= start < end <= round(1000 * frames / 60)
        speeds.append(len(" ".join(text)) * 1000 / (end - start))
        previous = end
    summary = {
        "scope": "Provisional English captions for a partial voiced preview, on the 60 fps clock.",
        "cues": list(lines),
        "chapters": selected,
        "frames": frames,
        "sentences": sum(map(len, lines.values())),
        "blocks": len(english),
        "srt_sha256": digest(srt),
        "vtt_sha256": digest(vtt),
        "source_report_sha256": digest(source),
        "native_report_sha256": digest(report_dir / "report.json"),
        "characters_per_second_max": max(speeds),
        "blocks_above_20_cps": sum(speed > 20.0 for speed in speeds),
        "listening_acceptance": False,
    }
    voice["write_json"](out / "caption-readback01.json", summary)
    print(json.dumps(summary, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("inspect", "launch", "execute"))
    args = parser.parse_args()
    paths, chapters, _, _, lines, frames = frozen()
    if args.mode == "inspect":
        print(f"{len(chapters)} clocks, {frames} frames and {len(lines)} bound WAVs.")
    elif args.mode == "execute":
        execute()
    else:
        assert not (WORK / JOB).exists()
        guard = voice["ASR"]["GUARD"]
        command = [sys.executable, "-I", "-B", str(guard), "--phase", "native"]
        for path in paths:
            command.extend(("--input", str(path)))
        command.extend((JOB, "--", sys.executable, "-I", "-B", str(Path(__file__).resolve()), "execute"))
        os.execv(sys.executable, command)


if __name__ == "__main__":
    main()
