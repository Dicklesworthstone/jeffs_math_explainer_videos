"""Measures the prototype pad (proto/ output WAV) against the narration, and
checks PolyBLEP aliasing.

Usage: python3 -I -B newpad.py PAD_WAV NARRATION_DIR     (~1 min)
  PAD_WAV: a WAV written by proto (e.g. `proto pad64.wav 64`); the first
  60 s are analysed.
Reproduces memo: prototype pad at 15 LU under the voice costs ~0.0085 SII,
at ~18 LU ~0.0016; centroid ~400 Hz with energy above 2 kHz ~-35 dB; a saw
at 1.78 kHz has aliased energy -20.3 dB naive vs -55.5 dB with PolyBLEP.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np  # noqa: E402
from scipy import signal  # noqa: E402
from scipy.io import wavfile  # noqa: E402
from lufs import lufs  # noqa: E402
import sii  # noqa: E402


def poly_blep(tp, dt):
    out = np.zeros_like(tp)
    a = tp < dt; u = tp[a] / dt; out[a] = u + u - u * u - 1
    b = tp > 1 - dt; u = (tp[b] - 1) / dt; out[b] = u * u + u + u + 1
    return out


def alias_db(sig, f0, fs):
    n = len(sig)
    F = np.abs(np.fft.rfft(sig * np.hanning(n))) ** 2; fr = np.fft.rfftfreq(n, 1 / fs)
    h = np.zeros_like(F, bool)
    for k in range(1, int(fs / 2 / f0) + 1):
        h |= np.abs(fr - k * f0) < 6
    band = fr < 5000
    return 10 * np.log10(F[band & ~h].sum() / F[band & h].sum())


def main(pad_wav, narr_dir):
    sr, x = wavfile.read(pad_wav)
    x = x.astype(float)
    x = x[:60 * sr]; x /= np.abs(x).max()
    L = lufs(x); print('prototype pad, first %.0f s: %.1f LUFS as rendered (peak-normalised)' % (len(x) / sr, L))
    S, V = sii.speech_bands(narr_dir)
    for target in [V - 15.1, V - 17.8]:
        y = x * 10 ** ((target - L) / 20); B = sii.bands(y)
        print('pad at %.1f LUFS (%.1f LU under voice %.1f): SII loss %.4f' % (target, V - target, V, sii.sii_loss(S, B)))
        print('   speech-pad dB per band 160 Hz..8 kHz:', (S - B).round(0))
    f, P = signal.welch(x.mean(1), sr, nperseg=8192)
    print('prototype pad centroid %.0f Hz; energy above 2 kHz %.1f dB re total'
          % ((f * P).sum() / P.sum(), 10 * np.log10(P[f > 2000].sum() / P.sum())))
    fs = 48000; f0 = 1760.0 * 1.0123
    ph = (f0 * np.arange(fs) / fs) % 1
    naive = 2 * ph - 1
    print('saw f0=%.0f Hz: aliased energy below 5 kHz re harmonics: naive %.1f dB, PolyBLEP %.1f dB'
          % (f0, alias_db(naive, f0, fs), alias_db(naive - poly_blep(ph, f0 / fs), f0, fs)))


if __name__ == '__main__':
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    main(sys.argv[1], sys.argv[2])
