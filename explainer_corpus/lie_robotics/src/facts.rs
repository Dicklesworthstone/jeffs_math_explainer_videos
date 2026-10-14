//! Every number Video G shows or says, computed here from `model.rs` and
//! checked two ways: `pin` against a literal worked out by hand or read from
//! a source (the fact sheet names each), and `./x lie_robotics facts-check`
//! against `oracle.py` (same keys), which reaches every value by another
//! route: the URDF chain, a generic series exponential and logarithm, finite
//! differences and triangle geometry.

use std::f64::consts::PI;

use explainer_kit::Facts;

use crate::model::{
    self, Arm, CUBE_AXES, DEG, I3, M3, N, Pose, QUIZZES, SWEEP_DEG, V3, X, Y, Z, add, along_screw, angle_axis, apply,
    column, degrees, det, exp6, gap, goal, halfway_deg, hat, leftover, log6, mul, norm, planar_centre, planar_exp,
    poses, rank, rings, rings_left, roll_then_tilt, roll_turn, scale, screw_of, still_speeds, sub, tilt_then_roll,
    tilt_turn, transpose, turn, twist, twist_from, unit, wheel_speeds, wrap_deg,
};

/// Numbers quoted from the sources (fact sheet rows G-L; the oracle prints
/// the same literals).
pub mod cited {
    /// KUKA, LBR iiwa 14 R820 data sheet: maximum reach, rated payload, axes.
    pub const KUKA_REACH_MM: u32 = 820;
    pub const KUKA_PAYLOAD_KG: u32 = 14;
    pub const KUKA_AXES: u32 = 7;
    /// KUKA, KMR iiwa data sheet: velocity in the longitudinal direction.
    pub const KMR_SPEED_KMH: f64 = 3.6;
    /// The Apollo guidance computer's gimbal monitor (Comanche 055,
    /// GLOCKMON): the lamp above 70 degrees of middle gimbal angle, the
    /// reference given up above 85; three gimbals.
    pub const APOLLO_LAMP_DEG: u32 = 70;
    pub const APOLLO_LOST_DEG: u32 = 85;
    pub const APOLLO_GIMBALS: u32 = 3;
    /// Euler's E478, presented 9 October 1775.
    pub const EULER_YEAR: u32 = 1775;
    /// Mozzi's screw axis (Ceccarelli 2000, title) and Chasles's note.
    pub const MOZZI_YEAR: u32 = 1763;
    pub const CHASLES_YEAR: u32 = 1830;
    /// Rodrigues, Journal de Mathématiques Pures et Appliquées 5.
    pub const RODRIGUES_YEAR: u32 = 1840;
    /// Lie began his theory of continuous groups in the winter of 1873-74.
    pub const LIE_YEAR: u32 = 1873;
    /// Sophus Lie, 1842 to 1899 (MacTutor): the years on 05_7's name card.
    pub const LIE_BORN: u32 = 1842;
    pub const LIE_DIED: u32 = 1899;
    /// Brockett's talk at MTNS-83, Beer Sheva (printed 1984).
    pub const BROCKETT_YEAR: u32 = 1983;
    /// Unitree G1, `g1_29dof.urdf`: revolute joints.
    pub const G1_JOINTS: usize = 29;
}

/// The worked examples the chapters draw.
pub mod shown {
    use super::V3;
    /// The two headings of chapter 04, degrees.
    pub const HEADINGS: (f64, f64) = (170.0, -170.0);
    /// The base's velocity and how long it is held: m/s, degrees/s, s.
    pub const BASE: (f64, f64, f64) = (0.5, 30.0, 6.0);
    /// The spin of chapter 05: about the book's diagonal, degrees/s.
    pub const DIAGONAL: V3 = [1.0, 1.0, 1.0];
    pub const DIAGONAL_RATE: f64 = 120.0;
    /// The book tossed in chapter 07: a quarter turn about the vertical and
    /// a slide.
    pub const TOSS_TURN_DEG: f64 = 90.0;
    pub const TOSS_SLIDE: V3 = [1.0, 1.0, 0.5];
    /// Ring angles (yaw, pitch, roll) that agree at gimbal lock.
    pub const LOCKED: ([f64; 3], [f64; 3]) = ([30.0, 90.0, 0.0], [0.0, 90.0, -30.0]);
    /// Two orientations close together whose angles are far apart.
    pub const NEAR: ([f64; 3], [f64; 3]) = ([0.0, 89.0, 0.0], [90.0, 89.0, 90.0]);
    /// Pulling the tip back at this speed (m/s), elbow bent these degrees.
    pub const PULL: f64 = 0.1;
    pub const BENDS: [f64; 4] = [30.0, 10.0, 3.0, 1.0];
    /// Stepping a spinning body: steps, step length (s), spin 1 rad/s.
    pub const DRIFT: (usize, f64) = (1000, 0.01);
    /// The small turns of the last quiz, degrees.
    pub const SMALL_TURNS: [f64; 3] = [1.0, 10.0, 90.0];
}

/// Fixed decimals, never a negative zero (the oracle's `fx`).
pub fn fx(x: f64, places: usize) -> String {
    let s = format!("{x:.places$}");
    match s.strip_prefix('-') {
        Some(rest) if rest.parse::<f64>() == Ok(0.0) => rest.to_owned(),
        _ => s,
    }
}

pub fn vec(v: &[f64], places: usize) -> String {
    v.iter().map(|x| fx(*x, places)).collect::<Vec<_>>().join(",")
}

fn whole(x: f64) -> Result<i64, String> {
    let r = x.round();
    if (x - r).abs() < 1e-9 {
        Ok(r as i64)
    } else {
        Err(format!("{x} is not a whole number"))
    }
}

fn ints(v: &[f64]) -> Result<String, String> {
    Ok(v.iter()
        .map(|x| whole(*x).map(|i| i.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .join(","))
}

fn grid(m: &M3) -> Result<String, String> {
    Ok(m.iter().map(|row| ints(row)).collect::<Result<Vec<_>, _>>()?.join(";"))
}

fn flag(b: bool) -> u8 {
    u8::from(b)
}

fn ypr(a: [f64; 3]) -> M3 {
    rings(a[0] * DEG, a[1] * DEG, a[2] * DEG)
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    let arm = Arm::iiwa14();
    the_arm(&mut f, &arm)?;
    the_book(&mut f)?;
    three_angles(&mut f)?;
    halfway(&mut f)?;
    spins(&mut f)?;
    the_base(&mut f)?;
    screws(&mut f)?;
    arm_poses(&mut f, &arm)?;
    velocities(&mut f, &arm)?;
    steps_and_paths(&mut f, &arm)?;
    coda(&mut f)?;
    edges(&mut f)?;
    quizzes(&mut f, &arm)?;
    sources(&mut f)?;
    Ok(f)
}

fn the_arm(f: &mut Facts, arm: &Arm) -> Result<(), String> {
    const XACRO: &str = "vendor/iiwa14.xacro (cmaes_explainer), joint origins";
    let offsets: Vec<f64> = model::JOINT_ORIGINS
        .iter()
        .map(|(xyz, _)| norm(*xyz))
        .chain([norm(model::FLANGE)])
        .collect();
    f.add("G-A.joints", N)?;
    f.pin(
        "G-A.offsets",
        vec(&offsets, 4),
        "0.1575,0.2025,0.2045,0.2155,0.1845,0.2155,0.0810,0.0450",
        XACRO,
    )?;
    let (shoulder, elbow, wrist) = (
        arm.origin[model::SHOULDER],
        arm.origin[model::ELBOW],
        arm.origin[model::WRIST],
    );
    f.pin("G-A.shoulder_height", fx(shoulder[2], 4), "0.3600", "0.1575 + 0.2025")?;
    f.pin(
        "G-A.upper_arm",
        fx(norm(sub(elbow, shoulder)), 4),
        "0.4200",
        "0.2045 + 0.2155",
    )?;
    f.pin(
        "G-A.forearm",
        fx(norm(sub(wrist, elbow)), 4),
        "0.4000",
        "0.1845 + 0.2155",
    )?;
    f.pin(
        "G-A.wrist_to_tip",
        fx(norm(sub(arm.home.p, wrist)), 4),
        "0.1260",
        "0.081 + 0.045",
    )?;
    f.pin("G-A.height", fx(arm.home.p[2], 4), "1.3060", "the eight offsets added")?;
    f.pin(
        "G-A.reach",
        fx(norm(sub(wrist, shoulder)), 4),
        "0.8200",
        "KUKA data sheet, LBR iiwa 14 R820: maximum reach 820 mm",
    )?;
    f.add("G-A.home.p", vec(&arm.home.p, 4))?;
    f.pin(
        "G-A.home.R",
        grid(&arm.home.r)?,
        "1,0,0;0,1,0;0,0,1",
        "the tip's frame at home is the room's",
    )?;
    let axes = ["0,0,1", "0,1,0", "0,0,1", "0,-1,0", "0,0,1", "0,1,0", "0,0,1"];
    for (i, want) in axes.iter().enumerate() {
        f.pin(
            &format!("G-A.axis.{}", i + 1),
            ints(&arm.axis[i])?,
            want,
            "the xacro's frames at zero",
        )?;
        f.add(&format!("G-A.axis_height.{}", i + 1), fx(arm.origin[i][2], 4))?;
    }
    f.pin(
        "G-A.limits_deg",
        ints(&model::LIMITS_DEG)?,
        "170,120,170,120,170,120,175",
        "KUKA data sheet: motion range A1 to A7",
    )?;
    f.pin(
        "G-A.speed_deg_s",
        ints(&model::SPEED_DEG_S)?,
        "85,85,100,75,130,135,135",
        "KUKA data sheet: speed with rated payload A1 to A7",
    )?;
    Ok(())
}

fn the_book(f: &mut Facts) -> Result<(), String> {
    let quarter = PI / 2.0;
    let (tw, wt) = (tilt_then_roll(quarter), roll_then_tilt(quarter));
    f.add("G-B.tilt_then_roll.R", grid(&tw)?)?;
    f.add("G-B.roll_then_tilt.R", grid(&wt)?)?;
    f.pin(
        "G-B.tilt_then_roll.cover",
        ints(&apply(&tw, Z))?,
        "0,-1,0",
        "the cover faces the viewer",
    )?;
    f.pin(
        "G-B.roll_then_tilt.cover",
        ints(&apply(&wt, Z))?,
        "1,0,0",
        "the cover faces the viewer's right",
    )?;
    for (name, r) in [("tilt_then_roll", &tw), ("roll_then_tilt", &wt)] {
        let (a, axis) = angle_axis(r);
        f.pin(
            &format!("G-B.{name}.angle"),
            fx(a / DEG, 4),
            "120.0000",
            "trace 0: cos = -1/2",
        )?;
        f.add(&format!("G-B.{name}.axis"), vec(&axis, 4))?;
    }
    let (a, axis) = angle_axis(&leftover(quarter));
    f.add("G-B.between.angle", fx(a / DEG, 4))?;
    f.add("G-B.between.axis", vec(&axis, 4))?;
    let (right, away) = ([0.1, 0.0, 0.0], [0.0, 0.1, 0.0]);
    f.add("G-B.slides.same", flag(add(right, away) == add(away, right)))?;
    f.add("G-B.turns.same", flag(gap(&tw, &wt) < 1e-9))?;
    // the two spins held together for one second: the sum of the two arrows
    let both = exp6(twist(add(scale(X, quarter), scale(Y, quarter)), [0.0; 3])).r;
    let (a, axis) = angle_axis(&both);
    f.add("G-B.both_at_once.angle", fx(a / DEG, 4))?;
    f.add("G-B.both_at_once.axis", vec(&axis, 4))?;
    for small in shown::SMALL_TURNS {
        let s = small * DEG;
        let (a, axis) = angle_axis(&leftover(s));
        let key = format!("G-B.leftover.{}", whole(small)?);
        f.add(&format!("{key}.angle"), fx(a / DEG, 4))?;
        f.add(&format!("{key}.axis"), vec(&axis, 4))?;
        f.add(&format!("{key}.product"), fx(s * s / DEG, 4))?;
    }
    Ok(())
}

fn three_angles(f: &mut Facts) -> Result<(), String> {
    f.add("G-E.rings.order", "z,y,x")?;
    let (a, b) = shown::LOCKED;
    f.pin(
        "G-E.lock.same",
        fx(gap(&ypr(a), &ypr(b)), 4),
        "0.0000",
        "at a quarter turn of the middle ring only roll minus yaw matters",
    )?;
    f.add(
        "G-E.free.differ",
        fx(gap(&ypr([a[0], 0.0, a[2]]), &ypr([b[0], 0.0, b[2]])), 4),
    )?;
    for (pitch, want) in [(0.0, "1.0000"), (70.0, "0.3420"), (85.0, "0.0872"), (90.0, "0.0000")] {
        f.pin(
            &format!("G-E.left.{}", whole(pitch)?),
            fx(rings_left(pitch * DEG), 4),
            want,
            "cos of the middle ring's angle, from tables",
        )?;
    }
    let (a, b) = shown::NEAR;
    f.add(
        "G-E.near.apart",
        fx(angle_axis(&mul(&transpose(&ypr(a)), &ypr(b))).0 / DEG, 4),
    )?;
    f.add("G-E.near.triples", format!("{};{}", ints(&a)?, ints(&b)?))?;
    f.add(
        "G-E.near.differ",
        ints(&[(b[0] - a[0]).abs(), (b[1] - a[1]).abs(), (b[2] - a[2]).abs()])?,
    )?;
    Ok(())
}

fn halfway(f: &mut Facts) -> Result<(), String> {
    let (a, b) = shown::HEADINGS;
    let mean = (a + b) / 2.0;
    let half = halfway_deg(a, b);
    f.add("G-H.a", ints(&[a])?)?;
    f.add("G-H.b", ints(&[b])?)?;
    f.add("G-H.mean", fx(mean, 4))?;
    f.pin(
        "G-H.gap",
        fx(wrap_deg(b - a), 4),
        "20.0000",
        "10 degrees either side of straight back",
    )?;
    f.pin("G-H.halfway", fx(half.abs(), 4), "180.0000", "straight back")?;
    f.add("G-H.off_by", fx(wrap_deg(half - mean).abs(), 4))?;
    Ok(())
}

fn spins(f: &mut Facts) -> Result<(), String> {
    let axis = unit(shown::DIAGONAL);
    let rate = shown::DIAGONAL_RATE;
    let spun = turn(axis, rate * DEG);
    f.add("G-R.diag.rate_deg", ints(&[rate])?)?;
    f.add("G-R.diag.R", grid(&spun)?)?;
    f.add("G-R.diag.arrow", vec(&scale(axis, rate * DEG), 4))?;
    f.pin(
        "G-R.diag.is_roll_then_tilt",
        flag(gap(&spun, &roll_then_tilt(PI / 2.0)) < 1e-9),
        "1",
        "both send each edge to the next",
    )?;
    let edges = [X, Y, Z]
        .iter()
        .map(|e| ints(&apply(&spun, *e)))
        .collect::<Result<Vec<_>, _>>()?;
    f.pin(
        "G-R.diag.edges",
        edges.join(";"),
        "0,1,0;0,0,1;1,0,0",
        "x to y, y to z, z to x",
    )?;
    let along = scale(axis, model::dot(X, axis));
    f.add("G-R.rod.along", vec(&along, 4))?;
    f.add("G-R.rod.around", vec(&sub(X, along), 4))?;
    f.add("G-R.rod.result", vec(&apply(&spun, X), 4))?;
    f.add("G-R.quarter_z.R", grid(&turn(Z, PI / 2.0))?)?;
    f.add("G-R.numbers.arrow", axis.len())?;
    f.add("G-R.numbers.matrix", spun.len() * spun[0].len())?;
    Ok(())
}

fn the_base(f: &mut Facts) -> Result<(), String> {
    let (v, spin_deg, t) = shown::BASE;
    let spin = spin_deg * DEG;
    let (x, y, heading) = planar_exp(v, 0.0, spin, t);
    f.add("G-P.v", fx(v, 1))?;
    f.add("G-P.spin_deg", ints(&[spin_deg])?)?;
    f.add("G-P.t", ints(&[t])?)?;
    f.pin(
        "G-P.end",
        vec(&[x, y], 4),
        "0.0000,1.9099",
        "a half circle: its diameter, 6 / pi metres, to the left",
    )?;
    f.add("G-P.heading", fx(wrap_deg(heading / DEG).abs(), 4))?;
    f.add("G-P.naive", vec(&[v * t, 0.0], 4))?;
    f.add("G-P.radius", fx(v / spin, 4))?;
    let (cx, cy) = planar_centre(v, 0.0, spin);
    f.add("G-P.centre", vec(&[cx, cy], 4))?;
    f.add("G-P.arc_length", fx(v * t, 4))?;
    let round = 360.0 / spin_deg;
    f.add("G-P.full_circle_t", ints(&[round])?)?;
    let (x, y, _) = planar_exp(v, 0.0, spin, round);
    f.add("G-P.full.end", vec(&[x, y], 4))?;
    // the same six seconds at other spins: the quiz's two other choices
    let (x, y, _) = planar_exp(v, 0.0, 0.0, t);
    f.add("G-P.spin0.end", vec(&[x, y], 4))?;
    let (x, y, _) = planar_exp(v, 0.0, 60.0 * DEG, t);
    f.add("G-P.spin60.end", vec(&[x, y], 4))?;
    let signs = |u: [f64; 4]| u.map(|s| if s > 0.0 { "+" } else { "-" }).join(",");
    f.add("G-P.wheels.forward", signs(wheel_speeds(1.0, 0.0, 0.0, 1.0, 1.0, 1.0)))?;
    f.add("G-P.wheels.sideways", signs(wheel_speeds(0.0, 1.0, 0.0, 1.0, 1.0, 1.0)))?;
    f.add("G-P.wheels.spin", signs(wheel_speeds(0.0, 0.0, 1.0, 1.0, 1.0, 1.0)))?;
    Ok(())
}

fn toss() -> Pose {
    Pose {
        r: turn(Z, shown::TOSS_TURN_DEG * DEG),
        p: shown::TOSS_SLIDE,
    }
}

fn screws(f: &mut Facts) -> Result<(), String> {
    let t = toss();
    let s = screw_of(&t);
    f.add("G-S.toss.turn_deg", fx(s.angle / DEG, 4))?;
    f.add("G-S.toss.axis", vec(&s.axis, 4))?;
    f.pin(
        "G-S.toss.point",
        vec(&s.point, 4),
        "0.0000,1.0000,0.0000",
        "the point a quarter turn carries to itself plus (1, 1)",
    )?;
    f.pin("G-S.toss.slide", fx(s.slide, 4), "0.5000", "the rise")?;
    f.add("G-S.toss.rebuilt", fx(along_screw(&s, 1.0).gap(&t), 4))?;
    f.add("G-S.numbers.pose", 3 + 3)?;
    f.add("G-S.numbers.twist", twist([0.0; 3], [0.0; 3]).len())?;
    f.add("G-S.numbers.turn_matrix", 3 * 3)?;
    f.add("G-S.numbers.pose_matrix", 4 * 4)?;
    f.add("G-S.numbers.quaternion", 4)?;
    Ok(())
}

fn arm_poses(f: &mut Facts, arm: &Arm) -> Result<(), String> {
    let ell = arm.tip(&degrees(poses::ELL));
    f.add("G-K.ell.q", ints(&poses::ELL)?)?;
    f.pin(
        "G-K.ell.p",
        vec(&ell.p, 4),
        "0.4200,0.0000,0.8860",
        "by hand: 0.42 forward; 0.36 + 0.40 + 0.126 up",
    )?;
    f.add("G-K.ell.R", grid(&ell.r)?)?;
    f.pin(
        "G-K.ell.after_elbow",
        vec(&arm.tip(&degrees(poses::ELBOW_ONLY)).p, 4),
        "-0.5260,0.0000,0.7800",
        "by hand: the forearm and wrist, 0.526, level at the elbow's height 0.78",
    )?;
    f.pin(
        "G-K.quarter.p",
        vec(&arm.tip(&degrees(poses::QUARTER)).p, 4),
        "0.0000,0.4200,0.8860",
        "the L shape swung a quarter turn about the vertical",
    )?;
    let (before, after) = (
        arm.skeleton(&degrees(poses::HOME)),
        arm.skeleton(&degrees(poses::ELBOW_ONLY)),
    );
    let moved: Vec<String> = (1..=N + 1)
        .filter(|&i| norm(sub(before[i], after[i])) > 1e-9)
        .map(|i| if i <= N { i.to_string() } else { "tip".to_owned() })
        .collect();
    f.add("G-K.elbow.moves", moved.join(","))?;
    Ok(())
}

fn velocities(f: &mut Facts, arm: &Arm) -> Result<(), String> {
    let ell = degrees(poses::ELL);
    let j = arm.jacobian(&ell);
    f.add("G-J.shape", format!("{}x{}", j.len(), j[0].len()))?;
    for i in 0..N {
        f.add(&format!("G-J.ell.lever.{}", i + 1), fx(arm.lever(&ell, i), 4))?;
    }
    f.pin(
        "G-J.ell.col2",
        vec(&column(&j, model::SHOULDER), 4),
        "0.0000,1.0000,0.0000,-0.3600,0.0000,0.0000",
        "the shoulder's axis: along y, through a point 0.36 up",
    )?;
    // the whole grid chapter 10 writes out: six rows (spin, then slide), seven joints
    f.add(
        "G-J.ell.J",
        j.iter().map(|row| vec(row, 2)).collect::<Vec<_>>().join(";"),
    )?;
    for (name, q) in [
        ("home", poses::HOME),
        ("stretch", poses::STRETCH),
        ("elbow_straight", poses::ELBOW_STRAIGHT),
        ("elbow_only", poses::ELBOW_ONLY),
        ("ell", poses::ELL),
        ("ell_wrist_turned", poses::ELL_WRIST_TURNED),
        ("swing", poses::SWING),
    ] {
        f.add(&format!("G-J.rank.{name}"), rank(&arm.jacobian(&degrees(q))))?;
    }
    // the L shape with one joint taken away: a six-joint arm
    let ell_j = arm.jacobian(&degrees(poses::ELL));
    for k in [3, 7] {
        let without = ell_j.map(|row| {
            let mut r = row;
            r[k - 1] = 0.0;
            r
        });
        f.add(&format!("G-J.rank.ell.without_{k}"), rank(&without))?;
    }
    let straight = degrees(poses::ELBOW_STRAIGHT);
    f.add("G-J.elbow_straight.p", vec(&arm.tip(&straight).p, 4))?;
    let only = degrees(poses::ELBOW_ONLY);
    f.pin(
        "G-J.elbow_only.reach",
        fx(
            norm(sub(
                arm.joint(&only, model::WRIST).0,
                arm.joint(&only, model::SHOULDER).0,
            )),
            4,
        ),
        "0.5800",
        "a right triangle with legs 0.42 and 0.40: inside the reach of 0.82",
    )?;
    f.pin(
        "G-J.elbow_straight.push_along_arm",
        fx(arm.push_along_arm(&straight), 4),
        "0.0000",
        "with the elbow straight no joint moves the wrist along the arm",
    )?;
    f.add(
        "G-J.swing.push_along_arm",
        fx(arm.push_along_arm(&degrees(poses::SWING)), 4),
    )?;
    f.add("G-J.blowup.pull", fx(shown::PULL, 1))?;
    for bend in shown::BENDS {
        f.add(
            &format!("G-J.blowup.{}", whole(bend)?),
            fx(arm.elbow_speed_to_pull(bend, shown::PULL), 2),
        )?;
    }
    let limit = model::SPEED_DEG_S[model::ELBOW];
    f.add("G-J.blowup.limit", whole(limit)?)?;
    // the bend at which the elbow's speed meets its limit (it falls as the bend grows)
    let (mut lo, mut hi) = (10.0, 30.0);
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        if arm.elbow_speed_to_pull(mid, shown::PULL) > limit {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    f.add("G-J.blowup.bend_at_limit", fx(lo, 1))?;
    f.add(
        "G-J.blowup.1.turns_per_s",
        fx(arm.elbow_speed_to_pull(1.0, shown::PULL) / 360.0, 2),
    )?;
    let swing = degrees(poses::SWING);
    f.add("G-J.elbow.radius", fx(arm.elbow_radius(&swing), 4))?;
    f.add("G-J.spare", N - rank(&arm.jacobian(&swing)))?;
    let js = arm.jacobian(&swing);
    let still = twist_from(&js, &still_speeds(&js));
    f.add(
        "G-J.swing.null_twist",
        fx(still.iter().map(|x| x.abs()).fold(0.0, f64::max), 4),
    )?;
    f.add("G-J.swing.p", vec(&arm.tip(&swing).p, 4))?;
    Ok(())
}

/// The straight screw from the start pose to the goal, in the room's frame.
pub fn path(arm: &Arm) -> (Pose, Pose, model::Screw) {
    let start = arm.tip(&degrees(poses::START));
    let goal = goal(arm);
    let screw = screw_of(&goal.then(&start.undo()));
    (start, goal, screw)
}

fn steps_and_paths(f: &mut Facts, arm: &Arm) -> Result<(), String> {
    let (start, goal, screw) = path(arm);
    f.add("G-I.start.q", ints(&poses::START)?)?;
    f.add("G-I.start.p", vec(&start.p, 4))?;
    f.add("G-I.start.R", grid(&start.r)?)?;
    f.add("G-I.goal.p", vec(&goal.p, 4))?;
    f.add("G-I.goal.R", grid(&goal.r)?)?;
    // the error at the start, as the six numbers chapter 13 writes out: the
    // twist, in the gripper's own frame, that reaches the goal in one second
    f.add("G-I.error.twist", vec(&log6(&start.undo().then(&goal)), 3))?;
    let run = arm.solve(&degrees(poses::START), &goal, model::TOL_SPIN, model::TOL_SLIDE);
    f.add("G-I.ik.steps", run.steps())?;
    f.add("G-I.ik.dist_mm", vec(&run.dist_mm, 2))?;
    f.add("G-I.ik.angle_deg", vec(&run.angle_deg, 4))?;
    f.add("G-I.ik.q_deg", vec(&run.q.map(|x| x / DEG), 2))?;
    f.add(
        "G-I.ik.within_limits",
        flag(run.q.iter().zip(model::LIMITS_DEG).all(|(x, lim)| x.abs() / DEG <= lim)),
    )?;
    f.add("G-I.ik.tolerances", format!("{},{}", model::TOL_SPIN, model::TOL_SLIDE))?;
    f.add("G-I.screw.angle", fx(screw.angle / DEG, 4))?;
    f.add("G-I.screw.axis", vec(&screw.axis, 4))?;
    f.add("G-I.screw.point", vec(&screw.point[..2], 4))?;
    f.add("G-I.screw.slide", fx(screw.slide, 4))?;
    let blend = start.r.map(|row| row.map(|x| x / 2.0));
    let blend: M3 = std::array::from_fn(|i| std::array::from_fn(|k| blend[i][k] + goal.r[i][k] / 2.0));
    f.pin(
        "G-I.blend.det",
        fx(det(&blend), 4),
        "0.5000",
        "cos squared of half a quarter turn",
    )?;
    // a quarter of the way: the blend repaired to the nearest turn, against the screw
    f.pin(
        "G-I.blend.quarter_turned",
        fx(model::blend_turned(screw.angle, 0.25) / DEG, 4),
        "18.4349",
        "atan of one third, in degrees",
    )?;
    let bow = (0..=100)
        .map(|k| {
            let s = f64::from(k) / 100.0;
            let on_screw = along_screw(&screw, s).then(&start).p;
            let on_line = add(start.p, scale(sub(goal.p, start.p), s));
            norm(sub(on_screw, on_line))
        })
        .fold(0.0, f64::max);
    f.add("G-I.bow", fx(bow, 4))?;
    let mid = along_screw(&screw, 0.5).then(&start);
    f.add("G-I.screw.mid.det", fx(det(&mid.r), 4))?;
    f.add(
        "G-I.screw.mid.turned",
        fx(angle_axis(&mul(&transpose(&start.r), &mid.r)).0 / DEG, 4),
    )?;
    let quarter = along_screw(&screw, 0.25).then(&start);
    f.add(
        "G-I.screw.quarter_turned",
        fx(angle_axis(&mul(&transpose(&start.r), &quarter.r)).0 / DEG, 4),
    )?;
    Ok(())
}

/// A body spinning at one radian a second about its diagonal, stepped two
/// ways: the spin added onto the nine numbers, and the exponential of the
/// spin applied. The determinants after the last step, and how much wider
/// the first body has grown across its axis.
pub fn drift() -> (f64, f64, f64) {
    let (steps, dt) = shown::DRIFT;
    let spin = hat(unit([1.0, 1.0, 1.0]));
    let one = turn(unit([1.0, 1.0, 1.0]), dt);
    let (mut added, mut stepped) = (I3, I3);
    for _ in 0..steps {
        let d = mul(&spin, &added);
        added = std::array::from_fn(|i| std::array::from_fn(|k| added[i][k] + dt * d[i][k]));
        stepped = mul(&one, &stepped);
    }
    let across = unit([1.0, -1.0, 0.0]);
    (det(&added), det(&stepped), norm(apply(&added, across)))
}

fn coda(f: &mut Facts) -> Result<(), String> {
    f.add("G-C.g1.joints", cited::G1_JOINTS)?;
    f.add("G-C.g1.velocity", 6 + cited::G1_JOINTS)?;
    f.add("G-C.g1.stored", 3 + 4 + cited::G1_JOINTS)?;
    let (steps, dt) = shown::DRIFT;
    let (added, stepped, width) = drift();
    f.add("G-C.drift.steps", steps)?;
    f.add("G-C.drift.dt", fx(dt, 2))?;
    f.pin(
        "G-C.drift.add",
        fx(added, 4),
        "1.1052",
        "(1 + 0.0001) to the power 1000",
    )?;
    f.pin("G-C.drift.exp", fx(stepped, 4), "1.0000", "a turn keeps volume")?;
    f.add("G-C.drift.width", fx(width, 4))?;
    Ok(())
}

/// The axis a turn leaves in place, also for a half turn.
fn fixed_axis(r: &M3) -> Option<V3> {
    let (angle, axis) = angle_axis(r);
    if norm(axis) > 0.0 {
        return Some(axis);
    }
    if angle < 1e-9 {
        return Some(Z);
    }
    // a half turn: R + I has the axis as its only direction
    (0..3)
        .map(|c| [r[0][c] + I3[0][c], r[1][c] + I3[1][c], r[2][c] + I3[2][c]])
        .max_by(|a, b| norm(*a).total_cmp(&norm(*b)))
        .map(unit)
}

fn sweep() -> Vec<(V3, f64, M3)> {
    CUBE_AXES
        .iter()
        .flat_map(|axis| SWEEP_DEG.iter().map(move |a| (*axis, *a, turn(unit(*axis), a * DEG))))
        .collect()
}

/// The sweep with no turn counted once (it is the same turn for every axis).
fn distinct_turns() -> Vec<(V3, f64, M3)> {
    let mut seen_nothing = false;
    sweep()
        .into_iter()
        .filter(|(_, a, _)| {
            let first = *a != 0.0 || !seen_nothing;
            seen_nothing |= *a == 0.0;
            first
        })
        .collect()
}

fn edges(f: &mut Facts) -> Result<(), String> {
    let turns = sweep();
    // a turn after a turn is one turn about one axis
    let (mut held, mut total) = (0, 0);
    for i in (0..turns.len()).step_by(5) {
        for k in (0..turns.len()).step_by(7) {
            let r = mul(&turns[i].2, &turns[k].2);
            total += 1;
            let rigid = gap(&mul(&transpose(&r), &r), &I3) < 1e-9 && (det(&r) - 1.0).abs() < 1e-9;
            let one_axis = fixed_axis(&r).is_some_and(|a| norm(sub(apply(&r, a), a)) < 1e-9);
            held += usize::from(rigid && one_axis);
        }
    }
    f.pin("G-X1.one_turn", format!("{held}/{total}"), "192/192", "Euler's theorem")?;
    // which pairs of a cube's turns do not care about order
    let cube = model::cube_turns();
    let times = |a: &[[i32; 3]; 3], b: &[[i32; 3]; 3]| -> [[i32; 3]; 3] {
        std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| a[i][k] * b[k][j]).sum()))
    };
    let swaps = cube
        .iter()
        .flat_map(|a| cube.iter().map(move |b| (a, b)))
        .filter(|(a, b)| times(a, b) == times(b, a))
        .count();
    f.add("G-X2.cube_turns", cube.len())?;
    f.pin(
        "G-X2.order_free_pairs",
        format!("{swaps}/{}", cube.len() * cube.len()),
        "120/576",
        "24 turns times the 5 classes of the cube's turns",
    )?;
    let (a, b) = (tilt_turn(40.0 * DEG), tilt_turn(70.0 * DEG));
    f.add("G-X2.same_axis_swap", flag(gap(&mul(&a, &b), &mul(&b, &a)) < 1e-9))?;
    let (a, b) = (tilt_turn(PI), roll_turn(PI));
    f.add("G-X2.half_turns_swap", flag(gap(&mul(&a, &b), &mul(&b, &a)) < 1e-9))?;
    // every pose is one screw
    let slides: [V3; 4] = [[0.0, 0.0, 0.0], [0.3, 0.0, 0.0], [0.2, -0.1, 0.4], [-0.5, 0.25, 0.1]];
    let (mut held, mut total) = (0, 0);
    let distinct = distinct_turns();
    for (_, _, r) in &distinct {
        for p in slides {
            let t = Pose { r: *r, p };
            total += 1;
            held += usize::from(along_screw(&screw_of(&t), 1.0).gap(&t) < 1e-9);
        }
    }
    f.pin(
        "G-X3.one_screw",
        format!("{held}/{total}"),
        "264/264",
        "Chasles's theorem: 66 different turns times 4 slides",
    )?;
    f.add(
        "G-X3.pure_slide.turn",
        fx(screw_of(&Pose { r: I3, p: slides[1] }).angle / DEG, 4),
    )?;
    // a blend of two turns is a turn only when the two are the same
    let (mut held, mut total) = (0, 0);
    for (_, a, r) in &distinct {
        let blend: M3 = std::array::from_fn(|i| std::array::from_fn(|k| (I3[i][k] + r[i][k]) / 2.0));
        let d = det(&blend);
        total += 1;
        held += usize::from((d - (a * DEG / 2.0).cos().powi(2)).abs() < 1e-12 && ((d < 1.0 - 1e-12) == (*a != 0.0)));
    }
    f.pin(
        "G-X4.blend",
        format!("{held}/{total}"),
        "66/66",
        "det = cos squared of half the angle",
    )?;
    // on a floor, a steady velocity is a turn about a point, or a slide
    let (mut held, mut total) = (0, 0);
    let times_s = [0.5, 1.0, 2.5, 4.0, 7.0];
    for vx in [-0.5, 0.0, 0.5] {
        for vy in [-0.5, 0.0, 0.5] {
            for spin_deg in [-60.0, -30.0, 0.0, 30.0, 60.0] {
                total += 1;
                let spin: f64 = spin_deg * DEG;
                let ok = times_s.iter().all(|&t| {
                    let (x, y, _) = planar_exp(vx, vy, spin, t);
                    if spin == 0.0 {
                        (x - vx * t).abs() < 1e-9 && (y - vy * t).abs() < 1e-9
                    } else {
                        let (cx, cy) = planar_centre(vx, vy, spin);
                        ((x - cx).hypot(y - cy) - cx.hypot(cy)).abs() < 1e-9
                    }
                });
                held += usize::from(ok);
            }
        }
    }
    f.pin(
        "G-X5.floor",
        format!("{held}/{total}"),
        "45/45",
        "a turn about a point, or a slide",
    )?;
    // every order of three rings loses a direction at some middle angle: the
    // twelve orders with neighbouring rings about different axes
    let named = [("x", X), ("y", Y), ("z", Z)];
    let (mut locked, mut total) = (0, 0);
    let mut where_flat = std::collections::BTreeMap::new();
    for (na, a) in named {
        for (nb, b) in named {
            for (nc, c) in named {
                if na == nb || nb == nc {
                    continue;
                }
                total += 1;
                let flat: Vec<String> = (0..=180)
                    .filter(|m| model::rings_span(a, b, c, f64::from(*m) * DEG) < 1e-9)
                    .map(|m| m.to_string())
                    .collect();
                locked += usize::from(!flat.is_empty());
                where_flat.insert(format!("{na}{nb}{nc}"), flat.join(","));
            }
        }
    }
    f.pin(
        "G-X6.ring_orders",
        format!("{locked}/{total}"),
        "12/12",
        "no order of three rings is free of a bad spot",
    )?;
    f.pin(
        "G-X6.bad_angle.zyx",
        &where_flat["zyx"],
        "90",
        "the video's rings lock at a quarter turn",
    )?;
    f.pin(
        "G-X6.bad_angle.zyz",
        &where_flat["zyz"],
        "0,180",
        "rings with a repeated axis lock when straight",
    )?;
    Ok(())
}

fn quizzes(f: &mut Facts, arm: &Arm) -> Result<(), String> {
    let quarter = PI / 2.0;
    let (tw, wt) = (tilt_then_roll(quarter), roll_then_tilt(quarter));
    let (a, b) = shown::LOCKED;
    let (h1, h2) = shown::HEADINGS;
    let spun = turn(unit(shown::DIAGONAL), shown::DIAGONAL_RATE * DEG);
    let (v, spin_deg, t) = shown::BASE;
    let (x, y, _) = planar_exp(v, 0.0, spin_deg * DEG, t);
    let (before, after) = (
        arm.skeleton(&degrees(poses::HOME)),
        arm.skeleton(&degrees(poses::ELBOW_ONLY)),
    );
    let past_elbow = (1..=N + 1).all(|i| (norm(sub(before[i], after[i])) > 1e-9) == (i > model::ELBOW + 1));
    let ell = degrees(poses::ELL);
    let best = (0..N)
        .max_by(|&i, &k| arm.lever(&ell, i).total_cmp(&arm.lever(&ell, k)))
        .unwrap_or(0);
    let (start, goal, _) = path(arm);
    let blend: M3 = std::array::from_fn(|i| std::array::from_fn(|k| (start.r[i][k] + goal.r[i][k]) / 2.0));
    let run = arm.solve(&degrees(poses::START), &goal, model::TOL_SPIN, model::TOL_SLIDE);
    let answers: [String; 15] = [
        if gap(&tw, &wt) < 1e-9 { "same" } else { "different" }.into(),
        ints(&[angle_axis(&tw).0 / DEG])?,
        if gap(&ypr(a), &ypr(b)) < 1e-9 {
            "same"
        } else {
            "different"
        }
        .into(),
        ints(&[halfway_deg(h1, h2).abs()])?,
        if norm(sub(apply(&spun, X), Y)) < 1e-9 {
            "next"
        } else {
            "other"
        }
        .into(),
        if x.abs() < 1e-9 && y > 1.0 { "beside" } else { "other" }.into(),
        (3 + 3).to_string(),
        if past_elbow { "past" } else { "other" }.into(),
        if arm.tip(&degrees(poses::QUARTER)).p[0].abs() < 1e-9 {
            "side"
        } else {
            "other"
        }
        .into(),
        (best + 1).to_string(),
        if rank(&arm.jacobian(&degrees(poses::ELBOW_STRAIGHT))) < 6 {
            "lost"
        } else {
            "all"
        }
        .into(),
        if N - rank(&arm.jacobian(&degrees(poses::SWING))) >= 1 {
            "moves"
        } else {
            "fixed"
        }
        .into(),
        run.steps().to_string(),
        if (det(&blend) - 1.0).abs() < 1e-9 {
            "pose"
        } else {
            "not"
        }
        .into(),
        if angle_axis(&leftover(DEG)).0 > 0.0 {
            "differ"
        } else {
            "same"
        }
        .into(),
    ];
    for (quiz, answer) in QUIZZES.iter().zip(answers) {
        f.add(&format!("G-Q.{}.seconds", quiz.line), quiz.seconds)?;
        f.add(&format!("G-Q.{}.answer", quiz.line), answer)?;
    }
    Ok(())
}

fn sources(f: &mut Facts) -> Result<(), String> {
    f.add("G-L.kuka.reach_mm", cited::KUKA_REACH_MM)?;
    f.add("G-L.kuka.payload_kg", cited::KUKA_PAYLOAD_KG)?;
    f.add("G-L.kuka.axes", cited::KUKA_AXES)?;
    f.add("G-L.kmr.speed_kmh", cited::KMR_SPEED_KMH)?;
    f.add("G-L.kmr.speed_m_s", fx(cited::KMR_SPEED_KMH / 3.6, 1))?;
    f.add("G-L.apollo.lamp_deg", cited::APOLLO_LAMP_DEG)?;
    f.add("G-L.apollo.lost_deg", cited::APOLLO_LOST_DEG)?;
    f.add("G-L.apollo.gimbals", cited::APOLLO_GIMBALS)?;
    f.add("G-L.euler.year", cited::EULER_YEAR)?;
    f.add("G-L.mozzi.year", cited::MOZZI_YEAR)?;
    f.add("G-L.chasles.year", cited::CHASLES_YEAR)?;
    f.add("G-L.rodrigues.year", cited::RODRIGUES_YEAR)?;
    f.add("G-L.lie.year", cited::LIE_YEAR)?;
    f.add("G-L.lie.born", cited::LIE_BORN)?;
    f.add("G-L.lie.died", cited::LIE_DIED)?;
    f.add("G-L.brockett.year", cited::BROCKETT_YEAR)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cross;

    #[test]
    fn every_pin_holds() {
        facts().unwrap().self_check().unwrap();
    }

    /// The screw the video draws from start to goal is a quarter turn about
    /// a vertical axis with a slide straight down.
    #[test]
    fn the_path_is_a_corkscrew_down() {
        let arm = Arm::iiwa14();
        let (start, goal, screw) = path(&arm);
        assert!((screw.angle / DEG - 90.0).abs() < 1e-9);
        assert!(norm(sub(screw.axis, Z)) < 1e-9);
        assert!((screw.slide - (goal.p[2] - start.p[2])).abs() < 1e-9);
        assert!(along_screw(&screw, 1.0).then(&start).gap(&goal) < 1e-9);
    }

    /// Edge: the screw's point is the one a quarter turn about the vertical
    /// carries to itself plus the slide's floor part.
    #[test]
    fn the_tossed_book_s_axis() {
        let s = screw_of(&toss());
        let moved = add(apply(&turn(Z, PI / 2.0), s.point), shown::TOSS_SLIDE);
        assert!(norm(sub([moved[0], moved[1], 0.0], [s.point[0], s.point[1], 0.0])) < 1e-12);
    }

    /// Narration truth (13_5): each step's error is far below the last.
    #[test]
    fn newton_s_steps_shrink_the_error() {
        let arm = Arm::iiwa14();
        let run = arm.solve(&degrees(poses::START), &goal(&arm), model::TOL_SPIN, model::TOL_SLIDE);
        assert_eq!(run.steps(), 3);
        assert!(run.dist_mm.windows(2).all(|w| w[1] < w[0] / 5.0), "{:?}", run.dist_mm);
        assert!(run.dist_mm[3] < 0.1);
    }

    /// Narration truth (11_5): the elbow's speed passes its limit between a
    /// bend of 30 degrees and one of 10.
    #[test]
    fn the_elbow_s_speed_passes_its_limit() {
        let arm = Arm::iiwa14();
        let limit = model::SPEED_DEG_S[model::ELBOW];
        assert!(arm.elbow_speed_to_pull(30.0, shown::PULL) < limit);
        assert!(arm.elbow_speed_to_pull(10.0, shown::PULL) > limit);
    }

    /// Edge (11_6): straightening the wrist lines up two of its axes, yet
    /// the arm with its spare joint still reaches all six directions.
    #[test]
    fn a_straight_wrist_lines_up_two_axes() {
        let arm = Arm::iiwa14();
        let q = degrees(poses::ELL);
        let (a5, a7) = (arm.joint(&q, 4).1, arm.joint(&q, 6).1);
        assert!(norm(cross(a5, a7)) < 1e-12);
        assert_eq!(rank(&arm.jacobian(&q)), 6);
    }

    #[test]
    fn formats_match_the_oracle_s() {
        assert_eq!(fx(-0.00001, 4), "0.0000");
        assert_eq!(fx(-0.5, 4), "-0.5000");
        assert_eq!(fx(1.0 / 3.0, 4), "0.3333");
        assert_eq!(vec(&[1.0, -2.5], 1), "1.0,-2.5");
    }
}
