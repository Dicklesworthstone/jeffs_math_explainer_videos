//! Canonical, optimistic live-clock commands. A captured seek and an advance
//! are deliberately different commands: the latter executes authored updaters.

use crate::protocol::ProtocolError;
use fmn_hash::{Schema, Writer, sha256};
use fmn_scene::{CommandKind, CommandRecord};

const PREFIX: &str = "studio advance ";
const SCHEMA: Schema = Schema::new(*b"FMNI", 5, 1, 0);
pub const MAX_ADVANCE_FRAMES: u32 = 240;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StudioAdvance {
    pub frame: i64,
    pub revision: u64,
    /// Exact nominal steps, never elapsed-wall-time-derived sampling.
    pub frames: u32,
}

/// Authenticate all fields and the selected scene, without changing the
/// durable outer protocol or confusing the command with a timeline seek.
pub fn studio_advance_command(
    scene: &str,
    advance: StudioAdvance,
) -> Result<CommandRecord, ProtocolError> {
    if scene.is_empty()
        || scene.len() > 4096
        || scene.contains('\0')
        || advance.frame < 0
        || !(1..=MAX_ADVANCE_FRAMES).contains(&advance.frames)
    {
        return Err(ProtocolError::Malformed(
            "invalid live advance scene/frame/count",
        ));
    }
    let mut identity = Writer::new(SCHEMA);
    identity
        .put_str(scene)
        .put_i64(advance.frame)
        .put_u64(advance.revision)
        .put_u32(advance.frames);
    Ok(CommandRecord {
        kind: CommandKind::Wait,
        identity: sha256(&identity.finish()?),
        label: format!(
            "{PREFIX}{} {} {}",
            advance.frame, advance.revision, advance.frames
        ),
    })
}

/// Only a dispatch hint; callers must authenticate with `studio_advance_payload`
/// before executing any scene code.
#[must_use]
pub fn is_advance_command(command: &CommandRecord) -> bool {
    command.label.starts_with(PREFIX)
}

pub fn studio_advance_payload(
    scene: &str,
    command: &CommandRecord,
) -> Result<StudioAdvance, ProtocolError> {
    if command.label.len() > 128 {
        return Err(ProtocolError::Malformed("live advance label budget"));
    }
    let text = command
        .label
        .strip_prefix(PREFIX)
        .ok_or(ProtocolError::Malformed("live advance label"))?;
    let mut parts = text.split(' ');
    let invalid = || ProtocolError::Malformed("live advance parameters");
    let advance = StudioAdvance {
        frame: parts
            .next()
            .ok_or_else(invalid)?
            .parse()
            .map_err(|_| invalid())?,
        revision: parts
            .next()
            .ok_or_else(invalid)?
            .parse()
            .map_err(|_| invalid())?,
        frames: parts
            .next()
            .ok_or_else(invalid)?
            .parse()
            .map_err(|_| invalid())?,
    };
    if parts.next().is_some() || studio_advance_command(scene, advance)? != *command {
        return Err(ProtocolError::Malformed("live advance identity"));
    }
    Ok(advance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_advance_is_bounded_canonical_and_bound_to_scene_revision_and_count() {
        let value = StudioAdvance {
            frame: 17,
            revision: 3,
            frames: 2,
        };
        let command = studio_advance_command("Demo", value).unwrap();
        assert_eq!(studio_advance_payload("Demo", &command).unwrap(), value);
        assert!(studio_advance_payload("Other", &command).is_err());
        assert!(crate::protocol::studio_seek_frame("Demo", &command).is_err());
        for label in [
            "studio advance 17 4 2",
            "studio advance 17 3 3",
            "studio advance 017 3 2",
            "studio advance 17 3 2 ",
        ] {
            let mut changed = command.clone();
            changed.label = label.into();
            assert!(studio_advance_payload("Demo", &changed).is_err());
        }
        let mut changed = command.clone();
        changed.kind = CommandKind::Input;
        assert!(studio_advance_payload("Demo", &changed).is_err());
        for frames in [0, 241, u32::MAX] {
            assert!(studio_advance_command("Demo", StudioAdvance { frames, ..value }).is_err());
        }
        assert!(studio_advance_command("Demo", StudioAdvance { frame: -1, ..value }).is_err());
        assert!(studio_advance_command(&"s".repeat(4097), value).is_err());
    }
}
