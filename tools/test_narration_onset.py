#!/usr/bin/env python3
"""The cut-in (`hear`) on synthetic reads: a first word that opens on loud frication before the pitch
tracker hears a voice keeps its onset; a long noise bed under the speech is still left out. No TTS or
ASR runs.

    uv run --no-project --with numpy python3 -I tools/test_narration_onset.py SKILL_SCRIPTS

Synthetic sound tests the rule, not whether a real read keeps its first word: the take sheets do that
(Video E 07_1.1, "Six patterns": the old rule cut the s of "Six", 0.55 to 0.74 s of the take).
"""
import importlib.util
from pathlib import Path
import sys
import unittest

import numpy as np

if len(sys.argv) < 2 or sys.argv[1].startswith("-"):
    sys.exit("pass the narration skill's scripts directory (containing narrate.py)")
SKILL_SCRIPTS = str(Path(sys.argv.pop(1)).resolve())
sys.dont_write_bytecode = True
_spec = importlib.util.spec_from_file_location("narration", Path(__file__).resolve().parent / "narration.py")
tool = importlib.util.module_from_spec(_spec)
_argv, sys.argv = sys.argv, [sys.argv[0], SKILL_SCRIPTS]  # narration.py finds narrate.py by argv[1]
_spec.loader.exec_module(tool)
sys.argv = _argv

RATE = 24000
rng = np.random.default_rng(11)


def at(x, db):
    """x scaled to an RMS of `db` dBFS."""
    return x * 10 ** (db / 20) / np.sqrt(np.mean(x ** 2))


def silence(s):
    return np.zeros(int(s * RATE))


def hiss(s, db, lo=3000.0, hi=9000.0):
    """Band noise, an s or a breath."""
    spec = np.fft.rfft(rng.standard_normal(int(s * RATE)))
    f = np.fft.rfftfreq(int(s * RATE), 1 / RATE)
    spec[(f < lo) | (f > hi)] = 0
    return at(np.fft.irfft(spec, int(s * RATE)), db)


def creak(s, db):
    """Glottal pulses at wildly uneven intervals: a creaky vowel onset with no pitch to track."""
    x = np.zeros(int(s * RATE))
    i = 0
    while i < len(x):
        x[i] = 1.0
        i += int(RATE / rng.uniform(25, 140))
    kernel = np.exp(-np.arange(int(0.004 * RATE)) / (0.001 * RATE)) * np.sin(2 * np.pi * 700 * np.arange(int(0.004 * RATE)) / RATE)
    return at(np.convolve(x, kernel)[:len(x)], db)


def vowel(s, db, f0=120.0):
    t = np.arange(int(s * RATE)) / RATE
    return at(sum(np.sin(2 * np.pi * f0 * k * t) / k for k in range(1, 25)), db)


def take(*parts):
    return np.concatenate(parts)


class CutIn(unittest.TestCase):
    def onset_kept(self, x, onset_s):
        h = tool.hear(x, RATE, None)
        self.assertIsNotNone(h)
        return h["cut_in"] <= onset_s + 0.01, h

    def test_a_loud_s_before_a_creaky_vowel_is_the_first_word(self):
        # "Six": 0.19 s of s at speech level, then a quarter second of creak the tracker cannot follow
        onset = 0.40
        x = take(silence(onset), hiss(0.19, -24), creak(0.25, -26), vowel(0.6, -22), silence(0.4))
        kept, h = self.onset_kept(x, onset)
        self.assertTrue(kept, f"the s was cut: cut-in {h['cut_in']:.3f} s, onset {onset} s")

    def test_a_short_aspiration_at_speech_level_is_kept(self):
        # "Take": a t burst and its aspiration, then the vowel after a creaky start
        onset = 0.35
        x = take(silence(onset), hiss(0.06, -20, 1500, 9000), creak(0.25, -27), vowel(0.5, -22), silence(0.3))
        kept, h = self.onset_kept(x, onset)
        self.assertTrue(kept, f"the t was cut: cut-in {h['cut_in']:.3f} s, onset {onset} s")

    def test_a_long_bed_under_the_speech_is_still_left_out(self):
        # robert: 0.6 s of noise about 12 dB under the speech before the first word
        # (the rule keeps the last 0.2 s before the voice and starts 120 ms before that: a soft
        # consonant's worth)
        bed, word = 0.20, 0.80
        x = take(silence(bed), hiss(0.60, -36, 300, 9000), vowel(0.6, -22), silence(0.3))
        h = tool.hear(x, RATE, None)
        self.assertGreater(h["cut_in"], bed + 0.2, f"the bed was kept: cut-in {h['cut_in']:.3f} s")
        self.assertLessEqual(h["cut_in"], word + 0.01, f"the word was cut: cut-in {h['cut_in']:.3f} s")

    def test_a_long_loud_noise_before_the_voice_is_still_left_out(self):
        # 0.6 s of hiss at speech level is no syllable onset: the rule keeps treating it as a bed
        x = take(silence(0.2), hiss(0.60, -24, 300, 9000), vowel(0.6, -22), silence(0.3))
        h = tool.hear(x, RATE, None)
        self.assertGreater(h["cut_in"], 0.4, f"the noise was kept: cut-in {h['cut_in']:.3f} s")


if __name__ == "__main__":
    unittest.main()
