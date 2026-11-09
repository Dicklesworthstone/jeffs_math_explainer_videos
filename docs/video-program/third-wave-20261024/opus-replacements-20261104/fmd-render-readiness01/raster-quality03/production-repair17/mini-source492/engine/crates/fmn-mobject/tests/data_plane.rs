//! fm-cus acceptance: the data plane's three layers.
//!
//! - Layout locks: schema byte offsets equal NumPy's structured-dtype
//!   packing for the Reference dtypes (the zero-copy export contract).
//!   The live fnp-backed round-trip lands when SUITE.lock makes the suite
//!   consumable (follow-up bead, blocked on fm-g2c).
//! - View protocol: resize under a live view, field-scoped views and
//!   dirty, ranged edits (precise-dirty opt-in).
//! - Mirrors: coherence (mirror ≡ buffer at every observation point),
//!   laziness (untouched fields never rematerialize), conservative refresh
//!   under writable views.
//! - Custom dtypes end-to-end; locking as copy-elision state;
//!   resize-with-interpolation semantics ported from the Reference.

use fmn_mobject::{MirrorSet, RecordBuffer, RecordSchema};

// ------------------------------------------------------------------ layout

#[test]
fn schema_offsets_match_numpy_structured_packing() {
    // numpy: itemsize 28, point@0, rgba@12 (byte offsets).
    let mobject = RecordSchema::mobject();
    assert_eq!(mobject.stride() * 4, 28);
    assert_eq!(mobject.offset("point").unwrap() * 4, 0);
    assert_eq!(mobject.offset("rgba").unwrap() * 4, 12);
    assert_eq!(mobject.aligned_keys(), ["point"]);
    assert_eq!(mobject.pointlike_keys(), ["point"]);

    // numpy: itemsize 68; offsets point@0, stroke_rgba@12, stroke_width@28,
    // joint_angle@32, fill_rgba@36, base_normal@52, fill_border_width@64.
    let vmobject = RecordSchema::vmobject();
    assert_eq!(vmobject.stride() * 4, 68);
    for (field, byte_offset) in [
        ("point", 0),
        ("stroke_rgba", 12),
        ("stroke_width", 28),
        ("joint_angle", 32),
        ("fill_rgba", 36),
        ("base_normal", 52),
        ("fill_border_width", 64),
    ] {
        assert_eq!(
            vmobject.offset(field).unwrap() * 4,
            byte_offset,
            "byte offset of {field}"
        );
    }
}

// ------------------------------------------------------------ view protocol

#[test]
fn field_scoped_views_and_dirty() {
    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), 4).unwrap();
    let point_view = buffer.export_field_view("point", true).unwrap();

    // Scoped views only touch their field.
    assert!(point_view.write(0, "point", &[1.0, 2.0, 3.0]));
    assert!(!point_view.write(0, "fill_rgba", &[1.0; 4]));
    assert!(point_view.read(0, "fill_rgba").is_none());
    assert_eq!(buffer.read(0, "point").unwrap(), vec![1.0, 2.0, 3.0]);

    // Writable-view bookkeeping is per-field.
    assert!(buffer.field_has_writable_view("point"));
    assert!(!buffer.field_has_writable_view("fill_rgba"));
    assert!(!buffer.has_writable_whole_view());
    assert!(buffer.writable_view_affects("point"));
    assert!(!buffer.writable_view_affects("fill_rgba"));
    drop(point_view);
    assert!(!buffer.field_has_writable_view("point"));

    // Field revisions move independently.
    let point_rev = buffer.field_revision("point").unwrap();
    let fill_rev = buffer.field_revision("fill_rgba").unwrap();
    buffer.write(1, "fill_rgba", &[0.5; 4]);
    assert_eq!(buffer.field_revision("point").unwrap(), point_rev);
    assert!(buffer.field_revision("fill_rgba").unwrap() > fill_rev);
}

#[test]
fn writable_view_detach_conservatively_revises_exactly_its_scope() {
    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), 2).unwrap();
    let point_before = buffer.field_revision("point").unwrap();
    let fill_before = buffer.field_revision("fill_rgba").unwrap();

    // Do not call `RecordView::write`: this models a foreign zero-copy writer,
    // which can mutate the exported cells without an engine callback. Merely
    // exposing writable storage must close the final invalidation window when
    // the view detaches.
    let point_view = buffer.export_field_view("point", true).unwrap();
    assert!(point_view.write_foreign(0, "point", &[3.0, 4.0, 5.0]));
    assert_eq!(
        buffer.field_revision("point"),
        Some(point_before),
        "a foreign write cannot advance the engine's revision"
    );
    assert_eq!(buffer.read(0, "point"), Some(vec![3.0, 4.0, 5.0]));
    assert_eq!(buffer.take_dirty_span("point"), Some((0, 1)));
    assert_eq!(
        buffer.take_dirty_span("point"),
        Some((0, 1)),
        "foreign writes remain possible after a span is taken"
    );
    assert_eq!(buffer.take_dirty_span("fill_rgba"), None);
    drop(point_view);
    assert!(buffer.field_revision("point").unwrap() > point_before);
    assert_eq!(buffer.field_revision("fill_rgba").unwrap(), fill_before);
    assert_eq!(buffer.take_dirty_span("point"), Some((0, 1)));
    assert_eq!(buffer.take_dirty_span("fill_rgba"), None);

    let revisions: Vec<u64> = buffer
        .schema()
        .fields()
        .iter()
        .map(|field| buffer.field_revision(&field.name).unwrap())
        .collect();
    let whole_view = buffer.export_view(true);
    drop(whole_view);
    for (field, before) in buffer.schema().fields().iter().zip(revisions) {
        assert!(
            buffer.field_revision(&field.name).unwrap() > before,
            "whole-buffer detach missed {}",
            field.name
        );
    }
}

#[test]
fn frame_snapshots_of_a_viewed_buffer_stay_view_dirty_for_its_scope() {
    // A frame freeze copies a viewed buffer eagerly (V5), revisions included.
    // A foreign write between two frames changes the cells but not the
    // revision, so a revision-keyed render cache would reuse the first frame's
    // paint unless the copies keep reporting the view's scope as dirty.
    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), 2).unwrap();
    let fill_view = buffer.export_field_view("fill_rgba", true).unwrap();
    let first = buffer.snapshot_clone();
    assert!(fill_view.write_foreign(0, "fill_rgba", &[0.25; 4]));
    let second = buffer.snapshot_clone();
    assert_eq!(
        first.field_revision("fill_rgba"),
        second.field_revision("fill_rgba")
    );
    assert_ne!(first.read(0, "fill_rgba"), second.read(0, "fill_rgba"));
    for frame in [&first, &second, &second.snapshot_clone()] {
        assert!(frame.writable_view_affects("fill_rgba"));
        assert!(!frame.writable_view_affects("point"));
        assert_eq!(frame.live_view_count(), 0);
        assert!(!frame.has_writable_whole_view());
    }
    drop(fill_view);
    assert!(!buffer.snapshot_clone().writable_view_affects("fill_rgba"));

    let whole_view = buffer.export_view(true);
    let frame = buffer.snapshot_clone();
    assert!(
        buffer
            .schema()
            .fields()
            .iter()
            .all(|field| frame.writable_view_affects(&field.name))
    );
    drop(whole_view);
    assert!(!buffer.snapshot_clone().writable_view_affects("point"));
}

#[test]
fn ranged_edits_accumulate_precise_dirty_spans() {
    let mut buffer = RecordBuffer::new(RecordSchema::mobject(), 10).unwrap();
    assert_eq!(buffer.take_dirty_span("point"), None);

    // A bulk write covering records 2..=4.
    assert!(buffer.write_range("point", 2, &[1.0; 9]));
    // A single write at record 7 widens the span.
    assert!(buffer.write(7, "point", &[2.0, 2.0, 2.0]));
    assert_eq!(buffer.take_dirty_span("point"), Some((2, 7)));
    // Taking clears.
    assert_eq!(buffer.take_dirty_span("point"), None);
    // Other fields untouched.
    assert_eq!(buffer.take_dirty_span("rgba"), None);

    // Bounds and width are checked.
    assert!(!buffer.write_range("point", 9, &[0.0; 6]));
    assert!(!buffer.write_range("point", 0, &[0.0; 4]));
    // The rejected writes left no dirty span behind.
    assert_eq!(buffer.take_dirty_span("point"), None);
}

#[test]
fn resize_under_live_view_detaches_naturally() {
    let mut buffer = RecordBuffer::new(RecordSchema::mobject(), 2).unwrap();
    buffer.write(0, "point", &[9.0, 9.0, 9.0]);
    let view = buffer.export_view(false);

    buffer.resize(5).unwrap();
    assert!(!view.is_attached_to(&buffer));
    // The view still reads the old generation; growth is null-padded.
    assert_eq!(view.read(0, "point").unwrap(), vec![9.0, 9.0, 9.0]);
    assert_eq!(buffer.read(4, "point").unwrap(), vec![0.0, 0.0, 0.0]);
    assert_eq!(buffer.read(0, "point").unwrap(), vec![9.0, 9.0, 9.0]);
}

// ---------------------------------------------------------------- mirrors

#[test]
fn mirror_coherence_at_every_observation_point() {
    let mut buffer = RecordBuffer::new(RecordSchema::mobject(), 3).unwrap();
    let mut mirrors = MirrorSet::new();

    let observe = |mirrors: &mut MirrorSet, buffer: &RecordBuffer| {
        for field in ["point", "rgba"] {
            let width = buffer.schema().field_width(field).unwrap();
            let len = buffer.len();
            let lanes = mirrors.sync(buffer, field).unwrap().to_vec();
            let column = buffer.read_column(field).unwrap();
            for record in 0..len {
                for lane in 0..width {
                    assert_eq!(
                        lanes[lane * len + record],
                        column[record * width + lane],
                        "{field} record {record} lane {lane}"
                    );
                }
            }
        }
    };

    observe(&mut mirrors, &buffer);
    buffer.write(1, "point", &[1.0, 2.0, 3.0]);
    observe(&mut mirrors, &buffer);
    buffer.write_range("rgba", 0, &[0.25; 12]);
    observe(&mut mirrors, &buffer);
    buffer.resize_with_interpolation(7).unwrap();
    observe(&mut mirrors, &buffer);
    // Writes through a view are observed too (conservative refresh).
    let view = buffer.export_view(true);
    view.write(2, "point", &[5.0, 5.0, 5.0]);
    observe(&mut mirrors, &buffer);
}

#[test]
fn mirror_laziness_untouched_fields_never_rematerialize() {
    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), 8).unwrap();
    let mut mirrors = MirrorSet::new();

    mirrors.sync(&buffer, "point").unwrap();
    mirrors.sync(&buffer, "fill_rgba").unwrap();
    assert_eq!(mirrors.materializations(), 2);

    // Repeated observation with no writes: nothing rematerializes.
    for _ in 0..5 {
        mirrors.sync(&buffer, "point").unwrap();
        mirrors.sync(&buffer, "fill_rgba").unwrap();
    }
    assert_eq!(mirrors.materializations(), 2);

    // Touch one field: only it rematerializes.
    buffer.write(3, "point", &[1.0, 1.0, 1.0]);
    mirrors.sync(&buffer, "point").unwrap();
    mirrors.sync(&buffer, "fill_rgba").unwrap();
    assert_eq!(mirrors.materializations(), 3);

    // A writable whole-buffer view forces conservative refresh every
    // observation — a live view never gets weaker semantics.
    let view = buffer.export_view(true);
    mirrors.sync(&buffer, "fill_rgba").unwrap();
    mirrors.sync(&buffer, "fill_rgba").unwrap();
    assert_eq!(mirrors.materializations(), 5);
    drop(view);
    mirrors.sync(&buffer, "fill_rgba").unwrap();
    let settled = mirrors.materializations();
    mirrors.sync(&buffer, "fill_rgba").unwrap();
    assert_eq!(mirrors.materializations(), settled);
}

// ------------------------------------------------------------ custom dtype

#[test]
fn custom_dtype_end_to_end() {
    // A user-declared record type through the same schema machinery.
    let schema = RecordSchema::new(
        &[("position", 3), ("velocity", 3), ("charge", 1)],
        &["position"],
        &["position", "velocity"],
    )
    .unwrap();
    assert_eq!(schema.stride(), 7);
    let mut buffer = RecordBuffer::new(schema, 2).unwrap();
    assert!(buffer.write(0, "charge", &[-1.0]));
    assert!(buffer.write(1, "velocity", &[0.0, 9.8, 0.0]));

    let view = buffer.export_field_view("velocity", true).unwrap();
    assert!(view.write(0, "velocity", &[1.0, 0.0, 0.0]));
    assert_eq!(buffer.read(0, "velocity").unwrap(), vec![1.0, 0.0, 0.0]);

    let mut mirrors = MirrorSet::new();
    let len = buffer.len();
    let lanes = mirrors.sync(&buffer, "charge").unwrap();
    assert_eq!(lanes.len(), len);
    assert_eq!(lanes[0], -1.0);

    // Unknown fields are precise failures, never silence.
    assert!(!buffer.write(0, "spin", &[1.0]));
    assert!(buffer.export_field_view("spin", true).is_none());
    assert_eq!(buffer.field_revision("spin"), None);
}

// -------------------------------------------------------------- lock state

#[test]
fn data_locking_is_copy_elision_state() {
    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), 4).unwrap();
    let same = buffer.deep_clone();
    assert!(buffer.column_eq(&same, "point"));
    assert!(!buffer.column_eq(&same, "nonexistent"));
    buffer.write(0, "point", &[1.0, 1.0, 1.0]);
    assert!(!buffer.column_eq(&same, "point"));

    buffer.lock_data(["point", "base_normal", "nonexistent"]);
    assert!(buffer.is_locked("point"));
    assert!(buffer.is_locked("base_normal"));
    assert!(!buffer.is_locked("fill_rgba"));
    assert_eq!(buffer.locked_keys(), vec!["point", "base_normal"]);

    // Locking never gates access (it is an animation-engine contract).
    assert!(buffer.write(0, "point", &[1.0, 1.0, 1.0]));

    // Lock state survives snapshot/deep clones (it is animation state).
    assert!(buffer.snapshot_clone().is_locked("point"));
    assert!(buffer.deep_clone().is_locked("base_normal"));

    buffer.unlock_data();
    assert!(buffer.locked_keys().is_empty());
}

// -------------------------------------------- resize-with-interpolation

#[test]
fn resize_with_interpolation_matches_reference_semantics() {
    // Linear ramp over 3 records → 5 records keeps the ramp.
    let mut buffer = RecordBuffer::new(RecordSchema::mobject(), 3).unwrap();
    for (i, x) in [0.0f32, 1.0, 2.0].iter().enumerate() {
        buffer.write(i, "point", &[*x, 0.0, 0.0]);
    }
    buffer.resize_with_interpolation(5).unwrap();
    assert_eq!(buffer.len(), 5);
    let xs: Vec<f32> = (0..5)
        .map(|i| buffer.read(i, "point").unwrap()[0])
        .collect();
    assert_eq!(xs, vec![0.0, 0.5, 1.0, 1.5, 2.0]);

    // A single record repeats.
    let mut single = RecordBuffer::new(RecordSchema::mobject(), 1).unwrap();
    single.write(0, "point", &[7.0, 0.0, 0.0]);
    single.resize_with_interpolation(4).unwrap();
    for i in 0..4 {
        assert_eq!(single.read(i, "point").unwrap()[0], 7.0);
    }

    // An all-equal buffer repeats rather than interpolating.
    let mut constant = RecordBuffer::new(RecordSchema::mobject(), 3).unwrap();
    for i in 0..3 {
        constant.write(i, "point", &[4.0, 4.0, 4.0]);
        constant.write(i, "rgba", &[1.0, 0.0, 0.0, 1.0]);
    }
    constant.resize_with_interpolation(6).unwrap();
    for i in 0..6 {
        assert_eq!(constant.read(i, "point").unwrap(), vec![4.0, 4.0, 4.0]);
    }

    // Zero target empties; same length is a no-op that keeps the storage.
    let mut empty_target = RecordBuffer::new(RecordSchema::mobject(), 3).unwrap();
    empty_target.resize_with_interpolation(0).unwrap();
    assert!(empty_target.is_empty());
    let mut same = RecordBuffer::new(RecordSchema::mobject(), 3).unwrap();
    let id_before = same.storage_id();
    same.resize_with_interpolation(3).unwrap();
    assert_eq!(same.storage_id(), id_before);
}

// -------------------------------------------------- fm-vek.2: fallible sizing

use fmn_mobject::RecordError;

#[test]
fn schema_stride_is_checked_at_exact_and_one_over_boundaries() {
    // Exact boundary: a single field of usize::MAX lanes sums without wrap.
    let boundary = RecordSchema::new(&[("a", usize::MAX)], &[], &[]).unwrap();
    assert_eq!(boundary.stride(), usize::MAX);

    // One over: adding a single lane to usize::MAX is a typed refusal.
    assert_eq!(
        RecordSchema::new(&[("a", usize::MAX), ("b", 1)], &[], &[]),
        Err(RecordError::StrideOverflow)
    );

    // Mixed widths: usize::MAX/2 + usize::MAX/2 + 1 == usize::MAX exactly…
    let mixed = RecordSchema::new(
        &[("a", usize::MAX / 2), ("b", usize::MAX / 2), ("c", 1)],
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(mixed.stride(), usize::MAX);
    // …and one lane past that overflows.
    assert_eq!(
        RecordSchema::new(
            &[
                ("a", usize::MAX / 2),
                ("b", usize::MAX / 2),
                ("c", 1),
                ("d", 1),
            ],
            &[],
            &[],
        ),
        Err(RecordError::StrideOverflow)
    );
}

#[test]
fn buffer_construction_proves_its_size_before_allocating() {
    // Zero records of an enormous stride allocate nothing: exact boundary.
    let wide = RecordSchema::new(&[("a", usize::MAX)], &[], &[]).unwrap();
    let empty = RecordBuffer::new(wide.clone(), 0).unwrap();
    assert!(empty.is_empty());

    // One record of usize::MAX lanes: the lane product fits usize exactly,
    // but the four-byte size cannot be addressed — a typed refusal.
    assert_eq!(
        RecordBuffer::new(wide, 1).unwrap_err(),
        RecordError::SizeOverflow {
            len: 1,
            stride: usize::MAX,
        }
    );

    // The multiplication-overflow seam, exact and one over:
    // usize::MAX == 3 * 0x5555_5555_5555_5555 exactly.
    let stride3 = RecordSchema::new(&[("a", 3)], &[], &[]).unwrap();
    const SEAM: usize = usize::MAX / 3;
    assert_eq!(
        RecordBuffer::new(stride3.clone(), SEAM).unwrap_err(),
        RecordError::SizeOverflow {
            len: SEAM,
            stride: 3,
        }
    );
    assert_eq!(
        RecordBuffer::new(stride3, SEAM + 1).unwrap_err(),
        RecordError::SizeOverflow {
            len: SEAM + 1,
            stride: 3,
        }
    );

    // The isize byte-capacity seam: lanes fit usize, but the byte count
    // exceeds what any Rust allocation may hold.
    let at_cap = RecordSchema::new(&[("a", (isize::MAX as usize) / 4 + 1)], &[], &[]).unwrap();
    assert_eq!(
        RecordBuffer::new(at_cap, 1).unwrap_err(),
        RecordError::SizeOverflow {
            len: 1,
            stride: (isize::MAX as usize) / 4 + 1,
        }
    );

    // len = usize::MAX against the mobject schema refuses, not wraps.
    assert_eq!(
        RecordBuffer::new(RecordSchema::mobject(), usize::MAX).unwrap_err(),
        RecordError::SizeOverflow {
            len: usize::MAX,
            stride: 7,
        }
    );
}

#[test]
fn failed_resizes_are_atomic_and_keep_live_views() {
    let expected = Err(RecordError::SizeOverflow {
        len: usize::MAX,
        stride: 7,
    });
    for resize in [
        RecordBuffer::resize as fn(&mut RecordBuffer, usize) -> Result<(), RecordError>,
        RecordBuffer::resize_with_interpolation,
        RecordBuffer::resize_preserving_order,
    ] {
        let mut buffer = RecordBuffer::new(RecordSchema::mobject(), 2).unwrap();
        buffer.write(0, "point", &[9.0, 9.0, 9.0]);
        buffer.write(1, "point", &[4.0, 4.0, 4.0]);
        let view = buffer.export_view(false);
        let storage_before = buffer.storage_id();
        let revision_before = buffer.revision();

        assert_eq!(resize(&mut buffer, usize::MAX), expected);

        // §8.2 live-view semantics: the refusal touched nothing — length,
        // generation, revisions, contents, and the view's attachment.
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.storage_id(), storage_before);
        assert_eq!(buffer.revision(), revision_before);
        assert!(view.is_attached_to(&buffer));
        assert_eq!(view.read(0, "point").unwrap(), vec![9.0, 9.0, 9.0]);
        assert_eq!(buffer.read(1, "point").unwrap(), vec![4.0, 4.0, 4.0]);
        drop(view);

        // A valid resize afterwards behaves exactly as before.
        resize(&mut buffer, 4).unwrap();
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.read(0, "point").unwrap(), vec![9.0, 9.0, 9.0]);
    }
}
