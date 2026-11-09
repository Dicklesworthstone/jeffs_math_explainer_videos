"""Executed-source provenance through the real scoped Python module loader.

No native renderer is replaced here: these tests exercise the Python import
boundary independently of the optional extension.
"""
from __future__ import annotations

import hashlib
import importlib
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python import scene_loading
from fmn_python.scene_loading import SceneSource


class SourceCaptureTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def write(self, name, data):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    def test_unexecuted_source_is_not_claimed(self):
        path = self.write("scene.py", b"VALUE = 1\n")
        source = SceneSource(path, object)
        self.assertEqual(source.sources, {})
        with source:
            self.assertEqual(source.sources, {"scene.py": b"VALUE = 1\n"})

    def test_self_modification_records_executed_bytes(self):
        data = (b"from pathlib import Path\nVALUE = 1\n"
                b"Path(__file__).write_bytes(b'VALUE = 2\\n')\n")
        path = self.write("scene.py", data)
        with SceneSource(path, object) as source:
            self.assertEqual(source.module.VALUE, 1)
            self.assertEqual(path.read_bytes(), b"VALUE = 2\n")
            self.assertEqual(source.sources, {"scene.py": data})
            self.assertEqual(source.source_digests[path], hashlib.sha256(data).hexdigest())
        self.assertEqual(source.sources, {"scene.py": data})

    def test_lazy_import_survives_edit_and_deletion(self):
        data = b"def construct():\n    import capture_late_helper\n    return capture_late_helper.VALUE\n"
        path = self.write("scene.py", data)
        helper = self.write("capture_late_helper.py", b"VALUE = 42\n")
        with SceneSource(path, object) as source:
            self.assertNotIn("capture_late_helper.py", source.sources)
            self.assertEqual(source.module.construct(), 42)
            helper.write_bytes(b"VALUE = 99\n")
            path.unlink()
            self.assertEqual(source.sources, {
                "scene.py": data, "capture_late_helper.py": b"VALUE = 42\n",
            })
        helper.unlink()
        self.assertEqual(source.sources["capture_late_helper.py"], b"VALUE = 42\n")

    def test_snapshot_is_detached_and_preserves_source_encoding(self):
        data = b"# coding: latin-1\nVALUE = 'caf\xe9'\n"
        path = self.write("scene.py", data)
        with SceneSource(path, object) as source:
            self.assertEqual(source.module.VALUE, "caf\u00e9")
            snapshot = source.sources
            snapshot["scene.py"] = b"wrong"
            self.assertEqual(source.sources, {"scene.py": data})

    def test_same_basename_declared_helpers_are_not_overwritten(self):
        data = b"import capture_shared_a.helper, capture_shared_b.helper\n"
        path = self.write("project/helper.py", data)
        a = self.write("shared/capture_shared_a/helper.py", b"VALUE = 1\n")
        b = self.write("shared/capture_shared_b/helper.py", b"VALUE = 2\n")
        declared = {p: hashlib.sha256(p.read_bytes()).hexdigest() for p in (path, a, b)}
        sys.path.insert(0, str(self.root / "shared"))
        self.addCleanup(sys.path.remove, str(self.root / "shared"))
        with SceneSource(path, object, source_inputs=declared) as source:
            self.assertEqual(source.sources, {
                "project/helper.py": data,
                "shared/capture_shared_a/helper.py": b"VALUE = 1\n",
                "shared/capture_shared_b/helper.py": b"VALUE = 2\n",
            })
            self.assertTrue(all(not Path(name).is_absolute() and ".." not in Path(name).parts
                                for name in source.sources))

    def test_changed_reload_runs_normally_but_cannot_claim_one_source_version(self):
        path = self.write("scene.py", b"import capture_reload_helper\n")
        helper = self.write("capture_reload_helper.py", b"VALUE = 1\n")
        with SceneSource(path, object) as source:
            module = source.module.capture_reload_helper
            helper.write_bytes(b"VALUE = 2\n")
            importlib.reload(module)
            self.assertEqual(module.VALUE, 2)
            with self.assertRaisesRegex(RuntimeError, "source changed during execution"):
                _ = source.sources

    def test_unchanged_reload_keeps_one_capture(self):
        path = self.write("scene.py", b"import capture_reload_helper\n")
        self.write("capture_reload_helper.py", b"VALUE = 1\n")
        with SceneSource(path, object) as source:
            before = source.sources
            importlib.reload(source.module.capture_reload_helper)
            self.assertEqual(source.sources, before)
            self.assertEqual(len(source.source_digests), 2)

    def test_declared_mismatch_still_precedes_execution(self):
        path = self.write("scene.py", b"raise AssertionError('must not execute')\n")
        with self.assertRaisesRegex(RuntimeError, "changed project source"):
            with SceneSource(path, object, source_inputs={path: "0" * 64}):
                self.fail("unreachable")
        # A failed import must release its interpreter ownership as before.
        path.write_bytes(b"VALUE = 3\n")
        with SceneSource(path, object) as source:
            self.assertEqual(source.module.VALUE, 3)

    def test_capture_byte_budget_preserves_ordinary_execution(self):
        data = b"VALUE = 7\n"
        path = self.write("scene.py", data)
        with patch.object(scene_loading, "_MAX_CAPTURED_SOURCE_BYTES", len(data)):
            with SceneSource(path, object) as source:
                self.assertEqual(source.sources, {"scene.py": data})
        with patch.object(scene_loading, "_MAX_CAPTURED_SOURCE_BYTES", len(data) - 1):
            with SceneSource(path, object) as source:
                self.assertEqual(source.module.VALUE, 7)
                self.assertEqual(source._source_bytes, {})
                with self.assertRaisesRegex(RuntimeError, "capture exceeds"):
                    _ = source.sources

    def test_capture_count_budget_never_returns_partial_success(self):
        path = self.write("scene.py", b"import capture_budget_helper\nVALUE = 5\n")
        self.write("capture_budget_helper.py", b"VALUE = 1\n")
        with patch.object(scene_loading, "_MAX_CAPTURED_SOURCES", 1):
            with SceneSource(path, object) as source:
                self.assertEqual(source.module.VALUE, 5)
                self.assertEqual(len(source._source_bytes), 1)
                with self.assertRaisesRegex(RuntimeError, "capture exceeds"):
                    _ = source.sources

    def test_unchanged_reload_does_not_spend_capture_budget_twice(self):
        data = b"import capture_reload_helper\n"
        helper_data = b"VALUE = 8\n"
        path = self.write("scene.py", data)
        self.write("capture_reload_helper.py", helper_data)
        limit = len(data) + len(helper_data)
        with patch.object(scene_loading, "_MAX_CAPTURED_SOURCE_BYTES", limit):
            with SceneSource(path, object) as source:
                importlib.reload(source.module.capture_reload_helper)
                self.assertEqual(source.sources, {"scene.py": data, "capture_reload_helper.py": helper_data})
                self.assertEqual(source._source_capture_bytes, limit)

    def test_snapshot_does_not_touch_filesystem(self):
        data = b"VALUE = 8\n"
        path = self.write("scene.py", data)
        with SceneSource(path, object) as source:
            with patch.object(Path, "read_bytes", side_effect=AssertionError("unexpected reread")):
                self.assertEqual(source.sources, {"scene.py": data})


if __name__ == "__main__":
    unittest.main()
