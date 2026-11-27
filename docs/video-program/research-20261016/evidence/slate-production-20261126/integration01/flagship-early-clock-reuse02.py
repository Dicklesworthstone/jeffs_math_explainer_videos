"""Carry the repaired Chapter 11 timing into the flagship's final score."""

import os
from pathlib import Path
import runpy


HERE = Path(__file__).resolve().parent
ORIGINAL = HERE / "flagship-early-clock-reuse01.py"
previous = runpy.run_path(str(ORIGINAL))
read, sha, relocate = (previous[name] for name in ("read", "sha", "relocate"))
TABLES = previous["TABLES"]
CHAPTER = "11_betting_on_the_ranks"


def stable_manifest(work, job, *, exit_code):
    path = job / "resource-receipt.json"
    if not path.is_file():
        return None
    state = read(path)
    if (not state.get("admitted") or not state.get("finished") or state.get("phase") != "native"
            or state.get("exit") != exit_code or state.get("stopped_for_resource_limit")
            or state.get("error") or state.get("stop_reason") or not state.get("inputs_sha256")
            or state["inputs_sha256"] != state.get("final_inputs_sha256")):
        return None
    manifest = work / "navier-production-20261120" / f"{job.name}-inputs.json"
    assert sha(manifest) == state["inputs_sha256"], manifest
    return {Path(row["path"]): row["sha256"] for row in read(manifest)["files"]}, {path, manifest}


def repaired_clock(work, narration, lines, binary, script):
    # The baseline succeeded at timing and failed only at the later picture
    # comparison. The completed repair froze that baseline and all its tables.
    earlier = work / "anytime-valid-early-picture01-11"
    repaired = work / "anytime-valid-repaired-uhd01-11"
    baseline = stable_manifest(work, earlier, exit_code=1)
    successor = stable_manifest(work, repaired, exit_code=0)
    if baseline is None or successor is None:
        return None
    initial, initial_paths = baseline
    frozen, repair_paths = successor
    baseline_hash = sha(binary)
    if initial.get(binary) != baseline_hash or frozen.get(binary) != baseline_hash:
        return None
    if frozen.get(script) != sha(script):
        return None

    receipt = repaired / "early-picture-receipt01.json"
    record = read(receipt)
    assert record["chapter"]["key"] == CHAPTER and record["script_sha256"] == sha(script)
    assert record["picture_repair"]["binary_sha256"] == record["binary_sha256"]
    assert record["chapter"]["frames"] == 4269
    source, baseline_audio = repaired / "narration", earlier / "narration"
    report = source / "report.json"
    assert sha(report) == record["narration_report_sha256"]
    early = read(report)
    cues = tuple(row["cue"] for row in record["chapter"]["cues"])
    assert early["meta"]["rate"] == 48000 and early["meta"]["script_sha256"] == sha(script)
    assert list(cues) == list(early["lines"]) == list(record["narration"])
    assert [cue for cue in lines if cue.startswith("11_")] == list(cues)
    wavs = [narration / f"{cue}.wav" for cue in cues]
    if (any(early["lines"][cue] != lines[cue] for cue in cues)
            or any(sha(wav) != record["narration"][wav.stem]["sha256"] for wav in wavs)):
        return None

    tables = {extension: repaired / "pictures" / f"{CHAPTER}.{extension}" for extension in TABLES}
    sources = {extension: earlier / "clock60" / path.name for extension, path in tables.items()}
    for extension, path in tables.items():
        old = sources[extension]
        assert sha(old) == frozen[old] and sha(path) == record["sha256"][extension]
        assert path.read_bytes() == old.read_bytes().replace(
            os.fsencode(baseline_audio) + b"/", os.fsencode(source) + b"/",
        ), extension
    return ((source, tables, cues, record["chapter"]["frames"]),
            initial_paths | repair_paths | {receipt, report, *wavs, *tables.values(), *sources.values()})


def bind(work, keys, narration, lines, binary, script):
    reusable, paths = previous["bind"](work, keys, narration, lines, binary, script)
    if CHAPTER in keys and CHAPTER not in reusable and not (narration / "phrase-timings.json").exists():
        repair = repaired_clock(work, narration, lines, binary, script)
        if repair is not None:
            reusable[CHAPTER], repair_paths = repair
            paths.update(repair_paths)
    if reusable:
        paths.add(Path(__file__).resolve())
    return {key: reusable[key] for key in keys if key in reusable}, paths
