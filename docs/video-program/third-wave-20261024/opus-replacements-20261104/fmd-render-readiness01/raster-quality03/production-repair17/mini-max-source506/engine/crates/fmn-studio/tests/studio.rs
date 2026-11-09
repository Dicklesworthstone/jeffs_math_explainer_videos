//! fm-yh0 Studio acceptance: bounded frame fanout, deterministic inspector
//! ids and overlays, transient-vs-committed timeline seeks, terminal protocol
//! bytes, and a real pseudoterminal write on Unix.

use std::io::{self, Write};
use std::sync::Arc;

use fmn_anim::Timeline;
use fmn_codec::{CompressionLevel, encode_rgba8};
use fmn_core::rng::RngRoot;
use fmn_hash::sha256;
use fmn_mobject::{Mobject, Stage};
use fmn_scene::{RenderBackendRecord, RenderBackendRole};
use fmn_studio::{
    CapabilityToken, DebugLayerSet, DebugOverlaySnapshot, FrameEncoding, FrameHub, FramePayload,
    FrameStream, InspectError, InspectorLimits, InspectorSnapshot, InspectorView,
    NativeSpanBinding, NodeOverlay, ProtocolLimits, ScrubMode, SpanKind, SpanRegistry,
    TerminalPreview, TerminalProtocol, TileOverlay, TuiError, TuiLimits, commit_timeline_frame,
    preview_timeline_frame,
};

struct PayloadWitness<'a> {
    payload: &'a [u8],
    saw_borrowed_payload: bool,
    bytes: Vec<u8>,
}

fn test_render_backend() -> RenderBackendRecord {
    RenderBackendRecord::new(
        RenderBackendRole::FrameStream,
        b"studio-test-render-backend".to_vec(),
    )
    .unwrap()
}

impl Write for PayloadWitness<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if std::ptr::eq(bytes, self.payload) {
            self.saw_borrowed_payload = true;
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn capability_is_explicit_fixed_width_and_redacted() {
    let token = CapabilityToken::new([0x5a; 32]).unwrap();
    let hex = token.try_expose_hex().unwrap();
    assert_eq!(hex.len(), 64);
    assert_eq!(
        CapabilityToken::from_hex(&hex)
            .unwrap()
            .try_expose_hex()
            .unwrap(),
        hex
    );
    assert_eq!(format!("{token:?}"), "CapabilityToken([REDACTED])");
    assert!(CapabilityToken::new([0; 32]).is_err());
}

#[test]
fn frame_hub_validates_converts_and_bounds_multipart_png() {
    let rgba = vec![255, 0, 0, 255, 0, 255, 0, 255];
    let stream = FrameStream {
        scene: "Demo".to_owned(),
        frame_index: 7,
        width: 2,
        height: 1,
        stride: 8,
        encoding: FrameEncoding::Rgba8,
        payload: FramePayload::Pipe {
            digest: sha256(&rgba),
            bytes: rgba,
        },
        render_backends: vec![test_render_backend()],
    };
    let hub = FrameHub::new(2, 1024 * 1024).unwrap();
    let frame = hub.publish(&stream, Default::default()).unwrap();
    assert!(frame.png.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(frame.digest, sha256(&frame.png));
    assert_eq!(frame.publication_sequence, 0);
    assert_eq!(hub.latest().unwrap().frame_index, 7);
    let mut part = PayloadWitness {
        payload: &frame.png,
        saw_borrowed_payload: false,
        bytes: Vec::new(),
    };
    FrameHub::write_multipart_part(&mut part, &frame).unwrap();
    assert!(part.saw_borrowed_payload);
    assert!(
        part.bytes
            .starts_with(b"--fmn-frame\r\nContent-Type: image/png\r\n")
    );
    assert!(part.bytes.ends_with(b"\r\n"));

    let mut scrubbed = stream;
    scrubbed.frame_index = 2;
    let second = hub.publish(&scrubbed, Default::default()).unwrap();
    assert_eq!(second.publication_sequence, 1);
    assert_eq!(
        hub.wait_after(Some(frame.publication_sequence), std::time::Duration::ZERO)
            .unwrap()
            .frame_index,
        2
    );
}

#[test]
fn frame_hub_rejects_invalid_or_mismatched_png_without_history_mutation() {
    let rgba = [255, 0, 0, 255, 0, 255, 0, 255];
    let png = encode_rgba8(2, 1, &rgba, CompressionLevel::Fast);
    let stream = |width, height, frame_index, bytes: Vec<u8>| FrameStream {
        scene: "Demo".to_owned(),
        frame_index,
        width,
        height,
        stride: 0,
        encoding: FrameEncoding::Png,
        payload: FramePayload::Pipe {
            digest: sha256(&bytes),
            bytes,
        },
        render_backends: vec![test_render_backend()],
    };
    let hub = FrameHub::new(2, png.len()).unwrap();
    let first = hub
        .publish(&stream(2, 1, 7, png.clone()), Default::default())
        .expect("exact byte and pixel boundaries are admitted");
    assert_eq!(first.publication_sequence, 0);

    let mut corrupt = png.clone();
    *corrupt.last_mut().expect("encoded PNG is nonempty") ^= 1;
    let error = hub
        .publish(&stream(2, 1, 8, corrupt.clone()), Default::default())
        .expect_err("a corrupt PNG must not enter preview history");
    assert!(error.to_string().contains("crc"));
    assert_eq!(hub.latest().unwrap().frame_index, 7);

    let error = hub
        .publish(&stream(1, 2, 8, png.clone()), Default::default())
        .expect_err("equal-area transposed dimensions must be refused");
    assert!(error.to_string().contains("dimensions"));
    assert_eq!(hub.latest().unwrap().frame_index, 7);

    let error = hub
        .publish(&stream(1, 1, 8, png.clone()), Default::default())
        .expect_err("the PNG must honor the declared frame pixel count");
    assert!(error.to_string().contains("1-pixel budget"));
    assert_eq!(hub.latest().unwrap().frame_index, 7);

    let second = hub
        .publish(&stream(2, 1, 8, png), Default::default())
        .expect("a later valid frame is still admitted");
    assert_eq!(second.publication_sequence, 1);

    let size_first = FrameHub::new(1, 8).unwrap();
    let error = size_first
        .publish(&stream(2, 1, 9, corrupt), Default::default())
        .expect_err("the host byte ceiling must precede PNG codec work");
    assert!(error.to_string().contains("host budget"));
    assert!(size_first.latest().is_none());
}

#[test]
fn frame_hub_binds_png_decode_pixels_to_session_frame_budget() {
    let width = 64u32;
    let height = 64u32;
    let rgba = vec![0; 64 * 64 * 4];
    let png = encode_rgba8(width, height, &rgba, CompressionLevel::Fast);
    assert!(
        png.len() < rgba.len(),
        "the fixture must exercise compression"
    );
    let stream = FrameStream {
        scene: "Demo".to_owned(),
        frame_index: 11,
        width,
        height,
        stride: 0,
        encoding: FrameEncoding::Png,
        payload: FramePayload::Pipe {
            digest: sha256(&png),
            bytes: png.clone(),
        },
        render_backends: vec![test_render_backend()],
    };
    let hub = FrameHub::new(1, png.len()).unwrap();
    let compressed_only = ProtocolLimits {
        max_frame_bytes: png.len(),
        ..ProtocolLimits::default()
    };
    let error = hub
        .publish(&stream, compressed_only)
        .expect_err("decoded RGBA must stay within the negotiated frame budget");
    assert!(error.to_string().contains("pixel budget"));
    assert!(hub.latest().is_none());

    let exact_decoded = ProtocolLimits {
        max_frame_bytes: rgba.len(),
        ..ProtocolLimits::default()
    };
    let published = hub
        .publish(&stream, exact_decoded)
        .expect("the exact decoded RGBA boundary is admitted");
    assert_eq!(published.publication_sequence, 0);
}

#[test]
fn inspector_storage_refusal_is_typed_and_source_preserving() {
    let mut impossible = Vec::<u8>::new();
    let source = impossible
        .try_reserve(usize::MAX)
        .expect_err("impossible capacity must refuse");
    let error = InspectError::StorageUnavailable {
        field: "inspector nodes",
        additional: usize::MAX,
        source,
    };
    assert!(matches!(
        &error,
        InspectError::StorageUnavailable {
            field: "inspector nodes",
            additional,
            ..
        } if *additional == usize::MAX
    ));
    assert!(std::error::Error::source(&error).is_some());
    assert!(error.to_string().contains("inspector nodes"));
}

#[test]
fn studio_json_refuses_before_growing_past_the_first_atom() {
    let tiny = InspectorLimits {
        max_json_bytes: 1,
        ..InspectorLimits::default()
    };
    let inspection = InspectorSnapshot {
        version: 1,
        scene_time: 0.0,
        view: None,
        nodes: Vec::new(),
        truncated: false,
    };
    let overlays = DebugOverlaySnapshot {
        version: 1,
        layers: DebugLayerSet::NONE,
        tiles: Vec::new(),
        nodes: Vec::new(),
        truncated: false,
    };

    for error in [
        inspection.to_json(tiny).unwrap_err(),
        overlays.to_json(tiny).unwrap_err(),
    ] {
        assert!(matches!(
            error,
            InspectError::JsonLimit {
                limit: 1,
                needed: 11
            }
        ));
    }

    let limits = InspectorLimits::default();
    assert_eq!(
        inspection.to_json(limits).unwrap(),
        br#"{"version":1,"scene_time":0,"truncated":false,"nodes":[]}"#
    );
    assert_eq!(
        overlays.to_json(limits).unwrap(),
        br#"{"version":1,"layers":0,"truncated":false,"tiles":[],"nodes":[]}"#
    );
}

#[test]
fn inspector_view_is_additive_and_obeys_the_exact_json_byte_limit() {
    let limits = InspectorLimits::default();
    let mut snapshot =
        InspectorSnapshot::capture(&Stage::new(), &SpanRegistry::new(), limits).unwrap();
    assert_eq!(snapshot.view, None);
    let before = snapshot.to_json(limits).unwrap();
    snapshot.view = Some(
        InspectorView::new(
            2,
            3,
            8,
            fmn_render::Viewport {
                width: 96,
                height: 54,
            },
            fmn_render::ScreenMap {
                scale: 6.75,
                origin: [48.0, 27.0],
                y_up: true,
            },
            false,
        )
        .unwrap(),
    );
    let json = snapshot.to_json(limits).unwrap();
    assert_eq!(snapshot.version, 1);
    assert_eq!(
        json,
        br#"{"version":1,"scene_time":0,"truncated":false,"nodes":[],"view":{"frame_index":2,"frame_count":3,"fps":8,"width":96,"height":54,"scale":6.75,"origin":[48,27],"y_up":true,"input_events":false}}"#
    );
    assert_eq!(
        snapshot
            .to_json(InspectorLimits {
                max_json_bytes: json.len(),
                ..limits
            })
            .unwrap(),
        json
    );
    assert!(matches!(
        snapshot.to_json(InspectorLimits { max_json_bytes: json.len() - 1, ..limits }),
        Err(InspectError::JsonLimit { limit, needed }) if limit == json.len() - 1 && needed == json.len()
    ));
    // Native workers also accept raw pixel-oriented maps. The wire view must
    // distinguish them, rather than making the UI guess from scene identity.
    let map = fmn_render::ScreenMap {
        scale: 6.75,
        origin: [48.0, 27.0],
        y_up: false,
    };
    snapshot.view = Some(
        InspectorView::new(
            2,
            3,
            8,
            fmn_render::Viewport {
                width: 96,
                height: 54,
            },
            map,
            false,
        )
        .unwrap(),
    );
    assert_eq!(
        snapshot.to_json(limits).unwrap(),
        br#"{"version":1,"scene_time":0,"truncated":false,"nodes":[],"view":{"frame_index":2,"frame_count":3,"fps":8,"width":96,"height":54,"scale":6.75,"origin":[48,27],"y_up":false,"input_events":false}}"#
    );
    snapshot.view = None;
    assert_eq!(snapshot.to_json(limits).unwrap(), before);
}

#[test]
fn inspector_view_refuses_malformed_range_geometry_and_maps() {
    let viewport = fmn_render::Viewport {
        width: 96,
        height: 54,
    };
    let map = fmn_render::ScreenMap {
        scale: 6.75,
        origin: [48.0, 27.0],
        y_up: true,
    };
    for (index, count, fps) in [(0, 0, 8), (3, 3, 8), (0, u64::MAX, 8), (0, 3, 0)] {
        assert!(matches!(
            InspectorView::new(index, count, fps, viewport, map, false),
            Err(InspectError::InvalidView(_))
        ));
    }
    for viewport in [
        fmn_render::Viewport {
            width: 0,
            height: 54,
        },
        fmn_render::Viewport {
            width: 96,
            height: 0,
        },
        fmn_render::Viewport {
            width: u32::MAX,
            height: u32::MAX,
        },
    ] {
        assert!(matches!(
            InspectorView::new(0, 3, 8, viewport, map, false),
            Err(InspectError::InvalidView(_))
        ));
    }
    for scale in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(matches!(
            InspectorView::new(
                0,
                3,
                8,
                viewport,
                fmn_render::ScreenMap { scale, ..map },
                false
            ),
            Err(InspectError::InvalidView(_))
        ));
    }
    for origin in [
        [f64::NAN, 0.0],
        [0.0, f64::INFINITY],
        [f64::NEG_INFINITY, 0.0],
    ] {
        assert!(matches!(
            InspectorView::new(
                0,
                3,
                8,
                viewport,
                fmn_render::ScreenMap { origin, ..map },
                false
            ),
            Err(InspectError::InvalidView(_))
        ));
    }
    let mut snapshot = InspectorSnapshot::capture(
        &Stage::new(),
        &SpanRegistry::new(),
        InspectorLimits::default(),
    )
    .unwrap();
    let mut view = InspectorView::new(0, 3, 8, viewport, map, false).unwrap();
    view.frame_index = view.frame_count;
    snapshot.view = Some(view);
    assert!(matches!(
        snapshot.to_json(InspectorLimits::default()),
        Err(InspectError::InvalidView(_))
    ));
    view.frame_index = 0;
    view.scale = f64::NAN;
    snapshot.view = Some(view);
    assert!(matches!(
        snapshot.to_json(InspectorLimits::default()),
        Err(InspectError::InvalidView(_))
    ));
}

#[test]
fn source_span_registration_is_atomic_across_late_map_errors() {
    let mut stage = Stage::new();
    let root = stage.add(Mobject::from_points(&[[0.0, 0.0, 0.0]]));
    stage.add_to_scene(root).unwrap();
    let mut spans = SpanRegistry::new();
    spans
        .register(root, Arc::from("old"), 0, 3, SpanKind::TextGlyph)
        .unwrap();

    let error = spans
        .bind_native(
            Arc::from("new"),
            &[root],
            &[
                NativeSpanBinding {
                    submobject_index: 0,
                    start: 0,
                    end: 3,
                    kind: SpanKind::MathGlyph,
                },
                NativeSpanBinding {
                    submobject_index: 1,
                    start: 0,
                    end: 3,
                    kind: SpanKind::MathPath,
                },
            ],
        )
        .unwrap_err();
    assert!(matches!(error, InspectError::SpanMapMismatch(_)));

    let before = InspectorSnapshot::capture(&stage, &spans, InspectorLimits::default()).unwrap();
    let prior = before.nodes[0].source_span.as_ref().unwrap();
    assert_eq!(prior.excerpt, "old");
    assert_eq!(prior.kind, SpanKind::TextGlyph);

    spans
        .bind_native(
            Arc::from("new"),
            &[root],
            &[NativeSpanBinding {
                submobject_index: 0,
                start: 0,
                end: 3,
                kind: SpanKind::MathGlyph,
            }],
        )
        .unwrap();
    let after = InspectorSnapshot::capture(&stage, &spans, InspectorLimits::default()).unwrap();
    let replacement = after.nodes[0].source_span.as_ref().unwrap();
    assert_eq!(replacement.excerpt, "new");
    assert_eq!(replacement.kind, SpanKind::MathGlyph);
}

#[test]
fn inspector_and_debug_overlays_follow_visible_family_order() {
    let first = Mobject::from_points(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
    let second = Mobject::from_points(&[[2.0, 2.0, 1.0]]);
    let mut stage = Stage::new();
    let root = stage.add(Mobject::group(vec![first, second]));
    stage.add_to_scene(root).unwrap();
    let children = stage.get(root).unwrap().submobjects().to_vec();

    let mut spans = SpanRegistry::new();
    spans
        .bind_native(
            Arc::from("α + β"),
            &children,
            &[NativeSpanBinding {
                submobject_index: 0,
                start: 0,
                end: "α".len(),
                kind: SpanKind::TextGlyph,
            }],
        )
        .unwrap();
    let limits = InspectorLimits::default();
    let inspection = InspectorSnapshot::capture(&stage, &spans, limits).unwrap();
    assert_eq!(inspection.nodes.len(), 3);
    assert_eq!(inspection.nodes[0].children, vec![1, 2]);
    assert_eq!(
        inspection.nodes[1].source_span.as_ref().unwrap().excerpt,
        "α"
    );
    let json = inspection.to_json(limits).unwrap();
    assert!(json.starts_with(b"{\"version\":1"));
    assert!(std::str::from_utf8(&json).unwrap().contains("\"point\""));

    let overlays = DebugOverlaySnapshot::capture(
        &stage,
        None,
        DebugLayerSet::CONTROL_POINTS
            | DebugLayerSet::BOUNDING_BOXES
            | DebugLayerSet::WINDING
            | DebugLayerSet::DEPTH,
        limits,
    )
    .unwrap();
    assert_eq!(overlays.nodes.len(), 3);
    assert_eq!(
        overlays.nodes[1].winding,
        Some(fmn_studio::WindingDirection::CounterClockwise)
    );
    assert_eq!(overlays.nodes[2].center_z, Some(1.0));
    assert!(!overlays.to_json(limits).unwrap().is_empty());

    let bounded = InspectorLimits {
        max_links_per_node: 1,
        max_total_links: 1,
        max_points_per_node: 1,
        max_total_points: 1,
        ..limits
    };
    let bounded_inspection = InspectorSnapshot::capture(&stage, &spans, bounded).unwrap();
    assert!(bounded_inspection.truncated);
    assert_eq!(bounded_inspection.nodes[0].children, vec![1]);
    let bounded_overlays =
        DebugOverlaySnapshot::capture(&stage, None, DebugLayerSet::WINDING, bounded).unwrap();
    assert!(bounded_overlays.truncated);
    assert_eq!(bounded_overlays.nodes[1].winding, None);
}

#[test]
fn inspector_bounds_traversal_work_across_seen_shared_edges() {
    let mut stage = Stage::new();
    let root = stage.add(Mobject::new());
    let left = stage.add(Mobject::new());
    let right = stage.add(Mobject::new());
    let shared = stage.add(Mobject::new());
    stage.attach(root, left).unwrap();
    stage.attach(left, shared).unwrap();
    stage.attach(root, right).unwrap();
    stage.attach(right, shared).unwrap();
    stage.add_to_scene(root).unwrap();

    let exact_limits = InspectorLimits {
        max_traversal_edges: 4,
        ..InspectorLimits::default()
    };
    let exact = InspectorSnapshot::capture(&stage, &SpanRegistry::new(), exact_limits).unwrap();
    assert_eq!(exact.nodes.len(), 4);
    assert!(!exact.truncated);
    let exact_overlay =
        DebugOverlaySnapshot::capture(&stage, None, DebugLayerSet::CONTROL_POINTS, exact_limits)
            .unwrap();
    assert_eq!(exact_overlay.nodes.len(), 4);
    assert!(!exact_overlay.truncated);

    let bounded_limits = InspectorLimits {
        max_traversal_edges: 3,
        ..exact_limits
    };
    let bounded = InspectorSnapshot::capture(&stage, &SpanRegistry::new(), bounded_limits).unwrap();
    assert_eq!(bounded.nodes.len(), 4);
    assert!(bounded.truncated);
    let bounded_overlay =
        DebugOverlaySnapshot::capture(&stage, None, DebugLayerSet::CONTROL_POINTS, bounded_limits)
            .unwrap();
    assert_eq!(bounded_overlay.nodes.len(), 4);
    assert!(bounded_overlay.truncated);
}

#[test]
fn inspector_bounds_source_excerpts_across_the_snapshot() {
    let first = Mobject::from_points(&[[0.0, 0.0, 0.0]]);
    let second = Mobject::from_points(&[[1.0, 0.0, 0.0]]);
    let third = Mobject::from_points(&[[2.0, 0.0, 0.0]]);
    let mut stage = Stage::new();
    let root = stage.add(Mobject::group(vec![first, second, third]));
    stage.add_to_scene(root).unwrap();
    let children = stage.get(root).unwrap().submobjects().to_vec();

    let source: Arc<str> = Arc::from("abcdα");
    let bindings = children
        .iter()
        .enumerate()
        .map(|(submobject_index, _)| NativeSpanBinding {
            submobject_index,
            start: 0,
            end: source.len(),
            kind: SpanKind::TextGlyph,
        })
        .collect::<Vec<_>>();
    let mut spans = SpanRegistry::new();
    spans.bind_native(source, &children, &bindings).unwrap();

    let limits = InspectorLimits {
        max_source_excerpt_bytes: 5,
        max_total_source_excerpt_bytes: 5,
        ..InspectorLimits::default()
    };
    let inspection = InspectorSnapshot::capture(&stage, &spans, limits).unwrap();
    let source_spans = inspection
        .nodes
        .iter()
        .filter_map(|node| node.source_span.as_ref())
        .collect::<Vec<_>>();

    assert_eq!(source_spans.len(), 3);
    assert_eq!(source_spans[0].excerpt, "abcd");
    assert_eq!(source_spans[1].excerpt, "a");
    assert_eq!(source_spans[2].excerpt, "");
    assert_eq!(
        source_spans
            .iter()
            .map(|span| span.excerpt.len())
            .sum::<usize>(),
        limits.max_total_source_excerpt_bytes
    );
    assert!(source_spans.iter().all(|span| span.start == 0));
    assert!(source_spans.iter().all(|span| span.end == "abcdα".len()));
    assert!(
        source_spans
            .iter()
            .all(|span| span.source_bytes == "abcdα".len())
    );
    assert!(source_spans.iter().all(|span| span.excerpt_truncated));
    assert!(inspection.truncated);
}

#[test]
fn overlay_json_budget_stops_during_tile_encoding() {
    let limits = InspectorLimits {
        max_json_bytes: 256,
        ..InspectorLimits::default()
    };
    let overrun = |tile_count| {
        let overlays = DebugOverlaySnapshot {
            version: 1,
            layers: DebugLayerSet::TILES,
            tiles: (0..tile_count)
                .map(|index| TileOverlay {
                    index,
                    rect: [0, 0, 1, 1],
                    draws: 1,
                    partial: 1,
                    interior: 0,
                })
                .collect(),
            nodes: Vec::new(),
            truncated: false,
        };
        match overlays.to_json(limits).unwrap_err() {
            InspectError::JsonLimit { limit, needed } => (limit, needed),
            error => std::panic::panic_any(format!("unexpected overlay encoding error: {error}")),
        }
    };

    let first_overrun = overrun(8);
    assert_eq!(first_overrun.0, 256);
    assert!(first_overrun.1 > first_overrun.0);
    assert_eq!(first_overrun, overrun(1_000));
}

#[test]
fn inspector_json_budget_stops_within_record_values() {
    let mut stage = Stage::new();
    let root = stage.add(Mobject::from_points(&[[0.0, 0.0, 0.0]]));
    stage.add_to_scene(root).unwrap();
    let limits = InspectorLimits {
        max_json_bytes: 4 * 1024,
        ..InspectorLimits::default()
    };
    let template = InspectorSnapshot::capture(&stage, &SpanRegistry::new(), limits).unwrap();
    assert!(template.to_json(limits).is_ok());
    let overrun = |value_count| {
        let mut inspection = template.clone();
        let field = &mut inspection.nodes[0].fields[0];
        field.values = vec![f32::MAX; value_count];
        field.total_values = value_count;
        match inspection.to_json(limits).unwrap_err() {
            InspectError::JsonLimit { limit, needed } => (limit, needed),
            error => std::panic::panic_any(format!("unexpected inspector encoding error: {error}")),
        }
    };

    let first_overrun = overrun(256);
    assert_eq!(first_overrun.0, limits.max_json_bytes);
    assert!(first_overrun.1 > first_overrun.0);
    let trailing_overrun = overrun(1_000);
    assert_eq!(first_overrun.0, trailing_overrun.0);
    assert_eq!(
        first_overrun.1.abs_diff(trailing_overrun.1),
        256_usize
            .to_string()
            .len()
            .abs_diff(1_000_usize.to_string().len())
    );
}

#[test]
fn overlay_json_budget_stops_within_control_points() {
    let limits = InspectorLimits {
        max_json_bytes: 4 * 1024,
        ..InspectorLimits::default()
    };
    let snapshot = |point_count| DebugOverlaySnapshot {
        version: 1,
        layers: DebugLayerSet::CONTROL_POINTS,
        tiles: Vec::new(),
        nodes: vec![NodeOverlay {
            id: 0,
            control_points: vec![[f64::MAX; 3]; point_count],
            total_points: point_count,
            bounds: None,
            winding: None,
            center_z: None,
            z_index: None,
            depth_test: None,
        }],
        truncated: false,
    };
    assert!(snapshot(1).to_json(limits).is_ok());
    let overrun = |point_count| match snapshot(point_count).to_json(limits).unwrap_err() {
        InspectError::JsonLimit { limit, needed } => (limit, needed),
        error => std::panic::panic_any(format!("unexpected overlay encoding error: {error}")),
    };

    let first_overrun = overrun(64);
    assert_eq!(first_overrun.0, limits.max_json_bytes);
    assert!(first_overrun.1 > first_overrun.0);
    let trailing_overrun = overrun(1_000);
    assert_eq!(first_overrun.0, trailing_overrun.0);
    assert_eq!(
        first_overrun.1.abs_diff(trailing_overrun.1),
        64_usize
            .to_string()
            .len()
            .abs_diff(1_000_usize.to_string().len())
    );
}

#[test]
fn timeline_preview_restores_live_stage_while_commit_leaves_target() {
    let mut timeline = Timeline::new(10).unwrap();
    timeline.wait(1.0).unwrap();
    let rng = RngRoot::from_seed(7);
    let mut stage = Stage::new();

    let preview = preview_timeline_frame(&mut timeline, &mut stage, &rng, 5).unwrap();
    assert_eq!(preview.mode(), ScrubMode::Preview);
    assert_eq!(preview.packet().frame_index(), 5);
    assert_eq!(stage.time(), 0.0);
    assert_eq!(preview.checkpointed_segments(), &[0]);

    let committed = commit_timeline_frame(&mut timeline, &mut stage, &rng, 5).unwrap();
    assert_eq!(committed.mode(), ScrubMode::Commit);
    assert_eq!(stage.time(), 0.5);
}

#[test]
fn terminal_adapters_emit_bounded_protocol_bytes() {
    let rgba = [255, 0, 0, 255, 0, 255, 0, 255];
    for (protocol, prefix) in [
        (TerminalProtocol::Kitty, b"\x1b_G".as_slice()),
        (TerminalProtocol::Sixel, b"\x1bP0;0;0q".as_slice()),
    ] {
        let mut out = Vec::new();
        TerminalPreview::new(protocol, TuiLimits::default())
            .unwrap()
            .write_rgba8(&mut out, 2, 1, &rgba)
            .unwrap();
        assert!(out.starts_with(prefix));
        assert!(out.ends_with(b"\x1b\\"));
    }
}

#[test]
fn preencoded_kitty_png_is_validated_under_the_declared_pixel_budget() {
    let rgba = [255, 0, 0, 255, 0, 255, 0, 255];
    let png = encode_rgba8(2, 1, &rgba, CompressionLevel::Fast);

    let exact = TerminalPreview::new(
        TerminalProtocol::Kitty,
        TuiLimits {
            max_pixels: 2,
            ..TuiLimits::default()
        },
    )
    .unwrap();
    let mut exact_output = Vec::new();
    exact.write_png(&mut exact_output, &png).unwrap();
    assert!(exact_output.starts_with(b"\x1b_G"));

    let over_limit = TerminalPreview::new(
        TerminalProtocol::Kitty,
        TuiLimits {
            max_pixels: 1,
            ..TuiLimits::default()
        },
    )
    .unwrap();
    let mut refused_output = Vec::new();
    let error = over_limit
        .write_png(&mut refused_output, &png)
        .expect_err("two pixels must exceed the one-pixel ceiling");
    assert!(error.to_string().contains("1-pixel budget"));
    assert!(refused_output.is_empty());

    let mut corrupt = png.clone();
    let last = corrupt.last_mut().expect("encoded PNG is nonempty");
    *last ^= 1;
    let error = exact
        .write_png(&mut refused_output, &corrupt)
        .expect_err("a corrupt PNG must not reach the terminal");
    assert!(error.to_string().contains("crc"));
    assert!(refused_output.is_empty());

    let size_first = TerminalPreview::new(
        TerminalProtocol::Kitty,
        TuiLimits {
            max_encoded_bytes: 1,
            ..TuiLimits::default()
        },
    )
    .unwrap();
    assert!(matches!(
        size_first.write_png(&mut refused_output, b"\x89PNG\r\n\x1a\n"),
        Err(TuiError::EncodedLimit { limit: 1, .. })
    ));
    assert!(refused_output.is_empty());
}

#[test]
fn kitty_admits_the_exact_actual_encoded_byte_boundary() {
    let rgba = [255, 0, 0, 255, 0, 255, 0, 255];
    let png = encode_rgba8(2, 1, &rgba, CompressionLevel::Fast);

    for kitty_chunk_bytes in [4096, 4] {
        let generous = TerminalPreview::new(
            TerminalProtocol::Kitty,
            TuiLimits {
                max_pixels: 2,
                kitty_chunk_bytes,
                ..TuiLimits::default()
            },
        )
        .unwrap();
        let mut expected = Vec::new();
        generous.write_png(&mut expected, &png).unwrap();

        let exact = TerminalPreview::new(
            TerminalProtocol::Kitty,
            TuiLimits {
                max_pixels: 2,
                max_encoded_bytes: expected.len(),
                kitty_chunk_bytes,
            },
        )
        .unwrap();
        let mut actual = Vec::new();
        exact
            .write_png(&mut actual, &png)
            .expect("the complete Kitty record fits the exact byte ceiling");
        assert_eq!(actual, expected);

        let one_byte_short = TerminalPreview::new(
            TerminalProtocol::Kitty,
            TuiLimits {
                max_pixels: 2,
                max_encoded_bytes: expected.len() - 1,
                kitty_chunk_bytes,
            },
        )
        .unwrap();
        let mut refused = Vec::new();
        assert!(matches!(
            one_byte_short.write_png(&mut refused, &png),
            Err(TuiError::EncodedLimit { limit, needed })
                if limit == expected.len() - 1 && needed == expected.len()
        ));
        assert!(refused.is_empty());
    }
}

#[cfg(feature = "metal")]
#[test]
fn studio_metal_feature_uses_the_truthful_preview_selector() {
    let renderer = fmn_studio::PreviewRenderer::new().unwrap();
    #[cfg(not(target_os = "macos"))]
    assert!(matches!(
        renderer,
        fmn_studio::PreviewRenderer::FastCpu(fmn_studio::PreviewFallback::Unavailable)
    ));
    #[cfg(target_os = "macos")]
    assert!(matches!(
        renderer,
        fmn_studio::PreviewRenderer::Metal(_)
            | fmn_studio::PreviewRenderer::FastCpu(fmn_studio::PreviewFallback::Unavailable)
    ));
}

#[cfg(all(feature = "metal", not(target_os = "macos")))]
#[test]
fn native_preview_unavailability_uses_the_cpu_visible_studio_stream() {
    let mut renderer = fmn_studio::StudioPreviewRenderer::new(
        fmn_studio::StudioPreviewConfig::new(16, 16, "fallback", 1),
    )
    .unwrap();
    assert_eq!(
        renderer.route(),
        fmn_studio::StudioPreviewRoute::CpuStream(fmn_studio::PreviewFallback::Unavailable)
    );
    assert!(renderer.presentation_pipeline_info().is_none());
    assert_eq!(renderer.poll_events().unwrap(), None);

    let stage = Stage::new();
    let config = fmn_render::FrameConfig::new(
        fmn_render::Viewport {
            width: 16,
            height: 16,
        },
        fmn_render::ScreenMap {
            scale: 1.0,
            origin: [8.0, 8.0],
            y_up: true,
        },
        fmn_core::color::Srgb::from_rgb8(12, 18, 24).to_linear(1.0),
    );
    let mut plan = fmn_render::RenderPlan::new();
    plan.sync(&stage, 0).expect("valid Studio preview fixture");
    let mono = fmn_render::MonoTable::build(&plan, config.map)
        .expect("bounded Studio preview monotone table");
    let binning = fmn_render::Binning::build(
        &plan,
        config.viewport,
        fmn_render::Tiling::default(),
        config.map,
    )
    .expect("bounded Studio test binning");
    let output = renderer.render(&plan, &mono, &binning, config).unwrap();
    let frame = output
        .into_stream()
        .expect("off-macOS Studio preview returns CPU-visible bytes");
    assert_eq!(
        frame.route,
        fmn_studio::PreviewRoute::FastCpu(fmn_studio::PreviewFallback::Unavailable)
    );
    assert_eq!(frame.frame.as_bytes().len(), 16 * 16 * 4);
    assert!(frame.metal.is_none());
    renderer.close().unwrap();
}

#[cfg(unix)]
#[test]
fn kitty_preview_crosses_a_real_pseudoterminal() {
    use std::fs::{File, OpenOptions};
    use std::io::Read;

    use rustix::pty::{OpenptFlags, grantpt, openpt, ptsname, unlockpt};

    let master = openpt(OpenptFlags::RDWR).unwrap();
    grantpt(&master).unwrap();
    unlockpt(&master).unwrap();
    let slave_name = ptsname(&master, Vec::new()).unwrap();
    let mut slave = OpenOptions::new()
        .read(true)
        .write(true)
        .open(slave_name.to_str().unwrap())
        .unwrap();
    let mut master = File::from(master);

    let preview = TerminalPreview::new(TerminalProtocol::Kitty, TuiLimits::default()).unwrap();
    let rgba = [12, 34, 56, 255];
    let mut expected = Vec::new();
    preview.write_rgba8(&mut expected, 1, 1, &rgba).unwrap();
    preview.write_rgba8(&mut slave, 1, 1, &rgba).unwrap();

    let mut observed = vec![0; expected.len()];
    master.read_exact(&mut observed).unwrap();
    assert_eq!(observed, expected);
}
