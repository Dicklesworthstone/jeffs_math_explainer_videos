//! 10 The rest of the family: where the rounding happens. A ruler from 1 to
//! 2 shows where a quotient starts to count as 2 (Jefferson at 2, Adams just
//! above 1, Webster at 1.5, Hill at √2); the three rows follow each rule at
//! fifteen seats; a table compares Hamilton, Webster and Hill at 15, 16 and
//! 17 seats; and Hill at three seats breaks the quota promise. Every seat
//! count comes from `model`.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    ROW_Y, Rows, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all, line_style, morph_all, pause, quota_box,
    readout, refill, say, share_readouts, stroke, think_here, token, town_color, wipe,
};
use crate::model::{self, ASH, CEDAR, POP, Rule, decimal, share};

pub struct RoundingRules {
    pub kit: Kit,
}

/// The ruler: quotients from 1 to 2 along the top of the frame.
const RY: f64 = 2.9;
const RX: (f64, f64) = (-1.6, 3.6);
/// The rule's name, left of the ruler.
const RULE_X: f64 = -6.5;

fn ruler_x(q: f64) -> f64 {
    RX.0 + (q - 1.0) * (RX.1 - RX.0)
}

/// The moving parts of the ruler: the part that stays 1 seat, the part that
/// becomes 2, the tick between them, the tick's value and the rule's name.
struct Ruler {
    parts: [Mob; 5],
}

/// Detached ruler parts with the threshold at quotient `q`.
fn ruler_parts(kit: &Kit, stage: &mut Stage<'_>, q: f64, value: &str, rule: &str) -> fmn::Result<Ruler> {
    // a threshold exactly at an end still leaves a sliver of each part
    let x = ruler_x(q.clamp(1.006, 1.994));
    let low = stroke(
        stage,
        vec![v(RX.0, RY), v(x, RY)],
        line_style(Palette::faint(), 10.0, 0.9),
    );
    let high = stroke(
        stage,
        vec![v(x, RY), v(RX.1, RY)],
        line_style(Palette::hero(), 10.0, 0.9),
    );
    let tick = stroke(
        stage,
        vec![v(x, RY - 0.24), v(x, RY + 0.24)],
        line_style(Palette::ink(), 4.0, 1.0),
    );
    let value = kit.text(stage, value, 20.0, Palette::ink(), v(x, RY + 0.45))?;
    let rule = kit.text_left(stage, rule, 24.0, Palette::ink(), v(RULE_X, RY))?;
    Ok(Ruler {
        parts: [low, high, tick, value, rule],
    })
}

/// Move the threshold: morph every moving part into a new one.
fn reruler(kit: &Kit, stage: &mut Stage<'_>, r: &Ruler, q: f64, value: &str, rule: &str, rt: f64) -> fmn::Result<()> {
    let new = ruler_parts(kit, stage, q, value, rule)?;
    let pairs: Vec<(Mob, Mob)> = r.parts.iter().copied().zip(new.parts.iter().copied()).collect();
    morph_all(stage, &pairs, rt)
}

/// The ruler's fixed labels: its ends and what each side means.
fn ruler_frame(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Vec<Mob>> {
    Ok(vec![
        kit.text(stage, "1", 18.0, Palette::dim(), v(RX.0, RY - 0.4))?,
        kit.text(stage, "2", 18.0, Palette::dim(), v(RX.1, RY - 0.4))?,
        kit.text(
            stage,
            "quotient",
            16.0,
            Palette::dim(),
            v((RX.0 + RX.1) / 2.0, RY - 0.4),
        )?,
        kit.text(stage, "1 seat", 20.0, Palette::dim(), v(RX.0 - 0.7, RY))?,
        kit.text(stage, "2 seats", 20.0, Palette::hero(), v(RX.1 + 0.8, RY))?,
    ])
}

/// Change the tokens on the rows to `seats`: new ones fade in, extra ones
/// fade out from the right.
fn retoken(stage: &mut Stage<'_>, toks: &mut [Vec<Mob>], seats: &[u64], rt: f64) -> fmn::Result<()> {
    let (mut add, mut drop) = (Vec::new(), Vec::new());
    for (i, row) in toks.iter_mut().enumerate() {
        let want = seats[i] as usize;
        while row.len() < want {
            let t = token(stage, ROW_Y[i], row.len(), town_color(i))?;
            add.push(t);
            row.push(t);
        }
        while row.len() > want {
            drop.extend(row.pop());
        }
    }
    fade_out_all(stage, &drop, rt / 2.0)?;
    fade_in_all(stage, &add, rt / 2.0)
}

/// "2, 3, 10".
fn triple(s: &[u64]) -> String {
    s.iter().map(u64::to_string).collect::<Vec<_>>().join(", ")
}

/// The comparison table: columns and rows.
const COL_X: [f64; 4] = [-4.2, -1.6, 1.0, 3.6];
const TABLE_Y: f64 = 1.3;
const TABLE_STEP: f64 = 0.8;

impl SceneConstruct for RoundingRules {
    fn name(&self) -> &str {
        "10_rounding_rules"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let facts = crate::facts::facts().map_err(fail)?;
        let fact = |key: &str| {
            facts
                .get(key)
                .map(str::to_owned)
                .ok_or_else(|| fail(format!("no fact {key}")))
        };
        let jefferson = model::divisor(Rule::Jefferson, &POP, 15).map_err(fail)?;
        let adams = model::divisor(Rule::Adams, &POP, 15).map_err(fail)?;
        let webster = model::divisor(Rule::Webster, &POP, 15).map_err(fail)?;
        let cedar_q = share(&POP, CEDAR, 15);
        if u128::from(adams[CEDAR]) >= cedar_q.floor() {
            return Err(fail("10_2 says Adams puts Cedar below its share".into()));
        }
        if webster != model::hamilton(&POP, 15).map_err(fail)? {
            return Err(fail("10_4 says Webster equals Hamilton at fifteen seats".into()));
        }

        // 10_1 Jefferson rounds down: a quotient becomes 2 only at 2
        let s = say(&kit, stage, "10_1")?;
        let frame = ruler_frame(&kit, stage)?;
        let ruler = ruler_parts(&kit, stage, 2.0, "2", "Jefferson: round down")?;
        let rows = Rows::build(&kit, stage, &POP, 15, 12)?;
        let reads = share_readouts(&kit, stage, &POP, 15)?;
        let mut toks: Vec<Vec<Mob>> = vec![Vec::new(), Vec::new(), Vec::new()];
        let mut first = rows.all();
        first.extend(&reads);
        fade_in_all(stage, &first, 0.6)?;
        let mut top = frame.clone();
        top.extend(ruler.parts);
        fade_in_lagged(stage, &top, 1.2, 0.15)?;
        retoken(stage, &mut toks, &jefferson, 0.8)?;
        s.wait_for(stage, "Change where the rounding happens")?;
        flash_all(stage, &ruler.parts[2..4], 0.9)?;

        // 10_2 Adams rounds up: any piece of a share earns a seat
        let s = say(&kit, stage, "10_2")?;
        s.wait_for(stage, "proposed rounding up")?;
        reruler(&kit, stage, &ruler, 1.0, "just above 1", "Adams: round up", 1.2)?;
        s.wait_for(stage, "lean flips toward small towns")?;
        flash_all(stage, &[ruler.parts[1]], 0.8)?;
        s.wait_for(stage, "At fifteen seats Adams gives")?;
        retoken(stage, &mut toks, &adams, 0.8)?;
        s.wait_for(stage, "Cedar falls below")?;
        let cbox = quota_box(stage, ROW_Y[CEDAR], cedar_q.floor(), cedar_q.ceil(), Palette::red())?;
        fade_in_all(stage, &[cbox], 0.4)?;
        flash_all(stage, &[cbox, reads[CEDAR]], 1.0)?;

        // 10_3 the compromise: round to the nearest
        say(&kit, stage, "10_3")?;
        fade_out_all(stage, &[cbox], 0.3)?;
        reruler(&kit, stage, &ruler, 1.5, "1.5", "round to the nearest", 1.2)?;
        let q = think_here(&kit, stage)?;

        // 10_4 Webster: the same as Hamilton, and it never takes a seat back
        let s = say(&kit, stage, "10_4")?;
        fade_out_all(stage, &[q], 0.2)?;
        retoken(stage, &mut toks, &webster, 0.8)?;
        s.wait_for(stage, "the same as Hamilton")?;
        let ham15 = kit.text(
            stage,
            &format!("Hamilton: {}", triple(&webster)),
            24.0,
            Palette::dim(),
            v(3.8, ROW_Y[1]),
        )?;
        fade_in_all(stage, &[ham15], 0.4)?;
        s.wait_for(stage, "This is Daniel Webster's rule")?;
        reruler(&kit, stage, &ruler, 1.5, "1.5", "Webster: round to the nearest", 0.6)?;
        s.wait_for(stage, "At sixteen seats it gives")?;
        let mut on_rows: Vec<Mob> = rows.all();
        on_rows.extend(&reads);
        on_rows.extend(toks.iter().flatten());
        on_rows.push(ham15);
        fade_out_all(stage, &on_rows, 0.5)?;
        let mut head = Vec::new();
        for (c, label) in ["seats", "Hamilton", "Webster", "Hill"].iter().enumerate().take(3) {
            head.push(kit.text(stage, label, 24.0, Palette::dim(), v(COL_X[c], TABLE_Y))?);
        }
        let hill_head = kit.text(stage, "Hill", 24.0, Palette::dim(), v(COL_X[3], TABLE_Y))?;
        let rule_line = stroke(
            stage,
            vec![v(COL_X[0] - 1.0, TABLE_Y - 0.35), v(COL_X[3] + 1.2, TABLE_Y - 0.35)],
            line_style(Palette::faint(), 2.0, 0.8),
        );
        let mut table_rows: Vec<Vec<Mob>> = Vec::new();
        let mut hill_cells = Vec::new();
        for (r, h) in [15_u64, 16, 17].into_iter().enumerate() {
            let y = TABLE_Y - TABLE_STEP * (r as f64 + 1.0);
            let ham = model::hamilton(&POP, h).map_err(fail)?;
            let web = model::divisor(Rule::Webster, &POP, h).map_err(fail)?;
            let hill = model::divisor(Rule::Hill, &POP, h).map_err(fail)?;
            if hill != web {
                return Err(fail(format!("10_5 says Hill agrees with Webster at {h} seats")));
            }
            let lost = r > 0 && ham[ASH] < model::hamilton(&POP, h - 1).map_err(fail)?[ASH];
            let row = vec![
                kit.text(stage, &h.to_string(), 26.0, Palette::ink(), v(COL_X[0], y))?,
                kit.text(
                    stage,
                    &triple(&ham),
                    26.0,
                    if lost { Palette::red() } else { Palette::ink() },
                    v(COL_X[1], y),
                )?,
                kit.text(stage, &triple(&web), 26.0, Palette::green(), v(COL_X[2], y))?,
            ];
            hill_cells.push(kit.text(stage, &triple(&hill), 26.0, Palette::teal(), v(COL_X[3], y))?);
            table_rows.push(row);
        }
        let mut t15 = head.clone();
        t15.push(rule_line);
        t15.extend(&table_rows[0]);
        fade_in_all(stage, &t15, 0.5)?;
        fade_in_lagged(stage, &table_rows[1], 0.8, 0.3)?;
        s.wait_for(stage, "and at seventeen")?;
        fade_in_lagged(stage, &table_rows[2], 0.8, 0.3)?;
        s.wait_for(stage, "Nobody ever loses a seat")?;
        let web_col: Vec<Mob> = table_rows.iter().map(|r| r[2]).collect();
        flash_all(stage, &web_col, 1.0)?;
        s.wait_for(stage, "what we hoped Hamilton would do")?;
        flash_all(stage, &[table_rows[1][1]], 0.9)?;

        // 10_5 Hill: round up past the square root of n times n + 1
        let s = say(&kit, stage, "10_5")?;
        s.wait_for(stage, "became the law")?;
        let law = kit.text(
            stage,
            &format!("United States House, since {}", fact("history.hill_law_year")?),
            20.0,
            Palette::dim(),
            v(COL_X[3], TABLE_Y + 0.55),
        )?;
        fade_in_all(stage, &[law], 0.4)?;
        s.wait_for(stage, "It is called the Huntington Hill method")?;
        let ghost = stroke(
            stage,
            vec![v(ruler_x(1.5), RY - 0.2), v(ruler_x(1.5), RY + 0.2)],
            line_style(Palette::dim(), 3.0, 0.8),
        );
        fade_in_all(stage, &[ghost], 0.3)?;
        let root = 2.0_f64.sqrt();
        s.wait_for(stage, "the square root of the two neighbouring")?;
        reruler(
            &kit,
            stage,
            &ruler,
            root,
            &format!("√(1 × 2) ≈ {root:.2}"),
            "Hill: past the square root",
            1.4,
        )?;
        s.wait_for(stage, "it agrees with Webster")?;
        fade_in_all(stage, &[hill_head], 0.3)?;
        fade_in_lagged(stage, &hill_cells, 0.9, 0.3)?;
        let mut pair = web_col.clone();
        pair.extend(&hill_cells);
        flash_all(stage, &pair, 1.0)?;

        // 10_6 Hill at three seats: Cedar's share is two, it gets one
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "10_6")?;
        let hill3 = model::divisor(Rule::Hill, &POP, 3).map_err(fail)?;
        let cedar3 = share(&POP, CEDAR, 3);
        if u128::from(hill3[CEDAR]) >= cedar3.floor() {
            return Err(fail(
                "10_6 says Hill gives Cedar less than its share at three seats".into(),
            ));
        }
        let rows3 = Rows::build(&kit, stage, &POP, 3, 4)?;
        let head3 = kit.text(stage, "Hill, 3 seats", 30.0, Palette::ink(), v(-1.4, 3.1))?;
        let mut first = rows3.all();
        first.push(head3);
        fade_in_all(stage, &first, 0.6)?;
        s.wait_for(stage, "At three seats it gives")?;
        let mut toks3: Vec<Vec<Mob>> = vec![Vec::new(), Vec::new(), Vec::new()];
        retoken(stage, &mut toks3, &hill3, 0.8)?;
        s.wait_for(stage, "though Cedar's share is exactly two")?;
        let read3 = readout(
            &kit,
            stage,
            &format!("share {}", decimal(cedar3, 2)),
            ROW_Y[CEDAR],
            town_color(CEDAR),
        )?;
        let box3 = quota_box(stage, ROW_Y[CEDAR], cedar3.floor(), cedar3.ceil(), Palette::red())?;
        fade_in_all(stage, &[read3, box3], 0.5)?;
        refill(stage, &[(toks3[CEDAR][0], Palette::red(), 1.0)], 0.4)?;
        flash_all(stage, &[box3], 0.9)?;
        s.wait_for(stage, "Every rule in this family")?;
        let one = kit.text(
            stage,
            "one list of signposts: never takes a seat back",
            24.0,
            Palette::green(),
            v(0.0, -2.75),
        )?;
        fade_in_all(stage, &[one], 0.5)?;
        s.wait_for(stage, "every one of them can break")?;
        let two = kit.text(
            stage,
            "but each can break the quota promise",
            24.0,
            Palette::red(),
            v(0.0, -3.3),
        )?;
        fade_in_all(stage, &[two], 0.5)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
