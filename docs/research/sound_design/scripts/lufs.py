"""ITU-R BS.1770 helpers at 48 kHz: K-weighting, gated integrated loudness,
momentary (400 ms) / short-term (3 s) loudness and 4x-oversampled true peak.
Library module; imported by the other scripts. Needs numpy + scipy."""
import numpy as np
from scipy import signal

# BS.1770 K-weighting biquads, published 48 kHz coefficients.
B1 = [1.53512485958697, -2.69169618940638, 1.19839281085285]
A1 = [1, -1.69065929318241, 0.73248077421585]
B2 = [1, -2, 1]
A2 = [1, -1.99004745483398, 0.99007225036621]


def kw(x, sr=48000):
    assert sr == 48000, "K-weighting coefficients are for 48 kHz"
    return signal.lfilter(B2, A2, signal.lfilter(B1, A1, x, axis=0), axis=0)


def _cols(x):
    return x[:, None] if x.ndim == 1 else x


def lufs(x, sr=48000):
    """Gated integrated loudness (abs gate -70 LUFS, relative gate -10 LU)."""
    z = kw(_cols(x))
    blk, hop = int(0.4 * sr), int(0.1 * sr)
    ms = np.array([np.sum(np.mean(z[i:i + blk] ** 2, 0)) for i in range(0, len(z) - blk + 1, hop)])
    l = -0.691 + 10 * np.log10(ms + 1e-30)
    g = ms[l > -70]
    if len(g) == 0:
        return -np.inf
    rel = -0.691 + 10 * np.log10(g.mean()) - 10
    g2 = ms[(l > -70) & (l > rel)]
    return -0.691 + 10 * np.log10(g2.mean())


def momentary_max(x, sr=48000):
    """Loudest 400 ms window (10 ms hop)."""
    z = kw(_cols(x))
    blk, hop = int(0.4 * sr), int(0.01 * sr)
    ms = np.array([np.sum(np.mean(z[i:i + blk] ** 2, 0)) for i in range(0, len(z) - blk + 1, hop)])
    return -0.691 + 10 * np.log10(ms.max())


def short_term(x, sr=48000, hop_s=0.1):
    """3 s short-term loudness series."""
    z = kw(_cols(x))
    blk, hop = int(3 * sr), int(hop_s * sr)
    ms = np.array([np.sum(np.mean(z[i:i + blk] ** 2, 0)) for i in range(0, len(z) - blk + 1, hop)])
    return -0.691 + 10 * np.log10(ms + 1e-30)


def true_peak(x, sr=48000):
    """dBTP via 4x polyphase oversampling."""
    y = signal.resample_poly(_cols(x), 4, 1, axis=0)
    return 20 * np.log10(np.abs(y).max())


def read_wav_48k_mono(path):
    """Read a PCM/float WAV, downmix to mono, resample to 48 kHz, scale to [-1, 1]."""
    from scipy.io import wavfile
    sr, x = wavfile.read(path)
    if x.dtype == np.int16:
        x = x.astype(float) / 32768
    elif x.dtype == np.int32:
        x = x.astype(float) / 2 ** 31
    else:
        x = x.astype(float)
    if x.ndim > 1:
        x = x.mean(1)
    gcd = np.gcd(48000, sr)
    if sr != 48000:
        x = signal.resample_poly(x, 48000 // gcd, sr // gcd)
    return x
