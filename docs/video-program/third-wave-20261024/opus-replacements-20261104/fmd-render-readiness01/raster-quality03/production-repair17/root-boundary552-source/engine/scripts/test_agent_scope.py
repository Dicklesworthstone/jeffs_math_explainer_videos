from __future__ import annotations

import datetime as dt
import unittest

import agent_brief
import agent_next


UTC = dt.timezone.utc


def issue(
    issue_id: str,
    title: str,
    *,
    status: str = "open",
    assignee: str | None = None,
) -> agent_brief.Issue:
    return agent_brief.Issue(
        id=issue_id,
        title=title,
        status=status,
        priority=2,
        issue_type="task",
        assignee=assignee,
        updated_at=dt.datetime(2026, 8, 31, tzinfo=UTC),
        dependencies=(),
        comments=(),
    )


class AgentScopeTests(unittest.TestCase):
    def test_broad_and_claim_projections_share_one_classifier(self) -> None:
        self.assertIs(agent_next.governed_workstream, agent_brief.governed_workstream)
        self.assertEqual(agent_next.UNSCOPED, agent_brief.UNSCOPED)

    def test_governed_vocabulary_is_exact_and_anchored(self) -> None:
        accepted = {"G0": "G0: foundation"}
        accepted.update({f"W{number}": f"W{number}: governed" for number in range(1, 12)})
        for expected, title in accepted.items():
            with self.subTest(title=title):
                candidate = issue(f"fm-{expected.lower()}", title)
                self.assertEqual(candidate.workstream, expected)
                self.assertEqual(agent_next.governed_workstream(candidate), expected)

        rejected = (
            "W0: invalid",
            "W12: invalid",
            "W999: invalid",
            "w10: invalid",
            "prefix W10: invalid",
            "W01: invalid",
            "G1: invalid",
        )
        for index, title in enumerate(rejected):
            with self.subTest(title=title):
                candidate = issue(f"fm-invalid-{index}", title)
                self.assertEqual(candidate.workstream, agent_brief.UNSCOPED)
                self.assertEqual(
                    agent_next.governed_workstream(candidate),
                    agent_brief.UNSCOPED,
                )

    def test_activation_views_cannot_disagree_on_scope(self) -> None:
        titles = {
            "fm-g0": "G0: foundation",
            "fm-w1": "W1: governed",
            "fm-w11": "W11: governed",
            "fm-w0": "W0: invalid",
            "fm-w12": "W12: invalid",
            "fm-lower": "w10: invalid",
            "fm-prefix": "prefix W10: invalid",
        }
        issues = {
            issue_id: issue(issue_id, title, status="in_progress", assignee="agent")
            for issue_id, title in titles.items()
        }
        as_of = dt.datetime(2026, 8, 31, tzinfo=UTC)
        brief = agent_brief.build_snapshot(
            issues,
            as_of=as_of,
            stale_days=2,
            activation_cap=4,
            limit=20,
        )
        plan = agent_next.build_plan(
            issues,
            as_of=as_of,
            stale_days=2,
            activation_cap=4,
            limit=20,
        )

        expected_scoped = ["G0", "W1", "W11"]
        expected_unscoped = {"fm-lower", "fm-prefix", "fm-w0", "fm-w12"}
        self.assertEqual(brief["activation"]["active_workstreams"], expected_scoped)
        self.assertEqual(plan["activation"]["active_workstreams"], expected_scoped)
        self.assertEqual(brief["activation"], plan["activation"])
        self.assertEqual(
            {row["id"] for row in brief["unscoped_active"]},
            expected_unscoped,
        )
        self.assertEqual(set(plan["integrity"]["unscoped_active"]), expected_unscoped)


if __name__ == "__main__":
    unittest.main()
