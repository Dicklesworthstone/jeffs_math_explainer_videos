//! The distributive law is earned by comparing two orders of computation.
use super::draw::{Map, hide, route_color, show};
use crate::model::{self, Arithmetic, Count, PLACES, Quickest, ROADS, Reachable, Widest, Width};
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;

pub struct Chapter04 {
    pub kit: Kit,
}

fn label(kit: &Kit, stage: &mut Stage<'_>, text: &str, x: f64, y: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text(stage, text, 25.0, color, v(x, y))
}

impl SceneConstruct for Chapter04 {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = &self.kit;
        let mut maps = Vec::new();
        let mut table = Vec::new();
        let xs = [-2.7, -0.6, 1.2, 3.3, 5.35];
        let ys = [1.8, 0.9, 0.0, -0.9];
        let questions = ["Quickest", "Count", "Reachable", "Widest"];
        let extension = ["+", "× 1", "AND open", "min"];
        let combination = ["min", "+", "OR", "max"];
        let colors = [Palette::gold(), Palette::teal(), route_color(1), route_color(2)];
        let quick = model::sweep::<Quickest>(PLACES.len(), &ROADS, 0).expect("tested DAG");
        let counts = model::sweep::<Count>(PLACES.len(), &ROADS, 0).expect("tested DAG");
        let reachable = model::sweep::<Reachable>(PLACES.len(), &ROADS, 0).expect("tested DAG");
        let widest = model::sweep::<Widest>(PLACES.len(), &ROADS, 0).expect("tested DAG");
        let Width::Finite(limit) = widest.badges[3] else {
            unreachable!()
        };
        let answers = [
            quick.badges[3].expect("connected Finish").to_string(),
            counts.badges[3].to_string(),
            if reachable.badges[3] { "yes" } else { "no" }.to_owned(),
            limit.to_string(),
        ];
        kit.say(stage, "04_1")?;
        for (i, y) in ys.iter().enumerate() {
            let miniature = Map::build(kit, stage, [-4.9, *y], 0.19, false)?;
            show(stage, &miniature.all(), 0.3)?;
            maps.extend(miniature.all());
            let answer = label(kit, stage, &answers[i], -4.4, *y, colors[i])?;
            show(stage, &[answer], 0.2)?;
            maps.push(answer);
            let name = label(kit, stage, questions[i], xs[0], *y, colors[i])?;
            show(stage, &[name], 0.2)?;
            table.push(name);
        }
        kit.hold(stage)?;
        kit.say(stage, "04_2")?;
        for (i, heading) in ["Question", "Extend", "Combine"].iter().enumerate() {
            let heading = label(kit, stage, heading, xs[i], 2.8, Palette::ink())?;
            show(stage, &[heading], 0.2)?;
            table.push(heading);
        }
        // Each operation row waits for its own spoken sentence.
        for i in 0..questions.len() {
            kit.until_sentence(stage, "04_2", i + 1)?;
            for (column, text) in [(1, extension[i]), (2, combination[i])] {
                let cell = label(kit, stage, text, xs[column], ys[i], colors[i])?;
                show(stage, &[cell], 0.25)?;
                table.push(cell);
            }
        }
        kit.hold(stage)?;
        kit.say(stage, "04_3")?;
        for (i, heading) in ["At Start", "No route"].iter().enumerate() {
            let heading = label(kit, stage, heading, xs[i + 3], 2.8, Palette::ink())?;
            show(stage, &[heading], 0.2)?;
            table.push(heading);
        }
        let starts = [
            Quickest::one().expect("empty route time").to_string(),
            Count::one().to_string(),
            "yes".to_owned(),
            "no limit".to_owned(),
        ];
        let absent = [
            "no route yet".to_owned(),
            Count::zero().to_string(),
            "no".to_owned(),
            "nothing".to_owned(),
        ];
        for i in 0..questions.len() {
            kit.until_sentence(stage, "04_3", i + 2)?;
            for (column, text) in [(3, &starts[i]), (4, &absent[i])] {
                let cell = label(kit, stage, text, xs[column], ys[i], colors[i])?;
                show(stage, &[cell], 0.25)?;
                table.push(cell);
            }
        }
        let infinity_note = kit.text(
            stage,
            "∞ represents no finite time / no width limit",
            24.0,
            Palette::dim(),
            v(1.2, -1.8),
        )?;
        let width_note = kit.text(
            stage,
            "Positive widths here: zero capacity would also let nothing through.",
            23.0,
            Palette::dim(),
            v(0.0, -2.45),
        )?;
        show(stage, &[infinity_note, width_note], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[infinity_note, width_note], 0.3)?;

        kit.say(stage, "04_4")?;
        hide(stage, &maps, 0.3)?;
        hide(stage, &table, 0.4)?;
        let a = ROADS[1].value;
        let b = ROADS[0].value + ROADS[2].value;
        let w = ROADS[4].value;
        let inputs = kit.text(
            stage,
            &format!("Lake offers: {a} and {b} minutes     Last road: +{w}"),
            32.0,
            Palette::ink(),
            v(0.0, 2.7),
        )?;
        let first = kit.text(stage, "Choose, then extend", 30.0, Palette::gold(), v(0.0, 1.6))?;
        let second = kit.text(stage, "Extend each, then choose", 30.0, Palette::teal(), v(0.0, -0.6))?;
        show(stage, &[inputs, first, second], 0.4)?;
        kit.hold(stage)?;
        kit.say(stage, "04_5")?;
        let chosen = Quickest::combine(&Some(a), &Some(b)).expect("finite offers");
        let left = Quickest::extend(&chosen, &Some(w)).expect("small sum").unwrap();
        let ea = Quickest::extend(&Some(a), &Some(w)).expect("small sum").unwrap();
        let eb = Quickest::extend(&Some(b), &Some(w)).expect("small sum").unwrap();
        let right = Quickest::combine(&Some(ea), &Some(eb)).expect("finite offers").unwrap();
        let left_line = kit.text(
            stage,
            &format!("min({a}, {b}) + {w} = {left}"),
            39.0,
            Palette::gold(),
            v(0.0, 0.65),
        )?;
        show(stage, &[left_line], 0.5)?;
        kit.until_sentence(stage, "04_5", 1)?;
        let right_line = kit.text(
            stage,
            &format!("min({a} + {w}, {b} + {w}) = min({ea}, {eb}) = {right}"),
            35.0,
            Palette::teal(),
            v(0.0, -1.55),
        )?;
        show(stage, &[right_line], 0.5)?;
        let equal = kit.text(stage, &format!("{left} = {right}"), 43.0, Palette::teal(), v(0.0, -2.7))?;
        show(stage, &[equal], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[inputs, left_line, right_line, equal], 0.3)?;

        kit.say(stage, "04_6")?;
        let b = ROADS[2].value;
        let wa = Width::Finite(a);
        let wb = Width::Finite(b);
        let ww = Width::Finite(w);
        let combined = Widest::combine(&wa, &wb).expect("width choice");
        let Width::Finite(left) = Widest::extend(&combined, &ww).expect("width limit") else {
            unreachable!()
        };
        let ea = Widest::extend(&wa, &ww).expect("width limit");
        let eb = Widest::extend(&wb, &ww).expect("width limit");
        let Width::Finite(right) = Widest::combine(&ea, &eb).expect("width choice") else {
            unreachable!()
        };
        let left_line = kit.text(
            stage,
            &format!("min(max({a}, {b}), {w}) = {left}"),
            37.0,
            Palette::gold(),
            v(0.0, 0.65),
        )?;
        show(stage, &[left_line], 0.4)?;
        // Sentence 1 reads the left form; the right form is sentence 2.
        kit.until_sentence(stage, "04_6", 2)?;
        let right_line = kit.text(
            stage,
            &format!("max(min({a}, {w}), min({b}, {w})) = {right}"),
            35.0,
            Palette::teal(),
            v(0.0, -1.55),
        )?;
        show(stage, &[right_line], 0.4)?;
        let one = Count::one();
        let sum = Count::combine(&one, &one).expect("small count");
        let count = Count::extend(&sum, &one).expect("small count");
        let count_line = kit.text(
            stage,
            &format!("Counting: ({one} + {one}) × {one} = {one} × {one} + {one} × {one} = {count}"),
            27.0,
            Palette::dim(),
            v(0.0, -2.8),
        )?;
        show(stage, &[count_line], 0.3)?;
        kit.hold(stage)?;
        hide(stage, &[first, second, left_line, right_line, count_line], 0.4)?;

        kit.say(stage, "04_7")?;
        show(stage, &maps, 0.3)?;
        show(stage, &table, 0.4)?;
        let name = kit.text(
            stage,
            "Semiring · distributive law",
            32.0,
            Palette::teal(),
            v(0.0, -1.9),
        )?;
        let law = kit.text(
            stage,
            "(x ⊕ y) ⊗ w = (x ⊗ w) ⊕ (y ⊗ w)",
            34.0,
            Palette::ink(),
            v(0.0, -2.65),
        )?;
        let legend = kit.text(stage, "⊕ combine       ⊗ extend", 24.0, Palette::dim(), v(0.0, -3.3))?;
        show(stage, &[name, law, legend], 0.5)?;
        kit.hold(stage)?;
        kit.say(stage, "04_8")?;
        hide(stage, &maps, 0.3)?;
        hide(stage, &table, 0.3)?;
        hide(stage, &[name, law, legend], 0.3)?;
        let map = Map::build(kit, stage, [0.0, 0.0], 1.0, true)?;
        show(stage, &map.all(), 0.5)?;
        let next = kit.text(
            stage,
            "Fastest route\nand how many tie?",
            37.0,
            Palette::ink(),
            v(2.8, 0.0),
        )?;
        show(stage, &[next], 0.4)?;
        kit.hold(stage)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_laws<A: Arithmetic>(values: &[A::Value]) {
        for x in values {
            assert_eq!(A::combine(x, &A::zero()).unwrap(), *x);
            assert_eq!(A::extend(x, &A::one()).unwrap(), *x);
            assert_eq!(A::extend(x, &A::zero()).unwrap(), A::zero());
            for y in values {
                assert_eq!(A::combine(x, y).unwrap(), A::combine(y, x).unwrap());
                for w in values {
                    let sum = A::combine(x, y).unwrap();
                    let xw = A::extend(x, w).unwrap();
                    let yw = A::extend(y, w).unwrap();
                    assert_eq!(A::extend(&sum, w).unwrap(), A::combine(&xw, &yw).unwrap());
                    let wx = A::extend(w, x).unwrap();
                    let wy = A::extend(w, y).unwrap();
                    assert_eq!(A::extend(w, &sum).unwrap(), A::combine(&wx, &wy).unwrap());
                }
            }
        }
    }

    #[test]
    fn four_law_rows_include_empty_routes_and_unreachable_values() {
        check_laws::<Quickest>(&[None, Some(0), Some(1), Some(3), Some(5)]);
        check_laws::<Count>(&[0, 1, 2, 3, 5]);
        check_laws::<Reachable>(&[false, true]);
        check_laws::<Widest>(&[
            Width::Unreachable,
            Width::Finite(1),
            Width::Finite(3),
            Width::Finite(5),
            Width::Unlimited,
        ]);
    }
}
