"""Build a complete 48 kHz narration candidate from all selected, intact originals."""

import argparse
import json
from pathlib import Path
import runpy
import shutil


HERE = Path(__file__).resolve().parent
CLOSING = HERE / "recognize-closing01.py"
PREPARE = HERE / "prepare-voiced-preview01.py"
ASR = runpy.run_path(str(CLOSING))
PREVIEW = runpy.run_path(str(PREPARE))
WORK, read, digest, checked_receipt = (ASR[name] for name in ("WORK", "read", "digest", "checked_receipt"))
checked_wav, check_boundaries = (PREVIEW[name] for name in ("checked_wav", "check_boundaries"))
PRIOR = WORK / "anytime-valid-voiced-preview01"


def retained_opening():
    speech, pending = ASR["script_cues"]()
    prior_guard = PRIOR / "resource-receipt.json"
    checked_receipt(prior_guard, 0)
    root = PRIOR / "narration"
    summary = root / "preview-audio-receipt01.json"
    report = root / "report.json"
    old = read(summary)
    assert old["script_sha256"] == digest(ASR["SCRIPT"])
    assert old["report_sha256"] == digest(report) and old["pending_cues"] == list(pending)
    old_report = read(report)
    assert old_report["meta"]["rate"] == 48000
    assert old["ready_cues"] == list(old_report["lines"]) == [cue for cue in speech if cue not in pending]
    assert len(old["ready_cues"]) == 33
    wavs, rows = {}, old_report["lines"]
    sources = {prior_guard, summary, report, PREPARE, CLOSING, Path(__file__).resolve()}
    for cue in old["ready_cues"]:
        take = root / f"{cue}.wav"
        original = PREVIEW["REPO"] / rows[cue][0]["source"]
        expected = old["wav_sha256"][cue]
        assert digest(take) == expected == digest(original)
        assert all(row["source_sha256"] == expected for row in rows[cue])
        check_boundaries(rows[cue], checked_wav(take), speech[cue])
        sources.update((take, original))
        wavs[cue] = take
    return speech, pending, wavs, rows, sources


def complete_inputs(choice):
    speech, closing, wavs, rows, paths = retained_opening()
    paths_from_recording, takes = ASR["bindings"]()
    run = WORK / ASR["JOB"]
    resource, log = run / "resource-receipt.json", run / "command.log"
    checked_receipt(resource, 0)
    proof = read(choice)
    assert proof["script_sha256"] == digest(ASR["SCRIPT"]) and proof["asr_log_sha256"] == digest(log)
    assert proof["perceptual_acceptance"] is False
    assert set(proof["takes"]) == set(closing)
    measured = [json.loads(line) for line in log.read_text().splitlines()]
    assert len(measured) == 16 and all(item["status"] == "ok" for item in measured)
    by_path = {Path(row["input"]): row["report"]["result"]["segments"] for row in measured}
    assert set(by_path) == {path for pair in takes.values() for path in pair}
    clock_file = PREVIEW["CLOCK"]
    clock = runpy.run_path(str(clock_file))["rows_for_cue"]
    for cue, text in closing.items():
        choice_for_cue = proof["takes"][cue]
        assert type(choice_for_cue) is int and choice_for_cue in (1, 2)
        take = takes[cue][choice_for_cue - 1]
        count = checked_wav(take)
        timed = clock(text, by_path[take], count)
        for row in timed:
            row.update(source=str(take.relative_to(PREVIEW["REPO"])), source_sha256=digest(take))
        check_boundaries(timed, count, text)
        rows[cue] = timed
        wavs[cue] = take
    assert list(rows) == list(wavs) == list(speech) and len(wavs) == 41
    paths.update((resource, log, clock_file, choice, ASR["SCRIPT"], *paths_from_recording))
    return wavs, rows, sorted(paths)


def write_json(path, payload):
    with path.open("x") as stream:
        json.dump(payload, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def package(output, choice):
    wavs, rows, paths = complete_inputs(choice)
    before = {str(path): digest(path) for path in paths}
    output.mkdir()
    for cue, source in wavs.items():
        target = output / (cue + ".wav")
        shutil.copyfile(source, target)
        assert digest(target) == before[str(source)]
    report = output / "report.json"
    write_json(report, {
        "meta": {"rate": 48000, "script_sha256": digest(ASR["SCRIPT"]),
                 "selection_sha256": digest(choice),
                 "scope": "All 41 intact originals. ASR-derived internal clocks remain provisional."},
        "lines": rows,
    })
    assert all(digest(Path(path)) == value for path, value in before.items())
    write_json(output / "voice-candidate-receipt01.json", {
        "cues": list(wavs),
        "sample_count": sum(row["out_samples"] for group in rows.values() for row in group),
        "sentence_rows": sum(map(len, rows.values())),
        "wav_sha256": {cue: digest(output / (cue + ".wav")) for cue in wavs},
        "script_sha256": digest(ASR["SCRIPT"]),
        "report_sha256": digest(report),
        "inputs_sha256": before,
        "perceptual_acceptance": False,
    })


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--inspect", action="store_true")
    mode.add_argument("--inputs", action="store_true")
    mode.add_argument("--output", type=Path)
    parser.add_argument("--selection", type=Path)
    args = parser.parse_args()
    if args.inspect:
        _, pending, retained, rows, _ = retained_opening()
        print(f"Verified {len(retained)} complete selected WAVs and {sum(map(len, rows.values()))} sentence rows.")
        print(f"{len(pending)} additional cues require stable recording and recognition receipts before full assembly.")
        return
    if args.selection is None:
        parser.error("--selection is required for a full candidate")
    if args.inputs:
        print(json.dumps([str(path) for path in complete_inputs(args.selection)[2]], indent=2))
    else:
        package(args.output, args.selection)


if __name__ == "__main__":
    main()
