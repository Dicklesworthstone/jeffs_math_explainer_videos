//! Native chapter review only: silent estimated narration, never a release cut.
//! Usage: preview CHAPTER_KEY NEW_OUTPUT_DIRECTORY
use algorithm_shapes_explainer::{SCRIPT, chapters};
use explainer_kit::{
    Kit, Linter, Narrator, Script,
    encode::{self, Quality},
};
use fmn::prelude::*;
use std::{path::PathBuf, rc::Rc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: preview CHAPTER_KEY NEW_OUTPUT_DIRECTORY".into());
    }
    let registry = chapters::registry();
    let chapter = registry.iter().find(|c| c.key == args[0]).ok_or("unknown chapter")?;
    let out = PathBuf::from(&args[1]);
    if out.exists() {
        return Err("preview output already exists; preserve it and choose a new directory".into());
    }
    let prefix = chapter.key.split('_').next().ok_or("chapter prefix missing")?;
    let script = Script::parse(SCRIPT)?.select(|id| id.starts_with(&format!("{prefix}_")));
    let narrator = Narrator::estimate(&script);
    let lint = Rc::new(Linter::sampled());
    let mut kit = Kit::new()?
        .with_narrator(Rc::clone(&narrator))
        .with_lint(Rc::clone(&lint));
    kit.chapter = chapter.key.to_owned();
    let finished = Rc::clone(&kit.finished);
    let mut scene = (chapter.make)(kit);
    std::fs::create_dir_all(&out)?;
    let mut options = encode::delivery_options(
        out.join("preview.mp4"),
        RenderFormat::Mp4,
        Quality {
            quality: 20,
            preset: "fast",
            ..Quality::delivery()
        },
    )?;
    options.config.camera.resolution = (1280, 720);
    options.config.camera.fps = 30;
    options.config.camera.background_color = explainer_kit::BACKGROUND.into();
    options.config.render.threads = fmn::config::config::ThreadPolicy::Fixed(2);
    let report = fmn::render(scene.as_mut(), options)?;
    std::fs::write(out.join("layout.tsv"), lint.report())?;
    let cues: String = narrator
        .said()
        .iter()
        .map(|(id, at)| format!("{id}\t{at:.3}\t{:.3}\n", narrator.length(id).unwrap_or(0.0)))
        .collect();
    std::fs::write(out.join("estimated-cues.tsv"), cues)?;
    std::fs::write(out.join("timeline-debug.txt"), format!("{:#?}", finished.borrow()))?;
    std::fs::write(
        out.join("review-scope.txt"),
        format!(
            "Chapter: {}\nFrames: {}\n1280x720 at30fps. Silent estimated narration timing.\nNot delivery, listening, fine sync or full-film acceptance.\n",
            chapter.key, report.artifact.frame_count
        ),
    )?;
    println!("native preview complete: {} frames", report.artifact.frame_count);
    Ok(())
}
