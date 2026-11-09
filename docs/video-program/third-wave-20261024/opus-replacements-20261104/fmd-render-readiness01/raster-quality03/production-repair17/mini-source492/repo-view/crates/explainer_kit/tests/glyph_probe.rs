//! The probe executes real chapter builders; blank registries and malformed
//! supplemental TeX fail. No narration, score or production video is used.

use explainer_kit::{Chapter, Explainer, Facts, Kit, Palette, TextKind, choose, v};
use fmn::prelude::*;
use std::path::PathBuf;

struct Content {
    kit: Kit,
}

impl SceneConstruct for Content {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let n = choose(10, 4);
        let text = self
            .kit
            .text(stage, &format!("netfuls: {n}"), 30.0, Palette::ink(), v(0.0, 2.0))?;
        let math = self
            .kit
            .tex(stage, &format!(r"\frac{{{n}}}{{2}}"), 40.0, Palette::ink(), &[], ORIGIN)?;
        let code = self
            .kit
            .mono(stage, "let caught = 2;", 30.0, Palette::ink(), v(0.0, -2.0))?;
        for m in [text, math, code] {
            stage.add_to_scene(m)?;
        }
        let source = format!("computed {}", n / 2);
        explainer_kit::glyphs::record(TextKind::Text, &source);
        let direct = Text::new(&source).font_size(25.0).build(&self.kit.book)?;
        stage.add(direct)?;
        stage.wait(1.0)?;
        self.kit.finish(stage)
    }
}

struct Blank;
impl SceneConstruct for Blank {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.wait(1.0)?;
        Ok(())
    }
}

fn chapters() -> Vec<Chapter> {
    vec![Chapter {
        key: "01_content",
        make: |kit| Box::new(Content { kit }),
        intensity: 0.5,
    }]
}

fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    f.pin("netfuls", choose(10, 4), "210", "math.comb")?;
    Ok(f)
}

fn explainer() -> Explainer {
    Explainer {
        crate_name: "glyph_probe_test",
        script: "01_1\tA real picture.\n",
        chapters,
        facts,
        glyph_text: &[],
        glyph_tex: &[],
        glyph_mono: &[],
        identity: explainer_score::score::Identity::hoeffding,
        narration_gain_db: 0.0,
        line_gaps: explainer_kit::narration::LineGaps::Constant,
        line_owner: None,
        thumbnail: None,
    }
}

fn fresh_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("glyph_probe_{tag}_{}_{nanos}", std::process::id()))
}

fn run(e: &Explainer, out: &std::path::Path) -> i32 {
    explainer_kit::cli::run(
        e,
        [
            "glyphs".into(),
            out.to_string_lossy().into_owned(),
            "--res".into(),
            "960x540".into(),
        ],
    )
}

#[test]
fn empty_static_lists_probe_the_actual_computed_scene() {
    let out = fresh_dir("content");
    assert_eq!(run(&explainer(), &out), 0);
    let registry = std::fs::read_to_string(out.join("registry.tsv")).unwrap();
    assert!(registry.contains("netfuls: 210"), "{registry}");
    assert!(registry.contains("computed 105"), "{registry}");
    assert!(registry.contains("Tex\t01_content"), "{registry}");
    assert!(registry.contains("Mono\t01_content\t\"let caught = 2;\""), "{registry}");
    assert_eq!(std::fs::read_to_string(out.join("failures.txt")).unwrap(), "");
    assert!(
        std::fs::read_dir(out.join("pages")).unwrap().any(|p| p
            .unwrap()
            .path()
            .extension()
            .is_some_and(|e| e == "png"))
    );
}

#[test]
fn a_chapter_with_no_glyph_content_cannot_pass() {
    let mut e = explainer();
    e.chapters = || {
        vec![Chapter {
            key: "01_content",
            make: |_| Box::new(Blank),
            intensity: 0.5,
        }]
    };
    let out = fresh_dir("blank");
    assert_eq!(run(&e, &out), 1);
    assert!(!out.join("pages").exists());
}

#[test]
fn malformed_math_fails_with_a_readable_report() {
    let mut e = explainer();
    e.glyph_tex = &[r"\frac{"];
    let out = fresh_dir("bad_tex");
    assert_eq!(run(&e, &out), 1);
    let failures = std::fs::read_to_string(out.join("failures.txt")).unwrap();
    assert!(failures.contains("frac"), "{failures}");
}
