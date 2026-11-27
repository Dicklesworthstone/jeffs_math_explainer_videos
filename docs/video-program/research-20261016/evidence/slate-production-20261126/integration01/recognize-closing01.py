"""Bind both intact Frederick takes for each closing cue before guarded ASR."""

import argparse
import csv
import os
from pathlib import Path
import runpy
import sys


HERE = Path(__file__).resolve().parent
PREVIEW = HERE / "prepare-voiced-preview01.py"
RECORDER = HERE / "retain-closing-reads01.py"
MIDDLE = HERE / "recognize-middle01.py"
previous = runpy.run_path(str(MIDDLE))
WORK, GUARD, FW, MODEL = (previous[name] for name in ("WORK", "GUARD", "FW", "MODEL"))
read, digest, checked_receipt = (previous[name] for name in ("read", "digest", "checked_receipt"))
RECORDING = WORK / "anytime-valid-frederick-closing01"
JOB = "anytime-valid-frederick-closing-asr01"
SCRIPT = HERE / "script02.tsv"
LOCK = HERE / "script02-receipt.json"


def script_cues():
    with SCRIPT.open(newline="") as file:
        script = {row["id"]: row["text"] for row in csv.DictReader(file, delimiter="\t")}
    assert len(script) == 41 and digest(SCRIPT) == read(LOCK)["script02_sha256"]
    closing = {k: v for k, v in script.items() if int(k.split("_")[0]) >= 12}
    assert len(closing) == 8 and sum(map(len, closing.values())) == 2978
    return script, closing


def bindings():
    _, closing = script_cues()
    resource = RECORDING / "resource-receipt.json"
    manifest = RECORDING / "retained-takes01.json"
    checked_receipt(resource, 0)
    batch = read(manifest)
    assert batch["script_sha256"] == digest(SCRIPT)
    assert [row["cue"] for row in batch["cues"]] == list(closing)
    verified = {Path(__file__).resolve(), PREVIEW, RECORDER, MIDDLE, SCRIPT, LOCK,
                resource, manifest, FW, MODEL}
    wav_check = runpy.run_path(str(PREVIEW))["checked_wav"]
    takes = {}
    last_balance = None
    for record in batch["cues"]:
        cue = record["cue"]
        receipt = RECORDING / cue / "receipt.json"
        assert record == read(receipt)
        assert record["text"] == closing[cue]
        assert record["voice"] == "Frederick Surrey - Smooth and Velvety"
        assert (record["model"], record["stability"]) == ("Eleven v3", 0.5)
        before, after = record["credits_before"], record["credits_after"]
        assert before - after == len(closing[cue])
        assert last_balance is None or last_balance == before
        last_balance = after
        verified.add(receipt)
        pair = record["takes"]
        assert len(pair) == 2 and len({item["sha256"] for item in pair}) == 2
        takes[cue] = []
        for index, take in enumerate(pair, 1):
            assert take["path"] == f"take{index}.wav"
            path = receipt.parent / take["path"]
            assert wav_check(path) == take["sample_frames"] and digest(path) == take["sha256"]
            assert take["sample_rate"] == 48000 and path not in verified
            verified.add(path)
            takes[cue].append(path)
    assert len(takes) == 8 and len({w for pair in takes.values() for w in pair}) == 16
    return sorted(verified), takes


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("inspect", "launch"))
    args = parser.parse_args()
    if args.mode == "inspect" and not (RECORDING / "retained-takes01.json").is_file():
        _, closing = script_cues()
        print(f"Closing script verified for {len(closing)} cues and 2,978 included credits. Retained WAVs are pending.")
        return
    paths, takes = bindings()
    if args.mode == "inspect":
        print(f"Verified 16 intact closing alternatives for {len(takes)} cues against {len(paths)} inputs.")
        return
    assert not (WORK / JOB).exists()
    command = [sys.executable, "-I", "-B", str(GUARD), "--phase", "asr"]
    for path in paths:
        command.extend(("--input", str(path)))
    command.extend((JOB, "--", str(FW), "transcribe", "--backend", "whisper-cpp",
                    "--no-diarize", "--no-persist", "--language", "en", "--split-on-word",
                    "--json", "--model", str(MODEL)))
    for pair in takes.values():
        for path in pair:
            command.extend(("--input", str(path)))
    os.execv(sys.executable, command)


if __name__ == "__main__":
    main()
