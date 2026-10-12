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

    def run_x(self, *args):
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


if __name__ == "__main__":
    unittest.main()
