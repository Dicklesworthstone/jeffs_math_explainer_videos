//! The 3D solids census (§12.4, fm-2u6 part 1): [`Surface`] and its UV-grid
//! semantics, [`ParametricSurface`], the `three_dimensions.py` solids, the
//! vectorized 3D groups, the wireframe [`SurfaceMesh`], and the textured
//! family ([`TexturedSurface`], [`TexturedGeometry`]).
//!
//! Ported from `manimlib/mobject/three_dimensions.py` and
//! `manimlib/mobject/types/surface.py` @ `6199a00d`. A [`Surface`] here is a
//! **value** like [`VMobject`]: sampled points plus the Reference's
//! `d_normal_point` column, per-point rgba, the fixed UV-grid triangle
//! indices, and the kept `(reflectiveness, gloss, shadow)` shading uniform
//! (fm-0gy's `fmn-render`[^render] lighting consumes exactly this data).
//! `Stage::add` moves one into the arena through
//! [`From<Surface> for Mobject`].
//!
//! [^render]: `crates/fmn-render/src/three_d.rs` — `SURFACE_SHADING`,
//! `SurfaceMesh::from_uv_grid`, `finalize_color`.
//!
//! # UV-grid semantics (ported exactly)
//!
//! * **Layout.** `resolution = (nu, nv)` counts *sampled points*, one more
//!   per axis than the rows/columns of approximating squares. Points are
//!   u-major: index `i·nv + j` is `uv_func(u_i, v_j)` with `u_i`, `v_j`
//!   `linspace` over the ranges (numpy `meshgrid(indexing='ij')` flattened
//!   in C order).
//! * **Normals.** Three evaluations per grid point: `f(u, v)`,
//!   `f(u + ε, v)`, `f(u + ε, v)` with `ε = epsilon` (default `1e-3`,
//!   added in UV space, **never clamped at the range boundary** — the
//!   forward difference at `u_max` samples past the end exactly as the
//!   Reference does). The normal is
//!   `normalize(cross(f(u+ε,v) − f(u,v), f(u,v+ε) − f(u,v)))`; a zero
//!   cross stays zero (numpy `normalize_along_axis`'s `out=zeros`).
//!   `d_normal_point = point + normal_nudge · normal` (nudge `1e-3`), and
//!   unit normals are re-derived on read as `d_normal_point − point`
//!   normalized — so any transform applied to *both* columns (the
//!   Reference's `pointlike_data_keys`) preserves normals, which is what
//!   [`Surface::map_points`] does.
//! * **Triangles.** The fixed six-index pattern per grid cell —
//!   `(TL, BL, TR)`, `(TR, BL, BR)` in row-major cell order — `6·(nu−1)·(nv−1)`
//!   indices ([`Surface::triangle_indices`], [`compute_triangle_indices`]).
//! * **The `apply_points_function` re-read is dead.** The Reference's
//!   `Surface.apply_points_function` calls `get_unit_normals()` and
//!   discards the result; the actual invariant is the pointlike transform
//!   above. Only that is ported.
//!
//! # Reference-fidelity notes and divergences
//!
//! * **`Sphere(true_normals)`.** After sampling, `d_normal_point` is
//!   replaced by `point · (radius + normal_nudge) / radius` — radial
//!   normals, bespoke to avoid the degenerate cross product at the poles
//!   (a `radius` of `0`, which is a `ZeroDivisionError` in the Reference,
//!   instead keeps the sampled column here).
//! * **Solid placement follows the Reference's transform defaults.**
//!   `scale`, `stretch` and `set_depth` act about the sampled shape's
//!   bounding-box center (`about_edge=ORIGIN`), and `apply_matrix` acts about
//!   the origin. A `Cone` is sampled on `v ∈ [0, 1]`, so it spans
//!   `z ∈ [½ − height/2, ½ + height/2]` along its axis: `[−½, 3⁄2]` by
//!   default, not `[0, height]`. A `Disk3D` with coarse angular sampling
//!   drifts off the origin by its sampled polygon's center offset, exactly as
//!   in the Reference.
//! * **`SurfaceMesh` style kwargs land on the point-less parent.** In the
//!   Reference the wireframe paths are bare `VMobject()` children, so they
//!   draw with VMobject defaults (GREY_A stroke, width `4`, `auto` joint,
//!   no depth test) while `stroke_width=1`/`joint_type='no_joint'`/
//!   `depth_test=True` sit on the empty group and style nothing. This is
//!   corrected here: stroke and uniform settings reach every wireframe
//!   child (BN-07). The `1e-2` normal nudge and floor/ceil interpolation
//!   are unchanged.
//! * **C-4 (`TexturedGeometry`).** The Reference's `init_points`
//!   triple-reads `triangle_indices[0::3]` into `v0`/`v1`/`v2` (dead code)
//!   and nudges normals by `1e-5·0` over `normals = points` — its stored
//!   normals are literally `normalize(0)`. Appendix C rules this *not
//!   replicated*: [`TexturedGeometry`] computes real area-weighted
//!   per-vertex normals from the faces (what `trimesh.vertex_normals`
//!   would give), and a degenerate mesh keeps a zero normal rather than
//!   the Reference's accidental one.
//! * **No `trimesh` ingress here.** [`TexturedGeometry::from_mesh`] takes
//!   explicit vertices/faces/uv; OBJ loading (`ThreeDModel`) is fm-2u6
//!   part 3. The uv v-flip (`v ↦ 1 − v`) is applied inside `from_mesh`,
//!   matching `TexturedGeometry.init_points`.
//! * **UV-texture hooks.** `Surface.color_by_uv_function` (per-point rgba
//!   from `(u, v)`) and [`TexturedSurface::set_image_coords_by_uv_func`]
//!   are ported. The pin has **no procedural checkerboard helper** —
//!   textured examples reference image assets — so there is nothing
//!   further to port; this note is the documented omission.
//! * **Other-tier operations.** `sort_faces_back_to_front` /
//!   `always_sort_to_camera` remain camera-coupled W9/Lumen behavior (the
//!   triangle data they permute is here). Surface partial reveal lives in
//!   Marionette's typed UV-grid operation and Choreo's `ShowPartial` driver;
//!   this crate supplies their `preferred_creation_axis` constructor data.
//! * **`ShapeTag`.** Surfaces are not paths, so they enter the arena as
//!   [`ShapeTag::General`]; the semantic raster hints of §10.8 do not
//!   apply to UV grids.

use fmn_core::color::Srgb;
use fmn_core::constants::{BLUE, BLUE_D, BLUE_E, GREY, GREY_A, IN, ORIGIN, OUT, PI, RIGHT, TAU};
use fmn_core::types::Vec3;
use fmn_geom::{QuadPath, space_ops};
use fmn_mobject::uniforms::{JointType, Uniforms};
use fmn_mobject::{Mobject, RecordBuffer, RecordSchema, RenderPrimitive, ShapeTag};

use crate::poly::{Polygon, Rectangle};
use crate::style::Style;
use crate::vmobject::{VMobject, v_group};
use crate::{SamplingBudget, SamplingError};

/// A surface construction refusal, separate from the caller's sample error.
#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceSampleError<E> {
    /// A dimension, aggregate size, or allocation exceeded its admitted budget.
    Budget(SamplingError),
    /// The specification cannot produce a finite record grid.
    InvalidControl(&'static str),
    /// A point or normal-control point is not finite and f32-representable.
    InvalidSample {
        /// Zero-based u-major sample index.
        index: usize,
        /// The failed evaluation or derived column.
        probe: &'static str,
    },
    /// The original callback error. No further callback is evaluated.
    Callback(E),
}

impl<E: std::fmt::Display> std::fmt::Display for SurfaceSampleError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Budget(error) => std::fmt::Display::fmt(error, f),
            Self::InvalidControl(name) => write!(f, "invalid surface sampling control: {name}"),
            Self::InvalidSample { index, probe } => write!(
                f,
                "surface sample {index} ({probe}) must be finite and f32-representable"
            ),
            Self::Callback(error) => write!(f, "surface callback failed: {error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for SurfaceSampleError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Budget(error) => Some(error),
            Self::Callback(error) => Some(error),
            Self::InvalidControl(_) | Self::InvalidSample { .. } => None,
        }
    }
}

impl<E> From<SamplingError> for SurfaceSampleError<E> {
    fn from(error: SamplingError) -> Self {
        Self::Budget(error)
    }
}

fn surface_storage<T>(count: usize) -> Result<Vec<T>, SamplingError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| SamplingError::AllocationFailed {
            context: "surface",
            samples: count,
        })?;
    Ok(values)
}

fn surface_record(point: Vec3) -> bool {
    point
        .iter()
        .all(|v| v.is_finite() && v.abs() <= f64::from(f32::MAX))
}

// Keep exactly the arithmetic used by linspace; do not substitute an FMA or
// a lerp, since sampled surface and normal-control bytes are an oracle.
fn surface_station(range: (f64, f64), count: usize, index: usize) -> f64 {
    if count <= 1 {
        range.0
    } else {
        range.0 + (range.1 - range.0) / (count - 1) as f64 * index as f64
    }
}

/// Default UV sampling budget, shared with Marionette's alignment ceiling.
///
/// Dense charts (including 301-by-301) are distinct from curves, fields and
/// wireframe output, whose [`SamplingBudget::DEFAULT`] remains unchanged.
pub const SURFACE_SAMPLING_BUDGET: SamplingBudget =
    SamplingBudget::new(fmn_mobject::stage::MAX_SURFACE_GRID_POINTS);

/// The Reference's `Surface(resolution=(101, 101))` default.
pub const SURFACE_RESOLUTION: (usize, usize) = (101, 101);
/// The Reference's `Surface(epsilon=1e-3)` — the du/dv step, added in UV
/// space.
pub const SURFACE_EPSILON: f64 = 1e-3;
/// The Reference's `Surface(normal_nudge=1e-3)`.
pub const SURFACE_NORMAL_NUDGE: f64 = 1e-3;
/// The kept `Surface` shading triple `(reflectiveness, gloss, shadow)` —
/// the same value as `fmn_render::three_d::SURFACE_SHADING` (fm-0gy).
pub const SURFACE_SHADING: Vec3 = [0.3, 0.2, 0.4];
/// The Reference's `Surface(color=GREY)`.
pub const SURFACE_COLOR: Srgb = GREY;
/// The Reference's `Cube(shading=(0.1, 0.5, 0.1))`.
pub const CUBE_SHADING: Vec3 = [0.1, 0.5, 0.1];
/// The Reference's `VGroup3D(shading=(0.2, 0.2, 0.2))`.
pub const VGROUP3D_SHADING: Vec3 = [0.2, 0.2, 0.2];
/// The Reference's `SurfaceMesh(resolution=(21, 11))` — the wireframe's
/// own (coarser) resolution, distinct from the sampled surface's.
pub const MESH_RESOLUTION: (usize, usize) = (21, 11);
/// The Reference's `SurfaceMesh(normal_nudge=1e-2)` — lifts the wireframe
/// off the surface to avoid z-fighting.
pub const MESH_NORMAL_NUDGE: f64 = 1e-2;

// ---------------------------------------------------------------------------
// small vector helpers (object space is f64, §6.1)

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn mul(v: Vec3, s: f64) -> Vec3 {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn lerp(a: Vec3, b: Vec3, t: f64) -> Vec3 {
    add(a, mul(sub(b, a), t))
}

/// `np.linspace(lo, hi, n)`: `n == 0` gives nothing, `n == 1` gives `lo`.
fn linspace(lo: f64, hi: f64, n: usize) -> Vec<f64> {
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![lo];
    }
    let step = (hi - lo) / (n - 1) as f64;
    (0..n).map(|i| lo + step * i as f64).collect()
}

/// `normalize_along_axis` row semantics: a zero (or non-finite) row
/// normalizes to the zero vector.
fn normalize_or_zero(v: Vec3) -> Vec3 {
    let n = space_ops::get_norm(v);
    if n > 0.0 && n.is_finite() {
        mul(v, 1.0 / n)
    } else {
        [0.0; 3]
    }
}

/// Row-vector `p · mᵀ` (the Reference's `np.dot(points, matrix.T)`).
fn apply_matrix(p: Vec3, m: &fmn_geom::Mat3) -> Vec3 {
    [
        m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2],
        m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2],
        m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2],
    ]
}

#[allow(clippy::cast_possible_truncation)]
fn flat_f32(points: &[Vec3]) -> Vec<f32> {
    points
        .iter()
        .flat_map(|p| p.iter().map(|v| *v as f32))
        .collect()
}

// ---------------------------------------------------------------------------
// record schemas (Reference dtypes, built from the public schema machinery)

/// The Reference's `Surface.data_dtype`:
/// `[('point', f32, 3), ('d_normal_point', f32, 3), ('rgba', f32, 4)]`,
/// with `pointlike_data_keys = ['point', 'd_normal_point']`.
#[must_use]
pub fn surface_schema() -> RecordSchema {
    RecordSchema::new(
        &[("point", 3), ("d_normal_point", 3), ("rgba", 4)],
        &["point"],
        &["point", "d_normal_point"],
    )
    .expect("the surface record schema is ten lanes")
}

/// The Reference's `TexturedSurface.data_dtype`:
/// `[('point', f32, 3), ('d_normal_point', f32, 3), ('im_coords', f32, 2),
/// ('opacity', f32, 1)]`.
#[must_use]
pub fn textured_surface_schema() -> RecordSchema {
    RecordSchema::new(
        &[
            ("point", 3),
            ("d_normal_point", 3),
            ("im_coords", 2),
            ("opacity", 1),
        ],
        &["point"],
        &["point", "d_normal_point"],
    )
    .expect("the textured-surface record schema is nine lanes")
}

/// The Reference's `Surface.compute_triangle_indices`: the fixed six-index
/// pattern over an `(nu, nv)` grid — `(TL, BL, TR)` then `(TR, BL, BR)`
/// per cell, cells in row-major (u-major) order.
#[must_use]
pub fn compute_triangle_indices(resolution: (usize, usize)) -> Vec<u32> {
    let (nu, nv) = resolution;
    if nu == 0 || nv == 0 {
        return Vec::new();
    }
    let mut indices = Vec::with_capacity(6 * nu.saturating_sub(1) * nv.saturating_sub(1));
    for i in 0..nu.saturating_sub(1) {
        for j in 0..nv.saturating_sub(1) {
            let tl = (i * nv + j) as u32;
            let bl = ((i + 1) * nv + j) as u32;
            let tr = (i * nv + j + 1) as u32;
            let br = ((i + 1) * nv + j + 1) as u32;
            indices.extend_from_slice(&[tl, bl, tr, tr, bl, br]);
        }
    }
    indices
}

// ---------------------------------------------------------------------------
// SurfaceSpec: the Reference's Surface.__init__ parameter surface

/// The sampling configuration every `Surface` subclass shares — the
/// Reference's `Surface.__init__` keyword surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceSpec {
    /// `(u_min, u_max)`; default `(0, 1)`.
    pub u_range: (f64, f64),
    /// `(v_min, v_max)`; default `(0, 1)`.
    pub v_range: (f64, f64),
    /// `(nu, nv)` sampled points per axis; default [`SURFACE_RESOLUTION`].
    pub resolution: (usize, usize),
    /// The du/dv step for the normal differences; default
    /// [`SURFACE_EPSILON`].
    pub epsilon: f64,
    /// The `d_normal_point` step-off; default [`SURFACE_NORMAL_NUDGE`].
    pub normal_nudge: f64,
    /// The axis `pointwise_become_partial` prefers (Reference default `1`);
    /// stored for the animation tier.
    pub preferred_creation_axis: usize,
    /// The surface color (Reference `GREY`); written to every rgba record.
    pub color: Srgb,
    /// The surface opacity; default `1.0`.
    pub opacity: f64,
    /// `(reflectiveness, gloss, shadow)`; default [`SURFACE_SHADING`].
    pub shading: Vec3,
    /// The Reference's `depth_test=True`.
    pub depth_test: bool,
}

impl Default for SurfaceSpec {
    fn default() -> Self {
        Self {
            u_range: (0.0, 1.0),
            v_range: (0.0, 1.0),
            resolution: SURFACE_RESOLUTION,
            epsilon: SURFACE_EPSILON,
            normal_nudge: SURFACE_NORMAL_NUDGE,
            preferred_creation_axis: 1,
            color: SURFACE_COLOR,
            opacity: 1.0,
            shading: SURFACE_SHADING,
            depth_test: true,
        }
    }
}

impl SurfaceSpec {
    /// Sample `uv_func` over the grid: points, `d_normal_point`, and the
    /// triangle indices, with the Reference's epsilon semantics (module
    /// docs).
    ///
    /// # Panics
    /// The specification or produced records violate the default sampling
    /// contract. For untrusted input use [`Self::try_sample`] instead.
    #[must_use]
    pub fn sample(&self, uv_func: impl Fn(f64, f64) -> Vec3) -> Surface {
        self.try_sample(|u, v| Ok::<_, std::convert::Infallible>(uv_func(u, v)))
            .expect("surface construction violated its sampling contract")
    }

    /// Sample a fallible, possibly stateful function within the default budget.
    /// The first failed point or derivative probe terminates the loop immediately.
    /// No partial surface is returned and the original error is retained.
    ///
    /// # Errors
    /// Invalid controls, allocation/budget refusal, nonrepresentable records,
    /// or the first callback error.
    pub fn try_sample<E>(
        &self,
        uv_func: impl FnMut(f64, f64) -> Result<Vec3, E>,
    ) -> Result<Surface, SurfaceSampleError<E>> {
        self.try_sample_with_budget(uv_func, SURFACE_SAMPLING_BUDGET)
    }

    /// The explicit-budget form of [`Self::try_sample`]. All dimensions,
    /// columns and triangle storage are admitted before the first callback.
    /// Empty grids and strips retain their ordinary no-triangle meaning.
    ///
    /// # Errors
    /// The same refusals as [`Self::try_sample`], under `budget`.
    pub fn try_sample_with_budget<E>(
        &self,
        mut uv_func: impl FnMut(f64, f64) -> Result<Vec3, E>,
        budget: SamplingBudget,
    ) -> Result<Surface, SurfaceSampleError<E>> {
        let (nu, nv) = self.resolution;
        budget.ensure_total("surface u axis", nu)?;
        budget.ensure_total("surface v axis", nv)?;
        let n = nu
            .checked_mul(nv)
            .ok_or(SamplingError::CapacityOverflow { context: "surface" })?;
        budget.ensure_total("surface", n)?;
        if n > u32::MAX as usize {
            return Err(SamplingError::CapacityOverflow {
                context: "surface indices",
            }
            .into());
        }
        let index_count = nu
            .saturating_sub(1)
            .checked_mul(nv.saturating_sub(1))
            .and_then(|cells| cells.checked_mul(6))
            .ok_or(SamplingError::CapacityOverflow {
                context: "surface triangles",
            })?;
        if !self.epsilon.is_finite() || self.epsilon <= 0.0 {
            return Err(SurfaceSampleError::InvalidControl("epsilon"));
        }
        if !self.normal_nudge.is_finite()
            || self.normal_nudge < 0.0
            || self.normal_nudge > f64::from(f32::MAX)
        {
            return Err(SurfaceSampleError::InvalidControl("normal_nudge"));
        }
        if self.preferred_creation_axis > 1 {
            return Err(SurfaceSampleError::InvalidControl(
                "preferred_creation_axis",
            ));
        }
        let rgba_value = [self.color.r, self.color.g, self.color.b, self.opacity];
        if !rgba_value
            .iter()
            .all(|v| v.is_finite() && v.abs() <= f64::from(f32::MAX))
            || !surface_record(self.shading)
        {
            return Err(SurfaceSampleError::InvalidControl("color/opacity/shading"));
        }
        for (name, range, count) in [("u_range", self.u_range, nu), ("v_range", self.v_range, nv)] {
            if !range.0.is_finite()
                || !range.1.is_finite()
                || !(range.1 - range.0).is_finite()
                || !(range.0 + self.epsilon).is_finite()
                || !(range.1 + self.epsilon).is_finite()
            {
                return Err(SurfaceSampleError::InvalidControl(name));
            }
            for index in 0..count {
                let value = surface_station(range, count, index);
                if !value.is_finite() || !(value + self.epsilon).is_finite() {
                    return Err(SurfaceSampleError::InvalidControl(name));
                }
            }
        }
        let mut points = surface_storage(n)?;
        let mut d_normal_point = surface_storage(n)?;
        let mut rgba = surface_storage(n)?;
        let mut triangle_indices = surface_storage(index_count)?;
        // Every allocation for the sampled value has succeeded before user code.
        for i in 0..nu {
            let u = surface_station(self.u_range, nu, i);
            for j in 0..nv {
                let v = surface_station(self.v_range, nv, j);
                let index = i * nv + j;
                let mut evaluate = |u, v, probe| {
                    let p = uv_func(u, v).map_err(SurfaceSampleError::Callback)?;
                    if !surface_record(p) {
                        return Err(SurfaceSampleError::InvalidSample { index, probe });
                    }
                    Ok(p)
                };
                let p = evaluate(u, v, "point")?;
                let du = sub(evaluate(u + self.epsilon, v, "u derivative")?, p);
                let dv = sub(evaluate(u, v + self.epsilon, "v derivative")?, p);
                let normal = normalize_or_zero(space_ops::cross(du, dv));
                let normal_point = add(p, mul(normal, self.normal_nudge));
                if !surface_record(normal_point) {
                    return Err(SurfaceSampleError::InvalidSample {
                        index,
                        probe: "d_normal_point",
                    });
                }
                d_normal_point.push(normal_point);
                points.push(p);
            }
        }
        rgba.resize(n, rgba_value);
        for i in 0..nu.saturating_sub(1) {
            for j in 0..nv.saturating_sub(1) {
                let tl = (i * nv + j) as u32;
                let bl = ((i + 1) * nv + j) as u32;
                triangle_indices.extend_from_slice(&[tl, bl, tl + 1, tl + 1, bl, bl + 1]);
            }
        }
        let uniforms = Uniforms {
            shading: self.shading,
            depth_test: self.depth_test,
            ..Uniforms::default()
        };

        Ok(Surface {
            points,
            d_normal_point,
            rgba,
            u_range: self.u_range,
            v_range: self.v_range,
            resolution: self.resolution,
            epsilon: self.epsilon,
            normal_nudge: self.normal_nudge,
            preferred_creation_axis: self.preferred_creation_axis,
            triangle_indices,
            uniforms,
            z_index: 0,
        })
    }
}

/// The spec setters every surface-family builder forwards, verbatim.
macro_rules! forward_spec {
    () => {
        /// The `u_range` parameter range.
        #[must_use]
        pub fn u_range(mut self, lo: f64, hi: f64) -> Self {
            self.spec.u_range = (lo, hi);
            self
        }

        /// The `v_range` parameter range.
        #[must_use]
        pub fn v_range(mut self, lo: f64, hi: f64) -> Self {
            self.spec.v_range = (lo, hi);
            self
        }

        /// The `(nu, nv)` sample counts.
        #[must_use]
        pub fn resolution(mut self, nu: usize, nv: usize) -> Self {
            self.spec.resolution = (nu, nv);
            self
        }

        /// The du/dv epsilon step.
        #[must_use]
        pub fn epsilon(mut self, epsilon: f64) -> Self {
            self.spec.epsilon = epsilon;
            self
        }

        /// The `d_normal_point` nudge length.
        #[must_use]
        pub fn normal_nudge(mut self, nudge: f64) -> Self {
            self.spec.normal_nudge = nudge;
            self
        }

        /// The axis preferred by partial-surface creation animations.
        #[must_use]
        pub fn preferred_creation_axis(mut self, axis: usize) -> Self {
            self.spec.preferred_creation_axis = axis;
            self
        }

        /// The surface color (all rgba records).
        #[must_use]
        pub fn color(mut self, color: Srgb) -> Self {
            self.spec.color = color;
            self
        }

        /// The surface opacity.
        #[must_use]
        pub fn opacity(mut self, opacity: f64) -> Self {
            self.spec.opacity = opacity;
            self
        }

        /// The `(reflectiveness, gloss, shadow)` shading triple.
        #[must_use]
        pub fn shading(mut self, shading: Vec3) -> Self {
            self.spec.shading = shading;
            self
        }

        /// The `depth_test` flag.
        #[must_use]
        pub fn depth_test(mut self, depth_test: bool) -> Self {
            self.spec.depth_test = depth_test;
            self
        }
    };
}

// ---------------------------------------------------------------------------
// Surface: the sampled value

/// A sampled parametric surface: the Reference's `Surface` data plane as a
/// detached value.
///
/// Points are u-major (`index = i·nv + j`), `d_normal_point` is the
/// Reference's normal column, and [`Surface::triangle_indices`] is the
/// fixed UV-grid pattern — exactly the triple
/// `fmn_render::three_d::SurfaceMesh::from_uv_grid` consumes (fm-0gy).
#[derive(Debug, Clone, PartialEq)]
pub struct Surface {
    points: Vec<Vec3>,
    d_normal_point: Vec<Vec3>,
    rgba: Vec<[f64; 4]>,
    u_range: (f64, f64),
    v_range: (f64, f64),
    resolution: (usize, usize),
    epsilon: f64,
    normal_nudge: f64,
    preferred_creation_axis: usize,
    triangle_indices: Vec<u32>,
    uniforms: Uniforms,
    z_index: i32,
}

impl Surface {
    /// The sampled grid points, u-major.
    #[must_use]
    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    /// The Reference's `d_normal_point` column.
    #[must_use]
    pub fn d_normal_points(&self) -> &[Vec3] {
        &self.d_normal_point
    }

    /// `get_unit_normals`: `(d_normal_point − point)` normalized per
    /// record; a zero difference yields a zero normal.
    #[must_use]
    pub fn unit_normals(&self) -> Vec<Vec3> {
        self.points
            .iter()
            .zip(&self.d_normal_point)
            .map(|(&p, &d)| normalize_or_zero(sub(d, p)))
            .collect()
    }

    /// The per-point rgba records.
    #[must_use]
    pub fn rgba(&self) -> &[[f64; 4]] {
        &self.rgba
    }

    /// `(nu, nv)` sampled-point counts.
    #[must_use]
    pub fn resolution(&self) -> (usize, usize) {
        self.resolution
    }

    /// The sampled `u` range.
    #[must_use]
    pub fn u_range(&self) -> (f64, f64) {
        self.u_range
    }

    /// The sampled `v` range.
    #[must_use]
    pub fn v_range(&self) -> (f64, f64) {
        self.v_range
    }

    /// The du/dv step the normals were sampled with.
    #[must_use]
    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }

    /// The `d_normal_point` nudge length.
    #[must_use]
    pub fn normal_nudge(&self) -> f64 {
        self.normal_nudge
    }

    /// The axis partial-surface animation prefers (Reference default `1`).
    #[must_use]
    pub fn preferred_creation_axis(&self) -> usize {
        self.preferred_creation_axis
    }

    /// The fixed UV-grid triangle indices.
    #[must_use]
    pub fn triangle_indices(&self) -> &[u32] {
        &self.triangle_indices
    }

    /// The uniform inventory (shading, depth test).
    #[must_use]
    pub fn uniforms(&self) -> &Uniforms {
        &self.uniforms
    }

    /// The scene-list sort key (§8.5).
    #[must_use]
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// `set_color`: overwrite rgb, keep alpha.
    #[must_use]
    pub fn with_color(mut self, color: Srgb) -> Self {
        for rgba in &mut self.rgba {
            rgba[0] = color.r;
            rgba[1] = color.g;
            rgba[2] = color.b;
        }
        self
    }

    /// `set_opacity`: overwrite alpha, keep rgb.
    #[must_use]
    pub fn with_opacity(mut self, opacity: f64) -> Self {
        for rgba in &mut self.rgba {
            rgba[3] = opacity;
        }
        self
    }

    /// `color_by_uv_function`: per-point rgba from the grid `(u, v)` —
    /// the uv-texture coloring hook.
    #[must_use]
    pub fn color_by_uv_function(mut self, f: impl Fn(f64, f64) -> Srgb) -> Self {
        let (nu, nv) = self.resolution;
        let u_values = linspace(self.u_range.0, self.u_range.1, nu);
        let v_values = linspace(self.v_range.0, self.v_range.1, nv);
        let mut index = 0;
        for &u in &u_values {
            for &v in &v_values {
                let c = f(u, v);
                self.rgba[index][0] = c.r;
                self.rgba[index][1] = c.g;
                self.rgba[index][2] = c.b;
                index += 1;
            }
        }
        self
    }

    /// Apply `f` to both pointlike columns (`point` and `d_normal_point`)
    /// — the Reference's `apply_points_function` invariant; normals are
    /// preserved because the difference transforms with the points.
    #[must_use]
    pub fn map_points(mut self, f: impl Fn(Vec3) -> Vec3 + Copy) -> Self {
        for p in &mut self.points {
            *p = f(*p);
        }
        for d in &mut self.d_normal_point {
            *d = f(*d);
        }
        self
    }

    /// `shift(offset)` on both pointlike columns.
    #[must_use]
    pub fn shifted(self, offset: Vec3) -> Self {
        self.map_points(|p| add(p, offset))
    }

    /// `scale(factor, about_point=ORIGIN)`: scaling about the origin.
    #[must_use]
    pub fn scaled(self, factor: f64) -> Self {
        self.map_points(|p| mul(p, factor))
    }

    /// `scale(factor)` with the Reference's defaults: `about_point=None`
    /// resolves `about_edge=ORIGIN`, the bounding-box center. A shape that
    /// is not centered on the origin (a cone on `v ∈ [0, 1]`) stays about
    /// its own center rather than scaling toward the origin.
    #[must_use]
    pub fn scaled_about_center(self, factor: f64) -> Self {
        let center = self.center_point();
        self.map_points(move |p| add(center, mul(sub(p, center), factor)))
    }

    /// `stretch(factor, dim)` with the Reference's defaults: about the
    /// bounding-box center.
    #[must_use]
    pub fn stretched_about_center(self, factor: f64, dim: usize) -> Self {
        let center = self.center_point()[dim];
        self.map_points(move |mut p| {
            p[dim] = center + (p[dim] - center) * factor;
            p
        })
    }

    /// `stretch(factor, dim)` about the origin.
    #[must_use]
    pub fn stretched(self, factor: f64, dim: usize) -> Self {
        self.map_points(move |mut p| {
            p[dim] *= factor;
            p
        })
    }

    /// `rotate(angle, axis)` about the origin.
    #[must_use]
    pub fn rotated(self, angle: f64, axis: Vec3) -> Self {
        let m = space_ops::rotation_matrix(angle, axis);
        self.map_points(move |p| apply_matrix(p, &m))
    }

    /// The axis-aligned bounding box, `None` when point-less.
    #[must_use]
    pub fn extent(&self) -> Option<(Vec3, Vec3)> {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let mut any = false;
        for p in &self.points {
            any = true;
            for d in 0..3 {
                min[d] = min[d].min(p[d]);
                max[d] = max[d].max(p[d]);
            }
        }
        any.then_some((min, max))
    }

    /// `length_over_dim(dim)`.
    #[must_use]
    pub fn length_over_dim(&self, dim: usize) -> f64 {
        self.extent().map_or(0.0, |(min, max)| max[dim] - min[dim])
    }

    /// The bounding-box center (`get_center`).
    #[must_use]
    pub fn center_point(&self) -> Vec3 {
        self.extent().map_or(ORIGIN, |(min, max)| {
            [
                0.5 * (min[0] + max[0]),
                0.5 * (min[1] + max[1]),
                0.5 * (min[2] + max[2]),
            ]
        })
    }

    /// `move_to(point)`: shift so the bounding-box center sits at `point`.
    #[must_use]
    pub fn moved_to(self, point: Vec3) -> Self {
        let c = self.center_point();
        self.shifted(sub(point, c))
    }

    /// `rescale_to_fit(length, dim, stretch)` about the bounding-box center,
    /// as the Reference's `stretch`/`scale` default. A zero current length
    /// is a no-op, exactly as the Reference returns early.
    #[must_use]
    pub fn rescaled_to_fit(self, length: f64, dim: usize, stretch: bool) -> Self {
        let old = self.length_over_dim(dim);
        if old == 0.0 {
            return self;
        }
        let factor = length / old;
        if stretch {
            self.stretched_about_center(factor, dim)
        } else {
            self.scaled_about_center(factor)
        }
    }

    /// `uv_to_point(u, v)`: clipped bilinear interpolation over the sampled
    /// grid — the Reference's exact index arithmetic.
    #[must_use]
    pub fn uv_to_point(&self, u: f64, v: f64) -> Vec3 {
        let (nu, nv) = self.resolution;
        if self.points.is_empty() || nu == 0 || nv == 0 {
            return ORIGIN;
        }
        let clip01 = |x: f64| x.clamp(0.0, 1.0);
        let inverse = |lo: f64, hi: f64, x: f64| (x - lo) / (hi - lo);
        let alpha_u = clip01(inverse(self.u_range.0, self.u_range.1, u));
        let alpha_v = clip01(inverse(self.v_range.0, self.v_range.1, v));
        let scaled_u = alpha_u * (nu - 1) as f64;
        let scaled_v = alpha_v * (nv - 1) as f64;
        let u_int = scaled_u as usize;
        let v_int = scaled_v as usize;
        let u_plus = (u_int + 1).min(nu - 1);
        let v_plus = (v_int + 1).min(nv - 1);
        let a = self.points[u_int * nv + v_int];
        let b = self.points[u_int * nv + v_plus];
        let c = self.points[u_plus * nv + v_int];
        let d = self.points[u_plus * nv + v_plus];
        let u_res = scaled_u % 1.0;
        let v_res = scaled_v % 1.0;
        lerp(lerp(a, b, v_res), lerp(c, d, v_res), u_res)
    }

    /// Overwrite the `d_normal_point` column (Sphere's bespoke radial
    /// normals).
    pub(crate) fn set_d_normal_points(&mut self, column: Vec<Vec3>) {
        if column.len() == self.d_normal_point.len() {
            self.d_normal_point = column;
        }
    }
}

impl From<Surface> for Mobject {
    fn from(s: Surface) -> Self {
        let mut buffer = RecordBuffer::new(surface_schema(), s.points.len())
            .expect("record sizing bounded by the surface grid");
        buffer.write_range("point", 0, &flat_f32(&s.points));
        buffer.write_range("d_normal_point", 0, &flat_f32(&s.d_normal_point));
        #[allow(clippy::cast_possible_truncation)]
        let rgba: Vec<f32> = s
            .rgba
            .iter()
            .flat_map(|c| c.iter().map(|v| *v as f32))
            .collect();
        buffer.write_range("rgba", 0, &rgba);
        Mobject {
            buffer,
            uniforms: s.uniforms,
            shape: ShapeTag::General,
            render_primitive: RenderPrimitive::SurfaceGrid {
                resolution: s.resolution,
            },
            image: None,
            defaults_row: None,
            z_index: s.z_index,
            submobjects: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// ParametricSurface

/// `ParametricSurface(uv_func, u_range, v_range)`: an arbitrary sampled
/// surface — the Reference's escape hatch, holding the callable.
pub struct ParametricSurface {
    uv_func: Box<dyn Fn(f64, f64) -> Vec3>,
    spec: SurfaceSpec,
}

impl std::fmt::Debug for ParametricSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParametricSurface")
            .field("spec", &self.spec)
            .finish_non_exhaustive()
    }
}

impl ParametricSurface {
    /// A surface over `uv_func(u, v)` with the `Surface` defaults
    /// (`u_range=v_range=(0, 1)`, resolution `(101, 101)`).
    #[must_use]
    pub fn new(uv_func: impl Fn(f64, f64) -> Vec3 + 'static) -> Self {
        Self {
            uv_func: Box::new(uv_func),
            spec: SurfaceSpec::default(),
        }
    }

    forward_spec!();

    /// Sample the grid.
    #[must_use]
    pub fn build(self) -> Surface {
        self.spec.sample(self.uv_func)
    }
}

impl From<ParametricSurface> for Surface {
    fn from(p: ParametricSurface) -> Self {
        p.build()
    }
}

impl From<ParametricSurface> for Mobject {
    fn from(p: ParametricSurface) -> Self {
        p.build().into()
    }
}

// ---------------------------------------------------------------------------
// the three_dimensions.py census

/// `Sphere(radius=1.0)`: `u ∈ (0, TAU)`, `v ∈ (0, PI)`,
/// resolution `(101, 51)`, with bespoke radial `true_normals` (module
/// docs).
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    spec: SurfaceSpec,
    radius: f64,
    clockwise: bool,
    true_normals: bool,
}

impl Sphere {
    /// A unit sphere.
    #[must_use]
    pub fn new(radius: f64) -> Self {
        Self {
            spec: SurfaceSpec {
                u_range: (0.0, TAU),
                v_range: (0.0, PI),
                resolution: (101, 51),
                ..SurfaceSpec::default()
            },
            radius,
            clockwise: false,
            true_normals: true,
        }
    }

    forward_spec!();

    /// The Reference's `clockwise` winding flip (negates `u`).
    #[must_use]
    pub fn clockwise(mut self, clockwise: bool) -> Self {
        self.clockwise = clockwise;
        self
    }

    /// Whether to overwrite the sampled normals with radial ones
    /// (Reference default `True`).
    #[must_use]
    pub fn true_normals(mut self, true_normals: bool) -> Self {
        self.true_normals = true_normals;
        self
    }

    /// Reference `Sphere.uv_func`: map angular parameters to the sphere
    /// using the same deterministic transcendental path as construction.
    #[must_use]
    pub fn uv_func(&self, u: f64, v: f64) -> Vec3 {
        let sign = if self.clockwise { -1.0 } else { 1.0 };
        let (sin_u, cos_u) = (fmn_dmath::sin(sign * u), fmn_dmath::cos(sign * u));
        let (sin_v, cos_v) = (fmn_dmath::sin(v), fmn_dmath::cos(v));
        [
            self.radius * cos_u * sin_v,
            self.radius * sin_u * sin_v,
            -self.radius * cos_v,
        ]
    }

    /// Sample the sphere.
    #[must_use]
    pub fn build(self) -> Surface {
        let radius = self.radius;
        let mut surface = self.spec.sample(|u, v| self.uv_func(u, v));
        if self.true_normals && radius != 0.0 {
            let factor = (radius + self.spec.normal_nudge) / radius;
            let column = surface.points().iter().map(|&p| mul(p, factor)).collect();
            surface.set_d_normal_points(column);
        }
        surface
    }
}

impl From<Sphere> for Surface {
    fn from(s: Sphere) -> Self {
        s.build()
    }
}

impl From<Sphere> for Mobject {
    fn from(s: Sphere) -> Self {
        s.build().into()
    }
}

/// `Torus(r1=3.0, r2=1.0)`: major radius `r1`, minor radius `r2`,
/// `u, v ∈ (0, TAU)`, the `Surface` default resolution.
#[derive(Debug, Clone, Copy)]
pub struct Torus {
    spec: SurfaceSpec,
    r1: f64,
    r2: f64,
}

impl Default for Torus {
    fn default() -> Self {
        Self::new(3.0, 1.0)
    }
}

impl Torus {
    /// A torus of major radius `r1` and minor radius `r2`.
    #[must_use]
    pub fn new(r1: f64, r2: f64) -> Self {
        Self {
            spec: SurfaceSpec {
                u_range: (0.0, TAU),
                v_range: (0.0, TAU),
                ..SurfaceSpec::default()
            },
            r1,
            r2,
        }
    }

    forward_spec!();

    /// Reference `Torus.uv_func`: map angular parameters to the torus using
    /// the same deterministic transcendental path as construction.
    #[must_use]
    pub fn uv_func(&self, u: f64, v: f64) -> Vec3 {
        let (sin_u, cos_u) = (fmn_dmath::sin(u), fmn_dmath::cos(u));
        let (sin_v, cos_v) = (fmn_dmath::sin(v), fmn_dmath::cos(v));
        let ring = self.r1 - self.r2 * cos_v;
        [ring * cos_u, ring * sin_u, -self.r2 * sin_v]
    }

    /// Sample the torus.
    #[must_use]
    pub fn build(self) -> Surface {
        self.spec.sample(|u, v| self.uv_func(u, v))
    }
}

impl From<Torus> for Surface {
    fn from(t: Torus) -> Self {
        t.build()
    }
}

impl From<Torus> for Mobject {
    fn from(t: Torus) -> Self {
        t.build().into()
    }
}

/// The post-sampling transform `Cylinder.init_points` shares with `Cone`
/// and `Line3D`: `scale(radius)`, `set_depth(height, stretch=True)`,
/// `apply_matrix(z_to_vector(axis))` — all about the origin, on both
/// pointlike columns.
fn finish_cylinder(surface: Surface, radius: f64, height: f64, axis: Vec3) -> Surface {
    // Reference Cylinder.init_points: scale and set_depth about the sampled
    // shape's center; apply_matrix about the origin.
    let m = space_ops::z_to_vector(axis);
    surface
        .scaled_about_center(radius)
        .rescaled_to_fit(height, 2, true)
        .map_points(move |p| apply_matrix(p, &m))
}

/// `Cylinder(height=2, radius=1, axis=OUT)`: `u ∈ (0, TAU)`,
/// `v ∈ (−1, 1)`, resolution `(101, 11)`.
#[derive(Debug, Clone, Copy)]
pub struct Cylinder {
    spec: SurfaceSpec,
    height: f64,
    radius: f64,
    axis: Vec3,
}

impl Default for Cylinder {
    fn default() -> Self {
        Self::new(2.0, 1.0)
    }
}

impl Cylinder {
    /// A cylinder of the given height and radius about `OUT`.
    #[must_use]
    pub fn new(height: f64, radius: f64) -> Self {
        Self {
            spec: SurfaceSpec {
                u_range: (0.0, TAU),
                v_range: (-1.0, 1.0),
                resolution: (101, 11),
                ..SurfaceSpec::default()
            },
            height,
            radius,
            axis: OUT,
        }
    }

    forward_spec!();

    /// The cylinder axis (Reference `axis=OUT`); the grid is rotated from
    /// `OUT` onto it with `z_to_vector`.
    #[must_use]
    pub fn axis(mut self, axis: Vec3) -> Self {
        self.axis = axis;
        self
    }

    /// The Reference's object-space cylinder parameterization before the
    /// constructor's radius, height, and axis transforms.
    #[must_use]
    pub fn uv_func(u: f64, v: f64) -> Vec3 {
        [fmn_dmath::cos(u), fmn_dmath::sin(u), v]
    }

    /// Sample the cylinder.
    #[must_use]
    pub fn build(self) -> Surface {
        let surface = self.spec.sample(Self::uv_func);
        finish_cylinder(surface, self.radius, self.height, self.axis)
    }
}

impl From<Cylinder> for Surface {
    fn from(c: Cylinder) -> Self {
        c.build()
    }
}

impl From<Cylinder> for Mobject {
    fn from(c: Cylinder) -> Self {
        c.build().into()
    }
}

/// `Cone(height=2, radius=1, axis=OUT)`: a `Cylinder` on `v ∈ (0, 1)`
/// whose radius tapers to the tip. **Not z-centered** (module docs).
#[derive(Debug, Clone, Copy)]
pub struct Cone {
    spec: SurfaceSpec,
    height: f64,
    radius: f64,
    axis: Vec3,
}

impl Default for Cone {
    fn default() -> Self {
        Self::new(2.0, 1.0)
    }
}

impl Cone {
    /// A cone of the given height and base radius about `OUT`.
    #[must_use]
    pub fn new(height: f64, radius: f64) -> Self {
        Self {
            spec: SurfaceSpec {
                u_range: (0.0, TAU),
                v_range: (0.0, 1.0),
                resolution: (101, 11),
                ..SurfaceSpec::default()
            },
            height,
            radius,
            axis: OUT,
        }
    }

    forward_spec!();

    /// The cone axis (Reference `axis=OUT`).
    #[must_use]
    pub fn axis(mut self, axis: Vec3) -> Self {
        self.axis = axis;
        self
    }

    /// Reference `Cone.uv_func`: the object-space tapered cylinder before
    /// the constructor's radius, height, and axis transforms.
    #[must_use]
    pub fn uv_func(u: f64, v: f64) -> Vec3 {
        [
            (1.0 - v) * fmn_dmath::cos(u),
            (1.0 - v) * fmn_dmath::sin(u),
            v,
        ]
    }

    /// Sample the cone.
    #[must_use]
    pub fn build(self) -> Surface {
        let surface = self.spec.sample(Self::uv_func);
        finish_cylinder(surface, self.radius, self.height, self.axis)
    }
}

impl From<Cone> for Surface {
    fn from(c: Cone) -> Self {
        c.build()
    }
}

impl From<Cone> for Mobject {
    fn from(c: Cone) -> Self {
        c.build().into()
    }
}

/// `Line3D(start, end, width=0.05)`: a thin `Cylinder` from `start` to
/// `end`, resolution `(21, 25)`.
#[derive(Debug, Clone, Copy)]
pub struct Line3D {
    spec: SurfaceSpec,
    start: Vec3,
    end: Vec3,
    width: f64,
}

impl Line3D {
    /// A line (thin cylinder) between the two points.
    #[must_use]
    pub fn new(start: Vec3, end: Vec3) -> Self {
        Self {
            spec: SurfaceSpec {
                u_range: (0.0, TAU),
                v_range: (-1.0, 1.0),
                resolution: (21, 25),
                ..SurfaceSpec::default()
            },
            start,
            end,
            width: 0.05,
        }
    }

    forward_spec!();

    /// The line's full width (Reference `width=0.05`); the cylinder radius
    /// is half of it.
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Sample the line.
    #[must_use]
    pub fn build(self) -> Surface {
        let axis = sub(self.end, self.start);
        let surface = self
            .spec
            .sample(|u, v| [fmn_dmath::cos(u), fmn_dmath::sin(u), v]);
        let cylinder = finish_cylinder(surface, self.width / 2.0, space_ops::get_norm(axis), axis);
        cylinder.shifted(mul(add(self.start, self.end), 0.5))
    }
}

impl From<Line3D> for Surface {
    fn from(l: Line3D) -> Self {
        l.build()
    }
}

impl From<Line3D> for Mobject {
    fn from(l: Line3D) -> Self {
        l.build().into()
    }
}

/// `Disk3D(radius=1)`: a filled disc in the xy-plane, `u ∈ (0, 1)` the
/// radius fraction, `v ∈ (0, TAU)` the angle, resolution `(2, 100)`.
///
/// As in the Reference, the `u = 0` row is `nv` coincident center points.
#[derive(Debug, Clone, Copy)]
pub struct Disk3D {
    spec: SurfaceSpec,
    radius: f64,
}

impl Disk3D {
    /// A disc of the given radius.
    #[must_use]
    pub fn new(radius: f64) -> Self {
        Self {
            spec: SurfaceSpec {
                u_range: (0.0, 1.0),
                v_range: (0.0, TAU),
                resolution: (2, 100),
                ..SurfaceSpec::default()
            },
            radius,
        }
    }

    forward_spec!();

    /// Reference `Disk3D.uv_func`: the unit-radius object-space disk before
    /// the constructor's radius scale.
    #[must_use]
    pub fn uv_func(u: f64, v: f64) -> Vec3 {
        [u * fmn_dmath::cos(v), u * fmn_dmath::sin(v), 0.0]
    }

    /// Sample the disc.
    #[must_use]
    pub fn build(self) -> Surface {
        let surface = self.spec.sample(Self::uv_func);
        surface.scaled_about_center(self.radius)
    }
}

impl From<Disk3D> for Surface {
    fn from(d: Disk3D) -> Self {
        d.build()
    }
}

impl From<Disk3D> for Mobject {
    fn from(d: Disk3D) -> Self {
        d.build().into()
    }
}

/// `Square3D(side_length=2.0)`: a flat square in the xy-plane,
/// `u, v ∈ (−1, 1)`, resolution `(2, 2)` — the `Cube` face.
#[derive(Debug, Clone, Copy)]
pub struct Square3D {
    spec: SurfaceSpec,
    side_length: f64,
}

impl Default for Square3D {
    fn default() -> Self {
        Self::new(2.0)
    }
}

impl Square3D {
    /// A square of the given side length, centered on the origin.
    #[must_use]
    pub fn new(side_length: f64) -> Self {
        Self {
            spec: SurfaceSpec {
                u_range: (-1.0, 1.0),
                v_range: (-1.0, 1.0),
                resolution: (2, 2),
                ..SurfaceSpec::default()
            },
            side_length,
        }
    }

    forward_spec!();

    /// Reference `Square3D.uv_func`: the object-space plane before the
    /// constructor's side-length scale.
    #[must_use]
    pub const fn uv_func(u: f64, v: f64) -> Vec3 {
        [u, v, 0.0]
    }

    /// Sample the square.
    #[must_use]
    pub fn build(self) -> Surface {
        let surface = self.spec.sample(Self::uv_func);
        surface.scaled_about_center(self.side_length / 2.0)
    }
}

impl From<Square3D> for Surface {
    fn from(s: Square3D) -> Self {
        s.build()
    }
}

impl From<Square3D> for Mobject {
    fn from(s: Square3D) -> Self {
        s.build().into()
    }
}

/// The geometry `square_to_cube_faces` needs from either face kind
/// (a sampled [`Surface`] or a vectorized [`VMobject`]).
trait CubeFace: Sized {
    fn height(&self) -> f64;
    fn moved_to(self, point: Vec3) -> Self;
    fn rotated(self, angle: f64, axis: Vec3) -> Self;
}

impl CubeFace for Surface {
    fn height(&self) -> f64 {
        self.length_over_dim(1)
    }
    fn moved_to(self, point: Vec3) -> Self {
        Surface::moved_to(self, point)
    }
    fn rotated(self, angle: f64, axis: Vec3) -> Self {
        Surface::rotated(self, angle, axis)
    }
}

impl CubeFace for VMobject {
    fn height(&self) -> f64 {
        self.length_over_dim(1)
    }
    fn moved_to(self, point: Vec3) -> Self {
        VMobject::moved_to(self, point)
    }
    fn rotated(self, angle: f64, axis: Vec3) -> Self {
        self.rotated_about(angle, axis, ORIGIN)
    }
}

/// `square_to_cube_faces(square)`: the face moved to `+z·(height/2)`, its
/// quarter-turns about the four compass directions, and the half-turn
/// back face — six faces, in the Reference's order.
fn square_to_cube_faces<F: CubeFace + Clone>(square: &F) -> Vec<F> {
    let radius = square.height() / 2.0;
    let face = square.clone().moved_to([0.0, 0.0, radius]);
    let mut result = Vec::with_capacity(6);
    result.push(face.clone());
    // RIGHT, UP, LEFT, DOWN — `compass_directions(4)` at the pin; n = 4
    // cannot exceed MAX_COMPASS_DIRECTIONS, so the empty fallback is
    // unreachable in practice.
    let directions = space_ops::compass_directions(4, RIGHT).unwrap_or_default();
    for vect in directions {
        result.push(face.clone().rotated(PI / 2.0, vect));
    }
    result.push(face.rotated(PI, RIGHT));
    result
}

// ---------------------------------------------------------------------------
// SGroup and the Cube/Prism family

/// `SGroup(*parametric_surfaces)`: a point-less `Surface` group (the
/// Reference constructs it with `resolution=(0, 0)`); the children carry
/// the geometry.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SGroup {
    children: Vec<Surface>,
    uniforms: Uniforms,
    z_index: i32,
}

impl SGroup {
    /// A group of surfaces.
    #[must_use]
    pub fn new(children: impl IntoIterator<Item = Surface>) -> Self {
        Self {
            children: children.into_iter().collect(),
            uniforms: Uniforms::default(),
            z_index: 0,
        }
    }

    /// The member surfaces.
    #[must_use]
    pub fn children(&self) -> &[Surface] {
        &self.children
    }

    /// The scene-list sort key (§8.5).
    #[must_use]
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// The combined bounding box over the children.
    #[must_use]
    pub fn extent(&self) -> Option<(Vec3, Vec3)> {
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        let mut any = false;
        for child in &self.children {
            if let Some((lo, hi)) = child.extent() {
                any = true;
                for d in 0..3 {
                    min[d] = min[d].min(lo[d]);
                    max[d] = max[d].max(hi[d]);
                }
            }
        }
        any.then_some((min, max))
    }

    /// `rescale_to_fit(length, dim, stretch=True)` on the whole group
    /// about the origin (the Reference rescales the family; the cube is
    /// origin-centered, so per-child origin transforms compose the same).
    #[must_use]
    pub fn rescaled_to_fit(self, length: f64, dim: usize, stretch: bool) -> Self {
        let Some((min, max)) = self.extent() else {
            return self;
        };
        let old = max[dim] - min[dim];
        if old == 0.0 {
            return self;
        }
        let factor = length / old;
        let map = move |child: Surface| {
            if stretch {
                child.stretched(factor, dim)
            } else {
                child.scaled(factor)
            }
        };
        Self {
            children: self.children.into_iter().map(map).collect(),
            uniforms: self.uniforms,
            z_index: self.z_index,
        }
    }
}

impl From<SGroup> for Mobject {
    fn from(g: SGroup) -> Self {
        Mobject {
            buffer: RecordBuffer::new(surface_schema(), 0)
                .expect("an empty surface buffer cannot overflow"),
            uniforms: g.uniforms,
            shape: ShapeTag::General,
            render_primitive: RenderPrimitive::Vector,
            image: None,
            defaults_row: None,
            z_index: g.z_index,
            submobjects: g.children.into_iter().map(Mobject::from).collect(),
        }
    }
}

/// `Cube(side_length=2)`: six [`Square3D`] faces, `color=BLUE`,
/// `opacity=1`, [`CUBE_SHADING`], `square_resolution=(2, 2)`.
#[derive(Debug, Clone, Copy)]
pub struct Cube {
    side_length: f64,
    color: Srgb,
    opacity: f64,
    shading: Vec3,
    square_resolution: (usize, usize),
}

impl Default for Cube {
    fn default() -> Self {
        Self::new(2.0)
    }
}

impl Cube {
    /// A cube of the given side length, centered on the origin.
    #[must_use]
    pub fn new(side_length: f64) -> Self {
        Self {
            side_length,
            color: BLUE,
            opacity: 1.0,
            shading: CUBE_SHADING,
            square_resolution: (2, 2),
        }
    }

    /// The face color (Reference `BLUE`).
    #[must_use]
    pub fn color(mut self, color: Srgb) -> Self {
        self.color = color;
        self
    }

    /// The face opacity (Reference `1`).
    #[must_use]
    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    /// The face shading triple (Reference `(0.1, 0.5, 0.1)`).
    #[must_use]
    pub fn shading(mut self, shading: Vec3) -> Self {
        self.shading = shading;
        self
    }

    /// The per-face sample counts (Reference `(2, 2)`).
    #[must_use]
    pub fn square_resolution(mut self, nu: usize, nv: usize) -> Self {
        self.square_resolution = (nu, nv);
        self
    }

    /// Build the six-face group.
    #[must_use]
    pub fn build(self) -> SGroup {
        let face = Square3D::new(self.side_length)
            .resolution(self.square_resolution.0, self.square_resolution.1)
            .color(self.color)
            .opacity(self.opacity)
            .shading(self.shading)
            .build();
        SGroup::new(square_to_cube_faces(&face))
    }
}

impl From<Cube> for SGroup {
    fn from(c: Cube) -> Self {
        c.build()
    }
}

impl From<Cube> for Mobject {
    fn from(c: Cube) -> Self {
        c.build().into()
    }
}

/// `Prism(width=3, height=2, depth=1)`: the default cube stretched to the
/// three dimensions.
#[derive(Debug, Clone, Copy)]
pub struct Prism {
    width: f64,
    height: f64,
    depth: f64,
    color: Srgb,
    opacity: f64,
    shading: Vec3,
    square_resolution: (usize, usize),
}

impl Prism {
    /// A prism of the given dimensions, centered on the origin.
    #[must_use]
    pub fn new(width: f64, height: f64, depth: f64) -> Self {
        Self {
            width,
            height,
            depth,
            color: BLUE,
            opacity: 1.0,
            shading: CUBE_SHADING,
            square_resolution: (2, 2),
        }
    }

    /// The face color (Reference `BLUE`).
    #[must_use]
    pub fn color(mut self, color: Srgb) -> Self {
        self.color = color;
        self
    }

    /// The face opacity (Reference `1`).
    #[must_use]
    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    /// The face shading triple (Reference `(0.1, 0.5, 0.1)`).
    #[must_use]
    pub fn shading(mut self, shading: Vec3) -> Self {
        self.shading = shading;
        self
    }

    /// The per-face sample counts (Reference `(2, 2)`).
    #[must_use]
    pub fn square_resolution(mut self, nu: usize, nv: usize) -> Self {
        self.square_resolution = (nu, nv);
        self
    }

    /// Build the stretched cube.
    #[must_use]
    pub fn build(self) -> SGroup {
        let mut group = Cube::new(2.0)
            .color(self.color)
            .opacity(self.opacity)
            .shading(self.shading)
            .square_resolution(self.square_resolution.0, self.square_resolution.1)
            .build();
        for (dim, value) in [self.width, self.height, self.depth]
            .into_iter()
            .enumerate()
        {
            group = group.rescaled_to_fit(value, dim, true);
        }
        group
    }
}

impl From<Prism> for SGroup {
    fn from(p: Prism) -> Self {
        p.build()
    }
}

impl From<Prism> for Mobject {
    fn from(p: Prism) -> Self {
        p.build().into()
    }
}

// ---------------------------------------------------------------------------
// the vectorized 3D groups

/// `VGroup3D(*vmobjects, depth_test=True, shading=(0.2, 0.2, 0.2),
/// joint_type='no_joint')`: a vectorized group whose settings recurse over
/// the family.
#[derive(Debug, Clone)]
pub struct VGroup3D {
    children: Vec<VMobject>,
    depth_test: bool,
    shading: Vec3,
    joint_type: JointType,
}

impl VGroup3D {
    /// A 3D vectorized group over the children.
    #[must_use]
    pub fn new(children: impl IntoIterator<Item = VMobject>) -> Self {
        Self {
            children: children.into_iter().collect(),
            depth_test: true,
            shading: VGROUP3D_SHADING,
            joint_type: JointType::NoJoint,
        }
    }

    /// The `depth_test` flag applied to every member (Reference `True`).
    #[must_use]
    pub fn depth_test(mut self, depth_test: bool) -> Self {
        self.depth_test = depth_test;
        self
    }

    /// The shading triple applied to every member.
    #[must_use]
    pub fn shading(mut self, shading: Vec3) -> Self {
        self.shading = shading;
        self
    }

    /// The joint type applied to every member (Reference `'no_joint'`).
    #[must_use]
    pub fn joint_type(mut self, joint_type: JointType) -> Self {
        self.joint_type = joint_type;
        self
    }

    /// The combined bounding box over the children (recursive: a child
    /// may itself be a group).
    #[must_use]
    pub fn extent(&self) -> Option<(Vec3, Vec3)> {
        fn grow(acc: &mut Option<(Vec3, Vec3)>, mob: &VMobject) {
            if let Some((lo, hi)) = mob.extent() {
                merge(acc, lo, hi);
            }
            for child in mob.children() {
                grow(acc, child);
            }
        }
        fn merge(acc: &mut Option<(Vec3, Vec3)>, lo: Vec3, hi: Vec3) {
            match acc {
                Some((min, max)) => {
                    for d in 0..3 {
                        min[d] = min[d].min(lo[d]);
                        max[d] = max[d].max(hi[d]);
                    }
                }
                None => *acc = Some((lo, hi)),
            }
        }
        let mut acc = None;
        for child in &self.children {
            grow(&mut acc, child);
        }
        acc
    }

    /// `rescale_to_fit(length, dim, stretch=True)` on the whole group
    /// about the origin.
    #[must_use]
    pub fn rescaled_to_fit(self, length: f64, dim: usize, stretch: bool) -> Self {
        let Some((min, max)) = self.extent() else {
            return self;
        };
        let old = max[dim] - min[dim];
        if old == 0.0 {
            return self;
        }
        let factor = length / old;
        let map = move |child: VMobject| {
            if stretch {
                child.stretched_about(factor, dim, ORIGIN)
            } else {
                child.scaled_about(factor, ORIGIN)
            }
        };
        Self {
            children: self.children.into_iter().map(map).collect(),
            depth_test: self.depth_test,
            shading: self.shading,
            joint_type: self.joint_type,
        }
    }

    /// Build the detached vectorized group, applying the 3D settings to
    /// every member (the Reference's recursive `set_shading` /
    /// `set_joint_type` / `apply_depth_test`).
    #[must_use]
    pub fn build(self) -> VMobject {
        let (shading, depth_test, joint_type) = (self.shading, self.depth_test, self.joint_type);
        v_group(self.children.into_iter().map(move |child| {
            child.map_uniforms(move |mut u| {
                u.shading = shading;
                u.depth_test = depth_test;
                u.joint_type = joint_type;
                u
            })
        }))
    }
}

impl From<VGroup3D> for VMobject {
    fn from(g: VGroup3D) -> Self {
        g.build()
    }
}

impl From<VGroup3D> for Mobject {
    fn from(g: VGroup3D) -> Self {
        g.build().into()
    }
}

/// `VCube(side_length=2.0)`: six vectorized `Square` faces,
/// `fill_color=BLUE_D`, `fill_opacity=1`, `stroke_width=0`.
#[derive(Debug, Clone, Copy)]
pub struct VCube {
    side_length: f64,
    fill_color: Srgb,
    fill_opacity: f64,
    stroke_width: f64,
}

impl Default for VCube {
    fn default() -> Self {
        Self::new(2.0)
    }
}

impl VCube {
    /// A vectorized cube of the given side length.
    #[must_use]
    pub fn new(side_length: f64) -> Self {
        Self {
            side_length,
            fill_color: BLUE_D,
            fill_opacity: 1.0,
            stroke_width: 0.0,
        }
    }

    /// The face fill color (Reference `BLUE_D`).
    #[must_use]
    pub fn fill_color(mut self, color: Srgb) -> Self {
        self.fill_color = color;
        self
    }

    /// The face fill opacity (Reference `1`).
    #[must_use]
    pub fn fill_opacity(mut self, opacity: f64) -> Self {
        self.fill_opacity = opacity;
        self
    }

    /// The face stroke width (Reference `0`).
    #[must_use]
    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    fn face(&self) -> VMobject {
        let style = Style::default()
            .fill(self.fill_color, self.fill_opacity)
            .stroke_width(self.stroke_width);
        Rectangle::square(self.side_length)
            .style(style)
            .build()
            .expect("a cube face never requests rounded corners")
    }

    /// Build the six-face vectorized group.
    #[must_use]
    pub fn build(self) -> VMobject {
        VGroup3D::new(square_to_cube_faces(&self.face())).build()
    }
}

impl From<VCube> for VMobject {
    fn from(c: VCube) -> Self {
        c.build()
    }
}

impl From<VCube> for Mobject {
    fn from(c: VCube) -> Self {
        c.build().into()
    }
}

/// `VPrism(width=3, height=2, depth=1)`: the default `VCube` stretched to
/// the three dimensions.
#[derive(Debug, Clone, Copy)]
pub struct VPrism {
    width: f64,
    height: f64,
    depth: f64,
    fill_color: Srgb,
    fill_opacity: f64,
    stroke_width: f64,
}

impl Default for VPrism {
    fn default() -> Self {
        Self::new(3.0, 2.0, 1.0)
    }
}

impl VPrism {
    /// A vectorized prism of the given dimensions.
    #[must_use]
    pub fn new(width: f64, height: f64, depth: f64) -> Self {
        Self {
            width,
            height,
            depth,
            fill_color: BLUE_D,
            fill_opacity: 1.0,
            stroke_width: 0.0,
        }
    }

    /// The face fill color (Reference `BLUE_D`).
    #[must_use]
    pub fn fill_color(mut self, color: Srgb) -> Self {
        self.fill_color = color;
        self
    }

    /// The face fill opacity (Reference `1`).
    #[must_use]
    pub fn fill_opacity(mut self, opacity: f64) -> Self {
        self.fill_opacity = opacity;
        self
    }

    /// The face stroke width (Reference `0`).
    #[must_use]
    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// Build the stretched vectorized cube.
    #[must_use]
    pub fn build(self) -> VMobject {
        let cube = VCube::new(2.0)
            .fill_color(self.fill_color)
            .fill_opacity(self.fill_opacity)
            .stroke_width(self.stroke_width);
        let mut group = VGroup3D::new(square_to_cube_faces(&cube.face()));
        for (dim, value) in [self.width, self.height, self.depth]
            .into_iter()
            .enumerate()
        {
            group = group.rescaled_to_fit(value, dim, true);
        }
        group.build()
    }
}

impl From<VPrism> for VMobject {
    fn from(p: VPrism) -> Self {
        p.build()
    }
}

impl From<VPrism> for Mobject {
    fn from(p: VPrism) -> Self {
        p.build().into()
    }
}

/// `Dodecahedron`: twelve pentagonal faces over the golden-ratio
/// coordinates, `fill=stroke=BLUE_E`, `stroke_width=1`.
#[derive(Debug, Clone, Copy)]
pub struct Dodecahedron {
    fill_color: Srgb,
    fill_opacity: f64,
    stroke_color: Srgb,
    stroke_width: f64,
    shading: Vec3,
}

impl Default for Dodecahedron {
    fn default() -> Self {
        Self::new()
    }
}

impl Dodecahedron {
    /// The Reference's default dodecahedron.
    #[must_use]
    pub fn new() -> Self {
        Self {
            fill_color: BLUE_E,
            fill_opacity: 1.0,
            stroke_color: BLUE_E,
            stroke_width: 1.0,
            shading: VGROUP3D_SHADING,
        }
    }

    /// The face fill color (Reference `BLUE_E`).
    #[must_use]
    pub fn fill_color(mut self, color: Srgb) -> Self {
        self.fill_color = color;
        self
    }

    /// The face fill opacity (Reference `1`).
    #[must_use]
    pub fn fill_opacity(mut self, opacity: f64) -> Self {
        self.fill_opacity = opacity;
        self
    }

    /// The face stroke color (Reference `BLUE_E`).
    #[must_use]
    pub fn stroke_color(mut self, color: Srgb) -> Self {
        self.stroke_color = color;
        self
    }

    /// The face stroke width (Reference `1`).
    #[must_use]
    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// The shading triple (Reference `(0.2, 0.2, 0.2)`).
    #[must_use]
    pub fn shading(mut self, shading: Vec3) -> Self {
        self.shading = shading;
        self
    }

    /// Build the twelve-pentagon group (the Reference's three rotated
    /// pairs plus their negations).
    #[must_use]
    pub fn build(self) -> VMobject {
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
        let style = Style::default()
            .fill(self.fill_color, self.fill_opacity)
            .stroke(self.stroke_color, self.stroke_width, 1.0);
        let face = |vertices: [Vec3; 5]| Polygon::new(vertices).style(style).build();

        // Two pentagons meeting back to back on the positive x-axis.
        let pentagon1 = face([
            [phi, 1.0 / phi, 0.0],
            [1.0, 1.0, 1.0],
            [1.0 / phi, 0.0, phi],
            [1.0, -1.0, 1.0],
            [phi, -1.0 / phi, 0.0],
        ]);
        let pentagon2 = pentagon1
            .clone()
            .stretched_about(-1.0, 2, ORIGIN)
            .reversed_points();
        let x_pair = [pentagon1, pentagon2];
        // `apply_matrix(np.array([z, -x, -y]).T)`: (x, y, z) ↦ (−y, −z, x).
        let z_pair = x_pair
            .clone()
            .map(|p| p.map_points(|q| [-q[1], -q[2], q[0]]));
        // `apply_matrix(np.array([y, z, x]).T)`: (x, y, z) ↦ (z, x, y).
        let y_pair = x_pair.clone().map(|p| p.map_points(|q| [q[2], q[0], q[1]]));

        let mut pentagons: Vec<VMobject> = Vec::with_capacity(12);
        for p in x_pair.into_iter().chain(y_pair).chain(z_pair) {
            pentagons.push(p);
        }
        // The Reference appends each negated, reversed copy after the six
        // originals.
        for p in pentagons.clone() {
            pentagons.push(p.map_points(|q| mul(q, -1.0)).reversed_points());
        }
        VGroup3D::new(pentagons).shading(self.shading).build()
    }
}

impl From<Dodecahedron> for VMobject {
    fn from(d: Dodecahedron) -> Self {
        d.build()
    }
}

impl From<Dodecahedron> for Mobject {
    fn from(d: Dodecahedron) -> Self {
        d.build().into()
    }
}

/// `Prismify(vmobject, depth=1.0, direction=IN)`: extrude a flat
/// vectorized mobject into a prism shell — the base copy, one wall per
/// adjacent anchor pair, and the reversed top copy.
///
/// As the Reference warns, this assumes straight edges: the walls are
/// corner-joined quads over the *anchors*.
#[derive(Debug, Clone)]
pub struct Prismify {
    source: VMobject,
    depth: f64,
    direction: Vec3,
}

impl Prismify {
    /// Extrude `source` by `depth` along `direction` (Reference `IN`).
    #[must_use]
    pub fn new(source: VMobject) -> Self {
        Self {
            source,
            depth: 1.0,
            direction: IN,
        }
    }

    /// The extrusion depth (Reference `1.0`).
    #[must_use]
    pub fn depth(mut self, depth: f64) -> Self {
        self.depth = depth;
        self
    }

    /// The extrusion direction (Reference `IN`).
    #[must_use]
    pub fn direction(mut self, direction: Vec3) -> Self {
        self.direction = direction;
        self
    }

    /// Build the shell: base, walls, reversed top.
    #[must_use]
    pub fn build(self) -> VMobject {
        let vect = mul(self.direction, self.depth);
        let mut pieces = Vec::new();
        pieces.push(self.source.clone());
        // `get_anchors()`: every second point of the shared-anchor layout.
        let anchors: Vec<Vec3> = self.source.points().iter().step_by(2).copied().collect();
        let style = self.source.style();
        for pair in anchors.windows(2) {
            let (p1, p2) = (pair[0], pair[1]);
            let wall = Polygon::polyline([p1, p2, add(p2, vect), add(p1, vect)]).style(style);
            pieces.push(wall.build());
        }
        pieces.push(self.source.shifted(vect).reversed_points());
        VGroup3D::new(pieces).build()
    }
}

impl From<Prismify> for VMobject {
    fn from(p: Prismify) -> Self {
        p.build()
    }
}

impl From<Prismify> for Mobject {
    fn from(p: Prismify) -> Self {
        p.build().into()
    }
}

// ---------------------------------------------------------------------------
// SurfaceMesh: the wireframe overlay

/// `SurfaceMesh(uv_surface, resolution=(21, 11))`: the iso-u/iso-v
/// wireframe over a sampled surface, lifted by `normal_nudge=1e-2` along
/// the surface normals.
///
/// The sampled grid may come from a native surface or from live portal
/// records. Both routes use this one mesher; neither reconstructs a solid
/// from constructor parameters or evaluates its original UV callback.
#[derive(Debug, Clone)]
pub struct SurfaceMesh {
    points: Vec<Vec3>,
    unit_normals: Vec<Vec3>,
    source_resolution: (usize, usize),
    resolution: (usize, usize),
    normal_nudge: f64,
    stroke_width: f64,
    stroke_color: Srgb,
    depth_test: bool,
    joint_type: JointType,
}

/// Why a live sampled grid cannot be admitted to [`SurfaceMesh`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceMeshError {
    /// Resolution and point/normal record counts do not agree.
    InvalidGrid,
    /// A source sample or a nudged point is not finite.
    NonFiniteGeometry,
    /// Nudge/width is not finite, or the stroke width is negative.
    InvalidStyle,
    /// Source or requested wireframe samples exceed the Atlas budget.
    SampleLimit,
}

impl std::fmt::Display for SurfaceMeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidGrid => "SurfaceMesh resolution must match its point and normal records",
            Self::NonFiniteGeometry => {
                "SurfaceMesh points and normals must produce finite geometry"
            }
            Self::InvalidStyle => {
                "SurfaceMesh requires a finite nudge and finite nonnegative stroke width"
            }
            Self::SampleLimit => "SurfaceMesh exceeds the Atlas sampling budget",
        })
    }
}

impl std::error::Error for SurfaceMeshError {}

impl SurfaceMesh {
    /// The wireframe over `uv_surface` at the default `(21, 11)` density.
    #[must_use]
    pub fn new(uv_surface: Surface) -> Self {
        let unit_normals = uv_surface.unit_normals();
        Self::from_parts(uv_surface.points, unit_normals, uv_surface.resolution)
    }

    fn from_parts(
        points: Vec<Vec3>,
        unit_normals: Vec<Vec3>,
        source_resolution: (usize, usize),
    ) -> Self {
        Self {
            points,
            unit_normals,
            source_resolution,
            resolution: MESH_RESOLUTION,
            normal_nudge: MESH_NORMAL_NUDGE,
            stroke_width: 1.0,
            stroke_color: GREY_A,
            depth_test: true,
            joint_type: JointType::NoJoint,
        }
    }

    /// Build over current, u-major sampled records rather than a UV recipe.
    ///
    /// Normals are the caller's unit-normal samples (including a portal
    /// subclass's `get_unit_normals` override). Records are owned snapshots;
    /// later source edits cannot mutate an already constructed wireframe.
    ///
    /// # Errors
    ///
    /// Refuses malformed, non-finite or over-budget grids before meshing.
    pub fn from_samples(
        points: Vec<Vec3>,
        unit_normals: Vec<Vec3>,
        resolution: (usize, usize),
    ) -> Result<Self, SurfaceMeshError> {
        let mesh = Self::from_parts(points, unit_normals, resolution);
        mesh.validate_grid()?;
        Ok(mesh)
    }

    fn validate_grid(&self) -> Result<(), SurfaceMeshError> {
        let budget = SURFACE_SAMPLING_BUDGET.max_samples();
        let (nu, nv) = self.source_resolution;
        let count = nu.checked_mul(nv).ok_or(SurfaceMeshError::SampleLimit)?;
        if count > budget || nu > budget || nv > budget {
            return Err(SurfaceMeshError::SampleLimit);
        }
        if self.points.len() != count || self.unit_normals.len() != count {
            return Err(SurfaceMeshError::InvalidGrid);
        }
        if self
            .points
            .iter()
            .chain(&self.unit_normals)
            .flatten()
            .any(|v| !v.is_finite())
        {
            return Err(SurfaceMeshError::NonFiniteGeometry);
        }
        Ok(())
    }

    /// The bounded entry point for caller-controlled wireframe settings.
    ///
    /// # Errors
    ///
    /// Refuses malformed grids, invalid styles, non-finite nudged points,
    /// and excessive line/sample counts before proportional allocation.
    pub fn try_build(self) -> Result<VMobject, SurfaceMeshError> {
        self.validate_grid()?;
        if !self.normal_nudge.is_finite()
            || !self.stroke_width.is_finite()
            || self.stroke_width < 0.0
        {
            return Err(SurfaceMeshError::InvalidStyle);
        }
        let budget = crate::SamplingBudget::DEFAULT.max_samples();
        let (nu, nv) = self.source_resolution;
        let (part_nu, part_nv) = self.resolution;
        let lines = part_nu
            .checked_add(part_nv)
            .ok_or(SurfaceMeshError::SampleLimit)?;
        let samples = part_nu
            .checked_mul(nv)
            .and_then(|u| part_nv.checked_mul(nu).and_then(|v| u.checked_add(v)))
            .ok_or(SurfaceMeshError::SampleLimit)?;
        if lines > budget || samples > budget {
            return Err(SurfaceMeshError::SampleLimit);
        }
        if self.points.iter().zip(&self.unit_normals).any(|(&p, &n)| {
            add(p, mul(n, self.normal_nudge))
                .iter()
                .any(|v| !v.is_finite())
        }) {
            return Err(SurfaceMeshError::NonFiniteGeometry);
        }
        Ok(self.build())
    }

    /// The wireframe's own `(part_nu, part_nv)` line counts.
    #[must_use]
    pub fn resolution(mut self, nu: usize, nv: usize) -> Self {
        self.resolution = (nu, nv);
        self
    }

    /// The normal nudge (Reference `1e-2`).
    #[must_use]
    pub fn normal_nudge(mut self, nudge: f64) -> Self {
        self.normal_nudge = nudge;
        self
    }

    /// The stroke width of every wireframe path (default `1`).
    #[must_use]
    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// The stroke color of every wireframe path (default `GREY_A`).
    #[must_use]
    pub fn stroke_color(mut self, color: Srgb) -> Self {
        self.stroke_color = color;
        self
    }

    /// The depth-test flag of every wireframe path (default `True`).
    #[must_use]
    pub fn depth_test(mut self, depth_test: bool) -> Self {
        self.depth_test = depth_test;
        self
    }

    /// The joint type of every wireframe path (default `'no_joint'`).
    #[must_use]
    pub fn joint_type(mut self, joint_type: JointType) -> Self {
        self.joint_type = joint_type;
        self
    }

    /// Build the wireframe group: `part_nu` iso-u paths then `part_nv`
    /// iso-v paths, each approx-smoothed (`set_points_smoothly`).
    #[must_use]
    pub fn build(self) -> VMobject {
        let (full_nu, full_nv) = self.source_resolution;
        // Empty sampled surfaces have no iso-curves, including (0, n)
        // and (n, 0). Do not index a missing row or synthesize fake points.
        let (part_nu, part_nv) = if self.points.is_empty() {
            (0, 0)
        } else {
            self.resolution
        };
        let nudged: Vec<Vec3> = self
            .points
            .iter()
            .zip(&self.unit_normals)
            .map(|(&p, &n)| add(p, mul(n, self.normal_nudge)))
            .collect();

        // Float indices between floor and ceil interpolate between the two
        // neighboring sampled rows/columns (the Reference's comment).
        let u_indices = linspace(0.0, full_nu.saturating_sub(1) as f64, part_nu);
        let v_indices = linspace(0.0, full_nv.saturating_sub(1) as f64, part_nv);

        let smooth_path = |samples: &[Vec3]| {
            let mut path = QuadPath::new();
            // On a smoothing failure the corner path remains — the
            // `let _ =` convention of `Polygon::build`.
            let _ = path.set_points_smoothly(samples, true);
            VMobject::from_path(&path).map_uniforms(|mut u| {
                u.depth_test = self.depth_test;
                u.joint_type = self.joint_type;
                u
            })
        };

        let mut children = Vec::with_capacity(part_nu + part_nv);
        for ui in u_indices {
            let ui = ui.clamp(0.0, full_nu.saturating_sub(1) as f64);
            let low = full_nv * (ui.floor() as usize);
            let high = full_nv * (ui.ceil() as usize);
            let t = ui % 1.0;
            let row: Vec<Vec3> = (0..full_nv)
                .map(|j| lerp(nudged[low + j], nudged[high + j], t))
                .collect();
            children.push(smooth_path(&row));
        }
        for vi in v_indices {
            let vi = vi.clamp(0.0, full_nv.saturating_sub(1) as f64);
            let low = vi.floor() as usize;
            let high = vi.ceil() as usize;
            let t = vi % 1.0;
            let col: Vec<Vec3> = (0..full_nu)
                .map(|k| lerp(nudged[low + k * full_nv], nudged[high + k * full_nv], t))
                .collect();
            children.push(smooth_path(&col));
        }

        let mut group = v_group(children)
            .map_style_deep(|s| s.stroke(self.stroke_color, self.stroke_width, s.stroke_opacity));
        group = group.map_uniforms(move |mut u| {
            u.depth_test = self.depth_test;
            u.joint_type = self.joint_type;
            u
        });
        group
    }
}

impl From<SurfaceMesh> for VMobject {
    fn from(m: SurfaceMesh) -> Self {
        m.build()
    }
}

impl From<SurfaceMesh> for Mobject {
    fn from(m: SurfaceMesh) -> Self {
        m.build().into()
    }
}

// ---------------------------------------------------------------------------
// the textured family

/// `TexturedSurface(uv_surface, image_file, dark_image_file=None)`: a
/// sampled surface textured by one image (or a light/dark pair crossfaded
/// by `dark_shift`, fm-0gy).
///
/// Points and `d_normal_point` are copied from `uv_surface`; `im_coords`
/// run `u: 0→1`, `v: 1→0` (the Reference's reversed y); `opacity` is the
/// surface's per-point alpha. The shading triple is the uv surface's.
#[derive(Debug, Clone, PartialEq)]
pub struct TexturedSurface {
    points: Vec<Vec3>,
    d_normal_point: Vec<Vec3>,
    im_coords: Vec<[f64; 2]>,
    opacity: Vec<f64>,
    resolution: (usize, usize),
    u_range: (f64, f64),
    v_range: (f64, f64),
    preferred_creation_axis: usize,
    light_texture: String,
    dark_texture: String,
    num_textures: u8,
    uniforms: Uniforms,
    z_index: i32,
}

impl TexturedSurface {
    /// Texture `uv_surface` with `image_file`; with no dark variant the
    /// light texture serves both ends (`num_textures = 1`).
    #[must_use]
    pub fn new(uv_surface: Surface, image_file: impl Into<String>) -> Self {
        Self::with_dark(uv_surface, image_file, None::<String>)
    }

    /// The two-texture form (`num_textures = 2` when a dark file is
    /// given).
    #[must_use]
    pub fn with_dark(
        uv_surface: Surface,
        image_file: impl Into<String>,
        dark_image_file: Option<impl Into<String>>,
    ) -> Self {
        let light_texture = image_file.into();
        let (dark_texture, num_textures) = match dark_image_file {
            Some(dark) => (dark.into(), 2),
            None => (light_texture.clone(), 1),
        };
        let (nu, nv) = uv_surface.resolution();
        // `for u in linspace(0, 1, nu) for v in linspace(1, 0, nv)` —
        // u-major like the points, y reversed.
        let us = linspace(0.0, 1.0, nu);
        let vs = linspace(1.0, 0.0, nv);
        let mut im_coords = Vec::with_capacity(nu * nv);
        for &u in &us {
            for &v in &vs {
                im_coords.push([u, v]);
            }
        }
        let opacity: Vec<f64> = uv_surface.rgba().iter().map(|c| c[3]).collect();
        let uniforms = Uniforms {
            shading: uv_surface.uniforms().shading,
            depth_test: true,
            ..Uniforms::default()
        };
        Self {
            points: uv_surface.points().to_vec(),
            d_normal_point: uv_surface.d_normal_points().to_vec(),
            im_coords,
            opacity,
            resolution: uv_surface.resolution(),
            u_range: uv_surface.u_range(),
            v_range: uv_surface.v_range(),
            preferred_creation_axis: uv_surface.preferred_creation_axis(),
            light_texture,
            dark_texture,
            num_textures,
            uniforms,
            z_index: 0,
        }
    }

    /// `set_image_coords_by_uv_func`: remap every `im_coord` through
    /// `f(u, v) → (u', v')` — the uv-texture hook.
    #[must_use]
    pub fn set_image_coords_by_uv_func(mut self, f: impl Fn(f64, f64) -> (f64, f64)) -> Self {
        for coord in &mut self.im_coords {
            let (u, v) = f(coord[0], coord[1]);
            *coord = [u, v];
        }
        self
    }

    /// `set_opacity`: overwrite every opacity record.
    #[must_use]
    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity.fill(opacity);
        self
    }

    /// The scene-list sort key (§8.5).
    #[must_use]
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// The copied grid points, u-major.
    #[must_use]
    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    /// The copied `d_normal_point` column.
    #[must_use]
    pub fn d_normal_points(&self) -> &[Vec3] {
        &self.d_normal_point
    }

    /// The texture coordinates (`u: 0→1`, `v: 1→0` by default).
    #[must_use]
    pub fn im_coords(&self) -> &[[f64; 2]] {
        &self.im_coords
    }

    /// The per-point opacities.
    #[must_use]
    pub fn opacities(&self) -> &[f64] {
        &self.opacity
    }

    /// `(nu, nv)` of the source surface.
    #[must_use]
    pub fn resolution(&self) -> (usize, usize) {
        self.resolution
    }

    /// The fixed UV-grid triangle indices (the source surface's grid).
    #[must_use]
    pub fn triangle_indices(&self) -> Vec<u32> {
        compute_triangle_indices(self.resolution)
    }

    /// The light texture path.
    #[must_use]
    pub fn light_texture(&self) -> &str {
        &self.light_texture
    }

    /// The dark texture path (the light path when one texture serves).
    #[must_use]
    pub fn dark_texture(&self) -> &str {
        &self.dark_texture
    }

    /// `1` for a single texture, `2` for the light/dark pair.
    #[must_use]
    pub fn num_textures(&self) -> u8 {
        self.num_textures
    }

    /// The uniform inventory (the uv surface's shading, `depth_test`).
    #[must_use]
    pub fn uniforms(&self) -> &Uniforms {
        &self.uniforms
    }
}

impl From<TexturedSurface> for Mobject {
    fn from(t: TexturedSurface) -> Self {
        let mut buffer = RecordBuffer::new(textured_surface_schema(), t.points.len())
            .expect("record sizing bounded by the surface grid");
        buffer.write_range("point", 0, &flat_f32(&t.points));
        buffer.write_range("d_normal_point", 0, &flat_f32(&t.d_normal_point));
        #[allow(clippy::cast_possible_truncation)]
        let im_coords: Vec<f32> = t
            .im_coords
            .iter()
            .flat_map(|c| c.iter().map(|v| *v as f32))
            .collect();
        buffer.write_range("im_coords", 0, &im_coords);
        #[allow(clippy::cast_possible_truncation)]
        let opacity: Vec<f32> = t.opacity.iter().map(|v| *v as f32).collect();
        buffer.write_range("opacity", 0, &opacity);
        Mobject {
            buffer,
            uniforms: t.uniforms,
            shape: ShapeTag::General,
            render_primitive: RenderPrimitive::SurfaceGrid {
                resolution: t.resolution,
            },
            image: None,
            defaults_row: None,
            z_index: t.z_index,
            submobjects: Vec::new(),
        }
    }
}

/// Why a [`TexturedGeometry`] mesh was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshError {
    /// The face list was not a multiple of three.
    IncompleteTriangle,
    /// A face index named no vertex.
    IndexOutOfRange {
        /// The offending index.
        index: u32,
        /// The vertex count.
        vertices: usize,
    },
    /// One uv per vertex is required (`trimesh.visual.uv`).
    UvLengthMismatch {
        /// Supplied uv count.
        uv: usize,
        /// Vertex count.
        vertices: usize,
    },
}

impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncompleteTriangle => {
                write!(f, "triangle index list must be a multiple of three")
            }
            Self::IndexOutOfRange { index, vertices } => {
                write!(f, "vertex index {index} exceeds the {vertices} vertices")
            }
            Self::UvLengthMismatch { uv, vertices } => {
                write!(f, "got {uv} texture coordinates for {vertices} vertices")
            }
        }
    }
}

impl std::error::Error for MeshError {}

/// `TexturedGeometry(geometry, texture_file)`: an explicitly indexed mesh
/// with a texture — the fm-2u6 part 3 OBJ loader's (`ThreeDModel`)
/// per-geometry leaf.
///
/// **C-4.** The Reference triple-reads `triangle_indices[0::3]` (dead
/// code) and stores zero normals; the ruling is *not replicated* — real
/// area-weighted per-vertex normals are computed from the faces (module
/// docs).
#[derive(Debug, Clone, PartialEq)]
pub struct TexturedGeometry {
    points: Vec<Vec3>,
    d_normal_point: Vec<Vec3>,
    im_coords: Vec<[f64; 2]>,
    opacity: f64,
    triangle_indices: Vec<u32>,
    texture_file: String,
    uniforms: Uniforms,
    z_index: i32,
}

impl TexturedGeometry {
    /// Build from mesh data: `vertices`, flattened triangle `faces`, and
    /// one uv per vertex in the `trimesh` (OpenGL, bottom-origin)
    /// convention — the `v ↦ 1 − v` flip of the Reference's `init_points`
    /// is applied here. Normals are area-weighted per-vertex.
    ///
    /// # Errors
    ///
    /// [`MeshError`] when the face list is incomplete, names a missing
    /// vertex, or the uv count mismatches.
    pub fn from_mesh(
        vertices: Vec<Vec3>,
        faces: &[u32],
        uv: &[[f64; 2]],
        texture_file: impl Into<String>,
    ) -> Result<Self, MeshError> {
        if !faces.len().is_multiple_of(3) {
            return Err(MeshError::IncompleteTriangle);
        }
        for &index in faces {
            if index as usize >= vertices.len() {
                return Err(MeshError::IndexOutOfRange {
                    index,
                    vertices: vertices.len(),
                });
            }
        }
        if uv.len() != vertices.len() {
            return Err(MeshError::UvLengthMismatch {
                uv: uv.len(),
                vertices: vertices.len(),
            });
        }

        // Area-weighted per-vertex normals (the C-4 ruling).
        let mut normals = vec![[0.0; 3]; vertices.len()];
        for tri in faces.as_chunks::<3>().0 {
            let (a, b, c) = (
                vertices[tri[0] as usize],
                vertices[tri[1] as usize],
                vertices[tri[2] as usize],
            );
            let n = space_ops::cross(sub(b, a), sub(c, a));
            for &index in tri {
                let slot = &mut normals[index as usize];
                *slot = add(*slot, n);
            }
        }
        let d_normal_point: Vec<Vec3> = vertices
            .iter()
            .zip(&normals)
            .map(|(&p, &n)| add(p, normalize_or_zero(n)))
            .collect();

        let im_coords: Vec<[f64; 2]> = uv.iter().map(|&[u, v]| [u, 1.0 - v]).collect();
        let uniforms = Uniforms {
            depth_test: true,
            ..Uniforms::default()
        };
        Ok(Self {
            points: vertices,
            d_normal_point,
            im_coords,
            opacity: 1.0,
            triangle_indices: faces.to_vec(),
            texture_file: texture_file.into(),
            uniforms,
            z_index: 0,
        })
    }

    /// The uniform opacity (Reference `1`).
    #[must_use]
    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    /// The scene-list sort key (§8.5).
    #[must_use]
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// The mesh vertices.
    #[must_use]
    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    /// The `d_normal_point` column (real unit normals, one past each
    /// vertex).
    #[must_use]
    pub fn d_normal_points(&self) -> &[Vec3] {
        &self.d_normal_point
    }

    /// The flipped texture coordinates.
    #[must_use]
    pub fn im_coords(&self) -> &[[f64; 2]] {
        &self.im_coords
    }

    /// The flattened triangle indices (`geometry.faces.flatten()`).
    #[must_use]
    pub fn triangle_indices(&self) -> &[u32] {
        &self.triangle_indices
    }

    /// The texture path.
    #[must_use]
    pub fn texture_file(&self) -> &str {
        &self.texture_file
    }

    /// Always `1` (the Reference hard-codes one texture).
    #[must_use]
    pub fn num_textures(&self) -> u8 {
        1
    }

    /// The uniform inventory.
    #[must_use]
    pub fn uniforms(&self) -> &Uniforms {
        &self.uniforms
    }
}

impl From<TexturedGeometry> for Mobject {
    fn from(t: TexturedGeometry) -> Self {
        // TriangleMesh consumes consecutive triples. Expanding only here
        // preserves shared-vertex normal computation while retaining face
        // winding, order, repetition and UV seams in the rendered records.
        let mut buffer = RecordBuffer::new(textured_surface_schema(), t.triangle_indices.len())
            .expect("record sizing bounded by the mesh face count");
        for (row, &index) in t.triangle_indices.iter().enumerate() {
            let index = index as usize; // validated by from_mesh
            #[allow(clippy::cast_possible_truncation)]
            let point = t.points[index].map(|value| value as f32);
            #[allow(clippy::cast_possible_truncation)]
            let normal = t.d_normal_point[index].map(|value| value as f32);
            #[allow(clippy::cast_possible_truncation)]
            let uv = t.im_coords[index].map(|value| value as f32);
            #[allow(clippy::cast_possible_truncation)]
            let opacity = [t.opacity as f32];
            buffer.write(row, "point", &point);
            buffer.write(row, "d_normal_point", &normal);
            buffer.write(row, "im_coords", &uv);
            buffer.write(row, "opacity", &opacity);
        }
        Mobject {
            buffer,
            uniforms: t.uniforms,
            shape: ShapeTag::General,
            render_primitive: RenderPrimitive::TriangleMesh,
            defaults_row: None,
            image: None,
            z_index: t.z_index,
            submobjects: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// tests: structural fixtures over the census

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_mobject::Stage;

    const EPS: f64 = 1e-12;

    fn assert_close(a: Vec3, b: Vec3) {
        for d in 0..3 {
            assert!((a[d] - b[d]).abs() < 1e-9, "component {d}: {a:?} != {b:?}");
        }
    }

    fn extents(surface: &Surface) -> (Vec3, Vec3) {
        surface.extent().unwrap_or(([f64::NAN; 3], [f64::NAN; 3]))
    }

    fn group_extent(group: &VMobject) -> (Vec3, Vec3) {
        VGroup3D::new(group.children().to_vec())
            .extent()
            .unwrap_or(([f64::NAN; 3], [f64::NAN; 3]))
    }

    #[test]
    fn uv_grid_layout_is_u_major() {
        // (nu, nv) = (3, 2) over the unit square: u linspace [0, .5, 1],
        // v linspace [0, 1]; point i*nv+j = (u_i, v_j).
        let surface = ParametricSurface::new(|u, v| [u, v, 0.0])
            .resolution(3, 2)
            .build();
        assert_eq!(surface.points().len(), 6);
        let expected = [
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.5, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        for (p, e) in surface.points().iter().zip(expected) {
            assert_close(*p, e);
        }
        assert_eq!(
            Mobject::from(surface).render_primitive,
            RenderPrimitive::SurfaceGrid { resolution: (3, 2) }
        );
    }

    #[test]
    fn triangle_indices_match_the_six_index_pattern() {
        // (3, 2): index grid [[0,1],[2,3],[4,5]] — two cells.
        let indices = compute_triangle_indices((3, 2));
        assert_eq!(indices, vec![0, 2, 1, 1, 2, 3, 2, 4, 3, 3, 4, 5]);
        assert_eq!(compute_triangle_indices((0, 0)), Vec::<u32>::new());
        assert_eq!(compute_triangle_indices((101, 51)).len(), 6 * 100 * 50);
    }

    #[test]
    fn normals_come_from_epsilon_forward_differences() {
        // f(u, v) = (u, v, u²): df/du = (1, 0, 2u), df/dv = (0, 1, 0).
        // The normal at u = 0.5 uses f(0.5 + ε) — the forward difference,
        // not the analytic tangent.
        let epsilon = 1e-3;
        let surface = ParametricSurface::new(|u, v| [u, v, u * u])
            .resolution(3, 2)
            .epsilon(epsilon)
            .build();
        let normals = surface.unit_normals();
        // Grid point (0.5, 0.0) is index 2; du = f(u+ε)−f(u) componentwise.
        let du = [epsilon, 0.0, 2.0 * 0.5 * epsilon + epsilon * epsilon];
        let dv = [0.0, epsilon, 0.0];
        let expected = normalize_or_zero(space_ops::cross(du, dv));
        assert_close(normals[2], expected);
        assert!((space_ops::get_norm(normals[2]) - 1.0).abs() < 1e-9);
        // d_normal_point = point + nudge · normal.
        let d = surface.d_normal_points()[2];
        let p = surface.points()[2];
        assert_close(sub(d, p), mul(expected, surface.normal_nudge()));
    }

    #[test]
    fn epsilon_difference_is_not_clamped_at_u_max() {
        // At u = 1 the Reference samples f(1 + ε): the recorded normal at
        // the last grid row reflects the function *beyond* the range.
        let epsilon = 1e-3;
        let surface = ParametricSurface::new(|u, v| [u, v, u * u])
            .resolution(3, 2)
            .epsilon(epsilon)
            .build();
        let normals = surface.unit_normals();
        // Index 4 is (1.0, 0.0): the forward difference samples f(1+ε).
        let du = [epsilon, 0.0, 2.0 * epsilon + epsilon * epsilon];
        let expected = normalize_or_zero(space_ops::cross(du, [0.0, epsilon, 0.0]));
        assert_close(normals[4], expected);
    }

    #[test]
    fn uv_to_point_is_clipped_bilinear() {
        let surface = ParametricSurface::new(|u, v| [u, v, 0.0])
            .resolution(3, 2)
            .build();
        assert_close(surface.uv_to_point(0.25, 0.75), [0.25, 0.75, 0.0]);
        assert_close(surface.uv_to_point(0.5, 1.0), [0.5, 1.0, 0.0]);
        // Out-of-range uv clips into the grid.
        assert_close(surface.uv_to_point(-1.0, 5.0), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn sphere_defaults_and_true_normals() {
        let sphere = Sphere::new(1.0).build();
        assert_eq!(sphere.resolution(), (101, 51));
        assert_eq!(sphere.points().len(), 101 * 51);
        // First grid point: (u, v) = (0, 0) → (0, 0, −radius).
        assert_close(sphere.points()[0], [0.0, 0.0, -1.0]);
        let (min, max) = extents(&sphere);
        for d in 0..3 {
            assert!((min[d] + 1.0).abs() < 1e-9, "min[{d}] = {}", min[d]);
            assert!((max[d] - 1.0).abs() < 1e-9, "max[{d}] = {}", max[d]);
        }
        // true_normals: unit normals are exactly radial — at the pole the
        // sampled cross product would be degenerate, this is (0, 0, −1).
        let normals = sphere.unit_normals();
        assert_close(normals[0], [0.0, 0.0, -1.0]);
        for (n, p) in normals.iter().zip(sphere.points()) {
            assert_close(*n, normalize_or_zero(*p));
        }
    }

    #[test]
    fn sphere_radius_scales_extents() {
        let sphere = Sphere::new(2.5).build();
        let (min, max) = extents(&sphere);
        for d in 0..3 {
            assert!((min[d] + 2.5).abs() < 1e-9);
            assert!((max[d] - 2.5).abs() < 1e-9);
        }
        assert_close(sphere.points()[0], [0.0, 0.0, -2.5]);
    }

    #[test]
    fn torus_radii_fix_extents() {
        let builder = Torus::new(3.0, 1.0);
        assert_close(builder.uv_func(0.0, 0.0), [2.0, 0.0, 0.0]);
        assert_close(builder.uv_func(0.0, PI), [4.0, 0.0, 0.0]);
        let torus = builder.build();
        assert_eq!(torus.resolution(), SURFACE_RESOLUTION);
        assert_eq!(torus.points().len(), 101 * 101);
        let (min, max) = extents(&torus);
        assert!((min[0] + 4.0).abs() < 1e-9 && (max[0] - 4.0).abs() < 1e-9);
        assert!((min[1] + 4.0).abs() < 1e-9 && (max[1] - 4.0).abs() < 1e-9);
        assert!((min[2] + 1.0).abs() < 1e-9 && (max[2] - 1.0).abs() < 1e-9);
        // (u, v) = (0, 0) → (r1 − r2, 0, 0).
        assert_close(torus.points()[0], [2.0, 0.0, 0.0]);
    }

    #[test]
    fn cylinder_is_centered_with_unit_cross_section() {
        let cylinder = Cylinder::new(2.0, 1.0).build();
        assert_eq!(cylinder.resolution(), (101, 11));
        assert_eq!(cylinder.points().len(), 101 * 11);
        let (min, max) = extents(&cylinder);
        assert!((min[0] + 1.0).abs() < 1e-9 && (max[0] - 1.0).abs() < 1e-9);
        assert!((min[1] + 1.0).abs() < 1e-9 && (max[1] - 1.0).abs() < 1e-9);
        assert!((min[2] + 1.0).abs() < 1e-9 && (max[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cylinder_axis_rotation() {
        let cylinder = Cylinder::new(4.0, 0.5).axis(RIGHT).build();
        let (min, max) = extents(&cylinder);
        // Depth now lies along x.
        assert!((min[0] + 2.0).abs() < 1e-9 && (max[0] - 2.0).abs() < 1e-9);
        assert!((min[1] + 0.5).abs() < 1e-9 && (max[1] - 0.5).abs() < 1e-9);
        assert!((min[2] + 0.5).abs() < 1e-9 && (max[2] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn cone_is_placed_about_its_center_like_the_reference() {
        assert_close(Cone::uv_func(0.0, 0.0), [1.0, 0.0, 0.0]);
        assert_close(Cone::uv_func(0.0, 1.0), [0.0, 0.0, 1.0]);
        let cone = Cone::new(2.0, 1.0).build();
        let (min, max) = extents(&cone);
        assert!((min[0] + 1.0).abs() < 1e-9 && (max[0] - 1.0).abs() < 1e-9);
        // set_depth stretches the unit cone (z ∈ [0, 1]) about its center
        // z = ½: the pinned Reference's Cone() spans z ∈ [−½, 3⁄2].
        assert!((min[2] + 0.5).abs() < 1e-9 && (max[2] - 1.5).abs() < 1e-9);
        // The v = 1 row is the tip: last grid point.
        let last = cone.points()[cone.points().len() - 1];
        assert_close(last, [0.0, 0.0, 1.5]);
        // scale(radius) is about the center too, before set_depth.
        let wide = Cone::new(2.0, 2.0).build();
        let (min, max) = extents(&wide);
        assert!((min[0] + 2.0).abs() < 1e-9 && (max[0] - 2.0).abs() < 1e-9);
        assert!((min[2] + 0.5).abs() < 1e-9 && (max[2] - 1.5).abs() < 1e-9);
    }

    #[test]
    fn line3d_runs_from_start_to_end() {
        let line = Line3D::new([0.0, 0.0, 0.0], [0.0, 0.0, 2.0]).build();
        assert_eq!(line.resolution(), (21, 25));
        assert_eq!(line.points().len(), 21 * 25);
        let (min, max) = extents(&line);
        assert!(min[2].abs() < 1e-9 && (max[2] - 2.0).abs() < 1e-9);
        // Radius = width/2 = 0.025 about the segment.
        assert!((min[0] + 0.025).abs() < 1e-9 && (max[0] - 0.025).abs() < 1e-9);
        assert!((line.center_point()[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn line3d_degenerate_endpoints_do_not_blow_up() {
        let line = Line3D::new([1.0, 1.0, 1.0], [1.0, 1.0, 1.0]).build();
        assert_eq!(line.points().len(), 21 * 25);
        for p in line.points() {
            assert!(p.iter().all(|c| c.is_finite()), "non-finite {p:?}");
        }
    }

    #[test]
    fn disk3d_has_coincident_center_row() {
        assert_close(Disk3D::uv_func(0.5, 0.0), [0.5, 0.0, 0.0]);
        let disk = Disk3D::new(2.0).build();
        assert_eq!(disk.resolution(), (2, 100));
        assert_eq!(disk.points().len(), 200);
        // scale(radius) is about the sampled unit disk's bounding-box center,
        // as in the Reference. The +x rim (v = 0) is a grid point; −x falls
        // between grid angles, so that center sits (1 + cos(49·TAU/99))/2
        // right of the origin, and the scaled disk drifts left by it.
        let offset = 0.5 * (1.0 + (49.0 * TAU / 99.0).cos());
        // u = 0: 100 copies of the center.
        for p in &disk.points()[..100] {
            assert_close(*p, [-offset, 0.0, 0.0]);
        }
        let (min, max) = extents(&disk);
        assert!((max[0] - (2.0 - offset)).abs() < 1e-9);
        assert!(min[0] > -2.0 && min[0] < -1.998, "min x = {}", min[0]);
        assert!(min[1] > -2.0 && min[1] < -1.999, "min y = {}", min[1]);
        assert!(max[1] < 2.0 && max[1] > 1.999, "max y = {}", max[1]);
        assert!(min[2].abs() < EPS && max[2].abs() < EPS);
        // Coarse angular sampling makes the drift visible: 12 angles leave
        // −x at cos(5·TAU/11), and radius 3 moves the center by 2× that offset.
        let coarse = Disk3D::new(3.0).resolution(3, 12).build();
        let offset = 0.5 * (1.0 + (5.0 * TAU / 11.0).cos());
        assert_close(coarse.points()[0], [-2.0 * offset, 0.0, 0.0]);
    }

    #[test]
    fn square3d_corners() {
        assert_close(Square3D::uv_func(-1.0, 1.0), [-1.0, 1.0, 0.0]);
        let square = Square3D::new(2.0).build();
        assert_eq!(square.resolution(), (2, 2));
        assert_eq!(square.points().len(), 4);
        let expected = [
            [-1.0, -1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        for (p, e) in square.points().iter().zip(expected) {
            assert_close(*p, e);
        }
    }

    #[test]
    fn cube_has_six_faces_and_unit_corners() {
        let cube = Cube::new(2.0).build();
        assert_eq!(cube.children().len(), 6);
        for face in cube.children() {
            assert_eq!(face.points().len(), 4);
            assert!((face.uniforms().shading[0] - 0.1).abs() < EPS);
        }
        let (min, max) = cube.extent().unwrap_or(([f64::NAN; 3], [f64::NAN; 3]));
        for d in 0..3 {
            assert!((min[d] + 1.0).abs() < 1e-9);
            assert!((max[d] - 1.0).abs() < 1e-9);
        }
        // Every face is a unit square offset one unit along its axis.
        assert!((cube.children()[0].center_point()[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn prism_stretches_the_cube() {
        let prism = Prism::new(3.0, 2.0, 1.0).build();
        assert_eq!(prism.children().len(), 6);
        let (min, max) = prism.extent().unwrap_or(([f64::NAN; 3], [f64::NAN; 3]));
        assert!((min[0] + 1.5).abs() < 1e-9 && (max[0] - 1.5).abs() < 1e-9);
        assert!((min[1] + 1.0).abs() < 1e-9 && (max[1] - 1.0).abs() < 1e-9);
        assert!((min[2] + 0.5).abs() < 1e-9 && (max[2] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn vcube_faces_carry_3d_uniforms() {
        let vcube = VCube::new(2.0).build();
        assert_eq!(vcube.children().len(), 6);
        let (min, max) = group_extent(&vcube);
        for d in 0..3 {
            assert!((min[d] + 1.0).abs() < 1e-9 && (max[d] - 1.0).abs() < 1e-9);
        }
        for face in vcube.children() {
            let style = face.style();
            assert_eq!(style.fill_color, BLUE_D);
            assert!((style.fill_opacity - 1.0).abs() < EPS);
            assert!(style.stroke_width.abs() < EPS);
        }
    }

    #[test]
    fn vprism_stretches_the_vcube() {
        let vprism = VPrism::new(3.0, 2.0, 1.0).build();
        let (min, max) = group_extent(&vprism);
        assert!((min[0] + 1.5).abs() < 1e-9 && (max[0] - 1.5).abs() < 1e-9);
        assert!((min[1] + 1.0).abs() < 1e-9 && (max[1] - 1.0).abs() < 1e-9);
        assert!((min[2] + 0.5).abs() < 1e-9 && (max[2] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn vgroup3d_settings_recurse() {
        let face = Rectangle::square(2.0)
            .build()
            .expect("the test face is unrounded");
        let mobject: Mobject = VGroup3D::new([face]).build().into();
        assert_eq!(mobject.submobjects.len(), 1);
        let uniforms = &mobject.submobjects[0].uniforms;
        assert_eq!(uniforms.shading, VGROUP3D_SHADING);
        assert!(uniforms.depth_test);
        assert_eq!(uniforms.joint_type, JointType::NoJoint);
    }

    #[test]
    fn dodecahedron_vertex_inventory() {
        let dodeca = Dodecahedron::new().build();
        assert_eq!(dodeca.children().len(), 12);
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
        // Every child is a closed pentagon: 5 vertices + repeat anchor →
        // 6 anchors → 11 shared-anchor points.
        for face in dodeca.children() {
            assert_eq!(face.points().len(), 11);
        }
        let (min, max) = group_extent(&dodeca);
        // Vertex coordinates are permutations of (±1, ±1, ±1),
        // (0, ±1/φ, ±φ), (±1/φ, ±φ, 0), (±φ, 0, ±1/φ): extent ±φ.
        for d in 0..3 {
            assert!((min[d] + phi).abs() < 1e-9, "min[{d}] = {} ≠ {phi}", min[d]);
            assert!((max[d] - phi).abs() < 1e-9, "max[{d}] = {} ≠ {phi}", max[d]);
        }
        // The 20 unique vertices each meet 3 faces: 60 corners over 12
        // pentagons, 20 distinct values.
        let mut corners: Vec<Vec3> = Vec::new();
        for face in dodeca.children() {
            for p in face.points().iter().step_by(2) {
                corners.push(*p);
            }
        }
        assert_eq!(corners.len(), 12 * 6);
        let mut unique: Vec<Vec3> = Vec::new();
        'outer: for c in corners {
            for u in &unique {
                if space_ops::get_dist(*u, c) < 1e-9 {
                    continue 'outer;
                }
            }
            unique.push(c);
        }
        assert_eq!(unique.len(), 20);
    }

    #[test]
    fn prismify_builds_base_walls_and_top() {
        let tri = Polygon::new([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]).build();
        // Closed polygon: anchors = 3 corners + repeat = 4 → 3 walls.
        let shell = Prismify::new(tri).build();
        assert_eq!(shell.children().len(), 1 + 3 + 1);
        // The top copy is shifted by depth·IN and reversed: its z is −1.
        let top = &shell.children()[4];
        let (min, max) = top.extent().unwrap_or(([f64::NAN; 3], [f64::NAN; 3]));
        assert!((min[2] + 1.0).abs() < 1e-9 && (max[2] + 1.0).abs() < 1e-9);
        // A wall spans z ∈ [−1, 0].
        let wall = &shell.children()[1];
        let (wmin, wmax) = wall.extent().unwrap_or(([f64::NAN; 3], [f64::NAN; 3]));
        assert!((wmin[2] + 1.0).abs() < 1e-9 && wmax[2].abs() < 1e-9);
    }

    #[test]
    fn surface_mesh_counts_and_nudge() {
        // A (4, 3) surface: wireframe at (2, 2) → 2 + 2 = 4 paths; u
        // paths have 3 anchors (5 points smoothed), v paths 4 (7 points).
        let surface = ParametricSurface::new(|u, v| [u, v, 0.0])
            .resolution(4, 3)
            .build();
        let mesh = SurfaceMesh::new(surface).resolution(2, 2).build();
        assert_eq!(mesh.children().len(), 4);
        assert_eq!(mesh.children()[0].points().len(), 5);
        assert_eq!(mesh.children()[2].points().len(), 7);
    }

    #[test]
    fn surface_mesh_lifts_along_normals() {
        // One iso line exactly at a sampled row over the plane: nudged by
        // 1e-2 along +z (the plane's normal).
        let surface = ParametricSurface::new(|u, v| [u, v, 0.0])
            .resolution(4, 3)
            .build();
        let mesh = SurfaceMesh::new(surface).resolution(1, 1).build();
        assert_eq!(mesh.children().len(), 2);
        // The single u-index is 0 → the v = 0 row: points (u, 0, 1e-2).
        let path = &mesh.children()[0];
        let first = path.points()[0];
        assert!((first[2] - 1e-2).abs() < 1e-9, "z = {}", first[2]);
    }

    #[test]
    fn surface_mesh_fractional_index_interpolates() {
        // part_nu = 2 over full_nu = 4: u indices {0, 3}; make it 3 →
        // {0, 1.5, 3}: the middle path interpolates rows 1 and 2 at t=.5.
        let surface = ParametricSurface::new(|u, v| [u, v, u])
            .resolution(4, 2)
            .build();
        let mesh = SurfaceMesh::new(surface)
            .resolution(3, 1)
            .normal_nudge(0.0)
            .build();
        // Middle u-path (index 1): u = 1.5/3 = 0.5 everywhere.
        let mid = &mesh.children()[1];
        let anchor = mid.points()[0];
        assert!((anchor[0] - 0.5).abs() < 1e-9, "x = {}", anchor[0]);
    }

    #[test]
    fn surface_mesh_live_grid_matches_native_surface() {
        let surface = ParametricSurface::new(|u, v| [u, v, u * v])
            .resolution(5, 3)
            .build()
            .stretched(2.0, 0)
            .shifted([3.0, -2.0, 0.5]);
        let live = SurfaceMesh::from_samples(
            surface.points().to_vec(),
            surface.unit_normals(),
            surface.resolution(),
        )
        .expect("valid sampled surface")
        .resolution(4, 2)
        .normal_nudge(0.15)
        .try_build()
        .expect("bounded mesh");
        let native = SurfaceMesh::new(surface)
            .resolution(4, 2)
            .normal_nudge(0.15)
            .build();
        assert_eq!(live, native);
    }

    #[test]
    fn surface_mesh_honors_live_normals_and_leaf_style() {
        let points = vec![
            [1.0, 0.0, 2.0],
            [1.0, 1.0, 2.0],
            [3.0, 0.0, 2.0],
            [3.0, 1.0, 2.0],
        ];
        let mesh = SurfaceMesh::from_samples(points, vec![RIGHT; 4], (2, 2))
            .expect("grid")
            .resolution(3, 1)
            .normal_nudge(0.25)
            .stroke_color(BLUE)
            .stroke_width(3.5)
            .depth_test(true)
            .joint_type(JointType::Bevel)
            .try_build()
            .expect("mesh");
        assert_eq!(mesh.children().len(), 4);
        assert_eq!(mesh.children()[1].points()[0], [2.25, 0.0, 2.0]);
        for child in mesh.children() {
            assert_eq!(child.style().stroke_color, BLUE);
            assert_eq!(child.style().stroke_width, 3.5);
            assert_eq!(child.style().fill_opacity, 0.0);
            assert!(child.uniforms().depth_test);
            assert_eq!(child.uniforms().joint_type, JointType::Bevel);
        }
    }

    #[test]
    fn surface_mesh_empty_and_singleton_grids_do_not_index_missing_rows() {
        for resolution in [(0, 0), (0, 3), (3, 0)] {
            let mesh = SurfaceMesh::from_samples(vec![], vec![], resolution)
                .expect("empty grid")
                .try_build()
                .expect("empty mesh");
            assert!(mesh.children().is_empty());
        }
        let mesh = SurfaceMesh::from_samples(vec![ORIGIN], vec![OUT], (1, 1))
            .expect("singleton")
            .resolution(1, 1)
            .try_build()
            .expect("mesh");
        assert_eq!(mesh.children().len(), 2);
        assert_eq!(
            mesh.children()[0].points(),
            &[[0.0, 0.0, MESH_NORMAL_NUDGE]]
        );
    }

    #[test]
    fn surface_mesh_refuses_malformed_and_unbounded_grid_data() {
        assert_eq!(
            SurfaceMesh::from_samples(vec![ORIGIN], vec![], (1, 1)).unwrap_err(),
            SurfaceMeshError::InvalidGrid
        );
        assert_eq!(
            SurfaceMesh::from_samples(vec![], vec![], (usize::MAX, 2)).unwrap_err(),
            SurfaceMeshError::SampleLimit
        );
        assert_eq!(
            SurfaceMesh::from_samples(vec![[f64::NAN, 0.0, 0.0]], vec![OUT], (1, 1)).unwrap_err(),
            SurfaceMeshError::NonFiniteGeometry
        );
        assert_eq!(
            SurfaceMesh::from_samples(vec![ORIGIN], vec![[0.0, f64::INFINITY, 0.0]], (1, 1))
                .unwrap_err(),
            SurfaceMeshError::NonFiniteGeometry
        );
        let mesh = SurfaceMesh::from_samples(vec![ORIGIN], vec![OUT], (1, 1)).expect("grid");
        assert_eq!(
            mesh.clone()
                .resolution(usize::MAX, 1)
                .try_build()
                .unwrap_err(),
            SurfaceMeshError::SampleLimit
        );
        assert_eq!(
            mesh.clone().normal_nudge(f64::NAN).try_build().unwrap_err(),
            SurfaceMeshError::InvalidStyle
        );
        assert_eq!(
            mesh.stroke_width(-1.0).try_build().unwrap_err(),
            SurfaceMeshError::InvalidStyle
        );
    }

    #[test]
    fn textured_surface_copies_grid_and_flips_v() {
        let surface = Square3D::new(2.0).build().with_opacity(0.5);
        let textured = TexturedSurface::new(surface, "grid.png");
        assert_eq!(textured.points().len(), 4);
        assert_eq!(textured.num_textures(), 1);
        assert_eq!(textured.light_texture(), "grid.png");
        assert_eq!(textured.dark_texture(), "grid.png");
        // im_coords: u outer 0→1, v inner 1→0.
        let expected = [[0.0, 1.0], [0.0, 0.0], [1.0, 1.0], [1.0, 0.0]];
        for (c, e) in textured.im_coords().iter().zip(expected) {
            assert!((c[0] - e[0]).abs() < EPS && (c[1] - e[1]).abs() < EPS);
        }
        // Opacity comes from the surface alpha.
        for o in textured.opacities() {
            assert!((o - 0.5).abs() < EPS);
        }
    }

    #[test]
    fn textured_surface_two_textures_and_uv_hook() {
        let surface = Square3D::new(2.0).build();
        let textured = TexturedSurface::with_dark(surface, "light.png", Some("dark.png"));
        assert_eq!(textured.num_textures(), 2);
        assert_eq!(textured.dark_texture(), "dark.png");
        let remapped = textured.set_image_coords_by_uv_func(|u, v| (1.0 - u, v * 2.0));
        assert!((remapped.im_coords()[0][0] - 1.0).abs() < EPS);
        assert!((remapped.im_coords()[0][1] - 2.0).abs() < EPS);
    }

    #[test]
    fn textured_surface_arena_records() {
        let surface = Square3D::new(2.0).build();
        let mut stage = Stage::new();
        let mob = stage.add(TexturedSurface::new(surface, "grid.png"));
        let points = stage.get_points(mob).unwrap_or_default();
        assert_eq!(points.len(), 4);
    }

    #[test]
    fn surface_enters_the_arena_with_normal_column() {
        let mut stage = Stage::new();
        let mob = stage.add(Sphere::new(1.0).resolution(3, 2).build());
        let points = stage.get_points(mob).unwrap_or_default();
        assert_eq!(points.len(), 6);
        assert_close(points[0], [0.0, 0.0, -1.0]);
        // The d_normal_point column is in the arena record, radial for a
        // true-normals sphere: d_normal_point[0] = (0, 0, −1.001).
        let Some(entry) = stage.get(mob) else {
            return;
        };
        let column = entry
            .buffer
            .read_column("d_normal_point")
            .unwrap_or_default();
        assert_eq!(column.len(), 18);
        assert!((f64::from(column[2]) + 1.001).abs() < 1e-5);
    }

    #[test]
    fn sgroup_enters_the_arena_as_a_family() {
        let mut stage = Stage::new();
        let mob = stage.add(Cube::new(2.0).build());
        // Root (point-less) + 6 faces.
        assert!(stage.get_points(mob).unwrap_or_default().is_empty());
        assert_eq!(stage.family(mob).len(), 7);
    }

    #[test]
    fn textured_geometry_tetrahedron() {
        let vertices = vec![
            [1.0, 1.0, 1.0],
            [1.0, -1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
        ];
        let faces: Vec<u32> = vec![0, 1, 2, 0, 3, 1, 0, 2, 3, 1, 3, 2];
        let uv = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.25, 0.75]];
        let mesh = TexturedGeometry::from_mesh(vertices, &faces, &uv, "tex.png");
        let Ok(mesh) = mesh else {
            return;
        };
        assert_eq!(mesh.points().len(), 4);
        assert_eq!(mesh.triangle_indices().len(), 12);
        assert_eq!(mesh.num_textures(), 1);
        // uv v-flip: [0.25, 0.75] → [0.25, 0.25].
        assert!((mesh.im_coords()[3][1] - 0.25).abs() < EPS);
        // C-4: real normals — unit length, pointing outward here.
        for (p, d) in mesh.points().iter().zip(mesh.d_normal_points()) {
            let n = sub(*d, *p);
            assert!(
                (space_ops::get_norm(n) - 1.0).abs() < 1e-9,
                "normal {n:?} not unit"
            );
            assert!(space_ops::dot(n, *p) > 0.0, "normal {n:?} not outward");
        }
    }

    #[test]
    fn textured_geometry_rejects_bad_meshes() {
        let vertices = vec![[0.0, 0.0, 0.0]];
        let uv = vec![[0.0, 0.0]];
        assert_eq!(
            TexturedGeometry::from_mesh(vertices.clone(), &[0, 1], &uv, "t"),
            Err(MeshError::IncompleteTriangle)
        );
        assert_eq!(
            TexturedGeometry::from_mesh(vertices.clone(), &[0, 0, 7], &uv, "t"),
            Err(MeshError::IndexOutOfRange {
                index: 7,
                vertices: 1
            })
        );
        assert_eq!(
            TexturedGeometry::from_mesh(vertices, &[0, 0, 0], &[], "t"),
            Err(MeshError::UvLengthMismatch { uv: 0, vertices: 1 })
        );
    }

    #[test]
    fn color_by_uv_function_paints_per_point() {
        let surface = ParametricSurface::new(|u, v| [u, v, 0.0])
            .resolution(2, 2)
            .build()
            .color_by_uv_function(|u, _v| Srgb {
                r: u,
                g: 0.0,
                b: 0.0,
            });
        // u-major: points 0,1 have u=0; points 2,3 have u=1.
        assert!(surface.rgba()[0][0].abs() < EPS);
        assert!((surface.rgba()[2][0] - 1.0).abs() < EPS);
    }
}
