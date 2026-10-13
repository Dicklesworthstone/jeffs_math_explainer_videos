#!/usr/bin/env python3
"""A director's pick wins the sentence but keeps its measured score (bead mx-e-w1-4qa.17).

    uv run --no-project --with numpy python3 -I tools/test_narration_picks.py SKILL_SCRIPTS

The pick used to add 1.0 to the reported score (a 0.8998 read became 1.8998), which hid it from
the weak-read list. `Picked` now sorts first under narrate.py's ranking key and is the measured
number everywhere else. The three uses below are narrate.py's own: `cands.sort(key=lambda c:
-c[0])`, `best["score"] < 0.8` for the weak-read list, and `best` written into report.json.
"""
import importlib.util
import json
from pathlib import Path
import sys
import unittest

if len(sys.argv) < 2 or sys.argv[1].startswith("-"):
    sys.exit("pass the narration skill's scripts directory (containing narrate.py)")
SKILL_SCRIPTS = str(Path(sys.argv.pop(1)).resolve())
sys.dont_write_bytecode = True
_spec = importlib.util.spec_from_file_location("narration", Path(__file__).resolve().parent / "narration.py")
tool = importlib.util.module_from_spec(_spec)
_argv, sys.argv = sys.argv, [sys.argv[0], SKILL_SCRIPTS]  # narration.py finds narrate.py by argv[1]
_spec.loader.exec_module(tool)
sys.argv = _argv


def rank(cands):
    """narrate.py's choose: the candidates sorted by their score, best first."""
    return sorted(cands, key=lambda c: -c[0])


class DirectorPick(unittest.TestCase):
    def test_the_pick_wins_over_a_higher_measured_read(self):
        picked = (tool.Picked(0.8998), "picked")
        best = rank([(1.0, "a perfect score"), picked, (0.95, "another")])[0]
        self.assertEqual(best[1], "picked")

    def test_the_report_keeps_the_measured_score(self):
        self.assertEqual(json.loads(json.dumps({"score": tool.Picked(0.8998)}))["score"], 0.8998)
        self.assertEqual(float(tool.Picked(0.42)), 0.42)

    def test_a_weak_pick_stays_on_the_weak_list(self):
        self.assertTrue(tool.Picked(0.5) < 0.8)
        self.assertFalse(tool.Picked(0.9) < 0.8)

    def test_unpicked_reads_rank_by_score(self):
        self.assertEqual([c[1] for c in rank([(0.7, "c"), (0.9, "a"), (0.8, "b")])], ["a", "b", "c"])


if __name__ == "__main__":
    unittest.main()
