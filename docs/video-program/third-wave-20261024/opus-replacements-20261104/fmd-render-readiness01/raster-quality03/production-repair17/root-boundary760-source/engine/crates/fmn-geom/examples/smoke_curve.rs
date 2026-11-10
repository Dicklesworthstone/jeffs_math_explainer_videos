//! Local smoke for the curve-aware boolean routes (fm-8dx, fm-qjy):
//! admitted curve scenes must route to a curve-aware class and preserve
//! quadratic handles. The integration battery separately proves agreement
//! with the forced flatten-and-clip Stage 1 on a point-in-fill grid.

use fmn_geom::{BooleanOperation, BooleanOptions, BooleanRoute, QuadPath, path_boolean};

fn rectangle(x0: f64, y0: f64, x1: f64, y1: f64) -> QuadPath {
    let mut path = QuadPath::new();
    path.start_new_path([x0, y0, 0.0]);
    for point in [[x1, y0], [x1, y1], [x0, y1], [x0, y0]] {
        path.add_line_to([point[0], point[1], 0.0], true)
            .expect("a started path accepts lines");
    }
    path
}

fn circle(radius: f64, center: [f64; 2]) -> QuadPath {
    QuadPath::try_arc(
        0.0,
        fmn_core::constants::TAU,
        radius,
        [center[0], center[1], 0.0],
        None,
    )
    .expect("valid arc")
}

fn main() {
    let scenes: Vec<(QuadPath, QuadPath, &str)> = vec![
        (
            circle(2.0, [0.0, 0.0]),
            circle(1.5, [1.0, 0.0]),
            "circle×circle",
        ),
        (
            circle(2.0, [0.0, 0.0]),
            rectangle(-1.0, -3.0, 1.0, 3.0),
            "circle×rectangle",
        ),
        (
            circle(3.0, [0.0, 0.0]),
            circle(1.0, [0.4, -0.3]),
            "nested circles",
        ),
    ];
    for (subject, clip, name) in &scenes {
        for operation in [
            BooleanOperation::Union,
            BooleanOperation::Intersection,
            BooleanOperation::Difference,
            BooleanOperation::Exclusion,
        ] {
            let result = path_boolean(subject, clip, operation, BooleanOptions::default())
                .expect("smoke scenes boolean cleanly");
            println!(
                "{name} {operation:?}: route={:?} contours={} segments={}",
                result.route, result.stats.output_contours, result.stats.output_segments
            );
            assert_eq!(
                result.route,
                BooleanRoute::CurveAwareTransversal,
                "{name}: transversal scene must admit"
            );
            assert_eq!(result.stats.flattened_segments, 0);
        }
    }
    println!("curve boolean smoke ok");
}
