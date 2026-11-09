//! Studio timeline scrubbing over Choreo's one authoritative seek path.
//!
//! A transient scrub freezes the current Stage, calls [`Timeline::seek`],
//! and restores the frozen Stage before returning the immutable packet.
//! A committed seek leaves the reconstructed state live. Both modes therefore
//! use the same animation drivers, rational clock, checkpoints, and purity
//! rules as ordinary playback.

use fmn_core::rng::RngRoot;
use fmn_scene::studio_bridge::{AnimError, FramePacket, Stage, Timeline};

/// Whether a scrub changes the live scene state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrubMode {
    /// Produce a preview packet and restore the prior Stage.
    Preview,
    /// Leave the target frame's state live.
    Commit,
}

/// Result of one timeline scrub.
pub struct ScrubResult {
    packet: FramePacket,
    mode: ScrubMode,
    checkpointed_segments: Vec<usize>,
}

impl ScrubResult {
    /// Immutable target frame.
    #[must_use]
    pub fn packet(&self) -> &FramePacket {
        &self.packet
    }

    /// Consume into the immutable target frame.
    #[must_use]
    pub fn into_packet(self) -> FramePacket {
        self.packet
    }

    /// Preview or committed operation.
    #[must_use]
    pub const fn mode(&self) -> ScrubMode {
        self.mode
    }

    /// Checkpoints available after the seek.
    #[must_use]
    pub fn checkpointed_segments(&self) -> &[usize] {
        &self.checkpointed_segments
    }
}

/// Seek a timeline with explicit preview-vs-commit semantics.
///
/// Checkpoint construction remains cached on the timeline in both modes; it is
/// a deterministic optimization. Only the live Stage is rolled back for a
/// preview.
pub fn scrub_timeline(
    timeline: &mut Timeline,
    stage: &mut Stage,
    rng: &RngRoot,
    frame: i64,
    mode: ScrubMode,
) -> Result<ScrubResult, AnimError> {
    let packet = match mode {
        ScrubMode::Preview => {
            let before = stage.snapshot();
            let before_time = stage.time();
            let result = timeline.seek(stage, rng, frame);
            stage.restore(&before);
            stage.set_time_from_clock(before_time);
            result?
        }
        ScrubMode::Commit => timeline.seek(stage, rng, frame)?,
    };
    Ok(ScrubResult {
        packet,
        mode,
        checkpointed_segments: timeline.checkpointed_segments(),
    })
}

/// Preview convenience wrapper.
pub fn preview_timeline_frame(
    timeline: &mut Timeline,
    stage: &mut Stage,
    rng: &RngRoot,
    frame: i64,
) -> Result<ScrubResult, AnimError> {
    scrub_timeline(timeline, stage, rng, frame, ScrubMode::Preview)
}

/// Committed-seek convenience wrapper.
pub fn commit_timeline_frame(
    timeline: &mut Timeline,
    stage: &mut Stage,
    rng: &RngRoot,
    frame: i64,
) -> Result<ScrubResult, AnimError> {
    scrub_timeline(timeline, stage, rng, frame, ScrubMode::Commit)
}
