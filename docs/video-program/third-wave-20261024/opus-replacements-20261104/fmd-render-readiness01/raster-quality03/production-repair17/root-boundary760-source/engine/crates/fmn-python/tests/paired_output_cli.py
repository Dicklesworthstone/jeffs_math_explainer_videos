"""Native console and ordered-batch paired outputs; no parser or sink doubles."""
from __future__ import annotations

import builtins
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

from fmn_python.__main__ import main


class PairedConsole(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="fmn-paired-console-"))
        self.events = []
        self.key = "_fmn_paired_events_" + self.root.name.rsplit("-", 1)[-1]
        setattr(builtins, self.key, self.events)
        self.addCleanup(delattr, builtins, self.key)
        self.source = self.root / "scene.py"
        self.source.write_text(f'''
from manimlib import Scene, Square, RIGHT, linear
import builtins
events = getattr(builtins, {self.key!r})
events.append(("import", __file__))
print("source chatter")
class A(Scene):
    def __init__(self):
        events.append(("new", type(self).__name__))
        super().__init__()
    def construct(self):
        events.append(("run", type(self).__name__))
        print("construct chatter", type(self).__name__)
        box = Square(fill_opacity=1)
        self.add(box)
        self.wait(0.125)
        self.play(box.animate.shift(RIGHT), run_time=0.25, rate_func=linear)
    def tear_down(self):
        events.append(("done", type(self).__name__))
class B(A):
    pass
''')

    def append_source(self, text):
        self.source.write_text(self.source.read_text() + text)

    def invoke(self, destination, *selectors, format="y4m", extra=(), paired=True):
        argv = ["fmn-python", "--robot", str(self.source), *selectors,
                "--format", format, "--resolution", "96x54", "--fps", "8", "--threads", "1"]
        if destination is not None:
            argv += ["--video_dir", str(destination)]
        if paired:
            argv.append("--save-last-frame")
        argv += list(extra)
        before_argv, before_path = sys.argv, list(sys.path)
        out, err = io.StringIO(), io.StringIO()
        try:
            sys.argv = argv
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                code = main()
        finally:
            sys.argv = before_argv
        self.assertEqual(sys.path, before_path)
        lines = out.getvalue().splitlines()
        self.assertEqual(len(lines), 1, out.getvalue())
        report = json.loads(lines[0])
        self.assertEqual(report["exit"]["code"], code)
        return code, report, err.getvalue()

    def receipt(self, result):
        data = Path(result["destination"]).read_bytes()
        self.assertEqual(len(data), result["bytes"])
        self.assertEqual(hashlib.sha256(data).hexdigest(), result["digest"])
        return data

    def created(self):
        return [name for kind, name in self.events if kind == "new"]

    def test_single_console_produces_one_pair_and_one_robot_receipt(self):
        code, report, stderr = self.invoke(self.root / "single.y4m", "A")
        self.assertEqual(code, 0, report)
        self.assertEqual(report["kind"], "render-paired")
        pair = report["pair"]
        self.assertTrue(pair["completed"])
        self.assertEqual(pair["still_destination"], str(self.root / "single.png"))
        self.assertEqual(pair["primary"]["frame_count"], 3)
        self.assertEqual(pair["still"]["frame_count"], 1)
        self.receipt(pair["primary"])
        self.assertTrue(self.receipt(pair["still"]).startswith(b"\x89PNG\r\n\x1a\n"))
        self.assertEqual(self.created(), ["A"])
        self.assertIn("source chatter", stderr)
        self.assertIn("construct chatter A", stderr)

    def test_sequence_console_keeps_final_image_outside_frame_collection(self):
        frames = self.root / "frames"
        code, report, _ = self.invoke(frames, "A", format="png_sequence")
        self.assertEqual(code, 0, report)
        pair = report["pair"]
        self.assertTrue(pair["completed"])
        self.assertEqual(pair["primary"]["frame_count"], 3)
        self.assertEqual(pair["still_destination"], str(self.root / "frames.png"))
        images = sorted(frames.glob("*.png"))
        self.assertEqual(len(images), 3)
        self.assertTrue(all(path.read_bytes().startswith(b"\x89PNG\r\n\x1a\n") for path in images))
        self.receipt(pair["still"])
        self.assertEqual(self.created(), ["A"])

    def test_named_batch_preserves_order_and_both_receipts(self):
        code, report, _ = self.invoke(self.root / "batch", "B", "A")
        self.assertEqual(code, 0, report)
        rows = report["batch"]["outcomes"]
        self.assertEqual([row["name"] for row in rows], ["B", "A"])
        self.assertEqual(self.created(), ["B", "A"])
        for row in rows:
            self.assertTrue(row["result"]["completed"])
            self.receipt(row["result"]["primary"])
            self.receipt(row["result"]["still"])

    def test_write_all_and_underscore_alias(self):
        code, report, _ = self.invoke(self.root / "all", paired=False,
            extra=("--write_all", "--save_last_frame"))
        self.assertEqual(code, 0, report)
        self.assertEqual(self.created(), ["A", "B"])
        self.assertTrue(report["batch"]["ok"])
        self.assertEqual(len(report["batch"]["outcomes"]), 2)

    def test_fail_fast_does_not_start_the_next_scene(self):
        self.append_source('''
class Failing(A):
    def construct(self):
        super().construct()
        raise RuntimeError("authored failure")
''')
        code, report, _ = self.invoke(self.root / "fail-fast", "A", "Failing", "B")
        self.assertEqual(code, 5, report)
        self.assertEqual(self.created(), ["A", "Failing"])
        rows = report["batch"]["outcomes"]
        self.assertEqual([row["status"] for row in rows], ["succeeded", "failed", "not_run"])
        self.receipt(rows[0]["result"]["primary"])
        self.receipt(rows[0]["result"]["still"])
        self.assertFalse(rows[1]["result"]["completed"])
        self.assertIsNone(rows[1]["result"]["primary"])

    def test_keep_going_after_a_partial_publication_reports_failure(self):
        self.append_source('''
class Late(A):
    def _finish_render(self):
        receipt = super()._finish_render()
        self.__dict__["_fmn_paired_render_session"].still_destination.write_bytes(b"competing output")
        return receipt
''')
        code, report, _ = self.invoke(self.root / "keep-going", "Late", "B", extra=("--keep-going",))
        self.assertEqual(code, 5, report)
        self.assertEqual(self.created(), ["Late", "B"])
        first, second = report["batch"]["outcomes"]
        self.assertEqual(first["status"], "failed")
        self.assertFalse(first["result"]["completed"])
        self.receipt(first["result"]["primary"])
        self.assertIsNone(first["result"]["still"])
        self.assertEqual(Path(first["result"]["still_destination"]).read_bytes(), b"competing output")
        self.assertEqual(second["status"], "succeeded")
        self.receipt(second["result"]["still"])

    def test_interrupt_retains_batch_progress_and_stops(self):
        self.append_source('''
class Interrupted(A):
    def construct(self):
        super().construct()
        raise KeyboardInterrupt("stop")
''')
        code, report, _ = self.invoke(self.root / "interrupt", "A", "Interrupted", "B", extra=("--keep-going",))
        self.assertEqual(code, 130, report)
        self.assertEqual(self.created(), ["A", "Interrupted"])
        self.assertEqual([row["status"] for row in report["batch"]["outcomes"]],
                         ["succeeded", "cancelled", "not_run"])
        self.assertFalse(report["batch"]["outcomes"][1]["result"]["completed"])

    def test_all_still_paths_are_checked_before_any_constructor(self):
        destination = self.root / "preflight"
        destination.mkdir()
        (destination / "B.png").write_bytes(b"keep this image")
        code, report, _ = self.invoke(destination, "A", "B")
        self.assertEqual(code, 6, report)
        self.assertEqual(self.created(), [])
        self.assertEqual(list(destination.iterdir()), [destination / "B.png"])
        self.assertEqual((destination / "B.png").read_bytes(), b"keep this image")

    def test_subdivided_batch_has_one_still_per_scene_not_per_clip(self):
        code, report, _ = self.invoke(self.root / "subdivide", "A", "B", extra=("--subdivide",))
        self.assertEqual(code, 0, report)
        for row in report["batch"]["outcomes"]:
            pair = row["result"]
            primary = pair["primary"]
            self.assertEqual(len(primary["segments"]), 2)
            for clip in primary["segments"]:
                self.receipt(clip["render"])
            self.assertEqual(Path(pair["still_destination"]).name, "clips.png")
            self.receipt(pair["still"])

    def test_selected_range_is_preserved_in_the_pair(self):
        code, report, _ = self.invoke(self.root / "range.y4m", "A", extra=("-n", "1,2"))
        self.assertEqual(code, 0, report)
        self.assertEqual(report["pair"]["primary"]["frame_count"], 2)
        self.assertEqual(report["pair"]["primary"]["animation_range"], [1, 2])
        self.assertEqual(report["pair"]["still"]["animation_range"], [1, 2])
        self.receipt(report["pair"]["still"])

    def test_incompatible_options_refuse_before_source_loading(self):
        # A paired batch with --checkpoint/--resume-key is supported since
        # 5a3dcc97; batch_checkpoint_acceptance.py proves it resumes.
        # Subdivided paired recovery is still excluded and refuses before the
        # source is imported (fm-djox).
        cases = (("png", (), 2), ("svg", (), 2), ("y4m", ("-s",), 2),
                 ("png_sequence", ("--reproducible",), 4),
                 ("y4m", ("--save_last_frame",), 2),
                 ("y4m", ("--subdivide", "--checkpoint", str(self.root / "journal"),
                          "--resume-key", "inputs"), 2))
        for index, (format, extra, expected) in enumerate(cases):
            with self.subTest(format=format, extra=extra):
                self.events.clear()
                code, report, _ = self.invoke(self.root / f"invalid{index}", "A", "B", format=format, extra=extra)
                self.assertEqual(code, expected, report)
                self.assertEqual(self.events, [])

    def test_switch_like_native_value_is_not_consumed_as_a_flag(self):
        previous = Path.cwd()
        try:
            os.chdir(self.root)
            code, report, _ = self.invoke("--save-last-frame", "A", paired=False)
        finally:
            os.chdir(previous)
        self.assertEqual(code, 0, report)
        self.assertEqual(report["kind"], "render")
        self.assertTrue((self.root / "--save-last-frame").is_file())
        self.assertNotIn("pair", report)

    def test_default_destination_is_frozen_before_source_changes_cwd(self):
        previous = Path.cwd()
        other = self.root / "other"
        other.mkdir()
        self.append_source(f'\nimport os\nos.chdir({str(other)!r})\n')
        try:
            os.chdir(self.root)
            code, report, _ = self.invoke(None, "A")
        finally:
            os.chdir(previous)
        self.assertEqual(code, 0, report)
        self.assertEqual(Path(report["pair"]["still_destination"]), self.root / "media/videos/scene/A.png")
        self.assertEqual(list(other.iterdir()), [])

    @unittest.skipUnless(shutil.which("ffmpeg"), "optional ffmpeg not installed")
    def test_mp4_retains_native_ffmpeg_receipt_and_png(self):
        code, report, _ = self.invoke(self.root / "video.mp4", "A", format="mp4")
        self.assertEqual(code, 0, report)
        primary = report["pair"]["primary"]
        self.assertTrue(primary["ffmpeg_invocations"])
        self.assertEqual(primary["frame_count"], 3)
        self.receipt(primary)
        self.receipt(report["pair"]["still"])


suite = unittest.defaultTestLoader.loadTestsFromTestCase(PairedConsole)
assert suite.countTestCases() == 14
if not unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful():
    raise AssertionError("paired console and batch acceptance failed")
