from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path

import agent_brief
import agent_claim_guard


BASE_PREFIX = (
    b'{"id":"fm-json","title":"W10: strict JSON","status":"open",'
    b'"priority":1,"issue_type":"task","created_at":"2026-08-01T00:00:00Z",'
    b'"updated_at":"2026-09-01T00:00:00Z"'
)


class StrictJsonLedgerTests(unittest.TestCase):
    def ledger(self, suffix: bytes) -> Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        path = Path(directory.name) / "issues.jsonl"
        path.write_bytes(BASE_PREFIX + suffix + b"}\n")
        return path

    def test_nonfinite_constants_are_rejected_even_in_extension_fields(self) -> None:
        for spelling in (b"NaN", b"Infinity", b"-Infinity"):
            with self.subTest(spelling=spelling):
                path = self.ledger(b',"extension":' + spelling)
                with self.assertRaisesRegex(
                    agent_brief.BriefError,
                    rf"non-finite JSON constant '{spelling.decode()}' is forbidden",
                ):
                    agent_brief.load_issues(path)

    def test_nested_nonfinite_constants_are_rejected_before_comment_projection(self) -> None:
        for suffix in (
            b',"comments":[{"text":"ok","score":NaN}]',
            b',"dependencies":[],"extension":{"nested":[Infinity]}',
        ):
            with self.subTest(suffix=suffix):
                with self.assertRaisesRegex(agent_brief.BriefError, "non-finite JSON constant"):
                    agent_brief.load_issues(self.ledger(suffix))

    def test_quoted_spellings_remain_ordinary_json_strings(self) -> None:
        issues = agent_brief.load_issues(
            self.ledger(b',"extension":["NaN","Infinity","-Infinity"]')
        )
        self.assertEqual(tuple(issues), ("fm-json",))

    def test_cli_refusal_emits_no_projection_and_names_the_constant(self) -> None:
        path = self.ledger(b',"extension":NaN')
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = agent_brief.main(
                [
                    "--ledger",
                    str(path),
                    "--as-of",
                    "2026-09-01T00:00:00Z",
                    "--format",
                    "json",
                    "--check",
                ]
            )
        self.assertEqual(status, 2)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("non-finite JSON constant 'NaN' is forbidden", stderr.getvalue())

    def test_claim_graph_version_records_full_task_semantics(self) -> None:
        self.assertEqual(agent_claim_guard.CLAIM_GRAPH_VERSION, 3)
        self.assertEqual(agent_claim_guard.CLAIM_INPUT_VERSION, 3)
        self.assertEqual(
            agent_claim_guard.schema_contract()["claim_graph"],
            {"schema": "fmn.agent.claim-graph", "version": 3},
        )


if __name__ == "__main__":
    unittest.main()
