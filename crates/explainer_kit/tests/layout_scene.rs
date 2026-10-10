//! The layout lint on a real stage: a cut-off label is found, two
//! colliding texts are found, a fully transparent text is ignored, and a
//! group is measured by its painted members.

use std::cell::RefCell;
use std::rc::Rc;

use explainer_kit::{Kit, Linter, Palette, ink_style, rect, v};
use fmn::prelude::*;

struct Busy {
    kit: Kit,
    findings: Rc<RefCell<String>>,
}

impl SceneConstruct for Busy {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        // a label hanging off the bottom of the 8-unit-tall frame
        let low = kit.text(stage, "height", 30.0, Palette::ink(), v(0.0, -3.98))?;
        // two labels on top of each other
        let a = kit.text(stage, "alpha", 40.0, Palette::ink(), v(-2.0, 1.0))?;
        let b = kit.text(stage, "alphabet", 40.0, Palette::ink(), v(-1.9, 1.05))?;
        // a fully transparent label right on top of "alpha": not visible
        let ghost = kit.text(stage, "ghost", 40.0, Palette::ink(), v(-2.0, 1.0))?;
        stage.set_opacity(ghost, 0.0, true);
        // a group whose painted member sits far right, partly off-frame
        let g = rect(v(6.9, 0.0), 1.0, 0.5, ink_style(Palette::blue()))?;
        let g = stage.arena_mut().add(v_group([g]));
        for m in [low, a, b, ghost, g] {
            stage.add_to_scene(m)?;
        }
        kit.hold(stage)?;
        *self.findings.borrow_mut() = kit.lint.as_ref().unwrap().report();
        stage.wait(0.1)?;
        Ok(())
    }
}

#[test]
fn the_lint_finds_what_a_viewer_would_see() {
    let dir = std::env::temp_dir().join(format!("explainer_kit_layout_{}", std::process::id()));
    let findings = Rc::new(RefCell::new(String::new()));
    let mut scene = Busy {
        kit: Kit::new().unwrap().with_lint(Rc::new(Linter::new())),
        findings: Rc::clone(&findings),
    };
    let mut opts = RenderOptions::with_format(&dir, RenderFormat::PngSequence).unwrap();
    opts.config.camera.resolution = (32, 18);
    opts.config.camera.fps = 10;
    fmn::render(&mut scene, opts).expect("render");
    let report = findings.borrow().clone();
    let rows: Vec<&str> = report.lines().collect();
    assert!(
        rows.iter().any(|r| r.contains("outside") && r.contains("\"height\"")),
        "{report}"
    );
    assert!(
        rows.iter()
            .any(|r| r.contains("overlap") && r.contains("\"alpha\"") && r.contains("\"alphabet\"")),
        "{report}"
    );
    assert!(!report.contains("ghost"), "a transparent text is not visible: {report}");
    assert!(
        rows.iter()
            .any(|r| r.contains("outside") && r.contains("an unlabeled shape")),
        "the group is measured by its painted rectangle: {report}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
