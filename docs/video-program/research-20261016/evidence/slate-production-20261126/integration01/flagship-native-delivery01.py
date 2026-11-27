"""Render and verify the flagship against its complete narration and checked score."""

import argparse
from fractions import Fraction
import json
import os
from pathlib import Path
import re
import runpy
import shutil
import subprocess
import sys


ENTRY = Path(__file__).resolve()
HERE = ENTRY.parent
CLOCK = HERE / "flagship-clock-and-score02.py"
CAPTIONS = HERE / "audit-voiced-captions02.py"
MASTER_CLOCK = HERE / "master-clock-remux01.py"
clock = runpy.run_path(str(CLOCK))
voice = clock["voice"]
WORK, digest, read, write = (voice[key] for key in ("WORK", "digest", "read", "write_json"))
score_work = WORK / clock["JOB"]
BIN = clock["EXECUTABLE"]
MASTER_JOB = "anytime-valid-native-master01"
MASTER_NAME = "probability_native_master.mp4"
DELIVERY_NAME = "probability-native-4k60.mp4"
TABLES = ("timeline.tsv", "cues.tsv", "cuelog.tsv", "layout.tsv")
RENDER_OPTIONS = (
    "--res", "3840x2160", "--fps", "60", "--encoder", "libx264",
    "--crf", "16", "--preset", "veryfast", "--threads", "3", "--lint",
)
FFMPEG, FFPROBE = (Path(shutil.which(name)).resolve() for name in ("ffmpeg", "ffprobe"))


def verify_av(clip, frames, *, delivery=False):
    probe = subprocess.check_output([FFPROBE, "-v", "error", "-show_streams", "-of", "json", clip])
    video, audio = json.loads(probe)["streams"]
    assert video["codec_type"] == "video" and audio["codec_type"] == "audio", clip
    assert (video["width"], video["height"]) == (3840, 2160), clip
    assert video["codec_name"] == "h264" and video["pix_fmt"] == "yuv420p", clip
    assert video["color_space"] == "bt709" and video["color_range"] == "tv", clip
    if delivery:
        assert all(video.get(key) == "bt709" for key in ("color_primaries", "color_transfer")), clip
    assert all(Fraction(video[key]) == 60 for key in ("r_frame_rate", "avg_frame_rate")), clip
    assert int(video["nb_frames"]) == frames, clip
    assert Fraction(video["time_base"]) * int(video["duration_ts"]) == Fraction(frames, 60), clip
    assert audio["codec_name"] == "aac" and audio["sample_rate"] == "48000" and audio["channels"] == 2, clip
    audio_duration = Fraction(audio["time_base"]) * int(audio["duration_ts"])
    assert abs(audio_duration - Fraction(frames, 60)) <= Fraction(1024, 48000) + Fraction(1, 1000), clip
    return {"frames": frames, "audio_seconds": float(audio_duration), "bytes": clip.stat().st_size}


def score_inputs():
    narration, speech, lines, keys, paths = clock["frozen"]()
    guard = score_work / "resource-receipt.json"
    score_path = score_work / "clock-and-score01.json"
    voice["checked_receipt"](guard, 0)
    binding = read(score_path)
    timeline = score_work / "clock60"
    chapters = clock["shared"]["verify_clock"](timeline, keys, narration, speech, lines)
    assert chapters == binding["chapters"] and sum(ch["frames"] for ch in chapters) == binding["frames"]
    stems = score_work / "score/stems"
    paths.update((ENTRY, CLOCK, MASTER_CLOCK, FFMPEG, FFPROBE, guard, score_path))
    frames_by_key = {ch["key"]: ch["frames"] for ch in chapters}
    for key in keys:
        qa, stem = stems / "qa" / f"{key}.json", stems / f"{key}.music.f32.wav"
        record = binding["score"][key]
        assert read(qa)["passed"] is True and digest(qa) == record["qa_sha256"]
        media = clock["shared"]["score_stem_parts"](stem, 800 * frames_by_key[key])
        assert all(record[field] == value for field, value in media.items()), key
        paths.update((qa, *(stems / row["file"] for row in media["parts"])))
        if media["manifest_sha256"] is not None:
            paths.add(stem.with_suffix(".parts.tsv"))
        paths.update(timeline / f"{key}.{ext}" for ext in TABLES[:3])
    return narration, lines, frames_by_key, stems, binding["score"], paths


def render_job(key):
    return f"anytime-valid-native-render01-{key[:2]}"


def verify_cue_log(pictured, expected, stems, parts):
    def data(path):
        return [line.split("\t") for line in path.read_text().splitlines() if line and not line.startswith("#")]
    rows = data(pictured)
    assert all(len(row) == 13 for row in rows), pictured
    paths = [stems / file["file"] for file in parts]
    music = [row for row in rows if Path(row[0]) in paths]
    narration = [row for row in rows if Path(row[0]) not in paths]
    assert narration == data(expected), pictured
    assert len(music) == len(parts), pictured
    for row, path, part in zip(music, paths, parts):
        assert Path(row[0]) == path and row[1:3] == [part["sha256"], str(path.stat().st_size)], pictured
        assert row[4] == "60" and int(row[8], 16) == 0, pictured
        assert row[9:13] == ["48000", "2", str(part["samples"]), str(part["start_sample"])], pictured


def render_files(folder, key):
    files = {ext: folder / f"{key}.{ext}" for ext in (*TABLES, "mp4")}
    directory = folder / "soundtrack" / key
    captures = list(directory.glob("soundtrack_*.wav"))
    assert captures and set(captures) == {directory / f"soundtrack_{n}.wav" for n in range(len(captures))}, key
    # Follow the kit's master, which replays the last capture after a retry.
    files["captured.wav"] = directory / f"soundtrack_{len(captures)-1}.wav"
    return files


def remux_delivery(source, target, frames):
    clock_tool = runpy.run_path(str(MASTER_CLOCK))
    clock = clock_tool["remux"](FFMPEG, FFPROBE, source, target, frames)
    return verify_av(target, frames, delivery=True), clock


def render_chapter(key, inputs, *, executable=BIN, picture_repair=None):
    narration, _, chapters, stems, score, _ = inputs
    frames = chapters[key]
    out = WORK / render_job(key)
    pictures = out / "pictures"
    assert not pictures.exists()
    subprocess.run([
        executable, "render", key, *RENDER_OPTIONS, "--narration", narration, "--score", stems, "--out", pictures,
    ], check=True, timeout=14400)
    files = render_files(pictures, key)
    score_clock = score_work / "clock60"
    for ext in ("timeline.tsv", "cues.tsv"):
        assert files[ext].read_bytes() == (score_clock / f"{key}.{ext}").read_bytes(), (key, ext)
    verify_cue_log(files["cuelog.tsv"], score_clock / f"{key}.cuelog.tsv", stems, score[key]["parts"])
    av = verify_av(files["mp4"], frames)
    record = {
        "chapter": key, "media": av, "flags": files["layout.tsv"].read_text().splitlines(),
        "sha256": {ext: digest(path) for ext, path in files.items()},
    }
    if picture_repair is not None:
        record["picture_repair"] = picture_repair
    write(out / "chapter-render01.json", record)


def master_inputs(inputs):
    _, _, chapters, _, _, paths = inputs
    artifacts = {}
    for key, frames in chapters.items():
        source = WORK / render_job(key)
        guard, record_path = source / "resource-receipt.json", source / "chapter-render01.json"
        voice["checked_receipt"](guard, 0)
        record = read(record_path)
        files = render_files(source / "pictures", key)
        assert record["chapter"] == key and record["media"]["frames"] == frames
        assert {ext: digest(path) for ext, path in files.items()} == record["sha256"], key
        paths.update((guard, record_path, *files.values()))
        artifacts[key] = files
    paths.add(CAPTIONS)
    return artifacts


def master(inputs, artifacts):
    narration, lines, chapters, _, _, _ = inputs
    out = WORK / MASTER_JOB / "delivery"
    out.mkdir()
    for key, files in artifacts.items():
        for ext, source in files.items():
            if ext == "captured.wav":
                target = out / "soundtrack" / key / "soundtrack_0.wav"
                target.parent.mkdir(parents=True, exist_ok=True)
            else:
                target = out / f"{key}.{ext}"
            os.link(source, target)
    subprocess.run([BIN, "master", "--narration", narration, "--out", out], check=True, timeout=7200)
    subprocess.run([BIN, "captions", "all", "--narration", narration, "--out", out], check=True)
    parse = runpy.run_path(str(CAPTIONS))["captions"]
    srt, vtt = (out / f"probability_native.en.{suffix}" for suffix in ("srt", "vtt"))
    english = parse(srt, False)
    assert english and english == parse(vtt, True)
    expected = [word for rows in lines.values() for row in rows for word in row["sentence"].split()]
    assert " ".join(" ".join(text) for _, _, text in english).split() == expected
    frames, last, speeds = sum(chapters.values()), 0, []
    for start, end, text in english:
        assert last <= start < end <= round(Fraction(1000 * frames, 60))
        last = end
        speeds.append(len(" ".join(text)) * 1000 / (end - start))
    native, video = out / MASTER_NAME, out / DELIVERY_NAME
    media, clock = remux_delivery(native, video, frames)
    decode_log = out / "decode-and-loudness01.log"
    with decode_log.open("x") as handle:
        subprocess.run([
            FFMPEG, "-nostdin", "-hide_banner", "-v", "info", "-nostats", "-xerror", "-i", video,
            "-map", "0:v:0", "-map", "0:a:0", "-af", "ebur128=peak=true:framelog=verbose",
            "-f", "null", "-",
        ], check=True, timeout=7200, stderr=handle)
    summary = decode_log.read_text().rsplit("Summary:", 1)[1]
    integrated, = re.findall(r"I:\s*(-?\d+\.\d+)\s*LUFS", summary)
    true_peak, = re.findall(r"Peak:\s*(-?\d+\.\d+)\s*dBFS", summary)
    write(out / "delivery01.json", {
        "media": media, "chapters": list(chapters.items()), "english_caption_blocks": len(english),
        "characters_per_second_max": max(speeds), "blocks_above_20_cps": sum(x > 20 for x in speeds),
        "sha256": {path.name: digest(path) for path in (native, video, srt, vtt)},
        "encoded_integrated_lufs": float(integrated), "encoded_true_peak_dbtp": float(true_peak),
        "nominal_loudness_pass": -17.0 <= float(integrated) <= -15.0,
        "true_peak_pass": float(true_peak) <= -1.0, "decode_log_sha256": digest(decode_log),
        "remux_clock": clock,
        "full_av_decode": True, "continuous_listening": False, "fine_sync": False,
        "independent_av_review": False, "final_picture_review": False,
    })


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="phase", required=True)
    for command in ("launch-render", "render"):
        commands.add_parser(command).add_argument("chapter")
    for command in ("launch-master", "master"):
        commands.add_parser(command)
    args = parser.parse_args()
    inputs = score_inputs()
    _, _, chapters, _, _, paths = inputs
    if args.phase.endswith("render"):
        if args.chapter not in chapters:
            parser.error("use an exact native chapter key")
        if args.phase == "launch-render":
            voice["guard"](render_job(args.chapter), "native", paths,
                           [sys.executable, "-I", "-B", ENTRY, "render", args.chapter])
        else:
            render_chapter(args.chapter, inputs)
    else:
        artifacts = master_inputs(inputs)
        if args.phase == "launch-master":
            voice["guard"](MASTER_JOB, "native", paths, [sys.executable, "-I", "-B", ENTRY, "master"])
        else:
            master(inputs, artifacts)


if __name__ == "__main__":
    main()
