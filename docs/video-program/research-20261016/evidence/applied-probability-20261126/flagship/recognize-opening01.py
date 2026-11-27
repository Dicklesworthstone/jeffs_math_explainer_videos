"""Verify retained opening recordings and run the local word-timed recognizer."""

import csv
import hashlib
import json
import os
from pathlib import Path
import sys
import wave


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[5]
WORK = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
RECORDING = WORK / "anytime-valid-frederick-browser01"
GUARD = WORK / "navier-production-20261120/live-assembly01/wake-dense-fields01/capture_guard184.py"
FW = Path("/Users/jemanuel/.local/bin/fw")
MODEL = Path("/Users/jemanuel/.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin")
JOB = "anytime-valid-frederick-opening-asr01"


def read(path):
    return json.loads(path.read_text())


def digest(path):
    with path.open("rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def bindings():
    guard_path = RECORDING / "resource-receipt.json"
    guard = read(guard_path)
    assert guard["admitted"] and guard["finished"] and guard["exit"] == 0
    assert guard["inputs_sha256"] == guard["final_inputs_sha256"]
    assert not guard.get("stopped_for_resource_limit")
    lock = HERE / "script-lock01.json"
    script = HERE / "script01.tsv"
    recorded = RECORDING / "retained-takes01.json"
    manifest = read(recorded)
    assert read(lock)["script_sha256"] == manifest["script_sha256"] == digest(script)
    with script.open(newline="") as text:
        rows = list(csv.DictReader(text, delimiter="\t"))[:21]
    assert len(rows) == len(manifest["cues"]) == 21
    inputs = {Path(__file__).resolve(), FW, MODEL, guard_path, lock, script, recorded}
    audio = []
    for row, receipt in zip(rows, manifest["cues"], strict=True):
        cue = row["id"]
        receipt_path = RECORDING / cue / "receipt.json"
        assert receipt == read(receipt_path)
        assert (receipt["cue"], receipt["text"]) == (cue, row["text"])
        assert receipt["voice"] == "Frederick Surrey - Smooth and Velvety"
        assert receipt["model"] == "Eleven v3" and receipt["stability"] == 0.5
        assert receipt["credits_before"] - receipt["credits_after"] == len(row["text"])
        assert len(receipt["takes"]) == 2
        inputs.add(receipt_path)
        for index, take in enumerate(receipt["takes"], 1):
            assert take["path"] == f"take{index}.wav"
            path = RECORDING / cue / take["path"]
            assert digest(path) == take["sha256"] and take["sample_rate"] == 48000
            with wave.open(str(path), "rb") as wav:
                assert wav.getparams()[:4] == (1, 2, 48000, take["sample_frames"])
                assert len(wav.readframes(take["sample_frames"])) == 2 * take["sample_frames"]
            inputs.add(path)
            audio.append(path)
    assert len(audio) == len(set(audio)) == 42
    assert FW.is_file() and MODEL.is_file()
    return sorted(inputs), audio


def main():
    options = sys.argv[1:]
    assert options in (["inspect"], ["launch"])
    inputs, audio = bindings()
    if options == ["inspect"]:
        print(f"{len(audio)} full original WAVs; {len(inputs)} frozen inputs.")
        return
    assert not (WORK / JOB).exists()
    cmd = [sys.executable, "-I", "-B", str(GUARD), "--phase", "asr"]
    for path in inputs:
        cmd.extend(["--input", str(path)])
    cmd.extend([JOB, "--", str(FW), "transcribe", "--backend", "whisper-cpp",
                "--no-diarize", "--no-persist", "--language", "en", "--split-on-word",
                "--json", "--model", str(MODEL)])
    for path in audio:
        cmd.extend(["--input", str(path)])
    os.execv(sys.executable, cmd)


if __name__ == "__main__":
    main()
