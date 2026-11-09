//! ValueTrackers, `always_redraw`/`f_always` closure binding, and the C-6
//! group-addition correction (§8.6, fm-yra).
//!
//! # ValueTrackers
//!
//! The Reference's `ValueTracker` is "a mobject not meant to be displayed":
//! the value rides the mobject machinery so it can be animated like anything
//! else. It stores the encoded value in `uniforms["value"]` as float64 /
//! complex128 — full double precision, deliberately *not* the f32 record
//! plane. Here that is the entry's typed [`Tracker`] state ([`TrackerKind`]
//! plus up to two f64 lanes), with the same encodings:
//!
//! - [`TrackerKind::Plain`] — one lane, the value itself;
//! - [`TrackerKind::Exponential`] — one lane holding `ln(value)`, so
//!   interpolation of the lane is geometric interpolation of the value
//!   (`ExponentialValueTracker`);
//! - [`TrackerKind::Complex`] — two lanes, re/im (`ComplexValueTracker`).
//!
//! `increment` is `set(get + d)` through the encoding, exactly as the
//! Reference implements it (so incrementing an exponential tracker
//! multiplies the stored exponential's argument correctly).
//!
//! # `always_redraw` / `f_always`
//!
//! `always_redraw(f)` binds a rebuild closure into the clock: every
//! [`Stage::update`] tick replaces the returned mobject's content with a
//! fresh `f()` result. The returned handle is a stable container whose
//! children are swapped per tick, allowing the rebuilt family to change
//! shape. Replaced private content is reclaimed after the new family is
//! installed; shared, rooted, and reused content stays alive. `f_always` is the
//! named-parity form of a non-dt updater: in Rust a closure already closes
//! over its argument generators.
//!
//! # C-6: group addition is a value operation
//!
//! The Reference's `Mobject.__add__` builds a NEW group from its operands,
//! but `Group.__add__` mutates the left operand in place and returns it —
//! two different semantics under one operator (Appendix C, C-6).
//! [`Stage::group_add`] always builds a new group (Behavior Note
//! BN-07-updater-and-group-fixes).

use crate::StageError;
use crate::mobject::Mobject;
use crate::stage::{IdBuildHasher, Mob, Stage, UpdaterId};
use std::collections::HashSet;

/// Which tracker encoding a mobject carries.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrackerKind {
    /// `ValueTracker`: one lane, value stored directly.
    Plain,
    /// `ExponentialValueTracker`: one lane, `ln(value)` stored.
    Exponential,
    /// `ComplexValueTracker`: two lanes, re/im.
    Complex,
}

/// ValueTracker state: the encoding kind plus the stored f64 lanes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Tracker {
    /// The encoding.
    pub kind: TrackerKind,
    /// Stored lanes (`[value, 0]`, `[ln(value), 0]`, or `[re, im]`).
    pub lanes: [f64; 2],
}

impl Stage {
    // ----------------------------------------------------------- trackers

    fn add_tracker(&mut self, kind: TrackerKind, lanes: [f64; 2]) -> Mob {
        let mob = self.add(Mobject::new());
        if let Some(entry) = self.get_mut(mob) {
            entry.tracker = Some(Tracker { kind, lanes });
        }
        mob
    }

    /// A `ValueTracker` holding `value`.
    pub fn add_value_tracker(&mut self, value: f64) -> Mob {
        self.add_tracker(TrackerKind::Plain, [value, 0.0])
    }

    /// An `ExponentialValueTracker` holding `value` (stored as `ln(value)`,
    /// so lane interpolation is geometric).
    pub fn add_exponential_value_tracker(&mut self, value: f64) -> Mob {
        self.add_tracker(TrackerKind::Exponential, [fmn_dmath::ln(value), 0.0])
    }

    /// A `ComplexValueTracker` holding `re + im·i`.
    pub fn add_complex_value_tracker(&mut self, re: f64, im: f64) -> Mob {
        self.add_tracker(TrackerKind::Complex, [re, im])
    }

    /// The raw tracker state, if `mob` is a tracker.
    #[must_use]
    pub fn tracker(&self, mob: Mob) -> Option<Tracker> {
        self.get(mob).and_then(|e| e.tracker)
    }

    /// Replace the raw typed tracker payload attached to `mob`.
    ///
    /// This is the state-transfer seam used by binding-tier detached-copy
    /// restoration and by Choreo's Transform interpolation.  Ordinary users
    /// should prefer the decoded scalar/complex accessors below; this method
    /// deliberately preserves the encoded lane representation (notably the
    /// logarithmic lane of [`TrackerKind::Exponential`]).
    ///
    /// # Errors
    /// [`StageError::StaleHandle`] if `mob` is not live.
    pub fn set_tracker_state(
        &mut self,
        mob: Mob,
        tracker: Option<Tracker>,
    ) -> Result<(), StageError> {
        self.get_mut(mob).ok_or(StageError::StaleHandle)?.tracker = tracker;
        Ok(())
    }

    /// `get_value` for scalar trackers (`Plain` decodes directly,
    /// `Exponential` decodes through `exp`); `None` for non-trackers and
    /// complex trackers (use [`Stage::tracker_complex_value`]).
    #[must_use]
    pub fn tracker_value(&self, mob: Mob) -> Option<f64> {
        match self.tracker(mob)? {
            Tracker {
                kind: TrackerKind::Plain,
                lanes,
            } => Some(lanes[0]),
            Tracker {
                kind: TrackerKind::Exponential,
                lanes,
            } => Some(fmn_dmath::exp(lanes[0])),
            Tracker {
                kind: TrackerKind::Complex,
                ..
            } => None,
        }
    }

    /// `get_value` for complex trackers.
    #[must_use]
    pub fn tracker_complex_value(&self, mob: Mob) -> Option<(f64, f64)> {
        match self.tracker(mob)? {
            Tracker {
                kind: TrackerKind::Complex,
                lanes,
            } => Some((lanes[0], lanes[1])),
            _ => None,
        }
    }

    /// `set_value` for scalar trackers, through the encoding.
    ///
    /// # Errors
    /// [`StageError::StaleHandle`] if `mob` is dead or not a scalar tracker.
    pub fn set_tracker_value(&mut self, mob: Mob, value: f64) -> Result<(), StageError> {
        let entry = self.get_mut(mob).ok_or(StageError::StaleHandle)?;
        match &mut entry.tracker {
            Some(t) if t.kind == TrackerKind::Plain => {
                t.lanes[0] = value;
                Ok(())
            }
            Some(t) if t.kind == TrackerKind::Exponential => {
                t.lanes[0] = fmn_dmath::ln(value);
                Ok(())
            }
            _ => Err(StageError::StaleHandle),
        }
    }

    /// `set_value` for complex trackers.
    ///
    /// # Errors
    /// [`StageError::StaleHandle`] if `mob` is dead or not a complex tracker.
    pub fn set_tracker_complex_value(
        &mut self,
        mob: Mob,
        re: f64,
        im: f64,
    ) -> Result<(), StageError> {
        let entry = self.get_mut(mob).ok_or(StageError::StaleHandle)?;
        match &mut entry.tracker {
            Some(t) if t.kind == TrackerKind::Complex => {
                t.lanes = [re, im];
                Ok(())
            }
            _ => Err(StageError::StaleHandle),
        }
    }

    /// `increment_value`: `set(get + d)` through the encoding (the
    /// Reference's exact composition, so exponential trackers increment the
    /// decoded value, not the stored log).
    ///
    /// # Errors
    /// [`StageError::StaleHandle`] if `mob` is dead or not a scalar tracker.
    pub fn increment_tracker_value(&mut self, mob: Mob, d_value: f64) -> Result<(), StageError> {
        let current = self.tracker_value(mob).ok_or(StageError::StaleHandle)?;
        self.set_tracker_value(mob, current + d_value)
    }

    /// Increment a complex tracker's decoded real and imaginary components.
    ///
    /// # Errors
    /// [`StageError::StaleHandle`] for a dead or non-complex tracker.
    pub fn increment_tracker_complex_value(
        &mut self,
        mob: Mob,
        real: f64,
        imaginary: f64,
    ) -> Result<(), StageError> {
        let (re, im) = self
            .tracker_complex_value(mob)
            .ok_or(StageError::StaleHandle)?;
        self.set_tracker_complex_value(mob, re + real, im + imaginary)
    }

    // ---------------------------------------------- closure-clock binding

    /// `always_redraw`: build a mobject from `f` now, and rebuild it on
    /// every update tick. The returned handle is stable across rebuilds
    /// (it is the container whose content is swapped); attach it, position
    /// it, and play against it freely.
    ///
    /// The factory runs while the previous content is alive. After a
    /// successful replacement, old members are reclaimed unless they belong
    /// to the new family, remain scene roots, or have parents outside the
    /// replaced family. Retained members keep their complete descendants.
    /// A cached raw [`Mob`] alone does not retain retired content; pinned
    /// retired members follow [`Stage::delete`]'s deferred destruction rules.
    ///
    /// # Panics
    /// Panics with a [`StageError`] if the factory returns a stale handle or
    /// a replacement would create a cycle, or updater registration fails.
    /// A refused replacement leaves the published children unchanged; the
    /// factory's own mutations are not rolled back.
    pub fn always_redraw(&mut self, f: impl Fn(&mut Stage) -> Mob + 'static) -> Mob {
        let first = f(self);
        self.try_get(first)
            .unwrap_or_else(|error| std::panic::panic_any(error));
        let container = self.add(Mobject::new());
        self.attach(container, first)
            .unwrap_or_else(|error| std::panic::panic_any(error));
        self.add_updater(
            container,
            move |stage, me| {
                // Capture before the factory: it may reuse descendants and
                // remove their old wrapper while building the replacement.
                let old: Vec<_> = stage.family(me).into_iter().skip(1).collect();
                let old_members: HashSet<Mob, IdBuildHasher> = old.iter().copied().collect();
                let fresh = f(stage);
                stage
                    .replace_children(me, &[fresh])
                    .unwrap_or_else(|error| std::panic::panic_any(error));

                let mut pending: Vec<_> = stage
                    .roots()
                    .iter()
                    .copied()
                    .filter(|root| old_members.contains(root))
                    .collect();
                pending.push(fresh);
                for &member in &old {
                    if stage.get(member).is_some_and(|entry| {
                        entry
                            .parents()
                            .iter()
                            .any(|parent| !old_members.contains(parent))
                    }) {
                        pending.push(member);
                    }
                }
                let mut retained: HashSet<Mob, IdBuildHasher> = HashSet::default();
                while let Some(member) = pending.pop() {
                    if !retained.insert(member) {
                        continue;
                    }
                    if let Some(entry) = stage.get(member) {
                        pending.extend_from_slice(entry.submobjects());
                    }
                }

                // Decide retention before deleting anything: a diamond or
                // a pinned, doomed parent must not keep private descendants
                // alive merely because its other edge still exists.
                for member in old {
                    if !retained.contains(&member) && stage.contains(member) {
                        // Deletion also removes a whole family from the
                        // scene. Sever outgoing edges first so a retired
                        // wrapper cannot hide a retained shared descendant.
                        stage
                            .replace_children(member, &[])
                            .unwrap_or_else(|error| std::panic::panic_any(error));
                        stage
                            .delete(member)
                            .unwrap_or_else(|error| std::panic::panic_any(error));
                    }
                }
            },
            false,
        )
        .unwrap_or_else(|error| std::panic::panic_any(error));
        container
    }

    /// `f_always`: run `f(stage, mob)` on every update tick — the
    /// named-parity form of [`Stage::add_updater`] (in the Reference,
    /// `f_always` exists to bind argument *generators*; a Rust closure
    /// already closes over them).
    ///
    /// # Errors
    /// [`StageError::StaleHandle`].
    pub fn f_always(
        &mut self,
        mob: Mob,
        f: impl FnMut(&mut Stage, Mob) + 'static,
    ) -> Result<UpdaterId, StageError> {
        self.add_updater(mob, f, false)
    }

    // ------------------------------------------------------ C-6: grouping

    /// Group addition as a value operation: ALWAYS builds a new group
    /// containing `a` and `b` — including when `a` is itself a group
    /// (where the Reference's `Group.__add__` mutates `a` in place and
    /// returns it, diverging from `Mobject.__add__`; C-6, Behavior Note).
    ///
    /// # Errors
    /// [`StageError::StaleHandle`] if either operand is dead.
    pub fn group_add(&mut self, a: Mob, b: Mob) -> Result<Mob, StageError> {
        if !self.contains(a) || !self.contains(b) {
            return Err(StageError::StaleHandle);
        }
        let group = self.add(Mobject::new());
        self.attach(group, a)?;
        self.attach(group, b)?;
        Ok(group)
    }
}
