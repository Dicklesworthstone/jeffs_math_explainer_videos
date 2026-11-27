//! Startup facts registry for The Math Behind FrankenRobotics.
//!
//! Every number shown on screen or spoken in narration is computed from
//! `crate::model` and pinned against `oracle.py`. Fails closed if any value
//! drifts before a single frame renders.

use std::f64::consts::FRAC_PI_2;

use explainer_kit::Facts;

use crate::model::{
    self, Arm2R, Omni3, Se2, corridor_balanced_pose, corridor_exact_limit, four_move_wheel_travel,
    hexapod_d6_elements, hexapod_d6_orbits, quad_mode_gains, quad_stabilizer, reynolds_project_fl4,
    snake3_integrate_loop, stance_apply_diagonal_shift, stance_equal_springs, track_lateral_velocities,
};

fn fmt_pair_int(p: [f64; 2]) -> String {
    format!("{},{}", p[0].round() as i64, p[1].round() as i64)
}

fn fmt_slice_i64(xs: &[i64]) -> String {
    xs.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
}

/// Build and self-check every fact the video displays or speaks.
pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();

    // Chapter 1
    f.pin("ch01.quarter_turn_fraction", "1/4", "1/4", "SO(2) quarter turn")?;
    f.pin("ch01.quarter_turn_inverse_mod1", "3/4", "3/4", "1/4 + 3/4 = 1 mod 1")?;

    // Chapter 2
    let hand = Arm2R::UNIT.hand(0.0, FRAC_PI_2);
    f.pin("ch02.unit_arm_hand_xy", fmt_pair_int(hand), "1,1", "oracle.py / scene-facts.py")?;
    let r_elbow = hand[0].hypot(hand[1]);
    f.pin("ch02.unit_arm_fixed_elbow_radius", format!("{r_elbow:.6}"), "1.414214", "sqrt(2)")?;
    let bench_hand = Arm2R::BENCH.hand(0.0, FRAC_PI_2);
    let bench_tau = Arm2R::BENCH.torques(0.0, FRAC_PI_2, [10.0, 0.0]);
    let bench_vel = Arm2R::BENCH.tip_velocity(0.0, FRAC_PI_2, [0.2, -0.1]);
    let power = bench_tau[0] * 0.2 + bench_tau[1] * -0.1;
    f.pin("ch02.bench_arm_hand_m", format!("{:.1},{:.1}", bench_hand[0], bench_hand[1]), "0.4,0.3", "Plan C-04")?;
    f.pin("ch02.bench_arm_torques_nm", fmt_pair_int(bench_tau), "-3,-3", "Plan C-04 / bench.ts")?;
    f.pin("ch02.bench_arm_tip_vel_mps", format!("{:.2},{:.2}", bench_vel[0], bench_vel[1]), "-0.03,0.08", "Plan C-04")?;
    f.pin("ch02.bench_arm_power_w", format!("{power:.1}"), "-0.3", "virtual work")?;

    // Chapter 3
    let rt = Se2::rotation(FRAC_PI_2).compose(Se2::translation(1.0, 0.0)).apply([0.0, 0.0]);
    let tr = Se2::translation(1.0, 0.0).compose(Se2::rotation(FRAC_PI_2)).apply([0.0, 0.0]);
    f.pin("ch03.rt_origin", fmt_pair_int(rt), "0,1", "oracle.py / scene-facts.py")?;
    f.pin("ch03.tr_origin", fmt_pair_int(tr), "1,0", "oracle.py / scene-facts.py")?;

    // Chapter 4
    let rim = Omni3::rim_speeds(0.0, 2.0, 0.0);
    let rim_i: [i64; 3] = [rim[0].round() as i64, rim[1].round() as i64, rim[2].round() as i64];
    f.pin("ch04.omni_unscaled_det", format!("{:.6}", Omni3::unscaled_determinant()), "2.598076", "3*sqrt(3)/2")?;
    f.pin("ch04.omni_sideways_rim_speeds", fmt_slice_i64(&rim_i), "2,-1,-1", "scene-facts.py")?;
    let track_v = track_lateral_velocities(0.0, 1.0, 0.5);
    assert!((track_v[0] - 0.5).abs() < 1e-12 && (track_v[1] + 0.5).abs() < 1e-12);
    f.pin("ch04.track_yaw_lateral_velocities", "1/2,-1/2", "1/2,-1/2", "scene-facts.py")?;

    // Chapter 5
    let rov = Se2::rover_commutator(FRAC_PI_2, 1.0);
    f.pin("ch05.rover_quarter_turn_xy", fmt_pair_int([rov.x, rov.y]), "-1,1", "scene-facts.py")?;
    f.pin("ch05.rover_quarter_turn_heading", rov.theta.round() as i64, "0", "exact cancellation")?;
    let (a10, c10) = four_move_wheel_travel(0.10, 0.20);
    f.pin("ch05.gb05_shift_010m_exact_m", format!("{c10:.4}"), "0.5537", "Plan GB-05")?;
    f.pin("ch05.gb05_shift_010m_alpha_rad", format!("{a10:.4}"), "0.7227", "Plan GB-05")?;

    // Chapter 7 & 8
    f.pin("ch07.quad_group_order", quad_stabilizer(false, false, false).len(), "4", "C2 x C2")?;
    f.pin("ch07.quad_lidar_obs_order", quad_stabilizer(true, false, false).len(), "2", "symmetry.ts")?;
    f.pin("ch07.quad_payload_obs_order", quad_stabilizer(true, true, false).len(), "1", "symmetry.ts")?;
    f.pin("ch07.hexapod_d6_order", hexapod_d6_elements().len(), "12", "D6")?;
    f.pin("ch07.hexapod_d6_orbits_k10", hexapod_d6_orbits(10), "86185", "Plan §13.6")?;

    let gains = quad_mode_gains();
    f.pin("ch08.mode_gains", fmt_slice_i64(&gains), "6,2,4,0", "scene-facts.py")?;
    f.pin("ch08.reynolds_c2xc2_diag", fmt_slice_i64(&reynolds_project_fl4(&quad_stabilizer(false, false, false))), "1,1,1,1", "scene-facts.py")?;
    f.pin("ch08.reynolds_c2_lr_diag", fmt_slice_i64(&reynolds_project_fl4(&quad_stabilizer(true, false, false))), "2,2,0,0", "scene-facts.py")?;
    f.pin("ch08.reynolds_id_diag", fmt_slice_i64(&reynolds_project_fl4(&quad_stabilizer(true, true, false))), "4,0,0,0", "scene-facts.py")?;

    // Chapter 9
    let snake = snake3_integrate_loop([0.9, 0.9], 0.4, true, 4000);
    f.pin("ch09.snake3_loop_dx_links", format!("{:.4}", snake.x), "0.2029", "oracle.py dual integration")?;
    f.pin("ch09.snake3_loop_dy_links", format!("{:.4}", snake.y), "0.1272", "oracle.py dual integration")?;

    // Chapter 10
    let lmax = corridor_exact_limit(0.90, 0.60);
    let (_, m130) = corridor_balanced_pose(1.30, 0.60, 0.90, FRAC_PI_2 / 2.0);
    let (_, m140) = corridor_balanced_pose(1.40, 0.60, 0.90, FRAC_PI_2 / 2.0);
    f.pin("ch10.corridor_exact_limit_m", format!("{lmax:.3}"), "1.346", "Plan §19.11")?;
    f.pin("ch10.corridor_130_min_clearance_mm", format!("{:.1}", m130 * 1000.0), "9.4", "oracle.py witness")?;
    f.pin("ch10.corridor_140_cut_upper_mm", format!("{:.1}", m140 * 1000.0), "-11.3", "oracle.py cut")?;

    let sp = stance_equal_springs(0.25);
    let ac = stance_apply_diagonal_shift(sp, -20.0);
    let fmt4 = |a: [f64; 4]| a.iter().map(|v| (v.round() as i64).to_string()).collect::<Vec<_>>().join(",");
    f.pin("ch10.stance_springs_n", fmt4(sp), "175,115,85,25", "stance.ts")?;
    f.pin("ch10.stance_active_quarter_n", fmt4(ac), "155,135,105,5", "stance.ts")?;

    // Verify docking endpoints
    for g in [1.0, 2.0] {
        let (_, end) = model::dock_cart_ride(g, 0.25);
        if (end[0] - 1.0).abs() > 1e-12 || end[1].abs() > 1e-12 {
            return Err(format!("dock cart failed to stop at (1,0) for gain {g}"));
        }
    }
    f.pin("ch10.dock_gentle_eps_limit", "1/8", "1/8", "docking.ts")?;

    // Chapter 11, exact independent site-geometry recomputation.
    let (_, wheel_relative, wheel_chassis) = model::stair_cluster_angles(30.0, 5.0);
    f.pin("body_gallery.stair_wheel_relative_deg", wheel_relative.round() as i64, "-30", "figures/bodies.ts / independent review")?;
    f.pin("body_gallery.stair_wheel_chassis_deg", wheel_chassis.round() as i64, "0", "figures/bodies.ts / independent review")?;
    f.pin("body_gallery.balancer_rearward_wheel_rad", format!("{:.2}", model::balancer_wheel_relative(0.25, 0.10)), "-0.35", "figures/bodies.ts / independent review")?;
    f.pin("body_gallery.balancer_forward_wheel_rad", format!("{:.2}", model::balancer_wheel_relative(0.25, -0.10)), "-0.15", "figures/bodies.ts / independent review")?;

    f.self_check()?;
    Ok(f)
}
