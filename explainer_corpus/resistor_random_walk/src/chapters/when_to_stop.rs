//! Chapter 11, when to stop: for programmers, the weighted averaging round
//! after round, every junction from the round before (11_1); two runs, from
//! all zeros and from all ones at the middle junctions, the top and the
//! bottom held in both, the true values between them (11_2); within a
//! thousandth of each other after nine rounds (11_3); the prediction,
//! beside an island (11_4); there the runs never move, the sign that its
//! voltage is not unique (11_5); on any finite network whose middle
//! junctions reach an end, the voltage is the walker's chance (11_6).

use explainer_kit::props::{CodePanel, precisely};
use explainer_kit::{Kit, Palette, fade_all, v};
use fmn::prelude::*;

use super::draw::{
    END_LABEL_GAP, JUNCTION_R, LABEL_GAP, LABEL_SIZE, Picture, WIRE_W, fade_in_all, fade_in_lagged, fade_out_all,
    fail, flash_all, flat, line_style, outside_link_spot, pause, say, small_tex, tex, think, wait_until,
    weighted_picture,
};
use crate::facts::{self, RUN_ROWS};
use crate::model::{self, BOTTOM, LEFT, LINE_START, Q, RIGHT, TOP};
use crate::palette;

pub struct WhenToStop {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const BOTH: &str = "voltage or chance";
const HEADERS: [&str; 4] = ["round", "left", "right", "bigger gap"];
const ISLAND: &str = "an island";
const UNKNOWN: &str = "?";
const SAME: &str = "voltage = chance";
const GAP_NOTE: &str = "a gap of 1, every round";
/// The rows between round 2 and round 8, left out of the table.
const SKIPPED: &str = "...";

/// The "for programmers" panel (11_1): the top and the bottom held in both
/// runs, every middle junction updated at once from the round before, and
/// the guarantee's arithmetic. Typewriter text, ASCII only.
const CODE: [&str; 9] = [
    "lo[top] = hi[top] = 1; lo[bottom] = hi[bottom] = 0",
    "lo[j] = 0; hi[j] = 1 at every middle junction j",
    "repeat:",
    "  for every middle junction j:",
    "    new_lo[j] = sum(c[j,k] * lo[k]) / sum(c[j,k])",
    "    new_hi[j] = sum(c[j,k] * hi[k]) / sum(c[j,k])",
    "  lo[j], hi[j] = new_lo[j], new_hi[j] for every j",
    "until max(hi[j] - lo[j]) < tolerance",
    "# exact fractions; floats: round lo down, hi up",
];
const CODE_PT: f64 = 16.0;
const CODE_W: f64 = 6.6;

/// 11_2's exact version (card I): the bracket's guarantee.
const CARD_I: &str = "positive weights keep order: in exact arithmetic the run from 0 only rises, the run from 1 only \
                      falls, and the answer lies between. Stop when they differ by less than the tolerance at every \
                      middle junction.";
/// 11_6's exact version (card J): the theorem's hypotheses.
const CARD_J: &str = "finite network; conductances positive and finite; a path of wires from each middle junction to \
                      the top or bottom; top 1, bottom 0.";

/// Where the picture stands; the table on the right (its columns, header
/// and rows, the "..." between rounds 2 and 8 taking a row); the island and
/// its countdown, as chapter 09 placed them; the closing rows.
const PICTURE_AT: [f64; 2] = [-4.0, 0.3];
const COLUMN_X: [f64; 4] = [0.45, 2.05, 4.45, 6.25];
const HEADER_Y: f64 = 2.85;
const ROW0_Y: f64 = 2.3;
const ROW_STEP: f64 = 0.55;
const CELL_SIZE: f64 = 24.0;
const HEADER_SIZE: f64 = 20.0;
const UNDER_X: f64 = 5.95;
const UNDER_Y: f64 = -1.1;
const ISLAND_AT: [[f64; 2]; 2] = [[2.4, 0.6], [5.4, 0.6]];
const ISLAND_LABEL_DY: f64 = 0.6;
const ISLAND_NAME_DY: f64 = 1.3;
const ISLAND_NOTE_Y: [f64; 2] = [-0.2, -0.7];
const THINK_AT: [f64; 2] = [3.9, -1.45];
const ISLAND_ROUNDS: usize = 4;
const RECAP_X: f64 = 3.2;
const RECAP_Y: [f64; 4] = [2.2, 1.5, 0.9, 0.3];
/// How far a link's conductance sits from it, outward; a lower-third's fade
/// in and out; a round of the island's runs; the beat after the last line.
const LINK_GAP: f64 = 0.35;
const CARD_IN: f64 = 0.4;
const CARD_OUT: f64 = 0.3;
const ROUND_S: f64 = 0.6;
const END_BEAT: f64 = 0.6;

/// Seconds a lower-third needs on screen to be read: fifteen characters a
/// second, from the moment it is fully in.
fn read_s(text: &str) -> f64 {
    text.chars().count() as f64 / 15.0
}

/// A junction's two runs as TeX: "low to high".
fn bracket(low: Q, high: Q) -> String {
    format!(r"{}\text{{ to }}{}", small_tex(low), small_tex(high))
}

/// 11_2, 11_3: the table's rows for the rounds `RUN_ROWS`: round, the
/// left's two runs, the right's, and the bigger gap.
fn table_rows(low: &[Vec<Q>], high: &[Vec<Q>]) -> Result<Vec<[String; 4]>, String> {
    RUN_ROWS
        .iter()
        .map(|&n| -> Result<[String; 4], String> {
            let (l, h) = (low.get(n).ok_or("too few rounds")?, high.get(n).ok_or("too few rounds")?);
            let gap = facts::widths(low, high, n, &[LEFT, RIGHT])
                .into_iter()
                .max()
                .ok_or("no gap")?;
            Ok([
                n.to_string(),
                bracket(l[LEFT], h[LEFT]),
                bracket(l[RIGHT], h[RIGHT]),
                small_tex(gap),
            ])
        })
        .collect()
}

/// 11_6: the three networks the video solved, each a voltage that is a
/// chance: the line's middle junctions, the bridge's, these wires'.
fn recap_rows() -> Result<[String; 3], String> {
    let line = facts::line()?.walk()?.top;
    let middle: Vec<String> = line[1..line.len() - 1].iter().map(Q::to_string).collect();
    let bridge = facts::chances()?;
    let unequal = facts::unequal()?.walk()?.top;
    if line[LINE_START] != Q::new(1, 5) {
        return Err(format!("the line's walker reaches the top first with chance {}", line[LINE_START]));
    }
    Ok([
        format!("a line of {} wires: {}", model::LINE_WIRES, middle.join(", ")),
        format!("the bridge: {} and {}", bridge[LEFT], bridge[RIGHT]),
        format!("these wires: {} and {}", unequal[LEFT], unequal[RIGHT]),
    ])
}

/// A junction's label on `pic`. Detached.
fn label(kit: &Kit, stage: &mut Stage<'_>, pic: &Picture, x: usize, src: &str) -> fmn::Result<Mob> {
    let gap = if x == TOP || x == BOTTOM { LABEL_GAP } else { END_LABEL_GAP };
    kit.tex(stage, src, LABEL_SIZE, Palette::ink(), &[], pic.label_spot(x, gap))
}

/// The y of the table's display row `i` (row 3 is the "...").
fn row_y(i: usize) -> f64 {
    ROW0_Y - ROW_STEP * i as f64
}

/// A table cell, TeX at column `c` and height `y`. Detached.
fn cell(kit: &Kit, stage: &mut Stage<'_>, src: &str, c: usize, y: f64) -> fmn::Result<Mob> {
    kit.tex(stage, src, CELL_SIZE, Palette::ink(), &[], v(COLUMN_X[c], y))
}

/// The island's junction `i`'s centre.
fn island_spot(i: usize) -> Vec3 {
    v(ISLAND_AT[i][0], ISLAND_AT[i][1])
}

/// The point midway between the island's junctions, at height `y`.
fn island_under(y: f64) -> Vec3 {
    v((ISLAND_AT[0][0] + ISLAND_AT[1][0]) / 2.0, y)
}

/// The island: its two junctions, faint, and its one wire. Detached.
fn island(stage: &mut Stage<'_>) -> Vec<Mob> {
    let (p, q) = (island_spot(0), island_spot(1));
    let path = vec![v(p[0] + JUNCTION_R, p[1]), v(q[0] - JUNCTION_R, q[1])];
    let wire = stage
        .arena_mut()
        .add(polyline(path).style(line_style(palette::wire(), WIRE_W)).build());
    let mut out = vec![wire];
    for at in [p, q] {
        let dot = Circle::new()
            .radius(JUNCTION_R)
            .style(flat(palette::unknown(), 1.0))
            .build()
            .moved_to(at);
        out.push(stage.arena_mut().add(dot));
    }
    out
}

impl SceneConstruct for WhenToStop {
    fn name(&self) -> &str {
        "11_when_to_stop"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let unequal = facts::unequal().map_err(fail)?;
        let held = unequal.kirchhoff(facts::BATTERY[0], facts::BATTERY[1]).map_err(fail)?;
        let (low, high) = unequal.two_runs(facts::RUN_CHECK_ROUNDS).map_err(fail)?;
        let rows = table_rows(&low, &high).map_err(fail)?;
        let within = facts::runs_within(&low, &high, &unequal.middle())
            .ok_or_else(|| fail("the two runs never come within the tolerance".into()))?;
        let side = model::beside_island(&unequal).map_err(fail)?;
        let cut = side.cut_off();
        let (island_low, island_high) = side.two_runs(ISLAND_ROUNDS).map_err(fail)?;

        // the unequal wires as chapter 10 left them, their values solved
        let colors: Vec<Srgb> = held.iter().map(|&q| palette::value(q)).collect();
        let pic = weighted_picture(stage, &unequal, PICTURE_AT, &colors)?;
        let name = kit.text(stage, BOTH, 26.0, Palette::dim(), pic.name_spot())?;
        let values: Vec<Mob> = (0..unequal.size())
            .map(|x| label(&kit, stage, &pic, x, &tex(held[x])))
            .collect::<fmn::Result<_>>()?;
        let mut numbers = Vec::new();
        for w in unequal.wires() {
            numbers.push(kit.tex(
                stage,
                &tex(w.conductance),
                28.0,
                Palette::ink(),
                &[],
                outside_link_spot(&pic, w.a, w.b, LINK_GAP),
            )?);
        }
        let mut arriving = pic.mobs();
        arriving.push(name);
        arriving.extend(&values);
        arriving.extend(&numbers);
        fade_in_all(stage, &arriving, 0.6)?;

        // 11_1: the code panel slides in
        say(&kit, stage, "11_1")?;
        let panel = CodePanel::build(&kit, stage, &CODE, CODE_PT, CODE_W)?;
        stage.add_to_scene(panel.group)?;
        stage.play(panel.slide_in(0.8)?)?;

        // 11_2: the panel goes; the table of the two runs, rounds 0 to 2;
        // the bracket's guarantee (card I)
        let line = say(&kit, stage, "11_2")?;
        fade_out_all(stage, &[panel.group], 0.3)?;
        let mut table = Vec::new();
        for (c, h) in HEADERS.iter().enumerate() {
            table.push(kit.text(stage, h, HEADER_SIZE, Palette::dim(), v(COLUMN_X[c], HEADER_Y))?);
        }
        let mut first = Vec::new();
        for (i, cells) in rows.iter().take(3).enumerate() {
            for (c, src) in cells.iter().enumerate() {
                first.push(cell(&kit, stage, src, c, row_y(i))?);
            }
        }
        let mut arriving = table.clone();
        arriving.extend(&first);
        fade_in_lagged(stage, &arriving, 0.9, 0.2)?;
        table.extend(&first);
        line.wait_for(stage, "never swaps the order")?;
        let card = precisely(&kit, stage, CARD_I)?;
        fade_in_all(stage, &[card], CARD_IN)?;
        let card_read = stage.scene().time().to_f64() + read_s(CARD_I);

        // 11_3: rounds 8 and 9 under a "..."; the bigger gap under a
        // thousandth at round 9; the answer between the runs
        let line = say(&kit, stage, "11_3")?;
        line.wait_for(stage, "within a thousandth")?;
        let mut later = vec![kit.text(stage, SKIPPED, CELL_SIZE, Palette::dim(), v(COLUMN_X[1], row_y(3)))?];
        let mut last_gap = None;
        for (k, cells) in rows.iter().skip(3).enumerate() {
            for (c, src) in cells.iter().enumerate() {
                let m = cell(&kit, stage, src, c, row_y(4 + k))?;
                if c == 3 {
                    last_gap = Some(m);
                }
                later.push(m);
            }
        }
        fade_in_lagged(stage, &later, 0.6, 0.3)?;
        table.extend(&later);
        line.wait_for(stage, "after nine rounds")?;
        if RUN_ROWS.last() != Some(&within) {
            return Err(fail(format!("the table ends at round {:?}, not round {within}", RUN_ROWS.last())));
        }
        let under = kit.tex(
            stage,
            &format!(r"\text{{under }}{}", small_tex(facts::tolerance())),
            CELL_SIZE,
            Palette::dim(),
            &[],
            v(UNDER_X, UNDER_Y),
        )?;
        fade_in_all(stage, &[under], 0.3)?;
        table.push(under);
        let gap_mob = last_gap.ok_or_else(|| fail("no gap cell".into()))?;
        flash_all(stage, &[gap_mob, under], 0.6)?;
        line.wait_for(stage, "the answer lies between them")?;
        flash_all(stage, &[values[LEFT], values[RIGHT]], 0.6)?;

        // 11_4: the table goes; an island beside the network, its runs
        // hidden; the card stays while the question is asked and goes as it
        // ends; the countdown under the island
        let question = say(&kit, stage, "11_4")?;
        fade_out_all(stage, &table, 0.3)?;
        let drawn = island(stage);
        let asked = [
            kit.text(
                stage,
                UNKNOWN,
                LABEL_SIZE,
                Palette::hero(),
                v(ISLAND_AT[0][0], ISLAND_AT[0][1] + ISLAND_LABEL_DY),
            )?,
            kit.text(
                stage,
                UNKNOWN,
                LABEL_SIZE,
                Palette::hero(),
                v(ISLAND_AT[1][0], ISLAND_AT[1][1] + ISLAND_LABEL_DY),
            )?,
        ];
        let island_name = kit.text(
            stage,
            ISLAND,
            26.0,
            Palette::dim(),
            island_under(ISLAND_AT[0][1] + ISLAND_NAME_DY),
        )?;
        let mut arriving = drawn.clone();
        arriving.extend(asked);
        arriving.push(island_name);
        fade_in_all(stage, &arriving, 0.5)?;
        wait_until(stage, card_read.max(question.start + question.len - CARD_OUT))?;
        fade_out_all(stage, &[card], CARD_OUT)?;
        let countdown = think(&kit, stage, v(THINK_AT[0], THINK_AT[1]))?;

        // 11_5: the island's runs never move, round after round; its gap
        // stays a whole 1
        let line = say(&kit, stage, "11_5")?;
        let runs_at = |n: usize| -> Vec<String> {
            cut.iter()
                .map(|&x| bracket(island_low[n][x], island_high[n][x]))
                .collect()
        };
        let place = |i: usize| v(ISLAND_AT[i][0], ISLAND_AT[i][1] + ISLAND_LABEL_DY);
        let mut shown: Vec<Mob> = Vec::new();
        for (i, src) in runs_at(0).iter().enumerate() {
            shown.push(kit.tex(stage, src, CELL_SIZE, Palette::ink(), &[], place(i))?);
        }
        fade_out_all(stage, &[countdown, asked[0], asked[1]], 0.15)?;
        fade_in_all(stage, &shown, 0.15)?;
        line.wait_for(stage, "only average each other")?;
        flash_all(stage, &drawn[..1], 0.6)?;
        line.wait_for(stage, "its zeros stay zeros")?;
        let note = kit.text(stage, GAP_NOTE, 22.0, Palette::dim(), island_under(ISLAND_NOTE_Y[1]))?;
        fade_in_all(stage, &[note], 0.3)?;
        let mut counter: Option<Mob> = None;
        for n in 1..=ISLAND_ROUNDS {
            let round = kit.text(
                stage,
                &format!("round {n}"),
                22.0,
                Palette::dim(),
                island_under(ISLAND_NOTE_Y[0]),
            )?;
            let mut next = Vec::new();
            for (i, src) in runs_at(n).iter().enumerate() {
                next.push(kit.tex(stage, src, CELL_SIZE, Palette::ink(), &[], place(i))?);
            }
            let mut old = shown.clone();
            old.extend(counter);
            fade_out_all(stage, &old, ROUND_S / 4.0)?;
            let mut new = next.clone();
            new.push(round);
            fade_in_all(stage, &new, ROUND_S / 4.0)?;
            pause(stage, ROUND_S / 2.0)?;
            shown = next;
            counter = Some(round);
        }

        // 11_6: the island goes; the three networks solved, each a voltage
        // that is a chance; the hypotheses (card J)
        let line = say(&kit, stage, "11_6")?;
        let mut going = drawn.clone();
        going.extend(&shown);
        going.extend(counter);
        going.extend([note, island_name]);
        fade_out_all(stage, &going, 0.3)?;
        line.wait_for(stage, "the voltage is the walker's chance")?;
        let mut recap = vec![kit.text(stage, SAME, 24.0, Palette::dim(), v(RECAP_X, RECAP_Y[0]))?];
        for (i, text) in recap_rows().map_err(fail)?.iter().enumerate() {
            recap.push(kit.text(stage, text, 22.0, Palette::ink(), v(RECAP_X, RECAP_Y[i + 1]))?);
        }
        fade_in_lagged(stage, &recap, 0.9, 0.3)?;
        line.wait_for(stage, "One rule")?;
        let hypotheses = precisely(&kit, stage, CARD_J)?;
        fade_in_all(stage, &[hypotheses], CARD_IN)?;
        let hypotheses_read = stage.scene().time().to_f64() + read_s(CARD_J);
        wait_until(stage, line.start + line.len)?;
        pause(stage, END_BEAT)?;
        wait_until(stage, hypotheses_read)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::*;

    /// Every phrase the scene waits for is in its line of script.tsv, once.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).unwrap();
        for (id, phrase) in [
            ("11_2", "never swaps the order"),
            ("11_3", "within a thousandth"),
            ("11_3", "after nine rounds"),
            ("11_3", "the answer lies between them"),
            ("11_5", "only average each other"),
            ("11_5", "its zeros stay zeros"),
            ("11_6", "the voltage is the walker's chance"),
            ("11_6", "One rule"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert_eq!(text.matches(phrase).count(), 1, "{id} must say {phrase:?} once: {text}");
        }
    }

    /// The table is the facts' two runs, the round it stops at the first
    /// within a thousandth; the code holds the top at 1 and the bottom at 0
    /// in both runs and updates every junction from the round before; the
    /// island's runs never move.
    #[test]
    fn the_table_and_the_panel_are_the_facts() {
        let unequal = facts::unequal().unwrap();
        let (low, high) = unequal.two_runs(facts::RUN_CHECK_ROUNDS).unwrap();
        let rows = table_rows(&low, &high).unwrap();
        let want = [
            ["0", r"0\text{ to }1", r"0\text{ to }1", "1"],
            [
                "1",
                r"\tfrac{1}{4}\text{ to }\tfrac{3}{4}",
                r"\tfrac{1}{5}\text{ to }\tfrac{3}{5}",
                r"\tfrac{1}{2}",
            ],
            [
                "2",
                r"\tfrac{7}{20}\text{ to }\tfrac{11}{20}",
                r"\tfrac{3}{10}\text{ to }\tfrac{1}{2}",
                r"\tfrac{1}{5}",
            ],
            [
                "8",
                r"\tfrac{273}{625}\text{ to }\tfrac{274}{625}",
                r"\tfrac{234}{625}\text{ to }\tfrac{47}{125}",
                r"\tfrac{1}{625}",
            ],
            [
                "9",
                r"\tfrac{1093}{2500}\text{ to }\tfrac{219}{500}",
                r"\tfrac{1171}{3125}\text{ to }\tfrac{1173}{3125}",
                r"\tfrac{1}{1250}",
            ],
        ];
        assert_eq!(rows, want);
        let f = facts::facts().unwrap();
        assert_eq!(f.get("R-F16.first_within"), Some("9"));
        assert_eq!(facts::runs_within(&low, &high, &unequal.middle()), Some(9));
        assert_eq!(RUN_ROWS.last(), Some(&9));
        assert!(CODE[0].contains("lo[top] = hi[top] = 1") && CODE[0].contains("lo[bottom] = hi[bottom] = 0"));
        assert!(CODE.iter().any(|l| l.contains("new_lo[j]")) && CODE.iter().any(|l| l.contains("new_hi[j]")));
        assert!(CODE.iter().all(|l| l.is_ascii()), "the typewriter face holds ASCII");
        assert!(CODE.last().unwrap().contains("exact fractions; floats: round lo down, hi up"));
        assert_eq!(f.get("R-X9.island.moves"), Some("false"));
        assert_eq!(
            recap_rows().unwrap(),
            [
                "a line of 5 wires: 1/5, 2/5, 3/5, 4/5",
                "the bridge: 3/5 and 2/5",
                "these wires: 7/16 and 3/8",
            ]
        );
    }

    /// Cards I and J fit a lower-third of three lines; I can be read from
    /// "never swaps the order" to the end of the 11_4 question, by the
    /// scratch voice with this video's gaps.
    #[test]
    fn the_cards_fit_and_can_be_read() {
        for text in [CARD_I, CARD_J] {
            let lines = explainer_kit::animatic::wrap(text, 84, 10);
            assert!(lines.len() <= 3, "{} lines: {text}", lines.len());
        }
        assert!(CARD_J.starts_with("finite network; conductances positive and finite"));
        assert!(CARD_I.contains("in exact arithmetic"));
        let script = explainer_kit::Script::parse(crate::SCRIPT)
            .unwrap()
            .with_line_gaps(crate::EXPLAINER.line_gaps);
        let voice = explainer_kit::Narrator::estimate(&script);
        let len = |id: &str| voice.length(id).unwrap_or_else(|| panic!("no line {id}"));
        let text = script.text("11_2").unwrap();
        let at = text[..text.find("never swaps the order").unwrap()].chars().count() as f64 / text.chars().count() as f64;
        let up = len("11_2") * (1.0 - at) - CARD_IN
            + voice.line_gap("11_2", "11_3")
            + len("11_3")
            + voice.line_gap("11_3", "11_4")
            + len("11_4")
            - CARD_OUT;
        assert!(read_s(CARD_I) <= up, "{:.2} s to read, {up:.2} s up", read_s(CARD_I));
    }

    /// The table, the island and the closing rows stay right of the picture
    /// and inside the title-safe area; the countdown sits under the island
    /// and above the frame's safe bottom.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        use crate::chapters::draw::THINK_R;
        const { assert!(COLUMN_X[0] - 0.3 > PICTURE_AT[0] + 1.7 + END_LABEL_GAP + 0.3) };
        const { assert!(COLUMN_X[3] + 0.5 < FRAME_X_RADIUS - 0.25) };
        const { assert!(COLUMN_X[2] - COLUMN_X[1] > 2.3) };
        const { assert!(HEADER_Y + 0.2 < FRAME_Y_RADIUS - 0.15) };
        const { assert!(ROW0_Y - ROW_STEP * 5.0 - 0.3 > UNDER_Y + 0.25) };
        const { assert!(UNDER_Y - 0.3 > -FRAME_Y_RADIUS + 1.58) };
        const { assert!(ISLAND_AT[1][0] + JUNCTION_R + 0.6 < FRAME_X_RADIUS - 0.25) };
        // the countdown under the island's dots (its notes come after it)
        const { assert!(THINK_AT[1] + THINK_R < ISLAND_AT[0][1] - JUNCTION_R - 0.3) };
        const { assert!(UNDER_X + 0.7 < FRAME_X_RADIUS - 0.25) };
        const { assert!(THINK_AT[1] - THINK_R - 0.45 > -FRAME_Y_RADIUS + 0.15) };
        const { assert!(RECAP_Y[3] - 0.3 > -FRAME_Y_RADIUS + 1.58) };
        assert!((ISLAND_AT[0][1] - ISLAND_AT[1][1]).abs() < 1e-12);
    }
}
