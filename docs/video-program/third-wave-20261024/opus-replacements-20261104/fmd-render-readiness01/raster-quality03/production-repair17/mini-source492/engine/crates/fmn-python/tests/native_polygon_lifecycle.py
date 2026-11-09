"""Polygon/line constructor hooks and their actual rendered scene output."""
import pathlib
import struct
import tempfile
import unittest
import zlib

import numpy as np
import manimlib as m


CASES = (
    (m.Polygon, ([0, 0, 0], [2, 0, 0], [0, 2, 0]), {}),
    (m.Polyline, ([0, 0, 0], [1, 1, 0], [2, 0, 0]), {}),
    (m.RegularPolygon, (), {"n": 5, "radius": 2}),
    (m.Rectangle, (), {"width": 3, "height": 2}),
    (m.RoundedRectangle, (), {"width": 3, "height": 2, "corner_radius": 0.2}),
    (m.Square, (), {"side_length": 2}),
    (m.Triangle, (), {}),
    (m.ScreenRectangle, (), {"height": 2}),
    (m.FullScreenRectangle, (), {}),
    (m.FullScreenFadeRectangle, (), {}),
    (m.CubicBezier, ([-1, 0, 0], [-1, 2, 0], [1, 2, 0], [1, 0, 0]), {}),
    (m.Line, ([-1, 0, 0], [1, 0, 0]), {"buff": 0.1}),
)


def png_pixels(path):
    """Decode the native non-interlaced RGBA8 output without a Pillow dependency."""
    payload = pathlib.Path(path).read_bytes()
    if payload[:8] != b"\x89PNG\r\n\x1a\n":
        raise AssertionError("not a PNG")
    offset, compressed = 8, bytearray()
    while offset < len(payload):
        size = int.from_bytes(payload[offset:offset + 4], "big")
        kind = payload[offset + 4:offset + 8]
        body = payload[offset + 8:offset + 8 + size]
        if kind == b"IHDR":
            width, height, depth, color, compression, filtering, interlace = struct.unpack(">IIBBBBB", body)
            if (depth, color, compression, filtering, interlace) != (8, 6, 0, 0, 0):
                raise AssertionError("not the native RGBA8 PNG encoding")
        elif kind == b"IDAT":
            compressed.extend(body)
        elif kind == b"IEND":
            break
        offset += size + 12
    raw = zlib.decompress(compressed)
    stride = width * 4
    if len(raw) != height * (stride + 1):
        raise AssertionError("unexpected PNG data length")
    pixels = bytearray(height * stride)
    for y in range(height):
        mode = raw[y * (stride + 1)]
        if mode not in range(5):
            raise AssertionError("unknown PNG filter")
        row = raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)]
        for x, value in enumerate(row):
            left = pixels[y * stride + x - 4] if x >= 4 else 0
            above = pixels[(y - 1) * stride + x] if y else 0
            diagonal = pixels[(y - 1) * stride + x - 4] if y and x >= 4 else 0
            estimate = left + above - diagonal
            paeth = min((left, above, diagonal), key=lambda v: abs(estimate - v))
            predictor = (0, left, above, (left + above) // 2, paeth)[mode]
            pixels[y * stride + x] = (value + predictor) & 255
    return np.frombuffer(pixels, dtype=np.uint8).reshape(height, width, 4)


class NativePolygonLifecycle(unittest.TestCase):
    def test_native_families_dispatch_all_hooks_once(self):
        for base, args, kwargs in CASES:
            with self.subTest(base=base.__name__):
                class Authored(base):
                    def init_data(self):
                        self.events = ["data"]

                    def init_points(self):
                        self.events.append("points")
                        super().init_points()
                        self.shift(2 * m.UP)

                    def init_uniforms(self):
                        self.events.append("uniforms")
                        super().init_uniforms()

                    def init_colors(self):
                        self.events.append("colors")
                        super().init_colors()
                        self.set_color(m.GREEN)

                obj = Authored(*args, **kwargs)
                self.assertEqual(obj.events, ["data", "points", "uniforms", "colors"])
                self.assertGreater(obj.get_center()[1], 1.9)
                self.assertEqual(obj.get_color(), m.GREEN)
                self.assertGreater(obj.get_num_points(), 0)

    def test_overrides_can_replace_native_geometry(self):
        points = np.array([[0., 0., 0.], [.5, .5, 0.], [1., 0., 0.]])
        for base, args, kwargs in CASES:
            with self.subTest(base=base.__name__):
                class Authored(base):
                    def init_points(self):
                        self.empty_at_entry = self.get_num_points() == 0
                        self.set_points(points)

                obj = Authored(*args, **kwargs)
                self.assertTrue(obj.empty_at_entry)
                np.testing.assert_array_equal(obj.get_points(), points)

    def test_custom_dtype_for_every_native_family(self):
        for base, args, kwargs in CASES:
            with self.subTest(base=base.__name__):
                class Authored(base):
                    data_dtype = m.VMobject.data_dtype + [("weight", 1)]

                    def init_points(self):
                        super().init_points()
                        self.set_field("weight", 0, [7])

                obj = Authored(*args, **kwargs)
                self.assertEqual(obj.get_field("weight", 0), [7])
                self.assertGreater(obj.get_num_points(), 0)

    def test_line_mobject_endpoints_are_resolved_before_dispatch(self):
        class Authored(m.Line):
            def init_points(self):
                self.recipe = (self.start.copy(), self.end.copy(), self.buff)
                super().init_points()

        a = m.Circle(radius=.5).shift(2 * m.LEFT)
        b = m.Square(side_length=1).shift(2 * m.RIGHT)
        obj = Authored(a, b, buff=0)
        np.testing.assert_allclose(obj.recipe[0], [-1.5, 0, 0], atol=1e-6)
        np.testing.assert_allclose(obj.recipe[1], [1.5, 0, 0], atol=1e-6)
        np.testing.assert_allclose(obj.get_start(), obj.recipe[0], atol=1e-6)
        np.testing.assert_allclose(obj.get_end(), obj.recipe[1], atol=1e-6)

    def test_polygon_and_bezier_recipe_arrays_are_owned(self):
        vertices = np.array([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]])
        a, b = m.Polygon(*vertices), m.Polygon(*vertices)
        vertices[:] = 90
        a.vertices[0, 0] = 40
        np.testing.assert_array_equal(b.vertices, [[0, 0, 0], [1, 0, 0], [0, 1, 0]])
        controls = np.array([[-1., 0, 0], [-1, 2, 0], [1, 2, 0], [1, 0, 0]])
        curve = m.CubicBezier(*controls)
        saved = curve.control_points.copy()
        controls[:] = 90
        np.testing.assert_array_equal(curve.control_points, saved)

    def test_bound_rectangle_and_line_regenerate_without_losing_family(self):
        rectangle, line = m.Rectangle(), m.Line()
        child = m.Dot()
        rectangle.add(child)
        scene = m.Scene()
        scene.add(rectangle, line)
        rectangle.width = 6
        rectangle.height = 3
        rectangle.set_color(m.BLUE)
        rectangle.init_points()
        self.assertIs(rectangle[0], child)
        self.assertAlmostEqual(rectangle.get_width(), 6)
        self.assertAlmostEqual(rectangle.get_height(), 3)
        self.assertEqual(rectangle.get_color(), m.BLUE)
        line.start = np.array([-3., 1, 0])
        line.end = np.array([3., 1, 0])
        line.init_points()
        self.assertIs(scene.mobjects[1], line)
        np.testing.assert_allclose(line.get_start(), [-3, 1, 0], atol=1e-6)
        np.testing.assert_allclose(line.get_end(), [3, 1, 0], atol=1e-6)

    def test_hook_refusal_propagates_once(self):
        for base, args, kwargs in CASES:
            with self.subTest(base=base.__name__):
                error = ValueError("authored construction failure")
                calls = []

                def init_points(self):
                    calls.append(1)
                    raise error

                cls = type("Broken", (base,), {"init_points": init_points})
                with self.assertRaises(ValueError) as caught:
                    cls(*args, **kwargs)
                self.assertIs(caught.exception, error)
                self.assertEqual(calls, [1])

    def test_existing_constructor_errors_stay_explicit(self):
        with self.assertRaises(ZeroDivisionError):
            m.RegularPolygon(n=0)
        with self.assertRaises(IndexError):
            m.RegularPolygon(n=-1)
        with self.assertRaises(IndexError):
            m.Polygon()
        with self.assertRaises(TypeError):
            m.Rectangle(unknown_geometry_option=True)

    def test_custom_init_points_is_visible_in_real_png_output(self):
        class RaisedSquare(m.Square):
            def init_points(self):
                super().init_points()
                self.shift(2 * m.UP)

        class AuthoredScene(m.Scene):
            default_camera_config = dict(resolution=(96, 54), fps=8)

            def construct(self):
                self.add(RaisedSquare(side_length=1, fill_color=m.WHITE,
                                      fill_opacity=1, stroke_width=0))
                self.wait(1 / 8)

        destination = pathlib.Path(tempfile.mkdtemp(prefix="fmn-geometry-hooks-")) / "raised.png"
        result = AuthoredScene().render(destination, threads=1)
        self.assertEqual(result.frame_count, 1)
        pixels = png_pixels(destination)
        ys, xs = np.nonzero((pixels[:, :, :3] > 180).all(axis=2))
        self.assertGreater(len(xs), 10)
        self.assertLess(float(ys.mean()), 20)
        self.assertAlmostEqual(float(xs.mean()), 47.5, delta=1)


if __name__ == "__main__":
    unittest.main()
