#!/usr/bin/env python3
"""uv run --no-project --with jsonschema python3 -I tools/test_video_catalog.py"""
import contextlib
import copy
import importlib.util
import io
import json
import unittest
from pathlib import Path
from unittest import mock

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

    def test_board_tracker_reads_disable_sync_and_report_degraded_status(self):
        data = {"videos": []}
        responses = [
            {"workspace_health": "degraded", "coverage_drift": True, "dirty_count": 3},
            {"issues": []}, {"issues": []},
        ]
        calls = []
        def run(argv, **kwargs):
            calls.append(argv)
            return type("Result", (), {"returncode": 0, "stdout": json.dumps(responses.pop(0)), "stderr": ""})()
        with mock.patch.object(catalog.subprocess, "run", side_effect=run):
            _, _, _, warnings = catalog.tracker_snapshot(data)
        self.assertTrue(all("--no-auto-import" in call and "--no-auto-flush" in call for call in calls))
        self.assertIn("sync", calls[0])
        self.assertTrue(any("degraded" in warning and "dirty count: 3" in warning for warning in warnings))

    def test_board_retains_public_and_private_cuts_and_blockers(self):
        data = fixture()
        video = data["videos"][0]
        video["work_epic"] = "epic"
        video["publications"][0]["visibility"] = "public"
        private = copy.deepcopy(video["publications"][0])
        private.update(external_id="private-cut", revision_id="replacement", visibility="private")
        video["publications"].append(private)
        video["revisions"][0]["evidence_refs"] = ["epic", "mx-missing", "absent/file.json"]
        parked = copy.deepcopy(video)
        parked.update(video_id="parked-idea", work_epic="missing-epic", revisions=[], publications=[],
                      decisions=[{"decision": "parked"}])
        data["videos"].append(parked)
        issues = [{"id": "epic", "title": "Epic", "status": "in_progress"},
                  {"id": "draft-task", "title": "Finish draft", "status": "open"}]
        rows = catalog.board_rows(data, issues, [{"id": "draft-task", "blocked_by": ["review"]}],
                                  {"epic": ["draft-task"]}, root=Path("/nonexistent-board-fixture"))
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[1]["editorial_decision"]["decision"], "parked")
        self.assertEqual(rows[1]["tracker_anchor_state"], "unknown")
        cuts = {cut["revision_id"]: cut for cut in rows[0]["revisions"]}
        self.assertEqual(cuts["legacy"]["publications"][0]["visibility"], "public")
        self.assertEqual(cuts["replacement"]["publications"][0]["visibility"], "private")
        self.assertEqual(cuts["legacy"]["generation_provenance"], "unknown")
        self.assertFalse(cuts["legacy"]["master_path_exists"])
        self.assertEqual([ref["state"] for ref in cuts["legacy"]["evidence_refs"]],
                         ["tracker_issue_exists", "tracker_issue_missing", "local_missing"])
        self.assertIn("master", cuts["draft"]["unrecorded_artifact_roles"])
        self.assertEqual(next(w for w in rows[0]["work"] if w["id"] == "draft-task")["blocked_by"], ["review"])

    def test_board_covers_audited_original_ids_without_duplicate_video_rows(self):
        data = catalog.read_json(catalog.CATALOG)
        rows = catalog.board_rows(data, [], [], {})
        self.assertEqual(len(rows), len({row["video_id"] for row in rows}))
        by_id = {row["video_id"]: row for row in rows}
        audit = catalog.read_json(catalog.ROOT / "docs/video-program/publication-audit-20261015.json")
        self.assertEqual(len(audit), 11)
        for original in audit:
            cuts = by_id[original["slug"]]["revisions"]
            ids = [pub["external_id"] for cut in cuts for pub in cut["publications"]]
            self.assertEqual(ids.count(original["id"]), 1)

    def test_text_board_surfaces_unchecked_references_in_every_record(self):
        data = fixture()
        video = data["videos"][0]
        video["source_refs"] = ["https://example.com/source"]
        video["revisions"][0]["master"]["evidence_state"] = "stale"
        video["revisions"][0]["evidence_refs"] = ["docs/video-program/catalog.json#not-checked"]
        video["revisions"][0]["milestones"] = [{"stage": "uploaded", "recorded_as_of": "2026-01-01",
                                                   "evidence_state": "stale", "evidence_refs": ["missing/milestone.json"]}]
        video["publications"][0]["evidence_state"] = "stale"
        video["publications"][0]["evidence_refs"] = ["https://example.com/publication"]
        next(item for item in data["artifacts"] if item["kind"] == "script")["evidence_refs"] = ["https://example.com/artifact"]
        rows = catalog.board_rows(data, [], [], {})
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            catalog.print_board({"catalog_updated_on": data["updated_on"], "videos": rows,
                                 "tracker_warning": None, "claim_scope": "recorded only",
                                 "unmapped_lane_work": [], "lane_scope": "fixture"})
        text = output.getvalue()
        self.assertIn("master evidence: stale", text)
        self.assertIn("| evidence: stale", text)
        self.assertIn("external_not_checked: https://example.com/source", text)
        self.assertIn("local_exists_anchor_unchecked: docs/video-program/catalog.json#not-checked", text)
        self.assertIn("external_not_checked: https://example.com/publication", text)
        self.assertIn("local_missing: missing/milestone.json", text)
        self.assertIn("external_not_checked: https://example.com/artifact", text)

    def test_board_cli_filters_by_recorded_blocker_and_lane(self):
        issues = [{"id": "mx-video-program-program-87do.12", "title": "German tank", "status": "in_progress"},
                  {"id": "pending", "title": "Pending", "status": "open", "labels": ["lane:narration"]},
                  {"id": "shared", "title": "Shared work", "status": "open", "labels": ["lane:narration"]},
                  {"id": "other", "title": "Unblocked shared work", "status": "open", "labels": ["lane:narration"]}]
        state = (issues, [{"id": "pending", "blocked_by": ["review-1"]},
                          {"id": "shared", "blocked_by": ["review-1"]}],
                 {"mx-video-program-program-87do.12": ["pending"]}, ["import disabled", "sync degraded"])
        output = io.StringIO()
        with mock.patch.object(catalog, "tracker_snapshot", return_value=state), contextlib.redirect_stdout(output):
            self.assertEqual(catalog.main(["board", "--decision", "published", "--lane", "narration",
                                           "--blocker", "review-1"]), 0)
        result = json.loads(output.getvalue())
        self.assertEqual(result["tracker_warning"], "import disabled | sync degraded")
        rows = result["videos"]
        self.assertEqual([row["video_id"] for row in rows], ["german_tank"])
        self.assertIn("review-1", next(w for w in rows[0]["work"] if w["id"] == "pending")["blocked_by"])
        self.assertEqual([item["id"] for item in result["unmapped_lane_work"]], ["shared"])
        output = io.StringIO()
        with mock.patch.object(catalog, "tracker_snapshot", return_value=state), contextlib.redirect_stdout(output):
            self.assertEqual(catalog.main(["board", "--format", "text", "--blocker", "review-1"]), 0)
        self.assertIn("german_tank", output.getvalue())
        self.assertIn("unlisted as of", output.getvalue())
        self.assertIn("public as of", output.getvalue())
        self.assertIn("review-1", output.getvalue())

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
