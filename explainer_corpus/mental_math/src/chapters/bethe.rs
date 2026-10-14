//! Chapter 09, Bethe: a true story from Feynman's book, told with nothing
//! added. Los Alamos and the two men on name cards, and the square their
//! formula called for (09_1); the calculator, and Bethe's first answer
//! (09_2); his exact answer stands still while the machine is still running,
//! and the machine catches up (09_3); his question, verbatim (09_4); the
//! question put to the viewer, with the square of forty-eight inside the
//! outline of fifty (09_5), held. The two squares stay to the end: chapter
//! 10 picks them up and answers the question with the picture.

use explainer_kit::animatic::HOLD_S;
use explainer_kit::{Kit, Palette, Timed, play, v};
use fmn::prelude::*;

use super::draw::{
    Calculator, MoveFigure, QuoteCard, SIDE_PT, Side, eq, fade_in_all, fade_in_lagged, fade_out_all, fail, fifty_board,
    fly, glyphs, line_style, middle_of, num, on_stage, panel, pause, quiz, repaint, say, slate, swap, tween_linear,
    wait_until, why_pause,
};
use super::sound;
use crate::facts::cited;
use crate::model::{FIFTY, near_fifty, one_move};

pub struct Bethe {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const LOS_ALAMOS: &str = "Los Alamos";
/// The two name cards: "Hans Bethe, physicist, head of the Theoretical
/// Division" and "Richard Feynman, physicist, in Bethe's division", each
/// broken into the name and the lines under it.
const BETHE: &str = "Hans Bethe";
const BETHE_ROLE: [&str; 3] = ["physicist,", "head of the", "Theoretical Division"];
const FEYNMAN: &str = "Richard Feynman";
const FEYNMAN_ROLE: [&str; 2] = ["physicist,", "in Bethe's division"];
/// The book's word for the machine; the drawing is generic.
const MARCHANT: &str = "a Marchant calculator";
/// One of the video's two verbatim quotations, and its source.
const QUESTION: &str = "“Don’t you know how to square numbers near 50?”";
const AS_TOLD: &str = "Hans Bethe, as told by Richard P. Feynman";
const BOOK: &str = "“Surely You’re Joking, Mr. Feynman!”";
const BOOK_CHAPTER: &str = "“Lucky Numbers”";

/// Los Alamos as it opens the story, and the square between the cards.
const PLACE_PT: f64 = 90.0;
const SQUARE_PT: f64 = 110.0;
/// The name on a card, and the lines under it.
const NAME_PT: f64 = 42.0;
const ROLE_PT: f64 = 30.0;
/// The cards side by side, and the captions they shrink to (the name alone,
/// at 36 points).
const CARD_X: f64 = 4.4;
const CAPTION_AT: (f64, f64) = (3.6, 2.9);
const CAPTION_SCALE: f64 = 36.0 / NAME_PT;
/// Bethe's answer under his caption, and the calculator under Feynman's.
const ANSWER_PT: f64 = 150.0;
const CALC_SCALE: f64 = 1.5;
const CALC_Y: f64 = -0.55;
/// The quote card, larger than the kit's: it has the screen to itself.
const QUOTE_ZOOM: f64 = 1.5;
/// The question card of 09_5 (beside the squares).
const ASK_PT: f64 = 52.0;

/// How fast the machine's digits run while the voice speaks (seconds a
/// change), and how they slow to a stop in the silent window: the gap
/// before the first change and how much longer each gap is than the last.
const RUN_STEP: f64 = 0.11;
const FIRST_GAP: f64 = 0.07;
const SLOWING: f64 = 1.13;
/// Changes in the silent window, the last of which stops the last wheel.
const WINDOW_CHANGES: usize = 11;
/// The listen window the machine's ticks fill.
const MACHINE_S: f64 = 2.0;
/// Each wheel's step a change (each coprime to ten, so a wheel passes every
/// digit) and where it starts while it runs.
const WHEEL_STEP: [u64; 4] = [1, 3, 7, 9];
const WHEEL_START: [u64; 4] = [0, 5, 2, 7];

/// The digits of `n`, most significant first.
fn digits_of(n: u64) -> Vec<u64> {
    num(n).chars().filter_map(|c| c.to_digit(10)).map(u64::from).collect()
}

fn reading_of(wheels: &[u64]) -> String {
    wheels.iter().map(|d| d.to_string()).collect()
}

/// The readings the window shows while the voice speaks, in a loop of ten:
/// every wheel turns at once, and none of them is either of Bethe's answers.
fn running_readings(wheels: usize) -> Vec<String> {
    (0..10)
        .map(|k| {
            let w: Vec<u64> = (0..wheels)
                .map(|j| (WHEEL_START[j % 4] + k * WHEEL_STEP[j % 4]) % 10)
                .collect();
            reading_of(&w)
        })
        .collect()
}

/// The readings of the silent window, one a change: the wheels keep turning
/// and stop one by one, left to right, each as it reaches its digit of
/// `answer`; the last reading is `answer`.
fn stopping_readings(answer: u64) -> Vec<String> {
    let target = digits_of(answer);
    let n = target.len();
    let last = WINDOW_CHANGES;
    (0..=last)
        .map(|k| {
            let w: Vec<u64> = (0..n)
                .map(|j| {
                    let stop = last + j + 1 - n;
                    let step = WHEEL_STEP[j % 4];
                    if k >= stop {
                        target[j]
                    } else {
                        // `stop - k` steps short of its digit
                        (target[j] + 10 - ((stop - k) as u64 * step) % 10) % 10
                    }
                })
                .collect();
            reading_of(&w)
        })
        .collect()
}

/// When each change of the silent window comes (seconds into it): the gaps
/// lengthen, so the ticks slow down.
fn stopping_times() -> Vec<f64> {
    let mut out = vec![0.0];
    let mut gap = FIRST_GAP;
    for _ in 0..WINDOW_CHANGES {
        let t = out[out.len() - 1] + gap;
        out.push(t);
        gap *= SLOWING;
    }
    out
}

/// A name card: the name and the lines under it on a dark back. When it
/// shrinks to a caption the back and the lines go and the name stays.
struct NameCard {
    back: Mob,
    name: Mob,
    role: Vec<Mob>,
}

impl NameCard {
    /// The card `w` by `h` centred at `at`. Detached.
    #[allow(clippy::too_many_arguments)]
    fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        name: &str,
        role: &[&str],
        at: Vec3,
        w: f64,
        h: f64,
    ) -> fmn::Result<Self> {
        let (name_h, role_h) = (0.66, 0.44);
        let content = name_h + role_h * role.len() as f64;
        let top = at[1] + content / 2.0 - name_h / 2.0;
        let back = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.14)
            .style(Style::default().fill(panel(), 0.94).stroke(Palette::faint(), 1.5, 1.0))
            .build()?
            .moved_to(at);
        let back = stage.arena_mut().add(back);
        let name = kit.text_styled(stage, name, NAME_PT, Palette::ink(), v(at[0], top), true, false)?;
        let role = role
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let y = top - name_h / 2.0 - role_h * (i as f64 + 0.5) - 0.06;
                kit.text(stage, line, ROLE_PT, Palette::dim(), v(at[0], y))
            })
            .collect::<fmn::Result<_>>()?;
        Ok(Self { back, name, role })
    }

    /// The back, the name and the lines, back to front.
    fn mobs(&self) -> Vec<Mob> {
        let mut out = vec![self.back, self.name];
        out.extend(&self.role);
        out
    }
}

/// The widest of several detached texts.
fn widest(stage: &Stage<'_>, mobs: &[Mob]) -> f64 {
    mobs.iter().map(|&m| stage.get_width(m)).fold(0.0, f64::max)
}

impl SceneConstruct for Bethe {
    fn name(&self) -> &str {
        "09_bethe"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let bethe_number = cited::BETHE_NUMBER;
        let (rough, exact) = (cited::BETHE_ROUGH, cited::BETHE_EXACT);
        let way = near_fifty(bethe_number);
        if way.rough != rough || way.square != exact {
            return Err(fail(format!(
                "near fifty gives {} and {} for {bethe_number}, the book {rough} and {exact}",
                way.rough, way.square
            )));
        }

        // 09_1: Los Alamos; the two men on name cards; forty-eight squared
        // the chapter opens on the place before the line's first words ("The
        // next trick comes with a story."; the line starts 0.6 to 1.0 s in),
        // not on an empty frame until "Los Alamos, ..." (3.1 s in the
        // finished read)
        let place = kit.text_styled(stage, LOS_ALAMOS, PLACE_PT, ink, v(0.0, 0.3), true, false)?;
        play!(stage; fade_in(stage.arena_mut(), place, v(0.0, -0.15), 1.0)?.rt(0.5));
        let line = say(&kit, stage, "09_1")?;
        line.wait_sentence(stage, 2)?;
        // both cards the same size: the wider name or line, the longer role
        let mut probe = Vec::new();
        for name in [BETHE, FEYNMAN] {
            probe.push(kit.text_styled(stage, name, NAME_PT, ink, ORIGIN, true, false)?);
        }
        for role in BETHE_ROLE.iter().chain(FEYNMAN_ROLE.iter()) {
            probe.push(kit.text(stage, role, ROLE_PT, Palette::dim(), ORIGIN)?);
        }
        let card_w = widest(stage, &probe) + 0.8;
        let card_h = 0.66 + 0.44 * BETHE_ROLE.len().max(FEYNMAN_ROLE.len()) as f64 + 0.6;
        let bethe = NameCard::build(&kit, stage, BETHE, &BETHE_ROLE, v(-CARD_X, -0.2), card_w, card_h)?;
        let feynman = NameCard::build(&kit, stage, FEYNMAN, &FEYNMAN_ROLE, v(CARD_X, -0.2), card_w, card_h)?;
        // each card on its name, sliding in from its side
        let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(fly(place, v(0.0, 3.0), 0.6, 0.7)?)?];
        for m in bethe.mobs() {
            let a = fade_in(stage.arena_mut(), m, v(0.25, 0.0), 1.0)?;
            anims.push(stage.prepare(a.rt(0.7))?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, "Richard Feynman")?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for m in feynman.mobs() {
            let a = fade_in(stage.arena_mut(), m, v(-0.25, 0.0), 1.0)?;
            anims.push(stage.prepare(a.rt(0.7))?);
        }
        stage.play_prepared(anims)?;
        line.wait_for(stage, "forty-eight squared")?;
        let square = eq(&kit, stage, &format!("{bethe_number}^2"), SQUARE_PT, v(0.0, -0.2))?;
        play!(stage; fade_in(stage.arena_mut(), square, v(0.0, -0.15), 1.0)?.rt(0.6));

        // 09_2: the cards shrink to captions at the sides (the names stay);
        // the calculator under Feynman's; Bethe's first answer under his
        let line = say(&kit, stage, "09_2")?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for (card, x) in [(&bethe, -CAPTION_AT.0), (&feynman, CAPTION_AT.0)] {
            anims.push(stage.prepare(fly(card.name, v(x, CAPTION_AT.1), CAPTION_SCALE, 0.8)?)?);
            // the back shrinks towards the caption as it goes; the lines on
            // it simply fade, so they never crowd the name on its way
            let to = v(
                x - stage.get_center(card.back)[0],
                CAPTION_AT.1 - stage.get_center(card.back)[1],
            );
            let a = fade_out(stage.arena_mut(), card.back, to, 0.3)?;
            anims.push(stage.prepare(a.rt(0.7))?);
            for &m in &card.role {
                let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(0.3))?);
            }
        }
        anims.push(stage.prepare(fly(square, v(0.0, CAPTION_AT.1), 0.75, 0.8)?)?);
        let a = fade_out(stage.arena_mut(), place, v(0.0, 0.3), 1.0)?;
        anims.push(stage.prepare(a.rt(0.5))?);
        stage.play_prepared(anims)?;
        let calc = Calculator::build(stage, v(CAPTION_AT.0, CALC_Y), CALC_SCALE)?;
        let label_y = CALC_Y - 1.5 * CALC_SCALE - 0.4;
        let label = kit.text(stage, MARCHANT, 32.0, Palette::dim(), v(CAPTION_AT.0, label_y))?;
        fade_in_all(stage, &calc.parts[..2], 0.4)?;
        fade_in_lagged(stage, &calc.parts[2..], 0.8, 0.15)?;
        play!(stage; fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?.rt(0.4));
        line.wait_sentence(stage, 2)?;
        // 2300, with its hundreds and its last two digits set apart so the
        // last two can give way at 09_3; level with the machine's window, so
        // the two answers can be compared
        let answer_at = v(-CAPTION_AT.0, calc.window()[1]);
        let whole = eq(&kit, stage, &num(rough), ANSWER_PT, answer_at)?;
        let all = glyphs(stage, whole);
        if all.len() != 4 {
            return Err(fail(format!("{rough} is not four digits")));
        }
        let (front_at, back_at) = (middle_of(stage, &all[..2])?, middle_of(stage, &all[2..])?);
        let front = eq(&kit, stage, &num(rough / 100), ANSWER_PT, front_at)?;
        let zeros = eq(&kit, stage, &format!("{:02}", rough % 100), ANSWER_PT, back_at)?;
        play!(stage;
            fade_in(stage.arena_mut(), front, v(0.0, -0.12), 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), zeros, v(0.0, -0.12), 1.0)?.rt(0.5),
        );

        // 09_3: Feynman keys it in and the machine runs; Bethe's 04; the
        // machine catches up in the silence and stops on 2, 3, 0, 4
        let line = say(&kit, stage, "09_3")?;
        // four keys pressed, one after another (the keyboard is generic)
        for key in [6usize, 12, 8, 14] {
            let key = calc.parts[2 + key];
            play!(stage; repaint(key, ink, 0.85, 0.12)?);
            play!(stage; repaint(key, ink, 0.0, 0.14)?);
        }
        let wheels = digits_of(exact).len();
        let running = running_readings(wheels);
        let stopping = stopping_readings(exact);
        for r in running.iter().chain(&stopping[..stopping.len() - 1]) {
            if r == &num(rough) || r == &num(exact) {
                return Err(fail(format!("the running machine would show an answer: {r}")));
            }
        }
        if stopping.last() != Some(&num(exact)) {
            return Err(fail(format!("the machine stops on {stopping:?}, not {exact}")));
        }
        let shown: Vec<Mob> = running
            .iter()
            .map(|r| calc.reading(&kit, stage, r, ink))
            .collect::<fmn::Result<_>>()?;
        for &m in &shown {
            stage.set_opacity(m, 0.0, true);
            stage.add_to_scene(m)?;
        }
        // the digits run while the voice goes on: one updater shows the
        // reading a tracker counts to, and the tracker is tweened at an even
        // rate through every wait (a dt updater only ran while something
        // else animated, so a reading sat still for a second: R2, M9)
        let runner = stage.arena_mut().add(VMobject::new());
        stage.add_to_scene(runner)?;
        let count = stage.add_value_tracker(0.0);
        let loop_of = shown.clone();
        let mut last = usize::MAX;
        stage.add_updater(
            runner,
            move |s, _| {
                let now = s.tracker_value(count).unwrap_or(0.0) as usize % loop_of.len();
                if now != last {
                    for (k, &m) in loop_of.iter().enumerate() {
                        s.set_opacity(m, if k == now { 1.0 } else { 0.0 }, true);
                    }
                    last = now;
                }
            },
            true,
        )?;
        let mut counted = 0.0;
        let mut run_until = |stage: &mut Stage<'_>, t_end: f64| -> fmn::Result<()> {
            let span = t_end - stage.scene().time().to_f64();
            if span > 0.0 {
                counted += span / RUN_STEP;
                stage.play(tween_linear(count, counted, span)?)?;
            }
            Ok(())
        };
        run_until(stage, line.at("twenty-three oh four")?)?;
        let four = eq(&kit, stage, &format!("{:02}", exact % 100), ANSWER_PT, back_at)?;
        swap(stage, &[zeros], &[four], 0.5)?;
        run_until(stage, line.end() + explainer_kit::narration::BREATH)?;
        // the window: the narrator silent, dry ticks that slow, one a change
        let times = stopping_times();
        let ticks = sound::ticks(&times, MACHINE_S)?;
        kit.sonify(stage, MACHINE_S, "09_bethe:09_3", &ticks)?;
        let start = stage.scene().time().to_f64();
        stage.clear_updaters(runner, true);
        stage.remove_from_scene(runner);
        for &m in &shown {
            stage.remove_from_scene(m);
        }
        let mut current: Option<Mob> = None;
        for (r, &t) in stopping.iter().zip(&times) {
            let next = calc.reading(&kit, stage, r, ink)?;
            wait_until(stage, start + t)?;
            if let Some(m) = current {
                stage.remove_from_scene(m);
            }
            stage.add_to_scene(next)?;
            current = Some(next);
        }

        // 09_4: the stage clears for the question; the quote card; its
        // source after the hold
        let line = say(&kit, stage, "09_4")?;
        line.wait_sentence(stage, 1)?;
        let source_line = format!("{BOOK} ({}), {BOOK_CHAPTER}", cited::FEYNMAN_YEAR);
        let quote_at = v(0.0, 0.2);
        let card = QuoteCard::build(&kit, stage, QUESTION, &[AS_TOLD, &source_line], quote_at, 60, 1)?;
        let mut parts = vec![card.back];
        parts.extend(&card.lines);
        parts.extend(&card.source);
        for &m in &parts {
            stage.scale_about(m, QUOTE_ZOOM, Some(quote_at), None);
        }
        // the stage gives way to the quote as the question is asked: it was
        // cleared at "And Bethe asks him a question." and stood bare for 1.8 s
        // (finished read) until the quote came
        line.wait_sentence(stage, 2)?;
        let mut first = vec![card.back];
        first.extend(&card.lines);
        swap(stage, &on_stage(stage), &first, 0.7)?;
        // the source comes in as the question ends, so it is up for the
        // whole hold (R2, M10: after the hold it showed for a second and a
        // half)
        kit.hold(stage)?;
        fade_in_all(stage, &card.source, 0.5)?;
        kit.music_breath(stage, HOLD_S)?;
        pause(stage, 0.6)?;

        // 09_5: fifty squared, and the square of forty-eight inside it; the
        // question, held
        let line = say(&kit, stage, "09_5")?;
        fade_out_all(stage, &parts, 0.5)?;
        let board = fifty_board();
        let fifty = FIFTY as f64;
        let outline = board.rect(stage, [0.0, 0.0, fifty, fifty], line_style(ink, 2.5));
        let fifty_label = board.label(
            &kit,
            stage,
            &num(FIFTY),
            [0.0, 0.0, fifty, fifty],
            Side::Above,
            SIDE_PT,
            ink,
        )?;
        let m48 = one_move(bethe_number, FIFTY - bethe_number)
            .ok_or_else(|| fail(format!("{bethe_number} is not below {FIFTY}")))?;
        let figure = MoveFigure::build(stage, board, m48, slate(), slate(), 0.0)?;
        let side48 = board.label(
            &kit,
            stage,
            &num(m48.side),
            figure.square_rect(),
            Side::Left,
            SIDE_PT,
            ink,
        )?;
        // the question card: fifty squared on the left, forty-eight squared
        // on the right, edged in the quiz's colour
        let card_at = v(3.5, 2.75);
        let known = eq(&kit, stage, &format!("{FIFTY}^2 = {}", FIFTY * FIFTY), ASK_PT, ORIGIN)?;
        // forty-eight squared is set whole twice, with the question mark and
        // with the answer, only to place its pieces: "48² =", then "?" and
        // later "2304" where each sits in its whole, so the question mark can
        // give way to the answer on the same baseline
        let gap = 0.8;
        let asked_src = format!("{bethe_number}^2 =");
        // (braced, the question mark is spaced from the equals sign as a
        // number would be)
        let with_mark = eq(&kit, stage, &format!("{asked_src} {{?}}"), ASK_PT, ORIGIN)?;
        let with_answer = eq(&kit, stage, &format!("{asked_src} {exact}"), ASK_PT, ORIGIN)?;
        let row_w = stage.get_width(known) + gap + stage.get_width(with_answer);
        let left = card_at[0] - row_w / 2.0;
        stage.move_to(known, v(left, card_at[1]), LEFT);
        let asked_x = stage.get_right(known)[0] + gap;
        // both wholes reach from the superscript down to the digits' foot,
        // so the same left edge and middle put "48² =" in the same place
        for m in [with_mark, with_answer] {
            stage.move_to(m, v(asked_x, card_at[1]), LEFT);
        }
        let (mark_glyphs, answer_glyphs) = (glyphs(stage, with_mark), glyphs(stage, with_answer));
        let head = mark_glyphs.len() - 1;
        if answer_glyphs.len() != head + num(exact).len() {
            return Err(fail(format!(
                "{asked_src} is set differently before {exact} and before ?"
            )));
        }
        let asked = eq(&kit, stage, &asked_src, ASK_PT, middle_of(stage, &mark_glyphs[..head])?)?;
        let unknown = eq(&kit, stage, "?", ASK_PT, middle_of(stage, &mark_glyphs[head..])?)?;
        let back = Rectangle::new()
            .width(row_w + 0.7)
            .height(1.25)
            .corner_radius(0.14)
            .style(Style::default().fill(panel(), 0.96).stroke(quiz(), 2.5, 1.0))
            .build()?
            .moved_to(card_at);
        let back = stage.arena_mut().add(back);
        fade_in_all(stage, &[outline, fifty_label, back, known], 0.6)?;
        line.wait_sentence(stage, 1)?;
        let mut inside = figure.square();
        inside.extend([side48, asked, unknown]);
        fade_in_all(stage, &inside, 0.6)?;
        stage.scene_mut().bring_to_front(&[outline])?;
        why_pause(&kit, stage)?;

        kit.music_breath(stage, super::draw::END_HOLD_S)?;
        // the two squares stay for chapter 10, which answers the question
        // with the picture; it starts from them and from fifty squared at the
        // head of its column
        let fifty_squared = FIFTY * FIFTY;
        let mut keep = vec![outline];
        keep.extend(figure.square());
        let head_at = v(2.0, 3.2);
        let head = eq(&kit, stage, &format!("{}^2 = {fifty_squared}", FIFTY), 48.0, head_at)?;
        stage.move_to(head, head_at, LEFT);
        let leaving: Vec<Mob> = stage
            .scene()
            .mobjects()
            .iter()
            .copied()
            .filter(|m| !keep.contains(m))
            .collect();
        swap(stage, &leaving, &[head], 0.8)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_machine_runs_without_showing_an_answer_and_stops_on_the_exact_one() {
        let (rough, exact) = (num(cited::BETHE_ROUGH), num(cited::BETHE_EXACT));
        let running = running_readings(exact.len());
        let stopping = stopping_readings(cited::BETHE_EXACT);
        assert_eq!(stopping.len(), stopping_times().len(), "one tick a change");
        assert_eq!(stopping.last(), Some(&exact));
        for r in running.iter().chain(&stopping[..stopping.len() - 1]) {
            assert!(r != &rough && r != &exact, "{r}");
        }
        // the wheels stop one by one, left to right, and never start again:
        // the first reading from which wheel j keeps its digit comes later
        // for each wheel
        let target: Vec<char> = exact.chars().collect();
        let stops: Vec<usize> = (0..target.len())
            .map(|j| {
                stopping
                    .iter()
                    .rposition(|r| r.chars().nth(j) != Some(target[j]))
                    .map_or(0, |k| k + 1)
            })
            .collect();
        assert!(stops.windows(2).all(|w| w[0] < w[1]), "{stops:?}");
        assert!(
            stops[0] > stopping.len() / 2,
            "the wheels run before they stop: {stops:?}"
        );
    }

    #[test]
    fn the_ticks_slow_down_and_end_inside_the_window() {
        let t = stopping_times();
        assert!(t.windows(3).all(|w| w[2] - w[1] > w[1] - w[0]));
        assert!(*t.last().unwrap() < MACHINE_S - 0.3);
    }
}
