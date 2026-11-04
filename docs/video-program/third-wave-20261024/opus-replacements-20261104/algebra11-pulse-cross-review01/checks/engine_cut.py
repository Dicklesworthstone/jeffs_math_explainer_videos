"""The pulse's arithmetic, on functions cut verbatim from the pinned engine (fmn pin a399d62), std only.

Usage from the repo root:  python3 -I -B engine_cut.py SCRATCH_DIR
Cuts smooth, there_and_back, powi, Placement (whole file), interpolate_placement, the Straight path and frames_covering
out of the cargo checkout, compiles them with rustc into SCRATCH_DIR and runs them. This is not the proposed native
test and it runs no Stage, no Scene and no animation object: it checks the numbers those would be built from.
Nothing is written in the repo. No picture or listening claim.
"""
import os
import re
import subprocess
import sys

CRATES = os.path.expanduser("~/.cargo/git/checkouts/franken_manim-6641d0e09642f63e/a399d62/crates/")
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
fails = 0


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def read(path):
    return open(CRATES + path, encoding="utf-8").read()


def cut(source, signature):
    at = source.index(signature)
    start = source.rfind("\n", 0, at) + 1
    depth = 0
    for i in range(at, len(source)):
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
            if depth == 0:
                return source[start : i + 1]
    raise ValueError(signature)


rate = read("fmn-core/src/rate.rs")
dmath = read("fmn-dmath/src/pow.rs")
placement = read("fmn-mobject/src/placement.rs")
motion = read("fmn-anim/src/motion_path.rs")
clock = read("fmn-anim/src/clock.rs")

straight = "Self::Straight => std::array::from_fn(|i| (1.0 - alpha) * start[i] + alpha * end[i]),"
ck(straight in motion, "the Straight path is (1 - a) s + a e, as the source states")
ck("let raw = time.to_f64() / self.run_time;" in clock and "alpha: raw.clamp(0.0, 1.0)," in clock, "a frame's alpha is (k / fps) / run_time, clamped")
ck("self.frames as f64 / f64::from(self.fps)" in clock, "k / fps is computed in f64")

placement_body = placement.replace("use fmn_core::types::Vec3;\n", "").replace("use fmn_geom::Mat3;\n", "")
placement_body = placement_body[: placement_body.index("#[cfg(test)]")]
placement_body = "\n".join(line for line in placement_body.split("\n") if not line.startswith("//!"))
decompose = cut(clock, "fn decompose(")
main = (
    "#![allow(dead_code, unused)]\n"
    "type Vec3 = [f64; 3];\ntype Mat3 = [[f64; 3]; 3];\n"
    "#[derive(Debug)] enum ClockError { NonFiniteRunTime, RunTimeTooLong }\n"
    "mod fmn_dmath { " + cut(dmath, "pub fn powi(") + " }\n"
    + cut(rate, "pub fn smooth(") + "\n" + cut(rate, "pub fn there_and_back(") + "\n"
    + decompose + "\n" + cut(clock, "fn frames_covering(") + "\n"
    + placement_body + "\n"
    + "#[derive(Clone, Copy)] enum PathFunc { Straight }\n"
    + "impl PathFunc { fn eval(&self, start: Vec3, end: Vec3, alpha: f64) -> Vec3 { match *self { " + straight + " } } }\n"
    + cut(motion, "fn interpolate_placement(").replace("pub(super) fn", "fn") + "\n"
    + r"""
fn alpha(k: i64, fps: u32, run_time: f64) -> f64 { (k as f64 / f64::from(fps) / run_time).clamp(0.0, 1.0) }
fn pulse_target(start: Placement, factor: f64, pivot: Vec3) -> Placement {
    let linear = [[factor, 0.0, 0.0], [0.0, factor, 0.0], [0.0, 0.0, factor]];
    Placement::about(linear, pivot).compose(start)
}
fn main() {
    for (rt, fps) in [(0.6, 60), (0.6, 30), (0.6, 10), (0.5, 60)] {
        println!("frames {rt} {fps} {}", frames_covering(rt, fps).unwrap());
    }
    println!("alpha18 {:?} alpha36 {:?}", alpha(18, 60, 0.6), alpha(36, 60, 0.6));
    println!("rate0 {:?} rate.5 {:?} rate1 {:?}", there_and_back(0.0), there_and_back(0.5), there_and_back(1.0));
    let n = 36;
    let symmetric = (1..n).all(|k| (there_and_back(alpha(k, 60, 0.6)) - there_and_back(alpha(n - k, 60, 0.6))).abs() < 1e-12);
    println!("symmetric {symmetric}");
    let rises = (1..18).all(|k| there_and_back(alpha(k, 60, 0.6)) < there_and_back(alpha(k + 1, 60, 0.6)));
    println!("rises {rises}");
    println!("first_rate {:.6} frame17_rate {:.6}", there_and_back(alpha(1, 60, 0.6)), there_and_back(alpha(17, 60, 0.6)));

    // Start placements of the kind a token carries: moved, scaled, moved again, about varied pivots.
    let mut cases = 0;
    let (mut end_exact, mut peak_exact, mut pivot_held, mut ratio_ok) = (true, true, true, true);
    let (mut smooth_stays, mut smooth_half) = (true, true);
    for tx in [-5.4, -3.2, -0.15, 0.0, 1.0 / 3.0, 4.7] {
        for ty in [-3.15, -1.1, 0.0, 0.1, 2.45] {
            for s in [1.0, 0.37, 1.25, 1.7] {
                for (px, py) in [(0.0, 0.0), (tx, ty), (tx + 0.013, ty - 0.21)] {
                    cases += 1;
                    let start = Placement::new([[s, 0.0, 0.0], [0.0, s, 0.0], [0.0, 0.0, s]], [tx, ty, 0.0]);
                    let pivot = [px, py, 0.0];
                    let target = pulse_target(start, 1.25, pivot);
                    let at = |rate_value: f64| interpolate_placement(start, target, rate_value, PathFunc::Straight);
                    end_exact &= at(there_and_back(alpha(36, 60, 0.6))) == start;
                    peak_exact &= at(there_and_back(alpha(18, 60, 0.6))) == target;
                    let object_pivot = [(px - tx) / s, (py - ty) / s, 0.0];
                    for k in 1..=36 {
                        let p = at(there_and_back(alpha(k, 60, 0.6)));
                        let world = p.apply_point(object_pivot);
                        pivot_held &= (world[0] - px).abs() < 1e-12 && (world[1] - py).abs() < 1e-12;
                        let want = s * (1.0 + 0.25 * there_and_back(alpha(k, 60, 0.6)));
                        ratio_ok &= (p.linear()[0][0] - want).abs() < 1e-12;
                    }
                    // Control: the engine's default rate, as if rate_func were left out.
                    let default_end = interpolate_placement(start, target, smooth(alpha(36, 60, 0.6)), PathFunc::Straight);
                    smooth_stays &= default_end == target;
                    let default_mid = interpolate_placement(start, target, smooth(alpha(18, 60, 0.6)), PathFunc::Straight);
                    smooth_half &= (default_mid.linear()[0][0] - s * 1.125).abs() < 1e-12;
                }
            }
        }
    }
    println!("cases {cases}");
    println!("end_exact {end_exact} peak_exact {peak_exact} pivot_held {pivot_held} ratio_ok {ratio_ok}");
    println!("default_rate_stays_swollen {smooth_stays} default_rate_only_half_at_midpoint {smooth_half}");
}
"""
)
path = os.path.join(S, "engine_cut.rs")
with open(path, "w", encoding="utf-8") as out:
    out.write(main)
binary = os.path.join(S, "engine_cut")
r = subprocess.run(["rustc", "--edition", "2024", "-o", binary, path], capture_output=True, text=True)
ck(r.returncode == 0, "the cut functions compile", r.stderr[-1500:].replace(S, "<scratch>") if r.returncode else "")
if r.returncode:
    print("RESULT stopped: nothing to run")
    sys.exit(1)
output = subprocess.run([binary], capture_output=True, text=True, check=True).stdout
print(output.rstrip())


def has(pattern):
    return re.search(pattern, output) is not None


ck(has(r"frames 0\.6 60 36\n"), "0.6 s at 60 fps is 36 frames")
ck(has(r"frames 0\.5 60 30\n"), "control: 0.5 s would be 30 frames")
ck(has(r"alpha18 0\.5 alpha36 1\.0\n"), "frame 18 is exactly alpha 0.5, and frame 36 exactly alpha 1")
ck(has(r"rate0 0\.0 rate\.5 1\.0 rate1 0\.0\n"), "there_and_back is exactly 0, 1 and 0 at the three ends")
ck(has(r"symmetric true\n") and has(r"rises true\n"), "the rate rises strictly to frame 18, and frames mirror about it")
ck(has(r"end_exact true peak_exact true pivot_held true ratio_ok true\n"), "end placement equals the start exactly, the peak equals the target exactly, and the pivot holds")
ck(has(r"default_rate_stays_swollen true default_rate_only_half_at_midpoint true\n"), "control: without rate_func the mobject stays at 1.25, and is only at 1.125 at the midpoint")

print("RESULT", "PASS" if not fails else f"{fails} FAIL")
sys.exit(1 if fails else 0)
