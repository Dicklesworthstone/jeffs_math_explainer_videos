"""Retain the corrected Frederick readings and assemble the complete voice candidate."""

import argparse
import json
import os
from pathlib import Path
import runpy
import shutil
import sys


HERE = Path(__file__).resolve().parent
PREVIOUS = HERE / "prepare-complete-voice01.py"
REVISION = HERE / "verify-flagship-revision01.py"
PROOF = HERE / "flagship-script03-receipt01.json"
REPORT = HERE / "native-caption-report01.py"
SLATE_GUARD = HERE / "slate-resource200.py"
CAPTURE_BATCHES = HERE / "flagship-capture-batches01.py"
old = runpy.run_path(str(PREVIOUS))
revision = runpy.run_path(str(REVISION))
ASR, PREVIEW = old["ASR"], old["PREVIEW"]
WORK, read, digest, checked_receipt, write_json = (
    old[key] for key in ("WORK", "read", "digest", "checked_receipt", "write_json")
)
REPO = PREVIEW["REPO"]
SCRIPT, FLAGSHIP = (revision[key] for key in ("SCRIPT", "FLAGSHIP"))
KEEPER = FLAGSHIP / "retain-browser-opening01.py"
CAPTURE = "anytime-valid-frederick-revision01"
RECOGNITION = "anytime-valid-frederick-revision-asr01"
PACKAGE = "anytime-valid-revised-voice01"
RENDER = WORK / "anytime-valid-math-revision02"
BET_VISUAL = WORK / "anytime-valid-bet-label01"


def retained():
    _, _, originals, all_rows, inputs = old["retained_opening"]()
    speech, pending = revision["revision_cues"]()
    proof = read(PROOF)
    assert proof["script_sha256"] == digest(SCRIPT)
    picked = {cue: originals[cue] for cue in speech if cue not in pending}
    old_speech = revision["read_script"](revision["PREVIOUS"])
    assert list(picked) == proof["reused_originals"] and len(picked) == 28
    assert all(old_speech[cue] == speech[cue] for cue in picked)
    assert proof["pending"] == {cue: len(text) for cue, text in pending.items()}
    inputs.update((Path(__file__).resolve(), PREVIOUS, REVISION, PROOF, REPORT, KEEPER, SCRIPT))
    return speech, pending, picked, {cue: all_rows[cue] for cue in picked}, inputs


def keeper():
    helper = runpy.run_path(str(KEEPER))
    globals_ = helper["retain"].__globals__
    globals_["OUTPUT"] = WORK / CAPTURE
    globals_["SCRIPT"] = SCRIPT
    return helper


def guard(job, phase, paths, command):
    assert not (WORK / job).exists(), job
    invocation = [sys.executable, "-I", "-B", str(SLATE_GUARD), "--phase", phase]
    for path in sorted(paths):
        invocation.extend(("--input", str(path)))
    invocation.extend((job, "--", *map(str, command)))
    os.execv(sys.executable, invocation)


def capture_inputs():
    _, pending, _, _, inputs = retained()
    for candidate in (RENDER, BET_VISUAL):
        resource = candidate / "resource-receipt.json"
        checked_receipt(resource, 0)
        inputs.add(resource)
    return pending, inputs


def recorded():
    pending, inputs = capture_inputs()
    output = WORK / CAPTURE
    batching = runpy.run_path(str(CAPTURE_BATCHES))
    if not output.exists():
        takes, paths, _ = batching["batches"](WORK, pending, digest(SCRIPT))
        if list(takes) != list(pending):
            raise ValueError(f"{len(pending) - len(takes)} flagship cues still need complete takes.")
        inputs.update((*paths, ASR["FW"], ASR["MODEL"]))
        return takes, inputs
    if any(WORK.glob(batching["PREFIX"] + "*")):
        raise ValueError("Both legacy and batched captures exist. Reconcile the exact takes first.")
    resource, manifest = output / "resource-receipt.json", output / "retained-takes01.json"
    checked_receipt(resource, 0)
    batch = read(manifest)
    assert batch["script_sha256"] == digest(SCRIPT)
    assert [r["cue"] for r in batch["cues"]] == list(pending)
    inputs.update((resource, manifest, ASR["FW"], ASR["MODEL"]))
    takes, previous_balance = {}, None
    for row in batch["cues"]:
        cue = row["cue"]
        receipt = output / cue / "receipt.json"
        assert row == read(receipt) and row["text"] == pending[cue]
        assert row["voice"] == "Frederick Surrey - Smooth and Velvety"
        assert (row["model"], row["stability"]) == ("Eleven v3", 0.5)
        before, after = row["credits_before"], row["credits_after"]
        assert before - after == len(pending[cue]) and after >= 0
        assert previous_balance is None or before == previous_balance
        previous_balance = after
        inputs.add(receipt)
        pair = row["takes"]
        assert len(pair) == len({take["sha256"] for take in pair}) == 2
        paths = []
        for index, take in enumerate(pair, 1):
            assert take["path"] == f"take{index}.wav"
            path = receipt.parent / take["path"]
            assert path not in inputs and take["sample_rate"] == 48000
            assert old["checked_wav"](path) == take["sample_frames"] and digest(path) == take["sha256"]
            paths.append(path)
            inputs.add(path)
        takes[cue] = paths
    assert len(takes) == 13
    return takes, inputs


def complete(selection):
    speech, pending, originals, rows, inputs = retained()
    takes, captured = recorded()
    recognition = WORK / RECOGNITION
    resource, log = recognition / "resource-receipt.json", recognition / "command.log"
    checked_receipt(resource, 0)
    chosen = read(selection)
    assert chosen["script_sha256"] == digest(SCRIPT) and chosen["asr_log_sha256"] == digest(log)
    assert chosen["perceptual_acceptance"] is False and set(chosen["takes"]) == set(pending)
    data = [json.loads(line) for line in log.read_text().splitlines()]
    assert len(data) == 26 and all(item["status"] == "ok" for item in data)
    matched = {Path(row["input"]): row["report"]["result"]["segments"] for row in data}
    assert set(matched) == {path for pair in takes.values() for path in pair}
    timer = PREVIEW["CLOCK"]
    timing = runpy.run_path(str(timer))["rows_for_cue"]
    for cue, line in pending.items():
        choice = chosen["takes"][cue]
        assert type(choice) is int and choice in (1, 2), cue
        path = takes[cue][choice - 1]
        samples = old["checked_wav"](path)
        sentences = timing(line, matched[path], samples)
        for sentence in sentences:
            sentence.update(source=str(path.relative_to(REPO)), source_sha256=digest(path))
        old["check_boundaries"](sentences, samples, line)
        originals[cue], rows[cue] = path, sentences
    inputs.update((*captured, resource, log, selection, timer))
    return ({cue: originals[cue] for cue in speech}, {cue: rows[cue] for cue in speech}, inputs)


def assemble(selection):
    originals, rows, inputs = complete(selection)
    hashes = {str(path): digest(path) for path in sorted(inputs)}
    folder = WORK / PACKAGE / "narration"
    folder.mkdir()
    for cue, source in originals.items():
        target = folder / f"{cue}.wav"
        shutil.copyfile(source, target)
        assert digest(target) == hashes[str(source)]
    report = runpy.run_path(str(REPORT))["compatible_report"]({
        "meta": {"rate": 48000, "script_sha256": digest(SCRIPT),
                 "selection_sha256": digest(selection),
                 "scope": "Intact original recordings on provisional ASR sentence clocks."},
        "lines": rows,
    })
    report_file = folder / "report.json"
    write_json(report_file, report)
    assert all(digest(Path(path)) == sha for path, sha in hashes.items())
    write_json(folder / "voice-candidate-receipt01.json", {
        "cues": list(originals),
        "sample_count": sum(row["out_samples"] for sentences in rows.values() for row in sentences),
        "sentence_rows": sum(map(len, rows.values())),
        "wav_sha256": {cue: digest(folder / f"{cue}.wav") for cue in originals},
        "script_sha256": digest(SCRIPT), "report_sha256": digest(report_file),
        "inputs_sha256": hashes, "perceptual_acceptance": False,
    })


def launch_asr():
    takes, inputs = recorded()
    command = [ASR["FW"], "transcribe", "--backend", "whisper-cpp", "--no-diarize",
               "--no-persist", "--language", "en", "--split-on-word", "--json",
               "--model", ASR["MODEL"]]
    for pair in takes.values():
        for path in pair:
            command.extend(("--input", path))
    guard(RECOGNITION, "asr", inputs, command)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="mode", required=True)
    for mode in ("inspect", "watch", "seal", "launch-asr"):
        commands.add_parser(mode)
    launch = commands.add_parser("launch-recording")
    launch.add_argument("included_credits", type=int)
    take = commands.add_parser("retain")
    take.add_argument("cue")
    take.add_argument("before", type=int)
    take.add_argument("after", type=int)
    take.add_argument("downloads", nargs=2, type=Path)
    for mode in ("launch-assembly", "assemble"):
        cmd = commands.add_parser(mode)
        cmd.add_argument("selection", type=Path)
    args = parser.parse_args()
    if args.mode == "inspect":
        _, pending, originals, _, _ = retained()
        for cue, text in pending.items():
            print(f"{cue}\t{len(text)}\t{text}")
        print(f"{len(originals)} intact prior reads and {len(pending)} outstanding cues "
              f"at {sum(map(len, pending.values()))} included credits.")
    elif args.mode == "launch-recording":
        batching = runpy.run_path(str(CAPTURE_BATCHES))
        if any(WORK.glob(batching["PREFIX"] + "*")):
            raise ValueError("Use the existing batched capture route.")
        _, pending, _, _, _ = retained()
        missing = sum(map(len, pending.values()))
        if args.included_credits < missing:
            raise SystemExit(f"Included balance {args.included_credits} is below {missing}.")
        _, inputs = capture_inputs()
        guard(CAPTURE, "tts", inputs, [sys.executable, "-I", "-B", Path(__file__), "watch"])
    elif args.mode == "watch":
        keeper()["watch"]()
    elif args.mode == "seal":
        _, pending = revision["revision_cues"]()
        keeper()["seal"](pending)
    elif args.mode == "retain":
        _, pending = revision["revision_cues"]()
        left = [cue for cue in pending if not (WORK / CAPTURE / cue / "receipt.json").exists()]
        assert left and left[0] == args.cue
        assert args.before >= sum(len(pending[cue]) for cue in left)
        keeper()["retain"](pending, args.cue, args.before, args.after, args.downloads)
    elif args.mode == "launch-asr":
        launch_asr()
    elif args.mode == "launch-assembly":
        selection = args.selection.resolve()
        _, _, inputs = complete(selection)
        guard(PACKAGE, "asr", inputs, [sys.executable, "-I", "-B", Path(__file__), "assemble", selection])
    else:
        assemble(args.selection.resolve())


if __name__ == "__main__":
    main()
