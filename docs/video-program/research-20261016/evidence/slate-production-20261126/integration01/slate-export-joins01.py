"""Check word timing at every chapter join in a guarded slate export."""

import argparse
import json
from pathlib import Path
import runpy
import shutil
import subprocess
import sys
import wave


ENTRY = Path(__file__).resolve()
PACKAGE = ENTRY.with_name("slate-publication-package01.py")
AUTHORING = ENTRY.with_name("companion-recording-plan01.py")
GUARD_WRAPPER = ENTRY.with_name("slate-resource200.py")
package = runpy.run_path(str(PACKAGE))
WORK, sha, read = (package[key] for key in ("WORK", "sha", "read_json"))
JOIN_AUDIT = Path("/Users/jemanuel/.codex/skills/math-explainer-videos-end-to-end-with-franken-manim-frankentts/scripts/join_audit.py")
MODEL = Path("/Users/jemanuel/models/whisper/ggml-small.en.bin")
FW, FFMPEG = (Path(shutil.which(name)).resolve() for name in ("fw", "ffmpeg"))


def export_sources(film):
    job, filename, _, _, _ = package["film_inputs"](film)
    receipt_path = job / "resource-receipt.json"
    state = package["stable_success"](receipt_path)
    assert state["phase"] == "native"
    delivery = job / "delivery"
    report_path = delivery / "delivery01.json"
    report = read(report_path)
    movie = delivery / filename
    frames = report["media"]["frames"]
    clock = report["remux_clock"]
    assert report["full_av_decode"] is True
    assert clock["video_packets"] == frames and clock["video_packet_payloads_unchanged"] is True
    assert clock["aac_packets_and_clocks_unchanged"] is True
    assert sha(movie) == report["sha256"][filename]
    chapters = report["chapters"]
    pairs = list(chapters.items()) if isinstance(chapters, dict) else chapters
    manifest = WORK / "navier-production-20261120" / f"{job.name}-inputs.json"
    assert sha(manifest) == state["inputs_sha256"]
    project_inputs = {Path(row["path"]): row["sha256"] for row in read(manifest)["files"]}
    if film == "anytime":
        script = ENTRY.with_name("script03.tsv")
        keys = [key for key, _ in pairs]
        extra = set()
    else:
        authoring = runpy.run_path(str(AUTHORING))
        plan_path = authoring["recording_plan_path"](film)
        entry, = [row for row in read(plan_path)["films"] if row["film"] == film]
        script = package["REPO"] / entry["script"]
        assert sha(script) == entry["script_sha256"]
        keys = entry["chapters"]
        extra = {AUTHORING, plan_path}
    assert project_inputs[script] == sha(script)
    package["verified_starts"](delivery, chapters, keys, frames)
    paths = {movie, script, report_path, receipt_path, manifest, *extra}
    paths.update(delivery / f"{key}.timeline.tsv" for key in keys)
    binding = {
        "master_sha256": sha(movie), "delivery_sha256": sha(report_path),
        "master_guard_sha256": sha(receipt_path), "script_sha256": sha(script),
        "ffmpeg_sha256": sha(FFMPEG), "frankenwhisper_sha256": sha(FW),
        "small_en_sha256": sha(MODEL), "join_audit_sha256": sha(JOIN_AUDIT),
        "frames": frames, "chapters": [list(row) for row in pairs],
    }
    return movie, script, binding, paths


def recognize(directory, movie, script, binding):
    receipt = read(directory / "resource-receipt.json")
    assert receipt["phase"] == "asr" and receipt["admitted"] and not receipt.get("finished")
    wav = directory / "export-16k-mono.wav"
    subprocess.run([FFMPEG, "-nostdin", "-v", "error", "-xerror", "-n", "-threads", "1",
                    "-i", movie, "-map", "0:a:0", "-vn", "-ac", "1", "-ar", "16000",
                    "-c:a", "pcm_s16le", wav], check=True, timeout=180)
    with wave.open(str(wav), "rb") as pcm:
        assert (pcm.getnchannels(), pcm.getframerate(), pcm.getsampwidth()) == (1, 16000, 2)
        samples = pcm.getnframes()
    assert abs(samples - binding["frames"] * 16000 / 60) <= 343
    words = directory / "export-words.json"
    with words.open("xb") as output:
        subprocess.run([FW, "transcribe", "--input", wav, "--backend", "whisper-cpp", "--language", "en",
                        "--no-diarize", "--no-persist", "--json", "--model", MODEL,
                        "--max-segment-length", "1", "--split-on-word", "--threads", "2", "--timeout", "1800"],
                       stdout=output, check=True, timeout=1860)
    segments = read(words)["result"]["segments"]
    assert segments and all(0 <= row["start_sec"] <= row["end_sec"] <= samples/16000+0.05 for row in segments)
    audit = directory / "joins01.txt"
    with audit.open("xb") as output:
        subprocess.run([sys.executable, "-I", "-B", JOIN_AUDIT, words, script,
                        *(f"{key}:{count/60:.12f}" for key, count in binding["chapters"])],
                       stdout=output, check=True, timeout=120)
    assert len(audit.read_text().splitlines()) == len(binding["chapters"])
    with (directory / "binding01.json").open("x") as output:
        json.dump({**binding, "sample_rate": 16000, "samples": samples, "asr_segments": len(segments),
                   "derived_wav_sha256": sha(wav), "segments_sha256": sha(words), "join_report_sha256": sha(audit),
                   "scope": "Recognition from the exact joined export. Listening and perceptual sync remain open."},
                  output, indent=2)
        output.write("\n")


def verify(directory, binding):
    assert package["stable_success"](directory / "resource-receipt.json")["phase"] == "asr"
    saved = read(directory / "binding01.json")
    assert all(saved[key] == expected for key, expected in binding.items())
    for key, name in (("derived_wav_sha256", "export-16k-mono.wav"), ("segments_sha256", "export-words.json"),
                      ("join_report_sha256", "joins01.txt")):
        assert saved[key] == sha(directory / name)
    words = read(directory / "export-words.json")["result"]["segments"]
    assert len(words) == saved["asr_segments"] > 0
    audit = directory / "joins01.txt"
    rows = audit.read_text().splitlines()
    word_count = sum(bool(row["text"].strip()) for row in words)
    assert len(rows) == len(binding["chapters"]) and rows[-1].endswith(f"{word_count} words")
    return audit


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("film", choices=tuple(package["FILMS"]))
    parser.add_argument("mode", choices=("launch", "verify", "recognize"))
    parser.add_argument("--job")
    args = parser.parse_args()
    job = args.job or f"{args.film}-native-master-join-asr01"
    if args.film == "anytime":
        assert args.mode == "verify" and args.job is None, "Reuse the checked flagship recognition."
        job = "anytime-valid-master-join-asr01"
    assert job.startswith(f"{args.film}-") and all(character.islower() or character.isdigit() or character == "-" for character in job)
    directory = WORK / job
    movie, script, binding, paths = export_sources(args.film)
    if args.mode == "recognize":
        recognize(directory, movie, script, binding)
    elif args.mode == "verify" or directory.exists():
        print(verify(directory, binding))
    else:
        paths.update((ENTRY, PACKAGE, FW, FFMPEG, JOIN_AUDIT, MODEL))
        command = [sys.executable, "-I", "-B", GUARD_WRAPPER, "--phase", "asr"]
        for path in sorted(paths):
            command.extend(("--input", path))
        command.extend((job, "--", sys.executable, "-I", "-B", ENTRY, args.film, "recognize", "--job", job))
        subprocess.run(command, check=True)
        print(verify(directory, binding))


if __name__ == "__main__":
    main()
