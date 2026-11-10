"""Actual native Transform material admission and endpoint identity."""
from __future__ import annotations

import copy
import unittest
import numpy as np
import manimlib as m


def pixels(color, shape=(2, 3)):
    return np.full((*shape, 4), color, dtype=np.uint8)


def bare_like(mob):
    """A plain Mobject with `mob`'s record schema, not its class. The
    Reference's Mobject takes no data_dtype keyword; a subclass declares it
    (fm-5wq.20)."""
    class Bare(m.Mobject):
        data_dtype = mob.data_dtype
    return Bare().become(mob)


RED, BLUE, GREEN = (255, 0, 0, 255), (0, 0, 255, 255), (0, 255, 0, 255)


class TransformMaterialKernelTests(unittest.TestCase):
    def setUp(self):
        self.a, self.b = m.ImageMobject(pixels(RED)), m.ImageMobject(pixels(BLUE))
        self.Plan = m._RasterTransition

    def test_plain_geometry_needs_no_material_plan(self):
        a, b = m.Square(), m.Circle()
        self.assertEqual(self.Plan.required_texels(a, b), 0)
        self.assertIsNone(self.Plan.between(a, b))

    def test_plans_use_native_resources_not_class_or_filename_metadata(self):
        self.a.image_path = self.b.image_path = 'not a readable path'
        start, end = bare_like(self.a), bare_like(self.b)
        self.assertEqual(self.Plan.required_texels(start, end), 12)
        plan = self.Plan.between(start, end)
        self.assertTrue(plan.matches(start, end))
        self.assertEqual(plan.sample(.5).pixels(), pixels((188, 0, 188, 255)).tobytes())
        self.assertTrue(plan.matches(self.a, self.b))
        plan.apply(start, 1)
        self.assertTrue(m._raster_images_equal(start, end))
        self.assertFalse(plan.matches(start, end))

    def test_equal_resources_are_exact_and_do_not_require_decoded_storage(self):
        self.b.set_pixel_array(pixels(RED))
        self.assertEqual(self.Plan.required_texels(self.a, self.b), 0)
        plan = self.Plan.between(self.a, self.b)
        for alpha in (0, .3, 1):
            self.assertEqual(plan.sample(alpha).pixels(), pixels(RED).tobytes())
        self.assertTrue(copy.copy(plan).matches(self.a, self.b))
        self.assertFalse(plan.has_dark)

    def test_dark_only_edits_and_absent_sides_are_detected(self):
        a = m.TexturedSurface(m.Surface(resolution=(2, 2)), pixels(RED), pixels(GREEN))
        b = m.TexturedSurface(m.Surface(resolution=(3, 4)), pixels(RED), pixels(BLUE))
        plan = self.Plan.between(a, b)
        self.assertEqual(self.Plan.required_texels(a, b), 24)
        self.assertTrue(plan.start_has_dark and plan.end_has_dark)
        b.set_pixel_array(pixels(RED))
        self.assertFalse(plan.matches(a, b))
        updated = self.Plan.between(a, b)
        self.assertTrue(updated.start_has_dark)
        self.assertFalse(updated.end_has_dark)
        updated.apply(a, 1)
        self.assertTrue(m._raster_images_equal(a, b))
        constant = self.Plan.between(a, b)
        self.assertEqual(self.Plan.required_texels(a, b), 0)
        self.assertFalse(constant.has_dark)

    def test_incompatible_primitives_refuse_without_mutating_either_endpoint(self):
        surface = m.TexturedSurface(m.Surface(resolution=(2, 2)), pixels(BLUE))
        for other in (m.Square(), surface):
            before = self.a.data.copy(), other.data.copy()
            for operation in (self.Plan.required_texels, self.Plan.between):
                with self.assertRaisesRegex(ValueError, 'compatible'):
                    operation(self.a, other)
                np.testing.assert_array_equal(self.a.data, before[0])
                np.testing.assert_array_equal(other.data, before[1])
        np.testing.assert_array_equal(self.a.get_pixel_array(), pixels(RED))

    def test_admission_refuses_crossed_output_dimensions_before_decode(self):
        a = m.ImageMobject(pixels(RED, (1, 4097)))
        b = m.ImageMobject(pixels(BLUE, (4097, 1)))
        for operation in (self.Plan.required_texels, self.Plan.between):
            with self.assertRaisesRegex(ValueError, 'output.*budget'):
                operation(a, b)
        np.testing.assert_array_equal(a.get_pixel_array(), pixels(RED, (1, 4097)))

    def test_plan_refresh_observes_pixel_dimensions_and_live_resource_replacement(self):
        plan = self.Plan.between(self.a, self.b)
        old = plan.sample(.5).pixels()
        self.b.set_pixel_array(pixels(GREEN, (3, 2)))
        self.assertFalse(plan.matches(self.a, self.b))
        self.assertEqual(plan.sample(.5).pixels(), old)
        updated = self.Plan.between(self.a, self.b)
        self.assertEqual(updated.sample(.5).size, (3, 3))
        self.assertEqual(updated.sample(.5).pixels(), pixels((188, 188, 0, 255), (3, 3)).tobytes())


result = unittest.TextTestRunner(verbosity=2).run(
    unittest.defaultTestLoader.loadTestsFromTestCase(TransformMaterialKernelTests))
if not result.wasSuccessful():
    raise AssertionError('native Transform material admission failed')
