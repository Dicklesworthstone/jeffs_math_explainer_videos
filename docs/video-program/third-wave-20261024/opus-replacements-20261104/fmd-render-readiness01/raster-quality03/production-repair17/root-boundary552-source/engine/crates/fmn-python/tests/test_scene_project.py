"""Real source/import transactions with an explicit native lifecycle test double.

These tests execute the production SceneSource, reload and SceneProject code.
They are not native geometry, pixel, installed-wheel or certification evidence.
"""
from __future__ import annotations

import importlib
import os
from pathlib import Path
import sys
import tempfile
import threading
from types import ModuleType, SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python.scene_loading import SceneSource
from fmn_python.source_reload import active_source, reload_source
from fmn_python.scene_project import SceneProject


def native_fixture():
    native = ModuleType("manimlib")
    native.events = []
    native.preview_failure = None
    native.construct_failure = None
    native.constructed = 0

    class EndScene(Exception):
        pass

    class Snapshot:
        def __init__(self, cells):
            self.values = tuple(cells)
        def _repr_png_(self):
            return b"explicit snapshot double: " + repr(self.values).encode()

    class Camera:
        def capture_snapshot(self, *cells):
            native.events.append("preview")
            if native.preview_failure is not None:
                raise native.preview_failure
            return Snapshot(cells)

    class Scene:
        def __init__(self, **kwargs):
            native.constructed += 1
            if native.construct_failure is not None:
                raise native.construct_failure
            self.kwargs = kwargs
            self.mobjects = []
            self.camera = Camera()
            self.clock = 0
            self.undo_stack = []
        def setup(self):
            native.events.append("setup")
        def construct(self):
            pass
        def tear_down(self):
            native.events.append("tear_down")
        def run(self):
            self.setup()
            try:
                self.construct()
            except EndScene:
                pass
            finally:
                self.tear_down()
        def add(self, *objects):
            self.mobjects.extend(objects)
        def wait(self, dt=1):
            self.clock += dt
        def get_state(self):
            return list(self.mobjects), self.clock
        def restore_state(self, state):
            self.mobjects, self.clock = list(state[0]), state[1]

    native.Scene = Scene
    native.EndScene = EndScene
    return native


class SceneProjectTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="fmn-project-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.native = native_fixture()
        self.patch = patch.dict(sys.modules, {"manimlib": self.native})
        self.patch.start()
        self.addCleanup(self.patch.stop)

    def write(self, name, source):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(source)
        return path

    def source(self, value=1, extra="", name="scene.py"):
        return self.write(name, "from manimlib import Scene\n" +
                          f"VALUE = {value}\nclass Demo(Scene):\n"
                          "    def construct(self):\n"
                          "        self.add(VALUE)\n        self.wait(.25)\n" + extra)

    def project(self, path, **kwargs):
        return SceneProject(path, "Demo", _native=self.native, **kwargs)

    def test_initial_entry_runs_lifecycle_then_preview(self):
        path = self.source(4)
        project = self.project(path)
        self.assertEqual(project.generation, 0)
        self.assertEqual(self.native.constructed, 0)
        with project:
            self.assertEqual(project.scene.mobjects, [4])
            self.assertEqual(project.scene.clock, .25)
            self.assertEqual(self.native.events, ["setup", "tear_down", "preview"])
            self.assertEqual(project.preview.values, (4,))
            self.assertIs(project.namespace["self"], project.scene)
            self.assertIs(project.namespace["scene"], project.scene)
            self.assertIs(project.module, active_source(path).module)
        self.assertIsNone(active_source())
        self.assertIsNone(project.preview)

    def test_rebuild_reexecutes_construct_with_fresh_class_scene_and_preview(self):
        path = self.source(1)
        with self.project(path) as project:
            old, old_module, old_preview = project.scene, project.module, project.preview
            self.source(8)
            result = project.rebuild()
            self.assertIs(result, project.scene)
            self.assertIsNot(project.scene, old)
            self.assertIsNot(type(project.scene), type(old))
            self.assertIsNot(project.module, old_module)
            self.assertEqual(project.generation, 2)
            self.assertEqual(old.mobjects, [1])
            self.assertEqual(old_preview.values, (1,))
            self.assertEqual(project.scene.mobjects, [8])
            self.assertEqual(project.preview.values, (8,))
            self.assertEqual(project.scene.clock, .25)
            self.assertEqual(project.scene.undo_stack, [])

    def test_unchanged_conditional_rebuild_does_not_execute(self):
        with self.project(self.source()) as project:
            scene = project.scene
            before = list(self.native.events)
            self.assertIs(project.rebuild(if_changed=True), scene)
            self.assertEqual(project.generation, 1)
            self.assertEqual(self.native.events, before)
            scene.add(42)
            self.assertIsNot(project.rebuild(), scene)
            self.assertEqual(project.scene.mobjects, [1])

    def test_same_timestamp_transitive_and_lazy_helper_edits_are_seen(self):
        self.write("project_leaf.py", "VALUE = 1\n")
        helper = self.write("project_helper.py", "from project_leaf import VALUE\n")
        path = self.write("scene.py", "from manimlib import Scene\nclass Demo(Scene):\n"
                          "    def construct(self):\n        import project_helper\n"
                          "        self.add(project_helper.VALUE)\n")
        with self.project(path) as project:
            before = (self.root / "project_leaf.py").stat()
            self.write("project_leaf.py", "VALUE = 9\n")
            os.utime(self.root / "project_leaf.py", ns=(before.st_atime_ns, before.st_mtime_ns))
            project.rebuild(if_changed=True)
            self.assertEqual(project.scene.mobjects, [9])
            self.assertIn(helper, active_source(path).source_digests)
        self.assertNotIn("project_helper", sys.modules)
        self.assertNotIn("project_leaf", sys.modules)

    def test_constructor_options_keep_shallow_snapshot_and_are_reused(self):
        options = {"amount": 3}
        path = self.write("scene.py", "from manimlib import Scene\nclass Demo(Scene):\n"
                          "    def construct(self):\n        self.add(self.kwargs['amount'])\n")
        project = self.project(path, scene_kwargs=options)
        options["amount"] = 10
        with project:
            self.assertEqual(project.scene.mobjects, [3])
            project.scene.kwargs["amount"] = 20
            project.rebuild()
            self.assertEqual(project.scene.mobjects, [3])

    def test_construct_failure_restores_previous_module_graph_and_scene(self):
        helper = self.write("project_helper.py", "VALUE = 1\n")
        path = self.write("scene.py", "from manimlib import Scene\nimport project_helper\n"
                          "class Demo(Scene):\n    def construct(self):\n        self.add(project_helper.VALUE)\n")
        with self.project(path) as project:
            old_scene, old_module, old_helper = project.scene, project.module, sys.modules["project_helper"]
            old_preview = project.preview
            helper.write_text("VALUE = 2\n")
            with path.open("a") as stream:
                stream.write("        raise LookupError('failed construction')\n")
            with self.assertRaisesRegex(LookupError, "failed construction"):
                project.rebuild()
            self.assertIs(project.scene, old_scene)
            self.assertIs(project.module, old_module)
            self.assertIs(active_source(path).module, old_module)
            self.assertIs(sys.modules["project_helper"], old_helper)
            self.assertIs(project.preview, old_preview)
            self.assertEqual(project.generation, 1)
            self.assertEqual(old_scene.mobjects, [1])
            path.write_text(path.read_text().replace("        raise LookupError('failed construction')\n", ""))
            project.rebuild()
            self.assertEqual(project.scene.mobjects, [2])

    def test_failure_in_preview_preserves_working_pair(self):
        path = self.source(1)
        with self.project(path) as project:
            old_scene, old_module, old_preview = project.scene, project.module, project.preview
            self.source(2)
            error = OSError("capture failed")
            self.native.preview_failure = error
            with self.assertRaises(OSError) as caught:
                project.rebuild()
            self.assertIs(caught.exception, error)
            self.assertIs(project.scene, old_scene)
            self.assertIs(project.module, old_module)
            self.assertIs(project.preview, old_preview)
            self.native.preview_failure = None
            project.rebuild()
            self.assertEqual(project.preview.values, (2,))

    def test_constructor_setup_and_teardown_failures_preserve_generation(self):
        path = self.source(1)
        with self.project(path) as project:
            old = project.scene
            for name in ("__init__", "setup", "tear_down"):
                self.source(2, f"    def {name}(self, **kwargs):\n        raise ArithmeticError('{name}')\n")
                with self.subTest(name=name), self.assertRaisesRegex(ArithmeticError, name):
                    project.rebuild()
                self.assertIs(project.scene, old)
                self.assertEqual(project.module.VALUE, 1)
                self.assertEqual(project.generation, 1)
            self.source(3)
            project.rebuild()
            self.assertEqual(project.scene.mobjects, [3])

    def test_syntax_error_executes_nothing_and_is_retryable(self):
        path = self.source()
        with self.project(path) as project:
            before, scene = list(self.native.events), project.scene
            path.write_text("not valid python !!!\n")
            with self.assertRaises(SyntaxError):
                project.rebuild()
            self.assertEqual(self.native.events, before)
            self.assertIs(project.scene, scene)
            self.source(5)
            project.rebuild()
            self.assertEqual(project.scene.mobjects, [5])

    def test_removed_class_and_import_error_leave_existing_scene(self):
        path = self.source()
        with self.project(path) as project:
            old = project.scene
            for text, error in (("VALUE = 9\n", ValueError), ("import no_such_project_module_xyz\n", ModuleNotFoundError)):
                path.write_text(text)
                with self.assertRaises(error):
                    project.rebuild()
                self.assertIs(project.scene, old)
                self.assertEqual(project.module.VALUE, 1)

    def test_new_lazy_modules_are_removed_after_failed_build(self):
        path = self.source()
        with self.project(path) as project:
            self.write("project_new_helper.py", "VALUE = 7\n")
            self.source(2, "        import project_new_helper\n        raise ValueError('after import')\n")
            with self.assertRaises(ValueError):
                project.rebuild()
            self.assertNotIn("project_new_helper", sys.modules)
            self.assertEqual(project.module.VALUE, 1)

    def test_rebuild_after_separate_definition_reload_does_not_skip(self):
        path = self.source()
        with self.project(path) as project:
            old = project.scene
            self.source(7)
            active_source(path).reload()
            self.assertIs(project.scene, old)
            project.rebuild(if_changed=True)
            self.assertIsNot(project.scene, old)
            self.assertEqual(project.scene.mobjects, [7])

    def test_package_relative_imports_and_dataclasses_work_during_build(self):
        self.write("demo_pkg/__init__.py", "")
        self.write("demo_pkg/helper.py", "VALUE = 1\n")
        path = self.write("demo_pkg/scene.py", "from manimlib import Scene\nfrom dataclasses import dataclass\n"
                          "from .helper import VALUE\n@dataclass\nclass Data:\n    x: int = VALUE\n"
                          "class Demo(Scene):\n    def construct(self):\n        self.add(Data().x)\n")
        with self.project(path) as project:
            self.write("demo_pkg/helper.py", "VALUE = 2\n")
            project.rebuild()
            self.assertEqual(project.scene.mobjects, [2])
            self.assertEqual(project.module.Data().x, 2)

    def test_inherited_or_imported_scene_class_is_not_misidentified_as_local(self):
        self.write("project_helper.py", "from manimlib import Scene\nclass Demo(Scene): pass\n")
        path = self.write("scene.py", "from project_helper import Demo\n")
        with self.assertRaisesRegex(ValueError, "not declared"):
            with self.project(path):
                pass
        self.assertIsNone(active_source())

    def test_existing_source_context_is_reused_and_never_closed(self):
        path = self.source()
        with SceneSource(path, self.native.Scene) as loaded:
            with self.project(loaded) as project:
                self.source(2)
                project.rebuild()
            self.assertIs(active_source(path), loaded)
            self.assertEqual(loaded.module.VALUE, 2)
        self.assertIsNone(active_source())

    def test_failed_initial_build_closes_only_owned_source(self):
        path = self.source(extra="        raise ValueError('first build')\n")
        with self.assertRaises(ValueError):
            with self.project(path):
                pass
        self.assertIsNone(active_source())
        with SceneSource(path, self.native.Scene) as loaded:
            with self.assertRaises(ValueError):
                with self.project(loaded):
                    pass
            self.assertIs(active_source(path), loaded)

    def test_another_project_cannot_steal_import_owner(self):
        path = self.source()
        with self.project(path) as project:
            with self.assertRaisesRegex(RuntimeError, "another scene source"):
                with self.project(path):
                    pass
            self.assertIs(active_source(path).module, project.module)

    def test_context_reentry_and_inactive_rebuild_are_refused(self):
        project = self.project(self.source())
        with self.assertRaises(RuntimeError):
            project.rebuild()
        with project:
            with self.assertRaises(RuntimeError):
                project.__enter__()
        project.close()
        with self.assertRaises(RuntimeError):
            project.rebuild()
        with self.assertRaises(RuntimeError):
            _ = project.scene

    def test_cross_thread_access_refuses_without_touching_native_scene(self):
        with self.project(self.source()) as project:
            failures = []
            def worker():
                for call in (project.rebuild, project.close, lambda: project.scene, lambda: project.namespace):
                    try:
                        call()
                    except RuntimeError as error:
                        failures.append(str(error))
                self.assertIsNone(project._repr_png_())
            thread = threading.Thread(target=worker)
            thread.start()
            thread.join()
            self.assertEqual(len(failures), 4)
            self.assertTrue(all("creating thread" in failure for failure in failures))
            self.assertEqual(project.generation, 1)

    def test_active_output_segment_or_console_prevents_rebuild(self):
        with self.project(self.source()) as project:
            for key in ("_fmn_owned_render_session", "_fmn_scene_execution",
                        "_fmn_scene_console", "_fmn_studio_worker_request"):
                vars(project.scene)[key] = object()
                with self.subTest(key=key), self.assertRaises(RuntimeError):
                    project.rebuild()
                self.assertEqual(project.generation, 1)
                vars(project.scene).pop(key)

    def test_authored_reentrancy_and_close_during_build_are_refused(self):
        path = self.source()
        with self.project(path) as project:
            self.native.project = project
            self.source(2, "        from manimlib import project\n        project.rebuild()\n")
            with self.assertRaisesRegex(RuntimeError, "already in progress"):
                project.rebuild()
            self.assertEqual(project.scene.mobjects, [1])
            self.source(2, "        from manimlib import project\n        project.close()\n")
            with self.assertRaisesRegex(RuntimeError, "during reconstruction"):
                project.rebuild()
            self.native.project = None

    def test_early_completion_uses_normal_scene_lifecycle(self):
        path = self.source(2, "        from manimlib import EndScene\n        raise EndScene()\n")
        with self.project(path) as project:
            self.assertEqual(project.scene.mobjects, [2])
            self.assertEqual(self.native.events, ["setup", "tear_down", "preview"])

    def test_capture_can_be_disabled_without_faking_a_preview(self):
        self.native.preview_failure = AssertionError("must not capture")
        with self.project(self.source(), capture=False) as project:
            self.assertIsNone(project.preview)
            self.assertIsNone(project._repr_png_())
            project.rebuild()
            self.assertNotIn("preview", self.native.events)

    def test_namespace_snapshot_is_detached(self):
        with self.project(self.source()) as project:
            namespace = project.namespace
            namespace["VALUE"] = 99
            self.assertEqual(project.namespace["VALUE"], 1)
            self.assertEqual(project.module.VALUE, 1)

    def test_returning_previous_scene_is_refused_before_run(self):
        path = self.source()
        with self.project(path) as project:
            self.native.previous = project.scene
            self.source(extra="    def __new__(cls):\n        from manimlib import previous\n        return previous\n")
            with self.assertRaisesRegex(RuntimeError, "fresh instance"):
                project.rebuild()
            self.assertEqual(project.scene.mobjects, [1])
            self.native.previous = None

    def test_async_run_is_rejected_instead_of_publishing_an_unexecuted_scene(self):
        path = self.source(extra="    async def run(self):\n        self.add(99)\n")
        with self.assertRaisesRegex(TypeError, "synchronously"):
            with self.project(path):
                pass
        self.assertIsNone(active_source())

    def test_validation_and_declared_studio_inputs(self):
        path = self.source()
        for name in ("", "a.b", 3, "x" * 513):
            with self.assertRaises(ValueError):
                SceneProject(path, name, _native=self.native)
        for kwargs in ({"capture": 1}, {"scene_kwargs": []}, {"scene_kwargs": {1: 2}}):
            with self.assertRaises(TypeError):
                self.project(path, **kwargs)
        import hashlib
        declared = {path: hashlib.sha256(path.read_bytes()).hexdigest()}
        loaded = SceneSource(path, self.native.Scene, source_inputs=declared)
        with self.assertRaisesRegex(RuntimeError, "supervisor"):
            self.project(loaded)

    def test_failed_rebuild_keeps_mixed_source_provenance_refusal(self):
        path = self.source()
        with self.project(path) as project:
            self.source(2, "        raise ValueError('failure')\n")
            with self.assertRaises(ValueError):
                project.rebuild()
            with self.assertRaisesRegex(RuntimeError, "changed during execution"):
                _ = active_source(path).sources

    def test_preparation_runs_inside_import_transaction(self):
        path = self.source()
        with SceneSource(path, self.native.Scene) as loaded:
            old = loaded.module
            self.source(2)
            def prepare(module):
                self.assertIs(loaded.module, module)
                self.assertIs(loaded.scenes["Demo"], module.Demo)
                self.assertIsNot(module, old)
                raise OSError("validation failed")
            with self.assertRaises(OSError):
                reload_source(loaded, _prepare=prepare)
            self.assertIs(loaded.module, old)
            self.assertIs(sys.modules[loaded.name], old)
            with self.assertRaises(TypeError):
                reload_source(loaded, _prepare=1)


if __name__ == "__main__":
    unittest.main()
