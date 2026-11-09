//! Provenance manifest generation and publication for the Python portal (§16.7, docs/INPUT_CLOSURE.md).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fmn_core::rng::RNG_LAYOUT_VERSION;
use fmn_hash::Digest;
use fmn_output::{
    ClosureItem, ManifestIdentity, ManifestMode, ManifestOutput, ProvenanceManifest,
    StructuralField,
};
use fmn_platform::fs::{FileSystem, StdFs};
use fmn_platform::process::{ProcessRunner, StdProcessRunner};
use fmn_render::{EngineIdentity, FrameConfig, ScreenMap, Tiling, Viewport};
use fmn_runtime::{ExecutionPlan, OutputPixelFormat, PlanRequest, RenderIntent};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::CapabilityError;

const BUILD_ID: &str = env!("FMN_BUILD_ID");

/// Certified output only from a wheel whose build identity names its sources
/// (ADR-0025); see `fmn_output::certified_build_refusal`.
pub(super) fn certified_build_check() -> PyResult<()> {
    fmn_output::certified_build_refusal(BUILD_ID).map_or(Ok(()), |reason| {
        Err(CapabilityError::new_err(format!("CAPABILITY: {reason}")))
    })
}
const TARGET_TRIPLE: &str = env!("FMN_TARGET_TRIPLE");
const CARGO_PROFILE: &str = env!("FMN_CARGO_PROFILE");
const SUITE_LOCK_BYTES: &[u8] = include_bytes!("../../../SUITE.lock");
const SUITE_LOCK_TEXT: &str = include_str!("../../../SUITE.lock");

pub(crate) fn pinned_toolchain() -> Result<&'static str, String> {
    let mut toolchain = false;
    for line in SUITE_LOCK_TEXT.lines() {
        if line.starts_with('[') {
            toolchain = line == "[toolchain]";
            continue;
        }
        if toolchain {
            let mut fields = line.split('\t');
            if fields.next() == Some("rustc")
                && let Some(value) = fields.next()
                && !value.is_empty()
            {
                return Ok(value);
            }
        }
    }
    Err("embedded SUITE.lock has no pinned rustc identity".to_owned())
}

pub(crate) const fn active_compiled_tier() -> &'static str {
    if cfg!(all(
        target_arch = "x86_64",
        target_feature = "avx512f",
        target_feature = "avx512bw",
        target_feature = "avx512dq",
        target_feature = "avx512vl"
    )) {
        "x86-64-v4"
    } else if cfg!(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "bmi2",
        target_feature = "fma"
    )) {
        "x86-64-v3"
    } else if cfg!(all(target_arch = "aarch64", target_feature = "neon")) {
        "aarch64+neon"
    } else {
        "portable"
    }
}

pub(crate) fn active_target_features() -> &'static str {
    match active_compiled_tier() {
        "x86-64-v3" => "+avx2,+bmi2,+fma",
        "x86-64-v4" => "+avx512f,+avx512bw,+avx512dq,+avx512vl",
        "aarch64+neon" => "+neon",
        _ => "baseline",
    }
}

const fn plan_determinism_name(determinism: fmn_runtime::Determinism) -> &'static str {
    match determinism {
        fmn_runtime::Determinism::Standard => "standard",
        fmn_runtime::Determinism::Certified => "certified",
    }
}

const fn plan_engine_name(engine: fmn_runtime::ExecutionEngine) -> &'static str {
    match engine {
        fmn_runtime::ExecutionEngine::CertifiedCpu => "certified-cpu",
        fmn_runtime::ExecutionEngine::FastCpu => "fast-cpu",
        fmn_runtime::ExecutionEngine::Metal => "metal",
        fmn_runtime::ExecutionEngine::Cuda => "cuda",
    }
}

const fn plan_intent_name(intent: fmn_runtime::RenderIntent) -> &'static str {
    match intent {
        fmn_runtime::RenderIntent::Preview => "preview",
        fmn_runtime::RenderIntent::Offline => "offline",
    }
}

const fn plan_output_name(format: fmn_runtime::OutputPixelFormat) -> &'static str {
    match format {
        fmn_runtime::OutputPixelFormat::Rgba16F => "rgba16f",
        fmn_runtime::OutputPixelFormat::Rgba8 => "rgba8",
        fmn_runtime::OutputPixelFormat::Bgra8 => "bgra8",
        fmn_runtime::OutputPixelFormat::Nv12 => "nv12",
        fmn_runtime::OutputPixelFormat::P010 => "p010",
    }
}

const fn plan_tuning_name(source: fmn_runtime::TuningSource) -> &'static str {
    match source {
        fmn_runtime::TuningSource::CertifiedProfile => "certified-profile",
        fmn_runtime::TuningSource::StandardBaseline => "standard-baseline",
        fmn_runtime::TuningSource::StandardAutotuneCache => "standard-autotune-cache",
    }
}

/// The distinct files one certified scene may read (`fmn_python.effect_audit`).
const MAX_SCENE_READS: usize = 4_096;

/// C6 items for the files a certified Python scene read, as recorded by
/// `fmn_python.effect_audit`: each is named `read/<sha256>/<basename>` and
/// bound to that digest.
fn scene_read_items(reads: Vec<(String, String)>) -> PyResult<Vec<ClosureItem>> {
    if reads.len() > MAX_SCENE_READS {
        return Err(PyValueError::new_err(
            "scene file reads exceed the 4096-input budget",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    reads
        .into_iter()
        .map(|(path, hex)| {
            let name = path
                .strip_prefix("read/")
                .and_then(|rest| rest.strip_prefix(hex.as_str()))
                .and_then(|rest| rest.strip_prefix('/'));
            let named = hex.len() == 64
                && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
                && name.is_some_and(|name| {
                    !name.is_empty() && !name.contains('/') && !name.chars().any(char::is_control)
                });
            if !named {
                return Err(PyValueError::new_err(format!(
                    "scene read {path:?} is not named read/<sha256>/<basename>"
                )));
            }
            if !seen.insert(path.clone()) {
                return Err(PyValueError::new_err(format!(
                    "duplicate scene read {path:?}"
                )));
            }
            let digest =
                Digest::from_hex(&hex).map_err(|e| PyValueError::new_err(e.to_string()))?;
            ClosureItem::digest_input(6, path, digest, "file read by the scene")
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        })
        .collect()
}

pub(crate) fn adjacent_manifest_destination(artifact: &Path) -> Result<PathBuf, String> {
    let leaf = artifact.file_name().ok_or_else(|| {
        format!(
            "render artifact {} has no leaf name for its provenance sidecar",
            artifact.display()
        )
    })?;
    let mut sidecar_leaf = leaf.to_os_string();
    sidecar_leaf.push(".manifest");
    Ok(artifact.with_file_name(sidecar_leaf))
}

pub(crate) fn preflight_manifest_generation(
    fs: &dyn FileSystem,
    destination: &Path,
) -> Result<(), String> {
    if fs
        .node_kind_no_follow(destination)
        .map_err(|error| {
            format!(
                "could not inspect manifest destination {}: {error}",
                destination.display()
            )
        })?
        .is_some()
    {
        return Err(format!(
            "manifest destination {} already exists; sidecars are no-clobber generations",
            destination.display()
        ));
    }
    Ok(())
}

pub(crate) fn publish_manifest_generation(
    fs: &Arc<dyn FileSystem>,
    destination: &Path,
    manifest: &ProvenanceManifest,
) -> Result<PathBuf, String> {
    let binary = manifest
        .to_bytes()
        .map_err(|error| format!("serialize FMNP manifest: {error}"))?;
    let text = manifest.to_text();
    let mut writer = Arc::clone(fs)
        .begin_atomic_directory(destination)
        .map_err(|error| {
            format!(
                "could not stage manifest generation {}: {error}",
                destination.display()
            )
        })?;
    writer
        .write_file(Path::new("manifest.fmnp"), &binary)
        .and_then(|()| writer.write_file(Path::new("manifest.txt"), text.as_bytes()))
        .map_err(|error| {
            format!(
                "could not write manifest generation {}: {error}",
                destination.display()
            )
        })?;
    writer
        .prepare()
        .and_then(|prepared| prepared.commit())
        .map_err(|error| {
            format!(
                "could not publish manifest generation {}: {error}",
                destination.display()
            )
        })?;
    Ok(destination.join("manifest.fmnp"))
}

/// The configuration a certified portal render resolves, as closure item C4
/// states it: the defaults overlaid with the render's own settings, exactly as
/// the native CLI overlays its flags (`resolve_render_config`).
///
/// `Config::canonical_bytes` serializes the merged configuration document, so
/// typed fields assigned after resolution never reach C4: before fm-arog, every
/// certified portal render published the defaults document, and two `fps`
/// values produced different frames under one closure digest.
/// `render.threads` is recorded but canonicalized to its inert marker, so the
/// scheduler width leaves the digest unchanged.
///
/// Run as a preflight before rendering and again at publication. A seed the
/// configuration cannot state (its integers are signed 64-bit) is refused
/// before any frame exists.
pub(crate) fn certified_render_config(
    resolution: (u32, u32),
    fps: u32,
    threads: usize,
    seed: u64,
) -> PyResult<fmn_config::Config> {
    use fmn_config::Value;
    let (width, height) = resolution;
    let seed = i64::try_from(seed).map_err(|_| {
        CapabilityError::new_err(format!(
            "CAPABILITY: certified rendering states the seed in its resolved configuration \
             (closure item C4), whose integers are signed 64-bit; seed {seed} exceeds {}",
            i64::MAX
        ))
    })?;
    let threads = i64::try_from(threads)
        .map_err(|_| PyValueError::new_err("render thread count exceeds a 64-bit integer"))?;
    let overlay = fmn_config::config::overlay([
        ("camera.fps", Value::Int(i64::from(fps))),
        (
            "camera.resolution",
            Value::Str(format!("({width}, {height})")),
        ),
        ("determinism.mode", Value::Str("certified".to_owned())),
        ("determinism.seed", Value::Int(seed)),
        ("render.engine", Value::Str("cpu".to_owned())),
        ("render.threads", Value::Int(threads)),
    ]);
    Ok(fmn_config::Config::resolve(&[], Some(overlay))
        .map_err(|e| PyValueError::new_err(e.to_string()))?
        .config)
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (
    destination,
    format,
    resolution,
    fps,
    threads,
    seed,
    artifact_report,
    sources,
    runtime_identities,
    cue_assets = None,
    scene_reads = None,
))]
pub(crate) fn _portal_publish_manifest(
    _py: Python<'_>,
    destination: PathBuf,
    format: &str,
    resolution: (u32, u32),
    fps: u32,
    threads: usize,
    seed: u64,
    artifact_report: &Bound<'_, PyDict>,
    sources: &Bound<'_, PyDict>,
    runtime_identities: &Bound<'_, PyDict>,
    cue_assets: Option<Vec<(String, Vec<u8>)>>,
    scene_reads: Option<Vec<(String, String)>>,
) -> PyResult<(String, String)> {
    if !matches!(format, "png" | "png_sequence" | "wav") {
        return Err(CapabilityError::new_err(format!(
            "CAPABILITY: certified reproducibility excludes format {format:?}; use --format png, png_sequence, or wav"
        )));
    }
    if cfg!(target_os = "windows") {
        return Err(CapabilityError::new_err(
            "CAPABILITY: windows-x86-64 is excluded from certified reproducibility by ADR-0019",
        ));
    }
    certified_build_check()?;

    let path_str: String = artifact_report
        .get_item("path")?
        .ok_or_else(|| PyValueError::new_err("missing artifact path"))?
        .extract()?;
    let artifact_path = PathBuf::from(path_str);
    let digest_hex: String = artifact_report
        .get_item("digest")?
        .ok_or_else(|| PyValueError::new_err("missing artifact digest"))?
        .extract()?;
    let artifact_digest =
        Digest::from_hex(&digest_hex).map_err(|e| PyValueError::new_err(e.to_string()))?;
    // Which Lumen route rasterized the frames (fm-sq8.11): (retained planar,
    // camera) counts and why the first camera-route frame left the planar one.
    let routes: Option<(u64, u64, Option<String>)> = artifact_report
        .get_item("routes")?
        .map(|value| value.extract())
        .transpose()?;

    let artifact_kind = match format {
        "png" => "canonical_png",
        "png_sequence" => "canonical_png_sequence",
        "wav" => "wav",
        _ => return Err(CapabilityError::new_err("unsupported certified format")),
    };

    let mut source_items = Vec::new();
    let source_pairs = sources.items();
    let mut is_first = true;
    for pair in source_pairs {
        let (virtual_path, bytes): (String, Vec<u8>) = pair.extract()?;
        let detail = if is_first {
            is_first = false;
            "scene source"
        } else {
            "imported python module"
        };
        source_items.push(
            ClosureItem::byte_input(1, virtual_path, &bytes, detail)
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        );
    }
    if source_items.is_empty() {
        return Err(CapabilityError::new_err(
            "CAPABILITY: portal input closure cannot be established: no source files provided",
        ));
    }

    let cue_list = cue_assets.unwrap_or_default();
    let cue_count = cue_list.len();
    for (asset_path, asset_bytes) in cue_list {
        source_items.push(
            ClosureItem::byte_input(1, asset_path, &asset_bytes, "scene sound-cue asset")
                .map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        );
    }

    let build_item = ClosureItem::byte_input(
        2,
        "franken_manim.build",
        BUILD_ID.as_bytes(),
        "franken_manim git commit or release build id",
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let suite_item = ClosureItem::byte_input(
        2,
        "SUITE.lock",
        SUITE_LOCK_BYTES,
        "complete governed dependency and toolchain lock",
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let toolchain = pinned_toolchain().map_err(PyRuntimeError::new_err)?;
    let cpython: String = runtime_identities
        .get_item("cpython")?
        .ok_or_else(|| PyValueError::new_err("missing cpython runtime identity"))?
        .extract()?;
    let abi: String = runtime_identities
        .get_item("abi")?
        .ok_or_else(|| PyValueError::new_err("missing abi runtime identity"))?
        .extract()?;
    let wheel: String = runtime_identities
        .get_item("wheel")?
        .ok_or_else(|| PyValueError::new_err("missing wheel runtime identity"))?
        .extract()?;
    let numpy: String = runtime_identities
        .get_item("numpy")?
        .ok_or_else(|| PyValueError::new_err("missing numpy runtime identity"))?
        .extract()?;

    // C3 in two parts, as the native CLI records it: the toolchain and the
    // Python runtime versions are platform-neutral; the target triple and
    // feature set are the `platform/` build record the semantic digest
    // leaves out (fm-certified-closure-integrity-4fei).
    let c3 = ClosureItem::structural(
        3,
        "native Rust toolchain; Python portal runtime",
        &[
            StructuralField::Text(toolchain),
            StructuralField::Text(CARGO_PROFILE),
            StructuralField::Text(&cpython),
            StructuralField::Text(&abi),
            StructuralField::Text(&wheel),
            StructuralField::Text(&numpy),
        ],
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let c3_platform = ClosureItem::structural_at(
        3,
        "platform/target",
        "compiled target triple and target-feature set",
        &[
            StructuralField::Text(TARGET_TRIPLE),
            StructuralField::Text(active_target_features()),
        ],
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let (width, height) = resolution;
    let config = certified_render_config(resolution, fps, threads, seed)?;
    let config_bytes = config
        .canonical_bytes()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let c4 = ClosureItem::byte_input(
        4,
        "resolved-config.fmnf",
        &config_bytes,
        "fully resolved configuration after defaults, files, and CLI overlay",
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let c5 = ClosureItem::structural(
        5,
        "PCG64DXSM root seed and named-substream layout",
        &[
            StructuralField::U64(seed),
            StructuralField::U64(u64::from(RNG_LAYOUT_VERSION)),
        ],
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    // Files the scene read (fmn_python.effect_audit): C6 inputs named by
    // content and basename, so equal inputs give equal closures on any host.
    let read_items = scene_read_items(scene_reads.unwrap_or_default())?;
    let read_count = read_items.len();

    // Fonts: every face compiled into the extension is a C6 byte input, as
    // on the native route (bundling is not an exemption).
    let bundled_faces = fmn_library::bundled_faces();
    let font_items = bundled_faces
        .iter()
        .map(|(name, bytes)| {
            ClosureItem::byte_input(
                6,
                format!("bundled-font/{name}"),
                bytes,
                "font face compiled into the extension",
            )
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
        })
        .collect::<PyResult<Vec<_>>>()?;
    let face_count = bundled_faces.len() as u64;
    let c6 = match (cue_count, read_count) {
        (0, 0) => ClosureItem::structural(
            6,
            "no asset reads on the portal route; bundled fonts listed",
            &[
                StructuralField::Absent("asset reads"),
                StructuralField::U64(face_count),
            ],
        ),
        (_, 0) => ClosureItem::structural(
            6,
            "scene sound-cue asset reads on the portal route; bundled fonts listed",
            &[
                StructuralField::U64(cue_count as u64),
                StructuralField::U64(face_count),
            ],
        ),
        _ => ClosureItem::structural(
            6,
            "scene file and sound-cue asset reads on the portal route; bundled fonts listed",
            &[
                StructuralField::U64(cue_count as u64),
                StructuralField::U64(read_count as u64),
                StructuralField::U64(face_count),
            ],
        ),
    }
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let output_pixel_format = OutputPixelFormat::Rgba8;
    let req = PlanRequest::certified(
        RenderIntent::Offline,
        fmn_runtime::SurfaceSpec::lumen(width, height),
        output_pixel_format,
    )
    .with_max_cpu_threads(threads);
    let plan = ExecutionPlan::derive(
        req,
        &fmn_platform::topology::HardwareTopology::current(),
        None,
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let (engine_identity, route, renderer_document) = if format == "wav" {
        (EngineIdentity::certified(), "composition", Vec::new())
    } else {
        let identity = EngineIdentity::certified();
        let background = fmn_core::color::Srgb::from_hex(&config.camera.background_color)
            .map_err(|e| PyValueError::new_err(e.to_string()))?
            .to_linear(config.camera.background_opacity);
        let frame_config = FrameConfig::new(
            Viewport { width, height },
            ScreenMap {
                scale: f64::from(height) / config.sizes.frame_height,
                origin: [f64::from(width) / 2.0, f64::from(height) / 2.0],
                y_up: true,
            },
            background,
        )
        .with_aa_policy(config.render.aa);
        let tiling = Tiling {
            macro_tile: plan.macro_tile,
            fine_tile: plan.fine_tile,
        };
        let doc = fmn_render::engine::journal(identity, &frame_config, tiling);
        let route = match routes {
            Some((planar, 0, _)) if planar > 0 => "cpu-certified-lumen:retained-2d",
            Some((0, camera, _)) if camera > 0 => "cpu-certified-lumen:camera",
            Some((planar, camera, _)) if planar > 0 && camera > 0 => {
                "cpu-certified-lumen:retained-2d+camera"
            }
            _ => "cpu-certified-lumen",
        };
        (identity, route, doc)
    };
    let c7_detail = match &routes {
        Some((planar, camera, reason)) if format != "wav" => format!(
            "semantic renderer and execution backend: {route}, {planar} retained-2d and \
             {camera} camera frames{}",
            reason.as_deref().map_or_else(String::new, |reason| format!(
                " (first camera frame: {reason})"
            ))
        ),
        _ => "semantic renderer and execution backend".to_owned(),
    };

    let c7 = ClosureItem::structural(
        7,
        c7_detail,
        &[
            StructuralField::Text(&engine_identity.closure_string()),
            StructuralField::Text(route),
            StructuralField::Bytes(&renderer_document),
        ],
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let c8 = ClosureItem::structural(
        8,
        "engine-visible locale and timezone are fixed by owned parsers and rational time",
        &[StructuralField::Text("C"), StructuralField::Text("UTC")],
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let fs: Arc<dyn FileSystem> = Arc::new(StdFs);
    let process_mechanism = StdProcessRunner.mechanism();
    let c9 = ClosureItem::structural(
        9,
        "portal capability policy; no external tool invoked",
        &[
            StructuralField::Text(fs.identity()),
            StructuralField::Text(process_mechanism.identity()),
            StructuralField::U64(u64::from(process_mechanism.policy_version())),
            StructuralField::Absent("host clock on render path"),
            StructuralField::Absent("AssetFetcher"),
            StructuralField::Text("fmn-python portal"),
            StructuralField::Absent("ffmpeg invocation"),
        ],
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let single_frame = format == "png";
    let c10 = ClosureItem::structural(
        10,
        "determinism mode and declared execution configuration",
        &[
            StructuralField::Text("certified"),
            StructuralField::Text(plan_determinism_name(plan.determinism)),
            StructuralField::Text(plan_engine_name(plan.engine)),
            StructuralField::Text(plan_intent_name(plan.intent)),
            StructuralField::Text(plan_output_name(plan.output_format)),
            StructuralField::Text(artifact_kind),
            StructuralField::Text(plan_tuning_name(plan.tuning_source)),
            StructuralField::U64(u64::try_from(plan.frames_in_flight).unwrap_or(u64::MAX)),
            StructuralField::U64(u64::from(plan.fine_tile)),
            StructuralField::U64(u64::from(plan.macro_tile)),
            StructuralField::Bool(false),
            StructuralField::Bool(single_frame),
            StructuralField::Absent("start_at_play"),
            StructuralField::Absent("end_at_play"),
            StructuralField::Bool(false),
            StructuralField::Text(route),
            StructuralField::Bytes(&renderer_document),
        ],
    )
    .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let identity = ManifestIdentity {
        build_id: BUILD_ID.to_owned(),
        suite_lock_digest: suite_item.digest,
        toolchain: toolchain.to_owned(),
        target_triple: TARGET_TRIPLE.to_owned(),
        target_features: active_target_features().to_owned(),
        engine: engine_identity.closure_string(),
        simd_tier: active_compiled_tier().to_owned(),
        declared_config_digest: c10.digest,
    };

    let outputs = vec![ManifestOutput {
        // The artifact's name in its output directory, never a host path,
        // so manifests from two machines compare by name and digest.
        virtual_path: artifact_path.file_name().map_or_else(
            || artifact_path.to_string_lossy().into_owned(),
            |name| name.to_string_lossy().into_owned(),
        ),
        kind: artifact_kind.to_owned(),
        digest: artifact_digest,
        certified: true,
    }];

    let mut items = vec![
        build_item,
        suite_item,
        c3,
        c3_platform,
        c4,
        c5,
        c6,
        c7,
        c8,
        c9,
        c10,
    ];
    items.extend(source_items);
    items.extend(read_items);
    items.extend(font_items);

    let manifest = ProvenanceManifest::new(ManifestMode::Certified, items, identity, outputs, None)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let sidecar_dir =
        adjacent_manifest_destination(&destination).map_err(PyRuntimeError::new_err)?;
    preflight_manifest_generation(fs.as_ref(), &sidecar_dir).map_err(PyRuntimeError::new_err)?;
    let manifest_file = publish_manifest_generation(&fs, &sidecar_dir, &manifest)
        .map_err(PyRuntimeError::new_err)?;

    Ok((
        manifest_file.to_string_lossy().into_owned(),
        manifest.closure_digest.to_hex(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_toolchain_is_present() {
        let toolchain = pinned_toolchain().expect("pinned toolchain");
        assert!(!toolchain.is_empty());
    }

    #[test]
    fn active_compiled_tier_is_valid() {
        let tier = active_compiled_tier();
        assert!(["portable", "x86-64-v3", "x86-64-v4", "aarch64+neon"].contains(&tier));
    }

    #[test]
    fn adjacent_manifest_destination_is_sidecar() {
        let artifact = Path::new("/tmp/test/render.png");
        let sidecar = adjacent_manifest_destination(artifact).expect("adjacent manifest");
        assert_eq!(sidecar, PathBuf::from("/tmp/test/render.png.manifest"));
    }

    #[test]
    fn scene_reads_are_content_named_c6_items() {
        let data = fmn_hash::sha256(b"x,1\n");
        let hex = data.to_hex();
        let named = |name: &str| (format!("read/{hex}/{name}"), hex.clone());
        let items = scene_read_items(vec![named("data.csv"), named("copy.csv")]).expect("valid");
        assert_eq!(items.len(), 2);
        for item in &items {
            assert_eq!((item.item_id, item.digest), (6, data));
        }
        let other = fmn_hash::sha256(b"x,2\n").to_hex();
        for bad in [
            (format!("read/{other}/data.csv"), hex.clone()), // name disagrees with digest
            (format!("data/{hex}/data.csv"), hex.clone()),
            (format!("read/{hex}/"), hex.clone()),
            (format!("read/{hex}/a/b.csv"), hex.clone()),
            (format!("read/{hex}/a\nb"), hex.clone()),
            (
                format!("read/{}/data.csv", hex.to_uppercase()),
                hex.to_uppercase(),
            ),
        ] {
            assert!(scene_read_items(vec![bad.clone()]).is_err(), "{bad:?}");
        }
        assert!(scene_read_items(vec![named("data.csv"), named("data.csv")]).is_err());
        assert!(scene_read_items(vec![named("data.csv"); MAX_SCENE_READS + 1]).is_err());
    }

    fn c4_bytes(resolution: (u32, u32), fps: u32, threads: usize, seed: u64) -> Vec<u8> {
        certified_render_config(resolution, fps, threads, seed)
            .expect("statable certified configuration")
            .canonical_bytes()
            .expect("canonical configuration bytes")
    }

    #[test]
    fn certified_config_states_the_render_settings_fm_arog() {
        use fmn_config::config::{DeterminismMode, Engine, ThreadPolicy};
        let config = certified_render_config((64, 36), 20, 4, 7).expect("resolves");
        assert_eq!(config.camera.resolution, (64, 36));
        assert_eq!(config.camera.fps, 20);
        assert_eq!(config.determinism.seed, 7);
        assert_eq!(config.determinism.mode, DeterminismMode::Certified);
        assert_eq!(config.render.engine, Engine::Cpu);
        assert_eq!(config.render.threads, ThreadPolicy::Fixed(4));
        // The bytes C4 hashes move with every closure-relevant setting...
        let base = c4_bytes((64, 36), 10, 1, 0);
        assert_ne!(c4_bytes((64, 36), 20, 1, 0), base, "fps");
        assert_ne!(c4_bytes((96, 54), 10, 1, 0), base, "resolution");
        assert_ne!(c4_bytes((64, 36), 10, 1, 1), base, "seed");
        // ...and differ from the defaults document every render used to publish.
        let defaults = fmn_config::Config::resolve(&[], None)
            .expect("defaults")
            .config
            .canonical_bytes()
            .expect("defaults bytes");
        assert_ne!(base, defaults);
        // The scheduler width is recorded but proven inert.
        assert_eq!(c4_bytes((64, 36), 10, 4, 0), base, "threads");
        assert_eq!(c4_bytes((64, 36), 10, 96, 0), base, "threads");
    }

    #[test]
    fn certified_config_refuses_a_seed_it_cannot_state() {
        let largest = u64::try_from(i64::MAX).expect("positive");
        assert!(certified_render_config((64, 36), 10, 1, largest).is_ok());
        assert!(certified_render_config((64, 36), 10, 1, largest + 1).is_err());
        assert!(certified_render_config((64, 36), 10, 1, u64::MAX).is_err());
    }
}
