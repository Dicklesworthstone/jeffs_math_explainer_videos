//! Live-clock ownership over the same bounded final view and opaque journal.
use super::*;
use crate::advance::{StudioAdvance, studio_advance_payload};
use fmn_scene::CommandRecord;

impl RecordedTimeline {
    /// The live owner must also implement nominal updater stepping. Merely
    /// exposing pointer/key input does not advertise clock execution support.
    pub fn enable_live_advance(&mut self) -> Result<(), ServiceError> {
        if self.input_revision.is_none() {
            return Err(invalid("live advance requires a live scene owner"));
        }
        self.advance_enabled = true;
        Ok(())
    }

    pub fn prepare_live_advance(
        &self,
        scene: &str,
        command: &CommandRecord,
    ) -> Result<StudioAdvance, ServiceError> {
        let advance = studio_advance_payload(scene, command).map_err(failed)?;
        self.check_live_owner(scene, advance.frame, advance.revision)?;
        if !self.advance_enabled {
            return Err(invalid(
                "this worker does not implement live clock stepping",
            ));
        }
        let view = self.frames[self.position]
            .inspection
            .view
            .as_ref()
            .ok_or_else(|| invalid("live view is missing"))?;
        if advance.frames > view.fps {
            return Err(invalid("live advance is limited to one second per command"));
        }
        Ok(advance)
    }

    pub fn commit_live_advance(
        &mut self,
        command: CommandRecord,
    ) -> Result<WorkerResponse, ServiceError> {
        let advance = self.prepare_live_advance(&self.scene, &command)?;
        let next = advance
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("live input revision exhausted"))?;
        let response = self.record_opaque(command)?;
        self.input_revision = Some(next);
        Ok(response)
    }

    pub(super) fn check_live_owner(
        &self,
        scene: &str,
        frame: i64,
        revision: u64,
    ) -> Result<(), ServiceError> {
        self.check_scene(scene)?;
        let current = self
            .input_revision
            .ok_or_else(|| invalid("live input is disabled; reload the scene"))?;
        if frame != self.frames.len() as i64 - 1 || frame != self.position as i64 {
            return Err(invalid(
                "live input requires the selected final frame; historical frames are read-only",
            ));
        }
        if revision != current {
            return Err(invalid("stale live input revision"));
        }
        if revision == u64::MAX {
            return Err(invalid("live input revision exhausted"));
        }
        if self.commands >= 4096 {
            return Err(invalid("recorded position journal budget exceeded"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::advance::studio_advance_command;
    use crate::protocol::{StudioInput, studio_input_command};
    use fmn_scene::{EventPayload, Key, Modifiers};

    #[test]
    fn live_clock_shares_revision_with_events_and_never_executes_history() {
        let mut timeline =
            RecordedTimeline::new("s".into(), sha256(b"b"), sha256(b"s"), 2, 1 << 20).unwrap();
        for value in [0, 120] {
            let (f, i) = super::super::tests::capture("s", value);
            timeline.push(f, i).unwrap();
        }
        let request = StudioAdvance {
            frame: 1,
            revision: 0,
            frames: 2,
        };
        let command = studio_advance_command("s", request).unwrap();
        assert!(timeline.prepare_live_advance("s", &command).is_err());
        timeline.enable_live_input().unwrap();
        timeline.enable_live_advance().unwrap();
        assert!(timeline.prepare_live_advance("s", &command).is_err());
        timeline.select("s", 1).unwrap();
        assert_eq!(
            timeline.prepare_live_advance("s", &command).unwrap(),
            request
        );
        let before = timeline.frames[0].state_hash;
        let (f, i) = super::super::tests::capture("s", 255);
        timeline.replace_live_frame(f, i).unwrap();
        let WorkerResponse::JournalSegment {
            journal,
            start_entry,
            ..
        } = timeline.commit_live_advance(command.clone()).unwrap()
        else {
            panic!()
        };
        assert_eq!(start_entry, 0);
        assert!(Journal::from_bytes(&journal).unwrap().entries()[0].is_replay_barrier());
        assert!(timeline.prepare_live_advance("s", &command).is_err());
        let event = studio_input_command(
            "s",
            &StudioInput {
                frame: 1,
                revision: 1,
                target: None,
                event: EventPayload::KeyPress {
                    key: Key::Enter,
                    modifiers: Modifiers::NONE,
                },
            },
        )
        .unwrap();
        timeline.prepare_live_input("s", &event).unwrap();
        timeline.commit_live_input(event).unwrap();
        assert_eq!(timeline.input_revision, Some(2));
        assert_eq!(timeline.frame_count(), 2);
        assert_eq!(timeline.frames[0].state_hash, before);
        let too_many = studio_advance_command(
            "s",
            StudioAdvance {
                revision: 2,
                frames: 31,
                ..request
            },
        )
        .unwrap();
        assert!(timeline.prepare_live_advance("s", &too_many).is_err());
        timeline.disable_live_input();
        let valid = studio_advance_command(
            "s",
            StudioAdvance {
                revision: 2,
                ..request
            },
        )
        .unwrap();
        assert!(timeline.prepare_live_advance("s", &valid).is_err());
    }
}
