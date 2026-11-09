//! `HardwareTopology` introspection (§17.4, Rev 4): the machine shape that
//! fmn-runtime's `ExecutionPlan` derivation consumes.
//!
//! Behavior keys on **introspected topology, never marketing names** (§17.6).
//! What the planner needs to know, this struct carries: logical/physical
//! cores, packages, SMT siblings, performance classes (P/E), L2/L3 cache
//! domains (the CCD sharding §17.6's playbooks are built on), NUMA nodes,
//! Windows processor groups, the SIMD build-tier the CPU supports, and
//! available memory.
//!
//! Introspection obeys the capability doctrine: Linux detection reads sysfs
//! **through** [`FileSystem`], so a synthetic machine is a fixture handed to
//! [`HardwareTopology::detect_linux`] — that is how the aarch64 big.LITTLE
//! and Windows processor-group shapes are tested on any host.
//!
//! **macOS** detection reads the `hw.*` sysctl MIB **through**
//! [`SysctlSource`]: [`HardwareTopology::detect_macos`] maps Apple's
//! performance levels (`hw.perflevel{N}`) to P/E classes and their
//! `cpusperl2` widths to L2 cluster domains, and a recorded `sysctl(8)`
//! listing ([`SysctlSnapshot`]) is its fixture on any host. A *live* source
//! needs `sysctlbyname(3)`, which is FFI. D3 forbids authoring that here, and
//! no governed dependency exposes it safely yet, so [`HardwareTopology::current`]
//! still uses the flat fallback on macOS until a governed suite crate supplies
//! the reader.
//!
//! **Windows processor groups (§17.4):** systems above 64 logical processors
//! span groups, and explicit scheduling code must know it. The model here is
//! groups of at most 64 logical CPUs; [`HardwareTopology::from_group_sizes`]
//! builds synthetic groupings and [`windows_group_split`] is the documented
//! split rule. Native Windows introspection is not implemented (it needs
//! Win32 calls outside the governed closure's current surface); Windows
//! builds use [`HardwareTopology::fallback`], which already applies the
//! group split — so no scheduling consumer can be written that ignores
//! groups. R18's functional CI keeps this honest.

use crate::fs::{FileSystem, FsError};
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// The SIMD build tiers (§17.3, capped by R22 at exactly these four).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SimdTier {
    /// Portable scalar/std::simd baseline — always available.
    Portable,
    /// x86-64-v3: AVX2 + FMA + BMI2 class.
    X86_64V3,
    /// x86-64-v4: AVX-512 (F/BW/VL/DQ) class.
    X86_64V4,
    /// aarch64 with NEON (the aarch64 baseline).
    Aarch64Neon,
}

impl SimdTier {
    /// The tier's canonical name (matches SUITE.lock / release artifact
    /// naming).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Portable => "portable",
            Self::X86_64V3 => "x86-64-v3",
            Self::X86_64V4 => "x86-64-v4",
            Self::Aarch64Neon => "aarch64-neon",
        }
    }
}

/// Performance class of a logical CPU (P/E cores).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PerfClass {
    /// Performance core (or any core on a uniform machine).
    Performance,
    /// Efficiency core.
    Efficiency,
}

/// One logical CPU.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LogicalCpu {
    /// Kernel CPU id.
    pub id: u32,
    /// Physical package (socket) id.
    pub package_id: u32,
    /// Core id within the package; logical CPUs sharing `(package_id,
    /// core_id)` are SMT siblings.
    pub core_id: u32,
    /// Scheduler capacity (arm `cpu_capacity`), if exposed.
    pub capacity: Option<u32>,
    /// Maximum frequency in kHz, if exposed.
    pub max_freq_khz: Option<u64>,
    /// Derived performance class.
    pub class: PerfClass,
}

/// A cache domain: one cache of `level` shared by `cpus`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CacheDomain {
    /// Cache level (2 or 3 are recorded).
    pub level: u8,
    /// Size in bytes, if exposed.
    pub size_bytes: Option<u64>,
    /// The logical CPUs sharing this cache, ascending.
    pub cpus: Vec<u32>,
}

/// A NUMA node and its CPUs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NumaNode {
    /// Node id.
    pub id: u32,
    /// The node's logical CPUs, ascending.
    pub cpus: Vec<u32>,
}

/// A processor group (Windows scheduling domain; one group elsewhere).
/// Invariant: at most 64 logical CPUs per group.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProcessorGroup {
    /// Group id.
    pub id: u32,
    /// The group's logical CPUs, ascending.
    pub cpus: Vec<u32>,
}

/// The introspected machine shape (§17.4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct HardwareTopology {
    /// Every online logical CPU, ascending by id.
    pub cpus: Vec<LogicalCpu>,
    /// Distinct physical cores (unique `(package, core)` pairs).
    pub physical_cores: u32,
    /// Distinct physical packages (sockets).
    pub packages: u32,
    /// L2 cache domains (E-core clusters share L2; SMT siblings share L2 on
    /// x86).
    pub l2_domains: Vec<CacheDomain>,
    /// L3 cache domains — the CCD boundaries §17.6's sharding keys on.
    pub l3_domains: Vec<CacheDomain>,
    /// NUMA nodes (one node on a desktop part).
    pub numa_nodes: Vec<NumaNode>,
    /// Processor groups (≤ 64 CPUs each; exactly one group on machines that
    /// fit, per the Windows model above).
    pub processor_groups: Vec<ProcessorGroup>,
    /// The SIMD tier this machine supports.
    pub simd_tier: SimdTier,
    /// Total physical memory in bytes, if known.
    pub total_memory_bytes: Option<u64>,
}

/// A topology introspection failure.
#[derive(Debug)]
pub enum TopologyError {
    /// A required sysfs/procfs file is missing.
    Missing {
        /// The missing path.
        path: PathBuf,
    },
    /// A file existed but did not parse.
    Parse {
        /// The offending path.
        path: PathBuf,
        /// What was wrong.
        detail: String,
    },
    /// A structural invariant failed (e.g. a processor group above 64).
    Invalid {
        /// What was wrong.
        detail: String,
    },
    /// A required `sysctl` OID is absent.
    MissingSysctl {
        /// The missing OID name.
        name: String,
    },
    /// A `sysctl` OID existed but its value did not parse.
    SysctlParse {
        /// The offending OID name.
        name: String,
        /// What was wrong.
        detail: String,
    },
}

impl fmt::Display for TopologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { path } => write!(f, "topology file missing: {}", path.display()),
            Self::Parse { path, detail } => {
                write!(f, "topology parse failure at {}: {detail}", path.display())
            }
            Self::Invalid { detail } => write!(f, "invalid topology: {detail}"),
            Self::MissingSysctl { name } => write!(f, "topology sysctl missing: {name}"),
            Self::SysctlParse { name, detail } => {
                write!(f, "topology sysctl parse failure at {name}: {detail}")
            }
        }
    }
}

impl std::error::Error for TopologyError {}

/// Allocation and iteration ceiling for one kernel CPU-list document.
///
/// This is intentionally far above the scheduler profiles the project
/// targets while keeping a compact range from expanding across the complete
/// `u32` domain.
const MAX_CPU_LIST_ENTRIES: usize = 16 * 1024;
const MAX_INVALID_LIST_PREVIEW_BYTES: usize = 128;

/// Parse a kernel CPU list (`"0-7,64-71"`) into ascending ids.
///
/// # Errors
/// [`TopologyError::Invalid`] on malformed input.
pub fn parse_cpu_list(s: &str) -> Result<Vec<u32>, TopologyError> {
    let s = s.trim();
    let mut out = Vec::new();
    if s.is_empty() {
        return Ok(out);
    }
    for part in s.split(',') {
        let part = part.trim();
        if let Some((a, b)) = part.split_once('-') {
            let a: u32 = a.parse().map_err(|_| invalid_list(s))?;
            let b: u32 = b.parse().map_err(|_| invalid_list(s))?;
            if a > b {
                return Err(invalid_list(s));
            }
            let additional = u64::from(b) - u64::from(a) + 1;
            reserve_cpu_list_entries(&mut out, additional)?;
            out.extend(a..=b);
        } else {
            reserve_cpu_list_entries(&mut out, 1)?;
            out.push(part.parse().map_err(|_| invalid_list(s))?);
        }
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

fn reserve_cpu_list_entries(out: &mut Vec<u32>, additional: u64) -> Result<(), TopologyError> {
    let additional = usize::try_from(additional).map_err(|_| cpu_list_limit())?;
    let needed = out
        .len()
        .checked_add(additional)
        .ok_or_else(cpu_list_limit)?;
    if needed > MAX_CPU_LIST_ENTRIES {
        return Err(cpu_list_limit());
    }
    out.reserve(additional);
    Ok(())
}

fn cpu_list_limit() -> TopologyError {
    TopologyError::Invalid {
        detail: format!("CPU list expands beyond the {MAX_CPU_LIST_ENTRIES}-entry limit"),
    }
}

fn invalid_list(s: &str) -> TopologyError {
    let detail = if s.len() <= MAX_INVALID_LIST_PREVIEW_BYTES {
        format!("malformed CPU list {s:?}")
    } else {
        format!("malformed CPU list ({} input bytes)", s.len())
    };
    TopologyError::Invalid { detail }
}

/// Render ascending CPU ids as a compact kernel-style list (`"0-7,64-71"`),
/// the inverse of [`parse_cpu_list`]. Used by the snapshot format.
#[must_use]
pub fn format_cpu_list(cpus: &[u32]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < cpus.len() {
        let start = cpus[i];
        let mut end = start;
        while i + 1 < cpus.len() && end.checked_add(1) == Some(cpus[i + 1]) {
            i += 1;
            end = cpus[i];
        }
        if start == end {
            parts.push(start.to_string());
        } else {
            parts.push(format!("{start}-{end}"));
        }
        i += 1;
    }
    parts.join(",")
}

/// The documented Windows split of `logical` processors into groups: full
/// groups of 64, then the remainder. (Real Windows assigns by NUMA
/// proximity; the invariant scheduling code must honor — no group above 64,
/// possibly several groups — is identical, and that is what consumers key
/// on.)
#[must_use]
pub fn windows_group_split(logical: u32) -> Vec<u32> {
    let mut sizes = Vec::new();
    let mut left = logical;
    while left > 64 {
        sizes.push(64);
        left -= 64;
    }
    if left > 0 || sizes.is_empty() {
        sizes.push(left);
    }
    sizes
}

/// Detect the SIMD tier of the running CPU. Safe feature detection via
/// `std::arch`; dispatch stays by build tier (§17.3) — this value reports
/// what the machine *could* run, for `fmn doctor` and artifact selection.
#[must_use]
pub fn detect_simd_tier() -> SimdTier {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx512f")
            && std::arch::is_x86_feature_detected!("avx512bw")
            && std::arch::is_x86_feature_detected!("avx512vl")
            && std::arch::is_x86_feature_detected!("avx512dq")
        {
            SimdTier::X86_64V4
        } else if std::arch::is_x86_feature_detected!("avx2")
            && std::arch::is_x86_feature_detected!("fma")
            && std::arch::is_x86_feature_detected!("bmi2")
        {
            SimdTier::X86_64V3
        } else {
            SimdTier::Portable
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // NEON is architecturally mandatory on aarch64.
        SimdTier::Aarch64Neon
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        SimdTier::Portable
    }
}

/// The macOS `sysctl` MIB that [`HardwareTopology::detect_macos`] reads: the
/// Darwin counterpart of the sysfs files [`HardwareTopology::detect_linux`]
/// reads through [`FileSystem`].
///
/// Detection is a pure function of this capability, so a recorded machine is
/// a fixture on any host. A live implementation calls `sysctlbyname(3)`. That
/// is FFI, which D3 keeps out of this crate.
pub trait SysctlSource {
    /// The value of `name` exactly as `sysctl(8)` prints it, or `None` when
    /// the OID does not exist on this machine.
    fn value(&self, name: &str) -> Option<String>;
}

/// Byte ceiling for one recorded `sysctl(8)` listing.
const MAX_SYSCTL_LISTING_BYTES: usize = 1024 * 1024;
/// Entry ceiling for one recorded `sysctl(8)` listing.
const MAX_SYSCTL_ENTRIES: usize = 4096;
/// Length ceiling for one OID name.
const MAX_SYSCTL_NAME_BYTES: usize = 128;
/// Preview ceiling for malformed values quoted in errors.
const MAX_SYSCTL_PREVIEW_BYTES: usize = 64;
/// Ceiling on `hw.nperflevels` (Apple ships two; the bound keeps a corrupt
/// value from driving an unbounded OID walk).
const MAX_PERF_LEVELS: u32 = 8;

/// A recorded `sysctl(8)` listing: `name: value` lines, with blank lines and
/// `#` comments ignored. The committed `fixtures/sysctl_*.txt` files are
/// unedited `sysctl` stdout under a comment header.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SysctlSnapshot {
    values: BTreeMap<String, String>,
}

impl SysctlSnapshot {
    /// Parse a `sysctl(8)` listing.
    ///
    /// # Errors
    /// [`TopologyError::Invalid`] for a listing over the size or entry
    /// budget, a line that is not `name: value`, a malformed OID name, an
    /// oversized value, or a duplicated OID.
    pub fn parse(text: &str) -> Result<Self, TopologyError> {
        if text.len() > MAX_SYSCTL_LISTING_BYTES {
            return Err(TopologyError::Invalid {
                detail: format!(
                    "sysctl listing exceeds the {MAX_SYSCTL_LISTING_BYTES}-byte limit ({} bytes)",
                    text.len()
                ),
            });
        }
        let mut values = BTreeMap::new();
        for (index, line) in text.lines().enumerate() {
            let line = line.trim_end_matches('\r');
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let line_no = index + 1;
            let Some((name, value)) = line.split_once(": ") else {
                return Err(TopologyError::Invalid {
                    detail: format!("sysctl listing line {line_no} is not `name: value`"),
                });
            };
            if name.is_empty()
                || name.len() > MAX_SYSCTL_NAME_BYTES
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_')
            {
                return Err(TopologyError::Invalid {
                    detail: format!("sysctl listing line {line_no} has a malformed OID name"),
                });
            }
            if value.len() > MAX_TOPOLOGY_SCALAR_BYTES {
                return Err(TopologyError::Invalid {
                    detail: format!(
                        "sysctl listing line {line_no} value exceeds the \
                         {MAX_TOPOLOGY_SCALAR_BYTES}-byte limit"
                    ),
                });
            }
            if values.len() == MAX_SYSCTL_ENTRIES {
                return Err(TopologyError::Invalid {
                    detail: format!("sysctl listing exceeds the {MAX_SYSCTL_ENTRIES}-entry limit"),
                });
            }
            if values.insert(name.to_string(), value.to_string()).is_some() {
                return Err(TopologyError::Invalid {
                    detail: format!("sysctl listing repeats {name} (line {line_no})"),
                });
            }
        }
        Ok(Self { values })
    }
}

impl SysctlSource for SysctlSnapshot {
    fn value(&self, name: &str) -> Option<String> {
        self.values.get(name).cloned()
    }
}

/// One Darwin performance level (`hw.perflevel{N}`), or the whole machine
/// when the OS predates performance levels.
struct DarwinLevel {
    logical: u32,
    physical: u32,
    l2: SharedCache,
    l3: SharedCache,
}

/// A cache level as Darwin reports it: a size and how many logical CPUs share
/// one instance. Darwin reports widths, never memberships.
struct SharedCache {
    size_bytes: Option<u64>,
    cpus_per_domain: Option<u32>,
}

fn sysctl_optional_u64(
    sysctl: &dyn SysctlSource,
    name: &str,
) -> Result<Option<u64>, TopologyError> {
    let Some(raw) = sysctl.value(name) else {
        return Ok(None);
    };
    raw.trim()
        .parse::<u64>()
        .map(Some)
        .map_err(|_| TopologyError::SysctlParse {
            name: name.to_string(),
            detail: format!(
                "expected an unsigned integer, found {:?}",
                sysctl_preview(&raw)
            ),
        })
}

fn sysctl_optional_u32(
    sysctl: &dyn SysctlSource,
    name: &str,
) -> Result<Option<u32>, TopologyError> {
    sysctl_optional_u64(sysctl, name)?
        .map(|value| {
            u32::try_from(value).map_err(|_| TopologyError::SysctlParse {
                name: name.to_string(),
                detail: format!("{value} exceeds u32"),
            })
        })
        .transpose()
}

fn sysctl_u32(sysctl: &dyn SysctlSource, name: &str) -> Result<u32, TopologyError> {
    sysctl_optional_u32(sysctl, name)?.ok_or_else(|| TopologyError::MissingSysctl {
        name: name.to_string(),
    })
}

/// `hw.cacheconfig`: logical CPUs sharing each level (index 0 is memory,
/// then L1, L2, L3, …); `0` means the level is absent.
fn sysctl_cacheconfig(sysctl: &dyn SysctlSource) -> Result<Vec<u32>, TopologyError> {
    const NAME: &str = "hw.cacheconfig";
    let Some(raw) = sysctl.value(NAME) else {
        return Ok(Vec::new());
    };
    raw.split_whitespace()
        .map(|field| {
            field
                .parse::<u32>()
                .map_err(|_| TopologyError::SysctlParse {
                    name: NAME.to_string(),
                    detail: format!(
                        "expected unsigned sharing counts, found {:?}",
                        sysctl_preview(&raw)
                    ),
                })
        })
        .collect()
}

fn sysctl_preview(raw: &str) -> String {
    raw.chars().take(MAX_SYSCTL_PREVIEW_BYTES).collect()
}

fn darwin_levels(sysctl: &dyn SysctlSource) -> Result<Vec<DarwinLevel>, TopologyError> {
    let Some(count) = sysctl_optional_u32(sysctl, "hw.nperflevels")? else {
        // Pre-performance-level macOS: one uniform level, with cache sharing
        // from `hw.cacheconfig` when it is reported.
        let sharing = sysctl_cacheconfig(sysctl)?;
        let width = |level: usize| sharing.get(level).copied().filter(|&n| n > 0);
        return Ok(vec![DarwinLevel {
            logical: sysctl_u32(sysctl, "hw.logicalcpu")?,
            physical: sysctl_u32(sysctl, "hw.physicalcpu")?,
            l2: SharedCache {
                size_bytes: sysctl_optional_u64(sysctl, "hw.l2cachesize")?,
                cpus_per_domain: width(2),
            },
            l3: SharedCache {
                size_bytes: sysctl_optional_u64(sysctl, "hw.l3cachesize")?,
                cpus_per_domain: width(3),
            },
        }]);
    };
    if count == 0 || count > MAX_PERF_LEVELS {
        return Err(TopologyError::Invalid {
            detail: format!("hw.nperflevels is {count}; expected 1..={MAX_PERF_LEVELS}"),
        });
    }
    (0..count)
        .map(|level| {
            let key = |field: &str| format!("hw.perflevel{level}.{field}");
            Ok(DarwinLevel {
                logical: sysctl_u32(sysctl, &key("logicalcpu"))?,
                physical: sysctl_u32(sysctl, &key("physicalcpu"))?,
                l2: SharedCache {
                    size_bytes: sysctl_optional_u64(sysctl, &key("l2cachesize"))?,
                    cpus_per_domain: sysctl_optional_u32(sysctl, &key("cpusperl2"))?
                        .filter(|&n| n > 0),
                },
                l3: SharedCache {
                    size_bytes: sysctl_optional_u64(sysctl, &key("l3cachesize"))?,
                    cpus_per_domain: sysctl_optional_u32(sysctl, &key("cpusperl3"))?
                        .filter(|&n| n > 0),
                },
            })
        })
        .collect()
}

/// Append one cache domain per `cpus_per_domain`-wide run of `cpus`; a width
/// at least as wide as `cpus` is one domain holding all of them. Unknown
/// sharing records no domain: a width is never guessed, and neither is a
/// membership. A narrower width that does not divide `cpus` would end in a
/// shorter run that Darwin never reported (the M4 Pro's `hw.cacheconfig`
/// gives an L2 width of 4 over 14 CPUs clustered 5 + 5 + 4), so it records
/// none.
fn push_darwin_domains(out: &mut Vec<CacheDomain>, level: u8, cache: &SharedCache, cpus: &[u32]) {
    let Some(width) = cache.cpus_per_domain else {
        return;
    };
    let width = usize::try_from(width).unwrap_or(usize::MAX);
    if width < cpus.len() && !cpus.len().is_multiple_of(width) {
        return;
    }
    for chunk in cpus.chunks(width) {
        out.push(CacheDomain {
            level,
            size_bytes: cache.size_bytes,
            cpus: chunk.to_vec(),
        });
    }
}

const SYS_CPU: &str = "/sys/devices/system/cpu";
const SYS_NODE: &str = "/sys/devices/system/node";
const PROC_MEMINFO: &str = "/proc/meminfo";
const MAX_TOPOLOGY_SCALAR_BYTES: usize = 4 * 1024;
const MAX_CPU_LIST_BYTES: usize = 1024 * 1024;
const MAX_MEMINFO_BYTES: usize = 1024 * 1024;

impl HardwareTopology {
    /// Logical CPU count.
    #[must_use]
    pub fn logical_cores(&self) -> u32 {
        u32::try_from(self.cpus.len()).unwrap_or(u32::MAX)
    }

    /// Whether SMT is active (any core carries more than one logical CPU).
    #[must_use]
    pub fn smt_active(&self) -> bool {
        self.logical_cores() > self.physical_cores
    }

    /// The topology of the running machine. Linux introspects sysfs; other
    /// hosts (and a sysfs read failure) fall back to
    /// [`HardwareTopology::fallback`] over `available_parallelism` — the
    /// planner always gets *a* topology, just a flat one. macOS included:
    /// [`HardwareTopology::detect_macos`] needs a live [`SysctlSource`],
    /// and no governed dependency provides one yet (see the module doc).
    ///
    /// wasm32 (W5 tier 1, fm-l97): the browser sandbox is single-threaded —
    /// no shared memory, no atomics, no cross-origin isolation (that is the
    /// documented tier-2 question) — so the planner is handed exactly one
    /// logical CPU, always. SIMD128 is not a §17.3 build tier (R22 caps the
    /// tier set at four), so the flat portable fallback is the honest shape.
    #[must_use]
    pub fn current() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::fallback(1)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let logical = std::thread::available_parallelism()
                .map(|n| u32::try_from(n.get()).unwrap_or(1))
                .unwrap_or(1);
            if cfg!(target_os = "linux") {
                Self::detect_linux(&crate::fs::StdFs).unwrap_or_else(|_| Self::fallback(logical))
            } else {
                Self::fallback(logical)
            }
        }
    }

    /// A flat topology from a logical-CPU count: one package, no SMT
    /// knowledge, no cache/NUMA structure, groups per the Windows split,
    /// the detected SIMD tier, and unknown memory.
    #[must_use]
    pub fn fallback(logical: u32) -> Self {
        let logical = logical.max(1);
        let cpus: Vec<LogicalCpu> = (0..logical)
            .map(|id| LogicalCpu {
                id,
                package_id: 0,
                core_id: id,
                capacity: None,
                max_freq_khz: None,
                class: PerfClass::Performance,
            })
            .collect();
        let mut topo = Self {
            physical_cores: logical,
            packages: 1,
            l2_domains: Vec::new(),
            l3_domains: Vec::new(),
            numa_nodes: vec![NumaNode {
                id: 0,
                cpus: cpus.iter().map(|c| c.id).collect(),
            }],
            processor_groups: Vec::new(),
            simd_tier: detect_simd_tier(),
            total_memory_bytes: None,
            cpus,
        };
        topo.processor_groups = group_by_split(&topo.cpu_ids());
        topo
    }

    /// A synthetic topology from explicit processor-group sizes — the
    /// Windows-model constructor the synthetic tests drive.
    ///
    /// # Errors
    /// [`TopologyError::Invalid`] if a group is empty or exceeds 64 CPUs, or
    /// if the aggregate topology is empty or exceeds its resource budget.
    pub fn from_group_sizes(sizes: &[u32]) -> Result<Self, TopologyError> {
        if sizes.is_empty() {
            return Err(TopologyError::Invalid {
                detail: "zero logical CPUs".to_string(),
            });
        }
        if sizes.contains(&0) {
            return Err(TopologyError::Invalid {
                detail: "processor groups must contain at least one CPU".to_string(),
            });
        }
        if let Some(&too_big) = sizes.iter().find(|&&s| s > 64) {
            return Err(TopologyError::Invalid {
                detail: format!("processor group of {too_big} CPUs exceeds the 64-CPU limit"),
            });
        }
        let total = sizes
            .iter()
            .try_fold(0_u32, |total, &size| total.checked_add(size))
            .ok_or_else(|| TopologyError::Invalid {
                detail: "processor-group CPU total exceeds u32".to_string(),
            })?;
        let total_entries = usize::try_from(total).map_err(|_| TopologyError::Invalid {
            detail: "processor-group CPU total exceeds addressable memory".to_string(),
        })?;
        if total_entries > MAX_CPU_LIST_ENTRIES {
            return Err(TopologyError::Invalid {
                detail: format!(
                    "processor-group CPU total exceeds the {MAX_CPU_LIST_ENTRIES}-CPU limit"
                ),
            });
        }
        let mut topo = Self::fallback(total);
        let mut groups = Vec::new();
        let mut next = 0u32;
        for (gid, &size) in sizes.iter().enumerate() {
            groups.push(ProcessorGroup {
                id: u32::try_from(gid).unwrap_or(u32::MAX),
                cpus: (next..next + size).collect(),
            });
            next += size;
        }
        topo.processor_groups = groups;
        Ok(topo)
    }

    fn cpu_ids(&self) -> Vec<u32> {
        self.cpus.iter().map(|c| c.id).collect()
    }

    /// Introspect a Linux sysfs/procfs tree through the filesystem
    /// capability. Required: the online-CPU list. Everything else degrades
    /// gracefully (missing optional files leave `None`s / empty domains).
    ///
    /// # Errors
    /// [`TopologyError`] when the online list is missing or malformed.
    pub fn detect_linux(fs: &dyn FileSystem) -> Result<Self, TopologyError> {
        let online_path = PathBuf::from(SYS_CPU).join("online");
        let online = read_string(fs, &online_path)?;
        let ids = parse_cpu_list(&online)?;
        if ids.is_empty() {
            return Err(TopologyError::Invalid {
                detail: "empty online CPU list".to_string(),
            });
        }

        let mut cpus = Vec::with_capacity(ids.len());
        for &id in &ids {
            let base = PathBuf::from(SYS_CPU).join(format!("cpu{id}"));
            let package_id = read_optional_u32(fs, &base.join("topology/physical_package_id"))
                .unwrap_or_default();
            let core_id = read_optional_u32(fs, &base.join("topology/core_id")).unwrap_or(id);
            let capacity = read_optional_u32(fs, &base.join("cpu_capacity"));
            let max_freq_khz = read_optional_u64(fs, &base.join("cpufreq/cpuinfo_max_freq"));
            cpus.push(LogicalCpu {
                id,
                package_id,
                core_id,
                capacity,
                max_freq_khz,
                class: PerfClass::Performance, // assigned below
            });
        }
        assign_perf_classes(&mut cpus);

        let mut cores: Vec<(u32, u32)> = cpus.iter().map(|c| (c.package_id, c.core_id)).collect();
        cores.sort_unstable();
        cores.dedup();
        let mut packages: Vec<u32> = cpus.iter().map(|c| c.package_id).collect();
        packages.sort_unstable();
        packages.dedup();

        let (l2_domains, l3_domains) = read_cache_domains(fs, &ids);
        let numa_nodes = read_numa_nodes(fs).unwrap_or_else(|| {
            vec![NumaNode {
                id: 0,
                cpus: ids.clone(),
            }]
        });
        let total_memory_bytes = read_meminfo_total(fs);

        Ok(Self {
            physical_cores: u32::try_from(cores.len()).unwrap_or(u32::MAX),
            packages: u32::try_from(packages.len()).unwrap_or(u32::MAX),
            l2_domains,
            l3_domains,
            numa_nodes,
            processor_groups: group_by_split(&ids),
            simd_tier: detect_simd_tier(),
            total_memory_bytes,
            cpus,
        })
    }

    /// Introspect a macOS `hw.*` sysctl MIB through the sysctl capability.
    ///
    /// Each performance level (`hw.perflevel{N}`, highest performance first)
    /// contributes its cores in order: level 0 is [`PerfClass::Performance`],
    /// every later level [`PerfClass::Efficiency`]. A level's `cpusperl2` /
    /// `cpusperl3` width splits its CPUs into cache domains when it divides
    /// them (a width that does not records none). macOS predating
    /// performance levels is one uniform level, with cache sharing from
    /// `hw.cacheconfig`.
    ///
    /// macOS exposes neither a per-CPU cluster map nor thread affinity, so
    /// CPU ids here are stable labels in performance-level order (SMT
    /// siblings adjacent), not kernel processor numbers. Capacities and
    /// frequencies are not reported and stay `None`.
    ///
    /// # Errors
    /// [`TopologyError::MissingSysctl`] when `hw.logicalcpu`,
    /// `hw.physicalcpu` or a level's CPU counts are absent;
    /// [`TopologyError::SysctlParse`] for a malformed value; and
    /// [`TopologyError::Invalid`] when the levels do not partition the
    /// machine, a level has no cores, fewer logical CPUs than cores, or
    /// logical CPUs that are not a whole multiple of its cores, or the host
    /// reports more than one package.
    pub fn detect_macos(sysctl: &dyn SysctlSource) -> Result<Self, TopologyError> {
        let logical = sysctl_u32(sysctl, "hw.logicalcpu")?;
        let physical = sysctl_u32(sysctl, "hw.physicalcpu")?;
        let logical_entries = usize::try_from(logical).unwrap_or(usize::MAX);
        if logical == 0 || physical == 0 || physical > logical {
            return Err(TopologyError::Invalid {
                detail: format!("{logical} logical CPUs over {physical} physical cores"),
            });
        }
        if logical_entries > MAX_CPU_LIST_ENTRIES {
            return Err(TopologyError::Invalid {
                detail: format!(
                    "{logical} logical CPUs exceed the {MAX_CPU_LIST_ENTRIES}-CPU limit"
                ),
            });
        }
        if let Some(packages) = sysctl_optional_u32(sysctl, "hw.packages")?
            && packages != 1
        {
            return Err(TopologyError::Invalid {
                detail: format!(
                    "hw.packages is {packages}; Darwin reports no CPU-to-package map, \
                     so only single-package hosts are modeled"
                ),
            });
        }

        let levels = darwin_levels(sysctl)?;
        let (level_logical, level_physical) =
            levels.iter().fold((0_u64, 0_u64), |(l, p), level| {
                (l + u64::from(level.logical), p + u64::from(level.physical))
            });
        if level_logical != u64::from(logical) || level_physical != u64::from(physical) {
            return Err(TopologyError::Invalid {
                detail: format!(
                    "performance levels hold {level_logical} logical CPUs over \
                     {level_physical} cores; the host reports {logical} over {physical}"
                ),
            });
        }

        let mut cpus = Vec::with_capacity(logical_entries);
        let mut l2_domains = Vec::new();
        let mut l3_domains = Vec::new();
        let mut next_id = 0_u32;
        let mut next_core = 0_u32;
        for (index, level) in levels.iter().enumerate() {
            // Every core carries at least one logical CPU: zero is a whole
            // multiple of any core count, so it is refused by name.
            if level.physical == 0
                || level.logical < level.physical
                || !level.logical.is_multiple_of(level.physical)
            {
                return Err(TopologyError::Invalid {
                    detail: format!(
                        "performance level {index} has {} logical CPUs over {} cores",
                        level.logical, level.physical
                    ),
                });
            }
            let class = if index == 0 {
                PerfClass::Performance
            } else {
                PerfClass::Efficiency
            };
            let threads_per_core = level.logical / level.physical;
            let first = next_id;
            for _ in 0..level.physical {
                for _ in 0..threads_per_core {
                    cpus.push(LogicalCpu {
                        id: next_id,
                        package_id: 0,
                        core_id: next_core,
                        capacity: None,
                        max_freq_khz: None,
                        class,
                    });
                    next_id += 1;
                }
                next_core += 1;
            }
            let ids: Vec<u32> = (first..next_id).collect();
            push_darwin_domains(&mut l2_domains, 2, &level.l2, &ids);
            push_darwin_domains(&mut l3_domains, 3, &level.l3, &ids);
        }

        let ids: Vec<u32> = cpus.iter().map(|cpu| cpu.id).collect();
        Ok(Self {
            physical_cores: physical,
            packages: 1,
            l2_domains,
            l3_domains,
            numa_nodes: vec![NumaNode {
                id: 0,
                cpus: ids.clone(),
            }],
            processor_groups: group_by_split(&ids),
            simd_tier: detect_simd_tier(),
            total_memory_bytes: sysctl_optional_u64(sysctl, "hw.memsize")?,
            cpus,
        })
    }

    /// A deterministic text rendering — the committed fixture format
    /// (`fixtures/*.snapshot.txt`) and `fmn doctor`'s raw form.
    #[must_use]
    pub fn snapshot_text(&self) -> String {
        let mut out = String::from("# fmn hardware-topology snapshot v1\n");
        let push = |out: &mut String, line: String| {
            out.push_str(&line);
            out.push('\n');
        };
        push(&mut out, format!("logical_cores\t{}", self.logical_cores()));
        push(&mut out, format!("physical_cores\t{}", self.physical_cores));
        push(&mut out, format!("packages\t{}", self.packages));
        push(&mut out, format!("smt\t{}", self.smt_active()));
        push(&mut out, format!("simd_tier\t{}", self.simd_tier.name()));
        push(
            &mut out,
            format!(
                "memory_bytes\t{}",
                self.total_memory_bytes
                    .map_or_else(|| "-".to_string(), |m| m.to_string())
            ),
        );
        for c in &self.cpus {
            push(
                &mut out,
                format!(
                    "cpu\t{}\tpackage\t{}\tcore\t{}\tclass\t{}\tcapacity\t{}\tmax_freq_khz\t{}",
                    c.id,
                    c.package_id,
                    c.core_id,
                    match c.class {
                        PerfClass::Performance => "P",
                        PerfClass::Efficiency => "E",
                    },
                    c.capacity
                        .map_or_else(|| "-".to_string(), |v| v.to_string()),
                    c.max_freq_khz
                        .map_or_else(|| "-".to_string(), |v| v.to_string()),
                ),
            );
        }
        for (label, domains) in [("l2", &self.l2_domains), ("l3", &self.l3_domains)] {
            for d in domains {
                push(
                    &mut out,
                    format!(
                        "{label}\tsize\t{}\tcpus\t{}",
                        d.size_bytes
                            .map_or_else(|| "-".to_string(), |v| v.to_string()),
                        format_cpu_list(&d.cpus)
                    ),
                );
            }
        }
        for n in &self.numa_nodes {
            push(
                &mut out,
                format!("numa\t{}\tcpus\t{}", n.id, format_cpu_list(&n.cpus)),
            );
        }
        for g in &self.processor_groups {
            push(
                &mut out,
                format!("group\t{}\tcpus\t{}", g.id, format_cpu_list(&g.cpus)),
            );
        }
        out
    }
}

/// Assign P/E classes: if the machine exposes heterogeneous capacities, the
/// maximum-capacity CPUs are Performance and the rest Efficiency; failing
/// that, heterogeneous max frequencies decide the same way (with a 20%
/// hysteresis so binning variance is not misread as E-cores); a uniform
/// machine is all Performance.
fn assign_perf_classes(cpus: &mut [LogicalCpu]) {
    let capacities: Vec<u32> = cpus.iter().filter_map(|c| c.capacity).collect();
    if let Some(&max) = capacities.iter().max()
        && capacities.iter().any(|&c| c != max)
        && capacities.len() == cpus.len()
    {
        for c in cpus {
            c.class = if c.capacity == Some(max) {
                PerfClass::Performance
            } else {
                PerfClass::Efficiency
            };
        }
        return;
    }
    let freqs: Vec<u64> = cpus.iter().filter_map(|c| c.max_freq_khz).collect();
    if let Some(&max) = freqs.iter().max()
        && freqs.len() == cpus.len()
    {
        let threshold = max - max / 5;
        if freqs.iter().any(|&f| f < threshold) {
            for c in cpus {
                c.class = if c.max_freq_khz.unwrap_or(0) >= threshold {
                    PerfClass::Performance
                } else {
                    PerfClass::Efficiency
                };
            }
        }
    }
}

fn group_by_split(ids: &[u32]) -> Vec<ProcessorGroup> {
    let mut groups = Vec::new();
    for (gid, chunk) in ids.chunks(64).enumerate() {
        groups.push(ProcessorGroup {
            id: u32::try_from(gid).unwrap_or(u32::MAX),
            cpus: chunk.to_vec(),
        });
    }
    groups
}

/// Bounded ceiling for one `/proc/<pid>/status` read; the real file is ~2 KiB.
const MAX_PROC_STATUS_BYTES: usize = 64 * 1024;

/// Resident-set size of the calling process, in bytes, through the capability
/// filesystem.
///
/// Linux only by construction: parses the `VmRSS:` row of `/proc/self/status`
/// (kernel-reported in kB). Hosts without that file — macOS, Windows, wasm —
/// return `Ok(None)`, and consumers such as PG-6's leak soak retain their
/// samples as inconclusive rather than inventing a number. A present file with
/// an unparsable `VmRSS` row is a typed parse error, never a silent `None`.
///
/// # Errors
/// [`TopologyError::Parse`] when the status file exists but its `VmRSS` row is
/// malformed.
pub fn current_rss_bytes(fs: &dyn FileSystem) -> Result<Option<u64>, TopologyError> {
    let path = Path::new("/proc/self/status");
    let Some(status) = read_optional_string(fs, path, MAX_PROC_STATUS_BYTES) else {
        return Ok(None);
    };
    let Some(row) = status.lines().find_map(|line| line.strip_prefix("VmRSS:")) else {
        // Kernel threads and some restricted mounts omit the row; absence is
        // an unsupported surface, not corruption.
        return Ok(None);
    };
    let parse = || -> Option<u64> {
        let mut fields = row.split_whitespace();
        let value: u64 = fields.next()?.parse().ok()?;
        match fields.next() {
            Some("kB") => value.checked_mul(1024),
            _ => None,
        }
    };
    parse().map(Some).ok_or_else(|| TopologyError::Parse {
        path: path.to_path_buf(),
        detail: format!("malformed VmRSS row {row:?}"),
    })
}

fn read_string(fs: &dyn FileSystem, path: &Path) -> Result<String, TopologyError> {
    match fs.read_to_string_bounded(path, MAX_CPU_LIST_BYTES) {
        Ok(s) => Ok(s),
        Err(FsError::NotFound { .. }) => Err(TopologyError::Missing {
            path: path.to_path_buf(),
        }),
        Err(e) => Err(TopologyError::Parse {
            path: path.to_path_buf(),
            detail: e.to_string(),
        }),
    }
}

fn read_optional_string(fs: &dyn FileSystem, path: &Path, max_bytes: usize) -> Option<String> {
    fs.read_to_string_bounded(path, max_bytes).ok()
}

fn read_optional_u32(fs: &dyn FileSystem, path: &Path) -> Option<u32> {
    read_optional_string(fs, path, MAX_TOPOLOGY_SCALAR_BYTES)?
        .trim()
        .parse()
        .ok()
}

fn read_optional_u64(fs: &dyn FileSystem, path: &Path) -> Option<u64> {
    read_optional_string(fs, path, MAX_TOPOLOGY_SCALAR_BYTES)?
        .trim()
        .parse()
        .ok()
}

/// Collect deduplicated L2/L3 cache domains from every CPU's cache indexes.
fn read_cache_domains(fs: &dyn FileSystem, ids: &[u32]) -> (Vec<CacheDomain>, Vec<CacheDomain>) {
    let mut l2: Vec<CacheDomain> = Vec::new();
    let mut l3: Vec<CacheDomain> = Vec::new();
    for &id in ids {
        for index in 0..8u32 {
            let base = PathBuf::from(SYS_CPU).join(format!("cpu{id}/cache/index{index}"));
            let Some(level) = read_optional_u32(fs, &base.join("level")) else {
                continue;
            };
            if level != 2 && level != 3 {
                continue;
            }
            let cache_type = fs
                .read_to_string_bounded(&base.join("type"), MAX_TOPOLOGY_SCALAR_BYTES)
                .map(|s| s.trim().to_string())
                .unwrap_or_default();
            if cache_type != "Unified" && cache_type != "Data" {
                continue;
            }
            let Ok(shared) =
                fs.read_to_string_bounded(&base.join("shared_cpu_list"), MAX_CPU_LIST_BYTES)
            else {
                continue;
            };
            let Ok(cpus) = parse_cpu_list(&shared) else {
                continue;
            };
            let size_bytes = fs
                .read_to_string_bounded(&base.join("size"), MAX_TOPOLOGY_SCALAR_BYTES)
                .ok()
                .and_then(|s| parse_cache_size(s.trim()));
            let domain = CacheDomain {
                level: u8::try_from(level).unwrap_or(u8::MAX),
                size_bytes,
                cpus,
            };
            let bucket = if level == 2 { &mut l2 } else { &mut l3 };
            if !bucket.iter().any(|d| d.cpus == domain.cpus) {
                bucket.push(domain);
            }
        }
    }
    for bucket in [&mut l2, &mut l3] {
        bucket.sort_by_key(|d| d.cpus.first().copied().unwrap_or(0));
    }
    (l2, l3)
}

/// Parse a sysfs cache size (`"32768K"`, `"32M"`) into bytes.
fn parse_cache_size(s: &str) -> Option<u64> {
    if let Some(k) = s.strip_suffix('K') {
        k.parse::<u64>().ok()?.checked_mul(1024)
    } else if let Some(m) = s.strip_suffix('M') {
        m.parse::<u64>().ok()?.checked_mul(1024 * 1024)
    } else {
        s.parse::<u64>().ok()
    }
}

/// Read NUMA nodes from `/sys/devices/system/node/node*/cpulist`; `None`
/// when the node directory is absent (the caller substitutes one node).
fn read_numa_nodes(fs: &dyn FileSystem) -> Option<Vec<NumaNode>> {
    let mut nodes = Vec::new();
    for id in 0..1024u32 {
        let path = PathBuf::from(SYS_NODE).join(format!("node{id}/cpulist"));
        match fs.read_to_string_bounded(&path, MAX_CPU_LIST_BYTES) {
            Ok(list) => {
                if let Ok(cpus) = parse_cpu_list(&list) {
                    nodes.push(NumaNode { id, cpus });
                }
            }
            Err(_) => break,
        }
    }
    if nodes.is_empty() { None } else { Some(nodes) }
}

/// `MemTotal` from `/proc/meminfo`, in bytes.
fn read_meminfo_total(fs: &dyn FileSystem) -> Option<u64> {
    let text = fs
        .read_to_string_bounded(Path::new(PROC_MEMINFO), MAX_MEMINFO_BYTES)
        .ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb: u64 = rest.trim().trim_end_matches("kB").trim().parse().ok()?;
            return kb.checked_mul(1024);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_list_round_trips() {
        for (text, ids) in [
            ("0-3", vec![0, 1, 2, 3]),
            ("0,2,4", vec![0, 2, 4]),
            ("0-1,64-65", vec![0, 1, 64, 65]),
            ("7", vec![7]),
            ("", vec![]),
        ] {
            assert_eq!(parse_cpu_list(text).unwrap(), ids, "{text}");
            assert_eq!(parse_cpu_list(&format_cpu_list(&ids)).unwrap(), ids);
        }
        assert!(parse_cpu_list("3-1").is_err());
        assert!(parse_cpu_list("a-b").is_err());

        let exact = format!("0-{}", MAX_CPU_LIST_ENTRIES - 1);
        assert_eq!(parse_cpu_list(&exact).unwrap().len(), MAX_CPU_LIST_ENTRIES);
        let over = format!("0-{MAX_CPU_LIST_ENTRIES}");
        assert!(matches!(
            parse_cpu_list(&over),
            Err(TopologyError::Invalid { detail }) if detail.contains("entry limit")
        ));
        assert!(parse_cpu_list("0-4294967295").is_err());
        assert_eq!(
            format_cpu_list(&[u32::MAX, u32::MAX]),
            "4294967295,4294967295"
        );
    }

    #[test]
    fn windows_split_never_exceeds_64() {
        assert_eq!(windows_group_split(1), vec![1]);
        assert_eq!(windows_group_split(64), vec![64]);
        assert_eq!(windows_group_split(96), vec![64, 32]);
        assert_eq!(windows_group_split(128), vec![64, 64]);
        assert_eq!(windows_group_split(192), vec![64, 64, 64]);
        assert_eq!(windows_group_split(0), vec![0]);
        assert!(HardwareTopology::from_group_sizes(&[0, 1]).is_err());
        let too_many = vec![64; MAX_CPU_LIST_ENTRIES / 64 + 1];
        assert!(HardwareTopology::from_group_sizes(&too_many).is_err());
    }

    #[test]
    fn cache_size_parses_k_and_m() {
        assert_eq!(parse_cache_size("32768K"), Some(32768 * 1024));
        assert_eq!(parse_cache_size("32M"), Some(32 * 1024 * 1024));
        assert_eq!(parse_cache_size("512"), Some(512));
        assert_eq!(parse_cache_size("x"), None);
        assert_eq!(parse_cache_size(&format!("{}K", u64::MAX)), None);
        assert_eq!(parse_cache_size(&format!("{}M", u64::MAX)), None);
    }

    #[test]
    fn topology_file_limits_and_memory_overflow_degrade_precisely() {
        use crate::fs::VirtualFs;

        let oversized = VirtualFs::new();
        oversized.insert(
            PathBuf::from(SYS_CPU).join("online"),
            vec![b'0'; MAX_CPU_LIST_BYTES + 1],
        );
        assert!(matches!(
            HardwareTopology::detect_linux(&oversized),
            Err(TopologyError::Parse { detail, .. }) if detail.contains("byte limit")
        ));

        let overflow = VirtualFs::new();
        overflow.insert(PathBuf::from(SYS_CPU).join("online"), b"0\n".to_vec());
        overflow.insert(
            PROC_MEMINFO,
            format!("MemTotal: {} kB\n", u64::MAX).into_bytes(),
        );
        let topology = HardwareTopology::detect_linux(&overflow).unwrap();
        assert_eq!(topology.total_memory_bytes, None);
    }

    #[test]
    fn current_rss_reads_vmrss_and_degrades_precisely() {
        use crate::fs::VirtualFs;
        let status = PathBuf::from("/proc/self/status");

        // The documented Linux shape, in bytes.
        let linux = VirtualFs::new();
        linux.insert(
            status.clone(),
            b"Name:\tfmn\nVmPeak:\t  9000 kB\nVmRSS:\t  4321 kB\nThreads:\t4\n".to_vec(),
        );
        assert_eq!(current_rss_bytes(&linux).unwrap(), Some(4321 * 1024));

        // No procfs (macOS/Windows/wasm): an unsupported surface, not an error.
        assert_eq!(current_rss_bytes(&VirtualFs::new()).unwrap(), None);

        // A status file without the row (kernel threads): also unsupported.
        let rowless = VirtualFs::new();
        rowless.insert(status.clone(), b"Name:\tfmn\nThreads:\t4\n".to_vec());
        assert_eq!(current_rss_bytes(&rowless).unwrap(), None);

        // A present but malformed row is corruption, never a silent None.
        for bad in ["VmRSS:\tpotato kB\n", "VmRSS:\t12 MB\n", "VmRSS:\t\n"] {
            let malformed = VirtualFs::new();
            malformed.insert(status.clone(), format!("Name:\tfmn\n{bad}").into_bytes());
            assert!(matches!(
                current_rss_bytes(&malformed),
                Err(TopologyError::Parse { detail, .. }) if detail.contains("VmRSS")
            ));
        }

        // Overflowing kB counts refuse rather than wrap.
        let huge = VirtualFs::new();
        huge.insert(status, format!("VmRSS:\t{} kB\n", u64::MAX).into_bytes());
        assert!(current_rss_bytes(&huge).is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn current_rss_reports_the_live_process_on_linux() {
        let rss = current_rss_bytes(&crate::fs::StdFs)
            .expect("host status parses")
            .expect("linux reports VmRSS");
        assert!(rss > 0, "a running test process is resident");
    }
}
