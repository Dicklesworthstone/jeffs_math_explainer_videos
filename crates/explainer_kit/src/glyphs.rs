//! Glyph sources captured from the actual scene builders, including computed
//! equations and updater text. Capture is opt-in and scoped to the calling
//! render thread; ordinary renders retain no extra strings.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::path::PathBuf;
use std::rc::Rc;

use crate::kit::TextKind;
use crate::{Kit, Narrator, Palette, Script, ink_style, v};
use fmn::prelude::*;

type Sources = Rc<RefCell<BTreeSet<(TextKind, String)>>>;

thread_local! {
    static ACTIVE: RefCell<Option<Sources>> = const { RefCell::new(None) };
}

/// A chapter's live glyph registry. Dropping it restores the preceding
/// capture even when scene construction fails.
pub struct Capture {
    sources: Sources,
    previous: Option<Sources>,
}

impl Default for Capture {
    fn default() -> Self {
        Self::new()
    }
}

impl Capture {
    pub fn new() -> Self {
        let sources = Rc::new(RefCell::new(BTreeSet::new()));
        let previous = ACTIVE.with(|active| active.replace(Some(Rc::clone(&sources))));
        Self { sources, previous }
    }

    /// Distinct source strings, preserving their typesetting family.
    pub fn sources(&self) -> Vec<(TextKind, String)> {
        self.sources.borrow().iter().cloned().collect()
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        ACTIVE.with(|active| active.replace(self.previous.take()));
    }
}

/// Record text built outside the kit (for example by an `always_redraw`
/// updater). Pass exactly the source sent to Text/Tex, not a duplicate.
pub fn record(kind: TextKind, source: &str) {
    if kind == TextKind::Live || source.trim().is_empty() {
        return;
    }
    ACTIVE.with(|active| {
        if let Some(sources) = active.borrow().as_ref() {
            sources.borrow_mut().insert((kind, source.to_owned()));
        }
    });
}

/// Build every chapter with scratch timing, capture its actual strings,
/// and probe them in their proper font. The tiny collection renders never
/// read narration or score stems and are not visual or timing acceptance.
pub(crate) fn probe(e: &crate::Explainer, args: &crate::cli::Args) -> Result<i32, Box<dyn Error>> {
    let out = PathBuf::from(args.targets.first().map(String::as_str).unwrap_or("glyphs"));
    crate::cli::set_aside(&out)?;
    std::fs::create_dir_all(&out)?;
    let script = Script::parse(e.script)?;
    let chapters = (e.chapters)();
    let keys: Vec<_> = chapters.iter().map(|c| c.key).collect();
    crate::cli::check_script_owners(&script, &keys, e.line_owner)?;
    let mut sources: BTreeMap<(TextKind, String), BTreeSet<String>> = BTreeMap::new();
    for chapter in chapters {
        let capture = Capture::new();
        let mut kit = Kit::new()?.with_narrator(Narrator::estimate(&script));
        kit.chapter = chapter.key.to_owned();
        let camera = kit.clone();
        let mut scene = (chapter.make)(kit);
        let path = out.join("capture").join(chapter.key);
        let opts = crate::cli::options(path, RenderFormat::PngSequence, (32, 18), 1)?;
        let report = match camera.camera_shot() {
            Some(shot) => crate::cli::render_camera_chapter(&camera, shot, scene.as_mut(), opts)?,
            None => render(scene.as_mut(), opts)?,
        };
        let rows = capture.sources();
        drop(capture);
        if rows.is_empty() {
            return Err(format!("glyph probe: chapter {} registered no strings", chapter.key).into());
        }
        println!(
            "glyph registry {}: {} sources -> {}",
            chapter.key,
            rows.len(),
            report.artifact.path.display()
        );
        for row in rows {
            sources.entry(row).or_default().insert(chapter.key.to_owned());
        }
    }
    for (kind, extra) in [
        (TextKind::Text, e.glyph_text),
        (TextKind::Tex, e.glyph_tex),
        (TextKind::Mono, e.glyph_mono),
    ] {
        for &source in extra {
            if !source.trim().is_empty() {
                sources
                    .entry((kind, source.to_owned()))
                    .or_default()
                    .insert("supplement".into());
            }
        }
    }
    if sources.is_empty() {
        return Err("glyph probe: no chapter strings registered".into());
    }
    let registry: String = sources
        .iter()
        .map(|((kind, source), chapters)| {
            format!(
                "{kind:?}\t{}\t{source:?}\n",
                chapters.iter().cloned().collect::<Vec<_>>().join(",")
            )
        })
        .collect();
    std::fs::write(out.join("registry.tsv"), registry)?;
    let count = sources.len();
    let mut sheet = Sheet {
        kit: Kit::new()?,
        title: format!("{} · {} actual glyph sources", e.crate_name, count),
        sources: sources.into_keys().collect(),
        failures: Vec::new(),
    };
    let pages = out.join("pages");
    let report = render(
        &mut sheet,
        crate::cli::options(pages.clone(), RenderFormat::PngSequence, args.res, 1)?,
    )?;
    std::fs::write(out.join("failures.txt"), sheet.failures.join("\n"))?;
    println!(
        "glyph probe: {count} sources, {} failures -> {}",
        sheet.failures.len(),
        report.artifact.path.display()
    );
    if sheet.failures.is_empty() {
        Ok(0)
    } else {
        for failure in &sheet.failures {
            eprintln!("glyph probe: {failure}");
        }
        Ok(1)
    }
}

/// Break long prose/code into readable rows without losing characters.
/// TeX stays intact: its complete expression is the unit being checked.
fn lines(source: &str) -> Vec<String> {
    source
        .split('\n')
        .flat_map(|line| {
            let chars: Vec<_> = line.chars().collect();
            if chars.is_empty() {
                vec![String::new()]
            } else {
                chars.chunks(64).map(|chunk| chunk.iter().collect()).collect()
            }
        })
        .collect()
}

// Native TeX recovers from incomplete groups. A probe must report them:
// successful recovery is not evidence that the supplied formula is valid.
fn check_groups(source: &str) -> Result<(), String> {
    let (mut depth, mut escaped) = (0usize, false);
    for ch in source.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '{' => depth += 1,
            '}' => {
                if depth == 0 {
                    return Err("unmatched closing TeX group".into());
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err("unclosed TeX group".into());
    }
    Ok(())
}

struct Sheet {
    kit: Kit,
    title: String,
    sources: Vec<(TextKind, String)>,
    failures: Vec<String>,
}

impl Sheet {
    fn heading(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let title = self
            .kit
            .text_left(stage, &self.title, 24.0, Palette::dim(), v(-6.7, 3.55))?;
        stage.add_to_scene(title)?;
        Ok(())
    }
}

impl SceneConstruct for Sheet {
    fn name(&self) -> &str {
        "glyphs"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let mut y = 3.0;
        self.heading(stage)?;
        for (kind, source) in &self.sources {
            let rows = if *kind == TextKind::Tex {
                vec![source.clone()]
            } else {
                lines(source)
            };
            for row in rows {
                if row.is_empty() {
                    y -= 0.3;
                    continue;
                }
                let built = if *kind == TextKind::Tex {
                    match check_groups(&row) {
                        Err(error) => Err(error),
                        Ok(()) => Tex::new(&row)
                            .font_size(36.0)
                            .style(ink_style(Palette::ink()))
                            .build(stage.tex_engine()?)
                            .map(|t| stage.arena_mut().add(t))
                            .map_err(|e| e.to_string()),
                    }
                } else {
                    let mut text = Text::new(&row).font_size(34.0).style(ink_style(Palette::ink()));
                    if *kind == TextKind::Mono {
                        text = text.font(crate::kit::MONO_FAMILY);
                    }
                    text.build(&self.kit.book)
                        .map(|t| stage.arena_mut().add(t))
                        .map_err(|e| e.to_string())
                };
                let m = match built {
                    Ok(m) => m,
                    Err(error) => {
                        self.failures.push(format!("{kind:?} {row:?}: {error}"));
                        self.kit.text(
                            stage,
                            "TYPESETTING FAILED — see failures.txt",
                            28.0,
                            Palette::red(),
                            ORIGIN,
                        )?
                    }
                };
                // The expression must fit, rather than silently falling
                // off the sheet's right edge.
                let width = stage.get_width(m);
                if width > 13.4 {
                    stage.scale(m, 13.4 / width);
                }
                let height = stage.get_height(m).max(0.25);
                if y - height < -3.35 {
                    stage.wait(1.0)?;
                    stage.scene_mut().clear();
                    self.heading(stage)?;
                    y = 3.0;
                }
                stage.move_to(m, v(-6.7, y), UL);
                stage.add_to_scene(m)?;
                y -= height + 0.22;
            }
        }
        stage.wait(1.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tex_groups_keep_escaped_delimiters_and_reject_incomplete_formulas() {
        assert!(check_groups(r"\frac{a}{b}").is_ok());
        assert!(check_groups(r"\left\{a\right\}").is_ok());
        assert!(check_groups(r"\frac{").is_err());
        assert!(check_groups(r"a}").is_err());
    }

    #[test]
    fn actual_sources_are_deduplicated_without_merging_fonts() {
        let capture = Capture::new();
        let computed = format!("packet {}", 7 + 2);
        record(TextKind::Text, &computed);
        record(TextKind::Text, &computed);
        record(TextKind::Mono, &computed);
        record(TextKind::Live, "live number");
        record(TextKind::Text, " ");
        assert_eq!(
            capture.sources(),
            vec![(TextKind::Text, computed.clone()), (TextKind::Mono, computed)]
        );
    }

    #[test]
    fn nested_capture_does_not_leak_into_the_previous_chapter() {
        let outer = Capture::new();
        record(TextKind::Text, "outer");
        {
            let inner = Capture::new();
            record(TextKind::Tex, "x^2");
            assert_eq!(inner.sources(), vec![(TextKind::Tex, "x^2".into())]);
        }
        record(TextKind::Text, "after");
        assert_eq!(outer.sources().len(), 2);
        assert!(outer.sources().iter().all(|(_, s)| s != "x^2"));
    }
}
