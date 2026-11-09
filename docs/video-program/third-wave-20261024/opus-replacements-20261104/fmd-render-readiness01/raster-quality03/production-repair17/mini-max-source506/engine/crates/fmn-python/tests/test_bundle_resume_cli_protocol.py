"""Bundle checkpoint options through the shipping CLI with a modeled native sink."""
import json
from pathlib import Path
import unittest

import test_bundle_batch_cli_protocol as fixtures


class BundleResumeCliProtocol(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.BundleBatchCliProtocol()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.root = self.fixture.root
        self.progress = self.root / "progress.json"

    def invoke(self, *extra, names=()):
        return self.fixture.batch("--checkpoint", str(self.progress), "--resume-key", "inputs-v1",
                                  *extra, names=names)

    def test_camera_write_all_resume_uses_no_new_native_generation(self):
        code, first, _ = self.invoke("--write_all", "--bundle-camera")
        self.assertEqual(code, 0, first)
        self.fixture.calls.clear()
        self.fixture.native.instances.clear()
        code, resumed, _ = self.invoke("--write_all", "--bundle-camera", "--resume")
        self.assertEqual(code, 0, resumed)
        self.assertEqual(first, resumed)
        self.assertEqual(self.fixture.calls, [])
        self.assertEqual(self.fixture.native.instances, [])

    def test_explicit_planar_order_resumes_and_changed_order_refuses(self):
        code, first, _ = self.invoke(names=("Zulu", "Alpha"))
        self.assertEqual(code, 0, first)
        self.fixture.calls.clear()
        self.fixture.native.instances.clear()
        code, resumed, _ = self.invoke("--resume", names=("Zulu", "Alpha"))
        self.assertEqual(code, 0, resumed)
        self.assertEqual(first, resumed)
        before = self.progress.read_bytes()
        code, error, _ = self.invoke("--resume", names=("Alpha", "Zulu"))
        self.assertEqual(code, 2)
        self.assertIn("plan", error["message"])
        self.assertEqual(self.fixture.calls, [])
        self.assertEqual(self.fixture.native.instances, [])
        self.assertEqual(self.progress.read_bytes(), before)

    def test_corrupt_completed_file_refuses_without_overwriting_or_constructing(self):
        code, report, _ = self.invoke("--write_all")
        self.assertEqual(code, 0, report)
        artifact = Path(report["batch"]["outcomes"][-1]["destination"])
        artifact.write_bytes(b"tampered")
        self.fixture.calls.clear()
        self.fixture.native.instances.clear()
        code, refused, _ = self.invoke("--write_all", "--resume")
        self.assertEqual(code, 2, refused)
        self.assertIn("modified", refused["message"])
        self.assertEqual(self.fixture.calls, [])
        self.assertEqual(self.fixture.native.instances, [])
        self.assertEqual(artifact.read_bytes(), b"tampered")

    def test_duplicate_missing_and_single_scene_recovery_options_are_usage(self):
        self.fixture.source.write_text('raise AssertionError("source must not execute")\n')
        cases = (("--write_all", "--resume"),
                 ("--write_all", "--checkpoint", str(self.progress)),
                 ("--write_all", "--resume-key", "inputs"),
                 ("--write_all", "--checkpoint", str(self.progress), "--resume-key", ""),
                 ("--write_all", "--checkpoint", str(self.progress), "--resume-key", "v1", "--resume", "--resume"),
                 ("--checkpoint", str(self.progress), "--resume-key", "v1"))
        for flags in cases:
            with self.subTest(flags=flags):
                code, report, _ = self.fixture.batch(*flags)
                self.assertEqual(code, 2, report)
                self.assertEqual(report["phase"], "options")
        self.assertEqual(self.fixture.calls, [])
        self.assertEqual(self.fixture.native.instances, [])

    def test_equals_options_and_flag_shaped_values_are_parsed_without_interception(self):
        code, first, _ = self.fixture.batch("--write_all", "--checkpoint=" + str(self.progress),
                                            "--resume-key=--bundle-camera")
        self.assertEqual(code, 0, first)
        self.assertNotIn("camera_track", first["batch"]["outcomes"][0]["result"])
        self.fixture.calls.clear()
        code, resumed, _ = self.fixture.batch("--write_all", "--checkpoint=" + str(self.progress),
                                              "--resume-key=--bundle-camera", "--resume")
        self.assertEqual(code, 0, resumed)
        self.assertEqual(first, resumed)
        self.assertEqual(self.fixture.calls, [])

    def test_resume_key_and_camera_changes_are_not_silent_cache_hits(self):
        code, first, _ = self.invoke("--write_all", "--bundle-camera")
        self.assertEqual(code, 0, first)
        self.fixture.calls.clear()
        self.fixture.native.instances.clear()
        code, error, _ = self.invoke("--write_all", "--resume")
        self.assertEqual(code, 2, error)
        code, error, _ = self.fixture.batch("--write_all", "--bundle-camera", "--resume",
                                            "--checkpoint", str(self.progress), "--resume-key", "different")
        self.assertEqual(code, 2, error)
        self.assertEqual(self.fixture.calls, [])
        self.assertEqual(self.fixture.native.instances, [])

    def test_resume_missing_journal_is_error_not_fresh_export(self):
        code, report, _ = self.invoke("--write_all", "--resume")
        self.assertEqual(code, 6, report)
        self.assertFalse(self.fixture.directory.exists())
        self.assertFalse(self.progress.exists())
        self.assertEqual(self.fixture.calls, [])
        self.assertEqual(self.fixture.native.instances, [])

    def test_checkpoint_leaf_symlink_is_refused_and_target_preserved(self):
        target = self.root / "original.json"
        target.write_text('{"user":"data"}')
        self.progress.symlink_to(target)
        code, report, _ = self.invoke("--write_all")
        self.assertEqual(code, 2, report)
        self.assertIn("symlink", report["message"])
        self.assertEqual(json.loads(target.read_text()), {"user": "data"})
        self.assertEqual(self.fixture.calls, [])
        self.assertEqual(self.fixture.native.instances, [])

    def test_checkpoint_paths_remain_frozen_when_source_changes_directory(self):
        changed = self.root / "changed"
        changed.mkdir()
        self.fixture.source.write_text("import os\nos.chdir(" + repr(str(changed)) + ")\n" + fixtures.SOURCE)
        code, report, _ = self.fixture.invoke("--robot", self.fixture.source, "--write_all", "--format=fmtl",
                                              "--checkpoint=progress.json", "--resume-key=v1", "--video_dir=output")
        self.assertEqual(code, 0, report)
        self.assertTrue(self.progress.is_file())
        self.assertFalse((changed / "progress.json").exists())
        self.assertTrue((self.fixture.directory / "Alpha.fmtl").is_file())

    def test_help_lists_recovery_without_loading_source(self):
        self.fixture.source.write_text('raise AssertionError("source must not execute")\n')
        code, report, _ = self.fixture.batch("--write_all", "--help")
        self.assertEqual(code, 0, report)
        self.assertIn("--checkpoint", report["help"])
        self.assertIn("--resume-key", report["help"])
        self.assertEqual(self.fixture.calls, [])


if __name__ == "__main__":
    unittest.main()
