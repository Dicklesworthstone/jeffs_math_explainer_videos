"""Actual native multi-scene FMTL capture through the existing batch executor.

No renderer, recorder, publication boundary, or Scene implementation is doubled.
This runs directly against an installed matching wheel; test artifacts remain.
"""
from pathlib import Path
import hashlib
import json
import tempfile
import unittest

import manimlib as m
from fmn_python import BatchRenderError, RenderJob, export_bundle, render_scenes


CONSTRUCTIONS = []
EXECUTIONS = []


class Planar(m.Scene):
    def __init__(self, *args, **kwargs):
        CONSTRUCTIONS.append(type(self).__name__)
        super().__init__(*args, **kwargs)

    def construct(self):
        EXECUTIONS.append(type(self).__name__)
        square = m.Square(side_length=0.5, fill_opacity=1, stroke_width=0, color=m.BLUE)
        self.add(square)
        self.play(square.animate.shift(m.RIGHT), run_time=0.25, rate_func=m.linear)


class Camera(m.Scene):
    def __init__(self, *args, **kwargs):
        CONSTRUCTIONS.append(type(self).__name__)
        super().__init__(*args, **kwargs)

    def construct(self):
        EXECUTIONS.append(type(self).__name__)
        self.frame.rotate(0.4, axis=m.RIGHT).scale(0.6)
        self.camera.background_color = "#152536"
        self.add(m.Cube(side_length=1.5, color=m.YELLOW))
        self.play(m.Rotate(self.frame, angle=0.6, axis=m.UP), run_time=0.25,
                  rate_func=lambda alpha: alpha * alpha)


class Broken(Camera):
    def construct(self):
        super().construct()
        raise ValueError("authored batch failure after capture")


class Stop(Camera):
    def construct(self):
        super().construct()
        raise KeyboardInterrupt()


class BundleBatching(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="fmn-native-bundle-batch-"))
        CONSTRUCTIONS.clear()
        EXECUTIONS.clear()
        self.assertTrue(callable(getattr(m, "_portal_begin_camera_bundle", None)))

    def batch(self, jobs, directory="batch", **options):
        return render_scenes(jobs, self.root / directory, format="fmtl",
                             resolution=(96, 54), fps=8, **options)

    def test_planar_and_camera_jobs_equal_independent_single_exports(self):
        for mode, cls in ((False, Planar), (True, Camera)):
            with self.subTest(camera=mode):
                CONSTRUCTIONS.clear()
                EXECUTIONS.clear()
                report = self.batch([RenderJob("Second", cls), RenderJob("First", cls)],
                                    directory=str(mode), bundle_camera=mode)
                self.assertTrue(report.ok)
                self.assertEqual(CONSTRUCTIONS, [cls.__name__] * 2)
                self.assertEqual(EXECUTIONS, [cls.__name__] * 2)
                single_path = self.root / f"single-{mode}.fmtl"
                single = export_bundle(cls, single_path, camera=mode, resolution=(96, 54), fps=8)
                expected = single_path.read_bytes()
                for outcome in report.outcomes:
                    self.assertEqual(outcome.destination.read_bytes(), expected)
                    self.assertEqual(outcome.result.digest, single.digest)
                    self.assertEqual(outcome.result.frame_count, 2)
                    self.assertEqual(outcome.result.camera_track, mode)
                self.assertEqual([r.name for r in report.outcomes], ["Second", "First"])
                data = json.loads(json.dumps(report.as_dict()))
                self.assertFalse(data["certified"])
                self.assertFalse(data["all_scenes_certified"])
                self.assertEqual(data["counts"]["succeeded"], 2)

    def test_failed_middle_scene_retains_previous_and_runs_later_when_requested(self):
        report = self.batch([Planar, Broken, Camera], bundle_camera=True, continue_on_error=True)
        self.assertEqual([row.status for row in report.outcomes], ["succeeded", "failed", "succeeded"])
        self.assertEqual(CONSTRUCTIONS, ["Planar", "Broken", "Camera"])
        self.assertIn("authored batch failure", report.outcomes[1].message)
        self.assertTrue(report.outcomes[0].destination.is_file())
        self.assertFalse(report.outcomes[1].destination.exists())
        self.assertTrue(report.outcomes[2].destination.is_file())

    def test_fail_fast_and_interrupt_stop_without_constructing_later_scene(self):
        for cls, error in ((Broken, BatchRenderError), (Stop, KeyboardInterrupt)):
            with self.subTest(scene=cls.__name__):
                CONSTRUCTIONS.clear()
                with self.assertRaises(error) as caught:
                    self.batch([Planar, cls, Camera], directory=cls.__name__, bundle_camera=True)
                report = (caught.exception.result if error is BatchRenderError
                          else caught.exception.render_batch_result)
                self.assertEqual(CONSTRUCTIONS, ["Planar", cls.__name__])
                self.assertEqual(report.outcomes[0].status, "succeeded")
                self.assertEqual(report.outcomes[1].status, "failed" if error is BatchRenderError else "cancelled")
                self.assertEqual(report.outcomes[2].status, "not_run")
                self.assertTrue(report.outcomes[0].destination.exists())
                self.assertFalse(report.outcomes[1].destination.exists())
                self.assertFalse(report.outcomes[2].destination.exists())

    def test_bounds_and_later_collision_are_admitted_before_any_constructor(self):
        occupied = self.root / "batch" / "Camera.fmtl"
        occupied.parent.mkdir()
        occupied.write_bytes(b"existing user artifact")
        with self.assertRaises(FileExistsError):
            self.batch([Planar, Camera], bundle_camera=True)
        self.assertEqual(CONSTRUCTIONS, [])
        self.assertEqual(occupied.read_bytes(), b"existing user artifact")
        for options, error in (({"bundle_limits": {"max_frames": 0}}, ValueError),
                               ({"bundle_limits": {"unknown": 1}}, TypeError),
                               ({"threads": 2}, m._CapabilityError)):
            with self.subTest(options=options):
                with self.assertRaises(error):
                    self.batch([Camera], directory="invalid", bundle_camera=True, **options)
                self.assertEqual(CONSTRUCTIONS, [])

    def test_exact_output_budget_is_per_scene_and_one_byte_less_refuses(self):
        reference = self.root / "reference.fmtl"
        export_bundle(Camera, reference, camera=True, resolution=(96, 54), fps=8)
        expected = reference.read_bytes()
        report = self.batch([RenderJob("A", Camera), RenderJob("B", Camera)], bundle_camera=True,
                            bundle_limits={"max_output_bytes": len(expected)})
        self.assertTrue(report.ok)
        self.assertEqual(sum(r.result.bytes for r in report.outcomes), 2 * len(expected))
        for row in report.outcomes:
            self.assertEqual(row.destination.read_bytes(), expected)
        refused = self.batch([RenderJob("A", Camera), RenderJob("B", Camera)], directory="small",
                             bundle_camera=True, continue_on_error=True,
                             bundle_limits={"max_output_bytes": len(expected) - 1})
        self.assertEqual(refused.counts["failed"], 2)
        for row in refused.outcomes:
            self.assertEqual(row.error_type, "RuntimeError")
            self.assertIn("output budget", row.message)
            self.assertFalse(row.destination.exists())
        self.assertEqual(hashlib.sha256(reference.read_bytes()).hexdigest(), report.outcomes[0].result.digest)

    def test_capture_limit_cannot_be_swallowed_or_poison_a_later_scene(self):
        class Swallowed(m.Scene):
            def construct(self):
                self.add(m.Square())
                try:
                    self.wait(0.5)
                except (ValueError, RuntimeError):
                    pass
        report = self.batch([RenderJob("EmptyBefore", m.Scene), Swallowed,
                             RenderJob("EmptyAfter", m.Scene)], bundle_camera=True,
                            continue_on_error=True, bundle_limits={"max_frames": 1})
        self.assertEqual([r.status for r in report.outcomes], ["succeeded", "failed", "succeeded"])
        self.assertFalse(report.outcomes[1].destination.exists())
        self.assertEqual(report.outcomes[0].destination.read_bytes(), report.outcomes[2].destination.read_bytes())

    def test_observer_failure_preserves_receipt_and_releases_native_owner(self):
        scene = Camera()
        error = LookupError("stop after publication")
        def observe(row):
            self.assertTrue(row.destination.exists())
            raise error
        with self.assertRaises(LookupError) as caught:
            self.batch([RenderJob("First", scene), Planar], bundle_camera=True, on_result=observe)
        self.assertIs(caught.exception, error)
        report = error.render_batch_result
        self.assertEqual([row.status for row in report.outcomes], ["succeeded", "not_run"])
        self.assertEqual(CONSTRUCTIONS, ["Camera"])
        self.assertNotIn("_fmn_owned_render_session", vars(scene))
        self.assertNotIn("_fmn_subdivision_session", vars(scene))
        self.assertTrue(report.outcomes[0].destination.is_file())


if __name__ == "__main__":
    unittest.main()
