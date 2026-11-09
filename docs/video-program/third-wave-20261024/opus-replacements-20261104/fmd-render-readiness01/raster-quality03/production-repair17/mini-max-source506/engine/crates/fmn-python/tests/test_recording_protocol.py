"""Recording lifecycle over the real adapter and a native-publication double.

Marker files prove orchestration only; recording_sessions.py uses native pixels.
"""
from contextlib import contextmanager
import hashlib
import os
from pathlib import Path
import sys
import tempfile
import threading
import types
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python.recording import RecordingSession, record_scene
from fmn_python.scene_console import SceneConsole
from fmn_python import record_scene as public_record_scene


class Camera:
    def __init__(self):
        self.fps = 60  # Deliberately different from the authoritative live clock.
        self.shape = (96, 54)
        self._core = self
    def get_pixel_shape(self):
        return self.shape
    def set_pixel_shape(self, *shape):
        raise AssertionError("recording must not mutate the live camera")


class Scene:
    def __init__(self):
        self.camera, self.random_seed = Camera(), 17
        self.fps, self.frame = 30, 90
        self.mobjects = [object()]
        self.events = []
        self.active = False
        self.start_error = self.finish_error = self.abort_error = None
        self._render_invocations = [{"argv": ["native-encoder"]}]
        self._render_audio_inputs = [{"decoder": "native-wav"}]
    def _begin_native_output(self, *args):
        raise AssertionError("must not replace the populated Scene")
    def run(self):
        raise AssertionError("must not reexecute authored lifecycle")
    def _finish_render(self):
        self.events.append("finish")
        if self.finish_error:
            raise self.finish_error
        path, format, width, height, fps, threads = self.request
        data = b"native-boundary-marker"
        destination = Path(path)
        destination.parent.mkdir(parents=True, exist_ok=True)
        if format == "png_sequence":
            destination.mkdir()
            (destination / "frame-marker").write_bytes(data)
        else:
            with destination.open("xb") as output:
                output.write(data)
        self.active = False
        return path, max(1, self.frame - self.start), len(data), hashlib.sha256(data).hexdigest(), "native-double", threads
    def _abort_render(self):
        self.events.append("abort")
        self.active = False
        if self.abort_error:
            raise self.abort_error
    def get_state(self):
        return self.frame
    def restore_state(self, state):
        self.events.append("restore")
        self.frame = state
    @contextmanager
    def temp_config_change(self, skip, record, progress_bar):
        if record:
            raise RuntimeError("recording needs an explicit destination")
        yield


class CheckpointManager:
    def __init__(self):
        self.checkpoint_states = {}
    def clear_checkpoints(self):
        self.checkpoint_states.clear()


def begin(scene, *request):
    scene.events.append("begin")
    if scene.active:
        raise RuntimeError("external generation active")
    if scene.start_error:
        raise scene.start_error
    if request[-2] != scene.fps:
        raise ValueError("live Scene clock changed")
    scene.active, scene.request, scene.start = True, request, scene.frame
    return scene.frame


native = types.SimpleNamespace(Scene=Scene, CheckpointManager=CheckpointManager,
    _portal_scene_clock=lambda scene: (scene.fps, scene.frame), _portal_begin_recording=begin)


class RecordingTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.scene = Scene()
        imports = patch.dict(sys.modules, {"manimlib": native})
        imports.start()
        self.addCleanup(imports.stop)
    def session(self, filename="clip.y4m", **options):
        options.setdefault("threads", 1)
        return record_scene(self.scene, self.root / filename, **options)
    def test_live_scene_not_reconstructed_and_native_clock_wins(self):
        mob, camera = self.scene.mobjects[0], self.scene.camera
        session = self.session()
        self.assertEqual(self.scene.events, [])
        with session:
            self.assertTrue(self.scene.active)
            self.assertEqual(session.start_frame, 90)
            self.scene.frame += 3
        self.assertIs(self.scene.camera, camera)
        self.assertEqual(camera.fps, 60)
        self.assertIs(self.scene.mobjects[0], mob)
        self.assertEqual((session.start_frame, session.end_frame), (90, 93))
        self.assertEqual(session.result.fps, 30)
        self.assertFalse(session.result.certified)
        self.assertEqual(session.result.digest, hashlib.sha256(session.destination.read_bytes()).hexdigest())
        self.assertIs(session.finish(), session.result)
        self.assertEqual(self.scene.events, ["begin", "finish"])
        self.assertNotIn("_fmn_owned_render_session", vars(self.scene))
    def test_repeated_independent_clips(self):
        for index in range(3):
            with self.session(f"clip-{index}.y4m") as session:
                self.scene.frame += 3
            self.assertEqual(session.start_frame, 90 + index * 3)
        self.assertEqual(self.scene.frame, 99)
    def test_all_temporal_formats_share_receipt_and_finish(self):
        for format in ("png_sequence", "gif", "y4m", "wav", "mp4", "mov"):
            with self.subTest(format=format):
                with self.session(format, format=format) as session:
                    self.scene.frame += 3
                self.assertTrue(session.destination.exists())
                self.assertEqual(session.result.format, format)
                if format in ("wav", "mp4", "mov"):
                    self.assertEqual(session.result.audio_inputs[0]["decoder"], "native-wav")
    def test_bad_inputs_refuse_before_native_start(self):
        for filename, options in (("a.png", {}), ("a.svg", {}), ("a.bad", {}),
                                  ("a.y4m", {"fps": 8}), ("a.y4m", {"fps": True}),
                                  ("a.y4m", {"threads": 0}), ("a.y4m", {"resolution": (0, 2)})):
            with self.subTest(options=options, filename=filename):
                with self.assertRaises((TypeError, ValueError)):
                    self.session(filename, **options)
        self.assertEqual(self.scene.events, [])
    def test_text_path_validation(self):
        for path in (b"x.y4m", "", "x\0.y4m"):
            with self.assertRaises((TypeError, ValueError)):
                record_scene(self.scene, path)
    def test_scene_instance_required(self):
        with self.assertRaises(TypeError):
            record_scene(Scene, self.root / "x.y4m")
    def test_missing_native_capability_is_explicit(self):
        with self.assertRaisesRegex(RuntimeError, "matching wheel"):
            RecordingSession(self.scene, self.root / "x.y4m", _native=types.SimpleNamespace(Scene=Scene))
    def test_relative_path_frozen_before_execution(self):
        before = os.getcwd()
        self.addCleanup(os.chdir, before)
        os.chdir(self.root)
        session = record_scene(self.scene, "relative.y4m", threads=1)
        os.chdir(before)
        with session:
            self.scene.frame += 1
        self.assertEqual(session.result.destination, self.root / "relative.y4m")
    def test_start_refusal_does_not_abort_external_owner(self):
        self.scene.active = True
        with self.assertRaisesRegex(RuntimeError, "external"):
            with self.session():
                pass
        self.assertTrue(self.scene.active)
        self.assertNotIn("abort", self.scene.events)
    def test_nested_context_preserves_first_owner(self):
        with self.session() as first:
            with self.assertRaisesRegex(RuntimeError, "already has"):
                with self.session("nested.y4m"):
                    pass
            self.assertIs(vars(self.scene)["_fmn_owned_render_session"], first)
    def test_context_enters_only_once(self):
        session = self.session()
        with session:
            pass
        with self.assertRaisesRegex(RuntimeError, "only once"):
            session.__enter__()
    def test_unentered_session_cannot_finish(self):
        with self.assertRaisesRegex(RuntimeError, "active"):
            self.session().finish()
    def test_clock_mismatch_after_construction_refuses_without_abort(self):
        session = self.session()
        self.scene.fps = 8
        with self.assertRaisesRegex(ValueError, "clock changed"):
            session.__enter__()
        self.assertNotIn("abort", self.scene.events)
    def test_interrupt_cancels_output_but_keeps_scene_effects(self):
        session = self.session()
        interrupted = KeyboardInterrupt("stop recording")
        with self.assertRaises(KeyboardInterrupt) as caught:
            with session:
                self.scene.frame += 3
                raise interrupted
        self.assertIs(caught.exception, interrupted)
        self.assertFalse(session.destination.exists())
        self.assertIsNone(session.result)
        self.assertEqual(self.scene.frame, 93)
        self.assertEqual(self.scene.events, ["begin", "abort"])
    def test_abort_failure_does_not_replace_original_exception(self):
        self.scene.abort_error = RuntimeError("abort failure")
        original = ValueError("authored failure")
        with self.assertRaises(ValueError) as caught:
            with self.session():
                raise original
        self.assertIs(caught.exception, original)
        self.assertIn("abort failure", original.__notes__[0])
        self.assertNotIn("_fmn_owned_render_session", vars(self.scene))
    def test_finish_failure_cancels_generation(self):
        self.scene.finish_error = OSError("disk full")
        with self.assertRaises(OSError):
            with self.session():
                self.scene.frame += 3
        self.assertEqual(self.scene.events, ["begin", "finish", "abort"])
    def test_explicit_abort_is_idempotent(self):
        with self.session() as session:
            session.abort()
            session.abort()
        self.assertEqual(self.scene.events, ["begin", "abort"])
        self.assertIsNone(session.result)
    def test_existing_destination_preserved(self):
        path = self.root / "clip.y4m"
        path.write_bytes(b"preserve")
        with self.assertRaises(FileExistsError):
            with self.session():
                self.scene.frame += 3
        self.assertEqual(path.read_bytes(), b"preserve")
    def test_frame_reset_aborts_instead_of_publishing(self):
        with self.assertRaisesRegex(RuntimeError, "clock was reset"):
            with self.session():
                self.scene.frame = 0
        self.assertEqual(self.scene.events, ["begin", "abort"])
    def test_active_animation_cannot_begin_recording(self):
        self.scene._fmn_scene_execution = object()
        with self.assertRaisesRegex(RuntimeError, "between"):
            self.session().__enter__()
        self.assertEqual(self.scene.events, [])
    def test_active_animation_cannot_finish_recording(self):
        with self.assertRaisesRegex(RuntimeError, "between"):
            with self.session():
                self.scene._fmn_scene_execution = object()
        self.assertEqual(self.scene.events, ["begin", "abort"])
    def test_owner_thread_checked_before_native_proxy(self):
        session = self.session()
        errors = []
        def use():
            for action in (session.__enter__, session.finish, session.abort):
                try:
                    action()
                except RuntimeError as error:
                    errors.append(error)
        worker = threading.Thread(target=use)
        worker.start()
        worker.join()
        self.assertEqual(len(errors), 3)
        self.assertEqual(self.scene.events, [])

    def test_public_export(self):
        self.assertIs(public_record_scene, record_scene)

    def console(self, **options):
        console = SceneConsole(self.scene, capture=False, _native=native, **options)
        self.addCleanup(console.close)
        return console

    def test_checkpoint_retry_restores_before_recording_opens(self):
        console = self.console()
        for index in range(2):
            console.run_cell("# clip\nscene.frame += 3", record_to=self.root / f"cell-{index}.y4m",
                             recording_options={"threads": 1})
            self.assertEqual(self.scene.frame, 93)
            self.assertEqual(console.last_recording.frame_count, 3)
        self.assertEqual(self.scene.events, ["begin", "finish", "restore", "begin", "finish"])
        self.assertEqual(len(console.checkpoint_manager.checkpoint_states), 1)

    def test_recorded_cell_error_cancels_without_losing_last_success(self):
        console = self.console()
        console.run_cell("# success\nscene.frame += 3", record_to=self.root / "good.y4m")
        receipt = console.last_recording
        with self.assertRaisesRegex(ValueError, "authored"):
            console.run_cell("# bad\nscene.frame += 2\nraise ValueError('authored')",
                             record_to=self.root / "bad.y4m")
        self.assertIs(console.last_recording, receipt)
        self.assertFalse((self.root / "bad.y4m").exists())
        self.assertEqual(self.scene.frame, 95)
        self.assertFalse(self.scene.active)

    def test_invalid_recording_options_do_not_restore_checkpoint(self):
        console = self.console()
        console.run_cell("# clip\nscene.frame += 3")
        with self.assertRaises(ValueError):
            console.run_cell("# clip\nscene.frame += 1", record_to=self.root / "bad.png")
        self.assertEqual(self.scene.frame, 93)
        self.assertEqual(self.scene.events, [])

    def test_syntax_error_precedes_recording_and_checkpoint_restore(self):
        console = self.console()
        console.run_cell("# clip\nscene.frame += 3")
        with self.assertRaises(SyntaxError):
            console.run_cell("# clip\nif", record_to=self.root / "bad.y4m")
        self.assertEqual(self.scene.frame, 93)
        self.assertEqual(self.scene.events, [])

    def test_clipboard_recording_uses_same_session(self):
        console = self.console(clipboard=lambda: "# pasted\nscene.frame += 2")
        console.checkpoint_paste(record=True, record_to=self.root / "paste.gif")
        self.assertEqual(console.last_recording.format, "gif")
        self.assertEqual(console.last_recording.frame_count, 2)

    def test_options_without_destination_refuse(self):
        with self.assertRaisesRegex(ValueError, "requires record_to"):
            self.console().run_cell("scene.frame += 1", recording_options={"threads": 2})
        self.assertEqual(self.scene.frame, 90)

    def test_ipython_error_result_aborts_recording(self):
        error = RuntimeError("shell cell failed")
        shell = types.SimpleNamespace(run_cell=lambda code: types.SimpleNamespace(error_in_exec=error))
        with self.assertRaises(RuntimeError) as caught:
            self.console(shell=shell).run_cell("pass", record_to=self.root / "error.y4m")
        self.assertIs(caught.exception, error)
        self.assertEqual(self.scene.events, ["begin", "abort"])


if __name__ == "__main__":
    unittest.main()
