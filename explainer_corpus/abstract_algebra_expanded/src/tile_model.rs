//! Rigid 3D tile state for the replacement film. Not yet registered in a scene.
//! All face vertices, edges and emblems must use this same transformation.
pub type Point = [f64; 3];
pub type Matrix = [[f64; 3]; 3];
pub const IDENTITY: Matrix = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
pub const CORNERS: [Point; 4] = [[1., 1., 0.], [1., -1., 0.], [-1., -1., 0.], [-1., 1., 0.]];

pub fn apply(m: Matrix, p: Point) -> Point {
    m.map(|row| row.iter().zip(p).map(|(a, b)| a * b).sum())
}

/// Left matrix acts last, matching function composition.
pub fn compose(a: Matrix, b: Matrix) -> Matrix {
    std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| a[i][k] * b[k][j]).sum()))
}

pub fn turn(t: f64) -> Matrix {
    let (s, c) = (std::f64::consts::FRAC_PI_2 * t).sin_cos();
    [[c, -s, 0.], [s, c, 0.], [0., 0., 1.]]
}

/// A physical half-turn about the horizontal hinge, not an in-plane collapse.
pub fn flip(t: f64) -> Matrix {
    let (s, c) = (std::f64::consts::PI * t).sin_cos();
    [[1., 0., 0.], [0., c, -s], [0., s, c]]
}

/// Perspective from a fixed camera in front of the tile. World state remains 3D.
pub fn project(p: Point, camera_distance: f64) -> Option<[f64; 2]> {
    let depth = camera_distance - p[2];
    if !camera_distance.is_finite() || camera_distance <= 0. || depth <= 0. {
        return None;
    }
    Some([camera_distance * p[0] / depth, camera_distance * p[1] / depth])
}

pub fn vertices(state: Matrix) -> [Point; 4] {
    CORNERS.map(|p| apply(state, p))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn close(a: Point, b: Point) {
        for i in 0..3 {
            assert!((a[i] - b[i]).abs() < 1e-10);
        }
    }
    #[test]
    fn opening_endpoints() {
        close(apply(compose(flip(1.), turn(1.)), CORNERS[0]), [-1., -1., 0.]);
        close(apply(compose(turn(1.), flip(1.)), CORNERS[0]), [1., 1., 0.]);
    }
    #[test]
    fn motion_preserves_every_pairwise_distance() {
        for n in 0..=100 {
            for motion in [turn(n as f64 / 100.), flip(n as f64 / 100.)] {
                let points = vertices(motion);
                for i in 0..4 {
                    for j in 0..4 {
                        let distance = |a: Point, b: Point| (0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f64>();
                        assert!((distance(points[i], points[j]) - distance(CORNERS[i], CORNERS[j])).abs() < 1e-10);
                    }
                }
            }
        }
    }
    #[test]
    fn turnover_changes_face_and_four_turns_restore_tile() {
        close(apply(flip(1.), [0., 0., 1.]), [0., 0., -1.]);
        let r = turn(1.);
        for p in CORNERS {
            close(apply(compose(compose(r, r), compose(r, r)), p), p);
        }
        assert!(project([0., 0., 6.], 6.).is_none());
    }
}
