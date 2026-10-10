"""Narrator f0 (YIN on 8 kHz-decimated audio) and pauses inside sentences.

Usage: python3 -I -B yin2.py NARRATION_DIR      (about 1 min for 60 sentences)
Reproduces memo: robert f0 median ~136 Hz, IQR 121-154, 5-95% 107-200 Hz;
in-sentence pauses >=150 ms about 4.1 per sentence, median 0.20 s, p90 0.52 s, max 0.91 s.
"""
import glob
import os
import sys

import numpy as np
from numpy.lib.stride_tricks import sliding_window_view as swv
from scipy.io import wavfile
from scipy.signal import resample_poly


def main(d):
    files = sorted(glob.glob(os.path.join(d, '*.wav')))
    if not files:
        sys.exit(f"no .wav files in {d}")
    f0s = []; gaps = []
    for f in files:
        sr, x = wavfile.read(f)
        x = x.astype(float) / (32768 if x.dtype == np.int16 else 1)
        if x.ndim > 1:
            x = x.mean(1)
        g = np.gcd(8000, sr)
        x = resample_poly(x, 8000 // g, sr // g); sr = 8000
        W = int(0.03 * sr); hop = int(0.01 * sr); tmax = int(sr / 60); tmin = int(sr / 400)
        fr = swv(x, W + tmax)[::hop]
        rms = np.sqrt(np.mean(fr[:, :W] ** 2, 1))
        fr = fr[rms > 0.015]
        if len(fr):
            D = np.stack([np.sum((fr[:, :W] - fr[:, t:t + W]) ** 2, 1) for t in range(1, tmax)], 1)
            cm = D * np.arange(1, tmax) / np.maximum(np.cumsum(D, 1), 1e-12)
            for row in cm:
                c = np.nonzero(row[tmin - 1:] < 0.12)[0]
                if len(c) == 0:
                    continue
                t = tmin - 1 + c[0]
                while t + 1 < len(row) and row[t + 1] < row[t]:
                    t += 1
                f0s.append(sr / (t + 1))
        e = np.sqrt(np.mean(swv(x, W)[::hop] ** 2, 1)); act = e > 0.01
        idx = np.nonzero(act)[0]
        if len(idx) == 0:
            continue
        a = act[idx[0]:idx[-1] + 1]; run = 0
        for v in a:
            if not v:
                run += 1
            else:
                if run * 0.01 >= 0.15:
                    gaps.append(run * 0.01)
                run = 0
    f0 = np.array(f0s)
    print('YIN f0 pct 5/10/25/50/75/90/95:', np.percentile(f0, [5, 10, 25, 50, 75, 90, 95]).round(1), 'frames', len(f0))
    print('in-sentence pauses >=150 ms: n %d, per sentence %.2f, median %.2f s, p90 %.2f s, max %.2f s'
          % (len(gaps), len(gaps) / len(files), np.median(gaps), np.percentile(gaps, 90), max(gaps)))


if __name__ == '__main__':
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
