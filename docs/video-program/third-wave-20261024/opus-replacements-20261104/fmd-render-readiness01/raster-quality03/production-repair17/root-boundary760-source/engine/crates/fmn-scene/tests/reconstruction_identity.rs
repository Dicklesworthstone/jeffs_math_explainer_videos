//! Renderer identity and every captured input belong to the reconstruction proof.
use fmn_anim::bundle::RECONSTRUCTION_LAW_VERSION;
use fmn_anim::{
    AnimConfig, AnimState, Animation, AnimationSignature, RateFunc, Timeline, prepare_animation,
};
use fmn_core::{rate, rng::RngRoot};
use fmn_hash::{serial::Writer, sha256};
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::{
    ImageColorSpace, ImageResource, ImageSampler, Mob, Mobject, RecordBuffer, RecordSchema,
    RenderPrimitive, ShapeTag, Stage,
};
use fmn_render::engine::EngineIdentity;
use fmn_scene::{
    BundleReadError, BundleSegmentKind, TIMELINE_BUNDLE_SCHEMA, TimelineBundle,
    bundle_engine_version, export_timeline_bundle,
};

#[test]
fn bundle_identity_binds_renderer_and_shared_reconstruction_law() {
    assert_eq!(RECONSTRUCTION_LAW_VERSION, 2);
    assert_eq!(
        bundle_engine_version(),
        format!(
            "{}:fmtl-law:2",
            EngineIdentity::certified().closure_string()
        )
    );
}

#[test]
fn legacy_and_mismatched_laws_refuse_before_later_fields_are_interpreted() {
    let renderer = EngineIdentity::certified().closure_string();
    for identity in [
        renderer.clone(),
        format!("{renderer}:fmtl-law:1"),
        format!("{renderer}:fmtl-law:3"),
    ] {
        let mut writer = Writer::new(TIMELINE_BUNDLE_SCHEMA);
        writer.put_str(&identity);
        let bytes = writer.finish().unwrap();
        match TimelineBundle::from_bytes(&bytes) {
            Err(BundleReadError::EngineMismatch { wanted, found }) => {
                assert_eq!(wanted, identity);
                assert_eq!(found, bundle_engine_version());
            }
            _ => panic!("incompatible law must fail before reading the intentionally absent fps"),
        }
    }
}

#[test]
fn native_tracker_exports_carry_the_law_that_reconstructs_their_state() {
    let mut stage = Stage::new();
    let tracker = stage.add_value_tracker(1.0);
    stage.add_to_scene(tracker).unwrap();
    let builder = tracker
        .animate()
        .set_anim_args(AnimateArgs {
            rate_func: Some(rate::linear),
            ..AnimateArgs::default()
        })
        .unwrap()
        .set_value(9.0)
        .unwrap();
    let animation = prepare_animation(builder, &mut stage).unwrap();
    let mut timeline = Timeline::new(4).unwrap();
    timeline.play(vec![animation]).unwrap();
    let bytes = export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(0)).unwrap();
    let bundle = TimelineBundle::from_bytes(&bytes).unwrap();
    assert_eq!(bundle.engine_version(), bundle_engine_version());
    let frame = bundle.stage_at(1).unwrap();
    assert_eq!(frame.tracker_value(frame.roots()[0]), Some(5.0));
}

/// These mutations are functions of their begin state and alpha. They really
/// are pure, but the shared record-lerp law cannot express their discrete work.
/// The exporter must prove representability rather than trusting the signature.
struct PureMutation<F> {
    state: AnimState,
    change: F,
}

impl<F: FnMut(&mut Stage, f64)> Animation for PureMutation<F> {
    fn state(&self) -> &AnimState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut AnimState {
        &mut self.state
    }

    fn interpolate(&mut self, stage: &mut Stage, alpha: f64) {
        (self.change)(stage, alpha);
    }

    fn interpolate_submobject(&mut self, _: &mut Stage, _: &[Mob], _: f64) {}

    fn effect_signature(&self) -> AnimationSignature {
        AnimationSignature::Pure
    }
}

fn mutation(mob: Mob, change: impl FnMut(&mut Stage, f64) + 'static) -> Timeline {
    let mut timeline = Timeline::new(4).unwrap();
    timeline
        .play(vec![Box::new(PureMutation {
            state: AnimState::new(
                mob,
                AnimConfig {
                    rate_func: RateFunc::linear(),
                    ..AnimConfig::default()
                },
            ),
            change,
        })])
        .unwrap();
    timeline
}

fn middle(alpha: f64) -> bool {
    (0.5..1.0).contains(&alpha)
}

/// Compare the actual driver's captures with both production playback paths.
/// Render-only canonical bytes cover aliasing, material identity and all draw
/// inputs while excluding private animation copies and execution bookkeeping.
fn round_trip(
    name: &str,
    make: impl Fn() -> (Stage, Timeline),
    kind: BundleSegmentKind,
) -> (TimelineBundle, Vec<Stage>) {
    let rng = RngRoot::from_seed(19);
    let (mut direct, mut timeline) = make();
    let mut observed = Vec::new();
    let reports = timeline
        .render(&mut direct, &rng, &mut |packet| {
            observed.push(packet.materialize_stage());
        })
        .unwrap();
    assert!(
        reports[0].purity.is_pure(),
        "{name}: signature was not nominated"
    );
    let (mut source, timeline) = make();
    let bytes = export_timeline_bundle(timeline, &mut source, &rng).unwrap();
    let bundle = TimelineBundle::from_bytes(&bytes).unwrap();
    assert_eq!(bundle.frame_count(), 4);
    let shared = TimelineBundle::from_bytes(&bytes)
        .unwrap()
        .into_shared()
        .unwrap();
    let mut cache = fmn_scene::timeline_bundle::TimelineFrameCache::default();
    for index in [3, 1, 0, 2, 1] {
        let expected = observed[index as usize]
            .snapshot()
            .to_render_bytes()
            .unwrap();
        for (path, actual) in [
            ("ordinary", bundle.stage_at(index).unwrap()),
            (
                "shared",
                cache
                    .materialize(&shared.frame_job(index).unwrap())
                    .unwrap(),
            ),
        ] {
            let actual = actual.snapshot().to_render_bytes().unwrap();
            let expected_digest = sha256(&expected).to_hex();
            let actual_digest = sha256(&actual).to_hex();
            println!(
                "{{\"scenario\":\"fmtl-proof-{name}\",\"frame\":{index},\"path\":\"{path}\",\"expected\":\"{expected_digest}\",\"actual\":\"{actual_digest}\",\"equal\":{}}}",
                expected == actual,
            );
            assert!(expected == actual, "{name}: frame {index} on {path} replay");
        }
    }
    assert_eq!(bundle.segment_kind(0), Some(kind), "{name}");
    (bundle, observed)
}

#[test]
fn discrete_uniform_changes_cannot_pass_the_pure_export_proof() {
    for flag in 0..5 {
        round_trip(
            &format!("uniform-{flag}"),
            || {
                let mut stage = Stage::new();
                let mob = stage.add(Mobject::from_points(&[[0.0, 0.0, 0.0]]));
                stage.add_to_scene(mob).unwrap();
                let timeline = mutation(mob, move |stage, alpha| {
                    let uniforms = stage.get_mut(mob).unwrap().uniforms_mut();
                    let value = middle(alpha);
                    match flag {
                        0 => uniforms.flat_stroke = value,
                        1 => uniforms.scale_stroke_with_zoom = value,
                        2 => uniforms.stroke_behind = value,
                        3 => uniforms.depth_test = value,
                        4 => uniforms.use_winding_fill = value,
                        _ => unreachable!(),
                    }
                });
                (stage, timeline)
            },
            BundleSegmentKind::Stateful,
        );
    }
}

#[test]
fn shared_children_cannot_be_replaced_by_equal_but_independent_drawables() {
    let (bundle, _) = round_trip(
        "shared-child",
        || {
            let mut stage = Stage::new();
            let left = stage.add(Mobject::new());
            let right = stage.add(Mobject::new());
            let a = stage.add(Mobject::from_points(&[[0.0, 0.0, 0.0]]));
            let b = stage.add(Mobject::from_points(&[[0.0, 0.0, 0.0]]));
            stage.attach(left, a).unwrap();
            stage.attach(right, b).unwrap();
            stage.add_many_to_scene(&[left, right]).unwrap();
            let timeline = mutation(left, move |stage, alpha| {
                stage
                    .replace_children(right, &[if middle(alpha) { a } else { b }])
                    .unwrap();
            });
            (stage, timeline)
        },
        BundleSegmentKind::Stateful,
    );
    let shared_frame = bundle.stage_at(1).unwrap();
    let roots = shared_frame.roots();
    assert_eq!(
        shared_frame.get(roots[0]).unwrap().submobjects(),
        shared_frame.get(roots[1]).unwrap().submobjects(),
    );
    let separate_frame = bundle.stage_at(3).unwrap();
    let roots = separate_frame.roots();
    assert_ne!(
        separate_frame.get(roots[0]).unwrap().submobjects(),
        separate_frame.get(roots[1]).unwrap().submobjects(),
    );
}

#[test]
fn shape_identity_and_hint_validity_belong_to_the_proof() {
    for invalidate in [false, true] {
        round_trip(
            if invalidate {
                "hint-validity"
            } else {
                "shape-tag"
            },
            || {
                let mut stage = Stage::new();
                let mob = stage.add(Mobject::from_points(&[[0.0, 0.0, 0.0]]));
                stage.add_to_scene(mob).unwrap();
                let circle = ShapeTag::Circle {
                    center: [0.0; 3],
                    radius: 1.0,
                };
                let timeline = mutation(mob, move |stage, alpha| {
                    stage.set_shape(mob, circle);
                    if middle(alpha) {
                        if invalidate {
                            // Same coordinates, different truth of the hint.
                            stage
                                .get_mut(mob)
                                .unwrap()
                                .buffer
                                .write(0, "point", &[0.0, 0.0, 0.0]);
                        } else {
                            stage.clear_shape(mob);
                        }
                    }
                });
                (stage, timeline)
            },
            BundleSegmentKind::Stateful,
        );
    }
}

#[test]
fn record_layout_key_sets_cannot_disappear_during_reconstruction() {
    round_trip(
        "record-schema",
        || {
            let mut stage = Stage::new();
            let mob = stage.add(Mobject::from_points(&[[0.0, 0.0, 0.0]]));
            stage.add_to_scene(mob).unwrap();
            let timeline = mutation(mob, move |stage, alpha| {
                let keys: &[&str] = if middle(alpha) { &[] } else { &["point"] };
                let schema = RecordSchema::new(&[("point", 3), ("rgba", 4)], keys, keys).unwrap();
                stage.get_mut(mob).unwrap().buffer = RecordBuffer::new(schema, 1).unwrap();
            });
            (stage, timeline)
        },
        BundleSegmentKind::Stateful,
    );
}

#[test]
fn renderer_program_changes_cannot_be_proven_by_equal_record_columns() {
    round_trip(
        "render-program",
        || {
            let mut stage = Stage::new();
            let mob = stage.add(Mobject::from_points(&[[0.0, 0.0, 0.0]]));
            stage.add_to_scene(mob).unwrap();
            let timeline = mutation(mob, move |stage, alpha| {
                stage
                    .install(
                        mob,
                        Mobject::from_points(&[[0.0, 0.0, 0.0]]).with_render_primitive(
                            if middle(alpha) {
                                RenderPrimitive::DotCloud
                            } else {
                                RenderPrimitive::Vector
                            },
                        ),
                    )
                    .unwrap();
            });
            (stage, timeline)
        },
        BundleSegmentKind::Stateful,
    );
}

fn image(rgba: [u8; 4]) -> ImageResource {
    ImageResource::rgba8(
        1,
        1,
        rgba.to_vec(),
        ImageColorSpace::Srgb,
        ImageSampler::default(),
    )
    .unwrap()
}

fn image_quad(resource: ImageResource) -> Mobject {
    let schema = RecordSchema::new(
        &[("point", 3), ("im_coords", 2), ("opacity", 1)],
        &["point"],
        &["point"],
    )
    .unwrap();
    let mut buffer = RecordBuffer::new(schema, 6).unwrap();
    let corners = [
        [-2.0, -2.0, 0.0],
        [-2.0, 2.0, 0.0],
        [2.0, -2.0, 0.0],
        [2.0, 2.0, 0.0],
    ];
    for (row, index) in [0, 1, 2, 2, 1, 3].into_iter().enumerate() {
        buffer.write(row, "point", &corners[index]);
        buffer.write(row, "im_coords", &[0.5, 0.5]);
        buffer.write(row, "opacity", &[1.0]);
    }
    Mobject::from_buffer(buffer).with_image_resource(resource)
}

#[test]
fn primary_and_dark_image_replacements_are_recorded_and_replayed() {
    for dark in [false, true] {
        let (bundle, observed) = round_trip(
            if dark { "dark-image" } else { "primary-image" },
            || {
                let mut stage = Stage::new();
                let red = image([255, 0, 0, 255]);
                let blue = image([0, 0, 255, 255]);
                let initial = if dark {
                    red.clone().with_dark_image(red.clone()).unwrap()
                } else {
                    red.clone()
                };
                let mob = stage.add(image_quad(initial));
                stage.add_to_scene(mob).unwrap();
                let timeline = mutation(mob, move |stage, alpha| {
                    let color = if middle(alpha) { &blue } else { &red };
                    let resource = if dark {
                        red.clone().with_dark_image(color.clone()).unwrap()
                    } else {
                        color.clone()
                    };
                    stage.set_image_resource(mob, Some(resource)).unwrap();
                });
                (stage, timeline)
            },
            BundleSegmentKind::Stateful,
        );
        if !dark {
            let render = |stage: &Stage| {
                use fmn_render::{
                    Camera, CameraConfig, FrameConfig, RetainedFrameRenderer,
                    RetainedFrameRendererConfig, ScreenMap, Tiling, Viewport,
                };
                let camera = Camera::new(CameraConfig {
                    resolution: (32, 24),
                    fps: 4,
                    samples: 1,
                    ..CameraConfig::default()
                })
                .unwrap();
                let mut renderer = RetainedFrameRenderer::new(RetainedFrameRendererConfig {
                    frame: FrameConfig::new(
                        Viewport {
                            width: 32,
                            height: 24,
                        },
                        ScreenMap::y_up(3.0, [16.0, 12.0]),
                        camera.background(),
                    ),
                    tiling: Tiling {
                        macro_tile: 16,
                        fine_tile: 8,
                    },
                    engine: EngineIdentity::certified(),
                    threads: 1,
                })
                .unwrap();
                renderer.render_with_camera(stage, &camera).unwrap();
                renderer.frame().plane(0).to_vec()
            };
            assert_ne!(render(&observed[0]), render(&observed[1]));
            for index in 0..4 {
                assert_eq!(
                    render(&observed[index as usize]),
                    render(&bundle.stage_at(index).unwrap()),
                    "image replacement frame {index}",
                );
            }
        }
    }
}

#[test]
fn unchanged_materials_and_shared_geometry_keep_proven_compaction() {
    round_trip(
        "pure-image-hold",
        || {
            let mut stage = Stage::new();
            let image = stage.add(image_quad(image([255, 0, 0, 255])));
            let left = stage.add(Mobject::new());
            let right = stage.add(Mobject::new());
            stage.attach(left, image).unwrap();
            stage.attach(right, image).unwrap();
            stage.add_many_to_scene(&[left, right]).unwrap();
            let mut timeline = Timeline::new(4).unwrap();
            timeline.wait(1.0).unwrap();
            (stage, timeline)
        },
        BundleSegmentKind::Pure,
    );
}
