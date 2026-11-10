//! Partial-reveal mechanism semantics (fm-cye, §9.4 family 2):
//! ShowCreation/Uncreate bounds and lifecycle, the passing-flash window
//! and its teardown restore, DrawBorderThenFill's two halves against
//! creation.py:122, Write's derived parameters, and the subset animations'
//! raw-alpha rounding rules.

use fmn_anim::animation::Animation;
use fmn_anim::{
    DrawBorderThenFill, IntRound, RateFunc, RevealBounds, show_creation, show_increasing_subsets,
    show_passing_flash, show_submobjects_one_by_one, show_surface_creation, uncreate,
    uncreate_surface, write,
};
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::{Mob, Mobject, Stage};

fn vmob(stage: &mut Stage, points: &[[f64; 3]]) -> Mob {
    let mob = stage.add(Mobject::new());
    let entry = stage.get_mut(mob).unwrap();
    entry.buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
    #[allow(clippy::cast_possible_truncation)]
    let flat: Vec<f32> = points
        .iter()
        .flat_map(|p| p.iter().map(|v| *v as f32))
        .collect();
    entry.buffer.write_range("point", 0, &flat);
    mob
}

/// A 2-curve straight-line vmobject with x = [0, 1, 2, 3, 4].
fn line5(stage: &mut Stage) -> Mob {
    vmob(
        stage,
        &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
        ],
    )
}

fn xs_of(stage: &Stage, mob: Mob) -> Vec<f32> {
    #[allow(clippy::cast_possible_truncation)]
    stage
        .get_points(mob)
        .unwrap()
        .into_iter()
        .map(|point| point[0] as f32)
        .collect()
}

fn surface3(stage: &mut Stage) -> Mob {
    let schema = RecordSchema::new(
        &[("point", 3), ("d_normal_point", 3), ("rgba", 4)],
        &["point"],
        &["point", "d_normal_point"],
    )
    .expect("surface schema");
    let mut buffer = RecordBuffer::new(schema, 9).expect("3x3 surface");
    let mut points = Vec::new();
    let mut normals = Vec::new();
    for u in [0.0_f32, 1.0, 2.0] {
        for v in [0.0_f32, 1.0, 2.0] {
            points.extend_from_slice(&[u, v, 0.0]);
            normals.extend_from_slice(&[u, v, 1.0]);
        }
    }
    buffer.write_range("point", 0, &points);
    buffer.write_range("d_normal_point", 0, &normals);
    buffer.write_range("rgba", 0, &[1.0; 36]);
    stage.add(Mobject::from_buffer(buffer))
}

// ------------------------------------------------------------ ShowPartial

#[test]
fn reveal_bounds_formulas() {
    assert_eq!(RevealBounds::Creation.eval(0.3), (0.0, 0.3));
    // PassingFlash tw=0.5 at α=0.5: upper = 0.75, lower = 0.25.
    let (lo, hi) = RevealBounds::PassingFlash { time_width: 0.5 }.eval(0.5);
    assert!((lo - 0.25).abs() < 1e-12 && (hi - 0.75).abs() < 1e-12);
    // Clamps at both ends.
    assert_eq!(
        RevealBounds::PassingFlash { time_width: 0.5 }.eval(0.0),
        (0.0, 0.0)
    );
    let (lo, hi) = RevealBounds::PassingFlash { time_width: 0.5 }.eval(1.0);
    assert!((lo - 1.0).abs() < 1e-12 && (hi - 1.0).abs() < 1e-12);
}

#[test]
fn show_creation_reveals_from_collapsed_to_full() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let original = xs_of(&stage, mob);
    let mut anim = show_creation(mob);
    anim.state_mut().config.rate_func = RateFunc::linear();
    assert_eq!(anim.state().config.lag_ratio, 1.0);
    anim.begin(&mut stage).unwrap();
    // begin's interpolate(0): everything collapses onto the start point.
    assert_eq!(xs_of(&stage, mob), vec![0.0; 5]);
    // Halfway: the first curve, then collapse.
    anim.interpolate(&mut stage, 0.5);
    assert_eq!(xs_of(&stage, mob), vec![0.0, 1.0, 2.0, 2.0, 2.0]);
    anim.finish(&mut stage);
    assert_eq!(xs_of(&stage, mob), original);
    assert!(!anim.is_remover());
}

#[test]
fn uncreate_runs_reversed_and_removes() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let original = xs_of(&stage, mob);
    let mut anim = uncreate(mob);
    assert!(anim.should_match_start());
    anim.begin(&mut stage).unwrap();
    // Reversed rate: alpha 0 shows the full mobject.
    assert_eq!(xs_of(&stage, mob), original);
    anim.finish(&mut stage);
    // final alpha 1 → smooth(0) = 0 → collapsed; the scene runtime
    // consumes the remover flag.
    assert_eq!(xs_of(&stage, mob), vec![0.0; 5]);
    assert!(anim.is_remover());
}

#[test]
fn passing_flash_windows_then_restores_on_teardown() {
    let mut stage = Stage::new();
    let mob = line5(&mut stage);
    let original = xs_of(&stage, mob);
    let mut anim = show_passing_flash(mob, 0.5);
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim.begin(&mut stage).unwrap();
    // α = 0.5 → window [0.25, 0.75] → the two-curve interior restriction.
    anim.interpolate(&mut stage, 0.5);
    assert_eq!(xs_of(&stage, mob), vec![1.0, 1.5, 2.0, 2.5, 3.0]);
    // finish lands on the collapsed end window, then teardown restores
    // the full run (indication.py:188).
    anim.finish(&mut stage);
    assert_eq!(xs_of(&stage, mob), original);
    assert!(anim.is_remover());
}

#[test]
fn show_creation_lag_reveals_children_successively() {
    let mut stage = Stage::new();
    let root = stage.add(Mobject::new());
    let c1 = line5(&mut stage);
    let c2 = line5(&mut stage);
    stage.attach(root, c1).unwrap();
    stage.attach(root, c2).unwrap();
    let mut anim = show_creation(root);
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim.begin(&mut stage).unwrap();
    // Rows: (root, c1, c2), lag_ratio 1 → full_length 3. At α = 0.5,
    // value = 1.5: c1 at sub-alpha 0.5, c2 still at 0.
    anim.interpolate(&mut stage, 0.5);
    assert_eq!(xs_of(&stage, c1), vec![0.0, 1.0, 2.0, 2.0, 2.0]);
    assert_eq!(xs_of(&stage, c2), vec![0.0; 5]);
}

#[test]
fn surface_show_creation_uses_uv_grid_partial_reveal() {
    let mut stage = Stage::new();
    let surface = surface3(&mut stage);
    let original = stage.get_points(surface).expect("points");
    let mut anim = show_surface_creation(surface, (3, 3), 1);
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim.begin(&mut stage).expect("surface animation begins");
    assert_eq!(
        stage.get_points(surface).expect("collapsed points"),
        vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
        ]
    );
    anim.interpolate(&mut stage, 0.5);
    assert_eq!(
        stage.get_points(surface).expect("half surface"),
        vec![
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
        ]
    );
    anim.finish(&mut stage);
    assert_eq!(
        stage.get_points(surface).expect("restored surface"),
        original
    );

    let mut reverse = uncreate_surface(surface, (3, 3), 1);
    reverse.begin(&mut stage).expect("surface uncreate begins");
    assert_eq!(
        stage.get_points(surface).expect("full reverse start"),
        original
    );
    reverse.finish(&mut stage);
    assert!(reverse.is_remover());
    assert_eq!(
        stage.get_points(surface).expect("collapsed reverse end"),
        vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
        ]
    );
}

// ---------------------------------------------------- DrawBorderThenFill

/// A filled, stroked one-curve vmobject for border-then-fill tests.
fn styled_vmob(stage: &mut Stage) -> Mob {
    let mob = vmob(stage, &[[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [2.0, 0.0, 0.0]]);
    let entry = stage.get_mut(mob).unwrap();
    entry
        .buffer
        .write_range("fill_rgba", 0, &[0.2, 0.4, 0.6, 1.0].repeat(3));
    entry
        .buffer
        .write_range("stroke_rgba", 0, &[1.0, 0.0, 0.0, 0.8].repeat(3));
    entry.buffer.write_range("stroke_width", 0, &[4.0; 3]);
    mob
}

fn column(stage: &Stage, mob: Mob, field: &str) -> Vec<f32> {
    stage.get(mob).unwrap().buffer.read_column(field).unwrap()
}

#[test]
fn draw_border_then_fill_two_halves() {
    let mut stage = Stage::new();
    let mob = styled_vmob(&mut stage);
    let mut anim = DrawBorderThenFill::new(mob);
    anim.state_mut().config.rate_func = RateFunc::linear();
    assert_eq!(anim.state().config.run_time, 2.0);
    anim.begin(&mut stage).unwrap();

    // The outline: fill opacity 0, stroke width 2, own stroke color.
    let outline = anim.outline().unwrap();
    assert!(
        column(&stage, outline, "fill_rgba")[3..]
            .iter()
            .step_by(4)
            .all(|&a| a == 0.0)
    );
    assert_eq!(column(&stage, outline, "stroke_width"), vec![2.0; 3]);
    assert_eq!(
        &column(&stage, outline, "stroke_rgba")[..3],
        &[1.0, 0.0, 0.0]
    );
    // match_style after begin: the live mobject wears the outline style.
    assert_eq!(column(&stage, mob, "fill_rgba")[3], 0.0);
    assert_eq!(column(&stage, mob, "stroke_width"), vec![2.0; 3]);

    // First half: partial reveal of the outline at sub-alpha 0.5.
    anim.interpolate(&mut stage, 0.25);
    assert_eq!(xs_of(&stage, mob), vec![0.0, 0.5, 1.0]);
    assert_eq!(column(&stage, mob, "fill_rgba")[3], 0.0);

    // Second half: cross-fade outline → start at sub-alpha 0.5.
    anim.interpolate(&mut stage, 0.75);
    assert!((column(&stage, mob, "fill_rgba")[3] - 0.5).abs() < 1e-6);
    assert!((column(&stage, mob, "stroke_width")[0] - 3.0).abs() < 1e-6);

    // Finish lands exactly on the start data.
    anim.finish(&mut stage);
    assert_eq!(column(&stage, mob, "fill_rgba")[3], 1.0);
    assert_eq!(column(&stage, mob, "stroke_width"), vec![4.0; 3]);
    assert_eq!(xs_of(&stage, mob), vec![0.0, 1.0, 2.0]);
}

#[test]
fn write_derives_reference_parameters() {
    let mut stage = Stage::new();
    let mob = styled_vmob(&mut stage);
    let anim = write(&stage, mob);
    // Family size 1: run_time 1, lag_ratio min(4/2, 0.2) = 0.2.
    assert_eq!(anim.state().config.run_time, 1.0);
    assert!((anim.state().config.lag_ratio - 0.2).abs() < 1e-12);
    assert_eq!(anim.state().config.name, "Write");
}

// ------------------------------------------------- ShowIncreasingSubsets

#[test]
fn increasing_subsets_round_ties_even() {
    let mut stage = Stage::new();
    let root = stage.add(Mobject::new());
    let children: Vec<Mob> = (0..3).map(|_| line5(&mut stage)).collect();
    for &c in &children {
        stage.attach(root, c).unwrap();
    }
    let mut anim = show_increasing_subsets(&stage, root).unwrap();
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim.begin(&mut stage).unwrap();
    // begin's interpolate(0): no children shown.
    assert!(stage.get(root).unwrap().submobjects().is_empty());
    // α = 0.5 → 1.5 → banker's rounding → 2.
    anim.interpolate(&mut stage, 0.5);
    assert_eq!(stage.get(root).unwrap().submobjects(), &children[..2]);
    anim.finish(&mut stage);
    assert_eq!(stage.get(root).unwrap().submobjects(), &children[..]);
}

#[test]
fn one_by_one_uses_ceiling_and_reference_indexing() {
    let mut stage = Stage::new();
    let root = stage.add(Mobject::new());
    let children: Vec<Mob> = (0..3).map(|_| line5(&mut stage)).collect();
    for &c in &children {
        stage.attach(root, c).unwrap();
    }
    let anim = show_submobjects_one_by_one(&stage, root).unwrap();
    assert_eq!(
        anim.with_int_round(IntRound::Ceil).state().config.name,
        "ShowSubmobjectsOneByOne"
    );
    let mut anim = show_submobjects_one_by_one(&stage, root).unwrap();
    anim.state_mut().config.rate_func = RateFunc::linear();
    anim.begin(&mut stage).unwrap();
    assert!(stage.get(root).unwrap().submobjects().is_empty());
    // ceil(0.2·3) = 1 → the first child alone.
    anim.interpolate(&mut stage, 0.2);
    assert_eq!(stage.get(root).unwrap().submobjects(), &children[..1]);
    // ceil(0.5·3) = 2 → the second child alone.
    anim.interpolate(&mut stage, 0.5);
    assert_eq!(stage.get(root).unwrap().submobjects(), &children[1..2]);
    // The Reference clips to n−1 before its index−1 lookup, so the final
    // frame shows the second-to-last child — ported verbatim.
    anim.interpolate(&mut stage, 1.0);
    assert_eq!(stage.get(root).unwrap().submobjects(), &children[1..2]);
}

// ---------------------------------------------------- AddTextWordByWord

#[test]
fn add_text_word_by_word_reveals_word_groups_in_order() {
    // fm-5wq.4.54: three "word groups" (each a group of glyph children the
    // caller partitioned by span maps) appear in order under linear rate.
    let mut stage = Stage::new();
    let grouped = stage.add(Mobject::new());
    let words: Vec<Mob> = (0..3)
        .map(|_| {
            let word = stage.add(Mobject::new());
            let glyph = line5(&mut stage);
            stage.attach(word, glyph).unwrap();
            word
        })
        .collect();
    for &word in &words {
        stage.attach(grouped, word).unwrap();
    }
    let mut anim = fmn_anim::add_text_word_by_word(&stage, grouped, 0.2, -1.0).unwrap();
    // The Reference's parameterization: linear rate, 0.2 s per word.
    assert_eq!(anim.state().config.name, "AddTextWordByWord");
    assert!((anim.state().config.run_time - 0.6).abs() < 1e-12);
    assert_eq!(anim.state().config.rate_func.eval(0.25), 0.25);
    anim.begin(&mut stage).unwrap();
    assert!(stage.get(grouped).unwrap().submobjects().is_empty());
    // Banker's rounding of raw alpha, exactly ShowIncreasingSubsets:
    // round(0.4·3) = 1 word, round(0.9·3) = 3 words.
    anim.interpolate(&mut stage, 0.4);
    assert_eq!(stage.get(grouped).unwrap().submobjects(), &words[..1]);
    anim.interpolate(&mut stage, 0.9);
    assert_eq!(stage.get(grouped).unwrap().submobjects(), &words[..]);
    anim.finish(&mut stage);
    assert_eq!(stage.get(grouped).unwrap().submobjects(), &words[..]);
}

#[test]
fn add_text_word_by_word_keeps_explicit_run_time() {
    let mut stage = Stage::new();
    let grouped = stage.add(Mobject::new());
    let word = line5(&mut stage);
    stage.attach(grouped, word).unwrap();
    let anim = fmn_anim::add_text_word_by_word(&stage, grouped, 0.2, 1.5).unwrap();
    // A non-negative run_time is the caller's, not the sentinel recompute.
    assert!((anim.state().config.run_time - 1.5).abs() < 1e-12);
}

#[test]
fn add_text_word_by_word_refuses_empty_and_stale() {
    let mut stage = Stage::new();
    let empty = stage.add(Mobject::new());
    // No word groups: named refusal, never a degenerate zero-word play.
    match fmn_anim::add_text_word_by_word(&stage, empty, 0.2, -1.0) {
        Err(fmn_anim::animation::AnimError::NoWordGroups) => {}
        other => panic!("expected NoWordGroups, got {other:?}"),
    }
    let stale = {
        let mut scratch = Stage::new();
        scratch.add(Mobject::new())
    };
    assert!(matches!(
        fmn_anim::add_text_word_by_word(&stage, stale, 0.2, -1.0),
        Err(fmn_anim::animation::AnimError::StaleHandle(_))
    ));
}

// -------------------------------------------------- AddTextLetterByLetter

#[test]
fn add_text_letter_by_letter_reveals_letters_in_order() {
    // fm-5wq.4.59: four letter groups (one glyph each) appear in order
    // under linear rate; run_time derives from time_per_char.
    let mut stage = Stage::new();
    let grouped = stage.add(Mobject::new());
    let letters: Vec<Mob> = (0..4)
        .map(|_| {
            let letter = stage.add(Mobject::new());
            let glyph = line5(&mut stage);
            stage.attach(letter, glyph).unwrap();
            letter
        })
        .collect();
    for &letter in &letters {
        stage.attach(grouped, letter).unwrap();
    }
    let mut anim = fmn_anim::add_text_letter_by_letter(&stage, grouped, 0.1, -1.0).unwrap();
    assert_eq!(anim.state().config.name, "AddTextLetterByLetter");
    assert!((anim.state().config.run_time - 0.4).abs() < 1e-12);
    assert_eq!(anim.state().config.rate_func.eval(0.75), 0.75);
    anim.begin(&mut stage).unwrap();
    assert!(stage.get(grouped).unwrap().submobjects().is_empty());
    // Banker's rounding of raw alpha: round(0.3·4) = 1, round(0.6·4) = 2.
    anim.interpolate(&mut stage, 0.3);
    assert_eq!(stage.get(grouped).unwrap().submobjects(), &letters[..1]);
    anim.interpolate(&mut stage, 0.6);
    assert_eq!(stage.get(grouped).unwrap().submobjects(), &letters[..2]);
    anim.finish(&mut stage);
    assert_eq!(stage.get(grouped).unwrap().submobjects(), &letters[..]);
}

#[test]
fn add_text_letter_by_letter_keeps_explicit_run_time_and_refuses_empty() {
    let mut stage = Stage::new();
    let grouped = stage.add(Mobject::new());
    let letter = line5(&mut stage);
    stage.attach(grouped, letter).unwrap();
    let anim = fmn_anim::add_text_letter_by_letter(&stage, grouped, 0.1, 2.0).unwrap();
    assert!((anim.state().config.run_time - 2.0).abs() < 1e-12);

    let empty = stage.add(Mobject::new());
    match fmn_anim::add_text_letter_by_letter(&stage, empty, 0.1, -1.0) {
        Err(fmn_anim::animation::AnimError::NoLetterGroups) => {}
        other => panic!("expected NoLetterGroups, got {other:?}"),
    }
}
