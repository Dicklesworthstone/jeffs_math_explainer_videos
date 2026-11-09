"""Construction-only first-error guards for legacy native sampling loops.

The native bridge owns the first PyErr and result-publication decision. These
short-lived guards stop further authored calls without growing its traceback.
They do not replace any native sampling, smoothing or contouring algorithm.
"""
from __future__ import annotations

import math
import numbers

F32_MAX = 3.4028234663852886e38


class FirstFailure:
    def __init__(self, function, convert, sentinel):
        if not callable(function):
            raise TypeError("sampling function must be callable")
        self.function, self.convert, self.sentinel = function, convert, sentinel
        self.failed = False

    def __call__(self, *args):
        if self.failed:
            return self.sentinel
        try:
            return self.convert(self.function(*args))
        except BaseException:
            self.failed = True
            raise

    def close(self):
        self.failed = True
        self.function = self.convert = None


def point_sample(value, *, np, label):
    # Runs once per sampled point (7.5M calls in one corpus scene), so no
    # per-coordinate generators: unpack once, and one chained comparison per
    # coordinate, which is false for NaN, infinities and |v| > F32_MAX alike.
    array = np.asarray(value)
    if array.shape != (3,) or array.dtype.kind not in "biuf":
        raise ValueError(label + " must return exactly three real coordinates")
    x, y, z = array.tolist()
    x, y, z = float(x), float(y), float(z)
    if not (-F32_MAX <= x <= F32_MAX and -F32_MAX <= y <= F32_MAX and -F32_MAX <= z <= F32_MAX):
        raise ValueError(label + " must return finite f32-representable coordinates")
    return (x, y, z)


def scalar_sample(value, *, finite_record):
    # float() honors numeric __float__/__index__ conversion once, but unlike
    # the PyO3 numeric extraction also parses text. Do not admit numeric text.
    if isinstance(value, (str, bytes, bytearray)):
        raise TypeError("sampling function must return a real scalar, not text")
    if isinstance(value, numbers.Complex) and not isinstance(value, numbers.Real):
        raise TypeError("sampling function must return a real scalar, not a complex number")
    result = float(value)
    if finite_record and (not math.isfinite(result) or abs(result) > F32_MAX):
        raise ValueError("function graph must return finite f32-representable values; declare discontinuities")
    # Implicit fields deliberately retain non-finite undefined-region samples.
    # Chisel owns their subdivision policy; field values are not point records.
    return result
