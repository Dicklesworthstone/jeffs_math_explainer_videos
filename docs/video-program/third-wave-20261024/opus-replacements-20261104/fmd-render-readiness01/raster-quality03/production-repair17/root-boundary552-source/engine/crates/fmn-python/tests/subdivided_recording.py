"""Per-play native Reel generations; no mock Scene, clock, or renderer."""
from pathlib import Path
import hashlib
import json
import tempfile
import traceback
import unittest

import numpy as np
import manimlib as m
from fmn_python import record_scene, record_subdivided_scene, render_session


def payloads(path):
    header, body = path.read_bytes().split(b"\n", 1)
    assert header == b"YUV4MPEG2 W96 H54 F30:1 Ip A1:1 C420mpeg2", header
    size = 96 * 54 * 3 // 2
    result = []
    while body:
        assert body.startswith(b"FRAME\n") and len(body) >= size + 6
        result.append(body[6:size + 6])
        body = body[size + 6:]
    return result


def white_square():
    return m.Square(side_length=1, fill_color=m.WHITE, fill_opacity=1, stroke_width=0)


class SubdivisionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="fmn-subdivision-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def session(self, scene, name="clips", **options):
        options.setdefault("format", "y4m")
        options.setdefault("resolution", (96, 54))
        options.setdefault("threads", 1)
        return record_subdivided_scene(scene, self.root / name, **options)

    def assert_released(self, scene):
        self.assertNotIn("_fmn_subdivision_session", vars(scene))
        self.assertNotIn("_fmn_owned_render_session", vars(scene))
        self.assertNotIn("_fmn_scene_execution", vars(scene))

    def test_populated_scene_views_clock_and_actual_motion(self):
        scene = m.Scene()
        square = white_square().shift(2 * m.LEFT)
        scene.add(square)
        scene.wait(.125)
        view = square.data
        with self.session(scene) as run:
            view["point"][:, 0] += .25
            scene.play(square.animate.shift(3 * m.RIGHT), run_time=.125, rate_func=m.linear)
            scene.wait(.125)
        self.assertIs(scene.mobjects[0], square)
        np.testing.assert_allclose(square.get_center(), [1.25, 0, 0], atol=1e-6)
        self.assertEqual(m._portal_scene_clock(scene), (30, 12))
        self.assertEqual([(s.play_index, s.kind, s.start_frame, s.end_frame)
                          for s in run.segments], [(1, "play", 4, 8), (2, "wait", 8, 12)])
        self.assertEqual(run.result.frame_count, 8)
        self.assertTrue(run.result.completed)
        self.assertIs(run.finish(), run.result)
        centers = []
        for frame in payloads(run.segments[0].render.destination):
            xs = [i % 96 for i, value in enumerate(frame[:96 * 54]) if value > 200]
            self.assertGreater(len(xs), 10)
            centers.append(sum(xs) / len(xs))
        self.assertTrue(all(b > a + 3 for a, b in zip(centers, centers[1:])), centers)
        for segment in run.segments:
            data = segment.render.destination.read_bytes()
            self.assertEqual(segment.render.bytes, len(data))
            self.assertEqual(segment.render.digest, hashlib.sha256(data).hexdigest())
            self.assertFalse(segment.render.certified)
        self.assertEqual(json.loads(json.dumps(run.result.as_dict()))["schema"], "fmn.subdivided-render")
        self.assert_released(scene)

    def test_subdivision_is_byte_equivalent_to_continuous_recording(self):
        def exercise(scene, square):
            scene.play(square.animate.shift(m.RIGHT), run_time=.125, rate_func=m.linear)
            scene.wait(.125)
            scene.play(square.animate.shift(m.LEFT), run_time=.125, rate_func=m.linear)
        baseline = m.Scene()
        square = white_square()
        baseline.add(square)
        with record_scene(baseline, self.root / "whole.y4m", resolution=(96, 54), threads=1):
            exercise(baseline, square)
        expected = payloads(self.root / "whole.y4m")
        for threads in (1, 4):
            scene = m.Scene()
            square = white_square()
            scene.add(square)
            observed = []
            square.add_updater(lambda mob, dt: observed.append(dt))
            with self.session(scene, f"split-{threads}", threads=threads) as run:
                exercise(scene, square)
            frames = [frame for segment in run.segments for frame in payloads(segment.render.destination)]
            self.assertEqual(frames, expected)
            self.assertEqual(m._portal_scene_clock(scene), (30, 12))
            self.assertTrue(observed)
            self.assertTrue(all(np.isfinite(dt) for dt in observed))

    def test_supported_native_formats_and_counts(self):
        for format in ("png_sequence", "gif", "y4m"):
            with self.subTest(format=format):
                scene = m.Scene()
                scene.add(white_square())
                with self.session(scene, format, format=format) as run:
                    scene.wait(.125)
                    scene.wait(.125)
                self.assertEqual([s.render.frame_count for s in run.segments], [4, 4])
                for segment in run.segments:
                    path = segment.render.destination
                    if format == "png_sequence":
                        frames = sorted(path.glob("*.png"))
                        self.assertEqual(len(frames), 4)
                        self.assertTrue(all(p.read_bytes().startswith(b"\x89PNG\r\n\x1a\n") for p in frames))
                    elif format == "gif":
                        self.assertTrue(path.read_bytes().startswith(b"GIF89a"))
                    else:
                        self.assertEqual(len(payloads(path)), 4)

    def test_source_range_preroll_and_exclusive_end(self):
        class Motion(m.Scene):
            def setup(self):
                self.calls = ["setup"]
            def construct(self):
                self.calls.append("construct")
                self.square = white_square()
                self.add(self.square)
                for _ in range(3):
                    self.play(self.square.animate.shift(m.RIGHT), run_time=.125, rate_func=m.linear)
                self.reached_tail = True
            def tear_down(self):
                self.calls.append("tear_down")
        scene = Motion(start_at_animation_number=1, end_at_animation_number=2)
        with self.session(scene) as run:
            try:
                scene.run()
            except m.EndScene:
                pass
        self.assertEqual([s.play_index for s in run.segments], [1])
        self.assertEqual(scene.calls, ["setup", "construct", "tear_down"])
        self.assertFalse(hasattr(scene, "reached_tail"))
        self.assertEqual(run.result.frame_count, 4)
        np.testing.assert_allclose(scene.square.get_center(), [2, 0, 0], atol=1e-6)
        self.assertEqual(sorted(p.name for p in run.destination.iterdir()), ["segment-000001.y4m"])

    def test_empty_skip_nested_group_and_zero_duration(self):
        scene = m.Scene()
        square = white_square()
        scene.add(square)
        with self.session(scene) as run:
            scene.play()
            with scene.temp_skip():
                scene.wait(.125)
            scene.play(m.AnimationGroup(m.Transform(square, square.copy().shift(m.RIGHT))), run_time=.125)
            scene.wait(0)
        self.assertEqual([s.play_index for s in run.segments], [1, 2])
        self.assertEqual([s.render.frame_count for s in run.segments], [4, 1])
        self.assertEqual(run.segments[-1].start_frame, run.segments[-1].end_frame)

    def test_failure_retains_completed_clip_and_primary_exception(self):
        scene = m.Scene()
        square = white_square()
        scene.add(square)
        error = ValueError("authored failure")
        class Broken(m.Animation):
            def interpolate_mobject(self, alpha):
                if alpha > 0:
                    raise error
        run = self.session(scene)
        try:
            with run:
                scene.wait(.125)
                scene.play(Broken(square), run_time=.125)
        except ValueError as caught:
            self.assertIs(caught, error)
            traceback.clear_frames(caught.__traceback__)
            caught.__traceback__ = None
        else:
            self.fail("failure was swallowed")
        self.assertIsNone(run.result)
        self.assertTrue(run.artifact_published)
        self.assertEqual(len(run.partial_result.segments), 1)
        self.assertFalse(run.partial_result.completed)
        self.assertEqual(len(payloads(run.segments[0].render.destination)), 4)
        self.assertFalse((run.destination / "segment-000001.y4m").exists())
        self.assertFalse(square._is_updating_suspended())
        self.assert_released(scene)
        scene.wait(.125)

    def test_caught_failure_can_retry_same_segment(self):
        scene = m.Scene()
        scene.add(white_square())
        with self.session(scene) as run:
            with self.assertRaises(ValueError):
                scene.wait(-1)
            scene.wait(.125)
        self.assertEqual([s.play_index for s in run.segments], [0])
        self.assertEqual(run.result.frame_count, 4)

    def test_abort_preserves_completed_output_without_claiming_success(self):
        scene = m.Scene()
        scene.add(white_square())
        with self.session(scene) as run:
            scene.wait(.125)
            run.abort()
            run.abort()
            scene.wait(.125)
        self.assertIsNone(run.result)
        self.assertEqual(len(run.segments), 1)
        self.assert_released(scene)

    def test_existing_directory_and_child_cannot_be_clobbered(self):
        scene = m.Scene()
        self.root.joinpath("clips").mkdir()
        marker = self.root / "clips" / "keep"
        marker.write_bytes(b"existing data")
        with self.assertRaises(FileExistsError):
            with self.session(scene):
                self.fail("existing directory admitted")
        self.assertEqual(marker.read_bytes(), b"existing data")
        self.assert_released(scene)
        run = self.session(scene, "race")
        with run:
            child = run.destination / "segment-000000.y4m"
            child.write_bytes(b"another publisher")
            with self.assertRaises((FileExistsError, RuntimeError)):
                scene.wait(.125)
            self.assertEqual(child.read_bytes(), b"another publisher")
        self.assertEqual(run.result.frame_count, 0)
        self.assertEqual(m._portal_scene_clock(scene), (30, 0))

    def test_nested_owners_are_not_stolen(self):
        scene = m.Scene()
        with record_scene(scene, self.root / "outer.y4m", resolution=(96, 54)):
            with self.assertRaises(RuntimeError):
                with self.session(scene):
                    self.fail("nested subdivision admitted")
            scene.wait(.125)
        with self.session(scene, "owned") as run:
            with self.assertRaises(RuntimeError):
                with self.session(scene, "nested"):
                    self.fail("nested collection admitted")
            scene.wait(.125)
        self.assertEqual(run.result.frame_count, 4)
        self.assertEqual(len(payloads(self.root / "outer.y4m")), 4)
        self.assert_released(scene)

    def test_reentrant_play_refused_without_cancelling_outer_clip(self):
        scene = m.Scene()
        square = white_square()
        scene.add(square)
        attempts = []
        def updater(mob, dt):
            if dt:
                with self.assertRaises(RuntimeError):
                    scene.wait(.125)
                attempts.append(dt)
        square.add_updater(updater)
        with self.session(scene) as run:
            scene.wait(.125)
        self.assertEqual(run.result.frame_count, 4)
        self.assertEqual(len(attempts), 4)
        self.assert_released(scene)

    def test_resource_and_format_admission_precedes_filesystem(self):
        scene = m.Scene()
        for options in ({"fps": 12}, {"format": "png"}, {"format": "svg"},
                        {"max_segments": 0}, {"max_segments": True}, {"threads": 0}):
            with self.subTest(options=options):
                with self.assertRaises((ValueError, TypeError)):
                    self.session(scene, **options)
        self.assertEqual(list(self.root.iterdir()), [])
        self.assertEqual(m._portal_scene_clock(scene), (30, 0))

    def test_segment_budget_preserves_prior_receipts(self):
        scene = m.Scene()
        run = self.session(scene, max_segments=1)
        with self.assertRaisesRegex(ValueError, "budget"):
            with run:
                scene.wait(.125)
                scene.wait(.125)
        self.assertEqual(len(run.partial_result.segments), 1)
        self.assertEqual(m._portal_scene_clock(scene), (30, 4))
        self.assert_released(scene)

    def test_writer_subdivision_flag_uses_owned_path_only(self):
        scene = m.Scene(file_writer_config={"subdivide_output": True})
        with self.assertRaises(m._CapabilityError):
            record_scene(scene, self.root / "ordinary.y4m")
        with self.session(scene) as run:
            scene.wait(.125)
        self.assertEqual(run.result.frame_count, 4)
        self.assertTrue(scene.file_writer.subdivide_output)

    def test_post_publication_receipt_failure_is_visible(self):
        scene = m.Scene()
        finish = scene._finish_render
        def broken_receipt():
            finish()  # Publish the real native artifact, then inject a bad receipt.
            return ()
        scene._finish_render = broken_receipt
        run = self.session(scene)
        with self.assertRaises(ValueError):
            with run:
                scene.wait(.125)
        self.assertTrue(run.artifact_published)
        self.assertEqual(run.segments, ())
        self.assertIsNone(run.result)
        paths = run.partial_result.unreceipted_artifacts
        self.assertEqual(paths, (run.destination / "segment-000000.y4m",))
        self.assertEqual(len(payloads(paths[0])), 4)
        self.assert_released(scene)

    def test_broken_exception_notes_do_not_replace_authored_failure(self):
        class AuthoredError(ValueError):
            def add_note(self, note):
                raise RuntimeError("broken annotation")
        scene = m.Scene()
        error = AuthoredError("primary")
        try:
            with self.session(scene):
                raise error
        except AuthoredError as caught:
            self.assertIs(caught, error)
            traceback.clear_frames(caught.__traceback__)
            caught.__traceback__ = None
        else:
            self.fail("primary exception replaced")
        self.assert_released(scene)

    def test_legacy_partial_movie_flags_delegate_only_to_an_owned_native_clip(self):
        scene = m.Scene(file_writer_config={"subdivide_output": True, "write_to_movie": True})
        writer = scene.file_writer
        with self.assertRaises(NotImplementedError):
            writer.begin_animation()
        with self.session(scene) as run:
            scene.wait(.125)
            scene.wait(.125)
        self.assertEqual([s.render.frame_count for s in run.segments], [4, 4])
        self.assertTrue(writer.subdivide_output and writer.write_to_movie)
        with self.assertRaises(NotImplementedError):
            writer.end_animation()
        self.assert_released(scene)

    def test_authored_writer_overrides_keep_one_begin_and_end_per_clip(self):
        calls = []
        class Writer(m.SceneFileWriter):
            def begin_animation(self):
                calls.append("begin")
                return super().begin_animation()
            def end_animation(self):
                calls.append("end")
                return super().end_animation()
        scene = m.Scene()
        scene.file_writer = Writer(scene, subdivide_output=True, write_to_movie=True)
        with self.session(scene) as run:
            scene.wait(.125)
            scene.wait(.125)
        self.assertEqual(calls, ["begin", "end", "begin", "end"])
        self.assertEqual(run.result.frame_count, 8)


    def test_completed_clip_budget_allows_empty_and_skipped_calls(self):
        scene = m.Scene()
        with self.session(scene, max_segments=1) as run:
            scene.wait(.125)
            scene.play()
            with scene.temp_skip():
                scene.wait(.125)
        self.assertTrue(run.result.completed)
        self.assertEqual(run.result.frame_count, 4)
        self.assertEqual([s.play_index for s in run.segments], [0])
        self.assertEqual(m._portal_scene_clock(scene), (30, 8))
        self.assert_released(scene)

    def test_budget_does_not_hide_exclusive_source_range_completion(self):
        class Motion(m.Scene):
            def construct(self):
                self.wait(.125)
                self.wait(.125)
                self.tail = True
        scene = Motion(start_at_animation_number=0, end_at_animation_number=1)
        with self.session(scene, max_segments=1) as run:
            try:
                scene.run()
            except m.EndScene:
                pass
        self.assertTrue(run.result.completed)
        self.assertEqual(run.result.frame_count, 4)
        self.assertFalse(hasattr(scene, "tail"))
        self.assert_released(scene)

    def test_idle_collection_refuses_live_and_fresh_output_owners(self):
        scene = m.Scene()
        with self.session(scene) as run:
            for factory in (record_scene, render_session):
                destination = self.root / (factory.__name__ + ".y4m")
                with self.assertRaisesRegex(RuntimeError, "subdivision output owner"):
                    with factory(scene, destination, resolution=(96, 54), threads=1):
                        self.fail("another output owner entered an idle collection")
                self.assertFalse(destination.exists())
                self.assertEqual(m._portal_scene_clock(scene), (30, 0))
            scene.wait(.125)
        self.assertEqual(run.result.frame_count, 4)
        self.assert_released(scene)


suite = unittest.defaultTestLoader.loadTestsFromTestCase(SubdivisionTests)
result = unittest.TextTestRunner(verbosity=2).run(suite)
assert result.wasSuccessful(), "native subdivided recording acceptance failed"
