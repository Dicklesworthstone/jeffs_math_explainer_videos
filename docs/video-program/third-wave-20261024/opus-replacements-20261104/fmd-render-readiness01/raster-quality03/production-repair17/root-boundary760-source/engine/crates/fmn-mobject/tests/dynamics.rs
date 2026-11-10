//! The §8.6 dynamic-behavior corpus (fm-yra acceptance): mixed dt/non-dt
//! ordering, the suspend/resume matrix, snapshot-per-tick add/remove, the
//! C-5 once-only correction, ValueTrackers, `always_redraw`/`f_always`, and
//! the C-6 group-addition correction.

use fmn_mobject::{Mob, Mobject, Stage, StageError, TrackerKind};
use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

fn square() -> Mobject {
    Mobject::from_points(&[
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ])
}

type Log = Rc<RefCell<Vec<String>>>;

fn logger(log: &Log, tag: &str) -> impl FnMut(&mut Stage, Mob) + 'static {
    let log = Rc::clone(log);
    let tag = tag.to_string();
    move |_, _| log.borrow_mut().push(tag.clone())
}

#[test]
fn execution_is_insertion_order_across_mixed_kinds() {
    let mut stage = Stage::new();
    let mob = stage.add(square());
    stage.add_to_scene(mob).unwrap();
    let log: Log = Rc::default();

    stage.add_updater(mob, logger(&log, "a"), false).unwrap();
    {
        let log = Rc::clone(&log);
        stage
            .add_dt_updater(
                mob,
                move |_, _, dt| log.borrow_mut().push(format!("b(dt={dt})")),
                false,
            )
            .unwrap();
    }
    stage.add_updater(mob, logger(&log, "c"), false).unwrap();

    stage.update(0.25);
    assert_eq!(
        log.borrow().as_slice(),
        ["a", "b(dt=0.25)", "c"],
        "one list, insertion order, dt only to the dt kind"
    );
}

#[test]
fn children_update_before_parents() {
    let mut stage = Stage::new();
    let parent = stage.add(Mobject::new());
    let child = stage.add(square());
    let grandchild = stage.add(square());
    stage.attach(parent, child).unwrap();
    stage.attach(child, grandchild).unwrap();
    stage.add_to_scene(parent).unwrap();
    let log: Log = Rc::default();

    stage
        .add_updater(parent, logger(&log, "parent"), false)
        .unwrap();
    stage
        .add_updater(child, logger(&log, "child"), false)
        .unwrap();
    stage
        .add_updater(grandchild, logger(&log, "grandchild"), false)
        .unwrap();

    stage.update(0.1);
    assert_eq!(
        log.borrow().as_slice(),
        ["grandchild", "child", "parent"],
        "the Reference recurses submobjects before running its own updaters"
    );
}

#[test]
fn insert_and_remove_shape_the_list() {
    let mut stage = Stage::new();
    let mob = stage.add(square());
    stage.add_to_scene(mob).unwrap();
    let log: Log = Rc::default();

    let a = stage.add_updater(mob, logger(&log, "a"), false).unwrap();
    let _b = stage.insert_updater(mob, 0, logger(&log, "b")).unwrap();
    let c = stage.add_updater(mob, logger(&log, "c"), false).unwrap();
    assert_eq!(stage.updater_ids(mob), vec![_b, a, c]);

    stage.update(0.0);
    assert_eq!(log.borrow().as_slice(), ["b", "a", "c"]);

    log.borrow_mut().clear();
    stage.remove_updater(mob, a);
    stage.update(0.0);
    assert_eq!(log.borrow().as_slice(), ["b", "c"]);

    stage.clear_updaters(mob, true);
    assert!(stage.updater_ids(mob).is_empty());
    assert!(!stage.has_updaters_in_family(mob));
}

#[test]
fn suspend_resume_matrix() {
    let mut stage = Stage::new();
    let parent = stage.add(Mobject::new());
    let child = stage.add(square());
    stage.attach(parent, child).unwrap();
    stage.add_to_scene(parent).unwrap();
    let log: Log = Rc::default();
    stage
        .add_updater(child, logger(&log, "child"), false)
        .unwrap();

    // Suspending the PARENT (even non-recursively) prunes the subtree: the
    // child's updater cannot run through a suspended ancestor.
    stage.suspend_updating(parent, false);
    assert!(!stage.is_updating_suspended(child));
    stage.update(0.1);
    assert!(
        log.borrow().is_empty(),
        "suspended parent prunes the subtree"
    );

    // Recursive suspension marks the child too.
    stage.suspend_updating(parent, true);
    assert!(stage.is_updating_suspended(child));

    // resume_updating with call_updater runs one immediate update(0) pass.
    stage.resume_updating(parent, true, true);
    assert_eq!(
        log.borrow().as_slice(),
        ["child"],
        "resume ran update(0) once"
    );

    log.borrow_mut().clear();
    stage.update(0.1);
    assert_eq!(
        log.borrow().as_slice(),
        ["child"],
        "updates flow after resume"
    );

    log.borrow_mut().clear();
    stage.update_mobject_with_recurse(parent, 0.2, false);
    assert!(
        log.borrow().is_empty(),
        "recurse=false does not descend into child updaters"
    );
    stage.update_mobject_with_recurse(parent, 0.2, true);
    assert_eq!(log.borrow().as_slice(), ["child"]);
}

#[test]
fn resume_clears_the_ancestor_chain_transitively() {
    let mut stage = Stage::new();
    let grandparent = stage.add(Mobject::new());
    let parent = stage.add(Mobject::new());
    let child = stage.add(square());
    stage.attach(grandparent, parent).unwrap();
    stage.attach(parent, child).unwrap();
    stage.add_to_scene(grandparent).unwrap();

    stage.suspend_updating(grandparent, true);
    // Resuming the CHILD clears its whole ancestor chain (each Reference
    // parent resumes its own parents), without recursing into subtrees.
    stage.resume_updating(child, false, false);
    assert!(!stage.is_updating_suspended(child));
    assert!(!stage.is_updating_suspended(parent));
    assert!(!stage.is_updating_suspended(grandparent));
}

#[test]
fn list_is_snapshotted_per_tick() {
    let mut stage = Stage::new();
    let mob = stage.add(square());
    stage.add_to_scene(mob).unwrap();
    let log: Log = Rc::default();

    // Updater "a" removes itself and registers "d" while iterating: neither
    // change affects THIS tick (the list was snapshotted); both hold next
    // tick.
    let a_id: Rc<RefCell<Option<fmn_mobject::UpdaterId>>> = Rc::default();
    {
        let log = Rc::clone(&log);
        let a_id_inner = Rc::clone(&a_id);
        let id = stage
            .add_updater(
                mob,
                move |stage, me| {
                    log.borrow_mut().push("a".into());
                    if let Some(id) = *a_id_inner.borrow() {
                        stage.remove_updater(me, id);
                    }
                    let log2 = Rc::clone(&log);
                    stage
                        .add_updater(me, move |_, _| log2.borrow_mut().push("d".into()), false)
                        .unwrap();
                },
                false,
            )
            .unwrap();
        *a_id.borrow_mut() = Some(id);
    }
    stage.add_updater(mob, logger(&log, "b"), false).unwrap();

    stage.update(0.1);
    assert_eq!(
        log.borrow().as_slice(),
        ["a", "b"],
        "mid-tick add/remove takes effect next tick"
    );

    log.borrow_mut().clear();
    stage.update(0.1);
    assert_eq!(
        log.borrow().as_slice(),
        ["b", "d"],
        "a removed itself; d joined at the tail"
    );
}

#[test]
fn c5_call_runs_the_update_pass_exactly_once() {
    let mut stage = Stage::new();
    let mob = stage.add(square());
    stage.add_to_scene(mob).unwrap();
    let log: Log = Rc::default();

    // A pre-existing updater sees exactly ONE update(0) pass when a new
    // updater registers with call=true (the Reference runs update(0) and
    // then update() again — the C-5 double call, fixed; Behavior Note).
    stage
        .add_updater(mob, logger(&log, "existing"), false)
        .unwrap();
    stage.add_updater(mob, logger(&log, "new"), true).unwrap();
    assert_eq!(
        log.borrow().as_slice(),
        ["existing", "new"],
        "one update(0) pass, not two"
    );
}

#[test]
fn value_trackers_encode_and_increment() {
    let mut stage = Stage::new();

    let plain = stage.add_value_tracker(2.5);
    assert_eq!(stage.tracker_value(plain), Some(2.5));
    stage.set_tracker_value(plain, -1.0).unwrap();
    stage.increment_tracker_value(plain, 0.25).unwrap();
    assert_eq!(stage.tracker_value(plain), Some(-0.75));

    // Exponential: lane holds ln(value); get decodes; increment acts on the
    // decoded value (set(get + d)), exactly the Reference's composition.
    let exp = stage.add_exponential_value_tracker(4.0);
    let stored = stage.tracker(exp).unwrap();
    assert_eq!(stored.kind, TrackerKind::Exponential);
    assert!((stored.lanes[0] - 4.0f64.ln()).abs() < 1e-12);
    assert!((stage.tracker_value(exp).unwrap() - 4.0).abs() < 1e-12);
    stage.increment_tracker_value(exp, 1.0).unwrap();
    assert!((stage.tracker_value(exp).unwrap() - 5.0).abs() < 1e-12);

    // Complex: two lanes; scalar accessors refuse it.
    let complex = stage.add_complex_value_tracker(1.0, -2.0);
    assert_eq!(stage.tracker_complex_value(complex), Some((1.0, -2.0)));
    assert_eq!(stage.tracker_value(complex), None);
    assert_eq!(
        stage.set_tracker_value(complex, 3.0),
        Err(StageError::StaleHandle)
    );
    stage.set_tracker_complex_value(complex, 0.5, 0.5).unwrap();
    assert_eq!(stage.tracker_complex_value(complex), Some((0.5, 0.5)));

    // Binding-tier detached copies restore the exact encoded payload rather
    // than reconstructing a decoded scalar and losing the tracker kind.
    let restored = stage.tracker(exp).unwrap();
    stage.set_tracker_state(exp, None).unwrap();
    assert_eq!(stage.tracker(exp), None);
    stage.set_tracker_state(exp, Some(restored)).unwrap();
    assert_eq!(stage.tracker(exp), Some(restored));
    assert!((stage.tracker_value(exp).unwrap() - 5.0).abs() < 1e-12);

    // Non-trackers refuse tracker operations.
    let mob = stage.add(square());
    assert_eq!(stage.tracker_value(mob), None);
    assert!(stage.set_tracker_value(mob, 1.0).is_err());
}

#[test]
fn trackers_drive_updaters_through_the_clock() {
    // The canonical composition: a tracker animated (here: incremented by a
    // dt updater) drives a dependent mobject through a non-dt updater.
    let mut stage = Stage::new();
    let tracker = stage.add_value_tracker(0.0);
    let mob = stage.add(square());
    stage.add_to_scene(tracker).unwrap();
    stage.add_to_scene(mob).unwrap();

    stage
        .add_dt_updater(
            tracker,
            move |stage, me, dt| {
                stage.increment_tracker_value(me, dt).unwrap();
            },
            false,
        )
        .unwrap();
    stage
        .add_updater(
            mob,
            move |stage, me| {
                let x = stage.tracker_value(tracker).unwrap();
                stage.set_x(me, x);
            },
            false,
        )
        .unwrap();

    stage.update(0.5);
    stage.update(0.5);
    assert!((stage.tracker_value(tracker).unwrap() - 1.0).abs() < 1e-12);
    assert!((stage.get_x(mob) - 1.0).abs() < 1e-6);
}

#[test]
fn always_redraw_rebuilds_per_tick_without_arena_growth() {
    let mut stage = Stage::new();
    let phase = Rc::new(RefCell::new(0.0_f64));
    let phase_for_closure = Rc::clone(&phase);
    let built = Rc::new(RefCell::new(Vec::new()));
    let built_for_closure = Rc::clone(&built);
    let drawn = stage.always_redraw(move |stage| {
        let x = *phase_for_closure.borrow();
        let fresh = stage.add(Mobject::from_points(&[[x, 0.0, 0.0]]));
        built_for_closure.borrow_mut().push(fresh);
        fresh
    });
    stage.add_to_scene(drawn).unwrap();

    assert!((stage.get_x(drawn) - 0.0).abs() < 1e-6);
    *phase.borrow_mut() = 3.0;
    stage.update(0.1);
    assert!(
        (stage.get_x(drawn) - 3.0).abs() < 1e-6,
        "content rebuilt from the closure on tick"
    );

    // The handle stays stable and the arena stays bounded: replaced content
    // is deleted and its slots recycled.
    let after_two = stage.family(drawn).len();
    for _ in 0..10 {
        stage.update(0.1);
    }
    assert_eq!(stage.family(drawn).len(), after_two);
    let built = built.borrow();
    let (current, retired) = built.split_last().unwrap();
    assert!(stage.contains(*current));
    for &mob in retired {
        assert!(
            !stage.contains(mob),
            "retired content must be reclaimed, not merely detached: {mob:?}"
        );
    }
}

#[test]
fn always_redraw_copies_share_factory_but_retire_only_their_own_content() {
    let mut stage = Stage::new();
    let builds = Rc::new(Cell::new(0));
    let factory_builds = Rc::clone(&builds);
    let original = stage.always_redraw(move |stage| {
        let x = factory_builds.get();
        factory_builds.set(x + 1);
        stage.add(Mobject::from_points(&[[f64::from(x), 0.0, 0.0]]))
    });
    let copy = stage.copy_family(original).unwrap();
    let original_child = stage.get(original).unwrap().submobjects()[0];
    let retired_copy_child = stage.get(copy).unwrap().submobjects()[0];
    stage.update_mobject(copy, 0.25);
    let current_copy_child = stage.get(copy).unwrap().submobjects()[0];
    assert_eq!(stage.get_x(copy), 1.0);
    assert_eq!(stage.get_x(original), 0.0);
    assert!(stage.contains(original_child));
    assert!(!stage.contains(retired_copy_child));
    stage.update_mobject(original, 0.25);
    assert_eq!(stage.get_x(original), 2.0);
    assert_eq!(stage.get_x(copy), 1.0);
    assert!(!stage.contains(original_child));
    assert!(stage.contains(current_copy_child));
    assert_eq!(builds.get(), 3, "factory identity stays shared on copy");
}

#[test]
fn clearing_updaters_during_a_tick_preserves_pending_native_attributes() {
    let mut stage = Stage::new();
    let mob = stage.add(square());
    stage
        .add_updater(mob, |stage, me| stage.clear_updaters(me, false), false)
        .unwrap();
    let native = stage
        .add_dt_updater_with_state(
            mob,
            vec![1.0_f64],
            |_, _| {},
            |stage, me, id, dt| {
                stage
                    .updater_state_mut::<Vec<f64>>(me, id)
                    .unwrap()
                    .push(dt);
            },
            false,
        )
        .unwrap();
    stage.update_mobject(mob, 2.0);
    assert!(stage.updater_ids(mob).is_empty());
    assert_eq!(
        stage.updater_state::<Vec<f64>>(mob, native),
        Some(&vec![1.0, 2.0])
    );
    stage.update_mobject(mob, 4.0);
    assert_eq!(
        stage.updater_state::<Vec<f64>>(mob, native),
        Some(&vec![1.0, 2.0])
    );
    assert_eq!(stage.updater_state::<f64>(mob, native), None);
    assert_eq!(stage.updater_state_mut::<f64>(mob, native), None);
    assert_eq!(
        stage.set_updater_state(mob, native, 10.0, |_, _| {}),
        Err(StageError::UpdaterBindingMismatch),
        "rebinding state requires an installed callable identity"
    );
}

#[test]
fn always_redraw_keeps_reused_content_alive_even_when_pinned() {
    for pinned in [false, true] {
        let mut stage = Stage::new();
        let original = stage.add(square());
        let points = stage.get_points(original).unwrap();
        let drawn = stage.always_redraw(move |_| original);
        stage.add_to_scene(drawn).unwrap();
        if pinned {
            stage.pin(original).unwrap();
        }

        for _ in 0..3 {
            stage.update(0.1);
            assert_eq!(stage.get(drawn).unwrap().submobjects(), &[original]);
            assert_eq!(stage.get_points(original).unwrap(), points);
        }
        if pinned {
            stage.unpin(original);
        }
        assert!(
            stage.contains(original),
            "reused content must never acquire a deferred deletion"
        );
    }
}

#[test]
fn always_redraw_factory_can_reuse_a_detached_descendant() {
    let mut stage = Stage::new();
    let old = stage.add(Mobject::new());
    let reused = stage.add(square());
    let discarded = stage.add(square());
    stage.attach(old, reused).unwrap();
    stage.attach(old, discarded).unwrap();
    let first = Cell::new(true);
    let drawn = stage.always_redraw(move |stage| {
        if first.replace(false) {
            return old;
        }
        assert!(stage.contains(reused), "factory reads the previous family");
        stage.detach(old, reused);
        stage.delete(old).unwrap();
        reused
    });
    stage.add_to_scene(drawn).unwrap();

    stage.update(0.1);

    assert_eq!(stage.get(drawn).unwrap().submobjects(), &[reused]);
    assert!(stage.contains(reused));
    assert!(!stage.contains(old));
    assert!(
        !stage.contains(discarded),
        "capture the old family before the factory removes its wrapper"
    );
}

#[test]
fn always_redraw_preserves_shared_and_rooted_subtrees() {
    for independently_rooted in [false, true] {
        let mut stage = Stage::new();
        let old = stage.add(Mobject::new());
        let shared = stage.add(Mobject::new());
        let leaf = stage.add(square());
        let points = stage.get_points(leaf).unwrap();
        let outside = stage.add(Mobject::new());
        let fresh = stage.add(square());
        stage.attach(old, shared).unwrap();
        stage.attach(shared, leaf).unwrap();
        if independently_rooted {
            stage.add_to_scene(shared).unwrap();
        } else {
            stage.attach(outside, shared).unwrap();
            stage.add_to_scene(outside).unwrap();
        }
        let next = Rc::new(Cell::new(old));
        let next_for_closure = Rc::clone(&next);
        let drawn = stage.always_redraw(move |_| next_for_closure.get());
        next.set(fresh);

        // Keep the independent scene admission while driving this updater.
        stage.update_mobject(drawn, 0.1);

        assert_eq!(stage.get(drawn).unwrap().submobjects(), &[fresh]);
        assert!(!stage.contains(old), "the exclusive wrapper is retired");
        assert!(stage.contains(shared));
        assert!(stage.contains(leaf), "retention protects the full subtree");
        assert_eq!(stage.family(shared), vec![shared, leaf]);
        assert_eq!(stage.get_points(leaf).unwrap(), points);
        if independently_rooted {
            assert_eq!(stage.roots(), &[shared]);
        } else {
            assert_eq!(stage.roots(), &[outside]);
            assert_eq!(stage.get(outside).unwrap().submobjects(), &[shared]);
        }
    }
}

#[test]
fn always_redraw_factory_panic_keeps_published_content() {
    let mut stage = Stage::new();
    let original = stage.add(square());
    let points = stage.get_points(original).unwrap();
    let fail = Rc::new(Cell::new(false));
    let fail_for_closure = Rc::clone(&fail);
    let drawn = stage.always_redraw(move |_| {
        if fail_for_closure.get() {
            std::panic::panic_any("factory refused");
        }
        original
    });
    fail.set(true);

    let failure = catch_unwind(AssertUnwindSafe(|| stage.update_mobject(drawn, 0.1)));
    assert!(failure.is_err());
    assert_eq!(stage.get(drawn).unwrap().submobjects(), &[original]);
    assert_eq!(stage.get_points(original).unwrap(), points);

    fail.set(false);
    stage.update_mobject(drawn, 0.1);
    assert_eq!(stage.get(drawn).unwrap().submobjects(), &[original]);
}

#[test]
fn always_redraw_invalid_replacement_keeps_published_content() {
    for cyclic in [false, true] {
        let mut stage = Stage::new();
        let original = stage.add(square());
        let points = stage.get_points(original).unwrap();
        let stale = stage.add(Mobject::new());
        stage.delete(stale).unwrap();
        let next = Rc::new(Cell::new(original));
        let next_for_closure = Rc::clone(&next);
        let drawn = stage.always_redraw(move |_| next_for_closure.get());
        next.set(if cyclic { drawn } else { stale });

        let failure = catch_unwind(AssertUnwindSafe(|| stage.update_mobject(drawn, 0.1)))
            .expect_err("an invalid factory result must be reported");
        let expected = if cyclic {
            StageError::CycleDetected
        } else {
            StageError::StaleHandle
        };
        assert_eq!(failure.downcast_ref::<StageError>(), Some(&expected));
        assert_eq!(stage.get(drawn).unwrap().submobjects(), &[original]);
        assert_eq!(stage.get_points(original).unwrap(), points);
        assert_eq!(stage.get(original).unwrap().parents(), &[drawn]);

        next.set(original);
        stage.update_mobject(drawn, 0.1);
        assert_eq!(stage.get(drawn).unwrap().submobjects(), &[original]);
    }
}

#[test]
fn always_redraw_rejects_a_stale_initial_result() {
    let mut stage = Stage::new();
    let stale = stage.add(square());
    stage.delete(stale).unwrap();
    let failure = catch_unwind(AssertUnwindSafe(|| stage.always_redraw(move |_| stale)))
        .expect_err("an invalid initial factory result must be reported");
    assert_eq!(
        failure.downcast_ref::<StageError>(),
        Some(&StageError::StaleHandle)
    );
}

#[test]
fn always_redraw_reclaims_private_dag_with_pinned_parent_and_live_view() {
    let mut stage = Stage::new();
    let old = stage.add(Mobject::new());
    let left = stage.add(Mobject::new());
    let right = stage.add(Mobject::new());
    let leaf = stage.add(square());
    let fresh = stage.add(square());
    stage.attach(old, left).unwrap();
    stage.attach(old, right).unwrap();
    stage.attach(left, leaf).unwrap();
    stage.attach(right, leaf).unwrap();
    stage.pin(old).unwrap();
    let view = stage.get_mut(leaf).unwrap().buffer.export_view(false);
    let next = Rc::new(Cell::new(old));
    let next_for_closure = Rc::clone(&next);
    let drawn = stage.always_redraw(move |_| next_for_closure.get());
    next.set(fresh);

    stage.update_mobject(drawn, 0.1);

    assert_eq!(stage.get(drawn).unwrap().submobjects(), &[fresh]);
    assert!(stage.contains(old), "the explicit pin defers destruction");
    for retired in [left, right, leaf] {
        assert!(
            !stage.contains(retired),
            "a pending parent or a second private DAG edge cannot leak {retired:?}"
        );
    }
    assert_eq!(view.read(0, "point"), Some(vec![-0.5, -0.5, 0.0]));
    stage.unpin(old);
    assert!(!stage.contains(old));
    assert!(stage.contains(fresh));
}

#[test]
fn f_always_binds_the_closure_into_the_clock() {
    let mut stage = Stage::new();
    let mob = stage.add(square());
    stage.add_to_scene(mob).unwrap();
    stage
        .f_always(mob, |stage, me| {
            stage.shift(me, [1.0, 0.0, 0.0]);
        })
        .unwrap();
    let x0 = stage.get_x(mob);
    stage.update(0.1);
    stage.update(0.1);
    assert!((stage.get_x(mob) - x0 - 2.0).abs() < 1e-6);
}

#[test]
fn c6_group_add_is_a_value_operation() {
    let mut stage = Stage::new();
    let a1 = stage.add(square());
    let a2 = stage.add(square());
    let group = stage.group_add(a1, a2).unwrap();
    assert_ne!(group, a1);
    assert_eq!(stage.get(group).unwrap().submobjects(), &[a1, a2]);

    // The C-6 case: the left operand IS a group. The Reference's
    // Group.__add__ would mutate `group` in place; the correction builds a
    // new group and leaves the operand untouched (Behavior Note).
    let b = stage.add(square());
    let combined = stage.group_add(group, b).unwrap();
    assert_ne!(combined, group, "a new group, not the mutated left operand");
    assert_eq!(
        stage.get(group).unwrap().submobjects(),
        &[a1, a2],
        "the existing group is untouched"
    );
    assert_eq!(stage.get(combined).unwrap().submobjects(), &[group, b]);

    // Dead operands are refused.
    let dead = stage.add(square());
    stage.delete(dead).unwrap();
    assert_eq!(stage.group_add(a1, dead), Err(StageError::StaleHandle));
}

#[test]
fn animating_status_covers_family_and_all_ancestors() {
    // §9.1's set_animating_status: family (per recurse) + the transitive
    // ancestor closure — including a diamond's second parent.
    let mut stage = Stage::new();
    let child = stage.add(square());
    let parent_a = stage.add(Mobject::new());
    let parent_b = stage.add(Mobject::new());
    let grandparent = stage.add(Mobject::new());
    stage.attach(parent_a, child).unwrap();
    stage.attach(parent_b, child).unwrap();
    stage.attach(grandparent, parent_a).unwrap();

    stage.set_animating_status(child, true, true);
    for mob in [child, parent_a, parent_b, grandparent] {
        assert!(stage.is_animating(mob), "{mob:?} must be marked");
    }
    stage.set_animating_status(child, false, true);
    for mob in [child, parent_a, parent_b, grandparent] {
        assert!(!stage.is_animating(mob), "{mob:?} must be cleared");
    }

    // Non-recursive: the mobject itself + ancestors, not the subtree.
    let inner = stage.add(square());
    stage.attach(child, inner).unwrap();
    stage.set_animating_status(child, true, false);
    assert!(stage.is_animating(child));
    assert!(stage.is_animating(parent_a), "ancestors always marked");
    assert!(
        !stage.is_animating(inner),
        "subtree untouched without recurse"
    );
}

#[test]
fn is_changing_reads_animating_or_own_updaters() {
    let mut stage = Stage::new();
    let mob = stage.add(square());
    assert!(!stage.is_changing(mob));

    let id = stage.add_updater(mob, |_stage, _mob| {}, false).unwrap();
    assert!(stage.is_changing(mob), "own updaters count");
    stage.remove_updater(mob, id);
    assert!(!stage.is_changing(mob));

    stage.set_animating_status(mob, true, true);
    assert!(stage.is_changing(mob), "animating counts");

    // Self-only, as in the Reference: a child's updater does not make the
    // parent 'changing'.
    let child = stage.add(square());
    stage.attach(mob, child).unwrap();
    stage.set_animating_status(mob, false, true);
    stage.add_updater(child, |_stage, _mob| {}, false).unwrap();
    assert!(stage.is_changing(child));
    assert!(!stage.is_changing(mob), "is_changing is self-only");
}
