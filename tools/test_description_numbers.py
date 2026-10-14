#!/usr/bin/env python3
"""tools/description_numbers.py on small descriptions with planted wrong numbers (Video J's real ones among
them: 70.7 times closer for 8.1, 0.3208 for 0.35), and on what must not be read as a number.

    python3 -I tools/test_description_numbers.py
"""
import contextlib
import io
import sys
import tempfile
import unittest
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import description_numbers as dn  # noqa: E402

ORACLE = """# a comment line, with a 999 in it
F-C.pade.m3.times_closer\t8.1
F-C.mvl.term.largest\t9303002.5
F-C.mvl.f32.scaled.worst_error\t0.000034
F-T.share\t0.34
F-C.mvl.size\t89.6324
F-C.mvl.f32.series\t-0.788594,1.314455;-0.126288,-0.839240
F-C.mvl.f32.terms\t62
F-V.13_5.series\t2.71806
F-N.flag\ttrue
"""


def run(text, oracle=ORACLE, allow=None):
    rows = [dict(r, used=0) for r in (allow or [])]
    report, problems, counts = dn.check(text, oracle, rows)
    return problems, counts, rows


def allow(number, context, kind, reason, no=1):
    return {"no": no, "number": number, "context": context, "kind": kind, "reason": reason}


class Extraction(unittest.TestCase):
    def numbers(self, text):
        return [str(n) for n in dn.extract(text)[0]]

    def test_reads_decimals_commas_percent_and_powers(self):
        got = self.numbers("It is 9,303,002.5 or 3.4 × 10⁻⁵ or 34% or 9.3 million or 10^{-6}.")
        self.assertEqual(got, ["9303002.5", "3.4e-5", "34%", "9.3", "1e-6"])

    def test_not_read(self):
        text = "\n".join(
            [
                "<!-- Draft R2, 2026-10-14. 12 numbers in a comment",
                "still a comment 77 -->",
                "1. The book",
                "See F-C.mvl.f32.series and 01_6 and chapter 13 at 12:34 on https://x.org/a/123 now.",
                "The x264 encoder, an f32 and A_2 and 3Blue1Brown.",
                "",
                "## Sources",
                "Moler and Van Loan, SIAM Review 20(4), 1978, pp. 801-836.",
                "## After",
                "Here 42 stays.",
            ]
        )
        numbers, exempt = dn.extract(text)
        self.assertEqual([str(n) for n in numbers], ["42"])
        for reason in ("list", "key", "beat", "chapter", "timestamp", "url", "sources"):
            self.assertIn(reason, exempt, reason)

    def test_a_name_with_digits_is_not_a_number_but_a_unit_is(self):
        self.assertEqual(self.numbers("3Blue1Brown, a 20 Hz tone, 5 ms and 2x."), ["20", "5", "2"])


class Matching(unittest.TestCase):
    def test_the_real_defect_of_video_j(self):
        problems, _, _ = run("It is `70.7` times closer.")
        self.assertEqual(len(problems), 1)
        self.assertIn("70.7", problems[0])
        problems, counts, _ = run("It is `8.1` times closer.")
        self.assertEqual(problems, [])
        self.assertEqual(counts["fact"], 1)

    def test_a_description_may_round_a_fact_but_not_change_it(self):
        for ok in ("9.3 million", "9,303,002.5", "9,303,003", "9.30 million"):
            self.assertEqual(run(f"The largest is {ok}.")[0], [], ok)
        for bad in ("9.4 million", "8.36 × 10⁷", "9,303,004"):
            self.assertEqual(len(run(f"The largest is {bad}.")[0]), 1, bad)

    def test_scientific_and_percent_forms(self):
        self.assertEqual(run("Worst error 3.4 × 10⁻⁵ and 0.000034.")[0], [])
        self.assertEqual(len(run("Worst error 7 × 10⁻⁶.")[0]), 1)
        self.assertEqual(run("A share of 34%.")[0], [])
        self.assertEqual(len(run("A share of 35%.")[0]), 1)

    def test_a_small_whole_number_must_be_a_facts_number_exactly(self):
        self.assertEqual(run("It stops at k = 62.")[0], [])
        problems, _, _ = run("It stops at k = 61.")
        self.assertEqual(len(problems), 1)
        # 8.1 is a fact, 8 is not a rounding of it for a whole number this small
        self.assertEqual(len(run("It is 8 times closer.")[0]), 1)

    def test_every_number_of_a_matrix_is_read(self):
        self.assertEqual(run("It returns [−0.788594, 1.314455; −0.126288, −0.839240].")[0], [])
        problems, _, _ = run("It returns [−1.275748, 0.697661].")
        self.assertEqual(len(problems), 2)

    def test_booleans_and_comments_in_the_oracle_are_no_numbers(self):
        self.assertEqual(len(run("There are 999 of them.")[0]), 1)

    def test_signs_are_not_checked(self):
        self.assertEqual(run("An entry of 0.788594.")[0], [])


class AllowList(unittest.TestCase):
    def test_a_year_row_covers_its_number_in_its_context(self):
        rows = [allow("2010", "Higham 2010", "year", "Estrada and Higham, cited")]
        problems, counts, _ = run("Estrada and Higham 2010 wrote it.", allow=[allow("2010", "Higham 2010", "year", "cited")])
        self.assertEqual(problems, [])
        self.assertEqual(counts["allowed"], 1)
        problems, _, _ = run("Baez 2010 wrote it.", allow=rows)
        self.assertEqual(len(problems), 2, problems)  # the number is not covered here, and the row is unused

    def test_a_row_nothing_uses_is_an_error(self):
        problems, _, _ = run("Nothing odd.", allow=[allow("1978", "Moler", "year", "cited", no=7)])
        self.assertEqual(len(problems), 1)
        self.assertIn(":7:", problems[0])

    def test_a_derived_row_is_evaluated(self):
        good = allow("0.35", "size", "derived", '=f("F-C.mvl.size")/256')
        problems, counts, _ = run("Its size 0.35 is under one.", allow=[good])
        self.assertEqual(problems, [])
        self.assertEqual(counts["allowed"], 1)
        bad = allow("0.3208", "size", "derived", '=f("F-C.mvl.size")/256')
        problems, _, _ = run("Its size 0.3208 is under one.", allow=[bad])
        self.assertTrue(any("does not give it" in p for p in problems), problems)

    def test_derived_expressions_use_facts_and_a_count_and_nothing_else(self):
        self.assertEqual(run("It is 1957 in all.", allow=[allow("1957", "1957", "derived", '=f("F-V.13_5.series")*720')])[0], [])
        self.assertEqual(run("Of 9 keys.", allow=[allow("9", "keys", "derived", "=n_facts()")])[0], [])
        problems, _, _ = run("Of 3 keys.", allow=[allow("3", "keys", "derived", '=__import__("os")')])
        self.assertTrue(problems)
        problems, _, _ = run("Of 3 keys.", allow=[allow("3", "keys", "derived", '=f("F-NO.SUCH")')])
        self.assertTrue(any("fails" in p for p in problems), problems)

    def test_the_file_needs_its_four_columns_and_a_known_kind(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "description_numbers.tsv"
            path.write_text("# c\n2010\tHigham 2010\tyear\tcited\n")
            self.assertEqual(len(dn.read_allow(path)), 1)
            path.write_text("2010\tHigham 2010\tguess\tcited\n")
            with self.assertRaises(SystemExit):
                dn.read_allow(path)
            path.write_text("2010\tHigham 2010\tyear\n")
            with self.assertRaises(SystemExit):
                dn.read_allow(path)


class CommandLine(unittest.TestCase):
    def main(self, root, *argv):
        dn.ROOT = root
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = dn.main(list(argv))
        return code, out.getvalue()

    def test_exit_codes_and_the_summary_line(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            docs = root / "explainer_corpus" / "demo" / "docs"
            docs.mkdir(parents=True)
            oracle = root / "oracle.txt"
            oracle.write_text(ORACLE)
            (docs / "description.md").write_text("# Demo\n\nIt is `8.1` times closer.\n")
            code, out = self.main(root, "demo", "--oracle", str(oracle))
            self.assertEqual(code, 0, out)
            self.assertIn("description-numbers demo: 1 numbers: 1 are facts", out)
            (docs / "description.md").write_text("# Demo\n\nIt is `70.7` times closer.\n")
            code, out = self.main(root, "demo", "--oracle", str(oracle))
            self.assertEqual(code, 1)
            self.assertIn("MISMATCH line 3: 70.7 is not a fact", out)


if __name__ == "__main__":
    unittest.main()
