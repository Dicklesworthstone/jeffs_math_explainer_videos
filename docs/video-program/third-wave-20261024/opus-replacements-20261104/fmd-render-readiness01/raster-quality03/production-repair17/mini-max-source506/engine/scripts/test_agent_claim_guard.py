from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import agent_claim_guard as guard


def record(
    issue_id: str,
    *,
    title: str = "W10: ready",
    status: str = "open",
    priority: int = 2,
    issue_type: str = "task",
    assignee: str | None = None,
    updated_at: str = "2026-08-31T08:15:00Z",
    dependencies: list[dict] | None = None,
    comments: list[dict] | None = None,
) -> dict:
    row = {
        "id": issue_id,
        "title": title,
        "status": status,
        "priority": priority,
        "issue_type": issue_type,
        "created_at": "2026-08-01T00:00:00Z",
        "updated_at": updated_at,
    }
    if assignee is not None:
        row["assignee"] = assignee
    if dependencies is not None:
        row["dependencies"] = dependencies
    if comments is not None:
        row["comments"] = comments
    return row


def dependency(owner: str, target: str, kind: str = "blocks") -> dict:
    return {"issue_id": owner, "depends_on_id": target, "type": kind}


class AgentClaimGuardTests(unittest.TestCase):
    def ledger(self, rows: list[dict]) -> Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        path = Path(directory.name) / "issues.jsonl"
        path.write_text(
            "".join(json.dumps(row, separators=(",", ":")) + "\n" for row in rows),
            encoding="utf-8",
        )
        return path

    def invoke(self, path: Path, *arguments: str) -> tuple[int, str, str]:
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = guard.main(["--ledger", str(path), *arguments])
        return status, stdout.getvalue(), stderr.getvalue()

    def token(self, path: Path, *arguments: str) -> str:
        status, output, error = self.invoke(path, "--format", "token", *arguments)
        self.assertEqual(status, 0, error)
        self.assertEqual(error, "")
        return output.strip()

    def assert_stale(self, path: Path, token: str, *arguments: str) -> None:
        status, output, error = self.invoke(
            path,
            "--expect-token",
            token,
            *arguments,
        )
        self.assertEqual(status, 4, error)
        self.assertEqual(output, "")
        self.assertIn("claim token is stale", error)

    def test_v2_token_round_trip_binds_graph_policy_and_schema_contract(self) -> None:
        path = self.ledger([record("fm-next", priority=1)])
        token = self.token(path, "--require")
        self.assertRegex(token, r"^v2:[0-9a-f]{64}:fm-next$")

        status, output, error = self.invoke(
            path,
            "--format",
            "json",
            "--expect-token",
            token,
            "--require",
        )
        self.assertEqual(status, 0)
        self.assertEqual(error, "")
        payload = json.loads(output)
        digest = token.split(":", 2)[1]
        self.assertEqual(payload["schema"], "fmn.agent.claim-guard")
        self.assertEqual(payload["version"], 2)
        self.assertEqual(payload["token"], token)
        self.assertEqual(payload["claim_sha256"], digest)
        self.assertRegex(payload["graph_sha256"], r"^[0-9a-f]{64}$")
        self.assertNotEqual(payload["claim_sha256"], payload["graph_sha256"])
        self.assertEqual(payload["recommendation_id"], "fm-next")
        self.assertEqual(
            payload["policy"],
            {
                "activation_cap": 4,
                "as_of": "2026-08-31T08:15:00Z",
                "limit": 20,
                "stale_days": 2,
            },
        )
        self.assertEqual(
            payload["schemas"],
            {
                "agent_brief_snapshot_version": guard.agent_brief.SNAPSHOT_SCHEMA_VERSION,
                "agent_next": {
                    "schema": guard.agent_next.SCHEMA,
                    "version": guard.agent_next.SCHEMA_VERSION,
                },
                "claim_graph": {
                    "schema": guard.CLAIM_GRAPH_SCHEMA,
                    "version": guard.CLAIM_GRAPH_VERSION,
                },
                "claim_guard": {
                    "schema": guard.SCHEMA,
                    "version": guard.SCHEMA_VERSION,
                },
                "claim_input": {
                    "schema": guard.CLAIM_INPUT_SCHEMA,
                    "version": guard.CLAIM_INPUT_VERSION,
                },
                "token_version": guard.TOKEN_VERSION,
            },
        )

    def test_scope_policy_propagates_through_guard_and_token_publication(self) -> None:
        unscoped = self.ledger(
            [record("fm-unscoped", title="Maintenance without a workstream", priority=0)]
        )
        token = self.token(unscoped)
        self.assertRegex(token, r"^v2:[0-9a-f]{64}:none$")
        status, output, error = self.invoke(unscoped, "--require")
        self.assertEqual(status, 3)
        self.assertEqual(output, "")
        self.assertIn("no claimable recommendation", error)

        governed = self.ledger(
            [
                record(
                    "fm-g0-active",
                    title="G0: active",
                    status="in_progress",
                    assignee="agent",
                ),
                record("fm-g0-next", title="G0: next", priority=4),
                record("fm-w11", title="W11: new stream", priority=1),
                record("fm-w12", title="W12: invalid", priority=0),
            ]
        )
        token = self.token(governed, "--require")
        self.assertTrue(token.endswith(":fm-g0-next"))
        status, output, error = self.invoke(governed, "--format", "json", "--require")
        self.assertEqual(status, 0)
        self.assertEqual(error, "")
        payload = json.loads(output)
        self.assertEqual(payload["recommendation_id"], "fm-g0-next")
        self.assertEqual(payload["activation"]["active_workstreams"], ["G0"])
        self.assertEqual(payload["schemas"]["agent_next"]["version"], 4)

        invalid = self.ledger(
            [
                record(
                    "fm-unscoped-active",
                    title="Maintenance without a workstream",
                    status="in_progress",
                    assignee="agent",
                ),
                record("fm-scoped", title="W10: governed"),
            ]
        )
        status, output, error = self.invoke(invalid, "--format", "json")
        self.assertEqual(status, 1)
        self.assertEqual(output, "")
        self.assertIn("task-graph integrity failed", error)

    def test_any_authoritative_graph_change_invalidates_the_token(self) -> None:
        path = self.ledger([record("fm-next", comments=[{"text": "first"}])])
        token = self.token(path)

        path.write_text(
            json.dumps(
                record("fm-next", comments=[{"text": "first"}, {"text": "new handoff"}]),
                separators=(",", ":"),
            )
            + "\n",
            encoding="utf-8",
        )
        self.assert_stale(path, token)

    def test_recommendation_change_invalidates_the_token(self) -> None:
        path = self.ledger(
            [
                record("fm-first", priority=1),
                record("fm-second", priority=2),
            ]
        )
        token = self.token(path)
        self.assertTrue(token.endswith(":fm-first"))

        path.write_text(
            "".join(
                json.dumps(row, separators=(",", ":")) + "\n"
                for row in [
                    record("fm-first", priority=3),
                    record("fm-second", priority=1),
                ]
            ),
            encoding="utf-8",
        )
        self.assert_stale(path, token)

    def test_digest_is_independent_of_issue_and_dependency_row_order(self) -> None:
        parent = record("fm-parent", status="closed")
        child_dependencies = [
            dependency("fm-child", "fm-parent", "blocks"),
            dependency("fm-child", "fm-container", "parent-child"),
        ]
        child = record("fm-child", dependencies=child_dependencies)
        container = record("fm-container", issue_type="epic")
        first = self.ledger([parent, child, container])
        second = self.ledger(
            [
                container,
                record("fm-child", dependencies=list(reversed(child_dependencies))),
                parent,
            ]
        )
        first_issues = guard.agent_brief.load_issues(first)
        second_issues = guard.agent_brief.load_issues(second)
        self.assertEqual(guard.graph_digest(first_issues), guard.graph_digest(second_issues))
        self.assertEqual(self.token(first), self.token(second))

    def test_every_policy_input_invalidates_an_otherwise_identical_claim(self) -> None:
        path = self.ledger([record("fm-next", priority=1)])
        token = self.token(path)
        for arguments in (
            ("--as-of", "2026-09-01T00:00:00Z"),
            ("--stale-days", "3"),
            ("--activation-cap", "5"),
            ("--limit", "21"),
        ):
            with self.subTest(arguments=arguments):
                self.assert_stale(path, token, *arguments)

    def test_schema_version_changes_invalidate_an_otherwise_identical_claim(self) -> None:
        path = self.ledger([record("fm-next", priority=1)])
        token = self.token(path)
        patches = (
            mock.patch.object(
                guard.agent_brief,
                "SNAPSHOT_SCHEMA_VERSION",
                guard.agent_brief.SNAPSHOT_SCHEMA_VERSION + 1,
            ),
            mock.patch.object(
                guard.agent_next,
                "SCHEMA_VERSION",
                guard.agent_next.SCHEMA_VERSION + 1,
            ),
            mock.patch.object(guard, "SCHEMA_VERSION", guard.SCHEMA_VERSION + 1),
        )
        for patch in patches:
            with self.subTest(patch=patch):
                with patch:
                    self.assert_stale(path, token)

    def test_full_planner_output_is_bound_even_when_recommendation_is_unchanged(self) -> None:
        path = self.ledger([record("fm-next", priority=1)])
        token = self.token(path)
        original = guard.agent_next.build_plan

        def changed_plan(*args, **kwargs):
            plan = original(*args, **kwargs)
            plan["planner_semantics_fixture"] = "changed"
            return plan

        with mock.patch.object(guard.agent_next, "build_plan", side_effect=changed_plan):
            self.assert_stale(path, token)

    def test_no_recommendation_has_a_stable_none_token_and_require_exit(self) -> None:
        path = self.ledger([record("fm-assigned", assignee="peer", priority=1)])
        first = self.invoke(path, "--format", "token")
        second = self.invoke(path, "--format", "token")
        self.assertEqual(first, second)
        self.assertEqual(first[0], 0)
        self.assertRegex(first[1].strip(), r"^v2:[0-9a-f]{64}:none$")

        status, output, error = self.invoke(path, "--require")
        self.assertEqual(status, 3)
        self.assertEqual(output, "")
        self.assertIn("no claimable recommendation", error)

    def test_literal_none_issue_id_is_reserved_and_never_ambiguous(self) -> None:
        path = self.ledger([record("none", priority=1)])
        status, output, error = self.invoke(path)
        self.assertEqual(status, 2)
        self.assertEqual(output, "")
        self.assertIn("reserved", error)
        self.assertIn("no-recommendation token sentinel", error)

    def test_malformed_legacy_token_and_output_budget_fail_without_stdout(self) -> None:
        path = self.ledger([record("fm-next")])
        for malformed in (
            "not-a-token",
            "v1:" + "0" * 64 + ":fm-next",
            "v2:" + "A" * 64 + ":fm-next",
        ):
            with self.subTest(malformed=malformed):
                status, output, error = self.invoke(path, "--expect-token", malformed)
                self.assertEqual(status, 2)
                self.assertEqual(output, "")
                self.assertIn("must have the form v2", error)

        with mock.patch.object(guard, "MAX_OUTPUT_BYTES", 8):
            status, output, error = self.invoke(path, "--format", "json")
        self.assertEqual(status, 2)
        self.assertEqual(output, "")
        self.assertIn("8-byte limit", error)

    def test_integrity_failure_precedes_expected_token_validation(self) -> None:
        path = self.ledger(
            [
                record(
                    "fm-bad",
                    dependencies=[dependency("fm-bad", "fm-absent")],
                )
            ]
        )
        status, output, error = self.invoke(path, "--expect-token", "malformed")
        self.assertEqual(status, 1)
        self.assertEqual(output, "")
        self.assertIn("task-graph integrity failed", error)
        self.assertNotIn("must have the form", error)


if __name__ == "__main__":
    unittest.main()
