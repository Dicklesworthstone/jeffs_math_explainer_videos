from __future__ import annotations

import contextlib
import datetime as dt
import io
import json
import os
import tempfile
import unittest
from pathlib import Path

import agent_brief


def record(
    issue_id: str,
    title: str,
    status: str,
    *,
    priority: int = 2,
    issue_type: str = "task",
    assignee: str | None = None,
    updated_at: str = "2026-08-30T00:00:00Z",
    blockers: tuple[str, ...] = (),
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
    if blockers:
        row["dependencies"] = [
            {"issue_id": issue_id, "depends_on_id": blocker, "type": "blocks"}
            for blocker in blockers
        ]
    return row


def dependency(issue_id: str, target: str, kind: str = "blocks") -> dict:
    return {"issue_id": issue_id, "depends_on_id": target, "type": kind}


class AgentBriefTests(unittest.TestCase):
    def path(self) -> Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        return Path(directory.name) / "issues.jsonl"

    def write_raw(self, data: bytes) -> Path:
        path = self.path()
        path.write_bytes(data)
        return path

    def write_ledger(self, rows: list[dict], *, final_lf: bool = True) -> Path:
        path = self.path()
        text = "\n".join(json.dumps(row, separators=(",", ":")) for row in rows)
        path.write_text(text + ("\n" if final_lf else ""), encoding="utf-8")
        return path

    def snapshot(self, rows: list[dict], *, cap: int = 4) -> dict:
        issues = agent_brief.load_issues(self.write_ledger(rows))
        return agent_brief.build_snapshot(
            issues,
            as_of=dt.datetime(2026, 8, 31, tzinfo=dt.timezone.utc),
            stale_days=2,
            activation_cap=cap,
            limit=20,
        )

    def test_snapshot_distinguishes_ready_blocked_and_stale_claims(self) -> None:
        rows = [
            record("fm-done", "W1: dependency", "closed"),
            record(
                "fm-active",
                "W10: active",
                "in_progress",
                priority=1,
                assignee="Lilac",
                updated_at="2026-08-20T00:00:00Z",
            ),
            record("fm-ready", "W10: ready", "open", priority=1, blockers=("fm-done",)),
            record("fm-blocked", "W11: blocked", "open", blockers=("fm-missing",)),
        ]
        snapshot = self.snapshot(rows)
        self.assertEqual(snapshot["schema_version"], 5)
        self.assertNotIn("recommendation", snapshot)
        self.assertFalse(snapshot["situational_priority"]["claim_safe"])
        self.assertEqual(
            snapshot["situational_priority"]["claim_surface"],
            "scripts/agent_next.py",
        )
        self.assertEqual(snapshot["activation"]["active_workstreams"], ["W10"])
        self.assertEqual([row["id"] for row in snapshot["ready"]], ["fm-ready"])
        self.assertEqual([row["id"] for row in snapshot["blocked"]], ["fm-blocked"])
        self.assertEqual(snapshot["blocked"][0]["blockers"], ["fm-missing"])
        self.assertEqual([row["id"] for row in snapshot["stale_claims"]], ["fm-active"])
        self.assertFalse(snapshot["integrity"]["within_contract"])
        self.assertEqual(
            snapshot["integrity"]["missing_blockers"],
            [{"issue_id": "fm-blocked", "depends_on_id": "fm-missing"}],
        )
        self.assertIsNone(snapshot["situational_priority"]["issue"])

    def test_situational_priority_prefers_an_active_workstream(self) -> None:
        snapshot = self.snapshot(
            [
                record("fm-active", "W10: active", "in_progress", assignee="agent"),
                record("fm-same-stream", "W10: ready", "open", priority=2),
                record("fm-new-stream", "W11: ready", "open", priority=1),
            ]
        )
        situational = snapshot["situational_priority"]
        self.assertEqual(situational["issue"]["id"], "fm-same-stream")
        self.assertFalse(situational["activates_workstream"])
        self.assertFalse(situational["claim_safe"])
        self.assertIn("already-active W10", situational["reason"])

    def test_assigned_and_epic_work_are_visible_but_not_broad_priority(self) -> None:
        snapshot = self.snapshot(
            [
                record("fm-active", "W10: active", "in_progress", assignee="agent"),
                record(
                    "fm-assigned",
                    "W10: reserved leaf",
                    "open",
                    priority=0,
                    assignee="peer",
                ),
                record("fm-epic", "W10: container", "open", priority=0, issue_type="epic"),
                record("fm-leaf", "W10: free candidate", "open", priority=2),
            ]
        )
        self.assertEqual(snapshot["counts"]["dependency_ready"], 3)
        self.assertEqual(snapshot["counts"]["assigned_ready"], 1)
        self.assertEqual(snapshot["counts"]["container_ready"], 1)
        self.assertEqual([row["id"] for row in snapshot["assigned_ready"]], ["fm-assigned"])
        self.assertEqual([row["id"] for row in snapshot["container_ready"]], ["fm-epic"])
        self.assertEqual([row["id"] for row in snapshot["ready"]], ["fm-leaf"])
        self.assertEqual(snapshot["situational_priority"]["issue"]["id"], "fm-leaf")

    def test_only_assigned_or_epic_work_yields_no_situational_priority(self) -> None:
        snapshot = self.snapshot(
            [
                record("fm-assigned", "W10: reserved", "open", assignee="peer"),
                record("fm-epic", "W10: epic", "open", issue_type="epic"),
            ]
        )
        situational = snapshot["situational_priority"]
        self.assertIsNone(situational["issue"])
        self.assertIn("unassigned non-epic", situational["reason"])

    def test_full_activation_cap_suppresses_new_stream_priority(self) -> None:
        rows = [
            record(f"fm-active-{index}", f"W{index}: active", "in_progress", assignee="agent")
            for index in range(1, 5)
        ]
        rows.append(record("fm-new", "W9: ready", "open", priority=1))
        situational = self.snapshot(rows)["situational_priority"]
        self.assertIsNone(situational["issue"])
        self.assertIn("activation cap is full", situational["reason"])

    def test_unscoped_and_unowned_active_claims_are_visible_but_do_not_consume_cap(self) -> None:
        snapshot = self.snapshot([record("fm-x", "Operational cleanup", "in_progress")])
        self.assertEqual(snapshot["activation"]["count"], 0)
        self.assertEqual(snapshot["counts"]["unowned_active"], 1)
        self.assertEqual(snapshot["counts"]["unscoped_active"], 1)
        self.assertEqual(snapshot["unowned_active"][0]["id"], "fm-x")

    def test_parent_child_edges_do_not_block_broad_readiness_or_integrity(self) -> None:
        row = record("fm-child", "W10: child", "open")
        row["dependencies"] = [dependency("fm-child", "fm-parent", "parent-child")]
        issues = agent_brief.load_issues(self.write_ledger([row]))
        self.assertEqual(agent_brief.unresolved_blockers(issues["fm-child"], issues), ())
        snapshot = agent_brief.build_snapshot(
            issues,
            as_of=dt.datetime(2026, 8, 31, tzinfo=dt.timezone.utc),
            stale_days=2,
            activation_cap=4,
            limit=20,
        )
        self.assertTrue(snapshot["integrity"]["within_contract"])
        self.assertEqual(snapshot["counts"]["missing_links"], 1)
        self.assertEqual(snapshot["situational_priority"]["issue"]["id"], "fm-child")
        self.assertFalse(snapshot["situational_priority"]["claim_safe"])

    def test_dependency_owner_self_reference_and_duplicate_edges_fail_closed(self) -> None:
        wrong_owner = record("fm-child", "W10: child", "open")
        wrong_owner["dependencies"] = [dependency("fm-other", "fm-parent")]
        with self.assertRaisesRegex(agent_brief.BriefError, "owned by 'fm-other'"):
            agent_brief.load_issues(self.write_ledger([wrong_owner]))

        self_edge = record("fm-self", "W10: self", "open")
        self_edge["dependencies"] = [dependency("fm-self", "fm-self")]
        with self.assertRaisesRegex(agent_brief.BriefError, "self-referential"):
            agent_brief.load_issues(self.write_ledger([self_edge]))

        duplicate = record("fm-dup", "W10: duplicate", "open")
        duplicate["dependencies"] = [
            dependency("fm-dup", "fm-parent"),
            dependency("fm-dup", "fm-parent"),
        ]
        with self.assertRaisesRegex(agent_brief.BriefError, "duplicate dependency"):
            agent_brief.load_issues(self.write_ledger([duplicate]))

    def test_unknown_status_is_rejected_before_projection(self) -> None:
        unknown = record("fm-x", "W10: x", "paused")
        with self.assertRaisesRegex(agent_brief.BriefError, "unknown status 'paused'"):
            agent_brief.load_issues(self.write_ledger([unknown]))

    def test_blocking_cycle_suppresses_situational_priority_and_stdout(self) -> None:
        first = record("fm-a", "W10: first", "open", priority=1)
        second = record("fm-b", "W10: second", "open", priority=1)
        first["dependencies"] = [dependency("fm-a", "fm-b")]
        second["dependencies"] = [dependency("fm-b", "fm-a")]
        snapshot = self.snapshot([second, first])
        self.assertEqual(snapshot["integrity"]["blocking_cycles"], [["fm-a", "fm-b"]])
        self.assertFalse(snapshot["integrity"]["within_contract"])
        self.assertIsNone(snapshot["situational_priority"]["issue"])
        self.assertIn("suppress situational", snapshot["situational_priority"]["reason"])

        path = self.write_ledger([second, first])
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = agent_brief.main(
                [
                    "--ledger",
                    str(path),
                    "--as-of",
                    "2026-08-31T00:00:00Z",
                    "--format",
                    "json",
                    "--check",
                ]
            )
        self.assertEqual(status, 1)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("dependency integrity is invalid", stderr.getvalue())

    def test_long_blocking_chain_is_iterative_and_acyclic(self) -> None:
        count = 1_500
        rows = []
        for index in range(count):
            issue_id = f"fm-{index:04d}"
            row = record(issue_id, f"W10: chain {index}", "open", priority=2)
            if index + 1 < count:
                row["dependencies"] = [
                    dependency(issue_id, f"fm-{index + 1:04d}")
                ]
            rows.append(row)
        snapshot = self.snapshot(rows)
        self.assertTrue(snapshot["integrity"]["within_contract"])
        self.assertEqual(snapshot["integrity"]["blocking_cycles"], [])
        self.assertEqual(snapshot["situational_priority"]["issue"]["id"], "fm-1499")

    def test_activation_cap_is_fail_closed(self) -> None:
        rows = [
            record(f"fm-{index}", f"W{index}: active", "in_progress", assignee="agent")
            for index in range(1, 6)
        ]
        path = self.write_ledger(rows)
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = agent_brief.main(
                [
                    "--ledger",
                    str(path),
                    "--as-of",
                    "2026-08-31T00:00:00Z",
                    "--check",
                ]
            )
        self.assertEqual(status, 1)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("active workstream cap breached", stderr.getvalue())

    def test_legacy_next_format_fails_closed_without_reading_the_ledger(self) -> None:
        missing = self.path()
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = agent_brief.main(
                ["--ledger", str(missing), "--format", "next"]
            )
        self.assertEqual(status, 2)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn(
            "was removed because the broad projection is not leaf-safe",
            stderr.getvalue(),
        )
        self.assertIn("scripts/agent_next.py", stderr.getvalue())

    def test_valid_check_emits_the_requested_situational_projection(self) -> None:
        path = self.write_ledger([record("fm-ready", "W10: ready", "open")])
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = agent_brief.main(
                [
                    "--ledger",
                    str(path),
                    "--as-of",
                    "2026-08-31T00:00:00Z",
                    "--format",
                    "json",
                    "--check",
                ]
            )
        self.assertEqual(status, 0)
        self.assertEqual(stderr.getvalue(), "")
        payload = json.loads(stdout.getvalue())
        self.assertTrue(payload["integrity"]["within_contract"])
        self.assertTrue(payload["activation"]["within_cap"])
        self.assertEqual(payload["ready"][0]["id"], "fm-ready")
        self.assertNotIn("recommendation", payload)
        self.assertEqual(
            payload["situational_priority"]["claim_surface"],
            "scripts/agent_next.py",
        )

    def test_duplicate_ids_and_missing_final_lf_are_rejected(self) -> None:
        duplicate = record("fm-x", "W1: x", "open")
        with self.assertRaisesRegex(agent_brief.BriefError, "duplicate issue id"):
            agent_brief.load_issues(self.write_ledger([duplicate, duplicate]))
        with self.assertRaisesRegex(agent_brief.BriefError, "missing its final LF"):
            agent_brief.load_issues(self.write_ledger([duplicate], final_lf=False))

    def test_duplicate_json_keys_are_rejected_at_any_object_depth(self) -> None:
        top_level = (
            b'{"id":"fm-x","id":"fm-y","title":"W10: x","status":"open",'
            b'"priority":1,"issue_type":"task","created_at":"2026-08-01T00:00:00Z",'
            b'"updated_at":"2026-08-31T00:00:00Z"}\n'
        )
        with self.assertRaisesRegex(agent_brief.BriefError, "duplicate JSON object key 'id'"):
            agent_brief.load_issues(self.write_raw(top_level))

        nested = (
            b'{"id":"fm-x","title":"W10: x","status":"open","priority":1,'
            b'"issue_type":"task","created_at":"2026-08-01T00:00:00Z",'
            b'"updated_at":"2026-08-31T00:00:00Z","dependencies":['
            b'{"issue_id":"fm-x","depends_on_id":"fm-y","type":"blocks","type":"parent-child"}]}\n'
        )
        with self.assertRaisesRegex(agent_brief.BriefError, "duplicate JSON object key 'type'"):
            agent_brief.load_issues(self.write_raw(nested))

    def test_optional_arrays_accept_null_but_reject_falsey_non_arrays(self) -> None:
        nullable = record("fm-null", "W10: nullable", "open")
        nullable["dependencies"] = None
        nullable["comments"] = None
        issues = agent_brief.load_issues(self.write_ledger([nullable]))
        self.assertEqual(issues["fm-null"].dependencies, ())
        self.assertEqual(issues["fm-null"].comments, ())

        for field, value in (("dependencies", ""), ("comments", 0), ("comments", {})):
            malformed = record("fm-bad", "W10: bad", "open")
            malformed[field] = value
            with self.assertRaisesRegex(
                agent_brief.BriefError, f"{field} must be an array or null"
            ):
                agent_brief.load_issues(self.write_ledger([malformed]))

    def test_malformed_comments_and_present_invalid_updated_at_are_rejected(self) -> None:
        malformed_comment = record("fm-comment", "W10: comment", "open")
        malformed_comment["comments"] = ["not-an-object"]
        with self.assertRaisesRegex(agent_brief.BriefError, "comment 0 must be an object"):
            agent_brief.load_issues(self.write_ledger([malformed_comment]))

        malformed_text = record("fm-text", "W10: text", "open")
        malformed_text["comments"] = [{"text": 7}]
        with self.assertRaisesRegex(agent_brief.BriefError, "text must be a string or null"):
            agent_brief.load_issues(self.write_ledger([malformed_text]))

        invalid_timestamp = record("fm-time", "W10: time", "open")
        invalid_timestamp["updated_at"] = ""
        with self.assertRaisesRegex(
            agent_brief.BriefError, "updated_at must be a non-empty timestamp"
        ):
            agent_brief.load_issues(self.write_ledger([invalid_timestamp]))

    @unittest.skipUnless(hasattr(os, "O_NOFOLLOW"), "host lacks no-follow open")
    def test_ledger_symlink_is_refused_before_reading(self) -> None:
        target = self.write_ledger([record("fm-x", "W10: x", "open")])
        link = target.with_name("linked.jsonl")
        try:
            link.symlink_to(target)
        except OSError as exc:
            self.skipTest(f"host cannot create symlinks: {exc}")
        with self.assertRaisesRegex(agent_brief.BriefError, "cannot open"):
            agent_brief.load_issues(link)

    def test_markdown_is_situational_and_names_the_claim_surface(self) -> None:
        snapshot = self.snapshot(
            [
                record("fm-active", "W10: active", "in_progress", assignee="agent"),
                record("fm-ready", "W10: ready", "open", priority=1),
            ]
        )
        rendered = agent_brief.render_markdown(snapshot)
        self.assertIn("Read-only projection of `.beads/issues.jsonl`", rendered)
        self.assertIn("Ledger integrity: **clean**", rendered)
        self.assertIn("**1/4** active", rendered)
        self.assertIn("Broad priority", rendered)
        self.assertNotIn("Recommended next", rendered)
        self.assertIn("Broad dependency-ready queue", rendered)
        self.assertIn("`fm-ready`", rendered)
        self.assertIn("Never claim directly from this projection", rendered)
        self.assertIn("`python3 scripts/agent_next.py`", rendered)
        self.assertIn("Beads as authoritative", rendered)


if __name__ == "__main__":
    unittest.main()
