//! Point clouds (§12.4, fm-2u6): `PMobject`/`PGroup` and the DotCloud
//! lineage — `DotCloud`, `TrueDot`, `GlowDots`, `GlowDot` (Appendix A
//! `types/point_cloud_mobject.py`, `types/dot_cloud.py`).
//!
//! A point cloud is a mobject whose geometry is an unordered bag of
//! points, each with its own RGBA and — for the DotCloud lineage — its own
//! world-space radius. There is no path, no fill, no stroke: every point
//! is a camera-facing radial sprite.
//!
//! # The kept glow (G0-2, decision rows "Glow falloff"/"Glow AA band", L7)
//!
//! The look-study ratified the Reference's radial profile exactly:
//!
//! * falloff `alpha *= (1 - r/R)^glow_factor`, a plain power law in the
//!   normalized radius reaching exactly zero at the rim — not a Gaussian
//!   (`true_dot/frag.glsl:26`; measured exponent 1.986 against the declared
//!   2.0, quadratic wins the model comparison by an order of magnitude);
//! * `glow_factor = 2.0` for `GlowDot`, `0.0` (hard disc) for `DotCloud`;
//! * silhouette band `anti_alias_width = 2.0` px of smoothstep
//!   (`dot_cloud.py:42`).
//!
//! [`glow_falloff`] is the continuous profile; [`glow_layers`] slices it
//! into the concentric annular layers a CPU rasterizer paints (each
//! annulus flat-filled at the falloff's mid-annulus value), and
//! [`rim_coverage`] is the smoothstep silhouette band the outermost layer
//! is modulated by.
//!
//! # Carriage deviations (documented, not silent)
//!
//! * **`glow_factor` rides a record lane.** The Reference keeps it in the
//!   shader uniform inventory; our [`fmn_mobject::Uniforms`] has no
//!   `glow_factor` slot (§8.4 predates point clouds), so the detached
//!   record schema appends one constant `glow_factor` lane to the
//!   Reference's `(point, radius, rgba)` dtype. A §8.4 inventory extension
//!   is the clean cutover; until then the lane is the only channel that
//!   survives `Stage::add`.
//! * **`ingest_submobjects` clears the children.** The Reference vstacks
//!   the family data and *keeps* the submobjects, which double-draws every
//!   ingested point; D5's dedup doctrine rules that wrong, so the children
//!   are consumed.
//! * `point_from_proportion` and the bounding box return `Option` rather
//!   than indexing into empty data.

use fmn_core::color::{Srgb, color_gradient};
use fmn_core::constants::{GREY_C, WHITE, YELLOW};
use fmn_core::types::Vec3;
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::uniforms::Uniforms;
use fmn_mobject::{Mobject, RenderPrimitive};

/// `dot_cloud.py`'s `DEFAULT_DOT_RADIUS` (0.05) — distinct from the
/// vectorized `Dot`'s 0.08 (`geometry.py`), which lives in [`crate::arc`].
pub const DEFAULT_DOT_CLOUD_RADIUS: f64 = 0.05;

/// `dot_cloud.py`'s `DEFAULT_GLOW_DOT_RADIUS`.
pub const DEFAULT_GLOW_DOT_RADIUS: f64 = 0.2;

/// `dot_cloud.py`'s `DEFAULT_GRID_HEIGHT` for [`DotCloud::to_grid`].
pub const DEFAULT_GRID_HEIGHT: f64 = 6.0;

/// `dot_cloud.py`'s `DEFAULT_BUFF_RATIO` for [`DotCloud::to_grid`].
pub const DEFAULT_BUFF_RATIO: f64 = 0.5;

/// The kept glow exponent for `GlowDot`: `(1 - r/R)²` (G0-2 L7; measured
/// 1.986 against the declared 2.0). `DotCloud`'s own default is 0.0.
pub const GLOW_DOT_FACTOR: f64 = 2.0;

/// The kept silhouette band for the whole DotCloud lineage: 2.0 px of
/// smoothstep (`dot_cloud.py:42`, G0-2 "Glow AA band" row).
pub const DOT_CLOUD_AA_WIDTH: f64 = 2.0;

/// `make_3d`'s Reference defaults `(reflectiveness, gloss, shadow)`
/// (`dot_cloud.py:142`).
pub const DOT_CLOUD_SHADING: Vec3 = [0.5, 0.1, 0.2];

// ------------------------------------------------------------------- glow

/// The kept radial glow falloff: `(1 - r)^glow_factor` on `r ∈ [0, 1)`,
/// exactly zero at and beyond the rim (`true_dot/frag.glsl:26`, G0-2 L7).
///
/// With `glow_factor = 0.0` (a plain `DotCloud`) the interior is hard:
/// `(1 - r)⁰ = 1` everywhere inside the disc.
#[must_use]
pub fn glow_falloff(r_fraction: f64, glow_factor: f64) -> f64 {
    if !(0.0..1.0).contains(&r_fraction) {
        return 0.0;
    }
    fmn_dmath::pow(1.0 - r_fraction, glow_factor)
}

/// The silhouette anti-alias band: the Reference's
/// `smoothstep(1.0, 1.0 - scaled_aaw, r)` (G0-2's kept profile
/// `t²(3 - 2t)`), expressed in fractions of the sprite radius. Coverage is
/// 1 inside `1 - aa_fraction`, falls smoothly to 0 at the rim.
#[must_use]
pub fn rim_coverage(r_fraction: f64, aa_fraction: f64) -> f64 {
    if aa_fraction <= 0.0 {
        return if r_fraction < 1.0 { 1.0 } else { 0.0 };
    }
    let t = ((1.0 - r_fraction) / aa_fraction).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// One annular layer of the CPU-side glow discretization: the ring
/// between the previous layer's [`outer_fraction`](Self::outer_fraction)
/// and this one, flat-filled at `alpha`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlowLayer {
    /// Outer radius of the annulus, as a fraction of the sprite radius.
    pub outer_fraction: f64,
    /// The falloff sampled at the annulus's mid-radius.
    pub alpha: f64,
}

/// Slice the kept falloff into `layers` concentric annuli of equal radial
/// width — the annular glow layers a CPU rasterizer paints, innermost
/// first. Each annulus takes the falloff's value at its mid-radius, the
/// midpoint rule that makes the discretization second-order in
/// `1/layers`. `layers = 0` yields no layers.
#[must_use]
pub fn glow_layers(glow_factor: f64, layers: usize) -> Vec<GlowLayer> {
    if layers == 0 {
        return Vec::new();
    }
    #[allow(clippy::cast_precision_loss)]
    let n = layers as f64;
    (0..layers)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let outer = (i as f64 + 1.0) / n;
            #[allow(clippy::cast_precision_loss)]
            let mid = (i as f64 + 0.5) / n;
            GlowLayer {
                outer_fraction: outer,
                alpha: glow_falloff(mid, glow_factor),
            }
        })
        .collect()
}

// --------------------------------------------------------------- PMobject

/// `PMobject`: a mobject that is a bare point cloud — the Reference's
/// `(point, rgba)` record pair with no path machinery
/// (`types/point_cloud_mobject.py`).
///
/// Construction and recoloring follow the tier's by-value builder style;
/// the additive operations (`add_points`, `ingest_submobjects`, …) consume
/// and return `self` the same way.
#[derive(Debug, Clone, PartialEq)]
pub struct PMobject {
    points: Vec<Vec3>,
    rgbas: Vec<[f64; 4]>,
    color: Srgb,
    opacity: f64,
    z_index: i32,
    children: Vec<PMobject>,
}

impl Default for PMobject {
    fn default() -> Self {
        Self::new()
    }
}

impl PMobject {
    /// An empty point cloud (Reference `Mobject` defaults: WHITE, opacity 1).
    #[must_use]
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            rgbas: Vec::new(),
            color: WHITE,
            opacity: 1.0,
            z_index: 0,
            children: Vec::new(),
        }
    }

    /// Replace the points (`set_points`). Existing colors are resampled
    /// onto the new count by linear interpolation — the Reference's
    /// `resize_points` (`resize_with_interpolation`); with no existing
    /// colors every point gets the current default `(color, opacity)`.
    #[must_use]
    pub fn with_points(mut self, points: impl IntoIterator<Item = Vec3>) -> Self {
        self.points = points.into_iter().collect();
        self.rgbas = resample_rgba(&self.rgbas, self.points.len(), (self.color, self.opacity));
        self
    }

    /// Append points, each colored with the current default
    /// (`add_points(points)` with the Reference's `color=None` path).
    #[must_use]
    pub fn add_points(mut self, points: impl IntoIterator<Item = Vec3>) -> Self {
        let rgba = [self.color.r, self.color.g, self.color.b, self.opacity];
        for point in points {
            self.points.push(point);
            self.rgbas.push(rgba);
        }
        self
    }

    /// Append points with explicit per-point RGBAs (`add_points` with
    /// `rgbas=`). When `rgbas` runs short the remaining points take the
    /// current default — the useful reading of the Reference's
    /// `rgbas=None` fallback (its own mismatched-length path is a numpy
    /// broadcast error).
    #[must_use]
    pub fn add_colored_points(
        mut self,
        points: impl IntoIterator<Item = Vec3>,
        rgbas: impl IntoIterator<Item = [f64; 4]>,
    ) -> Self {
        let fallback = [self.color.r, self.color.g, self.color.b, self.opacity];
        let mut rgbas = rgbas.into_iter();
        for point in points {
            self.points.push(point);
            self.rgbas.push(rgbas.next().unwrap_or(fallback));
        }
        self
    }

    /// Append one point (`add_point`); `rgba = None` takes the default.
    #[must_use]
    pub fn add_point(self, point: Vec3, rgba: Option<[f64; 4]>) -> Self {
        match rgba {
            Some(rgba) => self.add_colored_points([point], [rgba]),
            None => self.add_points([point]),
        }
    }

    /// Recolor every point and set the default for future points
    /// (Reference `set_color` on the data).
    #[must_use]
    pub fn colored(mut self, color: Srgb, opacity: f64) -> Self {
        self.color = color;
        self.opacity = opacity;
        let rgba = [color.r, color.g, color.b, opacity];
        self.rgbas.fill(rgba);
        self
    }

    /// `set_color_by_gradient`: resample the color ramp onto the points in
    /// order. The Reference's `color_to_rgba(color)` resets alpha to 1.0
    /// per point, and so does this. Zero colors recolors nothing; one
    /// color fills uniformly (the Reference's gradient needs ≥ 2).
    #[must_use]
    pub fn color_by_gradient(mut self, colors: &[Srgb]) -> Self {
        match colors {
            [] => {}
            [only] => {
                self.rgbas.fill([only.r, only.g, only.b, 1.0]);
            }
            _ => {
                for (slot, color) in self
                    .rgbas
                    .iter_mut()
                    .zip(color_gradient(colors, self.points.len()))
                {
                    *slot = [color.r, color.g, color.b, 1.0];
                }
            }
        }
        self
    }

    /// `match_colors`: resample another cloud's RGBA run onto this one's
    /// point count by linear interpolation.
    #[must_use]
    pub fn match_colors(mut self, other: &PMobject) -> Self {
        if !other.rgbas.is_empty() {
            self.rgbas = resample_rgba(&other.rgbas, self.points.len(), (self.color, self.opacity));
        }
        self
    }

    /// `filter_out`: drop the points the condition names, here and in
    /// every descendant (the Reference's `family_members_with_points`).
    #[must_use]
    pub fn filter_out(mut self, condition: impl Fn(Vec3) -> bool + Copy) -> Self {
        retain_pairs(&mut self.points, &mut self.rgbas, |p| !condition(p));
        self.children = self
            .children
            .into_iter()
            .map(|child| child.filter_out(condition))
            .collect();
        self
    }

    /// `sort_points` with the Reference's default key (`p[0]`).
    #[must_use]
    pub fn sort_points(self) -> Self {
        self.sort_points_by(|p| p[0])
    }

    /// `sort_points(function)`: stable-sort the points by the key, keeping
    /// each point's RGBA attached, here and in every descendant. Keys that
    /// compare unordered (NaN) keep their relative order.
    #[must_use]
    pub fn sort_points_by(mut self, key: impl Fn(Vec3) -> f64 + Copy) -> Self {
        let mut order: Vec<usize> = (0..self.points.len()).collect();
        order.sort_by(|&a, &b| {
            key(self.points[a])
                .partial_cmp(&key(self.points[b]))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let (points, rgbas) = (
            std::mem::take(&mut self.points),
            std::mem::take(&mut self.rgbas),
        );
        self.points = order.iter().map(|&i| points[i]).collect();
        self.rgbas = order.iter().map(|&i| rgbas[i]).collect();
        self.children = self
            .children
            .into_iter()
            .map(|child| child.sort_points_by(key))
            .collect();
        self
    }

    /// Apply a point transform here and in every descendant (the
    /// Reference's family-wide point transforms).
    #[must_use]
    pub fn map_points(mut self, f: impl Fn(Vec3) -> Vec3 + Copy) -> Self {
        for point in &mut self.points {
            *point = f(*point);
        }
        self.children = self
            .children
            .into_iter()
            .map(|child| child.map_points(f))
            .collect();
        self
    }

    /// Attach child clouds (`PGroup`'s composition; Reference `add`).
    #[must_use]
    pub fn with_children(mut self, children: impl IntoIterator<Item = PMobject>) -> Self {
        self.children.extend(children);
        self
    }

    /// `ingest_submobjects`: the family's data becomes this cloud's own,
    /// depth-first, self first — and the children are consumed. The
    /// Reference keeps the submobjects after the vstack, double-drawing
    /// every ingested point; D5's dedup doctrine rules that wrong.
    #[must_use]
    pub fn ingest_submobjects(mut self) -> Self {
        let children = std::mem::take(&mut self.children);
        for child in children {
            let child = child.ingest_submobjects();
            self.points.extend(child.points);
            self.rgbas.extend(child.rgbas);
        }
        self
    }

    /// `point_from_proportion`: the point at `int(alpha * (n - 1))`, with
    /// out-of-range proportions clamped (the Reference's negative
    /// proportions wrap from the end). `None` on an empty cloud, where the
    /// Reference would index error.
    #[must_use]
    pub fn point_from_proportion(&self, alpha: f64) -> Option<Vec3> {
        let n = self.points.len();
        if n == 0 {
            return None;
        }
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = (alpha.clamp(0.0, 1.0) * (n - 1) as f64) as usize;
        self.points.get(index.min(n - 1)).copied()
    }

    /// `pointwise_become_partial(pmobject, a, b)`: take the contiguous
    /// point run `int(a*n)..int(b*n)` of `other`. Proportions are clamped
    /// to `[0, 1]` — the Reference's negative slice indices wrap, which is
    /// never the intent of a *partial* become.
    #[must_use]
    pub fn pointwise_become_partial(mut self, other: &PMobject, a: f64, b: f64) -> Self {
        let n = other.points.len();
        let clamp_index = |alpha: f64| -> usize {
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let index = (alpha.clamp(0.0, 1.0) * n as f64) as usize;
            index.min(n)
        };
        let (lo, hi) = (clamp_index(a), clamp_index(b));
        if lo >= hi {
            self.points = Vec::new();
            self.rgbas = Vec::new();
        } else {
            self.points = other.points[lo..hi].to_vec();
            self.rgbas = other.rgbas[lo..hi].to_vec();
        }
        self
    }

    /// The scene-list sort key (§8.5; Reference `z_index`).
    #[must_use]
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// The points, in order.
    #[must_use]
    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    /// The per-point RGBAs, aligned with [`points`](Self::points).
    #[must_use]
    pub fn rgbas(&self) -> &[[f64; 4]] {
        &self.rgbas
    }

    /// Point count (`get_num_points`).
    #[must_use]
    pub fn num_points(&self) -> usize {
        self.points.len()
    }

    /// The child clouds.
    #[must_use]
    pub fn children(&self) -> &[PMobject] {
        &self.children
    }

    /// Axis-aligned bounds of the bare points; `None` when empty.
    #[must_use]
    pub fn extent(&self) -> Option<(Vec3, Vec3)> {
        extent_of(&self.points)
    }
}

/// `PGroup(*pmobs)`: a point cloud whose content is its child clouds
/// (`types/point_cloud_mobject.py:PGroup`). The Reference refuses
/// non-PMobject members at runtime; here that is the type system's job.
#[must_use]
pub fn p_group(pmobs: impl IntoIterator<Item = PMobject>) -> PMobject {
    PMobject::new().with_children(pmobs)
}

impl From<PMobject> for Mobject {
    fn from(pm: PMobject) -> Self {
        let PMobject {
            points,
            rgbas,
            z_index,
            children,
            ..
        } = pm;
        // The Reference's Mobject dtype: [('point', f32, 3), ('rgba', f32, 4)].
        let mut buffer = RecordBuffer::new(RecordSchema::mobject(), points.len())
            .expect("record sizing bounded by the point list");
        #[allow(clippy::cast_possible_truncation)]
        let flat_points: Vec<f32> = points
            .iter()
            .flat_map(|p| p.iter().map(|v| *v as f32))
            .collect();
        buffer.write_range("point", 0, &flat_points);
        #[allow(clippy::cast_possible_truncation)]
        let flat_rgba: Vec<f32> = rgbas
            .iter()
            .flat_map(|c| c.iter().map(|v| *v as f32))
            .collect();
        buffer.write_range("rgba", 0, &flat_rgba);

        let mut mob = Mobject::from_buffer(buffer).with_z_index(z_index);
        mob.submobjects
            .extend(children.into_iter().map(Mobject::from));
        mob
    }
}

// --------------------------------------------------------------- DotCloud

/// `DotCloud`: a point cloud of camera-facing radial sprites — every point
/// a world-space disc with its own radius and RGBA
/// (`types/dot_cloud.py`). `TrueDot`, `GlowDots`, and `GlowDot` are
/// constructor specializations ([`true_dot`], [`glow_dots`], [`glow_dot`]).
#[derive(Debug, Clone, PartialEq)]
pub struct DotCloud {
    cloud: PMobject,
    radii: Vec<f64>,
    /// The radius an empty cloud remembers — the Reference's `self.radius`
    /// attribute, which survives `set_points([])` via `_data_defaults`.
    configured_radius: f64,
    glow_factor: f64,
    anti_alias_width: f64,
    shading: Vec3,
    depth_test: bool,
}

impl DotCloud {
    /// `DotCloud(points, color=GREY_C, opacity=1, radius=0.05,
    /// glow_factor=0, anti_alias_width=2)` — the Reference's defaults,
    /// G0-2's kept constants.
    #[must_use]
    pub fn new(points: impl IntoIterator<Item = Vec3>) -> Self {
        let cloud = PMobject::new().colored(GREY_C, 1.0).with_points(points);
        let radii = vec![DEFAULT_DOT_CLOUD_RADIUS; cloud.num_points()];
        Self {
            cloud,
            radii,
            configured_radius: DEFAULT_DOT_CLOUD_RADIUS,
            glow_factor: 0.0,
            anti_alias_width: DOT_CLOUD_AA_WIDTH,
            shading: [0.0; 3],
            depth_test: false,
        }
    }

    /// Recolor every point (Reference `color=`).
    #[must_use]
    pub fn colored(mut self, color: Srgb, opacity: f64) -> Self {
        self.cloud = self.cloud.colored(color, opacity);
        self
    }

    /// One radius for every point (`set_radius`).
    #[must_use]
    pub fn with_radius(mut self, radius: f64) -> Self {
        self.configured_radius = radius;
        self.radii.fill(radius);
        self
    }

    /// Per-point radii, resampled onto the point count by linear
    /// interpolation (`set_radii`'s `resize_with_interpolation`).
    #[must_use]
    pub fn with_radii(mut self, radii: impl IntoIterator<Item = f64>) -> Self {
        let radii: Vec<f64> = radii.into_iter().collect();
        if let Some(max) = radii.iter().copied().max_by(f64::total_cmp) {
            self.configured_radius = max;
        }
        self.radii = resize_with_interpolation(&radii, self.cloud.num_points());
        self
    }

    /// Scale every radius (`scale_radii`).
    #[must_use]
    pub fn scale_radii(mut self, factor: f64) -> Self {
        for radius in &mut self.radii {
            *radius *= factor;
        }
        self.configured_radius *= factor;
        self
    }

    /// The glow exponent (`set_glow_factor`). 0.0 is a hard disc; 2.0 is
    /// the kept `(1 - r/R)²` glow.
    #[must_use]
    pub fn with_glow_factor(mut self, glow_factor: f64) -> Self {
        self.glow_factor = glow_factor;
        self
    }

    /// The silhouette band in output pixels (`anti_alias_width`).
    #[must_use]
    pub fn with_anti_alias_width(mut self, anti_alias_width: f64) -> Self {
        self.anti_alias_width = anti_alias_width;
        self
    }

    /// `make_3d(reflectiveness=0.5, gloss=0.1, shadow=0.2)`: shade the
    /// sprites as spheres and depth-test them (`dot_cloud.py:142`).
    #[must_use]
    pub fn make_3d(mut self) -> Self {
        self.shading = DOT_CLOUD_SHADING;
        self.depth_test = true;
        self
    }

    /// `to_grid(n_rows, n_cols, n_layers, …)`: lay the points out on a
    /// regular 3D grid, spacing each axis so adjacent grid lines sit
    /// `2·radius·(1 + buff_ratio)` apart, then set the total height and
    /// center (`dot_cloud.py:59`). A `None` `height` skips the rescale;
    /// the Reference's `buff_ratio` shorthand sets all three axes.
    ///
    /// A one-line axis (`n == 1`) is left at coordinate 0 rather than
    /// scaled by `0/0`; the Reference's `rescale_to_fit` divides by the
    /// old extent, which is zero there.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn to_grid(
        mut self,
        n_rows: usize,
        n_cols: usize,
        n_layers: usize,
        buff_ratio: Option<f64>,
        h_buff_ratio: f64,
        v_buff_ratio: f64,
        d_buff_ratio: f64,
        height: Option<f64>,
    ) -> Self {
        let n_points = n_rows * n_cols * n_layers;
        if n_points == 0 {
            self.cloud = self.cloud.with_points([]);
            self.radii = Vec::new();
            return self;
        }
        // The grid spacing reads the configured radius before the points
        // (and their per-point radii) are replaced — the Reference's
        // `radius = self.get_radius()` ahead of `set_points`.
        let radius = self.radius();
        let mut points = Vec::with_capacity(n_points);
        for i in 0..n_points {
            #[allow(clippy::cast_precision_loss)]
            let point = [
                (i % n_cols) as f64,
                ((i / n_cols) % n_rows) as f64,
                (i / (n_rows * n_cols)) as f64,
            ];
            points.push(point);
        }
        self.cloud = self.cloud.with_points(points);
        self.radii = vec![radius; n_points];

        let (h, v, d) = match buff_ratio {
            Some(br) => (br, br, br),
            None => (h_buff_ratio, v_buff_ratio, d_buff_ratio),
        };
        let ns = [n_cols, n_rows, n_layers];
        let brs = [h, v, d];
        // Per-axis rescale reads the bare point extent — the Reference's
        // `set_radius(0)` dance, which exists only to make its bounding box
        // radius-free for the same computation.
        if let Some((min, max)) = self.cloud.extent() {
            let center = mid(min, max);
            for dim in 0..3 {
                let extent = max[dim] - min[dim];
                #[allow(clippy::cast_precision_loss)]
                let target = 2.0 * radius * (1.0 + brs[dim]) * (ns[dim] - 1) as f64;
                if extent > 0.0 && ns[dim] > 1 {
                    let scale = target / extent;
                    self.cloud = self
                        .cloud
                        .map_points(|p| scale_about_dim(p, dim, scale, center[dim]));
                }
            }
        }

        if let Some(height) = height {
            self = self.with_height(height);
        }
        self.centered()
    }

    /// Uniform scale about the origin (`scale(scale_factor)`, which scales
    /// the radii too — `scale_radii=True` is the Reference's default).
    #[must_use]
    pub fn scaled(mut self, factor: f64) -> Self {
        self.cloud = self
            .cloud
            .map_points(|p| [p[0] * factor, p[1] * factor, p[2] * factor]);
        self.scale_radii(factor)
    }

    /// Uniform scale about the bounding-box center so the box (radius
    /// included) stands `height` tall (Reference `set_height`).
    #[must_use]
    pub fn with_height(mut self, height: f64) -> Self {
        if let Some((min, max)) = self.bounding_box() {
            let extent = max[1] - min[1];
            if extent > 0.0 {
                let scale = height / extent;
                let center = mid(min, max);
                self.cloud = self.cloud.map_points(|p| scale_about(p, scale, center));
                self = self.scale_radii(scale);
            }
        }
        self
    }

    /// Shift so the bounding box (radius included) centers on the origin
    /// (Reference `center`).
    #[must_use]
    pub fn centered(mut self) -> Self {
        if let Some((min, max)) = self.bounding_box() {
            let center = mid(min, max);
            self.cloud = self
                .cloud
                .map_points(|p| [p[0] - center[0], p[1] - center[1], p[2] - center[2]]);
        }
        self
    }

    /// The points, in order.
    #[must_use]
    pub fn points(&self) -> &[Vec3] {
        self.cloud.points()
    }

    /// The per-point RGBAs.
    #[must_use]
    pub fn rgbas(&self) -> &[[f64; 4]] {
        self.cloud.rgbas()
    }

    /// The per-point radii.
    #[must_use]
    pub fn radii(&self) -> &[f64] {
        &self.radii
    }

    /// `get_radius`: the largest radius — what the bounding box grows by.
    /// An empty cloud answers its configured radius, like the Reference's
    /// `_data_defaults` fallback.
    #[must_use]
    pub fn radius(&self) -> f64 {
        self.radii
            .iter()
            .copied()
            .fold(self.configured_radius, f64::max)
    }

    /// `get_glow_factor`.
    #[must_use]
    pub fn get_glow_factor(&self) -> f64 {
        self.glow_factor
    }

    /// The silhouette band in output pixels.
    #[must_use]
    pub fn get_anti_alias_width(&self) -> f64 {
        self.anti_alias_width
    }

    /// Point count.
    #[must_use]
    pub fn num_points(&self) -> usize {
        self.cloud.num_points()
    }

    /// `compute_bounding_box`: the point extent grown by the largest
    /// radius on every side; `None` when empty.
    #[must_use]
    pub fn bounding_box(&self) -> Option<(Vec3, Vec3)> {
        let (min, max) = self.cloud.extent()?;
        let radius = self.radius();
        Some((add(min, -radius), add(max, radius)))
    }
}

/// `TrueDot(center)`: a one-point DotCloud (`dot_cloud.py:151`).
#[must_use]
pub fn true_dot(center: Vec3) -> DotCloud {
    DotCloud::new([center])
}

/// `GlowDots(points, color=YELLOW, radius=0.2, glow_factor=2.0)` — the
/// Reference's density field: a cloud of radial glows (`dot_cloud.py:156`).
#[must_use]
pub fn glow_dots(points: impl IntoIterator<Item = Vec3>) -> DotCloud {
    DotCloud::new(points)
        .colored(YELLOW, 1.0)
        .with_radius(DEFAULT_GLOW_DOT_RADIUS)
        .with_glow_factor(GLOW_DOT_FACTOR)
}

/// `GlowDot(center)`: a one-point GlowDots (`dot_cloud.py:174`).
#[must_use]
pub fn glow_dot(center: Vec3) -> DotCloud {
    glow_dots([center])
}

impl From<DotCloud> for Mobject {
    fn from(cloud: DotCloud) -> Self {
        let DotCloud {
            cloud,
            radii,
            glow_factor,
            anti_alias_width,
            shading,
            depth_test,
            ..
        } = cloud;
        let PMobject {
            points,
            rgbas,
            z_index,
            children,
            ..
        } = cloud;
        // The Reference's DotCloud dtype — [('point', f32, 3), ('radius',
        // f32, 1), ('rgba', f32, 4)] — plus one constant `glow_factor`
        // lane carrying the shader uniform the §8.4 inventory has no slot
        // for yet (see the module docs).
        let schema = RecordSchema::new(
            &[("point", 3), ("radius", 1), ("rgba", 4), ("glow_factor", 1)],
            &["point"],
            &["point"],
        )
        .expect("the point-cloud record schema is nine lanes");
        let mut buffer = RecordBuffer::new(schema, points.len())
            .expect("record sizing bounded by the point list");
        #[allow(clippy::cast_possible_truncation)]
        let flat_points: Vec<f32> = points
            .iter()
            .flat_map(|p| p.iter().map(|v| *v as f32))
            .collect();
        buffer.write_range("point", 0, &flat_points);
        #[allow(clippy::cast_possible_truncation)]
        let flat_radii: Vec<f32> = radii.iter().map(|r| *r as f32).collect();
        buffer.write_range("radius", 0, &flat_radii);
        #[allow(clippy::cast_possible_truncation)]
        let flat_rgba: Vec<f32> = rgbas
            .iter()
            .flat_map(|c| c.iter().map(|v| *v as f32))
            .collect();
        buffer.write_range("rgba", 0, &flat_rgba);
        #[allow(clippy::cast_possible_truncation)]
        let flat_glow: Vec<f32> = vec![glow_factor as f32; points.len()];
        buffer.write_range("glow_factor", 0, &flat_glow);

        let uniforms = Uniforms {
            anti_alias_width,
            shading,
            depth_test,
            ..Uniforms::default()
        };
        let mut mob = Mobject::from_buffer(buffer)
            .with_uniforms(uniforms)
            .with_render_primitive(RenderPrimitive::DotCloud)
            .with_z_index(z_index);
        mob.submobjects
            .extend(children.into_iter().map(Mobject::from));
        mob
    }
}

// ---------------------------------------------------------------- helpers

/// Resample an RGBA run onto `length` positions by per-lane linear
/// interpolation; an empty source fills from the default.
fn resample_rgba(source: &[[f64; 4]], length: usize, default: (Srgb, f64)) -> Vec<[f64; 4]> {
    if source.is_empty() {
        let (color, opacity) = default;
        return vec![[color.r, color.g, color.b, opacity]; length];
    }
    let lanes: Vec<Vec<f64>> = (0..4)
        .map(|lane| {
            resize_with_interpolation(&source.iter().map(|c| c[lane]).collect::<Vec<_>>(), length)
        })
        .collect();
    (0..length)
        .map(|i| [lanes[0][i], lanes[1][i], lanes[2][i], lanes[3][i]])
        .collect()
}

/// The Reference's `resize_with_interpolation` for scalar lanes: resample
/// `values` onto `length` evenly spaced positions by linear interpolation.
/// A single value fills; an empty list yields an empty resample.
fn resize_with_interpolation(values: &[f64], length: usize) -> Vec<f64> {
    if values.is_empty() || length == 0 {
        return Vec::new();
    }
    if values.len() == 1 {
        return vec![values[0]; length];
    }
    (0..length)
        .map(|i| {
            if length == 1 {
                return values[0];
            }
            #[allow(clippy::cast_precision_loss)]
            let alpha = i as f64 / (length - 1) as f64;
            #[allow(clippy::cast_precision_loss)]
            let scaled = alpha * (values.len() - 1) as f64;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let lo = (scaled.floor() as usize).min(values.len() - 1);
            let hi = (lo + 1).min(values.len() - 1);
            #[allow(clippy::cast_precision_loss)]
            let t = scaled - lo as f64;
            values[lo] + (values[hi] - values[lo]) * t
        })
        .collect()
}

/// Drop the `(point, rgba)` pairs the predicate rejects, keeping order.
fn retain_pairs(points: &mut Vec<Vec3>, rgbas: &mut Vec<[f64; 4]>, keep: impl Fn(Vec3) -> bool) {
    let mut write = 0;
    for read in 0..points.len() {
        if keep(points[read]) {
            points[write] = points[read];
            rgbas[write] = rgbas[read];
            write += 1;
        }
    }
    points.truncate(write);
    rgbas.truncate(write);
}

/// Axis-aligned extent of a point run; `None` when empty.
fn extent_of(points: &[Vec3]) -> Option<(Vec3, Vec3)> {
    let mut iter = points.iter();
    let first = *iter.next()?;
    let mut min = first;
    let mut max = first;
    for p in iter {
        for dim in 0..3 {
            min[dim] = min[dim].min(p[dim]);
            max[dim] = max[dim].max(p[dim]);
        }
    }
    Some((min, max))
}

/// Componentwise midpoint.
fn mid(a: Vec3, b: Vec3) -> Vec3 {
    [
        (a[0] + b[0]) / 2.0,
        (a[1] + b[1]) / 2.0,
        (a[2] + b[2]) / 2.0,
    ]
}

/// Add the same scalar to every component.
fn add(p: Vec3, delta: f64) -> Vec3 {
    [p[0] + delta, p[1] + delta, p[2] + delta]
}

/// Uniform scale about a center point.
fn scale_about(p: Vec3, factor: f64, about: Vec3) -> Vec3 {
    [
        about[0] + (p[0] - about[0]) * factor,
        about[1] + (p[1] - about[1]) * factor,
        about[2] + (p[2] - about[2]) * factor,
    ]
}

/// Scale one axis about a center coordinate (Reference
/// `rescale_to_fit(…, dim, stretch=True)`).
fn scale_about_dim(p: Vec3, dim: usize, factor: f64, about: f64) -> Vec3 {
    let mut out = p;
    out[dim] = about + (p[dim] - about) * factor;
    out
}

// ------------------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_core::constants::{BLUE, ORIGIN, RED};

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn close_vec(a: Vec3, b: Vec3) -> bool {
        (0..3).all(|k| (a[k] - b[k]).abs() < 1e-9)
    }

    // ---- G0-2 glow calibration ------------------------------------------

    #[test]
    fn glow_constants_are_the_g02_calibration() {
        // The kept rows of G0-2's decision table.
        assert_eq!(GLOW_DOT_FACTOR, 2.0, "G0-2 L7: (1-r/R)^2, measured 1.986");
        assert_eq!(DOT_CLOUD_AA_WIDTH, 2.0, "dot_cloud.py:42");
        assert_eq!(DEFAULT_DOT_CLOUD_RADIUS, 0.05);
        assert_eq!(DEFAULT_GLOW_DOT_RADIUS, 0.2);
        // The measured exponent sits within its fit tolerance of the kept 2.0.
        assert!((GLOW_DOT_FACTOR - 1.986).abs() < 0.02);
    }

    #[test]
    fn glow_falloff_is_the_kept_power_law() {
        // Sampled against the closed form at eighths of the radius.
        for i in 0..8 {
            #[allow(clippy::cast_precision_loss)]
            let r = i as f64 / 8.0;
            assert!(close(glow_falloff(r, 2.0), (1.0 - r) * (1.0 - r)));
        }
        // Zero glow factor: hard disc (DotCloud).
        assert!(close(glow_falloff(0.25, 0.0), 1.0));
        assert!(close(glow_falloff(0.999, 0.0), 1.0));
        // Exactly zero at and beyond the rim — a power law, not a Gaussian.
        assert_eq!(glow_falloff(1.0, 2.0), 0.0);
        assert_eq!(glow_falloff(1.5, 2.0), 0.0);
        assert!(close(glow_falloff(0.0, 2.0), 1.0));
    }

    #[test]
    fn rim_coverage_is_the_kept_smoothstep() {
        // smoothstep(1, 1 - aa, r) == t^2(3 - 2t) with t = (1 - r)/aa.
        let aa = 0.1;
        assert!(close(rim_coverage(0.85, aa), 1.0));
        assert!(close(rim_coverage(1.0, aa), 0.0));
        // Mid-band: t = 0.5 → 0.25 * 2 = 0.5.
        assert!(close(rim_coverage(0.95, aa), 0.5));
        // t = 0.25 → 0.0625 * 2.5 = 0.15625.
        assert!(close(rim_coverage(0.975, aa), 0.15625));
        // A zero-width band is a hard silhouette.
        assert_eq!(rim_coverage(0.999, 0.0), 1.0);
        assert_eq!(rim_coverage(1.0, 0.0), 0.0);
    }

    #[test]
    fn glow_layers_trace_the_calibration() {
        let layers = glow_layers(GLOW_DOT_FACTOR, 8);
        assert_eq!(layers.len(), 8);
        for (i, layer) in layers.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let i = i as f64;
            // Concentric equal-width annuli, innermost first.
            assert!(close(layer.outer_fraction, (i + 1.0) / 8.0));
            // Mid-annulus opacity is exactly the kept falloff there.
            let mid = (i + 0.5) / 8.0;
            assert!(close(layer.alpha, (1.0 - mid) * (1.0 - mid)));
        }
        // Monotone decreasing — the glow never brightens outward.
        assert!(layers.windows(2).all(|w| w[0].alpha > w[1].alpha));
        // The outermost annulus approaches zero as the rim closes in.
        let fine = glow_layers(GLOW_DOT_FACTOR, 1000);
        assert!(fine[999].alpha < 3e-7);
        // Midpoint sampling is second-order: the layer average tracks the
        // exact integral 1/3 of (1-r)^2 to O(1/n^2).
        let coarse = glow_layers(GLOW_DOT_FACTOR, 64);
        #[allow(clippy::cast_precision_loss)]
        let mean = coarse.iter().map(|l| l.alpha).sum::<f64>() / 64.0;
        assert!((mean - 1.0 / 3.0).abs() < 1e-4);
        assert!(glow_layers(GLOW_DOT_FACTOR, 0).is_empty());
    }

    // ---- DotCloud / TrueDot / GlowDots / GlowDot -------------------------

    #[test]
    fn dot_cloud_constructor_fixture() {
        let cloud = DotCloud::new([[1.0, 2.0, 3.0], [-1.0, 0.0, 0.5]]);
        assert_eq!(cloud.num_points(), 2);
        // GREY_C (#888888) at full opacity, radius 0.05, hard disc, 2px band.
        for rgba in cloud.rgbas() {
            assert!(close(rgba[0], 0x88 as f64 / 255.0));
            assert!(close(rgba[1], 0x88 as f64 / 255.0));
            assert!(close(rgba[2], 0x88 as f64 / 255.0));
            assert!(close(rgba[3], 1.0));
        }
        assert!(cloud.radii().iter().all(|&r| close(r, 0.05)));
        assert_eq!(cloud.get_glow_factor(), 0.0);
        assert_eq!(cloud.get_anti_alias_width(), 2.0);
    }

    #[test]
    fn true_dot_is_a_one_point_cloud() {
        let dot = true_dot([1.0, 2.0, 0.0]);
        assert_eq!(dot.num_points(), 1);
        assert!(close_vec(dot.points()[0], [1.0, 2.0, 0.0]));
        assert_eq!(dot.get_glow_factor(), 0.0, "TrueDot is a hard disc");
    }

    #[test]
    fn glow_dot_is_yellow_and_glowing() {
        let dot = glow_dot(ORIGIN);
        assert_eq!(dot.num_points(), 1);
        let rgba = dot.rgbas()[0];
        assert!(close(rgba[0], 1.0) && close(rgba[1], 1.0) && close(rgba[2], 0.0));
        assert!(close(dot.radius(), DEFAULT_GLOW_DOT_RADIUS));
        assert_eq!(dot.get_glow_factor(), GLOW_DOT_FACTOR);
        let dots = glow_dots([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]);
        assert_eq!(dots.num_points(), 2);
        assert_eq!(dots.get_glow_factor(), 2.0);
    }

    #[test]
    fn bounding_box_grows_by_the_radius() {
        let cloud = DotCloud::new([[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]).with_radius(0.25);
        let (min, max) = cloud.bounding_box().expect("nonempty");
        assert!(close_vec(min, [-0.25, -0.25, -0.25]));
        assert!(close_vec(max, [2.25, 0.25, 0.25]));
        assert!(DotCloud::new([]).bounding_box().is_none());
    }

    #[test]
    fn to_grid_lays_out_the_reference_grid() {
        // 2x2x1 grid, radius 0.1, buff 0.5: spacing 2*0.1*1.5 = 0.3.
        let cloud =
            DotCloud::new([])
                .with_radius(0.1)
                .to_grid(2, 2, 1, Some(0.5), 1.0, 1.0, 1.0, None);
        assert_eq!(cloud.num_points(), 4);
        let mut xs: Vec<f64> = cloud.points().iter().map(|p| p[0]).collect();
        xs.sort_by(f64::total_cmp);
        xs.dedup();
        assert_eq!(xs.len(), 2, "two grid lines on x");
        assert!(close(xs[1] - xs[0], 0.3), "spacing is 2r(1+buff)");
        // Centered on the origin.
        let (min, max) = cloud.bounding_box().expect("nonempty");
        assert!(close_vec(mid(min, max), ORIGIN));
        // With a height, the box (radius included) stands exactly that tall.
        let sized = DotCloud::new([]).with_radius(0.1).to_grid(
            2,
            2,
            1,
            Some(0.5),
            1.0,
            1.0,
            1.0,
            Some(2.0),
        );
        let (min, max) = sized.bounding_box().expect("nonempty");
        assert!(close(max[1] - min[1], 2.0));
    }

    #[test]
    fn make_3d_sets_shading_and_depth_test() {
        let mob = Mobject::from(true_dot(ORIGIN).make_3d());
        assert_eq!(mob.uniforms.shading, DOT_CLOUD_SHADING);
        assert!(mob.uniforms.depth_test);
    }

    #[test]
    fn dot_cloud_records_carry_the_reference_dtype_plus_glow() {
        let mob = Mobject::from(
            DotCloud::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]])
                .colored(RED, 0.5)
                .with_radii([0.1, 0.3])
                .with_glow_factor(2.0),
        );
        assert_eq!(mob.buffer.len(), 2);
        let names: Vec<&str> = mob
            .buffer
            .schema()
            .fields()
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        // Reference order, then the constant glow lane.
        assert_eq!(names, ["point", "radius", "rgba", "glow_factor"]);
        assert_eq!(
            mob.buffer.read(1, "point").expect("field"),
            vec![4.0, 5.0, 6.0]
        );
        assert_eq!(mob.buffer.read(0, "radius").expect("field"), vec![0.1]);
        assert_eq!(mob.buffer.read(1, "radius").expect("field"), vec![0.3]);
        let rgba = mob.buffer.read(0, "rgba").expect("field");
        assert!((f64::from(rgba[0]) - RED.r).abs() < 1e-3);
        assert!((f64::from(rgba[1]) - RED.g).abs() < 1e-3);
        assert!((rgba[3] - 0.5).abs() < 1e-6);
        assert_eq!(mob.buffer.read(1, "glow_factor").expect("field"), vec![2.0]);
        // The DotCloud AA band lands in the uniform inventory.
        assert_eq!(mob.uniforms.anti_alias_width, DOT_CLOUD_AA_WIDTH);
        assert_eq!(mob.render_primitive, RenderPrimitive::DotCloud);
    }

    #[test]
    fn scaling_scales_radii() {
        let cloud = DotCloud::new([[1.0, 0.0, 0.0]])
            .with_radius(0.2)
            .scaled(2.0);
        assert!(close(cloud.radius(), 0.4));
        assert!(close_vec(cloud.points()[0], [2.0, 0.0, 0.0]));
    }

    // ---- PMobject / PGroup -----------------------------------------------

    #[test]
    fn pmobject_add_and_color() {
        let cloud = PMobject::new()
            .colored(BLUE, 0.5)
            .add_points([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]])
            .add_point([2.0, 0.0, 0.0], Some([1.0, 0.0, 0.0, 1.0]));
        assert_eq!(cloud.num_points(), 3);
        assert!(close(cloud.rgbas()[0][0], BLUE.r));
        assert!(close(cloud.rgbas()[0][1], BLUE.g));
        assert!(close(cloud.rgbas()[0][2], BLUE.b));
        assert!(close(cloud.rgbas()[0][3], 0.5));
        assert!(close(cloud.rgbas()[2][0], 1.0)); // explicit rgba wins
        assert!(close(cloud.rgbas()[2][3], 1.0));
    }

    #[test]
    fn gradient_and_match_colors_resample() {
        let cloud = PMobject::new()
            .add_points([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]])
            .color_by_gradient(&[RED, BLUE]);
        // Endpoints are the ramp ends; the midpoint blends (G0-2's sqrt ramp).
        assert!(close(cloud.rgbas()[0][0], RED.r) && close(cloud.rgbas()[0][2], RED.b));
        assert!(close(cloud.rgbas()[2][0], BLUE.r) && close(cloud.rgbas()[2][2], BLUE.b));
        assert!(cloud.rgbas()[1][0] > RED.r.min(BLUE.r) && cloud.rgbas()[1][2] > RED.b.min(BLUE.b));

        let short = PMobject::new().add_colored_points([[9.0, 9.0, 9.0]], [[0.25, 0.5, 0.75, 1.0]]);
        let matched = cloud.match_colors(&short);
        // A one-entry source fills every point.
        assert!(
            matched
                .rgbas()
                .iter()
                .all(|c| close(c[0], 0.25) && close(c[1], 0.5))
        );
    }

    #[test]
    fn filter_sort_partial_and_proportion() {
        let cloud = PMobject::new().add_points([
            [3.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
        ]);
        let sorted = cloud.clone().sort_points();
        assert!(close_vec(sorted.points()[0], [1.0, 0.0, 0.0]));
        assert!(close_vec(sorted.points()[3], [4.0, 0.0, 0.0]));

        let filtered = cloud.clone().filter_out(|p| p[0] > 2.5);
        assert_eq!(filtered.num_points(), 2);

        assert!(close_vec(
            cloud.point_from_proportion(0.0).expect("nonempty"),
            [3.0, 0.0, 0.0]
        ));
        assert!(
            cloud
                .clone()
                .filter_out(|_| true)
                .point_from_proportion(0.5)
                .is_none()
        );

        let partial = PMobject::new().pointwise_become_partial(&cloud, 0.25, 0.75);
        assert_eq!(partial.num_points(), 2);
        assert!(close_vec(partial.points()[0], [1.0, 0.0, 0.0]));
        assert!(close_vec(partial.points()[1], [2.0, 0.0, 0.0]));
    }

    #[test]
    fn p_group_and_ingest() {
        let a = PMobject::new().add_points([[0.0, 0.0, 0.0]]);
        let b = PMobject::new().add_points([[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        let group = p_group([a, b]);
        assert_eq!(group.children().len(), 2);
        let flat = group.ingest_submobjects();
        assert_eq!(flat.num_points(), 3);
        assert!(flat.children().is_empty(), "ingested children are consumed");

        // Conversion nests children as detached submobjects.
        let mob = Mobject::from(p_group([
            PMobject::new().add_points([[0.0, 0.0, 0.0]]),
            PMobject::new().add_points([[1.0, 1.0, 0.0]]),
        ]));
        assert_eq!(mob.submobjects.len(), 2);
        assert_eq!(mob.buffer.len(), 0, "a bare PGroup holds no points itself");
        let names: Vec<&str> = mob
            .buffer
            .schema()
            .fields()
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, ["point", "rgba"], "PMobject's Reference dtype");
    }
}
