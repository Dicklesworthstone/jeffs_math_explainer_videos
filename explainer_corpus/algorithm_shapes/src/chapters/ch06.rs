//! Last-road partition, finite route languages, and the cost of a large badge.
use super::draw::{Map, ROUTES, dot, hide, prediction, route_color, route_total, show};
use crate::model::{self, Arithmetic, Error, PLACES, Quickest, ROADS, Road};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use std::collections::BTreeSet;

pub struct Chapter06 {
    pub kit: Kit,
}

const LETTERS: [char; 5] = ['a', 'b', 'c', 'd', 'e'];
struct Words<const FRONT: bool>;
impl<const FRONT: bool> Arithmetic for Words<FRONT> {
    type Value = BTreeSet<String>;
    fn zero() -> Self::Value {
        BTreeSet::new()
    }
    fn one() -> Self::Value {
        BTreeSet::from([String::new()])
    }
    fn road(road: Road) -> Self::Value {
        if !road.open {
            return Self::zero();
        }
        let index = ROADS
            .iter()
            .position(|r| r.from == road.from && r.to == road.to)
            .expect("this film's five distinct roads");
        BTreeSet::from([LETTERS[index].to_string()])
    }
    fn combine(a: &Self::Value, b: &Self::Value) -> Result<Self::Value, Error> {
        Ok(a.union(b).cloned().collect())
    }
    fn extend(a: &Self::Value, b: &Self::Value) -> Result<Self::Value, Error> {
        Ok(a.iter()
            .flat_map(|x| {
                b.iter()
                    .map(move |y| if FRONT { format!("{y}{x}") } else { format!("{x}{y}") })
            })
            .collect())
    }
}
fn spell(route: &[usize]) -> String {
    route.iter().map(|&i| LETTERS[i]).collect()
}
fn badge(words: &BTreeSet<String>) -> String {
    format!(
        "{{ {} }}",
        words
            .iter()
            .map(|w| if w.is_empty() { "ε" } else { w })
            .collect::<Vec<_>>()
            .join(", ")
    )
}
fn text(kit: &Kit, stage: &mut Stage<'_>, s: &str, y: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, s, 28.0, color, v(2.7, y))
}
fn strip(kit: &Kit, stage: &mut Stage<'_>, route: &[usize], y: f64, color: Srgb) -> fmn::Result<Vec<Mob>> {
    let mut mobs = Vec::new();
    for (j, &edge) in route.iter().enumerate() {
        let x = 0.6 + j as f64 * 1.55;
        let block = stage.arena_mut().add(
            Rectangle::new()
                .width(1.35)
                .height(0.52)
                .style(Style::default().stroke(color, 1.5, 1.0))
                .build()?
                .moved_to(v(x, y)),
        );
        let label = kit.text(stage, &ROADS[edge].value.to_string(), 25.0, color, v(x, y))?;
        show(stage, &[block, label], 0.18)?;
        mobs.extend([block, label]);
    }
    let result = kit.text(stage, &format!("= {}", route_total(route)), 27.0, color, v(5.5, y))?;
    show(stage, &[result], 0.2)?;
    mobs.push(result);
    Ok(mobs)
}
impl SceneConstruct for Chapter06 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let quick = model::sweep::<Quickest>(4, &ROADS, 0).expect("tested map");
        let words = model::sweep::<Words<false>>(4, &ROADS, 0).expect("tested map");
        let reversed = model::sweep::<Words<true>>(4, &ROADS, 0).expect("tested map");
        let map = Map::build(kit, stage, [0.0, 0.0], 1.0, true)?;
        kit.say(stage, "06_1")?;
        show(stage, &map.all(), 0.5)?;
        let mut panels = Vec::new();
        for (index, y) in [(1, 1.65), (2, 0.65), (0, -1.25)] {
            map.travel(stage, ROUTES[index], route_color(index))?;
            panels.extend(strip(kit, stage, ROUTES[index], y, route_color(index))?);
        }
        kit.hold(stage)?;
        kit.say(stage, "06_2")?;
        for (s, y) in [
            ("Last road from Lake", 2.4),
            ("Last road from Hill", -0.45),
            ("No overlap · none missing", -2.45),
        ] {
            let label = text(kit, stage, s, y, Palette::ink())?;
            show(stage, &[label], 0.3)?;
            panels.push(label);
        }
        kit.hold(stage)?;
        hide(stage, &panels, 0.4)?;
        kit.say(stage, "06_3")?;
        let lake = quick.badges[2].expect("reachable");
        let last = ROADS[4].value;
        let individual = ROUTES[1..].iter().map(|r| route_total(r)).collect::<Vec<_>>();
        let a = text(
            kit,
            stage,
            &format!(
                "Extend both, then choose\nmin({}, {}) = {}",
                individual[0],
                individual[1],
                lake + last
            ),
            0.9,
            Palette::teal(),
        )?;
        let b = text(
            kit,
            stage,
            &format!(
                "Choose, then extend\nmin({}, {}) + {last} = {}",
                ROADS[1].value,
                lake,
                lake + last
            ),
            -0.9,
            Palette::gold(),
        )?;
        // Lake's summary is spoken first; the per-route comparison lands with sentence 2.
        show(stage, &[b], 0.4)?;
        kit.until_sentence(stage, "06_3", 2)?;
        show(stage, &[a], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &[a, b], 0.3)?;
        kit.say(stage, "06_4")?;
        let offers = text(
            kit,
            stage,
            &format!(
                "Finish: min({}, {}) = {}",
                route_total(ROUTES[0]),
                lake + last,
                quick.badges[3].unwrap()
            ),
            0.6,
            Palette::teal(),
        )?;
        show(stage, &[offers], 0.3)?;
        for i in [3, 2, 1, 0] {
            let halo = stage.arena_mut().add(
                Circle::new()
                    .radius(0.22)
                    .style(Style::default().stroke(Palette::gold(), 2.0, 1.0))
                    .build()
                    .moved_to(map.points[i]),
            );
            let name = text(
                kit,
                stage,
                &format!("Explain {} from earlier badges", PLACES[i]),
                -0.7,
                Palette::ink(),
            )?;
            show(stage, &[halo, name], 0.3)?;
            stage.wait(0.55)?;
            hide(stage, &[halo, name], 0.25)?;
        }
        kit.hold(stage)?;
        hide(stage, &[offers], 0.3)?;
        kit.say(stage, "06_5")?;
        let token = dot(stage, map.points[0], 0.14, Palette::gold());
        show(stage, &[token], 0.3)?;
        let start = text(
            kit,
            stage,
            &format!("No roads yet\n0 + {} = {}\n1 × 1 = 1", ROADS[0].value, ROADS[0].value),
            0.0,
            Palette::gold(),
        )?;
        show(stage, &[start], 0.3)?;
        map.travel(stage, &[0], Palette::gold())?;
        kit.hold(stage)?;
        hide(stage, &[start, token], 0.3)?;
        kit.say(stage, "06_6")?;
        let empty = text(
            kit,
            stage,
            &format!(
                "No route + {} → no route\nmin(no route, {}) = {}",
                ROADS[0].value, ROADS[1].value, ROADS[1].value
            ),
            0.0,
            Palette::ink(),
        )?;
        show(stage, &[empty], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &[empty], 0.3)?;

        kit.say(stage, "06_7")?;
        hide(stage, &map.costs, 0.3)?;
        let mut letters = Vec::new();
        let offsets = [[-0.2, 0.3], [-0.2, -0.3], [0.3, 0.0], [0.2, 0.3], [0.2, -0.3]];
        for (i, road) in ROADS.iter().enumerate() {
            let a = map.points[road.from];
            let b = map.points[road.to];
            let l = kit.text(
                stage,
                &LETTERS[i].to_string(),
                31.0,
                Palette::gold(),
                v((a[0] + b[0]) * 0.5 + offsets[i][0], (a[1] + b[1]) * 0.5 + offsets[i][1]),
            )?;
            show(stage, &[l], 0.2)?;
            letters.push(l);
        }
        let mut rows = Vec::new();
        let start = text(
            kit,
            stage,
            &format!("Start: {}  no roads yet", badge(&Words::<false>::one())),
            2.3,
            Palette::ink(),
        )?;
        show(stage, &[start], 0.3)?;
        rows.push(start);
        kit.hold(stage)?;
        kit.say(stage, "06_8")?;
        for i in 1..4 {
            // Each row lands while its place is named; its incoming roads trace afterwards.
            // Same total duration as tracing first, so later cues keep their times.
            let row = text(
                kit,
                stage,
                &format!("{}: {}", PLACES[i], badge(&words.badges[i])),
                2.3 - i as f64 * 0.8,
                Palette::teal(),
            )?;
            show(stage, &[row], 0.3)?;
            rows.push(row);
            for offer in words.offers.iter().filter(|o| ROADS[o.road_index].to == i) {
                map.travel(stage, &[offer.road_index], Palette::teal())?;
            }
        }
        kit.hold(stage)?;
        kit.say(stage, "06_9")?;
        let prefix = kit.text(stage, "a", 38.0, Palette::teal(), v(2.1, -1.8))?;
        let suffix = kit.text(stage, "c", 38.0, Palette::gold(), v(4.2, -1.8))?;
        show(stage, &[prefix, suffix], 0.3)?;
        stage.play(
            suffix
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.0),
                    ..AnimateArgs::default()
                })?
                .move_to(v(2.45, -1.8), ORIGIN)?,
        )?;
        let extend = text(kit, stage, "Append the road letter on the right", -2.6, Palette::ink())?;
        show(stage, &[extend], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[prefix, suffix, extend], 0.3)?;
        kit.say(stage, "06_10")?;
        let question = text(
            kit,
            stage,
            "What if each letter goes at the FRONT?",
            -1.6,
            Palette::ink(),
        )?;
        show(stage, &[question], 0.3)?;
        prediction(kit, stage, 4, v(2.7, -2.5))?;
        hide(stage, &[question], 0.2)?;
        kit.say(stage, "06_11")?;
        let mirror = text(
            kit,
            stage,
            &format!(
                "Right: {}\nLeft: {}",
                badge(&words.badges[3]),
                badge(&reversed.badges[3])
            ),
            -1.6,
            Palette::gold(),
        )?;
        show(stage, &[mirror], 0.4)?;
        let order = text(
            kit,
            stage,
            "Extension keeps order · combination ignores order",
            -2.8,
            Palette::dim(),
        )?;
        show(stage, &[order], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[mirror, order], 0.3)?;
        hide(stage, &rows, 0.3)?;
        kit.say(stage, "06_12")?;
        let list = text(kit, stage, &badge(&words.badges[3]), 2.5, Palette::ink())?;
        show(stage, &[list], 0.3)?;
        let count = words.badges[3].len();
        let widest = ROUTES
            .iter()
            .map(|r| r.iter().map(|&i| ROADS[i].value).min().unwrap())
            .max()
            .unwrap();
        let summaries = [
            format!("Count: {count}"),
            format!("Quickest: {}", quick.badges[3].unwrap()),
            format!("Widest: {widest}"),
            format!("Reachable: {}", if words.badges[3].is_empty() { "no" } else { "yes" }),
        ];
        let mut summary_mobs = vec![list];
        for (i, s) in summaries.iter().enumerate() {
            let y = 1.35 - i as f64 * 0.9;
            let arrow = stage
                .arena_mut()
                .add(Arrow::new(v(0.25, 2.0), v(0.25, y)).buff(0.1).thickness(1.5).build()?);
            let row = text(kit, stage, s, y, Palette::teal())?;
            show(stage, &[arrow, row], 0.3)?;
            summary_mobs.extend([arrow, row]);
        }
        kit.hold(stage)?;
        kit.say(stage, "06_13")?;
        kit.hold(stage)?;
        hide(stage, &summary_mobs, 0.3)?;
        hide(stage, &letters, 0.3)?;
        // The minute labels left at 06_7; fading them again would put them back on screen.
        let drawn: Vec<Mob> = map.nodes.iter().chain(&map.names).chain(&map.roads).copied().collect();
        hide(stage, &drawn, 0.3)?;
        kit.say(stage, "06_14")?;
        let mut copies = Vec::new();
        for i in 0..10 {
            // Same shared Finish/Start geometry as chapter01: no extra roads.
            let scale = 0.25;
            let shift = [-5.7 + 5.3 * scale + i as f64 * 4.2 * scale, 1.65];
            let small = Map::build(kit, stage, shift, scale, false)?;
            show(stage, &small.all(), 0.15)?;
            copies.extend(small.all());
        }
        let n = 10u32;
        let min_len = ROUTES.iter().map(|r| r.len()).min().unwrap() * n as usize;
        let max_len = ROUTES.iter().map(|r| r.len()).max().unwrap() * n as usize;
        let totals = kit.text(
            stage,
            &format!(
                "{} road updates\n{} spellings · {min_len}–{max_len} letters each",
                ROADS.len() * n as usize,
                count.pow(n)
            ),
            34.0,
            Palette::gold(),
            v(0.0, 0.2),
        )?;
        show(stage, &[totals], 0.4)?;
        let spellings = ROUTES.iter().map(|r| spell(r)).collect::<Vec<_>>();
        let mut lines = Vec::new();
        for k in 0..5usize {
            let s = (0..10)
                .map(|i| spellings[(i + k) % spellings.len()].as_str())
                .collect::<String>();
            let line = kit.text(stage, &s, 20.0, Palette::dim(), v(0.0, -1.1 - k as f64 * 0.4))?;
            show(stage, &[line], 0.1)?;
            lines.push(line);
        }
        kit.hold(stage)?;
        hide(stage, &copies, 0.3)?;
        hide(stage, &lines, 0.3)?;
        hide(stage, &[totals], 0.3)?;
        kit.say(stage, "06_15")?;
        show(stage, &map.all(), 0.4)?;
        let recurrence = kit.tex(
            stage,
            r"F(v)=b(v)\oplus\bigoplus_{u\to v} F(u)\otimes w(u,v)",
            28.0,
            Palette::ink(),
            &[],
            v(2.7, 1.4),
        )?;
        show(stage, &[recurrence], 0.4)?;
        let example = text(
            kit,
            stage,
            &format!(
                "Lake: min(∞, 0 + {}, {} + {}) = {}",
                ROADS[1].value, ROADS[0].value, ROADS[2].value, lake
            ),
            -0.4,
            Palette::teal(),
        )?;
        show(stage, &[example], 0.4)?;
        let scope = text(
            kit,
            stage,
            "Finite map without loops\nEvery incoming road reports first",
            -1.9,
            Palette::dim(),
        )?;
        show(stage, &[scope], 0.3)?;
        kit.finish(stage)
    }
}

pub(crate) fn export_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    let forward = model::sweep::<Words<false>>(4, &ROADS, 0).map_err(|e| format!("{e:?}"))?;
    let reversed = model::sweep::<Words<true>>(4, &ROADS, 0).map_err(|e| format!("{e:?}"))?;
    for (i, place) in PLACES.iter().enumerate() {
        facts.add(&format!("words.forward.{place}"), badge(&forward.badges[i]))?;
        facts.add(&format!("words.reversed.{place}"), badge(&reversed.badges[i]))?;
    }
    let sorted: BTreeSet<_> = ROUTES.iter().map(|r| spell(r)).collect();
    facts.add("words.listed", badge(&sorted))?;
    facts.add(
        "words.ten.min_length",
        sorted.iter().map(String::len).min().ok_or("empty routes")? * 10,
    )?;
    facts.add(
        "words.ten.max_length",
        sorted.iter().map(String::len).max().ok_or("empty routes")? * 10,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn languages_match_independent_route_listing_for_every_closure() {
        for mask in 0..32 {
            let roads = std::array::from_fn::<_, 5, _>(|i| Road {
                open: mask & (1 << i) == 0,
                ..ROADS[i]
            });
            let listed: BTreeSet<_> = ROUTES
                .iter()
                .filter(|r| r.iter().all(|&i| roads[i].open))
                .map(|r| spell(r))
                .collect();
            let swept = model::sweep::<Words<false>>(4, &roads, 0).unwrap();
            assert_eq!(swept.badges[3], listed);
            let reverse: BTreeSet<String> = listed.iter().map(|s| s.chars().rev().collect()).collect();
            assert_eq!(model::sweep::<Words<true>>(4, &roads, 0).unwrap().badges[3], reverse);
        }
        assert_ne!(Words::<false>::zero(), Words::<false>::one());
        assert_eq!(3usize.pow(10), 59049);
    }
}
