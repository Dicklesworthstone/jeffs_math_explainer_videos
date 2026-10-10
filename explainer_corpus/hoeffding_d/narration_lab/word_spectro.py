#!/usr/bin/env python3
"""Spectrograms of one word across takes, plus a stop-consonant measurement.

    uv run --with numpy python3 word_spectro.py ASR.json WAV_DIR WORD_REGEX OUT_DIR TAG...

For each tag, the word matching WORD_REGEX (from ASR word timings) is cut
(+-80 ms) and drawn as a 0-8 kHz spectrogram PNG via ffmpeg. It also reports
the stop after the fricative: closure length, low-frequency voicing during
the closure (a /d/ keeps a voice bar; a /t/ goes silent) and the burst-to-
voicing lag (a /t/ is followed by aspiration noise before the vowel).
"""
import json
import subprocess
import sys
import wave
from pathlib import Path

import numpy as np

asr = json.loads(Path(sys.argv[1]).read_text())
wav_dir, pattern, out_dir, tags = Path(sys.argv[2]), sys.argv[3], Path(sys.argv[4]), sys.argv[5:]
out_dir.mkdir(parents=True, exist_ok=True)
import re

rx = re.compile(pattern, re.I)
for tag in tags:
    words = asr[tag]["words"]
    hit = next((w for w in words if rx.search(w["w"])), None)
    if not hit:
        print(f"{tag}: no word matching {pattern}")
        continue
    s, e = max(0.0, hit["s"] - 0.08), hit["e"] + 0.08
    wav = wav_dir / f"{tag}.wav"
    png = out_dir / f"{tag}.png"
    subprocess.run(["ffmpeg", "-v", "error", "-y", "-ss", f"{s:.3f}", "-t", f"{e - s:.3f}", "-i", str(wav),
                    "-lavfi", "showspectrumpic=s=600x300:legend=0:scale=log:fscale=lin:stop=8000:color=intensity",
                    str(png)], check=True)
    with wave.open(str(wav)) as w:
        rate = w.getframerate()
        x = np.frombuffer(w.readframes(w.getnframes()), dtype=np.int16).astype(float) / 32768
    seg = x[int(s * rate):int(e * rate)]
    hop, win = int(0.005 * rate), int(0.015 * rate)
    frames = np.lib.stride_tricks.sliding_window_view(seg, win)[::hop]
    spec = np.abs(np.fft.rfft(frames * np.hanning(win), axis=1)) ** 2
    freqs = np.fft.rfftfreq(win, 1 / rate)
    low = 10 * np.log10(spec[:, (freqs > 80) & (freqs < 400)].sum(axis=1) + 1e-12)   # voicing
    high = 10 * np.log10(spec[:, (freqs > 3500) & (freqs < 9000)].sum(axis=1) + 1e-12)  # frication/aspiration
    total = 10 * np.log10(spec.sum(axis=1) + 1e-12)
    print(f"{tag}: word {hit['w']!r} {hit['s']:.2f}-{hit['e']:.2f}s")
    # One compact text strip: per 5 ms frame, F=frication, V=voiced, .=quiet, A=aspirated burst
    strip = []
    for lo, hi, tot in zip(low, high, total):
        if tot < total.max() - 35:
            strip.append(".")
        elif hi > lo + 3:
            strip.append("F")
        elif lo > total.max() - 22:
            strip.append("V")
        else:
            strip.append("~")
    print("   ", "".join(strip))
