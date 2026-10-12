#!/usr/bin/env python3
"""Corpus-driver metadata and argument checks. Fixtures exercise dispatch only;
the real corpus lint/probe runs are separate acceptance checks. Nothing is deleted."""

import json
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

    def dispatch_executable(self, directory, name, source):
        path = directory / name
        path.write_text(source, encoding="utf-8")
        path.chmod(0o755)
        return path

    def watch_dispatch(self, slug, voice, kind):
        # Only the driver is real: stop at an encoder recorder, with text markers
        # in place of media. No compiler, speech synthesis, or encoder runs.
        crate = self.crate(slug)
        (crate / "Cargo.toml").write_text(
            f'[package]\nname = "{slug}_explainer"\n[[bin]]\nname = "video"\n',
            encoding="utf-8",
        )
        binary_dir = self.root / "dispatch tools"
        binary_dir.mkdir()
        target = self.root / "dispatch target"
        (target / "release").mkdir(parents=True)
        self.dispatch_executable(target / "release", "video", "#!/bin/sh\nprintf '01_hook\\n'\n")
        self.dispatch_executable(binary_dir, "cargo", "#!/bin/sh\nexit 0\n")
        observed = self.root / "encoder-args.json"
        self.dispatch_executable(
            binary_dir, "ffmpeg",
            "#!/usr/bin/env python3\nimport json, sys\nfrom pathlib import Path\n"
            f"Path({str(observed)!r}).write_text(json.dumps(sys.argv[1:]))\n"
            "raise SystemExit(17)\n",
        )
        own_voice = "matt" if slug == "capture_recapture" else "michael"
        base = crate / "variants" / voice \
            if voice and voice != own_voice and kind != "draft" else crate
        primary = crate / "renders"
        selected = base / "renders"
        for directory in dict.fromkeys((primary, selected)):
            (directory / kind).mkdir(parents=True)
            (directory / kind / "01_hook.mp4").write_text("chapter marker\n", encoding="utf-8")
            (directory / f"{slug}_{kind}_watch.mp4").write_text("watch marker\n", encoding="utf-8")
        env = dict(os.environ, SKILLS=str(self.root / "missing skills"),
                   CARGO_TARGET_DIR=str(target),
                   PATH=str(binary_dir) + os.pathsep + os.environ["PATH"])
        env.pop("VOICE", None)
        if voice:
            env["VOICE"] = voice
        result = self.run_x(slug, "watch", kind, env=env)
        self.assertEqual(result.returncode, 17, result.stderr)
        args = json.loads(observed.read_text(encoding="utf-8"))
        self.assertEqual(Path(args[args.index("-i") + 1]), selected / kind / "watch.concat.txt")
        self.assertEqual(Path(args[-1]), selected / f"{slug}_{kind}_watch.mp4")
        self.assertEqual((selected / kind / "watch.concat.txt").read_text(), "file '01_hook.mp4'\n")
        archived = list((selected / "previous").glob("*.mp4"))
        self.assertEqual(len(archived), 1)
        self.assertEqual(archived[0].read_text(), "watch marker\n")
        if base != crate:
            self.assertEqual((primary / f"{slug}_{kind}_watch.mp4").read_text(), "watch marker\n")
            self.assertFalse((primary / kind / "watch.concat.txt").exists())
            self.assertFalse((primary / "previous").exists())

    def test_variant_watch_keeps_primary_inputs_output_and_archive_untouched(self):
        for slug, voice, kind in (("capture_recapture", "michael", "narrated"),
                                  ("raptorq_gf256", "leo", "scored")):
            with self.subTest(slug=slug, voice=voice, kind=kind):
                case = Driver()
                case.setUp()
                case.watch_dispatch(slug, voice, kind)

    def test_default_and_same_voice_watch_use_primary_cut(self):
        for slug, voice, kind in (("capture_recapture", None, "narrated"),
                                  ("raptorq_gf256", "michael", "scored"),
                                  ("raptorq_gf256", "leo", "draft")):
            with self.subTest(slug=slug, voice=voice, kind=kind):
                case = Driver()
                case.setUp()
                case.watch_dispatch(slug, voice, kind)

    def remote_dispatch(self, voice):
        # Exercise the actual ssh command locally: the fixture ssh invokes sh,
        # and systemd-run records its argv then exits before any worker starts.
        remote = self.root / "remote"
        remote.mkdir()
        binary_dir = self.root / "remote dispatch tools"
        binary_dir.mkdir()
        self.dispatch_executable(binary_dir, "git", "#!/bin/sh\nprintf 'x\\n'\n")
        self.dispatch_executable(binary_dir, "rsync", "#!/bin/sh\ncat >/dev/null\n")
        self.dispatch_executable(
            binary_dir, "ssh",
            "#!/usr/bin/env python3\nimport subprocess, sys\n"
            "raise SystemExit(subprocess.run(['/bin/sh', '-c', sys.argv[2]], check=False).returncode)\n",
        )
        observed = self.root / "unit-args.json"
        self.dispatch_executable(
            binary_dir, "systemd-run",
            "#!/usr/bin/env python3\nimport json, sys\nfrom pathlib import Path\n"
            f"Path({str(observed)!r}).write_text(json.dumps(sys.argv[1:]))\n"
            "raise SystemExit(17)\n",
        )
        env = dict(os.environ, SKILLS=str(self.root / "missing skills"),
                   REMOTE_HOST="dispatch-fixture", REMOTE_DIR=str(remote),
                   PATH=str(binary_dir) + os.pathsep + os.environ["PATH"])
        env.pop("VOICE", None)
        if voice is not None:
            env["VOICE"] = voice
        result = self.run_x("remote", "capture_recapture", "narrate", env=env)
        return result, observed

    def test_remote_forwards_variant_voice_to_the_unit(self):
        result, observed = self.remote_dispatch("michael")
        self.assertEqual(result.returncode, 17, result.stderr)
        self.assertIn("--setenv=VOICE=michael", json.loads(observed.read_text()))

    def test_remote_default_clears_a_worker_voice_override(self):
        result, observed = self.remote_dispatch(None)
        self.assertEqual(result.returncode, 17, result.stderr)
        self.assertIn("--setenv=VOICE=", json.loads(observed.read_text()))

    def test_remote_rejects_unsupported_or_path_like_voice_before_dispatch(self):
        for voice in ("unknown", "../primary", "michael;echo injected"):
            with self.subTest(voice=voice):
                case = Driver()
                case.setUp()
                result, observed = case.remote_dispatch(voice)
                self.assertEqual(result.returncode, 2, result.stderr)
                self.assertIn("unsupported VOICE", result.stderr)
                self.assertFalse(observed.exists())


if __name__ == "__main__":
    unittest.main()
