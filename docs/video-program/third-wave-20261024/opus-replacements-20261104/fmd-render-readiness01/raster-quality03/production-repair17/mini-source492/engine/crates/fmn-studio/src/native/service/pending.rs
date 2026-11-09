//! Explicit Save for browser input, using the existing committed-seek protocol.
//! Preview events remain uncommitted until a same-frame Play seek saves them.

use super::{NativeSceneWorker, execution_error, invalid};
use crate::protocol::{StudioInput, studio_input_command, studio_seek_frame};
use crate::{ServiceError, WorkerResponse, protocol_digest};
use fmn_scene::{CommandKind, CommandRecord, Entry, EventPayload, Journal};

impl NativeSceneWorker {
    pub(super) fn staged_command(
        &mut self,
        event: &EventPayload,
    ) -> Result<Option<CommandRecord>, ServiceError> {
        if !self.config.stage_input_events || !self.input_enabled() {
            return Ok(None);
        }
        let frame = self
            .program
            .as_ref()
            .ok_or_else(|| invalid("seek before staging native input"))?
            .frame_index();
        let count = self
            .edits
            .visible_count(frame)
            .checked_add(self.pending_inputs.len() as u64)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| invalid("native staged input count exhausted"))?;
        if count > self.config.max_recorded_inputs as u64 || count > self.config.max_replay_inputs {
            return Err(invalid(
                "native staged input exceeds its event budget; save or discard first",
            ));
        }
        let revision = self
            .position
            .checked_add(self.pending_inputs.len() as u64)
            .ok_or_else(|| invalid("native staged input revision exhausted"))?;
        let command = studio_input_command(
            &self.config.scene,
            &StudioInput {
                frame: frame as i64,
                revision,
                target: None,
                event: event.clone(),
            },
        )
        .map_err(execution_error)?;
        self.pending_inputs
            .try_reserve(1)
            .map_err(execution_error)?;
        Ok(Some(command))
    }

    pub(super) fn saves_pending(&self, command: &CommandRecord) -> Result<bool, ServiceError> {
        if self.pending_inputs.is_empty() || command.kind == CommandKind::Input {
            return Ok(false);
        }
        let frame = self.target(
            studio_seek_frame(&self.config.scene, command)
                .map_err(|error| invalid(error.to_string()))?,
        )?;
        Ok(self
            .program
            .as_ref()
            .is_some_and(|program| program.frame_index() == frame))
    }

    /// Construct one atomic journal segment for the staged input batch and its
    /// save command. Re-execute the source only once, compare with the displayed
    /// owner's actual state, and retain just the final checkpoint in the batch.
    /// No partial journal/track is installed on any ordinary returned error.
    pub(super) fn save_pending(
        &mut self,
        command: CommandRecord,
    ) -> Result<WorkerResponse, ServiceError> {
        let frame =
            self.target(studio_seek_frame(&self.config.scene, &command).map_err(execution_error)?)?;
        let next = self
            .position
            .checked_add(self.pending_inputs.len() as u64)
            .and_then(|n| n.checked_add(1))
            .ok_or_else(|| invalid("native input batch position exhausted"))?;
        let mut simulated = self.edits.try_clone()?;
        for (offset, command) in self.pending_inputs.iter().enumerate() {
            let at =
                self.prepare_command(command, self.position + offset as u64, &mut simulated)?;
            if at != frame {
                return Err(invalid("native input batch spans different preview frames"));
            }
        }
        self.require_input_budget(&simulated, frame)?;
        let displayed = self
            .program
            .as_mut()
            .ok_or_else(|| invalid("native preview is unavailable"))?
            .state_bytes()?;
        let mut program = self.fresh_at(frame, &self.edits)?;
        let mut edits = self.edits.try_clone()?;
        let mut journal = Journal::new();
        for (offset, command) in self.pending_inputs.iter().enumerate() {
            let position = self.position + offset as u64;
            self.prepare_command(command, position, &mut edits)?;
            let input = crate::protocol::studio_input_payload(&self.config.scene, command)
                .map_err(execution_error)?;
            program.dispatch(input.event)?;
            let state = self.state_bytes(&mut program, &edits, position + 1)?;
            journal
                .record(Entry {
                    command: command.clone(),
                    effect: self.effect(),
                    reads: self.reads.clone(),
                    subprocesses: Vec::new(),
                    checkpoint: None,
                    state_hash: protocol_digest(&state),
                })
                .map_err(execution_error)?;
        }
        if program.state_bytes()? != displayed {
            return Err(execution_error(
                "staged native input diverged from fresh execution; preview was not saved",
            ));
        }
        let state = self.state_bytes(&mut program, &edits, next)?;
        let state_hash = protocol_digest(&state);
        let mut raster = self.new_raster()?;
        let frame_response = raster.frame(&self.config.scene, &program)?;
        self.checked(frame_response)?;
        journal
            .record(Entry {
                command,
                effect: self.effect(),
                reads: self.reads.clone(),
                subprocesses: Vec::new(),
                checkpoint: Some(state),
                state_hash,
            })
            .map_err(execution_error)?;
        let bytes = journal.to_bytes().map_err(execution_error)?;
        let response = self.checked(WorkerResponse::JournalSegment {
            scene: self.config.scene.clone(),
            start_entry: self.position,
            journal: bytes.clone(),
        })?;
        self.program = Some(program);
        self.edits = edits;
        self.preview_clean = true;
        self.pending_inputs.clear();
        self.raster = raster;
        self.position = next;
        self.last_hash = Some(state_hash);
        self.last_checkpoint = Some(frame);
        self.tail = bytes;
        Ok(response)
    }
}
