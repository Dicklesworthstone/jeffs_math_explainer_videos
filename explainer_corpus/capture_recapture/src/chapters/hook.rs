//! Chapter 01, the hook: the fish nobody caught. Feller's story drawn one
//! fish per ten on the seeded lake (01_1), the catches flying into the table
//! and the table growing into the mosaic (01_2–01_3), the title over the
//! living lake (01_4), then one net dipped and scaled up, three times
//! (01_5–01_8; the lake is `model.rs`, which is oracle A-F27 fish for fish).

use explainer_kit::props::{Pose, RowStrips, Swarm, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, children, fade_all, hex, play, thousands, tween, v};
use fmn::core::color::interpolate_color;
use fmn::prelude::*;

use super::draw::{Role, fish, flat, line_style, marked_fish, ring};
use super::lake::{
    Cell, LAKE_H, LAKE_W, LakeArt, PER_GLYPH, Shade, Wander, behind, cell_backs, cell_numbers, cell_rect, fade_out_all,
    fit, fly, header_places, key, lake_p, mosaic_cells, places_in, plated, shuffled, table_cells, table_labels, wander,
};
use crate::facts::FELLER;
use crate::model::{GRID, LAKE_FISH, Lake, SEED, SPOTS, SplitMix64, cell_of};

/// Which catch a drawn fish was in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fate {
    Twice,
    FirstOnly,
    SecondOnly,
    Untouched,
}

impl Fate {
    fn cell(self) -> usize {
        match self {
            Self::Twice => 0,
            Self::FirstOnly => 1,
            Self::SecondOnly => 2,
            Self::Untouched => 3,
        }
    }
}

/// How bright the fish no net touched stay once the table forms.
const SILHOUETTE: f64 = 0.28;
/// The widest the title may be (the frame is 14.2 wide).
const TITLE_MAX_W: f64 = 12.4;

pub struct Hook {
    pub kit: Kit,
}

/// An animation that moves (and optionally rescales) `m`, prepared.
fn glide(
    stage: &mut Stage<'_>,
    m: Mob,
    to: Vec3,
    edge: Vec3,
    scale: f64,
    secs: f64,
) -> fmn::Result<Box<dyn Animation>> {
    let mut a = m
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(secs),
            ..AnimateArgs::default()
        })?
        .move_to(to, edge)?;
    if (scale - 1.0).abs() > 1e-9 {
        a = a.scale(scale)?;
    }
    stage.prepare(a)
}

impl SceneConstruct for Hook {
    fn name(&self) -> &str {
        "01_hook"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let lake = Lake::draw();
        // the drawn fish: one in ten of the lake's fish
        let drawn: Vec<Vec3> = (0..LAKE_FISH)
            .step_by(PER_GLYPH)
            .map(|i| lake_p(lake.x[i], lake.y[i]))
            .collect();
        let n = drawn.len();
        let (big_m, big_n, k) = (FELLER.0 as usize, FELLER.1 as usize, FELLER.2 as usize);
        let (dm, dn, dk) = (big_m / PER_GLYPH, big_n / PER_GLYPH, k / PER_GLYPH);
        // who was caught: seeded, so every render draws the same story
        let mut rng = SplitMix64::new(SEED ^ 0x0101);
        let order = shuffled(n, &mut rng);
        let mut fate = vec![Fate::Untouched; n];
        for (rank, &i) in order.iter().enumerate() {
            fate[i] = if rank < dk {
                Fate::Twice
            } else if rank < dm {
                Fate::FirstOnly
            } else if rank < dm + dn - dk {
                Fate::SecondOnly
            } else {
                Fate::Untouched
            };
        }
        let marked: Vec<usize> = (0..n)
            .filter(|&i| matches!(fate[i], Fate::Twice | Fate::FirstOnly))
            .collect();
        let second: Vec<usize> = (0..n)
            .filter(|&i| matches!(fate[i], Fate::Twice | Fate::SecondOnly))
            .collect();
        let wanders = Wander::round(&drawn, &mut rng);

        // 01_1: the lake, a thousand fish, the two catches
        let art = LakeArt::whole(stage)?;
        let the_key = key(&kit, stage)?;
        let blue = fish(Role::unmarked());
        let poses0: Vec<Pose> = wanders.iter().map(|w| w.pose(0.0)).collect();
        let plain = Swarm::build(stage, &blue.mob, &poses0, None)?;
        kit.say(stage, "01_1")?;
        play!(stage;
            fade_in(stage.arena_mut(), art, ORIGIN, 1.0)?.rt(1.2),
            fade_in(stage.arena_mut(), plain.group, ORIGIN, 1.0)?.rt(1.6),
            fade_in(stage.arena_mut(), the_key, ORIGIN, 1.0)?.rt(1.0),
        );
        // the wander clock starts with the wandering: every later swarm joins
        // it at `now`, so the gold bodies and the rings sit on their fish
        let t_start = stage.scene().time().to_f64();
        wander(stage, &plain, wanders.clone(), 0.0)?;
        stage.wait(1.0)?;
        let now = |stage: &Stage<'_>| stage.scene().time().to_f64() - t_start;
        // the first catch: marked, glowing, let go where it was
        let t = now(stage);
        let gold_glyph = marked_fish();
        let gold_w: Vec<Wander> = marked.iter().map(|&i| wanders[i]).collect();
        let gold_poses: Vec<Pose> = gold_w.iter().map(|w| w.pose(t)).collect();
        let gold = Swarm::build(stage, &gold_glyph.mob, &gold_poses, None)?;
        let gold_shade = Shade::new(stage, &gold.members, &gold_glyph.paints, 0.0)?;
        stage.add_to_scene(gold.group)?;
        wander(stage, &gold, gold_w, t)?;
        stage.play(gold_shade.to(1.0, 1.2)?)?;
        stage.wait(1.0)?;
        // the second catch: a green ring on each fish it caught
        let t = now(stage);
        let ring_glyph = ring();
        let ring_w: Vec<Wander> = second.iter().map(|&i| wanders[i]).collect();
        let ring_poses: Vec<Pose> = ring_w.iter().map(|w| w.pose(t)).collect();
        let rings = Swarm::build(stage, &ring_glyph.mob, &ring_poses, None)?;
        let ring_shade = Shade::new(stage, &rings.members, &ring_glyph.paints, 0.0)?;
        stage.add_to_scene(rings.group)?;
        wander(stage, &rings, ring_w, t)?;
        stage.play(ring_shade.to(1.0, 1.0)?)?;
        kit.hold(stage)?;

        // 01_2: the fish no net touched dim to silhouettes, the lake gives
        // way to the table, and every fish flies to its cell
        let untouched: Vec<Mob> = (0..n)
            .filter(|&i| fate[i] == Fate::Untouched)
            .map(|i| plain.members[i])
            .collect();
        let dim = Shade::new(stage, &untouched, &blue.paints, 1.0)?;
        let cells = table_cells(0.5, 0.5);
        let backs = cell_backs(stage, &cells)?;
        behind(stage, backs)?;
        stage.set_opacity(backs, 0.0, true);
        let mut place_rng = SplitMix64::new(SEED ^ 0x0102);
        let targets = targets_for(&fate, &cells, &mut place_rng);
        kit.say(stage, "01_2")?;
        stage.play(dim.to(SILHOUETTE, 1.2)?)?;
        let t = now(stage);
        let here = |ids: &[usize]| -> Vec<Pose> { ids.iter().map(|&i| wanders[i].pose(t)).collect() };
        let all: Vec<usize> = (0..n).collect();
        let to = |ids: &[usize]| -> Vec<Pose> { ids.iter().map(|&i| targets[i]).collect() };
        fly(stage, &plain, here(&all), to(&all), 2.4)?;
        fly(stage, &gold, here(&marked), to(&marked), 2.4)?;
        fly(stage, &rings, here(&second), to(&second), 2.4)?;
        let back_in = backs
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.4),
                ..AnimateArgs::default()
            })?
            .set_opacity(1.0)?;
        play!(stage;
            fade_out(stage.arena_mut(), art, ORIGIN, 1.0)?.rt(1.4),
            back_in,
        );
        stage.wait(1.1)?;
        let headers = table_labels(&kit, stage, &cells)?;
        let numbers = cell_numbers(&kit, stage, &cells, [k, big_m - k, big_n - k])?;
        let question = plated(&kit, stage, "?", 96.0, Palette::dim(), cells[3].c)?;
        play!(stage;
            fade_in(stage.arena_mut(), headers, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), numbers, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), question, ORIGIN, 1.0)?.rt(0.8),
        );
        kit.hold(stage)?;

        // 01_3: the best guess for the dark cell, and the table becomes the
        // mosaic: areas in proportion
        let u = (big_m - k) * (big_n - k) / k;
        let estimate = big_m * big_n / k;
        kit.say(stage, "01_3")?;
        let count = stage.add_value_tracker(0.0);
        // the guess's plate first, the count climbing on it, then the guess
        let guess = plated(
            &kit,
            stage,
            &format!("≈ {}", thousands(u as u64)),
            60.0,
            Palette::ink(),
            cells[3].c,
        )?;
        let guess_text = children(stage, guess)[1];
        stage.set_opacity(guess_text, 0.0, true);
        play!(stage;
            fade_out(stage.arena_mut(), question, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), guess, ORIGIN, 1.0)?.rt(0.4),
        );
        let live = kit.live_count(stage, count, 60.0, Palette::ink(), cells[3].c)?;
        stage.play(tween(count, u as f64, 1.4)?)?;
        stage.remove_from_scene(live);
        stage.set_opacity(guess_text, 1.0, true);
        // the mosaic: the marked column M of an estimated Mn/k wide, each
        // column's second catch on top
        let mosaic = mosaic_cells(estimate as f64, big_m as f64, k as f64, big_n as f64);
        let mut place_rng = SplitMix64::new(SEED ^ 0x0103);
        let mosaic_targets = targets_for(&fate, &mosaic, &mut place_rng);
        let to2 = |ids: &[usize]| -> Vec<Pose> { ids.iter().map(|&i| mosaic_targets[i]).collect() };
        let secs = 2.6;
        fly(stage, &plain, to(&all), to2(&all), secs)?;
        fly(stage, &gold, to(&marked), to2(&marked), secs)?;
        fly(stage, &rings, to(&second), to2(&second), secs)?;
        let mut morph: Vec<Box<dyn Animation>> = Vec::new();
        for (back, cell) in children(stage, backs).into_iter().zip(mosaic) {
            let a = back
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(secs),
                    ..AnimateArgs::default()
                })?
                .set_width(cell.w, true)?
                .set_height(cell.h, true)?
                .move_to(cell.c, ORIGIN)?;
            morph.push(stage.prepare(a)?);
        }
        // the headers follow their columns and rows; each number shrinks to
        // fit its new cell
        for (m, (at, edge)) in children(stage, headers).into_iter().zip(header_places(&mosaic)) {
            morph.push(glide(stage, m, at, edge, 1.0, secs)?);
        }
        for (m, cell) in children(stage, numbers).into_iter().zip(mosaic) {
            let f = fit(stage.get_width(m), stage.get_height(m), cell);
            morph.push(glide(stage, m, cell.c, ORIGIN, f, secs)?);
        }
        morph.push(glide(stage, guess, mosaic[3].c, ORIGIN, 1.0, secs)?);
        // the top row is the second catch now: its rings would only pile up
        morph.push(stage.prepare(ring_shade.to(0.0, secs)?)?);
        stage.play_prepared(morph)?;
        kit.hold(stage)?;

        // 01_4: the title over the living lake, the motif unresolved (the
        // lake comes in while the title writes; the title leaves under 01_5)
        fade_all(stage, 0.5)?;
        let scene = DipLake::build(stage, &drawn)?;
        // a veil over the lake (and its bank) so the title reads
        let veil = stage.arena_mut().add(
            Rectangle::new()
                .width(LAKE_W + 0.3)
                .height(LAKE_H + 0.3)
                .corner_radius(super::lake::WATER_R + 0.12)
                .style(flat(hex("#0B0F14"), 0.62))
                .build()?
                .moved_to(super::lake::LAKE_C),
        );
        let title = kit.text_styled(
            stage,
            "How to Count Fish You've Never Seen",
            72.0,
            Palette::hero(),
            v(0.0, 0.2),
            true,
            false,
        )?;
        let w = stage.get_width(title);
        if w > TITLE_MAX_W {
            stage.scale(title, TITLE_MAX_W / w);
        }
        // the school is moving, so it fades by opacity (never a Transform);
        // the lake goes in first and fades by opacity too, under the fish (a
        // fade_in would re-root it on top of them)
        stage.add_to_scene(scene.art)?;
        stage.set_opacity(scene.art, 0.0, true);
        let school_shade = Shade::new(stage, &scene.school.members, &fish(Role::unmarked()).paints, 0.0)?;
        stage.add_to_scene(scene.school.group)?;
        wander(stage, &scene.school, scene.wanders.clone(), 0.0)?;
        let art_in = scene
            .art
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.2),
                ..AnimateArgs::default()
            })?
            .set_opacity(1.0)?;
        play!(stage;
            art_in,
            school_shade.to(1.0, 1.2)?,
            fade_in(stage.arena_mut(), veil, ORIGIN, 1.0)?.rt(0.6),
            write(stage.arena(), title).rt(1.3),
        );
        kit.listen(stage, 3.0, "01_4")?;

        // 01_5–01_8: a 10 × 10 grid, one net dipped
        dips(&kit, stage, &lake, &scene, [title, veil])?;

        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// Where each drawn fish sits in the table: its fate's cell, on a grid.
fn targets_for(fate: &[Fate], cells: &[Cell; 4], rng: &mut SplitMix64) -> Vec<Pose> {
    let mut out = vec![
        Pose {
            x: 0.0,
            y: 0.0,
            heading: 0.0
        };
        fate.len()
    ];
    for f in [Fate::Twice, Fate::FirstOnly, Fate::SecondOnly, Fate::Untouched] {
        let ids: Vec<usize> = (0..fate.len()).filter(|&i| fate[i] == f).collect();
        for (&i, p) in ids.iter().zip(places_in(cells[f.cell()], ids.len(), rng)) {
            out[i] = p;
        }
    }
    out
}

/// The lake of the dips: its picture and every drawn fish wandering round
/// its own place (01_4–01_8).
struct DipLake {
    art: Mob,
    school: Swarm,
    wanders: Vec<Wander>,
}

impl DipLake {
    fn build(stage: &mut Stage<'_>, drawn: &[Vec3]) -> fmn::Result<Self> {
        let art = LakeArt::whole(stage)?;
        let mut rng = SplitMix64::new(SEED ^ 0x0105);
        let wanders = Wander::round(drawn, &mut rng);
        let poses: Vec<Pose> = wanders.iter().map(|w| w.pose(0.0)).collect();
        let school = Swarm::build(stage, &fish(Role::unmarked()).mob, &poses, None)?;
        Ok(Self { art, school, wanders })
    }
}

/// The dip net: an outline, a faint fill and a mesh, as one detached group
/// the size of one grid cell, centred at the origin.
fn dip_net(stage: &mut Stage<'_>, w: f64, h: f64) -> fmn::Result<Mob> {
    let c = Role::net();
    let mut parts = vec![
        Rectangle::new()
            .width(w)
            .height(h)
            .style(Style::default().fill(c, 0.1).stroke(c, 4.0, 1.0))
            .build()?,
    ];
    for i in 1..4 {
        let x = -w / 2.0 + w * f64::from(i) / 4.0;
        let y = -h / 2.0 + h * f64::from(i) / 4.0;
        parts.push(
            Line::new(v(x, -h / 2.0), v(x, h / 2.0))
                .style(line_style(c, 1.2))
                .build()?,
        );
        parts.push(
            Line::new(v(-w / 2.0, y), v(w / 2.0, y))
                .style(line_style(c, 1.2))
                .build()?,
        );
    }
    Ok(stage.arena_mut().add(v_group(parts)))
}

/// The 10 × 10 grid, every line cut at the cells (short pieces under the
/// moving fish).
fn dip_grid(stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    let mut lines = Vec::new();
    let f = |i: usize| i as f64 / GRID as f64;
    for i in 0..=GRID {
        for j in 0..GRID {
            for (a, b) in [
                (lake_p(f(i), f(j)), lake_p(f(i), f(j + 1))),
                (lake_p(f(j), f(i)), lake_p(f(j + 1), f(i))),
            ] {
                lines.push(Line::new(a, b).style(line_style(hex("#2C4560"), 1.5)).build()?);
            }
        }
    }
    Ok(stage.arena_mut().add(v_group(lines)))
}

/// The screen centre of the grid cell holding a lake point.
fn spot_centre(p: (f64, f64)) -> Vec3 {
    let (c, r) = cell_of(p.0, p.1);
    cell_rect(c, r).0
}

/// 01_5–01_8: count one patch and scale up; two other patches; the heat
/// map that explains why one net cannot know its share of the fish.
fn dips(kit: &Kit, stage: &mut Stage<'_>, lake: &Lake, scene: &DipLake, title: [Mob; 2]) -> fmn::Result<()> {
    let the_key = key(kit, stage)?;
    let grid = dip_grid(stage)?;
    let (_, cw, ch) = cell_rect(0, 0);
    let net = dip_net(stage, cw, ch)?;
    let [weeds, open, deep] = SPOTS.map(|(_, p)| p);

    // 01_5: one net on the deep end, counted and multiplied by a hundred
    let deep_fish = lake.count_at(deep);
    let deep_c = spot_centre(deep);
    stage.move_to(net, v(deep_c[0], deep_c[1] + 2.4), ORIGIN);
    kit.say(stage, "01_5")?;
    fade_out_all(stage, &title, 0.8)?;
    play!(stage;
        fade_in(stage.arena_mut(), grid, ORIGIN, 1.0)?.rt(0.8),
        fade_in(stage.arena_mut(), the_key, ORIGIN, 1.0)?.rt(0.8),
    );
    stage.add_to_scene(net)?;
    let drop = net
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.9),
            ..AnimateArgs::default()
        })?
        .move_to(deep_c, ORIGIN)?;
    stage.play(drop)?;
    let label = kit.text_left(stage, "fish in this net:", 30.0, Palette::dim(), v(-6.6, 3.45))?;
    stage.add_to_scene(label)?;
    let tracker = stage.add_value_tracker(0.0);
    let readout = kit.live_count(stage, tracker, 40.0, Role::net(), v(-2.7, 3.45))?;
    stage.play(tween(tracker, f64::from(deep_fish), 1.4)?)?;
    let sum = kit.text(
        stage,
        &format!("{} × 100 = {}", thousands(deep_fish), thousands(deep_fish * 100)),
        40.0,
        Role::dark(),
        v(2.4, 3.45),
    )?;
    play!(stage; write(stage.arena(), sum).rt(1.0));
    kit.hold(stage)?;

    // 01_6: the same net over the weeds and over open water
    kit.say(stage, "01_6")?;
    stage.clear_updaters(readout, true);
    play!(stage;
        fade_out(stage.arena_mut(), sum, ORIGIN, 1.0)?.rt(0.4),
        fade_out(stage.arena_mut(), readout, ORIGIN, 1.0)?.rt(0.4),
        fade_out(stage.arena_mut(), label, ORIGIN, 1.0)?.rt(0.4),
    );
    for spot in [weeds, open] {
        let hover = net
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.0),
                ..AnimateArgs::default()
            })?
            .move_to(spot_centre(spot), ORIGIN)?;
        stage.play(hover)?;
        stage.wait(0.4)?;
    }
    let q = predict_pause(kit, stage, v(6.3, 2.4))?;

    // 01_7: each spot's answer where it was netted, then all three on one
    // number line, 31 times apart
    kit.say(stage, "01_7")?;
    play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
    let spots = [("deep end", deep), ("weeds", weeds), ("open water", open)];
    let mut tags = Vec::new();
    let mut pins = Vec::new();
    for (i, &(_, p)) in spots.iter().enumerate() {
        let c = spot_centre(p);
        let e = lake.count_at(p) * 100;
        let tag = plated(
            kit,
            stage,
            &thousands(e),
            30.0,
            Role::dark(),
            v(c[0], c[1] + ch / 2.0 + 0.3),
        )?;
        if i > 0 {
            // "In the weeds, …", "In open water, …": each hop on its words
            kit.until_sentence(stage, "01_7", i)?;
            let hop = net
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.7),
                    ..AnimateArgs::default()
                })?
                .move_to(c, ORIGIN)?;
            stage.play(hop)?;
        }
        let pin = stage.arena_mut().add(
            Rectangle::new()
                .width(cw)
                .height(ch)
                .style(Style::default().fill(Role::dark(), 0.08).stroke(Role::dark(), 4.5, 1.0))
                .build()?
                .moved_to(c),
        );
        play!(stage;
            fade_in(stage.arena_mut(), pin, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), tag, v(0.0, -0.15), 1.0)?.rt(0.5),
        );
        tags.push(tag);
        pins.push(pin);
    }
    // the strip above the lake (its bank tops out at y = 2.95)
    let (x0, x1, y) = (-6.0, 4.0, 3.3);
    let top = 100_000.0;
    let scale = |e: u32| x0 + (x1 - x0) * f64::from(e) / top;
    let mut marks = vec![
        Line::new(v(x0, y), v(x1, y))
            .style(line_style(Palette::dim(), 2.5))
            .build()?,
    ];
    let estimates: Vec<(&str, u32)> = spots.iter().map(|&(name, p)| (name, lake.count_at(p) * 100)).collect();
    for &(_, e) in &estimates {
        marks.push(
            Line::new(v(scale(e), y - 0.13), v(scale(e), y + 0.13))
                .style(line_style(Role::dark(), 4.0))
                .build()?,
        );
    }
    let axis = stage.arena_mut().add(v_group(marks));
    let mut words = Vec::new();
    for &(name, e) in &estimates {
        words.push(kit.text(
            stage,
            &format!("{name}  {}", thousands(e)),
            24.0,
            Role::dark(),
            v(scale(e), y + 0.32),
        )?);
    }
    let lo = estimates.iter().map(|e| e.1).min().unwrap_or(1).max(1);
    let hi = estimates.iter().map(|e| e.1).max().unwrap_or(1);
    let apart = kit.text(
        stage,
        &format!("{}× apart", (f64::from(hi) / f64::from(lo)).floor()),
        30.0,
        Role::bias(),
        v(5.6, y),
    )?;
    play!(stage; fade_in(stage.arena_mut(), axis, ORIGIN, 1.0)?.rt(0.6));
    for w in words {
        play!(stage; fade_in(stage.arena_mut(), w, v(0.0, -0.15), 1.0)?.rt(0.4));
    }
    play!(stage; fade_in(stage.arena_mut(), apart, ORIGIN, 1.0)?.rt(0.6));
    kit.hold(stage)?;

    // 01_8: the heat map under the fish: a hundredth of the water is not a
    // hundredth of the fish
    // (it fades in by recolouring from the water's own colour: a fade_in
    // would re-root it above the fish)
    let max = f64::from(lake.all_counts().max().unwrap_or(1));
    let heat_at = |u: f64| -> Vec<Vec<Srgb>> {
        let (low, high) = (Role::water(), interpolate_color(Role::water(), hex("#2F7398"), u));
        (0..GRID)
            .rev()
            .map(|r| {
                (0..GRID)
                    .map(|c| interpolate_color(low, high, (f64::from(lake.counts[r][c]) / max).sqrt()))
                    .collect()
            })
            .collect()
    };
    let heat = RowStrips::build(
        stage,
        lake_p(0.0, 1.0),
        (LAKE_W / GRID as f64, LAKE_H / GRID as f64),
        &heat_at(0.0),
    )?;
    let steps: Vec<Vec<Vec<Srgb>>> = (0..=24).map(|i| heat_at(f64::from(i) / 24.0)).collect();
    let box_text = kit.text(
        stage,
        "how many fish in all?  (coming up)",
        30.0,
        Palette::ink(),
        v(0.0, -0.2),
    )?;
    let (bw, bh) = (stage.get_width(box_text) + 0.5, stage.get_height(box_text) + 0.35);
    let frame = stage.arena_mut().add(
        Rectangle::new()
            .width(bw)
            .height(bh)
            .corner_radius(0.1)
            .style(
                Style::default()
                    .fill(hex("#1A2130"), 0.94)
                    .stroke(Palette::faint(), 2.0, 1.0),
            )
            .build()?
            .moved_to(v(0.0, -0.2)),
    );
    kit.say(stage, "01_8")?;
    play!(stage;
        fade_out(stage.arena_mut(), net, ORIGIN, 1.0)?.rt(0.6),
        fade_out(stage.arena_mut(), grid, ORIGIN, 1.0)?.rt(0.6),
    );
    // the heat lies on the water, under the fish, pins and tags
    stage.add_to_scene(heat.group)?;
    let mut front = vec![scene.school.group];
    front.extend(&pins);
    front.extend(&tags);
    stage.scene_mut().bring_to_front(&front)?;
    let level = stage.add_value_tracker(0.0);
    stage.add_to_scene(level)?;
    let rows = heat.rows[0];
    stage.add_dt_updater(
        rows,
        move |s, _, _| {
            let u = s.tracker_value(level).unwrap_or(0.0).clamp(0.0, 1.0);
            heat.recolor(s, &steps[(u * 24.0).round() as usize]);
        },
        false,
    )?;
    stage.play(tween(level, 1.0, 1.6)?)?;
    play!(stage;
        fade_in(stage.arena_mut(), frame, ORIGIN, 1.0)?.rt(0.6),
        fade_in(stage.arena_mut(), box_text, ORIGIN, 1.0)?.rt(0.6),
    );
    let note = precisely(
        kit,
        stage,
        &format!(
            "averaging many random dips does work (all 100 squares here average exactly {}), but only {} of the 100 \
             single dips land within 2× of it. One net can't tell what share of the fish it reached.",
            thousands(LAKE_FISH as u64),
            lake.dips_within_2x()
        ),
    )?;
    play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.8));
    Ok(())
}
