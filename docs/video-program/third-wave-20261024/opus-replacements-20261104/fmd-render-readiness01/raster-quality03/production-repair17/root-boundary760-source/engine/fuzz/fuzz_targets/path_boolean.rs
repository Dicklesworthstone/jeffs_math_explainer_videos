//! fm-ntp fuzz target: `fmn_library::boolean_ops` admitted routes.
//!
//! Resource-budget contract (§16.5/R14): for ANY path input the boolean
//! routine must either refuse with a typed `BooleanMobjectError` or
//! produce a `BooleanBuild` — never panic, never hang. The admitted
//! routes (Plan §7.4: separated-control-hulls + transversal-interiors)
//! are the target. Only totality is asserted: the results are discarded,
//! with no differential against the forced-fallback route.
//!
//! The input bytes are split into two point lists; each list seeds a
//! minimal `QuadPath` via the typed `from_points` builder. The boolean
//! routines are then driven against the two constructed `VMobject` paths.
//! This keeps the fuzzer inside the typed public surface.
#![no_main]

use fmn_library::boolean_ops::{difference, difference_with_options, union, union_with_options};
use fmn_library::BooleanOptions;
use fmn_library::QuadPath;
use fmn_library::VMobject;
use libfuzzer_sys::fuzz_target;

fn vms_from(input: &[u8]) -> Vec<VMobject> {
    // Split the fuzzer input at the first null byte (or mid-point) so
    // every byte sequence produces two point lists of arbitrary size.
    let mid = input
        .iter()
        .position(|b| *b == 0)
        .unwrap_or(input.len() / 2);
    let (left, right) = input.split_at(mid);
    [&left, &right]
        .iter()
        .filter_map(|chunk| {
            let mut points = Vec::with_capacity(chunk.len() / 12);
            for group in chunk.chunks(12) {
                if group.len() < 12 {
                    break;
                }
                let x = f32::from_le_bytes([group[0], group[1], group[2], group[3]]) as f64;
                let y = f32::from_le_bytes([group[4], group[5], group[6], group[7]]) as f64;
                let z = f32::from_le_bytes([group[8], group[9], group[10], group[11]]) as f64;
                points.push([x, y, z]);
            }
            if points.len() < 2 {
                return None;
            }
            let path = QuadPath::from_points(points).ok()?;
            // `from_path` returns `Self` (the new VMobject), not a Result;
            // an invalid path returns a zero-pointed VMobject that the
            // boolean routines will then refuse with a typed error.
            Some(VMobject::from_path(&path))
        })
        .collect()
}

fuzz_target!(|data: &[u8]| {
    let mobs = vms_from(data);
    if mobs.len() < 2 {
        return;
    }
    let options = BooleanOptions::default();
    let _ = union(&mobs);
    let _ = union_with_options(&mobs, options);
    let _ = difference(&mobs[0], &mobs[1]);
    let _ = difference_with_options(&mobs[0], &mobs[1], options);
});
