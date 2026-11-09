//! fm-fc53: point-free layout anchors are positions, not phantom geometry.
use fmn_core::constants::{DOWN, ORIGIN, OUT};
use fmn_mobject::{Mobject, Placement, Snapshot, Stage};

#[test]
fn empty_layout_anchor_supports_shift_move_to_and_next_to() {
    let mut stage = Stage::new();
    let anchor = stage.add(Mobject::new());
    let square = stage.add(Mobject::from_points(&[[-1.0, -1.0, 0.0], [1.0, 1.0, 0.0]]));
    let revision = stage.get(anchor).unwrap().buffer.revision();
    assert_eq!(stage.get_center(anchor), ORIGIN);

    stage.shift(anchor, [2.0, 0.0, 0.0]);
    assert_eq!(stage.get_center(anchor), [2.0, 0.0, 0.0]);
    stage.move_to(anchor, [0.0, 3.0, 0.0], ORIGIN);
    assert_eq!(stage.get_center(anchor), [0.0, 3.0, 0.0]);
    stage.next_to(anchor, square, DOWN, 0.25, ORIGIN);
    assert_eq!(stage.get_center(anchor), [0.0, -1.25, 0.0]);

    let bbox = stage.get_bounding_box(anchor);
    assert_eq!(bbox.min, bbox.mid);
    assert_eq!(bbox.mid, bbox.max);
    assert_eq!(stage.get_points(anchor).unwrap(), Vec::<[f64; 3]>::new());
    assert_eq!(stage.get(anchor).unwrap().buffer.revision(), revision);
}

#[test]
fn empty_anchor_affine_operations_use_the_requested_pivot() {
    let mut stage = Stage::new();
    let anchor = stage.add(Mobject::new());
    stage.shift(anchor, [2.0, 3.0, 0.0]);
    stage.scale(anchor, 7.0); // Scaling about its own center must not move it.
    assert_eq!(stage.get_center(anchor), [2.0, 3.0, 0.0]);
    stage.scale_about(anchor, 2.0, Some(ORIGIN), None);
    assert_eq!(stage.get_center(anchor), [4.0, 6.0, 0.0]);
    // Rotation is not box-preserving: the Reference recomputes the empty
    // family's box, which is the origin (measured at the pin, see
    // `reference_measured` below).
    stage.rotate(anchor, std::f64::consts::FRAC_PI_2, OUT, Some(ORIGIN), None);
    assert_eq!(stage.get_center(anchor), ORIGIN);
}

#[test]
fn empty_points_readback_and_live_views_do_not_erase_the_anchor() {
    let mut stage = Stage::new();
    let anchor = stage.add(Mobject::new());
    stage.shift(anchor, [2.0, 3.0, 4.0]);
    assert!(!stage.bake_placement(anchor).unwrap());
    let view = stage
        .get_mut(anchor)
        .unwrap()
        .buffer
        .export_field_view("point", true)
        .unwrap();
    assert_eq!(stage.get_center(anchor), [2.0, 3.0, 4.0]);
    stage.shift(anchor, [1.0, -2.0, 3.0]);
    stage.bake_family_placements(anchor).unwrap();
    assert_eq!(stage.get_center(anchor), [3.0, 1.0, 7.0]);
    assert!(view.is_attached_to(&stage.get(anchor).unwrap().buffer));
    assert_eq!(stage.get(anchor).unwrap().buffer.len(), 0);
    drop(view);
    assert_eq!(stage.get_center(anchor), [3.0, 1.0, 7.0]);
}

#[test]
fn empty_parents_and_siblings_do_not_expand_real_geometry_bounds() {
    let mut stage = Stage::new();
    let parent = stage.add(Mobject::new());
    let empty = stage.add(Mobject::new());
    let child = stage.add(Mobject::from_points(&[
        [10.0, 12.0, 0.0],
        [20.0, 14.0, 0.0],
    ]));
    stage.shift(parent, [-100.0, -100.0, 0.0]);
    stage.shift(empty, [100.0, 100.0, 0.0]);
    stage.attach(parent, empty).unwrap();
    stage.attach(parent, child).unwrap();
    assert_eq!(
        stage.get_bounding_box(parent),
        stage.get_bounding_box(child)
    );
    assert_eq!(stage.get_center(parent), [15.0, 13.0, 0.0]);
    stage.shift(parent, [1.0, 2.0, 3.0]);
    assert_eq!(stage.get_center(parent), [16.0, 15.0, 3.0]);
    assert_eq!(
        stage.get_bounding_box(parent),
        stage.get_bounding_box(child)
    );
    assert_eq!(stage.get_center(empty), [101.0, 102.0, 3.0]);
}

#[test]
fn empty_anchor_cache_invalidates_on_placement_and_geometry_changes() {
    let mut stage = Stage::new();
    let anchor = stage.add(Mobject::new());
    assert_eq!(stage.get_center(anchor), ORIGIN);
    let first = stage.bbox_materializations(anchor);
    assert_eq!(stage.get_center(anchor), ORIGIN);
    assert_eq!(stage.bbox_materializations(anchor), first);
    stage.shift(anchor, [8.0, 4.0, 0.0]);
    assert_eq!(stage.get_center(anchor), [8.0, 4.0, 0.0]);
    assert_eq!(stage.bbox_materializations(anchor), first + 1);
    stage.set_points(anchor, &[[1.0, 2.0, 3.0]]).unwrap();
    assert_eq!(stage.placement(anchor), Some(Placement::IDENTITY));
    assert_eq!(stage.get_center(anchor), [1.0, 2.0, 3.0]);
    stage.set_points(anchor, &[]).unwrap();
    assert_eq!(stage.get_center(anchor), ORIGIN);
}

#[test]
fn empty_anchor_survives_copy_snapshot_restore_and_durable_decode() {
    let mut stage = Stage::new();
    let anchor = stage.add(Mobject::new());
    stage.shift(anchor, [2.0, -3.0, 4.0]);
    stage.add_to_scene(anchor).unwrap();
    let copy = stage.copy_family(anchor).unwrap();
    assert_eq!(stage.get_center(copy), [2.0, -3.0, 4.0]);
    let snapshot = stage.snapshot();
    let bytes = snapshot.to_bytes().unwrap();
    stage.shift(anchor, [10.0, 20.0, 30.0]);
    assert_eq!(snapshot.materialize().get_center(anchor), [2.0, -3.0, 4.0]);
    stage.restore(&snapshot);
    assert_eq!(stage.get_center(anchor), [2.0, -3.0, 4.0]);
    let decoded = Snapshot::from_bytes(&bytes, &stage).unwrap();
    let replay = decoded.snapshot.materialize();
    assert_eq!(replay.get_center(anchor), [2.0, -3.0, 4.0]);
    assert_eq!(replay.get_center(copy), [2.0, -3.0, 4.0]);
    assert_eq!(replay.snapshot_bytes().unwrap(), bytes);
}

/// Each case is the Reference's measured result at the pin
/// (3b1b/manim@6199a00d, `get_center()`/`get_bounding_box()` of the same
/// construction, identical detached and scene-bound), re-derived through the
/// Stage API. Values are exact in the Reference up to its f32/f64 rounding.
mod reference_measured {
    use fmn_core::constants::{DOWN, LEFT, ORIGIN, OUT, RIGHT, UL, UP, UR};
    use fmn_mobject::{Mob, Mobject, Stage};
    use std::f64::consts::{FRAC_PI_2, PI};

    fn close(actual: [f64; 3], expected: [f64; 3]) {
        for (a, e) in actual.into_iter().zip(expected) {
            assert!((a - e).abs() < 1e-6, "{actual:?} != {expected:?}");
        }
    }

    fn empty(stage: &mut Stage) -> Mob {
        stage.add(Mobject::new())
    }

    fn shifted(stage: &mut Stage, v: [f64; 3]) -> Mob {
        let mob = empty(stage);
        stage.shift(mob, v);
        mob
    }

    /// The Reference `Square()` box: side 2 about the origin.
    fn square(stage: &mut Stage) -> Mob {
        stage.add(Mobject::from_points(&[
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
        ]))
    }

    fn group(stage: &mut Stage, children: &[Mob]) -> Mob {
        let parent = empty(stage);
        for &child in children {
            stage.attach(parent, child).unwrap();
        }
        parent
    }

    #[test]
    fn positioning_moves_the_empty_box() {
        let mut stage = Stage::new();
        let sq = square(&mut stage);
        let shifted_left = square(&mut stage);
        stage.shift(shifted_left, LEFT);
        let up2 = square(&mut stage);
        stage.shift(up2, [0.0, 2.0, 0.0]);
        let left5 = square(&mut stage);
        stage.shift(left5, [-5.0, 0.0, 0.0]);

        let g = empty(&mut stage);
        stage.shift(g, [2.0, 0.0, 0.0]);
        close(stage.get_center(g), [2.0, 0.0, 0.0]); // shift
        stage.shift(g, [-1.0, 0.0, 0.0]);
        stage.move_to(g, [0.0, 3.0, 0.0], ORIGIN);
        close(stage.get_center(g), [0.0, 3.0, 0.0]); // move_to

        let g = empty(&mut stage);
        stage.move_to(g, shifted_left, UL);
        close(stage.get_center(g), [-2.0, 1.0, 0.0]); // move_to_mob_aligned_edge
        let g = empty(&mut stage);
        stage.next_to(g, sq, DOWN, 0.25, ORIGIN);
        close(stage.get_center(g), [0.0, -1.25, 0.0]); // next_to_square_down
        let g = empty(&mut stage);
        stage.next_to(g, [1.0, 1.0, 0.0], UR, 0.5, LEFT);
        close(stage.get_center(g), [1.5, 1.5, 0.0]); // next_to_point_aligned
        let target = shifted(&mut stage, [2.0, 0.0, 0.0]);
        let s = square(&mut stage);
        stage.next_to(s, target, RIGHT, 0.25, ORIGIN);
        close(stage.get_center(s), [3.25, 0.0, 0.0]); // square_next_to_empty

        let g = empty(&mut stage);
        stage.align_to(g, up2, UP);
        close(stage.get_center(g), [0.0, 3.0, 0.0]); // align_to_mob_up
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.align_to(g, [-3.0, 7.0, 0.0], LEFT);
        close(stage.get_center(g), [-3.0, 3.0, 4.0]); // align_to_point_left

        let g = empty(&mut stage);
        stage.to_edge(g, UP, 0.5);
        close(stage.get_center(g), [0.0, 3.5, 0.0]); // to_edge_up
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.to_edge(g, LEFT, 1.0);
        close(stage.get_center(g), [-6.111_111, 3.0, 4.0]); // to_edge_left_buff
        let g = empty(&mut stage);
        stage.to_corner(g, UR, 0.5);
        close(stage.get_center(g), [6.611_111, 3.5, 0.0]); // to_corner_ur

        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.center(g);
        close(stage.get_center(g), ORIGIN); // center_after_shift
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.set_x(g, -1.0).set_y(g, -2.0).set_z(g, -3.0);
        close(stage.get_center(g), [-1.0, -2.0, -3.0]); // set_x_y_z
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.match_x(g, left5);
        close(stage.get_center(g), [-5.0, 3.0, 4.0]); // match_x
        let g = shifted(&mut stage, [1.0, 0.0, 0.0]);
        stage.set_width(g, 3.0, false);
        close(stage.get_center(g), [1.0, 0.0, 0.0]); // set_width_empty
    }

    #[test]
    fn box_preserving_maps_carry_the_position() {
        let mut stage = Stage::new();
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.scale(g, 7.0);
        close(stage.get_center(g), [2.0, 3.0, 4.0]); // scale_about_center
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.scale_about(g, 3.0, None, Some(UL));
        close(stage.get_center(g), [2.0, 3.0, 4.0]); // scale_about_edge
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.scale_about(g, 2.0, Some(ORIGIN), None);
        close(stage.get_center(g), [4.0, 6.0, 8.0]); // scale_about_origin
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.stretch_about(g, 2.0, 0, Some(ORIGIN), None);
        close(stage.get_center(g), [4.0, 3.0, 4.0]); // stretch_about_origin
    }

    #[test]
    fn maps_that_recompute_the_box_return_it_to_the_origin() {
        let mut stage = Stage::new();
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.rotate(g, FRAC_PI_2, OUT, Some(ORIGIN), None);
        close(stage.get_center(g), ORIGIN); // rotate_about_origin
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.rotate(g, FRAC_PI_2, OUT, None, None);
        close(stage.get_center(g), ORIGIN); // rotate_about_center
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.rotate(g, FRAC_PI_2, OUT, Some([1.0, 1.0, 0.0]), None);
        close(stage.get_center(g), ORIGIN); // rotate_about_point
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.rotate(g, PI, UP, None, None);
        close(stage.get_center(g), ORIGIN); // flip
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.apply_affine(
            g,
            fmn_mobject::Placement::new(
                [[2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 1.0]],
                ORIGIN,
            ),
        );
        close(stage.get_center(g), ORIGIN); // apply_matrix
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.apply_points_function(
            g,
            |p| [2.0 * p[0], 2.0 * p[1], 2.0 * p[2]],
            Some(ORIGIN),
            None,
        );
        close(stage.get_center(g), ORIGIN); // apply_function
        let g = shifted(&mut stage, [1.0, 0.0, 0.0]);
        stage.refresh_bounding_box(g, false, true);
        close(stage.get_center(g), ORIGIN); // explicit_refresh
    }

    #[test]
    fn getters_report_the_degenerate_box() {
        let mut stage = Stage::new();
        let fresh = empty(&mut stage);
        let b = stage.get_bounding_box(fresh);
        assert_eq!((b.min, b.mid, b.max), (ORIGIN, ORIGIN, ORIGIN)); // fresh_getters

        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        let p = [2.0, 3.0, 4.0];
        let b = stage.get_bounding_box(g);
        assert_eq!((b.min, b.mid, b.max), (p, p, p));
        for point in [
            stage.get_center(g),
            stage.get_top(g),
            stage.get_bottom(g),
            stage.get_left(g),
            stage.get_right(g),
            stage.get_zenith(g),
            stage.get_nadir(g),
            stage.get_corner(g, UR),
            stage.get_edge_center(g, DOWN),
            stage.get_bounding_box_point(g, UL),
        ] {
            assert_eq!(point, p);
        }
        assert_eq!(
            [stage.get_width(g), stage.get_height(g), stage.get_depth(g)],
            [0.0; 3]
        );
        assert_eq!([stage.get_x(g), stage.get_y(g), stage.get_z(g)], p);
        assert_eq!(stage.get_coord(g, 1, UP), 3.0);
    }

    #[test]
    fn families_keep_each_members_own_box() {
        let mut stage = Stage::new();
        // nested_empty_shift
        let inner = empty(&mut stage);
        let outer = group(&mut stage, &[inner]);
        stage.shift(outer, RIGHT);
        close(stage.get_center(outer), [1.0, 0.0, 0.0]);
        close(stage.get_center(inner), [1.0, 0.0, 0.0]);
        // child_shift_refreshes_empty_parent
        stage.shift(inner, RIGHT);
        close(stage.get_center(inner), [2.0, 0.0, 0.0]);
        close(stage.get_center(outer), ORIGIN);

        // empty_child_only: a parent's box is its own, not its child's.
        let inner = shifted(&mut stage, [5.0, 0.0, 0.0]);
        let outer = group(&mut stage, &[inner]);
        close(stage.get_center(outer), ORIGIN);
        close(stage.get_center(inner), [5.0, 0.0, 0.0]);

        // shift_parent_with_shifted_empty_child_and_square
        let inner = shifted(&mut stage, [5.0, 0.0, 0.0]);
        let sq = square(&mut stage);
        let outer = group(&mut stage, &[inner, sq]);
        close(stage.get_center(outer), ORIGIN);
        stage.shift(outer, UP);
        close(stage.get_center(outer), [0.0, 1.0, 0.0]);
        close(stage.get_center(inner), [5.0, 1.0, 0.0]);

        // scale_family_with_empty_child
        let inner = shifted(&mut stage, RIGHT);
        let sq = square(&mut stage);
        let g = group(&mut stage, &[inner, sq]);
        stage.scale(g, 2.0);
        close(stage.get_center(inner), [2.0, 0.0, 0.0]);
        let b = stage.get_bounding_box(g);
        close(b.min, [-2.0, -2.0, 0.0]);
        close(b.max, [2.0, 2.0, 0.0]);

        // rotate_family_with_empty_child
        let inner = shifted(&mut stage, RIGHT);
        let sq = square(&mut stage);
        let g = group(&mut stage, &[inner, sq]);
        stage.rotate(g, FRAC_PI_2, OUT, None, None);
        close(stage.get_center(inner), ORIGIN);
        close(stage.get_center(g), ORIGIN);

        // flip_family_with_empty_child_and_points
        let inner = shifted(&mut stage, RIGHT);
        let sq = square(&mut stage);
        stage.shift(sq, LEFT);
        let g = group(&mut stage, &[inner, sq]);
        stage.rotate(g, PI, RIGHT, None, None);
        close(stage.get_center(inner), ORIGIN);
    }

    #[test]
    fn arrange_places_empty_members() {
        let mut stage = Stage::new();
        // arrange_with_empty_member: Square(), VGroup(), Circle() (radius 1).
        let a = square(&mut stage);
        let e = empty(&mut stage);
        let c = square(&mut stage);
        let g = group(&mut stage, &[a, e, c]);
        stage.arrange(g, RIGHT, 0.25, true);
        close(stage.get_center(a), [-1.25, 0.0, 0.0]);
        close(stage.get_center(e), ORIGIN);
        close(stage.get_center(c), [1.25, 0.0, 0.0]);
        close(stage.get_center(g), ORIGIN);

        // arrange_all_empty
        let members = [empty(&mut stage), empty(&mut stage), empty(&mut stage)];
        let g = group(&mut stage, &members);
        stage.arrange(g, DOWN, 0.25, true);
        close(stage.get_center(members[0]), ORIGIN);
        close(stage.get_center(members[1]), [0.0, -0.25, 0.0]);
        close(stage.get_center(members[2]), [0.0, -0.5, 0.0]);
        close(stage.get_center(g), ORIGIN);

        // arrange_in_grid_empties (2x2, buff 1)
        let cells = [
            empty(&mut stage),
            empty(&mut stage),
            empty(&mut stage),
            empty(&mut stage),
        ];
        let g = group(&mut stage, &cells);
        stage.arrange_in_grid(g, 2, 2, 1.0, 1.0, ORIGIN);
        let expected = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
        ];
        for (&cell, want) in cells.iter().zip(expected) {
            close(stage.get_center(cell), want);
        }
    }

    #[test]
    fn copies_and_states_carry_the_position_and_point_edits_refresh_it() {
        let mut stage = Stage::new();
        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        let copy = stage.copy_family(g).unwrap();
        close(stage.get_center(copy), [2.0, 3.0, 4.0]); // copy_keeps_position
        stage.save_state(g).unwrap();
        stage.shift(g, LEFT);
        stage.restore_mobject(g).unwrap();
        close(stage.get_center(g), [2.0, 3.0, 4.0]); // save_state_restore
        let target = stage.generate_target(g).unwrap();
        stage.shift(target, DOWN);
        close(stage.get_center(target), [2.0, 2.0, 4.0]); // generate_target
        close(stage.get_center(g), [2.0, 3.0, 4.0]);
        let receiver = empty(&mut stage);
        stage.become_mobject(receiver, g, false).unwrap();
        close(stage.get_center(receiver), [2.0, 3.0, 4.0]); // become_empty

        let g = shifted(&mut stage, [2.0, 3.0, 4.0]);
        stage.set_points(g, &[[1.0, 2.0, 3.0]]).unwrap();
        close(stage.get_center(g), [1.0, 2.0, 3.0]); // set_points_after_shift
        let m = stage.add(Mobject::from_points(&[[1.0, 2.0, 3.0]]));
        stage.shift(m, RIGHT);
        stage.set_points(m, &[]).unwrap();
        close(stage.get_center(m), ORIGIN); // shift_then_clear_points

        // match_points_empty_source
        let source = shifted(&mut stage, [0.0, 3.0, 0.0]);
        let m = stage.add(Mobject::from_points(&[
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ]));
        stage.match_points(m, source);
        close(stage.get_center(m), ORIGIN);
    }
}
