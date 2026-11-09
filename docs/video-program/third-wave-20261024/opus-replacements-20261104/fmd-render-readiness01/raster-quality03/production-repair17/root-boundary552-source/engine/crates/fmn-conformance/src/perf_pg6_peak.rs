//! Canonical PG-6 4K 3D-gallery peak-residency producer.
//!
//! One repetition constructs and renders the complete three-view 4K corpus
//! through the public W8 solid builders and Lumen's certified [`ThreeDJob`].
//! A concurrent `VmRSS` probe plus explicit before/after samples retains the
//! largest resident-set observation while the frame and prepared job are
//! live. Every repetition is real work; samples are never duplicated to meet
//! the policy denominator.

use crate::perf::{
    Baseline, Direction, Enforcement, EvidenceKind, EvidenceRef, GateId, GateScope,
    MeasurementBatch, MetricUnit, Sample, require_compiled_cargo_profile, validate_producer_commit,
};
use crate::perf_pg6::{Pg6Error, pg6_identity};
use fmn_core::color::Srgb;
use fmn_core::constants::{BLUE_C, DEFAULT_BACKGROUND_COLOR, GOLD_C, PI, RED_C, RIGHT, TEAL_C};
use fmn_hash::{Digest, sha256};
use fmn_library::{Cube, Cylinder, Sphere, Surface, Torus};
use fmn_render::{
    Camera, CameraConfig, SurfaceDraw, SurfaceMaterial, SurfaceMesh, SurfaceVertex, ThreeDDraw,
    ThreeDJob, ThreeDPreparationLimits, Tiling, frame_digest,
};
use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

/// Stable workload-definition schema.
pub const PG6_PEAK_DEFINITION_SCHEMA: &str = "fmn-perf-pg6-peak-definition/1";
/// Stable phase-trace schema.
pub const PG6_PEAK_TRACE_SCHEMA: &str = "fmn-perf-pg6-peak-trace/1";
/// Policy-catalog scenario implemented by this producer.
pub const PG6_PEAK_SCENARIO: &str = "gallery-4k-3d-peak";
/// The policy's strict peak-residency ceiling.
pub const PG6_PEAK_TARGET_BYTES: u64 = 1_500_000_000;
/// Nine attributable repetitions are required.
pub const PG6_PEAK_MIN_VALID_SAMPLES: usize = 9;
/// Two explicitly invalid host observations are tolerated.
pub const PG6_PEAK_MAX_INVALID_SAMPLES: usize = 2;
/// Nine required repetitions plus the two retained-invalid slots.
pub const PG6_PEAK_SAMPLE_COUNT: usize = 11;
/// The three camera states in the certified corpus.
pub const PG6_PEAK_CASE_COUNT: usize = 3;
/// UHD frame width.
pub const PG6_PEAK_WIDTH: u32 = 3840;
/// UHD frame height.
pub const PG6_PEAK_HEIGHT: u32 = 2160;
/// Fixed certified render-team size.
pub const PG6_PEAK_THREADS: usize = 4;
/// Adaptive edge-sampling ceiling used by the 3D camera.
pub const PG6_PEAK_CAMERA_SAMPLES: u8 = 4;
/// Retained reason when the host has no resident-set capability.
pub const PG6_PEAK_UNSUPPORTED_REASON: &str = "rss-unsupported-host";

const BUILD_PROFILE: &str = "release-perf";
const THREAD_PROFILE: &str = "fixed-4-plus-rss-sampler";
const CACHE_STATE: &str = "cold-gallery-pass";
const OUTPUT_MODE: &str = "raw-rgba16f";
const POLL_INTERVAL: Duration = Duration::from_millis(1);
const TILING: Tiling = Tiling {
    macro_tile: 128,
    fine_tile: 16,
};

/// Canonical first row of the certified 4K 3D corpus lock.
pub const GALLERY_3D_LOCK_HEADER: &str = "# fmn-gallery-3d-lock v1 key=certified";
/// Bit-locked frame identities for every camera state.
pub const GALLERY_3D_LOCK: &str = include_str!("../goldens/gallery_3d.certified.lock");

#[derive(Clone, Copy, Debug, PartialEq)]
struct GalleryCase {
    name: &'static str,
    euler: [f64; 3],
}

const CASES: [GalleryCase; PG6_PEAK_CASE_COUNT] = [
    GalleryCase {
        name: "solids-surfaces.front.v1",
        euler: [0.0, 0.0, 0.0],
    },
    GalleryCase {
        name: "solids-surfaces.quarter.v1",
        euler: [-PI / 4.0, 13.0 * PI / 36.0, 0.0],
    },
    GalleryCase {
        name: "solids-surfaces.orbit.v1",
        euler: [11.0 * PI / 18.0, 5.0 * PI / 12.0, -PI / 15.0],
    },
];

/// Content-addressed definition of the 4K 3D peak-residency workload.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pg6PeakDefinition;

impl Pg6PeakDefinition {
    /// Construct the sole canonical peak-residency definition.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Exact definition bytes hashed into [`crate::perf::BenchmarkKey`].
    #[must_use]
    pub fn to_tsv(self) -> String {
        let mut output = String::new();
        {
            let mut row = |name: &str, value: &dyn fmt::Display| {
                let _ = writeln!(output, "{name}\t{value}");
            };
            row("schema", &PG6_PEAK_DEFINITION_SCHEMA);
            row("gate", &GateId::Pg6);
            row("scenario", &PG6_PEAK_SCENARIO);
            row("unit", &MetricUnit::Bytes.name());
            row("target", &PG6_PEAK_TARGET_BYTES);
            row("sample_count", &PG6_PEAK_SAMPLE_COUNT);
            row("sample_scope", &"one-complete-three-view-corpus-pass");
            row("threads", &PG6_PEAK_THREADS);
            row("thread_profile", &THREAD_PROFILE);
            row("rss_poll_interval_us", &POLL_INTERVAL.as_micros());
            row("cache_state", &CACHE_STATE);
            row("output_mode", &OUTPUT_MODE);
            row("frame_width_px", &PG6_PEAK_WIDTH);
            row("frame_height_px", &PG6_PEAK_HEIGHT);
            row("camera_samples", &PG6_PEAK_CAMERA_SAMPLES);
            row("engine", &pg6_identity().engine.name());
            row("tier", &pg6_identity().tier.name());
            row("gallery_lock_digest", &self.corpus_lock_digest());
            row("config_digest", &self.config_digest());
        }
        for (index, case) in CASES.iter().enumerate() {
            let _ = writeln!(
                output,
                "scene\t{index}\t{}\t{}\t{}\t{}",
                case.name, case.euler[0], case.euler[1], case.euler[2],
            );
        }
        output
    }

    /// SHA-256 of [`Self::to_tsv`].
    #[must_use]
    pub fn digest(self) -> Digest {
        sha256(self.to_tsv().as_bytes())
    }

    /// Exact camera, renderer, corpus, and resource configuration identity.
    #[must_use]
    pub fn config_digest(self) -> Digest {
        let mut config = String::new();
        let _ = writeln!(config, "schema\tfmn-perf-pg6-peak-config/1");
        let _ = writeln!(config, "resolution\t{PG6_PEAK_WIDTH}\t{PG6_PEAK_HEIGHT}");
        let _ = writeln!(config, "camera_samples\t{PG6_PEAK_CAMERA_SAMPLES}");
        let _ = writeln!(config, "background\t#333333");
        let _ = writeln!(
            config,
            "tiling\t{}\t{}",
            TILING.macro_tile, TILING.fine_tile
        );
        let _ = writeln!(config, "threads\t{PG6_PEAK_THREADS}");
        let limits = ThreeDPreparationLimits::default();
        let _ = writeln!(
            config,
            "preparation_limits\t{}\t{}\t{}\t{}\t{}",
            limits.max_draws,
            limits.max_projected_curves,
            limits.max_fill_pieces,
            limits.max_raster_triangles,
            limits.max_working_bytes,
        );
        for case in CASES {
            let _ = writeln!(
                config,
                "camera\t{}\t{}\t{}\t{}",
                case.name, case.euler[0], case.euler[1], case.euler[2],
            );
        }
        sha256(config.as_bytes())
    }

    /// SHA-256 identity of the certified corpus lock.
    #[must_use]
    pub fn corpus_lock_digest(self) -> Digest {
        sha256(GALLERY_3D_LOCK.as_bytes())
    }

    /// Parse and prove exact lock/corpus set equality.
    pub fn validate_corpus_lock(self) -> Result<(), Pg6Error> {
        let lock = parse_lock()?;
        if lock.len() != CASES.len() {
            return Err(Pg6Error::Fixture(format!(
                "3D gallery lock has {} rows, expected {}",
                lock.len(),
                CASES.len(),
            )));
        }
        let missing: Vec<_> = CASES
            .iter()
            .filter(|case| !lock.contains_key(case.name))
            .map(|case| case.name)
            .collect();
        if !missing.is_empty() {
            return Err(Pg6Error::Fixture(format!(
                "3D gallery lock/corpus mismatch: missing {missing:?}",
            )));
        }
        Ok(())
    }

    /// Validate a baseline against the complete compiled producer identity.
    pub fn validate_baseline(self, baseline: &Baseline) -> Result<(), Pg6Error> {
        baseline.validate()?;
        let key = &baseline.key;
        let mut mismatches = Vec::new();
        if baseline.policy.gate != GateId::Pg6 {
            mismatches.push("gate");
        }
        if baseline.policy.scenario != PG6_PEAK_SCENARIO {
            mismatches.push("scenario");
        }
        if baseline.policy.unit != MetricUnit::Bytes {
            mismatches.push("unit");
        }
        if baseline.policy.direction != Direction::AtMost {
            mismatches.push("direction");
        }
        if baseline.policy.target != Some(PG6_PEAK_TARGET_BYTES) {
            mismatches.push("target");
        }
        if baseline.policy.min_valid_samples != PG6_PEAK_MIN_VALID_SAMPLES {
            mismatches.push("min_valid_samples");
        }
        if baseline.policy.max_invalid_samples != PG6_PEAK_MAX_INVALID_SAMPLES {
            mismatches.push("max_invalid_samples");
        }
        if baseline.policy.max_mad_bps != 1000 {
            mismatches.push("max_mad_bps");
        }
        if baseline.policy.alert_regression_bps != 500 {
            mismatches.push("alert_regression_bps");
        }
        if baseline.policy.block_regression_bps != 1000 {
            mismatches.push("block_regression_bps");
        }
        if baseline.policy.enforcement != Enforcement::Blocking {
            mismatches.push("enforcement");
        }
        if baseline.policy.scope != GateScope::Core {
            mismatches.push("scope");
        }
        if !baseline.policy.require_regression_profile {
            mismatches.push("require_regression_profile");
        }
        if key.benchmark_definition != self.digest() {
            mismatches.push("benchmark_definition");
        }
        if key.config_digest != self.config_digest() {
            mismatches.push("config_digest");
        }
        if key.build_profile != BUILD_PROFILE {
            mismatches.push("build_profile");
        }
        if key.engine != pg6_identity().engine.name() {
            mismatches.push("engine");
        }
        if key.tier != pg6_identity().tier.name() {
            mismatches.push("tier");
        }
        if key.thread_profile != THREAD_PROFILE {
            mismatches.push("thread_profile");
        }
        if key.cache_state != CACHE_STATE {
            mismatches.push("cache_state");
        }
        if key.output_mode != OUTPUT_MODE {
            mismatches.push("output_mode");
        }
        if key.external_tool_fingerprint.is_some() {
            mismatches.push("external_tool_fingerprint");
        }
        if mismatches.is_empty() {
            Ok(())
        } else {
            Err(Pg6Error::Identity(format!(
                "{PG6_PEAK_SCENARIO} baseline differs from the compiled producer in: {}",
                mismatches.join(", "),
            )))
        }
    }
}

/// One locked camera state's rendered proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg6PeakCaseResult {
    /// Stable lock/corpus name.
    pub scene: &'static str,
    /// Certified raw-frame identity.
    pub frame_digest: Digest,
    /// Conservative bytes admitted while preparing the camera-bound job.
    pub preparation_bytes: u64,
}

/// One real repetition over the complete three-view corpus.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg6PeakPass {
    /// Zero-based repetition index.
    pub repetition: usize,
    /// Largest resident-set observation while this pass was live.
    pub peak_rss_bytes: u64,
    /// Every camera state rendered and lock-verified in order.
    pub cases: Vec<Pg6PeakCaseResult>,
}

/// Measurement output before the caller persists trace and raw artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pg6PeakArtifacts {
    /// Canonical raw measurement bundle.
    pub batch: MeasurementBatch,
    /// Exact bytes named by the phase-trace evidence row.
    pub trace_tsv: String,
    /// Supported-host passes; empty when residency is unavailable.
    pub passes: Vec<Pg6PeakPass>,
}

/// Render and lock-check the complete production 4K corpus once.
pub fn render_locked_gallery_once() -> Result<Vec<Pg6PeakCaseResult>, Pg6Error> {
    Pg6PeakDefinition::new().validate_corpus_lock()?;
    render_gallery(
        (PG6_PEAK_WIDTH, PG6_PEAK_HEIGHT),
        ThreeDPreparationLimits::default(),
        true,
    )
}

/// Run the real 4K corpus and retain one independently sampled peak per pass.
pub fn measure_pg6_peak(
    baseline: &Baseline,
    producer_commit: &str,
    trace_path: impl Into<String>,
    rss_probe: &(dyn Fn() -> Result<Option<u64>, String> + Sync),
    qualification: Option<&crate::perf_host::HostQualification>,
) -> Result<Pg6PeakArtifacts, Pg6Error> {
    let definition = Pg6PeakDefinition::new();
    definition.validate_baseline(baseline)?;
    validate_producer_commit(producer_commit)?;
    let trace_path = trace_path.into();
    let _ = EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path.clone(), &[])?;
    require_compiled_cargo_profile(BUILD_PROFILE)?;
    definition.validate_corpus_lock()?;

    let (key, host_evidence) =
        crate::perf_host::measurement_identity(&baseline.key, qualification)?;
    let mut batch = MeasurementBatch {
        key,
        producer_commit: producer_commit.to_owned(),
        samples: Vec::with_capacity(PG6_PEAK_SAMPLE_COUNT),
        evidence: host_evidence,
    };
    let _ = batch.to_tsv()?;

    let mut passes = Vec::new();
    match rss_probe().map_err(|detail| Pg6Error::Fixture(format!("rss probe: {detail}")))? {
        None => {
            for _ in 0..PG6_PEAK_SAMPLE_COUNT {
                batch
                    .samples
                    .push(Sample::invalid(0, PG6_PEAK_UNSUPPORTED_REASON));
            }
        }
        Some(_) => {
            passes.reserve(PG6_PEAK_SAMPLE_COUNT);
            for repetition in 0..PG6_PEAK_SAMPLE_COUNT {
                let (peak_rss_bytes, cases) =
                    sample_peak_rss(rss_probe, render_locked_gallery_once)?;
                batch.samples.push(Sample::valid(peak_rss_bytes));
                passes.push(Pg6PeakPass {
                    repetition,
                    peak_rss_bytes,
                    cases,
                });
            }
        }
    }

    let trace_tsv = render_trace(definition, &passes);
    let evidence =
        EvidenceRef::from_bytes(EvidenceKind::PhaseTrace, trace_path, trace_tsv.as_bytes())?;
    batch.evidence.push(evidence);
    let _ = batch.to_tsv()?;
    Ok(Pg6PeakArtifacts {
        batch,
        trace_tsv,
        passes,
    })
}

fn sample_peak_rss<T>(
    rss_probe: &(dyn Fn() -> Result<Option<u64>, String> + Sync),
    work: impl FnOnce() -> Result<T, Pg6Error>,
) -> Result<(u64, T), Pg6Error> {
    let first = rss_probe()
        .map_err(|detail| Pg6Error::Fixture(format!("rss probe: {detail}")))?
        .ok_or_else(|| Pg6Error::Fixture("rss probe stopped reporting mid-pass".to_owned()))?;
    let peak = AtomicU64::new(first);
    let stop = AtomicBool::new(false);
    let failure = Mutex::new(None::<String>);

    let result = std::thread::scope(|scope| {
        let sampler = scope.spawn(|| {
            while !stop.load(Ordering::Acquire) {
                match rss_probe() {
                    Ok(Some(value)) => {
                        peak.fetch_max(value, Ordering::Relaxed);
                    }
                    Ok(None) => {
                        let mut slot = failure.lock().unwrap_or_else(PoisonError::into_inner);
                        slot.get_or_insert_with(|| {
                            "rss probe stopped reporting mid-pass".to_owned()
                        });
                        break;
                    }
                    Err(detail) => {
                        let mut slot = failure.lock().unwrap_or_else(PoisonError::into_inner);
                        slot.get_or_insert_with(|| format!("rss probe: {detail}"));
                        break;
                    }
                }
                std::thread::sleep(POLL_INTERVAL);
            }
        });
        let value = work();
        match rss_probe() {
            Ok(Some(value)) => {
                peak.fetch_max(value, Ordering::Relaxed);
            }
            Ok(None) => {
                let mut slot = failure.lock().unwrap_or_else(PoisonError::into_inner);
                slot.get_or_insert_with(|| "rss probe stopped reporting mid-pass".to_owned());
            }
            Err(detail) => {
                let mut slot = failure.lock().unwrap_or_else(PoisonError::into_inner);
                slot.get_or_insert_with(|| format!("rss probe: {detail}"));
            }
        }
        stop.store(true, Ordering::Release);
        sampler
            .join()
            .map_err(|_| Pg6Error::Fixture("rss sampler thread panicked".to_owned()))?;
        value
    });
    let value = result?;
    if let Some(detail) = failure.into_inner().unwrap_or_else(PoisonError::into_inner) {
        return Err(Pg6Error::Fixture(detail));
    }
    Ok((peak.load(Ordering::Relaxed), value))
}

fn render_gallery(
    resolution: (u32, u32),
    limits: ThreeDPreparationLimits,
    verify_lock: bool,
) -> Result<Vec<Pg6PeakCaseResult>, Pg6Error> {
    let expected = if verify_lock {
        parse_lock()?
    } else {
        BTreeMap::new()
    };
    let mut results = Vec::with_capacity(CASES.len());
    for case in CASES {
        let (digest, preparation_bytes) = render_case(case, resolution, limits)?;
        if let Some(expected_digest) = expected.get(case.name)
            && digest != *expected_digest
        {
            return Err(Pg6Error::Render(format!(
                "3D gallery frame drift for {}: expected {}, got {}",
                case.name, expected_digest, digest,
            )));
        }
        results.push(Pg6PeakCaseResult {
            scene: case.name,
            frame_digest: digest,
            preparation_bytes,
        });
    }
    Ok(results)
}

fn render_case(
    case: GalleryCase,
    resolution: (u32, u32),
    limits: ThreeDPreparationLimits,
) -> Result<(Digest, u64), Pg6Error> {
    let surfaces = gallery_surfaces();
    let meshes: Vec<_> = surfaces
        .iter()
        .map(surface_mesh)
        .collect::<Result<_, _>>()?;
    let draws: Vec<_> = meshes
        .iter()
        .zip(&surfaces)
        .map(|(mesh, surface)| {
            let uniforms = surface.uniforms();
            ThreeDDraw::Surface(SurfaceDraw {
                mesh,
                material: SurfaceMaterial::VertexColor,
                shading: uniforms.shading,
                is_fixed_in_frame: uniforms.is_fixed_in_frame,
                clip_planes: uniforms.clip_planes,
                depth_test: uniforms.depth_test,
            })
        })
        .collect();
    let mut camera = Camera::new(CameraConfig {
        resolution,
        samples: PG6_PEAK_CAMERA_SAMPLES,
        background: DEFAULT_BACKGROUND_COLOR.to_linear(1.0),
        ..CameraConfig::default()
    })
    .map_err(|error| Pg6Error::Fixture(format!("{} camera: {error}", case.name)))?;
    camera
        .frame_mut()
        .set_euler_angles(
            Some(case.euler[0]),
            Some(case.euler[1]),
            Some(case.euler[2]),
        )
        .map_err(|error| Pg6Error::Fixture(format!("{} camera pose: {error}", case.name)))?;
    let job = ThreeDJob::new_with_limits(&camera, &draws, TILING, limits)
        .map_err(|error| Pg6Error::Fixture(format!("{} preparation: {error}", case.name)))?;
    let preparation_bytes = job.preparation_bytes();
    let frame = job
        .render(PG6_PEAK_THREADS)
        .map_err(|error| Pg6Error::Render(format!("{} frame: {error}", case.name)))?;
    let digest = frame_digest(&frame)
        .map_err(|error| Pg6Error::Render(format!("{} digest: {error}", case.name)))?;
    Ok((digest, preparation_bytes))
}

fn gallery_surfaces() -> Vec<Surface> {
    let mut surfaces = vec![
        Sphere::new(1.25)
            .resolution(25, 13)
            .color(BLUE_C)
            .build()
            .shifted([-2.4, 1.25, 0.0]),
        Torus::new(1.2, 0.42)
            .resolution(25, 13)
            .color(GOLD_C)
            .build()
            .shifted([2.2, 1.15, 0.0]),
        Cylinder::new(2.8, 0.7)
            .resolution(25, 7)
            .axis(RIGHT)
            .color(TEAL_C)
            .build()
            .shifted([0.0, -1.75, 0.0]),
    ];
    surfaces.extend(
        Cube::new(1.5)
            .color(RED_C)
            .build()
            .children()
            .iter()
            .cloned()
            .map(|face| face.shifted([0.0, 0.0, 0.5])),
    );
    surfaces
}

fn surface_mesh(surface: &Surface) -> Result<SurfaceMesh, Pg6Error> {
    let normals = surface.unit_normals();
    let vertices = surface
        .points()
        .iter()
        .zip(normals)
        .zip(surface.rgba())
        .map(|((&point, normal), rgba)| {
            SurfaceVertex::colored(
                point,
                normal,
                Srgb {
                    r: rgba[0],
                    g: rgba[1],
                    b: rgba[2],
                }
                .to_linear(rgba[3]),
            )
        })
        .collect();
    let (nu, nv) = surface.resolution();
    let resolution = (
        u32::try_from(nu)
            .map_err(|_| Pg6Error::Fixture("surface u resolution exceeds u32".to_owned()))?,
        u32::try_from(nv)
            .map_err(|_| Pg6Error::Fixture("surface v resolution exceeds u32".to_owned()))?,
    );
    SurfaceMesh::from_uv_grid(vertices, resolution)
        .map_err(|error| Pg6Error::Fixture(format!("surface mesh: {error}")))
}

fn parse_lock() -> Result<BTreeMap<&'static str, Digest>, Pg6Error> {
    let mut lines = GALLERY_3D_LOCK.lines();
    if lines.next() != Some(GALLERY_3D_LOCK_HEADER) {
        return Err(Pg6Error::Fixture(
            "3D gallery lock header does not match the certified v1 schema".to_owned(),
        ));
    }
    let mut rows = BTreeMap::new();
    for (index, line) in lines.enumerate() {
        let mut fields = line.split('\t');
        let name = fields.next().unwrap_or_default();
        let digest = fields.next().unwrap_or_default();
        if name.is_empty() || fields.next().is_some() {
            return Err(Pg6Error::Fixture(format!(
                "malformed 3D gallery lock row {}",
                index + 2,
            )));
        }
        let case = CASES
            .iter()
            .find(|case| case.name == name)
            .ok_or_else(|| Pg6Error::Fixture(format!("stale 3D gallery lock row {name:?}")))?;
        let digest = Digest::from_hex(digest).map_err(|_| {
            Pg6Error::Fixture(format!(
                "malformed 3D gallery lock digest on row {}",
                index + 2
            ))
        })?;
        if rows.insert(case.name, digest).is_some() {
            return Err(Pg6Error::Fixture(format!(
                "duplicate 3D gallery lock row {name:?}",
            )));
        }
    }
    Ok(rows)
}

fn render_trace(definition: Pg6PeakDefinition, passes: &[Pg6PeakPass]) -> String {
    let mut output = String::new();
    {
        let mut row = |name: &str, value: &dyn fmt::Display| {
            let _ = writeln!(output, "{name}\t{value}");
        };
        row("schema", &PG6_PEAK_TRACE_SCHEMA);
        row("gate", &GateId::Pg6);
        row("scenario", &PG6_PEAK_SCENARIO);
        row("benchmark_definition", &definition.digest());
        row("config_digest", &definition.config_digest());
        row("gallery_lock_digest", &definition.corpus_lock_digest());
        row("engine", &pg6_identity().engine.name());
        row("tier", &pg6_identity().tier.name());
        row("thread_profile", &THREAD_PROFILE);
        row("threads", &PG6_PEAK_THREADS);
        row("sample_count", &passes.len());
        if passes.is_empty() {
            row("unsupported", &PG6_PEAK_UNSUPPORTED_REASON);
        }
    }
    for pass in passes {
        let _ = writeln!(
            output,
            "sample\t{}\t{}\t{}",
            pass.repetition,
            pass.peak_rss_bytes,
            pass.cases.len(),
        );
        for case in &pass.cases {
            let _ = writeln!(
                output,
                "frame\t{}\t{}\t{}\t{}",
                pass.repetition, case.scene, case.frame_digest, case.preparation_bytes,
            );
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn definition_binds_every_peak_and_corpus_axis() {
        let definition = Pg6PeakDefinition::new();
        let text = definition.to_tsv();
        assert!(text.contains("scenario\tgallery-4k-3d-peak\n"));
        assert!(text.contains("sample_scope\tone-complete-three-view-corpus-pass\n"));
        assert!(text.contains("frame_width_px\t3840\n"));
        assert!(text.contains("frame_height_px\t2160\n"));
        assert_eq!(
            text.lines()
                .filter(|line| line.starts_with("scene\t"))
                .count(),
            PG6_PEAK_CASE_COUNT,
        );
    }

    #[test]
    fn embedded_lock_and_compiled_gallery_are_exactly_aligned() {
        Pg6PeakDefinition::new()
            .validate_corpus_lock()
            .expect("committed 3D lock and corpus agree");
    }

    #[test]
    fn camera_moves_change_real_small_frame_bits() {
        let cases = render_gallery((160, 90), ThreeDPreparationLimits::default(), false)
            .expect("small real 3D gallery");
        assert_eq!(cases.len(), PG6_PEAK_CASE_COUNT);
        assert!(cases.iter().all(|case| case.preparation_bytes > 0));
        assert_ne!(cases[0].frame_digest, cases[1].frame_digest);
        assert_ne!(cases[1].frame_digest, cases[2].frame_digest);
    }

    #[test]
    fn preparation_limit_refuses_before_frame_allocation() {
        let limits = ThreeDPreparationLimits {
            max_raster_triangles: 0,
            ..ThreeDPreparationLimits::default()
        };
        let error = render_case(CASES[0], (160, 90), limits)
            .expect_err("zero triangle budget must fail closed");
        assert!(error.to_string().contains("raster triangles"), "{error}");
    }

    #[test]
    fn concurrent_sampler_retains_a_transient_high_water_mark() {
        let calls = AtomicUsize::new(0);
        let probe = || {
            let call = calls.fetch_add(1, Ordering::SeqCst);
            Ok(Some(if call == 2 { 900 } else { 100 }))
        };
        let (peak, value) = sample_peak_rss(&probe, || {
            while calls.load(Ordering::SeqCst) < 3 {
                std::thread::yield_now();
            }
            Ok("rendered")
        })
        .expect("sampled pass");
        assert_eq!(peak, 900);
        assert_eq!(value, "rendered");
    }

    #[test]
    #[ignore = "real three-frame UHD corpus; run explicitly under --profile release-perf"]
    fn production_4k_gallery_matches_the_certified_lock() {
        let actual = render_gallery(
            (PG6_PEAK_WIDTH, PG6_PEAK_HEIGHT),
            ThreeDPreparationLimits::default(),
            false,
        )
        .expect("production 4K gallery");
        let expected = parse_lock().expect("certified lock");
        for case in &actual {
            eprintln!("{}\t{}", case.scene, case.frame_digest);
        }
        assert!(
            actual
                .iter()
                .all(|case| { expected.get(case.scene) == Some(&case.frame_digest) }),
            "one or more certified 4K gallery frames drifted"
        );
    }
}
