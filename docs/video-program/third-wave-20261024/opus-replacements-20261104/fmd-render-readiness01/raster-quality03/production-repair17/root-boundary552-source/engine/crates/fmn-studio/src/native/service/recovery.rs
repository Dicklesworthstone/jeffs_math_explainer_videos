//! Transactional native command replay and cold checkpoint reconstruction.

use fmn_scene::Journal;
use fmn_scene::studio_bridge::{SceneState, Stage};

use super::super::edits::decode_state;
use super::{NativeSceneWorker, execution_error, refuse_owned};
use crate::{
    Checkpoint, JournalReplay, ServiceError, WorkerErrorCode, WorkerResponse, protocol_digest,
};

impl NativeSceneWorker {
    pub(super) fn replay(&mut self, replay: JournalReplay) -> Result<WorkerResponse, ServiceError> {
        self.require_scene(&replay.scene)?;
        self.require_replay(WorkerErrorCode::ReplayFailed)?;
        let refuse = |message| ServiceError::new(WorkerErrorCode::ReplayFailed, message);
        if replay.from_entry != self.position
            || replay.from_entry > replay.through_entry
            || replay.journal.len() > self.config.limits.max_journal_bytes
        {
            return Err(refuse("native replay range or byte budget is invalid"));
        }
        let journal = Journal::from_bytes(&replay.journal)
            .map_err(|error| refuse_owned(WorkerErrorCode::ReplayFailed, error))?;
        let from = usize::try_from(replay.from_entry)
            .map_err(|_| refuse("native replay start exceeds usize"))?;
        let through = usize::try_from(replay.through_entry)
            .map_err(|_| refuse("native replay end exceeds usize"))?;
        let entries = journal
            .entries()
            .get(from..through)
            .ok_or_else(|| refuse("native replay range exceeds the journal"))?;
        if !journal.events().is_empty() || entries.len() > self.config.limits.max_replay_hashes {
            return Err(refuse(
                "native replay contains unowned input events or too many entries",
            ));
        }
        let backend = self.raster.backend()?;
        if journal
            .render_backends()
            .iter()
            .any(|record| record != &backend)
        {
            return Err(refuse(
                "native replay renderer identity differs from this worker",
            ));
        }
        // Simulate the whole branch/revision history and charge both kinds of
        // callback work before invoking even the first factory. Zero-frame
        // event floods cannot hide behind a nominal zero frame-work total.
        let mut simulated = self.edits.try_clone()?;
        let mut total_frames = 0_u64;
        let mut total_inputs = 0_u64;
        for (offset, entry) in entries.iter().enumerate() {
            if entry.reads != self.reads
                || entry.effect != self.effect()
                || !entry.subprocesses.is_empty()
            {
                return Err(refuse(
                    "native replay closure or effect differs from this factory",
                ));
            }
            let position = replay.from_entry + offset as u64;
            let frame = self
                .prepare_command(&entry.command, position, &mut simulated)
                .map_err(|error| refuse_owned(WorkerErrorCode::ReplayFailed, error))?;
            total_frames = total_frames
                .checked_add(frame)
                .ok_or_else(|| refuse("native replay work overflows"))?;
            total_inputs = total_inputs
                .checked_add(simulated.visible_count(frame))
                .ok_or_else(|| refuse("native input replay work overflows"))?;
            if total_frames > self.config.max_replay_frames
                || total_inputs > self.config.max_replay_inputs
            {
                return Err(refuse(
                    "native replay exceeds its total frame or input work budget",
                ));
            }
            if let Some(state) = &entry.checkpoint
                && (state.len() > self.config.limits.max_checkpoint_bytes
                    || protocol_digest(state) != entry.state_hash)
            {
                return Err(refuse(
                    "native replay checkpoint digest or budget is invalid",
                ));
            }
        }
        let mut edits = self.edits.try_clone()?;
        let mut hashes = Vec::new();
        hashes
            .try_reserve_exact(entries.len())
            .map_err(execution_error)?;
        let mut candidate = None;
        let mut last_checkpoint = self.last_checkpoint;
        for (offset, entry) in entries.iter().enumerate() {
            let position = replay.from_entry + offset as u64;
            let frame = self.prepare_command(&entry.command, position, &mut edits)?;
            let mut program = self
                .fresh_at(frame, &edits)
                .map_err(|error| refuse_owned(WorkerErrorCode::ReplayFailed, error))?;
            let state = self.state_bytes(&mut program, &edits, position + 1)?;
            let hash = protocol_digest(&state);
            if hash != entry.state_hash {
                return Err(refuse(
                    "native replay state diverged; no partial replay was installed",
                ));
            }
            hashes.push(hash);
            if entry.checkpoint.is_some() {
                last_checkpoint = Some(frame);
            }
            candidate = Some(program);
        }
        let mut tail = None;
        if let Some(entry) = entries.last() {
            let mut segment = Journal::new();
            segment
                .record(entry.try_clone().map_err(execution_error)?)
                .map_err(execution_error)?;
            tail = Some(segment.to_bytes().map_err(execution_error)?);
        }
        let last_hash = hashes.last().copied().or(self.last_hash);
        let response = self.checked(WorkerResponse::ReplayComplete {
            from_entry: replay.from_entry,
            state_hashes: hashes,
        })?;
        if let Some(program) = candidate {
            let raster = self.new_raster()?;
            self.program = Some(program);
            self.edits = edits;
            self.pending_inputs.clear();
            self.preview_clean = true;
            self.raster = raster;
        }
        self.position = replay.through_entry;
        self.last_checkpoint = last_checkpoint;
        self.last_hash = last_hash;
        if let Some(tail) = tail {
            self.tail = tail;
        }
        Ok(response)
    }

    pub(super) fn restore(
        &mut self,
        checkpoint: Checkpoint,
    ) -> Result<WorkerResponse, ServiceError> {
        self.require_scene(&checkpoint.scene)?;
        self.require_replay(WorkerErrorCode::CheckpointRejected)?;
        let refuse = |message| ServiceError::new(WorkerErrorCode::CheckpointRejected, message);
        if checkpoint.state.len() > self.config.limits.max_checkpoint_bytes
            || protocol_digest(&checkpoint.state) != checkpoint.state_hash
        {
            return Err(refuse("native checkpoint byte budget or digest is invalid"));
        }
        let next = checkpoint
            .after_entry
            .checked_add(1)
            .ok_or_else(|| refuse("native checkpoint journal position exhausted"))?;
        let restored = decode_state(
            &checkpoint.state,
            &self.config.scene,
            next,
            &self.reads,
            self.config.max_recorded_inputs,
            self.config.frame_count,
            self.config.limits.max_checkpoint_bytes,
        )
        .map_err(|error| refuse_owned(WorkerErrorCode::CheckpointRejected, error))?;
        if !restored.edits.is_empty() && !self.input_enabled() {
            return Err(refuse(
                "edited checkpoint requires native committed input support",
            ));
        }
        let decoded = SceneState::from_bytes(restored.state, &Stage::new())
            .map_err(|error| refuse_owned(WorkerErrorCode::CheckpointRejected, error))?;
        let frame = self
            .target(decoded.frames_elapsed)
            .map_err(|error| refuse_owned(WorkerErrorCode::CheckpointRejected, error))?;
        if decoded.fps != self.config.fps {
            return Err(refuse("native checkpoint uses a different frame clock"));
        }
        drop(decoded); // Never install decoded updater identities as executable state.
        self.require_input_budget(&restored.edits, frame)
            .map_err(|error| refuse_owned(WorkerErrorCode::CheckpointRejected, error))?;
        let mut program = self.fresh_at(frame, &restored.edits)?;
        let state = self.state_bytes(&mut program, &restored.edits, next)?;
        if state != checkpoint.state {
            return Err(refuse(
                "native checkpoint does not match fresh callback and input execution",
            ));
        }
        let response = self.checked(WorkerResponse::Ack {
            state_hash: Some(checkpoint.state_hash),
            journal_len: next,
        })?;
        let raster = self.new_raster()?;
        self.program = Some(program);
        self.edits = restored.edits;
        self.pending_inputs.clear();
        self.preview_clean = true;
        self.raster = raster;
        self.position = next;
        self.last_hash = Some(checkpoint.state_hash);
        self.last_checkpoint = Some(frame);
        self.tail.clear();
        Ok(response)
    }
}
