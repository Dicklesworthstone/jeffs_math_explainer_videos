"""How quiet must a tonal cue be during speech? Measures the narration's loudest
400 ms per sentence, then the SII-lite loss of a soft modal 'marimba' cue
(partials 1 / 3.93 / 9.2, 4 ms raised-cosine attack, panned off-centre) at
several offsets below that speech level, raw or low-passed at 2.2 kHz.

Usage: python3 -I -B cue.py NARRATION_DIR [--first N]   (default N=20 files; ~1 min)
Reproduces memo: speech loudest-400 ms median -20.1 LUFS; a cue at -14 LU costs
0.03-0.05 SII, at -20 LU 0.010-0.016 (basis of the -18 to -20 LU rule).
"""
import glob
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np  # noqa: E402
from scipy import signal  # noqa: E402
from lufs import momentary_max, read_wav_48k_mono  # noqa: E402
import sii  # noqa: E402

R = 48000


def marimba(f0, dur=1.5, partials=((1, 1, 0.9), (3.93, 0.35, 0.25), (9.2, 0.12, 0.08)), lp=None):
    t = np.arange(int(dur * R)) / R
    y = sum(a * np.exp(-t / d) * np.sin(2 * np.pi * f0 * m * t + 0.3 * k) for k, (m, a, d) in enumerate(partials))
    att = np.minimum(t / 0.004, 1); att = 0.5 - 0.5 * np.cos(np.pi * att); y = y * att
    if lp:
        b, a = signal.butter(2, lp / (R / 2)); y = signal.lfilter(b, a, y)
    return y


def main(narr_dir, first):
    fs = sorted(glob.glob(os.path.join(narr_dir, '*.wav')))[:first]
    if not fs:
        sys.exit(f"no .wav files in {narr_dir}")
    mm = np.median([momentary_max(read_wav_48k_mono(f)) for f in fs])
    print('speech loudest-400 ms, median over %d sentences: %.1f LUFS' % (len(fs), mm))
    S, _ = sii.speech_bands(narr_dir)
    for off in [14, 18, 20, 22]:
        for name, y in [('D5 LP2.2k', marimba(587.33, lp=2200)), ('D4 LP2.2k', marimba(293.66, lp=2200)), ('D5 raw', marimba(587.33))]:
            st = np.stack([y * 0.8, y * 0.6], 1)
            st *= 10 ** ((mm - off - momentary_max(st)) / 20)
            w = st[:int(0.4 * R)]
            print('cue at -%d LU  %-10s SII loss (first 400 ms) %.4f' % (off, name, sii.sii_loss(S, sii.bands(np.concatenate([w] * 5)))))


if __name__ == '__main__':
    args = sys.argv[1:]
    if not args or args[0].startswith('-'):
        sys.exit(__doc__)
    n = 20
    if '--first' in args:
        n = int(args[args.index('--first') + 1])
    main(args[0], n)
