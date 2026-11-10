//! A live owner may replace the final capture, never reinterpret history as
//! executable state. Input commands use the existing canonical Studio protocol
//! and share the same opaque journal sequence as committed timeline seeks.

use super::*;
use crate::protocol::{StudioInput, studio_input_payload};
use fmn_scene::CommandRecord;

impl RecordedTimeline {
    /// Opt into a mutable final view. The caller must retain and dispatch the
    /// actual scene callbacks; this does not make a recorded worker executable.
    pub fn enable_live_input(&mut self) -> Result<(), ServiceError> {
        self.check_scene(&self.scene)?;
        if self.input_revision.is_some() {
            return Err(invalid("live capture is already active"));
        }
        self.input_revision = Some(0);
        Ok(())
    }

    /// Freeze input after a callback/render failure. Historical inspection is
    /// still usable, but arbitrary effects cannot be rolled back or retried.
    pub fn disable_live_input(&mut self) {
        self.input_revision = None;
    }

    /// Replace only the final captured frame, atomically and under the original
    /// byte and protocol budgets. Old history is immutable; repeated paused
    /// edits do not consume one frame budget per pointer event.
    pub fn replace_live_frame(
        &mut self,
        stream: FrameStream,
        inspection: InspectorSnapshot,
    ) -> Result<(), ServiceError> {
        if self.input_revision.is_none() {
            return Err(invalid("no live capture owner"));
        }
        let index = self
            .frames
            .len()
            .checked_sub(1)
            .ok_or_else(|| invalid("empty live capture"))?;
        let occupied = self.encoded_bytes - self.frames[index].encoded_bytes;
        let candidate = self.admit_frame(stream, inspection, index as u64, occupied)?;
        self.encoded_bytes = occupied + candidate.encoded_bytes;
        self.frames[index] = candidate;
        Ok(())
    }

    /// Validate before entering authored code. Old revisions, historical frame
    /// edits, foreign scenes, native-object targeting and exhausted journals
    /// never invoke a callback. Picking belongs to the live scene dispatcher.
    pub fn prepare_live_input(
        &self,
        scene: &str,
        command: &CommandRecord,
    ) -> Result<StudioInput, ServiceError> {
        let input = studio_input_payload(scene, command).map_err(failed)?;
        self.check_live_owner(scene, input.frame, input.revision)?;
        if input.target.is_some() {
            return Err(invalid(
                "Python input uses scene hit testing, not native object targets",
            ));
        }
        Ok(input)
    }

    /// Attach the post-callback frame identity to the exact input, with an
    /// opaque barrier and no callback/checkpoint replay claim.
    pub fn commit_live_input(
        &mut self,
        command: CommandRecord,
    ) -> Result<WorkerResponse, ServiceError> {
        let input = self.prepare_live_input(&self.scene, &command)?;
        let next = input
            .revision
            .checked_add(1)
            .ok_or_else(|| invalid("live input revision exhausted"))?;
        let response = self.record_opaque(command)?;
        self.input_revision = Some(next);
        Ok(response)
    }

    pub(super) fn record_opaque(
        &mut self,
        command: CommandRecord,
    ) -> Result<WorkerResponse, ServiceError> {
        if self.commands >= 4096 {
            return Err(invalid("recorded position journal budget exceeded"));
        }
        let mut journal = Journal::new();
        journal
            .record(Entry {
                command,
                effect: EffectClass::Opaque,
                reads: vec![AssetRead {
                    path: "portal/capture-generation".into(),
                    digest: self.source_digest,
                }],
                subprocesses: Vec::new(),
                checkpoint: None,
                state_hash: self.frames[self.position].state_hash,
            })
            .map_err(failed)?;
        let response = WorkerResponse::JournalSegment {
            scene: self.scene.clone(),
            start_entry: self.commands,
            journal: journal.to_bytes().map_err(failed)?,
        };
        self.commands += 1;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{studio_input_command, studio_seek_command};
    use fmn_scene::{EventPayload, Key, Modifiers};

    #[test]
    fn live_tail_preserves_history_and_rejects_stale_commands_before_execution() {
        let mut timeline =
            RecordedTimeline::new("s".into(), sha256(b"b"), sha256(b"s"), 2, 1 << 20).unwrap();
        for v in [0, 120] {
            let (f, i) = super::super::tests::capture("s", v);
            timeline.push(f, i).unwrap();
        }
        timeline.enable_live_input().unwrap();
        let input = StudioInput {
            frame: 1,
            revision: 0,
            target: None,
            event: EventPayload::KeyPress {
                key: Key::ArrowRight,
                modifiers: Modifiers::NONE,
            },
        };
        let command = studio_input_command("s", &input).unwrap();
        assert!(timeline.prepare_live_input("s", &command).is_err());
        timeline
            .handle(SupervisorRequest::Play {
                scene: "s".into(),
                command: studio_seek_command("s", 1).unwrap(),
            })
            .unwrap();
        timeline.prepare_live_input("s", &command).unwrap();
        let used = timeline.encoded_bytes();
        for _ in 0..10 {
            let (f, i) = super::super::tests::capture("s", 255);
            timeline.replace_live_frame(f, i).unwrap();
        }
        assert_eq!(timeline.frame_count(), 2);
        assert!(timeline.encoded_bytes() < used + 64);
        let WorkerResponse::JournalSegment {
            start_entry,
            journal,
            ..
        } = timeline.commit_live_input(command.clone()).unwrap()
        else {
            panic!()
        };
        assert_eq!(start_entry, 1);
        assert!(Journal::from_bytes(&journal).unwrap().entries()[0].is_replay_barrier());
        assert!(timeline.prepare_live_input("s", &command).is_err());
        let WorkerResponse::StudioData { bytes, .. } = timeline
            .handle(SupervisorRequest::Inspect { scene: "s".into() })
            .unwrap()
        else {
            panic!()
        };
        assert!(
            String::from_utf8(bytes)
                .unwrap()
                .contains("\"input_revision\":1")
        );
        assert!(timeline.prepare_live_input("s", &command).is_err());
        timeline
            .handle(SupervisorRequest::Scrub {
                scene: "s".into(),
                frame: 0,
            })
            .unwrap();
        let WorkerResponse::StudioData { bytes, .. } = timeline
            .handle(SupervisorRequest::Inspect { scene: "s".into() })
            .unwrap()
        else {
            panic!()
        };
        assert!(
            String::from_utf8(bytes)
                .unwrap()
                .contains("\"input_events\":false")
        );
        assert_eq!(
            timeline.frames[0].state_hash,
            timeline.last_state_hash().unwrap()
        );
        timeline.disable_live_input();
        let (f, i) = super::super::tests::capture("s", 0);
        assert!(timeline.replace_live_frame(f, i).is_err());
    }

    #[test]
    fn invalid_live_replacement_retains_complete_previous_capture() {
        let mut timeline =
            RecordedTimeline::new("s".into(), sha256(b"b"), sha256(b"s"), 1, 1 << 20).unwrap();
        let (f, i) = super::super::tests::capture("s", 120);
        timeline.push(f, i).unwrap();
        timeline.enable_live_input().unwrap();
        let before = (timeline.last_state_hash(), timeline.encoded_bytes());
        let (mut f, i) = super::super::tests::capture("s", 255);
        f.width += 1;
        assert!(timeline.replace_live_frame(f, i).is_err());
        assert_eq!(
            (timeline.last_state_hash(), timeline.encoded_bytes()),
            before
        );
        let (f, i) = super::super::tests::capture("s", 255);
        timeline.max_bytes = 1;
        assert!(timeline.replace_live_frame(f, i).is_err());
        assert_eq!(
            (timeline.last_state_hash(), timeline.encoded_bytes()),
            before
        );
    }
}
