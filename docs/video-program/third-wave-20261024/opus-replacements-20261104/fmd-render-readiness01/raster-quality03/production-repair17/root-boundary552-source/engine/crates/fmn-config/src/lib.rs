//! YAML-subset parser, typed configuration, and the preamble-pack registry (§6.4, §13.6).
//!
//! Three layers, strictly ordered:
//!
//! - [`yaml`] — the owned parser for the actual shipped config-file shapes
//!   (see its module docs for the precise subset and the named diagnostics
//!   for everything outside it), plus the Reference-exact recursive merge.
//! - [`config`] — the typed configuration: the Reference's
//!   `default_config.yml` key surface, resolved through the Reference's
//!   precedence exactly — **built-in defaults → user config file(s) → CLI
//!   overlay** — with tuple-strings typed at this layer, the way the
//!   Reference `literal_eval`s them. The *extraction* itself is generated:
//!   `generated.rs` is produced from the one API schema (§16.2, fm-vn6), so
//!   the config surface has a single definition instead of a document and a
//!   hand-written reader that can drift apart. Adding a key to
//!   `default_config.yml` without binding it in `API_OVERLAY.tsv` fails the
//!   coverage check; hand-editing `generated.rs` fails the drift gate.
//! - [`packs`] — the preamble-pack registry: the `tex_templates.yml` concept
//!   reborn as named fmd-math preamble packs, with the compatibility mapping
//!   for the common templates. This crate owns the registry *surface*
//!   (naming, lookup, config keys); pack *content* is W6's business
//!   (fm-kg9).
//!
//! # The quality-knob doctrine (§4)
//!
//! [`config::DeterminismConfig`] and [`config::RenderConfig`] carry the
//! determinism-mode and engine/backend keys (`standard | certified`;
//! `cpu | metal | cuda`; AA policy; thread policy) as opaque typed enums.
//! They select engines and schedules — they are **never scene-visible data**:
//! no path exists from these knobs into mobject state, animation timing, or
//! any other semantic surface, so a quality/backend change can change speed
//! but structurally cannot change meaning.
#![forbid(unsafe_code)]

pub mod config;
mod generated;
pub mod packs;
pub mod yaml;

pub use config::{Config, ConfigError};
pub use packs::{Pack, PackError, PackRegistry};
pub use yaml::{ParseError, Value, Warning};
