//! Hoeffding's D — a 3Blue1Brown-style explainer, rendered end to end by
//! franken_manim's native Rust front door (no Python, no LaTeX) and encoded
//! through its one sandboxed external tool, ffmpeg.
//!
//! ```text
//! hoeffding stats                      # the numbers every chapter shows
//! hoeffding glyphs OUT.png             # typesetting probe sheet
//! hoeffding script                     # narration lines as TSV (id, text)
//! hoeffding render all|<chapter>... [--res 1920x1080] [--fps 60] [--out DIR]
//!                  [--narration DIR] [--crf N] [--preset P] [--silent]
//!                  [--encoder libx264|h264_nvenc|hevc_nvenc|h264_videotoolbox]
//!                  [--gpu N] [--threads N] [--max-frames N]
//! hoeffding help                       # the full option list
//! ```

mod chapters;
mod encode;
mod kit;
mod narration;
mod sound;
mod stats;

use std::path::PathBuf;
use std::time::Instant;

use fmn::prelude::*;

use crate::kit::{BACKGROUND, Kit};

const USAGE: &str = "\
usage: hoeffding <command> [options]

commands:
  stats                    print every number the video shows
  script                   print the narration as TSV (id, text)
  glyphs [OUT.png]         render the typesetting probe sheet
  render [all|CHAPTER...]  render chapters to OUT/<chapter>.mp4
                           (01_hook 02_gallery 03_quadruples 04_ranks
                            05_counting 06_formula 07_shuffle 08_outro)
  help                     show this text

render options:
  --res WxH         frame size (default 1920x1080)
  --fps N           frame rate (default 60)
  --out DIR         output directory (default renders)
  --narration DIR   voice-over: one <id>.wav per script line
  --silent          no narration, pad or chimes
  --crf N           delivery quality: x264 CRF, or NVENC constant quality;
                    also enables --preset, --gpu and 256k AAC
  --preset NAME     x264 preset, ultrafast..veryslow (with --crf; NVENC
                    always uses p7)
  --encoder NAME    libx264 (default), h264_nvenc, hevc_nvenc,
                    h264_videotoolbox (no rate control: --crf is ignored)
  --gpu N           NVENC device index (with --crf)
  --threads N       fixed render thread count
  --max-frames N    stop after N frames (profiling; the render then fails
                    by design and publishes nothing)
";

#[derive(Debug)]
struct Args {
    command: String,
    help: bool,
    targets: Vec<String>,
    res: (u32, u32),
    fps: u32,
    out: PathBuf,
    silent: bool,
    narration: Option<PathBuf>,
    crf: Option<u8>,
    preset: Option<&'static str>,
    threads: Option<u32>,
    encoder: String,
    gpu: Option<u32>,
    max_frames: Option<u64>,
}

fn parse_args(argv: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut it = argv.into_iter();
    let command = it.next().ok_or("missing command")?;
    let mut args = Args {
        help: matches!(command.as_str(), "help" | "-h" | "--help"),
        command,
        targets: Vec::new(),
        res: (1920, 1080),
        fps: 60,
        out: PathBuf::from("renders"),
        silent: false,
        narration: None,
        crf: None,
        preset: None,
        threads: None,
        encoder: "libx264".to_owned(),
        gpu: None,
        max_frames: None,
    };
    while let Some(a) = it.next() {
        match a.as_str() {
            "--res" => {
                let r = it.next().ok_or("--res needs WxH")?;
                let (w, h) = r.split_once('x').ok_or("--res needs WxH")?;
                args.res = (
                    w.parse().map_err(|_| "bad width")?,
                    h.parse().map_err(|_| "bad height")?,
                );
            }
            "--fps" => {
                args.fps = it
                    .next()
                    .ok_or("--fps needs N")?
                    .parse()
                    .map_err(|_| "bad fps")?
            }
            "--out" => args.out = PathBuf::from(it.next().ok_or("--out needs DIR")?),
            "--silent" => args.silent = true,
            "-h" | "--help" => args.help = true,
            "--narration" => {
                args.narration = Some(PathBuf::from(it.next().ok_or("--narration needs DIR")?))
            }
            "--encoder" => {
                let e = it.next().ok_or("--encoder needs a name")?;
                if !matches!(
                    e.as_str(),
                    "libx264" | "h264_nvenc" | "hevc_nvenc" | "h264_videotoolbox"
                ) {
                    return Err(
                        "--encoder is libx264, h264_nvenc, hevc_nvenc or h264_videotoolbox".into(),
                    );
                }
                args.encoder = e;
            }
            "--max-frames" => {
                args.max_frames = Some(
                    it.next()
                        .ok_or("--max-frames needs N")?
                        .parse()
                        .map_err(|_| "bad frame count")?,
                )
            }
            "--gpu" => {
                args.gpu = Some(
                    it.next()
                        .ok_or("--gpu needs N")?
                        .parse()
                        .map_err(|_| "bad gpu index")?,
                )
            }
            "--threads" => {
                args.threads = Some(
                    it.next()
                        .ok_or("--threads needs N")?
                        .parse()
                        .map_err(|_| "bad thread count")?,
                )
            }
            "--crf" => {
                args.crf = Some(
                    it.next()
                        .ok_or("--crf needs N")?
                        .parse()
                        .map_err(|_| "bad crf")?,
                )
            }
            "--preset" => {
                args.preset = Some(match it.next().ok_or("--preset needs a name")?.as_str() {
                    "ultrafast" => "ultrafast",
                    "veryfast" => "veryfast",
                    "faster" => "faster",
                    "fast" => "fast",
                    "medium" => "medium",
                    "slow" => "slow",
                    "slower" => "slower",
                    "veryslow" => "veryslow",
                    _ => return Err("unknown x264 preset".into()),
                })
            }
            other if other.starts_with('-') => return Err(format!("unknown option {other}")),
            other => args.targets.push(other.to_owned()),
        }
    }
    Ok(args)
}

fn options(
    path: PathBuf,
    format: RenderFormat,
    res: (u32, u32),
    fps: u32,
) -> Result<RenderOptions, Box<dyn std::error::Error>> {
    let mut options = RenderOptions::with_format(path, format)?;
    options.config.camera.resolution = res;
    options.config.camera.fps = fps;
    options.config.camera.background_color = BACKGROUND.to_owned();
    Ok(options)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    stats::self_check();
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("hoeffding: {error}\n\n{USAGE}");
            std::process::exit(2);
        }
    };
    if args.help {
        print!("{USAGE}");
        return Ok(());
    }
    match args.command.as_str() {
        "stats" => print_stats(),
        "script" => {
            for (id, text) in narration::SCRIPT {
                println!("{id}\t{text}");
            }
        }
        "glyphs" => {
            let out = args
                .targets
                .first()
                .cloned()
                .unwrap_or_else(|| "glyphs.png".into());
            let mut scene = chapters::glyphs::Glyphs { kit: Kit::new()? };
            let report = render(
                &mut scene,
                options(out.into(), RenderFormat::PngSequence, args.res, 1)?,
            )?;
            println!("glyph sheet -> {}", report.artifact.path.display());
        }
        "render" => {
            let all = chapters::registry();
            let wanted: Vec<&chapters::Entry> =
                if args.targets.iter().any(|t| t == "all") || args.targets.is_empty() {
                    all.iter().collect()
                } else {
                    args.targets
                        .iter()
                        .map(|t| {
                            all.iter()
                                .find(|e| e.key == t)
                                .ok_or_else(|| format!("unknown chapter {t}"))
                        })
                        .collect::<Result<_, _>>()?
                };
            if args.crf.is_none() && (args.preset.is_some() || args.gpu.is_some()) {
                eprintln!(
                    "note: --preset and --gpu take effect only together with --crf; \
                     encoding with the engine's default rate control"
                );
            }
            if args.crf.is_some() && args.encoder == "h264_videotoolbox" {
                eprintln!("note: h264_videotoolbox gets no rate-control flags; --crf is ignored");
            }
            std::fs::create_dir_all(&args.out)?;
            let audio = args.out.join("audio");
            std::fs::create_dir_all(&audio)?;
            let narrator = match &args.narration {
                Some(dir) => {
                    let n = narration::Narrator::load(dir)?;
                    println!(
                        "narration: {} lines, {:.1}s of speech from {}",
                        narration::SCRIPT.len(),
                        n.total_seconds(),
                        dir.display()
                    );
                    Some(n)
                }
                None => None,
            };
            let total = Instant::now();
            let mut frames = 0;
            for entry in wanted {
                let path = args.out.join(format!("{}.mp4", entry.key));
                let started = Instant::now();
                // macOS: a transient Darwin killpg EPERM after ffmpeg exits fails
                // the whole render (fm-darwin-killpg-eperm-race-7aae). Publication
                // is atomic, so nothing partial exists; render the chapter again.
                let mut attempt = 1;
                let report = loop {
                    let mut kit = Kit::new()?;
                    if !args.silent {
                        kit.score = Some(sound::Score {
                            dir: audio.clone(),
                            under_voice: narrator.is_some(),
                        });
                        if let Some(n) = &narrator {
                            n.reset();
                        }
                        kit.narrator = narrator.clone();
                    }
                    let mut scene = (entry.make)(kit);
                    let mut opts = options(path.clone(), RenderFormat::Mp4, args.res, args.fps)?;
                    if let Some(n) = args.threads {
                        opts.config.render.threads = fmn::config::config::ThreadPolicy::Fixed(n);
                    }
                    opts.config.file_writer.video_codec = args.encoder.clone();
                    if let Some(n) = args.max_frames {
                        // Profiling scenarios: stop after n frames (the render
                        // then fails by design and publishes nothing).
                        opts.max_frames = n;
                    }
                    if let Some(crf) = args.crf {
                        opts.ffmpeg = Some(encode::capability(encode::Quality {
                            quality: crf,
                            gpu: args.gpu,
                            preset: args.preset.unwrap_or("medium"),
                            audio_bitrate: "256k",
                            timeout: std::time::Duration::from_secs(4 * 3600),
                        }));
                    }
                    match render(scene.as_mut(), opts) {
                        Ok(report) => break report,
                        Err(error)
                            if attempt < 3
                                && format!("{error:?}").contains(
                                    "process-tree completion kill failed: Operation not permitted",
                                ) =>
                        {
                            eprintln!(
                                "{}: transient Darwin EPERM after ffmpeg exit (attempt {attempt}); rendering again",
                                entry.key
                            );
                            attempt += 1;
                        }
                        Err(error) => return Err(error.into()),
                    }
                };
                let secs = started.elapsed().as_secs_f64();
                frames += report.artifact.frame_count;
                println!(
                    "{:<14} {:>5} frames  {:>6.1}s  {:>6.1} fps  -> {}{}",
                    entry.key,
                    report.artifact.frame_count,
                    secs,
                    report.artifact.frame_count as f64 / secs,
                    report.artifact.path.display(),
                    if attempt > 1 {
                        format!("  (attempt {attempt})")
                    } else {
                        String::new()
                    }
                );
            }
            let secs = total.elapsed().as_secs_f64();
            println!(
                "total          {frames:>5} frames  {secs:>6.1}s  {:>6.1} fps",
                frames as f64 / secs
            );
        }
        other => return Err(format!("unknown command {other}").into()),
    }
    Ok(())
}

fn print_stats() {
    use stats::Shape::*;
    for shape in [Line, Parabola, Ring, Cross, Wave, Noise] {
        let pts = stats::shape_points(shape, chapters::gallery::N, chapters::gallery::seed(shape));
        let m = stats::measures(&pts);
        let q = stats::null_q99(&pts, chapters::gallery::TRIALS, 99);
        println!(
            "{:10} r={:+.3} (q99 {:.3})  rho={:+.3} (q99 {:.3})  tau={:+.3} (q99 {:.3})  D={:+.4} (q99 {:.4})",
            shape.label(),
            m.pearson,
            q.pearson,
            m.spearman,
            q.spearman,
            m.kendall,
            q.kendall,
            m.hoeffding,
            q.hoeffding
        );
    }
    let h = stats::hoeffding(&stats::HEIGHTS, &stats::WEIGHTS);
    println!(
        "worked example: R={:?} S={:?} Q={:?} D1={} D2={} D3={} D={}",
        h.r, h.s, h.q, h.d1, h.d2, h.d3, h.d
    );
    println!("C(5000,4) = {}", stats::choose(5000, 4));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(line: &str) -> Result<Args, String> {
        parse_args(line.split_whitespace().map(str::to_owned))
    }

    #[test]
    fn render_parallel_invocation_parses_exactly() {
        // render_parallel.sh with HOEFFDING_ARGS="--threads 16 --encoder h264_nvenc --crf 16".
        let a = parse(
            "render 05_counting --res 3840x2160 --fps 60 --preset slow --narration narration_v5 \
             --out out/05_counting --threads 16 --encoder h264_nvenc --crf 16 --gpu 1",
        )
        .unwrap();
        assert_eq!(a.command, "render");
        assert_eq!(a.targets, ["05_counting"]);
        assert_eq!((a.res, a.fps), ((3840, 2160), 60));
        assert_eq!(a.preset, Some("slow"));
        assert_eq!(a.narration, Some(PathBuf::from("narration_v5")));
        assert_eq!(a.out, PathBuf::from("out/05_counting"));
        assert_eq!((a.threads, a.crf, a.gpu), (Some(16), Some(16), Some(1)));
        assert_eq!(a.encoder, "h264_nvenc");
        assert!(!a.silent && !a.help && a.max_frames.is_none());
    }

    #[test]
    fn defaults() {
        let a = parse("render").unwrap();
        assert!(a.targets.is_empty());
        assert_eq!(
            (a.res, a.fps, a.out),
            ((1920, 1080), 60, PathBuf::from("renders"))
        );
        assert_eq!(
            (a.crf, a.preset, a.gpu, a.threads),
            (None, None, None, None)
        );
        assert_eq!(a.encoder, "libx264");
        assert!(a.narration.is_none() && !a.silent);
    }

    #[test]
    fn remaining_flags_parse() {
        let a =
            parse("render 01_hook 08_outro --silent --max-frames 900 --encoder h264_videotoolbox")
                .unwrap();
        assert_eq!(a.targets, ["01_hook", "08_outro"]);
        assert!(a.silent);
        assert_eq!(a.max_frames, Some(900));
        assert_eq!(a.encoder, "h264_videotoolbox");
    }

    #[test]
    fn typos_are_errors_not_chapter_names() {
        assert_eq!(
            parse("render --fsp 30").unwrap_err(),
            "unknown option --fsp"
        );
        assert_eq!(
            parse("stats --verbose").unwrap_err(),
            "unknown option --verbose"
        );
        assert!(parse("render --preset ludicrous").is_err());
        assert!(parse("render --encoder libx265").is_err());
        assert!(parse("render --res 1920").is_err());
        assert!(parse("render --crf").is_err());
        assert!(parse("").is_err());
    }

    #[test]
    fn help_in_any_position() {
        for line in ["help", "-h", "--help", "render --help", "render 01_hook -h"] {
            assert!(parse(line).unwrap().help, "{line}");
        }
        assert!(!parse("stats").unwrap().help);
    }
}
