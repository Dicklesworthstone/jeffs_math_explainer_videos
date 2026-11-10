"""Inspect/construct packaged scenes through the shipped entrypoint.

Scene facts and raster output are fixtures; Python imports, lifetime, command
routing and stdout isolation execute production code.
"""
from __future__ import annotations

import contextlib
import importlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

import test_console_rendering_protocol as fixtures

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))
from fmn_python import __main__ as entrypoint
from fmn_python.scene_controls import try_scene_cli


class SceneControlsCli(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.native = fixtures.native_fixture()
        self.native._native = self.native
        self.native._console_main = lambda: 77
        self.native.Scene._engine_facts = lambda scene: (1, 3, [-1, -2, 0], [1, 2, 0])
        self.native.Scene.time = property(lambda scene: 1.25)
        modules = patch.dict(sys.modules, {"manimlib": self.native})
        modules.start()
        self.addCleanup(modules.stop)
        exclusive = patch.object(entrypoint, "_ensure_exclusive_manimlib_namespace")
        exclusive.start()
        self.addCleanup(exclusive.stop)
        package = self.root / "inspect_project"
        package.mkdir()
        (package / "__init__.py").write_text("print('package output')\n")
        (package / "helper.py").write_text("from manimlib import Scene\nVALUE = 17\nclass Imported(Scene): pass\n")
        self.source = package / "scenes.py"
        self.source.write_text(
            "from manimlib import Scene\nfrom .helper import Imported\n"
            "print('module output')\n"
            "class A(Scene):\n"
            "    def construct(self):\n"
            "        from .helper import VALUE\n"
            "        from . import scenes\n"
            "        assert scenes.A is A and VALUE == 17\n"
            "        print('construct output')\n"
            "    def tear_down(self):\n"
            "        import sys\n"
            "        assert sys.modules[__name__].A is A\n"
            "        print('teardown output')\n"
            "        super().tear_down()\n"
        )

    def invoke(self, *arguments):
        out, err = io.StringIO(), io.StringIO()
        before_path, before_finders = list(sys.path), list(sys.meta_path)
        with patch.object(sys, "argv", ["fmn-python", *map(str, arguments)]), \
                contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = entrypoint.main()
        self.assertEqual(sys.path, before_path)
        self.assertEqual(sys.meta_path, before_finders)
        self.assertFalse(any(name == "inspect_project" or name.startswith("inspect_project.")
                             for name in sys.modules))
        text = out.getvalue()
        report = json.loads(text) if text.startswith("{") else None
        if report is not None:
            self.assertEqual(len(text.splitlines()), 1)
        return code, report, text, err.getvalue()

    def test_list_is_sorted_local_only_and_never_constructs(self):
        with self.source.open("a") as file:
            file.write("class Z(Scene): pass\nclass B(Scene): pass\n")
        code, report, _, err = self.invoke("--robot", "--list-scenes", self.source)
        self.assertEqual(code, 0, report)
        self.assertEqual(report["kind"], "scene-list")
        self.assertEqual(report["scenes"], ["A", "B", "Z"])
        self.assertIn("package output", err)
        self.assertIn("module output", err)
        self.assertEqual(self.native.instances, [])

    def test_construct_keeps_identity_through_teardown_without_rendering(self):
        code, report, _, err = self.invoke("--robot", "--construct-only", self.source)
        self.assertEqual(code, 0, report)
        self.assertEqual(report["kind"], "construct-only")
        self.assertFalse(report["rendered"])
        self.assertEqual(report["scene_time"], 1.25)
        self.assertEqual((report["root_count"], report["family_count"]), (1, 3))
        self.assertEqual(report["bounds_low"], [-1.0, -2.0, 0.0])
        self.assertEqual(self.native.instances[0].events, ["run", "tear_down"])
        self.assertIn("construct output", err)
        self.assertIn("teardown output", err)

    def test_named_construct_selects_without_running_other_scenes(self):
        with self.source.open("a") as file:
            file.write("class Z(Scene):\n    def __init__(self): raise AssertionError('not selected')\n")
        code, report, _, _ = self.invoke("--robot", "--construct-only", self.source, "A")
        self.assertEqual(code, 0, report)
        self.assertEqual(report["scene"], "A")
        self.assertEqual(len(self.native.instances), 1)

    def test_ambiguous_and_missing_selections_construct_nothing(self):
        with self.source.open("a") as file:
            file.write("class B(Scene): pass\n")
        for selection in ((), ("Missing",), ("Imported",)):
            with self.subTest(selection=selection):
                code, report, _, _ = self.invoke("--robot", "--construct-only", self.source, *selection)
                self.assertEqual(code, 5)
                self.assertEqual(report["phase"], "select")
                self.assertEqual(self.native.instances, [])

    def test_end_scene_is_normal_completion(self):
        self.source.write_text("from manimlib import Scene, EndScene\nclass A(Scene):\n    def construct(self): raise EndScene('done')\n")
        code, report, _, _ = self.invoke("--robot", "--construct-only", self.source)
        self.assertEqual(code, 0, report)
        self.assertEqual(self.native.instances[0].events, ["run", "tear_down"])

    def test_source_interrupts_are_nonzero_terminal_receipts(self):
        for control in ("--list-scenes", "--construct-only"):
            for exception, expected in (("SystemExit(0)", 5), ("KeyboardInterrupt()", 130)):
                with self.subTest(control=control, exception=exception):
                    self.source.write_text("print('before interrupt')\nraise " + exception + "\n")
                    code, report, _, err = self.invoke("--robot", control, self.source)
                    self.assertEqual(code, expected, report)
                    self.assertEqual(report["phase"], "load")
                    self.assertFalse(report["rendered"])
                    self.assertIn("before interrupt", err)
                    self.assertEqual(self.native.instances, [])

    def test_scene_interrupt_runs_teardown_and_returns_130(self):
        self.source.write_text("from manimlib import Scene\nclass A(Scene):\n    def construct(self): raise KeyboardInterrupt()\n")
        code, report, _, _ = self.invoke("--robot", "--construct-only", self.source)
        self.assertEqual(code, 130, report)
        self.assertEqual(report["kind"], "construct-interrupted")
        self.assertEqual(report["phase"], "execute")
        self.assertEqual(self.native.instances[0].events, ["run", "tear_down"])

    def test_load_errors_keep_robot_stdout_parseable(self):
        self.source.write_text("print('before failure')\nraise ValueError('bad source')\n")
        code, report, _, err = self.invoke("--robot", "--list-scenes", self.source)
        self.assertEqual(code, 5)
        self.assertIn("bad source", report["message"])
        self.assertIn("before failure", err)

    def test_empty_source_lists_nothing(self):
        self.source.write_text("from manimlib import Scene\n")
        code, report, _, _ = self.invoke("--robot", "--list-scenes", self.source)
        self.assertEqual(code, 0)
        self.assertEqual(report["scenes"], [])

    def test_invalid_controls_refuse_before_source_execution(self):
        self.source.write_text("raise AssertionError('must not execute')\n")
        for extra in (("--construct-only",), ("--list-scenes",), ("--robot",),
                      ("--fps", "30"), ("--version",), ("A",)):
            with self.subTest(extra=extra):
                code, report, _, _ = self.invoke("--robot", "--list-scenes", self.source, *extra)
                self.assertEqual(code, 2, report)
                self.assertEqual(report["kind"], "usage-error")
        self.assertEqual(self.native.events, [])

    def test_missing_and_extra_arguments_are_usage_errors(self):
        for arguments in (("--list-scenes",), ("--construct-only",),
                          ("--construct-only", self.source, "A", "B")):
            self.assertEqual(self.invoke("--robot", *arguments)[0], 2)

    def test_version_and_studio_remain_native_owned(self):
        self.assertEqual(self.invoke("--robot", "--version")[0], 77)
        self.assertEqual(self.invoke("--robot", "studio", "--construct-only")[0], 2)

    def test_switch_like_option_values_do_not_select_a_control(self):
        self.assertIsNone(try_scene_cli(self.native, [str(self.source), "--video_dir", "--list-scenes"]))
        self.assertEqual(self.native.events, [])

    def test_text_mode_preserves_authored_stdout(self):
        code, _, out, _ = self.invoke("--list-scenes", self.source)
        self.assertEqual(code, 0)
        self.assertIn("module output", out)
        self.assertIn("A", out)


if __name__ == "__main__":
    unittest.main()
