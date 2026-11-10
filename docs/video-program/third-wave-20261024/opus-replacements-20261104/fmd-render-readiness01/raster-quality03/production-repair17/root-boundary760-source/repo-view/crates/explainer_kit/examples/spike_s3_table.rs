//! Spike S-3 (plan §6.6, §10.7; Video B chapter 07_7): the GF(256)
//! multiplication table as 255 row strips, rows then columns reordered by
//! moving strips.
//!
//! ```text
//! cargo run --release -p explainer_kit --example spike_s3_table -- OUT_DIR [WxH] [MECH] [MODE] [CELLS]
//! ```
//!
//! The table is 256×256 cells, colour = the product byte, at 4 px per cell at
//! 1080p (8 px at 4K: cell edges sit on pixel edges at both). Row 0 and
//! column 0 (zero has no log) are flat bars that are set aside, dimmed,
//! before the sort; the 255×255 body is 255 strips, one VMobject each with one
//! piece per cell (a square sub-path or a polyline segment) and the cell
//! colours written per point. Rows move to
//! their discrete-log order by a dt updater per strip (placement only); at a
//! hold the row strips are swapped for 255 column strips (the same pixels),
//! and the columns move the same way, leaving diagonal stripes (the log of a
//! product is the sum of the logs mod 255).
//!
//! MECH, the per-cell colour mechanism (default `polyline`):
//! - `fill`: per-point `fill_rgba` over square sub-paths (the plan's
//!   mechanism). franken_manim fills a varying `fill_rgba` with a mean-value
//!   interpolant over every station of every contour, so cells blend;
//! - `border`: the same with an inner `fill_border_width` wider than half a
//!   cell, which paints each pixel with its nearest boundary's colour (crisp),
//!   cells inset by a hair so the nearest boundary is the pixel's own cell;
//! - `stroke`: one open polyline per strip (a null curve between cells makes
//!   the colour jump), per-point `stroke_rgba`, stroke width = one cell, and
//!   a transparent half cell at each end, so the round caps paint nothing;
//! - `polyline`: `stroke`, with the strips tagged as the polylines they are,
//!   so the renderer takes its capsule (straight-segment) path: the same
//!   pixels for less work.
//!
//! MODE (default `video`):
//! - `video`: `table.mp4`, 12.5 s at 60 fps (noise hold, set-aside, row sort
//!   4 s, a hold with the swap in it, column sort 4 s, stripes hold 2 s),
//!   x264 CRF 16 through `delivery_options`; then `swap/`, two PNG frames on
//!   either side of the row → column swap, compared by ffmpeg (PSNR, SSIM);
//! - `stills`: `noise/` and `stripes/`, one PNG frame each of the first and
//!   last states (the per-frame cost of a mechanism without a video).
//!
//! CELLS (default 255: one strip per row) splits each row and column into
//! strips of that many cells, moved together. The stroke renderer tests every
//! segment of a shape at every pixel it shades, so a pixel's cost grows with
//! the cells per strip; but a chunk's ends leave a few corner pixels of its
//! end cells off by some code values, so 255 is the one to use.
//!
//! One line per render on stdout: name, mechanism, resolution, fps, frames,
//! wall seconds.

use std::path::{Path, PathBuf};
use std::time::Instant;

use explainer_kit::encode::{self, Quality};
use explainer_kit::{hex, play, rect};
use fmn::core::color::color_gradient;
use fmn::mobject::ShapeTag;
use fmn::prelude::*;
use fmn::rendering::RenderError;

/// x⁸ = x⁴ + x³ + x² + 1 (0x11D): the low byte folded back in on overflow.
const FOLD: u8 = 0x1D;

/// Multiply by x (by 2) in GF(256) mod 0x11D.
fn xtime(a: u8) -> u8 {
    if a & 0x80 == 0 { a << 1 } else { (a << 1) ^ FOLD }
}

/// Shift-and-add multiplication in GF(256) mod 0x11D.
fn mul(mut a: u8, mut b: u8) -> u8 {
    let mut p = 0;
    while b != 0 {
        if b & 1 != 0 {
            p ^= a;
        }
        a = xtime(a);
        b >>= 1;
    }
    p
}

/// Discrete logs base 2: `exp[k] = 2^k`, `log[exp[k]] = k` (log 0 unused).
struct Field {
    exp: [u8; 255],
    log: [u8; 256],
}

impl Field {
    fn new() -> Self {
        assert_eq!(mul(0x57, 0x83), 0x31, "0x57 · 0x83 mod 0x11D");
        assert_eq!(mul(2, 142), 1, "142 is the inverse of 2");
        let (mut exp, mut log, mut seen) = ([0u8; 255], [0u8; 256], [false; 256]);
        let mut x = 1u8;
        for k in 0..255u8 {
            assert!(!seen[x as usize], "2 repeats after {k} steps: not a generator");
            seen[x as usize] = true;
            exp[k as usize] = x;
            log[x as usize] = k;
            x = xtime(x);
        }
        assert_eq!(x, 1, "2^255 = 1");
        assert!(seen[1..].iter().all(|&s| s), "2 generates all 255 nonzero bytes");
        // the stripes: log(ab) = log a + log b mod 255, over the whole body
        for a in 1..=255u8 {
            for b in 1..=255u8 {
                let sum = (u16::from(log[a as usize]) + u16::from(log[b as usize])) % 255;
                assert_eq!(u16::from(log[mul(a, b) as usize]), sum);
            }
        }
        Self { exp, log }
    }

    /// The slot (0..=255) of nonzero `a` in log order: 1 + log₂ a.
    fn sorted(&self, a: u8) -> f64 {
        1.0 + f64::from(self.log[a as usize])
    }
}

// Geometry in 1080p pixels (4K doubles every number, so cell edges stay on
// pixel edges): the frame is 8 units tall, 135 px per unit. Slot s of the
// table (byte order: slot = the byte) spans [450 + 4s, 454 + 4s] across and
// [30 + 4s, 34 + 4s] down; set aside, column 0 sits at 446..450 and row 0 at
// 26..30, so the whole picture is centred.
const UNIT_PX: f64 = 135.0;
const CELL_PX: f64 = 4.0;
const LEFT_PX: f64 = 450.0;
const TOP_PX: f64 = 30.0;
const CELL: f64 = CELL_PX / UNIT_PX;
/// The border mechanism's cell inset (1080p px): enough to break the tie
/// between two cells' shared edge, far under one 8-bit level of coverage.
const INSET_PX: f64 = 0.002;
/// The set-aside bars' opacity.
const DIM: f64 = 0.35;
const SORT_S: f64 = 4.0;

fn x_of(px: f64) -> f64 {
    (px - 960.0) / UNIT_PX
}
fn y_of(py: f64) -> f64 {
    (540.0 - py) / UNIT_PX
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mech {
    Stroke,
    Polyline,
    Fill,
    Border,
}

impl Mech {
    fn name(self) -> &'static str {
        match self {
            Self::Stroke => "stroke",
            Self::Polyline => "polyline",
            Self::Fill => "fill",
            Self::Border => "border",
        }
    }

    fn strokes(self) -> bool {
        matches!(self, Self::Stroke | Self::Polyline)
    }
}

/// Which way a strip runs.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Run {
    /// Along x at row slot `across`.
    Row,
    /// Along y at column slot `across`.
    Column,
}

/// One strip of cells, `products[k]` in slot `first + k`, at slot `across`,
/// coloured from `pal` per point.
fn strip(mech: Mech, pal: &[Srgb], products: &[u8], first: usize, run: Run, across: f64) -> Mobject {
    // a point at (along, across) in pixels, as a scene point
    let at = |along: f64, side: f64| -> Vec3 {
        match run {
            Run::Row => [x_of(along), y_of(side), 0.0],
            Run::Column => [x_of(side), y_of(along), 0.0],
        }
    };
    let (along0, across0) = match run {
        Run::Row => (LEFT_PX, TOP_PX),
        Run::Column => (TOP_PX, LEFT_PX),
    };
    let lo = across0 + CELL_PX * across;
    let mut points: Vec<Vec3> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    for (k, &p) in products.iter().enumerate() {
        let s = pal[p as usize];
        let c = [s.r as f32, s.g as f32, s.b as f32, 1.0];
        let clear = [s.r as f32, s.g as f32, s.b as f32, 0.0];
        let a0 = along0 + CELL_PX * (first + k) as f64;
        let a1 = a0 + CELL_PX;
        match mech {
            Mech::Stroke | Mech::Polyline => {
                // shared anchors; a null curve (h = a = the boundary) before
                // each cell gives the boundary two records, so the colour
                // jumps. Each end of the strip runs on half a cell, clear, so
                // its round cap paints nothing over a neighbour.
                let mid = lo + CELL_PX / 2.0;
                let quarter = CELL_PX / 4.0;
                if k == 0 {
                    points.extend([at(a0 - 2.0 * quarter, mid), at(a0 - quarter, mid), at(a0, mid)]);
                    colors.extend([clear; 3]);
                }
                let last = *points.last().expect("the strip has started");
                points.extend([last, last, at((a0 + a1) / 2.0, mid), at(a1, mid)]);
                colors.extend([c; 4]);
                if k + 1 == products.len() {
                    let end = at(a1, mid);
                    points.extend([end, end, at(a1 + quarter, mid), at(a1 + 2.0 * quarter, mid)]);
                    colors.extend([clear; 4]);
                }
            }
            Mech::Fill | Mech::Border => {
                let e = if mech == Mech::Border { INSET_PX } else { 0.0 };
                let corners = [
                    at(a0 + e, lo + e),
                    at(a1 - e, lo + e),
                    at(a1 - e, lo + CELL_PX - e),
                    at(a0 + e, lo + CELL_PX - e),
                ];
                if k > 0 {
                    // a handle on the previous anchor ends that sub-path
                    let last = *points.last().expect("a previous cell");
                    points.push(last);
                    colors.push(c);
                }
                for i in 0..4 {
                    let (u, v) = (corners[i], corners[(i + 1) % 4]);
                    points.extend([u, [(u[0] + v[0]) / 2.0, (u[1] + v[1]) / 2.0, 0.0]]);
                    colors.extend([c, c]);
                }
                points.push(corners[0]);
                colors.push(c);
            }
        }
    }
    let c0 = pal[products[0] as usize];
    let style = match mech {
        // stroke width in Reference units (0.01 scene units each): one cell
        Mech::Stroke | Mech::Polyline => Style::default().fill(c0, 0.0).stroke(c0, CELL / 0.01, 1.0),
        Mech::Fill => Style::default().fill(c0, 1.0).stroke(c0, 0.0, 0.0),
        // an inner band two cells wide covers every pixel of a cell
        Mech::Border => Style::default()
            .fill(c0, 1.0)
            .stroke(c0, 0.0, 0.0)
            .fill_border_width(2.0 * CELL / 0.01),
    };
    let lane = if mech.strokes() { "stroke_rgba" } else { "fill_rgba" };
    let vertices = points.len().div_ceil(2);
    let mut v = VMobject::from_points(points).with_style(style);
    if mech == Mech::Polyline {
        // every drawn piece is straight (the null curves are dropped), so
        // the strip is honestly a polyline: the renderer then measures each
        // segment as a capsule instead of a general quadratic
        v = v.with_shape(ShapeTag::Polyline {
            vertices,
            closed: false,
        });
    }
    let mut m = Mobject::from(v);
    assert_eq!(m.buffer.len(), colors.len(), "one colour per record");
    assert!(m.buffer.write_range(lane, 0, colors.as_flattened()), "{lane} lane");
    m
}

/// A flat bar over slots [x0, x1) × [y0, y1) in 1080p px.
fn bar(x0: f64, x1: f64, y0: f64, y1: f64, color: Srgb, opacity: f64) -> fmn::Result<VMobject> {
    let style = Style::default().fill(color, opacity).stroke(color, 0.0, 0.0);
    rect(
        [x_of(x0), y_of(y1), 0.0],
        (x1 - x0) / UNIT_PX,
        (y1 - y0) / UNIT_PX,
        style,
    )
}

fn smoother(t: f64) -> f64 {
    t * t * t * (t * (6.0 * t - 15.0) + 10.0)
}

/// Slide `mob` by `distance` along `axis` over `seconds` from the next frame:
/// a dt updater that shifts it (placement only) to where the ease puts it.
fn slide(stage: &mut Stage<'_>, mob: Mob, axis: usize, distance: f64, seconds: f64) -> fmn::Result<()> {
    let (mut elapsed, mut done) = (0.0, 0.0);
    stage.add_dt_updater(
        mob,
        move |s, me, dt| {
            elapsed += dt;
            let at = distance * smoother((elapsed / seconds).clamp(0.0, 1.0));
            if at != done {
                let mut v = [0.0; 3];
                v[axis] = at - done;
                s.shift(me, v);
                done = at;
            }
        },
        false,
    )?;
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Plan {
    /// The whole sequence.
    Video,
    /// One frame of the first state (byte order).
    Noise,
    /// One frame of the last state (both sorted).
    Stripes,
    /// One frame of sorted rows as row strips, one as column strips.
    Swap,
}

struct Table {
    mech: Mech,
    /// Cells per strip mobject: 255 is one strip per row (or column).
    chunk: usize,
    plan: Plan,
    field: Field,
    pal: Vec<Srgb>,
}

impl Table {
    /// Add a strip; a stroke strip gets a 0.5 px antialiasing band, so a
    /// pixel-aligned edge covers its pixels exactly, also at the renderer's
    /// 2×2 sub-pixel probes (measured: the default 1.5 px band left 93 %
    /// coverage on the edge pixels, a seam between strips; 1 px still left
    /// a third of the cells a few code values off at one edge).
    fn add_strip(&self, stage: &mut Stage<'_>, m: Mobject) -> fmn::Result<Mob> {
        let mob = stage.add(m)?;
        if self.mech.strokes() {
            stage.set_anti_alias_width(mob, 0.5, false);
        }
        Ok(mob)
    }

    /// A row's or a column's cells (slots 1..=255) as strips of `chunk` cells.
    fn strips(&self, stage: &mut Stage<'_>, products: &[u8], run: Run, across: f64) -> fmn::Result<Vec<Mob>> {
        products
            .chunks(self.chunk)
            .enumerate()
            .map(|(i, cells)| {
                let first = 1 + i * self.chunk;
                self.add_strip(stage, strip(self.mech, &self.pal, cells, first, run, across))
            })
            .collect()
    }

    /// Rows 1..=255 in byte order, each as its strips.
    fn row_strips(&self, stage: &mut Stage<'_>) -> fmn::Result<Vec<Vec<Mob>>> {
        (1..=255u8)
            .map(|a| {
                let products: Vec<u8> = (1..=255u8).map(|b| mul(a, b)).collect();
                self.strips(stage, &products, Run::Row, f64::from(a))
            })
            .collect()
    }

    /// Columns 1..=255, each as its strips, with the rows already in log order.
    fn column_strips(&self, stage: &mut Stage<'_>, sorted: bool) -> fmn::Result<Vec<Vec<Mob>>> {
        (1..=255u8)
            .map(|b| {
                let products: Vec<u8> = self.field.exp.iter().map(|&a| mul(a, b)).collect();
                let across = if sorted { self.field.sorted(b) } else { f64::from(b) };
                self.strips(stage, &products, Run::Column, across)
            })
            .collect()
    }

    /// Row 0 (slots 1..=255) and column 0 (slots 0..=255), flat zero colour,
    /// in place or set aside (dimmed).
    fn zero_bars(&self, stage: &mut Stage<'_>, aside: bool) -> fmn::Result<[Mob; 2]> {
        let (shift, opacity) = if aside { (CELL_PX, DIM) } else { (0.0, 1.0) };
        let (lo, hi) = (LEFT_PX + CELL_PX, LEFT_PX + 256.0 * CELL_PX);
        let row = bar(lo, hi, TOP_PX - shift, TOP_PX + CELL_PX - shift, self.pal[0], opacity)?;
        let col = bar(
            LEFT_PX - shift,
            LEFT_PX + CELL_PX - shift,
            TOP_PX,
            TOP_PX + 256.0 * CELL_PX,
            self.pal[0],
            opacity,
        )?;
        Ok([stage.add(row)?, stage.add(col)?])
    }

    fn sequence(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let rows = self.row_strips(stage)?;
        let [zero_row, zero_col] = self.zero_bars(stage, false)?;
        stage.wait(1.5)?;
        // zero has no log: set row 0 and column 0 aside, dimmed
        let up = [0.0, CELL, 0.0];
        let left = [-CELL, 0.0, 0.0];
        play!(stage, 0.5;
            zero_row.animate().shift(up)?.set_opacity(DIM)?,
            zero_col.animate().shift(left)?.set_opacity(DIM)?);
        // rows to log order
        for (a, row) in (1..=255u8).zip(&rows) {
            let distance = -(self.field.sorted(a) - f64::from(a)) * CELL;
            for &m in row {
                slide(stage, m, 1, distance, SORT_S)?;
            }
        }
        stage.wait(SORT_S)?;
        let rows: Vec<Mob> = rows.concat();
        for &m in &rows {
            stage.clear_updaters(m, true);
        }
        stage.wait(0.25)?;
        // the swap: column strips over the same pixels, rows gone
        let cols = self.column_strips(stage, false)?;
        stage.scene_mut().remove(&rows);
        stage.wait(0.25)?;
        // columns to log order
        for (b, col) in (1..=255u8).zip(&cols) {
            let distance = (self.field.sorted(b) - f64::from(b)) * CELL;
            for &m in col {
                slide(stage, m, 0, distance, SORT_S)?;
            }
        }
        stage.wait(SORT_S)?;
        for &m in &cols.concat() {
            stage.clear_updaters(m, true);
        }
        stage.wait(2.0)?;
        Ok(())
    }
}

impl SceneConstruct for Table {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let frame = 1.0 / 60.0;
        match self.plan {
            Plan::Video => self.sequence(stage)?,
            Plan::Noise => {
                self.row_strips(stage)?;
                self.zero_bars(stage, false)?;
                stage.wait(frame)?;
            }
            Plan::Stripes => {
                self.column_strips(stage, true)?;
                self.zero_bars(stage, true)?;
                stage.wait(frame)?;
            }
            Plan::Swap => {
                let rows = self.row_strips(stage)?;
                for (a, row) in (1..=255u8).zip(&rows) {
                    let v = [0.0, -(self.field.sorted(a) - f64::from(a)) * CELL, 0.0];
                    for &m in row {
                        stage.shift(m, v);
                    }
                }
                let rows = rows.concat();
                self.zero_bars(stage, true)?;
                stage.wait(frame)?;
                self.column_strips(stage, false)?;
                stage.scene_mut().remove(&rows);
                stage.wait(frame)?;
            }
        }
        Ok(())
    }
}

fn configure(o: &mut RenderOptions, res: (u32, u32)) {
    o.config.camera.resolution = res;
    o.config.camera.fps = 60;
    o.config.camera.background_color = explainer_kit::palette::BACKGROUND.into();
}

fn log(name: &str, mech: Mech, res: (u32, u32), frames: u64, started: Instant) {
    let secs = started.elapsed().as_secs_f64();
    println!(
        "{name}\t{}\t{}x{}\t{:.2} fps\t{frames} frames\t{secs:.1} s",
        mech.name(),
        res.0,
        res.1,
        frames as f64 / secs
    );
}

fn render(
    name: &str,
    mech: Mech,
    chunk: usize,
    plan: Plan,
    res: (u32, u32),
    opts: impl Fn() -> Result<RenderOptions, fmn::config::ConfigError>,
) -> Result<(), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let (report, _) = encode::retrying(name, || {
        let mut table = Table {
            mech,
            chunk,
            plan,
            field: Field::new(),
            // 3Blue1Brown-ish ramp: blue, teal, green, gold, red
            pal: color_gradient(
                &[
                    "#2B3A8C", "#3D7FD9", "#58C4DD", "#83C167", "#FFD866", "#F0AC5F", "#FC6255",
                ]
                .map(hex),
                256,
            ),
        };
        let mut o = opts().map_err(|e| RenderError::Scene(e.into()))?;
        configure(&mut o, res);
        fmn::render(&mut table, o)
    });
    log(name, mech, res, report?.artifact.frame_count, started);
    Ok(())
}

/// PSNR and SSIM of two frames (ffmpeg), one line each.
fn compare(name: &str, a: &Path, b: &Path) -> std::io::Result<()> {
    let out = std::process::Command::new("ffmpeg")
        .args(["-nostdin", "-hide_banner", "-i"])
        .arg(a)
        .arg("-i")
        .arg(b)
        .args(["-lavfi", "[0][1]psnr;[0][1]ssim", "-f", "null", "-"])
        .output()?;
    for line in String::from_utf8_lossy(&out.stderr).lines() {
        if let Some(k) = line.find("PSNR").or_else(|| line.find("SSIM")) {
            println!("{name}\t{}", &line[k..]);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let usage = "usage: spike_s3_table OUT_DIR [WxH] [polyline|stroke|fill|border] [video|stills] [CELLS]";
    let out = PathBuf::from(args.first().ok_or(usage)?);
    let res = match args.get(1) {
        Some(r) => {
            let (w, h) = r.split_once('x').ok_or("WxH")?;
            (w.parse()?, h.parse()?)
        }
        None => (1920, 1080),
    };
    let mech = match args.get(2).map(String::as_str) {
        Some("stroke") => Mech::Stroke,
        None | Some("polyline") => Mech::Polyline,
        Some("fill") => Mech::Fill,
        Some("border") => Mech::Border,
        Some(_) => return Err(usage.into()),
    };
    let video = match args.get(3).map(String::as_str) {
        None | Some("video") => true,
        Some("stills") => false,
        Some(_) => return Err(usage.into()),
    };
    let chunk: usize = args.get(4).map_or(Ok(255), |s| s.parse())?;
    if !(1..=255).contains(&chunk) {
        return Err(usage.into());
    }
    if out.exists() {
        return Err(format!("{} exists", out.display()).into());
    }
    std::fs::create_dir_all(&out)?;
    println!(
        "GF(256) mod 0x11D checked; {} px per cell at {}x{}; {} cells per strip: {} strips per row, {} strip mobjects per orientation",
        CELL_PX * f64::from(res.1) / 1080.0,
        res.0,
        res.1,
        chunk,
        255usize.div_ceil(chunk),
        255 * 255usize.div_ceil(chunk)
    );
    let png = |dir: &str| {
        let dir = out.join(dir);
        move || RenderOptions::with_format(dir.clone(), RenderFormat::PngSequence)
    };
    if video {
        let mp4 = out.join("table.mp4");
        let quality = Quality {
            quality: 16,
            preset: "slow",
            ..Quality::delivery()
        };
        render("table", mech, chunk, Plan::Video, res, || {
            encode::delivery_options(mp4.clone(), RenderFormat::Mp4, quality.clone())
        })?;
        render("swap", mech, chunk, Plan::Swap, res, png("swap"))?;
        let frame = |i: u32| out.join("swap").join(format!("frame_{i:06}.png"));
        compare("swap", &frame(0), &frame(1))?;
    } else {
        render("noise", mech, chunk, Plan::Noise, res, png("noise"))?;
        render("stripes", mech, chunk, Plan::Stripes, res, png("stripes"))?;
    }
    Ok(())
}
