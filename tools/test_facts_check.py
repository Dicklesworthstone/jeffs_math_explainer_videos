#!/usr/bin/env python3
"""tools/facts_check.py's description step (no cargo, no oracle run): a video with a description
allow-list has its published numbers checked against the facts; one without is left alone.

    python3 -I tools/test_facts_check.py
"""
import sys
import tempfile
import unittest
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import facts_check as fc  # noqa: E402

# Video J's chapter-10 tanks: the facts say 90.9 and 9.1 g at t = 1; the description once said 81.9 and 18.1
FACTS = "F-T.tanks.t1.left\t90.9\nF-T.tanks.t1.right\t9.1\nF-C.mvl.term.largest\t9303002.5\n"
ALLOW_HEADER = "# number\tcontext\tkind\treason\n"


def crate(tmp, description, allow=None):
    """A crate directory with docs/description.md and, if given, docs/description_numbers.tsv."""
    root = Path(tmp) / "fake_video"
    (root / "docs").mkdir(parents=True)
    (root / "docs" / "description.md").write_text(description)
    if allow is not None:
        (root / "docs" / "description_numbers.tsv").write_text(ALLOW_HEADER + allow)
    return root


class DescriptionStep(unittest.TestCase):
    def test_without_an_allow_list_the_description_is_not_checked(self):
        with tempfile.TemporaryDirectory() as tmp:
            self.assertIsNone(fc.description_problems(crate(tmp, "The tanks hold 81.9 and 18.1 g."), FACTS))

    TAGS = " [F-T.tanks.t1.left, F-T.tanks.t1.right]"

    def test_the_facts_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            summary, problems = fc.description_problems(crate(tmp, "At one minute the tanks hold 90.9 and 9.1 g." + self.TAGS, ""), FACTS)
            self.assertEqual(problems, [], summary)
            self.assertIn("PASS", summary)

    def test_a_typed_wrong_number_fails(self):
        # the planted negative: Video J's real error
        with tempfile.TemporaryDirectory() as tmp:
            summary, problems = fc.description_problems(crate(tmp, "At one minute the tanks hold 81.9 and 18.1 g." + self.TAGS, ""), FACTS)
            self.assertTrue(problems, summary)
            self.assertTrue(all(p.startswith("description: ") for p in problems), problems)
            self.assertIn("FAIL", summary)

    def test_a_number_that_names_no_fact_needs_a_row_even_when_it_equals_one(self):
        with tempfile.TemporaryDirectory() as tmp:
            summary, problems = fc.description_problems(crate(tmp, "At one minute the tanks hold 90.9 and 9.1 g.", ""), FACTS)
            self.assertEqual(len(problems), 2, problems)
            self.assertIn("FAIL", summary)
        with tempfile.TemporaryDirectory() as tmp:
            legacy = crate(tmp, "At one minute the tanks hold 90.9 and 9.1 g.", "")
            (legacy / "docs" / "description_numbers.tsv").write_text("#! legacy: not tagged yet\n" + ALLOW_HEADER)
            summary, problems = fc.description_problems(legacy, FACTS)
            self.assertEqual(problems, [], summary)
            self.assertIn("LEGACY", summary)

    def test_an_allow_listed_number_passes(self):
        allow = "1985\tThe picture was published in\tyear\ta year, not a computed number\n"
        with tempfile.TemporaryDirectory() as tmp:
            summary, problems = fc.description_problems(crate(tmp, "The picture was published in 1985.", allow), FACTS)
            self.assertEqual(problems, [], summary)

    def test_an_allow_list_without_a_description_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = crate(tmp, "", "")
            (root / "docs" / "description.md").rename(root / "docs" / "description.previous.md")
            summary, problems = fc.description_problems(root, FACTS)
            self.assertEqual(problems, ["description: missing"])


if __name__ == "__main__":
    unittest.main()
