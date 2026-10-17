#!/usr/bin/env python3
"""Real br integration on retained synthetic trackers; no audio quality claims.

uv run --no-project --with jsonschema python3 -I tools/test_video_lifecycle.py
Native integration is explicitly skipped when br is absent (e.g. an unprovisioned CI runner).
"""
import copy
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


life = module("video_lifecycle", "video_lifecycle.py")
fixtures = module("evidence_fixtures", "test_video_evidence.py")


@unittest.skipUnless(shutil.which("br"), "native br is not installed; native gate qualification was NOT run")
class LifecycleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.retained = Path(tempfile.mkdtemp(prefix="video-lifecycle-tests-"))
        print("Retained native fixtures: " + str(cls.retained), flush=True)

    def setUp(self):
        self.fx = fixtures.EvidenceTests()
        self.fx.setUp()
        self.root = self.fx.root
        (self.retained / (self._testMethodName + ".txt")).write_text(str(self.root) + "\n")
        (self.root / ".beads").mkdir()
        life.br(self.root, "init", "--prefix", "a4")
        (self.root / ".beads/policy.yaml").write_text(json.dumps(life.NATIVE_POLICY))
        issue = life.br(self.root, "create", "Synthetic exact-cut fixture", "--labels", life.LABEL)
        self.issue = issue["id"]
        self.receipt = self.fx.run_gate("dry_voice")
        (self.root / "qualification.md").write_text("Synthetic prefix checker is qualified ONLY for this integration fixture. No acoustic claim.\n")
        self.policy = {"schema_version": 1, "profiles": {"fixture": {
            "gate": "dry_voice", "checker_sha256": life.catalog.digest(self.receipt["checker"]),
            "scope": "synthetic script-prefix test only", "qualification_ref": "qualification.md"}},
            "bindings": [{"issue": self.issue, "video_id": "hoeffding_d", "revision_id": "replacement",
                          "voice": self.receipt["subject"]["voice"],
                          "checks": [{"profile": "fixture", "receipt": "receipt-1.json"}]}]}
        (self.root / "docs/video-program").mkdir(parents=True)
        self.save()

    def save(self):
        (self.root / "docs/video-program/catalog.json").write_text(json.dumps(self.fx.data))
        (self.root / life.POLICY_PATH).write_text(json.dumps(self.policy))

    def close(self, root=None, voice=None, revision="replacement"):
        return life.close_task(root or self.root, "hoeffding_d", revision, voice, self.issue)

    def assert_open(self):
        self.assertEqual(life.issue_state(self.root, self.issue)["status"], "open")

    def native_blocked(self, root=None):
        with self.assertRaisesRegex(ValueError, "requires gate"):
            life.br(root or self.root, "close", self.issue)

    def test_positive_native_closure_and_portable_comment(self):
        self.native_blocked()
        result = self.close()
        self.assertEqual(result["status"], "closed")
        issue = life.issue_state(self.root, self.issue)
        self.assertIn("receipt-1.json", json.dumps(issue["comments"]))
        self.assertIn(life.evidence.file_digest(self.root / "receipt-1.json"), json.dumps(issue["comments"]))

    def test_stale_audio_revokes_preexisting_native_pass(self):
        life.report(self.root, self.issue, True, "fixture prior pass")
        self.fx.mutate("take", update_catalog=False)
        with self.assertRaisesRegex(ValueError, "current bytes differ"):
            self.close()
        self.assert_open()
        self.native_blocked()
        row = life.status(self.root, "hoeffding_d", "replacement")["revisions"][0]
        self.assertIn("current bytes differ", row["next_blocker"])

    def test_updated_catalog_hash_still_stales_receipt(self):
        self.fx.mutate("take")
        self.save()
        with self.assertRaisesRegex(ValueError, "inputs or bindings changed"):
            self.close()
        self.native_blocked()

    def test_wrong_voice_and_revision_never_fallback(self):
        for voice, revision in [("not-the-recorded-voice", "replacement"), (None, "draft")]:
            with self.subTest(voice=voice, revision=revision):
                with self.assertRaisesRegex(ValueError, "differs from task binding"):
                    self.close(voice=voice, revision=revision)
        self.fx.data["videos"][0]["revisions"][1]["voice"] = "changed-voice"
        self.save()
        with self.assertRaisesRegex(ValueError, "voice|subject"):
            self.close()
        self.native_blocked()

    def test_leave_reenter_requires_new_native_result(self):
        life.report(self.root, self.issue, True, "fixture prior pass")
        life.br(self.root, "update", self.issue, "--status", "in_progress")
        life.br(self.root, "update", self.issue, "--status", "open")
        self.native_blocked()
        self.assertEqual(self.close()["status"], "closed")
        life.br(self.root, "reopen", self.issue)
        self.native_blocked()

    def test_clone_import_revalidates_instead_of_inheriting_gate(self):
        # Ordinary comments + unchanged files travel; gate metadata does not.
        life.br(self.root, "comments", "add", self.issue, "Portable fixture receipt: receipt-1.json")
        life.report(self.root, self.issue, True, "source-machine-only gate pass")
        life.br(self.root, "sync", "--flush-only")
        clone = self.retained / "clone"
        clone.mkdir()
        for entry in self.root.iterdir():
            if entry.name in (".beads", ".x"): continue
            if entry.is_dir(): shutil.copytree(entry, clone / entry.name)
            else: shutil.copy2(entry, clone / entry.name)
        (clone / ".beads").mkdir()
        life.br(clone, "init", "--prefix", "a4")
        shutil.copy2(self.root / ".beads/policy.yaml", clone / ".beads/policy.yaml")
        shutil.copy2(self.root / ".beads/issues.jsonl", clone / ".beads/issues.jsonl")
        life.br(clone, "sync", "--import-only")
        self.native_blocked(clone)
        self.assertIn("receipt-1.json", json.dumps(life.issue_state(clone, self.issue)["comments"]))
        original = (clone / "fixture/take").read_bytes()
        (clone / "fixture/take").write_bytes(original + b"changed clone")
        with self.assertRaisesRegex(ValueError, "current bytes differ"):
            self.close(root=clone)
        self.native_blocked(clone)
        (clone / "fixture/take").write_bytes(original)
        self.assertEqual(self.close(root=clone)["status"], "closed")

    def test_unknown_checker_missing_receipt_and_advisory_block(self):
        original = copy.deepcopy(self.policy)
        self.policy["profiles"]["fixture"]["checker_sha256"] = "0" * 64
        self.save()
        with self.assertRaisesRegex(ValueError, "qualified pinned profile"):
            self.close()
        self.policy = copy.deepcopy(original)
        self.policy["bindings"][0]["checks"][0]["receipt"] = "absent.json"
        self.save()
        with self.assertRaises(ValueError): self.close()
        advisory = self.fx.run_gate("dry_voice", advisory=True)
        self.policy = copy.deepcopy(original)
        self.policy["bindings"][0]["checks"][0]["receipt"] = "receipt-2.json"
        self.save()
        self.assertEqual(advisory["classification"], "advisory")
        with self.assertRaisesRegex(ValueError, "advisory"):
            self.close()
        self.native_blocked()

    def test_strict_false_and_missing_label_block_adapter(self):
        policy = copy.deepcopy(life.NATIVE_POLICY)
        policy["workflow"]["strict"] = False
        (self.root / ".beads/policy.yaml").write_text(json.dumps(policy))
        with self.assertRaisesRegex(ValueError, "strict"):
            self.close()
        (self.root / ".beads/policy.yaml").write_text(json.dumps(life.NATIVE_POLICY))
        life.br(self.root, "label", "remove", self.issue, life.LABEL)
        with self.assertRaisesRegex(ValueError, "carry"):
            self.close()

    def test_ordinary_task_unaffected_and_legacy_marks_not_authority(self):
        normal = life.br(self.root, "create", "ordinary task")
        life.br(self.root, "close", normal["id"])
        (self.root / ".x").mkdir()
        (self.root / ".x/master.ok").write_text("arbitrary old pass, wrong voice")
        result = life.status(self.root, "hoeffding_d", "legacy")
        self.assertEqual(result["revisions"][0]["state"], "blocked")
        self.assertIn("historical markers", result["revisions"][0]["next_blocker"])

    def test_reservation_inherits_and_excludes_second_producer(self):
        probe = self.root / "lock_probe.py"
        probe.write_text("import fcntl, sys\nfrom pathlib import Path\n"
                         "with Path('.x/producer.lock').open('a+') as f:\n"
                         " try: fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)\n"
                         " except BlockingIOError: sys.exit(0)\n"
                         " sys.exit(1)\n")
        command = [sys.executable, "-I", str(Path(life.__file__)), "--root", str(self.root)]
        with life.reservation(self.root) as fd:
            result = subprocess.run([sys.executable, "-I", str(probe)], cwd=self.root)
            self.assertEqual(result.returncode, 0)
            inherited = subprocess.run(command + ["verify-reservation"], env={**os.environ, life.LOCK_ENV: str(fd)}, pass_fds=(fd,))
            self.assertEqual(inherited.returncode, 0)
        self.assertEqual(subprocess.run([sys.executable, "-I", str(probe)], cwd=self.root).returncode, 1)
        self.assertTrue((self.root / ".x/producer.lock").exists())

    def test_other_native_requirement_aborts_and_revokes_our_pass(self):
        policy = copy.deepcopy(life.NATIVE_POLICY)
        policy["workflow"]["gates"]["open -> closed"]["require_all"] = ["another_required_check"]
        (self.root / ".beads/policy.yaml").write_text(json.dumps(policy))
        with self.assertRaisesRegex(ValueError, "another_required_check"):
            self.close()
        self.assert_open()
        edge = life.br(self.root, "gate", "list", self.issue)["gated_transitions"][0]
        own = next(g for g in edge["gates"] if g["gate"] == life.GATE)
        self.assertFalse(own["satisfied"])


class ProsodyExecutionTests(unittest.TestCase):
    def test_real_cli_rejects_missing_crashed_empty_and_failed_reports(self):
        root = Path(tempfile.mkdtemp(prefix="prosody-execution-test-"))
        helper = Path(__file__).with_name("check_prosody_report.py")
        rows = [("PASS    useful check\n", 0, 0), ("ADVISORY uncertain\nFLAG uncertain\n", 0, 0),
                ("PASS stale output from failed process\n", 2, 1), ("", 0, 1),
                ("Traceback from checker\n", 0, 1), ("PASS one\nFAIL two\n", 0, 1),
                (None, 0, 1)]
        for index, (content, code, expected) in enumerate(rows):
            with self.subTest(index=index):
                path = root / f"report-{index}.txt"
                if content is not None: path.write_text(content)
                result = subprocess.run([sys.executable, "-I", str(helper), str(path), str(code)])
                self.assertEqual(result.returncode, expected)


if __name__ == "__main__":
    unittest.main()
