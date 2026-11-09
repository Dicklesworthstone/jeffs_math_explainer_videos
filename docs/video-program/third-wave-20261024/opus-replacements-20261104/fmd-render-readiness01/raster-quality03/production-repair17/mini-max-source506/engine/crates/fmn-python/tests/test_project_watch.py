"""Live-watch filesystem, scheduling, and source-transaction regressions.

Scheduling tests use an explicit project double. Integration tests execute real
SceneSource/reload/SceneProject code with test_scene_project.native_fixture.
These are not native geometry, preview pixel, or installed-wheel evidence.
"""
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import sys
import tempfile
import threading
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python import project_watch
from fmn_python.scene_project import SceneProject
from fmn_python.source_reload import active_source
from test_scene_project import native_fixture


class ProjectDouble:
    def __init__(self, root):
        self.path = root / "scene.py"
        self.path.write_text("1\n")
        self._source = SimpleNamespace(path=self.path, root=root, source_digests={})
        self._editor = None
        self.thread = threading.get_ident()
        self.active = True
        self.generation = 1
        self.scene = 1
        self.preview = 1
        self.calls = []
        self.on_build = None
        self.remember_sources()

    def remember_sources(self):
        self._source.source_digests = {
            path: hashlib.sha256(path.read_bytes()).hexdigest()
            for path in self._source.root.glob("*.py")
        }

    def _check(self):
        if self.thread != threading.get_ident():
            raise RuntimeError("creating thread")
        if not self.active:
            raise RuntimeError("active context")

    def rebuild(self, *, if_changed=False):
        self._check()
        self.calls.append(if_changed)
        if if_changed and all(path.exists() and hashlib.sha256(path.read_bytes()).hexdigest() == digest
                              for path, digest in self._source.source_digests.items()):
            return self.scene
        value = int(self.path.read_text())
        if self.on_build is not None:
            self.on_build()
        self.generation += 1
        self.scene = self.preview = value
        self.remember_sources()
        return self.scene


class ScriptedStop(threading.Event):
    """Advance virtual time and apply edits without sleeps or watcher threads."""
    def __init__(self, actions, clock):
        super().__init__()
        self.actions = iter(actions)
        self.clock = clock
        self.waits = []

    def wait(self, timeout=None):
        self.waits.append(timeout)
        if self.is_set():
            return True
        self.clock[0] += timeout
        action = next(self.actions, None)
        if action is None:
            self.set()
            return True
        action()
        return self.is_set()


class ProjectWatchTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="fmn-project-watch-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.project = ProjectDouble(self.root)
        self.clock = [0.0]
        timer = patch.object(project_watch.time, "monotonic", side_effect=lambda: self.clock[0])
        timer.start()
        self.addCleanup(timer.stop)

    def edit(self, value):
        return lambda: self.project.path.write_text(str(value) + "\n")

    def run_watch(self, actions=(), **kwargs):
        stop = ScriptedStop(actions, self.clock)
        results = list(project_watch.watch_project(self.project, stop=stop,
                       poll_interval=.25, **{"debounce": 0, **kwargs}))
        return results, stop

    def test_unchanged_sources_never_reconstruct_the_scene(self):
        scenes, stop = self.run_watch([lambda: None] * 4)
        self.assertEqual(scenes, [1])
        self.assertEqual(self.project.generation, 1)
        self.assertEqual(self.project.calls, [True])
        self.assertTrue(stop.is_set())

    def test_edit_before_start_is_not_mistaken_for_built_content(self):
        self.edit(2)()
        scenes, _ = self.run_watch()
        self.assertEqual(scenes, [1, 2])
        self.assertEqual(self.project.calls, [True])

    def test_content_edits_preserving_size_and_mtime_are_detected(self):
        original = self.project.path.stat()
        def edit():
            self.edit(9)()
            os.utime(self.project.path, ns=(original.st_atime_ns, original.st_mtime_ns))
        scenes, _ = self.run_watch([edit])
        self.assertEqual(scenes, [1, 9])

    def test_rapid_edits_are_coalesced_until_stable(self):
        scenes, _ = self.run_watch([self.edit(2), self.edit(3), lambda: None, lambda: None], debounce=.5)
        self.assertEqual(scenes, [1, 3])
        self.assertEqual(self.project.calls, [False])

    def test_failure_is_reported_once_then_recovers_after_edit(self):
        failures = []
        scenes, _ = self.run_watch([self.edit("invalid"), lambda: None,
                                    lambda: None, self.edit(5)], on_error=failures.append)
        self.assertEqual(scenes, [1, 5])
        self.assertEqual(len(failures), 1)
        self.assertIsInstance(failures[0], ValueError)
        self.assertEqual(self.project.calls, [True, False, False])
        self.assertEqual(self.project.preview, 5)

    def test_missing_new_helper_can_recover_without_another_entry_edit(self):
        helper = self.root / "new_helper.py"
        def build():
            if self.project.path.read_text().strip() == "2":
                if not helper.exists():
                    raise ModuleNotFoundError("new_helper")
                self.assertEqual(helper.read_text(), "VALUE = 7\n")
        self.project.on_build = build
        failures = []
        scenes, _ = self.run_watch([self.edit(2), lambda: None,
                                    lambda: helper.write_text("VALUE = 7\n")], on_error=failures.append)
        self.assertEqual(scenes, [1, 2])
        self.assertEqual(len(failures), 1)
        self.assertEqual(self.project.generation, 2)

    def test_new_nested_helper_file_is_watched(self):
        folder = self.root / "helpers"
        folder.mkdir()
        scenes, _ = self.run_watch([lambda: (folder / "leaf.py").write_text("VALUE = 2\n")])
        self.assertEqual(scenes, [1, 1])
        self.assertEqual(self.project.generation, 2)

    def test_deleted_source_keeps_preview_and_recreation_recovers(self):
        failures = []
        scenes, _ = self.run_watch([self.project.path.unlink, lambda: None, self.edit(8)], on_error=failures.append)
        self.assertEqual(scenes, [1, 8])
        self.assertEqual(len(failures), 1)
        self.assertIsInstance(failures[0], FileNotFoundError)

    def test_explicit_asset_changes_force_reconstruction(self):
        asset = self.root / "image.dat"
        asset.write_bytes(b"first")
        scenes, _ = self.run_watch([lambda: asset.write_bytes(b"other")], paths=[asset])
        self.assertEqual(scenes, [1, 1])
        self.assertEqual(self.project.calls, [True, False])

    def test_asset_edit_while_initial_scene_is_yielded_is_not_lost(self):
        asset = self.root / "image.dat"
        asset.write_bytes(b"first")
        stop = ScriptedStop([], self.clock)
        stream = project_watch.watch_project(self.project, stop=stop, debounce=0, paths=[asset])
        self.addCleanup(stream.close)
        self.assertEqual(next(stream), 1)
        asset.write_bytes(b"other")
        self.assertEqual(list(stream), [1])
        self.assertEqual(self.project.calls, [False])

    def test_edits_during_reconstruction_trigger_the_next_generation(self):
        def build():
            if self.project.path.read_text().strip() == "2":
                self.edit(3)()
        self.project.on_build = build
        scenes, _ = self.run_watch([self.edit(2), lambda: None])
        self.assertEqual(scenes, [1, 2, 3])

    def test_scan_failure_deduplicates_and_restarts_debounce(self):
        failures = []
        with patch.object(project_watch, "_MAX_BYTES", 8):
            scenes, _ = self.run_watch([self.edit("x" * 9), lambda: None, self.edit(4)], on_error=failures.append)
        self.assertEqual(scenes, [1, 4])
        self.assertEqual(len(failures), 1)
        self.assertIn("byte budget", str(failures[0]))

    def test_default_error_handling_propagates(self):
        with self.assertRaises(ValueError):
            self.run_watch([self.edit("bad")])
        self.assertEqual(self.project.scene, 1)

    def test_callback_errors_and_process_interrupts_propagate(self):
        def callback(error):
            raise LookupError("host callback failed")
        with self.assertRaisesRegex(LookupError, "host callback"):
            self.run_watch([self.edit("bad")], on_error=callback)
        for error in (KeyboardInterrupt(), SystemExit(3)):
            with self.subTest(error=type(error)):
                self.edit(1)()
                self.project.remember_sources()
                def build():
                    raise error
                self.project.on_build = build
                failures = []
                with self.assertRaises(type(error)):
                    self.run_watch([self.edit(2)], on_error=failures.append)
                self.assertEqual(failures, [])

    def test_stopped_iterator_does_not_scan_or_yield(self):
        stop = threading.Event()
        stop.set()
        with patch.object(project_watch, "_snapshot", side_effect=AssertionError("scanned")):
            self.assertEqual(list(project_watch.watch_project(self.project, stop=stop)), [])

    def test_context_and_thread_ownership_are_enforced(self):
        self.project.active = False
        with self.assertRaisesRegex(RuntimeError, "active context"):
            next(project_watch.watch_project(self.project))
        self.project.active = True
        failures = []
        def worker():
            try:
                next(project_watch.watch_project(self.project))
            except RuntimeError as error:
                failures.append(str(error))
        thread = threading.Thread(target=worker)
        thread.start()
        thread.join()
        self.assertEqual(failures, ["creating thread"])

    def test_project_close_and_editor_are_checked_on_resume(self):
        stream = project_watch.watch_project(self.project)
        next(stream)
        self.project.active = False
        with self.assertRaisesRegex(RuntimeError, "active context"):
            next(stream)
        self.project.active = True
        stream = project_watch.watch_project(self.project)
        next(stream)
        self.project._editor = object()
        with self.assertRaisesRegex(RuntimeError, "active editor"):
            next(stream)

    def test_cancellation_during_scan_never_starts_a_rebuild(self):
        stop = threading.Event()
        original = project_watch._snapshot
        scans = 0
        def snapshot(*args):
            nonlocal scans
            scans += 1
            result = original(*args)
            if scans == 2:
                stop.set()
            return result
        with patch.object(project_watch, "_snapshot", side_effect=snapshot):
            stream = project_watch.watch_project(self.project, stop=stop, debounce=0)
            self.assertEqual(next(stream), 1)
            self.edit(2)()
            self.assertEqual(list(stream), [])
        self.assertEqual(self.project.calls, [])
        self.assertEqual(self.project.generation, 1)

    def test_scan_outage_cannot_hide_an_initial_asset_edit(self):
        asset = self.root / "image.dat"
        asset.write_bytes(b"before")
        stop = ScriptedStop([lambda: None], self.clock)
        original = project_watch._snapshot
        scans = 0
        failures = []
        def snapshot(*args):
            nonlocal scans
            scans += 1
            if scans == 2:
                asset.write_bytes(b"after")
                raise PermissionError("temporary scan outage")
            return original(*args)
        with patch.object(project_watch, "_snapshot", side_effect=snapshot):
            scenes = list(project_watch.watch_project(self.project, stop=stop,
                          debounce=0, paths=[asset], on_error=failures.append))
        self.assertEqual(scenes, [1, 1])
        self.assertEqual(self.project.calls, [False])
        self.assertEqual(len(failures), 1)

    def test_invalid_options_are_rejected(self):
        cases = ({"poll_interval": 0}, {"poll_interval": -1}, {"poll_interval": float("inf")},
                 {"poll_interval": True}, {"debounce": -1}, {"debounce": float("nan")},
                 {"stop": object()}, {"on_error": 1}, {"paths": "scene.py"})
        for kwargs in cases:
            with self.subTest(kwargs=kwargs), self.assertRaises((TypeError, ValueError)):
                next(project_watch.watch_project(self.project, **kwargs))

    def test_file_budget_and_nonregular_explicit_inputs_are_refused(self):
        with patch.object(project_watch, "_MAX_FILES", 1):
            (self.root / "extra.py").write_text("1")
            with self.assertRaisesRegex(ValueError, "file count budget"):
                next(project_watch.watch_project(self.project))
        with self.assertRaisesRegex(ValueError, "regular file"):
            next(project_watch.watch_project(self.project, paths=[self.root]))

    def test_ignored_directory_is_included_when_imported(self):
        hidden = self.root / ".hidden"
        hidden.mkdir()
        helper = hidden / "helper.py"
        helper.write_text("VALUE = 1\n")
        self.project._source.source_digests[helper] = hashlib.sha256(helper.read_bytes()).hexdigest()
        scenes, _ = self.run_watch([lambda: helper.write_text("VALUE = 2\n")])
        self.assertEqual(scenes, [1, 1])


class ProjectWatchIntegrationTests(unittest.TestCase):
    """Production imports/rebuilds, using the existing explicit native fixture."""
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="fmn-watch-integration-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.path = self.root / "scene.py"
        self.native = native_fixture()
        modules = patch.dict(sys.modules, {"manimlib": self.native})
        modules.start()
        self.addCleanup(modules.stop)
        self.clock = [0.0]
        timer = patch.object(project_watch.time, "monotonic", side_effect=lambda: self.clock[0])
        timer.start()
        self.addCleanup(timer.stop)
        self.source("self.add(1)")

    def source(self, body):
        self.path.write_text("from manimlib import Scene\nclass Demo(Scene):\n"
                             "    def construct(self):\n" +
                             "\n".join("        " + line for line in body.splitlines()) + "\n")

    def consume(self, project, actions=(), **kwargs):
        stop = ScriptedStop(actions, self.clock)
        return list(project.watch(stop=stop, poll_interval=.25, debounce=0, **kwargs))

    def test_public_watch_leaves_unchanged_generation_alone(self):
        with SceneProject(self.path, "Demo", _native=self.native) as project:
            original = project.scene
            self.assertEqual(self.consume(project, [lambda: None] * 3), [original])
            self.assertEqual(self.native.constructed, 1)
            self.assertEqual(project.generation, 1)
        self.assertIsNone(active_source())

    def test_transitive_helper_edit_produces_matching_scene_and_preview(self):
        leaf = self.root / "watch_leaf.py"
        leaf.write_text("VALUE = 1\n")
        (self.root / "watch_helper.py").write_text("from watch_leaf import VALUE\n")
        self.source("from watch_helper import VALUE\nself.add(VALUE)")
        original_stat = leaf.stat()
        def edit():
            leaf.write_text("VALUE = 9\n")
            os.utime(leaf, ns=(original_stat.st_atime_ns, original_stat.st_mtime_ns))
        with SceneProject(self.path, "Demo", _native=self.native) as project:
            scenes = self.consume(project, [edit])
            self.assertEqual([scene.mobjects for scene in scenes], [[1], [9]])
            self.assertIsNot(type(scenes[0]), type(scenes[1]))
            self.assertEqual(project.preview.values, (9,))
            self.assertEqual(project.generation, 2)
        self.assertNotIn("watch_leaf", sys.modules)
        self.assertNotIn("watch_helper", sys.modules)

    def test_syntax_failure_preserves_module_scene_and_preview_until_fixed(self):
        failures = []
        with SceneProject(self.path, "Demo", _native=self.native) as project:
            original = (project.scene, project.module, project.preview)
            def failed(error):
                failures.append(error)
                self.assertIs(project.scene, original[0])
                self.assertIs(project.module, original[1])
                self.assertIs(project.preview, original[2])
                self.assertEqual(project.generation, 1)
            scenes = self.consume(project, [lambda: self.path.write_text("broken !!!\n"),
                                  lambda: None, lambda: self.source("self.add(4)")], on_error=failed)
            self.assertEqual([scene.mobjects for scene in scenes], [[1], [4]])
            self.assertEqual(len(failures), 1)
            self.assertIsInstance(failures[0], SyntaxError)
            self.assertEqual(self.native.constructed, 2)

    def test_new_helper_creation_recovers_a_real_failed_import(self):
        failures = []
        with SceneProject(self.path, "Demo", _native=self.native) as project:
            scenes = self.consume(project, [
                lambda: self.source("from watch_new_helper import VALUE\nself.add(VALUE)"),
                lambda: None,
                lambda: (self.root / "watch_new_helper.py").write_text("VALUE = 7\n"),
            ], on_error=failures.append)
            self.assertEqual([scene.mobjects for scene in scenes], [[1], [7]])
            self.assertEqual(len(failures), 1)
            self.assertIsInstance(failures[0], ModuleNotFoundError)
            self.assertEqual(project.preview.values, (7,))
        self.assertNotIn("watch_new_helper", sys.modules)

    def test_asset_edits_rebuild_even_when_python_source_is_unchanged(self):
        asset = self.root / "label.txt"
        asset.write_text("before")
        self.source(f"from pathlib import Path\nself.add(Path({str(asset)!r}).read_text())")
        with SceneProject(self.path, "Demo", _native=self.native) as project:
            scenes = self.consume(project, [lambda: asset.write_text("after")], paths=[asset])
            self.assertEqual([scene.mobjects for scene in scenes], [["before"], ["after"]])
            self.assertEqual(project.preview.values, ("after",))

    def test_preview_failure_uses_existing_transaction_rollback(self):
        failures = []
        with SceneProject(self.path, "Demo", _native=self.native) as project:
            original = (project.scene, project.module, project.preview)
            def break_capture():
                self.native.preview_failure = OSError("capture failure")
                self.source("self.add(2)")
            def failed(error):
                failures.append(error)
                self.assertIs(project.scene, original[0])
                self.assertIs(project.module, original[1])
                self.assertIs(project.preview, original[2])
            def recover():
                self.native.preview_failure = None
                self.source("self.add(3)")
            scenes = self.consume(project, [break_capture, lambda: None, recover], on_error=failed)
            self.assertEqual([scene.mobjects for scene in scenes], [[1], [3]])
            self.assertEqual(len(failures), 1)
            self.assertEqual(project.generation, 2)


if __name__ == "__main__":
    unittest.main()
