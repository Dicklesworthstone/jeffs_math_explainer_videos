"""Scratch, read-only: 10 ms RMS energy (dBFS) around one recognizer word segment of a 48 kHz mono PCM16 WAV.

usage: s9_energy_probe01.py WAV START_S END_S
Reads samples with the wave module; writes nothing. Not listening.
"""
import array
import math
import sys
import wave

path, a, b = sys.argv[1], float(sys.argv[2]), float(sys.argv[3])
with wave.open(path, "rb") as w:
    assert (w.getnchannels(), w.getsampwidth()) == (1, 2)
    rate, n = w.getframerate(), w.getnframes()
    pcm = array.array("h", w.readframes(n))
hop = rate // 100


def db(i):
    seg = pcm[i * hop:(i + 1) * hop]
    if not seg:
        return -120.0
    rms = math.sqrt(sum(x * x for x in seg) / len(seg))
    return 20 * math.log10(rms / 32768) if rms > 0 else -120.0


frames = [db(i) for i in range(n // hop)]
s = sorted(frames)
print(f"{path.split('/')[-1]} rate {rate} frames10ms {len(frames)} p05 {s[len(s) // 20]:.1f} p10 {s[len(s) // 10]:.1f} "
      f"p50 {s[len(s) // 2]:.1f} p90 {s[9 * len(s) // 10]:.1f} max {s[-1]:.1f}")
lo, hi = max(0, int(a * 100) - 10), min(len(frames), int(b * 100) + 10)
print(" ".join(f"{(k / 100):.2f}:{frames[k]:.0f}" for k in range(lo, hi)))
