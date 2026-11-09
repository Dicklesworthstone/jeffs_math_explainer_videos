#!/usr/bin/env python3
"""One-shot source-guarded repair for the claim transaction budget contract."""

from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one source match, found {count}")
    return text.replace(old, new, 1)


def patch_claim() -> None:
    path = ROOT / "scripts" / "agent_claim.py"
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        "SCHEMA_VERSION = 6",
        "SCHEMA_VERSION = 7",
        "claim schema version",
    )
    text = replace_once(
        text,
        '''    @property
    def produced(self) -> int:
        with self._lock:
            return self._produced


Runner = Callable[[tuple[str, ...], Path], CommandResult]
''',
        '''    @property
    def produced(self) -> int:
        with self._lock:
            return self._produced


class _ClaimTransactionBudget:
    """One deadline and produced-byte ceiling for the complete mutation."""

    def __init__(self, timeout_seconds: float, output_bytes: int):
        self.timeout_seconds = _command_timeout(timeout_seconds)
        self.output_bytes = _command_output_budget(output_bytes)
        self.started_at = time.monotonic()
        self.deadline = self.started_at + self.timeout_seconds
        self.produced = 0
        self.commands_completed = 0

    def remaining_seconds(self, phase: str) -> float:
        remaining = self.deadline - time.monotonic()
        if remaining <= 0:
            raise ClaimError(
                f"claim transaction timed out after {self.timeout_seconds:g} seconds {phase}",
                5,
            )
        return remaining

    def remaining_output_bytes(self) -> int:
        return max(0, self.output_bytes - self.produced)

    def consume(self, result: CommandResult, label: str) -> None:
        self.produced += len(result.stdout) + len(result.stderr)
        self.commands_completed += 1
        if self.produced > self.output_bytes:
            raise ClaimError(
                "claim transaction produced more than "
                f"{self.output_bytes} total output bytes across "
                "br update --claim and br sync --flush-only",
                5,
            )
        self.remaining_seconds(f"after {label}")


Runner = Callable[[tuple[str, ...], Path], CommandResult]
''',
        "transaction budget insertion",
    )
    text = replace_once(
        text,
        '''def run_command(
    argv: tuple[str, ...],
    cwd: Path,
    *,
    timeout_seconds: float = DEFAULT_COMMAND_TIMEOUT_SECONDS,
    total_output_bytes: int | None = None,
) -> CommandResult:
''',
        '''def run_command(
    argv: tuple[str, ...],
    cwd: Path,
    *,
    timeout_seconds: float = DEFAULT_COMMAND_TIMEOUT_SECONDS,
    total_output_bytes: int | None = None,
    _deadline_monotonic: float | None = None,
) -> CommandResult:
''',
        "run_command signature",
    )
    text = replace_once(
        text,
        '''    total_output_bytes = _command_output_budget(total_output_bytes)
    env = os.environ.copy()
''',
        '''    total_output_bytes = _command_output_budget(total_output_bytes)
    local_deadline = time.monotonic() + timeout_seconds
    if _deadline_monotonic is None:
        deadline = local_deadline
    else:
        if (
            isinstance(_deadline_monotonic, bool)
            or not isinstance(_deadline_monotonic, (int, float))
            or not math.isfinite(float(_deadline_monotonic))
        ):
            raise ClaimError("internal command deadline must be finite", 5)
        deadline = min(local_deadline, float(_deadline_monotonic))
    env = os.environ.copy()
''',
        "absolute command deadline",
    )
    text = replace_once(
        text,
        '''    deadline = time.monotonic() + timeout_seconds
    returncode: int | None = None
''',
        '''    returncode: int | None = None
''',
        "remove renewed command deadline",
    )
    text = replace_once(
        text,
        '''    return result


def _commands(
''',
        '''    return result


def _run_transaction_checked(
    transaction: _ClaimTransactionBudget,
    runner: Runner | None,
    argv: tuple[str, ...],
    cwd: Path,
    label: str,
) -> CommandResult:
    remaining_seconds = transaction.remaining_seconds(f"before {label}")
    remaining_output_bytes = transaction.remaining_output_bytes()
    if runner is None:

        def bounded_runner(command: tuple[str, ...], command_cwd: Path) -> CommandResult:
            return run_command(
                command,
                command_cwd,
                timeout_seconds=remaining_seconds,
                total_output_bytes=max(1, remaining_output_bytes),
                _deadline_monotonic=transaction.deadline,
            )

        active_runner = bounded_runner
    else:
        active_runner = runner
    result = _run_checked(
        active_runner,
        argv,
        cwd,
        label,
        remaining_output_bytes,
    )
    transaction.consume(result, label)
    return result


def _commands(
''',
        "transaction command wrapper",
    )
    text = replace_once(
        text,
        '''    if runner is None:
        def active_runner(argv: tuple[str, ...], cwd: Path) -> CommandResult:
            return run_command(
                argv,
                cwd,
                timeout_seconds=command_timeout_seconds,
                total_output_bytes=command_output_budget_bytes,
            )
    else:
        active_runner = runner

    repo_root = _repo_root(repo_root)
''',
        '''    repo_root = _repo_root(repo_root)
''',
        "remove per-command budget renewal",
    )
    text = replace_once(
        text,
        '''                "command_timeout_seconds": command_timeout_seconds,
                "command_output_bytes_per_stream": MAX_COMMAND_OUTPUT_BYTES,
                "command_output_budget_bytes_total": command_output_budget_bytes,
''',
        '''                "claim_transaction_scope": "br-update-through-postcondition/v1",
                "claim_transaction_timeout_seconds": command_timeout_seconds,
                "command_output_bytes_per_stream": MAX_COMMAND_OUTPUT_BYTES,
                "claim_transaction_output_budget_bytes": command_output_budget_bytes,
''',
        "receipt transaction policy",
    )
    text = replace_once(
        text,
        '''        update_result = _run_checked(
            active_runner,
            update_command,
            repo_root,
            "br update --claim",
            command_output_budget_bytes,
        )
        atomic_claim = _verify_atomic_claim_response(update_result, issue, assignee)
        _run_checked(
            active_runner,
            sync_command,
            repo_root,
            "br sync --flush-only",
            command_output_budget_bytes,
        )
        try:
            after = agent_brief.load_issues(ledger)
        except agent_brief.BriefError as exc:
            raise ClaimError(f"post-claim ledger verification failed: {exc}", 5) from exc
''',
        '''        transaction = _ClaimTransactionBudget(
            command_timeout_seconds,
            command_output_budget_bytes,
        )
        update_result = _run_transaction_checked(
            transaction,
            runner,
            update_command,
            repo_root,
            "br update --claim",
        )
        atomic_claim = _verify_atomic_claim_response(update_result, issue, assignee)
        transaction.remaining_seconds("after br update --claim response validation")
        _run_transaction_checked(
            transaction,
            runner,
            sync_command,
            repo_root,
            "br sync --flush-only",
        )
        transaction.remaining_seconds("before post-claim ledger verification")
        try:
            after = agent_brief.load_issues(ledger)
        except agent_brief.BriefError as exc:
            raise ClaimError(f"post-claim ledger verification failed: {exc}", 5) from exc
        transaction.remaining_seconds("after post-claim ledger verification")
''',
        "transactional command execution",
    )
    text = replace_once(
        text,
        '''        claim_delta = _verify_claim_only_delta(
            issues,
            after,
            issue_id,
            assignee,
            transition_comment,
        )
        try:
''',
        '''        claim_delta = _verify_claim_only_delta(
            issues,
            after,
            issue_id,
            assignee,
            transition_comment,
        )
        transaction.remaining_seconds("after claim-delta verification")
        try:
''',
        "claim delta deadline checkpoint",
    )
    text = replace_once(
        text,
        '''        except agent_claim_guard.GuardError as exc:
            raise ClaimError(f"post-claim ledger verification failed: {exc}", 5) from exc
        if atomic_claim["updated_at"] != claim_delta["updated_at_after"]:
''',
        '''        except agent_claim_guard.GuardError as exc:
            raise ClaimError(f"post-claim ledger verification failed: {exc}", 5) from exc
        transaction.remaining_seconds("after post-claim graph authentication")
        if atomic_claim["updated_at"] != claim_delta["updated_at_after"]:
''',
        "graph authentication deadline checkpoint",
    )
    text = replace_once(
        text,
        '''        if atomic_claim["updated_at"] != claim_delta["updated_at_after"]:
            raise ClaimError(
                "br update --claim JSON timestamp disagreed with the exported ledger",
                5,
            )
        receipt.update(
''',
        '''        if atomic_claim["updated_at"] != claim_delta["updated_at_after"]:
            raise ClaimError(
                "br update --claim JSON timestamp disagreed with the exported ledger",
                5,
            )
        transaction.remaining_seconds("after claim timestamp reconciliation")
        receipt.update(
''',
        "timestamp deadline checkpoint",
    )
    text = replace_once(
        text,
        '''                "atomic_claim": atomic_claim,
                "claim_delta": claim_delta,
            }
''',
        '''                "atomic_claim": atomic_claim,
                "claim_delta": claim_delta,
                "transaction_usage": {
                    "commands_completed": transaction.commands_completed,
                    "output_bytes_produced": transaction.produced,
                },
            }
''',
        "transaction usage receipt",
    )
    text = replace_once(
        text,
        '            "maximum wall-clock time for each br command; "',
        '            "maximum wall-clock time from br update through postcondition verification; "',
        "timeout CLI help",
    )
    text = replace_once(
        text,
        '            "maximum combined stdout/stderr bytes produced by each br command; "',
        '            "maximum combined stdout/stderr bytes produced by the br update/sync transaction; "',
        "output CLI help",
    )
    path.write_text(text, encoding="utf-8")


def patch_existing_tests() -> None:
    path = ROOT / "scripts" / "test_agent_claim.py"
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        '        self.assertEqual(receipt["version"], 6)\n',
        '        self.assertEqual(receipt["version"], 7)\n',
        "claim test schema version",
    )
    text = replace_once(
        text,
        '''                "command_timeout_seconds": 17.5,
                "command_output_bytes_per_stream": agent_claim.MAX_COMMAND_OUTPUT_BYTES,
                "command_output_budget_bytes_total": 4096,
''',
        '''                "claim_transaction_scope": "br-update-through-postcondition/v1",
                "claim_transaction_timeout_seconds": 17.5,
                "command_output_bytes_per_stream": agent_claim.MAX_COMMAND_OUTPUT_BYTES,
                "claim_transaction_output_budget_bytes": 4096,
''',
        "claim test transaction policy",
    )
    path.write_text(text, encoding="utf-8")


def patch_gate() -> None:
    path = ROOT / "scripts" / "check.sh"
    text = path.read_text(encoding="utf-8")
    text = replace_once(
        text,
        '''    scripts/test_agent_claim.py \\
    scripts/test_agent_claim_subprocess.py \\
    scripts/test_generate_agent_brief.py \\
''',
        '''    scripts/test_agent_claim.py \\
    scripts/test_agent_claim_subprocess.py \\
    scripts/test_agent_claim_transaction.py \\
    scripts/test_generate_agent_brief.py \\
''',
        "check py_compile transaction test",
    )
    text = replace_once(
        text,
        '''python3 scripts/test_agent_claim.py
python3 scripts/test_agent_claim_subprocess.py
python3 scripts/test_generate_agent_brief.py
''',
        '''python3 scripts/test_agent_claim.py
python3 scripts/test_agent_claim_subprocess.py
python3 scripts/test_agent_claim_transaction.py
python3 scripts/test_generate_agent_brief.py
''',
        "check transaction test execution",
    )
    path.write_text(text, encoding="utf-8")


def write_transaction_tests() -> None:
    path = ROOT / "scripts" / "test_agent_claim_transaction.py"
    if path.exists():
        raise SystemExit(f"refusing to overwrite existing {path}")
    path.write_text(
        '''from __future__ import annotations

import json
import unittest
from unittest import mock

import agent_claim
from test_agent_claim import FakeRunner, Fixture, record


class Clock:
    def __init__(self, value: float = 100.0) -> None:
        self.value = value

    def __call__(self) -> float:
        return self.value

    def advance(self, seconds: float) -> None:
        self.value += seconds


def update_payload(assignee: str = "agent@example.com") -> bytes:
    return json.dumps(
        [
            {
                "id": "fm-next",
                "title": "W10: guarded claim fixture",
                "status": "in_progress",
                "priority": 1,
                "assignee": assignee,
                "owner": None,
                "updated_at": "2026-09-01T00:00:01Z",
            }
        ],
        separators=(",", ":"),
    ).encode()


class AgentClaimTransactionTests(unittest.TestCase):
    def fixture(self) -> Fixture:
        fixture = Fixture([record("fm-next")])
        self.addCleanup(fixture.cleanup)
        return fixture

    def test_one_deadline_and_output_ceiling_feed_both_commands(self) -> None:
        fixture = self.fixture()
        base = FakeRunner(fixture)
        clock = Clock()
        calls: list[tuple[str, float, int, float | None]] = []
        produced: list[int] = []

        def run(
            argv,
            cwd,
            *,
            timeout_seconds,
            total_output_bytes,
            _deadline_monotonic=None,
        ):
            label = base.command(argv)
            calls.append(
                (
                    label,
                    timeout_seconds,
                    total_output_bytes,
                    _deadline_monotonic,
                )
            )
            result = base(argv, cwd)
            produced.append(len(result.stdout) + len(result.stderr))
            clock.advance(3.0 if label == "update" else 2.0)
            return result

        with mock.patch.object(agent_claim.time, "monotonic", clock), mock.patch.object(
            agent_claim,
            "run_command",
            side_effect=run,
        ):
            receipt = agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                command_timeout_seconds=10.0,
                command_output_budget_bytes=4096,
            )

        self.assertEqual([call[0] for call in calls], ["update", "sync"])
        self.assertAlmostEqual(calls[0][1], 10.0)
        self.assertAlmostEqual(calls[1][1], 7.0)
        self.assertEqual(calls[0][2], 4096)
        self.assertEqual(calls[1][2], 4096 - produced[0])
        self.assertEqual(calls[0][3], 110.0)
        self.assertEqual(calls[1][3], 110.0)
        self.assertEqual(
            receipt["transaction_usage"],
            {
                "commands_completed": 2,
                "output_bytes_produced": sum(produced),
            },
        )

    def test_timeout_is_not_renewed_after_update(self) -> None:
        fixture = self.fixture()
        base = FakeRunner(fixture)
        clock = Clock()
        calls: list[str] = []

        def run(
            argv,
            cwd,
            *,
            timeout_seconds,
            total_output_bytes,
            _deadline_monotonic=None,
        ):
            del timeout_seconds, total_output_bytes, _deadline_monotonic
            calls.append(base.command(argv))
            result = base(argv, cwd)
            clock.advance(5.0)
            return result

        with mock.patch.object(agent_claim.time, "monotonic", clock), mock.patch.object(
            agent_claim,
            "run_command",
            side_effect=run,
        ):
            with self.assertRaisesRegex(
                agent_claim.ClaimError,
                "claim transaction timed out after 5 seconds after br update --claim",
            ):
                agent_claim.execute_claim(
                    fixture.root,
                    fixture.token(),
                    "agent@example.com",
                    command_timeout_seconds=5.0,
                )
        self.assertEqual(calls, ["update"])

    def test_total_output_budget_is_shared_across_update_and_sync(self) -> None:
        fixture = self.fixture()
        payload = update_payload()
        runner = FakeRunner(
            fixture,
            update_stdout=payload,
            sync_stdout=b"123456",
        )
        with self.assertRaisesRegex(
            agent_claim.ClaimError,
            "br sync --flush-only produced more than 5 total output bytes",
        ):
            agent_claim.execute_claim(
                fixture.root,
                fixture.token(),
                "agent@example.com",
                command_output_budget_bytes=len(payload) + 5,
                runner=runner,
            )
        self.assertEqual(
            [runner.command(argv) for argv, _cwd in runner.calls],
            ["update", "sync"],
        )

    def test_exact_transaction_output_budget_allows_silent_sync(self) -> None:
        fixture = self.fixture()
        payload = update_payload()
        runner = FakeRunner(
            fixture,
            update_stdout=payload,
            sync_stdout=b"",
        )
        receipt = agent_claim.execute_claim(
            fixture.root,
            fixture.token(),
            "agent@example.com",
            command_output_budget_bytes=len(payload),
            runner=runner,
        )
        self.assertTrue(receipt["claimed"])
        self.assertEqual(
            receipt["transaction_usage"],
            {
                "commands_completed": 2,
                "output_bytes_produced": len(payload),
            },
        )

    def test_postcondition_verification_remains_inside_deadline(self) -> None:
        fixture = self.fixture()
        runner = FakeRunner(fixture)
        clock = Clock()
        token = fixture.token()
        real_load = agent_claim.agent_brief.load_issues
        load_count = 0

        def load(path):
            nonlocal load_count
            load_count += 1
            result = real_load(path)
            if load_count == 2:
                clock.advance(5.0)
            return result

        with mock.patch.object(agent_claim.time, "monotonic", clock), mock.patch.object(
            agent_claim.agent_brief,
            "load_issues",
            side_effect=load,
        ):
            with self.assertRaisesRegex(
                agent_claim.ClaimError,
                "claim transaction timed out after 5 seconds "
                "after post-claim ledger verification",
            ):
                agent_claim.execute_claim(
                    fixture.root,
                    token,
                    "agent@example.com",
                    command_timeout_seconds=5.0,
                    runner=runner,
                )
        self.assertEqual(load_count, 2)

    def test_response_validation_remains_inside_deadline(self) -> None:
        fixture = self.fixture()
        runner = FakeRunner(fixture)
        clock = Clock()
        real_verify = agent_claim._verify_atomic_claim_response

        def verify(*args, **kwargs):
            result = real_verify(*args, **kwargs)
            clock.advance(5.0)
            return result

        with mock.patch.object(agent_claim.time, "monotonic", clock), mock.patch.object(
            agent_claim,
            "_verify_atomic_claim_response",
            side_effect=verify,
        ):
            with self.assertRaisesRegex(
                agent_claim.ClaimError,
                "after br update --claim response validation",
            ):
                agent_claim.execute_claim(
                    fixture.root,
                    fixture.token(),
                    "agent@example.com",
                    command_timeout_seconds=5.0,
                    runner=runner,
                )
        self.assertEqual(
            [runner.command(argv) for argv, _cwd in runner.calls],
            ["update"],
        )


if __name__ == "__main__":
    unittest.main()
''',
        encoding="utf-8",
    )


def main() -> None:
    patch_claim()
    patch_existing_tests()
    patch_gate()
    write_transaction_tests()


if __name__ == "__main__":
    main()
