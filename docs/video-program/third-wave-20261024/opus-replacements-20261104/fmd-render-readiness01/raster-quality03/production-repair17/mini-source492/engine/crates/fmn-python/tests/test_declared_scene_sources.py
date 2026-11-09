"""Actual CPython import/bytecode behavior under a declared Studio source map."""
import hashlib
import importlib
import os
from pathlib import Path
import py_compile
import sys
import tempfile
from types import ModuleType
import unittest
from unittest.mock import patch

from fmn_python.scene_loading import SceneSource


class Scene:
    pass


class DeclaredSourceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="fmn-declared-source-")
        self.root = Path(self.temp.name)
        self.project = self.root / "project"
        self.shared = self.root / "shared"
        self.project.mkdir()
        self.shared.mkdir()
        self.source = self.project / "lesson.py"
        self.source.write_text("VALUE = 0\n")
        self.helper = self.shared / "fmn_shared_helper.py"
        self.helper.write_text("VALUE = 1\n")
        self.path = patch.object(sys, "path", [str(self.shared), *sys.path])
        self.path.start()
        self.saved = dict(sys.modules)

    def tearDown(self):
        self.path.stop()
        for name in list(sys.modules):
            if name.startswith(("fmn_shared_", "fmn_test_namespace", "fmn_test_package")):
                if name in self.saved:
                    sys.modules[name] = self.saved[name]
                else:
                    sys.modules.pop(name, None)
        self.temp.cleanup()

    def inputs(self, *extra):
        return {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
                for path in (self.source, self.helper, *extra)}

    def load(self, inputs):
        return SceneSource(self.source, Scene, source_inputs=inputs)

    def test_equal_length_same_timestamp_shared_helper_ignores_stale_pyc(self):
        prior = importlib.import_module("fmn_shared_helper")
        py_compile.compile(str(self.helper), doraise=True)
        stamp = self.helper.stat().st_mtime_ns
        self.helper.write_text("VALUE = 2\n")
        os.utime(self.helper, ns=(stamp, stamp))
        self.source.write_text("import fmn_shared_helper\nVALUE = fmn_shared_helper.VALUE\n")
        expected = self.inputs()
        with self.load(expected) as loaded:
            self.assertEqual(loaded.module.VALUE, 2)
            self.assertIsNot(sys.modules["fmn_shared_helper"], prior)
            self.assertEqual(loaded.source_digests[self.helper], expected[str(self.helper)])
        self.assertIs(sys.modules["fmn_shared_helper"], prior)
        self.assertEqual(prior.VALUE, 1)

    def test_changed_helper_is_refused_before_its_top_level_effects(self):
        self.source.write_text("import fmn_shared_helper\n")
        expected = self.inputs()
        marker = self.root / "executed"
        self.helper.write_text(f"from pathlib import Path\nPath({str(marker)!r}).write_text('bad')\n")
        with self.assertRaisesRegex(RuntimeError, "executed changed project source"):
            with self.load(expected):
                pass
        self.assertFalse(marker.exists())
        self.assertNotIn("fmn_shared_helper", sys.modules)
        # An exception must release the global import owner.
        with self.load(self.inputs()):
            pass
        self.assertTrue(marker.exists())

    def test_changed_entry_does_not_execute_before_hash_refusal(self):
        expected = self.inputs()
        marker = self.root / "entry-executed"
        self.source.write_text(f"from pathlib import Path\nPath({str(marker)!r}).write_text('bad')\n")
        with self.assertRaisesRegex(RuntimeError, "executed changed"):
            with self.load(expected):
                pass
        self.assertFalse(marker.exists())

    def test_lazy_shared_import_is_fresh_and_observed(self):
        self.source.write_text("def later():\n import fmn_shared_helper\n return fmn_shared_helper.VALUE\n")
        with self.load(self.inputs()) as loaded:
            self.assertNotIn(self.helper, loaded.source_digests)
            self.assertEqual(loaded.module.later(), 1)
            self.assertIn(self.helper, loaded.source_digests)
        self.assertNotIn("fmn_shared_helper", sys.modules)

    def test_undeclared_project_source_refuses_before_execution(self):
        local = self.project / "new_helper.py"
        local.write_text("raise AssertionError('must not execute')\n")
        self.source.write_text("import new_helper\n")
        with self.assertRaisesRegex(RuntimeError, "undeclared project source"):
            with self.load(self.inputs()):
                pass
        self.assertNotIn("new_helper", sys.modules)

    def test_external_package_initializer_and_helper_are_refreshed_together(self):
        package = self.shared / "fmn_test_package"
        package.mkdir()
        init = package / "__init__.py"
        init.write_text("from . import helper\n")
        helper = package / "helper.py"
        helper.write_text("VALUE = 1\n")
        prior = importlib.import_module("fmn_test_package")
        helper.write_text("VALUE = 2\n")
        self.source.write_text("from fmn_test_package import helper\nVALUE = helper.VALUE\n")
        with self.load(self.inputs(init, helper)) as loaded:
            self.assertEqual(loaded.module.VALUE, 2)
            self.assertIsNot(sys.modules["fmn_test_package"], prior)
            self.assertIn(init, loaded.source_digests)
        self.assertIs(sys.modules["fmn_test_package"], prior)
        self.assertEqual(prior.helper.VALUE, 1)

    def test_partial_regular_package_declaration_refuses_cold_and_cached_parents(self):
        package = self.shared / "fmn_test_package"
        package.mkdir()
        init = package / "__init__.py"
        init.write_text("from . import helper\n")
        helper = package / "helper.py"
        helper.write_text("VALUE = 1\n")
        self.source.write_text("from fmn_test_package import helper\n")
        for cached in (False, True):
            if cached:
                importlib.import_module("fmn_test_package")
            with self.subTest(cached=cached), self.assertRaisesRegex(ImportError, "undeclared initializer"):
                with self.load(self.inputs(helper)):
                    pass

    def test_external_namespace_container_does_not_reuse_old_child_attributes(self):
        namespace = self.shared / "fmn_test_namespace"
        namespace.mkdir()
        helper = namespace / "helper.py"
        helper.write_text("VALUE = 1\n")
        prior = importlib.import_module("fmn_test_namespace.helper")
        parent = sys.modules["fmn_test_namespace"]
        helper.write_text("VALUE = 2\n")
        self.source.write_text("from fmn_test_namespace import helper\nVALUE = helper.VALUE\n")
        with self.load(self.inputs(helper)) as loaded:
            self.assertEqual(loaded.module.VALUE, 2)
            self.assertIsNot(sys.modules["fmn_test_namespace"], parent)
        self.assertIs(sys.modules["fmn_test_namespace"], parent)
        self.assertIs(parent.helper, prior)

    def test_mixed_undeclared_namespace_locations_are_still_refused(self):
        (self.shared / "fmn_test_namespace").mkdir()
        helper = self.shared / "fmn_test_namespace" / "helper.py"
        helper.write_text("VALUE = 1\n")
        foreign = self.root / "foreign"
        (foreign / "fmn_test_namespace").mkdir(parents=True)
        self.source.write_text("from fmn_test_namespace import helper\n")
        with patch.object(sys, "path", [str(foreign), *sys.path]):
            with self.assertRaisesRegex(ImportError, "outside the scene project"):
                with self.load(self.inputs(helper)):
                    pass

    def test_expectations_are_copied_and_do_not_modify_import_search_paths(self):
        expected = self.inputs()
        loader = self.load(expected)
        expected[str(self.source)] = "0" * 64
        with loader as loaded:
            self.assertEqual(loaded.module.VALUE, 0)
            self.assertEqual(sys.path.count(str(self.shared)), 1)

    def test_without_declared_inputs_foreign_modules_keep_existing_behavior(self):
        self.source.write_text("import fmn_shared_helper\nVALUE = fmn_shared_helper.VALUE\n")
        prior = importlib.import_module("fmn_shared_helper")
        self.helper.write_text("VALUE = 2\n")
        with SceneSource(self.source, Scene) as loaded:
            self.assertEqual(loaded.module.VALUE, 1)
            self.assertNotIn(self.helper, loaded.source_digests)
            self.assertIs(sys.modules["fmn_shared_helper"], prior)

    def test_protected_portal_modules_are_never_reloaded(self):
        engine = ModuleType("manimlib")
        engine.__file__ = str(self.helper)
        self.source.write_text("import manimlib\n")
        with patch.dict(sys.modules, {"manimlib": engine}):
            with self.load(self.inputs()) as loaded:
                self.assertIs(loaded.module.manimlib, engine)
            self.assertIs(sys.modules["manimlib"], engine)

    def test_invalid_input_maps_refuse_before_acquiring_import_authority(self):
        for values in ({}, [], {"relative": "0"*64}, {str(self.source): "X"*64},
                       {str(self.root/".."/"escape.py"): "0"*64},
                       {str(self.root/str(i)): "0"*64 for i in range(4097)}):
            with self.subTest(values=str(values)[:80]), self.assertRaises(ValueError):
                self.load(values)
        with self.load(self.inputs()):
            pass


if __name__ == "__main__":
    unittest.main()
