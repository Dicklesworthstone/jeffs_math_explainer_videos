"""Protocol/filesystem regressions; these do not claim native renderer coverage."""
from __future__ import annotations

import hashlib
import importlib
import json
import pathlib
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

ROOT = pathlib.Path(__file__).resolve().parents[1] / "python" / "fmn_python"
PACKAGE = "_fmn_batch_checkpoint_tests"
package = types.ModuleType(PACKAGE)
package.__path__ = [str(ROOT)]
sys.modules[PACKAGE] = package
batch = importlib.import_module(PACKAGE + ".batch_rendering")
checkpoint_module = importlib.import_module(PACKAGE + ".batch_checkpoint")


class Scene:
    pass


class First(Scene):
    pass


class Second(Scene):
    pass


class Third(Scene):
    pass


class CheckpointTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = pathlib.Path(self.temporary.name)
        self.journal = self.root / "progress.json"
        self.output = self.root / "outputs"
        self.jobs = {"first": First, "second": Second, "third": Third}
        self.calls, self.failures = [], {}
        self.native = patch.dict(sys.modules, {"manimlib": types.SimpleNamespace(Scene=Scene)})
        self.native.start()
        self.addCleanup(self.native.stop)
        self.render = patch.object(batch, "render_scene", self.fake_render)
        self.render.start()
        self.addCleanup(self.render.stop)

    def fake_render(self, scene, destination, *, format, resolution, fps, threads, **kwargs):
        self.calls.append(scene)
        if scene in self.failures:
            raise self.failures[scene]
        content = (scene.__name__ + "-native-publication-protocol-fixture").encode()
        if format == "png_sequence":
            destination.mkdir(parents=True)
            for index in range(2):
                (destination / f"{index:06}.png").write_bytes(content + bytes([index]))
            digest, size = "0" * 64, 2 * (len(content) + 1)
        else:
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(content)
            digest, size = hashlib.sha256(content).hexdigest(), len(content)
        return batch.RenderResult(destination, format, resolution, fps, threads,
                                  "test-protocol-not-native", size, digest, 2, None, 7)

    def run_batch(self, **kwargs):
        options = dict(format="png", resolution=(96, 54), fps=8, threads=1,
                       checkpoint=self.journal, resume_key="scene-and-assets-v1")
        options.update(kwargs)
        return batch.render_scenes(self.jobs, self.output, **options)

    def document(self):
        return json.loads(self.journal.read_text())

    def test_fail_fast_resume_only_unfinished_jobs(self):
        self.failures[Second] = RuntimeError("authored failure")
        with self.assertRaises(batch.BatchRenderError) as caught:
            self.run_batch()
        self.assertEqual([item.status for item in caught.exception.result.outcomes],
                         ["succeeded", "failed", "not_run"])
        before = (self.output / "first.png").stat().st_mtime_ns
        self.failures.clear()
        self.calls.clear()
        result = self.run_batch(resume=True)
        self.assertTrue(result.ok)
        self.assertEqual(self.calls, [Second, Third])
        self.assertEqual(before, (self.output / "first.png").stat().st_mtime_ns)
        self.assertEqual(result.outcomes[0].result.as_dict(), self.document()["outcomes"][0]["result"])

    def test_complete_resume_does_not_execute_scenes(self):
        first = self.run_batch()
        self.calls.clear()
        second = self.run_batch(resume=True)
        self.assertEqual(self.calls, [])
        self.assertEqual(first.as_dict(), second.as_dict())
        self.assertFalse(second.as_dict()["certified"])

    def test_continue_after_error_retains_later_success(self):
        self.failures[Second] = ValueError("failure")
        self.assertFalse(self.run_batch(continue_on_error=True).ok)
        self.calls.clear()
        self.failures.clear()
        self.assertTrue(self.run_batch(resume=True).ok)
        self.assertEqual(self.calls, [Second])

    def test_keyboard_interrupt_is_journaled_and_propagated(self):
        original = KeyboardInterrupt("stop")
        self.failures[Second] = original
        with self.assertRaises(KeyboardInterrupt) as caught:
            self.run_batch(continue_on_error=True)
        self.assertIs(caught.exception, original)
        self.assertEqual(self.document()["outcomes"][1]["status"], "cancelled")
        self.assertEqual(original.render_batch_result.counts["succeeded"], 1)
        self.failures.clear()
        self.calls.clear()
        self.assertTrue(self.run_batch(resume=True).ok)
        self.assertEqual(self.calls, [Second, Third])

    def test_observer_failure_preserves_publication_in_checkpoint(self):
        def observer(outcome):
            self.assertEqual(self.document()["outcomes"][0]["status"], "succeeded")
            raise RuntimeError("observer failure")
        with self.assertRaisesRegex(RuntimeError, "observer failure") as caught:
            self.run_batch(on_result=observer)
        self.assertEqual(caught.exception.render_batch_result.counts["succeeded"], 1)
        self.calls.clear()
        self.assertTrue(self.run_batch(resume=True).ok)
        self.assertEqual(self.calls, [Second, Third])

    def test_changed_output_is_rejected_before_any_execution(self):
        self.run_batch()
        (self.output / "third.png").write_bytes(b"tampered")
        self.calls.clear()
        with self.assertRaisesRegex(ValueError, "modified"):
            self.run_batch(resume=True)
        self.assertEqual(self.calls, [])
        self.assertEqual((self.output / "third.png").read_bytes(), b"tampered")

    def test_missing_output_is_rejected_before_any_execution(self):
        self.run_batch()
        (self.output / "third.png").rename(self.output / "removed.png")
        self.calls.clear()
        with self.assertRaises(FileNotFoundError):
            self.run_batch(resume=True)
        self.assertEqual(self.calls, [])

    def test_sequence_recovery_and_added_frame_detection(self):
        first = self.run_batch(format="png_sequence")
        self.calls.clear()
        self.assertEqual(self.run_batch(format="png_sequence", resume=True).as_dict(), first.as_dict())
        self.assertEqual(self.calls, [])
        (self.output / "third/frames/extra.png").write_bytes(b"unexpected")
        with self.assertRaisesRegex(ValueError, "modified"):
            self.run_batch(format="png_sequence", resume=True)
        self.assertEqual(self.calls, [])

    def test_sequence_changed_frame_detection(self):
        self.run_batch(format="png_sequence")
        self.calls.clear()
        (self.output / "third/frames/000000.png").write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "modified"):
            self.run_batch(format="png_sequence", resume=True)
        self.assertEqual(self.calls, [])

    def test_plan_options_and_input_key_must_match(self):
        self.run_batch()
        self.calls.clear()
        for options in ({"fps": 12}, {"threads": 2}, {"resolution": (128, 72)},
                        {"resume_key": "different-inputs"}, {"animation_range": (1, 3)}):
            with self.subTest(options=options), self.assertRaises(ValueError):
                self.run_batch(resume=True, **options)
        self.assertEqual(self.calls, [])

    def test_constructor_kwargs_must_match(self):
        self.jobs = [batch.RenderJob("first", First, {"random_seed": 7})]
        self.run_batch()
        self.calls.clear()
        self.jobs = [batch.RenderJob("first", First, {"random_seed": 8})]
        with self.assertRaisesRegex(ValueError, "plan"):
            self.run_batch(resume=True)
        self.assertEqual(self.calls, [])

    def test_job_reordering_is_rejected(self):
        self.run_batch()
        self.calls.clear()
        self.jobs = dict(reversed(tuple(self.jobs.items())))
        with self.assertRaisesRegex(ValueError, "plan"):
            self.run_batch(resume=True)
        self.assertEqual(self.calls, [])

    def test_existing_journal_requires_explicit_resume(self):
        self.run_batch()
        before = self.journal.read_bytes()
        self.calls.clear()
        with self.assertRaises(FileExistsError):
            self.run_batch()
        self.assertEqual(self.journal.read_bytes(), before)
        self.assertEqual(self.calls, [])

    def test_resume_missing_journal_is_not_a_fresh_batch(self):
        with self.assertRaises(FileNotFoundError):
            self.run_batch(resume=True)
        self.assertEqual(self.calls, [])

    def test_unrecorded_publication_is_never_silently_adopted(self):
        self.failures[Second] = RuntimeError("failure")
        with self.assertRaises(batch.BatchRenderError):
            self.run_batch()
        path = self.output / "second.png"
        path.write_bytes(b"unrecorded native publication")
        self.calls.clear()
        with self.assertRaises(FileExistsError):
            self.run_batch(resume=True)
        self.assertEqual(self.calls, [])
        self.assertEqual(path.read_bytes(), b"unrecorded native publication")

    def test_checkpoint_cannot_overlap_output(self):
        for path in (self.output / "first.png", self.output / "first/frames/progress.json"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                self.run_batch(checkpoint=path, format="png_sequence" if "frames" in path.parts else "png")
        self.assertEqual(self.calls, [])

    def test_checkpoint_requires_key_and_boolean_resume(self):
        for options, kind in (({"resume_key": None}, ValueError), ({"resume": 1}, TypeError),
                              ({"checkpoint": None, "resume": True}, ValueError),
                              ({"checkpoint": None}, ValueError)):
            with self.subTest(options=options), self.assertRaises(kind):
                self.run_batch(**options)
        self.assertEqual(self.calls, [])

    def test_live_writer_lock_and_release(self):
        owner = checkpoint_module.BatchCheckpoint(self.journal, resume=False, key="v1")
        with owner:
            with self.assertRaises(OSError):
                with checkpoint_module.BatchCheckpoint(self.journal, resume=False, key="v1"):
                    self.fail("a second writer acquired a live lock")
        with checkpoint_module.BatchCheckpoint(self.journal, resume=False, key="v1"):
            pass

    def test_symlink_artifact_is_rejected(self):
        self.run_batch()
        original = self.output / "third.png"
        target = self.output / "saved.png"
        original.rename(target)
        original.symlink_to(target)
        self.calls.clear()
        with self.assertRaisesRegex(ValueError, "symlink"):
            self.run_batch(resume=True)
        self.assertEqual(self.calls, [])

    def test_invalid_json_and_receipts_release_lock(self):
        self.run_batch()
        original = self.journal.read_bytes()
        self.journal.write_text("{broken")
        self.calls.clear()
        with self.assertRaises(ValueError):
            self.run_batch(resume=True)
        self.journal.write_bytes(original)
        document = self.document()
        document["outcomes"][0]["result"]["digest"] = "f" * 64
        self.journal.write_text(json.dumps(document))
        with self.assertRaisesRegex(ValueError, "receipt"):
            self.run_batch(resume=True)
        self.journal.write_bytes(original)
        self.assertTrue(self.run_batch(resume=True).ok)
        self.assertEqual(self.calls, [])

    def test_checkpoint_reporting_failure_retains_actual_success(self):
        record = checkpoint_module.BatchCheckpoint.record
        def fail_after_publication(owner, outcomes):
            if any(item.status == "succeeded" for item in outcomes):
                raise OSError("journal device failed")
            return record(owner, outcomes)
        with patch.object(checkpoint_module.BatchCheckpoint, "record", fail_after_publication):
            with self.assertRaisesRegex(OSError, "device failed") as caught:
                self.run_batch()
        self.assertEqual(caught.exception.render_batch_result.counts["succeeded"], 1)
        self.assertTrue((self.output / "first.png").is_file())
        self.assertEqual(self.calls, [First])

    def test_plain_batches_are_unchanged(self):
        result = self.run_batch(checkpoint=None, resume_key=None)
        self.assertTrue(result.ok)
        self.assertFalse(self.journal.exists())
        self.assertFalse(self.journal.with_name(self.journal.name + ".lock").exists())
        with self.assertRaises(FileExistsError):
            self.run_batch(checkpoint=None, resume_key=None)

    def test_non_json_constructor_arguments_fail_before_execution(self):
        self.jobs = [batch.RenderJob("first", First, {"opaque": object()})]
        with self.assertRaises(TypeError):
            self.run_batch()
        self.assertEqual(self.calls, [])

    def test_atomic_replace_failure_keeps_previous_journal(self):
        self.run_batch()
        original = self.journal.read_bytes()
        with patch.object(checkpoint_module.os, "replace", side_effect=OSError("replace failure")):
            with self.assertRaisesRegex(OSError, "replace failure"):
                self.run_batch(resume=True)
        self.assertEqual(self.journal.read_bytes(), original)
        self.assertTrue(self.run_batch(resume=True).ok)


if __name__ == "__main__":
    unittest.main()
