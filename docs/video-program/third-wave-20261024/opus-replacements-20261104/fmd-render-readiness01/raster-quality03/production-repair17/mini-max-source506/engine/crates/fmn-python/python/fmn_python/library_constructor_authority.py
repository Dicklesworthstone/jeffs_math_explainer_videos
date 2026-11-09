"""Machine-readable authority map for Rust library constructor aliases.

The Rust library exposes ergonomic snake_case constructor helpers, while the
pinned ``manimlib`` surface exposes CamelCase classes.  Every helper tracked by
``fm-5wq.4.141`` has one record below describing the actual authority path:
direct native construction, inherited construction, a hardened/equivalent
native bridge, a native algorithm wrapped by identity-preserving Python
shells, or an intentionally Python-owned identity/composition route.
"""

from __future__ import annotations

from types import MappingProxyType
from typing import Final, NamedTuple


class ConstructorAuthority(NamedTuple):
    rust_helper: str
    reference_module: str
    reference_class: str
    binding_kind: str
    python_base: str
    python_authority_class: str
    python_authority_token: str
    native_builder: str | None
    rust_source: str
    rust_function: str
    rust_authority_token: str
    bridge_function: str | None
    bridge_authority_token: str | None
    rationale: str


_BINDING_KINDS: Final = frozenset(
    {
        "direct_native_builder",
        "hardened_native_builder",
        "inherited_native_builder",
        "native_algorithm_python_shell",
        "native_equivalent_builder",
        "python_identity_container",
        "python_reference_composition",
    }
)
_PYTHON_ONLY_BINDING_KINDS: Final = frozenset(
    {
        "python_identity_container",
        "python_reference_composition",
    }
)

_REFERENCE_CLASS_BY_RUST_HELPER: Final = {
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
REFERENCE_CLASS_BY_RUST_HELPER: Final = MappingProxyType(
    _REFERENCE_CLASS_BY_RUST_HELPER
)

LIBRARY_CONSTRUCTOR_AUTHORITIES: Final = (
    ConstructorAuthority(
        rust_helper="curved_arrow",
        reference_module="manimlib.mobject.geometry",
        reference_class="CurvedArrow",
        binding_kind="native_equivalent_builder",
        python_base="ArcBetweenPoints",
        python_authority_class="CurvedArrow",
        python_authority_token="self._build_curved_arrow(",
        native_builder="_build_curved_arrow",
        rust_source="crates/fmn-library/src/arc.rs",
        rust_function="curved_arrow",
        rust_authority_token="Ok(attach_tip(",
        bridge_function="_build_curved_arrow",
        bridge_authority_token="fmn_library::tip::attach_tip(",
        rationale=(
            "The bridge uses the same ArcBetweenPoints and tip algebra as the "
            "facade while preserving the Reference's explicit n_components knob."
        ),
    ),
    ConstructorAuthority(
        rust_helper="curved_double_arrow",
        reference_module="manimlib.mobject.geometry",
        reference_class="CurvedDoubleArrow",
        binding_kind="native_equivalent_builder",
        python_base="CurvedArrow",
        python_authority_class="CurvedDoubleArrow",
        python_authority_token="self._build_curved_arrow(",
        native_builder="_build_curved_arrow",
        rust_source="crates/fmn-library/src/arc.rs",
        rust_function="curved_double_arrow",
        rust_authority_token="let once = curved_arrow(start, end, angle, style)?;",
        bridge_function="_build_curved_arrow",
        bridge_authority_token="if double {",
        rationale=(
            "The shared bridge selects the second start tip explicitly so both "
            "CurvedArrow classes retain one native shaft-and-tip authority."
        ),
    ),
    ConstructorAuthority(
        rust_helper="curves_as_submobjects",
        reference_module="manimlib.mobject.types.vectorized_mobject",
        reference_class="CurvesAsSubmobjects",
        binding_kind="direct_native_builder",
        python_base="VGroup",
        python_authority_class="CurvesAsSubmobjects",
        python_authority_token="self._build_curves_as_submobjects(",
        native_builder="_build_curves_as_submobjects",
        rust_source="crates/fmn-library/src/vmobject.rs",
        rust_function="curves_as_submobjects",
        rust_authority_token="for tuple in path.bezier_tuples() {",
        bridge_function="_build_curves_as_submobjects",
        bridge_authority_token=(
            "fmn_library::vmobject::curves_as_submobjects(&source)"
        ),
        rationale=(
            "Atlas splits the shared-anchor run; Python reapplies the live "
            "source style after installing the native child shells."
        ),
    ),
    ConstructorAuthority(
        rust_helper="dashed_vmobject",
        reference_module="manimlib.mobject.types.vectorized_mobject",
        reference_class="DashedVMobject",
        binding_kind="native_algorithm_python_shell",
        python_base="VMobject",
        python_authority_class="DashedVMobject",
        python_authority_token="vmobject._dash_curve_intervals(",
        native_builder="_dash_curve_intervals",
        rust_source="crates/fmn-library/src/vmobject.rs",
        rust_function="dashed_vmobject",
        rust_authority_token="for (a, b) in dash_curve_intervals(",
        bridge_function="_dash_curve_intervals",
        bridge_authority_token=(
            "fmn_library::vmobject::dash_curve_intervals("
        ),
        rationale=(
            "Atlas owns true-arclength placement, while Python performs live "
            "partial copies so every source record lane and proxy identity survives."
        ),
    ),
    ConstructorAuthority(
        rust_helper="group",
        reference_module="manimlib.mobject.mobject",
        reference_class="Group",
        binding_kind="python_identity_container",
        python_base="Mobject",
        python_authority_class="Group",
        python_authority_token="self._ingest_args(*mobjects)",
        native_builder=None,
        rust_source="crates/fmn-library/src/vmobject.rs",
        rust_function="group",
        rust_authority_token="Mobject::group(children.into_iter().collect())",
        bridge_function=None,
        bridge_authority_token=None,
        rationale=(
            "Group must preserve the exact heterogeneous Python proxy identities; "
            "a detached Rust reconstruction would change observable object identity."
        ),
    ),
    ConstructorAuthority(
        rust_helper="polyline",
        reference_module="manimlib.mobject.geometry",
        reference_class="Polyline",
        binding_kind="direct_native_builder",
        python_base="VMobject",
        python_authority_class="Polyline",
        python_authority_token=(
            '_set_native_vmobject_points(self, "_build_polyline", '
            "[_vec3(vertex) for vertex in self.vertices])"
        ),
        native_builder="_build_polyline",
        rust_source="crates/fmn-library/src/poly.rs",
        rust_function="polyline",
        rust_authority_token="Polygon::polyline(vertices)",
        bridge_function="_build_polyline",
        bridge_authority_token="fmn_library::Polygon::polyline(vertices)",
        rationale=(
            "Polyline owns a direct native point-run builder while retaining the "
            "Reference class and mutation surface."
        ),
    ),
    ConstructorAuthority(
        rust_helper="rounded_rectangle",
        reference_module="manimlib.mobject.geometry",
        reference_class="RoundedRectangle",
        binding_kind="native_equivalent_builder",
        python_base="Rectangle",
        python_authority_class="RoundedRectangle",
        python_authority_token=(
            '_set_native_vmobject_points(self, "_build_rounded_rectangle", '
            "self.width, self.height, self.corner_radius)"
        ),
        native_builder="_build_rounded_rectangle",
        rust_source="crates/fmn-library/src/poly.rs",
        rust_function="rounded_rectangle",
        rust_authority_token="Rectangle::new()",
        bridge_function="_build_rounded_rectangle",
        bridge_authority_token="fmn_library::Rectangle::new()",
        rationale=(
            "The bridge drives the same Rectangle builder directly, then installs "
            "the rounded native point run into the Reference subclass."
        ),
    ),
    ConstructorAuthority(
        rust_helper="sector",
        reference_module="manimlib.mobject.geometry",
        reference_class="Sector",
        binding_kind="inherited_native_builder",
        python_base="AnnularSector",
        python_authority_class="AnnularSector",
        python_authority_token=(
            "super().__init__(angle, inner_radius=0.0, outer_radius=radius, **kwargs)"
        ),
        native_builder="_build_annular_sector",
        rust_source="crates/fmn-library/src/arc.rs",
        rust_function="sector",
        rust_authority_token="AnnularSector::sector(angle, radius)",
        bridge_function="_build_annular_sector",
        bridge_authority_token="fmn_library::AnnularSector::new()",
        rationale=(
            "Sector is the Reference's constrained AnnularSector facade, so the "
            "parent class remains the single native-builder authority."
        ),
    ),
    ConstructorAuthority(
        rust_helper="small_dot",
        reference_module="manimlib.mobject.geometry",
        reference_class="SmallDot",
        binding_kind="inherited_native_builder",
        python_base="Dot",
        python_authority_class="Dot",
        python_authority_token="super().__init__(point, radius, **kwargs)",
        native_builder="_build_dot",
        rust_source="crates/fmn-library/src/arc.rs",
        rust_function="small_dot",
        rust_authority_token="Dot::small().point(point)",
        bridge_function="_build_dot",
        bridge_authority_token="fmn_library::Dot::new()",
        rationale=(
            "SmallDot is a parameter-specialized Dot and must reuse Dot's native "
            "builder rather than fork its live proxy semantics."
        ),
    ),
    ConstructorAuthority(
        rust_helper="svg_mobject",
        reference_module="manimlib.mobject.svg.svg_mobject",
        reference_class="SVGMobject",
        binding_kind="hardened_native_builder",
        python_base="VMobject",
        python_authority_class="SVGMobject",
        python_authority_token="self._build_svg_mobject(",
        native_builder="_build_svg_mobject",
        rust_source="crates/fmn-library/src/svg.rs",
        rust_function="svg_mobject",
        rust_authority_token="Ok(svg_document_mobject(&document))",
        bridge_function="_build_svg_mobject",
        bridge_authority_token="fmn_library::svg_document_mobject(&document)",
        rationale=(
            "The portal uses the same native document builder after enforcing the "
            "untrusted-input byte, nesting, and feature budgets before allocation."
        ),
    ),
    ConstructorAuthority(
        rust_helper="tangent_line",
        reference_module="manimlib.mobject.geometry",
        reference_class="TangentLine",
        binding_kind="direct_native_builder",
        python_base="Line",
        python_authority_class="TangentLine",
        python_authority_token="self._build_tangent_line(",
        native_builder="_build_tangent_line",
        rust_source="crates/fmn-library/src/line.rs",
        rust_function="tangent_line",
        rust_authority_token="let line = Line::new(p1, p2)",
        bridge_function="_build_tangent_line",
        bridge_authority_token="fmn_library::line::tangent_line(",
        rationale=(
            "The live source point run crosses once into Atlas's true-arclength "
            "tangent helper, then the native result is installed atomically."
        ),
    ),
    ConstructorAuthority(
        rust_helper="triangle",
        reference_module="manimlib.mobject.geometry",
        reference_class="Triangle",
        binding_kind="inherited_native_builder",
        python_base="RegularPolygon",
        python_authority_class="RegularPolygon",
        python_authority_token="super().__init__(n=3, **kwargs)",
        native_builder="_build_regular_polygon",
        rust_source="crates/fmn-library/src/poly.rs",
        rust_function="triangle",
        rust_authority_token="RegularPolygon::triangle()",
        bridge_function="_build_regular_polygon",
        bridge_authority_token="fmn_library::RegularPolygon::new(n)",
        rationale=(
            "Triangle is exactly RegularPolygon(n=3); inheritance preserves the "
            "Reference constructor and keeps one bounded compass kernel."
        ),
    ),
    ConstructorAuthority(
        rust_helper="v_group",
        reference_module="manimlib.mobject.types.vectorized_mobject",
        reference_class="VGroup",
        binding_kind="python_identity_container",
        python_base="VMobject",
        python_authority_class="VGroup",
        python_authority_token="self._ingest_args(*vmobjects)",
        native_builder=None,
        rust_source="crates/fmn-library/src/vmobject.rs",
        rust_function="v_group",
        rust_authority_token="VMobject::new().with_children(children)",
        bridge_function=None,
        bridge_authority_token=None,
        rationale=(
            "VGroup validates VMobject membership while retaining the caller's "
            "exact Python child proxies and the Reference MRO used by Axes."
        ),
    ),
    ConstructorAuthority(
        rust_helper="v_highlight",
        reference_module="manimlib.mobject.types.vectorized_mobject",
        reference_class="VHighlight",
        binding_kind="python_reference_composition",
        python_base="VGroup",
        python_authority_class="VHighlight",
        python_authority_token="_color_gradient(color_bounds, n_layers)",
        native_builder=None,
        rust_source="crates/fmn-library/src/vmobject.rs",
        rust_function="v_highlight",
        rust_authority_token="let colors = color_gradient(&color_bounds, n_layers);",
        bridge_function=None,
        bridge_authority_token=None,
        rationale=(
            "The Reference composition mutates full live Python families and keeps "
            "their copy identities; the Rust helper remains the Rust-front-door value."
        ),
    ),
    ConstructorAuthority(
        rust_helper="vector",
        reference_module="manimlib.mobject.geometry",
        reference_class="Vector",
        binding_kind="inherited_native_builder",
        python_base="Arrow",
        python_authority_class="Arrow",
        python_authority_token=(
            "super().__init__(_ORIGIN, direction, buff=buff, **kwargs)"
        ),
        native_builder="_build_arrow",
        rust_source="crates/fmn-library/src/line.rs",
        rust_function="vector",
        rust_authority_token="Arrow::vector(direction)",
        bridge_function="_build_arrow",
        bridge_authority_token="fmn_library::line::Arrow::new",
        rationale=(
            "Vector is the Reference's origin-anchored Arrow facade and therefore "
            "inherits Arrow's native geometry and tip-installation authority."
        ),
    ),
    ConstructorAuthority(
        rust_helper="vectorized_point",
        reference_module="manimlib.mobject.types.vectorized_mobject",
        reference_class="VectorizedPoint",
        binding_kind="direct_native_builder",
        python_base="Point, VMobject",
        python_authority_class="VectorizedPoint",
        python_authority_token="self._build_vectorized_point(",
        native_builder="_build_vectorized_point",
        rust_source="crates/fmn-library/src/vmobject.rs",
        rust_function="vectorized_point",
        rust_authority_token="VMobject::from_points(vec![location])",
        bridge_function="_build_vectorized_point",
        bridge_authority_token=(
            "fmn_library::vmobject::vectorized_point(location)"
        ),
        rationale=(
            "Atlas creates the one-record invisible location value directly while "
            "the Python class retains its Point/VMobject compatibility MRO."
        ),
    ),
)

LIBRARY_CONSTRUCTOR_AUTHORITY_BY_HELPER: Final = MappingProxyType(
    {item.rust_helper: item for item in LIBRARY_CONSTRUCTOR_AUTHORITIES}
)
REFERENCE_MODULE_BY_RUST_HELPER: Final = MappingProxyType(
    {
        item.rust_helper: item.reference_module
        for item in LIBRARY_CONSTRUCTOR_AUTHORITIES
    }
)
REFERENCE_SYMBOL_BY_RUST_HELPER: Final = MappingProxyType(
    {
        item.rust_helper: (item.reference_module, item.reference_class)
        for item in LIBRARY_CONSTRUCTOR_AUTHORITIES
    }
)


def _validate() -> None:
    helpers = [item.rust_helper for item in LIBRARY_CONSTRUCTOR_AUTHORITIES]
    expected_helpers = sorted(REFERENCE_CLASS_BY_RUST_HELPER)
    if helpers != expected_helpers:
        missing = sorted(set(expected_helpers) - set(helpers))
        extra = sorted(set(helpers) - set(expected_helpers))
        raise RuntimeError(
            "library constructor authority coverage drift: "
            f"missing={missing}, extra={extra}, ordered={helpers == sorted(helpers)}"
        )
    if len(helpers) != len(set(helpers)):
        raise RuntimeError("library constructor authority helpers must be unique")
    if len(set(REFERENCE_SYMBOL_BY_RUST_HELPER.values())) != len(helpers):
        raise RuntimeError("library constructor Reference symbols must be unique")
    for item in LIBRARY_CONSTRUCTOR_AUTHORITIES:
        if item.binding_kind not in _BINDING_KINDS:
            raise RuntimeError(
                f"unknown constructor binding kind {item.binding_kind!r}"
            )
        expected_class = REFERENCE_CLASS_BY_RUST_HELPER.get(item.rust_helper)
        if expected_class != item.reference_class:
            raise RuntimeError(
                f"{item.rust_helper!r} maps to {expected_class!r}, "
                f"not {item.reference_class!r}"
            )
        for field_name in (
            "reference_module",
            "reference_class",
            "python_base",
            "python_authority_class",
            "python_authority_token",
            "rust_source",
            "rust_function",
            "rust_authority_token",
            "rationale",
        ):
            if not getattr(item, field_name):
                raise RuntimeError(
                    f"{item.rust_helper!r} has an empty {field_name}"
                )
        has_builder = item.native_builder is not None
        has_bridge = (
            item.bridge_function is not None
            and item.bridge_authority_token is not None
        )
        if item.binding_kind in _PYTHON_ONLY_BINDING_KINDS:
            if has_builder or has_bridge:
                raise RuntimeError(
                    f"{item.rust_helper!r} Python-owned binding cannot name a native bridge"
                )
        elif not has_builder or not has_bridge:
            raise RuntimeError(
                f"{item.rust_helper!r} native binding requires a complete bridge"
            )
        elif item.native_builder != item.bridge_function:
            raise RuntimeError(
                f"{item.rust_helper!r} builder and bridge function disagree"
            )


_validate()
del _validate
