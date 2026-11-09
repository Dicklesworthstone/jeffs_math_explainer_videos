//! Rendered regression for the plate fade (cc2-plate-fade-investigation246). Install as
//! `src/chapters/plate_fade_probe.rs` beside the candidate `draw.rs`, which declares it under
//! `cfg(test)`. It renders, so it is ignored by default; run it on purpose:
//!
//! `cargo test --release --bin complex_analysis_visual plate_fade_probe -- --ignored --nocapture`
//!
//! One small native render through the film's own route (`crate::declare_plan_budget`: the
//! retained camera renderer that raster plates need, and the film's plan limits), Y4M into
//! memory, 640×360 at 10 fps, 95 frames. Three phases, each fade 1 s (alpha k/10, k = 1..10),
//! each hold 0.5 s (5 frames):
//!
//! - A: a static `Picture` (`PLATE`, left half) and a vector square (right half), faded in and
//!   out together by `fade_in_all` / `fade_out_all`. The square is the measuring control.
//! - B: an updater-backed `Painter` (left half) through `Painter::show` / `Painter::leave`.
//! - C: the same static Picture as A through production19's fade bodies, kept verbatim below:
//!   the defect control. The probe must see C pop on at full strength and stay full to the
//!   last fade-out frame, or it cannot be trusted to see the fix.
//!
//! Luma is read per pixel against the bare background frame and each phase's own
//! full-strength hold (share 0 = background, 1 = full plate). Checked for A, B and the square:
//! first fade-in frame ≈ 0, mid frame (k = 5) between, last fade-in frame and the first hold
//! frame both full, mid fade-out between, the last fade-out frame ≈ 0 and the first frame of
//! the hold after it bare background (no full-bright flash when the fade out gives the lane
//! back at 1 as the remover takes the plate off). Timeline: every phase's frame counts equal
//! C's, which are production19's, and the whole render is 95 frames.

use std::path::Path;
use std::sync::Arc;

use fmn::platform::fs::{FileSystem, VirtualFs};

use super::*;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;
const FADE: f64 = 1.0;
const HOLD: f64 = 0.5;
const LIGHT: Srgb = Srgb {
    r: 0.85,
    g: 0.85,
    b: 0.85,
};

/// Production19's `fade_in_all`, verbatim (draw.rs at cd394e5a): the defect control.
fn legacy_fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Production19's `fade_out_all`, verbatim (draw.rs at cd394e5a): the defect control.
fn legacy_fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// A 4 × 3 static plate of one light colour, its top left at (−5.5, 1.5): left half only.
fn plate(stage: &mut Stage<'_>) -> fmn::Result<Picture> {
    Picture::build(stage, v(-5.5, 1.5), (0.1, 0.1), &vec![vec![LIGHT; 40]; 30], &PLATE)
}

#[derive(Default)]
struct Probe {
    /// Clock frames at each phase boundary.
    marks: Vec<(&'static str, i64)>,
}

impl Probe {
    fn mark(&mut self, stage: &Stage<'_>, name: &'static str) {
        self.marks.push((name, stage.scene().time().frames()));
    }

    fn at(&self, name: &str) -> usize {
        let (_, frame) = self.marks.iter().find(|(n, _)| *n == name).expect("a mark");
        usize::try_from(*frame).expect("a frame")
    }
}

impl SceneConstruct for Probe {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.wait(HOLD)?;
        let pic = plate(stage)?;
        let pts = vec![v(1.5, 1.5), v(4.5, 1.5), v(4.5, -1.5), v(1.5, -1.5)];
        let square = stage.arena_mut().add(Polygon::new(pts).style(flat(LIGHT, 1.0)).build());
        self.mark(stage, "a_in");
        fade_in_all(stage, &[pic.group, square], FADE)?;
        self.mark(stage, "a_hold");
        stage.wait(HOLD)?;
        self.mark(stage, "a_out");
        fade_out_all(stage, &[pic.group, square], FADE)?;
        self.mark(stage, "a_gone");
        stage.wait(HOLD)?;

        let plane = Plane::new(v(-3.5, 0.0), 3.0, (-1.2, 1.2), (-1.2, 1.2));
        let painter = Painter::build(stage, Some(Func::Identity), plane, 1.0)?;
        self.mark(stage, "b_in");
        painter.show(stage, FADE)?;
        self.mark(stage, "b_hold");
        stage.wait(HOLD)?;
        self.mark(stage, "b_out");
        painter.leave(stage, FADE)?;
        self.mark(stage, "b_gone");
        stage.wait(HOLD)?;

        let old = plate(stage)?;
        self.mark(stage, "c_in");
        legacy_fade_in_all(stage, &[old.group], FADE)?;
        self.mark(stage, "c_hold");
        stage.wait(HOLD)?;
        self.mark(stage, "c_out");
        legacy_fade_out_all(stage, &[old.group], FADE)?;
        self.mark(stage, "c_gone");
        stage.wait(HOLD)?;
        self.mark(stage, "end");
        Ok(())
    }
}

/// The luma plane of every frame of a 4:2:0 Y4M stream.
fn luma_planes(bytes: &[u8]) -> Vec<Vec<u8>> {
    let header_end = bytes.iter().position(|&b| b == b'\n').expect("a Y4M header");
    let header = std::str::from_utf8(&bytes[..header_end]).expect("an ASCII header");
    assert!(header.starts_with("YUV4MPEG2 "), "{header}");
    let dim = |tag: char| {
        header
            .split(' ')
            .find_map(|t| t.strip_prefix(tag))
            .and_then(|n| n.parse::<usize>().ok())
            .expect("width and height")
    };
    assert_eq!((dim('W'), dim('H')), (WIDTH, HEIGHT), "{header}");
    let luma = WIDTH * HEIGHT;
    let chroma = 2 * WIDTH.div_ceil(2) * HEIGHT.div_ceil(2);
    let mut at = header_end + 1;
    let mut frames = Vec::new();
    while at < bytes.len() {
        let line = bytes[at..].iter().position(|&b| b == b'\n').expect("a FRAME line");
        assert!(bytes[at..at + line].starts_with(b"FRAME"), "frame {}", frames.len());
        let y0 = at + line + 1;
        frames.push(bytes[y0..y0 + luma].to_vec());
        at = y0 + luma + chroma;
    }
    assert_eq!(at, bytes.len(), "whole frames only");
    frames
}

/// The share of its full strength one half of `frame` shows: Σ(frame − ground) / Σ(full −
/// ground) over that half's pixels where the full picture differs from the ground by at least
/// 24 luma steps. 0 is the bare background, 1 the full-strength hold.
fn share(frame: &[u8], ground: &[u8], full: &[u8], left: bool) -> f64 {
    let (mut seen, mut whole, mut n) = (0.0, 0.0, 0usize);
    for (i, ((&f, &g), &u)) in frame.iter().zip(ground).zip(full).enumerate() {
        if (i % WIDTH < WIDTH / 2) != left {
            continue;
        }
        let d = f64::from(u) - f64::from(g);
        if d.abs() < 24.0 {
            continue;
        }
        seen += f64::from(f) - f64::from(g);
        whole += d;
        n += 1;
    }
    assert!(
        n >= 2000,
        "the {} half shows only {n} contrasted pixels",
        if left { "left" } else { "right" }
    );
    seen / whole
}

/// The shares of one region over one fade's ten frames, the fade starting at clock frame
/// `start` (its k-th frame is Y4M frame `start + k − 1`).
fn fade(frames: &[Vec<u8>], start: usize, ground: &[u8], full: &[u8], left: bool) -> Vec<f64> {
    (0..10).map(|k| share(&frames[start + k], ground, full, left)).collect()
}

/// A correct fade: first frame ≈ 0, mid frame between, last frame full (in), or the reverse
/// (out), monotone in between.
fn assert_fades(name: &str, s: &[f64], rising: bool) {
    let (first, last) = if rising { (s[0], s[9]) } else { (1.0 - s[0], 1.0 - s[9]) };
    assert!(
        first <= 0.05,
        "{name}: first frame {:.3} (opacity {:.3}): {s:.3?}",
        s[0],
        if rising { 0.009 } else { 0.991 }
    );
    assert!(
        (0.25..=0.75).contains(&s[4]),
        "{name}: mid frame {:.3}, opacity 0.5: {s:.3?}",
        s[4]
    );
    assert!(last >= 0.99, "{name}: last frame {:.3}: {s:.3?}", s[9]);
    for w in s.windows(2) {
        let step = if rising { w[1] - w[0] } else { w[0] - w[1] };
        assert!(step >= -0.005, "{name}: not monotone: {s:.3?}");
    }
}

#[test]
#[ignore = "renders 95 small frames: run on purpose with --ignored"]
fn plates_fade_in_pixels_on_the_films_own_frames() {
    let mut opts = RenderOptions::with_format("/plate-fade.y4m", RenderFormat::Y4m).expect("Y4M options");
    opts.config.camera.resolution = (640, 360);
    opts.config.camera.fps = 10;
    crate::declare_plan_budget(&mut opts);
    let fs = Arc::new(VirtualFs::new());
    let mut probe = Probe::default();
    let report = fmn::render_with_fs(&mut probe, opts, fs.clone()).expect("the probe renders");
    let frames = luma_planes(&fs.read(Path::new("/plate-fade.y4m")).expect("the Y4M is published"));

    // timeline: candidate phases A and B take exactly the frames production19's C takes
    let span = |a: &str, b: &str| probe.at(b) - probe.at(a);
    for p in ["a", "b", "c"] {
        let name = |s: &str| format!("{p}_{s}");
        let got = [
            span(&name("in"), &name("hold")),
            span(&name("hold"), &name("out")),
            span(&name("out"), &name("gone")),
        ];
        assert_eq!(got, [10, 5, 10], "phase {p}: fade in, hold, fade out frames");
    }
    assert_eq!(probe.at("end"), 95);
    assert_eq!(frames.len(), 95, "one Y4M frame per clock frame, none at t = 0");
    assert_eq!(report.artifact.frame_count, 95);

    let ground = &frames[probe.at("a_in") - 1];
    let a_full = &frames[probe.at("a_out") - 1];
    let b_full = &frames[probe.at("b_out") - 1];
    let c_full = &frames[probe.at("c_out") - 1];
    let half = |f: &[u8]| -> Vec<u8> {
        f.chunks(WIDTH)
            .flat_map(|row| row[..WIDTH / 2].iter().copied())
            .collect()
    };
    let report_rows = |name: &str, s: &[f64]| eprintln!("{name:>18}: {s:.3?}");

    // A: static Picture and vector square, one play each way
    let pin = fade(&frames, probe.at("a_in"), ground, a_full, true);
    let pout = fade(&frames, probe.at("a_out"), ground, a_full, true);
    let sin = fade(&frames, probe.at("a_in"), ground, a_full, false);
    let sout = fade(&frames, probe.at("a_out"), ground, a_full, false);
    for (name, s) in [
        ("A picture in", &pin),
        ("A square in", &sin),
        ("A picture out", &pout),
        ("A square out", &sout),
    ] {
        report_rows(name, s);
    }
    assert_fades("A square in (control)", &sin, true);
    assert_fades("A square out (control)", &sout, false);
    assert_fades("A picture in", &pin, true);
    assert_fades("A picture out", &pout, false);
    eprintln!(
        "A picture − square at k = 5: in {:+.3}, out {:+.3} (diagnostic)",
        pin[4] - sin[4],
        pout[4] - sout[4]
    );

    // B: the updater-backed Painter through show and leave
    let bin = fade(&frames, probe.at("b_in"), ground, b_full, true);
    let bout = fade(&frames, probe.at("b_out"), ground, b_full, true);
    report_rows("B painter in", &bin);
    report_rows("B painter out", &bout);
    assert_fades("B painter in", &bin, true);
    assert_fades("B painter out", &bout, false);

    // the frames either side of each fade's end: no dip after a fade in, no flash after a fade out
    for (p, full) in [("a", a_full), ("b", b_full)] {
        let hold = probe.at(&format!("{p}_hold"));
        assert_eq!(
            &frames[hold - 1],
            &frames[hold],
            "{p}: last fade-in frame and first hold frame differ"
        );
        assert_eq!(&frames[hold], full, "{p}: the hold changes while it holds");
        let gone = probe.at(&format!("{p}_gone"));
        let last_out = share(&frames[gone - 1], ground, full, true);
        let first_after = share(&frames[gone], ground, full, true);
        eprintln!("{p}: last fade-out frame {last_out:.4}, first frame after {first_after:.4}");
        assert!(
            last_out.abs() <= 0.01,
            "{p}: the last fade-out frame still shows {last_out:.3}"
        );
        assert_eq!(
            &frames[gone], ground,
            "{p}: the first frame after the fade out is not bare background (flash?)"
        );
        assert_eq!(
            &frames[gone + 4],
            ground,
            "{p}: the hold after the fade out is not bare background"
        );
    }

    // C: production19's bodies must show the defect, or this probe cannot see it
    let cin = fade(&frames, probe.at("c_in"), ground, c_full, true);
    let cout = fade(&frames, probe.at("c_out"), ground, c_full, true);
    report_rows("C legacy in", &cin);
    report_rows("C legacy out", &cout);
    assert!(
        cin.iter().chain(&cout).all(|&s| s >= 0.95),
        "the defect control no longer pops: the probe is not measuring what it claims"
    );
    assert_eq!(&frames[probe.at("c_gone")], ground, "C leaves at its remover");

    // full strength is unchanged: the candidate's hold is production19's hold, pixel for pixel
    assert_eq!(
        half(a_full),
        half(c_full),
        "A's full-strength plate differs from production19's"
    );
}
