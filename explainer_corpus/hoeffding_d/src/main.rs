//! Hoeffding's D — a 3Blue1Brown-style explainer, rendered end to end by
//! franken_manim's native Rust front door (no Python, no LaTeX) and encoded
//! through its one sandboxed external tool, ffmpeg. Built on the corpus's
//! shared kit and CLI (`explainer_kit`).
//!
//! ```text
//! hoeffding stats                      # the numbers every chapter shows
//! hoeffding glyphs OUT_DIR             # typesetting probe pages
//! hoeffding script                     # narration lines as TSV (id, text)
//! hoeffding render all|<chapter>... [--res 1920x1080] [--fps 60] [--out DIR]
//!                  [--narration DIR] [--crf N] [--preset P] [--silent] ...
//! hoeffding help                       # the full option list
//!
//! # the score (two passes), then the PCM master
//! hoeffding render --timeline-only --narration DIR --out OUT
//! hoeffding score --out OUT            # stems and QA in OUT/stems
//! hoeffding render all --narration DIR --score OUT/stems --crf 16 --out OUT
//! hoeffding master --out OUT
//!
//! HOEFFDING_V5_SOUND=1 hoeffding ...   # the v5 pad and chimes instead (the A/B's old side)
//! ```

mod chapters;
mod kit;
mod sound;
mod stats;

pub(crate) use explainer_kit::play;
use explainer_kit::{Explainer, Facts};

/// The narration lines, with the v5 ids (h1..o4) so `narration_v5` matches.
const SCRIPT: &str = include_str!("../script.tsv");

/// v5's voice/score balance: the narration sat 9.4 dB down under the v5 pad.
/// v6 places it at 0 dB under the score engine (D-21).
const V5_NARRATION_GAIN_DB: f64 = -9.4;

/// `HOEFFDING_V5_SOUND=1`: the v5 pad and chimes, with the v5 balance, for
/// the A/B's old side (the picture is v6's either way).
pub(crate) fn v5_sound() -> bool {
    std::env::var_os("HOEFFDING_V5_SOUND").is_some_and(|v| v == "1")
}

/// The chapter of a v5 line id: its first letter (h1 → 01_hook, ...).
fn line_owner(id: &str) -> Option<&'static str> {
    Some(match id.chars().next()? {
        'h' => "01_hook",
        'g' => "02_gallery",
        'q' => "03_quadruples",
        'r' => "04_ranks",
        'c' => "05_counting",
        'f' => "06_formula",
        's' => "07_shuffle",
        'o' => "08_outro",
        _ => return None,
    })
}

/// Hoeffding pins its numbers in `stats::self_check` (asserts); the shared
/// CLI runs this before every command. Its numbers print with `stats`.
fn facts() -> Result<Facts, String> {
    std::panic::catch_unwind(stats::self_check)
        .map_err(|_| "stats::self_check failed".to_owned())?;
    Ok(Facts::new())
}

const EXPLAINER: Explainer = Explainer {
    crate_name: "hoeffding",
    script: SCRIPT,
    chapters: chapters::registry,
    facts,
    glyph_text: chapters::glyphs::TEXT,
    glyph_tex: chapters::glyphs::TEX,
    identity: explainer_score::score::Identity::hoeffding,
    narration_gain_db: explainer_kit::narration::GAIN_DB,
    line_owner: Some(line_owner),
};

/// The same video with the v5 sound (see [`v5_sound`]).
const EXPLAINER_V5: Explainer = Explainer {
    narration_gain_db: V5_NARRATION_GAIN_DB,
    ..EXPLAINER
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("stats") && args.len() == 1 {
        stats::self_check();
        print_stats();
        return;
    }
    let e = if v5_sound() {
        &EXPLAINER_V5
    } else {
        &EXPLAINER
    };
    std::process::exit(explainer_kit::cli::run(e, args));
}

/// Every number the video shows (the v5 `stats` output, unchanged).
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
    use std::path::PathBuf;

    use explainer_kit::cli::{Args, check_script_owners, parse_args};
    use explainer_kit::{Script, narration};

    use super::*;

    fn parse(line: &str) -> Result<Args, String> {
        parse_args(line.split_whitespace().map(str::to_owned))
    }

    // the v5 CLI tests, against the shared parser that replaced it

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

    // the migration's own checks

    #[test]
    fn every_v5_line_maps_to_exactly_one_chapter() {
        let script = Script::parse(SCRIPT).unwrap();
        assert_eq!(script.lines().len(), 61, "the v5 script has 61 lines");
        let keys: Vec<&str> = chapters::registry().iter().map(|c| c.key).collect();
        check_script_owners(&script, &keys, Some(line_owner)).unwrap();
        // every chapter speaks at least one line
        for k in &keys {
            assert!(
                script
                    .lines()
                    .iter()
                    .any(|(id, _)| line_owner(id) == Some(*k)),
                "{k} is silent"
            );
        }
    }

    #[test]
    fn v6_speaks_at_0_db_and_the_v5_side_keeps_its_balance() {
        assert_eq!(EXPLAINER.narration_gain_db, narration::GAIN_DB);
        assert_eq!(EXPLAINER_V5.narration_gain_db, -9.4);
        assert_eq!(EXPLAINER_V5.crate_name, EXPLAINER.crate_name);
    }
}
