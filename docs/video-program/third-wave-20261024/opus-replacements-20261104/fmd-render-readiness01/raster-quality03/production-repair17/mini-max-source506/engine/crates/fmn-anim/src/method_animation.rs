//! Complete native method animations over the shared Transform mechanism.
use super::{AnimConfig, AnimError, AnimState, Animation, AnimationSignature};
use crate::transform::Transform;
use fmn_mobject::{BuiltAnimate, Mob, Stage};

/// A built `.animate` recording using the complete Transform mechanism.
///
/// The recording still resolves its target at build time. At begin, Transform
/// aligns a private target when necessary and freezes the starting family.
/// Sharing these hooks also gives method animations point-path interpolation,
/// numeric uniforms/trackers, matching-field locks, and correct host-view
/// materialization without a second, reduced implementation of Transform.
#[derive(Debug, Clone)]
pub struct MethodAnimation {
    transform: Transform,
    target: Mob,
}

impl MethodAnimation {
    /// Wrap a built recording, retaining its path and lifecycle arguments.
    /// Handle and schema validation occurs at prepare/begin, as for Transform.
    pub fn new(built: BuiltAnimate) -> Result<Self, AnimError> {
        let mut config = AnimConfig::from_animate_args(&built.args);
        if config.name.is_empty() {
            config.name = "MethodAnimation".to_owned();
        }
        let transform = Transform::new(built.source, built.target)
            .with_config(config)
            .with_path_arc(
                built.args.path_arc.unwrap_or(0.0),
                built.args.path_arc_axis.unwrap_or(fmn_core::constants::OUT),
            );
        Ok(Self {
            transform,
            target: built.target,
        })
    }

    /// The original build-time target; alignment never mutates this family.
    #[must_use]
    pub fn target(&self) -> Mob {
        self.target
    }
}

impl Animation for MethodAnimation {
    fn state(&self) -> &AnimState {
        self.transform.state()
    }

    fn state_mut(&mut self) -> &mut AnimState {
        self.transform.state_mut()
    }

    fn effect_signature(&self) -> AnimationSignature {
        self.transform.effect_signature()
    }

    fn setup(&mut self, stage: &mut Stage) -> Result<(), AnimError> {
        self.transform.setup(stage)
    }

    fn create_starting_mobject(&self, stage: &mut Stage) -> Result<Mob, AnimError> {
        self.transform.create_starting_mobject(stage)
    }

    fn after_begin(&mut self, stage: &mut Stage) {
        self.transform.after_begin(stage);
    }

    fn teardown(&mut self, stage: &mut Stage) {
        self.transform.teardown(stage);
    }

    fn clean_up_from_scene(&mut self, stage: &mut Stage) {
        self.transform.clean_up_from_scene(stage);
    }

    fn all_mobjects(&self) -> Vec<Mob> {
        self.transform.all_mobjects()
    }

    fn preflight_mobjects(&self) -> Vec<Mob> {
        self.transform.preflight_mobjects()
    }

    fn interpolate_submobject(&mut self, stage: &mut Stage, mobs: &[Mob], sub_alpha: f64) {
        self.transform
            .interpolate_submobject(stage, mobs, sub_alpha);
    }
}
