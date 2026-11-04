"""Where the last sound of a sentence-final word stops, from take01's samples. Reads only.

python3 -I -B speech_end.py   (from the repo root; prints only)

Recognition gives a sentence-final word an end time that runs up to the next word, so it cannot say when the word
actually finishes. This looks for the longest run of quiet frames between the word's start and the next word's
recognised start, using frame energy in 10 ms frames, quiet meaning 30 dB under the cue's 95th percentile frame.

This is signal energy, not hearing. It can place the quiet; it cannot say how a phrase sounds. The control: the end of
each quiet run should land near the next word's recognised start.
"""
import array
import math
import sys
import wave
from pathlib import Path

LAB = Path("narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103")
FRAME = 0.010
QUIET_DB = 30.0
# (cue, final word, its recognised start on the large model, next word, next word's recognised start on the large model)
SITES = [
    ("13_3", "squared.", 9.10, "Three", 10.42),
    ("13_3", "cubed.", 11.98, "Four", 13.12),
    ("13_3", "position.", 15.46, "That", 16.62),
    ("13_7", "turn.", 9.16, "Pairs", 10.22),
    ("13_8", "move.", 9.24, "A", 10.28),
    ("13_8", "turns.", 12.78, "A", 13.82),
]


def frames(path):
    with wave.open(str(path), "rb") as w:
        rate, width, channels, count = w.getframerate(), w.getsampwidth(), w.getnchannels(), w.getnframes()
        raw = w.readframes(count)
    if width != 2:
        raise SystemExit(f"{path}: {width}-byte samples; this script reads 16-bit only")
    samples = array.array("h")
    samples.frombytes(raw)
    if sys.byteorder == "big":
        samples.byteswap()
    if channels > 1:
        samples = samples[::channels]
    step = int(rate * FRAME)
    out = []
    for i in range(0, len(samples) - step, step):
        chunk = samples[i : i + step]
        out.append(math.sqrt(sum(v * v for v in chunk) / step))
    return out, rate, channels, count / rate


def main():
    for cue, word, start, next_word, next_start in SITES:
        path = LAB / f"{cue}-take01.wav"
        rms, rate, channels, seconds = frames(path)
        loud = sorted(rms)[int(len(rms) * 0.95)]
        limit = loud * 10 ** (-QUIET_DB / 20)
        a, b = int(start / FRAME), int(next_start / FRAME) + 8
        best = (0, a)
        run = 0
        for i in range(a, min(b, len(rms))):
            run = run + 1 if rms[i] < limit else 0
            if run > best[0]:
                best = (run, i)
        length, stop = best
        begin = (stop - length + 1) * FRAME
        end = (stop + 1) * FRAME
        print(f"{cue} {word:9s} recognised from {start:6.2f}; quiet {begin:6.2f} to {end:6.2f} ({length * FRAME:.2f} s); "
              f"{next_word} recognised at {next_start:.2f}, quiet ends {end - next_start:+.2f} from it  "
              f"[{rate} Hz, {channels} ch, {seconds:.2f} s]")


if __name__ == "__main__":
    main()
