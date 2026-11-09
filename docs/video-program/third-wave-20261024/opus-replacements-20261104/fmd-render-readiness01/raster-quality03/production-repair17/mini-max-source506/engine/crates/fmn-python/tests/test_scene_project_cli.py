"""Project imports through production CLI owners, with a fixture output sink.

Python loading and scene execution are real; these tests do not assert native
rasterization, encoding, or certified input closure.
"""
from __future__ import annotations

import contextlib
import importlib
import io
import json
import os
from pathlib import Path
import py_compile
import sys
import tempfile
import unittest
from unittest.mock import patch

import test_console_rendering_protocol as fixtures


class SceneProjectCli(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.native = fixtures.native_fixture()
        modules = patch.dict(sys.modules, {"manimlib": self.native})
        modules.start()
        self.addCleanup(modules.stop)
        self.path_before = list(sys.path)
        self.finders_before = list(sys.meta_path)
        self.source = self.write("lesson_project/scenes.py", "")
        self.write("lesson_project/__init__.py", "print('package initialization')\n")
        self.write("lesson_project/helper.py", "VALUE = 17\n")

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def invoke(self, *extra, legacy=False, destination="out"):
        handler = fixtures.console.try_render_cli
        if legacy:
            handler = importlib.import_module(fixtures.PACKAGE + ".batch_cli").try_batch_cli
        out, err = io.StringIO(), io.StringIO()
        arguments = ["--robot", str(self.source), "--format", "png",
                     "--video_dir", str(self.root / destination), *extra]
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = handler(self.native, arguments)
        self.assertEqual(len(out.getvalue().splitlines()), 1, out.getvalue())
        self.assertEqual(sys.path, self.path_before)
        self.assertEqual(sys.meta_path, self.finders_before)
        self.assertFalse(any(name == "lesson_project" or name.startswith("lesson_project.")
                             for name in sys.modules))
        return code, json.loads(out.getvalue()), err.getvalue()

    def project(self, second=False):
        self.source.write_text(
            "from __future__ import annotations\n"
            "from dataclasses import dataclass\n"
            "from manimlib import Scene, events\n"
            "from .helper import VALUE\n"
            "events.append(('source',))\n"
            "@dataclass\nclass Model:\n    value: int = VALUE\n"
            "class A(Scene):\n"
            "    def construct(self):\n"
            "        import pickle, sys\n"
            "        from . import scenes\n"
            "        assert scenes is sys.modules[__name__]\n"
            "        assert scenes.A is A\n"
            "        assert pickle.loads(pickle.dumps(Model())).value == 17\n"
            "    def tear_down(self):\n"
            "        from .late import READY\n"
            "        assert READY\n"
            "        super().tear_down()\n"
            + ("class B(A): pass\n" if second else ""), encoding="utf-8",
        )
        self.write("lesson_project/late.py", "READY = True\n")

    def test_single_scene_keeps_package_identity_through_teardown(self):
        self.project()
        code, report, err = self.invoke()
        self.assertEqual(code, 0, report)
        self.assertEqual(report["kind"], "render")
        self.assertIn("package initialization", err)
        self.assertTrue(self.native.instances[0].published)

    def test_named_batch_uses_one_live_source_in_requested_order(self):
        self.project(second=True)
        code, report, _ = self.invoke("B", "A")
        self.assertEqual(code, 0, report)
        self.assertEqual(self.native.events.count(("source",)), 1)
        self.assertEqual([row["name"] for row in report["batch"]["outcomes"]], ["B", "A"])
        self.assertTrue(all(scene.published for scene in self.native.instances))

    def test_write_all_supports_packages_on_both_entry_paths(self):
        self.project(second=True)
        for legacy in (False, True):
            with self.subTest(legacy=legacy):
                code, report, _ = self.invoke("--write_all", legacy=legacy,
                                              destination=f"out-{legacy}")
                self.assertEqual(code, 0, report)
                self.assertEqual([row["name"] for row in report["batch"]["outcomes"]], ["A", "B"])

    def test_imported_scene_is_not_a_write_all_candidate(self):
        self.write("lesson_project/helper.py", "from manimlib import Scene\nclass Foreign(Scene): pass\n")
        self.source.write_text("from .helper import Foreign\nclass Local(Foreign): pass\n")
        code, report, _ = self.invoke("-a")
        self.assertEqual(code, 0, report)
        self.assertEqual([row["name"] for row in report["batch"]["outcomes"]], ["Local"])

    def test_repeated_invocation_reads_same_timestamp_helper_edits(self):
        helper = self.root / "lesson_project/helper.py"
        timestamp = helper.stat().st_mtime_ns
        py_compile.compile(str(helper), doraise=True)
        self.source.write_text("from manimlib import Scene, events\nfrom .helper import VALUE\nclass A(Scene):\n    def construct(self): events.append(('value', VALUE))\n")
        self.assertEqual(self.invoke(destination="first.png")[0], 0)
        helper.write_text("VALUE = 29\n")
        os.utime(helper, ns=(timestamp, timestamp))
        self.assertEqual(self.invoke(destination="second.png")[0], 0)
        self.assertEqual([event for event in self.native.events if event[0] == "value"],
                         [("value", 17), ("value", 29)])

    def test_failed_import_is_cleaned_up_before_retry(self):
        self.write("lesson_project/helper.py", "raise ValueError('incomplete helper')\n")
        self.project()
        code, report, _ = self.invoke()
        self.assertEqual(code, 5)
        self.assertIn("incomplete helper", report["message"])
        self.assertEqual(self.native.instances, [])
        self.write("lesson_project/helper.py", "VALUE = 17\n")
        self.assertEqual(self.invoke()[0], 0)

    def test_interrupt_unloads_project_and_cancels_output(self):
        self.source.write_text("from manimlib import Scene\nclass A(Scene):\n    def construct(self):\n        from . import helper\n        raise KeyboardInterrupt()\n")
        code, report, _ = self.invoke()
        self.assertEqual(code, 130, report)
        self.assertEqual(self.native.instances[0].events[-1], "abort")
        self.assertFalse((self.root / "out").exists())

    def test_package_initializer_imports_source_only_once(self):
        self.project()
        self.write("lesson_project/__init__.py", "from . import scenes\n")
        code, report, _ = self.invoke()
        self.assertEqual(code, 0, report)
        self.assertEqual(self.native.events.count(("source",)), 1)

    def test_unknown_name_refuses_before_constructor(self):
        self.project()
        code, report, _ = self.invoke("Missing")
        self.assertEqual(code, 5)
        self.assertIn("Missing", report["message"])
        self.assertEqual(self.native.instances, [])

    def test_reproducible_remains_fail_closed_before_source_execution(self):
        self.source.write_text("raise AssertionError('must not execute')\n")
        code, report, _ = self.invoke("--reproducible")
        self.assertEqual(code, 4)
        self.assertEqual(report["kind"], "render-capability-unavailable")
        self.assertEqual(self.native.events, [])


if __name__ == "__main__":
    unittest.main()
