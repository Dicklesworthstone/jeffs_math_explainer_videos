"""Verify a complete companion voice package and build its native clock, captions and score."""

import argparse
from fractions import Fraction
from pathlib import Path
import runpy
import shutil
import subprocess
import sys


HERE = Path(__file__).resolve().parent
NARRATION = HERE / "companion-narration01.py"
NATIVE_CLOCK = HERE / "flagship-clock-and-score01.py"
CAPTION_READBACK = HERE / "audit-voiced-captions02.py"
audio = runpy.run_path(str(NARRATION))
clock = runpy.run_path(str(NATIVE_CLOCK))
read, digest, write, REPO, WORK = (audio[key] for key in ("read", "digest", "write", "REPO", "WORK"))


def package_inputs(narration, speech, script_hash):
    package_path = narration / "voice-candidate-receipt01.json"
    report_path = narration / "report.json"
    package, report = read(package_path), read(report_path)
    assert package["perceptual_acceptance"] is False
    assert package["script_sha256"] == report["meta"]["script_sha256"] == script_hash
    assert package["report_sha256"] == digest(report_path) and report["meta"]["rate"] == 48000
    adapt = runpy.run_path(str(audio["CAPTION_REPORT"]))["compatible_report"]
    assert adapt(report) == report
    rows = report["lines"]
    assert list(speech) == package["cues"] == list(rows) == list(package["wav_sha256"])
    assert set(narration.glob("*.wav")) == {narration / f"{cue}.wav" for cue in speech}
    paths = {package_path, report_path}
    samples = 0
    for cue, text in speech.items():
        wav = narration / f"{cue}.wav"
        size = audio["checked_wav"](wav)
        audio["check_boundaries"](rows[cue], size, text)
        sha = package["wav_sha256"][cue]
        originals = {(REPO / row["source"]).resolve() for row in rows[cue]}
        original, = originals
        assert digest(wav) == digest(original) == sha, cue
        assert all(row["source_sha256"] == sha for row in rows[cue]), cue
        samples += size
        paths.update((wav, original))
    assert samples == package["sample_count"] and sum(map(len, rows.values())) == package["sentence_rows"]
    return rows, paths


# Native outputs use the crate name compiled into the executable.
def native_artifact_stem(binary):
    if binary.name == "frankenrobotics_native_representation_maps":
        return "frankenrobotics_native_representation"
    return binary.name


def caption_check(folder, name, lines, frames):
    parse = runpy.run_path(str(CAPTION_READBACK))["captions"]
    srt, vtt = (folder / f"{name}.en.{ext}" for ext in ("srt", "vtt"))
    english = parse(srt, False)
    assert english and english == parse(vtt, True)
    expected = [word for group in lines.values() for row in group for word in row["sentence"].split()]
    actual = [word for _, _, block in english for line in block for word in line.split()]
    assert actual == expected, "caption text must match all spoken sentences in order"
    previous = 0
    speeds = []
    for start, end, block in english:
        assert previous <= start < end <= round(Fraction(frames * 1000, 60))
        speeds.append(len(" ".join(block)) * 1000 / (end - start))
        previous = end
    return {"blocks": len(english), "maximum_characters_per_second": max(speeds),
            "blocks_above_20_cps": sum(rate > 20 for rate in speeds),
            "sha256": {path.name: digest(path) for path in (srt, vtt)}}


def clock_captions_score(output, binary, keys, narration, speech, lines):
    timeline = output / "clock60"
    subprocess.run([
        binary, "render", *keys, "--timeline-only", "--res", "3840x2160", "--fps", "60",
        "--threads", "3", "--narration", narration, "--out", timeline,
    ], check=True)
    chapters = clock["verify_clock"](timeline, keys, narration, speech, lines)
    frames = sum(chapter["frames"] for chapter in chapters)
    subprocess.run([
        binary, "captions", *keys, "--fps", "60", "--narration", narration, "--out", timeline,
    ], check=True)
    captions = caption_check(timeline, native_artifact_stem(binary), lines, frames)

    score = output / "score"
    subprocess.run([binary, "score", "--timelines", timeline, "--out", score], check=True)
    stems = score / "stems"
    scores = {}
    for chapter in chapters:
        key = chapter["key"]
        qa = stems / "qa" / f"{key}.json"
        assert read(qa)["passed"] is True, key
        stem = stems / f"{key}.music.f32.wav"
        actual = clock["score_stem_parts"](stem, 800 * chapter["frames"])
        scores[key] = {"qa_sha256": digest(qa), **actual}
    result = {
        "frames": frames, "chapters": chapters, "captions": captions, "score": scores,
        "scope": "Exact full take placements, native caption readback and score QA on a 60 fps clock.",
        "continuous_listening": False, "fine_sync": False, "independent_review": False,
    }
    write(output / "clock-captions-score01.json", result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("film", choices=tuple(audio["registry"]["FILMS"]))
    parser.add_argument("mode", choices=("inspect", "launch", "execute"))
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--job")
    args = parser.parse_args()
    _, entry, speech, _, inputs = audio["film_inputs"](args.film)
    package = args.package.resolve()
    package_guard = audio["successful_receipt"](package, "asr")
    narration = package / "narration"
    lines, paths = package_inputs(narration, speech, entry["script_sha256"])
    inputs.update((*paths, package_guard, Path(__file__).resolve(), NARRATION, NATIVE_CLOCK,
                   CAPTION_READBACK, Path(shutil.which("ffprobe")).resolve()))
    binary = REPO / entry["native_executable"]
    if args.mode == "inspect":
        print(f"{len(speech)} exact audio cues across {len(entry['chapters'])} native chapters.")
        return
    assert args.job and args.job.startswith(f"{args.film}-")
    if args.mode == "launch":
        command = [sys.executable, "-I", "-B", Path(__file__).resolve(), args.film, "execute",
                   "--package", package, "--job", args.job]
        audio["voice"]["guard"](args.job, "native", inputs, command)
        return
    output = WORK / args.job
    active = read(output / "resource-receipt.json")
    assert active["admitted"] and active["phase"] == "native" and not active.get("finished")
    clock_captions_score(output, binary, entry["chapters"], narration, speech, lines)


if __name__ == "__main__":
    main()
