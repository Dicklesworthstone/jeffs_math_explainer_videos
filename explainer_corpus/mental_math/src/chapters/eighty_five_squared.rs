//! Chapter 01, eighty-five squared: a quiz before anything is taught (01_1)
//! and its answer in three short sentences (01_2); the title (01_3); the
//! usual way set out as on paper, two rows and three carries (01_4); the
//! same numbers held in the head, where a row is lost (01_5); the trick
//! holds one number (01_6); and it rests on a picture, as every trick here
//! does (01_7).

use explainer_kit::{Kit, Palette, Timed, fade_all, play, v};
use fmn::prelude::*;

use super::draw::{
    Board, MoveFigure, QUIZ_AT, QuizLook, Sheet, corner, eq, fade_in_all, fade_in_lagged, fade_out_all, fail, fly,
    glyphs, hundreds, in_your_head, line_style, middle_of, num, panel, paper_ink, quiz, say, strip, tile_row,
    your_turn,
};
use super::glyphs::TITLE;
use crate::facts::shown;
use crate::model::{ends_in_five, one_move, paper};

pub struct EightyFiveSquared {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const ROWS: &str = "rows";
const CARRIES: &str = "carries";
const SO_FAR: &str = "the sum so far";
const A_CARRY: &str = "a carry";
const YOUR_TURN: &str = "your turn";
/// Where the title breaks into its two lines.
const TITLE_BREAK: &str = " That ";

/// The working under the quiz card, and the numbers flown into its answer.
const WORK_PT: f64 = 64.0;
/// The type on the sheet of paper, and its small carried digits.
const SHEET_PT: f64 = 56.0;
const CARRY_PT: f64 = 30.0;

impl SceneConstruct for EightyFiveSquared {
    fn name(&self) -> &str {
        "01_eighty_five_squared"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let hook = shown::HOOK;
        let five = ends_in_five(hook).ok_or_else(|| fail(format!("{hook} does not end in five")))?;
        let m = one_move(hook, hook % 10).ok_or_else(|| fail(format!("no strip comes off a square of {hook}")))?;
        let (blue, orange, ink) = (hundreds(), corner(), Palette::ink());

        // 01_1: the quiz card, cold: no label, no pause line
        let look = QuizLook {
            at: QUIZ_AT,
            label: None,
            pause_line: false,
            lands: true,
            scale: 1.0,
        };
        let card = your_turn(&kit, stage, "01_1", look)?;

        // 01_2: eight times nine is seventy-two; then write twenty-five
        let line = say(&kit, stage, "01_2")?;
        let digits = glyphs(stage, card.answer);
        let split = num(five.hundreds).len();
        if digits.len() <= split {
            return Err(fail(format!("{} has no digits after its hundreds", five.square)));
        }
        let (lead_at, tail_at) = (middle_of(stage, &digits[..split])?, middle_of(stage, &digits[split..])?);
        line.wait_sentence(stage, 1)?;
        let work_y = -3.05;
        let product_src = format!(r"{} \times {} = {}", five.n, five.n + 1, five.hundreds);
        let product = kit.tex(
            stage,
            &product_src,
            WORK_PT,
            ink,
            &[(&num(five.hundreds), blue)],
            v(-1.3, work_y),
        )?;
        play!(stage; fade_in(stage.arena_mut(), product, v(0.0, -0.12), 1.0)?.rt(0.5));
        line.wait_sentence(stage, 2)?;
        let then = kit.tex(stage, &num(m.corner()), WORK_PT, orange, &[], v(2.2, work_y))?;
        play!(stage; fade_in(stage.arena_mut(), then, v(0.3, 0.0), 1.0)?.rt(0.4));
        // the 72 and the 25 slide up into the 7225 already there, which
        // takes their colours as they arrive
        let product_glyphs = glyphs(stage, product);
        let from = middle_of(stage, &product_glyphs[product_glyphs.len() - split..])?;
        let lead = kit.tex(stage, &num(five.hundreds), WORK_PT, blue, &[], from)?;
        stage.add_to_scene(lead)?;
        play!(stage;
            fly(lead, lead_at, look.pt() / WORK_PT, 0.8)?,
            fly(then, tail_at, look.pt() / WORK_PT, 0.8)?,
        );
        for &g in &digits[..split] {
            stage.set_color(g, blue, None, true);
        }
        for &g in &digits[split..] {
            stage.set_color(g, orange, None, true);
        }
        stage.remove_from_scene(lead);
        stage.remove_from_scene(then);
        kit.hold(stage)?;

        // 01_3: the title; the motif's question, left open
        let mut gone = card.mobs();
        gone.push(product);
        fade_out_all(stage, &gone, 0.6)?;
        let (head, tail) = TITLE
            .split_once(TITLE_BREAK)
            .ok_or_else(|| fail("the title has no place to break".into()))?;
        let tail = format!("{}{tail}", TITLE_BREAK.trim_start());
        let title = [
            kit.text_styled(stage, head, 76.0, ink, v(0.0, 0.55), true, false)?,
            kit.text(stage, &tail, 52.0, Palette::dim(), v(0.0, -0.6))?,
        ];
        fade_in_all(stage, &title, 0.8)?;
        kit.listen(stage, 3.0, "01_eighty_five_squared:01_3")?;
        fade_out_all(stage, &title, 0.6)?;

        // 01_4: the usual way, as on a sheet of paper
        let line = say(&kit, stage, "01_4")?;
        let p = paper(hook, hook);
        if p.rows.len() != 2 || p.carried.len() != 3 {
            return Err(fail(format!(
                "the sheet is laid out for two rows and three carries: {p:?}"
            )));
        }
        let times = format!("× {hook}");
        let rows = [num(hook), times, num(p.rows[0]), num(p.rows[1]), num(p.product)];
        let row_refs: Vec<&str> = rows.iter().map(String::as_str).collect();
        let sheet = Sheet::build(&kit, stage, v(-2.4, 0.0), &row_refs, &[1, 3], SHEET_PT)?;
        line.wait_sentence(stage, 1)?;
        fade_in_all(stage, &[sheet.back, sheet.rows[0], sheet.rows[1], sheet.rules[0]], 0.6)?;
        line.wait_sentence(stage, 2)?;
        // the carries, small, where school writes them: the two of the rows
        // over the top number's tens, the one of the sum over the thousands
        let digit_w = stage.get_width(sheet.rows[3]) / rows[3].chars().count() as f64;
        let column = |k: f64| sheet.right - digit_w * (k + 0.5);
        let small =
            |kit: &Kit, stage: &mut Stage<'_>, d: u64, at: Vec3| kit.mono(stage, &num(d), CARRY_PT, paper_ink(), at);
        let over = sheet.row_ys[0] + 0.68 * sheet.line_h;
        let carried = [
            small(&kit, stage, p.carried[0], v(column(1.0) - 0.3 * digit_w, over))?,
            small(&kit, stage, p.carried[1], v(column(1.0) + 0.3 * digit_w, over))?,
            small(
                &kit,
                stage,
                p.carried[2],
                v(column(3.0), sheet.row_ys[2] + 0.22 * sheet.line_h),
            )?,
        ];
        fade_in_all(stage, &[sheet.rows[2], carried[0]], 0.5)?;
        fade_in_all(stage, &[sheet.rows[3], carried[1]], 0.5)?;
        line.wait_for(stage, "two rows")?;
        let counts_x = 3.2;
        let row_count = kit.text(stage, &format!("{} {ROWS}", p.rows.len()), 60.0, ink, v(counts_x, 0.75))?;
        play!(stage; fade_in(stage.arena_mut(), row_count, v(-0.2, 0.0), 1.0)?.rt(0.4));
        line.wait_for(stage, "three carries")?;
        fade_in_all(stage, &[sheet.rules[1], sheet.rows[4], carried[2]], 0.5)?;
        let carry_count = kit.text(
            stage,
            &format!("{} {CARRIES}", p.carries),
            60.0,
            ink,
            v(counts_x, -0.55),
        )?;
        play!(stage; fade_in(stage.arena_mut(), carry_count, v(-0.2, 0.0), 1.0)?.rt(0.4));

        // 01_5: in your head there is no paper to remember the rows
        let line = say(&kit, stage, "01_5")?;
        line.wait_sentence(stage, 1)?;
        let mut on_paper = sheet.mobs();
        on_paper.extend(carried);
        on_paper.extend([row_count, carry_count]);
        fade_out_all(stage, &on_paper, 0.6)?;
        let head_at = v(0.0, -0.2);
        let head = in_your_head(&kit, stage, head_at, 10.5, 5.0)?;
        // what is in hand at the worst moment: both rows, the sum so far and
        // a carry
        let so_far = p.product % 1000;
        let held = [
            kit.mono(stage, &num(p.rows[0]), 84.0, ink, v(-2.9, 1.0))?,
            kit.mono(stage, &num(p.rows[1]), 84.0, ink, v(2.2, 1.2))?,
            kit.mono(stage, &num(so_far), 84.0, ink, v(-1.5, -1.0))?,
            kit.mono(stage, &num(p.carried[2]), 84.0, ink, v(3.2, -0.8))?,
        ];
        let notes = [
            kit.text(stage, SO_FAR, 30.0, Palette::dim(), v(-1.5, -1.8))?,
            kit.text(stage, A_CARRY, 30.0, Palette::dim(), v(3.2, -1.6))?,
        ];
        fade_in_all(stage, &head, 0.5)?;
        fade_in_lagged(stage, &held, 1.6, 0.5)?;
        fade_in_all(stage, &notes, 0.4)?;
        line.wait_for(stage, "while you add them")?;
        // the first row slips away
        play!(stage; held[0].animate().set_anim_args(AnimateArgs {
            run_time: Some(1.2),
            ..AnimateArgs::default()
        })?.set_opacity(0.12)?);

        // 01_6: the trick holds one number
        let line = say(&kit, stage, "01_6")?;
        let mut all_held = held.to_vec();
        all_held.extend(notes);
        fade_out_all(stage, &all_held, 0.5)?;
        line.wait_sentence(stage, 1)?;
        let whole = eq(&kit, stage, &num(five.square), 150.0, head_at)?;
        let whole_digits = glyphs(stage, whole);
        let (one_at, rest_at) = (
            middle_of(stage, &whole_digits[..split])?,
            middle_of(stage, &whole_digits[split..])?,
        );
        let one = eq(&kit, stage, &num(five.hundreds), 150.0, one_at)?;
        let rest = kit.tex(stage, &num(m.corner()), 150.0, orange, &[], rest_at)?;
        play!(stage; fade_in(stage.arena_mut(), one, ORIGIN, 1.0)?.rt(0.5));
        line.wait_end(stage)?;
        play!(stage; fade_in(stage.arena_mut(), rest, v(0.5, 0.0), 1.0)?.rt(0.6));

        // 01_7: it rests on a picture; every trick here comes with its own
        let line = say(&kit, stage, "01_7")?;
        fade_out_all(stage, &[head[0], head[1], one, rest], 0.5)?;
        let side = m.side as f64;
        let unit = 3.0 / side;
        let board = Board::new(v(-side * unit / 2.0 - 0.2, -1.6), unit);
        let figure = MoveFigure::build(stage, board, m, hundreds(), strip(), 0.0)?;
        let mut square = figure.square();
        square.push(figure.cut);
        fade_in_all(stage, &square, 0.6)?;
        stage.remove_from_scene(figure.cut);
        stage
            .scene_mut()
            .bring_to_front(&[figure.strip.fill, figure.strip.edge])?;
        play!(stage; figure.strip.turn_to(1.0)?);
        stage.add_to_scene(figure.seam)?;
        figure.show_corner(stage, 0.4)?;
        line.wait_sentence(stage, 2)?;
        // the six small pictures of what is coming (the tiles of chapter 12)
        fade_out_all(stage, &figure.mobs(), 0.5)?;
        let tiles = tile_row(stage, 0.75, 1.8, 0.3)?;
        let tile_mobs: Vec<Mob> = tiles
            .iter()
            .map(|t| super::draw::group(stage, &t.mobs()))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &tile_mobs, 2.4, 0.4)?;
        line.wait_sentence(stage, 3)?;
        // and the quiz card, in small
        let at = v(0.0, -1.75);
        let mini = Rectangle::new()
            .width(2.6)
            .height(1.5)
            .corner_radius(0.12)
            .style(Style::default().fill(panel(), 0.96).stroke(quiz(), 2.5, 1.0))
            .build()?
            .moved_to(at);
        let mini = stage.arena_mut().add(mini);
        let ring = stage.arena_mut().add(
            Circle::new()
                .radius(0.26)
                .arc_center(v(at[0], at[1] - 0.2))
                .style(line_style(quiz(), 5.0))
                .build(),
        );
        let words = kit.text(stage, YOUR_TURN, 24.0, quiz(), v(at[0], at[1] + 0.42))?;
        fade_in_all(stage, &[mini, ring, words], 0.6)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 0.8)?;
        kit.finish(stage)
    }
}
