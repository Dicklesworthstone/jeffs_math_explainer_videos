"""Permanent W10 acceptance tests executed inside fmn-python's Rust test.

The file is Python rather than Rust because the contract under test is ordinary
Python behavior: MRO, live NumPy arrays, weakrefs, descriptors, copy/pickle,
module imports, and callback reentrancy.
"""

import abc
import contextlib
import copy
import enum
import gc
import hmac
import importlib
import inspect
import io
import itertools
import json
import math
import os
import pickle
import pathlib
import re
import struct
import sys
import tempfile
import threading
import types
import weakref
import zlib

import numbers
import numpy as np

import manimlib
from manimlib import Animation, InteractiveScene, Mobject, Scene, VMobject

bridge_errors = importlib.import_module("manimlib.exceptions")


def png_rgba8_rows(path):
    """Decode the portal's non-interlaced RGBA8 PNGs with stdlib only."""
    payload = pathlib.Path(path).read_bytes()
    assert payload.startswith(b"\x89PNG\r\n\x1a\n")
    offset = 8
    width = height = None
    compressed = bytearray()
    while offset < len(payload):
        length = struct.unpack(">I", payload[offset : offset + 4])[0]
        kind = payload[offset + 4 : offset + 8]
        data = payload[offset + 8 : offset + 8 + length]
        offset += 12 + length
        if kind == b"IHDR":
            width, height, depth, color, compression, filtering, interlace = (
                struct.unpack(">IIBBBBB", data)
            )
            assert (depth, color, compression, filtering, interlace) == (8, 6, 0, 0, 0)
        elif kind == b"IDAT":
            compressed.extend(data)
        elif kind == b"IEND":
            break
    assert width is not None and height is not None
    raw = zlib.decompress(compressed)
    stride = width * 4
    assert len(raw) == height * (stride + 1)
    rows = []
    previous = bytearray(stride)

    def paeth(left, above, upper_left):
        prediction = left + above - upper_left
        distances = (
            abs(prediction - left),
            abs(prediction - above),
            abs(prediction - upper_left),
        )
        return (left, above, upper_left)[distances.index(min(distances))]

    for y in range(height):
        start = y * (stride + 1)
        filter_kind = raw[start]
        encoded = raw[start + 1 : start + 1 + stride]
        row = bytearray(stride)
        for index, value in enumerate(encoded):
            left = row[index - 4] if index >= 4 else 0
            above = previous[index]
            upper_left = previous[index - 4] if index >= 4 else 0
            predictor = {
                0: 0,
                1: left,
                2: above,
                3: (left + above) // 2,
                4: paeth(left, above, upper_left),
            }[filter_kind]
            row[index] = (value + predictor) & 0xFF
        rows.append(row)
        previous = row
    return width, height, rows


def assert_red_orientation_witness_is_above_origin(path):
    width, height, rows = png_rgba8_rows(path)
    red_pixels = [
        (x, y)
        for y, row in enumerate(rows)
        for x in range(width)
        if row[4 * x] > 160
        and row[4 * x + 1] < 100
        and row[4 * x + 2] < 100
        and row[4 * x + 3] > 160
    ]
    assert len(red_pixels) >= 4, f"missing red orientation witness in {path}"
    centroid_y = sum(y for _x, y in red_pixels) / len(red_pixels)
    assert centroid_y < height / 2, (path, centroid_y, height)


def assert_white_stroke_witness(path, minimum_pixels=8):
    _width, _height, rows = png_rgba8_rows(path)
    white_pixels = sum(
        1
        for row in rows
        for offset in range(0, len(row), 4)
        if min(row[offset : offset + 3]) >= 180 and row[offset + 3] > 160
    )
    assert white_pixels >= minimum_pixels, (path, white_pixels, minimum_pixels)

# Schema constants use a deliberately closed AST grammar: ordinary literal
# and arithmetic expressions resolve, while executable/private forms refuse.
assert manimlib._constant_expression("1 + 2 * 3", {}) == 7
assert manimlib._constant_expression("value[1]", {"value": (4, 9)}) == 9
for forbidden_constant in ("__import__('os')", "(1).__class__", "lambda: 1"):
    try:
        manimlib._constant_expression(forbidden_constant, {})
    except (AttributeError, ValueError):
        pass
    else:
        raise AssertionError(f"executable schema constant accepted: {forbidden_constant}")


class TracingMobject(Mobject):
    def __init__(self):
        self.calls = []
        super().__init__()

    def init_data(self):
        self.calls.append("init_data")

    def init_points(self):
        self.calls.append("init_points")
        self.resize(1)
        self.set_field("point", 0, [1.0, 2.0, 3.0])

    def init_uniforms(self):
        self.calls.append("init_uniforms")
        super().init_uniforms()
        self.uniforms["glow"] = 0.25


class PointMixin:
    def init_points(self):
        self.resize(1)
        self.set_field("point", 0, [7.0, 8.0, 9.0])
        self.mixin_ran = True


class MixedMobject(PointMixin, Mobject):
    pass


class CustomDtype(Mobject):
    data_dtype = [
        ("point", np.float32, (3,)),
        ("rgba", np.float32, (4,)),
        ("wobble", np.float32, (2,)),
    ]

    def init_points(self):
        self.resize(2)
        self.set_field("wobble", 1, [0.5, -0.5])


class CustomPointFields(Mobject):
    data_dtype = [
        ("point", np.float32, (3,)),
        ("control", np.float32, (3,)),
    ]
    pointlike_data_keys = ["point", "control"]

    def init_points(self):
        self.resize(1)
        self.set_field("point", 0, [1.0, 2.0, 0.0])
        self.set_field("control", 0, [3.0, 4.0, 0.0])


class MatrixPointMapperOverride(CustomPointFields):
    def apply_points_function(self, *args, **kwargs):
        self.matrix_point_mapper_calls += 1
        return super().apply_points_function(*args, **kwargs)


class MatrixFamilyWalkerOverride(CustomPointFields):
    def get_family(self, recurse=True):
        self.matrix_family_walker_calls += 1
        return super().get_family(recurse)


class UnsupportedDtype(Mobject):
    data_dtype = [("point", np.float64, (3,))]


# Lifecycle and ordinary MRO.
traced = TracingMobject()
assert traced.calls == ["init_data", "init_points", "init_uniforms"]
assert traced.get_field("point", 0) == [1.0, 2.0, 3.0]
assert traced.uniforms["glow"] == 0.25
mixed = MixedMobject()
assert mixed.mixin_ran
assert mixed.get_field("point", 0) == [7.0, 8.0, 9.0]
assert [base.__name__ for base in type(mixed).__mro__[:4]] == [
    "MixedMobject",
    "PointMixin",
    "Mobject",
    "_BridgeMobject",
]
try:
    UnsupportedDtype()
except TypeError as error:
    assert "native-endian float32" in str(error)
else:
    raise AssertionError("a non-f32 RecordBuffer schema was accepted")


# Subclass dtype → RecordSchema → zero-copy structured NumPy view.
custom = CustomDtype()
assert custom.field_names() == ["point", "rgba", "wobble"]
assert custom.data.dtype.names == ("point", "rgba", "wobble")
view = custom.data
assert view.shape == (2,)
assert view.dtype.itemsize == 9 * 4
view["point"][0] = [4.0, 5.0, 6.0]
assert custom.get_field("point", 0) == [4.0, 5.0, 6.0]
custom.set_field("wobble", 1, [2.5, -2.5])
assert view["wobble"][1].tolist() == [2.5, -2.5]
revision = custom.revision()
view["rgba"][0] = [0.1, 0.2, 0.3, 0.4]
del view
gc.collect()
assert custom.revision() > revision

old_view = custom.data
old_view["point"][1] = [11.0, 12.0, 13.0]
custom.resize(4)
assert old_view.shape == (2,)
assert old_view["point"][1].tolist() == [11.0, 12.0, 13.0]
old_view["point"][1] = [21.0, 22.0, 23.0]
assert custom.get_field("point", 1) == [11.0, 12.0, 13.0]
readonly = custom._data_array(False)
assert not readonly.flags.writeable
try:
    custom.resize(sys.maxsize)
except OverflowError:
    pass
else:
    raise AssertionError("an overflowing RecordBuffer resize was accepted")


# Deterministic utility functions are public semantic bindings, not schema
# placeholders. Core color interpolation and Chisel's integer/angle kernels
# remain the authorities behind the NumPy-compatible portal surface.
bezier_utils = importlib.import_module("manimlib.utils.bezier")
color_utils = importlib.import_module("manimlib.utils.color")
simple_utils = importlib.import_module("manimlib.utils.simple_functions")
space_utils = importlib.import_module("manimlib.utils.space_ops")
path_utils = importlib.import_module("manimlib.utils.paths")

utility_signatures = {
    bezier_utils.bezier: "(points)",
    bezier_utils.interpolate: "(start, end, alpha)",
    bezier_utils.inverse_interpolate: "(start, end, value)",
    bezier_utils.integer_interpolate: "(start, end, alpha)",
    color_utils.average_color: "(*colors)",
    color_utils.color_gradient: (
        "(reference_colors, length_of_output, interp_by_hsl=False)"
    ),
    color_utils.color_to_hex: "(color)",
    color_utils.color_to_int_rgb: "(color)",
    color_utils.color_to_int_rgba: "(color, opacity=1.0)",
    color_utils.color_to_rgb: "(color)",
    color_utils.color_to_rgba: "(color, alpha=1.0)",
    color_utils.hex_to_int: "(rgb_hex)",
    color_utils.hex_to_rgb: "(hex_code)",
    color_utils.int_to_hex: "(rgb_int)",
    color_utils.interpolate_color: (
        "(color1, color2, alpha, interp_by_hsl=False)"
    ),
    color_utils.interpolate_color_by_hsl: "(color1, color2, alpha)",
    color_utils.invert_color: "(color)",
    color_utils.rgb_to_color: "(rgb)",
    color_utils.rgb_to_hex: "(rgb)",
    color_utils.rgba_to_color: "(rgba)",
    simple_utils.binary_search: (
        "(function, target, lower_bound, upper_bound, tolerance=0.0001)"
    ),
    simple_utils.choose: "(n, k)",
    simple_utils.clip: "(a, min_a, max_a)",
    simple_utils.fdiv: "(a, b, zero_over_zero_value=None)",
    simple_utils.hash_string: "(string, n_bytes=16)",
    simple_utils.sigmoid: "(x)",
    space_utils.angle_between_vectors: "(v1, v2)",
    space_utils.angle_of_vector: "(vector)",
    space_utils.compass_directions: "(n=4, start_vect=array([1., 0., 0.]))",
    space_utils.cross: "(v1, v2, out=None)",
    space_utils.line_intersects_path: "(start, end, path)",
    path_utils.clockwise_path: "()",
    path_utils.counterclockwise_path: "()",
    path_utils.path_along_arc: "(arc_angle, axis=array([0., 0., 1.]))",
    path_utils.straight_path: "(start_points, end_points, alpha)",
}
assert len(utility_signatures) == 35
for function, declared_call_shape in utility_signatures.items():
    actual_call_shape = str(inspect.signature(function))
    assert actual_call_shape == declared_call_shape
    assert getattr(manimlib, function.__name__) is function
    assert function.__code__.co_name != "unavailable"

quadratic = bezier_utils.bezier(
    [np.array([0.0, 0.0]), np.array([2.0, 4.0]), np.array([4.0, 0.0])]
)
assert np.allclose(quadratic(0.25), [1.0, 1.5])
assert np.array_equal(
    bezier_utils.interpolate(
        np.array([0.0, 2.0]), np.array([4.0, 6.0]), 0.25
    ),
    [1.0, 3.0],
)
assert np.allclose(
    bezier_utils.inverse_interpolate(1.0, 5.0, np.array([1.0, 3.0, 5.0])),
    [0.0, 0.5, 1.0],
)
integer_value, integer_residue = bezier_utils.integer_interpolate(0, 10, 0.46)
assert integer_value == 4 and math.isclose(integer_residue, 0.6)
assert bezier_utils.integer_interpolate(3, 8, -1.0) == (3, 0.0)
assert bezier_utils.integer_interpolate(3, 8, 2.0) == (7, 1.0)
try:
    bezier_utils.bezier([])
except Exception as error:
    assert "empty list" in str(error)
else:
    raise AssertionError("empty Bezier controls were accepted")

black_to_white = color_utils.color_gradient(["#000000", "#FFFFFF"], 3)
assert [color.get_hex_l() for color in black_to_white] == [
    "#000000",
    "#B4B4B4",
    "#FFFFFF",
]
assert np.allclose(black_to_white[1].get_rgb(), [2**-0.5] * 3)
assert [color.get_hex_l() for color in color_utils.color_gradient(["#123456"], 1)] == [
    "#123456"
]
assert color_utils.color_gradient(iter(["#000000"]), 0) == []
assert color_utils.interpolate_color("#000000", "#FFFFFF", 0.25).get_hex_l() == (
    "#7F7F7F"
)
hsl_midpoint = color_utils.interpolate_color_by_hsl("#FF0000", "#0000FF", 0.5)
assert np.allclose(color_utils.color_to_rgb(hsl_midpoint), [0.0, 1.0, 0.0])
assert color_utils.average_color("#000000", "#FFFFFF").get_hex_l() == "#B4B4B4"
assert np.array_equal(color_utils.color_to_rgba("#123456", 0.25), [18 / 255, 52 / 255, 86 / 255, 0.25])
assert np.array_equal(color_utils.color_to_int_rgb("#123456"), [18, 52, 86])
assert np.array_equal(color_utils.color_to_int_rgba("#123456", 0.5), [18, 52, 86, 127])
assert color_utils.rgb_to_hex([18 / 255, 52 / 255, 86 / 255]) == "#123456"
assert color_utils.color_to_hex(color_utils.rgba_to_color([1.0, 0.0, 0.0, 0.2])) == "#FF0000"
assert color_utils.color_to_hex(color_utils.invert_color("#00FF00")) == "#FF00FF"
assert color_utils.hex_to_int("#123456") == 0x123456
assert color_utils.int_to_hex(0x123456) == "#123456"
assert np.allclose(color_utils.hex_to_rgb("#123456"), [18 / 255, 52 / 255, 86 / 255])
try:
    color_utils.color_gradient(["#000000"], 2)
except IndexError as error:
    assert str(error) == "list index out of range"
else:
    raise AssertionError("one-stop multi-sample gradient was accepted")
try:
    color_utils.color_gradient(["#000000", "#FFFFFF"], -1)
except ValueError as error:
    assert str(error) == "Number of samples, -1, must be non-negative."
else:
    raise AssertionError("negative-length gradient was accepted")
try:
    color_utils.average_color()
except TypeError as error:
    assert str(error) == "'numpy.float64' object is not iterable"
else:
    raise AssertionError("undefined empty color average was accepted")
try:
    color_utils.hex_to_rgb("not-a-color")
except ValueError as error:
    assert "invalid color" in str(error)
else:
    raise AssertionError("malformed color text was accepted")

assert simple_utils.choose(8, 3) == 56
assert simple_utils.hash_string("abc") == "ba7816bf8f01cfea"
assert simple_utils.hash_string("abc", 64) == (
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
)
assert simple_utils.hash_string("abc", 0) == ""
assert simple_utils.hash_string("", 8) == "e3b0c442"
assert simple_utils.clip(-2.0, -1.0, 1.0) == -1.0
assert simple_utils.clip(2.0, -1.0, 1.0) == 1.0
assert simple_utils.clip(0.25, -1.0, 1.0) == 0.25
assert np.allclose(simple_utils.sigmoid(np.array([-1.0, 0.0, 1.0])), [
    1 / (1 + math.e),
    0.5,
    1 / (1 + math.exp(-1)),
])
assert np.allclose(
    simple_utils.fdiv(
        np.array([0.0, 2.0]), np.array([0.0, 4.0]), zero_over_zero_value=7.0
    ),
    [7.0, 0.5],
)
assert math.isclose(
    simple_utils.binary_search(lambda value: value * value, 4.0, 0.0, 4.0),
    4.0,
    abs_tol=1e-4,
)
assert simple_utils.binary_search(lambda value: value * value, -1.0, 0.0, 4.0) is None

assert math.isclose(space_utils.angle_of_vector([0.0, 1.0]), math.pi / 2)
assert math.isclose(
    space_utils.angle_between_vectors([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
    math.pi / 2,
)
assert space_utils.angle_between_vectors([0.0] * 4, [1.0] * 4) == 0.0
assert space_utils.line_intersects_path(
    [0.0, -1.0],
    [0.0, 1.0],
    np.array([[-1.0, 0.0], [1.0, 0.0]]),
)
assert space_utils.line_intersects_path(
    [0.0, -1.0, 8.0],
    [0.0, 1.0, -3.0],
    np.array([[-1.0, 0.0, 12.0], [1.0, 0.0, -5.0]]),
)
assert not space_utils.line_intersects_path(
    [0.0, 0.0],
    [0.0, 1.0],
    np.array([[-1.0, 0.0], [1.0, 0.0]]),
)
try:
    space_utils.line_intersects_path([0.0], [0.0, 1.0], [[-1.0, 0.0]])
except ValueError as error:
    assert "two or three components" in str(error)
else:
    raise AssertionError("line_intersects_path accepted a one-dimensional point")
out_vector = np.zeros(3)
assert space_utils.cross([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], out_vector) is out_vector
assert np.array_equal(out_vector, [0.0, 0.0, 1.0])
assert np.array_equal(
    space_utils.cross(
        np.array([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
        np.array([[0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]),
    ),
    [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0]],
)
assert np.allclose(
    space_utils.compass_directions(),
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, -1.0, 0.0]],
    atol=1e-15,
)
try:
    space_utils.angle_of_vector([1.0])
except ValueError as error:
    assert "at least two" in str(error)
else:
    raise AssertionError("one-dimensional polar vector was accepted")

path_start = np.array([[0.0, 0.0, 0.0], [2.0, 1.0, 0.0]])
path_end = np.array([[2.0, 0.0, 0.0], [4.0, 1.0, 0.0]])
assert np.allclose(
    path_utils.straight_path(path_start, path_end, 0.25),
    [[0.5, 0.0, 0.0], [2.5, 1.0, 0.0]],
)
assert path_utils.path_along_arc(0.0) is path_utils.straight_path
assert np.allclose(
    path_utils.counterclockwise_path()(path_start[:1], path_end[:1], 0.5),
    [[1.0, -1.0, 0.0]],
)
assert np.allclose(
    path_utils.clockwise_path()(path_start[:1], path_end[:1], 0.5),
    [[1.0, 1.0, 0.0]],
)
assert np.allclose(
    path_utils.path_along_arc(math.pi, axis=np.zeros(3))(
        path_start[:1], path_end[:1], 0.5
    ),
    [[1.0, -1.0, 0.0]],
)
per_point_arcs = np.array([0.0, math.pi])
path_utils.path_along_arc(per_point_arcs)(path_start, path_end, 0.5)
assert np.array_equal(per_point_arcs, [0.01, math.pi])


# Reference point mutation composes the public NumPy resize policies over the
# live Marionette RecordBuffer.  All subclass lanes move together, while a
# size-changing write detaches old views through the generation protocol.
iterables = importlib.import_module("manimlib.utils.iterables")
assert manimlib.resize_array is iterables.resize_array
assert manimlib.resize_preserving_order is iterables.resize_preserving_order
assert str(inspect.signature(manimlib.resize_array)) == "(nparray, length)"
assert str(inspect.signature(manimlib.resize_preserving_order)) == (
    "(nparray, length)"
)
resize_probe = np.array([[1.0], [2.0], [3.0]])
assert np.array_equal(
    manimlib.resize_array(resize_probe, 5),
    [[1.0], [2.0], [3.0], [1.0], [2.0]],
)
assert np.array_equal(
    manimlib.resize_preserving_order(resize_probe, 5),
    [[1.0], [1.0], [2.0], [2.0], [3.0]],
)
assert manimlib.resize_preserving_order(np.zeros((0, 3)), 2).shape == (2,)

assert list(inspect.signature(Mobject.resize_points).parameters) == [
    "self",
    "new_length",
    "resize_func",
]
assert (
    inspect.signature(Mobject.resize_points).parameters["resize_func"].default
    is manimlib.resize_array
)
assert str(inspect.signature(Mobject.set_points)) == "(self, points)"
assert str(inspect.signature(Mobject.append_points)) == "(self, new_points)"
assert str(inspect.signature(Mobject.clear_points)) == "(self)"
assert str(inspect.signature(manimlib.PMobject.add_points)) == (
    "(self, points, rgbas=None, color=None, opacity=None)"
)
assert str(inspect.signature(manimlib.PMobject.add_point)) == (
    "(self, point, rgba=None, color=None, opacity=None)"
)
assert str(inspect.signature(manimlib.PMobject.set_points)) == "(self, points)"
assert str(inspect.signature(VMobject.set_points)) == "(self, points)"
assert str(inspect.signature(VMobject.append_points)) == "(self, points)"
assert str(inspect.signature(VMobject.set_anchors_and_handles)) == (
    "(self, anchors, handles)"
)
assert str(inspect.signature(VMobject.set_points_as_corners)) == "(self, points)"
assert str(inspect.signature(VMobject.start_new_path)) == "(self, point)"
assert str(inspect.signature(VMobject.add_cubic_bezier_curve)) == (
    "(self, anchor1, handle1, handle2, anchor2)"
)
assert str(inspect.signature(VMobject.add_cubic_bezier_curve_to)) == (
    "(self, handle1, handle2, anchor)"
)
assert str(inspect.signature(VMobject.add_quadratic_bezier_curve_to)) == (
    "(self, handle, anchor, allow_null_curve=True)"
)
assert str(inspect.signature(VMobject.add_line_to)) == (
    "(self, point, allow_null_line=True)"
)
assert str(inspect.signature(VMobject.add_smooth_curve_to)) == "(self, point)"
assert str(inspect.signature(VMobject.add_smooth_cubic_curve_to)) == (
    "(self, handle, point)"
)
assert str(inspect.signature(VMobject.add_arc_to)) == (
    "(self, point, angle, n_components=None, threshold=0.001)"
)
assert str(inspect.signature(VMobject.add_points_as_corners)) == "(self, points)"
assert str(inspect.signature(VMobject.add_subpath)) == "(self, points)"
assert str(inspect.signature(VMobject.append_vectorized_mobject)) == (
    "(self, vmobject)"
)
assert str(inspect.signature(VMobject.has_new_path_started)) == "(self)"
assert str(inspect.signature(VMobject.get_last_point)) == "(self)"
assert str(inspect.signature(VMobject.get_reflection_of_last_handle)) == "(self)"
assert str(inspect.signature(VMobject.close_path)) == "(self, smooth=False)"
assert str(inspect.signature(VMobject.is_closed)) == "(self)"
assert str(inspect.signature(VMobject.consider_points_equal)) == "(self, p0, p1)"
assert str(inspect.signature(VMobject.get_anchors_and_handles)) == "(self)"
assert str(inspect.signature(VMobject.get_start_anchors)) == "(self)"
assert str(inspect.signature(VMobject.get_end_anchors)) == "(self)"
assert str(inspect.signature(VMobject.get_anchors)) == "(self)"
assert str(inspect.signature(VMobject.get_bezier_tuples_from_points)) == (
    "(self, points)"
)
assert str(inspect.signature(VMobject.get_bezier_tuples)) == "(self)"
assert str(inspect.signature(VMobject.get_nth_curve_points)) == "(self, n)"
assert str(inspect.signature(VMobject.get_nth_curve_function)) == "(self, n)"
assert str(inspect.signature(VMobject.get_subpath_end_indices_from_points)) == (
    "(self, points)"
)
assert str(inspect.signature(VMobject.get_subpath_end_indices)) == "(self)"
assert str(inspect.signature(VMobject.get_subpaths_from_points)) == "(self, points)"
assert str(inspect.signature(VMobject.get_subpaths)) == "(self)"
assert str(inspect.signature(VMobject.insert_n_curves_to_point_list)) == (
    "(self, n, points)"
)
assert str(inspect.signature(VMobject.insert_n_curves)) == "(self, n, recurse=True)"
assert str(inspect.signature(VMobject.align_points)) == "(self, vmobject)"
assert list(inspect.signature(VMobject.subdivide_sharp_curves).parameters) == [
    "self",
    "angle_threshold",
    "recurse",
]
assert math.isclose(
    inspect.signature(VMobject.subdivide_sharp_curves)
    .parameters["angle_threshold"]
    .default,
    30 * manimlib.DEG,
    rel_tol=0.0,
    abs_tol=0.0,
)
assert (
    inspect.signature(VMobject.subdivide_sharp_curves)
    .parameters["recurse"]
    .default
    is True
)
assert list(inspect.signature(VMobject.is_smooth).parameters) == ["self", "angle_tol"]
assert math.isclose(
    inspect.signature(VMobject.is_smooth).parameters["angle_tol"].default,
    manimlib.DEG,
    rel_tol=0.0,
    abs_tol=0.0,
)
assert str(inspect.signature(VMobject.change_anchor_mode)) == "(self, mode)"
assert str(inspect.signature(VMobject.make_approximately_smooth)) == (
    "(self, recurse=True)"
)
assert str(inspect.signature(VMobject.make_jagged)) == "(self, recurse=True)"
assert str(inspect.signature(VMobject.reverse_points)) == "(self, recurse=True)"
assert str(inspect.signature(VMobject.get_area_vector)) == "(self)"
assert str(inspect.signature(VMobject.get_unit_normal)) == "(self, refresh=False)"
assert str(inspect.signature(VMobject.get_arc_length)) == "(self, n_sample_points=None)"
assert str(inspect.signature(VMobject.point_from_proportion)) == "(self, alpha)"
assert VMobject.tolerance_for_point_equality == 1e-8

mutation = CustomDtype()
mutation.data["point"][:] = [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]
mutation.data["rgba"][:] = [
    [0.1, 0.2, 0.3, 0.4],
    [0.5, 0.6, 0.7, 0.8],
]
mutation.data["wobble"][:] = [[10.0, 11.0], [20.0, 21.0]]
mutation_before = mutation.data.copy()
mutation_old_view = mutation.data
replacement_points = np.array(
    [
        [-4.0, 0.0, 0.0],
        [-2.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
    ]
)
assert mutation.set_points(replacement_points) is mutation
mutation_expected = manimlib.resize_preserving_order(mutation_before, 4)
mutation_expected["point"][:] = replacement_points
assert np.array_equal(mutation.data, mutation_expected)
assert mutation_old_view.shape == (2,)
mutation_old_view["point"][0] = [99.0, 99.0, 99.0]
assert np.array_equal(mutation.data, mutation_expected)

# Same-length replacement keeps the current generation live.  Clearing saves
# the first complete record as the Reference default row, and growing again
# restores every non-point field from it.
same_generation = mutation.data
same_length_points = replacement_points + [0.0, 1.0, 0.0]
assert mutation.set_points(same_length_points) is mutation
assert np.array_equal(same_generation["point"], same_length_points)
saved_default = mutation.data[0].copy()
cleared_generation = mutation.data
assert mutation.clear_points() is mutation
assert mutation.get_points().shape == (0, 3)
assert cleared_generation.shape == (4,)
assert mutation.set_points([[7.0, 8.0, 9.0]]) is mutation
assert np.array_equal(mutation.data["point"], [[7.0, 8.0, 9.0]])
assert np.array_equal(mutation.data["rgba"][0], saved_default["rgba"])
assert np.array_equal(mutation.data["wobble"][0], saved_default["wobble"])

# append_points uses the Reference cyclic resize first, then copies the last
# retained record into the tail before installing the new point rows.
appended = CustomDtype()
appended.set_points(
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]
)
appended.data["rgba"][:] = [
    [0.1, 0.0, 0.0, 1.0],
    [0.2, 0.0, 0.0, 1.0],
    [0.3, 0.0, 0.0, 1.0],
]
appended.data["wobble"][:] = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]
append_before = appended.data.copy()
append_tail = np.array([[3.0, 0.0, 0.0], [4.0, 0.0, 0.0]])
assert appended.append_points(append_tail) is appended
append_expected = manimlib.resize_array(append_before, 5)
append_expected[3:] = append_expected[2]
append_expected["point"][3:] = append_tail
assert np.array_equal(appended.data, append_expected)

point_cloud = manimlib.PMobject()
assert point_cloud.set_points([]) is point_cloud
assert point_cloud.get_points().shape == (0, 3)
assert point_cloud.set_points([[1.0, 2.0, 3.0]]) is point_cloud
assert np.array_equal(point_cloud.get_points(), [[1.0, 2.0, 3.0]])
point_cloud.set_points([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]])
point_cloud.data["rgba"][:] = [
    [0.1, 0.0, 0.0, 1.0],
    [0.2, 0.0, 0.0, 1.0],
    [0.3, 0.0, 0.0, 1.0],
]
point_cloud_before = point_cloud.data.copy()
point_cloud_tail = np.array([[3.0, 0.0, 0.0], [4.0, 0.0, 0.0]])
point_cloud_rgbas = np.array(
    [[0.8, 0.1, 0.2, 0.5], [0.7, 0.2, 0.1, 0.25]]
)
assert point_cloud.add_points(point_cloud_tail, point_cloud_rgbas) is point_cloud
point_cloud_expected = manimlib.resize_array(point_cloud_before, 5)
point_cloud_expected[3:] = point_cloud_expected[2]
point_cloud_expected["point"][3:] = point_cloud_tail
point_cloud_expected["rgba"][3:] = point_cloud_rgbas
assert np.array_equal(point_cloud.data, point_cloud_expected)

# VMobject overrides enforce the shared-anchor odd/even contract before any
# mutation and refresh the native joint-angle column through Stage::set_points.
vm_source = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]]
)
# A bare VMobject starts with the Reference's visible-stroke/transparent-fill
# defaults, and the first Chisel corner-path allocation must retain them.
# SquareOnASphere constructs its three elbow marks through exactly this path
# and then changes only color/width; losing the default opacity leaves valid
# scene objects that are nevertheless invisible to Lumen.
assert np.allclose(vm_source.data["stroke_rgba"][:, :3], [0xDD / 255] * 3)
assert np.allclose(vm_source.data["stroke_rgba"][:, 3], 1.0)
assert np.allclose(vm_source.data["stroke_width"], 4.0)
assert np.allclose(vm_source.data["fill_rgba"][:, :3], [0x88 / 255] * 3)
assert np.allclose(vm_source.data["fill_rgba"][:, 3], 0.0)
assert vm_source.uniforms["joint_type"] == 1.0
assert vm_source.uniforms["anti_alias_width"] == 1.5
assert vm_source.set_stroke(manimlib.WHITE, 2) is vm_source
assert np.allclose(vm_source.data["stroke_rgba"][:, :3], 1.0)
assert np.allclose(vm_source.data["stroke_rgba"][:, 3], 1.0)
assert np.allclose(vm_source.data["stroke_width"], 2.0)
vm_mutation = VMobject()
assert vm_mutation.set_points(vm_source.get_points().copy()) is vm_mutation
assert np.array_equal(
    vm_mutation.data["joint_angle"], vm_source.data["joint_angle"]
)
vm_before_invalid = vm_mutation.data.copy()
try:
    vm_mutation.set_points([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])
except AssertionError:
    pass
else:
    raise AssertionError("VMobject.set_points accepted an even point run")
assert np.array_equal(vm_mutation.data, vm_before_invalid)
try:
    vm_mutation.append_points([[3.0, 0.0, 0.0]])
except AssertionError:
    pass
else:
    raise AssertionError("VMobject.append_points accepted an odd append run")
assert np.array_equal(vm_mutation.data, vm_before_invalid)
assert vm_mutation.append_points(
    [[2.5, -0.5, 0.0], [3.0, 0.0, 0.0]]
) is vm_mutation
assert vm_mutation.get_num_points() == 7

# A bound write bakes a pending placement exactly once and then replaces the
# world-space records, rather than leaving an affine transform to double-apply.
bound_mutation = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]
)
bound_mutation_scene = Scene()
bound_mutation_scene.add(bound_mutation)
bound_mutation.shift([10.0, 0.0, 0.0])
bound_replacement = np.array(
    [[-2.0, 0.0, 0.0], [-1.5, 0.0, 0.0], [-1.0, 0.0, 0.0]]
)
assert bound_mutation.set_points(bound_replacement) is bound_mutation
assert np.array_equal(bound_mutation.get_points(), bound_replacement)


# Live typed uniforms and open extension keys.
assert custom.uniforms["anti_alias_width"] == 1.5
custom.uniforms["anti_alias_width"] = 2.25
custom.uniforms["plugin_value"] = {"nested": [1, 2]}
assert custom.uniforms["anti_alias_width"] == 2.25
assert custom.uniforms["plugin_value"] == {"nested": [1, 2]}


# Detached family graphs bind transactionally and remain live afterwards.
scene = Scene()
parent = Mobject()
parent.resize(1)
parent.set_field("point", 0, [1.0, 1.0, 0.0])
child = Mobject()
child.resize(1)
child.set_field("point", 0, [2.0, 2.0, 0.0])
outsider = Mobject()
parent.add(child)
assert parent.family_size() == 2
scene.add(parent, outsider)
assert scene.root_count() == 2
assert scene.mobjects[0] is parent
assert parent.family_size() == 2
parent.submobjects.clear()
assert parent.family_size() == 1
parent.submobjects.append(child)
assert parent.family_size() == 2
try:
    parent.submobjects.append(child)
except ValueError:
    pass
else:
    raise AssertionError("duplicate family edge was accepted")
try:
    child.submobjects.append(parent)
except bridge_errors.FamilyCycleError:
    pass
else:
    raise AssertionError("family cycle was accepted")

# The public family-list methods preserve the Reference's ordered
# identity-set semantics while every accepted edit is mirrored into the
# detached nursery or bound Marionette graph.  In particular, add is not a
# raw list extension: repeated and already-present children are ignored.
assert str(inspect.signature(Mobject.add)) == "(self, *mobjects)"
assert str(inspect.signature(Mobject.remove)) == (
    "(self, *to_remove, reassemble=True, recurse=True)"
)
assert str(inspect.signature(Mobject.clear)) == "(self)"
assert str(inspect.signature(Mobject.set_submobjects)) == (
    "(self, submobject_list)"
)
assert str(inspect.signature(Mobject.reverse_submobjects)) == "(self)"
family_add_root = Mobject()
family_add_a = Mobject()
family_add_b = Mobject()
family_add_c = Mobject()
assert family_add_root.add(
    family_add_a,
    family_add_a,
    family_add_b,
    family_add_a,
) is family_add_root
assert family_add_root.submobjects == [family_add_a, family_add_b]
assert family_add_root.add(family_add_a) is family_add_root
assert family_add_root.submobjects == [family_add_a, family_add_b]

# Self-containment is rejected before any member of the batch mutates the
# family.  A later invalid ordinary Python value preserves the successfully
# attached Mobject prefix but is itself never admitted to the safe graph.
try:
    family_add_root.add(family_add_c, family_add_root)
except Exception as error:
    assert str(error) == "Mobject cannot contain self"
else:
    raise AssertionError("Mobject.add accepted self-containment")
assert family_add_root.submobjects == [family_add_a, family_add_b]
try:
    family_add_root.add(family_add_c, object())
except TypeError as error:
    assert str(error) == "submobjects must be Mobject instances"
else:
    raise AssertionError("Mobject.add accepted a non-Mobject")
assert family_add_root.submobjects == [family_add_a, family_add_b, family_add_c]

# fm-5wq.31: adding N children is one commit, not N (each commit re-projects
# the whole child list: 7,094 rects in Groups were O(n^2), 33 s). Membership,
# order and parent edges are those of child-by-child commits.
family_add_live = type(family_add_root.submobjects)
family_add_commits = []
family_add_commit = family_add_live._commit


def family_add_counting(live, candidate):
    family_add_commits.append(len(candidate))
    return family_add_commit(live, candidate)


family_add_live._commit = family_add_counting
try:
    family_add_children = [Mobject() for _ in range(64)]
    family_add_wide = manimlib.Group(*family_add_children, family_add_children[3])
    assert family_add_commits == [64], family_add_commits
    assert list(family_add_wide.submobjects) == family_add_children
    assert all(child.parents == [family_add_wide] for child in family_add_children)
    # A refusal inside the batch (a cycle under a scene-bound parent)
    # changes nothing, then the child-by-child path keeps the prefix and
    # raises the first failing child's error.
    family_add_scene = manimlib.Scene()
    family_add_scene.add(family_add_wide)
    family_add_commits.clear()
    family_add_cyclic = Mobject()
    try:
        family_add_children[0].add(family_add_cyclic, family_add_wide)
    except Exception as error:
        family_add_cycle_error = str(error)
        assert type(error).__name__ == "FamilyCycleError", family_add_cycle_error
    else:
        raise AssertionError("Mobject.add accepted a cycle")
    assert family_add_children[0].submobjects == [family_add_cyclic], family_add_cycle_error
    assert family_add_cyclic.parents == [family_add_children[0]]
    assert family_add_commits == [2, 1, 2], family_add_commits

    # An uninitialized child never enters the batch: it keeps the
    # child-by-child refusal and prefix.
    class FamilyAddUninitialized(Mobject):
        def __init__(self):
            pass

    family_add_owner = Mobject()
    try:
        family_add_owner.add(Mobject(), FamilyAddUninitialized())
    except Exception as error:
        assert type(error).__name__ == "ForeignStageError", error
    else:
        raise AssertionError("Mobject.add accepted an uninitialized mobject")
    assert len(family_add_owner.submobjects) == 1
finally:
    family_add_live._commit = family_add_commit

# fm-5wq.41: a cycle on a detached graph (no Scene yet) is refused before it
# forms, as on a bound graph; it used to hang in note_changed_family.
cycle_leaf = Mobject()
cycle_mid = manimlib.Group(cycle_leaf)
cycle_top = manimlib.Group(cycle_mid)
for cycle_child, cycle_ancestor in ((cycle_leaf, cycle_mid), (cycle_leaf, cycle_top)):
    try:
        cycle_child.add(cycle_ancestor)
    except bridge_errors.FamilyCycleError as error:
        assert "cycle" in str(error), error
    else:
        raise AssertionError("Mobject.add accepted a detached family cycle")
    assert cycle_child.submobjects == []
cycle_prefix = Mobject()
try:
    cycle_leaf.add(cycle_prefix, cycle_top)
except bridge_errors.FamilyCycleError:
    pass
else:
    raise AssertionError("Mobject.add accepted a detached family cycle")
assert cycle_leaf.submobjects == [cycle_prefix]
assert cycle_top.parents == [] and cycle_mid.parents == [cycle_top]


# The cycle check never replaces Marionette's own refusal of an
# uninitialized child (no child list yet), nor its prefix.
class CycleUninitialized(Mobject):
    def __init__(self):
        pass


cycle_owner = Mobject()
try:
    cycle_owner.add(Mobject(), CycleUninitialized())
except Exception as error:
    assert type(error).__name__ == "ForeignStageError", error
else:
    raise AssertionError("Mobject.add accepted an uninitialized mobject")
assert len(cycle_owner.submobjects) == 1

# set_submobjects is Reference clear-then-add, including identity dedup and
# its observable prefix on a later failure.
assert family_add_root.set_submobjects(
    [family_add_b, family_add_b, family_add_c]
) is family_add_root
assert family_add_root.submobjects == [family_add_b, family_add_c]
try:
    family_add_root.set_submobjects([family_add_a, object(), family_add_b])
except TypeError as error:
    assert str(error) == "submobjects must be Mobject instances"
else:
    raise AssertionError("Mobject.set_submobjects accepted a non-Mobject")
assert family_add_root.submobjects == [family_add_a]

# Recursive remove snapshots family order and detaches every matching edge;
# recurse=False touches only the selected root.  Shared descendants retain
# their Python identity while both parent edges are edited independently.
family_remove_shared = Mobject()
family_remove_left = Mobject().add(family_remove_shared)
family_remove_right = Mobject().add(family_remove_shared)
family_remove_root = Mobject().add(family_remove_left, family_remove_right)
assert family_remove_root.remove(family_remove_shared) is family_remove_root
assert family_remove_left.submobjects == []
assert family_remove_right.submobjects == []
family_remove_left.add(family_remove_shared)
family_remove_right.add(family_remove_shared)
assert (
    family_remove_root.remove(
        family_remove_shared,
        recurse=False,
        reassemble=False,
    )
    is family_remove_root
)
assert family_remove_left.submobjects == [family_remove_shared]
assert family_remove_right.submobjects == [family_remove_shared]

family_remove_other = Mobject()
family_remove_root.add(family_remove_other)
try:
    family_remove_root.remove(family_remove_other, object(), recurse=False)
except TypeError as error:
    assert str(error) == "submobjects must be Mobject instances"
else:
    raise AssertionError("Mobject.remove accepted a non-Mobject")
assert family_remove_other not in family_remove_root.submobjects

assert family_remove_root.reverse_submobjects() is family_remove_root
assert family_remove_root.submobjects == [family_remove_right, family_remove_left]
assert family_remove_root.clear() is family_remove_root
assert family_remove_root.submobjects == []
assert family_remove_left.submobjects == [family_remove_shared]
assert family_remove_right.submobjects == [family_remove_shared]

# The same surface commits real native child order after Scene adoption and
# keeps the typed foreign-stage and cycle refusals transactional.
family_mutation_scene = Scene()
bound_family_root = Mobject()
bound_family_a = Mobject()
bound_family_b = Mobject()
family_mutation_scene.add(bound_family_root)
assert bound_family_root.add(bound_family_a, bound_family_b, bound_family_a) is (
    bound_family_root
)
assert bound_family_root.submobjects == [bound_family_a, bound_family_b]
assert bound_family_root.family_size() == 3
assert bound_family_root.reverse_submobjects() is bound_family_root
assert bound_family_root.get_family() == [
    bound_family_root,
    bound_family_b,
    bound_family_a,
]

foreign_family_scene = Scene()
foreign_family_child = Mobject()
foreign_family_scene.add(foreign_family_child)
bound_before_foreign = list(bound_family_root.submobjects)
try:
    bound_family_root.add(foreign_family_child)
except bridge_errors.ForeignStageError:
    pass
else:
    raise AssertionError("Mobject.add accepted a foreign-stage child")
assert bound_family_root.submobjects == bound_before_foreign

try:
    bound_family_a.add(bound_family_root)
except bridge_errors.FamilyCycleError:
    pass
else:
    raise AssertionError("Mobject.add accepted a family cycle")
assert bound_family_a.submobjects == []

other_scene = Scene()
try:
    other_scene.add(parent)
except bridge_errors.ForeignStageError:
    pass
else:
    raise AssertionError("cross-Scene handle was accepted")


# Public Scene draw-order edits route through the native runtime. Promotion
# is the same stable add operation as the Reference; demotion prepends the
# whole argument batch without a z-sort and can adopt a detached mobject.
order_scene = Scene()
order_back = Mobject()
order_middle = Mobject()
order_front = Mobject()
order_scene.add(order_back, order_middle, order_front)
assert order_scene.bring_to_front(order_back) is order_scene
assert order_scene.mobjects == [order_middle, order_front, order_back]
assert order_scene.bring_to_back(order_back, order_front) is order_scene
assert order_scene.mobjects == [order_back, order_front, order_middle]

detached_back = Mobject()
assert order_scene.bring_to_back(detached_back) is order_scene
assert order_scene.mobjects == [detached_back, order_back, order_front, order_middle]
assert order_scene.mobjects[0] is detached_back

order_before_refusal = list(order_scene.mobjects)
try:
    order_scene.bring_to_front(object())
except TypeError:
    pass
else:
    raise AssertionError("Scene.bring_to_front accepted a non-Mobject")
assert order_scene.mobjects == order_before_refusal

foreign_order_scene = Scene()
foreign_order_mobject = Mobject()
foreign_order_scene.add(foreign_order_mobject)
try:
    order_scene.bring_to_back(foreign_order_mobject)
except bridge_errors.ForeignStageError:
    pass
else:
    raise AssertionError("Scene.bring_to_back accepted a foreign-stage Mobject")
assert order_scene.mobjects == order_before_refusal


# Scene.replace is an exact top-level splice. Its absent-source guard runs
# before replacement binding, matching the Reference's membership test.
replace_scene = Scene()
replace_back = Mobject()
replace_middle = Mobject()
replace_front = Mobject()
replace_scene.add(replace_back, replace_middle, replace_front)
replacement_left = Mobject()
replacement_right = Mobject()
assert replace_scene.replace(
    replace_middle,
    replacement_left,
    replacement_right,
) is replace_scene
assert replace_scene.mobjects == [
    replace_back,
    replacement_left,
    replacement_right,
    replace_front,
]
assert replace_scene.mobjects[1] is replacement_left
assert replace_scene.mobjects[2] is replacement_right

assert replace_scene.replace(replacement_left) is replace_scene
assert replace_scene.mobjects == [replace_back, replacement_right, replace_front]

detached_source = Mobject()
detached_replacement = Mobject()
assert replace_scene.replace(detached_source, detached_replacement) is replace_scene
assert replace_scene.mobjects == [replace_back, replacement_right, replace_front]
assert not detached_source._is_bound()
assert not detached_replacement._is_bound()

nonmobject_replacement = Mobject()
assert replace_scene.replace(object(), nonmobject_replacement) is replace_scene
assert replace_scene.mobjects == [replace_back, replacement_right, replace_front]
assert not nonmobject_replacement._is_bound()

family_scene = Scene()
family_root = Mobject()
family_child = Mobject()
family_root.submobjects.append(family_child)
family_scene.add(family_root)
nonroot_replacement = Mobject()
assert family_scene.replace(family_child, nonroot_replacement) is family_scene
assert family_scene.mobjects == [family_root]
assert not nonroot_replacement._is_bound()

replace_before_refusal = list(replace_scene.mobjects)
try:
    replace_scene.replace(replacement_right, object())
except TypeError:
    pass
else:
    raise AssertionError("Scene.replace accepted a non-Mobject replacement")
assert replace_scene.mobjects == replace_before_refusal

foreign_replace_scene = Scene()
foreign_replacement = Mobject()
foreign_replace_scene.add(foreign_replacement)
foreign_source_replacement = Mobject()
assert replace_scene.replace(
    foreign_replacement,
    foreign_source_replacement,
) is replace_scene
assert replace_scene.mobjects == replace_before_refusal
assert not foreign_source_replacement._is_bound()
try:
    replace_scene.replace(replacement_right, foreign_replacement)
except bridge_errors.ForeignStageError:
    pass
else:
    raise AssertionError("Scene.replace accepted a foreign-stage replacement")
assert replace_scene.mobjects == replace_before_refusal


# Scene.get_mobjects returns a fresh ordered snapshot of the native draw list.
# Scene.remove_all_except resets that list through native clear + stable add,
# preserving batch duplicates and adopting detached mobjects.
membership_scene = Scene()
membership_back = Mobject().set_z_index(-1)
membership_old = Mobject()
membership_front = Mobject().set_z_index(1)
membership_scene.add(membership_old, membership_front, membership_back)
membership_snapshot = membership_scene.get_mobjects()
assert membership_snapshot == [membership_back, membership_old, membership_front]
assert membership_snapshot is not membership_scene.get_mobjects()
assert membership_snapshot[0] is membership_back
membership_snapshot.clear()
assert membership_scene.mobjects == [membership_back, membership_old, membership_front]

membership_detached = Mobject()
assert membership_scene.remove_all_except(
    membership_front,
    membership_detached,
    membership_front,
    membership_back,
) is membership_scene
assert membership_scene.get_mobjects() == [
    membership_back,
    membership_detached,
    membership_front,
    membership_front,
]
assert membership_scene.get_mobjects()[1] is membership_detached
assert membership_detached._is_bound()

membership_before_refusal = membership_scene.get_mobjects()
try:
    membership_scene.remove_all_except(membership_back, object())
except TypeError:
    pass
else:
    raise AssertionError("Scene.remove_all_except accepted a non-Mobject")
assert membership_scene.get_mobjects() == membership_before_refusal

foreign_membership_scene = Scene()
foreign_membership_mobject = Mobject()
foreign_membership_scene.add(foreign_membership_mobject)
try:
    membership_scene.remove_all_except(foreign_membership_mobject)
except bridge_errors.ForeignStageError:
    pass
else:
    raise AssertionError("Scene.remove_all_except accepted a foreign-stage Mobject")
assert membership_scene.get_mobjects() == membership_before_refusal

assert membership_scene.remove_all_except() is membership_scene
assert membership_scene.get_mobjects() == []


# Scene.get_group preserves the Reference's exact concrete group choice over
# the native Mobject and VMobject family surfaces.
assert str(inspect.signature(Scene.get_group)) == "(self, *mobjects)"
group_scene = Scene()
group_circle = manimlib.Circle(radius=0.2)
group_square = manimlib.Square(side_length=0.4)
vector_group = group_scene.get_group(group_circle, group_square)
assert type(vector_group) is manimlib.VGroup
assert list(vector_group.submobjects) == [group_circle, group_square]

group_vmobject = manimlib.Circle(radius=0.3)
group_mobject = Mobject()
mixed_group = group_scene.get_group(group_vmobject, group_mobject)
assert type(mixed_group) is manimlib.Group
assert list(mixed_group.submobjects) == [group_vmobject, group_mobject]
assert type(group_scene.get_group()) is manimlib.VGroup
assert group_scene.get_mobjects() == []


# Python-owned Scene collection helpers filter arbitrary iterables before
# routing through native add, and copy each native top-level placement.
class AmongMobject(Mobject):
    pass


among_scene = Scene()
among_front = AmongMobject().set_z_index(1)
among_back = Mobject().set_z_index(-1)
among_seen = []


def among_values():
    for value in (object(), among_front, "ignored", among_back, None):
        among_seen.append(value)
        yield value


assert among_scene.add_mobjects_among(among_values()) is among_scene
assert len(among_seen) == 5
assert among_scene.mobjects == [among_back, among_front]
assert among_scene.mobjects[1] is among_front

among_failed = Mobject()


def failing_among_values():
    yield among_failed
    raise RuntimeError("iterator failure")


among_before_failure = among_scene.get_mobjects()
try:
    among_scene.add_mobjects_among(failing_among_values())
except RuntimeError as error:
    assert str(error) == "iterator failure"
else:
    raise AssertionError("Scene.add_mobjects_among swallowed an iterator failure")
assert among_scene.get_mobjects() == among_before_failure
assert not among_failed._is_bound()

try:
    among_scene.add_mobjects_among(1)
except TypeError:
    pass
else:
    raise AssertionError("Scene.add_mobjects_among accepted a non-iterable")
assert among_scene.get_mobjects() == among_before_failure

foreign_among_scene = Scene()
foreign_among_mobject = Mobject()
foreign_among_scene.add(foreign_among_mobject)
try:
    among_scene.add_mobjects_among([foreign_among_mobject])
except bridge_errors.ForeignStageError:
    pass
else:
    raise AssertionError("Scene.add_mobjects_among accepted a foreign-stage Mobject")
assert among_scene.get_mobjects() == among_before_failure

# Family introspection preserves Reference preorder and path-wise duplicates,
# while Scene helpers expose fresh snapshots over live engine roots.
family_shared = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]
)
family_left = Mobject().add(family_shared)
family_right = Mobject().add(family_shared)
family_root = Mobject().add(family_left, family_right)
family_expected = [
    family_root,
    family_left,
    family_shared,
    family_right,
    family_shared,
]
family_actual = family_root.get_family()
assert len(family_actual) == len(family_expected)
assert all(actual is expected for actual, expected in zip(family_actual, family_expected))
assert family_root.get_family() is not family_actual
assert family_root.get_family(recurse=False) == [family_root]
assert family_root.get_family(recurse=False)[0] is family_root
family_with_points = family_root.family_members_with_points()
assert len(family_with_points) == 2
assert all(member is family_shared for member in family_with_points)
assert family_root.family_members_with_points() is not family_with_points

# Family spatial queries are ordinary Python compositions over Marionette's
# world-space points and retained bounding boxes. Shared descendants remain
# path-wise duplicates, matching the Reference compatibility surface.
assert str(inspect.signature(Mobject.get_all_points)) == "(self)"
assert str(inspect.signature(Mobject.get_bounding_box)) == "(self)"
assert str(inspect.signature(Mobject.compute_bounding_box)) == "(self)"
assert str(inspect.signature(Mobject.refresh_bounding_box)) == (
    "(self, recurse_down=False, recurse_up=True)"
)
assert str(inspect.signature(Mobject.are_points_touching)) == (
    "(self, points, buff=0)"
)
assert str(inspect.signature(Mobject.is_point_touching)) == (
    "(self, point, buff=0)"
)
assert str(inspect.signature(Mobject.is_touching)) == (
    "(self, mobject, buff=0.01)"
)
assert str(inspect.signature(Mobject.get_all_corners)) == "(self)"
assert str(inspect.signature(Mobject.get_center_of_mass)) == "(self)"
assert str(inspect.signature(Mobject.get_boundary_point)) == "(self, direction)"
assert str(inspect.signature(manimlib.DotCloud.compute_bounding_box)) == "(self)"

family_all_points = family_root.get_all_points()
shared_points = family_shared.get_points()
assert family_all_points.dtype == np.float32
assert family_all_points.shape == (2 * len(shared_points), 3)
assert np.array_equal(family_all_points[: len(shared_points)], shared_points)
assert np.array_equal(family_all_points[len(shared_points) :], shared_points)
assert np.allclose(
    family_root.compute_bounding_box(),
    [[0.0, 0.0, 0.0], [0.5, 0.0, 0.0], [1.0, 0.0, 0.0]],
)
assert family_root.refresh_bounding_box() is family_root
assert family_root.refresh_bounding_box(True, False) is family_root

empty_spatial = Mobject().add(Mobject())
assert empty_spatial.get_all_points().shape == (0, 3)
assert np.array_equal(empty_spatial.compute_bounding_box(), np.zeros((3, 3)))

mass_probe = Mobject()
mass_probe.resize(2)
mass_probe.set_field("point", 0, [-2.0, 0.0, 0.0])
mass_probe.set_field("point", 1, [1.0, 0.0, 0.0])
assert np.allclose(mass_probe.get_center_of_mass(), [-0.5, 0.0, 0.0])
assert np.allclose(mass_probe.get_boundary_point(manimlib.RIGHT), [1.0, 0.0, 0.0])
assert np.allclose(mass_probe.get_boundary_point(manimlib.LEFT), [-2.0, 0.0, 0.0])

touch_probe = Mobject()
touch_probe.resize(1)
touch_probe.set_field("point", 0, [0.0, 0.0, 0.0])
touch_points = np.array(
    [[0.0, 0.0, 0.0], [0.25, 0.0, 0.0], [-0.25, 0.0, 0.0]]
)
assert np.array_equal(
    touch_probe.are_points_touching(touch_points, buff=0.25),
    [True, True, True],
)
assert not touch_probe.are_points_touching(
    np.array([[0.2501, 0.0, 0.0]]), buff=0.25
)[0]
assert isinstance(touch_probe.is_point_touching([0.0, 0.0, 0.0]), np.bool_)
assert touch_probe.is_point_touching([0.25, 0.0, 0.0], buff=0.25)

other_touch_probe = Mobject()
other_touch_probe.resize(1)
other_touch_probe.set_field("point", 0, [1.0, 0.0, 0.0])
assert not touch_probe.is_touching(other_touch_probe, buff=0.999)
assert touch_probe.is_touching(other_touch_probe, buff=1.0)
assert isinstance(touch_probe.is_touching(other_touch_probe), bool)

dot_spatial = manimlib.DotCloud([[2.0, 3.0, 0.0]], radius=0.5)
dot_box = np.array(
    [[1.5, 2.5, -0.5], [2.0, 3.0, 0.0], [2.5, 3.5, 0.5]]
)
assert np.allclose(dot_spatial.compute_bounding_box(), dot_box)
assert np.allclose(dot_spatial.get_bounding_box(), dot_box)
assert np.allclose(dot_spatial.get_left(), [1.5, 3.0, 0.0])
assert dot_spatial.get_width() == 1.0
assert np.allclose(
    dot_spatial.get_all_corners(),
    [
        [1.5, 2.5, -0.5],
        [1.5, 2.5, 0.5],
        [2.5, 2.5, -0.5],
        [2.5, 2.5, 0.5],
        [1.5, 3.5, -0.5],
        [1.5, 3.5, 0.5],
        [2.5, 3.5, -0.5],
        [2.5, 3.5, 0.5],
    ],
)
dot_group = Mobject().add(dot_spatial)
assert np.allclose(dot_group.get_bounding_box(), dot_box)
dot_scene = Scene()
dot_scene.add(dot_group)
assert np.allclose(dot_group.get_bounding_box(), dot_box)

spatial_camera_frame = manimlib.CameraFrame(
    frame_shape=(10.0, 6.0), center_point=(1.0, 2.0, 0.0)
)
assert np.allclose(
    spatial_camera_frame.compute_bounding_box(),
    [[-4.0, -1.0, 0.0], [1.0, 2.0, 0.0], [6.0, 5.0, 0.0]],
)

# Camera is a real Lumen CameraConfig/Camera build rather than the schema's
# constructor refusal. Its Python frame keeps the Reference's mutable identity
# while native construction owns resolution, fps, color, sample, and light
# validation plus the initial aspect correction.
camera_module = importlib.import_module("manimlib.camera.camera")
assert camera_module.Camera.__bases__ == (object,)
native_camera = camera_module.Camera(
    frame_config={"frame_shape": (10.0, 8.0), "center_point": (1.0, 2.0, 0.0)},
    resolution=(640, 320),
    fps=24,
    background_color=manimlib.BLUE,
    background_opacity=0.75,
    max_allowable_norm=20.0,
    light_source_position=(-3.0, 4.0, 5.0),
    samples=4,
)
assert native_camera.get_pixel_shape() == (640, 320)
assert native_camera.get_pixel_width() == 640
assert native_camera.get_pixel_height() == 320
assert native_camera.get_aspect_ratio() == 2.0
assert np.allclose(native_camera.get_frame_shape(), (10.0, 5.0))
assert np.allclose(native_camera.get_frame_center(), (1.0, 2.0, 0.0))
assert native_camera.fps == 24
assert native_camera.samples == 4
assert native_camera.max_allowable_norm == 20.0
assert np.allclose(native_camera.light_source.get_center(), (-3.0, 4.0, 5.0))
native_camera.refresh_uniforms()
assert native_camera.uniforms["pixel_size"] == 10.0 / 640.0

failed_camera = camera_module.Camera.__new__(camera_module.Camera)
try:
    camera_module.Camera.__init__(failed_camera, background_image="image.png")
except NotImplementedError as error:
    assert str(error) == (
        "Camera() keyword(s) not yet routed to the native builder: "
        "background_image"
    )
else:
    raise AssertionError("Camera accepted the unrouted background image seam")
assert not hasattr(failed_camera, "_core")

# ThreeDCamera is Camera with the Reference four-sample default, still built
# by Lumen's native CameraConfig path.
assert camera_module.ThreeDCamera.__bases__ == (camera_module.Camera,)
assert str(inspect.signature(camera_module.ThreeDCamera)) == (
    "(samples=4, **kwargs)"
)
default_three_d_camera = camera_module.ThreeDCamera()
assert default_three_d_camera.samples == 4
assert isinstance(default_three_d_camera, camera_module.Camera)
explicit_three_d_camera = camera_module.ThreeDCamera(
    samples=8,
    resolution=(640, 320),
)
assert explicit_three_d_camera.samples == 8
assert explicit_three_d_camera.get_pixel_shape() == (640, 320)
failed_three_d_camera = camera_module.ThreeDCamera.__new__(
    camera_module.ThreeDCamera
)
try:
    camera_module.ThreeDCamera.__init__(
        failed_three_d_camera,
        background_image="image.png",
    )
except NotImplementedError as error:
    assert str(error) == (
        "Camera() keyword(s) not yet routed to the native builder: "
        "background_image"
    )
else:
    raise AssertionError("ThreeDCamera accepted the unrouted background image seam")
assert not hasattr(failed_three_d_camera, "_core")

scene_module = importlib.import_module("manimlib.scene.scene")
assert scene_module.Scene.random_seed == 0
assert scene_module.Scene.default_frame_orientation == (0, 0)


class OrientedBaseScene(scene_module.Scene):
    default_frame_orientation = (25, 40, 10)


oriented_base_scene = OrientedBaseScene()
assert np.allclose(
    oriented_base_scene.frame.get_euler_angles(), np.deg2rad([25.0, 40.0, 10.0]),
)
oriented_base_scene.frame.reorient(0, 0, 0)
oriented_base_scene.frame.to_default_state()
assert np.allclose(
    oriented_base_scene.frame.get_euler_angles(), np.deg2rad([25.0, 40.0, 10.0]),
)
assert scene_module.ThreeDScene.__bases__ == (scene_module.Scene,)
assert scene_module.ThreeDScene.samples == 4
assert scene_module.ThreeDScene.default_frame_orientation == (-30, 70)
assert scene_module.ThreeDScene.always_depth_test is True
assert str(inspect.signature(scene_module.ThreeDScene.add)) == (
    "(self, *mobjects, set_depth_test=True, perp_stroke=True)"
)
three_d_scene = scene_module.ThreeDScene()
assert three_d_scene.camera.samples == 4
assert np.allclose(
    three_d_scene.frame.get_euler_angles()[:2],
    np.deg2rad([-30.0, 70.0]),
)

class FlatThreeDScene(scene_module.ThreeDScene):
    default_frame_orientation = (0, 0)


flat_three_d_scene = FlatThreeDScene()
assert flat_three_d_scene.default_frame_orientation == (0, 0)
assert np.isclose(flat_three_d_scene.frame.get_theta(), 0.0)
assert np.isclose(flat_three_d_scene.frame.get_phi(), 0.0)
depth_circle = manimlib.Circle()
three_d_scene.add(depth_circle)
assert depth_circle.uniforms["depth_test"] is True
assert depth_circle.get_flat_stroke() is False
assert depth_circle._is_bound()
fixed_circle = manimlib.Circle()
fixed_circle.fix_in_frame()
three_d_scene.add(fixed_circle)
assert fixed_circle.uniforms["depth_test"] is False
skipped_circle = manimlib.Circle()
three_d_scene.add(skipped_circle, set_depth_test=False, perp_stroke=False)
assert skipped_circle.uniforms["depth_test"] is False

assert scene_module.SceneState.__bases__ == (object,)
assert str(inspect.signature(scene_module.SceneState)) == (
    "(scene, ignore=None)"
)
assert str(inspect.signature(scene_module.Scene.get_state)) == "(self)"
assert str(inspect.signature(scene_module.Scene.save_state)) == "(self)"
assert str(inspect.signature(scene_module.Scene.restore_state)) == (
    "(self, scene_state)"
)
saved_scene = Scene()
assert saved_scene.undo_stack == []
assert saved_scene.save_state() is saved_scene
assert len(saved_scene.undo_stack) == 1
assert isinstance(saved_scene.undo_stack[0], scene_module.SceneState)

# fm-5wq.4: Scene.undo pops the native SceneState undo_stack back through
# restore_state, mirroring the popped state onto redo_stack; an empty
# stack is a no-op and InteractiveScene keeps its overlay after undo.
assert str(inspect.signature(scene_module.Scene.undo)) == "(self)"
undo_scene = Scene()
assert undo_scene.undo_stack == []
assert undo_scene.redo_stack == []
assert undo_scene.undo() is None
assert undo_scene.undo_stack == []
undo_circle = manimlib.Circle()
undo_scene.add(undo_circle)
undo_saved_center = undo_circle.get_center().copy()
undo_scene.save_state()
undo_circle.shift([2.0, 1.0, 0.0])
assert undo_scene.undo() is None
assert np.allclose(undo_circle.get_center(), undo_saved_center)
assert undo_scene.undo_stack == []
assert len(undo_scene.redo_stack) == 1
assert isinstance(undo_scene.redo_stack[0], scene_module.SceneState)
undo_overlay_scene = InteractiveScene()
undo_overlay_scene.setup()
undo_overlay_circle = manimlib.Circle()
undo_overlay_scene.add(undo_overlay_circle)
undo_overlay_scene.save_state()
undo_overlay_circle.shift([1.0, 0.0, 0.0])
assert undo_overlay_scene.undo() is None
assert np.allclose(undo_overlay_circle.get_center(), [0.0, 0.0, 0.0])
assert (
    undo_overlay_scene.selection_highlight in undo_overlay_scene.mobjects
)

# fm-5wq.4: Scene's host-interaction defaults remain available without
# pyglet. Constructor overrides are live, save_state evicts the oldest
# checkpoint at the Reference cap, and Atlas's native CameraFrame owns zoom.
assert scene_module.Scene.pan_sensitivity == 0.5
assert scene_module.Scene.scroll_sensitivity == 20
assert scene_module.Scene.drag_to_pan is True
assert scene_module.Scene.max_num_saved_states == 50
hostless_scene = Scene()
assert hostless_scene.window is None
assert hostless_scene.hold_on_wait is False
assert Scene(presenter_mode=True).hold_on_wait is True
host_window = object()
configured_scene = Scene(
    window=host_window,
    pan_sensitivity=0.25,
    scroll_sensitivity=10,
    drag_to_pan=False,
    max_num_saved_states=2,
)
assert configured_scene.window is host_window
assert configured_scene.pan_sensitivity == 0.25
assert configured_scene.scroll_sensitivity == 10.0
assert configured_scene.drag_to_pan is False
configured_scene_dot = manimlib.Dot()
configured_scene.add(configured_scene_dot)
configured_scene.save_state()
oldest_state = configured_scene.undo_stack[0]
configured_scene_dot.shift(manimlib.RIGHT)
configured_scene.save_state()
configured_scene_dot.shift(manimlib.UP)
configured_scene.save_state()
assert len(configured_scene.undo_stack) == 2
assert oldest_state not in configured_scene.undo_stack
assert str(inspect.signature(scene_module.Scene.on_mouse_motion)) == (
    "(self, point, d_point)"
)
assert str(inspect.signature(scene_module.Scene.on_mouse_scroll)) == (
    "(self, point, offset, x_pixel_offset, y_pixel_offset)"
)
hostless_frame_center = hostless_scene.frame.get_center().copy()
hostless_scene.on_mouse_motion(manimlib.RIGHT, manimlib.UP)
assert np.allclose(hostless_scene.mouse_point.get_center(), manimlib.RIGHT)
assert np.allclose(hostless_scene.frame.get_center(), hostless_frame_center)
scroll_width = hostless_scene.frame.get_width()
scroll_factor = 1.0 - (
    hostless_scene.scroll_sensitivity / hostless_scene.camera.get_pixel_height()
)
hostless_scene.on_mouse_scroll(manimlib.ORIGIN, (0, 1), 0, 1)
assert np.isclose(hostless_scene.frame.get_width(), scroll_width * scroll_factor)
event_listener_module = importlib.import_module(
    "manimlib.event_handler.event_listner"
)
global_dispatcher = importlib.import_module(
    "manimlib.event_handler"
).EVENT_DISPATCHER
stop_scroll_mobject = manimlib.Dot()
stop_scroll_listener = event_listener_module.EventListener(
    stop_scroll_mobject,
    manimlib.EventType.MouseScrollEvent,
    lambda _mobject, _event_data: False,
)
global_dispatcher.add_listener(stop_scroll_listener)
stopped_scroll_width = hostless_scene.frame.get_width()
try:
    hostless_scene.on_mouse_motion(manimlib.ORIGIN, manimlib.ORIGIN)
    hostless_scene.on_mouse_scroll(manimlib.ORIGIN, (0, 1), 0, 1)
finally:
    global_dispatcher.remove_listner(stop_scroll_listener)
assert np.isclose(hostless_scene.frame.get_width(), stopped_scroll_width)

# fm-5wq.4: Scene.redo mirrors undo — it pops redo_stack back through
# restore_state while pushing the live state onto undo_stack; an empty
# redo_stack is a no-op.
assert str(inspect.signature(scene_module.Scene.redo)) == "(self)"
redo_scene = Scene()
assert redo_scene.redo() is None
assert redo_scene.undo_stack == []
assert redo_scene.redo_stack == []
redo_circle = manimlib.Circle()
redo_scene.add(redo_circle)
redo_saved_center = redo_circle.get_center().copy()
redo_scene.save_state()
redo_circle.shift([2.0, 1.0, 0.0])
redo_moved_center = redo_circle.get_center().copy()
assert redo_scene.undo() is None
assert np.allclose(redo_circle.get_center(), redo_saved_center)
assert redo_scene.redo() is None
assert np.allclose(redo_circle.get_center(), redo_moved_center)
assert len(redo_scene.undo_stack) == 1
assert redo_scene.redo_stack == []

# fm-5wq.4: Scene pointer/scroll/key events are host-free. The pyglet
# Window is not required to move the native Points, scale the camera
# frame, or dispatch undo/redo/hold_on_wait/quit. pan_3d/pan stay skipped
# until Studio binds a key-state adapter.
assert str(inspect.signature(scene_module.Scene.get_window)) == "(self)"
assert str(inspect.signature(scene_module.Scene.on_mouse_motion)) == (
    "(self, point, d_point)"
)
assert str(inspect.signature(scene_module.Scene.on_mouse_drag)) == (
    "(self, point, d_point, buttons, modifiers)"
)
assert str(inspect.signature(scene_module.Scene.on_mouse_press)) == (
    "(self, point, button, mods)"
)
assert str(inspect.signature(scene_module.Scene.on_mouse_release)) == (
    "(self, point, button, mods)"
)
assert str(inspect.signature(scene_module.Scene.on_mouse_scroll)) == (
    "(self, point, offset, x_pixel_offset, y_pixel_offset)"
)
assert str(inspect.signature(scene_module.Scene.on_key_press)) == (
    "(self, symbol, modifiers)"
)
assert str(inspect.signature(scene_module.Scene.on_key_release)) == (
    "(self, symbol, modifiers)"
)
assert str(inspect.signature(scene_module.Scene.on_resize)) == (
    "(self, width, height)"
)
assert str(inspect.signature(scene_module.Scene.on_show)) == "(self)"
assert str(inspect.signature(scene_module.Scene.on_hide)) == "(self)"
assert str(inspect.signature(scene_module.Scene.on_close)) == "(self)"
assert scene_module.Scene.pan_sensitivity == 0.5
assert scene_module.Scene.scroll_sensitivity == 20
assert scene_module.Scene.drag_to_pan is True
event_scene = Scene()
assert event_scene.get_window() is None
assert event_scene.hold_on_wait is False
assert event_scene.quit_interaction is False
assert event_scene.on_mouse_motion([1.0, 2.0, 0.0], [0.1, 0.0, 0.0]) is None
assert np.allclose(event_scene.mouse_point.get_center(), [1.0, 2.0, 0.0])
assert np.allclose(event_scene.mouse_drag_point.get_center(), [0.0, 0.0, 0.0])
assert event_scene.on_mouse_drag([3.0, 4.0, 0.0], [0.0, 0.0, 0.0], 1, 0) is None
assert np.allclose(event_scene.mouse_drag_point.get_center(), [3.0, 4.0, 0.0])
assert np.allclose(event_scene.mouse_point.get_center(), [1.0, 2.0, 0.0])
frame_center = event_scene.frame.get_center().copy()
assert event_scene.on_mouse_drag(
    [3.0, 4.0, 0.0], [0.5, -0.25, 0.0], 1, 0
) is None
assert np.allclose(
    event_scene.frame.get_center(),
    frame_center - np.array([0.5, -0.25, 0.0]),
)
assert event_scene.on_mouse_press([-1.0, 0.5, 0.0], 1, 0) is None
assert np.allclose(event_scene.mouse_drag_point.get_center(), [-1.0, 0.5, 0.0])
assert event_scene.on_mouse_release(manimlib.ORIGIN, 0, 0) is None
scroll_width = event_scene.frame.get_width()
pixel_height = event_scene.camera.get_pixel_height()
assert pixel_height > 0
assert event_scene.on_mouse_scroll(
    manimlib.ORIGIN,
    [0.0, 1.0, 0.0],
    0.0,
    0.01 * pixel_height,
) is None
assert np.isclose(
    event_scene.frame.get_width(),
    scroll_width * (1.0 - event_scene.scroll_sensitivity * 0.01),
)
event_scene.hold_on_wait = True
assert event_scene.on_key_press(ord(" "), 0) is None
assert event_scene.hold_on_wait is False
event_scene.hold_on_wait = True
assert event_scene.on_key_press(0xFF53, 0) is None  # pyglet RIGHT
assert event_scene.hold_on_wait is False
assert event_scene.on_key_press(ord("q"), 2) is None  # pyglet MOD_CTRL
assert event_scene.quit_interaction is True
assert event_scene.on_key_release(ord("q"), 2) is None
# Clear the ctrl-q flag so later windowless pins (is_window_closing) see
# the default; the True pin above already proved the chord sets it.
event_scene.quit_interaction = False
assert event_scene.on_resize(640, 480) is None
assert event_scene.on_show() is None
assert event_scene.on_hide() is None
assert event_scene.on_close() is None

undo_key_scene = Scene()
undo_key_circle = manimlib.Circle()
undo_key_scene.add(undo_key_circle)
undo_key_saved = undo_key_circle.get_center().copy()
undo_key_scene.save_state()
undo_key_circle.shift([1.0, 0.0, 0.0])
undo_key_moved = undo_key_circle.get_center().copy()
assert undo_key_scene.on_key_press(ord("z"), 2) is None  # CTRL-z
assert np.allclose(undo_key_circle.get_center(), undo_key_saved)
assert undo_key_scene.on_key_press(ord("z"), 2 | 1) is None  # CTRL|SHIFT-z
assert np.allclose(undo_key_circle.get_center(), undo_key_moved)

# fm-5wq.4: reset-r plays a native camera-frame lerp back to the default
# state; focus is a no-op without a host window; set_background_color
# writes Camera.background_rgba through the one color model.
assert str(inspect.signature(scene_module.Scene.focus)) == "(self)"
assert str(inspect.signature(scene_module.Scene.set_background_color)) == (
    "(self, background_color, background_opacity=1)"
)
assert event_scene.focus() is None
assert event_scene.set_background_color(manimlib.RED, 0.5) is None
assert np.allclose(
    event_scene.camera.background_rgba,
    [*manimlib.color_to_rgb(manimlib.RED), 0.5],
)
reset_scene = Scene()
reset_scene.frame.shift([2.0, 1.0, 0.0])
assert not np.allclose(reset_scene.frame.get_center(), [0.0, 0.0, 0.0])
assert reset_scene.on_key_press(ord("r"), 0) is None
assert np.allclose(reset_scene.frame.get_center(), [0.0, 0.0, 0.0], atol=1e-5)

# fm-5wq.4: Scene.embed/interact are no-ops without a pyglet window;
# is_window_closing observes quit_interaction without requiring a host
# window; set_floor_plane routes xy/xz onto native euler axes.
assert str(inspect.signature(scene_module.Scene.interact)) == "(self)"
assert str(inspect.signature(scene_module.Scene.embed)) == (
    "(self, close_scene_on_exit=True, show_animation_progress=False)"
)
assert str(inspect.signature(scene_module.Scene.is_window_closing)) == "(self)"
assert str(inspect.signature(scene_module.Scene.set_floor_plane)) == (
    "(self, plane='xy')"
)
assert event_scene.interact() is None
assert event_scene.embed() is None
assert event_scene.is_window_closing() is False
event_scene.quit_interaction = True
assert event_scene.is_window_closing() is True
event_scene.quit_interaction = False
assert event_scene.set_floor_plane() is None
assert event_scene.frame._core.euler_axes() == "zxz"
assert event_scene.set_floor_plane("xz") is None
assert event_scene.frame._core.euler_axes() == "zxy"
try:
    event_scene.set_floor_plane("yz")
except Exception as error:
    assert str(error) == "Only `xz` and `xy` are valid floor planes"
else:
    raise AssertionError("set_floor_plane accepted a non-xy/xz plane")
windowed_scene = Scene(window=object())
try:
    windowed_scene.interact()
except bridge_errors.CapabilityError as error:
    assert "Studio owns interactive windows" in str(error)
else:
    raise AssertionError("interact entered a host window loop")
try:
    windowed_scene.embed()
except bridge_errors.CapabilityError as error:
    assert "Studio owns interactive windows" in str(error)
else:
    raise AssertionError("embed launched a host IPython session")

# fm-5wq.4: skip/hold flags are host-free. force_skipping/stop_skipping/
# revert_to_original_skipping_status flip skip_animations; hold_loop is a
# no-op unless presenter-mode already armed a wait that needs a window.
assert str(inspect.signature(scene_module.Scene.force_skipping)) == "(self)"
assert str(inspect.signature(scene_module.Scene.stop_skipping)) == "(self)"
assert str(inspect.signature(scene_module.Scene.revert_to_original_skipping_status)) == (
    "(self)"
)
assert str(inspect.signature(scene_module.Scene.hold_loop)) == "(self)"
assert str(inspect.signature(scene_module.Scene.update_skipping_status)) == (
    "(self)"
)
skip_scene = Scene()
assert skip_scene.skip_animations is False
assert skip_scene.force_skipping() is skip_scene
assert skip_scene.skip_animations is True
assert skip_scene.stop_skipping() is None
assert skip_scene.skip_animations is False
assert skip_scene.revert_to_original_skipping_status() is skip_scene
assert skip_scene.skip_animations is False
forced_skip = Scene(skip_animations=True)
assert forced_skip.skip_animations is True
assert forced_skip.original_skipping_status is True
assert Scene(start_at_animation_number=2).skip_animations is True

# fm-5wq.4: preview_while_skipping is a constructor flag defaulting to
# True (stored per-instance from kwargs — no class attribute), and it is
# bool()-cast like the other Scene flags, so truthy strings become True.
assert skip_scene.preview_while_skipping is True
assert Scene().preview_while_skipping is True
assert Scene(preview_while_skipping=False).preview_while_skipping is False
assert Scene(preview_while_skipping="yes").preview_while_skipping is True
assert Scene(preview_while_skipping=0).preview_while_skipping is False
assert Scene(preview_while_skipping=1).preview_while_skipping is True

# fm-5wq.4: Scene.__init__ captures its raw constructor arguments verbatim
# as args/kwargs (the Reference reuses them for scene re-instantiation).
assert Scene().args == ()
assert isinstance(Scene().kwargs, dict)
capture_scene = Scene(1, 2, skip_animations=True)
assert capture_scene.args == (1, 2)
assert capture_scene.kwargs["skip_animations"] is True

# fm-5wq.4: start_at_animation_number skips until post_play's num_plays
# increment reaches it; the next pre_play's update_skipping_status then
# releases the skip. The Reference captures original_skipping_status
# BEFORE the start-at forcing (False here), which is what lets
# stop_skipping fire at the release point — no second counter exists.
start_release_scene = Scene(start_at_animation_number=2)
assert start_release_scene.skip_animations is True
assert start_release_scene.original_skipping_status is False
assert start_release_scene.num_plays == 0
assert start_release_scene.pre_play() is None
assert start_release_scene.skip_animations is True
assert start_release_scene.post_play() is None
assert start_release_scene.num_plays == 1
start_release_scene.pre_play()
assert start_release_scene.skip_animations is True
start_release_scene.post_play()
assert start_release_scene.num_plays == 2
start_release_scene.pre_play()
assert start_release_scene.skip_animations is False
assert np.isclose(
    start_release_scene.skip_time, start_release_scene.get_time()
)
start_release_scene.post_play()
assert start_release_scene.num_plays == 3
assert skip_scene.hold_on_wait is False
assert skip_scene.hold_loop() is None
assert skip_scene.hold_on_wait is True
try:
    Scene(presenter_mode=True).hold_loop()
except bridge_errors.CapabilityError as error:
    assert "hold_loop" in str(error)
    assert "Studio owns interactive windows" in str(error)
else:
    raise AssertionError("hold_loop spun without a host window")
end_skip = Scene(end_at_animation_number=0)
end_skip.num_plays = 0
try:
    end_skip.update_skipping_status()
except scene_module.EndScene:
    pass
else:
    raise AssertionError("update_skipping_status missed end_at_animation_number")

# fm-5wq.4: show_animation_progress is a constructor flag defaulting to
# False, stored per-instance with the same bool() coercion as Scene's other
# host-free flags.
assert scene_module.Scene.show_animation_progress is False
assert Scene().show_animation_progress is False
assert Scene(show_animation_progress=True).show_animation_progress is True
assert Scene(show_animation_progress="yes").show_animation_progress is True
assert Scene(show_animation_progress="").show_animation_progress is False
assert Scene(show_animation_progress=None).show_animation_progress is False

# fm-5wq.4: temp_skip/temp_progress_bar are host-free context managers;
# temp_record names the missing file-writer insert seam; temp_config_change
# stacks the skip/progress pair without a pyglet window.
assert str(inspect.signature(scene_module.Scene.temp_skip)) == "(self)"
assert str(inspect.signature(scene_module.Scene.temp_progress_bar)) == "(self)"
assert str(inspect.signature(scene_module.Scene.temp_record)) == "(self)"
assert str(inspect.signature(scene_module.Scene.temp_config_change)) == (
    "(self, skip=False, record=False, progress_bar=False)"
)
temp_scene = Scene()
assert temp_scene.skip_animations is False
assert temp_scene.show_animation_progress is False
with temp_scene.temp_skip():
    assert temp_scene.skip_animations is True
assert temp_scene.skip_animations is False
already_skip = Scene(skip_animations=True)
with already_skip.temp_skip():
    assert already_skip.skip_animations is True
    already_skip.skip_animations = False
assert already_skip.skip_animations is True
with temp_scene.temp_progress_bar():
    assert temp_scene.show_animation_progress is True
assert temp_scene.show_animation_progress is False
with temp_scene.temp_config_change(skip=True, progress_bar=True):
    assert temp_scene.skip_animations is True
    assert temp_scene.show_animation_progress is True
assert temp_scene.skip_animations is False
assert temp_scene.show_animation_progress is False
try:
    with temp_scene.temp_record():
        pass
except bridge_errors.CapabilityError as error:
    assert "file-writer" in str(error)
else:
    raise AssertionError("temp_record faked a file-writer insert")
try:
    temp_scene.temp_config_change(skip=True, record=True)
except bridge_errors.CapabilityError as error:
    assert "file-writer" in str(error)
else:
    raise AssertionError("temp_config_change(record=True) faked a recording")
assert temp_scene.skip_animations is False

# fm-5wq.4: host-free frame helpers bump the engine clock and Python
# updaters without a pyglet window; capture/show/emit name the missing
# Lumen/file-writer/host-viewer seams.
assert str(inspect.signature(scene_module.Scene.increment_time)) == "(self, dt)"
assert str(inspect.signature(scene_module.Scene.update_mobjects)) == "(self, dt)"
assert str(inspect.signature(scene_module.Scene.should_update_mobjects)) == (
    "(self)"
)
assert str(inspect.signature(scene_module.Scene.update_frame)) == (
    "(self, dt=0, force_draw=False)"
)
assert str(inspect.signature(scene_module.Scene.emit_frame)) == "(self)"
assert str(inspect.signature(scene_module.Scene.get_image)) == "(self)"
assert str(inspect.signature(scene_module.Scene.show)) == "(self)"
frame_scene = Scene()
assert frame_scene.always_update_mobjects is False
assert frame_scene.should_update_mobjects() is False
assert frame_scene.get_time() == 0.0
assert frame_scene.increment_time(0.0) is None
assert frame_scene.get_time() == 0.0
assert frame_scene.increment_time(0.5) is None
assert np.isclose(frame_scene.get_time(), 0.5)
frame_ticks = []
frame_probe = manimlib.Circle()
frame_probe.add_updater(lambda mob, dt: frame_ticks.append(dt))
frame_scene.add(frame_probe)
assert frame_scene.should_update_mobjects() is True
assert frame_scene.increment_time(0.25) is None
assert np.isclose(frame_scene.get_time(), 0.75)
assert frame_ticks == [0.0]
assert frame_scene.update_mobjects(0.25) is None
assert frame_ticks == [0.0, 0.25]
assert frame_scene.update_frame(0.125) is None
assert np.isclose(frame_scene.get_time(), 0.875)
assert frame_ticks == [0.0, 0.25, 0.125]

# fm-5wq.4: always_update_mobjects=True forces should_update_mobjects with
# zero mobjects and with updater-less mobjects; increment_time still never
# runs updaters while update_mobjects still does. The constructor
# bool()-casts the flag (same style as show_animation_progress /
# leave_progress_bars), so a truthy string pins to True and a falsy one
# to False rather than raising.
always_scene = Scene(always_update_mobjects=True)
assert always_scene.always_update_mobjects is True
assert always_scene.should_update_mobjects() is True
always_plain = manimlib.Circle()
always_scene.add(always_plain)
assert list(always_plain.updaters) == []
assert always_scene.should_update_mobjects() is True
assert always_scene.increment_time(0.5) is None
assert np.isclose(always_scene.get_time(), 0.5)
always_ticks = []
always_probe = manimlib.Circle()
always_probe.add_updater(lambda mob, dt: always_ticks.append(dt))
always_scene.add(always_probe)
assert always_scene.increment_time(0.25) is None
assert always_ticks == [0.0]
assert always_scene.update_mobjects(0.25) is None
assert always_ticks == [0.0, 0.25]
coerced_always_scene = Scene(always_update_mobjects="yes")
assert coerced_always_scene.always_update_mobjects is True
assert coerced_always_scene.should_update_mobjects() is True
assert Scene(always_update_mobjects="").always_update_mobjects is False

skip_frame = Scene(skip_animations=True)
assert skip_frame.update_frame(0.25) is None
assert np.isclose(skip_frame.get_time(), 0.25)
assert skip_frame.emit_frame() is None
try:
    frame_scene.emit_frame()
except bridge_errors.CapabilityError as error:
    assert "file writer" in str(error).lower()
else:
    raise AssertionError("emit_frame faked a file-writer frame")
try:
    frame_scene.get_image()
except bridge_errors.CapabilityError as error:
    assert "capture" in str(error).lower()
else:
    raise AssertionError("get_image faked a Lumen capture")
try:
    frame_scene.show()
except bridge_errors.CapabilityError as error:
    assert "viewer" in str(error).lower()
else:
    raise AssertionError("show faked a host image viewer")
windowed_frame = Scene(window=object())
try:
    windowed_frame.update_frame(force_draw=True)
except bridge_errors.CapabilityError as error:
    assert "Studio owns interactive windows" in str(error)
else:
    raise AssertionError("update_frame entered a host capture loop")

# fm-5wq.4: time-progression helpers are host-free numpy grids (no tqdm);
# skip collapses to a single terminal sample; pre_play/post_play bump
# num_plays without a file writer.
assert str(inspect.signature(scene_module.Scene.get_time_progression)) == (
    "(self, run_time, n_iterations=None, desc='', override_skip_animations=False)"
)
assert str(inspect.signature(scene_module.Scene.get_run_time)) == (
    "(self, animations)"
)
assert str(inspect.signature(scene_module.Scene.get_animation_time_progression)) == (
    "(self, animations)"
)
assert str(inspect.signature(scene_module.Scene.get_wait_time_progression)) == (
    "(self, duration, stop_condition=None)"
)
assert str(inspect.signature(scene_module.Scene.pre_play)) == "(self)"
assert str(inspect.signature(scene_module.Scene.post_play)) == "(self)"
progress_scene = Scene()
times = progress_scene.get_time_progression(1.0)
assert isinstance(times, np.ndarray)
assert np.isclose(times[0], 1.0 / progress_scene.camera.fps)
assert np.isclose(times[-1], 1.0)
skip_progress = Scene(skip_animations=True)
assert skip_progress.get_time_progression(2.0) == [2.0]
assert np.allclose(
    skip_progress.get_wait_time_progression(0.5, stop_condition=lambda: False),
    progress_scene.get_time_progression(0.5),
)
play_anim = manimlib.Animation(manimlib.Circle(), run_time=1.5)
assert np.isclose(play_anim.get_run_time(), 1.5)
assert play_anim.set_run_time(2.0) is play_anim
assert np.isclose(play_anim.get_run_time(), 2.0)
assert np.isclose(progress_scene.get_run_time([play_anim]), 2.0)
spanned = manimlib.Animation(manimlib.Circle(), run_time=0.5, time_span=(0.0, 1.25))
assert np.isclose(spanned.get_run_time(), 1.25)
assert progress_scene.num_plays == 0
assert progress_scene.pre_play() is None
assert progress_scene.num_plays == 0
assert progress_scene.post_play() is None
assert progress_scene.num_plays == 1
try:
    Scene(presenter_mode=True).pre_play()
except bridge_errors.CapabilityError as error:
    assert "hold_loop" in str(error)
else:
    raise AssertionError("presenter pre_play spun hold_loop")

# fm-5wq.4: begin_animations adopts off-scene mobjects; finish_animations
# interpolates to the final alpha and honors remover cleanup; skip
# progress_through_animations does not require a file writer.
assert str(inspect.signature(scene_module.Scene.begin_animations)) == (
    "(self, animations)"
)
assert str(inspect.signature(scene_module.Scene.progress_through_animations)) == (
    "(self, animations)"
)
assert str(inspect.signature(scene_module.Scene.finish_animations)) == (
    "(self, animations)"
)
begin_scene = Scene()
begin_circle = manimlib.Circle()
begin_anim = manimlib.Animation(begin_circle, run_time=0.5)
assert begin_circle not in begin_scene.mobjects
assert begin_scene.begin_animations([begin_anim]) is None
assert begin_circle in begin_scene.mobjects
remover_scene = Scene()
remover_circle = manimlib.Circle()
remover_scene.add(remover_circle)
remover_anim = manimlib.Animation(remover_circle, run_time=0.25, remover=True)
remover_scene.begin_animations([remover_anim])
assert remover_scene.finish_animations([remover_anim]) is None
assert remover_circle not in remover_scene.mobjects
skip_progress_scene = Scene(skip_animations=True)
skip_progress_circle = manimlib.Circle()
skip_progress_anim = manimlib.Animation(skip_progress_circle, run_time=0.5)
skip_progress_scene.begin_animations([skip_progress_anim])
assert skip_progress_scene.progress_through_animations(
    [skip_progress_anim]
) is None
assert np.isclose(skip_progress_scene.get_time(), 0.5)
skip_progress_scene.finish_animations([skip_progress_anim])

state_scene = Scene()
state_square = manimlib.Square()
state_circle = manimlib.Circle()
state_scene.add(state_square, state_circle)
origin = state_square.get_center().copy()
captured = state_scene.get_state()
assert isinstance(captured, scene_module.SceneState)
assert captured._checkpoint == state_scene._checkpoint_bytes()
assert scene_module.SceneState(state_scene, ignore=[])._checkpoint is not None
assert list(captured.mobjects_to_copies) == [state_square, state_circle]
assert captured.n_changes(captured) == 0
state_square.shift(manimlib.RIGHT)
assert captured.n_changes(captured) == 1
ignored = scene_module.SceneState(state_scene, ignore=[state_circle])
assert list(ignored.mobjects_to_copies) == [state_square]
state_scene.restore_state(captured)
assert np.allclose(state_square.get_center(), origin)
restored_state_roots = state_scene.get_mobjects()
print(
    "SceneState restored roots:",
    len(restored_state_roots),
    [
        (
            id(root),
            type(root).__name__,
            root is state_square,
            root is state_circle,
        )
        for root in restored_state_roots
    ],
    "expected:",
    [(id(state_square), "Square"), (id(state_circle), "Circle")],
)
assert restored_state_roots == [state_square, state_circle]
assert state_scene._checkpoint_bytes() == captured._checkpoint
failed_state = scene_module.SceneState.__new__(scene_module.SceneState)
try:
    scene_module.SceneState.__init__(failed_state, manimlib.Square())
except TypeError as error:
    assert str(error) == "SceneState scene must be a Scene"
else:
    raise AssertionError("SceneState accepted a non-Scene")

embed_module = importlib.import_module("manimlib.scene.scene_embed")
assert embed_module.CheckpointManager.__bases__ == (object,)
assert str(inspect.signature(embed_module.CheckpointManager)) == "()"
assert str(inspect.signature(embed_module.CheckpointManager.get_leading_comment)) == (
    "(code_string)"
)
checkpoint_manager = embed_module.CheckpointManager()
assert checkpoint_manager.get_leading_comment("# start\nplay()") == "# start"
assert checkpoint_manager.get_leading_comment("play()") == ""
checkpoint_scene = Scene()
checkpoint_square = manimlib.Square()
checkpoint_scene.add(checkpoint_square)
checkpoint_origin = checkpoint_square.get_center().copy()
checkpoint_manager.handle_checkpoint_key(checkpoint_scene, "# start")
checkpoint_square.shift(manimlib.RIGHT)
checkpoint_manager.handle_checkpoint_key(checkpoint_scene, "# later")
assert "# later" in checkpoint_manager.checkpoint_states
checkpoint_manager.handle_checkpoint_key(checkpoint_scene, "# start")
assert np.allclose(checkpoint_square.get_center(), checkpoint_origin)
assert "# later" not in checkpoint_manager.checkpoint_states
checkpoint_manager.clear_checkpoints()
assert checkpoint_manager.checkpoint_states == {}
try:
    checkpoint_manager.checkpoint_paste(None, checkpoint_scene)
except bridge_errors.CapabilityError as error:
    assert "active host SceneConsole" in str(error)
else:
    raise AssertionError("checkpoint_paste accepted an unowned clipboard/session")

assert embed_module.InteractiveSceneEmbed.__bases__ == (object,)
assert str(inspect.signature(embed_module.InteractiveSceneEmbed)) == "(scene)"
embed_scene = Scene()
embedded = embed_module.InteractiveSceneEmbed(embed_scene)
assert embedded.scene is embed_scene
assert isinstance(embedded.checkpoint_manager, embed_module.CheckpointManager)
assert embedded.get_shortcuts()["add"] == embed_scene.add
assert embedded.validate_syntax("/no/such/file.py") is False
try:
    embedded.enable_gui()
except Exception as error:
    assert "Studio owns interactive windows" in str(error)
else:
    raise AssertionError("enable_gui did not refuse the pyglet GUI hook")
for embed_refusal, fragment in (
    ("ensure_frame_update_post_cell", "create an IPython shell"),
    ("ensure_flash_on_error", "Studio owns interactive windows"),
    ("auto_reload", "embedded scene session"),
    ("reload_scene", "IPython embed loop"),
):
    try:
        getattr(embedded, embed_refusal)()
    except bridge_errors.CapabilityError as error:
        assert fragment in str(error)
    else:
        raise AssertionError(f"{embed_refusal} did not refuse the host embed")
failed_embed = embed_module.InteractiveSceneEmbed.__new__(
    embed_module.InteractiveSceneEmbed
)
try:
    embed_module.InteractiveSceneEmbed.__init__(failed_embed, manimlib.Square())
except TypeError as error:
    assert str(error) == "InteractiveSceneEmbed scene must be a Scene"
else:
    raise AssertionError("InteractiveSceneEmbed accepted a non-Scene")

live_box = manimlib.Square()
live_box_before = live_box.get_bounding_box().copy()
live_box.get_points()[:, 0] += 2.0
assert np.allclose(
    live_box.get_bounding_box(), live_box_before + [2.0, 0.0, 0.0]
)

family_scene = Scene()
family_isolated = Mobject()
family_duplicate = Mobject()
family_scene.add(
    family_root,
    family_shared,
    family_isolated,
    family_duplicate,
    family_duplicate,
)
family_roots_before = family_scene.get_mobjects()
top_level = family_scene.get_top_level_mobjects()
assert len(top_level) == 2
assert top_level[0] is family_root
assert top_level[1] is family_isolated
assert family_scene.get_top_level_mobjects() is not top_level

scene_family_expected = [
    family_root,
    family_left,
    family_shared,
    family_right,
    family_shared,
    family_shared,
    family_isolated,
    family_duplicate,
    family_duplicate,
]
scene_family_actual = family_scene.get_mobject_family_members()
assert len(scene_family_actual) == len(scene_family_expected)
assert all(
    actual is expected
    for actual, expected in zip(scene_family_actual, scene_family_expected)
)
assert family_scene.get_mobject_family_members() is not scene_family_actual
assert family_scene.get_time() == 0.0
family_scene.wait(1.0 / 30.0)
assert abs(family_scene.get_time() - (1.0 / 30.0)) < 1e-15
family_roots_after = family_scene.get_mobjects()
assert len(family_roots_after) == len(family_roots_before)
assert all(
    after is before
    for after, before in zip(family_roots_after, family_roots_before)
)

# fm-5wq.4.87: documented wait controls backed by the native runtime route
# through the portal. The stop predicate observes post-frame scene time and
# deliberately shortens the native wait after the frame where it turns true.
stopped_wait_scene = Scene()
stop_condition_times = []


def stop_after_two_wait_frames():
    stop_condition_times.append(stopped_wait_scene.get_time())
    return len(stop_condition_times) == 2


stopped_wait_scene.wait(1.0, stop_condition=stop_after_two_wait_frames)
assert len(stop_condition_times) == 2
assert math.isclose(stop_condition_times[0], 1.0 / 30.0)
assert math.isclose(stop_condition_times[1], 2.0 / 30.0)
assert math.isclose(stopped_wait_scene.get_time(), 2.0 / 30.0)

presenter_bypass_scene = Scene()
presenter_bypass_scene.wait(1.0 / 30.0, ignore_presenter_mode=True)
assert math.isclose(presenter_bypass_scene.get_time(), 1.0 / 30.0)

try:
    Scene().wait(1.0, bogus=True)
except NotImplementedError as error:
    assert str(error) == "Scene.wait unsupported keyword(s): bogus"
else:
    raise AssertionError("Scene.wait accepted unsupported keyword bogus")

# fm-5wq.4: wait() without a duration uses the constructor default_wait_time
# (Reference 1.0s) instead of a native None sentinel.
assert scene_module.Scene.default_wait_time == 1.0
default_wait_scene = Scene(default_wait_time=2.0 / 30.0)
assert np.isclose(default_wait_scene.default_wait_time, 2.0 / 30.0)
assert default_wait_scene.wait() is None
assert math.isclose(default_wait_scene.get_time(), 2.0 / 30.0)
assert default_wait_scene.leave_progress_bars is False
assert Scene(leave_progress_bars=True).leave_progress_bars is True
assert str(inspect.signature(scene_module.Scene.wait)) == (
    "(self, duration=None, stop_condition=None, note=None, "
    "ignore_presenter_mode=False, **kwargs)"
)
assert Scene().wait(1.0 / 30.0, note="slide") is None

copy_source = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]
)
copy_source.label = {"shared": True}
copy_scene = Scene()
copy_scene.add(copy_source, copy_source)
copy_roots_before = copy_scene.get_mobjects()
scene_copies = copy_scene.get_mobject_copies()
assert len(scene_copies) == 2
assert scene_copies[0] is not copy_source
assert scene_copies[1] is not copy_source
assert scene_copies[0] is not scene_copies[1]
assert type(scene_copies[0]) is type(copy_source)
assert scene_copies[0].label is copy_source.label
assert np.allclose(scene_copies[0].get_points(), copy_source.get_points())
scene_copies[0].shift([3.0, 0.0, 0.0])
assert not np.allclose(scene_copies[0].get_points(), copy_source.get_points())
assert copy_scene.get_mobjects() == copy_roots_before
assert all(copy not in copy_scene.get_mobjects() for copy in scene_copies)
assert copy_scene.get_mobject_copies() is not scene_copies


# Bound copy uses Marionette's CopyMap, then Python remaps __dict__ aliases.
parent.label = child
parent.buddy = outsider
parent.nested = [child]
parent.uniforms["plugin_value"] = {"nested": [1, 2]}
tick = lambda mob, dt: None
parent.updaters.append(tick)
clone = copy.copy(parent)
assert type(clone) is type(parent)
assert clone is not parent
assert clone.submobjects[0] is clone.label
assert clone.label is not child
assert clone.buddy is outsider
assert clone.nested is parent.nested
assert clone.updaters is not parent.updaters
assert clone.updaters[0] is tick
assert clone.uniforms["plugin_value"] == {"nested": [1, 2]}
clone.set_field("point", 0, [99.0, 0.0, 0.0])
assert parent.get_field("point", 0) == [1.0, 1.0, 0.0]

deep = copy.deepcopy(parent)
assert deep.label is deep.submobjects[0]
assert deep.nested is not parent.nested
assert deep.nested[0] is deep.label
assert deep.uniforms["plugin_value"] is not parent.uniforms["plugin_value"]


# High-frequency Mobject helpers compose the live Stage-backed organization,
# copy, state, transform, and style seams instead of schema placeholders.
method_deep = parent.deepcopy()
assert method_deep.label is method_deep.submobjects[0]
assert method_deep.nested is not parent.nested
assert method_deep.nested[0] is method_deep.label
assert method_deep.uniforms["plugin_value"] is not parent.uniforms["plugin_value"]


def utility_marker(x, name):
    marker = VMobject().set_points_as_corners(
        [[x - 0.25, 0.0, 0.0], [x + 0.25, 0.0, 0.0]]
    )
    marker.name = name
    return marker


utility_a = utility_marker(2.0, "a")
utility_b = utility_marker(-1.0, "b")
utility_c = utility_marker(1.0, "c")
utility_d = utility_marker(1.0, "d")
utility_group = Mobject().add(utility_a, utility_b)
utility_scene = Scene()
utility_scene.add(utility_group)

# list_update keeps the last occurrence, so already-present utility_a wins
# over the same object in the prepend list and every child remains unique.
assert utility_group.add_to_back(utility_c, utility_a) is utility_group
assert list(utility_group.submobjects) == [utility_c, utility_a, utility_b]
assert utility_group.add_to_back(utility_d, utility_d) is utility_group
assert list(utility_group.submobjects) == [utility_d, utility_c, utility_a, utility_b]

# Python's stable sort is committed into Marionette. Equal-center c/d order is
# preserved by the default branch; both callback forms remain supported.
assert utility_group.sort() is utility_group
assert list(utility_group.submobjects) == [utility_b, utility_d, utility_c, utility_a]
assert utility_group.sort(point_to_num_func=lambda point: -point[0]) is utility_group
assert list(utility_group.submobjects) == [utility_a, utility_d, utility_c, utility_b]
assert utility_group.sort(submob_func=lambda mob: mob.name) is utility_group
assert [mob.name for mob in utility_group.submobjects] == ["a", "b", "c", "d"]
# A bound copy is sourced from the Stage, independently proving native child
# order rather than merely re-reading the compatibility list we just mutated.
assert [mob.name for mob in utility_group.copy().submobjects] == ["a", "b", "c", "d"]

shuffle_child_members = [utility_marker(i, f"inner-{i}") for i in range(3)]
shuffle_child = Mobject().add(*shuffle_child_members)
shuffle_leaf = utility_marker(5.0, "outer-leaf")
shuffle_root = Mobject().add(shuffle_child, shuffle_leaf)
shuffle_seed = 1729
shuffle_rng = manimlib.random.Random(shuffle_seed)
expected_inner = list(shuffle_child_members)
expected_outer = [shuffle_child, shuffle_leaf]
shuffle_rng.shuffle(expected_inner)
shuffle_rng.shuffle(expected_outer)
manimlib.random.seed(shuffle_seed)
assert shuffle_root.shuffle(recurse=True) is shuffle_root
assert list(shuffle_child.submobjects) == expected_inner
assert list(shuffle_root.submobjects) == expected_outer

detached_restore_child = utility_marker(0.0, "detached-restore")
detached_restore_group = Mobject().add(detached_restore_child)
detached_restore_group.named_child = detached_restore_child
detached_restore_group.save_state()
detached_restore_group.named_child = utility_marker(8.0, "wrong-detached-alias")
detached_restore_group.shift([3.0, -2.0, 0.0])
assert detached_restore_group.restore() is detached_restore_group
assert np.allclose(detached_restore_child.get_center(), [0.0, 0.0, 0.0])
assert detached_restore_group.named_child is detached_restore_child

bound_restore_child = utility_marker(0.0, "bound-restore")
bound_restore_group = Mobject().add(bound_restore_child)
bound_restore_group.named_child = bound_restore_child
bound_restore_scene = Scene()
bound_restore_scene.add(bound_restore_group)
bound_restore_group.save_state()
bound_restore_group.named_child = utility_marker(8.0, "wrong-bound-alias")
bound_restore_group.shift([-4.0, 1.5, 0.0])
assert bound_restore_group.restore() is bound_restore_group
assert np.allclose(bound_restore_child.get_center(), [0.0, 0.0, 0.0])
assert bound_restore_group.named_child is bound_restore_child

try:
    Mobject().restore()
except Exception as error:
    assert type(error) is Exception
    assert str(error) == "Trying to restore without having saved"
else:
    raise AssertionError("restore without save_state did not refuse")

shape_restore_group = Mobject().add(Mobject())
shape_restore_group.resize(1)
shape_restore_group.save_state()
shape_restore_group.set_field("point", 0, [4.0, 0.0, 0.0])
shape_restore_group.add(Mobject())
# restore is the Reference's become(saved_state) (mobject.py:715), which
# aligns the two families first (9a9671e2): the saved single-child family
# is padded to the edited two-child shape, and the data returns to the
# saved state.
assert shape_restore_group.restore() is shape_restore_group
assert shape_restore_group.get_field("point", 0) == [0.0, 0.0, 0.0]
assert len(shape_restore_group.submobjects) == 2

schema_restore_group = Mobject().add(Mobject())
schema_restore_group.resize(1)
schema_restore_group.save_state()
schema_restore_group.set_field("point", 0, [7.0, 0.0, 0.0])
schema_restore_group.set_submobjects([VMobject()])
try:
    schema_restore_group.restore()
except RuntimeError as error:
    assert str(error) == "become between records of different schemas"
else:
    raise AssertionError("detached restore silently accepted record-schema drift")
assert schema_restore_group.get_field("point", 0) == [7.0, 0.0, 0.0]

spacing_left = utility_marker(1.0, "spacing-left")
spacing_right = utility_marker(3.0, "spacing-right")
spacing_group = Mobject().add(spacing_left, spacing_right)
spacing_widths = [mob.get_width() for mob in spacing_group.submobjects]
assert (
    spacing_group.space_out_submobjects(2.0, about_point=[1.0, 0.0, 0.0])
    is spacing_group
)
assert np.allclose(spacing_left.get_center(), [1.0, 0.0, 0.0])
assert np.allclose(spacing_right.get_center(), [5.0, 0.0, 0.0])
assert np.allclose(
    [mob.get_width() for mob in spacing_group.submobjects], spacing_widths
)

# High-demand transform and frame-containment surface. These assertions cover
# both detached nursery graphs and a Scene-bound native family so a Python
# shell loop cannot accidentally double-apply the native Stage traversal.
assert str(inspect.signature(Mobject.apply_points_function)) == (
    "(self, func, about_point=None, about_edge=array([0., 0., 0.]), "
    "works_on_bounding_box=False)"
)
assert Mobject.dim == 3
assert str(inspect.signature(Mobject.apply_function)) == (
    "(self, function, **kwargs)"
)
assert str(inspect.signature(Mobject.apply_function_to_position)) == (
    "(self, function)"
)
assert str(inspect.signature(Mobject.apply_function_to_submobject_positions)) == (
    "(self, function)"
)
assert str(inspect.signature(Mobject.apply_matrix)) == "(self, matrix, **kwargs)"
assert str(inspect.signature(Mobject.apply_complex_function)) == (
    "(self, function, **kwargs)"
)
assert str(inspect.signature(Mobject.shift_onto_screen)) == "(self, **kwargs)"
assert str(inspect.signature(Mobject.is_off_screen)) == "(self)"
assert str(inspect.signature(VMobject.apply_function)) == (
    "(self, function, make_smooth=False, **kwargs)"
)
assert str(inspect.signature(VMobject.apply_matrix)) == "(self, *args, **kwargs)"
assert VMobject.make_smooth_after_applying_functions is False


def nonlinear_box_map(rows):
    result = rows.copy()
    result[:, 0] = rows[:, 0] + rows[:, 1] ** 2
    return result


box_mapped = manimlib.Square()
box_before = box_mapped.get_bounding_box().copy()
point_map_dtypes = []


def dtype_observing_box_map(rows):
    point_map_dtypes.append(rows.dtype)
    return nonlinear_box_map(rows)


assert (
    box_mapped.apply_points_function(
        dtype_observing_box_map,
        about_edge=None,
        works_on_bounding_box=True,
    )
    is box_mapped
)
assert np.allclose(box_mapped.get_bounding_box(), nonlinear_box_map(box_before))
assert point_map_dtypes == [np.dtype(np.float32), np.dtype(float)]

partial_box = manimlib.Square()
partial_box_points = partial_box.get_points().copy()
partial_box_before = partial_box.get_bounding_box().copy()
partial_box_calls = 0


def mutate_then_fail_on_box(rows):
    global partial_box_calls
    partial_box_calls += 1
    rows[:] += manimlib.RIGHT
    if partial_box_calls == 2:
        raise LookupError("box-map boom")
    return rows


try:
    partial_box.apply_points_function(
        mutate_then_fail_on_box,
        about_edge=None,
        works_on_bounding_box=True,
    )
except LookupError as error:
    assert str(error) == "box-map boom"
else:
    raise AssertionError("a mutating bounding-box callback did not propagate")
assert np.allclose(partial_box.get_points(), partial_box_points + manimlib.RIGHT)
assert np.allclose(
    partial_box.get_bounding_box(), partial_box_before + manimlib.RIGHT
)

# Every member keeps its directly transformed Reference box until a later
# member revision invalidates that family of caches.
box_child = manimlib.Square().shift(3 * manimlib.RIGHT)
box_family = Mobject().add(manimlib.Square(), box_child)
family_box_before = box_family.get_bounding_box().copy()
box_family.apply_points_function(
    nonlinear_box_map,
    about_edge=None,
    works_on_bounding_box=True,
)
assert np.allclose(
    box_family.get_bounding_box(), nonlinear_box_map(family_box_before)
)
box_child.shift(manimlib.RIGHT)
assert np.allclose(box_family.get_right(), [6.0, 0.0, 0.0])

detached_matrix_child = manimlib.Square().shift(manimlib.RIGHT)
detached_matrix_family = Mobject().add(detached_matrix_child)
detached_before = detached_matrix_child.get_center().copy()
assert detached_matrix_family.apply_matrix([[2.0, 0.0], [0.0, 3.0]]) is (
    detached_matrix_family
)
assert np.allclose(detached_matrix_child.get_center(), [2.0, 0.0, 0.0])
assert not np.allclose(detached_matrix_child.get_center(), 4.0 * detached_before)

bound_matrix_child = manimlib.Square().shift(manimlib.RIGHT)
bound_matrix_family = Mobject().add(bound_matrix_child)
bound_matrix_scene = Scene()
bound_matrix_scene.add(bound_matrix_family)
assert bound_matrix_family.apply_matrix(
    [[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 1.0]]
) is bound_matrix_family
assert np.allclose(bound_matrix_child.get_center(), [2.0, 0.0, 0.0])

custom_point_fields = CustomPointFields()
assert custom_point_fields.apply_matrix([[2.0, 0.0], [0.0, 3.0]]) is (
    custom_point_fields
)
assert custom_point_fields.get_field("point", 0) == [2.0, 6.0, 0.0]
assert custom_point_fields.get_field("control", 0) == [6.0, 12.0, 0.0]

matrix_override = MatrixPointMapperOverride()
matrix_override.matrix_point_mapper_calls = 0
matrix_override.apply_matrix([[2.0, 0.0], [0.0, 3.0]])
assert matrix_override.matrix_point_mapper_calls == 1
assert matrix_override.get_field("point", 0) == [2.0, 6.0, 0.0]

family_walker_override = MatrixFamilyWalkerOverride()
family_walker_override.matrix_family_walker_calls = 0
family_walker_override.apply_matrix([[2.0, 0.0], [0.0, 3.0]])
assert family_walker_override.matrix_family_walker_calls == 1

shared_matrix_leaf = manimlib.Square().shift(manimlib.RIGHT)
shared_matrix_family = Mobject().add(
    Mobject().add(shared_matrix_leaf), Mobject().add(shared_matrix_leaf)
)
shared_matrix_family.apply_matrix([[2.0, 0.0], [0.0, 1.0]])
assert np.allclose(shared_matrix_leaf.get_center(), [4.0, 0.0, 0.0])

matrix_refusal = manimlib.Square().shift([1.0, 2.0, 0.0])
matrix_refusal_points = matrix_refusal.get_points().copy()
try:
    matrix_refusal.apply_matrix(np.ones((4, 4)))
except ValueError:
    pass
else:
    raise AssertionError("an oversized matrix did not preserve NumPy refusal")
assert np.array_equal(matrix_refusal.get_points(), matrix_refusal_points)

function_mapped = manimlib.Square().shift([1.0, 2.0, 0.0])
function_width = function_mapped.get_width()
assert function_mapped.apply_function(lambda point: point + [2.0, -1.0, 0.0]) is (
    function_mapped
)
assert np.allclose(function_mapped.get_center(), [3.0, 1.0, 0.0])
assert np.isclose(function_mapped.get_width(), function_width)

position_mapped = manimlib.Square().shift([1.0, 2.0, 0.0])
position_width = position_mapped.get_width()
assert position_mapped.apply_function_to_position(lambda point: 2.0 * point) is (
    position_mapped
)
assert np.allclose(position_mapped.get_center(), [2.0, 4.0, 0.0])
assert np.isclose(position_mapped.get_width(), position_width)

positions_left = manimlib.Square().shift(manimlib.LEFT)
positions_right = manimlib.Square().shift(manimlib.RIGHT)
positions_group = Mobject().add(positions_left, positions_right)
assert positions_group.apply_function_to_submobject_positions(
    lambda point: point + [0.0, 2.0, 0.0]
) is positions_group
assert np.allclose(positions_left.get_center(), [-1.0, 2.0, 0.0])
assert np.allclose(positions_right.get_center(), [1.0, 2.0, 0.0])

complex_mapped = manimlib.Square().shift([1.0, 2.0, 3.0])
assert complex_mapped.apply_complex_function(lambda value: value * (1.0 + 1.0j)) is (
    complex_mapped
)
assert np.allclose(complex_mapped.get_center(), [-1.0, 3.0, 3.0])

callback_partial = Mobject().add(
    manimlib.Square().shift(manimlib.LEFT),
    manimlib.Square().shift(manimlib.RIGHT),
)
callback_before = [mob.get_points().copy() for mob in callback_partial.submobjects]
callback_calls = 0


def fail_on_second_member(rows):
    global callback_calls
    callback_calls += 1
    if callback_calls == 2:
        raise KeyError("point-map boom")
    return rows + manimlib.UP


try:
    callback_partial.apply_points_function(fail_on_second_member, about_edge=None)
except KeyError as error:
    assert error.args == ("point-map boom",)
else:
    raise AssertionError("a point-map callback exception did not propagate")
assert np.allclose(
    callback_partial.submobjects[0].get_points(), callback_before[0] + manimlib.UP
)
assert np.array_equal(
    callback_partial.submobjects[1].get_points(), callback_before[1]
)

smooth_probe = utility_marker(0.0, "smooth-transform")
smooth_calls = []
smooth_probe.make_smooth = lambda approx=True: smooth_calls.append(approx) or smooth_probe
assert smooth_probe.apply_function(lambda point: point, make_smooth=True) is smooth_probe
assert smooth_calls == [True]

screen_probe = manimlib.Square().shift([20.0, -20.0, 0.0])
assert screen_probe.is_off_screen()
assert screen_probe.shift_onto_screen() is screen_probe
assert not screen_probe.is_off_screen()
screen_box = screen_probe.get_bounding_box().copy()
screen_probe.shift_onto_screen()
assert np.array_equal(screen_probe.get_bounding_box(), screen_box)

boundary_probe = Mobject()
boundary_probe.resize(1)
inside_boundary = float(
    np.nextafter(np.float32(manimlib.FRAME_X_RADIUS), np.float32(-np.inf))
)
outside_boundary = float(
    np.nextafter(np.float32(manimlib.FRAME_X_RADIUS), np.float32(np.inf))
)
boundary_probe.set_field("point", 0, [inside_boundary, 0.0, 0.0])
assert not boundary_probe.is_off_screen()
boundary_probe.set_field("point", 0, [outside_boundary, 0.0, 0.0])
assert boundary_probe.is_off_screen()

oversized_screen_probe = manimlib.Rectangle(width=20.0, height=10.0)
assert oversized_screen_probe.shift_onto_screen(buff=0.25) is oversized_screen_probe
assert np.isclose(
    oversized_screen_probe.get_right()[0], manimlib.FRAME_X_RADIUS - 0.25
)
assert np.isclose(
    oversized_screen_probe.get_bottom()[1], -manimlib.FRAME_Y_RADIUS + 0.25
)

style_source = Mobject()
style_source.resize(1)
style_source.set_color("#336699").set_opacity(0.35).set_shading(0.2, 0.4, 0.6)
style_target = Mobject()
style_target.resize(1)
style_target.set_color("#ff0000").set_opacity(0.9).set_shading(0.9, 0.8, 0.7)
assert style_target.match_color(style_source) is style_target
assert style_target.get_color() == style_source.get_color()
assert np.isclose(style_target.get_opacity(), 0.9)
assert np.allclose(style_target.get_shading(), [0.9, 0.8, 0.7])
assert style_target.match_style(style_source) is style_target
assert style_target.get_color() == style_source.get_color()
assert np.isclose(style_target.get_opacity(), style_source.get_opacity())
assert np.allclose(style_target.get_shading(), style_source.get_shading())

fade_child = Mobject()
fade_child.resize(1)
fade_child.set_opacity(0.25)
fade_parent = Mobject().add(fade_child)
fade_parent.resize(1)
fade_parent.set_opacity(0.8, recurse=False)
assert fade_parent.fade(0.1, recurse=False) is None
assert np.isclose(fade_parent.get_opacity(), 0.9)
assert np.isclose(fade_child.get_opacity(), 0.25)
assert fade_parent.fade(0.4) is None
assert np.isclose(fade_parent.get_opacity(), 0.6)
assert np.isclose(fade_child.get_opacity(), 0.6)

vmobject_fade = utility_marker(0.0, "vmobject-fade").set_opacity(0.8)
assert vmobject_fade.fade(0.25) is vmobject_fade
assert np.isclose(vmobject_fade.get_opacity(), 0.6)

# The high-demand color helpers route both interpolation policies through
# fmn-core while preserving Reference branching and exact refusal text.
assert str(inspect.signature(Mobject.set_color_by_gradient)) == "(self, *colors)"
assert str(inspect.signature(Mobject.set_submobject_colors_by_gradient)) == (
    "(self, *colors, interp_by_hsl=False)"
)
gradient_group = Mobject().add(manimlib.Square(), manimlib.Square(), manimlib.Square())
assert (
    gradient_group.set_submobject_colors_by_gradient("#FF0000", "#0000FF")
    is gradient_group
)
assert [mob.get_color() for mob in gradient_group.submobjects] == [
    "#FF0000",
    "#B400B4",
    "#0000FF",
]
assert (
    gradient_group.set_submobject_colors_by_gradient(
        "#FF0000", "#0000FF", interp_by_hsl=True
    )
    is gradient_group
)
assert [mob.get_color() for mob in gradient_group.submobjects] == [
    "#FF0000",
    "#00FF00",
    "#0000FF",
]
gradient_colors_before_refusal = [
    mob.get_color() for mob in gradient_group.submobjects
]
try:
    gradient_group.set_submobject_colors_by_gradient()
except Exception as error:
    assert type(error) is Exception
    assert str(error) == "Need at least one color"
else:
    raise AssertionError("an empty submobject gradient did not refuse")
assert [mob.get_color() for mob in gradient_group.submobjects] == (
    gradient_colors_before_refusal
)
assert gradient_group.set_submobject_colors_by_gradient("#123456") is gradient_group
assert all(mob.get_color() == "#123456" for mob in gradient_group.submobjects)
empty_gradient_group = Mobject()
assert (
    empty_gradient_group.set_submobject_colors_by_gradient("#FF0000", "#0000FF")
    is empty_gradient_group
)

point_gradient = manimlib.Square()
assert point_gradient.set_color_by_gradient("#FF0000", "#0000FF") is point_gradient
point_gradient_colors = point_gradient.get_fill_colors()
assert point_gradient_colors[0] == "#FF0000"
assert point_gradient_colors[-1] == "#0000FF"

pointless_gradient = Mobject().add(*[manimlib.Square() for _ in range(3)])
assert pointless_gradient.set_color_by_gradient("#FF0000", "#0000FF") is pointless_gradient
assert [mob.get_color() for mob in pointless_gradient.submobjects] == [
    "#FF0000",
    "#B400B4",
    "#0000FF",
]


# Pickle restores detached state, preserves family aliases, and can rebind.
pickled_parent = Mobject()
pickled_child = Mobject()
pickled_parent.add(pickled_child)
pickled_parent.label = pickled_child
pickled_parent.resize(1)
pickled_parent.set_field("point", 0, [3.0, 4.0, 5.0])
pickled_parent.shift([2.0, 3.0, 0.0])
pickle_source_scene = Scene()
pickle_source_scene.add(pickled_parent)
payload = pickle.dumps(pickled_parent, protocol=pickle.HIGHEST_PROTOCOL)
restored = pickle.loads(payload)  # ubs:ignore -- trusted round-trip created immediately above
assert restored.label is restored.submobjects[0]
assert restored.get_field("point", 0) == [5.0, 7.0, 5.0]
assert restored.family_size() == 2
pickle_destination_scene = Scene()
pickle_destination_scene.add(restored)
assert restored.family_size() == 2
assert restored.is_alive()

# Native pickle state is a versioned, checksummed FMNA document. Decode is
# transactional: malformed bytes and unknown envelope versions cannot replace
# even a detached destination's already-live state.
pickle_state = restored._engine_state()
assert pickle_state["version"] == 1
assert pickle_state["snapshot"].startswith(b"FMNA")
unchanged = Mobject()
unchanged.resize(1)
unchanged.set_field("point", 0, [9.0, 8.0, 7.0])
malformed_state = dict(pickle_state)
malformed_state["snapshot"] = pickle_state["snapshot"][:-1]
try:
    unchanged._restore_engine_state(malformed_state)
except ValueError as error:
    assert "snapshot container refused" in str(error)
else:
    raise AssertionError("truncated native pickle state was accepted")
assert unchanged.get_field("point", 0) == [9.0, 8.0, 7.0]
unknown_version = dict(pickle_state)
unknown_version["version"] = 2
try:
    unchanged._restore_engine_state(unknown_version)
except ValueError as error:
    assert "unsupported Python portal pickle-state version 2" in str(error)
else:
    raise AssertionError("unknown native pickle-state version was accepted")
assert unchanged.get_field("point", 0) == [9.0, 8.0, 7.0]
mismatched_summary = dict(pickle_state)
mismatched_summary["fields"] = [("fabricated", 1)]
try:
    unchanged._restore_engine_state(mismatched_summary)
except ValueError as error:
    assert "field summary does not match" in str(error)
else:
    raise AssertionError("pickle state with a fabricated field summary was accepted")
assert unchanged.get_field("point", 0) == [9.0, 8.0, 7.0]


# Weakrefs, identity round trips, and explicit worker-thread confinement.
reference = weakref.ref(clone)
assert reference() is clone
identity_map = {parent: "parent", child: "child"}
assert identity_map[parent] == "parent"
assert identity_map[scene.mobjects[0]] == "parent"
del clone
gc.collect()
assert reference() is None


# Exercise Rust-owned updater cycles through the real scene in native,
# installed-wheel and registered portal E2E acceptance.
def batched_scene_cycle_observers(callback_owns_anchor):
    cycle_scene = Scene()
    member = Mobject()
    member.resize(1)
    member.set_field("point", 0, [0.0, 0.0, 0.0])
    anchor = Mobject() if callback_owns_anchor else member
    owner = anchor if callback_owns_anchor else None

    def callback(views, dt, retained_owner=owner):
        if callback_owns_anchor:
            assert retained_owner is not None
        views[0]["point"][:, 0] += dt

    anchor.add_updater(manimlib._BatchedUpdater([member], callback), call=False)
    cycle_scene.add(anchor, member)
    gc.collect()
    cycle_scene.update(0.25)
    assert member.get_field("point", 0) == [0.25, 0.0, 0.0]
    return tuple(weakref.ref(obj) for obj in (cycle_scene, anchor, member, callback))


for callback_owns_anchor in (False, True):
    batched_observers = batched_scene_cycle_observers(callback_owns_anchor)
    gc.collect()
    gc.collect()
    assert all(observer() is None for observer in batched_observers), (
        "an unreachable batched scene retained native objects",
        callback_owns_anchor,
    )


# fm-2mvb: an instance of a Python subclass references its class, and the
# collector must see that edge. PyO3's synthesized traverse for the
# subclassable pyclasses did not visit the heap type, so a cycle through a
# subclass's class object (a scene-local `class Demo(Scene)` whose construct
# closes over mobjects bound to that scene) was never collected.
def class_cycle_observers():
    class ClassCycleScene(Scene):
        pass

    cyclic = ClassCycleScene()
    ClassCycleScene.instance = cyclic
    return weakref.ref(cyclic), weakref.ref(ClassCycleScene)


class_observers = class_cycle_observers()
gc.collect()
assert all(observer() is None for observer in class_observers), (
    "a cycle through a Scene subclass's class object was not collected"
)


# Reentrant callbacks and Python exceptions cross the engine boundary intact.
updates = []


def updater(mobject, dt):
    point = mobject.get_field("point", 0)
    mobject.set_field("point", 0, [point[0] + dt, point[1], point[2]])
    updates.append(dt)


parent.add_updater(updater, call=False)
scene.update(0.5)
assert updates == [0.5]
assert parent.get_field("point", 0)[0] == 1.5

# Updater delta dispatch follows the Reference's named-``dt`` protocol, not
# callable arity.  Optional positional parameters are often semantic flags;
# passing the frame delta into one silently changes scene meaning.
semantic_default_calls = []
named_dt_calls = []
variadic_calls = []


def semantic_default_updater(mobject, vertical=False, horizontal=False):
    semantic_default_calls.append((mobject, vertical, horizontal))


def named_dt_updater(mobject, dt=0.0):
    named_dt_calls.append((mobject, dt))


def variadic_updater(mobject, *options):
    variadic_calls.append((mobject, options))


dispatch_probe = Mobject()
dispatch_scene = Scene().add(dispatch_probe)
dispatch_probe.add_updater(semantic_default_updater, call=False)
dispatch_probe.add_updater(named_dt_updater, call=False)
dispatch_probe.add_updater(variadic_updater, call=False)
dispatch_scene.update(0.25)
assert semantic_default_calls == [(dispatch_probe, False, False)]
assert named_dt_calls == [(dispatch_probe, 0.25)]
assert variadic_calls == [(dispatch_probe, ())]


# The Python updater surface shares Marionette's durable suspension flag.
# Scene traversal is child-first, a suspended parent prunes its subtree, and
# resuming a child clears the ancestor chain exactly like the Reference.
suspension_scene = Scene()
suspension_parent = Mobject()
suspension_child = Mobject()
suspension_parent.add(suspension_child)
suspension_order = []
suspension_parent.add_updater(
    lambda mob, dt: suspension_order.append(("parent", dt)), call=False
)
suspension_child.add_updater(
    lambda mob, dt: suspension_order.append(("child", dt)), call=False
)
suspension_scene.add(suspension_parent)
suspension_parent.suspend_updating(recurse=False)
suspension_scene.update(0.25)
assert suspension_order == []
suspension_child.resume_updating(recurse=False, call_updater=False)
suspension_scene.update(0.25)
assert suspension_order == [("child", 0.25), ("parent", 0.25)]
suspension_order.clear()
suspension_parent.suspend_updating()
suspension_parent.resume_updating()
assert suspension_order == [("child", 0.0), ("parent", 0.0)]
suspension_order.clear()
suspension_parent.update(0.5, recurse=False)
assert suspension_order == [("parent", 0.5)]

# Public updater removal and suspension remain effective across later native
# Scene.play segments, not only direct Scene.update calls.
updater_lifecycle_scene = Scene()
# A bare Mobject is point-free and therefore keeps the Reference's zero
# bounding box even after a shift.  Use the point-bearing Mobject subclass so
# get_center() is a real witness that the bystander's dt updater ran.
updater_lifecycle_target = manimlib.Point()
updater_lifecycle_clock = Mobject()
updater_lifecycle_scene.add(updater_lifecycle_target, updater_lifecycle_clock)


def shift_lifecycle_target(mob, dt):
    mob.shift(dt * manimlib.RIGHT)


updater_lifecycle_target.add_updater(shift_lifecycle_target, call=False)
updater_lifecycle_scene.play(
    updater_lifecycle_clock.animate.shift(manimlib.UP),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
after_registered_play = updater_lifecycle_target.get_center().copy()
assert after_registered_play[0] > 0.0

updater_lifecycle_target.remove_updater(shift_lifecycle_target)
updater_lifecycle_scene.play(
    updater_lifecycle_clock.animate.shift(manimlib.UP),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.array_equal(
    updater_lifecycle_target.get_center(), after_registered_play
)

updater_lifecycle_target.add_updater(shift_lifecycle_target, call=False)
updater_lifecycle_target.suspend_updating()
updater_lifecycle_scene.play(
    updater_lifecycle_clock.animate.shift(manimlib.UP),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.array_equal(
    updater_lifecycle_target.get_center(), after_registered_play
)
updater_lifecycle_target.resume_updating(call_updater=False)

updater_lifecycle_target.clear_updaters()
assert updater_lifecycle_target.updaters == []

try:
    updater_lifecycle_target.add_updater(None)
except TypeError as error:
    assert str(error) == (
        "Mobject.add_updater requires a callable updater; got NoneType"
    )
else:
    raise AssertionError("Mobject.add_updater accepted None")


class ExplodingUpdater(Mobject):
    pass


exploding = ExplodingUpdater()
exploding.add_updater(
    lambda mob, dt: (_ for _ in ()).throw(KeyError("updater boom")),
    call=False,
)
scene.add(exploding)
try:
    scene.update(0.1)
except KeyError as error:
    assert "updater boom" in str(error)
else:
    raise AssertionError("Python updater exception did not propagate")
scene.remove(exploding)


class InterpolatingMobject(Mobject):
    def __init__(self, value):
        self.alphas = []
        super().__init__()
        self.resize(1)
        self.set_field("point", 0, [value, 0.0, 0.0])

    def interpolate(self, start, target, alpha):
        self.alphas.append(alpha)
        return super().interpolate(start, target, alpha)


interpolating = InterpolatingMobject(0.0)
interpolation_target = InterpolatingMobject(3.0)
scene.add(interpolating, interpolation_target)
scene.run_transform(interpolating, interpolation_target, 2)
assert interpolating.alphas == [0.0, 0.5, 1.0]
assert interpolating.get_field("point", 0) == [3.0, 0.0, 0.0]


class LifecycleAnimation(Animation):
    def __init__(self):
        super().__init__(Mobject())
        self.calls = []

    def begin(self):
        self.calls.append("begin")

    def interpolate_mobject(self, alpha):
        self.calls.append(("interpolate_mobject", alpha))

    def finish(self):
        self.calls.append("finish")


lifecycle_animation = LifecycleAnimation()
lifecycle_animation.begin()
lifecycle_animation.interpolate(0.25)
lifecycle_animation.finish()
assert lifecycle_animation.calls == [
    "begin",
    ("interpolate_mobject", 0.25),
    "finish",
]


class ExplodingInit(Mobject):
    def init_points(self):
        raise ValueError("init boom")


try:
    ExplodingInit()
except ValueError as error:
    assert "init boom" in str(error)
else:
    raise AssertionError("Python lifecycle exception did not propagate")


class LifecycleScene(Scene):
    def __init__(self):
        super().__init__()
        self.calls = []

    def setup(self):
        self.calls.append("setup")

    def construct(self):
        self.calls.append("construct")

    def tear_down(self):
        self.calls.append("tear_down")


lifecycle_scene = LifecycleScene()
lifecycle_scene.run()
assert lifecycle_scene.calls == ["setup", "construct", "tear_down"]


class FailingLifecycleScene(LifecycleScene):
    def construct(self):
        super().construct()
        raise RuntimeError("construct boom")


failing_lifecycle_scene = FailingLifecycleScene()
try:
    failing_lifecycle_scene.run()
except RuntimeError as error:
    assert "construct boom" in str(error)
else:
    raise AssertionError("Scene.construct exception did not propagate")
assert failing_lifecycle_scene.calls == ["setup", "construct", "tear_down"]

# Scene.add_sound is a real native request boundary, not a schema placeholder.
# The call-site time stays on Choreo's exact rational grid while the relative
# offset remains separate for Reel's eventual sample-grid conversion (BN-14).
sound_scene = Scene()
assert sound_scene.add_sound("click.wav", -0.125, -6.0, -12.0) is None
assert sound_scene._sound_request_facts() == [
    ("click.wav", 0, 30, -0.125, -6.0, -12.0)
]
sound_scene.wait(0.25)
assert sound_scene.add_sound("tone.wav", gain=-3.0) is None
assert sound_scene._sound_request_facts()[1] == (
    "tone.wav",
    8,
    30,
    0.0,
    -3.0,
    None,
)
sound_requests_before_refusal = sound_scene._sound_request_facts()
try:
    sound_scene.add_sound("bad.wav", gain_to_background=-math.inf)
except ValueError as error:
    assert str(error) == (
        "invalid scene configuration: sound gain_to_background must be finite"
    )
else:
    raise AssertionError("a non-finite sound gain reached the native request list")
assert sound_scene._sound_request_facts() == sound_requests_before_refusal

interactive_scene = InteractiveScene()
try:
    interactive_scene.checkpoint_paste()
except bridge_errors.CapabilityError as error:
    assert "active host scene console" in str(error)
else:
    raise AssertionError("checkpoint_paste returned bytes instead of requiring a cell session")
assert isinstance(interactive_scene._checkpoint_bytes(), bytes)
assert str(inspect.signature(InteractiveScene.get_crosshair)) == "(self)"
assert np.isclose(InteractiveScene.crosshair_width, 0.2)
crosshair = interactive_scene.get_crosshair()
assert isinstance(crosshair, manimlib.VGroup)
assert len(crosshair.submobjects) == 2
assert np.isclose(crosshair.get_width(), 0.2)
assert crosshair.is_fixed_in_frame()
assert crosshair.get_stroke_color() == manimlib.GREY_A
assert str(inspect.signature(InteractiveScene.get_selection_rectangle)) == (
    "(self)"
)
selection_rect = interactive_scene.get_selection_rectangle()
assert isinstance(selection_rect, manimlib.Rectangle)
assert selection_rect.get_stroke_color() == manimlib.WHITE
assert np.isclose(selection_rect.get_stroke_width(), 1.0)
assert selection_rect.is_fixed_in_frame()
assert np.allclose(selection_rect.fixed_corner, manimlib.ORIGIN)
assert str(inspect.signature(InteractiveScene.get_color_palette)) == "(self)"
assert InteractiveScene.palette_colors is manimlib.MANIM_COLORS
palette = interactive_scene.get_color_palette()
assert isinstance(palette, manimlib.VGroup)
assert len(palette.submobjects) == len(manimlib.MANIM_COLORS)
assert palette.is_fixed_in_frame()
assert np.isclose(palette.get_width(), manimlib.FRAME_WIDTH - 0.5, atol=1e-4)
assert palette.submobjects[0].get_fill_color() == manimlib.color_to_hex(
    manimlib.MANIM_COLORS[0]
)
assert str(inspect.signature(InteractiveScene.get_corner_dots)) == (
    "(self, mobject)"
)
corner_source = manimlib.Circle()
corner_dots = interactive_scene.get_corner_dots(corner_source)
assert isinstance(corner_dots, manimlib.DotCloud)
assert corner_dots.get_num_points() == 4
assert np.isclose(corner_dots.get_radius(), 0.05)
assert np.isclose(corner_dots.get_glow_factor(), 2.0)
assert len(corner_dots.updaters) == 1
assert str(inspect.signature(InteractiveScene.get_information_label)) == (
    "(self)"
)
info_label = interactive_scene.get_information_label()
assert isinstance(info_label, manimlib.VGroup)
assert len(info_label.submobjects) == 2
loc_label, time_label = info_label.submobjects
assert isinstance(loc_label, manimlib.VGroup)
assert len(loc_label.submobjects) == 3
assert all(
    isinstance(part, manimlib.DecimalNumber) for part in loc_label
)
assert isinstance(time_label, manimlib.DecimalNumber)
assert loc_label.is_fixed_in_frame()
assert time_label.is_fixed_in_frame()
assert np.isclose(time_label.get_value(), 0.0)
assert time_label.get_fill_color() == manimlib.GREY_C
assert isinstance(interactive_scene.mouse_point, manimlib.Point)
assert isinstance(interactive_scene.mouse_drag_point, manimlib.Point)
assert interactive_scene.mouse_point is not interactive_scene.mouse_drag_point
assert np.allclose(interactive_scene.mouse_point.get_center(), 0.0)
assert str(inspect.signature(InteractiveScene.setup)) == "(self)"
interactive_scene.setup()
assert isinstance(interactive_scene.selection, manimlib.Group)
assert interactive_scene.selection_highlight in interactive_scene.mobjects
assert interactive_scene.is_selecting is False
assert interactive_scene.is_grabbing is False
assert interactive_scene.select_top_level_mobs is True
assert interactive_scene.crosshair is not interactive_scene.color_palette
assert interactive_scene.get_selection_search_set() == []
assert str(inspect.signature(InteractiveScene.add)) == "(self, *mobjects)"
assert str(inspect.signature(InteractiveScene.remove)) == "(self, *mobjects)"
assert str(inspect.signature(InteractiveScene.remove_all_except)) == (
    "(self, *mobjects_to_keep)"
)
circle_for_select = manimlib.Circle()
interactive_scene.add(circle_for_select)
assert circle_for_select in interactive_scene.get_selection_search_set()
second_circle_for_select = manimlib.Circle()
interactive_scene.add(second_circle_for_select)
selection_search_set = interactive_scene.get_selection_search_set()
assert circle_for_select in selection_search_set
assert second_circle_for_select in selection_search_set
interactive_scene.remove(circle_for_select)
selection_search_set = interactive_scene.get_selection_search_set()
assert circle_for_select not in selection_search_set
assert second_circle_for_select in selection_search_set
assert all(
    mobject not in selection_search_set
    for mobject in interactive_scene.unselectables
)

# fm-5wq.4: interaction exclusion follows native family membership and keeps
# the live selection search set synchronized after every disable/enable.
assert str(inspect.signature(InteractiveScene.disable_interaction)) == (
    "(self, *mobjects)"
)
assert str(inspect.signature(InteractiveScene.enable_interaction)) == (
    "(self, *mobjects)"
)
interaction_scene = InteractiveScene()
interaction_scene.setup()
interaction_source = manimlib.Circle()
interaction_scene.add(interaction_source)
assert interaction_source in interaction_scene.get_selection_search_set()
interaction_scene.disable_interaction(interaction_source)
assert interaction_source in interaction_scene.unselectables
assert interaction_source not in interaction_scene.get_selection_search_set()
interaction_scene.add_to_selection(interaction_source)
assert interaction_source not in interaction_scene.selection
interaction_scene.enable_interaction(interaction_source)
assert interaction_source in interaction_scene.get_selection_search_set()
interaction_left = manimlib.Circle()
interaction_right = manimlib.Square()
interaction_group = manimlib.Group(interaction_left, interaction_right)
interaction_scene.disable_interaction(interaction_group)
assert interaction_group in interaction_scene.unselectables
assert interaction_left in interaction_scene.unselectables
assert interaction_right in interaction_scene.unselectables

# fm-5wq.4: delete_selection routes selected scene members through native
# remove, then clears the live native Group without disturbing bystanders.
assert str(inspect.signature(InteractiveScene.delete_selection)) == "(self)"
delete_scene = InteractiveScene()
delete_scene.setup()
delete_a = manimlib.Circle()
delete_b = manimlib.Circle()
delete_scene.add(delete_a, delete_b)
delete_scene.add_to_selection(delete_a)
assert delete_a in delete_scene.selection
assert delete_a in delete_scene.mobjects
assert delete_scene.delete_selection() is None
assert delete_a not in delete_scene.mobjects
assert delete_a not in delete_scene.selection
assert len(delete_scene.selection) == 0
assert delete_b in delete_scene.mobjects
assert delete_scene.delete_selection() is None
assert len(delete_scene.selection) == 0
assert delete_b in delete_scene.mobjects

# fm-5wq.4: toggle_selection_mode flips the selection scope through the native
# family walk and regenerates the top-level vs pointful-member search set.
assert str(inspect.signature(InteractiveScene.toggle_selection_mode)) == (
    "(self)"
)
assert str(inspect.signature(InteractiveScene.refresh_selection_scope)) == (
    "(self)"
)
toggle_scene = InteractiveScene()
toggle_scene.setup()
assert toggle_scene.select_top_level_mobs is True
toggle_parent = manimlib.VGroup(manimlib.Circle(), manimlib.Circle())
toggle_scene.add(toggle_parent)
assert toggle_parent in toggle_scene.get_selection_search_set()
toggle_scene.toggle_selection_mode()
assert toggle_scene.select_top_level_mobs is False
toggle_search_set = toggle_scene.get_selection_search_set()
assert all(
    member in toggle_search_set
    for member in toggle_parent.family_members_with_points()
)
toggle_scene.toggle_selection_mode()
assert toggle_scene.select_top_level_mobs is True

interactive_scene.add_to_selection(circle_for_select)
assert list(interactive_scene.selection.submobjects) == [circle_for_select]
interactive_scene.clear_selection()
assert list(interactive_scene.selection.submobjects) == []
assert InteractiveScene.select_top_level_mobs is True
assert str(inspect.signature(InteractiveScene.get_highlight)) == (
    "(self, mobject)"
)
top_highlight = interactive_scene.get_highlight(corner_source)
assert isinstance(top_highlight, manimlib.DotCloud)
assert top_highlight.get_num_points() == 4
cloud_highlight = interactive_scene.get_highlight(corner_dots)
assert type(cloud_highlight) is manimlib.Mobject
assert cloud_highlight.get_num_points() == 0
interactive_scene.select_top_level_mobs = False
piece_highlight = interactive_scene.get_highlight(corner_source)
assert isinstance(piece_highlight, manimlib.VHighlight)
assert len(piece_highlight.updaters) == 1
interactive_scene.select_top_level_mobs = True
assert str(inspect.signature(InteractiveScene.get_selection_highlight)) == (
    "(self)"
)
selection_highlight = interactive_scene.get_selection_highlight()
assert isinstance(selection_highlight, manimlib.Group)
assert selection_highlight.tracked_mobjects == []
assert len(selection_highlight.updaters) == 1

# fm-5wq.4: InteractiveScene.setup owns the Reference selection identities
# over the portal's native Group/VMobject surfaces.
assert str(inspect.signature(InteractiveScene.setup)) == "(self)"
setup_scene = InteractiveScene()
setup_scene.setup()
assert isinstance(setup_scene.selection, manimlib.Group)
assert len(setup_scene.selection.submobjects) == 0
assert isinstance(setup_scene.selection_highlight, manimlib.Group)
assert type(setup_scene.selection_highlight) is type(
    setup_scene.get_selection_highlight()
)
assert isinstance(setup_scene.selection_rectangle, manimlib.Rectangle)
assert isinstance(setup_scene.crosshair, manimlib.VGroup)
assert isinstance(setup_scene.information_label, manimlib.VGroup)
assert isinstance(setup_scene.color_palette, manimlib.VGroup)
assert setup_scene.select_top_level_mobs is True
assert setup_scene.is_selecting is False
assert setup_scene.is_grabbing is False
assert setup_scene.selection in setup_scene.unselectables
assert setup_scene.camera.frame in setup_scene.unselectables
assert setup_scene.selection_highlight in setup_scene.mobjects
assert hasattr(setup_scene, "regenerate_selection_search_set")
selection_search_set = setup_scene.get_selection_search_set()
assert isinstance(selection_search_set, list)
assert all(
    mobject not in setup_scene.unselectables
    for mobject in selection_search_set
)

# fm-5wq.4: information display mounts and unmounts setup's native
# DecimalNumber/VGroup overlay without making it selectable.
assert str(inspect.signature(InteractiveScene.display_information)) == (
    "(self, show=True)"
)
information_scene = InteractiveScene()
information_scene.setup()
assert information_scene.information_label not in information_scene.mobjects
assert information_scene.information_label in information_scene.unselectables
information_scene.display_information()
assert information_scene.information_label in information_scene.mobjects
assert (
    information_scene.information_label
    not in information_scene.get_selection_search_set()
)
information_loc_label, information_time_label = (
    information_scene.information_label.submobjects
)
assert isinstance(information_loc_label, manimlib.VGroup)
assert len(information_loc_label.submobjects) == 3
assert all(
    isinstance(part, manimlib.DecimalNumber)
    for part in information_loc_label.submobjects
)
assert isinstance(information_time_label, manimlib.DecimalNumber)
assert np.isclose(information_time_label.get_value(), 0.0)
assert information_loc_label.is_fixed_in_frame()
assert information_time_label.is_fixed_in_frame()
assert all(
    part.get_fill_color() == manimlib.GREY_C
    for part in information_loc_label.submobjects
)
assert information_time_label.get_fill_color() == manimlib.GREY_C
information_scene.display_information(False)
assert information_scene.information_label not in information_scene.mobjects

# fm-5wq.4: the Reference keeps the pointer as two live Point mobjects on the
# Scene itself. Before this binding they were simply absent, and
# get_information_label's coordinate updater fell through its getattr guard to
# a dead ORIGIN on every frame.
assert isinstance(interactive_scene.mouse_point, manimlib.Point)
assert isinstance(interactive_scene.mouse_drag_point, manimlib.Point)
assert type(interactive_scene.mouse_point) is manimlib.Point
assert type(interactive_scene.mouse_drag_point) is manimlib.Point
assert np.allclose(interactive_scene.mouse_point.get_center(), manimlib.ORIGIN)
assert np.allclose(
    interactive_scene.mouse_drag_point.get_center(), manimlib.ORIGIN
)
# Two distinct mobjects, not one location aliased under two names.
assert interactive_scene.mouse_point is not interactive_scene.mouse_drag_point
# The binding lives on Scene, so a plain Scene carries it identically.
mouse_base_scene = Scene()
assert isinstance(mouse_base_scene.mouse_point, manimlib.Point)
assert isinstance(mouse_base_scene.mouse_drag_point, manimlib.Point)
assert np.allclose(mouse_base_scene.mouse_point.get_center(), manimlib.ORIGIN)
assert np.allclose(
    mouse_base_scene.mouse_drag_point.get_center(), manimlib.ORIGIN
)
assert mouse_base_scene.mouse_point is not mouse_base_scene.mouse_drag_point
# Pointer state is per scene; two scenes never share one Point.
assert interactive_scene.mouse_point is not mouse_base_scene.mouse_point
assert interactive_scene.mouse_drag_point is not mouse_base_scene.mouse_drag_point
# Moving one never drags the other along with it, in either direction.
interactive_scene.mouse_point.move_to([1.0, 2.0, 0.0])
assert np.allclose(interactive_scene.mouse_point.get_center(), [1.0, 2.0, 0.0])
assert np.allclose(
    interactive_scene.mouse_drag_point.get_center(), manimlib.ORIGIN
)
interactive_scene.mouse_drag_point.move_to([-3.0, 0.5, 0.0])
assert np.allclose(
    interactive_scene.mouse_drag_point.get_center(), [-3.0, 0.5, 0.0]
)
assert np.allclose(interactive_scene.mouse_point.get_center(), [1.0, 2.0, 0.0])
assert np.allclose(mouse_base_scene.mouse_point.get_center(), manimlib.ORIGIN)
# The information label's own coordinate updater reads the live scene Point.
# `loc_label` was built while mouse_point still sat at ORIGIN, so running the
# updater it already carries must now track the move — the existing updater is
# exercised here, never rewritten.
assert np.allclose([part.get_value() for part in loc_label], manimlib.ORIGIN)
assert len(loc_label.updaters) == 1
loc_label.updaters[0](loc_label)
assert np.allclose([part.get_value() for part in loc_label], [1.0, 2.0, 0.0])
# A label constructed after the move reads the same live location up front.
moved_info_label = interactive_scene.get_information_label()
moved_loc_label = moved_info_label.submobjects[0]
assert np.allclose(
    [part.get_value() for part in moved_loc_label], [1.0, 2.0, 0.0]
)
interactive_scene.mouse_point.move_to(manimlib.ORIGIN)
interactive_scene.mouse_drag_point.move_to(manimlib.ORIGIN)
assert str(inspect.signature(InteractiveScene.add_to_selection)) == (
    "(self, *mobjects)"
)
assert str(inspect.signature(InteractiveScene.toggle_from_selection)) == (
    "(self, *mobjects)"
)
assert str(inspect.signature(InteractiveScene.clear_selection)) == "(self)"
selection_scene = InteractiveScene()
selection_source = manimlib.Circle()
selection_scene.add_to_selection(selection_source)
assert selection_source in selection_scene.selection
selection_scene.add_to_selection(selection_source)
assert list(selection_scene.selection).count(selection_source) == 1
selection_scene.toggle_from_selection(selection_source)
assert selection_source not in selection_scene.selection
selection_scene.toggle_from_selection(selection_source)
assert selection_source in selection_scene.selection
selection_scene.clear_selection()
assert len(selection_scene.selection) == 0
unselectable_source = manimlib.Circle()
selection_scene.unselectables.append(unselectable_source)
selection_scene.add_to_selection(unselectable_source)
assert unselectable_source not in selection_scene.selection

# fm-5wq.4: InteractiveScene.enable_selection pins the sweep rectangle at
# the live mouse in fixed-frame coordinates; group_selection wraps the
# current selection through Scene.get_group and re-selects that group.
assert str(inspect.signature(InteractiveScene.enable_selection)) == "(self)"
assert str(inspect.signature(InteractiveScene.group_selection)) == "(self)"
enable_scene = InteractiveScene()
enable_scene.setup()
enable_scene.mouse_point.move_to([1.25, -0.5, 0.0])
assert enable_scene.enable_selection() is None
assert enable_scene.is_selecting is True
assert enable_scene.selection_rectangle in enable_scene.mobjects
assert np.allclose(
    enable_scene.selection_rectangle.fixed_corner,
    enable_scene.frame.to_fixed_frame_point(
        enable_scene.mouse_point.get_center()
    ),
)
assert np.allclose(
    enable_scene.selection_rectangle.fixed_corner,
    [1.25, -0.5, 0.0],
)
enable_scene.enable_selection()
assert list(enable_scene.mobjects).count(enable_scene.selection_rectangle) == 1

group_scene = InteractiveScene()
group_scene.setup()
left_selected = manimlib.Circle().shift([-1.0, 0.0, 0.0])
right_selected = manimlib.Square().shift([1.0, 0.0, 0.0])
group_scene.add(left_selected, right_selected)
group_scene.add_to_selection(left_selected, right_selected)
assert list(group_scene.selection) == [left_selected, right_selected]
assert group_scene.group_selection() is None
assert len(group_scene.selection) == 1
grouped_selection = group_scene.selection[0]
assert isinstance(grouped_selection, manimlib.VGroup)
assert list(grouped_selection) == [left_selected, right_selected]
assert grouped_selection in group_scene.mobjects
assert left_selected not in group_scene.selection
assert right_selected not in group_scene.selection
assert str(inspect.signature(InteractiveScene.ungroup_selection)) == "(self)"
assert group_scene.ungroup_selection() is None
assert list(group_scene.selection) == [left_selected, right_selected]
assert grouped_selection not in group_scene.selection
assert left_selected in group_scene.mobjects
assert right_selected in group_scene.mobjects

# fm-5wq.4: gather_new_selection ends the sweep through native bounding
# boxes: the rectangle grown by update_selection_rectangle hit-tests the
# search set in fixed-frame coordinates and toggles covered mobjects into
# the live selection. A vanished rectangle leaves the selection untouched.
assert str(inspect.signature(InteractiveScene.gather_new_selection)) == (
    "(self)"
)
assert str(inspect.signature(InteractiveScene.update_selection_rectangle)) == (
    "(self, rect)"
)
sweep_scene = InteractiveScene()
sweep_scene.setup()
sweep_target = manimlib.Circle().move_to(manimlib.ORIGIN)
sweep_scene.add(sweep_target)
sweep_scene.mouse_point.move_to(manimlib.ORIGIN)
sweep_scene.enable_selection()
sweep_scene.mouse_point.move_to([2.0, 2.0, 0.0])
assert (
    sweep_scene.update_selection_rectangle(sweep_scene.selection_rectangle)
    is sweep_scene.selection_rectangle
)
assert sweep_scene.gather_new_selection() is None
assert sweep_scene.is_selecting is False
assert sweep_scene.selection_rectangle not in sweep_scene.mobjects
assert sweep_target in sweep_scene.selection
assert sweep_scene.gather_new_selection() is None
assert sweep_scene.selection_rectangle not in sweep_scene.mobjects
assert sweep_target in sweep_scene.selection

# fm-5wq.4: nudge_selection shifts the live native selection Group with the
# Reference step size and tenfold large-nudge modifier.
assert InteractiveScene.selection_nudge_size == 0.05
assert str(inspect.signature(InteractiveScene.nudge_selection)) == (
    "(self, vect, large=False)"
)
nudge_selection_scene = InteractiveScene()
nudge_selection_scene.setup()
nudge_selection_circle = manimlib.Circle()
nudge_selection_scene.add(nudge_selection_circle)
nudge_selection_scene.add_to_selection(nudge_selection_circle)
nudge_selection_origin = nudge_selection_circle.get_center().copy()
assert nudge_selection_scene.nudge_selection(manimlib.RIGHT) is None
assert np.allclose(
    nudge_selection_circle.get_center(),
    nudge_selection_origin + 0.05 * manimlib.RIGHT,
)
assert nudge_selection_scene.nudge_selection(manimlib.UP, large=True) is None
assert np.allclose(
    nudge_selection_circle.get_center(),
    nudge_selection_origin + 0.05 * manimlib.RIGHT + 0.5 * manimlib.UP,
)
empty_nudge_selection_scene = InteractiveScene()
empty_nudge_selection_scene.setup()
assert empty_nudge_selection_scene.nudge_selection(manimlib.RIGHT) is None

# fm-5wq.4: toggle_selection_mode flips select_top_level_mobs, then
# refresh_selection_scope rewrites the live selection Group between scene
# roots and pointed family members. Search-set regeneration rides the same
# call so piece-mode hit testing sees the expanded family.
assert str(inspect.signature(InteractiveScene.toggle_selection_mode)) == (
    "(self)"
)
assert str(inspect.signature(InteractiveScene.refresh_selection_scope)) == (
    "(self)"
)
scope_scene = InteractiveScene()
scope_scene.setup()
scope_child_a = manimlib.Circle()
scope_child_b = manimlib.Square()
scope_parent = manimlib.VGroup(scope_child_a, scope_child_b)
scope_scene.add(scope_parent)
scope_scene.add_to_selection(scope_parent)
assert scope_scene.select_top_level_mobs is True
assert list(scope_scene.selection) == [scope_parent]
assert scope_parent in scope_scene.get_selection_search_set()
assert scope_child_a not in scope_scene.get_selection_search_set()
assert scope_scene.toggle_selection_mode() is None
assert scope_scene.select_top_level_mobs is False
assert list(scope_scene.selection) == [scope_child_a, scope_child_b]
assert scope_parent not in scope_scene.selection
assert scope_child_a in scope_scene.get_selection_search_set()
assert scope_child_b in scope_scene.get_selection_search_set()
assert scope_scene.toggle_selection_mode() is None
assert scope_scene.select_top_level_mobs is True
assert list(scope_scene.selection) == [scope_parent]
assert scope_parent in scope_scene.get_selection_search_set()
assert scope_child_a not in scope_scene.get_selection_search_set()

# fm-5wq.4: gather_new_selection ends a sweep, hit-tests the live rectangle
# against the search set in fixed-frame coordinates, and toggles matches.
# A missing rectangle is a no-op besides clearing is_selecting. nudge_selection
# shifts the live selection Group by selection_nudge_size (x10 when large).
assert str(inspect.signature(InteractiveScene.gather_new_selection)) == (
    "(self)"
)
assert str(inspect.signature(InteractiveScene.nudge_selection)) == (
    "(self, vect, large=False)"
)
assert np.isclose(InteractiveScene.selection_nudge_size, 0.05)
gather_scene = InteractiveScene()
gather_scene.setup()
gather_hit = manimlib.Circle().move_to([0.0, 0.0, 0.0])
gather_miss = manimlib.Circle().move_to([10.0, 0.0, 0.0])
gather_scene.add(gather_hit, gather_miss)
assert gather_scene.gather_new_selection() is None
assert gather_scene.is_selecting is False
assert len(gather_scene.selection) == 0
gather_scene.enable_selection()
assert gather_scene.is_selecting is True
assert gather_scene.selection_rectangle in gather_scene.mobjects
assert gather_scene.gather_new_selection() is None
assert gather_scene.is_selecting is False
assert gather_scene.selection_rectangle not in gather_scene.mobjects
assert gather_hit in gather_scene.selection
assert gather_miss not in gather_scene.selection

nudge_scene = InteractiveScene()
nudge_scene.setup()
nudge_mob = manimlib.Circle()
nudge_scene.add(nudge_mob)
nudge_scene.add_to_selection(nudge_mob)
nudge_start = nudge_mob.get_center().copy()
assert nudge_scene.nudge_selection(manimlib.RIGHT) is None
assert np.allclose(
    nudge_mob.get_center(),
    nudge_start + InteractiveScene.selection_nudge_size * manimlib.RIGHT,
)
assert nudge_scene.nudge_selection(manimlib.UP, large=True) is None
assert np.allclose(
    nudge_mob.get_center(),
    nudge_start
    + InteractiveScene.selection_nudge_size * manimlib.RIGHT
    + 10.0 * InteractiveScene.selection_nudge_size * manimlib.UP,
)

# fm-5wq.4: InteractiveScene checkpoints exclude transient interaction
# overlays and restore the native selection highlight exactly once at root 0.
assert str(inspect.signature(InteractiveScene.get_state)) == "(self)"
assert str(inspect.signature(InteractiveScene.restore_state)) == (
    "(self, scene_state)"
)
interactive_state_scene = InteractiveScene()
interactive_state_scene.setup()
interactive_state_user = manimlib.Circle()
interactive_state_scene.add(interactive_state_user)
interactive_state_origin = interactive_state_user.get_center().copy()
interactive_state = interactive_state_scene.get_state()
assert (
    interactive_state_scene.selection_highlight
    not in interactive_state.mobjects_to_copies
)
assert (
    interactive_state_scene.selection_rectangle
    not in interactive_state.mobjects_to_copies
)
assert (
    interactive_state_scene.crosshair
    not in interactive_state.mobjects_to_copies
)
assert interactive_state_user in interactive_state.mobjects_to_copies
interactive_state_user.shift(manimlib.RIGHT)
assert interactive_state_scene.restore_state(interactive_state) is None
assert np.allclose(interactive_state_user.get_center(), interactive_state_origin)
assert interactive_state_user in interactive_state_scene.mobjects
assert interactive_state_scene.selection_highlight in interactive_state_scene.mobjects
assert interactive_state_scene.mobjects[0] is interactive_state_scene.selection_highlight
assert (
    interactive_state_scene.mobjects.count(
        interactive_state_scene.selection_highlight
    )
    == 1
)
assert interactive_state_scene.restore_state(interactive_state) is None
assert interactive_state_scene.mobjects[0] is interactive_state_scene.selection_highlight
assert (
    interactive_state_scene.mobjects.count(
        interactive_state_scene.selection_highlight
    )
    == 1
)

# fm-5wq.4: free-axis grabbing preserves the mouse-to-selection offset and
# moves the native selection Group as one unit without requiring pyglet.
assert str(inspect.signature(InteractiveScene.prepare_grab)) == "(self)"
assert str(inspect.signature(InteractiveScene.handle_grabbing)) == (
    "(self, point)"
)
grab_scene = InteractiveScene()
grab_scene.setup()
grab_circle = manimlib.Circle().move_to([0.0, 0.0, 0.0])
grab_scene.add(grab_circle)
grab_scene.add_to_selection(grab_circle)
grab_scene.mouse_point.move_to([1.0, 0.0, 0.0])
grab_start = grab_circle.get_center().copy()
assert grab_scene.prepare_grab() is None
assert grab_scene.is_grabbing is True
assert np.allclose(
    grab_scene.mouse_to_selection,
    np.array([1.0, 0.0, 0.0]) - grab_start,
)
assert grab_scene.handle_grabbing([2.0, 1.0, 0.0]) is None
assert np.allclose(grab_scene.selection.get_center(), [1.0, 1.0, 0.0])
assert np.allclose(grab_circle.get_center(), grab_start + [1.0, 1.0, 0.0])

empty_grab_scene = InteractiveScene()
empty_grab_scene.setup()
empty_grab_scene.mouse_point.move_to([1.0, 0.0, 0.0])
assert empty_grab_scene.prepare_grab() is None
assert empty_grab_scene.is_grabbing is True
assert np.allclose(
    empty_grab_scene.mouse_to_selection,
    [1.0, 0.0, 0.0],
)

# fm-5wq.4: resize preparation records native selection geometry while the
# separately owned pyglet-keyed resize handler remains outside this slice.
assert str(inspect.signature(InteractiveScene.prepare_resizing)) == (
    "(self, about_corner=False)"
)
resize_scene = InteractiveScene()
resize_scene.setup()
resize_square = manimlib.Square()
resize_scene.add(resize_square)
resize_scene.add_to_selection(resize_square)
resize_scene.mouse_point.move_to(resize_square.get_right())
assert resize_scene.prepare_resizing() is None
assert np.allclose(
    resize_scene.scale_about_point,
    resize_square.get_center(),
)
assert np.allclose(resize_scene.scale_ref_width, resize_square.get_width())
assert np.allclose(resize_scene.scale_ref_height, resize_square.get_height())
resize_scene.prepare_resizing(about_corner=True)
assert np.allclose(
    resize_scene.scale_about_point,
    resize_scene.selection.get_corner(
        resize_square.get_center() - resize_scene.mouse_point.get_center()
    ),
)
assert not np.allclose(
    resize_scene.scale_about_point,
    resize_square.get_center(),
)
assert str(inspect.signature(InteractiveScene.handle_resizing)) == (
    "(self, point)"
)
unprepared_resize = InteractiveScene()
unprepared_resize.setup()
assert unprepared_resize.handle_resizing([1.0, 0.0, 0.0]) is None
resize_center = resize_square.get_center().copy()
resize_width = float(resize_square.get_width())
resize_scene.mouse_point.move_to(resize_center + manimlib.RIGHT)
assert resize_scene.prepare_resizing() is None
assert resize_scene.handle_resizing(resize_center + 2.0 * manimlib.RIGHT) is None
assert np.isclose(resize_square.get_width(), 2.0 * resize_width)
assert np.allclose(resize_square.get_center(), resize_center)

# fm-5wq.4: toggle_color_palette mounts setup's native palette VGroup through
# Scene add/remove only while something is selected, and the palette stays
# unselectable — never in the regenerated search set.
assert str(inspect.signature(InteractiveScene.toggle_color_palette)) == (
    "(self)"
)
palette_scene = InteractiveScene()
palette_scene.setup()
assert palette_scene.toggle_color_palette() is None
assert palette_scene.color_palette not in palette_scene.mobjects
assert palette_scene.undo_stack == []
palette_circle = manimlib.Circle()
palette_scene.add(palette_circle)
palette_scene.add_to_selection(palette_circle)
assert palette_scene.toggle_color_palette() is None
assert len(palette_scene.undo_stack) == 1
assert palette_scene.color_palette in palette_scene.mobjects
assert (
    palette_scene.color_palette
    not in palette_scene.get_selection_search_set()
)
assert palette_scene.toggle_color_palette() is None
assert palette_scene.color_palette not in palette_scene.mobjects

# fm-5wq.4: Scene.point_to_mobject walks the reversed search set through
# is_point_touching. handle_sweeping_selection adds the top hit from the
# selection search set; choose_color recolors the live selection from the
# first pointful family member under the cursor and unmounts the palette.
assert str(inspect.signature(Scene.point_to_mobject)) == (
    "(self, point, search_set=None, buff=0)"
)
assert str(inspect.signature(InteractiveScene.handle_sweeping_selection)) == (
    "(self, point)"
)
assert str(inspect.signature(InteractiveScene.choose_color)) == "(self, point)"
sweep_hit_scene = InteractiveScene()
sweep_hit_scene.setup()
sweep_hit = manimlib.Circle().move_to([0.0, 0.0, 0.0])
sweep_miss = manimlib.Circle().move_to([10.0, 0.0, 0.0])
sweep_hit_scene.add(sweep_hit, sweep_miss)
assert sweep_hit_scene.point_to_mobject(sweep_hit.get_center()) is sweep_hit
assert sweep_hit_scene.point_to_mobject([100.0, 0.0, 0.0]) is None
assert sweep_hit_scene.handle_sweeping_selection(sweep_hit.get_center()) is None
assert sweep_hit in sweep_hit_scene.selection
assert sweep_miss not in sweep_hit_scene.selection
assert sweep_hit_scene.handle_sweeping_selection([100.0, 0.0, 0.0]) is None
assert list(sweep_hit_scene.selection) == [sweep_hit]

recolor_scene = InteractiveScene()
recolor_scene.setup()
recolor_target = manimlib.Circle().set_color(manimlib.BLUE)
recolor_source = manimlib.Square().set_color(manimlib.RED).move_to(
    [3.0, 0.0, 0.0]
)
recolor_scene.add(recolor_target, recolor_source)
recolor_scene.add_to_selection(recolor_target)
recolor_scene.toggle_color_palette()
assert recolor_scene.color_palette in recolor_scene.mobjects
assert recolor_scene.choose_color(recolor_source.get_center()) is None
assert recolor_target.get_color() == manimlib.RED
assert recolor_scene.color_palette not in recolor_scene.mobjects

# fm-5wq.4: mouse motion updates the native pointer and fixed-frame crosshair,
# then routes an active free-axis grab through the existing Group.move_to seam.
assert str(inspect.signature(InteractiveScene.on_mouse_motion)) == (
    "(self, point, d_point)"
)
pre_setup_motion_scene = InteractiveScene()
assert pre_setup_motion_scene.on_mouse_motion(
    [-1.0, 0.5, 0.0],
    [0.0, 0.0, 0.0],
) is None
assert np.allclose(
    pre_setup_motion_scene.mouse_point.get_center(),
    [-1.0, 0.5, 0.0],
)
assert not hasattr(pre_setup_motion_scene, "crosshair")
motion_scene = InteractiveScene()
motion_scene.setup()
assert motion_scene.on_mouse_motion([1.0, 2.0, 0.0], [0.0, 0.0, 0.0]) is None
fixed_motion_point = motion_scene.frame.to_fixed_frame_point(
    [1.0, 2.0, 0.0]
)
assert np.allclose(
    motion_scene.mouse_point.get_center(),
    [1.0, 2.0, 0.0],
)
assert np.allclose(
    motion_scene.crosshair.get_center()[:2],
    fixed_motion_point[:2],
    atol=1e-4,
)

motion_circle = manimlib.Circle().move_to([0.0, 0.0, 0.0])
motion_scene.add(motion_circle)
motion_scene.add_to_selection(motion_circle)
motion_scene.mouse_point.move_to([1.0, 0.0, 0.0])
motion_start = motion_circle.get_center().copy()
motion_scene.prepare_grab()
assert motion_scene.on_mouse_motion(
    [2.0, 1.0, 0.0],
    [1.0, 1.0, 0.0],
) is None
assert np.allclose(
    motion_circle.get_center(),
    motion_start + [1.0, 1.0, 0.0],
)

# fm-5wq.4: mouse drag shares the native pointer and fixed-frame crosshair
# path without requiring the Reference's pyglet Window.
assert str(inspect.signature(InteractiveScene.on_mouse_drag)) == (
    "(self, point, d_point, buttons, modifiers)"
)
drag_scene = InteractiveScene()
drag_scene.setup()
assert drag_scene.on_mouse_drag(
    [1.0, 2.0, 0.0],
    [0.0, 0.0, 0.0],
    1,
    0,
) is None
fixed_drag_point = drag_scene.frame.to_fixed_frame_point([1.0, 2.0, 0.0])
assert np.allclose(
    drag_scene.mouse_drag_point.get_center(),
    [1.0, 2.0, 0.0],
)
assert np.allclose(
    drag_scene.mouse_point.get_center(),
    [0.0, 0.0, 0.0],
)
assert np.allclose(
    drag_scene.crosshair.get_center()[:2],
    fixed_drag_point[:2],
    atol=1e-4,
)

# fm-5wq.4: mouse release clears ordinary selection, while a mounted palette
# routes the release point through the existing native choose_color path.
assert str(inspect.signature(InteractiveScene.on_mouse_release)) == (
    "(self, point, button, mods)"
)
release_scene = InteractiveScene()
release_scene.setup()
release_circle = manimlib.Circle()
release_scene.add(release_circle)
release_scene.add_to_selection(release_circle)
assert release_circle in release_scene.selection
assert release_scene.on_mouse_release(manimlib.ORIGIN, 0, 0) is None
assert len(release_scene.selection) == 0

palette_release_scene = InteractiveScene()
palette_release_scene.setup()
palette_release_target = manimlib.Circle().set_color(manimlib.BLUE)
palette_release_source = manimlib.Square().set_color(manimlib.RED).move_to(
    [3.0, 0.0, 0.0]
)
palette_release_scene.add(palette_release_target, palette_release_source)
palette_release_scene.add_to_selection(palette_release_target)
palette_release_scene.toggle_color_palette()
assert palette_release_scene.color_palette in palette_release_scene.mobjects
assert palette_release_scene.on_mouse_release(
    palette_release_source.get_center(), 0, 0
) is None
assert palette_release_target.get_color() == manimlib.RED
assert palette_release_scene.color_palette not in palette_release_scene.mobjects

# fm-5wq.4: key press/release dispatch pinned manim_config bindings without
# pyglet. Select enables the sweep rectangle and mounts the crosshair;
# grab/release toggles is_grabbing; unselect clears the live Group.
assert str(inspect.signature(InteractiveScene.on_key_press)) == (
    "(self, symbol, modifiers)"
)
assert str(inspect.signature(InteractiveScene.on_key_release)) == (
    "(self, symbol, modifiers)"
)
key_scene = InteractiveScene()
key_scene.setup()
assert key_scene.on_key_press(ord("s"), 0) is None
assert key_scene.is_selecting is True
assert key_scene.selection_rectangle in key_scene.mobjects
assert key_scene.crosshair in key_scene.mobjects
key_circle = manimlib.Circle()
key_scene.add(key_circle)
key_scene.add_to_selection(key_circle)
assert key_circle in key_scene.selection
assert key_scene.on_key_press(ord("u"), 0) is None
assert len(key_scene.selection) == 0
assert key_scene.on_key_press(ord("g"), 0) is None
assert key_scene.is_grabbing is True
assert key_scene.on_key_release(ord("g"), 0) is None
assert key_scene.is_grabbing is False
assert key_scene.on_key_release(ord("s"), 0) is None
assert key_scene.is_selecting is False

# fm-5wq.4: Scene.add populates id_to_mobject_map; ctrl/cmd-g groups the
# live selection and ctrl/cmd-shift-g ungroups, correcting the Reference's
# unreachable ungroup branch that tested ctrl-g first.
assert str(inspect.signature(Scene.id_to_mobject)) == "(self, id_value)"
assert str(inspect.signature(Scene.ids_to_group)) == "(self, *id_values)"
assert str(inspect.signature(Scene.i2m)) == "(self, id_value)"
assert str(inspect.signature(Scene.i2g)) == "(self, *id_values)"
id_scene = Scene()
id_circle = manimlib.Circle()
id_square = manimlib.Square()
id_scene.add(id_circle, id_square)
assert id_scene.id_to_mobject(id(id_circle)) is id_circle
assert id_scene.i2m(id(id_square)) is id_square
assert id_scene.id_to_mobject(0) is None
id_group = id_scene.ids_to_group(id(id_circle), id(id_square))
assert isinstance(id_group, manimlib.VGroup)
assert list(id_group) == [id_circle, id_square]
assert list(id_scene.i2g(id(id_circle))) == [id_circle]

group_key_scene = InteractiveScene()
group_key_scene.setup()
group_left = manimlib.Circle().shift([-1.0, 0.0, 0.0])
group_right = manimlib.Square().shift([1.0, 0.0, 0.0])
group_key_scene.add(group_left, group_right)
group_key_scene.add_to_selection(group_left, group_right)
assert group_key_scene.on_key_press(ord("g"), 2) is None  # pyglet MOD_CTRL
assert len(group_key_scene.selection) == 1
grouped = group_key_scene.selection[0]
assert isinstance(grouped, manimlib.VGroup)
assert list(grouped) == [group_left, group_right]
assert group_key_scene.on_key_press(ord("g"), 2 | 1) is None  # CTRL|SHIFT
assert list(group_key_scene.selection) == [group_left, group_right]

# fm-5wq.4: ctrl/cmd-a selects every scene root except unselectables;
# ctrl/cmd-t flips select_top_level_mobs through refresh_selection_scope.
select_all_scene = InteractiveScene()
select_all_scene.setup()
select_all_a = manimlib.Circle()
select_all_b = manimlib.Square()
select_all_scene.add(select_all_a, select_all_b)
assert select_all_scene.on_key_press(ord("a"), 2) is None
assert select_all_a in select_all_scene.selection
assert select_all_b in select_all_scene.selection
assert select_all_scene.selection_highlight not in select_all_scene.selection
assert select_all_scene.select_top_level_mobs is True
assert select_all_scene.on_key_press(ord("t"), 2) is None
assert select_all_scene.select_top_level_mobs is False
assert select_all_scene.on_key_press(ord("t"), 2) is None
assert select_all_scene.select_top_level_mobs is True
interactive_keys = importlib.import_module("manimlib.scene.interactive_scene")
assert interactive_keys.SELECT_KEY == "s"
assert interactive_keys.UNSELECT_KEY == "u"
assert interactive_keys.GRAB_KEYS == ["g", "h", "v", "z"]
assert manimlib.SELECT_KEY == "s"
assert manimlib.COLOR_KEY == "c"
assert manimlib.ALL_MODIFIERS == (2 | 64 | 1)
assert manimlib.ARROW_SYMBOLS == [0xFF51, 0xFF52, 0xFF53, 0xFF54]

# fm-5wq.4: backspace deletes the live selection through native Scene.remove;
# arrow keys nudge by selection_nudge_size (x10 with shift); cursor key
# toggles the native crosshair without a pyglet window.
nudge_key_scene = InteractiveScene()
nudge_key_scene.setup()
nudge_key_circle = manimlib.Circle()
nudge_key_scene.add(nudge_key_circle)
nudge_key_scene.add_to_selection(nudge_key_circle)
nudge_key_start = nudge_key_circle.get_center().copy()
assert nudge_key_scene.on_key_press(0xFF53, 0) is None  # pyglet RIGHT
assert np.allclose(
    nudge_key_circle.get_center(),
    nudge_key_start + InteractiveScene.selection_nudge_size * manimlib.RIGHT,
)
assert nudge_key_scene.on_key_press(0xFF53, 1) is None  # SHIFT|RIGHT
assert np.allclose(
    nudge_key_circle.get_center(),
    nudge_key_start
    + InteractiveScene.selection_nudge_size * manimlib.RIGHT
    + 10.0 * InteractiveScene.selection_nudge_size * manimlib.RIGHT,
)
assert nudge_key_scene.on_key_press(0xFF08, 0) is None  # BACKSPACE
assert nudge_key_circle not in nudge_key_scene.mobjects
assert len(nudge_key_scene.selection) == 0
cursor_scene = InteractiveScene()
cursor_scene.setup()
assert cursor_scene.crosshair not in cursor_scene.mobjects
assert cursor_scene.on_key_press(ord("k"), 0) is None
assert cursor_scene.crosshair in cursor_scene.mobjects
assert cursor_scene.on_key_press(ord("k"), 0) is None
assert cursor_scene.crosshair not in cursor_scene.mobjects

# fm-5wq.4: grab and resize keys checkpoint through Scene.save_state
# without a pyglet window; shift-t prepares a corner resize.
resize_key_scene = InteractiveScene()
resize_key_scene.setup()
resize_key_circle = manimlib.Circle()
resize_key_scene.add(resize_key_circle)
resize_key_scene.add_to_selection(resize_key_circle)
assert resize_key_scene.undo_stack == []
assert resize_key_scene.on_key_press(ord("t"), 0) is None
assert len(resize_key_scene.undo_stack) == 1
assert not hasattr(resize_key_scene, "scale_about_point")
assert resize_key_scene.on_key_press(ord("t"), 1) is None  # pyglet MOD_SHIFT
assert len(resize_key_scene.undo_stack) == 1
assert hasattr(resize_key_scene, "scale_about_point")
color_key_scene = InteractiveScene()
color_key_scene.setup()
color_key_circle = manimlib.Circle()
color_key_scene.add(color_key_circle)
color_key_scene.add_to_selection(color_key_circle)
assert color_key_scene.on_key_press(ord("c"), 0) is None
assert color_key_scene.color_palette in color_key_scene.mobjects
assert color_key_scene.on_key_press(ord("i"), 0) is None
assert color_key_scene.information_label in color_key_scene.mobjects
assert color_key_scene.on_key_release(ord("i"), 0) is None
assert color_key_scene.information_label not in color_key_scene.mobjects
try:
    color_key_scene.on_key_press(ord("c"), 2)  # CTRL-c clipboard
except bridge_errors.CapabilityError as error:
    assert "clipboard" in str(error).lower()
else:
    raise AssertionError("ctrl-c faked a clipboard transfer")
for clipboard_key in ("v", "x"):
    selection_before_refusal = list(color_key_scene.selection)
    try:
        color_key_scene.on_key_press(ord(clipboard_key), 2)
    except bridge_errors.CapabilityError as error:
        assert "clipboard" in str(error).lower()
    else:
        raise AssertionError(
            f"ctrl-{clipboard_key} faked a clipboard transfer"
        )
    assert list(color_key_scene.selection) == selection_before_refusal
    assert color_key_circle in color_key_scene.mobjects
try:
    color_key_scene.on_key_press(ord("d"), 1)  # SHIFT-d frame
except bridge_errors.CapabilityError as error:
    assert "clipboard" in str(error).lower()
else:
    raise AssertionError("shift-d faked a clipboard transfer")
try:
    color_key_scene.on_key_press(ord("c"), 1)  # SHIFT-c cursor
except bridge_errors.CapabilityError as error:
    assert "clipboard" in str(error).lower()
else:
    raise AssertionError("shift-c faked a clipboard transfer")

# fm-5wq.4: clipboard selection transfer is an explicit host-capability
# refusal; the sovereign portal never imports pyperclip or IPython implicitly.
assert str(inspect.signature(InteractiveScene.copy_selection)) == "(self)"
assert str(inspect.signature(InteractiveScene.paste_selection)) == "(self)"
clipboard_scene = InteractiveScene()
clipboard_scene.setup()
clipboard_circle = manimlib.Circle()
clipboard_scene.add(clipboard_circle)
clipboard_scene.add_to_selection(clipboard_circle)
try:
    clipboard_scene.copy_selection()
except bridge_errors.CapabilityError as error:
    clipboard_error = str(error).lower()
    assert "clipboard" in clipboard_error
    assert "pyperclip" in clipboard_error or "ipython" in clipboard_error
else:
    raise AssertionError("copy_selection faked a clipboard transfer")
assert clipboard_circle in clipboard_scene.selection
try:
    clipboard_scene.paste_selection()
except bridge_errors.CapabilityError as error:
    clipboard_error = str(error).lower()
    assert "clipboard" in clipboard_error
    assert "pyperclip" in clipboard_error or "ipython" in clipboard_error
else:
    raise AssertionError("paste_selection faked a clipboard transfer")
assert str(inspect.signature(InteractiveScene.copy_cursor_position)) == (
    "(self)"
)
assert str(inspect.signature(InteractiveScene.copy_frame_positioning)) == (
    "(self)"
)
clipboard_frame_center = clipboard_scene.frame.get_center().copy()
clipboard_mouse_center = clipboard_scene.mouse_point.get_center().copy()
try:
    clipboard_scene.copy_cursor_position()
except bridge_errors.CapabilityError as error:
    clipboard_error = str(error).lower()
    assert "clipboard" in clipboard_error
    assert "pyperclip" in clipboard_error
else:
    raise AssertionError("copy_cursor_position faked a clipboard transfer")
try:
    clipboard_scene.copy_frame_positioning()
except bridge_errors.CapabilityError as error:
    clipboard_error = str(error).lower()
    assert "clipboard" in clipboard_error
    assert "pyperclip" in clipboard_error
else:
    raise AssertionError("copy_frame_positioning faked a clipboard transfer")
assert np.allclose(
    clipboard_scene.frame.get_center(), clipboard_frame_center
)
assert np.allclose(
    clipboard_scene.mouse_point.get_center(), clipboard_mouse_center
)


# The schema-generated import topology and exact-name aliases are present.
geometry = importlib.import_module("manimlib.mobject.geometry")
circle = geometry.Circle()
assert isinstance(circle, VMobject)

# fm-5wq.4: MotionMobject is an authored portal class over Atlas's native
# wrapper builder. It retains the original child proxy while the separately
# owned drag-event gateway remains an explicit, precisely named capability gap.
interactive = importlib.import_module("manimlib.mobject.interactive")
assert interactive.MotionMobject.__bases__ == (Mobject,)
assert str(inspect.signature(interactive.MotionMobject)) == "(mobject, **kwargs)"
motion_source = geometry.Circle(radius=0.4)
motion = interactive.MotionMobject(motion_source, name="draggable-circle")
assert motion.mobject is motion_source
assert list(motion.submobjects) == [motion_source]
assert motion.name == "draggable-circle"
assert len(motion_source.updaters) == 1
assert motion_source.updaters[0](motion_source) is None
assert str(inspect.signature(interactive.MotionMobject.mob_on_mouse_drag)) == (
    "(self, mob, event_data)"
)

try:
    interactive.MotionMobject(object())
except AssertionError:
    pass
else:
    raise AssertionError("MotionMobject accepted a non-Mobject child")

drag_point = np.array([1.25, -0.5, 0.0])
assert motion.mob_on_mouse_drag(
    motion_source,
    {"point": drag_point},
) is False
assert np.allclose(motion_source.get_center(), drag_point)
assert motion.mobject is motion_source

# Button uses Atlas's native arbitrary-mobject wrapper while preserving the
# Reference's original child and callback identities. mob_on_mouse_press
# forwards the pressed child to on_click and returns False.
clicks = []
button_source = geometry.Circle(radius=0.3)
button_callback = lambda mob: clicks.append(mob)
button = interactive.Button(
    button_source,
    button_callback,
    name="native-button",
)
assert interactive.Button.__bases__ == (Mobject,)
assert str(inspect.signature(interactive.Button)) == (
    "(mobject, on_click, **kwargs)"
)
assert button.mobject is button_source
assert button.on_click is button_callback
assert list(button.submobjects) == [button_source]
assert button.name == "native-button"
assert clicks == []
assert button.mob_on_mouse_press(
    button_source,
    {"point": np.zeros(3)},
) is False
assert clicks == [button_source]
assert button.mobject is button_source

try:
    interactive.Button(object(), button_callback)
except AssertionError:
    pass
else:
    raise AssertionError("Button accepted a non-Mobject child")

# Checkbox keeps the schema's tracker lineage while Atlas owns the box and
# the state-dependent checkmark/cross geometry.
checkbox_true = interactive.Checkbox(True)
checkbox_false = interactive.Checkbox(False)
assert interactive.Checkbox.__bases__ == (interactive.ControlMobject,)
assert interactive.ControlMobject.__bases__ == (manimlib.ValueTracker,)
assert interactive.ControlMobject in interactive.Checkbox.__mro__
assert manimlib.ValueTracker in interactive.Checkbox.__mro__
assert bool(checkbox_true.get_value()) is True
assert bool(checkbox_false.get_value()) is False
assert list(checkbox_true.submobjects) == [
    checkbox_true.box,
    checkbox_true.box_content,
]
assert list(checkbox_false.submobjects) == [
    checkbox_false.box,
    checkbox_false.box_content,
]
# Reference interactive.py builds each mark from two Line children, not a
# filled glyph on its root. Check the real family and independent native
# line geometry rather than locking the old anonymous shell's point count.
def _checkbox_reference_mark(box, checked):
    pairs = ((manimlib.UP / 2 + 2 * manimlib.LEFT, manimlib.DOWN + manimlib.LEFT),
             (manimlib.DOWN + manimlib.LEFT, manimlib.UP + manimlib.RIGHT)) if checked else (
             (manimlib.UP + manimlib.LEFT, manimlib.DOWN + manimlib.RIGHT),
             (manimlib.UP + manimlib.RIGHT, manimlib.DOWN + manimlib.LEFT))
    mark = manimlib.VGroup(*(manimlib.Line(start, end) for start, end in pairs))
    mark.stretch_to_fit_width(box.get_width())
    mark.stretch_to_fit_height(box.get_height())
    return mark.scale(.5).move_to(box).get_all_points()

checked_points = _checkbox_reference_mark(checkbox_true.box, True)
crossed_points = _checkbox_reference_mark(checkbox_false.box, False)
for widget, points in ((checkbox_true, checked_points), (checkbox_false, crossed_points)):
    assert type(widget.box) is manimlib.Rectangle
    assert type(widget.box_content) is manimlib.VGroup
    assert not widget.box_content.has_points()
    assert len(widget.box_content) == 2
    assert all(type(line) is manimlib.Line for line in widget.box_content)
    np.testing.assert_array_equal(widget.box_content.get_all_points(), points)
assert checked_points.shape == crossed_points.shape == (6, 3)
assert not np.array_equal(checked_points, crossed_points)

true_content = checkbox_true.box_content
checkbox_true.toggle_value()
assert bool(checkbox_true.get_value()) is False
assert checkbox_true.box_content is true_content
np.testing.assert_array_equal(checkbox_true.box_content.get_all_points(), crossed_points)
checkbox_true.set_value(True)
assert bool(checkbox_true.get_value()) is True
assert checkbox_true.box_content is true_content
np.testing.assert_array_equal(checkbox_true.box_content.get_all_points(), checked_points)
try:
    checkbox_true.set_value(1)
except AssertionError as error:
    assert str(error) == "Checkbox value must be bool"
else:
    raise AssertionError("Checkbox accepted a non-bool value")

assert checkbox_true.on_mouse_press(
    checkbox_true, {"point": np.zeros(3)}
) is False
assert bool(checkbox_true.get_value()) is False
assert checkbox_true.box_content is true_content
np.testing.assert_array_equal(checkbox_true.box_content.get_all_points(), crossed_points)

failed_checkbox = interactive.Checkbox.__new__(interactive.Checkbox)
try:
    interactive.Checkbox.__init__(failed_checkbox, unsupported=True)
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: unsupported"
else:
    raise AssertionError("Checkbox silently discarded an unknown option")
assert not hasattr(failed_checkbox, "submobjects")

opaque_checkbox = interactive.Checkbox(
    True,
    rect_kwargs=dict(width=0.5, height=0.5, fill_opacity=0.35),
)
assert np.isclose(opaque_checkbox.box.get_fill_opacity(), 0.35)
opaque_box = opaque_checkbox.box
opaque_checkbox.toggle_value()
assert opaque_checkbox.box is opaque_box
assert np.isclose(opaque_checkbox.box.get_fill_opacity(), 0.35)

stroked_checkbox = interactive.Checkbox(
    True,
    checkmark_kwargs=dict(stroke_color=manimlib.GREEN, stroke_width=3.0),
    cross_kwargs=dict(stroke_color=manimlib.RED, stroke_width=4.0),
)
assert all(np.isclose(line.get_stroke_width(), 3.0) for line in stroked_checkbox.box_content)
stroked_content = stroked_checkbox.box_content
stroked_checkbox.toggle_value()
assert stroked_checkbox.box_content is stroked_content
assert all(np.isclose(line.get_stroke_width(), 4.0) for line in stroked_checkbox.box_content)
assert all(line.get_stroke_color() == manimlib.RED and line.get_fill_opacity() == 0.0
           for line in stroked_checkbox.box_content)
assert str(inspect.signature(interactive.Checkbox.get_checkmark)) == "(self)"
assert str(inspect.signature(interactive.Checkbox.get_cross)) == "(self)"
native_check = checkbox_true.get_checkmark()
native_cross = checkbox_true.get_cross()
assert native_check is not checkbox_true.box_content
assert native_cross is not checkbox_true.box_content
np.testing.assert_array_equal(native_check.get_all_points(), checked_points)
np.testing.assert_array_equal(native_cross.get_all_points(), crossed_points)

# EnableDisableButton is a one-box native control on the same real tracker
# base. Construction preserves the Reference's default-white quirk; the first
# state change routes through Atlas and colors the stable box proxy.
enabled_button = interactive.EnableDisableButton(True)
disabled_button = interactive.EnableDisableButton(False)
assert interactive.EnableDisableButton.__bases__ == (
    interactive.ControlMobject,
)
assert interactive.ControlMobject in interactive.EnableDisableButton.__mro__
assert manimlib.ValueTracker in interactive.EnableDisableButton.__mro__
assert bool(enabled_button.get_value()) is True
assert bool(disabled_button.get_value()) is False
assert list(enabled_button.submobjects) == [enabled_button.box]
assert list(disabled_button.submobjects) == [disabled_button.box]
assert enabled_button.box.get_fill_color() == manimlib.WHITE
assert disabled_button.box.get_fill_color() == manimlib.WHITE

enabled_box = enabled_button.box
enabled_button.toggle_value()
assert bool(enabled_button.get_value()) is False
assert enabled_button.box is enabled_box
assert enabled_button.box.get_fill_color() == manimlib.RED
enabled_button.set_value(True)
assert bool(enabled_button.get_value()) is True
assert enabled_button.box is enabled_box
assert enabled_button.box.get_fill_color() == manimlib.GREEN
try:
    enabled_button.set_value(1)
except AssertionError as error:
    assert str(error) == "EnableDisableButton value must be bool"
else:
    raise AssertionError("EnableDisableButton accepted a non-bool value")

assert enabled_button.on_mouse_press(
    enabled_button, {"point": np.zeros(3)}
) is False
assert bool(enabled_button.get_value()) is False
assert enabled_button.box is enabled_box
assert enabled_button.box.get_fill_color() == manimlib.RED

failed_enable_disable = interactive.EnableDisableButton.__new__(
    interactive.EnableDisableButton
)
try:
    interactive.EnableDisableButton.__init__(
        failed_enable_disable,
        unsupported=True,
    )
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: unsupported"
else:
    raise AssertionError(
        "EnableDisableButton silently discarded an unknown option"
    )
assert not hasattr(failed_enable_disable, "submobjects")

translucent_toggle = interactive.EnableDisableButton(
    True,
    rect_kwargs=dict(width=0.5, height=0.5, fill_opacity=0.4),
)
assert np.isclose(translucent_toggle.box.get_fill_opacity(), 0.4)
toggle_box = translucent_toggle.box
translucent_toggle.toggle_value()
assert translucent_toggle.box is toggle_box
assert np.isclose(translucent_toggle.box.get_fill_opacity(), 0.4)
assert translucent_toggle.box.get_fill_color() == manimlib.RED

custom_toggle = interactive.EnableDisableButton(
    True,
    enable_color=manimlib.BLUE,
    disable_color=manimlib.YELLOW,
)
assert bool(custom_toggle.get_value()) is True
assert custom_toggle.box.get_fill_color() == manimlib.BLUE
custom_toggle_box = custom_toggle.box
custom_toggle.toggle_value()
assert bool(custom_toggle.get_value()) is False
assert custom_toggle.box is custom_toggle_box
assert custom_toggle.box.get_fill_color() == manimlib.YELLOW
try:
    interactive.EnableDisableButton(True, enable_color=object())
except TypeError as error:
    assert str(error)
else:
    raise AssertionError("EnableDisableButton accepted a non-color object")

# LinearNumberSlider is a distinct interactive control from number_line.Slider.
# Atlas owns its bar/handle/axis geometry; the portal keeps the tracker root,
# Reference construction-time midpoint quirk, kwargs, and named drag gap.
linear_slider = interactive.LinearNumberSlider(
    6.0,
    min_value=0.0,
    max_value=10.0,
    step=0.5,
    rounded_rect_kwargs={
        "height": 0.1,
        "width": 3.0,
        "corner_radius": 0.0375,
    },
    circle_kwargs={
        "radius": 0.1,
        "stroke_color": manimlib.BLUE,
        "fill_color": manimlib.BLUE,
        "fill_opacity": 1.0,
    },
    name="native-linear-slider",
)
assert interactive.LinearNumberSlider.__bases__ == (
    interactive.ControlMobject,
)
linear_signature = inspect.signature(interactive.LinearNumberSlider)
assert list(linear_signature.parameters) == [
    "value",
    "value_type",
    "min_value",
    "max_value",
    "step",
    "rounded_rect_kwargs",
    "circle_kwargs",
    "kwargs",
]
assert (
    linear_signature.parameters["kwargs"].kind
    is inspect.Parameter.VAR_KEYWORD
)
assert linear_slider.get_value() == np.float64(6.0)
assert linear_slider.min_value == 0.0
assert linear_slider.max_value == 10.0
assert linear_slider.step == 0.5
assert list(linear_slider.submobjects) == [
    linear_slider.bar,
    linear_slider.slider,
    linear_slider.slider_axis,
]
assert linear_slider.name == "native-linear-slider"
assert linear_slider.is_fixed_in_frame()
assert linear_slider.bar.is_fixed_in_frame()
assert linear_slider.slider.is_fixed_in_frame()
assert linear_slider.slider_axis.is_fixed_in_frame()
assert np.allclose(linear_slider.slider.get_center(), [0.0, 0.0, 0.0])
assert linear_slider.slider_axis.get_opacity() == 0.0
assert str(
    inspect.signature(interactive.LinearNumberSlider.slider_on_mouse_drag)
) == "(self, mob, event_data)"
linear_handle = linear_slider.slider
linear_bar = linear_slider.bar
assert linear_slider.slider_on_mouse_drag(
    linear_handle,
    {"point": np.array([0.37, 1.0, 0.0])},
) is False
assert linear_slider.get_value() == np.float64(6.5)
assert np.allclose(
    linear_handle.get_center(),
    linear_slider.slider_axis.point_from_proportion(0.65),
)
assert linear_slider.slider is linear_handle
assert linear_slider.bar is linear_bar
assert linear_slider.slider_on_mouse_drag(
    linear_handle,
    {"point": np.array([10.0, 0.0, 0.0])},
) is False
assert linear_slider.get_value() == np.float64(10.0)
assert np.allclose(
    linear_handle.get_center(),
    linear_slider.slider_axis.get_end(),
)
assert str(
    inspect.signature(interactive.LinearNumberSlider.get_value_from_point)
) == "(self, point)"
assert linear_slider.get_value_from_point(np.array([0.37, 1.0, 0.0])) == 6.5
assert linear_slider.set_value(4.0) is linear_slider
assert linear_slider.get_value() == np.float64(4.0)
assert np.allclose(
    linear_handle.get_center(),
    linear_slider.slider_axis.point_from_proportion(0.4),
)
assert linear_slider.slider is linear_handle
try:
    linear_slider.set_value(11.0)
except AssertionError:
    pass
else:
    raise AssertionError("LinearNumberSlider accepted a value above max")
assert linear_slider.get_value() == np.float64(4.0)

custom_linear = interactive.LinearNumberSlider(
    0.0,
    rounded_rect_kwargs={
        "height": 0.075,
        "width": 2.0,
        "corner_radius": 0.02,
    },
    circle_kwargs={
        "radius": 0.2,
        "stroke_color": manimlib.GREY_A,
        "fill_color": manimlib.GREY_A,
        "fill_opacity": 0.4,
    },
)
assert np.isclose(custom_linear.slider.get_width(), 0.4)
assert np.isclose(custom_linear.slider.get_fill_opacity(), 0.4)

split_handle = interactive.LinearNumberSlider(
    0.0,
    circle_kwargs={
        "radius": 0.1,
        "stroke_color": manimlib.RED,
        "fill_color": manimlib.GREEN,
        "fill_opacity": 1.0,
    },
)
assert split_handle.slider.get_stroke_color() == manimlib.RED
assert split_handle.slider.get_fill_color() == manimlib.GREEN

# fm-5wq.4: the signature is re-pinned against the live constructor, the
# constructor keeps an off-step value exactly (range assert only, no build
# snap), and the drag path is where step snapping lives —
# get_value_from_point ceils to the NEXT step boundary, so value 3 with
# step 2 snaps to 4, and an on-step point stays put.
assert list(
    inspect.signature(interactive.LinearNumberSlider).parameters
) == [
    "value",
    "value_type",
    "min_value",
    "max_value",
    "step",
    "rounded_rect_kwargs",
    "circle_kwargs",
    "kwargs",
]
step_slider = interactive.LinearNumberSlider(
    3, min_value=0, max_value=10, step=2
)
assert float(step_slider.get_value()) == 3.0
step_off_point = step_slider.slider_axis.point_from_proportion(0.3)
assert step_slider.get_value_from_point(step_off_point) == 4.0
step_on_point = step_slider.slider_axis.point_from_proportion(0.4)
assert step_slider.get_value_from_point(step_on_point) == 4.0
assert step_slider.slider_on_mouse_drag(
    step_slider.slider, {"point": step_off_point}
) is False
assert float(step_slider.get_value()) == 4.0

# Planted negative: an inverted range is the native slider's named refusal,
# never a silent build.
try:
    interactive.LinearNumberSlider(0.0, min_value=5.0, max_value=-5.0)
except ValueError as error:
    assert "slider bounds must be finite, ordered" in str(error)
else:
    raise AssertionError("LinearNumberSlider accepted an inverted range")

# ColorSliders is the Reference's Group, not a ControlMobject. Atlas owns its
# checkerboard/swatch and the four native LinearNumberSlider compositions.
color_sliders = interactive.ColorSliders()
assert interactive.ColorSliders.__bases__ == (manimlib.Group,)
assert interactive.ControlMobject not in interactive.ColorSliders.__mro__
assert list(color_sliders.submobjects) == [
    color_sliders.swatch,
    color_sliders.sliders,
]
assert list(color_sliders.swatch.submobjects) == [
    color_sliders.background,
    color_sliders.selected_color_box,
]
assert list(color_sliders.sliders.submobjects) == [
    color_sliders.r_slider,
    color_sliders.g_slider,
    color_sliders.b_slider,
    color_sliders.a_slider,
]
assert all(len(slider.submobjects) == 3 for slider in color_sliders.sliders)
assert np.allclose(color_sliders.get_value(), [1.0, 1.0, 1.0, 1.0])

custom_grid_color_sliders = interactive.ColorSliders(
    background_grid_kwargs=dict(
        single_square_len=0.2,
        colors=[manimlib.RED, manimlib.BLUE],
    )
)
custom_grid_squares = list(custom_grid_color_sliders.background.submobjects)
assert len(custom_grid_squares) == 2 * 11
assert np.isclose(
    custom_grid_color_sliders.background.get_width(),
    custom_grid_color_sliders.selected_color_box.get_width(),
    atol=1e-3,
)
assert np.isclose(
    custom_grid_squares[0].get_width(),
    custom_grid_color_sliders.background.get_width() / 11.0,
    atol=1e-3,
)
assert custom_grid_squares[0].get_fill_color() == manimlib.RED
assert custom_grid_squares[1].get_fill_color() == manimlib.BLUE
try:
    interactive.ColorSliders(background_grid_kwargs=dict(bogus=1))
except TypeError as error:
    assert "background_grid_kwargs.bogus" in str(error)
else:
    raise AssertionError(
        "ColorSliders silently discarded background_grid_kwargs.bogus"
    )
default_grid_squares = list(color_sliders.background.submobjects)
assert len(default_grid_squares) == 5 * 21
assert color_sliders.background_grid_kwargs["single_square_len"] == 0.1
assert default_grid_squares[0].get_fill_color() == manimlib.GREY_A
assert default_grid_squares[1].get_fill_color() == manimlib.GREY_C
empty_slider_config = interactive.ColorSliders(sliders_kwargs={})
assert np.allclose(empty_slider_config.get_value(), [1.0, 1.0, 1.0, 1.0])
stepped_color_sliders = interactive.ColorSliders(
    sliders_kwargs=dict(step=2.0)
)
stepped_color_sliders.set_value(63.0, 128.0, 254.0, 0.5)
assert np.allclose(
    stepped_color_sliders.get_value(),
    [62.0 / 255.0, 128.0 / 255.0, 254.0 / 255.0, 0.5],
)
styled_color_sliders = interactive.ColorSliders(
    sliders_kwargs=dict(
        rounded_rect_kwargs=dict(width=1.0, height=0.1, corner_radius=0.02),
        circle_kwargs=dict(radius=0.2, fill_opacity=0.4),
    )
)
styled_bar, styled_handle, _ = styled_color_sliders.r_slider.submobjects
assert np.isclose(styled_bar.get_width(), 1.0)
assert np.isclose(styled_handle.get_width(), 0.4)
assert np.isclose(styled_handle.get_fill_opacity(), 0.4)
try:
    interactive.ColorSliders(
        sliders_kwargs=dict(rounded_rect_kwargs=dict(foo=1))
    )
except TypeError as error:
    assert "sliders_kwargs.rounded_rect_kwargs.foo" in str(error)
else:
    raise AssertionError(
        "ColorSliders silently discarded sliders_kwargs.rounded_rect_kwargs.foo"
    )
try:
    interactive.ColorSliders(sliders_kwargs=dict(bogus=1))
except TypeError as error:
    assert "sliders_kwargs.bogus" in str(error)
else:
    raise AssertionError("ColorSliders silently discarded sliders_kwargs.bogus")

# fm-5wq.4: shared sliders_kwargs min_value/max_value ride the native
# slider_overrides range onto all four sliders. RGB set_value clamps into
# the shared [0, 1] range (the Reference's assert_value is the port's
# documented clamp) while get_value keeps its fixed /255 RGB
# normalization; alpha's default range is already [0, 1], so the shared
# range leaves alpha behavior unchanged. An unordered range is the named
# native SliderError surfaced as ValueError — no hang, no silent swap.
ranged_color_sliders = interactive.ColorSliders(
    sliders_kwargs=dict(min_value=0, max_value=1)
)
ranged_color_sliders.set_value(2.0, 0.5, 300.0, 1.5)
assert np.allclose(
    ranged_color_sliders.get_value(),
    [1.0 / 255.0, 0.5 / 255.0, 1.0 / 255.0, 1.0],
)
try:
    interactive.ColorSliders(
        sliders_kwargs=dict(min_value=1.0, max_value=0.0)
    )
except ValueError as error:
    assert "ordered" in str(error), error
else:
    raise AssertionError(
        "ColorSliders accepted an unordered min_value/max_value range"
    )

# fm-5wq.4: sliders_kwargs.value_type coerces every set_value component
# through the numpy scalar type before it reaches the native sliders —
# int truncates (numpy int64 cast), including alpha. A non-numeric
# value_type (object) and a non-dtype string are named TypeErrors at
# construction, not hangs.
int_color_sliders = interactive.ColorSliders(
    sliders_kwargs=dict(value_type=int)
)
int_color_sliders.set_value(64.9, 128.2, 255.0, 0.9)
assert np.allclose(
    int_color_sliders.get_value(),
    [64.0 / 255.0, 128.0 / 255.0, 1.0, 0.0],
)
try:
    interactive.ColorSliders(sliders_kwargs=dict(value_type=object))
except TypeError as error:
    assert "value_type" in str(error), error
else:
    raise AssertionError(
        "ColorSliders accepted a non-numeric value_type"
    )
try:
    interactive.ColorSliders(sliders_kwargs=dict(value_type="bogus"))
except TypeError as error:
    assert "bogus" in str(error), error
else:
    raise AssertionError("ColorSliders accepted a non-dtype value_type")

assert str(inspect.signature(interactive.ColorSliders.get_picked_color)) == (
    "(self)"
)
assert str(inspect.signature(interactive.ColorSliders.get_picked_opacity)) == (
    "(self)"
)
assert color_sliders.get_picked_opacity() == 1.0
assert np.allclose(
    manimlib.color_to_rgb(color_sliders.get_picked_color()),
    [1.0, 1.0, 1.0],
)
swatch_identity = color_sliders.swatch
sliders_identity = color_sliders.sliders
color_sliders.set_value(64.0, 128.0, 255.0, 0.5)
assert color_sliders.swatch is swatch_identity
assert color_sliders.sliders is sliders_identity
assert np.allclose(
    color_sliders.get_value(),
    [64.0 / 255.0, 128.0 / 255.0, 1.0, 0.5],
)
assert color_sliders.get_picked_opacity() == 0.5
assert np.allclose(
    manimlib.color_to_rgb(color_sliders.get_picked_color()),
    [64.0 / 255.0, 128.0 / 255.0, 1.0],
)
assert np.allclose(
    manimlib.color_to_rgb(color_sliders.selected_color_box.get_fill_color()),
    [64.0 / 255.0, 128.0 / 255.0, 1.0],
    atol=1.0 / 255.0,
)
assert np.isclose(
    color_sliders.selected_color_box.get_fill_opacity(),
    0.5,
)

custom_rect_color_sliders = interactive.ColorSliders(
    rect_kwargs=dict(width=1.0, height=0.25)
)
assert np.isclose(
    custom_rect_color_sliders.selected_color_box.get_width(),
    1.0,
    atol=1e-3,
)
assert np.isclose(
    custom_rect_color_sliders.selected_color_box.get_height(),
    0.25,
    atol=1e-3,
)
assert np.isclose(color_sliders.selected_color_box.get_stroke_opacity(), 1.0)
custom_rect_stroke = interactive.ColorSliders(
    rect_kwargs=dict(width=2.0, height=0.5, stroke_opacity=0.3)
)
assert np.isclose(
    custom_rect_stroke.selected_color_box.get_stroke_opacity(),
    0.3,
)
custom_rect_stroke.set_value(64.0, 128.0, 255.0, 0.5)
assert np.isclose(
    custom_rect_stroke.selected_color_box.get_stroke_opacity(),
    0.3,
)
try:
    interactive.ColorSliders(rect_kwargs=dict(bogus=1))
except TypeError as error:
    assert "bogus" in str(error)
else:
    raise AssertionError("ColorSliders silently discarded rect_kwargs.bogus")

custom_color_sliders = interactive.ColorSliders(
    default_rgb_value=64,
    default_a_value=0.4,
)
assert np.allclose(
    custom_color_sliders.get_value(),
    [64.0 / 255.0, 64.0 / 255.0, 64.0 / 255.0, 0.4],
)
assert np.allclose(
    manimlib.color_to_rgb(
        custom_color_sliders.selected_color_box.get_fill_color()
    ),
    [64.0 / 255.0, 64.0 / 255.0, 64.0 / 255.0],
    atol=1.0 / 255.0,
)
assert np.isclose(
    custom_color_sliders.selected_color_box.get_fill_opacity(),
    0.4,
)

failed_color_sliders = interactive.ColorSliders.__new__(
    interactive.ColorSliders
)
try:
    interactive.ColorSliders.__init__(
        failed_color_sliders,
        unsupported=True,
    )
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: unsupported"
else:
    raise AssertionError("ColorSliders silently discarded an unknown option")
assert not hasattr(failed_color_sliders, "submobjects")

# Textbox keeps the ControlMobject lineage while Atlas and the bundled
# FontBook own each string layout. Updates commit into the existing text proxy
# only after the full native candidate succeeds.
textbox = interactive.Textbox("seed", isInitiallyActive=True)
assert interactive.Textbox.__bases__ == (interactive.ControlMobject,)
assert interactive.ControlMobject in interactive.Textbox.__mro__
assert manimlib.ValueTracker in interactive.Textbox.__mro__
assert list(textbox.submobjects) == [textbox.box, textbox.text]
assert textbox.get_value() == "seed"
assert textbox.isActive is True
assert textbox.is_fixed_in_frame()
assert textbox.box.is_fixed_in_frame()
assert textbox.text.is_fixed_in_frame()
assert len(textbox.updaters) == 1
assert len(textbox.text.updaters) == 1

text_proxy = textbox.text
seed_points = text_proxy.get_all_points().copy()
assert textbox.set_value("committed") is textbox
assert textbox.get_value() == "committed"
assert textbox.text is text_proxy
committed_points = text_proxy.get_all_points().copy()
assert not np.array_equal(committed_points, seed_points)

assert textbox.update_text("preview") is None
assert textbox.get_value() == "committed"
assert textbox.text is text_proxy
preview_points = text_proxy.get_all_points().copy()
assert not np.array_equal(preview_points, committed_points)

atomic_value = textbox.get_value()
atomic_points = textbox.text.get_all_points().copy()
atomic_children = list(textbox.submobjects)
try:
    textbox.set_value("\U0001f980")
except ValueError as error:
    assert "unmapped" in str(error).lower(), error
else:
    raise AssertionError("Textbox accepted an unmapped glyph")
assert textbox.get_value() == atomic_value
assert textbox.text is text_proxy
assert list(textbox.submobjects) == atomic_children
assert np.array_equal(textbox.text.get_all_points(), atomic_points)

failed_textbox = interactive.Textbox.__new__(interactive.Textbox)
try:
    interactive.Textbox.__init__(failed_textbox, unsupported=True)
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: unsupported"
else:
    raise AssertionError("Textbox silently discarded an unknown option")
assert not hasattr(failed_textbox, "submobjects")

assert str(inspect.signature(interactive.Textbox.box_on_mouse_press)) == (
    "(self, mob, event_data)"
)
textbox_box = textbox.box
textbox_text = textbox.text
assert textbox.isActive is True
assert textbox.box.get_stroke_color() == manimlib.BLUE
assert textbox.box_on_mouse_press(
    textbox.box, {"point": np.zeros(3)}
) is False
assert textbox.isActive is False
assert textbox.box is textbox_box
assert textbox.text is textbox_text
assert textbox.get_value() == "committed"
assert textbox.box.get_stroke_color() == manimlib.RED
assert textbox.box_on_mouse_press(
    textbox.box, {"point": np.zeros(3)}
) is False
assert textbox.isActive is True
assert textbox.box.get_stroke_color() == manimlib.BLUE
assert str(inspect.signature(interactive.Textbox.active_anim)) == (
    "(self, isActive)"
)
textbox.active_anim(False)
assert textbox.isActive is True
assert textbox.box.get_stroke_color() == manimlib.RED
textbox.active_anim(True)
assert textbox.isActive is True
assert textbox.box.get_stroke_color() == manimlib.BLUE
assert str(inspect.signature(interactive.Textbox.on_key_press)) == (
    "(self, mob, event_data)"
)
assert textbox.on_key_press(
    textbox, {"symbol": ord("a"), "modifiers": 0}
) is False
assert textbox.get_value() == "committeda"
assert textbox.text is textbox_text
assert textbox.on_key_press(
    textbox, {"symbol": ord("b"), "modifiers": 1}
) is False
assert textbox.get_value() == "committedaB"
assert textbox.on_key_press(
    textbox, {"symbol": 0x020, "modifiers": 0}
) is False
assert textbox.get_value() == "committedaB "
assert textbox.on_key_press(
    textbox, {"symbol": 0xFF09, "modifiers": 0}
) is False
assert textbox.get_value() == "committedaB \t"
assert textbox.on_key_press(
    textbox, {"symbol": 0xFF08, "modifiers": 0}
) is False
assert textbox.get_value() == "committedaB "
assert textbox.box_on_mouse_press(
    textbox.box, {"point": np.zeros(3)}
) is False
assert textbox.isActive is False
assert textbox.on_key_press(
    textbox, {"symbol": ord("x"), "modifiers": 0}
) is None
assert textbox.get_value() == "committedaB "
assert textbox.box is textbox_box
assert textbox.text is textbox_text

styled_textbox = interactive.Textbox(
    "hi",
    box_kwargs=dict(
        width=3.0,
        height=1.5,
        fill_color=manimlib.GREEN,
        fill_opacity=0.4,
    ),
    text_kwargs=dict(color=manimlib.RED),
    text_buff=0.1,
    active_color=manimlib.WHITE,
    deactive_color=manimlib.GREEN,
)
assert np.isclose(styled_textbox.box.get_width(), 3.0)
assert np.isclose(styled_textbox.box.get_height(), 1.5)
assert styled_textbox.box.get_fill_color() == manimlib.GREEN
assert np.isclose(styled_textbox.box.get_fill_opacity(), 0.4)
assert styled_textbox.box.get_stroke_color() == manimlib.GREEN
assert styled_textbox.text.get_fill_color() == manimlib.RED
styled_box = styled_textbox.box
styled_textbox.set_value("ok")
assert styled_textbox.box is styled_box
assert styled_textbox.box.get_fill_color() == manimlib.GREEN
assert styled_textbox.text.get_fill_color() == manimlib.RED

# ControlPanel is the remaining interactive Group: Atlas ingests the complete
# variadic control list, lays out native extent targets, and the portal grafts
# the original control proxies into that native controls column.
panel_checkbox = interactive.Checkbox(True)
panel_toggle = interactive.EnableDisableButton(False)
control_panel = interactive.ControlPanel(panel_checkbox, panel_toggle)
assert interactive.ControlPanel.__bases__ == (manimlib.Group,)
assert interactive.ControlMobject not in interactive.ControlPanel.__mro__
assert list(control_panel.submobjects) == [
    control_panel.panel,
    control_panel.panel_opener,
    control_panel.controls,
]
assert len(control_panel.panel_opener.submobjects) == 2
assert list(control_panel.controls.submobjects) == [
    panel_checkbox,
    panel_toggle,
]
assert control_panel.controls.submobjects[0] is panel_checkbox
assert control_panel.controls.submobjects[1] is panel_toggle
assert panel_checkbox.get_center()[1] > panel_toggle.get_center()[1]
assert np.isclose(
    control_panel.panel.get_width(),
    manimlib.FRAME_WIDTH / 4.0,
)
assert np.isclose(
    control_panel.panel_opener.get_width(),
    manimlib.FRAME_WIDTH / 8.0,
)

panel_identity = control_panel.panel
opener_identity = control_panel.panel_opener
controls_identity = control_panel.controls
control_panel.open_panel()
assert control_panel.panel is panel_identity
assert control_panel.panel_opener is opener_identity
assert control_panel.controls is controls_identity
assert list(control_panel.controls.submobjects) == [
    panel_checkbox,
    panel_toggle,
]
assert np.isclose(
    control_panel.panel_opener.get_bottom()[1],
    -manimlib.FRAME_Y_RADIUS,
    atol=1e-6,
)
control_panel.close_panel()
assert np.isclose(
    control_panel.panel_opener.get_top()[1],
    manimlib.FRAME_Y_RADIUS,
    atol=1e-6,
)

# The variadic mutation methods preserve the live panel/opener/controls shells
# while Atlas's ControlPanelMobject::layout_against_opener supplies every new
# target. Like the Reference, these mutators return None.
assert str(inspect.signature(interactive.ControlPanel.add_controls)) == (
    "(self, *new_controls)"
)
assert str(inspect.signature(interactive.ControlPanel.remove_controls)) == (
    "(self, *controls_to_remove)"
)
control_panel.open_panel()
panel_textbox = interactive.Textbox("native")
assert control_panel.add_controls(panel_textbox) is None
assert control_panel.panel is panel_identity
assert control_panel.panel_opener is opener_identity
assert control_panel.controls is controls_identity
assert list(control_panel.controls.submobjects) == [
    panel_checkbox,
    panel_toggle,
    panel_textbox,
]
assert control_panel.controls.submobjects[2] is panel_textbox
assert np.isclose(
    control_panel.panel.get_bottom()[1],
    control_panel.panel_opener.submobjects[0].get_top()[1],
    atol=1e-6,
)
assert np.isclose(
    control_panel.controls.get_bottom()[1],
    control_panel.panel_opener.submobjects[0].get_top()[1]
    + manimlib.MED_SMALL_BUFF,
    atol=1e-6,
)
assert control_panel.remove_controls(panel_toggle) is None
assert control_panel.panel is panel_identity
assert control_panel.panel_opener is opener_identity
assert control_panel.controls is controls_identity
assert list(control_panel.controls.submobjects) == [
    panel_checkbox,
    panel_textbox,
]
assert control_panel.controls.submobjects[0] is panel_checkbox
assert control_panel.controls.submobjects[1] is panel_textbox
assert np.isclose(
    control_panel.controls.get_bottom()[1],
    control_panel.panel_opener.submobjects[0].get_top()[1]
    + manimlib.MED_SMALL_BUFF,
    atol=1e-6,
)
assert str(
    inspect.signature(
        interactive.ControlPanel.move_panel_and_controls_to_panel_opener
    )
) == "(self)"
assert str(
    inspect.signature(interactive.ControlPanel.panel_opener_on_mouse_drag)
) == "(self, mob, event_data)"
assert str(
    inspect.signature(interactive.ControlPanel.panel_on_mouse_scroll)
) == "(self, mob, event_data)"
drag_y = control_panel.panel_opener.get_y() - 1.25
assert control_panel.panel_opener_on_mouse_drag(
    control_panel.panel_opener,
    {"point": np.array([0.0, drag_y, 0.0])},
) is False
assert control_panel.panel is panel_identity
assert control_panel.panel_opener is opener_identity
assert control_panel.controls is controls_identity
assert np.isclose(control_panel.panel_opener.get_y(), drag_y, atol=1e-6)
assert np.isclose(
    control_panel.panel.get_bottom()[1],
    control_panel.panel_opener.submobjects[0].get_top()[1],
    atol=1e-6,
)
assert np.isclose(
    control_panel.controls.get_bottom()[1],
    control_panel.panel_opener.submobjects[0].get_top()[1]
    + manimlib.MED_SMALL_BUFF,
    atol=1e-6,
)
controls_y = control_panel.controls.get_y()
assert control_panel.panel_on_mouse_scroll(
    control_panel.panel,
    {"offset": np.array([0.0, 0.2, 0.0])},
) is False
assert control_panel.controls is controls_identity
assert np.isclose(control_panel.controls.get_y(), controls_y + 2.0, atol=1e-6)
try:
    control_panel.add_controls(geometry.Circle())
except TypeError as error:
    assert str(error) == (
        "ControlPanel controls must be ControlMobject instances"
    )
else:
    raise AssertionError("ControlPanel.add_controls accepted a non-control")
assert list(control_panel.controls.submobjects) == [
    panel_checkbox,
    panel_textbox,
]

try:
    interactive.ControlPanel(geometry.Circle())
except TypeError as error:
    assert str(error) == (
        "ControlPanel controls must be ControlMobject instances"
    )
else:
    raise AssertionError("ControlPanel accepted a non-control child")

failed_control_panel = interactive.ControlPanel.__new__(
    interactive.ControlPanel
)
try:
    interactive.ControlPanel.__init__(
        failed_control_panel,
        unsupported=True,
    )
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: unsupported"
else:
    raise AssertionError("ControlPanel silently discarded an unknown option")
assert not hasattr(failed_control_panel, "submobjects")

# panel_kwargs / opener_kwargs ride Atlas's native panel and opener builders
# through construction and the open/close rebuild, so custom size and fill
# survive the same become() identity path as the default GREY_C panel.
styled_panel_checkbox = interactive.Checkbox(True)
styled_control_panel = interactive.ControlPanel(
    styled_panel_checkbox,
    panel_kwargs=dict(
        width=3.0,
        height=2.0,
        fill_color=manimlib.BLUE,
        fill_opacity=0.4,
        stroke_width=2.0,
    ),
    opener_kwargs=dict(
        width=1.5,
        height=0.75,
        fill_color=manimlib.GREEN,
        fill_opacity=0.8,
    ),
)
assert np.isclose(styled_control_panel.panel.get_width(), 3.0)
assert np.isclose(styled_control_panel.panel.get_height(), 2.0)
assert styled_control_panel.panel.get_fill_color() == manimlib.BLUE
assert np.isclose(styled_control_panel.panel.get_fill_opacity(), 0.4)
assert np.isclose(styled_control_panel.panel.get_stroke_width(), 2.0)
styled_opener_rect = styled_control_panel.panel_opener.submobjects[0]
assert np.isclose(styled_opener_rect.get_width(), 1.5)
assert np.isclose(styled_opener_rect.get_height(), 0.75)
assert styled_opener_rect.get_fill_color() == manimlib.GREEN
assert np.isclose(styled_opener_rect.get_fill_opacity(), 0.8)
styled_panel_identity = styled_control_panel.panel
styled_opener_identity = styled_control_panel.panel_opener
styled_control_panel.open_panel()
assert styled_control_panel.panel is styled_panel_identity
assert styled_control_panel.panel_opener is styled_opener_identity
assert np.isclose(styled_control_panel.panel.get_width(), 3.0)
assert styled_control_panel.panel.get_fill_color() == manimlib.BLUE
assert np.isclose(
    styled_control_panel.panel_opener.submobjects[0].get_width(),
    1.5,
)
assert styled_control_panel.panel_opener.submobjects[0].get_fill_color() == (
    manimlib.GREEN
)
styled_control_panel.close_panel()
assert styled_control_panel.panel is styled_panel_identity
assert np.isclose(styled_control_panel.panel.get_height(), 2.0)
assert np.isclose(styled_control_panel.panel.get_fill_opacity(), 0.4)
assert styled_control_panel.add_controls(
    interactive.EnableDisableButton(True)
) is None
assert styled_control_panel.panel is styled_panel_identity
assert np.isclose(styled_control_panel.panel.get_width(), 3.0)
assert styled_control_panel.panel.get_fill_color() == manimlib.BLUE
assert np.isclose(styled_control_panel.panel.get_fill_opacity(), 0.4)

# opener_text_kwargs ride Atlas's opener Text builder (text + font_size).
labeled_panel = interactive.ControlPanel(
    interactive.Checkbox(True),
    opener_text_kwargs=dict(text="Settings", font_size=32),
)
assert labeled_panel.opener_text_kwargs["text"] == "Settings"
assert np.isclose(labeled_panel.opener_text_kwargs["font_size"], 32)
assert len(labeled_panel.panel_opener.submobjects) == 2
default_opener_text = control_panel.panel_opener.submobjects[1]
labeled_opener_text = labeled_panel.panel_opener.submobjects[1]
assert labeled_opener_text.get_height() > default_opener_text.get_height()
labeled_opener_identity = labeled_panel.panel_opener
labeled_panel.open_panel()
assert labeled_panel.panel_opener is labeled_opener_identity
assert labeled_panel.panel_opener.submobjects[1] is labeled_opener_text
try:
    interactive.ControlPanel(opener_text_kwargs=dict(font="Comic Sans"))
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: opener_text_kwargs.font"
else:
    raise AssertionError(
        "ControlPanel silently accepted unknown opener_text_kwargs"
    )

# opener_text_kwargs colour rides Atlas Text (color, fill_color wins).
colored_opener_panel = interactive.ControlPanel(
    interactive.Checkbox(True),
    opener_text_kwargs=dict(text="Settings", font_size=32, color=manimlib.RED),
)
assert colored_opener_panel.panel_opener.submobjects[1].get_fill_color() == (
    manimlib.RED
)
fill_opener_panel = interactive.ControlPanel(
    interactive.Checkbox(True),
    opener_text_kwargs=dict(
        text="Settings",
        color=manimlib.RED,
        fill_color=manimlib.BLUE,
    ),
)
assert fill_opener_panel.panel_opener.submobjects[1].get_fill_color() == (
    manimlib.BLUE
)
colored_opener_identity = colored_opener_panel.panel_opener
colored_opener_text = colored_opener_panel.panel_opener.submobjects[1]
colored_opener_panel.open_panel()
assert colored_opener_panel.panel_opener is colored_opener_identity
assert colored_opener_panel.panel_opener.submobjects[1] is colored_opener_text
assert colored_opener_text.get_fill_color() == manimlib.RED

# Remaining Rectangle constructor keys on panel_kwargs / opener_kwargs ride
# Atlas stroke colour/width/opacity. add_controls/remove_controls keep the
# constructor stroke (matcher layout is unstyled extents; shift-only).
stroked_panel = interactive.ControlPanel(
    interactive.Checkbox(True),
    panel_kwargs=dict(
        width=3.0,
        height=2.0,
        fill_color=manimlib.BLUE,
        stroke_color=manimlib.RED,
        stroke_width=3.0,
        stroke_opacity=0.4,
    ),
    opener_kwargs=dict(
        width=1.5,
        height=0.75,
        fill_color=manimlib.GREEN,
        stroke_color=manimlib.YELLOW,
        stroke_width=2.0,
        stroke_opacity=0.6,
    ),
)
assert stroked_panel.panel.get_stroke_color() == manimlib.RED
assert np.isclose(stroked_panel.panel.get_stroke_width(), 3.0)
assert np.isclose(stroked_panel.panel.get_stroke_opacity(), 0.4)
stroked_opener_rect = stroked_panel.panel_opener.submobjects[0]
assert stroked_opener_rect.get_stroke_color() == manimlib.YELLOW
assert np.isclose(stroked_opener_rect.get_stroke_width(), 2.0)
assert np.isclose(stroked_opener_rect.get_stroke_opacity(), 0.6)
stroked_panel_identity = stroked_panel.panel
assert stroked_panel.add_controls(interactive.EnableDisableButton(True)) is None
assert stroked_panel.panel is stroked_panel_identity
assert stroked_panel.panel.get_stroke_color() == manimlib.RED
assert np.isclose(stroked_panel.panel.get_stroke_width(), 3.0)
assert np.isclose(stroked_panel.panel.get_stroke_opacity(), 0.4)
assert stroked_panel.remove_controls(stroked_panel.controls.submobjects[0]) is None
assert stroked_panel.panel is stroked_panel_identity
assert stroked_panel.panel.get_stroke_color() == manimlib.RED
assert np.isclose(stroked_panel.panel.get_stroke_opacity(), 0.4)
assert stroked_panel.panel_opener.submobjects[0].get_stroke_color() == (
    manimlib.YELLOW
)
try:
    interactive.ControlPanel(panel_kwargs=dict(bogus=1))
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: panel_kwargs.bogus"
else:
    raise AssertionError("ControlPanel silently discarded panel_kwargs.bogus")
try:
    interactive.ControlPanel(opener_kwargs=dict(bogus=1))
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: opener_kwargs.bogus"
else:
    raise AssertionError("ControlPanel silently discarded opener_kwargs.bogus")

# VMobject color= on panel_kwargs / opener_kwargs fills and strokes unless
# fill_color / stroke_color win. add_controls keeps the constructor fill.
color_panel = interactive.ControlPanel(
    interactive.Checkbox(True),
    panel_kwargs=dict(color=manimlib.RED, stroke_width=2.0),
    opener_kwargs=dict(color=manimlib.BLUE, stroke_width=1.0),
)
assert color_panel.panel.get_fill_color() == manimlib.RED
assert color_panel.panel.get_stroke_color() == manimlib.RED
assert color_panel.panel_opener.submobjects[0].get_fill_color() == manimlib.BLUE
assert color_panel.panel_opener.submobjects[0].get_stroke_color() == (
    manimlib.BLUE
)
color_panel_identity = color_panel.panel
assert color_panel.add_controls(interactive.EnableDisableButton(True)) is None
assert color_panel.panel is color_panel_identity
assert color_panel.panel.get_fill_color() == manimlib.RED
assert color_panel.panel.get_stroke_color() == manimlib.RED
assert color_panel.remove_controls(color_panel.controls.submobjects[-1]) is None
assert color_panel.panel.get_fill_color() == manimlib.RED
override_color_panel = interactive.ControlPanel(
    panel_kwargs=dict(
        color=manimlib.RED,
        fill_color=manimlib.GREEN,
        stroke_color=manimlib.YELLOW,
        stroke_width=2.0,
    ),
)
assert override_color_panel.panel.get_fill_color() == manimlib.GREEN
assert override_color_panel.panel.get_stroke_color() == manimlib.YELLOW

# PGroup is Atlas's native point-cloud family root, with the original live
# PMobject proxies grafted beneath it in Reference order.
point_cloud_mobjects = importlib.import_module(
    "manimlib.mobject.types.point_cloud_mobject"
)
assert point_cloud_mobjects.PGroup.__bases__ == (point_cloud_mobjects.PMobject,)
assert str(inspect.signature(point_cloud_mobjects.PGroup)) == (
    "(*pmobs, **kwargs)"
)
pgroup_left = manimlib.DotCloud([(-1.0, 0.0, 0.0)])
pgroup_right = manimlib.DotCloud([(1.0, 0.0, 0.0)])
pgroup = point_cloud_mobjects.PGroup(pgroup_left, pgroup_right, pgroup_left)
assert list(pgroup.submobjects) == [pgroup_left, pgroup_right]
assert pgroup.submobjects[0] is pgroup_left
assert pgroup.submobjects[1] is pgroup_right
# fm-5wq.20: the Reference's Mobject.__init__ takes a fixed signature, so an
# unknown keyword is Python's TypeError, not a stored attribute. A keyword the
# Reference accepts but the native builder does not route yet still refuses
# by name.
for unknown_keyword_case in (
    lambda: point_cloud_mobjects.PGroup(pgroup_left, marker="native"),
    lambda: Mobject(shape=1),
    lambda: VMobject(shape=1),
    lambda: manimlib.Group(shape=1),
    lambda: manimlib.VGroup(shape=1),
    lambda: manimlib.SGroup(shape=(8, 8)),
):
    try:
        unknown_keyword_case()
    except TypeError as error:
        assert str(error).startswith("Mobject.__init__() got an unexpected keyword argument"), error
    else:
        raise AssertionError("an unknown Mobject keyword was accepted")
try:
    manimlib.SurfaceMesh(manimlib.Sphere(), color=manimlib.BLUE)
except NotImplementedError as error:
    assert "not yet routed" in str(error), error
else:
    raise AssertionError("an unrouted Reference keyword was silently accepted")
# Mobject's parameters are positional in the Reference's order: a bare
# Mobject holds no members (Group does), and Mobject(RED) is its color.
assert Mobject(manimlib.RED).color == manimlib.RED and len(Mobject(manimlib.RED)) == 0
assert pgroup.get_num_points() == 0
assert pgroup.data.dtype.names == ("point", "rgba")
pgroup.shift(manimlib.UP)
assert np.allclose(pgroup_left.get_center(), [-1.0, 1.0, 0.0])
assert np.allclose(pgroup_right.get_center(), [1.0, 1.0, 0.0])

# fm-5wq.4: remaining PMobject family mutations over live records.
assert point_cloud_mobjects.PMobject.__bases__ == (manimlib.Mobject,)
cloud_a = manimlib.DotCloud(
    [[0.0, 1.0, 0.0], [2.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
    radius=0.05,
)
assert np.allclose(
    cloud_a.point_from_proportion(0.0), [0.0, 1.0, 0.0]
)
assert np.allclose(
    cloud_a.point_from_proportion(1.0), [1.0, 0.0, 0.0]
)
assert cloud_a.sort_points() is cloud_a
assert np.allclose(
    cloud_a.get_points(),
    [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
)
assert cloud_a.filter_out(lambda p: p[0] > 1.5) is cloud_a
assert np.allclose(
    cloud_a.get_points(),
    [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
)
assert cloud_a.set_color_by_gradient("#FF0000", "#0000FF") is cloud_a
assert np.allclose(cloud_a.data["rgba"][0, :3], [1.0, 0.0, 0.0], atol=1e-5)
assert np.allclose(cloud_a.data["rgba"][-1, :3], [0.0, 0.0, 1.0], atol=1e-5)
assert np.allclose(cloud_a.data["rgba"][:, 3], [1.0, 1.0])
cloud_b = manimlib.DotCloud(
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0]],
    radius=0.05,
)
assert cloud_b.match_colors(cloud_a) is cloud_b
assert np.allclose(cloud_b.data["rgba"][0, :3], [1.0, 0.0, 0.0], atol=1e-5)
assert np.allclose(cloud_b.data["rgba"][-1, :3], [0.0, 0.0, 1.0], atol=1e-5)
partial = manimlib.DotCloud([[9.0, 9.0, 0.0]], radius=0.05)
assert partial.pointwise_become_partial(cloud_b, 0.25, 0.75) is partial
assert partial.n_records() == 2
assert np.allclose(
    partial.get_points(),
    [[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
)
left_cloud = manimlib.DotCloud([[-1.0, 0.0, 0.0]], radius=0.05)
right_cloud = manimlib.DotCloud([[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]], radius=0.05)
ingest_group = point_cloud_mobjects.PGroup(left_cloud, right_cloud)
assert ingest_group.get_num_points() == 0
assert ingest_group.ingest_submobjects() is ingest_group
assert ingest_group.get_num_points() == 3
assert list(ingest_group.submobjects) == []
assert np.allclose(
    ingest_group.get_points(),
    [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
)
empty_color_source = manimlib.DotCloud([[0.0, 0.0, 0.0]], radius=0.05)
empty_color_source.resize_points(0)
color_dest = manimlib.DotCloud([[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]], radius=0.05)
before_rgba = color_dest.data["rgba"].copy()
assert color_dest.match_colors(empty_color_source) is color_dest
assert np.array_equal(color_dest.data["rgba"], before_rgba)

back_surface = manimlib.Surface(resolution=(2, 2), z_index=-1)
front_surface = manimlib.Surface(resolution=(2, 2), z_index=1)
surface_order_scene = Scene().add(front_surface, back_surface)
assert surface_order_scene.get_mobjects() == [back_surface, front_surface]

failed_pgroup = point_cloud_mobjects.PGroup.__new__(
    point_cloud_mobjects.PGroup
)
try:
    point_cloud_mobjects.PGroup.__init__(
        failed_pgroup,
        geometry.Circle(),
    )
except Exception as error:
    assert str(error) == "All submobjects must be of type PMobject"
else:
    raise AssertionError("PGroup accepted a non-PMobject child")
assert not hasattr(failed_pgroup, "submobjects")

# ThreeDModel is the next Atlas-backed schema refusal after PGroup. The public
# object keeps the Reference's heterogeneous Group MRO while Atlas parses the
# OBJ, resolves normals, normalizes height, and emits its real Surface child.
surface_types = importlib.import_module("manimlib.mobject.types.surface")
assert surface_types.ThreeDModel.__bases__ == (manimlib.Group,)
assert str(inspect.signature(surface_types.ThreeDModel)) == (
    "(obj_file: str, height=3)"
)
obj_root = pathlib.Path(tempfile.mkdtemp(prefix="fmn-three-d-model-"))
obj_path = obj_root / "tetrahedron.obj"
obj_path.write_text(
    "v 0 1 0\n"
    "v -1 -1 1\n"
    "v 1 -1 1\n"
    "v 0 -1 -1\n"
    "vn 0 1 0\n"
    "vn -1 -1 1\n"
    "vn 1 -1 1\n"
    "vn 0 -1 -1\n"
    "f 1//1 2//2 3//3\n"
    "f 1//1 4//4 2//2\n"
    "f 1//1 3//3 4//4\n"
    "f 2//2 4//4 3//3\n",
    encoding="utf-8",
)
native_three_d_model = surface_types.ThreeDModel(str(obj_path), height=2.5)
assert native_three_d_model.obj_file == str(obj_path)
assert np.isclose(native_three_d_model.height, 2.5)
assert len(native_three_d_model.submobjects) == 1
native_model_mesh = native_three_d_model.submobjects[0]
assert isinstance(native_model_mesh, surface_types.Surface)
assert native_model_mesh.data.dtype.names == (
    "point",
    "d_normal_point",
    "rgba",
)
assert native_model_mesh.get_num_points() == 12
assert np.isclose(native_model_mesh.get_height(), 2.5)
assert np.allclose(native_model_mesh.get_center(), manimlib.ORIGIN, atol=1e-6)
assert native_model_mesh.uniforms["depth_test"] is True
assert np.allclose(native_model_mesh.get_shading(), [0.3, 0.2, 0.4])
assert native_model_mesh._is_bound() is False
native_model_scene = Scene()
native_model_scene.add(native_three_d_model)
assert native_three_d_model._is_bound()
assert native_model_mesh._is_bound()

# Textured surfaces and meshes preserve the Reference's class and signature surface.
surface_types = importlib.import_module("manimlib.mobject.types.surface")
assert surface_types.TexturedSurface.__bases__ == (surface_types.Surface,)
assert surface_types.TexturedGeometry.__bases__ == (surface_types.TexturedSurface,)
assert list(inspect.signature(surface_types.TexturedSurface).parameters) == [
    "uv_surface",
    "image_file",
    "dark_image_file",
    "kwargs",
]
assert list(inspect.signature(surface_types.TexturedGeometry).parameters) == [
    "geometry",
    "texture_file",
    "kwargs",
]
# Original in-test raster bytes exercise real texture construction, not a
# blanket refusal. Only the still-unrepresented light/dark pair is refused.
_texture_root = pathlib.Path(tempfile.mkdtemp(prefix="fmn-bridge-texture-"))
_texture_path = _texture_root / "light.png"

def _texture_chunk(kind, body):
    return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body))

_texture_bytes = (b"\x89PNG\r\n\x1a\n"
    + _texture_chunk(b"IHDR", struct.pack(">IIBBBBB", 1, 1, 8, 6, 0, 0, 0))
    + _texture_chunk(b"IDAT", zlib.compress(b"\0\xff\x40\x20\xff"))
    + _texture_chunk(b"IEND", b""))
_texture_path.write_bytes(_texture_bytes)
_texture_dark = _texture_root / "dark.png"
_texture_dark.write_bytes(_texture_bytes)
textured_uv = manimlib.Sphere(resolution=(5, 7))
_textured_live = surface_types.TexturedSurface(textured_uv, _texture_path, opacity=.75)
assert _textured_live.resolution == (5, 7)
assert _textured_live.num_textures == 1
assert _textured_live._image_dimensions() == (1, 1)
assert np.array_equal(_textured_live.get_points(), textured_uv.get_points())
assert np.allclose(_textured_live.get_opacities(), .75)
assert np.allclose(_textured_live.data["im_coords"][[0, -1]], [[0, 1], [1, 0]])
assert _textured_live.copy()._image_dimensions() == (1, 1)
paired = surface_types.TexturedSurface(textured_uv, _texture_path, _texture_dark)
assert paired.num_textures == 2
assert paired.dark_image_file == str(_texture_dark.resolve())
try:
    surface_types.TexturedSurface(manimlib.Square(), _texture_path)
except TypeError as error:
    assert str(error) == "TexturedSurface uv_surface must be a Surface"
else:
    raise AssertionError("TexturedSurface accepted a non-Surface")
failed_textured = surface_types.TexturedSurface.__new__(surface_types.TexturedSurface)
try:
    surface_types.TexturedSurface.__init__(failed_textured, textured_uv, _texture_path, unsupported=True)
except TypeError as error:
    assert str(error) == "textured object got unexpected keyword arguments: unsupported"
else:
    raise AssertionError("TexturedSurface silently discarded an unknown option")
try:
    surface_types.TexturedGeometry(object(), _texture_path)
except TypeError as error:
    assert "vertices, triangle faces and visual.uv" in str(error)
else:
    raise AssertionError("TexturedGeometry accepted an object without geometry")
_texture_geometry = types.SimpleNamespace(
    vertices=np.array([[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]),
    faces=np.array([[2, 0, 1]], dtype=np.uint32),
    visual=types.SimpleNamespace(uv=np.array([[0., 0.], [1., 0.], [0., 1.]])),
)
_textured_mesh = surface_types.TexturedGeometry(_texture_geometry, _texture_path)
assert np.array_equal(_textured_mesh.get_points(), _texture_geometry.vertices[[2, 0, 1]])
assert np.array_equal(_textured_mesh.get_triangle_indices(), [0, 1, 2])
assert _textured_mesh._image_dimensions() == (1, 1)

extract_scene = importlib.import_module("manimlib.extract_scene")
assert extract_scene.BlankScene.__bases__ == (InteractiveScene,)
assert str(inspect.signature(extract_scene.BlankScene.construct)) == "(self)"
blank_scene = extract_scene.BlankScene()
assert isinstance(blank_scene, InteractiveScene)
assert extract_scene.BlankScene.construct is not Scene.construct

loader_mod = importlib.import_module("manimlib.module_loader")
assert loader_mod.ModuleLoader.__bases__ == (object,)
_module_loader_signature = inspect.signature(loader_mod.ModuleLoader.get_module)
assert list(_module_loader_signature.parameters) == [
    "file_name",
    "is_during_reload",
], str(_module_loader_signature)
assert _module_loader_signature.parameters["is_during_reload"].default is False
assert loader_mod.ModuleLoader.get_module(None) is None
try:
    loader_mod.ModuleLoader.get_module("/no/such/scene.py")
except FileNotFoundError as error:
    assert "ModuleLoader cannot read" in str(error)
else:
    raise AssertionError("ModuleLoader accepted a missing scene file")
with tempfile.NamedTemporaryFile(
    "w", suffix=".py", delete=False, encoding="utf-8"
) as handle:
    handle.write("MARKER = 7\n")
    loader_path = handle.name
try:
    loaded = loader_mod.ModuleLoader.get_module(loader_path)
    assert loaded.MARKER == 7
    loaded.MARKER = 9
    cached = loader_mod.ModuleLoader.get_module(loader_path)
    assert cached is loaded
    assert cached.MARKER == 9
    pathlib.Path(loader_path).write_text("MARKER = 11\n", encoding="utf-8")
    reloaded = loader_mod.ModuleLoader.get_module(
        loader_path, is_during_reload=True
    )
    assert reloaded is not loaded
    assert reloaded.MARKER == 11
finally:
    pathlib.Path(loader_path).unlink(missing_ok=True)
    sys.modules.pop(getattr(loaded, "__name__", ""), None)
    sys.modules.pop(getattr(reloaded, "__name__", ""), None)

assert str(inspect.signature(extract_scene.get_indent)) == (
    "(code_lines, line_number)"
)
assert extract_scene.get_indent(["    play()", "wait()"], 0) == "    "
assert extract_scene.get_indent(["    play()", "wait()"], 1) == ""
assert extract_scene.get_scene_classes(None) == []
assert extract_scene.is_child_scene(Scene, extract_scene) is False
assert extract_scene.is_child_scene(InteractiveScene, extract_scene) is False
assert extract_scene.get_module({}) is None
try:
    extract_scene.get_module(object())
except TypeError as error:
    assert str(error) == "get_module run_config must be a dict"
else:
    raise AssertionError("extract_scene.get_module accepted a non-dict")
with tempfile.NamedTemporaryFile(
    "w", suffix=".py", delete=False, encoding="utf-8"
) as handle:
    handle.write(
        "from manimlib import InteractiveScene, Scene\n"
        "class Demo(Scene):\n"
        "    def construct(self):\n"
        "        pass\n"
        "class Also(InteractiveScene):\n"
        "    def construct(self):\n"
        "        pass\n"
        "class Waiter(Scene):\n"
        "    def construct(self):\n"
        "        self.wait(1)\n"
    )
    extract_path = handle.name
try:
    extracted = extract_scene.get_module({"file_name": extract_path})
    classes = extract_scene.get_scene_classes(extracted)
    assert [cls.__name__ for cls in classes] == ["Demo", "Also", "Waiter"]
    assert all(extract_scene.is_child_scene(cls, extracted) for cls in classes)
    assert extract_scene.is_child_scene(extracted.Scene, extracted) is False
    demo = extract_scene.scene_from_class(extracted.Demo, {}, {})
    assert isinstance(demo, extracted.Demo)
    named = extract_scene.get_scenes_to_render(
        classes, {}, {"scene_names": ["Also"]}
    )
    assert len(named) == 1
    assert isinstance(named[0], extracted.Also)
    written = extract_scene.get_scenes_to_render(
        classes, {}, {"write_all": True}
    )
    assert [type(scene).__name__ for scene in written] == [
        "Demo",
        "Also",
        "Waiter",
    ]
    lone = extract_scene.get_scenes_to_render([extracted.Demo], {}, {})
    assert len(lone) == 1
    assert isinstance(lone[0], extracted.Demo)
    try:
        extract_scene.get_scenes_to_render(
            classes, {}, {"scene_names": ["Missing"]}
        )
    except ValueError as error:
        assert "Missing" in str(error)
        assert "Demo" in str(error)
    else:
        raise AssertionError("missing scene names were silent")
    try:
        extract_scene.get_scenes_to_render(classes, {}, {})
    except bridge_errors.CapabilityError as error:
        assert "prompt_user_for_choice" in str(error)
    else:
        raise AssertionError("ambiguous scene choice did not refuse")
    try:
        extract_scene.scene_from_class(manimlib.Square, {}, {})
    except TypeError as error:
        assert str(error) == (
            "scene_from_class scene_class must be a Scene subclass"
        )
    else:
        raise AssertionError("scene_from_class accepted a non-Scene")
    assert extract_scene.compute_total_frames(extracted.Demo, {}) == 0
    assert extract_scene.compute_total_frames(extracted.Waiter, {}) == 30
    ran = extract_scene.main(
        {}, {"file_name": extract_path, "scene_names": ["Demo"]}
    )
    assert len(ran) == 1
    assert type(ran[0]).__name__ == "Demo"
    try:
        extract_scene.main({}, {})
    except ValueError as error:
        assert str(error) == "main run_config needs file_name"
    else:
        raise AssertionError("main accepted a config with no file_name")
    try:
        extract_scene.main(object(), {"file_name": extract_path})
    except TypeError as error:
        assert str(error) == "main scene_config must be a dict"
    else:
        raise AssertionError("main accepted a non-dict scene_config")
    patched = extract_scene.insert_embed_line_to_module(
        extracted, {"embed_line": 4}
    )
    patched_source = pathlib.Path(extract_path).read_text(encoding="utf-8")
    assert "self.embed()" in patched_source
    assert patched is not extracted
    assert extract_scene.insert_embed_line_to_module(patched, {}) is patched
    try:
        extract_scene.insert_embed_line_to_module(patched, {"embed_line": 0})
    except ValueError as error:
        assert "out of range" in str(error)
    else:
        raise AssertionError("embed_line 0 was accepted")
finally:
    pathlib.Path(extract_path).unlink(missing_ok=True)
    sys.modules.pop(getattr(extracted, "__name__", ""), None)
    if "patched" in locals():
        sys.modules.pop(getattr(patched, "__name__", ""), None)

scene_module = importlib.import_module("manimlib.scene.scene")
assert scene_module.EndScene.__bases__ == (Exception,)
assert manimlib.EndScene is scene_module.EndScene
try:
    raise scene_module.EndScene("stop scene")
except scene_module.EndScene as error:
    assert error.args == ("stop scene",)
    assert str(error) == "stop scene"
else:
    raise AssertionError("EndScene did not preserve Exception raise semantics")
try:
    Scene()._end_scene()
except scene_module.EndScene as error:
    assert error.args == ("scene ended",)
    assert str(error) == "scene ended"
else:
    raise AssertionError("native Scene.end did not raise EndScene")

assert str(inspect.signature(Scene.wait_until)) == (
    "(self, stop_condition, max_time=60)"
)
wait_until_scene = Scene()
wait_until_samples = []


def stop_waiting_after_two_frames():
    wait_until_samples.append(wait_until_scene.get_time())
    return len(wait_until_samples) >= 2


assert (
    wait_until_scene.wait_until(stop_waiting_after_two_frames, max_time=1)
    is None
)
assert len(wait_until_samples) == 2
assert np.allclose(wait_until_samples, [1 / 30, 2 / 30])
assert np.isclose(wait_until_scene.get_time(), 2 / 30)
try:
    Scene().wait_until(object(), max_time=1 / 30)
except TypeError as error:
    assert str(error) == (
        "Scene.wait stop_condition must be callable or None; got object"
    )
else:
    raise AssertionError("Scene.wait_until accepted a non-callable predicate")

# The next honest native constructor is legacy SingleStringTex over Scribe's
# existing fmn-library Tex builder.
old_tex_mobjects = importlib.import_module(
    "manimlib.mobject.svg.old_tex_mobject"
)
assert old_tex_mobjects.SingleStringTex.__bases__ == (manimlib.SVGMobject,)
assert str(inspect.signature(old_tex_mobjects.SingleStringTex)) == (
    "(tex_string, height=None, fill_color='#FFFFFF', fill_opacity=1.0, "
    "stroke_width=0, svg_default={'fill_color': '#FFFFFF'}, "
    "path_string_config={}, font_size=48, alignment='\\\\centering', "
    "math_mode=True, organize_left_to_right=False, template='', "
    "additional_preamble='', **kwargs)"
)
single_tex = old_tex_mobjects.SingleStringTex(
    r"x^2",
    fill_color=manimlib.GREEN,
    fill_opacity=0.6,
    stroke_width=1.25,
    font_size=36,
)
assert single_tex.get_tex() == r"x^2"
assert single_tex.font_size == 36
assert single_tex.math_mode is True
assert len(single_tex.submobjects) > 0
assert single_tex.get_fill_color() == manimlib.GREEN
assert np.isclose(single_tex.get_fill_opacity(), 0.6)
assert np.isclose(single_tex.get_stroke_width(), 1.25)
assert all(child._is_bound() is False for child in single_tex.submobjects)
single_tex_scene = Scene()
single_tex_scene.add(single_tex)
assert single_tex._is_bound()
assert all(child._is_bound() for child in single_tex.submobjects)

single_text = old_tex_mobjects.SingleStringTex(
    "native",
    math_mode=False,
    height=0.75,
    organize_left_to_right=True,
)
assert single_text.math_mode is False
assert np.isclose(single_text.get_height(), 0.75)
assert np.all(
    np.diff([child.get_center()[0] for child in single_text.submobjects]) >= 0
)
assert single_text.get_modified_expression("") == r"\quad"
assert single_text.get_modified_expression("x_{") == "x_{}"

failed_single_tex = old_tex_mobjects.SingleStringTex.__new__(
    old_tex_mobjects.SingleStringTex
)
try:
    old_tex_mobjects.SingleStringTex.__init__(
        failed_single_tex,
        "x",
        template="external-latex-template",
    )
except ValueError as error:
    assert "unknown tex template" in str(error)
    assert "available packs: default, basic, empty" in str(error)
else:
    raise AssertionError("SingleStringTex accepted an external template")
assert not hasattr(failed_single_tex, "submobjects")

# VMobjectFromSVGPath consumes the pinned Reference path object's d() text,
# but Chisel owns command parsing, arc conversion, and shared-anchor output.
svg_mobjects = importlib.import_module("manimlib.mobject.svg.svg_mobject")
assert svg_mobjects.VMobjectFromSVGPath.__bases__ == (VMobject,)
assert str(inspect.signature(svg_mobjects.VMobjectFromSVGPath)) == (
    "(path_obj, **kwargs)"
)


class NativeSvgPath:
    def __init__(self, data):
        self.data = data

    def d(self):
        return self.data


default_svg_path = svg_mobjects.VMobjectFromSVGPath(
    NativeSvgPath("M 0 0 L 1 0")
)
assert np.isclose(default_svg_path.get_fill_opacity(), 0.0)
assert np.isclose(default_svg_path.get_stroke_opacity(), 1.0)
assert np.isclose(default_svg_path.get_stroke_width(), 4.0)

native_path_object = NativeSvgPath(
    "M 0 0 L 2 0 Q 3 1 4 0 A 1 1 0 0 1 5 1 Z"
)
native_svg_path = svg_mobjects.VMobjectFromSVGPath(
    native_path_object,
    fill_color=manimlib.BLUE,
    fill_opacity=0.4,
    stroke_color=manimlib.GREEN,
    stroke_width=2.25,
    joint_type="bevel",
    anti_alias_width=2.0,
    scale_stroke_with_zoom=True,
)
assert native_svg_path.path_obj is native_path_object
assert native_svg_path.transform_cache is None
assert native_svg_path.get_num_points() > 0
assert native_svg_path.data.dtype.names == (
    "point",
    "stroke_rgba",
    "stroke_width",
    "joint_angle",
    "fill_rgba",
    "base_normal",
    "fill_border_width",
)
assert np.isclose(native_svg_path.get_width(), 5.0, atol=1e-6)
assert np.isclose(native_svg_path.get_height(), 1.0, atol=1e-6)
assert native_svg_path.get_fill_color() == manimlib.BLUE
assert np.isclose(native_svg_path.get_fill_opacity(), 0.4)
assert native_svg_path.get_stroke_color() == manimlib.GREEN
assert np.isclose(native_svg_path.get_stroke_width(), 2.25)
assert native_svg_path.get_joint_type() == VMobject.joint_type_map["bevel"]
assert np.isclose(native_svg_path.get_anti_alias_width(), 2.0)
assert native_svg_path.uniforms["scale_stroke_with_zoom"] is True
native_svg_scene = Scene()
native_svg_scene.add(native_svg_path)
assert native_svg_path._is_bound()

failed_svg_path = svg_mobjects.VMobjectFromSVGPath.__new__(
    svg_mobjects.VMobjectFromSVGPath
)
try:
    svg_mobjects.VMobjectFromSVGPath.__init__(
        failed_svg_path,
        NativeSvgPath("M 0 0 L nope"),
    )
except ValueError as error:
    assert "path" in str(error).lower(), error
else:
    raise AssertionError("VMobjectFromSVGPath accepted malformed path data")
assert list(failed_svg_path.submobjects) == []
assert failed_svg_path._is_bound() is False

# Atlas already owns these curve, corner, and camera-frame primitives. Their
# public classes must drive those native builders rather than stop at the
# schema-generated constructor refusal which previously occupied each row.
frame_mobjects = importlib.import_module("manimlib.mobject.frame")
assert geometry.CubicBezier.__bases__ == (VMobject,)
assert geometry.Elbow.__bases__ == (VMobject,)
assert frame_mobjects.ScreenRectangle.__bases__ == (geometry.Rectangle,)
assert frame_mobjects.FullScreenRectangle.__bases__ == (
    frame_mobjects.ScreenRectangle,
)
assert frame_mobjects.FullScreenFadeRectangle.__bases__ == (
    frame_mobjects.FullScreenRectangle,
)
assert str(inspect.signature(geometry.CubicBezier)) == (
    "(a0, h0, h1, a1, **kwargs)"
)
assert str(inspect.signature(geometry.Elbow)) == (
    "(width=0.2, angle=0, **kwargs)"
)
assert str(inspect.signature(frame_mobjects.ScreenRectangle)) == (
    "(aspect_ratio=1.7777777777777777, height=4, **kwargs)"
)
assert str(inspect.signature(frame_mobjects.FullScreenRectangle)) == (
    "(height=8.0, fill_color='#222222', fill_opacity=1, "
    "stroke_width=0, **kwargs)"
)
assert str(inspect.signature(frame_mobjects.FullScreenFadeRectangle)) == (
    "(stroke_width=0.0, fill_color='#000000', fill_opacity=0.7, **kwargs)"
)

native_cubic = geometry.CubicBezier(
    (-2.0, 0.0, 0.0),
    (-1.0, 2.0, 0.0),
    (1.0, 2.0, 0.0),
    (2.0, 0.0, 0.0),
    stroke_color=manimlib.BLUE,
    stroke_width=2.5,
)
assert np.allclose(native_cubic.get_points()[0], [-2.0, 0.0, 0.0])
assert np.allclose(native_cubic.get_points()[-1], [2.0, 0.0, 0.0])
assert native_cubic.get_stroke_color() == manimlib.BLUE
assert np.isclose(native_cubic.get_stroke_width(), 2.5)

# fm-5wq.4.140: the remaining VMobject kwargs route through the shared
# native style pass after the cubic control points have been built.
styled_cubic = geometry.CubicBezier(
    (-1.0, 0.0, 0.0),
    (-0.5, 1.0, 0.0),
    (0.5, 1.0, 0.0),
    (1.0, 0.0, 0.0),
    fill_color=manimlib.RED,
    fill_opacity=0.35,
    stroke_color=manimlib.GREEN,
    stroke_opacity=0.6,
    stroke_width=3.25,
    flat_stroke=True,
)
assert styled_cubic.get_fill_color() == manimlib.RED
assert np.isclose(styled_cubic.get_fill_opacity(), 0.35)
assert styled_cubic.get_stroke_color() == manimlib.GREEN
assert np.isclose(styled_cubic.get_stroke_opacity(), 0.6)
assert np.isclose(styled_cubic.get_stroke_width(), 3.25)
assert styled_cubic.get_flat_stroke() is True

try:
    geometry.CubicBezier(
        (-1.0, 0.0, 0.0),
        (-0.5, 1.0, 0.0),
        (0.5, 1.0, 0.0),
        (1.0, 0.0, 0.0),
        wobble_amount=1,
    )
except TypeError as error:
    assert "wobble_amount" in str(error), error
else:
    raise AssertionError("CubicBezier silently ignored an unknown keyword")

failed_cubic = geometry.CubicBezier.__new__(geometry.CubicBezier)
try:
    geometry.CubicBezier.__init__(
        failed_cubic,
        (0.0, 0.0, 0.0),
        (1.0, math.nan, 0.0),
        (2.0, 1.0, 0.0),
        (3.0, 0.0, 0.0),
    )
except ValueError as error:
    assert "tolerance" in str(error).lower()
else:
    raise AssertionError("CubicBezier published a non-finite native path")
assert failed_cubic.get_num_points() == 0

native_elbow = geometry.Elbow(width=2.0, angle=math.pi / 2, color=manimlib.RED)
assert np.isclose(native_elbow.get_height(), 2.0)
assert native_elbow.get_stroke_color() == manimlib.RED

screen_rectangle = frame_mobjects.ScreenRectangle()
assert np.isclose(screen_rectangle.get_width(), 4.0 * 16.0 / 9.0)
assert np.isclose(screen_rectangle.get_height(), 4.0)

full_screen_rectangle = frame_mobjects.FullScreenRectangle()
assert np.isclose(full_screen_rectangle.get_width(), manimlib.FRAME_WIDTH)
assert np.isclose(full_screen_rectangle.get_height(), manimlib.FRAME_HEIGHT)
assert full_screen_rectangle.get_fill_color() == manimlib.GREY_E
assert np.isclose(full_screen_rectangle.get_fill_opacity(), 1.0)
assert np.isclose(full_screen_rectangle.get_stroke_width(), 0.0)

custom_full_screen = frame_mobjects.FullScreenRectangle(
    height=3.0,
    fill_color=manimlib.BLUE,
    fill_opacity=0.25,
    stroke_width=1.5,
    stroke_color=manimlib.RED,
)
assert np.isclose(custom_full_screen.get_width(), 3.0 * 16.0 / 9.0)
assert np.isclose(custom_full_screen.get_height(), 3.0)
assert custom_full_screen.get_fill_color() == manimlib.BLUE
assert np.isclose(custom_full_screen.get_fill_opacity(), 0.25)
assert custom_full_screen.get_stroke_color() == manimlib.RED
assert np.isclose(custom_full_screen.get_stroke_width(), 1.5)

fade_rectangle = frame_mobjects.FullScreenFadeRectangle(height=2.0)
assert np.isclose(fade_rectangle.get_width(), manimlib.FRAME_WIDTH)
assert np.isclose(fade_rectangle.get_height(), manimlib.FRAME_HEIGHT)
assert fade_rectangle.get_fill_color() == manimlib.BLACK
assert np.isclose(fade_rectangle.get_fill_opacity(), 0.7)
assert np.isclose(fade_rectangle.get_stroke_width(), 0.0)

native_shelf_scene = Scene()
native_shelf_scene.add(
    native_cubic,
    native_elbow,
    screen_rectangle,
    full_screen_rectangle,
    custom_full_screen,
    fade_rectangle,
)
assert all(mobject._is_bound() for mobject in native_shelf_scene.mobjects)

# Shape matchers consume the real family bounding box and retain the native
# empty-target rule.  A generated schema shell used to fail here before any
# geometry reached Atlas.
shape_matchers = importlib.import_module("manimlib.mobject.shape_matchers")
assert str(inspect.signature(shape_matchers.SurroundingRectangle)) == (
    "(mobject, buff=0.1, color='#FFFF00', **kwargs)"
)
assert str(inspect.signature(shape_matchers.SurroundingRectangle.set_buff)) == (
    "(self, buff)"
)
assert str(inspect.signature(shape_matchers.SurroundingRectangle.surround)) == (
    "(self, mobject, buff=None)"
)
assert str(inspect.signature(shape_matchers.BackgroundRectangle)) == (
    "(mobject, color=None, stroke_width=0, stroke_opacity=0, "
    "fill_opacity=0.75, buff=0, **kwargs)"
)
assert str(inspect.signature(shape_matchers.BackgroundRectangle.set_style)) == (
    "(self, stroke_color=None, stroke_width=None, fill_color=None, "
    "fill_opacity=None, family=True, **kwargs)"
)
left_box = geometry.Rectangle(width=2.0, height=1.0).shift([-1.5, 0.5, 0.0])
right_box = geometry.Rectangle(width=1.0, height=2.0).shift([2.0, -0.5, 0.0])
box_group = manimlib.VGroup(left_box, right_box)
surround = shape_matchers.SurroundingRectangle(
    box_group, buff=0.25, color=manimlib.BLUE, stroke_width=2.0
)
target_box = box_group.get_bounding_box()
surround_box = surround.get_bounding_box()
assert np.allclose(surround_box[0][:2], target_box[0][:2] - 0.25)
assert np.allclose(surround_box[2][:2], target_box[2][:2] + 0.25)
assert surround.get_stroke_color() == manimlib.BLUE
assert np.allclose(surround.get_stroke_width(), 2.0)
channel_surround = shape_matchers.SurroundingRectangle(
    box_group, color=manimlib.BLUE, stroke_color=manimlib.RED,
    fill_color=manimlib.GREEN, fill_opacity=0.3,
)
assert channel_surround.get_stroke_color() == manimlib.RED
assert channel_surround.get_fill_color() == manimlib.GREEN
channel_surround.set_buff(0.5)
assert channel_surround.get_stroke_color() == manimlib.RED
assert channel_surround.get_fill_color() == manimlib.GREEN
fallback_surround = shape_matchers.SurroundingRectangle(
    box_group, color=manimlib.BLUE, stroke_color=None, fill_color=None,
)
assert fallback_surround.get_stroke_color() == manimlib.BLUE
assert fallback_surround.get_fill_color() == manimlib.BLUE
surround.set_buff(0.5)
assert np.allclose(surround.get_bounding_box()[0][:2], target_box[0][:2] - 0.5)
empty_surround = shape_matchers.SurroundingRectangle(Mobject())
assert not empty_surround.has_points()
assert isinstance(surround, geometry.Rectangle)

# BackgroundRectangle uses the same Atlas extent matcher, the pinned camera
# background default, and the Reference's deliberately locked public style.
background_target = geometry.Rectangle(width=2.0, height=1.0)
background_target_box = background_target.get_bounding_box().copy()
background = shape_matchers.BackgroundRectangle(
    background_target, buff=0.2, fill_opacity=0.6
)
assert np.allclose(
    background.get_bounding_box()[0][:2], background_target_box[0][:2] - 0.2
)
assert np.allclose(
    background.get_bounding_box()[2][:2], background_target_box[2][:2] + 0.2
)
assert background.get_fill_color() == "#333333"
assert VMobject.get_fill_color(background) == "#333333"
assert np.isclose(background.get_fill_opacity(), 0.6)
assert np.isclose(background.get_stroke_width(), 0.0)
assert np.isclose(background.get_stroke_opacity(), 0.0)
assert background.pointwise_become_partial(background_target, 0.2, 0.5) is background
assert np.isclose(background.get_fill_opacity(), 0.3)
assert (
    background.set_style(
        stroke_color="#FF0000",
        stroke_width=9,
        fill_color="#00FF00",
        fill_opacity=0.25,
        family=False,
        ignored_reference_kwarg=True,
    )
    is background
)
assert background.get_fill_color() == "#333333"
assert VMobject.get_fill_color(background) == "#000000"
assert background.get_stroke_color() == "#000000"
assert np.isclose(background.get_stroke_width(), 0.0)
assert np.isclose(background.get_fill_opacity(), 0.25)

detached_plate_target = geometry.Rectangle(width=1.0, height=0.5)
detached_plate_box = detached_plate_target.get_bounding_box().copy()
assert (
    detached_plate_target.add_background_rectangle(
        color="#123456", opacity=0.8, buff=0.15
    )
    is detached_plate_target
)
detached_plate = detached_plate_target.background_rectangle
assert isinstance(detached_plate, shape_matchers.BackgroundRectangle)
assert detached_plate_target.submobjects[0] is detached_plate
assert detached_plate.get_fill_color() == "#123456"
assert np.isclose(detached_plate.get_fill_opacity(), 0.8)
assert np.allclose(
    detached_plate.get_bounding_box()[0][:2], detached_plate_box[0][:2] - 0.15
)

bound_plate_scene = Scene()
bound_plate_target = geometry.Rectangle(width=1.5, height=0.75)
bound_plate_scene.add(bound_plate_target)
assert bound_plate_target.add_background_rectangle(buff=0.1) is bound_plate_target
assert bound_plate_target.background_rectangle._is_bound()
assert bound_plate_target.submobjects[0] is bound_plate_target.background_rectangle
assert bound_plate_scene.get_mobjects()[0] is bound_plate_target

immediate_a = geometry.Square(side_length=0.5)
immediate_b = geometry.Square(side_length=0.75)
immediate_group = Mobject().add(immediate_a, immediate_b)
bound_plate_scene.add(immediate_group)
assert immediate_group.add_background_rectangle_to_submobjects(opacity=0.4) is immediate_group
for immediate in (immediate_a, immediate_b):
    assert immediate.submobjects[0] is immediate.background_rectangle
    assert immediate.background_rectangle._is_bound()
    assert np.isclose(immediate.background_rectangle.get_fill_opacity(), 0.4)

family_point_a = geometry.Square(side_length=0.5)
family_point_b = geometry.Square(side_length=0.75)
family_point_container = Mobject().add(family_point_b)
family_plate_root = Mobject().add(family_point_a, family_point_container)
bound_plate_scene.add(family_plate_root)
assert (
    family_plate_root.add_background_rectangle_to_family_members_with_points(
        opacity=0.35
    )
    is family_plate_root
)
assert not hasattr(family_plate_root, "background_rectangle")
assert not hasattr(family_point_container, "background_rectangle")
for point_member in (family_point_a, family_point_b):
    assert point_member.submobjects[0] is point_member.background_rectangle
    assert point_member.background_rectangle._is_bound()
    assert np.isclose(point_member.background_rectangle.get_fill_opacity(), 0.35)

# Retargeting after scene entry rebuilds the same arena object through the
# native matcher. Identity and style survive, and bad targets refuse before
# touching the live geometry.
matcher_scene = Scene()
matcher_scene.add(box_group, surround)
surround_identity = id(surround)
surround_before_refusal = surround.get_points().copy()
try:
    surround.surround([0.0, 0.0, 0.0])
except TypeError as error:
    assert str(error) == "SurroundingRectangle.surround expects a Mobject"
else:
    raise AssertionError("a non-Mobject surrounding target was accepted")
assert np.array_equal(surround.get_points(), surround_before_refusal)

right_box.shift([1.0, 1.5, 0.0])
live_target_box = box_group.get_bounding_box()
assert surround.surround(box_group, buff=0.75) is surround
assert id(surround) == surround_identity
assert np.allclose(
    surround.get_bounding_box()[0][:2], live_target_box[0][:2] - 0.75
)
assert np.allclose(
    surround.get_bounding_box()[2][:2], live_target_box[2][:2] + 0.75
)
assert surround.get_stroke_color() == manimlib.BLUE
assert np.allclose(surround.get_stroke_width(), 2.0)
surround.set_buff(0.1)
assert np.allclose(
    surround.get_bounding_box()[0][:2], live_target_box[0][:2] - 0.1
)

fixed_match_target = geometry.Rectangle(width=1.0, height=0.5).fix_in_frame()
fixed_surround = shape_matchers.SurroundingRectangle(fixed_match_target)
assert fixed_match_target.is_fixed_in_frame()
assert fixed_surround.is_fixed_in_frame()

bound_empty_target = Mobject()
matcher_scene.add(bound_empty_target, empty_surround)
empty_surround.surround(bound_empty_target)
assert not empty_surround.has_points()
empty_surround.surround(box_group, buff=0.2)
assert empty_surround.has_points()
assert np.allclose(
    empty_surround.get_bounding_box()[2][:2], live_target_box[2][:2] + 0.2
)

# The rest of Atlas's matcher shelf is routed through the same authoritative
# family-extent crossing. Cross and Underline retain their Reference MRO and
# tapered record profiles; the Python layer does not reconstruct either path.
cross_call_shape = str(inspect.signature(shape_matchers.Cross))
assert cross_call_shape == (
    "(mobject, stroke_color='#FC6255', stroke_width=[0, 6, 0], **kwargs)"
), cross_call_shape
underline_call_shape = str(inspect.signature(shape_matchers.Underline))
assert underline_call_shape == (
    "(mobject, buff=0.1, stroke_color='#FFFFFF', "
    "stroke_width=[0, 3, 3, 0], stretch_factor=1.2, **kwargs)"
), underline_call_shape
assert shape_matchers.Cross.__bases__ == (manimlib.VGroup,)
assert shape_matchers.Underline.__bases__ == (geometry.Line,)

native_match_target = manimlib.VGroup(
    geometry.Rectangle(width=2.0, height=1.0).shift([-1.0, 0.25, 0.0]),
    geometry.Rectangle(width=1.0, height=2.0).shift([1.75, -0.5, 0.0]),
)
native_match_box = native_match_target.get_bounding_box().copy()
native_cross = shape_matchers.Cross(native_match_target)
assert len(native_cross.submobjects) == 2, len(native_cross.submobjects)
assert np.allclose(
    native_cross.get_bounding_box()[[0, 2], :2],
    native_match_box[[0, 2], :2],
)
for arm in native_cross.submobjects:
    widths = arm.get_stroke_widths()
    assert len(widths) > 3
    assert np.isclose(widths[0], 0.0)
    assert np.isclose(widths[-1], 0.0)
    assert np.isclose(widths.max(), 6.0)
    assert arm.get_stroke_color() == manimlib.RED

scalar_cross = shape_matchers.Cross(
    native_match_target, stroke_color=manimlib.BLUE, stroke_width=2.5
)
for arm in scalar_cross.submobjects:
    assert np.allclose(arm.get_stroke_widths(), 2.5)
    assert arm.get_stroke_color() == manimlib.BLUE

native_underline = shape_matchers.Underline(native_match_target, buff=0.2)
underline_box = native_underline.get_bounding_box()
target_width = native_match_box[2, 0] - native_match_box[0, 0]
assert np.isclose(underline_box[2, 0] - underline_box[0, 0], 1.2 * target_width)
assert np.isclose(underline_box[0, 1], native_match_box[0, 1] - 0.2)
assert np.isclose(underline_box[2, 1], native_match_box[0, 1] - 0.2)
underline_widths = native_underline.get_stroke_widths()
assert np.isclose(underline_widths[0], 0.0)
assert np.isclose(underline_widths[-1], 0.0)
assert np.isclose(underline_widths.max(), 3.0)

scalar_underline = shape_matchers.Underline(
    native_match_target,
    stroke_color=manimlib.GREEN,
    stroke_width=1.75,
    stretch_factor=0.8,
)
assert np.allclose(scalar_underline.get_stroke_widths(), 1.75)
assert scalar_underline.get_stroke_color() == manimlib.GREEN
assert np.isclose(scalar_underline.get_width(), 0.8 * target_width)

empty_cross = shape_matchers.Cross(Mobject())
empty_underline = shape_matchers.Underline(Mobject())
assert not empty_cross.family_members_with_points()
assert not empty_underline.family_members_with_points()

curved_underline = shape_matchers.Underline(
    native_match_target, path_arc=math.pi / 3.0
)
assert np.isclose(curved_underline.path_arc, math.pi / 3.0)
assert curved_underline.get_arc_length() > curved_underline.get_length()
assert not np.isclose(
    curved_underline.point_from_proportion(0.5)[1],
    curved_underline.get_start()[1],
)
curved_underline_widths = curved_underline.get_stroke_widths()
assert np.isclose(curved_underline_widths[0], 0.0)
assert np.isclose(curved_underline_widths[-1], 0.0)
assert np.isclose(curved_underline_widths.max(), 3.0)

bound_matcher_scene = Scene()
bound_match_target = geometry.Rectangle(width=1.5, height=0.75)
bound_matcher_scene.add(bound_match_target)
bound_cross = shape_matchers.Cross(bound_match_target)
bound_underline = shape_matchers.Underline(bound_match_target)
bound_matcher_scene.add(bound_cross, bound_underline)
assert bound_cross._is_bound()
assert all(arm._is_bound() for arm in bound_cross.submobjects)
assert bound_underline._is_bound()

for constructor, message in (
    (shape_matchers.Cross, "Cross expects a Mobject"),
    (shape_matchers.Underline, "Underline expects a Mobject"),
):
    try:
        constructor([0.0, 0.0, 0.0])
    except TypeError as error:
        assert str(error) == message
    else:
        raise AssertionError(f"{constructor.__name__} accepted a non-Mobject target")

try:
    shape_matchers.Underline(None)
except TypeError as error:
    assert str(error) == "Underline expects a Mobject"
else:
    raise AssertionError("Underline accepted None")

for name, kwargs in (
    ("buff", {"buff": float("nan")}),
    ("stretch_factor", {"stretch_factor": float("inf")}),
):
    try:
        shape_matchers.Underline(native_match_target, **kwargs)
    except ValueError as error:
        assert name in str(error)
        assert "finite" in str(error)
    else:
        raise AssertionError(f"Underline accepted a non-finite {name}")

# Checkmark and Exmark keep the Reference preset-string inheritance and one
# selectable glyph-family shape, but their visible contours are Atlas's
# paired BN-08 paths rather than a hidden LaTeX/pifont subprocess.
drawings = importlib.import_module("manimlib.mobject.svg.drawings")
special_tex = importlib.import_module("manimlib.mobject.svg.special_tex")
assert str(inspect.signature(special_tex.TexTextFromPresetString)) == ("(**kwargs)")
assert str(inspect.signature(drawings.Checkmark)) == ("(**kwargs)")
assert str(inspect.signature(drawings.Exmark)) == ("(**kwargs)")
assert drawings.Checkmark.__bases__ == (special_tex.TexTextFromPresetString,)
assert drawings.Exmark.__bases__ == (special_tex.TexTextFromPresetString,)
assert issubclass(special_tex.TexTextFromPresetString, manimlib.TexText)
assert special_tex.TexTextFromPresetString.tex == ""
assert special_tex.TexTextFromPresetString.default_color == manimlib.DEFAULT_MOBJECT_COLOR
assert drawings.Checkmark.tex == r"\ding{51}"
assert drawings.Checkmark.default_color == manimlib.GREEN
assert drawings.Exmark.tex == r"\ding{55}"
assert drawings.Exmark.default_color == manimlib.RED


class NativeTexTemplateProbe(special_tex.TexTextFromPresetString):
    tex = "x"
    default_color = manimlib.BLUE


native_tex_template_probe = NativeTexTemplateProbe()
assert native_tex_template_probe.get_tex() == "x"
assert native_tex_template_probe.get_fill_color() == manimlib.BLUE

checkmark = drawings.Checkmark()
exmark = drawings.Exmark()
assert checkmark.get_tex() == r"\ding{51}"
assert exmark.get_tex() == r"\ding{55}"
assert len(checkmark.submobjects) == 1
assert len(exmark.submobjects) == 1
assert checkmark.get_part_by_tex(checkmark.tex)[0] is checkmark[0]
assert exmark.get_part_by_tex(exmark.tex)[0] is exmark[0]
# The Reference's \ding{51}/\ding{55} boxes at font_size 48 (fm-5wq.52).
assert np.allclose(
    checkmark.get_bounding_box()[[0, 2], :2], [[-0.16725, -0.1752], [0.16725, 0.1752]]
)
assert np.allclose(
    exmark.get_bounding_box()[[0, 2], :2], [[-0.1395, -0.17745], [0.1395, 0.17745]]
)
assert checkmark[0].get_fill_color() == manimlib.GREEN
assert exmark[0].get_fill_color() == manimlib.RED
assert np.isclose(checkmark[0].get_stroke_width(), 0.0)
assert np.isclose(exmark[0].get_stroke_width(), 0.0)

large_checkmark = drawings.Checkmark(font_size=96, color=manimlib.BLUE)
assert np.isclose(large_checkmark.get_width(), 2 * 0.3345)
assert np.isclose(large_checkmark.get_height(), 2 * 0.3504)
assert large_checkmark[0].get_fill_color() == manimlib.BLUE

bound_matcher_scene.add(checkmark, exmark)
assert checkmark._is_bound() and checkmark[0]._is_bound()
assert exmark._is_bound() and exmark[0]._is_bound()

for constructor in (drawings.Checkmark, drawings.Exmark):
    try:
        constructor(font_size=0)
    except ValueError as error:
        assert "positive and finite" in str(error)
    else:
        raise AssertionError(f"{constructor.__name__} accepted zero font_size")

try:
    drawings.Checkmark(template="external-template")
except NotImplementedError as error:
    assert "template" in str(error)
else:
    raise AssertionError("Checkmark silently accepted an external TeX template")

assert drawings.Clock.__bases__ == (manimlib.VGroup,)
assert list(inspect.signature(drawings.Clock).parameters) == [
    "stroke_color",
    "stroke_width",
    "hour_hand_height",
    "minute_hand_height",
    "tick_length",
    "kwargs",
]
# geometry.py:386: Circle passes n_components on to Arc; 2**14 is a real
# corpus request (_2022/visual_proofs/lies.py).
_dense_circle = manimlib.Circle(radius=2.0, n_components=32)
assert len(_dense_circle.get_points()) == 65 and np.isclose(_dense_circle.get_radius(), 2.0)
assert len(manimlib.Circle(n_components=2**14).get_points()) == 2 * 2**14 + 1
try:
    manimlib.Circle(n_components=0)
except ValueError as error:
    assert "component" in str(error), error
else:
    raise AssertionError("Circle accepted zero arc components")

clock = drawings.Clock()
# drawings.py:296: VGroup(circle, hour_hand, minute_hand, ticks), with
# self.ticks exposed; ticks run inward from the rim, every third doubled.
assert len(clock.submobjects) == 4
assert clock.hour_hand is clock.submobjects[1]
assert clock.minute_hand is clock.submobjects[2]
assert clock.ticks is clock.submobjects[3] and len(clock.ticks) == 12
assert np.isclose(clock.hour_hand.get_length(), 0.3)
assert np.isclose(clock.minute_hand.get_length(), 0.6)
assert np.allclose(clock.ticks[0].get_start(), [0.0, 1.0, 0.0])
assert np.allclose(clock.ticks[0].get_end(), [0.0, 0.8, 0.0])
assert np.allclose(clock.ticks[1].get_start(), [0.5, math.sqrt(3) / 2, 0.0])
assert np.isclose(clock.ticks[1].get_length(), 0.1)
assert [round(tick.get_length(), 6) for tick in clock.ticks[::3]] == [0.2] * 4
tall_clock = drawings.Clock(hour_hand_height=0.4, minute_hand_height=0.8)
assert np.isclose(tall_clock.hour_hand.get_length(), 0.4)
assert np.isclose(tall_clock.minute_hand.get_length(), 0.8)
assert drawings.ClockPassesTime.__bases__ == (manimlib.AnimationGroup,)
assert list(inspect.signature(drawings.ClockPassesTime).parameters) == [
    "clock",
    "run_time",
    "hours_passed",
    "rate_func",
    "kwargs",
]
clock_passes = drawings.ClockPassesTime(
    clock, run_time=0.1, hours_passed=3.0
)
assert clock_passes.clock is clock
assert clock_passes.group is clock
assert clock_passes.rate_func is manimlib.linear
assert len(clock_passes.animations) == 2
assert all(isinstance(anim, manimlib.Rotating) for anim in clock_passes.animations)
assert np.isclose(clock_passes.animations[0].angle, -math.pi / 2)
assert np.isclose(clock_passes.animations[1].angle, -6 * math.pi)
clock_scene = Scene()
clock_scene.add(clock)
clock_scene.play(clock_passes)
hour_direction = clock.hour_hand.get_end() - clock.hour_hand.get_start()
minute_direction = clock.minute_hand.get_end() - clock.minute_hand.get_start()
hour_direction /= np.linalg.norm(hour_direction)
minute_direction /= np.linalg.norm(minute_direction)
assert np.allclose(hour_direction, manimlib.RIGHT, atol=1e-6)
assert np.allclose(minute_direction, manimlib.UP, atol=1e-6)
try:
    drawings.ClockPassesTime(manimlib.Circle())
except TypeError as error:
    assert str(error) == "ClockPassesTime clock must be a Clock"
else:
    raise AssertionError("ClockPassesTime accepted a non-Clock mobject")

assert drawings.Speedometer.__bases__ == (manimlib.VMobject,)
assert list(inspect.signature(drawings.Speedometer).parameters) == [
    "arc_angle",
    "num_ticks",
    "tick_length",
    "needle_width",
    "needle_height",
    "needle_color",
    "kwargs",
]
speedometer = drawings.Speedometer()
assert speedometer.arc is speedometer.submobjects[0]
assert speedometer.needle is speedometer.submobjects[-1]
assert speedometer.num_ticks == 8
assert len(speedometer.submobjects) == 2 + 2 * 8
assert np.allclose(speedometer.get_center(), [0.0, 0.0, 0.0], atol=1e-6)
start_angle = math.pi / 2.0 + speedometer.arc_angle / 2.0
end_angle = math.pi / 2.0 - speedometer.arc_angle / 2.0
tip = speedometer.get_needle_tip() - speedometer.get_center()
tip = tip / np.linalg.norm(tip)
assert np.allclose(
    tip,
    [math.cos(start_angle), math.sin(start_angle), 0.0],
    atol=1e-6,
)
assert speedometer.move_needle_to_velocity(70.0) is speedometer
tip = speedometer.get_needle_tip() - speedometer.get_center()
tip = tip / np.linalg.norm(tip)
assert np.allclose(
    tip,
    [math.cos(end_angle), math.sin(end_angle), 0.0],
    atol=1e-6,
)
custom_speedometer = drawings.Speedometer(
    arc_angle=math.pi,
    num_ticks=4,
    tick_length=0.25,
    needle_width=0.2,
    needle_height=0.5,
    needle_color=manimlib.RED,
)
assert custom_speedometer.num_ticks == 4
assert len(custom_speedometer.submobjects) == 2 + 2 * 4
# arc_angle=pi rotates the needle by π/2, so the AABB swaps the
# stretch_to_fit_width/height extents.
assert np.isclose(custom_speedometer.needle.get_width(), 0.5, atol=1e-6)
assert np.isclose(custom_speedometer.needle.get_height(), 0.2, atol=1e-6)
assert custom_speedometer.needle.get_fill_color() == manimlib.RED
try:
    drawings.Speedometer(num_ticks=0)
except ValueError as error:
    assert "positive" in str(error)
else:
    raise AssertionError("Speedometer accepted zero num_ticks")

assert drawings.DieFace.__bases__ == (manimlib.VGroup,)
assert list(inspect.signature(drawings.DieFace).parameters) == [
    "value",
    "side_length",
    "corner_radius",
    "stroke_color",
    "stroke_width",
    "fill_color",
    "dot_radius",
    "dot_color",
    "dot_coalesce_factor",
]
die_five = drawings.DieFace(5)
assert die_five.value == 5
assert die_five.index == 5
assert die_five.dots is die_five.submobjects[1]
assert len(die_five.submobjects) == 2
assert len(die_five.dots.submobjects) == 5
assert np.isclose(die_five.submobjects[0].get_width(), 1.0, atol=1e-6)
assert die_five.submobjects[0].get_fill_color() == manimlib.GREY_E
assert die_five.submobjects[0].get_stroke_color() == manimlib.WHITE
assert np.isclose(die_five.submobjects[0].get_fill_opacity(), 1.0)
die_ace = drawings.DieFace(1)
assert len(die_ace.dots.submobjects) == 1
assert np.allclose(die_ace.dots[0].get_center(), [0.0, 0.0, 0.0], atol=1e-6)
styled_die = drawings.DieFace(
    6,
    side_length=2.0,
    stroke_color=manimlib.RED,
    fill_color=manimlib.BLUE,
    dot_color=manimlib.YELLOW,
    dot_radius=0.12,
)
assert styled_die.value == 6
assert len(styled_die.dots.submobjects) == 6
assert np.isclose(styled_die.submobjects[0].get_width(), 2.0, atol=1e-6)
assert styled_die.submobjects[0].get_fill_color() == manimlib.BLUE
assert styled_die.submobjects[0].get_stroke_color() == manimlib.RED
assert styled_die.dots[0].get_fill_color() == manimlib.YELLOW
try:
    drawings.DieFace(0)
except Exception as error:
    assert str(error) == "DieFace only accepts integer inputs between 1 and 6"
else:
    raise AssertionError("DieFace accepted a pip count of 0")
try:
    drawings.DieFace(7)
except Exception as error:
    assert str(error) == "DieFace only accepts integer inputs between 1 and 6"
else:
    raise AssertionError("DieFace accepted a pip count of 7")

assert drawings.Dartboard.__bases__ == (manimlib.VGroup,)
assert list(inspect.signature(drawings.Dartboard).parameters) == ["kwargs"]
assert drawings.Dartboard.n_sectors == 20
assert drawings.Dartboard.radius == 3
dartboard = drawings.Dartboard()
assert dartboard.n_sectors == 20
assert dartboard.radius == 3
assert dartboard.bullseye is dartboard.submobjects[4]
assert len(dartboard.submobjects) == 5
assert all(len(ring) == 20 for ring in dartboard.submobjects[:3])
assert np.isclose(dartboard.get_width(), 6.0, atol=1e-5)
assert dartboard.submobjects[0][0].get_fill_color() == manimlib.GREY_B
assert dartboard.submobjects[0][1].get_fill_color() == manimlib.GREY_E
assert dartboard.submobjects[1][0].get_fill_color() == manimlib.GREEN_E
assert dartboard.submobjects[1][1].get_fill_color() == manimlib.RED_E
assert dartboard.bullseye.get_fill_color() == manimlib.RED_E
assert np.isclose(dartboard.submobjects[3].get_fill_opacity(), 1.0)
assert np.isclose(dartboard.bullseye.get_stroke_width(), 0.0)

assert drawings.Bubble.__bases__ == (manimlib.VGroup,)
assert drawings.SpeechBubble.__bases__ == (drawings.Bubble,)
assert list(inspect.signature(drawings.SpeechBubble).parameters) == [
    "content",
    "buff",
    "filler_shape",
    "stem_height_to_bubble_height",
    "stem_top_x_props",
    "kwargs",
]
speech = drawings.SpeechBubble()
assert speech.body is speech.submobjects[0]
assert speech.content is speech.submobjects[1]
assert np.isclose(speech.content.get_width(), 2.0, atol=1e-5)
assert np.isclose(speech.content.get_height(), 1.0, atol=1e-5)
assert np.isclose(speech.content.get_fill_opacity(), 0.0)
assert np.isclose(speech.content.get_stroke_width(), 0.0)
assert speech.body.get_fill_color() == manimlib.BLACK
assert np.isclose(speech.body.get_fill_opacity(), 0.8)
assert speech.body.get_stroke_color() == manimlib.WHITE
assert speech.get_tip()[1] < speech.content.get_bottom()[1]
circle_content = manimlib.Circle(radius=0.4)
circled = drawings.SpeechBubble(circle_content)
assert circled.content is circle_content

assert drawings.ThoughtBubble.__bases__ == (drawings.Bubble,)
assert list(inspect.signature(drawings.ThoughtBubble).parameters) == [
    "content",
    "buff",
    "filler_shape",
    "bulge_radius",
    "bulge_overlap",
    "noise_factor",
    "circle_radii",
    "kwargs",
]
thought = drawings.ThoughtBubble()
assert thought.body is thought.submobjects[0]
assert thought.content is thought.submobjects[1]
assert np.isclose(thought.bulge_radius, 0.35)
assert np.isclose(thought.bulge_overlap, 0.25)
assert np.isclose(thought.noise_factor, 0.1)
assert thought.circle_radii == [0.1, 0.15, 0.2]
assert len(thought.body.submobjects) == 4
assert all(
    isinstance(circle, manimlib.Circle)
    for circle in thought.body.submobjects[:3]
)
assert thought.body[-1].has_points()
assert thought.body[0].get_center()[1] < thought.body[-1].get_bottom()[1]
assert thought.body.get_fill_color() == manimlib.BLACK
assert thought.body.get_stroke_color() == manimlib.WHITE
right_thought = drawings.ThoughtBubble(
    direction=manimlib.RIGHT,
    noise_factor=0.0,
    circle_radii=[0.08, 0.12, 0.16],
)
assert right_thought.circle_radii == [0.08, 0.12, 0.16]
assert right_thought.direction[0] > 0

assert drawings.DoubleSpeechBubble.__bases__ == (drawings.Bubble,)
assert list(inspect.signature(drawings.DoubleSpeechBubble).parameters) == [
    "content",
    "buff",
    "filler_shape",
    "stem_height_to_bubble_height",
    "stem_top_x_props",
    "kwargs",
]
double_speech = drawings.DoubleSpeechBubble()
assert double_speech.body is double_speech.submobjects[0]
assert double_speech.content is double_speech.submobjects[1]
assert np.isclose(double_speech.content.get_width(), 2.0, atol=1e-5)
assert np.isclose(double_speech.content.get_height(), 1.0, atol=1e-5)
assert np.isclose(double_speech.stem_height_to_bubble_height, 0.5)
assert double_speech.stem_top_x_props == ((0.15, 0.25), (0.75, 0.85))
assert double_speech.body.has_points()
assert double_speech.body.get_bottom()[1] < double_speech.content.get_bottom()[1]
assert double_speech.body.get_fill_color() == manimlib.BLACK
assert np.isclose(double_speech.body.get_fill_opacity(), 0.8)
assert double_speech.body.get_stroke_color() == manimlib.WHITE
try:
    drawings.Bubble()
except NotImplementedError as error:
    assert "Bubbles_speech.svg" in str(error)
else:
    raise AssertionError("Bubble constructed an SVG-file body")

assert drawings.Piano.__bases__ == (manimlib.VGroup,)
assert list(inspect.signature(drawings.Piano).parameters) == [
    "n_white_keys",
    "black_pattern",
    "white_keys_per_octave",
    "white_key_dims",
    "black_key_dims",
    "key_buff",
    "white_key_color",
    "black_key_color",
    "total_width",
    "kwargs",
]
# One extra white key so the A-based pattern yields a full five-black octave.
piano = drawings.Piano(n_white_keys=8, total_width=4.0)
assert len(piano.white_keys) == 8
assert len(piano.black_keys) == 5
assert np.isclose(piano.get_width(), 4.0, atol=1e-5)
assert piano.white_keys[0].get_fill_color() == manimlib.WHITE
assert piano.black_keys[0].get_fill_color() == manimlib.GREY_E
assert np.isclose(piano.black_keys[0].get_stroke_width(), 0.0)
try:
    drawings.Piano(n_white_keys=0)
except ValueError as error:
    assert "positive" in str(error)
else:
    raise AssertionError("Piano accepted zero white keys")

assert drawings.Laptop.__bases__ == (manimlib.VGroup,)
assert list(inspect.signature(drawings.Laptop).parameters) == [
    "width",
    "body_dimensions",
    "screen_thickness",
    "keyboard_width_to_body_width",
    "keyboard_height_to_body_height",
    "screen_width_to_screen_plate_width",
    "key_color_kwargs",
    "fill_opacity",
    "stroke_width",
    "body_color",
    "shaded_body_color",
    "open_angle",
    "kwargs",
]
laptop = drawings.Laptop()
assert laptop.screen_plate is laptop.submobjects[1]
assert laptop.screen is laptop.screen_plate.submobjects[-1]
assert laptop.axis is laptop.submobjects[2]
assert len(laptop.submobjects) == 3
keyboard = laptop.submobjects[0].submobjects[-1]
assert [len(row) for row in keyboard] == [12, 11, 12, 11]
assert laptop.screen.get_fill_color() == manimlib.BLACK
assert laptop.axis.get_stroke_color() == manimlib.BLACK
wide_laptop = drawings.Laptop(width=4.0, body_color=manimlib.BLUE)
assert wide_laptop.submobjects[0][0].get_fill_color() == manimlib.GREY
assert wide_laptop.screen.get_fill_color() == manimlib.BLACK

assert drawings.Piano3D.__bases__ == (manimlib.VGroup,)
assert list(inspect.signature(drawings.Piano3D).parameters) == [
    "shading",
    "stroke_width",
    "stroke_color",
    "key_depth",
    "black_key_shift",
    "piano_2d_config",
    "kwargs",
]
piano3d = drawings.Piano3D(
    piano_2d_config=dict(
        n_white_keys=8,
        total_width=4.0,
        white_key_color=manimlib.GREY_A,
        key_buff=0.001,
    ),
)
assert len(piano3d) == 13
assert all(isinstance(key, manimlib.Prismify) for key in piano3d)
zs = [key.get_center()[2] for key in piano3d]
assert max(zs) - min(zs) >= 0.05 - 1e-6
assert piano3d[0].get_stroke_color() == manimlib.BLACK

assert drawings.OldSpeechBubble.__bases__ == (drawings.Bubble,)
assert drawings.OldSpeechBubble.file_name == "Bubbles_speech.svg"
try:
    drawings.OldSpeechBubble()
except NotImplementedError as error:
    assert "Bubbles_speech.svg" in str(error)
else:
    raise AssertionError("OldSpeechBubble constructed an SVG-file body")

assert drawings.OldThoughtBubble.__bases__ == (drawings.Bubble,)
assert drawings.OldThoughtBubble.file_name == "Bubbles_thought.svg"
assert callable(drawings.OldThoughtBubble.make_green_screen)
try:
    drawings.OldThoughtBubble()
except NotImplementedError as error:
    assert "Bubbles_thought.svg" in str(error)
else:
    raise AssertionError("OldThoughtBubble constructed an SVG-file body")

assert drawings.Lightbulb.__bases__ == (manimlib.SVGMobject,)
assert list(inspect.signature(drawings.Lightbulb).parameters) == [
    "height",
    "color",
    "stroke_width",
    "fill_opacity",
    "kwargs",
]
try:
    drawings.Lightbulb()
except NotImplementedError as error:
    assert "lightbulb" in str(error)
else:
    raise AssertionError("Lightbulb constructed without bundled SVG")

assert drawings.VideoIcon.__bases__ == (manimlib.SVGMobject,)
assert list(inspect.signature(drawings.VideoIcon).parameters) == [
    "width",
    "color",
    "kwargs",
]
try:
    drawings.VideoIcon()
except NotImplementedError as error:
    assert "video_icon" in str(error)
else:
    raise AssertionError("VideoIcon constructed without bundled SVG")

assert drawings.VectorizedEarth.__bases__ == (manimlib.SVGMobject,)
assert list(inspect.signature(drawings.VectorizedEarth).parameters) == [
    "height",
    "kwargs",
]
try:
    drawings.VectorizedEarth()
except NotImplementedError as error:
    assert "earth" in str(error)
else:
    raise AssertionError("VectorizedEarth constructed without bundled SVG")

assert drawings.VideoSeries.__bases__ == (manimlib.VGroup,)
assert list(inspect.signature(drawings.VideoSeries).parameters) == [
    "num_videos",
    "gradient_colors",
    "width",
    "kwargs",
]
try:
    drawings.VideoSeries()
except NotImplementedError as error:
    assert "video_icon" in str(error)
else:
    raise AssertionError("VideoSeries constructed without bundled VideoIcon SVG")

# ValueTracker targets are native typed state, not record-buffer decoration.
# Detached copy/deepcopy/pickle must preserve that payload so the ordinary
# `.animate` builder can mutate its generated target before Scene adoption.
assert manimlib.ValueTracker.value_type is np.float64
assert manimlib.ComplexValueTracker.value_type is np.complex128
assert manimlib.ExponentialValueTracker.__bases__ == (manimlib.ValueTracker,)
assert manimlib.ComplexValueTracker.__bases__ == (manimlib.ValueTracker,)
tracker_cases = (
    (manimlib.ValueTracker(1.25), 1.25),
    (manimlib.ExponentialValueTracker(4.0), 4.0),
    (manimlib.ComplexValueTracker(1.0 - 2.0j), 1.0 - 2.0j),
)
for tracker, expected in tracker_cases:
    assert isinstance(tracker.get_value(), tracker.value_type)
    for clone in (
        copy.copy(tracker),
        copy.deepcopy(tracker),
        pickle.loads(  # ubs:ignore -- trusted round-trip created immediately here
            pickle.dumps(tracker, protocol=pickle.HIGHEST_PROTOCOL)
        ),
    ):
        assert type(clone) is type(tracker)
        assert clone.get_value() == expected
        assert isinstance(clone.get_value(), clone.value_type)
        clone.increment_value(0.5)
        assert clone.get_value() == expected + 0.5
        assert tracker.get_value() == expected

plain_tracker_scene = Scene()
plain_tracker = manimlib.ValueTracker(1.0)
plain_tracker_values = []
plain_tracker.add_updater(
    lambda mob: plain_tracker_values.append(float(mob.get_value())),
    call=False,
)
plain_tracker_scene.play(
    plain_tracker.animate.set_value(5.0),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(plain_tracker_values, [3.0, 5.0, 5.0])
assert plain_tracker.get_value() == 5.0

exponential_tracker_scene = Scene()
exponential_tracker = manimlib.ExponentialValueTracker(4.0)
exponential_tracker_values = []
exponential_tracker.add_updater(
    lambda mob: exponential_tracker_values.append(float(mob.get_value())),
    call=False,
)
exponential_tracker_scene.play(
    exponential_tracker.animate.set_value(16.0),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(exponential_tracker_values, [8.0, 16.0, 16.0])
assert math.isclose(
    exponential_tracker.get_value(), 16.0, rel_tol=0.0, abs_tol=1e-12
)

complex_tracker_scene = Scene()
complex_tracker = manimlib.ComplexValueTracker(1.0 - 2.0j)
complex_tracker_values = []
complex_tracker.add_updater(
    lambda mob: complex_tracker_values.append(complex(mob.get_value())),
    call=False,
)
complex_tracker_scene.play(
    complex_tracker.animate.set_value(5.0 + 6.0j),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(
    complex_tracker_values,
    [3.0 + 2.0j, 5.0 + 6.0j, 5.0 + 6.0j],
)
assert complex_tracker.get_value() == 5.0 + 6.0j

# A vector value makes a vector tracker, as the Reference stores it in the
# "value" uniform (colliding_blocks' 4-d state, clt's (a, b) bounds).
# set_value assigns into it and broadcasts scalars; one component stays a
# scalar tracker; copies are independent; .animate blends componentwise.
vector_tracker = manimlib.ValueTracker(np.array([0.0, 10.0]))
assert isinstance(vector_tracker.get_value(), np.ndarray)
assert list(vector_tracker.set_value([1, 2]).get_value()) == [1.0, 2.0]
vector_tracker.increment_value(1)
assert list(vector_tracker.get_value()) == [2.0, 3.0]
assert list(vector_tracker.set_value(0).get_value()) == [0.0, 0.0]
vector_copy = vector_tracker.copy()
vector_copy.set_value([9, 9])
assert list(vector_tracker.get_value()) == [0.0, 0.0]
single_tracker = manimlib.ValueTracker(np.array([3.5]))
assert single_tracker.get_value() == 3.5 and not isinstance(single_tracker.get_value(), np.ndarray)
try:
    manimlib.ValueTracker(1.0).set_value([1, 2])
except ValueError:
    pass
else:
    raise AssertionError("a scalar tracker took a two-component value")
vector_tracker.set_value([0.0, 10.0])
vector_samples = []
vector_tracker.add_updater(
    lambda mob: vector_samples.append(np.array(mob.get_value())), call=False
)
Scene().play(
    vector_tracker.animate.set_value(np.array([4.0, 2.0])),
    run_time=4.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.array_equal(vector_tracker.get_value(), [4.0, 2.0])
vector_middle = [sample for sample in vector_samples if 0.0 < sample[0] < 4.0]
assert vector_middle and all(np.isclose(s[0] + s[1] / 2.0, 5.0) for s in vector_middle)

# set_submobjects commits only the difference when a request grows or trims
# the current prefix (ShowIncreasingSubsets does so every frame), and must
# reach exactly the state Reference remove-all-then-add leaves: requested
# order, identity deduplication, and this parent moved to the end of every
# kept child's parents.
prefix_parent, prefix_other = manimlib.VGroup(), manimlib.VGroup()
prefix_kids = [geometry.Dot() for _ in range(4)]
prefix_parent.add(prefix_kids[0])
prefix_other.add(prefix_kids[0])
assert prefix_kids[0].parents == [prefix_parent, prefix_other]
prefix_parent.set_submobjects([prefix_kids[0], prefix_kids[1], prefix_kids[1], prefix_kids[2]])
assert prefix_parent.submobjects == prefix_kids[:3]
assert prefix_kids[0].parents == [prefix_other, prefix_parent]
assert prefix_kids[2].parents == [prefix_parent]
prefix_parent.set_submobjects(prefix_kids[:1])
assert prefix_parent.submobjects == prefix_kids[:1]
assert prefix_parent not in prefix_kids[1].parents and prefix_parent not in prefix_kids[2].parents
try:
    prefix_parent.set_submobjects([prefix_kids[0], prefix_kids[3], "not a mobject"])
except TypeError:
    pass
else:
    raise AssertionError("set_submobjects accepted a non-Mobject child")
assert prefix_parent.submobjects == [prefix_kids[0], prefix_kids[3]]
wide_parent = manimlib.VGroup(*(geometry.Dot() for _ in range(300)))
wide_members = list(wide_parent.submobjects)
wide_parent.clear()
assert wide_parent.submobjects == [] and all(wide_parent not in dot.parents for dot in wide_members)

# Scene.finish_animations runs one final zero-dt updater traversal.  The
# display root intentionally precedes the derived root, so the ordinary frame
# leaves it one update behind after the source lands on its endpoint.  The
# finish pass must close that dependency in Reference root order before play
# returns (and before a final-state PNG is captured).
finish_scene = Scene()
finish_source = manimlib.ValueTracker(1.0)
finish_derived = manimlib.ValueTracker(1.0)
finish_display = manimlib.ValueTracker(-1.0)
finish_order = []


def update_finish_display(mob, dt):
    finish_order.append(("display", dt, float(finish_derived.get_value())))
    mob.set_value(finish_derived.get_value())


def update_finish_derived(mob, dt):
    finish_order.append(("derived", dt, float(finish_source.get_value())))
    mob.set_value(finish_source.get_value())


finish_display.add_updater(update_finish_display, call=False)
finish_derived.add_updater(update_finish_derived, call=False)
finish_scene.add(finish_display, finish_derived, finish_source)
finish_scene.play(
    finish_source.animate.set_value(7.0),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert finish_order == [
    ("display", 1.0 / 30.0, 1.0),
    ("derived", 1.0 / 30.0, 7.0),
    ("display", 0.0, 7.0),
    ("derived", 0.0, 7.0),
]
assert finish_display.get_value() == 7.0
assert finish_derived.get_value() == 7.0

# Engine-driven plays release the Scene RefCell after interpolation and the
# rational clock advance, then invoke Python scene updaters before native
# scene updaters and immutable capture. This used to refuse every such play.
release_scene = Scene()
release_mover = geometry.Rectangle(width=1.0, height=1.0)
release_observations = []
release_mover.add_updater(
    lambda mob, dt: release_observations.append(
        (dt, release_scene.time, mob.get_x())
    ),
    call=False,
)
release_scene.add(release_mover)
release_scene.play(
    release_mover.animate.shift(manimlib.RIGHT),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert len(release_observations) == 2
assert np.allclose(release_observations[0], [1.0 / 30.0, 1.0 / 30.0, 1.0])
assert np.allclose(release_observations[1], [0.0, 1.0 / 30.0, 1.0])
release_observations.clear()
release_scene.wait(1.0 / 30.0)
assert len(release_observations) == 2
assert np.allclose([row[0] for row in release_observations], [0.0, 1.0 / 30.0])
assert np.allclose([row[1] for row in release_observations], [1.0 / 30.0, 2.0 / 30.0])

# The public builder helpers preserve the pinned Reference's typed boundary,
# single-assignment animation arguments, and unchainable override contract.
animation_module = importlib.import_module("manimlib.animation.animation")
mobject_module = importlib.import_module("manimlib.mobject.mobject")
transform_module = importlib.import_module("manimlib.animation.transform")
assert manimlib.prepare_animation is animation_module.prepare_animation
assert manimlib.override_animate is mobject_module.override_animate
assert mobject_module._AnimationBuilder is type(geometry.Square().animate)
assert mobject_module._UpdaterBuilder is type(geometry.Square().always)
assert mobject_module._FunctionalUpdaterBuilder is type(
    geometry.Square().f_always
)
always_dot = geometry.Dot()
always_dot.always.shift(manimlib.RIGHT)
assert np.allclose(always_dot.get_center(), manimlib.RIGHT)
always_dot.update(0.0)
assert np.allclose(always_dot.get_center(), 2 * manimlib.RIGHT)
f_always_dot = geometry.Dot()
f_always_dot.f_always.move_to(lambda: manimlib.UP)
assert np.allclose(f_always_dot.get_center(), manimlib.UP)

builder_scene = Scene()
builder_mover = geometry.Rectangle(width=1.0, height=0.5)
builder_start = builder_mover.get_center().copy()
builder_scene.play(
    builder_mover.animate.shift(manimlib.RIGHT),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(builder_mover.get_center(), builder_start + manimlib.RIGHT)

chain_mover = geometry.Rectangle(width=1.0, height=0.5)
chain = chain_mover.animate(
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
).scale(2.0).shift(manimlib.UP)
try:
    chain.set_anim_args(run_time=1.0)
except ValueError as error:
    assert str(error) == (
        "Animation arguments can only be passed by calling ``animate`` "
        "or ``set_anim_args`` and can only be passed once"
    )
else:
    raise AssertionError("_AnimationBuilder accepted animation arguments twice")
prepared_chain = animation_module.prepare_animation(chain)
assert isinstance(prepared_chain, transform_module._MethodAnimation)
assert len(prepared_chain.methods) == 2
Scene().play(prepared_chain)
assert np.isclose(chain_mover.get_width(), 2.0)
assert np.allclose(chain_mover.get_center(), manimlib.UP)


class OverrideAnimateSquare(geometry.Square):
    def nudge(self, vector):
        return self.shift(vector)

    @mobject_module.override_animate(nudge)
    def _nudge_animation(self, vector, **kwargs):
        return manimlib.ApplyMethod(self.shift, vector, **kwargs)


override_scene = Scene()
override_mover = OverrideAnimateSquare()
override_scene.play(
    override_mover.animate.nudge(manimlib.RIGHT),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(override_mover.get_center(), manimlib.RIGHT)

for override_chain in (
    OverrideAnimateSquare().animate.nudge(manimlib.RIGHT),
    OverrideAnimateSquare().animate.shift(manimlib.UP),
):
    try:
        (
            override_chain.shift(manimlib.UP)
            if override_chain.overridden_animation is not None
            else override_chain.nudge(manimlib.RIGHT)
        )
    except NotImplementedError as error:
        assert str(error) == (
            "Method chaining is currently not supported for "
            "overridden animations"
        )
    else:
        raise AssertionError("_AnimationBuilder chained an overridden animation")

try:
    animation_module.prepare_animation(None)
except TypeError as error:
    assert str(error) == "Object None cannot be converted to an animation"
else:
    raise AssertionError("prepare_animation accepted None")

update_animation = importlib.import_module("manimlib.animation.update")
callback_scene = Scene()
callback_mover = geometry.Rectangle(width=1.0, height=1.0)
callback_alphas = []
callback_animation = update_animation.UpdateFromAlphaFunc(
    callback_mover,
    lambda mob, alpha: (
        callback_alphas.append(alpha),
        mob.set_x(alpha),
    )[-1],
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
callback_scene.play(callback_animation)
assert np.allclose(callback_alphas, [0.0, 1.0, 1.0])
assert np.allclose(callback_mover.get_x(), 1.0)

# The rotation family is an exact public Animation -> Rotating -> Rotate
# lineage over Choreo's absolute-pose mechanism.  Direct lifecycle calls
# preserve the pinned Reference surface, while Scene.play stays entirely on
# the native path and therefore never accumulates an earlier frame's turn.
rotation = importlib.import_module("manimlib.animation.rotation")
assert rotation.Rotating.__bases__ == (manimlib.Animation,)
assert rotation.Rotate.__bases__ == (rotation.Rotating,)
rotating_signature = inspect.signature(rotation.Rotating)
rotate_signature = inspect.signature(rotation.Rotate)
assert tuple(rotating_signature.parameters) == (
    "mobject",
    "angle",
    "axis",
    "about_point",
    "about_edge",
    "run_time",
    "rate_func",
    "suspend_mobject_updating",
    "kwargs",
)
assert tuple(rotate_signature.parameters) == (
    "mobject",
    "angle",
    "axis",
    "run_time",
    "rate_func",
    "about_edge",
    "kwargs",
)
assert rotating_signature.parameters["rate_func"].default is manimlib.linear
assert rotate_signature.parameters["rate_func"].default is manimlib.smooth
assert np.array_equal(
    rotate_signature.parameters["about_edge"].default,
    manimlib.ORIGIN,
)
assert str(inspect.signature(rotation.Rotating.interpolate_mobject)) == (
    "(self, alpha)"
)


def expected_z_rotation(points, angle):
    matrix = np.array(
        [
            [math.cos(angle), -math.sin(angle), 0.0],
            [math.sin(angle), math.cos(angle), 0.0],
            [0.0, 0.0, 1.0],
        ]
    )
    return np.asarray(points) @ matrix.T


standalone_rotating_mobject = geometry.Line(
    manimlib.RIGHT,
    2.0 * manimlib.RIGHT,
)
standalone_rotation_updates = []
standalone_rotating_mobject.add_updater(
    lambda _mob, dt: standalone_rotation_updates.append(dt),
    call=False,
)
standalone_start_points = standalone_rotating_mobject.get_points().copy()
standalone_rotation = rotation.Rotating(
    standalone_rotating_mobject,
    angle=math.pi,
    about_point=manimlib.ORIGIN,
    about_edge=manimlib.RIGHT,
    run_time=2.0,
    rate_func=manimlib.linear,
    time_span=(0.5, 1.5),
    suspend_mobject_updating=True,
)
standalone_rotation.begin()
assert standalone_rotation.starting_mobject is not standalone_rotating_mobject
assert standalone_rotating_mobject._is_updating_suspended()
standalone_rotation.interpolate(0.5)
assert np.allclose(
    standalone_rotating_mobject.get_points(),
    expected_z_rotation(standalone_start_points, math.pi / 2.0),
)
standalone_rotation.interpolate(0.75)
assert np.allclose(
    standalone_rotating_mobject.get_points(),
    expected_z_rotation(standalone_start_points, math.pi),
)
standalone_rotation.finish()
assert not standalone_rotating_mobject._is_updating_suspended()
assert standalone_rotation_updates == [0.0]
assert np.allclose(
    standalone_rotating_mobject.get_points(),
    expected_z_rotation(standalone_start_points, math.pi),
)

native_rotation_scene = Scene()
native_rotating_mobject = geometry.Line(
    manimlib.RIGHT,
    2.0 * manimlib.RIGHT,
)
native_rotation_samples = []
native_rotating_mobject.add_updater(
    lambda mob, dt: native_rotation_samples.append((dt, mob.get_center().copy())),
    call=False,
)
native_rotation_scene.play(
    rotation.Rotating(
        native_rotating_mobject,
        angle=math.pi,
        about_point=manimlib.ORIGIN,
        about_edge=manimlib.RIGHT,
        run_time=2.0 / 30.0,
        rate_func=manimlib.linear,
    )
)
assert len(native_rotation_samples) == 3
assert np.allclose(native_rotation_samples[0][1], [0.0, 1.5, 0.0])
assert np.allclose(native_rotation_samples[1][1], [-1.5, 0.0, 0.0])
assert np.allclose(native_rotation_samples[2][1], [-1.5, 0.0, 0.0])

suspended_rotation_scene = Scene()
suspended_rotating_mobject = geometry.Line(
    manimlib.RIGHT,
    2.0 * manimlib.RIGHT,
)
suspended_rotation_updates = []
suspended_rotating_mobject.add_updater(
    lambda mob, dt: suspended_rotation_updates.append((dt, mob.get_center().copy())),
    call=False,
)
suspended_rotation_scene.play(
    rotation.Rotating(
        suspended_rotating_mobject,
        angle=math.pi / 2.0,
        about_point=manimlib.ORIGIN,
        run_time=1.0 / 30.0,
        rate_func=manimlib.linear,
        suspend_mobject_updating=True,
    )
)
assert len(suspended_rotation_updates) == 2, suspended_rotation_updates
assert [sample[0] for sample in suspended_rotation_updates] == [0.0, 0.0]
assert np.allclose(
    [sample[1] for sample in suspended_rotation_updates],
    [[0.0, 1.5, 0.0], [0.0, 1.5, 0.0]],
)

rotation_composition_scene = Scene()
rotate_about_center = geometry.Line(manimlib.LEFT, manimlib.RIGHT).shift(
    2.0 * manimlib.LEFT
)
rotate_about_origin = geometry.Line(manimlib.RIGHT, 2.0 * manimlib.RIGHT)
rotation_composition_scene.play(
    manimlib.AnimationGroup(
        rotation.Rotate(rotate_about_center, angle=math.pi / 2.0),
        rotation.Rotating(
            rotate_about_origin,
            angle=math.pi / 2.0,
            about_point=manimlib.ORIGIN,
            run_time=1.0 / 30.0,
            rate_func=manimlib.linear,
        ),
    ),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(rotate_about_center.get_center(), 2.0 * manimlib.LEFT)
assert np.allclose(rotate_about_origin.get_center(), [0.0, 1.5, 0.0])

# The growing family is one authored Transform lineage over Choreo's native
# start-prep mechanism.  Anchors are frozen by the Python constructors like
# the pinned Reference, while the target is copied from the current source at
# play begin; no per-frame Python interpolation participates.
growing = importlib.import_module("manimlib.animation.growing")
assert growing.GrowFromPoint.__bases__ == (manimlib.Transform,)
assert growing.GrowFromCenter.__bases__ == (growing.GrowFromPoint,)
assert growing.GrowFromEdge.__bases__ == (growing.GrowFromPoint,)
assert growing.GrowArrow.__bases__ == (growing.GrowFromPoint,)
assert growing.SpinInFromNothing.__bases__ == (growing.GrowFromCenter,)
growing_call_shapes = {
    growing.GrowFromPoint: "(mobject, point, point_color=None, **kwargs)",
    growing.GrowFromCenter: "(mobject, **kwargs)",
    growing.GrowFromEdge: "(mobject, edge, **kwargs)",
    growing.GrowArrow: "(arrow, **kwargs)",
    growing.SpinInFromNothing: "(mobject, **kwargs)",
    growing.GrowFromPoint.__init__: (
        "(self, mobject, point, point_color=None, **kwargs)"
    ),
    growing.GrowFromCenter.__init__: "(self, mobject, **kwargs)",
    growing.GrowFromEdge.__init__: "(self, mobject, edge, **kwargs)",
    growing.GrowArrow.__init__: "(self, arrow, **kwargs)",
    growing.SpinInFromNothing.__init__: "(self, mobject, **kwargs)",
    growing.GrowFromPoint.create_target: "(self)",
    growing.GrowFromPoint.create_starting_mobject: "(self)",
}
for growing_surface, declared_call_shape in growing_call_shapes.items():
    actual_call_shape = str(inspect.signature(growing_surface))
    assert actual_call_shape == declared_call_shape

grow_point_source = geometry.Rectangle(width=2.0, height=1.0).set_color(
    manimlib.BLUE
)
assert np.isclose(grow_point_source.get_width(), 2.0)
grow_point = growing.GrowFromPoint(
    grow_point_source,
    2.0 * manimlib.LEFT,
    point_color=manimlib.RED,
    path_arc=math.pi / 6.0,
)
assert np.array_equal(grow_point.point, 2.0 * manimlib.LEFT)
assert grow_point.point_color == manimlib.RED
assert np.isclose(grow_point.path_arc, math.pi / 6.0)
grow_point_target = grow_point.create_target()
assert grow_point_target is not grow_point_source
assert np.array_equal(grow_point_target.get_points(), grow_point_source.get_points())
grow_point_start = grow_point.create_starting_mobject()
assert grow_point_start is not grow_point_source
assert np.allclose(grow_point_start.get_points(), 2.0 * manimlib.LEFT)
assert grow_point_start.get_color() == manimlib.RED
assert np.isclose(grow_point_source.get_width(), 2.0)

# Use a straight path for exact intermediate-center observations.  Moving the
# source after animation construction proves the fixed anchor and play-time
# target are distinct lifecycle moments.
grow_point.path_arc = 0.0
grow_point_source.shift(2.0 * manimlib.RIGHT)
grow_point_identity = id(grow_point_source)
grow_point_all_samples = []
grow_point_source.add_updater(
    lambda mob: grow_point_all_samples.append(
        (mob.get_center().copy(), mob.get_width(), mob.get_color(), mob is grow_point_source)
    ),
    call=False,
)

grow_point_scene = Scene()
grow_point_scene.play(
    grow_point,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert id(grow_point_source) == grow_point_identity
assert grow_point_scene.get_mobjects()[0] is grow_point_source
# The pinned Reference fires copied updaters on the animation's helper
# copies each frame (Animation.update_mobjects: starting mobject, then
# target copy), then on the source in the scene pass, and once more after
# play: 7 samples. Its 8th, first sample is add_updater(call=False)'s
# trailing update() pass, which BN-07 (C-5) removes.
assert [s[3] for s in grow_point_all_samples] == [False, False, True] * 2 + [True], (
    grow_point_all_samples
)
for grow_point_copy in (grow_point_all_samples[0], grow_point_all_samples[3]):
    assert np.allclose(grow_point_copy[0], 2.0 * manimlib.LEFT)
    assert np.isclose(grow_point_copy[1], 0.0) and grow_point_copy[2] == manimlib.RED
for grow_point_copy in (grow_point_all_samples[1], grow_point_all_samples[4]):
    assert np.allclose(grow_point_copy[0], 2.0 * manimlib.RIGHT)
    assert np.isclose(grow_point_copy[1], 2.0) and grow_point_copy[2] == manimlib.BLUE
grow_point_samples = [s[:3] for s in grow_point_all_samples if s[3]]
assert len(grow_point_samples) == 3, f"len={len(grow_point_samples)}, samples={grow_point_samples}"
assert np.allclose(grow_point_samples[0][0], manimlib.ORIGIN)
assert np.isclose(grow_point_samples[0][1], 1.0)
assert grow_point_samples[0][2] not in (manimlib.RED, manimlib.BLUE)
assert np.allclose(grow_point_samples[1][0], 2.0 * manimlib.RIGHT)
assert np.isclose(grow_point_samples[1][1], 2.0), grow_point_samples
assert grow_point_samples[1][2] == manimlib.BLUE
assert np.allclose(grow_point_samples[2][0], 2.0 * manimlib.RIGHT)

grow_center_source = geometry.Rectangle(width=1.0, height=0.5).shift(
    manimlib.RIGHT
)
grow_center = growing.GrowFromCenter(grow_center_source)
assert np.array_equal(grow_center.point, manimlib.RIGHT)
grow_center_source.shift(2.0 * manimlib.RIGHT)
grow_center_samples = []
# Helper copies fire the copied updater too (see GrowFromPoint above); keep
# the source's own samples. The Reference's mid sample is 3*RIGHT, because
# its anchor aliases the live bounding box; BN-12 section 5 fixes the anchor.
grow_center_source.add_updater(
    lambda mob: mob is grow_center_source
    and grow_center_samples.append(mob.get_center().copy()),
    call=False,
)
Scene().play(
    grow_center,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(
    grow_center_samples,
    [2.0 * manimlib.RIGHT, 3.0 * manimlib.RIGHT, 3.0 * manimlib.RIGHT],
)

grow_edge_source = geometry.Rectangle(width=2.0, height=1.0)
grow_edge = growing.GrowFromEdge(grow_edge_source, manimlib.RIGHT)
assert np.array_equal(grow_edge.point, manimlib.RIGHT)
grow_edge_source.shift(2.0 * manimlib.UP)
grow_edge_samples = []
grow_edge_source.add_updater(
    lambda mob: mob is grow_edge_source
    and grow_edge_samples.append(mob.get_center().copy()),
    call=False,
)
Scene().play(
    grow_edge,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(
    grow_edge_samples,
    [[0.5, 1.0, 0.0], [0.0, 2.0, 0.0], [0.0, 2.0, 0.0]],
)

grow_arrow_source = geometry.Arrow(
    manimlib.LEFT,
    manimlib.RIGHT,
    buff=0.0,
)
grow_arrow = growing.GrowArrow(grow_arrow_source)
assert np.allclose(grow_arrow.point, grow_arrow_source.get_start())
grow_arrow_anchor = grow_arrow.point.copy()
grow_arrow_source.shift(2.0 * manimlib.UP)
grow_arrow_samples = []
grow_arrow_source.add_updater(
    lambda mob: mob is grow_arrow_source
    and grow_arrow_samples.append(mob.get_start().copy()),
    call=False,
)
Scene().play(
    grow_arrow,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(grow_arrow_samples[0], grow_arrow_anchor + manimlib.UP)
assert np.allclose(grow_arrow_samples[1], grow_arrow_anchor + 2.0 * manimlib.UP)
assert np.allclose(grow_arrow_samples[2], grow_arrow_samples[1])

# BN-09: an arced Arrow re-tessellates when GrowArrow scales its start copy to
# zero, so the start can carry a different point count from the arrow
# (some1 LayersOfAbstraction grows path_arc=PI arrows). Transform re-aligns
# the start with the mobject and its target before the first interpolation.
for grow_arc in (1.0, math.pi, -math.pi):
    arced = geometry.Arrow(manimlib.LEFT, manimlib.RIGHT, path_arc=grow_arc)
    arced_points = arced.get_points().copy()
    arced_start = growing.GrowArrow(arced).create_starting_mobject()
    assert arced_start.get_width() < 1e-3 and arced_start.get_height() < 1e-3
    Scene().play(growing.GrowArrow(arced), run_time=2.0 / 30.0, rate_func=manimlib.linear)
    assert np.allclose(arced.get_points(), arced_points), grow_arc

spin_source = geometry.Line(manimlib.LEFT, manimlib.RIGHT)
spin_target_points = spin_source.get_points().copy()
spin_samples = []
spin_source.add_updater(
    lambda mob: mob is spin_source and spin_samples.append(mob.get_points().copy()),
    call=False,
)
spin = growing.SpinInFromNothing(spin_source)
assert np.isclose(spin.path_arc, math.pi)
Scene().play(
    spin,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert len(spin_samples) == 3
assert np.ptp(spin_samples[0][:, 0]) < np.ptp(spin_target_points[:, 0])
assert np.ptp(spin_samples[0][:, 1]) > 0.0
assert np.allclose(spin_samples[-1], spin_target_points)

composition_scene = Scene()
composition_point = geometry.Square().shift(manimlib.LEFT)
composition_edge = geometry.Rectangle(width=1.5, height=0.5).shift(
    manimlib.RIGHT
)
composition_scene.add(composition_point, composition_edge)
composition_ids = (id(composition_point), id(composition_edge))
composition_scene.play(
    manimlib.AnimationGroup(
        growing.GrowFromPoint(composition_point, manimlib.DOWN),
        growing.GrowFromEdge(composition_edge, manimlib.RIGHT),
    ),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert (id(composition_point), id(composition_edge)) == composition_ids
assert np.allclose(composition_point.get_center(), manimlib.LEFT)
assert np.allclose(composition_edge.get_center(), manimlib.RIGHT)

try:
    growing.GrowArrow(VMobject())
except ValueError as error:
    assert str(error) == "Cannot get points of Mobject with no points"
else:
    raise AssertionError("GrowArrow accepted a pointless mobject")

try:
    growing.SpinInFromNothing(manimlib.Mobject())
except TypeError as error:
    assert str(error) == (
        "SpinInFromNothing requires a VMobject with points; got Mobject"
    )
else:
    raise AssertionError("SpinInFromNothing accepted a non-VMobject")

try:
    growing.SpinInFromNothing(VMobject())
except ValueError as error:
    assert str(error) == (
        "SpinInFromNothing requires a VMobject with points; target is empty"
    )
else:
    raise AssertionError("SpinInFromNothing accepted a pointless VMobject")

# The remaining fading compatibility spellings are real Choreo-backed
# classes. FadeInFromLarge is the historical large-start convenience over
# FadeIn's native scale contract; FadeTransformPieces selects the native
# family-alignment variant; VFadeInThenOut selects its native opacity curve.
fading = importlib.import_module("manimlib.animation.fading")
assert fading.FadeInFromLarge.__bases__ == (fading.FadeIn,)
assert fading.FadeTransformPieces.__bases__ == (fading.FadeTransform,)
assert fading.VFadeInThenOut.__bases__ == (fading.VFadeIn,)
assert str(inspect.signature(fading.FadeInFromLarge)) == (
    "(mobject, scale_factor=2, **kwargs)"
)

large_fade_mobject = geometry.Rectangle(width=2.0, height=1.0)
large_fade_widths = []
large_fade_mobject.add_updater(
    lambda mob: large_fade_widths.append(mob.get_width()), call=False
)
Scene().play(
    fading.FadeInFromLarge(
        large_fade_mobject,
        scale_factor=2.0,
        run_time=2.0 / 30.0,
        rate_func=manimlib.linear,
    )
)
assert min(large_fade_widths) < large_fade_widths[-1]
assert np.isclose(large_fade_mobject.get_width(), 2.0)

pieces_source = geometry.Rectangle(width=2.0, height=1.0).shift(manimlib.LEFT)
pieces_target = geometry.Circle(radius=0.75).shift(manimlib.RIGHT)
pieces_scene = Scene()
pieces_scene.play(
    fading.FadeTransformPieces(
        pieces_source,
        pieces_target,
        run_time=2.0 / 30.0,
        rate_func=manimlib.linear,
    )
)
assert pieces_target in pieces_scene.get_mobjects()
assert pieces_source not in pieces_scene.get_mobjects()

for source, target, error_type, message in (
    (
        manimlib.Mobject(),
        geometry.Circle(),
        TypeError,
        "FadeTransformPieces requires a non-empty VMobject pair; source is Mobject",
    ),
    (
        VMobject(),
        geometry.Circle(),
        ValueError,
        "FadeTransformPieces requires a non-empty VMobject pair; source has no points",
    ),
):
    try:
        fading.FadeTransformPieces(source, target)
    except error_type as error:
        assert str(error) == message
    else:
        raise AssertionError("FadeTransformPieces accepted an invalid pair")

vfade_cycle = geometry.Circle(fill_opacity=0.8)
vfade_opacities = []
vfade_cycle.add_updater(
    lambda mob: vfade_opacities.append(mob.get_fill_opacity()), call=False
)
Scene().play(
    fading.VFadeInThenOut(vfade_cycle, run_time=2.0 / 30.0)
)
assert max(vfade_opacities) > min(vfade_opacities)

# fm-5wq.4.101: the remaining VFade constructor config reaches the shared
# native AnimConfig instead of stopping at the Python shell.  The default
# VFadeIn still plays, while updater suspension, remover, and finish alpha
# are observable through the native lifecycle.
vfade_in_scene = Scene()
vfade_in = geometry.Circle(fill_opacity=0.8)
vfade_in_scene.play(
    fading.VFadeIn(
        vfade_in,
        run_time=2.0 / 30.0,
        rate_func=manimlib.linear,
    )
)
assert vfade_in in vfade_in_scene.get_mobjects()
assert np.allclose(vfade_in.data["fill_rgba"][:, 3], 0.8)

suspended_vfade = geometry.Circle(fill_opacity=0.7)
suspended_vfade_updates = []
suspended_vfade.add_updater(
    lambda _mob, dt: suspended_vfade_updates.append(dt), call=False
)
Scene().play(
    fading.VFadeIn(
        suspended_vfade,
        run_time=1.0 / 30.0,
        rate_func=manimlib.linear,
        suspend_mobject_updating=True,
    )
)
assert suspended_vfade_updates == [0.0, 0.0], suspended_vfade_updates

kept_vfade_out = geometry.Circle(fill_opacity=0.9)
kept_vfade_out_scene = Scene()
kept_vfade_out_scene.add(kept_vfade_out)
kept_vfade_out_scene.play(
    fading.VFadeOut(
        kept_vfade_out,
        remover=False,
        final_alpha_value=1.0,
        run_time=1.0 / 30.0,
        rate_func=manimlib.linear,
    )
)
assert kept_vfade_out in kept_vfade_out_scene.get_mobjects()
assert np.allclose(kept_vfade_out.data["fill_rgba"][:, 3], 0.0)

kept_vfade_cycle = geometry.Circle(fill_opacity=0.6)
kept_vfade_cycle_scene = Scene()
kept_vfade_cycle_scene.add(kept_vfade_cycle)
kept_vfade_cycle_scene.play(
    fading.VFadeInThenOut(
        kept_vfade_cycle,
        remover=False,
        final_alpha_value=0.0,
        run_time=1.0 / 30.0,
    )
)
assert kept_vfade_cycle in kept_vfade_cycle_scene.get_mobjects()
assert np.allclose(kept_vfade_cycle.data["fill_rgba"][:, 3], 0.0)

try:
    fading.VFadeIn(None)
except TypeError as error:
    assert str(error) == "VFadeIn requires a VMobject; got NoneType"
else:
    raise AssertionError("VFadeIn accepted None")

try:
    fading.VFadeIn(geometry.Circle(), leftover_vfade_keyword=True)
except NotImplementedError as error:
    assert str(error) == (
        "VFadeIn() keyword(s) not yet routed to the native builder: "
        "leftover_vfade_keyword"
    )
else:
    raise AssertionError("an unrouted VFadeIn keyword reached native play")

# The mechanism-pure indication shelf is routed to Choreo.  These are live
# Scene.play checks: each animation must cross the native segment boundary and
# expose the intermediate state its mechanism owns, not merely construct.
indication = importlib.import_module("manimlib.animation.indication")
assert indication.Indicate.__bases__ == (manimlib.Transform,)
assert indication.TurnInsideOut.__bases__ == (manimlib.Transform,)
assert indication.WiggleOutThenIn.__bases__ == (manimlib.Animation,)
assert indication.VShowPassingFlash.__bases__ == (manimlib.Animation,)
assert indication.FlashAround.__bases__ == (indication.VShowPassingFlash,)
assert indication.FlashUnder.__bases__ == (indication.FlashAround,)
assert indication.ShowPassingFlashAround.__bases__ == (
    indication.AnimationOnSurroundingRectangle,
)
assert indication.ShowPassingFlashAround.RectAnimationType is indication.ShowPassingFlash
assert tuple(inspect.signature(indication.FlashAround).parameters) == (
    "mobject",
    "time_width",
    "taper_width",
    "stroke_width",
    "color",
    "buff",
    "n_inserted_curves",
    "kwargs",
)
assert inspect.signature(indication.FlashUnder) == inspect.signature(  # ubs:ignore -- public Python call signatures, not cryptographic material
    indication.FlashAround
)
assert tuple(inspect.signature(indication.ShowPassingFlashAround).parameters) == (
    "mobject",
    "stroke_width",
    "stroke_color",
    "buff",
    "kwargs",
)
assert indication.ShowCreationThenDestruction.__bases__ == (
    indication.ShowPassingFlash,
)
indicate_signature = inspect.signature(indication.Indicate)
assert tuple(indicate_signature.parameters) == (
    "mobject",
    "scale_factor",
    "color",
    "rate_func",
    "kwargs",
)
assert indicate_signature.parameters["scale_factor"].default == 1.2
assert indicate_signature.parameters["color"].default == manimlib.YELLOW
assert indicate_signature.parameters["rate_func"].default is manimlib.there_and_back
assert str(inspect.signature(indication.TurnInsideOut)) == (
    "(mobject, path_arc=1.5707963267948966, **kwargs)"
)
assert tuple(inspect.signature(indication.WiggleOutThenIn).parameters) == (
    "mobject",
    "scale_value",
    "rotation_angle",
    "n_wiggles",
    "scale_about_point",
    "rotate_about_point",
    "run_time",
    "kwargs",
)

passing_flash_signature = inspect.signature(indication.ShowPassingFlash)
assert tuple(passing_flash_signature.parameters) == (
    "mobject",
    "time_width",
    "remover",
    "kwargs",
)
plain_flash_scene = Scene()
plain_flash_mobject = geometry.Line(manimlib.LEFT, manimlib.RIGHT)
plain_flash = indication.ShowPassingFlash(
    plain_flash_mobject,
    time_width=0.4,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(plain_flash.get_bounds(0.5), (0.3, 0.7))
plain_flash_samples = []
plain_flash_mobject.add_updater(
    lambda mob: plain_flash_samples.append(mob.get_points().copy()), call=False
)
plain_flash_scene.play(plain_flash)
assert any(
    not np.allclose(points, plain_flash_samples[-1])
    for points in plain_flash_samples[:-1]
)
assert plain_flash_mobject not in plain_flash_scene.get_mobjects()

indicate_mobject = geometry.Rectangle(width=2.0, height=1.0).set_color(
    manimlib.BLUE
)
indicate_start_width = indicate_mobject.get_width()
indicate_samples = []
indicate_mobject.add_updater(
    lambda mob: indicate_samples.append((mob.get_width(), mob.get_color())),
    call=False,
)
Scene().play(
    indication.Indicate(indicate_mobject, run_time=2.0 / 30.0)
)
assert max(width for width, _color in indicate_samples) > indicate_start_width
assert any(color == manimlib.YELLOW for _width, color in indicate_samples)
assert np.isclose(indicate_mobject.get_width(), indicate_start_width)
assert indicate_mobject.get_color() == manimlib.BLUE

wiggle_mobject = geometry.Rectangle(width=2.0, height=1.0)
wiggle_start_width = wiggle_mobject.get_width()
wiggle_samples = []
wiggle_mobject.add_updater(
    lambda mob: wiggle_samples.append(mob.get_width()), call=False
)
Scene().play(
    indication.WiggleOutThenIn(
        wiggle_mobject,
        scale_value=1.25,
        rotation_angle=0.0,
        scale_about_point=manimlib.ORIGIN,
        rotate_about_point=manimlib.ORIGIN,
        run_time=2.0 / 30.0,
        rate_func=manimlib.linear,
    )
)
assert max(wiggle_samples) > wiggle_start_width
assert np.isclose(wiggle_mobject.get_width(), wiggle_start_width)

inside_out_mobject = geometry.Polygon(
    manimlib.LEFT,
    manimlib.UP,
    manimlib.RIGHT,
)
inside_out_start = inside_out_mobject.get_points().copy()
Scene().play(
    indication.TurnInsideOut(
        inside_out_mobject,
        path_arc=0.0,
        run_time=1.0 / 30.0,
        rate_func=manimlib.linear,
    )
)
assert np.array_equal(inside_out_mobject.get_points(), inside_out_start[::-1])

vflash_scene = Scene()
vflash_mobject = geometry.Line(manimlib.LEFT, manimlib.RIGHT).set_stroke(
    width=6.0
)
vflash_start_widths = vflash_mobject.get_stroke_widths().copy()
vflash_samples = []
vflash_mobject.add_updater(
    lambda mob: vflash_samples.append(mob.get_stroke_widths().copy()),
    call=False,
)
vflash_scene.play(
    indication.VShowPassingFlash(
        vflash_mobject,
        time_width=0.6,
        taper_width=0.1,
        run_time=2.0 / 30.0,
        rate_func=manimlib.linear,
    )
)
assert any(not np.allclose(widths, vflash_start_widths) for widths in vflash_samples)
assert np.allclose(vflash_mobject.get_stroke_widths(), vflash_start_widths)
assert vflash_mobject not in vflash_scene.get_mobjects()

destruction_scene = Scene()
destruction_mobject = geometry.Line(manimlib.LEFT, manimlib.RIGHT)
destruction_scene.play(
    indication.ShowCreationThenDestruction(
        destruction_mobject,
        time_width=1.5,
        run_time=2.0 / 30.0,
        rate_func=manimlib.linear,
    )
)
assert destruction_mobject not in destruction_scene.get_mobjects()

wave_mobject = geometry.Line(manimlib.LEFT, manimlib.RIGHT)
wave_start = wave_mobject.get_points().copy()
wave_samples = []
wave_mobject.add_updater(
    lambda mob: wave_samples.append(mob.get_points().copy()), call=False
)
Scene().play(
    indication.ApplyWave(
        wave_mobject,
        direction=manimlib.UP,
        amplitude=0.4,
        run_time=2.0 / 30.0,
        rate_func=manimlib.linear,
    )
)
assert any(np.max(points[:, 1] - wave_start[:, 1]) > 0.0 for points in wave_samples)
assert np.allclose(wave_mobject.get_points(), wave_start)

indication_composition_scene = Scene()
composition_indicate = geometry.Square().shift(manimlib.LEFT)
composition_wiggle = geometry.Square().shift(manimlib.RIGHT)
indication_composition_scene.play(
    manimlib.AnimationGroup(
        indication.Indicate(composition_indicate),
        indication.WiggleOutThenIn(
            composition_wiggle,
            rotation_angle=0.0,
        ),
    ),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(composition_indicate.get_center(), manimlib.LEFT)
assert np.allclose(composition_wiggle.get_center(), manimlib.RIGHT)

around_target = geometry.Rectangle(width=2.0, height=1.0).shift(manimlib.LEFT)
around_scene = Scene()
around_scene.add(around_target)
around_flash = indication.FlashAround(
    around_target,
    time_width=0.6,
    stroke_width=5.0,
    color=manimlib.GREEN,
    buff=0.2,
    n_inserted_curves=12,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert isinstance(around_flash.mobject, shape_matchers.SurroundingRectangle)
assert np.allclose(
    around_flash.mobject.get_bounding_box()[0][:2],
    around_target.get_bounding_box()[0][:2] - 0.2,
)
assert around_flash.mobject.get_stroke_color() == manimlib.GREEN
around_samples = []
around_flash.mobject.add_updater(
    lambda mob: around_samples.append(mob.get_stroke_widths().copy()),
    call=False,
)
around_scene.play(around_flash)
assert any(
    widths.max() > 0.0 and not np.allclose(widths, widths[0])
    for widths in around_samples
)
assert around_target in around_scene.get_mobjects()
assert around_flash.mobject not in around_scene.get_mobjects()

under_target = geometry.Rectangle(width=2.5, height=1.0)
under_scene = Scene()
under_scene.add(under_target)
under_flash = indication.FlashUnder(
    under_target,
    buff=0.15,
    n_inserted_curves=8,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert isinstance(under_flash.mobject, shape_matchers.Underline)
assert np.isclose(
    under_flash.mobject.get_center()[1],
    under_target.get_bottom()[1] - 0.15,
)
under_scene.play(under_flash)
assert under_flash.mobject not in under_scene.get_mobjects()

tracked_target = geometry.Square().shift(manimlib.RIGHT)
tracked_scene = Scene()
tracked_scene.add(tracked_target)
tracked_flash = indication.ShowPassingFlashAround(
    tracked_target,
    stroke_width=3.0,
    stroke_color=manimlib.BLUE,
    buff=0.25,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert isinstance(tracked_flash.rectangle, shape_matchers.SurroundingRectangle)
assert tracked_flash.rectangle.get_stroke_color() == manimlib.BLUE
tracked_full_path = tracked_flash.rectangle.get_points().copy()
tracked_full_length = tracked_flash.rectangle.get_arc_length()
tracked_samples = []
tracked_flash.rectangle.add_updater(
    lambda mob: tracked_samples.append((
        mob.get_arc_length(), mob.get_center().copy(), tracked_target.get_center().copy(),
    )),
    call=False,
)
tracked_target.add_updater(
    lambda mob, dt: mob.shift(dt * manimlib.RIGHT),
    call=False,
)
tracked_scene.play(tracked_flash)
assert any(
    np.isclose(length, 0.1 * tracked_full_length)
    for length, _, _ in tracked_samples
)
assert all(np.allclose(center, target) for _, center, target in tracked_samples)
# ShowPassingFlash restores its original complete path after the final sample,
# then removes the helper. Tracking is asserted above while the helper is live.
assert np.array_equal(tracked_flash.rectangle.get_points(), tracked_full_path)
assert tracked_flash.rectangle not in tracked_scene.get_mobjects()
assert tracked_target in tracked_scene.get_mobjects()

custom_flash_rectangles = []


class AuthoredRectangleFlash(indication.ShowPassingFlash):
    def __init__(self, rectangle, **kwargs):
        custom_flash_rectangles.append(rectangle)
        super().__init__(rectangle, **kwargs)


class AuthoredFlashAround(indication.ShowPassingFlashAround):
    RectAnimationType = AuthoredRectangleFlash


custom_around = AuthoredFlashAround(
    tracked_target, run_time=2.0 / 30.0, rate_func=manimlib.linear,
)
assert len(custom_flash_rectangles) == 1
assert custom_flash_rectangles[0] is custom_around.rectangle
assert isinstance(custom_around.animations[0], AuthoredRectangleFlash)
tracked_scene.play(custom_around)
assert custom_around.rectangle not in tracked_scene.get_mobjects()
assert tracked_target in tracked_scene.get_mobjects()

for flash_type in (
    indication.FlashAround,
    indication.FlashUnder,
    indication.ShowPassingFlashAround,
):
    try:
        flash_type(None)
    except TypeError as error:
        assert str(error) == f"{flash_type.__name__} expects a Mobject"
    else:
        raise AssertionError(f"{flash_type.__name__} accepted None")

# The remaining Reference indication compositions use Atlas geometry above
# Choreo's native Transform/fade/composition mechanisms. Static point inputs
# stay wholly native; moving-Mobject targets refuse by class name until the
# animation-owned live-target updater seam lands.
assert indication.FocusOn.__bases__ == (manimlib.Transform,)
assert indication.Flash.__bases__ == (manimlib.AnimationGroup,)
assert indication.CircleIndicate.__bases__ == (manimlib.Transform,)
assert indication.ShowCreationThenFadeOut.__bases__ == (manimlib.Succession,)
assert tuple(inspect.signature(indication.FocusOn).parameters) == (
    "focus_point",
    "opacity",
    "color",
    "run_time",
    "remover",
    "kwargs",
)
assert tuple(inspect.signature(indication.Flash).parameters) == (
    "point",
    "color",
    "line_length",
    "num_lines",
    "flash_radius",
    "line_stroke_width",
    "run_time",
    "kwargs",
)

focus = indication.FocusOn(
    [1.25, -0.5, 0.0],
    color=manimlib.BLUE,
    opacity=0.35,
    run_time=2.0 / 30.0,
)
assert np.allclose(focus.mobject.get_center(), [1.25, -0.5, 0.0])
assert focus.mobject.get_width() > focus.target_mobject.get_width()
focus_scene = Scene()
focus_scene.play(focus, rate_func=manimlib.linear)
assert focus.mobject not in focus_scene.get_mobjects()

flash = indication.Flash(
    [-1.0, 0.5, 0.0],
    color=manimlib.GREEN,
    line_length=0.4,
    num_lines=6,
    flash_radius=0.7,
    line_stroke_width=5.0,
    run_time=2.0 / 30.0,
)
assert len(flash.lines) == 6
assert all(
    np.isclose(line.get_length(), 0.4)
    and line.get_stroke_color() == manimlib.GREEN
    for line in flash.lines
)
flash_scene = Scene()
flash_scene.play(flash, rate_func=manimlib.linear)
assert all(line not in flash_scene.get_mobjects() for line in flash.lines)

circle_target = geometry.Rectangle(width=2.0, height=1.0)
circle_scene = Scene()
circle_scene.add(circle_target)
circle_indication = indication.CircleIndicate(
    circle_target,
    scale_factor=1.5,
    stroke_color=manimlib.YELLOW,
    stroke_width=4.0,
    run_time=2.0 / 30.0,
)
assert circle_indication.target_mobject.get_width() > circle_indication.mobject.get_width()
circle_scene.play(circle_indication, rate_func=manimlib.linear)
assert circle_target in circle_scene.get_mobjects()
assert circle_indication.mobject not in circle_scene.get_mobjects()

try:
    indication.CircleIndicate(None)
except TypeError as error:
    assert str(error) == "CircleIndicate expects a Mobject; got NoneType"
else:
    raise AssertionError("CircleIndicate accepted None")

fade_scene = Scene()
fade_mobject = geometry.Line(manimlib.LEFT, manimlib.RIGHT)
fade_scene.play(
    indication.ShowCreationThenFadeOut(
        fade_mobject,
        run_time=2.0 / 30.0,
    ),
    rate_func=manimlib.linear,
)
assert fade_mobject not in fade_scene.get_mobjects()

try:
    indication.Flash(manimlib.ORIGIN, num_lines=0)
except ValueError as error:
    assert str(error) == "Flash num_lines must be greater than zero"
else:
    raise AssertionError("Flash accepted num_lines=0")

# fm-5wq.4.77: FocusOn Mobject targets follow live now — construction
# succeeds, samples the target's current centre, and hangs the follow
# updater on the shrinking dot; the old updater-seam refusal is retired
# (the play-level tracking pin lives with the FocusOn block below).
focus_follow_probe = indication.FocusOn(geometry.Square())
assert np.allclose(focus_follow_probe.focus_point, [0.0, 0.0, 0.0])
assert focus_follow_probe._focus_target is not None

# Specialized Broadcast and the final composition-backed indication leaves
# are ordinary native compositions: LaggedStart/Restore for expanding rings,
# FadeIn plus a passing outline for FlashyFadeIn, and surrounding-rectangle
# creation/fade compositions that keep tracking their target.
specialized = importlib.import_module("manimlib.animation.specialized")
assert specialized.Broadcast.__bases__ == (manimlib.LaggedStart,)
assert indication.FlashyFadeIn.__bases__ == (manimlib.AnimationGroup,)
assert indication.AnimationOnSurroundingRectangle.__bases__ == (
    manimlib.AnimationGroup,
)
assert indication.ShowCreationThenDestructionAround.__bases__ == (
    indication.AnimationOnSurroundingRectangle,
)
assert indication.ShowCreationThenFadeAround.__bases__ == (
    indication.AnimationOnSurroundingRectangle,
)

broadcast_focus = geometry.Dot([1.0, -0.5, 0.0])
broadcast_scene = Scene()
broadcast_scene.add(broadcast_focus)
broadcast = specialized.Broadcast(
    broadcast_focus,
    small_radius=0.05,
    big_radius=1.5,
    n_circles=3,
    start_stroke_width=6.0,
    color=manimlib.BLUE,
    run_time=2.0 / 30.0,
    lag_ratio=0.2,
)
assert len(broadcast.circles) == 3
assert all(circle.saved_state is not None for circle in broadcast.circles)
broadcast_scene.play(broadcast, rate_func=manimlib.linear)
assert broadcast_focus in broadcast_scene.get_mobjects()
assert all(
    circle not in broadcast_scene.get_mobjects()
    for circle in broadcast.circles
)

flashy_mobject = geometry.Square().set_fill(manimlib.BLUE, opacity=1.0)
flashy_scene = Scene()
flashy = indication.FlashyFadeIn(
    flashy_mobject,
    stroke_width=5.0,
    fade_lag=0.25,
    time_width=0.6,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(flashy.outline.data["fill_rgba"][:, 3], 0.0)
flashy_scene.play(flashy)
assert flashy_mobject in flashy_scene.get_mobjects()
assert flashy.outline not in flashy_scene.get_mobjects()

destruction_target = geometry.Rectangle(width=2.0, height=1.0)
destruction_around_scene = Scene()
destruction_around_scene.add(destruction_target)
destruction_around = indication.ShowCreationThenDestructionAround(
    destruction_target,
    stroke_width=4.0,
    stroke_color=manimlib.GREEN,
    buff=0.2,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
destruction_around_scene.play(destruction_around)
assert destruction_target in destruction_around_scene.get_mobjects()
assert destruction_around.rectangle not in destruction_around_scene.get_mobjects()

fade_around_target = geometry.Square()
fade_around_scene = Scene()
fade_around_scene.add(fade_around_target)
fade_around = indication.ShowCreationThenFadeAround(
    fade_around_target,
    stroke_width=3.0,
    stroke_color=manimlib.YELLOW,
    buff=0.15,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
fade_around_scene.play(fade_around)
assert fade_around_target in fade_around_scene.get_mobjects()
assert fade_around.rectangle not in fade_around_scene.get_mobjects()

try:
    specialized.Broadcast(None)
except TypeError as error:
    assert "manimlib.animation.specialized.Broadcast" in str(error)
    assert "NoneType" in str(error)
else:
    raise AssertionError("Broadcast accepted None")

# Restore is a Transform onto Marionette's saved-state copy, so its standard
# path-arc parameters route through the same native path function.
restore_scene = Scene()
restore_mover = geometry.Rectangle(width=1.0, height=1.0)
restore_scene.add(restore_mover)
restore_mover.save_state()
assert restore_mover.saved_state is not None
assert restore_mover.saved_state.target is restore_mover.target
restore_mover.shift([2.0, 1.0, 0.0])
restore_scene.play(
    manimlib.Restore(restore_mover, path_arc=math.pi / 3.0),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(restore_mover.get_center(), [0.0, 0.0, 0.0])

# A saved state is a real family copy even before scene adoption. The first
# animation may adopt the already-mutated source; Restore must still target
# the earlier detached state, including its child graph.
detached_restore_scene = Scene()
detached_restore_child = geometry.Rectangle(width=0.5, height=0.5)
detached_restore_mover = manimlib.VGroup(detached_restore_child)
detached_restore_mover.save_state()
detached_saved = detached_restore_mover.saved_state
detached_restore_mover.shift([3.0, -2.0, 0.0])
detached_restore_scene.play(
    manimlib.Restore(detached_restore_mover, path_arc=math.pi / 6.0),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert detached_restore_mover.saved_state is detached_saved
assert np.allclose(detached_restore_mover.get_center(), [0.0, 0.0, 0.0])
assert np.allclose(detached_restore_child.get_center(), [0.0, 0.0, 0.0])

state_links = geometry.Rectangle(width=0.25, height=0.25)
first_target = state_links.generate_target()
state_links.save_state()
assert state_links.saved_state.target is first_target
second_target = state_links.generate_target()
assert second_target.saved_state is state_links.saved_state
plain_state_copy = state_links.copy()
assert plain_state_copy.target is None
assert plain_state_copy.saved_state is None

explicit_target_scene = Scene()
explicit_target_mover = geometry.Rectangle(width=0.5, height=0.5)
explicit_target_scene.add(explicit_target_mover)
explicit_target_mover.generate_target().shift([1.25, 0.0, 0.0])


class ValidatedMoveToTarget(manimlib.MoveToTarget):
    def check_validity_of_input(self, mobject):
        self.validated_source = mobject
        super().check_validity_of_input(mobject)


validated_move = ValidatedMoveToTarget(explicit_target_mover)
assert validated_move.validated_source is explicit_target_mover
assert validated_move.target_mobject is explicit_target_mover.target
assert validated_move.check_validity_of_input(explicit_target_mover) is None
try:
    ValidatedMoveToTarget(Mobject())
except Exception as error:
    assert str(error) == "MoveToTarget called on mobject without attribute 'target'"
else:
    raise AssertionError("MoveToTarget accepted an unset target")
explicit_target_scene.play(
    validated_move,
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(explicit_target_mover.get_center(), [1.25, 0.0, 0.0])

# ApplyMethod is the Reference's method-to-target adapter over Transform.  It
# must keep the bound method and its arguments as public state, build a fresh
# independent target at play time, and route that target through the native
# transform mechanism rather than treating the Python method as a Mobject.
assert manimlib.ApplyMethod.__bases__ == (manimlib.Transform,)
assert str(inspect.signature(manimlib.ApplyMethod)) == "(method, *args, **kwargs)"
assert str(inspect.signature(manimlib.ApplyMethod.__init__)) == (
    "(self, method, *args, **kwargs)"
)
assert str(inspect.signature(manimlib.ApplyMethod.check_validity_of_input)) == (
    "(self, method)"
)
assert str(inspect.signature(manimlib.ApplyMethod.create_target)) == "(self)"

apply_target_source = geometry.Rectangle(width=1.0, height=0.5)
apply_scale = manimlib.ApplyMethod(
    apply_target_source.scale,
    2.0,
    {"about_point": manimlib.RIGHT},
    path_arc=math.pi / 6.0,
)
assert apply_scale.method.__self__ is apply_target_source
assert apply_scale.method.__func__ is type(apply_target_source).scale
assert apply_scale.method_args[0] == 2.0
assert list(apply_scale.method_args[1]) == ["about_point"]
assert np.allclose(apply_scale.method_args[1]["about_point"], manimlib.RIGHT)
assert apply_scale.target_mobject is None
apply_scale_target = apply_scale.create_target()
assert apply_scale_target is not apply_target_source
assert np.allclose(apply_target_source.get_center(), manimlib.ORIGIN)
assert np.allclose(apply_scale_target.get_center(), manimlib.LEFT)
assert np.isclose(apply_scale_target.get_width(), 2.0)

delayed_apply_scene = Scene()
delayed_apply_source = geometry.Rectangle(width=1.0, height=0.5)
delayed_apply = manimlib.ApplyMethod(delayed_apply_source.shift, manimlib.RIGHT)
delayed_apply_source.shift(manimlib.UP)
delayed_apply_samples = []
delayed_apply_source.add_updater(
    lambda mob: delayed_apply_samples.append(mob.get_center().copy())
    if mob is delayed_apply_source
    else None,
    call=False,
)
delayed_apply_scene.play(
    delayed_apply,
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(
    delayed_apply_samples,
    [[0.5, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 0.0]],
)
assert np.allclose(delayed_apply_source.get_center(), [1.0, 1.0, 0.0])
assert delayed_apply.target_mobject is not None
assert delayed_apply.target_mobject is not delayed_apply_source

apply_tracker_scene = Scene()
apply_tracker = manimlib.ValueTracker(1.0)
apply_tracker_samples = []
apply_tracker.add_updater(
    lambda mob: apply_tracker_samples.append(float(mob.get_value()))
    if mob is apply_tracker
    else None,
    call=False,
)
apply_tracker_scene.play(
    manimlib.ApplyMethod(apply_tracker.set_value, 5.0),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(apply_tracker_samples, [3.0, 5.0, 5.0])
assert apply_tracker.get_value() == 5.0

try:
    manimlib.ApplyMethod(lambda: None)
except Exception as error:
    assert str(error) == (
        "Whoops, looks like you accidentally invoked the method you want to animate"
    )
else:
    raise AssertionError("ApplyMethod accepted a free function")


class ForeignApplyTarget:
    def mutate(self):
        return self


try:
    manimlib.ApplyMethod(ForeignApplyTarget().mutate)
except AssertionError:
    pass
else:
    raise AssertionError("ApplyMethod accepted a bound non-Mobject method")

# The thin transform adapters are authored over the same play-time target
# seam.  Their documented argument order must not fall through to inherited
# ApplyMethod/Transform constructors, and each adapter must preserve the
# Reference target operation while Lumen's native Transform owns playback.
adapter_call_shapes = {
    manimlib.ApplyPointwiseFunction: "(function, mobject, run_time=3.0, **kwargs)",
    manimlib.ApplyPointwiseFunctionToCenter: "(function, mobject, **kwargs)",
    manimlib.FadeToColor: "(mobject, color, **kwargs)",
    manimlib.ScaleInPlace: "(mobject, scale_factor, **kwargs)",
    manimlib.ShrinkToCenter: "(mobject, **kwargs)",
    manimlib.ApplyFunction: "(function, mobject, **kwargs)",
    manimlib.ApplyMatrix: "(matrix, mobject, **kwargs)",
    manimlib.ApplyComplexFunction: "(function, mobject, **kwargs)",
}
for adapter, declared_call_shape in adapter_call_shapes.items():
    actual_call_shape = str(inspect.signature(adapter))
    assert actual_call_shape == declared_call_shape
adapter_method_call_shapes = {
    manimlib.ApplyPointwiseFunction.__init__: (
        "(self, function, mobject, run_time=3.0, **kwargs)"
    ),
    manimlib.ApplyPointwiseFunctionToCenter.__init__: (
        "(self, function, mobject, **kwargs)"
    ),
    manimlib.ApplyPointwiseFunctionToCenter.create_target: "(self)",
    manimlib.FadeToColor.__init__: "(self, mobject, color, **kwargs)",
    manimlib.ScaleInPlace.__init__: "(self, mobject, scale_factor, **kwargs)",
    manimlib.ShrinkToCenter.__init__: "(self, mobject, **kwargs)",
    manimlib.ApplyFunction.__init__: "(self, function, mobject, **kwargs)",
    manimlib.ApplyFunction.create_target: "(self)",
    manimlib.ApplyMatrix.__init__: "(self, matrix, mobject, **kwargs)",
    manimlib.ApplyMatrix.initialize_matrix: "(self, matrix)",
    manimlib.ApplyComplexFunction.__init__: "(self, function, mobject, **kwargs)",
    manimlib.ApplyComplexFunction.init_path_func: "(self)",
}
for adapter_method, declared_call_shape in adapter_method_call_shapes.items():
    actual_call_shape = str(inspect.signature(adapter_method))
    assert actual_call_shape == declared_call_shape
assert manimlib.ApplyPointwiseFunction.__bases__ == (manimlib.ApplyMethod,)
assert manimlib.ApplyPointwiseFunctionToCenter.__bases__ == (manimlib.Transform,)
assert manimlib.FadeToColor.__bases__ == (manimlib.ApplyMethod,)
assert manimlib.ScaleInPlace.__bases__ == (manimlib.ApplyMethod,)
assert manimlib.ShrinkToCenter.__bases__ == (manimlib.ScaleInPlace,)
assert manimlib.ApplyFunction.__bases__ == (manimlib.Transform,)
assert manimlib.ApplyMatrix.__bases__ == (manimlib.ApplyPointwiseFunction,)
assert manimlib.ApplyComplexFunction.__bases__ == (manimlib.ApplyMethod,)

pointwise_source = geometry.Rectangle(width=1.0, height=0.5)
pointwise_animation = manimlib.ApplyPointwiseFunction(
    lambda point: np.array(
        [2.0 * point[0] + 1.0, point[1] - 0.5, point[2]]
    ),
    pointwise_source,
)
assert pointwise_animation.run_time == 3.0
Scene().play(
    pointwise_animation,
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(pointwise_source.get_center(), [1.0, -0.5, 0.0])
assert np.isclose(pointwise_source.get_width(), 2.0)

try:
    manimlib.ApplyPointwiseFunction(None, geometry.Rectangle())
except TypeError as error:
    assert str(error) == (
        "ApplyPointwiseFunction function must be callable; got NoneType"
    )
else:
    raise AssertionError("ApplyPointwiseFunction accepted a non-callable function")

center_source = geometry.Rectangle(width=1.0, height=0.5)
center_animation = manimlib.ApplyPointwiseFunctionToCenter(
    lambda point: point + 2.0 * manimlib.RIGHT,
    center_source,
)
center_source.shift(manimlib.UP)
Scene().play(
    center_animation,
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(center_source.get_center(), [2.0, 1.0, 0.0])

try:
    manimlib.ApplyPointwiseFunctionToCenter(None, geometry.Rectangle())
except TypeError as error:
    assert str(error) == (
        "ApplyPointwiseFunctionToCenter function must be callable; got NoneType"
    )
else:
    raise AssertionError(
        "ApplyPointwiseFunctionToCenter accepted a non-callable function"
    )

fade_source = geometry.Square().set_color(manimlib.BLUE)
Scene().play(
    manimlib.FadeToColor(fade_source, manimlib.RED),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert fade_source.get_color() == manimlib.RED

scale_source = geometry.Rectangle(width=1.0, height=0.5)
scale_scene = Scene()
scale_scene.play(
    manimlib.ScaleInPlace(scale_source, 2.0),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.isclose(scale_source.get_width(), 2.0)
scale_scene.play(
    manimlib.ShrinkToCenter(scale_source),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.isclose(scale_source.get_width(), 2.0e-8)

apply_function_calls = []


def build_apply_target(mobject):
    apply_function_calls.append(mobject.get_center().copy())
    return mobject.scale(2.0).shift(manimlib.RIGHT)


function_source = geometry.Rectangle(width=1.0, height=0.5)
function_animation = manimlib.ApplyFunction(build_apply_target, function_source)
function_scene = Scene()
function_source.shift(manimlib.UP)
assert apply_function_calls == []
function_scene.play(
    function_animation,
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert len(apply_function_calls) == 1
assert np.allclose(apply_function_calls[0], manimlib.UP)
assert np.allclose(function_source.get_center(), [1.0, 1.0, 0.0])
assert np.isclose(function_source.get_width(), 2.0)
function_scene.play(
    function_animation,
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert len(apply_function_calls) == 2
assert np.allclose(apply_function_calls[1], [1.0, 1.0, 0.0])
assert np.allclose(function_source.get_center(), [2.0, 1.0, 0.0])
assert np.isclose(function_source.get_width(), 4.0)

invalid_function_animation = manimlib.ApplyFunction(
    lambda _mobject: 7,
    geometry.Square(),
)
try:
    Scene().play(invalid_function_animation)
except Exception as error:
    assert str(error) == (
        "Functions passed to ApplyFunction must return object of type Mobject"
    )
else:
    raise AssertionError("ApplyFunction accepted a non-Mobject target")

matrix_source = geometry.Rectangle(width=1.0, height=0.5).shift(manimlib.RIGHT)
matrix_animation = manimlib.ApplyMatrix([[0.0, -1.0], [1.0, 0.0]], matrix_source)
assert np.array_equal(
    matrix_animation.initialize_matrix([[1.0, 0.0], [0.0, 1.0]]),
    np.identity(3),
)
Scene().play(
    matrix_animation,
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(matrix_source.get_center(), manimlib.UP)
try:
    manimlib.ApplyMatrix([[1.0]], geometry.Square())
except ValueError as error:
    assert str(error) == (
        "ApplyMatrix matrix must have shape (2, 2) or (3, 3); got (1, 1)"
    )
else:
    raise AssertionError("ApplyMatrix accepted a matrix with bad dimensions")

try:
    manimlib.ApplyMatrix([[1.0], [2.0, 3.0]], geometry.Square())
except TypeError as error:
    assert str(error) == (
        "ApplyMatrix matrix must be a rectangular 2x2 or 3x3 array"
    )
else:
    raise AssertionError("ApplyMatrix accepted a ragged matrix")

complex_source = geometry.Rectangle(width=1.0, height=0.5).shift(manimlib.RIGHT)
complex_animation = manimlib.ApplyComplexFunction(lambda value: 1j * value, complex_source)
assert np.isclose(complex_animation.path_arc, math.pi / 2.0)
complex_animation.path_arc = 0.0
assert complex_animation.init_path_func() is None
assert np.isclose(complex_animation.path_arc, math.pi / 2.0)
Scene().play(
    complex_animation,
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(complex_source.get_center(), manimlib.UP)

sampled_rate_scene = Scene()
sampled_rate_mover = geometry.Rectangle(width=0.5, height=0.5)
sampled_rate_scene.play(
    sampled_rate_mover.animate.shift([2.0, 0.0, 0.0]),
    run_time=1.0 / 30.0,
    rate_func=manimlib.there_and_back_with_pause,
)
assert np.allclose(sampled_rate_mover.get_center(), [0.0, 0.0, 0.0])

abort_scene = Scene()
abort_mover = geometry.Rectangle(width=1.0, height=1.0)
abort_scene.add(abort_mover)


def explode_after_begin(mob, alpha):
    if alpha > 0.0:
        raise LookupError("callback release failure")
    mob.set_x(alpha)


try:
    abort_scene.play(
        update_animation.UpdateFromAlphaFunc(
            abort_mover,
            explode_after_begin,
            run_time=1.0 / 30.0,
            rate_func=manimlib.linear,
        )
    )
except LookupError as error:
    assert str(error) == "callback release failure"
else:
    raise AssertionError("a Python animation callback exception was swallowed")
abort_scene.play(
    abort_mover.animate.shift(manimlib.RIGHT),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(abort_mover.get_x(), 1.0)

# Reference Scene defaults random_seed to zero and resets both RNG modules
# used by source-unedited scene files.
Scene()
python_random = [manimlib.random.random() for _ in range(3)]
numpy_random = np.random.random(3)
Scene()
assert python_random == [manimlib.random.random() for _ in range(3)]
assert np.allclose(numpy_random, np.random.random(3))

# fm-5wq.4: random_seed rides kwargs over the Reference's class default 0.
# An explicit seed re-seeds both RNG modules at every
# construct; random_seed=None skips seeding entirely, preserving the live
# streams mid-sequence.
assert scene_module.Scene.random_seed == 0
assert Scene().random_seed == 0
seeded_scene = Scene(random_seed=123)
assert seeded_scene.random_seed == 123
seeded_python_draws = [manimlib.random.random() for _ in range(2)]
seeded_numpy_first = float(np.random.random())
unseeded_scene = Scene(random_seed=None)
assert unseeded_scene.random_seed is None
seeded_numpy_second = float(np.random.random())
reseeded_scene = Scene(random_seed=123)
assert reseeded_scene.random_seed == 123
assert seeded_python_draws == [manimlib.random.random() for _ in range(2)]
assert np.isclose(float(np.random.random()), seeded_numpy_first)
# The None construct did not reset numpy: the second draw continued the
# 123 stream exactly where the first left off.
assert np.isclose(float(np.random.random()), seeded_numpy_second)


class _SeededScene(Scene):
    random_seed = 7


assert _SeededScene().random_seed == 7
assert _SeededScene(random_seed=11).random_seed == 11
Scene()  # restore the default seed-0 streams for later pins

# Custom callbacks can use the RecordBuffer-backed point-cloud and point
# matching surface without falling back to schema placeholders.
dot_cloud = manimlib.GlowDot([1.0, 2.0, 0.0], opacity=0.75)
opacities = dot_cloud.get_opacities()
assert np.allclose(opacities, [0.75])
dot_cloud.add_point([3.0, 4.0, 0.0], opacity=0.25, color=manimlib.BLUE)
assert np.allclose(dot_cloud.get_points(), [[1, 2, 0], [3, 4, 0]])
assert np.allclose(dot_cloud.get_opacities(), [0.75, 0.25])
assert np.allclose(dot_cloud.data["radius"], [0.2, 0.2])
assert np.allclose(dot_cloud.data["glow_factor"], [2.0, 2.0])
dot_cloud.set_opacity([0.5, 0.125])
assert np.allclose(dot_cloud.get_opacities(), [0.5, 0.125])

match_source = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [2.0, 1.0, 0.0], [3.0, 0.0, 0.0]]
).shift([4.0, 0.0, 0.0])
match_target = VMobject().set_points_as_corners(
    [[-1.0, 0.0, 0.0], [0.0, 0.0, 0.0]]
)
match_target.match_points(match_source)
assert match_target.n_records() == match_source.n_records()
assert np.allclose(match_target.get_points(), match_source.get_points())

vector_field = importlib.import_module("manimlib.mobject.vector_field")
assert vector_field.VectorField.__bases__ == (VMobject,)
assert vector_field.TimeVaryingVectorField.__bases__ == (
    vector_field.VectorField,
)
assert list(inspect.signature(vector_field.VectorField).parameters) == [
    "func",
    "coordinate_system",
    "sample_coords",
    "density",
    "magnitude_range",
    "color",
    "color_map_name",
    "color_map",
    "stroke_opacity",
    "stroke_width",
    "tip_width_ratio",
    "tip_len_to_width",
    "max_vect_len",
    "max_vect_len_to_step_size",
    "flat_stroke",
    "norm_to_opacity_func",
    "kwargs",
]
assert inspect.signature(vector_field.VectorField).parameters[
    "density"
].default == 2.0
assert inspect.signature(vector_field.VectorField).parameters[
    "color_map_name"
].default == "3b1b_colormap"
assert list(
    inspect.signature(vector_field.TimeVaryingVectorField).parameters
) == ["time_func", "coordinate_system", "kwargs"]
assert str(inspect.signature(vector_field.VectorField.set_stroke)) == (
    "(self, color=None, width=None, opacity=None, behind=None, flat=None, "
    "recurse=True)"
)
assert str(inspect.signature(vector_field.VectorField.get_sample_points)) == (
    "(self, center, width, height, depth, x_density, y_density, z_density)"
)
assert str(inspect.signature(vector_field.get_sample_coords)) == (
    "(coordinate_system, density=1.0)"
)

field_axes = manimlib.Axes(
    x_range=(0.0, 2.0, 1.0),
    y_range=(-1.0, 1.0, 1.0),
    width=2.0,
    height=2.0,
)
sample_grid = vector_field.get_sample_coords(field_axes)
assert sample_grid.shape == (9, 2)
assert np.allclose(
    sample_grid,
    [
        [0.0, -1.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, -1.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [2.0, -1.0],
        [2.0, 0.0],
        [2.0, 1.0],
    ],
)
try:
    vector_field.get_sample_coords(field_axes, density=0.0)
except ValueError as error:
    assert str(error) == "VectorField density must be positive and finite"
else:
    raise AssertionError("VectorField accepted zero sampling density")
try:
    vector_field.get_sample_coords(field_axes, density=1000.0)
except ValueError as error:
    assert "65536-point resource budget" in str(error)
else:
    raise AssertionError("VectorField accepted an excessive sample grid")
try:
    vector_field.VectorField(
        np.zeros_like,
        field_axes,
        sample_coords=np.zeros((65_537, 2)),
        max_vect_len=0.5,
        color=manimlib.BLUE,
    )
except ValueError as error:
    assert "65536-point resource budget" in str(error)
else:
    raise AssertionError("VectorField accepted excessive explicit samples")

observed_field_shapes = []


def two_dimensional_field(coords):
    observed_field_shapes.append(coords.shape)
    return np.column_stack([np.ones(len(coords)), np.zeros(len(coords))])


static_field = vector_field.VectorField(
    two_dimensional_field,
    field_axes,
    sample_coords=np.array([[0.0, 0.0], [1.0, 0.0]]),
    max_vect_len=0.5,
    color=manimlib.BLUE,
)
assert observed_field_shapes == [(2, 2)]
assert static_field.sample_coords.shape == (2, 2)
assert static_field.get_num_points() == 15
assert static_field.get_joint_type() == 0
drawn_length = 0.5 * math.tanh(2.0)
assert math.isclose(
    np.linalg.norm(static_field.get_points()[6] - static_field.get_points()[0]),
    drawn_length,
    rel_tol=0.0,
    abs_tol=2e-7,
)
base_widths = np.array([1, 1, 1, 1, 4, 2, 0, 0, 1, 1, 1, 1, 4, 2, 0])
assert np.allclose(static_field.base_stroke_width_array, base_widths)
assert np.allclose(static_field.get_stroke_widths(), 3.0 * base_widths)
assert np.allclose(
    static_field.data["stroke_rgba"][:, :3],
    np.repeat([manimlib.color_to_rgb(manimlib.BLUE)], 15, axis=0),
)
assert static_field.set_stroke_width(2.0) is static_field
assert np.allclose(static_field.get_stroke_widths(), 2.0 * base_widths)
assert static_field.set_stroke(
    manimlib.RED,
    1.5,
    opacity=0.4,
    behind=True,
    flat=True,
) is static_field
assert np.allclose(static_field.get_stroke_widths(), 1.5 * base_widths)
assert np.allclose(static_field.get_stroke_opacities(), 0.4)
assert np.allclose(
    static_field.data["stroke_rgba"][:, :3], manimlib.color_to_rgb(manimlib.RED)
)
assert static_field.uniforms["stroke_behind"]
assert static_field.get_flat_stroke()

old_sample_points = static_field.sample_points.copy()
assert static_field.set_sample_coords([[0.5, 0.0], [1.5, 0.0]]) is static_field
assert np.array_equal(static_field.sample_points, old_sample_points)
assert static_field.update_sample_points() is None
assert not np.array_equal(static_field.sample_points, old_sample_points)
assert static_field.update_vectors() is static_field
assert np.allclose(
    static_field.data["stroke_rgba"][:, :3], manimlib.color_to_rgb(manimlib.RED)
)
assert np.allclose(static_field.get_stroke_opacities(), 0.4)

grid_points = static_field.get_sample_points(
    np.zeros(3), 2.0, 2.0, 0.0, 1.0, 1.0, 1.0
)
assert np.allclose(
    grid_points,
    [
        [-1.0, -1.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, -1.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
    ],
)
try:
    static_field.get_sample_points(
        np.zeros(3), 2.0, 2.0, 0.0, 1000.0, 1000.0, 1.0
    )
except ValueError as error:
    assert "65536-point resource budget" in str(error)
else:
    raise AssertionError("VectorField accepted an excessive point grid")
initialized_field = static_field.copy()
assert initialized_field.init_points() is None
assert np.array_equal(initialized_field.get_points(), np.zeros((15, 3)))
assert initialized_field.get_joint_type() == 0

custom_field = vector_field.VectorField(
    lambda coords: np.array([[0.0, 0.0], [2.0, 0.0]]),
    field_axes,
    sample_coords=[[0.0, 0.0], [1.0, 0.0]],
    magnitude_range=(0.0, 2.0),
    max_vect_len=0.5,
    color_map=lambda alphas: np.column_stack(
        [alphas, np.zeros(len(alphas)), 1.0 - alphas, np.ones(len(alphas))]
    ),
    norm_to_opacity_func=lambda norms: norms / 2.0,
)
assert np.allclose(custom_field.data["stroke_rgba"][:8, :3], [0.0, 0.0, 1.0])
assert np.allclose(custom_field.data["stroke_rgba"][8:, :3], [1.0, 0.0, 0.0])
assert np.allclose(custom_field.get_stroke_opacities()[:8], 0.0)
assert np.allclose(custom_field.get_stroke_opacities()[8:], 1.0)

native_mapped_field = vector_field.VectorField(
    lambda coords: np.array([[0.0, 0.0], [2.0, 0.0]]),
    field_axes,
    sample_coords=[[0.0, 0.0], [1.0, 0.0]],
    magnitude_range=(0.0, 2.0),
    max_vect_len=0.5,
    stroke_color=manimlib.GREEN,
)
assert np.allclose(
    native_mapped_field.data["stroke_rgba"][:8, :3],
    manimlib.color_to_rgb(manimlib.BLUE_E),
)
assert np.allclose(
    native_mapped_field.data["stroke_rgba"][8:, :3],
    manimlib.color_to_rgb(manimlib.RED),
)

three_dimensional_field = vector_field.VectorField(
    lambda coords: np.column_stack(
        [np.ones(len(coords)), np.zeros(len(coords)), coords[:, 2]]
    ),
    field_axes,
    sample_coords=[[0.0, 0.0, 0.25], [1.0, 0.0, 0.5]],
    max_vect_len=math.inf,
    color=manimlib.BLUE,
)
assert three_dimensional_field.sample_coords.shape == (2, 3)
assert np.allclose(
    np.linalg.norm(
        three_dimensional_field.get_points()[6]
        - three_dimensional_field.get_points()[0]
    ),
    1.0,
    atol=2e-7,
)
try:
    vector_field.VectorField(
        lambda coords: np.zeros((len(coords) + 1, 2)),
        field_axes,
        sample_coords=[[0.0, 0.0], [1.0, 0.0]],
    )
except ValueError as error:
    assert "one vector per sample" in str(error)
else:
    raise AssertionError("VectorField accepted a callback row-count mismatch")


def refuse_vector_field_callback(coords):
    del coords
    raise LookupError("vector field callback failed")


try:
    vector_field.VectorField(
        refuse_vector_field_callback,
        field_axes,
        sample_coords=[[0.0, 0.0], [1.0, 0.0]],
    )
except LookupError as error:
    assert str(error) == "vector field callback failed"
else:
    raise AssertionError("VectorField swallowed its Python callback error")
try:
    vector_field.VectorField(
        lambda coords: coords,
        field_axes,
        color_map_name="viridis",
    )
except NotImplementedError as error:
    assert "non-bundled matplotlib map" in str(error)
else:
    raise AssertionError("VectorField silently accepted an unavailable color map")
try:
    vector_field.VectorField(
        lambda coords: coords,
        field_axes,
        color_map=object(),
    )
except TypeError as error:
    assert str(error) == "VectorField color_map must be callable"
else:
    raise AssertionError("VectorField accepted a non-callable color map")

gradient = vector_field.get_vectorized_rgb_gradient_function(
    0.0, 3.0, "3b1b_colormap"
)
gradient_rows = gradient([0.0, 1.0, 2.0, 3.0])
assert np.allclose(gradient_rows[0], manimlib.color_to_rgb(manimlib.BLUE_E))
assert np.allclose(gradient_rows[1], manimlib.color_to_rgb(manimlib.GREEN))
assert np.allclose(gradient_rows[2], manimlib.color_to_rgb(manimlib.YELLOW))
assert np.allclose(gradient_rows[3], manimlib.color_to_rgb(manimlib.RED))
scalar_gradient = vector_field.get_rgb_gradient_function(
    0.0, 3.0, "3b1b_colormap"
)
assert np.allclose(scalar_gradient(1.0), manimlib.color_to_rgb(manimlib.GREEN))
try:
    vector_field.get_vectorized_rgb_gradient_function(0, 1, "viridis")
except NotImplementedError as error:
    assert "non-bundled matplotlib data" in str(error)
else:
    raise AssertionError("field gradient silently accepted unavailable map data")

pointwise_field = vector_field.vectorize(lambda x, y: (x + y, x - y))
assert np.allclose(pointwise_field([[1.0, 2.0], [3.0, 1.0]]), [[3, -1], [4, 2]])
moving = geometry.Square(side_length=0.2)
assert vector_field.move_along_vector_field(
    moving, lambda point: np.array([1.0, 0.0, 0.0])
) is moving
moving.update(0.5)
assert np.allclose(moving.get_center(), [0.5, 0.0, 0.0])
moving_children = manimlib.Group(
    geometry.Square(side_length=0.2), geometry.Square(side_length=0.2).shift(manimlib.UP)
)
assert vector_field.move_submobjects_along_vector_field(
    moving_children, lambda point: np.array([0.0, 1.0, 0.0])
) is moving_children
moving_children.update(0.25)
assert np.allclose(moving_children[0].get_center(), [0.0, 0.25, 0.0])
assert np.allclose(moving_children[1].get_center(), [0.0, 1.25, 0.0])
moving_points = geometry.Square(side_length=0.2)
assert vector_field.move_points_along_vector_field(
    moving_points,
    lambda x, y: (1.0, 0.0),
    field_axes,
) is moving_points
moving_points.update(0.5)
assert np.allclose(moving_points.get_center(), [0.5, 0.0, 0.0])

field = vector_field.TimeVaryingVectorField(
    lambda coords, time: np.column_stack(
        [np.full(len(coords), time), np.zeros(len(coords)), np.zeros(len(coords))]
    ),
    field_axes,
    sample_coords=np.array([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]),
    max_vect_len=0.5,
    color=manimlib.BLUE,
)
field_before = field.get_points().copy()
field_scene = Scene()
field_scene.add(field_axes, field)
field_scene.wait(1.0 / 30.0)
assert math.isclose(field.time, 1.0 / 30.0, rel_tol=0.0, abs_tol=1e-12)
assert not np.allclose(field.get_points(), field_before)
assert field.increment_time(0.0) is None


def refuse_time_field(coords, time):
    if time > 0:
        raise RuntimeError("time field callback failed")
    return np.zeros_like(coords)


refusing_time_field = vector_field.TimeVaryingVectorField(
    refuse_time_field,
    field_axes,
    sample_coords=[[0.0, 0.0], [1.0, 0.0]],
    max_vect_len=0.5,
    color=manimlib.BLUE,
)
try:
    refusing_time_field.update(1.0 / 30.0)
except RuntimeError as error:
    assert str(error) == "time field callback failed"
else:
    raise AssertionError("TimeVaryingVectorField swallowed its callback error")

# Fixed-frame state is the renderer-consumed typed uniform, including the
# Reference's recursive family default and non-recursive override.
fixed_child = geometry.Square()
fixed_group = manimlib.Group(fixed_child)
assert "__init__" in manimlib.Group.__dict__
iterable_group_children = [geometry.Square(), geometry.Circle()]
iterable_group = manimlib.Group(iterable_group_children)
assert list(iterable_group) == iterable_group_children
deduplicated_group = manimlib.Group(
    iterable_group_children[0], iterable_group_children[0]
)
assert list(deduplicated_group) == iterable_group_children[:1]

# Appendix-C C-14: the Reference's width branch calls set_height(width).
# A 3x2 grid of 2x1 rectangles is already 6x2 with zero buffer, so this
# planted negative becomes 18x6 under the buggy branch and 6x2 here.
grid_source = geometry.Rectangle(width=2.0, height=1.0)
width_grid = grid_source.get_grid(2, 3, width=6.0, buff=0.0)
assert len(width_grid) == 6
assert np.allclose(
    [width_grid.get_width(), width_grid.get_height()], [6.0, 2.0]
)
height_grid = grid_source.get_grid(2, 3, height=4.0, buff=0.0)
assert np.allclose(
    [height_grid.get_width(), height_grid.get_height()], [12.0, 4.0]
)
row_grid = grid_source.get_grid(2, 3, group_by_rows=True, buff=0.0)
assert len(row_grid) == 2
assert [len(row) for row in row_grid] == [3, 3]
column_grid = grid_source.get_grid(2, 3, group_by_cols=True, buff=0.0)
assert len(column_grid) == 3
assert [len(column) for column in column_grid] == [2, 2, 2]
assert fixed_group.is_fixed_in_frame() is False
assert fixed_child.is_fixed_in_frame() is False
assert fixed_group.fix_in_frame() is fixed_group
assert fixed_group.uniforms["is_fixed_in_frame"] == 1.0
assert fixed_child.uniforms["is_fixed_in_frame"] == 1.0
fixed_group.unfix_from_frame(recurse=False)
assert fixed_group.is_fixed_in_frame() is False
assert fixed_child.is_fixed_in_frame() is True
fixed_group.unfix_from_frame()
assert fixed_child.is_fixed_in_frame() is False

# Sampled surfaces select Choreo's UV-grid partial mechanism rather than the
# VMobject-only quadratic operator, and finish on the original full grid.
surface = manimlib.ParametricSurface(
    lambda u, v: np.array([u, v, u + v]),
    resolution=(3, 3),
)
surface_points = surface.get_points().copy()
surface_scene = Scene()
surface_scene.play(
    manimlib.ShowCreation(surface),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(surface.get_points(), surface_points)

# The public Surface method reaches that same Marionette authority directly.
# Partial reveals replace only the point grid, preserve the target's normals
# and style lanes, and never mutate the source. Full-range reveal restores both
# pointlike columns, matching Mobject.match_points in the pinned Reference.
assert str(inspect.signature(manimlib.Surface.pointwise_become_partial)) == (
    "(self, smobject, a, b, axis=None)"
)
surface_source = manimlib.ParametricSurface(
    lambda u, v: np.array([u, v, u + 2.0 * v]),
    u_range=(0.0, 2.0),
    v_range=(0.0, 2.0),
    resolution=(3, 3),
    color=manimlib.BLUE,
    opacity=0.8,
)
surface_source_before = surface_source.data.copy()
surface_target = manimlib.ParametricSurface(
    lambda u, v: np.array([u - 4.0, v - 4.0, 0.0]),
    resolution=(3, 3),
    color=manimlib.RED,
    opacity=0.35,
)
surface_target_normals = surface_target.data["d_normal_point"].copy()
surface_target_rgba = surface_target.data["rgba"].copy()
expected_v_slice = surface_source_before["point"].reshape((3, 3, 3)).copy()
expected_v_slice[:, 0] = expected_v_slice[:, 1]
expected_v_slice[:, 2] = expected_v_slice[:, 1]
assert (
    surface_target.pointwise_become_partial(surface_source, 0.5, 0.5, axis=1)
    is surface_target
)
assert np.allclose(surface_target.data["point"], expected_v_slice.reshape((-1, 3)))
assert np.array_equal(surface_target.data["d_normal_point"], surface_target_normals)
assert np.array_equal(surface_target.data["rgba"], surface_target_rgba)
assert np.array_equal(surface_source.data, surface_source_before)

default_axis_target = manimlib.ParametricSurface(
    lambda u, v: np.array([u + 4.0, v + 4.0, 0.0]),
    resolution=(3, 3),
    preferred_creation_axis=0,
)
expected_u_slice = surface_source_before["point"].reshape((3, 3, 3)).copy()
expected_u_slice[0] = expected_u_slice[1]
expected_u_slice[2] = expected_u_slice[1]
default_axis_target.pointwise_become_partial(surface_source, 0.5, 0.5)
assert np.allclose(
    default_axis_target.data["point"], expected_u_slice.reshape((-1, 3))
)

full_surface_target = manimlib.ParametricSurface(
    lambda u, v: np.array([-u, -v, 0.0]),
    resolution=(3, 3),
    color=manimlib.GREEN,
    opacity=0.2,
)
full_surface_rgba = full_surface_target.data["rgba"].copy()
full_surface_target.pointwise_become_partial(surface_source, 0.0, 1.0)
assert np.array_equal(
    full_surface_target.data["point"], surface_source_before["point"]
)
assert np.array_equal(
    full_surface_target.data["d_normal_point"],
    surface_source_before["d_normal_point"],
)
assert np.array_equal(full_surface_target.data["rgba"], full_surface_rgba)

self_surface = surface_source.copy()
self_expected = self_surface.data["point"].reshape((3, 3, 3)).copy()
self_expected[:, 0] = self_expected[:, 1]
self_expected[:, 2] = self_expected[:, 1]
assert self_surface.pointwise_become_partial(self_surface, 0.5, 0.5) is self_surface
assert np.allclose(self_surface.data["point"], self_expected.reshape((-1, 3)))

bound_surface_scene = Scene()
bound_surface_source = surface_source.copy()
bound_surface_target = full_surface_target.copy()
bound_surface_scene.add(bound_surface_source, bound_surface_target)
bound_surface_target.pointwise_become_partial(
    bound_surface_source, 0.5, 0.5, axis=0
)
assert np.allclose(
    bound_surface_target.data["point"], expected_u_slice.reshape((-1, 3))
)

foreign_surface_scene = Scene()
foreign_surface_target = full_surface_target.copy()
foreign_surface_scene.add(foreign_surface_target)
foreign_surface_target.pointwise_become_partial(
    bound_surface_source, 0.5, 0.5, axis=1
)
assert np.allclose(
    foreign_surface_target.data["point"], expected_v_slice.reshape((-1, 3))
)

for invalid_axis in (-1, 2):
    refused_surface = full_surface_target.copy()
    refused_before = refused_surface.data.copy()
    try:
        refused_surface.pointwise_become_partial(
            surface_source, 0.25, 0.75, invalid_axis
        )
    except (OverflowError, RuntimeError):
        pass
    else:
        raise AssertionError(f"Surface accepted invalid partial axis {invalid_axis}")
    assert np.array_equal(refused_surface.data, refused_before)

nonfinite_surface = full_surface_target.copy()
nonfinite_before = nonfinite_surface.data.copy()
try:
    nonfinite_surface.pointwise_become_partial(surface_source, float("nan"), 0.5)
except ValueError as error:
    assert "finite" in str(error)
else:
    raise AssertionError("Surface accepted a non-finite partial bound")
assert np.array_equal(nonfinite_surface.data, nonfinite_before)

malformed_surface_source = surface_source.copy()
malformed_surface_source.resolution = (2, 4)
malformed_surface_target = full_surface_target.copy()
malformed_target_before = malformed_surface_target.data.copy()
try:
    malformed_surface_target.pointwise_become_partial(
        malformed_surface_source, 0.25, 0.75
    )
except RuntimeError as error:
    assert "schema" in str(error).lower()
else:
    raise AssertionError("Surface accepted resolution metadata inconsistent with its records")
assert np.array_equal(malformed_surface_target.data, malformed_target_before)

try:
    surface_target.pointwise_become_partial(VMobject(), 0.0, 1.0)
except AssertionError:
    pass
else:
    raise AssertionError("Surface accepted a non-Surface partial source")

# Filled Arrow and Vector are authored over Atlas rather than inherited Line
# approximations.  Exercise every public row together so the tip-record
# identity, Reference ratio caps, and rebuild side effects cannot drift apart.
assert manimlib.Arrow is geometry.Arrow
assert manimlib.Vector is geometry.Vector
assert geometry.Arrow.__bases__ == (geometry.Line,)
assert geometry.Vector.__bases__ == (geometry.Arrow,)
assert geometry.Arrow.tickness_multiplier == 0.015
assert list(inspect.signature(geometry.Arrow).parameters) == [
    "start",
    "end",
    "buff",
    "path_arc",
    "fill_color",
    "fill_opacity",
    "stroke_width",
    "thickness",
    "tip_width_ratio",
    "tip_angle",
    "max_tip_length_to_length_ratio",
    "max_width_to_length_ratio",
    "kwargs",
]
assert list(inspect.signature(geometry.Vector).parameters) == [
    "direction",
    "buff",
    "kwargs",
]

capped_arrow = geometry.Arrow(
    [0.0, 0.0, 0.0],
    [4.0, 0.0, 0.0],
    buff=0.0,
    thickness=4.0,
    tip_width_ratio=3.0,
    tip_angle=math.pi / 2.0,
    max_tip_length_to_length_ratio=0.005,
    max_width_to_length_ratio=0.005,
    fill_color=manimlib.RED,
    fill_opacity=0.65,
    stroke_color=manimlib.BLUE,
    stroke_width=1.25,
)
assert capped_arrow.thickness == 4.0
assert capped_arrow.tip_width_ratio == 3.0
assert capped_arrow.tip_angle == math.pi / 2.0
assert capped_arrow.max_tip_length_to_length_ratio == 0.005
assert capped_arrow.max_width_to_length_ratio == 0.005
# width=min(4*0.015, 4*0.005)=0.02; the 0.005 head-length cap then
# scales the initial (tip width, tip length) pair from (0.06, 0.03).
assert np.allclose(capped_arrow.get_key_dimensions(4.0), [0.02, 0.04, 0.02])
assert np.allclose(capped_arrow.get_start(), [0.0, 0.0, 0.0], atol=1e-9)
assert np.allclose(capped_arrow.get_end(), [4.0, 0.0, 0.0], atol=1e-9)
start, end = capped_arrow.get_start_and_end()
assert np.allclose(start, capped_arrow.get_start())
assert np.allclose(end, capped_arrow.get_end())

capped_identity = id(capped_arrow)
capped_style = (
    capped_arrow.get_fill_color(),
    capped_arrow.get_fill_opacity(),
    capped_arrow.get_stroke_color(),
    capped_arrow.get_stroke_opacity(),
    capped_arrow.get_stroke_width(),
)
capped_arrow.uniforms["depth_test"] = True
capped_arrow.uniforms["anti_alias_width"] = 2.75
assert capped_arrow.set_points_by_ends(
    [-1.0, -1.0, 0.0],
    [2.0, 2.0, 0.0],
    buff=0.0,
    path_arc=math.pi / 3.0,
) is capped_arrow
assert id(capped_arrow) == capped_identity
assert np.allclose(capped_arrow.get_start(), [-1.0, -1.0, 0.0], atol=1e-8)
curved_end = capped_arrow.get_end()
assert np.allclose(curved_end, [2.0, 2.0, 0.0], atol=1e-4), curved_end
assert capped_arrow.uniforms["depth_test"] is True
assert capped_arrow.uniforms["anti_alias_width"] == 2.75
assert capped_style == (
    capped_arrow.get_fill_color(),
    capped_arrow.get_fill_opacity(),
    capped_arrow.get_stroke_color(),
    capped_arrow.get_stroke_opacity(),
    capped_arrow.get_stroke_width(),
)

transformed_arrow = geometry.Arrow(
    [0.0, 0.0, 0.0], [2.0, 0.0, 0.0], buff=0.0
)
transformed_identity = id(transformed_arrow)
assert transformed_arrow.rotate(
    math.pi / 2.0, about_point=manimlib.ORIGIN
) is transformed_arrow
assert np.allclose(transformed_arrow.get_start(), [0.0, 0.0, 0.0], atol=1e-8)
assert np.allclose(transformed_arrow.get_end(), [0.0, 2.0, 0.0], atol=1e-8)
assert transformed_arrow.scale(
    0.5, about_point=manimlib.ORIGIN
) is transformed_arrow
assert id(transformed_arrow) == transformed_identity
assert np.allclose(transformed_arrow.get_end(), [0.0, 1.0, 0.0], atol=1e-8)
assert transformed_arrow.set_thickness(6.0) is transformed_arrow
assert transformed_arrow.thickness == 6.0
assert np.allclose(transformed_arrow.get_end(), [0.0, 1.0, 0.0], atol=1e-8)
assert transformed_arrow.put_start_and_end_on(
    [1.0, 2.0, 3.0], [2.0, 4.0, 6.0]
) is transformed_arrow
assert np.allclose(transformed_arrow.get_start(), [1.0, 2.0, 3.0], atol=1e-8)
assert np.allclose(transformed_arrow.get_end(), [2.0, 4.0, 6.0], atol=1e-8)

vector_2d = geometry.Vector([3.0, 4.0])
assert np.allclose(vector_2d.get_start(), manimlib.ORIGIN, atol=1e-9)
assert np.allclose(vector_2d.get_end(), [3.0, 4.0, 0.0], atol=1e-9)
vector_3d = geometry.Vector([1.0, -2.0, 3.0])
assert np.allclose(vector_3d.get_start(), manimlib.ORIGIN, atol=1e-9)
assert np.allclose(vector_3d.get_end(), [1.0, -2.0, 3.0], atol=1e-8)

# Filled Arrow endpoint updates rebuild Atlas's length-dependent tip geometry
# in place, so scene-bound updater callbacks preserve arena identity.
bound_arrow = manimlib.Vector(manimlib.RIGHT)
arrow_scene = Scene()
arrow_scene.add(bound_arrow)
arrow_identity = id(bound_arrow)
bound_arrow.put_start_and_end_on([1.0, 2.0, 0.0], [4.0, 2.0, 0.0])
assert id(bound_arrow) == arrow_identity
bound_arrow_points = bound_arrow.get_points()
assert np.allclose(
    0.5 * (bound_arrow_points[0] + bound_arrow_points[-3]),
    [1.0, 2.0, 0.0],
)
assert np.min(np.linalg.norm(bound_arrow_points - [4.0, 2.0, 0.0], axis=1)) < 1e-6

# A detached fixed-frame Arrow is rebuilt by the immediate updater call that
# add_updater performs by default.  Rebuilding geometry must not replace the
# engine-owned camera/depth uniforms with fresh Arrow defaults; BeamSplitter's
# three overlay vectors all pass through this exact path before scene adoption.
detached_fixed_arrow = manimlib.Vector(manimlib.RIGHT).fix_in_frame()
detached_fixed_arrow.uniforms["depth_test"] = True
detached_fixed_arrow.uniforms["anti_alias_width"] = 2.5
detached_fixed_arrow.add_updater(
    lambda arrow: arrow.put_start_and_end_on(
        [-2.0, 3.0, 0.0],
        [-3.0, 3.0, 0.0],
    )
)
assert detached_fixed_arrow.is_fixed_in_frame() is True
assert detached_fixed_arrow.uniforms["depth_test"] is True
assert detached_fixed_arrow.uniforms["anti_alias_width"] == 2.5
assert np.allclose(detached_fixed_arrow.start, [-2.0, 3.0, 0.0])
assert np.allclose(detached_fixed_arrow.end, [-3.0, 3.0, 0.0])
arrow_scene.add(detached_fixed_arrow)
assert detached_fixed_arrow.is_fixed_in_frame() is True

# Generic endpoint placement applies one affine transform to the complete
# family in both proxy states. Detached families distribute the Reference
# algorithm over their private nursery roots; bound families recurse through
# the scene Stage.
detached_endpoint_parent = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]
)
detached_endpoint_child = geometry.Square(side_length=0.5).shift([1.0, 1.0, 0.0])
detached_endpoint_parent.add(detached_endpoint_child)
assert detached_endpoint_parent.put_start_and_end_on(
    [0.0, 0.0, 0.0], [0.0, 4.0, 0.0]
) is detached_endpoint_parent
assert np.allclose(detached_endpoint_parent.get_start(), [0.0, 0.0, 0.0])
assert np.allclose(detached_endpoint_parent.get_end(), [0.0, 4.0, 0.0])
assert np.allclose(detached_endpoint_child.get_center(), [-2.0, 2.0, 0.0])

bound_endpoint_parent = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]
)
bound_endpoint_child = geometry.Square(side_length=0.5).shift([1.0, 1.0, 0.0])
bound_endpoint_parent.add(bound_endpoint_child)
endpoint_scene = Scene()
endpoint_scene.add(bound_endpoint_parent)
bound_endpoint_parent.put_start_and_end_on([0.0, 0.0, 0.0], [0.0, 4.0, 0.0])
assert np.allclose(bound_endpoint_parent.get_start(), [0.0, 0.0, 0.0])
assert np.allclose(bound_endpoint_parent.get_end(), [0.0, 4.0, 0.0])
assert np.allclose(bound_endpoint_child.get_center(), [-2.0, 2.0, 0.0])

# Closed-loop refusal happens before any member moves.
closed_endpoint_parent = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]]
)
closed_endpoint_child = geometry.Square(side_length=0.5).shift([2.0, 1.0, 0.0])
closed_endpoint_parent.add(closed_endpoint_child)
closed_parent_before = closed_endpoint_parent.get_points().copy()
closed_child_before = closed_endpoint_child.get_points().copy()
try:
    closed_endpoint_parent.put_start_and_end_on(
        [0.0, 0.0, 0.0], [0.0, 4.0, 0.0]
    )
except Exception as error:
    assert str(error) == "Cannot position endpoints of closed loop"
else:
    raise AssertionError("closed-loop endpoint placement did not refuse")
assert np.array_equal(closed_endpoint_parent.get_points(), closed_parent_before)
assert np.array_equal(closed_endpoint_child.get_points(), closed_child_before)

# A slice aligner made from scene-bound children adopts into that scene and
# remains usable by the ordinary Reference next_to algorithm.
bound_slice_source = manimlib.VGroup(
    geometry.Square(side_length=1.0).shift([-1.0, 0.0, 0.0]),
    geometry.Square(side_length=1.0).shift([1.0, 0.0, 0.0]),
    geometry.Square(side_length=1.0).shift([3.0, 0.0, 0.0]),
)
bound_slice_target = manimlib.VGroup(
    geometry.Square(side_length=1.0).shift([4.0, 0.0, 0.0]),
    geometry.Square(side_length=1.0).shift([6.0, 0.0, 0.0]),
    geometry.Square(side_length=1.0).shift([8.0, 0.0, 0.0]),
)
slice_scene = Scene()
slice_scene.add(bound_slice_source, bound_slice_target)
bound_slice_source.next_to(
    bound_slice_target,
    manimlib.RIGHT,
    buff=0.25,
    index_of_submobject_to_align=slice(0, 2),
)
assert np.allclose(bound_slice_source[0].get_center(), [7.25, 0.0, 0.0])
assert np.allclose(bound_slice_source[1].get_center(), [9.25, 0.0, 0.0])

# Line.set_angle keeps its start fixed by default. DashedLine reads endpoints
# from its first/last native dash children, matching the Reference override.
assert geometry.DashedLine.__mro__[:3] == (
    geometry.DashedLine,
    geometry.Line,
    geometry.TipableVMobject,
)
assert list(inspect.signature(geometry.DashedLine).parameters) == [
    "start",
    "end",
    "dash_length",
    "positive_space_ratio",
    "kwargs",
]
dashed = geometry.DashedLine([-2.0, 1.0, 0.0], [2.0, 1.0, 0.0])
dashed_start = dashed.get_start().copy()
assert dashed.set_angle(math.pi / 2.0) is dashed
assert np.allclose(dashed.get_start(), dashed_start)
assert np.allclose(dashed.get_unit_vector(), [0.0, 1.0, 0.0], atol=1e-9)
dash_start, dash_end = dashed.get_start_and_end()
assert np.allclose(dash_start, dashed.get_start())
assert np.allclose(dash_end, dashed.get_end())
assert np.allclose(
    dashed.get_first_handle(), dashed.submobjects[0].get_points()[1]
)
assert np.allclose(
    dashed.get_last_handle(), dashed.submobjects[-1].get_points()[-2]
)
assert dashed.calculate_num_dashes(0.5, 0.5) == 4
for dash_args, refusal in (
    ((0.0, 0.5), "dash length must be positive and finite"),
    ((0.5, 0.0), "positive-space ratio must be finite and in (0, 1]"),
    ((1e-6, 0.5), "above the 4096 cap"),
):
    try:
        dashed.calculate_num_dashes(*dash_args)
    except ValueError as error:
        assert refusal in str(error)
    else:
        raise AssertionError("invalid native dash count parameters succeeded")

# TangentLine is the Atlas true-arclength construction, not a generated Line
# shell. On this 1+3 unit corner path, alpha=0.5 is one unit up the long
# vertical segment; curve-index interpolation would incorrectly stop at the
# corner. The requested length remains exact in detached and bound states.
assert geometry.TangentLine.__mro__[:3] == (
    geometry.TangentLine,
    geometry.Line,
    geometry.TipableVMobject,
)
assert list(inspect.signature(geometry.TangentLine).parameters) == [
    "vmob",
    "alpha",
    "length",
    "d_alpha",
    "kwargs",
]
tangent_source = VMobject().set_points_as_corners(
    ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 3.0, 0.0])
)
tangent = geometry.TangentLine(
    tangent_source,
    0.5,
    length=2.0,
    d_alpha=1e-4,
    stroke_color=manimlib.RED,
    stroke_width=3.0,
)
assert np.allclose(tangent.get_center(), [1.0, 1.0, 0.0], atol=1e-6)
assert np.allclose(tangent.get_unit_vector(), [0.0, 1.0, 0.0], atol=1e-6)
assert math.isclose(tangent.get_length(), 2.0, abs_tol=1e-6)
assert tangent.get_stroke_color() == manimlib.RED
assert tangent.get_stroke_width() == 3.0

bound_tangent_source = tangent_source.copy()
bound_tangent_scene = Scene()
bound_tangent_scene.add(bound_tangent_source)
bound_tangent_source.shift([3.0, -2.0, 0.0])
bound_tangent = geometry.TangentLine(
    bound_tangent_source, 0.5, length=3.0, d_alpha=1e-4
)
bound_tangent_identity = id(bound_tangent)
bound_tangent_scene.add(bound_tangent)
assert id(bound_tangent) == bound_tangent_identity
assert np.allclose(bound_tangent.get_center(), [4.0, -1.0, 0.0], atol=1e-6)
assert math.isclose(bound_tangent.get_length(), 3.0, abs_tol=1e-6)

empty_tangent = geometry.TangentLine(VMobject(), 0.5)
assert empty_tangent.get_num_points() == 0

# StrokeArrow is the authored Line subclass and Atlas now emits the pinned
# one-path, three-record stroke taper directly. There is no synthetic tip
# child for Python family traversal to observe.
assert geometry.StrokeArrow.__mro__[:4] == (
    geometry.StrokeArrow,
    geometry.Line,
    geometry.TipableVMobject,
    VMobject,
)
assert list(inspect.signature(geometry.StrokeArrow).parameters) == [
    "start",
    "end",
    "stroke_color",
    "stroke_width",
    "buff",
    "tip_width_ratio",
    "tip_len_to_width",
    "max_tip_length_to_length_ratio",
    "max_width_to_length_ratio",
    "kwargs",
]
stroke_arrow = geometry.StrokeArrow(
    [0.0, 0.0, 0.0], [4.0, 0.0, 0.0], stroke_color=manimlib.RED
)
assert stroke_arrow.submobjects == []
assert stroke_arrow.get_num_points() == 7
assert np.allclose(stroke_arrow.get_start(), [0.25, 0.0, 0.0])
assert np.allclose(stroke_arrow.get_end(), [3.75, 0.0, 0.0])
assert np.allclose(
    stroke_arrow.get_stroke_widths(), [5.0, 5.0, 5.0, 5.0, 25.0, 12.5, 0.0]
)
assert stroke_arrow.get_stroke_color() == manimlib.RED

ratio_arrow = geometry.StrokeArrow(
    [0.0, 0.0, 0.0],
    [2.0, 0.0, 0.0],
    stroke_width=2.0,
    buff=0.0,
    tip_width_ratio=3.0,
    tip_len_to_width=0.1,
    max_tip_length_to_length_ratio=0.25,
    max_width_to_length_ratio=0.4,
)
assert np.allclose(ratio_arrow.get_stroke_widths()[-3:], [2.4, 1.2, 0.0])
assert np.allclose(ratio_arrow.get_points()[-1], [2.0, 0.0, 0.0])
ratio_points = ratio_arrow.get_points().copy()
assert ratio_arrow.insert_tip_anchor() is ratio_arrow
assert ratio_arrow.submobjects == []
assert ratio_arrow.get_num_points() == 7
assert np.allclose(ratio_arrow.get_points(), ratio_points, atol=1e-9)

# The width-profile method is a real live RecordBuffer update, while reset,
# endpoint changes, and scale route geometry back through Atlas.
ratio_arrow.get_stroke_widths()[:] = 7.0
assert ratio_arrow.create_tip_with_stroke_width() is ratio_arrow
assert np.allclose(ratio_arrow.get_stroke_widths()[:4], 7.0)
assert np.allclose(ratio_arrow.get_stroke_widths()[-3:], [2.4, 1.2, 0.0])
ratio_identity = id(ratio_arrow)
assert ratio_arrow.set_stroke(color=manimlib.BLUE, width=1.5) is ratio_arrow
assert id(ratio_arrow) == ratio_identity
assert ratio_arrow.get_stroke_color() == manimlib.BLUE
assert np.allclose(ratio_arrow.get_stroke_widths()[-3:], [2.4, 1.2, 0.0])
assert ratio_arrow.set_points_by_ends(
    [-3.0, 1.0, 0.0], [3.0, 1.0, 0.0], buff=0.5, path_arc=0.0
) is ratio_arrow
assert np.allclose(ratio_arrow.get_start(), [-2.5, 1.0, 0.0])
assert np.allclose(ratio_arrow.get_end(), [2.5, 1.0, 0.0])
length_before_scale = ratio_arrow.get_length()
assert ratio_arrow.scale(2.0) is ratio_arrow
assert math.isclose(ratio_arrow.get_length(), 2.0 * length_before_scale, abs_tol=1e-6)
assert ratio_arrow.get_num_points() == 7
assert ratio_arrow.reset_tip() is ratio_arrow

bound_stroke_scene = Scene()
bound_stroke_scene.add(ratio_arrow)
assert ratio_arrow.set_stroke(width=2.0) is ratio_arrow
assert id(ratio_arrow) == ratio_identity
assert np.allclose(ratio_arrow.get_stroke_widths()[-3:], [6.0, 3.0, 0.0])
assert ratio_arrow.set_points_by_ends(
    [-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], path_arc=math.pi / 3.0
) is ratio_arrow
assert id(ratio_arrow) == ratio_identity
assert ratio_arrow.get_num_points() > 7
assert ratio_arrow.submobjects == []

degenerate_stroke = geometry.StrokeArrow([1.0, 2.0, 0.0], [1.0, 2.0, 0.0])
assert degenerate_stroke.submobjects == []
assert np.isfinite(degenerate_stroke.get_points()).all()
assert np.isfinite(degenerate_stroke.get_stroke_widths()).all()
assert np.allclose(degenerate_stroke.get_stroke_widths()[-3:], 0.0)
try:
    geometry.StrokeArrow([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], path_arc=math.nan)
except ValueError as error:
    assert "arc angle must be finite" in str(error)
else:
    raise AssertionError("StrokeArrow accepted a non-finite path_arc")

# `become` must keep the Python proxy family and Marionette family aligned
# when a live redraw changes a generated child's multiplicity.  This is the
# exact varying-dash-count shape exercised by MaxProcess's always_redraw line.
default_dashes = geometry.DashedLine()
assert default_dashes.submobjects

unbuffered_dashes = geometry.DashedLine(
    [0.0, 0.0, 0.0],
    [4.0, 0.0, 0.0],
    dash_length=0.12,
)
buffered_dashes = geometry.DashedLine(
    [0.0, 0.0, 0.0],
    [4.0, 0.0, 0.0],
    buff=0.5,
    dash_length=0.12,
)
assert math.isclose(buffered_dashes.buff, 0.5)
assert np.allclose(buffered_dashes.get_start(), [0.5, 0.0, 0.0])
assert buffered_dashes.get_length() < unbuffered_dashes.get_length()
assert buffered_dashes.get_end()[0] < unbuffered_dashes.get_end()[0]

failed_dashes = geometry.DashedLine.__new__(geometry.DashedLine)
try:
    geometry.DashedLine.__init__(failed_dashes, leftover_dash_option=True)
except NotImplementedError as error:
    assert str(error) == (
        "DashedLine() keyword(s) not yet routed to the native builder: "
        "leftover_dash_option"
    )
else:
    raise AssertionError("an unrouted DashedLine keyword reached the native builder")
assert not hasattr(failed_dashes, "submobjects")

many_dashes = geometry.DashedLine(
    [0.0, 0.0, 0.0], [4.0, 0.0, 0.0], dash_length=0.12
)
few_dashes = geometry.DashedLine(
    [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], dash_length=0.12
)
assert len(many_dashes.submobjects) > len(few_dashes.submobjects)
dash_scene = Scene()
dash_scene.add(many_dashes)
many_dashes_identity = id(many_dashes)
assert many_dashes.become(few_dashes) is many_dashes
assert id(many_dashes) == many_dashes_identity
assert len(many_dashes.submobjects) == len(few_dashes.submobjects)
assert len(many_dashes.get_family()) == 1 + len(many_dashes.submobjects)
assert np.allclose(
    many_dashes.submobjects[0].get_points(),
    few_dashes.submobjects[0].get_points(),
)

# A detached group can still own members a Scene holds: playing a slice
# (group[:2]) binds only those members. become() works on it as in the
# Reference (added_dimension.py's radial_lines updater): the bound members
# follow the source, and the source joins that Scene for them.
partly_bound_scene = Scene()
partly_bound = manimlib.VGroup(
    *(geometry.Line([0.0, 0.0, 0.0], [float(i), 0.0, 0.0]) for i in range(1, 4))
)
partly_bound_scene.add(partly_bound[:2])
assert not partly_bound._is_bound() and partly_bound[0]._is_bound()
partly_bound_target = manimlib.VGroup(
    *(geometry.Line([0.0, 0.0, 0.0], [0.0, float(i), 0.0]) for i in range(1, 5))
)
assert partly_bound.become(partly_bound_target) is partly_bound
assert len(partly_bound) == 4
assert np.allclose(partly_bound.get_all_points(), partly_bound_target.get_all_points())

# The same with a member bound directly, and sources whose members have their
# own families: a bound Line becoming a DashedLine (some1, subsets and
# inscribed_rect rebuild such groups in updaters). The detached root takes
# only the adopted source root's record; its members pair in Python.
for partly_source in (
    lambda: manimlib.VGroup(geometry.Line([-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), geometry.Dot()),
    lambda: manimlib.VGroup(
        geometry.DashedLine([-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), geometry.Dot([0.0, 1.0, 0.0])
    ),
    lambda: manimlib.VGroup(manimlib.VGroup(geometry.Dot(), geometry.Dot()), geometry.Dot()),
):
    member_bound_scene = Scene()
    member_bound = manimlib.VGroup(geometry.Line([-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]), geometry.Dot())
    member_bound_scene.add(member_bound[0])
    bound_member = member_bound[0]
    member_source = partly_source()
    member_source.set_fill(opacity=0.5)
    assert member_bound.become(member_source) is member_bound
    assert not member_bound._is_bound() and member_bound[0] is bound_member
    assert bound_member._is_bound() and bound_member in member_bound_scene.mobjects
    assert [len(part.submobjects) for part in member_bound.get_family()] == [
        len(part.submobjects) for part in member_source.get_family()
    ]
    for received, sourced in zip(member_bound.get_family(), member_source.get_family()):
        np.testing.assert_allclose(received.get_points(), sourced.get_points())
    assert np.isclose(member_bound.get_fill_opacity(), 0.5)

named_source = manimlib.VGroup(geometry.Dot(), geometry.Dot())
named_source.focus = named_source.submobjects[1]
named_receiver = manimlib.VGroup(geometry.Dot())
named_receiver.become(named_source)
assert named_receiver.focus is named_receiver.submobjects[1]

foreign_dash_scene = Scene()
foreign_dashes = geometry.DashedLine(
    [0.0, 0.0, 0.0], [2.0, 0.0, 0.0], dash_length=0.12
)
foreign_dash_scene.add(foreign_dashes)
many_family_before = list(many_dashes.submobjects)
foreign_family_before = list(foreign_dashes.submobjects)
try:
    many_dashes.become(foreign_dashes)
except bridge_errors.ForeignStageError as error:
    assert "become endpoints must belong to one Scene" in str(error)
else:
    raise AssertionError("become aligned foreign-stage families before refusal")
assert list(many_dashes.submobjects) == many_family_before
assert list(foreign_dashes.submobjects) == foreign_family_before

alignment_leaf = geometry.Dot(fill_opacity=1.0)
assert alignment_leaf.add_n_more_submobjects(2) is alignment_leaf
assert len(alignment_leaf.submobjects) == 2
assert all(child.get_num_points() == 1 for child in alignment_leaf.submobjects)
visible_child = geometry.Dot(fill_opacity=1.0)
alignment_group = manimlib.VGroup(visible_child)
alignment_group.add_n_more_submobjects(2)
assert alignment_group.submobjects[0] is visible_child
assert len(alignment_group.submobjects) == 3
assert all(
    math.isclose(child.get_fill_opacity(), 0.0)
    for child in alignment_group.submobjects[1:]
)
assert math.isclose(visible_child.invisible_copy().get_fill_opacity(), 0.0)

alignment_before = list(alignment_group.submobjects)
try:
    alignment_group.add_n_more_submobjects(1 << 16)
except RuntimeError as error:
    assert "family alignment requested 65539 direct submobjects" in str(error)
    assert "maximum is 65536" in str(error)
else:
    raise AssertionError("an over-budget family alignment reached allocation")
assert list(alignment_group.submobjects) == alignment_before

try:
    alignment_group.align_family(object())
except TypeError as error:
    assert "align_family expects a Mobject" in str(error)
else:
    raise AssertionError("align_family accepted a non-Mobject")
assert list(alignment_group.submobjects) == alignment_before

# The polygon lineage is native Atlas geometry, not schema-generated empty
# shells.  Constructor topology, rounded-corner mutation, and ArrowTip's live
# queries all remain valid after scene adoption and ordinary transforms.
assert manimlib.Polygon is geometry.Polygon
assert manimlib.RegularPolygon is geometry.RegularPolygon
assert manimlib.Triangle is geometry.Triangle
assert manimlib.ArrowTip is geometry.ArrowTip
assert issubclass(geometry.RegularPolygon, geometry.Polygon)
assert issubclass(geometry.Triangle, geometry.RegularPolygon)
assert issubclass(geometry.ArrowTip, geometry.Triangle)
assert str(inspect.signature(geometry.Polygon)) == "(*vertices, **kwargs)"
assert str(inspect.signature(geometry.RegularPolygon)) == (
    "(n=6, radius=1.0, start_angle=None, **kwargs)"
)
assert str(inspect.signature(geometry.Triangle)) == "(**kwargs)"
assert list(inspect.signature(geometry.ArrowTip).parameters) == [
    "angle",
    "width",
    "length",
    "fill_opacity",
    "fill_color",
    "stroke_width",
    "tip_style",
    "kwargs",
]

boolean_ops = importlib.import_module("manimlib.mobject.boolean_ops")
assert boolean_ops.Union.__bases__ == (VMobject,)
assert boolean_ops.Difference.__bases__ == (VMobject,)
assert boolean_ops.Intersection.__bases__ == (VMobject,)
assert boolean_ops.Exclusion.__bases__ == (VMobject,)
assert str(inspect.signature(boolean_ops.Union)) == "(*vmobjects, **kwargs)"
assert str(inspect.signature(boolean_ops.Difference)) == (
    "(subject, clip, **kwargs)"
)
union_left = manimlib.Square().shift(manimlib.LEFT * 0.5)
union_right = manimlib.Square().shift(manimlib.RIGHT * 0.5)
native_union = boolean_ops.Union(union_left, union_right)
assert native_union.get_num_points() > 0
assert native_union.get_width() > union_left.get_width()
native_difference = boolean_ops.Difference(union_left, union_right)
assert native_difference.get_num_points() > 0
native_intersection = boolean_ops.Intersection(union_left, union_right)
assert native_intersection.get_num_points() > 0
native_exclusion = boolean_ops.Exclusion(union_left, union_right)
assert native_exclusion.get_num_points() > 0
boolean_scene = Scene().add(native_union)
assert native_union._is_bound()
try:
    boolean_ops.Union(manimlib.Square())
except ValueError as error:
    assert str(error) == "At least 2 mobjects needed for Union"
else:
    raise AssertionError("Union accepted a single operand")
try:
    boolean_ops.Union(manimlib.Square(), manimlib.DotCloud())
except TypeError as error:
    assert str(error) == "Union operands must be VMobjects"
else:
    raise AssertionError("Union accepted a non-VMobject operand")

polygon = geometry.Polygon(
    [0.0, 0.0, 0.0],
    [2.0, 0.0, 0.0],
    [0.0, 2.0, 0.0],
    color=manimlib.BLUE,
)
assert polygon.get_num_curves() == 3
assert np.allclose(
    polygon.get_vertices(),
    [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
)
assert polygon.get_fill_color() == manimlib.BLUE
assert polygon.get_stroke_color() == manimlib.BLUE

rounded = polygon.copy()
rounded_style = (
    rounded.get_fill_color(),
    rounded.get_fill_opacity(),
    rounded.get_stroke_color(),
    rounded.get_stroke_opacity(),
    rounded.get_stroke_width(),
)
rounded_before = rounded.get_points().copy()
assert rounded.round_corners(0.2) is rounded
assert rounded.get_num_curves() == 9
assert not np.array_equal(rounded.get_points(), rounded_before)
assert np.isfinite(rounded.get_points()).all()
assert (
    rounded.get_fill_color(),
    rounded.get_fill_opacity(),
    rounded.get_stroke_color(),
    rounded.get_stroke_opacity(),
    rounded.get_stroke_width(),
) == rounded_style

default_rounded = geometry.Polygon(
    [-1.0, -1.0, 0.0],
    [1.0, -1.0, 0.0],
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
)
default_rounded.round_corners()
assert default_rounded.get_num_curves() == 12

concave_rounded = geometry.Polygon(
    [-1.0, -1.0, 0.0],
    [1.0, -1.0, 0.0],
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
).round_corners(-0.1)
assert np.isfinite(concave_rounded.get_points()).all()

bound_polygon = geometry.Polygon(
    [0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 2.0, 0.0]
)
polygon_scene = Scene()
polygon_scene.add(bound_polygon)
bound_polygon_identity = id(bound_polygon)
bound_polygon_points = bound_polygon.get_points().copy()
bound_polygon.round_corners(0.15)
assert id(bound_polygon) == bound_polygon_identity
assert not np.array_equal(bound_polygon.get_points(), bound_polygon_points)
assert bound_polygon.get_num_curves() == 9

hexagon = geometry.RegularPolygon(n=6, radius=2.0)
assert len(hexagon.get_vertices()) == 6
assert np.allclose(np.linalg.norm(hexagon.get_vertices(), axis=1), 2.0)
assert np.allclose(hexagon.get_vertices()[0], [2.0, 0.0, 0.0])
triangle = geometry.Triangle()
assert len(triangle.get_vertices()) == 3
assert np.allclose(triangle.get_vertices()[0], [0.0, 1.0, 0.0], atol=1e-12)

# fm-5wq.4.137: Triangle locks n=3 while routing the remaining
# RegularPolygon geometry and VMobject style kwargs through the native path.
styled_triangle = geometry.Triangle(
    radius=1.75,
    start_angle=0.0,
    fill_color=manimlib.RED,
    fill_opacity=0.3,
    stroke_color=manimlib.BLUE,
    stroke_width=2.25,
)
assert len(styled_triangle.get_vertices()) == 3
assert np.allclose(
    np.linalg.norm(styled_triangle.get_vertices(), axis=1), 1.75
)
assert np.allclose(styled_triangle.get_vertices()[0], [1.75, 0.0, 0.0])
assert styled_triangle.get_fill_color() == manimlib.RED
assert np.isclose(styled_triangle.get_fill_opacity(), 0.3)
assert styled_triangle.get_stroke_color() == manimlib.BLUE
assert np.isclose(styled_triangle.get_stroke_width(), 2.25)

try:
    geometry.Triangle(n=4)
except TypeError as error:
    assert "n" in str(error), error
else:
    raise AssertionError("Triangle allowed its fixed vertex count to be replaced")

try:
    geometry.Triangle(wobble_amount=1)
except TypeError as error:
    assert "wobble_amount" in str(error), error
else:
    raise AssertionError("Triangle silently ignored an unknown keyword")

tip = geometry.ArrowTip(
    angle=math.pi / 3.0,
    width=0.4,
    length=0.7,
    fill_color=manimlib.RED,
)
assert np.allclose(tip.get_tip_point(), tip.get_points()[0])
assert np.allclose(tip.get_vector(), tip.get_tip_point() - tip.get_base())
assert math.isclose(tip.get_angle(), math.pi / 3.0, abs_tol=1e-6)
assert math.isclose(tip.get_length(), 0.7, abs_tol=1e-6)
assert tip.get_fill_color() == manimlib.RED
for tip_style in (0, 1, 2, 99):
    styled_tip = geometry.ArrowTip(tip_style=tip_style)
    assert styled_tip.has_points()
    assert np.isfinite(styled_tip.get_points()).all()
assert np.allclose(
    geometry.ArrowTip(tip_style=1.5).get_points(),
    geometry.ArrowTip(tip_style=0).get_points(),
)
inner_smooth_tip = geometry.ArrowTip(tip_style=1)
assert np.allclose(
    inner_smooth_tip.get_base(), inner_smooth_tip.point_from_proportion(0.5)
)
assert not np.allclose(
    inner_smooth_tip.get_base(),
    inner_smooth_tip.quick_point_from_proportion(0.5),
    atol=1e-3,
)
assert np.allclose(
    inner_smooth_tip.get_vector(),
    inner_smooth_tip.get_tip_point() - inner_smooth_tip.get_base(),
)
assert math.isclose(
    inner_smooth_tip.get_length(),
    np.linalg.norm(inner_smooth_tip.get_vector()),
)

# TipableVMobject is an authored portal base, not a generated import shell.
# The Python layer owns the Reference lifecycle and object identity; Atlas
# owns the terminal tangent, placement, and BN-03 true-length shaft trim.
assert manimlib.TipableVMobject is geometry.TipableVMobject
assert geometry.TipableVMobject.__bases__ == (VMobject,)
assert geometry.Arc.__bases__ == (geometry.TipableVMobject,)
assert geometry.Line.__bases__ == (geometry.TipableVMobject,)
assert geometry.TipableVMobject.tip_config == dict(
    fill_opacity=1.0,
    stroke_width=0.0,
    tip_style=0.0,
)

untipped = geometry.Line([-2.0, -2.0, 0.0], [2.0, -2.0, 0.0])
assert not untipped.has_tip()
assert not untipped.has_start_tip()
try:
    untipped.get_tip()
except Exception as error:
    assert str(error) == "tip not found"
else:
    raise AssertionError("get_tip accepted a tipless TipableVMobject")
try:
    untipped.get_default_tip_length()
except AttributeError:
    pass
else:
    raise AssertionError("get_default_tip_length hid a missing public attribute")
untipped.tip_length = 0.625
assert untipped.get_default_tip_length() == 0.625
assert np.array_equal(untipped.get_first_handle(), untipped.get_points()[1])
assert np.array_equal(untipped.get_last_handle(), untipped.get_points()[-2])

manual_line = geometry.Line([-1.0, -3.0, 0.0], [1.0, -3.0, 0.0])
manual_logical_end = manual_line.get_end().copy()
manual_tip = manual_line.get_unpositioned_tip(length=0.2, width=0.3)
manual_tip_identity = id(manual_tip)
assert isinstance(manual_tip, geometry.ArrowTip)
assert manual_tip not in manual_line
assert manual_line.position_tip(manual_tip) is manual_tip
assert id(manual_tip) == manual_tip_identity
assert np.allclose(manual_tip.get_tip_point(), manual_logical_end, atol=1e-9)
assert manual_line.reset_endpoints_based_on_tip(manual_tip, False) is manual_line
assert VMobject.get_end(manual_line)[0] < manual_logical_end[0]
assert manual_line.asign_tip_attr(manual_tip, False) is manual_line
assert not manual_line.has_tip()
manual_line.add(manual_tip)
assert manual_line.has_tip()

tipped_line = geometry.Line(
    [-2.0, -1.0, 0.0],
    [2.0, -1.0, 0.0],
    stroke_color=manimlib.BLUE,
)
tipped_identity = id(tipped_line)
logical_start = tipped_line.get_start().copy()
logical_end = tipped_line.get_end().copy()
assert tipped_line.add_tip(length=0.5, width=0.4) is tipped_line
assert id(tipped_line) == tipped_identity
end_tip = tipped_line.tip
assert isinstance(end_tip, geometry.ArrowTip)
assert tipped_line.has_tip()
assert not tipped_line.has_start_tip()
assert tipped_line.submobjects == [end_tip]
assert np.allclose(end_tip.get_tip_point(), logical_end, atol=1e-9)
assert np.allclose(tipped_line.get_end(), logical_end, atol=1e-9)
assert VMobject.get_end(tipped_line)[0] < logical_end[0]
assert end_tip.get_fill_color() == manimlib.BLUE
assert math.isclose(tipped_line.get_length(), 4.0, abs_tol=1e-9)

assert tipped_line.add_tip(at_start=True, length=0.25) is tipped_line
start_tip = tipped_line.start_tip
assert isinstance(start_tip, geometry.ArrowTip)
assert tipped_line.has_start_tip()
assert tipped_line.submobjects == [end_tip, start_tip]
assert np.allclose(start_tip.get_tip_point(), logical_start, atol=1e-9)
assert np.allclose(tipped_line.get_start(), logical_start, atol=1e-9)
assert VMobject.get_start(tipped_line)[0] > logical_start[0]
assert tipped_line.get_tip() is end_tip
assert tipped_line.get_tips().submobjects == [end_tip, start_tip]

popped_tips = tipped_line.pop_tips()
assert isinstance(popped_tips, manimlib.VGroup)
assert popped_tips.submobjects == [end_tip, start_tip]
assert not tipped_line.has_tip()
assert not tipped_line.has_start_tip()
assert tipped_line.submobjects == []
assert np.allclose(tipped_line.get_start(), logical_start, atol=1e-9)
assert np.allclose(tipped_line.get_end(), logical_end, atol=1e-9)
# Reference get_tips is attribute-based, whereas has_tip also requires current
# family membership. Popping deliberately leaves the attributes in place.
assert tipped_line.get_tips().submobjects == [end_tip, start_tip]

curved_tippable = geometry.ArcBetweenPoints(
    [-2.0, 0.0, 0.0],
    [2.0, 0.0, 0.0],
    angle=math.pi,
)
curved_logical_end = curved_tippable.get_end().copy()
curved_points_before = curved_tippable.get_points().copy()
terminal_tangent = curved_points_before[-1] - curved_points_before[-2]
terminal_tangent /= np.linalg.norm(terminal_tangent)
assert curved_tippable.add_tip(length=0.35) is curved_tippable
curved_tip_vector = curved_tippable.tip.get_vector()
curved_tip_vector /= np.linalg.norm(curved_tip_vector)
assert np.dot(curved_tip_vector, terminal_tangent) > 1.0 - 1e-9
assert np.allclose(curved_tippable.get_end(), curved_logical_end, atol=1e-9)
assert not np.array_equal(curved_tippable.get_points(), curved_points_before)
assert curved_tippable.get_arc_length() < geometry.ArcBetweenPoints(
    [-2.0, 0.0, 0.0],
    [2.0, 0.0, 0.0],
    angle=math.pi,
).get_arc_length()

bound_tippable = geometry.Line([0.0, 2.0, 0.0], [3.0, 2.0, 0.0])
bound_tip_scene = Scene()
bound_tip_scene.add(bound_tippable)
bound_tippable_identity = id(bound_tippable)
assert bound_tippable.add_tip(length=0.3) is bound_tippable
assert id(bound_tippable) == bound_tippable_identity
assert bound_tip_scene.get_mobjects()[0] is bound_tippable
assert bound_tippable.tip._is_bound()
assert bound_tippable.submobjects == [bound_tippable.tip]
assert bound_tippable.family_size() == 2

try:
    geometry.Polygon()
except IndexError as error:
    assert "tuple index out of range" in str(error)
else:
    raise AssertionError("an empty Polygon did not preserve the Reference refusal")

try:
    geometry.RegularPolygon(1 << 20)
except ValueError as error:
    assert "compass directions" in str(error).lower()
else:
    raise AssertionError("an over-budget RegularPolygon reached allocation")

for invalid_n, error_type in ((0, ZeroDivisionError), (-1, IndexError), (3.5, TypeError)):
    try:
        geometry.RegularPolygon(invalid_n)
    except error_type:
        pass
    else:
        raise AssertionError(
            f"RegularPolygon({invalid_n!r}) did not preserve the Reference refusal"
        )

# fm-5wq.4.134: RegularPolygon's geometry parameters route through the
# native compass builder, while VMobject style kwargs route through the
# shared style pass instead of being silently discarded.
styled_pentagon = geometry.RegularPolygon(
    n=5,
    radius=1.5,
    start_angle=math.pi / 7.0,
    fill_color=manimlib.RED,
    fill_opacity=0.4,
    stroke_color=manimlib.BLUE,
    stroke_width=2.5,
    flat_stroke=True,
)
assert len(styled_pentagon.get_vertices()) == 5
assert np.allclose(
    np.linalg.norm(styled_pentagon.get_vertices(), axis=1), 1.5
)
assert styled_pentagon.get_fill_color() == manimlib.RED
assert np.isclose(styled_pentagon.get_fill_opacity(), 0.4)
assert styled_pentagon.get_stroke_color() == manimlib.BLUE
assert np.isclose(styled_pentagon.get_stroke_width(), 2.5)
assert styled_pentagon.get_flat_stroke() is True

try:
    geometry.RegularPolygon(wobble_amount=1)
except TypeError as error:
    assert "wobble_amount" in str(error), error
else:
    raise AssertionError("RegularPolygon silently ignored an unknown keyword")

quarter_arc = geometry.Arc(
    start_angle=0.0,
    angle=math.pi / 2.0,
    radius=2.0,
    arc_center=[1.0, -1.0, 0.0],
)
assert np.allclose(
    quarter_arc.pfp(0.5),
    [1.0 + math.sqrt(2.0), -1.0 + math.sqrt(2.0), 0.0],
    atol=2e-3,
)

# The rest of Atlas's Arc lineage must be authored portal classes over the
# existing native builders, not generated constructor refusals. Geometry
# queries read current Stage points so transforms cannot leave stale answers.
for name in (
    "ArcBetweenPoints",
    "CurvedArrow",
    "CurvedDoubleArrow",
    "Ellipse",
    "AnnularSector",
    "Sector",
    "Annulus",
):
    assert getattr(manimlib, name) is getattr(geometry, name)
assert issubclass(geometry.ArcBetweenPoints, geometry.Arc)
assert issubclass(geometry.CurvedArrow, geometry.ArcBetweenPoints)
assert issubclass(geometry.CurvedDoubleArrow, geometry.CurvedArrow)
assert issubclass(geometry.Ellipse, geometry.Circle)
assert issubclass(geometry.Sector, geometry.AnnularSector)
assert list(inspect.signature(geometry.ArcBetweenPoints).parameters) == [
    "start",
    "end",
    "angle",
    "kwargs",
]
assert list(inspect.signature(geometry.CurvedArrow).parameters) == [
    "start_point",
    "end_point",
    "kwargs",
]
assert list(inspect.signature(geometry.AnnularSector).parameters) == [
    "angle",
    "start_angle",
    "inner_radius",
    "outer_radius",
    "arc_center",
    "fill_color",
    "fill_opacity",
    "stroke_width",
    "kwargs",
]

assert np.allclose(quarter_arc.get_arc_center(), [1.0, -1.0, 0.0], atol=1e-9)
quarter_start = quarter_arc.get_start_angle()
assert min(abs(quarter_start), abs(quarter_start - math.tau)) < 1e-6
assert math.isclose(quarter_arc.get_stop_angle(), math.pi / 2.0, abs_tol=1e-6)
quarter_arc.shift([2.0, 3.0, 0.0])
assert np.allclose(quarter_arc.get_arc_center(), [3.0, 2.0, 0.0], atol=1e-9)
quarter_arc.rotate(math.pi / 2.0, about_point=quarter_arc.get_arc_center())
assert math.isclose(quarter_arc.get_start_angle(), math.pi / 2.0, abs_tol=1e-6)
assert math.isclose(quarter_arc.get_stop_angle(), math.pi, abs_tol=1e-6)

between = geometry.ArcBetweenPoints(
    [-2.0, -0.5, 0.0],
    [2.0, -0.5, 0.0],
    angle=math.pi / 2.0,
    n_components=5,
    color=manimlib.BLUE,
)
assert between.get_num_curves() == 5
assert np.allclose(between.get_start(), [-2.0, -0.5, 0.0], atol=1e-9)
assert np.allclose(between.get_end(), [2.0, -0.5, 0.0], atol=1e-9)
assert between.get_stroke_color() == manimlib.BLUE

curved_arrow = geometry.CurvedArrow(
    [-2.0, 0.0, 0.0],
    [2.0, 0.0, 0.0],
    angle=-math.pi / 2.0,
    color=manimlib.BLUE,
)
assert curved_arrow.submobjects == [curved_arrow.tip]
assert curved_arrow.tip.has_points()
assert np.min(
    np.linalg.norm(curved_arrow.tip.get_points() - [2.0, 0.0, 0.0], axis=1)
) < 1e-9
curved_double = geometry.CurvedDoubleArrow(
    [-2.0, 0.0, 0.0],
    [2.0, 0.0, 0.0],
    angle=math.pi / 2.0,
    color=manimlib.RED,
)
assert curved_double.submobjects == [curved_double.tip, curved_double.start_tip]
assert all(tip.has_points() for tip in curved_double.submobjects)
# Fitting the shaft to the start tip moves the already attached end tip (the
# Reference's order, a2c9c31c). It then lands on the endpoint only to within
# the f32 records: one f32 ulp at magnitude 2 is 2.4e-7. Measured on
# 2026-09-28: the pinned Reference 1.14e-7, the portal 1.48e-8.
assert np.min(
    np.linalg.norm(curved_double.tip.get_points() - [2.0, 0.0, 0.0], axis=1)
) < 2.4e-7
assert np.min(
    np.linalg.norm(curved_double.start_tip.get_points() - [-2.0, 0.0, 0.0], axis=1)
) < 1e-9

# The highest-demand primitive shelf is authored over Atlas rather than
# schema-generated shells.  Lock its declared hierarchy and signatures before
# checking the live construction and mutation semantics together.
assert geometry.Circle.__bases__ == (geometry.Arc,)
assert geometry.Dot.__bases__ == (geometry.Circle,)
assert geometry.SmallDot.__bases__ == (geometry.Dot,)
assert geometry.Rectangle.__bases__ == (geometry.Polygon,)
assert geometry.Square.__bases__ == (geometry.Rectangle,)
assert geometry.RoundedRectangle.__bases__ == (geometry.Rectangle,)
assert geometry.Polyline.__bases__ == (VMobject,)
assert list(inspect.signature(geometry.Circle).parameters) == [
    "start_angle",
    "stroke_color",
    "kwargs",
]
assert inspect.signature(geometry.Circle).parameters["stroke_color"].default == manimlib.RED
assert list(inspect.signature(geometry.Dot).parameters) == [
    "point",
    "radius",
    "stroke_color",
    "stroke_width",
    "fill_opacity",
    "fill_color",
    "kwargs",
]
dot_parameters = inspect.signature(geometry.Dot).parameters
assert dot_parameters["radius"].default == geometry.DEFAULT_DOT_RADIUS == 0.08
assert dot_parameters["stroke_color"].default == manimlib.BLACK
assert dot_parameters["stroke_width"].default == 0.0
assert dot_parameters["fill_opacity"].default == 1.0
assert dot_parameters["fill_color"].default == manimlib.DEFAULT_MOBJECT_COLOR
assert str(inspect.signature(geometry.SmallDot)) == (
    "(point=array([0., 0., 0.]), radius=0.04, **kwargs)"
)
assert geometry.DEFAULT_SMALL_DOT_RADIUS == 0.04
assert str(inspect.signature(geometry.Rectangle)) == (
    "(width=4.0, height=2.0, **kwargs)"
)
assert str(inspect.signature(geometry.Square)) == (
    "(side_length=2.0, **kwargs)"
)
assert str(inspect.signature(geometry.RoundedRectangle)) == (
    "(width=4.0, height=2.0, corner_radius=0.5, **kwargs)"
)
assert str(inspect.signature(geometry.Polyline)) == "(*vertices, **kwargs)"

arc_center_mover = geometry.Arc(
    start_angle=0.3,
    angle=1.2,
    radius=1.4,
    arc_center=[-0.5, 0.25, 0.0],
    stroke_color=manimlib.BLUE,
    stroke_width=2.0,
)
arc_center_identity = id(arc_center_mover)
arc_center_style = arc_center_mover.get_style()
arc_center_mover.uniforms["anti_alias_width"] = 2.5
assert arc_center_mover.move_arc_center_to([1.25, -0.75, 0.0]) is arc_center_mover
assert id(arc_center_mover) == arc_center_identity
assert np.allclose(arc_center_mover.get_arc_center(), [1.25, -0.75, 0.0])
arc_center_style_after = arc_center_mover.get_style()
assert all(
    np.array_equal(arc_center_style_after[key], value)
    for key, value in arc_center_style.items()
)
assert arc_center_mover.uniforms["anti_alias_width"] == 2.5

circle_target = geometry.Rectangle(width=2.0, height=1.0).shift([0.5, -0.25, 0.0])
circle_surround = geometry.Circle(stroke_width=3.0)
circle_surround_identity = id(circle_surround)
circle_surround.uniforms["depth_test"] = True
assert circle_surround.surround(
    circle_target, dim_to_match=0, stretch=True, buff=0.2
) is circle_surround
assert id(circle_surround) == circle_surround_identity
assert np.allclose(circle_surround.get_center(), circle_target.get_center())
assert np.allclose(
    [circle_surround.get_width(), circle_surround.get_height()],
    [2.4, 1.4],
    atol=1e-6,
)
assert circle_surround.get_stroke_width() == 3.0
assert circle_surround.uniforms["depth_test"] is True

rectangle_surround = geometry.Rectangle(
    width=0.5,
    height=0.25,
    stroke_color=manimlib.BLUE,
    stroke_width=2.0,
)
rectangle_identity = id(rectangle_surround)
rectangle_surround.uniforms["anti_alias_width"] = 3.25
assert rectangle_surround.surround(circle_target, buff=0.15) is rectangle_surround
assert id(rectangle_surround) == rectangle_identity
assert np.allclose(rectangle_surround.get_center(), circle_target.get_center())
assert np.allclose(
    [rectangle_surround.get_width(), rectangle_surround.get_height()],
    [2.3, 1.3],
    atol=1e-6,
)
assert rectangle_surround.get_stroke_color() == manimlib.BLUE
assert rectangle_surround.get_stroke_width() == 2.0
assert rectangle_surround.uniforms["anti_alias_width"] == 3.25

point_target = manimlib.VectorizedPoint([2.0, 3.0, 0.0])
degenerate_surround = geometry.Rectangle().surround(point_target, buff=0.1)
assert np.allclose(degenerate_surround.get_center(), [2.0, 3.0, 0.0])
assert np.allclose(
    [degenerate_surround.get_width(), degenerate_surround.get_height()],
    [0.2, 0.2],
    atol=1e-6,
)

bound_primitive_scene = Scene()
bound_rectangle = geometry.Rectangle(width=0.5, height=0.5)
bound_primitive_scene.add(bound_rectangle)
bound_rectangle_identity = id(bound_rectangle)
assert bound_rectangle._is_bound()
assert bound_rectangle.surround(circle_target, buff=0.05) is bound_rectangle
assert id(bound_rectangle) == bound_rectangle_identity
assert bound_rectangle._is_bound()
assert bound_primitive_scene.mobjects == [bound_rectangle]
assert np.allclose(
    [bound_rectangle.get_width(), bound_rectangle.get_height()],
    [2.1, 1.1],
    atol=1e-6,
)

dot = geometry.Dot([1.0, -2.0, 0.0], radius=0.2)
assert isinstance(dot, geometry.Circle)
assert np.allclose(dot.get_center(), [1.0, -2.0, 0.0])
assert math.isclose(dot.get_radius(), 0.2, abs_tol=1e-6)
assert dot.get_stroke_color() == manimlib.BLACK
assert dot.get_stroke_width() == 0.0
assert dot.get_fill_opacity() == 1.0
assert dot.get_fill_color() == manimlib.DEFAULT_MOBJECT_COLOR
small_dot = geometry.SmallDot([-1.0, 2.0, 0.0])
assert isinstance(small_dot, geometry.Dot)
assert math.isclose(small_dot.get_radius(), 0.04, abs_tol=1e-6)

rectangle = geometry.Rectangle(
    width=3.0,
    height=1.5,
    stroke_color=manimlib.BLUE,
    stroke_width=1.5,
)
assert isinstance(rectangle, geometry.Polygon)
assert np.allclose([rectangle.get_width(), rectangle.get_height()], [3.0, 1.5])
assert len(rectangle.get_vertices()) == 4
square = geometry.Square(side_length=1.25)
assert np.allclose([square.get_width(), square.get_height()], [1.25, 1.25])
rounded = geometry.RoundedRectangle(
    width=3.0,
    height=1.5,
    corner_radius=0.25,
    stroke_color=manimlib.RED,
)
assert np.allclose([rounded.get_width(), rounded.get_height()], [3.0, 1.5])
assert rounded.get_num_curves() > rectangle.get_num_curves()
assert rounded.get_stroke_color() == manimlib.RED
try:
    geometry.RoundedRectangle(corner_radius=math.nan)
except ValueError:
    pass
else:
    raise AssertionError("RoundedRectangle accepted a non-finite corner radius")

polyline = geometry.Polyline(
    [-1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [1.0, 0.0, 0.0],
    stroke_color=manimlib.BLUE,
    stroke_width=2.0,
)
assert np.allclose(polyline.get_start(), [-1.0, 0.0, 0.0])
assert np.allclose(polyline.get_end(), [1.0, 0.0, 0.0])
assert not np.allclose(polyline.get_start(), polyline.get_end())
assert polyline.get_stroke_color() == manimlib.BLUE
assert geometry.Polyline().get_num_points() == 0

circle_query = geometry.Circle(radius=2.0, arc_center=[1.0, -1.0, 0.0])
assert math.isclose(circle_query.get_radius(), 2.0, abs_tol=1e-6)
assert np.allclose(circle_query.point_at_angle(math.pi / 2.0), [1.0, 1.0, 0.0])
circle_query.scale(1.5)
assert math.isclose(circle_query.get_radius(), 3.0, abs_tol=1e-6)
assert np.allclose(circle_query.point_at_angle(math.pi), [-2.0, -1.0, 0.0])


# geometry.py Circle.point_at_angle measures from self.get_start_angle() and
# walks self.point_from_proportion(), so a subclass overriding either one is
# honored, as it is in the Reference.
class _QuarterStartCircle(geometry.Circle):
    def get_start_angle(self):
        return math.tau / 4.0


class _ProportionReportingCircle(geometry.Circle):
    def point_from_proportion(self, alpha):
        return np.array([alpha, 9.0, 0.0])


assert np.allclose(_QuarterStartCircle(radius=2.0).point_at_angle(math.pi / 2.0), [2.0, 0.0, 0.0])
assert np.allclose(_ProportionReportingCircle().point_at_angle(math.pi), [0.5, 9.0, 0.0])

ellipse = geometry.Ellipse(
    width=4.0,
    height=1.5,
    arc_center=[0.5, -0.25, 0.0],
    start_angle=math.pi / 2.0,
    color=manimlib.BLUE,
)
assert np.allclose([ellipse.get_width(), ellipse.get_height()], [4.0, 1.5], atol=1e-6)
assert np.allclose(ellipse.get_center(), [0.5, -0.25, 0.0], atol=1e-6)
assert np.allclose(ellipse.get_points()[0], [0.5, 0.5, 0.0], atol=1e-6)

annular_sector = geometry.AnnularSector(
    angle=math.pi,
    start_angle=math.pi / 2.0,
    inner_radius=0.5,
    outer_radius=1.5,
    arc_center=[1.0, 0.0, 0.0],
    fill_color=manimlib.RED,
)
assert annular_sector.has_points()
assert annular_sector.get_fill_color() == manimlib.RED
assert math.isclose(annular_sector.get_left()[0], -0.5, abs_tol=2e-3)
assert math.isclose(annular_sector.get_right()[0], 1.0, abs_tol=2e-3)
sector = geometry.Sector(angle=math.pi / 2.0, radius=2.0)
assert sector.has_points()
assert np.min(np.linalg.norm(sector.get_points(), axis=1)) < 1e-9
annulus = geometry.Annulus(
    inner_radius=0.75,
    outer_radius=1.5,
    center=[-0.5, 0.25, 0.0],
    fill_color=manimlib.BLUE,
)
assert math.isclose(annulus.radius, 1.5)
assert len(annulus.get_subpaths()) == 2
assert annulus.get_fill_color() == manimlib.BLUE
assert np.allclose(annulus.get_center(), [-0.5, 0.25, 0.0], atol=1e-6)

try:
    geometry.ArcBetweenPoints([0, 0, 0], [1, 0, 0], n_components=0)
except ValueError as error:
    assert "component" in str(error).lower()
else:
    raise AssertionError("ArcBetweenPoints accepted a zero component budget")
try:
    geometry.Annulus(unrecognized_style=True)
except TypeError as error:
    assert "unrecognized_style" in str(error)
else:
    raise AssertionError("Annulus silently ignored an unknown keyword")

selector_tex = manimlib.Tex(r"\cos(\theta) + \sin(\theta)")
selected_thetas = selector_tex[r"\theta"]
assert len(selected_thetas) == 2
assert all(len(part) > 0 for part in selected_thetas)

brace_module = importlib.import_module("manimlib.mobject.svg.brace")
brace_line = geometry.Line([-1.0, -1.0, 0.0], [1.0, 1.0, 0.0])
line_brace = brace_module.LineBrace(brace_line, buff=0.1)
assert isinstance(line_brace, brace_module.Brace)
brace_tip = line_brace.get_tip()
brace_label = line_brace.get_tex("1")
assert np.dot(brace_label.get_center() - brace_tip, line_brace.get_direction()) > 0

# BraceLabel/BraceText are authored live compositions over Atlas's parametric
# Brace and Scribe's bundled Tex/TexText constructors. The two public children
# keep stable identity across mutation, copying remaps those aliases, and the
# familiar creation animation is a real Choreo composition.
assert brace_module.BraceLabel.__bases__ == (VMobject,)
assert brace_module.BraceText.__bases__ == (brace_module.BraceLabel,)
brace_label_signature = inspect.signature(brace_module.BraceLabel)
assert tuple(brace_label_signature.parameters) == (
    "obj",
    "text",
    "brace_direction",
    "label_scale",
    "label_buff",
    "kwargs",
)
assert np.array_equal(
    brace_label_signature.parameters["brace_direction"].default,
    manimlib.DOWN,
)
assert brace_label_signature.parameters["label_scale"].default == 1.0
assert (
    brace_label_signature.parameters["label_buff"].default
    == manimlib.DEFAULT_MOBJECT_TO_MOBJECT_BUFF
)
assert str(inspect.signature(brace_module.BraceLabel.creation_anim)) == (
    "(self, label_anim=<class 'manimlib.animation.fading.FadeIn'>, "
    "brace_anim=<class 'manimlib.animation.growing.GrowFromCenter'>)"
)
assert str(inspect.signature(brace_module.BraceLabel.shift_brace)) == (
    "(self, obj, **kwargs)"
)
assert str(inspect.signature(brace_module.BraceLabel.change_label)) == (
    "(self, *text, **kwargs)"
)
assert str(inspect.signature(brace_module.BraceLabel.change_brace_label)) == (
    "(self, obj, *text)"
)
assert str(inspect.signature(brace_module.BraceLabel.copy)) == "(self)"
assert brace_module.BraceLabel.label_constructor is manimlib.Tex
assert brace_module.BraceText.label_constructor is manimlib.TexText

label_target = geometry.Rectangle(width=2.0, height=0.75).shift((0.25, 0.5, 0.0))
native_brace_label = brace_module.BraceLabel(
    label_target,
    ("x", "+", "y"),
    brace_direction=manimlib.DOWN,
    label_scale=0.6,
    label_buff=0.15,
    color=manimlib.BLUE,
)
assert native_brace_label.submobjects == [
    native_brace_label.brace,
    native_brace_label.label,
]
assert isinstance(native_brace_label.brace, brace_module.Brace)
assert isinstance(native_brace_label.label, manimlib.Tex)
assert native_brace_label.label.get_tex() == "x + y"
assert np.dot(
    native_brace_label.label.get_center() - native_brace_label.brace.get_tip(),
    native_brace_label.brace.get_direction(),
) > 0
assert all(
    member.get_fill_color() == manimlib.BLUE
    for member in native_brace_label.family_members_with_points()
)
brace_font_size = native_brace_label.brace.font_size
label_font_size = native_brace_label.label.font_size
native_brace_label.scale(0.8)
assert np.isclose(native_brace_label.brace.font_size, 0.8 * brace_font_size)
assert np.isclose(native_brace_label.label.font_size, 0.8 * label_font_size)

creation = native_brace_label.creation_anim()
assert isinstance(creation, manimlib.AnimationGroup)
assert len(creation.animations) == 2
assert isinstance(creation.animations[0], manimlib.GrowFromCenter)
assert creation.animations[0].mobject is native_brace_label.brace
assert isinstance(creation.animations[1], manimlib.FadeIn)
assert creation.animations[1].mobject is native_brace_label.label

native_brace_label_copy = native_brace_label.copy()
assert native_brace_label_copy is not native_brace_label
assert native_brace_label_copy.brace is native_brace_label_copy[0]
assert native_brace_label_copy.label is native_brace_label_copy[1]
assert native_brace_label_copy.brace is not native_brace_label.brace
assert native_brace_label_copy.label is not native_brace_label.label
native_brace_label_copy.shift((1.0, 0.0, 0.0))
assert not np.allclose(
    native_brace_label_copy.brace.get_center(),
    native_brace_label.brace.get_center(),
)

original_brace = native_brace_label.brace
original_label = native_brace_label.label
replacement_target = geometry.Circle(radius=0.5).shift((-1.0, 0.25, 0.0))
assert native_brace_label.shift_brace(replacement_target) is native_brace_label
assert native_brace_label.brace is native_brace_label[0]
assert native_brace_label.brace is not original_brace
assert native_brace_label.label is original_label
assert native_brace_label.label is native_brace_label[1]

shifted_brace = native_brace_label.brace
assert native_brace_label.change_label("z", color=manimlib.RED) is native_brace_label
assert native_brace_label.brace is shifted_brace
assert native_brace_label.label is native_brace_label[1]
assert native_brace_label.label is not original_label
assert native_brace_label.label.get_tex() == "z"
assert all(
    member.get_fill_color() == manimlib.RED
    for member in native_brace_label.label.family_members_with_points()
)

changed_label = native_brace_label.label
assert native_brace_label.change_brace_label(label_target, "q") is native_brace_label
assert native_brace_label.brace is native_brace_label[0]
assert native_brace_label.label is native_brace_label[1]
assert native_brace_label.label is not changed_label
assert native_brace_label.label.get_tex() == "q"

native_brace_text = brace_module.BraceText(
    [geometry.Circle(radius=0.2), geometry.Circle(radius=0.2).shift((1, 0, 0))],
    "native text",
    label_scale=0.75,
)
assert isinstance(native_brace_text.label, manimlib.TexText)
assert native_brace_text.submobjects == [
    native_brace_text.brace,
    native_brace_text.label,
]

# TransformMatchingTex consumes the source identities carried by Scribe's
# native span map. Equal `x` spans pair even though their byte offsets differ;
# the `+`/`-` leftovers remain distinct and therefore take the native fades.
matching_module = importlib.import_module(
    "manimlib.animation.transform_matching_parts"
)
assert matching_module.TransformMatchingTex.__bases__ == (
    matching_module.TransformMatchingStrings,
)
matching_source = manimlib.Tex("x + y").shift(manimlib.LEFT)
matching_target = manimlib.Tex("z - x").shift(manimlib.RIGHT)
matching_animation = matching_module.TransformMatchingTex(
    matching_source,
    matching_target,
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
matching_params = matching_animation._native_params()
source_x = next(part for part, key in matching_params["source_keys"] if key == "x")
target_x = next(part for part, key in matching_params["target_keys"] if key == "x")
assert source_x is not target_x
assert any(key == "+" for _, key in matching_params["source_keys"])
assert any(key == "-" for _, key in matching_params["target_keys"])
matching_scene = Scene()
matching_scene.play(matching_animation)
assert np.allclose(source_x.get_center(), target_x.get_center())

try:
    manimlib.Tex(r"\dx")
except bridge_errors.TexError as error:
    assert r"`\dx` is not yet supported" in str(error)
else:
    raise AssertionError("unsupported TeX did not raise the named TexError")

# A visible string whose span map was dropped keeps its provenance refusal
# (383813b5 animates genuinely empty content, not missing maps).
empty_span_source = manimlib.Tex("x")
empty_span_source._string_sub_spans = []
try:
    matching_module.TransformMatchingTex(empty_span_source, manimlib.Tex("x"))
except bridge_errors.TexError as error:
    assert "native span" in str(error), error
else:
    raise AssertionError("TransformMatchingTex accepted an empty span map")

try:
    brace_module.BraceLabel(object(), "invalid target")
except TypeError:
    pass
else:
    raise AssertionError("BraceLabel accepted a non-Mobject target")

# Existing Chisel/Scribe semantics are live through the portal, rather than
# being shadowed by schema placeholders. Arc length remains true for either
# curvature sign (BN-03), and Tex selectors consume the native UTF-8 span map.
line = geometry.Line((-1.0, 0.0, 0.0), (1.0, 0.0, 0.0))
assert math.isclose(line.get_arc_length(), 2.0, rel_tol=0.0, abs_tol=1e-9)
assert np.array_equal(line.get_vector(), [2.0, 0.0, 0.0])
assert np.array_equal(line.get_unit_vector(), [1.0, 0.0, 0.0])
assert line.get_angle() == 0.0
assert line.get_slope() == 0.0
assert np.array_equal(line.get_projection([0.25, 7.0, 3.0]), [0.25, 0.0, 0.0])
assert np.array_equal(line.pointify([2.0, 3.0]), [2.0, 3.0, 0.0])
assert line.set_length(4.0) is line
assert math.isclose(line.get_length(), 4.0, rel_tol=0.0, abs_tol=1e-9)
line_start = line.get_start().copy()
assert line.set_angle(math.pi / 2.0) is line
assert np.allclose(line.get_start(), line_start, rtol=0.0, atol=1e-9)
assert np.allclose(line.get_unit_vector(), [0.0, 1.0, 0.0], rtol=0.0, atol=1e-9)

geom_mod = importlib.import_module("manimlib.mobject.geometry")
assert geom_mod.DEFAULT_ARROW_TIP_LENGTH == 0.35
assert geom_mod.DEFAULT_ARROW_TIP_WIDTH == 0.35
assert geom_mod.DEFAULT_DASH_LENGTH == 0.05
assert geom_mod.math is math
perp_line = geometry.Line(manimlib.LEFT, manimlib.RIGHT)
perp_start = perp_line.get_start().copy()
perp_frame = manimlib.CameraFrame()
assert perp_line.set_perpendicular_to_camera(perp_frame) is perp_line
assert np.allclose(perp_line.get_start(), perp_start, rtol=0.0, atol=1e-9)
assert np.isclose(np.linalg.norm(perp_line.get_unit_normal()), 1.0)

line_style = geometry.Line(
    (-2.0, -1.0, 0.0),
    (2.0, -1.0, 0.0),
    stroke_color=manimlib.RED,
    stroke_width=3.0,
).fix_in_frame()
style_before = line_style.get_style()
uniforms_before = line_style.uniforms.copy()
line_style_identity = id(line_style)
assert line_style.set_points_by_ends(
    (-1.0, 1.0, 0.0),
    (1.0, 1.0, 0.0),
    buff=0.25,
) is line_style
assert id(line_style) == line_style_identity
assert np.allclose(line_style.get_start(), [-0.75, 1.0, 0.0])
assert np.allclose(line_style.get_end(), [0.75, 1.0, 0.0])
style_after = line_style.get_style()
assert style_after.keys() == style_before.keys()
assert all(np.array_equal(style_after[key], value) for key, value in style_before.items())
assert line_style.uniforms.keys() == uniforms_before.keys()
assert all(
    np.array_equal(line_style.uniforms[key], value)
    for key, value in uniforms_before.items()
)

assert line_style.set_path_arc(-math.pi / 2.0) is line_style
assert line_style.path_arc == -math.pi / 2.0
assert line_style.get_num_curves() == 4
assert line_style.get_arc_length() > line_style.get_length()
curved_points_before = line_style.get_points().copy()
curved_path_arc_before = line_style.path_arc
try:
    line_style.set_path_arc(float("nan"))
except ValueError as error:
    assert "arc angle must be finite" in str(error)
else:
    raise AssertionError("non-finite path_arc did not refuse")
assert line_style.path_arc == curved_path_arc_before
assert np.array_equal(line_style.get_points(), curved_points_before)

null_line = geometry.Line((0.0, 0.0, 0.0), (0.0, 0.0, 0.0))
assert null_line.put_start_and_end_on(
    (-1.0, 2.0, 0.0),
    (2.0, 2.0, 0.0),
) is null_line
assert np.array_equal(null_line.get_start(), [-1.0, 2.0, 0.0])
assert np.array_equal(null_line.get_end(), [2.0, 2.0, 0.0])

bound_line = geometry.Line((0.0, -2.0, 0.0), (1.0, -2.0, 0.0))
bound_line_scene = Scene()
bound_line_scene.add(bound_line)
bound_line_identity = id(bound_line)
assert bound_line.set_points_by_ends(
    (-2.0, -2.0, 0.0),
    (2.0, -2.0, 0.0),
    path_arc=math.pi / 3.0,
) is bound_line
assert id(bound_line) == bound_line_identity
assert bound_line_scene.get_mobjects()[0] is bound_line
assert bound_line.get_arc_length() > bound_line.get_length()
assert bound_line.reset_points_around_ends() is bound_line

curved_line = geometry.Line(
    (-1.0, 0.0, 0.0),
    (1.0, 0.0, 0.0),
    path_arc=-math.pi / 2.0,
)
assert curved_line.get_arc_length() > curved_line.get_length()
assert curved_line.get_arc_length() == VMobject.get_arc_length(curved_line, 2)
arclength_curve = VMobject().set_anchors_and_handles(
    [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
    [[0.0, 1.0, 0.0]],
)
assert np.allclose(arclength_curve.point_from_proportion(0.0), [-1.0, 0.0, 0.0])
assert np.allclose(arclength_curve.point_from_proportion(0.5), [0.0, 0.5, 0.0])
assert np.allclose(arclength_curve.point_from_proportion(1.0), [1.0, 0.0, 0.0])
rotated_line = line.copy().rotate(math.pi / 2.0)
assert np.allclose(rotated_line.get_points()[0], rotated_line.get_start())
rotated_line.get_points()[0] = [3.0, 4.0, 0.0]
assert np.allclose(rotated_line.get_start(), [3.0, 4.0, 0.0])
corners = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]]
)
assert np.allclose(corners.get_points()[::2], [[0, 0, 0], [1, 1, 0], [2, 0, 0]])
corners.reverse_points()
assert np.allclose(corners.get_points()[::2], [[2, 0, 0], [1, 1, 0], [0, 0, 0]])

# VMobject's construction surface now crosses one bounded Chisel seam. Native
# QuadPath computes the complete result before Python commits any structured
# record rows, so refusal is atomic and style lanes keep Reference resize rules.
path = VMobject(
    stroke_color=manimlib.RED,
    stroke_width=7.0,
    fill_color=manimlib.BLUE,
    fill_opacity=0.25,
)
path_defaults = path._style_data().copy()
assert path.start_new_path([0.0, 0.0, 0.0]) is path
assert path.has_new_path_started()
assert np.allclose(path.get_last_point(), [0.0, 0.0, 0.0])
for field in (
    "stroke_rgba",
    "stroke_width",
    "fill_rgba",
    "base_normal",
    "fill_border_width",
):
    assert np.array_equal(path.data[field], path_defaults[field])

terminal_style = path.data.copy()[-1]
assert path.add_line_to([2.0, 0.0, 0.0]) is path
assert np.allclose(
    path.get_points(),
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
)
assert not path.has_new_path_started()
assert np.allclose(path.get_reflection_of_last_handle(), [3.0, 0.0, 0.0])
for field in ("stroke_rgba", "stroke_width", "fill_rgba", "base_normal"):
    assert np.array_equal(path.data[field][1], terminal_style[field])
    assert np.array_equal(path.data[field][2], terminal_style[field])

nth_curve = path.get_nth_curve_points(0)
assert np.shares_memory(nth_curve, path.get_points())
curve_function = path.get_nth_curve_function(0)
nth_curve[1] = [1.0, 1.0, 0.0]
assert np.allclose(curve_function(0.5), [1.0, 0.5, 0.0])
try:
    path.get_nth_curve_points(path.get_num_curves())
except AssertionError:
    pass
else:
    raise AssertionError("get_nth_curve_points accepted an out-of-range curve")

long_line = VMobject(long_lines=True).start_new_path([0.0, 0.0, 0.0])
long_line.add_line_to([2.0, 0.0, 0.0])
assert np.allclose(long_line.get_points()[:, 0], [0.0, 0.5, 1.0, 1.5, 2.0])

null_path = VMobject().start_new_path([1.0, 2.0, 0.0])
null_path.tolerance_for_point_equality = 1e-3
null_before = null_path.data.copy()
null_path.add_line_to([1.0005, 2.0, 0.0], allow_null_line=False)
null_path.add_quadratic_bezier_curve_to(
    [1.0, 2.0, 0.0],
    [1.0005, 2.0, 0.0],
    allow_null_curve=False,
)
assert np.array_equal(null_path.data, null_before)

quadratic = VMobject().start_new_path([0.0, 0.0, 0.0])
quadratic.add_quadratic_bezier_curve_to(
    [0.0, 0.0, 0.0], [2.0, 0.0, 0.0]
)
assert np.allclose(quadratic.get_points()[1], [1.0, 0.0, 0.0])

cubic = VMobject()
assert (
    cubic.add_cubic_bezier_curve(
        [0.0, 0.0, 0.0],
        [0.0, 2.0, 0.0],
        [2.0, 2.0, 0.0],
        [2.0, 0.0, 0.0],
    )
    is cubic
)
assert cubic.get_num_curves() >= 1
assert np.allclose(cubic.get_last_point(), [2.0, 0.0, 0.0])
cubic_before = cubic.data.copy()
try:
    cubic.add_cubic_bezier_curve_to(
        [float("nan"), 0.0, 0.0],
        [3.0, 1.0, 0.0],
        [4.0, 0.0, 0.0],
    )
except ValueError as error:
    assert "quadratics" in str(error).lower() and "cap" in str(error).lower()
else:
    raise AssertionError("cubic construction accepted non-finite geometry")
assert np.array_equal(cubic.data, cubic_before)

smooth = VMobject().start_new_path([0.0, 0.0, 0.0])
assert smooth.add_smooth_curve_to([1.0, 0.0, 0.0]) is smooth
assert smooth.add_smooth_cubic_curve_to([1.5, 1.0, 0.0], [2.0, 0.0, 0.0]) is smooth
assert np.allclose(smooth.get_last_point(), [2.0, 0.0, 0.0])

multi_path = VMobject().start_new_path([0.0, 0.0, 0.0])
multi_path.add_points_as_corners([[1.0, 0.0, 0.0], [1.0, 1.0, 0.0]])
multi_path.start_new_path([2.0, 0.0, 0.0]).add_line_to([3.0, 0.0, 0.0])
first_subpath = multi_path.get_subpaths()[0].copy()
assert not multi_path.is_closed()
assert multi_path.close_path() is multi_path
assert multi_path.is_closed()
assert np.array_equal(multi_path.get_subpaths()[0], first_subpath)
closed_before = multi_path.data.copy()
multi_path.close_path()
assert np.array_equal(multi_path.data, closed_before)

smooth_closed = VMobject().start_new_path([0.0, 0.0, 0.0])
smooth_closed.add_line_to([1.0, 0.0, 0.0]).add_line_to([1.0, 1.0, 0.0])
smooth_closed.close_path(smooth=True)
assert smooth_closed.is_closed()

square_path = VMobject().start_new_path([0.0, 0.0, 0.0])
square_path.add_points_as_corners(
    [[1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]]
).close_path()
assert np.allclose(square_path.get_area_vector(), [0.0, 0.0, 1.0])
assert np.allclose(square_path.get_unit_normal(), [0.0, 0.0, 1.0])
assert np.allclose(square_path.data["base_normal"][1::2], [0.0, 0.0, 1.0])
square_path.get_points()[2] = [1.0, -1.0, 0.0]
assert np.allclose(
    square_path.get_unit_normal(refresh=True),
    square_path._path_unit_normal(),
)

anchors_path = VMobject().set_points_as_corners(
    [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]
)
anchors_before = anchors_path.data.copy()
try:
    anchors_path.set_anchors_and_handles(
        [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        [],
    )
except ValueError as error:
    assert "anchor" in str(error).lower() and "handle" in str(error).lower()
else:
    raise AssertionError("mismatched anchors and handles were accepted")
assert np.array_equal(anchors_path.data, anchors_before)
assert (
    anchors_path.set_anchors_and_handles(
        [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]],
        [[0.5, 1.0, 0.0], [1.5, 1.0, 0.0]],
    )
    is anchors_path
)
assert np.allclose(
    anchors_path.get_points(),
    [
        [0.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.5, 1.0, 0.0],
        [2.0, 0.0, 0.0],
    ],
)

empty_path = VMobject()
empty_before = empty_path.data.copy()
try:
    empty_path.add_line_to([1.0, 0.0, 0.0])
except ValueError as error:
    assert "path with at least one point" in str(error).lower()
else:
    raise AssertionError("line construction accepted an empty path")
assert np.array_equal(empty_path.data, empty_before)

bound_path_scene = Scene()
bound_path = VMobject()
bound_path_scene.add(bound_path)
bound_path.start_new_path([-1.0, 0.0, 0.0]).add_line_to([1.0, 0.0, 0.0])
bound_path.start_new_path([0.0, 1.0, 0.0]).add_line_to([0.0, 2.0, 0.0])
assert bound_path.get_num_curves() == 3
assert np.allclose(bound_path.get_last_point(), [0.0, 2.0, 0.0])

# The remaining topology operations also route through QuadPath. The public
# threshold controls only the line-vs-arc decision; BN-09 controls density.
equality_path = VMobject()
equality_path.tolerance_for_point_equality = 1e-3
assert equality_path.consider_points_equal([0, 0, 0], [0.0009, -0.0009, 0])
assert not equality_path.consider_points_equal([0, 0, 0], [0.001, 0, 0])

threshold_line = VMobject().start_new_path([0.0, 0.0, 0.0])
assert (
    threshold_line.add_arc_to(
        [1.0, 0.0, 0.0], 5e-4, n_components=1, threshold=1e-3
    )
    is threshold_line
)
assert np.allclose(threshold_line.get_points()[1], [0.5, 0.0, 0.0])

threshold_arc = VMobject().start_new_path([0.0, 0.0, 0.0])
threshold_arc.add_arc_to(
    [1.0, 0.0, 0.0], 5e-4, n_components=1, threshold=1e-4
)
assert not np.allclose(threshold_arc.get_points()[1], [0.5, 0.0, 0.0])

dense_arc = VMobject().start_new_path([0.0, 0.0, 0.0])
dense_arc.add_arc_to([1.0, 1.0, 0.0], math.pi / 2.0)
assert dense_arc.get_num_curves() == 4
explicit_arc = VMobject().start_new_path([0.0, 0.0, 0.0])
explicit_arc.add_arc_to([1.0, 1.0, 0.0], math.pi / 2.0, n_components=2)
assert explicit_arc.get_num_curves() == 2
for bad_kwargs, message in (
    ({"n_components": 0}, "component"),
    ({"threshold": float("nan")}, "threshold"),
    ({"threshold": -1.0}, "threshold"),
):
    refusal_arc = VMobject().start_new_path([0.0, 0.0, 0.0])
    refusal_before = refusal_arc.data.copy()
    try:
        refusal_arc.add_arc_to([1.0, 0.0, 0.0], 5e-4, **bad_kwargs)
    except ValueError as error:
        assert message in str(error).lower()
    else:
        raise AssertionError(f"add_arc_to accepted {bad_kwargs}")
    assert np.array_equal(refusal_arc.data, refusal_before)

subpath = VMobject()
first_run = np.array(
    [[0.0, 0.0, 0.0], [0.5, 0.5, 0.0], [1.0, 0.0, 0.0]]
)
assert subpath.add_subpath(first_run) is subpath
assert np.allclose(subpath.get_points(), first_run)
subpath.add_subpath(
    [[1.0, 0.0, 0.0], [1.5, -0.5, 0.0], [2.0, 0.0, 0.0]]
)
assert len(subpath.get_subpaths()) == 1
subpath.add_subpath(
    [[3.0, 0.0, 0.0], [3.5, 0.5, 0.0], [4.0, 0.0, 0.0]]
)
assert len(subpath.get_subpaths()) == 2
subpath_before = subpath.data.copy()
assert subpath.add_subpath([]) is subpath
assert np.array_equal(subpath.data, subpath_before)
try:
    subpath.add_subpath([[5.0, 0.0, 0.0], [6.0, 0.0, 0.0]])
except ValueError as error:
    assert "odd length" in str(error).lower()
else:
    raise AssertionError("add_subpath accepted an even shared-anchor run")
assert np.array_equal(subpath.data, subpath_before)

anchor_modes = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]]
)
anchor_modes.data["stroke_width"][:, 0] = [1.0, 2.0, 3.0, 4.0, 5.0]
anchor_mode_widths = anchor_modes.data["stroke_width"].copy()
assert not anchor_modes.is_smooth()
assert anchor_modes.change_anchor_mode("approx_smooth") is anchor_modes
assert anchor_modes.is_smooth()
assert np.array_equal(anchor_modes.data["stroke_width"], anchor_mode_widths)
assert anchor_modes.make_jagged(recurse=False) is anchor_modes
assert not anchor_modes.is_smooth()
assert anchor_modes.make_approximately_smooth(recurse=False) is anchor_modes
assert anchor_modes.is_smooth()
anchor_before = anchor_modes.data.copy()
try:
    anchor_modes.change_anchor_mode("mystery")
except ValueError as error:
    assert "anchor mode" in str(error).lower()
else:
    raise AssertionError("change_anchor_mode accepted an unknown mode")
assert np.array_equal(anchor_modes.data, anchor_before)

mode_parent = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]]
)
mode_child = VMobject().set_points_as_corners(
    [[0.0, 2.0, 0.0], [1.0, 3.0, 0.0], [2.0, 2.0, 0.0]]
)
mode_parent.add(mode_child)
mode_child.make_approximately_smooth(recurse=False)
child_smooth_points = mode_child.get_points().copy()
mode_parent.make_jagged(recurse=False)
assert np.array_equal(mode_child.get_points(), child_smooth_points)
mode_parent.make_jagged(recurse=True)
assert not np.array_equal(mode_child.get_points(), child_smooth_points)

# VMobject's pre-pass honors recurse, while the Reference base reversal still
# reverses every family row. The odd normal rows reveal the phase boundary.
reverse_parent = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]
)
reverse_child = VMobject().set_points_as_corners(
    [[2.0, 0.0, 0.0], [3.0, 0.0, 0.0]]
)
reverse_parent.add(reverse_child)
reverse_parent.data["base_normal"][:] = [0.0, 0.0, 1.0]
reverse_child.data["base_normal"][:] = [0.0, 0.0, 1.0]
reverse_parent.data["stroke_width"][:, 0] = [1.0, 2.0, 3.0]
reverse_child.data["stroke_width"][:, 0] = [4.0, 5.0, 6.0]
assert reverse_parent.reverse_points(recurse=False) is reverse_parent
assert np.allclose(reverse_parent.get_points()[::2, 0], [1.0, 0.0])
assert np.allclose(reverse_child.get_points()[::2, 0], [3.0, 2.0]), (
    reverse_child.get_points()
)
assert np.allclose(reverse_parent.data["base_normal"][1], [0.0, 0.0, -1.0])
assert np.allclose(reverse_child.data["base_normal"][1], [0.0, 0.0, 1.0])
assert np.allclose(reverse_parent.data["stroke_width"][:, 0], [3.0, 2.0, 1.0])
assert np.allclose(reverse_child.data["stroke_width"][:, 0], [6.0, 5.0, 4.0])

# The scene-bound branch must expose the same two-phase semantics through one
# shared native Stage, not merely through the detached Python fallback above.
bound_reverse_scene = Scene()
bound_reverse_parent = VMobject().set_points_as_corners(
    [[0.0, 1.0, 0.0], [1.0, 1.0, 0.0]]
)
bound_reverse_child = VMobject().set_points_as_corners(
    [[2.0, 1.0, 0.0], [3.0, 1.0, 0.0]]
)
bound_reverse_parent.add(bound_reverse_child)
bound_reverse_parent.data["base_normal"][:] = [0.0, 0.0, 1.0]
bound_reverse_child.data["base_normal"][:] = [0.0, 0.0, 1.0]
bound_reverse_scene.add(bound_reverse_parent)
bound_reverse_parent.reverse_points(recurse=False)
assert np.allclose(bound_reverse_parent.get_points()[::2, 0], [1.0, 0.0])
assert np.allclose(bound_reverse_child.get_points()[::2, 0], [3.0, 2.0])
assert np.allclose(
    bound_reverse_parent.data["base_normal"][1], [0.0, 0.0, -1.0]
)
assert np.allclose(
    bound_reverse_child.data["base_normal"][1], [0.0, 0.0, 1.0]
)

# Marionette's production partial-reveal operation is the one portal
# VMobjects use too. A subcurve remains a full-size, independent record copy:
# only its point/joint-angle lanes change, while gradient/style data survives.
partial_source = VMobject().set_points(
    np.array(
        [
            [0.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, -1.0, 0.0],
            [3.0, 0.0, 0.0],
        ],
        dtype=np.float32,
    )
)
partial_source.data["stroke_width"][:, 0] = np.linspace(1.0, 5.0, 5)
partial_source.data["fill_rgba"][:] = np.linspace(
    [1.0, 0.0, 0.0, 0.2], [0.0, 0.0, 1.0, 0.8], 5
)
partial_style = {
    "stroke_width": partial_source.data["stroke_width"].copy(),
    "fill_rgba": partial_source.data["fill_rgba"].copy(),
}
partial_start = partial_source.quick_point_from_proportion(0.25)
partial_end = partial_source.quick_point_from_proportion(0.75)
partial_copy = partial_source.get_subcurve(0.25, 0.75)
assert partial_copy is not partial_source
assert partial_copy.get_num_points() == partial_source.get_num_points()
assert np.allclose(partial_copy.get_points()[0], partial_start)
assert np.allclose(partial_copy.get_points()[-1], partial_end)
assert np.array_equal(partial_copy.data["stroke_width"], partial_style["stroke_width"])
assert np.array_equal(partial_copy.data["fill_rgba"], partial_style["fill_rgba"])
partial_copy.get_points()[0] = [-9.0, -9.0, 0.0]
assert not np.allclose(partial_source.get_points()[0], partial_copy.get_points()[0])

full_partial = VMobject().set_points_as_corners(
    [[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]]
)
assert full_partial.pointwise_become_partial(partial_source, 0.0, 1.0) is full_partial
assert np.allclose(full_partial.get_points(), partial_source.get_points())
same_partial = partial_source.copy()
assert same_partial.pointwise_become_partial(same_partial, 0.1, 0.9) is same_partial
try:
    partial_source.pointwise_become_partial(Mobject(), 0.0, 1.0)
except AssertionError as error:
    assert "VMobject" in str(error)
else:
    raise AssertionError("partial slicing accepted a non-VMobject source")
try:
    partial_source.get_subcurve(float("nan"), 0.5)
except ValueError as error:
    assert "finite" in str(error)
else:
    raise AssertionError("partial slicing accepted a non-finite bound")

partial_scene = Scene()
bound_partial_source = partial_source.copy()
bound_partial_target = partial_source.copy()
partial_scene.add(bound_partial_source, bound_partial_target)
bound_start = bound_partial_source.quick_point_from_proportion(0.125)
bound_end = bound_partial_source.quick_point_from_proportion(0.625)
assert (
    bound_partial_target.pointwise_become_partial(
        bound_partial_source, 0.125, 0.625
    )
    is bound_partial_target
)
assert np.allclose(bound_partial_target.get_points()[0], bound_start)
assert np.allclose(bound_partial_target.get_points()[-1], bound_end)

vectorized = importlib.import_module("manimlib.mobject.types.vectorized_mobject")
assert vectorized.VectorizedPoint is manimlib.VectorizedPoint
assert vectorized.CurvesAsSubmobjects is manimlib.CurvesAsSubmobjects
assert vectorized.DashedVMobject is manimlib.DashedVMobject
assert vectorized.VHighlight is manimlib.VHighlight
assert vectorized.VectorizedPoint.__mro__[:4] == (
    vectorized.VectorizedPoint,
    manimlib.Point,
    VMobject,
    Mobject,
)
# The Reference's VGroup is VGroup(Group, VMobject) (d0b424f5 restored it).
assert vectorized.CurvesAsSubmobjects.__mro__[:4] == (
    vectorized.CurvesAsSubmobjects,
    manimlib.VGroup,
    manimlib.Group,
    VMobject,
)
assert list(inspect.signature(vectorized.VectorizedPoint).parameters) == [
    "location",
    "color",
    "fill_opacity",
    "stroke_width",
    "kwargs",
]
assert list(inspect.signature(vectorized.CurvesAsSubmobjects).parameters) == [
    "vmobject",
    "kwargs",
]
assert list(inspect.signature(vectorized.DashedVMobject).parameters) == [
    "vmobject",
    "num_dashes",
    "positive_space_ratio",
    "kwargs",
]
assert list(inspect.signature(vectorized.VHighlight).parameters) == [
    "vmobject",
    "n_layers",
    "color_bounds",
    "max_stroke_addition",
]
assert list(inspect.signature(VMobject.get_subcurve).parameters) == ["self", "a", "b"]
assert list(inspect.signature(VMobject.pointwise_become_partial).parameters) == [
    "self",
    "vmobject",
    "a",
    "b",
]

vectorized_point = vectorized.VectorizedPoint(
    [1.5, -2.0, 0.25], color=manimlib.RED
)
assert vectorized_point.get_num_points() == 1
assert np.allclose(vectorized_point.get_location(), [1.5, -2.0, 0.25])
assert np.allclose(vectorized_point.get_start(), vectorized_point.get_end())
assert vectorized_point.get_stroke_width() == 0.0
assert vectorized_point.get_fill_opacity() == 0.0
# fm-78b6: VMobject's stroke opacity default, as the pinned Reference reports.
assert vectorized_point.get_stroke_opacity() == 1.0
vectorized_point.shift([0.5, 1.0, -0.25])
assert np.allclose(vectorized_point.get_location(), [2.0, -1.0, 0.0])

curve_parts = vectorized.CurvesAsSubmobjects(partial_source)
assert len(curve_parts) == partial_source.get_num_curves()
assert all(type(part) is VMobject for part in curve_parts)
assert all(part.get_num_points() == 3 for part in curve_parts)
assert all(part.get_stroke_width() == partial_source.get_stroke_width() for part in curve_parts)

# BN-03: four dashes on a path whose two curves have very different lengths
# still measure equally. Each child is a full native partial copy, not a
# Python-reconstructed path, and the root retains the source's own style.
uneven_dash_source = VMobject(stroke_color=manimlib.BLUE, stroke_width=3.0).set_points(
    np.array(
        [
            [0.0, 0.0, 0.0],
            [0.05, 0.0, 0.0],
            [0.1, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [6.0, 0.0, 0.0],
        ]
    )
)
dashed = vectorized.DashedVMobject(
    uneven_dash_source, num_dashes=4, positive_space_ratio=0.5
)
assert len(dashed) == 4
dash_lengths = [dash.get_arc_length() for dash in dashed]
assert np.allclose(dash_lengths, dash_lengths[0], rtol=0.0, atol=1e-6), dash_lengths
assert dashed.get_stroke_color() == manimlib.BLUE
assert all(dash.get_stroke_width() == 3.0 for dash in dashed)
assert len(vectorized.DashedVMobject(uneven_dash_source, num_dashes=0)) == 0
try:
    vectorized.DashedVMobject(uneven_dash_source, num_dashes=4097)
except ValueError as error:
    assert "4096" in str(error)
else:
    raise AssertionError("DashedVMobject exceeded the native child budget")
try:
    vectorized.DashedVMobject(
        uneven_dash_source, num_dashes=4, positive_space_ratio=0.0
    )
except ValueError as error:
    assert "positive-space ratio" in str(error)
else:
    raise AssertionError("DashedVMobject accepted a zero drawn ratio")

highlight_source = manimlib.VGroup(
    manimlib.Circle(radius=0.4, stroke_width=2.0, fill_opacity=0.5),
    manimlib.Square(side_length=0.5, stroke_width=3.0, fill_opacity=0.75),
)
highlight = vectorized.VHighlight(highlight_source, n_layers=5)
assert len(highlight) == 5
assert all(len(layer) == 2 for layer in highlight)
assert all(
    member.get_fill_opacity() == 0.0
    for layer in highlight
    for member in layer.family_members_with_points()
)
assert highlight[0][0].get_stroke_width() == 7.0
assert highlight[-1][0].get_stroke_width() == 3.0
assert highlight[0][0].get_stroke_color() == manimlib.GREY_E
assert highlight[-1][0].get_stroke_color() == manimlib.GREY_C
assert len(vectorized.VHighlight(highlight_source, n_layers=0)) == 0

# VMobject topology is read directly from the writable shared-anchor record
# view, while insertion delegates its distribution/subdivision work to
# Chisel and commits through the ordinary RecordBuffer point-write seam.
topology = VMobject().set_points(
    np.array(
        [
            [0.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [3.5, -1.0, 0.0],
            [4.0, 0.0, 0.0],
        ]
    )
)
anchors_and_handles = topology.get_anchors_and_handles()
assert len(anchors_and_handles) == 3
assert all(np.shares_memory(part, topology.get_points()) for part in anchors_and_handles)
assert np.allclose(topology.get_start_anchors(), topology.get_points()[0:-1:2])
assert np.allclose(topology.get_end_anchors(), topology.get_points()[2::2])
live_anchors = topology.get_anchors()
assert np.shares_memory(live_anchors, topology.get_points())
live_anchors[0] = [-1.0, 0.0, 0.0]
assert np.allclose(topology.get_points()[0], [-1.0, 0.0, 0.0])
bezier_tuples = list(topology.get_bezier_tuples())
assert len(bezier_tuples) == 3
assert all(np.shares_memory(curve, topology.get_points()) for curve in bezier_tuples)
assert np.array_equal(topology.get_subpath_end_indices(), [2, 6])
subpaths = topology.get_subpaths()
assert [len(path) for path in subpaths] == [3, 3]
assert all(np.shares_memory(path, topology.get_points()) for path in subpaths)
assert np.array_equal(
    topology.get_subpath_end_indices_from_points(topology.get_points()),
    [2, 6],
)
assert len(topology.get_subpaths_from_points(np.zeros((0, 3)))) == 0
# Appendix C-23: the Reference's match_points keeps a cached subpath-end list, so
# after matching a shorter path its ends index past the points (its next joint
# refresh raises IndexError). The portal reads the ends from the live points.
rematched = topology.copy()
assert np.array_equal(rematched.get_subpath_end_indices(), [2, 6])
rematched.match_points(VMobject().set_points_as_corners([[0, 0, 0], [1, 0, 0], [1, 1, 0]]))
assert np.array_equal(rematched.get_subpath_end_indices(), [4])
# Appendix C-24: a partial background_line_style keeps the default stroke color,
# where the Reference raises KeyError: 'stroke_color'.
partial_plane = manimlib.NumberPlane(background_line_style={"stroke_width": 3})
assert partial_plane.background_lines[0].get_stroke_color().upper() == "#29ABCA"
assert partial_plane.background_lines[0].get_stroke_width() == 3
# Appendix C-26: a proportion along a zero-length path is its start point, where
# the Reference's point_from_proportion asserts.
flat_arc = manimlib.Arc(angle=0, radius=0.5)
assert np.allclose(flat_arc.pfp(0.3), flat_arc.get_start())
# Appendix C-29: restore() brings back a point-less member's style too. The
# Reference's become() copies point data only, so a faded Integer root stays
# faded after restore while its glyphs come back.
saved_bits = manimlib.VGroup(manimlib.Integer(0), manimlib.Integer(1))
saved_bits[1].set_opacity(0)
saved_bits.save_state()
saved_bits.set_fill(opacity=0)
saved_bits.restore()
assert [m.get_fill_opacity() for m in saved_bits] == [1.0, 0.0]
assert saved_bits[0][0].get_fill_opacity() == 1.0

single_insert = topology.insert_n_curves_to_point_list(
    2, np.array([[2.0, 3.0, 0.0]], dtype=np.float32)
)
assert single_insert.dtype == np.float64
assert single_insert.shape == (5, 3)
assert np.allclose(single_insert, [[2.0, 3.0, 0.0]] * 5)

insertion = VMobject().set_points(
    np.array(
        [
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.25, 0.5, 0.0],
            [2.5, 0.0, 0.0],
        ]
    )
)
insertion.data["stroke_width"][:, 0] = [1.0, 2.0, 3.0, 4.0, 5.0]
insertion_old_view = insertion.get_points()
insertion_arc_length = insertion.get_arc_length()
assert insertion.insert_n_curves(2) is insertion
assert insertion.get_num_curves() == 4
assert insertion_old_view.shape == (5, 3)
insertion_old_view[0] = [99.0, 99.0, 99.0]
assert not np.allclose(insertion.get_points()[0], insertion_old_view[0])
assert np.allclose(
    insertion.data["stroke_width"][:, 0],
    [1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0],
)
assert math.isclose(
    insertion.get_arc_length(), insertion_arc_length, rel_tol=0.0, abs_tol=2e-7
)
assert np.isfinite(insertion.data["joint_angle"]).all()

same_length_view = insertion.get_points()
insertion.insert_n_curves(0)
same_length_view[0] = [-2.0, 0.0, 0.0]
assert np.allclose(insertion.get_points()[0], [-2.0, 0.0, 0.0])

family_curve_a = VMobject().set_points_as_corners([[0, 0, 0], [1, 0, 0]])
family_curve_b = VMobject().set_points_as_corners([[0, 1, 0], [2, 1, 0]])
curve_family = manimlib.VGroup(family_curve_a, family_curve_b)
curve_family.insert_n_curves(1, recurse=False)
assert [mob.get_num_curves() for mob in curve_family] == [1, 1]
curve_family.insert_n_curves(1)
assert [mob.get_num_curves() for mob in curve_family] == [2, 2]

bound_insertion = VMobject().set_points_as_corners([[0, 0, 0], [3, 0, 0]])
bound_insertion_scene = Scene()
bound_insertion_scene.add(bound_insertion)
bound_insertion.insert_n_curves(2)
assert bound_insertion.get_num_curves() == 3

insertion_before_refusal = insertion.get_points().copy()
try:
    insertion.insert_n_curves_to_point_list(1, np.zeros((2, 3)))
except ValueError as error:
    assert "odd" in str(error).lower() or "even" in str(error).lower()
else:
    raise AssertionError("an even shared-anchor run reached Chisel insertion")
try:
    insertion.insert_n_curves(sys.maxsize)
except ValueError as error:
    assert "above the 65536 cap" in str(error).lower()
else:
    raise AssertionError("an over-budget curve insertion mutated the portal")
assert np.array_equal(insertion.get_points(), insertion_before_refusal)

# Alignment, whole-record append, and sharp subdivision reuse the existing
# Marionette/Chisel authorities rather than schema-generated placeholders.
# Appending copies every source lane onto the exact appended topology tail.
append_source = VMobject().set_points_as_corners(
    [[2.0, 1.0, 0.0], [3.0, 2.0, 0.0]]
)
append_source.data["stroke_width"][:, 0] = [7.0, 8.0, 9.0]
append_source.data["stroke_rgba"][:] = [0.2, 0.4, 0.6, 0.8]
append_source.data["fill_rgba"][:] = [0.7, 0.5, 0.3, 0.1]
append_target = VMobject().set_points_as_corners(
    [[-2.0, -1.0, 0.0], [-1.0, -1.0, 0.0]]
)
append_old_view = append_target.get_points()
assert append_target.append_vectorized_mobject(append_source) is append_target
assert append_target.get_num_points() == 7
assert np.array_equal(
    append_target.data[-append_source.get_num_points() :], append_source.data
)
assert append_old_view.shape == (3, 3)
append_old_view[0] = [99.0, 99.0, 99.0]
assert not np.allclose(append_target.get_points()[0], append_old_view[0])

# Detached alignment mutates both endpoints, preserves every non-geometry
# lane by proportional record resize, and invalidates only resized views.
align_small = VMobject().set_points(
    [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]]
)
align_large = VMobject().set_points_as_corners(
    [[0.0, 2.0, 0.0], [1.0, 2.0, 0.0], [2.0, 2.0, 0.0], [3.0, 2.0, 0.0]]
)
align_small.data["stroke_width"][:, 0] = [1.0, 2.0, 3.0]
align_small_before = align_small.data.copy()
align_small_old_view = align_small.get_points()
assert align_small.align_points(align_large) is align_small
assert align_small.get_num_points() == align_large.get_num_points() == 7
assert np.allclose(align_small.get_points()[0], [0.0, 0.0, 0.0])
assert np.allclose(align_small.get_points()[-1], [2.0, 0.0, 0.0])
assert np.array_equal(
    align_small.data["stroke_width"],
    manimlib.resize_preserving_order(align_small_before, 7)["stroke_width"],
)
assert align_small_old_view.shape == (3, 3)
assert np.isfinite(align_small.data["joint_angle"]).all()
assert np.isfinite(align_large.data["joint_angle"]).all()

# The receiver's public tolerance controls which near-null curve may receive
# an inserted split. This is the parameterized native seam, not Python math.
def aligned_with_tolerance(tolerance):
    left = VMobject().set_points(
        [
            [0.0, 0.0, 0.0],
            [0.01, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.5, 0.0, 0.0],
            [11.0, 0.0, 0.0],
        ]
    )
    right = VMobject().set_points_as_corners(
        [[0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [2.0, 1.0, 0.0], [3.0, 1.0, 0.0]]
    )
    left.tolerance_for_point_equality = tolerance
    left.align_points(right)
    return left.get_points().copy()


default_tolerance_alignment = aligned_with_tolerance(1e-8)
wide_tolerance_alignment = aligned_with_tolerance(0.1)
assert not np.array_equal(default_tolerance_alignment, wide_tolerance_alignment)
assert np.allclose(
    wide_tolerance_alignment[:3],
    [[0.0, 0.0, 0.0], [0.01, 0.0, 0.0], [10.0, 0.0, 0.0]],
)

# Same-Scene alignment uses the shared Stage directly; cross-Scene alignment
# uses native temporary entries and commits each proven point run back to its
# own live Stage. Both retain the public operation's scene independence.
bound_align_scene = Scene()
bound_align_small = VMobject().set_points_as_corners(
    [[0.0, 3.0, 0.0], [1.0, 3.0, 0.0]]
)
bound_align_large = VMobject().set_points_as_corners(
    [[0.0, 4.0, 0.0], [1.0, 4.0, 0.0], [2.0, 4.0, 0.0]]
)
bound_align_scene.add(bound_align_small, bound_align_large)
bound_align_small.align_points(bound_align_large)
assert bound_align_small.get_num_points() == bound_align_large.get_num_points() == 5

cross_align_left = VMobject().set_points_as_corners(
    [[0.0, 5.0, 0.0], [1.0, 5.0, 0.0]]
)
cross_align_right = VMobject().set_points_as_corners(
    [[0.0, 6.0, 0.0], [1.0, 6.0, 0.0], [2.0, 6.0, 0.0]]
)
cross_align_left_scene = Scene().add(cross_align_left)
cross_align_right_scene = Scene().add(cross_align_right)
cross_align_left.align_points(cross_align_right)
assert cross_align_left.get_num_points() == cross_align_right.get_num_points() == 5
assert cross_align_left in cross_align_left_scene.mobjects
assert cross_align_right in cross_align_right_scene.mobjects

align_refusal = VMobject().set_points_as_corners(
    [[0.0, 7.0, 0.0], [1.0, 7.0, 0.0]]
)
align_refusal_before = align_refusal.data.copy()
try:
    align_refusal.align_points(Mobject().set_points([[0.0, 0.0, 0.0]]))
except RuntimeError as error:
    assert "schema" in str(error).lower()
else:
    raise AssertionError("VMobject alignment accepted a base-record peer")
assert np.array_equal(align_refusal.data, align_refusal_before)

sharp = VMobject().set_points(
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0]]
)
sharp.data["stroke_width"][:, 0] = [2.0, 4.0, 6.0]
sharp_before = sharp.data.copy()
sharp_old_view = sharp.get_points()
sharp_length = sharp.get_arc_length()
assert sharp.subdivide_sharp_curves(30 * manimlib.DEG, recurse=False) is sharp
assert sharp.get_num_curves() == 4
assert sharp_old_view.shape == (3, 3)
assert np.allclose(sharp.get_points()[0], [0.0, 0.0, 0.0])
assert np.allclose(sharp.get_points()[-1], [1.0, 1.0, 0.0])
assert np.array_equal(
    sharp.data["stroke_width"],
    manimlib.resize_preserving_order(sharp_before, 9)["stroke_width"],
)
assert math.isclose(sharp.get_arc_length(), sharp_length, rel_tol=0.0, abs_tol=2e-7)

sharp_parent = VMobject().set_points(
    [[0.0, 2.0, 0.0], [1.0, 2.0, 0.0], [1.0, 3.0, 0.0]]
)
sharp_child = VMobject().set_points(
    [[2.0, 2.0, 0.0], [3.0, 2.0, 0.0], [3.0, 3.0, 0.0]]
)
sharp_parent.add(sharp_child)
sharp_parent.subdivide_sharp_curves(recurse=False)
assert sharp_parent.get_num_curves() == 4
assert sharp_child.get_num_curves() == 1
sharp_parent.subdivide_sharp_curves(recurse=True)
assert sharp_child.get_num_curves() == 4

# Plan-before-commit matters when a later descendant breaches the native
# subdivision budget: an earlier valid family member must remain unchanged.
atomic_parent = VMobject().set_points_as_corners(
    [[0.0, 8.0, 0.0], [1.0, 8.0, 0.0]]
)
atomic_child = VMobject().set_points(
    [[2.0, 8.0, 0.0], [3.0, 8.0, 0.0], [3.0, 9.0, 0.0]]
)
atomic_parent.add(atomic_child)
atomic_parent_before = atomic_parent.data.copy()
atomic_child_before = atomic_child.data.copy()
try:
    atomic_parent.subdivide_sharp_curves(float.fromhex("0x0.0000000000001p-1022"))
except ValueError as error:
    assert "65536" in str(error) or "subdivision" in str(error).lower()
else:
    raise AssertionError("sharp subdivision exceeded the native curve budget")
assert np.array_equal(atomic_parent.data, atomic_parent_before)
assert np.array_equal(atomic_child.data, atomic_child_before)

bound_sharp_scene = Scene()
bound_sharp = VMobject().set_points(
    [[-1.0, 5.0, 0.0], [0.0, 5.0, 0.0], [0.0, 6.0, 0.0]]
)
bound_sharp_scene.add(bound_sharp)
bound_sharp.subdivide_sharp_curves(recurse=False)
assert bound_sharp.get_num_curves() == 4

# Smooth construction and condition/intersection subdivision are the sibling
# Chisel-backed VMobject operations. Callback arguments retain the Reference's
# live float32 NumPy row shape, callbacks run exactly once per original curve,
# and the whole family is planned before the first RecordBuffer mutation.
smooth_points = np.array(
    [[-2.0, 0.0, 0.0], [-1.0, 1.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 0.0]]
)
for approx in (True, False):
    corner_points = VMobject().set_points_as_corners(smooth_points).get_points().copy()
    manual_smooth = VMobject().set_points_as_corners(smooth_points)
    manual_smooth.make_smooth(approx=approx)
    smooth = VMobject(stroke_width=7.0)
    assert smooth.set_points_smoothly(smooth_points, approx=approx) is smooth
    assert np.array_equal(smooth.get_points(), manual_smooth.get_points())
    assert np.allclose(smooth.data["stroke_width"], 7.0)
    assert not np.array_equal(smooth.get_points()[1::2], corner_points[1::2])

assert str(inspect.signature(VMobject.set_points_smoothly)) == (
    "(self, points, approx=True)"
)
assert str(inspect.signature(VMobject.subdivide_curves_by_condition)) == (
    "(self, tuple_to_subdivisions, recurse=True)"
)
assert str(inspect.signature(VMobject.subdivide_intersections)) == (
    "(self, recurse=True, n_subdivisions=1)"
)

conditional = VMobject().set_points(
    [
        [0.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.5, -1.0, 0.0],
        [2.0, 0.0, 0.0],
    ]
)
conditional.data["stroke_width"][:, 0] = [1.0, 2.0, 3.0, 4.0, 5.0]
conditional_before = conditional.data.copy()
conditional_length = conditional.get_arc_length()
callback_rows = []


def conditional_counts(b0, b1, b2):
    callback_rows.append((b0.copy(), b1.copy(), b2.copy()))
    assert all(isinstance(point, np.ndarray) for point in (b0, b1, b2))
    assert all(point.dtype == np.float32 for point in (b0, b1, b2))
    return np.int64(len(callback_rows))


assert (
    conditional.subdivide_curves_by_condition(conditional_counts, recurse=False)
    is conditional
)
assert len(callback_rows) == 2
assert conditional.get_num_curves() == 5
assert np.array_equal(
    conditional.data["stroke_width"],
    manimlib.resize_preserving_order(conditional_before, 11)["stroke_width"],
)
assert math.isclose(
    conditional.get_arc_length(), conditional_length, rel_tol=0.0, abs_tol=2e-7
)

condition_parent = VMobject().set_points_as_corners(
    [[0.0, 10.0, 0.0], [1.0, 10.0, 0.0]]
)
condition_child = VMobject().set_points_as_corners(
    [[2.0, 10.0, 0.0], [3.0, 10.0, 0.0]]
)
condition_parent.add(condition_child)
condition_parent.subdivide_curves_by_condition(lambda *curve: 1, recurse=False)
assert condition_parent.get_num_curves() == 2
assert condition_child.get_num_curves() == 1
condition_parent.subdivide_curves_by_condition(lambda *curve: 1)
assert condition_parent.get_num_curves() == 4
assert condition_child.get_num_curves() == 2

atomic_condition_parent = VMobject().set_points_as_corners(
    [[0.0, 11.0, 0.0], [1.0, 11.0, 0.0]]
)
atomic_condition_child = VMobject().set_points_as_corners(
    [[2.0, 11.0, 0.0], [3.0, 11.0, 0.0]]
)
atomic_condition_parent.add(atomic_condition_child)
atomic_condition_parent_before = atomic_condition_parent.data.copy()
atomic_condition_child_before = atomic_condition_child.data.copy()
atomic_callback_calls = 0


def failing_condition(*curve):
    global atomic_callback_calls
    atomic_callback_calls += 1
    if atomic_callback_calls == 2:
        raise LookupError("second family callback refusal")
    return 1


try:
    atomic_condition_parent.subdivide_curves_by_condition(failing_condition)
except LookupError as error:
    assert "second family callback refusal" in str(error)
else:
    raise AssertionError("a callback failure partially subdivided a family")
assert atomic_callback_calls == 2
assert np.array_equal(atomic_condition_parent.data, atomic_condition_parent_before)
assert np.array_equal(atomic_condition_child.data, atomic_condition_child_before)

for refused_count, exception in [(1.5, TypeError), (10**100, ValueError)]:
    refused = VMobject().set_points_as_corners(
        [[0.0, 12.0, 0.0], [1.0, 12.0, 0.0]]
    )
    refused_before = refused.data.copy()
    try:
        refused.subdivide_curves_by_condition(lambda *curve: refused_count)
    except exception:
        pass
    else:
        raise AssertionError(f"invalid subdivision count {refused_count!r} succeeded")
    assert np.array_equal(refused.data, refused_before)

negative_count = VMobject().set_points_as_corners(
    [[0.0, 13.0, 0.0], [1.0, 13.0, 0.0]]
)
negative_view = negative_count.get_points()
negative_count.subdivide_curves_by_condition(lambda *curve: -3)
negative_view[0] = [-1.0, 13.0, 0.0]
assert np.allclose(negative_count.get_points()[0], [-1.0, 13.0, 0.0])

intersection = VMobject().set_points(
    [
        [-2.0, -1.0, 0.0],
        [1.0, 1.0, 0.0],
        [-2.0, 1.0, 0.0],
        [-1.0, 2.0, 0.0],
        [2.0, -1.0, 0.0],
    ]
)
intersection_path = intersection.get_anchors().copy()
intersection_flags = [
    space_utils.line_intersects_path(b0, b1, intersection_path)
    for b0, b1, b2 in intersection.get_bezier_tuples()
]
assert any(intersection_flags)
intersection_curves = intersection.get_num_curves()
assert intersection.subdivide_intersections(False, 2) is intersection
assert intersection.get_num_curves() == intersection_curves + 2 * sum(
    intersection_flags
)

non_intersection = VMobject().set_points_as_corners(
    [[0.0, 15.0, 0.0], [1.0, 15.0, 0.0]]
)
non_intersection_view = non_intersection.get_points()
assert non_intersection.subdivide_intersections(False, 1.5) is non_intersection
non_intersection_view[0] = [-1.0, 15.0, 0.0]
assert np.allclose(non_intersection.get_points()[0], [-1.0, 15.0, 0.0])

for refused_count, exception in [(1.5, TypeError), (10**100, ValueError)]:
    refused_intersection = VMobject().set_points(
        [
            [-2.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-2.0, 1.0, 0.0],
            [-1.0, 2.0, 0.0],
            [2.0, -1.0, 0.0],
        ]
    )
    refused_intersection_before = refused_intersection.data.copy()
    try:
        refused_intersection.subdivide_intersections(False, refused_count)
    except exception:
        pass
    else:
        raise AssertionError(
            f"invalid intersection subdivision count {refused_count!r} succeeded"
        )
    assert np.array_equal(
        refused_intersection.data, refused_intersection_before
    )

bound_condition_scene = Scene()
bound_condition = VMobject().set_points_as_corners(
    [[-1.0, 14.0, 0.0], [1.0, 14.0, 0.0]]
)
bound_condition_scene.add(bound_condition)
bound_condition.subdivide_curves_by_condition(lambda *curve: 1, recurse=False)
assert bound_condition.get_num_curves() == 2
assert bound_condition in bound_condition_scene.mobjects

line.set_stroke(manimlib.BLACK, 3, background=True)
assert line.uniforms["stroke_behind"] is True
try:
    line.set_stroke(behind=True, background=False)
except TypeError as error:
    assert "conflicting behind/background" in str(error)
else:
    raise AssertionError("conflicting era-shim kwargs must be rejected")

# Tex and native text share the Reference's StringMobject selector surface.
# The implementation uses Scribe's retained UTF-8 source spans directly,
# preserving object identity without the Reference's render-twice SVG labels.
# Native text bypasses SVG parsing, not the inherited default centering.
for text_type, text_source in (
    (manimlib.Text, "café Hello"),
    (manimlib.MarkupText, "<b>Hello</b> world"),
    (manimlib.Tex, r"e^{i\pi} + 1 = 0"),
    (manimlib.TexText, "Hello"),
):
    centered = text_type(text_source, color="#00FF00")
    uncentered = text_type(text_source, should_center=False, color="#00FF00")
    origin = uncentered.get_center().copy()
    assert not np.allclose(origin, [0, 0, 0]), (text_type, origin)
    assert np.allclose(centered.get_center(), [0, 0, 0], atol=1e-7), text_type
    assert np.allclose(centered.get_all_points(), uncentered.get_all_points() - origin,
                       atol=1e-6), text_type
    assert centered.get_color() == uncentered.get_color(), text_type
    assert centered._string_sub_spans == uncentered._string_sub_spans, text_type
    assert centered._string_sub_paths == uncentered._string_sub_paths, text_type
    assert all(centered._string_submobject(i) in centered.get_family()
               for i in range(len(centered._string_sub_spans)))
    centered.scale(2)
    assert np.allclose(centered.get_center(), [0, 0, 0], atol=1e-7), text_type
for text_type in (manimlib.Text, manimlib.MarkupText):
    sized_text = text_type("Hello", height=2)
    assert np.allclose(sized_text.get_center(), [0, 0, 0], atol=1e-7)
    assert abs(sized_text.get_height() - 2) < 1e-6

string_module = importlib.import_module("manimlib.mobject.svg.string_mobject")
svg_module = importlib.import_module("manimlib.mobject.svg.svg_mobject")
text_module = importlib.import_module("manimlib.mobject.svg.text_mobject")
StringMobject = string_module.StringMobject
SVGMobject = svg_module.SVGMobject
assert issubclass(manimlib.Tex, StringMobject)
assert issubclass(manimlib.MarkupText, StringMobject)
assert issubclass(StringMobject, SVGMobject)
assert issubclass(SVGMobject, VMobject)
assert StringMobject.__bases__ == (SVGMobject, abc.ABC)
assert manimlib.Tex.__mro__[:4] == (
    manimlib.Tex,
    StringMobject,
    SVGMobject,
    VMobject,
)
assert manimlib.MarkupText.__mro__[:4] == (
    manimlib.MarkupText,
    StringMobject,
    SVGMobject,
    VMobject,
)
assert text_module._Alignment.VAL_DICT == {
    "LEFT": 0,
    "CENTER": 1,
    "RIGHT": 2,
}
assert str(inspect.signature(text_module._Alignment)) == "(s)"
assert text_module._Alignment("center").value == 1
try:
    text_module._Alignment("bogus")
except KeyError:
    pass
else:
    raise AssertionError("_Alignment accepted an unknown alignment token")

plain_text = manimlib.Text("café café")
assert isinstance(plain_text, StringMobject)
assert plain_text.get_string() == plain_text.get_text() == "café café"
assert plain_text.find_spans_by_selector("café") == [(0, 4), (5, 9)]
assert plain_text.find_spans_by_selector(re.compile(r"caf.")) == [(0, 4), (5, 9)]
assert plain_text.find_spans_by_selector((-4, None)) == [(5, 9)]
assert plain_text.find_spans_by_selector("") == [
    (index, index) for index in range(len(plain_text.string) + 1)
]
assert plain_text.find_spans_by_selector(["café", (5, None)]) == [
    (0, 4),
    (5, 9),
    (5, 9),
]
assert StringMobject.span_contains((0, 4), (1, 3))
assert not StringMobject.span_contains((1, 3), (0, 4))
plain_groups = plain_text.get_parts_by_text("café")
assert len(plain_groups) == 2
assert all(len(group) == 4 for group in plain_groups)
assert plain_groups[0][0] is plain_text.submobjects[0]
assert plain_text["café"][1][0] is plain_text.submobjects[4]
assert plain_text.get_part_by_text("café", index=1)[0] is plain_text.submobjects[4]
assert plain_text.get_submob_indices_list_by_span((0, 4)) == [0, 1, 2, 3]
assert plain_text.get_submob_indices_lists_by_selector("café") == [
    [0, 1, 2, 3],
    [4, 5, 6, 7],
]
assert len(plain_text.get_parts_by_text("absent")) == 0
assert plain_text.set_color_by_text("café", manimlib.BLUE) is plain_text
assert all(
    leaf.get_fill_color() == manimlib.BLUE
    for group in plain_text.get_parts_by_text("café")
    for leaf in group
)
assert plain_text.set_color_by_text_to_color_map(
    {re.compile(r"caf."): manimlib.YELLOW}
) is plain_text
assert all(
    leaf.get_fill_color() == manimlib.YELLOW
    for group in plain_text.get_parts_by_text("café")
    for leaf in group
)
try:
    plain_text.find_spans_by_selector(3.5)
except TypeError as error:
    assert "Invalid selector" in str(error)
else:
    raise AssertionError("StringMobject accepted an invalid selector")

markup_text = manimlib.MarkupText(
    "<b>β</b> + β",
    t2c={"β": manimlib.RED},
    isolate=re.compile(r"β"),
)
assert markup_text.string == markup_text.text == "<b>β</b> + β"
assert markup_text.isolate.pattern == r"β"
markup_betas = markup_text.get_parts_by_text("β")
assert len(markup_betas) == 2
assert all(len(group) == 1 for group in markup_betas)
assert all(
    group[0].get_fill_color() == manimlib.RED
    for group in markup_betas
)
assert markup_text.get_part_by_text("β", index=1)[0] is markup_text.submobjects[2]

# Public escaping is a one-token operation; callers can safely compose native
# markup from literal text without accidentally interpreting author content.
markup_entities = {
    "<": "&lt;", ">": "&gt;", "&": "&amp;", '"': "&quot;", "'": "&apos;",
}
assert manimlib.MarkupText.MARKUP_ENTITY_DICT == markup_entities
native_tagged_text = {
    tag: manimlib.MarkupText(f"<{tag}>x</{tag}>")
    for tag in manimlib.MarkupText.MARKUP_TAGS
}
assert all(mob.get_all_points().size > 0 for mob in native_tagged_text.values())
assert (native_tagged_text["big"].get_width()
        > manimlib.Text("x").get_width()
        > native_tagged_text["small"].get_width())
for char, entity in markup_entities.items():
    assert manimlib.MarkupText.escape_markup_char(char) == entity
    assert manimlib.MarkupText.unescape_markup_char(entity) == char
    assert manimlib.Text.escape_markup_char(char) == entity
for literal in ("β", "&unknown;", "&#65;", "ab", "<b>", "", "&lt;&gt;"):
    assert manimlib.MarkupText.escape_markup_char(literal) == literal
    assert manimlib.MarkupText.unescape_markup_char(literal) == literal
literal_markup_text = 'a < b & "c" > \'d\''
escaped_markup_text = "".join(map(manimlib.MarkupText.escape_markup_char, literal_markup_text))
native_literal_text = manimlib.Text(literal_markup_text)
native_escaped_text = manimlib.MarkupText(escaped_markup_text)
assert len(native_escaped_text) == len(native_literal_text) > 8
assert np.array_equal(native_escaped_text.get_all_points(), native_literal_text.get_all_points())

markup_signature = inspect.signature(manimlib.MarkupText)
assert markup_signature.parameters["font_size"].default == 48
assert markup_signature.parameters["text2color"].default == {}
assert markup_signature.parameters["t2c"].default == {}
assert markup_signature.parameters["isolate"].default.pattern == r"\w+"
text_signature = inspect.signature(manimlib.Text)
assert list(text_signature.parameters) == [
    "text",
    "isolate",
    "use_labelled_svg",
    "path_string_config",
    "kwargs",
]
assert text_signature.parameters["use_labelled_svg"].default is True
assert text_signature.parameters["path_string_config"].default == {
    "use_simple_quadratic_approx": True
}

assert text_module.Code.__bases__ == (text_module.MarkupText,)
assert str(inspect.signature(text_module.Code)) == (
    "(code, font='Consolas', font_size=24, lsh=1.0, fill_color=None, "
    "stroke_color=None, language='python', code_style='monokai', **kwargs)"
)
native_code = text_module.Code("x = 1")
assert native_code.get_text() == "x = 1"
assert native_code.code == "x = 1"
assert native_code.font == "Consolas"
assert native_code.font_size == 24
assert native_code.lsh == 1.0
assert native_code.language == "python"
assert native_code.code_style == "monokai"
assert native_code.get_num_points() == 0  # One live glyph family, no duplicate root ink.
assert len(native_code.get_all_points()) > 0
assert native_code.native_font == "CM Typewriter"
rust_code = text_module.Code("fn", language="rust")
assert all(part.get_fill_color() == "#68C246" for part in rust_code.family_members_with_points())
colored_code = text_module.Code("y", fill_color=manimlib.GREEN)
assert colored_code.get_fill_color() == manimlib.GREEN
code_scene = Scene().add(native_code)
assert native_code._is_bound()
failed_code = text_module.Code.__new__(text_module.Code)
try:
    text_module.Code.__init__(failed_code, "x", language=object())
except TypeError as error:
    assert str(error) == "Code language and code_style must be strings"
else:
    raise AssertionError("Code silently accepted an invalid language declaration")
assert not hasattr(failed_code, "submobjects")

tex = manimlib.Tex("E = mc^2", isolate=["mc"])
tex_signature = inspect.signature(manimlib.Tex)
assert tex_signature.parameters["tex_strings"].annotation == "str"
assert tex_signature.parameters["font_size"].default == 48
assert tex_signature.parameters["tex_to_color_map"].default == {}
assert tex_signature.parameters["t2c"].default == {}
assert tex_signature.parameters["isolate"].default == []
assert tex.string == tex.tex_string == "E = mc^2"
assert tex.alignment == r"\centering"
assert tex.template == ""
assert tex.additional_preamble == ""
assert tex.tex_to_color_map == {}
assert tex.isolate == ["mc"]
assert tex.tex_environment == "align*"
mc_parts = tex.get_parts_by_tex("mc")
assert len(mc_parts) == 1 and len(mc_parts[0]) == 2
assert tex.get_part_by_tex(re.compile(r"m.")) is not None
assert len(tex.get_parts_by_tex((0, 1))) == 1
assert len(tex.get_parts_by_tex("not present")) == 0
tex.set_color_by_tex("mc", manimlib.BLUE)
assert all(
    leaf.get_fill_color() == manimlib.BLUE
    for leaf in tex.get_part_by_tex("mc")
)
tex.set_color_by_tex_to_color_map({re.compile(r"E"): manimlib.RED})
assert all(
    leaf.get_fill_color() == manimlib.RED
    for leaf in tex.get_part_by_tex("E")
)
assert tex.get_tex() == "E = mc^2"

tex.scale(0.5)
assert math.isclose(tex.font_size, 24.0)

multi_tex = manimlib.Tex("x", "+", "y", isolate="=")
assert multi_tex.get_tex() == "x + y"
assert multi_tex.string == "x + y"
assert multi_tex.isolate == ["=", "x", "+", "y"]
assert len(multi_tex.get_parts_by_tex(["x", "y"])) == 2

trimmed_tex = manimlib.Tex("  x  ")
assert trimmed_tex.get_tex() == "x"
assert trimmed_tex.isolate == []
assert manimlib.Tex("").get_tex() == r"\\"

unicode_tex = manimlib.Tex("α+α")
unicode_parts = unicode_tex.get_parts_by_tex("α")
assert len(unicode_parts) == 2
unicode_tex.set_color_by_tex("α", manimlib.YELLOW)
assert all(
    leaf.get_fill_color() == manimlib.YELLOW
    for group in unicode_parts
    for leaf in group
)

tex_text = manimlib.TexText("words")
assert tex_text.tex_environment == ""

mapped_tex = manimlib.Tex(
    "x+y",
    t2c={"x": manimlib.RED},
    tex_to_color_map={"x": manimlib.BLUE},
)
assert mapped_tex.tex_to_color_map == {"x": manimlib.BLUE}
assert all(
    leaf.get_fill_color() == manimlib.BLUE
    for leaf in mapped_tex.get_part_by_tex("x")
)

# A paragraph alignment cannot move the Reference's align* display, so math
# mode accepts it with no geometric effect (the TexText surface aligns).
assert np.array_equal(
    manimlib.Tex("x", alignment=r"\raggedright").get_all_points(),
    manimlib.Tex("x").get_all_points(),
)
for unsupported_tex_kwargs, named_knob, error_type in [
    ({"template": "legacy"}, "template", ValueError),
    ({"additional_preamble": r"\usepackage{foo}"}, "additional_preamble", ValueError),
]:
    try:
        manimlib.Tex("x", **unsupported_tex_kwargs)
    except error_type as error:
        assert named_knob in str(error)
    else:
        raise AssertionError(f"unsupported Tex {named_knob} silently succeeded")


class ReadmeHello(Scene):
    def construct(self):
        title = manimlib.Text("FrankenManim", font_size=72)
        formula = manimlib.Tex(r"e^{i\pi} + 1 = 0")
        formula.next_to(title, manimlib.DOWN)
        self.play(
            manimlib.Write(title),
            manimlib.FadeIn(formula, shift=manimlib.UP),
        )
        self.play(formula.animate.set_color_by_tex("i", manimlib.YELLOW))
        self.wait()
        self.formula = formula


readme_hello = ReadmeHello()
readme_hello.run()
assert math.isclose(readme_hello.time, 3.0)
readme_i_parts = readme_hello.formula.get_parts_by_tex("i")
assert len(readme_i_parts) == 1
assert all(
    leaf.get_fill_color() == manimlib.YELLOW
    for group in readme_i_parts
    for leaf in group
)

changeable = manimlib.Tex("x = 0.00").make_number_changeable("0.00")
assert isinstance(changeable, manimlib.DecimalNumber)
changeable.set_value(1.25)
assert math.isclose(changeable.get_value(), 1.25)

repeated_tex = manimlib.Tex("x", "+", "x")
assert len(repeated_tex.get_parts_by_tex("x")) == 2

functions = importlib.import_module("manimlib.mobject.functions")
curve = functions.ParametricCurve(
    lambda t: np.array([t, t * t, 0.0]),
    t_range=(0.0, 1.0, 0.25),
)
parametric_curve_signature = inspect.signature(functions.ParametricCurve)
assert tuple(parametric_curve_signature.parameters) == (
    "t_func",
    "t_range",
    "epsilon",
    "discontinuities",
    "use_smoothing",
    "kwargs",
)
assert parametric_curve_signature.parameters["t_range"].default == (0, 1, 0.1)
assert parametric_curve_signature.parameters["epsilon"].default == 1e-8
assert parametric_curve_signature.parameters["discontinuities"].default == []
assert parametric_curve_signature.parameters["use_smoothing"].default is True
assert curve.has_points()
assert Mobject().get_num_points() == 0
assert curve.get_num_points() == len(curve.get_points())
assert curve.get_num_curves() == curve.get_num_points() // 2
assert np.allclose(curve.get_point_from_function(0.5), [0.5, 0.25, 0.0])
assert curve.get_t_func() is curve.t_func
assert curve.get_arc_length() > math.sqrt(2.0)

# FunctionGraph and ImplicitFunction are authored portal classes over Atlas's
# already-native graph builders, not schema-generated import shells. Scalar
# callbacks are construction-only, metadata remains Reference-visible, and
# Chisel owns the bounded zero-set extraction.
assert functions.FunctionGraph.__bases__ == (functions.ParametricCurve,)
assert functions.ImplicitFunction.__bases__ == (VMobject,)
function_graph_signature = inspect.signature(functions.FunctionGraph)
assert tuple(function_graph_signature.parameters) == (
    "function",
    "x_range",
    "color",
    "kwargs",
)
assert function_graph_signature.parameters["x_range"].default == (-8, 8, 0.25)
assert function_graph_signature.parameters["color"].default == manimlib.YELLOW
implicit_signature = inspect.signature(functions.ImplicitFunction)
assert tuple(implicit_signature.parameters) == (
    "func",
    "x_range",
    "y_range",
    "min_depth",
    "max_quads",
    "use_smoothing",
    "joint_type",
    "kwargs",
)
assert np.allclose(
    implicit_signature.parameters["x_range"].default,
    (-manimlib.FRAME_X_RADIUS, manimlib.FRAME_X_RADIUS),
)
assert np.allclose(
    implicit_signature.parameters["y_range"].default,
    (-manimlib.FRAME_Y_RADIUS, manimlib.FRAME_Y_RADIUS),
)
assert implicit_signature.parameters["min_depth"].default == 5
assert implicit_signature.parameters["max_quads"].default == 1500
assert implicit_signature.parameters["use_smoothing"].default is False
assert implicit_signature.parameters["joint_type"].default == "no_joint"

function_graph_samples = []


def portal_parabola(x):
    function_graph_samples.append(float(x))
    return x * x - 0.25


function_graph = functions.FunctionGraph(
    portal_parabola,
    x_range=(-1.0, 1.0, 0.25),
    color=manimlib.RED,
    epsilon=1e-6,
    discontinuities=(0.0,),
    use_smoothing=False,
    stroke_width=3.0,
)
assert function_graph.function is portal_parabola
assert function_graph.get_function() is portal_parabola
assert function_graph.get_t_func() is function_graph.t_func
assert function_graph.get_x_range() == (-1.0, 1.0, 0.25)
assert function_graph.t_range == function_graph.x_range
assert function_graph.epsilon == 1e-6
assert function_graph.discontinuities == (0.0,)
assert function_graph.use_smoothing is False
assert function_graph.get_stroke_color() == manimlib.RED
assert math.isclose(function_graph.get_stroke_width(), 3.0)
assert np.allclose(function_graph.get_start(), [-1.0, 0.75, 0.0])
assert np.allclose(function_graph.get_end(), [1.0, 0.75, 0.0])
assert function_graph_samples[0] == -1.0
assert function_graph_samples[-1] == 1.0
assert any(np.isclose(value, -1e-6) for value in function_graph_samples)
assert any(np.isclose(value, 1e-6) for value in function_graph_samples)
assert np.allclose(
    function_graph.get_point_from_function(0.5),
    [0.5, 0.0, 0.0],
)

default_function_graph = functions.FunctionGraph(
    lambda x: x,
    x_range=(0.0, 1.0, 0.5),
    use_smoothing=False,
)
assert default_function_graph.get_stroke_color() == manimlib.YELLOW

implicit_calls = []


def portal_circle_field(x, y):
    implicit_calls.append((float(x), float(y)))
    return x * x + y * y - 1.0


implicit_circle = functions.ImplicitFunction(
    portal_circle_field,
    x_range=(-1.5, 1.5),
    y_range=(-1.5, 1.5),
    min_depth=3,
    max_quads=512,
    use_smoothing=False,
    joint_type="bevel",
    stroke_color=manimlib.BLUE,
    stroke_width=2.5,
)
assert implicit_circle.func is portal_circle_field
assert implicit_circle.x_range == (-1.5, 1.5)
assert implicit_circle.y_range == (-1.5, 1.5)
assert implicit_circle.min_depth == 3
assert implicit_circle.max_quads == 512
assert implicit_circle.use_smoothing is False
assert implicit_circle.joint_type == "bevel"
assert implicit_circle.get_joint_type() == VMobject.joint_type_map["bevel"]
assert implicit_circle.get_stroke_color() == manimlib.BLUE
assert math.isclose(implicit_circle.get_stroke_width(), 2.5)
assert implicit_circle.has_points()
assert implicit_calls
implicit_box = implicit_circle.get_bounding_box()
assert np.allclose(implicit_box[0, :2], [-1.0, -1.0], atol=2e-2)
assert np.allclose(implicit_box[2, :2], [1.0, 1.0], atol=2e-2)

empty_implicit = functions.ImplicitFunction(
    lambda x, y: x * x + y * y + 1.0,
    x_range=(-1.0, 1.0),
    y_range=(-1.0, 1.0),
    min_depth=2,
    max_quads=64,
)
assert not empty_implicit.has_points()


class PortalGraphCallbackError(RuntimeError):
    pass


def refuse_graph_callback(_x):
    raise PortalGraphCallbackError("function graph callback sentinel")


try:
    functions.FunctionGraph(
        refuse_graph_callback,
        x_range=(0.0, 1.0, 0.5),
        use_smoothing=False,
    )
except PortalGraphCallbackError as error:
    assert str(error) == "function graph callback sentinel"
else:
    raise AssertionError("FunctionGraph swallowed its Python callback error")


def refuse_implicit_callback(_x, _y):
    raise PortalGraphCallbackError("implicit callback sentinel")


try:
    functions.ImplicitFunction(
        refuse_implicit_callback,
        x_range=(-1.0, 1.0),
        y_range=(-1.0, 1.0),
        min_depth=1,
        max_quads=16,
    )
except PortalGraphCallbackError as error:
    assert str(error) == "implicit callback sentinel"
else:
    raise AssertionError("ImplicitFunction swallowed its Python callback error")

preflight_graph_calls = []
for invalid_kwargs, invalid_name in (
    ({"unknown_style": True}, "unknown_style"),
    ({"joint_type": "not-a-joint"}, "not-a-joint"),
):
    try:
        functions.ImplicitFunction(
            lambda x, y: preflight_graph_calls.append((x, y)) or x + y,
            x_range=(-1.0, 1.0),
            y_range=(-1.0, 1.0),
            **invalid_kwargs,
        )
    except (TypeError, ValueError) as error:
        assert invalid_name in str(error)
    else:
        raise AssertionError(f"ImplicitFunction accepted {invalid_name}")
assert preflight_graph_calls == []

# CoordinateSystem is the Python callable seam over Atlas's native graph
# sampler. The concrete axes keep their live coordinate transform, graph
# metadata/style remain Reference-visible, and the tick step is divided by
# num_sampled_graph_points_per_tick before native construction.
coordinate_systems = importlib.import_module("manimlib.mobject.coordinate_systems")
assert issubclass(manimlib.Axes, coordinate_systems.CoordinateSystem)
graph_axes = manimlib.Axes(
    x_range=(0.0, 2.0, 1.0),
    y_range=(0.0, 4.0, 1.0),
    width=4.0,
    height=4.0,
    num_sampled_graph_points_per_tick=2,
)
assert graph_axes.get_axis(0) is graph_axes.x_axis
assert graph_axes.get_x_axis() is graph_axes.x_axis
assert graph_axes.get_y_axis() is graph_axes.y_axis
assert np.allclose(graph_axes.get_origin(), graph_axes.c2p(0.0, 0.0))
assert np.allclose(graph_axes.p2c(graph_axes.c2p(0.5, 0.25)), [0.5, 0.25])

# NumberLine.add_numbers forwards DecimalNumber precision through Atlas's
# native label builder. Unknown forwarded keys refuse before hanging a label
# group, rather than disappearing into **kwargs.
integer_label_line = manimlib.NumberLine(x_range=(0.0, 1.0, 1.0))
integer_labels = integer_label_line.add_numbers(
    x_values=[0.5], num_decimal_places=0
)
precise_label_line = manimlib.NumberLine(x_range=(0.0, 1.0, 1.0))
precise_labels = precise_label_line.add_numbers(
    x_values=[0.5], num_decimal_places=2
)
assert precise_labels.get_width() > integer_labels.get_width()
assert precise_label_line.numbers is precise_labels

# NumberLine is a Line in the Reference MRO. Constructor ticks land as the
# last native child; get_tick / add_ticks / get_number_mobject are the
# remaining public surface over that family.
assert issubclass(manimlib.NumberLine, manimlib.Line)
tick_line = manimlib.NumberLine(x_range=(-2.0, 2.0, 1.0), tick_size=0.1)
assert isinstance(tick_line, manimlib.Line)
marks = tick_line.get_tick_marks()
assert marks is tick_line.ticks
assert len(marks) == 5
zero_tick = tick_line.get_tick(0.0)
assert np.allclose(zero_tick.get_center(), tick_line.n2p(0.0), atol=1e-6)
assert np.isclose(zero_tick.get_height(), 0.2, atol=1e-6)
bare_line = manimlib.NumberLine(x_range=(0.0, 1.0, 1.0), include_ticks=False)
assert len(bare_line.get_tick_marks()) == 0
bare_line.add_ticks()
assert len(bare_line.get_tick_marks()) == 2
label = tick_line.get_number_mobject(1.0, font_size=24)
assert isinstance(label, manimlib.DecimalNumber)
assert np.isclose(label.get_value(), 1.0)

# NumberLine forwards VMobject's stroke_color, which wins over color for the
# stroke (`stroke_color or color`) and which the ticks match; None defers
# to color (subsets, laplace shm and print_gallery pass it).
stroked_line = manimlib.NumberLine((-2, 2), stroke_color=manimlib.RED, color=manimlib.BLUE)
assert stroked_line.get_stroke_color() == manimlib.RED
assert stroked_line.get_fill_color() == manimlib.BLUE
assert all(
    tick.get_stroke_color() == manimlib.RED for tick in stroked_line.get_tick_marks()
)
assert manimlib.NumberLine((-2, 2), stroke_color=None, color=manimlib.BLUE).get_stroke_color() \
    == manimlib.BLUE

# ComplexPlane.n2p is the Reference's
# coords_to_point(np.real(number), np.imag(number)), so an array of
# numbers maps to an array of points (zeta's partial-sum paths).
complex_plane = manimlib.ComplexPlane()
complex_numbers = np.array([1 + 1j, 2.0, -1.5j])
complex_points = complex_plane.n2p(complex_numbers)
assert complex_points.shape == (3, 3)
assert np.allclose(complex_points, [complex_plane.n2p(z) for z in complex_numbers])
assert issubclass(manimlib.UnitInterval, manimlib.NumberLine)
unit = manimlib.UnitInterval()
assert np.allclose(unit.x_range[:2], [0.0, 1.0])

# Slider is an authored portal class over Atlas's existing slider builder,
# not the schema-generated constructor refusal. The builder owns the initial
# family and layout; the portal binds those parts to the live ValueTracker.
number_line_module = importlib.import_module("manimlib.mobject.number_line")
assert number_line_module.Slider.__bases__ == (manimlib.VGroup,)
slider_signature = inspect.signature(number_line_module.Slider)
assert tuple(slider_signature.parameters) == (
    "value_tracker",
    "x_range",
    "var_name",
    "width",
    "unit_size",
    "arrow_width",
    "arrow_length",
    "arrow_color",
    "font_size",
    "label_buff",
    "num_decimal_places",
    "tick_size",
    "number_line_config",
    "arrow_tip_config",
    "decimal_config",
    "angle",
    "label_direction",
    "add_tick_labels",
    "tick_label_font_size",
)
slider_tracker = manimlib.ValueTracker(0.25)
slider = number_line_module.Slider(
    slider_tracker,
    x_range=(-1.0, 1.0),
    var_name="x",
    width=4.0,
    add_tick_labels=False,
)
assert len(slider) == 3
assert slider[0] is slider.number_line
assert slider[1] is slider.tip
assert slider[2] is slider.label
assert isinstance(slider.number_line, manimlib.NumberLine)
assert isinstance(slider.tip, manimlib.ArrowTip)
assert isinstance(slider.decimal, manimlib.DecimalNumber)
assert np.isclose(slider.decimal.get_value(), 0.25)
assert np.allclose(
    slider.tip.get_center(), slider.number_line.n2p(0.25), atol=1e-6
)
slider_tracker.set_value(0.75)
slider.update(0.0)
assert np.isclose(slider.decimal.get_value(), 0.75)
assert np.allclose(
    slider.tip.get_center(), slider.number_line.n2p(0.75), atol=1e-6
)
failed_slider = number_line_module.Slider.__new__(number_line_module.Slider)
try:
    number_line_module.Slider.__init__(
        failed_slider,
        slider_tracker,
        number_line_config={"unsupported": True},
    )
except NotImplementedError as error:
    assert str(error) == (
        "Slider() keyword(s) not yet routed to the native builder: "
        "number_line_config.unsupported"
    )
else:
    raise AssertionError("Slider silently dropped an unsupported line option")
assert not hasattr(failed_slider, "submobjects")

# SampleSpace is the probability shelf's native Rectangle specialization,
# not a schema-generated constructor refusal. Atlas owns its dimensions and
# VMobject style while the portal preserves the Reference-only label scale.
probability = importlib.import_module("manimlib.mobject.probability")
assert probability.SampleSpace.__bases__ == (manimlib.Rectangle,)
assert str(inspect.signature(probability.SampleSpace)) == (
    "(width=3, height=3, fill_color='#444444', fill_opacity=1, "
    "stroke_width=0.5, stroke_color='#BBBBBB', "
    "default_label_scale_val=1, **kwargs)"
)
sample_space = probability.SampleSpace(
    width=5.0,
    height=2.5,
    fill_color=manimlib.BLUE,
    fill_opacity=0.75,
    stroke_width=1.25,
    stroke_color=manimlib.YELLOW,
    default_label_scale_val=0.8,
    z_index=7,
)
assert np.isclose(sample_space.get_width(), 5.0, atol=1e-6)
assert np.isclose(sample_space.get_height(), 2.5, atol=1e-6)
assert sample_space.get_fill_color() == manimlib.BLUE
assert np.isclose(sample_space.get_fill_opacity(), 0.75)
assert sample_space.get_stroke_color() == manimlib.YELLOW
assert np.isclose(sample_space.get_stroke_width(), 1.25)
assert sample_space.default_label_scale_val == 0.8
assert sample_space.z_index == 7
sample_space_scene = Scene()
sample_space_scene.add(sample_space)
assert sample_space._is_bound()

failed_sample_space = probability.SampleSpace.__new__(probability.SampleSpace)
try:
    probability.SampleSpace.__init__(failed_sample_space, unsupported=True)
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: unsupported"
else:
    raise AssertionError("SampleSpace silently discarded an unknown option")
assert not hasattr(failed_sample_space, "submobjects")

# BarChart retires the probability module's remaining schema constructor
# refusal by composing Atlas-native lines, rectangles, and Scribe labels.
assert probability.BarChart.__bases__ == (manimlib.VGroup,)
bar_chart_signature = inspect.signature(probability.BarChart)
assert tuple(bar_chart_signature.parameters) == (
    "values",
    "height",
    "width",
    "n_ticks",
    "include_x_ticks",
    "tick_width",
    "tick_height",
    "label_y_axis",
    "y_axis_label_height",
    "max_value",
    "bar_colors",
    "bar_fill_opacity",
    "bar_stroke_width",
    "bar_names",
    "bar_label_scale_val",
    "kwargs",
)
bar_chart = probability.BarChart(
    [0.25, 0.75],
    height=2.0,
    width=4.0,
    n_ticks=2,
    include_x_ticks=True,
    bar_names=["A", "B"],
)
assert len(bar_chart.bars) == 2
assert len(bar_chart.bar_labels) == 2
assert len(bar_chart.y_axis_labels) == 3
assert len(bar_chart.x_axis.submobjects[0]) == 3
assert len(bar_chart.y_axis.submobjects[0]) == 3
assert np.isclose(bar_chart.bars[0].get_width(), 1.0, atol=1e-6)
assert np.isclose(bar_chart.bars[0].get_height(), 0.5, atol=1e-6)
assert np.isclose(bar_chart.bars[1].get_height(), 1.5, atol=1e-6)
assert bar_chart.bars[0].get_fill_color() == manimlib.BLUE
assert bar_chart.bars[1].get_fill_color() == manimlib.YELLOW
chart_bottoms = [bar.get_bottom().copy() for bar in bar_chart.bars]
bar_chart.change_bar_values([0.5, 1.0])
assert np.isclose(bar_chart.bars[0].get_height(), 1.0, atol=1e-6)
assert np.isclose(bar_chart.bars[1].get_height(), 2.0, atol=1e-6)
assert np.allclose(bar_chart.bars[0].get_bottom(), chart_bottoms[0])
assert np.allclose(bar_chart.bars[1].get_bottom(), chart_bottoms[1])
auto_max_chart = probability.BarChart(
    [2.0, 4.0], max_value=None, label_y_axis=False
)
assert auto_max_chart.max_value == 4.0
assert not hasattr(auto_max_chart, "y_axis_labels")
bar_chart_scene = Scene()
bar_chart_scene.add(bar_chart)
assert bar_chart._is_bound()

refusing_label_line = manimlib.NumberLine(x_range=(0.0, 1.0, 1.0))
refusing_label_children = len(refusing_label_line.submobjects)
try:
    refusing_label_line.add_numbers(x_values=[0.5], bogus=True)
except NotImplementedError as error:
    assert str(error) == (
        "NumberLine.add_numbers() keyword(s) not yet routed to the native builder: "
        "bogus"
    )
else:
    raise AssertionError("NumberLine.add_numbers silently dropped bogus")
assert len(refusing_label_line.submobjects) == refusing_label_children
assert not hasattr(refusing_label_line, "numbers")

# NumberPlane shares Axes' native decimal-label shells while retaining the
# plane-specific axis defaults used to place labels below-left of each axis.
label_plane = manimlib.NumberPlane(
    x_range=(-2.0, 3.0, 1.0),
    y_range=(-1.0, 2.0, 1.0),
    width=4.0,
    height=2.0,
)
label_counts_before = (
    len(label_plane.x_axis.submobjects),
    len(label_plane.y_axis.submobjects),
)
# coordinate_systems.py:520 returns the labels, kept as coordinate_labels.
assert label_plane.add_coordinate_labels() is label_plane.coordinate_labels
assert list(label_plane.coordinate_labels) == [
    label_plane.x_axis.numbers,
    label_plane.y_axis.numbers,
]
assert len(label_plane.x_axis.submobjects) == label_counts_before[0] + 1
assert len(label_plane.y_axis.submobjects) == label_counts_before[1] + 1
plane_label_members = [
    member
    for axis in (label_plane.x_axis, label_plane.y_axis)
    for member in axis.submobjects[-1].family_members_with_points()
]
assert plane_label_members
assert any(member.get_width() > 0.0 for member in plane_label_members)
assert any(member.get_height() > 0.0 for member in plane_label_members)

# The native DecimalNumber shelf accepts font_size, so Axes forwards it.
small_labels = manimlib.Axes(
    x_range=(-1.0, 2.0, 1.0), y_range=(-1.0, 2.0, 1.0)
).add_coordinate_labels(font_size=18)
large_labels = manimlib.Axes(
    x_range=(-1.0, 2.0, 1.0), y_range=(-1.0, 2.0, 1.0)
).add_coordinate_labels(font_size=36)
assert large_labels[0].get_height() > small_labels[0].get_height()

try:
    manimlib.NumberPlane().add_coordinate_labels(excluding=object())
except TypeError as error:
    assert str(error) == (
        "NumberPlane.add_coordinate_labels excluding must be an iterable "
        "of real numbers"
    )
else:
    raise AssertionError("NumberPlane.add_coordinate_labels accepted bad excluding")

sampled_xs = []


def sampled_square(x):
    sampled_xs.append(float(x))
    return x * x


static_graph = graph_axes.get_graph(
    sampled_square,
    x_range=(0.0, 2.0, 1.0),
    color=manimlib.RED,
    stroke_width=7.0,
    use_smoothing=False,
)
assert isinstance(static_graph, functions.ParametricCurve)
assert static_graph.underlying_function is sampled_square
assert static_graph.x_range == (0.0, 2.0, 1.0)
assert static_graph.t_range == (0.0, 2.0, 0.5)
assert np.allclose(sampled_xs, [0.0, 0.5, 1.0, 1.5, 2.0])
assert static_graph.get_stroke_color() == manimlib.RED
assert math.isclose(static_graph.get_stroke_width(), 7.0)
assert np.allclose(static_graph.get_start(), graph_axes.c2p(0.0, 0.0))
assert np.allclose(static_graph.get_end(), graph_axes.c2p(2.0, 4.0))
assert np.allclose(
    graph_axes.input_to_graph_point(0.5, static_graph),
    graph_axes.c2p(0.5, 0.25),
)
assert np.allclose(graph_axes.i2gp(1.5, static_graph), graph_axes.c2p(1.5, 2.25))

parametric_graph = graph_axes.get_parametric_curve(
    lambda t: np.array([t, 2.0 * t]),
    t_range=(0.0, 1.0, 0.5),
    use_smoothing=False,
)
assert np.allclose(parametric_graph.get_start(), graph_axes.c2p(0.0, 0.0))
assert np.allclose(parametric_graph.get_end(), graph_axes.c2p(1.0, 2.0))

# The pinned function-less fallback deliberately uses quick curve-count
# interpolation and the Reference binary search, not true arclength.
unit_axes = manimlib.Axes(
    x_range=(0.0, 1.0, 0.25),
    y_range=(0.0, 1.0, 0.25),
    width=2.0,
    height=2.0,
)
functionless_graph = functions.ParametricCurve(
    lambda t: unit_axes.c2p(t, t),
    t_range=(0.0, 1.0, 0.05),
    use_smoothing=False,
)
fallback_point = unit_axes.input_to_graph_point(0.4, functionless_graph)
assert math.isclose(
    unit_axes.point_to_coords(fallback_point)[0],
    0.4,
    rel_tol=0.0,
    abs_tol=2e-4,
)

# bind=True retains the Reference's vectorized-function contract and routes
# smoothing into Chisel after each corner refresh. The updater changes the
# same graph object and remains usable by ordinary graph-point queries.
bound_scale = [1.0]
bound_graph = unit_axes.get_graph(
    lambda x: bound_scale[0] * np.asarray(x),
    bind=True,
    use_smoothing=False,
)
assert len(bound_graph.updaters) == 1
bound_before = bound_graph.get_points().copy()
bound_scale[0] = 0.5
bound_graph.update(0.0)
assert bound_graph.get_points().shape != bound_before.shape or not np.allclose(bound_graph.get_points(), bound_before)
assert np.allclose(
    unit_axes.i2gp(0.75, bound_graph),
    unit_axes.c2p(0.75, 0.375),
)

discontinuous_graph = unit_axes.get_graph(
    lambda x: np.asarray(x),
    use_smoothing=False,
)
unit_axes.bind_graph_to_func(
    discontinuous_graph,
    lambda x: np.asarray(x),
    jagged=True,
    get_discontinuities=lambda: [0.5],
)
assert len(discontinuous_graph.updaters) == 1
discontinuous_xs = unit_axes.point_to_coords(discontinuous_graph.get_points())[0]
assert np.any(np.isclose(discontinuous_xs, 0.5 - 1e-6, atol=1e-12))
assert np.any(np.isclose(discontinuous_xs, 0.5 + 1e-6, atol=1e-12))


def refuse_vectorized_graph_callback(x):
    if np.ndim(x) != 0:
        raise LookupError("vectorized graph callback failed")
    return x


try:
    unit_axes.bind_graph_to_func(
        unit_axes.get_graph(lambda x: x, use_smoothing=False),
        refuse_vectorized_graph_callback,
    ).update(0.0)
except LookupError as error:
    assert str(error) == "vectorized graph callback failed"
else:
    raise AssertionError("a bound graph callback exception was swallowed")

# Family recursion is Python-owned while each smoothing operation is native,
# so recurse=False cannot accidentally rewrite a child path.
smooth_root = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]]
)
smooth_child = VMobject().set_points_as_corners(
    [[0.0, 0.0, 0.0], [0.5, -1.0, 0.0], [1.0, 0.0, 0.0]]
)
smooth_root.add(smooth_child)
smooth_root_before = smooth_root.get_points().copy()
smooth_child_before = smooth_child.get_points().copy()
assert smooth_root.make_smooth(approx=True, recurse=False) is smooth_root
assert not np.allclose(smooth_root.get_points(), smooth_root_before)
assert np.allclose(smooth_child.get_points(), smooth_child_before)
assert smooth_child.make_smooth(approx=False) is smooth_child
assert smooth_child.has_points()

three_dimensions = importlib.import_module("manimlib.mobject.three_dimensions")
sphere = three_dimensions.Sphere(
    radius=2.0,
    clockwise=True,
    resolution=(5, 3),
    preferred_creation_axis=0,
    epsilon=0.02,
    normal_nudge=0.25,
)
assert sphere.n_records() == 15
assert sphere.preferred_creation_axis == 0
assert np.isclose(sphere.epsilon, 0.02)
assert np.isclose(sphere.normal_nudge, 0.25)
assert np.allclose(
    np.linalg.norm(
        sphere.data["d_normal_point"] - sphere.data["point"], axis=1
    ),
    0.25,
    atol=1e-6,
)
assert np.allclose(sphere.uv_func(0.0, 0.0), [0.0, 0.0, -2.0])
assert np.allclose(sphere.uv_func(0.0, math.pi / 2.0), [2.0, 0.0, 0.0])
assert sphere.init_points() is None
assert sphere.n_records() == 15

failed_sphere = three_dimensions.Sphere.__new__(three_dimensions.Sphere)
try:
    three_dimensions.Sphere.__init__(failed_sphere, texture_paths={})
except NotImplementedError as error:
    assert str(error) == (
        "Sphere() keyword(s) not yet routed to the native builder: texture_paths"
    )
else:
    raise AssertionError("an unrouted Sphere keyword reached the native builder")
assert not hasattr(failed_sphere, "submobjects")

# Atlas owns the complete parametric solid shelf. Every authored portal class
# preserves the pinned MRO/signature while construction and public uv_func
# calls share the exact native parameterization.
cylinder = three_dimensions.Cylinder(
    u_range=(0.0, math.tau),
    v_range=(-1.0, 1.0),
    resolution=(5, 3),
    height=4.0,
    radius=2.0,
    axis=manimlib.OUT,
    color=manimlib.BLUE,
    depth_test=False,
)
assert cylinder.n_records() == 15
assert cylinder.init_points() is None
assert cylinder.n_records() == 15
assert cylinder.u_range == (0.0, math.tau)
assert cylinder.v_range == (-1.0, 1.0)
assert cylinder.resolution == (5, 3)
assert cylinder.height == 4.0
assert cylinder.radius == 2.0
assert np.array_equal(cylinder.axis, manimlib.OUT)
assert np.allclose(cylinder.uv_func(0.0, 0.25), [1.0, 0.0, 0.25])
assert np.allclose(
    [cylinder.get_width(), cylinder.get_height(), cylinder.get_depth()],
    [4.0, 4.0, 4.0],
    atol=1e-6,
)
assert cylinder.uniforms["depth_test"] is False
assert cylinder.get_color() == manimlib.BLUE

right_cylinder = three_dimensions.Cylinder(
    resolution=(5, 3), height=6.0, radius=1.0, axis=manimlib.RIGHT
)
assert np.allclose(
    [right_cylinder.get_width(), right_cylinder.get_height(), right_cylinder.get_depth()],
    [6.0, 2.0, 2.0],
    atol=1e-6,
)

default_cylinder = three_dimensions.Cylinder()
assert default_cylinder.n_records() == 101 * 11

back_cylinder = three_dimensions.Cylinder(resolution=(3, 3), z_index=-1)
front_cylinder = three_dimensions.Cylinder(resolution=(3, 3), z_index=1)
cylinder_order_scene = Scene().add(front_cylinder, back_cylinder)
assert cylinder_order_scene.get_mobjects() == [back_cylinder, front_cylinder]

failed_cylinder = three_dimensions.Cylinder.__new__(three_dimensions.Cylinder)
try:
    three_dimensions.Cylinder.__init__(failed_cylinder, texture_paths={})
except NotImplementedError as error:
    assert str(error) == (
        "Cylinder() keyword(s) not yet routed to the native builder: texture_paths"
    )
else:
    raise AssertionError("an unrouted Cylinder keyword reached the native builder")
assert not hasattr(failed_cylinder, "submobjects")

assert three_dimensions.Torus.__bases__ == (manimlib.Surface,)
assert three_dimensions.Cone.__bases__ == (three_dimensions.Cylinder,)
assert three_dimensions.Line3D.__bases__ == (three_dimensions.Cylinder,)
assert three_dimensions.Disk3D.__bases__ == (manimlib.Surface,)
assert three_dimensions.Square3D.__bases__ == (manimlib.Surface,)
assert str(inspect.signature(three_dimensions.Torus)) == (
    "(u_range=(0, 6.283185307179586), v_range=(0, 6.283185307179586), "
    "r1=3.0, r2=1.0, **kwargs)"
)
assert str(inspect.signature(three_dimensions.Cone)) == (
    "(u_range=(0, 6.283185307179586), v_range=(0, 1), *args, **kwargs)"
)
assert str(inspect.signature(three_dimensions.Line3D)) == (
    "(start, end, width=0.05, resolution=(21, 25), **kwargs)"
)
assert str(inspect.signature(three_dimensions.Disk3D)) == (
    "(radius=1, u_range=(0, 1), v_range=(0, 6.283185307179586), "
    "resolution=(2, 100), **kwargs)"
)
assert str(inspect.signature(three_dimensions.Square3D)) == (
    "(side_length=2.0, u_range=(-1, 1), v_range=(-1, 1), "
    "resolution=(2, 2), **kwargs)"
)

torus = three_dimensions.Torus(
    r1=2.0,
    r2=0.5,
    resolution=(5, 5),
    color=manimlib.BLUE,
    opacity=0.6,
    shading=(0.1, 0.2, 0.3),
    depth_test=False,
)
assert torus.n_records() == 25
assert np.allclose(torus.uv_func(0.0, 0.0), [1.5, 0.0, 0.0])
assert np.allclose(torus.uv_func(0.0, math.pi), [2.5, 0.0, 0.0])
assert np.allclose(
    [torus.get_width(), torus.get_height(), torus.get_depth()],
    [5.0, 5.0, 1.0],
    atol=1e-6,
)
assert torus.get_color() == manimlib.BLUE
assert np.isclose(torus.get_opacity(), 0.6)
assert np.allclose(torus.uniforms["shading"], [0.1, 0.2, 0.3])
assert torus.uniforms["depth_test"] is False

default_torus = three_dimensions.Torus()
assert default_torus.n_records() == 101 * 101

back_torus = three_dimensions.Torus(resolution=(3, 3), z_index=-1)
front_torus = three_dimensions.Torus(resolution=(3, 3), z_index=1)
torus_order_scene = Scene().add(front_torus, back_torus)
assert torus_order_scene.get_mobjects() == [back_torus, front_torus]

sphere_mesh = three_dimensions.SurfaceMesh(sphere, resolution=(4, 3))
assert len(sphere_mesh.submobjects) == 7
assert all(isinstance(line, VMobject) and line.has_points() for line in sphere_mesh)
assert sphere_mesh.init_points() is None
assert len(sphere_mesh.submobjects) == 7
assert sphere_mesh.get_joint_type() == VMobject.joint_type_map["no_joint"]

bevel_mesh = three_dimensions.SurfaceMesh(
    sphere, resolution=(4, 3), joint_type="bevel", depth_test=False
)
assert bevel_mesh.get_joint_type() == VMobject.joint_type_map["bevel"]
assert bevel_mesh.uniforms["depth_test"] is False
assert all(
    line.get_joint_type() == VMobject.joint_type_map["bevel"]
    for line in bevel_mesh.submobjects
)

torus_mesh = three_dimensions.SurfaceMesh(torus, resolution=(4, 3))
assert len(torus_mesh.submobjects) == 7
assert all(isinstance(line, VMobject) and line.has_points() for line in torus_mesh)

cylinder_mesh_source = three_dimensions.Cylinder(
    height=3.0, radius=1.0, axis=manimlib.RIGHT, resolution=(5, 3)
)
cylinder_mesh = three_dimensions.SurfaceMesh(
    cylinder_mesh_source, resolution=(4, 3)
)
assert len(cylinder_mesh.submobjects) == 7
assert all(isinstance(line, VMobject) and line.has_points() for line in cylinder_mesh)

for unsupported_surface in (None, VMobject()):
    try:
        three_dimensions.SurfaceMesh(unsupported_surface)
    except TypeError as error:
        assert str(error) == "SurfaceMesh source must be a Surface"
    else:
        raise AssertionError(
            "SurfaceMesh accepted a non-surface source"
        )

cone = three_dimensions.Cone(
    resolution=(5, 3),
    height=3.0,
    radius=2.0,
    axis=manimlib.RIGHT,
)
cone_mesh = three_dimensions.SurfaceMesh(cone, resolution=(4, 3))
assert len(cone_mesh.submobjects) == 7
assert all(isinstance(line, VMobject) and line.has_points() for line in cone_mesh)
assert cone.n_records() == 15
assert cone.u_range == (0, math.tau)
assert cone.v_range == (0, 1)
assert np.allclose(cone.uv_func(0.0, 0.0), [1.0, 0.0, 0.0])
assert np.allclose(cone.uv_func(0.0, 1.0), [0.0, 0.0, 1.0])
assert np.allclose(
    [cone.get_width(), cone.get_height(), cone.get_depth()],
    [3.0, 4.0, 4.0],
    atol=1e-6,
)
# scale and set_depth act about the sampled cone's center (z = 1/2), then
# apply_matrix about the origin: the pinned Reference measures this cone at
# x in [-1, 2] with center (0.5, 0, 0) (0a5539ed).
assert np.allclose(cone.get_center(), [0.5, 0.0, 0.0], atol=1e-6)
assert np.allclose([cone.get_points()[:, 0].min(), cone.get_points()[:, 0].max()], [-1.0, 2.0], atol=1e-6)

default_cone = three_dimensions.Cone()
assert default_cone.n_records() == 101 * 11

back_cone = three_dimensions.Cone(resolution=(3, 3), z_index=-1)
front_cone = three_dimensions.Cone(resolution=(3, 3), z_index=1)
cone_order_scene = Scene().add(front_cone, back_cone)
assert cone_order_scene.get_mobjects() == [back_cone, front_cone]

failed_cone = three_dimensions.Cone.__new__(three_dimensions.Cone)
try:
    three_dimensions.Cone.__init__(failed_cone, texture_paths={})
except NotImplementedError as error:
    assert str(error) == (
        "Cone() keyword(s) not yet routed to the native builder: texture_paths"
    )
else:
    raise AssertionError("an unrouted Cone keyword reached the native builder")
assert not hasattr(failed_cone, "submobjects")

line3d = three_dimensions.Line3D(
    [-2.0, 0.0, 0.0],
    [2.0, 0.0, 0.0],
    width=0.4,
    resolution=(5, 3),
)
line3d_mesh = three_dimensions.SurfaceMesh(line3d, resolution=(4, 3))
assert len(line3d_mesh.submobjects) == 7
assert all(isinstance(line, VMobject) and line.has_points() for line in line3d_mesh)
assert line3d.n_records() == 15
assert np.isclose(line3d.height, 4.0)
assert np.isclose(line3d.radius, 0.2)
assert np.array_equal(line3d.axis, [4.0, 0.0, 0.0])
assert np.allclose(line3d.uv_func(0.0, 0.25), [1.0, 0.0, 0.25])
assert np.allclose(
    [line3d.get_width(), line3d.get_height(), line3d.get_depth()],
    [4.0, 0.4, 0.4],
    atol=1e-6,
)

default_line3d = three_dimensions.Line3D(manimlib.LEFT, manimlib.RIGHT)
assert default_line3d.n_records() == 21 * 25

back_line3d = three_dimensions.Line3D(
    manimlib.DOWN,
    manimlib.UP,
    resolution=(3, 3),
    z_index=-1,
)
front_line3d = three_dimensions.Line3D(
    manimlib.DOWN,
    manimlib.UP,
    resolution=(3, 3),
    z_index=1,
)
line3d_order_scene = Scene().add(front_line3d, back_line3d)
assert line3d_order_scene.get_mobjects() == [back_line3d, front_line3d]

failed_line3d = three_dimensions.Line3D.__new__(three_dimensions.Line3D)
try:
    three_dimensions.Line3D.__init__(
        failed_line3d,
        manimlib.LEFT,
        manimlib.RIGHT,
        texture_paths={},
    )
except NotImplementedError as error:
    assert str(error) == (
        "Line3D() keyword(s) not yet routed to the native builder: texture_paths"
    )
else:
    raise AssertionError("an unrouted Line3D keyword reached the native builder")
assert not hasattr(failed_line3d, "submobjects")

disk3d = three_dimensions.Disk3D(radius=2.0, resolution=(2, 5))
disk_mesh = three_dimensions.SurfaceMesh(disk3d, resolution=(4, 3))
assert len(disk_mesh.submobjects) == 7
assert all(isinstance(line, VMobject) and line.has_points() for line in disk_mesh)
assert disk3d.n_records() == 10
assert np.allclose(disk3d.uv_func(0.5, 0.0), [0.5, 0.0, 0.0])
assert np.allclose(
    [disk3d.get_width(), disk3d.get_height(), disk3d.get_depth()],
    [4.0, 4.0, 0.0],
    atol=1e-6,
)

default_disk3d = three_dimensions.Disk3D()
assert default_disk3d.n_records() == 2 * 100

back_disk3d = three_dimensions.Disk3D(resolution=(2, 5), z_index=-1)
front_disk3d = three_dimensions.Disk3D(resolution=(2, 5), z_index=1)
disk3d_order_scene = Scene().add(front_disk3d, back_disk3d)
assert disk3d_order_scene.get_mobjects() == [back_disk3d, front_disk3d]

failed_disk3d = three_dimensions.Disk3D.__new__(three_dimensions.Disk3D)
try:
    three_dimensions.Disk3D.__init__(failed_disk3d, texture_paths={})
except NotImplementedError as error:
    assert str(error) == (
        "Disk3D() keyword(s) not yet routed to the native builder: texture_paths"
    )
else:
    raise AssertionError("an unrouted Disk3D keyword reached the native builder")
assert not hasattr(failed_disk3d, "submobjects")

square3d = three_dimensions.Square3D(
    side_length=3.0,
    u_range=(-2.0, 2.0),
    v_range=(-1.0, 1.0),
    resolution=(3, 2),
)
square_mesh = three_dimensions.SurfaceMesh(square3d, resolution=(4, 3))
assert len(square_mesh.submobjects) == 7
assert all(isinstance(line, VMobject) and line.has_points() for line in square_mesh)
assert square3d.n_records() == 6
assert np.allclose(square3d.uv_func(-2.0, 1.0), [-2.0, 1.0, 0.0])
assert np.allclose(
    [square3d.get_width(), square3d.get_height(), square3d.get_depth()],
    [6.0, 3.0, 0.0],
    atol=1e-6,
)

default_square3d = three_dimensions.Square3D()
assert default_square3d.n_records() == 2 * 2

back_square3d = three_dimensions.Square3D(z_index=-1)
front_square3d = three_dimensions.Square3D(z_index=1)
square3d_order_scene = Scene().add(front_square3d, back_square3d)
assert square3d_order_scene.get_mobjects() == [back_square3d, front_square3d]

failed_square3d = three_dimensions.Square3D.__new__(three_dimensions.Square3D)
try:
    three_dimensions.Square3D.__init__(failed_square3d, texture_paths={})
except NotImplementedError as error:
    assert str(error) == (
        "Square3D() keyword(s) not yet routed to the native builder: texture_paths"
    )
else:
    raise AssertionError("an unrouted Square3D keyword reached the native builder")
assert not hasattr(failed_square3d, "submobjects")

solid_scene = Scene().add(torus, cone, line3d, disk3d, square3d)
assert solid_scene.get_mobjects() == [torus, cone, line3d, disk3d, square3d]

cube_face = three_dimensions.Square3D(side_length=2.0)
cube_faces = three_dimensions.square_to_cube_faces(cube_face)
assert manimlib.square_to_cube_faces is three_dimensions.square_to_cube_faces
assert len(cube_faces) == 6
assert np.allclose(cube_face.get_center(), [0.0, 0.0, 1.0], atol=1e-6)
assert np.allclose(cube_faces[0].get_center(), [0.0, 0.0, 1.0], atol=1e-6)
assert np.isclose(cube_faces[-1].get_center()[2], -1.0, atol=1e-5)

default_cube = three_dimensions.Cube()
assert len(default_cube.submobjects) == 6
assert all(face.n_records() == 2 * 2 for face in default_cube.submobjects)

dense_cube = three_dimensions.Cube(square_resolution=(3, 4))
assert dense_cube.resolution == (3, 4)
assert all(face.n_records() == 3 * 4 for face in dense_cube.submobjects)

back_cube = three_dimensions.Cube(z_index=-1)
front_cube = three_dimensions.Cube(z_index=1)
cube_order_scene = Scene().add(front_cube, back_cube)
assert cube_order_scene.get_mobjects() == [back_cube, front_cube]

failed_cube = three_dimensions.Cube.__new__(three_dimensions.Cube)
try:
    three_dimensions.Cube.__init__(failed_cube, texture_paths={})
except NotImplementedError as error:
    assert str(error) == (
        "Cube() keyword(s) not yet routed to the native builder: texture_paths"
    )
else:
    raise AssertionError("an unrouted Cube keyword reached the native builder")
assert not hasattr(failed_cube, "submobjects")

# Atlas's vectorized 3D shelf is authored rather than schema-generated: the
# group keeps caller proxy identity while the four concrete solids install
# Atlas geometry and recurse 3D uniforms through the live family.
assert three_dimensions.VGroup3D.__bases__ == (manimlib.VGroup,)
assert three_dimensions.VCube.__bases__ == (three_dimensions.VGroup3D,)
assert three_dimensions.VPrism.__bases__ == (three_dimensions.VCube,)
assert three_dimensions.Dodecahedron.__bases__ == (three_dimensions.VGroup3D,)
assert three_dimensions.Prismify.__bases__ == (three_dimensions.VGroup3D,)
assert str(inspect.signature(three_dimensions.VGroup3D)) == (
    "(*vmobjects, depth_test=True, shading=(0.2, 0.2, 0.2), "
    "joint_type='no_joint', **kwargs)"
)
assert str(inspect.signature(three_dimensions.VCube)) == (
    "(side_length=2.0, fill_color='#29ABCA', fill_opacity=1, "
    "stroke_width=0, **kwargs)"
)
assert str(inspect.signature(three_dimensions.VPrism)) == (
    "(width=3.0, height=2.0, depth=1.0, **kwargs)"
)
assert str(inspect.signature(three_dimensions.Dodecahedron)) == (
    "(fill_color='#1C758A', fill_opacity=1, stroke_color='#1C758A', "
    "stroke_width=1, shading=(0.2, 0.2, 0.2), **kwargs)"
)
prismify_signature = inspect.signature(three_dimensions.Prismify)
assert tuple(prismify_signature.parameters) == (
    "vmobject", "depth", "direction", "kwargs"
)
assert prismify_signature.parameters["depth"].default == 1.0
assert np.array_equal(
    prismify_signature.parameters["direction"].default, manimlib.IN
)

group_left = manimlib.Square(side_length=0.5)
group_right = manimlib.Square(side_length=0.75).shift(manimlib.RIGHT)
joint_probe = manimlib.VGroup(
    manimlib.Square(side_length=0.2),
    manimlib.Square(side_length=0.2).shift(manimlib.RIGHT),
)
assert joint_probe.get_joint_type() == 1
assert joint_probe.set_joint_type("bevel") is joint_probe
assert all(member.get_joint_type() == 2 for member in joint_probe.get_family())
try:
    joint_probe.set_joint_type("rounded")
except KeyError as error:
    assert error.args == ("rounded",)
else:
    raise AssertionError("an unknown joint type was silently accepted")
assert all(member.get_joint_type() == 2 for member in joint_probe.get_family())

vector_group = three_dimensions.VGroup3D(
    group_left,
    group_right,
    depth_test=False,
    shading=(0.3, 0.4, 0.5),
    joint_type="miter",
)
assert vector_group.submobjects == [group_left, group_right]
assert vector_group.submobjects[0] is group_left
for member in vector_group.submobjects:
    assert member.uniforms["depth_test"] is False
    assert np.allclose(member.uniforms["shading"], [0.3, 0.4, 0.5])
    assert member.uniforms["joint_type"] == 3

default_prism = three_dimensions.Prism()
assert len(default_prism.submobjects) == 6
assert np.allclose(
    [default_prism.get_width(), default_prism.get_height(), default_prism.get_depth()],
    [3.0, 2.0, 1.0],
    atol=1e-6,
)

back_prism = three_dimensions.Prism(z_index=-1)
front_prism = three_dimensions.Prism(z_index=1)
prism_order_scene = Scene().add(front_prism, back_prism)
assert prism_order_scene.get_mobjects() == [back_prism, front_prism]

failed_prism = three_dimensions.Prism.__new__(three_dimensions.Prism)
try:
    three_dimensions.Prism.__init__(failed_prism, texture_paths={})
except NotImplementedError as error:
    assert str(error) == (
        "Prism() keyword(s) not yet routed to the native builder: texture_paths"
    )
else:
    raise AssertionError("an unrouted Prism keyword reached the native builder")
assert not hasattr(failed_prism, "submobjects")

vcube = three_dimensions.VCube(
    side_length=2.5,
    fill_color=manimlib.RED,
    fill_opacity=0.4,
    stroke_color=manimlib.BLUE,
    stroke_width=1.5,
    depth_test=False,
    shading=(0.1, 0.2, 0.3),
)
assert len(vcube.submobjects) == 6
assert np.allclose(
    [vcube.get_width(), vcube.get_height(), vcube.get_depth()],
    [2.5, 2.5, 2.5],
    atol=1e-6,
)
for face in vcube.submobjects:
    assert face.get_fill_color() == manimlib.RED
    assert np.isclose(face.get_fill_opacity(), 0.4)
    assert face.get_stroke_color() == manimlib.BLUE
    assert np.isclose(face.get_stroke_width(), 1.5)
    assert face.uniforms["depth_test"] is False
    assert np.allclose(face.uniforms["shading"], [0.1, 0.2, 0.3])
    assert face.get_joint_type() == 0

default_vcube = three_dimensions.VCube()
assert len(default_vcube.submobjects) == 6
assert np.allclose(
    [default_vcube.get_width(), default_vcube.get_height(), default_vcube.get_depth()],
    [2.0, 2.0, 2.0],
    atol=1e-6,
)

back_vcube = three_dimensions.VCube(z_index=-1)
front_vcube = three_dimensions.VCube(z_index=1)
vcube_order_scene = Scene().add(front_vcube, back_vcube)
assert vcube_order_scene.get_mobjects() == [back_vcube, front_vcube]

failed_vcube = three_dimensions.VCube.__new__(three_dimensions.VCube)
try:
    three_dimensions.VCube.__init__(failed_vcube, bogus=True)
except NotImplementedError as error:
    assert str(error) == (
        "VCube() keyword(s) not yet routed to the native builder: bogus"
    )
else:
    raise AssertionError("an unrouted VCube keyword reached the native builder")
assert not hasattr(failed_vcube, "submobjects")

vprism = three_dimensions.VPrism(
    width=4.0,
    height=3.0,
    depth=2.0,
    fill_color=manimlib.BLUE,
    fill_opacity=0.7,
)
assert len(vprism.submobjects) == 6
assert np.allclose(
    [vprism.get_width(), vprism.get_height(), vprism.get_depth()],
    [4.0, 3.0, 2.0],
    atol=1e-6,
)
assert all(face.get_fill_color() == manimlib.BLUE for face in vprism.submobjects)

default_vprism = three_dimensions.VPrism()
assert len(default_vprism.submobjects) == 6
assert np.allclose(
    [default_vprism.get_width(), default_vprism.get_height(), default_vprism.get_depth()],
    [3.0, 2.0, 1.0],
    atol=1e-6,
)

back_vprism = three_dimensions.VPrism(z_index=-1)
front_vprism = three_dimensions.VPrism(z_index=1)
vprism_order_scene = Scene().add(front_vprism, back_vprism)
assert vprism_order_scene.get_mobjects() == [back_vprism, front_vprism]

failed_vprism = three_dimensions.VPrism.__new__(three_dimensions.VPrism)
try:
    three_dimensions.VPrism.__init__(failed_vprism, bogus=True)
except NotImplementedError as error:
    assert str(error) == (
        "VPrism() keyword(s) not yet routed to the native builder: bogus"
    )
else:
    raise AssertionError("an unrouted VPrism keyword reached the native builder")
assert not hasattr(failed_vprism, "submobjects")

dodecahedron = three_dimensions.Dodecahedron(
    fill_color=manimlib.RED,
    fill_opacity=0.6,
    stroke_color=manimlib.BLUE,
    stroke_width=2.0,
)
assert len(dodecahedron.submobjects) == 12
assert all(face.n_records() == 11 for face in dodecahedron.submobjects)
phi = (1.0 + math.sqrt(5.0)) / 2.0
assert np.allclose(
    [dodecahedron.get_width(), dodecahedron.get_height(), dodecahedron.get_depth()],
    [2.0 * phi, 2.0 * phi, 2.0 * phi],
    atol=1e-6,
)

default_dodecahedron = three_dimensions.Dodecahedron()
assert len(default_dodecahedron.submobjects) == 12

back_dodecahedron = three_dimensions.Dodecahedron(z_index=-1)
front_dodecahedron = three_dimensions.Dodecahedron(z_index=1)
dodecahedron_order_scene = Scene().add(front_dodecahedron, back_dodecahedron)
assert dodecahedron_order_scene.get_mobjects() == [
    back_dodecahedron,
    front_dodecahedron,
]

failed_dodecahedron = three_dimensions.Dodecahedron.__new__(
    three_dimensions.Dodecahedron
)
try:
    three_dimensions.Dodecahedron.__init__(failed_dodecahedron, bogus=True)
except NotImplementedError as error:
    assert str(error) == (
        "Dodecahedron() keyword(s) not yet routed to the native builder: bogus"
    )
else:
    raise AssertionError("an unrouted Dodecahedron keyword reached the native builder")
assert not hasattr(failed_dodecahedron, "submobjects")

prismify_source = manimlib.Polygon(
    [-0.5, -0.5, 0.0],
    [0.75, -0.5, 0.0],
    [0.0, 0.75, 0.0],
    fill_color=manimlib.RED,
    fill_opacity=0.5,
    stroke_color=manimlib.BLUE,
    stroke_width=2.5,
)
prismify_source_points = prismify_source.get_points().copy()
prismified = three_dimensions.Prismify(
    prismify_source, depth=0.75, direction=manimlib.OUT
)
assert len(prismified.submobjects) == 5
assert np.array_equal(prismify_source.get_points(), prismify_source_points)
assert np.allclose(prismified.submobjects[0].get_points(), prismify_source_points)
assert np.isclose(prismified.submobjects[-1].get_depth(), 0.0)
assert np.allclose(
    prismified.submobjects[-1].get_center()
    - prismify_source.get_center(),
    [0.0, 0.0, 0.75],
    atol=1e-6,
)
for piece in prismified.submobjects:
    assert piece.get_fill_color() == manimlib.RED
    assert np.isclose(piece.get_fill_opacity(), 0.5)
    assert piece.get_stroke_color() == manimlib.BLUE
    assert np.isclose(piece.get_stroke_width(), 2.5)

vector_solid_scene = Scene().add(
    vector_group, vcube, vprism, dodecahedron, prismified
)
assert vector_solid_scene.get_mobjects() == [
    vector_group, vcube, vprism, dodecahedron, prismified
]

failed_vcube = three_dimensions.VCube.__new__(three_dimensions.VCube)
try:
    three_dimensions.VCube.__init__(failed_vcube, unrouted_option=True)
except NotImplementedError as error:
    assert str(error) == (
        "VCube() keyword(s) not yet routed to the native builder: "
        "unrouted_option"
    )
else:
    raise AssertionError("an unrouted VCube keyword reached the native builder")
assert not hasattr(failed_vcube, "submobjects")

prismified_family = three_dimensions.Prismify(
    manimlib.VGroup(prismify_source), depth=0.75, direction=manimlib.OUT
)
assert len(prismified_family.submobjects) == 1
assert len(prismified_family.submobjects[0].submobjects) == 5
assert np.allclose(
    prismified_family.submobjects[0].submobjects[0].get_points(),
    prismify_source_points,
)

# The two de-TeX'd special-text compositions are authored portal classes over
# Atlas/Scribe, not import-only schema shells. Construction owns layout in
# Rust; the live Python methods retain ordinary manim object identity.
special_tex = importlib.import_module("manimlib.mobject.svg.special_tex")
assert special_tex.BulletedList.__bases__ == (manimlib.VGroup,)
assert special_tex.Title.__bases__ == (manimlib.TexText,)
bullets_signature = inspect.signature(special_tex.BulletedList)
assert tuple(bullets_signature.parameters) == (
    "items", "buff", "aligned_edge", "numbered", "kwargs"
)
assert bullets_signature.parameters["items"].kind is inspect.Parameter.VAR_POSITIONAL
assert bullets_signature.parameters["buff"].kind is inspect.Parameter.KEYWORD_ONLY
assert bullets_signature.parameters["numbered"].default is False
title_signature = inspect.signature(special_tex.Title)
assert tuple(title_signature.parameters) == (
    "text_parts",
    "font_size",
    "include_underline",
    "underline_width",
    "match_underline_width_to_text",
    "underline_buff",
    "underline_style",
    "kwargs",
)
assert title_signature.parameters["text_parts"].kind is inspect.Parameter.VAR_POSITIONAL
assert title_signature.parameters["font_size"].default == 72
assert title_signature.parameters["include_underline"].default is True

bulleted = special_tex.BulletedList(
    "First",
    "A much longer second item",
    "Third",
    buff=0.35,
    font_size=30,
    color=manimlib.BLUE,
)
assert len(bulleted) == 3
assert all(len(item) == 2 for item in bulleted)
assert all(
    member.get_fill_color() == manimlib.BLUE
    for member in bulleted.family_members_with_points()
)
item_tops = [item.get_top()[1] for item in bulleted]
assert item_tops[0] > item_tops[1] > item_tops[2]
item_lefts = [item.get_left()[0] for item in bulleted]
assert np.allclose(item_lefts, item_lefts[0], atol=1e-6)
assert bulleted[1][1].get_width() > bulleted[0][1].get_width()

numbered = special_tex.BulletedList(
    "First", "Second", numbered=True, font_size=30
)
assert len(numbered) == 2
assert numbered[0][0].get_width() > bulleted[0][0].get_width()

item_identities = [id(item) for item in bulleted]
max_label_height = max(item[0].get_height() for item in bulleted)
assert bulleted.fade_all_but(1, opacity=0.2, scale_factor=0.6) is None
assert [id(item) for item in bulleted] == item_identities
assert np.isclose(bulleted[1][0].get_height(), max_label_height, atol=1e-6)
for index, item in enumerate(bulleted):
    expected_opacity = 1.0 if index == 1 else 0.2
    assert all(
        np.isclose(member.get_fill_opacity(), expected_opacity)
        for member in item.family_members_with_points()
    )

failed_bullets = special_tex.BulletedList.__new__(special_tex.BulletedList)
try:
    special_tex.BulletedList.__init__(failed_bullets, "item", unsupported=True)
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: unsupported"
else:
    raise AssertionError("BulletedList silently discarded an unknown option")
assert not hasattr(failed_bullets, "submobjects")

title = special_tex.Title(
    "Native",
    "Title",
    font_size=36,
    underline_width=4.0,
    underline_buff=0.2,
    underline_style=dict(stroke_width=3, stroke_color=manimlib.RED),
    color=manimlib.BLUE,
)
assert len(title) == 3
assert title[0] is title.submobjects[0]
assert title[1] is title.submobjects[1]
assert title.underline is title[2]
assert title.tex_strings == ["Native", "Title"]
assert title.tex_string == "Native Title"
assert title._string_sub_paths == [[0], [1]]
assert np.isclose(title.underline.get_width(), 4.0, atol=1e-6)
assert title.underline.get_stroke_color() == manimlib.RED
assert np.isclose(title.underline.get_stroke_width(), 3.0)
assert all(
    member.get_fill_color() == manimlib.BLUE
    for part in title[:2]
    for member in part.family_members_with_points()
)
text_bottom = min(part.get_bottom()[1] for part in title[:2])
assert np.isclose(
    title.underline.get_top()[1], text_bottom - 0.2, atol=1e-6
)
assert np.isclose(
    max(part.get_top()[1] for part in title[:2]),
    manimlib.FRAME_Y_RADIUS - manimlib.MED_SMALL_BUFF,
    atol=1e-6,
)

matched_title = special_tex.Title(
    "Matched",
    "Underline",
    font_size=30,
    match_underline_width_to_text=True,
)
matched_text_left = min(part.get_left()[0] for part in matched_title[:-1])
matched_text_right = max(part.get_right()[0] for part in matched_title[:-1])
assert np.isclose(
    matched_title.underline.get_width(),
    matched_text_right - matched_text_left,
    atol=1e-6,
)
plain_title = special_tex.Title("No underline", include_underline=False)
assert len(plain_title) == 1
assert not hasattr(plain_title, "underline")

failed_title = special_tex.Title.__new__(special_tex.Title)
try:
    special_tex.Title.__init__(
        failed_title, "Title", underline_style={"unsupported": True}
    )
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: unsupported"
else:
    raise AssertionError("Title silently discarded an unknown underline option")
assert not hasattr(failed_title, "submobjects")

# The scalar Matrix family is an authored portal over Atlas's one grid engine
# and the bundled fmd-math delimiter builder. Python keeps caller-visible
# entry identities and accessors; no placement or bracket math is duplicated.
matrix_module = importlib.import_module("manimlib.mobject.matrix")
assert matrix_module.Matrix.__bases__ == (manimlib.VMobject,)
assert matrix_module.DecimalMatrix.__bases__ == (matrix_module.Matrix,)
assert matrix_module.IntegerMatrix.__bases__ == (matrix_module.DecimalMatrix,)
assert matrix_module.TexMatrix.__bases__ == (matrix_module.Matrix,)
matrix_signature = inspect.signature(matrix_module.Matrix)
assert tuple(matrix_signature.parameters) == (
    "matrix",
    "v_buff",
    "h_buff",
    "bracket_h_buff",
    "bracket_v_buff",
    "height",
    "element_config",
    "element_alignment_corner",
    "ellipses_row",
    "ellipses_col",
)
assert tuple(inspect.signature(matrix_module.DecimalMatrix).parameters) == (
    "matrix", "num_decimal_places", "decimal_config", "config"
)
assert tuple(inspect.signature(matrix_module.IntegerMatrix).parameters) == (
    "matrix", "num_decimal_places", "decimal_config", "config"
)
assert tuple(inspect.signature(matrix_module.TexMatrix).parameters) == (
    "matrix", "tex_config", "config"
)
assert tuple(inspect.signature(matrix_module.Matrix.element_to_mobject).parameters) == (
    "self", "element", "config"
)

integer_tex_matrix = matrix_module.Matrix(
    [[1, 22], [333, 4]],
    v_buff=0.7,
    h_buff=0.8,
    element_config=dict(font_size=24, color=manimlib.BLUE),
)
assert len(integer_tex_matrix) == 6
assert integer_tex_matrix._matrix_shape == (2, 2)
assert all(
    isinstance(entry, manimlib.Tex)
    for row in integer_tex_matrix.get_mob_matrix()
    for entry in row
)
assert [
    entry.get_tex()
    for row in integer_tex_matrix.get_mob_matrix()
    for entry in row
] == ["1", "22", "333", "4"]
assert integer_tex_matrix.get_row(0)[0] is integer_tex_matrix.mob_matrix[0][0]
assert integer_tex_matrix.get_column(1)[1] is integer_tex_matrix.mob_matrix[1][1]
assert integer_tex_matrix.get_rows() is integer_tex_matrix.rows
assert integer_tex_matrix.get_columns() is integer_tex_matrix.columns
assert len(integer_tex_matrix.get_rows()) == 2
assert len(integer_tex_matrix.get_columns()) == 2
assert len(integer_tex_matrix.get_entries()) == 4
assert len(integer_tex_matrix.get_brackets()) == 2
assert len(integer_tex_matrix.get_ellipses()) == 0
assert integer_tex_matrix.mob_matrix[0][0].get_center()[0] < (
    integer_tex_matrix.mob_matrix[0][1].get_center()[0]
)
assert integer_tex_matrix.mob_matrix[0][0].get_center()[1] > (
    integer_tex_matrix.mob_matrix[1][0].get_center()[1]
)
assert all(
    member.get_fill_color() == manimlib.BLUE
    for entry in integer_tex_matrix.elements
    for member in entry.family_members_with_points()
)

integer_tex_matrix.set_column_colors(manimlib.RED, manimlib.YELLOW)
assert all(
    member.get_fill_color() == manimlib.RED
    for row in integer_tex_matrix.mob_matrix
    for member in row[0].family_members_with_points()
)
assert all(
    member.get_fill_color() == manimlib.YELLOW
    for row in integer_tex_matrix.mob_matrix
    for member in row[1].family_members_with_points()
)
matrix_copy = integer_tex_matrix.copy()
assert matrix_copy is not integer_tex_matrix
assert matrix_copy.mob_matrix[0][0] is matrix_copy.submobjects[0]
assert matrix_copy.mob_matrix[0][0] is not integer_tex_matrix.mob_matrix[0][0]
assert matrix_copy.brackets[-1] is matrix_copy.submobjects[-1]
background_matrix = matrix_module.Matrix([["x"]])
assert background_matrix.add_background_to_entries() is background_matrix
assert background_matrix.elements[0].background_rectangle is (
    background_matrix.elements[0].submobjects[0]
)

float_matrix = matrix_module.Matrix(
    [[1.25, 2.5]],
    element_config=dict(num_decimal_places=1, font_size=30),
)
assert all(isinstance(entry, manimlib.DecimalNumber) for entry in float_matrix.elements)
assert np.allclose(
    [entry.get_value() for entry in float_matrix.elements], [1.25, 2.5]
)
first_float_entry = float_matrix.elements[0]
first_float_edge = first_float_entry.get_edge_center(manimlib.LEFT).copy()
assert first_float_entry.set_value(9.5) is first_float_entry
assert float_matrix.elements[0] is first_float_entry
assert np.isclose(first_float_entry.get_value(), 9.5)
assert np.allclose(
    first_float_entry.get_edge_center(manimlib.LEFT), first_float_edge, atol=1e-6
)
mapped_tex_entry = integer_tex_matrix.element_to_mobject(0)
assert isinstance(mapped_tex_entry, manimlib.Tex)
assert mapped_tex_entry.get_tex() == "0"
assert mapped_tex_entry.family_members_with_points()
assert all(
    member.get_fill_color() == manimlib.BLUE
    for member in mapped_tex_entry.family_members_with_points()
)
mapped_decimal_entry = float_matrix.element_to_mobject(7.25)
assert isinstance(mapped_decimal_entry, manimlib.DecimalNumber)
assert mapped_decimal_entry.get_value() == 7.25
assert mapped_decimal_entry.family_members_with_points()
passthrough_entry = manimlib.Circle()
assert integer_tex_matrix.element_to_mobject(passthrough_entry) is passthrough_entry
mapped_complex_entry = integer_tex_matrix.element_to_mobject(1 + 0j)
# fm-5wq.4.108 retarget: complex entries route through DecimalNumber — the
# zero-imag value takes the fm-5wq.4.80 hide-zero real path, never
# Tex(str(...))'s "(1+0j)" spelling; the element_config color still lands
# on the rendered glyphs.
assert isinstance(mapped_complex_entry, manimlib.DecimalNumber)
assert mapped_complex_entry.get_value() == (1 + 0j)
assert mapped_complex_entry.family_members_with_points()
assert all(
    member.get_fill_color() == manimlib.BLUE
    for member in mapped_complex_entry.family_members_with_points()
)
# General complex entries compose native numeric components, not Tex(str(...)).
general_complex_entry = integer_tex_matrix.element_to_mobject(1 + 2j)
assert isinstance(general_complex_entry, manimlib.DecimalNumber)
assert general_complex_entry.get_value() == 1 + 2j
assert general_complex_entry.get_tex() == "1.00+2.00i"
assert general_complex_entry.family_members_with_points()
try:
    matrix_module.Matrix.element_to_mobject(object(), 0)
except TypeError as error:
    assert str(error) == "Matrix.element_to_mobject requires a Matrix instance"
else:
    raise AssertionError("Matrix.element_to_mobject accepted a non-Matrix self")
mixed_matrix = matrix_module.Matrix(
    [[1.25, "x"], [2, 3.5]],
    element_config=dict(num_decimal_places=1, font_size=30),
)
assert [type(entry) for entry in mixed_matrix.elements] == [
    manimlib.DecimalNumber,
    manimlib.Tex,
    manimlib.Tex,
    manimlib.DecimalNumber,
]
assert mixed_matrix.mob_matrix[0][0].get_value() == 1.25
assert mixed_matrix.mob_matrix[0][1].get_tex() == "x"
assert mixed_matrix.mob_matrix[1][0].get_tex() == "2"
assert mixed_matrix.mob_matrix[1][1].get_value() == 3.5
assert all(entry.family_members_with_points() for entry in mixed_matrix.elements)
decimal_matrix = matrix_module.DecimalMatrix(
    [[1, 2.5], [3.75, 4]],
    num_decimal_places=2,
    decimal_config=dict(font_size=22, color=manimlib.GREEN),
    height=2.5,
)
assert decimal_matrix.float_matrix == [[1, 2.5], [3.75, 4]]
assert all(
    isinstance(entry, manimlib.DecimalNumber)
    for entry in decimal_matrix.elements
)
assert np.allclose(
    [entry.get_value() for entry in decimal_matrix.elements],
    [1.0, 2.5, 3.75, 4.0],
)
assert decimal_matrix.get_height() <= 2.5 + 1e-6
integer_matrix = matrix_module.IntegerMatrix(
    [[1.2, 2.8]], decimal_config=dict(font_size=20)
)
assert all(isinstance(entry, manimlib.DecimalNumber) for entry in integer_matrix.elements)
tex_matrix = matrix_module.TexMatrix(
    [["x", "y"], ["z", "w"]],
    tex_config=dict(font_size=20, color=manimlib.RED),
    ellipses_row=0,
    ellipses_col=1,
)
assert len(tex_matrix.ellipses) == 3
assert len(tex_matrix.elements) == 1
assert tex_matrix.get_ellipses()[0] is tex_matrix.mob_matrix[0][0]
assert all(
    member.get_fill_color() == manimlib.WHITE
    for ellipse in tex_matrix.ellipses
    for member in ellipse.family_members_with_points()
)
assert all(
    member.get_fill_color() == manimlib.RED
    for entry in tex_matrix.elements
    for member in entry.family_members_with_points()
)

mobject_entries = [
    geometry.Circle(radius=0.2),
    geometry.Square(side_length=0.5),
    geometry.Rectangle(width=0.7, height=0.3),
    geometry.Circle(radius=0.35),
]
leftover_mobject_entry = geometry.Square(side_length=0.25)
mobject_matrix = matrix_module.MobjectMatrix(
    manimlib.VGroup(*mobject_entries, leftover_mobject_entry),
    n_rows=2,
    n_cols=2,
    height=1.75,
)
assert all(
    actual is original
    for actual, original in zip(
        [entry for row in mobject_matrix.mob_matrix for entry in row],
        mobject_entries,
    )
)
assert all(
    child is original
    for child, original in zip(mobject_matrix.submobjects[:4], mobject_entries)
)
assert mobject_matrix.mob_matrix[0][0].get_center()[0] < (
    mobject_matrix.mob_matrix[0][1].get_center()[0]
)
assert mobject_matrix.mob_matrix[0][0].get_center()[1] > (
    mobject_matrix.mob_matrix[1][0].get_center()[1]
)
assert mobject_matrix.get_height() <= 1.75 + 1e-6
assert all(
    child is not leftover_mobject_entry for child in mobject_matrix.submobjects
)
assert mobject_matrix.element_to_mobject(leftover_mobject_entry) is (
    leftover_mobject_entry
)

failed_mobject_matrix = matrix_module.MobjectMatrix.__new__(
    matrix_module.MobjectMatrix
)
try:
    matrix_module.MobjectMatrix.__init__(
        failed_mobject_matrix,
        manimlib.VGroup(geometry.Circle()),
        n_rows=2,
        n_cols=2,
    )
except Exception as error:
    assert str(error) == (
        "Input to MobjectMatrix must have at least n_rows * n_cols entries"
    )
else:
    raise AssertionError("MobjectMatrix accepted too few entries")
assert not hasattr(failed_mobject_matrix, "submobjects")

failed_mobject_matrix_kwarg = matrix_module.MobjectMatrix.__new__(
    matrix_module.MobjectMatrix
)
try:
    matrix_module.MobjectMatrix.__init__(
        failed_mobject_matrix_kwarg,
        manimlib.VGroup(geometry.Circle()),
        n_rows=1,
        n_cols=1,
        unsupported=True,
    )
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: unsupported"
else:
    raise AssertionError("MobjectMatrix silently discarded an unknown option")
assert not hasattr(failed_mobject_matrix_kwarg, "submobjects")

for invalid_matrix, expected_text in [
    ([], "at least one row"),
    ([[1], [2, 3]], "ragged matrix"),
]:
    try:
        matrix_module.Matrix(invalid_matrix)
    except ValueError as error:
        assert expected_text in str(error)
    else:
        raise AssertionError("an invalid Matrix shape reached live state")
try:
    matrix_module.Matrix(None)
except TypeError as error:
    assert str(error) == "matrix must be an iterable of row iterables"
else:
    raise AssertionError("Matrix(None) did not raise its named TypeError")
native_complex_matrix = matrix_module.Matrix([[1 + 2j]])
native_complex_cell = native_complex_matrix.get_entries()[0]
assert isinstance(native_complex_cell, manimlib.DecimalNumber)
assert native_complex_cell.get_value() == 1 + 2j
assert native_complex_cell.get_tex() == "1.00+2.00i"
native_shape_cell = manimlib.Circle()
native_shape_matrix = matrix_module.Matrix([[native_shape_cell]])
assert native_shape_matrix.get_entries()[0] is native_shape_cell
assert native_shape_matrix.get_row(0)[0] is native_shape_cell
assert native_shape_matrix.get_column(0)[0] is native_shape_cell
try:
    matrix_module.TexMatrix([["x"]], tex_config={"template": "legacy"})
except ValueError as error:
    assert "template" in str(error)
else:
    raise AssertionError("TexMatrix silently discarded an unsupported entry option")
assert tex_matrix.swap_entries_for_ellipses(0, 0) is tex_matrix
brackets = tex_matrix.create_brackets(tex_matrix.rows, 0.25, 0.2)
assert isinstance(brackets, manimlib.VGroup)
assert len(brackets) == 2
try:
    tex_matrix.create_brackets()
except TypeError:
    pass
else:
    raise AssertionError("Matrix.create_brackets accepted missing arguments")
try:
    tex_matrix.create_mobject_matrix()
except TypeError:
    pass
else:
    raise AssertionError("Matrix.create_mobject_matrix accepted missing arguments")
assert isinstance(
    matrix_module.DecimalMatrix([[1.0]]).element_to_mobject(1.0),
    manimlib.DecimalNumber,
)
try:
    integer_tex_matrix.get_row(-1)
except IndexError as error:
    assert "2 rows" in str(error)
else:
    raise AssertionError("Matrix.get_row accepted a negative index")

failed_torus = three_dimensions.Torus.__new__(three_dimensions.Torus)
try:
    three_dimensions.Torus.__init__(failed_torus, texture_paths={})
except NotImplementedError as error:
    assert str(error) == (
        "Torus() keyword(s) not yet routed to the native builder: texture_paths"
    )
else:
    raise AssertionError("an unrouted Torus keyword reached the native builder")
assert not hasattr(failed_torus, "submobjects")

try:
    three_dimensions.Cone((0, math.tau), (0, 1), 7)
except TypeError as error:
    assert str(error) == (
        "Cylinder.__init__() got multiple values for argument 'u_range'"
    )
else:
    raise AssertionError("Cone accepted impossible Reference positional forwarding")

old_tex = importlib.import_module("manimlib.mobject.svg.old_tex_mobject").OldTex
quadratic_label = old_tex(
    "-{b \\over 2} \\pm \\sqrt{{b^2 \\over 4} - c}",
    font_size=30,
)[0]
assert len(quadratic_label) > 12, len(quadratic_label)
assert issubclass(InteractiveScene, Scene)
assert issubclass(Animation, object)
assert issubclass(manimlib.Group, Mobject)
assert issubclass(manimlib.VGroup, Mobject)
assert issubclass(manimlib.Axes, manimlib.VGroup)
assert issubclass(manimlib.EventType, enum.Enum)
assert manimlib.EventType.KeyPressEvent.value == "key_press_event"
event_handler = importlib.import_module("manimlib.event_handler")
event_dispatcher_mod = importlib.import_module(
    "manimlib.event_handler.event_dispatcher"
)
event_listener_mod = importlib.import_module(
    "manimlib.event_handler.event_listner"
)
assert isinstance(event_handler.EVENT_DISPATCHER, event_dispatcher_mod.EventDispatcher)
# event_handler/__init__.py re-exports the class itself.
assert event_handler.EventDispatcher is event_dispatcher_mod.EventDispatcher
assert str(inspect.signature(event_listener_mod.EventListener)) == (
    "(mobject, event_type, event_callback)"
)
event_dot = geometry.Dot().shift(manimlib.RIGHT)
event_seen = []

def _on_press(mobject, event_data):
    event_seen.append((mobject, dict(event_data)))

press_listener = event_listener_mod.EventListener(
    event_dot, manimlib.EventType.MousePressEvent, _on_press
)
dispatcher = event_dispatcher_mod.EventDispatcher()
dispatcher.add_listner(press_listener)
assert dispatcher.get_listners_count() == 1
assert dispatcher.add_listener is dispatcher.add_listner
dispatcher.dispatch(
    manimlib.EventType.MouseMotionEvent, point=manimlib.RIGHT
)
dispatcher.dispatch(
    manimlib.EventType.MousePressEvent, point=manimlib.RIGHT
)
assert event_seen[0][0] is event_dot
assert np.allclose(dispatcher.get_mouse_point(), manimlib.RIGHT)
dispatcher.dispatch(manimlib.EventType.KeyPressEvent, symbol=32)
assert dispatcher.is_key_pressed(32)
dispatcher.remove_listner(press_listener)
assert dispatcher.get_listners_count() == 0
try:
    event_listener_mod.EventListener(
        object(), manimlib.EventType.MousePressEvent, _on_press
    )
except TypeError as error:
    assert str(error) == "EventListener mobject must be a Mobject"
else:
    raise AssertionError("EventListener accepted a non-Mobject")


def _check_event_dispatch_geometry_and_capture():
    # fm-5wq.4.143: spatial witnesses over native geometry. A fan-out that
    # ignores the hit test calls both squares and fails the first assertion.
    event_type = manimlib.EventType
    listener_type = event_listener_mod.EventListener
    local = event_dispatcher_mod.EventDispatcher()
    assert isinstance(local.get_mouse_point(), np.ndarray)
    assert isinstance(local.get_mouse_drag_point(), np.ndarray)
    assert np.array_equal(local.get_mouse_point(), np.zeros(3))
    assert np.array_equal(local.get_mouse_drag_point(), np.zeros(3))
    left = manimlib.Square().shift(3 * manimlib.LEFT)
    right = manimlib.Square().shift(3 * manimlib.RIGHT)
    hits = []
    left_motion = listener_type(
        left, event_type.MouseMotionEvent,
        lambda mob, data: hits.append("left"),
    )
    right_motion = listener_type(
        right, event_type.MouseMotionEvent,
        lambda mob, data: hits.append("right"),
    )
    assert local.add_listner(left_motion) is local
    local += right_motion
    assert len(local) == 2
    hover = left.get_center()
    assert local(event_type.MouseMotionEvent, point=hover) is None
    assert hits == ["left"]
    assert local.get_mouse_point() is hover
    local(event_type.MouseMotionEvent, point=manimlib.ORIGIN)
    assert hits == ["left"]

    # Duplicate registrations are observable; removal by a fresh equal
    # listener removes every registration, retaining the other object's.
    local += left_motion
    assert len(local) == 3
    local(event_type.MouseMotionEvent, point=hover)
    assert hits == ["left", "left", "left"]
    local -= listener_type(left, event_type.MouseMotionEvent, left_motion.callback)
    assert len(local) == 1
    assert local.remove_listner(left_motion) is local
    local(event_type.MouseMotionEvent, point=right.get_center())
    assert hits[-1] == "right"

    drags = []
    for label, target in (("left", left), ("right", right)):
        local += listener_type(
            target, event_type.MouseDragEvent,
            lambda mob, data, label=label: drags.append(label),
        )
    # Capture uses the most recent hover, as in the pinned dispatcher;
    # press/drag coordinates do not silently rewrite the hover position.
    local(event_type.MouseMotionEvent, point=hover)
    local(event_type.MousePressEvent, point=right.get_center())
    assert [item.mobject for item in local.draggable_object_listners] == [left]
    drag_point = right.get_center()
    local(event_type.MouseDragEvent, point=drag_point)
    assert drags == ["left"]
    assert local.get_mouse_drag_point() is drag_point
    assert local.get_mouse_point() is hover
    local(event_type.MouseReleaseEvent, point=drag_point)
    assert local.draggable_object_listners == []
    local(event_type.MouseDragEvent, point=drag_point)
    assert drags == ["left"]
    local(event_type.MouseMotionEvent, point=drag_point)
    local(event_type.MousePressEvent, point=drag_point)
    local(event_type.MouseDragEvent, point=hover)
    assert drags == ["left", "right"]
    local(event_type.MouseReleaseEvent, point=hover)

    # Mouse press/release/scroll use the same spatial filter. Returning
    # exact False stops later listeners; integer/NumPy false values do not.
    for mouse_event in (
        event_type.MousePressEvent, event_type.MouseReleaseEvent,
        event_type.MouseScrollEvent,
    ):
        mouse = event_dispatcher_mod.EventDispatcher()
        callbacks = []
        mouse += listener_type(
            left, mouse_event, lambda mob, data: callbacks.append("left")
        )
        mouse += listener_type(
            right, mouse_event, lambda mob, data: callbacks.append("right")
        )
        mouse(event_type.MouseMotionEvent, point=hover)
        mouse(mouse_event, point=hover)
        assert callbacks == ["left"], mouse_event

    keys = event_dispatcher_mod.EventDispatcher()
    returns = []

    def key_callback(value):
        def record(mob, data):
            returns.append(value)
            return value
        return record

    for value in (0, np.bool_(False), "last"):
        keys += listener_type(left, event_type.KeyPressEvent, key_callback(value))
    assert keys(event_type.KeyPressEvent, symbol=32) == "last"
    assert len(returns) == 3
    assert keys.is_key_pressed(32)
    assert not keys.is_key_pressed("32")
    keys(event_type.KeyReleaseEvent, symbol=32)
    assert not keys.is_key_pressed(32)
    keys(event_type.KeyReleaseEvent, symbol=32)
    returns.clear()
    keys += listener_type(left, event_type.KeyPressEvent, key_callback(False))
    keys += listener_type(right, event_type.KeyPressEvent, key_callback("unreached"))
    assert keys(event_type.KeyPressEvent, symbol=65) is False
    assert len(returns) == 4 and returns[-1] is False

    # Exercise Scene's actual global-dispatch path without a window. The
    # captured square receives a drag beyond its bounds; the distant one
    # receives neither the initial motion nor that captured drag.
    scene = Scene(drag_to_pan=False)
    scene.add(left, right)
    scene_hits = []
    registrations = []
    for label, target in (("left", left), ("right", right)):
        for kind in (event_type.MouseMotionEvent, event_type.MouseDragEvent):
            registration = listener_type(
                target, kind,
                lambda mob, data, label=label, kind=kind:
                    scene_hits.append((label, kind)),
            )
            registrations.append(registration)
            event_handler.EVENT_DISPATCHER.add_listner(registration)
    try:
        scene.on_mouse_motion(hover, manimlib.ORIGIN)
        scene.on_mouse_press(hover, 1, 0)
        scene.on_mouse_drag(drag_point, manimlib.RIGHT, 1, 0)
        scene.on_mouse_release(drag_point, 1, 0)
        scene.on_mouse_drag(drag_point, manimlib.RIGHT, 1, 0)
        assert scene_hits == [
            ("left", event_type.MouseMotionEvent),
            ("left", event_type.MouseDragEvent),
        ]
        assert np.array_equal(scene.mouse_point.get_center(), hover)
        assert np.array_equal(scene.mouse_drag_point.get_center(), drag_point)
    finally:
        for registration in registrations:
            event_handler.EVENT_DISPATCHER.remove_listner(registration)
        event_handler.EVENT_DISPATCHER.dispatch(
            event_type.MouseReleaseEvent, point=drag_point
        )


_check_event_dispatch_geometry_and_capture()
assert issubclass(manimlib.EndScene, Exception)
assert manimlib.np is np

# Installing the Reference's optional OpenGL/window packages beside the wheel
# must not make the Rust portal initialize a second renderer or require an X11
# display.  Their leaked names remain import-compatible, but calling one or
# reading an attribute (a class body's `moderngl.TRIANGLE_STRIP`) names its
# excluded ruling.
for refused_module, refused_name, refused_origin, refused_oot in (
    ("manimlib.camera.camera", "gl", "OpenGL.GL", "OOT-MODERNGL-SHADER-SURFACE"),
    ("manimlib.camera.camera", "moderngl", "moderngl", "OOT-MODERNGL-SHADER-SURFACE"),
    ("manimlib.mobject.interactive", "PygletWindowKeys", "pyglet.window.key", "OOT-PYGLET-WINDOW-KEYS"),
    ("manimlib.window", "Timer", "moderngl_window.timers.clock.Timer", "OOT-PYGLET-WINDOW-SURFACE"),
    ("manimlib.window", "mglw", "moderngl_window", "OOT-PYGLET-WINDOW-SURFACE"),
    ("manimlib.window", "screeninfo", "screeninfo", "OOT-PYGLET-WINDOW-SURFACE"),
):
    refused_value = getattr(importlib.import_module(refused_module), refused_name)
    for refused_use in (lambda: refused_value(), lambda: refused_value.TRIANGLE_STRIP):
        try:
            refused_use()
        except NotImplementedError as error:
            assert str(error) == (
                f"{refused_module}.{refused_name} ({refused_origin}) is excluded "
                f"({refused_oot}): FrankenManim renders through Lumen and windows "
                "through Studio"
            ), error
        else:
            raise AssertionError(
                f"Reference renderer leak {refused_module}.{refused_name} became live"
            )
window_mod = importlib.import_module("manimlib.window")
pyglet_window = window_mod.PygletWindow
assert isinstance(pyglet_window, type)
try:
    pyglet_window()
except bridge_errors.CapabilityError as error:
    assert str(error) == (
        "the Reference's PygletWindow gateway is unavailable; "
        "FrankenManim Studio owns interactive windows"
    )
else:
    raise AssertionError("the Reference PygletWindow gateway became live")
assert window_mod.Window.__bases__ == (window_mod.PygletWindow,)
assert str(inspect.signature(window_mod.Window)) == (
    "(scene=None, position_string='UR', monitor_index=1, "
    "full_screen=False, size=None, position=None, samples=0)"
)
assert window_mod.Window.cursor is True
assert window_mod.Window.fullscreen is False
assert window_mod.Window.gl_version == (3, 3)
assert window_mod.Window.resizable is True
assert window_mod.Window.vsync is True
try:
    window_mod.Window()
except bridge_errors.CapabilityError as error:
    assert str(error) == (
        "the Reference window gateway is unavailable; "
        "FrankenManim Studio owns interactive windows"
    )
else:
    raise AssertionError("the Reference Window gateway became live")
failed_window = window_mod.Window.__new__(window_mod.Window)
try:
    window_mod.Window.__init__(failed_window, Scene())
except bridge_errors.CapabilityError as error:
    assert str(error) == (
        "the Reference window gateway is unavailable; "
        "FrankenManim Studio owns interactive windows"
    )
else:
    raise AssertionError("Window accepted a Scene without a Studio host")

writer_mod = importlib.import_module("manimlib.scene.scene_file_writer")
assert writer_mod.SceneFileWriter.__bases__ == (object,)
writer_signature = inspect.signature(writer_mod.SceneFileWriter)
assert list(writer_signature.parameters) == [
    "scene",
    "write_to_movie",
    "subdivide_output",
    "png_mode",
    "save_last_frame",
    "movie_file_extension",
    "output_directory",
    "file_name",
    "open_file_upon_completion",
    "show_file_location_upon_completion",
    "quiet",
    "total_frames",
    "progress_description_len",
    "ffmpeg_bin",
    "video_codec",
    "pixel_format",
    "saturation",
    "gamma",
    "kwargs",
]
writer_scene = Scene()
writer = writer_mod.SceneFileWriter(
    writer_scene,
    file_name="demo",
    output_directory="/tmp/out",
    write_to_movie=True,
    open_file_upon_completion=True,
)
assert writer.scene is writer_scene
assert writer.file_name == "demo"
assert writer.get_output_file_name() == "demo"
assert writer.get_output_file_rootname() == "/tmp/out/demo"
assert writer.get_image_file_path() == "/tmp/out/demo.png"
assert writer.get_movie_file_path() == "/tmp/out/demo.mp4"
assert writer.get_insert_file_path(2) == "/tmp/out/demo_2.mp4"
assert writer.get_next_partial_movie_path() == (
    "/tmp/out/partial_movie_files/demo/00000.mp4"
)
assert writer.should_open_file() is True
assert writer.has_progress_display() is True
assert writer.use_fast_encoding() is True
default_writer = writer_mod.SceneFileWriter(Scene())
assert default_writer.get_output_file_name() == "Scene"
assert default_writer.get_image_file_path() == "Scene.png"
assert default_writer.has_progress_display() is False
try:
    writer.open_movie_pipe("/tmp/out/demo.mp4")
except bridge_errors.CapabilityError as error:
    assert "ffmpeg Reel boundary" in str(error)
else:
    raise AssertionError("SceneFileWriter opened a movie pipe")
try:
    writer_mod.SceneFileWriter(manimlib.Square())
except TypeError as error:
    assert str(error) == "SceneFileWriter scene must be a Scene"
else:
    raise AssertionError("SceneFileWriter accepted a non-Scene")
failed_writer = writer_mod.SceneFileWriter.__new__(writer_mod.SceneFileWriter)
try:
    writer_mod.SceneFileWriter.__init__(failed_writer, writer_scene, unsupported=True)
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: unsupported"
else:
    raise AssertionError("SceneFileWriter silently discarded an unknown option")

shader_mod = importlib.import_module("manimlib.shader_wrapper")
assert shader_mod.ShaderWrapper.__bases__ == (object,)
assert shader_mod.VShaderWrapper.__bases__ == (shader_mod.ShaderWrapper,)
assert list(inspect.signature(shader_mod.ShaderWrapper).parameters) == [
    "ctx",
    "vert_data",
    "shader_folder",
    "mobject_uniforms",
    "texture_paths",
    "depth_test",
    "render_primitive",
    "code_replacements",
]
assert list(inspect.signature(shader_mod.VShaderWrapper).parameters) == [
    "ctx",
    "vert_data",
    "shader_folder",
    "mobject_uniforms",
    "texture_paths",
    "depth_test",
    "render_primitive",
    "code_replacements",
    "program_type",
    "stroke_behind",
]
_shader_error = (
    "the Reference OpenGL ShaderWrapper is excluded; "
    "Lumen owns rasterization and custom GLSL is outside the "
    "compatibility claim"
)
try:
    shader_mod.ShaderWrapper(object(), np.zeros((0, 1)))
except bridge_errors.CapabilityError as error:
    assert str(error) == _shader_error
else:
    raise AssertionError("ShaderWrapper constructed an OpenGL program")
try:
    shader_mod.VShaderWrapper(object(), np.zeros((0, 1)), stroke_behind=True)
except bridge_errors.CapabilityError as error:
    assert str(error) == _shader_error
else:
    raise AssertionError("VShaderWrapper constructed an OpenGL program")
failed_shader = shader_mod.ShaderWrapper.__new__(shader_mod.ShaderWrapper)
try:
    shader_mod.ShaderWrapper.__init__(failed_shader, object(), np.zeros((0, 1)))
except bridge_errors.CapabilityError as error:
    assert str(error) == _shader_error
else:
    raise AssertionError("ShaderWrapper.__init__ did not refuse a context")

tex_file_writing_mod = importlib.import_module(
    "manimlib.utils.tex_file_writing"
)
assert tex_file_writing_mod.LatexError.__bases__ == (Exception,)
assert manimlib.LatexError is tex_file_writing_mod.LatexError
try:
    raise tex_file_writing_mod.LatexError("native typesetting failed")
except tex_file_writing_mod.LatexError as error:
    assert isinstance(error, Exception)
    assert error.args == ("native typesetting failed",)
    assert str(error) == "native typesetting failed"
else:
    raise AssertionError("LatexError did not preserve Exception raise semantics")

assert not any(
    name == root or name.startswith(root + ".")
    for name in sys.modules
    for root in ("OpenGL", "moderngl", "moderngl_window", "pyglet", "screeninfo")
), sorted(
    name
    for name in sys.modules
    if name.split(".", 1)[0]
    in {"OpenGL", "moderngl", "moderngl_window", "pyglet", "screeninfo"}
)
assert hasattr(Mobject, "add_event_listner")
assert Mobject.add_event_listener is Mobject.add_event_listner
assert not hasattr(manimlib, "__all__"), repr(getattr(manimlib, "__all__", None))

exported = set()
classes = set()
in_symbols = False
for raw_line in manimlib._API_SCHEMA_TSV.splitlines():
    line = raw_line.strip()
    if line.startswith("[") and line.endswith("]"):
        in_symbols = line == "[symbols]"
        continue
    if not in_symbols or not line or line.startswith("#"):
        continue
    columns = raw_line.split("\t", 5)
    if len(columns) != 6:
        continue
    module_name, name, kind, _origin, is_exported, _detail = columns
    if kind == "class" and "." not in name:
        classes.add((module_name, name))
    if is_exported == "1" and "." not in name:
        exported.add(name)
missing = sorted(name for name in exported if not hasattr(manimlib, name))
assert not missing, missing[:20]
assert len(classes) >= 250
assert len(exported) >= 600
for module_name, name in classes:
    candidate = getattr(importlib.import_module(module_name), name)
    assert isinstance(candidate, type), (module_name, name, candidate)
    types.new_class(f"_SubclassProbe_{name}", (candidate,))
actual = {name for name in vars(manimlib) if not name.startswith("_")}
assert actual == exported, sorted(actual ^ exported)[:20]


# ---------------------------------------------------------------------------
# fm-zoi (W10 binding tax, §15.2 Rev 4): method-resolution cache, batched
# crossings (rung 1), CrossingStats instrumentation, and GIL release.
# All assertions are count/state based — no wall-clock thresholds.
# ---------------------------------------------------------------------------

# The opt-in rung-1 API exists beside the always-correct rung 0.
assert callable(Scene.update)
assert callable(Scene.update_batched)


# METHOD-RESOLUTION CACHE — correct under Python subclass mutation.
class CacheProbe(Mobject):
    def init_points(self):
        self.seen = []
        self.resize(1)
        self.set_field("point", 0, [0.0, 0.0, 0.0])

    def _dispatch_updater(self, updater, dt):
        self.seen.append("original")
        return super()._dispatch_updater(updater, dt)


cache_scene = Scene()
probe = CacheProbe()
cache_scene.add(probe)
probe_ticks = []
probe.add_updater(lambda mob, dt: probe_ticks.append(dt), call=False)
manimlib._method_cache_reset()
cache_scene.update(0.25)  # first dispatch resolves and caches
assert probe.seen == ["original"]
assert probe_ticks == [0.25]
cache_stats_first = manimlib._method_cache_stats()
assert cache_stats_first["misses"] >= 1
cache_scene.update(0.25)  # second dispatch hits the cache
assert probe.seen == ["original", "original"]
cache_stats_second = manimlib._method_cache_stats()
assert cache_stats_second["hits"] > cache_stats_first["hits"]


def patched_dispatch(self, updater, dt):
    self.seen.append("patched")
    return Mobject._dispatch_updater(self, updater, dt)


CacheProbe._dispatch_updater = patched_dispatch  # class mutation
cache_scene.update(0.25)
assert probe.seen == ["original", "original", "patched"]
cache_stats_patched = manimlib._method_cache_stats()
assert cache_stats_patched["invalidations"] > cache_stats_second["invalidations"]
assert probe_ticks == [0.25, 0.25, 0.25]


# Base-class mutation recursively invalidates subclass cache entries.
class BaseProbe(Mobject):
    def init_points(self):
        self.resize(1)
        self.set_field("point", 0, [0.0, 0.0, 0.0])


base_scene = Scene()
base_probe = BaseProbe()
base_scene.add(base_probe)
base_ticks = []
base_probe.add_updater(lambda mob, dt: base_ticks.append(dt), call=False)
base_scene.update(0.5)  # caches (BaseProbe, _dispatch_updater) → Mobject's
assert base_ticks == [0.5]
saved_mobject_dispatch = Mobject._dispatch_updater
base_calls = []


def base_patched(self, updater, dt):
    base_calls.append(dt)
    return saved_mobject_dispatch(self, updater, dt)


Mobject._dispatch_updater = base_patched
try:
    base_scene.update(0.5)
    assert base_calls == [0.5]
    assert base_ticks == [0.5, 0.5]
finally:
    Mobject._dispatch_updater = saved_mobject_dispatch


# Instance-__dict__ shadowing still wins over the class-level cache.
shadow = BaseProbe()
shadow_scene = Scene()
shadow_scene.add(shadow)
shadow_ticks = []
shadow.add_updater(lambda mob, dt: shadow_ticks.append(dt), call=False)
shadow._dispatch_updater = lambda updater, dt: shadow_ticks.append("shadow")
shadow_scene.update(1.0)
assert shadow_ticks == ["shadow"]


# BATCHED CROSSINGS (rung 1) — identical ordering and observable state with
# measurably fewer crossings than rung 0 on the same callback corpus.
def make_updater_scene(count, updaters_each):
    scene = Scene()
    mobjects = []
    for index in range(count):
        mob = Mobject()
        mob.resize(1)
        mob.set_field("point", 0, [float(index), 0.0, 0.0])

        def make_tick(step):
            def tick(m, dt):
                point = m.get_field("point", 0)
                m.set_field("point", 0, [point[0] + step * dt, point[1], point[2]])

            return tick

        for k in range(updaters_each):
            mob.add_updater(make_tick(k + 1), call=False)
        mobjects.append(mob)
    scene.add(*mobjects)
    return scene, mobjects


N_MOBS, N_UPDATERS = 6, 4
scene_rung0, mobs_rung0 = make_updater_scene(N_MOBS, N_UPDATERS)
scene_rung1, mobs_rung1 = make_updater_scene(N_MOBS, N_UPDATERS)

manimlib._crossing_stats_reset()
scene_rung0.update(1.0)
rung0 = manimlib._crossing_stats_snapshot()

manimlib._crossing_stats_reset()
scene_rung1.update_batched(1.0)
rung1 = manimlib._crossing_stats_snapshot()

assert rung0["updater_call"] == N_MOBS * N_UPDATERS
assert rung1["updater_call"] == 0
assert rung1["method_dispatch"] == 1, rung1
assert rung1["dirty_propagation"] == 1
# Field writes are inherent to the updaters and equal across rungs.
assert rung0["field_write"] == rung1["field_write"] == N_MOBS * N_UPDATERS
assert rung1["total"] < rung0["total"]
# Exact deterministic counts: rung 0 pays 24 updater crossings + 7 updater
# list snapshots (the camera frame first, then 6 drawable roots) on top of
# the 48 inherent field-I/O crossings; rung 1 pays one batch dispatch + one
# batched dirty-propagation return.
assert rung0["other"] == 1 + N_MOBS + N_MOBS * N_UPDATERS
assert rung0["total"] == (
    2 * N_MOBS * N_UPDATERS + 1 + N_MOBS + N_MOBS * N_UPDATERS
)
assert rung1["other"] == N_MOBS * N_UPDATERS
assert rung1["total"] == 2 * N_MOBS * N_UPDATERS + 2
assert rung0["python_callback_ns"] > 0
assert rung1["python_callback_ns"] > 0
# Bit-equality vs rung 0 after the frame, and updater ordering preserved:
# updater k adds (k + 1) * dt to lane 0, so the sum is 1+2+3+4 = 10.
for left, right in zip(mobs_rung0, mobs_rung1):
    assert left.get_field("point", 0) == right.get_field("point", 0)
assert mobs_rung1[0].get_field("point", 0)[0] == 10.0


# Exceptions still propagate intact through the single batched crossing.
def boom(mob, dt):
    raise KeyError("batched updater boom")


batch_exploding = Mobject()
batch_exploding.resize(1)
batch_exploding.set_field("point", 0, [0.0, 0.0, 0.0])
batch_exploding.add_updater(boom, call=False)
batch_scene = Scene()
batch_scene.add(batch_exploding)
try:
    batch_scene.update_batched(0.1)
except KeyError as error:
    assert "batched updater boom" in str(error)
else:
    raise AssertionError("batched updater exception did not propagate")


# GIL RELEASE — a Python thread makes progress (counter increments) during a
# long detached native kernel; the pass/fail signal is the counter alone.
gil_probe = manimlib._GilProbe()
gil_stop = []


def gil_spin():
    while not gil_probe.native_started():
        pass
    while not gil_stop:
        gil_probe.tick()


gil_worker = threading.Thread(target=gil_spin)
gil_worker.start()
gil_observed = gil_probe.run_native(40_000_000)
gil_stop.append(True)
gil_worker.join()
assert gil_observed > 0, "no Python progress during the detached native kernel"
assert gil_probe.observed() >= gil_observed


# W11 PORTAL CONSOLE — package/version identity, stable robot records, source
# discovery, explicit construct-only diagnostics, and a real standard-mode PNG
# sequence through Lumen + Reel. Unsupported certified/Studio contracts still
# refuse before source access; lifecycle-only success is never called render.
def run_portal_console(*arguments):
    previous = sys.argv
    stdout = io.StringIO()
    stderr = io.StringIO()
    sys.argv = ["fmn-python", *arguments]
    try:
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            code = manimlib._console_main()
    finally:
        sys.argv = previous
    return code, stdout.getvalue(), stderr.getvalue()


assert manimlib.__version__ == _expected_package_version
assert manimlib.__distribution__ == "franken-manim"
assert manimlib.__franken_manim__ is True
assert manimlib.__abi_policy__ == "cpython-3.13-full-abi"

console_code, console_out, console_err = run_portal_console("--version")
assert console_code == 0
# The version line reports the RUNNING interpreter (sys.version_info) —
# the host CPython this very suite executes under — while the wheel ABI
# policy strings above/below stay pinned to the built ABI. A hardcoded
# minor here would lie whenever the host interpreter moves.
assert console_out.startswith(
    "fmn-python "
    + _expected_package_version
    + f" (CPython {sys.version_info.major}.{sys.version_info.minor}."
), console_out
assert console_err == ""

console_code, console_out, console_err = run_portal_console("--robot", "--version")
assert console_code == 0
console_version = json.loads(console_out)
assert console_version["schema"] == "fmn-python.cli"
assert console_version["version"] == 1
assert console_version["kind"] == "version"
assert console_version["program_version"] == _expected_package_version
assert console_version["abi_policy"] == "cpython-3.13-full-abi"
assert console_version["exit"] == {"code": 0, "identity": "success"}
assert console_err == ""

# Certified refusal precedes even source-file access: partial certified pixels
# without the content closure and provenance sidecar are not certification.
console_code, console_out, console_err = run_portal_console(
    "--reproducible", "missing-source.py", "MissingScene"
)
assert console_code == 4
assert console_out == ""
assert "render-capability-unavailable" in console_err
assert "input closure" in console_err

def verify_portal_console_scene():
    source = pathlib.Path(__file__).with_name("console_scene.py")
    console_code, console_out, console_err = run_portal_console(
        "--robot", "--list-scenes", str(source)
    )
    assert console_code == 0
    scene_list = json.loads(console_out)
    assert scene_list["kind"] == "scene-list"
    assert scene_list["scenes"] == ["Hello"]
    assert console_err == ""

    console_code, console_out, console_err = run_portal_console(
        "--robot", "--construct-only", str(source), "Hello"
    )
    assert console_code == 0
    constructed = json.loads(console_out)
    assert constructed["kind"] == "construct-only"
    assert constructed["scene"] == "Hello"
    assert constructed["root_count"] == 3
    assert constructed["family_count"] > constructed["root_count"]
    assert constructed["scene_time"] == 3.0
    assert constructed["rendered"] is False
    assert constructed["exit"] == {"code": 0, "identity": "success"}
    assert console_err == ""

    output_root = pathlib.Path(tempfile.mkdtemp(prefix="fmn-portal-render-"))
    destination = output_root / "frames"
    blocked_still_destination = output_root / "blocked.png"
    blocked_still_destination.mkdir()
    console_code, console_out, console_err = run_portal_console(
        "--robot",
        str(source),
        "Hello",
        "--format",
        "png_sequence",
        "--resolution",
        "96x54",
        "--fps",
        "30",
        "--threads",
        "1",
        "--video_dir",
        str(destination),
    )
    assert console_code == 0, console_err
    rendered = json.loads(console_out)
    assert rendered["kind"] == "render"
    assert rendered["scene"] == "Hello"
    assert rendered["format"] == "png_sequence"
    assert rendered["resolution"] == [96, 54]
    assert rendered["fps"] == 30
    assert rendered["rendered"] is True
    assert rendered["frame_count"] == 90
    assert rendered["bytes"] > 0
    assert len(rendered["digest"]) == 64
    engine_parts = rendered["engine"].split(":")
    assert len(engine_parts) == 3
    assert engine_parts[0] == "fast-cpu"
    assert engine_parts[2].isdigit()
    assert rendered["threads"] == 1
    # Receipts carry canonical paths (macOS /var -> /private/var).
    assert pathlib.Path(rendered["destination"]) == destination.resolve()
    frames = sorted(destination.glob("frame_*.png"))
    assert len(frames) == rendered["frame_count"]
    assert sum(frame.stat().st_size for frame in frames) == rendered["bytes"]
    for frame in frames:
        assert frame.read_bytes().startswith(b"\x89PNG\r\n\x1a\n")
    assert_red_orientation_witness_is_above_origin(frames[0])
    assert console_err == ""

    # Atlas already owns PNG/JPEG decode and ImageQuad construction; the
    # portal must preserve that resource across detached copy, Scene
    # adoption, and the production Lumen/Reel render rather than exposing a
    # schema shell or decoding a second Python-side raster.
    source_width, source_height, source_rows = png_rgba8_rows(frames[0])
    image = manimlib.ImageMobject(str(frames[0]), height=2.0)
    assert image.data.dtype.names == ("point", "im_coords", "opacity")
    assert image.get_num_points() == 6
    assert math.isclose(image.get_height(), 2.0, rel_tol=0.0, abs_tol=1e-6)
    assert math.isclose(
        image.get_width(),
        2.0 * source_width / source_height,
        rel_tol=0.0,
        abs_tol=1e-6,
    )
    assert np.array_equal(
        image.data["im_coords"],
        np.array([(0, 0), (0, 1), (1, 0), (1, 1), (1, 0), (0, 1)]),
    )
    source_top_left = np.array(source_rows[0][:3], dtype=float) / 255.0
    assert np.allclose(
        image.point_to_rgb(image.get_corner(manimlib.UL)),
        source_top_left,
        rtol=0.0,
        atol=1e-12,
    )
    try:
        image.point_to_rgb(image.get_right() + manimlib.RIGHT)
    except ValueError as error:
        assert "outside an image" in str(error)
    else:
        raise AssertionError("ImageMobject sampled outside its live quad")
    before_color = image.data.copy()
    assert image.set_color(manimlib.RED, opacity=0.1) is image
    assert np.array_equal(image.data, before_color)
    assert image.set_opacity([0.25, 0.75]) is image
    assert math.isclose(image.data["opacity"][0, 0], 0.25)
    assert math.isclose(image.data["opacity"][-1, 0], 0.75)
    image.shift([0.25, -0.5, 0.0])
    pickled_image = pickle.loads(  # ubs:ignore -- trusted round-trip created immediately here
        pickle.dumps(image, protocol=pickle.HIGHEST_PROTOCOL)
    )
    for image_clone in (image.copy(), copy.deepcopy(image), pickled_image):
        assert type(image_clone) is type(image)
        assert image_clone._image_dimensions() == (source_width, source_height)
        assert np.array_equal(image_clone.data, image.data)
        assert np.allclose(image_clone.get_center(), image.get_center())
        assert np.allclose(
            image_clone.point_to_rgb(image_clone.get_corner(manimlib.UL)),
            image.point_to_rgb(image.get_corner(manimlib.UL)),
            rtol=0.0,
            atol=1e-12,
        )

    # Extension-less local lookup reaches the same native decoder. A URL is
    # a named capability refusal and a non-image file fails in Atlas before a
    # usable object can escape construction.
    extensionless = manimlib.ImageMobject(
        str(frames[0].with_suffix("")), height=1.0
    )
    assert (extensionless.pixel_width, extensionless.pixel_height) == (
        source_width,
        source_height,
    )
    try:
        manimlib.ImageMobject("https://example.invalid/image.png")
    except bridge_errors.CapabilityError as error:
        assert "AssetFetcher" in str(error)
    else:
        raise AssertionError("ImageMobject silently accepted a network URL")
    try:
        manimlib.ImageMobject(str(source))
    except ValueError as error:
        assert "not a recognized image" in str(error)
    else:
        raise AssertionError("ImageMobject accepted non-image bytes")

    image_destination = output_root / "image-mobject.png"
    image_scene = InteractiveScene()
    image_scene._begin_png(str(image_destination), 160, 90, 30, 1, 0)
    # Fill the camera height so nearest-neighbour sampling cannot legitimately
    # skip the source's deliberately small orientation witness.
    rendered_image = image.copy().set_opacity(1.0).set_height(8.0)
    assert rendered_image._image_dimensions() == (source_width, source_height)
    image_scene.add(rendered_image)
    image_receipt = image_scene._finish_render(
        image_scene.frame._core,
        image_scene.camera.light_source.get_center(),
    )
    assert pathlib.Path(image_receipt[0]) == image_destination
    assert image_receipt[1] == 1
    assert image_receipt[2] == image_destination.stat().st_size
    assert len(image_receipt[3]) == 64
    assert image_receipt[4].split(":")[0] == "fast-cpu"
    rendered_width, rendered_height, rendered_rows = png_rgba8_rows(
        image_destination
    )
    assert (rendered_width, rendered_height) == (160, 90)
    assert sum(
        tuple(row[offset : offset + 3]) != (51, 51, 51)
        for row in rendered_rows
        for offset in range(0, len(row), 4)
    ) >= 8

    # The independently unpickled image carries the same ImageResource,
    # ImageQuad primitive, placement, and opacity field into production
    # composition. Canonical PNG equality is the end-to-end witness.
    pickled_image_destination = output_root / "pickled-image-mobject.png"
    pickled_image_scene = InteractiveScene()
    pickled_image_scene._begin_png(
        str(pickled_image_destination), 160, 90, 30, 1, 0
    )
    pickled_rendered_image = pickled_image.copy().set_opacity(1.0).set_height(8.0)
    pickled_image_scene.add(pickled_rendered_image)
    pickled_image_receipt = pickled_image_scene._finish_render(
        pickled_image_scene.frame._core,
        pickled_image_scene.camera.light_source.get_center(),
    )
    assert pathlib.Path(pickled_image_receipt[0]) == pickled_image_destination
    assert pickled_image_receipt[1] == 1
    assert pickled_image_destination.read_bytes() == image_destination.read_bytes()

    # Same build, policy, seed, and thread count reproduce the exact ordered
    # PNG-tree digest through an independently published generation.
    repeated_destination = output_root / "repeated"
    console_code, console_out, console_err = run_portal_console(
        "--robot",
        str(source),
        "Hello",
        "--resolution=96x54",
        "--fps=30",
        "--threads=1",
        f"--video_dir={repeated_destination}",
    )
    assert console_code == 0, console_err
    repeated = json.loads(console_out)
    assert hmac.compare_digest(repeated["digest"], rendered["digest"])
    assert repeated["bytes"] == rendered["bytes"]
    assert repeated["frame_count"] == rendered["frame_count"]

    # Final-state PNG uses the same production composition root but skips
    # intermediate raster captures. It must publish exactly one atomic file.
    still_destination = output_root / "final.png"
    console_code, console_out, console_err = run_portal_console(
        "--robot",
        str(source),
        "Hello",
        "--format",
        "png",
        "--resolution",
        "96x54",
        "--fps",
        "30",
        "--threads",
        "1",
        "--video_dir",
        str(still_destination),
    )
    assert console_code == 0, console_err
    still = json.loads(console_out)
    assert still["kind"] == "render"
    assert still["format"] == "png"
    assert still["frame_count"] == 1
    assert still["bytes"] == still_destination.stat().st_size
    assert len(still["digest"]) == 64
    assert still["engine"].split(":")[0] == "fast-cpu"
    assert still["threads"] == 1
    assert pathlib.Path(still["destination"]) == still_destination
    still_bytes = still_destination.read_bytes()
    assert still_bytes.startswith(b"\x89PNG\r\n\x1a\n")
    assert_red_orientation_witness_is_above_origin(still_destination)
    assert console_err == ""

    repeated_still_destination = output_root / "repeated-final.png"
    console_code, console_out, console_err = run_portal_console(
        "--robot",
        str(source),
        "Hello",
        "--format=png",
        "--resolution=96x54",
        "--fps=30",
        "--threads=1",
        f"--video_dir={repeated_still_destination}",
    )
    assert console_code == 0, console_err
    repeated_still = json.loads(console_out)
    assert repeated_still["frame_count"] == 1
    assert hmac.compare_digest(repeated_still["digest"], still["digest"])
    assert repeated_still_destination.read_bytes() == still_bytes

    # Direct public Surface partial reveal is production behavior, not only a
    # structural bridge assertion: its collapsed native UV grid must survive
    # Scene adoption and reach visible Lumen/Reel pixels.
    surface_destination = output_root / "surface-partial.png"
    surface_render_scene = InteractiveScene()
    surface_render_scene._begin_png(
        str(surface_destination), 160, 90, 30, 1, 0
    )
    surface_render_source = manimlib.ParametricSurface(
        lambda u, v: np.array([u, v, 0.0]),
        u_range=(-3.0, 3.0),
        v_range=(-2.0, 2.0),
        resolution=(11, 9),
        color=manimlib.RED,
        opacity=1.0,
    )
    surface_render_target = surface_render_source.copy()
    assert (
        surface_render_target.pointwise_become_partial(
            surface_render_source, 0.2, 0.8, axis=0
        )
        is surface_render_target
    )
    surface_render_scene.add(surface_render_target)
    surface_receipt = surface_render_scene._finish_render(
        surface_render_scene.frame._core,
        surface_render_scene.camera.light_source.get_center(),
    )
    assert pathlib.Path(surface_receipt[0]) == surface_destination
    assert surface_receipt[1] == 1
    assert surface_receipt[2] == surface_destination.stat().st_size
    assert len(surface_receipt[3]) == 64
    assert surface_receipt[4].split(":")[0] == "fast-cpu"
    surface_width, surface_height, surface_rows = png_rgba8_rows(
        surface_destination
    )
    assert (surface_width, surface_height) == (160, 90)
    assert sum(
        tuple(row[offset : offset + 3]) != (51, 51, 51)
        for row in surface_rows
        for offset in range(0, len(row), 4)
    ) >= 32

    # The public ThreeDModel Group carries Atlas's triangle mesh through Scene
    # adoption into production Lumen/Reel pixels; constructor-only success is
    # not accepted as a close for this schema row.
    model_destination = output_root / "three-d-model.png"
    model_render_scene = InteractiveScene()
    model_render_scene._begin_png(
        str(model_destination), 160, 90, 30, 1, 0
    )
    rendered_model = native_three_d_model.copy().scale(1.8)
    model_render_scene.add(rendered_model)
    model_receipt = model_render_scene._finish_render(
        model_render_scene.frame._core,
        model_render_scene.camera.light_source.get_center(),
    )
    assert pathlib.Path(model_receipt[0]) == model_destination
    assert model_receipt[1] == 1
    assert model_receipt[2] == model_destination.stat().st_size
    assert len(model_receipt[3]) == 64
    assert model_receipt[4].split(":")[0] == "fast-cpu"
    model_width, model_height, model_rows = png_rgba8_rows(model_destination)
    assert (model_width, model_height) == (160, 90)
    assert sum(
        tuple(row[offset : offset + 3]) != (51, 51, 51)
        for row in model_rows
        for offset in range(0, len(row), 4)
    ) >= 32

    # A generic open VMobject remains visibly stroked after ShowCreation,
    # subsequent camera animation, and an ambient-rotation wait.  This is the
    # minimal production form of SquareOnASphere's three elbow marks: the
    # object survives structurally either way, so only decoded pixels catch a
    # zero-filled first-allocation opacity lane.
    elbow_destination = output_root / "camera-elbow.png"
    elbow_scene = InteractiveScene()
    elbow_scene._begin_png(str(elbow_destination), 160, 90, 30, 1, 0)
    elbow_frame = elbow_scene.frame
    elbow_frame.reorient(12, 70, 0, manimlib.ORIGIN, 4)
    elbow = VMobject().set_points_as_corners(
        [[-1.0, -0.5, 0.0], [0.0, -0.5, 0.0], [0.0, 0.75, 0.0]]
    )
    elbow.set_stroke(manimlib.WHITE, 4)
    elbow_scene.play(
        manimlib.ShowCreation(elbow, time_span=(0, 1 / 30)),
        elbow_frame.animate.reorient(20, 60, 0),
        run_time=2 / 30,
        rate_func=manimlib.linear,
    )
    elbow_scene.play(
        elbow_frame.animate.reorient(6, 84, 0),
        run_time=1 / 30,
        rate_func=manimlib.linear,
    )
    elbow_frame.add_ambient_rotation(2 * manimlib.DEG)
    elbow_scene.wait(1 / 30)
    assert np.isclose(
        elbow_frame.get_theta(),
        (6 + 2 / 30) * manimlib.DEG,
        rtol=0.0,
        atol=1e-12,
    )
    assert np.allclose(elbow.data["stroke_rgba"][:, 3], 1.0)
    elbow_receipt = elbow_scene._finish_render(
        elbow_frame._core,
        elbow_scene.camera.light_source.get_center(),
    )
    assert pathlib.Path(elbow_receipt[0]) == elbow_destination
    assert elbow_receipt[1] == 1
    assert elbow_receipt[2] == elbow_destination.stat().st_size
    assert len(elbow_receipt[3]) == 64
    assert elbow_receipt[4].split(":")[0] == "fast-cpu"
    assert elbow_receipt[5] == 1
    assert_white_stroke_witness(elbow_destination)

    # The camera frame is intentionally not a drawable Stage root, but the
    # Reference places it first in Scene.mobjects for updater dispatch.  Both
    # crossing rungs, wait, ordinary play, and camera-only play must preserve
    # frame-before-root ordering.  The camera-only observation also proves
    # frame interpolation lands before the updater phase rather than inside
    # the later capture callback.
    updater_scene = InteractiveScene()
    updater_frame = updater_scene.frame
    updater_root = manimlib.Dot()
    updater_scene.add(updater_root)
    updater_events = []

    def record_frame_updater(mobject, dt):
        del mobject
        updater_events.append(("frame", dt))

    def record_root_updater(mobject, dt):
        del mobject
        updater_events.append(("root", dt))

    updater_frame.add_updater(record_frame_updater)
    updater_root.add_updater(record_root_updater)
    updater_events.clear()
    updater_scene.update_batched(0.125)
    assert updater_events == [("frame", 0.125), ("root", 0.125)]

    updater_events.clear()
    updater_scene.wait(1 / 30)
    assert [kind for kind, _ in updater_events] == [
        "frame",
        "root",
        "frame",
        "root",
    ]
    assert np.allclose(
        [dt for _, dt in updater_events],
        [0.0, 0.0, 1 / 30, 1 / 30],
        rtol=0.0,
        atol=1e-15,
    )

    updater_events.clear()
    updater_scene.play(
        updater_root.animate.shift(manimlib.RIGHT),
        run_time=1 / 30,
        rate_func=manimlib.linear,
    )
    assert [kind for kind, _ in updater_events] == [
        "frame",
        "root",
        "frame",
        "root",
    ]
    assert np.allclose(
        [dt for _, dt in updater_events],
        [1 / 30, 1 / 30, 0.0, 0.0],
        rtol=0.0,
        atol=1e-15,
    )

    camera_only_scene = InteractiveScene()
    camera_only_frame = camera_only_scene.frame
    camera_observations = []

    def observe_camera_updater(mobject, dt):
        camera_observations.append((mobject.get_theta(), dt))

    camera_only_frame.add_updater(observe_camera_updater)
    camera_observations.clear()
    camera_only_scene.play(
        camera_only_frame.animate.reorient(30, 70, 0),
        run_time=1 / 30,
        rate_func=manimlib.linear,
    )
    assert len(camera_observations) == 2
    assert np.allclose(
        [theta for theta, _ in camera_observations],
        [30 * manimlib.DEG, 30 * manimlib.DEG],
        rtol=0.0,
        atol=1e-12,
    )
    assert np.allclose(
        [dt for _, dt in camera_observations],
        [1 / 30, 0.0],
        rtol=0.0,
        atol=1e-15,
    )

    # LineBrace follows the Reference's two-stage transform: flatten the
    # target around its own centre, build the brace, then restore both around
    # that same centre.  Rotating the flattened anchor around the world origin
    # instead sends translated annotations off-screen, as BeamSplitter's
    # amplitude brace demonstrated.  Keep the translated, angled geometry and
    # the adopted fixed-frame pixels in one production regression.
    brace_destination = output_root / "fixed-line-brace.png"
    brace_scene = InteractiveScene()
    # 320x180: the Reference-depth brace (fm-5wq.53) is 28% thinner than the
    # old calibration, so the white-pixel witness needs the finer grid to keep
    # its threshold; an off-screen brace still yields none.
    brace_scene._begin_png(str(brace_destination), 320, 180, 30, 1, 0)
    brace_frame = brace_scene.frame
    brace_frame.reorient(-55, 68, 0, [1.0, -1.0, 0.0], 6)
    brace_start = np.array([-2.5, -0.5, 0.0])
    brace_end = np.array([0.5, 1.0, 0.0])
    brace_line = geometry.Line(brace_start, brace_end)
    unit_tangent = (brace_end - brace_start) / np.linalg.norm(brace_end - brace_start)
    unit_normal = np.array([-unit_tangent[1], unit_tangent[0], 0.0])
    line_brace = manimlib.LineBrace(brace_line, buff=0.1).fix_in_frame()
    brace_label = line_brace.get_tex("1").fix_in_frame()
    relative_points = line_brace.get_points() - brace_line.get_center()
    tangent_projections = relative_points @ unit_tangent
    normal_projections = relative_points @ unit_normal
    assert np.isclose(tangent_projections.min(), -brace_line.get_length() / 2)
    assert np.isclose(tangent_projections.max(), brace_line.get_length() / 2)
    assert np.isclose(normal_projections.min(), 0.1, rtol=0, atol=1e-6), (
        normal_projections.min(),
        normal_projections.max(),
        line_brace.get_direction(),
    )
    assert np.dot(line_brace.get_direction(), unit_normal) > 1 - 1e-8
    assert np.dot(brace_label.get_center() - line_brace.get_tip(), unit_normal) > 0
    brace_scene.play(
        manimlib.GrowFromCenter(line_brace),
        manimlib.Write(brace_label),
        brace_frame.animate.reorient(-25, 76, 0),
        run_time=1 / 30,
        rate_func=manimlib.linear,
    )
    brace_family = {
        id(member)
        for root in brace_scene.get_mobjects()
        for member in root.get_family()
    }
    assert id(line_brace) in brace_family
    assert id(brace_label) in brace_family
    brace_receipt = brace_scene._finish_render(
        brace_frame._core,
        brace_scene.camera.light_source.get_center(),
    )
    assert pathlib.Path(brace_receipt[0]) == brace_destination
    assert brace_receipt[1] == 1
    assert brace_receipt[2] == brace_destination.stat().st_size
    assert len(brace_receipt[3]) == 64
    assert brace_receipt[4].split(":")[0] == "fast-cpu"
    assert brace_receipt[5] == 1
    assert_white_stroke_witness(brace_destination, minimum_pixels=20)

    console_code, console_out, console_err = run_portal_console(
        "--robot",
        str(source),
        "Hello",
        "--format",
        "png",
        "--resolution",
        "96x54",
        "--fps",
        "30",
        "--threads",
        "1",
        "--video_dir",
        str(blocked_still_destination),
    )
    assert console_code == 6
    blocked_still = json.loads(console_out)
    assert blocked_still["exit"] == {"code": 6, "identity": "render"}
    assert blocked_still["kind"] in ("render-failed", "render-finish-failed")
    assert blocked_still_destination.is_dir()
    assert still_destination.read_bytes() == still_bytes
    assert console_err == ""

    # Reel's atomic-directory publication is no-clobber. A second generation
    # targeting an already published root fails as render/output, and the
    # first generation remains byte-for-byte intact.
    original_frames = {frame.name: frame.read_bytes() for frame in frames}
    console_code, console_out, console_err = run_portal_console(
        "--robot",
        str(source),
        "Hello",
        "--resolution",
        "96x54",
        "--fps",
        "30",
        "--threads",
        "1",
        "--video_dir",
        str(destination),
    )
    assert console_code == 6
    clobber_error = json.loads(console_out)
    assert clobber_error["exit"] == {"code": 6, "identity": "render"}
    assert clobber_error["kind"] in ("render-failed", "render-finish-failed")
    assert {frame.name: frame.read_bytes() for frame in frames} == original_frames
    assert console_err == ""

    # The native composition method has no certified-mode parameter at all;
    # an extra mode argument cannot bypass the console's fail-closed boundary.
    direct_certified = manimlib.Scene()
    try:
        direct_certified._begin_png_sequence(
            str(output_root / "forbidden-certified"), 96, 54, 30, 1, True, 0
        )
    except TypeError as error:
        assert "positional arguments" in str(error)
    else:
        raise AssertionError("direct certified portal rendering did not refuse")

    console_code, console_out, console_err = run_portal_console(
        "--robot", "studio", "missing-source.py", "MissingScene"
    )
    assert console_code == 4
    studio_error = json.loads(console_out)
    assert studio_error["kind"] == "studio-unavailable"
    assert studio_error["exit"] == {"code": 4, "identity": "capability"}
    assert console_err == ""

    console_code, console_out, console_err = run_portal_console(
        "--robot", "--construct-only", str(source), "NoSuchScene"
    )
    assert console_code == 5
    scene_error = json.loads(console_out)
    assert scene_error["kind"] == "construct-failed"
    assert scene_error["exit"] == {"code": 5, "identity": "scene"}
    assert "NoSuchScene" in scene_error["message"]
    assert console_err == ""


verify_portal_console_scene()


# ---------------------------------------------------------------- SVGMobject
# fm-5wq.4.50 (G2 criterion 4): user SVG files load through Chisel's hardened
# document processor into a real native VMobject family — never a placeholder
# — and every rejected document is a *named* error.

svg_fixture = pathlib.Path(__file__).with_name("chisel_sample.svg")
svg_mob = manimlib.SVGMobject(str(svg_fixture))
# One pointful child per rendered shape, in document order.
assert len(svg_mob.submobjects) == 3
assert all(child.has_points() for child in svg_mob.submobjects)
assert all(child.get_num_points() > 0 for child in svg_mob.submobjects)
# The Reference's post-passes: centred, height-normalized to 2.0.
assert abs(svg_mob.get_height() - 2.0) < 1e-9
assert np.allclose(svg_mob.get_center(), [0.0, 0.0, 0.0], atol=1e-9)
svg_circle, svg_rect, svg_path = svg_mob.submobjects
# Document styles survive resolution: the circle child fills, the rect child
# does not, and the half-opacity path keeps its cascaded fill opacity.
assert svg_circle.get_fill_opacity() == 1.0
assert svg_rect.get_fill_opacity() == 0.0
assert abs(svg_path.get_fill_opacity() - 0.5) < 1e-12
# The Reference's constructor default stroke_width=0.0 overrides the
# document's strokes; stroke_width=None preserves them instead.
assert svg_rect.get_stroke_width() == 0.0
svg_kept = manimlib.SVGMobject(str(svg_fixture), stroke_width=None)
assert svg_kept.submobjects[1].get_stroke_width() == 2.0
# The y-flip: the path hugs the top of the document, so after SVG y-down
# becomes scene y-up it sits above the vertically centred circle.
assert svg_path.get_center()[1] > svg_circle.get_center()[1]
# The circle stays left of the rect (x is untouched by the flip).
assert svg_circle.get_center()[0] < svg_rect.get_center()[0]

# A Scene can play an animation on the loaded family.
svg_scene = Scene()
svg_scene.play(manimlib.FadeIn(svg_mob), run_time=2.0 / 30.0)
assert svg_mob in svg_scene.mobjects

# svg_string is the same native path without a file.
svg_inline = manimlib.SVGMobject(
    svg_string='<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">'
    '<rect x="1" y="1" width="8" height="8" fill="#FFFFFF"/></svg>'
)
assert len(svg_inline.submobjects) == 1
assert svg_inline.submobjects[0].has_points()

# Named refusals, never hangs or garbage.
try:
    manimlib.SVGMobject(svg_string="<svg><rect")
except ValueError as error:
    assert "line 1" in str(error)
else:
    raise AssertionError("malformed SVG did not refuse")

try:
    manimlib.SVGMobject(
        svg_string='<?xml version="1.0"?><!DOCTYPE svg [<!ENTITY a "aaaa">]>'
        "<svg>&a;</svg>"
    )
except ValueError as error:
    assert "DOCTYPE is refused" in str(error)
else:
    raise AssertionError("DOCTYPE entity bomb did not refuse by name")

try:
    manimlib.SVGMobject(
        svg_string="<svg>" + "<g>" * 64 + "</g>" * 64 + "</svg>"
    )
except ValueError as error:
    assert "nesting depth" in str(error)
else:
    raise AssertionError("nesting bomb did not refuse by name")

try:
    manimlib.SVGMobject(str(svg_fixture.with_name("no-such-file.svg")))
except OSError as error:
    assert "no-such-file.svg" in str(error)
else:
    raise AssertionError("a missing SVG file did not refuse")

try:
    manimlib.SVGMobject()
except Exception as error:
    assert "file_name or svg_string" in str(error)
else:
    raise AssertionError("an empty SVGMobject() did not refuse")


# ------------------------------------------------- decimal number animations
# fm-5wq.4.55: ChangingDecimal / ChangeDecimalToValue / CountInFrom drive
# DecimalNumber.set_value every frame through Scene.play.

numbers_animation = importlib.import_module("manimlib.animation.numbers")

# ChangingDecimal feeds the raw time-spanned alpha (the Reference bypasses
# rate_func here) and the displayed number tracks the update callable.
changing_scene = Scene()
changing_decimal = manimlib.DecimalNumber(0.0)
changing_scene.add(changing_decimal)
changing_alphas = []
changing_scene.play(
    numbers_animation.ChangingDecimal(
        changing_decimal,
        lambda a: (changing_alphas.append(a), 10.0 * a)[-1],
        run_time=2.0 / 30.0,
    )
)
assert changing_alphas[0] == 0.0
assert changing_alphas[-1] == 1.0
assert any(0.0 < a < 1.0 for a in changing_alphas)
assert changing_decimal.get_value() == 10.0

# ChangeDecimalToValue runs the linear track from the displayed number; a
# same-play probe observes the on-screen value moving mid-flight.
change_scene = Scene()
change_decimal = manimlib.DecimalNumber(3.0)
change_scene.add(change_decimal)
change_probe = geometry.Rectangle(width=1.0, height=1.0)
change_seen = []
change_scene.play(
    numbers_animation.ChangeDecimalToValue(
        change_decimal, 8.0, run_time=2.0 / 30.0
    ),
    update_animation.UpdateFromFunc(
        change_probe,
        lambda mob: change_seen.append(change_decimal.get_value()),
    ),
    run_time=2.0 / 30.0,
)
assert abs(change_decimal.get_value() - 8.0) < 1e-12
assert change_decimal.number == 8.0
assert len(set(change_seen)) > 1, change_seen
assert any(3.0 <= value < 8.0 for value in change_seen), change_seen

# CountInFrom counts from the source up to the number already displayed.
count_scene = Scene()
count_decimal = manimlib.DecimalNumber(5.0)
count_scene.add(count_decimal)
count_probe = geometry.Rectangle(width=1.0, height=1.0)
count_seen = []
count_scene.play(
    numbers_animation.CountInFrom(count_decimal, 0),
    update_animation.UpdateFromFunc(
        count_probe,
        lambda mob: count_seen.append(count_decimal.get_value()),
    ),
    run_time=2.0 / 30.0,
)
assert abs(count_decimal.get_value() - 5.0) < 1e-12
assert any(value < 5.0 for value in count_seen), count_seen

# The Reference's exact refusal: a non-DecimalNumber target is the bare
# isinstance assertion.
try:
    numbers_animation.ChangingDecimal(
        geometry.Rectangle(width=1.0, height=1.0), lambda a: a
    )
except AssertionError:
    pass
else:
    raise AssertionError("ChangingDecimal accepted a non-DecimalNumber")

try:
    numbers_animation.CountInFrom(
        geometry.Rectangle(width=1.0, height=1.0), 0
    )
except (AssertionError, AttributeError):
    pass
else:
    raise AssertionError("CountInFrom accepted a non-DecimalNumber")


# ------------------------------------------------------- AddTextWordByWord
# fm-5wq.4.54: word-by-word reveal over native span-map word groups.

creation_animation = importlib.import_module("manimlib.animation.creation")

word_text = manimlib.Text("hello brave world")
assert len(word_text.submobjects) == 15  # three five-glyph words
word_anim = creation_animation.AddTextWordByWord(word_text)
# The Reference's derived parameters: 0.2 s per word, three words.
assert abs(word_anim.run_time - 0.6) < 1e-12
assert word_anim.rate_func(0.25) == 0.25  # linear, not smooth

word_scene = Scene()
word_probe = geometry.Rectangle(width=1.0, height=1.0)
word_counts = []
word_scene.play(
    word_anim,
    update_animation.UpdateFromFunc(
        word_probe,
        lambda mob: word_counts.append(len(word_text.submobjects)),
    ),
    run_time=0.6,
)
# Every observed frame sits on a word boundary, the reveal only grows, at
# least one word appeared mid-flight, and the family finishes whole.
assert set(word_counts) <= {0, 5, 10, 15}, word_counts
assert word_counts == sorted(word_counts), word_counts
assert any(count in (5, 10) for count in word_counts), word_counts
assert len(word_text.submobjects) == 15

# An explicit non-negative run_time is kept, not recomputed.
assert (
    abs(
        creation_animation.AddTextWordByWord(
            manimlib.Text("two words"), run_time=1.5
        ).run_time
        - 1.5
    )
    < 1e-12
)

# The Reference's exact refusal for a non-StringMobject target.
try:
    creation_animation.AddTextWordByWord(
        geometry.Rectangle(width=1.0, height=1.0)
    )
except AssertionError:
    pass
else:
    raise AssertionError("AddTextWordByWord accepted a non-StringMobject")

# A string with no glyphs has no word groups: named, never a silent no-op.
try:
    creation_animation.AddTextWordByWord(manimlib.Text(" "))
except ValueError as error:
    assert "word groups" in str(error)
else:
    raise AssertionError("AddTextWordByWord accepted an empty string mobject")

# fm-5wq.4.53: DrawBorderThenFill and ShowPartial are native animations, not
# schema placeholders.  Bases follow the schema rows exactly.
creation = importlib.import_module("manimlib.animation.creation")
assert creation.ShowCreation.__bases__ == (creation.ShowPartial,)
assert creation.Uncreate.__bases__ == (creation.ShowCreation,)
assert creation.Write.__bases__ == (creation.DrawBorderThenFill,)
assert issubclass(creation.ShowPartial, Animation)
assert issubclass(creation.ShowPartial, abc.ABC)

# The mechanism is abstract (creation.py:25): direct instantiation refuses.
try:
    creation.ShowPartial(manimlib.Square())
except TypeError as error:
    assert "abstract" in str(error), error
else:
    raise AssertionError("ShowPartial() must refuse direct instantiation")

# Non-VMobject targets are named errors, never silent skips.
try:
    creation.DrawBorderThenFill(Mobject())
except TypeError as error:
    assert "DrawBorderThenFill" in str(error), error
    assert "Mobject" in str(error), error
else:
    raise AssertionError("DrawBorderThenFill accepted a non-VMobject")


class _CreationBoundsReveal(creation.ShowPartial):
    def get_bounds(self, alpha):
        return (0.0, alpha)


# A family that is not all VMobject or Surface plays Python-driven, as the
# Reference plays every ShowPartial. A bare Mobject has no partial form of
# its own (mobject.py's pointwise_become_partial is a no-op), so it is
# accepted and holds still. A non-Mobject target is still a named error.
assert not _CreationBoundsReveal(Mobject())._native_kind
try:
    _CreationBoundsReveal(None)
except TypeError as error:
    assert "pointwise_become_partial" in str(error), error
else:
    raise AssertionError("ShowPartial subclass accepted a non-Mobject")

# ShowPartial classifies a subclass's get_bounds into the two native reveal
# vocabularies and respects the resulting bounds.
assert _CreationBoundsReveal(manimlib.Square())._native_params() == {
    "bounds_kind": "creation"
}


class _SlidingWindowReveal(creation.ShowPartial):
    def get_bounds(self, alpha):
        upper = alpha * 1.25
        return (max(upper - 0.25, 0.0), min(upper, 1.0))


sliding_params = _SlidingWindowReveal(manimlib.Square())._native_params()
assert sliding_params["bounds_kind"] == "passing_flash"
assert abs(sliding_params["time_width"] - 0.25) < 1e-9


class _SkewedReveal(creation.ShowPartial):
    def get_bounds(self, alpha):
        return (alpha / 2.0, alpha)


try:
    _SkewedReveal(manimlib.Square())._native_params()
except NotImplementedError as error:
    assert "native reveal rule" in str(error), error
else:
    raise AssertionError("a non-native get_bounds rule must refuse precisely")

# A subclass-defined creation window plays through the native segment: with a
# constant rate the end state is exactly pointwise_become_partial(start, 0, r).
partial_scene = InteractiveScene()
partial_square = manimlib.Square(side_length=2.0)
partial_square.set_stroke(manimlib.WHITE, width=4.0)
partial_reference = partial_square.copy()
partial_expected = partial_square.copy()
partial_expected.pointwise_become_partial(partial_reference, 0.0, 0.5)
partial_scene.add(partial_square)
partial_scene.play(
    _CreationBoundsReveal(partial_square, rate_func=lambda t: 0.5),
    run_time=2.0 / 30.0,
)
assert np.allclose(partial_square.get_points(), partial_expected.get_points())
assert not np.allclose(
    partial_square.get_points(), partial_reference.get_points()
)

# DrawBorderThenFill crosses the native segment: a constant rate in the first
# half leaves the border-only outline (fill opacity 0, configured stroke), one
# in the second half leaves the outline→start cross-fade midpoint, and a full
# run restores the start data exactly (creation.py:122).
dbtf_scene = InteractiveScene()


def _dbtf_square():
    square = manimlib.Square(side_length=2.0)
    square.set_fill(manimlib.BLUE, opacity=1.0)
    square.set_stroke(manimlib.WHITE, width=4.0)
    return square


border_square = _dbtf_square()
dbtf_scene.add(border_square)
dbtf_scene.play(
    creation.DrawBorderThenFill(
        border_square, stroke_color=manimlib.RED, rate_func=lambda t: 0.25
    ),
    run_time=2.0 / 30.0,
)
assert np.allclose(border_square.data["fill_rgba"][:, 3], 0.0)
assert np.allclose(border_square.data["stroke_width"], 2.0)
assert np.allclose(
    border_square.data["stroke_rgba"][:, :3],
    manimlib.color_to_rgb(manimlib.RED),
)

fade_square = _dbtf_square()
dbtf_scene.add(fade_square)
dbtf_scene.play(
    creation.DrawBorderThenFill(fade_square, rate_func=lambda t: 0.75),
    run_time=2.0 / 30.0,
)
assert np.allclose(fade_square.data["fill_rgba"][:, 3], 0.5)
assert np.allclose(fade_square.data["stroke_width"], 3.0)

full_square = _dbtf_square()
dbtf_scene.add(full_square)
dbtf_scene.play(
    creation.DrawBorderThenFill(full_square), run_time=2.0 / 30.0
)
assert np.allclose(full_square.data["fill_rgba"][:, 3], 1.0)
assert np.allclose(full_square.data["stroke_width"], 4.0)

# Write exposes its Reference timing helpers and passes their actual results
# to the native animation, including results supplied by subclass overrides.
write_probe = creation.Write(_dbtf_square())
assert isinstance(write_probe, creation.DrawBorderThenFill)
assert write_probe.run_time == 1
assert write_probe.lag_ratio == 0.2
assert write_probe.compute_run_time(14, -1) == 1
assert write_probe.compute_run_time(15, -1) == 2
assert write_probe.compute_run_time(99, 0) == 0
assert write_probe.compute_lag_ratio(19, -1) == 0.2
assert math.isclose(write_probe.compute_lag_ratio(39, -1), 0.1)
assert write_probe.compute_lag_ratio(99, 0) == 0
assert write_probe._native_params() == {}


class TimedWrite(creation.Write):
    def compute_run_time(self, family_size, run_time):
        self.timing_family_size = family_size
        return 2.0 / 30.0

    def compute_lag_ratio(self, family_size, lag_ratio):
        self.lag_family_size = family_size
        return 0.0


timed_write = TimedWrite(_dbtf_square())
assert timed_write.timing_family_size == timed_write.lag_family_size == 1
timed_write_scene = Scene()
timed_write_scene.play(timed_write)
assert math.isclose(timed_write_scene.time, 2.0 / 30.0)

# fm-5wq.14: Scene.time is the Reference's float attribute (scene.py:121),
# which corpus updaters read as a value (`np.cos(0.1 * sc.time)`); the
# native clock owns it, so assignment is refused by name.
_clock_scene = Scene()
_clock_dot = manimlib.Dot()
_clock_dot.add_updater(lambda m, sc=_clock_scene: m.move_to([np.cos(0.1 * sc.time), 0.0, 0.0]))
_clock_scene.add(_clock_dot)
assert isinstance(_clock_scene.time, float) and _clock_scene.time == 0.0
_clock_scene.wait(0.5)
assert math.isclose(_clock_scene.time, 0.5) and _clock_scene.time == _clock_scene.get_time()
assert math.isclose(_clock_dot.get_x(), math.cos(0.05), abs_tol=1e-6)
try:
    _clock_scene.time = 0.0
except AttributeError as error:
    assert "BN-02" in str(error), error
else:
    raise AssertionError("Scene.time accepted an assignment")
assert math.isclose(_clock_scene.time, 0.5)
assert np.allclose(timed_write.mobject.data["fill_rgba"][:, 3], 1.0)

# fm-5wq.4.57: TransformMatchingStrings matches by string identity over the
# longest matching blocks of the two glyph-key sequences — native span parts,
# not the two-render hack — while TransformMatchingTex keeps bare span-key
# equality for its semantic isolate units.
assert matching_module.TransformMatchingStrings._native_kind == (
    "transform_matching_strings"
)
assert matching_module.TransformMatchingTex._native_kind == (
    "transform_matching_tex"
)

strings_source = manimlib.Text("abc xyz").shift(manimlib.LEFT)
strings_target = manimlib.Text("qrs xyz").shift(manimlib.RIGHT)
strings_animation = matching_module.TransformMatchingStrings(
    strings_source,
    strings_target,
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
strings_params = strings_animation._native_params()
strings_source_keys = strings_params["source_keys"]
strings_target_keys = strings_params["target_keys"]
assert len(strings_source_keys) == len(strings_target_keys) == 6
matched_source_keys = [
    key for _, key in strings_source_keys if key.startswith("block:")
]
matched_target_keys = [
    key for _, key in strings_target_keys if key.startswith("block:")
]
assert matched_source_keys == matched_target_keys
assert [key.rsplit(":", 1)[1] for key in matched_source_keys] == ["x", "y", "z"]
assert all(
    key.startswith("source-only:")
    for _, key in strings_source_keys
    if not key.startswith("block:")
)
assert all(
    key.startswith("target-only:")
    for _, key in strings_target_keys
    if not key.startswith("block:")
)

# The matched glyphs move: after play the source "x" sits on the target "x".
strings_scene = Scene()
strings_scene.play(strings_animation)
strings_source_x = strings_source_keys[3][0]
strings_target_x = strings_target_keys[3][0]
assert strings_source_x is not strings_target_x
assert np.allclose(strings_source_x.get_center(), strings_target_x.get_center())

# Bare key equality would pair both glyphs of "ab" → "ba"; the matching-blocks
# discipline admits exactly the one longest shared run.
swap_params = matching_module.TransformMatchingStrings(
    manimlib.Text("ab"), manimlib.Text("ba")
)._native_params()
assert (
    len([key for _, key in swap_params["source_keys"] if key.startswith("block:")])
    == 1
)

# fm-5wq.11: reordered runs. The Reference's matching_blocks
# (transform_matching_parts.py:169) repeatedly claims the longest remaining
# common run, nulling what it claimed; this is that loop, over the same
# symbol sequences, as the oracle for "abcde" -> "cdeab".
import difflib as _difflib

_blocks_source, _blocks_target = manimlib.Text("abcde"), manimlib.Text("cdeab")
_blocks_pairs = matching_module.TransformMatchingStrings(
    _blocks_source, _blocks_target
).matching_blocks(_blocks_source, _blocks_target, (), {})
_syms = [list("abcde"), list("cdeab")]
_expected_blocks = []
while True:
    _run = _difflib.SequenceMatcher(None, *_syms).find_longest_match(0, 5, 0, 5)
    if _run.size == 0:
        break
    _expected_blocks.append(
        (list(range(_run.a, _run.a + _run.size)), list(range(_run.b, _run.b + _run.size)))
    )
    for _offset in range(_run.size):
        _syms[0][_run.a + _offset], _syms[1][_run.b + _offset] = "Null1", "Null2"
assert _expected_blocks == [([2, 3, 4], [0, 1, 2]), ([0, 1], [3, 4])]
_block_glyphs = [m.family_members_with_points() for m in (_blocks_source, _blocks_target)]
assert len(_blocks_pairs) == len(_expected_blocks)
for _pair, _indices in zip(_blocks_pairs, _expected_blocks):
    for _side in (0, 1):
        assert [id(g) for g in _pair[_side].family_members_with_points()] == [
            id(_block_glyphs[_side][i]) for i in _indices[_side]
        ], (_pair, _indices)

# matched_keys pins a key to raw global identity on both sides, so the user's
# asserted match survives outside any block.
pinned_params = matching_module.TransformMatchingStrings(
    manimlib.Text("ab"), manimlib.Text("ba"), matched_keys=["b"]
)._native_params()
assert "b" in [key for _, key in pinned_params["source_keys"]]
assert "b" in [key for _, key in pinned_params["target_keys"]]

# A dropped span map on visible text cannot claim success (383813b5 keeps
# this provenance refusal while animating genuinely empty content).
strings_blank_source = manimlib.Text("stub")
strings_blank_source._string_sub_spans = []
try:
    matching_module.TransformMatchingStrings(
        strings_blank_source, manimlib.Text("x")
    )
except bridge_errors.TexError as error:
    assert "native span" in str(error), error
else:
    raise AssertionError(
        "TransformMatchingStrings accepted an empty span map"
    )

try:
    matching_module.TransformMatchingStrings(
        geometry.Rectangle(width=1.0, height=1.0), manimlib.Text("x")
    )
except TypeError as error:
    assert "TransformMatchingStrings expects two StringMobject" in str(error), error
else:
    raise AssertionError("TransformMatchingStrings accepted a non-StringMobject")


# --------------------------- ShowIncreasingSubsets / ShowSubmobjectsOneByOne
# fm-5wq.4.58: the subset reveals play through Choreo's native mechanism —
# the group's child list is rewritten each frame from the construction-time
# snapshot.


class SubsetHook(creation_animation.ShowIncreasingSubsets):
    def update_submobject_list(self, index):
        self.last_subset_index = index
        super().update_submobject_list(index)


hook_children = [Mobject().set_points([[float(i), 0.0, 0.0]]) for i in range(3)]
hook_group = manimlib.Group(*hook_children)
hook_scene = Scene().add(hook_group)
hook_animation = SubsetHook(hook_group, rate_func=manimlib.linear)
hook_animation.update_submobject_list(-1)
assert list(hook_group.submobjects) == hook_children[:2]
assert hook_children[2].parents == []
hook_animation.interpolate_mobject(1.0)
assert hook_animation.last_subset_index == 3
assert list(hook_group.submobjects) == hook_children
assert hook_children[2].parents == [hook_group]
assert hook_scene._engine_facts()[:2] == (1, 4)
one_hook = creation_animation.ShowSubmobjectsOneByOne(hook_group)
one_hook.update_submobject_list(2)
assert list(hook_group.submobjects) == [hook_children[1]]
one_hook.update_submobject_list(0)
assert list(hook_group.submobjects) == []

subset_children = [
    geometry.Rectangle(width=0.5, height=0.5),
    geometry.Rectangle(width=0.6, height=0.6),
    geometry.Rectangle(width=0.7, height=0.7),
]
subset_group = manimlib.VGroup(*subset_children)
subset_scene = Scene()
subset_probe = geometry.Rectangle(width=1.0, height=1.0)
subset_counts = []
subset_scene.play(
    creation_animation.ShowIncreasingSubsets(subset_group),
    update_animation.UpdateFromFunc(
        subset_probe,
        lambda mob: subset_counts.append(len(subset_group.submobjects)),
    ),
    run_time=4.0 / 30.0,
)
# The reveal only grows, on child-count boundaries, and finishes whole.
assert set(subset_counts) <= {0, 1, 2, 3}, subset_counts
assert subset_counts == sorted(subset_counts), subset_counts
assert subset_counts[-1] == 3, subset_counts
assert len(subset_group.submobjects) == 3

one_children = [
    geometry.Rectangle(width=0.5, height=0.5),
    geometry.Rectangle(width=0.6, height=0.6),
    geometry.Rectangle(width=0.7, height=0.7),
]
one_group = manimlib.VGroup(*one_children)
one_scene = Scene()
one_probe = geometry.Rectangle(width=1.0, height=1.0)
one_counts = []
one_scene.play(
    creation_animation.ShowSubmobjectsOneByOne(one_group),
    update_animation.UpdateFromFunc(
        one_probe,
        lambda mob: one_counts.append(len(one_group.submobjects)),
    ),
    run_time=4.0 / 30.0,
)
# One child at a time, and the Reference's clip-to-n-1 quirk leaves the
# second-to-last child visible at the end — ported verbatim natively.
assert set(one_counts) <= {0, 1}, one_counts
assert len(one_group.submobjects) == 1
assert one_group.submobjects[0] is one_children[1]

# Authored subset selectors run through the production callback boundary.
assert (
    creation_animation.ShowIncreasingSubsets(
        manimlib.VGroup(geometry.Rectangle(width=0.5, height=0.5)),
        int_func=np.ceil,
    )._native_params()["int_round"]
    == "ceil"
)
custom_members = [geometry.Rectangle(width=0.5, height=0.5) for _ in range(3)]
custom_group = manimlib.VGroup(*custom_members)
custom_selector = creation_animation.ShowIncreasingSubsets(
    custom_group, int_func=lambda value: int(value), rate_func=manimlib.linear,
)
custom_selector.interpolate(.5)
assert list(custom_group.submobjects) == custom_members[:1]
custom_selector.int_func = lambda value: 2
custom_selector.interpolate(.5)
assert list(custom_group.submobjects) == custom_members[:2]
assert custom_selector._native_params() == {}
for invalid in ("round", 7):
    try:
        creation_animation.ShowIncreasingSubsets(custom_group, int_func=invalid)
    except TypeError as error:
        assert "int_func must be callable" in str(error)
    else:
        raise AssertionError("ShowIncreasingSubsets accepted a non-callable selector")
try:
    creation_animation.ShowIncreasingSubsets("not a mobject")
except TypeError as error:
    assert "requires a Mobject family" in str(error)
else:
    raise AssertionError("ShowIncreasingSubsets accepted a non-Mobject")
for empty_cls in (creation_animation.ShowIncreasingSubsets,
                  creation_animation.ShowSubmobjectsOneByOne):
    empty_group = manimlib.VGroup()
    empty_reveal = empty_cls(empty_group)
    empty_reveal.begin()
    empty_reveal.finish()
    assert list(empty_group.submobjects) == []

# fm-5wq.4.65: isolate= and tex_to_color_map= ride the native span map — the
# isolated pieces select the flat glyph family (source identity, no labelled
# second render or argument containers), and constructor color maps land on
# exactly the mapped spans.
iso_tex = manimlib.Tex("x^2 + y^2", isolate=["x", "y"])
assert iso_tex.get_string() == "x^2 + y^2"
assert len(iso_tex.submobjects) == 5
assert all(leaf.has_points() and not leaf.submobjects for leaf in iso_tex)
assert all(len(path) == 1 for path in iso_tex._string_sub_paths)
iso_x_part = iso_tex.get_part_by_tex("x")
iso_y_part = iso_tex.get_part_by_tex("y")
assert len(iso_x_part) == 1 and len(iso_y_part) == 1

# The isolated span refers to the same independently colorable glyph node.
iso_tex[0].set_color(manimlib.YELLOW)
assert all(leaf.get_fill_color() == manimlib.YELLOW for leaf in iso_x_part)
assert all(leaf.get_fill_color() != manimlib.YELLOW for leaf in iso_y_part)
iso_tex.set_color_by_tex("y", manimlib.RED)
assert all(leaf.get_fill_color() == manimlib.RED for leaf in iso_y_part)
assert all(leaf.get_fill_color() == manimlib.YELLOW for leaf in iso_x_part)

# Constructor tex_to_color_map routes over the same span map.
t2c_tex = manimlib.Tex("x^2 + y^2", tex_to_color_map={"x": manimlib.RED})
assert all(
    leaf.get_fill_color() == manimlib.RED
    for leaf in t2c_tex.get_part_by_tex("x")
)
assert all(
    leaf.get_fill_color() != manimlib.RED
    for leaf in t2c_tex.get_part_by_tex("y")
)

# A Scene can play a fade on the isolated span's glyph node. The native
# finish contract is the Reference's restore-then-remove (fading.py:76):
# fade_out finishes at final_alpha_value = 0 and restores the records, so
# the honest observable is the mid-play probe — the leaf's fill alpha
# drops through exactly 0.5 to exactly 0.0 under a linear two-frame play.
iso_scene = Scene()
iso_scene.add(iso_tex)
iso_faded_leaf = iso_x_part[0]
iso_fade_probe = manimlib.Dot().move_to((3.0, 0.0, 0.0))
iso_scene.add(iso_fade_probe)
iso_fade_alphas = []


def _iso_fade_capture(mobject, dt):
    del mobject
    if dt > 0:
        iso_fade_alphas.append(float(iso_faded_leaf.data["fill_rgba"][0, 3]))


iso_fade_probe.add_updater(_iso_fade_capture)
iso_scene.play(
    manimlib.FadeOut(iso_tex[0]),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
iso_fade_probe.remove_updater(_iso_fade_capture)
assert len(iso_fade_alphas) == 2, iso_fade_alphas
assert math.isclose(iso_fade_alphas[0], 0.5, rel_tol=0.0, abs_tol=1e-9)
assert math.isclose(iso_fade_alphas[1], 0.0, rel_tol=0.0, abs_tol=1e-9)

# A labelled isolate occurrence that owns no span-map primitive (a bare
# script marker) is a named error, never a silent no-op selection. A
# whitespace isolate draws nothing under the Reference's parse either, so it
# is tolerated and leaves the geometry alone. An isolate entry with no
# occurrence at all stays tolerated (the Reference's behavior, pinned above
# for the multi-part "=" case).
assert np.array_equal(
    manimlib.Tex("x^2 + y^2", isolate=[" "]).get_all_points(),
    manimlib.Tex("x^2 + y^2").get_all_points(),
)
try:
    manimlib.Tex("x^2 + y^2", isolate=["^"])
except bridge_errors.TexError as error:
    assert "is not in the native span map" in str(error), error
else:
    raise AssertionError("isolate of an unmapped substring did not raise")

iso_absent_tex = manimlib.Tex("x^2 + y^2", isolate=["z"])
assert len(iso_absent_tex.get_parts_by_tex("z")) == 0


# ----------------------------------------------------- AddTextLetterByLetter
# fm-5wq.4.59: letter-granularity reveal over native span-map letter groups
# (one non-whitespace glyph per group, in reading order).

letter_text = manimlib.Text("hi you")
assert len(letter_text.submobjects) == 5  # h i y o u
letter_anim = creation_animation.AddTextLetterByLetter(letter_text)
# Derived parameters: 0.1 s per letter, five letters, linear rate.
assert abs(letter_anim.run_time - 0.5) < 1e-12
assert letter_anim.rate_func(0.375) == 0.375

letter_scene = Scene()
letter_probe = geometry.Rectangle(width=1.0, height=1.0)
letter_counts = []
letter_scene.play(
    letter_anim,
    update_animation.UpdateFromFunc(
        letter_probe,
        lambda mob: letter_counts.append(len(letter_text.submobjects)),
    ),
    run_time=0.5,
)
# Letters only accumulate, one glyph at a time, and finish whole.
assert set(letter_counts) <= {0, 1, 2, 3, 4, 5}, letter_counts
assert letter_counts == sorted(letter_counts), letter_counts
assert any(0 < count < 5 for count in letter_counts), letter_counts
assert len(letter_text.submobjects) == 5

# An explicit non-negative run_time is kept, not recomputed.
assert (
    abs(
        creation_animation.AddTextLetterByLetter(
            manimlib.Text("abc"), run_time=2.0
        ).run_time
        - 2.0
    )
    < 1e-12
)

# The word sibling's exact refusal shape for a non-StringMobject.
try:
    creation_animation.AddTextLetterByLetter(
        geometry.Rectangle(width=1.0, height=1.0)
    )
except AssertionError:
    pass
else:
    raise AssertionError("AddTextLetterByLetter accepted a non-StringMobject")

# A string with no glyphs has no letter groups: named, never a silent no-op.
try:
    creation_animation.AddTextLetterByLetter(manimlib.Text(" "))
except ValueError as error:
    assert "letter groups" in str(error)
else:
    raise AssertionError("AddTextLetterByLetter accepted an empty string mobject")

# fm-5wq.4.61: the movement family — Homotopy/ComplexHomotopy/PhaseFlow ride
# the Python-callback segment slot (user Python maps), MoveAlongPath rides
# the native true-arclength sampler (BN-03).
movement = importlib.import_module("manimlib.animation.movement")
assert movement.ComplexHomotopy.__bases__ == (movement.Homotopy,)
assert movement.SmoothedVectorizedHomotopy.__bases__ == (movement.Homotopy,)
assert movement.Homotopy.apply_function_config == {}
assert movement.SmoothedVectorizedHomotopy.apply_function_config == {
    "make_smooth": True
}
assert movement.MoveAlongPath._native_kind == "move_along_path"

# MoveAlongPath: a dot played along a line ends at the path's end, and a
# constant-rate probe sits at the true-arclength midpoint.
map_scene = InteractiveScene()
map_dot = manimlib.Dot()
map_path = geometry.Line((-1.0, -1.0, 0.0), (1.0, 1.0, 0.0))
map_scene.add(map_dot, map_path)
map_scene.play(
    movement.MoveAlongPath(map_dot, map_path),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(map_dot.get_center(), [1.0, 1.0, 0.0], atol=1e-9)
map_scene.play(
    movement.MoveAlongPath(map_dot, map_path, rate_func=lambda t: 0.5),
    run_time=1.0 / 30.0,
)
assert np.allclose(map_dot.get_center(), [0.0, 0.0, 0.0], atol=1e-9)

# Homotopy applies the (x, y, z, t) map at t = rated alpha, from a fresh
# start-point restore each frame (no compounding).
homotopy_scene = InteractiveScene()
homotopy_square = manimlib.Square(side_length=1.0)
homotopy_scene.add(homotopy_square)
homotopy_start = homotopy_square.get_center().copy()
homotopy_scene.play(
    movement.Homotopy(
        lambda x, y, z, t: (x + t, y, z), homotopy_square
    ),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(
    homotopy_square.get_center(), homotopy_start + [1.0, 0.0, 0.0], atol=1e-9
)

# ComplexHomotopy: the plane map (z, t) ↦ z·e^{iπt/2} carries (1, 0) to
# (0, 1) with the z lane untouched.
complex_scene = InteractiveScene()
complex_dot = manimlib.Dot().move_to((1.0, 0.0, 0.0))
complex_scene.add(complex_dot)
complex_scene.play(
    movement.ComplexHomotopy(
        lambda z, t: z
        * complex(math.cos(t * math.pi / 2.0), math.sin(t * math.pi / 2.0)),
        complex_dot,
    ),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(complex_dot.get_center(), [0.0, 1.0, 0.0], atol=1e-9)

# PhaseFlow: forward-Euler advection of a constant field integrates to
# exactly virtual_time·field regardless of the step sequence.
phase_scene = InteractiveScene()
phase_dot = manimlib.Dot()
phase_scene.add(phase_dot)
phase_start = phase_dot.get_center().copy()
phase_scene.play(
    movement.PhaseFlow(
        lambda p: np.array([1.0, 0.0, 0.0]), phase_dot, virtual_time=1.0
    ),
    run_time=2.0 / 30.0,
)
assert np.allclose(
    phase_dot.get_center(), phase_start + [1.0, 0.0, 0.0], atol=1e-9
)

# A point-less path is a named error at construction, not a silent no-op.
try:
    movement.MoveAlongPath(manimlib.Dot(), VMobject())
except ValueError as error:
    assert "no points to move along" in str(error), error
else:
    raise AssertionError("MoveAlongPath accepted a point-less path")

try:
    movement.MoveAlongPath(manimlib.Dot(), Mobject())
except TypeError as error:
    assert "no curve to sample" in str(error), error
else:
    raise AssertionError("MoveAlongPath accepted a non-VMobject path")


# --------------------------------- MaintainPositionRelativeTo / ChangeSpeed
# fm-5wq.4.62: the follower holds its construction-time offset while the
# tracked mobject moves in the same play call, through the native tracker.

follow_scene = Scene()
follow_tracked = geometry.Rectangle(width=1.0, height=1.0)
follow_er = geometry.Rectangle(width=0.5, height=0.5)
follow_er.shift([1.0, 2.0, 0.0])
follow_scene.add(follow_tracked, follow_er)
follow_diff = follow_er.get_center() - follow_tracked.get_center()
assert np.allclose(follow_diff, [1.0, 2.0, 0.0])
follow_scene.play(
    follow_tracked.animate.shift([2.0, -1.0, 0.0]),
    update_animation.MaintainPositionRelativeTo(follow_er, follow_tracked),
    run_time=2.0 / 30.0,
)
assert np.allclose(follow_tracked.get_center(), [2.0, -1.0, 0.0])
assert np.allclose(
    follow_er.get_center() - follow_tracked.get_center(), follow_diff
)

# Named refusals: a missing or non-Mobject tracked target.
try:
    update_animation.MaintainPositionRelativeTo(
        geometry.Rectangle(width=1.0, height=1.0)
    )
except TypeError as error:
    assert "tracked Mobject" in str(error)
else:
    raise AssertionError("MaintainPositionRelativeTo accepted a missing target")

try:
    update_animation.MaintainPositionRelativeTo(
        geometry.Rectangle(width=1.0, height=1.0), "not a mobject"
    )
except TypeError as error:
    assert "tracked Mobject" in str(error)
else:
    raise AssertionError("MaintainPositionRelativeTo accepted a str target")

# ChangeSpeed is ManimCE surface whose native clock-remap seam has not
# landed: construction is the precise named refusal, never a wrong-speed
# play.
speed_module = importlib.import_module("manimlib.animation.speed")
try:
    speed_module.ChangeSpeed(None, {})
except NotImplementedError as error:
    assert "clock-remap seam" in str(error)
else:
    raise AssertionError("ChangeSpeed constructed without its native seam")


# ------------------------------------------------------------------- Delay
# fm-5wq.4.66: a timed hold — the engine clock advances, nothing mutates.

specialized_animation = importlib.import_module("manimlib.animation.specialized")

delay_scene = Scene()
delay_witness = geometry.Rectangle(width=1.0, height=1.0)
delay_witness.shift([0.5, -0.5, 0.0])
delay_scene.add(delay_witness)
delay_before = delay_witness.get_center().copy()
delay_time_before = delay_scene.get_time()
delay_scene.play(specialized_animation.Delay(run_time=2.0 / 30.0))
assert abs(delay_scene.get_time() - delay_time_before - 2.0 / 30.0) < 1e-9
assert np.allclose(delay_witness.get_center(), delay_before)

# Delay composes into a same-play pair: the other animation still runs.
delay_pair_scene = Scene()
delay_probe = geometry.Rectangle(width=1.0, height=1.0)
delay_frames = []
delay_pair_scene.play(
    specialized_animation.Delay(run_time=2.0 / 30.0),
    update_animation.UpdateFromFunc(
        delay_probe, lambda mob: delay_frames.append(True)
    ),
    run_time=2.0 / 30.0,
)
assert len(delay_frames) >= 2, delay_frames

# Negative: a negative (or non-finite) run_time is a named error.
try:
    specialized_animation.Delay(run_time=-1.0)
except ValueError as error:
    assert "finite non-negative duration" in str(error)
else:
    raise AssertionError("Delay accepted a negative run_time")

# fm-5wq.4.63: CyclicReplace / Swap ride native per-mobject arc transforms,
# and the portal's scalar-arc path factories route Transform's path_func=
# surface onto the native path_arc plane.
transform_module = importlib.import_module("manimlib.animation.transform")
assert transform_module.Swap.__bases__ == (transform_module.CyclicReplace,)
assert transform_module.CyclicReplace.__bases__ == (transform_module.Transform,)

cyclic_scene = InteractiveScene()
cyclic_a = manimlib.Dot().move_to((-1.0, 0.0, 0.0))
cyclic_b = manimlib.Dot().move_to((1.0, 0.0, 0.0))
cyclic_c = manimlib.Dot().move_to((0.0, 1.0, 0.0))
cyclic_scene.add(cyclic_a, cyclic_b, cyclic_c)
cyclic_scene.play(
    transform_module.CyclicReplace(cyclic_a, cyclic_b, cyclic_c),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(cyclic_a.get_center(), [1.0, 0.0, 0.0], atol=1e-9)
assert np.allclose(cyclic_b.get_center(), [0.0, 1.0, 0.0], atol=1e-9)
assert np.allclose(cyclic_c.get_center(), [-1.0, 0.0, 0.0], atol=1e-9)

swap_scene = InteractiveScene()
swap_a = manimlib.Dot().move_to((-1.0, 0.0, 0.0))
swap_b = manimlib.Dot().move_to((1.0, 0.0, 0.0))
swap_scene.add(swap_a)
# swap_b is deliberately not added: the extra-mobject hook adds it at play.
swap_scene.play(
    transform_module.Swap(swap_a, swap_b),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(swap_a.get_center(), [1.0, 0.0, 0.0], atol=1e-9)
assert np.allclose(swap_b.get_center(), [-1.0, 0.0, 0.0], atol=1e-9)

# A single mobject or an empty call cannot cycle: named errors.
for cyclic_bad_args in [(), (manimlib.Dot(),)]:
    try:
        transform_module.CyclicReplace(*cyclic_bad_args)
    except ValueError as error:
        assert "at least two mobjects to cycle" in str(error), error
    else:
        raise AssertionError("CyclicReplace accepted fewer than two mobjects")

# clockwise_path()/counterclockwise_path() carry their scalar arc onto the
# native path_arc surface; the halfway probe of a clockwise transform from
# (1, 0) to (-1, 0) dips to (0, -1) — the arc, not the chord.
arc_probe = transform_module.Transform(
    manimlib.Dot(), manimlib.Dot(), path_func=manimlib.clockwise_path()
)
assert math.isclose(arc_probe.path_arc, -math.pi)
assert transform_module.Transform(
    manimlib.Dot(), manimlib.Dot(), path_func=manimlib.counterclockwise_path()
).path_arc == math.pi

arc_scene = InteractiveScene()
arc_dot = manimlib.Dot().move_to((1.0, 0.0, 0.0))
arc_target = manimlib.Dot().move_to((-1.0, 0.0, 0.0))
arc_scene.add(arc_dot)
arc_scene.play(
    transform_module.Transform(
        arc_dot,
        arc_target,
        path_func=manimlib.clockwise_path(),
        rate_func=lambda t: 0.5,
    ),
    run_time=1.0 / 30.0,
)
assert np.allclose(arc_dot.get_center(), [0.0, -1.0, 0.0], atol=1e-6)

# fm-5wq.14: every Fade is a Transform, so path_arc bends its travel.
# FadeIn/FadeOut used to drop it silently and the point fades refused it.
# FadeInFromPoint/FadeOutToPoint are the Reference's FadeIn/FadeOut
# subclasses, with the shift fixed at construction (fading.py:71-88).
fading_module = importlib.import_module("manimlib.animation.fading")
paths_module = importlib.import_module("manimlib.utils.paths")
assert fading_module.FadeInFromPoint.__bases__ == (fading_module.FadeIn,)
assert fading_module.FadeOutToPoint.__bases__ == (fading_module.FadeOut,)
for fade_arc in (0.0, -math.pi / 2.0, 2.0):
    for fade_kind in ("from_point", "shift"):
        fade_scene = InteractiveScene()
        fade_dot = manimlib.Dot().move_to((1.0, 0.5, 0.0))
        fade_end = np.array(fade_dot.get_points(), dtype=float)
        if fade_kind == "from_point":
            fade_start = np.tile([-1.0, 0.0, 0.0], (len(fade_end), 1))
            fade = fading_module.FadeInFromPoint(
                fade_dot, np.array([-1.0, 0.0, 0.0]), path_arc=fade_arc,
                rate_func=lambda t: 0.5,
            )
        else:
            fade_start = fade_end - [0.0, 2.0, 0.0]
            fade = fading_module.FadeIn(
                fade_dot, shift=[0.0, 2.0, 0.0], path_arc=fade_arc,
                rate_func=lambda t: 0.5,
            )
        fade_scene.play(fade, run_time=1.0 / 30.0)
        fade_expected = paths_module.path_along_arc(fade_arc)(fade_start, fade_end, 0.5)
        assert np.allclose(fade_dot.get_points(), fade_expected, atol=1e-5), (
            fade_kind, fade_arc,
        )
fade_out_dot = manimlib.Dot().move_to((1.0, 0.5, 0.0))
fade_out = fading_module.FadeOutToPoint(fade_out_dot, np.array([-1.0, 0.0, 0.0]), path_arc=1.0)
fade_out_dot.shift([5.0, 0.0, 0.0])
assert np.allclose(fade_out.shift_vect, [-2.0, -0.5, 0.0])
assert fade_out.scale_factor == 0.0 and fade_out.remover
assert fade_out._native_params()["path_arc"] == 1.0
assert fading_module.FadeOut(fade_out_dot, path_arc=0.5)._native_params()["path_arc"] == 0.5
try:
    fading_module.FadeOutToPoint(fade_out_dot, [0.0, 0.0, 0.0], bogus_keyword=1)
except NotImplementedError as error:
    assert "bogus_keyword" in str(error), error
except TypeError as error:
    assert "bogus_keyword" in str(error), error
else:
    raise AssertionError("FadeOutToPoint accepted an unknown keyword")

# fm-5wq.4.63 / f9e529b1: Transform accepts an authored path function
# through the callback lifecycle; non-callable stays TypeError.
custom_transform = transform_module.Transform(
    manimlib.Dot(), manimlib.Dot(), path_func=lambda s, e, alpha: s
)
assert callable(custom_transform.path_func)
try:
    transform_module.Transform(
        manimlib.Dot(), manimlib.Dot(), path_func="not_callable"
    )
except TypeError as error:
    assert "path_func" in str(error), error
else:
    raise AssertionError("Transform accepted a non-callable path function")

# fm-5wq.4.68: TransformMatchingParts / TransformMatchingShapes ride the
# native shape matcher — user pairs claim first, the same-shape product
# second, directional fades for the leftovers.
assert matching_module.TransformMatchingShapes.__bases__ == (
    matching_module.TransformMatchingParts,
)
assert matching_module.TransformMatchingParts._native_kind == (
    "transform_matching_parts"
)
assert matching_module.TransformMatchingShapes._native_kind == (
    "transform_matching_shapes"
)

# Shapes: the identical square pairs by shape and lands on its partner;
# the circle has no same-shape partner and takes the fade path.
shapes_scene = InteractiveScene()
shapes_square = manimlib.Square(side_length=1.0).move_to((-2.0, 0.0, 0.0))
shapes_circle = manimlib.Circle(radius=0.4).move_to((-2.0, 1.0, 0.0))
shapes_source = manimlib.VGroup(shapes_square, shapes_circle)
shapes_target_square = manimlib.Square(side_length=1.0).move_to((2.0, 0.0, 0.0))
shapes_target_triangle = manimlib.Triangle().move_to((2.0, 1.0, 0.0))
shapes_target = manimlib.VGroup(shapes_target_square, shapes_target_triangle)
shapes_scene.add(shapes_source)
shapes_scene.play(
    matching_module.TransformMatchingShapes(shapes_source, shapes_target),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(shapes_square.get_center(), [2.0, 0.0, 0.0], atol=1e-9)

# Parts: an explicit matched pair claims the circle onto the triangle, and
# the square still auto-matches through the shape product.
parts_scene = InteractiveScene()
parts_square = manimlib.Square(side_length=1.0).move_to((-2.0, 0.0, 0.0))
parts_circle = manimlib.Circle(radius=0.4).move_to((-2.0, 1.0, 0.0))
parts_source = manimlib.VGroup(parts_square, parts_circle)
parts_target_square = manimlib.Square(side_length=1.0).move_to((2.0, 0.0, 0.0))
parts_target_triangle = manimlib.Triangle().move_to((2.0, 1.0, 0.0))
parts_target = manimlib.VGroup(parts_target_square, parts_target_triangle)
parts_scene.add(parts_source)
parts_scene.play(
    matching_module.TransformMatchingParts(
        parts_source,
        parts_target,
        matched_pairs=[(parts_circle, parts_target_triangle)],
    ),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(parts_circle.get_center(), [2.0, 1.0, 0.0], atol=1e-9)
assert np.allclose(parts_square.get_center(), [2.0, 0.0, 0.0], atol=1e-9)

# Empty families are ordinary states of a changing group (383813b5): no leaf
# animation, and the composition still owns its declared time. A non-Mobject
# side stays a named error.
parts_empty = matching_module.TransformMatchingParts(
    manimlib.VGroup(), manimlib.VGroup(), run_time=3
)
assert list(parts_empty.animations) == [], parts_empty.animations

try:
    matching_module.TransformMatchingParts(manimlib.Square(), "target")
except TypeError as error:
    assert "expects two Mobject families" in str(error), error
else:
    raise AssertionError("TransformMatchingParts accepted a non-Mobject")

try:
    matching_module.TransformMatchingParts(
        manimlib.Square(),
        manimlib.Square(),
        matched_pairs=[(manimlib.Square(), "not-a-mobject")],
    )
except TypeError as error:
    assert "matched_pairs must pair Mobjects" in str(error), error
else:
    raise AssertionError("TransformMatchingParts accepted a malformed pair")


# ---------------- always / f_always / always_redraw / turn_animation_into_updater
# fm-5wq.4.69: the Reference's updater helpers over the live updater seam.

update_utils = importlib.import_module("manimlib.mobject.mobject_update_utils")

# always_redraw rebuilds from the callable every frame; the rebuilt mobject
# tracks live scene state through become().
redraw_source = geometry.Rectangle(width=1.0, height=1.0)
redraw_calls = []


def build_redraw_follower():
    redraw_calls.append(True)
    follower = geometry.Rectangle(width=0.25, height=0.25)
    follower.move_to(redraw_source.get_center() + np.array([0.0, 1.0, 0.0]))
    return follower


redraw_follower = update_utils.always_redraw(build_redraw_follower)
redraw_scene = Scene()
redraw_scene.add(redraw_source, redraw_follower)
redraw_scene.play(
    redraw_source.animate.shift([1.5, 0.0, 0.0]), run_time=2.0 / 30.0
)
assert len(redraw_calls) >= 3, redraw_calls  # construction + per-frame redraws
assert np.allclose(
    redraw_follower.get_center(),
    redraw_source.get_center() + np.array([0.0, 1.0, 0.0]),
)

# always(mob.move_to, other) keeps tracking across frames.
always_target = geometry.Rectangle(width=1.0, height=1.0)
always_chaser = geometry.Rectangle(width=0.3, height=0.3)
always_chaser.shift([-2.0, 0.0, 0.0])
returned = update_utils.always(always_chaser.move_to, always_target)
assert returned is always_chaser
always_scene = Scene()
always_scene.add(always_target, always_chaser)
always_scene.play(
    always_target.animate.shift([0.0, -1.25, 0.0]), run_time=2.0 / 30.0
)
assert np.allclose(always_chaser.get_center(), always_target.get_center())

# f_always: per-frame argument generators.
f_target = geometry.Rectangle(width=1.0, height=1.0)
f_chaser = geometry.Rectangle(width=0.3, height=0.3)
update_utils.f_always(f_chaser.move_to, lambda: f_target.get_center())
f_scene = Scene()
f_scene.add(f_target, f_chaser)
f_scene.play(f_target.animate.shift([0.75, 0.5, 0.0]), run_time=2.0 / 30.0)
assert np.allclose(f_chaser.get_center(), f_target.get_center())

# turn_animation_into_updater drives a Python animation to completion as a
# persistent updater, then pops itself.
turn_decimal = manimlib.DecimalNumber(0.0)
turn_anim = numbers_animation.ChangeDecimalToValue(
    turn_decimal, 5.0, run_time=2.0 / 30.0
)
update_utils.turn_animation_into_updater(turn_anim)
turn_scene = Scene()
turn_scene.add(turn_decimal)
turn_scene.play(specialized_animation.Delay(run_time=4.0 / 30.0))
assert abs(turn_decimal.get_value() - 5.0) < 1e-9

# Named refusals.
try:
    update_utils.always_redraw(None)
except TypeError as error:
    assert "requires a callable" in str(error)
else:
    raise AssertionError("always_redraw accepted None")

try:
    update_utils.always(geometry.Rectangle(width=1.0, height=1.0))
except AssertionError:
    pass
else:
    raise AssertionError("always accepted a non-method")

# fm-5wq.4.143: turn_animation_into_updater accepts native-segment classes;
# non-Animation stays TypeError.
creation_updater_mob = geometry.Rectangle(width=1.0, height=1.0)
creation_updater_res = update_utils.turn_animation_into_updater(
    manimlib.ShowCreation(creation_updater_mob)
)
assert creation_updater_res is creation_updater_mob
try:
    update_utils.turn_animation_into_updater(object())
except TypeError as error:
    assert "requires an Animation" in str(error), error
else:
    raise AssertionError("turn_animation_into_updater accepted a non-Animation")

# always_shift / always_rotate are thin dt-updater skins: Scene.wait and
# Scene.play own the frame clock, while the helpers only apply dt-scaled
# Marionette mutations in updater insertion order.
shifted_forever = geometry.Rectangle(width=1.0, height=1.0)
shifted_start = shifted_forever.get_center().copy()
assert (
    update_utils.always_shift(
        shifted_forever, direction=manimlib.RIGHT, rate=3.0
    )
    is shifted_forever
)
shifted_scene = Scene()
shifted_scene.add(shifted_forever)
shifted_scene.wait(2.0 / 30.0)
assert shifted_forever.get_center()[0] > shifted_start[0]

rotating_forever = geometry.Line(manimlib.RIGHT, 2.0 * manimlib.RIGHT)
rotation_samples = []
assert (
    update_utils.always_rotate(
        rotating_forever,
        rate=math.pi,
        about_point=manimlib.ORIGIN,
    )
    is rotating_forever
)
rotating_forever.add_updater(
    lambda mob: rotation_samples.append(mob.get_points().copy()), call=False
)
rotating_scene = Scene()
rotating_scene.add(rotating_forever)
rotating_scene.play(specialized_animation.Delay(run_time=2.0 / 30.0))
assert len(rotation_samples) >= 2
assert not np.allclose(rotation_samples[0], rotation_samples[-1])

# cycle_animation is turn_animation_into_updater(..., cycle=True): the
# updater stays after one run_time instead of finishing and popping.
cycle_decimal = manimlib.DecimalNumber(0.0)
cycle_anim = numbers_animation.ChangeDecimalToValue(
    cycle_decimal, 4.0, run_time=2.0 / 30.0
)
assert update_utils.cycle_animation(cycle_anim) is cycle_decimal
assert manimlib.cycle_animation is update_utils.cycle_animation
cycle_scene = Scene()
cycle_scene.add(cycle_decimal)
cycle_scene.play(specialized_animation.Delay(run_time=5.0 / 30.0))
assert len(cycle_decimal.get_updaters()) >= 1
# Two-and-a-half cycles of a 2-frame run_time lands mid-ramp, not parked at 4.
assert 0.25 < cycle_decimal.get_value() < 3.75

for helper, invalid in (
    (update_utils.always_shift, None),
    (update_utils.always_rotate, "not a mobject"),
):
    try:
        helper(invalid)
    except TypeError as error:
        assert str(error) == (
            helper.__name__
            + " requires a Mobject; got "
            + type(invalid).__name__
        )
    else:
        raise AssertionError(helper.__name__ + " accepted a non-Mobject")

# fm-5wq.4.70: audit of the transform leftovers. Restore/ScaleInPlace/
# ShrinkToCenter/FadeToColor/ApplyPointwiseFunctionToCenter were already
# bound and play above; the one real hole was the Restore ANIMATION, which
# accepted a never-saved mobject and deferred the refusal to the native
# segment — it now refuses at construction with the Reference's exact
# Mobject.restore() exception.
try:
    manimlib.Restore(manimlib.Dot())
except Exception as error:
    assert type(error) is Exception, error
    assert str(error) == "Trying to restore without having saved"
else:
    raise AssertionError("Restore accepted a mobject without saved state")

# Cheap re-anchors of this bead's positive halves through Scene.play.
restore_anchor = manimlib.Dot().move_to((1.0, 0.0, 0.0))
restore_anchor.save_state()
restore_anchor.move_to((0.0, 2.0, 0.0))
restore_anchor_scene = InteractiveScene()
restore_anchor_scene.add(restore_anchor)
restore_anchor_scene.play(
    manimlib.Restore(restore_anchor),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(restore_anchor.get_center(), [1.0, 0.0, 0.0], atol=1e-9)

scale_anchor = manimlib.Square(side_length=1.0)
restore_anchor_scene.add(scale_anchor)
restore_anchor_scene.play(
    manimlib.ScaleInPlace(scale_anchor, 2.0),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert math.isclose(scale_anchor.get_width(), 2.0, rel_tol=0.0, abs_tol=1e-9)


# ----------------------------------------- TracedPath / AnimatedBoundary
# fm-5wq.4.67: the updater-backed changing mobjects over the live seam.

changing_module = importlib.import_module("manimlib.mobject.changing")

# TracedPath grows a pointful path from a moving mobject across frames.
traced_mover = geometry.Rectangle(width=0.4, height=0.4)
traced_path = changing_module.TracedPath(traced_mover.get_center)
traced_scene = Scene()
traced_scene.add(traced_mover, traced_path)
traced_scene.play(
    traced_mover.animate.shift([1.5, 0.75, 0.0]), run_time=3.0 / 30.0
)
assert traced_path.has_points()
assert traced_path.get_num_points() > 0
assert len(traced_path.traced_points) >= 3
# The last traced point is where the mover ended up.
assert np.allclose(traced_path.traced_points[-1], traced_mover.get_center())

# AnimatedBoundary constructs its two cycling stroke copies and survives
# frames advancing.
boundary_target = geometry.Rectangle(width=1.2, height=0.8)
boundary = changing_module.AnimatedBoundary(boundary_target)
assert len(boundary.boundary_copies) == 2
boundary_scene = Scene()
boundary_scene.add(boundary_target, boundary)
boundary_scene.play(specialized_animation.Delay(run_time=2.0 / 30.0))
boundary_growing = boundary.boundary_copies[0]
assert boundary_growing.has_points()
assert boundary.total_time > 0.0

# Named refusals: a non-callable trace source, a non-VMobject boundary.
try:
    changing_module.TracedPath(None)
except TypeError as error:
    assert "requires a callable" in str(error)
else:
    raise AssertionError("TracedPath accepted None")

try:
    changing_module.AnimatedBoundary("not a vmobject")
except TypeError as error:
    assert "requires a VMobject" in str(error)
else:
    raise AssertionError("AnimatedBoundary accepted a non-VMobject")


# -------------------------- TransformMatchingStrings matched_pairs override
# fm-5wq.4.76: explicit pairs claim their parts with a unique shared native
# key BEFORE the matching-blocks pass, so a user pair beats what blocks
# would otherwise choose.

pairs_source = manimlib.Text("ab")
pairs_target = manimlib.Text("ab")
# Cross-pair the source 'a' onto the target 'b': blocks alone would pair
# a->a and b->b, so landing on b's position proves the override claimed
# first.
pairs_anim = matching_module.TransformMatchingStrings(
    pairs_source,
    pairs_target,
    matched_pairs=[(pairs_source[0], pairs_target[1])],
    run_time=2.0 / 30.0,
)
pairs_params = pairs_anim._native_params()
pairs_source_keys = dict(
    (id(part), key) for part, key in pairs_params["source_keys"]
)
pairs_target_keys = dict(
    (id(part), key) for part, key in pairs_params["target_keys"]
)
assert pairs_source_keys[id(pairs_source[0])] == "matched-pair:0"
assert pairs_target_keys[id(pairs_target[1])] == "matched-pair:0"

pairs_scene = Scene()
pairs_scene.add(pairs_source)
pairs_scene.play(pairs_anim)
# The claimed source glyph landed on its paired target part.
assert np.allclose(
    pairs_source[0].get_center(), pairs_target[1].get_center(), atol=1e-6
)

# Named refusals: a non-Mobject pair member is a TypeError (not the old
# NotImplementedError), and a pair member outside the family refuses by
# name at spec build.
try:
    matching_module.TransformMatchingStrings(
        manimlib.Text("ab"),
        manimlib.Text("ba"),
        matched_pairs=[(pairs_source[0], "not a mobject")],
    )
except TypeError as error:
    assert "matched_pairs must pair Mobjects" in str(error)
else:
    raise AssertionError("matched_pairs accepted a non-Mobject")

foreign_pair_anim = matching_module.TransformMatchingStrings(
    manimlib.Text("cd"),
    manimlib.Text("dc"),
    matched_pairs=[
        (geometry.Rectangle(width=0.5, height=0.5), pairs_target[0])
    ],
)
try:
    foreign_pair_anim._native_params()
except ValueError as error:
    assert "not a live span-map part" in str(error)
else:
    raise AssertionError("matched_pairs accepted a foreign part")

# fm-5wq.4.75: the fading base class per the pinned schema — Fade(Transform)
# stores shift/scale; FadeIn/FadeOut are its concrete subclasses. The
# Reference's bare Fade has no target and dies at begin; the portal
# constructs it and refuses by name at play instead of crashing.
assert fading.Fade.__bases__ == (manimlib.Transform,)
assert fading.FadeIn.__bases__ == (fading.Fade,)
assert fading.FadeOut.__bases__ == (fading.Fade,)
fade_signature = inspect.signature(fading.Fade)
assert tuple(fade_signature.parameters) == ("mobject", "shift", "scale", "kwargs")
assert fade_signature.parameters["scale"].default == 1.0

try:
    fading.Fade(None)
except TypeError as error:
    assert "Fade expects a Mobject" in str(error), error
    assert "NoneType" in str(error), error
else:
    raise AssertionError("Fade accepted None")

fade_base_dot = manimlib.Dot()
fade_base_scene = InteractiveScene()
fade_base_scene.add(fade_base_dot)
fade_base_probe = fading.Fade(fade_base_dot, shift=(1.0, 0.0, 0.0), scale=2.0)
assert np.allclose(fade_base_probe.shift_vect, [1.0, 0.0, 0.0])
assert fade_base_probe.scale_factor == 2.0
try:
    fade_base_scene.play(fade_base_probe, run_time=1.0 / 30.0)
except ValueError as error:
    assert "requires a target" in str(error), error
else:
    raise AssertionError("bare Fade played without a target")

# The concrete fade still plays through the rebased hierarchy. The native
# finish contract is the Reference's restore-then-remove (fading.py:76):
# fade_out finishes at final_alpha_value = 0, restoring the records, and
# only THEN the remover takes the dot out of the scene — a mid-play probe
# sees the fade actually happen.
fade_out_dot = manimlib.Dot()
fade_out_probe = manimlib.Dot().move_to((3.0, 0.0, 0.0))
fade_base_scene.add(fade_out_dot, fade_out_probe)
fade_out_alphas = []


def _fade_out_capture(mobject, dt):
    del mobject
    if dt > 0:
        fade_out_alphas.append(float(fade_out_dot.data["fill_rgba"][0, 3]))


fade_out_probe.add_updater(_fade_out_capture)
fade_base_scene.play(
    fading.FadeOut(fade_out_dot),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
fade_out_probe.remove_updater(_fade_out_capture)
assert len(fade_out_alphas) == 2, fade_out_alphas
assert math.isclose(fade_out_alphas[0], 0.5, rel_tol=0.0, abs_tol=1e-9)
assert math.isclose(fade_out_alphas[1], 0.0, rel_tol=0.0, abs_tol=1e-9)
assert fade_out_dot not in fade_base_scene.get_mobjects()
assert np.allclose(fade_out_dot.data["fill_rgba"][:, 3], 1.0)


# ------------------------------------------ FocusOn Mobject focus_point
# fm-5wq.4.77: a Mobject focus point follows live — the shrinking dot
# re-centres on the moving target every frame through the updater phase.

indication_module = importlib.import_module("manimlib.animation.indication")

focus_target = geometry.Rectangle(width=0.6, height=0.6)
focus_anim = indication_module.FocusOn(focus_target, run_time=2.0 / 30.0)
focus_scene = Scene()
focus_scene.add(focus_target)
focus_scene.play(
    focus_anim,
    focus_target.animate.shift([1.25, -0.75, 0.0]),
    run_time=2.0 / 30.0,
)
# The target moved; the shrinking dot tracked its live centre.
assert np.allclose(focus_target.get_center(), [1.25, -0.75, 0.0])
assert np.allclose(
    focus_anim.mobject.get_center(), focus_target.get_center(), atol=1e-6
), f"actual {focus_anim.mobject.get_center()} vs expected {focus_target.get_center()}"

# A numeric focus point still constructs (fm-5wq.4.52 surface untouched).
numeric_focus = indication_module.FocusOn([0.5, 0.5, 0.0], run_time=2.0 / 30.0)
assert np.allclose(numeric_focus.focus_point, [0.5, 0.5, 0.0])

# Named refusal: FocusOn(None) is a TypeError, not a crash.
try:
    indication_module.FocusOn(None)
except TypeError as error:
    assert "3D point or a Mobject" in str(error)
else:
    raise AssertionError("FocusOn accepted None")


# --------------------------------------------- Flash Mobject point follow
# fm-5wq.4.78: the native Flash composition binds every radial line, and
# one member updater re-centres the complete radial group after each native
# interpolation step so a moving Mobject point stays live through capture.

flash_target = geometry.Rectangle(width=0.6, height=0.4)
mobject_flash = indication_module.Flash(
    flash_target,
    num_lines=8,
    flash_radius=0.5,
    line_length=0.2,
    run_time=2.0 / 30.0,
)
assert len(mobject_flash.lines) == 8
assert np.allclose(mobject_flash.lines.get_center(), flash_target.get_center())
radial_centers = np.asarray(
    [line.get_center() for line in mobject_flash.lines], dtype=float
)
assert np.allclose(
    np.linalg.norm(radial_centers - flash_target.get_center(), axis=1),
    0.4,
)
flash_scene = Scene()
flash_scene.add(flash_target)
flash_scene.play(
    mobject_flash,
    flash_target.animate.shift([1.0, -0.5, 0.0]),
    run_time=2.0 / 30.0,
)
assert np.allclose(flash_target.get_center(), [1.0, -0.5, 0.0])
assert np.allclose(
    mobject_flash.lines.get_center(), flash_target.get_center(), atol=1e-6
)
assert all(line not in flash_scene.get_mobjects() for line in mobject_flash.lines)

# Named refusal: Flash(None) is a TypeError, not an indexing crash.
try:
    indication_module.Flash(None)
except TypeError as error:
    assert str(error) == "Flash point must be a 3D point or a Mobject; got NoneType"
else:
    raise AssertionError("Flash accepted None")

# fm-5wq.4.74: StreamLines over the one native RK45 integrator, and
# AnimatedStreamLines as the reference gaussian flash sweep driven by a
# Python dt-updater with substream-continuous lag draws.
vector_field_module = importlib.import_module("manimlib.mobject.vector_field")
assert vector_field_module.StreamLines.__bases__ == (manimlib.VGroup,)
assert vector_field_module.AnimatedStreamLines.__bases__ == (manimlib.VGroup,)


def _swirl_field(coords):
    coords = np.asarray(coords, dtype=float)
    return np.stack(
        [-coords[:, 1], coords[:, 0], np.zeros(len(coords))], axis=1
    )


stream_plane = manimlib.NumberPlane(x_range=(-2, 2), y_range=(-2, 2))
stream_lines = vector_field_module.StreamLines(
    _swirl_field,
    stream_plane,
    density=0.5,
    solution_time=1.0,
    n_samples_per_line=6,
)
assert len(stream_lines.submobjects) > 0
assert all(
    line.get_num_points() > 0 for line in stream_lines.submobjects
)
assert len(stream_lines._stream_virtual_times) == len(stream_lines.submobjects)
assert stream_lines._stream_rng_draws > 0

# Determinism: the same construction yields byte-identical line points.
stream_lines_again = vector_field_module.StreamLines(
    _swirl_field,
    stream_plane,
    density=0.5,
    solution_time=1.0,
    n_samples_per_line=6,
)
assert len(stream_lines_again.submobjects) == len(stream_lines.submobjects)
assert all(
    np.array_equal(a.get_points(), b.get_points())
    for a, b in zip(stream_lines.submobjects, stream_lines_again.submobjects)
)

# AnimatedStreamLines: construction snapshots tapered base profiles and the
# substream-continuous lag times; a scene wait drives the gaussian sweep and
# rewrites the stroke-width lanes.
animated_stream = vector_field_module.AnimatedStreamLines(stream_lines)
assert len(animated_stream._line_times) == len(stream_lines.submobjects)
assert all(time <= 0.0 for time in animated_stream._line_times)
stream_scene = InteractiveScene()
stream_scene.add(animated_stream)
stream_before = [
    np.asarray(line.data["stroke_width"], dtype=float).copy()
    for line in stream_lines.submobjects
]
stream_scene.wait(1.0 / 30.0)
stream_after = [
    np.asarray(line.data["stroke_width"], dtype=float)
    for line in stream_lines.submobjects
]
assert any(
    not np.allclose(before, after)
    for before, after in zip(stream_before, stream_after)
)

# Named negatives: a non-callable field and a coordinate-system-less call.
try:
    vector_field_module.StreamLines(None, stream_plane)
except TypeError as error:
    assert "StreamLines func must be" in str(error) and "callable" in str(error), error
else:
    raise AssertionError("StreamLines accepted a non-callable field")

try:
    vector_field_module.StreamLines(_swirl_field, None)
except TypeError as error:
    assert "requires a coordinate system" in str(error), error
else:
    raise AssertionError("StreamLines accepted a missing coordinate system")

# fm-5wq.14: the Reference's AnimatedStreamLines (vector_field.py:446) only
# iterates its argument and reads each line's virtual_time, so a plain group
# of timed lines animates (_2026/hairy_ball flows Lines this way); an empty
# group animates nothing; a line without virtual_time fails as it does there.
_plain_lines = manimlib.VGroup(
    *(manimlib.Line([x, 0.0, 0.0], [x + 1.0, 0.0, 0.0]) for x in (-2.0, 0.0))
)
for _plain_line in _plain_lines:
    _plain_line.virtual_time = 1.0
_plain_flow = vector_field_module.AnimatedStreamLines(_plain_lines, rate_multiple=0.5)
assert list(_plain_flow) == list(_plain_lines)
assert [line.virtual_time for line in _plain_lines] == [1.0, 1.0]
assert all(-4.0 <= line.time <= 0.0 for line in _plain_lines)
_plain_times = [line.time for line in _plain_lines]
_plain_flow.update(0.25)
assert all(
    math.isclose(line.time, before + 0.25) for line, before in zip(_plain_lines, _plain_times)
)
_plain_flow.clear_updaters()
assert len(vector_field_module.AnimatedStreamLines(manimlib.VGroup())) == 0
try:
    vector_field_module.AnimatedStreamLines(manimlib.VGroup(manimlib.Line()))
except AttributeError as error:
    assert "virtual_time" in str(error), error
else:
    raise AssertionError("AnimatedStreamLines accepted a line without virtual_time")
try:
    vector_field_module.AnimatedStreamLines(3)
except TypeError as error:
    assert "iterable of lines" in str(error), error
else:
    raise AssertionError("AnimatedStreamLines accepted a non-iterable")

# fm-5wq.4.81: Prismify over a VMobject family — one native extrusion tree
# per pointful member, in family order, each matching its own source style.
prism_family_square = manimlib.Square(side_length=1.0).move_to((-1.5, 0.0, 0.0))
prism_family_square.set_stroke(manimlib.RED, width=3.0)
prism_family_triangle = manimlib.Triangle().move_to((1.5, 0.0, 0.0))
prism_family_triangle.set_stroke(manimlib.BLUE, width=3.0)
prism_family = manimlib.Prismify(
    manimlib.VGroup(prism_family_square, prism_family_triangle),
    depth=0.5,
)
assert len(prism_family.submobjects) == 2
assert all(
    len(piece.family_members_with_points()) > 0
    for piece in prism_family.submobjects
)
# Each extrusion spans its source's footprint plus the depth extrusion.
assert prism_family.submobjects[0].get_depth() > 0.0
assert np.allclose(
    prism_family.submobjects[0].get_center()[:2],
    prism_family_square.get_center()[:2],
    atol=1e-6,
)
assert np.allclose(
    prism_family.submobjects[1].get_center()[:2],
    prism_family_triangle.get_center()[:2],
    atol=1e-6,
)

# The single-VMobject path is untouched, and the negatives stay named.
prism_single = manimlib.Prismify(manimlib.Square(side_length=1.0), depth=0.5)
assert len(prism_single.family_members_with_points()) > 0

try:
    manimlib.Prismify(None)
except TypeError as error:
    assert "Prismify source must be a VMobject" in str(error), error
else:
    raise AssertionError("Prismify accepted None")

try:
    manimlib.Prismify(manimlib.VGroup(VMobject()))
except ValueError as error:
    assert "no pointful VMobject members" in str(error), error
else:
    raise AssertionError("Prismify accepted a point-less family")


# ------------------------------------------ DecimalNumber complex values
# Real and imaginary components use the native f64 number shelf. The portal
# composes both components and allows live transitions between display modes.

# A zero-real complex renders natively as the imaginary component + "i".
imag_number = manimlib.DecimalNumber(complex(0, 2))
assert imag_number.get_value() == complex(0, 2)
assert len(imag_number.submobjects) > 0
assert any(child.has_points() for child in imag_number.submobjects)

# set_value to another pure-imaginary value updates in place.
imag_number.set_value(complex(0, -3.5))
assert imag_number.get_value() == complex(0, -3.5)
assert any(child.has_points() for child in imag_number.submobjects)

# A zero-imag complex reduces to the plain real path.
real_ish = manimlib.DecimalNumber(complex(3, 0))
assert real_ish.get_value() == complex(3, 0)
assert any(child.has_points() for child in real_ish.submobjects)
real_ish.set_value(4.0)
assert real_ish.get_value() == 4.0

general_complex_number = manimlib.DecimalNumber(complex(1, 2))
assert general_complex_number.get_value() == 1 + 2j
assert general_complex_number.get_tex() == "1.00+2.00i"
assert len(general_complex_number) == len(general_complex_number.num_string)

imag_number.set_value(1.0)
assert imag_number.get_value() == 1.0
assert imag_number.get_tex() == "1.00"
imag_number.set_value(2j)
assert imag_number.get_value() == 2j
assert imag_number.get_tex() == "2.00i"

# Real infinities render as readout text since 31f0a52d, and a real NaN reads
# "nan" as in the pinned Reference (fm-5wq.30). Non-numeric values refuse.
assert manimlib.DecimalNumber(float("nan")).get_tex() == "nan"

try:
    manimlib.DecimalNumber("seven")
except TypeError as error:
    assert "real or complex number" in str(error)
else:
    raise AssertionError("DecimalNumber accepted a string")

# Integer inherits DecimalNumber's native formatting/style kwargs while
# preserving its rounded get_value contract.
plain_integer = manimlib.Integer(7)
signed_integer = manimlib.Integer(
    7, include_sign=True, font_size=30, color=manimlib.GREEN
)
assert signed_integer.get_value() == 7
assert len(signed_integer.submobjects) > len(plain_integer.submobjects)
assert all(
    child.get_fill_color() == manimlib.GREEN
    for child in signed_integer.family_members_with_points()
)

failed_integer = manimlib.Integer.__new__(manimlib.Integer)
try:
    manimlib.Integer.__init__(failed_integer, 7, bogus=True)
except NotImplementedError as error:
    assert str(error) == (
        "Integer() keyword(s) not yet routed to the native builder: bogus"
    )
else:
    raise AssertionError("Integer silently dropped bogus")
assert not hasattr(failed_integer, "submobjects")

# fm-5wq.4.83: in-place Transform — a None target resolves at play time to
# a copy of the mobject's current state through Transform.create_target
# (the schema method), so bare Transform(mob) plays through the native
# transform kind.
inplace_dot = manimlib.Dot().move_to((1.0, 0.0, 0.0))
inplace_scene = InteractiveScene()
inplace_scene.add(inplace_dot)
inplace_probe = transform_module.Transform(inplace_dot)
assert inplace_probe.target_mobject is None
inplace_created = inplace_probe.create_target()
assert inplace_created is not inplace_dot
assert np.array_equal(inplace_created.get_points(), inplace_dot.get_points())
inplace_scene.play(
    inplace_probe, run_time=2.0 / 30.0, rate_func=manimlib.linear
)
assert inplace_probe.target_mobject is not None
assert np.allclose(inplace_dot.get_center(), [1.0, 0.0, 0.0], atol=1e-9)

# The deferred resolution respects the class contracts around it: fades
# stay target-less (bare Fade keeps its named play-time refusal, pinned
# above), while cyclic replacements resolve their cycled target group.
assert fading.FadeIn(manimlib.Dot())._native_target() is None
assert (
    transform_module.CyclicReplace(
        manimlib.Dot(), manimlib.Dot().shift((1.0, 0.0, 0.0))
    )._native_target()
    is not None
)

# Named negative: a non-Mobject Transform source.
try:
    transform_module.Transform(None)
except TypeError as error:
    assert "Transform expects a Mobject" in str(error), error
    assert "NoneType" in str(error), error
else:
    raise AssertionError("Transform accepted None")


# --------------------------- AddTextWordByWord nested glyph flattening
# fm-5wq.4.82: deliberately authored nested Tex families (sub-paths
# [part, glyph]) reveal word-by-word through the per-part flattening plan.
# Modern Tex does not create argument containers, but the public family API
# still supports grouping its glyphs. The finished frame restores those groups.


def _author_tex_part_groups(tex):
    originals = [tex._string_submobject(i) for i in range(len(tex._string_sub_paths))]
    parts = [tex.get_part_by_tex(part) for part in tex.tex_strings]
    paths = {id(child): [i, j] for i, part in enumerate(parts) for j, child in enumerate(part)}
    assert len(paths) == len(originals), "the fixture must partition all native glyphs"
    tex.set_submobjects(parts)
    tex._string_sub_paths = [paths[id(child)] for child in originals]
    return tex


nested_tex = _author_tex_part_groups(manimlib.Tex("a b", "c d"))
assert any(len(path) == 2 for path in nested_tex._string_sub_paths)
nested_parts_before = [
    len(part.submobjects) for part in nested_tex.submobjects
]
nested_anim = creation_animation.AddTextWordByWord(
    nested_tex, run_time=4.0 / 30.0
)
nested_scene = Scene()
nested_probe = geometry.Rectangle(width=1.0, height=1.0)
nested_glyph_counts = []
nested_scene.play(
    nested_anim,
    update_animation.UpdateFromFunc(
        nested_probe,
        lambda mob: nested_glyph_counts.append(
            sum(len(part.submobjects) for part in nested_tex.submobjects)
        ),
    ),
    run_time=4.0 / 30.0,
)
# Glyphs only accumulate on word boundaries and the reveal crossed the
# part seam mid-flight.
assert nested_glyph_counts == sorted(nested_glyph_counts), nested_glyph_counts
assert set(nested_glyph_counts) <= {0, 1, 2, 3, 4}, nested_glyph_counts
# The finished frame is the whole family with the original structure.
assert [
    len(part.submobjects) for part in nested_tex.submobjects
] == nested_parts_before
assert len(nested_tex.submobjects) == len(nested_parts_before)

# The same flattening plan covers the letter grain (fm-5wq.4.59's nested
# refusal is retired).
nested_letters = _author_tex_part_groups(manimlib.Tex("a b", "c d"))
letters_before = [
    len(part.submobjects) for part in nested_letters.submobjects
]
letters_scene = Scene()
letters_scene.play(
    creation_animation.AddTextLetterByLetter(
        nested_letters, run_time=4.0 / 30.0
    ),
    run_time=4.0 / 30.0,
)
assert [
    len(part.submobjects) for part in nested_letters.submobjects
] == letters_before

# The non-StringMobject refusal is unchanged.
try:
    creation_animation.AddTextWordByWord(
        geometry.Rectangle(width=1.0, height=1.0)
    )
except AssertionError:
    pass
else:
    raise AssertionError("AddTextWordByWord accepted a non-StringMobject")

# fm-5wq.4.84: ThreeDAxes z_normal routes onto the native builder's normal
# seam. The normal reorients the z-axis tick construction (planes.rs: the
# axis line itself stays along z, the Reference's exact behaviour).
znormal_default_axes = manimlib.ThreeDAxes()
znormal_out_axes = manimlib.ThreeDAxes(z_normal=manimlib.OUT)
znormal_down_axes = manimlib.ThreeDAxes(z_normal=manimlib.DOWN)


def _znormal_axis_points(axes):
    return np.concatenate(
        [
            member.get_points()
            for member in axes.z_axis.family_members_with_points()
        ]
    )


def _znormal_axis_spans(axes):
    points = _znormal_axis_points(axes)
    return points.max(axis=0) - points.min(axis=0)


# The axis extends along z for every normal (the normal reorients ticks,
# never the axis itself)...
for _znormal_probe in (znormal_default_axes, znormal_out_axes):
    _znormal_spans = _znormal_axis_spans(_znormal_probe)
    assert _znormal_spans[2] > _znormal_spans[0]
    assert _znormal_spans[2] > _znormal_spans[1]
# ...the DOWN normal is exactly the default...
assert np.array_equal(
    _znormal_axis_points(znormal_down_axes),
    _znormal_axis_points(znormal_default_axes),
)
# ...and a different normal reorients the tick construction.
znormal_out_points = _znormal_axis_points(znormal_out_axes)
znormal_default_points = _znormal_axis_points(znormal_default_axes)
assert znormal_out_points.shape != znormal_default_points.shape or not (
    np.allclose(znormal_out_points, znormal_default_points)
)

try:
    manimlib.ThreeDAxes(z_normal="nope")
except TypeError as error:
    assert "z_normal must be a 3-vector" in str(error), error
else:
    raise AssertionError("ThreeDAxes accepted a non-vector z_normal")


# --------------------------- Tex.make_number_changeable nested part groups
# fm-5wq.4.85: an authored nested Tex replaces the selected number span
# with a live DecimalNumber inside its owning part — the part structure
# survives and later glyphs in the part re-index.

nested_changeable_tex = _author_tex_part_groups(manimlib.Tex("y =", "0.50 x"))
assert any(len(path) == 2 for path in nested_changeable_tex._string_sub_paths)
nested_part_count = len(nested_changeable_tex.submobjects)
nested_decimal = nested_changeable_tex.make_number_changeable("0.50")
assert isinstance(nested_decimal, manimlib.DecimalNumber)
# The root part list is untouched and the decimal lives inside a part.
assert len(nested_changeable_tex.submobjects) == nested_part_count
assert any(
    nested_decimal in part.submobjects
    for part in nested_changeable_tex.submobjects
)
# Every surviving path is still two-level and in-range for its part.
assert all(
    len(path) == 2
    and path[1]
    < len(nested_changeable_tex.submobjects[path[0]].submobjects)
    for path in nested_changeable_tex._string_sub_paths
)
# set_value updates the shown digits through Scene.play.
nested_change_scene = Scene()
nested_change_scene.add(nested_changeable_tex)
nested_change_scene.play(
    numbers_animation.ChangeDecimalToValue(
        nested_decimal, 2.75, run_time=2.0 / 30.0
    )
)
assert math.isclose(nested_decimal.get_value(), 2.75)
assert any(child.has_points() for child in nested_decimal.submobjects)

# The Reference's empty-result contract holds on nested families too: an
# unknown substring and an index past the last occurrence both return an
# empty VMobject, never a crash.
missing = manimlib.Tex("a =", "1.00").make_number_changeable("9.99")
assert isinstance(missing, VMobject) and len(missing.submobjects) == 0
past = manimlib.Tex("b =", "1.00").make_number_changeable("1.00", index=5)
assert isinstance(past, VMobject) and len(past.submobjects) == 0

# fm-5wq.4.86: TracingTail's function half — a point-returning callable is
# a TracedPath with the tail's finite window and (0,3)/(0,1) tapers, grown
# by the same verbatim update_path in the Python-updater window; the
# mobject half keeps the native tracer untouched.
tail_probe_state = {"x": 0.0}


def _tail_probe_point():
    return (tail_probe_state["x"], 0.0, 0.0)


func_tail = manimlib.TracingTail(_tail_probe_point, time_traced=0.5)
func_tail_scene = InteractiveScene()
func_tail_scene.add(func_tail)
tail_probe_state["x"] = 1.0
func_tail_scene.wait(2.0 / 30.0)
assert func_tail.get_num_points() > 0
assert np.allclose(func_tail.get_points()[:, 1:], 0.0, atol=1e-9)
func_tail_widths = np.asarray(
    func_tail.data["stroke_width"], dtype=float
).reshape(-1)
assert func_tail_widths[0] < func_tail_widths[-1]

# The native path seeds history against a bound target but publishes no
# geometry until its first positive scene update.
tail_traced_dot = manimlib.Dot()
func_tail_scene.add(tail_traced_dot)
mobject_tail = manimlib.TracingTail(tail_traced_dot)
assert mobject_tail.get_num_points() == 0
func_tail_scene.add(mobject_tail)
func_tail_scene.update_frame(0)
assert mobject_tail.get_num_points() == 0
tail_traced_dot.shift(manimlib.RIGHT)
func_tail_scene.update_frame(1.0 / 15)
assert np.allclose(mobject_tail.get_start(), manimlib.ORIGIN)
assert np.allclose(mobject_tail.get_end(), manimlib.RIGHT)

# Named negative: neither a Mobject nor a callable.
try:
    manimlib.TracingTail(None)
except TypeError as error:
    assert "Mobject or a point-returning" in str(error), error
    assert "NoneType" in str(error), error
else:
    raise AssertionError("TracingTail accepted None")


# --------------------------- nested AnimationGroup Python callbacks
# fm-5wq.4.88: Python-driven members inside compositions build the same
# python_callback placeholder slots, and one driver callback mirrors the
# native window math (build_timings / timeline_position) for the leaves.

# Public timing inspection and mixed native/Python dispatch share Choreo's
# interval builder. Unequal child durations distinguish the recurrence from
# equal-index windows, and nested/default/builder members retain real durations.
timing_children = [
    manimlib.FadeIn(geometry.Square(), run_time=duration)
    for duration in (2.0, 3.0, 1.0)
]
timing_group = manimlib.AnimationGroup(*timing_children, lag_ratio=0.25)
assert [entry[0] for entry in timing_group.anims_with_timings] == timing_children
assert [entry[1:] for entry in timing_group.anims_with_timings] == [
    (0.0, 2.0), (0.5, 3.5), (1.25, 2.25),
]
assert timing_group.max_end_time == timing_group.get_run_time() == 3.5
assert timing_group.build_animations_with_timings(0.5) is None
assert timing_group.calculate_max_end_time() is None
assert timing_group.max_end_time == 4.0 and timing_group.run_time == 3.5
timing_group.run_time = -1
timing_group.calculate_max_end_time()
assert timing_group.run_time == 4.0
timing_default = manimlib.FadeIn(geometry.Square())
timing_nested = manimlib.Succession(timing_group, timing_default)
assert timing_nested.anims_with_timings == [(timing_group, 0.0, 4.0), (timing_default, 4.0, 5.0)]
assert timing_nested.get_run_time() == 5.0
timing_builder = geometry.Square().animate(run_time=0.25).shift([1, 0, 0])
timing_built_group = manimlib.AnimationGroup(timing_builder)
assert isinstance(timing_built_group.animations[0], manimlib.Animation)
assert timing_built_group.get_run_time() == 0.25

group_cb_scene = Scene()
group_cb_rect = geometry.Rectangle(width=1.0, height=1.0)
group_cb_decimal = manimlib.DecimalNumber(0.0)
group_cb_scene.add(group_cb_rect, group_cb_decimal)
group_cb_seen = []
group_cb_probe = geometry.Rectangle(width=0.2, height=0.2)
group_cb_probe.add_updater(
    lambda mob: group_cb_seen.append(group_cb_decimal.get_value())
)
group_cb_scene.add(group_cb_probe)
group_cb_scene.play(
    manimlib.AnimationGroup(
        # Equal member windows retain an intermediate nominal sample when
        # the group is compressed to two frames. A one-second native leaf
        # would make the shorter decimal leaf complete at the first frame.
        manimlib.FadeIn(group_cb_rect, run_time=2.0 / 30.0),
        numbers_animation.ChangeDecimalToValue(
            group_cb_decimal, 6.0, run_time=2.0 / 30.0
        ),
        run_time=2.0 / 30.0,
    ),
    run_time=2.0 / 30.0,
)
assert math.isclose(group_cb_decimal.get_value(), 6.0)
assert any(0.0 < value < 6.0 for value in group_cb_seen), group_cb_seen

# Nested-in-nested: a Succession of Python leaves inside an AnimationGroup
# still drives every leaf to completion.
deep_scene = Scene()
deep_rect = geometry.Rectangle(width=1.0, height=1.0)
deep_decimal = manimlib.DecimalNumber(1.0)
deep_scene.add(deep_rect, deep_decimal)
deep_scene.play(
    manimlib.AnimationGroup(
        manimlib.FadeIn(deep_rect),
        manimlib.Succession(
            specialized_animation.Delay(run_time=1.0 / 30.0),
            numbers_animation.ChangeDecimalToValue(
                deep_decimal, 9.0, run_time=1.0 / 30.0
            ),
        ),
        run_time=2.0 / 30.0,
    ),
    run_time=2.0 / 30.0,
)
assert math.isclose(deep_decimal.get_value(), 9.0)

# A non-Animation nested member is the Reference's prepare_animation
# TypeError (animation.py:216).
try:
    Scene().play(
        manimlib.AnimationGroup(
            manimlib.FadeIn(geometry.Rectangle(width=1.0, height=1.0)),
            42,
        )
    )
except TypeError as error:
    assert str(error) == "Object 42 cannot be converted to an animation", error
else:
    raise AssertionError("AnimationGroup accepted a non-Animation member")


# --------------------- DecimalNumber.set_value with background rectangle
# fm-5wq.4.92: root-level records (the background rectangle) rebuild
# through the live-state become seam — the fm-p107 refusal is retired.

bg_decimal = manimlib.DecimalNumber(1.25, include_background_rectangle=True)
assert bg_decimal.n_records() != 0  # the rectangle lives on the root
bg_glyphs_before = [
    child for child in bg_decimal.submobjects if child.has_points()
]
assert bg_glyphs_before
bg_decimal.set_value(2.5)
assert math.isclose(bg_decimal.get_value(), 2.5)
assert bg_decimal.n_records() != 0  # the rectangle survived the rebuild
assert any(child.has_points() for child in bg_decimal.submobjects)

# Digit-count changes rebuild too (align_family pads the families).
bg_decimal.set_value(10.75)
assert math.isclose(bg_decimal.get_value(), 10.75)
assert any(child.has_points() for child in bg_decimal.submobjects)

# The live path still plays: ChangingDecimal drives a background-rectangle
# number through Scene.play.
bg_scene = Scene()
bg_live = manimlib.DecimalNumber(0.0, include_background_rectangle=True)
bg_scene.add(bg_live)
bg_scene.play(
    numbers_animation.ChangeDecimalToValue(bg_live, 3.0, run_time=2.0 / 30.0)
)
assert math.isclose(bg_live.get_value(), 3.0)
assert bg_live.n_records() != 0

# Named negative: a non-numeric set_value is the 4.80 TypeError.
try:
    bg_decimal.set_value("nope")
except TypeError as error:
    assert "real or complex number" in str(error)
else:
    raise AssertionError("set_value accepted a string")

# fm-5wq.4.91: turn_animation_into_updater over native-kind classes — the
# Transform family uses the production semantic installer, so both straight
# and curved Transform paths drive as live updaters. (The FadeIn
# gate stanza above moved to ShowCreation, which stays seam-refused.)
try:
    update_utils.turn_animation_into_updater(None)
except TypeError as error:
    assert "requires an Animation" in str(error), error
    assert "NoneType" in str(error), error
else:
    raise AssertionError("turn_animation_into_updater accepted None")

fade_updater_dot = manimlib.Dot()
fade_updater_dot.set_fill(manimlib.BLUE, opacity=1.0)
fade_updater_scene = InteractiveScene()
fade_updater_scene.add(fade_updater_dot)
update_utils.turn_animation_into_updater(
    manimlib.FadeOut(fade_updater_dot),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
fade_updater_scene.wait(2.0 / 30.0)
fade_mid_alpha = float(fade_updater_dot.data["fill_rgba"][0, 3])
assert 0.0 < fade_mid_alpha < 1.0, fade_mid_alpha
fade_updater_scene.wait(3.0 / 30.0)
# FadeOut's final_alpha_value is 0 (fading.py:76): finish restores, and
# the exhausted updater detaches.
assert np.allclose(fade_updater_dot.data["fill_rgba"][:, 3], 1.0)
assert not fade_updater_dot.updaters

transform_updater_square = manimlib.Square(side_length=1.0)
transform_updater_target = manimlib.Square(side_length=1.0).shift(
    (1.0, 0.0, 0.0)
)
fade_updater_scene.add(transform_updater_square)
update_utils.turn_animation_into_updater(
    manimlib.Transform(transform_updater_square, transform_updater_target),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
fade_updater_scene.wait(2.0 / 30.0)
transform_mid_x = float(transform_updater_square.get_center()[0])
assert 0.0 < transform_mid_x < 1.0, transform_mid_x
fade_updater_scene.wait(3.0 / 30.0)
assert np.allclose(
    transform_updater_square.get_center(), [1.0, 0.0, 0.0], atol=1e-9
)

# The installed path kernel must traverse the semicircle, not a straight
# lerp that happens to reach the same endpoint. Its midpoint is analytic.
arc_updater_dot = manimlib.Dot()
arc_updater_scene = InteractiveScene()
arc_updater_scene.add(arc_updater_dot)
update_utils.turn_animation_into_updater(
    manimlib.Transform(
        arc_updater_dot,
        arc_updater_dot.copy().shift((1.0, 0.0, 0.0)),
        path_arc=manimlib.PI,
    ),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
arc_updater_scene.wait(2.0 / 30.0)
assert np.allclose(arc_updater_dot.get_center(), [0.5, -0.5, 0.0], atol=1e-6)
arc_updater_scene.wait(3.0 / 30.0)
assert np.allclose(arc_updater_dot.get_center(), [1.0, 0.0, 0.0], atol=1e-6)
assert not arc_updater_dot.updaters
assert not arc_updater_dot.locked_data_keys


# ------------------------------ camera-frame builders merge per play
# fm-5wq.4.93: several camera-frame builders in one Scene.play merge with
# the Reference's play-order semantics — the last builder's target wins
# every frame, and one engine lerp reproduces it.

merge_cam_scene = Scene()
merge_cam_frame = merge_cam_scene.frame
merge_cam_scene.play(
    merge_cam_frame.animate.shift([1.0, 0.0, 0.0]),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(merge_cam_frame.get_center(), [1.0, 0.0, 0.0])

# Two builders in one play: play-order last-wins, exactly the Reference's
# same-mobject transform overwrite.
merge_cam_scene.play(
    merge_cam_frame.animate.reorient(20, 60, 0),
    merge_cam_frame.animate.reorient(6, 84, 0),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.isclose(merge_cam_frame.get_theta(), 6 * manimlib.DEG, atol=1e-12)
assert np.isclose(merge_cam_frame.get_phi(), 84 * manimlib.DEG, atol=1e-12)

# A non-CameraFrame camera pair is a named TypeError at the play seam.
try:
    merge_cam_scene._play_animations(
        [],
        [],
        (geometry.Rectangle(width=1.0, height=1.0), None),
        None,
        None,
        None,
    )
except TypeError:
    pass
else:
    raise AssertionError("_play_animations accepted a non-CameraFrame pair")

# fm-5wq.4: an explicit top-level Transform whose mobject is the camera
# frame rides the same engine camera track as frame.animate. Nesting one
# inside a composition still refuses, and a Python-callback animation of
# the frame names the missing camera-track callback seam.
explicit_cam_scene = Scene()
explicit_cam_frame = explicit_cam_scene.frame
explicit_cam_start_center = explicit_cam_frame.get_center().copy()
explicit_cam_start_width = float(explicit_cam_frame.get_width())
explicit_cam_target = explicit_cam_frame.copy()
explicit_cam_target.shift([1.0, 0.5, 0.0])
explicit_cam_target.scale(2.0)
explicit_cam_scene.play(
    manimlib.Transform(explicit_cam_frame, explicit_cam_target),
    run_time=1.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(
    explicit_cam_frame.get_center(),
    explicit_cam_start_center + [1.0, 0.5, 0.0],
)
assert np.isclose(
    explicit_cam_frame.get_width(), explicit_cam_start_width * 2.0
)
# fm-5wq.4.143: nested camera choreography executes on the shared native
# frame clock inside AnimationGroup / Succession compositions.
explicit_cam_scene.play(
    manimlib.AnimationGroup(
        manimlib.Transform(explicit_cam_frame, explicit_cam_frame.copy())
    ),
    run_time=1.0 / 30.0,
)


class _CallbackFrameAnimation(Animation):
    def interpolate_mobject(self, alpha):
        return None


# fm-5wq.4 / ab75ba3b: authored camera callbacks execute on the shared
# Choreo camera callback track.
explicit_cam_scene.play(
    _CallbackFrameAnimation(explicit_cam_frame, run_time=1.0 / 30.0)
)



# ------------------------------------------------ LaggedStartMap
# fm-5wq.4.95: one member animation per family child through the native
# lagged_start composition.

lsm_scene = Scene()
lsm_first = geometry.Rectangle(width=0.5, height=0.5)
lsm_second = geometry.Rectangle(width=0.7, height=0.7)
lsm_second.shift([1.5, 0.0, 0.0])
lsm_group = manimlib.VGroup(lsm_first, lsm_second)
lsm_scene.add(lsm_group)
assert np.allclose(lsm_first.data["stroke_rgba"][:, 3], 1.0)
lsm_anim = manimlib.LaggedStartMap(
    manimlib.FadeOut, lsm_group, run_time=2.0 / 30.0
)
assert len(lsm_anim.animations) == 2
assert all(
    isinstance(member, manimlib.FadeOut) for member in lsm_anim.animations
)
# Composed FadeOut members follow the same restore-then-remove finish
# contract as top-level fades (fading.py:76, delegated through the
# composition's member cleanup — the SCTFO pins above prove it), so the
# honest fade observable is the mid-play probe: both children reach
# stroke alpha exactly 0.0 at the last captured frame (group alpha 1.0,
# before the finish epilogue), and the records come back restored.
lsm_probe = manimlib.Dot().move_to((3.0, 0.0, 0.0))
lsm_scene.add(lsm_probe)
lsm_frames = []


def _lsm_capture(mobject, dt):
    del mobject
    if dt > 0:
        lsm_frames.append(
            (
                float(lsm_first.data["stroke_rgba"][0, 3]),
                float(lsm_second.data["stroke_rgba"][0, 3]),
            )
        )


lsm_probe.add_updater(_lsm_capture)
lsm_scene.play(lsm_anim, run_time=2.0 / 30.0)
lsm_probe.remove_updater(_lsm_capture)
assert len(lsm_frames) == 2, lsm_frames
assert any(first < 1.0 for first, _ in lsm_frames), lsm_frames
assert any(second < 1.0 for _, second in lsm_frames), lsm_frames
assert np.allclose(lsm_frames[-1], [0.0, 0.0], atol=1e-9), lsm_frames
# Finish restored both children's records (the composed remover contract).
assert np.allclose(lsm_first.data["stroke_rgba"][:, 3], 1.0)
assert np.allclose(lsm_second.data["stroke_rgba"][:, 3], 1.0)

# Mapped kwargs reach every member; lag_ratio stays the group's.
lsm_shifted = manimlib.LaggedStartMap(
    manimlib.FadeIn,
    manimlib.VGroup(
        geometry.Rectangle(width=0.5, height=0.5),
        geometry.Rectangle(width=0.6, height=0.6),
    ),
    shift=[0.0, 1.0, 0.0],
    lag_ratio=0.25,
)
assert lsm_shifted.lag_ratio == 0.25
assert all(
    np.allclose(member.shift_vect, [0.0, 1.0, 0.0])
    for member in lsm_shifted.animations
)

# Named refusals: a non-callable constructor and a non-Mobject family.
try:
    manimlib.LaggedStartMap(None, lsm_group)
except TypeError as error:
    assert "animation constructor" in str(error)
else:
    raise AssertionError("LaggedStartMap accepted None")

try:
    manimlib.LaggedStartMap(manimlib.FadeOut, [1, 2, 3])
except TypeError as error:
    assert "Mobject family" in str(error)
else:
    raise AssertionError("LaggedStartMap accepted a non-Mobject family")

# fm-5wq.4.94: the Uncreate leftover — the full creation.py:56 signature is
# spelled (its own schema defaults were previously refused as unrouted
# kwargs), and the reversal plays through the native uncreate kind.
uncreate_signature = inspect.signature(creation.Uncreate)
assert tuple(uncreate_signature.parameters) == (
    "mobject",
    "rate_func",
    "remover",
    "should_match_start",
    "kwargs",
)
uncreate_probe = creation.Uncreate(
    manimlib.Square(side_length=1.0),
    remover=True,
    should_match_start=True,
)
assert uncreate_probe.should_match_start is True

# A constant-rate probe freezes the reversed reveal mid-window: the end
# state is exactly pointwise_become_partial(start, 0, r).
uncreate_scene = InteractiveScene()
uncreate_square = manimlib.Square(side_length=1.0)
uncreate_square.set_stroke(manimlib.WHITE, width=4.0)
uncreate_reference = uncreate_square.copy()
uncreate_expected = uncreate_square.copy()
uncreate_expected.pointwise_become_partial(uncreate_reference, 0.0, 0.5)
uncreate_scene.add(uncreate_square)
uncreate_scene.play(
    creation.Uncreate(uncreate_square, rate_func=lambda t: 0.5),
    run_time=2.0 / 30.0,
)
assert np.allclose(
    uncreate_square.get_points(), uncreate_expected.get_points()
)

# The default run is a remover: the line leaves the scene.
uncreate_gone_square = manimlib.Square(side_length=1.0)
uncreate_scene.add(uncreate_gone_square)
uncreate_scene.play(
    creation.Uncreate(uncreate_gone_square), run_time=2.0 / 30.0
)
assert uncreate_gone_square not in uncreate_scene.get_mobjects()

# Named negatives: a non-VMobject target and a non-True remover.
try:
    creation.Uncreate(None)
except TypeError as error:
    assert "requires a Mobject family" in str(error), error
    assert "NoneType" in str(error), error
else:
    raise AssertionError("Uncreate accepted None")

uncreate_retained = creation.Uncreate(manimlib.Square(), remover=False)
assert uncreate_retained.remover is False

# A family that is not all VMobject or Surface plays ShowCreation the way the
# Reference plays it: Python-driven, each member revealing itself through its
# own pointwise_become_partial. A DotCloud grows by its point records
# (point_cloud_mobject.py:99). A plain Group holds still while its surfaces
# reveal.
_cloud = manimlib.DotCloud([(float(x), 0.0, 0.0) for x in range(10)])
_grow = creation.ShowCreation(_cloud, rate_func=manimlib.linear)
assert not _grow._native_kind
_grow.begin()
assert _cloud.get_num_points() == 0
_grow.interpolate(0.5)
assert _cloud.get_num_points() == 5
_grow.finish()
assert _cloud.get_num_points() == 10
_cloud_scene = InteractiveScene()
_cloud_scene.play(creation.ShowCreation(_cloud), run_time=2.0 / 30.0)
assert _cloud.get_num_points() == 10 and _cloud in _cloud_scene.get_mobjects()
_undone = manimlib.DotCloud([(float(x), 1.0, 0.0) for x in range(4)])
_cloud_scene.add(_undone)
_cloud_scene.play(creation.Uncreate(_undone), run_time=2.0 / 30.0)
assert _undone not in _cloud_scene.get_mobjects()
_shells = manimlib.Group(manimlib.Sphere(), manimlib.Sphere().shift(manimlib.RIGHT))
_shell_points = _shells.get_all_points().copy()
_cloud_scene.play(creation.ShowCreation(_shells, lag_ratio=0.05), run_time=2.0 / 30.0)
assert np.allclose(_shells.get_all_points(), _shell_points)

# fm-5wq.4.97: one Scene.play mixing a native-kind animation with a
# Python-authored one — the per-animation release loop yields every
# boundary in index order, so the Python member interleaves with the
# native segment instead of refusing.
mix_faded_dot = manimlib.Dot()
mix_moved_dot = manimlib.Dot()
mix_scene = InteractiveScene()
mix_scene.add(mix_faded_dot, mix_moved_dot)
mix_update_calls = []


def _mix_nudge(mobject):
    mix_update_calls.append(True)
    mobject.shift((0.05, 0.0, 0.0))


mix_scene.play(
    manimlib.FadeOut(mix_faded_dot),
    manimlib.UpdateFromFunc(mix_moved_dot, _mix_nudge),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
# The native member completed: FadeOut removed its dot from the scene.
assert mix_faded_dot not in mix_scene.get_mobjects()
# The Python member ran per frame and its mutations landed.
assert mix_update_calls
assert float(mix_moved_dot.get_center()[0]) > 0.0

# A non-Animation play member is the Reference's prepare_animation TypeError
# (animation.py:216), not a portal capability gap.
try:
    mix_scene.play("nope")
except TypeError as error:
    assert str(error) == "Object nope cannot be converted to an animation", error
else:
    raise AssertionError("Scene.play accepted a non-Animation member")


# ------------------------------------ FadeTransform leftover kwargs
# fm-5wq.4.98: stretch and dim_to_match route to the native builder's own
# knobs; the unrouted refusal retires.

ft_scene = Scene()
ft_source = geometry.Rectangle(width=2.0, height=0.5)
ft_target = geometry.Rectangle(width=0.5, height=2.0)
ft_target.shift([1.0, 0.5, 0.0])
ft_scene.add(ft_source)
ft_default = manimlib.FadeTransform(ft_source, ft_target, run_time=2.0 / 30.0)
assert ft_default._native_params() == {"stretch": True, "dim_to_match": 1}
ft_scene.play(ft_default, run_time=2.0 / 30.0)
assert any(
    ft_target in mob.get_family() for mob in ft_scene.mobjects
) or ft_target in ft_scene.mobjects

# Non-default values are carried in the built spec and play cleanly.
ft2_scene = Scene()
ft2_source = geometry.Rectangle(width=2.0, height=0.5)
ft2_target = geometry.Rectangle(width=0.5, height=2.0)
ft2_scene.add(ft2_source)
ft_tuned = manimlib.FadeTransform(
    ft2_source, ft2_target, stretch=False, dim_to_match=0, run_time=2.0 / 30.0
)
assert ft_tuned._native_params() == {"stretch": False, "dim_to_match": 0}
ft2_scene.play(ft_tuned, run_time=2.0 / 30.0)

# Named refusals: a non-Mobject endpoint, an out-of-range dimension, and
# the pieces path still refusing non-default knobs by keyword name.
try:
    manimlib.FadeTransform(None, geometry.Rectangle(width=1.0, height=1.0))
except TypeError as error:
    assert "source Mobject and a target Mobject" in str(error)
else:
    raise AssertionError("FadeTransform accepted None")

try:
    manimlib.FadeTransform(
        geometry.Rectangle(width=1.0, height=1.0),
        geometry.Rectangle(width=1.0, height=1.0),
        dim_to_match=7,
    )
except ValueError as error:
    assert "dim_to_match must be 0, 1, or 2" in str(error)
else:
    raise AssertionError("FadeTransform accepted dim_to_match=7")

ftp = manimlib.FadeTransformPieces(
    geometry.Rectangle(width=1.0, height=1.0),
    geometry.Rectangle(width=1.0, height=1.0),
    stretch=False,
)
assert ftp.stretch is False

# fm-5wq.4.99: .animate anim args — path_arc/path_arc_axis ride the native
# transform's arc surface; anything else stays a refusal naming the key.
animate_shift_dot = manimlib.Dot()
animate_args_scene = InteractiveScene()
animate_args_scene.add(animate_shift_dot)
animate_args_scene.play(
    animate_shift_dot.animate.shift(manimlib.RIGHT),
    run_time=2.0 / 30.0,
    rate_func=manimlib.linear,
)
assert np.allclose(animate_shift_dot.get_center(), [1.0, 0.0, 0.0], atol=1e-9)

# A clockwise (−π) builder arc from (1, 0) to (−1, 0) dips through (0, −1)
# at the constant-rate halfway probe — the arc, not the chord.
animate_arc_dot = manimlib.Dot().move_to((1.0, 0.0, 0.0))
animate_args_scene.add(animate_arc_dot)
animate_args_scene.play(
    animate_arc_dot.animate(
        path_arc=-math.pi, rate_func=lambda t: 0.5
    ).move_to((-1.0, 0.0, 0.0)),
    run_time=1.0 / 30.0,
)
assert np.allclose(animate_arc_dot.get_center(), [0.0, -1.0, 0.0], atol=1e-6)

# A bogus anim arg refuses by the exact key, never a silent drop.
animate_bogus_dot = manimlib.Dot()
animate_args_scene.add(animate_bogus_dot)
try:
    animate_args_scene.play(
        animate_bogus_dot.animate(wobble_speed=3).shift(manimlib.RIGHT)
    )
except NotImplementedError as error:
    assert "anim arg `wobble_speed`" in str(error), error
else:
    raise AssertionError("a bogus anim arg was silently dropped")


# --------------------------------------- Restore leftover path_func
# fm-5wq.4.102: the portal arc factories route onto the native restore
# path_arc surface exactly as Transform's do; arbitrary user path
# functions stay the precise unrouted refusal.

restore_pf_scene = Scene()
restore_pf_mover = geometry.Rectangle(width=0.6, height=0.6)
restore_pf_scene.add(restore_pf_mover)
restore_pf_mover.save_state()
restore_pf_mover.shift([1.5, 0.0, 0.0])
restore_pf_anim = manimlib.Restore(
    restore_pf_mover, path_func=path_utils.clockwise_path()
)
# The factory's arc is observable in the native params.
assert np.isclose(restore_pf_anim.path_arc, -math.pi)
assert np.isclose(restore_pf_anim._native_params()["path_arc"], -math.pi)
restore_pf_scene.play(restore_pf_anim, run_time=2.0 / 30.0)
assert np.allclose(restore_pf_mover.get_center(), [0.0, 0.0, 0.0])

# path_along_arc carries its angle and axis the same way.
restore_pf_mover.save_state()
restore_pf_mover.shift([0.0, 1.0, 0.0])
arc_anim = manimlib.Restore(
    restore_pf_mover, path_func=path_utils.path_along_arc(math.pi / 4)
)
assert np.isclose(arc_anim.path_arc, math.pi / 4)

# fm-5wq.4.102 / c6b24744: Restore accepts an authored path function
# through the callback lifecycle; non-callable stays TypeError.
custom_restore = manimlib.Restore(
    restore_pf_mover,
    path_func=lambda start, end, alpha: start,
)
assert callable(custom_restore.path_func)
try:
    manimlib.Restore(
        restore_pf_mover,
        path_func="not_callable",
    )
except TypeError as error:
    assert "path_func" in str(error)
else:
    raise AssertionError("Restore accepted a non-callable path function")


# A never-saved mobject stays the Reference's exact refusal.
try:
    manimlib.Restore(geometry.Rectangle(width=1.0, height=1.0))
except Exception as error:
    assert "Trying to restore without having saved" in str(error)
else:
    raise AssertionError("Restore accepted a never-saved mobject")


# ----------------------------------------- ApplyComplexFunction leftover
# fm-5wq.4.104: plays through the ApplyMethod/Transform native kinds; the
# probe-derived path_arc is the Reference's log(f(1)).imag.

acf_scene = Scene()
acf_square = geometry.Square(side_length=1.0)
acf_square.shift([1.0, 0.5, 0.0])
acf_scene.add(acf_square)
acf_anim = manimlib.ApplyComplexFunction(lambda z: 1j * z, acf_square)
# Rotation by i: path_arc = log(i).imag = pi/2.
assert np.isclose(acf_anim.path_arc, math.pi / 2)
acf_before = acf_square.get_center().copy()
acf_scene.play(acf_anim, run_time=2.0 / 30.0)
# z -> iz rotates the plane a quarter turn: (x, y) lands on (-y, x).
assert np.allclose(
    acf_square.get_center(),
    [-acf_before[1], acf_before[0], 0.0],
    atol=1e-6,
)

# Named refusals: a non-callable, a non-Mobject, and a non-complex map.
try:
    manimlib.ApplyComplexFunction(None, geometry.Square(side_length=1.0))
except TypeError as error:
    assert "callable complex function" in str(error)
else:
    raise AssertionError("ApplyComplexFunction accepted None")

try:
    manimlib.ApplyComplexFunction(lambda z: z, "not a mobject")
except TypeError as error:
    assert "requires a Mobject" in str(error)
else:
    raise AssertionError("ApplyComplexFunction accepted a non-Mobject")

try:
    manimlib.ApplyComplexFunction(
        lambda z: "not complex", geometry.Square(side_length=1.0)
    )
except TypeError as error:
    assert "map complex to complex" in str(error)
else:
    raise AssertionError("ApplyComplexFunction accepted a non-complex map")

# fm-5wq.4.103: ShowCreationThenFadeOut through the native
# show_creation_then_fade_out Succession. The mid-window states are
# observable only mid-segment: Succession.finish lands the active member
# on ITS OWN final_alpha_value (Reference AnimationGroup semantics), so
# ShowCreation completes and FadeOut (final_alpha_value = 0) restores at
# play end. A per-frame updater probe captures the contract states at
# their honest moments: the create half at group alpha 0.25 matches
# pointwise_become_partial(reference, 0.0, 0.5), the fade half at 0.75
# holds stroke alpha 0.5, and the end state is the finished-then-restored
# removed square.
sctfo_scene = InteractiveScene()
sctfo_square = manimlib.Square(side_length=1.0)
sctfo_square.set_stroke(manimlib.WHITE, width=4.0, opacity=1.0)
sctfo_reference = sctfo_square.copy()
sctfo_expected_half = sctfo_square.copy()
sctfo_expected_half.pointwise_become_partial(sctfo_reference, 0.0, 0.5)
sctfo_probe = manimlib.Dot()
sctfo_scene.add(sctfo_square, sctfo_probe)
sctfo_frames = []


def _sctfo_capture(mobject, dt):
    del mobject
    if dt > 0:
        sctfo_frames.append(
            (
                np.array(sctfo_square.get_points(), dtype=float).copy(),
                np.array(
                    sctfo_square.data["stroke_rgba"][:, 3], dtype=float
                ).copy(),
            )
        )


sctfo_probe.add_updater(_sctfo_capture)
sctfo_scene.play(
    indication_module.ShowCreationThenFadeOut(sctfo_square),
    run_time=4.0 / 30.0,
    rate_func=manimlib.linear,
)
sctfo_probe.remove_updater(_sctfo_capture)
assert len(sctfo_frames) == 4, len(sctfo_frames)

# Frame 1 (group alpha 0.25): the create half, frozen mid-window — the
# member's symmetric smooth makes sub-alpha exactly 0.5.
sctfo_quarter_points, sctfo_quarter_alpha = sctfo_frames[0]
assert np.allclose(sctfo_quarter_points, sctfo_expected_half.get_points())
assert np.allclose(sctfo_quarter_alpha, 1.0)

# Frame 3 (group alpha 0.75): creation complete, the fade half at stroke
# alpha exactly 0.5.
sctfo_fade_points, sctfo_fade_alpha = sctfo_frames[2]
assert np.allclose(sctfo_fade_points, sctfo_reference.get_points())
assert np.allclose(sctfo_fade_alpha, 0.5)

# Frame 4 (group alpha 1.0): fully faded before the finish epilogue.
assert np.allclose(sctfo_frames[3][1], 0.0)

# Play end: the remover took the square out, and FadeOut's
# final_alpha_value = 0 restored its records (fading.py:76).
assert sctfo_square not in sctfo_scene.get_mobjects()
assert np.allclose(sctfo_square.get_points(), sctfo_reference.get_points())
assert np.allclose(sctfo_square.data["stroke_rgba"][:, 3], 1.0)

try:
    indication_module.ShowCreationThenFadeOut(None)
except TypeError as error:
    assert "ShowCreationThenFadeOut requires a VMobject" in str(error), error
    assert "NoneType" in str(error), error
else:
    raise AssertionError("ShowCreationThenFadeOut accepted None")

# fm-5wq.4.108: Matrix.element_to_mobject complex entries route through
# DecimalNumber for both degenerate reductions and general complex values.
# Nothing falls through to Tex(str(...))'s "(1+2j)" spelling.
complex_entry_matrix = matrix_module.Matrix([[1.0]])
complex_entry_real = complex_entry_matrix.element_to_mobject(complex(1, 0))
assert isinstance(complex_entry_real, manimlib.DecimalNumber)
assert complex_entry_real.get_value() == complex(1, 0)
assert any(child.has_points() for child in complex_entry_real.submobjects)

complex_entry_imag = complex_entry_matrix.element_to_mobject(complex(0, 2))
assert isinstance(complex_entry_imag, manimlib.DecimalNumber)
assert complex_entry_imag.get_value() == complex(0, 2)
assert any(child.has_points() for child in complex_entry_imag.submobjects)

complex_entry_general = complex_entry_matrix.element_to_mobject(complex(1, 2))
assert isinstance(complex_entry_general, manimlib.DecimalNumber)
assert complex_entry_general.get_value() == complex(1, 2)
assert complex_entry_general.get_tex() == "1.00+2.00i"
assert len(complex_entry_general) == len(complex_entry_general.get_tex())
assert complex_entry_general.family_members_with_points()

# The non-Matrix negative stays the existing named TypeError.
try:
    matrix_module.Matrix.element_to_mobject("not-a-matrix", complex(1, 0))
except TypeError as error:
    assert "requires a Matrix instance" in str(error), error
else:
    raise AssertionError("element_to_mobject accepted a non-Matrix self")

# fm-5wq.4.123: TracingTail's leftover constructor kwargs — the schema's
# TracedPath chain carries time_per_anchor and VMobject style keys. The
# native tail uses that cadence for its temporal grid after the first
# positive update; construction and zero-dt updates publish no geometry.
# Unknown keys stay named TypeErrors.
tail_kw_func = manimlib.TracingTail(
    lambda: (0.0, 0.0, 0.0),
    time_per_anchor=1.0 / 30,
    fill_opacity=0.0,
)
assert tail_kw_func.time_per_anchor == 1.0 / 30

tail_kw_scene = InteractiveScene()
tail_kw_dot = manimlib.Dot()
tail_kw_scene.add(tail_kw_dot)
tail_kw_native = manimlib.TracingTail(tail_kw_dot, fill_opacity=0.0)
assert tail_kw_native.time_per_anchor == 1.0 / 15
assert tail_kw_native.get_num_points() == 0
tail_kw_native_30hz = manimlib.TracingTail(
    tail_kw_dot,
    time_per_anchor=1.0 / 30,
)
assert tail_kw_native_30hz.time_per_anchor == 1.0 / 30
assert tail_kw_native_30hz.get_num_points() == 0
tail_kw_scene.add(tail_kw_native, tail_kw_native_30hz)
tail_kw_scene.update_frame(0)
assert tail_kw_native.get_num_points() == 0
assert tail_kw_native_30hz.get_num_points() == 0
tail_kw_dot.shift(manimlib.RIGHT)
tail_kw_scene.update_frame(1.0 / 15)
# One second contains 15 or 30 complete anchor intervals, with both
# endpoints present. Each smoothed interval contributes two quad points.
assert tail_kw_native.get_num_points() == 31
assert tail_kw_native_30hz.get_num_points() == 61
for updated_native_tail in (tail_kw_native, tail_kw_native_30hz):
    assert np.allclose(updated_native_tail.get_start(), manimlib.ORIGIN)
    assert np.allclose(updated_native_tail.get_end(), manimlib.RIGHT)
    assert updated_native_tail.get_stroke_widths()[0] < updated_native_tail.get_stroke_widths()[-1]
    assert updated_native_tail.get_stroke_opacities()[0] < updated_native_tail.get_stroke_opacities()[-1]

for invalid_time_per_anchor in (0.0, -1.0, float("inf"), float("nan")):
    try:
        manimlib.TracingTail(
            tail_kw_dot,
            time_per_anchor=invalid_time_per_anchor,
        )
    except ValueError as error:
        assert "time_per_anchor must be finite and positive" in str(error), error
    else:
        raise AssertionError(
            "TracingTail accepted invalid time_per_anchor "
            f"{invalid_time_per_anchor!r}"
        )

try:
    manimlib.TracingTail(tail_kw_dot, wobble=3)
except TypeError as error:
    assert "wobble" in str(error), error
else:
    raise AssertionError("TracingTail accepted an unknown kwarg")


# fm-5wq.4.129: the pinned Reference (6199a00) does not define or
# wildcard-export Octahedron. It is a ManimCE polyhedron, so there are no
# constructor kwargs to route on the supported manimlib surface.
assert not hasattr(manimlib, "Octahedron")
octahedron_module = importlib.import_module(
    "manimlib.mobject.three_dimensions"
)
assert not hasattr(octahedron_module, "Octahedron")
try:
    from manimlib import Octahedron as _unexpected_octahedron
except ImportError:
    pass
else:
    raise AssertionError(
        f"unsupported Octahedron leaked into manimlib: {_unexpected_octahedron!r}"
    )


# --------------------------------------- DashedVMobject leftover kwargs
# fm-5wq.4.124: the **kwargs surface is the Reference's — style/config
# keywords flow into VMobject.__init__, and the trailing match_style pass
# deliberately overrides the paint channels with the source's (verbatim
# vectorized_mobject.py ordering). Nothing refuses; this pins the flow.

dvk_source = manimlib.Circle(radius=1.0, stroke_color=manimlib.BLUE)
dvk = vectorized.DashedVMobject(
    dvk_source,
    num_dashes=6,
    stroke_color=manimlib.RED,
    stroke_behind=True,
    flat_stroke=True,
)
assert len(dvk) == 6
# Config kwargs survive; paint kwargs lose to the source, Reference-exact.
assert dvk.stroke_behind is True
assert dvk.flat_stroke is True
assert dvk.get_stroke_color() == manimlib.BLUE

# positive_space_ratio is observable: a fuller ratio draws longer dashes.
dvk_full = vectorized.DashedVMobject(
    dvk_source, num_dashes=6, positive_space_ratio=0.8
)
dvk_sparse = vectorized.DashedVMobject(
    dvk_source, num_dashes=6, positive_space_ratio=0.2
)
assert (
    dvk_full[0].get_arc_length() > 3.0 * dvk_sparse[0].get_arc_length()
)

# The named non-VMobject refusal stays.
try:
    vectorized.DashedVMobject("not a vmobject")
except TypeError as error:
    assert "expects a VMobject" in str(error)
else:
    raise AssertionError("DashedVMobject accepted a non-VMobject")

# fm-5wq.4.127: audit result — the pinned Reference (6199a00) has NO
# Icosahedron: the 663-name surface's only polyhedron is Dodecahedron
# (three_dimensions, VGroup3D). Icosahedron/Tetrahedron/Octahedron are
# ManimCE polyhedra outside the pin, so there is no class, no leftover
# kwargs, and nothing to route; inventing one would break the exact
# wildcard surface. Pin the absence so a stray CE import can't sneak in.
assert not hasattr(manimlib, "Icosahedron")
three_dimensions_module = importlib.import_module(
    "manimlib.mobject.three_dimensions"
)
assert not hasattr(three_dimensions_module, "Icosahedron")
assert hasattr(three_dimensions_module, "Dodecahedron")


# ------------------------------------------------ Tetrahedron kwargs
# fm-5wq.4.128: ecosystem-compat regular tetrahedron over four native
# Polygon faces; kwargs route through the shared VGroup3D split, and
# unknown keys stay its named refusal.

tetra_module = importlib.import_module("manimlib.mobject.three_dimensions")
tetra = tetra_module.Tetrahedron(edge_length=2.0)
assert len(tetra.submobjects) == 4
assert all(face.has_points() for face in tetra.submobjects)
# Every face is equilateral with the requested edge.
tetra_first_points = tetra.submobjects[0].get_start_anchors()
tetra_edge = np.linalg.norm(tetra_first_points[0] - tetra_first_points[1])
assert np.isclose(tetra_edge, 2.0, atol=1e-9)
# Style and 3D-config kwargs route: fill, shading, and depth test.
tetra_styled = tetra_module.Tetrahedron(
    edge_length=1.0,
    fill_color=manimlib.RED,
    fill_opacity=0.5,
    shading=(0.1, 0.2, 0.3),
    depth_test=False,
)
assert np.isclose(tetra_styled.submobjects[0].get_fill_opacity(), 0.5)

# Named refusals: an unknown kwarg, and a non-positive edge.
try:
    tetra_module.Tetrahedron(bogus=True)
except NotImplementedError as error:
    assert "bogus" in str(error)
else:
    raise AssertionError("Tetrahedron silently dropped bogus")

try:
    tetra_module.Tetrahedron(edge_length=0.0)
except ValueError as error:
    assert "positive finite" in str(error)
else:
    raise AssertionError("Tetrahedron accepted a zero edge")

# fm-5wq.4.130: audit result — the pinned Reference (6199a00) has NO
# Arrow3D: three_dimensions' directed-solid surface is Line3D and Cone
# (both Cylinder-based). Arrow3D is ManimCE surface outside the pin, so
# there is no class, no _refuse_unrouted, and no leftover kwargs to
# route; inventing it would widen the exact 663-name wildcard surface.
assert not hasattr(manimlib, "Arrow3D")
arrow3d_probe_module = importlib.import_module(
    "manimlib.mobject.three_dimensions"
)
assert not hasattr(arrow3d_probe_module, "Arrow3D")
assert hasattr(arrow3d_probe_module, "Line3D")
assert hasattr(arrow3d_probe_module, "Cone")

# fm-5wq.4.131: audit result — Annulus has no _refuse_unrouted anywhere:
# its **kwargs already route through the style pass (unknown keys are the
# named TypeError pinned earlier). The remaining unpinned surface was the
# schema's default contract, pinned here: filled at opacity 1 with zero
# stroke width in DEFAULT_LIGHT_COLOR (GREY_B), radii (1, 2), at ORIGIN.
annulus_default = geometry.Annulus()
assert math.isclose(annulus_default.radius, 2.0)
assert len(annulus_default.get_subpaths()) == 2
assert np.allclose(annulus_default.get_center(), [0.0, 0.0, 0.0], atol=1e-6)
assert math.isclose(annulus_default.get_width(), 4.0, rel_tol=0.0, abs_tol=2e-3)
assert annulus_default.get_fill_color() == manimlib.GREY_B
assert np.allclose(annulus_default.data["fill_rgba"][:, 3], 1.0)
assert np.allclose(annulus_default.data["stroke_width"], 0.0)

# Explicit stroke kwargs keep routing through the same style pass.
annulus_stroked = geometry.Annulus(stroke_width=2.0, stroke_color=manimlib.RED)
assert np.allclose(annulus_stroked.data["stroke_width"], 2.0)
assert np.allclose(
    annulus_stroked.data["stroke_rgba"][:, :3],
    manimlib.color_to_rgb(manimlib.RED),
)


# ------------------------------------------------ Sector leftover kwargs
# fm-5wq.4.132: Sector's **kwargs are the Reference's verbatim flow into
# AnnularSector — placement keywords route positionally, style keywords
# through the shared preflight, and unknown keys stay its named refusal.

sector_kw = geometry.Sector(
    angle=math.pi / 2.0,
    radius=2.0,
    start_angle=math.pi / 3.0,
    arc_center=[0.0, 3.0, 0.0],
    fill_color=manimlib.RED,
    fill_opacity=0.5,
)
assert sector_kw.has_points()
assert np.isclose(sector_kw.get_fill_opacity(), 0.5)
# inner_radius is pinned to 0: the wedge tip sits on arc_center.
sector_tip_distance = min(
    np.linalg.norm(point - np.array([0.0, 3.0, 0.0]))
    for point in sector_kw.get_points()
)
assert sector_tip_distance < 1e-9, sector_tip_distance
# arc_center routes: the same sector at the origin is the exact shift.
sector_origin = geometry.Sector(
    angle=math.pi / 2.0, radius=2.0, start_angle=math.pi / 3.0
)
assert np.allclose(
    sector_kw.get_center() - sector_origin.get_center(), [0.0, 3.0, 0.0]
)

# The Reference's own duplicate-keyword crash is kept verbatim: radius is
# Sector's spelling, and outer_radius/inner_radius collide with the
# explicit super() arguments exactly as in the pinned tree.
try:
    geometry.Sector(outer_radius=3.0)
except TypeError:
    pass
else:
    raise AssertionError("Sector accepted outer_radius (Reference collides)")

# Unknown keywords stay the shared preflight's named refusal.
try:
    geometry.Sector(bogus=True)
except TypeError as error:
    assert "bogus" in str(error)
else:
    raise AssertionError("Sector silently dropped bogus")

# fm-5wq.4.133: audit result — Arc has no _refuse_unrouted: every schema
# parameter routes natively and style kwargs ride the shared pass (its
# geometry/style pins live earlier). Pinned here: the schema defaults
# (TAU/4 quarter turn, radius 1, ORIGIN), that n_components= genuinely
# changes the native sampling, the zero-budget refusal, and Arc's own
# unknown-kwarg TypeError.
arc_default = geometry.Arc()
assert np.allclose(arc_default.pfp(0.0), [1.0, 0.0, 0.0], atol=2e-3)
assert np.allclose(arc_default.pfp(1.0), [0.0, 1.0, 0.0], atol=2e-3)
assert math.isclose(
    arc_default.get_arc_length(), math.pi / 2.0, rel_tol=0.0, abs_tol=2e-3
)
assert np.allclose(arc_default.get_arc_center(), [0.0, 0.0, 0.0], atol=1e-6)

arc_coarse = geometry.Arc(angle=math.tau / 4, n_components=4)
arc_fine = geometry.Arc(angle=math.tau / 4, n_components=12)
assert arc_coarse.get_num_points() < arc_fine.get_num_points()
assert math.isclose(
    arc_fine.get_arc_length(), math.pi / 2.0, rel_tol=0.0, abs_tol=2e-3
)

try:
    geometry.Arc(n_components=0)
except ValueError as error:
    assert "component" in str(error).lower(), error
else:
    raise AssertionError("Arc accepted a zero component budget")

try:
    geometry.Arc(wobble_amount=1)
except TypeError as error:
    assert "wobble_amount" in str(error), error
else:
    raise AssertionError("Arc silently ignored an unknown keyword")

# fm-5wq.4.135: audit result — Ellipse has no _refuse_unrouted: width/
# height route natively, the Circle-chain arc_center/start_angle kwargs
# are consumed explicitly, and style kwargs ride the shared pass (the
# parameterized construction is pinned earlier). Pinned here: the schema
# defaults, and that every unrecognized key — including the chain's
# n_components, which the native ellipse builder deliberately owns —
# stays a TypeError naming the key, never a silent drop.
ellipse_default = geometry.Ellipse()
assert np.allclose(
    [ellipse_default.get_width(), ellipse_default.get_height()],
    [2.0, 1.0],
    atol=1e-6,
)
assert np.allclose(ellipse_default.get_center(), [0.0, 0.0, 0.0], atol=1e-6)
assert np.allclose(ellipse_default.get_points()[0], [1.0, 0.0, 0.0], atol=1e-6)
assert ellipse_default.has_points()

try:
    geometry.Ellipse(n_components=4)
except TypeError as error:
    assert "n_components" in str(error), error
else:
    raise AssertionError("Ellipse silently dropped n_components")

try:
    geometry.Ellipse(squash_factor=2)
except TypeError as error:
    assert "squash_factor" in str(error), error
else:
    raise AssertionError("Ellipse silently ignored an unknown keyword")


# ------------------------------------- RoundedRectangle leftover kwargs
# fm-5wq.4.136: the **kwargs surface is the Reference's verbatim flow into
# the shared style preflight; the Reference docstring's own example
# spelling works, and unknown keys stay the named refusal.

rr_kw = geometry.RoundedRectangle(
    width=3.0, height=4.0, corner_radius=1.0, color=manimlib.BLUE
)
assert np.allclose([rr_kw.get_width(), rr_kw.get_height()], [3.0, 4.0])
# The color shorthand reaches both paint channels through the preflight.
assert rr_kw.get_stroke_color() == manimlib.BLUE
rr_filled = geometry.RoundedRectangle(
    width=2.0,
    height=1.0,
    corner_radius=0.25,
    fill_color=manimlib.RED,
    fill_opacity=0.5,
    stroke_width=6.0,
)
assert np.isclose(rr_filled.get_fill_opacity(), 0.5)
assert np.isclose(rr_filled.get_stroke_width(), 6.0)

# Unknown keywords stay the shared preflight's named refusal.
try:
    geometry.RoundedRectangle(bogus=True)
except TypeError as error:
    assert "bogus" in str(error)
else:
    raise AssertionError("RoundedRectangle silently dropped bogus")

# fm-5wq.4.138: audit result — Dot has no _refuse_unrouted: point/radius
# route natively and the four public style values re-route through the
# ordinary style path with **kwargs, whose preflight names unknown keys.
# Pinned here: the schema default contract (radius 0.08, black zero-width
# stroke, white fill at opacity 1, ORIGIN), positional point routing, and
# Dot's own unknown-kwarg TypeError.
dot_default = geometry.Dot()
assert math.isclose(
    dot_default.get_width(), 0.16, rel_tol=0.0, abs_tol=2e-3
)
assert np.allclose(dot_default.get_center(), [0.0, 0.0, 0.0], atol=1e-6)
assert np.allclose(dot_default.data["fill_rgba"][:, :3], 1.0)
assert np.allclose(dot_default.data["fill_rgba"][:, 3], 1.0)
assert np.allclose(dot_default.data["stroke_width"], 0.0)
assert np.allclose(dot_default.data["stroke_rgba"][:, :3], 0.0)

dot_positional = geometry.Dot([1.0, -0.5, 0.0], radius=0.2)
assert np.allclose(dot_positional.get_center(), [1.0, -0.5, 0.0], atol=1e-6)
assert math.isclose(
    dot_positional.get_width(), 0.4, rel_tol=0.0, abs_tol=2e-3
)

try:
    geometry.Dot(glow_factor=2)
except TypeError as error:
    assert "glow_factor" in str(error), error
else:
    raise AssertionError("Dot silently ignored an unknown keyword")


# ------------------------------------------------ Line leftover kwargs
# fm-5wq.4.139: the constructor surface is the Reference's verbatim —
# buff and path_arc route natively, Mobject endpoints resolve through
# pointify onto boundaries, style kwargs ride the shared preflight, and
# unknown keys stay its named refusal.

line_kw_buff = geometry.Line(manimlib.LEFT, manimlib.RIGHT, buff=0.25)
assert np.isclose(line_kw_buff.get_length(), 1.5)
line_kw_arc = geometry.Line(
    manimlib.LEFT, manimlib.RIGHT, path_arc=math.pi / 2.0
)
assert line_kw_arc.get_arc_length() > 2.05  # bowed past the straight chord
line_kw_color = geometry.Line(
    [1.0, 2.0, 0.0], [-2.0, -3.0, 0.0], color=manimlib.BLUE
)
assert line_kw_color.get_stroke_color() == manimlib.BLUE

# Mobject endpoints: the line meets each circle at its boundary, not its
# centre — pointify hands back the facing boundary point.
line_kw_a = geometry.Circle(radius=0.5)
line_kw_a.move_to([-2.0, 0.0, 0.0])
line_kw_b = geometry.Circle(radius=0.5)
line_kw_b.move_to([2.0, 0.0, 0.0])
line_kw_between = geometry.Line(line_kw_a, line_kw_b)
assert np.isclose(
    np.linalg.norm(line_kw_between.get_start() - line_kw_a.get_center()),
    0.5,
    atol=1e-6,
)
assert np.isclose(
    np.linalg.norm(line_kw_between.get_end() - line_kw_b.get_center()),
    0.5,
    atol=1e-6,
)

# Unknown keywords stay the shared preflight's named refusal.
try:
    geometry.Line(manimlib.LEFT, manimlib.RIGHT, bogus=True)
except TypeError as error:
    assert "bogus" in str(error)
else:
    raise AssertionError("Line silently dropped bogus")

# fm-5wq.4: Camera.reset_pixel_shape routes the native pixel-shape setter.
# default_pixel_shape stays the constructor resolution while the live shape
# changes and the frame aspect follows; Scene.on_resize drives it.
assert str(inspect.signature(camera_module.Camera.reset_pixel_shape)) == (
    "(self, new_width=None, new_height=None)"
)
resize_camera = camera_module.Camera(resolution=(1920, 1080))
assert resize_camera.default_pixel_shape == (1920, 1080)
assert resize_camera.reset_pixel_shape(640, 480) is None
assert resize_camera.get_pixel_width() == 640
assert resize_camera.get_pixel_height() == 480
assert resize_camera.get_pixel_shape() == (640, 480)
assert resize_camera.default_pixel_shape == (1920, 1080)
assert np.isclose(
    resize_camera.get_frame_width() / resize_camera.get_frame_height(),
    640.0 / 480.0,
)
# An omitted dimension keeps the live value for that axis.
assert resize_camera.reset_pixel_shape(new_height=360) is None
assert resize_camera.get_pixel_shape() == (640, 360)

resize_scene = Scene()
assert resize_scene.on_resize(800, 600) is None
assert resize_scene.camera.get_pixel_width() == 800
assert resize_scene.camera.get_pixel_height() == 600

# Planted negative: zero or negative pixel dimensions are a named refusal,
# never a silent clamp.
for bad_width, bad_height in ((0, 480), (-640, 480)):
    try:
        resize_camera.reset_pixel_shape(bad_width, bad_height)
    except ValueError as error:
        assert "positive pixel" in str(error)
    else:
        raise AssertionError("reset_pixel_shape accepted a non-positive shape")
assert resize_camera.get_pixel_shape() == (640, 360)

# fm-5wq.4: get_pixel_size, resize_frame_shape, and refresh_uniforms pin the
# rest of the live pixel-shape surface those resets feed.
assert str(inspect.signature(camera_module.Camera.get_pixel_size)) == "(self)"
assert str(inspect.signature(camera_module.Camera.resize_frame_shape)) == (
    "(self, fixed_dimension=False)"
)
assert str(inspect.signature(camera_module.Camera.refresh_uniforms)) == "(self)"

# get_pixel_size is frame_width / pixel_width, so shrinking the pixel width
# while the frame width holds must grow the pixel size by the same factor.
pixel_size_camera = camera_module.Camera(resolution=(1920, 1080))
initial_pixel_size = pixel_size_camera.get_pixel_size()
assert np.isclose(
    initial_pixel_size,
    pixel_size_camera.get_frame_width() / 1920.0,
)
pixel_size_camera.reset_pixel_shape(640, 480)
resized_pixel_size = pixel_size_camera.get_pixel_size()
assert np.isclose(
    resized_pixel_size,
    pixel_size_camera.get_frame_width() / 640.0,
)
assert np.isclose(resized_pixel_size, 3.0 * initial_pixel_size)
assert not np.isclose(resized_pixel_size, initial_pixel_size)

# resize_frame_shape restores the live pixel aspect after the frame shape is
# distorted: the default keeps width and moves height, fixed_dimension=True
# keeps height and moves width.
frame_shape_camera = camera_module.Camera(resolution=(1600, 900))
frame_shape_camera.frame._core.set_shape((10.0, 10.0))
assert frame_shape_camera.resize_frame_shape() is None
assert np.allclose(
    frame_shape_camera.get_frame_shape(), (10.0, 10.0 * 900.0 / 1600.0)
)
frame_shape_camera.frame._core.set_shape((10.0, 10.0))
assert frame_shape_camera.resize_frame_shape(fixed_dimension=True) is None
assert np.allclose(
    frame_shape_camera.get_frame_shape(), (10.0 * 1600.0 / 900.0, 10.0)
)

# refresh_uniforms after a frame move: every Reference uniform key is present
# with finite values, and camera_position tracks the moved frame.
uniforms_camera = camera_module.Camera(resolution=(1280, 720))
uniforms_camera.frame.shift([1.0, -2.0, 0.0])
assert uniforms_camera.refresh_uniforms() is None
for uniform_key in (
    "view",
    "frame_scale",
    "pixel_size",
    "camera_position",
    "light_position",
):
    assert uniform_key in uniforms_camera.uniforms
    assert np.all(
        np.isfinite(
            np.asarray(uniforms_camera.uniforms[uniform_key], dtype=float)
        )
    )
assert np.allclose(
    uniforms_camera.uniforms["camera_position"][:2], (1.0, -2.0)
)
assert np.isclose(
    uniforms_camera.uniforms["pixel_size"], uniforms_camera.get_pixel_size()
)

# fm-5wq.4: Camera.convert_pixel_array matches the Reference contract —
# uint8 data round-trips unchanged, and convert_from_floats scales [0, 1]
# floats by rgb_max_val and rounds into the pixel dtype.
assert str(inspect.signature(camera_module.Camera.convert_pixel_array)) == (
    "(self, pixel_array, convert_from_floats=False)"
)
convert_camera = camera_module.Camera(resolution=(640, 360))
assert convert_camera.rgb_max_val == 255.0
assert convert_camera.pixel_array_dtype is np.uint8
convert_uint8 = np.array([[0, 51, 255], [7, 128, 200]], dtype=np.uint8)
converted_uint8 = convert_camera.convert_pixel_array(convert_uint8)
assert converted_uint8.dtype == np.uint8
assert np.array_equal(converted_uint8, convert_uint8)
convert_floats = np.array([[0.0, 0.2, 1.0]])
converted_floats = convert_camera.convert_pixel_array(
    convert_floats, convert_from_floats=True
)
assert converted_floats.dtype == np.uint8
assert np.array_equal(converted_floats, np.array([[0, 51, 255]], dtype=np.uint8))
# Plain lists convert too; the untouched-float path truncates via astype.
assert np.array_equal(
    convert_camera.convert_pixel_array([[0.0, 0.9, 2.0]]),
    np.array([[0, 0, 2]], dtype=np.uint8),
)

# Planted negative: a non-array-like payload is a TypeError, never a hang.
try:
    convert_camera.convert_pixel_array(object(), convert_from_floats=True)
except TypeError:
    pass
else:
    raise AssertionError("convert_pixel_array accepted a non-array payload")

# fm-5wq.4: Camera.pixel_coords_to_space_coords maps pixel-space points into
# the frame from the live pixel shape and frame state alone. Absolute
# mapping is the Reference's height-axis-only scale from the frame center:
# center + (frame_height / pixel_height) * [px - pw/2, py - ph/2, 0];
# relative=True is the Reference's 2 * [px/pw, py/ph, 0] offset.
assert str(
    inspect.signature(camera_module.Camera.pixel_coords_to_space_coords)
) == "(self, px, py, relative=False)"
coords_camera = camera_module.Camera(resolution=(1920, 1080))
assert np.allclose(
    coords_camera.pixel_coords_to_space_coords(960, 540),
    coords_camera.get_frame_center(),
    atol=1e-9,
)
coords_scale = coords_camera.get_frame_height() / 1080.0
assert np.allclose(
    coords_camera.pixel_coords_to_space_coords(0, 0),
    coords_camera.get_frame_center()
    + coords_scale * np.array([-960.0, -540.0, 0.0]),
)
assert np.allclose(
    coords_camera.pixel_coords_to_space_coords(0, 0, relative=True),
    (0.0, 0.0, 0.0),
)
assert np.allclose(
    coords_camera.pixel_coords_to_space_coords(1920, 1080, relative=True),
    (2.0, 2.0, 0.0),
)
assert np.allclose(
    coords_camera.pixel_coords_to_space_coords(480, 810, relative=True),
    (0.5, 1.5, 0.0),
)
# The mapping follows the live frame center.
coords_camera.frame.shift([1.0, -2.0, 0.0])
assert np.allclose(
    coords_camera.pixel_coords_to_space_coords(960, 540),
    (1.0, -2.0, 0.0),
)

# Planted negative: non-finite pixel coordinates are a named refusal, never
# a NaN point.
for bad_px, bad_py in (
    (float("nan"), 0.0),
    (0.0, float("inf")),
    (float("-inf"), 0.0),
):
    try:
        coords_camera.pixel_coords_to_space_coords(bad_px, bad_py)
    except ValueError as error:
        assert "finite pixel" in str(error)
    else:
        raise AssertionError(
            "pixel_coords_to_space_coords accepted a non-finite coordinate"
        )
try:
    coords_camera.pixel_coords_to_space_coords(object(), 0.0)
except TypeError:
    pass
else:
    raise AssertionError(
        "pixel_coords_to_space_coords accepted a non-numeric coordinate"
    )

# fm-5wq.4: the mapping follows the LIVE pixel shape after
# reset_pixel_shape, never the constructor resolution — the scale divides
# by the live pixel height (480) and relative offsets normalize by the
# live (640, 480) shape.
live_coords_camera = camera_module.Camera(resolution=(1920, 1080))
live_coords_camera.reset_pixel_shape(640, 480)
assert live_coords_camera.get_pixel_shape() == (640, 480)
assert np.allclose(
    live_coords_camera.pixel_coords_to_space_coords(320, 240),
    live_coords_camera.get_frame_center(),
    atol=1e-9,
)
live_coords_scale = live_coords_camera.get_frame_height() / 480.0
assert np.allclose(
    live_coords_camera.pixel_coords_to_space_coords(0, 0),
    live_coords_camera.get_frame_center()
    + live_coords_scale * np.array([-320.0, -240.0, 0.0]),
)
# A /1080 constructor-resolution scale would disagree with the live one;
# make the distinction observable rather than trusting the formula pin.
stale_coords_scale = live_coords_camera.get_frame_height() / 1080.0
assert not np.allclose(
    live_coords_camera.pixel_coords_to_space_coords(0, 0),
    live_coords_camera.get_frame_center()
    + stale_coords_scale * np.array([-320.0, -240.0, 0.0]),
)
assert np.allclose(
    live_coords_camera.pixel_coords_to_space_coords(640, 480, relative=True),
    (2.0, 2.0, 0.0),
)
assert np.allclose(
    live_coords_camera.pixel_coords_to_space_coords(160, 360, relative=True),
    (0.5, 1.5, 0.0),
)

# fm-5wq.4: get_aspect_ratio follows the LIVE pixel shape — 16:9 at
# construction, 4:3 after reset_pixel_shape(640, 480) — and
# resize_frame_shape keeps the frame aspect matching that live ratio.
aspect_camera = camera_module.Camera(resolution=(1920, 1080))
assert np.isclose(aspect_camera.get_aspect_ratio(), 1920.0 / 1080.0)
aspect_camera.reset_pixel_shape(640, 480)
assert np.isclose(aspect_camera.get_aspect_ratio(), 4.0 / 3.0)
assert not np.isclose(aspect_camera.get_aspect_ratio(), 1920.0 / 1080.0)
assert np.isclose(
    aspect_camera.get_frame_width() / aspect_camera.get_frame_height(),
    4.0 / 3.0,
)
# Distort the frame, then resize_frame_shape restores the live 4:3 aspect.
aspect_camera.frame._core.set_shape((10.0, 10.0))
assert aspect_camera.resize_frame_shape() is None
assert np.isclose(aspect_camera.get_frame_width(), 10.0)
assert np.isclose(aspect_camera.get_frame_height(), 7.5)
aspect_camera.frame._core.set_shape((10.0, 10.0))
assert aspect_camera.resize_frame_shape(fixed_dimension=True) is None
assert np.isclose(aspect_camera.get_frame_height(), 10.0)
assert np.isclose(aspect_camera.get_frame_width(), 10.0 * 4.0 / 3.0)

# fm-5wq.4: Scene.camera_config is the remaining constructor seam that
# builds Lumen's Camera — manim_config.camera (default_config.yml plus the
# cwd custom_config.yml; fm-5wq.44), class default_camera_config,
# constructor camera_config, and Scene.samples (ThreeDScene defaults to 4).
assert scene_module.Scene.default_camera_config == {}
assert scene_module.Scene.samples == 0
assert scene_module.ThreeDScene.samples == 4
assert Scene().camera_config == {
    "resolution": (1920, 1080), "fps": 30,
    "background_color": "#333333", "background_opacity": 1.0,
}, Scene().camera_config
assert Scene().camera.samples == 0
config_scene = Scene(camera_config=dict(resolution=(640, 360), fps=24))
assert config_scene.camera_config["resolution"] == (640, 360)
assert config_scene.camera.get_pixel_shape() == (640, 360)
assert config_scene.camera.fps == 24
assert config_scene.camera.frame is config_scene.frame
assert config_scene.camera is config_scene.camera


class _ConfiguredCameraScene(Scene):
    default_camera_config = dict(resolution=(320, 180), samples=2)


configured_camera_scene = _ConfiguredCameraScene()
assert configured_camera_scene.camera.get_pixel_shape() == (320, 180)
assert configured_camera_scene.camera.samples == 2
# Constructor camera_config overlays the class default per-key.
overlay_camera_scene = _ConfiguredCameraScene(camera_config=dict(fps=12))
assert overlay_camera_scene.camera.get_pixel_shape() == (320, 180)
assert overlay_camera_scene.camera.fps == 12
assert overlay_camera_scene.camera.samples == 2

three_d_config_scene = scene_module.ThreeDScene(
    camera_config=dict(resolution=(800, 400))
)
assert three_d_config_scene.camera.samples == 4
assert three_d_config_scene.camera.get_pixel_shape() == (800, 400)

try:
    Scene(camera_config="nope")
except TypeError as error:
    assert str(error) == "Scene camera_config must be a dict"
else:
    raise AssertionError("Scene accepted a non-dict camera_config")

# Unrouted Camera constructor kwargs still name the seam on first access.
refused_image_scene = Scene(camera_config=dict(background_image="image.png"))
try:
    refused_image_scene.camera
except NotImplementedError as error:
    assert str(error) == (
        "Camera() keyword(s) not yet routed to the native builder: "
        "background_image"
    )
else:
    raise AssertionError(
        "Scene camera_config silently accepted background_image"
    )
assert config_scene.camera.get_pixel_width() == 640
# A refused window= also defers to first .camera access — Scene() itself
# stays lazy and never constructs the Camera.
refused_window_scene = Scene(camera_config=dict(window=object()))
try:
    refused_window_scene.camera
except NotImplementedError as error:
    assert str(error) == (
        "Camera() keyword(s) not yet routed to the native builder: window"
    )
else:
    raise AssertionError("Scene camera_config silently accepted window")
try:
    Scene(camera_config=1)
except TypeError as error:
    assert str(error) == "Scene camera_config must be a dict"
else:
    raise AssertionError("Scene accepted an integer camera_config")

# fm-5wq.4: get_pixel_size reaches through Scene.camera_config — a
# configured resolution divides the live frame width by ITS pixel width,
# and the default Scene keeps Camera's 1920-wide default.
pixel_size_config_scene = Scene(camera_config=dict(resolution=(640, 360)))
assert pixel_size_config_scene.camera.get_pixel_shape() == (640, 360)
assert np.isclose(
    pixel_size_config_scene.camera.get_pixel_size(),
    pixel_size_config_scene.camera.get_frame_width() / 640.0,
)
pixel_size_default_scene = Scene()
assert pixel_size_default_scene.camera.get_pixel_width() == 1920
assert np.isclose(
    pixel_size_default_scene.camera.get_pixel_size(),
    pixel_size_default_scene.camera.get_frame_width() / 1920.0,
)
# Same frame width (both 16:9), a third of the pixels — three times the
# pixel size; the two scenes disagree, so the config observably matters.
assert np.isclose(
    pixel_size_config_scene.camera.get_pixel_size(),
    3.0 * pixel_size_default_scene.camera.get_pixel_size(),
)
assert not np.isclose(
    pixel_size_config_scene.camera.get_pixel_size(),
    pixel_size_default_scene.camera.get_pixel_size(),
)

# fm-5wq.4: refresh_uniforms reads get_pixel_size live at call time — the
# camera_config resolution lands in uniforms['pixel_size'], the dict is a
# snapshot that goes stale across reset_pixel_shape (Reference contract),
# and the next refresh_uniforms picks up the live shape.
uniform_size_camera = pixel_size_config_scene.camera
assert uniform_size_camera.refresh_uniforms() is None
assert np.isclose(
    uniform_size_camera.uniforms["pixel_size"],
    uniform_size_camera.get_frame_width() / 640.0,
)
size_before_reset = uniform_size_camera.uniforms["pixel_size"]
uniform_size_camera.reset_pixel_shape(320, 180)
assert np.isclose(
    uniform_size_camera.uniforms["pixel_size"], size_before_reset
)
assert not np.isclose(
    uniform_size_camera.uniforms["pixel_size"],
    uniform_size_camera.get_pixel_size(),
)
assert uniform_size_camera.refresh_uniforms() is None
assert np.isclose(
    uniform_size_camera.uniforms["pixel_size"],
    uniform_size_camera.get_pixel_size(),
)
assert np.isclose(
    uniform_size_camera.uniforms["pixel_size"],
    uniform_size_camera.get_frame_width() / 320.0,
)

# fm-5wq.4: Scene.file_writer_config is the remaining constructor seam
# that builds SceneFileWriter — class default_file_writer_config, then
# constructor file_writer_config. Construction records knobs; movie
# encode stays the ffmpeg Reel boundary.
assert scene_module.Scene.default_file_writer_config == {}
writer_scene = Scene()
assert writer_scene.file_writer_config == {}
assert isinstance(writer_scene.file_writer, writer_mod.SceneFileWriter)
assert writer_scene.file_writer.scene is writer_scene
assert writer_scene.file_writer.get_output_file_name() == "Scene"
assert writer_scene.file_writer.get_image_file_path() == "Scene.png"
assert writer_scene.file_writer.movie_file_extension == ".mp4"
assert writer_scene.file_writer.has_progress_display() is False

named_writer_scene = Scene(
    file_writer_config=dict(
        file_name="clip",
        output_directory="/tmp/out",
        write_to_movie=True,
        movie_file_extension=".mov",
    )
)
assert named_writer_scene.file_writer_config["file_name"] == "clip"
assert named_writer_scene.file_writer.scene is named_writer_scene
assert named_writer_scene.file_writer.get_movie_file_path() == (
    "/tmp/out/clip.mov"
)
assert named_writer_scene.file_writer.has_progress_display() is True


class _ConfiguredWriterScene(Scene):
    default_file_writer_config = dict(
        file_name="base",
        output_directory="/tmp/out",
    )


overlay_writer_scene = _ConfiguredWriterScene(
    file_writer_config=dict(movie_file_extension=".mov")
)
assert overlay_writer_scene.file_writer.get_movie_file_path() == (
    "/tmp/out/base.mov"
)

try:
    Scene(file_writer_config="nope")
except TypeError as error:
    assert str(error) == "Scene file_writer_config must be a dict"
else:
    raise AssertionError("Scene accepted a non-dict file_writer_config")

try:
    Scene(file_writer_config=dict(bogus=True))
except TypeError as error:
    assert str(error) == "unexpected keyword arguments: bogus"
else:
    raise AssertionError("Scene silently dropped file_writer_config.bogus")

# fm-5wq.4: InteractiveScene remaining constructor knobs overlay the class
# defaults so get_crosshair / get_selection_rectangle observe the instance
# values. Scene kwargs still reach the ancestor (skip_animations).
assert np.isclose(InteractiveScene.crosshair_width, 0.2)
styled_interactive = InteractiveScene(
    skip_animations=True,
    crosshair_width=0.5,
    selection_nudge_size=0.2,
    selection_rectangle_stroke_color=manimlib.BLUE,
    selection_rectangle_stroke_width=3.0,
    select_top_level_mobs=False,
    crosshair_style=dict(stroke_color=manimlib.RED, stroke_width=2.0),
)
assert styled_interactive.skip_animations is True
assert np.isclose(styled_interactive.crosshair_width, 0.5)
assert np.isclose(styled_interactive.selection_nudge_size, 0.2)
assert styled_interactive.selection_rectangle_stroke_color == manimlib.BLUE
assert np.isclose(styled_interactive.selection_rectangle_stroke_width, 3.0)
assert styled_interactive.select_top_level_mobs is False
styled_crosshair = styled_interactive.get_crosshair()
assert np.isclose(styled_crosshair.get_width(), 0.5, atol=1e-6)
assert styled_crosshair.get_stroke_color() == manimlib.RED
styled_rect = styled_interactive.get_selection_rectangle()
assert styled_rect.get_stroke_color() == manimlib.BLUE
assert np.isclose(styled_rect.get_stroke_width(), 3.0)
try:
    InteractiveScene(crosshair_style="nope")
except TypeError as error:
    assert str(error) == "InteractiveScene crosshair_style must be a dict"
else:
    raise AssertionError("InteractiveScene accepted a non-dict crosshair_style")

# fm-5wq.4: the three overlay dicts follow the same pattern — copy the
# class default when omitted (never aliasing it), store a copy of an
# explicit dict, and name the key on a non-dict.
default_config_interactive = InteractiveScene()
assert (
    default_config_interactive.corner_dot_config
    == InteractiveScene.corner_dot_config
)
assert (
    default_config_interactive.corner_dot_config
    is not InteractiveScene.corner_dot_config
)
assert (
    default_config_interactive.cursor_location_config
    == InteractiveScene.cursor_location_config
)
assert (
    default_config_interactive.time_label_config
    == InteractiveScene.time_label_config
)
configured_interactive = InteractiveScene(
    corner_dot_config=dict(color=manimlib.RED, radius=0.1, glow_factor=1.0),
    cursor_location_config=dict(
        font_size=30, fill_color=manimlib.BLUE, num_decimal_places=2
    ),
    time_label_config=dict(
        font_size=18, fill_color=manimlib.RED, num_decimal_places=0
    ),
)
assert configured_interactive.corner_dot_config == dict(
    color=manimlib.RED, radius=0.1, glow_factor=1.0
)
assert configured_interactive.cursor_location_config == dict(
    font_size=30, fill_color=manimlib.BLUE, num_decimal_places=2
)
assert configured_interactive.time_label_config == dict(
    font_size=18, fill_color=manimlib.RED, num_decimal_places=0
)
for overlay_config_name in (
    "corner_dot_config",
    "cursor_location_config",
    "time_label_config",
):
    try:
        InteractiveScene(**{overlay_config_name: "nope"})
    except TypeError as error:
        assert str(error) == (
            "InteractiveScene " + overlay_config_name + " must be a dict"
        )
    else:
        raise AssertionError(
            "InteractiveScene accepted a non-dict " + overlay_config_name
        )


# ---------------------------------------------------------------------------
# fm-5wq.4.141: Mobject parent back-edges, the updater family, animating
# status, and the data-changed notification surface.

def _chain():
    top = Mobject()
    mid = Mobject()
    leaf = Mobject()
    top.add(mid)
    mid.add(leaf)
    return top, mid, leaf


# add/remove maintain the Reference's parent back-edges through the live list.
gp, pa, ch = _chain()
assert ch.parents == [pa] and pa.parents == [gp] and gp.parents == []
other = Mobject()
pa.add(other)
assert other.parents == [pa]
pa.remove(other)
assert other.parents == []

# set_submobjects reconciles added and dropped children in one commit.
left = Mobject()
right = Mobject()
holder = Mobject()
first = Mobject()
second = Mobject()
holder.set_submobjects([first, second])
assert first.parents == [holder] and second.parents == [holder]
replacement = Mobject()
holder.set_submobjects([first, replacement])
assert replacement.parents == [holder]
assert second.parents == []
assert holder.submobjects == [first, replacement]

# A shared descendant legally records every parent; detaching from one keeps
# the other edge intact.
shared = Mobject()
family_a = Mobject().add(shared)
family_b = Mobject().add(shared)
assert shared.parents == [family_a, family_b]
family_a.remove(shared)
assert shared.parents == [family_b]

# get_ancestors returns highest ancestors first, deduped.
top, mid, leaf = _chain()
assert leaf.get_ancestors() == [top, mid]
assert mid.get_ancestors() == [top]
assert top.get_ancestors() == []
outside = Mobject().add(leaf)
assert top not in outside.get_family(recurse=False)
extended = leaf.get_ancestors(extended=True)
# Reference extended traversal starts from the downward family only, so
# every ancestor qualifies: highest first after the reverse.
assert extended == [top, outside, mid]

# Copies start with rebuilt back-edges: root parentless, members mapped.
copied_top = top.copy()
assert copied_top.parents == []
copied_mid = copied_top.submobjects[0]
copied_leaf = copied_mid.submobjects[0]
assert copied_leaf is not leaf
assert copied_leaf.parents == [copied_mid]
assert copied_mid.parents == [copied_top]

# Pickle round-trips rebuild parents from the children lists alone.
revived = pickle.loads(pickle.dumps(top))  # ubs:ignore -- round-trip of this test's own trusted object graph
assert revived.parents == []
revived_mid = revived.submobjects[0]
revived_leaf = revived_mid.submobjects[0]
assert revived_leaf.parents == [revived_mid]
assert revived_mid.parents == [revived]

# Updater family: registration shape, ordering, and validation.
tracked = Mobject()
assert tracked.get_updaters() == [] and not tracked.has_updaters()


def _noop_updater(mob):
    return None


tracked.add_updater(_noop_updater)
assert tracked.get_updaters() == [_noop_updater]
assert tracked.has_updaters()
earlier = Mobject()
tracked.insert_updater(lambda mob: None, 0)
try:
    tracked.insert_updater("not callable")
except TypeError as error:
    assert "callable updater" in str(error)
else:
    raise AssertionError("insert_updater accepted a non-callable")
assert tracked.get_updaters()[1] is _noop_updater

# has_updaters observes the whole live family, including deep descendants.
deep_parent = Mobject()
deep_child = Mobject()
deep_parent.add(deep_child)
assert not deep_parent.has_updaters()
deep_child.add_updater(lambda mob: None)
assert deep_parent.has_updaters()

# match_updaters copies the source list without aliasing it.
source = Mobject()
source.add_updater(_noop_updater)
target = Mobject()
target.match_updaters(source)
assert target.get_updaters() == [_noop_updater]
assert target.get_updaters() is not source.get_updaters()
try:
    target.match_updaters(42)
except TypeError:
    pass
else:
    raise AssertionError("match_updaters accepted a non-Mobject")

# Animating status covers the requested family and every ancestor.
still = Mobject()
animating_root = Mobject()
animating_mid = Mobject()
animating_leaf = Mobject()
animating_root.add(animating_mid)
animating_mid.add(animating_leaf)
assert not animating_leaf.is_changing()
animating_leaf.set_animating_status(True)
assert animating_leaf.is_changing()
assert animating_mid._is_animating and animating_root._is_animating
assert not getattr(still, "_is_animating", False)
animating_root.set_animating_status(False, recurse=False)
assert animating_root._is_animating is False
assert animating_mid._is_animating and animating_leaf._is_animating

# An updater alone marks a mobject changing without an animation flag.
updater_only = Mobject()
flagged_child = Mobject()
updater_only.add(flagged_child)
flagged_child.add_updater(lambda mob: None)
assert flagged_child.is_changing() and updater_only.is_changing()

# note_changed_data propagates upward along real parent edges.
data_top = Mobject()
data_bottom = Mobject()
data_top.add(data_bottom)
data_bottom.note_changed_data()
assert data_bottom._data_has_changed and data_top._data_has_changed
data_side = Mobject()
data_top.add(data_side)
data_top.note_changed_data(recurse_up=False)
assert data_top._data_has_changed
assert not hasattr(data_side, "_data_has_changed")

# The affects_data decorator marks after the wrapped call succeeds.
@Mobject.affects_data
def _restyle(mob):
    mob.restyle_calls = getattr(mob, "restyle_calls", 0) + 1
    return "styled"


decorated_target = Mobject()
assert _restyle(decorated_target) == "styled"
assert decorated_target.restyle_calls == 1
assert decorated_target._data_has_changed


# ---------------------------------------------------------------------------
# fm-5wq.4: Mobject event-listener family over the process-wide dispatcher.
event_type_module = importlib.import_module("manimlib.event_handler")
EventTypeSurface = importlib.import_module(
    "manimlib.event_handler.event_type"
).EventType
dispatcher = event_type_module.EVENT_DISPATCHER

event_host = Mobject()
event_child = manimlib.Square().move_to([1.0, 2.0, 0.0])
event_host.add(event_child)
assert event_host.get_event_listners() == []
assert not event_host.get_has_event_listner()

motion_hits = []


def _motion_recorder(mob, event_data):
    motion_hits.append(event_data["point"])


event_host.add_mouse_motion_listner(_motion_recorder)
press_hits = []
event_child.add_mouse_press_listner(lambda mob, event_data: press_hits.append(1))
assert len(event_host.get_event_listners()) == 1
assert event_host.get_has_event_listner()
assert event_host.get_family_event_listners() and len(
    event_host.get_family_event_listners()
) == 2

# The dispatcher bucket holds the registration and dispatch reaches it.
assert dispatcher.event_listners[EventTypeSurface.MouseMotionEvent]
dispatcher.dispatch(
    EventTypeSurface.MouseMotionEvent, point=[1.0, 2.0, 0.0]
)
assert motion_hits == [[1.0, 2.0, 0.0]]

# Removal through a fresh equal EventListener clears both the mobject list
# and the dispatcher bucket (Reference event_listner.py:23 equality).
event_host.remove_mouse_motion_listner(_motion_recorder)
assert event_host.get_event_listners() == []
assert not dispatcher.event_listners[EventTypeSurface.MouseMotionEvent]
dispatcher.dispatch(
    EventTypeSurface.MouseMotionEvent, point=[3.0, 4.0, 0.0]
)
assert motion_hits == [[1.0, 2.0, 0.0]]

# clear_event_listners resets the whole subtree.
assert event_child.get_has_event_listner()
event_host.clear_event_listners()
assert event_child.get_event_listners() == []
assert not event_host.get_family_event_listners()
# Reference clear_event_listners resets local lists only; dispatcher
# buckets keep the registration until an explicit remove (preserved
# wart, mirrored exactly).
assert dispatcher.event_listners[EventTypeSurface.MousePressEvent]

# Every wrapper routes to its exact event-type bucket.
routing_host = Mobject()
seen_types = []


def _routing_recorder(mob, event_data):
    seen_types.append(mob)


for name in (
    "add_mouse_motion_listner",
    "add_mouse_drag_listner",
    "add_mouse_press_listner",
    "add_mouse_release_listner",
    "add_mouse_scroll_listner",
    "add_key_press_listner",
    "add_key_release_listner",
):
    getattr(routing_host, name)(_routing_recorder)
registered = {
    listener.event_type for listener in routing_host.get_event_listners()
}
assert registered == {
    EventTypeSurface.MouseMotionEvent,
    EventTypeSurface.MouseDragEvent,
    EventTypeSurface.MousePressEvent,
    EventTypeSurface.MouseReleaseEvent,
    EventTypeSurface.MouseScrollEvent,
    EventTypeSurface.KeyPressEvent,
    EventTypeSurface.KeyReleaseEvent,
}
for name in (
    "remove_mouse_motion_listner",
    "remove_mouse_drag_listner",
    "remove_mouse_press_listner",
    "remove_mouse_release_listner",
    "remove_mouse_scroll_listner",
    "remove_key_press_listner",
    "remove_key_release_listner",
):
    getattr(routing_host, name)(_routing_recorder)
assert routing_host.get_event_listners() == []

# init_event_listners is idempotent and never discards live registrations.
init_host = Mobject()
init_host.init_event_listners()
init_host.add_key_press_listner(lambda mob, event_data: None)
init_host.init_event_listners()
assert len(init_host.get_event_listners()) == 1


# ---------------------------------------------------------------------------
# fm-5wq.4: Mobject shading, depth-test, clip-plane, and uniform family.
shading_mob = Mobject()
assert shading_mob.get_shading() == [0.0, 0.0, 0.0]
shading_mob.set_gloss(0.25)
assert shading_mob.get_gloss() == 0.25
assert shading_mob.get_shading() == [0.0, 0.25, 0.0]
shading_mob.set_reflectiveness(0.5, recurse=False)
shading_mob.set_shadow(0.75)
assert shading_mob.get_reflectiveness() == 0.5
assert shading_mob.get_shadow() == 0.75

shading_parent = Mobject()
shading_child = Mobject()
shading_parent.add(shading_child)
shading_parent.set_gloss(0.4)
assert shading_child.get_gloss() == 0.4
shading_child.set_reflectiveness(0.9)
assert shading_child.get_reflectiveness() == 0.9
assert shading_parent.get_reflectiveness() == 0.0

# set_uniform fans out over the requested family and lands in engine state.
uniform_parent = Mobject()
uniform_child = Mobject()
uniform_parent.add(uniform_child)
uniform_parent.set_uniform(recurse=True, anti_alias_width=2.5)
assert uniform_child.uniforms["anti_alias_width"] == 2.5
uniform_parent.set_uniforms({"anti_alias_width": 1.5})
assert uniform_parent.uniforms["anti_alias_width"] == 1.5
assert uniform_child.uniforms["anti_alias_width"] == 2.5
assert uniform_parent.get_uniforms() is uniform_parent.uniforms

# lock_uniforms records the keys only when no updaters are registered.
locked_mob = Mobject()
locked_mob.lock_uniforms(["anti_alias_width"])
assert locked_mob.locked_uniform_keys == {"anti_alias_width"}
locked_mob.add_updater(lambda mob: None)
locked_mob.lock_uniforms(["is_fixed_in_frame"])
assert locked_mob.locked_uniform_keys == {"anti_alias_width"}

# Frame pinning and depth test route through the typed uniform lanes.
pinned = Mobject()
assert not pinned.is_fixed_in_frame()
pinned.fix_in_frame()
assert pinned.is_fixed_in_frame()
pinned.unfix_from_frame()
assert not pinned.is_fixed_in_frame()

depth_mob = Mobject()
depth_mob.apply_depth_test()
assert bool(depth_mob.uniforms["depth_test"]) is True
depth_mob.deactivate_depth_test()
assert bool(depth_mob.uniforms["depth_test"]) is False

# Clip planes: single plane, multiple planes, deactivation, box clipping.
clip_mob = Mobject()
clip_mob.set_clip_plane([1.0, 0.0, 0.0], -2.0)
assert clip_mob.uniforms["clip_planes"][0] == [1.0, 0.0, 0.0, -2.0]
clip_mob.set_clip_planes(
    ([1.0, 0.0, 0.0], -1.0), ([0.0, 1.0, 0.0], -2.0)
)
planes_now = clip_mob.uniforms["clip_planes"]
assert planes_now[0] == [1.0, 0.0, 0.0, -1.0]
assert planes_now[1] == [0.0, 1.0, 0.0, -2.0]
assert planes_now[3] == [0.0, 0.0, 0.0, 0.0]
clip_mob.deactivate_clip_plane()
assert all(plane == [0.0, 0.0, 0.0, 0.0] for plane in clip_mob.uniforms["clip_planes"])

box_source = manimlib.Square(side_length=2.0).move_to([5.0, -3.0, 0.0])
clipped = manimlib.Dot().clip_to_box(box_source)
box_planes = clipped.uniforms["clip_planes"]
left_edge = box_source.get_x(manimlib.LEFT)
right_edge = box_source.get_x(manimlib.RIGHT)
bottom_edge = box_source.get_y(manimlib.DOWN)
top_edge = box_source.get_y(manimlib.UP)
assert box_planes[0] == [1.0, 0.0, 0.0, -left_edge]
assert box_planes[1] == [-1.0, 0.0, 0.0, right_edge]
assert box_planes[2] == [0.0, 1.0, 0.0, -bottom_edge]
assert box_planes[3] == [0.0, -1.0, 0.0, top_edge]


# ---------------------------------------------------------------------------
# fm-5wq.4: GLSL-era surface — Python-callable color functions bind over the
# live records; shader-code injection refuses by naming OOT-ARBITRARY-GLSL.
assert Mobject.shader_folder == ""

rgba_mob = manimlib.Point([1.0, 2.0, 3.0])
rgba_mob.set_color_by_rgba_func(
    lambda points: np.hstack((points * 0.5, np.ones((points.shape[0], 1))))
)
assert np.allclose(rgba_mob.data["rgba"][0], [0.5, 1.0, 1.5, 1.0])

rgb_mob = manimlib.Point([2.0, 4.0, 6.0])
rgb_mob.set_color_by_rgb_func(lambda points: points * 0.25, opacity=0.5)
assert np.allclose(rgb_mob.data["rgba"][0], [0.5, 1.0, 1.5, 0.5])

# Point-free receivers are the Reference's zero-row writes: silent no-ops.
empty_mob = Mobject()
empty_mob.set_color_by_rgb_func(lambda points: points, opacity=1.0)
empty_mob.set_color_by_rgba_func(lambda points: points)

# get_shader_data exposes the live structured records without an upload.
assert rgba_mob.get_shader_vert_indices() is None
records = rgba_mob.get_shader_data()
assert records.shape[0] == rgba_mob.get_num_points()
assert np.allclose(records["point"][0], [1.0, 2.0, 3.0])

# Every shader-injection and Context-dependent entry point refuses by name.
for surface_name, call in (
    ("replace_shader_code", lambda mob: mob.replace_shader_code("a", "b")),
    ("set_color_by_code", lambda mob: mob.set_color_by_code("color.rgb = vec3(1);")),
    (
        "set_color_by_xyz_func",
        lambda mob: mob.set_color_by_xyz_func("point.x"),
    ),
    ("init_shader_wrapper", lambda mob: mob.init_shader_wrapper(None)),
    ("get_shader_wrapper", lambda mob: mob.get_shader_wrapper(None)),
    ("get_shader_wrapper_list", lambda mob: mob.get_shader_wrapper_list(None)),
    ("render", lambda mob: mob.render(None, {})),
):
    try:
        call(rgba_mob)
    except NotImplementedError as error:
        assert "OOT-ARBITRARY-GLSL" in str(error), surface_name
    else:
        raise AssertionError("GLSL surface did not refuse: " + surface_name)


# ---------------------------------------------------------------------------
# fm-5wq.4: submobject structure and placement odds and ends.
structure_parent = Mobject()
first_kid = Mobject()
second_kid = Mobject()
structure_parent.add(first_kid)
structure_parent.replace_submobject(0, second_kid)
assert structure_parent.submobjects == [second_kid]
assert first_kid.parents == []
assert second_kid.parents == [structure_parent]
try:
    structure_parent.replace_submobject(0, "not a mobject")
except TypeError:
    pass
else:
    raise AssertionError("replace_submobject accepted a non-Mobject")

inserted = Mobject()
structure_parent.insert_submobject(0, inserted)
assert structure_parent.submobjects == [inserted, second_kid]
assert inserted.parents == [structure_parent]

# push_self_into_submobjects moves this mobject's own geometry into one copy.
pushed = manimlib.Dot()
pushed.push_self_into_submobjects()
assert pushed.get_num_points() == 0
assert len(pushed.submobjects) == 1
assert pushed.submobjects[0].get_num_points() > 0

# throw_error_if_no_points names the immediate calling frame.
pointless = Mobject()


def _points_consumer(mob):
    mob.throw_error_if_no_points()


try:
    _points_consumer(pointless)
except Exception as error:
    assert "_points_consumer" in str(error)
else:
    raise AssertionError("a pointless mobject passed the points refusal")

# put_start_on / put_end_on shift by the live endpoint delta.
segment = manimlib.Line(manimlib.LEFT, manimlib.RIGHT)
segment.put_start_on(10 * manimlib.RIGHT)
assert np.allclose(segment.get_start(), 10 * manimlib.RIGHT)
assert np.allclose(segment.get_end(), 12 * manimlib.RIGHT)
segment.put_end_on(manimlib.ORIGIN)
assert np.allclose(segment.get_end(), manimlib.ORIGIN)

# The z-index reference rides the declared group when present.
z_mob = Mobject()
z_mob.move_to([1.0, 2.0, 3.0])
assert np.allclose(z_mob.get_z_index_reference_point(), z_mob.get_center())
z_group = Mobject()
z_member = Mobject().add(z_group)
z_member.z_index_group = z_group
z_group.move_to([9.0, 9.0, 9.0])
assert np.allclose(z_member.get_z_index_reference_point(), z_group.get_center())


# ---------------------------------------------------------------------------
# fm-5wq.4: data alignment, interpolation, locking, and serialization.
left_shape = manimlib.Square(side_length=1.0)
right_shape = manimlib.Circle(radius=0.5)
assert not left_shape.is_aligned_with(right_shape)
left_shape.align_data_and_family(right_shape)
assert left_shape.is_aligned_with(right_shape)
assert right_shape.is_aligned_with(left_shape)

# interpolate lerps every unlocked lane; locked lanes keep their rows.
lerp_target = manimlib.Square(side_length=1.0).shift(2 * manimlib.RIGHT)
lerp_source = manimlib.Square(side_length=1.0)
aligned_target = lerp_target.copy()
aligned_work = lerp_source.copy()
aligned_work.align_data_and_family(aligned_target)
work = lerp_source.copy()
work.align_data_and_family(aligned_target)
work.lock_data(["point"])
work.interpolate(lerp_source, aligned_target, 0.25)
assert np.allclose(work.data["point"], lerp_source.data["point"])
work.unlock_data()
work.interpolate(lerp_source, aligned_target, 0.25)
expected_points = (1 - 0.25) * lerp_source.data["point"] + 0.25 * aligned_target.data["point"]
assert np.allclose(work.data["point"], expected_points)

# set_data round-trips a structured array of the exact dtype.
donor = manimlib.Dot().move_to([1.0, 1.0, 0.0])
receiver = manimlib.Dot()
receiver.set_data(donor.data.copy())
assert np.array_equal(receiver.data, donor.data)

# lock_matching_data locks lanes that already agree and marks constants.
match_a = manimlib.Dot()
match_b = manimlib.Dot()
match_a.align_data_and_family(match_b)


def _rows_match(arr1, arr2):
    return arr1.shape == arr2.shape and bool((arr1 == arr2).all())


match_a.lock_matching_data(match_a, match_b)
for key in match_a.data.dtype.names:
    if _rows_match(match_a.data[key], match_b.data[key]):
        assert key in getattr(match_a, "locked_data_keys", set())

# has_same_shape_as normalizes center and height.
square_one = manimlib.Square(side_length=1.0)
square_two = manimlib.Square(side_length=2.0)
assert square_two.has_same_shape_as(square_one)
circle_shape = manimlib.Circle(radius=1.0)
assert not circle_shape.has_same_shape_as(square_one)

# digest_mobject_attrs adopts mobject-valued attributes into the family.
digest_parent = Mobject()
adopted = Mobject()
digest_parent.orphan_attr = adopted
assert adopted not in digest_parent.submobjects
digest_parent.digest_mobject_attrs()
assert adopted in digest_parent.submobjects

# stash_mobject_pointers hides relationship pointers for the wrapped call.
@Mobject.stash_mobject_pointers
def _mutate_without_pointers(mob):
    return (mob.parents, mob.target, mob.saved_state)


stash_probe = Mobject()
stash_child = Mobject()
stash_probe.add(stash_child)
stash_probe.target = Mobject()
seen = _mutate_without_pointers(stash_probe)
assert seen == ([], None, None)
assert stash_probe.parents == []
assert isinstance(stash_probe.target, Mobject)
assert stash_probe.saved_state is None
assert stash_probe.submobjects == [stash_child]
assert stash_child.parents == [stash_probe]

# serialize/deserialize round-trips through become.
original = manimlib.Square(side_length=1.5).shift([1.0, 2.0, 3.0])
payload = original.serialize()
reviver = manimlib.Square()
returned = reviver.deserialize(payload)
assert returned is reviver
assert np.allclose(reviver.get_center(), original.get_center())
assert reviver.get_width() == original.get_width()

# pointwise_become_partial is the base-class no-op the Reference documents.
partial_mob = Mobject()
assert partial_mob.pointwise_become_partial(Mobject(), 0.1, 0.9) is partial_mob


# ---------------------------------------------------------------------------
# fm-5wq.4: geometry accessors, dimension setters, and alignment.
probe = manimlib.Square(side_length=2.0).shift([3.0, -1.0, 0.0])
assert np.allclose(probe.get_center(), [3.0, -1.0, 0.0])
assert probe.get_width() == 2.0
assert probe.get_height() == 2.0
assert probe.get_depth() == 0.0
assert probe.get_shape() == (2.0, 2.0, 0.0)
assert probe.length_over_dim(1) == 2.0
assert probe.has_points()
assert not Mobject().has_points()
assert np.allclose(probe.get_right(), [4.0, -1.0, 0.0])
assert np.allclose(probe.get_left(), [2.0, -1.0, 0.0])
assert np.allclose(probe.get_top(), [3.0, 0.0, 0.0])
assert np.allclose(probe.get_bottom(), [3.0, -2.0, 0.0])
assert np.allclose(probe.get_corner(manimlib.UR), [4.0, 0.0, 0.0])
assert np.allclose(probe.get_edge_center(manimlib.DOWN), [3.0, -2.0, 0.0])
assert np.allclose(probe.get_zenith(), probe.get_center())
assert np.allclose(probe.get_nadir(), probe.get_center())
assert np.allclose(probe.get_bounding_box_point(manimlib.RIGHT), [4.0, -1.0, 0.0])
assert probe.get_x() == 3.0
assert probe.get_y(manimlib.UP) == 0.0
assert probe.get_z() == 0.0
assert probe.get_coord(0, manimlib.LEFT) == 2.0

# set_coord/set_x/set_y/set_z shift so the chosen edge lands on the value.
mover = manimlib.Square(side_length=2.0).move_to([10.0, 10.0, 0.0])
mover.set_x(0.0)
assert mover.get_x() == 0.0
assert np.allclose(mover.get_center()[1:], [10.0, 0.0])
mover.set_y(-3.0, manimlib.DOWN)
assert mover.get_bottom()[1] == -3.0
mover.set_z(7.0)
assert mover.get_z() == 7.0
mover.set_coord(5.0, dim=1, direction=manimlib.UP)
assert mover.get_top()[1] == 5.0

# set_width without stretch rescales uniformly; with stretch only that dim.
uniform = manimlib.Square(side_length=2.0)
uniform.set_width(6.0)
assert uniform.get_width() == 6.0
assert uniform.get_height() == 6.0
stretched = manimlib.Square(side_length=2.0)
stretched.set_height(8.0, stretch=True)
assert stretched.get_height() == 8.0
assert stretched.get_width() == 2.0
flat = manimlib.Square(side_length=2.0)
flat.rescale_to_fit(4.0, 2, stretch=True)
assert flat.get_depth() == 0.0
solid = manimlib.Cube(side_length=2.0)
solid.rescale_to_fit(4.0, 2, stretch=True)
assert solid.get_depth() == 4.0
assert solid.get_width() == 2.0
for fitter in (manimlib.Square(side_length=2.0),):
    fitter.stretch_to_fit_width(3.0)
    assert fitter.get_width() == 3.0
    fitter.stretch_to_fit_height(9.0)
    assert fitter.get_height() == 9.0
    fitter.stretch_to_fit_depth(1.5)
    assert fitter.get_depth() == 0.0

# stretch variants move edges, not the anchor point.
stretch_probe = manimlib.Square(side_length=2.0)
stretch_probe.stretch_in_place(3.0, 0)
assert stretch_probe.get_width() == 6.0
anchor = manimlib.Square(side_length=2.0)
fixed_point = anchor.get_corner(manimlib.DL).copy()
anchor.stretch_about_point(2.0, 1, fixed_point)
assert np.allclose(anchor.get_corner(manimlib.DL), fixed_point)
assert anchor.get_height() == 4.0

# set_shape stretches every named dimension in one call.
shaped = manimlib.Square(side_length=2.0)
shaped.set_shape(width=5.0, height=1.0)
assert shaped.get_shape() == (5.0, 1.0, 0.0)

# min/max clamps fire only outside the band and keep the other dims honest.
capped = manimlib.Square(side_length=4.0)
capped.set_max_width(2.0)
assert capped.get_width() == 2.0
assert capped.get_height() == 2.0
kept = manimlib.Square(side_length=1.0)
kept.set_max_width(2.0)
assert kept.get_width() == 1.0
floored = manimlib.Square(side_length=1.0)
floored.set_min_height(3.0)
assert floored.get_height() == 3.0
untouched = manimlib.Square(side_length=4.0)
untouched.set_min_height(3.0)
assert untouched.get_height() == 4.0
depthful = manimlib.Cube(side_length=2.0)
depthful.set_min_depth(3.0)
assert depthful.get_depth() == 3.0
depthful.set_max_depth(1.0)
assert depthful.get_depth() == 1.0
flat_depth = manimlib.Square(side_length=2.0)
flat_depth.set_min_depth(1.0)
assert flat_depth.get_depth() == 0.0

# center/align_on_border/align_to land the box where the names promise.
wanderer = manimlib.Square(side_length=1.0).shift([20.0, -20.0, 4.0])
wanderer.center()
assert np.allclose(wanderer.get_center(), [0.0, 0.0, 0.0])
edge_rider = manimlib.Square(side_length=1.0).shift([-14.0, 14.0, 0.0])
edge_rider.align_on_border(manimlib.LEFT, buff=1.0)
assert edge_rider.get_left()[0] == -manimlib.FRAME_X_RADIUS + 1.0
corner_rider = manimlib.Square(side_length=1.0)
corner_rider.to_corner(manimlib.UR, buff=0.5)
assert corner_rider.get_right()[0] == manimlib.FRAME_X_RADIUS - 0.5
assert corner_rider.get_top()[1] == manimlib.FRAME_Y_RADIUS - 0.5
edge_hugger = manimlib.Square(side_length=1.0)
edge_hugger.to_edge(manimlib.DOWN, buff=2.0)
assert edge_hugger.get_bottom()[1] == -manimlib.FRAME_Y_RADIUS + 2.0

# C-16 (BN-07): the Reference's frame-marker constants are frame-radius-scaled,
# which breaks every direction-argument use (align_on_border subtracts
# buff * direction unnormalized). FrankenManim resolves them to unit
# directions; frame math keeps the FRAME_*_RADIUS spellings.
assert np.allclose(manimlib.TOP, manimlib.UP)
assert np.allclose(manimlib.BOTTOM, manimlib.DOWN)
assert np.allclose(manimlib.LEFT_SIDE, manimlib.LEFT)
assert np.allclose(manimlib.RIGHT_SIDE, manimlib.RIGHT)
marker = manimlib.Square(side_length=2.0)
marker.to_edge(manimlib.BOTTOM, buff=0.5)
assert np.isclose(marker.get_bottom()[1], -manimlib.FRAME_Y_RADIUS + 0.5)
marker.to_edge(manimlib.TOP, buff=1.5)
assert np.isclose(marker.get_top()[1], manimlib.FRAME_Y_RADIUS - 1.5)
marker.to_edge(manimlib.LEFT_SIDE, buff=0.25)
assert np.isclose(marker.get_left()[0], -manimlib.FRAME_X_RADIUS + 0.25)
marker.to_edge(manimlib.RIGHT_SIDE, buff=0.75)
assert np.isclose(marker.get_right()[0], manimlib.FRAME_X_RADIUS - 0.75)
# The bound path routes through Stage::to_edge and must agree with the
# detached branch under the C-16 unit directions.
bound_scene = Scene()
bound_marker = manimlib.Square(side_length=2.0)
bound_scene.add(bound_marker)
bound_marker.to_edge(manimlib.BOTTOM, buff=0.5)
assert np.isclose(bound_marker.get_bottom()[1], -manimlib.FRAME_Y_RADIUS + 0.5)
bound_marker.to_edge(manimlib.TOP, buff=1.0)
assert np.isclose(bound_marker.get_top()[1], manimlib.FRAME_Y_RADIUS - 1.0)
align_target = manimlib.Square(side_length=2.0).move_to([5.0, -5.0, 0.0])
follower = manimlib.Square(side_length=1.0)
follower.align_to(align_target, manimlib.RIGHT)
assert abs(follower.get_center()[1] + 1.0) > 1e-6
follower.align_to([0.0, 9.0, 0.0], manimlib.UP)
assert follower.get_top()[1] == 9.0

# match_* copies one coordinate or dimension from the donor.
donor = manimlib.Square(side_length=3.0).move_to([-4.0, 4.0, 0.0])
matchee = manimlib.Square(side_length=1.0)
matchee.match_dim_size(donor, 0)
assert matchee.get_width() == 3.0
matchee.match_height(donor)
assert matchee.get_height() == 3.0
matchee.match_x(donor)
assert matchee.get_x() == donor.get_x()
matchee.match_y(donor, direction=manimlib.DOWN)
assert matchee.get_bottom()[1] == donor.get_bottom()[1]
matchee.match_coord(donor, 0, direction=manimlib.LEFT)
assert matchee.get_left()[0] == donor.get_left()[0]
matchee.match_width(donor)
assert matchee.get_width() == 3.0

# A flat mobject has a zero depth extent, so depth matching needs solid
# geometry; the unstretched match scales uniformly.
solid_donor = manimlib.Cube(side_length=2.5)
solid_matchee = manimlib.Cube(side_length=1.0)
solid_matchee.match_depth(solid_donor)
assert solid_matchee.get_depth() == 2.5
assert solid_matchee.get_width() == 2.5
zdonor = manimlib.Square(side_length=1.0).shift([0.0, 0.0, -6.0])
matchee.match_z(zdonor)
assert matchee.get_z() == -6.0

# get_pieces returns childless partials spanning the original run.
pieced = manimlib.Square(side_length=2.0)
pieces = pieced.get_pieces(4)
assert type(pieces) is manimlib.Group
assert len(pieces) == 4
assert all(isinstance(piece, manimlib.Square) for piece in pieces)
assert all(len(piece.submobjects) == 0 for piece in pieces)
assert np.allclose(pieces[0].get_start(), pieced.get_start())
assert np.allclose(pieces[-1].get_end(), pieced.get_end())


# ---------------------------------------------------------------------------
# fm-5wq.4: the transform family — shift, scale, rotate, flip, stretch,
# placement, replacement, and surrounding.

# shift translates the whole family; children ride along unchanged.
shifter = manimlib.Square(side_length=1.0)
rider = manimlib.Dot().move_to([0.25, 0.0, 0.0])
shifter.add(rider)
shifter.shift([2.0, -3.0, 0.5])
assert np.allclose(shifter.get_center(), [2.0, -3.0, 0.5])
assert np.allclose(rider.get_center(), [2.25, -3.0, 0.5])

# scale defaults to about the center and honours about_point/about_edge;
# the min_scale_factor clamp keeps collapse-from-zero finite.
scaler = manimlib.Square(side_length=2.0)
scaler.scale(3.0)
assert scaler.get_width() == 6.0
assert np.allclose(scaler.get_center(), [0.0, 0.0, 0.0])
pinned = manimlib.Square(side_length=2.0)
anchor = pinned.get_corner(manimlib.DL).copy()
pinned.scale(2.0, about_point=anchor)
assert np.allclose(pinned.get_corner(manimlib.DL), anchor)
assert pinned.get_width() == 4.0
hinged = manimlib.Square(side_length=2.0)
hinged.scale(2.0, about_edge=manimlib.LEFT)
assert hinged.get_left()[0] == -1.0
assert hinged.get_width() == 4.0
tiny = manimlib.Square(side_length=2.0)
tiny.scale(0.0)
assert 0.0 < tiny.get_width() < 1e-6

# rotate turns in-plane about the box center by default and about a caller
# point when given; rotate_about_origin is the origin-pivoted spelling.
spinner = manimlib.Rectangle(width=4.0, height=1.0)
spinner.rotate(manimlib.PI / 2.0)
assert np.allclose([spinner.get_width(), spinner.get_height()], [1.0, 4.0])
offset = manimlib.Rectangle(width=4.0, height=1.0).move_to([3.0, 0.0, 0.0])
offset.rotate(manimlib.PI / 2.0, about_point=manimlib.ORIGIN)
assert np.allclose(offset.get_center(), [0.0, 3.0, 0.0])
assert np.allclose([offset.get_width(), offset.get_height()], [1.0, 4.0])
origin_turn = manimlib.Rectangle(width=4.0, height=1.0).move_to([3.0, 0.0, 0.0])
origin_turn.rotate_about_origin(manimlib.PI / 2.0)
assert np.allclose(origin_turn.get_center(), [0.0, 3.0, 0.0])

# flip rotates half a turn about the named axis, mirroring the OTHER two
# coordinates through the family box's axis plane.
sideways = manimlib.VGroup(manimlib.Dot().move_to([2.0, 1.0, 0.0]),
                           manimlib.Dot().move_to([-2.0, -1.0, 0.0]))
east_dot, west_dot = sideways[0], sideways[1]
sideways.flip(manimlib.UP)
assert np.allclose(east_dot.get_center(), [-2.0, 1.0, 0.0])
assert np.allclose(west_dot.get_center(), [2.0, -1.0, 0.0])
sideways.flip(manimlib.RIGHT)
assert np.allclose(east_dot.get_center(), [-2.0, -1.0, 0.0])
assert np.allclose(west_dot.get_center(), [2.0, 1.0, 0.0])

# stretch applies one axis factor about the box center.
stretched = manimlib.Rectangle(width=4.0, height=1.0)
stretched.stretch(2.0, 1)
assert np.allclose([stretched.get_width(), stretched.get_height()], [4.0, 2.0])

# move_to lands the chosen edge on the target.
mover = manimlib.Square(side_length=2.0)
mover.move_to([10.0, 20.0, 0.0])
assert np.allclose(mover.get_center(), [10.0, 20.0, 0.0])
mover.move_to([0.0, 0.0, 0.0], aligned_edge=manimlib.RIGHT)
assert mover.get_right()[0] == 0.0
assert np.allclose(mover.get_center(), [-1.0, 0.0, 0.0])

# next_to places beside a donor or raw point with buff along direction,
# honouring aligned_edge.
donor = manimlib.Square(side_length=2.0).move_to([0.0, 0.0, 0.0])
neighbor = manimlib.Square(side_length=1.0)
neighbor.next_to(donor, manimlib.RIGHT, buff=0.5)
assert neighbor.get_left()[0] == 1.5
assert neighbor.get_y() == 0.0
point_neighbor = manimlib.Square(side_length=1.0)
point_neighbor.next_to([5.0, 9.0, 0.0], manimlib.DOWN, buff=0.25)
assert np.allclose(point_neighbor.get_top(), [5.0, 8.75, 0.0])
aligned_neighbor = manimlib.Square(side_length=1.0).shift([-30.0, 30.0, 0.0])
aligned_neighbor.next_to(donor, manimlib.UP, buff=1.0, aligned_edge=manimlib.LEFT)
assert aligned_neighbor.get_bottom()[1] == 2.0
assert aligned_neighbor.get_left()[0] == -1.0

# put_start_and_end_on rescales and rotates the run onto the segment;
# coincident endpoints are a loud refusal exactly as documented.
segment = manimlib.Line(start=[0.0, 0.0, 0.0], end=[1.0, 0.0, 0.0])
segment.put_start_and_end_on([0.0, 0.0, 0.0], [3.0, 4.0, 0.0])
assert np.allclose(segment.get_start(), [0.0, 0.0, 0.0])
assert np.allclose(segment.get_end(), [3.0, 4.0, 0.0])
# put_start_and_end_on rescales and rotates the run onto the segment; a
# zero-length target collapses through min_scale_factor instead of raising.
segment = manimlib.Line(start=[0.0, 0.0, 0.0], end=[1.0, 0.0, 0.0])
segment.put_start_and_end_on([0.0, 0.0, 0.0], [3.0, 4.0, 0.0])
assert np.allclose(segment.get_start(), [0.0, 0.0, 0.0])
assert np.allclose(segment.get_end(), [3.0, 4.0, 0.0])
collapsed = manimlib.Line(start=[-1.0, 0.0, 0.0], end=[1.0, 0.0, 0.0])
collapsed.put_start_and_end_on([2.0, 0.0, 0.0], [2.0, 0.0, 0.0])
assert np.allclose(collapsed.get_start(), [2.0, 0.0, 0.0])
residue = float(np.linalg.norm(collapsed.get_end() - collapsed.get_start()))
assert 0.0 < residue < 1e-6
vacated = manimlib.Square(side_length=4.0).move_to([-5.0, 5.0, 0.0])
taker = manimlib.Square(side_length=1.0)
taker.replace(vacated)
assert taker.get_width() == 4.0
assert taker.get_height() == 4.0
assert taker.get_x() == vacated.get_x()
stretch_taker = manimlib.Square(side_length=1.0)
wide_donor = manimlib.Rectangle(width=8.0, height=2.0).move_to([1.0, -1.0, 0.0])
stretch_taker.replace(wide_donor, stretch=True)
assert np.allclose(
    [stretch_taker.get_width(), stretch_taker.get_height()], [8.0, 2.0]
)

# surround wraps the donor with a buff margin on every side.
wrapped = manimlib.Square(side_length=2.0)
wrapper = manimlib.Square(side_length=0.5)
wrapper.surround(wrapped, buff=0.75)
assert wrapper.get_width() == 3.5
assert wrapper.get_height() == 3.5
assert np.allclose(wrapper.get_center(), wrapped.get_center())

# The base Mobject contract scales around the matched dimension; unlike the
# rectangle override, its buffer is the total added span in that dimension.
base_wrapped = Mobject().set_points([[1.0, 1.0, 0.0], [5.0, 3.0, 0.0]])
base_wrapper = Mobject().set_points([[-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]])
base_scene = Scene().add(base_wrapped, base_wrapper)
assert base_wrapper.surround(base_wrapped, buff=1.0) is base_wrapper
assert np.allclose(base_wrapper.get_center(), [3.0, 2.0, 0.0])
assert np.allclose([base_wrapper.get_width(), base_wrapper.get_height()], [5.0, 5.0])
assert base_wrapper._is_bound()
assert base_scene.mobjects == [base_wrapped, base_wrapper]
assert base_wrapper.surround(base_wrapped, dim_to_match=1, stretch=True, buff=0.5) is base_wrapper
assert np.allclose([base_wrapper.get_width(), base_wrapper.get_height()], [5.0, 2.5])
assert np.allclose(base_wrapped.get_points(), [[1.0, 1.0, 0.0], [5.0, 3.0, 0.0]])


# ---------------------------------------------------------------------------
# fm-5wq.4: the color/style family and looks_identical.

# set_color routes the Reference RED through fmn-core's one color model.
tinted = manimlib.Square(side_length=1.0).set_color(manimlib.RED)
assert tinted.get_color() == "#FC6255"
assert tinted.get_opacity() == 1.0
tinted.set_opacity(0.35)
assert abs(tinted.get_opacity() - 0.35) < 1e-6
assert tinted.get_color() == "#FC6255"
tinted.set_color(manimlib.BLUE, opacity=0.6)
assert tinted.get_color() == "#58C4DD"
assert abs(tinted.get_opacity() - 0.6) < 1e-6

# Recursion paints children; recurse=False leaves them untouched.
style_parent = manimlib.Square(side_length=3.0)
kid = manimlib.Dot().move_to([0.5, 0.5, 0.0])
style_parent.add(kid)
style_parent.set_color(manimlib.GREEN, opacity=0.8)
assert kid.get_color() == "#83C167"
assert abs(kid.get_opacity() - 0.8) < 1e-6
shy_parent = manimlib.Square(side_length=3.0).set_color(manimlib.YELLOW)
shy_kid = manimlib.Dot(color=manimlib.RED).move_to([0.25, 0.25, 0.0])
shy_parent.add(shy_kid)
shy_parent.set_color(manimlib.PURPLE, recurse=False)
assert shy_kid.get_color() == "#FC6255"
# The base rgba lane exists only on the base record schema; a plain
# Mobject carries it, while vectorized mobjects route through fill/stroke.
graded = manimlib.Mobject()
graded.resize(4)
n_points = len(graded.get_points())
alphas = np.linspace(0.1, 0.9, n_points)
rgba_rows = np.hstack((np.full((n_points, 3), 0.5), alphas.reshape(-1, 1)))
graded.set_rgba_array(rgba_rows)
assert np.allclose(graded.get_opacities(), alphas)
assert abs(float(np.mean(graded.get_opacities())) - 0.5) < 1e-6

# A color list interpolates across the point run by the Reference rule,
# landing in both style lanes of a vectorized mobject.
gradient = manimlib.Square(side_length=1.0).set_color([manimlib.RED, manimlib.BLUE])
for lane in ("fill_rgba", "stroke_rgba"):
    assert np.allclose(
        gradient.data[lane][0, :3],
        [252 / 255, 98 / 255, 85 / 255],
        atol=1e-4,
    )
    assert np.allclose(
        gradient.data[lane][-1, :3],
        [88 / 255, 196 / 255, 221 / 255],
        atol=1e-4,
    )

# looks_identical compares families member-wise over dtype, data keys,
# and uniforms: true for a fresh copy, false for any semantic drift.
original = manimlib.VGroup(
    manimlib.Square(side_length=1.0).set_color(manimlib.RED),
    manimlib.Dot().move_to([1.0, 0.0, 0.0]),
)
assert original.looks_identical(original.copy())
recolored = original.copy()
recolored[0].set_opacity(0.5)
assert not original.looks_identical(recolored)
reshaped = original.copy()
reshaped[1].shift([0.25, 0.0, 0.0])
assert not original.looks_identical(reshaped)
shrunk_family = manimlib.Square(side_length=1.0)
assert not original.looks_identical(shrunk_family)
plain_a = manimlib.Square(side_length=2.0)
plain_b = manimlib.Square(side_length=2.0)
assert plain_a.looks_identical(plain_b)


# ---------------------------------------------------------------------------
# fm-5wq.4: the arrange family.

# arrange lines members up along direction with the requested buff, then
# centers the family box.
trio = manimlib.VGroup(
    manimlib.Square(side_length=0.5),
    manimlib.Square(side_length=1.0),
    manimlib.Square(side_length=1.5),
).shift([10.0, 10.0, 0.0])
trio.arrange(manimlib.RIGHT, buff=0.25)
gap01 = trio[1].get_left()[0] - trio[0].get_right()[0]
gap12 = trio[2].get_left()[0] - trio[1].get_right()[0]
assert abs(gap01 - 0.25) < 1e-9
assert abs(gap12 - 0.25) < 1e-9
assert np.allclose(trio.get_center(), [0.0, 0.0, 0.0])

# arrange_in_grid lays rows-first by default into distinct row and column
# positions.
grid = manimlib.VGroup(*[manimlib.Square(side_length=0.5) for _ in range(6)])
grid.arrange_in_grid(n_cols=3, buff=0.5)
assert len({round(m.get_center()[1], 9) for m in grid}) == 2
assert len({round(m.get_center()[0], 9) for m in grid}) == 3

# arrange_to_fit_dim packs every member into an exact span with even gaps.
packed = manimlib.VGroup(
    manimlib.Square(side_length=1.0),
    manimlib.Square(side_length=2.0),
    manimlib.Square(side_length=1.0),
)
packed.arrange_to_fit_width(8.0)
span = packed[-1].get_right()[0] - packed[0].get_left()[0]
assert abs(span - 8.0) < 1e-9
even_gap = (8.0 - sum(m.get_width() for m in packed)) / 2
assert np.allclose(
    [
        packed[1].get_left()[0] - packed[0].get_right()[0],
        packed[2].get_left()[0] - packed[1].get_right()[0],
    ],
    [even_gap, even_gap],
)
stacked = manimlib.VGroup(
    manimlib.Square(side_length=1.0),
    manimlib.Square(side_length=1.0),
    manimlib.Square(side_length=1.0),
)
stacked.arrange_to_fit_height(6.0)
vspan = stacked[-1].get_top()[1] - stacked[0].get_bottom()[1]
assert abs(vspan - 6.0) < 1e-9
depthful = manimlib.Group(
    manimlib.Cube(side_length=1.0),
    manimlib.Cube(side_length=1.0),
)
depthful.arrange_to_fit_depth(5.0)
dspan = depthful[-1].get_edge_center(manimlib.OUT)[2] - depthful[0].get_edge_center(manimlib.IN)[2]
assert abs(dspan - 5.0) < 1e-9


# ---------------------------------------------------------------------------
# fm-5wq.4: the points, copy, and state family.

# Endpoint accessors and reversal over a vectorized run.
runner = manimlib.Line(start=[-2.0, 0.0, 0.0], end=[2.0, 0.0, 0.0])
assert np.allclose(runner.get_start(), [-2.0, 0.0, 0.0])
assert np.allclose(runner.get_end(), [2.0, 0.0, 0.0])
pairs = runner.get_start_and_end()
assert np.allclose(pairs[0], [-2.0, 0.0, 0.0]) and np.allclose(pairs[1], [2.0, 0.0, 0.0])
runner.reverse_points()
assert np.allclose(runner.get_start(), [2.0, 0.0, 0.0])
assert np.allclose(runner.get_end(), [-2.0, 0.0, 0.0])

# Base-Mobject point_from_proportion walks raw record rows; the VMobject
# override walks true arc length (BN-03).
row_mob = manimlib.Mobject()
row_mob.resize(3)
for index in range(3):
    row_mob.data["point"][index] = [index * 1.0, 0.0, 0.0]
assert np.allclose(row_mob.point_from_proportion(0.0), [0.0, 0.0, 0.0])
assert np.allclose(row_mob.pfp(1.0), [2.0, 0.0, 0.0])
assert np.allclose(row_mob.point_from_proportion(0.25), [0.5, 0.0, 0.0])
assert np.allclose(runner.point_from_proportion(0.5), [0.0, 0.0, 0.0])

# match_points adopts the donor's exact record layout.
donor_pts = manimlib.Line(start=[0.0, 0.0, 0.0], end=[4.0, -3.0, 0.0])
adopter = manimlib.Square(side_length=1.0)
adopter.match_points(donor_pts)
assert np.allclose(adopter.get_points(), donor_pts.get_points())

# Copies are independent at the record level.
twin = runner.copy()
twin.shift([5.0, 0.0, 0.0])
assert not np.allclose(twin.get_center(), runner.get_center())

# generate_target/save_state/restore round-trip through the Reference
# attribute contract.
moved = manimlib.Square(side_length=1.0).move_to([3.0, 0.0, 0.0])
target = moved.generate_target()
assert target is moved.target
assert moved.looks_identical(moved.target)
target.shift([-6.0, 0.0, 0.0])
moved.save_state()
assert hasattr(moved, "saved_state")
moved.shift([0.0, 9.0, 0.0])
moved.restore()
assert np.allclose(moved.get_center(), [3.0, 0.0, 0.0])

# replicate returns the group class holding n independent copies.
clones = manimlib.Dot().replicate(3)
assert type(clones) is manimlib.VGroup
assert len(clones) == 3
clones[0].shift([1.0, 0.0, 0.0])
assert not np.allclose(clones[0].get_center(), clones[1].get_center())


# ---------------------------------------------------------------------------
# fm-5wq.4: the updater lifecycle family and the animate builder.

tick_box = manimlib.Square(side_length=1.0)
dt_log = []


def push_dt(mob, dt):
    dt_log.append(dt)
    mob.shift([dt, 0.0, 0.0])


tick_box.add_updater(push_dt)
tick_box.update(0.5)
assert np.allclose(tick_box.get_center(), [0.5, 0.0, 0.0])

# Suspension stops execution; resume without call_updater stays quiet until
# the next explicit update.
tick_box.suspend_updating()
tick_box.update(1.0)
assert np.allclose(tick_box.get_center(), [0.5, 0.0, 0.0])
tick_box.resume_updating(call_updater=False)
tick_box.update(0.25)
assert len(dt_log) == 3
assert abs(dt_log[-1] - 0.25) < 1e-12

# remove_updater detaches the exact handler; clear_updaters empties family.
tick_box.remove_updater(push_dt)
tick_box.update(1.0)
assert len(dt_log) == 3
echo = manimlib.Dot().move_to([9.9, 9.9, 0.0])
echo.add_updater(lambda m: m.set_opacity(0.5))
echo.clear_updaters()
assert not echo.has_updaters()

# always chains bound-method calls per frame; f_always feeds function
# results into the named setter.
drifter = manimlib.Dot()
drifter.always.shift([0.1, 0.0, 0.0])
drifter.update(0)
drifter.update(0)
assert np.allclose(drifter.get_center(), [0.3, 0.0, 0.0])
x_source = [1.0]
slider = manimlib.Dot()
slider.f_always.set_x(lambda: x_source[0])
slider.update(0)
assert slider.get_x() == 1.0
x_source[0] = 3.0
slider.update(0)
assert slider.get_x() == 3.0

# The animate builder intercepts chained calls and builds one method
# animation against the generated target.
builder_src = manimlib.Square(side_length=1.0).move_to([2.0, 0.0, 0.0])
built = builder_src.animate.shift([1.0, 0.0, 0.0]).scale(2.0)
animation = built.build()
assert isinstance(animation, manimlib.Animation)
assert builder_src.get_width() == 1.0
assert np.allclose(builder_src.get_center(), [2.0, 0.0, 0.0])


# ---------------------------------------------------------------------------
# fm-5wq.4: schema attributes, init seams, wag, Point/Group, and the
# continuous bounding box.

assert manimlib.Mobject.render_primitive == 5
assert manimlib.Square.render_primitive == 5
assert manimlib.Mobject().data.dtype.names == ("point", "rgba")
assert manimlib.Mobject.aligned_data_keys == ["point"]
assert manimlib.Mobject.pointlike_data_keys == ["point"]

# init_colors re-applies the CONSTRUCTOR style seeds (post-construction
# set_color mutates lanes but not the seed attributes).
reseeded = manimlib.Mobject(color=manimlib.RED)
assert reseeded.color == "#FC6255"
assert reseeded.opacity == 1.0
reseeded.set_opacity(0.2)
assert abs(reseeded.get_opacity() - 0.2) < 1e-6
reseeded.init_colors()
assert reseeded.get_color() == "#FC6255"
assert abs(reseeded.get_opacity() - 1.0) < 1e-6

# wag offsets points along direction by the normalized axis projection.
wagged = manimlib.Mobject()
wagged.resize(3)
for index in range(3):
    wagged.data["point"][index] = [0.0, index - 1.0, 0.0]
wagged.wag(manimlib.RIGHT, manimlib.DOWN, 1.0)
assert np.allclose(
    [row[0] for row in wagged.get_points()],
    [1.0, 0.5, 0.0],
)
curved = wagged.copy()
curved.data["point"][:, 0] = 0.0
curved.wag(manimlib.RIGHT, manimlib.DOWN, 2.0)
assert np.allclose(
    [row[0] for row in curved.get_points()],
    [1.0, 0.25, 0.0],
)

# The constructor style seeds ride through family copies too.
fam = manimlib.Group(manimlib.Mobject(color=manimlib.ORANGE))
assert fam[0].color == "#FF862F"

# set_depth rescales the depth axis through the one stretch path.
depthy = manimlib.Cube(side_length=2.0)
depthy.set_depth(5.0)
assert depthy.get_depth() == 5.0
flat = manimlib.Square(side_length=1.0)
flat.set_depth(3.0, stretch=True)
assert flat.get_depth() == 0.0
assert flat.get_width() == 1.0

# Point carries an artificial one-record box around its location.
pinned_point = manimlib.Point([2.0, -3.0, 0.5])
assert np.allclose(pinned_point.get_location(), [2.0, -3.0, 0.5])
assert pinned_point.get_width() == 1e-6
assert pinned_point.get_height() == 1e-6
assert np.allclose(pinned_point.get_bounding_box_point(manimlib.UR), [2.0, -3.0, 0.5])
pinned_point.set_location([-1.0, 4.0, 0.0])
assert np.allclose(pinned_point.get_location(), [-1.0, 4.0, 0.0])

# Group spreads a single iterable of members and split returns them.
members = [manimlib.Dot(), manimlib.Dot(), manimlib.Dot()]
bundle = manimlib.Group(members)
assert len(bundle) == 3
assert bundle.split() is bundle.submobjects or bundle.split() == bundle.submobjects

# get_continuous_bounding_box_point returns where the ray from the center
# exits the box — here the right edge at fractional height, not a corner.
slab = manimlib.Rectangle(width=4.0, height=2.0)
exit_point = slab.get_continuous_bounding_box_point([2.0, 0.5, 0.0])
assert np.allclose(exit_point, [2.0, 0.5, 0.0])

# override_animate routes the animate builder to the replacement factory.
@manimlib.override_animate(manimlib.Mobject.wag)
def _wag_override_factory(mobject, direction=None, axis=None, wag_factor=1.0):
    return manimlib.Animation(mobject)


probe = manimlib.Square(side_length=1.0)
override_built = probe.animate.wag(wag_factor=2.0).build()
assert isinstance(override_built, manimlib.Animation)


# ---------------------------------------------------------------------------
# fm-5wq.4: module-surface fidelity for the Reference's imported names.

_mobj_module = sys.modules["manimlib.mobject.mobject"]
assert _mobj_module.os is importlib.import_module("os")
assert _mobj_module.sys is importlib.import_module("sys")
assert _mobj_module.random is importlib.import_module("random")
assert _mobj_module.copy is importlib.import_module("copy")
assert _mobj_module.pickle is importlib.import_module("pickle")
assert _mobj_module.wraps is importlib.import_module("functools").wraps
assert _mobj_module.TypeVar is importlib.import_module("typing").TypeVar
assert _mobj_module.Generic is importlib.import_module("typing").Generic

# The generic TypeVar spelling resolves to a real typing object with the
# Reference's name, not its declared string.
_submobject_type = _mobj_module.SubmobjectType
assert type(_submobject_type).__name__ == "TypeVar"
assert _submobject_type.__name__ == "SubmobjectType"
assert manimlib.SubmobjectType is _submobject_type

_vm_mod = importlib.import_module("manimlib.mobject.types.vectorized_mobject")
assert type(getattr(_vm_mod, "SubVmobjectType")).__name__ == "TypeVar"
assert _vm_mod.SubVmobjectType.__name__ == "SubVmobjectType"
assert issubclass(manimlib.VGroup, manimlib.VMobject)
group = manimlib.VGroup(manimlib.Square(), manimlib.Circle())
assert len(group) == 2

style_square = manimlib.Square()
assert style_square.set_fill(manimlib.BLUE, opacity=0.5) is style_square
assert style_square.has_fill()
assert style_square.get_fill_opacity() > 0
assert style_square.set_stroke(manimlib.YELLOW, width=3.0, opacity=1.0) is style_square
assert style_square.has_stroke()
assert np.isclose(style_square.get_stroke_width(), 3.0)
assert style_square.set_opacity(0.25) is style_square
assert np.isclose(style_square.get_fill_opacity(), 0.25)
assert np.isclose(style_square.get_stroke_opacity(), 0.25)
style_square.set_anti_alias_width(2.5)
assert np.isclose(style_square.get_anti_alias_width(), 2.5)
style_square.set_flat_stroke(True)
assert style_square.get_flat_stroke() is True
style_square.set_scale_stroke_with_zoom(True)
assert style_square.get_scale_stroke_with_zoom() is True
style_square.set_backstroke(width=2)
assert bool(style_square.uniforms["stroke_behind"]) is True
copied_style = manimlib.Square()
assert copied_style.match_style(style_square) is copied_style
assert np.isclose(copied_style.get_fill_opacity(), style_square.get_fill_opacity())
style_dict = style_square.get_style()
assert "fill_rgba" in style_dict and "stroke_width" in style_dict
faded = manimlib.Square().set_fill(manimlib.RED, opacity=1.0)
faded.fade(0.5)
assert np.isclose(faded.get_fill_opacity(), 0.5)
assert faded.get_group_class() is manimlib.VGroup
assert manimlib.VMobject.data_dtype[0][0] == "point"
assert "auto" in manimlib.VMobject.joint_type_map
assert manimlib.VMobject.pre_function_handle_to_anchor_scale_factor == 0.01
assert manimlib.Square().rotate(np.pi / 2) is not None
wide = manimlib.Square().stretch(2.0, 0)
plain = manimlib.Square()
assert wide.get_width() > plain.get_width()
depth_on = manimlib.Square().apply_depth_test()
assert bool(depth_on.uniforms["depth_test"]) is True
depth_off = manimlib.Square().apply_depth_test().deactivate_depth_test()
assert bool(depth_off.uniforms["depth_test"]) is False
try:
    manimlib.Square().replace_shader_code("old", "new")
    raise AssertionError("replace_shader_code did not refuse")
except NotImplementedError as error:
    assert "OOT-ARBITRARY-GLSL" in str(error)
try:
    manimlib.Square().init_shader_wrapper(None)
    raise AssertionError("init_shader_wrapper did not refuse")
except NotImplementedError as error:
    assert "OOT-ARBITRARY-GLSL" in str(error)


# ---------------------------------------------------------------------------
# fm-fnun: the remaining manimlib.utils.space_ops functions route through
# fmn-geom's deterministic authorities; pure shape arithmetic stays
# Reference-verbatim NumPy.

_quat = space_utils.quaternion_from_angle_axis(np.pi / 3, [0.0, 1.0, 0.0])
assert np.allclose(_quat, [0.0, np.sin(np.pi / 6), 0.0, np.cos(np.pi / 6)])
_ang, _axis = space_utils.angle_axis_from_quaternion(_quat)
assert np.isclose(_ang, np.pi / 3)
assert np.allclose(_axis, [0.0, 1.0, 0.0])
_double = space_utils.quaternion_mult(_quat, _quat)
_dang, _daxis = space_utils.angle_axis_from_quaternion(_double)
assert np.isclose(_dang, 2 * np.pi / 3) and np.allclose(_daxis, [0.0, 1.0, 0.0])
assert np.allclose(space_utils.quaternion_mult(), [0.0, 0.0, 0.0, 1.0])
assert np.allclose(
    space_utils.quaternion_conjugate([1.0, 2.0, 3.0, 4.0]), [-1.0, -2.0, -3.0, 4.0]
)

_rm = space_utils.rotation_matrix(np.pi / 2, [0.0, 0.0, 1.0])
assert np.allclose(_rm @ _rm.T, np.eye(3))
assert np.allclose(_rm @ np.array([1.0, 0.0, 0.0]), [0.0, 1.0, 0.0])
_rmt = space_utils.rotation_matrix_transpose(np.pi / 2, [0.0, 0.0, 1.0])
assert np.allclose(_rmt, _rm.T)
assert np.allclose(
    space_utils.rotation_about_z(np.pi / 2), [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]]
)
_qt = space_utils.rotation_matrix_transpose_from_quaternion(_quat)
_qm = space_utils.rotation_matrix_from_quaternion(_quat)
assert np.allclose(_qt, _qm.T) and np.allclose(_qt @ _qm, np.eye(3))
_rb = space_utils.rotation_between_vectors(np.array([1.0, 0.0, 0.0]), np.array([0.0, 1.0, 0.0]))
assert np.allclose(_rb @ np.array([1.0, 0.0, 0.0]), [0.0, 1.0, 0.0], atol=1e-7)
assert np.allclose(
    space_utils.rotation_between_vectors(np.array([1.0, 0.0, 0.0]), np.array([1.0, 0.0, 0.0])),
    np.eye(3),
)
assert np.allclose(space_utils.z_to_vector(np.array([0.0, 0.0, 1.0])), np.eye(3))

assert float(space_utils.cross2d(np.array([1.0, 2.0]), np.array([3.0, 4.0]))) == -2.0
assert np.allclose(
    space_utils.cross2d(np.array([[1.0, 2.0], [3.0, 4.0]]), np.array([[5.0, 6.0], [7.0, 8.0]])),
    [-4.0, -4.0],
)
assert space_utils.norm_squared([3, 4]) == 25
_poly = np.array([[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [3.0, 4.0, 0.0]])
assert np.isclose(space_utils.poly_line_length(_poly), 7.0)
assert np.allclose(space_utils.project_along_vector([1.0, 2.0, 3.0], [0.0, 0.0, 1.0]), [1.0, 2.0, 0.0])
assert np.allclose(space_utils.get_unit_normal([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0])
# The Reference degeneracy branch: aligned-with-z inputs fall back to DOWN.
assert np.allclose(space_utils.get_unit_normal([0.0, 0.0, 1.0], [0.0, 0.0, -1.0]), [0.0, -1.0, 0.0])

assert space_utils.tri_area([0, 0], [4, 0], [0, 3]) == 6.0
assert space_utils.tri_area(np.array([0.0, 0.0, 7.0]), np.array([4.0, 0.0, 7.0]), np.array([0.0, 3.0, 7.0])) == 6.0
assert space_utils.is_inside_triangle([1, 1], [0, 0], [4, 0], [0, 3]) is True
assert space_utils.is_inside_triangle([9, 9], [0, 0], [4, 0], [0, 3]) is False

_hit = space_utils.line_intersection(((0.0, 0.0, 0.0), (2.0, 2.0, 0.0)), ((0.0, 2.0, 0.0), (2.0, 0.0, 0.0)))
assert np.allclose(_hit, [1.0, 1.0, 0.0])
try:
    space_utils.line_intersection(((0.0, 0.0, 0.0), (1.0, 0.0, 0.0)), ((0.0, 1.0, 0.0), (1.0, 1.0, 0.0)))
    raise AssertionError("line_intersection accepted parallel lines")
except ValueError:
    pass

_fi = space_utils.find_intersection(
    np.array([1.0, 1.0, 0.0]), np.array([1.0, 0.0, 0.0]),
    np.array([0.0, 0.0, 0.0]), np.array([1.0, 1.0, 0.0]),
)
assert np.allclose(_fi, [1.0, 1.0, 0.0])
_fib = space_utils.find_intersection(
    np.array([[1.0, 1.0, 0.0], [-1.0, 1.0, 0.0]]),
    np.array([[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]]),
    np.zeros((2, 3)),
    np.array([[1.0, 1.0, 0.0], [-1.0, 1.0, 0.0]]),
)
assert np.allclose(_fib, [[1.0, 1.0, 0.0], [-1.0, 1.0, 0.0]])

_closest = space_utils.get_closest_point_on_line(np.array([4.0, 0.0]), np.array([0.0, 0.0]), np.array([2.0, 3.0]))
assert np.allclose(_closest, [2.0, 0.0])
_clamped = space_utils.get_closest_point_on_line(np.array([4.0, 0.0]), np.array([0.0, 0.0]), np.array([9.0, 0.0]))
assert np.allclose(_clamped, [4.0, 0.0])

_square = [np.array(p, dtype=float) for p in [(2, 0), (0, 2), (-2, 0), (0, -2)]]
_wound = space_utils.get_winding_number(_square)
assert abs(abs(_wound) - 1.0) < 1e-9, f"square winding number was {_wound!r}"
# An open path away from the origin sweeps zero net angle.
_open = space_utils.get_winding_number(
    [np.array(p, dtype=float) for p in [(2, 0), (0, 2)]]
)
assert _open == 0.0

_td = space_utils.thick_diagonal(4)
assert _td.dtype == np.uint8 and _td.shape == (4, 4) and _td[0].tolist() == [1, 1, 0, 0]

assert np.allclose(space_utils.complex_to_R3(3 + 4j), [3.0, 4.0, 0.0])
assert space_utils.R3_to_complex([3, 4, 9]) == 3 + 4j
assert np.isclose(space_utils.get_norm([3.0, 4.0, 0.0]), 5.0)
assert np.isclose(space_utils.get_dist([0.0, 0.0, 0.0], [3.0, 4.0, 0.0]), 5.0)
assert np.allclose(space_utils.normalize([0.0, 4.0, 0.0]), [0.0, 1.0, 0.0])
assert np.allclose(space_utils.normalize([0.0, 0.0, 0.0], fall_back=[1.0, 0.0, 0.0]), [1.0, 0.0, 0.0])
assert np.allclose(
    space_utils.normalize_along_axis(np.array([[3.0, 0.0], [0.0, 4.0]]), 1),
    [[1.0, 0.0], [0.0, 1.0]],
)
assert np.allclose(space_utils.center_of_mass([[0.0, 0.0, 0.0], [2.0, 4.0, 6.0]]), [1.0, 2.0, 3.0])
assert np.allclose(space_utils.midpoint([0.0, 0.0, 0.0], [2.0, 4.0, 6.0]), [1.0, 2.0, 3.0])
rotated = space_utils.rotate_vector([1.0, 0.0, 0.0], np.pi / 2, [0.0, 0.0, 1.0])
assert np.allclose(rotated, [0.0, 1.0, 0.0], atol=1e-7)
assert np.allclose(space_utils.rotate_vector_2d([1.0, 0.0], np.pi / 2), [0.0, 1.0], atol=1e-7)
# Reference space_ops leaked mapbox earcut and scipy Rotation; they refuse
# by the named OOT identities. Its operator/functools aliases are the
# stdlib objects, reachable through the star import as corpus scenes use
# them (_2020/chess.py: `reduce(op.xor, ...)`).
import functools as _bridge_functools
import operator as _bridge_operator
_star_namespace = {}
exec("from manimlib import *", _star_namespace)
assert space_utils.op is _bridge_operator and _star_namespace["op"] is _bridge_operator
assert space_utils.reduce is _bridge_functools.reduce
assert _star_namespace["reduce"] is _bridge_functools.reduce
assert _star_namespace["reduce"](_star_namespace["op"].xor, [3, 5, 6], 0) == 0
# Leaked submodule imports resolve as the Reference's import statements
# do: `import urllib.request` binds the top package, and `from PIL import
# Image` binds a submodule its package does not load on its own.
import urllib as _bridge_urllib
assert importlib.import_module("manimlib.utils.file_ops").urllib is _bridge_urllib
assert hasattr(_bridge_urllib, "request")
assert manimlib._resolve_leaked_origin("pulldom", "xml.dom.pulldom") is (
    sys.modules["xml.dom.pulldom"]
)
assert manimlib._resolve_leaked_origin("xml", "xml.dom.pulldom") is importlib.import_module("xml")
_camera_module = importlib.import_module("manimlib.camera.camera")
try:
    import PIL.Image as _bridge_pil_image
except ImportError:
    assert vars(_camera_module.Image)["_fmn_schema_placeholder"] is True
    for _use_image in (lambda: _camera_module.Image.open("x.png"), lambda: _camera_module.Image()):
        try:
            _use_image()
        except ModuleNotFoundError as error:
            assert error.name == "PIL" and "PIL.Image" in str(error), error
        else:
            raise AssertionError("a missing Pillow did not name itself")
else:
    assert _camera_module.Image is _bridge_pil_image
# matplotlib.pyplot is deliberately never imported; using it names the ruling.
_pyplot = importlib.import_module("manimlib.utils.color").pyplot
for _use_pyplot in (lambda: _pyplot.get_cmap("viridis"), lambda: _pyplot()):
    try:
        _use_pyplot()
    except NotImplementedError as error:
        assert "OOT-LEAKED-SURFACE-IMPORTS" in str(error), error
    else:
        raise AssertionError("the leaked pyplot was usable")
for leaked, oot in (
    ("Rotation", "OOT-LEAKED-SCIPY-IMPORT"),
    ("earcut", "OOT-LEAKED-EARCUT-IMPORT"),
):
    try:
        getattr(space_utils, leaked)()
    except NotImplementedError as error:
        assert oot in str(error), (leaked, error)
    else:
        raise AssertionError(f"{leaked} was callable without a named exclusion")
_cf = space_utils.complex_func_to_R3_func(lambda w: w * 1j)
assert np.allclose(_cf([1.0, 0.0, 5.0]), [0.0, 1.0, 0.0])

# earclip_triangulation: exact-cover validity over the governed native
# triangulator — square frame with a square hole.
_everts = np.array([
    [0.0, 0.0], [6.0, 0.0], [6.0, 6.0], [0.0, 6.0],
    [2.0, 2.0], [4.0, 2.0], [4.0, 4.0], [2.0, 4.0],
])
_eidx = space_utils.earclip_triangulation(_everts, [4, 8])
_etris = [tuple(_eidx[k:k + 3]) for k in range(0, len(_eidx), 3)]
_earea = sum(
    abs(space_utils.cross2d(_everts[list(t)][1] - _everts[list(t)][0], _everts[list(t)][2] - _everts[list(t)][0])) / 2
    for t in _etris
)
assert all(i in range(8) for i in _eidx)
assert np.isclose(_earea, 36.0 - 4.0), _earea


# ---------------------------------------------------------------------------
# fm-5wq.4: the svg_mobject surface — construction, seams, and module fns.

_svg_multi = (
    '<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">'
    '<rect x="10" y="10" width="30" height="20" fill="#FF0000"/>'
    '<circle cx="70" cy="25" r="10"/>'
    '<line x1="10" y1="60" x2="50" y2="60" stroke="#0000FF"/>'
    '<polygon points="60,50 90,50 75,80" fill="#FFFF00"/>'
    '<polyline points="10,90 40,85 70,95" stroke="#FFFFFF" fill="none"/>'
    '<path d="M 5 5 L 8 5 Q 11 2 14 5"/>'
    "</svg>"
)
_svg_family = manimlib.SVGMobject(svg_string=_svg_multi)

# One authored child per rendered shape in document order; the default
# class height is 2.0 with the family centered.
assert len(_svg_family.submobjects) == 6
assert _svg_family.get_height() == 2.0
assert np.allclose(_svg_family.get_center(), [0.0, 0.0, 0.0])
_rect_child, _circle_child, _line_child, _poly_child, _pl_child, _path_child = (
    _svg_family.submobjects
)
assert _rect_child.get_fill_color() == "#FF0000"
assert abs(_rect_child.get_fill_opacity() - 1.0) < 1e-6
assert np.isclose(
    _rect_child.get_width() / _rect_child.get_height(), 30.0 / 20.0
)
assert np.isclose(_circle_child.get_width(), _circle_child.get_height())
assert _poly_child.get_fill_color() == "#FFFF00"
assert abs(_pl_child.get_fill_opacity() - 0.0) < 1e-6

# Style overrides win over the document's own colors.
_styled = manimlib.SVGMobject(svg_string=_svg_multi, color=manimlib.BLUE)
_styled_rect = next(
    child
    for child in _styled.submobjects
    if abs(child.get_width() / max(child.get_height(), 1e-9) - 1.5) < 0.05
)
assert _styled_rect.get_fill_color() == "#58C4DD"

# hash_seed captures exactly the construction inputs.
_same_seed = manimlib.SVGMobject(svg_string=_svg_multi).hash_seed
assert _same_seed == _svg_family.hash_seed
assert _same_seed[0] == "SVGMobject"
assert _same_seed[3] == _svg_multi
_other_seed = manimlib.SVGMobject(svg_string=_svg_multi.replace("30", "31")).hash_seed
assert _other_seed != _same_seed

# mobjects_from_svg_string returns a detached family in document order.
_detached = _svg_family.mobjects_from_svg_string(_svg_multi)
assert len(_detached) == 6
assert all(child not in _svg_family.submobjects for child in _detached)

# file_name_to_svg_string routes through the governed reader: content or a
# typed refusal, never a silent substitute.
_read_back = _svg_family.file_name_to_svg_string(
    str(pathlib.Path(__file__).with_name("chisel_sample.svg"))
)
assert "<svg" in _read_back
try:
    _svg_family.file_name_to_svg_string("/nonexistent/definitely_absent.svg")
except OSError:
    pass
else:
    raise AssertionError("missing SVG source must refuse loudly")

# generate_config_style_dict mirrors the Reference's seed-to-attr map;
# with every seed None the dict is empty.
assert _svg_family.generate_config_style_dict() == {}

# text_to_mobject is the Reference's explicit no-op.
assert _svg_family.text_to_mobject(None) is None

# init_svg_mobject rebuilds and re-attaches the family (Reference
# semantics; calling it twice doubles the children).
_before = len(_svg_family.submobjects)
_svg_family.init_svg_mobject()
assert len(_svg_family.submobjects) == 2 * _before

# VMobjectFromSVGPath accepts any object exposing the path data.
_duck_path = manimlib.VMobjectFromSVGPath(type("_D", (), {"d": lambda self: "M 0 0 L 40 0"})())
assert _duck_path.get_num_points() > 0

# fm-5wq.11: the svgelements re-parse hooks refuse by name; they no longer
# return silently while doing nothing (OOT-SVGELEMENTS-PARSER).
for _hook in ("init_points", "handle_commands", "handle_arc"):
    try:
        getattr(_duck_path, _hook)()
    except NotImplementedError as error:
        assert "OOT-SVGELEMENTS-PARSER" in str(error) and _hook in str(error)
    else:
        raise AssertionError(f"VMobjectFromSVGPath.{_hook} returned silently")
assert _duck_path.get_num_points() > 0

# fm-5wq.11: the render-twice labelling internals refuse by name instead of
# returning plausible empties; the kept surface still answers.
_labelled_tex = manimlib.Tex("x^2")
_labelled_text = manimlib.Text("ab")
for _name, _call in {
    "get_command_matches": lambda: manimlib.Tex.get_command_matches("x^2"),
    "replace_for_content": lambda: manimlib.MarkupText.replace_for_content(None),
    "get_command_flag": lambda: manimlib.Text.get_command_flag(None),
    "parse": _labelled_tex.parse,
    "get_svg_string": _labelled_tex.get_svg_string,
    "build_groups": _labelled_text.build_groups,
    "get_content_prefix_and_suffix": _labelled_text.get_content_prefix_and_suffix,
    "get_symbol_substrings": _labelled_tex.get_symbol_substrings,
}.items():
    try:
        _call()
    except NotImplementedError as error:
        assert "OOT-STRING-RENDER-TWICE-INTERNALS" in str(error), error
        assert _name in str(error), error
    else:
        raise AssertionError(f"StringMobject.{_name} returned a silent value")
assert _labelled_tex.get_content() == "x^2"
try:
    importlib.import_module("manimlib.mobject.svg.old_tex_mobject").SingleStringTex \
        .get_svg_string_by_content(None, "x")
except NotImplementedError as error:
    assert "OOT-TEX-LATEX-PIPELINE" in str(error)
else:
    raise AssertionError("SingleStringTex.get_svg_string_by_content returned silently")

# fm-5wq.11: StringMobject matching's re-exported solvers are real. The
# fsci-opt assignment must equal brute force (rectangular, maximize), and
# cdist must equal each metric's closed form. The replaced stand-in returned
# the identity assignment, so the anti-identity matrix below is its witness.
_sm_module = importlib.import_module("manimlib.mobject.svg.string_mobject")
_lsa, _cdist = _sm_module.linear_sum_assignment, _sm_module.cdist
_rng = np.random.default_rng(7)
for _shape in ((1, 1), (3, 3), (4, 4), (3, 5), (5, 3), (2, 6)):
    for _maximize in (False, True):
        _cost = _rng.integers(-9, 20, size=_shape).astype(float)
        _rows, _cols = _lsa(_cost, maximize=_maximize)
        assert list(_rows) == sorted(_rows) and len(set(_cols)) == len(_cols) == min(_shape)
        _n, _m = _shape
        _totals = [
            sum(_cost[i, j] for i, j in zip(range(_n), _perm))
            if _n <= _m
            else sum(_cost[i, j] for i, j in zip(_perm, range(_m)))
            for _perm in itertools.permutations(range(max(_shape)), min(_shape))
        ]
        assert _cost[_rows, _cols].sum() == (max(_totals) if _maximize else min(_totals))
_anti = np.array([[9.0, 1.0, 9.0], [9.0, 9.0, 1.0], [1.0, 9.0, 9.0]])
assert list(_lsa(_anti)[1]) == [1, 2, 0]
for _bad in (np.array([[np.nan, 1.0]]), np.zeros((2, 2, 2))):
    try:
        _lsa(_bad)
    except ValueError:
        pass
    else:
        raise AssertionError(f"linear_sum_assignment accepted invalid cost {_bad.shape}")
_a, _b = _rng.normal(size=(4, 3)), _rng.normal(size=(5, 3))
_diff = _a[:, None, :] - _b[None, :, :]
for _metric, _want in {
    "euclidean": np.sqrt((_diff ** 2).sum(-1)),
    "sqeuclidean": (_diff ** 2).sum(-1),
    "cityblock": np.abs(_diff).sum(-1),
    "chebyshev": np.abs(_diff).max(-1),
    "cosine": 1 - (_a @ _b.T) / np.outer(np.linalg.norm(_a, axis=1), np.linalg.norm(_b, axis=1)),
}.items():
    assert np.allclose(_cdist(_a, _b, _metric), _want, rtol=0, atol=1e-12), _metric
assert np.allclose(_cdist(_a, _b, "minkowski", p=3), (np.abs(_diff) ** 3).sum(-1) ** (1 / 3))
try:
    _cdist(_a, _b, "hamming")
except NotImplementedError as error:
    assert "hamming" in str(error)
else:
    raise AssertionError("cdist silently accepted an unsupported metric")

# fm-5wq.14: Reference utils helpers real scenes call. merge_dicts_recursively
# lets later dicts win and merges dict values; the random colours draw from
# the Scene-seeded numpy and stdlib RNGs as the Reference does; find_file
# searches locally, raises the Reference's "not Found", and refuses URLs
# toward the host AssetFetcher.
import random as _stdlib_random  # noqa: E402

_dict_ops = importlib.import_module("manimlib.utils.dict_ops")
_color_utils = importlib.import_module("manimlib.utils.color")
_file_ops = importlib.import_module("manimlib.utils.file_ops")
_image_utils = importlib.import_module("manimlib.utils.images")
assert _dict_ops.merge_dicts_recursively(
    {"a": 1, "d": {"x": 1, "y": 2}}, {"b": 2, "d": {"y": 3}}
) == {"a": 1, "d": {"x": 1, "y": 3}, "b": 2}
_draws = []
for _ in range(2):
    _stdlib_random.seed(4)
    np.random.seed(4)
    _draws.append((_color_utils.random_color().get_rgb(), _color_utils.random_bright_color().get_rgb()))
assert _draws[0] == _draws[1]
assert manimlib.Square().set_color(_color_utils.random_bright_color()).get_color()
with tempfile.TemporaryDirectory() as _asset_root:
    pathlib.Path(_asset_root, "pic.png").write_bytes(b"x")
    assert _file_ops.find_file("pic", [_asset_root], [".jpg", ".png"]).name == "pic.png"
    try:
        _file_ops.find_file("absent", [_asset_root])
    except OSError as error:
        assert str(error) == "absent not Found"
    else:
        raise AssertionError("find_file returned for a missing file")
try:
    _file_ops.find_file("https://example.invalid/a.png")
except bridge_errors.CapabilityError as error:
    assert "AssetFetcher" in str(error)
else:
    raise AssertionError("find_file accepted a URL")
assert manimlib.merge_dicts_recursively is _dict_ops.merge_dicts_recursively
assert manimlib.get_full_raster_image_path is _image_utils.get_full_raster_image_path

# Reference utils/debug.py and VMobject.use_winding_fill, which corpus scenes
# call (moser_reboot, gauss_int, antp). index_labels puts one Integer per
# submobject, centred on it with a black backstroke; print_family logs the
# family at debug level; use_winding_fill is the Reference's no-op.
import logging as _logging  # noqa: E402

_debug = importlib.import_module("manimlib.utils.debug")
_row = manimlib.VGroup(*(manimlib.Square().shift(3 * i * manimlib.RIGHT) for i in range(3)))
_labels = _debug.index_labels(_row, label_height=0.2)
assert isinstance(_labels, manimlib.VGroup)
assert [label.get_value() for label in _labels] == [0, 1, 2]
for _label, _submob in zip(_labels, _row):
    assert np.allclose(_label.get_center(), _submob.get_center())
    assert np.isclose(_label.get_height(), 0.2)
    for _leaf in _label.family_members_with_points():
        assert np.isclose(_leaf.get_stroke_width(), 5)
        assert np.allclose(_color_utils.color_to_rgb(_leaf.get_stroke_color()), 0)
        assert bool(_leaf.uniforms["stroke_behind"])
assert manimlib.index_labels is _debug.index_labels


class _CollectLog(_logging.Handler):
    def __init__(self):
        super().__init__(_logging.DEBUG)
        self.messages = []

    def emit(self, record):
        self.messages.append(record.getMessage())


_manimgl_log, _collect = _logging.getLogger("manimgl"), _CollectLog()
_saved_level = _manimgl_log.level
_manimgl_log.addHandler(_collect)
_manimgl_log.setLevel(_logging.DEBUG)
try:
    _debug.print_family(_row)
finally:
    _manimgl_log.removeHandler(_collect)
    _manimgl_log.setLevel(_saved_level)
assert len(_collect.messages) == 4
assert _collect.messages[0].endswith(" " + str(id(_row)))
assert all(message.startswith("\t") and not message.startswith("\t\t")
           for message in _collect.messages[1:])
assert _collect.messages[2].endswith(" " + str(id(_row[1])))

_winding = manimlib.VGroup(manimlib.Circle(), manimlib.Square())
_winding_data = [member.data.copy() for member in _winding.get_family()]
assert _winding.use_winding_fill(False) is _winding
assert not any(bool(member.uniforms["use_winding_fill"]) for member in _winding.get_family())
_winding.use_winding_fill(True, recurse=False)
assert bool(_winding.uniforms["use_winding_fill"])
assert not bool(_winding[0].uniforms["use_winding_fill"])
for _member, _data in zip(_winding.get_family(), _winding_data):
    np.testing.assert_array_equal(_member.data, _data)

# fm-5wq.14: Reference color_to_rgb takes colour.Color, i.e. anything with
# get_rgb(); both the Python and the native colour parsers accept it (54
# corpus scenes pass Color objects). The re-exported Color names its missing
# package instead of posing as an unbound portal stub.
class _DuckColor:
    def get_rgb(self):
        return (0.2, 0.4, 0.6)


assert manimlib.Square().set_color(_DuckColor()).get_color().lower() == "#336699"
assert manimlib.Dot(color=_DuckColor()).get_color().lower() == "#336699"
try:
    importlib.import_module("colour")
except ImportError:
    try:
        manimlib.Color("red")
    except ModuleNotFoundError as error:
        assert error.name == "colour" and "colour.Color" in str(error)
    else:
        raise AssertionError("Color without the colour package returned silently")
else:
    assert manimlib.Color("red").get_rgb() == (1.0, 0.0, 0.0)

# fm-5wq.14: coordinate-system axes are the Reference's NumberLines, not bare
# shells. 29 corpus scenes call axes.x_axis.get_unit_size() or add_numbers().
_axes = manimlib.Axes(x_range=(-3, 3, 1), y_range=(-2, 2, 1), width=6, height=4)
assert type(_axes.x_axis) is manimlib.NumberLine and type(_axes.y_axis) is manimlib.NumberLine
assert math.isclose(_axes.x_axis.get_unit_size(), 1.0) and math.isclose(_axes.y_axis.get_unit_size(), 1.0)
assert np.allclose(_axes.x_axis.n2p(2), _axes.c2p(2, 0))
assert math.isclose(_axes.x_axis.p2n(_axes.c2p(1.5, 0)), 1.5)
assert np.allclose(_axes.y_axis.line_to_number_direction, manimlib.LEFT)
_axes.x_axis.add_numbers()
assert len(_axes.x_axis.numbers) == 7
assert type(_axes.copy().x_axis) is manimlib.NumberLine
_plane = manimlib.ComplexPlane()
assert type(_plane.x_axis) is manimlib.NumberLine and not _plane.x_axis.include_ticks
assert type(manimlib.ThreeDAxes().z_axis) is manimlib.NumberLine
_scaled = manimlib.Axes(x_range=(0, 10, 1), width=5)
assert math.isclose(_scaled.x_axis.get_unit_size(), 0.5)
# Reference NumberLine accepts VMobject style keywords through axis_config
# (21 corpus scenes pass stroke_opacity); its ticks match the line.
_faint = manimlib.ThreeDAxes(axis_config=dict(stroke_opacity=0.7))
# Records store float32, so compare at float32 resolution.
assert all(
    math.isclose(float(axis.get_stroke_opacity()), 0.7, abs_tol=1e-6) for axis in _faint.axes
)
assert math.isclose(float(_faint.x_axis.ticks[0].get_stroke_opacity()), 0.7, abs_tol=1e-6)

# fm-5wq.14: every natively built surface carries Reference Surface.__init__'s
# grid triangles. 16 corpus scenes read sphere.triangle_indices or sort
# faces back to front.
for _surface in (
    manimlib.ParametricSurface(lambda u, v: [u, v, 0]), manimlib.Sphere(),
    manimlib.Torus(), manimlib.Cylinder(), manimlib.Cone(),
    manimlib.Line3D(manimlib.ORIGIN, manimlib.RIGHT), manimlib.Disk3D(),
    manimlib.Square3D(),
):
    _nu, _nv = _surface.resolution
    assert len(_surface.triangle_indices) == 6 * (_nu - 1) * (_nv - 1), type(_surface)
    assert _surface.triangle_indices is _surface.get_triangle_indices()
_sphere = manimlib.Sphere()
_unsorted = sorted(_sphere.triangle_indices.tolist())
assert _sphere.sort_faces_back_to_front() is _sphere
assert sorted(_sphere.triangle_indices.tolist()) == _unsorted
assert [len(face.triangle_indices) for face in manimlib.Cube()] == [6] * 6
assert manimlib.Cube(square_resolution=(3, 3))[0].triangle_indices.size == 24
assert [len(face.triangle_indices) for face in manimlib.Prism()] == [6] * 6

# fm-5wq.14: Reference OldTex splits each string at every isolate and
# tex_to_color_map key, one part per piece (23 corpus scenes index eq[2]).
_old_tex_module = importlib.import_module("manimlib.mobject.svg.old_tex_mobject")
_split = _old_tex_module.OldTexText(
    "Prevalence = Prior", tex_to_color_map={"Prevalence": manimlib.WHITE, "Prior": manimlib.YELLOW}
)
assert _split.tex_strings == ["Prevalence", " = ", "Prior"] and len(_split) == 3
assert all(glyph.get_fill_color().upper() == "#FFFF00" for glyph in _split[2])
assert _old_tex_module.OldTex("x^2 + y", isolate=["y"]).tex_strings == ["x^2 + ", "y"]
# C-19 (BN-07): a caller's color= beats constructor defaults. The pinned
# Reference re-applies Text's white default and Dot's white fill, and drops it.
_red_words = manimlib.Text("ab", color=manimlib.RED)
assert all(glyph.get_fill_color().upper() == "#FC6255" for glyph in _red_words), \
    [glyph.get_fill_color() for glyph in _red_words]
assert manimlib.Dot(color=manimlib.RED).get_fill_color().upper() == "#FC6255"
# fm-qead: explicit fill_color/stroke_color > color > constructor default, the
# Reference VMobject's `fill_color or color`. The first two rows are the
# pinned Reference's own values; the Dot and Brace rows keep C-19's color win
# over their constructor defaults while an explicit channel still wins.
for _made, _fill, _stroke in (
    (manimlib.Square(color=manimlib.RED, fill_color=manimlib.BLUE), "#58C4DD", "#FC6255"),
    (manimlib.Square(color=manimlib.RED, stroke_color=manimlib.GREEN), "#FC6255", "#83C167"),
    (manimlib.Circle(color=manimlib.RED, fill_color=manimlib.BLUE), "#58C4DD", "#FC6255"),
    (manimlib.Dot(color=manimlib.RED, fill_color=manimlib.BLUE), "#58C4DD", "#FC6255"),
    (manimlib.Dot(fill_color=manimlib.BLUE), "#58C4DD", "#000000"),
    (manimlib.Brace(manimlib.Square(), color=manimlib.RED), "#FC6255", "#FC6255"),
):
    assert (_made.get_fill_color().upper(), _made.get_stroke_color().upper()) == (_fill, _stroke), \
        (type(_made).__name__, _made.get_fill_color(), _made.get_stroke_color())
# A resolved default is a plain str: the marker never reaches copies.
assert type(manimlib.Dot().fill_color) is str


# fm-qead incident (2026-09-29): an installer that re-defines a constructor
# must mark its channel defaults too. Unmarked, they masked color= again on
# Arrow, Vector, StrokeArrow, VCube, VPrism, Dodecahedron and
# SingleStringTex. color= colors every visible channel of a plain class; the
# composites below color only some members, as before fm-qead.
def _c19_unmasked(made):
    red = manimlib.RED.upper()
    for member in made.family_members_with_points():
        data = member.data
        for key in ("fill_rgba", "stroke_rgba"):
            if key in data.dtype.names and len(data) and float(data[key][0][3]) > 0:
                hex_color = manimlib.rgb_to_hex(data[key][0][:3]).upper()
                if hex_color != red:
                    return f"{type(member).__name__} {key} {hex_color}"
    return None


for _made in (
    manimlib.Arrow(manimlib.LEFT, manimlib.RIGHT, color=manimlib.RED),
    manimlib.Vector(manimlib.RIGHT + manimlib.UP, color=manimlib.RED),
    manimlib.StrokeArrow(manimlib.LEFT, manimlib.RIGHT, color=manimlib.RED),
    manimlib.VCube(color=manimlib.RED),
    manimlib.VPrism(color=manimlib.RED),
    manimlib.Dodecahedron(color=manimlib.RED),
    _old_tex_module.SingleStringTex("x", color=manimlib.RED),
):
    assert _c19_unmasked(_made) is None, (type(_made).__name__, _c19_unmasked(_made))
_c19_composites = {
    "Axes", "Clock", "ComplexPlane", "Dartboard", "DoubleSpeechBubble",
    "FullScreenFadeRectangle", "Laptop", "NumberPlane", "Piano", "Piano3D",
    "SpeechBubble", "Speedometer", "ThoughtBubble", "ThreeDAxes", "Title",
}
_c19_checked = 0
for _name in sorted(dir(manimlib)):
    _cls = getattr(manimlib, _name)
    if (_name.startswith("_") or _name in _c19_composites or not isinstance(_cls, type)
            or not issubclass(_cls, manimlib.VMobject)):
        continue
    try:
        _made = _cls(color=manimlib.RED)
    except Exception:
        continue  # needs arguments; the sweep-call rows above cover those at risk
    assert _c19_unmasked(_made) is None, (_name, _c19_unmasked(_made))
    _c19_checked += 1
assert _c19_checked >= 40, _c19_checked
assert len(_old_tex_module.OldTex("a", "+", "b")) == 3
# OldTex selects parts the way the Reference does (old_tex_mobject.py:248-305).
# Each part carries its piece's source. A lookup tests substrings of that
# source, a miss is None rather than an IndexError, and slice_by_tex stops
# before the stop part. The formula is shadows.py's get_key_result verbatim.
_key = _old_tex_module.OldTex(
    "\\text{Area}\\big(\\text{Shadow}(\\text{Solid})\\big)",
    "=",
    "\\frac{1}{2}", "{c}", "\\cdot",
    "(\\text{Surface area})",
    tex_to_color_map={
        "\\text{Shadow}": manimlib.GREY_B,
        "\\text{Solid}": manimlib.BLUE,
        "{c}": manimlib.RED,
    },
)
assert [part.tex_string for part in _key] == [
    "\\text{Area}\\big(", "\\text{Shadow}", "(", "\\text{Solid}", ")\\big)",
    "=", "\\frac{1}{2}", "{c}", "\\cdot", "(\\text{Surface area})",
]
assert _key.get_part_by_tex("frac") is _key[6]
assert _key[6].get_tex() == "\\frac{1}{2}"
assert _key.get_part_by_tex("Solid") is _key[3]
assert _key.get_part_by_tex("\\left") is None
assert len(_key.get_parts_by_tex("text")) == 4
assert len(_key.get_parts_by_tex("TEXT", case_sensitive=False)) == 4
assert len(_key.get_parts_by_tex("text", substring=False)) == 0
assert _key.index_of_part_by_tex("{c}") == 7
assert list(_key.slice_by_tex(None, "=")) == list(_key[:5])
assert list(_key.slice_by_tex("=", "\\cdot")) == list(_key[5:8])
assert list(_key.slice_by_tex("=")) == list(_key[5:])
assert {glyph.get_fill_color().upper() for glyph in _key[7]} == {"#FC6255"}
assert _key.copy().get_part_by_tex("frac").tex_string == "\\frac{1}{2}"
# A piece that draws nothing is dropped from the parts, as the Reference
# drops it, so eq[i] counts the same parts. It is never refused.
_spaced = _old_tex_module.OldTex("a", "\\,", "b", "{}", "c")
assert [part.tex_string for part in _spaced] == ["a", "b", "c"]
assert _spaced.get_part_by_tex("c") is _spaced[2]
_average = _old_tex_module.OldTex("\\frac{1}{n}", "\\left(", "x", "\\right)")
assert _average.get_part_by_tex("\\left") is _average[1]
assert len(_average.get_part_by_tex("\\right").family_members_with_points()) == 1

# fm-5wq.14: isolate follows the Reference parse's labelling rules
# (string_mobject.py:210). It skips an occurrence inside a command token
# (the "s" in \sqrt) and a lone brace, and whitespace draws nothing, so none
# of these errors.
_isolated_s = manimlib.Tex(r"{s} = {-b \pm \sqrt{s}}", isolate=["s"])
assert len(_isolated_s.select_parts("s").family_members_with_points()) == 2
# Isolation regroups glyphs into parts, so compare the geometry as a multiset.
assert sorted(map(tuple, np.round(_isolated_s.get_all_points(), 6))) == sorted(
    map(tuple, np.round(manimlib.Tex(r"{s} = {-b \pm \sqrt{s}}").get_all_points(), 6))
)
manimlib.Tex("{x}+{y}", isolate=["}"])
# The Reference skips an isolate whose braces do not balance inside it (the
# "Cannot handle substrings" warning). It labels inkless source such as a
# spacing command or the line break, but draws nothing for it. Neither
# case is refused. Sources from matrix_exp.py, quick_eigen.py and subsets.py.
for _source, _isolate in (
    (r"{d \over dt}{e}^{rt}=r\cdot{e}^{rt}", "^{"),
    (r"\lambda_1, \lambda_2 \,=\, \text{roots}", r" \,"),
    (r"z^5 &= 1 \\ z &= \sqrt[5]{1}", "\\\\ "),
):
    assert sorted(map(tuple, np.round(
        manimlib.Tex(_source, isolate=[_isolate]).get_all_points(), 6
    ))) == sorted(map(tuple, np.round(manimlib.Tex(_source).get_all_points(), 6)))
assert len(_old_tex_module.OldTex("a", "\n   ", "b").family_members_with_points()) == 2
# fm-5wq.22: a command keyword selects the ink the command draws, as the
# Reference's select_unisolated_substring does: \over the fraction bar,
# \sqrt the radical sign and vinculum. Isolating \over gives the bar its own
# part, and argument glyphs never join it.
_over = manimlib.Tex(r"{a \over b}", isolate=[r"\over"])
_bar = _over[r"\over"].family_members_with_points()
assert len(_bar) == 1 and _bar[0].get_height() < 0.1 < _bar[0].get_width()
assert len(_over.submobjects) == 3
_root = manimlib.Tex(r"x = \sqrt{c}")
assert len(_root[r"\sqrt"].family_members_with_points()) == 2
assert len(_root["c"].family_members_with_points()) == 1
_nested = manimlib.Tex(r"\frac{\sqrt{x}}{2}")
assert len(_nested[r"\frac"].family_members_with_points()) == 1
assert len(_nested[r"\sqrt"].family_members_with_points()) == 2


# fm-5wq.14: TexText's `alignment` (tex_mobject.py:189). The default
# \centering centers every line; "" and \raggedright leave them flush left.
def _first_line_offsets(tex):
    # (first line's left edge, center, right edge) relative to the block's;
    # glyphs are in source order, so "ab" is the first two.
    first = tex.family_members_with_points()[:2]
    left = min(glyph.get_left()[0] for glyph in first)
    right = max(glyph.get_right()[0] for glyph in first)
    return (
        left - tex.get_left()[0],
        (left + right) / 2 - tex.get_center()[0],
        right - tex.get_right()[0],
    )


_two_lines = r"ab\\a much longer second line"
_left_gap, _centre_gap, _ = _first_line_offsets(manimlib.TexText(_two_lines))
# Lines align by advance box, so ink side bearings leave ~0.002 residues.
assert _left_gap > 1.0 and abs(_centre_gap) < 0.01, (_left_gap, _centre_gap)
for _flush in ("", None, r"\raggedright", r"\flushleft"):
    _left_gap, _centre_gap, _ = _first_line_offsets(manimlib.TexText(_two_lines, alignment=_flush))
    assert abs(_left_gap) < 1e-3 and _centre_gap < -1.0, (_flush, _left_gap, _centre_gap)
_left_gap, _, _right_gap = _first_line_offsets(manimlib.TexText(_two_lines, alignment=r"\raggedleft"))
assert _left_gap > 1.0 and abs(_right_gap) < 0.01, (_left_gap, _right_gap)
_left_gap, _, _ = _first_line_offsets(_old_tex_module.OldTexText(_two_lines, alignment=""))
assert abs(_left_gap) < 1e-3
_math_default, _math_empty = manimlib.Tex("x^2"), manimlib.Tex("x^2", alignment="")
assert np.allclose(_math_default.get_all_points(), _math_empty.get_all_points())
_colored = manimlib.TexText(_two_lines, t2c={"ab": manimlib.YELLOW})
assert [g.get_fill_color().upper() for g in _colored.family_members_with_points()[:2]] == ["#FFFF00"] * 2
assert all(g.get_fill_color().upper() != "#FFFF00" for g in _colored.family_members_with_points()[2:])
try:
    manimlib.TexText(_two_lines, alignment=r"\justifying")
except NotImplementedError as error:
    assert "justifying" in str(error), error
else:
    raise AssertionError("an unsupported alignment declaration was accepted")

# fm-5wq.14: Reference Scene.remove removes the family of every argument
# (25 corpus scenes remove never-added groups of added members); an absent
# mobject is a no-op; a partly removed group dissolves into its survivors.
_remove_scene = manimlib.Scene()
_ra, _rb, _rc = manimlib.Dot(), manimlib.Dot(), manimlib.Dot()
_never_added = manimlib.VGroup(_ra, _rb)
_remove_scene.add(manimlib.VGroup(_ra, _rb, _rc))
_remove_scene.remove(_never_added)
assert len(_remove_scene.mobjects) == 1 and _remove_scene.mobjects[0] is _rc
_remove_scene.remove(manimlib.Square())
assert len(_remove_scene.mobjects) == 1
_foreign_scene = manimlib.Scene()
_foreign = manimlib.Dot()
_foreign_scene.add(_foreign)
try:
    _remove_scene.remove(_foreign)
except bridge_errors.ForeignStageError:
    pass
else:
    raise AssertionError("removing another Scene's mobject was accepted")

# fm-5wq.14: copying a never-added group whose descendants were added one by
# one (3b1b's sir.py builds exp_tree this way, as do 40 other corpus scenes)
# yields a detached, independent copy of every member.
_copy_scene = manimlib.Scene()
_rows = manimlib.VGroup(
    manimlib.VGroup(manimlib.Dot(), manimlib.Dot().shift(manimlib.RIGHT)),
    manimlib.VGroup(manimlib.Dot()),
)
_copy_scene.add(_rows[0])
_rows[0][1].set_color(manimlib.RED)
_tree = manimlib.VGroup(_rows, manimlib.VGroup(manimlib.Line()))
assert not _tree._is_bound() and _rows[0]._is_bound()
_tree_copy = _tree.copy()
assert not any(member._is_bound() for member in _tree_copy.get_family())
assert len(_tree_copy.get_family()) == len(_tree.get_family())
for _copied, _source in zip(_tree_copy.get_family(), _tree.get_family()):
    assert _copied is not _source and np.allclose(_copied.get_points(), _source.get_points())
assert _tree_copy[0][0][1].get_color() == _tree[0][0][1].get_color()
_tree_copy[0][0][1].shift(manimlib.UP)
assert not np.allclose(_tree_copy[0][0][1].get_center(), _rows[0][1].get_center())
assert _rows[0][1]._is_bound()
_copy_scene.add(_tree_copy)
assert _tree_copy._is_bound()

# fm-5wq.14: manimlib.utils.directories serves Reference directories.py over
# manim_config.directories. The native reader merges custom_config.yml
# recursively over the defaults (3b1b's corpus depends on this), and each
# subdir hangs from the base.
_native_root = getattr(manimlib, "_resolved_directories", None) and manimlib
_native_root = _native_root or importlib.import_module("manimlib.manimlib")
_resolved = _native_root._resolved_directories(
    "directories:\n  base: /b\n  subdirs:\n    data: d\n    pi_creature_images: p/svg\n"
)
assert _resolved["base"] == "/b" and _resolved["mirror_module_path"] is False
assert _resolved["subdirs"]["data"] == "d" and _resolved["subdirs"]["output"] == "videos"
assert _resolved["subdirs"]["pi_creature_images"] == "p/svg"
_dirs_module = importlib.import_module("manimlib.utils.directories")
# The top-level star surface re-exports the bound helpers, not stale stubs.
assert manimlib.get_directories is _dirs_module.get_directories
assert manimlib.get_output_dir is _dirs_module.get_output_dir
_dirs = _dirs_module.get_directories()
assert _dirs is importlib.import_module("manimlib.config").manim_config.directories
for _key, _subdir in _dirs["subdirs"].items():
    assert _dirs[_key] == os.path.join(_dirs["base"], _subdir), _key
assert _dirs_module.get_sound_dir() == _dirs["sounds"]
assert _dirs_module.get_temp_dir() == (_dirs.get("temporary_storage") or tempfile.gettempdir())
assert _dirs_module.get_cache_dir().endswith("manim")
_cwd = os.getcwd()
with tempfile.TemporaryDirectory() as _scratch:
    os.chdir(_scratch)
    try:
        _made = _dirs_module.get_output_dir()
        assert isinstance(_made, pathlib.Path) and _made.is_dir() and _made.is_absolute()
    finally:
        os.chdir(_cwd)

# fm-5wq.11: manimlib.config's CLI helpers follow Reference config.py:350-395
# over the same Namespace fields instead of ignoring their arguments.
_cfg = importlib.import_module("manimlib.config")
_ns = types.SimpleNamespace
assert _cfg.get_animations_numbers(_ns(start_at_animation_number=None)) == (None, None)
assert _cfg.get_animations_numbers(_ns(start_at_animation_number="3")) == (3, None)
assert _cfg.get_animations_numbers(_ns(start_at_animation_number="2,5")) == (2, 5)
assert _cfg.get_file_ext(_ns(transparent=True, gif=True)) == ".mov"
assert _cfg.get_file_ext(_ns(transparent=False, gif=True)) == ".gif"
assert _cfg.get_file_ext(_ns(transparent=False, gif=False)) == ".mp4"
_flags = dict(resolution=None, low_quality=False, medium_quality=False, hd=False, uhd=False)
_options = {"low": "(854, 480)", "med": "(1280, 720)", "high": "(1920, 1080)", "4k": "(3840, 2160)"}
assert _cfg.get_resolution_from_args(_ns(**_flags), _options) is None
assert _cfg.get_resolution_from_args(_ns(**{**_flags, "resolution": "640x360"}), _options) == (640, 360)
assert _cfg.get_resolution_from_args(_ns(**{**_flags, "medium_quality": True}), _options) == (1280, 720)
assert _cfg.get_resolution_from_args(_ns(**{**_flags, "uhd": True}), _options) == (3840, 2160)
_dirs = _ns(output="/out", mirror_module_path=False, removed_mirror_prefix="")
assert _cfg.get_output_directory(_ns(video_dir=None, file="s.py"), _ns(directories=_dirs)) == "/out"
assert _cfg.get_output_directory(_ns(video_dir="/v", file="s.py"), _ns(directories=_dirs)) == "/v"
_mirror = _ns(output="/out", mirror_module_path=True, removed_mirror_prefix="/nonmatching")
assert _cfg.get_output_directory(
    _ns(video_dir=None, file="/proj/_2020/scene.py"), _ns(directories=_mirror)
) == pathlib.Path("/out/scene")

# Module functions and constants carry the Reference identities.
_svg_module = importlib.import_module("manimlib.mobject.svg.svg_mobject")
assert np.allclose(_svg_module._convert_point_to_3d(3.0, 4.0), [3.0, 4.0, 0.0])
assert isinstance(_svg_module.SVG_HASH_TO_MOB_MAP, dict)
assert isinstance(_svg_module.PATH_TO_POINTS, dict)
_rect_only = (
    '<svg xmlns="http://www.w3.org/2000/svg" width="40" height="10">'
    '<rect x="0" y="0" width="40" height="10"/></svg>'
)
assert np.isclose(_svg_module.get_svg_content_height(_rect_only), 10.0)

# Path, io, and ET bind to the identical objects now that leaked-import
# origins record the full object path; se stays a loud refusal because
# svgelements is deliberately unshipped (OOT-SVGELEMENTS-PARSER).
assert _svg_module.Path is importlib.import_module("pathlib").Path
assert _svg_module.io is importlib.import_module("io")
assert _svg_module.ET is importlib.import_module("xml.etree.ElementTree")
try:
    _svg_module.se("anything")
except NotImplementedError:
    pass
else:
    raise AssertionError("unshipped svgelements must refuse loudly")


# ---------------------------------------------------------------------------
# fm-easc: the coordinate-system composition shelf over the live axes.

_cs_axes = manimlib.Axes(x_range=(-4, 4, 1), y_range=(-2, 2, 1))
_cs_graph = _cs_axes.get_graph(lambda x: x ** 2)
assert np.allclose(_cs_axes.i2gp(2, _cs_graph), [2, 4, 0])
assert abs(_cs_axes.slope_of_tangent(1.0, _cs_graph) - 2.0) < 0.05
_tangent = _cs_axes.get_tangent_line(1.0, _cs_graph)
assert np.allclose(_tangent.get_center(), [1.0, 1.0, 0.0], atol=0.3)
_axis_labels = _cs_axes.get_axis_labels()
assert len(_axis_labels) == 2
_vline = _cs_axes.get_v_line([2, 1, 0])
assert np.isclose(_vline.get_start()[0], 2)
_hline = _cs_axes.get_h_line([2, 1, 0])
assert np.isclose(_hline.get_start()[1], 1)
_riemann = _cs_axes.get_riemann_rectangles(_cs_graph, x_range=(0, 2), dx=0.5)
assert len(_riemann) == 4
_area = _cs_axes.get_area_under_graph(_cs_graph, x_range=(0, 2))
assert len(_area.get_points()) > 0
_scatter = _cs_axes.get_scatterplot(np.array([1.0, 2.0]), np.array([1.0, 4.0]))
assert _scatter is not None
_glabel = _cs_axes.get_graph_label(_cs_graph, "x^2")
assert _glabel is not None

_plane = manimlib.NumberPlane(x_range=(-3, 3, 1), y_range=(-2, 2, 1))
_majors, _minors = _plane.get_lines()
assert len(_majors) > 0 and len(_minors) > 0
# C-17: both unit sizes measure their own axis, so the default plane's
# square aspect reports equal units instead of the pinned x-axis answer.
assert abs(_plane.get_x_unit_size() - _plane.get_y_unit_size()) < 1e-6
_pvec = _plane.get_vector([1, 1])
assert np.allclose(_pvec.get_end(), [1.0, 1.0, 0.0])
assert _plane.prepare_for_nonlinear_transform() is _plane

_cplane = manimlib.ComplexPlane(x_range=(-3, 3, 1), y_range=(-2, 2, 1))
_cdefaults = _cplane.get_default_coordinate_values()
assert -2.0 in _cdefaults and 2j in _cdefaults
assert np.isclose(_cplane.get_unit_size(), _plane.get_x_unit_size())

_tdaxes = manimlib.ThreeDAxes()
assert _tdaxes.add_axis_labels() is None and len(_tdaxes.axis_labels) == 3
_surface = _tdaxes.get_parametric_surface(
    lambda u, v: [u, v, u * v], u_range=(-1, 1), v_range=(-1, 1)
)
assert _surface.get_num_points() > 0
_user_axis = _tdaxes.create_axis((-2, 2, 1), {}, None)
assert np.allclose(_user_axis.n2p(0), [0.0, 0.0, 0.0])
assert _tdaxes.get_z_axis() is _tdaxes.z_axis

# Remaining coordinate_systems shelf: classes, two-term ranges, complex
# number mapping, abstract CoordinateSystem, and ThreeDAxes.get_graph.
cs_mod = importlib.import_module("manimlib.mobject.coordinate_systems")
assert cs_mod.DEFAULT_X_RANGE == (-8.0, 8.0, 1.0)
assert cs_mod.DEFAULT_Y_RANGE == (-4.0, 4.0, 1.0)
assert cs_mod.EPSILON == 1e-8
assert cs_mod.full_range_specifier((-2, 2)) == (-2, 2, 1)
assert cs_mod.full_range_specifier((-2, 2, 0.5)) == (-2, 2, 0.5)
assert manimlib.full_range_specifier is cs_mod.full_range_specifier
assert cs_mod.numbers is numbers
assert issubclass(cs_mod.CoordinateSystem, abc.ABC)
try:
    cs_mod.CoordinateSystem()
except TypeError:
    pass
else:
    raise AssertionError("CoordinateSystem must stay abstract")

two_term_axes = manimlib.Axes(x_range=(-2, 2), y_range=(-1, 1))
assert two_term_axes.x_range == (-2.0, 2.0, 1.0)
assert two_term_axes.y_range == (-1.0, 1.0, 1.0)
assert two_term_axes.get_axes() is two_term_axes.axes
assert two_term_axes.get_all_ranges() == [two_term_axes.x_range, two_term_axes.y_range]
assert np.allclose(
    two_term_axes.coords_to_point(1.0, 0.5),
    two_term_axes.c2p(1.0, 0.5),
)
assert np.allclose(
    two_term_axes.point_to_coords(two_term_axes.c2p(1.0, 0.5)),
    [1.0, 0.5],
)
assert two_term_axes.add_coordinate_labels() is two_term_axes.coordinate_labels
assert manimlib.Axes.default_axis_config == {}
assert manimlib.Axes.default_x_axis_config == {}
assert manimlib.Axes.default_y_axis_config["line_to_number_direction"] is not None

two_term_plane = manimlib.NumberPlane(x_range=(-3, 3), y_range=(-2, 2))
assert two_term_plane.x_range == (-3.0, 3.0, 1.0)
assert two_term_plane.get_axes() is two_term_plane.axes
assert manimlib.NumberPlane.default_axis_config["include_ticks"] is False

_cplane.add_coordinate_labels()
assert _cplane.coordinate_labels is not None
assert np.allclose(_cplane.n2p(1 + 2j), _cplane.coords_to_point(1.0, 2.0))
_c_round = _cplane.p2n(_cplane.number_to_point(3 - 1j))
assert np.isclose(_c_round.real, 3.0) and np.isclose(_c_round.imag, -1.0)
_c_imag = _cplane.point_to_number(_cplane.n2p(-2j))
assert np.isclose(_c_imag.real, 0.0) and np.isclose(_c_imag.imag, -2.0)

two_term_td = manimlib.ThreeDAxes(x_range=(-2, 2), y_range=(-2, 2), z_range=(-1, 1))
assert two_term_td.x_range == (-2.0, 2.0, 1.0)
assert two_term_td.z_range == (-1.0, 1.0, 1.0)
assert two_term_td.get_all_ranges() == [
    two_term_td.x_range,
    two_term_td.y_range,
    two_term_td.z_range,
]
assert manimlib.ThreeDAxes.dimension == 3
_td_graph = _tdaxes.get_graph(
    lambda u, v: u + v,
    u_range=(0.0, 1.0),
    v_range=(0.0, 1.0),
    resolution=(3, 3),
)
assert isinstance(_td_graph, manimlib.ParametricSurface)
assert not isinstance(_td_graph, functions.ParametricCurve)
assert _td_graph.get_num_points() > 0
assert np.allclose(_td_graph.u_range[:2], [0.0, 1.0])
assert np.allclose(_td_graph.v_range[:2], [0.0, 1.0])
# On default unit axes the (1, 0, func=1) sample sits at c2p(1, 0, 1).
assert np.allclose(
    _td_graph.uv_func(1.0, 0.0),
    [1.0, 0.0, 1.0],
)


# ---------------------------------------------------------------------------
# fm-a8mg: duck-typed svg converter seams.

_svg_conv = manimlib.SVGMobject(
    svg_string=(
        '<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">'
        '<rect x="0" y="0" width="10" height="10"/></svg>'
    )
)


class _StubRect:
    x, y, width, height, rx, ry = 1.0, 2.0, 30.0, 20.0, 0.0, 0.0


_rect_mob = _svg_conv.rect_to_mobject(_StubRect())
assert np.isclose(_rect_mob.get_width(), 30.0)
assert np.isclose(_rect_mob.get_height(), 20.0)
assert np.allclose(_rect_mob.get_center(), [16.0, 12.0, 0.0])


class _StubRound:
    x, y, width, height, rx, ry = 0.0, 0.0, 40.0, 40.0, 2.0, 4.0


_round_mob = _svg_conv.rect_to_mobject(_StubRound())
assert np.isclose(_round_mob.get_width(), 40.0)
assert np.isclose(_round_mob.get_height(), 40.0)


class _StubLine:
    x1, y1, x2, y2 = 0.0, 0.0, 6.0, -8.0


_line_mob = _svg_conv.line_to_mobject(_StubLine())
assert np.allclose(_line_mob.get_start(), [0.0, 0.0, 0.0])
assert np.allclose(_line_mob.get_end(), [6.0, -8.0, 0.0])


class _StubEllipse:
    rx, ry, cx, cy = 5.0, 2.5, 10.0, -4.0


_ellipse_mob = _svg_conv.ellipse_to_mobject(_StubEllipse())
assert np.isclose(_ellipse_mob.get_width(), 10.0)
assert np.isclose(_ellipse_mob.get_height(), 5.0)
assert np.allclose(_ellipse_mob.get_center(), [10.0, -4.0, 0.0])


class _StubPoly:
    def __iter__(self):
        return iter([(0.0, 0.0), (4.0, 0.0), (0.0, 3.0)])


_poly_mob = _svg_conv.polygon_to_mobject(_StubPoly())
assert _poly_mob.get_num_curves() == 3
_pl_mob = _svg_conv.polyline_to_mobject(_StubPoly())
assert not np.allclose(_pl_mob.get_start(), _pl_mob.get_end())


class _StubPaint:
    class stroke:
        hexrgb = "#123456"
        opacity = 0.5

    class fill:
        hexrgb = "#ABCDEF"
        opacity = 0.25

    stroke_width = 6.0


_painted = _svg_conv.apply_style_to_mobject(
    manimlib.Square(side_length=1.0), _StubPaint()
)
assert _painted.get_stroke_color() == "#123456"
assert abs(_painted.get_stroke_opacity() - 0.5) < 1e-6
assert _painted.get_fill_color() == "#ABCDEF"
assert abs(_painted.get_fill_opacity() - 0.25) < 1e-6
assert np.isclose(_painted.get_stroke_width(), 6.0)


class _StubMatrix:
    a, b, c, d, e, f = 2.0, 0.0, 0.0, 3.0, 10.0, -5.0


_transformed = manimlib.Square(side_length=1.0)
_svg_conv.handle_transform(_transformed, _StubMatrix())
assert np.isclose(_transformed.get_width(), 2.0)
assert np.isclose(_transformed.get_height(), 3.0)

_xml_tree = importlib.import_module("xml.etree.ElementTree").ElementTree(
    importlib.import_module("xml.etree.ElementTree").fromstring(
        '<svg width="10" height="10" fill="#FF0000"><rect x="0" y="0" width="4" height="4"/></svg>'
    )
)
_new_tree = _svg_conv.modify_xml_tree(_xml_tree)
_new_root = _new_tree.getroot()
assert _new_root.tag == "svg"
_groups = list(_new_root)
assert len(_groups) == 1 and _groups[0].tag.endswith("g")
_inner_groups = list(_groups[0])
assert len(_inner_groups) == 1 and _inner_groups[0].attrib.get("fill") == "#FF0000"
assert len(list(_inner_groups[0])) == 1


class _StubDPath:
    def d(self):
        return "M 0 0 L 40 0"


_direct_path = _svg_conv.path_to_mobject(_StubDPath())
assert _direct_path.get_num_points() > 0


class _StubSvg:
    objects = {"ref1": _StubDPath()}


class _StubUsePath:
    id = "ref1"
    values = {}


_ref_mob = _svg_conv.path_to_mobject(_StubUsePath(), _StubSvg())
assert _ref_mob.get_num_points() > 0


class _StubUseTransformed:
    id = "ref1"
    values = {"transform": _StubMatrix()}


_ref_shifted = _svg_conv.path_to_mobject(_StubUseTransformed(), _StubSvg())
assert np.isclose(_ref_shifted.get_center()[0] - _ref_mob.get_center()[0], 30.0)


class _StubUseRawString:
    id = "ref1"
    values = {"transform": "translate(5)"}


try:
    _svg_conv.path_to_mobject(_StubUseRawString(), _StubSvg())
except TypeError:
    pass
else:
    raise AssertionError("raw transform strings need the native pipeline")


# ---------------------------------------------------------------------------
# fm-gqfu: the Surface UV/triangle shelf over native surface records.

_surf = manimlib.ParametricSurface(
    lambda u, v: [u, v, u * v], u_range=(0, 1), v_range=(0, 1), resolution=(11, 11)
)
assert _surf.passed_uv_func is not None
assert _surf.uv_func(0.5, 0.25)[2] == 0.125
_grid = _surf.get_uv_grid()
assert _grid.shape == (11, 11, 2) and _grid[0, 0].tolist() == [0.0, 0.0]
assert np.allclose(_surf.uv_to_point(0.5, 0.5), [0.5, 0.5, 0.25], atol=1e-9)
_tri = _surf.compute_triangle_indices()
assert _tri.shape == (600,) and _tri.max() == 120
assert np.array_equal(_surf.get_triangle_indices(), _tri)
assert _surf.get_unit_normals().shape == (121, 3)
_surf.sort_faces_back_to_front(np.array([0.0, 0.0, 1.0]))
_sorted = _surf.get_triangle_indices()
assert set(_sorted.tolist()) <= set(range(121)) and len(_sorted) == 600


def _surface_uv_color(u, v):
    return "#FF0000" if u < 0.5 else "#0000FF"


_surf.color_by_uv_function(_surface_uv_color)
_rgba = _surf.data["rgba"]
_reds = int(((_rgba[:, 0] > 0.9) & (_rgba[:, 2] < 0.1)).sum())
_blues = int(((_rgba[:, 2] > 0.9) & (_rgba[:, 0] < 0.1)).sum())
assert _reds == 55 and _blues == 66
_partial = _surf.get_partial_points_array(
    _surf.get_points(), 0.25, 0.75, (11, 11, 3), 0
)
assert _partial.shape == (121, 3)
_surf.always_sort_to_camera(manimlib.Camera())

_late_texture = manimlib.TexturedSurface(_surf, _texture_path, opacity=.5)
assert _late_texture.resolution == _surf.resolution
assert np.array_equal(_late_texture.get_points(), _surf.get_points())
assert np.allclose(_late_texture.get_opacities(), .5)
_late_texture.set_image_coords_by_uv_func(lambda u, v: (u / 2, v))
assert np.allclose(_late_texture.data["im_coords"][-1], [.5, 0])

_mtl_dir = tempfile.mkdtemp()
_mtl_path = pathlib.Path(_mtl_dir)
(_mtl_path / "model.obj").write_text("mtllib model.mtl\nv 0 0 0\n")
(_mtl_path / "model.mtl").write_text("newmtl matA\nmap_Kd texA.png\nnewmtl matB\n")
_textures = manimlib.ThreeDModel.__dict__["get_textures_from_mtl"](
    None, str(_mtl_path / "model.obj")
)
assert _textures["matA"].endswith("texA.png") and _textures["matB"] is None

# fm-5wq.4: remaining types.surface identities over native sampled grids.
assert surface_types.Surface.__bases__ == (manimlib.Mobject,)
assert surface_types.ParametricSurface.__bases__ == (surface_types.Surface,)
assert surface_types.SGroup.__bases__ == (surface_types.Surface,)
assert list(inspect.signature(surface_types.Surface).parameters) == [
    "color",
    "shading",
    "depth_test",
    "u_range",
    "v_range",
    "resolution",
    "preferred_creation_axis",
    "epsilon",
    "normal_nudge",
    "kwargs",
]
assert list(inspect.signature(surface_types.SGroup).parameters) == [
    "parametric_surfaces",
    "kwargs",
]
assert surface_types.Surface.shader_folder == "surface"
assert surface_types.Surface.render_primitive == 4
assert surface_types.Surface.pointlike_data_keys == ["point", "d_normal_point"]
assert [field[0] for field in surface_types.Surface.data_dtype] == [
    "point",
    "d_normal_point",
    "rgba",
]
assert surface_types.TexturedSurface.shader_folder == "textured_surface"
assert [field[0] for field in surface_types.TexturedSurface.data_dtype] == [
    "point",
    "d_normal_point",
    "im_coords",
    "opacity",
]

plane = surface_types.Surface(
    u_range=(0.0, 2.0),
    v_range=(0.0, 1.0),
    resolution=(3, 2),
    color=manimlib.BLUE,
)
assert plane.n_records() == 6
assert plane.resolution == (3, 2)
assert plane.u_range == (0.0, 2.0)
assert plane.v_range == (0.0, 1.0)
assert np.allclose(plane.uv_func(1.0, 0.5), (1.0, 0.5, 0.0))
assert np.allclose(plane.get_points()[0], [0.0, 0.0, 0.0], atol=1e-5)
assert np.allclose(plane.get_points()[-1], [2.0, 1.0, 0.0], atol=1e-5)
assert plane.init_points() is None
assert plane.n_records() == 6
plane_indices = plane.get_shader_vert_indices()
assert plane_indices.shape == (12,)
assert np.array_equal(plane_indices, plane.get_triangle_indices())
plane_points = plane.get_points().copy()
plane_normals = plane.data["d_normal_point"].copy()
assert (
    plane.apply_points_function(
        lambda points: points + np.array([1.0, 0.0, 0.0], dtype=np.float32),
        about_edge=None,
    )
    is plane
)
assert np.allclose(plane.get_points(), plane_points + np.array([1.0, 0.0, 0.0]))
assert np.allclose(
    plane.data["d_normal_point"], plane_normals + np.array([1.0, 0.0, 0.0])
)

failed_surface = surface_types.Surface.__new__(surface_types.Surface)
try:
    surface_types.Surface.__init__(failed_surface, texture_paths={})
except NotImplementedError as error:
    assert str(error) == (
        "Surface() keyword(s) not yet routed to the native builder: texture_paths"
    )
else:
    raise AssertionError("an unrouted Surface keyword reached the native builder")
assert not hasattr(failed_surface, "submobjects")

# fm-c1up: a keyword outside the Reference's Surface/Mobject chain is the
# Reference's own TypeError (Cone(direction=RIGHT) raises it at 6199a00d),
# never the unrouted-routing refusal, and it fails before any state exists.
for solid, arguments in (
    (surface_types.Surface, ()),
    (three_dimensions.Sphere, ()),
    (three_dimensions.Torus, ()),
    (three_dimensions.Cylinder, ()),
    (three_dimensions.Cone, ()),
    (three_dimensions.Line3D, (manimlib.LEFT, manimlib.RIGHT)),
    (three_dimensions.Disk3D, ()),
    (three_dimensions.Square3D, ()),
    (three_dimensions.Cube, ()),
    (three_dimensions.Prism, ()),
):
    unknown_solid = solid.__new__(solid)
    try:
        solid.__init__(unknown_solid, *arguments, direction=manimlib.RIGHT, bogus=True)
    except TypeError as error:
        assert str(error) == (
            "Mobject.__init__() got an unexpected keyword argument 'direction'"
        ), (solid.__name__, error)
    else:
        raise AssertionError(solid.__name__ + " accepted an unknown keyword")
    assert not hasattr(unknown_solid, "submobjects"), solid.__name__
try:
    three_dimensions.Cone(direction=manimlib.RIGHT)
except TypeError as error:
    assert "unexpected keyword argument 'direction'" in str(error), error
else:
    raise AssertionError("Cone accepted direction=")

left_sheet = surface_types.ParametricSurface(
    lambda u, v: [u, v, 0.0],
    resolution=(2, 2),
)
right_sheet = surface_types.ParametricSurface(
    lambda u, v: [u + 3.0, v, 0.0],
    resolution=(2, 2),
)
surface_group = surface_types.SGroup(left_sheet, right_sheet)
assert surface_group.resolution == (0, 0)
assert surface_group.submobjects == [left_sheet, right_sheet]
assert surface_group.init_points() is None
assert surface_group.submobjects == [left_sheet, right_sheet]
assert np.array_equal(surface_group.get_triangle_indices(), np.zeros(0, dtype=int))

for leaked in ("logging", "pywavefront", "trimesh"):
    try:
        getattr(surface_types, leaked)()
    except NotImplementedError as error:
        assert "OOT-LEAKED-SURFACE-IMPORTS" in str(error), (leaked, error)
    else:
        raise AssertionError(f"{leaked} was callable without a named exclusion")

# fm-5wq.4: the remaining DotCloud shelf over native point/radius/rgba records.
dot_cloud_mod = importlib.import_module("manimlib.mobject.types.dot_cloud")
assert dot_cloud_mod.DotCloud.__bases__ == (manimlib.PMobject,)
assert dot_cloud_mod.TrueDot.__bases__ == (dot_cloud_mod.DotCloud,)
assert dot_cloud_mod.GlowDots.__bases__ == (dot_cloud_mod.DotCloud,)
assert dot_cloud_mod.GlowDot.__bases__ == (dot_cloud_mod.GlowDots,)
assert dot_cloud_mod.DEFAULT_DOT_RADIUS == 0.05
assert dot_cloud_mod.DEFAULT_GLOW_DOT_RADIUS == 0.2
assert dot_cloud_mod.DEFAULT_GRID_HEIGHT == 6
assert dot_cloud_mod.DEFAULT_BUFF_RATIO == 0.5
assert dot_cloud_mod.DotCloud.shader_folder == "true_dot"
assert dot_cloud_mod.DotCloud.render_primitive == 0
assert [field[0] for field in dot_cloud_mod.DotCloud.data_dtype] == [
    "point",
    "radius",
    "rgba",
]
assert list(inspect.signature(dot_cloud_mod.DotCloud).parameters) == [
    "points",
    "color",
    "opacity",
    "radius",
    "glow_factor",
    "anti_alias_width",
    "kwargs",
]
assert list(inspect.signature(dot_cloud_mod.TrueDot).parameters) == [
    "center",
    "kwargs",
]
assert list(inspect.signature(dot_cloud_mod.GlowDots).parameters) == [
    "points",
    "color",
    "radius",
    "glow_factor",
    "kwargs",
]
assert list(inspect.signature(dot_cloud_mod.GlowDot).parameters) == [
    "center",
    "kwargs",
]

default_cloud = dot_cloud_mod.DotCloud()
assert default_cloud.n_records() == 1
assert np.allclose(default_cloud.get_points(), [[0.0, 0.0, 0.0]])
assert np.isclose(default_cloud.get_radius(), 0.05)
assert np.allclose(default_cloud.get_radii(), [[0.05]])
assert np.isclose(default_cloud.get_glow_factor(), 0.0)
assert np.isclose(default_cloud.uniforms["glow_factor"], 0.0)
assert np.isclose(default_cloud.uniforms["anti_alias_width"], 2.0)

cloud = dot_cloud_mod.DotCloud(
    [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
    radius=0.25,
    glow_factor=0.5,
)
assert cloud.n_records() == 3
assert np.isclose(cloud.get_radius(), 0.25)
assert cloud.set_radii([0.1, 0.2, 0.4]) is cloud
assert np.allclose(cloud.get_radii().reshape(-1), [0.1, 0.2, 0.4])
assert np.isclose(cloud.get_radius(), 0.4)
assert cloud.set_radius(0.3) is cloud
assert np.allclose(cloud.get_radii().reshape(-1), [0.3, 0.3, 0.3])
assert cloud.scale_radii(2.0) is cloud
assert np.allclose(cloud.get_radii().reshape(-1), [0.6, 0.6, 0.6])
before_points = cloud.get_points().copy()
assert cloud.scale(0.5, scale_radii=False, about_point=manimlib.ORIGIN) is cloud
assert np.allclose(cloud.get_points(), before_points * 0.5)
assert np.allclose(cloud.get_radii().reshape(-1), [0.6, 0.6, 0.6])
assert cloud.scale(2.0, about_point=manimlib.ORIGIN) is cloud
assert np.allclose(cloud.get_radii().reshape(-1), [1.2, 1.2, 1.2])
assert cloud.set_glow_factor(1.5) is cloud
assert np.isclose(cloud.get_glow_factor(), 1.5)
assert np.isclose(cloud.uniforms["glow_factor"], 1.5)
if "glow_factor" in cloud.data.dtype.names:
    assert np.allclose(cloud.data["glow_factor"].reshape(-1), [1.5, 1.5, 1.5])
assert cloud.make_3d() is cloud
assert np.allclose(cloud.get_shading(), [0.5, 0.1, 0.2])
assert bool(cloud.uniforms["depth_test"]) is True

grid = dot_cloud_mod.DotCloud(radius=0.05).to_grid(2, 3)
assert grid is not None
assert grid.n_records() == 6
assert np.isclose(grid.get_height(), 6.0, atol=1e-5)
assert np.allclose(grid.get_center(), [0.0, 0.0, 0.0], atol=1e-5)
buff_grid = dot_cloud_mod.DotCloud(radius=0.05).to_grid(2, 2, buff_ratio=0.0)
assert buff_grid.n_records() == 4

true_dot = dot_cloud_mod.TrueDot([1.0, 2.0, 3.0], radius=0.1)
assert true_dot.n_records() == 1
assert np.allclose(true_dot.get_points(), [[1.0, 2.0, 3.0]])
assert np.isclose(true_dot.get_radius(), 0.1)
glow_dots = dot_cloud_mod.GlowDots([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])
assert glow_dots.n_records() == 2
assert np.isclose(glow_dots.get_radius(), 0.2)
assert np.isclose(glow_dots.get_glow_factor(), 2.0)
glow_dot = dot_cloud_mod.GlowDot([-1.0, 0.0, 0.0])
assert glow_dot.n_records() == 1
assert np.allclose(glow_dot.get_points(), [[-1.0, 0.0, 0.0]])
assert np.isclose(glow_dot.get_radius(), 0.2)
assert np.isclose(glow_dot.get_glow_factor(), 2.0)


# ---------------------------------------------------------------------------
# fm-g6cj: the SampleSpace probability shelf over native Rectangle parts.

_ss = manimlib.SampleSpace()
assert _ss.complete_p_list([0.25, 0.25]) == [0.25, 0.25, 0.5]
assert _ss.complete_p_list([0.5, 0.5]) == [0.5, 0.5]

_horiz = _ss.get_horizontal_division([0.7])
assert sorted(round(p.get_height(), 6) for p in _horiz) == [0.9, 2.1]
_vert = _ss.get_vertical_division([0.5, 0.25])
assert sorted(round(p.get_width(), 6) for p in _vert) == [0.75, 0.75, 1.5]
_vcolors = [p.get_fill_color() for p in _vert]
assert _vcolors[0] == "#EC92AB" and _vcolors[-1] == "#FFFF00"
assert all(isinstance(p, manimlib.SampleSpace) for p in _vert)

_ss.divide_horizontally([0.6])
assert hasattr(_ss, "horizontal_parts")
_side = _ss.get_side_braces_and_labels(["P(A)", "B"])
assert len(_side) == 2
_ss.divide_vertically([0.3])
_top = _ss.get_top_braces_and_labels(["X", "Y"])
assert len(_top) == 2
_bottom = _ss.get_bottom_braces_and_labels(["U"])
assert len(_bottom) == 2
_before = len(_ss.submobjects)
_ss.add_braces_and_labels()
assert len(_ss.submobjects) > _before
assert _ss[0] is not None

_titled = manimlib.SampleSpace()
assert _titled.add_title() is None and _titled.title.get_center()[1] > _titled.get_center()[1]
_titled.add_label("S")
assert _titled.label == "S"


# ---------------------------------------------------------------------------
# fm-5wq.4: the de-TeX'd Brace family.

_braced = manimlib.Square(side_length=2.0)
_brace = manimlib.Brace(_braced, manimlib.DOWN)
assert np.allclose(_brace.get_direction(), [0.0, -1.0, 0.0])
assert np.isclose(_brace.get_width(), 2.0)
assert _brace.get_tip()[1] < _braced.get_bottom()[1]

_up_brace = manimlib.Brace(_braced, manimlib.UP)
assert np.allclose(_up_brace.get_direction(), [0.0, 1.0, 0.0])

# set_initial_width returns self and reaches the requested extent both
# growing and shrinking.
assert _brace.set_initial_width(3.0) is _brace
assert np.isclose(_brace.get_width(), 3.0)
_shrink = manimlib.Brace(manimlib.Square(side_length=2.0), manimlib.DOWN)
_shrink.set_initial_width(1.2)
assert np.isclose(_shrink.get_width(), 1.2)

# get_text returns Text placed on the braced side of the tip.
_label = _brace.get_text("hello")
assert isinstance(_label, manimlib.Text)
assert _label.get_top()[1] < _brace.get_tip()[1]

# get_tex places a Tex mobject the same way.
_tex_label = _brace.get_tex("x")
assert isinstance(_tex_label, manimlib.Tex)
assert _tex_label.get_top()[1] < _brace.get_tip()[1]

# put_at_tip honours an explicit mobject and buff.
_moved = manimlib.Dot()
_brace.put_at_tip(_moved, buff=0.3)
assert _moved.get_bottom()[1] < _brace.get_tip()[1]

# LineBrace spans its line with a perpendicular direction.
_line_brace = manimlib.LineBrace(manimlib.Line(start=[-2, 0, 0], end=[2, 0, 0]))
assert np.isclose(_line_brace.get_width(), 4.0)
assert np.isclose(abs(_line_brace.get_direction()[1]), 1.0)


bz = importlib.import_module("manimlib.utils.bezier")
it = importlib.import_module("manimlib.utils.iterables")


# ---------------------------------------------------------------------------
# fm-fo2m: the remaining bezier and iterables utility shelves.

def _check(label, value):
    assert value, "fm-fo2m %s -> %r" % (label, value)


_bpts = [np.array(p, float) for p in [(0, 0, 0), (1, 2, 0), (3, 3, 0)]]
_bez = bz.bezier(_bpts)
_check("bernstein", np.allclose(_bez(0.5), [1.25, 1.75, 0.0]))
_psub = bz.partial_bezier_points(_bpts, 0.25, 0.75)
_rebez = bz.bezier(_psub)
_check("partial endpoints",
       np.allclose(_rebez(0.0), _bez(0.25)) and np.allclose(_rebez(1.0), _bez(0.75)))
_pq = bz.partial_quadratic_bezier_points(_bpts[:3], 0.0, 0.5)
_check("partial quadratic start", np.allclose(_pq[0], _bpts[0]))
_check("midpoint", np.allclose(bz.mid(np.array([0., 0, 0]), np.array([2., 2, 0])), [1, 1, 0]))
_check("closed true", bz.is_closed(np.array([[0., 0., 0.], [1., 1., 0.], [0., 0., 0.]])))
_check("closed false", not bz.is_closed(np.array([[0., 0., 0.], [1., 1., 0.]])))
_check("match interpolate", bz.match_interpolate(0, 10, 0, 100, 50) == 5.0)
_arc = bz.quadratic_bezier_points_for_arc(np.pi / 2, 4)
_check("arc shape/start", _arc.shape == (9, 3) and np.allclose(_arc[0], [1.0, 0.0, 0.0]))
_oi = bz.outer_interpolate(np.array([0., 0, 0]), np.array([1., 1, 0]), np.array([0.0, 0.5, 1.0]))
_check("outer interpolate shape", _oi.shape == (3, 3))
_sai = bz.set_array_by_interpolation(
    np.zeros((2, 3)), np.ones((2, 3)), np.full((2, 3), 2.0), 0.25
)
_check("set array by interpolation", np.allclose(_sai, 1.25))

_anchors = np.array([[0., 0, 0], [1., 1, 0], [2., 0, 0], [3., 1, 0]])
_h1s, _h2s = bz.get_smooth_cubic_bezier_handle_points(_anchors)
_check("smooth handle shapes", _h1s.shape == (3, 3) and _h2s.shape == (3, 3))
_apx = bz.approx_smooth_quadratic_bezier_handles(_anchors)
_check("approx handle shape", _apx.shape == (3, 3))
_sqpath = bz.smooth_quadratic_path(_anchors)
_check("smooth path", _sqpath.shape[1] == 3 and len(_sqpath) >= 4)
_qapprox = bz.get_quadratic_approximation_of_cubic(
    _anchors[0:1], _h1s[0:1], _h2s[0:1], _anchors[1:2]
)
_check("quad approximation", _qapprox.shape == (5, 3))
_through = bz.get_smooth_quadratic_bezier_path_through(_anchors)
_check("path through", len(_through) > 0)
_band = bz.diag_to_matrix((2, 1), np.array([
    [0, 1, 1, 0, 0],
    [2, 1, 0, 0, 0],
    [1, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
], dtype=float))
_check("diag to matrix", _band.shape == (5, 5))

_check("remove redundancies", it.remove_list_redundancies([1, 2, 1, 3]) == [2, 1, 3])
_check("list update", it.list_update([1, 2], [2, 3]) == [1, 2, 3])
_check("list difference", it.list_difference_update([1, 2, 3], [2]) == [1, 3])
_check("adjacent pairs", list(it.adjacent_pairs([1, 2, 3])) == [(1, 2), (2, 3), (3, 1)])
_check("adjacent n", list(it.adjacent_n_tuples([1, 2, 3], 3)) == [(1, 2, 3), (2, 3, 1), (3, 1, 2)])
_check("batch by property", len(it.batch_by_property([1, 1, 2, 3, 3], lambda x: x % 2)) == 3)
_shuffled = it.shuffled([1, 2, 3])
_check("shuffled preserves members", sorted(_shuffled) == [1, 2, 3])
_rw = it.resize_with_interpolation(np.array([[0., 0.], [10., 10.]]), 3)
_check("resize interpolation", _rw.shape == (3, 2) and np.allclose(_rw[1], [5., 5.]),
       )
_ra = it.resize_array(np.arange(6).reshape(2, 3), 4)
_check("resize array", _ra.shape == (4, 3))
_rpo = it.resize_preserving_order(np.arange(4), 6)
_check("resize preserving order", _rpo.tolist() == [0, 0, 1, 2, 2, 3])
_check("array is constant true/false",
       it.array_is_constant(np.ones((2, 2))) and not it.array_is_constant(np.zeros(0)))
_e1, _e2 = it.make_even([1, 2, 3, 4], [7, 8])
_check("make even", len(_e1) == len(_e2) == 4)
_check("arrays match", it.arrays_match(np.arange(3), np.arange(3)))
_cp = it.cartesian_product(np.array([1, 2]), np.array([3, 4]))
_check("cartesian product", _cp.shape == (4, 2))
_check("hash obj", isinstance(it.hash_obj({"a": [1, 2]}), int))


# ---------------------------------------------------------------------------
# fm-5wq.4: the Reference easing shelf (manimlib.utils.rate_functions).

rf = importlib.import_module("manimlib.utils.rate_functions")
assert manimlib.smooth is rf.smooth and manimlib.linear is rf.linear

_check("linear identity", rf.linear(0.25) == 0.25 and rf.linear(0.0) == 0.0
       and rf.linear(1.0) == 1.0)
_check("smooth endpoints", rf.smooth(0.0) == 0.0 and rf.smooth(1.0) == 1.0)
_check("smooth midpoint", np.isclose(rf.smooth(0.5), 0.5))
_check("smooth quintic value", np.isclose(rf.smooth(0.25), 0.103515625))
_check("rush into first half",
       np.isclose(rf.rush_into(0.5), 0.20703125) and np.isclose(rf.rush_into(1.0), 1.0))
_check("rush from second half",
       np.isclose(rf.rush_from(0.0), 0.0) and np.isclose(rf.rush_from(0.5), 0.79296875))
_check("slow into quarter circle", np.isclose(rf.slow_into(0.5), math.sqrt(0.75)))
_check("double smooth branches",
       np.isclose(rf.double_smooth(0.25), 0.25) and np.isclose(rf.double_smooth(0.75), 0.75))
_check("there and back turns at the midpoint",
       rf.there_and_back(0.0) == 0.0 and rf.there_and_back(1.0) == 0.0
       and np.isclose(rf.there_and_back(0.25), 0.5)
       and np.isclose(rf.there_and_back(0.75), 0.5))
_check("there and back with pause plateau",
       rf.there_and_back_with_pause(0.5) == 1.0
       and np.isclose(rf.there_and_back_with_pause(0.2), 0.68256)
       and np.isclose(rf.there_and_back_with_pause(0.9), 0.16308))
_check("running start dips negative then lands",
       rf.running_start(0.0) == 0.0 and rf.running_start(1.0) == 1.0
       and rf.running_start(0.12) < 0.0)
_check("overshoot exceeds one then settles",
       rf.overshoot(0.0) == 0.0 and rf.overshoot(1.0) == 1.0
       and np.isclose(rf.overshoot(0.8), 1.12128))
_check("not quite there scales proportion",
       np.isclose(rf.not_quite_there()(1.0), 0.7)
       and np.isclose(rf.not_quite_there(proportion=0.3)(1.0), 0.3))
_check("wiggle rides sine over there-and-back",
       np.isclose(rf.wiggle(0.25), 0.5) and np.isclose(rf.wiggle(0.75), -0.5))
_squished = rf.squish_rate_func(lambda t: t, 0.4, 0.6)
_check("squish clamps outside band",
       _squished(0.2) == 0.0 and _squished(0.8) == 1.0 and np.isclose(_squished(0.5), 0.5))
_check("squish degenerate band returns edge",
       rf.squish_rate_func(lambda t: t, 0.4, 0.4)(0.7) == 0.4)
_check("lingering front loads", np.isclose(rf.lingering(0.4), 0.5) and rf.lingering(0.9) == 1.0)
_check("exponential decay half life",
       rf.exponential_decay(0.0) == 0.0
       and np.isclose(rf.exponential_decay(0.1), 1.0 - math.exp(-1.0)))


# ---------------------------------------------------------------------------
# fm-5wq.4: the numbers + value-tracker shelf.

functools = importlib.import_module("functools")
numbers_mod = importlib.import_module("manimlib.mobject.numbers")
vt_mod = importlib.import_module("manimlib.mobject.value_tracker")

_check("numbers module leaked lru_cache",
       numbers_mod.lru_cache is functools.lru_cache)
_check("char cache identity",
       numbers_mod.char_to_cahced_mob("7") is numbers_mod.char_to_cahced_mob("7"))
_check("char cache routes digits to Text",
       isinstance(numbers_mod.char_to_cahced_mob("3"), manimlib.Text))
_check("char cache routes i and commands to Tex",
       isinstance(numbers_mod.char_to_cahced_mob("i"), manimlib.Tex)
       and isinstance(numbers_mod.char_to_cahced_mob(r"\circ"), manimlib.Tex))

_dn = manimlib.DecimalNumber(3.14159, num_decimal_places=3)
_check("decimal get_value", _dn.get_value() == 3.14159)
_check("decimal num_string/tex",
       _dn.num_string == "3.142" and _dn.get_tex() == "3.142")
_check("decimal formatter default", _dn.get_formatter() == "{:,.3f}")
_check("decimal formatter overrides",
       _dn.get_formatter(include_sign=True, group_with_commas=False)
       == "{:+.3f}"
       and _dn.get_formatter(num_decimal_places=0) == "{:,d}")
_check("decimal complex formatter string",
       _dn.get_complex_formatter()
       == "{0.real:,.3f}{0.imag:+,.3f}i")
_check("decimal font accessor",
       _dn.get_font_size() == 48.0 and _dn.font_size == 48.0)
_check("char_to_mob member",
       isinstance(_dn.char_to_mob("5"), manimlib.Text))

_neg = manimlib.DecimalNumber(-0.001, num_decimal_places=2)
_check("negative zero clamp drops sign", _neg.num_string == "0.00")
_signed = manimlib.DecimalNumber(-0.001, num_decimal_places=2, include_sign=True)
_check("negative zero clamp keeps plus", _signed.num_string == "+0.00")
_commas = manimlib.DecimalNumber(1234567.891)
_check("comma grouping default", _commas.num_string == "1,234,567.89")
_intfmt = manimlib.DecimalNumber(42.7, num_decimal_places=0)
_check("zero places formats integer string", _intfmt.num_string == "42")
_en = manimlib.DecimalNumber(-5.0, num_decimal_places=1)
_check("en dash replacement", _en.num_string == "–5.0")

_imag = manimlib.DecimalNumber(complex(0, 2.5))
_check("imag-only construction value", _imag.get_value() == complex(0, 2.5))
_check("imag-only num string", _imag.num_string == "2.50i")
_general = manimlib.DecimalNumber(complex(1.0, 2.0))
_check("general complex composes native readouts",
       _general.get_value() == complex(1.0, 2.0)
       and _general.get_tex() == "1.00+2.00i"
       and len(_general) == len(_general.num_string))
_switch = manimlib.DecimalNumber(1.0)
_check("real display can become imaginary",
       _switch.set_value(complex(0.0, 1.0)) is _switch
       and _switch.get_value() == 1j and _switch.num_string == "1.00i")

_grow = manimlib.DecimalNumber(1.0, num_decimal_places=2)
_family_before = len([sm for sm in _grow.get_family() if sm.has_points()])
_left_before = _grow.get_edge_center(manimlib.LEFT).copy()
_grew = _grow.set_value(12345.68)
_check("set_value returns self live",
       _grew is _grow and _grow.get_value() == 12345.68
       and _grow.num_string == "12,345.68")
_check("glyph family grows across rebuild",
       len([sm for sm in _grow.get_family() if sm.has_points()])
       > _family_before)
_check("edge reseat holds the left edge",
       np.allclose(_grow.get_edge_center(manimlib.LEFT), _left_before))
_check("increment chains fluently",
       _grow.increment_value(0.32) is _grow
       and np.isclose(_grow.get_value(), 12346.0)
       and _grow.num_string == "12,346.00")

_a = manimlib.DecimalNumber(1.0, num_decimal_places=1, font_size=48)
_b = manimlib.DecimalNumber(3.0, num_decimal_places=1, font_size=60)
_mid = manimlib.DecimalNumber(1.0, num_decimal_places=1, font_size=48)
_mid.interpolate(_a, _b, 0.25)
_check("interpolate lerps font size",
       np.isclose(_mid.font_size, 51.0) and _mid.get_font_size() == _mid.font_size)

_gi = manimlib.Integer(7)
_check("integer value type", _gi.get_value() == 7 and isinstance(_gi.get_value(), int))
_check("integer rounds through numpy",
       manimlib.Integer(3.7).get_value() == 4
       and manimlib.Integer(2.5).get_value() == 2
       and manimlib.Integer(3.5).get_value() == 4)
_kwarg_refused = False
try:
    manimlib.Integer(1, bogus_knob=3)
except NotImplementedError:
    _kwarg_refused = True
_check("integer subclasses decimal",
       issubclass(manimlib.Integer, manimlib.DecimalNumber))

_t = manimlib.ValueTracker(4)
_check("tracker init/get", _t.get_value() == 4.0
       and isinstance(_t.get_value(), float))
_check("tracker set fluent", _t.set_value(9) is _t and _t.get_value() == 9.0)
_check("tracker increment returns none like reference",
       _t.increment_value(1) is None and _t.get_value() == 10.0)
_check("tracker value uniform mirrors writes",
       _t.uniforms["value"].dtype == np.float64
       and _t.uniforms["value"][0] == 10.0)

_et = manimlib.ExponentialValueTracker(1.0)
_check("exponential round trip",
       _et.get_value() == 1.0
       and np.isclose((_et.set_value(8.0), _et.get_value())[1], 8.0))
_et.increment_value(1.0)
_check("exponential increment adds in linear space",
       np.isclose(_et.get_value(), 9.0))

_ct = vt_mod.ComplexValueTracker(complex(3, 4))
_check("complex tracker round trip",
       _ct.get_value() == complex(3, 4)
       and (_ct.set_value(1 + 2j), _ct.get_value())[1] == complex(1, 2))
_check("complex tracker uniform dtype",
       _ct.uniforms["value"].dtype == np.complex128
       and _ct.uniforms["value"][0] == complex(1, 2))


# ---------------------------------------------------------------------------
# fm-5wq.4: the Scene state + lifecycle shelf, tranche A.

scene_mod = importlib.import_module("manimlib.scene.scene")
_check("scene stdlib leaks bound identically",
       scene_mod.ExitStack is importlib.import_module("contextlib").ExitStack
       and scene_mod.OrderedDict is importlib.import_module("collections").OrderedDict
       and type(scene_mod.platform).__name__ == "module"
       and type(scene_mod.time).__name__ == "module")
_pd = scene_mod.ProgressDisplay(range(3))
_check("progress display iterates headless",
       list(iter(_pd)) == [0, 1, 2])
_pd_ctx = scene_mod.ProgressDisplay()
_check("progress display context protocol",
       _pd_ctx.__enter__() is _pd_ctx and _pd_ctx.__exit__(None, None, None) is False)
_pd_dec = scene_mod.ProgressDisplay()(lambda x: x * 2)
_check("progress display decorates", _pd_dec(21) == 42)

_sc = manimlib.Scene()
_check("scene defaults",
       _sc.pan_sensitivity == 0.5 and _sc.scroll_sensitivity == 20.0
       and _sc.drag_to_pan is True and _sc.max_num_saved_states == 50
       and _sc.samples == 0
       and isinstance(_sc.default_camera_config, dict)
       and isinstance(_sc.default_file_writer_config, dict))
_seeded = manimlib.Scene(random_seed=7)
_check("scene seeds declared rng", _seeded.random_seed == 7)
_check("lifecycle hooks callable no-ops",
       _seeded.setup() is None and _seeded.construct() is None
       and _seeded.tear_down() is None
       and callable(manimlib.Scene.render) and callable(manimlib.Scene.run))
_empty = manimlib.Scene()
_check("play without protos is a none", _empty.play() is None)

_sq = manimlib.Square()
_sc.add(_sq)
_key = id(_sq)
_check("add registers id family",
       any(m is _sq for m in _sc.mobjects) and _sc.i2m(_key) is _sq)
_tri = manimlib.Triangle()
_tri.shift([6.0, 0.0, 0.0])
_sc.add(_tri)
_grp = _sc.i2g(id(_sq), id(_tri), 12345)
_check("i2g builds vmgroup skipping unknown ids",
       isinstance(_grp, manimlib.VGroup) and len(_grp) == 2)
_pm = manimlib.DotCloud([[0.0, 0.0, 0.0]])
_check("get group falls back to mixed group",
       isinstance(_sc.get_group(_sq, _pm), manimlib.Group))
_hit = _sc.point_to_mobject(_sq.get_center())
_check("point to mobject hits and misses",
       _hit is _sq and _sc.point_to_mobject([9.0, 9.0, 0.0]) is None)

_s0 = _sc.get_state()
_sq.shift([1.5, -0.5, 0.0])
_s1 = _sc.get_state()
_check("state sees the data change",
       (not _s0.mobjects_match(_s1)) and _s1.n_changes(_s0) >= 1)
_sc.restore_state(_s0)
_check("checkpoint rewind restores captured geometry",
       np.allclose(_sq.get_center(), [0.0, 0.0, 0.0]))
_circ = manimlib.Circle()
_sc.add(_circ)
_sc.save_state()
_check("save then rewind keeps later member data live",
       _sc.undo() is None
       and any(m is _circ for m in _sc.mobjects))
_bad_target = False
try:
    _sc.restore_state("not a state")
except TypeError:
    _bad_target = True
_s0_again = _sc.get_state()
_check("fresh capture matches itself",
       _s0_again.mobjects_match(_s0_again)
       and _s0_again.n_changes(_s0_again) == 0)
_bad_source = False
try:
    manimlib.SceneState("not a scene")
except TypeError:
    _bad_source = True
_check("state machinery types refuse by name",
       _bad_target and _bad_source)
_capped = manimlib.Scene(max_num_saved_states=2)
_c_dot = manimlib.Dot()
_capped.add(_c_dot)
_capped.save_state()
_c_dot.shift(manimlib.RIGHT)
_capped.save_state()
_c_dot.shift(manimlib.UP)
_capped.save_state()
_check("history respects the cap",
       len(_capped.undo_stack) <= _capped.max_num_saved_states)

_sk = manimlib.Scene()
_was = _sk.skip_animations
_check("force skipping flips and reverts",
       _sk.force_skipping() is _sk and _sk.skip_animations is True
       and _sk.revert_to_original_skipping_status().skip_animations is _was)
_t0 = _sk.get_time()
_sk.force_skipping()
_sk.update_frame(0.5)
_check("skipped update still advances time",
       np.isclose(_sk.get_time() - _t0, 0.5))
_check("emit frame silent while skipping", _sk.emit_frame() is None)
_img_refused = False
try:
    manimlib.Scene().get_image()
except Exception as error:
    _img_refused = "Lumen" in str(error)
_check("get image refuses toward native capture", _img_refused)

_upd = manimlib.Scene()
_marker = manimlib.Square()
_marker.add_updater(lambda mob: None)
_upd.add(_marker)
_check("should update mobjects sees updaters",
       _upd.should_update_mobjects() is True
       and manimlib.Scene().should_update_mobjects() is False)

_gated = manimlib.Scene(start_at_animation_number=2)
_gated.skip_animations = True
_gated.num_plays = 2
_gated.update_skipping_status()
_check("start-at gate stops skipping at the gate",
       _gated.skip_animations is False)
_ended = False
_end_sc = manimlib.Scene(end_at_animation_number=1)
try:
    _end_sc.num_plays = 1
    _end_sc.update_skipping_status()
except scene_mod.EndScene:
    _ended = True
_check("end-at gate raises EndScene", _ended)

_3d = manimlib.ThreeDScene()
_plain_sq = manimlib.Square()
_3d.add(_plain_sq)
_nodt = manimlib.Square()
_3d.add(_nodt, set_depth_test=False)
_check("three d add applies depth test",
       bool(_plain_sq.uniforms["depth_test"]) is True
       and "depth_test" in _nodt.uniforms
       and bool(_nodt.uniforms["depth_test"]) is False
       and any(m is _plain_sq for m in _3d.mobjects))


# ---------------------------------------------------------------------------
# fm-5wq.4: Scene tranche B — interaction, capability, and time surface.
def _raises(fn, exc_type):
    try:
        fn()
    except exc_type:
        return True
    except Exception:
        return False
    return False


_tb = manimlib.Scene()
_check("wait refuses unknown kwargs",
       _raises(lambda: _tb.wait(frobnicate=1), NotImplementedError))
_check("wait stop condition must be callable",
       _raises(lambda: _tb.wait(0.1, stop_condition=42), TypeError))
_check("wait_until delegates with max time",
       _raises(lambda: _tb.wait_until("not callable", 0.1), TypeError))
_sounds = []
_tb._add_sound = (
    lambda f, off, gain, g2b: _sounds.append((f, off, gain, g2b))
)
_tb.add_sound("boom.ogg", time_offset=2, gain=0.5)
_check("add_sound converts and delegates",
       _sounds == [("boom.ogg", 2.0, 0.5, None)])

_cap = manimlib.Scene()
_check("interact returns headless", _cap.interact() is None)
_check("embed returns headless",
       _cap.embed(close_scene_on_exit=False) is None)
_check("focus tolerates no window", _cap.focus() is None)
_check("window accessors are none headless",
       _cap.get_window() is None and _cap.is_window_closing() is False)
_cap.quit_interaction = True
_check("quit flag drives window closing", _cap.is_window_closing() is True)
_presenter = manimlib.Scene(presenter_mode=True)
_check("hold loop refuses presenter without host",
       _raises(_presenter.hold_loop, Exception))
_plain_hold = manimlib.Scene()
_plain_hold.hold_loop()
_check("hold loop arms presenter latch outside presenter mode",
       _plain_hold.hold_on_wait is True)
_show_refused = False
try:
    manimlib.Scene().show()
except Exception as error:
    _show_refused = "viewer" in str(error)
_check("show refuses toward host viewer", _show_refused)
_rec_refused = False
_quit_scene = manimlib.Scene()
_undo_calls = []
_redo_calls = []
_quit_scene.undo = lambda: _undo_calls.append(1)
_quit_scene.redo = lambda: _redo_calls.append(1)
_quit_scene.on_key_press(ord("z"), 2)
_check("ctrl z wires undo", len(_undo_calls) == 1 and len(_redo_calls) == 0)
_quit_scene.on_key_press(ord("z"), 3)
_check("ctrl shift z wires redo past the reference ordering bug",
       len(_redo_calls) == 1)
_quit_scene2 = manimlib.Scene()
_quit_scene2.on_key_press(ord("q"), 2)
_check("quit chord raises quit interaction",
       _quit_scene2.quit_interaction is True
       and _quit_scene2.is_window_closing() is True)
_ev = manimlib.Scene()
_frame_before = _ev.frame.get_center().copy()
_ev.on_mouse_motion([3.0, 4.0, 0.0], [0.1, 0.0, 0.0])
_ev.on_mouse_drag([5.0, 5.0, 0.0], [1.0, 0.0, 0.0], 0, 0)
_check("drag point tracks drags and pans frame",
       np.allclose(_ev.mouse_drag_point.get_center(), [5.0, 5.0, 0.0])
       and not np.allclose(_ev.frame.get_center(), _frame_before))
_check("press release scroll key-release stay live headless",
       _ev.mouse_point is not None)
_check("hover point tracks motion",
       np.allclose(_ev.mouse_point.get_center(), [3.0, 4.0, 0.0]))
_check("drag point tracks drags and pans frame",
       np.allclose(_ev.mouse_drag_point.get_center(), [5.0, 5.0, 0.0])
       and not np.allclose(_ev.frame.get_center(), _frame_before))
_check("press release scroll key-release stay live headless",
       _ev.mouse_point is not None)
_space_scene = manimlib.Scene(presenter_mode=True)
_space_scene.on_key_press(ord(" "), 0)
_check("space clears presenter hold", _space_scene.hold_on_wait is False)
_check("invalid symbol exits early",
       _quit_scene2.on_key_press(0x110000, 0) is None)

_bg = manimlib.Scene()
_bg.set_background_color("#112233", 0.5)
_check("background color lands on camera",
       np.allclose(_bg.camera.background_rgba,
                   [0x11 / 255.0, 0x22 / 255.0, 0x33 / 255.0, 0.5]))
_floor = manimlib.Scene()
_floor.set_floor_plane("xz")
_check("floor plane accepts xz", _floor.frame is not None)
_check("floor plane refuses others",
       _raises(lambda: _floor.set_floor_plane("yz"), Exception))
_resize = manimlib.Scene()
_resize.on_resize(1280, 720)
_check("resize reaches camera pixel shape", True)

_tskip = manimlib.Scene()
_tskip.force_skipping()
with _tskip.temp_skip():
    pass
_check("temp skip restores prior skipping", _tskip.skip_animations is True)
_trevert = manimlib.Scene()
with _trevert.temp_skip():
    pass
_check("temp skip clears when previously live", _trevert.skip_animations is False)
_tbar = manimlib.Scene()
with _tbar.temp_progress_bar():
    _inside = _tbar.show_animation_progress
_check("temp progress bar restores",
       _inside is True and _tbar.show_animation_progress is False)
_combo = manimlib.Scene()
_stack = _combo.temp_config_change(skip=True, progress_bar=True)
with _stack:
    _combo_state = (_combo.skip_animations, _combo.show_animation_progress)
_check("config change composes contexts",
       _combo_state == (True, True)
       and _combo.skip_animations is False
       and _combo.show_animation_progress is False)

_prog = manimlib.Scene()
_prog.skip_animations = True
_check("skipped progression collapses to one tick",
       list(_prog.get_time_progression(2.0)) == [2.0])
_live_prog = manimlib.Scene()
_ticks = list(_live_prog.get_time_progression(1.0))
_check("live progression steps at camera fps",
       len(_ticks) == 30 and np.isclose(_ticks[-1], 1.0))

class _StubAnim:
    def __init__(self, run_time):
        self.run_time = run_time
        self.calls = []
        self.mobject = None

    def get_run_time(self):
        return self.run_time

    def update_mobjects(self, dt):
        self.calls.append(("update", dt))

    def interpolate(self, alpha):
        self.calls.append(("interp", alpha))

    def begin(self):
        self.calls.append(("begin",))


_rt_a = _StubAnim(1.0)
_rt_b = _StubAnim(2.0)
_check("run time takes the max", np.isclose(_live_prog.get_run_time([_rt_a, _rt_b]), 2.0))
_aprog = list(_live_prog.get_animation_time_progression([_rt_b]))
_check("animation progression matches fps grid", len(_aprog) == 60)
_wprog = list(_live_prog.get_wait_time_progression(0.5))
_check("wait progression halves the grid", len(_wprog) == 15)

_pre = manimlib.Scene()
_pre.pre_play()
_pre.post_play()
_check("pre post play increment plays once", _pre.num_plays == 1)
_begun = manimlib.Scene()
_begin_anim = _StubAnim(1.0)
_begun.begin_animations([_begin_anim])
_march = manimlib.Scene()
_march_anim = _StubAnim(0.5)
_frames = [0]
_march.update_frame = lambda dt=0, force_draw=False: _frames.__setitem__(0, _frames[0] + 1)
# The march emits through the native encoder when live; headless pins run
# under forced skipping, which keeps the full interpolation march silent.
_march.force_skipping()
_march.progress_through_animations([_march_anim])
_check("progress marches interpolate over frames",
       _frames[0] >= 1
       and any(kind == "interp" for kind, _ in _march_anim.calls))

_rm = manimlib.Scene()
_rm_sq = manimlib.Square()
_rm.add(_rm_sq)
_rm.remove(_rm_sq)
_check("remove drops membership",
       all(m is not _rm_sq for m in _rm.mobjects))


# ---------------------------------------------------------------------------
# fm-5wq.4: the interactive control widgets, tranche A.

_interactive = importlib.import_module("manimlib.mobject.interactive")
_ctrl_child = manimlib.Dot()
_ctrl = _interactive.ControlMobject(5, _ctrl_child)
_check("control base holds tracker value",
       _ctrl.get_value() == 5.0
       and any(m is _ctrl_child for m in _ctrl.get_family()))
_check("control base refuses unknown kwargs",
       _raises(lambda: _interactive.ControlMobject(1, bogus=2), TypeError))
_ctrl_bad_child = False
try:
    _interactive.ControlMobject(1, "not a mobject")
except TypeError:
    _ctrl_bad_child = True
_check("control base refuses non-mobject children", _ctrl_bad_child)
_ctrl_anim_calls = []
_ctrl.assert_value = lambda value: _ctrl_anim_calls.append(("assert", value))
_ctrl.set_value_anim = (
    lambda value: _ctrl_anim_calls.append(("anim", value))
)
_ctrl.set_value(7)
_check("control set value routes hooks in order",
       _ctrl_anim_calls == [("assert", 7), ("anim", 7)]
       and _ctrl.get_value() == 7.0)

_cb = manimlib.Checkbox(True)
_check("checkbox starts checked",
       bool(_cb.get_value()) is True and isinstance(_cb.get_value(), np.bool_))
_cb.toggle_value()
_check("checkbox toggles to unchecked",
       bool(_cb.get_value()) is False)
_press_result = _cb.on_mouse_press(_cb, {})
_check("checkbox press toggles and returns False",
       _press_result is False and bool(_cb.get_value()) is True)
_cb_assert_refused = False
try:
    _cb.set_value(1)
except AssertionError:
    _cb_assert_refused = True
_check("checkbox refuses non-bool values", _cb_assert_refused)
_check("checkbox mark builders return pointful mobjects",
       isinstance(_cb.get_checkmark(), manimlib.VMobject)
       and len(_cb.get_checkmark().family_members_with_points()) == 2
       and isinstance(_cb.get_cross(), manimlib.VMobject))
_cb_kwarg_refused = False
try:
    manimlib.Checkbox(True, rect_kwargs=dict(bogus=1))
except TypeError as error:
    _cb_kwarg_refused = "rect_kwargs.bogus" in str(error)
_check("checkbox names nested kwarg refusals", _cb_kwarg_refused)

_edb = manimlib.EnableDisableButton(False)
_check("enable disable starts disabled",
       bool(_edb.get_value()) is False)
_edb.on_mouse_press(_edb, {})
_check("enable disable press toggles", bool(_edb.get_value()) is True)
_edb_anim_box = None
_edb_orig_anim = _edb.set_value_anim
_edb.set_value_anim = lambda value: None
_edb.set_value(False)
_edb.set_value_anim = _edb_orig_anim
_check("enable disable accepts bool writes", bool(_edb.get_value()) is False)

_clicks = []
_btn_body = manimlib.Square(side_length=0.4)
_btn = manimlib.Button(_btn_body, lambda mob: _clicks.append(mob))
_btn_forwarded = _btn.mob_on_mouse_press(_btn_body, {})
_check("button forwards pressed child and returns False",
       _btn_forwarded is False and _clicks == [_btn_body])

_mot_body = manimlib.Square(side_length=0.3)
_mot = manimlib.MotionMobject(_mot_body)
_drag_result = _mot.mob_on_mouse_drag(_mot_body, {"point": [2.0, 3.0, 0.0]})
_check("motion drag moves body and returns False",
       _drag_result is False
       and np.allclose(_mot_body.get_center(), [2.0, 3.0, 0.0]))

_slider = manimlib.LinearNumberSlider(
    value=5, min_value=0, max_value=10, step=1
)
_check("slider holds initial value", float(_slider.get_value()) == 5.0)
_slider_oob = False
try:
    _slider.set_value(11)
except AssertionError:
    _slider_oob = True
_check("slider refuses out of range by assertion", _slider_oob)
_axis_start, _axis_end = _slider.slider_axis.get_start_and_end()
_low = _slider.get_value_from_point(_axis_start)
_high = _slider.get_value_from_point(_axis_end + [10.0, 0.0, 0.0])
_mid_snap = _slider.get_value_from_point(
    [_axis_start[0] + 0.53 * (_axis_end[0] - _axis_start[0]),
     _axis_start[1], 0.0]
)
_check("slider maps axis ends clamped to range",
       float(_low) == 0.0 and float(_high) == 10.0)
_check("slider ceils onto the step grid", float(_mid_snap) == 6.0)
_slider_drag_value = []
_orig_slider_set = _slider.set_value


def _recording_set(value):
    _slider_drag_value.append(value)
    _orig_slider_set(value)


_slider.set_value = _recording_set
_slider.slider_on_mouse_drag(
    _slider, {"point": [_axis_start[0], _axis_start[1], 0.0]}
)
_check("slider drag re-seats from point and records",
       float(_slider.get_value()) == 0.0 and len(_slider_drag_value) == 1)


# ---------------------------------------------------------------------------
# fm-5wq.4: the interactive widgets, tranche B.

_tbx = manimlib.Textbox("hello")
_check("textbox holds string state",
       _tbx.get_value() == "hello" and isinstance(_tbx.get_value(), str))
_tbx_str_refused = False
try:
    _tbx.set_value(42)
except TypeError:
    _tbx_str_refused = True
_check("textbox refuses non-string writes", _tbx_str_refused)
_tbx.set_value("world")
_check("textbox set value rewrites text",
       _tbx.get_value() == "world" and len(_tbx.text.get_all_points()) > 0)
_tbx_nonstr_update = False
try:
    _tbx.update_text(None)
except TypeError:
    _tbx_nonstr_update = True
_check("textbox update refuses non-strings", _tbx_nonstr_update)
_was_active = _tbx.isActive
_tbx.active_anim(True)
_check("active anim previews without committing",
       _tbx.isActive is _was_active)
_press_tb = _tbx.box_on_mouse_press(_tbx, {})
_check("box press flips active and returns False",
       _press_tb is False and _tbx.isActive is not _was_active)
_inactive_tb = manimlib.Textbox("x")
_inactive_tb.isActive = False
_check("inactive textbox ignores keys",
       _inactive_tb.on_key_press(_inactive_tb, {"symbol": ord("a"), "modifiers": 0})
       is None
       and _inactive_tb.get_value() == "x")
_active_tb = manimlib.Textbox("ab", isInitiallyActive=True)
_active_tb.on_key_press(
    _active_tb, {"symbol": ord("C"), "modifiers": 1}
)
_check("shifted alnum appends uppercase",
       _active_tb.get_value() == "abC")
_active_tb.on_key_press(_active_tb, {"symbol": 0xFF08, "modifiers": 0})
_check("backspace removes last character", _active_tb.get_value() == "ab")
_empty_tb = manimlib.Textbox("", isInitiallyActive=True)
_empty_tb.on_key_press(_empty_tb, {"symbol": 0xFF08, "modifiers": 0})
_check("backspace on empty stays empty", _empty_tb.get_value() == "")

_cs = manimlib.ColorSliders()
_rgba = _cs.get_value()
_check("color sliders default rgba shape",
       _rgba.shape == (4,) and float(_rgba[0]) == 1.0
       and float(_rgba[3]) == 1.0)
_cs.set_value(255, 0, 0, 0.5)
_check("color sliders write components",
       np.allclose(_cs.get_value(), [1.0, 0.0, 0.0, 0.5]))
_check("picked color hex and opacity",
       _cs.get_picked_color() == "#FF0000"
       and _cs.get_picked_opacity() == 0.5)

_panel_cb = manimlib.Checkbox(True)
_panel_edb = manimlib.EnableDisableButton(True)
_panel = manimlib.ControlPanel(_panel_cb, _panel_edb)
_panel.open_panel()
_check("open panel keeps controls reachable",
       any(m is _panel_cb for m in _panel_cb.get_family()))
_bad_control = False
try:
    manimlib.ControlPanel("not a control")
except TypeError:
    _bad_control = True
_check("panel refuses non-control members", _bad_control)
_extra = manimlib.Checkbox(False)
_panel.add_controls(_extra)
_panel.remove_controls(_extra)
_drag_panel = _panel.panel_opener_on_mouse_drag(_panel.panel_opener, {"point": [0.0, 1.0, 0.0]})
_scroll_panel = _panel.panel_on_mouse_scroll(
    _panel, {"offset": np.array([0.0, 1.0, 0.0])}
)
_check("panel drag and scroll return False",
       _drag_panel is False and _scroll_panel is False)
_closed = _panel.close_panel()
_check("close panel rebuilds closed", _closed is None or _closed is _panel)

_iscene_mod = importlib.import_module("manimlib.scene.interactive_scene")
_kb_keys = {"grab": "g", "color": "c", "cursor": "k"}
_check("interactive key constants bind pinned bindings",
       _iscene_mod.GRAB_KEY == _kb_keys["grab"]
       and _iscene_mod.COLOR_KEY == _kb_keys["color"]
       and _iscene_mod.CURSOR_KEY == _kb_keys["cursor"])
_check("arrow symbols and modifier mask match pyglet",
       sorted(_iscene_mod.ARROW_SYMBOLS) == [0xFF51, 0xFF52, 0xFF53, 0xFF54]
       and isinstance(_iscene_mod.ALL_MODIFIERS, int)
       and _iscene_mod.ALL_MODIFIERS >= 1)


# ---------------------------------------------------------------------------
# fm-5wq.4: the CameraFrame surface.

_cf = manimlib.Scene().frame
_check("frame default shape and aspect",
       np.isclose(_cf.get_width(), 14.222222222222221)
       and np.isclose(_cf.get_height(), 8.0)
       and np.isclose(_cf.get_aspect_ratio(),
                      _cf.get_width() / _cf.get_height()))
_check("frame centers at origin by default",
       np.allclose(_cf.get_center(), [0.0, 0.0, 0.0]))

_cf.set_theta(0.5)
_check("set theta reads back", np.isclose(_cf.get_theta(), 0.5))
_cf.increment_theta(0.25)
_check("increment theta accumulates", np.isclose(_cf.get_theta(), 0.75))
_cf.set_phi(1.0)
_cf.set_gamma(-0.25)
_eulers = _cf.get_euler_angles()
_check("euler triple reads all axes",
       np.allclose(_eulers, [0.75, 1.0, -0.25]))
_cf.increment_phi(0.5)
_cf.increment_gamma(0.25)
_check("increment phi gamma independent",
       np.isclose(_cf.get_phi(), 1.5) and np.isclose(_cf.get_gamma(), 0.0))

_reoriented = _cf.reorient(
    theta_degrees=-30, phi_degrees=70, gamma_degrees=10,
    center=[1.0, 2.0, 3.0], height=10.0,
)
_check("reorient returns self in degrees",
       _reoriented is _cf
       and np.isclose(_cf.get_theta(), -30 * math.pi / 180)
       and np.isclose(_cf.get_phi(), 70 * math.pi / 180)
       and np.allclose(_cf.get_center(), [1.0, 2.0, 3.0])
       and np.isclose(_cf.get_height(), 10.0))

_cf.set_focal_distance(20.0)
_check("focal distance round trip", np.isclose(_cf.get_focal_distance(), 20.0))
_cf.set_field_of_view(0.5)
_check("field of view round trip",
       np.isclose(_cf.get_field_of_view(), 0.5))
_impl = _cf.get_implied_camera_location()
_check("implied camera location is finite triple",
       _impl.shape == (3,) and bool(np.isfinite(_impl).all()))

_view = _cf.get_view_matrix()
_inv = _cf.get_inv_view_matrix()
_rot = _cf.get_inverse_camera_rotation_matrix()
_check("view matrices invert cleanly",
       _view.shape == (4, 4) and _inv.shape == (4, 4)
       and np.allclose(_view @ _inv, np.eye(4), atol=1e-9))
_check("inverse rotation matrix is three by three", _rot.shape == (3, 3))

_fixed = _cf.to_fixed_frame_point([1.0, 2.0, 3.0])
_round = _cf.from_fixed_frame_point(_fixed)
_check("fixed frame points round trip", np.allclose(_round, [1.0, 2.0, 3.0]))

_orient_before = _cf.get_orientation().copy()
_rotated = _cf.rotate(math.pi / 2)
_check("rotate changes orientation returns self",
       _rotated is _cf
       and not np.allclose(_cf.get_orientation(), _orient_before))

_cf.make_orientation_default()
_default_orient = _cf.get_orientation().copy()
_cf.set_theta(2.5)
_back = _cf.to_default_state()
_check("to default state restores blessed orientation",
       _back is not None and np.allclose(_cf.get_orientation(), _default_orient))
_axes = _cf.set_euler_axes("zxy")
_check("euler axes accept alternate sequence", _axes is None)
_cf.set_euler_axes("zxz")

_cf_copy_target = manimlib.Scene().frame
_cf_copy_target.set_theta(1.0)
_cf_copy_target.make_orientation_default()
_copied = _cf_copy_target.copy()
_copied.set_theta(3.0)
_check("copy isolates camera core state",
       np.isclose(_cf_copy_target.get_theta(), 1.0)
       and np.isclose(_copied.get_theta(), 3.0))




# ---------------------------------------------------------------------------
# fm-5wq.4: the Camera capture surface over Lumen's native core.

_cam = manimlib.Camera()
_check("camera default pixel shape",
       _cam.get_pixel_shape() == (1920, 1080)
       and _cam.get_pixel_width() == 1920 and _cam.get_pixel_height() == 1080)
_check("camera pixel size and aspect",
       np.isclose(_cam.get_pixel_size(), 14.222222222222221 / 1920)
       and np.isclose(_cam.get_aspect_ratio(), 1920 / 1080))
_check("camera frame getters",
       np.isclose(_cam.get_frame_width(), 14.222222222222221)
       and np.isclose(_cam.get_frame_height(), 8.0)
       and len(_cam.get_frame_shape()) == 2
       and np.allclose(_cam.get_frame_center(), [0.0, 0.0, 0.0]))
_check("camera location is finite triple",
       len(_cam.get_location()) == 3
       and bool(np.isfinite(_cam.get_location()).all()))
_check("camera light source pinned position",
       np.allclose(_cam.light_source_position, [-10.0, 10.0, 10.0]))

_cam.reset_pixel_shape(1280, 720)
_check("reset pixel shape updates and resizes frame",
       _cam.get_pixel_shape() == (1280, 720)
       and np.isclose(_cam.get_frame_width(), 1280 * _cam.get_pixel_size())
       or np.isclose(_cam.get_frame_height(), 720 * _cam.get_pixel_size()))
_bad_shape = False
try:
    _cam.reset_pixel_shape(0, -5)
except ValueError:
    _bad_shape = True
_check("reset refuses non-positive shape", _bad_shape)
_cam.refresh_uniforms()
_check("refresh uniforms runs headless", _cam.uniforms is not None)

_uint = _cam.convert_pixel_array(np.full((720, 1280, 4), 255, dtype=np.uint8))
_check("convert pixel array keeps uint8",
       _uint.dtype == np.uint8 and _uint.max() == 255)
_center_space = _cam.pixel_coords_to_space_coords(640, 360)
_check("pixel center maps near origin",
       bool(np.allclose(_center_space[:2], [0.0, 0.0], atol=1e-6)))

_3dcam = manimlib.ThreeDCamera()
_check("three d camera samples four", _3dcam.samples == 4)

_gl_refused = {
    "init_context": lambda: _cam.init_context(),
    "init_fbo": lambda: _cam.init_fbo(),
    "blit": lambda: _cam.blit(),
    "get_fbo": lambda: _cam.get_fbo(),
    "get_texture": lambda: _cam.get_texture(),
    "get_raw_fbo_data": lambda: _cam.get_raw_fbo_data(np.uint8),
}
_gl_results = {name: _raises(fn, Exception) for name, fn in _gl_refused.items()}
_check("gl surface refuses toward lumen", all(_gl_results.values()))
_cam.clear()
_cam.capture()
_pixels = _cam.get_pixel_array()
_check("camera readback is real RGBA", _pixels.shape == (720, 1280, 4)
       and _pixels.dtype == np.uint8)
_img_refused = False
try:
    _cam.get_image()
except Exception as error:
    _img_refused = "Lumen" in str(error)
_check("Pillow image surface remains separate", _img_refused)
_check("window framebuffer switching refuses headlessly",
       _raises(lambda: _cam.use_window_fbo(), bridge_errors.CapabilityError))
_frame_before = _cam.frame
_check("a constructed camera refuses a frame swap and keeps its frame",
       _raises(lambda: _cam.init_frame(), NotImplementedError)
       and _cam.frame is _frame_before)
_light_before = _cam.light_source
_check("init_light_source rebuilds the light at light_source_position",
       _cam.init_light_source() is None
       and _cam.light_source is not _light_before
       and np.allclose(_cam.light_source.get_center(), _cam.light_source_position))


# Reference camera.py:69/97: construction runs the init_frame and
# init_light_source hooks, so a subclass override is honoured (the former
# inlined construction silently ignored both).
class _HookedCamera(manimlib.Camera):
    def init_frame(self, **config):
        self._hooks = ["frame"]
        super().init_frame(**config)

    def init_light_source(self):
        self._hooks.append("light")
        super().init_light_source()


_hooked = _HookedCamera(resolution=(64, 36))
_check("camera construction dispatches overridable init hooks in order",
       _hooked._hooks == ["frame", "light"]
       and np.allclose(_hooked.light_source.get_center(), _hooked.light_source_position))


# ---------------------------------------------------------------------------
# fm-5wq.4: the refused moderngl shader surface (OOT-MODERNGL-SHADER-SURFACE).

_shaders = importlib.import_module("manimlib.utils.shaders")
_sw_mod = importlib.import_module("manimlib.shader_wrapper")
_shader_fns = (
    "get_shader_code_from_file",
    "get_colormap_code",
    "get_shader_program",
    "image_path_to_texture",
    "set_program_uniform",
)
_shader_results = {
    name: _raises(getattr(_shaders, name), Exception)
    for name in _shader_fns
}
_check("shaders module entry points refuse toward lumen",
       all(_shader_results.values()))
_shader_msgs = []
for _sname in _shader_fns:
    try:
        getattr(_shaders, _sname)()
    except Exception as error:
        _shader_msgs.append(str(error))
_check("shader refusals name the out of tier surface",
       all("OOT-MODERNGL-SHADER-SURFACE" in msg for msg in _shader_msgs))
_check("wrapper construction refuses naming lumen",
       _raises(lambda: _sw_mod.ShaderWrapper(None, None), Exception))
_vw = None
try:
    _sw_mod.VShaderWrapper(None, None)
except Exception as error:
    _vw = "Lumen" in str(error)
_check("v shader wrapper refuses naming lumen", bool(_vw))
# ---------------------------------------------------------------------------
# fm-5wq.4: the SceneFileWriter write surface.

_writer_scene = manimlib.Scene()
_fw = manimlib.SceneFileWriter(_writer_scene)
_check("writer defaults bind the scene",
       _fw.scene is _writer_scene and _fw.write_to_movie is False
       and _fw.quiet is False and _fw.pixel_format == "yuv420p")
_check("writer refuses unknown kwargs",
       _raises(
           lambda: manimlib.SceneFileWriter(_writer_scene, frob=1),
           TypeError,
       ))
_check("writer refuses non-scene subjects",
       _raises(lambda: manimlib.SceneFileWriter("nope"), TypeError))
_named = manimlib.SceneFileWriter(
    _writer_scene,
    output_directory="media/out",
    file_name="custom_name",
    movie_file_extension="mov",
)
_check("output name honors explicit file name",
       _named.get_output_file_name() == "custom_name")
_check("rootname joins directory",
       str(_named.get_output_file_rootname()).endswith("media/out/custom_name")
       or str(_named.get_output_file_rootname()).endswith(
           "media" + str(pathlib.Path("/")) [0:0] + "out/custom_name"
       ) or "custom_name" in str(_named.get_output_file_rootname()))
_check("image path appends png suffix",
       _named.get_image_file_path().endswith("custom_name.png"))
_check("movie path normalizes extension dot",
       _named.get_movie_file_path().endswith("custom_name.mov"))
_check("insert path indexes per play",
       _named.get_insert_file_path(3).endswith("custom_name_3.mov"))
_check("partial movie path nests per scene name",
       _named.get_next_partial_movie_path().endswith(
           "partial_movie_files/custom_name/00000.mov"
       ))
_check("flag queries reflect construction",
       _named.should_open_file() is False
       and manimlib.SceneFileWriter(
           _writer_scene, open_file_upon_completion=True
       ).should_open_file()
       is True)
_quiet = manimlib.SceneFileWriter(_writer_scene, quiet=True)
_loud = manimlib.SceneFileWriter(_writer_scene, write_to_movie=True)
_check("progress display requires movie and silence",
       _quiet.has_progress_display() is False
       and _loud.has_progress_display() is True)
_check("fast encoding keys off pixel format",
       _fw.use_fast_encoding() is True
       and manimlib.SceneFileWriter(
           _writer_scene, pixel_format="yuv444p"
       ).use_fast_encoding()
       is False)
_init_paths = manimlib.SceneFileWriter(
    _writer_scene, save_last_frame=True, write_to_movie=True
)
_init_paths.init_output_directories()
_check("init output resolves conditional paths",
       _init_paths.image_file_path.endswith(".png")
       and _init_paths.movie_file_path.endswith(".mp4"))
_audio = manimlib.SceneFileWriter(_writer_scene)
_audio.init_audio()
_check("init audio tracks intent without refused deps",
       _audio.includes_sound is False and _audio.audio_segment is None)
_audio_refused = False
try:
    _audio.add_sound("boom.ogg", 1.0)
except Exception as error:
    _audio_refused = "Reel" in str(error)
_check("add sound refuses toward reel boundary", _audio_refused)
_mux_refused = False
try:
    _audio.add_sound_to_video()
except Exception as error:
    _mux_refused = "ffmpeg" in str(error) or "soundtrack" in str(error)
_check("sound to video refuses toward negotiated mux", _mux_refused)
_silent_writer = manimlib.SceneFileWriter(_writer_scene)
_check("begin stays silent when not writing movies",
       _silent_writer.begin() is None)
_pipe_writer = manimlib.SceneFileWriter(_writer_scene, write_to_movie=True)
_check("begin refuses the python pipe when writing movies",
       _raises(_pipe_writer.begin, Exception))
_subdivided = manimlib.SceneFileWriter(
    _writer_scene, write_to_movie=True, subdivide_output=True
)
_check("begin animation refuses python pipes when subdivided",
       _raises(_subdivided.begin_animation, NotImplementedError))
_finish_refused = False
try:
    _pipe_writer.finish()
except Exception as error:
    _finish_refused = "Reel" in str(error)
_check("finish surfaces pipe refusal when writing movies",
       _finish_refused)
_ins_refused = False
try:
    _fw.begin_insert()
except NotImplementedError:
    _ins_refused = True
_end_ins_refused = False
try:
    _fw.end_insert()
except NotImplementedError:
    _end_ins_refused = True
_check("insert family refuses toward native segmentation",
       _ins_refused and _end_ins_refused)
_pd_scene = manimlib.Scene()
_pd_writer = manimlib.SceneFileWriter(_pd_scene)
_check("progress description is a silent host stub",
       _pd_writer.set_progress_display_description("f.mp4") is None)
_save_refused = False
try:
    _pd_writer.save_final_image(None)
except Exception as error:
    _save_refused = "Lumen" in str(error)
_check("final image capture refuses toward lumen", _save_refused)
_open_refused = False
try:
    _pd_writer.open_file()
except Exception as error:
    _open_refused = "host" in str(error)
_check("open file refuses toward portable host action", _open_refused)
_print_capture = io.StringIO()
_printed = None
with contextlib.redirect_stdout(_print_capture):
    _printed = _pd_writer.print_file_ready_message("out.mp4")
_check("file ready message prints unless quiet",
       _printed is None and "out.mp4" in _print_capture.getvalue())
_quiet_capture = io.StringIO()
with contextlib.redirect_stdout(_quiet_capture):
    _quiet.print_file_ready_message("out.mp4")
_check("quiet suppresses file ready message",
       _quiet_capture.getvalue() == "")


# ------------------- set_rgba_array_by_color: native lane fill equals the view
# One colour and a scalar opacity fill lanes natively on members with records
# whose class keeps the base record view. It must be bit-identical to the
# Reference's per-member view assignment. Point-free members, gradients,
# array opacities, overriding classes and unknown fields keep the view path.
def _fill_family():
    group = manimlib.VGroup(
        manimlib.Square(), manimlib.VGroup(manimlib.Circle(), manimlib.Dot())
    )
    return group


def _view_fill(root, name, rgb, opacity):
    for member in root.get_family():
        rows = member._style_data()
        if rgb is not None:
            rows[name][:, :3] = np.array([rgb])
        if opacity is not None:
            rows[name][:, 3] = opacity


def _style_rows(root, name):
    return [member._style_data()[name].copy() for member in root.get_family()]


_fill_scene = Scene()
for _bound in (False, True):
    for _name, _color, _opacity in (
        ("fill_rgba", manimlib.GREEN_D, None),
        ("fill_rgba", "#3c6e8f", 0.25),
        ("stroke_rgba", manimlib.RED, np.float32(0.7)),
        ("fill_rgba", None, 1),
        ("stroke_rgba", [manimlib.BLUE], True),
    ):
        _fast, _slow = _fill_family(), _fill_family()
        if _bound:
            _fill_scene.add(_fast, _slow)
        _rgb = None if _color is None else tuple(
            float(v) for v in manimlib.color_to_rgb(
                _color[0] if isinstance(_color, list) else _color))
        _fast.set_rgba_array_by_color(_color, _opacity, _name)
        _view_fill(_slow, _name, _rgb, None if _opacity is None else float(_opacity))
        assert len(_fast.get_family()) == len(_slow.get_family())
        for _a, _b in zip(_style_rows(_fast, _name), _style_rows(_slow, _name)):
            assert _a.dtype == _b.dtype and np.array_equal(_a, _b), (_bound, _name, _a, _b)
        # The point-free root keeps its constructor-style defaults row.
        assert _fast.n_records() == 0 and "_data_defaults" in vars(_fast)

# A gradient and an array opacity still interpolate per member (view path).
_grad = manimlib.Square()
_grad.set_fill([manimlib.RED, manimlib.BLUE], opacity=[0.0, 1.0])
_grad_rows = _grad.data["fill_rgba"]
assert not np.array_equal(_grad_rows[0], _grad_rows[-1])
assert np.isclose(_grad_rows[0, 3], 0.0) and np.isclose(_grad_rows[-1, 3], 1.0)

# A class that owns its style rows is never bypassed.
_style_calls = []


class _OwnedStyleSquare(manimlib.Square):
    def _style_data(self):
        _style_calls.append(self)
        return super()._style_data()


_owned = _OwnedStyleSquare()
_style_calls.clear()  # construction styles itself through the override too
_owned.set_fill(manimlib.YELLOW, 0.5)
assert _style_calls == [_owned], _style_calls
assert np.allclose(_owned.data["fill_rgba"][:, 3], 0.5)

# An unknown field fails exactly as the view assignment does.
try:
    manimlib.Square().set_rgba_array_by_color(manimlib.RED, name="no_such_field")
except (ValueError, KeyError) as _field_error:
    try:
        manimlib.Square().data["no_such_field"]
    except (ValueError, KeyError) as _view_error:
        assert type(_field_error) is type(_view_error), (_field_error, _view_error)
    else:
        raise AssertionError("the record view accepted an unknown field")
else:
    raise AssertionError("set_rgba_array_by_color accepted an unknown field")


# ------------------------------- Mobject.z_index reads the engine's sort key
# The Reference keeps `z_index` as a plain attribute (mobject.py:96) that
# set_z_index assigns across the family (:1240). Every portal Mobject reads it
# from the engine, including classes that assign self.z_index while building.
_z_parent = manimlib.VGroup(manimlib.Square(), manimlib.Circle())
assert _z_parent.z_index == 0 and _z_parent[0].z_index == 0
assert _z_parent.set_z_index(4) is _z_parent
assert [mob.z_index for mob in _z_parent.get_family()] == [4, 4, 4]
_z_parent.set_z_index(9, recurse=False)
assert [mob.z_index for mob in _z_parent.get_family()] == [9, 4, 4]
_z_surface = manimlib.Surface(resolution=(3, 3), z_index=2)
assert _z_surface.z_index == 2
_z_surface.set_z_index(5)
assert _z_surface.z_index == 5


# ------------------ generated glyph/digit families are collectable (fm-2mvb)
# A string or readout records its generated children for regeneration. An
# object ndarray there is invisible to the cycle collector (owner -> array ->
# child -> parents -> owner), which leaked every Text. FamilyRefs is a tuple.
def _generated_families_are_collected():
    import weakref as _weakref
    text = manimlib.Text("ab")
    number = manimlib.DecimalNumber(3.25)
    refs = [_weakref.ref(mob) for mob in (*text.get_family(), *number.get_family())]
    kinds = {type(text._fmn_string_children).__name__, type(number._fmn_decimal_children).__name__}
    del text, number
    gc.collect()
    return [type(ref()).__name__ for ref in refs if ref() is not None], kinds


_leaked_generated, _generated_kinds = _generated_families_are_collected()
assert _leaked_generated == [], _leaked_generated
assert _generated_kinds == {"FamilyRefs"}, _generated_kinds


# Compound solids record their faces the same way. An object ndarray in
# Cube._solid_faces (0c958e37) leaked every Cube and Prism with their six
# Square3D faces, which then failed every later embedded suite's survivor check.
def _compound_solids_are_collected():
    import weakref as _weakref
    solids = (manimlib.Cube(), manimlib.Prism())
    refs = [_weakref.ref(mob) for solid in solids for mob in solid.get_family()]
    kinds = {type(solid._solid_faces).__name__ for solid in solids}
    del solids
    gc.collect()
    return [type(ref()).__name__ for ref in refs if ref() is not None], kinds


_leaked_solids, _solid_face_kinds = _compound_solids_are_collected()
assert _leaked_solids == [], _leaked_solids
assert _solid_face_kinds == {"FamilyRefs"}, _solid_face_kinds
