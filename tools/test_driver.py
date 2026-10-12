#!/usr/bin/env python3
"""Corpus-driver metadata and argument checks. Fixtures exercise dispatch only;
the real corpus lint/probe runs are separate acceptance checks. Nothing is deleted."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class Driver(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="explainer_driver_"))
        shutil.copyfile(ROOT / "x", self.root / "x")
        (self.root / "x").chmod(0o755)
        self.corpus = self.root / "explainer_corpus"
        self.corpus.mkdir()

    def crate(self, slug, title=None):
        crate = self.corpus / slug
        crate.mkdir()
        (crate / "Cargo.toml").write_text(f'[package]\nname = "{slug}_explainer"\n', encoding="utf-8")
        if title is not None:
            (crate / "README.md").write_text(f"# {title}\n", encoding="utf-8")
        return crate

    def run_x(self, *args, env=None):
        if env is None:
            env = dict(os.environ, SKILLS=str(self.root / "missing skills"))
        return subprocess.run(
            [str(self.root / "x"), *args], cwd=self.root.parent,
            env=env, capture_output=True, text=True, check=False, timeout=10,
        )

    def test_list_reads_real_manifest_and_title_and_ignores_non_crates(self):
        self.crate("alpha", "A title with spaces")
        self.crate("beta")
        (self.corpus / "notes").mkdir()
        result = self.run_x("list")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.splitlines(), [
            "slug\tpackage\ttitle",
            "alpha\talpha_explainer\tA title with spaces",
            "beta\tbeta_explainer\t",
        ])

    def test_list_rejects_extra_arguments(self):
        self.assertEqual(self.run_x("list", "extra").returncode, 2)

    def test_list_uses_the_brief_when_a_readme_is_missing(self):
        crate = self.crate("alpha")
        (crate / "docs").mkdir()
        (crate / "docs" / "brief.md").write_text("# Brief — The actual title\n", encoding="utf-8")
        result = self.run_x("list")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("alpha\talpha_explainer\tThe actual title\n", result.stdout)

    def test_help_includes_new_commands_and_preserves_narration_commands(self):
        result = self.run_x("missing")
        self.assertEqual(result.returncode, 2)
        for command in ("./x list", " lint ", " probe ", " narrate ", " narration-sheets "):
            self.assertIn(command, result.stdout)

    def test_missing_lint_skill_fails_before_build(self):
        self.crate("alpha")
        result = self.run_x("alpha", "lint")
        self.assertEqual(result.returncode, 2)
        self.assertIn("no lint_script.py", result.stderr)

    def test_probe_rejects_extra_arguments_before_build(self):
        self.crate("alpha")
        self.assertEqual(self.run_x("alpha", "probe", "first", "second").returncode, 2)

    def test_lint_rejects_extra_arguments_before_build(self):
        self.crate("alpha")
        self.assertEqual(self.run_x("alpha", "lint", "extra").returncode, 2)

    def narration_dispatch(self, version="0.1.11"):
        # Dispatch-only fixture: stop at plan, before any speech synthesis.
        self.crate("capture_recapture")
        binary_dir = self.root / "local tools"
        binary_dir.mkdir()
        ftts = binary_dir / "ftts"
        ftts.write_text(f'#!/bin/sh\nprintf "ftts {version}\\n"\n', encoding="utf-8")
        ftts.chmod(0o755)
        uv = binary_dir / "uv"
        uv.write_text(
            '#!/bin/sh\nwhile [ "$#" -gt 0 ]; do\n'
            '  [ "$1" = python3 ] && exec "$@"\n  shift\ndone\nexit 99\n',
            encoding="utf-8",
        )
        uv.chmod(0o755)
        tools = self.root / "tools"
        tools.mkdir()
        observed = self.root / "observed-ftts.txt"
        (tools / "narration.py").write_text(
            "import os, sys\nfrom pathlib import Path\n"
            "p = Path(os.environ['FTTS'])\n"
            f"Path({str(observed)!r}).write_text(str(p), encoding='utf-8')\n"
            "sys.exit(17 if p.is_absolute() and p.is_file() else 16)\n",
            encoding="utf-8",
        )
        env = dict(os.environ, SKILLS=str(self.root / "skills"), FTTS_VERSION="0.1.11",
                   PATH=str(binary_dir) + os.pathsep + os.environ["PATH"])
        env.pop("FTTS", None)
        return env, ftts, observed

    def test_narrate_resolves_path_command_before_changing_directory(self):
        env, ftts, observed = self.narration_dispatch()
        result = self.run_x("capture_recapture", "narrate", env=env)
        self.assertEqual(result.returncode, 17, result.stderr)
        self.assertEqual(Path(observed.read_text(encoding="utf-8")).resolve(), ftts.resolve())

    def test_narrate_resolves_explicit_relative_binary(self):
        env, ftts, observed = self.narration_dispatch()
        env["FTTS"] = str(ftts.relative_to(self.root.parent))
        result = self.run_x("capture_recapture", "narrate", env=env)
        self.assertEqual(result.returncode, 17, result.stderr)
        self.assertEqual(Path(observed.read_text(encoding="utf-8")).resolve(), ftts.resolve())

    def test_narrate_still_rejects_wrong_version_before_plan(self):
        env, _, observed = self.narration_dispatch(version="0.1.5")
        result = self.run_x("capture_recapture", "narrate", env=env)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertIn("not ftts 0.1.11", result.stderr)
        self.assertFalse(observed.exists())


if __name__ == "__main__":
    unittest.main()
