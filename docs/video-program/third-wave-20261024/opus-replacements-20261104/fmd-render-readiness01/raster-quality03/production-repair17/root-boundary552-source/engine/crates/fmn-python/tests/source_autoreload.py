"""Installed-wheel source reload through real embedded IPython and native scenes.

The prompt loop is driven by scripted input, not a human terminal. IPython's
actual __call__/mainloop/events, the production Embedded/SceneConsole adapters,
native mobjects, checkpoints and Lumen preview all execute. Missing manimlib or
IPython is an error, not a skip or a fixture-storage fallback.
"""
from __future__ import annotations

import contextlib
import io
from pathlib import Path
import sys
import tempfile
import unittest

import manimlib as m
import numpy as np

from fmn_python.embedded_shell import _classes
from fmn_python.scene_loading import SceneSource
from fmn_python.source_reload import active_source


class NativeSourceAutoreloadAcceptance(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="fmn-native-source-reload-")
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        _, self.Embedded = _classes(m)

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        return path

    def run_cell(self, shell, code):
        stdout, stderr = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            result = shell.run_cell(code)
        self.assertTrue(result.success, (result.error_in_exec, result.error_before_exec,
                                         stdout.getvalue(), stderr.getvalue()))
        return stdout.getvalue() + stderr.getvalue()

    def session(self, scene, namespace, script):
        embedded = self.Embedded(scene)
        embedded._fmn_namespace = dict(namespace)
        embedded.shell = embedded.get_ipython_shell_for_embedded_scene()
        # Drive the real mainloop rather than replacing __call__, which would
        # miss IPython's temporary locals/module-globals separation.
        embedded.shell.interact = lambda: script(embedded)
        result = embedded.launch()
        self.assertIs(result, embedded.shell)
        self.assertEqual(embedded._fmn_hooks, [])
        self.assertNotIn("_fmn_source_autoreload", vars(embedded))
        self.assertNotIn("_fmn_scene_console", vars(scene))
        self.addCleanup(embedded.shell.history_manager.end_session)
        return embedded

    def scene(self):
        scene = m.Scene()
        scene.camera.reset_pixel_shape(128, 72)
        square = m.Square(side_length=.8).set_fill(m.BLUE, opacity=1)
        scene.add(square)
        return scene, square

    def test_live_native_scene_helpers_globals_checkpoints_and_preview_survive_reload(self):
        helper = self.write("live_helper.py", "from manimlib import RIGHT\ndef move(square): square.move_to(RIGHT)\n")
        path = self.write("scene.py", "from live_helper import move\n")
        scene, square = self.scene()
        time = scene.time
        callbacks, frames = [], []
        with SceneSource(path, m.Scene) as loaded:
            namespace = dict(vars(loaded.module), scene=scene, self=scene, square=square)
            def script(embedded):
                shell = embedded.shell
                self.assertIs(sys.modules[loaded.name], loaded.module)
                embedded.checkpoint_manager.handle_checkpoint_key(scene, "#before reload")
                saved = embedded.checkpoint_manager.checkpoint_states["#before reload"]
                self.run_cell(shell, "auto_reload()")
                frames.append(scene.camera.get_png())
                self.run_cell(shell, "move(square)")
                frames.append(scene.camera.get_png())
                np.testing.assert_allclose(square.get_center(), m.RIGHT, atol=1e-5)
                self.run_cell(shell, "def from_cell(): move(square)")
                helper.write_text("from manimlib import RIGHT\ndef move(square): square.move_to(2 * RIGHT)\n")
                self.run_cell(shell, "from_cell()")
                frames.append(scene.camera.get_png())
                np.testing.assert_allclose(square.get_center(), 2 * m.RIGHT, atol=1e-5)
                self.assertIs(shell.user_ns["square"], square)
                self.assertIs(shell.user_ns["scene"], scene)
                self.assertIs(embedded.checkpoint_manager.checkpoint_states["#before reload"], saved)
                self.assertEqual(scene.time, time)
                state = vars(embedded)["_fmn_source_autoreload"]
                callbacks.append(state.callback)
                self.assertIs(state.source, loaded)
            self.session(scene, namespace, script)
            module = loaded.module
            helper.write_text("from manimlib import RIGHT\ndef move(square): square.move_to(3 * RIGHT)\n")
            callbacks[0]()
            self.assertIs(loaded.module, module)
            self.assertTrue(loaded._active)
        self.assertTrue(all(isinstance(frame, bytes) and frame.startswith(b"\x89PNG\r\n\x1a\n") for frame in frames))
        self.assertNotEqual(frames[0], frames[1])
        self.assertNotEqual(frames[1], frames[2])

    def test_bad_edit_keeps_last_working_definitions_and_next_edit_recovers(self):
        helper = self.write("live_helper.py", "def value(): return 1\n")
        path = self.write("scene.py", "from live_helper import value\n")
        scene, square = self.scene()
        original_points = square.get_points().copy()
        with SceneSource(path, m.Scene) as loaded:
            namespace = dict(vars(loaded.module), scene=scene, square=square)
            def script(embedded):
                shell = embedded.shell
                self.run_cell(shell, "auto_reload()")
                previous = loaded.module
                helper.write_text("def value(\n")
                diagnostics = self.run_cell(shell, "observed=value()")
                self.assertIn("SyntaxError", diagnostics)
                self.assertEqual(shell.user_ns["observed"], 1)
                self.assertIs(loaded.module, previous)
                np.testing.assert_array_equal(square.get_points(), original_points)
                helper.write_text("def value(): return 7\n")
                self.run_cell(shell, "observed=value()")
                self.assertEqual(shell.user_ns["observed"], 7)
            self.session(scene, namespace, script)

    def test_shell_owned_source_scope_closes_without_replacing_native_objects(self):
        path = self.write("scene.py", "from manimlib import RIGHT\ndef move(square): square.move_to(RIGHT)\n")
        scene, square = self.scene()
        module_names = []
        def script(embedded):
            self.run_cell(embedded.shell, "reload_source(); move(square)")
            owned = active_source(path)
            self.assertIsNotNone(owned)
            module_names.extend(owned._loaded)
            np.testing.assert_allclose(square.get_center(), m.RIGHT, atol=1e-5)
        self.session(scene, {"__file__": str(path), "square": square}, script)
        self.assertIsNone(active_source(path))
        self.assertTrue(all(name not in sys.modules for name in module_names))
        self.assertTrue(any(root is square for root in scene.mobjects))

    def test_declared_worker_sources_remain_on_supervisor_reload_route(self):
        import hashlib
        path = self.write("scene.py", "VALUE=1\n")
        scene, square = self.scene()
        declared = {path: hashlib.sha256(path.read_bytes()).hexdigest()}
        with SceneSource(path, m.Scene, source_inputs=declared) as loaded:
            def script(embedded):
                previous = loaded.module
                with self.assertRaisesRegex(RuntimeError, "worker supervisor"):
                    embedded.reload_source()
                self.assertIs(loaded.module, previous)
                self.assertTrue(any(root is square for root in scene.mobjects))
            self.session(scene, dict(vars(loaded.module), scene=scene), script)


suite = unittest.defaultTestLoader.loadTestsFromTestCase(NativeSourceAutoreloadAcceptance)
if suite.countTestCases() != 4:
    raise AssertionError("native source autoreload acceptance must execute all four scenarios")
result = unittest.TextTestRunner(verbosity=2).run(suite)
if not result.wasSuccessful():
    raise AssertionError("native source autoreload acceptance failed")
