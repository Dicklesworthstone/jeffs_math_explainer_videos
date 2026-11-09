from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path

import generate_agent_brief as generator


def record(
    issue_id: str,
    title: str,
    status: str,
    *,
    updated_at: str,
    priority: int = 2,
    issue_type: str = "task",
    assignee: str | None = None,
    parent: str | None = None,
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
    if parent is not None:
        row["dependencies"] = [
            {"issue_id": issue_id, "depends_on_id": parent, "type": "parent-child"}
        ]
    return row


class GenerateAgentBriefTests(unittest.TestCase):
    def fixture(self) -> tuple[Path, Path]:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        return root / "issues.jsonl", root / "AGENT_BRIEF.md"

    def write_ledger(self, path: Path, rows: list[dict]) -> None:
        path.write_text(
            "".join(json.dumps(row, separators=(",", ":")) + "\n" for row in rows),
            encoding="utf-8",
        )

    def rows(self) -> list[dict]:
        return [
            record(
                "fm-active",
                "W10: active",
                "in_progress",
                updated_at="2026-08-30T12:00:00Z",
                assignee="agent",
            ),
            record(
                "fm-next",
                "W10: ready",
                "open",
                updated_at="2026-08-31T08:15:00Z",
                priority=1,
            ),
        ]

    def test_newest_ledger_timestamp_is_the_deterministic_as_of(self) -> None:
        ledger, _output = self.fixture()
        self.write_ledger(ledger, self.rows())
        document, snapshot = generator.build_document(
            ledger, stale_days=2, activation_cap=4, limit=20
        )
        self.assertEqual(snapshot["schema_version"], 5)
        self.assertEqual(snapshot["claim_plan"]["version"], 4)
        self.assertNotIn("recommendation", snapshot)
        self.assertEqual(snapshot["as_of"], "2026-08-31T08:15:00Z")
        self.assertIn("As of `2026-08-31T08:15:00Z`", document)
        self.assertIn("## Leaf-safe claim plan", document)
        self.assertIn("## Broad dependency-ready queue", document)
        self.assertIn("Claim contract: **none in this projection**", document)
        self.assertIn("**0** unscoped leaves", document)
        self.assertTrue(document.endswith("\n"))
        self.assertFalse(document.endswith("\n\n"))

    def test_human_claim_and_situational_priority_stay_distinct(self) -> None:
        ledger, _output = self.fixture()
        self.write_ledger(
            ledger,
            [
                record(
                    "fm-parent",
                    "W10: non-epic container",
                    "open",
                    updated_at="2026-08-31T08:00:00Z",
                    priority=0,
                ),
                record(
                    "fm-child",
                    "W10: true leaf",
                    "open",
                    updated_at="2026-08-31T08:15:00Z",
                    priority=2,
                    parent="fm-parent",
                ),
            ],
        )
        document, snapshot = generator.build_document(
            ledger, stale_days=2, activation_cap=4, limit=20
        )
        plan = snapshot["claim_plan"]
        situational = snapshot["situational_priority"]
        self.assertEqual(plan["recommendation"]["issue"]["id"], "fm-child")
        self.assertEqual(plan["ready_containers"][0]["id"], "fm-parent")
        self.assertEqual(situational["issue"]["id"], "fm-parent")
        self.assertFalse(situational["claim_safe"])
        self.assertEqual(document.count("Recommended next: **P2 `fm-child`** [W10]"), 1)
        self.assertIn("Broad priority: **P0 `fm-parent`** [W10]", document)
        self.assertNotIn("Recommended next: **P0 `fm-parent`**", document)
        self.assertIn("**1** ready containers (**1** non-epic)", document)
        self.assertIn("broader queue below is situational context only", document)

    def test_unscoped_broad_priority_is_suppressed_but_remains_visible(self) -> None:
        ledger, _output = self.fixture()
        self.write_ledger(
            ledger,
            [
                record(
                    "fm-unscoped",
                    "Maintenance without a workstream",
                    "open",
                    updated_at="2026-08-31T08:15:00Z",
                    priority=0,
                ),
                record(
                    "fm-scoped",
                    "W10: governed leaf",
                    "open",
                    updated_at="2026-08-31T08:14:00Z",
                    priority=3,
                ),
            ],
        )
        document, snapshot = generator.build_document(
            ledger, stale_days=2, activation_cap=4, limit=20
        )
        self.assertEqual(
            snapshot["claim_plan"]["recommendation"]["issue"]["id"],
            "fm-scoped",
        )
        self.assertIsNone(snapshot["situational_priority"]["issue"])
        self.assertIn("scope it before prioritization", snapshot["situational_priority"]["reason"])
        self.assertIn("Recommended next: **P3 `fm-scoped`** [W10]", document)
        self.assertIn("Broad priority: **none**", document)
        self.assertIn("**1** unscoped leaves", document)
        self.assertIn("[UNSCOPED]", document)

    def test_g0_is_scoped_in_the_human_projection(self) -> None:
        ledger, _output = self.fixture()
        self.write_ledger(
            ledger,
            [
                record(
                    "fm-g0-active",
                    "G0: active spike",
                    "in_progress",
                    updated_at="2026-08-31T08:00:00Z",
                    assignee="agent",
                ),
                record(
                    "fm-g0-next",
                    "G0: next spike",
                    "open",
                    updated_at="2026-08-31T08:15:00Z",
                    priority=1,
                ),
            ],
        )
        document, snapshot = generator.build_document(
            ledger, stale_days=2, activation_cap=4, limit=20
        )
        self.assertEqual(snapshot["activation"]["active_workstreams"], ["G0"])
        self.assertEqual(snapshot["counts"]["unscoped_active"], 0)
        self.assertIn("Workstreams: **1/4** active", document)
        self.assertIn("`G0`", document)
        self.assertIn("[G0]", document)
        self.assertNotIn("Unscoped active claim: `fm-g0-active`", document)

    def test_identical_ledger_bytes_produce_identical_document_bytes(self) -> None:
        ledger, _output = self.fixture()
        self.write_ledger(ledger, self.rows())
        first, _ = generator.build_document(ledger, stale_days=2, activation_cap=4, limit=20)
        second, _ = generator.build_document(ledger, stale_days=2, activation_cap=4, limit=20)
        self.assertEqual(first.encode(), second.encode())

    def test_stdout_mode_is_exact_and_never_touches_the_output_path(self) -> None:
        ledger, output = self.fixture()
        self.write_ledger(ledger, self.rows())
        output.write_text("sentinel\n", encoding="utf-8")
        expected, _snapshot = generator.build_document(
            ledger, stale_days=2, activation_cap=4, limit=20
        )
        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = generator.main(
                ["--ledger", str(ledger), "--output", str(output), "--stdout"]
            )
        self.assertEqual(status, 0)
        self.assertEqual(stdout.getvalue(), expected)
        self.assertEqual(stderr.getvalue(), "")
        self.assertEqual(output.read_text(encoding="utf-8"), "sentinel\n")

    def test_generate_then_check_succeeds_and_stale_or_missing_refuses(self) -> None:
        ledger, output = self.fixture()
        self.write_ledger(ledger, self.rows())
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            status = generator.main(["--ledger", str(ledger), "--output", str(output)])
        self.assertEqual(status, 0)
        self.assertTrue(output.is_file())
        self.assertIn("agent brief generated", stdout.getvalue())

        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            status = generator.main(
                ["--ledger", str(ledger), "--output", str(output), "--check"]
            )
        self.assertEqual(status, 0)
        self.assertIn("agent brief current", stdout.getvalue())

        output.write_text("stale\n", encoding="utf-8")
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            status = generator.main(
                ["--ledger", str(ledger), "--output", str(output), "--check"]
            )
        self.assertEqual(status, 1)
        self.assertIn("is stale", stderr.getvalue())
        _unused_ledger, missing_output = self.fixture()
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            status = generator.main(
                ["--ledger", str(ledger), "--output", str(missing_output), "--check"]
            )
        self.assertEqual(status, 1)
        self.assertIn("is missing", stderr.getvalue())

    def test_malformed_or_empty_ledger_never_overwrites_existing_output(self) -> None:
        ledger, output = self.fixture()
        output.write_text("sentinel\n", encoding="utf-8")
        ledger.write_text("not-json\n", encoding="utf-8")
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            status = generator.main(["--ledger", str(ledger), "--output", str(output)])
        self.assertEqual(status, 1)
        self.assertIn("invalid UTF-8 JSON", stderr.getvalue())
        self.assertEqual(output.read_text(encoding="utf-8"), "sentinel\n")

        ledger.write_text("", encoding="utf-8")
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            status = generator.main(["--ledger", str(ledger), "--output", str(output)])
        self.assertEqual(status, 1)
        self.assertIn("contains no issues", stderr.getvalue())
        self.assertEqual(output.read_text(encoding="utf-8"), "sentinel\n")

    def test_preexisting_temporary_path_is_never_followed_or_truncated(self) -> None:
        ledger, output = self.fixture()
        self.write_ledger(ledger, self.rows())
        temporary = output.with_name(f".{output.name}.tmp")
        temporary.write_text("do-not-touch\n", encoding="utf-8")
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            status = generator.main(["--ledger", str(ledger), "--output", str(output)])
        self.assertEqual(status, 1)
        self.assertIn("pre-existing temporary path", stderr.getvalue())
        self.assertEqual(temporary.read_text(encoding="utf-8"), "do-not-touch\n")
        self.assertFalse(output.exists())

    def test_activation_cap_breach_refuses_before_publication(self) -> None:
        ledger, output = self.fixture()
        rows = [
            record(
                f"fm-{index}",
                f"W{index}: active",
                "in_progress",
                updated_at=f"2026-08-{20 + index:02d}T00:00:00Z",
                assignee="agent",
            )
            for index in range(1, 6)
        ]
        self.write_ledger(ledger, rows)
        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            status = generator.main(["--ledger", str(ledger), "--output", str(output)])
        self.assertEqual(status, 1)
        self.assertIn("workstream cap breached", stderr.getvalue())
        self.assertFalse(output.exists())

        stdout = io.StringIO()
        stderr = io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            status = generator.main(["--ledger", str(ledger), "--stdout"])
        self.assertEqual(status, 1)
        self.assertEqual(stdout.getvalue(), "")
        self.assertIn("workstream cap breached", stderr.getvalue())

    def test_g0_counts_toward_activation_cap_before_publication(self) -> None:
        ledger, output = self.fixture()
        rows = [
            record(
                "fm-g0",
                "G0: active",
                "in_progress",
                updated_at="2026-08-20T00:00:00Z",
                assignee="agent",
            )
        ]
        rows.extend(
            record(
                f"fm-w{index}",
                f"W{index}: active",
                "in_progress",
                updated_at=f"2026-08-{20 + index:02d}T00:00:00Z",
                assignee="agent",
            )
            for index in range(1, 5)
        )
        self.write_ledger(ledger, rows)
        output.write_text("sentinel\n", encoding="utf-8")

        for mode in ([], ["--check"], ["--stdout"]):
            stdout = io.StringIO()
            stderr = io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                status = generator.main(
                    ["--ledger", str(ledger), "--output", str(output), *mode]
                )
            self.assertEqual(status, 1)
            self.assertEqual(stdout.getvalue(), "")
            self.assertIn("workstream cap breached (5/4)", stderr.getvalue())
            self.assertEqual(output.read_text(encoding="utf-8"), "sentinel\n")

    def test_integrity_failure_refuses_every_publication_mode(self) -> None:
        ledger, output = self.fixture()
        bad = record(
            "fm-bad",
            "W10: missing blocker",
            "open",
            updated_at="2026-08-31T09:00:00Z",
        )
        bad["dependencies"] = [
            {"issue_id": "fm-bad", "depends_on_id": "fm-absent", "type": "blocks"}
        ]
        self.write_ledger(ledger, [bad])
        output.write_text("sentinel\n", encoding="utf-8")

        for mode in ([], ["--check"], ["--stdout"]):
            stdout = io.StringIO()
            stderr = io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                status = generator.main(
                    ["--ledger", str(ledger), "--output", str(output), *mode]
                )
            self.assertEqual(status, 1)
            self.assertEqual(stdout.getvalue(), "")
            self.assertIn("task-graph integrity is invalid", stderr.getvalue())
            self.assertIn("1 missing blockers", stderr.getvalue())
            self.assertEqual(output.read_text(encoding="utf-8"), "sentinel\n")

    def test_containment_cycle_refuses_every_publication_mode(self) -> None:
        ledger, output = self.fixture()
        self.write_ledger(
            ledger,
            [
                record(
                    "fm-a",
                    "W10: containment A",
                    "open",
                    updated_at="2026-08-31T09:00:00Z",
                    parent="fm-b",
                ),
                record(
                    "fm-b",
                    "W10: containment B",
                    "open",
                    updated_at="2026-08-31T09:01:00Z",
                    parent="fm-a",
                ),
            ],
        )
        output.write_text("sentinel\n", encoding="utf-8")

        for mode in ([], ["--check"], ["--stdout"]):
            stdout = io.StringIO()
            stderr = io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                status = generator.main(
                    ["--ledger", str(ledger), "--output", str(output), *mode]
                )
            self.assertEqual(status, 1)
            self.assertEqual(stdout.getvalue(), "")
            self.assertIn("task-graph integrity is invalid", stderr.getvalue())
            self.assertIn("1 containment cycles", stderr.getvalue())
            self.assertEqual(output.read_text(encoding="utf-8"), "sentinel\n")

    def test_unscoped_active_refuses_every_publication_mode(self) -> None:
        ledger, output = self.fixture()
        self.write_ledger(
            ledger,
            [
                record(
                    "fm-unscoped",
                    "Maintenance without a workstream",
                    "in_progress",
                    updated_at="2026-08-31T09:00:00Z",
                    assignee="agent",
                )
            ],
        )
        output.write_text("sentinel\n", encoding="utf-8")

        for mode in ([], ["--check"], ["--stdout"]):
            stdout = io.StringIO()
            stderr = io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                status = generator.main(
                    ["--ledger", str(ledger), "--output", str(output), *mode]
                )
            self.assertEqual(status, 1)
            self.assertEqual(stdout.getvalue(), "")
            self.assertIn("task-graph integrity is invalid", stderr.getvalue())
            self.assertIn("1 unscoped active claims", stderr.getvalue())
            self.assertEqual(output.read_text(encoding="utf-8"), "sentinel\n")


if __name__ == "__main__":
    unittest.main()
