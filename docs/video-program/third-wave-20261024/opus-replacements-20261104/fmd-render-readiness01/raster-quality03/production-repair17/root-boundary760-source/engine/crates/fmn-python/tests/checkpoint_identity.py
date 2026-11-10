"""Native PNG publications cannot be resumed under a different scene/argument identity."""
from __future__ import annotations

import hashlib
import json
import math
from pathlib import Path
import tempfile
import unittest

import manimlib as m
from fmn_python import RenderJob, render_scenes


CALLS = []


class ParameterScene(m.Scene):
    def __init__(self, value=True, **kwargs):
        CALLS.append((type(self).__module__, type(value), repr(value)))
        super().__init__(**kwargs)
        self.value = value

    def construct(self):
        # Equal under Python == does not imply equal scene semantics.
        value = self.value
        red = (type(value) is bool or type(value) is int
               or (type(value) is float and math.copysign(1.0, value) > 0))
        self.add(m.Square(fill_opacity=1, stroke_width=0,
                          fill_color=m.RED if red else m.BLUE))


class PaletteScene(m.Scene):
    def __init__(self, palette, mutate=False, **kwargs):
        CALLS.append(palette["colors"][0])
        super().__init__(**kwargs)
        self.color = m.RED if palette["colors"][0] == "red" else m.BLUE
        if mutate:
            palette["colors"][0] = "blue"

    def construct(self):
        self.add(m.Square(fill_opacity=1, stroke_width=0, fill_color=self.color))


def inventory(directory):
    return {str(path.relative_to(directory)): (path.stat().st_mtime_ns,
            hashlib.sha256(path.read_bytes()).hexdigest())
            for path in directory.rglob("*") if path.is_file()}


class NativeCheckpointIdentityTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="fmn-checkpoint-identity-"))
        self.options = dict(format="png", resolution=(96, 54), fps=8, threads=1,
                            checkpoint=self.root / "progress.json", resume_key="unchanged-project")
        CALLS.clear()

    def render(self, cls=ParameterScene, value=True, *, resume=False, directory=None):
        return render_scenes([RenderJob("image", cls, {"value": value})],
                             self.root / "output" if directory is None else directory,
                             resume=resume, **self.options)

    def assert_refused(self, cls, value):
        before = inventory(self.root)
        CALLS.clear()
        with self.assertRaisesRegex(ValueError, "plan"):
            self.render(cls, value, resume=True)
        self.assertEqual(CALLS, [])
        self.assertEqual(inventory(self.root), before)

    def test_numeric_type_changes_refuse_before_constructor_or_publication(self):
        self.render(value=True)
        self.assert_refused(ParameterScene, 1)
        self.assert_refused(ParameterScene, 1.0)

    def test_negative_zero_does_not_reuse_positive_zero_pixels(self):
        first = self.render(value=0.0)
        self.assert_refused(ParameterScene, -0.0)
        other = render_scenes([RenderJob("image", ParameterScene, {"value": -0.0})],
                              self.root / "negative", format="png", resolution=(96, 54),
                              fps=8, threads=1)
        self.assertNotEqual(first.outcomes[0].result.digest, other.outcomes[0].result.digest)

    def test_equal_class_names_do_not_authorize_other_module_outputs(self):
        first = type("Demo", (ParameterScene,), {"__module__": "lessons.first"})
        second = type("Demo", (ParameterScene,), {"__module__": "lessons.second"})
        self.render(first)
        self.assert_refused(second, True)

    def test_identical_native_receipt_is_reused_without_side_effects(self):
        first = self.render(value=-0.0)
        before = inventory(self.root / "output")
        CALLS.clear()
        resumed = self.render(value=-0.0, resume=True)
        self.assertTrue(resumed.ok)
        self.assertEqual(first.as_dict(), resumed.as_dict())
        self.assertEqual(CALLS, [])
        self.assertEqual(inventory(self.root / "output"), before)

    def test_observer_mutation_cannot_change_later_native_pixels(self):
        palette = {"colors": ["red"]}
        jobs = [RenderJob(name, PaletteScene, {"palette": palette}) for name in ("first", "second")]
        def observer(outcome):
            if outcome.name == "first":
                palette["colors"][0] = "blue"
        result = render_scenes(jobs, self.root / "output", on_result=observer, **self.options)
        self.assertEqual(CALLS, ["red", "red"])
        self.assertEqual(result.outcomes[0].result.digest, result.outcomes[1].result.digest)
        document = json.loads(self.options["checkpoint"].read_text())
        self.assertTrue(all(job["scene_kwargs"] == {"palette": {"colors": ["red"]}}
                            for job in document["plan"]["jobs"]))
        palette["colors"][0] = "red"
        CALLS.clear()
        resumed = render_scenes(jobs, self.root / "output", resume=True, **self.options)
        self.assertEqual(resumed.as_dict(), result.as_dict())
        self.assertEqual(CALLS, [])

    def test_native_constructor_mutation_does_not_escape_its_job(self):
        palette = {"colors": ["red"]}
        jobs = [RenderJob(name, PaletteScene, {"palette": palette, "mutate": True})
                for name in ("first", "second")]
        result = render_scenes(jobs, self.root / "output", **self.options)
        self.assertEqual(CALLS, ["red", "red"])
        self.assertEqual(result.outcomes[0].result.digest, result.outcomes[1].result.digest)
        self.assertEqual(palette, {"colors": ["red"]})

    def test_non_json_constructor_values_fail_before_native_execution(self):
        jobs = [RenderJob("first", PaletteScene, {"palette": {"colors": ("red",)}})]
        with self.assertRaisesRegex(TypeError, "checkpoint.*JSON"):
            render_scenes(jobs, self.root / "output", **self.options)
        self.assertEqual(CALLS, [])
        self.assertFalse((self.root / "output").exists())
        self.assertFalse(self.options["checkpoint"].exists())


if __name__ in ("__main__", "<run_path>"):
    result = unittest.TextTestRunner(verbosity=2).run(
        unittest.defaultTestLoader.loadTestsFromTestCase(NativeCheckpointIdentityTests))
    if not result.wasSuccessful():
        raise AssertionError("native checkpoint identity regressions failed")
