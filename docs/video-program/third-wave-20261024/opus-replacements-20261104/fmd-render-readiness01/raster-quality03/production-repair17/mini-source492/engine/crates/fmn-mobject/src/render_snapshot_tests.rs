use super::*;
use crate::{ImageColorSpace, ImageResource, ImageSampler, Mobject, RecordBuffer, RecordSchema};
use crate::{RenderPrimitive, ShapeTag, Stage, UpdaterFn, UpdaterId, UpdaterSlot};
use std::cell::RefCell;
use std::rc::Rc;

fn line(stage: &mut Stage, x: f64) -> Mob {
    stage.add(Mobject::vector_from_points(&[
        [x, 0.0, 0.0],
        [x + 0.5, 0.0, 0.0],
        [x + 1.0, 0.0, 0.0],
    ]))
}

fn decode(bytes: &[u8]) -> Stage {
    let mut stage = Stage::new();
    let decoded = Snapshot::from_bytes(bytes, &stage).unwrap();
    assert!(decoded.updaters.entries.is_empty());
    stage.restore(&decoded.snapshot);
    stage
}

fn picture(history: usize, reverse_allocation: bool) -> Stage {
    let mut stage = Stage::new();
    for _ in 0..history {
        line(&mut stage, 42.0);
    }
    let (left, right) = if reverse_allocation {
        let right = line(&mut stage, 2.0);
        let left = line(&mut stage, -2.0);
        (left, right)
    } else {
        (line(&mut stage, -2.0), line(&mut stage, 2.0))
    };
    let group = stage.add(Mobject::new());
    stage.attach(group, left).unwrap();
    stage.attach(group, right).unwrap();
    stage.add_to_scene(group).unwrap();
    stage
}

#[test]
fn render_bytes_ignore_unrooted_history_and_allocation_order() {
    let clean = picture(0, false).snapshot();
    let noisy = picture(512, true).snapshot();
    // Negative control: the ordinary undo format really retains that history.
    assert_ne!(clean.to_bytes().unwrap(), noisy.to_bytes().unwrap());
    let bytes = clean.to_render_bytes().unwrap();
    assert_eq!(bytes, noisy.to_render_bytes().unwrap());
    assert!(noisy.to_bytes().unwrap().len() > 20 * bytes.len());
    assert_eq!(decode(&bytes).snapshot().slots.len(), 3);
}

#[test]
fn shared_children_keep_both_root_placements_and_painter_partitions() {
    let mut stage = Stage::new();
    let shared = line(&mut stage, 1.0);
    let own = line(&mut stage, -1.0);
    let left = stage.add(Mobject::new());
    let right = stage.add(Mobject::new());
    stage.attach(left, shared).unwrap();
    stage.attach(left, own).unwrap();
    stage.attach(right, shared).unwrap();
    stage.get_mut(right).unwrap().uniforms_mut().depth_test = true;
    stage.get_mut(own).unwrap().uniforms_mut().stroke_behind = true;
    stage.add_many_to_scene(&[right, left]).unwrap();
    let before = stage.draw_plan();
    let restored = decode(&stage.snapshot().to_render_bytes().unwrap());
    let after = restored.draw_plan();
    assert_eq!(before.items().len(), 3);
    assert_eq!(before.items().len(), after.items().len());
    assert_eq!(before.group_count(), after.group_count());
    assert_eq!(before.batch_trace(), after.batch_trace());
    for (a, b) in before.items().iter().zip(after.items()) {
        assert_eq!(
            (a.group, a.batch, a.key, a.passes),
            (b.group, b.batch, b.key, b.passes)
        );
        assert_eq!(stage.get_points(a.mob), restored.get_points(b.mob));
    }
    let roots = restored.roots();
    let a = restored.get(roots[0]).unwrap().submobjects()[0];
    let b = restored.get(roots[1]).unwrap().submobjects()[0];
    assert_eq!(a, b, "the shared drawable must not be duplicated as data");
    assert_eq!(restored.get(a).unwrap().parents().len(), 2);
}

#[test]
fn ordinary_snapshot_and_live_handles_are_untouched() {
    let mut stage = picture(8, false);
    let root = stage.roots()[0];
    let before = stage.snapshot().to_bytes().unwrap();
    let snapshot = stage.snapshot();
    let bytes = snapshot.to_render_bytes().unwrap();
    assert_eq!(before, snapshot.to_bytes().unwrap());
    assert_eq!(before, stage.snapshot().to_bytes().unwrap());
    stage.shift(root, [9.0, 0.0, 0.0]);
    assert!(stage.contains(root));
    assert_eq!(bytes, snapshot.to_render_bytes().unwrap());
    let restored = decode(&bytes);
    assert_eq!(restored.get_center(restored.roots()[0]), [0.5, 0.0, 0.0]);
    assert_eq!(stage.get_center(root), [9.5, 0.0, 0.0]);
}

#[test]
fn execution_metadata_is_omitted_without_running_or_removing_callbacks() {
    let stage = picture(2, false);
    let clean = stage.snapshot();
    let mut active = stage.snapshot();
    let root = active.roots[0];
    let target = Mob::from_parts(active.stage_id, 0, 0);
    let slot = active.slots[root.parts().0 as usize].1.as_mut().unwrap();
    slot.updaters.push(UpdaterSlot {
        id: UpdaterId::from_raw(77),
        func: UpdaterFn::NonDt(Rc::new(RefCell::new(|_: &mut Stage, _: Mob| {
            panic!("render projection must never execute a callback");
        }))),
    });
    slot.target = Some(target);
    slot.saved_state = Some(target);
    slot.pins = 17;
    slot.pending_delete = true;
    slot.updating_suspended = true;
    slot.is_animating = true;
    slot.buffer.lock_data(std::iter::once("point"));
    active.next_updater_id = 78;
    let full = active.to_bytes().unwrap();
    assert_ne!(full, clean.to_bytes().unwrap());
    let bytes = active.to_render_bytes().unwrap();
    assert_eq!(bytes, clean.to_render_bytes().unwrap());
    assert_eq!(full, active.to_bytes().unwrap());
    let projected = decode(&bytes).snapshot();
    assert_eq!(projected.next_updater_id, 1);
    for (_, slot) in &projected.slots {
        let slot = slot.as_ref().unwrap();
        assert!(slot.updaters.is_empty());
        assert!(slot.target.is_none() && slot.saved_state.is_none());
        assert_eq!(slot.pins, 0);
        assert!(!slot.pending_delete && !slot.is_animating && !slot.updating_suspended);
        assert!(!slot.buffer.is_locked("point"));
    }
}

#[test]
fn custom_columns_placement_uniforms_and_shape_validity_survive() {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::new());
    let schema = RecordSchema::new(
        &[("point", 3), ("aux", 3), ("rgba", 4), ("custom", 2)],
        &["point", "aux", "rgba", "custom"],
        &["point", "aux"],
    )
    .unwrap();
    let e = stage.get_mut(mob).unwrap();
    e.buffer = RecordBuffer::new(schema, 3).unwrap();
    e.buffer
        .write_range("point", 0, &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 2.0, 0.0, 0.0]);
    e.buffer
        .write_range("aux", 0, &[0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 2.0, 1.0, 0.0]);
    e.buffer
        .write_range("custom", 0, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    e.buffer
        .write_range("rgba", 0, &[0.25, 0.5, 0.75, 0.5].repeat(3));
    e.uniforms_mut().is_fixed_in_frame = 0.5;
    e.uniforms_mut().clip_planes[0] = [0.0, 1.0, 0.0, 0.5];
    stage.add_to_scene(mob).unwrap();
    stage.shift(mob, [2.0, 3.0, 4.0]);
    let mut source = stage.snapshot();
    let e = source.slots[0].1.as_mut().unwrap();
    e.shape.tag = ShapeTag::Circle {
        center: [0.0, 0.0, 0.0],
        radius: 1.0,
    };
    e.shape.point_revision = None; // A stale constructor hint must stay stale.
    let projected = project(&source, RenderSnapshotLimits::DEFAULT).unwrap();
    let a = source.slots[0].1.as_ref().unwrap();
    let b = projected.slots[0].1.as_ref().unwrap();
    assert_eq!(a.buffer.schema(), b.buffer.schema());
    for field in ["point", "aux", "rgba", "custom"] {
        assert_eq!(a.buffer.read_column(field), b.buffer.read_column(field));
    }
    assert_eq!(a.placement.coefficients(), b.placement.coefficients());
    assert_eq!(a.uniforms.is_fixed_in_frame, b.uniforms.is_fixed_in_frame);
    assert_eq!(a.uniforms.clip_planes, b.uniforms.clip_planes);
    assert_eq!(a.shape.tag, b.shape.tag);
    assert!(b.shape.point_revision.is_none());
    let decoded = decode(&source.to_render_bytes().unwrap());
    assert_eq!(
        decoded.get_points(decoded.roots()[0]),
        stage.get_points(mob)
    );
}

#[test]
fn primary_and_dark_image_resources_and_primitive_are_preserved() {
    let mut stage = Stage::new();
    let mob = line(&mut stage, 0.0);
    stage.add_to_scene(mob).unwrap();
    let make_image = |pixel| {
        ImageResource::rgba8(1, 1, pixel, ImageColorSpace::Srgb, ImageSampler::default()).unwrap()
    };
    let image = make_image(vec![255, 0, 0, 127])
        .with_dark_image(make_image(vec![0, 0, 255, 255]))
        .unwrap();
    let mut source = stage.snapshot();
    let entry = source.slots[0].1.as_mut().unwrap();
    entry.render_primitive = RenderPrimitive::TriangleMesh;
    entry.image = Some(image.clone());
    let projected = project(&source, RenderSnapshotLimits::DEFAULT).unwrap();
    let entry = projected.slots[0].1.as_ref().unwrap();
    assert_eq!(entry.render_primitive, RenderPrimitive::TriangleMesh);
    assert_eq!(entry.image.as_ref(), Some(&image));
    let decoded = decode(&source.to_render_bytes().unwrap()).snapshot();
    assert_eq!(
        decoded.slots[0].1.as_ref().unwrap().image.as_ref(),
        Some(&image)
    );
}

#[test]
fn empty_picture_drops_all_arena_history() {
    let mut stage = picture(32, true);
    let root = stage.roots()[0];
    stage.remove_from_scene(root);
    let bytes = stage
        .snapshot()
        .to_render_bytes_with_limits(RenderSnapshotLimits { max_graph_bytes: 0 })
        .unwrap();
    assert_eq!(bytes, Stage::new().snapshot().to_bytes().unwrap());
    assert!(decode(&bytes).snapshot().slots.is_empty());
    assert!(
        stage.contains(root),
        "projection must not free removed objects"
    );
}

#[test]
fn graph_budget_and_overflow_refuse_before_reservation() {
    let snapshot = picture(0, false).snapshot();
    assert!(matches!(
        snapshot.to_render_bytes_with_limits(RenderSnapshotLimits { max_graph_bytes: 0 }),
        Err(RenderSnapshotError::GraphLimit { limit: 0, .. })
    ));
    let mut budget = Budget {
        limit: usize::MAX,
        charged: 0,
    };
    let mut values = Vec::<u64>::new();
    assert!(matches!(
        budget.reserve(&mut values, usize::MAX, "overflow probe"),
        Err(RenderSnapshotError::GraphLimit {
            needed: usize::MAX,
            ..
        })
    ));
    assert_eq!(values.capacity(), 0);
    assert_eq!(budget.charged, 0);
}

#[test]
fn stale_foreign_and_cyclic_rooted_edges_are_refused() {
    let stage = picture(0, false);
    let mut source = stage.snapshot();
    let root = source.roots[0];
    let (_, generation) = root.parts();
    source.roots[0] = Mob::from_parts(source.stage_id, root.parts().0, generation + 1);
    assert!(matches!(
        source.to_render_bytes(),
        Err(RenderSnapshotError::Graph(StageError::StaleHandle))
    ));
    let mut other = Stage::new();
    source.roots[0] = line(&mut other, 0.0);
    assert!(matches!(
        source.to_render_bytes(),
        Err(RenderSnapshotError::Graph(StageError::StaleHandle))
    ));
    source.roots[0] = root;
    source.slots[root.parts().0 as usize]
        .1
        .as_mut()
        .unwrap()
        .submobjects
        .push(root);
    assert!(matches!(
        source.to_render_bytes(),
        Err(RenderSnapshotError::Graph(StageError::CycleDetected))
    ));
}

#[test]
fn duplicate_roots_and_edges_are_not_silently_deduplicated() {
    let stage = picture(0, false);
    let mut source = stage.snapshot();
    let root = source.roots[0];
    source.roots.push(root);
    assert!(matches!(
        source.to_render_bytes(),
        Err(RenderSnapshotError::InvalidGraph("duplicate scene root"))
    ));
    source.roots.pop();
    let entry = source.slots[root.parts().0 as usize].1.as_mut().unwrap();
    entry.submobjects.push(entry.submobjects[0]);
    assert!(matches!(
        source.to_render_bytes(),
        Err(RenderSnapshotError::InvalidGraph("duplicate child edge"))
    ));
}

#[test]
fn deeply_shared_dag_uses_objects_and_edges_not_expanded_paths() {
    let mut stage = Stage::new();
    let leaf = line(&mut stage, 0.0);
    let mut previous = vec![leaf];
    for _ in 0..32 {
        let level = vec![stage.add(Mobject::new()), stage.add(Mobject::new())];
        for &parent in &level {
            for &child in &previous {
                stage.attach(parent, child).unwrap();
            }
        }
        previous = level;
    }
    stage.add_many_to_scene(&previous).unwrap();
    let source = stage.snapshot();
    let projected = project(&source, RenderSnapshotLimits::DEFAULT).unwrap();
    assert_eq!(projected.slots.len(), 65);
    assert_eq!(projected.roots.len(), 2);
    assert_eq!(
        projected
            .slots
            .iter()
            .map(|(_, e)| e.as_ref().unwrap().submobjects.len())
            .sum::<usize>(),
        126,
    );
    let bytes = source.to_render_bytes().unwrap();
    assert_eq!(bytes, source.to_render_bytes().unwrap());
    assert_eq!(decode(&bytes).snapshot().slots.len(), 65);
}

#[test]
fn handle_generations_and_unrooted_parents_do_not_leak_into_picture_identity() {
    let mut stage = Stage::new();
    let child = line(&mut stage, 0.0);
    stage.add_to_scene(child).unwrap();
    let clean = stage.snapshot().to_render_bytes().unwrap();
    let outside = stage.add(Mobject::new());
    stage.attach(outside, child).unwrap();
    let mut snapshot = stage.snapshot();
    for (generation, slot) in &mut snapshot.slots {
        *generation = 7;
        if let Some(slot) = slot {
            for handle in slot.submobjects.iter_mut().chain(&mut slot.parents) {
                *handle = Mob::from_parts(snapshot.stage_id, handle.parts().0, 7);
            }
        }
    }
    for root in &mut snapshot.roots {
        *root = Mob::from_parts(snapshot.stage_id, root.parts().0, 7);
    }
    assert_eq!(clean, snapshot.to_render_bytes().unwrap());
    let restored = decode(&clean);
    assert!(
        restored
            .get(restored.roots()[0])
            .unwrap()
            .parents()
            .is_empty()
    );
    assert!(stage.contains(child) && stage.contains(outside));
}

#[test]
fn a_live_shape_hint_is_preserved_and_an_invalidated_one_is_not_revived() {
    let stage = picture(0, false);
    let mut snapshot = stage.snapshot();
    let original = snapshot.slots[0].1.as_mut().unwrap();
    original.shape.tag = ShapeTag::Line {
        start: [-2.0, 0.0, 0.0],
        end: [-1.0, 0.0, 0.0],
        path_arc: 0.0,
        buff: 0.0,
    };
    original.shape.point_revision = original.buffer.field_revision("point");
    let projected = project(&snapshot, RenderSnapshotLimits::DEFAULT).unwrap();
    // Canonical order is group, left, right, unlike this arena's slot order.
    let projected_left = projected.slots[1].1.as_ref().unwrap();
    assert!(projected_left.shape.point_revision.is_some());
    assert_eq!(
        projected_left.shape.point_revision,
        projected_left.buffer.field_revision("point")
    );
    snapshot.slots[0]
        .1
        .as_mut()
        .unwrap()
        .buffer
        .write_range("point", 0, &[9.0, 8.0, 7.0]);
    let projected = project(&snapshot, RenderSnapshotLimits::DEFAULT).unwrap();
    assert!(
        projected.slots[1]
            .1
            .as_ref()
            .unwrap()
            .shape
            .point_revision
            .is_none()
    );
}
