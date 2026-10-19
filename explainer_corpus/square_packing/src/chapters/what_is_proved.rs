//! Chapter 08, what is proved and what is found: the coda. Ten tiles fit a
//! box 3 + 1/√2 wide, nine straight and one turned as in the middle of five
//! (08_1, Stromquist's packing). Eleven fit a box 1939/500 = 3.878 wide, five
//! turned about 40° (08_2): does the picture prove that box the smallest?
//! (a four-second think). No: checked exactly, each tile a true unit square,
//! none overlapping, all inside, which shows the box is big enough and
//! nothing more (08_3). A proof about every arrangement is something else,
//! like the one built for five (08_4); for five there are both, and the
//! smallest box is exactly 2 + 1/√2 (08_5).
//!
//! Each packing is the model's (`model::ten`, `model::eleven`,
//! `model::five_turned`), drawn as it is: the pictures change by a cut, never
//! by a morph between packings. Nothing claims a smallest box for eleven,
//! and the ten's minimality (Stromquist's theorem) is not proved here.

use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    Board, arc_points, corner, dim, dot, fact, fail, faint, flat, ink, line_style, path, polygon, retrace, ring, say,
    stand, swap, touch, turned,
};
use super::four_points_five_tiles::B_IS;
use super::the_corner_lemma::{LAYER_MARKS, POINT_R};
use super::the_turned_tile::{BOARD, RING_AT, RING_R, THINK_S};
use crate::model::{self, Packing, Surd, Tile};

pub struct WhatIsProved {
    pub kit: Kit,
}

/// The narration lines this chapter speaks, in order.
pub const LINES: [&str; 5] = ["08_1", "08_2", "08_3", "08_4", "08_5"];

// the chapter's fixed words
pub const TEN_SIDE: &str = r"3 + \frac{1}{\sqrt{2}}";
pub const TEN_IS: &str = "ten tiles: nine straight, one turned";
pub const ELEVEN_SIDE: &str = r"T = \frac{1939}{500} = 3.878";
pub const ELEVEN_TURN: &str = r"\cos = \tfrac{105}{137},\ \sin = \tfrac{88}{137}";
pub const ELEVEN_IS: &str = "eleven tiles: six straight, five turned";
pub const UNIT: &str = "every tile a unit square";
pub const APART: &str = "no two overlap";
pub const INSIDE: &str = "all inside the box";
pub const ENOUGH: &str = "this box is big enough, and nothing more";
pub const A_FIT: &str = "a fit: one arrangement that works";
pub const A_PROOF: &str = "a proof: no arrangement does better";
pub const FOR_FIVE: &str = "for five: chapters 04 to 07";
pub const FITS: &str = "fits: chapter 01";
pub const NONE_NARROWER: &str = "nothing narrower: chapters 04 to 07";

// ---------------------------------------------------------------- layout

/// The larger boxes on screen: a tile is 1.55 scene units wide.
pub const WIDE: Board = Board {
    at: [-2.6, -0.3, 0.0],
    k: 1.55,
};
/// The left edge of the panel's text.
pub const TEXT_X: f64 = 1.1;
/// The panel's rows.
pub const ROWS: [f64; 7] = [3.0, 2.4, 1.8, 1.2, 0.6, 0.0, -0.6];

/// What the chapter reads from the facts, checked against the model.
struct Reads {
    ten_about: String,
    eleven_degrees: String,
    stromquist: String,
}

fn reads() -> fmn::Result<Reads> {
    for (key, want) in [
        ("beyond.ten.check", "fits"),
        ("beyond.ten.count", "10"),
        ("beyond.ten.turned", "1"),
        ("beyond.eleven.check", "fits"),
        ("beyond.eleven.count", "11"),
        ("beyond.eleven.turned", "5"),
        ("beyond.eleven.side", "1939/500"),
        ("beyond.eleven.side.3dp", "3.878"),
        ("beyond.eleven.turn", "105/137,88/137"),
        ("turned.check", "fits"),
    ] {
        if fact(key)? != want {
            return Err(fail(format!(
                "{key} is not {want:?}: the chapter would say what the facts do not"
            )));
        }
    }
    let ten = model::ten();
    if fact("beyond.ten.side")? != ten.side.to_string() {
        return Err(fail("beyond.ten.side is not the model's".into()));
    }
    let eleven = model::eleven();
    let t = eleven.tiles[eleven.tiles.len() - 1];
    let degrees = t.sin.to_f64().atan2(t.cos.to_f64()).to_degrees();
    let shown = fact("beyond.eleven.degrees.0dp")?;
    if shown != format!("{degrees:.0}") {
        return Err(fail("beyond.eleven.degrees.0dp is not the model's".into()));
    }
    Ok(Reads {
        ten_about: format!("about {}", fact("beyond.ten.side.2dp")?),
        eleven_degrees: format!("turned about {shown}°"),
        stromquist: format!("packing as shown by W. Stromquist, {}", fact("source.stromquist")?),
    })
}

/// An exact tile as drawn.
fn as_f64(t: &Tile<Surd>) -> Tile<f64> {
    Tile {
        centre: (t.centre.0.to_f64(), t.centre.1.to_f64()),
        cos: t.cos.to_f64(),
        sin: t.sin.to_f64(),
    }
}

/// A packing as drawn on `board`: its outline, then its tiles, straight ones
/// blue and turned ones gold. Detached, in drawing order.
fn draw_packing(stage: &mut Stage<'_>, board: Board, p: &Packing<Surd>) -> Vec<Mob> {
    let side = p.side.to_f64();
    let mut out: Vec<Mob> = p
        .tiles
        .iter()
        .map(|t| {
            let c = if t.sin == Surd::ZERO { corner() } else { turned() };
            polygon(stage, board.tile(side, &as_f64(t)), flat(c, 0.85))
        })
        .collect();
    out.push(path(stage, board.outline(side), line_style(ink(), 4.0)));
    out
}

impl SceneConstruct for WhatIsProved {
    fn name(&self) -> &str {
        "08_what_is_proved"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let r = reads()?;
        let caption = |stage: &mut Stage<'_>, s: &str, c: Srgb, row: usize| {
            kit.text_left(stage, s, 24.0, c, v(TEXT_X, ROWS[row]))
        };
        let tex = |stage: &mut Stage<'_>, s: &str, row: usize| {
            kit.tex(stage, s, 30.0, ink(), &[], v(TEXT_X + 1.6, ROWS[row]))
        };

        // ---------------------------------------------------------- 08_1
        // the ten, as Stromquist drew them: its turned tile touching four at
        // the middles of its sides, as where five fit
        let line_1 = say(&kit, stage, LINES[0])?;
        let ten = model::ten();
        let ten_view = draw_packing(stage, WIDE, &ten);
        let ten_side = tex(stage, TEN_SIDE, 0)?;
        let ten_about = caption(stage, &r.ten_about, dim(), 1)?;
        let mut first = ten_view.clone();
        first.extend([ten_side, ten_about]);
        swap(stage, &[], &first, 0.7)?;
        line_1.wait_for(stage, "nine straight")?;
        let side = ten.side.to_f64();
        let turned_tile = ten.tiles[ten.tiles.len() - 1];
        let touches: Vec<Mob> = turned_tile
            .midpoints()
            .iter()
            .filter(|&&m| ten.tiles.iter().any(|t| t.corners().contains(&m)))
            .map(|m| dot(stage, WIDE.p(side, (m.0.to_f64(), m.1.to_f64())), POINT_R, touch()))
            .collect();
        for &m in &touches {
            stage.set_z_index(m, LAYER_MARKS, true);
        }
        let ten_is = caption(stage, TEN_IS, ink(), 2)?;
        let source = caption(stage, &r.stromquist, dim(), 6)?;
        let mut more = touches.clone();
        more.extend([ten_is, source]);
        swap(stage, &[], &more, 0.5)?;

        // ---------------------------------------------------------- 08_2
        // a cut to the eleven: its box, its turn; the question; four seconds
        // to think
        let line_2 = say(&kit, stage, LINES[1])?;
        let mut leaving = ten_view;
        leaving.extend(touches);
        leaving.extend([ten_side, ten_about, ten_is, source]);
        swap(stage, &leaving, &[], 0.5)?;
        let eleven = model::eleven();
        let eleven_view = draw_packing(stage, WIDE, &eleven);
        let eleven_side = tex(stage, ELEVEN_SIDE, 0)?;
        let eleven_is = caption(stage, ELEVEN_IS, ink(), 2)?;
        let mut second = eleven_view.clone();
        second.extend([eleven_side, eleven_is]);
        swap(stage, &[], &second, 0.6)?;
        line_2.wait_for(stage, "five of them turned")?;
        let eleven_turn = tex(stage, ELEVEN_TURN, 1)?;
        let degrees = caption(stage, &r.eleven_degrees, turned(), 3)?;
        swap(stage, &[], &[eleven_turn, degrees], 0.4)?;
        line_2.wait_for(stage, "Does this picture")?;
        let track = ring(stage, RING_AT, RING_R, faint(), 6.0);
        let arc = path(stage, arc_points(RING_AT, RING_R, 1.0), line_style(ink(), 7.0));
        swap(stage, &[], &[track, arc], 0.4)?;
        kit.hold(stage)?;
        let mut elapsed = 0.0;
        stage.add_dt_updater(
            arc,
            move |s: &mut fmn::mobject::Stage, m, dt| {
                elapsed += dt;
                retrace(s, m, arc_points(RING_AT, RING_R, 1.0 - elapsed / THINK_S));
            },
            false,
        )?;
        kit.listen(stage, THINK_S, LINES[1])?;

        // ---------------------------------------------------------- 08_3
        // no: the exact check, and what it shows
        let line_3 = say(&kit, stage, LINES[2])?;
        swap(stage, &[track, arc, eleven_is, degrees], &[], 0.3)?;
        let checks = [
            caption(stage, UNIT, touch(), 2)?,
            caption(stage, APART, touch(), 3)?,
            caption(stage, INSIDE, touch(), 4)?,
        ];
        for (k, phrase) in ["every tile", "none overlap", "all are inside"].iter().enumerate() {
            line_3.wait_for(stage, phrase)?;
            swap(stage, &[], &[checks[k]], 0.3)?;
        }
        line_3.wait_for(stage, "big enough")?;
        let enough = caption(stage, ENOUGH, Palette::yellow(), 5)?;
        swap(stage, &[], &[enough], 0.4)?;

        // ---------------------------------------------------------- 08_4
        // a fit against a proof
        let line_4 = say(&kit, stage, LINES[3])?;
        let mut done = vec![eleven_side, eleven_turn, enough];
        done.extend(checks);
        let a_fit = caption(stage, A_FIT, ink(), 2)?;
        let a_proof = caption(stage, A_PROOF, ink(), 3)?;
        swap(stage, &done, &[a_fit, a_proof], 0.5)?;
        line_4.wait_for(stage, "like the one")?;
        let for_five = caption(stage, FOR_FIVE, dim(), 4)?;
        swap(stage, &[], &[for_five], 0.4)?;

        // ---------------------------------------------------------- 08_5
        // a cut back to five: the packing and the proof, B exactly
        let line_5 = say(&kit, stage, LINES[4])?;
        let mut out = eleven_view;
        out.extend([a_fit, a_proof, for_five]);
        swap(stage, &out, &[], 0.5)?;
        let five = model::five_turned();
        let five_view = draw_packing(stage, BOARD, &five);
        swap(stage, &[], &five_view, 0.6)?;
        let fits = caption(stage, FITS, touch(), 2)?;
        let narrower = caption(stage, NONE_NARROWER, touch(), 3)?;
        swap(stage, &[], &[fits, narrower], 0.5)?;
        line_5.wait_for(stage, "The smallest box")?;
        let b_is = tex(stage, B_IS, 0)?;
        swap(stage, &[], &[b_is], 0.5)?;

        // the line and its breath, and the closing reveal stands before the
        // wipe
        let up = stage.scene().time().to_f64();
        stand(&kit, stage, up)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The chapter's lines are its script's, and every phrase a beat waits
    /// for is in the line that beat speaks, exactly, in the order the beat
    /// uses them (08_3's three checks in turn).
    #[test]
    fn the_lines_are_the_chapters() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        let ids: Vec<&str> = script
            .lines()
            .iter()
            .map(|(id, _)| id.as_str())
            .filter(|id| id.starts_with("08_"))
            .collect();
        assert_eq!(ids, LINES.to_vec());
        let source = include_str!("what_is_proved.rs");
        let mut from = [0usize; LINES.len()];
        let mut cues = 0;
        for (k, _) in source.match_indices("line_") {
            let rest = &source[k + "line_".len()..];
            let Some(n) = rest.chars().next().and_then(|c| c.to_digit(10)) else {
                continue;
            };
            let call = &rest[1..];
            let Some(phrase) = [".wait_for(stage, \"", ".at(\""]
                .iter()
                .find_map(|p| call.strip_prefix(p))
                .and_then(|s| s.split('"').next())
            else {
                continue;
            };
            let i = n as usize - 1;
            let text = script.text(LINES[i]).unwrap();
            let at = text[from[i]..]
                .find(phrase)
                .unwrap_or_else(|| panic!("{} has no {phrase:?} at or after its earlier cues", LINES[i]));
            from[i] += at;
            cues += 1;
        }
        assert!(cues >= 6, "only {cues} cues found");
        let l3 = script.text(LINES[2]).unwrap();
        let mut at = 0;
        for phrase in ["every tile", "none overlap", "all are inside", "big enough"] {
            at += l3[at..]
                .find(phrase)
                .unwrap_or_else(|| panic!("08_3 has no {phrase:?} in order"));
        }
    }

    /// The larger boxes and the panel stay in the frame.
    #[test]
    fn the_layout_fits_the_frame() {
        for p in [model::ten(), model::eleven()] {
            let side = p.side.to_f64();
            for q in [(0.0, 0.0), (side, side)] {
                let at = WIDE.p(side, q);
                assert!(
                    at[0].abs() < FRAME_X_RADIUS - 0.3 && at[1].abs() < FRAME_Y_RADIUS - 0.3,
                    "{at:?}"
                );
                assert!(at[0] < TEXT_X - 0.4, "{at:?} reaches the panel");
            }
        }
        let b = model::turned_side().to_f64();
        assert!(BOARD.p(b, (b, b))[0] < TEXT_X - 0.4);
        assert!(ROWS.windows(2).all(|r| r[0] > r[1]));
    }
}
