"""Regression for help/typo invocations overwriting every topic's script."""
import contextlib
import importlib.util
import io
from pathlib import Path
import tempfile
import unittest


class StoryboardCli(unittest.TestCase):
    def setUp(self):
        spec = importlib.util.spec_from_file_location("storyboard_cli_under_test", Path(__file__).with_name("storyboard.py"))
        self.tool = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.tool)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.tool.ROOT = Path(self.tmp.name)
        self.tool.CORPUS = self.tool.ROOT / "explainer_corpus"
        for slug in ("alpha", "beta"):
            topic = self.tool.CORPUS / slug
            (topic / "docs").mkdir(parents=True)
            (topic / "docs/storyboard.md").write_text(
                "## 01_question: A question\n"
                "| 01_1 | What do you predict? ⟨predict⟩ | The question | card | |\n"
            )
            (topic / "script.tsv").write_text(f"preserved {slug} take\n")
        self.before = self.contents()

    def contents(self):
        return {p.relative_to(self.tool.ROOT).as_posix(): p.read_bytes()
                for p in self.tool.ROOT.rglob("*") if p.is_file()}

    def invoke(self, argv):
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            return self.tool.main(argv)

    def test_help_and_invalid_invocations_never_write(self):
        for argv, code in [(["--help"], 0), (["-h"], 0), (["--chek"], 2),
                           ([], 2), (["alpha", "typo"], 2), (["--all", "alpha"], 2),
                           (["../outside"], 2)]:
            with self.subTest(argv=argv):
                with self.assertRaises(SystemExit) as raised:
                    self.invoke(argv)
                self.assertEqual(raised.exception.code, code)
                self.assertEqual(self.contents(), self.before)

    def test_check_all_reports_stale_without_writing(self):
        self.assertEqual(self.invoke(["--check"]), 1)
        self.assertEqual(self.contents(), self.before)

    def test_explicit_topic_changes_only_its_script(self):
        self.assertEqual(self.invoke(["alpha"]), 0)
        after = self.contents()
        changed = [p for p in after if after[p] != self.before[p]]
        self.assertEqual(changed, ["explainer_corpus/alpha/script.tsv"])
        self.assertEqual(self.invoke(["--check", "alpha"]), 0)
        self.assertEqual(self.contents(), after)

    def test_explicit_all_regenerates_both(self):
        self.assertEqual(self.invoke(["--all"]), 0)
        self.assertEqual(self.invoke(["--check"]), 0)
        after = self.contents()
        self.assertEqual(sorted(p for p in after if after[p] != self.before[p]),
                         ["explainer_corpus/alpha/script.tsv", "explainer_corpus/beta/script.tsv"])

    def long_board(self):
        explanation = "Follow the same object and compare what changes. " * 40
        return (
            "## 01_question: Give the explanation time\n"
            f"| 01_1 | {explanation} | Same objects, controlled experiment | live plot | listen 5 s |\n"
            "| 01_2 | What changes now? ⟨predict⟩ | Unanswered choices | card | |\n"
        )

    def test_long_chapter_preserves_words_and_reports_advisory(self):
        board = self.tool.CORPUS / "alpha/docs/storyboard.md"
        board.write_text(self.long_board())
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            self.assertEqual(self.tool.main(["alpha"]), 0)
        self.assertIn("ADVISORY", output.getvalue())
        script = self.tool.CORPUS / "alpha/script.tsv"
        self.assertEqual(script.read_text().count("Follow the same object and compare what changes."), 40)
        self.assertEqual(board.read_text(), self.long_board())
        self.assertEqual((self.tool.CORPUS / "beta/script.tsv").read_text(), "preserved beta take\n")
        before = self.contents()
        self.assertEqual(self.invoke(["--check", "alpha"]), 0)
        self.assertEqual(self.contents(), before)
        script.write_text(script.read_text().replace("Follow", "Dropped", 1))
        stale = self.contents()
        self.assertEqual(self.invoke(["--check", "alpha"]), 1)
        self.assertEqual(self.contents(), stale)

    def test_long_chapter_keeps_structural_failures(self):
        board = self.tool.CORPUS / "alpha/docs/storyboard.md"
        valid = self.long_board()
        for name, broken in [
            ("duplicate id", valid.replace("01_2", "01_1")),
            ("wrong chapter", valid.replace("01_2", "02_2")),
            ("missing picture", valid.replace("Same objects, controlled experiment", "")),
            ("no prediction", valid.replace("⟨predict⟩", "")),
            ("answer before hold", valid.replace("What changes now?", "The answer is already here.")),
        ]:
            with self.subTest(name=name):
                board.write_text(broken)
                chapters = self.tool.parse(board)
                (self.tool.CORPUS / "alpha/script.tsv").write_text(self.tool.script_text("alpha", chapters))
                before = self.contents()
                self.assertEqual(self.invoke(["--check", "alpha"]), 1)
                self.assertEqual(self.contents(), before)


if __name__ == "__main__":
    unittest.main()
