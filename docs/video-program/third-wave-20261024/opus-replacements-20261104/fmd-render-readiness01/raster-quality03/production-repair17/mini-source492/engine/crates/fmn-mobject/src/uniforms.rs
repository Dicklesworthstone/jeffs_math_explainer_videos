//! The per-object uniform inventory (§1.1, §8.4): typed per-mobject render
//! state that scene code reads and writes directly (`mobject.uniforms[...]` is
//! API surface) and that Lumen's `StyleTable` synchronizes from.
//!
//! The Reference keeps these in an untyped `dict`; we make the complete
//! inventory explicit and typed so the whole set is locked and no key is a
//! stringly-typed surprise. The base (`Mobject`) uniforms are `is_fixed_in_frame`
//! (a float *mix*, per the kept camera model — not a bool), the `shading` triple
//! (reflectiveness, gloss, shadow), and four clip-plane slots; VMobjects add the
//! anti-alias width, joint type, and the `flat_stroke` / `scale_stroke_with_zoom`
//! flags. `stroke_behind` and `depth_test` are per-object flags the Reference
//! stores alongside the dict.
//!
//! # Appendix C rulings owned here
//!
//! - **C-2 (BN-07).** The Reference's `get_scale_stroke_with_zoom()` reads the
//!   *wrong* uniform (`flat_stroke`). Ours reads
//!   [`Uniforms::scale_stroke_with_zoom`] — the correct one. See
//!   [`Uniforms::get_scale_stroke_with_zoom`].
//! - **C-7.** `use_winding_fill` is a documented no-op in the shipped Reference
//!   (the ear-clip fill path is dead). We accept the API for source
//!   compatibility as an explicit no-op: the flag is stored and readable but
//!   affects no output — our analytic winding fill never needed it. See
//!   [`Uniforms::use_winding_fill`].

use fmn_core::types::Vec3;

/// Stroke joint style, mapped to the Reference's `joint_type_map` float codes
/// (`no_joint → 0, auto → 1, bevel → 2, miter → 3`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum JointType {
    /// No join (`0.0`).
    NoJoint,
    /// Automatic (`1.0`) — the default.
    #[default]
    Auto,
    /// Beveled (`2.0`).
    Bevel,
    /// Mitered (`3.0`).
    Miter,
}

impl JointType {
    /// The float code Lumen consumes (the Reference's `joint_type_map`).
    #[must_use]
    pub fn to_code(self) -> f64 {
        match self {
            Self::NoJoint => 0.0,
            Self::Auto => 1.0,
            Self::Bevel => 2.0,
            Self::Miter => 3.0,
        }
    }

    /// Inverse of [`to_code`](Self::to_code); unknown codes fall back to
    /// [`JointType::Auto`], matching the Reference's default.
    #[must_use]
    pub fn from_code(code: f64) -> Self {
        match code as i64 {
            0 => Self::NoJoint,
            2 => Self::Bevel,
            3 => Self::Miter,
            _ => Self::Auto,
        }
    }
}

/// The complete typed uniform inventory for one mobject.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Uniforms {
    /// A float *mix* in `[0, 1]` (kept camera model), not a bool: `0.0` tracks
    /// the camera, `1.0` is pinned to the frame. Reference default `0.0`.
    pub is_fixed_in_frame: f64,
    /// `[reflectiveness, gloss, shadow]`. Reference default `[0, 0, 0]`.
    pub shading: Vec3,
    /// Four clip-plane slots, each a `[a, b, c, d]` plane. Default all-zero.
    pub clip_planes: [[f64; 4]; 4],
    /// Per-kind anti-alias width in pixels. Reference default `1.5`.
    pub anti_alias_width: f64,
    /// Stroke joint style.
    pub joint_type: JointType,
    /// Whether the stroke is drawn flat (in the xy-plane) vs. billboarded.
    pub flat_stroke: bool,
    /// Whether stroke width scales with camera zoom.
    pub scale_stroke_with_zoom: bool,
    /// Whether the stroke is drawn behind the fill.
    pub stroke_behind: bool,
    /// Whether depth testing is enabled for this object.
    pub depth_test: bool,
    /// C-7: accepted for source compatibility, affects no output bits.
    pub use_winding_fill: bool,
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            is_fixed_in_frame: 0.0,
            shading: [0.0, 0.0, 0.0],
            clip_planes: [[0.0; 4]; 4],
            anti_alias_width: 1.5,
            joint_type: JointType::Auto,
            flat_stroke: false,
            scale_stroke_with_zoom: false,
            stroke_behind: false,
            depth_test: false,
            use_winding_fill: false,
        }
    }
}

impl Uniforms {
    /// The `[reflectiveness, gloss, shadow]` triple (Reference `get_shading`).
    #[must_use]
    pub fn shading(&self) -> Vec3 {
        self.shading
    }

    /// C-2 / BN-07: read the **correct** uniform. Where the Reference's
    /// `get_scale_stroke_with_zoom()` returns `flat_stroke`, ours returns
    /// [`Uniforms::scale_stroke_with_zoom`].
    #[must_use]
    pub fn get_scale_stroke_with_zoom(&self) -> bool {
        self.scale_stroke_with_zoom
    }

    /// Reads `flat_stroke` (kept distinct from the C-2 accessor above so the
    /// ruling is visible in the API, not just in a comment).
    #[must_use]
    pub fn get_flat_stroke(&self) -> bool {
        self.flat_stroke
    }

    /// C-7: `use_winding_fill` is an accepted no-op. Setting it records the
    /// flag (so a round-trip through the uniform surface is faithful) but
    /// changes no rendered bits — the analytic fill ignores it.
    pub fn use_winding_fill(&mut self, value: bool) -> &mut Self {
        self.use_winding_fill = value;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_defaults() {
        let u = Uniforms::default();
        assert_eq!(u.is_fixed_in_frame, 0.0);
        assert_eq!(u.shading(), [0.0, 0.0, 0.0]);
        assert_eq!(u.clip_planes, [[0.0; 4]; 4]);
        assert_eq!(u.anti_alias_width, 1.5);
        assert_eq!(u.joint_type, JointType::Auto);
        assert!(!u.flat_stroke);
        assert!(!u.scale_stroke_with_zoom);
    }

    #[test]
    fn joint_type_codes_round_trip() {
        for jt in [
            JointType::NoJoint,
            JointType::Auto,
            JointType::Bevel,
            JointType::Miter,
        ] {
            assert_eq!(JointType::from_code(jt.to_code()), jt);
        }
        assert_eq!(JointType::Auto.to_code(), 1.0);
        // Unknown codes fall back to Auto (Reference default behavior).
        assert_eq!(JointType::from_code(99.0), JointType::Auto);
    }

    #[test]
    fn c2_reads_the_correct_uniform() {
        // BN-07: scale_stroke_with_zoom and flat_stroke are independent, and
        // get_scale_stroke_with_zoom reflects the former — not the latter, as
        // the Reference bug would.
        let u = Uniforms {
            flat_stroke: true,
            scale_stroke_with_zoom: false,
            ..Default::default()
        };
        assert!(!u.get_scale_stroke_with_zoom()); // Reference would wrongly say true
        assert!(u.get_flat_stroke());

        let u = Uniforms {
            flat_stroke: false,
            scale_stroke_with_zoom: true,
            ..Default::default()
        };
        assert!(u.get_scale_stroke_with_zoom());
        assert!(!u.get_flat_stroke());
    }

    #[test]
    fn c7_use_winding_fill_is_an_accepted_no_op() {
        let mut u = Uniforms::default();
        assert!(!u.use_winding_fill);
        u.use_winding_fill(true);
        // The flag round-trips (accepted)...
        assert!(u.use_winding_fill);
        // ...but it is the only thing that changed: no other uniform moved.
        let baseline = Uniforms {
            use_winding_fill: true,
            ..Default::default()
        };
        assert_eq!(u, baseline);
    }
}
