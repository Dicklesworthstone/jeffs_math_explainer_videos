//! Chapter 13, proudest: the magic signs, and Lamport's own verdict on the
//! bakery's practical use (13_1); Arm's firmware takes a ticket while a core
//! wakes with its cache off (13_2); the trick of the bigger ticket and the
//! tie-breaking number reappears in Lamport's clocks and in Paxos (13_3);
//! the coda: five customers served one at a time, in the order of a real
//! run of the model, under the gold scanline, with Lamport's words, and the
//! receipts (13_4).

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{lower_third, predict_pause, receipts_card};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, thousands, v};
use fmn::prelude::*;

use super::draw::{
    badge, chip_outline, customer, fade_in_all, fade_out_all, faded_line, fail, flat, line_style, order_gold,
    quote_card, room_red, say, scanline, sign, slate, wait_until,
};
use super::glyphs;
use super::tunes;
use crate::facts::{self, Arch, cited, measured};
use crate::model::{Bakery, FIVE, Pc, State};

/// The coda's arrivals (1-based): each walks through her doorway in turn.
pub const ARRIVALS: [usize; 5] = [3, 1, 5, 2, 4];

/// One visit by each of `model`'s customers: each in `arrivals` order runs
/// her doorway alone (so she reads every ticket taken before her), then
/// everyone's waits run round robin, the next customer (cyclically) with a
/// move that is not a blocked wait taking it, until all have been in the
/// room and left. Returns the service order as (ticket, customer) pairs.
pub fn coda_service(model: &Bakery, arrivals: &[usize]) -> Result<Vec<(u32, usize)>, String> {
    let mut st: State = model.initial();
    let mut served = Vec::new();
    let mut visited = vec![false; model.n];
    let step = |st: &State, i: usize, visited: &[bool]| {
        model.successors(st).into_iter().find(|(c, lab, ns)| {
            *c == i && ns != st && !lab.ends_with(", wait") && !(visited[i] && st.local[i].pc == Pc::Ncs)
        })
    };
    for &k in arrivals {
        let i = k - 1;
        loop {
            let Some((_, _, next)) = step(&st, i, &visited) else {
                return Err(format!("customer {k} is stuck in her doorway"));
            };
            st = next;
            if st.local[i].pc == Pc::L2 {
                break;
            }
        }
    }
    let mut turn = 0;
    let mut idle = 0;
    while visited.iter().any(|v| !v) {
        let i = turn % model.n;
        turn += 1;
        let Some((_, label, next)) = step(&st, i, &visited) else {
            idle += 1;
            if idle > 4 * model.n {
                return Err("no customer can move".into());
            }
            continue;
        };
        idle = 0;
        if label.ends_with("ENTER CS") {
            served.push((next.number[i], i + 1));
        }
        if label.contains("(exit)") {
            visited[i] = true;
        }
        if model.in_room(&next).len() > 1 {
            return Err("two in the room".into());
        }
        st = next;
    }
    Ok(served)
}

pub struct Proudest {
    pub kit: Kit,
}

impl SceneConstruct for Proudest {
    fn name(&self) -> &str {
        "13_proudest"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();

        // 13_1: the magic signs, and Lamport on the bakery's practical use
        let line = say(&kit, stage, "13_1")?;
        // 02_7's magic sign, called back: it checks and flips in one motion
        let mut magic = sign(&kit, stage, false, v(-3.6, 0.6), 1.5, true)?;
        let rmw = lower_third(&kit, stage, glyphs::RMW, glyphs::RMW_TEXT)?;
        fade_in_all(stage, &magic.parts(), 0.6)?;
        play!(stage; fade_in(stage.arena_mut(), rmw, v(0.0, 0.2), 1.0)?.rt(0.7));
        line.wait_for(stage, "check and flip in one motion")?;
        magic.flip(&kit, stage, true, 0.5)?;
        line.wait_for(stage, "So does the bakery")?;
        let quote = quote_card(
            &kit,
            stage,
            glyphs::QUOTE_LITTLE_USE,
            &format!(
                "{}, {}, {}, {}{}",
                glyphs::TURING_LECTURE_SOURCE,
                cited::turing_journal(),
                cited::TURING_LECTURE_YEAR,
                glyphs::SECTION_SIGN,
                cited::PRACTICAL_USE_SECTION
            ),
            v(2.4, 1.0),
            28,
        )?;
        play!(stage; fade_in(stage.arena_mut(), quote, v(-0.2, 0.0), 1.0)?.rt(0.7));
        let q = predict_pause(&kit, stage, v(0.0, -1.3))?;

        // 13_2: Arm's own firmware takes a ticket while a core wakes
        let line = say(&kit, stage, "13_2")?;
        let mut old: Vec<Mob> = magic.parts().to_vec();
        old.extend([quote, rmw, q]);
        fade_out_all(stage, &old, 0.5)?;
        let title = kit.text(stage, glyphs::TF_A, 40.0, Palette::ink(), v(0.0, 2.3))?;
        let sub = kit.text(stage, glyphs::TF_A_WHY, 30.0, Palette::dim(), v(0.0, 1.6))?;
        play!(stage;
            fade_in(stage.arena_mut(), title, v(0.0, -0.1), 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), sub, v(0.0, -0.1), 1.0)?.rt(0.6),
        );
        line.wait_for(stage, "while a core wakes up")?;
        let core = chip_outline(&kit, stage, v(-2.6, -0.3), 2.0, glyphs::CORE)?;
        let cache = kit.text(stage, glyphs::CACHE_OFF, 26.0, room_red(), v(-2.6, -1.75))?;
        play!(stage;
            fade_in(stage.arena_mut(), core, ORIGIN, 0.9)?.rt(0.6),
            fade_in(stage.arena_mut(), cache, ORIGIN, 1.0)?.rt(0.6),
        );
        line.wait_for(stage, "the magic signs aren't guaranteed")?;
        let no_sign = kit.text(stage, glyphs::NOT_GUARANTEED, 26.0, Palette::dim(), v(2.6, 0.3))?;
        play!(stage; fade_in(stage.arena_mut(), no_sign, ORIGIN, 1.0)?.rt(0.5));
        line.wait_for(stage, "so the core takes a ticket")?;
        let ticket = slate(&kit, stage, 1, v(2.6, -0.8), 0.8)?;
        let tb = badge(&kit, stage, 1, 0.22, v(1.6, -0.8))?;
        let precisely = lower_third(&kit, stage, glyphs::PRECISELY, glyphs::TF_A_PRECISELY)?;
        play!(stage;
            fade_in(stage.arena_mut(), ticket.board, ORIGIN, 0.9)?.rt(0.5),
            write(stage.arena(), ticket.value).rt(0.6),
            fade_in(stage.arena_mut(), tb, ORIGIN, 0.9)?.rt(0.5),
            fade_in(stage.arena_mut(), precisely, v(0.0, 0.2), 1.0)?.rt(0.6),
        );
        kit.hold(stage)?;

        // 13_3: the bigger ticket and the tie-breaking number reappear: the
        // pair grid relabelled, and a timeline
        let line = say(&kit, stage, "13_3")?;
        fade_out_all(
            stage,
            &[
                title,
                sub,
                core,
                cache,
                no_sign,
                ticket.board,
                ticket.value,
                tb,
                precisely,
            ],
            0.5,
        )?;
        let mut grid = PairAxes::build(&kit, stage, v(-2.4, 0.6))?;
        fade_in_all(stage, &grid.all(), 0.7)?;
        line.wait_for(stage, "reappears in how Lamport")?;
        grid.relabel(&kit, stage)?;
        let years = [
            (cited::BAKERY_YEAR.to_string(), glyphs::THE_BAKERY),
            (cited::TIME_CLOCKS_YEAR.to_string(), glyphs::TIME_CLOCKS),
            (glyphs::PAXOS.to_owned(), glyphs::BALLOT_NUMBERS),
        ];
        let mut timeline = Vec::new();
        let axis = stage.arena_mut().add(
            Line::new(v(-5.2, -2.3), v(5.2, -2.3))
                .style(line_style(Palette::faint(), 2.5))
                .build()?,
        );
        play!(stage; show_creation(axis).rt(0.5));
        timeline.push(axis);
        for (i, (when, what)) in years.iter().enumerate() {
            if i == 2 {
                line.wait_for(stage, "Paxos")?;
            }
            let x = -4.2 + 4.2 * i as f64;
            let dot = stage
                .arena_mut()
                .add(Dot::new().point(v(x, -2.3)).radius(0.09).color(order_gold()).build());
            let top = kit.text(stage, when, 28.0, order_gold(), v(x, -1.9))?;
            let bottom = kit.text(stage, what, 24.0, Palette::dim(), v(x, -2.7))?;
            play!(stage;
                fade_in(stage.arena_mut(), dot, ORIGIN, 0.5)?.rt(0.4),
                fade_in(stage.arena_mut(), top, v(0.0, -0.1), 1.0)?.rt(0.4),
                fade_in(stage.arena_mut(), bottom, v(0.0, 0.1), 1.0)?.rt(0.4),
            );
            timeline.extend([dot, top, bottom]);
        }
        kit.hold(stage)?;

        // 13_4: the coda: five customers served one at a time, in the order
        // of a real run, the gold scanline sweeping; Lamport's words; receipts
        let order = coda_service(&FIVE, &ARRIVALS).map_err(fail)?;
        let line = say(&kit, stage, "13_4")?;
        let mut old = grid.all();
        old.extend(timeline);
        fade_out_all(stage, &old, 0.6)?;
        let ys: Vec<f64> = (0..5).map(|k| 2.2 - 0.95 * k as f64).collect();
        let mut base = Vec::new();
        for k in 1..=5 {
            let y = ys[k - 1];
            base.push(badge(&kit, stage, k, 0.2, v(-6.3, y))?);
            let ticket = order.iter().find(|(_, c)| *c == k).map_or(0, |(t, _)| *t);
            let s = slate(&kit, stage, ticket, v(-5.55, y), 0.42)?;
            base.extend(s.parts());
            base.push(
                stage.arena_mut().add(
                    Line::new(v(-4.95, y), v(6.6, y))
                        .style(faded_line(customer(k), 4.0, 0.6))
                        .build()?,
                ),
            );
        }
        fade_in_all(stage, &base, 0.7)?;
        let sweep = stage.arena_mut().add(scanline(-4.7, ys[4] - 0.45, ys[0] + 0.45)?);
        play!(stage; fade_in(stage.arena_mut(), sweep, ORIGIN, 1.0)?.rt(0.4));
        let captions = [
            (
                glyphs::QUOTE_DID_NOT_INVENT,
                glyphs::PUBLICATIONS_SOURCE_SHORT.to_owned(),
                "He discovered it",
            ),
            (
                glyphs::QUOTE_THIN_AIR,
                format!(
                    "{} ({}), {} {}",
                    glyphs::CHM_SOURCE,
                    cited::CHM_YEAR,
                    glyphs::PAGES,
                    cited::CHM_PAGES.replace('-', glyphs::EN_DASH)
                ),
                "It just seemed",
            ),
        ];
        // Lamport's words, each on its phrase
        let mut caption: Option<Mob> = None;
        for (words, source, phrase) in &captions {
            line.wait_for(stage, phrase)?;
            let c = quote_card(&kit, stage, words, source, v(0.0, -2.85), 70)?;
            let mut anims: Vec<Box<dyn Animation>> = Vec::new();
            if let Some(old) = caption.take() {
                let a = fade_out(stage.arena_mut(), old, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(0.5))?);
            }
            let a = fade_in(stage.arena_mut(), c, v(0.0, 0.15), 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
            stage.play_prepared(anims)?;
            caption = Some(c);
        }
        // then, after the voice, the coda's sound: each customer's "served" in
        // her own timbre, one at a time, in the run's service order; her room
        // band lights and the scanline reaches it as her tune starts
        let customers: Vec<usize> = order.iter().map(|&(_, k)| k).collect();
        let (sound, onsets) = tunes::turns(&customers);
        let (wav, secs) = tunes::write("13_4", sound).map_err(|e| fail(e.to_string()))?;
        let span = 11.0 / order.len() as f64;
        let mut bands = Vec::new();
        kit.sonify(stage, secs, "13_proudest:13_4", &wav)?;
        let t0 = stage.scene().time().to_f64();
        for (n, &k) in customers.iter().enumerate() {
            let x = -4.4 + span * (n as f64 + 0.5);
            let band = stage.arena_mut().add(
                Rectangle::new()
                    .width(span * 0.8)
                    .height(0.42)
                    .corner_radius(0.1)
                    .style(flat(room_red(), 0.55))
                    .build()?
                    .moved_to(v(x, ys[k - 1])),
            );
            wait_until(stage, t0 + onsets[n][0])?;
            let go = sweep
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.6),
                    ..AnimateArgs::default()
                })?
                .set_x(x + span * 0.45)?;
            play!(stage;
                go,
                fade_in(stage.arena_mut(), band, ORIGIN, 0.6)?.rt(0.6),
            );
            bands.push(band);
        }
        wait_until(stage, t0 + secs)?;
        kit.hold(stage)?;
        let (arm, x86) = (measured(Arch::Arm), measured(Arch::X86));
        let seq = arm.mode("seqcst", 2);
        let receipts = receipts_card(
            &kit,
            stage,
            &[
                &format!(
                    "{} {} {}",
                    glyphs::RECEIPT_COUNTS_A,
                    cited::BAKERY_YEAR,
                    glyphs::RECEIPT_COUNTS_B
                ),
                &format!(
                    "{} {} {}",
                    glyphs::RECEIPT_OUTBOXES,
                    facts::TSO_CHECKED_CUSTOMERS,
                    glyphs::CUSTOMERS
                ),
                &format!(
                    "{} {} {}, {} {}",
                    glyphs::RECEIPT_MEASURED,
                    seq.threads,
                    glyphs::CUSTOMERS,
                    thousands(seq.entries),
                    glyphs::VISITS_PER_LOCK
                ),
                &format!("    {}; {}", arm.cpu, x86.cpu),
                &format!(
                    "{} {}, {}; {} {};",
                    glyphs::RECEIPT_QUOTES,
                    cited::bakery_journal(),
                    cited::BAKERY_YEAR,
                    glyphs::RECEIPT_TURING,
                    cited::TURING_LECTURE_YEAR
                ),
                &format!("    {} {}", glyphs::RECEIPT_ORAL_HISTORY, cited::CHM_YEAR),
                glyphs::RECEIPT_FIRMWARE,
                glyphs::RECEIPT_CODE,
                glyphs::RECEIPT_REPO,
            ],
        )?;
        let mut away: Vec<Mob> = base.clone();
        away.extend(bands);
        away.push(sweep);
        away.extend(caption);
        fade_out_all(stage, &away, 0.6)?;
        play!(stage; fade_in(stage.arena_mut(), receipts, v(0.0, -0.1), 1.0)?.rt(0.8));
        kit.music_breath(stage, HOLD_S * 2.0)?;
        kit.hold(stage)?;
        fade_all(stage, 1.2)?;
        kit.finish(stage)
    }
}

/// 04's pair grid in miniature, for the callback: ticket across, customer
/// number up, five pairs as badges; relabelled (clock value, process).
struct PairAxes {
    x_axis: Mob,
    y_axis: Mob,
    x_label: Mob,
    y_label: Mob,
    dots: Vec<Mob>,
    origin: Vec3,
}

impl PairAxes {
    const W: f64 = 5.2;
    const H: f64 = 3.0;

    fn build(kit: &Kit, stage: &mut Stage<'_>, center: Vec3) -> fmn::Result<Self> {
        let origin = v(center[0] - Self::W / 2.0, center[1] - Self::H / 2.0);
        let x_axis = stage.arena_mut().add(
            Arrow::new(origin, v(origin[0] + Self::W, origin[1]))
                .thickness(3.0)
                .style(
                    Style::default()
                        .fill(Palette::dim(), 1.0)
                        .stroke(Palette::dim(), 3.0, 1.0),
                )
                .build()?,
        );
        let y_axis = stage.arena_mut().add(
            Arrow::new(origin, v(origin[0], origin[1] + Self::H))
                .thickness(3.0)
                .style(
                    Style::default()
                        .fill(Palette::dim(), 1.0)
                        .stroke(Palette::dim(), 3.0, 1.0),
                )
                .build()?,
        );
        let x_label = kit.text(
            stage,
            glyphs::TICKET_ACROSS,
            26.0,
            Palette::dim(),
            v(origin[0] + Self::W / 2.0, origin[1] - 0.4),
        )?;
        let y_label = kit.text(
            stage,
            glyphs::CUSTOMER_NUMBER_UP,
            26.0,
            Palette::dim(),
            v(origin[0] - 0.4, origin[1] + Self::H / 2.0),
        )?;
        stage.rotate(y_label, std::f64::consts::FRAC_PI_2, [0.0, 0.0, 1.0], None, None);
        let mut dots = Vec::new();
        for (ticket, k) in crate::facts::tie_pairs() {
            let p = v(origin[0] + 0.8 * f64::from(ticket), origin[1] + 0.5 * k as f64);
            dots.push(badge(kit, stage, k, 0.18, p)?);
        }
        Ok(Self {
            x_axis,
            y_axis,
            x_label,
            y_label,
            dots,
            origin,
        })
    }

    fn all(&self) -> Vec<Mob> {
        let mut m = vec![self.x_axis, self.y_axis, self.x_label, self.y_label];
        m.extend(&self.dots);
        m
    }

    /// The axes take the clock's names: clock value across, process up.
    fn relabel(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let o = self.origin;
        let x = kit.text(
            stage,
            glyphs::CLOCK_VALUE_ACROSS,
            26.0,
            order_gold(),
            v(o[0] + Self::W / 2.0, o[1] - 0.4),
        )?;
        let y = kit.text(
            stage,
            glyphs::PROCESS_UP,
            26.0,
            order_gold(),
            v(o[0] - 0.4, o[1] + Self::H / 2.0),
        )?;
        stage.rotate(y, std::f64::consts::FRAC_PI_2, [0.0, 0.0, 1.0], None, None);
        play!(stage;
            fmn::animation::replacement_transform(self.x_label, x).rt(0.9),
            fmn::animation::replacement_transform(self.y_label, y).rt(0.9),
        );
        // the new labels replace the old in the scene: keep them, so the
        // grid fades as a whole
        self.x_label = x;
        self.y_label = y;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_coda_plays_one_tune_at_a_time_in_service_order() {
        let order = coda_service(&FIVE, &ARRIVALS).unwrap();
        let customers: Vec<usize> = order.iter().map(|&(_, k)| k).collect();
        let (sound, onsets) = tunes::turns(&customers);
        let secs = sound.len() as f64 / explainer_score::FS;
        assert_eq!(onsets.len(), 5);
        // each tune starts after the last one's final note
        for w in onsets.windows(2) {
            assert!(w[1][0] > *w[0].last().unwrap());
        }
        // the fifth's last note sounds before the score's 0.8 s window fade
        assert!(*onsets[4].last().unwrap() < secs - 0.8);
    }

    #[test]
    fn the_coda_serves_everyone_once_lowest_ticket_first() {
        let order = coda_service(&FIVE, &ARRIVALS).unwrap();
        // tickets 1..5 in arrival order, served in that order: first come,
        // first served, the grid's sweep
        let want: Vec<(u32, usize)> = ARRIVALS.iter().enumerate().map(|(i, &k)| (i as u32 + 1, k)).collect();
        assert_eq!(order, want);
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(order, sorted);
    }
}
