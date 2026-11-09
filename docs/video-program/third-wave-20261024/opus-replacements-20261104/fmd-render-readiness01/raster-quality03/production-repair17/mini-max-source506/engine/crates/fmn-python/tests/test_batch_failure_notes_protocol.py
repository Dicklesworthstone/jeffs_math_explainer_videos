"""Batch failure diagnostics over the shared render-session boundary fixture."""
import unittest
import test_batch_rendering_protocol as support


class BatchFailureDiagnosticsTests(unittest.TestCase):
    setUp = support.BatchRenderingTests.setUp
    render = support.BatchRenderingTests.render

    def test_primary_error_retains_native_cleanup_notes(self):
        instance = support.First()
        instance.run_error = ValueError("primary")
        instance.abort_error = RuntimeError("cleanup failed")
        report = self.render([instance], continue_on_error=True)
        item = report.outcomes[0]
        self.assertEqual(item.message, "primary")
        self.assertIn("cleanup failed", item.notes[0])
        self.assertEqual(item.as_dict()["error"]["notes"], list(item.notes))

    def test_exception_notes_are_bounded_without_dropping_publication_warning(self):
        failure = RuntimeError("provenance unavailable")
        failure.add_note("native artifact is already published at output.png")
        for _ in range(20):
            failure.add_note("x" * 5000)
        instance = support.First()
        instance.run_error = failure
        report = self.render([instance], continue_on_error=True)
        notes = report.outcomes[0].notes
        self.assertEqual(len(notes), 8)
        self.assertIn("already published", notes[0])
        self.assertLessEqual(max(map(len, notes)), 1024)

    def test_native_no_clobber_still_handles_race_after_preflight(self):
        def observer(outcome):
            if outcome.name == "First":
                (outcome.destination.parent / "Second.png").write_bytes(b"racing-writer")
        report = self.render([support.First, support.Second], on_result=observer, continue_on_error=True)
        self.assertEqual([x.status for x in report.outcomes], ["succeeded", "failed"])
        self.assertEqual(report.outcomes[1].destination.read_bytes(), b"racing-writer")
        self.assertEqual(support.Scene.instances[-1].events[-1], "abort")


if __name__ == "__main__":
    unittest.main()
