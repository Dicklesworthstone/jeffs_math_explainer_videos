"""Narrator voice survey: file formats, crude autocorrelation f0 percentiles,
long-term average speech spectrum (third-octave, relative to the loudest band),
and leading/trailing silence per sentence file.

Usage: python3 -I -B voice.py NARRATION_DIR
Reproduces memo: LTASS peak at 250-630 Hz (robert voice). The f0 here is a
crude autocorrelation estimate with octave errors at the top; use yin2.py for f0.
"""
import glob
import os
import sys

import numpy as np
from scipy import signal
from scipy.io import wavfile

CENTS = [100, 125, 160, 200, 250, 315, 400, 500, 630, 800, 1000, 1250, 1600, 2000,
         2500, 3150, 4000, 5000, 6300, 8000, 10000]


def main(d):
    files = sorted(glob.glob(os.path.join(d, '*.wav')))
    if not files:
        sys.exit(f"no .wav files in {d}")
    rates = set(); allx = []; durs = []; f0s = []
    for f in files:
        sr, x = wavfile.read(f)
        rates.add((sr, str(x.dtype), x.shape[1] if x.ndim > 1 else 1))
        x = x.astype(np.float64)
        if np.abs(x).max() > 2:
            x /= 32768.0
        if x.ndim > 1:
            x = x.mean(1)
        allx.append(x); durs.append(len(x) / sr)
        fl, hop = int(0.04 * sr), int(0.01 * sr)
        for i in range(0, len(x) - fl, hop):
            fr = x[i:i + fl] * np.hanning(fl)
            if np.sqrt(np.mean(fr ** 2)) < 0.02:
                continue
            ac = np.correlate(fr, fr, 'full')[fl - 1:]
            lo, hi = int(sr / 300), int(sr / 70)
            k = lo + np.argmax(ac[lo:hi])
            if ac[k] / ac[0] > 0.5:
                f0s.append(sr / k)
    print('formats', rates, 'files', len(files), 'total s %.1f' % sum(durs), 'mean dur %.2f' % np.mean(durs))
    print('crude f0 pct 5/25/50/75/95:', np.percentile(np.array(f0s), [5, 25, 50, 75, 95]).round(1), 'n', len(f0s))
    x = np.concatenate(allx); sr = list(rates)[0][0]
    f, P = signal.welch(x, sr, nperseg=4096)
    lv = []
    for c in CENTS:
        m = (f >= c / 2 ** (1 / 6)) & (f < c * 2 ** (1 / 6))
        lv.append(10 * np.log10(np.sum(P[m]) * (f[1] - f[0]) + 1e-20))
    lv = np.array(lv); lv -= lv.max()
    print('LTASS third-octave, dB re loudest band:')
    for c, l in zip(CENTS, lv):
        print(f'{c:6d} {l:6.1f}')
    lead = []; trail = []
    for x in allx:
        idx = np.nonzero(np.abs(x) > 0.01)[0]
        lead.append(idx[0] / sr); trail.append((len(x) - idx[-1]) / sr)
    print('leading silence median %.3f s, trailing silence median %.3f s' % (np.median(lead), np.median(trail)))


if __name__ == '__main__':
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
