//! The shared command line (plan §3.3): each explainer's `main.rs` describes
//! the video as an [`Explainer`] and hands over to [`run`], so every video gets
//! the same commands, flags, safety rules and outputs.
//!
//! The music is scored in two passes (D-22): `render --timeline-only` runs
//! every chapter at 64×36 with the real narration to record its timeline;
//! `score` renders the music for the whole video from all the timelines
//! (cached under a fingerprint), slices it per chapter and runs the §7.6 QA,
//! failing on any violation; `render --score DIR` then lays each chapter's
//! stem under it, refusing a stem whose length is not the scene's.

use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use explainer_score::qa::{self, ScoreFacts};
use explainer_score::score::{self, ChapterPlan, Identity, Section};
use explainer_score::timeline::ChapterTimeline;
use explainer_score::wav;
use fmn::prelude::*;

use crate::captions;
use crate::encode::{self, Quality};
use crate::facts::Facts;
use crate::kit::{CameraShot, Kit};
use crate::layout::Linter;
use crate::narration::{Narrator, Script};
use crate::palette::{BACKGROUND, Palette, ink_style};
use crate::plot::v;

/// One chapter: its key (`NN_name`), its scene, and how intense its music is.
pub struct Chapter {
    pub key: &'static str,
    pub make: fn(Kit) -> Box<dyn SceneConstruct>,
    /// 0..1, the score's intensity for this chapter.
    pub intensity: f64,
}

/// Everything the shared CLI needs to know about a video.
pub struct Explainer {
    pub crate_name: &'static str,
    /// `include_str!("../script.tsv")`.
    pub script: &'static str,
    pub chapters: fn() -> Vec<Chapter>,
    /// Every number the video shows or says; `self_check` runs first, always.
    pub facts: fn() -> Result<Facts, String>,
    /// Every Text and Tex string the video uses, for the glyph probe.
    pub glyph_text: &'static [&'static str],
    pub glyph_tex: &'static [&'static str],
    /// Code and hex the video shows, set in CM Typewriter (`Kit::mono`).
    pub glyph_mono: &'static [&'static str],
    /// The video's musical identity.
    pub identity: fn() -> Identity,
    /// The narration's placement gain (dB): `narration::GAIN_DB` (0) under the
    /// score engine; an older video may keep its own balance until re-scored.
    pub narration_gain_db: f64,
    /// The chapter a script line belongs to, for line ids that do not follow
    /// `NN_k` (None: the line's `NN_` prefix names its chapter).
    pub line_owner: Option<fn(&str) -> Option<&'static str>>,
    /// The video's thumbnail: its central picture and a short title, held
    /// still (None: the video has none yet). `thumbnail` renders it.
    pub thumbnail: Option<fn(Kit) -> Box<dyn SceneConstruct>>,
}

const USAGE: &str = "\
usage: CRATE <command> [options]

commands:
  facts                    print every number the video shows or says (key<TAB>value)
  script                   print the narration as TSV (id, text)
  chapters                 print the chapter keys, in order
  glyphs [OUT_DIR]         render the typesetting probe pages; exits 1 if any string fails
  render [all|CHAPTER...]  render chapters to OUT/<chapter>.mp4
  score                    score the music from the chapters' timelines (OUT/<chapter>.timeline.tsv,
                           or --timelines DIR) into OUT/stems/, with QA; exits 1 on any QA failure
  master                   the PCM master: every chapter's cues replayed through fmn's own mixer and
                           null-tested against the soundtrack it muxed (renders made with --crf),
                           joined to OUT/CRATE_master.mp4 without re-encoding the video
  thumbnail                the video's thumbnail scene as one PNG, OUT/CRATE_thumbnail.png, at
                           --res (YouTube: 1280x720)
  captions                 OUT/CRATE.en.srt and .vtt, sentence by sentence, from the chapters'
                           timelines in OUT and the narration's report.json (--narration DIR)
  help                     show this text

render options:
  --res WxH         frame size (default 1920x1080; drafts 960x540, final 3840x2160)
  --fps N           frame rate (default 60)
  --out DIR         output directory (default renders)
  --narration DIR   the voice-over: one <id>.wav per script line; writes OUT/<chapter>.cues.tsv
                    (id, start s, length s), .cuelog.tsv (every cue, exact) and .timeline.tsv
  --scratch-voice   time the beats from estimated line lengths (no audio; drafts)
  --silent          no narration
  --lint            layout lint at every beat: OUT/<chapter>.layout.tsv
  --timeline-only   the score's first pass: 64x36 at --fps with --narration, timelines only
  --score DIR       lay DIR/<chapter>.music.f32.wav under each chapter (needs --narration)
  --crf N           delivery quality: x264 CRF or NVENC constant quality; also --preset, --gpu, AAC 256k
  --preset NAME     x264 preset, ultrafast..veryslow (with --crf)
  --encoder NAME    libx264 (default), h264_nvenc, hevc_nvenc, h264_videotoolbox
  --gpu N           NVENC device index (with --crf)
  --threads N       fixed render thread count
  --max-frames N    stop after N frames (profiling: OUT/profile/, publishes nothing)

score options:
  --timelines DIR   where the timelines are (default: --out)
  --out DIR         stems go to OUT/stems/ (default renders)
";

#[derive(Debug)]
pub struct Args {
    pub command: String,
    pub help: bool,
    pub targets: Vec<String>,
    pub res: (u32, u32),
    pub fps: u32,
    pub out: PathBuf,
    pub silent: bool,
    pub narration: Option<PathBuf>,
    pub scratch_voice: bool,
    pub lint: bool,
    pub timeline_only: bool,
    pub score: Option<PathBuf>,
    pub timelines: Option<PathBuf>,
    pub max_frames: Option<u64>,
    pub crf: Option<u8>,
    pub preset: Option<&'static str>,
    pub threads: Option<u32>,
    pub encoder: String,
    pub gpu: Option<u32>,
}

fn next_parsed<T: std::str::FromStr>(it: &mut impl Iterator<Item = String>, flag: &str) -> Result<T, String> {
    it.next()
        .ok_or(format!("{flag} needs a value"))?
        .parse()
        .map_err(|_| format!("bad value for {flag}"))
}

/// Parse the command line (without the program name). Typos are errors,
/// never chapter names.
pub fn parse_args(argv: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut it = argv.into_iter();
    let command = it.next().ok_or("missing command")?;
    let mut a = Args {
        help: matches!(command.as_str(), "help" | "-h" | "--help"),
        command,
        targets: Vec::new(),
        res: (1920, 1080),
        fps: 60,
        out: PathBuf::from("renders"),
        silent: false,
        narration: None,
        scratch_voice: false,
        lint: false,
        timeline_only: false,
        score: None,
        timelines: None,
        max_frames: None,
        crf: None,
        preset: None,
        threads: None,
        encoder: "libx264".to_owned(),
        gpu: None,
    };
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--res" => {
                let r = it.next().ok_or("--res needs WxH")?;
                let (w, h) = r.split_once('x').ok_or("--res needs WxH")?;
                a.res = (
                    w.parse().map_err(|_| "bad width")?,
                    h.parse().map_err(|_| "bad height")?,
                );
            }
            "--fps" => a.fps = next_parsed(&mut it, "--fps")?,
            "--out" => a.out = PathBuf::from(it.next().ok_or("--out needs DIR")?),
            "--silent" => a.silent = true,
            "--scratch-voice" => a.scratch_voice = true,
            "--lint" => a.lint = true,
            "--timeline-only" => a.timeline_only = true,
            "--score" => a.score = Some(PathBuf::from(it.next().ok_or("--score needs DIR")?)),
            "--timelines" => a.timelines = Some(PathBuf::from(it.next().ok_or("--timelines needs DIR")?)),
            "--max-frames" => a.max_frames = Some(next_parsed(&mut it, "--max-frames")?),
            "-h" | "--help" => a.help = true,
            "--narration" => a.narration = Some(PathBuf::from(it.next().ok_or("--narration needs DIR")?)),
            "--encoder" => {
                let e = it.next().ok_or("--encoder needs a name")?;
                if !matches!(
                    e.as_str(),
                    "libx264" | "h264_nvenc" | "hevc_nvenc" | "h264_videotoolbox"
                ) {
                    return Err("--encoder is libx264, h264_nvenc, hevc_nvenc or h264_videotoolbox".into());
                }
                a.encoder = e;
            }
            "--gpu" => a.gpu = Some(next_parsed(&mut it, "--gpu")?),
            "--threads" => a.threads = Some(next_parsed(&mut it, "--threads")?),
            "--crf" => a.crf = Some(next_parsed(&mut it, "--crf")?),
            "--preset" => {
                const PRESETS: [&str; 8] = [
                    "ultrafast",
                    "veryfast",
                    "faster",
                    "fast",
                    "medium",
                    "slow",
                    "slower",
                    "veryslow",
                ];
                let p = it.next().ok_or("--preset needs a name")?;
                a.preset = Some(PRESETS.into_iter().find(|&q| q == p).ok_or("unknown x264 preset")?);
            }
            other if other.starts_with('-') => return Err(format!("unknown option {other}")),
            other => a.targets.push(other.to_owned()),
        }
    }
    if a.narration.is_some() && a.scratch_voice {
        return Err("--narration and --scratch-voice are exclusive".into());
    }
    if a.silent && (a.narration.is_some() || a.scratch_voice) {
        return Err("--silent drops the voice; it excludes --narration and --scratch-voice".into());
    }
    if a.timeline_only && (a.narration.is_none() || a.score.is_some() || a.crf.is_some()) {
        return Err("--timeline-only needs --narration and excludes --score and --crf".into());
    }
    if a.score.is_some() && a.narration.is_none() {
        return Err("--score lays stems scored from the real narration; it needs --narration".into());
    }
    Ok(a)
}

/// Renders never overwrite: the engine refuses an existing path. So that
/// drafts can be re-rendered into the same directory, an earlier output is
/// moved to `<dir>/previous/<name>.vN<ext>` (never deleted).
pub fn set_aside(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let dir = path.parent().unwrap_or(Path::new(".")).join("previous");
    std::fs::create_dir_all(&dir)?;
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("output");
    let (stem, ext) = match name.split_once('.') {
        Some((s, e)) => (s.to_owned(), format!(".{e}")),
        None => (name.to_owned(), String::new()),
    };
    let mut n = 1;
    let target = loop {
        let candidate = dir.join(format!("{stem}.v{n}{ext}"));
        if !candidate.exists() {
            break candidate;
        }
        n += 1;
    };
    std::fs::rename(path, &target)?;
    eprintln!("note: moved the earlier {} to {}", path.display(), target.display());
    Ok(())
}

/// Every script line belongs to exactly one chapter: line `NN_k` to the one
/// chapter whose key starts `NN_`, or, given `owner`, to the chapter it names.
pub fn check_script_owners(
    script: &Script,
    keys: &[&str],
    owner: Option<fn(&str) -> Option<&'static str>>,
) -> Result<(), String> {
    for (id, _) in script.lines() {
        let owners = match owner {
            Some(f) => match f(id) {
                Some(k) if keys.contains(&k) => 1,
                Some(k) => return Err(format!("line {id} maps to {k:?}, which is not a chapter")),
                None => 0,
            },
            None => {
                let prefix = id.split('_').next().unwrap_or(id);
                keys.iter().filter(|k| k.starts_with(&format!("{prefix}_"))).count()
            }
        };
        if owners != 1 {
            return Err(format!(
                "line {id} maps to {owners} chapters (it must map to exactly one)"
            ));
        }
    }
    Ok(())
}

/// Run the CLI; returns the process exit code (0 ok, 1 a failed check, 2 usage).
pub fn run(e: &Explainer, argv: impl IntoIterator<Item = String>) -> i32 {
    // a wrong number must never render, or even print
    let facts = match (e.facts)().and_then(|f| f.self_check().map(|()| f)) {
        Ok(f) => f,
        Err(msg) => {
            eprintln!("{}: facts self-check failed:\n{msg}", e.crate_name);
            return 1;
        }
    };
    let usage = USAGE.replace("CRATE", e.crate_name);
    let args = match parse_args(argv) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("{}: {msg}\n\n{usage}", e.crate_name);
            return 2;
        }
    };
    if args.help {
        print!("{usage}");
        return 0;
    }
    let result: Result<i32, Box<dyn Error>> = (|| {
        let script = Script::parse(e.script)?;
        match args.command.as_str() {
            "facts" => print!("{}", facts.print()),
            "script" => {
                for (id, text) in script.lines() {
                    println!("{id}\t{text}");
                }
            }
            "chapters" => {
                for c in (e.chapters)() {
                    println!("{}", c.key);
                }
            }
            "glyphs" => return glyphs(e, &args),
            "render" => render_chapters(e, &args, &script)?,
            "score" => return score_video(e, &args),
            "master" => return master_video(e, &args),
            "captions" => return captions_video(e, &args),
            "thumbnail" => return thumbnail(e, &args),
            other => return Err(format!("unknown command {other}").into()),
        }
        Ok(0)
    })();
    match result {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{}: {err}", e.crate_name);
            1
        }
    }
}

fn options(path: PathBuf, format: RenderFormat, res: (u32, u32), fps: u32) -> Result<RenderOptions, Box<dyn Error>> {
    let mut o = RenderOptions::with_format(path, format)?;
    o.config.camera.resolution = res;
    o.config.camera.fps = fps;
    o.config.camera.background_color = BACKGROUND.to_owned();
    Ok(o)
}

/// A scene borrowed for the camera route, whose factory returns its program
/// by value.
struct Borrowed<'a>(&'a mut dyn SceneConstruct);

impl SceneConstruct for Borrowed<'_> {
    fn name(&self) -> &str {
        self.0.name()
    }
    fn tex_preflight(&self) -> Vec<TypesetRequest<'_>> {
        self.0.tex_preflight()
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.0.construct(stage)
    }
}

/// A chapter that asked for the camera route ([`Kit::request_camera`]):
/// rendered through [`encode::render_camera`] with `opts` (see
/// [`camera_options`]); the rig starts at the shot's angles and reaches the
/// scene through the kit.
fn render_camera_chapter(
    kit: &Kit,
    shot: CameraShot,
    scene: &mut dyn SceneConstruct,
    opts: RenderOptions,
) -> Result<fmn::rendering::RenderReport, fmn::rendering::RenderError> {
    encode::render_camera(
        |s, base| {
            let mut start = base.clone();
            start
                .frame
                .set_euler_angles(Some(shot.theta), Some(shot.phi), None)
                .map_err(fmn::scene::SceneError::from)?;
            let rig = fmn::scene::CameraRig::new(s, &start)?;
            kit.bind_camera(rig, start);
            Ok((Borrowed(scene), rig))
        },
        opts,
    )
}

/// A camera-route chapter's options: the kit's runner-wrapped encoder, as
/// a delivery render on the retained route has it. A draft (no `--crf`) gets
/// fmn's own default rate control, x264 CRF 23 at preset medium, through
/// that runner: the camera route installs no ffmpeg of its own.
fn camera_options(path: &Path, opts: RenderOptions) -> Result<RenderOptions, Box<dyn Error>> {
    if opts.ffmpeg.is_some() {
        return Ok(opts);
    }
    let mut o = encode::delivery_options(
        path.to_path_buf(),
        RenderFormat::Mp4,
        Quality {
            quality: 23,
            preset: "medium",
            ..Quality::delivery()
        },
    )?;
    o.config = opts.config;
    o.max_frames = opts.max_frames;
    Ok(o)
}

fn render_chapters(e: &Explainer, args: &Args, script: &Script) -> Result<(), Box<dyn Error>> {
    let all = (e.chapters)();
    let keys: Vec<&str> = all.iter().map(|c| c.key).collect();
    check_script_owners(script, &keys, e.line_owner)?;
    let wanted: Vec<&Chapter> = if args.targets.is_empty() || args.targets.iter().any(|t| t == "all") {
        all.iter().collect()
    } else {
        args.targets
            .iter()
            .map(|t| {
                all.iter()
                    .find(|c| c.key == t)
                    .ok_or_else(|| format!("unknown chapter {t}"))
            })
            .collect::<Result<_, _>>()?
    };
    if args.crf.is_none() && (args.preset.is_some() || args.gpu.is_some()) {
        eprintln!("note: --preset and --gpu take effect only together with --crf");
    }
    // A lane can review its finished chapter while another chapter awaits
    // new narration. The full render still validates every scripted line.
    let script = script.select(|id| {
        wanted.iter().any(|c| match e.line_owner {
            Some(owner) => owner(id) == Some(c.key),
            None => c.key.starts_with(&format!("{}_", id.split('_').next().unwrap_or(id))),
        })
    });
    std::fs::create_dir_all(&args.out)?;
    let narrator = match (&args.narration, args.scratch_voice) {
        (Some(dir), _) => Some(Narrator::load(&script, dir, e.narration_gain_db)?),
        (None, true) => Some(Narrator::estimate(&script)),
        _ => None,
    };
    let (res, profiling) = if args.timeline_only {
        ((64, 36), false)
    } else {
        (args.res, args.max_frames.is_some())
    };
    let total = Instant::now();
    let mut frames = 0;
    for c in wanted {
        let out_of = |ext: &str| args.out.join(format!("{}.{ext}", c.key));
        let path = if profiling {
            let dir = args.out.join("profile");
            std::fs::create_dir_all(&dir)?;
            dir.join(format!("{}.{}.mp4", c.key, std::process::id()))
        } else if args.timeline_only {
            let dir = args.out.join("timeline");
            std::fs::create_dir_all(&dir)?;
            let p = dir.join(format!("{}.mp4", c.key));
            set_aside(&p)?;
            p
        } else {
            let p = out_of("mp4");
            set_aside(&p)?;
            set_aside(&out_of("layout.tsv"))?;
            p
        };
        let mut linter = None;
        let started = Instant::now();
        // macOS: a transient Darwin killpg EPERM after ffmpeg exits fails the
        // whole render (fm-darwin-killpg-eperm-race-7aae). Publication is
        // atomic, so nothing partial exists; render the chapter again.
        let mut attempt = 1;
        let (report, timeline) = loop {
            let mut kit = Kit::new()?;
            kit.chapter = c.key.to_owned();
            if args.lint && !args.timeline_only {
                let l = Rc::new(Linter::new());
                kit = kit.with_lint(Rc::clone(&l));
                linter = Some(l);
            }
            if let Some(n) = &narrator {
                n.reset(); // every chapter is its own scene; its clock starts at zero
                kit = kit.with_narrator(Rc::clone(n));
            }
            if let Some(dir) = &args.score {
                kit.stem = Some(dir.join(format!("{}.music.f32.wav", c.key)));
            }
            let finished = Rc::clone(&kit.finished);
            // shares the kit's camera slot: `make` may ask for the camera route
            let camera = kit.clone();
            let mut scene = (c.make)(kit);
            let mut opts = options(path.clone(), RenderFormat::Mp4, res, args.fps)?;
            if let Some(n) = args.threads {
                opts.config.render.threads = fmn::config::config::ThreadPolicy::Fixed(n);
            }
            opts.config.file_writer.video_codec = args.encoder.clone();
            if let Some(n) = args.max_frames {
                opts.max_frames = n;
            }
            if let Some(crf) = args.crf {
                // a delivery render keeps the soundtrack fmn muxes, for the
                // PCM master's null test
                let capture = args.out.join("soundtrack").join(c.key);
                if !profiling {
                    set_aside(&capture)?;
                }
                let mut o = encode::delivery_options(
                    path.clone(),
                    RenderFormat::Mp4,
                    Quality {
                        quality: crf,
                        gpu: args.gpu,
                        preset: args.preset.unwrap_or("medium"),
                        capture_dir: (!profiling).then_some(capture),
                        ..Quality::delivery()
                    },
                )?;
                o.config = opts.config.clone();
                o.max_frames = opts.max_frames;
                opts = o;
            }
            let rendered = match camera.camera_shot() {
                None => render(scene.as_mut(), opts),
                Some(shot) => {
                    let opts = camera_options(&path, opts)?;
                    render_camera_chapter(&camera, shot, scene.as_mut(), opts)
                }
            };
            match rendered {
                Ok(report) => break (report, finished.borrow().clone()),
                Err(error) if attempt < encode::RENDER_ATTEMPTS && encode::is_transient_eperm(&error) => {
                    // the race clusters under heavy host load: back off
                    eprintln!(
                        "{}: transient Darwin EPERM after ffmpeg exit (attempt {attempt}); rendering again",
                        c.key
                    );
                    std::thread::sleep(std::time::Duration::from_millis(500 << attempt));
                    attempt += 1;
                }
                Err(error) => return Err(error.into()),
            }
        };
        let secs = started.elapsed().as_secs_f64();
        frames += report.artifact.frame_count;
        let clipped = report.artifact.soundtrack.as_ref().map_or(0, |s| s.clipped_samples);
        if clipped > 0 {
            eprintln!("WARNING {}: {clipped} soundtrack samples clipped", c.key);
        }
        if let Some(n) = narrator.as_ref().filter(|_| !profiling) {
            // the skill's cue_sheets.py reads `id, start s, length s` (for
            // review sheets only: timing lives in the exact logs below)
            let cues: String = timeline
                .iter()
                .flat_map(|t| t.lines.iter().map(move |l| (l, t.fps)))
                .map(|(l, fps)| {
                    format!(
                        "{}\t{:.3}\t{:.3}\n",
                        l.id,
                        l.start_frame as f64 / f64::from(fps),
                        n.length(&l.id).unwrap_or(0.0)
                    )
                })
                .collect();
            if args.narration.is_some() {
                for (ext, body) in [("cues.tsv", cues), ("cuelog.tsv", n.cue_log())] {
                    let p = out_of(ext);
                    set_aside(&p)?;
                    std::fs::write(&p, body)?;
                }
                match &timeline {
                    Some(t) => {
                        let p = out_of("timeline.tsv");
                        set_aside(&p)?;
                        std::fs::write(&p, t.to_text())?;
                    }
                    None => eprintln!("note {}: no timeline (the chapter does not call kit.finish)", c.key),
                }
            }
        }
        if let Some(l) = linter.as_ref().filter(|_| !profiling) {
            std::fs::write(out_of("layout.tsv"), l.report())?;
            if l.count() > 0 {
                eprintln!(
                    "LAYOUT {}: {} findings -> {}",
                    c.key,
                    l.count(),
                    out_of("layout.tsv").display()
                );
            }
        }
        println!(
            "{:<14} {:>5} frames  {:>6.1}s  {:>6.1} fps  -> {}{}",
            c.key,
            report.artifact.frame_count,
            secs,
            report.artifact.frame_count as f64 / secs.max(1e-9),
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
        frames as f64 / secs.max(1e-9)
    );
    Ok(())
}

/// The `score` command: the whole video's music from every chapter's
/// timeline, sliced per chapter into `OUT/stems/<chapter>.music.f32.wav`,
/// with a QA report per chapter. Returns 1 when any chapter fails QA (the
/// stems and reports are written either way, for inspection).
fn score_video(e: &Explainer, args: &Args) -> Result<i32, Box<dyn Error>> {
    let chapters = (e.chapters)();
    let dir = args.timelines.clone().unwrap_or_else(|| args.out.clone());
    let last = chapters.len().saturating_sub(1);
    let sections = chapters
        .iter()
        .enumerate()
        .map(|(k, c)| {
            let path = dir.join(format!("{}.timeline.tsv", c.key));
            let timeline = ChapterTimeline::read(&path).map_err(|err| {
                io::Error::new(
                    err.kind(),
                    format!("{}: {err} (run render --timeline-only first)", path.display()),
                )
            })?;
            Ok(Section {
                timeline,
                plan: ChapterPlan {
                    intensity: c.intensity,
                    resolve: k == last,
                },
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    let identity = (e.identity)();
    let stems = args.out.join("stems");
    std::fs::create_dir_all(stems.join("qa"))?;
    let fp = format!("{:016x}\n", score::fingerprint(&identity, &sections, None)?);
    let fp_path = stems.join("score.fingerprint");
    let up_to_date = std::fs::read_to_string(&fp_path).is_ok_and(|f| f == fp)
        && chapters
            .iter()
            .all(|c| stems.join(format!("{}.music.f32.wav", c.key)).exists());
    if up_to_date {
        let failed: Vec<&str> = chapters
            .iter()
            .filter(|c| {
                std::fs::read_to_string(stems.join(format!("qa/{}.json", c.key)))
                    .map_or(true, |j| !j.contains("\"passed\":true"))
            })
            .map(|c| c.key)
            .collect();
        println!("score up to date ({}); QA failing in: {failed:?}", fp.trim());
        return Ok(i32::from(!failed.is_empty()));
    }
    let started = Instant::now();
    let s = score::render(&identity, &sections, None)?;
    println!(
        "scored {:.1} s of music in {:.1} s: {} chords, {} notes, {} phrases; stem {:.1} dBTP",
        s.music.len() as f64 / 48_000.0,
        started.elapsed().as_secs_f64(),
        s.chords.len(),
        s.events.len(),
        s.phrases.len(),
        s.true_peak_db
    );
    if !s.skipped_marks.is_empty() {
        // deep in a sentence, at the very end, or too close to another cue
        println!("marks the score could not answer: {:?}", s.skipped_marks);
    }
    let mut failed = Vec::new();
    for (k, c) in chapters.iter().enumerate() {
        let music = s.chapter_music(k);
        let stem = stems.join(format!("{}.music.f32.wav", c.key));
        set_aside(&stem)?;
        wav::write(&stem, &music, wav::Format::F32)?;
        let report = qa::evaluate(
            c.key,
            &s.chapter_voice(k),
            &music,
            music.len(),
            Some(&ScoreFacts::chapter(&s, k)),
        );
        std::fs::write(stems.join(format!("qa/{}.json", c.key)), report.to_json())?;
        let verdict = if report.passed() {
            "PASS".to_owned()
        } else {
            format!(
                "FAIL {:?}",
                report.failures().iter().map(|m| m.name.as_str()).collect::<Vec<_>>()
            )
        };
        println!("{:<14} QA {verdict}", c.key);
        if !report.passed() {
            failed.push(c.key);
        }
    }
    std::fs::write(&fp_path, &fp)?;
    Ok(i32::from(!failed.is_empty()))
}

/// The `captions` command (plan §13): `OUT/CRATE.en.srt` and `.en.vtt` for
/// the whole video, sentence by sentence, from every chapter's timeline in
/// OUT and the narration skill's `report.json` (in the `--narration` DIR,
/// beside the WAVs); refused (exit 1) if any caption overlaps or overflows.
fn captions_video(e: &Explainer, args: &Args) -> Result<i32, Box<dyn Error>> {
    let dir = args
        .narration
        .as_ref()
        .ok_or("captions needs --narration DIR (the report.json beside the WAVs)")?;
    let report = captions::read_report(&dir.join("report.json"))?;
    let chapters = (e.chapters)()
        .iter()
        .map(|c| ChapterTimeline::read(&args.out.join(format!("{}.timeline.tsv", c.key))))
        .collect::<io::Result<Vec<_>>>()?;
    let caps = captions::video_captions(&chapters, &report)?;
    if let Err(m) = captions::validate(&caps) {
        eprintln!("captions refused: {m}");
        return Ok(1);
    }
    let sentences: usize = chapters
        .iter()
        .flat_map(|c| &c.lines)
        .map(|l| report.get(&l.id).map_or(0, Vec::len))
        .sum();
    for (ext, body) in [("en.srt", captions::to_srt(&caps)), ("en.vtt", captions::to_vtt(&caps))] {
        let p = args.out.join(format!("{}.{ext}", e.crate_name));
        set_aside(&p)?;
        std::fs::write(&p, body)?;
    }
    println!(
        "captions: {sentences} sentences as {} captions -> {}",
        caps.len(),
        args.out.join(format!("{}.en.srt", e.crate_name)).display()
    );
    Ok(0)
}

/// The `master` command (plan §7.7): replay every chapter's cue log through
/// fmn's own mixer, require it to equal the soundtrack fmn muxed (captured by
/// `render --crf`), concatenate, apply one fixed gain, and join the chapters'
/// video without re-encoding. Gates: the joined frame count is the sum of the
/// chapters'; the audio lasts as long as the picture (within one AAC frame).
fn master_video(e: &Explainer, args: &Args) -> Result<i32, Box<dyn Error>> {
    use crate::master;
    let chapters = (e.chapters)();
    let mut programme = explainer_score::Stereo::default();
    let (mut total_frames, mut fps_all) = (0u64, None);
    let mut mp4s = Vec::new();
    for c in &chapters {
        let tl = ChapterTimeline::read(&args.out.join(format!("{}.timeline.tsv", c.key)))?;
        if *fps_all.get_or_insert(tl.fps) != tl.fps {
            return Err(format!("{}: {} fps, the others {}", c.key, tl.fps, fps_all.unwrap_or(0)).into());
        }
        let cues = master::parse_cue_log(&std::fs::read_to_string(
            args.out.join(format!("{}.cuelog.tsv", c.key)),
        )?)?;
        let replay = master::replay(&cues, tl.frames, tl.fps)?;
        let dir = args.out.join("soundtrack").join(c.key);
        let captured = (0..)
            .map(|n| dir.join(format!("soundtrack_{n}.wav")))
            .take_while(|p| p.exists())
            .last()
            .ok_or_else(|| {
                format!(
                    "{}: no captured soundtrack in {} (render it with --crf)",
                    c.key,
                    dir.display()
                )
            })?;
        master::null_test(&replay, &std::fs::read(&captured)?)
            .map_err(|m| format!("{}: null test failed: {m}", c.key))?;
        let audio = master::to_stereo(&replay);
        let want = explainer_score::units::frame_to_sample(tl.frames, tl.fps) as usize;
        if audio.len() != want {
            return Err(format!("{}: soundtrack {} samples, picture {want}", c.key, audio.len()).into());
        }
        println!(
            "{:<14} replay = fmn's soundtrack, bit for bit ({} cues, {} samples)",
            c.key,
            cues.len(),
            audio.len()
        );
        programme.l.extend(&audio.l);
        programme.r.extend(&audio.r);
        total_frames += tl.frames;
        mp4s.push(args.out.join(format!("{}.mp4", c.key)));
    }
    let fps = fps_all.ok_or("no chapters")?;
    let (gain, lufs, tp) = master::apply_master_gain(&mut programme);
    let work = args.out.join("master");
    std::fs::create_dir_all(&work)?;
    let wav = work.join(format!("{}.master.wav", e.crate_name));
    set_aside(&wav)?;
    explainer_score::wav::write_master(&wav, &programme, explainer_score::wav::Format::F32)?;
    let out = args.out.join(format!("{}_master.mp4", e.crate_name));
    set_aside(&out)?;
    master::join(&mp4s, &wav, &work.join("join"), &out)?;
    let (frames, secs) = master::probe(&out)?;
    let picture = total_frames as f64 / f64::from(fps);
    println!(
        "master -> {}: gain {gain:+.2} dB, {lufs:.2} LUFS, {tp:.2} dBTP; {frames} frames (chapters {total_frames}); audio {secs:.3} s (picture {picture:.3} s)",
        out.display()
    );
    let grid = master::check_timestamps(&out, fps, total_frames);
    if let Err(e) = &grid {
        eprintln!("master gate failed: {e}");
    }
    let ok = frames == total_frames && (secs - picture).abs() <= 1024.0 / 48_000.0 + 1e-3 && grid.is_ok();
    if !ok {
        eprintln!("master gates failed: frames {frames} vs {total_frames}, audio {secs:.3} s vs {picture:.3} s");
    }
    Ok(i32::from(!ok))
}

/// The `thumbnail` command: the video's thumbnail scene as one PNG,
/// `OUT/<crate>_thumbnail.png` at `--res`. The scene ends on a one-second
/// hold, which at 1 fps is exactly one frame; an earlier thumbnail is moved
/// aside, never overwritten.
fn thumbnail(e: &Explainer, args: &Args) -> Result<i32, Box<dyn Error>> {
    let make = e.thumbnail.ok_or("this video has no thumbnail scene yet")?;
    std::fs::create_dir_all(&args.out)?;
    let frames = args.out.join("thumbnail");
    set_aside(&frames)?;
    let mut scene = make(Kit::new()?);
    let report = render(scene.as_mut(), options(frames, RenderFormat::PngSequence, args.res, 1)?)?;
    let mut pngs: Vec<PathBuf> = std::fs::read_dir(&report.artifact.path)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "png"))
        .collect();
    pngs.sort();
    let last = pngs.last().ok_or("the thumbnail scene drew no frame")?;
    let png = args.out.join(format!("{}_thumbnail.png", e.crate_name));
    set_aside(&png)?;
    std::fs::copy(last, &png)?;
    println!("thumbnail {}x{} -> {}", args.res.0, args.res.1, png.display());
    Ok(0)
}

/// The `glyphs` command: every Text and Tex string the video uses, on PNG
/// pages; each failure reported by name, exit 1 if any (the pages are written
/// either way). A string can typeset and still be wrong (math ρ/θ draw as
/// ϱ/ϑ), so look at every page.
fn glyphs(e: &Explainer, args: &Args) -> Result<i32, Box<dyn Error>> {
    let out = args.targets.first().cloned().unwrap_or_else(|| "glyphs".into());
    set_aside(Path::new(&out))?;
    let mut scene = Glyphs {
        kit: Kit::new()?,
        text: e.glyph_text,
        tex: e.glyph_tex,
        mono: e.glyph_mono,
        failures: Vec::new(),
    };
    let report = render(&mut scene, options(out.into(), RenderFormat::PngSequence, args.res, 1)?)?;
    println!("glyph sheet -> {}", report.artifact.path.display());
    if scene.failures.is_empty() {
        Ok(0)
    } else {
        eprintln!(
            "glyph probe: {} of {} strings failed to typeset:",
            scene.failures.len(),
            e.glyph_text.len() + e.glyph_tex.len() + e.glyph_mono.len()
        );
        for line in &scene.failures {
            eprintln!("  {line}");
        }
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    Text,
    Tex,
    Mono,
}

struct Glyphs {
    kit: Kit,
    text: &'static [&'static str],
    tex: &'static [&'static str],
    mono: &'static [&'static str],
    failures: Vec<String>,
}

impl SceneConstruct for Glyphs {
    fn name(&self) -> &str {
        "glyphs"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        const TOP: f64 = 3.5;
        const BOTTOM: f64 = -3.3;
        self.failures.clear();
        let mut y = TOP;
        let rows = self
            .text
            .iter()
            .map(|s| (Row::Text, *s, 0.8))
            .chain(self.tex.iter().map(|s| (Row::Tex, *s, 1.25)))
            .chain(self.mono.iter().map(|s| (Row::Mono, *s, 0.8)));
        for (row, s, step) in rows {
            if y < BOTTOM {
                // page full: capture it (1 s at the probe's 1 fps is one PNG)
                stage.wait(1.0)?;
                stage.scene_mut().clear();
                y = TOP;
            }
            let added = if row == Row::Tex {
                match Tex::new(s)
                    .font_size(36.0)
                    .style(ink_style(Palette::ink()))
                    .build(stage.tex_engine()?)
                {
                    Ok(t) => Some(stage.add(t)?),
                    Err(err) => {
                        self.failures.push(format!("TEX FAIL {s:?}: {err}"));
                        None
                    }
                }
            } else {
                let mut t = Text::new(s).font_size(34.0).style(ink_style(Palette::ink()));
                if row == Row::Mono {
                    t = t.font(crate::kit::MONO_FAMILY);
                }
                match t.build(&self.kit.book) {
                    Ok(t) => Some(stage.add(t)?),
                    Err(err) => {
                        self.failures.push(format!("{row:?} FAIL {s:?}: {err}"));
                        None
                    }
                }
            };
            if let Some(m) = added {
                stage.move_to(m, v(-6.8, y), LEFT);
            }
            y -= step;
        }
        stage.wait(1.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(line: &str) -> Result<Args, String> {
        parse_args(line.split_whitespace().map(str::to_owned))
    }

    #[test]
    fn final_render_invocation_parses_exactly() {
        let a = parse(
            "render 01_intro --res 3840x2160 --fps 60 --preset slow --narration narration \
             --out out/01_intro --threads 16 --encoder h264_nvenc --crf 16 --gpu 1 --score stems",
        )
        .unwrap();
        assert_eq!(a.targets, ["01_intro"]);
        assert_eq!((a.res, a.fps), ((3840, 2160), 60));
        assert_eq!(a.narration, Some(PathBuf::from("narration")));
        assert_eq!((a.threads, a.crf, a.gpu), (Some(16), Some(16), Some(1)));
        assert_eq!((a.encoder.as_str(), a.preset), ("h264_nvenc", Some("slow")));
        assert_eq!(a.score, Some(PathBuf::from("stems")));
    }

    #[test]
    fn typos_and_contradictions_are_errors() {
        assert_eq!(parse("render --fsp 30").unwrap_err(), "unknown option --fsp");
        assert!(parse("render --preset ludicrous").is_err());
        assert!(parse("render --encoder libx265").is_err());
        assert!(parse("render --res 1920").is_err());
        assert!(parse("render --crf").is_err());
        assert!(parse("render --narration n --scratch-voice").is_err());
        assert!(parse("render --silent --scratch-voice").is_err());
        assert!(
            parse("render --timeline-only").is_err(),
            "the timeline pass needs the real voice"
        );
        assert!(parse("render --timeline-only --narration n --score s").is_err());
        assert!(
            parse("render --score s").is_err(),
            "stems need the narration they were scored from"
        );
        assert!(parse("").is_err());
        assert!(parse("help").unwrap().help);
        assert!(parse("render --help").unwrap().help);
    }

    #[test]
    fn set_aside_keeps_every_earlier_output() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!("set_aside_test_{}_{nanos}", std::process::id()));
        assert!(!dir.exists());
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("01_intro.cues.tsv");
        for round in 1..=2 {
            std::fs::write(&f, format!("round {round}")).unwrap();
            set_aside(&f).unwrap();
            assert!(!f.exists());
        }
        let prev = dir.join("previous");
        assert_eq!(
            std::fs::read_to_string(prev.join("01_intro.v1.cues.tsv")).unwrap(),
            "round 1"
        );
        assert_eq!(
            std::fs::read_to_string(prev.join("01_intro.v2.cues.tsv")).unwrap(),
            "round 2"
        );
        set_aside(&dir.join("absent.mp4")).unwrap();
    }

    #[test]
    fn every_script_line_belongs_to_exactly_one_chapter() {
        let script = Script::parse("01_1\ta\n01_2\tb\n02_1\tc\n").unwrap();
        assert!(check_script_owners(&script, &["01_intro", "02_outro"], None).is_ok());
        assert!(
            check_script_owners(&script, &["01_intro"], None)
                .unwrap_err()
                .contains("02_1")
        );
        assert!(check_script_owners(&script, &["01_intro", "01_again", "02_outro"], None).is_err());
        // ids that do not follow NN_k, with an explicit map
        let v5 = Script::parse("h1\ta\ng1\tb\n").unwrap();
        let owner: fn(&str) -> Option<&'static str> = |id| match id.chars().next() {
            Some('h') => Some("01_hook"),
            Some('g') => Some("02_gallery"),
            _ => None,
        };
        assert!(check_script_owners(&v5, &["01_hook", "02_gallery"], Some(owner)).is_ok());
        assert!(
            check_script_owners(&v5, &["01_hook"], Some(owner))
                .unwrap_err()
                .contains("not a chapter")
        );
        let none = Script::parse("x1\tc\n").unwrap();
        assert!(check_script_owners(&none, &["01_hook"], Some(owner)).is_err());
    }
}
