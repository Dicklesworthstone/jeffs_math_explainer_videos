"""Build tiny real-encoder inputs plus independent decoded sample fixtures.

Test-only ffmpeg use. The engine neither imports this file nor shells out to
Python. Fixture filenames intentionally have no media extension.
"""
from __future__ import annotations

import math
import os
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import wave


def make(root: Path) -> None:
    root.mkdir(parents=True, exist_ok=True)
    ffmpeg = os.environ.get("FMN_AUDIO_FFMPEG") or shutil.which("ffmpeg")
    if not ffmpeg:
        raise RuntimeError("real audio fixtures require ffmpeg")
    rows = []
    for channels, rate, source_name in [(2, 48000, "source.wav"), (1, 32000, "mono.wav")]:
        # Non-s16 values exercise the full native 24-bit input precision.
        values = [round((0.4 * math.sin(2 * math.pi * (700 + c * 500) * t / rate) + 1 / 65536) * (1 << 23))
                  for t in range(rate // 8) for c in range(channels)]
        with wave.open(str(root / source_name), "wb") as output:
            output.setnchannels(channels)
            output.setsampwidth(3)
            output.setframerate(rate)
            output.writeframes(b"".join(v.to_bytes(3, "little", signed=True) for v in values))
    for name, codec, muxer, source, channels, rate in [
        ("flac", "flac", "flac", "source.wav", 2, 48000),
        ("mp3", "libmp3lame", "mp3", "source.wav", 2, 48000),
        ("aac", "aac", "adts", "source.wav", 2, 48000),
        ("m4a", "aac", "ipod", "source.wav", 2, 48000),
        ("vorbis", "libvorbis", "ogg", "source.wav", 2, 48000),
        ("opus", "libopus", "ogg", "source.wav", 2, 48000),
        ("aiff", "pcm_s24be", "aiff", "source.wav", 2, 48000),
        ("mono", "flac", "flac", "mono.wav", 1, 32000),
    ]:
        target = root / f"{name}.asset"
        subprocess.run([ffmpeg, "-hide_banner", "-loglevel", "error", "-nostdin", "-y",
                        "-i", str(root / source), "-c:a", codec, "-f", muxer, str(target)],
                       check=True, timeout=30, capture_output=True)
        reference = subprocess.run([ffmpeg, "-hide_banner", "-loglevel", "error", "-nostdin",
                                    "-threads", "1", "-i", str(target), "-map", "0:a:0",
                                    "-c:a", "pcm_f32le", "-f", "f32le", "pipe:1"],
                                   check=True, timeout=30, capture_output=True).stdout
        assert reference and len(reference) % (channels * 4) == 0
        (root / f"{name}.f32").write_bytes(reference)
        rows.append(f"{name}.asset\t{name}.f32\t{channels}\t{rate}")
    (root / "manifest.tsv").write_text("\n".join(rows) + "\n", encoding="utf-8")
    print(f"generated {len(rows)} real compressed-audio/reference fixtures")


if __name__ == "__main__":
    make(Path(sys.argv[1]))
