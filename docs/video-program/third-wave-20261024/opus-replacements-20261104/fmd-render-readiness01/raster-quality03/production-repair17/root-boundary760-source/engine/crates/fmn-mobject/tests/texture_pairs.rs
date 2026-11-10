//! Durable two-image materials: storage, revisions, replay and hostile bytes.
use fmn_hash::sha256;
use fmn_mobject::{
    ImageColorSpace, ImageResource, ImageResourceError, ImageSampler, ImageWrap, Mobject,
    PersistError, Snapshot, SnapshotLimits, Stage,
};

fn image(rgb: [u8; 3]) -> ImageResource {
    ImageResource::rgba8(
        1,
        1,
        vec![rgb[0], rgb[1], rgb[2], 255],
        ImageColorSpace::Linear,
        ImageSampler::default(),
    )
    .unwrap()
}

#[test]
fn immutable_pair_shares_both_buffers_and_can_replace_or_remove_dark() {
    let light = image([255, 0, 0]);
    let dark = image([0, 0, 255]);
    let pair = light.clone().with_dark_image(dark.clone()).unwrap();
    assert_eq!(pair.content_digest(), light.content_digest());
    assert_eq!(pair.pixels().as_ptr(), light.pixels().as_ptr());
    assert_eq!(
        pair.dark_image().unwrap().pixels().as_ptr(),
        dark.pixels().as_ptr()
    );
    assert_eq!(pair.clone().without_dark_image(), light);
    assert_eq!(
        pair.with_dark_image(image([0, 255, 0]))
            .unwrap()
            .dark_image()
            .unwrap()
            .pixels(),
        &[0, 255, 0, 255]
    );
}

#[test]
fn pairs_refuse_nesting_and_conflicting_sampler_policies() {
    let pair = image([1, 2, 3]).with_dark_image(image([4, 5, 6])).unwrap();
    assert_eq!(
        image([7, 8, 9]).with_dark_image(pair).unwrap_err(),
        ImageResourceError::NestedTexturePair
    );
    let dark = ImageResource::rgba8(
        1,
        1,
        vec![0; 4],
        ImageColorSpace::Srgb,
        ImageSampler {
            wrap_u: ImageWrap::ClampToEdge,
            ..ImageSampler::default()
        },
    )
    .unwrap();
    assert_eq!(
        image([1, 2, 3]).with_dark_image(dark).unwrap_err(),
        ImageResourceError::SamplerMismatch
    );
}

#[test]
fn pair_preserves_independent_dimensions_and_transfer_functions() {
    let dark = ImageResource::rgba8(
        2,
        1,
        vec![128; 8],
        ImageColorSpace::Srgb,
        ImageSampler::default(),
    )
    .unwrap();
    let pair = image([255, 0, 0]).with_dark_image(dark.clone()).unwrap();
    assert_eq!(pair.width(), 1);
    assert_eq!(pair.dark_image(), Some(&dark));
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::new().with_image_resource(pair.clone()));
    let bytes = stage.snapshot_bytes().unwrap();
    let decoded = Snapshot::from_bytes(&bytes, &stage).unwrap();
    stage.restore(&decoded.snapshot);
    assert_eq!(stage.get(mob).unwrap().image_resource(), Some(&pair));
    assert_eq!(stage.snapshot_bytes().unwrap(), bytes);
}

#[test]
fn copies_become_and_memory_restore_keep_both_resources() {
    let pair = image([255, 0, 0])
        .with_dark_image(image([0, 0, 255]))
        .unwrap();
    let mut stage = Stage::new();
    let source = stage.add(Mobject::new().with_image_resource(pair.clone()));
    let copied = stage.copy_family(source).unwrap();
    let target = stage.add(Mobject::new());
    stage.become_mobject(target, source, false).unwrap();
    for mob in [source, copied, target] {
        let actual = stage.get(mob).unwrap().image_resource().unwrap();
        assert_eq!(actual, &pair);
        assert_eq!(
            actual.dark_image().unwrap().pixels().as_ptr(),
            pair.dark_image().unwrap().pixels().as_ptr()
        );
    }
    let snapshot = stage.snapshot();
    stage.set_image_resource(target, None).unwrap();
    stage.restore(&snapshot);
    assert_eq!(stage.get(target).unwrap().image_resource(), Some(&pair));
}

#[test]
fn dark_only_replacement_changes_revision_and_snapshot_identity() {
    let pair = image([255, 0, 0])
        .with_dark_image(image([0, 0, 255]))
        .unwrap();
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::new().with_image_resource(pair.clone()));
    let revision = stage.get(mob).unwrap().image_revision();
    let first = stage.snapshot_bytes().unwrap();
    assert!(!stage.set_image_resource(mob, Some(pair.clone())).unwrap());
    assert_eq!(stage.get(mob).unwrap().image_revision(), revision);
    let changed = pair.with_dark_image(image([0, 255, 0])).unwrap();
    assert!(stage.set_image_resource(mob, Some(changed)).unwrap());
    assert_eq!(
        stage.get(mob).unwrap().image_revision(),
        revision.wrapping_add(1)
    );
    assert_ne!(first, stage.snapshot_bytes().unwrap());
    let mut fresh = Stage::new();
    let decoded = Snapshot::from_bytes(&first, &fresh).unwrap();
    fresh.restore(&decoded.snapshot);
    assert_eq!(fresh.snapshot_bytes().unwrap(), first);
}

#[test]
fn dark_digest_corruption_is_refused_even_with_a_valid_outer_checksum() {
    let dark = image([19, 37, 91]);
    let digest = dark.content_digest();
    let mut stage = Stage::new();
    stage
        .add(Mobject::new().with_image_resource(image([255, 0, 0]).with_dark_image(dark).unwrap()));
    let mut bytes = stage.snapshot_bytes().unwrap();
    let end = bytes.len() - 32;
    let at = bytes[..end]
        .windows(32)
        .position(|window| window == digest.as_bytes())
        .unwrap();
    bytes[at] ^= 1;
    let checksum = sha256(&bytes[..end]);
    bytes[end..].copy_from_slice(checksum.as_bytes());
    assert!(matches!(
        Snapshot::from_bytes(&bytes, &stage),
        Err(PersistError::Malformed("image content digest mismatch"))
    ));
}

#[test]
fn previous_minor_single_image_snapshots_remain_readable() {
    let mut stage = Stage::new();
    let resource = image([47, 83, 109]);
    let mob = stage.add(Mobject::new().with_image_resource(resource.clone()));
    let current = stage.snapshot_bytes().unwrap();
    let mut old = current[..current.len() - 32 - 2].to_vec();
    assert_eq!(&current[current.len() - 34..current.len() - 32], &[1, 0]);
    old[10..12].copy_from_slice(&7u16.to_le_bytes());
    let payload = u64::try_from(old.len() - 24).unwrap();
    old[16..24].copy_from_slice(&payload.to_le_bytes());
    old.extend_from_slice(sha256(&old).as_bytes());
    let decoded = Snapshot::from_bytes(&old, &stage).unwrap();
    stage.restore(&decoded.snapshot);
    assert_eq!(stage.get(mob).unwrap().image_resource(), Some(&resource));
    assert_eq!(stage.snapshot_bytes().unwrap(), current);
}

#[test]
fn pair_decode_charges_both_images_to_the_aggregate_budget() {
    let mut stage = Stage::new();
    stage.add(
        Mobject::new().with_image_resource(
            image([255, 0, 0])
                .with_dark_image(
                    ImageResource::rgba8(
                        128,
                        128,
                        vec![31; 128 * 128 * 4],
                        ImageColorSpace::Linear,
                        ImageSampler::default(),
                    )
                    .unwrap(),
                )
                .unwrap(),
        ),
    );
    let bytes = stage.snapshot_bytes().unwrap();
    assert!(Snapshot::from_bytes(&bytes, &stage).is_ok());
    assert!(matches!(
        Snapshot::from_bytes_with_limits(
            &bytes,
            &stage,
            SnapshotLimits {
                max_total_decoded_bytes: 65536
            }
        ),
        Err(PersistError::AllocationLimit { .. })
    ));
}
