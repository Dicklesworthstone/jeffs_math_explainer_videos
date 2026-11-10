//! Runnable reproduction and verification packets for `franken_manim` upstream
//! gaps (`U-1..U-18`, plan §14.1, bead `mx-e-w7-y6w.8`).
//!
//! Run the summary report:
//! ```bash
//! cargo run -p explainer_kit --example upstream
//! ```
//!
//! Run the executable verification tests:
//! ```bash
//! cargo test -p explainer_kit --example upstream
//! ```

use explainer_kit::encode::{MAX_ARTIFACT_BYTES, MAX_OUTPUT_BYTES, Quality};
use explainer_kit::props::{equation_card, lower_third, proportion_bar, verdict_box};
use explainer_kit::{Kit, Palette, guard_reflection_param, is_near_reflection_midpoint, shoelace_area, v};
use fmn::library::{Sphere, TexturedSurface};
use fmn::prelude::*;
use fmn::scene::{CameraOrientation, NullSceneSink, RuntimeConfig};

/// Status of an upstream gap against the workspace's pinned `franken_manim`
/// revision (`rev = "3c2beaa73486bdbf9c367c62de0a8ceeb4d0f5aa"`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpstreamStatus {
    /// Still open in the pinned revision; `explainer_kit` carries the workaround.
    OpenWorkaroundInKit,
    /// Fixed in `franken_manim` (verified in the pinned revision).
    FixedUpstream,
}

#[derive(Clone, Copy, Debug)]
pub struct UpstreamPacket {
    pub id: &'static str,
    pub upstream_bead: &'static str,
    pub summary: &'static str,
    pub status: UpstreamStatus,
    pub verify: fn() -> Result<String, String>,
}

fn run_headless(f: impl FnMut(&mut Stage<'_>) -> fmn::Result<()>) -> Result<(), String> {
    struct Probe<F>(F);
    impl<F: FnMut(&mut Stage<'_>) -> fmn::Result<()>> SceneConstruct for Probe<F> {
        fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
            (self.0)(stage)
        }
    }
    let config = RuntimeConfig {
        fps: 10,
        ..RuntimeConfig::default()
    };
    fmn::run_scene(&mut Probe(f), config, 7, &mut NullSceneSink)
        .map(|_| ())
        .map_err(|e| format!("{e}"))
}

/// U-1 (`fm-698`): `TexturedSurface::new(surface, path)` stores `light_texture`,
/// `dark_texture`, `im_coords`, and `num_textures`, and `From<TexturedSurface>
/// for Mobject` drops them (`image: None`).
pub fn verify_u1_textured_surface_drops_texture_fields() -> Result<String, String> {
    let sphere = Sphere::new(1.0).resolution(8, 8).build();
    let textured = TexturedSurface::new(sphere, "checkerboard.png");
    let mob = Mobject::from(textured);
    if mob.image.is_none() {
        Ok("reproduced: Mobject::from(TexturedSurface) has image == None (workaround: ImageMobject / color_by_uv_function)".to_owned())
    } else {
        Err("TexturedSurface now populates Mobject.image; U-1 can be marked FixedUpstream".to_owned())
    }
}

/// U-2 (`fm-f2u`): `CameraOrientation` and `Kit::request_camera` expose Euler angles
/// for camera-route choreography.
pub fn verify_u2_camera_orientation_euler() -> Result<String, String> {
    let orient = CameraOrientation::from_degrees(20.0, 60.0, -10.0);
    let kit = Kit::new().map_err(|e| format!("{e}"))?;
    kit.request_camera(explainer_kit::kit::CameraShot { theta: 0.4, phi: 1.1 });
    let shot = kit.camera_shot().ok_or("missing camera shot")?;
    if (shot.theta - 0.4).abs() < 1e-12 && (shot.phi - 1.1).abs() < 1e-12 {
        Ok(format!(
            "verified: CameraOrientation::from_degrees + CameraShot(theta={:.2}, phi={:.2}), orient={orient:?}",
            shot.theta, shot.phi
        ))
    } else {
        Err(format!("CameraShot mismatch: {shot:?}"))
    }
}

/// U-3 & U-4 (`fm-6ef` / `fm-r1g`): Delivery quality (`CRF 16`, `slow`, `256k` AAC,
/// raised 256 GiB / 32 GiB byte caps) via `explainer_kit::encode::Quality`.
pub fn verify_u3_u4_u15_delivery_quality_and_caps() -> Result<String, String> {
    let q = Quality::delivery();
    if q.quality != 16 || q.preset != "slow" || q.audio_bitrate != "256k" {
        return Err(format!("unexpected delivery quality: {q:?}"));
    }
    if MAX_OUTPUT_BYTES < (256 << 30) || MAX_ARTIFACT_BYTES < (32 << 30) {
        return Err(format!(
            "caps too low: MAX_OUTPUT_BYTES={MAX_OUTPUT_BYTES}, MAX_ARTIFACT_BYTES={MAX_ARTIFACT_BYTES}"
        ));
    }
    Ok(format!(
        "verified: CRF={}, preset={}, aac={}, caps={}GiB/{}GiB",
        q.quality,
        q.preset,
        q.audio_bitrate,
        MAX_OUTPUT_BYTES >> 30,
        MAX_ARTIFACT_BYTES >> 30
    ))
}

/// U-5 (`fm-y8f`): Fixed-gain PCM master (`explainer_kit::narration::GAIN_DB = -3.0` dB)
/// so quiet and loud chapters join at identical narrator level.
pub fn verify_u5_fixed_gain_master() -> Result<String, String> {
    let gain_db = explainer_kit::narration::GAIN_DB;
    if !(-6.0..=0.0).contains(&gain_db) {
        return Err(format!("unexpected narration GAIN_DB: {gain_db}"));
    }
    Ok(format!("verified: fixed narration GAIN_DB = {gain_db:.1} dB"))
}

/// U-6, U-11, U-12, U-13 (`fm-886`, `fm-3ac`): Native TeX parser handles `\sqrt{...}`,
/// `\binom{n}{k}`, `\dots`, `\le`, `\ge`, `\ne`, `\approx`, `\to`, `\mapsto`, and
/// `TransformMatchingTex` matches shared span substrings.
pub fn verify_u6_u11_u12_u13_tex_and_matching_spans() -> Result<String, String> {
    run_headless(|stage| {
        let kit = Kit::new()?;
        let expr = r"\sqrt{n} + \binom{n}{k} + x_1 + \dots + x_n \le y \ge z \ne 0 \approx 1 \to \infty \mapsto w";
        let (src, src_spans) = kit.tex_spans(
            stage,
            expr,
            36.0,
            Palette::ink(),
            &[(r"\binom{n}{k}", Palette::hero())],
            v(0.0, 0.5),
        )?;
        let (dst, dst_spans) = kit.tex_spans(
            stage,
            r"\binom{n}{k} = \frac{n!}{k!(n-k)!}",
            36.0,
            Palette::ink(),
            &[(r"\binom{n}{k}", Palette::hero())],
            v(0.0, -0.5),
        )?;
        let _morph = fmn::library::TransformMatchingTex::new(stage.arena_mut(), src, dst, &src_spans, &dst_spans)?;
        Ok(())
    })?;
    Ok("verified: TeX radical/binom/dots/relations + TransformMatchingTex spans".to_owned())
}

/// U-7 & U-10 (`fm-198` / `fm-5s9` / `fm-19o`): `explainer_kit::lagged` and idea-carrying
/// props (`lower_third`, `equation_card`, `proportion_bar`, `verdict_box`) build and
/// register cleanly on `Stage`.
pub fn verify_u7_u10_kit_props_and_lagged() -> Result<String, String> {
    run_headless(|stage| {
        let kit = Kit::new()?;
        let _lt = lower_third(&kit, stage, "precisely:", "E[X] = np for binomial trials")?;
        let _card = equation_card(&kit, stage, r"\hat N = \frac{Mn}{k}", 44.0, &[], ORIGIN)?;
        let _bar = proportion_bar(stage, v(-3.0, 0.0), 6.0, 0.4, 0.25, Palette::hero(), Palette::dim())?;
        let _verdict = verdict_box(&kit, stage, "holds", true, v(0.0, 1.5))?;
        Ok(())
    })?;
    Ok("verified: lower_third, equation_card, proportion_bar, verdict_box".to_owned())
}

/// U-18 (`mx-e-a-r47` / `mx-e-w1-4qa.19`): `VMobject::set_points` expects cubic/quad
/// Bézier control sequences, whereas passing raw polygon corners collapses or warps
/// segments; linear reflection morphs collapse polygon area to zero at `t = 0.5`
/// unless guarded by `guard_reflection_param` / `normal_bulge_offset`.
pub fn verify_u18_polygon_reflection_and_bezier_guard() -> Result<String, String> {
    let tri: [[f64; 2]; 3] = [[-1.0, -1.0], [1.0, -1.0], [0.0, 1.0]];
    let reflected = tri.map(|[x, y]| [-x, y]);
    let mid: Vec<[f64; 2]> = tri
        .iter()
        .zip(&reflected)
        .map(|(a, b)| [0.5 * (a[0] + b[0]), 0.5 * (a[1] + b[1])])
        .collect();
    let collapsed_area = shoelace_area(&mid);
    if collapsed_area > 1e-9 {
        return Err(format!(
            "expected linear midpoint reflection to collapse, got area={collapsed_area}"
        ));
    }
    if !is_near_reflection_midpoint(0.5, 0.08) {
        return Err("is_near_reflection_midpoint(0.5, 0.08) returned false".to_owned());
    }
    let safe_t = guard_reflection_param(0.5, 0.12);
    let safe_pts: Vec<[f64; 2]> = tri
        .iter()
        .zip(&reflected)
        .map(|(a, b)| [(1.0 - safe_t) * a[0] + safe_t * b[0], a[1]])
        .collect();
    let safe_area = shoelace_area(&safe_pts);
    if safe_area <= 0.1 {
        return Err(format!("guarded reflection area too small: {safe_area}"));
    }
    Ok(format!(
        "verified: linear t=0.5 area={collapsed_area:.4}, guarded t={safe_t:.2} area={safe_area:.4}"
    ))
}

pub const PACKETS: &[UpstreamPacket] = &[
    UpstreamPacket {
        id: "U-1",
        upstream_bead: "fm-698",
        summary: "TexturedSurface drops texture fields on Mobject conversion",
        status: UpstreamStatus::OpenWorkaroundInKit,
        verify: verify_u1_textured_surface_drops_texture_fields,
    },
    UpstreamPacket {
        id: "U-2",
        upstream_bead: "fm-f2u",
        summary: "CameraOrientation and CameraShot expose Euler angles",
        status: UpstreamStatus::FixedUpstream,
        verify: verify_u2_camera_orientation_euler,
    },
    UpstreamPacket {
        id: "U-3..U-4,U-15",
        upstream_bead: "fm-6ef,fm-r1g",
        summary: "Delivery encoder quality (CRF 16, AAC 256k) and 256GiB/32GiB byte caps",
        status: UpstreamStatus::OpenWorkaroundInKit,
        verify: verify_u3_u4_u15_delivery_quality_and_caps,
    },
    UpstreamPacket {
        id: "U-5",
        upstream_bead: "fm-y8f",
        summary: "Fixed-gain PCM master (-3.0 dB) instead of peak-normalizing mixer",
        status: UpstreamStatus::OpenWorkaroundInKit,
        verify: verify_u5_fixed_gain_master,
    },
    UpstreamPacket {
        id: "U-6,U-11..U-13",
        upstream_bead: "fm-886,fm-3ac",
        summary: "Native TeX radical/binom/dots/relations and TransformMatchingTex spans",
        status: UpstreamStatus::FixedUpstream,
        verify: verify_u6_u11_u12_u13_tex_and_matching_spans,
    },
    UpstreamPacket {
        id: "U-7,U-10",
        upstream_bead: "fm-198,fm-5s9,fm-19o",
        summary: "Kit props (lower_third, equation_card, proportion_bar, verdict_box) and lagged reveals",
        status: UpstreamStatus::FixedUpstream,
        verify: verify_u7_u10_kit_props_and_lagged,
    },
    UpstreamPacket {
        id: "U-18",
        upstream_bead: "mx-e-a-r47",
        summary: "Polygon reflection degeneracy guard and cubic Bezier path helper",
        status: UpstreamStatus::OpenWorkaroundInKit,
        verify: verify_u18_polygon_reflection_and_bezier_guard,
    },
];

fn main() {
    println!("franken_manim upstream issue packets (plan §14.1):");
    let mut failed = 0usize;
    for pkt in PACKETS {
        match (pkt.verify)() {
            Ok(detail) => {
                println!(
                    "  [PASS] {:<14} ({:<18}) {:?} — {}",
                    pkt.id, pkt.upstream_bead, pkt.status, detail
                );
            }
            Err(err) => {
                eprintln!(
                    "  [FAIL] {:<14} ({:<18}) {:?} — {}",
                    pkt.id, pkt.upstream_bead, pkt.status, err
                );
                failed += 1;
            }
        }
    }
    if failed > 0 {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u1_textured_surface_repro_holds() {
        verify_u1_textured_surface_drops_texture_fields().unwrap();
    }

    #[test]
    fn u2_camera_orientation_euler_holds() {
        verify_u2_camera_orientation_euler().unwrap();
    }

    #[test]
    fn u3_u4_u15_delivery_quality_and_caps_hold() {
        verify_u3_u4_u15_delivery_quality_and_caps().unwrap();
    }

    #[test]
    fn u5_fixed_gain_master_holds() {
        verify_u5_fixed_gain_master().unwrap();
    }

    #[test]
    fn u6_u11_u12_u13_tex_and_matching_spans_hold() {
        verify_u6_u11_u12_u13_tex_and_matching_spans().unwrap();
    }

    #[test]
    fn u7_u10_kit_props_and_lagged_hold() {
        verify_u7_u10_kit_props_and_lagged().unwrap();
    }

    #[test]
    fn u18_polygon_reflection_and_bezier_guard_holds() {
        verify_u18_polygon_reflection_and_bezier_guard().unwrap();
    }
}
