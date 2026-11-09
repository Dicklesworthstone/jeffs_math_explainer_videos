from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import agent_next


class AgentNextOutputTests(unittest.TestCase):
    def ledger(self, rows: list[dict]) -> Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        path = Path(directory.name) / "issues.jsonl"
        path.write_text(
            "".join(json.dumps(row, separators=(",", ":")) + "\n" for row in rows),
            encoding="utf-8",
        )
        return path

    @staticmethod
    def issue(ident: str, updated_at: str) -> dict:
        return {
            "id": ident,
            "title": "W10: ready",
            "status": "open",
            "priority": 1,
            "issue_type": "task",
            "created_at": "2026-08-01T00:00:00Z",
            "updated_at": updated_at,
        }

    def test_default_json_timestamp_is_ledger_derived_and_byte_deterministic(self) -> None:
        path = self.ledger(
            [
                self.issue("fm-old", "2026-08-30T12:00:00Z"),
                self.issue("fm-new", "2026-08-31T08:15:00Z"),
            ]
        )
        outputs: list[str] = []
        for _ in range(2):
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                status = agent_next.main(
                    ["--ledger", str(path), "--format", "json", "--check"]
                )
            self.assertEqual(status, 0)
            outputs.append(stdout.getvalue())
        self.assertEqual(outputs[0], outputs[1])
        payload = json.loads(outputs[0])
        self.assertEqual(payload["as_of"], "2026-08-31T08:15:00Z")

    def test_empty_ledger_has_a_stable_epoch_timestamp(self) -> None:
        path = self.ledger([])
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            status = agent_next.main(
                ["--ledger", str(path), "--format", "json", "--check"]
            )
        self.assertEqual(status, 0)
        payload = json.loads(stdout.getvalue())
        self.assertEqual(payload["as_of"], "1970-01-01T00:00:00Z")
        self.assertIsNone(payload["recommendation"]["issue"])

    def test_output_budget_refuses_before_any_machine_payload(self) -> None:
        path = self.ledger([self.issue("fm-next", "2026-08-31T08:15:00Z")])
        stdout = io.StringIO()
        stderr = io.StringIO()
        with mock.patch.object(agent_next, "MAX_PLAN_OUTPUT_BYTES", 32):
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                status = agent_next.main(
                    ["--ledger", str(path), "--format", "json", "--check"]
                )
        self.assertEqual(status, 2)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("claim plan exceeds the 32-byte output limit", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
