//! Chapter 07, one answer two ways: the two balances solved once, the left
//! put into the right's and multiplied by four (07_1); sixteen, fifteen and
//! six, so the right is two fifths (07_2); the left three fifths, and
//! fifteen not zero, so one solution (07_3); the walker by its routes, up
//! the pair or across then up (07_4), across and back to start again
//! (07_5), so nine sixteenths plus a sixteenth of itself, three fifths
//! (07_6); why that worked here (07_7); the prediction, the chance of the
//! bottom first (07_8); two fifths, the two chances adding to one (07_9).

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    END_LABEL_GAP, LABEL_GAP, LABEL_SIZE, Picture, Strand, fade_in_all, fade_out_all, fail, flash_all, link_spot,
    move_all, pause, picture, recolor, say, small_tex, tex, think, wait_until, walker_dot,
};
use crate::facts::{self, Routes, Solve};
use crate::model::{BOTTOM, LEFT, Q, RIGHT, TOP};
use crate::palette;

pub struct OneAnswer {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const BOTH: &str = "voltage or chance";
const BOTTOM_FIRST: &str = "chance of reaching the bottom first";
const UNKNOWN: &str = "?";

/// Where the picture stands, and the rows on the right.
const PICTURE_AT: [f64; 2] = [-4.0, 0.3];
const PANEL_X: f64 = 3.2;
const ROW_Y: [f64; 6] = [3.05, 2.1, 1.15, 0.2, -0.75, -1.7];
const ROW_SIZE: f64 = 34.0;
/// The prediction's countdown, in the panel.
const THINK_AT: [f64; 2] = [3.2, 0.2];
/// The walker's size and step; how far the routes' chances sit from their
/// links.
const WALKER_R: f64 = 0.11;
const STEP_S: f64 = 0.5;
const PAIR_GAP: f64 = -0.45;
const ROUTE_GAP: f64 = 0.35;
/// A lower-third's fade in and out, and the beat after the last line.
const CARD_IN: f64 = 0.4;
const CARD_OUT: f64 = 0.3;
const END_BEAT: f64 = 0.6;

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second, from the moment it is fully in.
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// `q` as a coefficient: nothing for one, else the number and a dot.
fn times(q: Q) -> String {
    if q == Q::ONE { String::new() } else { format!("{q}·") }
}

/// 07_1 to 07_3: the solve's rows, from the facts' solve.
fn solve_rows(s: &Solve) -> [String; 6] {
    let (a, cross) = (tex(s.a), tex(s.cross));
    let (cl, cr) = (tex(s.left_constant), tex(s.right_constant));
    let lone = |c: &str, x: &str| if c == "1" { x.to_owned() } else { format!("{c}{x}") };
    let (right, left) = (lone(&cross, "R"), lone(&cross, "L"));
    [
        format!("{a}L = {cl} + {right}"),
        format!("{a}R = {cr} + {left}"),
        format!(r"L = \frac{{{cl} + {right}}}{{{a}}}"),
        format!(r"{a}R = {cr} + \frac{{{cl} + {right}}}{{{a}}}"),
        format!("{}R = {} + {cl} + {right}", tex(s.sixteen), tex(s.four)),
        format!(
            r"{}R = {},\ R = \tfrac{{{}}}{{{}}} = {}",
            tex(s.fifteen),
            tex(s.six),
            tex(s.six),
            tex(s.fifteen),
            small_tex(s.right)
        ),
    ]
}

/// 07_3: the left from the right.
fn left_row(s: &Solve) -> String {
    format!(
        r"L = \frac{{{} + {}}}{{{}}} = \frac{{{}}}{{{}}} = {}",
        tex(s.left_constant),
        small_tex(s.right),
        tex(s.a),
        small_tex(s.left_sum),
        tex(s.a),
        small_tex(s.left)
    )
}

/// 07_3's exact version (card H): the two balances' determinant, from the
/// facts' solve.
fn determinant_card(s: &Solve) -> String {
    let (a, c) = (times(s.a), times(s.cross));
    format!(
        "{a}left − {c}right = {} and −{c}left + {a}right = {} have determinant {}·{} − {}·{} = {}, not 0, so \
         exactly one solution.",
        s.left_constant, s.right_constant, s.a, s.a, s.cross, s.cross, s.fifteen
    )
}

/// 07_4 to 07_6: the routes' rows, from the facts' routes. The chance of
/// not coming back and the chance of the top first share a denominator, so
/// their quotient is written over it unreduced first.
fn route_rows(r: &Routes) -> Result<[String; 4], String> {
    if r.exit_top.denom() != r.kept.denom() {
        return Err(format!("{} and {} do not share a denominator", r.exit_top, r.kept));
    }
    Ok([
        format!(
            r"{} + {} = {}",
            small_tex(r.up_now),
            small_tex(r.across_up),
            small_tex(r.exit_top)
        ),
        format!(r"S = {} + {}\,S", small_tex(r.exit_top), small_tex(r.back)),
        format!(r"{}\,S = {}", small_tex(r.kept), small_tex(r.exit_top)),
        format!(
            r"S = \tfrac{{{}}}{{{}}} = {}",
            r.exit_top.numer(),
            r.kept.numer(),
            small_tex(r.sum)
        ),
    ])
}

/// A junction's label on `pic`. Detached.
fn label(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, x: usize, src: &str, color: Srgb) -> fmn::Result<Mob> {
    let gap = if x == TOP || x == BOTTOM { LABEL_GAP } else { END_LABEL_GAP };
    kit.tex(stage, src, LABEL_SIZE, color, &[], pic.label_spot(x, gap))
}

/// A row of the panel. Detached.
fn row(kit: &Kit, stage: &mut Stage<'_>, src: &str, i: usize) -> fmn::Result<Mob> {
    kit.tex(stage, src, ROW_SIZE, Palette::ink(), &[], v(PANEL_X, ROW_Y[i]))
}

/// The walker takes `route` on `pic`, each step through its wire's middle.
fn walk_route(stage: &mut Stage<'_>, pic: &Picture, dot: Mob, route: &[usize]) -> fmn::Result<()> {
    for step in route.windows(2) {
        let s: &Strand = pic.strand_between(step[0], step[1])?;
        move_all(stage, &[(dot, pic.mid(s))], STEP_S / 2.0)?;
        move_all(stage, &[(dot, pic.spot(step[1]))], STEP_S / 2.0)?;
    }
    Ok(())
}

impl SceneConstruct for OneAnswer {
    fn name(&self) -> &str {
        "07_one_answer_two_ways"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let net = facts::network().map_err(fail)?;
        let volts = facts::voltages().map_err(fail)?;
        let walk = net.walk().map_err(fail)?;
        let solve = facts::solve(&net, &volts).map_err(fail)?;
        let routes = facts::routes_from_left(&net, &walk).map_err(fail)?;
        let rows = solve_rows(&solve);
        let route_text = route_rows(&routes).map_err(fail)?;

        // the network, "voltage or chance": the top 1, the bottom 0, both
        // middle junctions unknown
        let colors: Vec<Srgb> = (0..net.size())
            .map(|x| match x {
                TOP => palette::top(),
                BOTTOM => palette::bottom(),
                _ => palette::unknown(),
            })
            .collect();
        let pic = picture(stage, &net, PICTURE_AT, &colors)?;
        let name = kit.text(stage, BOTH, 26.0, Palette::dim(), pic.name_spot())?;
        let mut labels = [
            label(&kit, stage, &pic, TOP, &tex(walk.top[TOP]), Palette::ink())?,
            label(&kit, stage, &pic, LEFT, UNKNOWN, Palette::hero())?,
            label(&kit, stage, &pic, RIGHT, UNKNOWN, Palette::hero())?,
            label(&kit, stage, &pic, BOTTOM, &tex(walk.top[BOTTOM]), Palette::ink())?,
        ];
        let mut arriving = pic.mobs();
        arriving.push(name);
        arriving.extend(labels);
        fade_in_all(stage, &arriving, 0.6)?;

        // 07_1: the two balances; the left from the first, put into the
        // second
        let line = say(&kit, stage, "07_1")?;
        let balances = [row(&kit, stage, &rows[0], 0)?, row(&kit, stage, &rows[1], 1)?];
        fade_in_all(stage, &balances, 0.4)?;
        line.wait_for(stage, "the left is two plus the right")?;
        let left_from = row(&kit, stage, &rows[2], 2)?;
        fade_in_all(stage, &[left_from], 0.4)?;
        line.wait_for(stage, "Put that into the second")?;
        let put = row(&kit, stage, &rows[3], 3)?;
        fade_in_all(stage, &[put], 0.4)?;

        // 07_2: times four, sixteen of the right; fifteen of it is six; the
        // right is two fifths
        let line = say(&kit, stage, "07_2")?;
        line.wait_for(stage, "Sixteen times the right")?;
        let sixteen = row(&kit, stage, &rows[4], 4)?;
        fade_in_all(stage, &[sixteen], 0.4)?;
        line.wait_for(stage, "So fifteen times the right")?;
        let fifteen = row(&kit, stage, &rows[5], 5)?;
        fade_in_all(stage, &[fifteen], 0.4)?;
        line.wait_for(stage, "the right is two fifths")?;
        let right_value = label(&kit, stage, &pic, RIGHT, &tex(solve.right), Palette::ink())?;
        fade_out_all(stage, &[labels[RIGHT]], 0.15)?;
        fade_in_all(stage, &[right_value], 0.15)?;
        labels[RIGHT] = right_value;
        recolor(stage, &[(pic.junctions[RIGHT], palette::value(solve.right))], 0.3)?;

        // 07_3: the left from the right; fifteen is not zero (card H)
        let line = say(&kit, stage, "07_3")?;
        let working = [left_from, put, sixteen];
        fade_out_all(stage, &working, 0.3)?;
        move_all(stage, &[(fifteen, v(PANEL_X, ROW_Y[2]))], 0.4)?;
        let left_solved = row(&kit, stage, &left_row(&solve), 3)?;
        fade_in_all(stage, &[left_solved], 0.4)?;
        line.wait_for(stage, "which is three fifths")?;
        let left_value = label(&kit, stage, &pic, LEFT, &tex(solve.left), Palette::ink())?;
        fade_out_all(stage, &[labels[LEFT]], 0.15)?;
        fade_in_all(stage, &[left_value], 0.15)?;
        labels[LEFT] = left_value;
        recolor(stage, &[(pic.junctions[LEFT], palette::value(solve.left))], 0.3)?;
        line.wait_for(stage, "because fifteen")?;
        let card_text = determinant_card(&solve);
        let card = precisely(&kit, stage, &card_text)?;
        fade_in_all(stage, &[card], CARD_IN)?;
        let card_read = stage.scene().time().to_f64() + read_s(&card_text);

        // 07_4: the walker's routes from the left that reach the top before
        // coming back: straight up the pair, or across and then up
        let line = say(&kit, stage, "07_4")?;
        let mut solved_rows = balances.to_vec();
        solved_rows.extend([fifteen, left_solved]);
        fade_out_all(stage, &solved_rows, 0.3)?;
        let dot = walker_dot(stage, pic.spot(LEFT), WALKER_R, 1.0);
        fade_in_all(stage, &[dot], 0.3)?;
        line.wait_for(stage, "one half plus one sixteenth")?;
        let up_now = kit.tex(
            stage,
            &small_tex(routes.up_now),
            30.0,
            palette::walker(),
            &[],
            link_spot(&pic, TOP, LEFT, PAIR_GAP),
        )?;
        let across_up = kit.tex(
            stage,
            &small_tex(routes.across_up),
            30.0,
            palette::walker(),
            &[],
            link_spot(&pic, TOP, RIGHT, ROUTE_GAP),
        )?;
        let exit = row(&kit, stage, &route_text[0], 0)?;
        fade_in_all(stage, &[up_now, across_up, exit], 0.4)?;

        // 07_5: each route flashes; across and back, the walk starts again
        wait_until(stage, card_read)?;
        let line = say(&kit, stage, "07_5")?;
        fade_out_all(stage, &[card], CARD_OUT)?;
        let pair: Vec<Mob> = pic
            .strands
            .iter()
            .filter(|s| (s.a, s.b) == (TOP, LEFT))
            .map(|s| s.mob)
            .collect();
        let bridge = pic.strand_between(LEFT, RIGHT)?.mob;
        let right_up = pic.strand_between(TOP, RIGHT)?.mob;
        line.wait_for(stage, "Straight up the pair")?;
        flash_all(stage, &pair, 0.5)?;
        line.wait_for(stage, "across and then up")?;
        flash_all(stage, &[bridge, right_up], 0.5)?;
        line.wait_for(stage, "comes back to the left")?;
        let back = kit.tex(
            stage,
            &small_tex(routes.back),
            30.0,
            palette::walker(),
            &[],
            link_spot(&pic, LEFT, RIGHT, ROUTE_GAP),
        )?;
        fade_in_all(stage, &[back], 0.3)?;
        walk_route(stage, &pic, dot, &[LEFT, RIGHT, LEFT])?;
        flash_all(stage, &[dot], 0.4)?;

        // 07_6: its chance is nine sixteenths plus a sixteenth of itself
        let line = say(&kit, stage, "07_6")?;
        let itself = row(&kit, stage, &route_text[1], 1)?;
        fade_in_all(stage, &[itself], 0.4)?;
        line.wait_for(stage, "Fifteen sixteenths")?;
        let kept = row(&kit, stage, &route_text[2], 2)?;
        fade_in_all(stage, &[kept], 0.4)?;
        line.wait_for(stage, "which makes it three fifths")?;
        let sum = row(&kit, stage, &route_text[3], 3)?;
        fade_in_all(stage, &[sum], 0.4)?;
        flash_all(stage, &[sum, labels[LEFT]], 0.6)?;

        // 07_7: every detour here is the one trip across and back
        let line = say(&kit, stage, "07_7")?;
        line.wait_for(stage, "the same trip across and back")?;
        flash_all(stage, &[bridge], 0.6)?;

        // 07_8: the bottom first: the ends swap, both middle junctions
        // unknown, the countdown
        say(&kit, stage, "07_8")?;
        let swapped = walk.bottom.clone();
        let new_name = kit.text(stage, BOTTOM_FIRST, 26.0, Palette::dim(), pic.name_spot())?;
        let asked = [
            label(&kit, stage, &pic, TOP, &tex(swapped[TOP]), Palette::ink())?,
            label(&kit, stage, &pic, LEFT, UNKNOWN, Palette::hero())?,
            label(&kit, stage, &pic, RIGHT, UNKNOWN, Palette::hero())?,
            label(&kit, stage, &pic, BOTTOM, &tex(swapped[BOTTOM]), Palette::ink())?,
        ];
        let mut old = vec![name, up_now, across_up, back, exit, itself, kept, sum, dot];
        old.extend(labels);
        fade_out_all(stage, &old, 0.3)?;
        let mut new = vec![new_name];
        new.extend(asked);
        fade_in_all(stage, &new, 0.3)?;
        recolor(
            stage,
            &[
                (pic.junctions[TOP], palette::value(swapped[TOP])),
                (pic.junctions[LEFT], palette::unknown()),
                (pic.junctions[RIGHT], palette::unknown()),
                (pic.junctions[BOTTOM], palette::value(swapped[BOTTOM])),
            ],
            0.3,
        )?;
        let countdown = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 07_9: two fifths and three fifths; the two chances add to one
        let line = say(&kit, stage, "07_9")?;
        let answers = [
            label(&kit, stage, &pic, LEFT, &tex(swapped[LEFT]), Palette::ink())?,
            label(&kit, stage, &pic, RIGHT, &tex(swapped[RIGHT]), Palette::ink())?,
        ];
        fade_out_all(stage, &[countdown, asked[LEFT], asked[RIGHT]], 0.15)?;
        fade_in_all(stage, &answers, 0.15)?;
        recolor(
            stage,
            &[
                (pic.junctions[LEFT], palette::value(swapped[LEFT])),
                (pic.junctions[RIGHT], palette::value(swapped[RIGHT])),
            ],
            0.3,
        )?;
        line.wait_for(stage, "add to one")?;
        let total = walk.top[LEFT] + swapped[LEFT];
        if total != Q::ONE {
            return Err(fail(format!("the left's two chances add to {total}, not 1")));
        }
        let adds = row(
            &kit,
            stage,
            &format!("{} + {} = {}", small_tex(walk.top[LEFT]), small_tex(swapped[LEFT]), tex(total)),
            0,
        )?;
        fade_in_all(stage, &[adds], 0.4)?;
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Every phrase the scene waits for is in its line of script.tsv, once.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("07_1", "the left is two plus the right"),
            ("07_1", "Put that into the second"),
            ("07_2", "Sixteen times the right"),
            ("07_2", "So fifteen times the right"),
            ("07_2", "the right is two fifths"),
            ("07_3", "which is three fifths"),
            ("07_3", "because fifteen"),
            ("07_4", "one half plus one sixteenth"),
            ("07_5", "Straight up the pair"),
            ("07_5", "across and then up"),
            ("07_5", "comes back to the left"),
            ("07_6", "Fifteen sixteenths"),
            ("07_6", "which makes it three fifths"),
            ("07_7", "the same trip across and back"),
            ("07_9", "add to one"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The rows and the card are the facts' solve and routes, in the order
    /// the narration speaks them.
    #[test]
    fn the_rows_are_the_facts() {
        let net = facts::network().unwrap();
        let volts = facts::voltages().unwrap();
        let walk = net.walk().unwrap();
        let s = facts::solve(&net, &volts).unwrap();
        assert_eq!(
            solve_rows(&s),
            [
                "4L = 2 + R",
                "4R = 1 + L",
                r"L = \frac{2 + R}{4}",
                r"4R = 1 + \frac{2 + R}{4}",
                "16R = 4 + 2 + R",
                r"15R = 6,\ R = \tfrac{6}{15} = \tfrac{2}{5}",
            ]
        );
        assert_eq!(
            left_row(&s),
            r"L = \frac{2 + \tfrac{2}{5}}{4} = \frac{\tfrac{12}{5}}{4} = \tfrac{3}{5}"
        );
        assert_eq!(
            determinant_card(&s),
            "4·left − right = 2 and −left + 4·right = 1 have determinant 4·4 − 1·1 = 15, not 0, so exactly one \
             solution."
        );
        let r = facts::routes_from_left(&net, &walk).unwrap();
        assert_eq!(
            route_rows(&r).unwrap(),
            [
                r"\tfrac{1}{2} + \tfrac{1}{16} = \tfrac{9}{16}",
                r"S = \tfrac{9}{16} + \tfrac{1}{16}\,S",
                r"\tfrac{15}{16}\,S = \tfrac{9}{16}",
                r"S = \tfrac{9}{15} = \tfrac{3}{5}",
            ]
        );
        // the bottom first is one less the top first
        assert_eq!((walk.bottom[LEFT], walk.bottom[RIGHT]), (Q::new(2, 5), Q::new(3, 5)));
    }

    /// Card H fits a lower-third of three lines and can be read over 07_3's
    /// rest and 07_4, by the scratch voice with this video's gaps.
    #[test]
    fn the_card_fits_and_can_be_read() {
        let net = facts::network().unwrap();
        let s = facts::solve(&net, &facts::voltages().unwrap()).unwrap();
        let text = determinant_card(&s);
        assert!(explainer_kit::animatic::wrap(&text, 84, 10).len() <= 3);
        let script = explainer_kit::Script::parse(crate::SCRIPT)
            .unwrap()
            .with_line_gaps(crate::EXPLAINER.line_gaps);
        let voice = explainer_kit::Narrator::estimate(&script);
        let len = |id: &str| voice.length(id).unwrap_or_else(|| panic!("no line {id}"));
        let line = script.text("07_3").unwrap();
        let at = line[..line.find("because fifteen").unwrap()].chars().count() as f64;
        let up = len("07_3") * (1.0 - at / line.chars().count() as f64) - CARD_IN
            + voice.line_gap("07_3", "07_4")
            + len("07_4");
        assert!(read_s(&text) <= up, "{:.2} s to read, {up:.2} s up", read_s(&text));
    }

    /// The panel's rows stay right of the picture and inside the title-safe
    /// area, above a three-line lower-third.
    #[test]
    fn the_panel_keeps_clear_of_the_picture() {
        use crate::chapters::draw::THINK_R;
        const { assert!(PANEL_X - 2.8 > PICTURE_AT[0] + 1.7 + END_LABEL_GAP + 0.3) };
        const { assert!(PANEL_X + 2.8 < FRAME_X_RADIUS - 0.25) };
        const { assert!(ROW_Y[0] + 0.45 < FRAME_Y_RADIUS - 0.15) };
        const { assert!(ROW_Y[5] - 0.3 > -FRAME_Y_RADIUS + 1.58) };
        const { assert!(THINK_AT[1] + THINK_R + 0.3 < ROW_Y[0] - 0.3) };
    }
}
