"""Production paired owner; only native sinks and camera storage are modeled."""
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest

from fmn_python.paired_output import PairedRenderSession
from fmn_python.rendering import RenderResult


def fixture():
    events = []
    root = Path(tempfile.mkdtemp(prefix="fmn-pair-protocol-"))

    class Pending:
        def commit(self):
            events.append("still.commit")
            if primary.collision:
                raise FileExistsError("a concurrent PNG publisher")
            if primary.bad_receipt:
                return ()
            return root / "still.png", 10, "a" * 64
        def abort(self):
            events.append("still.abort")

    class Capture:
        size = (96, 54)
        def prepare_png(self, path, threads=1):
            events.append("still.prepare")
            if primary.fail_prepare:
                raise ValueError("prepare failed")
            return Pending()

    def capture(*roots):
        events.append("capture")
        if primary.fail_capture:
            raise ValueError("capture failed")
        if primary.abort_capture is not None:
            primary.abort_capture()
        return Capture()

    class Primary:
        def __init__(self):
            self.scene = SimpleNamespace(camera=SimpleNamespace(capture_snapshot=capture),
                mobjects=[], file_writer=SimpleNamespace(png_mode="RGBA"))
            self._native = SimpleNamespace(_CameraCapture=Capture, _CapabilityError=RuntimeError)
            self.destination, self.format = root / "movie.y4m", "y4m"
            self.resolution, self.fps, self.threads, self.seed = (96, 54), 8, 1, 0
            self.reproducible, self.animation_range = False, None
            self.result, self.artifact_published = None, False
            self.fail_capture = self.fail_prepare = self.fail_finish = False
            self.collision = self.bad_receipt = self.fail_enter = False
            self.abort_capture = None
        def _check_owner(self):
            pass
        def __enter__(self):
            if self.fail_enter:
                raise RuntimeError("another owner")
            events.append("primary.enter")
            return self
        def finish(self):
            events.append("primary.finish")
            if self.fail_finish:
                raise ValueError("finish failed")
            self.artifact_published = True
            self.result = RenderResult(self.destination, "y4m", (96, 54), 8, 1,
                "cpu", 100, "b" * 64, 3, None, 0)
        def abort(self):
            events.append("primary.abort")

    primary = Primary()
    return root, primary, events


class PairedProtocol(unittest.TestCase):
    def test_order_and_success_receipts(self):
        root, primary, events = fixture()
        with PairedRenderSession(primary, root / "still.png") as session:
            pass
        self.assertEqual(events, ["primary.enter", "capture", "still.prepare", "primary.finish", "still.commit"])
        self.assertTrue(session.result.completed)
        self.assertIs(session.result.primary, primary.result)
        self.assertEqual(session.result.still.frame_count, 1)
        self.assertIs(session.finish(), session.result)
        self.assertEqual(session.result.as_dict()["schema"], "fmn.paired-render")

    def test_failure_at_each_prepublication_phase(self):
        for flag in ("fail_capture", "fail_prepare", "fail_finish"):
            with self.subTest(flag=flag):
                root, primary, events = fixture()
                setattr(primary, flag, True)
                session = PairedRenderSession(primary, root / "still.png")
                with self.assertRaises(ValueError) as caught:
                    with session:
                        pass
                self.assertNotIn("still.commit", events)
                self.assertIn("primary.abort", events)
                self.assertFalse(caught.exception.render_pair_result.completed)
                self.assertFalse(session.artifact_published)
                self.assertIsNone(session.result)

    def test_late_collision_preserves_primary_receipt(self):
        root, primary, events = fixture()
        primary.collision = True
        session = PairedRenderSession(primary, root / "still.png")
        with self.assertRaises(FileExistsError) as caught:
            with session:
                pass
        report = caught.exception.render_pair_result
        self.assertIs(report.primary, primary.result)
        self.assertIsNone(report.still)
        self.assertFalse(report.completed)
        self.assertTrue(session.artifact_published)

    def test_still_receipt_failure_reports_published_path(self):
        root, primary, events = fixture()
        primary.bad_receipt = True
        session = PairedRenderSession(primary, root / "still.png")
        with self.assertRaises(ValueError) as caught:
            with session:
                pass
        self.assertEqual(caught.exception.render_pair_result.unreceipted_artifacts, (root / "still.png",))

    def test_authored_exception_is_not_replaced(self):
        root, primary, events = fixture()
        error = KeyboardInterrupt("stop")
        session = PairedRenderSession(primary, root / "still.png")
        with self.assertRaises(KeyboardInterrupt) as caught:
            with session:
                raise error
        self.assertIs(caught.exception, error)
        self.assertEqual(events, ["primary.enter", "primary.abort"])

    def test_native_entry_refusal_does_not_cancel_existing_owner(self):
        root, primary, events = fixture()
        primary.fail_enter = True
        with self.assertRaises(RuntimeError):
            with PairedRenderSession(primary, root / "still.png"):
                self.fail("entry should fail")
        self.assertEqual(events, [])

    def test_capture_cancellation_cannot_publish(self):
        root, primary, events = fixture()
        session = PairedRenderSession(primary, root / "still.png")
        primary.abort_capture = session.abort
        with self.assertRaises(RuntimeError):
            with session:
                pass
        self.assertNotIn("primary.finish", events)
        self.assertNotIn("still.prepare", events)

    def test_existing_png_is_checked_before_primary_entry(self):
        root, primary, events = fixture()
        path = root / "still.png"
        path.write_bytes(b"existing")
        with self.assertRaises(FileExistsError):
            with PairedRenderSession(primary, path):
                self.fail("entry should fail")
        self.assertEqual(path.read_bytes(), b"existing")
        self.assertEqual(events, [])

    def test_overlapping_paths_are_rejected(self):
        root, primary, events = fixture()
        for path in (primary.destination, primary.destination / "still.png", root):
            with self.assertRaises(ValueError):
                PairedRenderSession(primary, path)
        self.assertEqual(events, [])

    def test_standard_capture_never_inherits_certification(self):
        root, primary, events = fixture()
        primary.reproducible = True
        with self.assertRaisesRegex(RuntimeError, "standard-only"):
            PairedRenderSession(primary, root / "still.png")
        self.assertEqual(events, [])

    def test_manual_abort_and_second_entry(self):
        root, primary, events = fixture()
        session = PairedRenderSession(primary, root / "still.png")
        with session:
            session.abort()
        self.assertIsNone(session.result)
        self.assertNotIn("capture", events)
        with self.assertRaises(RuntimeError):
            session.__enter__()


if __name__ == "__main__":
    unittest.main()
