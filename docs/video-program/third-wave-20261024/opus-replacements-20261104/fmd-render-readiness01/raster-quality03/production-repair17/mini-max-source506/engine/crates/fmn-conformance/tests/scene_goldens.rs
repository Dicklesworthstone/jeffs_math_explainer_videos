//! Self-goldens at scale (§16.3 plane 2, D-16, fm-t1v W10): the ~25-scene
//! primitive-and-feature corpus, bit-locked through the [`golden`] rig.
//!
//! `src/scene_goldens.rs` owns the corpus and the artifact layout (geometry
//! snapshot + certified frame at the post-construct point, then again
//! post-transform); this target owns the renderer — `fmn-frame` is a
//! documented dev-only edge of this crate, so the frame itself is produced
//! here and injected into [`artifact`]. The gate:
//!
//! - every scene's artifact is checked against the shared certified-matrix
//!   lock (`goldens/scene_goldens.certified.lock`) — drift is a hard
//!   failure, and the rig writes the offending bytes to
//!   `goldens/scene_goldens.certified.actual/` for byte-level review;
//! - the certified frames are thread-count invariant at {1, 4} threads, so
//!   scheduling cannot silently change the locked bits — the PG-5 harness
//!   (`scripts/pg5_thread_determinism.sh`) widens that sweep through
//!   `FMN_PG5_THREAD_COUNTS` ({1,4,16} per commit, {1,32,96} weekly);
//! - every artifact is reproducible within a run (build twice, byte-equal).
//!
//! Blessing: `UPDATE_GOLDENS=1 cargo test -p fmn-conformance --test
//! scene_goldens`, review the lock diff, commit it (the rig never commits).
//! GOVERNANCE §5 applies — a drift is a finding to adjudicate, never a
//! number to re-bless.

use fmn_conformance::scene_goldens::{
    EQUIVALENCE_SUBSET, SCENES, TILING, artifact, corpus, frame_config, scene_named, store,
};
use fmn_frame::FrameBuffer;
use fmn_mobject::Stage;
use fmn_render::bin::Binning;
use fmn_render::engine::{EngineIdentity, FrameJob, encode_frame};
use fmn_render::fill::MonoTable;
use fmn_render::plan::RenderPlan;

/// Render a stage through one explicitly journaled engine identity into
/// the raw certified frame layout. The same derivation lives in
/// `tests/engine_equivalence.rs` — keep the two in step so the equivalence
/// lane and the bit-locked lane cannot silently diverge in plan
/// construction.
fn render_frame(stage: &Stage, identity: EngineIdentity, threads: usize) -> FrameBuffer {
    let config = frame_config();
    let mut plan = RenderPlan::new();
    plan.sync(stage, 0).expect("valid scene-golden fixture");
    let mono = MonoTable::build(&plan, config.map).expect("bounded scene-golden monotone table");
    let mut binning = Binning::build(&plan, config.viewport, TILING, config.map)
        .expect("bounded conformance binning");
    binning
        .prune_occluded(&plan)
        .expect("occlusion pruning matches the plan");
    FrameJob::with_identity(&plan, &mono, &binning, config, identity)
        .expect("frame artifacts match the plan")
        .render(threads)
        .expect("the engine renders the frame")
}

/// The certified single-threaded frame of a stage, encoded into its
/// canonical document — the byte form the lock hashes.
fn render_certified_doc(stage: &Stage) -> Vec<u8> {
    let frame = render_frame(stage, EngineIdentity::certified(), 1);
    encode_frame(&frame).expect("the frame encodes into its canonical document")
}

#[test]
fn the_corpus_is_bit_locked_across_the_certified_matrix() {
    let store = store();
    let corpus = corpus();
    let mut failures = Vec::new();
    for (index, case) in SCENES.iter().enumerate() {
        let bytes = artifact(case, corpus, index, &render_certified_doc);
        match store.check(case.name, &bytes) {
            Ok(_) => {}
            Err(error) => failures.push(error.to_string()),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// The thread counts PG-5's determinism sweep renders at, from
/// `FMN_PG5_THREAD_COUNTS` (comma-separated, must start at the 1-thread
/// baseline every other count is compared against). The default is the
/// historical {1, 4} pair so `scripts/check.sh` runs exactly what it
/// always ran; CI's PG-5 lanes set {1,4,16} per commit and {1,32,96}
/// on the weekly high-core cadence (fm-sol).
fn pg5_thread_counts() -> Vec<usize> {
    let Ok(raw) = std::env::var("FMN_PG5_THREAD_COUNTS") else {
        return vec![1, 4];
    };
    let counts: Vec<usize> = raw
        .split(',')
        .map(|field| {
            field
                .trim()
                .parse::<usize>()
                .expect("FMN_PG5_THREAD_COUNTS entries are positive integers")
        })
        .collect();
    assert!(
        counts.first() == Some(&1),
        "FMN_PG5_THREAD_COUNTS must start at the 1-thread baseline, got {raw:?}"
    );
    assert!(
        counts.iter().all(|&n| n >= 1),
        "thread counts must be at least 1, got {raw:?}"
    );
    counts
}

#[test]
fn every_certified_frame_is_thread_count_invariant() {
    let corpus = corpus();
    let counts = pg5_thread_counts();
    for case in SCENES {
        let built = (case.build)(corpus);
        let one = render_frame(&built.stage, EngineIdentity::certified(), 1);
        for &threads in &counts[1..] {
            let parallel = render_frame(&built.stage, EngineIdentity::certified(), threads);
            assert_eq!(
                one.as_bytes(),
                parallel.as_bytes(),
                "{} drifted between 1 and {threads} threads",
                case.name
            );
        }
    }
}

/// A locked frame that is all background proves nothing:
/// `title_underlined.v1` locked one (its title sat outside the viewport), so
/// the frame could not show fm-5wq.54's offset. Every scene's frame shows
/// ink, a pixel the empty stage does not have, at both lifecycle points.
#[test]
fn every_locked_frame_shows_ink() {
    let corpus = corpus();
    let empty = render_frame(&Stage::new(), EngineIdentity::certified(), 1);
    let mut blank = Vec::new();
    for (index, case) in SCENES.iter().enumerate() {
        let frames = std::cell::RefCell::new(Vec::new());
        let _artifact = artifact(case, corpus, index, &|stage| {
            let frame = render_frame(stage, EngineIdentity::certified(), 1);
            frames
                .borrow_mut()
                .push(frame.as_bytes() == empty.as_bytes());
            encode_frame(&frame).expect("the frame encodes into its canonical document")
        });
        for (point, is_blank) in frames.into_inner().into_iter().enumerate() {
            if is_blank {
                blank.push(format!("{} point {point}", case.name));
            }
        }
    }
    assert!(blank.is_empty(), "blank locked frames: {blank:?}");
}

#[test]
fn every_artifact_is_reproducible_within_a_run() {
    let corpus = corpus();
    for (index, case) in SCENES.iter().enumerate() {
        let first = artifact(case, corpus, index, &render_certified_doc);
        let second = artifact(case, corpus, index, &render_certified_doc);
        assert_eq!(first, second, "{} is not reproducible", case.name);
    }
}

#[test]
fn the_corpus_covers_the_landed_class_families() {
    // The corpus is the acceptance surface: it must stay ~25 scenes and the
    // lock names must satisfy the rig's character rules (they are path
    // components).
    assert!(SCENES.len() >= 25, "the corpus shrank to {}", SCENES.len());
    let mut names: Vec<&str> = SCENES.iter().map(|case| case.name).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(before, names.len(), "duplicate scene names");
    for case in SCENES {
        assert!(
            scene_named(case.name).is_some(),
            "{} is not findable",
            case.name
        );
    }
    // The equivalence subset must name real scenes — a typo there silently
    // narrows the engine blocker.
    for &name in EQUIVALENCE_SUBSET {
        assert!(
            scene_named(name).is_some(),
            "equivalence subset names unknown scene {name}"
        );
    }
}

#[test]
fn lock_entries_exist_for_every_scene() {
    // A missing entry is exactly what check mode reports as drift, but this
    // assertion turns "the lock file lost a row" into a pointed message
    // rather than a generic drift dump — except in bless mode, where a
    // missing row is the point.
    if std::env::var("UPDATE_GOLDENS").is_ok_and(|value| value == "1") {
        return;
    }
    let store = store();
    let entries = store.load_entries().expect("the lock file parses");
    for case in SCENES {
        assert!(
            entries.contains_key(case.name),
            "{} has no lock entry; bless with UPDATE_GOLDENS=1 and commit",
            case.name
        );
    }
    // And the reverse: a stale entry for a removed scene means the lock was
    // not re-blessed after a corpus edit.
    for name in entries.keys() {
        assert!(
            scene_named(name).is_some(),
            "stale lock entry {name}; re-bless and commit"
        );
    }
}

/// Each scene's certified post-construct frame as tight RGBA8 — the form a
/// re-bless panel shows a reviewer.
fn review_frame(case: &fmn_conformance::scene_goldens::SceneCase) -> Vec<u8> {
    let built = (case.build)(corpus());
    let frame = render_frame(&built.stage, EngineIdentity::certified(), 1);
    let config = frame_config();
    let mut rgba = FrameBuffer::new(
        fmn_frame::FrameLayout::tight(
            fmn_frame::PixelFormat::Rgba8,
            config.viewport.width,
            config.viewport.height,
        )
        .expect("the review frame layout exists"),
    );
    fmn_frame::convert::rgba16f_to_rgba8(&frame, &mut rgba).expect("certified frames convert");
    rgba.as_bytes().to_vec()
}

/// The re-bless protocol's tooling for this suite (docs/GOVERNANCE.md §4).
///
/// 1. At the old source, `FMN_GOLDEN_FRAMES_DIR=<before>` writes every
///    scene's review frame as `<before>/<scene>.rgba`.
/// 2. At the new source, bless the lock (`UPDATE_GOLDENS=1`), then run this
///    test with `FMN_REBLESS_BEFORE_DIR=<before>`: every scene whose frame
///    changed, or that is new (its frame on both sides), is stacked top to
///    bottom, before | after, through
///    [`side_by_side_png`], and written as the lock's artefact under
///    `REBLESS_ARTEFACT_DIR`, named for the blessed lock's digest.
///    When `<before>` also holds the old `scene_goldens.certified.lock`, a
///    scene whose lock row moved with an unchanged frame (a geometry-only
///    change) joins the panel too, so a lock at a new digest always carries
///    its side-by-side.
///
/// Neither switch set: nothing to do (an ordinary gate run).
#[test]
fn rebless_review_frames_and_panel() {
    use fmn_conformance::golden::{REBLESS_ARTEFACT_DIR, rebless_artefact_name, side_by_side_png};
    if let Ok(dir) = std::env::var("FMN_GOLDEN_FRAMES_DIR") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("frames directory");
        for case in SCENES {
            std::fs::write(dir.join(format!("{}.rgba", case.name)), review_frame(case))
                .expect("review frame written");
        }
    }
    let Ok(before_dir) = std::env::var("FMN_REBLESS_BEFORE_DIR") else {
        return;
    };
    let before_dir = std::path::PathBuf::from(before_dir);
    let lock = "scene_goldens.certified.lock";
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let current_lock = std::fs::read(manifest.join("goldens").join(lock)).expect("lock");
    let before_lock = match std::fs::read_to_string(before_dir.join(lock)) {
        Ok(text) => Some(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => panic!("before lock: {error}"),
    };
    let lock_row = |text: &str, name: &str| {
        text.lines()
            .find(|line| line.split('\t').next() == Some(name))
            .map(str::to_owned)
    };
    let current_text = String::from_utf8(current_lock.clone()).expect("the lock is UTF-8");
    let (mut before, mut after) = (Vec::new(), Vec::new());
    let mut rows = 0u32;
    for case in SCENES {
        let new = review_frame(case);
        // A scene the old source did not have is new: the protocol shows
        // its frame on both sides.
        let (old, added) = match std::fs::read(before_dir.join(format!("{}.rgba", case.name))) {
            Ok(old) => (old, false),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (new.clone(), true),
            Err(error) => panic!("{}: before frame: {error}", case.name),
        };
        let lock_moved = before_lock
            .as_deref()
            .is_some_and(|text| lock_row(text, case.name) != lock_row(&current_text, case.name));
        if added || old != new || lock_moved {
            before.extend_from_slice(&old);
            after.extend_from_slice(&new);
            rows += 1;
        }
    }
    assert!(
        rows > 0,
        "no scene frame or lock row changed; there is nothing to re-bless"
    );
    let config = frame_config();
    let panel = side_by_side_png(
        config.viewport.width,
        config.viewport.height * rows,
        &before,
        &after,
    )
    .expect("same-size before/after stacks");
    let digest = fmn_hash::sha256(&current_lock).to_hex();
    let out = manifest.join("../..").join(REBLESS_ARTEFACT_DIR);
    std::fs::create_dir_all(&out).expect("re-bless artefact directory");
    std::fs::write(out.join(rebless_artefact_name(lock, &digest)), panel).expect("panel written");
}
