//! 14 No rule keeps both: the five-town, ten-seat proof of Gölz, Peters and
//! Procaccia (arXiv 2202.11061v2, Theorem 1 and Table 1). Profile A (a city
//! of 824 and four villages of 44): the city may have 8 or 9 seats. If 9,
//! growing the fifth village (profile B) forces the city to at most 7 and
//! hands a seat to an unchanged village. If 8, shrinking villages 2 and 3
//! (profile C) forces the city to at least 9 and takes a seat from village 4
//! or 5. Shares and bounds from `model`, pinned to Table 1 in `facts`.

use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

use super::draw::{
    NAME_X, READ_SIZE, READ_X, TOKEN, band, check, cross, fade_in_all, fade_in_lagged, fade_out_all, fail, flash_all,
    grid, line_style, morph_all, pause, quota_box, refill, say, stroke, think, token, token_spot, wipe,
};
use crate::model::{GPP_A, GPP_B, GPP_C, GPP_HOUSE, Ratio, decimal, share};

pub struct NoRule {
    pub kit: Kit,
}

/// The extra assumption of Balinski and Young's 1982 proof, verbatim as
/// Gölz, Peters and Procaccia state it (arXiv 2202.11061v2, §3.1: "...all
/// assume what Robinson and Ullman (2010) call the order-preserving
/// property, i.e., if state i has strictly larger population than state j,
/// then i must receive at least as many seats as j."), in two screen lines.
const ORDER_PRESERVING: [&str; 2] = [
    "\"if state i has strictly larger population than state j,",
    "then i must receive at least as many seats as j\"",
];
/// A short source line; the description cites it in full.
const GPP_SECTION: &str = "Gölz, Peters & Procaccia 2024, §3.1";
/// Our example's number of towns, for 14_2's reminder.
const POP3_TOWNS: usize = crate::model::POP.len();

/// The five rows: the city on top, villages 2 to 5 below.
const ROW5_Y: [f64; 5] = [2.3, 1.15, 0.0, -1.15, -2.3];
const CITY: usize = 0;
/// Where the brackets and their notes sit, right of the readouts.
const BRACKET_X: f64 = 2.3;

fn color5(i: usize) -> Srgb {
    if i == CITY { Palette::blue() } else { Palette::teal() }
}

fn label5(i: usize) -> String {
    if i == CITY {
        "city".into()
    } else {
        format!("village {}", i + 1)
    }
}

fn people5(p: u64) -> String {
    if p == 1 {
        "1 person".into()
    } else {
        format!("{p} people")
    }
}

/// One row's changing parts: its population line, share band and readout.
struct RowParts {
    people: Mob,
    band: [Mob; 2],
    read: Mob,
}

fn row_parts(kit: &Kit, stage: &mut Stage<'_>, pops: &[u64; 5], i: usize, color: Srgb) -> fmn::Result<RowParts> {
    let y = ROW5_Y[i];
    let q = share(pops, i, GPP_HOUSE);
    Ok(RowParts {
        people: kit.text_left(stage, &people5(pops[i]), 18.0, Palette::dim(), v(NAME_X, y - 0.24))?,
        band: band(stage, y, q.to_f64(), color)?,
        read: kit.text_left(stage, &decimal(q, 2), READ_SIZE, color, v(READ_X, y))?,
    })
}

/// Morph rows `which` from their current parts to profile `pops`.
fn to_profile(
    kit: &Kit,
    stage: &mut Stage<'_>,
    parts: &[RowParts],
    pops: &[u64; 5],
    which: &[usize],
    hot: Option<usize>,
    rt: f64,
) -> fmn::Result<()> {
    let mut pairs = Vec::new();
    for &i in which {
        let color = if hot == Some(i) { Palette::hero() } else { color5(i) };
        let new = row_parts(kit, stage, pops, i, color)?;
        pairs.push((parts[i].people, new.people));
        pairs.push((parts[i].band[0], new.band[0]));
        pairs.push((parts[i].band[1], new.band[1]));
        pairs.push((parts[i].read, new.read));
    }
    morph_all(stage, &pairs, rt)
}

/// The five rows of a profile, faded in; returns (fixed parts, row parts).
fn five_rows(kit: &Kit, stage: &mut Stage<'_>, pops: &[u64; 5]) -> fmn::Result<(Vec<Mob>, Vec<RowParts>)> {
    let mut fixed = Vec::new();
    let mut parts = Vec::new();
    for (i, &y) in ROW5_Y.iter().enumerate() {
        fixed.push(kit.text_left(stage, &label5(i), 24.0, color5(i), v(NAME_X, y + 0.14))?);
        fixed.extend(grid(stage, y, 10)?);
        parts.push(row_parts(kit, stage, pops, i, color5(i))?);
    }
    Ok((fixed, parts))
}

fn quota_of(pops: &[u64; 5], i: usize) -> Ratio {
    share(pops, i, GPP_HOUSE)
}

/// A bracket right of rows `top..=bottom` with a note, detached.
fn bracket(
    kit: &Kit,
    stage: &mut Stage<'_>,
    top: usize,
    bottom: usize,
    note: &str,
    color: Srgb,
) -> fmn::Result<[Mob; 2]> {
    let (y0, y1) = (ROW5_Y[top] + 0.4, ROW5_Y[bottom] - 0.4);
    let b = stroke(
        stage,
        vec![
            v(BRACKET_X - 0.12, y0),
            v(BRACKET_X, y0),
            v(BRACKET_X, y1),
            v(BRACKET_X - 0.12, y1),
        ],
        line_style(color, 3.0, 1.0),
    );
    let t = kit.text_left(stage, note, 22.0, color, v(BRACKET_X + 0.25, (y0 + y1) / 2.0))?;
    Ok([b, t])
}

impl SceneConstruct for NoRule {
    fn name(&self) -> &str {
        "14_no_rule"
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
        let h = GPP_HOUSE as u128;
        // the proof's arithmetic, checked before anything is drawn
        let (a_city, b_city, b_fifth, c_city) = (
            quota_of(&GPP_A, CITY),
            quota_of(&GPP_B, CITY),
            quota_of(&GPP_B, 4),
            quota_of(&GPP_C, CITY),
        );
        let villages_b = h - b_city.ceil() - b_fifth.ceil();
        let left_c = h - c_city.floor();
        if (a_city.floor(), a_city.ceil()) != (8, 9)
            || (1..5).any(|i| quota_of(&GPP_A, i).floor() != 0)
            || villages_b < 1
            || left_c > 1
            || GPP_B[CITY] != GPP_A[CITY]
            || GPP_C[CITY] != GPP_A[CITY]
            || GPP_B[1..4] != GPP_A[1..4]
            || GPP_C[3..] != GPP_A[3..]
        {
            return Err(fail("14: the profiles do not support the proof as narrated".into()));
        }

        // 14_1 who proved what, and the extra assumption in the words of the
        // paper that names it
        let s = say(&kit, stage, "14_1")?;
        let by = kit.text(
            stage,
            &format!(
                "Balinski and Young, {}: no rule keeps both on every map",
                fact("history.by_impossibility_year")?
            ),
            26.0,
            Palette::ink(),
            v(0.0, 3.25),
        )?;
        fade_in_all(stage, &[by], 0.6)?;
        s.wait_for(stage, "with one extra assumption")?;
        let mut assume = vec![kit.text(
            stage,
            "assuming what is called the order-preserving property:",
            22.0,
            Palette::dim(),
            v(0.0, 2.7),
        )?];
        for (j, line) in ORDER_PRESERVING.iter().enumerate() {
            assume.push(kit.text_styled(
                stage,
                line,
                24.0,
                Palette::ink(),
                v(0.0, 2.15 - 0.48 * j as f64),
                false,
                true,
            )?);
        }
        assume.push(kit.text(stage, GPP_SECTION, 20.0, Palette::dim(), v(0.0, 1.2))?);
        fade_in_lagged(stage, &assume, 1.4, 0.25)?;
        s.wait_for(stage, "A recent paper")?;
        let gpp = [
            kit.text(
                stage,
                &format!(
                    "Gölz, Peters and Procaccia, {}: no extra assumption needed",
                    fact("history.gpp_year")?
                ),
                26.0,
                Palette::hero(),
                v(0.0, 0.1),
            )?,
            kit.text(
                stage,
                "for any rule that always gives the same answer to the same question",
                22.0,
                Palette::dim(),
                v(0.0, -0.4),
            )?,
            kit.text(
                stage,
                "arXiv 2202.11061, Theorem 1",
                20.0,
                Palette::dim(),
                v(0.0, -0.85),
            )?,
        ];
        fade_in_lagged(stage, &gpp, 1.2, 0.3)?;

        // 14_2 why the proof needs more than our three towns
        let s = say(&kit, stage, "14_2")?;
        s.wait_for(stage, "with three towns Webster's rule keeps both promises")?;
        let three = kit.text(
            stage,
            &format!("{} towns: Webster keeps both promises", POP3_TOWNS),
            24.0,
            Palette::green(),
            v(0.0, -1.75),
        )?;
        fade_in_all(stage, &[three], 0.5)?;
        s.wait_for(stage, "It uses a city and four villages")?;
        let size = kit.text(
            stage,
            &format!("{} towns, {GPP_HOUSE} seats", GPP_A.len()),
            34.0,
            Palette::ink(),
            v(0.0, -2.6),
        )?;
        fade_in_all(stage, &[size], 0.5)?;

        // 14_3 profile A: a city of 824 and four villages of 44
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "14_3")?;
        let head = kit.text(stage, "10 seats", 30.0, Palette::ink(), v(-1.4, 3.35))?;
        let (fixed, parts) = five_rows(&kit, stage, &GPP_A)?;
        let city_mobs: Vec<Mob> = fixed[..11].to_vec();
        let village_mobs: Vec<Mob> = fixed[11..].to_vec();
        let people_of = |which: &[usize]| -> Vec<Mob> { which.iter().map(|&i| parts[i].people).collect() };
        s.wait_for(stage, "One city")?;
        fade_in_all(stage, &[vec![head], city_mobs.clone(), people_of(&[0])].concat(), 0.6)?;
        s.wait_for(stage, "four villages")?;
        fade_in_lagged(stage, &[village_mobs, people_of(&[1, 2, 3, 4])].concat(), 1.2, 0.05)?;
        s.wait_for(stage, "The city's share is")?;
        fade_in_all(
            stage,
            &[parts[CITY].band.to_vec(), vec![parts[CITY].read]].concat(),
            0.5,
        )?;
        s.wait_for(stage, "and each village's is")?;
        let vill: Vec<Mob> = (1..5)
            .flat_map(|i| [parts[i].band[0], parts[i].band[1], parts[i].read])
            .collect();
        fade_in_lagged(stage, &vill, 1.0, 0.15)?;
        s.wait_for(stage, "eight seats or nine")?;
        let city_box = quota_box(stage, ROW5_Y[CITY], a_city.floor(), a_city.ceil(), Palette::hero())?;
        fade_in_all(stage, &[city_box], 0.4)?;
        flash_all(stage, &[city_box], 0.9)?;
        let q = think(&kit, stage, v(5.6, -0.4))?;

        // 14_4 if the city gets 9: grow the fifth village (profile B)
        let s = say(&kit, stage, "14_4")?;
        fade_out_all(stage, &[q], 0.2)?;
        s.wait_for(stage, "Suppose the city gets nine")?;
        let city9: Vec<Mob> = (0..9)
            .map(|k| token(stage, ROW5_Y[CITY], k, color5(CITY)))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &city9, 1.0, 0.3)?;
        s.wait_for(stage, "Then the last seat goes to one village")?;
        let fifth = token(stage, ROW5_Y[4], 0, color5(4))?;
        fade_in_all(stage, &[fifth], 0.4)?;
        s.wait_for(stage, "Call it the fifth")?;
        // each row's fixed parts are its label and ten cells; the fifth village is row 4
        flash_all(stage, &[fixed[4 * 11]], 0.8)?;
        s.wait_for(stage, "Now let only the fifth village grow")?;
        to_profile(&kit, stage, &parts, &GPP_B, &[4], Some(4), 1.2)?;
        s.wait_for(stage, "The city's share falls to")?;
        to_profile(&kit, stage, &parts, &GPP_B, &[0, 1, 2, 3], None, 0.9)?;
        let box_b = quota_box(stage, ROW5_Y[CITY], b_city.floor(), b_city.ceil(), Palette::red())?;
        s.wait_for(stage, "so it can have at most seven seats")?;
        morph_all(stage, &[(city_box, box_b)], 0.6)?;
        let over: Vec<(Mob, Srgb, f64)> = city9[b_city.ceil() as usize..]
            .iter()
            .map(|&m| (m, Palette::red(), 1.0))
            .collect();
        refill(stage, &over, 0.4)?;
        s.wait_for(stage, "The fifth village's share is")?;
        let fifth_box = quota_box(stage, ROW5_Y[4], b_fifth.floor(), b_fifth.ceil(), Palette::hero())?;
        fade_in_all(stage, &[fifth_box], 0.4)?;
        s.wait_for(stage, "That leaves at least one seat")?;
        let note_b = bracket(
            &kit,
            stage,
            1,
            3,
            &format!("at least {villages_b} seat"),
            Palette::hero(),
        )?;
        fade_in_all(stage, &note_b, 0.5)?;

        // 14_5 an unchanged village gains while the unchanged city loses
        let s = say(&kit, stage, "14_5")?;
        s.wait_for(stage, "goes from no seats to one")?;
        let gainer = kit.text(
            stage,
            "0 → 1",
            26.0,
            Palette::hero(),
            v(BRACKET_X + 0.9, ROW5_Y[2] - 0.55),
        )?;
        fade_in_all(stage, &[gainer], 0.4)?;
        s.wait_for(stage, "drops from nine seats to seven or fewer")?;
        fade_out_all(stage, &city9[b_city.ceil() as usize..], 0.6)?;
        let city_note = kit.text(
            stage,
            "9 → 7 or fewer",
            26.0,
            Palette::red(),
            v(BRACKET_X + 1.4, ROW5_Y[CITY]),
        )?;
        fade_in_all(stage, &[city_note], 0.4)?;
        s.wait_for(stage, "The city did not shrink")?;
        let arrow = stroke(
            stage,
            vec![
                v(BRACKET_X + 1.4, ROW5_Y[CITY] - 0.3),
                v(BRACKET_X + 1.4, ROW5_Y[1] + 0.1),
            ],
            line_style(Palette::red(), 4.0, 1.0),
        );
        fade_in_all(stage, &[arrow], 0.5)?;
        s.wait_for(stage, "The population promise is broken")?;
        let x = cross(stage, v(6.3, 0.3), 0.8, Palette::red());
        fade_in_all(stage, &x, 0.4)?;

        // 14_6 if the city gets 8: shrink villages 2 and 3 (profile C)
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "14_6")?;
        let head = kit.text(stage, "10 seats", 30.0, Palette::ink(), v(-1.4, 3.35))?;
        let (fixed, parts) = five_rows(&kit, stage, &GPP_A)?;
        let mut all: Vec<Mob> = fixed.clone();
        all.push(head);
        for p in &parts {
            all.extend([p.people, p.band[0], p.band[1], p.read]);
        }
        let city_box = quota_box(stage, ROW5_Y[CITY], a_city.floor(), a_city.ceil(), Palette::hero())?;
        all.push(city_box);
        fade_in_all(stage, &all, 0.6)?;
        s.wait_for(stage, "Suppose instead the city gets eight")?;
        let city8: Vec<Mob> = (0..8)
            .map(|k| token(stage, ROW5_Y[CITY], k, color5(CITY)))
            .collect::<fmn::Result<_>>()?;
        fade_in_lagged(stage, &city8, 1.0, 0.3)?;
        s.wait_for(stage, "Then two villages get one seat each")?;
        let pair = [
            token(stage, ROW5_Y[3], 0, color5(3))?,
            token(stage, ROW5_Y[4], 0, color5(4))?,
        ];
        fade_in_all(stage, &pair, 0.4)?;
        s.wait_for(stage, "Call them the fourth and fifth")?;
        flash_all(stage, &pair, 0.8)?;
        s.wait_for(stage, "shrink the other two villages")?;
        to_profile(&kit, stage, &parts, &GPP_C, &[1, 2], None, 1.2)?;
        s.wait_for(stage, "The city's share rises to")?;
        to_profile(&kit, stage, &parts, &GPP_C, &[0, 3, 4], None, 0.9)?;
        s.wait_for(stage, "so it needs at least nine seats")?;
        let box_c = quota_box(stage, ROW5_Y[CITY], c_city.floor(), c_city.ceil(), Palette::red())?;
        morph_all(stage, &[(city_box, box_c)], 0.6)?;
        let need = Rectangle::new()
            .width(TOKEN)
            .height(TOKEN)
            .corner_radius(0.06)
            .style(line_style(Palette::red(), 3.0, 1.0))
            .build()?
            .moved_to(token_spot(ROW5_Y[CITY], 8));
        let need = stage.arena_mut().add(need);
        fade_in_all(stage, &[need], 0.4)?;
        s.wait_for(stage, "Only one seat is left")?;
        let note_c = bracket(
            &kit,
            stage,
            1,
            4,
            &format!("at most {left_c} seat for all four"),
            Palette::red(),
        )?;
        fade_in_all(stage, &note_c, 0.5)?;
        s.wait_for(stage, "so one of them drops to zero")?;
        refill(
            stage,
            &[(pair[0], Palette::red(), 1.0), (pair[1], Palette::red(), 1.0)],
            0.4,
        )?;
        let either = kit.text(
            stage,
            "village 4 or 5: 1 → 0",
            24.0,
            Palette::red(),
            v(BRACKET_X + 1.6, ROW5_Y[4] - 0.65),
        )?;
        fade_in_all(stage, &[either], 0.4)?;
        s.wait_for(stage, "That village did not shrink")?;
        let x = cross(stage, v(6.3, 0.3), 0.8, Palette::red());
        fade_in_all(stage, &x, 0.4)?;

        // 14_7 both choices fail: what is known
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "14_7")?;
        s.wait_for(stage, "So no rule that keeps the quota promise")?;
        let five = kit.text_left(
            stage,
            "5 or more towns: no rule keeps both on every map",
            28.0,
            Palette::red(),
            v(-5.0, 1.6),
        )?;
        let five_x = cross(stage, v(-5.6, 1.6), 0.5, Palette::red());
        fade_in_all(stage, &[vec![five], five_x.to_vec()].concat(), 0.6)?;
        s.wait_for(stage, "With three towns, Webster's rule keeps both")?;
        let three = kit.text_left(
            stage,
            "3 towns: Webster's rule keeps both",
            28.0,
            Palette::green(),
            v(-5.0, 0.4),
        )?;
        let three_c = check(stage, v(-5.6, 0.4), 0.55, Palette::green());
        fade_in_all(stage, &[three, three_c], 0.6)?;
        s.wait_for(stage, "With four")?;
        let four = kit.text_left(
            stage,
            "4 towns: open, as of that paper",
            28.0,
            Palette::hero(),
            v(-5.0, -0.8),
        )?;
        let four_q = kit.text(stage, "?", 36.0, Palette::hero(), v(-5.6, -0.8))?;
        fade_in_all(stage, &[four, four_q], 0.6)?;

        // 14_8 the way out: draw lots
        wipe(&kit, stage)?;
        let s = say(&kit, stage, "14_8")?;
        let title = kit.text(stage, "a lottery rule (same paper)", 30.0, Palette::hero(), v(0.0, 2.9))?;
        fade_in_all(stage, &[title], 0.5)?;
        s.wait_for(stage, "it can keep the quota promise")?;
        let mut promises = Vec::new();
        for (j, text) in [
            "quota promise, in every draw",
            "never takes a seat back",
            "every town its exact share, on average",
        ]
        .iter()
        .enumerate()
        {
            let y = 1.8 - 0.75 * j as f64;
            promises.push(check(stage, v(-4.2, y), 0.42, Palette::green()));
            promises.push(kit.text_left(stage, text, 26.0, Palette::ink(), v(-3.6, y))?);
        }
        fade_in_lagged(stage, &promises[..4], 1.2, 0.3)?;
        s.wait_for(stage, "exactly its share on average")?;
        fade_in_all(stage, &promises[4..], 0.5)?;
        let ash = kit.text(
            stage,
            &format!(
                "Ash at 15 seats: 2 seats in {}% of draws, 1 seat in {}%, on average {}",
                fact("lottery.15.ash.two_pct")?,
                fact("lottery.15.ash.one_pct")?,
                fact("share.15.ash")?
            ),
            22.0,
            Palette::blue(),
            v(0.0, -1.2),
        )?;
        fade_in_all(stage, &[ash], 0.6)?;
        pause(stage, 0.6)?;
        kit.finish(stage)
    }
}
