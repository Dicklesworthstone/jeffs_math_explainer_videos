#!/usr/bin/env python3
"""The final s/z check (`sibilant_missing`) on synthetic reads: a vowel, then what the last word
ends in. No TTS or ASR runs.

    uv run --no-project --with numpy python3 -I tools/test_narration_sibilant.py SKILL_SCRIPTS

Synthetic sound tests the measure, not whether a real read has its ending: the take sheets and
the spectrogram do that.
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
rng = np.random.default_rng(7)


def at(x, db):
    """x scaled to an RMS of `db` dBFS."""
    return x * 10 ** (db / 20) / np.sqrt(np.mean(x ** 2))


def voiced(s, top_harmonic, db, f0=120.0):
    t = np.arange(int(s * RATE)) / RATE
    return at(sum(np.sin(2 * np.pi * f0 * k * t) / k for k in range(1, top_harmonic + 1)), db)


def noise(s, lo, hi, db):
    """White noise kept between lo and hi Hz."""
    spec = np.fft.rfft(rng.standard_normal(int(s * RATE)))
    f = np.fft.rfftfreq(int(s * RATE), 1 / RATE)
    spec[(f < lo) | (f >= hi)] = 0
    return at(np.fft.irfft(spec, int(s * RATE)), db)


VOWEL = voiced(0.35, 18, -14.0)        # harmonics to 2.2 kHz
S = noise(0.15, 3500, 9500, -22.0)      # an s
SH = noise(0.18, 2200, 6500, -22.0)     # an sh: lower
BREATH = noise(0.30, 300, 8000, -48.0)  # quieter than speech by more than SIBILANT_GATE
NASAL = voiced(0.10, 3, -24.0)          # an n: the murmur under 400 Hz


def missing(*parts):
    """sibilant_missing on the parts in order, then 50 ms of silence; the read ends at the WAV's end."""
    x = np.concatenate([*parts, np.zeros(int(0.05 * RATE))])
    db = tool.levels(x, RATE)
    top = float(np.percentile(db[db > db.max() - 45], 95))  # the speech level, as `hear` measures it
    return tool.sibilant_missing(x, RATE, len(x) / RATE, top)


class FinalSibilant(unittest.TestCase):
    def test_a_word_that_ends_in_its_s_passes(self):
        self.assertFalse(missing(VOWEL, S))
        self.assertFalse(missing(VOWEL, SH))

    def test_a_dropped_s_fails(self):
        # "phone" for "phones": the read ends in the vowel, or in the n before the s
        self.assertTrue(missing(VOWEL))
        self.assertTrue(missing(VOWEL, NASAL))

    def test_an_s_clipped_by_the_end_of_the_take_fails(self):
        # ftts ends the WAV 30 ms into it ("cell-")
        self.assertTrue(missing(VOWEL, S[:int(0.03 * RATE)]))

    def test_the_breath_after_a_full_s_is_not_its_ending(self):
        # "months", "ninety-six": the earlier check measured the breath and failed a full s
        self.assertFalse(missing(VOWEL, S, BREATH))

    def test_an_s_at_the_start_of_the_word_is_not_its_ending(self):
        # "slots" read as "slo": the earlier check found the first s inside its 0.3 s and passed it
        self.assertTrue(missing(S, voiced(0.20, 18, -14.0)))

    def test_a_voiced_z_passes(self):
        # "sees": a voice bar under 300 Hz under the frication
        z = voiced(0.12, 2, -20.0) + noise(0.12, 3500, 9500, -24.0)
        self.assertFalse(missing(VOWEL, z))

    def test_no_speech_in_reach_is_not_judged(self):
        self.assertFalse(tool.sibilant_missing(np.zeros(RATE), RATE, 1.0, -20.0))


if __name__ == "__main__":
    unittest.main()
