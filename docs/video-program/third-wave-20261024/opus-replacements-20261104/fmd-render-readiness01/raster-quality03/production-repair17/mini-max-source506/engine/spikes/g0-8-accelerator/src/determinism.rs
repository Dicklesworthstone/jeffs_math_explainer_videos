//! G0-6 (fm-zn9): one nontrivial frame, hashed, to decide OQ-1.
//!
//! ## The question
//!
//! §10.5 sketches a fallback the whole program is shaped around: if floating
//! screen-space arithmetic cannot be made bit-identical across platforms, the
//! certified engine adopts a **canonical fixed-point raster boundary** —
//! fixed-point screen coordinates, integer coverage accumulation at a defined
//! precision, explicit rounding. That is a different renderer. Deciding it
//! after W5 exists would invalidate every self-golden, which is why §20.1 puts
//! this spike in G0.
//!
//! So: render one frame on several platforms and compare **raw-buffer hashes**.
//! Not statistics, not a perceptual metric — the bits.
//!
//! ## What the frame has to contain, and why each piece is there
//!
//! fm-zn9 names the contents, and every one of them is a place two platforms
//! could disagree:
//!
//! - **arcs** — `QuadPath::arc` runs `sin`/`cos` per control point through
//!   fmn-dmath, then the fill and stroke solve roots on the result;
//! - **per-vertex `atan2` joints** — [`crate::ir::RenderIr::joint_angles`],
//!   reaching pixels through the `miter_gain` stand-in so a wrong `atan2` moves
//!   the hash instead of hiding;
//! - **rate functions** — `smooth`, `there_and_back`, `wiggle` (`sin`) and
//!   `exponential_decay` (`exp`) drive the scene's geometry and colour, so the
//!   frame is a *rendered animation frame* rather than a static diagram;
//! - **glow falloff** — the Reference's `true_dot` radial profile;
//! - **gradient fills** — nonzero-winding coverage plus a per-pixel gradient;
//! - **strokes with joins**, and **alpha compositing** throughout.
//!
//! ## Why the frame is small
//!
//! 480×270. The linux-aarch64 leg runs under qemu-user at roughly 30× the
//! native cost, and a spike whose evidence takes an hour to reproduce is a
//! spike nobody re-runs. The hash is no weaker for it: bit-identity over
//! 518 400 components either holds or it does not.

use crate::cpu::{self, Precision, Surface};
use crate::ir::{DrawKind, RenderIr, Style, TileGrid};
use crate::sdf::ANTI_ALIAS_WIDTH_PX;
use fmn_core::color::Srgb;
use fmn_core::constants::{BLUE_C, GREEN_B, MAROON_C, RED_C, TEAL_B, WHITE, YELLOW_C};
use fmn_core::rate;
use fmn_geom::quadpath::QuadPath;
use fmn_hash::sha256::{Digest, Sha256};

/// Frame width. Small on purpose — see the module docs.
pub const WIDTH: u32 = 480;
/// Frame height.
pub const HEIGHT: u32 = 270;
/// Tile edge.
pub const TILE: u32 = 16;

/// The animation time the frame is captured at, in `[0, 1]`.
///
/// A fixed, arbitrary-looking constant is the point: it is far from 0, 0.5 and
/// 1, so every rate function below is evaluated somewhere interesting rather
/// than at a value where several of them happen to agree.
pub const ALPHA: f64 = 0.37;

fn linear(c: Srgb, alpha: f64) -> [f32; 4] {
    let l = c.to_linear(alpha);
    [l.r as f32, l.g as f32, l.b as f32, l.a as f32]
}

/// Build the determinism frame's IR at the standard size.
pub fn frame_ir() -> RenderIr {
    build(WIDTH, HEIGHT, TILE, ALPHA)
}

/// The frame at an arbitrary size and animation time.
pub fn build(width: u32, height: u32, tile: u32, alpha: f64) -> RenderIr {
    let w = width as f64;
    let h = height as f64;
    let mut ir = RenderIr::new(
        TileGrid {
            width,
            height,
            tile,
        },
        linear(Srgb::from_rgb8(0x33, 0x33, 0x33), 1.0),
    );

    // Every rate function is evaluated once here and then drives geometry, so
    // a divergence in any of them moves the frame rather than a log line.
    let a_smooth = rate::smooth(alpha);
    let a_there_back = rate::there_and_back(alpha);
    let a_wiggle = rate::wiggle(alpha, 3.0);
    let a_decay = rate::exponential_decay(alpha, 0.25);
    let a_rush = rate::rush_into(alpha);

    let aaw = ANTI_ALIAS_WIDTH_PX as f32;

    // ---- a gradient-filled disc, its radius driven by a rate function.
    {
        let r = h * (0.18 + 0.10 * a_smooth);
        let c = [w * 0.30, h * 0.42, 0.0];
        let p = crate::arc(0.0, std::f64::consts::TAU, r, c, None);
        let mut st = Style::flat(linear(BLUE_C, 0.85), 0.0, aaw);
        st.rgba_end = linear(TEAL_B, 0.85);
        st.gradient_axis = [
            (c[0] - r) as f32,
            (c[1] - r) as f32,
            (c[0] + r) as f32,
            (c[1] + r) as f32,
        ];
        ir.compile_path(&p, st, DrawKind::Fill);
    }

    // ---- a filled ring: outer CCW, inner CW, so the nonzero rule must leave
    // a hole. A winding or crossing-order bug shows up as a solid disc.
    {
        let c = [w * 0.68, h * 0.40];
        let outer = h * 0.22;
        let inner = outer * (0.35 + 0.25 * a_there_back);
        let mut p = crate::arc(0.0, std::f64::consts::TAU, outer, [c[0], c[1], 0.0], None);
        let hole = crate::arc(0.0, -std::f64::consts::TAU, inner, [c[0], c[1], 0.0], None);
        p.add_subpath(hole.points())
            .expect("the hole is a valid subpath");
        let mut st = Style::flat(linear(YELLOW_C, 0.9), 0.0, aaw);
        st.rgba_end = linear(RED_C, 0.9);
        st.gradient_axis = [
            (c[0] - outer) as f32,
            c[1] as f32,
            (c[0] + outer) as f32,
            c[1] as f32,
        ];
        ir.compile_path(&p, st, DrawKind::Fill);
    }

    // ---- a sharp zigzag stroked with the joint-angle stand-in switched ON.
    // This is the only path in the frame that reads `joint_angles`, and it is
    // why a wrong `atan2` cannot pass unnoticed.
    {
        let mut p = QuadPath::default();
        let y0 = h * 0.78;
        let amp = h * 0.14 * (0.5 + 0.5 * a_wiggle);
        p.start_new_path([w * 0.08, y0, 0.0]);
        for i in 0..9 {
            let x = w * (0.08 + 0.09 * (i + 1) as f64);
            let y = if i % 2 == 0 { y0 - amp } else { y0 + amp * 0.4 };
            p.add_line_to([x, y, 0.0], false).unwrap();
        }
        let mut st = Style::flat(linear(MAROON_C, 1.0), 6.0, aaw);
        st.rgba_end = linear(WHITE, 1.0);
        st.miter_gain = 0.9;
        ir.compile_path(&p, st, DrawKind::Stroke);
    }

    // ---- a curved, tapered, gradient stroke over the fills.
    {
        let mut p = QuadPath::default();
        p.start_new_path([w * 0.05, h * 0.20, 0.0]);
        let bow = h * (0.10 + 0.35 * a_rush);
        p.add_quadratic_bezier_curve_to(
            [w * 0.50, h * 0.20 - bow, 0.0],
            [w * 0.95, h * 0.24, 0.0],
            false,
        )
        .unwrap();
        let mut st = Style::flat(linear(GREEN_B, 0.95), 11.0, aaw);
        st.width_end = 1.0;
        st.rgba_end = linear(BLUE_C, 0.4);
        ir.compile_path(&p, st, DrawKind::Stroke);
    }

    // ---- glow discs whose radii decay along the row: the `true_dot` profile
    // and `exp` in one.
    for i in 0..5 {
        let mut p = QuadPath::default();
        let cx = w * (0.14 + 0.18 * i as f64);
        let cy = h * 0.60;
        p.start_new_path([cx, cy, 0.0]);
        p.add_line_to([cx + 0.001, cy, 0.0], true).unwrap();
        let mut st = Style::flat(linear(WHITE, 0.75), 0.0, aaw);
        st.glow_radius = (h * 0.10 * (a_decay + 0.25 * i as f64)) as f32;
        // Alternating hard dots and true glows, so both branches of the
        // Reference's profile — with and without the `pow` — are in the hash.
        st.glow_factor = if i % 2 == 0 { 2.0 } else { 0.0 };
        ir.compile_path(&p, st, DrawKind::Glow);
    }

    // ---- a hairline finer than the AA band, so the sub-pixel regime is in
    // the hash too.
    {
        let mut p = QuadPath::default();
        p.start_new_path([w * 0.03, h * 0.94, 0.0]);
        p.add_quadratic_bezier_curve_to(
            [w * 0.5, h * 0.88 - h * 0.05 * a_smooth, 0.0],
            [w * 0.97, h * 0.94, 0.0],
            false,
        )
        .unwrap();
        ir.compile_path(
            &p,
            Style::flat(linear(WHITE, 1.0), 0.45, aaw),
            DrawKind::Stroke,
        );
    }

    ir.bin();
    ir
}

/// The canonical hash of a rendered surface.
///
/// Two things make this a hash of the *picture* rather than of a memory image:
///
/// 1. every component goes through `fmn_core::types::canonicalize_f32`, so
///    `-0.0` and `+0.0` hash alike and every NaN hashes as the one canonical
///    NaN — otherwise two runs that agree on every visible pixel could still
///    disagree on the digest, and the spike would report a divergence that is
///    not one;
/// 2. the bytes ride fmn-hash's versioned canonical container, so the digest is
///    a function of a declared schema rather than of `Vec<f32>`'s in-memory
///    layout.
pub fn hash_surface(surface: &Surface) -> Digest {
    let schema = fmn_hash::serial::Schema::new(*b"FMND", 1, 1, 0);
    let mut w = fmn_hash::serial::Writer::new(schema);
    w.put_u32(surface.width);
    w.put_u32(surface.height);
    for c in &surface.pixels {
        // put_f32 canonicalizes at the boundary (fmn-hash's own rule).
        w.put_f32(*c);
    }
    let bytes = w.finish().expect("surface fits the container limits");
    fmn_hash::sha256::sha256(&bytes)
}

/// One platform's complete determinism record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// Target triple, e.g. `linux-x86_64`.
    pub platform: String,
    /// Digest of the `f64` reference render.
    pub frame_f64: Digest,
    /// Digest of the `f32` render — carried because the *fast* CPU engine and
    /// the annex both live at f32, so a platform that agrees at f64 and
    /// disagrees at f32 is a finding about §6.1's mixed-precision licence, not
    /// about certification.
    pub frame_f32: Digest,
    /// Per-function fmn-dmath digests, in a fixed order.
    pub dmath: Vec<(&'static str, Digest)>,
}

impl Record {
    /// Produce this machine's record.
    pub fn measure() -> Record {
        let ir = frame_ir();
        Record {
            platform: platform_tag(),
            frame_f64: hash_surface(&cpu::render_at(&ir, Precision::Reference)),
            frame_f32: hash_surface(&cpu::render_at(&ir, Precision::AnnexF32)),
            dmath: dmath_digests(),
        }
    }

    /// The committed raw-data form: one `key<TAB>value` line per measurement,
    /// stable order, no timestamps — so two platforms' files diff cleanly and a
    /// re-run of the same platform produces byte-identical output.
    pub fn to_tsv(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("platform\t{}\n", self.platform));
        out.push_str(&format!("frame.f64\t{:x}\n", self.frame_f64));
        out.push_str(&format!("frame.f32\t{:x}\n", self.frame_f32));
        for (name, d) in &self.dmath {
            out.push_str(&format!("dmath.{name}\t{d:x}\n"));
        }
        out
    }
}

/// The platform tag, from compile-time target facts only.
pub fn platform_tag() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Per-function digests of fmn-dmath over a fixed grid.
///
/// fm-zn9 is also "fmn-dmath's first accuracy/portability shakedown", and a
/// single frame hash cannot say *which* function moved. These can: a divergence
/// lands on a named row.
///
/// The grid is deliberately awkward — irrational strides, negatives, values
/// past the argument-reduction thresholds — because the interesting
/// disagreements live at range boundaries, not at 0.5.
pub fn dmath_digests() -> Vec<(&'static str, Digest)> {
    fn digest_of(f: impl Fn(f64) -> f64, xs: &[f64]) -> Digest {
        let mut h = Sha256::new();
        for x in xs {
            h.update(&f(*x).to_bits().to_le_bytes());
        }
        h.finalize()
    }

    // 4001 points spanning ±40, crossing every π/2 boundary many times over.
    let wide: Vec<f64> = (0..4001).map(|i| (i as f64 - 2000.0) * 0.02).collect();
    // The domain-limited functions get their own grid on (-1, 1).
    let unit: Vec<f64> = (0..2001).map(|i| (i as f64 - 1000.0) / 1000.5).collect();
    // Strictly positive, spanning many decades, for ln and friends.
    let positive: Vec<f64> = (0..2001).map(|i| 1e-6 * 1.01f64.powi(i)).collect();

    vec![
        ("sin", digest_of(fmn_dmath::sin, &wide)),
        ("cos", digest_of(fmn_dmath::cos, &wide)),
        ("tan", digest_of(fmn_dmath::tan, &wide)),
        ("atan", digest_of(fmn_dmath::atan, &wide)),
        ("asin", digest_of(fmn_dmath::asin, &unit)),
        ("acos", digest_of(fmn_dmath::acos, &unit)),
        ("exp", digest_of(fmn_dmath::exp, &unit)),
        ("ln", digest_of(fmn_dmath::ln, &positive)),
        ("log2", digest_of(fmn_dmath::log2, &positive)),
        ("tanh", digest_of(fmn_dmath::tanh, &wide)),
        ("sinh", digest_of(fmn_dmath::sinh, &unit)),
        ("cosh", digest_of(fmn_dmath::cosh, &unit)),
        ("cbrt", digest_of(fmn_dmath::cbrt, &wide)),
        ("sqrt", digest_of(fmn_dmath::sqrt, &positive)),
        // atan2 needs both arguments varied, so it gets its own sweep over the
        // full circle including the axes, where the quadrant rules live.
        ("atan2", {
            let mut h = Sha256::new();
            for i in 0..1001 {
                let t = (i as f64) * std::f64::consts::TAU / 1000.0;
                let (y, x) = (fmn_dmath::sin(t) * 3.0, fmn_dmath::cos(t) * 3.0);
                h.update(&fmn_dmath::atan2(y, x).to_bits().to_le_bytes());
            }
            for (y, x) in [
                (0.0, 1.0),
                (0.0, -1.0),
                (1.0, 0.0),
                (-1.0, 0.0),
                (-0.0, -1.0),
                (-0.0, 1.0),
            ] {
                h.update(&fmn_dmath::atan2(y, x).to_bits().to_le_bytes());
            }
            h.finalize()
        }),
        // pow, over a grid that crosses 1 and includes fractional exponents.
        ("pow", {
            let mut h = Sha256::new();
            for i in 0..501 {
                let b = 0.01 + i as f64 * 0.02;
                for e in [-2.5f64, -1.0, 0.5, 1.0, 2.0, 3.75] {
                    h.update(&fmn_dmath::pow(b, e).to_bits().to_le_bytes());
                }
            }
            h.finalize()
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The frame's `f64` digest, bit-locked across the certified matrix.
    ///
    /// Measured identical on linux-x86_64 (glibc, native), linux-aarch64 (musl,
    /// qemu-emulated) and macos-aarch64 (Darwin, native) on 2026-07-25 —
    /// `docs/g0/g0-6-hashes/`. This is a **cross-platform** self-golden, which
    /// is the strongest kind: it fails on any machine the moment the renderer
    /// stops being reproducible, instead of waiting for someone to re-run the
    /// three-platform sweep.
    ///
    /// A drift here is a finding to adjudicate, never a number to re-bless
    /// (GOVERNANCE §5). If the frame is deliberately changed, the sweep is
    /// re-run and the note updated in the same commit.
    const FRAME_F64_GOLDEN: &str =
        "3ff2cac55c33a8b2f460b5e3d338a542a736a6093f9519b712535e31bbe675f7";

    /// The `f32` digest, same provenance. Locked separately because the fast
    /// CPU engine and the annex both live here, so a change that moves only
    /// this one is a finding about §6.1's mixed-precision licence.
    const FRAME_F32_GOLDEN: &str =
        "9e94bf5e95765eca5927178ac70da2fd21133d1a4f173cafa0c2428042595698";

    #[test]
    fn the_frame_matches_its_cross_platform_golden() {
        let r = Record::measure();
        assert_eq!(
            format!("{:x}", r.frame_f64),
            FRAME_F64_GOLDEN,
            "the f64 frame drifted from the G0-6 cross-platform golden"
        );
        assert_eq!(
            format!("{:x}", r.frame_f32),
            FRAME_F32_GOLDEN,
            "the f32 frame drifted from the G0-6 cross-platform golden"
        );
    }

    #[test]
    fn the_frame_exercises_everything_fm_zn9_names() {
        let ir = frame_ir();
        let kinds: Vec<_> = ir.paths.iter().map(|p| p.kind).collect();
        assert!(kinds.contains(&DrawKind::Fill), "no gradient fill");
        assert!(kinds.contains(&DrawKind::Stroke), "no stroke");
        assert!(kinds.contains(&DrawKind::Glow), "no glow falloff");
        assert!(
            ir.styles.iter().any(|s| s.miter_gain != 0.0),
            "no path reads the atan2 joint angles, so atan2 is untested"
        );
        assert!(
            ir.styles.iter().any(|s| s.rgba != s.rgba_end),
            "no gradient, so the gradient field is untested"
        );
        assert!(
            ir.joint_angles.iter().any(|a| *a != 0.0),
            "every joint angle is zero — the zigzag is not turning"
        );
        assert_eq!(ir.joint_angles.len(), ir.segments.len() * 2);
    }

    #[test]
    fn the_hash_is_stable_within_a_run() {
        // If this ever fails, the renderer has in-process nondeterminism and
        // every cross-platform comparison below it is meaningless.
        let ir = frame_ir();
        let a = hash_surface(&cpu::render_at(&ir, Precision::Reference));
        let b = hash_surface(&cpu::render_at(&ir, Precision::Reference));
        assert_eq!(a, b);
    }

    #[test]
    fn the_hash_notices_a_single_changed_component() {
        let ir = frame_ir();
        let s = cpu::render_at(&ir, Precision::Reference);
        let base = hash_surface(&s);
        let mut t = s.clone();
        t.pixels[12345] += 1e-7;
        assert_ne!(base, hash_surface(&t), "the hash is not sensitive enough");
    }

    #[test]
    fn signed_zero_does_not_change_the_hash() {
        // The canonicalization that stops the spike reporting a divergence
        // that no one could see. Without it, a platform that produced -0.0
        // where another produced +0.0 would fail a bit-identity check over two
        // pictures that are the same picture.
        let ir = frame_ir();
        let s = cpu::render_at(&ir, Precision::Reference);
        let mut t = s.clone();
        for c in t.pixels.iter_mut() {
            if *c == 0.0 {
                *c = -0.0;
            }
        }
        assert_eq!(hash_surface(&s), hash_surface(&t));
    }

    #[test]
    fn the_f32_and_f64_renders_are_different_pictures() {
        // Both digests are recorded, so they must actually differ — otherwise
        // the f32 path is not running and the record is quietly duplicating one
        // measurement under two names.
        let r = Record::measure();
        assert_ne!(r.frame_f64, r.frame_f32);
    }

    #[test]
    fn every_dmath_function_gets_its_own_row() {
        let d = dmath_digests();
        assert_eq!(d.len(), 16, "a function was added or lost");
        let mut names: Vec<_> = d.iter().map(|(n, _)| *n).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), d.len(), "duplicate function name");
        // Distinct functions on overlapping grids must not collide.
        let mut digests: Vec<_> = d.iter().map(|(_, x)| format!("{x:x}")).collect();
        digests.sort_unstable();
        digests.dedup();
        assert_eq!(digests.len(), d.len(), "two functions hashed identically");
    }

    #[test]
    fn the_tsv_is_ordered_and_free_of_run_specific_noise() {
        let r = Record::measure();
        let a = r.to_tsv();
        assert_eq!(a, Record::measure().to_tsv());
        assert!(a.starts_with("platform\t"));
        assert!(a.lines().count() >= 19);
        for line in a.lines() {
            assert_eq!(line.split('\t').count(), 2, "not a two-column row: {line}");
        }
    }
}

/// The §10.5(d) guard: no FMA contraction on the certified path.
///
/// Two layers, because the disassembly layer is the real evidence and the
/// source layer is the one that survives in CI:
///
/// - **Object code** (manual, recorded in the G0-6 note): `llvm-objdump -d` on
///   the aarch64 build finds 1652 scalar FP instructions and **zero**
///   `fmadd`/`fmsub`/`fnmadd`/`fnmsub`/`fmla`/`fmls`, on a target where FMA is
///   baseline. The scalar-op count is what makes the zero meaningful — an
///   earlier pass of this check used `objdump`, which cannot disassemble
///   aarch64 at all and duly reported zero of everything.
/// - **Source** ([`no_mul_add_on_the_certified_path`]): rustc performs no FP
///   contraction by default, so the object-code result confirms a default
///   rather than a setting. The realistic regression is therefore a hand-written
///   `mul_add`, and that is what the test below refuses.
#[cfg(test)]
mod fma_guard {
    /// Crate source roots that must stay contraction-free.
    const CERTIFIED_ROOTS: &[&str] = &[
        "src",
        "../../crates/fmn-dmath/src",
        "../../crates/fmn-geom/src",
        "../../crates/fmn-core/src",
        "../../crates/fmn-frame/src",
    ];

    fn rust_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                rust_files(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }

    #[test]
    fn no_mul_add_on_the_certified_path() {
        let mut files = Vec::new();
        for root in CERTIFIED_ROOTS {
            rust_files(std::path::Path::new(root), &mut files);
        }
        assert!(files.len() > 20, "found only {} sources", files.len());

        let mut offenders = Vec::new();
        for f in &files {
            let Ok(text) = std::fs::read_to_string(f) else {
                continue;
            };
            for (i, line) in text.lines().enumerate() {
                // Doc comments may discuss it; code may not call it.
                let code = line.trim_start();
                if code.starts_with("//") {
                    continue;
                }
                // The needle is assembled rather than written out, so this
                // file does not match itself — the first run of this test
                // failed on its own source, which was funny once.
                let dot_form = concat!(".mul", "_add(");
                let path_form = concat!("f64::mul", "_add");
                if code.contains(dot_form) || code.contains(path_form) {
                    offenders.push(format!("{}:{}", f.display(), i + 1));
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "§10.5(d) forbids FMA on certified paths; found mul_add at:\n{}",
            offenders.join("\n")
        );
    }
}
