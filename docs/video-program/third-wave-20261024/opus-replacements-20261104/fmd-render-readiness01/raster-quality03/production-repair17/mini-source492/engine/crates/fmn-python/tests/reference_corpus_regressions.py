"""Minimized constructs from corpus scenes the portal once refused or rendered
differently, each checked against what the pinned Reference showed.

The e2e scenario parity.reference_corpus_regressions.v1 runs this module in the
embedded interpreter (fm-5wq.27..30, fm-sq8.10 each add their cases here).
Every case records the bead, the construct, the observed and expected values,
the expectation's provenance and a verdict; no timestamps or addresses.
"""
import math

import manimlib as m

REFERENCE = "3b1b/manim@6199a00d"


def _decimal_text(value, **kwargs):
    return m.DecimalNumber(value, **kwargs).get_tex()


def _set_value_text(value):
    number = m.DecimalNumber(1.5)
    number.set_value(value)
    return number.get_tex()


# (bead, construct, observe, expected, provenance). Expected values were
# measured on the pinned Reference; _2023/clt/wordy_scenes.py AverageDiceValues
# shows a DecimalNumber of an empty average, which is nan.
CASES = (
    ("fm-5wq.30", "DecimalNumber(float('nan'))",
     lambda: _decimal_text(math.nan), "nan",
     REFERENCE + ":manimlib/mobject/numbers.py:get_num_string (measured)"),
    ("fm-5wq.30", "DecimalNumber(float('nan'), include_sign=True)",
     lambda: _decimal_text(math.nan, include_sign=True), "+nan",
     REFERENCE + ":manimlib/mobject/numbers.py:get_num_string (measured)"),
    ("fm-5wq.30", "DecimalNumber(1.5).set_value(float('nan'))",
     lambda: _set_value_text(math.nan), "nan",
     REFERENCE + ":manimlib/mobject/numbers.py:set_value (measured)"),
    ("fm-5wq.30", "DecimalNumber(float('inf'))",
     lambda: _decimal_text(math.inf), "inf",
     REFERENCE + ":manimlib/mobject/numbers.py:get_num_string (measured)"),
)


def run_reference_corpus_regressions():
    """One record per case; the caller fails the run on any non-pass."""
    records = []
    for bead, construct, observe, expected, provenance in CASES:
        try:
            observed = observe()
        except Exception as error:  # noqa: BLE001 - a raise is the observation
            observed = f"raises {type(error).__name__}"
        records.append({
            "bead": bead, "construct": construct, "observed": observed,
            "expected": expected, "provenance": provenance,
            "verdict": "pass" if observed == expected else "fail",
        })
    return records
