"""Render the three companion films from their checked native voice and score clocks."""

import argparse
import os
import re
import runpy
import subprocess
import sys
from itertools import count
from pathlib import Path


HERE = Path(__file__).resolve().parent
CLOCK = HERE / "companion-clock-captions01.py"
DELIVERY = HERE / "flagship-native-delivery01.py"
prepared = runpy.run_path(str(CLOCK))
shared = runpy.run_path(str(DELIVERY))
audio = prepared["audio"]
REPO, WORK, read, digest, write = (audio[key] for key in ("REPO", "WORK", "read", "digest", "write"))
TABLES, RENDER_OPTIONS, FFMPEG, FFPROBE = (
    shared[key] for key in ("TABLES", "RENDER_OPTIONS", "FFMPEG", "FFPROBE")
)


def score_inputs(film, package, clock_dir):
    _, entry, speech, _, paths = audio["film_inputs"](film)
    paths.add(audio["successful_receipt"](package, "asr"))
    narration = package / "narration"
    lines, files = prepared["package_inputs"](narration, speech, entry["script_sha256"])
    paths.update(files)
    paths.add(audio["successful_receipt"](clock_dir, "native"))
    score_record = clock_dir / "clock-captions-score01.json"
    record = read(score_record)
    timeline = clock_dir / "clock60"
    keys = entry["chapters"]
    chapters = prepared["clock"]["verify_clock"](timeline, keys, narration, speech, lines)
    frames = {chapter["key"]: chapter["frames"] for chapter in chapters}
    assert chapters == record["chapters"] and sum(frames.values()) == record["frames"]
    binary = REPO / entry["native_executable"]
    caption_files = prepared["caption_check"](timeline, binary.name, lines, record["frames"])
    assert caption_files == record["captions"]
    stems = clock_dir / "score/stems"
    paths.update((Path(__file__).resolve(), CLOCK, DELIVERY, shared["MASTER_CLOCK"], score_record, FFMPEG, FFPROBE,
                  prepared["NATIVE_CLOCK"], prepared["CAPTION_READBACK"]))
    paths.update(timeline / name for name in caption_files["sha256"])
    for key in keys:
        qa = stems / "qa" / f"{key}.json"
        stem = stems / f"{key}.music.f32.wav"
        parts = prepared["clock"]["score_stem_parts"](stem, frames[key] * 800)
        assert read(qa)["passed"] is True
        assert {"qa_sha256": digest(qa), **parts} == record["score"][key], key
        paths.update((qa, *(stems / row["file"] for row in parts["parts"])))
        if parts["manifest_sha256"] is not None:
            paths.add(stem.with_suffix(".parts.tsv"))
        paths.update(timeline / f"{key}.{ext}" for ext in TABLES[:3])
    return binary, narration, lines, frames, timeline, stems, record["score"], paths


def chapter_job(film, key):
    return f"{film}-native-render01-{key[:2]}"


def input_manifest(job):
    return WORK / "navier-production-20261120" / f"{job}-inputs.json"


def stage_job(base):
    for attempt in count(1):
        job = base if attempt == 1 else f"{base}-attempt{attempt:02d}"
        folder = WORK / job
        path = folder / "resource-receipt.json"
        if not folder.exists() and not input_manifest(job).exists():
            return job
        if not path.is_file() or not read(path).get("finished"):
            raise RuntimeError(f"{job} has no terminal receipt. Check the existing job.")
        try:
            audio["successful_receipt"](folder, "native")
        except AssertionError:
            continue
        return job


def render_chapter(job, key, inputs):
    binary, narration, _, frames, timeline, stems, score, _ = inputs
    output = WORK / job / "pictures"
    assert not output.exists()
    subprocess.run([
        binary, "render", key, *RENDER_OPTIONS, "--narration", narration, "--score", stems, "--out", output,
    ], check=True, timeout=14400)
    files = shared["render_files"](output, key)
    for ext in ("timeline.tsv", "cues.tsv"):
        assert files[ext].read_bytes() == (timeline / f"{key}.{ext}").read_bytes(), (key, ext)
    shared["verify_cue_log"](files["cuelog.tsv"], timeline / f"{key}.cuelog.tsv", stems, score[key]["parts"])
    media = shared["verify_av"](files["mp4"], frames[key])
    write(output.parent / "chapter-render01.json", {
        "chapter": key, "media": media, "layout": files["layout.tsv"].read_text().splitlines(),
        "render_options": list(RENDER_OPTIONS),
        "sha256": {extension: digest(path) for extension, path in files.items()},
    })


def bind_guard_inputs(directory, expected):
    receipt = audio["successful_receipt"](directory, "native")
    manifest = input_manifest(directory.name)
    assert digest(manifest) == read(receipt)["inputs_sha256"], manifest
    frozen = {Path(row["path"]): row["sha256"] for row in read(manifest)["files"]}
    for path, sha in expected.items():
        assert frozen.get(path) == sha, (directory.name, path)
    return receipt, manifest


def render_sources(inputs):
    binary, narration, _, _, timeline, _, _, paths = inputs
    data = {path for path in paths
            if path.is_relative_to(narration.parent) or path.is_relative_to(timeline.parent)}
    return {path.resolve(): digest(path) for path in {binary, FFMPEG, *data}}


def finished_chapters(film, inputs, expected):
    _, _, _, frames, timeline, stems, score, paths = inputs
    artifacts = {}
    for key, count in frames.items():
        directory = WORK / stage_job(chapter_job(film, key))
        receipt, manifest = bind_guard_inputs(directory, expected)
        chapter_record = directory / "chapter-render01.json"
        record = read(chapter_record)
        files = shared["render_files"](directory / "pictures", key)
        assert record["chapter"] == key and record["media"]["frames"] == count
        assert record["render_options"] == list(RENDER_OPTIONS), key
        assert record["sha256"] == {ext: digest(path) for ext, path in files.items()}, key
        for ext in ("timeline.tsv", "cues.tsv"):
            assert files[ext].read_bytes() == (timeline / f"{key}.{ext}").read_bytes(), (key, ext)
        shared["verify_cue_log"](files["cuelog.tsv"], timeline / f"{key}.cuelog.tsv", stems, score[key]["parts"])
        paths.update((receipt, manifest, chapter_record, *files.values()))
        artifacts[key] = files
    return artifacts


def master(job, inputs, artifacts):
    binary, narration, lines, frames, _, _, _, _ = inputs
    out = WORK / job / "delivery"
    out.mkdir()
    for key, files in artifacts.items():
        for extension, source in files.items():
            if extension == "captured.wav":
                target = out / "soundtrack" / key / "soundtrack_0.wav"
                target.parent.mkdir(parents=True, exist_ok=True)
            else:
                target = out / f"{key}.{extension}"
            os.link(source, target)
    subprocess.run([binary, "master", "--narration", narration, "--out", out], check=True, timeout=7200)
    subprocess.run([binary, "captions", "all", "--narration", narration, "--out", out],
                   check=True, timeout=900)
    count = sum(frames.values())
    captions = prepared["caption_check"](out, binary.name, lines, count)
    native, tagged = out / f"{binary.name}_master.mp4", out / f"{job}-4k60.mp4"
    media, clock = shared["remux_delivery"](native, tagged, count)
    log = out / "decode-and-loudness01.log"
    with log.open("x", encoding="utf-8") as handle:
        subprocess.run([
            FFMPEG, "-nostdin", "-hide_banner", "-v", "info", "-nostats", "-xerror", "-i", tagged,
            "-map", "0:v:0", "-map", "0:a:0", "-af", "ebur128=peak=true:framelog=verbose",
            "-f", "null", "-",
        ], check=True, timeout=7200, stderr=handle)
    summary = log.read_text(encoding="utf-8").rsplit("Summary:", 1)[1]
    integrated, = re.findall(r"I:\s*(-?\d+\.\d+)\s*LUFS", summary)
    peak, = re.findall(r"Peak:\s*(-?\d+\.\d+)\s*dBFS", summary)
    write(out / "delivery01.json", {
        "media": media, "chapters": frames, "captions": captions,
        "sha256": {path.name: digest(path) for path in (native, tagged, log)},
        "integrated_lufs": float(integrated), "true_peak_dbtp": float(peak),
        "full_av_decode": True, "remux_clock": clock,
        "final_picture_review": False, "continuous_listening": False,
        "fine_sync": False, "independent_av_review": False,
    })


def finish_film(film, package, clock):
    audio["successful_receipt"](package, "asr")
    if clock.parent == WORK:
        clock = WORK / stage_job(clock.name)
    if not clock.exists():
        assert clock.parent == WORK, "new guarded clocks must be in the production work folder"
        subprocess.run([sys.executable, "-I", "-B", CLOCK, film, "launch",
                        "--package", package, "--job", clock.name], check=True)

    inputs = score_inputs(film, package, clock)
    frames, paths = inputs[3], inputs[-1]
    entry = Path(__file__).resolve()
    command = [sys.executable, "-I", "-B", entry, film]
    sources = ["--package", package, "--clock", clock]
    expected, original_paths = render_sources(inputs), set(paths)
    for key in frames:
        directory = WORK / stage_job(chapter_job(film, key))
        if not directory.exists():
            subprocess.run([*command, "launch-render", *sources, "--chapter", key], check=True)
        bind_guard_inputs(directory, expected)

    finished_chapters(film, inputs, expected)
    job = WORK / stage_job(f"{film}-native-master01")
    if not job.exists():
        subprocess.run([*command, "launch-master", *sources], check=True)
    expected.update({path.resolve(): digest(path) for path in paths - original_paths})
    bind_guard_inputs(job, expected)
    output = job / "delivery"
    report = read(output / "delivery01.json")
    assert report["full_av_decode"] is True
    assert list(report["chapters"].items()) == list(frames.items())
    for name, sha in (report["sha256"] | report["captions"]["sha256"]).items():
        assert digest(output / name) == sha, name
    print(f"{film}: guarded master ready for final review at {output}")
    subprocess.run([sys.executable, "-I", "-B", HERE / "slate-master-audio01.py", film], check=True)
    subprocess.run([sys.executable, "-I", "-B", HERE / "slate-master-pictures01.py", film], check=True)
    subprocess.run([sys.executable, "-I", "-B", HERE / "slate-publication-package01.py", film], check=True)


def main():
    if not __debug__:
        raise SystemExit("Run without Python optimization so production checks remain active.")
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("film", choices=tuple(audio["registry"]["FILMS"]))
    parser.add_argument("mode", choices=("finish", "launch-render", "render", "launch-master", "master"))
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--clock", type=Path, required=True)
    parser.add_argument("--chapter")
    parser.add_argument("--job", help=argparse.SUPPRESS)
    args = parser.parse_args()
    package, clock = args.package.resolve(), args.clock.resolve()
    if args.mode == "finish":
        finish_film(args.film, package, clock)
        return
    inputs = score_inputs(args.film, package, clock)
    _, _, _, frames, _, _, _, paths = inputs
    rendering = args.mode.endswith("render")
    if rendering:
        assert args.chapter in frames, "use the exact native chapter key"
        base = chapter_job(args.film, args.chapter)
    else:
        base = f"{args.film}-native-master01"
        artifacts = finished_chapters(args.film, inputs, render_sources(inputs))
    if args.mode.startswith("launch-"):
        job = stage_job(base)
        if (WORK / job).exists():
            raise SystemExit(f"{job} already completed. Use the finished artifact.")
        command = [sys.executable, "-I", "-B", Path(__file__).resolve(), args.film,
                   args.mode.removeprefix("launch-"), "--package", package, "--clock", clock,
                   "--job", job]
        if rendering:
            command.extend(("--chapter", args.chapter))
        audio["voice"]["guard"](job, "native", paths, command)
        return
    job = args.job
    assert job and re.fullmatch(re.escape(base) + r"(?:-attempt[0-9]{2,})?", job), job
    active = read(WORK / job / "resource-receipt.json")
    assert active["admitted"] and active["phase"] == "native" and not active.get("finished")
    if rendering:
        render_chapter(job, args.chapter, inputs)
    else:
        master(job, inputs, artifacts)


if __name__ == "__main__":
    main()
