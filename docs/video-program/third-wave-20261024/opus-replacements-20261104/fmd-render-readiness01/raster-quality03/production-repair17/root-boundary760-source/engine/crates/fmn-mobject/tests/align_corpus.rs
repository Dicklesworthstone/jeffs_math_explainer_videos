//! The align_data fixture corpus (fm-cye, §9.4): a 1024-pair matrix over
//! (source subpath count × target subpath count × source children ×
//! target children), every pair driven through `align_data_and_family`
//! with STRUCTURAL assertions — family shapes reconciled, per-member
//! record counts equal, shared-anchor validity (odd point runs that
//! reparse), subpath counts equalized, joint angles finite, and each
//! original subpath's endpoint anchors surviving the alignment.
//! Alignment bugs are the classic manim wart; this corpus is the wall
//! against regressions.
//!
//! Generation is fully deterministic (no RNG): curve counts and offsets
//! derive from the loop indices, so the corpus is the same on every run
//! and platform.

use fmn_geom::QuadPath;
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::{Mob, Mobject, Stage};

type Vec3 = [f64; 3];
/// Per-subpath (start, end) anchors as exact f32 triples.
type Endpoints = Vec<([f32; 3], [f32; 3])>;
/// Handle-keyed pre-align snapshot: (member, endpoints, record count).
type PreSnapshot = Vec<(Mob, Endpoints, usize)>;

/// A deterministic squiggle subpath with `n_curves` curves (2n+1 shared-
/// anchor points), offset so distinct subpaths never coincide.
fn subpath_points(n_curves: usize, ox: f64, oy: f64) -> Vec<Vec3> {
    (0..2 * n_curves + 1)
        .map(|j| {
            let x = ox + j as f64 * 0.5;
            let y = oy + f64::from(((j * 3 + n_curves * 7) % 5) as u8) * 0.2;
            [x, y, 0.0]
        })
        .collect()
}

/// A vmobject whose point run has `subpaths` subpaths, curve counts
/// varying with `variant`.
fn build_vmob(stage: &mut Stage, subpaths: usize, variant: usize) -> Mob {
    let mut path =
        QuadPath::from_points(subpath_points(1 + (variant) % 3, 0.0, variant as f64)).unwrap();
    for i in 1..subpaths {
        path.add_subpath(&subpath_points(
            1 + (variant + i) % 3,
            i as f64 * 4.0,
            variant as f64 + i as f64 * 2.0,
        ))
        .unwrap();
    }
    let points = path.points().to_vec();
    let mob = stage.add(Mobject::new());
    let entry = stage.get_mut(mob).unwrap();
    entry.buffer = RecordBuffer::new(RecordSchema::vmobject(), points.len()).unwrap();
    #[allow(clippy::cast_possible_truncation)]
    let flat: Vec<f32> = points
        .iter()
        .flat_map(|p| p.iter().map(|v| *v as f32))
        .collect();
    entry.buffer.write_range("point", 0, &flat);
    mob
}

/// A family: childless → the vmobject itself; otherwise an empty
/// vmobject root with `children` squiggle children.
fn build_family(stage: &mut Stage, children: usize, subpaths: usize, variant: usize) -> Mob {
    if children == 0 {
        return build_vmob(stage, subpaths, variant);
    }
    let root = stage.add(Mobject::new());
    stage.get_mut(root).unwrap().buffer = RecordBuffer::new(RecordSchema::vmobject(), 0).unwrap();
    for c in 0..children {
        let child = build_vmob(stage, 1 + (subpaths + c) % 3, variant + c + 1);
        stage.attach(root, child).unwrap();
    }
    root
}

/// The (start, end) anchors of every subpath, as exact f32 triples.
fn subpath_endpoints(stage: &Stage, mob: Mob) -> Vec<([f32; 3], [f32; 3])> {
    let Some(column) = stage.get(mob).and_then(|e| e.buffer.read_column("point")) else {
        return Vec::new();
    };
    if column.is_empty() {
        return Vec::new();
    }
    let points: Vec<Vec3> = column
        .as_chunks::<3>()
        .0
        .iter()
        .map(|c| [f64::from(c[0]), f64::from(c[1]), f64::from(c[2])])
        .collect();
    let Ok(path) = QuadPath::from_points(points) else {
        return Vec::new();
    };
    #[allow(clippy::cast_possible_truncation)]
    path.subpaths()
        .into_iter()
        .map(|sp| {
            let f = sp.first().unwrap();
            let l = sp.last().unwrap();
            (
                [f[0] as f32, f[1] as f32, f[2] as f32],
                [l[0] as f32, l[1] as f32, l[2] as f32],
            )
        })
        .collect()
}

fn subpath_count(stage: &Stage, mob: Mob) -> usize {
    subpath_endpoints(stage, mob).len()
}

#[test]
fn corpus_1024_pairs_align_structurally() {
    let mut pairs = 0usize;
    for v in 0..4usize {
        for sa in 1..=4usize {
            for sb in 1..=4usize {
                for ca in 0..=3usize {
                    for cb in 0..=3usize {
                        let variant = v * 16 + sa * 4 + sb;
                        let mut stage = Stage::new();
                        let a = build_family(&mut stage, ca, sa, variant);
                        let b = build_family(&mut stage, cb, sb, variant + 17);

                        // Pre-align subpath endpoints and point counts,
                        // keyed by handle (alignment inserts ghost members
                        // mid-family, so indices do not survive it).
                        let snapshot = |stage: &Stage, root: Mob| -> PreSnapshot {
                            stage
                                .family(root)
                                .into_iter()
                                .map(|m| {
                                    (
                                        m,
                                        subpath_endpoints(stage, m),
                                        stage.get(m).map_or(0, |e| e.buffer.len()),
                                    )
                                })
                                .collect()
                        };
                        let pre_a = snapshot(&stage, a);
                        let pre_b = snapshot(&stage, b);
                        let pre_len_of = |pre: &PreSnapshot, mob: Mob| {
                            pre.iter().find(|(m, _, _)| *m == mob).map(|(_, _, l)| *l)
                        };

                        stage
                            .align_data_and_family(a, b)
                            .unwrap_or_else(|e| panic!("pair ({sa},{sb},{ca},{cb}): {e}"));

                        let ctx = format!("pair ({sa},{sb},{ca},{cb})");
                        assert!(stage.is_aligned_with(a, b), "{ctx}: is_aligned_with");
                        let fam_a = stage.family(a);
                        let fam_b = stage.family(b);
                        assert_eq!(fam_a.len(), fam_b.len(), "{ctx}: family sizes");

                        for (i, (&ma, &mb)) in fam_a.iter().zip(fam_b.iter()).enumerate() {
                            let ea = stage.get(ma).unwrap();
                            let eb = stage.get(mb).unwrap();
                            let (la, lb) = (ea.buffer.len(), eb.buffer.len());
                            assert_eq!(la, lb, "{ctx} member {i}: record counts");
                            if la == 0 {
                                continue;
                            }
                            // Shared-anchor validity: odd runs that reparse.
                            assert_eq!(la % 2, 1, "{ctx} member {i}: odd point run");
                            // Subpath counts are asserted equal only when
                            // neither side needed synthesis: a folded
                            // (doubled-back) or center-point-padded section
                            // may legitimately reparse with extra null-curve
                            // breaks — the Reference promises point counts,
                            // not reparse topology, there.
                            let real_a = pre_len_of(&pre_a, ma).is_some_and(|l| l >= 3);
                            let real_b = pre_len_of(&pre_b, mb).is_some_and(|l| l >= 3);
                            let pre_subs = |pre: &PreSnapshot, mob: Mob| {
                                pre.iter()
                                    .find(|(m, _, _)| *m == mob)
                                    .map(|(_, eps, _)| eps.len())
                            };
                            if real_a && real_b && pre_subs(&pre_a, ma) == pre_subs(&pre_b, mb) {
                                assert_eq!(
                                    subpath_count(&stage, ma),
                                    subpath_count(&stage, mb),
                                    "{ctx} member {i}: subpath counts equalized"
                                );
                            }
                            // Joint angles finite everywhere.
                            for field in [ma, mb] {
                                let angles = stage
                                    .get(field)
                                    .unwrap()
                                    .buffer
                                    .read_column("joint_angle")
                                    .unwrap();
                                assert!(
                                    angles.iter().all(|v| v.is_finite()),
                                    "{ctx} member {i}: finite joint angles"
                                );
                            }
                        }

                        // Interpolation endpoints: every original subpath's
                        // (start, end) anchors survive alignment exactly, on
                        // both sides (insertions never move anchors) — keyed
                        // by handle, since ghosts joined the families.
                        for pre in [&pre_a, &pre_b] {
                            for (member, pre_eps, pre_len) in pre {
                                if *pre_len < 3 {
                                    continue; // degenerate members re-anchor
                                }
                                let post = subpath_endpoints(&stage, *member);
                                for ep in pre_eps {
                                    assert!(
                                        post.contains(ep),
                                        "{ctx} member {member:?}: endpoint {ep:?} survived"
                                    );
                                }
                            }
                        }
                        pairs += 1;
                    }
                }
            }
        }
    }
    assert_eq!(pairs, 1024);
}
