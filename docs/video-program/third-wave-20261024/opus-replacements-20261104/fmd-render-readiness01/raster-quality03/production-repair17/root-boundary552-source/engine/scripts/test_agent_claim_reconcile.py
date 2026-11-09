from __future__ import annotations

import contextlib
import io
import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import agent_brief
import agent_claim_guard
import agent_claim_reconcile as reconcile
import agent_next


ASSIGNEE = "agent@example.com"
TRANSITION_COMMENT = "Claimed after final coordination check"


def record(
    issue_id: str,
    *,
    title: str | None = None,
    status: str = "open",
    priority: int = 1,
    assignee: str | None = None,
    updated_at: str = "2026-09-01T00:00:00Z",
    comments: list[dict] | None = None,
    description: str = "stable semantic field",
) -> dict:
    return {
        "id": issue_id,
        "title": title or f"W10: recovery fixture {issue_id}",
        "status": status,
        "priority": priority,
        "issue_type": "task",
        "assignee": assignee,
        "created_at": "2026-08-31T00:00:00Z",
        "updated_at": updated_at,
        "description": description,
        "comments": [] if comments is None else comments,
        "dependencies": [],
    }


class Fixture:
    def __init__(self, rows: list[dict]) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        (self.root / ".git").mkdir()
        (self.root / ".beads").mkdir()
        self.before = self.root / "before.jsonl"
        self.current = self.root / ".beads" / "issues.jsonl"
        self.write(self.before, rows)
        self.write(self.current, rows)

    def cleanup(self) -> None:
        self.temporary.cleanup()

    @staticmethod
    def write(path: Path, rows: list[dict]) -> None:
        path.write_text(
            "".join(
                json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n"
                for row in rows
            ),
            encoding="utf-8",
        )

    def token(self) -> str:
        issues = agent_brief.load_issues(self.before)
        guard = agent_claim_guard.build_guard(
            issues,
            as_of=agent_next.parse_as_of(None, issues),
            stale_days=2,
            activation_cap=4,
            limit=20,
        )
        return guard["token"]

    def reconcile(self, **kwargs):
        return reconcile.reconcile(
            self.root,
            self.token(),
            ASSIGNEE,
            before_ledger=self.before,
            **kwargs,
        )


class AgentClaimReconcileTests(unittest.TestCase):
    def fixture(self, rows: list[dict] | None = None) -> Fixture:
        fixture = Fixture(rows or [record("fm-next", priority=0)])
        self.addCleanup(fixture.cleanup)
        return fixture

    def test_unchanged_graph_is_decisive_but_old_token_is_never_reused(self) -> None:
        fixture = self.fixture()
        report = fixture.reconcile()
        self.assertEqual(report["classification"], reconcile.CLASS_UNCHANGED)
        self.assertEqual(report["reason"], "no-semantic-delta")
        self.assertFalse(report["retry_old_token"])
        self.assertEqual(
            report["next_action"],
            "refresh-coordination-and-issue-new-token",
        )
        self.assertEqual(
            report["baseline"]["graph_sha256"],
            report["current"]["graph_sha256"],
        )

    def test_harmless_row_and_object_order_remain_semantically_unchanged(self) -> None:
        rows = [record("fm-next", priority=0), record("fm-other", priority=2)]
        fixture = self.fixture(rows)
        reordered = [dict(reversed(list(row.items()))) for row in reversed(rows)]
        fixture.write(fixture.current, reordered)
        report = fixture.reconcile()
        self.assertEqual(report["classification"], reconcile.CLASS_UNCHANGED)
        self.assertNotEqual(
            report["baseline"]["ledger_sha256"],
            report["current"]["ledger_sha256"],
        )

    def test_exact_claim_delta_is_proved_from_authenticated_baseline(self) -> None:
        fixture = self.fixture()
        fixture.write(
            fixture.current,
            [
                record(
                    "fm-next",
                    priority=0,
                    status="in_progress",
                    assignee=ASSIGNEE,
                    updated_at="2026-09-01T00:00:01Z",
                )
            ],
        )
        report = fixture.reconcile()
        self.assertEqual(report["classification"], reconcile.CLASS_EXACT)
        self.assertEqual(report["reason"], "exact-exported-claim-delta")
        self.assertEqual(report["next_action"], "continue-with-claimed-work")
        self.assertFalse(report["retry_old_token"])
        self.assertEqual(report["claim_delta"]["changed_issue_ids"], ["fm-next"])
        self.assertFalse(report["claim_delta"]["transition_comment_appended"])

    def test_exact_transition_comment_must_be_supplied_for_exact_proof(self) -> None:
        fixture = self.fixture()
        comments = [{"text": TRANSITION_COMMENT}]
        fixture.write(
            fixture.current,
            [
                record(
                    "fm-next",
                    priority=0,
                    status="in_progress",
                    assignee=ASSIGNEE,
                    updated_at="2026-09-01T00:00:01Z",
                    comments=comments,
                )
            ],
        )
        without = fixture.reconcile()
        self.assertEqual(without["classification"], reconcile.CLASS_AMBIGUOUS)
        with_comment = fixture.reconcile(transition_comment=TRANSITION_COMMENT)
        self.assertEqual(with_comment["classification"], reconcile.CLASS_EXACT)
        self.assertTrue(with_comment["claim_delta"]["transition_comment_appended"])

    def test_different_assignee_is_a_conflicting_claim(self) -> None:
        fixture = self.fixture()
        fixture.write(
            fixture.current,
            [
                record(
                    "fm-next",
                    priority=0,
                    status="in_progress",
                    assignee="peer@example.com",
                    updated_at="2026-09-01T00:00:01Z",
                )
            ],
        )
        report = fixture.reconcile()
        self.assertEqual(report["classification"], reconcile.CLASS_CONFLICT)
        self.assertEqual(
            report["reason"],
            "target-assigned-to-different-actor",
        )
        self.assertEqual(
            report["next_action"],
            "stop-and-coordinate-with-current-assignee",
        )

    def test_expected_target_plus_unrelated_drift_is_ambiguous(self) -> None:
        fixture = self.fixture(
            [record("fm-next", priority=0), record("fm-other", priority=2)]
        )
        fixture.write(
            fixture.current,
            [
                record(
                    "fm-next",
                    priority=0,
                    status="in_progress",
                    assignee=ASSIGNEE,
                    updated_at="2026-09-01T00:00:01Z",
                ),
                record(
                    "fm-other",
                    priority=2,
                    description="concurrently changed semantic field",
                ),
            ],
        )
        report = fixture.reconcile()
        self.assertEqual(report["classification"], reconcile.CLASS_AMBIGUOUS)
        self.assertIn("expected-target-state-with-additional-drift", report["reason"])
        self.assertFalse(report["retry_old_token"])

    def test_unrelated_drift_without_target_claim_is_ambiguous(self) -> None:
        fixture = self.fixture(
            [record("fm-next", priority=0), record("fm-other", priority=2)]
        )
        fixture.write(
            fixture.current,
            [
                record("fm-next", priority=0),
                record("fm-other", priority=2, title="W10: concurrently renamed"),
            ],
        )
        report = fixture.reconcile()
        self.assertEqual(report["classification"], reconcile.CLASS_AMBIGUOUS)
        self.assertIn("baseline-drift-without-target-claim", report["reason"])

    def test_missing_target_is_ambiguous(self) -> None:
        fixture = self.fixture(
            [record("fm-next", priority=0), record("fm-other", priority=2)]
        )
        fixture.write(fixture.current, [record("fm-other", priority=2)])
        report = fixture.reconcile()
        self.assertEqual(report["classification"], reconcile.CLASS_AMBIGUOUS)
        self.assertIn("target-missing-from-current-ledger", report["reason"])
        self.assertIsNone(report["current"]["target"])

    def test_baseline_must_reproduce_the_old_token(self) -> None:
        fixture = self.fixture()
        token = fixture.token()
        fixture.write(
            fixture.before,
            [record("fm-next", priority=0, title="W10: wrong baseline")],
        )
        with self.assertRaises(reconcile.ReconcileError) as raised:
            reconcile.reconcile(
                fixture.root,
                token,
                ASSIGNEE,
                before_ledger=fixture.before,
            )
        self.assertEqual(raised.exception.exit_code, 4)
        self.assertIn("does not reproduce", str(raised.exception))

    def test_git_head_is_the_default_authenticated_baseline(self) -> None:
        if shutil.which("git") is None:
            self.skipTest("git is unavailable")
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        subprocess.run(["git", "init", "-q", root], check=True)
        subprocess.run(["git", "-C", root, "config", "user.name", "Fixture"], check=True)
        subprocess.run(
            ["git", "-C", root, "config", "user.email", "fixture@example.com"],
            check=True,
        )
        (root / ".beads").mkdir()
        ledger = root / ".beads" / "issues.jsonl"
        Fixture.write(ledger, [record("fm-next", priority=0)])
        subprocess.run(["git", "-C", root, "add", ".beads/issues.jsonl"], check=True)
        subprocess.run(["git", "-C", root, "commit", "-qm", "baseline"], check=True)
        before = agent_brief.load_issues(ledger)
        token = agent_claim_guard.build_guard(
            before,
            as_of=agent_next.parse_as_of(None, before),
            stale_days=2,
            activation_cap=4,
            limit=20,
        )["token"]
        Fixture.write(
            ledger,
            [
                record(
                    "fm-next",
                    priority=0,
                    status="in_progress",
                    assignee=ASSIGNEE,
                    updated_at="2026-09-01T00:00:01Z",
                )
            ],
        )
        report = reconcile.reconcile(root, token, ASSIGNEE)
        self.assertEqual(report["classification"], reconcile.CLASS_EXACT)
        self.assertEqual(report["baseline"]["source"], {"kind": "git-ref", "value": "HEAD"})

    def test_human_and_json_rendering_are_bounded_and_stable(self) -> None:
        fixture = self.fixture()
        report = fixture.reconcile()
        human = reconcile.render_human(report)
        self.assertIn("UNCHANGED", human)
        self.assertIn("retry_old_token=false", human)
        payload = json.loads(reconcile.render_json(report))
        self.assertEqual(payload["schema"], reconcile.SCHEMA)
        self.assertEqual(payload["version"], reconcile.SCHEMA_VERSION)
        self.assertFalse(payload["retry_old_token"])
        with mock.patch.object(reconcile, "MAX_OUTPUT_BYTES", 8):
            with self.assertRaises(reconcile.ReconcileError):
                reconcile.render_json(report)

    def test_cli_errors_emit_no_machine_payload(self) -> None:
        fixture = self.fixture()
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = reconcile.main(
                [
                    "--repo",
                    str(fixture.root),
                    "--expect-token",
                    "malformed",
                    "--assignee",
                    ASSIGNEE,
                    "--before-ledger",
                    str(fixture.before),
                    "--format",
                    "json",
                ]
            )
        self.assertEqual(status, 2)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("must have the form", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
