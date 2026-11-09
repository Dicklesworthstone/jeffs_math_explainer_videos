"""Keep Surface topology coherent across the shared Transform protocol.

Marionette owns all UV resampling, field alignment and generation publication.
This adapter owns Python-visible metadata and callback/native route admission;
it neither evaluates authored UV functions nor implements a second resampler.
"""
from __future__ import annotations

from types import SimpleNamespace

from .surface_admission import GRID_BUDGET


def install_surface_alignment(native):
    g = vars(native)
    if g.get("_FMN_SURFACE_ALIGNMENT_INSTALLED", False):
        return
    if not callable(g.get("_align_surface_grids")) or not callable(g.get("_surface_grid_resolution")):
        raise ImportError("native UV-grid alignment seam is missing")
    Surface, Mobject, Transform = g["Surface"], g["Mobject"], g["Transform"]
    original_align, original_aligned = Mobject.align_points, Mobject.is_aligned_with
    triangles = Surface.compute_triangle_indices

    def pair_shapes(left, right):
        resolution = g["_surface_grid_resolution"]
        a, b = resolution(left), resolution(right)
        if (a is None) != (b is None):
            raise TypeError("surface alignment requires two native UV-grid surfaces")
        return a, b

    def empty_pair(left, right):
        # SGroup is a Surface by MRO, but its root owns no UV grid. Likewise,
        # a Group may wrap the same surface family. Only their drawable leaves
        # need UV alignment; never invent a grid for empty container records.
        return isinstance(right, Mobject) and not left.has_points() and not right.has_points()

    def align_points(self, other):
        if not isinstance(self, Surface) and not isinstance(other, Surface):
            return original_align(self, other)
        if empty_pair(self, other):
            return original_align(self, other)
        if not isinstance(self, Surface) or not isinstance(other, Surface):
            raise TypeError("a UV-grid surface cannot align with an unstructured mobject")
        a, b = pair_shapes(self, other)
        if a is None:  # Both are unstructured meshes, not sampled UV grids.
            return original_align(self, other)
        shape = (max(a[0], b[0]), max(a[1], b[1]))
        if shape[0] * shape[1] > GRID_BUDGET:
            raise ValueError(f"surface alignment exceeds its {GRID_BUDGET}-point UV-grid budget")
        # Allocate derived Python indices before native publication. Invoke the
        # shipped constructor on inert holders, not callbacks on either object.
        # Independent arrays prevent one endpoint's face ordering affecting its peer.
        indices = [
            vars(member)["triangle_indices"]
            if old == shape and tuple(vars(member).get("resolution", ())) == shape
            and "triangle_indices" in vars(member)
            else triangles(SimpleNamespace(resolution=shape))
            for member, old in ((self, a), (other, b))
        ]
        g["_align_surface_grids"](self, other)
        for member, data in zip((self, other), indices):
            vars(member).update(resolution=shape, triangle_indices=data)
        return self

    def is_aligned_with(self, other):
        if isinstance(self, Surface) or isinstance(other, Surface):
            if empty_pair(self, other):
                return original_aligned(self, other)
            if not isinstance(self, Surface) or not isinstance(other, Surface):
                return False
            a, b = pair_shapes(self, other)
            if a is None:
                return original_aligned(self, other)
            if a != b or self.data.dtype != other.data.dtype:
                return False
        return original_aligned(self, other)

    for cls, name, method in ((Mobject, "align_points", align_points),
                              (Surface, "align_points", align_points),
                              (Mobject, "is_aligned_with", is_aligned_with)):
        method.__name__ = name
        method.__qualname__ = cls.__qualname__ + "." + name
        method.__module__ = cls.__module__
        setattr(cls, name, method)

    original_indices = Surface.get_triangle_indices

    def get_triangle_indices(self):
        # Atlas-backed subclasses need not call Surface.__init__. Derive a
        # missing public index array lazily; retain authored face order when it
        # already exists. Alignment installs fresh arrays before any frame runs.
        if "triangle_indices" not in vars(self):
            shape = g["_surface_grid_resolution"](self)
            if shape is not None:
                vars(self)["triangle_indices"] = triangles(SimpleNamespace(resolution=shape))
        return original_indices(self)

    get_triangle_indices.__name__ = "get_triangle_indices"
    get_triangle_indices.__qualname__ = Surface.__qualname__ + ".get_triangle_indices"
    get_triangle_indices.__module__ = Surface.__module__
    Surface.get_triangle_indices = get_triangle_indices

    previous_requires = g["_requires_python_animation"]

    def requires(animation):
        if previous_requires(animation):
            return True
        if isinstance(animation, Transform):
            # The preceding shipped-protocol classifier has already refused
            # authored attribute/family dispatch. Resolve no deferred target here.
            # Surface leaves use the existing ordered Transform lifecycle so
            # Python topology agrees with native records before interpolation,
            # including Groups, Succession and targets made at begin().
            for name in ("mobject", "target_mobject"):
                root = getattr(animation, name, None)
                if isinstance(root, Mobject) and any(
                        isinstance(member, Surface) for member in root.get_family()):
                    return True
        return False

    g["_requires_python_animation"] = requires
    g["_FMN_SURFACE_ALIGNMENT_INSTALLED"] = True
