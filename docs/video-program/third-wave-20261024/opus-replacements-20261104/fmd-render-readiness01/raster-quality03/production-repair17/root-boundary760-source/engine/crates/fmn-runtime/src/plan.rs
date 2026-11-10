//! `HardwareTopology` → `ExecutionPlan`.
//!
//! The plan records scheduling choices, not semantic choices. In particular,
//! a certified plan always pins the renderer's declared 16 px fine tile and
//! 128 px macrotile. Thread counts and team placement may still vary because
//! §10.5 requires those choices to be bit-inert.

use fmn_platform::topology::{HardwareTopology, PerfClass, SimdTier};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const CERTIFIED_FINE_TILE: u32 = 16;
const CERTIFIED_MACRO_TILE: u32 = 128;
const DEFAULT_SCRATCH_PER_WORKER: usize = 256 * 1024;
const PIPELINE_MEMORY_FRACTION: usize = 8;

/// The two determinism contracts (§4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Determinism {
    /// Deterministic on one build/platform; measured tuning is allowed.
    Standard,
    /// Cross-platform certified artifacts; all bit-affecting knobs are pinned.
    Certified,
}

/// Why frames are being produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderIntent {
    /// Latency-sensitive Studio or interactive preview.
    Preview,
    /// Throughput-sensitive offline export.
    Offline,
}

/// The execution engine selected by the plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionEngine {
    /// The scalar-definition certified CPU engine.
    CertifiedCpu,
    /// A standard-mode CPU build tier.
    FastCpu,
    /// The standard-only Metal annex.
    Metal,
    /// The standard-only CUDA annex.
    Cuda,
}

impl ExecutionEngine {
    /// Whether this is an Accelerator Annex engine.
    #[must_use]
    pub const fn is_annex(self) -> bool {
        matches!(self, Self::Metal | Self::Cuda)
    }
}

/// Stable planning vocabulary corresponding one-for-one with
/// `fmn-frame::PixelFormat`.
///
/// `fmn-runtime` intentionally cannot depend on `fmn-frame` without reversing
/// the governed crate DAG. The higher-layer adapter performs the exhaustive
/// conversion; keeping this enum exhaustive makes drift a compile error there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputPixelFormat {
    /// Canonical 8-bit RGBA.
    Rgba8,
    /// Compatibility 8-bit BGRA.
    Bgra8,
    /// Linear-light binary16 RGBA working/output surface.
    Rgba16F,
    /// 8-bit 4:2:0 Y′CbCr.
    Nv12,
    /// 10-bit 4:2:0 Y′CbCr in 16-bit containers.
    P010,
}

impl OutputPixelFormat {
    /// Stable manifest/doctor name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Rgba8 => "rgba8",
            Self::Bgra8 => "bgra8",
            Self::Rgba16F => "rgba16f",
            Self::Nv12 => "nv12",
            Self::P010 => "p010le",
        }
    }

    fn frame_bytes(self, width: u32, height: u32) -> Option<usize> {
        let pixels = (width as usize).checked_mul(height as usize)?;
        match self {
            Self::Rgba8 | Self::Bgra8 => pixels.checked_mul(4),
            Self::Rgba16F => pixels.checked_mul(8),
            Self::Nv12 => pixels.checked_add(pixels / 2),
            Self::P010 => pixels.checked_add(pixels / 2)?.checked_mul(2),
        }
    }

    const fn requires_even_dimensions(self) -> bool {
        matches!(self, Self::Nv12 | Self::P010)
    }
}

/// Geometry and working-surface cost used to derive the RAM-bound queue depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceSpec {
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Bytes per pixel in the renderer-owned working surface.
    pub working_bytes_per_pixel: u8,
}

impl SurfaceSpec {
    /// A Lumen frame: binary16 RGBA internally.
    #[must_use]
    pub const fn lumen(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            working_bytes_per_pixel: 8,
        }
    }

    fn working_bytes(self) -> Option<usize> {
        (self.width as usize)
            .checked_mul(self.height as usize)?
            .checked_mul(self.working_bytes_per_pixel as usize)
    }
}

/// Inputs whose combination determines an [`ExecutionPlan`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanRequest {
    determinism: Determinism,
    intent: RenderIntent,
    engine: ExecutionEngine,
    output_format: OutputPixelFormat,
    surface: SurfaceSpec,
    max_frames_in_flight: Option<usize>,
    max_cpu_threads: Option<usize>,
}

impl PlanRequest {
    /// A standard CPU request.
    #[must_use]
    pub const fn standard(
        intent: RenderIntent,
        surface: SurfaceSpec,
        output_format: OutputPixelFormat,
    ) -> Self {
        Self {
            determinism: Determinism::Standard,
            intent,
            engine: ExecutionEngine::FastCpu,
            output_format,
            surface,
            max_frames_in_flight: None,
            max_cpu_threads: None,
        }
    }

    /// A certified CPU request.
    #[must_use]
    pub const fn certified(
        intent: RenderIntent,
        surface: SurfaceSpec,
        output_format: OutputPixelFormat,
    ) -> Self {
        Self {
            determinism: Determinism::Certified,
            intent,
            engine: ExecutionEngine::CertifiedCpu,
            output_format,
            surface,
            max_frames_in_flight: None,
            max_cpu_threads: None,
        }
    }

    /// Select an engine. Annex engines remain invalid under certified mode.
    #[must_use]
    pub const fn with_engine(mut self, engine: ExecutionEngine) -> Self {
        self.engine = engine;
        self
    }

    /// Bound queue depth below the intent/topology default.
    #[must_use]
    pub const fn with_max_frames_in_flight(mut self, limit: usize) -> Self {
        self.max_frames_in_flight = Some(limit);
        self
    }

    /// Bound the physical-core representatives assigned to render teams.
    #[must_use]
    pub const fn with_max_cpu_threads(mut self, limit: usize) -> Self {
        self.max_cpu_threads = Some(limit);
        self
    }
}

/// Stable identity for standard-mode autotune records.
///
/// This is not a security digest. It is a fail-closed cache discriminator
/// whose value covers every topology field the planner consumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TopologyFingerprint {
    value: u64,
}

impl TopologyFingerprint {
    /// Compute the cache identity.
    #[must_use]
    pub fn of(topology: &HardwareTopology) -> Self {
        let mut hash = StableHash::new();
        hash.put_bytes(b"fmn-topology-fingerprint-v1");
        hash.put_u32(topology.logical_cores());
        hash.put_u32(topology.physical_cores);
        hash.put_u32(topology.packages);
        hash.put_bytes(topology.simd_tier.name().as_bytes());
        hash.put_option_u64(topology.total_memory_bytes);
        for cpu in &topology.cpus {
            hash.put_u32(cpu.id);
            hash.put_u32(cpu.package_id);
            hash.put_u32(cpu.core_id);
            hash.put_option_u32(cpu.capacity);
            hash.put_option_u64(cpu.max_freq_khz);
            hash.put_u32(match cpu.class {
                PerfClass::Performance => 1,
                PerfClass::Efficiency => 0,
            });
        }
        hash.put_u32(u32::try_from(topology.l2_domains.len()).unwrap_or(u32::MAX));
        for domain in &topology.l2_domains {
            hash.put_u32(u32::from(domain.level));
            hash.put_option_u64(domain.size_bytes);
            hash.put_u32(u32::try_from(domain.cpus.len()).unwrap_or(u32::MAX));
            for &cpu in &domain.cpus {
                hash.put_u32(cpu);
            }
        }
        hash.put_u32(u32::try_from(topology.l3_domains.len()).unwrap_or(u32::MAX));
        for domain in &topology.l3_domains {
            hash.put_u32(u32::from(domain.level));
            hash.put_option_u64(domain.size_bytes);
            hash.put_u32(u32::try_from(domain.cpus.len()).unwrap_or(u32::MAX));
            for &cpu in &domain.cpus {
                hash.put_u32(cpu);
            }
        }
        hash.put_u32(u32::try_from(topology.numa_nodes.len()).unwrap_or(u32::MAX));
        for node in &topology.numa_nodes {
            hash.put_u32(node.id);
            hash.put_u32(u32::try_from(node.cpus.len()).unwrap_or(u32::MAX));
            for &cpu in &node.cpus {
                hash.put_u32(cpu);
            }
        }
        hash.put_u32(u32::try_from(topology.processor_groups.len()).unwrap_or(u32::MAX));
        for group in &topology.processor_groups {
            hash.put_u32(group.id);
            hash.put_u32(u32::try_from(group.cpus.len()).unwrap_or(u32::MAX));
            for &cpu in &group.cpus {
                hash.put_u32(cpu);
            }
        }
        Self {
            value: hash.finish(),
        }
    }

    /// Stable 64-bit cache key.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.value
    }
}

/// A measured standard-mode scheduling profile.
///
/// OQ-11 owns how these records are produced. The baseline planner does not
/// pretend its priors are measurements; it consumes a record only when the
/// complete topology fingerprint matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutotuneProfile {
    /// Hardware identity this measurement belongs to.
    pub topology: TopologyFingerprint,
    /// Target physical workers in each render team.
    pub threads_per_render_team: usize,
    /// Requested global frame-slot budget.
    pub frames_in_flight: usize,
    /// Standard-mode fine tile.
    pub fine_tile: u32,
    /// Standard-mode macrotile.
    pub macro_tile: u32,
    /// Per-worker scratch arena.
    pub scratch_bytes_per_worker: usize,
}

/// In-memory view of the standard-only autotune cache.
///
/// Persistence belongs to the platform/cache boundary. This type gives that
/// boundary a deterministic replacement and lookup contract without ambient
/// filesystem access in `fmn-runtime`.
#[derive(Debug, Clone, Default)]
pub struct AutotuneCache {
    records: Vec<AutotuneProfile>,
}

impl AutotuneCache {
    /// Insert or replace the record for one topology.
    ///
    /// # Errors
    /// [`PlanError::InvalidAutotune`] if a measured value is structurally
    /// impossible.
    pub fn insert(&mut self, profile: AutotuneProfile) -> Result<(), PlanError> {
        validate_profile(&profile)?;
        if let Some(existing) = self
            .records
            .iter_mut()
            .find(|record| record.topology == profile.topology)
        {
            *existing = profile;
        } else {
            self.records.push(profile);
        }
        Ok(())
    }

    fn get(&self, fingerprint: TopologyFingerprint) -> Option<AutotuneProfile> {
        self.records
            .iter()
            .copied()
            .find(|record| record.topology == fingerprint)
    }
}

/// Where the plan's numeric priors came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuningSource {
    /// The certified declared profile (tile dimensions pinned).
    CertifiedProfile,
    /// Deterministic baseline; no measured cache record existed.
    StandardBaseline,
    /// A fingerprint-matched standard-mode measurement.
    StandardAutotuneCache,
}

/// A worker team's role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamRole {
    /// Latency-oriented scene/update and callback work.
    Scene,
    /// One independently schedulable frame raster team.
    Render(usize),
    /// Color conversion and sink preparation.
    Output,
}

/// One locality-first work lane inside a team.
///
/// A tile scheduler consumes lanes in order and steals within a lane before
/// crossing L3/NUMA/group boundaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalityLane {
    /// Windows processor-group id, or the synthetic group id on other hosts.
    pub processor_group: u32,
    /// NUMA node, if introspection exposed one.
    pub numa_node: Option<u32>,
    /// Index into `HardwareTopology::l3_domains`, if known.
    pub l3_domain: Option<usize>,
    /// Physical-core representative CPU ids in owner order.
    pub cpu_ids: Vec<u32>,
}

/// Advisory placement and scratch ownership for one worker team.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamPlan {
    /// Team purpose.
    pub role: TeamRole,
    /// One logical CPU id per planned worker.
    pub cpu_ids: Vec<u32>,
    /// Local-first queue/steal order.
    pub locality_lanes: Vec<LocalityLane>,
    /// Per-worker scratch arena bytes.
    pub scratch_bytes_per_worker: usize,
    /// Whether these ids are also assigned to another team.
    pub shares_cores: bool,
}

impl TeamPlan {
    /// Planned worker count.
    #[must_use]
    pub fn threads(&self) -> usize {
        self.cpu_ids.len().max(1)
    }

    /// Total scratch reserved for this team.
    #[must_use]
    pub fn scratch_bytes(&self) -> usize {
        self.scratch_bytes_per_worker.saturating_mul(self.threads())
    }
}

/// The complete scheduler decision reported by `fmn doctor`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    /// Determinism contract.
    pub determinism: Determinism,
    /// Preview vs export.
    pub intent: RenderIntent,
    /// Selected CPU/annex engine.
    pub engine: ExecutionEngine,
    /// Real global frame-slot limit.
    pub frames_in_flight: usize,
    /// Scene/update workers.
    pub scene_team: TeamPlan,
    /// Independent per-frame raster teams.
    pub render_teams: Vec<TeamPlan>,
    /// Conversion/output workers.
    pub output_team: TeamPlan,
    /// Fine-tile edge in pixels.
    pub fine_tile: u32,
    /// Macrotile edge in pixels.
    pub macro_tile: u32,
    /// Supported build tier reported by topology.
    pub simd_tier: SimdTier,
    /// Negotiated output surface format.
    pub output_format: OutputPixelFormat,
    /// Estimated working + output + scratch bytes held at queue capacity.
    pub estimated_in_flight_bytes: usize,
    /// Cache identity used for standard autotuning.
    pub topology_fingerprint: TopologyFingerprint,
    /// Provenance of tuning values.
    pub tuning_source: TuningSource,
}

impl ExecutionPlan {
    /// Workers an encoder-bound output stage may fan across: every planned
    /// CPU thread, render teams and output team together. While output is
    /// the bottleneck the render teams wait on the bounded frame ring, so a
    /// heavy codec (canonical PNG) can use their cores. Codec bytes never
    /// depend on this count (fm-hyqz).
    #[must_use]
    pub fn encoder_threads(&self) -> usize {
        self.render_teams
            .iter()
            .map(TeamPlan::threads)
            .sum::<usize>()
            .saturating_add(self.output_team.threads())
    }

    /// Derive a plan. A cache is consulted only for standard mode.
    ///
    /// # Errors
    /// [`PlanError`] when the request or topology is structurally invalid.
    pub fn derive(
        request: PlanRequest,
        topology: &HardwareTopology,
        cache: Option<&AutotuneCache>,
    ) -> Result<Self, PlanError> {
        validate_request(request, topology)?;
        validate_topology(topology)?;

        let fingerprint = TopologyFingerprint::of(topology);
        let cached = if request.determinism == Determinism::Standard {
            cache.and_then(|cache| cache.get(fingerprint))
        } else {
            None
        };
        if let Some(profile) = cached.as_ref() {
            validate_profile(profile)?;
        }

        let tuning_source = match (request.determinism, cached) {
            (Determinism::Certified, _) => TuningSource::CertifiedProfile,
            (Determinism::Standard, Some(_)) => TuningSource::StandardAutotuneCache,
            (Determinism::Standard, None) => TuningSource::StandardBaseline,
        };
        let target_threads = cached.map_or_else(
            || baseline_threads_per_team(request, topology),
            |profile| profile.threads_per_render_team,
        );
        let scratch_per_worker = cached.map_or(DEFAULT_SCRATCH_PER_WORKER, |profile| {
            profile.scratch_bytes_per_worker
        });
        let (fine_tile, macro_tile) = match request.determinism {
            Determinism::Certified => (CERTIFIED_FINE_TILE, CERTIFIED_MACRO_TILE),
            Determinism::Standard => cached
                .map_or((CERTIFIED_FINE_TILE, CERTIFIED_MACRO_TILE), |profile| {
                    (profile.fine_tile, profile.macro_tile)
                }),
        };

        let mut representatives = physical_representatives(topology);
        if let Some(limit) = request.max_cpu_threads {
            representatives.truncate(limit.min(representatives.len()));
        }
        let mut render_teams = build_render_teams(
            topology,
            &representatives,
            target_threads,
            scratch_per_worker,
        );

        let requested_slots = cached.map_or_else(
            || baseline_frames_in_flight(request, render_teams.len()),
            |profile| profile.frames_in_flight,
        );
        // Tuning selects within the product's declared latency/throughput
        // envelope; a corrupt or stale record cannot turn preview into a
        // six-frame queue or let an annex retain an unbounded number of
        // surfaces. The memory and caller caps below may still lower this.
        let (minimum_slots, maximum_slots) = frame_slot_bounds(request);
        let requested_slots = requested_slots.clamp(minimum_slots, maximum_slots);
        let requested_slots = request
            .max_frames_in_flight
            .map_or(requested_slots, |limit| requested_slots.min(limit));
        let bytes_per_frame = estimated_frame_bytes(
            request.surface,
            request.output_format,
            render_teams
                .iter()
                .map(TeamPlan::scratch_bytes)
                .max()
                .unwrap_or(scratch_per_worker),
        )
        .ok_or(PlanError::SizeOverflow)?;
        let memory_slots = memory_slot_cap(topology.total_memory_bytes, bytes_per_frame);
        let frames_in_flight = requested_slots.min(memory_slots).max(1);

        // A team without a possible simultaneous frame only consumes cores and
        // scratch. Truncation preserves processor-group-local chunks.
        render_teams.truncate(frames_in_flight);
        if render_teams.is_empty() {
            render_teams = build_render_teams(
                topology,
                &representatives,
                representatives.len().max(1),
                scratch_per_worker,
            );
            render_teams.truncate(1);
        }

        let scene_cpu = representatives
            .first()
            .copied()
            .or_else(|| topology.cpus.first().map(|cpu| cpu.id))
            .ok_or(PlanError::EmptyTopology)?;
        let scene_team = TeamPlan {
            role: TeamRole::Scene,
            cpu_ids: vec![scene_cpu],
            locality_lanes: locality_lanes(topology, &[scene_cpu]),
            scratch_bytes_per_worker: scratch_per_worker,
            // Scene/update overlaps raster and therefore shares the advisory
            // representative; no safe portable pinning API is assumed.
            shares_cores: render_teams
                .iter()
                .any(|team| team.cpu_ids.contains(&scene_cpu)),
        };
        let output_team = build_output_team(
            topology,
            &representatives,
            &render_teams,
            scratch_per_worker,
        );
        let estimated_in_flight_bytes = bytes_per_frame
            .checked_mul(frames_in_flight)
            .ok_or(PlanError::SizeOverflow)?;

        Ok(Self {
            determinism: request.determinism,
            intent: request.intent,
            engine: request.engine,
            frames_in_flight,
            scene_team,
            render_teams,
            output_team,
            fine_tile,
            macro_tile,
            simd_tier: topology.simd_tier,
            output_format: request.output_format,
            estimated_in_flight_bytes,
            topology_fingerprint: fingerprint,
            tuning_source,
        })
    }
}

/// Plan derivation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// No CPU can host even the coordinator.
    EmptyTopology,
    /// A caller-provided bound was zero.
    ZeroLimit(&'static str),
    /// Frame dimensions or working sample size were zero.
    InvalidSurface,
    /// A 4:2:0 format was paired with odd dimensions.
    OddSubsampledDimensions,
    /// Certified mode selected a standard-only engine.
    EngineNotCertifiable,
    /// Processor groups overlap, omit a CPU, or exceed 64 CPUs.
    InvalidProcessorGroups,
    /// A measured cache record contains an impossible zero or tile relation.
    InvalidAutotune,
    /// Surface/scratch arithmetic overflowed `usize`.
    SizeOverflow,
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTopology => f.write_str("hardware topology contains no CPUs"),
            Self::ZeroLimit(name) => write!(f, "{name} must be at least one"),
            Self::InvalidSurface => {
                f.write_str("frame width, height, and sample size must be nonzero")
            }
            Self::OddSubsampledDimensions => {
                f.write_str("NV12/P010 planning requires even frame dimensions")
            }
            Self::EngineNotCertifiable => {
                f.write_str("certified mode requires the certified CPU engine")
            }
            Self::InvalidProcessorGroups => f.write_str(
                "processor groups must cover each CPU exactly once and contain at most 64 CPUs",
            ),
            Self::InvalidAutotune => {
                f.write_str("autotune record contains invalid scheduling values")
            }
            Self::SizeOverflow => f.write_str("frame-pipeline memory calculation overflowed"),
        }
    }
}

impl std::error::Error for PlanError {}

fn validate_request(request: PlanRequest, topology: &HardwareTopology) -> Result<(), PlanError> {
    if topology.cpus.is_empty() || topology.physical_cores == 0 {
        return Err(PlanError::EmptyTopology);
    }
    if request.surface.width == 0
        || request.surface.height == 0
        || request.surface.working_bytes_per_pixel == 0
    {
        return Err(PlanError::InvalidSurface);
    }
    if request.output_format.requires_even_dimensions()
        && (!request.surface.width.is_multiple_of(2) || !request.surface.height.is_multiple_of(2))
    {
        return Err(PlanError::OddSubsampledDimensions);
    }
    if request.max_frames_in_flight == Some(0) {
        return Err(PlanError::ZeroLimit("max_frames_in_flight"));
    }
    if request.max_cpu_threads == Some(0) {
        return Err(PlanError::ZeroLimit("max_cpu_threads"));
    }
    if request.determinism == Determinism::Certified
        && request.engine != ExecutionEngine::CertifiedCpu
    {
        return Err(PlanError::EngineNotCertifiable);
    }
    Ok(())
}

fn validate_topology(topology: &HardwareTopology) -> Result<(), PlanError> {
    let ids: BTreeSet<u32> = topology.cpus.iter().map(|cpu| cpu.id).collect();
    let mut grouped = BTreeSet::new();
    for group in &topology.processor_groups {
        if group.cpus.is_empty() || group.cpus.len() > 64 {
            return Err(PlanError::InvalidProcessorGroups);
        }
        for &cpu in &group.cpus {
            if !ids.contains(&cpu) || !grouped.insert(cpu) {
                return Err(PlanError::InvalidProcessorGroups);
            }
        }
    }
    if topology.processor_groups.is_empty() || grouped != ids {
        return Err(PlanError::InvalidProcessorGroups);
    }
    Ok(())
}

fn validate_profile(profile: &AutotuneProfile) -> Result<(), PlanError> {
    if profile.threads_per_render_team == 0
        || profile.frames_in_flight == 0
        || profile.fine_tile == 0
        || profile.macro_tile == 0
        || profile.scratch_bytes_per_worker == 0
        || profile.macro_tile < profile.fine_tile
        || !profile.macro_tile.is_multiple_of(profile.fine_tile)
    {
        return Err(PlanError::InvalidAutotune);
    }
    Ok(())
}

fn baseline_threads_per_team(request: PlanRequest, topology: &HardwareTopology) -> usize {
    if request.engine.is_annex() {
        return topology.physical_cores.clamp(1, 4) as usize;
    }
    match request.intent {
        RenderIntent::Preview => topology.physical_cores.clamp(1, 16) as usize,
        RenderIntent::Offline => topology.physical_cores.clamp(1, 32) as usize,
    }
}

fn baseline_frames_in_flight(request: PlanRequest, render_teams: usize) -> usize {
    if request.engine.is_annex() {
        return 2;
    }
    match request.intent {
        RenderIntent::Preview => 2,
        RenderIntent::Offline => render_teams.clamp(3, 6),
    }
}

fn frame_slot_bounds(request: PlanRequest) -> (usize, usize) {
    if request.engine.is_annex() {
        return (2, 4);
    }
    match request.intent {
        RenderIntent::Preview => (1, 2),
        RenderIntent::Offline => (3, 6),
    }
}

fn physical_representatives(topology: &HardwareTopology) -> Vec<u32> {
    let mut cores: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for cpu in &topology.cpus {
        cores
            .entry((cpu.package_id, cpu.core_id))
            .and_modify(|selected| {
                let old = topology
                    .cpus
                    .iter()
                    .find(|candidate| candidate.id == *selected);
                let old_rank = old.map_or(2, |candidate| perf_rank(candidate.class));
                let new_rank = perf_rank(cpu.class);
                if (new_rank, cpu.id) < (old_rank, *selected) {
                    *selected = cpu.id;
                }
            })
            .or_insert(cpu.id);
    }
    let mut ids: Vec<u32> = cores.into_values().collect();
    ids.sort_by_key(|&id| {
        let cpu = topology.cpus.iter().find(|candidate| candidate.id == id);
        (
            cpu.map_or(2, |cpu| perf_rank(cpu.class)),
            processor_group_index(topology, id).unwrap_or(usize::MAX),
            l3_index(topology, id).unwrap_or(usize::MAX),
            id,
        )
    });
    ids
}

const fn perf_rank(class: PerfClass) -> u8 {
    match class {
        PerfClass::Performance => 0,
        PerfClass::Efficiency => 1,
    }
}

fn build_render_teams(
    topology: &HardwareTopology,
    representatives: &[u32],
    target_threads: usize,
    scratch_per_worker: usize,
) -> Vec<TeamPlan> {
    let mut by_group: BTreeMap<usize, Vec<u32>> = BTreeMap::new();
    for &cpu in representatives {
        let group = processor_group_index(topology, cpu).unwrap_or(0);
        by_group.entry(group).or_default().push(cpu);
    }

    let mut teams = Vec::new();
    for cpus in by_group.into_values() {
        for chunk in cpus.chunks(target_threads.max(1)) {
            let index = teams.len();
            teams.push(TeamPlan {
                role: TeamRole::Render(index),
                cpu_ids: chunk.to_vec(),
                locality_lanes: locality_lanes(topology, chunk),
                scratch_bytes_per_worker: scratch_per_worker,
                shares_cores: false,
            });
        }
    }
    teams
}

fn build_output_team(
    topology: &HardwareTopology,
    representatives: &[u32],
    render_teams: &[TeamPlan],
    scratch_per_worker: usize,
) -> TeamPlan {
    let primary: BTreeSet<u32> = representatives.iter().copied().collect();
    let assigned: BTreeSet<u32> = render_teams
        .iter()
        .flat_map(|team| team.cpu_ids.iter().copied())
        .collect();
    let rendered_cores: BTreeSet<(u32, u32)> = render_teams
        .iter()
        .flat_map(|team| team.cpu_ids.iter())
        .filter_map(|id| {
            topology
                .cpus
                .iter()
                .find(|cpu| cpu.id == *id)
                .map(|cpu| (cpu.package_id, cpu.core_id))
        })
        .collect();
    let mut siblings: Vec<u32> = topology
        .cpus
        .iter()
        .filter(|cpu| {
            !primary.contains(&cpu.id) && rendered_cores.contains(&(cpu.package_id, cpu.core_id))
        })
        .map(|cpu| cpu.id)
        .collect();
    siblings.sort_unstable();
    siblings.truncate(2);

    if siblings.is_empty() {
        let fallback = topology
            .cpus
            .iter()
            .find(|cpu| cpu.class == PerfClass::Efficiency)
            .or_else(|| topology.cpus.first())
            .map(|cpu| cpu.id)
            .unwrap_or(0);
        siblings.push(fallback);
    }
    let shares_cores = siblings.iter().any(|cpu| assigned.contains(cpu));
    TeamPlan {
        role: TeamRole::Output,
        locality_lanes: locality_lanes(topology, &siblings),
        cpu_ids: siblings,
        scratch_bytes_per_worker: scratch_per_worker,
        shares_cores,
    }
}

fn locality_lanes(topology: &HardwareTopology, cpus: &[u32]) -> Vec<LocalityLane> {
    let mut grouped: BTreeMap<(u32, Option<u32>, Option<usize>), Vec<u32>> = BTreeMap::new();
    for &cpu in cpus {
        let group = processor_group_index(topology, cpu)
            .and_then(|index| topology.processor_groups.get(index))
            .map_or(0, |group| group.id);
        let numa = topology
            .numa_nodes
            .iter()
            .find(|node| node.cpus.contains(&cpu))
            .map(|node| node.id);
        let l3 = l3_index(topology, cpu);
        grouped.entry((group, numa, l3)).or_default().push(cpu);
    }
    grouped
        .into_iter()
        .map(
            |((processor_group, numa_node, l3_domain), cpu_ids)| LocalityLane {
                processor_group,
                numa_node,
                l3_domain,
                cpu_ids,
            },
        )
        .collect()
}

fn processor_group_index(topology: &HardwareTopology, cpu: u32) -> Option<usize> {
    topology
        .processor_groups
        .iter()
        .position(|group| group.cpus.contains(&cpu))
}

fn l3_index(topology: &HardwareTopology, cpu: u32) -> Option<usize> {
    topology
        .l3_domains
        .iter()
        .position(|domain| domain.cpus.contains(&cpu))
}

fn estimated_frame_bytes(
    surface: SurfaceSpec,
    output: OutputPixelFormat,
    scratch_per_team: usize,
) -> Option<usize> {
    surface
        .working_bytes()?
        .checked_add(output.frame_bytes(surface.width, surface.height)?)?
        .checked_add(scratch_per_team)
}

fn memory_slot_cap(total_memory: Option<u64>, bytes_per_frame: usize) -> usize {
    let Some(total) = total_memory else {
        return usize::MAX;
    };
    let total = usize::try_from(total).unwrap_or(usize::MAX);
    (total / PIPELINE_MEMORY_FRACTION / bytes_per_frame.max(1)).max(1)
}

struct StableHash(u64);

impl StableHash {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    const fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn put_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn put_u32(&mut self, value: u32) {
        self.put_bytes(&value.to_le_bytes());
    }

    fn put_u64(&mut self, value: u64) {
        self.put_bytes(&value.to_le_bytes());
    }

    fn put_option_u32(&mut self, value: Option<u32>) {
        self.put_bytes(&[u8::from(value.is_some())]);
        if let Some(value) = value {
            self.put_u32(value);
        }
    }

    fn put_option_u64(&mut self, value: Option<u64>) {
        self.put_bytes(&[u8::from(value.is_some())]);
        if let Some(value) = value {
            self.put_u64(value);
        }
    }

    const fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmn_platform::topology::{CacheDomain, LogicalCpu};

    fn surface() -> SurfaceSpec {
        SurfaceSpec::lumen(1920, 1080)
    }

    fn offline() -> PlanRequest {
        PlanRequest::standard(RenderIntent::Offline, surface(), OutputPixelFormat::Rgba8)
    }

    fn topology_96() -> HardwareTopology {
        let mut topology = HardwareTopology::from_group_sizes(&[64, 32]).expect("fixture");
        topology.simd_tier = SimdTier::X86_64V3;
        topology.total_memory_bytes = Some(128 * 1024 * 1024 * 1024);
        topology.l3_domains = (0..12)
            .map(|index| CacheDomain {
                level: 3,
                size_bytes: Some(32 * 1024 * 1024),
                cpus: (index * 8..index * 8 + 8).collect(),
            })
            .collect();
        topology
    }

    fn apple_4p4e() -> HardwareTopology {
        let mut topology = HardwareTopology::fallback(8);
        for cpu in &mut topology.cpus {
            cpu.class = if cpu.id < 4 {
                PerfClass::Efficiency
            } else {
                PerfClass::Performance
            };
            cpu.capacity = Some(if cpu.id < 4 { 512 } else { 1024 });
        }
        topology.simd_tier = SimdTier::Aarch64Neon;
        topology.total_memory_bytes = Some(8 * 1024 * 1024 * 1024);
        topology
    }

    #[test]
    fn eight_core_laptop_gets_bounded_offline_pipeline() {
        let mut topology = HardwareTopology::fallback(8);
        topology.total_memory_bytes = Some(16 * 1024 * 1024 * 1024);
        let plan = ExecutionPlan::derive(offline(), &topology, None).expect("plan");
        assert_eq!(plan.frames_in_flight, 3);
        assert_eq!(plan.render_teams.len(), 1);
        assert_eq!(plan.render_teams[0].threads(), 8);
        assert_eq!(plan.fine_tile, 16);
        assert_eq!(plan.macro_tile, 128);
        assert!(plan.estimated_in_flight_bytes > 0);
    }

    #[test]
    fn ninety_six_cores_form_three_group_local_teams() {
        let topology = topology_96();
        let plan = ExecutionPlan::derive(offline(), &topology, None).expect("plan");
        assert_eq!(plan.frames_in_flight, 3);
        assert_eq!(plan.render_teams.len(), 3);
        assert!(plan.render_teams.iter().all(|team| team.threads() == 32));
        for team in &plan.render_teams {
            let groups: BTreeSet<_> = team
                .cpu_ids
                .iter()
                .map(|&cpu| processor_group_index(&topology, cpu).expect("group"))
                .collect();
            assert_eq!(groups.len(), 1, "a team crossed a processor group");
        }
        assert_eq!(
            plan.render_teams
                .iter()
                .flat_map(|team| team.cpu_ids.iter())
                .collect::<BTreeSet<_>>()
                .len(),
            96
        );
    }

    /// fm-hyqz: an encoder-bound output stage may use every planned CPU
    /// thread, not only the two-thread output team.
    #[test]
    fn encoder_threads_count_every_planned_cpu_thread() {
        let plan = ExecutionPlan::derive(offline(), &topology_96(), None).expect("plan");
        assert_eq!(plan.encoder_threads(), 96 + plan.output_team.threads());
        assert!(plan.encoder_threads() > plan.output_team.threads());
    }

    #[test]
    fn windows_groups_above_sixty_four_are_never_crossed() {
        let topology = HardwareTopology::from_group_sizes(&[64, 32]).expect("fixture");
        let plan = ExecutionPlan::derive(offline(), &topology, None).expect("plan");
        assert_eq!(plan.render_teams.len(), 3);
        assert!(plan.render_teams.iter().all(|team| {
            team.locality_lanes
                .iter()
                .map(|lane| lane.processor_group)
                .collect::<BTreeSet<_>>()
                .len()
                == 1
        }));
    }

    #[test]
    fn apple_scene_and_render_prioritize_performance_cores() {
        let topology = apple_4p4e();
        let request =
            PlanRequest::standard(RenderIntent::Preview, surface(), OutputPixelFormat::Rgba8);
        let plan = ExecutionPlan::derive(request, &topology, None).expect("plan");
        assert!(plan.scene_team.cpu_ids[0] >= 4);
        assert_eq!(&plan.render_teams[0].cpu_ids[..4], &[4, 5, 6, 7]);
        assert_eq!(plan.simd_tier, SimdTier::Aarch64Neon);
        assert!(plan.frames_in_flight <= 2);
    }

    #[test]
    fn unassigned_efficiency_output_cpu_is_not_reported_as_shared() {
        let topology = apple_4p4e();
        let request =
            PlanRequest::standard(RenderIntent::Preview, surface(), OutputPixelFormat::Rgba8)
                .with_max_cpu_threads(4);
        let plan = ExecutionPlan::derive(request, &topology, None).expect("plan");
        assert_eq!(plan.render_teams[0].cpu_ids, vec![4, 5, 6, 7]);
        assert_eq!(plan.output_team.cpu_ids, vec![0]);
        assert!(!plan.output_team.shares_cores);
    }

    #[test]
    fn smt_siblings_are_reserved_for_output() {
        let mut topology = HardwareTopology::fallback(8);
        topology.cpus = (0..16)
            .map(|id| LogicalCpu {
                id,
                package_id: 0,
                core_id: id % 8,
                capacity: None,
                max_freq_khz: None,
                class: PerfClass::Performance,
            })
            .collect();
        topology.physical_cores = 8;
        topology.processor_groups = HardwareTopology::from_group_sizes(&[16])
            .expect("groups")
            .processor_groups;
        let plan = ExecutionPlan::derive(offline(), &topology, None).expect("plan");
        assert!(!plan.output_team.shares_cores);
        assert!(plan.output_team.cpu_ids.iter().all(|id| *id >= 8));
    }

    #[test]
    fn cache_is_standard_only_and_fingerprint_matched() {
        let topology = topology_96();
        let fingerprint = TopologyFingerprint::of(&topology);
        let mut cache = AutotuneCache::default();
        cache
            .insert(AutotuneProfile {
                topology: fingerprint,
                threads_per_render_team: 24,
                frames_in_flight: 4,
                fine_tile: 8,
                macro_tile: 64,
                scratch_bytes_per_worker: 128 * 1024,
            })
            .expect("valid");

        let standard = ExecutionPlan::derive(offline(), &topology, Some(&cache)).expect("plan");
        assert_eq!(standard.tuning_source, TuningSource::StandardAutotuneCache);
        assert_eq!(standard.fine_tile, 8);
        assert_eq!(standard.macro_tile, 64);
        assert_eq!(standard.render_teams.len(), 4);
        assert_eq!(standard.frames_in_flight, 4);

        let certified = ExecutionPlan::derive(
            PlanRequest::certified(RenderIntent::Offline, surface(), OutputPixelFormat::Rgba8),
            &topology,
            Some(&cache),
        )
        .expect("plan");
        assert_eq!(certified.tuning_source, TuningSource::CertifiedProfile);
        assert_eq!(certified.fine_tile, CERTIFIED_FINE_TILE);
        assert_eq!(certified.macro_tile, CERTIFIED_MACRO_TILE);
    }

    #[test]
    fn cached_queue_depth_stays_inside_each_mode_envelope() {
        let topology = topology_96();
        let mut cache = AutotuneCache::default();
        cache
            .insert(AutotuneProfile {
                topology: TopologyFingerprint::of(&topology),
                threads_per_render_team: 8,
                frames_in_flight: 99,
                fine_tile: 16,
                macro_tile: 128,
                scratch_bytes_per_worker: 64 * 1024,
            })
            .expect("structurally valid profile");

        let preview = ExecutionPlan::derive(
            PlanRequest::standard(RenderIntent::Preview, surface(), OutputPixelFormat::Rgba8),
            &topology,
            Some(&cache),
        )
        .expect("preview plan");
        assert_eq!(preview.frames_in_flight, 2);

        let offline =
            ExecutionPlan::derive(offline(), &topology, Some(&cache)).expect("offline plan");
        assert_eq!(offline.frames_in_flight, 6);

        let annex = ExecutionPlan::derive(
            PlanRequest::standard(RenderIntent::Offline, surface(), OutputPixelFormat::Rgba8)
                .with_engine(ExecutionEngine::Metal),
            &topology,
            Some(&cache),
        )
        .expect("annex plan");
        assert_eq!(annex.frames_in_flight, 4);
    }

    #[test]
    fn memory_budget_reduces_queue_depth_but_never_to_zero() {
        let mut topology = topology_96();
        topology.total_memory_bytes = Some(64 * 1024 * 1024);
        let plan = ExecutionPlan::derive(offline(), &topology, None).expect("plan");
        assert_eq!(plan.frames_in_flight, 1);
        assert_eq!(plan.render_teams.len(), 1);
    }

    #[test]
    fn certified_refuses_standard_only_engines() {
        let topology = HardwareTopology::fallback(8);
        let request =
            PlanRequest::certified(RenderIntent::Offline, surface(), OutputPixelFormat::Rgba8)
                .with_engine(ExecutionEngine::Metal);
        assert_eq!(
            ExecutionPlan::derive(request, &topology, None),
            Err(PlanError::EngineNotCertifiable)
        );
    }

    #[test]
    fn yuv_surfaces_must_be_even() {
        let topology = HardwareTopology::fallback(8);
        let request = PlanRequest::standard(
            RenderIntent::Offline,
            SurfaceSpec::lumen(1919, 1080),
            OutputPixelFormat::Nv12,
        );
        assert_eq!(
            ExecutionPlan::derive(request, &topology, None),
            Err(PlanError::OddSubsampledDimensions)
        );
    }

    #[test]
    fn fingerprint_moves_with_consumed_topology_fields() {
        let a = HardwareTopology::fallback(8);
        let mut b = a.clone();
        b.cpus[0].class = PerfClass::Efficiency;
        assert_ne!(TopologyFingerprint::of(&a), TopologyFingerprint::of(&b));
    }

    #[test]
    fn fingerprint_distinguishes_absent_and_zero_optional_values() {
        let a = HardwareTopology::fallback(8);
        let mut b = a.clone();
        b.total_memory_bytes = Some(0);
        assert_ne!(TopologyFingerprint::of(&a), TopologyFingerprint::of(&b));

        let mut c = a.clone();
        c.cpus[0].capacity = Some(0);
        assert_ne!(TopologyFingerprint::of(&a), TopologyFingerprint::of(&c));
    }

    #[test]
    fn invalid_autotune_is_rejected_at_cache_boundary() {
        let mut cache = AutotuneCache::default();
        assert_eq!(
            cache.insert(AutotuneProfile {
                topology: TopologyFingerprint { value: 1 },
                threads_per_render_team: 0,
                frames_in_flight: 1,
                fine_tile: 16,
                macro_tile: 128,
                scratch_bytes_per_worker: 1,
            }),
            Err(PlanError::InvalidAutotune)
        );
    }
}
