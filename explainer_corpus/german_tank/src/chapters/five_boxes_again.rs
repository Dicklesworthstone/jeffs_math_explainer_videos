//! Closing transfer: the same five observations acquire one more number.
//! Geometry and every numeric readout are checked against the exact model.
use super::draw::{STRIP_Y, fade_in_all, fade_out_all, fail, line_style, pause, say, stroke, think, wait_until, x_at};
use super::friends_line::{FriendsLine, Start, arc_points, badge, drop_in_all, list, now, read_s, whole};
use crate::model::Q;
use crate::{facts, model, palette};
use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

pub struct FiveBoxesAgain {
    pub kit: Kit,
}

const END: f64 = 110.0;
const SOURCES: [&str; 4] = [
    "An Empirical Approach to Economic Intelligence",
    "in World War II",
    "Richard Ruggles and Henry Brodie (1947)",
    "J. Amer. Statist. Assoc. — publisher abstract",
];
const CONDITION: &str = "Consecutive labels from 1; every fixed-size handful equally likely, without replacement. Best unbiased means smallest average squared miss among recipes unbiased at every count.";

#[derive(Clone)]
struct Picture {
    friends: Vec<u64>,
    gaps: Vec<u64>,
    estimate: u64,
    shop: u64,
    range: Vec<u64>,
    sixth: u64,
    six_gaps: Vec<u64>,
    size: u64,
    unseen: u64,
    average: u64,
    next: u64,
}
fn picture() -> Result<Picture, String> {
    let f = facts::facts()?;
    checked(Picture {
        friends: list(&f, "G-F1.numbers")?,
        gaps: list(&f, "G-F3.gaps")?,
        estimate: whole(&f, "G-F3.estimate")?,
        shop: whole(&f, "G-F14.shop_boxes")?,
        range: list(&f, "G-F9.range")?,
        sixth: whole(&f, "G-F16.sixth")?,
        six_gaps: list(&f, "G-F16.gaps")?,
        size: whole(&f, "G-F16.size")?,
        unseen: whole(&f, "G-F16.unseen")?,
        average: whole(&f, "G-F16.average_gap")?,
        next: whole(&f, "G-F16.estimate")?,
    })
}
fn checked(p: Picture) -> Result<Picture, String> {
    if p.friends.len() < 2
        || p.friends[0] == 0
        || !p.friends.windows(2).all(|w| w[0] < w[1])
        || p.range.len() != 2
        || p.sixth <= *p.friends.last().unwrap()
        || p.sixth as f64 >= END
        || p.range[1] as f64 >= END
    {
        return Err("invalid closing handful, range or sixth number".into());
    }
    let k = p.friends.len() as u64;
    let m = *p.friends.last().unwrap();
    let mut six = p.friends.clone();
    six.push(p.sixth);
    if p.gaps != model::gaps(&p.friends)
        || p.six_gaps != model::gaps(&six)
        || p.size != k + 1
        || p.unseen != p.six_gaps.iter().sum::<u64>()
        || Q::new(p.unseen as i128, p.size as i128) != Q::int(p.average as i128)
        || model::gap_recipe(&p.friends) != Q::int(p.estimate as i128)
        || p.shop != p.estimate
        || model::gap_recipe(&six) != Q::int(p.next as i128)
        || p.next != p.sixth + p.average
        || p.range != vec![m, model::upper_bound(m, k, Q::new(1, 20))]
    {
        return Err("closing arithmetic or confidence endpoint disagrees with facts".into());
    }
    Ok(p)
}
fn line(kit: &Kit, stage: &mut Stage<'_>, text: &str, y: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, text, 28.0, color, v(0.0, y))
}
fn bracket(stage: &mut Stage<'_>, first: u64, last: u64, y: f64, color: Srgb) -> Mob {
    let (a, b) = (x_at(first as f64, END), x_at(last as f64, END));
    stroke(
        stage,
        vec![v(a, y + 0.1), v(a, y), v(b, y), v(b, y + 0.1)],
        line_style(color, 3.0, 1.0),
    )
}
fn gap(kit: &Kit, stage: &mut Stage<'_>, left: u64, right: u64, count: u64) -> fmn::Result<Vec<Mob>> {
    let (a, b) = (x_at(left as f64 + 0.5, END), x_at(right as f64 - 0.5, END));
    Ok(vec![
        stroke(
            stage,
            arc_points(a, b, STRIP_Y - 0.17, -0.3),
            line_style(palette::unseen(), 2.5, 1.0),
        ),
        kit.text(
            stage,
            &count.to_string(),
            24.0,
            palette::unseen(),
            v((a + b) / 2.0, -1.45),
        )?,
    ])
}
impl SceneConstruct for FiveBoxesAgain {
    fn name(&self) -> &str {
        "16_five_boxes_again"
    }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let p = picture().map_err(fail)?;
        let history = say(&kit, stage, "16_1")?;
        let mut citation = Vec::new();
        for (i, text) in SOURCES.iter().enumerate() {
            citation.push(line(&kit, stage, text, 1.5 - i as f64 * 0.6, Palette::dim())?);
        }
        fade_in_all(stage, &citation, 0.6)?;
        let cited = now(stage);
        history.wait_for(stage, "Allied analysts")?;
        kit.hold(stage)?;
        wait_until(stage, cited + read_s(&SOURCES.join(" ")))?;
        fade_out_all(stage, &citation, 0.4)?;

        say(&kit, stage, "16_2")?;
        let start = || Start {
            end: END,
            fan: 0.0,
            rise: 1.0,
            widest: END,
        };
        let friends = FriendsLine::build(&kit, stage, 1, END as u64, &p.friends, &[], start())?;
        let corner = badge(&kit, stage, v(-5.1, 3.5))?;
        fade_in_all(stage, &friends.opening(), 0.6)?;
        fade_in_all(stage, &[corner], 0.3)?;
        let origin = line(
            &kit,
            stage,
            "Consecutive labels from 1; a fair handful",
            2.6,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[origin], 0.4)?;

        let l = say(&kit, stage, "16_3")?;
        fade_out_all(stage, &[origin], 0.3)?;
        l.wait_for(stage, "at least sixty")?;
        let lower = bracket(stage, 1, p.range[0], -2.0, Palette::hero());
        let lower_text = line(
            &kit,
            stage,
            &format!("At least {} boxes", p.range[0]),
            2.6,
            Palette::hero(),
        )?;
        fade_in_all(stage, &[lower, lower_text], 0.4)?;
        l.wait_for(stage, "about seventy-one")?;
        let mut arcs = Vec::new();
        let mut previous = 0;
        for (&n, &g) in p.friends.iter().zip(&p.gaps) {
            arcs.extend(gap(&kit, stage, previous, n, g)?);
            previous = n;
        }
        fade_in_all(stage, &arcs, 0.6)?;
        let estimate = line(
            &kit,
            stage,
            &format!("Gap estimate: {} boxes", p.estimate),
            1.9,
            palette::estimate(),
        )?;
        fade_in_all(stage, &[estimate], 0.4)?;
        kit.hold(stage)?;
        pause(stage, 2.5)?;

        say(&kit, stage, "16_4")?;
        let best = line(
            &kit,
            stage,
            "Unbiased at every count; smallest average squared miss",
            1.2,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[best], 0.4)?;
        kit.hold(stage)?;
        pause(stage, 1.5)?;

        let l = say(&kit, stage, "16_5")?;
        fade_out_all(stage, &[lower, best], 0.3)?;
        l.wait_for(stage, "sixty to a hundred and seven")?;
        let interval = bracket(stage, p.range[0], p.range[1], -2.0, Palette::ink());
        let interval_text = line(
            &kit,
            stage,
            &format!("95% confidence range: {} to {}", p.range[0], p.range[1]),
            -2.45,
            Palette::ink(),
        )?;
        fade_in_all(stage, &[interval, interval_text], 0.5)?;
        l.wait_for(stage, "not a probability")?;
        let meaning = line(
            &kit,
            stage,
            "95% describes the procedure, not this count's probability",
            1.15,
            Palette::dim(),
        )?;
        fade_in_all(stage, &[meaning], 0.4)?;
        kit.hold(stage)?;
        pause(stage, 2.5)?;

        let l = say(&kit, stage, "16_6")?;
        fade_out_all(stage, &[interval, interval_text, meaning, corner], 0.4)?;
        l.wait_for(stage, "one shop")?;
        let (a, b) = (x_at(1.0, END) - 0.08, x_at(p.shop as f64, END) + 0.08);
        let pool = stroke(
            stage,
            vec![
                v(a, STRIP_Y - 0.13),
                v(b, STRIP_Y - 0.13),
                v(b, STRIP_Y + 0.13),
                v(a, STRIP_Y + 0.13),
                v(a, STRIP_Y - 0.13),
            ],
            line_style(Palette::red(), 2.5, 1.0),
        );
        let shop = line(
            &kit,
            stage,
            "Only the shop's numbered stock is sampled",
            1.15,
            Palette::red(),
        )?;
        fade_in_all(stage, &[pool, shop], 0.4)?;
        kit.hold(stage)?;
        pause(stage, 2.5)?;

        let l = say(&kit, stage, "16_7")?;
        fade_out_all(stage, &[pool, shop, lower_text, estimate], 0.4)?;
        fade_in_all(stage, &[corner], 0.3)?;
        l.wait_for(stage, "box ninety")?;
        let sixth = FriendsLine::build(&kit, stage, p.sixth, p.sixth, &[p.sixth], &[], start())?;
        fade_out_all(stage, &[friends.dots[p.sixth as usize - 1]], 0.15)?;
        drop_in_all(stage, &sixth.opening(), 0.6)?;
        let q = think(&kit, stage, v(4.6, 1.65))?;

        let l = say(&kit, stage, "16_8")?;
        fade_out_all(stage, &[q], 0.3)?;
        let sixth_arc = gap(
            &kit,
            stage,
            *p.friends.last().unwrap(),
            p.sixth,
            *p.six_gaps.last().unwrap(),
        )?;
        fade_in_all(stage, &sixth_arc, 0.5)?;
        l.wait_for(stage, "eighty-four boxes")?;
        let total = line(
            &kit,
            stage,
            &format!("{} unseen boxes in {} gaps below {}", p.unseen, p.size, p.sixth),
            2.6,
            palette::unseen(),
        )?;
        fade_in_all(stage, &[total], 0.4)?;
        l.wait_for(stage, "fourteen per gap")?;
        let average = line(
            &kit,
            stage,
            &format!("{} / {} = {} boxes per gap", p.unseen, p.size, p.average),
            1.95,
            palette::unseen(),
        )?;
        fade_in_all(stage, &[average], 0.4)?;
        l.wait_for(stage, "a hundred and four")?;
        let answer = line(
            &kit,
            stage,
            &format!("{} + {} = {} boxes", p.sixth, p.average, p.next),
            1.3,
            palette::estimate(),
        )?;
        fade_in_all(stage, &[answer], 0.4)?;
        kit.hold(stage)?;
        pause(stage, 3.0)?;

        say(&kit, stage, "16_9")?;
        kit.hold(stage)?;
        let card = precisely(&kit, stage, CONDITION)?;
        fade_in_all(stage, &[card], 0.5)?;
        let full = now(stage);
        wait_until(stage, full + read_s(CONDITION))?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finished_timeline_covers_the_entire_closing_picture() {
        use explainer_kit::narration::{Narrator, Script};
        use fmn::scene::{NullSceneSink, RuntimeConfig};

        struct ClosingProbe(FiveBoxesAgain);
        impl SceneConstruct for ClosingProbe {
            fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
                self.0.construct(stage)?;
                let finished = self.0.kit.finished.borrow();
                let timeline = finished.as_ref().expect("closing chapter records its timeline");
                assert_eq!(
                    self.0.kit.narrator.as_ref().unwrap().said().len(),
                    9,
                    "all closing scratch cues are scheduled"
                );
                assert_eq!(
                    timeline.frames,
                    u64::try_from(stage.scene().time().frames()).unwrap(),
                    "the score/master timeline must include the final visual fade"
                );
                Ok(())
            }
        }

        let script = Script::parse(include_str!("../../script.tsv")).unwrap();
        for fps in [15, 60] {
            let mut kit = Kit::new().unwrap().with_narrator(Narrator::estimate(&script));
            kit.chapter = "16_five_boxes_again".to_owned();
            let config = RuntimeConfig {
                fps,
                ..RuntimeConfig::default()
            };
            fmn::run_scene(&mut ClosingProbe(FiveBoxesAgain { kit }), config, 7, &mut NullSceneSink)
                .expect("complete closing scene constructs without rendering or audio");
        }
    }

    #[test]
    fn closure_refuses_changed_range_gap_count_or_estimate() {
        let p = picture().unwrap();
        for bad in [
            Picture {
                range: vec![60, 108],
                ..p.clone()
            },
            Picture {
                unseen: 85,
                ..p.clone()
            },
            Picture { size: 5, ..p.clone() },
            Picture { next: 105, ..p.clone() },
            Picture { sixth: 60, ..p.clone() },
            Picture { shop: 72, ..p.clone() },
        ] {
            assert!(checked(bad).is_err());
        }
    }
}
