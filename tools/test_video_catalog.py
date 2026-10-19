#!/usr/bin/env python3
"""uv run --no-project --with jsonschema python3 -I tools/test_video_catalog.py"""
import contextlib
import copy
import importlib.util
import io
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("video_catalog", Path(__file__).with_name("video_catalog.py"))
catalog = importlib.util.module_from_spec(spec)
spec.loader.exec_module(catalog)


def fixture():
    """Synthetic records only; these hashes do not assert any real media exists."""
    data = catalog.read_json(catalog.CATALOG)
    data["videos"] = [copy.deepcopy(data["videos"][0])]
    video = data["videos"][0]
    legacy = copy.deepcopy(video["revisions"][0])
    legacy["revision_id"] = "legacy"
    legacy["generation_provenance"] = "unknown"
    video["publications"] = [copy.deepcopy(video["publications"][0])]
    video["publications"][0]["revision_id"] = "legacy"
    complete = copy.deepcopy(legacy)
    complete.pop("master")
    complete["revision_id"] = "replacement"
    complete["generation_provenance"] = "Synthetic test fixture only"
    complete["milestones"] = []
    complete["artifact_refs"] = {role: [role + "-v1"] for role in catalog.ROLES}
    draft = copy.deepcopy(complete)
    draft["revision_id"] = "draft"
    draft["artifact_refs"] = {"script": ["script-v1"]}
    video["revisions"] = [legacy, complete, draft]
    parents = {"take": ["script-v1", "lexicon-v1", "voice-v1"],
               "cue": ["take-v1"], "scene": ["cue-v1"],
               "master": ["take-v1", "scene-v1", "music-v1"], "package": ["master-v1"]}
    data["artifacts"] = [
        {"artifact_id": role + "-v1", "kind": role, "path": "fixture/" + role,
         "sha256": catalog.digest({"fixture": role}), "inputs": parents.get(role, []),
         "evidence_state": "unknown", "evidence_refs": ["synthetic-test-only"]}
        for role in catalog.ROLES
    ]
    return data


class CatalogTests(unittest.TestCase):
    def assert_invalid(self, data, fragment):
        errors = catalog.validate(data)
        self.assertTrue(errors, "invalid fixture unexpectedly passed")
        self.assertIn(fragment, "\n".join(errors))

    def test_real_catalog_and_schema(self):
        self.assertEqual(catalog.validate(catalog.read_json(catalog.CATALOG)), [])

    def test_complete_draft_legacy_and_two_cuts_one_video(self):
        data = fixture()
        self.assertEqual(catalog.validate(data), [])
        full = catalog.cut_lineage(data, "hoeffding_d", "replacement")
        self.assertEqual(full["unrecorded_roles"], [])
        draft = catalog.cut_lineage(data, "hoeffding_d", "draft")
        self.assertEqual(len(draft["artifacts"]), 1)
        self.assertNotIn("script", draft["unrecorded_roles"])
        legacy = catalog.cut_lineage(data, "hoeffding_d", "legacy")
        self.assertEqual(legacy["revision"]["generation_provenance"], "unknown")
        self.assertEqual(legacy["artifacts"], [])
        self.assertEqual(len(legacy["publications"]), 1)
        self.assertEqual(full["publications"], [])

    def test_revision_ids_scoped_to_video(self):
        data = fixture()
        other = copy.deepcopy(data["videos"][0])
        other["video_id"] = "another_video"
        other["publications"] = []
        data["videos"].append(other)
        self.assertEqual(catalog.validate(data), [])
        other["revisions"].append(copy.deepcopy(other["revisions"][0]))
        self.assert_invalid(data, "video another_video: duplicate revision_id 'legacy'")

    def test_duplicate_video_and_artifact_ids(self):
        for field, key in [("videos", "video_id"), ("artifacts", "artifact_id")]:
            with self.subTest(field=field):
                data = fixture()
                data[field].append(copy.deepcopy(data[field][0]))
                self.assert_invalid(data, "duplicate " + key)

    def test_dangling_references(self):
        for target in ["input", "role", "publication", "related"]:
            with self.subTest(target=target):
                data = fixture()
                video = data["videos"][0]
                if target == "input": data["artifacts"][0]["inputs"] = ["missing"]
                if target == "role": video["revisions"][1]["artifact_refs"]["script"] = ["missing"]
                if target == "publication": video["publications"][0]["revision_id"] = "missing"
                if target == "related": video["related_video_ids"] = ["missing"]
                self.assert_invalid(data, "missing")

    def test_wrong_kind_and_cycles(self):
        data = fixture()
        data["videos"][0]["revisions"][1]["artifact_refs"]["take"] = ["script-v1"]
        self.assert_invalid(data, "expected 'take'")
        for parent in ["script-v1", "package-v1"]:
            with self.subTest(parent=parent):
                data = fixture()
                data["artifacts"][0]["inputs"] = [parent]
                self.assert_invalid(data, "cycle")

    def test_unsupported_version_and_scheduler_fields(self):
        data = fixture()
        data["schema_version"] = 2
        self.assert_invalid(data, "/schema_version")
        for where in ["root", "video", "revision", "artifact"]:
            data = fixture()
            target = {"root": data, "video": data["videos"][0],
                      "revision": data["videos"][0]["revisions"][0],
                      "artifact": data["artifacts"][0]}[where]
            target["task_status"] = "done"
            self.assert_invalid(data, "Additional properties")

    def test_unsafe_paths(self):
        for value in ["/tmp/voice", "../voice", "a/../b", "a/..", "a//b", "a/./b", "C:/voice", "a\\b", "file:voice", "a\nb"]:
            with self.subTest(path=value):
                data = fixture()
                data["artifacts"][0]["path"] = value
                self.assert_invalid(data, "repository-relative")

    def test_waivers_keep_measurement_and_authority(self):
        for where in ["artifact", "master", "milestone", "publication"]:
            with self.subTest(where=where):
                data = fixture()
                video = data["videos"][0]
                record = {"artifact": data["artifacts"][0], "master": video["revisions"][0]["master"],
                          "milestone": video["revisions"][0]["milestones"][0],
                          "publication": video["publications"][0]}[where]
                record["evidence_state"] = "waived"
                self.assert_invalid(data, "waiver")
                record["waiver"] = {"measurement": "failed test retained", "owner": "fixture owner",
                                    "disposition": "bounded exception", "scope": "one cut only",
                                    "source_ref": "synthetic-fixture-only"}
                self.assertEqual(catalog.validate(data), [])
                record["evidence_state"] = "pass"
                self.assert_invalid(data, "should not be valid")

    def test_digest_lengths_and_legacy_master_conflict(self):
        data = fixture()
        master = data["videos"][0]["revisions"][0]["master"]
        master["digest"]["algorithm"] = "sha256"
        self.assert_invalid(data, "requires 64")
        data = fixture()
        revision = data["videos"][0]["revisions"][1]
        revision["master"] = {"path": "fixture/master", "digest": {"algorithm": "sha256", "value": "0" * 64}, "evidence_state": "unknown"}
        self.assert_invalid(data, "SHA256 disagree")
        revision["master"]["digest"] = None
        revision["master"]["path"] = "different/master"
        self.assert_invalid(data, "match exactly one")

    def test_duplicate_publication_is_ambiguous(self):
        data = fixture()
        data["videos"][0]["publications"] *= 2
        self.assert_invalid(data, "duplicate publication identity")

    def test_json_roundtrip_and_deterministic_digest(self):
        data = fixture()
        roundtrip = json.loads(json.dumps(data, sort_keys=True, indent=4))
        self.assertEqual(catalog.validate(roundtrip), [])
        self.assertEqual(catalog.digest(data), catalog.digest(roundtrip))
        before = catalog.cut_lineage(data, "hoeffding_d", "replacement")["manifest_sha256"]
        data["artifacts"][0]["sha256"] = "f" * 64
        after = catalog.cut_lineage(data, "hoeffding_d", "replacement")["manifest_sha256"]
        self.assertNotEqual(before, after)
        self.assertEqual(after, catalog.cut_lineage(data, "hoeffding_d", "replacement")["manifest_sha256"])

    def test_unrelated_artifact_does_not_change_cut_digest(self):
        data = fixture()
        before = catalog.cut_lineage(data, "hoeffding_d", "draft")["manifest_sha256"]
        next(a for a in data["artifacts"] if a["kind"] == "music")["sha256"] = "f" * 64
        self.assertEqual(before, catalog.cut_lineage(data, "hoeffding_d", "draft")["manifest_sha256"])

    def test_strict_json_decoder(self):
        with self.assertRaisesRegex(ValueError, "duplicate JSON key"):
            json.loads('{"a":1,"a":2}', object_pairs_hook=catalog._unique_keys)
        with self.assertRaisesRegex(ValueError, "non-JSON"):
            json.loads('{"a":NaN}', parse_constant=catalog._invalid_constant)

    def test_real_cli_and_unknown_lookup(self):
        for args, expected in [(["validate"], 0), (["show", "hoeffding_d"], 0),
                               (["show", "missing_video"], 1),
                               (["show", "hoeffding_d", "--revision", "missing_cut"], 1)]:
            with self.subTest(args=args), contextlib.redirect_stdout(io.StringIO()) as output:
                self.assertEqual(catalog.main(args), expected)
                self.assertIsInstance(json.loads(output.getvalue()), dict)


if __name__ == "__main__":
    unittest.main()
