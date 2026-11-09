"""Real native bundle batches through the shipping console and host process.

No scene, parser, native recorder or publisher is substituted. Independent
single-scene exports provide the byte oracle for each actual console artifact.
"""
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import manimlib as m
from fmn_python import __main__ as console, export_bundle
from fmn_python.scene_loading import SceneSource


SOURCE = '''import manimlib as m
print("bundle-batch-source-loaded")
class Zulu(m.Scene):
    def __init__(self):
        print("construct-Zulu")
        super().__init__()
    def construct(self):
        square = m.Square(side_length=0.5, fill_opacity=1, stroke_width=0, color=m.BLUE)
        self.add(square)
        self.play(square.animate.shift(m.RIGHT), run_time=0.25, rate_func=m.linear)
class Alpha(m.Scene):
    def __init__(self):
        print("construct-Alpha")
        super().__init__()
    def construct(self):
        self.frame.rotate(0.4, axis=m.RIGHT).scale(0.6)
        self.camera.background_color = "#152536"
        self.add(m.Cube(side_length=1.5, color=m.YELLOW))
        self.play(m.Rotate(self.frame, angle=0.6, axis=m.UP), run_time=0.25,
                  rate_func=lambda alpha: alpha * alpha)
'''


class BundleBatchConsole(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="fmn-native-bundle-batch-console-"))
        self.source = self.root / "scenes.py"
        self.source.write_text(SOURCE)
        self.directory = self.root / "output"
        self.assertTrue(callable(getattr(m, "_portal_begin_camera_bundle", None)))

    def arguments(self, *extra, camera=True, names=(), directory=None):
        return ["--robot", str(self.source), *names, "--format", "fmtl",
                "--resolution", "96x54", "--fps", "8", "--video_dir",
                str(self.directory if directory is None else directory),
                *(("--bundle-camera",) if camera else ()), *extra]

    def invoke(self, *extra, **options):
        out, err = io.StringIO(), io.StringIO()
        with (patch.object(sys, "argv", ["fmn-python", *self.arguments(*extra, **options)]),
              contextlib.redirect_stdout(out), contextlib.redirect_stderr(err)):
            code = console.main()
        self.assertEqual(len(out.getvalue().splitlines()), 1, out.getvalue())
        return code, json.loads(out.getvalue()), err.getvalue()

    def compare_independent(self, report, camera):
        with contextlib.redirect_stdout(io.StringIO()), SceneSource(self.source, m.Scene) as loaded:
            for row in report["batch"]["outcomes"]:
                self.assertEqual(row["status"], "succeeded")
                reference = self.root / ("single-" + row["name"] + ".fmtl")
                expected = export_bundle(loaded.scenes[row["name"]], reference,
                                         camera=camera, resolution=(96, 54), fps=8)
                actual = Path(row["destination"]).read_bytes()
                self.assertEqual(actual, reference.read_bytes())
                self.assertEqual(row["result"]["sha256"], expected.digest)
                self.assertEqual(row["result"]["sha256"], hashlib.sha256(actual).hexdigest())
                self.assertEqual(row["result"]["frame_count"], 2)
                self.assertEqual(row["result"].get("camera_track", False), camera)

    def test_actual_console_camera_batches_equal_independent_exports(self):
        code, report, err = self.invoke("--write_all")
        self.assertEqual(code, 0, (report, err))
        self.assertEqual([r["name"] for r in report["batch"]["outcomes"]], ["Alpha", "Zulu"])
        self.assertEqual(err.count("bundle-batch-source-loaded"), 1)
        self.assertEqual(err.count("construct-Alpha"), 1)
        self.assertEqual(err.count("construct-Zulu"), 1)
        self.assertEqual(report["batch"]["counts"]["succeeded"], 2)
        self.compare_independent(report, True)

    def test_planar_named_order_keeps_minor_zero_and_exact_destination_semantics(self):
        self.source.write_text(SOURCE[:SOURCE.index("class Alpha")] + "class Beta(Zulu):\n    pass\n")
        code, report, err = self.invoke(camera=False, names=("Zulu", "Beta"))
        self.assertEqual(code, 0, (report, err))
        self.assertEqual([r["name"] for r in report["batch"]["outcomes"]], ["Zulu", "Beta"])
        self.compare_independent(report, False)
        exact = self.root / "exact.fmtl"
        code, single, err = self.invoke(camera=False, names=("Zulu",), directory=exact)
        self.assertEqual(code, 0, (single, err))
        self.assertEqual(single["kind"], "bundle-export")
        self.assertEqual(single["destination"], str(exact))
        self.assertEqual(exact.read_bytes(), (self.directory / "Zulu.fmtl").read_bytes())

    def test_failure_recovery_partial_success_and_fail_fast(self):
        self.source.write_text(SOURCE + '\nclass Middle(Zulu):\n    def construct(self):\n        super().construct()\n        raise ValueError("failed after native capture")\n')
        for keep in (False, True):
            with self.subTest(keep_going=keep):
                code, report, err = self.invoke("--write_all", *(("--keep-going",) if keep else ()),
                                                 directory=self.directory / str(keep))
                self.assertEqual(code, 5)
                rows = report["batch"]["outcomes"]
                self.assertEqual([r["status"] for r in rows], ["succeeded", "failed", "succeeded" if keep else "not_run"])
                self.assertIn("failed after native capture", rows[1]["error"]["message"])
                self.assertTrue(Path(rows[0]["destination"]).is_file())
                self.assertFalse(Path(rows[1]["destination"]).exists())
                self.assertEqual(Path(rows[2]["destination"]).is_file(), keep)
                # The failed middle is also a Zulu subclass, hence one vs two.
                self.assertEqual(err.count("construct-Zulu"), 2 if keep else 1)
        # A new generation still succeeds after the failed generation's cleanup.
        code, report, err = self.invoke(names=("Zulu",), directory=self.root / "recovered.fmtl")
        self.assertEqual(code, 0, (report, err))

    def test_native_interrupt_releases_owner_and_preserves_progress(self):
        for failure, expected in (("KeyboardInterrupt()", 130), ("SystemExit(0)", 5)):
            with self.subTest(failure=failure):
                self.source.write_text(SOURCE + f'\nclass Middle(Zulu):\n    def construct(self):\n        super().construct()\n        raise {failure}\n')
                code, report, err = self.invoke("--write_all", "--keep-going",
                                                 directory=self.directory / str(expected))
                self.assertEqual(code, expected)
                self.assertEqual(report["kind"], "render-batch-interrupted")
                rows = report["batch"]["outcomes"]
                self.assertEqual([r["status"] for r in rows], ["succeeded", "cancelled", "not_run"])
                self.assertTrue(Path(rows[0]["destination"]).exists())
                self.assertFalse(Path(rows[1]["destination"]).exists())
                self.assertFalse(Path(rows[2]["destination"]).exists())
                self.assertEqual(err.count("construct-Zulu"), 1)

    def test_no_constructor_runs_when_a_later_artifact_is_occupied(self):
        self.directory.mkdir()
        protected = self.directory / "Zulu.fmtl"
        protected.write_bytes(b"existing user artifact")
        code, report, err = self.invoke("--write_all")
        self.assertEqual(code, 6, report)
        self.assertEqual(protected.read_bytes(), b"existing user artifact")
        self.assertFalse((self.directory / "Alpha.fmtl").exists())
        self.assertNotIn("construct-", err)

    def test_all_names_validated_before_construct_and_no_camera_is_silently_dropped(self):
        code, report, err = self.invoke(names=("Zulu", "Missing"))
        self.assertEqual(code, 2)
        self.assertNotIn("construct-", err)
        self.assertFalse(self.directory.exists())
        code, report, err = self.invoke("--write_all", "--keep-going", camera=False)
        self.assertEqual(code, 5)
        rows = report["batch"]["outcomes"]
        self.assertEqual([r["status"] for r in rows], ["failed", "succeeded"])
        self.assertIn("camera", rows[0]["error"]["message"].lower())
        self.assertFalse(Path(rows[0]["destination"]).exists())
        self.assertTrue(Path(rows[1]["destination"]).is_file())

    def test_actual_python_module_process_exports_all_camera_scenes(self):
        # The installed entry point, not an extracted function or a fake parser.
        process = subprocess.run([sys.executable, "-m", "fmn_python", *self.arguments("-a")],
                                 cwd=self.root, stdin=subprocess.DEVNULL, capture_output=True,
                                 text=True, timeout=120)
        self.assertEqual(process.returncode, 0, (process.stdout, process.stderr))
        self.assertEqual(len(process.stdout.splitlines()), 1)
        report = json.loads(process.stdout)
        self.assertEqual(report["kind"], "render-batch")
        self.assertEqual(report["batch"]["counts"]["succeeded"], 2)
        self.assertEqual(process.stderr.count("bundle-batch-source-loaded"), 1)
        self.compare_independent(report, True)


if __name__ == "__main__":
    unittest.main()
