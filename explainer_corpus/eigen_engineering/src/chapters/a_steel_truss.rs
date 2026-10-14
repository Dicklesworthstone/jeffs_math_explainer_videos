//! Chapter 05, a steel truss: the carts' idea on a bridge. A frame of
//! triangles twenty-four metres long, one end pinned and the other on a
//! roller (05_1). Each member is a spring, k = EA/L: three hundred
//! kilonewtons to stretch the bottom one a millimetre (05_2). Twelve joints
//! make twenty-four moves, the supports hold three, twenty-one are left
//! (05_3). So the rule is a table again, twenty-one by twenty-one, written
//! a member at a time, most of it empty (05_4). Hang a hundred kilonewtons
//! on one joint: only that one, its neighbours, or every joint (05_5)? Every
//! joint but the pinned one, and the table's inverse is full (05_6). Load
//! the five deck joints and the middle sags eighteen millimetres (05_7);
//! the top is squeezed hardest, 450 kilonewtons, under a third of yield
//! (05_8).
//!
//! Every number is a fact: `K-T1.*` for the frame, `K-T2.*` for the
//! table, `K-T3.*` for the lone load, `K-T4.*` for the deck load, `K-T5.*`
//! for the forces; every position and move on the picture is the model's
//! stiffness method (`model::truss`), drawn through `truss.rs`.

use explainer_kit::props::{CodePanel, lower_third};
use explainer_kit::{Kit, Palette, fade_all, play, thousands, v};
use fmn::prelude::*;

use super::draw::{
    chip, fact, fade_in_all, fade_in_anim, fade_out_all, flat, group, minus, moves, numbers, own, predict_hold, pushes,
    run, say, squeezed, stretched, tween_all, tween_linear, wait_until, whole,
};
use super::marks::{mark, note};
use super::truss::{Layout, StiffnessTable, TrussPic, dimension, load_arrow, supports};
use crate::model::truss::{LONE_JOINT, Truss};

// ------------------------------------------------------------------ choices

/// Where 05_6's "1.3 mm" (the roller's slide) sits: above and to the right
/// of the roller joint, in the clear between the end of the truss and the
/// table's lattice (it stood 0.75 to the right and touched the lattice).
pub fn slide_label_at(roller: Vec3) -> Vec3 {
    v(roller[0] + 0.45, roller[1] + 0.45)
}

/// Where 05_8's "0 kN" tag sits: against the middle post, 0.34 to its right
/// and 0.78 of the way up, where the right diagonal (it rises one to one from
/// the post's foot) is still farther from the post than the words reach (at
/// the post's middle, 0.7 to the right, it had the diagonal through it).
pub fn post_label_at(bottom_mid: Vec3, top_mid: Vec3) -> Vec3 {
    v(
        bottom_mid[0] + 0.34,
        bottom_mid[1] + 0.78 * (top_mid[1] - bottom_mid[1]),
    )
}

/// The truss on screen: the pinned end, and scene units to a metre.
const LAYOUT: Layout = Layout {
    origin: [-6.3, 1.0, 0.0],
    scale: 0.27,
};
/// How many times larger than life the moves are drawn: the lone load
/// (storyboard 05_6) and the five deck loads (05_7).
const MAG_LONE: f64 = 200.0;
const MAG_DECK: f64 = 60.0;
/// The table's lattice: its top-left corner and the side of a cell.
const TABLE_AT: Vec3 = [1.2, 3.25, 0.0];
const CELL: f64 = 0.15;
/// How far down the code panel sits (it slides in from the right edge).
const PANEL_DROP: f64 = -1.7;
/// The member of 05_2 that is a spring: the second bottom member.
const SPRING_MEMBER: usize = 1;
/// 05_4's two lower-thirds say, in words, what the voice no longer does
/// (D-25): they are read for these seconds from the moment each is whole,
/// words at 4.5 a second at the most (it was 0.5 s and 0.7 s, after the
/// line had ended and nothing was left to hold them).
const WHY_READ_S: f64 = 3.0;
const ZERO_READ_S: f64 = 3.5;
/// A strip's fade, in and out.
const STRIP_FADE_S: f64 = 0.4;

/// 05_4's first strip: its label and its words.
const WHY: (&str, &str) = ("each member", "writes its spring only into its own two joints' rows");

/// 05_4's second strip: its label and its words (the count is a fact).
fn zero_words() -> fmn::Result<(&'static str, String)> {
    Ok((
        "without supports:",
        format!(
            "the table has {} own patterns of stiffness zero: slide, lift, turn",
            fact("K-X3.unsupported_zero")?
        ),
    ))
}
/// The code panel's lines (storyboard 05_4: the direct stiffness method).
const CODE: [&str; 9] = [
    "for each member from joint a to joint b:",
    "  d = pos[b] - pos[a]; L = length(d)",
    "  (cx, sy) = d / L",
    "  g = (-cx, -sy, cx, sy)",
    "  rows r = (2a, 2a+1, 2b, 2b+1)",
    "  K[r_i][r_j] += (E A / L) g_i g_j",
    "then strike out the 3 held rows",
    "and columns",
    "(joint j owns rows 2j and 2j+1, from 0)",
];

// ------------------------------------------------------------- the geometry

/// `6000` as TeX with its thousands separator in braces (`6{,}000`).
pub fn thousands_tex(n: u64) -> String {
    thousands(n).replace(',', "{,}")
}

/// How many of a joint's two directions (along the span, up and down) the
/// supports hold still: 2 at the pin, 1 at the roller, 0 anywhere else.
pub fn held_at(truss: &Truss, joint: usize) -> usize {
    truss.held.iter().filter(|&&d| d / 2 == joint).count()
}

/// The tonnes a force of `kn` kilonewtons weighs, to the nearest ten:
/// "about 30 tonnes" for the spring of 05_2.
pub fn about_tonnes(tonnes: f64) -> u64 {
    ((tonnes / 10.0).round() * 10.0) as u64
}

// -------------------------------------------------------------- the scene

pub struct ASteelTruss {
    pub kit: Kit,
}

impl SceneConstruct for ASteelTruss {
    fn name(&self) -> &str {
        "05_a_steel_truss"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (ink, dim) = (Palette::ink(), Palette::dim());
        let (blue, orange, gold) = (moves(), pushes(), own());

        // the model, solved the way the chapter says it is
        let t = Truss::pratt();
        let lone = t.all_moves(&t.moves(&t.lone_load()));
        let deck = t.all_moves(&t.moves(&t.deck_load()));
        let forces = t.member_forces(&t.moves(&t.deck_load()));
        let pic = TrussPic::build(stage, &t, LAYOUT, [lone, deck], [MAG_LONE, MAG_DECK], forces)?;
        let table = StiffnessTable::build(stage, &t, TABLE_AT, CELL)?;

        // ---- 05_1: the truss draws itself, joint by joint
        let line = say(&kit, stage, "05_1")?;
        pic.start(stage)?;
        let chord_mm2 = numbers("K-T1.chord_mm2")?[0] as u64;
        let web_mm2 = numbers("K-T1.web_mm2")?[0] as u64;
        let legend = [
            kit.tex(
                stage,
                &format!(
                    r"\text{{chords and end posts}}\ {}\,\text{{mm}}^2",
                    thousands_tex(chord_mm2)
                ),
                30.0,
                ink,
                &[],
                v(-3.2, 3.55),
            )?,
            kit.tex(
                stage,
                &format!(
                    r"\text{{posts and diagonals}}\ {}\,\text{{mm}}^2",
                    thousands_tex(web_mm2)
                ),
                30.0,
                ink,
                &[],
                v(-3.2, 3.05),
            )?,
        ];
        let sw = supports(stage, &t, LAYOUT);
        let span_m = numbers("K-T1.span_m")?[0];
        let depth_m = numbers("K-T1.depth_m")?[0];
        let bottom = LAYOUT.at(0.0, 0.0);
        let right = LAYOUT.at(span_m, 0.0);
        let mut dims = dimension(stage, v(bottom[0], bottom[1] - 0.85), v(right[0], right[1] - 0.85), dim);
        // the depth, at the roller's end (the pinned end is at the frame's edge)
        dims.extend(dimension(
            stage,
            v(right[0] + 0.45, right[1]),
            v(right[0] + 0.45, right[1] + LAYOUT.scale * depth_m),
            dim,
        ));
        let span_label = kit.text(
            stage,
            &format!("{} m", whole(span_m)),
            26.0,
            dim,
            v((bottom[0] + right[0]) / 2.0, bottom[1] - 1.2),
        )?;
        let depth_label = kit.text(
            stage,
            &format!("{} m", whole(depth_m)),
            26.0,
            dim,
            v(right[0] + 0.95, right[1] + LAYOUT.scale * depth_m / 2.0),
        )?;
        let model_tag = chip(&kit, stage, "a model", 26.0, dim, v(3.4, 1.55))?;
        line.wait_for(stage, "a steel truss bridge")?;
        play!(stage; tween_linear(pic.drawn, t.members.len() as f64, 3.4)?);
        line.wait_for(stage, "twenty-four metres long")?;
        let mut dims_in = dims.clone();
        dims_in.extend([span_label, depth_label]);
        fade_in_all(stage, &dims_in, 0.6)?;
        fade_in_all(stage, &[legend[0], legend[1], model_tag], 0.6)?;
        line.wait_for(stage, "One end is pinned")?;
        fade_in_all(stage, &sw, 0.6)?;

        // ---- 05_2: each member is a spring
        let line = say(&kit, stage, "05_2")?;
        stage.set_tracker_value(pic.lit, SPRING_MEMBER as f64)?;
        let spring_mid = {
            let m = t.members[SPRING_MEMBER];
            let (a, b) = (LAYOUT.joint(&t, m.a), LAYOUT.joint(&t, m.b));
            v((a[0] + b[0]) / 2.0, a[1])
        };
        let member_label = kit.text(stage, "member", 26.0, ink, v(spring_mid[0], spring_mid[1] - 0.45))?;
        let e_gpa = whole(numbers("K-T1.e_gpa")?[0]);
        let area = thousands_tex(chord_mm2);
        let panel_mm = thousands_tex((numbers("K-T1.panel_m")?[0] * 1000.0).round() as u64);
        let spring = whole(numbers("K-T1.spring_kn_per_mm")?[0]);
        let numbers_line = kit.tex(
            stage,
            &format!(
                r"{e_gpa}\,\text{{kN/mm}}^2 \times {area}\,\text{{mm}}^2 \div {panel_mm}\,\text{{mm}} = {spring}\,\text{{kN per mm}}"
            ),
            36.0,
            ink,
            &[],
            v(-0.4, -1.3),
        )?;
        let tonnes = numbers("K-T1.spring_tonnes")?[0];
        let weight = kit.text(
            stage,
            &format!("{spring} kN is the weight of about {} tonnes", about_tonnes(tonnes)),
            26.0,
            dim,
            v(-0.4, -2.0),
        )?;
        let letters = kit.tex(stage, r"k = \frac{EA}{L}", 52.0, gold, &[], v(-4.2, -3.0))?;
        let words = kit.text(
            stage,
            "how stiff steel is, times how thick, over how long",
            26.0,
            ink,
            v(0.9, -3.0),
        )?;
        line.wait_for(stage, "Each member is a spring")?;
        fade_in_all(stage, &[member_label], 0.5)?;
        line.wait_for(stage, "three hundred kilonewtons")?;
        fade_in_all(stage, &[numbers_line], 0.7)?;
        fade_in_all(stage, &[weight], 0.5)?;
        fade_in_all(stage, &[letters, words], 0.7)?;

        // ---- 05_3: twelve joints, twenty-four moves, three held, twenty-one left
        let line = say(&kit, stage, "05_3")?;
        stage.set_tracker_value(pic.lit, -1.0)?;
        let mut leaving = dims_in.clone();
        leaving.extend([member_label, numbers_line, weight, letters, words, model_tag]);
        fade_out_all(stage, &leaving, 0.5)?;
        let (joints_n, held, free) = (
            whole(numbers("K-T1.joints")?[0]),
            whole(numbers("K-T1.held")?[0]),
            whole(numbers("K-T1.free")?[0]),
        );
        let count = kit.tex(
            stage,
            &format!(r"{joints_n} \times 2 - {held} = {free}"),
            50.0,
            ink,
            &[],
            v(3.6, 2.2),
        )?;
        // how many of its two directions each support holds, from the model
        let (pin_holds, roller_holds) = (held_at(&t, 0), held_at(&t, crate::model::truss::PANELS));
        let pin_tag = kit.text_left(
            stage,
            &format!("holds both ways: {pin_holds}"),
            22.0,
            dim,
            v(-6.9, LAYOUT.origin[1] - 0.95),
        )?;
        let roller_tag = kit.text(
            stage,
            &format!("holds up and down only: {roller_holds}"),
            22.0,
            dim,
            v(right[0] - 0.9, right[1] - 0.95),
        )?;
        let flat_tag = kit.text(stage, "a flat model: it cannot sway or twist", 26.0, dim, v(3.6, 1.4))?;
        line.wait_for(stage, "Each joint can move along the span")?;
        tween_all(stage, &[(pic.arrows_on, 1.0)], 0.9)?;
        line.wait_for(stage, "Twelve joints make twenty-four moves")?;
        fade_in_all(stage, &[count, pin_tag, roller_tag], 0.6)?;
        line.wait_for(stage, "so twenty-one are left")?;
        fade_in_all(stage, &[flat_tag], 0.6)?;

        // ---- 05_4: the rule is a table again, 21 by 21, mostly empty
        let line = say(&kit, stage, "05_4")?;
        fade_out_all(stage, &[count, pin_tag, roller_tag, flat_tag], 0.4)?;
        tween_all(stage, &[(pic.arrows_on, 0.0)], 0.4)?;
        table.start(stage)?;
        let mirror = table.diagonal(stage);
        stage.add_to_scene(mirror)?;
        fade_in_all(stage, &[table.group], 0.6)?;
        let size_label = kit.text(
            stage,
            &format!("{0} by {0}", whole(numbers("K-T1.free")?[0])),
            26.0,
            dim,
            v(TABLE_AT[0] + table.size() / 2.0, TABLE_AT[1] + 0.35),
        )?;
        fade_in_all(stage, &[size_label], 0.4)?;
        let why = lower_third(&kit, stage, WHY.0, WHY.1)?;
        line.wait_for(stage, "twenty-one by twenty-one")?;
        // one member first, on its own: the strip says what a member does as
        // its truss member lights and its cells fill, and stays for its read
        fade_in_all(stage, &[why], STRIP_FADE_S)?;
        mark(&kit, stage, "05_4.why_in");
        let why_whole = stage.scene().time().to_f64();
        stage.set_tracker_value(pic.lit, 0.0)?;
        play!(stage;
            tween_linear(table.progress, 1.0, 1.2)?,
            tween_linear(pic.lit, 0.999, 1.2)?,
        );
        wait_until(stage, why_whole + WHY_READ_S)?;
        kit.hold(stage)?;
        // then the rest in a rush, with the second strip coming as the first
        // goes: it is about the table the rush fills
        let (zero_label, zero_text) = zero_words()?;
        let zero_strip = lower_third(&kit, stage, zero_label, &zero_text)?;
        let members = t.members.len() as f64;
        mark(&kit, stage, "05_4.why_out");
        // one strip goes before the next comes: both stand in the lower third
        fade_out_all(stage, &[why], 0.3)?;
        let zero_whole = stage.scene().time().to_f64() + STRIP_FADE_S;
        note(&kit, "05_4.zero_whole", zero_whole);
        let rush: Vec<Box<dyn Animation>> = vec![
            fade_in_anim(stage, zero_strip, STRIP_FADE_S)?,
            stage.prepare(tween_linear(table.progress, members, 2.0)?)?,
            stage.prepare(tween_linear(pic.lit, members, 2.0)?)?,
        ];
        stage.play_prepared(rush)?;
        stage.set_tracker_value(pic.lit, -1.0)?;
        let mirror_in = vec![stage.prepare(mirror.animate().set_anim_args(run(0.5))?.set_opacity(0.9)?)?];
        stage.play_prepared(mirror_in)?;
        // on the hold: the code panel, for programmers
        let panel = CodePanel::build(&kit, stage, &CODE, 20.0, 6.0)?;
        stage.shift(panel.group, v(0.0, PANEL_DROP));
        stage.add_to_scene(panel.group)?;
        stage.play(panel.slide_in(0.8)?)?;
        // the second strip is read from the moment it was whole
        wait_until(stage, zero_whole + ZERO_READ_S)?;
        kit.hold(stage)?;
        mark(&kit, stage, "05_4.zero_out");
        fade_out_all(stage, &[zero_strip], 0.3)?;

        // ---- 05_5: a hundred kilonewtons on one joint: who moves?
        let line = say(&kit, stage, "05_5")?;
        let load_at = pic.layout.joint(&t, LONE_JOINT);
        let lone_arrow = load_arrow(stage, v(load_at[0], load_at[1] - 0.05), 0.6, orange);
        let load_label = kit.text(
            stage,
            &format!(
                "{} kN, about {} tonnes",
                fact("K-T3.load_kn")?,
                about_tonnes(numbers("K-T3.load_tonnes")?[0])
            ),
            26.0,
            orange,
            v(load_at[0] + 0.2, load_at[1] - 0.95),
        )?;
        let choices = {
            let back = Rectangle::new()
                .width(5.6)
                .height(1.8)
                .corner_radius(0.12)
                .style(Style::default().fill(super::draw::panel(), 0.96).stroke(dim, 2.0, 1.0))
                .build()?
                .moved_to(v(-3.4, -2.55));
            let back = stage.arena_mut().add(back);
            let rows = [
                kit.text_left(stage, "only that joint", 28.0, ink, v(-6.0, -2.05))?,
                kit.text_left(stage, "its neighbours", 28.0, ink, v(-6.0, -2.55))?,
                kit.text_left(stage, "every joint", 28.0, ink, v(-6.0, -3.05))?,
            ];
            group(stage, &[back, rows[0], rows[1], rows[2]])?
        };
        line.wait_for(stage, "Hang a hundred kilonewtons")?;
        fade_in_all(stage, &[lone_arrow, load_label], 0.6)?;
        line.wait_for(stage, "Will only that joint move")?;
        fade_in_all(stage, &[choices], 0.6)?;
        predict_hold(&kit, stage)?;

        // ---- 05_6: every joint but the pinned one, and the table's inverse is full
        let line = say(&kit, stage, "05_6")?;
        fade_out_all(stage, &[choices, load_label], 0.4)?;
        let back_out = vec![stage.prepare(panel.group.animate().set_anim_args(run(0.6))?.shift([
            panel.width + 0.45,
            0.0,
            0.0,
        ])?)?];
        stage.play_prepared(back_out)?;
        // off the right edge again: out of the scene too, or the lint (and a
        // wide camera) would find it there
        stage.remove_from_scene(panel.group);
        stage.set_tracker_value(pic.arrows_mode, 1.0)?;
        tween_all(stage, &[(pic.arrows_on, 1.0)], 0.4)?;
        line.wait_for(stage, "Every joint but the pinned one")?;
        tween_all(stage, &[(pic.amp, 1.0)], 1.6)?;
        let tag_mag = kit.text(
            stage,
            &format!("moves \u{d7} {}", MAG_LONE as u64),
            26.0,
            dim,
            v(6.0, 3.65),
        )?;
        let held_tag = kit.text(stage, "held", 24.0, dim, v(LAYOUT.origin[0], LAYOUT.origin[1] - 0.75))?;
        let lone_drop = kit.text(
            stage,
            &format!("{} mm", fact("K-T3.drop_mm")?),
            26.0,
            ink,
            v(load_at[0] + 0.1, load_at[1] - 0.8),
        )?;
        let roller_pos = pic.layout.joint(&t, crate::model::truss::PANELS);
        let lone_slide = kit.text(
            stage,
            &format!("{} mm", fact("K-T3.slide_mm")?),
            26.0,
            ink,
            slide_label_at(roller_pos),
        )?;
        note(
            &kit,
            "05_6.slide_gap",
            TABLE_AT[0] - (stage.get_center(lone_slide)[0] + stage.get_width(lone_slide) / 2.0),
        );
        fade_in_all(stage, &[tag_mag, held_tag, lone_drop, lone_slide], 0.6)?;
        line.wait_for(stage, "still they all move")?;
        // the table turns into its inverse
        let counts = [
            kit.text(
                stage,
                &format!(
                    "the table: {} of {} not zero",
                    fact("K-T2.nonzero")?,
                    fact("K-T2.entries")?
                ),
                26.0,
                ink,
                v(TABLE_AT[0] + table.size() / 2.0, TABLE_AT[1] - table.size() - 0.35),
            )?,
            kit.text(
                stage,
                &format!(
                    "its inverse: {} of {}",
                    fact("K-T2.inverse_nonzero")?,
                    fact("K-T2.entries")?
                ),
                26.0,
                ink,
                v(TABLE_AT[0] + table.size() / 2.0, TABLE_AT[1] - table.size() - 0.8),
            )?,
        ];
        fade_in_all(stage, &[counts[0]], 0.5)?;
        tween_all(stage, &[(table.inverse_mix, 1.0)], 1.2)?;
        fade_in_all(stage, &[counts[1]], 0.5)?;

        // ---- 05_7: load all five deck joints: the middle sags
        let line = say(&kit, stage, "05_7")?;
        fade_out_all(
            stage,
            &[lone_arrow, lone_drop, lone_slide, held_tag, counts[0], counts[1]],
            0.4,
        )?;
        tween_all(stage, &[(pic.amp, 0.0), (table.inverse_mix, 0.0)], 0.9)?;
        stage.set_tracker_value(pic.field, 1.0)?;
        let mut loads = Vec::new();
        for j in t.deck_joints() {
            let p = pic.layout.joint(&t, j);
            loads.push(load_arrow(stage, v(p[0], p[1] - 0.05), 0.55, orange));
        }
        let equation = kit.tex(
            stage,
            r"K\,u = f",
            56.0,
            ink,
            &[("u", blue), ("f", orange)],
            v(5.9, 2.3),
        )?;
        let solved = kit.text(
            stage,
            "21 equations, solved together",
            24.0,
            dim,
            v(TABLE_AT[0] + table.size() / 2.0, TABLE_AT[1] - table.size() - 0.35),
        )?;
        let tag_deck = kit.text(
            stage,
            &format!("moves \u{d7} {}", MAG_DECK as u64),
            26.0,
            dim,
            v(6.0, 3.65),
        )?;
        let mid = pic.layout.joint(&t, t.midspan());
        let sag = kit.text(
            stage,
            &format!("{} mm", fact("K-T4.sag_mm_2dp")?),
            28.0,
            ink,
            v(mid[0], mid[1] - 1.05),
        )?;
        line.wait_for(stage, "Load all five deck joints")?;
        fade_out_all(stage, &[tag_mag], 0.3)?;
        fade_in_all(stage, &loads, 0.6)?;
        fade_in_all(stage, &[equation, solved, tag_deck], 0.6)?;
        tween_all(stage, &[(pic.amp, 1.0)], 1.8)?;
        line.wait_for(stage, "The middle sags eighteen millimetres")?;
        fade_in_all(stage, &[sag], 0.5)?;
        let programmers = lower_third(
            &kit,
            stage,
            "for programmers:",
            "real programs factor K once, or iterate, and reuse it for every load; nobody computes its eigenvectors for this",
        )?;
        fade_in_all(stage, &[programmers], 0.5)?;
        kit.hold(stage)?;

        // ---- 05_8: the top is squeezed hardest
        let line = say(&kit, stage, "05_8")?;
        let mut gone = loads.clone();
        gone.extend([
            equation,
            solved,
            tag_deck,
            sag,
            programmers,
            table.group,
            size_label,
            mirror,
        ]);
        fade_out_all(stage, &gone, 0.5)?;
        table.stop(stage);
        let why_force = lower_third(
            &kit,
            stage,
            "a member's force:",
            "how far its ends moved apart, times its stiffness",
        )?;
        fade_in_all(stage, &[why_force], 0.5)?;
        line.wait_for(stage, "The top is squeezed hardest")?;
        tween_all(stage, &[(pic.force_on, 1.0)], 1.2)?;
        // the forces, from the facts, beside the members that carry them
        let top_kn = numbers("K-T5.top_kn")?[0];
        let bottom_kn = numbers("K-T5.bottom_kn")?[0];
        let post_kn = numbers("K-T5.middle_post_kn")?[0];
        let mid_top = pic.layout.at(t.span_m() / 2.0, 4.0);
        let top_label = kit.text(
            stage,
            &format!("{} kN", minus(&whole(top_kn))),
            26.0,
            squeezed(),
            v(mid_top[0], mid_top[1] + 0.35),
        )?;
        let bottom_label = kit.text(
            stage,
            &format!("+{} kN", whole(bottom_kn)),
            26.0,
            stretched(),
            v(mid[0], mid[1] - 1.0),
        )?;
        let post_label = kit.text(
            stage,
            &format!("{} kN", whole(post_kn)),
            26.0,
            dim,
            post_label_at(mid, mid_top),
        )?;
        fade_in_all(stage, &[top_label, bottom_label, post_label], 0.6)?;
        // the gauge: the stress against what makes the steel yield
        let stress = numbers("K-T5.stress_mpa")?[0];
        let limit = numbers("K-T1.yield_mpa")?[0];
        let (gx, gy, gw) = (3.3, -0.9, 3.4);
        let track = {
            let r = Rectangle::new()
                .width(gw)
                .height(0.3)
                .corner_radius(0.05)
                .style(flat(Palette::faint(), 1.0))
                .build()?
                .moved_to(v(gx + gw / 2.0, gy));
            stage.arena_mut().add(r)
        };
        let share = stress / limit;
        let fill = {
            let r = Rectangle::new()
                .width(gw * share)
                .height(0.3)
                .corner_radius(0.05)
                .style(flat(squeezed(), 1.0))
                .build()?
                .moved_to(v(gx + gw * share / 2.0, gy));
            stage.arena_mut().add(r)
        };
        let gauge_label = kit.tex(
            stage,
            &format!(
                r"{}\ \text{{of}}\ {}\ \text{{N per mm}}^2\text{{, yield}}",
                whole(stress),
                whole(limit)
            ),
            30.0,
            ink,
            &[],
            v(gx + gw / 2.0, gy + 0.5),
        )?;
        // a squeezed member is also checked for buckling, another eigenvalue problem
        let mut buckling = Vec::new();
        for (k, words) in [
            "a squeezed member is also",
            "checked for buckling, another",
            "eigenvalue problem",
        ]
        .into_iter()
        .enumerate()
        {
            buckling.push(kit.text(stage, words, 24.0, dim, v(gx + gw / 2.0, gy - 0.65 - 0.38 * k as f64))?);
        }
        let mut gauge = vec![track, fill, gauge_label];
        gauge.extend(&buckling);
        fade_in_all(stage, &gauge, 0.7)?;
        kit.hold(stage)?;
        pic.stop(stage);
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::super::draw::testing;
    use super::super::marks;
    use super::*;
    use crate::chapters::truss::{member_entries, nonzero_count, partial_table};
    use crate::model::truss::PANEL_M;

    fn words(s: &str) -> f64 {
        s.split_whitespace().count() as f64
    }

    /// 05_4: the two lower-thirds that say in words what D-25 took from the
    /// voice are each on screen, whole, for a read of at most 4.5 words a
    /// second (they were 0.5 s and 0.7 s, set after the line had ended), and
    /// the beat's silence before 05_5 stays under 6.5 s.
    #[test]
    fn the_two_strips_of_the_table_beat_are_read_and_the_hold_stays_short() {
        let voiced = testing::run_voiced("05_a_steel_truss").unwrap();
        let tag = &voiced.tag;
        let why = marks::at(tag, "05_4.why_out") - marks::at(tag, "05_4.why_in");
        let zero = marks::at(tag, "05_4.zero_out") - marks::at(tag, "05_4.zero_whole");
        let (zl, zt) = zero_words().unwrap();
        let (why_words, zero_words) = (words(WHY.0) + words(WHY.1), words(zl) + words(&zt));
        assert!(why >= why_words / 4.5, "{why:.2} s for {why_words} words");
        assert!(why >= WHY_READ_S - 1e-6, "{why:.2} s");
        assert!(zero >= zero_words / 4.5, "{zero:.2} s for {zero_words} words");
        assert!(zero >= ZERO_READ_S - 1e-6, "{zero:.2} s");
        let (_, s4, l4) = voiced.lines.iter().find(|(id, ..)| id == "05_4").cloned().unwrap();
        let (_, s5, _) = voiced.lines.iter().find(|(id, ..)| id == "05_5").cloned().unwrap();
        assert!(s5 - (s4 + l4) <= 6.5, "{:.1} s of silence before 05_5", s5 - (s4 + l4));
    }

    /// 05_6: the roller's "1.3 mm" ends clear of the lattice (0.15 or more);
    /// 05_8: the "0 kN" tag has no member through it and stands beside the
    /// middle post it names.
    #[test]
    fn the_roller_slide_and_the_post_tag_are_clear_of_what_they_were_on() {
        let voiced = testing::run_voiced("05_a_steel_truss").unwrap();
        let gap = marks::at(&voiced.tag, "05_6.slide_gap");
        assert!(gap >= 0.15, "the lattice is {gap:.2} from the label");

        let t = Truss::pratt();
        let mid = LAYOUT.joint(&t, t.midspan());
        let top = LAYOUT.at(t.span_m() / 2.0, 4.0);
        let at = post_label_at(mid, top);
        // "0 kN" at 26 pt is about 0.47 wide and 0.3 high; 0.04 is added all round
        let (hw, hh) = (0.28, 0.2);
        let hits = |a: Vec3, b: Vec3| {
            // Liang-Barsky against the plate
            let (mut t0, mut t1) = (0.0_f64, 1.0_f64);
            let d = [b[0] - a[0], b[1] - a[1]];
            for (p, q) in [
                (-d[0], a[0] - (at[0] - hw)),
                (d[0], (at[0] + hw) - a[0]),
                (-d[1], a[1] - (at[1] - hh)),
                (d[1], (at[1] + hh) - a[1]),
            ] {
                if p.abs() < 1e-12 {
                    if q < 0.0 {
                        return false;
                    }
                } else {
                    let r = q / p;
                    if p < 0.0 {
                        t0 = t0.max(r);
                    } else {
                        t1 = t1.min(r);
                    }
                }
            }
            t0 <= t1
        };
        let crossing: Vec<(usize, usize)> = t
            .members
            .iter()
            .filter(|m| hits(LAYOUT.joint(&t, m.a), LAYOUT.joint(&t, m.b)))
            .map(|m| (m.a, m.b))
            .collect();
        assert!(crossing.is_empty(), "members through the tag: {crossing:?}");
        // and it is the middle post's: its left edge is within 0.2 of the post
        assert!(
            (at[0] - hw - mid[0]).abs() <= 0.2,
            "{:.2} from the post",
            at[0] - hw - mid[0]
        );
        assert!(at[1] + hh < top[1], "below the top chord");
    }

    /// 05_2: the spring is EA/L from the facts' steel, section and length,
    /// 300 kilonewtons a millimetre, and a weight of about thirty tonnes.
    #[test]
    fn a_member_is_a_spring_of_three_hundred_kilonewtons_a_millimetre() {
        let e = numbers("K-T1.e_gpa").unwrap()[0]; // kN per mm squared
        let a = numbers("K-T1.chord_mm2").unwrap()[0];
        let l = numbers("K-T1.panel_m").unwrap()[0] * 1000.0;
        let k = e * a / l;
        assert!((k - numbers("K-T1.spring_kn_per_mm").unwrap()[0]).abs() < 1e-9, "{k}");
        let t = Truss::pratt();
        let bottom = t.members[SPRING_MEMBER];
        assert_eq!(bottom.kind, crate::model::truss::Kind::Bottom);
        assert!((t.length_m(&bottom) - PANEL_M).abs() < 1e-12);
        assert_eq!(about_tonnes(numbers("K-T1.spring_tonnes").unwrap()[0]), 30);
        assert_eq!(thousands_tex(6000), "6{,}000");
    }

    /// 05_3: the pin holds both of its directions and the roller only up and
    /// down, no other joint is held, and together they are the facts' three.
    #[test]
    fn the_pin_holds_two_directions_and_the_roller_one() {
        let t = Truss::pratt();
        assert_eq!(held_at(&t, 0), 2, "the pin");
        assert_eq!(held_at(&t, crate::model::truss::PANELS), 1, "the roller");
        let elsewhere: usize = (1..t.joints.len())
            .filter(|&j| j != crate::model::truss::PANELS)
            .map(|j| held_at(&t, j))
            .sum();
        assert_eq!(elsewhere, 0);
        assert_eq!(held_at(&t, 0) + held_at(&t, crate::model::truss::PANELS), 3);
        assert_eq!(fact("K-T1.held").unwrap(), "3");
    }

    /// 05_3 and 05_4: 12 x 2 - 3 = 21, and the table the members write is
    /// 441 cells, 97 of them not zero, written a member at a time.
    #[test]
    fn twelve_joints_two_ways_each_less_three_is_twenty_one_and_the_table_is_mostly_empty() {
        let t = Truss::pratt();
        assert_eq!(t.joints.len() * 2 - t.held.len(), t.free().len());
        assert_eq!(t.free().len(), 21);
        let entries = member_entries(&t);
        let n = 21;
        let full = partial_table(&entries, entries.len() as f64, n);
        assert_eq!(nonzero_count(&full), 97);
        assert_eq!(n * n, 441);
        // the first member alone writes four cells: one joint's two rows, both ways
        let first = partial_table(&entries, 1.0, n);
        assert_eq!(
            nonzero_count(&first),
            1,
            "a bottom member next to the pin: one free direction"
        );
    }

    /// 05_6 and 05_7: the moves drawn are the model's, and the sag is the
    /// fact's 17.96 mm at midspan under five loads of 100 kN.
    #[test]
    fn the_moves_drawn_are_the_models_and_the_sag_is_the_facts() {
        let t = Truss::pratt();
        let lone = t.all_moves(&t.moves(&t.lone_load()));
        let drop_mm = -lone[2 * LONE_JOINT + 1] * 1e3;
        assert!(
            (drop_mm - numbers("K-T3.drop_mm").unwrap()[0]).abs() < 0.05,
            "{drop_mm}"
        );
        let deck = t.all_moves(&t.moves(&t.deck_load()));
        let sag_mm = -deck[2 * t.midspan() + 1] * 1e3;
        assert!(
            (sag_mm - numbers("K-T4.sag_mm_2dp").unwrap()[0]).abs() < 0.005,
            "{sag_mm}"
        );
        // every free joint but the pinned one moves under the lone load
        assert_eq!(numbers("K-T3.moving").unwrap()[0] as usize, 21);
        // magnified, the sag stays under the truss's own depth on screen
        assert!(sag_mm * 1e-3 * MAG_DECK * LAYOUT.scale < 4.0 * LAYOUT.scale);
        assert!(drop_mm * 1e-3 * MAG_LONE * LAYOUT.scale < 4.0 * LAYOUT.scale);
    }

    /// 05_8: the forces the colours show are the facts': the top squeezed at
    /// 450, the bottom pulled at 400, the middle post carrying nothing; the
    /// gauge is 75 of 250 N per mm squared.
    #[test]
    fn the_forces_and_the_gauge_are_the_facts() {
        let t = Truss::pratt();
        let n = t.member_forces(&t.moves(&t.deck_load()));
        let kn = |i: usize| n[i] / 1e3;
        let top = (0..n.len()).map(kn).fold(f64::INFINITY, f64::min);
        let bottom = (0..n.len()).map(kn).fold(f64::NEG_INFINITY, f64::max);
        assert!((top - numbers("K-T5.top_kn").unwrap()[0]).abs() < 0.05);
        assert!((bottom - numbers("K-T5.bottom_kn").unwrap()[0]).abs() < 0.05);
        assert_eq!(numbers("K-T5.middle_post_kn").unwrap()[0], 0.0);
        let stress = numbers("K-T5.stress_mpa").unwrap()[0];
        let limit = numbers("K-T1.yield_mpa").unwrap()[0];
        assert_eq!((stress, limit), (75.0, 250.0));
        assert!((stress / limit - 0.3).abs() < 1e-12, "under a third");
    }

    /// The chapter runs on the storyboard with the layout lint clean, types
    /// the strings it should, and stays inside the chapter budget.
    #[test]
    fn the_chapter_runs_on_its_storyboard_and_the_layout_lint_is_clean() {
        let ran = testing::run("05_a_steel_truss").unwrap();
        assert_eq!(ran.lint_count, 0, "{}", ran.lint);
        let words: Vec<&str> = ran.registered.iter().map(|r| r.source.as_str()).collect();
        for want in [
            "a model",
            r"\text{chords and end posts}\ 6{,}000\,\text{mm}^2",
            r"k = \frac{EA}{L}",
            "300 kN is the weight of about 30 tonnes",
            r"12 \times 2 - 3 = 21",
            "holds both ways: 2",
            "a flat model: it cannot sway or twist",
            "its inverse: 441 of 441",
            r"K\,u = f",
            "17.96 mm",
            "eigenvalue problem",
            "for programmers",
        ] {
            assert!(
                words.iter().any(|w| w.contains(want)),
                "the chapter never typesets {want:?}: {words:?}"
            );
        }
        // the length that counts is the one with the scratch voice placed
        let s = testing::run_voiced("05_a_steel_truss").unwrap().ran.seconds;
        eprintln!("05_a_steel_truss: {s:.1} s on the scratch voice");
        assert!(s > 45.0 && s < 85.0, "{s} s");
    }
}
