//! HardwareTopology introspection tests (fm-x68 acceptance):
//! synthetic sysfs trees (x86 SMT+CCD, aarch64 big.LITTLE) through the
//! filesystem capability, Windows processor-group handling against synthetic
//! topologies, and the real-machine snapshot fixture flow.
//!
//! To (re)record the committed snapshot of the machine running the tests:
//! `REGEN_TOPOLOGY=1 cargo test -p fmn-platform --test topology`, then
//! commit `fixtures/topology_<platform>.snapshot.txt`.

use fmn_platform::fs::VirtualFs;
use fmn_platform::topology::{
    CacheDomain, HardwareTopology, PerfClass, SysctlSnapshot, TopologyError,
};
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn load_tree(name: &str) -> VirtualFs {
    let manifest = std::fs::read_to_string(fixture_path(name)).expect("fixture manifest present");
    let fs = VirtualFs::new();
    fs.load_manifest(&manifest);
    fs
}

/// The real Apple M4 Pro (10P + 4E) `sysctl` recording.
const M4_PRO_SYSCTL: &str = "sysctl_macos-aarch64_m4pro.txt";
/// The committed snapshot derived from that recording.
const MACOS_AARCH64_SNAPSHOT: &str = "topology_macos-aarch64.snapshot.txt";

fn m4_pro_listing() -> String {
    std::fs::read_to_string(fixture_path(M4_PRO_SYSCTL)).expect("sysctl fixture present")
}

/// Detect from the M4 Pro recording with exactly one line replaced, so each
/// planted negative changes one recorded fact.
fn detect_m4_pro_with(line: &str, replacement: &str) -> Result<HardwareTopology, TopologyError> {
    let listing = m4_pro_listing();
    assert!(
        listing.lines().any(|l| l == line),
        "the recording carries {line:?}"
    );
    let edited: String = listing
        .lines()
        .map(|l| if l == line { replacement } else { l })
        .collect::<Vec<_>>()
        .join("\n");
    HardwareTopology::detect_macos(&SysctlSnapshot::parse(&edited).expect("edited listing parses"))
}

fn without_simd_line(snapshot: &str) -> String {
    snapshot
        .lines()
        .filter(|line| !line.starts_with("simd_tier\t"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn x86_smt_ccd_tree_parses_fully() {
    let fs = load_tree("sysfs_x86_smt_ccd.tsv");
    let t = HardwareTopology::detect_linux(&fs).expect("detect");

    assert_eq!(t.logical_cores(), 16);
    assert_eq!(t.physical_cores, 8);
    assert_eq!(t.packages, 1);
    assert!(t.smt_active());
    // Uniform frequencies: every CPU is Performance.
    assert!(t.cpus.iter().all(|c| c.class == PerfClass::Performance));
    // SMT pairs share an L2 (8 pair domains); two 4-core L3 domains (CCDs).
    assert_eq!(t.l2_domains.len(), 8);
    assert_eq!(t.l2_domains[0].cpus, vec![0, 8]);
    assert_eq!(t.l2_domains[0].size_bytes, Some(1024 * 1024));
    assert_eq!(t.l3_domains.len(), 2);
    assert_eq!(t.l3_domains[0].cpus, vec![0, 1, 2, 3, 8, 9, 10, 11]);
    assert_eq!(t.l3_domains[1].cpus, vec![4, 5, 6, 7, 12, 13, 14, 15]);
    assert_eq!(t.l3_domains[0].size_bytes, Some(16384 * 1024));
    // Two NUMA nodes matching the L3 split.
    assert_eq!(t.numa_nodes.len(), 2);
    assert_eq!(t.numa_nodes[0].cpus, t.l3_domains[0].cpus);
    // 16 CPUs fit one processor group.
    assert_eq!(t.processor_groups.len(), 1);
    assert_eq!(t.processor_groups[0].cpus.len(), 16);
    assert_eq!(t.total_memory_bytes, Some(32_768_000 * 1024));
}

#[test]
fn aarch64_biglittle_tree_derives_perf_classes() {
    let fs = load_tree("sysfs_aarch64_biglittle.tsv");
    let t = HardwareTopology::detect_linux(&fs).expect("detect");

    assert_eq!(t.logical_cores(), 8);
    assert_eq!(t.physical_cores, 8);
    assert!(!t.smt_active());
    // Capacity 512 → Efficiency (cpu0-3); capacity 1024 → Performance (cpu4-7).
    for c in &t.cpus {
        let expected = if c.id < 4 {
            PerfClass::Efficiency
        } else {
            PerfClass::Performance
        };
        assert_eq!(c.class, expected, "cpu{}", c.id);
        assert_eq!(c.capacity, Some(if c.id < 4 { 512 } else { 1024 }));
    }
    // Two L2 cluster domains, one DSU L3, implicit single NUMA node.
    assert_eq!(t.l2_domains.len(), 2);
    assert_eq!(t.l2_domains[0].cpus, vec![0, 1, 2, 3]);
    assert_eq!(t.l2_domains[1].cpus, vec![4, 5, 6, 7]);
    assert_eq!(t.l3_domains.len(), 1);
    assert_eq!(t.l3_domains[0].cpus.len(), 8);
    assert_eq!(t.numa_nodes.len(), 1);
    assert_eq!(t.total_memory_bytes, Some(8_000_000 * 1024));
}

#[test]
fn windows_groups_split_above_64_and_reject_oversize() {
    // A 96-logical machine models as two groups: 64 + 32.
    let t = HardwareTopology::from_group_sizes(&[64, 32]).expect("96-cpu model");
    assert_eq!(t.logical_cores(), 96);
    assert_eq!(t.processor_groups.len(), 2);
    assert_eq!(t.processor_groups[0].cpus.len(), 64);
    assert_eq!(t.processor_groups[1].cpus.len(), 32);
    assert_eq!(t.processor_groups[1].cpus[0], 64);

    // A single group above 64 violates the Windows invariant.
    assert!(matches!(
        HardwareTopology::from_group_sizes(&[65]),
        Err(TopologyError::Invalid { .. })
    ));
    assert!(matches!(
        HardwareTopology::from_group_sizes(&[]),
        Err(TopologyError::Invalid { .. })
    ));

    // The fallback constructor applies the same split: 128 logical → 2×64,
    // so even a topology-blind host cannot produce an oversized group.
    let f = HardwareTopology::fallback(128);
    assert_eq!(f.processor_groups.len(), 2);
    assert!(f.processor_groups.iter().all(|g| g.cpus.len() <= 64));
}

#[test]
fn degraded_tree_still_detects() {
    // Only the online list: everything optional missing. Detection succeeds
    // with sane defaults (core_id = cpu id, package 0, no domains).
    let fs = VirtualFs::new();
    fs.insert("/sys/devices/system/cpu/online", b"0-3\n".to_vec());
    let t = HardwareTopology::detect_linux(&fs).expect("degraded detect");
    assert_eq!(t.logical_cores(), 4);
    assert_eq!(t.physical_cores, 4);
    assert!(t.l3_domains.is_empty());
    assert_eq!(t.numa_nodes.len(), 1);
    assert_eq!(t.total_memory_bytes, None);

    // No online list at all: a named error, not a guess.
    let empty = VirtualFs::new();
    assert!(matches!(
        HardwareTopology::detect_linux(&empty),
        Err(TopologyError::Missing { .. })
    ));
}

#[test]
fn snapshot_text_is_deterministic_and_versioned() {
    let fs = load_tree("sysfs_x86_smt_ccd.tsv");
    let t = HardwareTopology::detect_linux(&fs).expect("detect");
    let a = t.snapshot_text();
    assert_eq!(a, t.snapshot_text());
    assert!(a.starts_with("# fmn hardware-topology snapshot v1\n"));
    assert!(a.contains("logical_cores\t16"));
    assert!(a.contains("l3\tsize\t16777216\tcpus\t0-3,8-11"));
}

/// The real machine: detection succeeds and the committed per-platform
/// snapshot fixture exists for this OS/arch (recorded, not asserted — the
/// snapshot is a fixture of *a* machine of this platform, and CI hosts
/// differ in core count).
#[test]
#[cfg(target_os = "linux")]
fn real_machine_detects_and_snapshot_recorded() {
    let t = HardwareTopology::current();
    assert!(t.logical_cores() >= 1);
    assert!(t.physical_cores >= 1 && t.physical_cores <= t.logical_cores());
    assert!(t.processor_groups.iter().all(|g| g.cpus.len() <= 64));

    let name = format!(
        "topology_{}-{}.snapshot.txt",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name);
    if std::env::var_os("REGEN_TOPOLOGY").is_some() {
        std::fs::write(&path, t.snapshot_text()).expect("write snapshot");
        eprintln!("recorded {}", path.display());
        return;
    }
    // x86-64 CI is the committed baseline; other platforms record theirs
    // the first time the suite runs there (see the module doc).
    if std::env::consts::ARCH == "x86_64" {
        let text = std::fs::read_to_string(&path)
            .expect("committed topology snapshot for linux-x86_64; record with REGEN_TOPOLOGY=1");
        assert!(text.starts_with("# fmn hardware-topology snapshot v1\n"));
    }
}

/// fm-macos-hardware-topology-l29x acceptance: the real M4 Pro recording
/// yields 10 performance + 4 efficiency cores, two 5-core P clusters on a
/// 16 MiB L2 each, and one 4-core E cluster on a 4 MiB L2.
#[test]
fn macos_m4_pro_sysctl_derives_ten_p_four_e_and_their_l2_clusters() {
    let sysctl = SysctlSnapshot::parse(&m4_pro_listing()).expect("recording parses");
    let t = HardwareTopology::detect_macos(&sysctl).expect("detect");

    assert_eq!(t.logical_cores(), 14);
    assert_eq!(t.physical_cores, 14);
    assert_eq!(t.packages, 1);
    assert!(!t.smt_active());
    let ids_of = |class: PerfClass| -> Vec<u32> {
        t.cpus
            .iter()
            .filter(|cpu| cpu.class == class)
            .map(|cpu| cpu.id)
            .collect()
    };
    assert_eq!(ids_of(PerfClass::Performance), (0..10).collect::<Vec<_>>());
    assert_eq!(ids_of(PerfClass::Efficiency), (10..14).collect::<Vec<_>>());
    assert!(
        t.cpus
            .iter()
            .all(|cpu| cpu.capacity.is_none() && cpu.max_freq_khz.is_none()),
        "macOS reports neither capacity nor frequency"
    );

    let l2 = |size_mib: u64, cpus: std::ops::Range<u32>| CacheDomain {
        level: 2,
        size_bytes: Some(size_mib * 1024 * 1024),
        cpus: cpus.collect(),
    };
    assert_eq!(
        t.l2_domains,
        vec![l2(16, 0..5), l2(16, 5..10), l2(4, 10..14)]
    );
    // Apple's system-level cache is not reported as an L3.
    assert!(t.l3_domains.is_empty());
    // REGEN_TOPOLOGY writes the committed snapshot from this code, so every
    // fact it records is pinned here as well: CPUs in order, one package, one
    // core per CPU (no SMT), and one NUMA node and one processor group that
    // each hold all fourteen CPUs.
    let all: Vec<u32> = (0..14).collect();
    assert_eq!(
        t.cpus
            .iter()
            .map(|cpu| (cpu.id, cpu.package_id, cpu.core_id))
            .collect::<Vec<_>>(),
        all.iter().map(|&i| (i, 0, i)).collect::<Vec<_>>()
    );
    assert_eq!(t.numa_nodes.len(), 1);
    assert_eq!((t.numa_nodes[0].id, &t.numa_nodes[0].cpus), (0, &all));
    assert_eq!(t.processor_groups.len(), 1);
    assert_eq!(
        (t.processor_groups[0].id, &t.processor_groups[0].cpus),
        (0, &all)
    );
    assert_eq!(t.total_memory_bytes, Some(64 * 1024 * 1024 * 1024));

    // Planted negative: the L2 assertion reads the recorded cluster width.
    // A 10-wide P cluster is one domain, so the expectation above fails.
    let wide = detect_m4_pro_with("hw.perflevel0.cpusperl2: 5", "hw.perflevel0.cpusperl2: 10")
        .expect("detect");
    assert_eq!(wide.l2_domains[0].cpus, (0..10).collect::<Vec<_>>());
    assert_ne!(wide.l2_domains, t.l2_domains);
}

/// The committed `topology_macos-aarch64.snapshot.txt` is the snapshot of
/// the recorded M4 Pro (the real-machine fixture fm-0do asks for on macOS).
/// The SIMD tier is the running process's `std::arch` answer, not a sysctl
/// fact, so that one line is compared only on aarch64 macOS. Re-record on
/// such a host with `REGEN_TOPOLOGY=1`.
#[test]
fn macos_snapshot_fixture_is_the_recorded_m4_pro() {
    let sysctl = SysctlSnapshot::parse(&m4_pro_listing()).expect("recording parses");
    let t = HardwareTopology::detect_macos(&sysctl).expect("detect");
    let path = fixture_path(MACOS_AARCH64_SNAPSHOT);
    let on_recorded_platform = cfg!(all(target_os = "macos", target_arch = "aarch64"));
    if on_recorded_platform && std::env::var_os("REGEN_TOPOLOGY").is_some() {
        std::fs::write(&path, t.snapshot_text()).expect("write snapshot");
        eprintln!("recorded {}", path.display());
        return;
    }
    let committed = std::fs::read_to_string(&path).expect("committed macOS snapshot");
    assert!(committed.starts_with("# fmn hardware-topology snapshot v1\n"));
    assert!(committed.contains("\nsimd_tier\taarch64-neon\n"));
    if on_recorded_platform {
        assert_eq!(committed, t.snapshot_text());
    } else {
        assert_eq!(
            without_simd_line(&committed),
            without_simd_line(&t.snapshot_text())
        );
    }

    // Planted negative: one recorded fact moves the snapshot, so the lock
    // above is not tautological.
    let smaller =
        detect_m4_pro_with("hw.memsize: 68719476736", "hw.memsize: 34359738368").expect("detect");
    assert_ne!(
        without_simd_line(&committed),
        without_simd_line(&smaller.snapshot_text())
    );
}

#[test]
fn macos_levels_must_partition_the_machine() {
    // A level that loses a core no longer sums to hw.logicalcpu/physicalcpu.
    for (line, replacement) in [
        ("hw.perflevel1.logicalcpu: 4", "hw.perflevel1.logicalcpu: 3"),
        (
            "hw.perflevel1.physicalcpu: 4",
            "hw.perflevel1.physicalcpu: 3",
        ),
    ] {
        assert!(
            matches!(
                detect_m4_pro_with(line, replacement),
                Err(TopologyError::Invalid { detail }) if detail.contains("performance levels hold")
            ),
            "{replacement}"
        );
    }
    // A level count beyond the recorded levels reads an absent level.
    assert!(matches!(
        detect_m4_pro_with("hw.nperflevels: 2", "hw.nperflevels: 3"),
        Err(TopologyError::MissingSysctl { name }) if name == "hw.perflevel2.logicalcpu"
    ));
    for count in ["0", "9"] {
        assert!(matches!(
            detect_m4_pro_with("hw.nperflevels: 2", &format!("hw.nperflevels: {count}")),
            Err(TopologyError::Invalid { detail }) if detail.contains("hw.nperflevels")
        ));
    }
    assert!(matches!(
        detect_m4_pro_with("hw.packages: 1", "hw.packages: 2"),
        Err(TopologyError::Invalid { detail }) if detail.contains("hw.packages")
    ));
}

#[test]
fn macos_missing_and_malformed_sysctls_are_named() {
    assert!(matches!(
        detect_m4_pro_with("hw.logicalcpu: 14", ""),
        Err(TopologyError::MissingSysctl { name }) if name == "hw.logicalcpu"
    ));
    assert!(matches!(
        detect_m4_pro_with("hw.perflevel1.physicalcpu: 4", ""),
        Err(TopologyError::MissingSysctl { name }) if name == "hw.perflevel1.physicalcpu"
    ));
    assert!(matches!(
        detect_m4_pro_with("hw.memsize: 68719476736", "hw.memsize: lots"),
        Err(TopologyError::SysctlParse { name, .. }) if name == "hw.memsize"
    ));
    assert!(matches!(
        detect_m4_pro_with("hw.physicalcpu: 14", "hw.physicalcpu: 4294967296"),
        Err(TopologyError::SysctlParse { name, detail }) if name == "hw.physicalcpu" && detail.contains("u32")
    ));
    // Optional facts degrade to "unknown", never to a guess.
    let no_memory = detect_m4_pro_with("hw.memsize: 68719476736", "").expect("detect");
    assert_eq!(no_memory.total_memory_bytes, None);
    let no_width = detect_m4_pro_with("hw.perflevel1.cpusperl2: 4", "").expect("detect");
    assert_eq!(
        no_width.l2_domains.len(),
        2,
        "no E-cluster domain without its width"
    );
}

#[test]
fn macos_smt_levels_and_pre_perflevel_hosts() {
    // One level with two threads per core: siblings are adjacent ids.
    let smt = SysctlSnapshot::parse(
        "hw.logicalcpu: 8\nhw.physicalcpu: 4\nhw.nperflevels: 1\n\
         hw.perflevel0.logicalcpu: 8\nhw.perflevel0.physicalcpu: 4\n\
         hw.perflevel0.l2cachesize: 262144\nhw.perflevel0.cpusperl2: 2\n",
    )
    .expect("synthetic listing");
    let t = HardwareTopology::detect_macos(&smt).expect("detect");
    assert!(t.smt_active());
    assert_eq!(t.physical_cores, 4);
    assert_eq!(t.cpus[1].core_id, 0);
    assert_eq!(t.cpus[2].core_id, 1);
    assert!(t.cpus.iter().all(|cpu| cpu.class == PerfClass::Performance));
    assert_eq!(t.l2_domains.len(), 4);
    assert_eq!(t.l2_domains[3].cpus, vec![6, 7]);

    // macOS before performance levels: one uniform level, cache sharing
    // from hw.cacheconfig (memory, L1, L2, L3 widths).
    let legacy = SysctlSnapshot::parse(
        "hw.logicalcpu: 8\nhw.physicalcpu: 4\nhw.cacheconfig: 8 2 2 8 0 0 0 0 0 0\n\
         hw.l2cachesize: 262144\nhw.l3cachesize: 8388608\n",
    )
    .expect("synthetic listing");
    let t = HardwareTopology::detect_macos(&legacy).expect("detect");
    assert_eq!(t.logical_cores(), 8);
    assert!(t.cpus.iter().all(|cpu| cpu.class == PerfClass::Performance));
    assert_eq!(t.l2_domains.len(), 4);
    assert_eq!(t.l3_domains.len(), 1);
    assert_eq!(t.l3_domains[0].cpus.len(), 8);
    assert_eq!(t.l3_domains[0].size_bytes, Some(8 * 1024 * 1024));

    // A level whose threads do not divide evenly over its cores is refused.
    let ragged = SysctlSnapshot::parse(
        "hw.logicalcpu: 3\nhw.physicalcpu: 2\nhw.nperflevels: 1\n\
         hw.perflevel0.logicalcpu: 3\nhw.perflevel0.physicalcpu: 2\n",
    )
    .expect("synthetic listing");
    assert!(matches!(
        HardwareTopology::detect_macos(&ragged),
        Err(TopologyError::Invalid { detail }) if detail.contains("performance level 0")
    ));

    // A level with cores but no logical CPUs is refused too, although zero
    // is a whole multiple of any core count and the levels still sum to the
    // host's totals. Admitted, it would report 10 cores without SMT while
    // its CPU list holds 5 cores of two threads each.
    let hollow = SysctlSnapshot::parse(
        "hw.logicalcpu: 10\nhw.physicalcpu: 10\nhw.nperflevels: 2\n\
         hw.perflevel0.logicalcpu: 10\nhw.perflevel0.physicalcpu: 5\n\
         hw.perflevel1.logicalcpu: 0\nhw.perflevel1.physicalcpu: 5\n",
    )
    .expect("synthetic listing");
    assert!(matches!(
        HardwareTopology::detect_macos(&hollow),
        Err(TopologyError::Invalid { detail }) if detail.contains("performance level 1")
    ));
}

/// Darwin reports how many CPUs share a cache, never which ones. A width that
/// does not divide a level cannot place the remainder, so no domain is
/// recorded for it rather than a guessed short one.
#[test]
fn macos_cache_widths_that_do_not_divide_a_level_record_no_domain() {
    let l2 = |size_mib: u64, cpus: std::ops::Range<u32>| CacheDomain {
        level: 2,
        size_bytes: Some(size_mib * 1024 * 1024),
        cpus: cpus.collect(),
    };

    // The M4 Pro's own hw.cacheconfig says 4 CPUs share an L2, over 14 CPUs
    // whose real clusters are 5 + 5 + 4. Read without its performance levels,
    // that is no domain, not 4 + 4 + 4 + 2.
    let flat = detect_m4_pro_with("hw.nperflevels: 2", "").expect("detect");
    assert_eq!(flat.logical_cores(), 14);
    assert!(flat.l2_domains.is_empty(), "{:?}", flat.l2_domains);

    // A P-level width of 4 over its 10 CPUs: no P domain, the E cluster stays.
    let ragged = detect_m4_pro_with("hw.perflevel0.cpusperl2: 5", "hw.perflevel0.cpusperl2: 4")
        .expect("detect");
    assert_eq!(ragged.l2_domains, vec![l2(4, 10..14)]);

    // A width at least as wide as the level is one domain holding all of it.
    let wider = detect_m4_pro_with("hw.perflevel0.cpusperl2: 5", "hw.perflevel0.cpusperl2: 12")
        .expect("detect");
    assert_eq!(wider.l2_domains, vec![l2(16, 0..10), l2(4, 10..14)]);
}

#[test]
fn sysctl_listings_parse_strictly() {
    for (bad, why) in [
        ("hw.logicalcpu 14\n", "not `name: value`"),
        ("hw logicalcpu: 14\n", "malformed OID name"),
        (
            "hw.logicalcpu: 14\nhw.logicalcpu: 8\n",
            "repeats hw.logicalcpu",
        ),
    ] {
        assert!(
            matches!(
                SysctlSnapshot::parse(bad),
                Err(TopologyError::Invalid { detail }) if detail.contains(why)
            ),
            "{bad:?}"
        );
    }
    let long_value = format!("hw.memsize: {}\n", "9".repeat(4097));
    assert!(SysctlSnapshot::parse(&long_value).is_err());
    let many: String = (0..4097).map(|i| format!("hw.x{i}: 1\n")).collect();
    assert!(matches!(
        SysctlSnapshot::parse(&many),
        Err(TopologyError::Invalid { detail }) if detail.contains("entry limit")
    ));
    // Comments, blank lines and CRLF endings are tolerated.
    let ok = SysctlSnapshot::parse("# header\n\nhw.logicalcpu: 1\r\nhw.physicalcpu: 1\n")
        .expect("parses");
    assert_eq!(
        HardwareTopology::detect_macos(&ok)
            .expect("detect")
            .logical_cores(),
        1
    );
}
