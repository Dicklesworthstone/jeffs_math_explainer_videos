//! Cost and count: a useful product, and an intentionally broken infinity rule.
use super::draw::{Map, ROUTES, hide, prediction, route_color, show};
use crate::model::{self, Arithmetic, Best, Count, Error, PLACES, QuickestCount, ROADS, Road};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct Chapter05 {
    pub kit: Kit,
}

/// Demonstration only. None means infinite time; unlike Best, this permits
/// the invalid (infinite, positive count) that the chapter explicitly rejects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BadPair(Option<u64>, u64);
struct Broken;
impl Arithmetic for Broken {
    type Value = BadPair;
    fn zero() -> BadPair {
        BadPair(None, 0)
    }
    fn one() -> BadPair {
        BadPair(Some(0), 1)
    }
    fn road(r: Road) -> BadPair {
        BadPair(r.open.then_some(r.value), 1)
    }
    fn combine(a: &BadPair, b: &BadPair) -> Result<BadPair, Error> {
        let ordering = match (a.0, b.0) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        };
        Ok(match ordering {
            std::cmp::Ordering::Less => *a,
            std::cmp::Ordering::Greater => *b,
            std::cmp::Ordering::Equal => BadPair(a.0, a.1.checked_add(b.1).ok_or(Error::Overflow)?),
        })
    }
    fn extend(a: &BadPair, b: &BadPair) -> Result<BadPair, Error> {
        let time = match (a.0, b.0) {
            (Some(x), Some(y)) => Some(x.checked_add(y).ok_or(Error::Overflow)?),
            _ => None,
        };
        Ok(BadPair(time, a.1.checked_mul(b.1).ok_or(Error::Overflow)?))
    }
}

fn bad_text(pair: BadPair) -> String {
    format!("({}, {})", pair.0.map_or_else(|| "∞".into(), |t| t.to_string()), pair.1)
}
fn best_text(pair: Best) -> String {
    match pair {
        Best::Unreachable => "(∞, 0)".into(),
        Best::Finite { time, count } => format!("({time}, {count})"),
    }
}
fn enumerate_bad(roads: &[Road; 5]) -> BadPair {
    ROUTES.iter().fold(Broken::zero(), |total, route| {
        let value = route.iter().fold(Broken::one(), |value, &edge| {
            Broken::extend(&value, &Broken::road(roads[edge])).expect("tiny products")
        });
        Broken::combine(&total, &value).expect("tiny counts")
    })
}
fn panel(kit: &Kit, stage: &mut Stage<'_>, text: &str, y: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, text, 29.0, color, v(2.8, y))
}

struct PairBoard {
    values: Vec<Mob>,
    visible: [bool; 4],
}
impl PairBoard {
    fn build(kit: &Kit, stage: &mut Stage<'_>, values: &[Best]) -> fmn::Result<Self> {
        let mut board = Self {
            values: Vec::new(),
            visible: [true; 4],
        };
        for (i, place) in PLACES.iter().enumerate() {
            let y = 1.8 - i as f64 * 0.8;
            let name = kit.text(stage, place, 27.0, Palette::ink(), v(0.7, y))?;
            let frame = stage.arena_mut().add(
                Rectangle::new()
                    .width(3.1)
                    .height(0.6)
                    .style(Style::default().stroke(Palette::dim(), 1.2, 1.0))
                    .build()?
                    .moved_to(v(3.8, y)),
            );
            show(stage, &[name, frame], 0.2)?;
            board.values.extend(Self::fields(kit, stage, i, values[i])?);
        }
        Ok(board)
    }
    fn fields(kit: &Kit, stage: &mut Stage<'_>, i: usize, value: Best) -> fmn::Result<[Mob; 2]> {
        let (time, count) = match value {
            Best::Unreachable => ("∞".to_owned(), 0),
            Best::Finite { time, count } => (time.to_string(), count),
        };
        let y = 1.8 - i as f64 * 0.8;
        let fields = [
            kit.text(stage, &time, 29.0, Palette::gold(), v(3.1, y))?,
            kit.text(stage, &count.to_string(), 29.0, Palette::teal(), v(4.5, y))?,
        ];
        show(stage, &fields, 0.2)?;
        Ok(fields)
    }
    fn set(&mut self, kit: &Kit, stage: &mut Stage<'_>, i: usize, value: Best) -> fmn::Result<()> {
        if self.visible[i] {
            hide(stage, &self.values[i * 2..i * 2 + 2], 0.15)?;
        } else {
            // A second fade-out resurrects a concealed value. Keep its time, not its picture.
            stage.wait(0.15)?;
        }
        let fields = Self::fields(kit, stage, i, value)?;
        self.values[i * 2..i * 2 + 2].copy_from_slice(&fields);
        self.visible[i] = true;
        Ok(())
    }
    fn conceal_finish(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        hide(stage, &self.values[6..8], 0.2)?;
        self.visible[3] = false;
        Ok(())
    }
}

impl SceneConstruct for Chapter05 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let original = model::sweep::<QuickestCount>(4, &ROADS, 0).expect("tested DAG");
        let mut edited = ROADS;
        // The authored intervention changes only the Hill-to-Finish toll.
        edited[3].value = ROADS[4].value + ROADS[2].value;
        let tied = model::sweep::<QuickestCount>(4, &edited, 0).expect("tested DAG");
        let mut closed = ROADS;
        closed[3].open = false;
        closed[4].open = false;
        let shut = model::sweep::<QuickestCount>(4, &closed, 0).expect("tested DAG");
        let wrong = model::sweep::<Broken>(4, &closed, 0).expect("tested DAG");
        let wrong_list = enumerate_bad(&closed);
        let map = Map::build(kit, stage, [0.0, 0.0], 1.0, true)?;
        kit.say(stage, "05_1")?;
        show(stage, &map.all(), 0.5)?;
        let headline = panel(
            kit,
            stage,
            &format!("Finish: {}\n(time, fastest-route count)", best_text(original.badges[3])),
            0.8,
            Palette::ink(),
        )?;
        show(stage, &[headline], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &[headline], 0.3)?;

        kit.say(stage, "05_2")?;
        let headings = [
            kit.text(stage, "Time", 29.0, Palette::gold(), v(3.1, 2.7))?,
            kit.text(stage, "Count", 29.0, Palette::teal(), v(4.5, 2.7))?,
        ];
        show(stage, &headings, 0.3)?;
        let mut initial = vec![Best::Unreachable; PLACES.len()];
        initial[0] = QuickestCount::one();
        let mut board = PairBoard::build(kit, stage, &initial)?;
        for offer in &original.offers {
            let incoming = panel(
                kit,
                stage,
                &format!("offer {}", best_text(offer.offered)),
                -1.8,
                Palette::teal(),
            )?;
            show(stage, &[incoming], 0.2)?;
            map.travel(stage, &[offer.road_index], Palette::teal())?;
            board.set(kit, stage, ROADS[offer.road_index].to, offer.combined)?;
            hide(stage, &[incoming], 0.2)?;
        }
        kit.hold(stage)?;

        kit.say(stage, "05_3")?;
        hide(stage, &[map.costs[3]], 0.2)?;
        let edited_cost = kit.text(
            stage,
            &format!("{} · edited", edited[3].value),
            28.0,
            Palette::gold(),
            v(-1.65, 1.5),
        )?;
        show(stage, &[edited_cost], 0.3)?;
        board.conceal_finish(stage)?;
        let question = panel(kit, stage, "How many fastest routes now?", -1.8, Palette::ink())?;
        show(stage, &[question], 0.3)?;
        prediction(kit, stage, 5, v(2.8, -2.7))?;
        hide(stage, &[question], 0.2)?;
        kit.say(stage, "05_4")?;
        for offer in tied.offers.iter().filter(|o| ROADS[o.road_index].to == 3) {
            let incoming = panel(
                kit,
                stage,
                &format!("offer {}", best_text(offer.offered)),
                -1.8,
                Palette::teal(),
            )?;
            show(stage, &[incoming], 0.3)?;
            let route = if offer.road_index == 3 { ROUTES[0] } else { ROUTES[2] };
            map.travel(stage, route, Palette::teal())?;
            board.set(kit, stage, 3, offer.combined)?;
            hide(stage, &[incoming], 0.2)?;
        }
        let slower_time: u64 = ROUTES[1].iter().map(|&i| edited[i].value).sum();
        let slower = panel(
            kit,
            stage,
            &format!("Direct via Lake: {slower_time} minutes"),
            -1.8,
            route_color(1),
        )?;
        show(stage, &[slower], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[slower], 0.2)?;

        kit.say(stage, "05_5")?;
        let duplicate = tied.offers.last().expect("last incoming offer").offered;
        let replayed = QuickestCount::combine(&tied.badges[3], &duplicate).expect("small count");
        let replay = panel(
            kit,
            stage,
            &format!(
                "REPLAY: {}\nWrong: {}\nSame route counted twice",
                best_text(duplicate),
                best_text(replayed)
            ),
            -1.9,
            route_color(0),
        )?;
        show(stage, &[replay], 0.4)?;
        map.travel(stage, ROUTES[2], route_color(0))?;
        kit.hold(stage)?;
        hide(stage, &[replay, edited_cost], 0.3)?;

        kit.say(stage, "05_6")?;
        // The tied (6, 2) belongs to the edited map: conceal it before the road returns to seven.
        board.conceal_finish(stage)?;
        show(stage, &[map.costs[3]], 0.3)?;
        let mut barriers = Vec::new();
        for i in [3, 4] {
            let a = map.points[ROADS[i].from];
            let b = map.points[3];
            barriers.push(kit.text(
                stage,
                "×",
                53.0,
                route_color(0),
                v((a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0),
            )?);
        }
        show(stage, &barriers, 0.3)?;
        let question = panel(kit, stage, "How many fastest OPEN routes?", -1.8, Palette::ink())?;
        show(stage, &[question], 0.3)?;
        prediction(kit, stage, 5, v(2.8, -2.7))?;
        hide(stage, &[question], 0.2)?;
        kit.say(stage, "05_7")?;
        board.set(kit, stage, 3, shut.badges[3])?;
        let none = panel(kit, stage, "Unreachable means no open route", -1.8, Palette::teal())?;
        show(stage, &[none], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[none], 0.2)?;

        kit.say(stage, "05_8")?;
        let wrong_label = panel(
            kit,
            stage,
            &format!(
                "WRONG: closed road counts one\nSweep: {}    List: {}",
                bad_text(wrong.badges[3]),
                bad_text(wrong_list)
            ),
            -1.9,
            route_color(0),
        )?;
        show(stage, &[wrong_label], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &[wrong_label], 0.3)?;
        kit.say(stage, "05_9")?;
        let kept = BadPair(Some(ROADS[0].value + ROADS[2].value), 1);
        let discarded = BadPair(Some(ROADS[1].value), 1);
        let bad_road = Broken::road(closed[4]);
        let arrivals = [kept, discarded];
        let mut arrival_labels = Vec::new();
        for (i, arrival) in arrivals.iter().enumerate() {
            let line = panel(
                kit,
                stage,
                &format!("Lake {}", bad_text(*arrival)),
                -1.5 - i as f64 * 0.8,
                route_color(0),
            )?;
            show(stage, &[line], 0.3)?;
            arrival_labels.push(line);
        }
        kit.until_sentence(stage, "05_9", 3)?;
        hide(stage, &arrival_labels, 0.2)?;
        arrival_labels.clear();
        for (i, arrival) in arrivals.iter().enumerate() {
            let line = panel(
                kit,
                stage,
                &format!(
                    "Lake {} → {}",
                    bad_text(*arrival),
                    bad_text(Broken::extend(arrival, &bad_road).expect("small product"))
                ),
                -1.5 - i as f64 * 0.8,
                route_color(0),
            )?;
            show(stage, &[line], 0.3)?;
            map.travel(stage, &[4], route_color(0))?;
            arrival_labels.push(line);
        }
        kit.hold(stage)?;
        hide(stage, &arrival_labels, 0.2)?;
        kit.say(stage, "05_10")?;
        let left = Broken::extend(&Broken::combine(&kept, &discarded).unwrap(), &bad_road).unwrap();
        let right = Broken::combine(
            &Broken::extend(&kept, &bad_road).unwrap(),
            &Broken::extend(&discarded, &bad_road).unwrap(),
        )
        .unwrap();
        let law = panel(
            kit,
            stage,
            &format!(
                "Choose then extend: {}\nExtend then choose: {}\nDistributivity BROKEN",
                bad_text(left),
                bad_text(right)
            ),
            -2.0,
            route_color(0),
        )?;
        show(stage, &[law], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &[law], 0.3)?;

        kit.say(stage, "05_11")?;
        let repaired = panel(
            kit,
            stage,
            &format!(
                "Closed road contributes {}\nSweep and list agree: {}",
                best_text(QuickestCount::road(closed[4])),
                best_text(shut.badges[3])
            ),
            -1.7,
            Palette::teal(),
        )?;
        show(stage, &[repaired], 0.4)?;
        hide(stage, &barriers, 0.2)?;
        hide(stage, &[map.roads[3], map.roads[4]], 0.4)?;
        show(stage, &[map.roads[3], map.roads[4]], 0.4)?;
        show(stage, &barriers, 0.2)?;
        let credit = kit.text(stage, "Cost-and-count pairing: Baril, Couceiro & Lagerkvist\nDiscrete Applied Mathematics 2026 · arXiv 2512.03916", 20.0, Palette::dim(), v(0.0, -3.1))?;
        show(stage, &[credit], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[repaired, credit], 0.3)?;
        kit.say(stage, "05_12")?;
        let mut cells = Vec::new();
        let mut checked = 0;
        for mask in 0..(1usize << ROADS.len()) {
            let roads = std::array::from_fn::<_, 5, _>(|i| Road {
                open: mask & (1 << i) == 0,
                ..ROADS[i]
            });
            let good = model::sweep::<QuickestCount>(4, &roads, 0).unwrap();
            let bad = model::sweep::<Broken>(4, &roads, 0).unwrap();
            let matches = good.badges.iter().zip(&bad.badges).all(|(g, b)| match g {
                Best::Unreachable => true,
                Best::Finite { time, count } => *b == BadPair(Some(*time), *count),
            });
            if matches {
                checked += 1;
            }
            let cell = stage.arena_mut().add(
                Rectangle::new()
                    .width(0.36)
                    .height(0.24)
                    .style(Style::default().fill(if matches { Palette::teal() } else { route_color(0) }, 1.0))
                    .build()?
                    .moved_to(v(1.2 + (mask % 8) as f64 * 0.48, -1.25 - (mask / 8) as f64 * 0.36)),
            );
            show(stage, &[cell], 0.04)?;
            cells.push(cell);
        }
        let checked_label = panel(
            kit,
            stage,
            &format!("{checked} closure patterns\nReachable answers match in these checks"),
            -3.0,
            Palette::ink(),
        )?;
        show(stage, &[checked_label], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &cells, 0.3)?;
        hide(stage, &[checked_label], 0.3)?;
        kit.say(stage, "05_13")?;
        let all = model::sweep::<Count>(4, &ROADS, 0).unwrap().badges[3];
        let separate = panel(
            kit,
            stage,
            &format!(
                "All routes, ignoring closures: {all}\nFastest OPEN routes: {}",
                best_text(shut.badges[3])
            ),
            -1.9,
            Palette::teal(),
        )?;
        show(stage, &[separate], 0.4)?;
        kit.hold(stage)?;
        hide(stage, &barriers, 0.3)?;
        hide(stage, &[separate], 0.3)?;
        board.set(kit, stage, 3, original.badges[3])?;
        kit.finish(stage)
    }
}

pub(crate) fn export_facts(facts: &mut explainer_kit::Facts) -> Result<(), String> {
    for mask in 0..32 {
        let roads = std::array::from_fn::<_, 5, _>(|i| Road {
            open: mask & (1 << i) == 0,
            ..ROADS[i]
        });
        let good = model::sweep::<QuickestCount>(4, &roads, 0).map_err(|e| format!("{e:?}"))?;
        let bad = model::sweep::<Broken>(4, &roads, 0).map_err(|e| format!("{e:?}"))?;
        for (i, place) in PLACES.iter().enumerate() {
            facts.add(&format!("pairs.closed.{mask}.good.{place}"), best_text(good.badges[i]))?;
            facts.add(&format!("pairs.closed.{mask}.bad.{place}"), bad_text(bad.badges[i]))?;
        }
        facts.add(&format!("pairs.closed.{mask}.listed"), bad_text(enumerate_bad(&roads)))?;
    }
    let mut tied = ROADS;
    tied[3].value = ROADS[4].value + ROADS[2].value;
    let result = model::sweep::<QuickestCount>(4, &tied, 0).map_err(|e| format!("{e:?}"))?;
    for (i, place) in PLACES.iter().enumerate() {
        facts.add(&format!("pairs.tied.{place}"), best_text(result.badges[i]))?;
    }
    let duplicate = Best::Finite {
        time: ROADS[0].value + tied[3].value,
        count: 1,
    };
    facts.add(
        "pairs.tied.replay",
        best_text(QuickestCount::combine(&result.badges[3], &duplicate).map_err(|e| format!("{e:?}"))?),
    )?;
    let a = BadPair(Some(ROADS[0].value + ROADS[2].value), 1);
    let b = BadPair(Some(ROADS[1].value), 1);
    let closed = BadPair(None, 1);
    let combined = Broken::combine(&a, &b).map_err(|e| format!("{e:?}"))?;
    let left = Broken::extend(&combined, &closed).map_err(|e| format!("{e:?}"))?;
    let ac = Broken::extend(&a, &closed).map_err(|e| format!("{e:?}"))?;
    let bc = Broken::extend(&b, &closed).map_err(|e| format!("{e:?}"))?;
    let right = Broken::combine(&ac, &bc).map_err(|e| format!("{e:?}"))?;
    facts.add("pairs.law.left", bad_text(left))?;
    facts.add("pairs.law.right", bad_text(right))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deliberate_failure_and_repair_are_distinct() {
        let mut roads = ROADS;
        roads[3].open = false;
        roads[4].open = false;
        assert_eq!(
            model::sweep::<Broken>(4, &roads, 0).unwrap().badges[3],
            BadPair(None, 2)
        );
        assert_eq!(enumerate_bad(&roads), BadPair(None, 3));
        assert_eq!(
            model::sweep::<QuickestCount>(4, &roads, 0).unwrap().badges[3],
            Best::Unreachable
        );
        let absent: Vec<_> = roads.into_iter().filter(|r| r.open).collect();
        assert_eq!(
            model::sweep::<QuickestCount>(4, &absent, 0).unwrap().badges[3],
            Best::Unreachable
        );
    }
    #[test]
    fn finite_answers_survive_all_closures_but_unreachable_counts_need_not() {
        let mut disagreements = 0;
        for mask in 0..32 {
            let roads = std::array::from_fn::<_, 5, _>(|i| Road {
                open: mask & (1 << i) == 0,
                ..ROADS[i]
            });
            let good = model::sweep::<QuickestCount>(4, &roads, 0).unwrap();
            let bad = model::sweep::<Broken>(4, &roads, 0).unwrap();
            for (g, b) in good.badges.iter().zip(&bad.badges) {
                if let Best::Finite { time, count } = g {
                    assert_eq!(*b, BadPair(Some(*time), *count));
                }
            }
            disagreements += usize::from(bad.badges[3] != enumerate_bad(&roads));
        }
        assert!(disagreements > 0);
    }
}
