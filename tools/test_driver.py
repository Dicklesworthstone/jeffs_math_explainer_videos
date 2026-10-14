#!/usr/bin/env python3
"""Corpus-driver metadata and argument checks. Fixtures exercise dispatch only;
the real corpus lint/probe runs are separate acceptance checks. Nothing is deleted."""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
import wave
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class Driver(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="explainer_driver_"))
        shutil.copyfile(ROOT / "x", self.root / "x")
        (self.root / "x").chmod(0o755)
        (self.root / "tools").mkdir(exist_ok=True)
        shutil.copyfile(ROOT / "tools" / "publish.py", self.root / "tools" / "publish.py")
        shutil.copyfile(ROOT / "tools" / "youtube_package.py", self.root / "tools" / "youtube_package.py")
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

    def fake_skills(self, *scripts):
        """A skills tree whose explainer scripts only record the arguments they were given."""
        skills = self.root / "skills"
        bin_dir = skills / "math-explainer-videos-end-to-end-with-franken-manim-frankentts" / "scripts"
        bin_dir.mkdir(parents=True)
        for name in scripts:
            (bin_dir / name).write_text(
                "import json, sys\nopen(sys.argv[0] + '.args', 'w').write(json.dumps(sys.argv[1:]))\n",
                encoding="utf-8")
        return skills, bin_dir

    def test_help_lists_the_review_commands(self):
        result = self.run_x("missing")
        self.assertEqual(result.returncode, 2)
        for command in (" protected ", " joins [scored|narrated|final] "):
            self.assertIn(command, result.stdout)

    def test_protected_needs_the_skill_script_and_the_list(self):
        crate = self.crate("alpha")
        result = self.run_x("alpha", "protected")
        self.assertEqual(result.returncode, 2)
        self.assertIn("no protected_check.py", result.stderr)
        skills, _ = self.fake_skills("protected_check.py")
        result = self.run_x("alpha", "protected", env=dict(os.environ, SKILLS=str(skills)))
        self.assertEqual(result.returncode, 2)
        self.assertIn("no docs/protected.tsv", result.stderr)
        self.assertFalse((crate / "docs").exists())

    def test_protected_checks_the_script_and_the_final_captions(self):
        crate = self.crate("alpha")
        (crate / "docs").mkdir()
        (crate / "docs" / "protected.tsv").write_text("a fact\tscript\twhy\n", encoding="utf-8")
        (crate / "script.tsv").write_text("01_1\tA fact.\n", encoding="utf-8")
        (crate / "renders" / "final").mkdir(parents=True)
        (crate / "renders" / "final" / "alpha.en.srt").write_text("1\n", encoding="utf-8")
        skills, bin_dir = self.fake_skills("protected_check.py")
        result = self.run_x("alpha", "protected", env=dict(os.environ, SKILLS=str(skills)))
        self.assertEqual(result.returncode, 0, result.stderr)
        args = json.loads((bin_dir / "protected_check.py.args").read_text(encoding="utf-8"))
        self.assertEqual([Path(a).name for a in args], ["protected.tsv", "script.tsv", "alpha.en.srt"])

    def test_protected_rejects_extra_arguments(self):
        self.crate("alpha")
        self.assertEqual(self.run_x("alpha", "protected", "extra").returncode, 2)

    def test_joins_rejects_an_unknown_cut_and_a_missing_skill_before_build(self):
        self.crate("alpha")
        self.assertEqual(self.run_x("alpha", "joins", "draft").returncode, 2)
        self.assertEqual(self.run_x("alpha", "joins", "scored", "extra").returncode, 2)
        result = self.run_x("alpha", "joins", "narrated")
        self.assertEqual(result.returncode, 2)
        self.assertIn("no join_audit.py", result.stderr)

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
        scripts = self.root / "skills" / "natural-sounding-narration-with-frankentts" / "scripts"
        scripts.mkdir(parents=True)
        observed = self.root / "observed-ftts.txt"
        (scripts / "narrate.py").write_text(
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

    def narrate_recorded(self, edges_failures, env_extra=None):
        # The whole `narrate` command with every tool a recorder: the skill's narrate.py and batch
        # scripts, and uv. calls.log is what each was asked; edges fails `edges_failures` times, then passes.
        crate = self.crate("capture_recapture")
        (crate / "script.tsv").write_text("01_1\tOne.\n", encoding="utf-8")
        bin_dir = self.root / "local bin"
        bin_dir.mkdir()
        log, left = self.root / "calls.log", self.root / "edges-left"
        left.write_text(str(edges_failures), encoding="utf-8")
        self.dispatch_executable(bin_dir, "ftts", '#!/bin/sh\nprintf "ftts 0.1.11\\n"\n')
        self.dispatch_executable(bin_dir, "uv", f'#!/bin/sh\necho "uv $*" >> {str(log)!r}\n'
                                 'while [ "$#" -gt 0 ]; do\n  [ "$1" = python3 ] && exec "$@"\n  shift\ndone\nexit 99\n')
        recorder = (
            "import os, sys\nfrom pathlib import Path\n"
            f"log, left = Path({str(log)!r}), Path({str(left)!r})\n"
            "name, args = Path(sys.argv[0]).name, sys.argv[1:]\n"
            "command = args[0] if name == 'narrate.py' and args else ''\n"
            "with log.open('a') as f:\n    f.write(f\"{name} {command} {' '.join(args[1:] if command else args)}\\n\")\n"
            "if command == 'edges' and int(left.read_text()):\n"
            "    left.write_text(str(int(left.read_text()) - 1))\n    sys.exit(1)\n"
            "if command == 'exclude':\n    sys.exit(int(os.environ.get('EXCLUDE_EXIT', '0')))\n")
        scripts = self.root / "skills" / "natural-sounding-narration-with-frankentts" / "scripts"
        scripts.mkdir(parents=True)
        for name in ("narrate.py", "tts_batch.py", "asr_batch.py", "qa_lines.py", "validate_narration.py"):
            (scripts / name).write_text(recorder, encoding="utf-8")
        env = {k: v for k, v in os.environ.items() if k not in ("FTTS", "JOINT", "JOINT_CREAK", "SEEDS", "EXCLUDE_EXIT")}
        env.update(SKILLS=str(self.root / "skills"), FTTS_VERSION="0.1.11",
                   PATH=str(bin_dir) + os.pathsep + os.environ["PATH"], **(env_extra or {}))
        result = self.run_x("capture_recapture", "narrate", env=env)
        return result, log.read_text(encoding="utf-8").splitlines()

    def test_narrate_without_joint_or_seeds_is_the_old_run_even_when_a_gate_fails(self):
        result, calls = self.narrate_recorded(edges_failures=1)
        self.assertEqual(result.returncode, 1, result.stderr)
        narration = [c for c in calls if c.startswith("narrate.py")]
        self.assertEqual([c.split()[1] for c in narration], ["plan", "choose", "edges"])
        self.assertFalse(any("--seeds" in c or "--joint" in c or "exclude" in c for c in calls), calls)
        self.assertFalse(any("praat-parselmouth" in c for c in calls), calls)

    def test_narrate_with_joint_and_seeds_chooses_again_after_a_gate_excludes_a_read(self):
        result, calls = self.narrate_recorded(edges_failures=1, env_extra={"JOINT": "1", "SEEDS": "2"})
        self.assertEqual(result.returncode, 0, result.stderr)
        narration = [c.split()[1] for c in calls if c.startswith("narrate.py")]
        self.assertEqual(narration, ["plan", "choose", "edges", "exclude", "choose", "edges"])
        plan = next(c for c in calls if c.startswith("narrate.py plan"))
        self.assertIn("--seeds=2", plan)
        self.assertEqual(sum("--joint" in c for c in calls if c.startswith("narrate.py choose")), 2)
        self.assertTrue(all("--with praat-parselmouth" in c for c in calls if c.startswith("uv ")), calls)

    def test_narrate_with_joint_stops_when_there_is_nothing_new_to_exclude(self):
        result, calls = self.narrate_recorded(edges_failures=99, env_extra={"JOINT": "1", "EXCLUDE_EXIT": "1"})
        self.assertEqual(result.returncode, 1, result.stderr)
        narration = [c.split()[1] for c in calls if c.startswith("narrate.py")]
        self.assertEqual(narration, ["plan", "choose", "edges", "exclude"])

    def test_narrate_with_joint_gives_up_after_twelve_exclusion_rounds(self):
        result, calls = self.narrate_recorded(edges_failures=99, env_extra={"JOINT": "1"})
        self.assertEqual(result.returncode, 1, result.stderr)
        narration = [c.split()[1] for c in calls if c.startswith("narrate.py")]
        self.assertEqual(narration.count("choose"), 13)
        self.assertEqual(narration.count("exclude"), 12)

    def test_narrate_prices_a_creaky_ending_only_when_asked_and_only_with_joint(self):
        _, calls = self.narrate_recorded(edges_failures=0, env_extra={"JOINT": "1", "JOINT_CREAK": "1"})
        chooses = [c for c in calls if c.startswith("narrate.py choose")]
        self.assertEqual(len(chooses), 1)
        self.assertIn("--joint", chooses[0])
        self.assertIn("--creak=1", chooses[0])

    def test_narrate_with_joint_alone_leaves_the_creak_unpriced(self):
        _, calls = self.narrate_recorded(edges_failures=0, env_extra={"JOINT": "1"})
        self.assertFalse(any("--creak" in c for c in calls), calls)

    def test_narrate_ignores_a_creak_price_without_joint(self):
        _, calls = self.narrate_recorded(edges_failures=0, env_extra={"JOINT_CREAK": "1"})
        self.assertFalse(any("--creak" in c or "--joint" in c for c in calls), calls)

    def dispatch_executable(self, directory, name, source):
        path = directory / name
        path.write_text(source, encoding="utf-8")
        path.chmod(0o755)
        return path

    def score_dispatch(self, mutate=None, voice=None, override=False, failure=0):
        # Dispatch unit test only: real PCM WAVs and the real driver/hash check,
        # with a command fixture instead of compiling/rendering the corpus.
        crate = self.crate("capture_recapture")
        (crate / "Cargo.toml").write_text(
            '[package]\nname = "capture_recapture"\n[[bin]]\nname = "video"\n',
            encoding="utf-8",
        )
        primary = crate / "narration" / "out"
        selected = crate / "variants" / voice / "narration" / "out" if voice else primary
        if override:
            selected = self.root / "custom narration"
        for directory in dict.fromkeys((primary, selected)):
            directory.mkdir(parents=True)
            with wave.open(str(directory / "line.wav"), "wb") as audio:
                audio.setparams((1, 2, 16000, 0, "NONE", "not compressed"))
                audio.writeframes(b"\x00\x00" * 160)
        binary_dir = self.root / "score tools"
        binary_dir.mkdir()
        target = self.root / "score target"
        (target / "release").mkdir(parents=True)
        calls = self.root / "score-calls.txt"
        output = self.root / "score-output.txt"
        self.dispatch_executable(binary_dir, "cargo", "#!/bin/sh\nexit 0\n")
        self.dispatch_executable(
            target / "release", "video",
            "#!/usr/bin/env python3\nimport os, sys\nfrom pathlib import Path\n"
            f"calls = Path({str(calls)!r})\n"
            "with calls.open('a') as log: log.write(sys.argv[1] + '\\n')\n"
            f"stage = {mutate!r}\n"
            "if stage == sys.argv[1] or (stage == 'primary' and sys.argv[1] == 'render'):\n"
            f"    path = Path({str(primary / 'line.wav')!r} if stage == 'primary' else {str(selected / 'line.wav')!r})\n"
            "    before = path.stat()\n"
            "    data = bytearray(path.read_bytes()); data[-2] = 1\n"
            "    path.write_bytes(data)\n"
            "    os.utime(path, ns=(before.st_atime_ns, before.st_mtime_ns))\n"
            "if sys.argv[1] == 'score':\n"
            f"    Path({str(output)!r}).write_text('dispatch output, not media\\n')\n"
            f"    raise SystemExit({failure})\n",
        )
        env = dict(os.environ, SKILLS=str(self.root / "missing skills"),
                   CARGO_TARGET_DIR=str(target),
                   PATH=str(binary_dir) + os.pathsep + os.environ["PATH"])
        for key in ("VOICE", "NARRATION"):
            env.pop(key, None)
        if voice:
            env["VOICE"] = voice
        if override:
            env["NARRATION"] = str(selected)
        result = self.run_x("capture_recapture", "score", env=env)
        return result, calls.read_text().splitlines(), output

    def test_score_runs_both_passes_with_unchanged_narration(self):
        result, calls, output = self.score_dispatch()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls, ["render", "score"])
        self.assertTrue(output.exists())
        self.assertIn("score used unchanged narration", result.stdout)

    def test_score_refuses_a_same_size_same_mtime_wave_change_before_music(self):
        result, calls, output = self.score_dispatch(mutate="render")
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(calls, ["render"])
        self.assertFalse(output.exists())
        self.assertIn("narration changed during the score timeline pass", result.stderr)

    def test_score_refuses_a_change_during_music_and_keeps_partial_output(self):
        result, calls, output = self.score_dispatch(mutate="score")
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(calls, ["render", "score"])
        self.assertTrue(output.exists())
        self.assertIn("narration changed during the music score pass", result.stderr)
        self.assertNotIn("score used unchanged narration", result.stdout)

    def test_score_variant_does_not_watch_the_primary_voice(self):
        result, calls, _ = self.score_dispatch(mutate="primary", voice="michael")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls, ["render", "score"])

    def test_score_watches_an_explicit_narration_override(self):
        result, calls, _ = self.score_dispatch(mutate="render", override=True)
        self.assertEqual(result.returncode, 2, result.stderr)
        self.assertEqual(calls, ["render"])
        self.assertIn("narration changed", result.stderr)

    def test_score_keeps_the_original_music_failure_status(self):
        result, calls, _ = self.score_dispatch(failure=17)
        self.assertEqual(result.returncode, 17, result.stderr)
        self.assertEqual(calls, ["render", "score"])
        self.assertNotIn("score used unchanged narration", result.stdout)

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

    def test_new_scaffolds_from_template_and_replaces_all_placeholders(self):
        shutil.copytree(ROOT / "templates", self.root / "templates")
        result = self.run_x("new", "demo_explainer", 'Demo "Quote" Title')
        self.assertEqual(result.returncode, 0, result.stderr)
        dest = self.corpus / "demo_explainer"
        self.assertTrue((dest / "Cargo.toml").is_file())
        self.assertTrue((dest / "src" / "main.rs").is_file())
        self.assertTrue((dest / "oracle.py").is_file())
        for path in dest.rglob("*"):
            if path.is_file():
                text = path.read_text(encoding="utf-8")
                for token in ("__SLUG__", "__TITLE__", "__TITLE_TOML__"):
                    self.assertNotIn(token, text, f"unreplaced {token} in {path}")
        manifest = (dest / "Cargo.toml").read_text(encoding="utf-8")
        self.assertIn('name = "demo_explainer"', manifest)
        self.assertIn('Demo \\"Quote\\" Title', manifest)
        # Refuses non-empty existing directory and invalid slug names
        again = self.run_x("new", "demo_explainer", "Again")
        self.assertEqual(again.returncode, 2)
        self.assertIn("already exists", again.stderr)
        bad = self.run_x("new", "Bad-Slug", "Title")
        self.assertEqual(bad.returncode, 2)
        self.assertIn("snake_case", bad.stderr)

    def test_status_and_reviewed_track_gate_transitions_and_staleness(self):
        shutil.copytree(ROOT / "templates", self.root / "templates")
        self.assertEqual(self.run_x("new", "demo_explainer", "Demo").returncode, 0)
        crate = self.corpus / "demo_explainer"
        res = self.run_x("demo_explainer", "status", "--porcelain")
        self.assertEqual(res.returncode, 0, res.stderr)
        gates = {line.split("\t")[0]: line.split("\t")[1] for line in res.stdout.splitlines()}
        self.assertEqual(gates["G0 brief"], "todo")
        self.assertEqual(gates["G0 protected"], "ok")
        self.assertEqual(gates["G5 drafts"], "todo")
        # Clear TODOs in brief.md -> G0 brief becomes ok
        brief = crate / "docs" / "brief.md"
        brief.write_text(brief.read_text(encoding="utf-8").replace("TODO", "filled"), encoding="utf-8")
        res2 = self.run_x("demo_explainer", "status", "--porcelain")
        gates2 = {line.split("\t")[0]: line.split("\t")[1] for line in res2.stdout.splitlines()}
        self.assertEqual(gates2["G0 brief"], "ok")
        # Record reviewed draft
        rev = self.run_x("demo_explainer", "reviewed", "draft")
        self.assertEqual(rev.returncode, 0, rev.stderr)
        self.assertTrue((crate / ".x" / "draft.reviewed").is_file())

    def test_fmn_local_refuses_g8_final_master_and_verify(self):
        self.crate("alpha")
        fm = self.root / "fake_fmn"
        (fm / "crates" / "fmn").mkdir(parents=True)
        env = dict(os.environ, SKILLS=str(self.root / "missing skills"), FRANKEN_MANIM=str(fm))
        for cmd in ("final", "master", "verify"):
            res = self.run_x("--fmn-local", "alpha", cmd, env=env)
            self.assertEqual(res.returncode, 2)
            self.assertIn("G8 refuses --fmn-local", res.stderr)

    def test_publish_assembles_release_bundle_and_refuses_unapproved_upload(self):
        crate = self.crate("alpha", "Alpha")
        (crate / "Cargo.toml").write_text('[package]\nname = "alpha"\n[[bin]]\nname = "alpha"\n', encoding="utf-8")
        finals = crate / "renders" / "final"
        finals.mkdir(parents=True)
        master_mp4 = finals / "alpha_master.mp4"
        subprocess.run(
            [
                "ffmpeg", "-hide_banner", "-loglevel", "error", "-y",
                "-f", "lavfi", "-i", "color=c=0x0E1116:s=640x360:r=15:d=0.2",
                "-c:v", "libx264", "-pix_fmt", "yuv420p", str(master_mp4),
            ],
            check=True,
            timeout=30,
        )
        (finals / "alpha.en.srt").write_text("1\n00:00:00,000 --> 00:00:01,000\nHello.\n", encoding="utf-8")
        (finals / "alpha.en.vtt").write_text("WEBVTT\n\n00:00:00.000 --> 00:00:01.000\nHello.\n", encoding="utf-8")
        # no package: refused before anything is assembled, every missing file named
        res = self.run_x("alpha", "publish")
        self.assertNotEqual(res.returncode, 0)
        self.assertIn("youtube.md: missing", res.stderr)
        self.assertFalse((crate / "publish" / "release_v1").exists())
        # a package that passes tools/youtube_package.py check
        pub = crate / "publish"
        (pub / "michael").mkdir(parents=True)
        desc = "Alpha.\n\nChapters\n0:00 One\n0:20 Two\n0:40 Three\n"
        (pub / "michael" / "title.txt").write_text("Alpha\n", encoding="utf-8")
        (pub / "michael" / "tags.txt").write_text("alpha, beta\n", encoding="utf-8")
        (pub / "michael" / "description.txt").write_text(desc, encoding="utf-8")
        (pub / "youtube.md").write_text(
            f"# Alpha\n\n- Title: Alpha\n- Tags: alpha, beta\n\n```text\n{desc}```\n", encoding="utf-8"
        )
        res = self.run_x("alpha", "publish")
        self.assertEqual(res.returncode, 0, res.stderr)
        bundle = crate / "publish" / "release_v1"
        self.assertTrue((bundle / "alpha_4k_v1.mp4").is_file())
        self.assertTrue((bundle / "alpha_1080p_v1.mp4").is_file())
        self.assertTrue((bundle / "alpha.en.srt").is_file())
        self.assertTrue((bundle / "alpha.en.vtt").is_file())
        self.assertTrue((bundle / "contact_sheet.png").is_file())
        self.assertTrue((bundle / "release_notes.md").is_file())
        self.assertTrue((bundle / "SHA256SUMS").is_file())
        self.assertTrue((crate / "publish" / "youtube.md").is_file())
        up = self.run_x("alpha", "publish", "--upload")
        self.assertEqual(up.returncode, 2)
        self.assertIn("G10", up.stderr)

    def test_narration_prosody_passes_gate_line_gaps_when_crate_opts_into_thought(self):
        crate = self.crate("alpha")
        (crate / "narration" / "out").mkdir(parents=True)
        (crate / "src").mkdir()
        (crate / "src" / "lib.rs").write_text("pub fn gaps() { let _ = LineGaps::Constant; }\n", encoding="utf-8")
        binary_dir = self.root / "prosody tools"
        binary_dir.mkdir()
        observed = self.root / "prosody-args.json"
        self.dispatch_executable(
            binary_dir, "uv",
            "#!/usr/bin/env python3\nimport json, sys\nfrom pathlib import Path\n"
            f"Path({str(observed)!r}).write_text(json.dumps(sys.argv[1:]))\n"
            "raise SystemExit(0)\n",
        )
        env = dict(os.environ, PATH=str(binary_dir) + os.pathsep + os.environ["PATH"])
        res1 = self.run_x("alpha", "narration-prosody", env=env)
        self.assertEqual(res1.returncode, 0, res1.stderr)
        args1 = json.loads(observed.read_text(encoding="utf-8"))
        self.assertNotIn("--gate-line-gaps", args1)

        (crate / "src" / "lib.rs").write_text("pub fn gaps() { let _ = LineGaps::Thought; }\n", encoding="utf-8")
        res2 = self.run_x("alpha", "narration-prosody", env=env)
        self.assertEqual(res2.returncode, 0, res2.stderr)
        args2 = json.loads(observed.read_text(encoding="utf-8"))
        self.assertIn("--gate-line-gaps", args2)


if __name__ == "__main__":
    unittest.main()

