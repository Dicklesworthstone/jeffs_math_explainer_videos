"""Checkpoint orchestration with explicit native Scene/Camera boundary doubles.

These are not render tests. scene_console_acceptance.py exercises native
SceneState, object identities, callbacks, clock, camera and decoded pixels.
"""
from contextlib import contextmanager
import copy
from pathlib import Path
import sys
import threading
from types import SimpleNamespace
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python import SceneConsole


class CapabilityError(RuntimeError):
    pass


class Manager:
    def __init__(self):
        self.checkpoint_states = {}
    def clear_checkpoints(self):
        self.checkpoint_states.clear()


class Camera:
    def __init__(self):
        self.captures = []
        self.error = None
    def capture(self, *roots):
        if self.error is not None:
            raise self.error
        self.captures.append(roots)


class Scene:
    def __init__(self):
        self.mobjects = [SimpleNamespace(x=0.0)]
        self.time, self.num_plays, self.rng = 0, 0, 17
        self.camera = Camera()
        self.config = (False, False, False)
        self.trace = []
        self.snapshot_error = self.restore_error = self.update_error = None
    def get_state(self):
        self.trace.append("snapshot")
        if self.snapshot_error:
            raise self.snapshot_error
        return self, self.time, self.num_plays, self.rng, tuple((m, m.x) for m in self.mobjects)
    def restore_state(self, state):
        self.trace.append("restore")
        if self.restore_error:
            raise self.restore_error
        owner, self.time, self.num_plays, self.rng, values = state
        assert owner is self
        self.mobjects[:] = [m for m, _ in values]
        for m, x in values:
            m.x = x
    @contextmanager
    def temp_config_change(self, skip=False, record=False, progress_bar=False):
        previous = self.config
        if record:
            raise CapabilityError("native insert recording is unavailable")
        self.config = (skip, record, progress_bar)
        try:
            yield
        finally:
            self.config = previous
    def is_window_closing(self):
        return False
    def update_frame(self, dt=0, force_draw=False):
        self.trace.append(("refresh", dt, force_draw))
        if self.update_error:
            raise self.update_error


NATIVE = SimpleNamespace(Scene=Scene, CheckpointManager=Manager, _CapabilityError=CapabilityError)


class SceneConsoleTests(unittest.TestCase):
    def setUp(self):
        self.scene = Scene()
        self.console = SceneConsole(self.scene, {"box": self.scene.mobjects[0]}, _native=NATIVE)
        self.addCleanup(self.console.close)
    def test_keyed_retry_restores_native_scene_clock_rng_and_identity(self):
        box = self.scene.mobjects[0]
        self.console.run_cell("# placement\nbox.x += 2\nscene.time += 3\nscene.num_plays += 1\nscene.rng += 4")
        self.assertEqual((box.x, self.scene.time, self.scene.num_plays, self.scene.rng), (2, 3, 1, 21))
        self.console.run_cell("# later\nbox.x += 100")
        self.console.run_cell("# placement\nbox.x += 7")
        self.assertIs(self.scene.mobjects[0], box)
        self.assertEqual((box.x, self.scene.time, self.scene.num_plays, self.scene.rng), (7, 0, 0, 17))
        self.assertEqual(list(self.console.checkpoint_manager.checkpoint_states), ["# placement"])
    def test_plain_cells_execute_without_a_checkpoint(self):
        self.console.run_cell("box.x += 2")
        self.console.run_cell("box.x += 3")
        self.assertEqual(self.scene.mobjects[0].x, 5)
        self.assertEqual(self.console.checkpoint_manager.checkpoint_states, {})
    def test_dedent_and_first_comment_follow_reference(self):
        self.console.run_cell("    # a  \n    box.x += 1   \n")
        self.console.run_cell("    # a\n    box.x += 2\n")
        self.assertEqual(self.scene.mobjects[0].x, 2)
        self.console.run_cell("\n# a\nbox.x += 3")
        self.assertEqual(self.scene.mobjects[0].x, 5)
    def test_syntax_error_does_not_restore_or_change_history(self):
        self.console.run_cell("# a\nbox.x = 1")
        self.console.run_cell("# b\nbox.x = 2")
        history = dict(self.console.checkpoint_manager.checkpoint_states)
        trace = list(self.scene.trace)
        with self.assertRaises(SyntaxError):
            self.console.run_cell("# a\nbroken(")
        self.assertEqual(self.scene.mobjects[0].x, 2)
        self.assertEqual(self.console.checkpoint_manager.checkpoint_states, history)
        self.assertEqual(self.scene.trace, trace)
    def test_runtime_error_keeps_partial_effects_but_retry_can_restore(self):
        sentinel = RuntimeError("authored failure")
        self.console.namespace["sentinel"] = sentinel
        with self.assertRaises(RuntimeError) as caught:
            self.console.run_cell("# retry\nbox.x = 8\nraise sentinel")
        self.assertIs(caught.exception, sentinel)
        self.assertEqual(self.scene.mobjects[0].x, 8)
        self.console.run_cell("# retry\nbox.x += 1")
        self.assertEqual(self.scene.mobjects[0].x, 1)
    def test_keyboard_interrupt_keeps_identity_releases_busy_and_restores_options(self):
        sentinel = KeyboardInterrupt("stop")
        self.console.namespace["sentinel"] = sentinel
        with self.assertRaises(KeyboardInterrupt) as caught:
            self.console.run_cell("# retry\nbox.x = 8\nraise sentinel", skip=True)
        self.assertIs(caught.exception, sentinel)
        self.assertEqual(self.scene.config, (False, False, False))
        self.console.run_cell("# retry\nbox.x += 2")
        self.assertEqual(self.scene.mobjects[0].x, 2)
    def test_native_preview_refresh_does_not_add_clock_time(self):
        self.console.run_cell("box.x = 2")
        self.assertEqual(self.scene.time, 0)
        self.assertIn(("refresh", 0, True), self.scene.trace)
        self.assertEqual(self.scene.camera.captures, [tuple(self.scene.mobjects)])
    def test_capture_can_be_explicitly_disabled(self):
        self.console.close()
        with SceneConsole(self.scene, capture=False, _native=NATIVE) as console:
            console.run_cell("scene.time = 5")
        self.assertEqual(self.scene.time, 5)
        self.assertEqual(self.scene.camera.captures, [])
    def test_refresh_error_does_not_replace_authored_error(self):
        primary = RuntimeError("primary")
        self.console.namespace["primary"] = primary
        self.scene.update_error = ValueError("secondary")
        with self.assertRaises(RuntimeError) as caught:
            self.console.run_cell("raise primary")
        self.assertIs(caught.exception, primary)
        self.assertIn("ValueError", primary.__notes__[0])
    def test_successful_cell_does_not_hide_failed_preview(self):
        self.scene.camera.error = ValueError("capture failed")
        with self.assertRaisesRegex(ValueError, "capture failed"):
            self.console.run_cell("box.x = 4")
        self.assertEqual(self.scene.mobjects[0].x, 4)
    def test_preview_is_atomic_at_native_capture_boundary(self):
        self.console.run_cell("box.x = 1")
        before = list(self.scene.camera.captures)
        self.scene.camera.error = RuntimeError("capture failed")
        with self.assertRaises(RuntimeError):
            self.console.run_cell("box.x = 2")
        self.assertEqual(self.scene.camera.captures, before)
    def test_snapshot_failure_executes_no_cell(self):
        self.scene.snapshot_error = RuntimeError("views pinned")
        with self.assertRaisesRegex(RuntimeError, "views pinned"):
            self.console.run_cell("# a\nbox.x = 2")
        self.assertEqual(self.scene.mobjects[0].x, 0)
        self.assertEqual(self.console.checkpoint_manager.checkpoint_states, {})
    def test_restore_failure_does_not_invalidate_later_checkpoints(self):
        self.console.run_cell("# a\nbox.x = 1")
        self.console.run_cell("# b\nbox.x = 2")
        self.scene.restore_error = RuntimeError("cannot rewind")
        with self.assertRaisesRegex(RuntimeError, "cannot rewind"):
            self.console.run_cell("# a\nbox.x = 3")
        self.assertEqual(list(self.console.checkpoint_manager.checkpoint_states), ["# a", "# b"])
        self.assertEqual(self.scene.mobjects[0].x, 2)
    def test_budget_never_evicts_an_earlier_checkpoint(self):
        self.console.max_checkpoints = 1
        self.console.run_cell("# a\nbox.x = 1")
        with self.assertRaisesRegex(ValueError, "budget"):
            self.console.run_cell("# b\nbox.x = 2")
        self.assertEqual(self.scene.mobjects[0].x, 1)
        self.console.run_cell("# a\nbox.x += 3")
        self.assertEqual(self.scene.mobjects[0].x, 3)
        self.console.clear_checkpoints()
        self.console.run_cell("# b\nbox.x += 1")
        self.assertEqual(self.scene.mobjects[0].x, 4)
    def test_source_budget_includes_dedented_whitespace_and_utf8(self):
        self.console.max_source_bytes = 8
        for source in (" " * 9, "#" + "é" * 4):
            with self.subTest(source=source), self.assertRaisesRegex(ValueError, "budget"):
                self.console.run_cell(source)
        self.assertEqual(self.scene.trace, [])
    def test_bad_source_is_rejected_before_snapshot_or_effects(self):
        for source, error in ((b"pass", TypeError), ("\0", ValueError), ("\ud800", ValueError)):
            with self.subTest(source=source), self.assertRaises(error):
                self.console.run_cell(source)
        self.assertEqual(self.scene.trace, [])
    def test_config_flags_are_honored_and_restored(self):
        self.console.run_cell("seen = scene.config", skip=True, progress_bar=False)
        self.assertEqual(self.console.namespace["seen"], (True, False, False))
        self.assertEqual(self.scene.config, (False, False, False))
    def test_record_refuses_before_restore_or_cell(self):
        self.console.run_cell("# a\nbox.x = 3")
        before = list(self.scene.trace)
        with self.assertRaises(CapabilityError):
            self.console.run_cell("# a\nbox.x = 7", record=True)
        self.assertEqual(self.scene.trace, before)
        self.assertEqual(self.scene.mobjects[0].x, 3)
    def test_nonboolean_flags_fail_before_execution(self):
        for name in ("skip", "record", "progress_bar"):
            with self.subTest(name=name), self.assertRaises(TypeError):
                self.console.run_cell("# a\nbox.x = 3", **{name: 1})
        self.assertEqual(self.scene.trace, [])
    def test_nested_cell_execution_is_rejected_before_effects(self):
        self.console.namespace["console"] = self.console
        with self.assertRaisesRegex(RuntimeError, "reenter"):
            self.console.run_cell("console.run_cell('box.x = 99')")
        self.assertEqual(self.scene.mobjects[0].x, 0)
        self.console.run_cell("box.x = 1")
    def test_multiple_consoles_do_not_share_scene_ownership(self):
        with self.console:
            other = SceneConsole(self.scene, _native=NATIVE)
            with self.assertRaisesRegex(RuntimeError, "active console"):
                other.run_cell("scene.time = 99")
            other.close()
            self.console.run_cell("box.x = 2")
        self.assertEqual(self.scene.time, 0)
    def test_close_during_cell_is_rejected_and_close_is_idempotent(self):
        self.console.namespace["console"] = self.console
        with self.assertRaisesRegex(RuntimeError, "reenter"):
            self.console.run_cell("console.close()")
        self.console.close()
        self.console.close()
        with self.assertRaisesRegex(RuntimeError, "closed"):
            self.console.run_cell("pass")
    def test_wrong_thread_does_not_touch_native_scene(self):
        failures = []
        def other_thread():
            for action in (lambda: self.console.run_cell("scene.time = 99"), self.console.close):
                try:
                    action()
                except RuntimeError as error:
                    failures.append(str(error))
        thread = threading.Thread(target=other_thread)
        thread.start(); thread.join()
        self.assertEqual(len(failures), 2)
        self.assertTrue(all("creating thread" in value for value in failures))
        self.assertEqual(self.scene.trace, [])
    def test_active_play_output_and_worker_protocol_are_not_reentered(self):
        for field in ("_fmn_scene_execution", "_fmn_owned_render_session", "_fmn_studio_worker_request"):
            with self.subTest(field=field):
                setattr(self.scene, field, object())
                try:
                    with self.assertRaises(RuntimeError):
                        self.console.run_cell("scene.time = 99")
                finally:
                    delattr(self.scene, field)
        self.assertEqual(self.scene.trace, [])
    def test_clipboard_is_host_owned_and_called_exactly_once(self):
        calls = []
        self.console.clipboard = lambda: calls.append("read") or "# clipboard\nbox.x += 3"
        self.console.checkpoint_paste()
        self.console.checkpoint_paste()
        self.assertEqual(calls, ["read", "read"])
        self.assertEqual(self.scene.mobjects[0].x, 3)
    def test_clipboard_refusal_and_failure_do_not_create_history(self):
        with self.assertRaisesRegex(CapabilityError, "host clipboard"):
            self.console.checkpoint_paste()
        sentinel = OSError("clipboard unavailable")
        def read():
            raise sentinel
        self.console.clipboard = read
        with self.assertRaises(OSError) as caught:
            self.console.checkpoint_paste()
        self.assertIs(caught.exception, sentinel)
        self.assertEqual(self.scene.trace, [])
    def test_clipboard_callback_cannot_recursively_execute_a_cell(self):
        self.console.clipboard = lambda: self.console.run_cell("scene.time = 99")
        with self.assertRaisesRegex(RuntimeError, "reenter"):
            self.console.checkpoint_paste()
        self.assertEqual(self.scene.time, 0)
    def test_namespace_is_not_a_fake_rollback_of_python_effects(self):
        original = {"effects": []}
        self.console.close()
        with SceneConsole(self.scene, original, _native=NATIVE) as console:
            console.run_cell("# a\neffects.append(1)\nvariable = 3")
            console.run_cell("# a\neffects.append(2)")
            self.assertEqual(console.namespace["variable"], 3)
        self.assertEqual(original, {"effects": [1, 2]})
    def test_close_releases_checkpoint_and_scene_owners(self):
        self.console.run_cell("# a\npass")
        self.console.close()
        self.assertNotIn("_fmn_scene_console", vars(self.scene))
        self.assertEqual(self.console.checkpoint_manager.checkpoint_states, {})
        self.assertIsNone(self.console.scene)
    def test_constructor_has_no_scene_or_clipboard_effects(self):
        self.assertEqual(self.scene.trace, [])
        self.assertNotIn("_fmn_scene_console", vars(self.scene))
    def test_bad_configuration_is_rejected_without_scene_mutation(self):
        for options in ({"max_checkpoints": 0}, {"max_checkpoints": True},
                        {"max_source_bytes": -1}, {"capture": 1}, {"clipboard": "x"},
                        {"shell": object()}):
            with self.subTest(options=options), self.assertRaises((TypeError, ValueError)):
                SceneConsole(self.scene, _native=NATIVE, **options)
        self.assertEqual(self.scene.trace, [])
    def test_shell_reports_errors_without_losing_original_exception(self):
        sentinel = ValueError("from IPython")
        self.console.shell = SimpleNamespace(run_cell=lambda _: SimpleNamespace(error_in_exec=sentinel))
        with self.assertRaises(ValueError) as caught:
            self.console.run_cell("# shell\npass")
        self.assertIs(caught.exception, sentinel)
    def test_shell_transform_is_validated_before_checkpoint(self):
        calls = []
        self.console.shell = SimpleNamespace(transform_cell=lambda _: "invalid(", run_cell=lambda _: calls.append(1))
        with self.assertRaises(SyntaxError):
            self.console.run_cell("# shell\n%time pass")
        self.assertEqual(calls, [])
        self.assertEqual(self.scene.trace, [])
    def test_async_shell_protocol_is_not_misreported_as_success(self):
        async def run(_):
            raise AssertionError("must not drive another event loop")
        self.console.shell = SimpleNamespace(run_cell=run)
        with self.assertRaisesRegex(TypeError, "synchronous"):
            self.console.run_cell("pass")


if __name__ == "__main__":
    unittest.main()
