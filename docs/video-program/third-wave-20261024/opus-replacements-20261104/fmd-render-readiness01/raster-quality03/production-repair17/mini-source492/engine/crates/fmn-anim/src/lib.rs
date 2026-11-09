//! Choreo: the animation engine — Animation trait, RationalFrameClock, timeline algebra (§9).
//!
//! Landed so far: the [`RationalFrameClock`] (§9.2, D-07, BN-02) — exact
//! rational time over the frame grid, with the Reference's emission
//! semantics and no off-grid API — and the Animation contract (§9.1,
//! fm-67a): the constructor surface, the `begin → interpolate(alpha) →
//! finish` lifecycle, the one normalized-alpha pipeline (`time_span` →
//! lag → rate), and the typed [`prepare_animation`] boundary shared with
//! fmn-mobject's `.animate` builder — and the six-step frame order with
//! the [`FramePacket`] freeze (§9.3, D-19, fm-x79): [`play_segment`] /
//! [`wait_segment`] drive the load-bearing order exactly (BN-10 corrects
//! the Reference's skip-mode double-dt), and capture freezes an immutable
//! CoW packet with derivable keyed RNG forks. The §9.5 segment-purity
//! classifier (fm-3xk) rides the drivers: every segment is classified
//! against a closed effect vocabulary (unknown demotes — R20), reported
//! for the replay journal, and pure segments carry their begin-state
//! snapshot so [`reconstruct_pure_frame`] can rebuild any frame
//! bit-identically from (snapshot, alpha, keyed RNG fork) — the five
//! mechanism families and the 80 classes they parameterize (fm-cye) — and
//! the composition and timeline algebra (§9.4, fm-hfe): one interval model
//! under [`AnimationGroup`] / [`Succession`] / [`lagged_start`] /
//! [`lagged_start_map`], with the two Reference defects BN-11 names fixed,
//! and the declarative [`Timeline`] (keyframes, labels, canonical
//! serialization, and seek that lands on the same frame a play-through
//! produced) compiling to those very primitives rather than a second
//! engine.
#![cfg_attr(
    any(
        all(
            target_arch = "x86_64",
            target_feature = "avx2",
            target_feature = "bmi2",
            target_feature = "fma"
        ),
        all(
            target_arch = "x86_64",
            target_feature = "avx512f",
            target_feature = "avx512bw",
            target_feature = "avx512dq",
            target_feature = "avx512vl"
        ),
        all(target_arch = "aarch64", target_feature = "neon")
    ),
    feature(portable_simd)
)]
#![forbid(unsafe_code)]

pub mod animation;
pub mod bundle;
pub mod clock;
pub mod composition;
pub mod creation;
pub mod fading;
pub mod frame;
pub mod growing;
pub mod indication;
pub mod ladder;
pub mod movement;
pub mod purity;
pub mod rotation;
pub mod timeline;
pub mod transform;
pub mod transform_matching;
pub mod update;

pub use animation::{
    AnimConfig, AnimError, AnimState, Animation, AnimationSignature, DEFAULT_ANIMATION_LAG_RATIO,
    DEFAULT_ANIMATION_RUN_TIME, IntoAnimation, IntoAnimations, MethodAnimation, RateFunc,
    prepare_animation, prepare_animations, sub_alpha, time_spanned_alpha,
};
pub use bundle::{
    PATH_STRAIGHT_TAG, bundle_sub_alpha, interpolate_between, path_from_tag, path_tag,
    rate_from_tag, rate_tag,
};
pub use clock::{ClockError, FrameSample, FrameSegment, RationalFrameClock, RationalTime};
pub use composition::{
    AnimationGroup, DEFAULT_LAGGED_START_LAG_RATIO, Interval, Succession, build_timings,
    lagged_start, lagged_start_map,
};
pub use creation::{
    DrawBorderThenFill, IntRound, RevealBounds, ShowIncreasingSubsets, ShowPartial,
    add_text_letter_by_letter, add_text_word_by_word, show_creation, show_increasing_subsets,
    show_passing_flash, show_submobjects_one_by_one, show_surface_creation, uncreate,
    uncreate_surface, write,
};
pub use fading::{
    FadeTransform, VFade, fade_in, fade_in_from_point, fade_out, fade_out_to_point, fade_transform,
    fade_transform_pieces, v_fade_in, v_fade_in_then_out, v_fade_out,
};
pub use frame::{
    AnimationBoundary, FramePacket, OpenSegment, OpenWait, SceneUpdaterBoundary, abort_open_play,
    abort_open_wait, advance_play, advance_play_with_boundary, complete_play_frame,
    complete_wait_frame, finish_open_play, finish_open_play_animations, finish_open_wait,
    open_play, open_play_with_mode, open_wait, open_wait_after_updaters, play_segment,
    play_segment_with_boundary, prepare_play_animation, prepare_play_frame, prepare_wait_frame,
    validate_play, wait_segment, wait_segment_upto, wait_segment_with_boundary,
};
pub use growing::{grow_arrow, grow_from_center, grow_from_edge, grow_from_point};
pub use indication::{
    INDICATION_YELLOW, VShowPassingFlash, WiggleOutThenIn, animation_on_surrounding_rectangle,
    apply_wave, broadcast, circle_indicate, flash, flash_around, flashy_fade_in, focus_on,
    indicate, show_creation_then_destruction, show_creation_then_destruction_around,
    show_creation_then_fade_around, show_creation_then_fade_out, show_passing_flash_around,
    turn_inside_out,
};
pub use ladder::{DeclaredOp, DeclaredUpdater, LadderError, sawtooth, triangle};
pub use movement::{Homotopy, MoveAlongPath, PhaseFlow, complex_homotopy, smoothed_homotopy};
pub use purity::{
    ImpureEffect, Purity, SegmentKind, SegmentReport, classify_play, classify_wait,
    reconstruct_pure_frame,
};
pub use rotation::{Rotating, clock_passes_time, rotate, rotate_default};
pub use timeline::{
    Label, PlannedSegment, Step, TIMELINE_SCHEMA, Timeline, TimelineError, TimelinePlan,
};
pub use transform::{
    PathFunc, STRAIGHT_PATH_THRESHOLD, StartPrep, Transform, apply_complex_function,
    apply_function, apply_matrix, apply_matrix_2d, apply_pointwise_function,
    apply_pointwise_function_to_center, cyclic_replace, fade_to_color, interpolate_fields,
    move_to_target, replacement_transform, restore, scale_in_place, shrink_to_center, swap,
    transform_from_copy,
};
pub use transform_matching::{
    has_same_shape_as, transform_matching_keys, transform_matching_parts,
};
pub use update::{MaintainPositionRelativeTo, UpdateFromFunc};
