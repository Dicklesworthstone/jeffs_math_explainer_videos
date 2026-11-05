//! Chapter 00, a ruler for ratios: seven A's, printed as they are read (00_1),
//! then the ordinary ruler, the low notes crowded into an eighth while one step
//! fills half (00_2), the same seven dots carried down onto a keyboard where each
//! twelve-semitone climb doubles (00_3), the half-octave prediction (00_4), the
//! square-root-of-two answer against the 660 guess (00_5), and the preview
//! of where the ruler will lose information (00_6).
//!
//! Rebuild candidate for `mx-video-program-program-87do.63`; uncompiled,
//! unrendered, unlistened. Sentence waits are by index into script.tsv.

use explainer_kit::{Kit, Palette, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    answer_c, dot, fact, fact_f64, fact_list, fade_in_all, fade_in_lagged, fade_out_all, flat, group, number_c, path,
    position_c, precisely, predict, seg, swap, title, value, warn_c,
};

pub struct RatioRuler {
    pub kit: Kit,
}

/// The ordinary ruler: 0 Hz at the left end, the top A at the right.
const LIN_LEFT: f64 = -6.2;
const LIN_RIGHT: f64 = 6.2;
const LIN_Y: f64 = 1.75;
/// The keyboard strip: A1 at the left edge, A7 at the right, 43 white keys.
const KEY_LEFT: f64 = -6.2;
const KEY_RIGHT: f64 = 6.2;
const KEY_TOP: f64 = -0.55;
const KEY_H: f64 = 1.25;
const WHITE_KEYS: usize = 43;
/// Where the A dots ride above the keyboard.
const DOT_Y: f64 = -0.2;
/// 00_6's preview: the dial's and the grid's centre line, and the row of words
/// beneath the pictures, below the lowest the grid's corners reach as it turns.
const PREVIEW_Y: f64 = 0.6;
const PREVIEW_WORD_Y: f64 = -0.8;
const DIAL_R: f64 = 0.9;
/// The preview's grid: its generator, its scene units per grid unit, its lines.
const SWIRL: crate::model::M2 = [[0.15, -1.2], [1.2, 0.15]];
const SWIRL_UNIT: f64 = 0.35;
const SWIRL_REACH: i32 = 2;

fn lin_x(hz: f64, top: f64) -> f64 {
    LIN_LEFT + (LIN_RIGHT - LIN_LEFT) * hz / top
}

fn white_w() -> f64 {
    (KEY_RIGHT - KEY_LEFT) / WHITE_KEYS as f64
}

/// The keyboard x of semitone `s` above A1 (A-major-ordered white keys: A B
/// C D E F G; black keys follow white keys 0, 2, 3, 5, 6 of each octave).
pub fn key_x(s: i32) -> f64 {
    let (octave, within) = (s.div_euclid(12), s.rem_euclid(12));
    // white index within the octave, and whether this semitone is black
    let (white, black) = match within {
        0 => (0, false),
        1 => (0, true),
        2 => (1, false),
        3 => (2, false),
        4 => (2, true),
        5 => (3, false),
        6 => (3, true),
        7 => (4, false),
        8 => (5, false),
        9 => (5, true),
        10 => (6, false),
        _ => (6, true),
    };
    let w = white_w();
    let index = 7 * octave + white;
    if black {
        KEY_LEFT + w * f64::from(index + 1)
    } else {
        KEY_LEFT + w * (f64::from(index) + 0.5)
    }
}

fn keyboard(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let w = white_w();
    let mut parts = Vec::new();
    for k in 0..WHITE_KEYS {
        let x = KEY_LEFT + w * k as f64;
        let r = Rectangle::new()
            .width(w * 0.92)
            .height(KEY_H)
            .style(flat(Palette::faint(), 1.0))
            .build()?
            .moved_to(v(x + w / 2.0, KEY_TOP - KEY_H / 2.0));
        parts.push(stage.arena_mut().add(r));
    }
    for octave in 0..6 {
        for white in [0, 2, 3, 5, 6] {
            let x = KEY_LEFT + w * f64::from(7 * octave + white + 1);
            let r = Rectangle::new()
                .width(w * 0.58)
                .height(KEY_H * 0.6)
                .style(flat(explainer_kit::hex(explainer_kit::BACKGROUND), 1.0))
                .build()?
                .moved_to(v(x, KEY_TOP - KEY_H * 0.3));
            parts.push(stage.arena_mut().add(r));
        }
    }
    group(stage, &parts)
}

/// A curved hop from `a` to `b` above the keyboard (a doubling, a half step).
fn hop(a: f64, b: f64, y: f64, height: f64) -> Vec<Vec3> {
    (0..=24)
        .map(|k| {
            let t = f64::from(k) / 24.0;
            v(a + (b - a) * t, y + height * 4.0 * t * (1.0 - t))
        })
        .collect()
}

impl SceneConstruct for RatioRuler {
    fn name(&self) -> &str {
        "00_ratio_ruler"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (green, blue, gold, red) = (number_c(), position_c(), answer_c(), warn_c());
        let freqs = fact_list("music.freqs_hz")?;
        let top = fact_f64("music.ruler_top_hz")?;
        let start = fact_f64("music.start_hz")?;
        let half_hz = fact_f64("music.half_octave_hz")?;
        let linear_mid = fact_f64("music.linear_mid_hz")?;
        let keys_per = fact_f64("music.keys_per_octave")? as i32;
        let half_keys = fact_f64("music.half_up_keys")? as i32;

        // 00_1: the title alone, then seven green numbers as the voice reads them
        kit.say(stage, "00_1")?;
        let heading = title(&kit, stage, "A ruler for ratios")?;
        let spine = seg(stage, v(LIN_LEFT, LIN_Y), v(LIN_RIGHT, LIN_Y), Palette::dim(), 3.0);
        let mut ticks = vec![spine];
        for q in 0..=4 {
            let hz = top * f64::from(q) / 4.0;
            let x = lin_x(hz, top);
            ticks.push(seg(stage, v(x, LIN_Y - 0.12), v(x, LIN_Y + 0.12), Palette::dim(), 2.5));
            ticks.push(kit.text(
                stage,
                &format!("{}", hz.round() as i64),
                22.0,
                Palette::dim(),
                v(x, LIN_Y - 0.38),
            )?);
        }
        let unit = kit.text(
            stage,
            "vibrations per second",
            22.0,
            Palette::dim(),
            v(4.6, LIN_Y + 0.95),
        )?;
        fade_in_all(stage, &[heading], 0.6)?;
        kit.until_sentence(stage, "00_1", 1)?;
        // the seven dots, each driven by one tracker so the same dot later
        // walks down onto its key (identity kept through the change)
        let even = stage.add_value_tracker(0.0);
        let key_xs: Vec<f64> = (0..7).map(|k| key_x(keys_per * k)).collect();
        let lin_xs: Vec<f64> = freqs.iter().map(|&f| lin_x(f, top)).collect();
        let mut dots = Vec::new();
        for k in 0..7 {
            let (from, to) = (v(lin_xs[k], LIN_Y), v(key_xs[k], DOT_Y));
            let d = stage.always_redraw(move |s| {
                let t = value(s, even);
                let t = t * t * (3.0 - 2.0 * t);
                let p = v(from[0] + (to[0] - from[0]) * t, from[1] + (to[1] - from[1]) * t);
                s.add(Dot::new().point(p).radius(0.1).style(flat(green, 1.0)).build())
            });
            dots.push(d);
        }
        // labels: the high four have room, the low three share one label
        let mut labels = Vec::new();
        for (k, &x) in lin_xs.iter().enumerate().take(7).skip(3) {
            labels.push(kit.text(
                stage,
                fact("music.freqs_hz")?.split(',').nth(k).unwrap_or("?"),
                24.0,
                green,
                v(x, LIN_Y + 0.45),
            )?);
        }
        let low_words = fact("music.freqs_hz")?
            .split(',')
            .take(3)
            .collect::<Vec<_>>()
            .join(", ");
        let low = kit.text(stage, &low_words, 22.0, green, v(lin_xs[0] + 0.55, LIN_Y + 0.95))?;
        let low_lead = seg(
            stage,
            v(lin_xs[1], LIN_Y + 0.15),
            v(lin_xs[0] + 0.45, LIN_Y + 0.75),
            green,
            1.8,
        );
        labels.insert(0, low);
        labels.push(unit);
        read_out(stage, &kit, &labels)?;

        // 00_2: the ruler it asks for, then half of it for one step, an eighth for the first four
        kit.say(stage, "00_2")?;
        fade_in_lagged(stage, &ticks, 1.0, 0.15)?;
        for &d in &dots {
            stage.add_to_scene(d)?;
            stage.wait(0.18)?;
        }
        fade_in_all(stage, &[low_lead], 0.2)?;
        kit.until_sentence(stage, "00_2", 1)?;
        let half_a = lin_x(freqs[5], top);
        let brace_y = LIN_Y - 0.75;
        let half_bar = path(
            stage,
            vec![
                v(half_a, brace_y + 0.12),
                v(half_a, brace_y),
                v(LIN_RIGHT, brace_y),
                v(LIN_RIGHT, brace_y + 0.12),
            ],
            gold,
            3.0,
        );
        let half_word = kit.text(
            stage,
            &format!("{} of the ruler", fact("music.last_step_share")?),
            24.0,
            gold,
            v((half_a + LIN_RIGHT) / 2.0, brace_y - 0.32),
        )?;
        fade_in_all(stage, &[half_bar, half_word], 0.6)?;
        kit.until_sentence(stage, "00_2", 2)?;
        let eighth_b = lin_x(freqs[3], top);
        let eighth_bar = path(
            stage,
            vec![
                v(LIN_LEFT, brace_y + 0.12),
                v(LIN_LEFT, brace_y),
                v(eighth_b, brace_y),
                v(eighth_b, brace_y + 0.12),
            ],
            gold,
            3.0,
        );
        let eighth_word = kit.text(
            stage,
            &format!("{} of the ruler", fact("music.first_four_share")?),
            24.0,
            gold,
            v(LIN_LEFT + 1.1, brace_y - 0.32),
        )?;
        fade_in_all(stage, &[eighth_bar, eighth_word], 0.6)?;

        // 00_3: the keyboard; the same seven dots walk down to their keys
        kit.say(stage, "00_3")?;
        let keys = keyboard(stage)?;
        fade_out_all(stage, &[half_bar, half_word, eighth_bar, eighth_word], 0.4)?;
        fade_in_all(stage, &[keys], 0.8)?;
        // faint ghosts keep the old places visible while the dots move
        let ghosts: Vec<Mob> = lin_xs
            .iter()
            .map(|&x| dot(stage, v(x, LIN_Y), 0.07, Palette::faint()))
            .collect();
        fade_in_all(stage, &ghosts, 0.2)?;
        play!(stage; tween(even, 1.0, 2.4)?);
        kit.until_sentence(stage, "00_3", 1)?;
        // count every key, black or white, across one octave from A4: twelve
        // semitone steps, the first one bracketed
        let count_y = KEY_TOP - KEY_H - 0.12;
        let base = 3 * keys_per;
        let counts: Vec<Mob> = (0..=keys_per)
            .map(|j| dot(stage, v(key_x(base + j), count_y), 0.05, blue))
            .collect();
        let one_step = seg(
            stage,
            v(key_x(base), count_y - 0.18),
            v(key_x(base + 1), count_y - 0.18),
            blue,
            3.0,
        );
        let one_word = kit.text(stage, "1 semitone", 22.0, blue, v(key_x(base) - 0.25, count_y - 0.45))?;
        fade_in_lagged(stage, &counts, 1.8, 0.15)?;
        fade_in_all(stage, &[one_step, one_word], 0.4)?;
        kit.until_sentence(stage, "00_3", 2)?;
        let mut hops = Vec::new();
        for k in 0..6 {
            hops.push(path(
                stage,
                hop(key_xs[k], key_xs[k + 1], DOT_Y + 0.15, 0.55),
                gold,
                3.0,
            ));
            hops.push(kit.text(
                stage,
                "×2",
                24.0,
                gold,
                v((key_xs[k] + key_xs[k + 1]) / 2.0, DOT_Y + 0.95),
            )?);
        }
        fade_in_lagged(stage, &hops, 2.0, 0.25)?;
        let twelve = kit.text(
            stage,
            &format!("{keys_per} semitones"),
            22.0,
            blue,
            v((key_xs[3] + key_xs[4]) / 2.0, count_y - 0.45),
        )?;
        let twelve_bar = path(
            stage,
            vec![v(key_xs[3], count_y - 0.18), v(key_xs[4], count_y - 0.18)],
            blue,
            3.0,
        );
        swap(stage, &[one_step, one_word], &[twelve, twelve_bar], 0.5)?;
        kit.until_sentence(stage, "00_3", 3)?;
        let ratio = kit.text(
            stage,
            "note interval depends on frequency ratio",
            26.0,
            Palette::ink(),
            v(0.0, -2.75),
        )?;
        fade_in_all(stage, &[ratio], 0.6)?;

        // 00_4: from 440, six semitones up — predict the frequency
        kit.say(stage, "00_4")?;
        fade_out_all(stage, &hops, 0.4)?;
        let mut counted = counts.clone();
        counted.extend([ratio, twelve, twelve_bar]);
        fade_out_all(stage, &counted, 0.3)?;
        let a4 = key_xs[3];
        let target_x = key_x(keys_per * 3 + half_keys);
        let a4_ring = super::draw::ring(stage, v(a4, DOT_Y), 0.2, gold, 3.0);
        let a4_word = kit.text(
            stage,
            &format!("{}", start.round() as i64),
            26.0,
            green,
            v(a4 - 0.5, DOT_Y + 0.45),
        )?;
        fade_in_all(stage, &[a4_ring, a4_word], 0.5)?;
        kit.until_sentence(stage, "00_4", 1)?;
        let up = path(stage, hop(a4, target_x, DOT_Y + 0.15, 0.6), blue, 3.0);
        let under = KEY_TOP - KEY_H - 0.1;
        let up_bar = path(stage, vec![v(a4, under), v(target_x, under)], blue, 3.0);
        let up_word = kit.text(
            stage,
            &format!("{half_keys} semitones"),
            24.0,
            blue,
            v((a4 + target_x) / 2.0, under - 0.3),
        )?;
        let mark = dot(stage, v(target_x, DOT_Y), 0.1, gold);
        let ask = kit.text(stage, "? Hz", 30.0, gold, v(target_x + 0.15, DOT_Y + 1.15))?;
        fade_in_all(stage, &[up, up_bar, up_word, mark, ask], 0.6)?;
        kit.until_sentence(stage, "00_4", 3)?;
        let guess_x = lin_x(linear_mid, top);
        let guess = seg(stage, v(guess_x, LIN_Y - 0.2), v(guess_x, LIN_Y + 0.2), red, 4.0);
        let guess_word = kit.text(
            stage,
            &format!("{} by difference?", linear_mid.round() as i64),
            22.0,
            red,
            v(guess_x + 0.2, LIN_Y - 0.62),
        )?;
        fade_in_all(stage, &[guess, guess_word], 0.5)?;
        predict(&kit, stage, v(3.6, -2.75), 3.0)?;

        // 00_5: two equal half-climbs double, so each is ×√2 ≈ 622
        kit.say(stage, "00_5")?;
        let a5 = key_xs[4];
        let second = path(stage, hop(target_x, a5, DOT_Y + 0.15, 0.6), blue, 3.0);
        let a5_word = kit.text(
            stage,
            &format!("{}", (2.0 * start).round() as i64),
            26.0,
            green,
            v(a5 + 0.5, DOT_Y + 0.45),
        )?;
        let both_unknown = kit.text(stage, "× ?   then   × ?   =   ×2", 28.0, blue, v(2.0, -2.75))?;
        fade_in_all(stage, &[second, a5_word, both_unknown], 0.6)?;
        kit.until_sentence(stage, "00_5", 1)?;
        let both = kit.text(stage, "×√2   then   ×√2   =   ×2", 28.0, blue, v(2.0, -2.75))?;
        swap(stage, &[both_unknown], &[both], 0.5)?;
        kit.until_sentence(stage, "00_5", 2)?;
        let shown_hz = fact("music.half_octave_hz")?;
        let answer = kit.text(
            stage,
            &format!("{shown_hz} Hz"),
            30.0,
            gold,
            v(target_x + 0.15, DOT_Y + 1.15),
        )?;
        swap(stage, &[ask], &[answer], 0.5)?;
        // on the ordinary ruler: the true note sits short of the guess
        let truth_x = lin_x(half_hz, top);
        let truth = seg(stage, v(truth_x, LIN_Y - 0.2), v(truth_x, LIN_Y + 0.2), gold, 4.0);
        let truth_word = kit.text(
            stage,
            fact("music.half_octave_spoken")?,
            22.0,
            gold,
            v(truth_x - 0.25, LIN_Y + 0.95),
        )?;
        let truth_lead = seg(
            stage,
            v(truth_x, LIN_Y + 0.22),
            v(truth_x - 0.2, LIN_Y + 0.75),
            gold,
            1.8,
        );
        let strike = seg(
            stage,
            v(guess_x - 0.75, LIN_Y - 0.7),
            v(guess_x + 1.15, LIN_Y - 0.54),
            red,
            3.0,
        );
        kit.allow_crossing(strike);
        fade_in_all(stage, &[truth, truth_word, truth_lead, strike], 0.6)?;
        kit.until_sentence(stage, "00_5", 3)?;
        let rule = kit.text(
            stage,
            "equal steps on the keyboard = equal ratios",
            28.0,
            Palette::ink(),
            v(0.0, -3.25),
        )?;
        let fine = precisely(
            &kit,
            stage,
            "equal temperament with A4 = 440 Hz (ISO 16); real pianos are often tuned slightly stretched",
        )?;
        fade_in_all(stage, &[rule, fine], 0.6)?;

        // 00_6: the preview — a slide, a dial that forgets laps, a flow
        kit.say(stage, "00_6")?;
        preview(&kit, stage)?;
        kit.finish(stage)
    }
}

/// 00_6's preview: three small pictures the film will make precise, each
/// arriving with the sentence that names it.
fn preview(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    let (blue, green, purple, teal) = (position_c(), number_c(), super::draw::lap_c(), super::draw::slide_c());
    kit.until_sentence(stage, "00_6", 1)?;
    let everything: Vec<Mob> = stage.scene().mobjects().to_vec();
    fade_out_all(stage, &everything, 0.6)?;
    let heading = title(kit, stage, "Where the ruler goes")?;
    // 1: two rulers, one sliding along the other
    let a = seg(stage, v(-6.0, 0.6), v(-2.6, 0.6), green, 4.0);
    let b = seg(stage, v(-5.2, 1.0), v(-1.8, 1.0), teal, 4.0);
    let w1 = kit.text(
        stage,
        "multiply by sliding",
        24.0,
        Palette::ink(),
        v(-4.0, PREVIEW_WORD_Y),
    )?;
    fade_in_all(stage, &[heading, a, b, w1], 0.6)?;
    super::draw::shift_all(stage, &[b], v(0.6, 0.0), 1.0)?;
    kit.until_sentence(stage, "00_6", 2)?;
    // 2: a small grid swirling under a flow
    let spin = stage.add_value_tracker(0.0);
    let grid = stage.always_redraw(move |s| {
        let m = crate::model::flow(SWIRL, value(s, spin));
        super::draw::grid_under(s, m, v(4.0, PREVIEW_Y), SWIRL_UNIT, SWIRL_REACH, blue, 0.9)
    });
    let w3 = kit.text(
        stage,
        "motions of the plane",
        24.0,
        Palette::ink(),
        v(4.0, PREVIEW_WORD_Y),
    )?;
    stage.add_to_scene(grid)?;
    fade_in_all(stage, &[w3], 0.5)?;
    play!(stage; tween(spin, 1.0, 2.5)?);
    kit.until_sentence(stage, "00_6", 3)?;
    {
        // 3: a dial whose pointer goes round and round, laps counted
        let centre = v(0.0, PREVIEW_Y);
        let dial = super::draw::ring(stage, centre, DIAL_R, Palette::dim(), 3.0);
        let turn = stage.add_value_tracker(0.0);
        let pointer = stage.always_redraw(move |s| {
            let a = std::f64::consts::TAU * value(s, turn);
            let tip = v(centre[0] + 0.8 * a.sin(), centre[1] + 0.8 * a.cos());
            super::draw::arrow_parts(s, centre, tip, purple, 4.0, 0.2)
        });
        let w2 = kit.text(stage, "laps forgotten", 24.0, Palette::ink(), v(0.0, PREVIEW_WORD_Y))?;
        fade_in_all(stage, &[dial, w2], 0.5)?;
        stage.add_to_scene(pointer)?;
        play!(stage; tween(turn, 2.0, 2.0)?);
    }
    kit.until_sentence(stage, "00_6", 4)?;
    let unreachable = kit.text(
        stage,
        "and a transformation no steady flow reaches",
        24.0,
        super::draw::warn_c(),
        v(4.0, PREVIEW_WORD_Y - 0.6),
    )?;
    fade_in_all(stage, &[unreachable], 0.5)?;
    kit.hold(stage)
}

const SCRIPT: &str = include_str!("../../script.tsv");
/// The words each printed number waits for in 00_1's second sentence, in reading
/// order: the low three share a label and come with "their frequencies are", then
/// 440 to 3520, then the unit.
const READ_PHRASES: [&str; 6] = [
    "their frequencies",
    "four hundred and forty",
    "eight hundred and eighty",
    "seventeen sixty",
    "thirty-five twenty",
    "vibrations per second",
];
const READ_FADE: f64 = 0.35;

/// Where each phrase opens, as a share of the sentence's characters: a rough
/// estimate of when it is read, never measured timing.
fn read_shares(sentence: &str) -> Option<Vec<f64>> {
    let total = sentence.chars().count() as f64;
    READ_PHRASES
        .iter()
        .map(|p| Some(sentence[..sentence.find(p)?].chars().count() as f64 / total))
        .collect()
}

/// Bring `words` in one at a time, as 00_1's second sentence reaches them. Without
/// a narrator, one every 0.9 s.
fn read_out(stage: &mut Stage<'_>, kit: &Kit, words: &[Mob]) -> fmn::Result<()> {
    let now = stage.scene().time().to_f64();
    let times = kit.narrator.as_ref().and_then(|n| {
        let s1 = n.sentence_at("00_1", 1)?;
        let end = s1 - n.sentence_starts("00_1").get(1).copied()? + n.length("00_1")?;
        let text = SCRIPT.lines().find_map(|row| row.strip_prefix("00_1\t"))?;
        let sentence = &text[text.find(". ")? + 2..];
        Some(
            read_shares(sentence)?
                .iter()
                .map(|s| s1 + (end - s1) * s)
                .collect::<Vec<f64>>(),
        )
    });
    for (j, &word) in words.iter().enumerate() {
        let untimed = now + 0.9 * j as f64;
        let at = times.as_ref().and_then(|t| t.get(j)).copied().unwrap_or(untimed);
        super::draw::wait_until(stage, at)?;
        fade_in_all(stage, &[word], READ_FADE)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scene code without these tests, for checks on the order of its beats.
    fn scene_source() -> &'static str {
        include_str!("ratio_ruler.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap_or("")
    }

    fn found(text: &str) -> usize {
        scene_source()
            .find(text)
            .unwrap_or_else(|| panic!("not in the scene: {text}"))
    }

    #[test]
    fn the_ruler_waits_for_the_words_that_ask_for_it() {
        // the title alone, the numbers through sentence 1, and only then the axis and the dots
        let order = [
            found("fade_in_all(stage, &[heading], 0.6)"),
            found(r#"kit.until_sentence(stage, "00_1", 1)"#),
            found("read_out(stage, &kit, &labels)"),
            found(r#"kit.say(stage, "00_2")"#),
            found("fade_in_lagged(stage, &ticks, 1.0, 0.15)"),
            found("stage.add_to_scene(d)"),
            found("fade_in_all(stage, &[low_lead], 0.2)"),
            found(r#"kit.until_sentence(stage, "00_2", 1)"#),
        ];
        assert!(order.windows(2).all(|w| w[0] < w[1]), "{order:?}");
        // the ruler, seven dots at 0.18 s and the lead fit the 3.0 s of "Mark them on an ordinary ruler
        // of frequency" that the scratch voice gives, with room for a quicker reading
        found("stage.wait(0.18)");
        let ruler_seconds: f64 = 1.0 + 7.0 * 0.18 + 0.2;
        assert!(ruler_seconds < 2.6, "{ruler_seconds}");
    }

    #[test]
    fn the_numbers_arrive_in_the_order_the_voice_reads_them() {
        let text = SCRIPT
            .lines()
            .find_map(|row| row.strip_prefix("00_1\t"))
            .expect("00_1 is in the script");
        let sentence = &text[text.find(". ").expect("two sentences") + 2..];
        let shares = read_shares(sentence).expect("every phrase is in the sentence");
        assert!(shares.windows(2).all(|w| w[0] < w[1]), "{shares:?}");
        assert!(shares[0] > 0.0 && shares[shares.len() - 1] < 1.0, "{shares:?}");
        // the labels vector matches the phrases: low three first, four high ones, the unit last
        assert_eq!(READ_PHRASES.len(), 1 + 4 + 1);
        assert!(found(".take(7).skip(3)") < found("labels.insert(0, low)"));
        assert!(found("labels.insert(0, low)") < found("labels.push(unit)"));
    }

    #[test]
    fn the_a_keys_are_twelve_semitones_apart_and_on_screen() {
        let xs: Vec<f64> = (0..7).map(|k| key_x(12 * k)).collect();
        let gap = xs[1] - xs[0];
        for w in xs.windows(2) {
            assert!((w[1] - w[0] - gap).abs() < 1e-9, "evenly spaced");
        }
        assert!(xs[0] > KEY_LEFT && xs[6] < KEY_RIGHT);
        // the half-octave key is black and sits between A4 and A5
        let d = key_x(36 + 6);
        assert!(d > xs[3] && d < xs[4]);
    }

    #[test]
    fn the_preview_words_clear_the_dial_and_the_grid_through_its_whole_turn() {
        // a word's top stands 0.13 above its centre line, read from preview frames; leave 0.1 more
        const { assert!(PREVIEW_Y - DIAL_R > PREVIEW_WORD_Y + 0.13 + 0.1) };
        let reach = f64::from(SWIRL_REACH);
        for j in 0..=200 {
            let m = crate::model::flow(SWIRL, f64::from(j) / 200.0);
            // the grid is a parallelogram, lowest at a corner
            for corner in [[reach, reach], [reach, -reach], [-reach, reach], [-reach, -reach]] {
                let y = PREVIEW_Y + SWIRL_UNIT * crate::model::apply(m, corner)[1];
                assert!(y > PREVIEW_WORD_Y + 0.13 + 0.1, "{j}: {y}");
            }
        }
    }

    #[test]
    fn the_linear_ruler_puts_the_guess_right_of_the_truth() {
        let top = 3520.0;
        assert!(lin_x(622.25, top) < lin_x(660.0, top));
        assert!((lin_x(1760.0, top) - (LIN_LEFT + LIN_RIGHT) / 2.0).abs() < 1e-12);
    }
}
