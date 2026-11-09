"""Host ownership tests with a native-boundary spy, not pixel/native acceptance."""
import concurrent.futures
from pathlib import Path
import sys
import types
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python import BundleExportSession, export_bundle
from test_render_session_protocol import Scene, EndScene


class CapabilityError(RuntimeError):
    pass


class Native:
    Scene = Scene
    EndScene = EndScene
    _CapabilityError = CapabilityError

    def __init__(self):
        self.events = []
        self.error = None

    def _portal_begin_bundle(self, scene, *arguments):
        if scene.active:
            raise RuntimeError("external owner")
        self.events.append(("begin", arguments))
        scene.active = True
        scene.destination = arguments[0]

    def _portal_bundle_segment(self, scene, kind, begin):
        self.events.append((kind, begin))
        if self.error is not None:
            raise self.error

    def _portal_finish_bundle(self, scene):
        self.events.append(("finish",))
        if self.error is not None:
            raise self.error
        scene.active = False
        return scene.destination, 7, 3, 789, "a" * 64


class BundleProtocol(unittest.TestCase):
    def setUp(self):
        self.native, self.scene = Native(), Scene()
        self.imports = patch.dict(sys.modules, {"manimlib": self.native})
        self.imports.start()
        self.addCleanup(self.imports.stop)

    def session(self, **options):
        return BundleExportSession(self.scene, "scene.fmtl", **options)

    def test_public_result_and_segment_boundaries(self):
        with self.session(fps=24, resolution=(128, 72)) as session:
            self.assertIs(vars(self.scene)["_fmn_owned_render_session"], session)
            self.assertIs(vars(self.scene)["_fmn_subdivision_session"], session)
            for kind in ("play", "wait"):
                with session._segment(kind) as segment:
                    segment.select(True)
            with session._segment("play") as segment:
                segment.select(False)  # empty play(), no native lifecycle
        self.assertEqual([e[0] for e in self.native.events],
                         ["begin", "play", "play", "wait", "wait", "finish"])
        self.assertEqual(self.native.events[1:5],
                         [("play", True), ("play", False), ("wait", True), ("wait", False)])
        self.assertEqual(session.result.frame_count, 7)
        self.assertEqual(session.result.segment_count, 3)
        self.assertEqual(session.result.as_dict()["certified_source"], False)
        self.assertEqual(session.result.resolution, (128, 72))
        self.assertTrue(session.artifact_published)
        self.assertNotIn("_fmn_owned_render_session", vars(self.scene))
        self.assertNotIn("_fmn_subdivision_session", vars(self.scene))

    def test_swallowed_segment_failure_cannot_publish_partial_bundle(self):
        error = LookupError("updater failed")
        session = self.session()
        with self.assertRaises(LookupError) as caught:
            with session:
                try:
                    with session._segment("play") as segment:
                        segment.select(True)
                        raise error
                except LookupError:
                    pass
        self.assertIs(caught.exception, error)
        self.assertNotIn(("finish",), self.native.events)
        self.assertFalse(session.artifact_published)
        self.assertFalse(self.scene.active)

    def test_swallowed_native_segment_finish_failure_is_sticky(self):
        error = RuntimeError("missing final capture")
        session = self.session()
        with self.assertRaises(RuntimeError) as caught:
            with session:
                try:
                    with session._segment("wait") as segment:
                        segment.select(True)
                        self.native.error = error
                except RuntimeError:
                    self.native.error = None
        self.assertIs(caught.exception, error)
        self.assertNotIn(("finish",), self.native.events)

    def test_presegment_end_scene_is_normal_but_inflight_end_is_not(self):
        with self.session() as session:
            try:
                with session._segment("play"):
                    raise EndScene()
            except EndScene:
                pass
        self.assertIsNotNone(session.result)
        self.scene = Scene()
        session = self.session()
        with self.assertRaises(EndScene):
            with session:
                try:
                    with session._segment("play") as segment:
                        segment.select(True)
                        raise EndScene()
                except EndScene:
                    pass
        self.assertIsNone(session.result)

    def test_primary_error_survives_abort_error(self):
        error = KeyboardInterrupt()
        self.scene.abort_error = ValueError("cleanup")
        with self.assertRaises(KeyboardInterrupt) as caught:
            with self.session():
                raise error
        self.assertIs(caught.exception, error)
        self.assertIn("ValueError", error.__notes__[0])
        self.assertNotIn("_fmn_subdivision_session", vars(self.scene))

    def test_unsupported_playback_is_refused_even_for_empty_scene(self):
        for key, value in (("skip_animations", True), ("presenter_mode", True),
                           ("start_at_animation_number", 1), ("end_at_animation_number", 3)):
            with self.subTest(key=key):
                self.scene = Scene()
                setattr(self.scene, key, value)
                with self.assertRaises(CapabilityError):
                    with self.session():
                        self.fail("body must not run")
                self.assertFalse(self.scene.active)

    def test_midflight_fps_change_is_refused(self):
        session = self.session()
        with self.assertRaisesRegex(ValueError, "FPS"):
            with session:
                self.scene.camera.fps += 1
        self.assertIsNone(session.result)

    def test_existing_output_owner_is_not_cancelled(self):
        self.scene.active = True
        with self.assertRaisesRegex(RuntimeError, "external owner"):
            self.session().__enter__()
        self.assertTrue(self.scene.active)
        self.assertNotIn(("abort",), self.scene.events)

    def test_nested_session_does_not_cancel_outer(self):
        with self.session() as session:
            with self.assertRaisesRegex(RuntimeError, "already has"):
                self.session().__enter__()
            self.assertTrue(self.scene.active)
        self.assertIsNotNone(session.result)

    def test_missing_native_capability_fails_before_constructing_scene(self):
        calls = []
        class UserScene(Scene):
            def __init__(self):
                calls.append(1)
                super().__init__()
        with patch.dict(sys.modules, {"manimlib": types.SimpleNamespace(Scene=Scene)}):
            with self.assertRaisesRegex(RuntimeError, "matching native wheel"):
                export_bundle(UserScene, "scene.fmtl")
        self.assertEqual(calls, [])

    def test_nonpositive_or_oversized_budgets_fail_before_native_acquisition(self):
        for key in ("max_frames", "max_capture_bytes", "max_output_bytes"):
            for value in (0, -1, 2**32, True):
                with self.subTest(key=key, value=value):
                    with self.assertRaises((TypeError, ValueError)):
                        self.session(**{key: value})
        self.assertEqual(self.native.events, [])

    def test_native_publication_error_has_no_success_receipt(self):
        session = self.session()
        with self.assertRaises(FileExistsError):
            with session:
                self.native.error = FileExistsError("destination won by another writer")
        self.assertIsNone(session.result)
        self.assertFalse(session.artifact_published)
        self.assertNotIn("_fmn_owned_render_session", vars(self.scene))

    def test_thread_confinement(self):
        with self.session() as session:
            with concurrent.futures.ThreadPoolExecutor(max_workers=1) as executor:
                with self.assertRaisesRegex(RuntimeError, "creating thread"):
                    executor.submit(session.abort).result()
            self.assertTrue(self.scene.active)
        self.assertIsNotNone(session.result)

    def test_scene_class_constructed_once_and_end_scene_accepted(self):
        calls = []
        class UserScene(Scene):
            def __init__(self, **kwargs):
                calls.append(kwargs)
                super().__init__(**kwargs)
            def run(self):
                raise EndScene()
        result = export_bundle(UserScene, "scene.fmtl", scene_kwargs={"random_seed": 17})
        self.assertEqual(calls, [{"random_seed": 17}])
        self.assertEqual(result.frame_count, 7)


if __name__ == "__main__":
    unittest.main()
