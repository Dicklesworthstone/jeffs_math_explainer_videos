"""Receipt recovery over the real batch/journal code, with a modeled native sink.

Marker payloads are not FMTL. The separate installed-wheel suite exercises real
native bundle capture and process-level CLI recovery without engine doubles.
"""
import copy
import json
from pathlib import Path
import unittest

import test_bundle_batch_protocol as fixtures
from fmn_python import BatchRenderError, RenderJob, render_scenes
from fmn_python.batch_checkpoint import BatchCheckpoint


class BundleCheckpointProtocol(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.BundleBatchProtocol()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.root = self.fixture.root
        self.progress = self.root / "progress.json"
        self.jobs = {"First": fixtures.existing.First, "Second": fixtures.existing.Second}

    def run_batch(self, **changes):
        options = dict(format="fmtl", bundle_camera=True, resolution=(96, 54), fps=8,
                       checkpoint=self.progress, resume_key="authored-inputs-v1")
        options.update(changes)
        return render_scenes(self.jobs, self.root / "output", **options)

    def document(self):
        return json.loads(self.progress.read_text())

    def assert_no_execution(self):
        self.assertEqual(self.fixture.starts, [])
        self.assertEqual(fixtures.existing.Scene.instances, [])

    def clear_execution(self):
        self.fixture.starts.clear()
        fixtures.existing.Scene.instances.clear()

    def test_completed_camera_and_planar_receipts_round_trip_without_constructors(self):
        for camera in (False, True):
            with self.subTest(camera=camera):
                self.progress = self.root / f"progress-{camera}.json"
                jobs = {f"First{camera}": fixtures.existing.First}
                self.jobs = jobs
                first = self.run_batch(bundle_camera=camera)
                self.clear_execution()
                second = self.run_batch(bundle_camera=camera, resume=True)
                self.assertEqual(first.as_dict(), second.as_dict())
                self.assertEqual(type(first.outcomes[0].result), type(second.outcomes[0].result))
                self.assertFalse(second.all_scenes_certified)
                self.assert_no_execution()

    def test_failure_and_cancellation_retry_only_incomplete_jobs(self):
        for kind in (ValueError, KeyboardInterrupt, SystemExit):
            with self.subTest(kind=kind):
                failure = [True]
                class Retry(fixtures.existing.Scene):
                    def run(self):
                        if failure[0]:
                            raise kind("retry me")
                self.progress = self.root / (kind.__name__ + ".json")
                self.jobs = {"First" + kind.__name__: fixtures.existing.First,
                             "Retry" + kind.__name__: Retry,
                             "Last" + kind.__name__: fixtures.existing.Second}
                with self.assertRaises(BatchRenderError if kind is ValueError else kind):
                    self.run_batch()
                self.assertEqual([r["status"] for r in self.document()["outcomes"]],
                                 ["succeeded", "failed" if kind is ValueError else "cancelled", "not_run"])
                failure[0] = False
                self.clear_execution()
                self.assertTrue(self.run_batch(resume=True).ok)
                self.assertEqual([r[0] for r in self.fixture.starts], ["Retry", "Second"])

    def test_changed_plan_including_capture_limits_refuses_before_constructors(self):
        self.run_batch()
        before = self.progress.read_bytes()
        self.clear_execution()
        for changes in ({"bundle_camera": False}, {"fps": 24}, {"resolution": (128, 72)},
                        {"bundle_limits": {"max_frames": 10}},
                        {"bundle_limits": {"max_capture_bytes": 100_000}},
                        {"bundle_limits": {"max_output_bytes": 100_000}},
                        {"resume_key": "changed-inputs"}):
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                self.run_batch(resume=True, **changes)
            self.assert_no_execution()
            self.assertEqual(self.progress.read_bytes(), before)

    def test_same_normalized_limits_and_kwargs_order_can_resume(self):
        self.jobs = [RenderJob("First", fixtures.existing.First, {"random_seed": 7, "data": [1, True]})]
        first = self.run_batch(bundle_limits={"max_frames": 9, "max_output_bytes": 2000})
        self.jobs = [RenderJob("First", fixtures.existing.First, {"data": [1, True], "random_seed": 7})]
        self.clear_execution()
        second = self.run_batch(resume=True, bundle_limits={"max_output_bytes": 2000, "max_frames": 9,
                                                           "max_capture_bytes": 256 * 1024 * 1024})
        self.assertEqual(first.as_dict(), second.as_dict())
        self.assert_no_execution()

    def test_corrupt_missing_and_symlink_artifacts_refuse_before_execution(self):
        self.run_batch()
        before = self.progress.read_bytes()
        path = self.root / "output" / "Second.fmtl"
        original = path.read_bytes()
        self.clear_execution()
        path.write_bytes(b"corrupted")
        with self.assertRaisesRegex(ValueError, "modified"):
            self.run_batch(resume=True)
        path.write_bytes(original)
        moved = path.with_suffix(".retained")
        path.rename(moved)
        with self.assertRaises(FileNotFoundError):
            self.run_batch(resume=True)
        path.symlink_to(moved)
        with self.assertRaisesRegex(ValueError, "symlink"):
            self.run_batch(resume=True)
        self.assert_no_execution()
        self.assertEqual(self.progress.read_bytes(), before)
        self.assertEqual(moved.read_bytes(), original)

    def test_invalid_bundle_receipts_are_not_promoted_into_success(self):
        self.run_batch()
        original = self.document()
        self.clear_execution()
        variants = ({"schema": "other"}, {"version": True}, {"format": "png"},
                    {"frame_count": True}, {"frame_count": -1}, {"segment_count": "2"},
                    {"bytes": True}, {"bytes": 0}, {"fps": 24}, {"fps": False},
                    {"replay_resolution": [128, 72]}, {"replay_resolution": [True, 54]},
                    {"sha256": "A" * 64}, {"sha256": "0" * 64},
                    {"certified_source": True}, {"certified": True},
                    {"camera_track": False}, {"camera_track": 1}, {"fmtl_minor": 0},
                    {"fmtl_minor": True}, {"unknown": "ignored?"})
        for change in variants:
            with self.subTest(change=change):
                document = copy.deepcopy(original)
                document["outcomes"][0]["result"].update(change)
                self.progress.write_text(json.dumps(document))
                before = self.progress.read_bytes()
                with self.assertRaises(ValueError):
                    self.run_batch(resume=True)
                self.assertEqual(self.progress.read_bytes(), before)
                self.assert_no_execution()
        for key in ("schema", "fps", "camera_track", "fmtl_minor", "certified_source"):
            with self.subTest(missing=key):
                document = copy.deepcopy(original)
                document["outcomes"][0]["result"].pop(key)
                self.progress.write_text(json.dumps(document))
                with self.assertRaises(ValueError):
                    self.run_batch(resume=True)
                self.assert_no_execution()

    def test_planar_receipt_rejects_camera_metadata_even_with_matching_file_hash(self):
        self.run_batch(bundle_camera=False)
        document = self.document()
        document["outcomes"][0]["result"].update(camera_track=True, fmtl_minor=1)
        self.progress.write_text(json.dumps(document))
        self.clear_execution()
        with self.assertRaisesRegex(ValueError, "camera"):
            self.run_batch(bundle_camera=False, resume=True)
        self.assert_no_execution()

    def test_checkpoint_is_published_before_observer_and_after_integrity_validation(self):
        def observe(row):
            self.assertEqual(self.document()["outcomes"][0]["status"], "succeeded")
            raise LookupError("observer stopped the batch")
        with self.assertRaises(LookupError):
            self.run_batch(on_result=observe)
        self.clear_execution()
        self.assertTrue(self.run_batch(resume=True).ok)
        self.assertEqual([r[0] for r in self.fixture.starts], ["Second"])

    def test_unrecorded_publication_is_not_silently_reused(self):
        self.run_batch()
        document = self.document()
        document["outcomes"][1].update(status="not_run", result=None)
        document["artifacts"].pop("Second")
        self.progress.write_text(json.dumps(document))
        self.clear_execution()
        with self.assertRaises(FileExistsError):
            self.run_batch(resume=True)
        self.assert_no_execution()

    def test_native_digest_mismatch_cannot_create_a_completed_journal_entry(self):
        finish = self.fixture.native._portal_finish_bundle
        def wrong_digest(scene):
            path, frames, segments, size, digest = finish(scene)
            return path, frames, segments, size, "0" * 64
        self.fixture.native._portal_finish_bundle = wrong_digest
        with self.assertRaisesRegex(ValueError, "native receipt") as caught:
            self.run_batch()
        self.assertEqual(caught.exception.render_batch_result.outcomes[0].status, "succeeded")
        self.assertEqual([row["status"] for row in self.document()["outcomes"]], ["not_run", "not_run"])
        artifact = self.root / "output/First.fmtl"
        self.assertTrue(artifact.exists(), "publication is not rolled back by journal failure")
        self.clear_execution()
        with self.assertRaises(FileExistsError):
            self.run_batch(resume=True)
        self.assert_no_execution()

    def test_native_counter_above_admitted_limit_is_not_journaled_as_complete(self):
        finish = self.fixture.native._portal_finish_bundle
        def over_limit(scene):
            path, frames, segments, size, digest = finish(scene)
            return path, 11, segments, size, digest
        self.fixture.native._portal_finish_bundle = over_limit
        with self.assertRaisesRegex(ValueError, "recording limits"):
            self.run_batch(bundle_limits={"max_frames": 10})
        self.assertEqual([row["status"] for row in self.document()["outcomes"]], ["not_run", "not_run"])
        self.assertEqual(len(self.fixture.starts), 1)
        self.assertTrue((self.root / "output/First.fmtl").exists())

    def test_exclusive_lock_and_journal_destination_conflicts_precede_constructors(self):
        with BatchCheckpoint(self.progress, resume=False, key="authored-inputs-v1"):
            with self.assertRaises(OSError):
                self.run_batch()
        for path in (self.root / "output/First.fmtl", self.root / "output/First.fmtl/nested.json"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                self.run_batch(checkpoint=path)
        self.assert_no_execution()


if __name__ == "__main__":
    unittest.main()
