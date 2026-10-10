"""Per-sentence narration loudness and true peak.

Usage: python3 -I -B nar.py NARRATION_DIR [NARRATION_DIR ...]
Reproduces memo: narration_v5 = -23.3 LUFS median per sentence, max true peak -7.5 dBTP.
"""
import glob
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np  # noqa: E402
from lufs import lufs, true_peak, read_wav_48k_mono  # noqa: E402


def main(dirs):
    for d in dirs:
        fs = sorted(glob.glob(os.path.join(d, '*.wav')))
        if not fs:
            sys.exit(f"no .wav files in {d}")
        L, T = [], []
        for f in fs:
            x = read_wav_48k_mono(f)
            L.append(lufs(x))
            T.append(true_peak(x))
        print('%s: %d files; LUFS median %.1f range %.1f..%.1f; TP max %.1f median %.1f dBTP'
              % (os.path.basename(os.path.normpath(d)), len(fs), np.median(L), min(L), max(L), max(T), np.median(T)))


if __name__ == '__main__':
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    main(sys.argv[1:])
