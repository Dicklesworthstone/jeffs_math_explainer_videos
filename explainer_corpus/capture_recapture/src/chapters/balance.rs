//! Chapter 08, the table balances: Feller's mosaic with a sliver of a dark
//! cell (08_1), the dark cell growing as the lake grows while each column's
//! green share is read off and the score climbs (08_2), the shares meeting
//! at 8,100 with the cross products equal, 100 × 8,100 = 900 × 900 (08_3),
//! the top score and the soup guess agreeing (08_4), and the tie at 9,999
//! and 10,000 from Feller's neighbour ratios (08_5).

use std::rc::Rc;

use explainer_kit::props::{precisely, predict_pause};
use explainer_kit::{Frame2, Kit, Palette, Timed, hex, ink_style, play, thousands, tween, v};
use fmn::prelude::*;

use super::draw::{Role, flat, line_style};
use super::lake::{Cell, GAP, Panel, cell_tints, fit, header_places, mosaic_in};
use crate::exact::{feller_ratio, relative_scores};
use crate::facts::{FELLER, TOY, best_sizes};

/// Where Feller's mosaic sits in 08 (and where 07 hands it over).
pub const PANEL: Panel = Panel {
    c: [1.5, -0.1, 0.0],
    w: 8.2,
    h: 4.6,
};

/// The sweep starts with a sliver of a dark cell: this many fish more than
/// the smallest possible lake.
pub const SLIVER: u64 = 100;

/// The lake size the chapter opens on.
pub fn opening_lake() -> u64 {
    let (m, n, k) = FELLER;
    m + n - k + SLIVER
}

/// Feller's mosaic at lake size `lake`.
pub fn cells_at(lake: f64) -> [Cell; 4] {
    let (m, n, k) = FELLER;
    mosaic_in(PANEL, lake, m as f64, k as f64, n as f64)
}

/// The two second-catch dividers' heights: the bottom edge of the green
/// part in the marked column, then in the unmarked column.
fn dividers(cells: &[Cell; 4]) -> (f64, f64) {
    let bottom = |c: &Cell| c.c[1] - c.h / 2.0 - GAP / 2.0;
    (bottom(&cells[0]), bottom(&cells[2]))
}

/// A text glyph group from the kit's fonts (for redrawn pictures).
fn text_vm(book: &FontBook, s: &str, size: f64, color: Srgb) -> Option<VMobject> {
    Text::new(s)
        .font_size(size)
        .style(ink_style(color))
        .build(book)
        .ok()
        .map(|t| t.vmob)
}

/// Feller's mosaic at `lake` as one picture: the cells, the dark cell's
/// rim, both dividers (one gold line once they meet), each cell's count
/// and the unmarked column's header.
pub fn mosaic_picture(book: &FontBook, lake: f64) -> VMobject {
    let (m, n, k) = FELLER;
    let cells = cells_at(lake);
    let mut parts: Vec<VMobject> = Vec::new();
    for (c, t) in cells.iter().zip(cell_tints()) {
        if let Ok(r) = Rectangle::new().width(c.w).height(c.h).style(flat(t, 1.0)).build() {
            parts.push(r.moved_to(c.c));
        }
    }
    let dark = cells[3];
    if let Ok(r) = Rectangle::new()
        .width(dark.w)
        .height(dark.h)
        .style(Style::default().fill(Role::dark(), 0.0).stroke(Role::dark(), 1.5, 0.45))
        .build()
    {
        parts.push(r.moved_to(dark.c));
    }
    let (yl, yr) = dividers(&cells);
    let straight = (yl - yr).abs() < 0.004;
    let (color, width) = if straight {
        (Palette::hero(), 7.0)
    } else {
        (Role::second(), 5.0)
    };
    let left = cells[0].c[0] - cells[0].w / 2.0 - GAP / 2.0;
    let mid = cells[2].c[0] - cells[2].w / 2.0 - GAP / 2.0;
    let right = cells[2].c[0] + cells[2].w / 2.0 + GAP / 2.0;
    for (x0, x1, y) in [(left, mid, yl), (mid, right, yr)] {
        if let Ok(l) = Line::new(v(x0, y), v(x1, y)).style(line_style(color, width)).build() {
            parts.push(l);
        }
    }
    let lake_n = lake.round() as u64;
    let counts = [
        (thousands(k), Role::marked()),
        (thousands(m - k), Role::marked()),
        (thousands(n - k), Role::second()),
        (thousands(lake_n.saturating_sub(m + n - k)), Role::dark()),
    ];
    for (cell, (s, color)) in cells.iter().zip(counts) {
        if let Some(t) = text_vm(book, &s, 40.0, color) {
            let (w, h) = t.extent().map_or((1.0, 1.0), |(lo, hi)| (hi[0] - lo[0], hi[1] - lo[1]));
            let f = fit(w, h, *cell);
            parts.push(t.scaled_about(f, ORIGIN).moved_to(cell.c));
        }
    }
    let top = PANEL.c[1] + PANEL.h / 2.0 + 0.35;
    if let Some(t) = text_vm(book, "unmarked", 28.0, Palette::ink()) {
        parts.push(t.moved_to(v(cells[2].c[0], top)));
    }
    v_group(parts)
}

/// The mosaic's fixed words: the marked column's header and the two row
/// words (the marked column keeps its rows at every lake size).
pub fn fixed_labels(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let places = header_places(&cells_at(opening_lake() as f64));
    let words = [
        (0, "marked (first catch)", 28.0, Role::marked()),
        (2, "in the second catch", 26.0, Role::second()),
        (3, "not", 26.0, Palette::dim()),
    ];
    let g = stage.arena_mut().add(VMobject::new());
    for (i, s, size, c) in words {
        let (at, edge) = places[i];
        let m = kit.text(stage, s, size, c, at)?;
        stage.move_to(m, at, edge);
        stage.attach(g, m)?;
    }
    Ok(g)
}

/// The small score plot of the left panel.
fn score_plot() -> Frame2 {
    Frame2 {
        center: v(-5.25, -1.85),
        width: 3.1,
        height: 1.05,
        x: (1_900.0, 16_000.0),
        y: (0.0, 1.12),
    }
}

/// A rounded card of `w` × `h` at `at` (the corpus's card look).
fn card(w: f64, h: f64, at: Vec3, edge: Srgb) -> fmn::Result<VMobject> {
    Ok(Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(Style::default().fill(hex("#1A2130"), 0.94).stroke(edge, 2.0, 0.9))
        .build()?
        .moved_to(at))
}

pub struct Balance {
    pub kit: Kit,
}

impl SceneConstruct for Balance {
    fn name(&self) -> &str {
        "08_the_table_balances"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (m, n, k) = FELLER;
        let truth = m * n / k;
        let start = opening_lake() as f64;
        let book = Rc::clone(&kit.book);

        // 08_1: the mosaic with a sliver of a dark cell
        let lake = stage.add_value_tracker(start);
        let b = Rc::clone(&book);
        let mosaic = stage.always_redraw(move |s| {
            let n = s.tracker_value(lake).unwrap_or(start);
            s.add(mosaic_picture(&b, n))
        });
        let labels = fixed_labels(&kit, stage)?;
        // the left panel: the lake size, the two green shares, the score
        let x0 = -6.8;
        let size_word = kit.text_left(stage, "lake size", 24.0, Palette::dim(), v(x0, 1.6))?;
        let share_word = kit.text_left(stage, "green share", 24.0, Palette::dim(), v(x0, 0.3))?;
        let left_word = kit.text_left(stage, "marked column", 22.0, Role::marked(), v(x0, -0.12))?;
        let right_word = kit.text_left(stage, "unmarked column", 22.0, Palette::ink(), v(x0, -0.55))?;
        let left_pct = kit.text(
            stage,
            &format!("{:.1}%", 100.0 * k as f64 / m as f64),
            28.0,
            Role::second(),
            v(0.0, -0.12),
        )?;
        stage.move_to(left_pct, v(-3.75, -0.12), RIGHT);
        let right_pct = kit.text_left(stage, "%", 28.0, Role::second(), v(-4.0, -0.55))?;
        let sp = score_plot();
        let rel = Rc::new(relative_scores(FELLER, sp.x.0 as u64, sp.x.1 as u64));
        let pts: Vec<Vec3> = rel
            .iter()
            .enumerate()
            .step_by(25)
            .map(|(i, &r)| sp.p(sp.x.0 + i as f64, r))
            .collect();
        let score_curve = stage
            .arena_mut()
            .add(Polygon::polyline(pts).style(line_style(Palette::purple(), 3.0)).build());
        let floor = stage.arena_mut().add(
            Line::new(v(sp.left(), sp.bottom()), v(sp.right(), sp.bottom()))
                .style(line_style(Palette::dim(), 2.0))
                .build()?,
        );
        let score_word = kit.text_left(stage, "score", 22.0, Palette::purple(), v(x0, sp.top() + 0.22))?;
        let r2 = Rc::clone(&rel);
        let dot = stage.always_redraw(move |s| {
            let n_lake = s.tracker_value(lake).unwrap_or(start);
            let i = ((n_lake - sp.x.0).max(0.0) as usize).min(r2.len() - 1);
            s.add(
                Circle::new()
                    .radius(0.07)
                    .arc_center(sp.p(n_lake, r2[i]))
                    .style(flat(Palette::hero(), 1.0))
                    .build(),
            )
        });
        kit.say(stage, "08_1")?;
        // the opening picture fades in still; the live mosaic takes over
        // (identical at this lake size) before anything moves
        let still = stage.arena_mut().add(mosaic_picture(&book, start));
        play!(stage;
            fade_in(stage.arena_mut(), still, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), labels, ORIGIN, 1.0)?.rt(0.8),
        );
        stage.remove_from_scene(still);
        stage.add_to_scene(mosaic)?;
        let mut panel_in: Vec<Box<dyn Animation>> = Vec::new();
        for mm in [
            size_word,
            share_word,
            left_word,
            right_word,
            left_pct,
            right_pct,
            score_curve,
            floor,
            score_word,
        ] {
            let a = fade_in(stage.arena_mut(), mm, ORIGIN, 1.0)?;
            panel_in.push(stage.prepare(a)?);
        }
        stage.play_prepared(panel_in)?;
        // the live readouts join once their words are in (a redrawn number
        // cannot fade)
        let size_no = kit.live_count(stage, lake, 40.0, Role::dark(), v(x0 + 1.15, 1.05))?;
        let right_no = kit.live_value(
            stage,
            move |s| {
                let n_lake = s.tracker_value(lake).unwrap_or(start).round();
                100.0 * (n - k) as f64 / (n_lake - m as f64)
            },
            1,
            false,
            28.0,
            Role::second(),
            v(-4.05, -0.55),
            RIGHT,
        )?;
        stage.add_to_scene(dot)?;
        let q = predict_pause(&kit, stage, v(4.9, -1.4))?;

        // 08_2: the lake grows; the dark cell rises out of nothing and the
        // unmarked column's divider climbs toward the marked one's
        kit.say(stage, "08_2")?;
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        stage.play(tween(lake, truth as f64, 8.5)?)?;
        kit.hold(stage)?;

        // 08_3: the cross products, and the line goes straight
        let cells = cells_at(truth as f64);
        let (a, b_, u) = (m - k, n - k, truth - (m + n - k));
        // the two cells of each product, outlined in the product's colour
        let rim = |stage: &mut Stage<'_>, cells: [&Cell; 2], color: Srgb| -> fmn::Result<Mob> {
            let mut parts = Vec::new();
            for c in cells {
                parts.push(
                    Rectangle::new()
                        .width(c.w + 0.02)
                        .height(c.h + 0.02)
                        .style(line_style(color, 4.0))
                        .build()?
                        .moved_to(c.c),
                );
            }
            Ok(stage.arena_mut().add(v_group(parts)))
        };
        let ku = rim(stage, [&cells[0], &cells[3]], Palette::hero())?;
        let ab = rim(stage, [&cells[1], &cells[2]], Role::second())?;
        let ku_word = kit.text(
            stage,
            &format!("{} × {} = {}", thousands(k), thousands(u), thousands(k * u)),
            32.0,
            Palette::hero(),
            v(cells[3].c[0] + 0.6, cells[3].c[1] - 1.0),
        )?;
        let ab_word = kit.text(
            stage,
            &format!("{} × {} = {}", thousands(a), thousands(b_), thousands(a * b_)),
            32.0,
            Role::second(),
            v(cells[3].c[0] + 0.6, cells[3].c[1] + 0.9),
        )?;
        kit.say(stage, "08_3")?;
        play!(stage;
            show_creation(ku).rt(0.8),
            fade_in(stage.arena_mut(), ku_word, ORIGIN, 1.0)?.rt(0.8),
        );
        play!(stage;
            fade_out(stage.arena_mut(), ku, ORIGIN, 1.0)?.rt(0.5),
            show_creation(ab).rt(0.8),
            fade_in(stage.arena_mut(), ab_word, ORIGIN, 1.0)?.rt(0.8),
        );
        kit.listen(stage, 5.0, "08_3")?;

        // 08_4: a straight line is independence on average; the top score
        // and the soup guess agree (rounded down to a whole fish)
        stage.clear_updaters(mosaic, true);
        stage.clear_updaters(dot, true);
        stage.clear_updaters(size_no, true);
        stage.clear_updaters(right_no, true);
        let mut clear: Vec<Box<dyn Animation>> = Vec::new();
        for mm in [
            ku,
            ab,
            share_word,
            left_word,
            right_word,
            left_pct,
            right_no,
            right_pct,
            score_curve,
            floor,
            score_word,
            dot,
        ] {
            let a = fade_out(stage.arena_mut(), mm, ORIGIN, 1.0)?;
            clear.push(stage.prepare(a)?);
        }
        // the score thins the bed into the key line (music lane)
        kit.mark(stage, "thin:08_4");
        kit.say(stage, "08_4")?;
        stage.play_prepared_with(
            clear,
            PlayOverrides {
                run_time: Some(0.6),
                ..PlayOverrides::default()
            },
        )?;
        // the two cards, then one
        let cx = -5.1;
        let top_card = stage.arena_mut().add(card(3.5, 1.0, v(cx, 0.55), Palette::purple())?);
        let top_t1 = kit.text(stage, "top score", 24.0, Palette::purple(), v(cx, 0.75))?;
        let top_t2 = kit.text(stage, &thousands(truth), 30.0, Palette::ink(), v(cx, 0.33))?;
        let soup_card = stage.arena_mut().add(card(3.5, 1.0, v(cx, -0.75), Role::dark())?);
        let soup_t1 = kit.text(stage, "soup guess", 24.0, Role::dark(), v(cx, -0.55))?;
        let soup_t2 = kit.text(
            stage,
            &format!(
                "{} × {} ÷ {} = {}",
                thousands(m),
                thousands(n),
                thousands(k),
                thousands(truth)
            ),
            24.0,
            Palette::ink(),
            v(cx, -0.97),
        )?;
        let mut cards_in: Vec<Box<dyn Animation>> = Vec::new();
        for mm in [top_card, top_t1, top_t2, soup_card, soup_t1, soup_t2] {
            let a = fade_in(stage.arena_mut(), mm, v(0.0, -0.1), 1.0)?;
            cards_in.push(stage.prepare(a)?);
        }
        stage.play_prepared(cards_in)?;
        let agree_card = stage.arena_mut().add(card(3.5, 1.15, v(cx, -0.1), Palette::hero())?);
        let agree_t1 = kit.text(stage, "they agree", 26.0, Palette::hero(), v(cx, 0.15))?;
        let agree_t2 = kit.text(stage, &thousands(truth), 34.0, Palette::ink(), v(cx, -0.3))?;
        let mut merge: Vec<Box<dyn Animation>> = Vec::new();
        for (from, to) in [
            (top_card, agree_card),
            (soup_card, agree_card),
            (top_t1, agree_t1),
            (soup_t1, agree_t1),
            (top_t2, agree_t2),
            (soup_t2, agree_t2),
        ] {
            let a = fmn::animation::fade_transform(stage.arena_mut(), from, to)?;
            merge.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            merge,
            PlayOverrides {
                run_time: Some(1.2),
                ..PlayOverrides::default()
            },
        )?;
        // the line glows; the mystery lake rounds down
        let (yl, _) = dividers(&cells);
        let x_left = cells[0].c[0] - cells[0].w / 2.0;
        let x_right = cells[2].c[0] + cells[2].w / 2.0;
        let glow_line = stage.arena_mut().add(
            Line::new(v(x_left, yl), v(x_right, yl))
                .style(line_style(Palette::hero(), 9.0))
                .build()?,
        );
        stage.add_to_scene(glow_line)?;
        play!(stage; fmn::animation::indicate(stage.arena_mut(), glow_line, 1.0, None)?.rt(1.2));
        let (tm, tn, tk) = TOY;
        let toy_best = best_sizes(TOY).last().copied().unwrap_or(0);
        let toy_word = kit.text(stage, "the mystery lake", 22.0, Palette::dim(), v(cx, -1.25))?;
        let toy = kit.tex(
            stage,
            &format!(
                r"{} \div {} = {}\tfrac{{{}}}{{{}}} \;\to\; {}",
                tm * tn,
                tk,
                tm * tn / tk,
                (tm * tn) % tk,
                tk,
                toy_best
            ),
            36.0,
            Palette::ink(),
            &[],
            v(cx, -1.75),
        )?;
        stage.attach(toy, toy_word)?;
        play!(stage; fade_in(stage.arena_mut(), toy, ORIGIN, 1.0)?.rt(0.6));
        let note = precisely(
            &kit,
            stage,
            "k·u = a·b: the table's odds ratio is 1, the 2×2 picture of independence; observed tables wobble \
             around it (chapter 09)",
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.8));
        kit.hold(stage)?;

        // 08_5: the line is exactly straight at 10,000, so 9,999 ties
        let best = best_sizes(FELLER);
        let mut rows: Vec<Mob> = Vec::new();
        let ratio_rows = [(truth - 1, ">"), (truth, "="), (truth + 1, "<")];
        for (i, &(lake_n, rel_sign)) in ratio_rows.iter().enumerate() {
            let (num, den) = feller_ratio(lake_n, FELLER);
            let y = 1.25 - 1.15 * i as f64;
            let head = kit.text(
                stage,
                &format!("{} vs {}", thousands(lake_n), thousands(lake_n - 1)),
                20.0,
                Palette::dim(),
                v(cx, y + 0.42),
            )?;
            let tex = kit.tex(
                stage,
                &format!(
                    r"\frac{{{}}}{{{}}} {} 1",
                    thousands(num).replace(',', "{,}"),
                    thousands(den).replace(',', "{,}"),
                    rel_sign
                ),
                30.0,
                if rel_sign == "=" {
                    Palette::hero()
                } else {
                    Palette::ink()
                },
                &[],
                v(cx, y - 0.15),
            )?;
            rows.push(head);
            rows.push(tex);
        }
        let tie = kit.text(
            stage,
            &format!(
                "top score: {}",
                best.iter().map(|&b| thousands(b)).collect::<Vec<_>>().join(" and ")
            ),
            24.0,
            Palette::hero(),
            v(cx, -2.25),
        )?;
        let note2 = precisely(
            &kit,
            stage,
            "the top score's size is ⌊Mn/k⌋; a tie with Mn/k − 1 when Mn/k is whole and k < min(M, n); none \
             when k = 0; exact fractions tie, an lgamma log-likelihood picks 10,000 by rounding",
        )?;
        kit.say(stage, "08_5")?;
        let mut swap: Vec<Box<dyn Animation>> = Vec::new();
        stage.clear_updaters(size_no, true);
        for mm in [agree_card, agree_t1, agree_t2, toy, note, size_word, size_no] {
            let a = fade_out(stage.arena_mut(), mm, ORIGIN, 1.0)?;
            swap.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            swap,
            PlayOverrides {
                run_time: Some(0.5),
                ..PlayOverrides::default()
            },
        )?;
        for pair in rows.chunks(2) {
            play!(stage;
                fade_in(stage.arena_mut(), pair[0], ORIGIN, 1.0)?.rt(0.5),
                fade_in(stage.arena_mut(), pair[1], v(0.0, -0.1), 1.0)?.rt(0.5),
            );
        }
        play!(stage;
            fade_in(stage.arena_mut(), tie, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), note2, v(0.0, 0.2), 1.0)?.rt(0.8),
        );
        kit.hold(stage)?;
        explainer_kit::fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_chapter_08_says_holds_exactly() {
        let (m, n, k) = FELLER;
        let truth = m * n / k;
        // 08_2: while the unmarked column's green share is the bigger one,
        // the score climbs: Mn − kN > 0 ⇔ (n − k)/(N − M) > k/M ⇔ L(N) > L(N − 1)
        for lake in (m + n - k + 1)..=(truth + 50) {
            let unmarked_bigger = (n - k) * m > k * (lake - m);
            let (num, den) = feller_ratio(lake, FELLER);
            assert_eq!(unmarked_bigger, num > den, "N {lake}");
        }
        // 08_3: 100 × 8,100 = 810,000 = 900 × 900, and the shares match
        let u = truth - (m + n - k);
        assert_eq!((k * u, (m - k) * (n - k)), (810_000, 810_000));
        assert_eq!((n - k) * m, k * (truth - m));
        // the dividers meet exactly at the balance, and not one fish before
        let (yl, yr) = dividers(&cells_at(truth as f64));
        assert!((yl - yr).abs() < 1e-9);
        let (yl, yr) = dividers(&cells_at((truth - 1) as f64));
        assert!(
            yr < yl,
            "with green on top the unmarked divider sits lower before the balance"
        );
        // 08_4: "like thirty-three from thirty-three and a third"
        let (tm, tn, tk) = TOY;
        assert_eq!((tm * tn / tk, (tm * tn) % tk, tk), (33, 1, 3));
        assert_eq!(best_sizes(TOY), vec![33]);
        // 08_5: the step up from 9,999 changes nothing; the two tie
        assert_eq!(feller_ratio(truth, FELLER), (81_000_000, 81_000_000));
        assert_eq!(best_sizes(FELLER), vec![truth - 1, truth]);
    }

    #[test]
    fn the_opening_has_a_sliver_of_a_dark_cell() {
        let cells = cells_at(opening_lake() as f64);
        let dark_area = cells[3].w * cells[3].h;
        assert!(dark_area > 0.05 * PANEL.w * PANEL.h * 0.5 && dark_area < 0.1 * PANEL.w * PANEL.h);
    }
}
