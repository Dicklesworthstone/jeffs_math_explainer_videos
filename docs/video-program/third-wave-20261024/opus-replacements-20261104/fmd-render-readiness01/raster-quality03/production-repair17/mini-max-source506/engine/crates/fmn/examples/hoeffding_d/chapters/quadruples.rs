//! Chapter 3 — why quadruples: two points fix a direction, three a bend,
//! four a turn; and how many quadruples a real dataset holds.

use std::rc::Rc;

use fmn::animation::grow_from_center;
use fmn::prelude::*;

use crate::kit::{Frame2, Kit, Palette, Timed, fade_all, stroke_style, v};
use crate::play;
use crate::stats;

pub struct Quadruples {
    pub kit: Kit,
}

struct Probe {
    points: &'static [(f64, f64)],
    curve: fn(f64) -> f64,
    span: [f64; 2],
    count: &'static str,
    sees: &'static str,
    who: &'static str,
    color: fn() -> Srgb,
}

const PROBES: [Probe; 3] = [
    Probe {
        points: &[(-0.9, -0.75), (0.9, 0.75)],
        curve: |x| 0.8333 * x,
        span: [-1.25, 1.25],
        count: "2 points",
        sees: "a direction",
        who: r"\text{Pearson } r",
        color: Palette::pearson,
    },
    Probe {
        points: &[(-1.0, -0.9), (0.0, 0.4), (1.0, 0.8)],
        curve: |x| -0.45 * x * x + 0.85 * x + 0.4,
        span: [-1.12, 1.25],
        count: "3 points",
        sees: "a bend",
        who: r"\text{Spearman, Kendall}",
        color: Palette::spearman,
    },
    Probe {
        points: &[(-1.0, -0.7), (-0.35, 0.704), (0.35, 0.704), (1.0, -0.7)],
        curve: |x| 0.9 - 1.6 * x * x,
        span: [-1.12, 1.12],
        count: "4 points",
        sees: "a turn",
        who: r"\text{Hoeffding's } D",
        color: Palette::hoeffding,
    },
];

impl SceneConstruct for Quadruples {
    fn name(&self) -> &str {
        "03_quadruples"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title = kit.text(
            stage,
            "How many points does it take to see a shape?",
            42.0,
            Palette::ink(),
            v(0.0, 3.35),
        )?;
        kit.say(stage, "q1")?;
        play!(stage; write(stage.arena(), title).rt(1.4));

        for (k, probe) in PROBES.iter().enumerate() {
            let cx = -4.6 + 4.6 * k as f64;
            let frame = Frame2 {
                center: v(cx, 0.35),
                width: 3.7,
                height: 3.0,
                x: (-1.3, 1.3),
                y: (-1.2, 1.2),
            };
            let color = (probe.color)();
            let border = stage.arena_mut().add(frame.border(Palette::faint())?);
            let count = kit.text(
                stage,
                probe.count,
                34.0,
                Palette::ink(),
                v(cx, frame.top() + 0.4),
            )?;
            kit.say(stage, ["q2", "q3", "q4"][k])?;
            play!(stage; show_creation(border).rt(0.6), fade_in(stage.arena_mut(), count, v(0.0, -0.2), 1.0)?.rt(0.6));

            let mut dots = Vec::new();
            for &(x, y) in probe.points {
                let d = Dot::new()
                    .point(frame.p(x, y))
                    .radius(0.09)
                    .color(Palette::ink())
                    .build();
                dots.push(stage.arena_mut().add(d));
            }
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            for &d in &dots {
                let a = grow_from_center(stage.arena_mut(), d, None)?;
                anims.push(stage.prepare(a)?);
            }
            let group = crate::kit::lagged(stage, anims, 0.3)?;
            play!(stage; group.rt(0.9));

            let curve = FunctionGraph::new(probe.curve)
                .x_range([probe.span[0], probe.span[1], 0.04])
                .style(stroke_style(color, 5.0))
                .build()?;
            let scale_x = frame.width / 2.6;
            let scale_y = frame.height / 2.4;
            let curve = curve_to_frame(curve, scale_x, scale_y, frame.center);
            let curve = stage.arena_mut().add(curve);
            let sees = kit.text(stage, probe.sees, 32.0, color, v(cx, frame.bottom() - 0.42))?;
            let who = kit.tex(
                stage,
                probe.who,
                30.0,
                color,
                &[],
                v(cx, frame.bottom() - 1.05),
            )?;
            play!(stage; show_creation(curve).rt(1.2), fade_in(stage.arena_mut(), sees, v(0.0, 0.2), 1.0)?.rt(1.0));
            play!(stage; fade_in(stage.arena_mut(), who, ORIGIN, 1.0)?.rt(0.6));
            stage.scene_mut().bring_to_front(&dots)?;
            stage.wait(0.6)?;
        }
        let rings = kit.text(
            stage,
            "Rings, crosses and waves turn — so detecting them needs groups of four.",
            30.0,
            Palette::dim(),
            v(0.0, -3.45),
        )?;
        kit.say(stage, "q5")?;
        play!(stage; fade_in(stage.arena_mut(), rings, v(0.0, 0.2), 1.0)?.rt(1.2));
        stage.wait(1.2)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;

        // The combinatorial wall.
        let binom = kit.tex(
            stage,
            r"\binom{N}{4} = \frac{N(N-1)(N-2)(N-3)}{24}",
            54.0,
            Palette::ink(),
            &[("N", Palette::spearman())],
            v(0.0, 1.9),
        )?;
        kit.say(stage, "q6")?;
        play!(stage; write(stage.arena(), binom).rt(1.6));
        let at = kit.text(
            stage,
            "quadruples among N = 5,000 points:",
            34.0,
            Palette::dim(),
            v(0.0, 0.45),
        )?;
        kit.say(stage, "q7")?;
        play!(stage; fade_in(stage.arena_mut(), at, v(0.0, 0.2), 1.0)?.rt(0.8));

        let total = stats::choose(5000, 4);
        let tracker = stage.add_value_tracker(0.0);
        let book = Rc::clone(&kit.book);
        let counter = stage.always_redraw(move |s| {
            let value = s.tracker_value(tracker).unwrap_or(0.0).round();
            let d = DecimalNumber::new(value)
                .num_decimal_places(0)
                .group_with_commas(true)
                .font_size(72.0)
                .color(Palette::hoeffding())
                .build(&book)
                .expect("bundled digits always lay out");
            let m = s.add(d.into_vmob());
            s.move_to(m, v(0.0, -0.55), ORIGIN);
            m
        });
        stage.add_to_scene(counter)?;
        let count_up = tracker
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(3.0),
                rate_func: Some(fmn::core::rate::rush_into),
                ..AnimateArgs::default()
            })?
            .set_value(total)?;
        stage.play(count_up)?;
        kit.chime(stage, 523.25)?;
        let approx = kit.tex(
            stage,
            r"\approx 2.6\times 10^{13}",
            44.0,
            Palette::dim(),
            &[],
            v(0.0, -1.55),
        )?;
        play!(stage; fade_in(stage.arena_mut(), approx, v(0.0, 0.2), 1.0)?.rt(0.8));
        stage.wait(1.2)?;
        let trick = kit.text(
            stage,
            "The trick: ranks fold every quadruple into one count per point.",
            34.0,
            Palette::ink(),
            v(0.0, -2.75),
        )?;
        kit.say(stage, "q8")?;
        play!(stage; write(stage.arena(), trick).rt(1.6));
        let q = kit.tex(
            stage,
            r"Q_1,\ Q_2,\ \dots,\ Q_N",
            44.0,
            Palette::q(),
            &[],
            v(0.0, -3.5),
        )?;
        play!(stage; fade_in(stage.arena_mut(), q, v(0.0, 0.2), 1.0)?.rt(1.0));
        stage.wait(1.2)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.pad(stage, 2)?;
        Ok(())
    }
}

/// `FunctionGraph` draws in raw (x, f(x)) coordinates; map it into a frame
/// with independent x/y scales.
fn curve_to_frame(curve: VMobject, sx: f64, sy: f64, center: Vec3) -> VMobject {
    curve.map_points(move |p| [center[0] + p[0] * sx, center[1] + p[1] * sy, 0.0])
}
