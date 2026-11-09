//! The first live self-goldens (§16.3 plane 2, D-16, fm-xb3): geometry
//! snapshots at lifecycle points, bit-locked per platform.
//!
//! Two artifacts are locked in `goldens/self_goldens.<platform>.lock`:
//!
//! - `geom_lifecycle.v1` — a QuadPath driven through its construction
//!   lifecycle (arc → line → smooth curve → close), with the full point run
//!   snapshotted after each step;
//! - `stage_lifecycle.v1` — a three-mobject Stage family driven through the
//!   positional API (attach → next_to → arrange → scale → to_edge), with
//!   every member's f32 records and the root bounding box snapshotted;
//! - `stream_lines.v1` (fm-9esi) — StreamLines over a polynomial limit-cycle
//!   field, every line's f64 points. Before frankenscipy `5a7aafa2` the RK45
//!   step control went through the platform `powf`, so these lines could not
//!   carry a cross-platform lock. Measured 2026-09-28 at frankenscipy
//!   `85edd7a9`: blessed on linux-x86_64 (glibc, debug) as `48d9c4cf…`
//!   (12,012 bytes). It passes unchanged on macos-aarch64 (Darwin, M4 Pro,
//!   native, `--release`) and on linux-aarch64 (static musl ELF cross-compiled
//!   with `--release`, run under binfmt/qemu on the dev box). One lock, three
//!   certified platforms, two profiles.
//!
//! Snapshots are serialized through fmn-hash's canonical Writer (versioned
//! schema, defined field order, float canonicalization, trailing checksum),
//! so the locked bytes are the §6.7 durable form, not a Debug dump.
//!
//! **Graduated to [`Scope::Certified`] on 2026-07-26 (fm-ig3), by measurement.**
//! This file used to say that cross-platform convergence *would* graduate these
//! locks once the certified arithmetic landed. It has, and both artifacts were
//! then rendered on all three certified platforms:
//!
//! | Platform | `geom_lifecycle.v1` | `stage_lifecycle.v1` |
//! |---|---|---|
//! | linux-x86_64 (glibc, native) | `f0e73e89…` | `958ff777…` |
//! | linux-aarch64 (musl, qemu-user) | `f0e73e89…` | `958ff777…` |
//! | macos-aarch64 (Darwin, M4 Pro, native) | `f0e73e89…` | `958ff777…` |
//!
//! The stage lock was re-measured on all three legs on 2026-07-29 (fm-7if)
//! after positional operations moved to a composed `f64` placement. Its
//! square and triangle records stayed byte-identical; the bar now rounds once
//! to the nearest final `f32` instead of accumulating intermediate record
//! writes, and the `f64` family bounds move correspondingly closer to the
//! analytic values.
//!
//! One lock now speaks for the matrix, which is the stronger arrangement for the
//! reason `docs/INPUT_CLOSURE.md` §6 gives: a per-platform lock passes everywhere
//! and waits for someone to re-run a sweep, while a shared one **fails on
//! whichever machine breaks it**. (`self_goldens.linux-x86_64.lock` is superseded
//! and no longer read.)
//!
//! Drift fails here — this is the merge blocker. Deliberate changes re-bless
//! with `UPDATE_GOLDENS=1 cargo test -p fmn-conformance --test self_goldens`
//! and commit the lock diff (the rig never commits; frame hashes join these
//! artifacts once Lumen exists).

use fmn_conformance::golden::{GoldenError, GoldenStore, Scope};
use fmn_core::constants::{DOWN, LEFT, RIGHT, TAU, UP};
use fmn_core::rng::RngRoot;
use fmn_core::types::Vec3;
use fmn_geom::QuadPath;
use fmn_hash::{Schema, Writer};
use fmn_library::coords::CoordinateSystem;
use fmn_library::fields::StreamLines;
use fmn_mobject::{Mob, Mobject, Stage};
use std::path::PathBuf;

/// Schema family for self-golden snapshot documents.
const GEOM_SCHEMA: Schema = Schema::new(*b"FMNS", 1, 1, 0);
const STAGE_SCHEMA: Schema = Schema::new(*b"FMNS", 2, 1, 0);
const STREAM_SCHEMA: Schema = Schema::new(*b"FMNS", 3, 1, 0);

fn store() -> GoldenStore {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("goldens");
    GoldenStore::new(dir, "self_goldens", Scope::Certified).expect("store")
}

/// Append a labeled point run to the document: label, count, then x/y/z f64s.
fn put_points_f64(w: &mut Writer, label: &str, points: &[Vec3]) {
    w.put_str(label).put_u64(points.len() as u64);
    for p in points {
        for &c in p {
            w.put_f64(c);
        }
    }
}

/// The QuadPath lifecycle document: point snapshots after each construction
/// step, under the original Reference op names (§7).
fn geom_lifecycle_doc() -> Vec<u8> {
    let mut w = Writer::new(GEOM_SCHEMA);

    // Step 1: a quarter arc of radius 1.5 centered off-origin.
    let mut path = QuadPath::try_arc(0.0, TAU / 4.0, 1.5, [0.5, -0.25, 0.0], Some(4))
        .expect("the lifecycle fixture arc is valid");
    put_points_f64(&mut w, "arc", path.points());

    // Step 2: a line to a corner point.
    path.add_line_to([2.0, 2.0, 0.0], false).expect("line");
    put_points_f64(&mut w, "line_to", path.points());

    // Step 3: a smooth continuation (reflected-handle rule).
    path.add_smooth_curve_to([-1.0, 2.5, 0.0]).expect("smooth");
    put_points_f64(&mut w, "smooth_curve_to", path.points());

    // Step 4: close the subpath (jagged closure).
    path.close_path(false).expect("close");
    put_points_f64(&mut w, "close_path", path.points());
    w.put_bool(path.is_closed());
    w.put_u64(path.num_curves() as u64);

    w.finish().expect("geometry snapshot encodes")
}

/// Append one mobject's own world-space points at record precision.
fn put_records_f32(w: &mut Writer, label: &str, stage: &Stage, mob: Mob) {
    let points = stage.get_points(mob).unwrap_or_default();
    w.put_str(label).put_u64((points.len() * 3) as u64);
    for point in points {
        for component in point {
            #[allow(clippy::cast_possible_truncation)]
            w.put_f32(component as f32);
        }
    }
}

/// The Stage lifecycle document: a family driven through the positional API,
/// with every member's records and the root bbox snapshotted at the end.
fn stage_lifecycle_doc() -> Vec<u8> {
    let mut w = Writer::new(STAGE_SCHEMA);
    let mut stage = Stage::new();

    // A unit square, a right triangle, and a wide bar.
    let square = stage.add(Mobject::from_points(&[
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
    ]));
    let triangle = stage.add(Mobject::from_points(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    ]));
    let bar = stage.add(Mobject::from_points(&[
        [-1.5, -0.1, 0.0],
        [1.5, -0.1, 0.0],
        [1.5, 0.1, 0.0],
        [-1.5, 0.1, 0.0],
    ]));
    let root = stage.add(Mobject::new());
    for child in [square, triangle, bar] {
        stage.attach(root, child).expect("attach");
    }

    // The positional lifecycle: relative placement, arrangement, scaling,
    // and a frame-edge alignment — each step visible in the final records.
    stage.next_to(triangle, square, RIGHT, 0.25, DOWN);
    stage.arrange(root, RIGHT, 0.5, true);
    stage.scale(root, 1.25);
    stage.to_edge(root, UP, 0.8);
    stage.next_to(bar, triangle, DOWN, 0.3, LEFT);

    for (label, mob) in [("square", square), ("triangle", triangle), ("bar", bar)] {
        put_records_f32(&mut w, label, &stage, mob);
    }

    let bb = stage.get_bounding_box(root);
    for corner in [bb.min, bb.mid, bb.max] {
        for &c in &corner {
            w.put_f64(c);
        }
    }

    w.finish().expect("stage snapshot encodes")
}

/// A 2-D plane over `[-2, 2]²` with unit sampling steps.
struct Plane;

impl CoordinateSystem for Plane {
    fn c2p(&self, coords: &[f64]) -> Vec3 {
        [
            coords.first().copied().unwrap_or(0.0),
            coords.get(1).copied().unwrap_or(0.0),
            0.0,
        ]
    }
    fn p2c(&self, point: Vec3) -> Vec3 {
        point
    }
    fn all_ranges(&self) -> Vec<[f64; 3]> {
        vec![[-2.0, 2.0, 1.0], [-2.0, 2.0, 1.0]]
    }
    fn dimension(&self) -> usize {
        2
    }
}

/// A polynomial limit-cycle field: rotation plus a radial pull toward the
/// unit circle. No transcendental, so every bit of the lines comes from the
/// integrator: adaptive RK45 step control (frankenscipy's `kth_root`, fm-9esi),
/// dense output, and the true-arclength cap.
fn limit_cycle(rows: &[[f64; 3]]) -> Vec<[f64; 3]> {
    rows.iter()
        .map(|&[x, y, _]| {
            let pull = 0.3 * (1.0 - x * x - y * y);
            [-y + pull * x, x + pull * y, 0.0]
        })
        .collect()
}

/// The StreamLines document: every line's points, in family order, plus the
/// seed-jitter draw count.
fn stream_lines_doc() -> Vec<u8> {
    let mut w = Writer::new(STREAM_SCHEMA);
    let built = StreamLines::new(limit_cycle, Plane, &RngRoot::from_seed(7))
        .with_noise_factor(0.25)
        .build()
        .expect("the fixture field builds");
    w.put_u64(built.rng_draws());
    let lines = built.vmob().children();
    w.put_u64(lines.len() as u64);
    for (index, line) in lines.iter().enumerate() {
        put_points_f64(&mut w, &format!("line{index}"), line.points());
    }
    w.finish().expect("stream-lines snapshot encodes")
}

#[test]
fn geom_lifecycle_is_bit_locked() -> Result<(), GoldenError> {
    let doc = geom_lifecycle_doc();
    store().check("geom_lifecycle.v1", &doc)?;
    Ok(())
}

#[test]
fn stage_lifecycle_is_bit_locked() -> Result<(), GoldenError> {
    let doc = stage_lifecycle_doc();
    store().check("stage_lifecycle.v1", &doc)?;
    Ok(())
}

#[test]
fn stream_lines_are_bit_locked() -> Result<(), GoldenError> {
    let doc = stream_lines_doc();
    store().check("stream_lines.v1", &doc)?;
    Ok(())
}

#[test]
fn snapshot_documents_are_reproducible_within_run() {
    // The rig's premise: the same engine state serializes to the same bytes.
    // A failure here is nondeterminism in the engine or the encoder, which
    // must be caught before it can masquerade as cross-commit drift.
    assert_eq!(geom_lifecycle_doc(), geom_lifecycle_doc());
    assert_eq!(stage_lifecycle_doc(), stage_lifecycle_doc());
    assert_eq!(stream_lines_doc(), stream_lines_doc());
}
