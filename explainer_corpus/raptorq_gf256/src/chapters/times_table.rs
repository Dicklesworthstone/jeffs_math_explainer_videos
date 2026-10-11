//! Chapter 12, the times table: the standard's table lookup on the byte
//! clock (12_0); the whole 256 × 256 table in byte order, each square
//! coloured by its answer, looking like noise (12_1); sort by clicks? (12_2);
//! the rows, then the columns, slide into log order (12_3, spike S-3: one
//! stroke strip per row, then per column, moved by placement only) and the
//! table is diagonal stripes, one total of clicks each (12_4); AES's other
//! labels for the same field (12_5); and the two clocks that close act II
//! (12_6).

use explainer_kit::props::{RowStrips, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, stroke_style, tween, v};
use fmn::core::color::color_gradient;
use fmn::prelude::*;

use super::byte_clock::{ByteClock, click_point, lookup_panel, two_clocks};
use super::draw::{Role, flat, group_of};
use super::fold::{aes_contrast, fade_list, math_lower_third};
use crate::model::{FOLD, exps, logs, mul, order};

/// The table's cells: 3 px at 1080p, 6 px at 4K (cell edges on pixel edges
/// at both; 1080p's frame is 8 units of 135 px).
const UNIT_PX: f64 = 135.0;
const CELL: f64 = 3.0 / UNIT_PX;
/// The table's top-left corner (slot 0 of row and column), centred
/// across, high enough to leave the foot for a lower-third.
const LEFT: f64 = -128.0 * CELL;
const TOP: f64 = (540.0 - 60.0) / UNIT_PX;
// 256 cells of 3 px at 1080p, 60 px from the top: the table fits the frame
// and leaves its foot free for a lower-third (checked when compiling).
const _: () = assert!(TOP < 3.6 && TOP - 256.0 * CELL > -2.4 && LEFT > -7.0 && LEFT + 256.0 * CELL < 7.0);
/// The set-aside zero row and column, dimmed.
const DIM: f64 = 0.35;
/// The sort: rows, then columns, each this long; the listen window holds both.
const SORT_S: f64 = 2.4;
const LISTEN_S: f64 = 6.0;
/// 12_0's two click counts, from 11_6.
const ONE_FIFTY: u8 = 150;
const THREE: u8 = 3;

/// The colour of a product: a ramp over the 256 bytes, blue to red.
fn palette() -> Vec<Srgb> {
    color_gradient(
        &[
            "#2B3A8C", "#3D7FD9", "#58C4DD", "#83C167", "#FFD866", "#F0AC5F", "#FC6255",
        ]
        .map(hex),
        256,
    )
}

/// The centre of slot `k` (0 … 255) across (columns) or down (rows).
fn x_of(k: f64) -> f64 {
    LEFT + CELL * (k + 0.5)
}
fn y_of(k: f64) -> f64 {
    TOP - CELL * (k + 0.5)
}

/// Slide `mob` by `distance` along `axis` over `seconds` from the next frame:
/// a dt updater that shifts it (placement only), smoothly (spike S-3).
fn slide(stage: &mut Stage<'_>, mob: Mob, axis: usize, distance: f64, seconds: f64) -> fmn::Result<()> {
    let (mut elapsed, mut done) = (0.0, 0.0);
    stage.add_dt_updater(
        mob,
        move |s, me, dt| {
            elapsed += dt;
            let t = (elapsed / seconds).clamp(0.0, 1.0);
            let at = distance * t * t * t * (t * (6.0 * t - 15.0) + 10.0);
            if at != done {
                let mut d = [0.0; 3];
                d[axis] = at - done;
                s.shift(me, d);
                done = at;
            }
        },
        false,
    )?;
    Ok(())
}

/// Row and column 0 (every product 0) as two flat bars in the table.
fn zero_bars(stage: &mut Stage<'_>, color: Srgb) -> fmn::Result<[Mob; 2]> {
    let row = Rectangle::new()
        .width(CELL * 255.0)
        .height(CELL)
        .style(flat(color, 1.0))
        .build()?
        .moved_to(v(x_of(128.0), y_of(0.0)));
    let col = Rectangle::new()
        .width(CELL)
        .height(CELL * 256.0)
        .style(flat(color, 1.0))
        .build()?
        .moved_to(v(x_of(0.0), y_of(127.5)));
    Ok([stage.arena_mut().add(row), stage.arena_mut().add(col)])
}

/// Strips whose cells are crisp: the renderer's antialiasing band at half a
/// pixel (S-3: the default 1.5 px left a seam between strips).
fn crisp(stage: &mut Stage<'_>, strips: &RowStrips) {
    for &m in &strips.rows {
        stage.set_anti_alias_width(m, 0.5, false);
    }
}

pub struct TimesTable {
    pub kit: Kit,
}

impl SceneConstruct for TimesTable {
    fn name(&self) -> &str {
        "12_the_times_table"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (exp, log) = (exps(), logs());
        let pal = palette();

        // 12_0: the standard's lookup: two click counts added, the answer looked up
        let cc = v(-3.9, 0.0);
        let rc = 2.1;
        let clock = ByteClock::build(stage, cc, rc)?;
        let (code, caption) = lookup_panel(&kit, stage)?;
        let (k150, k3) = (usize::from(log[ONE_FIFTY as usize]), usize::from(log[THREE as usize]));
        let product = exp[(k150 + k3) % 255];
        let look = kit.tex(
            stage,
            &format!(
                r"\text{{LOG}}[{ONE_FIFTY}] + \text{{LOG}}[{THREE}] = {k150} + {k3} = {}",
                k150 + k3
            ),
            38.0,
            Palette::ink(),
            &[],
            v(3.2, -0.7),
        )?;
        let answer = kit.tex(
            stage,
            &format!(r"\text{{EXP}}[{}] = {product}", k150 + k3),
            44.0,
            Role::byte(),
            &[],
            v(3.2, -1.6),
        )?;
        let at = stage.add_value_tracker(0.0);
        let hand = stage.always_redraw(move |s| {
            let k = s.tracker_value(at).unwrap_or(0.0);
            let line = Line::new(cc, click_point(cc, rc * 0.88, k))
                .style(stroke_style(Palette::ink(), 3.0))
                .build()
                .expect("a hand has length");
            let dot = Dot::new()
                .point(click_point(cc, rc, k))
                .radius(0.07)
                .color(Role::byte())
                .build();
            s.add(v_group([line, dot]))
        });
        kit.say(stage, "12_0")?;
        play!(stage;
            show_creation(clock.ring).rt(0.7),
            fade_in(stage.arena_mut(), clock.dots, ORIGIN, 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), clock.zero, ORIGIN, 1.0)?.rt(0.7),
            fade_in(stage.arena_mut(), code, v(-1.0, 0.0), 1.0)?.rt(0.7),
        );
        stage.add_to_scene(hand)?;
        play!(stage; fade_in(stage.arena_mut(), caption, ORIGIN, 1.0)?.rt(0.4), tween(at, k150 as f64, 0.8)?);
        play!(stage; write(stage.arena(), look).rt(0.8), tween(at, (k150 + k3) as f64, 0.8)?);
        play!(stage; write(stage.arena(), answer).rt(0.6));
        kit.hold(stage)?;

        // 12_1: every pair at once, in byte order: it looks like noise
        stage.clear_updaters(hand, true);
        fade_list(stage, &[clock.group, hand, code, caption, look, answer], 0.5)?;
        let rows: Vec<Vec<Srgb>> = (1..=255u8)
            .map(|a| (1..=255u8).map(|b| pal[mul(a, b, FOLD) as usize]).collect())
            .collect();
        let table = RowStrips::build(
            stage,
            v(x_of(1.0) - CELL / 2.0, y_of(1.0) + CELL / 2.0),
            (CELL, CELL),
            &rows,
        )?;
        crisp(stage, &table);
        let [zero_row, zero_col] = zero_bars(stage, pal[0])?;
        let legend = palette_key(&kit, stage, &pal)?;
        let corner = kit.tex(
            stage,
            r"a \times b",
            34.0,
            Palette::dim(),
            &[],
            v(LEFT - 0.9, TOP - 0.1),
        )?;
        let down = kit.text(stage, "a = 0 … 255", 24.0, Palette::dim(), v(LEFT - 0.9, 0.7))?;
        let across = kit.text(stage, "b = 0 … 255", 24.0, Palette::dim(), v(0.0, TOP + 0.15))?;
        kit.say(stage, "12_1")?;
        play!(stage;
            fade_in(stage.arena_mut(), table.group, ORIGIN, 1.0)?.rt(1.4),
            fade_in(stage.arena_mut(), zero_row, ORIGIN, 1.0)?.rt(1.4),
            fade_in(stage.arena_mut(), zero_col, ORIGIN, 1.0)?.rt(1.4),
        );
        play!(stage;
            fade_in(stage.arena_mut(), legend, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), corner, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), down, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), across, ORIGIN, 1.0)?.rt(0.6),
        );
        kit.hold(stage)?;

        // 12_2: sort the rows and the columns by clicks? (zero, with no click
        // count, set aside first)
        kit.say(stage, "12_2")?;
        play!(stage, 0.6;
            zero_row.animate().shift(v(0.0, CELL))?.set_opacity(DIM)?,
            zero_col.animate().shift(v(-CELL, 0.0))?.set_opacity(DIM)?,
        );
        let q = predict_pause(&kit, stage, v(5.4, 1.2))?;

        // 12_3: rows slide to their click order, then columns; the chords settle
        let order_rows: Vec<u8> = (1..=255u8).collect();
        let wav = super::sound::wav(
            "12_3_settle",
            &super::sound::settle(&order_rows, 2.0 * SORT_S, LISTEN_S),
        )?;
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        kit.sonify(stage, LISTEN_S, "12_3", &wav)?;
        for (a, &row) in (1..=255u8).zip(&table.rows) {
            let to = 1.0 + f64::from(log[a as usize]);
            slide(stage, row, 1, -(to - f64::from(a)) * CELL, SORT_S)?;
        }
        stage.wait(SORT_S)?;
        for &m in &table.rows {
            stage.clear_updaters(m, true);
        }
        // the swap: column strips over the same pixels (rows now in click
        // order), rows gone; a column strip is a row strip turned a quarter
        let cols: Vec<Vec<Srgb>> = (1..=255u8)
            .map(|b| exp.iter().rev().map(|&a| pal[mul(a, b, FOLD) as usize]).collect())
            .collect();
        let columns = RowStrips::build(
            stage,
            v(x_of(1.0) - CELL / 2.0, y_of(1.0) + CELL / 2.0),
            (CELL, CELL),
            &cols,
        )?;
        crisp(stage, &columns);
        let centre = v(x_of(128.0), y_of(128.0));
        stage.rotate(columns.group, std::f64::consts::FRAC_PI_2, OUT, Some(centre), None);
        stage.add_to_scene(columns.group)?;
        stage.scene_mut().remove(&[table.group]);
        stage.scene_mut().remove(&table.rows);
        let new_down = kit.text(stage, "a: by clicks", 24.0, Palette::dim(), v(LEFT - 0.9, 0.7))?;
        let new_across = kit.text(stage, "b: by clicks", 24.0, Palette::dim(), v(0.0, TOP + 0.15))?;
        play!(stage, 0.3;
            fmn::animation::replacement_transform(down, new_down),
        );
        for (b, &col) in (1..=255u8).zip(&columns.rows) {
            let to = 1.0 + f64::from(log[b as usize]);
            slide(stage, col, 0, (to - f64::from(b)) * CELL, SORT_S)?;
        }
        play!(stage, 0.3;
            fmn::animation::replacement_transform(across, new_across),
        );
        stage.wait(SORT_S - 0.3)?;
        for &m in &columns.rows {
            stage.clear_updaters(m, true);
        }
        let rest = LISTEN_S - 2.0 * SORT_S - 0.3;
        if rest > 0.0 {
            stage.wait(rest)?;
        }

        // 12_4: perfect diagonal stripes, one total of clicks each
        let stripe = Line::new(v(x_of(255.0), y_of(1.0)), v(x_of(1.0), y_of(255.0)))
            .style(
                Style::default()
                    .fill(Palette::ink(), 0.0)
                    .stroke(Palette::ink(), 2.0, 0.8),
            )
            .build()?;
        let stripe = stage.arena_mut().add(stripe);
        let total = kit.tex(
            stage,
            r"\text{clicks of } a + \text{clicks of } b = \text{one total}",
            30.0,
            Palette::ink(),
            &[],
            v(5.0, -1.75),
        )?;
        stage.set_width(total, 3.4, false);
        let counts = element_orders();
        let orders_tex = format!(
            r"\text{{orders }} {} \text{{ occur }} {} \text{{ times;}}\; {} = (2+1)(2^{{2}}+1)(2^{{4}}+1)",
            counts.iter().map(|(o, _)| o.to_string()).collect::<Vec<_>>().join(", "),
            counts.iter().map(|(_, c)| c.to_string()).collect::<Vec<_>>().join(", "),
            counts.last().map_or(0, |&(o, _)| o)
        );
        let orders = math_lower_third(&kit, stage, "precisely:", &orders_tex)?;
        kit.say(stage, "12_4")?;
        play!(stage; show_creation(stripe).rt(0.9), fade_in(stage.arena_mut(), total, ORIGIN, 1.0)?.rt(0.9));
        play!(stage; fade_in(stage.arena_mut(), orders, v(0.0, 0.2), 1.0)?.rt(0.5));
        kit.hold(stage)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;

        // 12_5: AES's prime polynomial: the same field, other labels
        let table_parts = [
            columns.group,
            zero_row,
            zero_col,
            legend,
            corner,
            new_down,
            new_across,
            stripe,
            total,
            orders,
        ];
        kit.say(stage, "12_5")?;
        fade_list(stage, &table_parts, 0.6)?;
        let aes = aes_contrast(&kit, stage)?;
        kit.hold(stage)?;

        // 12_6: the wrong clock counted bytes; the right one counts clicks
        kit.say(stage, "12_6")?;
        fade_list(stage, &aes, 0.5)?;
        let _clocks = two_clocks(&kit, stage)?;
        kit.hold(stage)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// The colour key beside the table: the ramp from 0 to 255, labelled.
fn palette_key(kit: &Kit, stage: &mut Stage<'_>, pal: &[Srgb]) -> fmn::Result<Mob> {
    let x = x_of(255.0) + 0.55;
    let colors: Vec<Vec<Srgb>> = pal.iter().rev().step_by(4).map(|&c| vec![c]).collect();
    let n = colors.len() as f64;
    let key = RowStrips::build(stage, v(x - 0.08, TOP - 0.2), (0.16, 4.6 / n), &colors)?;
    crisp(stage, &key);
    let hi = kit.text(stage, "255", 22.0, Palette::dim(), v(x + 0.42, TOP - 0.3))?;
    let lo = kit.text(stage, "0", 22.0, Palette::dim(), v(x + 0.42, TOP - 4.7))?;
    let head = kit.text(stage, "answer", 22.0, Palette::dim(), v(x, TOP + 0.05))?;
    group_of(stage, &[key.group, hi, lo, head])
}

/// 12_4's lower-third: each element order of the byte field and how many
/// bytes have it (1, 3, 5, …, 255 occur 1, 2, 4, …, 128 times).
fn element_orders() -> Vec<(u32, usize)> {
    let mut counts: Vec<(u32, usize)> = Vec::new();
    for a in 1..=255u8 {
        let o = order(a, FOLD);
        match counts.iter_mut().find(|(k, _)| *k == o) {
            Some((_, c)) => *c += 1,
            None => counts.push((o, 1)),
        }
    }
    counts.sort_unstable();
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorted_by_clicks_the_table_is_stripes_and_the_orders_are_powers_of_two() {
        // 12_4: in click order the cell (i, j) holds 2^(i + j mod 255): one
        // colour per total of clicks, a diagonal stripe each
        let exp = exps();
        for i in 0..255 {
            for j in 0..255 {
                assert_eq!(mul(exp[i], exp[j], FOLD), exp[(i + j) % 255]);
            }
        }
        // the lower-third: orders 1, 3, 5, 15, 17, 51, 85, 255 occur 1, 2, 4, …, 128 times
        let counts = element_orders();
        assert_eq!(
            counts.iter().map(|&(o, _)| o).collect::<Vec<_>>(),
            vec![1, 3, 5, 15, 17, 51, 85, 255]
        );
        assert_eq!(
            counts.iter().map(|&(_, c)| c).collect::<Vec<_>>(),
            vec![1, 2, 4, 8, 16, 32, 64, 128]
        );
        // the largest order, every nonzero byte's clock: (2+1)(2²+1)(2⁴+1)
        assert_eq!(counts.last().map(|&(o, _)| o), Some((2 + 1) * (4 + 1) * (16 + 1)));
    }
}
