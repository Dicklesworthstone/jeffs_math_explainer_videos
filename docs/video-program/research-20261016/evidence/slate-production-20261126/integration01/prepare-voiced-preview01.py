"""Bind intact Frederick Surrey takes to the revised script for Chapters 1–11."""

import argparse
import csv
from hashlib import file_digest
import json
from pathlib import Path
import runpy
import shutil
import wave


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[5]
FLAGSHIP = HERE.parents[1] / "applied-probability-20261126" / "flagship"
WORK = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
SCRIPT = HERE / "script02.tsv"
VISUAL = HERE / "probability-voice01" / "script.tsv"
PRIOR = WORK / "anytime-valid-opening-preview-audio01"
RECOGNITION = WORK / "anytime-valid-frederick-middle-asr01"
DECISIONS = HERE / "middle-provisional-selection01.json"
MIDDLE = HERE / "recognize-middle01.py"
CLOCK = FLAGSHIP / "opening-sentence-clock01.py"


def sha(path):
    with path.open("rb") as stream:
        return file_digest(stream, "sha256").hexdigest()


def read(path):
    return json.loads(path.read_text())


def checked_wav(path):
    with wave.open(str(path), "rb") as stream:
        assert (stream.getnchannels(), stream.getsampwidth(), stream.getframerate()) == (1, 2, 48000), path
        count = stream.getnframes()
        assert len(stream.readframes(count)) == count * 2, path
    return count


def check_boundaries(rows, count, text):
    boundary = 0
    for row in rows:
        assert row["source_in_sample"] == boundary
        next_boundary = row["source_out_sample"]
        assert next_boundary > boundary and row["gap_after_samples"] == 0
        assert row["out_samples"] == next_boundary - boundary
        boundary = next_boundary
    assert boundary == count
    assert " ".join(row["sentence"] for row in rows) == text


def preview_inputs():
    with SCRIPT.open(newline="") as stream:
        script = {row["id"]: row["text"] for row in csv.DictReader(stream, delimiter="\t")}
    native = dict(row.split("\t", 1) for row in VISUAL.read_text().splitlines() if row and not row.startswith("#"))
    assert list(script.items()) == list(native.items())
    ready = [cue for cue in script if int(cue.split("_")[0]) <= 11]
    pending = [cue for cue in script if cue not in ready]
    assert len(script) == 41 and len(ready) == 33 and len(pending) == 8
    assert sum(len(script[cue]) for cue in pending) == 2978

    recognized = RECOGNITION / "resource-receipt.json"
    middle = runpy.run_path(str(MIDDLE))
    middle["checked_receipt"](recognized, 0)
    verified_paths, recorded = middle["bindings"](True)
    log = RECOGNITION / "command.log"
    transcripts = [json.loads(record) for record in log.read_text().splitlines()]
    assert len(transcripts) == len(recorded) == 26
    assert {Path(row["input"]) for row in transcripts} == set(recorded)
    assert all(row["status"] == "ok" for row in transcripts)
    observed = {Path(row["input"]): row["report"]["result"] for row in transcripts}
    decision = read(DECISIONS)
    assert decision["script_sha256"] == sha(SCRIPT)
    assert decision["asr_log_sha256"] == sha(log)
    assert not decision["perceptual_acceptance"]
    middle_ids = [cue for cue in ready if cue.startswith(("08_", "09_", "10_", "11_"))] + ["03_1"]
    assert set(decision["takes"]) == set(middle_ids)
    assert len(middle_ids) == 13

    clock = runpy.run_path(str(CLOCK))["rows_for_cue"]
    old = read(PRIOR / "report.json")
    old_receipt = read(PRIOR / "preview-audio-receipt01.json")
    assert old_receipt["report_sha256"] == sha(PRIOR / "report.json")
    paths = {
        Path(__file__).resolve(), SCRIPT, VISUAL, DECISIONS, MIDDLE, CLOCK, recognized, log,
        PRIOR / "report.json", PRIOR / "preview-audio-receipt01.json", *verified_paths,
    }
    candidates = {}
    for index, cue in enumerate(middle_ids):
        pair = recorded[index * 2:index * 2 + 2]
        for item in pair:
            assert item.parent.name in (cue, f"{cue}-recovery02"), item
        chosen = decision["takes"][cue]
        assert chosen in (1, 2)
        take = pair[chosen - 1]
        count = checked_wav(take)
        rows = clock(script[cue], observed[take]["segments"], count)
        for row in rows:
            row.update(source=str(take.relative_to(REPO)), source_sha256=sha(take))
        candidates[cue] = (take, rows, count)
    takes, lines = {}, {}
    for cue in ready:
        if cue in candidates:
            source, rows, count = candidates[cue]
        else:
            source = PRIOR / f"{cue}.wav"
            rows = old["lines"][cue]
            raw = REPO / rows[0]["source"]
            assert sha(source) == old_receipt["wav_sha256"][source.name] == sha(raw) == rows[0]["source_sha256"]
            count = checked_wav(source)
            paths.add(raw)
        check_boundaries(rows, count, script[cue])
        paths.add(source)
        takes[cue] = source
        lines[cue] = rows
    return sorted(paths), takes, lines, pending


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    options = parser.add_mutually_exclusive_group(required=True)
    options.add_argument("--inputs", action="store_true")
    options.add_argument("--output", type=Path)
    args = parser.parse_args()
    paths, takes, lines, pending = preview_inputs()
    if args.inputs:
        print(json.dumps([str(path) for path in paths], indent=2))
        return
    args.output.mkdir()
    for cue, source in takes.items():
        output = args.output / f"{cue}.wav"
        shutil.copyfile(source, output)
        assert sha(output) == sha(source), source
    report = {
        "meta": {
            "rate": 48000,
            "script_sha256": sha(SCRIPT),
            "selection_sha256": sha(DECISIONS),
            "scope": "Full original takes with provisional recognition-derived sentence starts.",
        },
        "lines": lines,
    }
    with (args.output / "report.json").open("x") as stream:
        json.dump(report, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
    proof = {
        "ready_cues": list(takes),
        "pending_cues": pending,
        "sample_count": sum(row["out_samples"] for rows in lines.values() for row in rows),
        "sentence_rows": sum(map(len, lines.values())),
        "wav_sha256": {cue: sha(path) for cue, path in takes.items()},
        "script_sha256": sha(SCRIPT),
        "report_sha256": sha(args.output / "report.json"),
        "scope": "Provisional preview. Whole-take copying and sentence timing do not establish listening or fine sync.",
    }
    with (args.output / "preview-audio-receipt01.json").open("x") as stream:
        json.dump(proof, stream, indent=2)
        stream.write("\n")
    print(f"{len(takes)} intact WAVs, {proof['sentence_rows']} sentence rows, {len(pending)} cues outstanding")


if __name__ == "__main__":
    main()
