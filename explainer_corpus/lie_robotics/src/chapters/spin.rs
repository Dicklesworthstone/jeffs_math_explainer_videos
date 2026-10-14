//! Chapter 05, spin: a distance on the flat piece read as a speed held for a
//! second (05_1); arrows add, and adding the tilt's and the roll's gives both
//! turns at once, a third answer (05_2); holding one spin for one second is
//! the exponential (05_3, 05_4, 05_5); its formula is Rodrigues's (05_6); a
//! smooth group of motions is a Lie group and its flat piece the Lie algebra
//! (05_7).

use std::f64::consts::FRAC_PI_2;

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    Ask, BADGE_CORNER, BOOK_HALF, Badge, Choice, PartSpec, Pin, QuizLook, Shape, View, World, circle_points,
    edge_arrows, fact, fact_f64, fade_in_all, leave, line_style, mix, number_word, point_fn, pose_fn, say, spin_arrow,
    still, turns_in_order, tween_linear, value, your_turn,
};
use crate::model::{self, DEG, M3, Pose, V3, X, Y, add, dot, scale, sub, unit};
use crate::palette;

pub struct Spin {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const A_SECOND: &str = "a second, for one second";
const ONE_SECOND: &str = "1 s";
const THE_EXPONENTIAL: &str = "the exponential";
const THE_OTHER_ONE: &str = "the other one";
const ROLL_THEN_TILT: &str = "roll, then tilt";
const TILT_THEN_ROLL: &str = "tilt, then roll";
const BOTH_AT_ONCE: &str = "both at once";
const FOR_PROGRAMMERS: &str = "for programmers";
const LIE_GROUP: &str = "Lie group";
const LIE_ALGEBRA: &str = "Lie algebra";
const NAME: &str = "Sophus Lie";
const SAY_LEE: &str = "(say \"Lee\")";
const ARROW_NOTE: &str =
    "near doing nothing, arrows and orientations match one to one; far away they fold: a half turn has two arrows";
const EXP_NOTE_LABEL: &str = "the exponential:";
const MATRIX_NOTE: &str = "ω: a unit axis; t: the angle in radians; [ω]p means ω × p";
const ALONG: &str = "along";
const AROUND: &str = "around";
/// 05_1: the word "spin" stands on the arrow, with what it means under the
/// picture (R2 C-02: the first place the word is spoken).
const SPIN_WORD: &str = "spin";
const SPIN_CAPTION: &str = "spin: how fast it turns.\nThe arrow's length is the speed";
/// Where "spin" stands against the long arrow's tip.
const SPIN_WORD_SHIFT: [f64; 2] = [0.55, 0.2];
/// Where the caption stands (centred), under the book.
const SPIN_CAPTION_AT: [f64; 2] = [-3.7, -1.6];
/// 05_6: the cross product's arrow, named as the screen names it (R2 M-18),
/// and where its words stand under the formula.
const CROSS_WORDS: &str = "the arrow at right angles to both";
const CROSS_AT: [f64; 2] = [4.0, 2.5];
/// 05_7: what the card's two names mean (R2 C-10).
const LIE_GROUP_WORDS: &str = "a smooth curved space of motions";
const LIE_ALGEBRA_WORDS: &str = "its flat piece, plus a rule for two spins (the bracket, at the end)";
const BEGAN: &str = "began this in";
/// The name card's middle, and the three lines' heights (name, how to say
/// it, the years).
const CARD_X: f64 = 3.9;
const CARD_Y: [f64; 3] = [-1.9, -2.4, -2.9];
/// Where the two names stand, and how far under (over, for the algebra's)
/// its name each one's meaning stands.
const GROUP_AT: [f64; 2] = [0.0, -1.0];
const GROUP_MEANS_DROP: f64 = 0.65;
const ALGEBRA_AT: [f64; 2] = [0.0, 2.95];
const ALGEBRA_MEANS_DROP: f64 = 0.55;
/// Rodrigues's formula for a point, and for the matrix (TeX).
const POINT_FORMULA: &str = r"p' = \text{along} + \cos t\,\text{around} + \sin t\,(\omega \times \text{around})";
const MATRIX_FORMULA: &str = r"e^{[\omega]t} = I + \sin t\,[\omega] + (1 - \cos t)\,[\omega]^2";
const EXP_FORMULA: &str = r"e^{[\omega]t}";

/// The beats of the chapter, in order (storyboard ids).
const BEATS: [&str; 7] = ["05_1", "05_2", "05_3", "05_4", "05_5", "05_6", "05_7"];

/// 05_4's quiz: where do the edges point after a third of a turn about the
/// diagonal? The computed answer says each lands where the next one was.
pub const ASK: Ask = Ask {
    line: BEATS[3],
    question: "after one second,\nwhere do the edges point?",
    choices: &[
        Choice {
            key: "start",
            label: "back where they began",
        },
        Choice {
            key: "next",
            label: "each where the next one was",
        },
        Choice {
            key: "reversed",
            label: "each reversed",
        },
    ],
};

/// How long the tilt's arrow is for a quarter turn a second: an arrow is as
/// long as its speed, at this many scene units a radian a second.
const ARROW_PER_RAD: f64 = 1.7 / FRAC_PI_2;
/// The book here is drawn this fraction of full size.
const BOOK_FIT: f64 = 0.85;
/// Where 05_4's quiz card stands: its top ends below the badge in the corner
/// (at 0.3 it touched it) and its longest choice shrinks to fit.
const CARD_AT: [f64; 2] = [4.1, -0.05];
const CARD_SCALE: f64 = 0.92;
/// A point on the book that Rodrigues's formula follows: the tip of its x
/// edge arrow.
const EDGE: f64 = 1.25;

/// 05_3's exact version, its numbers from the facts and spelled as the script
/// spells them (R2 D-11): a unit axis and an angle, a spin's three numbers
/// in, the turn's 3 by 3 grid out, and the logarithm goes back.
fn exp_note_words() -> fmn::Result<String> {
    let (numbers, cells) = (
        fact_f64("G-R.numbers.arrow")? as u32,
        fact_f64("G-S.numbers.turn_matrix")? as u32,
    );
    let side = f64::from(cells).sqrt().round() as u32;
    if side * side != cells {
        return Err(super::draw::fail(format!("{cells} numbers are not a square grid")));
    }
    Ok(format!(
        "a unit axis ω, turned through t; {} numbers in, a {side} by {side} grid out, and the logarithm goes back",
        number_word(numbers)?
    ))
}

/// 05_7's name card: the years of Lie's life and the winter his theory began,
/// all from the facts (`1842–1899 · began this in 1873–74`).
fn lie_years() -> fmn::Result<String> {
    let (born, died, begun) = (
        fact_f64("G-L.lie.born")? as u32,
        fact_f64("G-L.lie.died")? as u32,
        fact_f64("G-L.lie.year")? as u32,
    );
    Ok(format!("{born}–{died} · {BEGAN} {begun}–{:02}", (begun + 1) % 100))
}

/// The diagonal between the three edges, and the one between tilt's and
/// roll's lines.
fn diagonal() -> V3 {
    unit([1.0, 1.0, 1.0])
}

fn tilt_plus_roll() -> V3 {
    unit([1.0, 1.0, 0.0])
}

/// Rodrigues: a point turned through `t` about the unit axis `w`: its part
/// along the axis stays, its part around swings. (The formula as the screen
/// writes it, checked against the model's own turn in the tests.)
#[cfg(test)]
fn rodrigues_point(p: V3, w: V3, t: f64) -> V3 {
    let along = scale(w, dot(w, p));
    let around = sub(p, along);
    add(
        along,
        add(scale(around, t.cos()), scale(model::cross(w, around), t.sin())),
    )
}

/// Rodrigues's matrix: `I + sin t [w] + (1 − cos t) [w]^2`.
#[cfg(test)]
fn rodrigues_matrix(w: V3, t: f64) -> M3 {
    let hat = model::hat(w);
    let hat2 = model::mul(&hat, &hat);
    std::array::from_fn(|i| {
        std::array::from_fn(|j| model::I3[i][j] + t.sin() * hat[i][j] + (1.0 - t.cos()) * hat2[i][j])
    })
}

/// A point on the circle of radius `r` about the centre `c`, at `heading`
/// degrees counter-clockwise from straight up.
fn on_circle(c: Vec3, r: f64, heading: f64) -> Vec3 {
    let t = heading * DEG;
    v(c[0] - r * t.sin(), c[1] + r * t.cos())
}

/// The part order of the book world (part 0 is the book itself).
mod part {
    pub const EDGES: usize = 1;
    pub const TILT: usize = 2;
    pub const SHORT: usize = 3;
    pub const ROLL: usize = 4;
    pub const ROLL_MOVED: usize = 5;
    pub const SUM: usize = 6;
    pub const DIAG: usize = 7;
    pub const OTHER: usize = 8;
    pub const GHOST_EDGES: usize = 9;
    pub const ALONG: usize = 10;
    pub const AROUND: usize = 11;
    pub const RING: usize = 12;
    pub const POINT: usize = 13;
    pub const CROSS: usize = 14;
}

impl SceneConstruct for Spin {
    fn name(&self) -> &str {
        "05_spin"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (blue, orange, teal) = (palette::turn(), palette::flat(), palette::place());
        let gap = fact_f64("G-H.gap")?;
        let both_deg = fact_f64("G-B.both_at_once.angle")?;
        let diag_rate = fact_f64("G-R.diag.rate_deg")?;

        // ---- the badge, in its corner, from here to the end of the chapter
        // (its arrow stays out of the scene until the move is named)
        let badge = Badge::build(stage, BADGE_CORNER, 1.0)?;

        // ---- the book world: one book at the origin, turned about the x line,
        // the line between the x and y lines, or the diagonal, by three
        // trackers (radians)
        let (ax, a01, ad) = (
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
            stage.add_value_tracker(0.0),
        );
        let (u01, ud) = (tilt_plus_roll(), diagonal());
        let spun = pose_fn(move |s| Pose {
            r: turns_in_order(&[(X, value(s, ax)), (u01, value(s, a01)), (ud, value(s, ad))]),
            p: [0.0; 3],
        });
        let half: V3 = BOOK_HALF.map(|x| BOOK_FIT * x);
        let quarter_len = ARROW_PER_RAD * FRAC_PI_2;
        // chapter 02's axis, which this video's diagonal is "the other one" to
        let (_, axis_other) = model::angle_axis(&model::tilt_then_roll(FRAC_PI_2));
        let p_edge = scale(X, EDGE);
        let along_v = scale(ud, dot(ud, p_edge));
        let around_len = model::norm(sub(p_edge, along_v));
        let ghosts: Vec<Shape> = edge_arrows(EDGE)
            .into_iter()
            .map(|s| match s {
                Shape::Arrow {
                    from, to, width, head, ..
                } => Shape::Arrow {
                    from,
                    to,
                    color: palette::steel(),
                    width,
                    head,
                },
                other => other,
            })
            .collect();
        let ud_turn = pose_fn(move |s| Pose {
            r: model::turn(ud, value(s, ad)),
            p: [0.0; 3],
        });
        let specs = vec![
            PartSpec::new(super::draw::book(half), spun.clone()),
            PartSpec::new(edge_arrows(EDGE), spun),
            PartSpec::new(spin_arrow([0.0; 3], X, quarter_len, 0.4), still(Pose::NOTHING)).layer(2),
            PartSpec::new(spin_arrow([0.0; 3], X, 0.45 * quarter_len, 0.28), still(Pose::NOTHING)).layer(2),
            PartSpec::new(spin_arrow([0.0; 3], Y, quarter_len, 0.4), still(Pose::NOTHING)).layer(2),
            PartSpec::new(
                vec![Shape::Arrow {
                    from: scale(X, quarter_len),
                    to: add(scale(X, quarter_len), scale(Y, quarter_len)),
                    color: orange,
                    width: 5.0,
                    head: 0.26,
                }],
                still(Pose::NOTHING),
            )
            .layer(2),
            PartSpec::new(
                spin_arrow([0.0; 3], u01, quarter_len * 2f64.sqrt(), 0.45),
                still(Pose::NOTHING),
            )
            .layer(2),
            PartSpec::new(
                spin_arrow([0.0; 3], ud, ARROW_PER_RAD * diag_rate * DEG, 0.45),
                still(Pose::NOTHING),
            )
            .layer(2),
            PartSpec::new(
                vec![Shape::Dashed {
                    a: scale(axis_other, -2.1),
                    b: scale(axis_other, 2.1),
                    color: mix(palette::screw(), super::draw::ground(), 0.4),
                    width: 4.0,
                    dashes: 12,
                }],
                still(Pose::NOTHING),
            )
            .layer(1),
            PartSpec::new(ghosts, still(Pose::NOTHING)),
            PartSpec::new(
                vec![Shape::Arrow {
                    from: [0.0; 3],
                    to: along_v,
                    color: teal,
                    width: 5.0,
                    head: 0.2,
                }],
                still(Pose::NOTHING),
            )
            .layer(2),
            PartSpec::new(
                vec![Shape::Arrow {
                    from: along_v,
                    to: p_edge,
                    color: blue,
                    width: 5.0,
                    head: 0.2,
                }],
                ud_turn.clone(),
            )
            .layer(2),
            PartSpec::new(
                vec![Shape::Path {
                    pts: circle_points(along_v, ud, around_len, 0.0, std::f64::consts::TAU, 48),
                    color: mix(blue, super::draw::ground(), 0.35),
                    width: 2.5,
                    closed: true,
                    bias: -0.01,
                }],
                still(Pose::NOTHING),
            ),
            PartSpec::new(
                vec![Shape::Dot {
                    at: p_edge,
                    color: Palette::ink(),
                    radius: 0.09,
                }],
                ud_turn.clone(),
            )
            .layer(3),
            // ω × around: from the same foot as `around`, at right angles to
            // it and to the axis, as long as `around` (R2 M-18)
            PartSpec::new(
                vec![Shape::Arrow {
                    from: along_v,
                    to: add(along_v, model::cross(ud, sub(p_edge, along_v))),
                    color: Palette::ink(),
                    width: 5.0,
                    head: 0.2,
                }],
                ud_turn,
            )
            .layer(2),
        ];
        let book_view = View::new(1.3, v(-3.7, 0.4));
        let other_tip = scale(axis_other, 2.1);
        let label = kit.text(stage, THE_OTHER_ONE, 26.0, Palette::dim(), v(0.0, 0.0))?;
        // the word "spin" stands against the long arrow's tip
        let spin_word = kit.text(stage, SPIN_WORD, 32.0, orange, v(0.0, 0.0))?;
        let spin_tip = scale(X, quarter_len);
        let pins = vec![
            Pin {
                mob: label,
                at: point_fn(move |_| other_tip),
                // right of the dashed line's end, which would run through the
                // label's first letters
                shift: v(0.95, 0.1),
            },
            Pin {
                mob: spin_word,
                at: point_fn(move |_| spin_tip),
                shift: v(SPIN_WORD_SHIFT[0], SPIN_WORD_SHIFT[1]),
            },
        ];
        let world = World::build(stage, book_view, 3.2, 0.2, specs, pins)?;
        for i in [
            part::EDGES,
            part::TILT,
            part::SHORT,
            part::ROLL,
            part::ROLL_MOVED,
            part::SUM,
            part::DIAG,
            part::OTHER,
            part::GHOST_EDGES,
            part::ALONG,
            part::AROUND,
            part::RING,
            part::POINT,
            part::CROSS,
        ] {
            stage.set_tracker_value(world.parts[i].shown, 0.0)?;
        }
        world.add_to_scene(stage)?;

        // 05_1: the circle's 20 degrees, read as a speed held for a second,
        // and then the same in space: one arrow along its axis
        let line = say(&kit, stage, BEATS[0])?;
        fade_in_all(stage, &[badge.body], 0.5)?;
        let centre = v(-3.7, 0.8);
        let circle = stage.arena_mut().add(
            Circle::new()
                .radius(1.3)
                .arc_center(centre)
                .style(line_style(Palette::faint(), 3.0))
                .build(),
        );
        let step: Vec<Vec3> = (0..=12)
            .map(|k| on_circle(centre, 1.3, gap * f64::from(k) / 12.0))
            .collect();
        let arc = stage
            .arena_mut()
            .add(polyline(step).style(line_style(orange, 7.0)).build());
        let words = kit.text(stage, &format!("{gap:.0}° {A_SECOND}"), 34.0, orange, v(-3.7, -1.2))?;
        line.wait_for(stage, "On the circle")?;
        fade_in_all(stage, &[circle, arc], 0.6)?;
        line.wait_for(stage, "Read it as a speed")?;
        fade_in_all(stage, &[words], 0.5)?;
        line.wait_for(stage, "In space")?;
        leave(stage, &[circle, arc, words], &[], 0.4)?;
        world.show(stage, 0.6)?;
        line.wait_for(stage, "Each is one arrow")?;
        play!(stage; tween(world.parts[part::SHORT].shown, 1.0, 0.5)?);
        // a short arrow turns the book slowly; a longer one, faster
        play!(stage; tween_linear(ax, 0.45 * FRAC_PI_2, 1.0)?);
        play!(stage; tween(ax, 0.0, 0.5)?);
        // the long arrow comes with its name and what the name means
        let spin_caption = kit.text(
            stage,
            SPIN_CAPTION,
            26.0,
            Palette::ink(),
            v(SPIN_CAPTION_AT[0], SPIN_CAPTION_AT[1]),
        )?;
        play!(stage;
            tween(world.parts[part::TILT].shown, 1.0, 0.4)?,
            fade_in(stage.arena_mut(), spin_word, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), spin_caption, ORIGIN, 1.0)?.rt(0.5),
        );
        play!(stage; tween_linear(ax, FRAC_PI_2, 1.0)?);
        play!(stage; tween(ax, 0.0, 0.6)?);

        // 05_2: two spins add tip to tail into one arrow: both turns at once,
        // a third answer, matching neither
        let line = say(&kit, stage, BEATS[1])?;
        line.wait_for(stage, "Add these two spins")?;
        leave(stage, &[spin_word, spin_caption], &[], 0.3)?;
        play!(stage;
            tween(world.parts[part::SHORT].shown, 0.0, 0.3)?,
            tween(world.parts[part::ROLL].shown, 1.0, 0.5)?,
        );
        play!(stage;
            tween(world.parts[part::ROLL].shown, 0.0, 0.2)?,
            tween(world.parts[part::ROLL_MOVED].shown, 1.0, 0.6)?,
        );
        line.wait_for(stage, "and you get both turns at once")?;
        play!(stage;
            tween(world.parts[part::SUM].shown, 1.0, 0.5)?,
            tween(world.parts[part::ROLL_MOVED].shown, 0.0, 0.5)?,
            tween(world.parts[part::TILT].shown, 0.0, 0.5)?,
        );
        play!(stage; tween_linear(a01, both_deg * DEG, 1.2)?);
        // beside the two books of chapter 01: matching neither
        let r_both = model::turn(u01, both_deg * DEG);
        let row: [(M3, f64, &str); 3] = [
            (model::tilt_then_roll(FRAC_PI_2), 1.2, TILT_THEN_ROLL),
            (model::roll_then_tilt(FRAC_PI_2), 3.9, ROLL_THEN_TILT),
            (r_both, 6.2, BOTH_AT_ONCE),
        ];
        let small: V3 = BOOK_HALF.map(|x| 0.5 * x);
        let row_view = View::new(1.0, v(0.0, 1.3));
        // a book at screen x sits at world x / cos(azimuth) on the row's view
        let across = View::basis().0[0];
        let row_specs: Vec<PartSpec> = row
            .iter()
            .map(|&(r, x, _)| {
                PartSpec::new(
                    super::draw::book(small),
                    still(Pose {
                        r,
                        p: [x / across, 0.0, 0.0],
                    }),
                )
            })
            .collect();
        let shelf = World::build(stage, row_view, 4.0, 0.1, row_specs, Vec::<Pin>::new())?;
        shelf.add_to_scene(stage)?;
        shelf.show(stage, 0.6)?;
        let mut captions = Vec::new();
        for &(_, x, text) in &row {
            captions.push(kit.text(stage, text, 24.0, Palette::dim(), v(x, 0.2))?);
        }
        // the exact version stands with the three books and stays into the
        // next line's first picture, so that it can be read
        fade_in_all(stage, &captions, 0.4)?;
        let note = lower_third(&kit, stage, "arrows:", ARROW_NOTE)?;
        play!(stage; fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.5));
        line.wait_for(stage, "a third answer")?;

        // 05_3: one spin held steady for one second: the move back to the
        // curved space is the exponential
        let line = say(&kit, stage, BEATS[2])?;
        leave(stage, &captions, &[shelf.shown], 0.5)?;
        play!(stage;
            tween(a01, 0.0, 0.6)?,
            tween(world.parts[part::SUM].shown, 0.0, 0.5)?,
            tween(world.parts[part::TILT].shown, 1.0, 0.5)?,
        );
        let clock = stage.add_value_tracker(0.0);
        let clock_at = v(1.6, 2.4);
        let ring = stage.always_redraw(move |s| {
            let f = value(s, clock).clamp(0.0, 1.0);
            let n = ((48.0 * f).ceil() as usize).max(1);
            let pts: Vec<Vec3> = (0..=n)
                .map(|k| on_circle(clock_at, 0.5, -360.0 * f * k as f64 / n as f64))
                .collect();
            s.add(polyline(pts).style(line_style(palette::quiz(), 6.0)).build())
        });
        let track = stage.arena_mut().add(
            Circle::new()
                .radius(0.5)
                .arc_center(clock_at)
                .style(line_style(Palette::faint(), 5.0))
                .build(),
        );
        let one = kit.text(stage, ONE_SECOND, 26.0, Palette::dim(), v(1.6, 1.55))?;
        line.wait_for(stage, "Hold one spin steady")?;
        stage.add_to_scene(ring)?;
        fade_in_all(stage, &[track, one], 0.3)?;
        play!(stage;
            tween_linear(ax, FRAC_PI_2, 1.0)?,
            tween_linear(clock, 1.0, 1.0)?,
        );
        // the previous line's exact version has had this line's first
        // sentence on screen, and leaves before the exponential's own
        leave(stage, &[note], &[], 0.4)?;
        line.wait_for(stage, "That move, from the flat piece")?;
        let named = kit.text(stage, THE_EXPONENTIAL, 30.0, Palette::ink(), v(4.3, 2.55))?;
        let formula = kit.tex(stage, EXP_FORMULA, 60.0, Palette::ink(), &[], v(3.6, 0.2))?;
        play!(stage;
            fade_in(stage.arena_mut(), badge.arrow, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), named, ORIGIN, 1.0)?.rt(0.6),
            write(stage.arena(), formula).rt(1.2),
        );
        let exp_note = lower_third(&kit, stage, EXP_NOTE_LABEL, &exp_note_words()?)?;
        play!(stage; fade_in(stage.arena_mut(), exp_note, ORIGIN, 1.0)?.rt(0.5));

        // 05_4: a third of a turn a second about the line between the three
        // edge arrows: where do the edges point after one second? (The
        // question's own line is said after this set-up, so first the
        // exponential's picture stays for the end of its line.)
        kit.hold(stage)?;
        leave(stage, &[ring, track, one, named, formula, exp_note], &[], 0.4)?;
        play!(stage;
            tween(ax, 0.0, 0.6)?,
            tween(world.parts[part::TILT].shown, 0.0, 0.4)?,
            tween(world.parts[part::EDGES].shown, 1.0, 0.5)?,
            tween(world.parts[part::DIAG].shown, 1.0, 0.5)?,
            tween(world.parts[part::OTHER].shown, 1.0, 0.5)?,
        );
        fade_in_all(stage, &[label], 0.4)?;
        let card = your_turn(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;

        // 05_5: each edge lands where the next one was: it is roll then tilt
        let line = say(&kit, stage, BEATS[4])?;
        leave(stage, &[label], &[world.parts[part::OTHER].shown], 0.4)?;
        play!(stage; tween(world.parts[part::GHOST_EDGES].shown, 1.0, 0.4)?);
        play!(stage; tween_linear(ad, diag_rate * DEG, 1.5)?);
        line.wait_for(stage, "That is where the book ended")?;
        leave(stage, &card.mobs(), &[], 0.4)?;
        let mut beside_specs = vec![PartSpec::new(
            super::draw::book(small),
            still(Pose {
                r: model::roll_then_tilt(FRAC_PI_2),
                p: [0.0; 3],
            }),
        )];
        beside_specs.push(PartSpec::new(
            edge_arrows(0.7),
            still(Pose {
                r: model::roll_then_tilt(FRAC_PI_2),
                p: [0.0; 3],
            }),
        ));
        let beside = World::build(
            stage,
            View::new(1.6, v(2.6, 0.4)),
            2.0,
            0.1,
            beside_specs,
            Vec::<Pin>::new(),
        )?;
        beside.add_to_scene(stage)?;
        beside.show(stage, 0.6)?;
        let equals = {
            let mut bar = |dy: f64| {
                stage.arena_mut().add(
                    polyline(vec![v(-0.3, 0.4 + dy), v(0.3, 0.4 + dy)])
                        .style(line_style(palette::good(), 7.0))
                        .build(),
                )
            };
            [bar(0.1), bar(-0.1)]
        };
        fade_in_all(stage, &equals, 0.4)?;

        // 05_6: Rodrigues: the part along the axis stays, the part around
        // swings
        let line = say(&kit, stage, BEATS[5])?;
        leave(
            stage,
            &equals,
            &[beside.shown, world.parts[part::GHOST_EDGES].shown],
            0.5,
        )?;
        play!(stage; tween(ad, 0.0, 0.6)?);
        play!(stage;
            tween(world.parts[part::DIAG].shown, 0.0, 0.3)?,
            tween(world.parts[part::ALONG].shown, 1.0, 0.5)?,
            tween(world.parts[part::AROUND].shown, 1.0, 0.5)?,
            tween(world.parts[part::RING].shown, 1.0, 0.5)?,
            tween(world.parts[part::POINT].shown, 1.0, 0.5)?,
        );
        let along_word = kit.text(stage, ALONG, 24.0, teal, v(-1.6, 1.5))?;
        let around_word = kit.text(stage, AROUND, 24.0, blue, v(-5.6, 1.9))?;
        fade_in_all(stage, &[along_word, around_word], 0.4)?;
        let point = kit.tex(stage, POINT_FORMULA, 40.0, Palette::ink(), &[], v(2.4, 3.1))?;
        // the name's credit comes in with the name, "named after Rodrigues",
        // and stays up for the rest of the line
        let year = fact("G-L.rodrigues.year")?;
        let named_for = lower_third(
            &kit,
            stage,
            "Rodrigues:",
            &format!("named for Olinde Rodrigues, who worked out in {year} how turns combine"),
        )?;
        line.wait_for(stage, "named after Rodrigues")?;
        play!(stage; fade_in(stage.arena_mut(), named_for, ORIGIN, 1.0)?.rt(0.5));
        line.wait_for(stage, "Any point splits")?;
        // the formula's last term is an arrow, drawn and named
        let cross_words = kit.text(stage, CROSS_WORDS, 22.0, Palette::ink(), v(CROSS_AT[0], CROSS_AT[1]))?;
        play!(stage;
            write(stage.arena(), point).rt(1.8),
            tween(world.parts[part::CROSS].shown, 1.0, 1.0)?,
            fade_in(stage.arena_mut(), cross_words, ORIGIN, 1.0)?.rt(1.0),
        );
        play!(stage; tween_linear(ad, diag_rate * DEG, 1.6)?);
        let matrix = kit.tex(stage, MATRIX_FORMULA, 34.0, Palette::ink(), &[], v(3.4, 1.2))?;
        let programmers = kit.text_left(stage, FOR_PROGRAMMERS, 24.0, Palette::dim(), v(0.1, 1.9))?;
        let what = kit.text_left(stage, MATRIX_NOTE, 24.0, Palette::dim(), v(0.1, 0.55))?;
        fade_in_all(stage, &[programmers, what], 0.4)?;
        play!(stage; write(stage.arena(), matrix).rt(1.4));

        // 05_7: a smooth group of motions is a Lie group; its flat piece is
        // its Lie algebra: the badge grows back to the centre
        let line = say(&kit, stage, BEATS[6])?;
        leave(
            stage,
            &[
                point,
                matrix,
                along_word,
                around_word,
                programmers,
                what,
                named_for,
                cross_words,
            ],
            &[world.shown],
            0.6,
        )?;
        let grow = badge.move_to(stage, 4.0, v(0.0, 0.6), 1.0)?;
        stage.play_prepared(grow)?;
        let group_word = kit.text(stage, LIE_GROUP, 44.0, blue, v(GROUP_AT[0], GROUP_AT[1]))?;
        let group_means = kit.text(
            stage,
            LIE_GROUP_WORDS,
            26.0,
            Palette::dim(),
            v(GROUP_AT[0], GROUP_AT[1] - GROUP_MEANS_DROP),
        )?;
        let algebra_word = kit.text(stage, LIE_ALGEBRA, 44.0, orange, v(ALGEBRA_AT[0], ALGEBRA_AT[1]))?;
        let algebra_means = kit.text(
            stage,
            LIE_ALGEBRA_WORDS,
            26.0,
            Palette::dim(),
            v(ALGEBRA_AT[0], ALGEBRA_AT[1] - ALGEBRA_MEANS_DROP),
        )?;
        let exp_word = kit.text(stage, THE_EXPONENTIAL, 34.0, Palette::ink(), v(3.4, 1.0))?;
        line.wait_for(stage, "A group of motions")?;
        fade_in_all(stage, &[group_word, group_means], 0.5)?;
        line.wait_for(stage, "Its flat piece")?;
        fade_in_all(stage, &[algebra_word, algebra_means, exp_word], 0.5)?;
        let name = kit.text(stage, NAME, 36.0, Palette::ink(), v(CARD_X, CARD_Y[0]))?;
        let lee = kit.text(stage, SAY_LEE, 24.0, Palette::dim(), v(CARD_X, CARD_Y[1]))?;
        let years = kit.text(stage, &lie_years()?, 24.0, Palette::dim(), v(CARD_X, CARD_Y[2]))?;
        fade_in_all(stage, &[name, lee, years], 0.5)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::{QuizCard, SAFE, cut_texts, probe, run_chapter};
    use super::*;

    /// 05_4: every choice, the longest ("each where the next one was") too,
    /// ends inside the card before the tick; the card's top stands below the
    /// badge in the corner and its right edge inside the safe area.
    #[test]
    fn the_quiz_card_holds_its_longest_choice_and_clears_the_badge() {
        probe(|stage| {
            let kit = Kit::new()?;
            let card = QuizCard::build(&kit, stage, ASK, QuizLook::at(v(CARD_AT[0], CARD_AT[1]), CARD_SCALE))?;
            let right = stage.get_right(card.back)[0];
            assert!(right < SAFE[0]);
            for &row in &card.rows {
                assert!(stage.get_right(row)[0] < right - 0.7 * CARD_SCALE);
            }
            let badge_bottom = BADGE_CORNER[1] - 0.3;
            assert!(
                stage.get_top(card.back)[1] < badge_bottom - 0.15,
                "the card's top is at {:.2}, the badge's bottom at {badge_bottom:.2}",
                stage.get_top(card.back)[1]
            );
            Ok(())
        });
    }

    #[test]
    fn the_beats_are_the_storyboards_and_the_scripts() {
        let board = explainer_kit::storyboard::parse(crate::STORYBOARD).unwrap();
        let chapter = board.iter().find(|c| c.key == "05_spin").expect("chapter 05");
        let ids: Vec<&str> = chapter.beats.iter().map(|b| b.id.as_str()).collect();
        assert_eq!(ids, BEATS);
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for beat in &chapter.beats {
            let words = beat.spoken().expect("every beat of chapter 05 is spoken");
            assert_eq!(script.text(&beat.id), Some(words.as_str()), "{}", beat.id);
        }
    }

    /// 05_4: five seconds; the computed answer is "each where the next one
    /// was".
    #[test]
    fn the_quiz_asks_where_the_edges_land() {
        assert_eq!(ASK.seconds().unwrap(), 5);
        assert_eq!(fact("G-Q.05_4.answer").unwrap(), "next");
        assert_eq!(ASK.answer_index().unwrap(), 1);
    }

    /// 05_1 and 05_2: an arrow is as long as its speed; adding the tilt's
    /// and the roll's arrows gives one arrow between them, the root of two
    /// longer, and holding it a second turns the book through 127 degrees
    /// about the line between them: a third orientation, matching neither
    /// result of chapter 01.
    #[test]
    fn two_spins_added_give_both_turns_at_once() {
        let quarter = ARROW_PER_RAD * FRAC_PI_2;
        assert!((quarter - 1.7).abs() < 1e-12);
        let sum = add(scale(X, FRAC_PI_2), scale(Y, FRAC_PI_2));
        let (len, axis) = (model::norm(sum), unit(sum));
        assert!((len / DEG - fact_f64("G-B.both_at_once.angle").unwrap()).abs() < 1e-3);
        assert!((len * ARROW_PER_RAD - quarter * 2f64.sqrt()).abs() < 1e-12);
        assert!(model::norm(sub(axis, tilt_plus_roll())) < 1e-12);
        let both = model::turn(axis, len);
        for other in [model::tilt_then_roll(FRAC_PI_2), model::roll_then_tilt(FRAC_PI_2)] {
            assert!(model::gap(&both, &other) > 0.1, "a third answer");
        }
    }

    /// 05_4 and 05_5: a third of a turn a second about the diagonal sends
    /// each edge to the next one's place, and is exactly roll then tilt.
    #[test]
    fn a_third_of_a_turn_about_the_diagonal_is_roll_then_tilt() {
        let rate = fact_f64("G-R.diag.rate_deg").unwrap();
        assert_eq!(rate, 120.0);
        let r = model::turn(diagonal(), rate * DEG);
        assert!(model::gap(&r, &model::roll_then_tilt(FRAC_PI_2)) < 1e-12);
        assert_eq!(fact("G-R.diag.is_roll_then_tilt").unwrap(), "1");
        let lands: Vec<V3> = [X, Y, model::Z].iter().map(|e| model::apply(&r, *e)).collect();
        for (i, p) in lands.iter().enumerate() {
            let next = [Y, model::Z, X][i];
            assert!(model::norm(sub(*p, next)) < 1e-12, "edge {i}");
        }
        // the arrow along the diagonal is a third of a turn a second long
        assert!((ARROW_PER_RAD * rate * DEG - 4.0 / 3.0 * 1.7).abs() < 1e-9);
    }

    /// 05_6: Rodrigues's point formula and matrix formula are the turn the
    /// model computes by its own route; the facts' decomposition of the x
    /// edge about the diagonal is the picture's (along, around).
    #[test]
    fn rodrigues_is_the_turn() {
        let w = diagonal();
        for deg in [0.0, 37.0, 120.0, 200.0] {
            let t = deg * DEG;
            let m = rodrigues_matrix(w, t);
            assert!(model::gap(&m, &model::turn(w, t)) < 1e-12, "{deg}");
            let p = [0.3, -1.1, 0.7];
            let want = model::apply(&model::turn(w, t), p);
            assert!(model::norm(sub(rodrigues_point(p, w, t), want)) < 1e-12, "{deg}");
        }
        let along = scale(w, dot(w, X));
        let around = sub(X, along);
        let listed = |key: &str| -> V3 {
            let v: Vec<f64> = fact(key).unwrap().split(',').map(|x| x.parse().unwrap()).collect();
            [v[0], v[1], v[2]]
        };
        assert!(model::norm(sub(along, listed("G-R.rod.along"))) < 1e-4);
        assert!(model::norm(sub(around, listed("G-R.rod.around"))) < 1e-4);
        let r = rodrigues_point(X, w, fact_f64("G-R.diag.rate_deg").unwrap() * DEG);
        assert!(model::norm(sub(r, listed("G-R.rod.result"))) < 1e-4);
        // the part along the axis is carried to itself
        assert!(model::norm(sub(model::apply(&model::turn(w, 1.0), along), along)) < 1e-12);
        assert_eq!(fact("G-L.rodrigues.year").unwrap(), "1840");
    }

    /// 05_7: the name card's years are the facts': 1842 to 1899, and the
    /// winter of 1873–74 the theory began, as the storyboard's card says.
    #[test]
    fn lie_lived_eighteen_forty_two_to_eighteen_ninety_nine() {
        assert_eq!(fact("G-L.lie.born").unwrap(), "1842");
        assert_eq!(fact("G-L.lie.died").unwrap(), "1899");
        assert_eq!(fact("G-L.lie.year").unwrap(), "1873");
        let years = lie_years().unwrap();
        assert_eq!(years, "1842–1899 · began this in 1873–74");
        assert!(
            crate::STORYBOARD.contains(&years),
            "the storyboard's card is not {years:?}"
        );
        let (born, died, theory): (u32, u32, u32) = (1842, 1899, 1873);
        assert!(born < theory && theory < died);
    }

    /// 05_3: the exact version of the exponential is the storyboard's words
    /// with the facts' numbers (three numbers in, a 3 by 3 grid out), fits its
    /// three lines inside the safe area, and stands below the book.
    #[test]
    fn the_exponentials_note_is_the_storyboards_and_fits() {
        let note = exp_note_words().unwrap();
        assert_eq!(
            note,
            "a unit axis ω, turned through t; three numbers in, a 3 by 3 grid out, and the logarithm goes back"
        );
        assert!(crate::STORYBOARD.contains(&note));
        assert_eq!(fact("G-S.numbers.turn_matrix").unwrap(), "9");
        assert_eq!(fact("G-R.numbers.arrow").unwrap(), "3");
        probe(|stage| {
            let kit = Kit::new()?;
            let band = lower_third(&kit, stage, EXP_NOTE_LABEL, &note)?;
            assert!(cut_texts(&kit).is_empty(), "cut: {:?}", cut_texts(&kit));
            assert!(stage.get_left(band)[0] > -SAFE[0] && stage.get_right(band)[0] < SAFE[0]);
            // the book's lowest point: its centre less its projected reach
            let lowest = 0.4 - 1.3 * BOOK_HALF.iter().map(|x| x * x).sum::<f64>().sqrt();
            assert!(
                stage.get_top(band)[1] < lowest - 0.1,
                "the band reaches {:.2}",
                stage.get_top(band)[1]
            );
            Ok(())
        });
    }

    /// 05_1: "spin" stands by the long arrow's tip, clear of the book and
    /// inside the frame; its caption (the storyboard's words, two lines) is
    /// centred under the book, inside the safe area.
    #[test]
    fn the_word_spin_and_its_caption_stand_clear() {
        assert!(crate::STORYBOARD.contains("spin: how fast it turns. The arrow's length is the speed"));
        assert_eq!(
            SPIN_CAPTION.replace('\n', " "),
            "spin: how fast it turns. The arrow's length is the speed"
        );
        let view = View::new(1.3, v(-3.7, 0.4));
        let tip = view.project(scale(X, ARROW_PER_RAD * FRAC_PI_2));
        let half: V3 = BOOK_HALF.map(|x| BOOK_FIT * x);
        let mut corners = Vec::new();
        for sx in [-1.0, 1.0] {
            for sy in [-1.0, 1.0] {
                for sz in [-1.0, 1.0] {
                    corners.push([sx * half[0], sy * half[1], sz * half[2]]);
                }
            }
        }
        let (lo, hi) = view.box_of(corners);
        probe(|stage| {
            let kit = Kit::new()?;
            let word = kit.text(stage, SPIN_WORD, 32.0, palette::flat(), ORIGIN)?;
            let (w, h) = (stage.get_width(word), stage.get_height(word));
            let c = (tip[0] + SPIN_WORD_SHIFT[0], tip[1] + SPIN_WORD_SHIFT[1]);
            assert!(
                c.0 - 0.5 * w > hi[0] + 0.1,
                "the word starts at {:.2}, the book ends at {:.2}",
                c.0 - 0.5 * w,
                hi[0]
            );
            assert!(c.0 + 0.5 * w < SAFE[0] && c.1 + 0.5 * h < SAFE[1]);
            assert!(
                (c.0 - tip[0]).hypot(c.1 - tip[1]) < 1.0,
                "the word is on the arrow, not off it"
            );
            let caption = kit.text(
                stage,
                SPIN_CAPTION,
                26.0,
                Palette::ink(),
                v(SPIN_CAPTION_AT[0], SPIN_CAPTION_AT[1]),
            )?;
            assert!(stage.get_top(caption)[1] < lo[1] - 0.2, "the caption touches the book");
            assert!(stage.get_bottom(caption)[1] > -SAFE[1]);
            assert!(stage.get_left(caption)[0] > -SAFE[0] && stage.get_right(caption)[0] < SAFE[0]);
            Ok(())
        });
    }

    /// 05_6: the cross product's arrow is at right angles to the axis and to
    /// the part around, as long as it; its words stand under the formula and
    /// clear of the programmers' lines and the matrix formula.
    #[test]
    fn the_cross_arrow_and_its_words() {
        let w = diagonal();
        let p = scale(X, EDGE);
        let around = sub(p, scale(w, dot(w, p)));
        let cross = model::cross(w, around);
        assert!(dot(cross, w).abs() < 1e-12 && dot(cross, around).abs() < 1e-12);
        assert!((model::norm(cross) - model::norm(around)).abs() < 1e-12);
        assert!(crate::STORYBOARD.contains(CROSS_WORDS));
        probe(|stage| {
            let kit = Kit::new()?;
            let boxed = |stage: &Stage<'_>, m: Mob| {
                (
                    stage.get_left(m)[0],
                    stage.get_right(m)[0],
                    stage.get_bottom(m)[1],
                    stage.get_top(m)[1],
                )
            };
            let words = kit.text(stage, CROSS_WORDS, 22.0, Palette::ink(), v(CROSS_AT[0], CROSS_AT[1]))?;
            let formula = kit.tex(stage, POINT_FORMULA, 40.0, Palette::ink(), &[], v(2.4, 3.1))?;
            let matrix = kit.tex(stage, MATRIX_FORMULA, 34.0, Palette::ink(), &[], v(3.4, 1.2))?;
            let programmers = kit.text_left(stage, FOR_PROGRAMMERS, 24.0, Palette::dim(), v(0.1, 1.9))?;
            let me = boxed(stage, words);
            assert!(
                me.0 > -SAFE[0] && me.1 < SAFE[0],
                "the words run {:.2} to {:.2}",
                me.0,
                me.1
            );
            for (name, other) in [
                ("the formula", formula),
                ("the matrix", matrix),
                ("the header", programmers),
            ] {
                let o = boxed(stage, other);
                let apart = me.1 < o.0 || me.0 > o.1 || me.3 < o.2 - 0.04 || me.2 > o.3 + 0.04;
                assert!(apart, "the cross words {me:?} meet {name} {o:?}");
            }
            Ok(())
        });
    }

    /// 05_7: the card's words are the storyboard's, each name stands with
    /// its meaning, the algebra's meaning clear of the badge's line, and the
    /// name card inside the safe area.
    #[test]
    fn the_two_names_and_the_name_card_fit() {
        for (name, words) in [(LIE_GROUP, LIE_GROUP_WORDS), (LIE_ALGEBRA, LIE_ALGEBRA_WORDS)] {
            let said = format!("{name}: {words}");
            assert!(
                crate::STORYBOARD.contains(&said),
                "the storyboard does not say {said:?}"
            );
        }
        let badge_line = 0.6 + 0.3 * 4.0; // the badge at the centre, four times the corner's size
        probe(|stage| {
            let kit = Kit::new()?;
            let group_name = kit.text(stage, LIE_GROUP, 44.0, palette::turn(), v(GROUP_AT[0], GROUP_AT[1]))?;
            let group_means = kit.text(
                stage,
                LIE_GROUP_WORDS,
                26.0,
                Palette::dim(),
                v(GROUP_AT[0], GROUP_AT[1] - GROUP_MEANS_DROP),
            )?;
            let algebra_name = kit.text(
                stage,
                LIE_ALGEBRA,
                44.0,
                palette::flat(),
                v(ALGEBRA_AT[0], ALGEBRA_AT[1]),
            )?;
            let algebra_means = kit.text(
                stage,
                LIE_ALGEBRA_WORDS,
                26.0,
                Palette::dim(),
                v(ALGEBRA_AT[0], ALGEBRA_AT[1] - ALGEBRA_MEANS_DROP),
            )?;
            for m in [group_name, group_means, algebra_name, algebra_means] {
                assert!(stage.get_left(m)[0] > -SAFE[0] && stage.get_right(m)[0] < SAFE[0]);
                assert!(stage.get_top(m)[1] < SAFE[1] && stage.get_bottom(m)[1] > -SAFE[1]);
            }
            // under its name, and the meaning of the algebra between its name and the line
            assert!(stage.get_top(group_means)[1] < stage.get_bottom(group_name)[1]);
            assert!(stage.get_top(algebra_means)[1] < stage.get_bottom(algebra_name)[1]);
            assert!(stage.get_bottom(algebra_means)[1] > badge_line + 0.1);
            // the name card: three lines, inside the safe area
            for (words, size, y) in [
                (NAME.to_owned(), 36.0, CARD_Y[0]),
                (SAY_LEE.to_owned(), 24.0, CARD_Y[1]),
                (lie_years().unwrap(), 24.0, CARD_Y[2]),
            ] {
                let m = kit.text(stage, &words, size, Palette::ink(), v(CARD_X, y))?;
                assert!(
                    stage.get_right(m)[0] < SAFE[0],
                    "{words:?} ends at {:.2}",
                    stage.get_right(m)[0]
                );
                assert!(stage.get_bottom(m)[1] > -SAFE[1]);
            }
            Ok(())
        });
    }

    /// The whole chapter constructs, headless.
    #[test]
    fn the_chapter_constructs_end_to_end() {
        run_chapter(&mut Spin {
            kit: Kit::new().expect("the kit"),
        });
    }

    /// The clock and the circle read their headings the way chapter 04 does:
    /// heading 0 is straight up and headings grow to the left.
    #[test]
    fn headings_grow_to_the_left() {
        let c = v(1.0, 2.0);
        let up = on_circle(c, 1.0, 0.0);
        assert!((up[0] - 1.0).abs() < 1e-12 && (up[1] - 3.0).abs() < 1e-12);
        let left = on_circle(c, 1.0, 90.0);
        assert!((left[0] - 0.0).abs() < 1e-12 && (left[1] - 2.0).abs() < 1e-12);
    }
}
