"""Actual Python import/reload transactions. No import or storage doubles.

The small Scene base marks discovery only; no native geometry or rendering is
claimed by these tests. Files are original temporary Python projects.
"""
from __future__ import annotations

import builtins
import hashlib
import importlib
import os
from pathlib import Path
import sys
import tempfile
import threading
from types import ModuleType
import unittest
from unittest.mock import patch

from fmn_python.scene_loading import SceneSource
from fmn_python import source_reload


class SourceReloadTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="fmn-reload-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.events = []
        support = ModuleType("fmn_reload_test_support")
        support.Scene = type("Scene", (), {})
        support.events = self.events
        self.Scene = support.Scene
        modules = patch.dict(sys.modules, {support.__name__: support})
        modules.start()
        self.addCleanup(modules.stop)

    def write(self, name, source):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(source.encode() if isinstance(source, str) else source)
        return path

    def test_primary_reload_updates_discovery_without_replacing_old_instances(self):
        path = self.write("scene.py", "from fmn_reload_test_support import Scene\nclass Old(Scene):\n    VALUE=1\n")
        with SceneSource(path, self.Scene) as source:
            previous = source.module
            instance = source.scenes["Old"]()
            path.write_text("from fmn_reload_test_support import Scene\nclass New(Scene):\n    VALUE=2\n")
            module = source.reload()
            self.assertIsNot(module, previous)
            self.assertEqual(list(source.scenes), ["New"])
            self.assertEqual(source.scenes["New"]().VALUE, 2)
            self.assertEqual(instance.VALUE, 1)
            self.assertIs(sys.modules[source.name], module)
            self.assertIs(sys.modules["scene"], module)
        self.assertNotIn(source.name, sys.modules)
        self.assertNotIn("scene", sys.modules)

    def test_unchanged_conditional_reload_does_not_execute_again(self):
        path = self.write("scene.py", "from fmn_reload_test_support import events\nevents.append(1)\n")
        with SceneSource(path, self.Scene) as source:
            old = source.module
            self.assertIs(source.reload(if_changed=True), old)
            self.assertEqual(self.events, [1])
            self.assertIsNot(source.reload(), old)
            self.assertEqual(self.events, [1, 1])
            self.assertEqual(source.sources["scene.py"], path.read_bytes())

    def test_equal_length_equal_timestamp_transitive_helper_is_fresh(self):
        helper = self.write("reload_helper.py", "VALUE = 10\n")
        path = self.write("scene.py", "from reload_helper import VALUE\n")
        with SceneSource(path, self.Scene) as source:
            timestamp = helper.stat().st_mtime_ns
            helper.write_text("VALUE = 20\n")
            os.utime(helper, ns=(timestamp, timestamp))
            self.assertEqual(source.reload(if_changed=True).VALUE, 20)
            self.assertEqual(sys.modules["reload_helper"].VALUE, 20)

    def test_package_relative_imports_and_dataclasses_keep_real_module_identity(self):
        self.write("reload_package/__init__.py", "from . import helper\n")
        helper = self.write("reload_package/helper.py", "VALUE=1\n")
        path = self.write("reload_package/scene.py", "from .helper import VALUE\nfrom dataclasses import dataclass\n@dataclass\nclass Model:\n    value:int=VALUE\n")
        with SceneSource(path, self.Scene) as source:
            helper.write_text("VALUE=3\n")
            module = source.reload()
            self.assertEqual(module.Model().value, 3)
            self.assertIs(sys.modules["reload_package"].scene, module)
            self.assertIs(sys.modules["reload_package"].helper, sys.modules["reload_package.helper"])

    def test_initializer_importing_primary_executes_primary_once(self):
        self.write("reload_package/__init__.py", "from . import scene\n")
        path = self.write("reload_package/scene.py", "from fmn_reload_test_support import events\nevents.append('scene')\n")
        with SceneSource(path, self.Scene) as source:
            source.reload()
            self.assertEqual(self.events, ["scene", "scene"])

    def test_namespace_containers_do_not_keep_stale_children(self):
        helper = self.write("reload_namespace/helper.py", "VALUE=1\n")
        path = self.write("scene.py", "from reload_namespace import helper\nVALUE=helper.VALUE\n")
        with SceneSource(path, self.Scene) as source:
            old_namespace = sys.modules["reload_namespace"]
            helper.write_text("VALUE=7\n")
            self.assertEqual(source.reload().VALUE, 7)
            self.assertIsNot(sys.modules["reload_namespace"], old_namespace)

    def test_lazy_helpers_are_included_in_next_reload(self):
        helper = self.write("reload_late.py", "VALUE=4\n")
        path = self.write("scene.py", "def value():\n    from reload_late import VALUE\n    return VALUE\n")
        with SceneSource(path, self.Scene) as source:
            self.assertEqual(source.module.value(), 4)
            helper.write_text("VALUE=9\n")
            self.assertEqual(source.reload(if_changed=True).value(), 9)

    def test_primary_syntax_failure_preserves_modules_and_prior_source_capture(self):
        path = self.write("scene.py", "VALUE=4\n")
        with SceneSource(path, self.Scene) as source:
            old, captured = source.module, source.sources
            path.write_text("def broken(\n")
            with self.assertRaises(SyntaxError):
                source.reload()
            self.assertIs(source.module, old)
            self.assertIs(sys.modules[source.name], old)
            self.assertEqual(source.sources, captured)

    def test_known_helper_syntax_is_checked_before_primary_side_effects(self):
        helper = self.write("reload_helper.py", "VALUE=1\n")
        path = self.write("scene.py", "from fmn_reload_test_support import events\nevents.append('execute')\nimport reload_helper\n")
        with SceneSource(path, self.Scene) as source:
            helper.write_text("if True\n")
            with self.assertRaises(SyntaxError):
                source.reload()
            self.assertEqual(self.events, ["execute"])

    def test_import_failure_rolls_back_entire_project_graph_and_can_retry(self):
        helper = self.write("reload_package/helper.py", "VALUE=1\n")
        self.write("reload_package/__init__.py", "from . import helper\n")
        data = "from .helper import VALUE\n"
        path = self.write("reload_package/scene.py", data)
        with SceneSource(path, self.Scene) as source:
            previous = {name: sys.modules[name] for name in source._loaded}
            helper.write_text("VALUE=9\n")
            path.write_text(data + "raise LookupError('authored failure')\n")
            with self.assertRaisesRegex(LookupError, "authored failure"):
                source.reload()
            self.assertTrue(all(sys.modules[name] is module for name, module in previous.items()))
            self.assertEqual(source.module.VALUE, 1)
            path.write_text(data)
            self.assertEqual(source.reload(if_changed=True).VALUE, 9)

    def test_new_helper_failure_removes_its_module_and_restores_foreign_parent_attribute(self):
        parent = ModuleType("reload_external")
        parent.__path__ = [str(self.root)]
        sys.modules[parent.__name__] = parent
        path = self.write("scene.py", "VALUE=1\n")
        self.write("new_helper.py", "VALUE=2\n")
        with SceneSource(path, self.Scene) as source:
            old = source.module
            path.write_text("import reload_external.new_helper\nraise RuntimeError('after import')\n")
            with self.assertRaisesRegex(RuntimeError, "after import"):
                source.reload()
            self.assertIs(source.module, old)
            self.assertNotIn("reload_external.new_helper", sys.modules)
            self.assertFalse(hasattr(parent, "new_helper"))

    def test_failed_reload_restores_existing_foreign_parent_child_binding(self):
        parent = ModuleType("reload_external")
        parent.__path__ = [str(self.root)]
        sys.modules[parent.__name__] = parent
        helper = self.write("helper.py", "VALUE=1\n")
        path = self.write("scene.py", "import reload_external.helper\n")
        with SceneSource(path, self.Scene) as source:
            old_child = parent.helper
            helper.write_text("VALUE=2\n")
            path.write_text("import reload_external.helper\nraise RuntimeError('after import')\n")
            with self.assertRaises(RuntimeError):
                source.reload()
            self.assertIs(parent.helper, old_child)
            self.assertIs(sys.modules["reload_external.helper"], old_child)

    def test_failed_reload_does_not_claim_to_undo_external_side_effects(self):
        path = self.write("scene.py", "VALUE=1\n")
        with SceneSource(path, self.Scene) as source:
            path.write_text("from fmn_reload_test_support import events\nevents.append('external effect')\nraise RuntimeError('fail')\n")
            with self.assertRaises(RuntimeError):
                source.reload()
            self.assertEqual(self.events, ["external effect"])
            with self.assertRaisesRegex(RuntimeError, "changed during execution"):
                _ = source.sources

    def test_preflight_bytes_are_exact_even_when_primary_edits_helper(self):
        helper = self.write("reload_helper.py", "VALUE=1\n")
        path = self.write("scene.py", "import reload_helper\nVALUE=reload_helper.VALUE\n")
        with SceneSource(path, self.Scene) as source:
            helper.write_text("VALUE=2\n")
            path.write_text(f"from pathlib import Path\nPath({str(helper)!r}).write_text('VALUE=3\\n')\nimport reload_helper\nVALUE=reload_helper.VALUE\n")
            self.assertEqual(source.reload().VALUE, 2)
            self.assertEqual(helper.read_text(), "VALUE=3\n")
            self.assertEqual(source.source_digests[helper], hashlib.sha256(b"VALUE=2\n").hexdigest())

    def test_removed_unused_helper_does_not_block_reload(self):
        helper = self.write("reload_helper.py", "VALUE=1\n")
        path = self.write("scene.py", "import reload_helper\n")
        with SceneSource(path, self.Scene) as source:
            helper.unlink()
            path.write_text("VALUE=2\n")
            module = source.reload()
            self.assertEqual(module.VALUE, 2)
            self.assertNotIn("reload_helper", sys.modules)
            self.assertIs(source.reload(if_changed=True), module)

    def test_removed_still_imported_helper_refuses_and_keeps_old_module(self):
        helper = self.write("reload_helper.py", "VALUE=1\n")
        path = self.write("scene.py", "import reload_helper\n")
        with SceneSource(path, self.Scene) as source:
            previous = source.module
            helper.unlink()
            with self.assertRaises(ModuleNotFoundError):
                source.reload()
            self.assertIs(source.module, previous)
            self.assertEqual(previous.reload_helper.VALUE, 1)

    def test_stdlib_import_function_and_external_modules_never_reloaded(self):
        path = self.write("scene.py", "import math, sys\nVALUE=1\n")
        import math
        previous_import = builtins.__import__
        with SceneSource(path, self.Scene) as source:
            self.assertIs(source.reload().math, math)
            self.assertIs(builtins.__import__, previous_import)
            self.assertIs(source.module.sys, sys)

    def test_declared_studio_sources_refuse_host_reload(self):
        path = self.write("scene.py", "VALUE=1\n")
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        with SceneSource(path, self.Scene, source_inputs={path: digest}) as source:
            with self.assertRaisesRegex(RuntimeError, "worker supervisor"):
                source.reload()

    def test_only_active_owner_thread_can_reload_or_close_during_reload(self):
        path = self.write("scene.py", "VALUE=1\n")
        source = SceneSource(path, self.Scene)
        with self.assertRaisesRegex(RuntimeError, "active"):
            source.reload()
        with source:
            failures = []
            def worker():
                try:
                    source.reload()
                except RuntimeError as error:
                    failures.append(str(error))
            thread = threading.Thread(target=worker)
            thread.start()
            thread.join()
            self.assertEqual(len(failures), 1)
            self.assertIn("thread", failures[0])
            path.write_text("from fmn_python.source_reload import active_source\nactive_source().reload()\n")
            with self.assertRaisesRegex(RuntimeError, "already in progress"):
                source.reload()
            path.write_text("from fmn_python.source_reload import active_source\nactive_source().__exit__(None,None,None)\n")
            with self.assertRaisesRegex(RuntimeError, "during reload"):
                source.reload()
            self.assertTrue(source._active)
        with self.assertRaisesRegex(RuntimeError, "active"):
            source.reload()

    def test_reload_count_and_byte_budgets_precede_execution(self):
        path = self.write("scene.py", "from fmn_reload_test_support import events\nevents.append(1)\n")
        with SceneSource(path, self.Scene) as source:
            for setting, limit in (("_MAX_INPUTS", 0), ("_MAX_BYTES", 1)):
                with patch.object(source_reload, setting, limit):
                    with self.assertRaisesRegex(ValueError, "budget"):
                        source.reload()
            self.assertEqual(self.events, [1])
            with patch.object(source_reload, "_MAX_BYTES", len(path.read_bytes())):
                source.reload()
            self.assertEqual(self.events, [1, 1])

    def test_new_imports_share_the_same_reload_budget(self):
        path = self.write("scene.py", "VALUE=1\n")
        self.write("reload_new.py", "VALUE=2\n")
        with SceneSource(path, self.Scene) as source:
            previous = source.module
            path.write_text("import reload_new\n")
            with patch.object(source_reload, "_MAX_INPUTS", 1):
                with self.assertRaisesRegex(ValueError, "count budget"):
                    source.reload()
            self.assertIs(source.module, previous)
            self.assertNotIn("reload_new", sys.modules)

    def test_colliding_new_project_filename_cannot_reuse_foreign_module(self):
        path = self.write("scene.py", "VALUE=1\n")
        with SceneSource(path, self.Scene) as source:
            previous = source.module
            self.write("math.py", "VALUE=99\n")
            path.write_text("import math\n")
            with self.assertRaisesRegex(ImportError, "another location"):
                source.reload()
            self.assertIs(source.module, previous)

    def test_explicit_module_replacement_is_preserved_and_refused(self):
        path = self.write("scene.py", "VALUE=1\n")
        with SceneSource(path, self.Scene) as source:
            replacement = ModuleType(source.name)
            sys.modules[source.name] = replacement
            with self.assertRaisesRegex(ImportError, "outside its source owner"):
                source.reload()
            self.assertIs(sys.modules[source.name], replacement)

    def test_context_restores_original_cached_modules_after_multiple_reloads(self):
        helper = self.write("reload_helper.py", "VALUE=1\n")
        path = self.write("scene.py", "from reload_helper import VALUE\n")
        sys.path.insert(0, str(self.root))
        self.addCleanup(sys.path.remove, str(self.root))
        original = importlib.import_module("reload_helper")
        original_path = list(sys.path)
        original_finders = list(sys.meta_path)
        with SceneSource(path, self.Scene) as source:
            for value in (2, 3, 4):
                helper.write_text(f"VALUE={value}\n")
                self.assertEqual(source.reload().VALUE, value)
        self.assertIs(sys.modules["reload_helper"], original)
        self.assertEqual(sys.path, original_path)
        self.assertEqual(sys.meta_path, original_finders)

    def test_keyboard_interrupt_rolls_back_import_graph_and_propagates(self):
        path = self.write("scene.py", "VALUE=1\n")
        with SceneSource(path, self.Scene) as source:
            previous = source.module
            path.write_text("raise KeyboardInterrupt\n")
            with self.assertRaises(KeyboardInterrupt):
                source.reload()
            self.assertIs(source.module, previous)
            self.assertIsNone(source._reload_inputs)


if __name__ == "__main__":
    unittest.main()
