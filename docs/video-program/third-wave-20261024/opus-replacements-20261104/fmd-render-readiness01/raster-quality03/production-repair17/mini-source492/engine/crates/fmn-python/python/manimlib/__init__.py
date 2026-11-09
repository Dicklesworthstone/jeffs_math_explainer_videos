"""FrankenManim's supported ``manimlib`` package surface.

The implementation lives in the private, CPython-versioned ``manimlib``
extension, including its shared animation initialization. This initializer
re-exports that module's public schema; geometry, rendering, and engine state
remain native.
"""

import sys as _sys

from fmn_python import (
    _ensure_exclusive_manimlib_namespace as _ensure_exclusive_manimlib_namespace,
)
from fmn_python.library_constructor_authority import (
    REFERENCE_CLASS_BY_RUST_HELPER as _CONTRACT_REFERENCE_CLASS_BY_RUST_HELPER,
    REFERENCE_MODULE_BY_RUST_HELPER as _CONTRACT_REFERENCE_MODULE_BY_RUST_HELPER,
)
from fmn_python.initialization import initialize as _initialize_portal

_ensure_exclusive_manimlib_namespace()

from . import manimlib as _native

# The extension initializes itself too. This idempotent call also makes the
# packaging boundary explicitly reject incomplete/failed native initialization.
_initialize_portal(_native)
del _initialize_portal


# The Rust API deliberately exposes ergonomic snake_case constructors, while
# the pinned Reference exports the corresponding CamelCase classes.  Treat a
# leak, missing class, or qualified-module identity split as an invalid wheel
# rather than silently widening, shrinking, or forking the compatibility API.
_REFERENCE_CLASS_BY_RUST_HELPER = {
    "group": "Group",
    "v_group": "VGroup",
    "vectorized_point": "VectorizedPoint",
    "small_dot": "SmallDot",
    "sector": "Sector",
    "vector": "Vector",
    "polyline": "Polyline",
    "triangle": "Triangle",
    "rounded_rectangle": "RoundedRectangle",
    "svg_mobject": "SVGMobject",
    "tangent_line": "TangentLine",
    "curved_arrow": "CurvedArrow",
    "curved_double_arrow": "CurvedDoubleArrow",
    "curves_as_submobjects": "CurvesAsSubmobjects",
    "dashed_vmobject": "DashedVMobject",
    "v_highlight": "VHighlight",
}
if _REFERENCE_CLASS_BY_RUST_HELPER != _CONTRACT_REFERENCE_CLASS_BY_RUST_HELPER:
    raise ImportError(
        "manimlib constructor alias guard drifted from the authority manifest"
    )
if set(_REFERENCE_CLASS_BY_RUST_HELPER) != set(
    _CONTRACT_REFERENCE_MODULE_BY_RUST_HELPER
):
    raise ImportError(
        "manimlib constructor module guard drifted from the authority manifest"
    )
for _rust_helper, _reference_class in _REFERENCE_CLASS_BY_RUST_HELPER.items():
    if hasattr(_native, _rust_helper):
        raise ImportError(
            f"Rust-only constructor helper {_rust_helper!r} leaked into manimlib"
        )
    if not hasattr(_native, _reference_class):
        raise ImportError(
            f"Reference constructor class {_reference_class!r} is missing from manimlib"
        )
    _reference_module = _CONTRACT_REFERENCE_MODULE_BY_RUST_HELPER[_rust_helper]
    _module = _sys.modules.get(_reference_module)
    if _module is None:
        raise ImportError(
            f"Reference constructor module {_reference_module!r} is missing from manimlib"
        )
    _reference_value = getattr(_native, _reference_class)
    if vars(_module).get(_reference_class) is not _reference_value:
        raise ImportError(
            f"{_reference_module}.{_reference_class} is not the root "
            "constructor identity"
        )
del _REFERENCE_CLASS_BY_RUST_HELPER, _rust_helper, _reference_class, _reference_module
del _reference_value, _module
del _CONTRACT_REFERENCE_CLASS_BY_RUST_HELPER
del _CONTRACT_REFERENCE_MODULE_BY_RUST_HELPER, _sys


for _name in dir(_native):
    if not _name.startswith("_"):
        globals()[_name] = getattr(_native, _name)

__version__ = _native.__version__
__distribution__ = _native.__distribution__
__franken_manim__ = _native.__franken_manim__
__abi_policy__ = _native.__abi_policy__
__engine__ = _native.__engine__
__thread_policy__ = _native.__thread_policy__
__reference_commit__ = _native.__reference_commit__


def __getattr__(name):
    """Forward private diagnostic seams without widening wildcard imports."""

    return getattr(_native, name)


def __dir__():
    return sorted(set(globals()) | set(dir(_native)))


# Importing the native member assembles ``manimlib.*`` compatibility modules.
# CPython attaches each child module to this parent package as a public
# attribute (``manimlib.scene``, ``manimlib.mobject``, ...), but the pinned
# Reference root schema exports none of those package handles.  Keep the wheel
# package's wildcard surface exactly equal to the native schema rather than
# leaking import-system bookkeeping.
for _name in list(globals()):
    if not _name.startswith("_") and not hasattr(_native, _name):
        del globals()[_name]


# Deliberately no __all__: the pinned Reference exports every non-underscore
# root name under `from manimlib import *`.
