from __future__ import annotations

import contextlib
import io
import json
import os
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import generate_agent_brief as generator


class GenerateAgentBriefIoTests(unittest.TestCase):
    def fixture(self) -> tuple[Path, Path]:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        ledger = root / "issues.jsonl"
        output = root / "AGENT_BRIEF.md"
        ledger.write_text(
            json.dumps(
                {
                    "id": "fm-next",
                    "title": "W10: ready",
                    "status": "open",
                    "priority": 1,
                    "issue_type": "task",
                    "created_at": "2026-08-01T00:00:00Z",
                    "updated_at": "2026-08-31T08:15:00Z",
                },
                separators=(",", ":"),
            )
            + "\n",
            encoding="utf-8",
        )
        return ledger, output

    def test_identity_fingerprint_detects_reused_inode_metadata(self) -> None:
        owned = SimpleNamespace(
            st_dev=11,
            st_ino=29,
            st_size=4096,
            st_mtime_ns=101,
            st_ctime_ns=103,
        )
        substitute = SimpleNamespace(
            st_dev=11,
            st_ino=29,
            st_size=8,
            st_mtime_ns=107,
            st_ctime_ns=109,
        )
        self.assertNotEqual(
            generator._file_identity(owned),
            generator._file_identity(substitute),
        )

    def test_replace_failure_removes_only_the_temporary_file_it_created(self) -> None:
        ledger, output = self.fixture()
        temporary = output.with_name(f".{output.name}.tmp")
        stderr = io.StringIO()
        with mock.patch.object(generator.os, "replace", side_effect=OSError("replace refused")):
            with contextlib.redirect_stderr(stderr):
                status = generator.main(
                    ["--ledger", str(ledger), "--output", str(output)]
                )
        self.assertEqual(status, 1)
        self.assertIn("replace refused", stderr.getvalue())
        self.assertFalse(output.exists())
        self.assertFalse(temporary.exists())

        with contextlib.redirect_stdout(io.StringIO()):
            status = generator.main(["--ledger", str(ledger), "--output", str(output)])
        self.assertEqual(status, 0)
        self.assertTrue(output.is_file())

    def test_cleanup_never_unlinks_a_substituted_temporary_path(self) -> None:
        ledger, output = self.fixture()
        temporary = output.with_name(f".{output.name}.tmp")

        def substitute_and_fail(source: str | os.PathLike[str], _destination) -> None:
            Path(source).unlink()
            Path(source).write_text("foreign\n", encoding="utf-8")
            raise OSError("replace refused after substitution")

        stderr = io.StringIO()
        with mock.patch.object(generator.os, "replace", side_effect=substitute_and_fail):
            with contextlib.redirect_stderr(stderr):
                status = generator.main(
                    ["--ledger", str(ledger), "--output", str(output)]
                )
        self.assertEqual(status, 1)
        self.assertIn("temporary output identity changed", stderr.getvalue())
        self.assertEqual(temporary.read_text(encoding="utf-8"), "foreign\n")
        self.assertFalse(output.exists())

    def test_check_mode_refuses_output_symlinks_without_reading_the_target(self) -> None:
        ledger, output = self.fixture()
        target = output.with_name("target.md")
        target.write_text("sentinel\n", encoding="utf-8")
        try:
            output.symlink_to(target)
        except OSError as exc:
            self.skipTest(f"host cannot create symlinks: {exc}")

        stderr = io.StringIO()
        with contextlib.redirect_stderr(stderr):
            status = generator.main(
                ["--ledger", str(ledger), "--output", str(output), "--check"]
            )
        self.assertEqual(status, 1)
        self.assertIn("refusing symlink output path", stderr.getvalue())
        self.assertEqual(target.read_text(encoding="utf-8"), "sentinel\n")


if __name__ == "__main__":
    unittest.main()
