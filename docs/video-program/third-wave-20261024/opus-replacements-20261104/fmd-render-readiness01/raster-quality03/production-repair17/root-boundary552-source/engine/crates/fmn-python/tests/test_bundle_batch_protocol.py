"""Batch planning and ownership with an explicit native bundle boundary double.

Published marker bytes are NOT FMTL. bundle_batching.py exercises the real
native recorder and canonical artifact with the same public batching API.
"""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

import test_batch_rendering_protocol as existing
from fmn_python import BatchRenderError, RenderJob, render_scenes


class CapabilityError(RuntimeError):
    pass


class BundleBatchProtocol(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="bundle-batch-protocol-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        existing.Scene.trace, existing.Scene.instances = [], []
        self.starts = []
        self.native = types.SimpleNamespace(Scene=existing.Scene, EndScene=existing.EndScene,
                                            _CapabilityError=CapabilityError)
        self.native._portal_begin_bundle = lambda scene, *args: self.begin(False, scene, *args)
        self.native._portal_begin_camera_bundle = lambda scene, *args: self.begin(True, scene, *args)
        self.native._portal_bundle_segment = lambda *args: None
        self.native._portal_finish_bundle = self.finish
        self.modules = patch.dict(sys.modules, {"manimlib": self.native})
        self.modules.start()
        self.addCleanup(self.modules.stop)

    def begin(self, camera, scene, path, width, height, fps, seed, *limits):
        if scene.active or scene.used:
            raise RuntimeError("scene already used or owned")
        self.starts.append((type(scene).__name__, camera, path, width, height, fps, seed, limits))
        scene.bundle_request = (path, camera, width, height, fps, seed)
        scene.active = True
        scene.events.append("bundle-begin")

    def finish(self, scene):
        if scene.finish_error is not None:
            raise scene.finish_error
        path, camera, width, height, fps, seed = scene.bundle_request
        payload = f"MODELED bundle {camera} {width}x{height} {fps} {seed}".encode()
        destination = Path(path)
        destination.parent.mkdir(parents=True, exist_ok=True)
        with destination.open("xb") as stream:
            stream.write(payload)
        scene.active, scene.used = False, True
        scene.events.append("bundle-finish")
        return path, 3, 2, len(payload), hashlib.sha256(payload).hexdigest()

    def batch(self, scenes, **options):
        return render_scenes(scenes, self.root / "output", format="fmtl", **options)

    def test_ordered_camera_jobs_keep_independent_native_receipts(self):
        report = self.batch({"Second": existing.Second, "First": existing.First},
                            bundle_camera=True, resolution=(96, 54), fps=24)
        self.assertTrue(report.ok)
        self.assertFalse(report.all_scenes_certified)
        self.assertEqual([r.name for r in report.outcomes], ["Second", "First"])
        for row in report.outcomes:
            self.assertEqual(row.destination, self.root / "output" / (row.name + ".fmtl"))
            self.assertTrue(row.result.camera_track)
            self.assertEqual(row.result.digest, hashlib.sha256(row.destination.read_bytes()).hexdigest())
            self.assertNotIn("begin", next(s for s in existing.Scene.instances
                                          if type(s).__name__ == row.name).events)
        data = json.loads(json.dumps(report.as_dict()))
        self.assertEqual(data["execution"], "sequential")
        self.assertEqual(data["counts"]["succeeded"], 2)
        self.assertEqual(data["outcomes"][0]["result"]["fmtl_minor"], 1)
        self.assertEqual([(event, name) for event, name, _ in existing.Scene.trace],
                         [("construct", "Second"), ("run", "Second"),
                          ("construct", "First"), ("run", "First")])

    def test_planar_batch_does_not_require_camera_entry_or_change_receipt(self):
        del self.native._portal_begin_camera_bundle
        report = self.batch([existing.First, existing.Second])
        self.assertTrue(report.ok)
        self.assertTrue(all(not start[1] for start in self.starts))
        self.assertNotIn("fmtl_minor", report.outcomes[0].result.as_dict())
        self.assertNotIn("camera_track", report.outcomes[0].result.as_dict())

    def test_camera_entry_checked_before_consuming_jobs_or_constructing_source(self):
        del self.native._portal_begin_camera_bundle
        def scenes():
            self.fail("native capability must be checked before iterating jobs")
            yield existing.First
        with self.assertRaises(CapabilityError):
            self.batch(scenes(), bundle_camera=True)
        self.assertEqual(existing.Scene.instances, [])

    def test_constructor_kwargs_and_seed_are_per_job(self):
        report = self.batch([RenderJob("A", existing.First, {"random_seed": 3}),
                             RenderJob("B", existing.First, {"random_seed": 5})])
        self.assertTrue(report.ok)
        self.assertEqual([start[6] for start in self.starts], [3, 5])
        self.assertNotEqual(report.outcomes[0].result.digest, report.outcomes[1].result.digest)

    def test_all_destinations_are_checked_before_the_first_constructor(self):
        (self.root / "output").mkdir()
        occupied = self.root / "output" / "Second.fmtl"
        occupied.write_bytes(b"user data")
        with self.assertRaises(FileExistsError):
            self.batch([existing.First, existing.Second])
        self.assertEqual(existing.Scene.instances, [])
        self.assertEqual(self.starts, [])
        self.assertEqual(occupied.read_bytes(), b"user data")

    def test_name_collision_and_max_jobs_refuse_before_execution(self):
        for jobs, options in (([RenderJob("Same", existing.First), RenderJob("same", existing.Second)], {}),
                              ([existing.First, existing.Second], {"max_jobs": 1})):
            with self.subTest(options=options):
                with self.assertRaises(ValueError):
                    self.batch(jobs, **options)
        self.assertEqual(existing.Scene.instances, [])
        self.assertEqual(self.starts, [])

    def test_limits_are_validated_before_constructing_any_scene(self):
        for limits, error in (({"max_frames": 0}, ValueError),
                              ({"max_frames": True}, TypeError),
                              ({"max_capture_bytes": 256 * 1024 * 1024 + 1}, ValueError),
                              ({"max_output_bytes": -1}, ValueError),
                              ({"unknown": 10}, TypeError), ([], TypeError)):
            with self.subTest(limits=limits):
                with self.assertRaises(error):
                    self.batch([existing.First], bundle_limits=limits)
        self.assertEqual(existing.Scene.instances, [])

    def test_limits_are_frozen_before_authored_mutation_and_passed_per_scene(self):
        limits = {"max_frames": 7, "max_capture_bytes": 100_000, "max_output_bytes": 20_000}
        class Mutates(existing.Scene):
            def run(self):
                limits.update(max_frames=0, max_capture_bytes=0, max_output_bytes=0)
                super().run()
        self.batch([Mutates, existing.Second], bundle_limits=limits)
        self.assertEqual([start[7] for start in self.starts], [(7, 100_000, 20_000)] * 2)

    def test_pixel_and_nonportable_modes_refuse_before_construction(self):
        modes = ({"threads": 1}, {"animation_range": (0, 1)}, {"subdivide": True},
                 {"save_last_frame": True},
                 {"reproducible": True}, {"sources": {"scene.py": b"pass"}},
                 {"runtime_identities": {}}, {"_output_options": {"transparent": True}})
        for options in modes:
            with self.subTest(options=options):
                with self.assertRaises(CapabilityError):
                    self.batch([existing.First], **options)
        self.assertEqual(existing.Scene.instances, [])
        self.assertEqual(self.starts, [])

    def test_invalid_resolution_and_fps_refuse_before_constructor(self):
        for options in ({"resolution": (100_000, 100_000)}, {"fps": 241}, {"fps": 0}):
            with self.subTest(options=options):
                with self.assertRaises(ValueError):
                    self.batch([existing.First], **options)
        self.assertEqual(existing.Scene.instances, [])

    def test_bundle_options_are_not_silently_ignored_by_pixel_formats(self):
        for options in ({"bundle_camera": True}, {"bundle_limits": {}}, {"bundle_camera": 1}):
            with self.subTest(options=options):
                with self.assertRaises((ValueError, TypeError)):
                    render_scenes([existing.First], self.root / "png", format="png", **options)
        self.assertEqual(existing.Scene.instances, [])

    def test_fail_fast_preserves_first_artifact_and_does_not_construct_third(self):
        failure = ValueError("authored failure")
        class Broken(existing.Scene):
            def run(self):
                raise failure
        with self.assertRaises(BatchRenderError) as caught:
            self.batch([existing.First, Broken, existing.Second], bundle_camera=True)
        self.assertIs(caught.exception.__cause__, failure)
        report = caught.exception.result
        self.assertEqual([row.status for row in report.outcomes], ["succeeded", "failed", "not_run"])
        self.assertTrue(report.outcomes[0].destination.exists())
        self.assertFalse(report.outcomes[1].destination.exists())
        self.assertEqual([type(s).__name__ for s in existing.Scene.instances], ["First", "Broken"])
        self.assertFalse(existing.Scene.instances[1].active)
        self.assertNotIn("_fmn_owned_render_session", vars(existing.Scene.instances[1]))

    def test_continue_on_error_publishes_later_bundle_without_partial_failed_file(self):
        class Broken(existing.Scene):
            def run(self):
                raise ValueError("authored failure")
        report = self.batch([existing.First, Broken, existing.Second], continue_on_error=True)
        self.assertEqual([row.status for row in report.outcomes], ["succeeded", "failed", "succeeded"])
        self.assertEqual(report.counts, {"succeeded": 2, "failed": 1, "cancelled": 0, "not_run": 0})
        self.assertFalse(report.ok)
        self.assertTrue(report.outcomes[2].destination.exists())
        self.assertFalse(report.outcomes[1].destination.exists())

    def test_cancellation_always_stops_and_preserves_progress(self):
        for kind in (KeyboardInterrupt, SystemExit):
            with self.subTest(kind=kind):
                class Stop(existing.Scene):
                    def run(self):
                        raise kind()
                with self.assertRaises(kind) as caught:
                    render_scenes([existing.First, Stop, existing.Second], self.root / kind.__name__,
                                  format="fmtl", continue_on_error=True)
                report = caught.exception.render_batch_result
                self.assertEqual([row.status for row in report.outcomes], ["succeeded", "cancelled", "not_run"])
                self.assertTrue(report.outcomes[0].destination.exists())
                self.assertFalse(report.outcomes[1].destination.exists())

    def test_observer_failure_cannot_relabel_a_published_bundle(self):
        error = LookupError("observer failed")
        def observe(outcome):
            self.assertTrue(outcome.destination.exists())
            raise error
        with self.assertRaises(LookupError) as caught:
            self.batch([existing.First, existing.Second], on_result=observe)
        self.assertIs(caught.exception, error)
        report = error.render_batch_result
        self.assertEqual([row.status for row in report.outcomes], ["succeeded", "not_run"])
        self.assertTrue(report.outcomes[0].destination.exists())
        self.assertEqual(len(self.starts), 1)

    def test_later_destination_race_is_native_no_clobber_not_rollback(self):
        occupied = self.root / "output" / "Second.fmtl"
        def observe(row):
            if row.name == "First":
                occupied.write_bytes(b"concurrent output")
        with self.assertRaises(BatchRenderError) as caught:
            self.batch([existing.First, existing.Second], on_result=observe)
        self.assertEqual(occupied.read_bytes(), b"concurrent output")
        self.assertTrue(caught.exception.result.outcomes[0].destination.exists())
        self.assertEqual(caught.exception.result.outcomes[1].status, "failed")


if __name__ == "__main__":
    unittest.main()
