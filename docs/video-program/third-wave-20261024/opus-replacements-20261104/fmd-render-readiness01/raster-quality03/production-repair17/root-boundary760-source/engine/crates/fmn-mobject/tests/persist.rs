//! The persistence layer's acceptance surface (§8.7, fm-879): round-trip
//! across a representative scene (geometry + uniforms + family topology +
//! trackers + §8.3 links + RNG), byte determinism (twice, and across a
//! re-open), the updater honesty clause, versioning and corruption
//! refusals, and cross-stage decode (handles re-bound to a fresh mint).

use std::cell::{Cell, RefCell};
use std::panic::AssertUnwindSafe;
use std::rc::Rc;

use fmn_core::rng::Pcg64Dxsm;
use fmn_hash::{Limits, SerialError, Writer, sha256};
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::{
    ImageColorSpace, ImageResource, ImageSampler, JointType, Mob, Mobject, PersistError,
    RenderPrimitive, SNAPSHOT_SCHEMA, SceneState, Snapshot, SnapshotLimits, Stage, StageError,
    UpdaterFn, UpdaterKindTag, UpdaterManifest,
};

fn vmob(stage: &mut Stage, points: &[[f64; 3]], fill: [f32; 4]) -> Mob {
    let mob = stage.add(Mobject::new());
    let entry = stage.get_mut(mob).unwrap();
    entry.buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
    #[allow(clippy::cast_possible_truncation)]
    let flat: Vec<f32> = points
        .iter()
        .flat_map(|p| p.iter().map(|v| *v as f32))
        .collect();
    entry.buffer.write_range("point", 0, &flat);
    entry
        .buffer
        .write_range("fill_rgba", 0, &fill.repeat(points.len()));
    mob
}

/// A representative scene: a rooted family with styled children, tweaked
/// uniforms, a value tracker, §8.3 links, and a pin.
fn build_scene(stage: &mut Stage) -> (Mob, Mob, Mob, Mob) {
    let root = stage.add(Mobject::new());
    let c1 = vmob(
        stage,
        &[[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
        [0.2, 0.4, 0.6, 1.0],
    );
    let c2 = vmob(
        stage,
        &[[0.0, 3.0, 0.0], [1.0, 3.0, 0.0], [2.0, 3.0, 0.0]],
        [0.9, 0.1, 0.1, 0.5],
    );
    stage.attach(root, c1).unwrap();
    stage.attach(root, c2).unwrap();
    stage.add_to_scene(root).unwrap();
    {
        let u = stage.get_mut(c1).unwrap().uniforms_mut();
        u.anti_alias_width = 2.5;
        u.flat_stroke = true;
        u.joint_type = JointType::Miter;
        u.shading = [0.1, 0.2, 0.3];
    }
    let tracker = stage.add_value_tracker(42.5);
    stage.generate_target(c1).unwrap();
    stage.save_state(c2).unwrap();
    stage.pin(c1).unwrap();
    (root, c1, c2, tracker)
}

fn column(stage: &Stage, mob: Mob, field: &str) -> Vec<f32> {
    stage.get(mob).unwrap().buffer.read_column(field).unwrap()
}

#[test]
fn round_trip_representative_scene() {
    let mut stage = Stage::new();
    let (root, c1, c2, tracker) = build_scene(&mut stage);
    let points_before = column(&stage, c1, "point");
    let fill_before = column(&stage, c2, "fill_rgba");
    let bytes = stage.snapshot_bytes().unwrap();

    // Mutate everything the snapshot should undo.
    stage.shift(c1, [5.0, 5.0, 0.0]);
    stage.set_family_opacity_zero(c2);
    stage.detach(root, c2);
    stage.set_tracker_value(tracker, -1.0).unwrap();

    let decoded = Snapshot::from_bytes(&bytes, &stage).unwrap();
    stage.restore(&decoded.snapshot);

    assert_eq!(column(&stage, c1, "point"), points_before);
    assert_eq!(column(&stage, c2, "fill_rgba"), fill_before);
    assert_eq!(stage.family(root), vec![root, c1, c2]);
    assert_eq!(stage.roots(), &[root]);
    let u = *stage.get(c1).unwrap().uniforms();
    assert_eq!(u.anti_alias_width, 2.5);
    assert!(u.flat_stroke);
    assert_eq!(u.joint_type, JointType::Miter);
    assert_eq!(u.shading, [0.1, 0.2, 0.3]);
    assert_eq!(stage.tracker_value(tracker), Some(42.5));
    assert!(stage.target(c1).is_some());
    assert!(stage.saved_state(c2).is_some());
    assert_eq!(stage.get(c1).unwrap().pins(), 1);
}

#[test]
fn durable_round_trip_preserves_affine_placement_without_baking_points() {
    let mut stage = Stage::new();
    let mob = vmob(
        &mut stage,
        &[[0.0, 0.0, 0.0], [1.0, 0.5, 0.0], [2.0, 1.0, 0.0]],
        [0.2, 0.4, 0.6, 1.0],
    );
    let object_points = stage.get_object_points(mob).unwrap();
    stage.shift(mob, [4.0, -3.0, 0.25]);
    stage.rotate(
        mob,
        std::f64::consts::FRAC_PI_3,
        [0.0, 0.0, 1.0],
        Some([4.0, -3.0, 0.25]),
        None,
    );
    let placement = stage.placement(mob).unwrap();
    let world_points = stage.get_points(mob).unwrap();

    let bytes = stage.snapshot_bytes().unwrap();
    stage.set_points(mob, &[[99.0, 99.0, 99.0]]).unwrap();
    let decoded = Snapshot::from_bytes(&bytes, &stage).unwrap();
    stage.restore(&decoded.snapshot);

    assert!(stage.placement(mob).unwrap().same_bits(placement));
    assert_eq!(stage.get_object_points(mob).unwrap(), object_points);
    assert_eq!(stage.get_points(mob).unwrap(), world_points);
}

#[test]
fn render_primitive_survives_add_copy_become_snapshot_and_fmna_round_trip() {
    let mut stage = Stage::new();
    let vector = stage.add(Mobject::new());
    let surface = stage.add(
        Mobject::new().with_render_primitive(RenderPrimitive::SurfaceGrid {
            resolution: (17, 9),
        }),
    );
    let triangle = stage.add(Mobject::new().with_render_primitive(RenderPrimitive::TriangleMesh));
    let dots = stage.add(Mobject::new().with_render_primitive(RenderPrimitive::DotCloud));
    let image_resource = ImageResource::rgba8(
        2,
        1,
        vec![255, 0, 0, 255, 0, 255, 0, 128],
        ImageColorSpace::Srgb,
        ImageSampler::default(),
    )
    .unwrap();
    let image = stage.add(Mobject::new().with_image_resource(image_resource.clone()));

    let identities = [
        (
            surface,
            RenderPrimitive::SurfaceGrid {
                resolution: (17, 9),
            },
        ),
        (triangle, RenderPrimitive::TriangleMesh),
        (dots, RenderPrimitive::DotCloud),
        (image, RenderPrimitive::ImageQuad),
    ];
    for &(source, expected) in &identities {
        assert_eq!(stage.get(source).unwrap().render_primitive(), expected);
        let copied = stage.copy_family(source).unwrap();
        assert_eq!(stage.get(copied).unwrap().render_primitive(), expected);
    }

    stage.become_mobject(vector, surface, false).unwrap();
    assert_eq!(
        stage.get(vector).unwrap().render_primitive(),
        RenderPrimitive::SurfaceGrid {
            resolution: (17, 9)
        }
    );

    let memory_snapshot = stage.snapshot();
    let plain = stage.add(Mobject::new());
    stage.become_mobject(vector, plain, false).unwrap();
    stage.restore(&memory_snapshot);
    assert_eq!(
        stage.get(vector).unwrap().render_primitive(),
        RenderPrimitive::SurfaceGrid {
            resolution: (17, 9)
        }
    );
    for &(source, expected) in &identities {
        assert_eq!(stage.get(source).unwrap().render_primitive(), expected);
    }
    assert_eq!(
        stage.get(image).unwrap().image_resource(),
        Some(&image_resource)
    );

    let bytes = stage.snapshot_bytes().unwrap();
    let decoded = Snapshot::from_bytes(&bytes, &stage).unwrap();
    let plain = stage.add(Mobject::new());
    stage.become_mobject(vector, plain, false).unwrap();
    stage.restore(&decoded.snapshot);
    assert_eq!(
        stage.get(vector).unwrap().render_primitive(),
        RenderPrimitive::SurfaceGrid {
            resolution: (17, 9)
        }
    );
    for &(source, expected) in &identities {
        assert_eq!(stage.get(source).unwrap().render_primitive(), expected);
    }
    assert_eq!(
        stage.get(image).unwrap().image_resource(),
        Some(&image_resource)
    );
}

#[test]
fn image_replacement_advances_only_on_semantic_change() {
    let first = ImageResource::rgba8(
        1,
        1,
        vec![1, 2, 3, 255],
        ImageColorSpace::Srgb,
        ImageSampler::default(),
    )
    .unwrap();
    let second = ImageResource::rgba8(
        1,
        1,
        vec![9, 8, 7, 255],
        ImageColorSpace::Srgb,
        ImageSampler::default(),
    )
    .unwrap();
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::new().with_image_resource(first.clone()));
    let initial = stage.get(mob).unwrap().image_revision();

    assert!(!stage.set_image_resource(mob, Some(first)).unwrap());
    assert_eq!(stage.get(mob).unwrap().image_revision(), initial);
    assert!(stage.set_image_resource(mob, Some(second.clone())).unwrap());
    assert_eq!(
        stage.get(mob).unwrap().image_revision(),
        initial.wrapping_add(1)
    );
    assert_eq!(stage.get(mob).unwrap().image_resource(), Some(&second));
}

#[test]
fn image_content_digest_mismatch_is_a_typed_corruption_refusal() {
    let resource = ImageResource::rgba8(
        2,
        1,
        vec![3, 5, 7, 255, 11, 13, 17, 255],
        ImageColorSpace::Srgb,
        ImageSampler::default(),
    )
    .unwrap();
    let digest = resource.content_digest();
    let mut stage = Stage::new();
    stage.add(Mobject::new().with_image_resource(resource));
    let mut bytes = stage.snapshot_bytes().unwrap();
    let body_len = bytes.len() - 32;
    let positions: Vec<usize> = bytes[..body_len]
        .windows(digest.as_bytes().len())
        .enumerate()
        .filter_map(|(index, window)| (window == digest.as_bytes()).then_some(index))
        .collect();
    assert_eq!(positions.len(), 1, "descriptor carries one content digest");
    bytes[positions[0]] ^= 1;
    let outer = sha256(&bytes[..body_len]);
    bytes[body_len..].copy_from_slice(outer.as_bytes());

    assert!(matches!(
        Snapshot::from_bytes(&bytes, &stage),
        Err(PersistError::Malformed("image content digest mismatch"))
    ));
}

#[test]
fn unknown_durable_render_primitive_is_a_typed_corruption_refusal() {
    let mut stage = Stage::new();
    stage.add(Mobject::new());
    let mut bytes = stage.snapshot_bytes().unwrap();
    let body_len = bytes.len() - 32;
    // The v1.6 primary and v1.8 dark-image tables each trail the primitive
    // table with one liveness bit and one absent-resource bit for this entry.
    assert_eq!(&bytes[body_len - 5..body_len], &[0, 1, 0, 1, 0]);
    bytes[body_len - 5] = u8::MAX;
    let digest = sha256(&bytes[..body_len]);
    bytes[body_len..].copy_from_slice(digest.as_bytes());

    assert!(matches!(
        Snapshot::from_bytes(&bytes, &stage),
        Err(PersistError::Malformed("unknown render primitive"))
    ));
}

#[test]
fn byte_determinism_twice_and_across_reopen() {
    let mut stage = Stage::new();
    build_scene(&mut stage);
    let b1 = stage.snapshot_bytes().unwrap();
    let b2 = stage.snapshot_bytes().unwrap();
    assert_eq!(b1, b2, "same state ⇒ same bytes");

    let decoded = Snapshot::from_bytes(&b1, &stage).unwrap();
    let reencoded = decoded.snapshot.to_bytes().unwrap();
    assert_eq!(b1, reencoded, "re-open ⇒ identical bytes (no callables)");

    let snap = stage.snapshot();
    assert_eq!(
        snap.content_hash().unwrap(),
        sha256(&b1),
        "content hash is the canonical bytes' sha256"
    );
}

#[test]
fn oversized_schema_counts_are_typed_refusals_not_truncated_lengths() {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::new());
    let too_wide = usize::from(u16::MAX) + 1;
    stage.get_mut(mob).unwrap().buffer = RecordBuffer::new(
        RecordSchema::new(&[("too_wide", too_wide)], &[], &[]).unwrap(),
        0,
    )
    .unwrap();

    assert_eq!(
        stage.snapshot_bytes().expect_err("u16 width must not wrap"),
        SerialError::SizeLimit {
            limit: usize::from(u16::MAX),
            needed: too_wide,
        }
    );
}

fn snapshot_with_declared_record_payload(field_count: u16, field_width: u16, len: u32) -> Vec<u8> {
    let mut writer = Writer::new(SNAPSHOT_SCHEMA);
    writer
        .put_u32(1)
        .put_u32(0)
        .put_bool(true)
        .put_u16(field_count);
    for _ in 0..field_count {
        writer.put_str("field").put_u16(field_width);
    }
    writer.put_u16(0).put_u16(0).put_u32(len);
    writer.finish().unwrap()
}

fn snapshot_with_max_pin_count(stage: &mut Stage, mob: Mob) -> Vec<u8> {
    let unpinned = stage.snapshot_bytes().unwrap();
    stage.pin(mob).unwrap();
    let mut pinned = stage.snapshot_bytes().unwrap();
    let body_len = pinned.len() - 32;
    assert_eq!(unpinned.len(), pinned.len());
    let changed: Vec<usize> = (0..body_len)
        .filter(|&offset| unpinned[offset] != pinned[offset])
        .collect();
    assert_eq!(changed.len(), 1, "pinning changes exactly one payload byte");

    let pin_offset = changed[0];
    assert_eq!(&unpinned[pin_offset..pin_offset + 8], &0_u64.to_le_bytes());
    assert_eq!(&pinned[pin_offset..pin_offset + 8], &1_u64.to_le_bytes());
    let maximum = u64::try_from(usize::MAX).expect("usize pin count fits the durable u64 field");
    pinned[pin_offset..pin_offset + 8].copy_from_slice(&maximum.to_le_bytes());

    let digest = sha256(&pinned[..body_len]);
    pinned[body_len..].copy_from_slice(digest.as_bytes());
    pinned
}

#[test]
fn persisted_max_pin_count_refuses_another_pin_without_state_change() {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::new());
    let bytes = snapshot_with_max_pin_count(&mut stage, mob);
    let decoded = Snapshot::from_bytes(&bytes, &stage).unwrap();
    stage.restore(&decoded.snapshot);
    assert_eq!(stage.get(mob).unwrap().pins(), usize::MAX);
    let before = stage.snapshot_bytes().unwrap();

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| stage.pin(mob)));
    assert_eq!(
        result.expect("pin-count exhaustion must never panic"),
        Err(StageError::PinCountExhausted)
    );
    assert_eq!(stage.get(mob).unwrap().pins(), usize::MAX);
    assert_eq!(stage.snapshot_bytes().unwrap(), before);

    stage.delete(mob).unwrap();
    assert!(stage.contains(mob), "outstanding pins defer deletion");
    stage.unpin(mob);
    assert!(
        stage.contains(mob),
        "one unpin from the maximum count cannot finalize deletion"
    );
}

#[test]
fn declared_record_payload_is_preflighted_before_buffer_allocation() {
    let stage = Stage::new();

    let truncated = snapshot_with_declared_record_payload(1, 3, 1_000_000);
    let error = Snapshot::from_bytes(&truncated, &stage)
        .map(|_| ())
        .expect_err("missing column bytes must be refused before allocation");
    assert!(
        matches!(
            error,
            PersistError::Serial(SerialError::UnexpectedEof {
                need: 12_000_000,
                remaining: 0,
            })
        ),
        "expected exact payload EOF, got {error:?}"
    );

    for bytes in [
        snapshot_with_declared_record_payload(1, 3, u32::MAX),
        snapshot_with_declared_record_payload(u16::MAX, u16::MAX, u32::MAX),
    ] {
        let error = Snapshot::from_bytes(&bytes, &stage)
            .map(|_| ())
            .expect_err("an impossible record payload must be refused before allocation");
        assert!(
            matches!(
                &error,
                PersistError::Serial(SerialError::SizeLimit { limit, needed })
                    if *limit == Limits::DEFAULT.max_total && *needed > *limit
            ),
            "expected a record-payload SizeLimit, got {error:?}"
        );
    }
}

#[test]
fn updater_identities_survive_but_callables_do_not() {
    let mut stage = Stage::new();
    let mob = vmob(&mut stage, &[[0.0; 3]], [1.0; 4]);
    let id = stage.add_updater(mob, |_, _| {}, false).unwrap();
    let with_updater = stage.snapshot_bytes().unwrap();

    let decoded = Snapshot::from_bytes(&with_updater, &stage).unwrap();
    // The manifest carries (id, kind)…
    assert_eq!(decoded.updaters.entries.len(), 1);
    let (_, ids) = &decoded.updaters.entries[0];
    assert_eq!(ids.len(), 1);
    assert_eq!(
        ids[0].1,
        fmn_mobject::UpdaterKindTag::NonDt,
        "kind recorded"
    );
    // …the restored stage carries no callables…
    stage.restore(&decoded.snapshot);
    assert!(stage.updater_ids(mob).is_empty());
    // …and re-encoding therefore differs (the documented honesty clause).
    let reencoded = stage.snapshot_bytes().unwrap();
    assert_ne!(with_updater, reencoded);

    let identities = decoded.updaters.identities(&stage).unwrap();
    assert_eq!(identities.len(), 1);
    assert_eq!(identities[0].mob, mob);
    assert_eq!(identities[0].id, id);
    let calls = Rc::new(Cell::new(0));
    let seen = Rc::clone(&calls);
    stage
        .restore_updater_bindings(vec![(
            identities[0],
            UpdaterFn::NonDt(Rc::new(RefCell::new(
                move |_stage: &mut Stage, _mob: Mob| {
                    seen.set(seen.get() + 1);
                },
            ))),
        )])
        .unwrap();
    assert_eq!(stage.updater_ids(mob), [id]);
    stage.update_mobject(mob, 0.0);
    assert_eq!(calls.get(), 1);
    assert_eq!(
        stage.snapshot_bytes().unwrap(),
        with_updater,
        "identity-preserving rebind restores the canonical state"
    );
    let next = stage.add_updater(mob, |_, _| {}, false).unwrap();
    assert_ne!(
        next, id,
        "durable restore must not reuse an identity carried by its manifest"
    );
}

#[test]
fn removed_updater_id_cursor_is_part_of_durable_state_and_hashing() {
    let mut stage = Stage::new();
    let mob = vmob(&mut stage, &[[0.0; 3]], [1.0; 4]);
    let before = stage.snapshot_bytes().unwrap();
    let removed = stage.add_updater(mob, |_, _| {}, false).unwrap();
    stage.remove_updater(mob, removed);
    let after = stage.snapshot_bytes().unwrap();
    assert_ne!(
        before, after,
        "identical active records with different future id sequences are different states"
    );

    let decoded = Snapshot::from_bytes(&after, &stage).unwrap();
    assert!(decoded.updaters.entries.is_empty());
    stage.restore(&decoded.snapshot);
    let next = stage.add_updater(mob, |_, _| {}, false).unwrap();
    assert_ne!(
        next, removed,
        "a barrier must not reuse an updater identity removed before capture"
    );
}

#[test]
fn updater_manifest_refuses_noncanonical_or_ambiguous_identities_before_resolution() {
    let mut stage = Stage::new();
    let mob = vmob(&mut stage, &[[0.0; 3]], [1.0; 4]);
    stage.add_updater(mob, |_, _| {}, false).unwrap();
    let decoded = Snapshot::from_bytes(&stage.snapshot_bytes().unwrap(), &stage).unwrap();
    let (slot, ids) = decoded.updaters.entries[0].clone();
    let id = ids[0].0;

    for (manifest, expected) in [
        (
            UpdaterManifest {
                entries: vec![(slot, vec![(0, UpdaterKindTag::NonDt)])],
            },
            "reserved zero identity",
        ),
        (
            UpdaterManifest {
                entries: vec![(
                    slot,
                    vec![(id, UpdaterKindTag::NonDt), (id, UpdaterKindTag::NonDt)],
                )],
            },
            "repeats an identity",
        ),
        (
            UpdaterManifest {
                entries: vec![
                    (slot, vec![(id, UpdaterKindTag::NonDt)]),
                    (slot, vec![(id + 1, UpdaterKindTag::NonDt)]),
                ],
            },
            "not strictly increasing",
        ),
        (
            UpdaterManifest {
                entries: vec![(slot, Vec::new())],
            },
            "empty slot",
        ),
    ] {
        let error = manifest
            .identities(&stage)
            .expect_err("malformed manifests must fail before callback resolution");
        assert!(
            error.to_string().contains(expected),
            "expected {expected:?}, got {error}"
        );
    }
}

#[test]
fn future_major_is_refused_by_name() {
    let mut stage = Stage::new();
    build_scene(&mut stage);
    let mut bytes = stage.snapshot_bytes().unwrap();
    // Header: magic[4] | schema u32 | major u16 LE at offset 8.
    bytes[8] = bytes[8].wrapping_add(1);
    // Re-seal the checksum so only the version differs.
    let body_len = bytes.len() - 32;
    let digest = sha256(&bytes[..body_len]);
    bytes[body_len..].copy_from_slice(digest.as_bytes());
    let err = Snapshot::from_bytes(&bytes, &stage)
        .map(|_| ())
        .expect_err("a future major must be refused");
    assert!(
        matches!(err, PersistError::Serial(SerialError::MajorMismatch { .. })),
        "expected MajorMismatch, got {err:?}"
    );
}

#[test]
fn corruption_is_detected_before_any_payload_is_read() {
    let mut stage = Stage::new();
    build_scene(&mut stage);
    let mut bytes = stage.snapshot_bytes().unwrap();
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0x40;
    let err = Snapshot::from_bytes(&bytes, &stage)
        .map(|_| ())
        .expect_err("corruption must be refused");
    assert!(
        matches!(err, PersistError::Serial(SerialError::ChecksumMismatch)),
        "expected ChecksumMismatch, got {err:?}"
    );
}

#[test]
fn scene_state_round_trips_with_the_rng() {
    let mut stage = Stage::new();
    build_scene(&mut stage);
    let mut rng = Pcg64Dxsm::from_seed(7);
    for _ in 0..5 {
        rng.next_u64();
    }
    let state = SceneState::capture(&stage, 0, 30, 3, &rng);
    let bytes = state.to_bytes().unwrap();
    let decoded = SceneState::from_bytes(&bytes, &stage).unwrap();
    assert_eq!(decoded.frames_elapsed, 0);
    assert_eq!(decoded.fps, 30);
    assert_eq!(decoded.play_count, 3);
    let mut restored = decoded.rng();
    assert_eq!(restored, rng, "generator state is bit-identical");
    assert_eq!(restored.next_u64(), rng.clone().next_u64());
}

#[test]
fn scene_state_distinguishes_adjacent_large_clock_frames() {
    let stage = Stage::new();
    let rng = Pcg64Dxsm::from_seed(7);
    let earlier_frame = 1_i64 << 53;
    let later_frame = earlier_frame + 1;

    let earlier = SceneState::capture(&stage, earlier_frame, 1, 0, &rng);
    let later = SceneState::capture(&stage, later_frame, 1, 0, &rng);

    let earlier_bytes = earlier.to_bytes().unwrap();
    let later_bytes = later.to_bytes().unwrap();
    assert_ne!(
        earlier_bytes, later_bytes,
        "adjacent valid clock frames must remain distinct durable states"
    );
    let decoded = SceneState::from_bytes(&later_bytes, &stage).unwrap();
    assert_eq!(decoded.frames_elapsed, later_frame);
    assert_eq!(decoded.fps, 1);

    let mut previous_major = later_bytes;
    previous_major[8..10].copy_from_slice(&1_u16.to_le_bytes());
    let body_len = previous_major.len() - 32;
    let digest = sha256(&previous_major[..body_len]);
    previous_major[body_len..].copy_from_slice(digest.as_bytes());
    let error = SceneState::from_bytes(&previous_major, &stage)
        .map(|_| ())
        .expect_err("the lossy v1 envelope must not be compatibility-shimmed");
    assert!(matches!(
        error,
        PersistError::Serial(SerialError::MajorMismatch { reader: 2, doc: 1 })
    ));
}

#[test]
fn cross_stage_decode_rebinds_handles() {
    let mut source = Stage::new();
    let (_, c1, _, _) = build_scene(&mut source);
    let points = column(&source, c1, "point");
    let bytes = source.snapshot_bytes().unwrap();

    // A fresh arena with a different process-local mint.
    let mut target = Stage::new();
    let decoded = Snapshot::from_bytes(&bytes, &target).unwrap();
    target.restore(&decoded.snapshot);

    let roots = target.roots().to_vec();
    assert_eq!(roots.len(), 1);
    let family = target.family(roots[0]);
    assert_eq!(family.len(), 3, "root + two children");
    // The first child's geometry travelled intact.
    let restored_points = target
        .get(family[1])
        .unwrap()
        .buffer
        .read_column("point")
        .unwrap();
    assert_eq!(restored_points, points);
    // And the re-bound handles are live in the new stage.
    assert!(target.contains(family[1]));
}

// ------------------------------------------- fm-vek.7: decode budget hardening

/// A tiny canonical container whose counts are extreme or truncated: the
/// decoder must refuse from the count preflight, before reserving the
/// destination storage the count names.
#[test]
fn decode_preflights_extreme_counts_in_tiny_containers() {
    let stage = Stage::new();

    // u32::MAX slots claimed by a four-byte payload.
    let mut w = Writer::new(SNAPSHOT_SCHEMA);
    w.put_u32(u32::MAX);
    let bytes = w.finish().unwrap();
    let error = Snapshot::from_bytes(&bytes, &stage)
        .map(|_| ())
        .unwrap_err();
    assert!(
        matches!(
            error,
            PersistError::Serial(SerialError::UnexpectedEof { .. })
        ),
        "u32::MAX slot count must be an EOF refusal, got {error:?}"
    );

    // One empty slot, then a u32::MAX free-slot ledger the bytes cannot carry.
    let mut w = Writer::new(SNAPSHOT_SCHEMA);
    w.put_u32(1).put_u32(7).put_bool(false).put_u32(u32::MAX);
    let bytes = w.finish().unwrap();
    let error = Snapshot::from_bytes(&bytes, &stage)
        .map(|_| ())
        .unwrap_err();
    assert!(
        matches!(
            error,
            PersistError::Serial(SerialError::UnexpectedEof { .. })
        ),
        "u32::MAX free count must be an EOF refusal, got {error:?}"
    );

    // One live slot whose record field table is truncated at the count.
    let mut w = Writer::new(SNAPSHOT_SCHEMA);
    w.put_u32(1).put_u32(0).put_bool(true).put_u16(u16::MAX);
    let bytes = w.finish().unwrap();
    let error = Snapshot::from_bytes(&bytes, &stage)
        .map(|_| ())
        .unwrap_err();
    assert!(
        matches!(
            error,
            PersistError::Serial(SerialError::UnexpectedEof { .. })
        ),
        "u16::MAX field count must be an EOF refusal, got {error:?}"
    );

    // The same extreme count, but with an explicit budget so tight the
    // aggregate decoded-allocation charge refuses it first — the typed
    // budget channel.
    let mut w = Writer::new(SNAPSHOT_SCHEMA);
    w.put_u32(2)
        .put_u32(0)
        .put_bool(false)
        .put_u32(0)
        .put_bool(false);
    w.put_u32(0).put_u32(0).put_u64(1); // free, roots, updater cursor
    let bytes = w.finish().unwrap();
    let error = Snapshot::from_bytes_with_limits(
        &bytes,
        &stage,
        SnapshotLimits {
            max_total_decoded_bytes: 1,
        },
    )
    .map(|_| ())
    .unwrap_err();
    assert!(
        matches!(
            error,
            PersistError::AllocationLimit {
                limit: 1,
                what: "arena slot table",
                ..
            }
        ),
        "the slot-table charge must be the typed budget refusal, got {error:?}"
    );
}

/// The budget is one aggregate account: the exact ceiling admits the
/// decode, one byte below it refuses with the typed error.
#[test]
fn decode_budget_boundary_success_and_one_byte_refusal() {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_points(&[[1.0, 2.0, 3.0]]));
    stage.add_to_scene(mob).unwrap();
    let bytes = stage.snapshot_bytes().unwrap();

    // Find the exact ceiling empirically: the smallest 64-byte step that
    // admits the decode, then binary-search the boundary inside the step.
    let mut step = 64_usize;
    while Snapshot::from_bytes_with_limits(
        &bytes,
        &stage,
        SnapshotLimits {
            max_total_decoded_bytes: step,
        },
    )
    .is_err()
    {
        step *= 2;
        assert!(step <= SnapshotLimits::DEFAULT.max_total_decoded_bytes);
    }
    let (mut lo, mut hi) = (0, step); // zero always refuses; hi admits
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if Snapshot::from_bytes_with_limits(
            &bytes,
            &stage,
            SnapshotLimits {
                max_total_decoded_bytes: mid,
            },
        )
        .is_ok()
        {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    // Boundary success: the exact ceiling decodes to the same canonical bytes.
    let decoded = Snapshot::from_bytes_with_limits(
        &bytes,
        &stage,
        SnapshotLimits {
            max_total_decoded_bytes: hi,
        },
    )
    .unwrap();
    assert_eq!(decoded.snapshot.to_bytes().unwrap(), bytes);
    // One byte under: the typed budget refusal, naming a structure.
    let error = Snapshot::from_bytes_with_limits(
        &bytes,
        &stage,
        SnapshotLimits {
            max_total_decoded_bytes: hi - 1,
        },
    )
    .map(|_| ())
    .unwrap_err();
    assert!(
        matches!(
            error,
            PersistError::AllocationLimit { limit, .. } if limit == hi - 1
        ),
        "one byte under the ceiling must be the typed refusal, got {error:?}"
    );
}

/// A large, valid, mostly-empty arena decodes under the default budget and
/// re-encodes to identical canonical bytes.
#[test]
fn large_valid_sparse_arena_decodes_under_the_default_budget() {
    const SLOTS: usize = 50_000;
    let mut stage = Stage::new();
    let mobs: Vec<Mob> = (0..SLOTS).map(|_| stage.add(Mobject::new())).collect();
    // Keep every eighth mobject live in the scene; delete the rest so the
    // arena is large and sparse (empty slots + a long free ledger).
    for (index, &mob) in mobs.iter().enumerate() {
        if index % 8 == 0 {
            stage.add_to_scene(mob).unwrap();
        } else {
            stage.delete(mob).unwrap();
        }
    }
    let bytes = stage.snapshot_bytes().unwrap();
    let decoded = Snapshot::from_bytes(&bytes, &stage).unwrap();
    // Canonical bytes survive the budget-checked decode exactly…
    assert_eq!(decoded.snapshot.to_bytes().unwrap(), bytes);
    // …and the decoded arena restores: every kept mobject is a live root.
    // Handles re-bind at decode, so decode against the restore target.
    let mut restored = Stage::new();
    let rebound = Snapshot::from_bytes(&bytes, &restored).unwrap();
    restored.restore(&rebound.snapshot);
    assert_eq!(restored.roots().len(), SLOTS / 8);
}

/// The scene-state envelope decodes its nested snapshot document by
/// borrowing, not cloning: a hostile nested count is refused by the same
/// preflight, through the envelope path.
#[test]
fn scene_state_decode_preflights_the_nested_snapshot() {
    let stage = Stage::new();
    let mut inner = Writer::new(SNAPSHOT_SCHEMA);
    inner.put_u32(u32::MAX);
    let inner_bytes = inner.finish().unwrap();

    let mut w = Writer::new(fmn_mobject::SCENE_STATE_SCHEMA);
    w.put_i64(0).put_u32(30).put_u64(0);
    w.put_u64(0).put_u64(0).put_u64(0).put_u64(0);
    w.put_bytes(&inner_bytes);
    let bytes = w.finish().unwrap();

    let error = SceneState::from_bytes(&bytes, &stage)
        .map(|_| ())
        .unwrap_err();
    assert!(
        matches!(
            error,
            PersistError::Serial(SerialError::UnexpectedEof { .. })
        ),
        "nested u32::MAX slot count must be an EOF refusal, got {error:?}"
    );

    // And a well-formed envelope still round-trips through the same path.
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_points(&[[0.0, 1.0, 2.0]]));
    stage.add_to_scene(mob).unwrap();
    let rng = Pcg64Dxsm::from_seed(42);
    let state = SceneState::capture(&stage, 0, 30, 7, &rng);
    let bytes = state.to_bytes().unwrap();
    let decoded = SceneState::from_bytes(&bytes, &stage).unwrap();
    assert_eq!(decoded.play_count, 7);
    assert_eq!(
        decoded.snapshot.to_bytes().unwrap(),
        state.snapshot.to_bytes().unwrap()
    );
}
