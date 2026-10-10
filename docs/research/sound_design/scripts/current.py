"""Replica of the CURRENT (pre-memo) sound design -- the detuned-sine chord pad
and the inharmonic bell chime from the Hoeffding explainer's sound.rs -- plus
its measured loudness, peaks, beating and spectrum.

Usage: python3 -I -B current.py          (no inputs; ~30 s)
Also imported by sii.py and newpad.py for pad()/chime().
Reproduces memo section 1: pad (D chord, -9 dB) -38.3 LUFS, chime (-10 dB)
loudest 400 ms ~ -26 LUFS, partials 1.44-4.75 kHz, 5.2-cent detune, beats
0.29-1.32 Hz, 12 dB AM depth, 4.4 dB breathe swing, centroid ~257 Hz,
no energy above 2 kHz.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import numpy as np  # noqa: E402
from scipy import signal  # noqa: E402
from lufs import lufs, momentary_max, true_peak  # noqa: E402

R = 48000
TAU = 2 * np.pi
CHIME_PARTIALS = [(1.0, 1.0, 1.6), (2.76, 0.45, 0.9), (5.4, 0.2, 0.5), (0.5, 0.25, 2.2)]  # (ratio, amp, decay s)
D_CHORD = [146.83, 220.00, 293.66, 369.99]


def g(db):
    return 10 ** (db / 20)


def pad(seconds, chord):
    """Detuned sine pairs, 0.07 Hz breathe, smoothstep fades -- verbatim from sound.rs."""
    n = int(round(seconds * R)); t = np.arange(n) / R
    l = np.zeros(n); r = np.zeros(n)
    fade = min(2.5, seconds / 3); level = 0.15 / len(chord)
    env = np.clip(np.minimum(np.minimum(t / fade, 1), (seconds - t) / fade), 0, None)
    env = env * env * (3 - 2 * env)
    for k, f in enumerate(chord):
        pan = 0.35 if k % 2 == 0 else 0.65; ph = k * 1.3
        br = 0.8 + 0.2 * np.sin(TAU * 0.07 * t + ph)
        s = np.sin(TAU * f * t) + 0.6 * np.sin(TAU * f * 1.003 * t + ph) + 0.12 * np.sin(TAU * 2 * f * t)
        v = level * env * br * s; l += v * (1 - pan); r += v * pan
    return np.stack([l, r], 1)


def chime(freq):
    """Inharmonic partials, 6 ms linear attack, mono duplicated to L/R -- verbatim from sound.rs."""
    n = int(2.8 * R); t = np.arange(n) / R
    o = 0.16 * np.minimum(t / 0.006, 1) * sum(a * np.exp(-t / d) * np.sin(TAU * freq * m * t) for m, a, d in CHIME_PARTIALS)
    return np.stack([o, o], 1)


def main():
    p = pad(60, D_CHORD) * g(-9)
    print('pad D @-9 dB: integrated %.1f LUFS, TP %.1f dBTP' % (lufs(p[5 * R:55 * R]), true_peak(p)))
    for f in [523.25, 587.33, 659.25, 880.0]:
        c = chime(f) * g(-10)
        print('chime %4.0f Hz @-10 dB: loudest 400 ms %.1f LUFS, whole-cue %.1f LUFS, sample peak %.1f dBFS, TP %.1f dBTP; partials Hz %s'
              % (f, momentary_max(c), lufs(c), 20 * np.log10(np.abs(c).max()), true_peak(c), [round(f * m) for m, _, _ in CHIME_PARTIALS]))
    print('detune 1.003 = %.2f cents' % (1200 * np.log2(1.003)))
    print('beat Hz per chord tone:', {f: round(0.003 * f, 2) for f in [98, 123.47, 146.83, 185, 220, 293.66, 369.99, 440]})
    print('AM depth of a 1.0 + 0.6 sine pair: %.1f dB' % (20 * np.log10(1.6 / 0.4)))
    print('breathe LFO 0.8 +/- 0.2 swing: %.1f dB, period %.1f s' % (20 * np.log10(1.0 / 0.6), 1 / 0.07))
    m = pad(20, D_CHORD).mean(1); f2, P2 = signal.welch(m, R, nperseg=8192)
    print('pad spectral centroid %.0f Hz; energy above 2 kHz %.1f dB re total'
          % ((f2 * P2).sum() / P2.sum(), 10 * np.log10(P2[f2 > 2000].sum() / P2.sum() + 1e-30)))


if __name__ == '__main__':
    main()
