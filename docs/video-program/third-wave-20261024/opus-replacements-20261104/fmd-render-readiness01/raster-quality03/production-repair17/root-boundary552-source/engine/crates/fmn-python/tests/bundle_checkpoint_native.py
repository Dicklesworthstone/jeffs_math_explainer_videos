"""Real native bundle publication and hash-verified batch/console recovery.

Uses the installed extension. No engine, receipt, parser or publication boundary
is replaced. Only the authored scene's fail/retry input changes between attempts.
Evidence directories persist for inspection; successful artifacts are not erased.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import manimlib as m
from fmn_python import BatchRenderError, RenderJob, export_bundle, render_scenes
from fmn_python.batch_checkpoint import BatchCheckpoint

CALLS = []
FAILURE = None
CAMERA = False


class First(m.Scene):
    def __init__(self, **kwargs):
        CALLS.append((type(self).__name__, "construct"))
        super().__init__(**kwargs)

    def construct(self):
        CALLS.append((type(self).__name__, "run"))
        if CAMERA:
            self.frame.rotate(0.3, axis=m.RIGHT).scale(0.7)
            self.camera.background_color = "#152536"
            self.add(m.Cube(side_length=1.2, color=m.YELLOW))
            self.play(m.Rotate(self.frame, angle=0.2, axis=m.UP), run_time=0.25)
        else:
            square = m.Square(side_length=0.5, fill_opacity=1, stroke_width=0)
            self.add(square)
            self.play(square.animate.shift(m.RIGHT), run_time=0.25, rate_func=m.linear)


class Retry(First):
    def construct(self):
        super().construct()
        if FAILURE is not None:
            raise FAILURE


class Last(First):
    pass


def identity(path):
    info = path.stat()
    return info.st_ino, info.st_mtime_ns, info.st_size, hashlib.sha256(path.read_bytes()).hexdigest()


class BundleCheckpointNative(unittest.TestCase):
    def setUp(self):
        global FAILURE, CAMERA
        FAILURE, CAMERA = None, False
        CALLS.clear()
        self.root = Path(tempfile.mkdtemp(prefix="fmn-native-bundle-resume-"))
        self.output = self.root / "output"
        self.progress = self.root / "progress.json"
        self.jobs = [First, Retry, Last]
        self.assertTrue(callable(getattr(m, "_portal_begin_camera_bundle", None)))

    def run_batch(self, **options):
        defaults = dict(format="fmtl", bundle_camera=CAMERA, resolution=(96, 54), fps=8,
                        checkpoint=self.progress, resume_key="native-scene-inputs-v1")
        defaults.update(options)
        return render_scenes(self.jobs, self.output, **defaults)

    def test_failed_middle_camera_and_planar_resume_keep_native_receipts_and_bytes(self):
        global FAILURE, CAMERA
        for camera in (False, True):
            with self.subTest(camera=camera):
                CAMERA = camera
                self.output = self.root / str(camera)
                self.progress = self.root / f"progress-{camera}.json"
                FAILURE = ValueError("after real frame capture")
                with self.assertRaises(BatchRenderError) as caught:
                    self.run_batch()
                partial = caught.exception.result
                self.assertEqual([r.status for r in partial.outcomes], ["succeeded", "failed", "not_run"])
                first = partial.outcomes[0]
                before = identity(first.destination)
                FAILURE = None
                CALLS.clear()
                report = self.run_batch(resume=True)
                self.assertTrue(report.ok)
                self.assertEqual(CALLS, [("Retry", "construct"), ("Retry", "run"),
                                        ("Last", "construct"), ("Last", "run")])
                self.assertEqual(report.outcomes[0].result.as_dict(), first.result.as_dict())
                self.assertEqual(identity(first.destination), before)
                reference = self.root / f"single-{camera}.fmtl"
                export_bundle(First, reference, camera=camera, resolution=(96, 54), fps=8)
                self.assertEqual(first.destination.read_bytes(), reference.read_bytes())
                CALLS.clear()
                self.assertEqual(self.run_batch(resume=True).as_dict(), report.as_dict())
                self.assertEqual(CALLS, [])
                self.assertFalse(report.as_dict()["certified"])
                self.assertFalse(report.all_scenes_certified)

    def test_cancellation_after_capture_is_resumable_but_never_continues(self):
        global FAILURE, CAMERA
        CAMERA = True
        for failure in (KeyboardInterrupt(), SystemExit(0)):
            with self.subTest(failure=type(failure).__name__):
                self.output = self.root / type(failure).__name__
                self.progress = self.output.with_suffix(".json")
                FAILURE = failure
                with self.assertRaises(type(failure)) as caught:
                    self.run_batch(continue_on_error=True)
                report = caught.exception.render_batch_result
                self.assertEqual([r.status for r in report.outcomes], ["succeeded", "cancelled", "not_run"])
                self.assertFalse(report.outcomes[1].destination.exists())
                self.assertFalse(report.outcomes[2].destination.exists())
                before = identity(report.outcomes[0].destination)
                FAILURE = None
                CALLS.clear()
                self.assertTrue(self.run_batch(resume=True).ok)
                self.assertNotIn(("First", "construct"), CALLS)
                self.assertEqual(identity(report.outcomes[0].destination), before)

    def test_keep_going_and_observer_errors_record_all_completed_bundles(self):
        global FAILURE
        FAILURE = RuntimeError("retry")
        partial = self.run_batch(continue_on_error=True)
        self.assertEqual([r.status for r in partial.outcomes], ["succeeded", "failed", "succeeded"])
        saved = {r.name: identity(r.destination) for r in partial.outcomes if r.result is not None}
        FAILURE = None
        CALLS.clear()
        self.assertTrue(self.run_batch(resume=True).ok)
        self.assertEqual(CALLS, [("Retry", "construct"), ("Retry", "run")])
        for name, expected in saved.items():
            self.assertEqual(identity(self.output / (name + ".fmtl")), expected)
        self.output = self.root / "observer"
        self.progress = self.root / "observer.json"
        def observer(row):
            document = json.loads(self.progress.read_text())
            self.assertEqual(document["outcomes"][0]["status"], "succeeded")
            raise LookupError("stop after atomic publication")
        with self.assertRaises(LookupError):
            self.run_batch(on_result=observer)
        CALLS.clear()
        self.assertTrue(self.run_batch(resume=True).ok)
        self.assertNotIn(("First", "construct"), CALLS)

    def test_integrity_failures_and_option_changes_do_not_execute_scenes(self):
        global CAMERA
        CAMERA = True
        report = self.run_batch()
        original_journal = self.progress.read_bytes()
        CALLS.clear()
        for changes in ({"bundle_camera": False}, {"fps": 24}, {"resolution": (128, 72)},
                        {"bundle_limits": {"max_frames": 100}}, {"resume_key": "different"}):
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                self.run_batch(resume=True, **changes)
            self.assertEqual(CALLS, [])
            self.assertEqual(self.progress.read_bytes(), original_journal)
        final = report.outcomes[-1].destination
        final.write_bytes(b"damaged native output")
        with self.assertRaisesRegex(ValueError, "modified"):
            self.run_batch(resume=True)
        self.assertEqual(CALLS, [])
        self.assertEqual(final.read_bytes(), b"damaged native output")
        self.assertEqual(self.progress.read_bytes(), original_journal)

    def test_native_exact_byte_budget_survives_reuse_and_plan_change_refuses(self):
        global CAMERA
        CAMERA = True
        reference = self.root / "reference.fmtl"
        export_bundle(First, reference, camera=True, resolution=(96, 54), fps=8)
        cap = reference.stat().st_size
        limits = {"max_output_bytes": cap}
        first = self.run_batch(bundle_limits=limits)
        self.assertTrue(first.ok)
        CALLS.clear()
        again = self.run_batch(resume=True, bundle_limits=limits)
        self.assertEqual(first.as_dict(), again.as_dict())
        self.assertEqual(CALLS, [])
        with self.assertRaises(ValueError):
            self.run_batch(resume=True, bundle_limits={"max_output_bytes": cap - 1})
        self.assertEqual(CALLS, [])
        self.output = self.root / "too-small"
        self.progress = self.root / "too-small.json"
        refused = self.run_batch(continue_on_error=True, bundle_limits={"max_output_bytes": cap - 1})
        self.assertEqual(refused.counts["failed"], 3)
        self.assertFalse(any(r.destination.exists() for r in refused.outcomes))

    def test_kernel_lock_refuses_second_writer_before_native_construction(self):
        with BatchCheckpoint(self.progress, resume=False, key="native-scene-inputs-v1"):
            with self.assertRaises(OSError):
                self.run_batch()
        self.assertEqual(CALLS, [])
        self.assertFalse(self.output.exists())
        self.assertTrue(self.run_batch().ok)

    def test_actual_console_process_retries_failed_scene_and_reuses_complete_batch(self):
        source, marker, ready = self.root / "scenes.py", self.root / "calls.txt", self.root / "ready"
        source.write_text(f'''from manimlib import *
from pathlib import Path
marker = Path({str(marker)!r})
ready = Path({str(ready)!r})
class A(Scene):
    def __init__(self):
        with marker.open("a") as stream: stream.write("A\\n")
        super().__init__()
    def construct(self):
        self.frame.rotate(0.3, axis=RIGHT).scale(0.6)
        self.add(Cube(side_length=1.2))
        self.wait(0.25)
class B(A):
    def __init__(self):
        with marker.open("a") as stream: stream.write("B\\n")
        Scene.__init__(self)
    def construct(self):
        super().construct()
        if not ready.exists(): raise RuntimeError("retry me")
''')
        arguments = [sys.executable, "-m", "fmn_python", "--robot", str(source), "--write_all",
                     "--format=fmtl", "--bundle-camera", "--resolution=96x54", "--fps=8",
                     "--video_dir", str(self.output), "--checkpoint=" + str(self.progress),
                     "--resume-key=console-fixture-v1"]
        def invoke(extra=()):
            result = subprocess.run([*arguments, *extra], cwd=self.root, capture_output=True,
                                    text=True, timeout=120, stdin=subprocess.DEVNULL)
            self.assertEqual(len(result.stdout.splitlines()), 1, (result.stdout, result.stderr))
            return result.returncode, json.loads(result.stdout)
        code, first = invoke()
        self.assertEqual(code, 5, first)
        self.assertEqual(first["batch"]["counts"]["succeeded"], 1)
        before = identity(self.output / "A.fmtl")
        ready.write_text("retry only B")
        code, second = invoke(["--resume"])
        self.assertEqual(code, 0, second)
        self.assertEqual(marker.read_text().splitlines(), ["A", "B", "B"])
        self.assertEqual(identity(self.output / "A.fmtl"), before)
        code, third = invoke(["--resume"])
        self.assertEqual(code, 0, third)
        self.assertEqual(second, third)
        self.assertEqual(marker.read_text().splitlines(), ["A", "B", "B"])
        (self.output / "B.fmtl").write_bytes(b"corrupted")
        code, bad = invoke(["--resume"])
        self.assertEqual(code, 2, bad)
        self.assertIn("modified", bad["message"])
        self.assertEqual(marker.read_text().splitlines(), ["A", "B", "B"])


if __name__ == "__main__":
    unittest.main()
