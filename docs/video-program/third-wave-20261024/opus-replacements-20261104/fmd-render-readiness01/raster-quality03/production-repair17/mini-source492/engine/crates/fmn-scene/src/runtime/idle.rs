//! Host-paced live frames on the existing rational clock. Unlike a wait,
//! an idle tick is not a new authored segment and does not consume a play index.

use std::rc::Rc;

use super::*;

/// One prepared idle frame, consumed exactly once by its originating Scene.
/// The host releases its Scene borrow between preparation and completion to
/// run host-language updaters. A superseded or foreign token cannot complete.
#[derive(Debug)]
pub struct IdleFrame {
    owner: Rc<()>,
    sequence: u64,
    time: RationalTime,
    play_count: u64,
}

impl IdleFrame {
    #[must_use]
    pub fn dt(&self) -> f64 {
        1.0 / f64::from(self.time.fps())
    }
    #[must_use]
    pub const fn time(&self) -> RationalTime {
        self.time
    }
}

impl Scene {
    /// Advance exactly one nominal frame, then yield before scene updaters.
    /// No elapsed-wall-time rounding, skipped sampling, or play-count change.
    /// A host callback failure is not rollback: the live owner must freeze the
    /// generation rather than automatically retrying arbitrary authored work.
    pub fn prepare_idle_frame(
        &mut self,
        sink: &mut dyn SceneSink,
    ) -> Result<IdleFrame, SceneError> {
        self.ensure_preflight(&[], sink)?;
        let sequence = self
            .idle_sequence
            .checked_add(1)
            .ok_or(SceneError::InvalidState("idle frame sequence exhausted"))?;
        self.clock.advance_frames(1).map_err(AnimError::Clock)?;
        self.sync_stage_time();
        self.idle_sequence = sequence;
        Ok(IdleFrame {
            owner: Rc::clone(&self.idle_owner),
            sequence,
            time: self.clock.now(),
            play_count: self.play_count,
        })
    }

    /// Complete native updaters and queued input, then capture the exact frame
    /// prepared above. The order is the same as a presenter hold, with a
    /// borrow-free host-updater window inserted before native updaters.
    pub fn complete_idle_frame(
        &mut self,
        frame: IdleFrame,
        sink: &mut dyn SceneSink,
    ) -> Result<(), SceneError> {
        self.ensure_ready()?;
        if !Rc::ptr_eq(&frame.owner, &self.idle_owner)
            || frame.sequence != self.idle_sequence
            || frame.time != self.clock.now()
            || frame.play_count != self.play_count
        {
            return Err(SceneError::InvalidState(
                "idle frame was superseded or belongs to another Scene",
            ));
        }
        self.stage.update_at_time(frame.dt(), frame.time.to_f64());
        self.dispatch_pending_events()?;
        sink.capture(
            CaptureReason::PresenterHold,
            FramePacket::freeze_barrier(&self.stage, &self.clock, &self.rng_root),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    struct Sink {
        frames: Vec<FramePacket>,
    }
    impl SceneSink for Sink {
        fn capture(
            &mut self,
            reason: CaptureReason,
            packet: FramePacket,
        ) -> Result<(), IntegrationError> {
            assert_eq!(reason, CaptureReason::PresenterHold);
            self.frames.push(packet);
            Ok(())
        }
    }

    #[test]
    fn idle_frames_use_nonzero_nominal_dt_and_do_not_consume_play_indices() {
        let mut scene = Scene::new(
            RuntimeConfig {
                fps: 7,
                end_at_play: Some(1),
                ..RuntimeConfig::default()
            },
            7,
        )
        .unwrap();
        let source = scene
            .add_mobject(Mobject::from_points(&[[0.0; 3]]))
            .unwrap();
        let follower = scene
            .add_mobject(Mobject::from_points(&[[0.0; 3]]))
            .unwrap();
        let seen = Rc::new(RefCell::new(Vec::new()));
        let observations = Rc::clone(&seen);
        scene
            .stage_mut()
            .add_updater(
                follower,
                move |stage, me| {
                    observations.borrow_mut().push(stage.time());
                    stage.set_x(me, stage.get_center(source)[0]);
                },
                false,
            )
            .unwrap();
        let mut sink = Sink::default();
        for index in 1..=21 {
            let frame = scene.prepare_idle_frame(&mut sink).unwrap();
            assert_eq!(frame.time().frames(), index);
            assert_eq!(frame.dt(), 1.0 / 7.0);
            assert_eq!(scene.stage().time(), index as f64 / 7.0);
            // Stand-in for host-language work with the Scene borrow released.
            scene.stage_mut().set_x(source, index as f64);
            scene.complete_idle_frame(frame, &mut sink).unwrap();
            assert_eq!(scene.play_count(), 0);
            assert_eq!(scene.stage().get_center(follower)[0], index as f64);
        }
        assert_eq!(scene.time().frames(), 21);
        assert_eq!(scene.time().to_f64(), 3.0);
        assert_eq!(
            seen.borrow().len(),
            21,
            "exactly one native updater pass per idle frame"
        );
        assert_eq!(sink.frames.len(), 21);
        assert_eq!(
            sink.frames[0].materialize_stage().get_center(follower)[0],
            1.0
        );
    }

    #[test]
    fn idle_tokens_reject_foreign_superseded_and_segment_advanced_scenes() {
        let mut scene = Scene::default();
        let mut other = Scene::default();
        let mut sink = Sink::default();
        let first = scene.prepare_idle_frame(&mut sink).unwrap();
        let _other = other.prepare_idle_frame(&mut sink).unwrap();
        assert!(other.complete_idle_frame(first, &mut sink).is_err());
        let older = scene.prepare_idle_frame(&mut sink).unwrap();
        let current = scene.prepare_idle_frame(&mut sink).unwrap();
        assert!(scene.complete_idle_frame(older, &mut sink).is_err());
        scene.complete_idle_frame(current, &mut sink).unwrap();
        let frame = scene.prepare_idle_frame(&mut sink).unwrap();
        scene.wait(Some(0.0), &mut NullSceneSink).unwrap();
        assert!(scene.complete_idle_frame(frame, &mut sink).is_err());
        assert_eq!(sink.frames.len(), 1);
    }

    #[test]
    fn idle_tokens_cannot_survive_discontinuous_seek_or_state_restore() {
        let mut scene = Scene::default();
        let mut sink = Sink::default();
        let before_seek = scene.prepare_idle_frame(&mut sink).unwrap();
        let time = before_seek.time().frames();
        scene.seek_interactive_frame(0).unwrap();
        scene.seek_interactive_frame(time).unwrap();
        assert!(scene.complete_idle_frame(before_seek, &mut sink).is_err());

        let before_restore = scene.prepare_idle_frame(&mut sink).unwrap();
        let state = scene.state().unwrap();
        scene.restore_state(&state).unwrap();
        assert!(
            scene
                .complete_idle_frame(before_restore, &mut sink)
                .is_err()
        );

        let before_durable_restore = scene.prepare_idle_frame(&mut sink).unwrap();
        let bytes = scene.state_bytes().unwrap();
        scene.restore_state_bytes(&bytes).unwrap();
        assert!(
            scene
                .complete_idle_frame(before_durable_restore, &mut sink)
                .is_err()
        );
        assert!(sink.frames.is_empty(), "superseded frames must not publish");

        let resumed = scene.prepare_idle_frame(&mut sink).unwrap();
        scene.complete_idle_frame(resumed, &mut sink).unwrap();
        assert_eq!(sink.frames.len(), 1);
    }

    #[test]
    fn rejected_seek_and_restore_leave_the_pending_idle_frame_valid() {
        let mut scene = Scene::default();
        let mut other = Scene::default();
        let mut sink = Sink::default();
        let frame = scene.prepare_idle_frame(&mut sink).unwrap();
        assert!(scene.seek_interactive_frame(-1).is_err());
        assert!(scene.restore_state(&other.state().unwrap()).is_err());
        assert!(scene.restore_state_bytes(b"not a scene state").is_err());
        scene.complete_idle_frame(frame, &mut sink).unwrap();
        assert_eq!(sink.frames.len(), 1);
    }
}
