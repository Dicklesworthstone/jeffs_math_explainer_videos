#!/usr/bin/env python3
"""Tests of tools/youtube_package.py on a fixture package and fixture timelines. Nothing is deleted."""

import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))

import youtube_package as yp  # noqa: E402

DESC = """A one-line pitch.

Chapters
?:?? One
?:?? Two
?:?? Three

Narration: synthetic speech (FrankenTTS).
"""


class Package(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="youtube_package_"))
        self.pub = self.root / "explainer_corpus" / "alpha" / "publish"
        (self.pub / "michael").mkdir(parents=True)
        self.finals = self.root / "final"
        self.finals.mkdir()
        # three chapters of 61.5 s, 47.25 s and 30 s at 60 fps: starts 0, 61, 108
        for key, frames in (("01_one", 3690), ("02_two", 2835), ("03_three", 1800)):
            (self.finals / f"{key}.timeline.tsv").write_text(
                f"# explainer timeline v1\nchapter\t{key}\nfps\t60\nframes\t{frames}\n", encoding="utf-8"
            )
        self.write(DESC)

    def tearDown(self):
        shutil.move(str(self.root), tempfile.gettempdir() + f"/{self.root.name}.done")

    def write(self, desc, title="Alpha: A Title", tags="alpha, beta gamma, math explainer", yt_desc=None):
        (self.pub / "michael" / "title.txt").write_text(title + "\n", encoding="utf-8")
        (self.pub / "michael" / "tags.txt").write_text(tags + "\n", encoding="utf-8")
        (self.pub / "michael" / "description.txt").write_text(desc, encoding="utf-8")
        block = desc if yt_desc is None else yt_desc
        (self.pub / "youtube.md").write_text(
            f"# Alpha: YouTube package\n\n- Title: {title} (exactly this)\n- Tags: {tags}\n\n"
            f"## Description\n\n```text\n{block}```\n",
            encoding="utf-8",
        )

    def check(self, finals=None):
        return yp.check("alpha", "michael", finals, self.root)

    def test_an_unfilled_package_fails_until_its_times_come_from_the_final(self):
        bad = self.check()
        self.assertEqual(sum("time not filled" in b for b in bad), 3, bad)
        self.assertEqual(yp.fill("alpha", self.finals, "michael", self.root), [])
        desc = (self.pub / "michael" / "description.txt").read_text(encoding="utf-8")
        self.assertIn("0:00 One\n1:01 Two\n1:48 Three\n", desc)
        self.assertEqual(self.check(self.finals), [])
        # the copy in youtube.md was filled too
        self.assertIn("1:48 Three", (self.pub / "youtube.md").read_text(encoding="utf-8"))

    def test_the_starts_are_the_ones_x_chapters_prints(self):
        self.assertEqual(yp.final_starts(self.finals), [("01_one", 0), ("02_two", 61), ("03_three", 108)])
        self.assertEqual(yp.clock(108), "1:48")

    def test_times_that_are_not_the_finals_fail(self):
        self.write(DESC.replace("?:?? One", "0:00 One").replace("?:?? Two", "1:02 Two").replace("?:?? Three", "1:48 Three"))
        self.assertEqual(self.check(), [])
        bad = self.check(self.finals)
        self.assertEqual(len(bad), 1, bad)
        self.assertIn("02_two starts at 1:01", bad[0])

    def test_padded_minutes_match_the_final_and_survive_fill(self):
        desc = DESC.replace("?:?? One", "00:00 One").replace("?:?? Two", "01:01 Two").replace("?:?? Three", "01:48 Three")
        self.write(desc)
        self.assertEqual(self.check(self.finals), [])
        self.assertEqual(yp.fill("alpha", self.finals, "michael", self.root), [])
        self.assertEqual((self.pub / "michael" / "description.txt").read_text(encoding="utf-8"), desc)
        self.assertEqual(self.check(self.finals), [])

    def test_padding_does_not_hide_a_wrong_start_or_invalid_seconds(self):
        for wrong in ("01:02", "00:61"):
            with self.subTest(timestamp=wrong):
                self.write(DESC.replace("?:?? One", "00:00 One")
                           .replace("?:?? Two", f"{wrong} Two").replace("?:?? Three", "01:48 Three"))
                bad = self.check(self.finals)
                self.assertEqual(len(bad), 1, bad)
                self.assertIn("02_two starts at 1:01", bad[0])

    def test_youtubes_limits(self):
        self.write(DESC + "x" * 5000)
        self.assertTrue(any("5000" in b for b in self.check()))
        self.write(DESC.replace("A one-line pitch.", "A <b>pitch</b>."))
        self.assertTrue(any("angle brackets" in b for b in self.check()))
        self.write(DESC, title="T" * 101)
        self.assertTrue(any("100" in b for b in self.check()))
        self.write(DESC, tags=", ".join(f"tag number {i}" for i in range(60)))
        self.assertTrue(any("500" in b for b in self.check()))

    def test_chapters_start_at_zero_and_stand_ten_seconds_apart(self):
        self.write(DESC.replace("?:?? One", "0:05 One").replace("?:?? Two", "0:20 Two").replace("?:?? Three", "0:25 Three"))
        bad = self.check()
        self.assertTrue(any("0:00" in b for b in bad), bad)
        self.assertTrue(any("5 s after" in b for b in bad), bad)

    def test_the_block_must_be_the_description_byte_for_byte(self):
        self.write(DESC, yt_desc=DESC.replace("pitch", "pitch!"))
        self.assertTrue(any("byte for byte" in b for b in self.check()))
        self.assertTrue(yp.fill("alpha", self.finals, "michael", self.root))  # refuses, says why

    def test_a_missing_file_is_named(self):
        (self.pub / "michael" / "tags.txt").rename(self.pub / "michael" / "tags.txt.aside")
        bad = self.check()
        self.assertEqual(len(bad), 1)
        self.assertIn("tags.txt: missing", bad[0])

    def test_a_protected_phrase_the_upload_text_dropped_fails(self):
        docs = self.root / "explainer_corpus" / "alpha" / "docs"
        docs.mkdir()
        (docs / "protected.tsv").write_text(
            "# phrase\twhere\twhy\n"
            "von Neumann\tdescription\twhose trick it is\n"
            "a title word\ttitle\tthe title keeps it\n"
            "only spoken\tscript\tnot the upload text's to keep\n",
            encoding="utf-8",
        )
        bad = [b for b in self.check() if "protected" in b]
        self.assertEqual(len(bad), 2, bad)
        self.assertIn("'von Neumann'", bad[0])
        self.assertIn("title.txt", bad[1])
        # case, punctuation and line breaks do not matter; whole words do
        self.write(DESC.replace("A one-line pitch.", "Von\nNEUMANN's pitch."), title="Alpha: A Title-Word")
        self.assertEqual([b for b in self.check() if "protected" in b], [])
        self.write(DESC.replace("A one-line pitch.", "von Neumannian pitch."), title="Alpha: a title word")
        self.assertEqual(len([b for b in self.check() if "protected" in b]), 1)

    def test_a_number_the_notes_do_not_say_fails(self):
        docs = self.root / "explainer_corpus" / "alpha" / "docs"
        docs.mkdir()
        (docs / "description.md").write_text(
            "The coin lands heads 80% of the time; 350,757 flips; a peak of −1.5 dBTP.\n", encoding="utf-8")
        body = ("A coin lands heads 80% of the time, in 350757 flips, with a peak of -1.5 dBTP, and 68% match.\n"
                "See https://github.com/x/y/blob/main/docs/description2024.md\n")
        self.write(DESC.replace("A one-line pitch.\n", body).replace("?:?? One", "0:00 One")
                   .replace("?:?? Two", "1:01 Two").replace("?:?? Three", "1:48 Three"))
        # the notes do not bind their numbers yet: the rule does not apply
        self.assertEqual(self.check(), [])
        (docs / "description_numbers.tsv").write_text("# number\tcontext\tkind\treason\n", encoding="utf-8")
        bad = self.check()
        self.assertEqual(len(bad), 1, bad)  # 80, 350757 (separator folded), 1.5 (sign folded) pass; links, times skip
        self.assertIn("the number 68 ", bad[0])
        self.write(DESC, title="Alpha in 3 Steps")
        self.assertTrue(any(b.startswith("title.txt: the number 3 ") for b in self.check()))

    def test_youtube_counts_quotes_round_a_tag_with_a_space(self):
        self.assertEqual(yp.tags_length(["ab", "c d"]), 2 + 1 + 3 + 2)


if __name__ == "__main__":
    unittest.main()
