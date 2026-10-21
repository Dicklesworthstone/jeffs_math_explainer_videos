//! Chapter 01, someone hands you C. The cold open: someone hands over the
//! product of two giant matrices; can it be checked without redoing it
//! (01_1)? With coin tosses: twenty checks need a fiftieth of the
//! multiplications at a thousand rows, right answers always pass and a
//! wrong one gets through less than once in a million (01_2), then the
//! title. Start small: A, B and the answer C (01_3); an entry is a row times
//! a column, 1·0 + 2·2 + 1·3 = 7 (01_4); C has 7 there, but the product is
//! nine entries of three multiplications (01_5). At a thousand rows, how
//! many? (01_6, a prediction.) A billion, and checking by redoing costs
//! another (01_7). Why anyone checks (01_8).

use explainer_kit::props::precisely;
use explainer_kit::{Kit, Palette, fade_all, rect, v};
use fmn::prelude::*;

use super::draw::{
    OPEN_EQUALS_X, OPEN_Y, cell_box, coin_mobs, column_box, fade_in_all, fade_in_lagged, fade_out_all, fail, flat, now,
    opening, pause, read_s, row_box, say, stand, swap, tex_thousands, texture, think, toss_coins, wait_until,
};
use super::glyphs::TITLE;
use crate::facts::{self, AGAIN, BIG_SIDE, MILLION};
use crate::model::{self, A, B, C, SIDE, SPOT};
use crate::palette;

pub struct HandedAnAnswer {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from the model and facts)
const NAMES: [&str; 3] = ["A", "B", "C"];
const TIMES: &str = r"\times";
const EQUALS: &str = "=";
const ASK: &str = "?";
const RIGHT_LINE: &str = "a right C: always passes";
const REDO_NOTE: &str = "multiplications to redo the product";
const QUESTION: [&str; 2] = ["How many multiplications,", "row by column, at n = 1000?"];
const CHECK_NOTE: &str = "to check by redoing:";
/// The coins the cold open tosses over C: any landing, never read as data.
const COLD_TOSS: [bool; 6] = [true, false, true, true, false, true];

// ------------------------------------------------------------------ layout

/// 01_1, 01_2: three giant matrices, as squares of faint lines.
const GIANT_XS: [f64; 3] = [-4.6, -1.3, 3.2];
const GIANT_Y: f64 = 2.0;
const GIANT_SIDE: f64 = 2.2;
const GIANT_CELLS: usize = 12;
const GIANT_NAME_SIZE: f64 = 48.0;
const GIANT_TIMES_X: f64 = -2.95;
const GIANT_EQUALS_X: f64 = 0.95;
const GIANT_LABEL_AT: Vec3 = [0.0, 0.55, 0.0];
const GIANT_COIN_Y: f64 = 3.45;
const GIANT_COIN_PITCH: f64 = 0.42;
/// 01_2: the two bars to scale, each under its label, and the two lines.
const BAR_X: f64 = -5.0;
const BAR_W: f64 = 10.0;
const BAR_H: f64 = 0.28;
const REDO_LABEL_Y: f64 = -0.05;
const REDO_BAR_Y: f64 = -0.45;
const CHECK_LABEL_Y: f64 = -0.95;
const CHECK_BAR_Y: f64 = -1.35;
const LINE_YS: [f64; 2] = [-1.85, -2.3];
/// The title card's two lines.
const TITLE_YS: [f64; 2] = [0.45, -0.45];
const TITLE_SIZE: f64 = 44.0;
const TITLE_HOLD: f64 = 2.0;
/// 01_3 to 01_8: the panel right of the opening picture.
const PANEL_X: f64 = 4.5;
const PANEL_YS: [f64; 4] = [3.0, 2.4, 1.5, 1.05];
const PANEL_SIZE: f64 = 30.0;
const NOTE_SIZE: f64 = 20.0;
const ANSWER_YS: [f64; 2] = [2.3, 1.7];
const AGAIN_YS: [f64; 2] = [0.9, 0.45];
const THINK_AT: Vec3 = [4.5, 0.2, 0.0];

/// The entry 01_4 works out by hand, row 1 of A times column 3 of B, as two
/// lines: the products in pairs, "1\cdot 0 + 2\cdot 2 + 1\cdot 3", and
/// their values added, "= 0 + 4 + 3 = 7". Refused unless the products are
/// the facts'.
fn entry_tex() -> Result<[String; 2], String> {
    let (i, j) = SPOT;
    let terms: Vec<String> = A[i]
        .iter()
        .zip(&B)
        .map(|(x, row)| format!(r"{x}\cdot {}", row[j]))
        .collect();
    let products = facts::spot_products();
    if products.len() != terms.len() {
        return Err("the hand-worked entry has the wrong number of products".into());
    }
    let values: Vec<String> = products.iter().map(i64::to_string).collect();
    let value: i64 = products.iter().sum();
    Ok([terms.join(" + "), format!("= {} = {value}", values.join(" + "))])
}

/// The cold open's numbers, from the model: the product's multiplications
/// and twenty checks' at a thousand rows (its bars' labels and their share),
/// and the bound on twenty fresh tosses. Refused unless twenty checks are a
/// fiftieth and the bound beats a million (the line says both).
struct ColdOpen {
    redo: String,
    check: String,
    share: f64,
    wrong_line: String,
    card: String,
}

fn cold_open() -> Result<ColdOpen, String> {
    let redo = model::product_multiplications(BIG_SIDE);
    let again = u64::from(AGAIN) * model::check_multiplications(BIG_SIDE);
    if facts::ratio(again as i64, redo as i64)? != "1/50" {
        return Err("twenty checks are not a fiftieth of the product".into());
    }
    let odds = facts::again_odds()?;
    if odds <= MILLION {
        return Err(format!("1/{odds} is not under one in a million"));
    }
    Ok(ColdOpen {
        redo: format!(r"\text{{redo the product, row by column: }} {}", tex_thousands(redo)),
        check: format!(r"\text{{{AGAIN} checks: }} {}", tex_thousands(again)),
        share: again as f64 / redo as f64,
        wrong_line: format!(
            r"\text{{a wrong }} C \text{{ passes all {AGAIN} with chance }} \le \frac{{1}}{{{}}}",
            tex_thousands(odds)
        ),
        card: format!(
            "at n = {BIG_SIDE}, {AGAIN} checks take {} multiplications against {} row by column; exact \
             arithmetic; fair coins tossed after C is fixed",
            explainer_kit::thousands(again),
            explainer_kit::thousands(redo)
        ),
    })
}

/// The baseline's exact version (01_7): what the count counts, and that the
/// video compares the check with that count only.
const BASELINE_CARD: &str = "the baseline is row-by-column multiplication, n cubed multiplications for two n by n \
                             matrices; the video compares the check with that count only";

/// A bar `w` long at height `y`, from the bars' left edge. Detached.
fn bar(stage: &mut Stage<'_>, y: f64, w: f64, style: Style) -> fmn::Result<Mob> {
    let r = rect(v(BAR_X, y - BAR_H / 2.0), w, BAR_H, style)?;
    Ok(stage.arena_mut().add(r))
}

impl SceneConstruct for HandedAnAnswer {
    fn name(&self) -> &str {
        "01_handed_an_answer"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let open_words = cold_open().map_err(fail)?;
        let by_hand = entry_tex().map_err(fail)?;
        let (big_entries, _, _) = facts::spot_half(BIG_SIDE);
        let redo_big = model::product_multiplications(BIG_SIDE);
        let per_entry = facts::spot_products().len();
        let (_, redo) = model::product(&A, &B);
        if big_entries * BIG_SIDE != redo_big {
            return Err(fail(
                "a million entries of a thousand multiplications must be the billion".into(),
            ));
        }

        // 01_1: two giant matrices; the product handed over; the question
        let line = say(&kit, stage, "01_1")?;
        let colors = [palette::matrix_a(), palette::matrix_b(), palette::claim()];
        let mut giants = Vec::with_capacity(3);
        for (&x, color) in GIANT_XS.iter().zip(colors) {
            let mut square = texture(stage, v(x, GIANT_Y), GIANT_SIDE, GIANT_CELLS, color)?;
            square.push(kit.tex(stage, NAMES[giants.len()], GIANT_NAME_SIZE, color, &[], v(x, GIANT_Y))?);
            giants.push(square);
        }
        let times = kit.tex(stage, TIMES, 36.0, Palette::dim(), &[], v(GIANT_TIMES_X, GIANT_Y))?;
        let size = kit.tex(
            stage,
            &format!(r"\text{{each }} {BIG_SIDE} \times {BIG_SIDE}"),
            28.0,
            Palette::dim(),
            &[],
            GIANT_LABEL_AT,
        )?;
        let mut arriving = giants[0].clone();
        arriving.extend(&giants[1]);
        arriving.extend([times, size]);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "the product")?;
        let equals = kit.tex(stage, EQUALS, 36.0, Palette::dim(), &[], v(GIANT_EQUALS_X, GIANT_Y))?;
        let mut arriving = giants[2].clone();
        arriving.push(equals);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "Can you check it")?;
        let ask = kit.text(stage, ASK, 34.0, Palette::hero(), v(GIANT_EQUALS_X, GIANT_Y + 0.6))?;
        fade_in_all(stage, &[ask], 0.3)?;

        // 01_2: coins over C; the bars to scale and their exact version; the
        // two promises; then the title
        let line = say(&kit, stage, "01_2")?;
        let xs: Vec<f64> = (0..COLD_TOSS.len())
            .map(|k| GIANT_XS[2] + (k as f64 - (COLD_TOSS.len() as f64 - 1.0) / 2.0) * GIANT_COIN_PITCH)
            .collect();
        let coins = toss_coins(&kit, stage, &COLD_TOSS, &xs, GIANT_COIN_Y)?;
        fade_in_lagged(stage, &coin_mobs(&coins), 0.7, 0.3)?;
        line.wait_for(stage, "twenty checks")?;
        let redo_label = kit.tex_left(
            stage,
            &open_words.redo,
            26.0,
            Palette::ink(),
            &[],
            v(BAR_X, REDO_LABEL_Y),
        )?;
        let redo_track = bar(stage, REDO_BAR_Y, BAR_W, flat(Palette::faint(), 0.6))?;
        let redo_fill = bar(stage, REDO_BAR_Y, BAR_W, flat(Palette::dim(), 0.9))?;
        let check_label = kit.tex_left(
            stage,
            &open_words.check,
            26.0,
            Palette::ink(),
            &[],
            v(BAR_X, CHECK_LABEL_Y),
        )?;
        let check_track = bar(stage, CHECK_BAR_Y, BAR_W, flat(Palette::faint(), 0.6))?;
        let check_fill = bar(
            stage,
            CHECK_BAR_Y,
            BAR_W * open_words.share,
            flat(palette::matrix_a(), 0.95),
        )?;
        let card = precisely(&kit, stage, &open_words.card)?;
        fade_in_all(
            stage,
            &[
                redo_label,
                redo_track,
                redo_fill,
                check_label,
                check_track,
                check_fill,
                card,
            ],
            0.5,
        )?;
        let card_up = now(stage);
        line.wait_for(stage, "Right answers")?;
        let promises = [
            kit.text(stage, RIGHT_LINE, 22.0, palette::caught(), v(0.0, LINE_YS[0]))?,
            kit.tex(
                stage,
                &open_words.wrong_line,
                26.0,
                Palette::ink(),
                &[],
                v(0.0, LINE_YS[1]),
            )?,
        ];
        fade_in_lagged(stage, &promises, 0.8, 0.4)?;
        kit.hold(stage)?;
        wait_until(stage, card_up + read_s(&open_words.card))?;
        let mut leaving: Vec<Mob> = giants.iter().flatten().copied().collect();
        leaving.extend(coin_mobs(&coins));
        leaving.extend([times, size, equals, ask, redo_label, redo_track, redo_fill]);
        leaving.extend([check_label, check_track, check_fill, card]);
        leaving.extend(promises);
        fade_out_all(stage, &leaving, 0.5)?;
        let split = TITLE
            .find(" Without")
            .ok_or_else(|| fail("the title has no second half".into()))?;
        let (first, second) = TITLE.split_at(split);
        let title = [
            kit.text(stage, first, TITLE_SIZE, Palette::ink(), v(0.0, TITLE_YS[0]))?,
            kit.text(
                stage,
                second.trim_start(),
                TITLE_SIZE,
                Palette::ink(),
                v(0.0, TITLE_YS[1]),
            )?,
        ];
        fade_in_all(stage, &title, 0.5)?;
        pause(stage, TITLE_HOLD)?;
        fade_out_all(stage, &title, 0.5)?;

        // 01_3: start small: A and B, the answer C, the question
        let line = say(&kit, stage, "01_3")?;
        let open = opening(&kit, stage, &C)?;
        let mut arriving = open.a.mobs();
        arriving.extend(open.b.mobs());
        arriving.extend([open.names[0], open.names[1], open.times]);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "the answer someone handed you")?;
        let mut arriving = open.c.mobs();
        arriving.extend([open.names[2], open.equals]);
        fade_in_all(stage, &arriving, 0.6)?;
        line.wait_for(stage, "Is C really")?;
        let ask = kit.text(stage, ASK, 34.0, Palette::hero(), v(OPEN_EQUALS_X, OPEN_Y + 0.55))?;
        fade_in_all(stage, &[ask], 0.3)?;

        // 01_4: an entry is a row times a column; one worked by hand
        let line = say(&kit, stage, "01_4")?;
        line.wait_for(stage, "a row of A")?;
        let (si, sj) = SPOT;
        let boxes = [
            row_box(stage, &open.a, si, palette::matrix_a())?,
            column_box(stage, &open.b, sj, palette::matrix_b())?,
            cell_box(stage, &open.c, si, sj, palette::claim())?,
        ];
        fade_in_all(stage, &boxes, 0.4)?;
        line.wait_for(stage, "Multiply them in pairs")?;
        let pairs = kit.tex(
            stage,
            &by_hand[0],
            PANEL_SIZE,
            Palette::ink(),
            &[],
            v(PANEL_X, PANEL_YS[0]),
        )?;
        fade_in_all(stage, &[pairs], 0.5)?;
        line.wait_for(stage, "gives zero")?;
        let sum = kit.tex(
            stage,
            &by_hand[1],
            PANEL_SIZE,
            Palette::ink(),
            &[],
            v(PANEL_X, PANEL_YS[1]),
        )?;
        fade_in_all(stage, &[sum], 0.5)?;

        // 01_5: C has 7 there too; but nine entries of three multiplications
        let line = say(&kit, stage, "01_5")?;
        if C[si][sj] != facts::spot_products().iter().sum::<i64>() {
            return Err(fail("C's hand-worked entry must be right".into()));
        }
        let right = cell_box(stage, &open.c, si, sj, palette::caught())?;
        swap(stage, &[boxes[2]], &[right], 0.3)?;
        line.wait_for(stage, "But the product")?;
        let count = kit.tex(
            stage,
            &format!(r"{} \times {per_entry} = {redo}", SIDE * SIDE),
            PANEL_SIZE,
            Palette::ink(),
            &[],
            v(PANEL_X, PANEL_YS[2]),
        )?;
        let count_note = kit.text(stage, REDO_NOTE, NOTE_SIZE, Palette::dim(), v(PANEL_X, PANEL_YS[3]))?;
        fade_in_all(stage, &[count, count_note], 0.5)?;

        // 01_6: a thousand rows and columns: how many? The think time, with
        // no count on screen
        let line = say(&kit, stage, "01_6")?;
        fade_out_all(
            stage,
            &[boxes[0], boxes[1], right, pairs, sum, count, count_note, ask],
            0.4,
        )?;
        let n_label = kit.tex(
            stage,
            &format!("n = {BIG_SIDE}"),
            PANEL_SIZE,
            Palette::ink(),
            &[],
            v(PANEL_X, PANEL_YS[0]),
        )?;
        fade_in_all(stage, &[n_label], 0.4)?;
        line.wait_for(stage, "How many")?;
        let question = [
            kit.text(stage, QUESTION[0], 22.0, Palette::ink(), v(PANEL_X, ANSWER_YS[0]))?,
            kit.text(stage, QUESTION[1], 22.0, Palette::ink(), v(PANEL_X, ANSWER_YS[0] - 0.4))?,
        ];
        fade_in_all(stage, &question, 0.4)?;
        let q = think(&kit, stage, THINK_AT)?;

        // 01_7: a billion, written as it is said; redoing it costs another
        let line = say(&kit, stage, "01_7")?;
        let answer = [
            kit.tex(
                stage,
                &format!(
                    r"{} \text{{ entries}} \times {}",
                    tex_thousands(big_entries),
                    tex_thousands(BIG_SIDE)
                ),
                24.0,
                Palette::ink(),
                &[],
                v(PANEL_X, ANSWER_YS[0]),
            )?,
            kit.tex(
                stage,
                &format!("= {}", tex_thousands(redo_big)),
                30.0,
                palette::mistake(),
                &[],
                v(PANEL_X, ANSWER_YS[1]),
            )?,
        ];
        fade_out_all(stage, &[q], 0.2)?;
        swap(stage, &question, &answer, 0.3)?;
        let baseline = precisely(&kit, stage, BASELINE_CARD)?;
        fade_in_all(stage, &[baseline], 0.3)?;
        let baseline_up = now(stage);
        line.wait_for(stage, "another billion")?;
        let again = [
            kit.text(stage, CHECK_NOTE, NOTE_SIZE, Palette::dim(), v(PANEL_X, AGAIN_YS[0]))?,
            kit.tex(
                stage,
                &format!(r"\text{{another }} {}", tex_thousands(redo_big)),
                26.0,
                Palette::ink(),
                &[],
                v(PANEL_X, AGAIN_YS[1]),
            )?,
        ];
        fade_in_all(stage, &again, 0.4)?;

        // 01_8: why anyone checks; the baseline card is read, then the fade
        say(&kit, stage, "01_8")?;
        let up = now(stage);
        stand(&kit, stage, up)?;
        wait_until(stage, baseline_up + read_s(BASELINE_CARD))?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The words on screen say what the model says.
    #[test]
    fn the_screen_words_are_built_from_the_model() {
        assert_eq!(
            entry_tex().unwrap(),
            [
                r"1\cdot 0 + 2\cdot 2 + 1\cdot 3".to_owned(),
                "= 0 + 4 + 3 = 7".to_owned()
            ]
        );
        let c = cold_open().unwrap();
        assert_eq!(c.redo, r"\text{redo the product, row by column: } 1{,}000{,}000{,}000");
        assert_eq!(c.check, r"\text{20 checks: } 20{,}000{,}000");
        assert!((c.share * 50.0 - 1.0).abs() < 1e-12, "{}", c.share);
        assert_eq!(
            c.wrong_line,
            r"\text{a wrong } C \text{ passes all 20 with chance } \le \frac{1}{1{,}048{,}576}"
        );
        assert_eq!(
            c.card,
            "at n = 1000, 20 checks take 20,000,000 multiplications against 1,000,000,000 row by column; exact \
             arithmetic; fair coins tossed after C is fixed"
        );
        assert!(TITLE.contains(" Without"));
    }

    /// The cards name the example, the unit and the conditions, and fit two
    /// lines with no claim about every method of multiplying.
    #[test]
    fn the_cards_fit_their_lines() {
        let c = cold_open().unwrap();
        assert!(BASELINE_CARD.contains("row-by-column multiplication"));
        assert!(!BASELINE_CARD.contains("known"), "{BASELINE_CARD}");
        for text in [c.card.as_str(), BASELINE_CARD] {
            for width in [80, 76] {
                let lines = explainer_kit::animatic::wrap(text, width, 3);
                assert!(lines.len() <= 2, "{} lines at {width}: {text}", lines.len());
            }
            assert!(read_s(text) < 12.0, "{text}");
            assert!(!text.contains('^'), "{text}");
        }
    }

    /// Every phrase the scene waits for is in its line of script.tsv.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("01_1", "the product"),
            ("01_1", "Can you check it"),
            ("01_2", "twenty checks"),
            ("01_2", "Right answers"),
            ("01_3", "the answer someone handed you"),
            ("01_3", "Is C really"),
            ("01_4", "a row of A"),
            ("01_4", "Multiply them in pairs"),
            ("01_4", "gives zero"),
            ("01_5", "But the product"),
            ("01_6", "How many"),
            ("01_7", "another billion"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// The layout keeps its pieces apart and inside the title-safe area.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use super::super::draw::COIN_R;
        use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
        const CARD_2: f64 = -FRAME_Y_RADIUS + 0.2 + 0.3 + 0.36 * 2.0;
        // the giant squares apart, inside the frame, the coins over C
        const {
            assert!(
                GIANT_XS[0] - GIANT_SIDE / 2.0 > -FRAME_X_RADIUS + 0.15,
                "squares inside"
            )
        };
        const { assert!(GIANT_XS[2] + GIANT_SIDE / 2.0 < FRAME_X_RADIUS - 0.15, "squares inside") };
        const { assert!(GIANT_XS[1] - GIANT_XS[0] > GIANT_SIDE + 0.6, "room for the times sign") };
        const { assert!(GIANT_XS[2] - GIANT_XS[1] > GIANT_SIDE + 0.6, "room for the equals sign") };
        const {
            assert!(
                GIANT_COIN_Y - COIN_R > GIANT_Y + GIANT_SIDE / 2.0 + 0.05,
                "coins over C"
            )
        };
        const { assert!(GIANT_COIN_Y + COIN_R < FRAME_Y_RADIUS - 0.15, "coins inside") };
        const { assert!(GIANT_COIN_PITCH > 2.0 * COIN_R, "coins apart") };
        // the label under the squares, the bars under it, the lines over the card
        const {
            assert!(
                GIANT_LABEL_AT[1] + 0.2 < GIANT_Y - GIANT_SIDE / 2.0,
                "label under the squares"
            )
        };
        const { assert!(REDO_LABEL_Y + 0.2 < GIANT_LABEL_AT[1] - 0.2, "bars under the label") };
        const { assert!(CHECK_BAR_Y - BAR_H / 2.0 > LINE_YS[0] + 0.2, "lines under the bars") };
        const { assert!(LINE_YS[1] - 0.35 > CARD_2, "lines over the card") };
        const { assert!(BAR_X > -FRAME_X_RADIUS + 0.15 && BAR_X + BAR_W < FRAME_X_RADIUS - 0.15) };
        // the panel right of C, the think ring under the question
        const { assert!(THINK_AT[1] + 0.62 < ANSWER_YS[0] - 0.4 - 0.2, "ring under the question") };
        const { assert!(AGAIN_YS[1] - 0.25 > CARD_2, "panel over the card") };
        const { assert!(TITLE_YS[0] - TITLE_YS[1] > 0.6, "title lines apart") };
    }
}
