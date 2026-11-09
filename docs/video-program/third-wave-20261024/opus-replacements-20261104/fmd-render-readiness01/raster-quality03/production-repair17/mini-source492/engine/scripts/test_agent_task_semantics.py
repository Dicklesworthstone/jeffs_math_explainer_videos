from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import agent_brief
import agent_claim_guard as guard
import agent_task_semantics as semantics


def record(issue_id: str = "fm-semantic", **overrides) -> dict:
    row = {
        "id": issue_id,
        "title": "W10: bind full task semantics",
        "status": "open",
        "priority": 1,
        "issue_type": "task",
        "assignee": None,
        "created_at": "2026-08-01T00:00:00Z",
        "created_by": "tester",
        "updated_at": "2026-09-01T00:00:00Z",
        "description": "original description",
        "design": "original design",
        "acceptance_criteria": "original acceptance",
        "notes": "original notes",
        "owner": "owner-a",
        "estimate": 5,
        "due_at": "2026-10-01T00:00:00Z",
        "defer_until": "2026-09-05T00:00:00Z",
        "labels": ["W10", "agent"],
        "extension": {"nested": {"b": 2, "a": 1}},
        "dependencies": [],
        "comments": [],
    }
    row.update(overrides)
    return row


class AgentTaskSemanticsTests(unittest.TestCase):
    def ledger(self, rows: list[dict]) -> Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        path = Path(directory.name) / "issues.jsonl"
        self.write(path, rows)
        return path

    def write(self, path: Path, rows: list[dict]) -> None:
        path.write_text(
            "".join(json.dumps(row, separators=(",", ":")) + "\n" for row in rows),
            encoding="utf-8",
        )

    def build(self, path: Path) -> dict:
        issues = agent_brief.load_issues(path)
        return guard.build_guard(
            issues,
            as_of=guard.agent_next.parse_as_of(None, issues),
            stale_days=2,
            activation_cap=4,
            limit=20,
        )

    def graph_digest(self, path: Path) -> str:
        issues = agent_brief.load_issues(path)
        return guard.digest(guard.canonical_graph(issues))

    def test_every_task_semantic_field_invalidates_the_claim_token(self) -> None:
        baseline = record()
        path = self.ledger([baseline])
        original = self.build(path)["token"]
        mutations = {
            "description": "changed description",
            "design": "changed design",
            "acceptance_criteria": "changed acceptance",
            "notes": "changed notes",
            "owner": "owner-b",
            "estimate": 8,
            "due_at": "2026-11-01T00:00:00Z",
            "defer_until": "2026-09-06T00:00:00Z",
            "labels": ["W10", "agent", "security"],
            "extension": {"nested": {"a": 1, "b": 3}},
        }
        for field, value in mutations.items():
            with self.subTest(field=field):
                changed = copy.deepcopy(baseline)
                changed[field] = value
                self.write(path, [changed])
                self.assertNotEqual(self.build(path)["token"], original)

    def test_canonicalization_ignores_only_harmless_ordering(self) -> None:
        dependency_a = {
            "issue_id": "fm-semantic",
            "depends_on_id": "fm-parent-a",
            "type": "related",
            "metadata": {"z": 2, "a": 1},
            "thread_id": "a",
        }
        dependency_b = {
            "issue_id": "fm-semantic",
            "depends_on_id": "fm-parent-b",
            "type": "related",
            "metadata": {"k": "v"},
            "thread_id": "b",
        }
        parents = [
            record("fm-parent-a", title="W10: parent a", status="closed"),
            record("fm-parent-b", title="W10: parent b", status="closed"),
        ]
        first_row = record(
            labels=["zeta", "alpha"],
            extension={"z": 2, "a": {"d": 4, "c": 3}},
            dependencies=[dependency_a, dependency_b],
        )
        second_row = {
            key: copy.deepcopy(value) for key, value in reversed(list(first_row.items()))
        }
        second_row["labels"] = list(reversed(first_row["labels"]))
        second_row["dependencies"] = list(reversed(first_row["dependencies"]))
        second_row["extension"] = {"a": {"c": 3, "d": 4}, "z": 2}
        first = self.ledger([parents[0], first_row, parents[1]])
        second = self.ledger([parents[1], second_row, parents[0]])
        self.assertEqual(self.graph_digest(first), self.graph_digest(second))
        self.assertEqual(self.build(first)["token"], self.build(second)["token"])

    def test_postclaim_allows_only_the_declared_core_transition(self) -> None:
        baseline = record()
        path = self.ledger([baseline])
        before = agent_brief.load_issues(path)
        guard.build_guard(
            before,
            as_of=guard.agent_next.parse_as_of(None, before),
            stale_days=2,
            activation_cap=4,
            limit=20,
        )
        claimed = copy.deepcopy(baseline)
        claimed.update(
            {
                "status": "in_progress",
                "assignee": "agent-a",
                "updated_at": "2026-09-02T00:00:00Z",
                "comments": [{"text": "claimed"}],
            }
        )
        self.write(path, [claimed])
        after = agent_brief.load_issues(path)
        self.assertRegex(guard.graph_digest(after), r"^[0-9a-f]{64}$")

    def test_postclaim_refuses_selected_or_unrelated_semantic_drift(self) -> None:
        baseline = record()
        peer = record("fm-peer", title="W10: peer", priority=2)
        for changed_id in ("fm-semantic", "fm-peer"):
            with self.subTest(changed_id=changed_id):
                path = self.ledger([baseline, peer])
                before = agent_brief.load_issues(path)
                guard.build_guard(
                    before,
                    as_of=guard.agent_next.parse_as_of(None, before),
                    stale_days=2,
                    activation_cap=4,
                    limit=20,
                )
                changed = [copy.deepcopy(baseline), copy.deepcopy(peer)]
                changed[0].update(
                    {
                        "status": "in_progress",
                        "assignee": "agent-a",
                        "updated_at": "2026-09-02T00:00:00Z",
                    }
                )
                target = changed[0] if changed_id == "fm-semantic" else changed[1]
                target["description"] = "drifted after guard"
                self.write(path, changed)
                after = agent_brief.load_issues(path)
                with self.assertRaisesRegex(
                    guard.GuardError,
                    rf"task-semantic field drift.*{changed_id}",
                ):
                    guard.graph_digest(after)

    def test_dependency_metadata_is_bound(self) -> None:
        parent = record("fm-parent", title="W10: parent", status="closed")
        child = record(
            dependencies=[
                {
                    "issue_id": "fm-semantic",
                    "depends_on_id": "fm-parent",
                    "type": "related",
                    "metadata": "{\"reason\":\"first\"}",
                    "thread_id": "thread-a",
                }
            ]
        )
        path = self.ledger([child, parent])
        original = self.build(path)["token"]
        child["dependencies"][0]["metadata"] = "{\"reason\":\"second\"}"
        self.write(path, [child, parent])
        self.assertNotEqual(self.build(path)["token"], original)

    def test_unknown_metadata_depth_is_bounded_before_planning(self) -> None:
        nested: object = "leaf"
        for _ in range(semantics.MAX_JSON_DEPTH + 1):
            nested = {"next": nested}
        path = self.ledger([record(extension=nested)])
        with self.assertRaisesRegex(agent_brief.BriefError, "JSON depth limit"):
            agent_brief.load_issues(path)

    def test_loader_refuses_a_ledger_that_changes_between_projections(self) -> None:
        path = self.ledger([record()])

        def mutating_loader(_path: Path) -> dict[str, agent_brief.Issue]:
            self.write(path, [record(description="changed during load")])
            return {}

        with self.assertRaisesRegex(semantics.SemanticError, "changed while"):
            semantics.load_semantic_issues(path, mutating_loader)

    def test_loader_refuses_a_core_projection_not_derived_from_the_same_bytes(self) -> None:
        path = self.ledger([record()])

        def wrong_loader(_path: Path) -> dict[str, agent_brief.Issue]:
            return {}

        with self.assertRaisesRegex(semantics.SemanticError, "core projection disagrees"):
            semantics.load_semantic_issues(path, wrong_loader)

    def test_semantic_prepass_enforces_the_global_dependency_budget(self) -> None:
        child = record(
            dependencies=[
                {
                    "issue_id": "fm-semantic",
                    "depends_on_id": "fm-parent-a",
                    "type": "related",
                },
                {
                    "issue_id": "fm-semantic",
                    "depends_on_id": "fm-parent-b",
                    "type": "related",
                },
            ]
        )
        path = self.ledger(
            [
                child,
                record("fm-parent-a", status="closed"),
                record("fm-parent-b", status="closed"),
            ]
        )
        with mock.patch.object(agent_brief, "MAX_DEPENDENCIES", 1):
            with self.assertRaisesRegex(semantics.SemanticError, "1-dependency limit"):
                semantics.load_task_semantics(path)

    def test_descriptor_read_failures_are_typed_semantic_errors(self) -> None:
        path = self.ledger([record()])
        with mock.patch.object(semantics.os, "read", side_effect=OSError("injected read failure")):
            with self.assertRaisesRegex(semantics.SemanticError, "cannot read.*injected"):
                semantics.load_task_semantics(path)

    def test_claim_graph_requires_full_semantic_provenance(self) -> None:
        path = self.ledger([record()])
        semantic_issues = agent_brief.load_issues(path)
        self.assertIsInstance(semantic_issues, semantics.SemanticIssues)
        with self.assertRaisesRegex(guard.GuardError, "full task-semantic loader"):
            guard.canonical_graph(dict(semantic_issues))

    def test_semantic_contract_versions_invalidate_the_token(self) -> None:
        path = self.ledger([record()])
        original = self.build(path)["token"]
        patches = (
            mock.patch.object(guard, "CLAIM_GRAPH_VERSION", guard.CLAIM_GRAPH_VERSION + 1),
            mock.patch.object(guard, "CLAIM_INPUT_VERSION", guard.CLAIM_INPUT_VERSION + 1),
            mock.patch.object(semantics, "SCHEMA_VERSION", semantics.SCHEMA_VERSION + 1),
        )
        for patch in patches:
            with self.subTest(patch=patch):
                with patch:
                    self.assertNotEqual(self.build(path)["token"], original)


if __name__ == "__main__":
    unittest.main()
