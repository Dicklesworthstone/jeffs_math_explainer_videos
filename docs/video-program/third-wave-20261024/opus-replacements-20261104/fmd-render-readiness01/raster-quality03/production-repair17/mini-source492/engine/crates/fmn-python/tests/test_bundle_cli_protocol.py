"""Real common CLI parser and SceneSource, with a native output-boundary spy."""
import contextlib
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python.bundle_cli import try_bundle_cli
from test_console_rendering_protocol import native_fixture


class BundleCliProtocol(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="bundle-cli-protocol-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.cwd = Path.cwd()
        os.chdir(self.root)
        self.addCleanup(os.chdir, self.cwd)
        self.native = native_fixture()
        self.modules = patch.dict(sys.modules, {"manimlib": self.native})
        self.modules.start()
        self.addCleanup(self.modules.stop)
        self.source = self.root / "source with spaces.py"
        self.source.write_text("from manimlib import Scene\nclass Example(Scene):\n    pass\n")
        self.output = self.root / "animation.fmtl"
        self.native.bundle_calls = []
        def begin(scene, path, width, height, fps, seed, *limits):
            self.native.bundle_calls.append(("begin", path, width, height, fps))
            scene.destination = path
            scene.active = True
        def segment(scene, kind, begin):
            self.native.bundle_calls.append((kind, begin))
        def finish(scene):
            self.native.bundle_calls.append(("finish",))
            # A boundary spy, NOT a valid FMTL or a substitute native renderer.
            with open(scene.destination, "xb") as output:
                output.write(b"test-boundary-not-FMTL")
            scene.active = False
            return scene.destination, 7, 2, 22, "a" * 64
        self.native._portal_begin_bundle = begin
        self.native._portal_bundle_segment = segment
        self.native._portal_finish_bundle = finish

    def invoke(self, *args):
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            result = try_bundle_cli(self.native, list(args))
        text = out.getvalue()
        return result, json.loads(text) if text.startswith("{") else None, err.getvalue()

    def export(self, *args):
        return self.invoke("--robot", str(self.source), "--format", "fmtl",
                           "--video_dir", str(self.output), *args)

    def test_selected_scene_and_common_options_reach_only_bundle_owner(self):
        code, receipt, _ = self.export("Example", "--resolution", "128x72", "--fps", "24")
        self.assertEqual(code, 0)
        self.assertTrue(receipt["exported"])
        self.assertEqual(receipt["bundle"]["replay_resolution"], [128, 72])
        self.assertFalse(receipt["bundle"]["certified_source"])
        self.assertEqual(self.native.bundle_calls,
                         [("begin", str(self.output), 128, 72, 24), ("finish",)])
        self.assertNotIn("begin", self.native.instances[0].events)
        self.assertEqual(len(self.native.instances), 1)

    def test_equals_options_and_default_destination(self):
        self.source = self.root / "simple.py"
        self.source.write_text("from manimlib import Scene\nclass Example(Scene):\n    pass\n")
        destination = self.root / "media/videos/simple/Example.fmtl"
        destination.parent.mkdir(parents=True)
        code, receipt, _ = self.invoke("--robot", str(self.source), "--format=fmtl", "--fps=24")
        self.assertEqual(code, 0)
        self.assertEqual(receipt["destination"], str(destination))

    def test_no_wrong_interception_for_option_values(self):
        for args in ((str(self.source), "--video_dir", "fmtl"),
                     (str(self.source), "--video_dir", "--format", "fmtl"),
                     (str(self.source), "--format", "png")):
            self.assertIsNone(self.invoke(*args)[0])
        self.assertEqual(self.native.bundle_calls, [])

    def test_unsupported_modes_fail_before_import(self):
        self.source.write_text("raise AssertionError('source executed')\n")
        for extra in (("--reproducible",), ("--threads", "4"),
                      ("--subdivide",), ("-n", "2"), ("--skip_animations",),
                      ("--transparent",), ("--vcodec", "auto")):
            with self.subTest(extra=extra):
                code, receipt, _ = self.export(*extra)
                self.assertEqual(code, 4)
                self.assertEqual(receipt["phase"], "options")
        self.assertEqual(self.native.instances, [])

    def test_invalid_common_values_are_usage_before_source(self):
        self.source.write_text("raise AssertionError('source executed')\n")
        for args in (("--resolution", "oops"), ("--fps", "0"),
                     ("--fps", "241"), ("--format", "gif")):
            with self.subTest(args=args):
                code, report, _ = self.export(*args)
                self.assertEqual(code, 2)
                self.assertEqual(report["phase"], "options")

    def test_missing_capability_before_source_execution(self):
        del self.native._portal_begin_bundle
        self.source.write_text("raise AssertionError('source executed')\n")
        code, receipt, _ = self.export()
        self.assertEqual(code, 4)
        self.assertIn("matching native wheel", receipt["message"])

    def test_robot_output_and_scoped_lazy_import(self):
        (self.root / "helper.py").write_text("VALUE = 'scoped helper'\n")
        self.source.write_text("from manimlib import Scene\nprint('module output')\n"
                               "class Example(Scene):\n"
                               "    def construct(self):\n"
                               "        import helper\n        print(helper.VALUE)\n")
        code, receipt, err = self.export()
        self.assertEqual(code, 0)
        self.assertEqual(receipt["kind"], "bundle-export")
        self.assertIn("module output", err)
        self.assertIn("scoped helper", err)
        self.assertNotIn("helper", sys.modules)

    def test_destination_collision_before_constructor_or_source(self):
        self.output.write_bytes(b"keep")
        self.source.write_text("raise AssertionError('source executed')\n")
        code, receipt, _ = self.export()
        self.assertEqual(code, 6)
        self.assertEqual(receipt["phase"], "start")
        self.assertEqual(self.output.read_bytes(), b"keep")
        self.assertEqual(self.native.instances, [])

    def test_multiple_or_unknown_selection_no_constructor(self):
        code, receipt, _ = self.export("Absent")
        self.assertEqual(code, 2)
        self.assertEqual(receipt["phase"], "select")
        code, _, _ = self.export("Example", "Other")
        self.assertEqual(code, 2)
        self.assertEqual(self.native.instances, [])

    def test_execution_exception_aborts_without_publication(self):
        self.source.write_text("from manimlib import Scene\nclass Example(Scene):\n"
                               "    def construct(self):\n        raise ValueError('authored failure')\n")
        code, receipt, _ = self.export()
        self.assertEqual(code, 5)
        self.assertEqual(receipt["phase"], "execute")
        self.assertFalse(receipt["artifact_published"])
        self.assertFalse(self.output.exists())
        self.assertEqual(self.native.instances[0].events[-1], "abort")

    def test_interrupt_aborts_without_publication(self):
        self.source.write_text("from manimlib import Scene\nclass Example(Scene):\n"
                               "    def construct(self):\n        raise KeyboardInterrupt()\n")
        code, receipt, _ = self.export()
        self.assertEqual(code, 130)
        self.assertFalse(receipt["artifact_published"])
        self.assertFalse(self.output.exists())

    def test_bundle_help_has_no_authored_work(self):
        code, receipt, _ = self.export("--help")
        self.assertEqual(code, 0)
        self.assertIn("code-free", receipt["help"])
        self.assertEqual(self.native.instances, [])


    def enable_camera(self):
        begin = self.native._portal_begin_bundle
        def camera_begin(*args):
            begin(*args)
            self.native.bundle_calls[-1] = ("camera", *self.native.bundle_calls[-1][1:])
        self.native._portal_begin_camera_bundle = camera_begin

    def test_camera_console_uses_only_camera_owner_and_reports_wire_version(self):
        self.enable_camera()
        code, receipt, _ = self.export("Example", "--bundle-camera", "--fps=24", "--resolution=128x72")
        self.assertEqual(code, 0)
        self.assertTrue(receipt["bundle"]["camera_track"])
        self.assertEqual(receipt["bundle"]["fmtl_minor"], 1)
        self.assertFalse(receipt["bundle"]["certified_source"])
        self.assertEqual(self.native.bundle_calls,
                         [("camera", str(self.output), 128, 72, 24), ("finish",)])
        self.assertEqual(len(self.native.instances), 1)
        self.assertNotIn("begin", self.native.instances[0].events)

    def test_camera_console_does_not_require_planar_entry(self):
        self.enable_camera()
        del self.native._portal_begin_bundle
        code, receipt, _ = self.export("--bundle-camera")
        self.assertEqual(code, 0)
        self.assertTrue(receipt["exported"])

    def test_default_receipt_stays_planar_even_with_camera_capability(self):
        self.enable_camera()
        code, receipt, _ = self.export()
        self.assertEqual(code, 0)
        self.assertNotIn("camera_track", receipt["bundle"])
        self.assertNotIn("fmtl_minor", receipt["bundle"])
        self.assertEqual(self.native.bundle_calls[0][0], "begin")

    def test_camera_capability_is_required_before_import(self):
        self.source.write_text("raise AssertionError('source executed')\n")
        for value in (None, False, "unavailable"):
            with self.subTest(value=value):
                self.native._portal_begin_camera_bundle = value
                code, receipt, _ = self.export("--bundle-camera")
                self.assertEqual(code, 4)
                self.assertEqual(receipt["phase"], "options")
                self.assertFalse(receipt["artifact_published"])
        self.assertEqual(self.native.instances, [])
        self.assertEqual(self.native.bundle_calls, [])

    def test_camera_requires_fmtl_and_is_not_repeatable(self):
        self.enable_camera()
        self.source.write_text("raise AssertionError('source executed')\n")
        for args in (("--bundle-camera",), ("--bundle-camera", "--format", "png_sequence"),
                     ("--bundle-camera", "--bundle-camera", "--format", "fmtl")):
            with self.subTest(args=args):
                code, receipt, _ = self.invoke("--robot", str(self.source), *args)
                self.assertEqual(code, 2)
                self.assertEqual(receipt["phase"], "options")
        self.assertEqual(self.native.instances, [])
        self.assertEqual(self.native.bundle_calls, [])

    def test_camera_switch_inside_option_value_is_not_interpreted(self):
        self.assertIsNone(self.invoke("--video_dir", "--bundle-camera")[0])
        code, receipt, _ = self.invoke("--robot", str(self.source), "--format=fmtl",
                                        "--video_dir=--bundle-camera")
        self.assertEqual(code, 0)
        self.assertNotIn("camera_track", receipt["bundle"])
        self.assertEqual(Path(receipt["destination"]).name, "--bundle-camera")
        self.assertEqual(self.native.bundle_calls[0][0], "begin")

    def test_camera_keeps_unsupported_playback_modes_fail_closed(self):
        self.enable_camera()
        self.source.write_text("raise AssertionError('source executed')\n")
        for extra in (("--reproducible",), ("--skip_animations",), ("-n", "2"),
                      ("--transparent",), ("--threads", "4")):
            with self.subTest(extra=extra):
                code, receipt, _ = self.export("--bundle-camera", *extra)
                self.assertEqual(code, 4)
                self.assertEqual(receipt["phase"], "options")
        self.assertEqual(self.native.instances, [])
        self.assertEqual(self.native.bundle_calls, [])

    def test_camera_publication_error_aborts_without_success(self):
        self.enable_camera()
        def fail(scene):
            raise ValueError("native output budget exceeded")
        self.native._portal_finish_bundle = fail
        code, receipt, _ = self.export("--bundle-camera")
        self.assertEqual(code, 6)
        self.assertEqual(receipt["phase"], "finish")
        self.assertFalse(receipt["exported"])
        self.assertFalse(receipt["artifact_published"])
        self.assertFalse(self.output.exists())
        self.assertEqual(self.native.instances[0].events[-1], "abort")

    def test_camera_help_needs_no_native_capability_or_source(self):
        self.source.write_text("raise AssertionError('source executed')\n")
        code, receipt, _ = self.export("--bundle-camera", "--help")
        self.assertEqual(code, 0)
        self.assertIn("--bundle-camera", receipt["help"])
        self.assertIn("minor-1", receipt["help"])
        self.assertEqual(self.native.instances, [])
        self.assertEqual(self.native.bundle_calls, [])


if __name__ == "__main__":
    unittest.main()
