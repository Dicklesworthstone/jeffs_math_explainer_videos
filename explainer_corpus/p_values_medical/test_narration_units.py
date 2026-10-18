#!/usr/bin/env python3
"""narration_units.py's rules, on the real script and on broken copies of
its units: each broken copy changes one thing (its text kept equal to its
lines where the break is not the text) and must fail for that reason.

    python3 -I test_narration_units.py
"""
import contextlib
import copy
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import narration_units as nu  # noqa: E402

SCRIPT_BYTES = nu.SCRIPT.read_bytes()
LINES = nu.parse_script(SCRIPT_BYTES.decode("utf-8"))
TEXT_OF = dict(LINES)


def unit(doc, uid):
    return next(u for u in doc["units"] if u["id"] == uid)


def set_lines(u, ids):
    """Give unit `u` these lines, its text their exact join."""
    u["line_ids"] = list(ids)
    u["text"] = " ".join(TEXT_OF[i] for i in ids)


class TheRealUnits(unittest.TestCase):
    def setUp(self):
        self.doc = nu.build(LINES, SCRIPT_BYTES)

    def test_the_file_on_disk_is_valid_and_current(self):
        on_disk = json.loads(nu.UNITS.read_text(encoding="utf-8"))
        self.assertEqual(nu.problems(on_disk, LINES, SCRIPT_BYTES), [])
        self.assertEqual(nu.dump(on_disk), nu.dump(self.doc))

    def test_every_line_once_in_script_order_chapter_01_first(self):
        said = [i for u in self.doc["units"] for i in u["line_ids"]]
        self.assertEqual(said, [i for i, _ in LINES])
        self.assertEqual(len(said), 56)
        self.assertEqual(self.doc["units"][0]["id"], "01u1")
        # chapter 01's silent title card (01_3) is no line, so no unit says it
        self.assertNotIn("01_3", said)

    # the one unit under 30 words: a one-line request split off at a seam the
    # take ran together (root, Mail 3384)
    SHORT = {"05u2a"}

    def test_units_are_whole_thoughts_within_a_chapter(self):
        for u in self.doc["units"]:
            self.assertEqual({nu.chapter_of(i) for i in u["line_ids"]}, {u["id"][:2]})
            low = 10 if u["id"] in self.SHORT else 30
            self.assertTrue(low <= nu.words(u["text"]) <= 90, u["id"])
            self.assertNotIn("⟨predict⟩", u["text"])
            self.assertNotIn("*(", u["text"])
        # the survey stops at the title card, which the next unit follows
        self.assertEqual(unit(self.doc, "01u1")["line_ids"][-1], "01_2")
        self.assertEqual(unit(self.doc, "01u2")["line_ids"][0], "01_4")

    def test_a_valid_document_is_quiet(self):
        self.assertEqual(nu.problems(self.doc, LINES, SCRIPT_BYTES), [])


class Omissions(unittest.TestCase):
    def setUp(self):
        self.doc = nu.build(LINES, SCRIPT_BYTES)

    def fails(self, doc, needle):
        found = nu.problems(doc, LINES, SCRIPT_BYTES)
        self.assertTrue(any(needle in p for p in found), found)

    def test_a_line_left_out_of_its_unit(self):
        set_lines(unit(self.doc, "01u2"), ["01_4", "01_6", "01_7", "01_8"])
        self.fails(self.doc, "lines in no unit: ['01_5']")

    def test_a_whole_unit_left_out(self):
        self.doc["units"] = [u for u in self.doc["units"] if u["id"] != "06u2"]
        self.fails(self.doc, "lines in no unit: ['06_4', '06_5', '06_6', '06_7']")

    def test_the_last_line_left_out(self):
        set_lines(unit(self.doc, "06u2"), ["06_4", "06_5", "06_6"])
        self.fails(self.doc, "lines in no unit: ['06_7']")

    def test_a_line_said_twice(self):
        set_lines(unit(self.doc, "01u2"), ["01_2", "01_4", "01_5", "01_6", "01_7", "01_8"])
        self.fails(self.doc, "lines in more than one place: ['01_2']")

    def test_a_line_that_is_not_in_the_script(self):
        unit(self.doc, "01u1")["line_ids"].append("01_3")
        self.fails(self.doc, "['01_3'] are not lines of script.tsv")

    def test_no_units(self):
        self.doc["units"] = []
        self.fails(self.doc, "units must be a non-empty list")

    def test_a_unit_with_no_lines(self):
        unit(self.doc, "03u1")["line_ids"] = []
        self.fails(self.doc, "line_ids must be a non-empty list")


class Boundaries(unittest.TestCase):
    def setUp(self):
        self.doc = nu.build(LINES, SCRIPT_BYTES)

    def fails(self, doc, needle, lines=LINES, script_bytes=SCRIPT_BYTES):
        found = nu.problems(doc, lines, script_bytes)
        self.assertTrue(any(needle in p for p in found), found)

    def test_a_unit_across_two_chapters(self):
        # 01_11 then 02_1 is still script order: only the chapter rule sees it
        set_lines(unit(self.doc, "01u3"), ["01_9", "01_10", "01_11", "02_1"])
        set_lines(unit(self.doc, "02u1"), ["02_2", "02_3", "02_4", "02_5", "02_6"])
        found = nu.problems(self.doc, LINES, SCRIPT_BYTES)
        self.assertEqual(len(found), 1, found)
        self.assertIn("holds lines of chapters ['01', '02']", found[0])

    def test_lines_moved_within_a_unit(self):
        set_lines(unit(self.doc, "04u1a"), ["04_1", "04_3", "04_2", "04_4"])
        self.fails(self.doc, "the units leave script order at '04_3', where the script has '04_2'")

    def test_units_swapped(self):
        units = self.doc["units"]
        units[0], units[1] = units[1], units[0]
        self.fails(self.doc, "the units leave script order at '01_4'")

    def test_a_line_cut_short(self):
        u = unit(self.doc, "03u1")
        u["text"] = u["text"].removesuffix(" the treated patient recovered first.")
        self.fails(self.doc, "text differs from its lines joined by one space")

    def test_a_line_cut_mid_sentence_ends_no_sentence(self):
        u = unit(self.doc, "05u3")
        u["text"] = u["text"].removesuffix(" would.")
        self.assertTrue(u["text"].endswith("A patient"))
        self.fails(self.doc, "text must end a sentence")

    def test_a_paraphrase(self):
        u = unit(self.doc, "04u2")
        u["text"] = u["text"].replace("about two percent", "roughly two percent")
        self.fails(self.doc, "where the script has 'about two percent")

    def test_a_spoken_stage_direction(self):
        u = unit(self.doc, "01u2")
        u["text"] = u["text"].replace("you have it? ", "you have it? (pause) ")
        self.fails(self.doc, "text differs")

    def test_lines_joined_by_two_spaces(self):
        u = unit(self.doc, "02u2")
        u["text"] = "  ".join(TEXT_OF[i] for i in u["line_ids"])
        self.fails(self.doc, "text differs")

    def test_a_trailing_space(self):
        unit(self.doc, "06u1")["text"] += " "
        self.fails(self.doc, "text differs")

    def test_a_hundred_words_pass_and_a_hundred_and_one_fail(self):
        for n in (100, 101):
            text = f"01_1\t{' '.join(['one'] * 60)}.\n01_2\t{' '.join(['two'] * (n - 60))}.\n"
            raw, lines = text.encode(), nu.parse_script(text)
            # each line alone is a valid unit; together they are one of n words
            self.assertEqual(len(nu.build(lines, raw, [("01_1",), ("01_2",)])["units"]), 2)
            if n == 100:
                self.assertEqual(nu.words(nu.build(lines, raw, [("01_1", "01_2")])["units"][0]["text"]), 100)
                continue
            with self.assertRaises(ValueError) as e:
                nu.build(lines, raw, [("01_1", "01_2")])
            self.assertIn("101 words, more than 100", str(e.exception))

    def test_the_wrong_ordinal(self):
        unit(self.doc, "02u2")["id"] = "02u3"
        self.fails(self.doc, "the id must be '02u2'")

    def test_the_wrong_chapter_in_an_id(self):
        unit(self.doc, "03u1")["id"] = "02u3"
        self.fails(self.doc, "the id must be '03u1'")

    def test_the_two_split_units_keep_their_places(self):
        ids = [u["id"] for u in self.doc["units"]]
        self.assertEqual(ids[ids.index("04u1a"):ids.index("04u1a") + 3], ["04u1a", "04u1b", "04u2"])
        self.assertEqual(ids[ids.index("05u2a"):ids.index("05u2a") + 3], ["05u2a", "05u2b", "05u3"])
        self.assertEqual(unit(self.doc, "04u1a")["line_ids"][-1], "04_4")
        self.assertEqual(unit(self.doc, "04u1b")["line_ids"], ["04_5", "04_6"])
        self.assertEqual(unit(self.doc, "05u2a")["line_ids"], ["05_5"])

    def test_a_split_unit_with_one_part(self):
        u = unit(self.doc, "04u1b")
        set_lines(unit(self.doc, "04u1a"), unit(self.doc, "04u1a")["line_ids"] + u["line_ids"])
        self.doc["units"].remove(u)
        self.fails(self.doc, "split unit 04u1 has only part 'a'")

    def test_a_split_part_out_of_order(self):
        unit(self.doc, "04u1a")["id"] = "04u1b"
        self.fails(self.doc, "the id must be '04u1'")

    def test_a_skipped_split_letter(self):
        unit(self.doc, "05u2b")["id"] = "05u2c"
        self.fails(self.doc, "the id must be '05u3'")

    def test_a_split_part_named_for_another_chapter(self):
        unit(self.doc, "04u1b")["id"] = "05u1b"
        self.fails(self.doc, "or '04u2a', '04u1b' for a split unit's part")

    def test_an_id_without_its_chapter_digits(self):
        unit(self.doc, "01u1")["id"] = "1u1"
        self.fails(self.doc, "the id must be '01u1'")

    def test_a_changed_script_is_refused(self):
        changed = SCRIPT_BYTES.replace(b"about two percent", b"about 2 percent", 1)
        self.assertNotEqual(changed, SCRIPT_BYTES)
        lines = nu.parse_script(changed.decode("utf-8"))
        found = nu.problems(self.doc, lines, changed)
        self.assertTrue(any("script_sha256" in p for p in found), found)
        self.assertTrue(any("text differs" in p for p in found), found)

    def test_a_stale_hash_alone(self):
        self.doc["script_sha256"] = "0" * 64
        self.fails(self.doc, "is not script.tsv's")

    def test_another_schema(self):
        for bad in (2, True, "1"):
            doc = copy.deepcopy(self.doc)
            doc["schema_version"] = bad
            self.fails(doc, "schema_version is")

    def test_unknown_keys(self):
        doc = copy.deepcopy(self.doc)
        doc["note"] = "x"
        self.fails(doc, "exactly the keys")
        doc = copy.deepcopy(self.doc)
        doc["units"][0]["start_s"] = 0.0
        self.fails(doc, "unit 1: must be an object with exactly the keys")


class TheGenerator(unittest.TestCase):
    def check(self, doc_text):
        """`narration_units.py --check` on a units file holding `doc_text`."""
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "narration_units.json"
            path.write_text(doc_text, encoding="utf-8")
            saved, nu.UNITS = nu.UNITS, path
            try:
                with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()) as err:
                    code = nu.main(["--check"])
            finally:
                nu.UNITS = saved
        return code, err.getvalue()

    def test_the_check_fails_on_a_broken_or_regrouped_file(self):
        doc = nu.build(LINES, SCRIPT_BYTES)
        self.assertEqual(self.check(nu.dump(doc))[0], 0)
        broken = copy.deepcopy(doc)
        set_lines(broken["units"][1], ["01_4", "01_6", "01_7", "01_8"])
        code, err = self.check(nu.dump(broken))
        self.assertEqual(code, 1)
        self.assertIn("FAIL lines in no unit: ['01_5']", err)
        # valid, but not the grouping GROUPS chose: stale
        regrouped = copy.deepcopy(doc)
        set_lines(regrouped["units"][0], ["01_1"])
        regrouped["units"].insert(1, {"id": "", "line_ids": ["01_2"], "text": TEXT_OF["01_2"]})
        per_chapter = {}
        for u in regrouped["units"]:
            c = nu.chapter_of(u["line_ids"][0])
            per_chapter[c] = per_chapter.get(c, 0) + 1
            u["id"] = f"{c}u{per_chapter[c]}"
        self.assertEqual(nu.problems(regrouped, LINES, SCRIPT_BYTES), [])
        code, err = self.check(nu.dump(regrouped))
        self.assertEqual(code, 1)
        self.assertIn("rerun python3 -I narration_units.py", err)
        self.assertEqual(self.check("{not json")[0], 1)

    def test_a_new_line_must_be_placed_by_hand(self):
        text = SCRIPT_BYTES.decode("utf-8") + "06_8\tOne more line.\n"
        with self.assertRaises(ValueError) as e:
            nu.build(nu.parse_script(text), text.encode())
        self.assertIn("lines in no unit: ['06_8']", str(e.exception))

    def test_the_script_is_read_as_the_kit_reads_it(self):
        self.assertEqual(nu.parse_script("# a\n\n01_1\t Words. \r\n"), [("01_1", "Words.")])
        for bad, why in [("01_1 Words.\n", "no tab"), ("01_1\tA\tB\n", "stray tab"),
                         ("01_1\t \n", "empty id or text"), ("01_1\tA.\n01_1\tB.\n", "repeats")]:
            with self.assertRaises(ValueError) as e:
                nu.parse_script(bad)
            self.assertIn(why, str(e.exception))


if __name__ == "__main__":
    unittest.main()
