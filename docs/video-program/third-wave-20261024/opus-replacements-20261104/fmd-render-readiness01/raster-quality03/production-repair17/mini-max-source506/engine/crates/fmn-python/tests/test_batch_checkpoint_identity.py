"""Checkpoint plan admission; renderer behavior is covered by the native suite."""
from __future__ import annotations

import json
import unittest

import test_batch_checkpoint_helpers as fixtures


class CheckpointIdentityTests(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.CheckpointTests()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)

    def assert_refused_without_writes(self, pattern="plan"):
        fixture = self.fixture
        before = fixture.journal.read_bytes()
        fixture.calls.clear()
        with self.assertRaisesRegex(ValueError, pattern):
            fixture.run_batch(resume=True)
        self.assertEqual(fixture.calls, [])
        self.assertEqual(fixture.journal.read_bytes(), before)

    def test_same_class_name_from_another_module_is_not_the_same_scene(self):
        first = type("Demo", (fixtures.Scene,), {"__module__": "lesson.first"})
        second = type("Demo", (fixtures.Scene,), {"__module__": "lesson.second"})
        self.fixture.jobs = {"first": first}
        self.fixture.run_batch()
        self.fixture.jobs = {"first": second}
        self.assert_refused_without_writes()

    def test_module_and_qualname_are_separate_identity_fields(self):
        first = type("Demo", (fixtures.Scene,), {"__module__": "lesson.nested"})
        second = type("Demo", (fixtures.Scene,), {
            "__module__": "lesson", "__qualname__": "nested.Demo",
        })
        self.fixture.jobs = {"first": first}
        self.fixture.run_batch()
        self.fixture.jobs = {"first": second}
        self.assert_refused_without_writes()

    def check_changed_parameter(self, first, second):
        self.fixture.jobs = [fixtures.batch.RenderJob("first", fixtures.First, {"value": first})]
        self.fixture.run_batch()
        self.fixture.jobs = [fixtures.batch.RenderJob("first", fixtures.First, {"value": second})]
        self.assert_refused_without_writes()

    def test_bool_is_not_an_integer_parameter(self):
        self.check_changed_parameter(True, 1)

    def test_integer_is_not_a_float_parameter(self):
        self.check_changed_parameter(1, 1.0)

    def test_signed_zero_is_preserved(self):
        self.check_changed_parameter(0.0, -0.0)

    def test_nested_numeric_types_are_preserved(self):
        self.check_changed_parameter({"levels": [False, {"weight": 1}]},
                                     {"levels": [0, {"weight": 1.0}]})

    def test_mapping_order_does_not_invalidate_identical_inputs(self):
        self.fixture.jobs = [fixtures.batch.RenderJob("first", fixtures.First,
                            {"style": {"red": 1, "blue": 0.5}})]
        result = self.fixture.run_batch()
        self.fixture.calls.clear()
        self.fixture.jobs = [fixtures.batch.RenderJob("first", fixtures.First,
                            {"style": {"blue": 0.5, "red": 1}})]
        self.assertEqual(self.fixture.run_batch(resume=True).as_dict(), result.as_dict())
        self.assertEqual(self.fixture.calls, [])

    def test_moduleless_class_is_rejected_before_execution(self):
        self.fixture.jobs = {"first": type("Demo", (fixtures.Scene,), {"__module__": None})}
        with self.assertRaisesRegex(ValueError, "scene identity"):
            self.fixture.run_batch()
        self.assertEqual(self.fixture.calls, [])
        self.assertFalse(self.fixture.journal.exists())
        self.assertFalse(self.fixture.output.exists())

    def test_version_one_cannot_assert_missing_scene_identity(self):
        self.fixture.run_batch()
        document = self.fixture.document()
        document["version"] = 1
        for job in document["plan"]["jobs"]:
            job["scene"] = "First"
        self.fixture.journal.write_text(json.dumps(document))
        self.assert_refused_without_writes("schema/version")

    def test_schema_version_requires_an_integer(self):
        self.fixture.run_batch()
        document = self.fixture.document()
        document["version"] = float(document["version"])
        self.fixture.journal.write_text(json.dumps(document))
        self.assert_refused_without_writes("schema/version")


if __name__ == "__main__":
    unittest.main()
