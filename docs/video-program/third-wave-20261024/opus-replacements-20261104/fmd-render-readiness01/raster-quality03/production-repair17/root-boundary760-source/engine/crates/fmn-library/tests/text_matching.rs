//! Native matching uses source provenance and the ordinary Choreo lifecycle.
use std::sync::Arc;

use fmn_anim::{AnimError, Animation, RationalFrameClock, play_segment};
use fmn_core::rng::RngRoot;
use fmn_library::{
    FontBook, SpanKindU8, SpanMapData, SpanMapEntry, Tex, TexEngine, Text, TextMatchingConfig,
    TextMatchingError, TransformMatchingStrings, TransformMatchingTex,
};
use fmn_mobject::{Mob, Mobject, Stage, StageError};

fn fixture(stage: &mut Stage, text: &str, x: f64) -> (Mob, SpanMapData, Vec<Mob>) {
    let root = stage.add(Mobject::new());
    let mut children = Vec::new();
    let mut entries = Vec::new();
    for (index, (start, ch)) in text.char_indices().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let dx = x + index as f64;
        // Every key deliberately has the SAME outline. Shape matching would
        // incorrectly transform unrelated source slices instead of fading them.
        let child = stage.add(Mobject::from_points(&[
            [dx, 0.0, 0.0],
            [dx + 0.5, 1.0, 0.0],
            [dx + 1.0, 0.0, 0.0],
        ]));
        stage.attach(root, child).unwrap();
        children.push(child);
        entries.push(SpanMapEntry {
            start,
            end: start + ch.len_utf8(),
            kind: SpanKindU8::TextGlyph,
        });
    }
    (
        root,
        SpanMapData {
            source: Arc::from(text),
            entries,
        },
        children,
    )
}

fn names(animation: &TransformMatchingTex) -> Vec<&str> {
    animation
        .animations()
        .iter()
        .map(|a| a.state().config.name.as_str())
        .collect()
}

#[test]
fn tex_matches_occurrences_not_outlines_and_does_not_root_target_early() {
    let mut stage = Stage::new();
    let (source, sm, s) = fixture(&mut stage, "xx+", 0.0);
    let (target, tm, t) = fixture(&mut stage, "x-", 10.0);
    stage.add_to_scene(source).unwrap();
    let animation = TransformMatchingTex::new(&mut stage, source, target, &sm, &tm).unwrap();
    assert_eq!(
        names(&animation),
        [
            "Transform",
            "FadeOutToPoint",
            "FadeOutToPoint",
            "FadeInFromPoint"
        ]
    );
    assert_eq!(animation.animations()[0].preflight_mobjects(), [s[0], t[0]]);
    assert_eq!(animation.animations()[1].state().mobject(), s[1]);
    assert_eq!(animation.animations()[3].state().mobject(), t[1]);
    assert_eq!(stage.roots(), &[source]);
    assert_eq!(animation.get_run_time(), 2.0);
    assert!(animation.preflight_mobjects().contains(&target));
}

#[test]
fn strings_move_reordered_runs_as_whole_blocks() {
    let mut stage = Stage::new();
    let (source, sm, s) = fixture(&mut stage, "ABCD", 0.0);
    let (target, tm, t) = fixture(&mut stage, "CDAB", 10.0);
    let animation = TransformMatchingStrings::new(&mut stage, source, target, &sm, &tm).unwrap();
    assert_eq!(animation.animations().len(), 2);
    for (index, (from, to)) in [(&s[..2], &t[2..]), (&s[2..], &t[..2])].iter().enumerate() {
        let pair = animation.animations()[index].preflight_mobjects();
        assert_eq!(stage.get(pair[0]).unwrap().submobjects(), *from);
        assert_eq!(stage.get(pair[1]).unwrap().submobjects(), *to);
    }
}

#[test]
fn consuming_a_block_does_not_invent_adjacency_across_its_old_position() {
    let mut stage = Stage::new();
    let (source, sm, _) = fixture(&mut stage, "AXB", 0.0);
    let (target, tm, _) = fixture(&mut stage, "XAB", 10.0);
    let animation = TransformMatchingStrings::new(&mut stage, source, target, &sm, &tm).unwrap();
    // All longest runs have length one. Deleting claimed slots would wrongly
    // create an AB block after consuming X.
    assert_eq!(animation.animations().len(), 3);
    assert!(animation.animations().iter().all(|a| {
        stage
            .get(a.state().mobject())
            .unwrap()
            .submobjects()
            .is_empty()
    }));
}

#[test]
fn playback_publishes_the_original_target_and_preserves_unrelated_roots() {
    let mut stage = Stage::new();
    let (other, _, _) = fixture(&mut stage, "keep", -10.0);
    let (source, sm, _) = fixture(&mut stage, "AB", 0.0);
    let (target, tm, t) = fixture(&mut stage, "AC", 10.0);
    stage.add_many_to_scene(&[other, source]).unwrap();
    let target_points: Vec<_> = t.iter().map(|&mob| stage.get_points(mob)).collect();
    let animation = TransformMatchingStrings::new(&mut stage, source, target, &sm, &tm).unwrap();
    let mut animations: Vec<Box<dyn Animation>> = vec![Box::new(animation)];
    let mut frames = 0;
    let report = play_segment(
        &mut stage,
        &mut RationalFrameClock::new(10).unwrap(),
        &RngRoot::from_seed(7),
        &mut animations,
        false,
        &mut |_| frames += 1,
    )
    .unwrap();
    assert_eq!(frames, 20);
    assert_eq!(report.n_frames, 20);
    assert!(report.purity.is_pure());
    assert_eq!(stage.roots(), &[other, target]);
    assert_eq!(
        t.iter()
            .map(|&mob| stage.get_points(mob))
            .collect::<Vec<_>>(),
        target_points
    );
    animations[0].clean_up_from_scene(&mut stage);
    assert_eq!(stage.roots(), &[other, target], "cleanup is idempotent");
}

#[test]
fn empty_operands_keep_replacement_and_clock_semantics_in_both_modes() {
    for skip in [false, true] {
        for (from, to) in [("", ""), ("", "A"), ("A", "")] {
            let mut stage = Stage::new();
            let (source, sm, _) = fixture(&mut stage, from, 0.0);
            let (target, tm, _) = fixture(&mut stage, to, 10.0);
            stage.add_to_scene(source).unwrap();
            let animation =
                TransformMatchingTex::new(&mut stage, source, target, &sm, &tm).unwrap();
            let mut animations: Vec<Box<dyn Animation>> = vec![Box::new(animation)];
            let mut clock = RationalFrameClock::new(10).unwrap();
            play_segment(
                &mut stage,
                &mut clock,
                &RngRoot::from_seed(7),
                &mut animations,
                skip,
                &mut |_| {},
            )
            .unwrap();
            assert_eq!(clock.now().frames(), 20);
            assert_eq!(stage.roots(), &[target]);
        }
    }
}

#[test]
fn topology_changes_after_planning_fail_before_begin_mutates_the_arena() {
    let mut stage = Stage::new();
    let (source, sm, s) = fixture(&mut stage, "AB", 0.0);
    let (target, tm, _) = fixture(&mut stage, "AB", 10.0);
    let mut animation = TransformMatchingTex::new(&mut stage, source, target, &sm, &tm).unwrap();
    stage.replace_children(source, &[s[1], s[0]]).unwrap();
    let epoch = stage.topology_epoch();
    assert_eq!(
        animation.begin(&mut stage),
        Err(AnimError::Stage(StageError::FamilyShapeMismatch))
    );
    assert_eq!(stage.topology_epoch(), epoch);
}

#[test]
fn malformed_utf8_spans_and_missing_maps_do_not_allocate_groups() {
    let mut stage = Stage::new();
    let (source, mut sm, _) = fixture(&mut stage, "é", 0.0);
    let (target, tm, _) = fixture(&mut stage, "é", 10.0);
    for invalid in [
        SpanMapEntry {
            start: 1,
            end: 2,
            kind: SpanKindU8::TextGlyph,
        },
        SpanMapEntry {
            start: 0,
            end: 99,
            kind: SpanKindU8::TextGlyph,
        },
        SpanMapEntry {
            start: 2,
            end: 1,
            kind: SpanKindU8::TextGlyph,
        },
    ] {
        sm.entries[0] = invalid;
        let epoch = stage.topology_epoch();
        assert!(matches!(
            TransformMatchingTex::new(&mut stage, source, target, &sm, &tm),
            Err(TextMatchingError::InvalidSpan {
                side: "source",
                index: 0
            })
        ));
        assert_eq!(stage.topology_epoch(), epoch);
    }
    sm.entries.clear();
    assert!(matches!(
        TransformMatchingTex::new(&mut stage, source, target, &sm, &tm),
        Err(TextMatchingError::InvalidLayout { .. })
    ));
}

#[test]
fn shared_members_are_refused_before_any_planning_side_effect() {
    let mut stage = Stage::new();
    let (source, sm, s) = fixture(&mut stage, "A", 0.0);
    let target = stage.add(Mobject::new());
    stage.attach(target, s[0]).unwrap();
    let epoch = stage.topology_epoch();
    assert!(matches!(
        TransformMatchingTex::new(&mut stage, source, target, &sm, &sm),
        Err(TextMatchingError::AliasedFamilies)
    ));
    assert_eq!(stage.topology_epoch(), epoch);
}

#[test]
fn decorations_not_in_the_glyph_map_are_faded_not_dropped() {
    let mut stage = Stage::new();
    let (source, sm, _) = fixture(&mut stage, "A", 0.0);
    let (target, tm, _) = fixture(&mut stage, "A", 10.0);
    let decoration = stage.add(Mobject::from_points(&[[0.0, -0.2, 0.0], [1.0, -0.2, 0.0]]));
    stage.attach(source, decoration).unwrap();
    let animation = TransformMatchingTex::new(&mut stage, source, target, &sm, &tm).unwrap();
    assert_eq!(names(&animation), ["Transform", "FadeOutToPoint"]);
    assert_eq!(animation.animations()[1].state().mobject(), decoration);
}

#[test]
fn actual_native_tex_and_unicode_text_maps_drive_matching_without_a_portal() {
    let mut stage = Stage::new();
    let engine = TexEngine::new("fmd-math/pack/default", None).unwrap();
    let a = Tex::new(r"x^2+1").build(&engine).unwrap();
    let b = Tex::new(r"x^3-1").build(&engine).unwrap();
    let (am, bm) = (a.span_map(), b.span_map());
    let (source, target) = (stage.add(a), stage.add(b));
    let animation = TransformMatchingTex::new(&mut stage, source, target, &am, &bm).unwrap();
    assert!(names(&animation).contains(&"Transform"));
    assert!(names(&animation).contains(&"FadeOutToPoint"));
    let book = FontBook::bundled().unwrap();
    let a = Text::new("éA").font("IBM Plex Sans").build(&book).unwrap();
    let b = Text::new("Aé").font("IBM Plex Sans").build(&book).unwrap();
    let (am, bm) = (a.span_map(), b.span_map());
    let (source, target) = (stage.add(a), stage.add(b));
    let mut animation =
        TransformMatchingStrings::new(&mut stage, source, target, &am, &bm).unwrap();
    assert_eq!(animation.animations().len(), 2);
    animation.begin(&mut stage).unwrap();
    animation.interpolate(&mut stage, 0.5);
    animation.finish(&mut stage);
    animation.clean_up_from_scene(&mut stage);
    assert!(stage.roots().contains(&target));
}

#[test]
fn explicit_timing_and_abort_use_shared_animation_lifecycle() {
    let mut stage = Stage::new();
    let (source, sm, _) = fixture(&mut stage, "AB", 0.0);
    let (target, tm, _) = fixture(&mut stage, "BC", 10.0);
    stage.add_to_scene(source).unwrap();
    let mut animation = TransformMatchingTex::with_config(
        &mut stage,
        source,
        target,
        &sm,
        &tm,
        TextMatchingConfig {
            run_time: 0.5,
            lag_ratio: 0.2,
            ..TextMatchingConfig::default()
        },
    )
    .unwrap();
    assert_eq!(animation.get_run_time(), 0.5);
    animation.begin(&mut stage).unwrap();
    animation.interpolate(&mut stage, 0.25);
    animation.abort(&mut stage);
    assert!(!stage.roots().contains(&target));
    animation.clean_up_from_scene(&mut stage);
    assert!(matches!(
        animation.deferred_error(),
        Some(AnimError::InvalidFramePhase(_))
    ));
}
