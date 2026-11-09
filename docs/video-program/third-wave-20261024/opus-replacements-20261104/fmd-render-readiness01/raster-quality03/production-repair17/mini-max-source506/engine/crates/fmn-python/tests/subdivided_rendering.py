"""Fresh native clock and public console subdivision; installed-wheel acceptance."""
from pathlib import Path
import contextlib
import io
import json
import sys
import tempfile
import textwrap
import unittest

import numpy as np
import manimlib as m
from fmn_python import render_subdivided_scene, subdivided_render_session, record_scene
from fmn_python.__main__ import main
from fmn_python.subdivision_rendering import take_subdivision_option


def frames(path, fps=8):
    header, body = path.read_bytes().split(b"\n", 1)
    assert header == f"YUV4MPEG2 W96 H54 F{fps}:1 Ip A1:1 C420mpeg2".encode(), header
    size = 96 * 54 * 3 // 2
    result = []
    while body:
        assert body.startswith(b"FRAME\n") and len(body) >= size + 6
        result.append(body[6:size + 6])
        body = body[size + 6:]
    return result


class FrontdoorTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="fmn-subdivision-frontdoor-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def options(self):
        return dict(format="y4m", resolution=(96, 54), fps=8, threads=1)

    def source(self, body):
        path = self.root / "source.py"
        path.write_text("from manimlib import *\n" + textwrap.dedent(body))
        return path

    def invoke(self, source, *extra, destination=None):
        if destination is None:
            destination = self.root / "cli-clips"
        arguments = ["fmn-python", "--robot", str(source), "Motion", "--subdivide",
                     "--format", "y4m", "--resolution", "96x54", "--fps", "8",
                     "--threads", "1", "--video_dir", str(destination), *extra]
        original, path = sys.argv, list(sys.path)
        stdout, stderr = io.StringIO(), io.StringIO()
        try:
            sys.argv = arguments
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                code = main()
        finally:
            sys.argv = original
        self.assertEqual(sys.path, path)
        lines = stdout.getvalue().splitlines()
        self.assertEqual(len(lines), 1, stdout.getvalue())
        report = json.loads(lines[0])
        self.assertEqual(report["schema"], "fmn-python.cli")
        self.assertEqual(report["exit"]["code"], code)
        return code, report, stderr.getvalue()

    def test_class_lifecycle_once_and_native_fps(self):
        calls = []
        class Motion(m.Scene):
            def __init__(self, marker):
                calls.append(("init", marker))
                super().__init__()
            def setup(self):
                calls.append(("setup", self.camera.fps))
            def construct(self):
                calls.append(("construct", self.camera.get_pixel_shape()))
                square = m.Square(fill_opacity=1)
                self.add(square)
                self.play(square.animate.shift(m.RIGHT), run_time=.25, rate_func=m.linear)
                self.wait(.5)
            def tear_down(self):
                calls.append(("tear_down", m._portal_scene_clock(self)))
        result = render_subdivided_scene(Motion, self.root / "clips",
                                         scene_kwargs={"marker": 7}, **self.options())
        self.assertEqual(calls, [("init", 7), ("setup", 8), ("construct", (96, 54)),
                                 ("tear_down", (8, 6))])
        self.assertEqual([s.render.frame_count for s in result.segments], [2, 4])
        self.assertTrue(result.completed)
        self.assertEqual([len(frames(s.render.destination)) for s in result.segments], [2, 4])

    def test_imperative_context_configures_pristine_scene_and_keeps_live_views(self):
        scene = m.Scene()
        camera = scene.camera.frame
        with subdivided_render_session(scene, self.root / "clips", **self.options()) as run:
            self.assertEqual(m._portal_scene_clock(scene), (8, 0))
            square = m.Square(fill_opacity=1)
            scene.add(square)
            view = square.data
            view["point"][:, 0] += 2
            scene.wait(.25)
            self.assertIs(scene.camera.frame, camera)
        self.assertEqual(run.result.frame_count, 2)
        np.testing.assert_allclose(square.get_center(), [2, 0, 0], atol=1e-6)
        self.assertIs(scene.mobjects[0], square)

    def test_programmatic_range_reaches_real_endpoints_without_preroll_output(self):
        class Motion(m.Scene):
            def construct(self):
                self.square = m.Square()
                self.add(self.square)
                for _ in range(3):
                    self.play(self.square.animate.shift(m.RIGHT), run_time=.25, rate_func=m.linear)
                self.tail = True
        scene = Motion()
        result = render_subdivided_scene(scene, self.root / "clips", animation_range=(1, 2), **self.options())
        self.assertEqual([s.play_index for s in result.segments], [1])
        self.assertEqual(result.frame_count, 2)
        self.assertFalse(hasattr(scene, "tail"))
        np.testing.assert_allclose(scene.square.get_center(), [2, 0, 0], atol=1e-6)

    def test_populated_or_advanced_scene_is_refused_without_losing_state(self):
        for use_time in (False, True):
            scene = m.Scene()
            square = m.Square()
            if use_time:
                scene.wait(.125)
            else:
                scene.add(square)
            clock = m._portal_scene_clock(scene)
            destination = self.root / str(use_time)
            with self.assertRaises(RuntimeError):
                with subdivided_render_session(scene, destination, **self.options()):
                    self.fail("non-pristine scene admitted")
            self.assertFalse(destination.exists())
            self.assertEqual(m._portal_scene_clock(scene), clock)
            if not use_time:
                self.assertIs(scene.mobjects[0], square)

    def test_native_preparation_cannot_steal_an_output_owner(self):
        scene = m.Scene()
        with record_scene(scene, self.root / "outer.y4m", resolution=(96, 54), threads=1):
            with self.assertRaises(RuntimeError):
                m._portal_prepare_recording_scene(scene, 96, 54, 8, 99)
            scene.wait(.125)
        self.assertEqual(len(frames(self.root / "outer.y4m", fps=30)), 4)
        self.assertEqual(m._portal_scene_clock(scene), (30, 4))

    def test_native_dimension_admission_does_not_change_clock(self):
        scene = m.Scene()
        for args in ((0, 54, 8, 0), (96, 0, 8, 0), (96, 54, 0, 0), (100000, 100000, 8, 0)):
            with self.assertRaises(ValueError):
                m._portal_prepare_recording_scene(scene, *args)
            self.assertEqual(m._portal_scene_clock(scene), (30, 0))

    def test_existing_destination_is_refused_before_native_reconfiguration(self):
        scene = m.Scene()
        destination = self.root / "clips"
        destination.mkdir()
        with self.assertRaises(FileExistsError):
            with subdivided_render_session(scene, destination, **self.options()):
                self.fail("existing output admitted")
        self.assertEqual(m._portal_scene_clock(scene), (30, 0))

    def test_cli_robot_receipts_and_authored_chatter(self):
        source = self.source('''
            print("module chatter")
            class Motion(Scene):
                def construct(self):
                    print("construct chatter")
                    square = Square(fill_opacity=1)
                    self.add(square)
                    self.play(square.animate.shift(RIGHT), run_time=.25, rate_func=linear)
                    self.wait(.5)
        ''')
        code, report, stderr = self.invoke(source)
        self.assertEqual(code, 0, report)
        self.assertEqual(report["kind"], "render-subdivided")
        self.assertEqual(report["frame_count"], 6)
        self.assertIn("module chatter", stderr)
        self.assertIn("construct chatter", stderr)
        rows = report["subdivision"]["segments"]
        self.assertEqual([row["play_index"] for row in rows], [0, 1])
        self.assertEqual([len(frames(Path(row["render"]["destination"]))) for row in rows], [2, 4])

    def test_cli_selected_range_and_thread_equivalent_payloads(self):
        source = self.source('''
            class Motion(Scene):
                def construct(self):
                    square = Square(fill_opacity=1)
                    self.add(square)
                    for _ in range(3):
                        self.play(square.animate.shift(RIGHT), run_time=.25, rate_func=linear)
        ''')
        results = []
        for threads in (1, 4):
            code, report, _ = self.invoke(source, "-n", "1,2", "--threads", str(threads),
                                          destination=self.root / f"selected-{threads}")
            self.assertEqual(code, 0, report)
            rows = report["subdivision"]["segments"]
            self.assertEqual([row["play_index"] for row in rows], [1])
            results.append(frames(Path(rows[0]["render"]["destination"])))
        self.assertEqual(results[0], results[1])

    def test_cli_failure_reports_retained_clips_and_no_incomplete_publication(self):
        source = self.source('''
            class Motion(Scene):
                def construct(self):
                    self.add(Square(fill_opacity=1))
                    self.wait(.25)
                    raise ValueError("authored failure")
        ''')
        code, report, _ = self.invoke(source)
        self.assertEqual(code, 5, report)
        self.assertTrue(report["artifact_published"])
        self.assertFalse(report["subdivision"]["completed"])
        rows = report["subdivision"]["segments"]
        self.assertEqual(len(rows), 1)
        self.assertEqual(len(frames(Path(rows[0]["render"]["destination"]))), 2)

    def test_cli_invalid_combinations_do_not_import_scene(self):
        source = self.source('raise AssertionError("source must not be imported")')
        for extra in (("--subdivide",), ("-s",), ("--format", "svg"),
                      ("--format", "png_sequence", "--reproducible"), ("--resume",)):
            with self.subTest(extra=extra):
                code, report, stderr = self.invoke(source, *extra)
                self.assertIn(code, (2, 4), report)
                self.assertNotIn("source must not be imported", stderr)
                self.assertFalse((self.root / "cli-clips").exists())

    def test_cli_existing_destination_is_refused_before_import(self):
        source = self.source('raise AssertionError("source must not be imported")')
        (self.root / "cli-clips").mkdir()
        code, report, _ = self.invoke(source)
        self.assertEqual(code, 2, report)
        self.assertEqual(report["kind"], "usage-error")

    def test_subdivide_lexer_does_not_steal_native_option_values(self):
        flags = {"--video_dir", "--vcodec", "--ffmpeg_bin", "-n"}
        for flag in flags:
            options = [flag, "--subdivide", "--subdivide"]
            self.assertEqual(take_subdivision_option(options, flags), ([flag, "--subdivide"], True))
        with self.assertRaises(ValueError):
            take_subdivision_option(["--subdivide", "--subdivide"], flags)


assert callable(getattr(m, "_portal_prepare_recording_scene", None)), "matching fresh native wheel required"
suite = unittest.defaultTestLoader.loadTestsFromTestCase(FrontdoorTests)
result = unittest.TextTestRunner(verbosity=2).run(suite)
assert result.wasSuccessful(), "native fresh-scene subdivision acceptance failed"
