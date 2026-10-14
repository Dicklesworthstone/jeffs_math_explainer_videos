//! Chapter 02, the marks measure the net: Feller's puzzle (02_1), red spots
//! become the gold glow (02_2), one fact read two ways as proportion bars
//! (02_3–02_6), the soup guess in numbers and then in letters (02_7), and
//! the three questions the video answers (02_8).

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{precisely, predict_pause, proportion_bar};
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, rect, thousands, v};
use fmn::animation::replacement_transform;
use fmn::prelude::*;

use super::draw::{fish_parts, flat, marked_fish};
use crate::facts::FELLER;

/// The proportion bars: width, height, left edge.
const BAR_W: f64 = 8.0;
const BAR_H: f64 = 0.55;
const BAR_X: f64 = -4.0;

/// A card behind text: the corpus's raised fill, a faint edge.
fn card(w: f64, h: f64, at: Vec3) -> fmn::Result<VMobject> {
    Ok(Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(
            Style::default()
                .fill(hex("#1A2130"), 0.94)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(at))
}

/// `count` copies of a bar segment `w` wide side by side from `left`, as one
/// group: the "copied ten times" stack.
fn copies(stage: &mut Stage<'_>, count: usize, left: f64, y: f64, w: f64, color: Srgb) -> fmn::Result<Mob> {
    let segs: Vec<VMobject> = (0..count)
        .map(|i| {
            rect(
                v(left + w * i as f64, y - BAR_H / 2.0),
                w - 0.04,
                BAR_H,
                flat(color, 0.85),
            )
        })
        .collect::<fmn::Result<_>>()?;
    Ok(stage.arena_mut().add(v_group(segs)))
}

pub struct Marks {
    pub kit: Kit,
}

impl SceneConstruct for Marks {
    fn name(&self) -> &str {
        "02_the_marks_measure_the_net"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (big_m, big_n, k) = FELLER;
        let lake = big_m * big_n / k;
        let seen = big_m + big_n - k;

        // 02_1: Feller's puzzle, in his first edition's words
        let quote = kit.text_styled(
            stage,
            "“Suppose that 1000 fish caught in a lake are marked by red spots…”",
            36.0,
            Palette::ink(),
            v(0.0, 0.45),
            false,
            true,
        )?;
        let cite = kit.text(
            stage,
            "William Feller, An Introduction to Probability Theory and Its Applications (1950), p. 37",
            22.0,
            Palette::dim(),
            v(0.0, -0.45),
        )?;
        let w = stage.get_width(quote).max(stage.get_width(cite)) + 0.8;
        let back = stage.arena_mut().add(card(w, 2.0, ORIGIN)?);
        kit.say(stage, "02_1")?;
        play!(stage;
            fade_in(stage.arena_mut(), back, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), quote, v(0.0, -0.15), 1.0)?.rt(1.2),
            fade_in(stage.arena_mut(), cite, ORIGIN, 1.0)?.rt(1.2),
        );
        kit.hold(stage)?;

        // 02_2: a red spot becomes the gold glow
        let [body, tail] = fish_parts(Palette::blue());
        let spot = Circle::new()
            .radius(0.016)
            .arc_center(v(0.03, 0.006))
            .style(flat(Palette::red(), 1.0))
            .build();
        let red = stage.arena_mut().add(v_group([body, tail, spot]));
        stage.scale(red, 9.0);
        stage.move_to(red, v(0.0, -0.2), ORIGIN);
        let gold = stage.arena_mut().add(marked_fish());
        stage.scale(gold, 9.0);
        stage.move_to(gold, v(0.0, -0.2), ORIGIN);
        kit.say(stage, "02_2")?;
        play!(stage;
            fade_out(stage.arena_mut(), quote, v(0.0, 1.0), 1.0)?.rt(0.8),
            fade_out(stage.arena_mut(), cite, v(0.0, 1.0), 1.0)?.rt(0.8),
            fade_out(stage.arena_mut(), back, v(0.0, 1.0), 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), red, ORIGIN, 1.0)?.rt(0.8),
        );
        stage.wait(0.6)?;
        stage.play(replacement_transform(red, gold).rt(1.4))?;
        kit.hold(stage)?;

        // 02_3: one reading. Of the thousand marked, the second net caught a
        // hundred: one in ten
        let gold_bar = proportion_bar(
            stage,
            v(BAR_X, 1.6),
            BAR_W,
            BAR_H,
            k as f64 / big_m as f64,
            Palette::hero(),
            Palette::gold(),
        )?;
        let marked_label = kit.text_left(
            stage,
            &format!("{} marked", thousands(big_m)),
            30.0,
            Palette::gold(),
            v(BAR_X, 2.25),
        )?;
        let twice_label = kit.text_left(
            stage,
            &format!("{} caught twice", thousands(k)),
            26.0,
            Palette::hero(),
            v(BAR_X, 0.95),
        )?;
        let one_in_ten = kit.text(stage, "one in ten", 30.0, Palette::ink(), v(5.55, 1.6))?;
        kit.say(stage, "02_3")?;
        play!(stage; fade_out(stage.arena_mut(), gold, v(0.0, 0.5), 0.2)?.rt(0.8));
        play!(stage;
            fade_in(stage.arena_mut(), gold_bar.group, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), marked_label, ORIGIN, 1.0)?.rt(0.8),
        );
        play!(stage; fmn::animation::indicate(stage.arena_mut(), gold_bar.part, 1.1, None)?.rt(1.0));
        play!(stage;
            fade_in(stage.arena_mut(), twice_label, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), one_in_ten, ORIGIN, 1.0)?.rt(0.6),
        );
        kit.hold(stage)?;

        // 02_4: so the second net caught about one fish in ten. A thousand
        // is a tenth of how many?
        let green_bar = proportion_bar(
            stage,
            v(BAR_X, -0.6),
            BAR_W,
            BAR_H,
            0.0,
            Palette::green(),
            Palette::green(),
        )?;
        let second_label = kit.text_left(
            stage,
            &format!("{} in the second catch", thousands(big_n)),
            30.0,
            Palette::green(),
            v(BAR_X, 0.05),
        )?;
        let ask = kit.text(
            stage,
            &format!("{} = one tenth of ?", thousands(big_n)),
            40.0,
            Palette::ink(),
            v(0.0, -2.1),
        )?;
        kit.say(stage, "02_4")?;
        play!(stage;
            fade_in(stage.arena_mut(), green_bar.group, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), second_label, ORIGIN, 1.0)?.rt(0.8),
        );
        play!(stage; write(stage.arena(), ask).rt(1.0));
        let q = predict_pause(&kit, stage, v(5.6, -2.1))?;

        // 02_5: ten thousand. The green catch copied ten times stacks into
        // the lake; all but the 1,900 fish we have seen dim
        let unit = 1.0;
        let left = -5.0;
        let stack = copies(stage, 10, left, -0.6, unit, Palette::green())?;
        let seen_at = left + 10.0 * unit * seen as f64 / lake as f64;
        let dark = stage.arena_mut().add(rect(
            v(seen_at, -0.6 - BAR_H / 2.0 - 0.02),
            left + 10.0 * unit - seen_at,
            BAR_H + 0.04,
            flat(hex("#0B0E13"), 0.72),
        )?);
        let total = kit.text(
            stage,
            &format!("{} fish", thousands(lake)),
            30.0,
            Palette::ink(),
            v(left + 5.0 * unit, -1.35),
        )?;
        let never = kit.text(
            stage,
            &format!(
                "{} − {} = {} never caught",
                thousands(lake),
                thousands(seen),
                thousands(lake - seen)
            ),
            30.0,
            Palette::dim(),
            v(0.0, -2.1),
        )?;
        kit.say(stage, "02_5")?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            fade_out(stage.arena_mut(), ask, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), green_bar.group, ORIGIN, 1.0)?.rt(0.5),
        );
        let reveal = fade_in(stage.arena_mut(), stack, v(-0.3, 0.0), 1.0)?;
        play!(stage; reveal.rt(1.8).lag(0.3));
        play!(stage; fade_in(stage.arena_mut(), total, ORIGIN, 1.0)?.rt(0.6));
        play!(stage;
            fade_in(stage.arena_mut(), dark, ORIGIN, 1.0)?.rt(1.2),
            fade_in(stage.arena_mut(), never, ORIGIN, 1.0)?.rt(1.2),
        );
        kit.hold(stage)?;

        // 02_6: the other reading: one in ten of the second catch is marked,
        // so about one in ten of the lake; the marked fish copied ten times
        fade_all(stage, 0.8)?;
        let marked_side = proportion_bar(
            stage,
            v(-6.4, 1.7),
            5.4,
            BAR_H,
            k as f64 / big_m as f64,
            Palette::hero(),
            Palette::gold(),
        )?;
        let catch_side = proportion_bar(
            stage,
            v(0.9, 1.7),
            5.4,
            BAR_H,
            k as f64 / big_n as f64,
            Palette::hero(),
            Palette::green(),
        )?;
        let left_words = kit.text_left(
            stage,
            &format!("{} of {} marked were caught again", thousands(k), thousands(big_m)),
            24.0,
            Palette::gold(),
            v(-6.4, 2.3),
        )?;
        let right_words = kit.text_left(
            stage,
            &format!("{} of {} caught are marked", thousands(k), thousands(big_n)),
            24.0,
            Palette::green(),
            v(0.9, 2.3),
        )?;
        let marked_stack = copies(stage, 10, left, -0.5, unit, Palette::gold())?;
        let lake_words = kit.text(
            stage,
            &format!("{} marked × 10 = {} in the lake", thousands(big_m), thousands(lake)),
            30.0,
            Palette::ink(),
            v(0.0, -1.25),
        )?;
        kit.say(stage, "02_6")?;
        play!(stage;
            fade_in(stage.arena_mut(), marked_side.group, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), left_words, ORIGIN, 1.0)?.rt(0.8),
        );
        play!(stage;
            fade_in(stage.arena_mut(), catch_side.group, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), right_words, ORIGIN, 1.0)?.rt(0.8),
        );
        let note = precisely(
            &kit,
            stage,
            "E[K] = nM/N for every size of second catch, by linearity: each of the n fish caught is marked \
             with chance M/N.",
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.8));
        let reveal = fade_in(stage.arena_mut(), marked_stack, v(-0.3, 0.0), 1.0)?;
        play!(stage; reveal.rt(1.8).lag(0.3));
        play!(stage; fade_in(stage.arena_mut(), lake_words, ORIGIN, 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 02_7: the soup guess, numbers first, then letters under words
        fade_all(stage, 0.8)?;
        let numbers = kit.tex(
            stage,
            &format!(
                r"\frac{{{} \times {}}}{{{}}} = {}",
                thousands(big_m).replace(',', "{,}"),
                thousands(big_n).replace(',', "{,}"),
                thousands(k),
                thousands(lake).replace(',', "{,}")
            ),
            64.0,
            Palette::ink(),
            &[],
            v(0.0, 1.3),
        )?;
        let letters = kit.tex(
            stage,
            r"\hat{N} = \frac{M\,n}{k}",
            72.0,
            Palette::ink(),
            &[("M", Palette::gold()), ("n", Palette::green()), ("k", Palette::hero())],
            v(0.0, -0.9),
        )?;
        let legend = [
            ("M  marked", Palette::gold(), -4.0),
            ("n  second catch", Palette::green(), 0.0),
            ("k  caught twice", Palette::hero(), 4.0),
        ];
        let mut words = Vec::new();
        for (s, c, x) in legend {
            words.push(kit.text(stage, s, 28.0, c, v(x, -2.5))?);
        }
        let name = kit.text(stage, "the soup guess", 34.0, Palette::dim(), v(0.0, 3.0))?;
        kit.say(stage, "02_7")?;
        play!(stage; write(stage.arena(), numbers).rt(1.6));
        stage.wait(0.6)?;
        play!(stage; write(stage.arena(), letters).rt(1.4));
        for w in words {
            play!(stage; fade_in(stage.arena_mut(), w, v(0.0, 0.15), 1.0)?.rt(0.4));
        }
        play!(stage; fade_in(stage.arena_mut(), name, ORIGIN, 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 02_8: the three questions; each returns as an act title
        fade_all(stage, 0.8)?;
        kit.say(stage, "02_8")?;
        for (i, s) in ["Why should that work?", "How sure can we be?", "When does it fail?"]
            .into_iter()
            .enumerate()
        {
            let x = -4.6 + 4.6 * i as f64;
            let t = kit.text(stage, s, 30.0, Palette::ink(), v(x, 0.0))?;
            let b = stage.arena_mut().add(card(4.2, 1.5, v(x, 0.0))?);
            play!(stage;
                fade_in(stage.arena_mut(), b, v(0.0, -0.2), 1.0)?.rt(0.6),
                fade_in(stage.arena_mut(), t, v(0.0, -0.2), 1.0)?.rt(0.6),
            );
        }
        kit.music_breath(stage, HOLD_S)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}
