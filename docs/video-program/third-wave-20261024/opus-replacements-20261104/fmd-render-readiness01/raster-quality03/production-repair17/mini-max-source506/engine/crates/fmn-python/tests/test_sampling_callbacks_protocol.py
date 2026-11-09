"""point_sample is exactly the previous per-point check, only cheaper (fm-5wq.31).

The previous implementation is kept here verbatim as the oracle. Pure Python
over NumPy; no native code is loaded.
"""
from __future__ import annotations

import importlib.util
import math
from pathlib import Path
import random
import struct
import unittest

import numpy as np

_PATH = Path(__file__).resolve().parents[1] / "python/fmn_python/sampling_callbacks.py"
_SPEC = importlib.util.spec_from_file_location("sampling_callbacks_under_test", _PATH)
callbacks = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(callbacks)
F32_MAX = callbacks.F32_MAX


def oracle(value, *, np, label):
    array = np.asarray(value)
    if array.shape != (3,) or array.dtype.kind not in "biuf":
        raise ValueError(label + " must return exactly three real coordinates")
    point = tuple(float(v) for v in array)
    if any(not math.isfinite(v) or abs(v) > F32_MAX for v in point):
        raise ValueError(label + " must return finite f32-representable coordinates")
    return point


def outcome(function, value):
    try:
        point = function(value, np=np, label="uv_func")
    except Exception as error:  # noqa: BLE001
        return ("raise", type(error).__name__, str(error))
    return ("value", tuple(type(v).__name__ for v in point),
            tuple(struct.pack("<d", v) for v in point))


def cases():
    specials = [0.0, -0.0, 1.5, -2.25, F32_MAX, -F32_MAX, math.nextafter(F32_MAX, math.inf),
                -math.nextafter(F32_MAX, math.inf), math.inf, -math.inf, math.nan, 5e-324, 1e308]
    rng = random.Random(20260929)
    for _ in range(400):
        yield [rng.choice(specials + [rng.uniform(-1e39, 1e39)]) for _ in range(3)]
    for dtype in (np.float16, np.float32, np.float64, np.int8, np.int64, np.uint64, np.bool_):
        yield np.array([1, 0, 1], dtype=dtype)
    yield np.array([2**63 - 1, 2**53 + 1, 7], dtype=np.int64)
    yield np.array([2**64 - 1, 2**63, 1], dtype=np.uint64)
    yield np.array([65504, -65504, 0.5], dtype=np.float16)
    yield (1, 2, 3)
    yield [True, False, True]
    yield (np.float32(0.1), 2, 3.5)
    yield np.array([1.0, 2.0, 3.0])[::-1]
    yield [1.0, 2.0]
    yield [[1.0, 2.0, 3.0]]
    yield ["1", "2", "3"]
    yield [1 + 2j, 0, 0]
    yield np.float64(3.0)
    yield [1.0, None, 2.0]
    yield np.zeros(3, dtype=object)


class PointSampleTests(unittest.TestCase):
    def test_identical_to_the_previous_check_on_every_case(self):
        compared = 0
        for value in cases():
            with self.subTest(value=repr(value)):
                self.assertEqual(outcome(callbacks.point_sample, value), outcome(oracle, value))
                compared += 1
        self.assertGreater(compared, 400)

    def test_returns_python_floats_and_names_the_label(self):
        self.assertEqual(callbacks.point_sample(np.array([1, 2, 3]), np=np, label="f"), (1.0, 2.0, 3.0))
        with self.assertRaisesRegex(ValueError, "^curve t_func must return finite"):
            callbacks.point_sample([0.0, math.nan, 0.0], np=np, label="curve t_func")


if __name__ == "__main__":
    unittest.main()
