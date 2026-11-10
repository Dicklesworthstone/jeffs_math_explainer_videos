#!/usr/bin/env python3
"""Bind timeline compatibility to the shared animation reconstruction law."""
from pathlib import Path
pending = {}
def patch(name, old, new):
    text = pending.get(name, Path(name).read_text())
    if new in text:
        return
    if text.count(old) != 1:
        raise SystemExit(f'changed timeline identity source anchor: {name}')
    pending[name] = text.replace(old, new, 1)

if 'pub fn increment_complex_value' not in Path('crates/fmn-mobject/src/animate.rs').read_text():
    raise SystemExit('native parameter animation must land before its reconstruction law version')
patch('crates/fmn-anim/src/bundle.rs',
      '/// The `path` tag of [`PathFunc::Straight`] — catalog entry 0.\n',
      '''/// Version of the shared frame reconstruction semantics, independent of the
/// rasterizer. Version 1 was implicit in renderer-only FMTL identities. Version
/// 2 includes typed tracker payloads in interpolation and uses the stable
/// translation-independent affine lift. Bump on any replay-law change that can
/// alter state or bits, not on bit-preserving implementation refactors.
pub const RECONSTRUCTION_LAW_VERSION: u32 = 2;

/// The `path` tag of [`PathFunc::Straight`] — catalog entry 0.
''')
patch('crates/fmn-scene/src/timeline_bundle.rs',
      '''pub fn bundle_engine_version() -> String {
    EngineIdentity::certified().closure_string()
}
''',
      '''pub fn bundle_engine_version() -> String {
    format!(
        "{}:fmtl-law:{}",
        EngineIdentity::certified().closure_string(),
        fmn_anim::bundle::RECONSTRUCTION_LAW_VERSION,
    )
}
''')
patch('crates/fmn-scene/src/timeline_bundle.rs',
      '''//! (record columns, placements, numeric uniforms), compared after the
''', '''//! (record columns, placements, typed trackers, numeric uniforms), compared after the
''')
patch('crates/fmn-scene/src/timeline_bundle.rs',
      '''//! [`EngineIdentity::certified`]'s closure string — the same identity the
//! certified input closure journals. The bundle's content is front-end
''',
      '''//! [`EngineIdentity::certified`]'s closure string followed by the shared
//! animation reconstruction-law version. A rasterizer version alone cannot
//! identify changes to parameter interpolation or affine motion. Legacy
//! renderer-only identities are refused: re-export their source timelines.
//! The bundle's content is front-end
''')
patch('crates/fmn-scene/src/timeline_bundle.rs',
      '''/// [`EngineIdentity::certified`]'s canonical closure string (see the
/// module docs for why the certified identity, not the build's fast tier).
''',
      '''/// [`EngineIdentity::certified`]'s canonical closure string plus the
/// shared animation reconstruction-law version (see the module docs).
''')
patch('crates/fmn-anim/src/bundle.rs',
      '/// uniforms. Compared on plain values — both sides are expected to come\n',
      '/// uniforms and typed tracker payloads. Compared on plain values — both sides are expected to come\n')
patch('crates/fmn-render/src/engine.rs',
      '''    /// FMTL/1 records this as its `engine_version` (fm-oee); the timeline
    /// player refuses a bundle whose recorded string differs from its own.
''',
      '''    /// FMTL/1 combines this with Choreo's reconstruction-law version in
    /// its `engine_version`; its player refuses either identity mismatch.
''')
patch('docs/FMNT1_TIMELINE_BUNDLE.md',
      '''1. `engine_version: string` — the engine identity string the player refuses
   on mismatch (the same identity the certified input closure records).
''',
      '''1. `engine_version: string` — the certified renderer closure followed by
   `:fmtl-law:<version>`, where the version is
   `fmn_anim::bundle::RECONSTRUCTION_LAW_VERSION`. The player refuses either
   component's mismatch before interpreting any later field. Renderer-only
   identities predate law version 2 and are deliberately refused: re-export
   the source timeline with the matching engine. Version 2 includes typed
   tracker payloads and stable translation-independent affine interpolation;
   an older player cannot safely reconstruct these merely because its
   rasterizer version matches. The field's wire type and FMTL/1 framing do
   not change.
''')
patch('docs/FMNT1_TIMELINE_BUNDLE.md',
      '''        through `path`, every other field linear, locked fields skipped,
        computed in f64 and stored at record precision. **Export rule: the
''',
      '''        through `path`, every other field linear, locked fields skipped,
        computed in f64 and stored at record precision. Typed tracker lanes
        also interpolate in f64: scalar/logarithmic/complex encodings retain
        their native meaning. **Export rule: the
''')
new_files = {'crates/fmn-scene/tests/reconstruction_identity.rs': r'''//! Renderer compatibility is not animation-law compatibility.
use fmn_anim::bundle::RECONSTRUCTION_LAW_VERSION;
use fmn_anim::{Timeline, prepare_animation};
use fmn_core::{rate, rng::RngRoot};
use fmn_hash::serial::Writer;
use fmn_mobject::animate::AnimateArgs;
use fmn_mobject::Stage;
use fmn_render::engine::EngineIdentity;
use fmn_scene::{BundleReadError, TIMELINE_BUNDLE_SCHEMA, TimelineBundle, bundle_engine_version, export_timeline_bundle};

#[test]
fn bundle_identity_binds_renderer_and_shared_reconstruction_law() {
    assert_eq!(RECONSTRUCTION_LAW_VERSION, 2);
    assert_eq!(bundle_engine_version(), format!("{}:fmtl-law:2", EngineIdentity::certified().closure_string()));
}

#[test]
fn legacy_and_mismatched_laws_refuse_before_later_fields_are_interpreted() {
    let renderer = EngineIdentity::certified().closure_string();
    for identity in [renderer.clone(), format!("{renderer}:fmtl-law:1"), format!("{renderer}:fmtl-law:3")] {
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
    let builder = tracker.animate().set_anim_args(AnimateArgs {
        rate_func: Some(rate::linear), ..AnimateArgs::default()
    }).unwrap().set_value(9.0).unwrap();
    let animation = prepare_animation(builder, &mut stage).unwrap();
    let mut timeline = Timeline::new(4).unwrap();
    timeline.play(vec![animation]).unwrap();
    let bytes = export_timeline_bundle(timeline, &mut stage, &RngRoot::from_seed(0)).unwrap();
    let bundle = TimelineBundle::from_bytes(&bytes).unwrap();
    assert_eq!(bundle.engine_version(), bundle_engine_version());
    let frame = bundle.stage_at(1).unwrap();
    assert_eq!(frame.tracker_value(frame.roots()[0]), Some(5.0));
}
'''}
for name, text in new_files.items():
    if Path(name).exists() and Path(name).read_text() != text:
        raise SystemExit('new source already exists with different contents: ' + name)
    pending[name] = text
for name, text in pending.items():
    Path(name).write_text(text)
