from __future__ import annotations

import contextlib
import hashlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import verify_portal_runtime_receipt as verifier


SCHEMA_TEXT = """[meta]
schema_version\t1
[symbols]
manimlib.fake\tWidget\tclass\tdefined\t1\tobject
manimlib.fake\tWidget.method\tmethod\tdefined\t0\t-
manimlib.other\tfunction\tfunction\tdefined\t1\t-
"""
OVERLAY_TEXT = """[status]
manimlib.fake:Widget\tsame\tevidence\ttests\tnote
manimlib.fake:Widget.method\timproved\tevidence\ttests\tnote
manimlib.other:function\ttiered\tevidence\ttests\tnote
"""


class PortalRuntimeReceiptTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.report_path = root / "report.json"
        self.schema_path = root / "API_SCHEMA.tsv"
        self.overlay_path = root / "API_OVERLAY.tsv"
        self.schema_path.write_text(SCHEMA_TEXT, encoding="utf-8")
        self.overlay_path.write_text(OVERLAY_TEXT, encoding="utf-8")

    def valid_report(self) -> dict:
        return {
            "schema": "fmn.portal.runtime-audit",
            "version": 1,
            "ok": True,
            "counts": {
                "status_rows": 3,
                "reviewed_implemented": 2,
                "runtime_placeholders": 0,
                "trivial_bodies": 0,
                "missing_reviewed": 0,
                "contradictions": 0,
            },
            "status_counts": {
                "excluded": 0,
                "improved": 1,
                "same": 1,
                "tiered": 1,
                "unreviewed": 0,
            },
            "contradictions": [],
            "overlay_sha256": hashlib.sha256(OVERLAY_TEXT.encode("utf-8")).hexdigest(),
            "api_schema_sha256": hashlib.sha256(SCHEMA_TEXT.encode("utf-8")).hexdigest(),
            "schema_provenance": {
                "version": 1,
                "counts": {
                    "schema_rows": 3,
                    "modules": 3,
                    "classes": 0,
                    "constructors": 0,
                    "functions": 0,
                    "methods": 0,
                },
            },
        }

    def write_report(self, report=None) -> None:
        if report is None:
            report = self.valid_report()
        self.report_path.write_text(
            json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n",
            encoding="utf-8",
        )

    def assert_receipt_error(self, callable_, *, exit_code: int, identity: str):
        with self.assertRaises(verifier.ReceiptError) as raised:
            callable_()
        self.assertEqual(raised.exception.exit_code, exit_code)
        self.assertEqual(raised.exception.identity, identity)
        return str(raised.exception)

    def verify(self):
        return verifier.verify_receipt(
            self.report_path,
            self.schema_path,
            self.overlay_path,
        )

    def test_valid_receipt_reconciles_authorities(self) -> None:
        self.write_report()
        receipt = self.verify()
        self.assertEqual(receipt["schema"], "fmn.portal.runtime-receipt")
        self.assertEqual(receipt["version"], 1)
        self.assertTrue(receipt["ok"])
        self.assertEqual(receipt["reviewed_implemented"], 2)
        self.assertEqual(
            receipt["api_schema_sha256"],
            hashlib.sha256(SCHEMA_TEXT.encode("utf-8")).hexdigest(),
        )
        self.assertEqual(
            receipt["api_overlay_sha256"],
            hashlib.sha256(OVERLAY_TEXT.encode("utf-8")).hexdigest(),
        )

    def test_main_renders_human_and_robot_receipts(self) -> None:
        self.write_report()
        for robot in (False, True):
            with self.subTest(robot=robot):
                stdout = io.StringIO()
                stderr = io.StringIO()
                argv = [
                    str(self.report_path),
                    "--schema",
                    str(self.schema_path),
                    "--overlay",
                    str(self.overlay_path),
                ]
                if robot:
                    argv.append("--robot")
                with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                    code = verifier.main(argv)
                self.assertEqual(code, 0)
                self.assertEqual(stderr.getvalue(), "")
                if robot:
                    payload = json.loads(stdout.getvalue())
                    self.assertEqual(payload["schema"], "fmn.portal.runtime-receipt")
                    self.assertTrue(payload["ok"])
                else:
                    self.assertIn("portal runtime receipt: PASS", stdout.getvalue())
                    self.assertIn("schema identity:", stdout.getvalue())
                    self.assertIn("overlay identity:", stdout.getvalue())

    def test_schema_and_overlay_drift_are_independent_stale_failures(self) -> None:
        self.write_report()
        self.schema_path.write_text(SCHEMA_TEXT + "# drift\n", encoding="utf-8")
        message = self.assert_receipt_error(
            self.verify,
            exit_code=1,
            identity="stale-artifact",
        )
        self.assertIn("API_SCHEMA.tsv", message)

        self.schema_path.write_text(SCHEMA_TEXT, encoding="utf-8")
        self.overlay_path.write_text(OVERLAY_TEXT + "# drift\n", encoding="utf-8")
        message = self.assert_receipt_error(
            self.verify,
            exit_code=1,
            identity="stale-artifact",
        )
        self.assertIn("API_OVERLAY.tsv", message)

    def test_success_envelope_cannot_hide_audit_failures(self) -> None:
        for field, value in (
            ("runtime_placeholders", 1),
            ("trivial_bodies", 1),
            ("missing_reviewed", 1),
            ("contradictions", 1),
        ):
            with self.subTest(field=field):
                report = self.valid_report()
                report["counts"][field] = value
                self.write_report(report)
                self.assert_receipt_error(
                    self.verify,
                    exit_code=1,
                    identity="audit-failed",
                )
        report = self.valid_report()
        report["contradictions"] = [{"symbol": "x"}]
        self.write_report(report)
        self.assert_receipt_error(
            self.verify,
            exit_code=1,
            identity="audit-failed",
        )

    def test_status_and_provenance_reconciliation_fail_closed(self) -> None:
        report = self.valid_report()
        report["status_counts"]["same"] = 2
        self.write_report(report)
        self.assert_receipt_error(
            self.verify,
            exit_code=1,
            identity="stale-artifact",
        )

        report = self.valid_report()
        report["schema_provenance"]["counts"]["modules"] = 2
        self.write_report(report)
        self.assert_receipt_error(
            self.verify,
            exit_code=1,
            identity="stale-artifact",
        )

        report = self.valid_report()
        report["schema_provenance"]["counts"]["functions"] = 2
        self.write_report(report)
        self.assert_receipt_error(
            self.verify,
            exit_code=2,
            identity="invalid-receipt",
        )

    def test_duplicate_nonfinite_and_unknown_json_fail_closed(self) -> None:
        raw_cases = (
            '{"schema":"x","schema":"y"}\n',
            '{"value":NaN}\n',
            '{"unknown":1}\n',
        )
        for raw in raw_cases:
            with self.subTest(raw=raw):
                self.report_path.write_text(raw, encoding="utf-8")
                self.assert_receipt_error(
                    self.verify,
                    exit_code=2,
                    identity="invalid-receipt",
                )

    def test_depth_node_and_byte_limits_fail_closed(self) -> None:
        self.report_path.write_text("[" * 40 + "0" + "]" * 40, encoding="utf-8")
        self.assert_receipt_error(
            self.verify,
            exit_code=2,
            identity="invalid-receipt",
        )

        self.report_path.write_text(json.dumps([0, 1, 2, 3]), encoding="utf-8")
        with mock.patch.object(verifier, "MAX_JSON_NODES", 3):
            self.assert_receipt_error(
                self.verify,
                exit_code=2,
                identity="invalid-receipt",
            )

        self.report_path.write_bytes(b"{}\n")
        with mock.patch.object(verifier, "MAX_REPORT_BYTES", 2):
            self.assert_receipt_error(
                self.verify,
                exit_code=2,
                identity="invalid-receipt",
            )

    def test_main_preserves_typed_failure_exit(self) -> None:
        self.report_path.write_text("{}\n", encoding="utf-8")
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            code = verifier.main(
                [
                    str(self.report_path),
                    "--schema",
                    str(self.schema_path),
                    "--overlay",
                    str(self.overlay_path),
                    "--robot",
                ]
            )
        self.assertEqual(code, 2)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("invalid-receipt", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
