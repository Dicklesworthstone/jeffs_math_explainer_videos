//! G0-1: the object-model & buffer-lifetime spike (fm-dzv).
//!
//! A compiling prototype ratifying (or amending) §8.1–8.2 and §15.1 before
//! Marionette's interfaces freeze — decision D-11. The ten lifetime
//! scenarios live in `tests/scenarios.rs`; the ratification note is
//! `docs/g0/G0-1-object-model-ratification.md`, and the view-protocol rules
//! are stated in [`record`]'s module docs precisely enough for W3 to
//! implement from that text alone.
//!
//! This crate is a **spike**: throwaway by charter, kept compiling so the
//! ratified decisions stay executable. W3 (fm-ce8, fm-cus) re-implements
//! the model production-grade inside fmn-mobject; nothing links against
//! this crate.
#![forbid(unsafe_code)]

pub mod animate;
pub mod mobject;
pub mod record;
pub mod stage;

pub use animate::{AnimBuilder, Command, StaleHandle};
pub use mobject::{Mobject, Square};
pub use record::{RecordBuffer, RecordSchema, RecordView};
pub use stage::{Entry, Mob, Snapshot, Stage};
