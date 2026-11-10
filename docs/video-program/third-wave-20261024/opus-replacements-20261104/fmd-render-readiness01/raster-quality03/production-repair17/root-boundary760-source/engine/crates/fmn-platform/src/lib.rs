//! Filesystem/process/clock/entropy/AssetFetcher capability traits plus
//! HardwareTopology introspection (§6, §17.4, fm-x68).
//!
//! # The capability doctrine (read this before adding any I/O to any crate)
//!
//! **Every capability is a trait object the engine receives, never an ambient
//! global.** No authoritative fmn crate calls `std::fs`, spawns a process,
//! reads a clock, or touches the network directly — it accepts the relevant
//! trait from this crate ([`fs::FileSystem`], [`process::ProcessRunner`],
//! [`clock::Clock`], [`entropy::HostEntropy`], [`fetch::AssetFetcher`]) and
//! uses only that. The reasons are load-bearing, not stylistic:
//!
//! - **Determinism.** The certified input closure (§16.7, docs/INPUT_CLOSURE.md)
//!   can only enumerate what the engine read if every read goes through a
//!   capability that can record it. An ambient `std::fs::read` is an
//!   unhashable side channel.
//! - **The deterministic lab.** Tests and the replay journal substitute
//!   [`fs::VirtualFs`], [`clock::FakeClock`], [`process::ScriptedRunner`],
//!   and [`fetch::ScriptedFetcher`] to make whole subsystems replayable
//!   without touching the host.
//! - **WASM tiers.** A browser build implements the same traits over
//!   virtual storage; code written against capabilities ports by
//!   construction (R15).
//! - **The Studio's isolated worker** receives narrowed capabilities — the
//!   security model is "hand the worker less", which is only expressible if
//!   everything is handed.
//!
//! Two doctrine points are stricter still:
//!
//! - **Process spawning exists for exactly one program.** ffmpeg is the only
//!   subprocess the engine will ever invoke (D2). [`process`] is the one
//!   sanctioned mechanism and carries the D2 protocol substrate: argv-only
//!   (no shell, ever), a cleared environment plus an explicit allowlist,
//!   timeouts, output-size limits, and kill-on-overrun.
//! - **Network exists only behind [`fetch::AssetFetcher`]** — host-provided,
//!   never implemented in core, no TLS anywhere in the closure (D2). The
//!   in-tree implementations are [`fetch::NoNetwork`] (the default: a named
//!   capability error) and a scripted test double.
//!
//! [`topology`] provides [`topology::HardwareTopology`]: the introspected
//! machine shape (cores, SMT, packages, P/E classes, cache/L3 domains, NUMA
//! nodes, Windows processor groups, SIMD tier, memory) that fmn-runtime's
//! `ExecutionPlan` derivation consumes (§17.4). Introspection itself obeys
//! the doctrine — it reads sysfs *through* [`fs::FileSystem`], so synthetic
//! machines are just fixtures.
#![forbid(unsafe_code)]

pub mod clock;
pub mod entropy;
pub mod fetch;
pub mod fs;
pub mod process;
pub mod profile;
pub mod topology;
