"""Real Python imports; no renderer or fake native-storage assertions."""
from __future__ import annotations

import hashlib
import importlib
import os
import pickle
import py_compile
import sys
import tempfile
import threading
import unittest
from pathlib import Path
from types import ModuleType
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))
from fmn_python.scene_loading import SceneSource


class Scene:
    pass


class SceneLoading(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.native = ModuleType("scene_test_base")
        self.native.Scene = Scene
        modules = patch.dict(sys.modules, {"scene_test_base": self.native})
        modules.start()
        self.addCleanup(modules.stop)
        self.initial_path = list(sys.path)
        self.initial_finders = list(sys.meta_path)

    def write(self, path, text):
        path = self.root / path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def source(self, text="class Demo(Scene): pass\n", name="explanation.py"):
        return self.write(name, "from scene_test_base import Scene\n" + text)

    def assert_restored(self):
        self.assertEqual(sys.path, self.initial_path)
        self.assertEqual(sys.meta_path, self.initial_finders)

    def test_module_lives_for_entire_scene_execution(self):
        path = self.source("class Demo(Scene):\n    def identity(self):\n        import sys\n        return sys.modules[__name__].Demo is type(self)\n")
        with SceneSource(path, Scene) as loaded:
            self.assertTrue(loaded.scenes["Demo"]().identity())
            self.assertIs(sys.modules[loaded.name], loaded.module)
            self.assertIs(pickle.loads(pickle.dumps(loaded.scenes["Demo"])), loaded.scenes["Demo"])
        self.assertNotIn(loaded.name, sys.modules)
        self.assert_restored()

    def test_package_relative_and_lazy_nested_imports(self):
        self.write("lesson/__init__.py", "VALUE = 3\n")
        self.write("lesson/chapter/__init__.py", "")
        helper = self.write("lesson/chapter/helper.py", "VALUE = 7\n")
        path = self.source("from .. import VALUE\nclass Demo(Scene):\n    def run(self):\n        from .helper import VALUE as n\n        return n + VALUE\n", "lesson/chapter/scenes.py")
        with SceneSource(path, Scene) as loaded:
            self.assertEqual(loaded.name, "lesson.chapter.scenes")
            self.assertEqual(loaded.scenes["Demo"]().run(), 10)
            self.assertEqual(loaded.source_digests[helper], hashlib.sha256(helper.read_bytes()).hexdigest())
            self.assertIs(importlib.import_module("lesson.chapter").scenes, loaded.module)
        self.assertNotIn("lesson", sys.modules)
        self.assertNotIn("lesson.chapter.helper", sys.modules)
        self.assert_restored()

    def test_sibling_and_self_import_share_one_module(self):
        self.write("scene_helper.py", "VALUE = 42\n")
        path = self.source("class Demo(Scene):\n    def run(self):\n        import explanation, scene_helper\n        return explanation.Demo is type(self), scene_helper.VALUE\n")
        with SceneSource(path, Scene) as loaded:
            self.assertEqual(loaded.scenes["Demo"]().run(), (True, 42))
        self.assertNotIn("explanation", sys.modules)
        self.assertNotIn("scene_helper", sys.modules)

    def test_imported_scene_classes_are_not_selected(self):
        self.write("scene_helper.py", "from scene_test_base import Scene\nclass Imported(Scene): pass\n")
        path = self.source("from scene_helper import Imported\nclass Demo(Imported): pass\n")
        with SceneSource(path, Scene) as loaded:
            self.assertEqual(list(loaded.scenes), ["Demo"])

    def test_dataclass_postponed_annotations(self):
        path = self.write("explanation.py", "from __future__ import annotations\nfrom dataclasses import dataclass\nfrom scene_test_base import Scene\n@dataclass\nclass Model:\n    value: int = 9\nclass Demo(Scene):\n    model = Model()\n")
        with SceneSource(path, Scene) as loaded:
            self.assertEqual(loaded.scenes["Demo"].model.value, 9)

    def test_equal_size_equal_timestamp_helper_edits_ignore_stale_pyc(self):
        helper = self.write("scene_helper.py", "VALUE = 11\n")
        timestamp = helper.stat().st_mtime_ns
        py_compile.compile(str(helper), doraise=True)
        path = self.source("from scene_helper import VALUE\nclass Demo(Scene):\n    value = VALUE\n")
        with SceneSource(path, Scene) as first:
            self.assertEqual(first.scenes["Demo"].value, 11)
        helper.write_text("VALUE = 22\n")
        os.utime(helper, ns=(timestamp, timestamp))
        with SceneSource(path, Scene) as second:
            self.assertEqual(second.scenes["Demo"].value, 22)
            self.assertNotEqual(first.source_digests[helper], second.source_digests[helper])

    def test_preexisting_local_module_is_restored_not_mutated(self):
        helper = self.write("scene_helper.py", "VALUE = 22\n")
        old = ModuleType("scene_helper")
        old.__file__, old.VALUE = str(helper), 11
        sys.modules["scene_helper"] = old
        path = self.source("from scene_helper import VALUE\nclass Demo(Scene):\n    value = VALUE\n")
        with SceneSource(path, Scene) as loaded:
            self.assertEqual(loaded.scenes["Demo"].value, 22)
            self.assertIsNot(sys.modules["scene_helper"], old)
        self.assertIs(sys.modules["scene_helper"], old)
        self.assertEqual(old.VALUE, 11)

    def test_foreign_helper_and_package_refuse_without_mutation(self):
        for name, file in (("scene_helper", "scene_helper.py"), ("lesson", "lesson/__init__.py")):
            with self.subTest(name=name):
                self.write(file, "")
                old = ModuleType(name)
                old.__file__ = "/foreign/" + file
                sys.modules[name] = old
                path = self.source("raise AssertionError('must not execute')\n")
                with self.assertRaisesRegex(ImportError, "another location"):
                    with SceneSource(path, Scene):
                        self.fail("foreign module admitted")
                self.assertIs(sys.modules[name], old)
                sys.modules.pop(name)
                self.assert_restored()

    def test_package_initializer_can_import_selected_source_once(self):
        self.write("lesson/__init__.py", "from . import scenes\n")
        path = self.source("import scene_test_base\nscene_test_base.executions = getattr(scene_test_base, 'executions', 0) + 1\nclass Demo(Scene): pass\n", "lesson/scenes.py")
        with SceneSource(path, Scene) as loaded:
            self.assertEqual(self.native.executions, 1)
            self.assertIs(sys.modules["lesson"].scenes, loaded.module)

    def test_package_initializer_itself_is_a_scene_source(self):
        self.write("lesson/helper.py", "VALUE = 8\n")
        path = self.source("from .helper import VALUE\nclass Demo(Scene):\n    value = VALUE\n", "lesson/__init__.py")
        with SceneSource(path, Scene) as loaded:
            self.assertEqual(loaded.name, "lesson")
            self.assertEqual(loaded.scenes["Demo"].value, 8)

    def test_pyw_and_nonidentifier_filenames(self):
        for name in ("scene-file.py", "scene.pyw"):
            with self.subTest(name=name):
                with SceneSource(self.source(name=name), Scene) as loaded:
                    self.assertEqual(list(loaded.scenes), ["Demo"])

    def test_source_encoding_cookie_is_respected(self):
        path = self.root / "encoded.py"
        path.write_bytes(b"# coding: latin-1\nfrom scene_test_base import Scene\nclass Demo(Scene):\n    label = 'caf\xe9'\n")
        with SceneSource(path, Scene) as loaded:
            self.assertEqual(loaded.scenes["Demo"].label, "caf\u00e9")

    def test_failures_and_interrupts_restore_imports_and_release_owner(self):
        self.write("scene_helper.py", "VALUE = 22\n")
        for exception in ("ValueError('broken')", "SystemExit(0)", "KeyboardInterrupt()"):
            path = self.source("import scene_helper\nraise " + exception + "\n")
            with self.assertRaises(BaseException):
                with SceneSource(path, Scene):
                    self.fail("failed source entered")
            self.assertNotIn("scene_helper", sys.modules)
            self.assert_restored()
        with SceneSource(self.source(), Scene) as loaded:
            self.assertIn("Demo", loaded.scenes)

    def test_context_body_failure_also_releases_imports(self):
        with self.assertRaisesRegex(RuntimeError, "body"):
            with SceneSource(self.source(), Scene):
                raise RuntimeError("body")
        self.assert_restored()

    def test_concurrent_owner_is_rejected_without_deadlock(self):
        path, errors = self.source(), []
        def load():
            try:
                with SceneSource(path, Scene):
                    pass
            except RuntimeError as error:
                errors.append(str(error))
        with SceneSource(path, Scene):
            worker = threading.Thread(target=load)
            worker.start()
            worker.join(5)
            self.assertFalse(worker.is_alive())
        self.assertEqual(len(errors), 1)
        self.assertIn("another scene source", errors[0])

    def test_authored_path_additions_are_not_discarded(self):
        marker = str(self.root / "authored")
        with SceneSource(self.source(), Scene):
            sys.path.append(marker)
        self.assertEqual(sys.path, self.initial_path + [marker])
        sys.path.pop()

    def test_a_source_context_cannot_be_reused(self):
        loaded = SceneSource(self.source(), Scene)
        with loaded:
            pass
        with self.assertRaisesRegex(RuntimeError, "only once"):
            with loaded:
                pass


if __name__ == "__main__":
    unittest.main()
