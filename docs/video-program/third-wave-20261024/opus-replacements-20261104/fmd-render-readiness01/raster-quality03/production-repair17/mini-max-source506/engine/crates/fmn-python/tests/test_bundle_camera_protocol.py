"""Camera export host ownership, with explicitly modeled native boundaries."""
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python import BundleExportSession, export_bundle
from test_bundle_export_protocol import Native, CapabilityError
from test_render_session_protocol import Scene, EndScene


class CameraNative(Native):
    def _portal_begin_camera_bundle(self, scene, *arguments):
        super()._portal_begin_bundle(scene, *arguments)
        self.events[-1] = ("camera-begin", arguments)


class CameraBundleProtocol(unittest.TestCase):
    def setUp(self):
        self.native, self.scene = CameraNative(), Scene()
        self.imports = patch.dict(sys.modules, {"manimlib": self.native})
        self.imports.start()
        self.addCleanup(self.imports.stop)

    def session(self, **options):
        return BundleExportSession(self.scene, "camera.fmtl", **options)

    def test_camera_mode_uses_native_recorder_and_shared_segments(self):
        with self.session(camera=True, fps=24, resolution=(96, 54)) as session:
            self.assertIs(vars(self.scene)["_fmn_owned_render_session"], session)
            for kind in ("play", "wait"):
                with session._segment(kind) as segment:
                    segment.select(True)
        self.assertEqual([event[0] for event in self.native.events],
                         ["camera-begin", "play", "play", "wait", "wait", "finish"])
        arguments = self.native.events[0][1]
        self.assertEqual(arguments[1:4], (96, 54, 24))
        self.assertEqual(len(arguments), 8)
        self.assertTrue(session.artifact_published)
        self.assertTrue(session.result.camera_track)
        self.assertEqual(session.result.as_dict()["fmtl_minor"], 1)
        self.assertTrue(session.result.as_dict()["camera_track"])
        self.assertFalse(session.result.as_dict()["certified_source"])
        self.assertNotIn("_fmn_owned_render_session", vars(self.scene))
        self.assertNotIn("_fmn_subdivision_session", vars(self.scene))

    def test_default_keeps_original_entry_and_receipt_keys(self):
        with self.session() as session:
            pass
        self.assertEqual(self.native.events[0][0], "begin")
        self.assertFalse(session.result.camera_track)
        self.assertNotIn("camera_track", session.result.as_dict())
        self.assertNotIn("fmtl_minor", session.result.as_dict())

    def test_camera_selection_must_be_boolean_before_factory_or_acquisition(self):
        constructed = []
        class Authored(Scene):
            def __init__(self):
                constructed.append(1)
                super().__init__()
        for camera in (None, 0, 1, "true", [], object()):
            with self.subTest(camera=camera):
                with self.assertRaisesRegex(TypeError, "camera must be a bool"):
                    export_bundle(Authored, "camera.fmtl", camera=camera)
        self.assertEqual(constructed, [])
        self.assertEqual(self.native.events, [])

    def test_missing_camera_capability_refuses_before_constructing(self):
        calls = []
        class Authored(Scene):
            def __init__(self):
                calls.append(1)
                super().__init__()
        with patch.dict(sys.modules, {"manimlib": Native()}):
            with self.assertRaisesRegex(CapabilityError, "matching native wheel"):
                export_bundle(Authored, "camera.fmtl", camera=True)
        self.assertEqual(calls, [])

    def test_camera_acquisition_does_not_cancel_existing_owner(self):
        self.scene.active = True
        with self.assertRaisesRegex(RuntimeError, "external owner"):
            self.session(camera=True).__enter__()
        self.assertTrue(self.scene.active)
        self.assertNotIn(("abort",), self.scene.events)
        self.assertNotIn("_fmn_owned_render_session", vars(self.scene))

    def test_camera_exports_preserve_swallowed_capture_failures(self):
        original = LookupError("camera capture failed")
        session = self.session(camera=True)
        with self.assertRaises(LookupError) as caught:
            with session:
                try:
                    with session._segment("play") as segment:
                        segment.select(True)
                        raise original
                except LookupError:
                    pass
        self.assertIs(caught.exception, original)
        self.assertNotIn(("finish",), self.native.events)
        self.assertFalse(session.artifact_published)
        self.assertFalse(self.scene.active)

    def test_camera_exports_keep_fps_and_offline_guards(self):
        with self.assertRaisesRegex(ValueError, "FPS"):
            with self.session(camera=True):
                self.scene.camera.fps += 1
        self.scene = Scene()
        self.scene.presenter_mode = True
        with self.assertRaises(CapabilityError):
            with self.session(camera=True):
                self.fail("presenter session must not enter user code")
        self.assertNotIn(("finish",), self.native.events)

    def test_constructor_once_with_args_and_normal_early_end(self):
        calls = []
        class Authored(Scene):
            def __init__(self, **kwargs):
                calls.append(kwargs)
                super().__init__(**kwargs)
            def run(self):
                calls.append("run")
                raise EndScene()
        result = export_bundle(Authored, "camera.fmtl", camera=True,
                               scene_kwargs={"random_seed": 41})
        self.assertEqual(calls, [{"random_seed": 41}, "run"])
        self.assertTrue(result.camera_track)
        self.assertEqual(result.frame_count, 7)

    def test_publication_failure_releases_tokens_without_success(self):
        session = self.session(camera=True)
        original = FileExistsError("concurrent output")
        with self.assertRaises(FileExistsError) as caught:
            with session:
                self.native.error = original
        self.assertIs(caught.exception, original)
        self.assertIsNone(session.result)
        self.assertFalse(session.artifact_published)
        self.assertNotIn("_fmn_owned_render_session", vars(self.scene))
        self.assertNotIn("_fmn_subdivision_session", vars(self.scene))


if __name__ == "__main__":
    unittest.main()
