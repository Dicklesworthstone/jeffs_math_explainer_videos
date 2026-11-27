"""Bind complete ElevenLabs batches to the companion scripts and native voice input."""

import argparse
import json
from pathlib import Path
import runpy
import shutil
import sys


HERE = Path(__file__).resolve().parent
REGISTRY = HERE / "companion-recording-plan01.py"
VOICE = HERE / "flagship-final-voice02.py"
CAPTION_REPORT = HERE / "native-caption-report01.py"
REQUIREMENTS = HERE / "native-sentence-requirements01.json"
registry = runpy.run_path(str(REGISTRY))
voice = runpy.run_path(str(VOICE))
REPO, WORK = voice["REPO"], voice["WORK"]
read, digest, write = (voice[key] for key in ("read", "digest", "write_json"))
checked_wav, check_boundaries = (
    voice["old"][key] for key in ("checked_wav", "check_boundaries")
)


def successful_receipt(directory, *phases):
    path = directory / "resource-receipt.json"
    receipt = read(path)
    assert receipt["admitted"] and receipt["finished"] and receipt["exit"] == 0, path
    assert receipt["phase"] in phases and not receipt.get("stopped_for_resource_limit"), path
    assert not receipt.get("stop_reason"), path
    assert receipt["inputs_sha256"] == receipt["final_inputs_sha256"], path
    return path


def film_inputs(film):
    bindings = registry["reviewed_robotics"]() if film == "robotics" else set()
    plan_path = registry["recording_plan_path"](film)
    plan = read(plan_path)
    entry, = [record for record in plan["films"] if record["film"] == film]
    assert registry["plan"](film) == entry, f"{film}: the native recording plan changed"
    speech = {row["cue"]: row["text"] for row in entry["cues"]}
    script = REPO / entry["script"]
    receipt = successful_receipt((REPO / entry["preview_receipt"]).parent, "native", "preview")
    source_name = script.parent.name
    requirements_path = HERE / "native-sentence-requirements04.json" if film == "robotics" else REQUIREMENTS
    requirements = read(requirements_path)["films"][source_name]
    assert requirements["script_sha256"] == digest(script)
    for name, expected in requirements["files_sha256"].items():
        assert digest(script.parent / name) == expected, name
    files = {Path(__file__).resolve(), plan_path, REGISTRY, VOICE, CAPTION_REPORT, requirements_path,
             voice["PREVIOUS"], HERE / "prepare-voiced-preview01.py", voice["PREVIEW"]["CLOCK"],
             script, receipt, REPO / entry["native_executable"], *bindings}
    return plan, entry, speech, requirements["required_sentence_indices"], files


def recorded_batches(batches, speech, script_hash, voice_settings):
    pairs, fingerprints, paths = {}, {}, set()
    for directory in batches:
        receipt = successful_receipt(directory, "tts")
        manifest = directory / "retained-takes01.json"
        batch = read(manifest)
        assert batch["script_sha256"] == script_hash and batch["cues"], manifest
        paths.update((receipt, manifest))
        previous_balance = None
        for row in batch["cues"]:
            cue = row["cue"]
            assert cue in speech and cue not in pairs and row["text"] == speech[cue], cue
            metadata = directory / cue / "receipt.json"
            assert row == read(metadata), metadata
            assert all(row[key] == voice_settings[key] for key in ("voice", "model", "stability")), cue
            before, after = row["credits_before"], row["credits_after"]
            assert before - after == len(speech[cue]) and after >= 0, cue
            assert previous_balance is None or before == previous_balance, cue
            previous_balance = after
            paths.add(metadata)
            alternatives = []
            assert len(row["takes"]) == len({take["sha256"] for take in row["takes"]}) == 2, cue
            for number, take in enumerate(row["takes"], 1):
                assert take["path"] == f"take{number}.wav", cue
                path = metadata.parent / take["path"]
                assert path not in fingerprints and take["sample_rate"] == 48000, path
                assert checked_wav(path) == take["sample_frames"] > 0, path
                assert digest(path) == take["sha256"], path
                fingerprints[path] = take["sha256"]
                alternatives.append(path)
                paths.add(path)
            pairs[cue] = alternatives
    return {cue: pairs[cue] for cue in speech if cue in pairs}, fingerprints, paths


def recognized_words(directories, fingerprints):
    segments, paths, log_hashes = {}, set(), []
    for directory in directories:
        receipt = successful_receipt(directory, "asr")
        log = directory / "command.log"
        manifest = WORK / "navier-production-20261120" / f"{directory.name}-inputs.json"
        recorded_inputs = {Path(row["path"]): row["sha256"] for row in read(manifest)["files"]}
        paths.update((receipt, log, manifest))
        log_hashes.append(digest(log))
        for line in log.read_text().splitlines():
            row = json.loads(line)
            path = Path(row["input"])
            assert row["status"] == "ok" and path in fingerprints and path not in segments, path
            assert recorded_inputs.get(path) == fingerprints[path], path
            segments[path] = row["report"]["result"]["segments"]
            assert segments[path], path
    assert set(segments) == set(fingerprints), "recognition coverage"
    return segments, paths, log_hashes


def selected_voice(speech, pairs, fingerprints, segments, choices, required_indices):
    assert list(speech) == list(pairs) and set(choices) == set(speech)
    timing = runpy.run_path(str(voice["PREVIEW"]["CLOCK"]))["rows_for_cue"]
    originals, rows = {}, {}
    for cue, line in speech.items():
        choice = choices[cue]
        assert type(choice) is int and choice in (1, 2), cue
        path = pairs[cue][choice - 1]
        count = checked_wav(path)
        try:
            timed = timing(line, segments[path], count)
            check_boundaries(timed, count, line)
        except (AssertionError, ValueError) as error:
            raise ValueError(f"Review provisional sentence boundaries for {cue}") from error
        assert all(index < len(timed) for index in required_indices.get(cue, [])), cue
        for sentence in timed:
            sentence.update(source=str(path.relative_to(REPO)), source_sha256=fingerprints[path])
        originals[cue], rows[cue] = path, timed
    return originals, rows


def write_package(output, originals, rows, inputs, script_hash, selection):
    before = {str(path): digest(path) for path in sorted(inputs)}
    output.mkdir()
    for cue, source in originals.items():
        target = output / f"{cue}.wav"
        with source.open("rb") as inp, target.open("xb") as out:
            shutil.copyfileobj(inp, out)
        assert digest(target) == before[str(source)] == rows[cue][0]["source_sha256"], cue
    report = runpy.run_path(str(CAPTION_REPORT))["compatible_report"]({
        "meta": {"rate": 48000, "script_sha256": script_hash, "selection_sha256": digest(selection),
                 "scope": "Intact original takes with provisional internal sentence boundaries."},
        "lines": rows,
    })
    report_path = output / "report.json"
    write(report_path, report)
    assert all(digest(Path(path)) == value for path, value in before.items())
    write(output / "voice-candidate-receipt01.json", {
        "cues": list(originals),
        "sample_count": sum(row["out_samples"] for group in rows.values() for row in group),
        "sentence_rows": sum(map(len, rows.values())),
        "wav_sha256": {cue: digest(output / f"{cue}.wav") for cue in originals},
        "script_sha256": script_hash, "report_sha256": digest(report_path),
        "inputs_sha256": before, "perceptual_acceptance": False,
    })


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("film", choices=tuple(registry["FILMS"]))
    parser.add_argument("mode", choices=("inspect", "launch-asr", "launch-package", "package"))
    parser.add_argument("--batch", type=Path, action="append", default=[])
    parser.add_argument("--asr", type=Path, action="append", default=[])
    parser.add_argument("--selection", type=Path)
    parser.add_argument("--job", default="")
    args = parser.parse_args()
    plan, entry, speech, requirements, inputs = film_inputs(args.film)
    pairs, fingerprints, paths = recorded_batches(
        [path.resolve() for path in args.batch], speech, entry["script_sha256"], plan,
    )
    inputs.update(paths)
    if args.mode == "inspect":
        print(f"{len(pairs)} of {len(speech)} recorded cues for {args.film} on its exact native script.")
        return
    assert args.job.startswith(f"{args.film}-") and list(pairs) == list(speech)
    if args.mode == "launch-asr":
        asr = voice["ASR"]
        paths = [path for pair in pairs.values() for path in pair]
        inputs.update((asr["FW"], asr["MODEL"]))
        command = [asr["FW"], "transcribe", "--backend", "whisper-cpp", "--no-diarize",
                   "--no-persist", "--language", "en", "--split-on-word", "--json", "--model", asr["MODEL"]]
        for path in paths:
            command.extend(("--input", path))
        voice["guard"](args.job, "asr", inputs, command)
        return

    assert args.selection is not None
    choice_path = args.selection.resolve()
    selection = read(choice_path)
    segments, paths, log_hashes = recognized_words([path.resolve() for path in args.asr], fingerprints)
    assert selection["script_sha256"] == entry["script_sha256"]
    assert selection["asr_log_sha256"] == log_hashes and selection["perceptual_acceptance"] is False
    inputs.update((*paths, choice_path))
    originals, rows = selected_voice(speech, pairs, fingerprints, segments, selection["takes"], requirements)
    if args.mode == "launch-package":
        command = [sys.executable, "-I", "-B", Path(__file__).resolve(), args.film, "package", "--job", args.job,
                   "--selection", choice_path]
        for option, values in (("--batch", args.batch), ("--asr", args.asr)):
            for value in values:
                command.extend((option, value.resolve()))
        voice["guard"](args.job, "asr", inputs, command)
        return
    package = WORK / args.job
    active = read(package / "resource-receipt.json")
    assert active["admitted"] and active["phase"] == "asr" and not active.get("finished")
    write_package(package / "narration", originals, rows, inputs, entry["script_sha256"], choice_path)


if __name__ == "__main__":
    main()
