"""Exercise console selection/source loading with the checkpoint protocol double."""
import contextlib
import importlib
import io
import pathlib
import sys
import types
import unittest
from unittest.mock import patch

import test_batch_checkpoint_helpers as fixtures

console = importlib.import_module(fixtures.PACKAGE + ".console_rendering")
cli = importlib.import_module(fixtures.PACKAGE + ".checkpoint_cli")


class RecoveryCliTests(unittest.TestCase):
    def setUp(self):
        self.case = fixtures.CheckpointTests()
        self.case.setUp()
        self.addCleanup(self.case.doCleanups)
        self.source = self.case.root / "scenes.py"
        self.source.write_text("from manimlib import Scene\nclass A(Scene): pass\nclass B(Scene): pass\n")
        self.messages = []
        self.native = types.SimpleNamespace(Scene=fixtures.Scene,
                                           _portal_cli_emit=self.emit,
                                           _portal_cli_render_arguments=self.parse)
        self.base = [str(self.source), "--write_all", "--robot", "--format", "png",
                     "--video_dir", str(self.case.output), "--checkpoint", str(self.case.journal),
                     "--resume-key", "inputs-v1"]

    def emit(self, code, identity, kind, message, robot, **details):
        self.messages.append(dict(code=code, kind=kind, message=message, **details))
        return code

    def parse(self, arguments):
        options = dict(format="png", video_dir=None)
        positionals, index = [], 0
        while index < len(arguments):
            value = arguments[index]
            if value in {"--format", "--video_dir"}:
                options[value[2:]] = arguments[index + 1]
                index += 2
            elif value.startswith("-"):
                raise ValueError("unconsumed option: " + value)
            else:
                positionals.append(value)
                index += 1
        return positionals, options, 96, 54, 8, 1

    def invoke(self, arguments=None):
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            return console.try_render_cli(self.native, self.base if arguments is None else arguments)

    def test_write_all_resume_reuses_publications(self):
        self.assertEqual(self.invoke(), 0, self.messages)
        original = self.messages[-1]["batch"]
        self.case.calls.clear()
        self.assertEqual(self.invoke(self.base + ["--resume"]), 0, self.messages)
        self.assertEqual(self.case.calls, [])
        self.assertEqual(self.messages[-1]["batch"], original)

    def test_explicit_selection_supports_recovery(self):
        arguments = [str(self.source), "B", "A", *self.base[2:]]
        self.assertEqual(self.invoke(arguments), 0, self.messages)
        self.assertEqual([row["name"] for row in self.messages[-1]["batch"]["outcomes"]], ["B", "A"])
        self.case.calls.clear()
        self.assertEqual(self.invoke(arguments + ["--resume"]), 0, self.messages)
        self.assertEqual(self.case.calls, [])

    def test_damaged_output_is_an_error_not_a_new_render(self):
        self.assertEqual(self.invoke(), 0, self.messages)
        (self.case.output / "B.png").write_bytes(b"damaged")
        self.case.calls.clear()
        self.assertEqual(self.invoke(self.base + ["--resume"]), 2, self.messages)
        self.assertIn("modified", self.messages[-1]["message"])
        self.assertEqual(self.case.calls, [])

    def test_single_scene_rejects_checkpoint_before_loading_source(self):
        self.source.write_text("raise AssertionError('must not load')\n")
        arguments = [str(self.source), "A", *self.base[2:]]
        self.assertEqual(self.invoke(arguments), 2, self.messages)
        self.assertIn("multiple scene", self.messages[-1]["message"])
        self.assertEqual(self.case.calls, [])

    def test_missing_key_and_duplicate_switches_are_usage_errors(self):
        for args in (self.base[:-2], self.base + ["--resume", "--resume"],
                     self.base + ["--checkpoint", "another.json"]):
            with self.subTest(args=args):
                self.assertEqual(self.invoke(args), 2, self.messages)
        self.assertEqual(self.case.calls, [])

    def test_paths_freeze_before_source_changes_working_directory(self):
        previous = pathlib.Path.cwd()
        import os
        self.addCleanup(os.chdir, previous)
        os.chdir(self.case.root)
        elsewhere = self.case.root / "elsewhere"
        elsewhere.mkdir()
        self.source.write_text(f"import os\nos.chdir({str(elsewhere)!r})\nfrom manimlib import Scene\nclass A(Scene): pass\nclass B(Scene): pass\n")
        arguments = ["scenes.py", "--write_all", "--checkpoint", "progress.json", "--resume-key", "v1",
                     "--video_dir", "outputs", "--format", "png"]
        self.assertEqual(self.invoke(arguments), 0, self.messages)
        self.assertTrue(self.case.journal.is_file())
        self.assertTrue((self.case.output / "A.png").is_file())
        self.assertFalse((elsewhere / "progress.json").exists())

    def test_checkpoint_values_are_not_control_switches(self):
        _, positionals, switches = console._tokens([
            "scenes.py", "--write_all", "--checkpoint", "--version", "--resume-key", "--audit-parity"])
        self.assertEqual(positionals, ["scenes.py"])
        self.assertEqual(switches, ["--write_all"])

    def test_native_option_values_are_not_stolen(self):
        arguments = ["--video_dir", "--resume", "--checkpoint", "progress.json", "--resume-key", "key"]
        remaining, options = cli.take_checkpoint_options(arguments, console._VALUE_FLAGS)
        self.assertEqual(remaining, ["--video_dir", "--resume"])
        self.assertFalse(options["resume"])
        self.assertEqual(options["resume_key"], "key")


if __name__ == "__main__":
    unittest.main()
