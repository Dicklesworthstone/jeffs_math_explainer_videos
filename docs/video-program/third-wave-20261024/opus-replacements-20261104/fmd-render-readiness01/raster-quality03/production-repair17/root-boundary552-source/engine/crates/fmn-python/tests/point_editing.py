"""Native point editing across empty paths, field views and animation families."""
import gc
import struct
import unittest

import numpy as np
import manimlib as m


class PointEditingTests(unittest.TestCase):
    def test_empty_point_tables_clear_geometry_without_changing_identity(self):
        for empty in ([], (), np.empty((0,)), np.empty((0, 3))):
            with self.subTest(empty=repr(empty)):
                mob = m.Square(fill_color=m.RED, fill_opacity=.5)
                scene = m.Scene()
                scene.add(mob)
                old = mob.get_points()
                old_values = old.copy()
                self.assertIs(mob.set_points(empty), mob)
                self.assertEqual(mob.get_points().shape, (0, 3))
                self.assertEqual(mob.get_subpaths(), [])
                self.assertIn(mob, scene.mobjects)
                np.testing.assert_array_equal(old, old_values)
                mob.set_points(old_values)
                np.testing.assert_array_equal(mob.get_points(), old_values)
                self.assertAlmostEqual(float(mob.get_fill_opacity()), .5)
                self.assertEqual(mob.get_fill_color(), m.RED)

    def test_malformed_points_do_not_resize_or_change_native_records(self):
        bad = ([1., 2., 3.], [[1., 2.]] * 3, np.empty((0, 2)),
               np.zeros((1, 1, 3)), [[np.nan, 0, 0]], [[np.inf, 0, 0]],
               [[1e300, 0, 0]], [[1j, 0, 0]], [["not a coordinate", 0, 0]])
        for points in bad:
            for operation in ("set_points", "append_points"):
                # VMobject parity checks are deliberately still authoritative.
                mob = m.Mobject().set_points([[0, 0, 0], [1, 0, 0]])
                before, view = mob.data.copy(), mob.get_points()
                with self.subTest(points=repr(points), operation=operation):
                    with self.assertRaises((ValueError, TypeError)):
                        getattr(mob, operation)(points)
                    np.testing.assert_array_equal(mob.data, before)
                    view[0] = [3, 2, 1]
                    np.testing.assert_array_equal(mob.get_points()[0], [3, 2, 1])

    def test_empty_append_is_a_noop_and_retains_live_views(self):
        for mob in (m.Mobject(), m.Square()):
            before = mob.data.copy()
            view = mob.get_points()
            self.assertIs(mob.append_points([]), mob)
            np.testing.assert_array_equal(mob.data, before)
            if len(view):
                view[0] += 1
                np.testing.assert_array_equal(mob.get_points()[0], view[0])

    def test_record_views_share_a_layout_but_not_a_renamed_one(self):
        # fm-5wq.31: one dtype per record layout, not one per `.data` access.
        first, second = m.Square(), m.Square()
        names = first.data.dtype.names
        self.assertEqual(second.data.dtype, first.data.dtype)
        self.assertIsNot(first.data, first.data)
        view = first.data
        view["point"][0] = [7., 8., 9.]
        np.testing.assert_array_equal(first.get_points()[0], [7., 8., 9.])
        self.assertFalse(np.any(second.get_points() == 7.))
        # dtype.names is assignable: renaming one view's fields must not
        # rename the fields of any later view of the same layout.
        view.dtype.names = tuple(f"renamed_{index}" for index in range(len(names)))
        self.assertEqual(first.data.dtype.names, names)
        self.assertEqual(second.data.dtype.names, names)
        self.assertEqual(view.dtype.names[0], "renamed_0")
        np.testing.assert_array_equal(first.data["point"][0], [7., 8., 9.])
        other_layout = m.Sphere(resolution=(4, 4)).data.dtype
        self.assertNotEqual(other_layout, first.data.dtype)

    def test_override_checks_follow_class_edits_between_calls(self):
        # fm-xte3: rotate's authored-mapping check keeps each class's verdict
        # while CPython's version tags of the class and metaclass hold. Every
        # edit below must be seen, and the cached answer must always equal the
        # uncached one.
        from fmn_python import movement
        cells = dict(zip(m.Mobject.rotate.__code__.co_freevars,
                         (cell.cell_contents for cell in m.Mobject.rotate.__closure__)))
        mapping = cells["rotation_mapping"]
        self.assertIsNotNone(mapping.tags)

        def agrees(obj):
            cached = movement._changed(obj, mapping)
            self.assertIs(cached, movement._changed_uncached(obj, mapping, None))
            return cached

        class Local(m.Square):
            pass
        square = Local()
        self.assertIs(agrees(square), False)
        self.assertIn(Local, mapping.verdicts)  # the cache is live, not bypassed
        self.assertIs(agrees(square), False)
        calls = []
        shipped = m.Mobject.apply_points_function

        def authored(self, *args, **kwargs):
            calls.append(self)
            return shipped(self, *args, **kwargs)
        m.Square.apply_points_function = authored  # a base of Local
        try:
            self.assertIs(agrees(square), True)
            square.rotate(.1)
            self.assertEqual(calls, [square])
        finally:
            del m.Square.apply_points_function
        self.assertIs(agrees(square), False)
        Local.get_family = lambda self, recurse=True: m.Square.get_family(self, recurse)
        self.assertIs(agrees(square), True)
        del Local.get_family
        self.assertIs(agrees(square), False)

        class Other(m.Square):
            def apply_points_function(self, *args, **kwargs):
                return shipped(self, *args, **kwargs)
        Local.__bases__ = (Other,)
        self.assertIs(agrees(square), True)
        Local.__bases__ = (m.Square,)
        self.assertIs(agrees(square), False)
        square.get_family = lambda recurse=True: [square]
        self.assertIs(agrees(square), True)
        del square.get_family
        self.assertIs(agrees(square), False)

        class Meta(type):
            pass

        class Metered(m.Square, metaclass=Meta):
            pass
        metered = Metered()
        self.assertIs(agrees(metered), False)
        # A heap metaclass could be swapped by __class__ assignment without
        # changing any tag, so such classes are decided every call.
        self.assertNotIn(Metered, mapping.verdicts)
        before = mapping.tags(Metered)
        Meta.marker = 1
        self.assertNotEqual(mapping.tags(Metered)[1], before[1])
        self.assertIs(agrees(metered), False)

    def test_first_append_uses_retained_style_defaults(self):
        mob = m.Square(fill_color=m.RED, fill_opacity=.25, stroke_width=7)
        saved = mob.data[0].copy()
        mob.clear_points()
        m.Mobject.append_points(mob, [[0, 0, 0], [1, 0, 0], [2, 0, 0]])
        for field in ("fill_rgba", "stroke_rgba", "stroke_width"):
            np.testing.assert_array_equal(mob.data[field], np.repeat(saved[field][None], 3, axis=0))

    def test_append_inherits_last_record_and_preserves_detached_old_view(self):
        mob = m.Square()
        mob.data["stroke_width"][-1] = 9
        old = mob.get_points()
        expected = old.copy()
        tail = old[:2][::-1]
        self.assertIs(mob.append_points(tail), mob)
        np.testing.assert_array_equal(mob.get_points(), np.vstack([expected, expected[:2][::-1]]))
        np.testing.assert_array_equal(old, expected)
        np.testing.assert_array_equal(mob.data["stroke_width"][-2:], [[9], [9]])
        old[:] = -42
        self.assertFalse(np.any(mob.get_points() == -42))

    def test_reversed_and_strided_aliases_survive_set_and_resize(self):
        for step in (-1, 2):
            mob = m.Square()
            expected = mob.get_points()[::step].copy()
            mob.set_points(mob.get_points()[::step])
            np.testing.assert_array_equal(mob.get_points(), expected)

    def test_authored_resize_hook_cannot_change_the_frozen_input(self):
        class Authored(m.Mobject):
            def resize_points(self, length, **kwargs):
                if hasattr(self, "source"):
                    self.source[:] = -99
                return super().resize_points(length, **kwargs)
        mob = Authored()
        values = np.array([[1., 2., 3.], [4., 5., 6.]])
        expected = values.copy()
        mob.source = values
        mob.set_points(values)
        np.testing.assert_array_equal(mob.get_points(), expected)
        np.testing.assert_array_equal(values, np.full((2, 3), -99))

    def test_vmobject_keeps_count_validation_and_parent_bounds(self):
        mob = m.Square()
        parent = m.VGroup(mob)
        old = mob.get_points().copy()
        with self.assertRaises(AssertionError):
            mob.set_points(np.zeros((2, 3)))
        np.testing.assert_array_equal(mob.get_points(), old)
        mob.set_points([[2, 0, 0], [3, 0, 0], [4, 0, 0]])
        np.testing.assert_allclose(parent.get_center(), [3, 0, 0])
        mob.set_points([])
        self.assertEqual(parent.family_members_with_points(), [])

    def test_grouped_authored_transform_aligns_empty_container_paths(self):
        class Authored(m.Transform):
            def interpolate_submobject(self, current, start, target, alpha):
                return super().interpolate_submobject(current, start, target, alpha)
        scene = m.Scene()
        source = m.VGroup(m.Square(), m.VGroup(m.Circle().shift(m.RIGHT)))
        target = source.copy().shift(2 * m.RIGHT)
        before = source.get_center().copy()
        scene.add(source)
        scene.play(Authored(source, target, rate_func=m.linear), run_time=.125)
        np.testing.assert_allclose(source.get_center(), before + 2 * m.RIGHT, atol=1e-6)
        self.assertEqual(source.get_points().shape, (0, 3))
        self.assertFalse(source._is_updating_suspended())

    def test_array_point_rows_convert_bit_for_bit_like_the_per_point_path(self):
        # fm-5wq.31: float32/float64 (N, 3) arrays skip the per-point _vec3 loop.
        def bits(rows):
            return [struct.pack("<3d", *row) for row in rows]
        wide = np.array([[0., -0., 5e-324, 1., 2., 3.], [1.5, -2.25, 3e300, 4., 5., 6.],
                         [np.nan, np.inf, -np.inf, 7., 8., 9.], [.1, 2. ** 53, -1e-310, 0., 0., 0.]])
        rows = wide[:, :3].copy()
        for points in (rows, rows[::-1], wide[:, ::2], wide[:, :3], np.asfortranarray(rows),
                       rows.astype(np.float32), rows.astype(">f8"), wide, rows.tolist(),
                       [tuple(row) for row in rows], list(rows), np.empty((0, 3)),
                       m.Circle().get_points(), m.Circle().get_points()[::-2]):
            with self.subTest(points=repr(points)):
                self.assertEqual(bits(m._vec3_rows(points)), bits([m._vec3(p) for p in points]))
                handed = m._point_array(points)
                if isinstance(handed, np.ndarray):
                    self.assertEqual((handed.dtype, handed.ndim, handed.flags["C_CONTIGUOUS"]),
                                     (np.dtype(np.float64), 2, True))
                    handed = handed.tolist()
                self.assertEqual(bits(handed), bits([m._vec3(p) for p in points]))
        for short in (np.zeros((2, 2)), [[0., 0.]]):
            with self.assertRaises(IndexError):
                m._vec3_rows(short)
        corners = np.array([[0., -0., 0.], [1., .1, -0.], [2.5, 1e-310, 3.]])
        for method in ("set_points_as_corners", "add_points_as_corners"):
            for other in (corners.tolist(), [tuple(row) for row in corners]):
                with self.subTest(method=method, other=type(other[0]).__name__):
                    from_array, from_rows = (m.VMobject().set_points_as_corners([[9., 9., 0.], [9., 8., 0.]])
                                             for _ in range(2))
                    getattr(from_array, method)(corners)
                    getattr(from_rows, method)(other)
                    self.assertEqual(from_array.get_points().tobytes(), from_rows.get_points().tobytes())
        path, other = m.VMobject().set_points_as_corners(corners), m.VMobject().set_points_as_corners(corners)
        path.add_subpath(corners[:, ::-1][:3] + 4.)
        other.add_subpath((corners[:, ::-1][:3] + 4.).tolist())
        self.assertEqual(path.get_points().tobytes(), other.get_points().tobytes())
        refusals = []
        for bad in (np.array([[np.nan, 0., 0.], [1., 1., 0.]]), [[np.nan, 0., 0.], [1., 1., 0.]]):
            try:
                m.VMobject().set_points_as_corners(bad)
            except Exception as error:
                refusals.append((type(error), str(error)))
            else:
                refusals.append(None)
        self.assertEqual(refusals[0], refusals[1])
        # The appended rows reach authored set_points/append_points as a
        # writable float64 (N, 3) array (the Reference passes array slices),
        # holding exactly the rows the engine appended.
        received = []

        class Recording(m.VMobject):
            def set_points(self, points):
                received.append(("set", points))
                return super().set_points(points)

            def append_points(self, points):
                received.append(("append", points))
                return super().append_points(points)

        path = Recording()
        path.add_subpath(corners)
        path.add_subpath(corners[:, ::-1] + 4.)
        path.add_points_as_corners(corners + 8.)
        self.assertEqual([kind for kind, _ in received], ["set", "append", "append"])
        tail = 0
        for kind, points in received:
            self.assertIsInstance(points, np.ndarray)
            self.assertEqual((points.dtype, points.shape[1], points.flags["WRITEABLE"]),
                             (np.dtype(np.float64), 3, True))
            tail += len(points)
        self.assertEqual(tail, path.get_num_points())
        np.testing.assert_array_equal(np.vstack([points for _, points in received]).astype(np.float32),
                                      path.get_points())


_suite = unittest.defaultTestLoader.loadTestsFromTestCase(PointEditingTests)
assert _suite.countTestCases() == 12, "native point-editing inventory changed"
_result = unittest.TextTestRunner(verbosity=2).run(_suite)
gc.collect()
if not _result.wasSuccessful():
    raise AssertionError("native point-editing acceptance failed")
