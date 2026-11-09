# ADR-0024 — Qualify an isolated 8-core bare-metal slice; Apple profile gates only Apple-named rows

**Status:** Accepted (owner ratified 2026-09-24: "approved, proceed")
**Date:** 2026-09-24
**Bead:** fm-5wq.8
**Amends:** policy under §17.2 ("pinned bare-metal profiles") as implemented by `crates/fmn-conformance/src/perf_host.rs` and `docs/performance/PERFORMANCE_GATES.md`

## Context

Every open gate from G2 onward waits, directly or through G2, on performance evidence that no reachable machine can produce. As of 2026-09-24 the repository records **no** FrankenManim timing for any PG gate — not qualified, not calibration.

- `perf_host.rs:41/680` requires the **whole machine** to have exactly 8 physical cores. The plan (§17.2) says "an 8-core x86-64 Linux box"; the code hardened that into a machine-shape rule. The bare-metal dev box (dual EPYC 7282, 32 physical cores) and ts2 (64) are refused on core count alone.
- `perf_host.rs:436/473/573` refuse macOS outright: a native topology/power probe needs `sysctl`, which D-02 forbids as a subprocess and D3 forbids through FFI in an authoritative crate.
- `PERFORMANCE_GATES.md:285-288` requires both the Linux and the macOS profile before fm-inr.1 or any whole-gate claim closes.

The same verifier already enforces what actually protects a measurement (`validate_linux_live_state`, `require_eight_distinct_cores`): no hypervisor and no DMI VM marker; the benchmark CPU set is exactly 8 distinct physical cores; those CPUs are in `isolcpus`, `nohz_full` and `rcu_nocbs`; a dedicated cgroup-v2 cpuset equals that set and contains only the measurement process; the scaling governor and boost state match the profile. The whole-machine count adds no protection those checks do not already give, and it excludes every bare-metal host the program owns.

## Decision

1. **Linux profile.** A host qualifies when the existing isolation checks pass for a benchmark set of exactly 8 distinct physical cores, on bare metal, **whatever the machine's total core count**. Added requirements: the benchmark cores and their SMT siblings are all isolated and the siblings carry no work; the cores lie in one NUMA node and the cgroup's `cpuset.mems` binds memory to it; the profile records the machine's total topology, and the attestation records the load on non-benchmark cores during the run. `LINUX_PHYSICAL_CORES` stays 8 as the size of the benchmark set, not of the machine.
2. **Apple profile.** No macOS host qualifies until a probe exists that D-02/D3 admit. macOS observations are recorded as calibration and never feed a core gate verdict. The Apple profile is required only for rows the plan names for Apple silicon: PG-A and G3's annex-preview criterion.
3. **Gate wording.** G2's PG-1(G2) and PG-7, and every core PG row, close on a qualified Linux observation. `PERFORMANCE_GATES.md`'s "both declared Linux and macOS profiles" becomes "the Linux profile for core rows; the Apple profile for Apple-named rows".

## Win/lose split of what the fix admits

| Host | Today | Under this ADR |
|---|---|---|
| dev box, dual EPYC 7282, bare metal | refused (32 cores) | qualifies once booted with `isolcpus/nohz_full/rcu_nocbs` covering 8 cores + siblings of one node, and a dedicated cgroup |
| ts2, 64 cores, bare metal | refused (64 cores) | qualifies under the same setup |
| owner's 5995WX, bare metal | refused (core count) | qualifies under the same setup |
| RCH workers (virtualized) | refused (hypervisor) | still refused |
| Mac M4 Pro | refused (no probe) | still refused; calibration only |

No host admitted today becomes refused. What an isolated slice cannot rule out is contention in shared resources (L3, memory bandwidth, thermal headroom) from work on the other cores. Two things mitigate it. The attestation records non-benchmark load. And a qualified run requires the rest of the machine to be quiescent, which PERFORMANCE_GATES.md sets as a numeric threshold before the verifier change lands.

## Consequences

- Unblocks qualified PG-1/PG-2/PG-3/PG-4/PG-5-timing/PG-6/PG-7 observations on hardware the program owns. That removes the structural stall on G2.
- Needs owner action on the host: a kernel boot parameter change and a cgroup setup. The verifier change (`perf_host.rs` `isolation_plan`/`require_quiescence`, attestation schema `/3`, monitor policy `-v2`) keeps every existing strictness test and adds a negative control per new requirement.
- "The attestation records the load on non-benchmark cores during the run" is implemented as an enforced ceiling rather than a recorded number: the attestation's content digest is bound into the measurement batch before the workload starts, so it cannot carry a run-window value. The monitor instead fails the run closed when the siblings exceed 10‰ or the rest of the machine 50‰ of their CPU time, judged cumulatively over the whole window, and the attestation records both ceilings.
- Plan §17.2 and PERFORMANCE_GATES.md are trued up in the same commit that marks this ADR Accepted.
