//! The kit in a real scene: text, math, live readouts, `play!`, `tween` and
//! `fade_all` through franken_manim's facade, rendered to a tiny PNG sequence
//! (no ffmpeg needed). Positions are read back from the stage.

use std::cell::RefCell;
use std::rc::Rc;

use explainer_kit::{Kit, Palette, TextKind, Timed, fade_all, ink_style, live_bar, play, rect, tween, v};
use fmn::prelude::*;

#[derive(Default)]
struct Seen {
    rect_dl: Option<Vec3>,
    text_center: Option<Vec3>,
    named_center: Option<Vec3>,
    tex_left: Option<Vec3>,
    readout_left_after: Option<Vec3>,
    tracker_after: Option<f64>,
    on_stage_after_fade: Option<usize>,
    /// Scene clock (seconds) after each step.
    clock: Vec<f64>,
}

struct Smoke {
    kit: Kit,
    seen: Rc<RefCell<Seen>>,
}

impl SceneConstruct for Smoke {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title = kit.text(stage, "Hoeffding's D", 36.0, Palette::ink(), v(0.0, 2.0))?;
        let named = kit.text_t2c(
            stage,
            "Blue and Green",
            24.0,
            Palette::ink(),
            &[("Blue", Palette::blue()), ("Green", Palette::green())],
            v(0.5, -3.0),
        )?;
        let eq = kit.tex_left(
            stage,
            r"D = 30\cdot\frac{Q}{n}",
            40.0,
            Palette::ink(),
            &[("Q", Palette::hero())],
            v(-3.0, 0.5),
        )?;
        let r = rect(v(1.0, -2.0), 3.0, 0.5, ink_style(Palette::blue()))?;
        let r = stage.arena_mut().add(r);
        let t = stage.add_value_tracker(0.0);
        let readout = kit.live_number(stage, t, 2, false, 32.0, Palette::hero(), v(-1.0, -1.0), LEFT)?;
        let _bar = live_bar(stage, t, v(-3.0, -2.5), 1.0, 0.2, Palette::green())?;
        {
            let mut seen = self.seen.borrow_mut();
            seen.text_center = Some(stage.get_center(title));
            seen.named_center = Some(stage.get_center(named));
            seen.tex_left = Some(stage.get_left(eq));
            seen.rect_dl = Some(stage.get_corner(r, DL));
        }
        play!(stage; write(stage.arena(), title).rt(0.3), write(stage.arena(), eq).rt(0.3), fade_in(stage.arena_mut(), r, ORIGIN, 1.0)?.rt(0.3));
        self.seen.borrow_mut().clock.push(stage.scene().time().to_f64());
        stage.play(tween(t, 2.5, 0.5)?)?;
        self.seen.borrow_mut().clock.push(stage.scene().time().to_f64());
        {
            let mut seen = self.seen.borrow_mut();
            seen.tracker_after = stage.tracker_value(t);
            seen.readout_left_after = Some(stage.get_left(readout));
        }
        fade_all(stage, 0.2)?;
        let mut seen = self.seen.borrow_mut();
        seen.clock.push(stage.scene().time().to_f64());
        seen.on_stage_after_fade = Some(stage.scene().mobjects().len());
        Ok(())
    }
}

#[test]
fn the_kit_builds_and_plays_a_scene() {
    let dir = std::env::temp_dir().join(format!("explainer_kit_smoke_{}", std::process::id()));
    let seen = Rc::new(RefCell::new(Seen::default()));
    let kit = Kit::new().expect("bundled fonts");
    let mut scene = Smoke {
        kit: kit.clone(),
        seen: Rc::clone(&seen),
    };
    let mut opts = RenderOptions::with_format(&dir, RenderFormat::PngSequence).unwrap();
    opts.config.camera.resolution = (64, 36);
    opts.config.camera.fps = 10;
    let report = fmn::render(&mut scene, opts).expect("render");
    let s = seen.borrow();
    // every step took the time asked of it, to within one frame (the clock
    // rounds each segment up to whole frames), and every frame of the scene
    // clock was published
    let steps: Vec<f64> = s.clock.windows(2).map(|w| w[1] - w[0]).collect();
    assert!((s.clock[0] - 0.3).abs() <= 0.1 + 1e-9, "{:?}", s.clock);
    assert!(
        (steps[0] - 0.5).abs() <= 0.1 + 1e-9 && (steps[1] - 0.2).abs() <= 0.1 + 1e-9,
        "{steps:?}"
    );
    assert_eq!(
        report.artifact.frame_count as f64,
        (s.clock[2] * 10.0).round(),
        "{:?}",
        s.clock
    );
    let close = |a: Vec3, b: Vec3| (a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6;
    assert!(close(s.text_center.unwrap(), v(0.0, 2.0)), "{:?}", s.text_center);
    assert!(close(s.named_center.unwrap(), v(0.5, -3.0)), "{:?}", s.named_center);
    assert!((s.tex_left.unwrap()[0] + 3.0).abs() < 1e-6, "{:?}", s.tex_left);
    assert!(close(s.rect_dl.unwrap(), v(1.0, -2.0)), "{:?}", s.rect_dl);
    assert_eq!(s.tracker_after, Some(2.5));
    // the readout's left edge stays pinned while its digits change
    assert!(
        (s.readout_left_after.unwrap()[0] + 1.0).abs() < 1e-6,
        "{:?}",
        s.readout_left_after
    );
    assert_eq!(s.on_stage_after_fade, Some(0), "fade_all leaves nothing on stage");

    let reg = kit.registered();
    let kinds: Vec<(TextKind, &str)> = reg.iter().map(|r| (r.kind, r.source.as_str())).collect();
    assert_eq!(
        kinds,
        vec![
            (TextKind::Text, "Hoeffding's D"),
            (TextKind::Text, "Blue and Green"),
            (TextKind::Tex, r"D = 30\cdot\frac{Q}{n}"),
            (TextKind::Live, "live number"),
        ]
    );
}
