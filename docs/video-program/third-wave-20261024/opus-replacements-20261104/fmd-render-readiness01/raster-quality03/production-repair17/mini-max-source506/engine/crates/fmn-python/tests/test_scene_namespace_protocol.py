"""Real CPython import regressions; these do not require or emulate rendering."""
from __future__ import annotations

import hashlib
import importlib
import importlib.resources
import importlib.util
import os
from pathlib import Path
import sys
import tempfile
from types import ModuleType
import unittest


_SOURCE = Path(__file__).resolve().parents[1] / "python" / "fmn_python" / "scene_loading.py"
# scene_loading imports its sibling `.runtime_paths`. Load it inside a synthetic
# package rooted at the same source directory, so the relative import resolves to
# the repository's own module without importing fmn_python's native extension.
_PACKAGE = "_fmn_namespace_loader_tests"
_PACKAGE_MODULE = ModuleType(_PACKAGE)
_PACKAGE_MODULE.__path__ = [str(_SOURCE.parent)]
sys.modules[_PACKAGE] = _PACKAGE_MODULE
_SPEC = importlib.util.spec_from_file_location(f"{_PACKAGE}.scene_loading", _SOURCE)
_LOADING = importlib.util.module_from_spec(_SPEC)
sys.modules[_SPEC.name] = _LOADING
_SPEC.loader.exec_module(_LOADING)
SceneSource = _LOADING.SceneSource


class Scene:
    pass


class NamespaceSceneTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.saved_path = list(sys.path)
        self.saved_meta = list(sys.meta_path)
        self.saved_modules = dict(sys.modules)
        self.addCleanup(self.restore_imports)
        self.source = self.write("scene.py", "from fmn_test_ns import helper\nvalue = helper.VALUE\n")
        self.helper = self.write("fmn_test_ns/helper.py", "VALUE = 11\n")

    def restore_imports(self):
        for name in tuple(sys.modules):
            if (name.startswith(("fmn_test_", "__fmn_scene_")) or name == "scene") and name not in self.saved_modules:
                sys.modules.pop(name, None)
        for name, module in self.saved_modules.items():
            if name.startswith("fmn_test_") or name == "scene":
                sys.modules[name] = module
        sys.path[:] = self.saved_path
        sys.meta_path[:] = self.saved_meta
        importlib.invalidate_caches()

    def write(self, relative, text):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return path

    def rewrite_helper(self, value=22):
        before = self.helper.stat()
        self.helper.write_text(f"VALUE = {value}\n", encoding="utf-8")
        os.utime(self.helper, ns=(before.st_atime_ns, before.st_mtime_ns))

    def test_namespace_and_child_are_scoped(self):
        with SceneSource(self.source, Scene) as loaded:
            self.assertEqual(loaded.module.value, 11)
            self.assertIn("fmn_test_ns", sys.modules)
            self.assertIn("fmn_test_ns.helper", sys.modules)
        self.assertNotIn("fmn_test_ns", sys.modules)
        self.assertNotIn("fmn_test_ns.helper", sys.modules)
        self.assertEqual(sys.path, self.saved_path)
        self.assertEqual(sys.meta_path, self.saved_meta)

    def test_equal_size_same_timestamp_edit_is_seen_and_hashed(self):
        with SceneSource(self.source, Scene) as first:
            self.assertEqual(first.module.value, 11)
            first_digest = first.source_digests[self.helper]
        self.rewrite_helper()
        with SceneSource(self.source, Scene) as second:
            self.assertEqual(second.module.value, 22)
            self.assertNotEqual(first_digest, second.source_digests[self.helper])
            self.assertEqual(second.source_digests[self.helper], hashlib.sha256(self.helper.read_bytes()).hexdigest())

    def test_existing_namespace_and_child_are_refreshed_then_restored(self):
        sys.path.insert(0, str(self.root))
        original = importlib.import_module("fmn_test_ns.helper")
        package = sys.modules["fmn_test_ns"]
        self.rewrite_helper()
        with SceneSource(self.source, Scene) as loaded:
            self.assertEqual(loaded.module.value, 22)
            self.assertIsNot(sys.modules["fmn_test_ns"], package)
            self.assertIsNot(sys.modules["fmn_test_ns.helper"], original)
        self.assertIs(sys.modules["fmn_test_ns"], package)
        self.assertIs(sys.modules["fmn_test_ns.helper"], original)
        self.assertIs(package.helper, original)
        self.assertEqual(package.helper.VALUE, 11)

    def test_nested_namespaces_and_lazy_imports_are_fresh(self):
        self.helper = self.write("fmn_test_ns/nested/helper.py", "VALUE = 11\n")
        self.source.write_text("def execute():\n    from fmn_test_ns.nested import helper\n    return helper.VALUE\n")
        for expected in (11, 22):
            with SceneSource(self.source, Scene) as loaded:
                self.assertNotIn(self.helper, loaded.source_digests)
                self.assertEqual(loaded.module.execute(), expected)
                self.assertIn(self.helper, loaded.source_digests)
            self.assertNotIn("fmn_test_ns.nested", sys.modules)
            self.assertNotIn("fmn_test_ns", sys.modules)
            self.rewrite_helper()

    def test_namespace_resources_keep_standard_loader_behavior(self):
        self.write("fmn_test_ns/assets/values.txt", "native resource readers remain available\n")
        with SceneSource(self.source, Scene):
            package = importlib.import_module("fmn_test_ns")
            self.assertIsNone(package.__file__)
            self.assertIsNone(package.__spec__.origin)
            self.assertEqual(
                importlib.resources.files(package).joinpath("assets", "values.txt").read_text(),
                "native resource readers remain available\n",
            )

    def test_failure_cleans_namespace_and_releases_owner(self):
        self.source.write_text("from fmn_test_ns import helper\nraise RuntimeError('authored failure')\n")
        with self.assertRaisesRegex(RuntimeError, "authored failure"):
            with SceneSource(self.source, Scene):
                pass
        self.assertNotIn("fmn_test_ns", sys.modules)
        self.assertNotIn("fmn_test_ns.helper", sys.modules)
        self.source.write_text("from fmn_test_ns import helper\nvalue = helper.VALUE\n")
        self.rewrite_helper()
        with SceneSource(self.source, Scene) as loaded:
            self.assertEqual(loaded.module.value, 22)

    def test_split_namespace_refuses_without_loading_local_child(self):
        with tempfile.TemporaryDirectory() as other:
            (Path(other) / "fmn_test_ns").mkdir()
            sys.path.insert(0, other)
            with self.assertRaisesRegex(ImportError, "namespace.*outside the scene project"):
                with SceneSource(self.source, Scene):
                    pass
            self.assertNotIn("fmn_test_ns.helper", sys.modules)
            self.assertNotIn("fmn_test_ns", sys.modules)

    def test_cached_foreign_namespace_cannot_redirect_scene_imports(self):
        with tempfile.TemporaryDirectory() as other:
            (Path(other) / "fmn_test_ns").mkdir()
            sys.path.insert(0, other)
            package = importlib.import_module("fmn_test_ns")
            with self.assertRaisesRegex(ImportError, "namespace.*outside the scene project"):
                with SceneSource(self.source, Scene):
                    pass
            self.assertIs(sys.modules["fmn_test_ns"], package)
            self.assertNotIn("fmn_test_ns.helper", sys.modules)

    def test_authored_namespace_replacement_is_preserved(self):
        replacement = ModuleType("fmn_test_ns")
        with SceneSource(self.source, Scene):
            sys.modules["fmn_test_ns"] = replacement
        self.assertIs(sys.modules["fmn_test_ns"], replacement)
        self.assertNotIn("fmn_test_ns.helper", sys.modules)

    def test_unrelated_namespace_and_stdlib_are_not_reloaded(self):
        with tempfile.TemporaryDirectory() as other:
            (Path(other) / "fmn_test_unrelated").mkdir()
            sys.path.insert(0, other)
            foreign = importlib.import_module("fmn_test_unrelated")
            with SceneSource(self.source, Scene):
                self.assertIs(sys.modules["fmn_test_unrelated"], foreign)
                self.assertIs(sys.modules["pathlib"], self.saved_modules["pathlib"])
            self.assertIs(sys.modules["fmn_test_unrelated"], foreign)


if __name__ == "__main__":
    unittest.main()
