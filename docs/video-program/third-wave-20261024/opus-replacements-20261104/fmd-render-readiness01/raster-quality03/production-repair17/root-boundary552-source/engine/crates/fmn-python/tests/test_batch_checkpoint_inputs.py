"""Checkpoint execution uses admitted JSON values, not mutable caller aliases."""
from __future__ import annotations

import copy
import json
import unittest
from unittest.mock import patch

import test_batch_checkpoint_helpers as fixtures


class CheckpointInputTests(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.CheckpointTests()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.palette = {"colors": ["red"]}
        self.fixture.jobs = [fixtures.batch.RenderJob(name, scene, {"palette": self.palette})
                             for name, scene in (("first", fixtures.First), ("second", fixtures.Second))]
        self.inputs = []

    def capture(self, scene, destination, **kwargs):
        self.inputs.append(copy.deepcopy(kwargs["scene_kwargs"]))
        return self.fixture.fake_render(scene, destination, **kwargs)

    def test_observer_mutation_cannot_change_a_later_admitted_job(self):
        def observer(outcome):
            if outcome.name == "first":
                self.palette["colors"][0] = "blue"
        with patch.object(fixtures.batch, "render_scene", self.capture):
            self.fixture.run_batch(on_result=observer)
        self.assertEqual(self.inputs, [{"palette": {"colors": ["red"]}}] * 2)
        self.assertEqual([job["scene_kwargs"] for job in self.fixture.document()["plan"]["jobs"]],
                         self.inputs)
        self.assertEqual(self.palette["colors"], ["blue"])

    def test_constructor_mutation_is_local_to_one_job(self):
        def render(scene, destination, **kwargs):
            self.inputs.append(copy.deepcopy(kwargs["scene_kwargs"]))
            kwargs["scene_kwargs"]["palette"]["colors"].append(scene.__name__)
            return self.fixture.fake_render(scene, destination, **kwargs)
        with patch.object(fixtures.batch, "render_scene", render):
            self.fixture.run_batch()
        self.assertEqual(self.inputs, [{"palette": {"colors": ["red"]}}] * 2)
        self.assertEqual(self.palette, {"colors": ["red"]})

    def test_resumed_observer_cannot_change_unfinished_job_inputs(self):
        self.fixture.failures[fixtures.Second] = RuntimeError("unfinished")
        with self.assertRaises(fixtures.batch.BatchRenderError):
            self.fixture.run_batch()
        self.fixture.failures.clear()
        def observer(outcome):
            if outcome.name == "first":
                self.palette["colors"].clear()
        with patch.object(fixtures.batch, "render_scene", self.capture):
            self.fixture.run_batch(resume=True, on_result=observer)
        self.assertEqual(self.inputs, [{"palette": {"colors": ["red"]}}])

    def test_noncheckpoint_execution_keeps_python_reference_semantics(self):
        def render(scene, destination, **kwargs):
            self.assertIs(kwargs["scene_kwargs"]["palette"], self.palette)
            return self.fixture.fake_render(scene, destination, **kwargs)
        with patch.object(fixtures.batch, "render_scene", render):
            self.fixture.run_batch(checkpoint=None, resume_key=None)

    def test_non_json_values_refuse_before_execution_or_journal_creation(self):
        class CustomInt(int):
            pass
        class CustomList(list):
            pass
        cyclic = []
        cyclic.append(cyclic)
        for index, value in enumerate(((1, 2), {1: "integer-key"}, {None: "null-key"},
                                       CustomInt(1), CustomList([1]), object(), cyclic)):
            with self.subTest(value_type=type(value).__name__):
                self.fixture.journal = self.fixture.root / f"progress-{index}.json"
                self.fixture.output = self.fixture.root / f"output-{index}"
                self.fixture.calls.clear()
                self.fixture.jobs = [fixtures.batch.RenderJob("first", fixtures.First, {"value": value})]
                with self.assertRaisesRegex((TypeError, ValueError), "checkpoint.*JSON"):
                    self.fixture.run_batch()
                self.assertEqual(self.fixture.calls, [])
                self.assertFalse(self.fixture.journal.exists())
                self.assertFalse(self.fixture.output.exists())

    def test_old_plan_without_frozen_input_protocol_is_not_reused(self):
        self.fixture.run_batch()
        document = self.fixture.document()
        document["plan"].pop("constructor_inputs", None)
        self.fixture.journal.write_text(json.dumps(document))
        before = self.fixture.journal.read_bytes()
        self.fixture.calls.clear()
        with self.assertRaisesRegex(ValueError, "plan"):
            self.fixture.run_batch(resume=True)
        self.assertEqual(self.fixture.calls, [])
        self.assertEqual(self.fixture.journal.read_bytes(), before)

    def test_shared_json_children_become_independent_values(self):
        colors = ["red"]
        self.fixture.jobs = [fixtures.batch.RenderJob("first", fixtures.First,
                                                    {"left": colors, "right": colors})]
        def render(scene, destination, **kwargs):
            inputs = kwargs["scene_kwargs"]
            self.assertEqual(inputs["left"], inputs["right"])
            self.assertIsNot(inputs["left"], inputs["right"])
            inputs["left"].append("blue")
            self.assertEqual(inputs["right"], ["red"])
            return self.fixture.fake_render(scene, destination, **kwargs)
        with patch.object(fixtures.batch, "render_scene", render):
            self.fixture.run_batch()
        self.assertEqual(colors, ["red"])


if __name__ == "__main__":
    unittest.main()
