from __future__ import annotations

import contextlib
import io
import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import agent_brief
import agent_claim
import agent_claim_guard
import agent_next


def record(
    issue_id: str,
    *,
    status: str = "open",
    assignee: str | None = None,
    priority: int = 1,
    dependencies: list[dict] | None = None,
    updated_at: str = "2026-09-01T00:00:00Z",
) -> dict:
    row = {
        "id": issue_id,
        "title": "W10: guarded claim fixture",
        "status": status,
        "priority": priority,
        "issue_type": "task",
        "created_at": "2026-08-01T00:00:00Z",
        "updated_at": updated_at,
    }
    if assignee is not None:
        row["assignee"] = assignee
    if dependencies is not None:
        row["dependencies"] = dependencies
    return row


def dependency(owner: str, target: str, kind: str = "blocks") -> dict:
    return {"issue_id": owner, "depends_on_id": target, "type": kind}


class Fixture:
    def __init__(self, rows: list[dict], *, worktree: bool = False):
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name) / "repo"
        self.root.mkdir()
        (self.root / ".beads").mkdir()
        if worktree:
            self.git_common_dir = Path(self.directory.name) / "git"
            self.git_dir = self.git_common_dir / "worktrees" / "fixture"
            self.git_dir.mkdir(parents=True)
            (self.git_dir / "commondir").write_text("../..\n", encoding="utf-8")
            relative = os.path.relpath(self.git_dir, self.root)
            (self.root / ".git").write_text(f"gitdir: {relative}\n", encoding="utf-8")
        else:
            self.git_dir = self.root / ".git"
            self.git_dir.mkdir()
            self.git_common_dir = self.git_dir
        self.ledger = self.root / ".beads" / "issues.jsonl"
        self.write(rows)

    def write(self, rows: list[dict]) -> None:
        self.ledger.write_text(
            "".join(json.dumps(row, separators=(",", ":")) + "\n" for row in rows),
            encoding="utf-8",
        )

    def rows(self) -> list[dict]:
        return [json.loads(line) for line in self.ledger.read_text(encoding="utf-8").splitlines()]

    def token(self, **policy) -> str:
        issues = agent_brief.load_issues(self.ledger)
        guard = agent_claim_guard.build_guard(
            issues,
            as_of=agent_next.parse_as_of(policy.get("as_of"), issues),
            stale_days=policy.get("stale_days", 2),
            activation_cap=policy.get("activation_cap", 4),
            limit=policy.get("limit", 20),
        )
        return guard["token"]

    def sibling_worktree(self, name: str) -> Path:
        root = Path(self.directory.name) / name
        root.mkdir()
        (root / ".beads").mkdir()
        (root / ".beads" / "issues.jsonl").write_bytes(self.ledger.read_bytes())
        git_dir = self.git_common_dir / "worktrees" / name
        git_dir.mkdir(parents=True)
        (git_dir / "commondir").write_text("../..\n", encoding="utf-8")
        relative = os.path.relpath(git_dir, root)
        (root / ".git").write_text(f"gitdir: {relative}\n", encoding="utf-8")
        return root

    def cleanup(self) -> None:
        self.directory.cleanup()


class FakeRunner:
    def __init__(
        self,
        fixture: Fixture,
        *,
        update_code: int = 0,
        sync_code: int = 0,
        mutate: bool = True,
        update_stdout: bytes | None = None,
        update_stderr: bytes = b"",
        sync_stdout: bytes = b"",
        sync_stderr: bytes = b"",
        envelope: bool = False,
        warnings: list[dict] | None = None,
        response_updated_at: str | None = None,
        after_update=None,
        after_sync=None,
    ):
        self.fixture = fixture
        self.update_code = update_code
        self.sync_code = sync_code
        self.mutate = mutate
        self.update_stdout = update_stdout
        self.update_stderr = update_stderr
        self.sync_stdout = sync_stdout
        self.sync_stderr = sync_stderr
        self.envelope = envelope
        self.warnings = warnings or []
        self.response_updated_at = response_updated_at
        self.after_update = after_update
        self.after_sync = after_sync
        self.calls: list[tuple[tuple[str, ...], Path]] = []

    @staticmethod
    def command(argv: tuple[str, ...]) -> str:
        if "update" in argv:
            return "update"
        if "sync" in argv:
            return "sync"
        raise AssertionError(f"unexpected command: {argv}")

    def __call__(self, argv: tuple[str, ...], cwd: Path) -> agent_claim.CommandResult:
        self.calls.append((argv, cwd))
        command = self.command(argv)
        if command == "update":
            update_index = argv.index("update")
            issue_id = argv[update_index + 1]
            assignee = argv[argv.index("--actor") + 1]
            if self.update_code == 0 and self.mutate:
                rows = self.fixture.rows()
                for row in rows:
                    if row["id"] == issue_id:
                        row["status"] = "in_progress"
                        row["assignee"] = assignee
                        row["updated_at"] = "2026-09-01T00:00:01Z"
                        if "--transition-comment" in argv:
                            comment = argv[argv.index("--transition-comment") + 1]
                            row.setdefault("comments", []).append({"text": comment})
                self.fixture.write(rows)
            if self.after_update is not None:
                self.after_update(self.fixture, argv)
            stdout = self.update_stdout
            if stdout is None and self.update_code == 0:
                row = next(row for row in self.fixture.rows() if row["id"] == issue_id)
                updated = {
                    "id": row["id"],
                    "title": row["title"],
                    "status": row["status"],
                    "priority": row["priority"],
                    "assignee": row.get("assignee"),
                    "owner": None,
                    "updated_at": self.response_updated_at or row["updated_at"],
                }
                document = (
                    {"updated": [updated], "warnings": self.warnings}
                    if self.envelope
                    else [updated]
                )
                stdout = json.dumps(document, separators=(",", ":")).encode()
            return agent_claim.CommandResult(
                self.update_code,
                stdout or b"",
                self.update_stderr,
            )
        if self.after_sync is not None:
            self.after_sync(self.fixture, argv)
        return agent_claim.CommandResult(
            self.sync_code,
            self.sync_stdout,
            self.sync_stderr,
        )


class AgentClaimTests(unittest.TestCase):
    def fixture(self, rows: list[dict], *, worktree: bool = False) -> Fixture:
        fixture = Fixture(rows, worktree=worktree)
        self.addCleanup(fixture.cleanup)
        return fixture

    def test_dry_run_pins_atomic_claim_argv_and_versioned_policy(self) -> None:
        fixture = self.fixture([record("fm-next")])
        runner = FakeRunner(fixture)
        receipt = agent_claim.execute_claim(
            fixture.root,
            fixture.token(),
            "agent@example.com",
            requested_issue="fm-next",
            transition_comment="claiming after reservation check",
            command_timeout_seconds=17.5,
            command_output_budget_bytes=4096,
            dry_run=True,
            runner=runner,
        )
        self.assertEqual(runner.calls, [])
        self.assertEqual(receipt["schema"], "fmn.agent.claim")
        self.assertEqual(receipt["version"], 6)
        self.assertEqual(receipt["mode"], "dry-run")
        self.assertFalse(receipt["claimed"])
        self.assertEqual(
            receipt["executor_policy"],
            {
                "beads_claim_mode": "beads.update.claim/v1",
                "beads_response_json_depth": 64,
                "beads_response_json_nodes": 100_000,
                "command_timeout_seconds": 17.5,
                "command_output_bytes_per_stream": agent_claim.MAX_COMMAND_OUTPUT_BYTES,
                "command_output_budget_bytes_total": 4096,
            },
        )
        self.assertEqual(
            receipt["commands"],
            [
                [
                    "br",
                    "update",
                    "fm-next",
                    "--claim",
                    "--actor",
                    "agent@example.com",
                    "--json",
                    "--transition-comment",
                    "claiming after reservation check",
                ],
                ["br", "sync", "--flush-only"],
            ],
        )
        update = receipt["commands"][0]
        self.assertIn("--claim", update)
        self.assertNotIn("--status", update)
        self.assertNotIn("--assignee", update)
        self.assertEqual(fixture.rows()[0]["status"], "open")

    def test_claim_requires_atomic_json_then_flushes_and_verifies_exact_delta(self) -> None:
        fixture = self.fixture([record("fm-next")])
        runner = FakeRunner(fixture)
        receipt = agent_claim.execute_claim(
            fixture.root,
            fixture.token(),
            "agent@example.com",
            runner=runner,
        )
        self.assertEqual([runner.command(call[0]) for call in runner.calls], ["update", "sync"])
        self.assertTrue(all(call[1] == fixture.root for call in runner.calls))
        self.assertTrue(receipt["claimed"])
        self.assertEqual(receipt["status"], "in_progress")
        self.assertEqual(
            receipt["atomic_claim"],
            {
                "mode": "beads.update.claim/v1",
                "shape": "array",
                "issue_id": "fm-next",
                "status": "in_progress",
                "assignee": "agent@example.com",
                "updated_at": "2026-09-01T00:00:01Z",
                "warning_count": 0,
            },
        )
        self.assertEqual(receipt["claim_delta"]["changed_issue_ids"], ["fm-next"])
        self.assertEqual(receipt["claim_delta"]["status_before"], "open")
        self.assertEqual(receipt["claim_delta"]["status_after"], "in_progress")
        self.assertFalse(receipt["claim_delta"]["transition_comment_appended"])
        self.assertRegex(receipt["before_claim_sha256"], r"^[0-9a-f]{64}$")
        self.assertRegex(receipt["before_graph_sha256"], r"^[0-9a-f]{64}$")
        self.assertRegex(receipt["after_graph_sha256"], r"^[0-9a-f]{64}$")

    def test_capacity_warning_envelope_is_accepted_and_counted(self) -> None:
        fixture = self.fixture([record("fm-next")])
        runner = FakeRunner(
            fixture,
            envelope=True,
            warnings=[{"capacity_kind": "actor", "message": "near limit"}],
        )
        receipt = agent_claim.execute_claim(
            fixture.root,
            fixture.token(),
            "agent@example.com",
            runner=runner,
        )
        self.assertEqual(receipt["atomic_claim"]["shape"], "updated-envelope")
        self.assertEqual(receipt["atomic_claim"]["warning_count"], 1)

    def test_atomic_json_response_is_strict_and_semantically_bound(self) -> None:
        fixture = self.fixture([record("fm-next")])
        issue = agent_brief.load_issues(fixture.ledger)["fm-next"]
        valid = {
            "id": "fm-next",
            "title": issue.title,
            "status": "in_progress",
            "priority": issue.priority,
            "assignee": "agent@example.com",
            "owner": None,
            "updated_at": "2026-09-01T00:00:01Z",
        }
        failures = {
            "empty": b"",
            "invalid": b"not json",
            "duplicate": b'[{"id":"fm-next","id":"fm-other"}]',
            "nonfinite": b'[{"id":"fm-next","priority":NaN}]',
            "multiple": json.dumps([valid, valid]).encode(),
            "wrong id": json.dumps([{**valid, "id": "fm-other"}]).encode(),
            "wrong status": json.dumps([{**valid, "status": "open"}]).encode(),
            "wrong assignee": json.dumps([{**valid, "assignee": "peer"}]).encode(),
            "wrong title": json.dumps([{**valid, "title": "other"}]).encode(),
            "wrong priority": json.dumps([{**valid, "priority": 4}]).encode(),
            "bad timestamp": json.dumps([{**valid, "updated_at": "soon"}]).encode(),
            "bad warnings": json.dumps({"updated": [valid], "warnings": {}}).encode(),
        }
        for name, payload in failures.items():
            with self.subTest(name=name):
                with self.assertRaises(agent_claim.ClaimError) as raised:
                    agent_claim._verify_atomic_claim_response(
                        agent_claim.CommandResult(0, payload, b""),
                        issue,
                        "agent@example.com",
                    )
                self.assertEqual(raised.exception.exit_code, 5)
        with self.assertRaisesRegex(agent_claim.ClaimError, "unexpected stderr"):
            agent_claim._verify_atomic_claim_response(
                agent_claim.CommandResult(
                    0,
                    json.dumps([valid]).encode(),
                    b"warning",
                ),
                issue,
                "agent@example.com",
            )

    def test_atomic_json_response_has_depth_node_number_and_unicode_bounds(self) -> None:
        exact_depth = ("[" * 63 + "0" + "]" * 63).encode()
        self.assertIsNotNone(
            agent_claim._strict_command_json(exact_depth, "br update --claim")
        )
        excessive_depth = ("[" * 64 + "0" + "]" * 64).encode()
        with self.assertRaisesRegex(agent_claim.ClaimError, "64-level JSON depth limit"):
            agent_claim._strict_command_json(excessive_depth, "br update --claim")

        with mock.patch.object(agent_claim, "MAX_COMMAND_JSON_NODES", 3):
            self.assertEqual(
                agent_claim._strict_command_json(b"[1,2]", "br update --claim"),
                [1, 2],
            )
            with self.assertRaisesRegex(agent_claim.ClaimError, "3-node JSON limit"):
                agent_claim._strict_command_json(b"[1,2,3]", "br update --claim")

        with self.assertRaisesRegex(agent_claim.ClaimError, "non-finite JSON number"):
            agent_claim._strict_command_json(b"1e999", "br update --claim")
        with self.assertRaisesRegex(agent_claim.ClaimError, "Unicode surrogate"):
            agent_claim._strict_command_json(b'"\\ud800"', "br update --claim")
        self.assertEqual(
            agent_claim._strict_command_json(
                b'"\\ud83d\\ude00"', "br update --claim"
            ),
            "😀",
        )

        parser_deep = ("[" * 1500 + "0" + "]" * 1500).encode()
        with self.assertRaisesRegex(
            agent_claim.ClaimError,
            "JSON parser resource limits|JSON depth limit",
        ):
            agent_claim._strict_command_json(parser_deep, "br update --claim")

    def test_malformed_success_response_stops_before_sync(self) -> None:
        fixture = self.fixture([record("fm-next")])
        runner = FakeRunner(fixture, update_stdout=b"not json")
        with self.assertRaises(agent_claim.ClaimError) as raised:
            agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                runner=runner,
            )
        self.assertEqual(raised.exception.exit_code, 5)
        self.assertIn("not valid JSON", str(raised.exception))
        self.assertEqual([runner.command(call[0]) for call in runner.calls], ["update"])

    def test_response_timestamp_must_match_exported_ledger(self) -> None:
        fixture = self.fixture([record("fm-next")])
        runner = FakeRunner(
            fixture,
            response_updated_at="2026-09-01T00:00:02Z",
        )
        with self.assertRaisesRegex(agent_claim.ClaimError, "timestamp disagreed"):
            agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                runner=runner,
            )

    def test_exact_transition_comment_is_the_only_optional_claim_delta(self) -> None:
        fixture = self.fixture([record("fm-next")])
        runner = FakeRunner(fixture)
        receipt = agent_claim.execute_claim(
            fixture.root,
            fixture.token(),
            "agent@example.com",
            transition_comment="coordinated claim",
            runner=runner,
        )
        self.assertTrue(receipt["claimed"])
        self.assertTrue(receipt["claim_delta"]["transition_comment_appended"])
        self.assertEqual(
            receipt["claim_delta"]["allowed_fields"],
            ["assignee", "status", "updated_at", "comments"],
        )

    def test_postcondition_rejects_unrelated_membership_and_target_drift(self) -> None:
        def unrelated(fixture: Fixture, _argv) -> None:
            rows = fixture.rows()
            rows[1]["title"] = "W10: concurrently changed"
            rows[1]["updated_at"] = "2026-09-01T00:00:02Z"
            fixture.write(rows)

        fixture = self.fixture([record("fm-next"), record("fm-other", priority=2)])
        with self.assertRaisesRegex(agent_claim.ClaimError, "unrelated graph drift"):
            agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                runner=FakeRunner(fixture, after_sync=unrelated),
            )

        def membership(fixture: Fixture, _argv) -> None:
            rows = fixture.rows()
            rows.append(record("fm-concurrent", priority=3))
            fixture.write(rows)

        fixture = self.fixture([record("fm-next")])
        with self.assertRaisesRegex(agent_claim.ClaimError, "issue-membership drift"):
            agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                runner=FakeRunner(fixture, after_sync=membership),
            )

        def target(fixture: Fixture, _argv) -> None:
            rows = fixture.rows()
            rows[0]["priority"] = 3
            fixture.write(rows)

        fixture = self.fixture([record("fm-next")])
        with self.assertRaisesRegex(agent_claim.ClaimError, "non-claim fields"):
            agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                runner=FakeRunner(fixture, after_sync=target),
            )

    def test_stale_token_issue_mismatch_and_integrity_fail_before_br(self) -> None:
        fixture = self.fixture([record("fm-next"), record("fm-other", priority=2)])
        token = fixture.token()
        rows = fixture.rows()
        rows[0]["title"] = "W10: changed after token issuance"
        rows[0]["updated_at"] = "2026-09-01T00:00:02Z"
        fixture.write(rows)
        runner = FakeRunner(fixture)
        with self.assertRaises(agent_claim.ClaimError) as stale:
            agent_claim.execute_claim(fixture.root, token, "agent@example.com", runner=runner)
        self.assertEqual(stale.exception.exit_code, 4)
        self.assertEqual(runner.calls, [])

        with self.assertRaises(agent_claim.ClaimError) as mismatch:
            agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                requested_issue="fm-other",
                runner=runner,
            )
        self.assertEqual(mismatch.exception.exit_code, 2)
        self.assertEqual(runner.calls, [])

        bad = self.fixture(
            [record("fm-bad", dependencies=[dependency("fm-bad", "fm-missing")])]
        )
        with self.assertRaises(agent_claim.ClaimError) as integrity:
            agent_claim.execute_claim(
                bad.root,
                "malformed",
                "agent@example.com",
                dry_run=True,
            )
        self.assertEqual(integrity.exception.exit_code, 1)

    def test_no_recommendation_update_failure_sync_failure_and_missing_mutation(self) -> None:
        assigned = self.fixture([record("fm-assigned", assignee="peer@example.com")])
        with self.assertRaises(agent_claim.ClaimError) as no_work:
            agent_claim.execute_claim(
                assigned.root,
                assigned.token(),
                "agent@example.com",
                dry_run=True,
            )
        self.assertEqual(no_work.exception.exit_code, 3)

        fixture = self.fixture([record("fm-next")])
        failed = FakeRunner(fixture, update_code=9, update_stderr=b"policy refused")
        with self.assertRaisesRegex(agent_claim.ClaimError, "policy refused"):
            agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                runner=failed,
            )
        self.assertEqual([failed.command(call[0]) for call in failed.calls], ["update"])

        fixture = self.fixture([record("fm-next")])
        failed = FakeRunner(fixture, sync_code=7, sync_stderr=b"flush failed")
        with self.assertRaisesRegex(agent_claim.ClaimError, "flush failed"):
            agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                runner=failed,
            )
        self.assertEqual([failed.command(call[0]) for call in failed.calls], ["update", "sync"])

        fixture = self.fixture([record("fm-next")])
        failed = FakeRunner(fixture, mutate=False)
        with self.assertRaisesRegex(agent_claim.ClaimError, "did not report status"):
            agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                runner=failed,
            )

    def test_worktrees_share_one_persistent_lock(self) -> None:
        fixture = self.fixture([record("fm-next")], worktree=True)
        receipt = agent_claim.execute_claim(
            fixture.root,
            fixture.token(),
            "agent@example.com",
            dry_run=True,
        )
        self.assertFalse(receipt["claimed"])
        lock = fixture.git_common_dir / agent_claim.LOCK_FILE_NAME
        self.assertTrue(lock.is_file())
        self.assertEqual(lock.read_bytes(), b"0")
        self.assertFalse((fixture.git_dir / agent_claim.LOCK_FILE_NAME).exists())

    @unittest.skipIf(os.name == "nt", "fcntl contention fixture is Unix-specific")
    def test_linked_worktrees_and_local_callers_contend_before_mutation(self) -> None:
        fixture = self.fixture([record("fm-next")], worktree=True)
        sibling = fixture.sibling_worktree("sibling")
        self.assertEqual(
            agent_claim.resolve_git_common_dir(fixture.root),
            agent_claim.resolve_git_common_dir(sibling),
        )
        with agent_claim.claim_lock(fixture.root):
            with self.assertRaises(agent_claim.ClaimError) as raised:
                with agent_claim.claim_lock(sibling):
                    self.fail("sibling worktree acquired the shared claim lock")
        self.assertEqual(raised.exception.exit_code, 5)

        token = fixture.token()
        with agent_claim.claim_lock(fixture.root):
            with self.assertRaises(agent_claim.ClaimError) as raised:
                agent_claim.execute_claim(
                    fixture.root,
                    token,
                    "agent@example.com",
                    dry_run=True,
                )
        self.assertEqual(raised.exception.exit_code, 5)

    def test_runner_output_receipt_and_configured_budget_bounds_fail_closed(self) -> None:
        fixture = self.fixture([record("fm-next")])
        oversized = FakeRunner(
            fixture,
            update_code=1,
            update_stderr=b"x" * (agent_claim.MAX_COMMAND_OUTPUT_BYTES + 1),
        )
        with self.assertRaisesRegex(agent_claim.ClaimError, "command-output limit"):
            agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                runner=oversized,
            )

        combined = FakeRunner(
            fixture,
            update_code=1,
            update_stdout=b"x" * 60,
            update_stderr=b"y" * 60,
        )
        with self.assertRaisesRegex(agent_claim.ClaimError, "100 total output bytes"):
            agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                command_output_budget_bytes=100,
                runner=combined,
            )

        for value, message in (
            (0, "must be positive"),
            (-1, "must be positive"),
            (True, "positive integer"),
            (agent_claim.MAX_COMMAND_OUTPUT_BUDGET_BYTES + 1, "exceeds the"),
        ):
            with self.subTest(value=value):
                with self.assertRaisesRegex(agent_claim.ClaimError, message):
                    agent_claim.execute_claim(
                        fixture.root,
                        fixture.token(),
                        "agent@example.com",
                        command_output_budget_bytes=value,
                        dry_run=True,
                    )

        with mock.patch.object(agent_claim, "MAX_OUTPUT_BYTES", 8):
            with self.assertRaises(agent_claim.ClaimError) as raised:
                agent_claim.render_json({"schema": "too-large"})
        self.assertEqual(raised.exception.exit_code, 5)

    def test_cli_failures_publish_no_stdout(self) -> None:
        fixture = self.fixture([record("fm-next")])
        token = fixture.token()
        rows = fixture.rows()
        rows[0]["updated_at"] = "2026-09-01T00:00:05Z"
        fixture.write(rows)
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = agent_claim.main(
                [
                    "--repo",
                    str(fixture.root),
                    "--expect-token",
                    token,
                    "--assignee",
                    "agent@example.com",
                    "--dry-run",
                ]
            )
        self.assertEqual(status, 4)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("claim token is stale", stderr.getvalue())

        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = agent_claim.main(
                [
                    "--repo",
                    str(fixture.root),
                    "--expect-token",
                    fixture.token(),
                    "--assignee",
                    "agent@example.com",
                    "--command-output-budget-bytes",
                    "0",
                    "--dry-run",
                ]
            )
        self.assertEqual(status, 2)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("must be positive", stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
