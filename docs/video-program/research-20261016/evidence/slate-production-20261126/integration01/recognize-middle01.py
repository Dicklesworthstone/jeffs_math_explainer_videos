"""Run the local recognizer on every original take from the middle batch."""
import argparse
import csv
import json
import os
from pathlib import Path
import runpy
import sys
from time import monotonic, sleep
import wave


HERE = Path(__file__).resolve().parent
OPENING = HERE.parents[1] / "applied-probability-20261126" / "flagship" / "recognize-opening01.py"
SOURCE = runpy.run_path(str(OPENING))
WORK, GUARD, FW, MODEL = (SOURCE[key] for key in ("WORK", "GUARD", "FW", "MODEL"))
read, digest = SOURCE["read"], SOURCE["digest"]
PREVIOUS = WORK/"anytime-valid-frederick-middle01"
RECOVERY = HERE/"08_3-recovery02"
RECORDINGS = WORK/"anytime-valid-frederick-middle02"
OUTPUT_JOB = "anytime-valid-frederick-middle-asr01"
SCRIPT = HERE/"script02.tsv"


def checked_receipt(path, exit_code):
    receipt = read(path)
    assert receipt["admitted"] and receipt["finished"] and receipt["exit"] == exit_code, path
    assert receipt["inputs_sha256"] == receipt["final_inputs_sha256"], path
    assert not receipt.get("stopped_for_resource_limit"), path


def bindings(require_finished):
    script_hash = digest(SCRIPT)
    assert script_hash == read(HERE/"script02-receipt.json")["script02_sha256"]
    with SCRIPT.open(newline="") as source:
        script = {row["id"]: row["text"] for row in csv.DictReader(source, delimiter="\t")}
    previous = PREVIOUS/"resource-receipt.json"
    checked_receipt(previous, 124)
    paths = {Path(__file__).resolve(), OPENING, SCRIPT, HERE/"script02-receipt.json", previous, FW, MODEL}
    cue_files = [PREVIOUS/cue/"receipt.json" for cue in ("08_1", "08_2")] + [RECOVERY/"receipt.json"]
    if require_finished:
        record_guard = RECORDINGS/"resource-receipt.json"
        checked_receipt(record_guard, 0)
        retained = RECORDINGS/"retained-takes01.json"
        recorded = read(retained)
        assert recorded["script_sha256"] == script_hash
        ids = [cue for cue in script if "09_1" <= cue <= "11_3"] + ["03_1"]
        assert [row["cue"] for row in recorded["cues"]] == ids
        for row in recorded["cues"]:
            cue_file = RECORDINGS/row["cue"]/"receipt.json"
            assert read(cue_file) == row
            cue_files.append(cue_file)
        paths.update((retained, record_guard))
    audio = []
    for receipt_file in cue_files:
        receipt = read(receipt_file)
        text = script[receipt["cue"]]
        assert text == receipt["text"]
        assert receipt["voice"] == "Frederick Surrey - Smooth and Velvety"
        assert (receipt["model"], receipt["stability"]) == ("Eleven v3", 0.5)
        assert receipt["credits_before"] - receipt["credits_after"] == len(text)
        takes = receipt["takes"]
        assert len(takes) == len({item["sha256"] for item in takes}) == 2
        paths.add(receipt_file)
        for take in takes:
            wav_path = receipt_file.parent/take["path"]
            frames = take["sample_frames"]
            assert take["sample_rate"] == 48000 and digest(wav_path) == take["sha256"]
            with wave.open(str(wav_path),"rb") as wav:
                assert wav.getparams()[:4] == (1,2,48000,frames)
                assert len(wav.readframes(frames)) == 2*frames
            assert wav_path not in paths
            paths.add(wav_path)
            audio.append(wav_path)
    assert len(audio) == (26 if require_finished else 6)
    assert all(path.is_file() for path in paths)
    return sorted(paths), audio


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode",choices=("inspect","wait"))
    args = parser.parse_args()
    if args.mode == "inspect":
        paths,audio = bindings(False)
        print(f"Verified {len(audio)} intact originals for Chapter 8 against {len(paths)} files.")
        return
    deadline = monotonic() + 3*3600
    guard = RECORDINGS/"resource-receipt.json"
    print("Waiting for the named middle recording guard to finish.",flush=True)
    while monotonic() < deadline:
        try:
            receipt = read(guard)
        except (FileNotFoundError, json.JSONDecodeError):
            receipt = {}
        if receipt.get("finished"):
            break
        sleep(10)
    else:
        raise SystemExit("Recording never reached a terminal receipt.")
    paths,audio = bindings(True)
    assert not (WORK/OUTPUT_JOB).exists()
    command = [sys.executable,"-I","-B",str(GUARD),"--phase","asr"]
    for path in paths:
        command.extend(["--input",str(path)])
    command.extend([OUTPUT_JOB,"--",str(FW),"transcribe","--backend","whisper-cpp",
                    "--no-diarize","--no-persist","--language","en","--split-on-word",
                    "--json","--model",str(MODEL)])
    for path in audio:
        command.extend(["--input",str(path)])
    print(f"Starting ASR for {len(audio)} originals.",flush=True)
    os.execv(sys.executable,command)


if __name__ == "__main__":
    main()
