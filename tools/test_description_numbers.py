#!/usr/bin/env python3
"""tools/description_numbers.py on small descriptions with planted wrong numbers (Video J's real ones among
them: 70.7 times closer for 8.1, 0.3208 for 0.35), on the single-number errors that the first version of the
tool let through (the off-by-one sweep of mx-zbp5: a wrong 4 for a rank of 5, 22.5 for 21.5, 36 for 35, all
of which equalled some other fact of the oracle), and on what must not be read as a number.

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

# the keys of the real oracles that the planted errors below stand on (names and values as the oracles print them)
REAL = """G-J.rank.stretch\t4
G-J.rank.elbow_straight\t5
G-J.blowup.bend_at_limit\t21.5
G-I.screw.quarter_turned\t22.5
G-B.leftover.1.axis\t-0.0087,0.0087,-0.9999
G-B.leftover.10.axis\t-0.0868,0.0868,-0.9924
G-C.g1.joints\t29
G-C.g1.velocity\t35
G-C.g1.stored\t36
G-S.numbers.twist\t6
G-R.numbers.arrow\t3
G-S.numbers.quaternion\t4
F-S.F6.searched.after\t47,65,74
F-A.aboard\t228
F-A.months\t22
F-L.sliver.pieces\t17
F-C.evenly_spaced.regions\t30
F-C.mvl.exact\t-0.735759,0.551819;-1.471518,1.103638
F-H.float.reaches_one\t54
F-R.F6\t85
K-P2.lone_pct.4m\t42
K-P2.lone_pct.8m\t76
K-H2.per_hz_at_1000\t5802
K-T5.forces_kn\t250.0,250.0,400.0,400.0,-353.6,212.1,70.7,70.7,212.1
K-E5.rounds\t30;10 15;11 9.3;8.5
K-T4.loads\t5
F-T.eigenvalues\t-0.200000,0.000000
"""
ORACLE_REAL = ORACLE + REAL


def run(text, oracle=ORACLE, allow=None, legacy=False):
    rows = dn.AllowList(dict(r, used=0) for r in (allow or []))
    rows.legacy = legacy
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
                "- **06.** The platform's velocity is held constant.",
                "See F-C.mvl.f32.series and K-T5.forces_kn and 01_6 and chapter 13 at 12:34 on https://x.org/a/123 now.",
                "The x264 encoder, an f32 and A_2 and 3Blue1Brown, joints 5 and 7, row 4, Table 3, §6.2.2, *Math. Mag.* 57(2).",
                "",
                "## Sources",
                "Moler and Van Loan, SIAM Review 20(4), 1978, pp. 801-836.",
                "## After",
                "Here 42 stays.",
            ]
        )
        numbers, exempt = dn.extract(text)
        self.assertEqual([str(n) for n in numbers], ["42"])
        for reason in ("list", "key", "beat", "chapter", "timestamp", "url", "sources", "index", "section", "volume"):
            self.assertIn(reason, exempt, reason)
        self.assertEqual(exempt["key"], 2)  # the two-character family K-T5 is a key too

    def test_a_bold_chapter_label_is_a_label_but_a_number_in_the_item_is_read(self):
        self.assertEqual(self.numbers("- **06.** After 6 seconds."), ["6"])

    def test_a_name_with_digits_is_not_a_number_but_a_unit_is(self):
        self.assertEqual(self.numbers("3Blue1Brown, a 20 Hz tone, 5 ms and 2x."), ["20", "5", "2"])


class Matching(unittest.TestCase):
    """The old rule, kept for a description not tagged yet (a `#! legacy` line in its allow-list)."""

    def test_the_real_defect_of_video_j(self):
        problems, _, _ = run("It is `70.7` times closer.", legacy=True)
        self.assertEqual(len(problems), 1)
        self.assertIn("70.7", problems[0])
        problems, counts, _ = run("It is `8.1` times closer.", legacy=True)
        self.assertEqual(problems, [])
        self.assertEqual(counts["fact"], 1)

    def test_a_description_may_round_a_fact_but_not_change_it(self):
        for ok in ("9.3 million", "9,303,002.5", "9,303,003", "9.30 million"):
            self.assertEqual(run(f"The largest is {ok}.", legacy=True)[0], [], ok)
        for bad in ("9.4 million", "8.36 × 10⁷", "9,303,004"):
            self.assertEqual(len(run(f"The largest is {bad}.", legacy=True)[0]), 1, bad)

    def test_scientific_and_percent_forms(self):
        self.assertEqual(run("Worst error 3.4 × 10⁻⁵ and 0.000034.", legacy=True)[0], [])
        self.assertEqual(len(run("Worst error 7 × 10⁻⁶.", legacy=True)[0]), 1)
        self.assertEqual(run("A share of 34%.", legacy=True)[0], [])
        self.assertEqual(len(run("A share of 35%.", legacy=True)[0]), 1)

    def test_a_small_whole_number_must_be_a_facts_number_exactly(self):
        self.assertEqual(run("It stops at k = 62.", legacy=True)[0], [])
        self.assertEqual(len(run("It stops at k = 61.", legacy=True)[0]), 1)
        # 8.1 is a fact, 8 is not a rounding of it for a whole number this small
        self.assertEqual(len(run("It is 8 times closer.", legacy=True)[0]), 1)

    def test_every_number_of_a_matrix_is_read(self):
        self.assertEqual(run("It returns [−0.788594, 1.314455; −0.126288, −0.839240].", legacy=True)[0], [])
        problems, _, _ = run("It returns [−1.275748, 0.697661].", legacy=True)
        self.assertEqual(len(problems), 2)

    def test_booleans_and_comments_in_the_oracle_are_no_numbers(self):
        self.assertEqual(len(run("There are 999 of them.", legacy=True)[0]), 1)

    def test_signs_are_not_checked(self):
        self.assertEqual(run("An entry of 0.788594.", legacy=True)[0], [])

    def test_truncation_is_no_rounding(self):
        self.assertEqual(run("The series gives 2.7181.", legacy=True)[0], [])  # 2.71806 rounded to four places
        self.assertEqual(len(run("The series gives 2.7180.", legacy=True)[0]), 1)  # the same cut off

    def test_a_named_fact_key_must_exist(self):
        self.assertEqual(run("Pinned by [F-C.mvl.size] and `F-C.mvl.*` and F-C.mvl.f32.series..terms.")[0], [])
        problems, _, _ = run("Pinned by [F-T.state.t1] and F-C.mvl.f32.series..nowhere and F-Z.*.")
        self.assertEqual(len(problems), 3, problems)
        self.assertIn("F-T.state.t1 is named but is no key", problems[0])
        self.assertEqual(run("<!-- F-NO.SUCH in a comment -->\nPlain text.")[0], [])
        # a family of two characters (K-T5, G-X6) is a key family too
        problems, _, _ = run("See [K-T5.nowhere].", oracle=ORACLE_REAL)
        self.assertEqual(len(problems), 1, problems)
        self.assertEqual(run("See [K-T5.forces_kn] and K-T5.*.", oracle=ORACLE_REAL)[0], [])


class Units(unittest.TestCase):
    TEXT = "\n".join(
        [
            "Intro paragraph with 1 line.",
            "",
            "- **A.** First item,",
            "  continued [F-C.mvl.size].",
            "- **B.** Second item.",
            "",
            "| Name | Value |",
            "|---|---|",
            "| x | 5 |",
            "[F-T.share]",
            "",
            "```python",
            "# facts: F-N.flag",
            "x = 62",
            "```",
        ]
    )

    def test_a_list_item_a_paragraph_a_table_and_a_code_block_are_units(self):
        ids = dn.unit_ids(self.TEXT)
        self.assertEqual(ids[0], 1)
        self.assertEqual(ids[1], 0)
        self.assertEqual(ids[2], ids[3])  # an item and its continuation line
        self.assertNotEqual(ids[3], ids[4])  # the next item
        self.assertEqual(len({ids[6], ids[7], ids[8], ids[9]}), 1)  # a table and the line under it
        self.assertEqual(len({ids[11], ids[12], ids[13], ids[14]}), 1)  # a code block

    def test_tags_include_the_suffix_shorthand_and_wildcards(self):
        _, tags = dn.unit_tags("- Twist [K-B1.twist_per_min, .twist_hz, .twist_period_s] and `F-C.mvl.*`.")
        (only,) = tags.values()
        self.assertIn("K-B1.twist_per_min", only)
        self.assertIn(("K-B1", ".twist_hz"), only)
        values = {"K-B1.twist_per_min": "12", "K-B1.twist_hz": "0.2", "K-B1.twist_period_s": "5", "F-C.mvl": "1", "F-C.mvl.size": "2", "F-C.mvlx": "3"}
        self.assertEqual(dn.resolve_tag(("K-B1", ".twist_hz"), values), ["K-B1.twist_hz"])
        self.assertEqual(sorted(dn.resolve_tag("F-C.mvl.*", values)), ["F-C.mvl", "F-C.mvl.size"])  # the parent key too, not F-C.mvlx
        self.assertEqual(dn.resolve_tag("K-B1", values).__len__(), 3)  # a key with children stands for them
        self.assertEqual(dn.resolve_tag("F-C.mvl.size", values), ["F-C.mvl.size"])

    def test_the_suffix_shorthand_continues_across_a_line_break_of_the_unit(self):
        _, tags = dn.unit_tags("- Within 0.03% of the tube's [K-R1.error_percent,\n  .error_bound_percent] at 40 slices.")
        (only,) = tags.values()
        self.assertIn(("K-R1", ".error_bound_percent"), only)


class Binding(unittest.TestCase):
    def test_a_number_in_a_tagged_unit_must_equal_a_number_of_that_fact(self):
        self.assertEqual(run("The rank is 5 [G-J.rank.elbow_straight].", ORACLE_REAL)[0], [])
        problems, _, _ = run("The rank is 4 [G-J.rank.elbow_straight].", ORACLE_REAL)  # 4 is G-J.rank.stretch
        self.assertEqual(len(problems), 1, problems)
        self.assertIn("agrees with none of the facts its text names", problems[0])
        self.assertIn("G-J.rank.stretch", problems[0])  # and the message says which fact it equals elsewhere

    def test_the_whole_unit_is_bound_to_the_union_of_its_tags(self):
        text = "- The rank is 5 and a bend of 21.5° [G-J.rank.elbow_straight, G-J.blowup.bend_at_limit]."
        self.assertEqual(run(text, ORACLE_REAL)[0], [])
        # the tags of the next item are not this item's
        text2 = text + "\n- The rank is 4 [G-J.rank.elbow_straight]."
        self.assertEqual(len(run(text2, ORACLE_REAL)[0]), 1)

    def test_a_number_in_an_untagged_unit_needs_a_row_however_many_facts_it_equals(self):
        problems, _, _ = run("The rank is 5.", ORACLE_REAL)
        self.assertEqual(len(problems), 1)
        self.assertIn("tag its text", problems[0])
        rows = [allow("5", "The rank is 5", "structural", "a label")]
        self.assertEqual(run("The rank is 5.", ORACLE_REAL, rows)[0], [])
        # and a fact the text does not name is no excuse for a decimal either
        self.assertEqual(len(run("A bend of 21.5°.", ORACLE_REAL)[0]), 1)

    def test_a_row_covers_a_number_in_a_tagged_unit_that_no_tagged_fact_has(self):
        rows = [allow("2.0", "2.0 km/h sideways", "cited", "the data sheet")]
        text = "KUKA gives 2.0 km/h sideways [G-J.rank.elbow_straight]."
        self.assertEqual(run(text, ORACLE_REAL, rows)[0], [])
        self.assertEqual(len(run("KUKA gives 2.0 km/h sideways [G-J.rank.elbow_straight].", ORACLE_REAL)[0]), 1)

    def test_the_legacy_line_keeps_the_old_rule_and_says_so(self):
        text = "The rank is 5 and a bend of 21.5°."
        self.assertEqual(run(text, ORACLE_REAL, legacy=True)[0], [])
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "description_numbers.tsv"
            path.write_text("#! legacy: not tagged yet\n# c\n2010\tHigham 2010\tyear\tcited\n")
            rows = dn.read_allow(path)
            self.assertTrue(rows.legacy)
            self.assertEqual(len(rows), 1)
            path.write_text("# c\n2010\tHigham 2010\tyear\tcited\n")
            self.assertFalse(dn.read_allow(path).legacy)


class FormulaDigits(unittest.TestCase):
    """What is not checked against the facts, exactly: 0 and 1, and a whole number under 100 that is a
    coefficient, an exponent or a term of an expression with a variable in it."""

    def formula(self, text):
        return [(str(n), n.formula) for n in dn.extract(text)[0]]

    def test_digits_of_a_formula_with_a_variable(self):
        for text, want in (
            ("2k + 1 tiles", [("2", True), ("1", True)]),
            ("n(n + 1) / 2", [("1", True), ("2", True)]),
            ("1 + 3 + … + (2n − 1) = n²", [("1", True), ("3", True), ("2", True), ("1", True)]),
            ("the (k + 1) by (k + 1) square", [("1", True), ("1", True)]),
            ("x^2 and 3x", [("3", True)]),
            ("(1, 1, 0)/√2", [("1", True), ("1", True), ("0", True), ("2", True)]),
            ("SE(3) and f(2)", [("3", True), ("2", True)]),
            ("the pose and 2 (0, 1, 0) at 0.360", [("2", True), ("0", True), ("1", True), ("0", True), ("0.360", False)]),
        ):
            self.assertEqual(self.formula(text), want, text)

    def test_the_terms_of_a_sum_of_plain_numbers_are_claims(self):
        for text in ("so 6 + 29 = 35 numbers", "a 3-4-5 triangle", "a 32-bit float", "the 7 by 7 square", "5 x 5 = 25", "an x = 62 case"):
            for number, is_formula in self.formula(text):
                self.assertFalse(is_formula, (text, number))
        # a letter beside a number with only a space between is a word, not a variable
        self.assertEqual(self.formula("a 3-4-5 triangle")[0], ("3", False))

    def test_decimals_percents_and_numbers_from_a_hundred_are_never_formula_digits(self):
        for text in ("e^{−0.2t}", "2 × 212.1 = 424.2", "n + 100", "k + 50%"):
            self.assertTrue(all(not f for n, f in self.formula(text) if "." in n or n == "100" or n == "50"), text)

    def test_a_hyphen_after_a_letter_is_a_name(self):
        self.assertEqual(self.formula("MTNS-83 met"), [("83", False)])


class Sums(unittest.TestCase):
    def wrong(self, text):
        return [m for _, m in dn.arithmetic_problems(text)]

    def test_a_complete_sum_must_come_out(self):
        for ok in ("6 + 29 = 35", "3 + 4 + 29 = 36", "49 − 24 = 25", "(2 + 3) x (4 + 6) - 8 - 18 = 24, so", "2.2 ÷ 1.9 = 1.2 Hz", "1,000 + 24 = 1,024", "504.99 − 93.93 = 411"):
            self.assertEqual(self.wrong(ok), [], ok)
        for bad in ("6 + 29 = 36", "3 + 4 + 28 = 36", "49 − 24 = 26", "(2 + 3) x (4 + 6) - 8 - 18 = 25", "1,000 + 24 = 1,025", "4 x 5 x 6 = 121 cells"):
            self.assertEqual(len(self.wrong(bad)), 1, bad)

    def test_a_sum_may_wrap_and_may_be_part_of_a_longer_thing(self):
        self.assertEqual(self.wrong("adding to 81 + 216 + 216 +\n  96 + 16 = 625 = 5⁴ here"), [])
        self.assertEqual(len(self.wrong("adding to 81 + 216 + 216 +\n  96 + 16 = 626 here")), 1)
        for left_alone in ("k(k − 1) = 7", "2 + 3 = 5 x 2", "x = 62", "1 + 2 + … + n = n(n+1)/2", "a + 3 × 4 = 99"):
            self.assertEqual(self.wrong(left_alone), [], left_alone)

    def test_it_runs_inside_check(self):
        problems, _, _ = run("It is 6 + 29 = 36 [F-C.mvl.f32.terms].", ORACLE_REAL)
        self.assertTrue(any("does not come out" in p for p in problems), problems)


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

    def test_a_derived_row_may_round_a_small_whole_number_and_use_trigonometry(self):
        row = allow("7", "(7° off)", "derived", '=degrees(asin(hypot(f("G-B.leftover.10.axis",0),f("G-B.leftover.10.axis",1))))')
        self.assertEqual(run("A turn (7° off) the axis [G-B.leftover.10.axis].", ORACLE_REAL, [row])[0], [])
        problems, _, _ = run("A turn (9° off) the axis [G-B.leftover.10.axis].", ORACLE_REAL, [dict(row, number="9", context="(9° off)")])
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


class PlantedErrors(unittest.TestCase):
    """The 17 single-number errors planted in the real descriptions of Videos G, H, I and K on 2026-10-14
    (mx-zbp5), 12 of which the first version of the tool passed. Each case is the sentence as the real
    description tags it, with the number right (must pass) and with the number changed by one unit (must fail)."""

    DEG = '=degrees(asin(hypot(f("G-B.leftover.{n}.axis",0),f("G-B.leftover.{n}.axis",1))))'

    def case(self, good, bad, rows=(), oracle=ORACLE_REAL):
        rows = list(rows)
        ok, _, _ = run(good, oracle, rows)
        self.assertEqual(ok, [], f"the right text must pass: {good!r}")
        bad_rows = [dict(r) for r in rows]
        problems, _, _ = run(bad, oracle, bad_rows)
        self.assertTrue(problems, f"the planted error must fail: {bad!r}")

    # Video G
    def test_g_rank_5_for_4(self):
        self.case("At the quiz pose the rank is 5 [G-J.rank.elbow_straight].", "At the quiz pose the rank is 4 [G-J.rank.elbow_straight].")

    def test_g_bend_21_5_for_22_5(self):
        self.case("The crossing is at a bend of 21.5° [G-J.blowup.bend_at_limit].", "The crossing is at a bend of 22.5° [G-J.blowup.bend_at_limit].")

    def test_g_0_7_degrees_off_for_0_9(self):
        row = allow("0.7", "(0.7° off", "derived", self.DEG.format(n=1))
        self.case("The axis (0.7° off −z) [G-B.leftover.1.axis].", "The axis (0.9° off −z) [G-B.leftover.1.axis].", [row])

    def test_g_7_degrees_off_for_9(self):
        row = allow("7", "(7° off)", "derived", self.DEG.format(n=10))
        self.case("The axis (7° off) [G-B.leftover.10.axis].", "The axis (9° off) [G-B.leftover.10.axis].", [row])

    def test_g_the_sum_6_plus_29_is_35_not_36(self):
        tags = "[G-C.g1.joints, G-C.g1.velocity, G-C.g1.stored, G-S.numbers.twist]"
        self.case(f"29 revolute joints, so 6 + 29 = 35 numbers of velocity {tags}.", f"29 revolute joints, so 6 + 29 = 36 numbers of velocity {tags}.")

    # Video H
    def test_h_0_65_for_0_64(self):
        row = allow("0.65", "it was 0.65", "derived", '=f("F-S.F6.searched.after",1)/100')
        self.case("Estimated afterwards it was 0.65 [F-S.F6.searched.after].", "Estimated afterwards it was 0.64 [F-S.F6.searched.after].", [row])

    def test_h_22_percent_for_23(self):
        row = allow("22", "would be 22%", "cited", "recomputed from the oracle's map (not a key)")
        self.case("From row 5 it would be 22% [F-R.F6].", "From row 5 it would be 23% [F-R.F6].", [row])

    def test_h_228_aboard_for_227(self):
        self.case("Paris went down with 228 aboard [F-A.aboard].", "Paris went down with 227 aboard [F-A.aboard].")

    def test_h_22_months_for_23(self):
        self.case("The wreck was found 22 months on [F-A.months].", "The wreck was found 23 months on [F-A.months].")

    # Video I
    def test_i_17_pieces_for_16(self):
        self.case("It falls into 17 pieces [F-L.sliver.pieces].", "It falls into 16 pieces [F-L.sliver.pieces].")

    def test_i_six_points_give_only_30_not_31(self):
        self.case("Six points give only 30 regions [F-C.evenly_spaced.regions].", "Six points give only 31 regions [F-C.evenly_spaced.regions].")

    def test_i_54_pieces_for_53(self):
        self.case("It is exactly 1.0 after 54 pieces [F-H.float.reaches_one].", "It is exactly 1.0 after 53 pieces [F-H.float.reaches_one].", [allow("1.0", "exactly 1.0", "structural", "the float value")])

    def test_i_3_and_a_half_slabs_for_4_and_a_half(self):
        # the slab count of the 8th cube is 7/2: a "3" written beside the half must be a fact too
        oracle = ORACLE_REAL + "F-K.arm.8.slabs\t3.5\n"
        row = allow("3", "(3½ slabs)", "derived", '=floor(f("F-K.arm.8.slabs"))')
        self.case("The arm is 8 by 28 (3½ slabs) [F-K.arm.8, F-K.arm.8.slabs].", "The arm is 8 by 28 (4½ slabs) [F-K.arm.8, F-K.arm.8.slabs].", [row], oracle=oracle + "F-K.arm.8\t8x28\n")

    # Video K
    def test_k_42_percent_for_43(self):
        self.case("A lone load gives 42% of the drop at 4 m [K-P2.lone_pct.4m].", "A lone load gives 43% of the drop at 4 m [K-P2.lone_pct.4m].", [allow("4", "at 4 m", "structural", "the position of the key")])

    def test_k_5802_per_hertz_for_5803(self):
        self.case("There are 5,802 in every hertz there [K-H2.per_hz_at_1000].", "There are 5,803 in every hertz there [K-H2.per_hz_at_1000].")

    def test_k_315_keys_for_316(self):
        row = allow("315", "(315 keys)", "derived", "=n_facts()")
        n = len(dn.fact_values(ORACLE_REAL))
        self.case(f"The two agree key by key ({n} keys).", f"The two agree key by key ({n + 1} keys).", [dict(row, number=str(n), context=f"({n} keys)")])

    def test_k_the_economy_keeps_35_not_36(self):
        row = allow("35", "economy keeps 35%", "derived", '=(1-(f("K-E5.rounds",2)+f("K-E5.rounds",3))/(f("K-E5.rounds",0)+f("K-E5.rounds",1)))*100')
        self.case("Of (30, 10) the economy keeps 35% [K-E5.rounds].", "Of (30, 10) the economy keeps 36% [K-E5.rounds].", [row])

    # a computed value written inside a formula is still a claim
    def test_a_wrong_entry_inside_a_matrix_formula_fails(self):
        good = "The exact exponential is e^A ≈ [−0.735759, 0.551819; −1.471518, 1.103638] [F-C.mvl.exact]."
        self.case(good, good.replace("0.551819", "0.551820"))
        self.case(good, good.replace("1.103638", "1.103683"))

    def test_a_wrong_decay_rate_inside_an_exponent_fails(self):
        good = "x(t) = 50 + 50 e^{−0.2t} [F-T.eigenvalues]"
        self.case(good, good.replace("0.2t", "0.3t"))

    def test_a_wrong_force_inside_a_product_fails(self):
        rows = [allow("424.2", "= 424.2 kN", "derived", '=2*f("K-T5.forces_kn",5)'), allow("2", "pull 2 ×", "structural", "the two outer diagonals")]
        good = "Two outer diagonals pull 2 × 212.1 = 424.2 kN in all [K-T5.forces_kn]."
        self.case(good, good.replace("212.1", "212.2").replace("424.2", "424.4"), rows)
        # a sum that is consistent but built on a wrong operand is no better
        self.case(good, "Two outer diagonals pull 2 × 212.2 = 424.4 kN in all [K-T5.forces_kn].", rows)

    def test_a_wrong_small_count_inside_a_plain_sum_fails(self):
        tags = "[G-C.g1.joints, G-C.g1.velocity, G-C.g1.stored, G-R.numbers.arrow, G-S.numbers.quaternion]"
        good = f"so 3 + 4 + 29 = 36 numbers of stored state {tags}."
        self.case(good, good.replace("3 + 4 + 29 = 36", "3 + 4 + 28 = 35"))  # 28 is no fact; the sum itself comes out
        self.case(good, good.replace("= 36", "= 35"))  # 35 is a fact (velocity) but the sum does not come out


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
            (docs / "description.md").write_text("# Demo\n\nIt is `8.1` times closer [F-C.pade.m3.times_closer].\n")
            code, out = self.main(root, "demo", "--oracle", str(oracle))
            self.assertEqual(code, 0, out)
            self.assertIn("description-numbers demo: 1 numbers: 1 bound to the facts their text names", out)
            (docs / "description.md").write_text("# Demo\n\nIt is `70.7` times closer [F-C.pade.m3.times_closer].\n")
            code, out = self.main(root, "demo", "--oracle", str(oracle))
            self.assertEqual(code, 1)
            self.assertIn("MISMATCH line 3: 70.7 agrees with none of the facts its text names", out)
            # untagged text is refused unless its allow-list says #! legacy
            (docs / "description.md").write_text("# Demo\n\nIt is `8.1` times closer.\n")
            code, out = self.main(root, "demo", "--oracle", str(oracle))
            self.assertEqual(code, 1, out)
            (docs / "description_numbers.tsv").write_text("#! legacy: not tagged yet\n")
            code, out = self.main(root, "demo", "--oracle", str(oracle))
            self.assertEqual(code, 0, out)
            self.assertIn("LEGACY", out)

    def test_a_legacy_pass_is_worded_as_one_and_fails_without_the_line_for_an_untagged_computed_number(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            docs = root / "explainer_corpus" / "demo" / "docs"
            docs.mkdir(parents=True)
            oracle = root / "oracle.txt"
            oracle.write_text(ORACLE)
            # two computed numbers (the oracle's F-C.mvl.size = 89.6324 and F-C.mvl.term.largest = 9303002.5) in a paragraph naming no key
            (docs / "description.md").write_text("# Demo\n\nThe matrix has size 89.6 here, and the largest term is 9.3 million.\n")
            tsv = docs / "description_numbers.tsv"
            tsv.write_text("#! legacy: not tagged yet\n# number\tcontext\tkind\treason\n")
            code, out = self.main(root, "demo", "--oracle", str(oracle))
            summary = out.strip().splitlines()[-1]
            self.assertEqual(code, 0, out)
            self.assertTrue(summary.startswith("description-numbers demo: LEGACY (not tag-bound) 2 numbers"), summary)
            self.assertIn("PASS", summary)
            tsv.write_text("# number\tcontext\tkind\treason\n")  # the same file with the line removed
            code, out = self.main(root, "demo", "--oracle", str(oracle))
            self.assertEqual(code, 1, out)
            self.assertIn("tag its text with the key it is about or give it a row", out)
            self.assertNotIn("LEGACY", out)
            self.assertIn("2 FAIL", out.strip().splitlines()[-1])


if __name__ == "__main__":
    unittest.main()
