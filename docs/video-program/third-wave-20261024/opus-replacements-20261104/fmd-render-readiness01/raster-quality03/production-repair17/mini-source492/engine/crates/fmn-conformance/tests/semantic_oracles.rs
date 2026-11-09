//! Semantic sanity oracles on the native golden-producing routes (fm-5wq.46).
//!
//! Each route draws the built-in semantic witness and the oracles in
//! `fmn_conformance::semantic` read its frame: orientation, placement,
//! colour and reading order. Each test also plants the regression the
//! oracles exist for, a vertical mirror of the same frame (fm-sq8.9's bug),
//! and requires the oracles to reject it. A route whose frame is mirrored,
//! misplaced or recoloured fails here even when every bit-locked golden
//! agrees with it. The portal's routes run in `fmn-python`
//! (`tests/semantic_witness.py`), and wasm-smoke runs the oracles inside a
//! wasm VM.
#![forbid(unsafe_code)]
#![allow(clippy::print_stderr)]

#[allow(dead_code)]
#[path = "support/semantic_routes.rs"]
mod semantic_routes;

use semantic_routes::RouteFrame;

fn check_route(route: fn() -> Result<RouteFrame, String>) -> Result<(), String> {
    let frame = route()?;
    let (rendered, flipped) = frame.read();
    for (label, readings) in [("rendered", &rendered), ("planted_vertical_flip", &flipped)] {
        for reading in readings {
            eprintln!(
                "semantic-oracle route={} frame={label} oracle={} fixture={:?} expected={:?} measured={:?} pass={}",
                frame.route,
                reading.oracle,
                reading.fixture,
                reading.expected,
                reading.measured,
                reading.pass
            );
        }
    }
    frame.verdict()
}

#[test]
fn native_library_2d_route_reads_right_side_up() -> Result<(), String> {
    check_route(semantic_routes::library_2d)
}

#[test]
fn native_cli_2d_route_reads_right_side_up() -> Result<(), String> {
    check_route(semantic_routes::cli_2d)
}

#[test]
fn native_camera_route_reads_right_side_up() -> Result<(), String> {
    check_route(semantic_routes::cli_camera)
}

#[test]
fn native_fmtl_player_route_reads_right_side_up() -> Result<(), String> {
    check_route(semantic_routes::fmtl_native)
}

#[test]
fn wasm_fmtl_player_route_reads_right_side_up() -> Result<(), String> {
    check_route(semantic_routes::fmtl_wasm_player)
}
