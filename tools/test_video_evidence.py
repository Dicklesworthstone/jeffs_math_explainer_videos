#!/usr/bin/env python3
"""Real subprocess/file-hash tests on synthetic inputs, not acoustic qualification.

uv run --no-project --with jsonschema python3 -I tools/test_video_evidence.py
"""
import contextlib
import copy
import importlib.util
import io
import json
import os
from pathlib import Path
import sys
import subprocess
import tempfile
import unittest


def module(name, filename):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(filename))
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


evidence = module("video_evidence", "video_evidence.py")
fixtures = module("catalog_fixtures", "test_video_catalog.py")


class EvidenceTests(unittest.TestCase):
    def setUp(self):
        # Repository instructions require retaining even our own test fixtures.
        self.root = Path(tempfile.mkdtemp(prefix="video-evidence-test-"))
        self.data = fixtures.fixture()
        for artifact in self.data["artifacts"]:
            path = self.root / artifact["path"]
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("fixture " + artifact["kind"])
            artifact["sha256"] = evidence.file_digest(path)
        self.checker = self.root / "checker.py"
        self.checker.write_text(
            "import json\nfrom pathlib import Path\n"
            "ok = Path('fixture/script').read_text().startswith('fixture ')\n"
            "Path('measurement.txt').write_text('synthetic script prefix check: ' + str(ok))\n"
            "print(json.dumps({'verdict':'pass' if ok else 'fail','output_refs':['measurement.txt'],"
            "'summary':'Synthetic script prefix only; no acoustic or production claim.'}))\n")
        self.sequence = 0

    def run_gate(self, gate="master", **kwargs):
        self.sequence += 1
        return evidence.run_checker(self.data, self.root, "hoeffding_d", "replacement", gate,
                                    [sys.executable, "-I", "checker.py"], "fixture-checker-1",
                                    ["checker.py"], "test-fixture", self.root / f"receipt-{self.sequence}.json", **kwargs)

    def check(self, receipt, gate=None, revision="replacement"):
        return evidence.evaluate(receipt, self.data, self.root, "hoeffding_d", revision, gate or receipt["gate"])

    def mutate(self, role, update_catalog=True):
        artifact = next(a for a in self.data["artifacts"] if a["kind"] == role)
        path = self.root / artifact["path"]
        path.write_text("fixture changed " + role)
        if update_catalog:
            artifact["sha256"] = evidence.file_digest(path)

    def test_positive_real_subprocess_and_fresh_hashes(self):
        receipt = self.run_gate()
        self.assertEqual(receipt["verdict"], "pass")
        self.assertEqual(receipt["returncode"], 0)
        self.assertTrue(self.check(receipt)["eligible_for_required_gate"])
        self.assertEqual(receipt["outputs"]["measurement.txt"], evidence.file_digest(self.root / "measurement.txt"))
        self.assertEqual(evidence._schema_errors(receipt, evidence.catalog.read_json(evidence.RECEIPT_SCHEMA)), [])

    def test_mutation_matrix(self):
        # Every role is changed against independently captured, otherwise identical
        # receipts. No success depends on a mocked stat, digest or checker runner.
        unaffected = {"music": {"script", "dry_voice", "timing", "visuals"},
                      "package": {"script", "dry_voice", "timing", "visuals", "mix", "master"},
                      "master": {"script", "dry_voice", "timing", "visuals"},
                      "scene": {"script", "dry_voice", "timing"},
                      "cue": {"script", "dry_voice"},
                      "voice": {"script"}, "take": {"script"}, "lexicon": set(), "script": set()}
        originals = copy.deepcopy(self.data)
        original_bytes = {a["path"]: (self.root / a["path"]).read_bytes() for a in self.data["artifacts"]}
        for role in evidence.catalog.ROLES:
            with self.subTest(role=role):
                self.data = copy.deepcopy(originals)
                for path, content in original_bytes.items(): (self.root / path).write_bytes(content)
                receipts = {gate: self.run_gate(gate) for gate in evidence.POLICY}
                self.mutate(role)
                for gate, receipt in receipts.items():
                    self.assertEqual(self.check(receipt)["eligible_for_required_gate"], gate in unaffected[role], (role, gate))

    def test_renamed_changed_bytes_and_mtime_only(self):
        receipt = self.run_gate("dry_voice")
        artifact = self.data["artifacts"][0]
        path = self.root / artifact["path"]
        os.utime(path, (1_000_000_000, 1_000_000_000))
        self.assertTrue(self.check(receipt)["eligible_for_required_gate"])
        new = self.root / "renamed-script"
        new.write_text("fixture different bytes at a new name")
        artifact["path"] = "renamed-script"
        artifact["sha256"] = evidence.file_digest(new)
        self.assertFalse(self.check(receipt)["eligible_for_required_gate"])

    def test_changed_bytes_without_updating_catalog(self):
        receipt = self.run_gate()
        self.mutate("script", update_catalog=False)
        result = self.check(receipt)
        self.assertFalse(result["eligible_for_required_gate"])
        self.assertIn("current bytes differ", str(result))
        with self.assertRaisesRegex(ValueError, "current bytes differ"):
            self.run_gate()

    def test_two_revisions_and_voice_identity(self):
        receipt = self.run_gate()
        revision = copy.deepcopy(self.data["videos"][0]["revisions"][1])
        revision["revision_id"] = "simultaneous-cut"
        self.data["videos"][0]["revisions"].append(revision)
        self.assertFalse(self.check(receipt, revision="simultaneous-cut")["eligible_for_required_gate"])
        self.assertTrue(self.check(receipt)["eligible_for_required_gate"])
        self.data["videos"][0]["revisions"][1]["voice"] = "different-voice"
        self.assertFalse(self.check(receipt)["eligible_for_required_gate"])

    def test_missing_corrupt_unknown_and_wrong_gate_receipts(self):
        receipt = self.run_gate()
        for mutation in [None, {}, {**receipt, "schema_version": 2}, {**receipt, "inputs": {}},
                         {**receipt, "returncode": 2}, {**receipt, "report": None},
                         {**receipt, "stdout": "corrupted"}, {**receipt, "verdict": "unknown"}]:
            with self.subTest(mutation=type(mutation).__name__):
                self.assertFalse(self.check(mutation, gate="master")["eligible_for_required_gate"])
        self.assertFalse(self.check(receipt, gate="dry_voice")["eligible_for_required_gate"])

    def test_checker_crash_missing_report_malformed_and_failed_measurement(self):
        for code in ["raise RuntimeError('checker crash')", "pass", "print('not json')",
                     "print('{\"verdict\":\"pass\"}')",
                     "import json; print(json.dumps({'verdict':'fail','output_refs':[],'summary':'measured failure'}))",
                     "import json; print(json.dumps({'verdict':'pass','output_refs':['absent.txt'],'summary':'missing output'}))"]:
            with self.subTest(code=code):
                self.checker.write_text(code)
                receipt = self.run_gate()
                self.assertEqual(receipt["verdict"], "fail")
                self.assertFalse(self.check(receipt)["eligible_for_required_gate"])

    def test_bounded_timeout_and_output(self):
        for code, timeout in [("import time; time.sleep(5)", 0.1), ("print('x'*1100000)", 2.0)]:
            with self.subTest(code=code):
                self.checker.write_text(code)
                receipt = self.run_gate(timeout=timeout)
                self.assertEqual(receipt["verdict"], "fail")
                self.assertTrue(receipt["problems"])

    def test_mid_check_input_mutation(self):
        self.checker.write_text("import json\nfrom pathlib import Path\nPath('fixture/script').write_text('changed')\nprint(json.dumps({'verdict':'pass','output_refs':[],'summary':'invalidated during check'}))")
        receipt = self.run_gate()
        self.assertEqual(receipt["verdict"], "fail")
        self.assertIn("post-check input verification failed", str(receipt["problems"]))

    def test_dirty_checker_and_changed_output(self):
        receipt = self.run_gate()
        original = self.checker.read_bytes()
        self.checker.write_bytes(original + b"\n# dirty source\n")
        self.assertIn("dirty checker source", str(self.check(receipt)))
        self.checker.write_bytes(original)
        (self.root / "measurement.txt").write_text("altered measurement")
        self.assertIn("checker output changed", str(self.check(receipt)))

    def test_advisory_pass_cannot_satisfy_required_gate(self):
        receipt = self.run_gate(advisory=True)
        self.assertEqual(receipt["verdict"], "pass")
        self.assertFalse(self.check(receipt)["eligible_for_required_gate"])
        self.assertIn("advisory", str(self.check(receipt)))

    def test_incomplete_legacy_cannot_mint_pass(self):
        with self.assertRaisesRegex(ValueError, "unrecorded required roles"):
            evidence.snapshot(self.data, self.root, "hoeffding_d", "legacy", "master")

    def test_transitive_model_input_and_new_optional_music(self):
        model = self.root / "model.bin"
        model.write_bytes(b"small synthetic model identity")
        self.data["artifacts"].append({"artifact_id": "model-v1", "kind": "model", "path": "model.bin",
                                       "sha256": evidence.file_digest(model), "inputs": [],
                                       "evidence_state": "unknown", "evidence_refs": ["synthetic only"]})
        next(a for a in self.data["artifacts"] if a["kind"] == "voice")["inputs"] = ["model-v1"]
        receipt = self.run_gate("dry_voice")
        self.assertTrue(self.check(receipt)["eligible_for_required_gate"])
        model.write_bytes(b"changed synthetic model")
        self.assertFalse(self.check(receipt)["eligible_for_required_gate"])

    def test_adding_music_invalidates_integrated_but_not_dry_voice(self):
        revision = self.data["videos"][0]["revisions"][1]
        music = revision["artifact_refs"].pop("music")
        master = next(a for a in self.data["artifacts"] if a["kind"] == "master")
        master["inputs"].remove("music-v1")
        dry, integrated = self.run_gate("dry_voice"), self.run_gate("master")
        revision["artifact_refs"]["music"] = music
        self.assertTrue(self.check(dry)["eligible_for_required_gate"])
        self.assertFalse(self.check(integrated)["eligible_for_required_gate"])

    def test_receipt_is_exclusive_and_symlink_escape_rejected(self):
        self.run_gate()
        self.sequence = 0
        with self.assertRaises(FileExistsError): self.run_gate()
        link = self.root / "escape"
        link.symlink_to(Path(__file__))
        with self.assertRaisesRegex(ValueError, "inside repository"):
            evidence.local_file(self.root, "escape")

    def test_missing_receipt_cli_is_blocked(self):
        with contextlib.redirect_stdout(io.StringIO()) as output:
            code = evidence.main(["--video", "hoeffding_d", "--revision", "legacy-savJOtXm7AQ", "--gate", "master",
                                  "check", str(self.root / "nonexistent.json")])
        self.assertEqual(code, 1)
        self.assertFalse(json.loads(output.getvalue())["eligible_for_required_gate"])

    def test_cli_run_check_then_changed_file_blocks(self):
        manifest = self.root / "catalog.json"
        manifest.write_text(json.dumps(self.data))
        receipt = self.root / "cli-receipt.json"
        command = [sys.executable, "-I", str(Path(evidence.__file__)), "--catalog", str(manifest),
                   "--root", str(self.root), "--video", "hoeffding_d", "--revision", "replacement", "--gate", "master"]
        run = subprocess.run(command + ["run", "--receipt", str(receipt), "--checker-version", "fixture-1",
                                       "--checker-file", "checker.py", "--reviewer", "fixture", "--",
                                       sys.executable, "-I", "checker.py"], capture_output=True, text=True)
        self.assertEqual(run.returncode, 0, run.stdout + run.stderr)
        check = subprocess.run(command + ["check", str(receipt)], capture_output=True, text=True)
        self.assertEqual(check.returncode, 0, check.stdout + check.stderr)
        self.mutate("music", update_catalog=False)
        check = subprocess.run(command + ["check", str(receipt)], capture_output=True, text=True)
        self.assertEqual(check.returncode, 1, check.stdout + check.stderr)
        self.assertIn("current bytes differ", check.stdout)
        self.assertTrue(receipt.exists())


if __name__ == "__main__":
    unittest.main()
