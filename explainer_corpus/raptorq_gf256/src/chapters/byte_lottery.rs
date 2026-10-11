//! Chapter 13, the byte lottery: a card with random byte amounts (13_1); the
//! danger meter at the last slot, ex-or's one in two (13_2), and the
//! prediction (13_3); one in 256, because exactly one value of one amount
//! lands in the lit part (13_4); a byte spare divides the risk by 256 on the
//! risk ladder, ex-or only halves it (13_5); the seeded race on ten pieces,
//! the oracle's own draws replayed (13_6); and a 0/1 card is an ex-or card
//! in either field (13_7).

use std::rc::Rc;

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{lower_third, precisely, predict_pause};
use explainer_kit::{Frame2, Kit, Palette, Timed, fade_all, play, thousands, v};
use fmn::prelude::*;

use super::draw::{Role, card_with_amounts, flat, group_of, line_style};
use super::props::{BucketProp, DangerMeter, RingProp, RiskLadder};
use super::voice::{Spoken, board};
use crate::decoding::{
    RACE_PIECES, RACES, RaceRun, exact_mean_spares, lit_values, mean_spares, race, random_byte_card,
};
use crate::model::{FOLD, PIECES, RING, mul, p_fail_limit, p_full, rank_gf2, rank_gf256, useless_chance};

const KEY: &str = "13_the_byte_lottery";
/// The bucket's slots, as in 05 (B-F5.danger_meter_K10).
const SLOTS: u32 = 10;

/// `x` rounded to `sig` significant digits, in plain decimals, trailing
/// zeros dropped: 0.39, 0.0015, 0.000006, 71.
pub fn sig_digits(x: f64, sig: i32) -> String {
    let places = (sig - 1 - x.abs().log10().floor() as i32).max(0) as usize;
    let s = format!("{x:.places$}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        s
    }
}

/// A chance as a percentage to two significant digits ("0.39%").
pub fn percent(p: f64) -> String {
    format!("{}%", sig_digits(100.0 * p, 2))
}

/// "1/n" for a chance that is exactly one in a whole number.
fn one_in(p: f64) -> String {
    format!("1/{}", (1.0 / p).round())
}

/// The 256 byte values as a 16 × 16 grid of dots, `size` wide, centred at
/// `at`, value v at row v / 16 and column v % 16; `color(v)` paints each.
fn byte_grid(stage: &mut Stage<'_>, at: Vec3, size: f64, color: impl Fn(u8) -> Srgb) -> fmn::Result<(Mob, Vec<Mob>)> {
    let step = size / 16.0;
    let dots: Vec<VMobject> = (0..=255u8)
        .map(|b| {
            let p = grid_pos(at, step, b);
            Dot::new().point(p).radius(step * 0.3).color(color(b)).build()
        })
        .collect();
    let group = stage.arena_mut().add(v_group(dots));
    let kids = explainer_kit::children(stage, group);
    Ok((group, kids))
}

fn grid_pos(at: Vec3, step: f64, b: u8) -> Vec3 {
    let (r, c) = (f64::from(b / 16), f64::from(b % 16));
    v(at[0] + (c - 7.5) * step, at[1] + (7.5 - r) * step)
}

pub struct ByteLottery {
    pub kit: Kit,
}

impl SceneConstruct for ByteLottery {
    fn name(&self) -> &str {
        KEY
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let board = board(KEY)?;
        self.card(&kit, stage, &board)?;
        self.last_slot(&kit, stage, &board)?;
        self.ladder(&kit, stage, &board)?;
        self.race(&kit, stage, &board)?;
        self.zeros_and_ones(&kit, stage, &board)?;
        kit.music_breath(stage, HOLD_S)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

impl ByteLottery {
    /// 13_1: a card with random byte amounts; the design is real.
    fn card(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let amounts = random_byte_card();
        let terms: Vec<(usize, u8)> = amounts.iter().copied().enumerate().collect();
        let value = terms.iter().fold(0, |acc, &(p, a)| acc ^ mul(a, PIECES[p], FOLD));
        let byte = card_with_amounts(kit, stage, &terms, Some(value), 0.95, v(0.0, 0.1))?;
        let label = kit.text(
            stage,
            "a byte card: every amount a random byte",
            34.0,
            Role::byte(),
            v(0.0, 1.35),
        )?;
        let line = Spoken::say(kit, stage, board, "13_1")?;
        play!(stage;
            fade_in(stage.arena_mut(), byte.card.group, v(0.0, -0.2), 1.0)?.rt(0.9),
            fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?.rt(0.9),
        );
        line.at(stage, 1)?;
        let note = lower_third(kit, stage, "search:", "random linear network coding")?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6));
        Ok(())
    }

    /// 13_2–13_4: the danger meter at the last slot, ex-or then bytes.
    fn last_slot(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let bucket = BucketProp::build(stage, SLOTS as usize, v(-4.3, -0.75), 2.6, 4.1)?;
        let held = SLOTS - 1;
        let ex_or_chance = useless_chance(held, SLOTS);
        let meter = DangerMeter::on(kit, stage, &bucket, 0.0)?;
        let mut water = Vec::new();
        for i in 0..held as usize {
            water.push(bucket.water(stage, i, Role::ex_or())?);
        }
        let water_g = group_of(stage, &water)?;
        let half = meter.readout(kit, stage, &one_in(ex_or_chance))?;
        let line = Spoken::say(kit, stage, board, "13_2")?;
        fade_all(stage, 0.6)?;
        play!(stage;
            fade_in(stage.arena_mut(), bucket.group, ORIGIN, 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), meter.group, ORIGIN, 1.0)?.rt(0.7),
        );
        stage.add_to_scene(meter.fill)?;
        let fill = fade_in(stage.arena_mut(), water_g, v(0.0, -0.2), 1.0)?;
        play!(stage; fill.rt(1.2).lag(0.15));
        line.at(stage, 1)?;
        // with ex-or amounts one amount has two values; the bucket holds
        // A ⊕ B, so with the A amount 1 exactly one B amount is lit
        let ex_or_lit = lit_values(1, 1);
        let mut two = Vec::new();
        for b in 0..=1u8 {
            let at = v(1.4 + 2.4 * f64::from(b), 0.0);
            let lit = ex_or_lit.contains(&b);
            let dot = Circle::new()
                .radius(0.55)
                .arc_center(at)
                .style(if lit {
                    Style::default().fill(Role::lit(), 0.35).stroke(Role::lit(), 5.0, 1.0)
                } else {
                    Style::default()
                        .fill(Palette::dim(), 0.25)
                        .stroke(Palette::dim(), 3.0, 1.0)
                })
                .build();
            let dot = stage.arena_mut().add(dot);
            let digit = kit.text(
                stage,
                &b.to_string(),
                60.0,
                if lit { Role::lit() } else { Palette::ink() },
                at,
            )?;
            let word = kit.text(
                stage,
                if lit { "useless" } else { "useful" },
                30.0,
                if lit { Role::lit() } else { Palette::ink() },
                v(at[0], at[1] - 0.95),
            )?;
            two.push(group_of(stage, &[dot, digit, word])?);
        }
        let two_caption = kit.text(
            stage,
            &format!("the {} values of one ex-or amount", 1u32 << 1),
            32.0,
            Palette::ink(),
            v(2.6, 1.3),
        )?;
        let two_g = group_of(stage, &[two[0], two[1], two_caption])?;
        play!(stage;
            meter.to(ex_or_chance, 1.0)?,
            fade_in(stage.arena_mut(), half, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), two_g, v(0.0, -0.2), 1.0)?.rt(0.8),
        );
        kit.hold(stage)?;

        // 13_3: the same bucket with byte cards: how often useless now?
        kit.say(stage, "13_3")?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &w in &water {
            let a = w.animate().set_color(Role::byte())?;
            anims.push(stage.prepare(a)?);
        }
        let a = fade_out(stage.arena_mut(), half, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a)?);
        stage.play_prepared_with(
            anims,
            PlayOverrides {
                run_time: Some(1.0),
                ..PlayOverrides::default()
            },
        )?;
        let q = predict_pause(kit, stage, v(meter.readout_at[0] + 0.3, meter.readout_at[1]))?;

        // 13_4: one in 256; one value of one amount lands in the lit part
        let byte_chance = 1.0 - p_full(256, 1, 0);
        let one_in_256 = meter.readout(kit, stage, &one_in(byte_chance))?;
        let line = Spoken::say(kit, stage, board, "13_4")?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            meter.to(byte_chance, 1.2)?,
            fade_in(stage.arena_mut(), one_in_256, ORIGIN, 1.0)?.rt(0.6),
        );
        line.at(stage, 1)?;
        // the bucket holds A ⊕ 3B; the new card's A amount is fixed at 7
        let (t, c1) = (3u8, 7u8);
        let lit = lit_values(t, c1);
        let grid_at = v(1.9, 0.25);
        let size = 4.6;
        let (grid, dots) = byte_grid(stage, grid_at, size, |_| Palette::dim())?;
        let caption = kit.text(
            stage,
            &format!("the {} values of one amount", 1u32 << 8),
            30.0,
            Palette::ink(),
            v(grid_at[0], grid_at[1] + size / 2.0 + 0.4),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), two_g, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), grid, ORIGIN, 1.0)?.rt(0.8).lag(0.004),
            fade_in(stage.arena_mut(), caption, ORIGIN, 1.0)?.rt(0.8),
        );
        line.at(stage, 2)?;
        // scaling by a nonzero byte reshuffles every value: each dot moves to
        // where the amount times 3 sits (0 stays)
        let step = size / 16.0;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for b in 0..=255u8 {
            let to = grid_pos(grid_at, step, mul(b, t, FOLD));
            let a = dots[b as usize]
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.6),
                    ..AnimateArgs::default()
                })?
                .move_to(to, ORIGIN)?;
            anims.push(stage.prepare(a)?);
        }
        stage.play_prepared(anims)?;
        line.at_share(stage, 0.7)?;
        // exactly one lands in the lit part: it lights; the other 255 do not
        let one = lit.len();
        let hit = grid_pos(grid_at, step, lit[0]);
        let ring = Circle::new()
            .radius(step * 0.75)
            .arc_center(hit)
            .style(Style::default().fill(Role::lit(), 0.25).stroke(Role::lit(), 4.0, 1.0))
            .build();
        let ring = stage.arena_mut().add(ring);
        // the dot now sitting at that place is the value whose product lands there
        let lands = (0..=255u8).find(|&b| mul(b, t, FOLD) == lit[0]).expect("a reshuffle");
        let useful = kit.text(
            stage,
            &format!("{} values: a useful card", (1usize << 8) - one),
            30.0,
            Palette::ink(),
            v(grid_at[0], grid_at[1] - size / 2.0 - 0.35),
        )?;
        let useless = kit.text_left(
            stage,
            &format!("{one}: useless"),
            30.0,
            Role::lit(),
            v(grid_at[0] + size / 2.0 + 0.3, hit[1]),
        )?;
        play!(stage;
            dots[lands as usize].animate().set_color(Role::lit())?,
            fade_in(stage.arena_mut(), ring, ORIGIN, 1.6)?.rt(0.6),
            fade_in(stage.arena_mut(), useless, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), useful, ORIGIN, 1.0)?.rt(0.6),
        );
        let note = precisely(
            kit,
            stage,
            &format!(
                "fix the other amounts; for a piece whose own card isn't lit, exactly one of its {} amounts puts the \
                 card in the lit part",
                1u32 << 8
            ),
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6));
        stage.clear_updaters(meter.fill, true);
        Ok(())
    }

    /// 13_5: the risk ladder: a byte spare drops eight halvings, ex-or one.
    fn ladder(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let decades = 8;
        let ladder = RiskLadder::build(kit, stage, v(-1.6, 0.35), 5.8, decades)?;
        // a faint rung per halving: a byte spare is eight of them
        let mut rungs = Vec::new();
        let mut k = 0;
        while 2f64.powi(-k) >= 10f64.powi(-(decades as i32)) {
            let r = ladder.rung(stage, 2f64.powi(-k), Palette::faint())?;
            rungs.push(r);
            k += 1;
        }
        let rungs = group_of(stage, &rungs)?;
        let byte: Vec<f64> = (0..3).map(|h| p_fail_limit(256, h)).collect();
        let ex_or: Vec<f64> = (0..3).map(|h| p_fail_limit(2, h)).collect();
        let line = Spoken::say(kit, stage, board, "13_5")?;
        fade_all(stage, 0.6)?;
        play!(stage;
            fade_in(stage.arena_mut(), ladder.group, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), rungs, ORIGIN, 1.0)?.rt(0.8),
        );
        let label = |h: usize, p: f64| {
            if h == 0 {
                format!("{}  no spare", percent(p))
            } else {
                format!("{}  {h} spare{}", percent(p), if h == 1 { "" } else { "s" })
            }
        };
        let mut marker = ladder.marker(kit, stage, byte[0], &label(0, byte[0]), Role::byte())?;
        play!(stage; fade_in(stage.arena_mut(), marker, v(0.2, 0.0), 1.0)?.rt(0.6));
        let mut ghosts = Vec::new();
        for (h, pair) in byte.windows(2).enumerate().map(|(i, w)| (i + 1, w)) {
            let next = ladder.marker(kit, stage, pair[1], &label(h, pair[1]), Role::byte())?;
            let jump = kit.text(
                stage,
                &format!("÷ {}", 1u32 << 8),
                28.0,
                Role::byte(),
                v(ladder.x + 2.9, (ladder.y(pair[0]) + ladder.y(pair[1])) / 2.0),
            )?;
            let ghost = stage.copy_family(marker)?;
            stage.add_to_scene(ghost)?;
            stage.set_opacity(ghost, 0.6, true);
            ghosts.push(ghost);
            play!(stage;
                fmn::animation::replacement_transform(marker, next).rt(1.0),
                fade_in(stage.arena_mut(), jump, ORIGIN, 1.0)?.rt(0.6),
            );
            ghosts.push(jump);
            marker = next;
        }
        line.at(stage, 1)?;
        // ex-or: one halving a spare, on the same ladder
        let mut blue = ladder.marker(kit, stage, ex_or[0], &label(0, ex_or[0]), Role::ex_or())?;
        play!(stage; fade_in(stage.arena_mut(), blue, v(0.2, 0.0), 1.0)?.rt(0.5));
        for (h, &p) in ex_or.iter().enumerate().skip(1) {
            let next = ladder.marker(kit, stage, p, &label(h, p), Role::ex_or())?;
            play!(stage; fmn::animation::replacement_transform(blue, next).rt(0.8));
            blue = next;
        }
        let note = precisely(
            kit,
            stage,
            &format!(
                "model, many pieces, exact: {}, {}, {} with 0, 1, 2 byte spares; the union bound 1/255 = {} then \
                 ÷ 256 a spare",
                percent_exact(byte[0]),
                percent_exact(byte[1]),
                percent_exact(byte[2]),
                percent_exact(1.0 / 255.0)
            ),
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6));
        Ok(())
    }

    /// 13_6: the race on ten pieces, a thousand times a side.
    fn race(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let line = Spoken::say(kit, stage, board, "13_6")?;
        fade_all(stage, 0.6)?;
        let race = race();
        let sides: [(&str, Srgb, f64, Rc<Vec<RaceRun>>); 2] = [
            ("ex-or cards", Role::ex_or(), -3.5, Rc::new(race.ex_or.clone())),
            ("byte cards", Role::byte(), 3.5, Rc::new(race.byte.clone())),
        ];
        let top = race
            .ex_or
            .iter()
            .chain(&race.byte)
            .map(RaceRun::spares)
            .max()
            .unwrap_or(0);
        let runs = stage.add_value_tracker(0.0);
        let mut keep = Vec::new();
        let mut live = Vec::new();
        for (title, color, x, data) in &sides {
            let head = kit.text(stage, title, 36.0, *color, v(*x, 3.3))?;
            let frame = Frame2 {
                center: v(*x, 0.3),
                width: 5.6,
                height: 2.7,
                x: (-0.7, top as f64 + 0.7),
                y: (0.0, RACES as f64),
            };
            let axis = Line::new(frame.p(-0.7, 0.0), frame.p(top as f64 + 0.7, 0.0))
                .style(line_style(Palette::dim(), 2.0))
                .build()?;
            let axis = stage.arena_mut().add(axis);
            let mut ticks = Vec::new();
            for s in (0..=top).step_by(5) {
                let p = frame.p(s as f64, 0.0);
                ticks.push(kit.text(stage, &s.to_string(), 22.0, Palette::dim(), v(p[0], p[1] - 0.25))?);
            }
            ticks.push(kit.text(
                stage,
                "spares needed",
                24.0,
                Palette::dim(),
                v(*x, frame.bottom() - 0.6),
            )?);
            let ticks = group_of(stage, &ticks)?;
            play!(stage;
                fade_in(stage.arena_mut(), head, ORIGIN, 1.0)?.rt(0.5),
                fade_in(stage.arena_mut(), axis, ORIGIN, 1.0)?.rt(0.5),
                fade_in(stage.arena_mut(), ticks, ORIGIN, 1.0)?.rt(0.5),
            );
            keep.extend([head, axis, ticks]);
            // the histogram of the races run so far, redrawn from the tracker
            let (data, color) = (Rc::clone(data), *color);
            let bars = stage.always_redraw(move |s| {
                let n = (s.tracker_value(runs).unwrap_or(0.0).round() as usize).min(data.len());
                let mut counts = vec![0usize; top + 1];
                for r in &data[..n] {
                    counts[r.spares()] += 1;
                }
                let w = 0.72 * frame.width / (frame.x.1 - frame.x.0);
                let rects: Vec<VMobject> = counts
                    .iter()
                    .enumerate()
                    .filter(|&(_, &c)| c > 0)
                    .map(|(sp, &c)| {
                        let base = frame.p(sp as f64, 0.0);
                        let h = (frame.p(sp as f64, c as f64)[1] - base[1]).max(0.004);
                        explainer_kit::rect(v(base[0] - w / 2.0, base[1]), w, h, flat(color, 0.85))
                            .expect("positive extents")
                    })
                    .collect();
                s.add(v_group(rects))
            });
            stage.add_to_scene(bars)?;
            live.push(bars);
        }
        let count = kit.live_count(stage, runs, 34.0, Palette::ink(), v(0.0, 2.55))?;
        let of = kit.text(
            stage,
            &format!("races of {}", thousands(RACES as u64)),
            22.0,
            Palette::dim(),
            v(0.0, 2.15),
        )?;
        live.push(count);
        play!(stage; fade_in(stage.arena_mut(), of, ORIGIN, 1.0)?.rt(0.4));
        line.at(stage, 1)?;
        let go = runs
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(5.0),
                rate_func: Some(fmn::core::rate::rush_into),
                ..AnimateArgs::default()
            })?
            .set_value(RACES as f64)?;
        stage.play(go)?;
        for m in live {
            stage.clear_updaters(m, true);
            keep.push(m);
        }
        line.at(stage, 2)?;
        let m_x = mean_spares(&race.ex_or);
        let m_b = mean_spares(&race.byte);
        let mean_x = kit.text(stage, &format!("mean {m_x} spares"), 32.0, Role::ex_or(), v(-3.5, -2.1))?;
        play!(stage; fade_in(stage.arena_mut(), mean_x, ORIGIN, 1.0)?.rt(0.5));
        line.at(stage, 3)?;
        let mean_b = kit.text(stage, &format!("mean {m_b} spares"), 32.0, Role::byte(), v(3.5, -2.1))?;
        let exact = kit.text(
            stage,
            &format!(
                "exact, ten pieces: {:.4} / {:.4}",
                exact_mean_spares(1, RACE_PIECES as u32),
                exact_mean_spares(8, RACE_PIECES as u32)
            ),
            28.0,
            Palette::dim(),
            v(0.0, -2.52),
        )?;
        let terms: Vec<String> = (1..=4).map(|i| format!("1/{}", (1u32 << i) - 1)).collect();
        let note = lower_third(
            kit,
            stage,
            "precisely:",
            &format!(
                "with many pieces, 1 + {} + … = {:.4}…; with ten pieces the sum stops at 1/{}: {:.4}. Erdős proved \
                 the full sum irrational in {}",
                terms[1..].join(" + "),
                exact_mean_spares(1, 100),
                (1u32 << RACE_PIECES) - 1,
                exact_mean_spares(1, RACE_PIECES as u32),
                crate::decoding::ERDOS_YEAR
            ),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), mean_b, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), exact, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6),
        );
        Ok(())
    }

    /// 13_7: a 0/1 card is an ex-or card in either field.
    fn zeros_and_ones(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let ring = RingProp::build(kit, stage, v(-3.3, 0.1), 2.45, 0.6, false)?;
        let rows: Vec<Vec<u8>> = RING
            .iter()
            .map(|&c| (0..5).map(|p| u8::from(c >> p & 1 == 1)).collect())
            .collect();
        let (r2, r256) = (rank_gf2(&RING), rank_gf256(&rows));
        let n = RING.len();
        let in_gf2 = kit.text(
            stage,
            &format!("as ex-or cards: {r2} of {n} useful"),
            30.0,
            Role::ex_or(),
            v(3.4, 2.6),
        )?;
        let in_gf256 = kit.text(
            stage,
            &format!("in the byte field: {r256} of {n} useful"),
            30.0,
            Role::byte(),
            v(3.4, 1.95),
        )?;
        let same = kit.text(
            stage,
            "the same number of useful cards",
            30.0,
            Palette::ink(),
            v(3.4, 1.3),
        )?;
        let (grid, _) = byte_grid(stage, v(3.4, -0.95), 2.9, |b| {
            if b <= 1 { Role::ex_or() } else { Role::byte() }
        })?;
        let others = kit.text(
            stage,
            &format!("the gain: the other {} amounts", (1u32 << 8) - 2),
            28.0,
            Role::byte(),
            v(3.4, -2.75),
        )?;
        let line = Spoken::say(kit, stage, board, "13_7")?;
        fade_all(stage, 0.6)?;
        play!(stage; fade_in(stage.arena_mut(), ring.group, ORIGIN, 1.0)?.rt(0.8));
        line.at(stage, 1)?;
        play!(stage; fade_in(stage.arena_mut(), in_gf2, ORIGIN, 1.0)?.rt(0.5));
        play!(stage;
            fade_in(stage.arena_mut(), in_gf256, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), same, ORIGIN, 1.0)?.rt(0.5),
        );
        // so our ring of five is still stuck (R3 Maya maya-16)
        line.at(stage, 2)?;
        play!(stage;
            fmn::animation::indicate(stage.arena_mut(), ring.group, 1.06, None)?.rt(0.9),
            fmn::animation::indicate(stage.arena_mut(), in_gf256, 1.15, None)?.rt(0.9),
        );
        line.at(stage, 3)?;
        play!(stage;
            fade_in(stage.arena_mut(), grid, ORIGIN, 1.0)?.rt(0.8).lag(0.003),
            fade_in(stage.arena_mut(), others, ORIGIN, 1.0)?.rt(0.6),
        );
        kit.hold(stage)?;
        Ok(())
    }
}

/// A small chance in percent to five significant digits ("0.39215%").
fn percent_exact(p: f64) -> String {
    format!("{}%", sig_digits(100.0 * p, 5))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoding::histogram;

    #[test]
    fn numbers_print_as_the_storyboard_writes_them() {
        assert_eq!(percent(p_fail_limit(256, 0)), "0.39%");
        assert_eq!(percent(p_fail_limit(256, 1)), "0.0015%");
        assert_eq!(percent(p_fail_limit(256, 2)), "0.000006%");
        assert_eq!(percent(p_fail_limit(2, 0)), "71%");
        assert_eq!(one_in(useless_chance(SLOTS - 1, SLOTS)), "1/2");
        assert_eq!(one_in(1.0 - p_full(256, 1, 0)), "1/256");
        assert_eq!(format!("{:.4}", exact_mean_spares(1, 10)), "1.6057");
        assert_eq!(format!("{:.4}", exact_mean_spares(8, 10)), "0.0039");
        assert_eq!(format!("{:.4}", exact_mean_spares(1, 100)), "1.6067");
    }

    /// Narration truth for 13 on the data the chapter draws.
    #[test]
    fn the_voice_of_13_matches_its_data() {
        // 13_2: "a random ex-or card was useless half the time"
        assert_eq!(useless_chance(SLOTS - 1, SLOTS), 0.5);
        // 13_4: "Only one time in two fifty-six"; "exactly one choice of that amount"
        assert_eq!(1.0 / (1.0 - p_full(256, 1, 0)), 256.0);
        assert_eq!(lit_values(3, 7).len(), 1);
        // 13_5: "every extra byte card divides the risk by two fifty-six" (the
        // exact ratios are within 0.2% of 256); "each extra card only roughly halves it"
        for h in 0..2 {
            let r = p_fail_limit(256, h) / p_fail_limit(256, h + 1);
            assert!((r / 256.0 - 1.0).abs() < 2e-3, "{r}");
            let x = p_fail_limit(2, h) / p_fail_limit(2, h + 1);
            assert!((1.5..2.0).contains(&x), "{x}");
        }
        // 13_6: "Ex-or needs one or two spares on average. Byte cards rarely need even one."
        let race = race();
        let m = mean_spares(&race.ex_or);
        assert!((1.0..2.0).contains(&m), "{m}");
        let needing = race.byte.iter().filter(|r| r.spares() >= 1).count();
        assert!(needing * 100 < race.byte.len(), "rarely: {needing}");
        assert_eq!(histogram(&race.byte).len(), 2);
        // 13_7: "still an ex-or card, even in the byte field"; "the other two fifty-four"
        let rows: Vec<Vec<u8>> = RING
            .iter()
            .map(|&c| (0..5).map(|p| u8::from(c >> p & 1 == 1)).collect())
            .collect();
        assert_eq!(rank_gf256(&rows), rank_gf2(&RING));
        assert_eq!((1u32 << 8) - 2, 254);
    }
}
