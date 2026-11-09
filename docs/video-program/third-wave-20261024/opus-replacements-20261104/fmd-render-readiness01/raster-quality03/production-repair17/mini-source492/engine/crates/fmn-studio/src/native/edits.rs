//! Canonical, bounded input tracks. Executable/editor state is reconstructed,
//! never deserialized as closures or replaced with a second editing engine.

use fmn_hash::{Digest, Limits, Reader, Schema, UnknownPolicy, Writer};
use fmn_scene::{AssetRead, CommandKind, CommandRecord};

use crate::ServiceError;
use crate::protocol::{StudioInput, studio_input_payload};

use super::NativeSceneProgram;
use super::program::{execution_error, invalid};

const EDIT_STATE_SCHEMA: Schema = Schema::new(*b"FMEI", 1, 2, 0);
/// Hard ceiling independent of caller policy; each label is separately bounded.
pub(super) const MAX_RECORDED_INPUTS: usize = 65_536;

struct InputRow {
    command: CommandRecord,
    input: StudioInput,
}

#[derive(Default)]
pub(super) struct EditTrack {
    rows: Vec<InputRow>,
}

impl EditTrack {
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn try_clone(&self) -> Result<Self, ServiceError> {
        let mut rows = Vec::new();
        rows.try_reserve_exact(self.rows.len())
            .map_err(execution_error)?;
        for row in &self.rows {
            let mut label = String::new();
            label
                .try_reserve_exact(row.command.label.len())
                .map_err(execution_error)?;
            label.push_str(&row.command.label);
            rows.push(InputRow {
                command: CommandRecord {
                    kind: row.command.kind,
                    identity: row.command.identity,
                    label,
                },
                input: row.input.clone(),
            });
        }
        Ok(Self { rows })
    }

    /// A new edit in the past abandons only the later input branch. Existing
    /// events at this frame keep their serial order. Revision is journal length,
    /// not frame number, so even two same-frame clients cannot overwrite edits.
    pub fn append(
        &mut self,
        scene: &str,
        command: &CommandRecord,
        position: u64,
        limit: usize,
    ) -> Result<u64, ServiceError> {
        let input = studio_input_payload(scene, command).map_err(execution_error)?;
        if input.revision != position {
            return Err(invalid("stale native input revision"));
        }
        if input.target.is_some() {
            return Err(invalid(
                "stale native input object or unsupported explicit target",
            ));
        }
        let keep = self
            .rows
            .partition_point(|row| row.input.frame <= input.frame);
        if keep >= limit {
            return Err(invalid(
                "native committed input track exceeds its event budget",
            ));
        }
        let mut label = String::new();
        label
            .try_reserve_exact(command.label.len())
            .map_err(execution_error)?;
        label.push_str(&command.label);
        self.rows.try_reserve(1).map_err(execution_error)?;
        self.rows.truncate(keep);
        let frame = u64::try_from(input.frame).map_err(execution_error)?;
        self.rows.push(InputRow {
            command: CommandRecord {
                kind: command.kind,
                identity: command.identity,
                label,
            },
            input,
        });
        Ok(frame)
    }

    pub fn visible_count(&self, target: u64) -> u64 {
        self.rows
            .partition_point(|row| row.input.frame.cast_unsigned() <= target) as u64
    }

    /// Reuse the one native dispatcher and actual stepped execution boundary.
    /// None means a fresh frame-zero owner (including frame-zero inputs).
    /// Some(frame) means that capture's inputs have already been applied.
    pub fn advance(
        &self,
        program: &mut NativeSceneProgram,
        target: u64,
        after: Option<u64>,
    ) -> Result<(), ServiceError> {
        for row in &self.rows {
            let frame = row.input.frame.cast_unsigned();
            if frame > target {
                break;
            }
            if after.is_some_and(|previous| frame <= previous) {
                continue;
            }
            program.advance_to(frame)?;
            program.dispatch(row.input.event.clone())?;
        }
        program.advance_to(target)
    }
}

/// Empty tracks keep the original SceneState wire format. An edited checkpoint
/// owns its full branch (including inputs after a backward scrub), the actual
/// SceneState at its capture, the input closure, and its exact journal position.
pub(super) fn encode_state(
    scene: &str,
    position: u64,
    reads: &[AssetRead],
    edits: &EditTrack,
    state: Vec<u8>,
    max_bytes: usize,
) -> Result<Vec<u8>, ServiceError> {
    if state.len() > max_bytes {
        return Err(execution_error(
            "native SceneState exceeds the checkpoint budget",
        ));
    }
    if edits.is_empty() {
        return Ok(state);
    }
    let mut writer = Writer::with_limits(
        EDIT_STATE_SCHEMA,
        Limits {
            max_total: max_bytes,
            max_field: max_bytes,
        },
    );
    writer
        .put_str(scene)
        .put_u64(position)
        .put_u32(reads.len() as u32);
    for read in reads {
        writer.put_str(&read.path).put_bytes(read.digest.as_bytes());
    }
    writer.put_u32(edits.rows.len() as u32);
    for row in &edits.rows {
        writer
            .put_bytes(row.command.identity.as_bytes())
            .put_str(&row.command.label);
    }
    writer.put_bytes(&state);
    writer.finish().map_err(execution_error)
}

pub(super) struct RestoredTrack<'a> {
    pub edits: EditTrack,
    pub state: &'a [u8],
}

pub(super) fn decode_state<'a>(
    bytes: &'a [u8],
    scene: &str,
    position: u64,
    reads: &[AssetRead],
    max_inputs: usize,
    frame_count: u64,
    max_bytes: usize,
) -> Result<RestoredTrack<'a>, ServiceError> {
    if bytes.len() > max_bytes {
        return Err(invalid("native checkpoint exceeds its byte budget"));
    }
    if !bytes.starts_with(b"FMEI") {
        // The caller still validates this through SceneState's strict reader.
        return Ok(RestoredTrack {
            edits: EditTrack::default(),
            state: bytes,
        });
    }
    let mut reader = Reader::open(
        bytes,
        EDIT_STATE_SCHEMA,
        Limits {
            max_total: max_bytes,
            max_field: max_bytes,
        },
        UnknownPolicy::Strict,
    )
    .map_err(execution_error)?;
    if reader.get_str().map_err(execution_error)? != scene
        || reader.get_u64().map_err(execution_error)? != position
    {
        return Err(invalid(
            "edited checkpoint scene or journal position differs",
        ));
    }
    if reader.get_u32().map_err(execution_error)? as usize != reads.len() {
        return Err(invalid("edited checkpoint input closure differs"));
    }
    for read in reads {
        if reader.get_str().map_err(execution_error)? != read.path
            || reader.get_bytes().map_err(execution_error)? != read.digest.as_bytes()
        {
            return Err(invalid("edited checkpoint input closure differs"));
        }
    }
    let count = reader.get_u32().map_err(execution_error)? as usize;
    if count == 0 || count > max_inputs || count > MAX_RECORDED_INPUTS || count > bytes.len() / 48 {
        return Err(invalid("edited checkpoint input count is invalid"));
    }
    let mut edits = EditTrack::default();
    edits
        .rows
        .try_reserve_exact(count)
        .map_err(execution_error)?;
    for _ in 0..count {
        let digest: [u8; 32] = reader
            .get_bytes()
            .map_err(execution_error)?
            .try_into()
            .map_err(execution_error)?;
        let label = reader.get_str().map_err(execution_error)?;
        if label.len() > crate::protocol::MAX_STUDIO_INPUT_LABEL_BYTES {
            return Err(invalid("edited checkpoint input label exceeds its budget"));
        }
        let mut owned = String::new();
        owned
            .try_reserve_exact(label.len())
            .map_err(execution_error)?;
        owned.push_str(label);
        let command = CommandRecord {
            kind: CommandKind::Input,
            identity: Digest::from_bytes(digest),
            label: owned,
        };
        let input = studio_input_payload(scene, &command).map_err(execution_error)?;
        if input.target.is_some()
            || input.revision >= position
            || input.frame.cast_unsigned() >= frame_count
            || edits.rows.last().is_some_and(|row| {
                row.input.frame > input.frame || row.input.revision >= input.revision
            })
        {
            return Err(invalid(
                "edited checkpoint input ordering or target is invalid",
            ));
        }
        edits.rows.push(InputRow { command, input });
    }
    let state = reader.get_bytes().map_err(execution_error)?;
    reader.finish().map_err(execution_error)?;
    Ok(RestoredTrack { edits, state })
}
