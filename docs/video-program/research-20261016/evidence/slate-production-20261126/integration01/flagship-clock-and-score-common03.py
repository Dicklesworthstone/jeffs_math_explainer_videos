"""Verify the full retained voice package, then build the native clock and score."""

import argparse
import json
import os
from pathlib import Path
import runpy
import subprocess
import sys


HERE = Path(__file__).resolve().parent
PREPARE = HERE / "prepare-complete-voice01.py"
CLOCK_REUSE = HERE / "flagship-early-clock-reuse02.py"
voice = runpy.run_path(str(PREPARE))
clock_reuse = runpy.run_path(str(CLOCK_REUSE))
WORK, PRIOR, read, digest = (voice[key] for key in ("WORK", "PRIOR", "read", "digest"))
REPO = voice["PREVIEW"]["REPO"]
EXECUTABLE = REPO / "target/debug/probability_native_packaged"
THUMBNAIL_JOB = WORK / "anytime-valid-thumbnail01"
JOB = "anytime-valid-final-clock-score01"


def native_list(binary, command):
    return subprocess.check_output([binary, command], text=True).splitlines()


def verify_clock(folder, keys, narration, speech, lines):
    expected = [folder / (key + ".timeline.tsv") for key in keys]
    assert set(expected) == set(folder.glob("*.timeline.tsv")), "chapter timeline set"
    ordered = []
    chapters = []
    for key, path in zip(keys, expected):
        rows = [row.split("\t") for row in path.read_text().splitlines() if not row.startswith("#")]
        assert [row[1] for row in rows if row[0] == "chapter"] == [key]
        fps, = [row[1] for row in rows if row[0] == "fps"]
        assert fps == "60", (key, fps)
        picture, = [int(row[1]) for row in rows if row[0] == "frames"]
        assert picture > 0, key
        cues = []
        previous = 0
        for _, cue, frame, level, source in (row for row in rows if row[0] == "line"):
            assert float(level) == -3.0 and Path(source) == narration / f"{cue}.wav", (key, cue)
            start = int(frame) * 800
            end = start + sum(row["out_samples"] for row in lines[cue])
            assert previous <= start < end <= picture * 800, (key, cue)
            previous = end
            cues.append({"cue": cue, "start_sample": start, "end_sample": end})
            ordered.append(cue)
        assert cues and all(x["cue"].startswith(key[:2] + "_") for x in cues), key
        chapters.append({
            "key": key, "frames": picture, "timeline_sha256": digest(path), "cues": cues,
        })
    assert ordered == list(speech) == list(lines), "every approved take exactly once in order"
    return chapters


def package_inputs(narration, script, speech):
    assert len(speech) == 41
    package_path = narration / "voice-candidate-receipt01.json"
    report_path = narration / "report.json"
    package, report = read(package_path), read(report_path)
    assert package["script_sha256"] == report["meta"]["script_sha256"] == digest(script)
    assert package["report_sha256"] == digest(report_path)
    assert report["meta"]["rate"] == 48000 and package["perceptual_acceptance"] is False
    rows = report["lines"]
    assert list(package["cues"]) == list(rows) == list(speech)
    assert set(narration.glob("*.wav")) == {narration / f"{cue}.wav" for cue in speech}
    paths = {package_path, report_path, script}
    samples = 0
    for cue, text in speech.items():
        wav = narration / f"{cue}.wav"
        size = voice["checked_wav"](wav)
        voice["check_boundaries"](rows[cue], size, text)
        sha = package["wav_sha256"][cue]
        assert digest(wav) == sha, cue
        originals = {(REPO / row["source"]).resolve() for row in rows[cue]}
        assert len(originals) == 1, cue
        original, = originals
        assert digest(original) == sha and all(row["source_sha256"] == sha for row in rows[cue])
        samples += size
        paths.update((wav, original))
    assert samples == package["sample_count"]
    assert sum(map(len, rows.values())) == package["sentence_rows"]
    return speech, rows, paths


def build_inputs(narration):
    narration = narration.resolve()
    speech, pending = voice["ASR"]["script_cues"]()
    assert len(pending) == 8
    speech, lines, paths = package_inputs(narration, voice["ASR"]["SCRIPT"], speech)
    receipt = THUMBNAIL_JOB / "resource-receipt.json"
    voice["checked_receipt"](receipt, 0)
    source = HERE / "probability-voice02"
    paths.update((Path(__file__).resolve(), PREPARE, receipt, EXECUTABLE, source / "script.tsv"))
    assert [tuple(line.split("\t", 1)) for line in native_list(EXECUTABLE, "script")] == list(speech.items())
    keys = native_list(EXECUTABLE, "chapters")
    assert len(keys) == len(set(keys)) == 14
    assert [key[:2] for key in keys] == [f"{chapter:02d}" for chapter in range(1, 15)]
    paths.update(PRIOR / "clock60" / (key + ".timeline.tsv") for key in keys[:11])
    return speech, lines, keys, sorted(paths)


def clock_without_audio_directory(path):
    rows = [row.split("\t") for row in path.read_text().splitlines() if not row.startswith("#")]
    for row in rows:
        if row[0] == "line":
            row[-1] = Path(row[-1]).name
    return rows


def score_stem_parts(stem, samples):
    """Read the kit's whole or split 48 kHz stereo float32 score."""
    manifest = stem.with_suffix(".parts.tsv")
    if manifest.is_file():
        header, *rows = manifest.read_text().splitlines()
        assert header == "score-parts\t1" and len(rows) >= 2, manifest
        parts = []
        for row in rows:
            start, length, rate, sha, basename = row.split("\t")
            assert Path(basename).name == basename and rate == "48000", manifest
            parts.append((int(start), int(length), sha, stem.with_name(basename)))
    else:
        parts = [(0, samples, digest(stem), stem)]
    expected_start = 0
    result = []
    for start, length, sha, path in parts:
        assert start == expected_start and length > 0 and digest(path) == sha, path
        stream, = json.loads(subprocess.check_output([
            "ffprobe", "-v", "error", "-show_streams", "-of", "json", path,
        ]))["streams"]
        assert stream["codec_name"] == "pcm_f32le" and stream["channels"] == 2, path
        assert stream["sample_rate"] == "48000" and stream["time_base"] == "1/48000", path
        assert int(stream["duration_ts"]) == length, path
        result.append({"file": path.name, "start_sample": start, "samples": length, "sha256": sha})
        expected_start += length
    assert expected_start == samples, stem
    return {"parts": result, "manifest_sha256": digest(manifest) if manifest.is_file() else None}


def materialize_clock(clock, narration, keys, executable, reusable):
    assert set(reusable).issubset(keys)
    pending = [key for key in keys if key not in reusable]
    if reusable:
        clock.mkdir()
        clock_reuse["relocate"](clock, narration, reusable)
    if pending:
        subprocess.run([
            executable, "render", *pending, "--timeline-only", "--fps", "60",
            "--threads", "3", "--narration", narration, "--out", clock,
        ], check=True)


def run_clock_and_score(narration, speech, lines, keys, executable, out, *, reusable):
    clock = out / "clock60"
    materialize_clock(clock, narration, keys, executable, reusable)
    chapters = verify_clock(clock, keys, narration, speech, lines)
    prior = PRIOR / "clock60"
    earlier = {}
    for key in keys[:11]:
        same = clock_without_audio_directory(prior / (key + ".timeline.tsv"))
        current = clock_without_audio_directory(clock / (key + ".timeline.tsv"))
        earlier[key] = same == current
    audio = out / "score"
    subprocess.run([
        executable, "score", "--timelines", clock, "--out", audio,
    ], check=True)
    qa = audio / "stems" / "qa"
    fingerprints = {}
    for chapter in chapters:
        key = chapter["key"]
        path = qa / (key + ".json")
        stem = audio / "stems" / (key + ".music.f32.wav")
        assert read(path)["passed"] is True, key
        fingerprints[key] = {"qa_sha256": digest(path), **score_stem_parts(stem, 800 * chapter["frames"])}
    voice["write_json"](out / "clock-and-score01.json", {
        "frames": sum(chapter["frames"] for chapter in chapters),
        "chapters": chapters, "matches_preview_timelines_except_audio_directory": earlier,
        "reused_exact_native_clocks": [key for key in keys if key in reusable],
        "score": fingerprints,
        "scope": "Exact 41 retained cue placements and native score QA. Sentence clock is provisional.",
        "continuous_listening": False, "fine_sync": False, "independent_review": False,
    })


def execute(narration):
    speech, lines, keys, _ = build_inputs(narration)
    run_clock_and_score(narration, speech, lines, keys, EXECUTABLE, WORK / JOB, reusable={})


def inspect_preview():
    speech, _, wavs, lines, _ = voice["retained_opening"]()
    keys = sorted(path.name.removesuffix(".timeline.tsv") for path in (PRIOR / "clock60").glob("*.timeline.tsv"))
    chapters = verify_clock(
        PRIOR / "clock60", keys, PRIOR / "narration",
        {cue: speech[cue] for cue in wavs}, lines,
    )
    print(json.dumps({"tested_against": str(PRIOR), "chapters": len(chapters),
                      "frames": sum(x["frames"] for x in chapters),
                      "voice_cues": sum(len(chapter["cues"]) for chapter in chapters)}, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("inspect-preview", "launch", "execute"))
    parser.add_argument("--narration", type=Path)
    args = parser.parse_args()
    if args.mode == "inspect-preview":
        inspect_preview()
        return
    if args.narration is None:
        parser.error("--narration is required for the complete voice package")
    narration = args.narration.resolve()
    if args.mode == "execute":
        execute(narration)
        return
    assert not (WORK / JOB).exists()
    _, _, _, paths = build_inputs(narration)
    command = [sys.executable, "-I", "-B", str(voice["ASR"]["GUARD"]), "--phase", "native"]
    for path in paths:
        command.extend(("--input", str(path)))
    command.extend((JOB, "--", sys.executable, "-I", "-B", str(Path(__file__).resolve()),
                    "execute", "--narration", str(narration)))
    os.execv(sys.executable, command)


if __name__ == "__main__":
    main()
