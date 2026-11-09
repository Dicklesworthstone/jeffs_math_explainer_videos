//! Positional-parity corpus (fm-jru, §8.4, §16.4): every scenario in
//! `fixtures/positional.txt` is rebuilt as a Stage family, driven through the
//! positional API with the same parameters the Reference reproduction used
//! (`scripts/gen_positional_fixtures.py`), and checked point-for-point plus the
//! root bounding box. Loose f32 tolerance, since we compute in f64 over f32
//! records.

use fmn_core::constants::{DL, DOWN, LEFT, ORIGIN, RIGHT, UP, UR};
use fmn_core::types::Vec3;
use fmn_mobject::{Mob, Mobject, Stage};

/// Absolute tolerance: coordinates are O(10) and stored as f32 (~1e-6 relative),
/// with a few chained ops; 2e-3 is comfortably loose per §16.4.
const TOL: f64 = 2e-3;

struct InNode {
    parent: i64,
    points: Vec<Vec3>,
}

struct Scenario {
    name: String,
    nodes: Vec<InNode>,
    out: Vec<Vec<Vec3>>,
    bbox: [Vec3; 3],
}

fn parse_points(fields: &[&str]) -> Vec<Vec3> {
    let (chunks, _rem) = fields.as_chunks::<3>();
    chunks
        .iter()
        .map(|c| {
            [
                c[0].parse().unwrap(),
                c[1].parse().unwrap(),
                c[2].parse().unwrap(),
            ]
        })
        .collect()
}

fn load() -> Vec<Scenario> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/positional.txt");
    let text = std::fs::read_to_string(path).expect("positional fixture present");
    let mut scenarios = Vec::new();
    let mut cur: Option<Scenario> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let tok: Vec<&str> = line.split_whitespace().collect();
        match tok[0] {
            "SCENARIO" => {
                cur = Some(Scenario {
                    name: tok[1].to_string(),
                    nodes: Vec::new(),
                    out: Vec::new(),
                    bbox: [[0.0; 3]; 3],
                });
            }
            "NODES" => {}
            "IN" => {
                let s = cur.as_mut().unwrap();
                let parent: i64 = tok[2].parse().unwrap();
                s.nodes.push(InNode {
                    parent,
                    points: parse_points(&tok[4..]),
                });
            }
            "OUT" => {
                let s = cur.as_mut().unwrap();
                s.out.push(parse_points(&tok[3..]));
            }
            "BBOX" => {
                let s = cur.as_mut().unwrap();
                let v = parse_points(&tok[1..]);
                s.bbox = [v[0], v[1], v[2]];
            }
            "END" => scenarios.push(cur.take().unwrap()),
            other => panic!("unknown fixture token {other}"),
        }
    }
    scenarios
}

/// Build the family in the stage; return the node handles in index order.
fn build(stage: &mut Stage, s: &Scenario) -> Vec<Mob> {
    let handles: Vec<Mob> = s
        .nodes
        .iter()
        .map(|n| stage.add(Mobject::from_points(&n.points)))
        .collect();
    for (i, n) in s.nodes.iter().enumerate() {
        if n.parent >= 0 {
            stage
                .attach(handles[n.parent as usize], handles[i])
                .expect("attach");
        }
    }
    handles
}

/// Apply the operation named by the scenario, with the same parameters the
/// Reference reproduction used.
fn apply(stage: &mut Stage, name: &str, h: &[Mob]) {
    match name {
        "shift" => {
            stage.shift(h[0], [1.0, -2.0, 0.5]);
        }
        "scale_center" => {
            stage.scale(h[0], 2.0);
        }
        "scale_about_edge" => {
            stage.scale_about(h[0], 0.5, None, Some(DL));
        }
        "scale_about_point" => {
            stage.scale_about(h[0], 3.0, Some([1.0, 1.0, 0.0]), None);
        }
        "stretch_x" => {
            stage.stretch(h[0], 2.5, 0);
        }
        "stretch_y" => {
            stage.stretch(h[0], 0.5, 1);
        }
        "center" => {
            stage.center(h[0]);
        }
        "to_edge_up" => {
            stage.to_edge(h[0], UP, 0.5);
        }
        "to_edge_left" => {
            stage.to_edge(h[0], LEFT, 0.5);
        }
        "to_corner_ur" => {
            stage.to_corner(h[0], UR, 0.25);
        }
        "next_to_right" => {
            stage.next_to(h[0], h[1], RIGHT, 0.25, ORIGIN);
        }
        "next_to_up_aligned" => {
            stage.next_to(h[0], h[1], UP, 0.5, LEFT);
        }
        "next_to_point" => {
            stage.next_to(h[0], [2.0, 2.0, 0.0], DOWN, 0.1, ORIGIN);
        }
        "move_to_point" => {
            stage.move_to(h[0], [1.0, 1.0, 0.0], ORIGIN);
        }
        "move_to_mob_edge" => {
            stage.move_to(h[0], h[1], UP);
        }
        "align_to_mob" => {
            stage.align_to(h[0], h[1], UP);
        }
        "set_x" => {
            stage.set_x(h[0], -3.0);
        }
        "set_y" => {
            stage.set_y(h[0], 2.5);
        }
        "set_width_scale" => {
            stage.set_width(h[0], 4.0, false);
        }
        "set_height_stretch" => {
            stage.set_height(h[0], 3.0, true);
        }
        "match_width" => {
            stage.match_width(h[0], h[1]);
        }
        "match_x" => {
            stage.match_x(h[0], h[1]);
        }
        "nested_bbox_shift" => {
            stage.shift(h[0], [1.0, 1.0, 0.0]);
        }
        "arrange_right" => {
            stage.arrange(h[0], RIGHT, 0.5, true);
        }
        "arrange_down" => {
            stage.arrange(h[0], DOWN, 0.25, true);
        }
        "arrange_in_grid" => {
            stage.arrange_in_grid(h[0], 2, 2, 0.3, 0.3, ORIGIN);
        }
        other => panic!("scenario {other} has no op binding in the parity test"),
    }
}

fn own_points(stage: &Stage, mob: Mob) -> Vec<Vec3> {
    stage.get_points(mob).unwrap_or_default()
}

fn close(a: Vec3, b: Vec3, ctx: &str) {
    for k in 0..3 {
        assert!(
            (a[k] - b[k]).abs() <= TOL,
            "{ctx}: axis {k}: got {} want {} (Δ {})",
            a[k],
            b[k],
            (a[k] - b[k]).abs()
        );
    }
}

#[test]
fn positional_parity_corpus() {
    let scenarios = load();
    assert!(scenarios.len() >= 20, "expected the full corpus");
    for s in &scenarios {
        let mut stage = Stage::new();
        let handles = build(&mut stage, s);
        apply(&mut stage, &s.name, &handles);

        // Every node's own points must match the Reference reproduction.
        for (i, &mob) in handles.iter().enumerate() {
            let got = own_points(&stage, mob);
            let want = &s.out[i];
            assert_eq!(got.len(), want.len(), "{}: node {i} point count", s.name);
            for (g, w) in got.iter().zip(want) {
                close(*g, *w, &format!("{} node {i}", s.name));
            }
        }

        // The root bounding box must match [min, mid, max].
        let bb = stage.get_bounding_box(handles[0]);
        close(bb.min, s.bbox[0], &format!("{} bbox.min", s.name));
        close(bb.mid, s.bbox[1], &format!("{} bbox.mid", s.name));
        close(bb.max, s.bbox[2], &format!("{} bbox.max", s.name));
    }
}

#[test]
fn affine_motion_has_an_independent_revision_and_object_space_buffer() {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_points(&[
        [0.0, 0.0, 0.0],
        [1.0, 0.5, 0.0],
        [2.0, 1.0, 0.0],
    ]));
    let object_points = stage.get_object_points(mob).unwrap();
    let point_revision = stage
        .get(mob)
        .unwrap()
        .buffer
        .field_revision("point")
        .unwrap();
    let placement_revision = stage.placement_revision(mob).unwrap();

    stage.shift(mob, [5.0, -2.0, 0.25]);
    assert_eq!(stage.get_object_points(mob).unwrap(), object_points);
    assert_eq!(
        stage.get(mob).unwrap().buffer.field_revision("point"),
        Some(point_revision)
    );
    assert!(stage.placement_revision(mob).unwrap() > placement_revision);
    assert_eq!(
        stage.get_points(mob).unwrap(),
        vec![[5.0, -2.0, 0.25], [6.0, -1.5, 0.25], [7.0, -1.0, 0.25],]
    );

    stage.rotate(
        mob,
        std::f64::consts::FRAC_PI_2,
        [0.0, 0.0, 1.0],
        Some([5.0, -2.0, 0.25]),
        None,
    );
    stage.scale_about(mob, 2.0, Some([5.0, -2.0, 0.25]), None);
    assert_eq!(
        stage.get_object_points(mob).unwrap(),
        object_points,
        "rotation and scale are placement edits too"
    );
    assert_eq!(
        stage.get(mob).unwrap().buffer.field_revision("point"),
        Some(point_revision)
    );
}

#[test]
fn affine_motion_stays_visible_through_an_existing_zero_copy_view() {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_points(&[[1.0, 2.0, 0.0], [3.0, 4.0, 0.0]]));
    let view = stage.get_mut(mob).expect("live").buffer.export_view(false);

    stage.shift(mob, [5.0, -1.0, 0.25]);

    assert_eq!(view.read(0, "point"), Some(vec![6.0, 1.0, 0.25]));
    assert_eq!(view.read(1, "point"), Some(vec![8.0, 3.0, 0.25]));
    assert!(
        stage.placement(mob).unwrap().is_identity(),
        "a pinned authoritative buffer receives the affine write directly"
    );
}

#[test]
fn an_arbitrary_point_map_bakes_the_current_placement_once() {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_points(&[[1.0, 2.0, 0.0]]));
    stage.shift(mob, [3.0, 4.0, 0.0]);
    let before = stage
        .get(mob)
        .unwrap()
        .buffer
        .field_revision("point")
        .unwrap();

    stage.apply_points_function(
        mob,
        |point| [point[0] * 2.0, point[1] - 1.0, point[2]],
        None,
        None,
    );

    assert!(stage.placement(mob).unwrap().is_identity());
    assert_eq!(stage.get_points(mob).unwrap(), vec![[8.0, 5.0, 0.0]]);
    assert!(
        stage
            .get(mob)
            .unwrap()
            .buffer
            .field_revision("point")
            .unwrap()
            > before
    );
}

#[test]
fn copies_and_in_memory_snapshots_carry_placement_state() {
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_points(&[[1.0, 2.0, 0.0]]));
    stage.shift(mob, [3.0, -1.0, 0.5]);
    stage.stretch_about(mob, 2.0, 0, Some([4.0, 1.0, 0.5]), None);
    let placement = stage.placement(mob).unwrap();
    let world_points = stage.get_points(mob).unwrap();

    let copy = stage.copy_family(mob).unwrap();
    assert!(stage.placement(copy).unwrap().same_bits(placement));
    assert_eq!(stage.get_points(copy).unwrap(), world_points);

    let snapshot = stage.snapshot();
    stage.shift(mob, [100.0, 0.0, 0.0]);
    stage.restore(&snapshot);
    assert!(stage.placement(mob).unwrap().same_bits(placement));
    assert_eq!(stage.get_points(mob).unwrap(), world_points);
}

#[test]
fn shift_onto_screen_pulls_back_only_the_edges_beyond_the_inset_frame() {
    use fmn_core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
    let square = |center: Vec3| {
        let [x, y, z] = center;
        Mobject::from_points(&[
            [x - 0.5, y - 0.5, z],
            [x + 0.5, y - 0.5, z],
            [x + 0.5, y + 0.5, z],
            [x - 0.5, y + 0.5, z],
        ])
    };
    let mut stage = Stage::new();
    // Beyond the right and top edges: both pulled to `radius - buff`.
    let off = stage.add(square([9.0, 5.0, 0.0]));
    stage.shift_onto_screen(off, 0.5);
    assert!((stage.get_right(off)[0] - (FRAME_X_RADIUS - 0.5)).abs() < TOL);
    assert!((stage.get_top(off)[1] - (FRAME_Y_RADIUS - 0.5)).abs() < TOL);
    // Inside the inset frame: untouched.
    let inside = stage.add(square([1.0, -1.0, 0.0]));
    stage.shift_onto_screen(inside, 0.5);
    assert!((stage.get_center(inside)[0] - 1.0).abs() < TOL);
    assert!((stage.get_center(inside)[1] + 1.0).abs() < TOL);
    // Beyond the bottom-left: pulled to the inset bottom-left.
    let low = stage.add(square([-9.0, -6.0, 0.0]));
    stage.shift_onto_screen(low, 0.25);
    assert!((stage.get_left(low)[0] + (FRAME_X_RADIUS - 0.25)).abs() < TOL);
    assert!((stage.get_bottom(low)[1] + (FRAME_Y_RADIUS - 0.25)).abs() < TOL);
}
