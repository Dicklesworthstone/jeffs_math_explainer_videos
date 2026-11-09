"""Real embed adapter and cell execution over explicit Scene/shell doubles.

The separate real-IPython and native acceptance suites exercise those layers.
No test in this file claims to render native pixels or run a terminal UI.
"""
import inspect
from pathlib import Path
import sys
import threading
from types import ModuleType, SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python import SceneConsole
from fmn_python import embedded_shell as adapter
from fmn_python.scene_console import _OWNER, _STUDIO_WORKER
from test_scene_console_protocol import Scene as SceneBoundary, CapabilityError


class Events:
    def __init__(self):
        self.callbacks, self.retained = {}, []
        self.error = None
    def register(self, name, callback):
        self.callbacks.setdefault(name, []).append(callback)
        self.retained.append(callback)
    def unregister(self, name, callback):
        if self.error is not None:
            raise self.error
        self.callbacks[name].remove(callback)
    def trigger(self, name, *args):
        for callback in tuple(self.callbacks.get(name, ())):
            callback(*args)


class Shell:
    def __init__(self, body=None):
        self.body = body or (lambda shell: None)
        self.user_module = ModuleType("_test_shell")
        self.user_ns = self.user_module.__dict__
        self.events, self.calls = Events(), []
    def run_cell(self, source):
        self.calls.append(source)
        result = SimpleNamespace(error_before_exec=None, error_in_exec=None)
        try:
            exec(compile(source, "<test-shell>", "exec"), self.user_ns, self.user_ns)
        except BaseException as error:
            result.error_in_exec = error
        finally:
            self.events.trigger("post_run_cell", result)
        return result
    def __call__(self, **options):
        assert options["local_ns"] is self.user_ns
        assert options["module"] is self.user_module
        return self.body(self)


def native_classes():
    # Each installation owns independent classes, like separate native modules.
    class Scene(SceneBoundary):
        def add(self, *objects):
            self.mobjects.extend(objects)
        def embed(self, close_scene_on_exit=True, show_animation_progress=False):
            return None
    class InteractiveScene(Scene):
        pass
    class CheckpointManager:
        @staticmethod
        def get_leading_comment(code_string):
            line = code_string.partition("\n")[0].lstrip()
            return line if line.startswith("#") else ""
    class InteractiveSceneEmbed:
        def get_shortcuts(self):
            return {"add": self.scene.add}
    for cls in (CheckpointManager, InteractiveSceneEmbed):
        cls.__module__ = "manimlib.scene.scene_embed"
    native = SimpleNamespace(Scene=Scene, InteractiveScene=InteractiveScene,
                             CheckpointManager=CheckpointManager,
                             InteractiveSceneEmbed=InteractiveSceneEmbed,
                             _CapabilityError=CapabilityError)
    adapter.install_embedded_shell(native)
    return native


class EmbeddedShellTests(unittest.TestCase):
    def setUp(self):
        self.native = native_classes()
        self.scene = self.native.InteractiveScene()
        self.embed = self.native.InteractiveSceneEmbed(self.scene)
        self.box = self.scene.mobjects[0]
    def shell(self, body):
        shell = Shell(body)
        shell.user_ns.update(box=self.box)
        self.embed.shell = shell
        return shell
    def assert_released(self):
        self.assertNotIn(_OWNER, vars(self.scene))
        self.assertNotIn(adapter._SESSION, vars(self.scene))
        self.assertIsNone(self.embed._fmn_console)
        self.assertFalse(self.embed._fmn_launching)
        self.assertIsNone(adapter._ACTIVE.get())
    def test_constructor_is_headless_and_installation_keeps_class_identity(self):
        classes = self.native.CheckpointManager, self.native.InteractiveSceneEmbed
        with patch.object(adapter.importlib, "import_module", side_effect=AssertionError("unexpected import")):
            headless = classes[1](self.scene)
            adapter.install_embedded_shell(self.native)
        self.assertIsNone(headless.shell)
        self.assertEqual(classes, (self.native.CheckpointManager, self.native.InteractiveSceneEmbed))
        self.assertEqual(str(inspect.signature(classes[0])), "()")
        self.assertEqual(str(inspect.signature(classes[1])), "(scene)")
        self.assertEqual(str(inspect.signature(classes[1].checkpoint_paste)),
                         "(self, skip=False, record=False, progress_bar=True, *, record_to=None, recording_options=None)")
        self.assertIsNone(self.native.Scene().embed())
    def test_paste_forwards_explicit_recording_destination(self):
        calls = []
        self.embed._fmn_launching = True
        self.embed._fmn_console = SimpleNamespace(checkpoint_paste=lambda **kw: calls.append(kw))
        self.embed.checkpoint_paste(record_to="clip.y4m", recording_options={"threads": 4})
        self.assertEqual(calls, [{"skip": False, "record": False, "progress_bar": True,
                                  "record_to": "clip.y4m", "recording_options": {"threads": 4}}])
    def test_launch_installs_real_bound_shortcuts_and_removes_callbacks(self):
        def body(shell):
            self.assertEqual(shell.user_ns["add"], self.scene.add)
            self.assertEqual(shell.user_ns["checkpoint_paste"], self.embed.checkpoint_paste)
            shell.run_cell("box.x = 4")
            self.assertEqual(len(self.scene.camera.captures), 1)
            self.embed.ensure_frame_update_post_cell()
            self.assertEqual(len(shell.events.retained), 1)
        shell = self.shell(body)
        self.assertIs(self.embed.launch(), shell)
        self.assertEqual(self.box.x, 4)
        self.assertEqual(shell.events.callbacks["post_run_cell"], [])
        self.assert_released()
        before = len(self.scene.camera.captures)
        shell.events.retained[0]()
        self.assertEqual(len(self.scene.camera.captures), before)
    def test_checkpoint_paste_executes_cells_instead_of_returning_snapshot_bytes(self):
        source = iter(("# edit\nbox.x += 2", "# later\nbox.x += 8", "# edit\nbox.x += 3"))
        self.embed.clipboard = lambda: next(source)
        def body(shell):
            for expected in (2, 10, 3):
                self.scene.checkpoint_paste()
                self.assertEqual(self.box.x, expected)
            self.assertEqual(list(self.embed.checkpoint_manager.checkpoint_states), ["# edit"])
            self.assertEqual(len(self.scene.camera.captures), 3)
        shell = self.shell(body)
        self.embed.launch()
        self.assertEqual(len(shell.calls), 3)
        self.assertEqual(self.embed.checkpoint_manager.checkpoint_states, {})
        self.assert_released()
    def test_manager_paste_delegates_to_its_active_console_and_shell(self):
        self.embed.clipboard = lambda: "# key\nbox.x += 7"
        def body(shell):
            self.embed.checkpoint_manager.checkpoint_paste(shell, self.scene)
            self.assertEqual(self.box.x, 7)
            with self.assertRaisesRegex(ValueError, "shell does not own"):
                self.embed.checkpoint_manager.checkpoint_paste(Shell(), self.scene)
            with self.assertRaises(CapabilityError):
                self.native.CheckpointManager().checkpoint_paste(shell, self.scene)
        self.shell(body)
        self.embed.launch()
        self.assert_released()
    def test_paste_flags_reach_scene_configuration_and_restore(self):
        self.embed.clipboard = lambda: "seen = scene.config"
        def body(shell):
            self.embed.checkpoint_paste(skip=True, progress_bar=False)
            self.assertEqual(shell.user_ns["seen"], (True, False, False))
            self.assertEqual(self.scene.config, (False, False, False))
            with self.assertRaisesRegex(CapabilityError, "recording"):
                self.embed.checkpoint_paste(record=True)
            self.assertEqual(len(shell.calls), 1)
        self.shell(body)
        self.embed.launch()
    def test_missing_clipboard_never_discovers_pyperclip_or_a_subprocess(self):
        def body(shell):
            with patch.object(adapter.importlib, "import_module", side_effect=AssertionError("unexpected import")):
                with self.assertRaisesRegex(CapabilityError, "host clipboard"):
                    self.embed.checkpoint_paste()
            self.assertFalse(self.embed.checkpoint_manager.checkpoint_states)
        self.shell(body)
        self.embed.launch()
    def test_paste_outside_session_refuses_before_reading_or_mutating(self):
        for operation in (self.embed.checkpoint_paste, self.scene.checkpoint_paste,
                          lambda: self.embed.checkpoint_manager.checkpoint_paste(None, self.scene)):
            with self.assertRaisesRegex(CapabilityError, "active"):
                operation()
        self.assertEqual(self.scene.trace, [])
    def test_interactive_paste_uses_a_host_scene_console_without_ipython(self):
        with SceneConsole(self.scene, {"box": self.box}, clipboard=lambda: "# x\nbox.x += 6", _native=self.native):
            self.scene.checkpoint_paste()
            self.scene.checkpoint_paste()
            self.assertEqual(self.box.x, 6)
        self.assert_released()
    def test_authored_interrupt_preserved_despite_hook_cleanup_failure(self):
        primary = KeyboardInterrupt("authored stop")
        def body(shell):
            shell.events.error = ValueError("cleanup failed")
            raise primary
        shell = self.shell(body)
        with self.assertRaises(KeyboardInterrupt) as caught:
            self.embed.launch()
        self.assertIs(caught.exception, primary)
        self.assertIn("ValueError", primary.__notes__[0])
        self.assert_released()
        shell.events.retained[0]()
        self.assertEqual(self.scene.trace, [])
    def test_cleanup_failure_is_visible_when_there_is_no_primary_error(self):
        def body(shell):
            shell.events.error = RuntimeError("cannot unregister")
        self.shell(body)
        with self.assertRaisesRegex(RuntimeError, "cannot unregister"):
            self.embed.launch()
        self.assert_released()
    def test_cell_exception_preserves_identity_and_preview_then_retry(self):
        error = ValueError("authored")
        self.embed.clipboard = lambda: "# x\nbox.x = 9\nraise error"
        def body(shell):
            shell.user_ns["error"] = error
            with self.assertRaises(ValueError) as caught:
                self.embed.checkpoint_paste()
            self.assertIs(caught.exception, error)
            self.assertEqual(self.box.x, 9)
            self.embed.clipboard = lambda: "# x\nbox.x += 2"
            # The SceneConsole freezes the explicitly granted callback, so
            # changing the reader's implementation does not replace ownership.
            self.embed._fmn_console.clipboard = self.embed.clipboard
            self.embed.checkpoint_paste()
            self.assertEqual(self.box.x, 2)
        self.shell(body)
        self.embed.launch()
    def test_nested_shells_are_refused_even_for_another_scene(self):
        def body(shell):
            with self.assertRaisesRegex(RuntimeError, "nest"):
                self.embed.launch()
            other = self.native.InteractiveSceneEmbed(self.native.Scene())
            other.shell = Shell(lambda s: self.fail("nested terminal entered"))
            with self.assertRaisesRegex(RuntimeError, "nest"):
                other.launch()
        self.shell(body)
        self.embed.launch()
        self.assert_released()
    def test_cross_thread_calls_refuse_before_shell_or_scene_access(self):
        self.shell(lambda s: self.fail("wrong-thread shell"))
        errors = []
        def worker():
            for call in (self.embed.launch, self.embed.get_ipython_shell_for_embedded_scene,
                         self.embed.checkpoint_paste, self.embed.checkpoint_manager.clear_checkpoints):
                try:
                    call()
                except BaseException as error:
                    errors.append(error)
        thread = threading.Thread(target=worker)
        thread.start(); thread.join()
        self.assertEqual(len(errors), 4)
        self.assertTrue(all("thread" in str(error) for error in errors))
        self.assertEqual(self.scene.trace, [])
    def test_active_output_and_play_ownership_are_not_stolen(self):
        self.shell(lambda s: self.fail("terminal entered"))
        for key in ("_fmn_owned_render_session", "_fmn_scene_execution"):
            marker = object()
            vars(self.scene)[key] = marker
            try:
                with self.assertRaises(RuntimeError):
                    self.embed.launch()
                self.assertIs(vars(self.scene)[key], marker)
                self.assert_released()
            finally:
                vars(self.scene).pop(key)
    def test_another_console_is_not_closed_by_a_failed_shell_launch(self):
        self.shell(lambda s: self.fail("terminal entered"))
        with SceneConsole(self.scene, _native=self.native) as owner:
            with self.assertRaisesRegex(RuntimeError, "active console"):
                self.embed.launch()
            self.assertIs(vars(self.scene)[_OWNER], owner)
            owner.run_cell("scene.time = 2")
        self.assertEqual(self.scene.time, 2)
    def test_studio_stdio_refuses_before_optional_import_or_terminal(self):
        token = _STUDIO_WORKER.set(True)
        try:
            with patch.object(adapter.importlib, "import_module", side_effect=AssertionError("unexpected import")):
                for call in (self.embed.launch, self.embed.get_ipython_shell_for_embedded_scene):
                    with self.assertRaisesRegex(CapabilityError, "native protocol"):
                        call()
                with self.assertRaisesRegex(RuntimeError, "native protocol"):
                    with SceneConsole(self.scene, _native=self.native):
                        self.fail("worker console acquired")
        finally:
            _STUDIO_WORKER.reset(token)
        self.assert_released()
    def test_manager_cannot_restore_another_scene_and_failure_preserves_history(self):
        manager = self.embed.checkpoint_manager
        manager.handle_checkpoint_key(self.scene, "# a")
        with self.assertRaisesRegex(ValueError, "another Scene"):
            manager.handle_checkpoint_key(self.native.Scene(), "# a")
        manager.handle_checkpoint_key(self.scene, "# b")
        self.scene.restore_error = RuntimeError("restore refused")
        with self.assertRaisesRegex(RuntimeError, "restore refused"):
            manager.handle_checkpoint_key(self.scene, "# a")
        self.assertEqual(list(manager.checkpoint_states), ["# a", "# b"])
        manager.clear_checkpoints()
        manager.handle_checkpoint_key(self.native.Scene(), "# a")
    def test_manager_key_limit_refuses_before_native_snapshot(self):
        for key in ([], "#" * 4097):
            with self.assertRaises(ValueError):
                self.embed.checkpoint_manager.handle_checkpoint_key(self.scene, key)
        self.assertEqual(self.scene.trace, [])
    def test_namespace_replacement_by_reentrant_ipython_is_not_overwritten(self):
        def body(shell):
            owner = self.embed._fmn_console
            shell.user_ns = dict(shell.user_ns, current=12)
            shell.user_ns.pop("box")
            owner.run_cell("current += 1")
            self.assertEqual(owner.namespace["current"], 13)
            self.assertNotIn("box", owner.namespace)
        self.shell(body)
        self.embed.launch()
    def test_public_embed_helper_threads_explicit_namespace_and_clipboard(self):
        from fmn_python import embed_scene
        requested = []
        def launch(instance):
            requested.append(instance)
            return "actual-shell-handle"
        namespace = {"box": self.box}
        reader = lambda: "# a\nbox.x += 1"
        with patch.dict(sys.modules, {"manimlib": self.native}), \
                patch.object(self.native.InteractiveSceneEmbed, "launch", launch):
            self.assertEqual(embed_scene(self.scene, namespace, clipboard=reader), "actual-shell-handle")
        self.assertIs(requested[0].clipboard, reader)
        self.assertEqual(requested[0]._fmn_namespace, namespace)
        self.assertIsNot(requested[0]._fmn_namespace, namespace)
    def test_interactive_embed_captures_authored_locals_and_preserves_override(self):
        captured = []
        def launch(instance):
            captured.append(instance._fmn_namespace)
            return "shell"
        authored_value = object()
        with patch.object(self.native.InteractiveSceneEmbed, "launch", launch):
            self.assertEqual(self.scene.embed(), "shell")
        self.assertIs(captured[0]["authored_value"], authored_value)
        self.assertIs(captured[0]["self"], self)
        class Custom(self.native.InteractiveScene):
            def embed(self, namespace=None):
                return "authored-override"
        self.assertEqual(Custom().embed(), "authored-override")
    def test_launch_refusal_precedes_ipython_import_and_retains_manual_checkpoints(self):
        manager = self.embed.checkpoint_manager
        manager.handle_checkpoint_key(self.scene, "# manual")
        state = manager.checkpoint_states["# manual"]
        vars(self.scene)["_fmn_owned_render_session"] = object()
        with patch.object(adapter.importlib, "import_module", side_effect=AssertionError("imported IPython")):
            with self.assertRaisesRegex(RuntimeError, "output generation"):
                self.embed.launch()
        self.assertIs(manager.checkpoint_states["# manual"], state)
        self.assertIsNone(self.embed.shell)
        self.assert_released()
    def test_shell_creation_failure_preserves_manual_checkpoint_history(self):
        manager = self.embed.checkpoint_manager
        manager.handle_checkpoint_key(self.scene, "# manual")
        with patch.object(adapter.importlib, "import_module", side_effect=ImportError("missing")):
            with self.assertRaises(CapabilityError):
                self.embed.launch()
        self.assertEqual(list(manager.checkpoint_states), ["# manual"])
        self.assert_released()
    def test_explicit_shell_namespace_aliases_keep_authored_identity(self):
        marker = object()
        shell = self.shell(lambda shell: self.assertIs(shell.user_ns["self"], marker))
        shell.user_ns["self"] = marker
        self.embed.launch()
        self.assertIs(shell.user_ns["self"], marker)
    def test_hook_requires_shell_and_non_scene_constructor_refuses(self):
        with self.assertRaisesRegex(CapabilityError, "create an IPython shell"):
            self.embed.ensure_frame_update_post_cell()
        with self.assertRaisesRegex(TypeError, "scene must be a Scene"):
            self.native.InteractiveSceneEmbed(object())
    def test_missing_ipython_offers_non_ipython_cell_route(self):
        with patch.object(adapter.importlib, "import_module", side_effect=ImportError("missing")):
            with self.assertRaisesRegex(CapabilityError, "SceneConsole.run_cell"):
                self.embed.launch()
        self.assert_released()


if __name__ == "__main__":
    unittest.main()
