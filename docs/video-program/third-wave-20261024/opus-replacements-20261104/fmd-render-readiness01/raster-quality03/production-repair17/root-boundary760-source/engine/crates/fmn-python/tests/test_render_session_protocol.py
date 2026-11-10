"""Production rendering orchestration with a native-boundary protocol double.

This suite does not render pixels or establish native sink acceptance. The
compiled-extension companion exercises real outputs through the same API.
"""
import concurrent.futures
import pathlib
import sys
import types
import unittest
from unittest.mock import patch

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / "python"))
from fmn_python import RenderResult, RenderSession, render_scene, render_session


class EndScene(Exception):
    pass


class Camera:
    def __init__(self):
        self.shape = (96, 54)
        self.fps = 8
        self._core = self
        self.frame = object()
        self.error = None

    def get_pixel_shape(self):
        return self.shape

    def set_pixel_shape(self, width, height):
        if self.error is not None:
            raise self.error
        self.shape = (width, height)


class Scene:
    def __init__(self, **kwargs):
        self.random_seed = kwargs.get("random_seed", 0)
        self.camera = Camera()
        self.events = []
        self.active = False
        self.start_error = self.run_error = self.finish_error = self.abort_error = None
        self._render_invocations = [{"argv": ["ffmpeg", "-i", "pipe:0"]}]
        self._render_audio_inputs = []

    def _begin_native_output(self, *args):
        self.events.append(("begin", args))
        if self.active:
            raise RuntimeError("external generation already active")
        if self.start_error is not None:
            raise self.start_error
        self.active = True
        self.request = args

    def _abort_render(self):
        self.events.append(("abort",))
        self.active = False
        if self.abort_error is not None:
            raise self.abort_error

    def _finish_render(self):
        self.events.append(("finish",))
        if self.finish_error is not None:
            raise self.finish_error
        self.active = False
        path, format, width, height, fps, threads, seed, *rest = self.request
        return (path, 4800 if format == "wav" else 4, 1234, "a" * 64, "native-test-boundary", threads)

    def run(self):
        self.events.append(("run",))
        if self.run_error is not None:
            raise self.run_error


native = types.SimpleNamespace(Scene=Scene, EndScene=EndScene)


class RenderSessionTests(unittest.TestCase):
    def setUp(self):
        self.import_patch = patch.dict(sys.modules, {"manimlib": native})
        self.import_patch.start()
        self.addCleanup(self.import_patch.stop)
        self.scene = Scene()

    def session(self, path="movie.y4m", **kwargs):
        return render_session(self.scene, path, **kwargs)

    def test_audio_decode_receipt_is_available_for_wav_and_detached_from_native(self):
        self.scene._render_audio_inputs = [{"source_sha256": "a" * 64, "decoder": "ffmpeg"}]
        result = render_scene(self.scene, "soundtrack.wav", threads=1)
        self.assertEqual(result.sample_frames, 4800)
        self.assertEqual(result.ffmpeg_invocations[0]["argv"], ["ffmpeg", "-i", "pipe:0"])
        self.scene._render_audio_inputs[0]["source_sha256"] = "changed"
        self.assertEqual(result.audio_inputs[0]["source_sha256"], "a" * 64)
        report = result.as_dict()
        report["audio_inputs"][0]["source_sha256"] = "changed again"
        self.assertEqual(result.audio_inputs[0]["source_sha256"], "a" * 64)

    def test_imperative_context_publishes_one_native_receipt(self):
        session = self.session()
        self.assertEqual(self.scene.events, [])
        with session as same:
            self.assertIs(same, session)
            self.assertTrue(self.scene.active)
            self.assertIsNone(session.result)
            self.scene.run()
        self.assertEqual([event[0] for event in self.scene.events], ["begin", "run", "finish"])
        self.assertEqual(session.result.frame_count, 4)
        self.assertEqual(session.result.bytes, 1234)
        self.assertFalse(session.result.certified)
        self.assertFalse(hasattr(self.scene, "_fmn_owned_render_session"))

    def test_render_scene_uses_class_kwargs_once(self):
        calls = []
        class Custom(Scene):
            def __init__(self, **kwargs):
                calls.append(dict(kwargs))
                super().__init__(**kwargs)
        arguments = {"random_seed": 29}
        result = render_scene(Custom, "out.png", scene_kwargs=arguments, threads=2)
        self.assertEqual(calls, [{"random_seed": 29}])
        self.assertEqual(arguments, {"random_seed": 29})
        self.assertEqual(result.seed, 29)
        self.assertEqual(result.threads, 2)

    def test_instance_configuration_is_not_reconstructed(self):
        result = render_scene(self.scene, "out.gif", fps=12, resolution=(128, 72))
        self.assertEqual(result.resolution, (128, 72))
        self.assertEqual(self.scene.camera.get_pixel_shape(), (128, 72))
        self.assertEqual(self.scene.camera.fps, 12)

    def test_pose_identity_is_preserved_by_resolution_override(self):
        frame = self.scene.camera.frame
        render_scene(self.scene, "out.gif", resolution=(128, 128))
        self.assertIs(self.scene.camera.frame, frame)

    def test_end_scene_is_successful_early_completion(self):
        self.scene.run_error = EndScene("finished early")
        result = render_scene(self.scene, "out.png")
        self.assertIsInstance(result, RenderResult)
        self.assertNotIn(("abort",), self.scene.events)

    def test_baseexceptions_abort_and_propagate_exact_instance(self):
        for failure in (ValueError("construct"), KeyboardInterrupt(), SystemExit(2)):
            self.scene = Scene()
            self.scene.run_error = failure
            with self.assertRaises(type(failure)) as raised:
                render_scene(self.scene, "out.png")
            self.assertIs(raised.exception, failure)
            self.assertEqual(self.scene.events[-1], ("abort",))
            self.assertFalse(self.scene.active)
            self.assertNotIn(("finish",), self.scene.events)

    def test_cancellation_failure_does_not_mask_original(self):
        failure = ValueError("primary")
        self.scene.run_error = failure
        self.scene.abort_error = RuntimeError("secondary")
        with self.assertRaises(ValueError) as raised:
            render_scene(self.scene, "out.png")
        self.assertIs(raised.exception, failure)
        self.assertIn("secondary", raised.exception.__notes__[0])
        self.assertFalse(hasattr(self.scene, "_fmn_owned_render_session"))

    def test_failed_open_does_not_abort_someone_elses_generation(self):
        self.scene.active = True
        with self.assertRaisesRegex(RuntimeError, "external generation"):
            render_scene(self.scene, "out.png")
        self.assertTrue(self.scene.active)
        self.assertEqual(len(self.scene.events), 1)

    def test_native_start_failure_is_not_false_success(self):
        self.scene.start_error = OSError("no-clobber")
        session = self.session()
        with self.assertRaises(OSError):
            with session:
                self.fail("body must not run")
        self.assertIsNone(session.result)
        self.assertEqual(len(self.scene.events), 1)

    def test_failed_camera_setup_cancels_newly_owned_generation(self):
        self.scene.camera.error = RuntimeError("camera configuration")
        with self.assertRaisesRegex(RuntimeError, "camera configuration"):
            self.session().__enter__()
        self.assertEqual([e[0] for e in self.scene.events], ["begin", "abort"])
        self.assertFalse(hasattr(self.scene, "_fmn_owned_render_session"))

    def test_failed_native_finish_aborts_and_has_no_receipt(self):
        self.scene.finish_error = OSError("sink failed")
        session = self.session()
        with self.assertRaisesRegex(OSError, "sink failed"):
            with session:
                pass
        self.assertIsNone(session.result)
        self.assertEqual(self.scene.events[-1], ("abort",))

    def test_finish_and_abort_are_idempotent_after_publication(self):
        session = self.session()
        with session:
            result = session.finish()
        self.assertIs(session.finish(), result)
        session.abort()
        self.assertEqual(self.scene.events.count(("finish",)), 1)
        self.assertNotIn(("abort",), self.scene.events)

    def test_explicit_abort_never_publishes(self):
        session = self.session()
        with session:
            session.abort()
            session.abort()
        with self.assertRaises(RuntimeError):
            session.finish()
        self.assertIsNone(session.result)
        self.assertEqual(self.scene.events.count(("abort",)), 1)
        self.assertNotIn(("finish",), self.scene.events)

    def test_nested_owned_session_fails_without_cancelling_outer(self):
        with self.session() as outer:
            with self.assertRaisesRegex(RuntimeError, "already has"):
                with self.session("nested.gif"):
                    pass
            self.assertTrue(self.scene.active)
        self.assertIsNotNone(outer.result)
        self.assertNotIn(("abort",), self.scene.events)

    def test_reenter_is_refused(self):
        session = self.session()
        with session:
            with self.assertRaisesRegex(RuntimeError, "only once"):
                session.__enter__()
        with self.assertRaisesRegex(RuntimeError, "only once"):
            session.__enter__()

    def test_thread_refusal_keeps_owner_able_to_finish(self):
        with self.session() as session:
            with concurrent.futures.ThreadPoolExecutor(max_workers=1) as executor:
                for operation in (session.abort, session.finish, session.__enter__):
                    with self.assertRaisesRegex(RuntimeError, "creating thread"):
                        executor.submit(operation).result()
            self.assertTrue(self.scene.active)
        self.assertIsNotNone(session.result)

    def test_wav_counts_samples_not_video_frames(self):
        result = render_scene(self.scene, "sound.wav")
        self.assertIsNone(result.frame_count)
        self.assertEqual(result.sample_frames, 4800)
        self.assertEqual(result.as_dict()["sample_rate"], 48000)
        self.assertEqual(result.as_dict()["channels"], 2)

    def test_video_provenance_is_detached_from_native_list(self):
        result = render_scene(self.scene, "out.mp4")
        self.scene._render_invocations[0]["argv"].append("changed")
        exported = result.as_dict()
        exported["ffmpeg_invocations"][0]["argv"].append("edited")
        self.assertEqual(result.ffmpeg_invocations[0]["argv"], ["ffmpeg", "-i", "pipe:0"])
        self.assertFalse(result.certified)

    def test_supported_explicit_formats_and_suffix_inference(self):
        for format in ("png", "png_sequence", "gif", "y4m", "wav", "mp4", "mov"):
            self.assertEqual(self.session("artifact", format=format).format, format)
        self.assertEqual(self.session("still.PNG").format, "png")
        self.assertEqual(self.session("frames").format, "png_sequence")
        self.assertEqual(self.session("frames.v1", format="png_sequence").format, "png_sequence")

    def test_unsupported_format_is_not_silently_substituted(self):
        for path, format in [("out.webp", None), ("out.png", "jpeg"), ("out.png", [])]:
            with self.assertRaises(ValueError):
                self.session(path, format=format)
        self.assertEqual(self.scene.events, [])

    def test_invalid_numeric_configuration_refuses_before_start(self):
        for key, values in {"fps": [0, -1, True, 1.5, 1 << 32], "threads": [0, False, "4"]}.items():
            for value in values:
                with self.assertRaises((TypeError, ValueError)):
                    self.session(**{key: value})
        for dimensions in [(0, 54), (96, -1), (True, 54), (96,), (96, 54, 1), None]:
            if dimensions is None:
                continue
            with self.assertRaises((TypeError, ValueError)):
                self.session(resolution=dimensions)
        self.assertEqual(self.scene.events, [])

    def test_invalid_destinations_refuse_before_start(self):
        for destination in ("", "x\0y", b"bytes.png", 42):
            with self.assertRaises((TypeError, ValueError)):
                self.session(destination)
        self.assertEqual(self.scene.events, [])

    def test_invalid_scene_and_instance_kwargs_refuse(self):
        with self.assertRaises(TypeError):
            render_scene(object(), "out.png")
        with self.assertRaises(TypeError):
            render_scene(self.scene, "out.png", scene_kwargs={})
        self.assertEqual(self.scene.events, [])

    def test_seed_validation_and_portal_none_default(self):
        self.scene.random_seed = None
        self.assertEqual(self.session().seed, 0)
        for value in (-1, 1 << 64, 1.2, True):
            self.scene.random_seed = value
            with self.assertRaises((ValueError, TypeError)):
                self.session()
        self.assertEqual(self.scene.events, [])


if __name__ == "__main__":
    unittest.main()
