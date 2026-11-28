"""Compare the master AAC decode with the native mixed soundtrack on the same clock."""

import argparse
import json
import math
from pathlib import Path
import runpy
import subprocess
import sys

import numpy as np


HERE = Path(__file__).resolve().parent
packaging = runpy.run_path(str(HERE / "slate-publication-package01.py"))
sha, read = (packaging[name] for name in ("sha", "read_json"))
RATE = 48_000
BYTES_PER_SAMPLE = 8  # Two interleaved 32-bit float channels.
QUIET_POWER = 10 ** (-65 / 10)


def audio_clock(path):
    data = subprocess.check_output([
        "ffprobe", "-v", "error", "-select_streams", "a:0", "-show_entries",
        "stream=codec_name,sample_rate,channels,time_base,start_pts,duration_ts",
        "-of", "json", path,
    ], timeout=60)
    stream, = json.loads(data)["streams"]
    assert (stream["sample_rate"], stream["channels"], stream["time_base"]) == (
        str(RATE), 2, f"1/{RATE}",
    ), path
    assert stream.get("start_pts", 0) == 0, path
    return stream


def decoder(path):
    return subprocess.Popen([
        "ffmpeg", "-nostdin", "-v", "error", "-xerror", "-threads", "1",
        "-i", path, "-map", "0:a:0", "-vn", "-sn", "-dn",
        "-c:a", "pcm_f32le", "-f", "f32le", "pipe:1",
    ], stdout=subprocess.PIPE)


def window(encoded, reference):
    encoded = np.frombuffer(encoded, dtype="<f4").reshape(-1, 2)
    reference = np.frombuffer(reference, dtype="<f4").reshape(-1, 2)
    assert encoded.shape == reference.shape
    assert np.isfinite(encoded).all() and np.isfinite(reference).all()
    actual = np.mean(np.square(encoded), axis=0, dtype=np.float64)
    expected = np.mean(np.square(reference), axis=0, dtype=np.float64)
    checked = np.maximum(actual, expected) >= QUIET_POWER
    if not np.any(checked):
        return None
    a, b = actual[checked], expected[checked]
    joint = np.mean(encoded[:, checked] * reference[:, checked], axis=0, dtype=np.float64)
    cosine = np.divide(joint, np.sqrt(a * b), out=np.zeros_like(joint), where=(a * b > 0))
    # A silent channel on either side stays visible through zero similarity.
    delta = np.abs(10 * np.log10(np.maximum(a, 1e-30) / np.maximum(b, 1e-30)))
    return {"lowest_similarity": float(np.min(cosine)), "largest_rms_difference_db": float(np.max(delta))}


def compare(encoded, reference, chapters):
    sources = {path: packaging["fingerprint"](path) for path in (encoded, reference)}
    audio, pcm = audio_clock(encoded), audio_clock(reference)
    assert audio["codec_name"] == "aac" and pcm["codec_name"] in ("pcm_f32le", "pcm_s16le")
    total = 800 * sum(frames for _, frames in chapters)
    assert pcm["duration_ts"] == total
    assert abs(audio["duration_ts"] - total) <= 1024
    position = 0
    results = {}
    with decoder(encoded) as actual, decoder(reference) as expected:
        try:
            for key, frames in chapters:
                end = position + 800 * frames
                silent, checked = 0, []
                while position < end:
                    count = min(RATE, end - position)
                    size = count * BYTES_PER_SAMPLE
                    a, b = actual.stdout.read(size), expected.stdout.read(size)
                    assert len(a) == len(b) == size, (key, position, len(a), len(b))
                    metrics = window(a, b)
                    if metrics is None:
                        silent += 1
                    else:
                        checked.append({"start_sample": position, "samples": count, **metrics})
                    position += count
                results[key] = {
                    "speech_or_score_windows": len(checked), "quiet_windows": silent,
                    "below_0_97_similarity": [row for row in checked if row["lowest_similarity"] < 0.97],
                    "above_1_db_rms_difference": [row for row in checked if row["largest_rms_difference_db"] > 1.0],
                    "lowest_similarity": min((row["lowest_similarity"] for row in checked), default=None),
                    "largest_rms_difference_db": max((row["largest_rms_difference_db"] for row in checked), default=None),
                }
            extra = actual.stdout.read(1025 * BYTES_PER_SAMPLE)
            assert len(extra) % BYTES_PER_SAMPLE == 0 and len(extra) <= 1024 * BYTES_PER_SAMPLE
            assert not expected.stdout.read(1)
            assert actual.wait(timeout=60) == expected.wait(timeout=60) == 0
        except BaseException:
            for child in (actual, expected):
                if child.poll() is None:
                    child.terminate()
            raise
    hashes = {"aac_container": sha(encoded), "mixed_pcm": sha(reference)}
    assert all(packaging["fingerprint"](path) == before for path, before in sources.items())
    return {"compared_sample_frames": position, "aac_tail_padding_frames": len(extra) // BYTES_PER_SAMPLE,
            "source_sha256": hashes, "chapters": results}


def build(film):
    job, filename, stem, *_ = packaging["film_inputs"](film)
    out = job / "delivery"
    guard, receipt = job / "resource-receipt.json", out / "delivery01.json"
    state, report = packaging["stable_success"](guard), read(receipt)
    assert state["phase"] == "native" and report["full_av_decode"] is True
    assert report["remux_clock"]["aac_packets_and_clocks_unchanged"] is True
    encoded, reference = out / filename, out / "master" / f"{stem}.master.wav"
    expected_hash = report["sha256"][filename]
    assert sha(encoded) == expected_hash
    chapters = report["chapters"]
    if isinstance(chapters, dict):
        chapters = list(chapters.items())
    assert sum(frames for _, frames in chapters) == report["media"]["frames"]
    path = job / "encoded-audio-readback01.json"
    bindings = {"guard_sha256": sha(guard), "delivery_sha256": sha(receipt)}
    if path.exists():
        previous = read(path)
        assert all(previous[name] == value for name, value in bindings.items())
        assert previous["source_sha256"] == {"aac_container": expected_hash, "mixed_pcm": sha(reference)}
        assert list(previous["chapters"]) == [key for key, _ in chapters]
        assert previous["compared_sample_frames"] == 800 * report["media"]["frames"]
        return path
    result = compare(encoded, reference, chapters)
    assert result["source_sha256"]["aac_container"] == expected_hash
    with path.open("x") as handle:
        json.dump({**bindings, **result,
                   "scope": "Technical comparison of the encoded AAC and native PCM at zero sample lag. "
                            "Continuous listening and perceptual sync remain separate checks."},
                  handle, indent=2)
        handle.write("\n")
    return path


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("film", choices=tuple(packaging["FILMS"]))
    film = parser.parse_args().film
    print(build(film), flush=True)
    mode = "verify" if film == "anytime" else "launch"
    audit = subprocess.run([sys.executable, "-I", "-B", HERE / "slate-export-joins01.py", film, mode])
    if audit.returncode:
        print(f"{film}: export join review is incomplete (exit {audit.returncode}).", file=sys.stderr)
