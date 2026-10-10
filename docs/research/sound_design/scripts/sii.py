"""SII-lite: a simplified ANSI S3.5 energetic-masking estimate (third-octave
bands 160 Hz-8 kHz, band audibility clip((S - N + 15)/30, 0, 1), ANSI
average-speech band-importance weights). Speech band levels come from the
narration's active frames; returns the SII *loss* a masker causes. No spread
of masking, hearing threshold or binaural effects -- a QA heuristic, not ANSI-compliant.

Usage: python3 -I -B sii.py NARRATION_DIR      (~1 min)
Library: bands(), speech_bands(), sii_loss() are imported by cue.py and newpad.py.
Reproduces memo: current pad SII loss ~0.013 and only ~5 dB below the voice
in the 160/200 Hz bands; a chime's first 400 ms costs 0.11-0.165.
"""
import glob
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np  # noqa: E402
from scipy import signal  # noqa: E402
from lufs import lufs, read_wav_48k_mono  # noqa: E402

C = [160, 200, 250, 315, 400, 500, 630, 800, 1000, 1250, 1600, 2000, 2500, 3150, 4000, 5000, 6300, 8000]
# ANSI S3.5-1997 one-third-octave band importance, average speech.
I = np.array([.0083, .0095, .0150, .0289, .0440, .0578, .0653, .0711, .0818, .0844,
              .0882, .0898, .0868, .0844, .0771, .0527, .0364, .0185])


def bands(x, sr=48000):
    """Third-octave band levels (dB) of a mono or stereo signal (stereo summed /sqrt2)."""
    if x.ndim > 1:
        x = x.sum(1) / np.sqrt(2)
    f, P = signal.welch(x, sr, nperseg=8192)
    out = []
    for c in C:
        m = (f >= c / 2 ** (1 / 6)) & (f < c * 2 ** (1 / 6))
        out.append(10 * np.log10(np.sum(P[m]) * (f[1] - f[0]) + 1e-30))
    return np.array(out)


def speech_bands(narr_dir):
    """Band levels and integrated loudness of the narration's active 20 ms frames (> -50 dBFS RMS)."""
    fs = sorted(glob.glob(os.path.join(narr_dir, '*.wav')))
    if not fs:
        sys.exit(f"no .wav files in {narr_dir}")
    xs = []
    for f in fs:
        x = read_wav_48k_mono(f)
        fr = x[:len(x) // 960 * 960].reshape(-1, 960)
        act = np.sqrt((fr ** 2).mean(1)) > 10 ** (-50 / 20)
        xs.append(fr[act].ravel())
    sp = np.concatenate(xs)
    return bands(sp), lufs(sp)


def sii_loss(S, Nb):
    """SII lost when masker band levels Nb are added under speech band levels S."""
    A0 = np.clip((S + 200 + 15) / 30, 0, 1)
    A = np.clip((S - Nb + 15) / 30, 0, 1)
    return float(np.sum(I * (A0 - A)))


def main(narr_dir):
    from current import pad, chime, g, R
    S, L = speech_bands(narr_dir)
    print('speech (active frames) integrated loudness %.1f LUFS' % L)
    p = pad(30, [146.83, 220.00, 293.66, 369.99])[5 * R:25 * R] * g(-9)
    Pb = bands(p)
    print('band   speech  pad@-9  speech-pad')
    for c, s, n in zip(C, S, Pb):
        print(f'{c:5d} {s:7.1f} {n:7.1f} {s - n:6.1f}')
    print('SII loss, current pad: %.3f' % sii_loss(S, Pb))
    for f in [523.25, 659.25, 880.0]:
        c = chime(f) * g(-10); w = c[:int(0.4 * R)]
        Cb = bands(np.concatenate([w] * 5))
        print('chime %4.0f Hz: SII loss in its first 400 ms %.3f; speech-chime dB, 1.25-4 kHz bands: %s'
              % (f, sii_loss(S, Cb), (S - Cb)[9:15].round(1)))
        print('   chime + pad: %.3f' % sii_loss(S, 10 * np.log10(10 ** (Cb / 10) + 10 ** (Pb / 10))))


if __name__ == '__main__':
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
