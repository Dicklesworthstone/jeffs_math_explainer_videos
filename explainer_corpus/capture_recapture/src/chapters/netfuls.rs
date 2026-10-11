//! Chapter 04, every possible netful: the lake shrunk to 20 fish, 5 marked,
//! and a net of 4 (04_1); random nets (04_2); counting them, 20·19·18·17
//! over 24 orders (04_3); five piles by marked fish (04_4); all 4,845
//! netfuls poured into them (04_5: spike S-4's one-updater flight, every
//! glyph one real netful); the one-marked pile as the soup again (04_6);
//! the two-marked pile as shirts and pants, 10 × 105 (04_7); the piles as
//! chances (04_8); and the rule, named (04_9).

use std::cell::RefCell;
use std::rc::Rc;

use explainer_kit::props::{Pose, Swarm, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, choose, fade_all, hex, play, rect, thousands, v};
use fmn::prelude::*;

use super::draw::{Role, fish, flat, line_style, marked_fish, ring};
use super::lake::{LAKE_C, LakeArt, PER_GLYPH, Shade, fade_in_all, fade_out_all, fold, lake_p, plated, shuffled};
use crate::facts::NETFULS;
use crate::model::{LAKE_FISH, Lake, SEED, SplitMix64};

/// The toy lake: fish, marked, net (A-F1 … A-F4).
const LAKE: usize = NETFULS.0 as usize;
const MARKED: usize = NETFULS.1 as usize;
const NET: usize = NETFULS.2 as usize;

/// The toy pond: the big lake shrunk by this much about its centre, its
/// fish this many times their lake size, on a 5 × 4 grid.
const POND: f64 = 0.62;
const BIG: f64 = 2.2;
const POND_COLS: usize = 5;

/// One netful glyph's cell, the block they start in and the bins.
const CELL: f64 = 0.06;
const SRC_COLS: usize = 190;
const SRC_TOP: f64 = 3.35;
const BIN_COLS: usize = 40;
const BIN_GAP: f64 = 0.3;
const BIN_FLOOR: f64 = -2.15;
/// The pour: departures spread over this long, each flight this long.
const SPREAD: f64 = 3.6;
const FLIGHT: f64 = 0.8;

/// Every netful of the toy lake, in lexicographic order.
pub fn netfuls() -> Vec<[usize; NET]> {
    let mut out = Vec::with_capacity(choose(LAKE as u64, NET as u64) as usize);
    for a in 0..LAKE {
        for b in a + 1..LAKE {
            for c in b + 1..LAKE {
                for d in c + 1..LAKE {
                    out.push([a, b, c, d]);
                }
            }
        }
    }
    out
}

/// Which toy fish are marked: five of the twenty, seeded.
pub fn marked_fish_ids() -> Vec<usize> {
    let mut rng = SplitMix64::new(SEED ^ 0x0401);
    let mut m: Vec<usize> = shuffled(LAKE, &mut rng)[..MARKED].to_vec();
    m.sort_unstable();
    m
}

/// The netfuls in each pile: by how many marked fish they hold.
pub fn pile_counts(nets: &[[usize; NET]], marked: &[usize]) -> [usize; NET + 1] {
    let mut counts = [0usize; NET + 1];
    for net in nets {
        counts[net.iter().filter(|f| marked.contains(f)).count()] += 1;
    }
    counts
}

fn source_cell(i: usize) -> Vec3 {
    let (r, c) = (i / SRC_COLS, i % SRC_COLS);
    v(
        -(SRC_COLS as f64) * CELL / 2.0 + (c as f64 + 0.5) * CELL,
        SRC_TOP - (r as f64 + 0.5) * CELL,
    )
}

fn bin_width() -> f64 {
    BIN_COLS as f64 * CELL
}

fn bin_left(k: usize) -> f64 {
    let total = (NET + 1) as f64 * bin_width() + NET as f64 * BIN_GAP;
    -total / 2.0 + k as f64 * (bin_width() + BIN_GAP)
}

fn bin_centre(k: usize) -> f64 {
    bin_left(k) + bin_width() / 2.0
}

fn bin_slot(k: usize, j: usize) -> Vec3 {
    v(
        bin_left(k) + ((j % BIN_COLS) as f64 + 0.5) * CELL,
        BIN_FLOOR + ((j / BIN_COLS) as f64 + 0.5) * CELL,
    )
}

/// One netful: four dots in a 2 × 2 cell, its marked fish gold.
fn glyph(net: &[usize; NET], marked: &[usize], center: Vec3) -> VMobject {
    let (a, r) = (0.2 * CELL, 0.16 * CELL);
    let offsets = [(-a, a), (a, a), (-a, -a), (a, -a)];
    v_group(net.iter().zip(offsets).map(|(f, (dx, dy))| {
        let c = if marked.contains(f) {
            Role::marked()
        } else {
            Role::unmarked()
        };
        Dot::new()
            .point(v(center[0] + dx, center[1] + dy))
            .radius(r)
            .color(c)
            .build()
    }))
}

/// A pair of fish as two dots side by side (04_7's shirts and pants).
fn pair_glyph(color: Srgb, at: Vec3, r: f64) -> VMobject {
    v_group([-1.0, 1.0].map(|s| {
        Dot::new()
            .point(v(at[0] + s * 1.3 * r, at[1]))
            .radius(r)
            .color(color)
            .build()
    }))
}

pub struct Netfuls {
    pub kit: Kit,
}

impl SceneConstruct for Netfuls {
    fn name(&self) -> &str {
        "04_netfuls"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let marked = marked_fish_ids();
        let nets = netfuls();
        let counts = pile_counts(&nets, &marked);
        let total = nets.len();

        // 04_1: the big lake shrinks to a pond of twenty fish
        let lake = Lake::draw();
        let art = LakeArt::whole(stage)?;
        let mut rng = SplitMix64::new(SEED ^ 0x0402);
        let drawn: Vec<Pose> = (0..LAKE_FISH)
            .step_by(PER_GLYPH)
            .map(|i| {
                let p = lake_p(lake.x[i], lake.y[i]);
                Pose {
                    x: p[0],
                    y: p[1],
                    heading: std::f64::consts::TAU * rng.uniform(),
                }
            })
            .collect();
        let keep = shuffled(drawn.len(), &mut rng)[..LAKE].to_vec();
        let rest: Vec<Pose> = (0..drawn.len())
            .filter(|i| !keep.contains(i))
            .map(|i| drawn[i])
            .collect();
        let blue = fish(Role::unmarked());
        let crowd = Swarm::build(stage, &blue.mob, &rest, None)?;
        let twenty: Vec<Pose> = keep.iter().map(|&i| drawn[i]).collect();
        let pond_fish = Swarm::build(stage, &blue.mob, &twenty, None)?;
        kit.say(stage, "04_1")?;
        fade_in_all(stage, &[art, crowd.group, pond_fish.group], 0.8)?;
        // all but twenty leave, and the lake shrinks round them
        play!(stage; fade_out(stage.arena_mut(), crowd.group, ORIGIN, 1.0)?.rt(0.9));
        let pond_c = v(0.0, 0.1);
        let shrink = art
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.4),
                ..AnimateArgs::default()
            })?
            .scale_about(POND, Some(LAKE_C), None)?
            .shift([pond_c[0] - LAKE_C[0], pond_c[1] - LAKE_C[1], 0.0])?;
        let pond_w = super::lake::LAKE_W * POND;
        let pond_h = super::lake::LAKE_H * POND;
        let slots: Vec<Vec3> = (0..LAKE)
            .map(|i| {
                let (c, r) = ((i % POND_COLS) as f64, (i / POND_COLS) as f64);
                let rows = (LAKE / POND_COLS) as f64;
                v(
                    pond_c[0]
                        + (c - (POND_COLS as f64 - 1.0) / 2.0) * 0.86 * pond_w / POND_COLS as f64
                        + 0.12 * (rng.uniform() - 0.5),
                    pond_c[1] + ((rows - 1.0) / 2.0 - r) * 0.82 * pond_h / rows + 0.1 * (rng.uniform() - 0.5),
                )
            })
            .collect();
        let headings: Vec<f64> = twenty.iter().map(|p| p.heading).collect();
        let grow = grow_into(stage, &pond_fish.members, &twenty, &slots, &headings, BIG, 1.4)?;
        play!(stage; shrink, grow);
        // five of them are marked
        let gold_glyph = marked_fish();
        let gold_poses: Vec<Pose> = marked
            .iter()
            .map(|&i| Pose {
                x: slots[i][0],
                y: slots[i][1],
                heading: 0.0,
            })
            .collect();
        let gold = Swarm::build(
            stage,
            &gold_glyph.mob.clone().scaled_about(BIG, ORIGIN),
            &gold_poses,
            None,
        )?;
        let gold_shade = Shade::new(stage, &gold.members, &gold_glyph.paints, 0.0)?;
        stage.add_to_scene(gold.group)?;
        let lake_words = kit.text_left(
            stage,
            &format!("{LAKE} fish, {MARKED} of them marked"),
            30.0,
            Palette::ink(),
            v(-pond_w / 2.0, pond_c[1] + pond_h / 2.0 + 0.45),
        )?;
        let net_words = kit.text_left(
            stage,
            &format!("our net holds {NET}"),
            30.0,
            Role::second(),
            v(-pond_w / 2.0, pond_c[1] - pond_h / 2.0 - 0.45),
        )?;
        play!(stage; gold_shade.to(1.0, 0.8)?, fade_in(stage.arena_mut(), lake_words, ORIGIN, 1.0)?.rt(0.8));
        fade_in_all(stage, &[net_words], 0.6)?;
        kit.hold(stage)?;

        // 04_2: random nets: every group of four equally likely
        let ring_glyph = ring();
        let ring_tpl = ring_glyph.mob.clone().scaled_about(BIG, ORIGIN);
        let mut flash_rng = SplitMix64::new(SEED ^ 0x0403);
        let flashes: Vec<[usize; NET]> = (0..7)
            .map(|_| {
                let mut f: Vec<usize> = shuffled(LAKE, &mut flash_rng)[..NET].to_vec();
                f.sort_unstable();
                [f[0], f[1], f[2], f[3]]
            })
            .collect();
        let at = |i: usize| Pose {
            x: slots[i][0],
            y: slots[i][1],
            heading: 0.0,
        };
        let rings = Swarm::build(stage, &ring_tpl, &flashes[0].map(at), None)?;
        kit.say(stage, "04_2")?;
        fade_in_all(stage, &[rings.group], 0.3)?;
        let mut shown = Vec::new();
        for (j, f) in flashes.iter().enumerate() {
            if j > 0 {
                let mut hops: Vec<Box<dyn Animation>> = Vec::new();
                for (&m, &i) in rings.members.iter().zip(f) {
                    let a = m
                        .animate()
                        .set_anim_args(AnimateArgs {
                            run_time: Some(0.3),
                            ..AnimateArgs::default()
                        })?
                        .move_to(slots[i], ORIGIN)?;
                    hops.push(stage.prepare(a)?);
                }
                stage.play_prepared(hops)?;
            }
            // each netful also drops into a row under the pond, as a glyph
            let g = stage
                .arena_mut()
                .add(glyph(f, &marked, v(-1.65 + 0.55 * j as f64, -2.85)));
            stage.scale(g, 5.0);
            fade_in_all(stage, &[g], 0.25)?;
            shown.push(g);
        }
        kit.hold(stage)?;

        // 04_3: counting them: 20 · 19 · 18 · 17 ordered picks, each netful
        // picked in 24 orders
        let picks = flashes[flashes.len() - 1];
        let mut away = vec![net_words];
        away.extend(&shown);
        fade_out_all(stage, &away, 0.4)?;
        // the pond moves left
        let left = v(-4.35, 0.2);
        let move_left: Vec<Mob> = vec![art, pond_fish.group, gold.group, rings.group, lake_words];
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for m in move_left {
            let a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.0),
                    ..AnimateArgs::default()
                })?
                .scale_about(0.62, Some(pond_c), None)?
                .shift([left[0] - pond_c[0], left[1] - pond_c[1], 0.0])?;
            moves.push(stage.prepare(a)?);
        }
        kit.say(stage, "04_3")?;
        stage.play_prepared(moves)?;
        let small = |p: Vec3| v(left[0] + 0.62 * (p[0] - pond_c[0]), left[1] + 0.62 * (p[1] - pond_c[1]));
        // four slots, first to fourth; each pick's choices under it
        let slot_x = |j: usize| 0.4 + 1.55 * j as f64;
        let slot_y = 2.45;
        let mut boxes = Vec::new();
        for j in 0..NET {
            boxes.push(
                stage.arena_mut().add(
                    Rectangle::new()
                        .width(1.2)
                        .height(1.0)
                        .corner_radius(0.1)
                        .style(
                            Style::default()
                                .fill(hex("#14202E"), 1.0)
                                .stroke(Role::second(), 2.0, 0.7),
                        )
                        .build()?
                        .moved_to(v(slot_x(j), slot_y)),
                ),
            );
        }
        fade_in_all(stage, &boxes, 0.4)?;
        // the picked fish fly from the pond to the slots, the number of
        // choices left under each
        let mut picked_glyphs = Vec::new();
        for (j, &i) in picks.iter().enumerate() {
            let tpl = if marked.contains(&i) {
                gold_glyph.mob.clone()
            } else {
                blue.mob.clone()
            };
            let g = stage.arena_mut().add(tpl.scaled_about(BIG * 0.62, ORIGIN));
            stage.move_to(g, small(slots[i]), ORIGIN);
            stage.add_to_scene(g)?;
            let choices = kit.text(
                stage,
                &(LAKE - j).to_string(),
                40.0,
                Palette::ink(),
                v(slot_x(j), slot_y - 0.85),
            )?;
            let fly = g
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(0.6),
                    ..AnimateArgs::default()
                })?
                .move_to(v(slot_x(j), slot_y), ORIGIN)?;
            play!(stage; fly, fade_in(stage.arena_mut(), choices, ORIGIN, 1.0)?.rt(0.6));
            picked_glyphs.push(g);
        }
        let falling: u64 = (0..NET as u64).map(|j| NETFULS.0 - j).product();
        let product = kit.tex(
            stage,
            &format!(
                r"{} = {}",
                (0..NET as u64)
                    .map(|j| (NETFULS.0 - j).to_string())
                    .collect::<Vec<_>>()
                    .join(r" \cdot "),
                thousands(falling).replace(',', "{,}")
            ),
            44.0,
            Palette::ink(),
            &[],
            v(2.75, 0.85),
        )?;
        play!(stage; write(stage.arena(), product).rt(1.0));
        // the same four fish, in order after order
        let orders: u64 = (1..=NET as u64).product();
        let mut perm_rng = SplitMix64::new(SEED ^ 0x0404);
        for _ in 0..6 {
            let order = shuffled(NET, &mut perm_rng);
            let mut swaps: Vec<Box<dyn Animation>> = Vec::new();
            for (j, &g) in picked_glyphs.iter().enumerate() {
                let a = g
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.32),
                        path_arc: Some(0.9),
                        ..AnimateArgs::default()
                    })?
                    .move_to(v(slot_x(order[j]), slot_y), ORIGIN)?;
                swaps.push(stage.prepare(a)?);
            }
            stage.play_prepared(swaps)?;
        }
        let orders_tex = kit.tex(
            stage,
            &format!(
                r"\text{{orders: }} {} = {}",
                (1..=NET as u64)
                    .rev()
                    .map(|j| j.to_string())
                    .collect::<Vec<_>>()
                    .join(r" \cdot "),
                orders
            ),
            40.0,
            Role::second(),
            &[],
            v(2.75, -0.35),
        )?;
        play!(stage; write(stage.arena(), orders_tex).rt(0.9));
        let netful_count = falling / orders;
        let divide = kit.tex(
            stage,
            &format!(
                r"\frac{{{}}}{{{}}} = {} = \binom{{{}}}{{{}}}",
                thousands(falling).replace(',', "{,}"),
                orders,
                thousands(netful_count).replace(',', "{,}"),
                NETFULS.0,
                NETFULS.2
            ),
            48.0,
            Palette::ink(),
            &[],
            v(2.75, -1.75),
        )?;
        play!(stage; write(stage.arena(), divide).rt(1.2));
        let choose_words = kit.text(
            stage,
            &format!("\u{201c}{} choose {}\u{201d}", NETFULS.0, NETFULS.2),
            28.0,
            Palette::dim(),
            v(2.75, -2.7),
        )?;
        fade_in_all(stage, &[choose_words], 0.5)?;
        kit.hold(stage)?;

        // 04_4: every netful, and five empty piles by marked fish
        fade_all(stage, 0.6)?;
        let mut glyphs = Vec::with_capacity(total);
        for (i, net) in nets.iter().enumerate() {
            glyphs.push(stage.arena_mut().add(glyph(net, &marked, source_cell(i))));
        }
        let title = kit.text(
            stage,
            &format!("all {} netfuls", thousands(total as u64)),
            30.0,
            Palette::ink(),
            v(0.0, 3.62),
        )?;
        let rows_max = counts.iter().max().copied().unwrap_or(0).div_ceil(BIN_COLS);
        let wall = rows_max as f64 * CELL + 0.12;
        let mut bin_parts = Vec::new();
        for k in 0..=NET {
            let (l, r, pad) = (bin_left(k), bin_left(k) + bin_width(), 0.04);
            let corners = [
                v(l - pad, BIN_FLOOR + wall),
                v(l - pad, BIN_FLOOR - pad),
                v(r + pad, BIN_FLOOR - pad),
                v(r + pad, BIN_FLOOR + wall),
            ];
            bin_parts.push(polyline(corners).style(line_style(Palette::faint(), 2.0)).build());
        }
        let bins = stage.arena_mut().add(v_group(bin_parts));
        // under each bin, one line: its name, then its count
        let label_y = BIN_FLOOR - 0.3;
        let count_at = |k: usize| v(bin_centre(k) + 0.6, label_y);
        let mut bin_labels = Vec::new();
        for k in 0..=NET {
            bin_labels.push(kit.text(
                stage,
                &format!("{k} marked"),
                24.0,
                Palette::dim(),
                v(bin_centre(k) - 0.45, label_y),
            )?);
        }
        kit.say(stage, "04_4")?;
        stage.add_many_to_scene(&glyphs)?;
        let mut intro = vec![title, bins];
        intro.extend(&bin_labels);
        fade_in_all(stage, &intro, 0.6)?;
        let q = predict_pause(&kit, stage, v(6.5, 2.75))?;

        // 04_5: the pour: one updater, placement only; each pile fills from
        // the bottom in the order its netfuls leave
        fade_out_all(stage, &[q], 0.3)?;
        let mut trackers = Vec::with_capacity(NET + 1);
        let mut lives = Vec::with_capacity(NET + 1);
        for k in 0..=NET {
            let t = stage.add_value_tracker(0.0);
            lives.push(kit.live_count(stage, t, 26.0, Palette::ink(), count_at(k))?);
            trackers.push(t);
        }
        let mut order: Vec<usize> = (0..total).collect();
        let mut pour_rng = SplitMix64::new(SEED ^ 0x0405);
        for i in (1..total).rev() {
            order.swap(i, (pour_rng.next_u64() % (i as u64 + 1)) as usize);
        }
        let mut filled = [0usize; NET + 1];
        let mut flights: Vec<(Mob, Vec3, Vec3, Vec3, f64, usize, Vec3)> = order
            .iter()
            .enumerate()
            .map(|(j, &i)| {
                let bin = nets[i].iter().filter(|f| marked.contains(f)).count();
                let (from, to) = (source_cell(i), bin_slot(bin, filled[bin]));
                filled[bin] += 1;
                let mid = v((from[0] + to[0]) / 2.0, from[1] + 0.5);
                (glyphs[i], from, mid, to, j as f64 / total as f64 * SPREAD, bin, from)
            })
            .collect();
        let arrived = Rc::new(RefCell::new([0usize; NET + 1]));
        let ctl = stage.add(VMobject::new())?;
        {
            let arrived = Rc::clone(&arrived);
            let trackers = trackers.clone();
            let (mut clock, mut next, mut active) = (0.0f64, 0usize, Vec::<usize>::new());
            stage.add_dt_updater(
                ctl,
                move |s, _, dt| {
                    clock += dt;
                    while next < flights.len() && flights[next].4 <= clock {
                        active.push(next);
                        next += 1;
                    }
                    let mut still = Vec::with_capacity(active.len());
                    for &i in &active {
                        let f = &mut flights[i];
                        let tau = ((clock - f.4) / FLIGHT).min(1.0);
                        let u = tau * tau * (3.0 - 2.0 * tau);
                        let (a, b, c) = ((1.0 - u) * (1.0 - u), 2.0 * u * (1.0 - u), u * u);
                        let p = v(
                            a * f.1[0] + b * f.2[0] + c * f.3[0],
                            a * f.1[1] + b * f.2[1] + c * f.3[1],
                        );
                        s.shift(f.0, [p[0] - f.6[0], p[1] - f.6[1], 0.0]);
                        f.6 = p;
                        if tau >= 1.0 {
                            let mut got = arrived.borrow_mut();
                            got[f.5] += 1;
                            let _ = s.set_tracker_value(trackers[f.5], got[f.5] as f64);
                        } else {
                            still.push(i);
                        }
                    }
                    active = still;
                },
                false,
            )?;
        }
        kit.mark(stage, "pour");
        kit.listen(stage, SPREAD + FLIGHT - 0.4, "04_5")?;
        stage.wait(0.5)?;
        stage.clear_updaters(ctl, true);
        if *arrived.borrow() != counts {
            return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(
                "04_5: the pour's arrivals differ from the pile counts",
            )));
        }
        // the counts have stopped: static text from here on
        let mut count_tags = Vec::with_capacity(NET + 1);
        for k in 0..=NET {
            stage.clear_updaters(lives[k], true);
            stage.remove_from_scene(lives[k]);
            let t = kit.text(stage, &thousands(counts[k] as u64), 26.0, Palette::ink(), count_at(k))?;
            stage.add_to_scene(t)?;
            count_tags.push(t);
        }

        // 04_6: the one-marked pile: one in four, as in the lake
        let one = 1usize;
        let block_h = |k: usize| counts[k] as f64 / BIN_COLS as f64 * CELL;
        let glow = stage.arena_mut().add(rect(
            v(bin_left(one) - 0.08, BIN_FLOOR - 0.08),
            bin_width() + 0.16,
            block_h(one) + 0.24,
            Style::default()
                .fill(Role::marked(), 0.08)
                .stroke(Role::marked(), 4.0, 1.0),
        )?);
        let soup = plated(
            &kit,
            stage,
            &format!("1 of {NET} marked, like {MARKED} of {LAKE} in the lake"),
            28.0,
            Role::marked(),
            v(bin_centre(one) + 2.2, BIN_FLOOR + block_h(one) + 0.5),
        )?;
        kit.say(stage, "04_6")?;
        fade_in_all(stage, &[glow, soup], 0.6)?;
        let vandermonde = precisely(
            &kit,
            stage,
            &format!(
                "{} = {} (Vandermonde: the sum over k of C({MARKED},k)·C({},{NET}−k) is C({LAKE},{NET}))",
                counts
                    .iter()
                    .map(|c| thousands(*c as u64))
                    .collect::<Vec<_>>()
                    .join(" + "),
                thousands(total as u64),
                LAKE - MARKED
            ),
        )?;
        fade_in_all(stage, &[vandermonde], 0.6)?;
        kit.hold(stage)?;
        // the piles settle into solid bars of the same areas
        let mut blocks = Vec::new();
        for k in 0..=NET {
            blocks.push(stage.arena_mut().add(rect(
                v(bin_left(k), BIN_FLOOR),
                bin_width(),
                block_h(k).max(0.01),
                flat(Role::chance(), 0.9),
            )?));
        }
        fade_in_all(stage, &blocks, 0.6)?;
        let mut gone: Vec<Mob> = glyphs.clone();
        gone.extend([glow, soup, vandermonde, title, bins]);
        fade_out_all(stage, &gone, 0.4)?;
        let mut chart: Vec<Mob> = blocks.clone();
        chart.extend(&bin_labels);
        chart.extend(&count_tags);

        // 04_7: the two-marked pile is 10 marked pairs times 105 unmarked
        // pairs, like shirts and pants; the chart shrinks out of the way
        let two = 2usize;
        let pivot = v(0.0, -2.6);
        let shrunk = 0.5;
        let others: Vec<Mob> = (0..=NET).filter(|&k| k != two).map(|k| blocks[k]).collect();
        kit.say(stage, "04_7")?;
        let anims = rescale(stage, &chart, shrunk, pivot, (&others, 0.25), 0.8)?;
        stage.play_prepared(anims)?;
        let (mp, up) = (choose(MARKED as u64, 2), choose((LAKE - MARKED) as u64, 2));
        let unmarked = LAKE - MARKED;
        let (grid_l, grid_r, grid_top, row_h) = (-5.2, 6.2, 2.75, 0.17);
        let col_w = (grid_r - grid_l) / up as f64;
        let mut pair_parts = Vec::new();
        for r in 0..mp as usize {
            pair_parts.push(pair_glyph(
                Role::marked(),
                v(grid_l - 0.35, grid_top - (r as f64 + 0.5) * row_h),
                0.045,
            ));
        }
        let marked_pairs = stage.arena_mut().add(v_group(pair_parts));
        let mp_label = kit.text(
            stage,
            &format!("{mp} ways to pick 2 of the {MARKED} marked"),
            26.0,
            Role::marked(),
            v(-3.1, 3.4),
        )?;
        let up_tex = kit.tex(
            stage,
            &format!(
                r"{LAKE} - {MARKED} = {unmarked} \text{{ unmarked: }} \frac{{{unmarked} \cdot {}}}{{2}} = {up}",
                unmarked - 1
            ),
            32.0,
            Role::unmarked(),
            &[],
            v(2.9, 3.35),
        )?;
        // the 10 × 105 grid: every marked pair with every unmarked pair
        let mut cells = Vec::new();
        for r in 0..mp as usize {
            for c in 0..up as usize {
                let shade = if (r + c) % 2 == 0 {
                    hex("#3B4A5E")
                } else {
                    hex("#2F3B4C")
                };
                cells.push(rect(
                    v(grid_l + c as f64 * col_w, grid_top - (r + 1) as f64 * row_h),
                    col_w * 0.9,
                    row_h * 0.88,
                    flat(shade, 1.0),
                )?);
            }
        }
        let grid = stage.arena_mut().add(v_group(cells));
        fade_in_all(stage, &[marked_pairs, mp_label], 0.6)?;
        play!(stage; write(stage.arena(), up_tex).rt(1.0));
        play!(stage; fmn::animation::grow_from_edge(stage.arena_mut(), grid, LEFT, None)?.rt(1.4));
        let product = mp * up;
        let shirts = kit.tex(
            stage,
            &format!(
                r"\binom{{{MARKED}}}{{2}}\binom{{{unmarked}}}{{2}} = {mp} \cdot {up} = {}",
                thousands(product).replace(',', "{,}")
            ),
            44.0,
            Palette::ink(),
            &[],
            v(0.6, 0.3),
        )?;
        play!(stage; write(stage.arena(), shirts).rt(1.2));
        // and that is the two-marked pile's count
        let small = |p: Vec3| {
            v(
                pivot[0] + shrunk * (p[0] - pivot[0]),
                pivot[1] + shrunk * (p[1] - pivot[1]),
            )
        };
        let pile_top = small(v(bin_centre(two), BIN_FLOOR + block_h(two)));
        let pile_ring = stage.arena_mut().add(rect(
            v(
                small(v(bin_left(two), 0.0))[0] - 0.06,
                small(v(0.0, BIN_FLOOR))[1] - 0.06,
            ),
            bin_width() * shrunk + 0.12,
            pile_top[1] - small(v(0.0, BIN_FLOOR))[1] + 0.12,
            Style::default()
                .fill(Palette::ink(), 0.0)
                .stroke(Palette::ink(), 3.0, 1.0),
        )?);
        fade_in_all(stage, &[pile_ring], 0.5)?;
        kit.hold(stage)?;

        // 04_8: every pile over all the netfuls: the chances
        fade_out_all(stage, &[marked_pairs, mp_label, up_tex, grid, shirts, pile_ring], 0.5)?;
        kit.say(stage, "04_8")?;
        let anims = rescale(stage, &chart, 1.0 / shrunk, pivot, (&others, 0.9), 0.8)?;
        stage.play_prepared(anims)?;
        let mut pcts = Vec::new();
        for (k, &pile) in counts.iter().enumerate() {
            let pct = 100.0 * pile as f64 / total as f64;
            let s = if pct < 1.0 {
                "<1%".to_owned()
            } else {
                format!("{}%", pct.round())
            };
            pcts.push(kit.text(
                stage,
                &s,
                34.0,
                Palette::ink(),
                v(bin_centre(k), BIN_FLOOR + block_h(k).max(0.01) + 0.35),
            )?);
        }
        let over = kit.text(
            stage,
            &format!("each pile ÷ {} = its chance", thousands(total as u64)),
            30.0,
            Role::chance(),
            v(0.0, 2.6),
        )?;
        fade_in_all(stage, &[over], 0.5)?;
        for &p in &pcts {
            fade_in_all(stage, &[p], 0.35)?;
        }
        let exact = precisely(
            &kit,
            stage,
            &format!(
                "exact: {}",
                (0..=NET)
                    .map(|k| {
                        let g = gcd(counts[k] as u64, total as u64);
                        format!("{}/{}", counts[k] as u64 / g, total as u64 / g)
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )?;
        fade_in_all(stage, &[exact], 0.6)?;
        kit.hold(stage)?;

        // 04_9: the rule in words, its name, and the question it can't answer
        chart.extend(&pcts);
        fade_out_all(stage, &[over, exact], 0.4)?;
        kit.say(stage, "04_9")?;
        let anims = rescale(stage, &chart, 0.55, v(0.0, BIN_FLOOR), (&[], 1.0), 0.8)?;
        stage.play_prepared(anims)?;
        let to_corner: Vec<Box<dyn Animation>> = chart
            .iter()
            .map(|&m| {
                let a = m
                    .animate()
                    .set_anim_args(AnimateArgs {
                        run_time: Some(0.6),
                        ..AnimateArgs::default()
                    })?
                    .shift([-3.2, -0.35, 0.0])?;
                stage.prepare(a)
            })
            .collect::<fmn::Result<_>>()?;
        stage.play_prepared(to_corner)?;
        let name = kit.text(
            stage,
            "the hypergeometric distribution",
            36.0,
            Role::chance(),
            v(0.0, 3.2),
        )?;
        let rule = kit.text(
            stage,
            "chance  =  (ways to pick the marked ones) × (ways to pick the unmarked ones)  ÷  (all netfuls)",
            24.0,
            Palette::ink(),
            v(0.0, 2.3),
        )?;
        let letters = kit.tex(
            stage,
            r"\binom{M}{k} \cdot \binom{N-M}{n-k} \div \binom{N}{n}",
            34.0,
            Palette::dim(),
            &[
                ("M", Role::marked()),
                ("k", Role::marked()),
                ("n", Role::second()),
                ("N", Role::dark()),
            ],
            v(0.0, 1.45),
        )?;
        fade_in_all(stage, &[name], 0.6)?;
        fade_in_all(stage, &[rule], 0.6)?;
        play!(stage; write(stage.arena(), letters).rt(1.0));
        // lake → catch is known; catch → lake is the question
        let lake_box = plated(&kit, stage, "the lake", 30.0, Role::dark(), v(2.2, -0.6))?;
        let catch_box = plated(&kit, stage, "the catch", 30.0, Role::second(), v(5.4, -0.6))?;
        let forward = stage.arena_mut().add(
            Arrow::new(v(3.05, -0.45), v(4.55, -0.45))
                .style(line_style(Palette::ink(), 3.0))
                .build()?,
        );
        let back = stage.arena_mut().add(
            Arrow::new(v(4.55, -0.85), v(3.05, -0.85))
                .style(line_style(Role::bias(), 3.0))
                .build()?,
        );
        let ask = kit.text(stage, "?", 44.0, Role::bias(), v(3.8, -1.35))?;
        fade_in_all(stage, &[lake_box, catch_box, forward], 0.6)?;
        let formula = precisely(&kit, stage, "P_N(K = k) = C(M,k)·C(N−M, n−k) / C(N,n)")?;
        fade_in_all(stage, &[formula], 0.6)?;
        fade_in_all(stage, &[back, ask], 0.6)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// Scale every one of `mobs` about `pivot` by `f`, and set the opacity of
/// those listed in `fade` (one prepared animation per mobject: two
/// animations of one mobject in one play would fight, the later one's
/// target taken before the earlier one moved it).
fn rescale(
    stage: &mut Stage<'_>,
    mobs: &[Mob],
    f: f64,
    pivot: Vec3,
    fade: (&[Mob], f64),
    secs: f64,
) -> fmn::Result<Vec<Box<dyn Animation>>> {
    mobs.iter()
        .map(|&m| {
            let mut a = m
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(secs),
                    ..AnimateArgs::default()
                })?
                .scale_about(f, Some(pivot), None)?;
            if fade.0.contains(&m) {
                a = a.set_opacity(fade.1)?;
            }
            stage.prepare(a)
        })
        .collect()
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// Fly each member from its pose to a slot, growing by `grow` and turning
/// to face east, by placement only (one updater; the animation it returns
/// is a tracker tween that drives it, to play with others).
fn grow_into(
    stage: &mut Stage<'_>,
    members: &[Mob],
    from: &[Pose],
    to: &[Vec3],
    headings: &[f64],
    grow: f64,
    secs: f64,
) -> fmn::Result<AnimBuilder> {
    let flights: Vec<(Mob, Vec3, Vec3, f64)> = members
        .iter()
        .zip(from)
        .zip(to)
        .zip(headings)
        .map(|(((&m, p), &t), &h)| (m, v(p.x, p.y), t, -fold(h)))
        .collect();
    let level = stage.add_value_tracker(0.0);
    stage.add_to_scene(level)?;
    let mut last: Vec<(Vec3, f64, f64)> = flights.iter().map(|f| (f.1, 1.0, 0.0)).collect();
    stage.add_dt_updater(
        level,
        move |s, me, _| {
            let u = s.tracker_value(me).unwrap_or(0.0).clamp(0.0, 1.0);
            for (f, prev) in flights.iter().zip(last.iter_mut()) {
                let p = v(f.1[0] + (f.2[0] - f.1[0]) * u, f.1[1] + (f.2[1] - f.1[1]) * u);
                let (size, turned) = (1.0 + (grow - 1.0) * u, f.3 * u);
                if p != prev.0 {
                    s.shift(f.0, [p[0] - prev.0[0], p[1] - prev.0[1], 0.0]);
                    s.rotate(f.0, turned - prev.2, OUT, Some(p), None);
                    s.scale_about(f.0, size / prev.1, Some(p), None);
                    *prev = (p, size, turned);
                }
            }
        },
        false,
    )?;
    explainer_kit::tween(level, 1.0, secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_piles_are_the_ledgers_counts() {
        let nets = netfuls();
        assert_eq!(nets.len() as u128, choose(NETFULS.0, NETFULS.2), "A-F1.C20_4");
        let marked = marked_fish_ids();
        assert_eq!(marked.len(), MARKED);
        // A-F2: 1,365 / 2,275 / 1,050 / 150 / 5, whichever five are marked
        let want: Vec<usize> = (0..=NET as u64)
            .map(|k| (choose(NETFULS.1, k) * choose(NETFULS.0 - NETFULS.1, NETFULS.2 - k)) as usize)
            .collect();
        assert_eq!(pile_counts(&nets, &marked).to_vec(), want);
        assert_eq!(want, [1_365, 2_275, 1_050, 150, 5]);
    }

    #[test]
    fn the_bins_hold_their_piles_and_fit_the_frame() {
        let counts = [1_365usize, 2_275, 1_050, 150, 5];
        // the tallest pile stays under the block of netfuls it pours from
        let top = BIN_FLOOR + counts[1].div_ceil(BIN_COLS) as f64 * CELL;
        let block_bottom = SRC_TOP - choose(NETFULS.0, NETFULS.2).div_ceil(SRC_COLS as u128) as f64 * CELL;
        assert!(top < block_bottom - 0.2, "{top} vs {block_bottom}");
        assert!(bin_left(0) > -6.96 && bin_left(NET) + bin_width() < 6.96);
    }
}
