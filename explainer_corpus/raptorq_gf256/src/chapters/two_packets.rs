//! Chapter 17, two packets: the video's ending. A real RaptorQ decoder, a
//! million times at ten pieces: 6,316 failures with exactly enough packets,
//! on the risk ladder (17_1); one more ex-or packet leaves a byte card spare
//! (17_2); 24 with one extra, 1 with two (17_3); the standard's promise
//! beside them (17_4); the originals go first (17_5); where it is used
//! (17_6); and the loop of the hook closes: the phones turn green, every
//! byte's partner on the byte clock, the fold that refuses to factor, and
//! the chalk's 3 × 7 = 9 (17_7), then the receipts.

use explainer_kit::props::{lower_third, names_to_search, precisely, predict_pause, receipts_card};
use explainer_kit::{Kit, Palette, Timed, children, fade_all, play, thousands, tween, v};
use fmn::prelude::*;

use super::byte_clock::ByteClock;
use super::draw::{Role, card_fill, group_of, line_style, packet, phone, tower, tower_tip};
use super::fold::prime_polynomial;
use super::hook::{chalk, field, sweep};
use super::props::{BucketProp, RiskLadder};
use super::voice::{Spoken, board};
use crate::decoding::{
    ATSC_A331_YEAR, LT_CODES_YEAR, MEASURED_K10, MEASURED_LARGER, MEASURED_STREAM, RFC_6330_YEAR, RFC_ONE_IN,
    STREAM_PIECES, TEN, clopper_pearson,
};

const KEY: &str = "17_two_packets";

/// The ladder: its centre, height and decades (to 1 in 10⁸, so 17_3's
/// interval fits).
const LADDER_AT: Vec3 = [-2.4, 0.45, 0.0];
const LADDER_H: f64 = 5.4;
const DECADES: u32 = 8;

/// `n` per million, as printed: "6,316 per million".
fn per_million(fails: u32, trials: u32) -> String {
    let m = f64::from(fails) * 1e6 / f64::from(trials);
    if (m - m.round()).abs() < 1e-9 {
        format!("{} per million", thousands(m.round() as u64))
    } else {
        format!("{m} per million")
    }
}

/// What 17 prints from the measured runs; facts.rs registers each, and
/// `./x raptorq_gf256 facts-check` holds them to the oracle's roundings.
pub struct Shown {
    /// 17_1's caption: the exact 95% interval with exactly enough packets
    /// at ten pieces, in percent ("0.616", "0.647").
    pub h0_interval_percent: [String; 2],
    /// 17_3: the exact 95% interval of the one failure with two extra, per
    /// million ("0.025", "5.6").
    pub h2_interval_per_million: [String; 2],
    /// 17_1's caption: the failure rate at the larger blocks, in percent.
    pub larger_percent: [String; 2],
    /// 17_3's lower-third: 6,316 ÷ 256 ÷ 256 failures expected in a million
    /// with two extra, and the chance of one or more, in whole percent.
    pub h2_predicted: String,
    pub h2_at_least_one_percent: String,
}

pub fn shown() -> Shown {
    use super::byte_lottery::sig_digits;
    let [(f0, n0), _, (f2, n2)] = MEASURED_K10;
    let (lo0, hi0) = clopper_pearson(f0, n0);
    let (lo2, hi2) = clopper_pearson(f2, n2);
    let predicted = f64::from(f0) / 65_536.0;
    Shown {
        h0_interval_percent: [sig_digits(100.0 * lo0, 3), sig_digits(100.0 * hi0, 3)],
        h2_interval_per_million: [sig_digits(1e6 * lo2, 2), sig_digits(1e6 * hi2, 2)],
        larger_percent: MEASURED_LARGER
            .map(|(_, fails, trials)| sig_digits(100.0 * f64::from(fails) / f64::from(trials), 2)),
        h2_predicted: sig_digits(predicted, 1),
        h2_at_least_one_percent: format!("{:.0}", -100.0 * (-predicted).exp_m1()),
    }
}

/// A card of text, `w` wide, centred at `at`: a title and a dim line.
fn text_card(kit: &Kit, stage: &mut Stage<'_>, title: &str, line: &str, w: f64, at: Vec3) -> fmn::Result<Mob> {
    let back = Rectangle::new()
        .width(w)
        .height(1.5)
        .corner_radius(0.14)
        .style(Style::default().fill(card_fill(), 0.96).stroke(Role::card(), 3.0, 1.0))
        .build()?
        .moved_to(at);
    let back = stage.arena_mut().add(back);
    let t = kit.text(stage, title, 38.0, Palette::ink(), v(at[0], at[1] + 0.25))?;
    let l = kit.text(stage, line, 26.0, Palette::dim(), v(at[0], at[1] - 0.35))?;
    group_of(stage, &[back, t, l])
}

pub struct TwoPackets {
    pub kit: Kit,
}

impl SceneConstruct for TwoPackets {
    fn name(&self) -> &str {
        KEY
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let board = board(KEY)?;
        self.measured(&kit, stage, &board)?;
        self.originals(&kit, stage, &board)?;
        self.coda(&kit, stage, &board)?;
        fade_all(stage, 0.6)?;
        kit.finish(stage)
    }
}

impl TwoPackets {
    /// 17_1–17_4: the measured decoder on the risk ladder, the spare byte
    /// card, and the standard's promise.
    fn measured(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let [(f0, n0), (f1, n1), (f2, n2)] = MEASURED_K10;
        let ladder = RiskLadder::build(kit, stage, LADDER_AT, LADDER_H, DECADES)?;
        // the ladder's last label says "model"; this one carries measurements
        if let Some(&model) = children(stage, ladder.group).last() {
            stage.detach(ladder.group, model);
        }
        let measured = kit.text_styled(
            stage,
            "measured",
            24.0,
            Palette::dim(),
            v(ladder.x, ladder.top + 0.4),
            false,
            true,
        )?;
        stage.attach(ladder.group, measured)?;
        let header = kit.text(
            stage,
            "a real RaptorQ decoder, ten pieces",
            30.0,
            Palette::ink(),
            v(3.9, 3.1),
        )?;
        let decodes_word = kit.text(stage, "decodes", 28.0, Palette::dim(), v(3.9, 0.0))?;
        let failed_word = kit.text(stage, "failed", 28.0, Role::lost(), v(3.9, -1.75))?;
        let runs = stage.add_value_tracker(0.0);
        let fails = stage.add_value_tracker(0.0);
        let shown = shown();
        let [(k1, _, t1), (k2, _, t2)] = MEASURED_LARGER;
        let caption = lower_third(
            kit,
            stage,
            "measured:",
            &format!(
                "raptorq 2.0.0 (Rust); 95% interval {}%; also {}% at {k1} pieces, {}% at {} ({} / {} trials)",
                shown.h0_interval_percent.join("–"),
                shown.larger_percent[0],
                shown.larger_percent[1],
                thousands(u64::from(k2)),
                thousands(u64::from(t1)),
                thousands(u64::from(t2)),
            ),
        )?;

        // 17_1: a million decodes, 6,316 failures
        let line = Spoken::say(kit, stage, board, "17_1")?;
        fade_all(stage, 0.6)?;
        play!(stage;
            fade_in(stage.arena_mut(), ladder.group, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), header, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), decodes_word, ORIGIN, 1.0)?.rt(0.8),
        );
        let live_runs = kit.live_count(stage, runs, 64.0, Palette::ink(), v(3.9, 0.75))?;
        stage.play(tween(runs, f64::from(n0), 2.4)?)?;
        line.at(stage, 1)?;
        play!(stage; fade_in(stage.arena_mut(), failed_word, ORIGIN, 1.0)?.rt(0.4));
        let live_fails = kit.live_count(stage, fails, 64.0, Role::lost(), v(3.9, -1.0))?;
        stage.play(tween(fails, f64::from(f0), 1.8)?)?;
        line.at(stage, 2)?;
        let p0 = f64::from(f0) / f64::from(n0);
        let m0 = ladder.marker(
            kit,
            stage,
            p0,
            &format!("exactly enough: {}", per_million(f0, n0)),
            Role::lost(),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), m0, v(0.2, 0.0), 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), caption, v(0.0, 0.2), 1.0)?.rt(0.6),
        );
        stage.clear_updaters(live_runs, true);
        stage.clear_updaters(live_fails, true);

        // 17_2: one more ex-or packet fills one more slot; a byte card is spare
        let line = Spoken::say(kit, stage, board, "17_2")?;
        let b = TEN;
        let bucket = BucketProp::build(stage, b.l(), v(4.3, -0.3), 1.4, 4.1)?;
        let mut blue = Vec::new();
        for i in 0..b.ex_or_slots() {
            blue.push(bucket.water(stage, i, Role::ex_or())?);
        }
        let mut gold = Vec::new();
        for i in b.ex_or_slots()..b.l() {
            gold.push(bucket.water(stage, i, Role::byte())?);
        }
        let blue_g = group_of(stage, &blue)?;
        let gold_g = group_of(stage, &gold)?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in [header, decodes_word, failed_word, live_runs, live_fails] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        for m in [bucket.group, blue_g, gold_g] {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.7))?);
        }
        stage.play_prepared(anims)?;
        // the extra ex-or card takes the first gold slot; every gold card
        // moves up one, and the top one has no slot left: spare
        let pitch = bucket.slot_centre(1)[1] - bucket.slot_centre(0)[1];
        let extra = bucket.water(stage, b.ex_or_slots(), Role::ex_or())?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &g in &gold {
            let a = g
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.9),
                    ..AnimateArgs::default()
                })?
                .shift([0.0, pitch, 0.0])?;
            anims.push(stage.prepare(a)?);
        }
        let a = fade_in(stage.arena_mut(), extra, v(0.0, -2.0), 1.0)?;
        anims.push(stage.prepare(a.rt(0.9))?);
        stage.play_prepared(anims)?;
        let top = *gold.last().expect("byte slots");
        let lift = top
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.6),
                ..AnimateArgs::default()
            })?
            .shift([0.0, 0.45, 0.0])?;
        stage.play(lift)?;
        let spare_at = stage.get_center(top);
        let spare = kit.text_left(stage, "spare", 30.0, Role::byte(), v(spare_at[0] + 0.95, spare_at[1]))?;
        play!(stage;
            fade_in(stage.arena_mut(), spare, ORIGIN, 1.0)?.rt(0.4),
            fmn::animation::indicate(stage.arena_mut(), top, 1.3, Some([1.0, 0.85, 0.4]))?.rt(0.8),
        );
        line.at_phrase(stage, "and that divides")?;
        // a spare byte card divides the risk by about 256: an arrow down the ladder
        let guess = p0 / 256.0;
        let arrow = Arrow::new(
            v(ladder.x + 0.35, ladder.y(p0) - 0.15),
            v(ladder.x + 0.35, ladder.y(guess) + 0.15),
        )
        .buff(0.0)
        .thickness(3.0)
        .style(line_style(Role::byte(), 3.0))
        .build()?;
        let arrow = stage.arena_mut().add(arrow);
        let div = kit.text_left(
            stage,
            "÷ 256",
            28.0,
            Role::byte(),
            v(ladder.x + 0.6, (ladder.y(p0) + ladder.y(guess)) / 2.0),
        )?;
        play!(stage;
            fmn::animation::grow_arrow(stage.arena_mut(), arrow)?.rt(0.7),
            fade_in(stage.arena_mut(), div, ORIGIN, 1.0)?.rt(0.5),
        );
        let q = predict_pause(kit, stage, v(ladder.x + 2.6, (ladder.y(p0) + ladder.y(guess)) / 2.0))?;

        // 17_3: twenty-four; two extra, just once
        let line = Spoken::say(kit, stage, board, "17_3")?;
        let p1 = f64::from(f1) / f64::from(n1);
        let p2 = f64::from(f2) / f64::from(n2);
        let m1 = ladder.marker(
            kit,
            stage,
            p1,
            &format!("one extra: {}", per_million(f1, n1)),
            Role::lost(),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            fade_out(stage.arena_mut(), div, ORIGIN, 1.0)?.rt(0.3),
            fade_in(stage.arena_mut(), m1, v(0.2, 0.0), 1.0)?.rt(0.5),
        );
        line.at(stage, 1)?;
        let m2 = ladder.marker(
            kit,
            stage,
            p2,
            &format!("two extra: {}", per_million(f2, n2)),
            Role::lost(),
        )?;
        play!(stage; fade_in(stage.arena_mut(), m2, v(0.2, 0.0), 1.0)?.rt(0.5));
        line.at(stage, 2)?;
        // the 95% interval of one failure in a million: two decades wide
        let (lo2, hi2) = clopper_pearson(f2, n2);
        let wx = ladder.x + 0.15;
        let whisker = v_group([
            Line::new(v(wx, ladder.y(lo2)), v(wx, ladder.y(hi2)))
                .style(line_style(Role::lost(), 4.0))
                .build()?,
            Line::new(v(wx - 0.12, ladder.y(lo2)), v(wx + 0.12, ladder.y(lo2)))
                .style(line_style(Role::lost(), 4.0))
                .build()?,
            Line::new(v(wx - 0.12, ladder.y(hi2)), v(wx + 0.12, ladder.y(hi2)))
                .style(line_style(Role::lost(), 4.0))
                .build()?,
        ]);
        let whisker = stage.arena_mut().add(whisker);
        let range = kit.text_left(
            stage,
            &format!(
                "95% interval: {} to {} per million",
                shown.h2_interval_per_million[0], shown.h2_interval_per_million[1]
            ),
            26.0,
            Role::lost(),
            v(ladder.x + 0.65, ladder.y(p2) - 0.42),
        )?;
        let note = lower_third(
            kit,
            stage,
            "precisely:",
            &format!(
                "÷256 twice predicts ≈ {} failures here; one or more happens ≈ {}% of the time",
                shown.h2_predicted, shown.h2_at_least_one_percent
            ),
        )?;
        play!(stage;
            show_creation(whisker).rt(0.6),
            fade_in(stage.arena_mut(), range, ORIGIN, 1.0)?.rt(0.6),
            fade_out(stage.arena_mut(), caption, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6),
        );
        // an unlucky phone just waits for one more packet
        line.at(stage, 3)?;
        let waiting = stage.arena_mut().add(phone(Role::waiting(), 1.3).moved_to(v(4.4, 0.0)));
        let late = stage.arena_mut().add(packet(0.36, Role::ex_or()).moved_to(v(6.3, 2.4)));
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in [bucket.group, blue_g, gold_g, extra, spare] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        let a = fade_in(stage.arena_mut(), waiting, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(0.6))?);
        stage.play_prepared(anims)?;
        play!(stage; fade_in(stage.arena_mut(), late, ORIGIN, 1.0)?.rt(0.3));
        let arrive = late
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(0.8),
                ..AnimateArgs::default()
            })?
            .move_to(v(4.4, 0.0), ORIGIN)?;
        stage.play(arrive)?;
        play!(stage;
            fade_out(stage.arena_mut(), late, ORIGIN, 1.0)?.rt(0.3),
            waiting.animate().set_color(Role::done())?,
        );

        // 17_4: the standard's promise, beside its packet counts
        let line = Spoken::say(kit, stage, board, "17_4")?;
        let promise = kit.text(
            stage,
            "the standard: at most",
            28.0,
            Role::byte(),
            v(ladder.x - 3.0, ladder.top + 0.4),
        )?;
        play!(stage; fade_in(stage.arena_mut(), promise, ORIGIN, 1.0)?.rt(0.5));
        let tags = ["exactly\nenough", "one\nextra", "two\nextra"];
        for (k, (&one_in, tag)) in RFC_ONE_IN.iter().zip(tags).enumerate() {
            match k {
                1 => line.at(stage, 1)?,
                2 => line.at_phrase(stage, "and with two")?,
                _ => {}
            }
            let p = 1.0 / f64::from(one_in);
            let rung = ladder.rung(stage, p, Role::byte())?;
            let t = kit.text(stage, tag, 44.0, Role::byte(), ORIGIN)?;
            stage.move_to(t, v(ladder.x - 3.2, ladder.y(p)), RIGHT);
            play!(stage;
                show_creation(rung).rt(0.5),
                fade_in(stage.arena_mut(), t, v(0.15, 0.0), 1.0)?.rt(0.5),
            );
        }
        // the standard's average is not the hook's stream; that one was
        // measured too (R3 Ruth R3-09, Dev D7)
        let (f2s, phones) = MEASURED_STREAM[2];
        let note2 = precisely(
            kit,
            stage,
            &format!(
                "RFC 6330 §5.8: averaged over randomly chosen packet IDs, for each padded block size K′. The hook's \
                 stream, originals first with 1 packet in {} lost, measured with this decoder at {} pieces: {} \
                 failures in {} phones with two extra",
                crate::model::LOSS_ONE_IN,
                thousands(u64::from(STREAM_PIECES)),
                f2s,
                thousands(u64::from(phones)),
            ),
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), note2, v(0.0, 0.2), 1.0)?.rt(0.6),
        );
        Ok(())
    }

    /// 17_5–17_6: the originals go first; where it is used.
    fn originals(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let line = Spoken::say(kit, stage, board, "17_5")?;
        fade_all(stage, 0.6)?;
        let base = v(-5.4, -2.2);
        let mast = stage.arena_mut().add(tower(3.2, base, Palette::dim())?);
        let tip = tower_tip(base, 3.2);
        let handset = stage
            .arena_mut()
            .add(phone(Role::waiting(), 1.4).moved_to(v(5.2, -0.4)));
        play!(stage;
            show_creation(mast).rt(0.6),
            fade_in(stage.arena_mut(), handset, ORIGIN, 1.0)?.rt(0.6),
        );
        // the originals, then repair packets: a row flying from the tower
        let n = TEN.k;
        let mut flights = Vec::new();
        for i in 0..n + 4 {
            let color = if i < n { Role::card() } else { Role::ex_or() };
            let p = stage.arena_mut().add(packet(0.36, color).moved_to(tip));
            flights.push(p);
        }
        let originals = kit.text(stage, "the originals, unchanged", 30.0, Role::card(), v(-0.4, 1.9))?;
        let repair = kit.text(stage, "then repair packets", 30.0, Role::ex_or(), v(-0.4, 1.35))?;
        play!(stage; fade_in(stage.arena_mut(), originals, ORIGIN, 1.0)?.rt(0.5));
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (i, &p) in flights.iter().enumerate() {
            let to = v(-3.4 + 0.62 * i as f64, 0.6);
            let a = fade_in(stage.arena_mut(), p, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.2))?);
            let a = p
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.8),
                    ..AnimateArgs::default()
                })?
                .move_to(to, ORIGIN)?;
            anims.push(stage.prepare(a)?);
        }
        let flight = explainer_kit::lagged(stage, anims, 0.05)?;
        stage.play(flight)?;
        play!(stage; fade_in(stage.arena_mut(), repair, ORIGIN, 1.0)?.rt(0.5));
        line.at_phrase(stage, "so a phone")?;
        let clean = kit.text(
            stage,
            "a clean connection: nothing to decode",
            30.0,
            Role::done(),
            v(1.6, -2.6),
        )?;
        play!(stage;
            fmn::animation::indicate(stage.arena_mut(), handset, 1.1, Some([0.51, 0.76, 0.40]))?.rt(0.6),
            handset.animate().set_color(Role::done())?,
            fade_in(stage.arena_mut(), clean, ORIGIN, 1.0)?.rt(0.6),
        );

        // 17_6: where it is used (text cards, no logos)
        let line = Spoken::say(kit, stage, board, "17_6")?;
        fade_all(stage, 0.6)?;
        let cards = [
            (
                "ATSC 3.0".to_owned(),
                format!("next-generation TV; A/331:{ATSC_A331_YEAR}, FEC optional"),
                v(-3.5, 1.3),
            ),
            (
                "Linux RAID-6".to_owned(),
                "the same byte field: survives two dead drives".to_owned(),
                v(3.5, 1.3),
            ),
            (
                "QR codes".to_owned(),
                "the same byte field, in every one".to_owned(),
                v(0.0, -0.9),
            ),
        ];
        let mut built = Vec::new();
        for (t, l, at) in &cards {
            built.push(text_card(kit, stage, t, l, 6.4, *at)?);
        }
        // the credits, as RFC 6330 cites them (header and §9.2)
        let credits = lower_third(
            kit,
            stage,
            "credits:",
            &format!(
                "LT codes: Luby, {LT_CODES_YEAR} · Raptor codes: Shokrollahi and Luby · RFC 6330 ({RFC_6330_YEAR}): \
                 Luby, Shokrollahi, Watson, Stockhammer, Minder"
            ),
        )?;
        for (k, &c) in built.iter().enumerate() {
            match k {
                1 => line.at(stage, 1)?,
                2 => line.at_phrase(stage, "and every QR code")?,
                _ => {}
            }
            play!(stage; fade_in(stage.arena_mut(), c, v(0.0, -0.2), 1.0)?.rt(0.5));
            if k == 1 {
                play!(stage; fade_in(stage.arena_mut(), credits, v(0.0, 0.2), 1.0)?.rt(0.6));
            }
        }
        Ok(())
    }

    /// 17_7: the loop of the hook closes, then the receipts.
    fn coda(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let line = Spoken::say(kit, stage, board, "17_7")?;
        fade_all(stage, 0.6)?;
        // two extra packets: the hook's phones, every one holding the picture
        let phones = field(stage);
        play!(stage; fade_in(stage.arena_mut(), phones.group, ORIGIN, 1.0)?.rt(0.8).lag(0.0008));
        sweep(stage, &phones.bodies, Role::done(), 1.6)?;
        // because every nonzero byte has a partner: the byte clock's mirror pairs
        line.at_phrase(stage, "because every nonzero")?;
        let clock = ByteClock::build(stage, v(-4.2, 0.0), 1.8)?;
        let pairs = clock.mirror_pairs(stage, Role::byte())?;
        play!(stage;
            fade_out(stage.arena_mut(), phones.group, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), clock.group, ORIGIN, 1.0)?.rt(0.8),
        );
        play!(stage; show_creation(pairs).rt(1.4));
        // and that's because a polynomial refuses to factor
        line.at(stage, 1)?;
        let fold = prime_polynomial(kit, stage, v(2.1, 1.9))?;
        let names = names_to_search(
            kit,
            stage,
            &[
                "fountain code",
                "LT code",
                "Raptor",
                "RaptorQ",
                "RFC 6330",
                "inactivation decoding",
                "GF(256)",
                "carry-less multiplication",
                "irreducible polynomial",
                "primitive polynomial",
                "discrete logarithm",
                "Gaussian elimination",
            ],
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), fold, v(0.0, -0.15), 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), names, v(0.0, 0.2), 1.0)?.rt(0.8),
        );
        // three times seven is nine
        line.at(stage, 2)?;
        let title_equation = chalk(kit, stage, 150.0, v(2.1, -0.5))?;
        play!(stage; write(stage.arena(), title_equation).rt(1.2));
        kit.hold(stage)?;
        // the end card: the reason and the payoff stay, the receipts come in
        // under them, and the motif resolves over all three (the score's 17_7
        // listen window)
        let receipts = receipts_card(
            kit,
            stage,
            &[
                "every number: computed in facts.rs, checked against an independent oracle.py",
                "seeded runs (the race, the peeling) replay the oracle's own random draws",
                "measured: the raptorq 2.0.0 decoder (Rust), recorded runs labelled as such",
                "the standard's promise: RFC 6330 §5.8",
            ],
        )?;
        stage.move_to(receipts, v(0.0, -2.25), ORIGIN);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in [clock.group, pairs, names] {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.6))?);
        }
        for (m, to) in [(fold, v(0.0, 2.75)), (title_equation, v(0.0, 1.0))] {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.6),
                    ..AnimateArgs::default()
                })?
                .move_to(to, ORIGIN)?;
            anims.push(stage.prepare(a)?);
        }
        let a = fade_in(stage.arena_mut(), receipts, v(0.0, 0.2), 1.0)?;
        anims.push(stage.prepare(a.rt(0.6))?);
        stage.play_prepared(anims)?;
        kit.listen(stage, 3.0, "17_7")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FOLD, mul};

    /// Narration truth for 17 on the data the chapter draws.
    #[test]
    fn the_voice_of_17_matches_its_data() {
        let [(f0, n0), (f1, _), (f2, _)] = MEASURED_K10;
        // 17_1: "a million times, on small blocks of ten pieces"; "six
        // thousand three hundred sixteen"; "about six in a thousand"
        assert_eq!(n0, 1_000_000);
        assert_eq!(TEN.k, 10);
        assert_eq!(f0, 6_316);
        assert_eq!((f64::from(f0) / f64::from(n0) * 1000.0).round(), 6.0);
        // 17_2: "divides the risk by about two fifty-six" (measured 263)
        let ratio = f64::from(f0) / f64::from(f1);
        assert!((200.0..330.0).contains(&ratio), "{ratio}");
        // 17_3: "Twenty-four. Two extra, just once."
        assert_eq!((f1, f2), (24, 1));
        // "too rare to measure precisely": the interval spans more than two decades
        let (lo, hi) = clopper_pearson(f2, n0);
        assert!(hi / lo > 100.0);
        // 17_4: one in a hundred, ten thousand, a million; 17_7 "Two extra
        // packets buy the standard's one in a million"
        assert_eq!(RFC_ONE_IN, [100, 10_000, 1_000_000]);
        // 01_2 "almost every phone that catches just two packets more than it
        // needs": the hook's own stream, measured, stays under one in a million
        // with two extra, and each extra packet cuts the failures
        let [(s0, n), (s1, _), (s2, _)] = MEASURED_STREAM;
        assert!(f64::from(s2) / f64::from(n) < 1e-6 && s2 < s1 && s1 < s0);
        assert_eq!(crate::model::LOSS_ONE_IN, 100);
        // 17_7: "Three times seven is nine"
        assert_eq!(mul(3, 7, FOLD), 9);
        assert_eq!(per_million(f0, n0), "6,316 per million");
        // the printed roundings (the fact sheet's B-F22 and its 17_3 lower-third)
        let shown = shown();
        assert_eq!(shown.h0_interval_percent, ["0.616", "0.647"]);
        assert_eq!(shown.h2_interval_per_million, ["0.025", "5.6"]);
        assert_eq!(shown.larger_percent, ["0.63", "0.54"]);
        assert_eq!(
            (shown.h2_predicted.as_str(), shown.h2_at_least_one_percent.as_str()),
            ("0.1", "9")
        );
    }

    /// Every phrase a picture waits for is in its line.
    #[test]
    fn the_pictures_cue_phrases_are_spoken() {
        let board = board(KEY).expect("chapter 17");
        for (id, phrase) in [
            ("17_2", "and that divides"),
            ("17_4", "and with two"),
            ("17_5", "so a phone"),
            ("17_6", "and every QR code"),
            ("17_7", "because every nonzero"),
        ] {
            let words = board
                .beats
                .iter()
                .find(|b| b.id == id)
                .and_then(explainer_kit::storyboard::Beat::spoken)
                .expect("a spoken line");
            assert!(
                super::super::voice::phrase_share(&words, phrase).is_some(),
                "{id}: {phrase}"
            );
        }
    }
}
