//! Menagerie + Atlas: the 161-class mobject library, coordinate systems, fields, 3D solids (§12).
//!
//! The library tier is thin compositions over Marionette (the arena and
//! its records), Chisel (paths, true arc length, space ops), and Scribe
//! (text and mathematics). Every class here is a **value**: chained
//! by-value setters producing a builder that `Stage::add` moves into the
//! arena, the §15.1 surface G0-1 ratified.
//!
//! Landed (fm-oab, §12.1): the vectorized base and its variants
//! ([`vmobject`]), the style surface ([`style`]), the Arc lineage
//! ([`arc`]), the Line lineage with the tip-attachment algebra
//! ([`mod@line`], [`tip`]), and polygons, rectangles, arrow tips, and the
//! frame rectangles ([`poly`]).
//!
//! Three properties hold across the whole tier and are tested as such:
//!
//! * **One arc-density rule** (BN-09). Every arc, wherever it is built,
//!   uses `max(1, ceil(16·|θ|/TAU))` components; the Reference's three
//!   inconsistent conventions are gone.
//! * **True arc length everywhere** (BN-03). Buffers, dashes, tips, and
//!   tangent proportions all measure along the actual curve, not along a
//!   chord or a curve index.
//! * **Semantic shape tags** (§10.8). Constructors record what they built
//!   ([`fmn_mobject::ShapeTag`]) so Lumen can route a circle to the arc
//!   kernel; any write to the points demotes the hint automatically.
//!
//! * **De-TeX'd natives** (BN-08, §11.6/§12.3). Classes the Reference routes
//!   through LaTeX for want of anything better are built natively here:
//!   [`brace`] is a parametric path family that is correct at any width
//!   rather than one glyph stretched, [`matchers`] carries the shape
//!   matchers plus the two `pifont` marks as drawn paths, [`numbers`] is
//!   `DecimalNumber`/`Integer` with glyph-recycling updates, [`matrix`]
//!   gets its brackets from fmd-math's extensible-delimiter engine,
//!   [`special_tex`] is `BulletedList`/`Title` composed on Scribe, and
//!   [`controls`] carries the `interactive.py` control compositions (event
//!   wiring is Proscenium's, W9).
//!
//! * **The Scribe bridge** (fm-p5d, §11.2–11.5). [`text`] turns a
//!   `fmn_text::TextLayout` into a `VMobject` family — one child per glyph,
//!   the `Text[a:b]` / `isolate=` submobject contract intact — and [`tex`]
//!   turns a `fmn_tex::Typeset` into one child per `Sub` with the span map
//!   intact. `Text`, `MarkupText`, `Tex`, and `TexText` are the first
//!   text-bearing mobjects; scale is calibrated the Reference's way (a "0"
//!   stands `font_size / font_size_for_unit_height` units tall).
//!
//! * **Atlas** (fm-v4l, §12.2). [`coords`] owns `CoordinateSystem`, axes,
//!   number lines, Riemann rectangles, and area helpers; [`planes`] owns the
//!   2D/3D plane families; [`graphs`] owns parametric, explicit, and implicit
//!   curves over Chisel's bounded isoline extractor.
//!
//! * **Clouds, images, and models** (fm-2u6, §12.4). [`pointcloud`] owns the
//!   `PMobject`/`PGroup` collections and the DotCloud lineage — `DotCloud`,
//!   `TrueDot`, `GlowDot` — with G0-2's kept glow falloff
//!   `(1-r/R)²`; [`image`] owns `ImageMobject` over fmn-codec's owned
//!   PNG/JPEG decode; [`obj_model`] owns `ThreeDModel` and the owned,
//!   budget-checked OBJ-subset reader that displaces trimesh/pywavefront.
//!
//! * **The 3D solids** (fm-2u6, §12.4). [`solids`] owns the sampled
//!   [`solids::Surface`] value — the Reference's UV-grid semantics
//!   exactly (u-major layout, unclamped epsilon forward differences, the
//!   six-index triangle pattern) — the `three_dimensions.py` census
//!   (`Sphere`…`Prismify`), the wireframe [`solids::SurfaceMesh`], and the
//!   textured family (`TexturedSurface`, `TexturedGeometry` with the C-4
//!   ruling: real area-weighted normals, the dead triple-read not
//!   replicated).
//!
//! * **The fields** (fm-2u6 part 2, §12.4). [`fields`] owns `VectorField`
//!   (tanh length compression on fmn-dmath, colors via the owned 3b1b
//!   colormap anchors), `TimeVaryingVectorField`, and `StreamLines` on
//!   fsci-integrate's adaptive RK45 with dense output — seeded from the
//!   single RNG's named `streamlines` substream and re-spaced at even true
//!   arc length — plus `AnimatedStreamLines` and the stateful tracers
//!   (`TracedPath`, `TracingTail`, `AnimatedBoundary`), whose dt-updater
//!   bindings register with the §9.5 purity classifier.
//!
//! * **Data, booleans, graphs, and drawings** (§12.6–12.7). [`data_mobjects`]
//!   owns Scribe-backed tables and native bar charts; [`network_graph`] and
//!   [`neural_network`] own the enhanced graph/data visualizations;
//!   [`boolean_ops`] exposes `Union`/`Difference`/`Intersection`/`Exclusion`
//!   as audited wrappers over Chisel's certified path-boolean kernel; and
//!   [`drawings`] contains the geometry-native shelf plus ADR-0020's explicit
//!   user-supplied SVG resolver path for asset-backed census classes. These
//!   modules are implemented source surfaces; remaining work is tracked at
//!   the individual compatibility, asset, portal, and release boundaries,
//!   not as a blanket missing-library tranche.
#![forbid(unsafe_code)]

pub mod arc;
pub mod boolean_ops;
pub mod brace;
pub mod code;
pub mod controls;
pub mod coords;
pub mod data_mobjects;
pub mod drawings;
pub mod fields;
pub mod graphs;
pub mod image;
pub mod line;
pub mod markdown;
pub mod matchers;
pub mod matrix;
pub mod network_graph;
pub mod neural_network;
pub mod numbers;
pub mod obj_materials;
pub mod obj_model;
pub mod planes;
pub mod pointcloud;
pub mod poly;
pub mod probability;
pub mod solids;
pub mod spans;
pub mod special_tex;
pub mod style;
pub mod svg;
pub mod svg_export;
pub mod tex;
pub mod text;
pub mod text_matching;
pub mod tip;
pub mod vmobject;

pub use arc::{
    AnnularSector, Annulus, Arc, ArcBetweenPoints, Circle, Dot, Ellipse, curved_arrow,
    curved_double_arrow, sector, small_dot,
};
pub use brace::{Brace, BraceLabel, line_brace};
pub use code::{Code, CodeTheme};
pub use controls::{
    Button, Checkbox, ColorSliders, ControlMob, ControlMobject, ControlPanel, ControlPanelMobject,
    EnableDisableButton, LinearNumberSlider, MotionMobject, ScalarControl, SliderError, Textbox,
    add_scalar_control,
};
pub use coords::{
    Axes, AxisConfig, CoordinateFrame, CoordinateFrameError, CoordinateSystem, CoordsError,
    LiveCoordinateSystem, NumberLine, RiemannConfig, Slider, UnitInterval, create_axis,
};
pub use data_mobjects::{BarChart, DataMobjectError, TableMobject};
pub use fields::{
    AnimatedBoundary, AnimatedStreamLines, FieldError, IntegratorTune, STREAM_LINES_SUBSTREAM,
    StreamLineMeta, StreamLineStyle, StreamLines, StreamLinesMobject, StreamSolution,
    StrokeProfile, TimeVaryingVectorField, TracedPath, TracingTail, VectorField,
    VectorFieldMobject, VectorFieldStyle, VectorGeometry, colormap_gradient, colormap_gradient_at,
    get_sample_coords, grid_sample_points, move_along_vector_field, move_points_along_vector_field,
    move_submobjects_along_vector_field, ode_solution_points, resample_even_arc,
    taper_by_true_length, vectorize,
};
pub use graphs::{
    DEFAULT_MAX_SAMPLES, FunctionGraph, GraphError, ImplicitFunction, ParametricCurve,
    SamplingBudget, SamplingError,
};
pub use image::{DEFAULT_IMAGE_HEIGHT, ImageError, ImageMobject};
pub use line::{Arrow, DashedLine, Elbow, Line, StrokeArrow, tangent_line, vector};
pub use markdown::{LaidBlock, Markdown, MarkdownMobject, heading_factor};
pub use matchers::{
    SurroundingRectangle, background_rectangle, checkmark, cross, exmark, underline,
};
pub use matrix::{
    DEFAULT_MAX_MATRIX_ENTRIES, DecimalMatrix, IntegerMatrix, Matrix, MatrixError, MatrixMobject,
    MobjectMatrix, TexMatrix,
};
pub use network_graph::{
    DEFAULT_SPRING_IDEAL_EDGE, DEFAULT_SPRING_ITERATIONS, GraphLayout, NetworkGraph,
    NetworkGraphError,
};
pub use neural_network::{
    DEFAULT_LAYER_SPACING, DEFAULT_MAX_DISPLAYED_NEURONS, DEFAULT_NEURON_RADIUS,
    DEFAULT_NEURON_SPACING, DEFAULT_SYNAPSE_STROKE_WIDTH, LayerKind, LayerSpec,
    NeuralNetworkConfig, NeuralNetworkError, NeuralNetworkMobject,
};
pub use numbers::{DEFAULT_MAX_NUMBER_CHARACTERS, DecimalNumber, Integer};
pub use obj_materials::{
    ObjAssetError, ObjAssetLimits, ObjDocument, ObjMaterial, ObjMaterialRun, parse_mtl,
};
pub use obj_model::{
    DEFAULT_MODEL_HEIGHT, MODEL_SHADING, ObjCorner, ObjError, ObjLimits, ObjMesh, ThreeDModel,
};
pub use planes::{ComplexPlane, NumberPlane, ThreeDAxes};
pub use pointcloud::{
    DEFAULT_BUFF_RATIO, DEFAULT_DOT_CLOUD_RADIUS, DEFAULT_GLOW_DOT_RADIUS, DEFAULT_GRID_HEIGHT,
    DOT_CLOUD_AA_WIDTH, DOT_CLOUD_SHADING, DotCloud, GLOW_DOT_FACTOR, GlowLayer, PMobject,
    glow_dot, glow_dots, glow_falloff, glow_layers, p_group, rim_coverage, true_dot,
};
pub use poly::{
    ArrowTip, CubicBezier, Polygon, Rectangle, RegularPolygon, Square, TipStyle, polyline,
    rounded_rectangle, triangle,
};
pub use probability::{ProbabilityError, SampleSpace};
pub use solids::{
    CUBE_SHADING, Cone, Cube, Cylinder, Disk3D, Dodecahedron, Line3D, MESH_NORMAL_NUDGE,
    MESH_RESOLUTION, MeshError, ParametricSurface, Prism, Prismify, SGroup, SURFACE_COLOR,
    SURFACE_EPSILON, SURFACE_NORMAL_NUDGE, SURFACE_RESOLUTION, SURFACE_SAMPLING_BUDGET,
    SURFACE_SHADING, Sphere, Square3D, Surface, SurfaceMesh, SurfaceSpec, TexturedGeometry,
    TexturedSurface, Torus, VCube, VGroup3D, VPrism, compute_triangle_indices, surface_schema,
    textured_surface_schema,
};
pub use special_tex::{BulletedList, BulletedListMobject, Title, TitleMobject};
pub use style::{Style, VStyle};
pub use svg::{svg_document_mobject, svg_mobject, vmobject_from_svg_path};
pub use tex::{LineAlign, OldTex, OldTexMobject, Tex, TexMobject, TexMobjectError, TexText};
pub use text::{MarkupText, Text, TextMobject, TextMobjectError};
pub use text_matching::{
    MAX_MATCHING_COMPARISONS, MAX_MATCHING_PARTS, TextMatchingConfig, TextMatchingError,
    TransformMatchingStrings, TransformMatchingTex,
};
pub use tip::TipEnd;
pub use vmobject::{
    DashError, MAX_DASHES, VMobject, curves_as_submobjects, dashed_vmobject, group, v_group,
    v_highlight, vectorized_point,
};

// The typesetting handles the builders take. Re-exported so
// above-library consumers (the Python portal's native-builder seam) can
// construct them through their declared fmn-library edge, mirroring
// fmn-scene's studio_bridge facade pattern (plan §19).
pub use fmn_geom::{
    AnchorMode, BooleanOperation, BooleanOptions, MAX_SUBDIVIDED_CURVES, Mat3, QuadPath,
    bezier::integer_interpolate,
    earclip::triangulate as earclip_triangulate,
    path_boolean, rotation_matrix,
    smoothing::{approx_smooth_quadratic_handles, smooth_cubic_handles, smooth_quadratic_path},
    space_ops::{
        angle_axis_from_quaternion, angle_between_vectors, angle_of_vector, find_intersection,
        get_closest_point_on_line, get_unit_normal, get_winding_number, is_inside_triangle,
        line_intersection, line_intersects_path, poly_line_length, project_along_vector,
        quaternion_conjugate, quaternion_from_angle_axis, quaternion_mult, rotation_about_z,
        rotation_between_vectors, rotation_matrix_from_quaternion, rotation_matrix_transpose,
        rotation_matrix_transpose_from_quaternion, thick_diagonal, tri_area, z_to_vector,
    },
};
pub use fmn_tex::{Prim as TexPrim, TexEngine, TypesetRequest};
pub use fmn_text::{FontBook, bundled_faces};

// The Scribe span-map seam: the composition root's translation data
// between the builders' native span tables and Studio's SpanRegistry.
pub use spans::{
    MAX_SPAN_RECORDS, SpanCollector, SpanCollectorError, SpanKindU8, SpanMapData, SpanMapEntry,
    SpanRecord, add_with_spans,
};
