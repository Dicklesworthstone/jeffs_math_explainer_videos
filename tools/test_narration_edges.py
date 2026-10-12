#!/usr/bin/env python3
"""Real edges CLI metadata controls; no audio, ASR or TTS is produced.

    uv run --no-project --with numpy python3 -I tools/test_narration_edges.py SKILL_SCRIPTS

Temporary controls are preserved. They test script/report/transcript binding,
not whether a narrator actually pronounced or finished a word.
"""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

if len(sys.argv) < 2 or sys.argv[1].startswith("-"):
    sys.exit("pass the narration skill's scripts directory (containing narrate.py)")
SKILL_SCRIPTS = Path(sys.argv.pop(1)).resolve()
sys.dont_write_bytecode = True
sys.path.insert(0, str(SKILL_SCRIPTS))
import narrate as nr

ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "tools" / "narration.py"


class Edges(unittest.TestCase):
    def setUp(self):
        self.work = Path(tempfile.mkdtemp(prefix="narration_edges_"))
        self.script = self.work / "script.tsv"
        self.text = "Two customers wait. Nobody enters."
        self.script.write_text("01_1\t" + self.text + "\n", encoding="utf-8")
        self.report = {"01_1": [{"j": j, "sentence": s}
                                  for j, s in enumerate(nr.sentences(self.text))]}

    def run_edges(self, report, heard=None):
        out = Path(tempfile.mkdtemp(prefix="case_", dir=self.work))
        (out / "report.json").write_text(json.dumps({"lines": report}), encoding="utf-8")
        if heard is None:
            heard = {lid: {"text": " ".join(s["sentence"] for s in sents)}
                     for lid, sents in report.items()}
        (out / "lines_asr.json").write_text(json.dumps(heard), encoding="utf-8")
        return subprocess.run(
            [sys.executable, "-I", str(TOOL), str(SKILL_SCRIPTS), "edges",
             str(self.script), str(out)],
            cwd=self.work, capture_output=True, text=True, timeout=20, check=False,
        )

    def expect(self, report, code, heard=None):
        result = self.run_edges(report, heard)
        self.assertEqual(result.returncode, code, result.stdout + result.stderr)

    def test_complete_and_normalized_number_spelling_pass(self):
        self.expect(self.report, 0)
        alternate = copy.deepcopy(self.report)
        alternate["01_1"][0]["sentence"] = "2 customers wait!"
        self.expect(alternate, 0)

    def test_missing_whole_sentence_in_actual_video_b_line_fails(self):
        script = ROOT / "explainer_corpus" / "raptorq_gf256" / "script.tsv"
        text = next(row.split("\t", 1)[1] for row in script.read_text(encoding="utf-8").splitlines()
                    if row.startswith("03_5\t"))
        sentences = nr.sentences(text)
        self.assertGreater(len(sentences), 1)
        self.script.write_text("03_5\t" + text + "\n", encoding="utf-8")
        report = {"03_5": [{"j": j, "sentence": s} for j, s in enumerate(sentences)]}
        self.expect(report, 0)
        report["03_5"].pop()
        self.expect(report, 1)

    def test_empty_line_or_missing_transcript_fails(self):
        self.expect({}, 1)
        self.expect({"01_1": []}, 1)
        self.expect(self.report, 1, {})

    def test_duplicate_blank_index_changed_content_and_extra_line_fail(self):
        for field, value in (("sentence", self.report["01_1"][0]["sentence"]),
                             ("sentence", ""), ("j", 0), ("sentence", "Somebody enters.")):
            with self.subTest(field=field, value=value):
                report = copy.deepcopy(self.report)
                report["01_1"][1][field] = value
                self.expect(report, 1)
        report = copy.deepcopy(self.report)
        report["02_1"] = [{"j": 0, "sentence": "An extra line."}]
        self.expect(report, 1)

    def test_complete_coverage_still_refuses_a_missing_first_or_last_word(self):
        for text in ("Customers wait. Nobody enters.", "Two customers wait. Nobody."):
            with self.subTest(text=text):
                self.expect(self.report, 1, {"01_1": {"text": text}})


if __name__ == "__main__":
    unittest.main()
