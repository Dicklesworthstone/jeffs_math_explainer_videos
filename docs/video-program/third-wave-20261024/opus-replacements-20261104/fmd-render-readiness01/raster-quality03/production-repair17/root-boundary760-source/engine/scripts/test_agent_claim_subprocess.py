from __future__ import annotations

import contextlib
import io
import os
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock

import agent_claim


class AgentClaimSubprocessTests(unittest.TestCase):
    def cwd(self) -> Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        return Path(directory.name)

    def run_python(
        self,
        source: str,
        *,
        timeout_seconds: float = agent_claim.DEFAULT_COMMAND_TIMEOUT_SECONDS,
        total_output_bytes: int | None = None,
    ) -> agent_claim.CommandResult:
        return agent_claim.run_command(
            (sys.executable, "-c", source),
            self.cwd(),
            timeout_seconds=timeout_seconds,
            total_output_bytes=total_output_bytes,
        )

    def test_exact_retained_limit_is_accepted_for_both_streams(self) -> None:
        with mock.patch.object(agent_claim, "MAX_COMMAND_OUTPUT_BYTES", 64):
            result = self.run_python(
                "import sys; "
                "sys.stdout.buffer.write(b'x' * 64); "
                "sys.stderr.buffer.write(b'y' * 64)"
            )
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b"x" * 64)
        self.assertEqual(result.stderr, b"y" * 64)

    def test_exact_total_output_limit_is_accepted_across_both_streams(self) -> None:
        with mock.patch.object(agent_claim, "MAX_COMMAND_OUTPUT_BYTES", 64):
            result = self.run_python(
                "import sys; "
                "sys.stdout.buffer.write(b'x' * 64); "
                "sys.stderr.buffer.write(b'y' * 64)",
                total_output_bytes=128,
            )
        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, b"x" * 64)
        self.assertEqual(result.stderr, b"y" * 64)

    def test_combined_streams_share_one_configured_output_budget(self) -> None:
        with mock.patch.object(agent_claim, "MAX_COMMAND_OUTPUT_BYTES", 64):
            with self.assertRaisesRegex(
                agent_claim.ClaimError,
                "produced more than 64 total output bytes",
            ):
                self.run_python(
                    "import sys; "
                    "sys.stdout.buffer.write(b'x' * 40); "
                    "sys.stderr.buffer.write(b'y' * 40)",
                    total_output_bytes=64,
                )

    def test_total_output_budget_terminates_a_spewing_child_promptly(self) -> None:
        started = time.monotonic()
        with mock.patch.object(agent_claim, "MAX_COMMAND_OUTPUT_BYTES", 64):
            with self.assertRaisesRegex(
                agent_claim.ClaimError,
                "produced more than 4096 total output bytes",
            ):
                self.run_python(
                    "import os; chunk=b'x' * 65536; \nwhile True: os.write(1, chunk)",
                    timeout_seconds=10.0,
                    total_output_bytes=4096,
                )
        self.assertLess(time.monotonic() - started, 3.0)

    def test_stdout_retained_overflow_is_detected_after_full_drain(self) -> None:
        with mock.patch.object(agent_claim, "MAX_COMMAND_OUTPUT_BYTES", 64):
            with self.assertRaisesRegex(
                agent_claim.ClaimError,
                "stdout exceeded the 64-byte command-output limit",
            ):
                self.run_python(
                    "import sys; "
                    "sys.stdout.buffer.write(b'x' * 1048576); "
                    "sys.stderr.buffer.write(b'ok')"
                )

    def test_stderr_retained_overflow_does_not_deadlock_on_stdout(self) -> None:
        with mock.patch.object(agent_claim, "MAX_COMMAND_OUTPUT_BYTES", 64):
            with self.assertRaisesRegex(
                agent_claim.ClaimError,
                "stderr exceeded the 64-byte command-output limit",
            ):
                self.run_python(
                    "import sys; "
                    "sys.stdout.buffer.write(b'ok'); "
                    "sys.stderr.buffer.write(b'y' * 1048576)"
                )

    def test_dual_large_streams_are_drained_concurrently_and_fail_closed(self) -> None:
        with mock.patch.object(agent_claim, "MAX_COMMAND_OUTPUT_BYTES", 64):
            with self.assertRaisesRegex(agent_claim.ClaimError, "command-output limit"):
                self.run_python(
                    "import sys, threading; "
                    "a=threading.Thread(target=lambda: sys.stdout.buffer.write(b'x' * 1048576)); "
                    "b=threading.Thread(target=lambda: sys.stderr.buffer.write(b'y' * 1048576)); "
                    "a.start(); b.start(); a.join(); b.join()"
                )

    def test_nonzero_exit_keeps_bounded_diagnostics(self) -> None:
        result = self.run_python(
            "import sys; sys.stderr.write('bounded failure'); raise SystemExit(7)"
        )
        self.assertEqual(result.returncode, 7)
        self.assertEqual(result.stdout, b"")
        self.assertEqual(result.stderr, b"bounded failure")

    def test_timeout_terminates_a_stalled_child_and_returns_promptly(self) -> None:
        started = time.monotonic()
        with self.assertRaises(agent_claim.ClaimError) as raised:
            self.run_python("import time; time.sleep(60)", timeout_seconds=0.1)
        elapsed = time.monotonic() - started
        self.assertEqual(raised.exception.exit_code, 5)
        self.assertIn("timed out after 0.1 seconds", str(raised.exception))
        self.assertLess(elapsed, 3.0)

    @unittest.skipIf(os.name == "nt", "POSIX process-group assertion")
    def test_timeout_terminates_descendants_in_the_command_session(self) -> None:
        marker = self.cwd() / "descendant-survived"
        grandchild = (
            "import pathlib,time; "
            "time.sleep(0.8); "
            f"pathlib.Path({str(marker)!r}).write_text('alive')"
        )
        parent = (
            "import subprocess,sys,time; "
            f"subprocess.Popen([sys.executable, '-c', {grandchild!r}]); "
            "time.sleep(60)"
        )
        with self.assertRaisesRegex(agent_claim.ClaimError, "timed out"):
            self.run_python(parent, timeout_seconds=0.1)
        time.sleep(1.0)
        self.assertFalse(marker.exists())

    @unittest.skipIf(os.name == "nt", "POSIX inherited-pipe assertion")
    def test_parent_exit_with_a_live_descendant_cannot_hold_readers_forever(self) -> None:
        source = (
            "import subprocess,sys; "
            "subprocess.Popen([sys.executable, '-c', 'import time; time.sleep(60)'])"
        )
        with mock.patch.object(agent_claim, "COMMAND_READER_JOIN_SECONDS", 0.1):
            started = time.monotonic()
            with self.assertRaisesRegex(
                agent_claim.ClaimError,
                "output pipes remained open",
            ):
                self.run_python(source, timeout_seconds=5.0)
            self.assertLess(time.monotonic() - started, 3.0)

    def test_timeout_contract_rejects_nonfinite_nonpositive_and_excessive_values(self) -> None:
        for value, message in (
            (0.0, "must be positive"),
            (-1.0, "must be positive"),
            (float("nan"), "must be finite"),
            (float("inf"), "must be finite"),
            (agent_claim.MAX_COMMAND_TIMEOUT_SECONDS + 1.0, "exceeds the"),
        ):
            with self.subTest(value=value):
                with self.assertRaisesRegex(agent_claim.ClaimError, message):
                    self.run_python("pass", timeout_seconds=value)

    def test_output_budget_contract_rejects_invalid_values(self) -> None:
        for value, message in (
            (0, "must be positive"),
            (-1, "must be positive"),
            (True, "positive integer"),
            (agent_claim.MAX_COMMAND_OUTPUT_BUDGET_BYTES + 1, "exceeds the"),
        ):
            with self.subTest(value=value):
                with self.assertRaisesRegex(agent_claim.ClaimError, message):
                    self.run_python("pass", total_output_bytes=value)

    def test_cli_resource_validation_emits_no_machine_payload(self) -> None:
        for flag, value, message in (
            ("--command-timeout-seconds", "nan", "must be finite"),
            ("--command-output-budget-bytes", "0", "must be positive"),
        ):
            stdout = io.StringIO()
            stderr = io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                status = agent_claim.main(
                    [
                        "--expect-token",
                        "v2:" + "a" * 64 + ":fm-next",
                        "--assignee",
                        "agent@example.com",
                        flag,
                        value,
                        "--dry-run",
                    ]
                )
            self.assertEqual(status, 2)
            self.assertEqual(stdout.getvalue(), "")
            self.assertIn(message, stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
