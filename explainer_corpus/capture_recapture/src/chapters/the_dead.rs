//! Chapter 15, counting the dead, and the coda. Kosovo in sober type only
//! (15_1): four lists make a table of sixteen cells, one per pattern of
//! lists a person is on; the cell of people on no list is the dark one, and
//! the estimate, its range and the later count stand on one line. The
//! numbers are recorded (`facts::KOSOVO`; Ball et al. 2002, Spagat 2014).
//! Then the three tables side by side (15_2), and the coda (15_3): the fish
//! of chapter 01, the same fish with the same fates, leave the mosaic and
//! swim back into the lake; the receipts card.

use explainer_kit::props::{Pose, Swarm, lower_third, predict_pause, receipts_card};
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, thousands, v};
use fmn::prelude::*;

use super::draw::{Role, fish, flat, line_style, marked_fish, ring};
use super::lake::{
    Cell, LakeArt, PER_GLYPH, Shade, Wander, cell_backs, fly, lake_p, mosaic_cells, places_in, shuffled, wander,
};
use super::ring_toss::Spoken;
use crate::facts::{FELLER, KOSOVO, KOSOVO_LISTS};
use crate::model::{LAKE_FISH, Lake, SEED, SplitMix64};

/// The four-list table: its top-left corner and each cell's size.
const GRID_X: f64 = -6.5;
const GRID_TOP: f64 = 2.3;
const CELL: f64 = 1.05;
/// The number line of 15_1: its ends in people, and where it is drawn.
const LINE: (f64, f64) = (0.0, 13_000.0);
const LINE_X: (f64, f64) = (-1.0, 6.5);
const LINE_Y: f64 = 0.45;

/// The sources, for 15_1's lower-third (R2 Dev dev-10).
const SOURCES: &str = "four lists; log-linear models (multiple systems estimation): Ball et al. 2002, report to \
                       the ICTY; the later count compared by Spagat 2014";

/// Lists a person is on, for each of the sixteen cells: rows hold lists 1
/// and 2 (both, 1 only, 2 only, neither), columns lists 3 and 4.
fn pattern(row: usize, col: usize) -> [bool; 4] {
    let pair = |i: usize| [[true, true], [true, false], [false, true], [false, false]][i];
    let (a, b) = (pair(row), pair(col));
    [a[0], a[1], b[0], b[1]]
}

/// One cell of the four-list table: a faint square holding its pattern as
/// four dots (filled: on that list), or the dark cell for no list at all.
fn list_cell(row: usize, col: usize) -> fmn::Result<VMobject> {
    let c = v(GRID_X + CELL * (col as f64 + 0.5), GRID_TOP - CELL * (row as f64 + 0.5));
    let on = pattern(row, col);
    let dark = on.iter().all(|&x| !x);
    let back = Rectangle::new()
        .width(CELL - 0.06)
        .height(CELL - 0.06)
        .style(flat(if dark { hex("#030405") } else { hex("#1B1F26") }, 1.0))
        .build()?
        .moved_to(c);
    let mut parts = vec![back];
    for (i, &is_on) in on.iter().enumerate() {
        let (dx, dy) = ([-0.15, 0.15][i % 2], [0.15, -0.15][i / 2]);
        let color = if dark { Palette::faint() } else { Palette::dim() };
        let style = if is_on {
            flat(color, 1.0)
        } else {
            Style::default().fill(color, 0.0).stroke(color, 1.5, 1.0)
        };
        parts.push(
            Circle::new()
                .radius(0.08)
                .arc_center(v(c[0] + dx, c[1] + dy))
                .style(style)
                .build(),
        );
    }
    Ok(v_group(parts))
}

/// A small dark-celled table for 15_2: `rows` × `cols` faint cells with the
/// bottom-right one dark, as one group.
fn small_table(rows: usize, cols: usize, size: f64, centre: Vec3) -> fmn::Result<VMobject> {
    let (w, h) = (size * cols as f64, size * rows as f64);
    let mut parts = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let dark = r == rows - 1 && c == cols - 1;
            parts.push(
                Rectangle::new()
                    .width(size - 0.05)
                    .height(size - 0.05)
                    .style(flat(if dark { hex("#030405") } else { hex("#2A303A") }, 1.0))
                    .build()?
                    .moved_to(v(
                        centre[0] - w / 2.0 + size * (c as f64 + 0.5),
                        centre[1] + h / 2.0 - size * (r as f64 + 0.5),
                    )),
            );
        }
    }
    Ok(v_group(parts))
}

fn people_x(n: f64) -> f64 {
    LINE_X.0 + (LINE_X.1 - LINE_X.0) * (n - LINE.0) / (LINE.1 - LINE.0)
}

pub struct TheDead {
    pub kit: Kit,
}

impl SceneConstruct for TheDead {
    fn name(&self) -> &str {
        "15_counting_the_dead"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (named, estimate, (lo, hi), later) = KOSOVO;

        // 15_1: the four lists' table, the named, the estimate, the later count
        let title = kit.text_left(stage, "Kosovo, 1999", 30.0, Palette::dim(), v(GRID_X, 3.3))?;
        let mut cells = Vec::new();
        for r in 0..4 {
            for c in 0..4 {
                cells.push(stage.arena_mut().add(list_cell(r, c)?));
            }
        }
        let key = kit.text_left(
            stage,
            "each cell: which of the four lists name a person (filled = on it)",
            20.0,
            Palette::dim(),
            v(GRID_X, GRID_TOP - 4.0 * CELL - 0.3),
        )?;
        let dark_at = v(GRID_X + CELL * 3.5, GRID_TOP - CELL * 3.5);
        let nobody = kit.text_left(
            stage,
            "on no list",
            20.0,
            Palette::dim(),
            v(GRID_X + 4.0 * CELL + 0.15, dark_at[1]),
        )?;
        // the line of people
        let mut axis = vec![
            Line::new(v(LINE_X.0, LINE_Y), v(LINE_X.1, LINE_Y))
                .style(line_style(Palette::dim(), 2.0))
                .build()?,
        ];
        for t in [0.0, 5_000.0, 10_000.0] {
            axis.push(
                Line::new(v(people_x(t), LINE_Y - 0.07), v(people_x(t), LINE_Y + 0.07))
                    .style(line_style(Palette::dim(), 2.0))
                    .build()?,
            );
        }
        let axis = stage.arena_mut().add(v_group(axis));
        for t in [0.0, 5_000.0, 10_000.0] {
            let m = kit.text(
                stage,
                &thousands(t as u64),
                20.0,
                Palette::dim(),
                v(people_x(t), LINE_Y - 0.3),
            )?;
            stage.attach(axis, m)?;
        }
        let named_bar = stage.arena_mut().add(explainer_kit::rect(
            v(people_x(0.0), LINE_Y + 0.12),
            people_x(named as f64) - people_x(0.0),
            0.22,
            flat(Palette::dim(), 0.9),
        )?);
        let named_words = kit.text_left(
            stage,
            &format!("named on the lists: {}", thousands(named)),
            24.0,
            Palette::ink(),
            v(people_x(0.0), LINE_Y + 0.55),
        )?;
        let band = stage.arena_mut().add(explainer_kit::rect(
            v(people_x(lo as f64), LINE_Y + 0.12),
            people_x(hi as f64) - people_x(lo as f64),
            0.22,
            flat(Role::dark(), 0.4),
        )?);
        let est_tick = stage.arena_mut().add(
            Line::new(
                v(people_x(estimate as f64), LINE_Y),
                v(people_x(estimate as f64), LINE_Y + 0.55),
            )
            .style(line_style(Role::dark(), 4.0))
            .build()?,
        );
        let est_at = v(LINE_X.1, LINE_Y + 0.85);
        let est_words = kit.text(
            stage,
            &format!(
                "estimate {}; 95% range {} – {}",
                thousands(estimate),
                thousands(lo),
                thousands(hi)
            ),
            24.0,
            Role::dark(),
            est_at,
        )?;
        stage.move_to(est_words, est_at, RIGHT);
        let later_tick = stage.arena_mut().add(
            Line::new(
                v(people_x(later as f64), LINE_Y - 0.15),
                v(people_x(later as f64), LINE_Y + 0.45),
            )
            .style(line_style(Palette::ink(), 3.0))
            .build()?,
        );
        let later_at = v(LINE_X.1, LINE_Y - 0.75);
        let later_words = kit.text(
            stage,
            &format!("name-by-name count, years later: {}", thousands(later)),
            24.0,
            Palette::ink(),
            later_at,
        )?;
        stage.move_to(later_words, later_at, RIGHT);
        // what four lists buy (R3 Dev D2; 13_9 points here), right-aligned
        // under the later count, clear of "on no list"
        let four_at = v(LINE_X.1, -1.05);
        let four_buy = kit.text(
            stage,
            &format!("{KOSOVO_LISTS} lists: the model can let lists lean on each other"),
            22.0,
            Palette::ink(),
            four_at,
        )?;
        stage.move_to(four_buy, four_at, RIGHT);
        let two_at = v(LINE_X.1, -1.45);
        let two_cant = kit.text(
            stage,
            "two catches: it has to assume they don't",
            22.0,
            Palette::dim(),
            two_at,
        )?;
        stage.move_to(two_cant, two_at, RIGHT);
        // silence under the numbers (storyboard 15_1): the score fades the
        // music out in the pause before this line and keeps the stem at zero
        // until the end mark below
        kit.mark(stage, "silence:kosovo");
        let line = Spoken::say(&kit, stage, "15_1")?;
        play!(stage; fade_in(stage.arena_mut(), title, ORIGIN, 1.0)?.rt(1.0));
        line.at(stage, "four lists named")?;
        play!(stage;
            fade_in(stage.arena_mut(), axis, ORIGIN, 1.0)?.rt(0.8),
            fmn::animation::grow_from_edge(stage.arena_mut(), named_bar, LEFT, None)?.rt(1.0),
            fade_in(stage.arena_mut(), named_words, ORIGIN, 1.0)?.rt(1.0),
        );
        line.at(stage, "make a table")?;
        let mut show: Vec<Box<dyn Animation>> = Vec::new();
        for &m in cells.iter().chain([key].iter()) {
            let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(1.2);
            show.push(stage.prepare(a)?);
        }
        stage.play_prepared(show)?;
        line.at(stage, "on no list")?;
        play!(stage; fade_in(stage.arena_mut(), nobody, v(-0.2, 0.0), 1.0)?.rt(0.6));
        line.at(stage, "researchers estimated")?;
        play!(stage;
            fmn::animation::grow_from_center(stage.arena_mut(), band, None)?.rt(1.0),
            fade_in(stage.arena_mut(), est_tick, ORIGIN, 1.0)?.rt(1.0),
            fade_in(stage.arena_mut(), est_words, ORIGIN, 1.0)?.rt(1.0),
        );
        line.at(stage, "With four lists")?;
        play!(stage; fade_in(stage.arena_mut(), four_buy, ORIGIN, 1.0)?.rt(0.8));
        line.at(stage, "which two catches")?;
        play!(stage; fade_in(stage.arena_mut(), two_cant, ORIGIN, 1.0)?.rt(0.6));
        line.at(stage, "a name-by-name count")?;
        play!(stage;
            fmn::animation::grow_from_edge(stage.arena_mut(), later_tick, DOWN, None)?.rt(0.8),
            fade_in(stage.arena_mut(), later_words, ORIGIN, 1.0)?.rt(0.8),
        );
        let note = lower_third(&kit, stage, "sources:", SOURCES)?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.8));
        kit.hold(stage)?;
        // the line is over: the music may return, before 15_2 and the coda
        kit.mark(stage, "silence_end:kosovo");
        stage.wait(1.0)?;
        fade_all(stage, 1.0)?;

        // 15_2: France, the bugs, Kosovo: three tables, one dark cell each
        let tables = [
            (small_table(2, 2, 0.9, v(-4.4, 0.3))?, "France", v(-4.4, -1.1)),
            (small_table(2, 2, 0.9, v(0.0, 0.3))?, "the bugs", v(0.0, -1.1)),
            (small_table(4, 4, 0.45, v(4.4, 0.3))?, "Kosovo", v(4.4, -1.1)),
        ];
        let mut parts = Vec::new();
        for (t, word, at) in tables {
            parts.push(stage.arena_mut().add(t));
            parts.push(kit.text(stage, word, 28.0, Palette::ink(), at)?);
        }
        Spoken::say(&kit, stage, "15_2")?;
        for pair in parts.chunks(2) {
            play!(stage;
                fade_in(stage.arena_mut(), pair[0], v(0.0, -0.15), 1.0)?.rt(0.5),
                fade_in(stage.arena_mut(), pair[1], ORIGIN, 1.0)?.rt(0.5),
            );
        }
        let q = predict_pause(&kit, stage, v(0.0, 2.3))?;
        let gone = fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.4);
        let mut darks: Vec<Box<dyn Animation>> = vec![stage.prepare(gone)?];
        for &table in parts.iter().step_by(2) {
            if let Some(&dark) = explainer_kit::children(stage, table).last() {
                let teal = Role::dark();
                let tint = [teal.r as f32, teal.g as f32, teal.b as f32];
                let a = fmn::animation::indicate(stage.arena_mut(), dark, 1.25, Some(tint))?.rt(1.2);
                darks.push(stage.prepare(a)?);
            }
        }
        stage.play_prepared(darks)?;
        kit.hold(stage)?;

        // 15_3: each is a table with one cell nobody can see, filled in by
        // the cells we can; the fish of chapter 01 swim back into the lake
        let line = Spoken::say(&kit, stage, "15_3")?;
        fade_all(stage, 0.5)?;
        self.coda(&kit, stage, &line)?;
        kit.listen(stage, 3.0, "15_3")?;
        let receipts = receipts_card(
            &kit,
            stage,
            &[
                "Every number in this video was computed by code or copied from a cited source:",
                "github.com/Dicklesworthstone/jeffs_math_explainer_videos",
                "The outline of France: Natural Earth (public domain).",
            ],
        )?;
        play!(stage; fade_in(stage.arena_mut(), receipts, v(0.0, -0.2), 1.0)?.rt(0.8));
        kit.music_breath(stage, 3.0)?;
        fade_all(stage, 1.5)?;
        kit.finish(stage)
    }
}

impl TheDead {
    /// Chapter 01's story in reverse: the same thousand drawn fish with the
    /// same fates (the same seeds), from Feller's mosaic back into the lake.
    fn coda(&self, _kit: &Kit, stage: &mut Stage<'_>, line: &Spoken) -> fmn::Result<()> {
        let lake = Lake::draw();
        let drawn: Vec<Vec3> = (0..LAKE_FISH)
            .step_by(PER_GLYPH)
            .map(|i| lake_p(lake.x[i], lake.y[i]))
            .collect();
        let n = drawn.len();
        let (big_m, big_n, k) = (FELLER.0 as usize, FELLER.1 as usize, FELLER.2 as usize);
        let (dm, dn, dk) = (big_m / PER_GLYPH, big_n / PER_GLYPH, k / PER_GLYPH);
        let mut rng = SplitMix64::new(SEED ^ 0x0101);
        let order = shuffled(n, &mut rng);
        // fate: 0 caught twice, 1 first catch only, 2 second catch only, 3 neither
        let mut fate = vec![3usize; n];
        for (rank, &i) in order.iter().enumerate() {
            fate[i] = if rank < dk {
                0
            } else if rank < dm {
                1
            } else if rank < dm + dn - dk {
                2
            } else {
                3
            };
        }
        let wanders = Wander::round(&drawn, &mut rng);
        let estimate = big_m * big_n / k;
        let mosaic: [Cell; 4] = mosaic_cells(estimate as f64, big_m as f64, k as f64, big_n as f64);
        let mut place_rng = SplitMix64::new(SEED ^ 0x0103);
        let mut at_table = vec![
            Pose {
                x: 0.0,
                y: 0.0,
                heading: 0.0
            };
            n
        ];
        for (f, &cell) in mosaic.iter().enumerate() {
            let ids: Vec<usize> = (0..n).filter(|&i| fate[i] == f).collect();
            for (&i, p) in ids.iter().zip(places_in(cell, ids.len(), &mut place_rng)) {
                at_table[i] = p;
            }
        }
        let marked: Vec<usize> = (0..n).filter(|&i| fate[i] <= 1).collect();
        let second: Vec<usize> = (0..n).filter(|&i| fate[i] == 0 || fate[i] == 2).collect();
        let pick = |ids: &[usize], from: &[Pose]| -> Vec<Pose> { ids.iter().map(|&i| from[i]).collect() };
        // the mosaic, its fish in their cells
        let backs = cell_backs(stage, &mosaic)?;
        let blue = fish(Role::unmarked());
        let plain = Swarm::build(stage, &blue.mob, &at_table, None)?;
        let gold = Swarm::build(stage, &marked_fish().mob, &pick(&marked, &at_table), None)?;
        let rings = Swarm::build(stage, &ring().mob, &pick(&second, &at_table), None)?;
        // the dark cell's fish, the ones nobody caught, dim as in 01_2
        let unseen: Vec<Mob> = (0..n).filter(|&i| fate[i] == 3).map(|i| plain.members[i]).collect();
        let shade = Shade::new(stage, &unseen, &blue.paints, 0.25)?;
        play!(stage;
            fade_in(stage.arena_mut(), backs, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), plain.group, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), gold.group, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), rings.group, ORIGIN, 1.0)?.rt(0.6),
        );
        line.at(stage, "filled in by the cells")?;
        // the mosaic sinks into the lake; the fish go home
        let art = LakeArt::whole(stage)?;
        stage.add_to_scene(art)?;
        stage.set_opacity(art, 0.0, true);
        stage.scene_mut().bring_to_back(&[art])?;
        let home: Vec<Pose> = wanders.iter().map(|w| w.pose(0.0)).collect();
        let secs = 3.0;
        fly(stage, &plain, at_table.clone(), home.clone(), secs)?;
        fly(stage, &gold, pick(&marked, &at_table), pick(&marked, &home), secs)?;
        fly(stage, &rings, pick(&second, &at_table), pick(&second, &home), secs)?;
        let sink = backs
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(secs),
                ..AnimateArgs::default()
            })?
            .set_opacity(0.0)?;
        let rise = art
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(secs),
                ..AnimateArgs::default()
            })?
            .set_opacity(1.0)?;
        // and as they swim home, the fish nobody caught come into view
        play!(stage; sink, rise, shade.to(1.0, secs)?);
        // and swim, as in the first shot
        let ws_marked: Vec<Wander> = marked.iter().map(|&i| wanders[i]).collect();
        let ws_second: Vec<Wander> = second.iter().map(|&i| wanders[i]).collect();
        wander(stage, &plain, wanders.clone(), 0.0)?;
        wander(stage, &gold, ws_marked, 0.0)?;
        wander(stage, &rings, ws_second, 0.0)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chapters::ring_toss::beats_are_spoken;

    #[test]
    fn the_table_has_a_cell_for_each_mix_of_lists() {
        let mut seen = std::collections::BTreeSet::new();
        for r in 0..4 {
            for c in 0..4 {
                seen.insert(pattern(r, c));
            }
        }
        assert_eq!(seen.len() as u64, 1 << KOSOVO_LISTS);
        assert_eq!(pattern(3, 3), [false; 4], "the dark cell is the bottom-right one");
    }

    #[test]
    fn every_beat_is_spoken() {
        beats_are_spoken(&[
            ("15_1", "four lists named"),
            ("15_1", "make a table"),
            ("15_1", "on no list"),
            ("15_1", "researchers estimated"),
            ("15_1", "With four lists"),
            ("15_1", "which two catches"),
            ("15_1", "a name-by-name count"),
            ("15_3", "filled in by the cells"),
        ])
        .unwrap();
    }
}
