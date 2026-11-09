from __future__ import annotations

import contextlib
import datetime as dt
import io
import json
import tempfile
import unittest
from pathlib import Path

import agent_claim_policy
import agent_next


def record(
    issue_id: str,
    *,
    title: str = "W10: ready",
    status: str = "open",
    priority: int = 2,
    labels: object = None,
) -> dict:
    row = {
        "id": issue_id,
        "title": title,
        "status": status,
        "priority": priority,
        "issue_type": "task",
        "created_at": "2026-08-01T00:00:00Z",
        "updated_at": "2026-08-31T00:00:00Z",
    }
    if labels is not None:
        row["labels"] = labels
    return row


class AgentClaimPolicyTests(unittest.TestCase):
    def ledger(self, rows: list[dict]) -> Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        path = Path(directory.name) / "issues.jsonl"
        path.write_text(
            "".join(json.dumps(row, separators=(",", ":")) + "\n" for row in rows),
            encoding="utf-8",
        )
        return path

    def plan(self, rows: list[dict]) -> dict:
        path = self.ledger(rows)
        issues = agent_next.agent_brief.load_issues(path)
        return agent_next.build_plan(
            issues,
            as_of=dt.datetime(2026, 8, 31, tzinfo=dt.timezone.utc),
            stale_days=2,
            activation_cap=4,
            limit=20,
        )

    def test_unlabelled_and_explicit_auto_are_autonomous(self) -> None:
        plan = self.plan(
            [
                record("fm-default", priority=2),
                record(
                    "fm-explicit",
                    priority=1,
                    labels=[agent_claim_policy.AUTO_LABEL],
                ),
            ]
        )
        self.assertTrue(plan["integrity"]["ok"])
        self.assertEqual(plan["recommendation"]["issue"]["id"], "fm-explicit")
        policies = {
            row["id"]: row["claim_policy"] for row in plan["claimable_leaves"]
        }
        self.assertEqual(policies["fm-default"]["source"], "default")
        self.assertEqual(policies["fm-explicit"]["source"], "label")
        self.assertTrue(policies["fm-default"]["autonomous"])
        self.assertTrue(policies["fm-explicit"]["autonomous"])

    def test_manual_and_external_leaves_stay_visible_but_never_win(self) -> None:
        plan = self.plan(
            [
                record(
                    "fm-manual",
                    priority=0,
                    labels=[agent_claim_policy.MANUAL_LABEL],
                ),
                record(
                    "fm-external",
                    priority=1,
                    labels=[agent_claim_policy.EXTERNAL_LABEL],
                ),
                record("fm-auto", priority=4),
            ]
        )
        self.assertTrue(plan["integrity"]["ok"])
        self.assertEqual(plan["recommendation"]["issue"]["id"], "fm-auto")
        self.assertEqual(
            [row["id"] for row in plan["non_autonomous_ready"]],
            ["fm-manual", "fm-external"],
        )
        self.assertEqual(plan["counts"]["claimable_leaves"], 1)
        self.assertEqual(plan["counts"]["non_autonomous_ready"], 2)
        self.assertEqual(plan["counts"]["manual_ready"], 1)
        self.assertEqual(plan["counts"]["external_ready"], 1)

    def test_only_non_autonomous_work_produces_no_recommendation(self) -> None:
        plan = self.plan(
            [
                record("fm-manual", labels=[agent_claim_policy.MANUAL_LABEL]),
                record("fm-external", labels=[agent_claim_policy.EXTERNAL_LABEL]),
            ]
        )
        self.assertTrue(plan["integrity"]["ok"])
        self.assertIsNone(plan["recommendation"]["issue"])
        self.assertIn("only manual/external", plan["recommendation"]["reason"])

    def test_unknown_duplicate_and_conflicting_reserved_labels_fail_closed(self) -> None:
        cases = {
            "unknown-claim-label": ["agent:claim:maybe"],
            "duplicate-claim-label": [
                agent_claim_policy.MANUAL_LABEL,
                agent_claim_policy.MANUAL_LABEL,
            ],
            "conflicting-claim-labels": [
                agent_claim_policy.MANUAL_LABEL,
                agent_claim_policy.EXTERNAL_LABEL,
            ],
        }
        for code, labels in cases.items():
            with self.subTest(code=code):
                rows = [record("fm-bad", labels=labels), record("fm-good", priority=4)]
                plan = self.plan(rows)
                self.assertFalse(plan["integrity"]["ok"])
                self.assertIsNone(plan["recommendation"]["issue"])
                self.assertEqual(
                    plan["integrity"]["claim_policy_violations"][0]["code"],
                    code,
                )
                path = self.ledger(rows)
                stdout = io.StringIO()
                stderr = io.StringIO()
                with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                    status = agent_next.main(["--ledger", str(path), "--require"])
                self.assertEqual(status, 1)
                self.assertEqual(stdout.getvalue(), "")
                self.assertIn("task-graph integrity failed", stderr.getvalue())

    def test_malformed_labels_fail_closed(self) -> None:
        plan = self.plan([record("fm-bad", labels=agent_claim_policy.MANUAL_LABEL)])
        self.assertFalse(plan["integrity"]["ok"])
        self.assertEqual(
            plan["integrity"]["claim_policy_violations"],
            [{"issue_id": "fm-bad", "code": "invalid-labels", "labels": []}],
        )

    def test_closed_policy_error_does_not_poison_current_live_selection(self) -> None:
        plan = self.plan(
            [
                record("fm-closed", status="closed", labels=["agent:claim:maybe"]),
                record("fm-live", priority=1),
            ]
        )
        self.assertTrue(plan["integrity"]["ok"])
        self.assertEqual(plan["integrity"]["claim_policy_violations"], [])
        self.assertEqual(plan["recommendation"]["issue"]["id"], "fm-live")

    def test_contract_is_exact_and_machine_readable(self) -> None:
        self.assertEqual(
            agent_claim_policy.contract(),
            {
                "schema": "fmn.agent.claim-policy",
                "version": 1,
                "label_prefix": "agent:claim:",
                "default_mode": "auto",
                "labels": {
                    "auto": "agent:claim:auto",
                    "manual": "agent:claim:manual",
                    "external": "agent:claim:external",
                },
            },
        )


if __name__ == "__main__":
    unittest.main()
