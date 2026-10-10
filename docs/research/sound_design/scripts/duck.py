"""Simulates a chapter's narration timeline (sentences placed back to back with a
fixed breath, as the explainer's Narrator does) and reports the silent gaps
the ducker would see, and how many speech regions remain after merging gaps.

Usage: python3 -I -B duck.py NARRATION_DIR [--prefix c] [--breath 0.6]   (seconds)
Reproduces memo: with BREATH = 0.6 s the longest gap between sentences is
~0.75 s, so merging gaps < 1.2 s leaves ONE region per chapter -- the music
can only answer in authored holds.
"""
import glob
import os
import sys

import numpy as np
from scipy.io import wavfile

HOP = 0.01


def regions(mask, fill):
    m = mask.copy()
    idx = np.nonzero(m)[0]
    for a, b in zip(idx[:-1], idx[1:]):
        if 1 < b - a <= fill / HOP:
            m[a:b] = True
    d = np.diff(np.concatenate([[0], m.astype(int), [0]]))
    return list(zip(np.nonzero(d == 1)[0] * HOP, np.nonzero(d == -1)[0] * HOP))


def main(narr_dir, prefix, breath):
    fs = sorted(glob.glob(os.path.join(narr_dir, prefix + '*.wav')))
    if not fs:
        sys.exit(f"no {prefix}*.wav files in {narr_dir}")
    t = 0.0; act = []
    for f in fs:
        sr, x = wavfile.read(f)
        x = x.astype(float) / (32768 if x.dtype == np.int16 else 1)
        if x.ndim > 1:
            x = x.mean(1)
        n = int(len(x) / sr / HOP); fr = x[:n * int(sr * HOP)].reshape(n, -1)
        rms = 20 * np.log10(np.sqrt((fr ** 2).mean(1)) + 1e-9)
        act += list(rms > -23 - 25)  # active: within 25 dB of the -23 LUFS line loudness
        t += len(x) / sr
        act += [False] * int(breath / HOP); t += breath
    act = np.array(act)
    d = np.diff(np.concatenate([[0], act.astype(int), [0]]))
    s = np.nonzero(d == 1)[0]; e = np.nonzero(d == -1)[0]
    g = (s[1:] - e[:-1]) * HOP
    print('lines %s*: %d sentences, %.1f s, speech-active fraction %.2f' % (prefix, len(fs), t, act.mean()))
    print('silent gaps: n=%d, >=0.15 s %d, >=0.7 s %d, >=1.0 s %d, >=1.5 s %d; max %.2f s'
          % (len(g), (g >= .15).sum(), (g >= .7).sum(), (g >= 1.0).sum(), (g >= 1.5).sum(), g.max()))
    for fill in [0.7, 1.2, 1.5]:
        print('merge gaps < %.1f s -> %d speech region(s)' % (fill, len(regions(act, fill))))


if __name__ == '__main__':
    args = sys.argv[1:]
    if not args or args[0].startswith('-'):
        sys.exit(__doc__)
    prefix = args[args.index('--prefix') + 1] if '--prefix' in args else 'c'
    breath = float(args[args.index('--breath') + 1]) if '--breath' in args else 0.6
    main(args[0], prefix, breath)
