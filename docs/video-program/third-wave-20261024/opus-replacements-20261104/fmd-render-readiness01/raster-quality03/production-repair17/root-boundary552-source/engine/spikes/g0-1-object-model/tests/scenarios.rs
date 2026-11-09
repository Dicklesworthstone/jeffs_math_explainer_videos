//! The ten lifetime scenarios of G0-1 (fm-dzv), one test each, in the
//! bead's order. Together with the fluent-API test at the bottom they are
//! the executable half of the D-11 ratification note.

use fmn_core::constants::{BLUE, PI, RED};
use fmn_spike_object_model::{Mobject, Square, Stage};

// Scenario 1: detached construction — a mobject built before any scene
// exists, composed while detached, added later.
#[test]
fn s1_detached_construction() {
    let square = Square::new().side_length(2.0).color(BLUE).build();
    let dot = Mobject::from_points(&[[0.0, 0.0, 0.0]], RED);
    let group = Mobject::group(vec![square, dot]);
    // No stage anywhere yet.
    let mut stage = Stage::new();
    let g = stage.add(group);
    assert_eq!(stage.family(g).len(), 3);
    let children = stage.get(g).unwrap().submobjects.clone();
    assert_eq!(children.len(), 2);
    assert_eq!(stage.get(children[0]).unwrap().buffer.len(), 4);
    assert_eq!(stage.get(children[1]).unwrap().buffer.len(), 1);
}

// Scenario 2: cross-group composition — one mobject participates in groups
// assembled independently, and regrouping is edge surgery, not data moves.
#[test]
fn s2_cross_group_composition() {
    let mut stage = Stage::new();
    let a = stage.add(Square::new());
    let b = stage.add(Square::new());
    let c = stage.add(Square::new());
    let group1 = stage.add(Mobject::group(vec![]));
    let group2 = stage.add(Mobject::group(vec![]));
    stage.attach(group1, a);
    stage.attach(group1, b);
    stage.attach(group2, b); // b is in both groups
    stage.attach(group2, c);
    assert_eq!(stage.family(group1), vec![group1, a, b]);
    assert_eq!(stage.family(group2), vec![group2, b, c]);
    // Move a from group1 to group2: pure edge surgery.
    stage.detach(group1, a);
    stage.attach(group2, a);
    assert_eq!(stage.family(group1), vec![group1, b]);
    assert_eq!(stage.family(group2), vec![group2, b, c, a]);
}

// Scenario 3: multiple parents — a shared submobject appears once per
// family traversal (diamond composition), and each parent sees it.
#[test]
fn s3_multiple_parents() {
    let mut stage = Stage::new();
    let shared = stage.add(Square::new());
    let left = stage.add(Mobject::group(vec![]));
    let right = stage.add(Mobject::group(vec![]));
    let top = stage.add(Mobject::group(vec![]));
    stage.attach(left, shared);
    stage.attach(right, shared);
    stage.attach(top, left);
    stage.attach(top, right);
    assert_eq!(stage.get(shared).unwrap().parents.len(), 2);
    // The diamond: shared is visited exactly once from the top.
    let family = stage.family(top);
    assert_eq!(family.iter().filter(|m| **m == shared).count(), 1);
    assert_eq!(family.len(), 4);
}

// Scenario 4: removal from a scene with live handles outstanding — scene
// membership is a root set, not ownership.
#[test]
fn s4_removal_with_live_handles() {
    let mut stage = Stage::new();
    let square = stage.add(Square::new());
    stage.add_to_scene(square);
    assert_eq!(stage.roots(), &[square]);
    stage.remove_from_scene(square);
    // The handle still resolves; the data is intact.
    assert!(stage.contains(square));
    assert_eq!(stage.get(square).unwrap().buffer.len(), 4);
    // Re-adding works; the handle never changed.
    stage.add_to_scene(square);
    assert_eq!(stage.roots(), &[square]);
}

// Scenario 5: the two-scene policy — handles are stage-scoped; foreign
// handles never resolve; content crosses stages only by copy.
#[test]
fn s5_two_scene_policy() {
    let mut stage_a = Stage::new();
    let mut stage_b = Stage::new();
    let square = stage_a.add(Square::new());
    // The same handle means nothing to another stage.
    assert!(!stage_b.contains(square));
    assert!(stage_b.get(square).is_none());
    assert!(!stage_b.add_to_scene(square));
    // Transfer is copy: independent data, native handle.
    let copied = stage_a.copy_into(square, &mut stage_b).unwrap();
    assert!(stage_b.contains(copied));
    stage_b
        .get_mut(copied)
        .unwrap()
        .buffer
        .write(0, "point", &[9.0, 9.0, 9.0]);
    assert_ne!(
        stage_a.get(square).unwrap().buffer.read(0, "point"),
        stage_b.get(copied).unwrap().buffer.read(0, "point")
    );
}

// Scenario 6: copy() — family-internal references remap, record data is
// independent, updater callables are shared by reference.
#[test]
fn s6_copy_remapping() {
    let mut stage = Stage::new();
    let parent = stage.add(Mobject::group(vec![]));
    let child = stage.add(Square::new());
    stage.attach(parent, child);
    stage.add_updater(child, |_, _, _| {}, false);

    let copy = stage.copy_family(parent).unwrap();
    assert_ne!(copy, parent);
    let copy_children = stage.get(copy).unwrap().submobjects.clone();
    assert_eq!(copy_children.len(), 1);
    // Family-internal reference remapped to the copied child…
    assert_ne!(copy_children[0], child);
    // …whose parent edge points back inside the copy, not at the original.
    assert_eq!(stage.get(copy_children[0]).unwrap().parents, vec![copy]);
    // Record data is independent.
    stage
        .get_mut(child)
        .unwrap()
        .buffer
        .write(0, "point", &[7.0, 7.0, 7.0]);
    assert_ne!(
        stage.get(child).unwrap().buffer.read(0, "point"),
        stage.get(copy_children[0]).unwrap().buffer.read(0, "point")
    );
}

// Scenario 7: proxy identity across collection round-trips — a pin keeps
// the entry (and thus handle → object identity) alive through removal and
// even a requested delete; the delete completes only at the last unpin.
#[test]
fn s7_proxy_identity_across_collection() {
    let mut stage = Stage::new();
    let square = stage.add(Square::new());
    stage.add_to_scene(square);
    // The Python bridge would mint a proxy here: one pin per proxy.
    assert!(stage.pin(square));
    // Scene round-trip: identity survives.
    stage.remove_from_scene(square);
    stage.add_to_scene(square);
    assert!(stage.contains(square));
    // A delete while pinned defers.
    assert!(stage.delete(square));
    assert!(stage.contains(square), "delete must defer while pinned");
    // The last unpin finalizes: the handle goes stale atomically.
    stage.unpin(square);
    assert!(!stage.contains(square));
    assert!(stage.get(square).is_none());
}

// Scenario 8: the §8.2 view protocol across write / resize / snapshot.
#[test]
fn s8_view_protocol() {
    let mut stage = Stage::new();
    let square = stage.add(Square::new());

    // Export a writable view (the NumPy structured view stands here).
    let entry = stage.get_mut(square).unwrap();
    let rev0 = entry.buffer.revision();
    let view = entry.buffer.export_view(true);
    assert_eq!(entry.buffer.live_view_count(), 1);

    // (a) Mutation through the view is visible to the engine and marks
    // render state dirty (revision bump).
    assert!(view.write(0, "point", &[5.0, 5.0, 0.0]));
    assert_eq!(entry.buffer.read(0, "point").unwrap(), vec![5.0, 5.0, 0.0]);
    assert!(entry.buffer.revision() > rev0);

    // (b) Engine writes are visible through the live view.
    entry.buffer.write(1, "point", &[6.0, 6.0, 0.0]);
    assert_eq!(view.read(1, "point").unwrap(), vec![6.0, 6.0, 0.0]);

    // (c) Copy-on-resize: the view detaches with NumPy-natural semantics —
    // it keeps the old generation; reallocation under it never happened.
    let old_storage_data = view.read(0, "point").unwrap();
    entry.buffer.resize(8);
    assert!(!view.is_attached_to(&entry.buffer));
    assert_eq!(view.read(0, "point").unwrap(), old_storage_data);
    // Post-resize engine writes no longer reach the detached view.
    entry.buffer.write(0, "point", &[1.0, 2.0, 3.0]);
    assert_eq!(view.read(0, "point").unwrap(), old_storage_data);
    // Growth is null-padded.
    assert_eq!(entry.buffer.read(7, "point").unwrap(), vec![0.0, 0.0, 0.0]);

    // (d) Rule 5: snapshots never share a generation with a live view.
    let fresh_view = entry.buffer.export_view(true);
    let snapshot = stage.snapshot();
    let entry = stage.get_mut(square).unwrap();
    // The write after the snapshot is visible to the live view…
    entry.buffer.write(2, "point", &[4.0, 4.0, 4.0]);
    assert_eq!(fresh_view.read(2, "point").unwrap(), vec![4.0, 4.0, 4.0]);
    // …and invisible to the snapshot (restore proves isolation).
    stage.restore(&snapshot);
    let entry = stage.get(square).unwrap();
    assert_ne!(
        entry.buffer.read(2, "point").unwrap(),
        vec![4.0, 4.0, 4.0],
        "snapshot must not have seen the post-snapshot write"
    );
}

// Scenario 9: updater closures capturing handles — insertion order, dt
// plumbing, and captured handles staying valid because they are Copy
// values resolved through the stage at call time.
#[test]
fn s9_updater_closures() {
    let mut stage = Stage::new();
    let follower = stage.add(Square::new());
    let leader = stage.add(Mobject::from_points(&[[10.0, 0.0, 0.0]], RED));
    stage.add_to_scene(follower);
    stage.add_to_scene(leader);

    // The closure captures the leader's handle.
    stage.add_updater(
        follower,
        move |stage, me, dt| {
            let target = stage.get(leader).unwrap().buffer.read(0, "point").unwrap();
            let entry = stage.get_mut(me).unwrap();
            let current = entry.buffer.read(0, "point").unwrap();
            let step = dt as f32;
            entry.buffer.write(
                0,
                "point",
                &[
                    current[0] + step * (target[0] - current[0]),
                    current[1] + step * (target[1] - current[1]),
                    current[2] + step * (target[2] - current[2]),
                ],
            );
        },
        false,
    );
    let before = stage
        .get(follower)
        .unwrap()
        .buffer
        .read(0, "point")
        .unwrap();
    stage.update(0.5);
    let after = stage
        .get(follower)
        .unwrap()
        .buffer
        .read(0, "point")
        .unwrap();
    assert!(
        after[0] > before[0],
        "updater must move follower toward leader"
    );
    assert!((stage.time() - 0.5).abs() < 1e-12);

    // add_updater(call_now = true) runs exactly once — the Reference's
    // double-call is a bug we fix (Behavior Note).
    let counter = std::rc::Rc::new(std::cell::Cell::new(0));
    let seen = std::rc::Rc::clone(&counter);
    stage.add_updater(follower, move |_, _, _| seen.set(seen.get() + 1), true);
    assert_eq!(counter.get(), 1);
}

// Scenario 10: snapshot/restore is CoW — cheap to take, isolated under
// mutation, and exact on restore.
#[test]
fn s10_snapshot_restore() {
    let mut stage = Stage::new();
    let a = stage.add(Square::new());
    let b = stage.add(Square::new().side_length(4.0));
    stage.add_to_scene(a);
    stage.add_to_scene(b);

    let a_storage = stage.get(a).unwrap().buffer.storage_id();
    let snapshot = stage.snapshot();
    // CoW: taking the snapshot did not copy record data (no live views).
    assert_eq!(stage.get(a).unwrap().buffer.storage_id(), a_storage);

    // Mutate the world: move a, delete b, add c, drop b from the scene.
    stage
        .get_mut(a)
        .unwrap()
        .buffer
        .write(0, "point", &[99.0, 0.0, 0.0]);
    // The write unshared from the snapshot (fresh generation for a).
    assert_ne!(stage.get(a).unwrap().buffer.storage_id(), a_storage);
    stage.delete(b);
    let c = stage.add(Square::new());
    stage.add_to_scene(c);
    assert!(!stage.contains(b));

    // Restore: the pre-mutation world, exactly.
    stage.restore(&snapshot);
    assert_eq!(
        stage.get(a).unwrap().buffer.read(0, "point").unwrap(),
        vec![1.0, 1.0, 0.0],
        "a's original corner is back"
    );
    assert!(stage.contains(b), "deleted entry restored");
    assert_eq!(stage.roots(), &[a, b]);
    // The handle allocated after the snapshot is stale after restore — the
    // generation bump makes this detectable, not silent.
    assert!(!stage.contains(c));
}

// The fluent front door (§15.1): scoped stage + builders + deferred-command
// .animate recording, compiling end-to-end.
#[test]
fn fluent_api_prototype() {
    let mut stage = Stage::new();
    let square = stage.add(Square::new().side_length(2.0).color(BLUE));
    stage.add_to_scene(square);

    stage
        .play(square.animate().rotate(PI / 4.0).set_opacity(0.5))
        .unwrap();
    let rgba = stage.get(square).unwrap().buffer.read(0, "rgba").unwrap();
    assert_eq!(rgba[3], 0.5);
    let p = stage.get(square).unwrap().buffer.read(0, "point").unwrap();
    // (1, 1) rotated 45° lands on the y-axis.
    assert!(p[0].abs() < 1e-6 && (p[1] - 2f32.sqrt()).abs() < 1e-6);

    // Tuple play + error on stale handles.
    let other = stage.add(Square::new());
    stage
        .play((
            other.animate().shift([1.0, 0.0, 0.0]),
            square.animate().rotate(0.1),
        ))
        .unwrap();
    stage.delete(other);
    assert!(stage.play(other.animate().shift([1.0, 0.0, 0.0])).is_err());
}
