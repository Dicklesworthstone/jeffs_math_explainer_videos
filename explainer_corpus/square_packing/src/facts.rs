//! Every number the five-tile video shows or says, read off the packings
//! `model.rs` builds and checked two ways: `pin` against a literal from the
//! sources (fact sheet §Sources), and `./x square_packing facts-check`
//! against `oracle.py` (same keys), which rebuilds every packing by its own
//! route (exact polygon clipping, not shadows; the side solved from another
//! condition). If a check fails, `facts()` fails and nothing renders.
//!
//! The tile order in every list: 0 lower left, 1 lower right, 2 upper left,
//! 3 upper right, 4 the middle.

use explainer_kit::Facts;

use crate::model::{self, MIDDLE, Packing, Point, Surd, TILES};

/// Literals quoted from the sources (docs/fact-sheet.md §Sources; the oracle
/// prints the same values).
/// What chapter 03 chooses to show (the oracle prints the same choices):
/// the lens's magnification over the board, the half-width of the window it
/// shows (tiles) and its scale bar, the held corner it watches, and the
/// ever smaller turns whose bite is quoted.
pub mod shown {
    pub const LENS_MAG: u32 = 44;
    pub const LENS_HALF: (i64, i64) = (1, 40);
    pub const LENS_BAR: (i64, i64) = (1, 100);
    /// The lower right corner tile: its inner corner is the one magnified.
    pub const LENS_CORNER: usize = 1;
    /// Half-angle tangents 1/n of the tiny turns.
    pub const TINY: [i64; 2] = [100, 10_000];
    /// Half-angle tangent 1/n of the turn past 30° whose overlap has four
    /// sides.
    pub const BEYOND: i64 = 3;
}

pub mod cited {
    /// F. Göbel, "Geometrical Packing and Covering Problems", in A. Schrijver
    /// (ed.), *Packing and Covering in Combinatorics*, Math Centrum Tracts 106
    /// (1979), 179–199 (Friedman, DS7, reference [7]): the five-tile packing.
    pub const GOBEL: u32 = 1979;
    /// The case page's side for five tiles, 2 + √2/2, in the facts' form;
    /// Friedman's Table 1 writes it 2 + 1/√2.
    pub const SIDE: &str = "2 + 1/2*sqrt(2)";
    /// The case page's decimal, 2.70710678118654752440084436210485, to five
    /// places.
    pub const SIDE_5DP: &str = "2.70711";
    /// "A 3×3 grid would need side 3" (the case page).
    pub const GRID: &str = "3";
}

fn ensure(ok: bool, what: impl FnOnce() -> String) -> Result<(), String> {
    if ok { Ok(()) } else { Err(what()) }
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    grid(&mut f)?;
    turn(&mut f)?;
    turned(&mut f)?;
    planted(&mut f)?;
    pins(&mut f)?;
    hair(&mut f)?;
    f.add("source.gobel", cited::GOBEL)?;
    Ok(f)
}

/// 01_1 to 01_3: five tiles; four make a full grid two wide; the straight
/// grid that holds five is three wide, with room for four more; the five
/// straight tiles fit there, the middle one touching its four neighbours at
/// their corners.
fn grid(f: &mut Facts) -> Result<(), String> {
    let full = model::full_grid(TILES);
    let side = model::grid_side(TILES);
    f.add("tiles", TILES)?;
    f.add("grid.full", full)?;
    f.add("grid.full.holds", full * full)?;
    f.pin(
        "grid.side",
        side,
        cited::GRID,
        "the case page: a 3×3 grid would need side 3",
    )?;
    f.add("grid.room", side * side - TILES)?;
    let straight = model::five_straight().check_exact();
    ensure(straight.fits(), || {
        format!("the straight grid does not fit: {straight:?}")
    })?;
    f.add("straight.check", straight.summary())?;
    f.add("straight.touching", straight.touching_list())?;
    Ok(())
}

/// 01_5: the turning tile's corners stay at the distance of the inner
/// corners from the centre (squared, 1/2 both), so it never reaches past
/// them; it turns an eighth of a turn.
fn turn(f: &mut Facts) -> Result<(), String> {
    let (reach, inner) = (model::turn_reach2(), model::turn_inner2());
    ensure(reach == inner, || {
        format!("the turning tile reaches {reach}, the inner corners are at {inner}")
    })?;
    f.add("turn.reach2", reach)?;
    f.add("turn.inner2", inner)?;
    let (c, s) = model::eighth_turn();
    let degrees = s.to_f64().atan2(c.to_f64()).to_degrees();
    f.add("turned.degrees", format!("{degrees:.0}"))?;
    Ok(())
}

/// 01_6 to 01_11: where the squeeze stops. The gap, the box, its decimals;
/// opposite inner corners one tile apart; the exact check (it fits, the
/// middle tile touches all four corner tiles, the corner tiles touch two
/// walls each) and the four touches at the midpoints of the turned tile's
/// sides.
fn turned(f: &mut Facts) -> Result<(), String> {
    let gap = model::turned_gap();
    f.add("turned.gap", gap)?;
    f.add("turned.gap.2dp", gap.decimals(2))?;
    let side = model::turned_side();
    f.pin(
        "turned.side",
        side,
        cited::SIDE,
        "the case page: 2 + √2/2; Friedman DS7, Table 1: 2 + 1/√2 (Göbel)",
    )?;
    f.add("turned.side.2dp", side.decimals(2))?;
    f.pin(
        "turned.side.5dp",
        side.decimals(5),
        cited::SIDE_5DP,
        "the case page: 2.70710678118654752440084436210485",
    )?;
    let p = model::five_turned();
    let (a, b) = (p.inner_corner(0), p.inner_corner(3));
    f.add(
        "turned.diagonal2",
        (b.0 - a.0) * (b.0 - a.0) + (b.1 - a.1) * (b.1 - a.1),
    )?;
    let v = p.check_exact();
    ensure(v.fits(), || format!("the five turned tiles do not fit: {v:?}"))?;
    f.add("turned.check", v.summary())?;
    f.add("turned.touching", v.touching_list())?;
    f.add("turned.walls", v.walls.len())?;
    let mids = p.tiles[MIDDLE].midpoints();
    f.add(
        "turned.midpoints",
        (0..4).filter(|&k| mids.contains(&p.inner_corner(k))).count(),
    )?;
    Ok(())
}

/// The planted failures (`model::planted`): each must be caught, and the
/// facts say which tiles broke.
fn planted(f: &mut Facts) -> Result<(), String> {
    for p in model::planted() {
        let v = p.packing.check_exact();
        ensure(!v.fits(), || format!("the planted failure {} passes the check", p.key))?;
        f.add(&format!("planted.{}", p.key), v.summary())?;
    }
    Ok(())
}

/// 02: the four pins. In the turned packing's box the marks cut each side
/// into three pieces shorter than a tile; every shadow and every straight
/// tile holds a mark or a pin strictly inside (checked exactly for every
/// position); not at three, where the closed version still "holds"; no room
/// at all below one tile; and who holds which pin in the chapter's three
/// pictures.
fn pins(f: &mut Facts) -> Result<(), String> {
    let side = model::turned_side();
    let threshold = model::pin_threshold();
    f.add("pins.count", model::pins(side).len())?;
    let piece = model::longest_piece(side);
    ensure(piece < Surd::int(1), || format!("a piece of the turned box is {piece}"))?;
    f.add("pins.piece", piece)?;
    f.add("pins.piece.2dp", piece.decimals(2))?;
    ensure(model::longest_piece(threshold) == Surd::int(1), || {
        format!("at {threshold} a piece is not one tile")
    })?;
    f.add("pins.threshold", threshold)?;
    f.add("pins.lemma", step(model::every_shadow_holds_a_mark(side)))?;
    f.add("pins.lemma.at3", step(model::every_shadow_holds_a_mark(threshold)))?;
    f.add(
        "pins.lemma.half",
        step(model::every_shadow_holds_a_mark(Surd::ratio(1, 2))),
    )?;
    f.add(
        "pins.closed.at3",
        step(model::every_closed_shadow_touches_a_mark(threshold)),
    )?;
    f.add(
        "pins.straight_lemma",
        step(model::every_straight_tile_holds_a_pin(side)),
    )?;
    f.add(
        "pins.straight_lemma.at3",
        step(model::every_straight_tile_holds_a_pin(threshold)),
    )?;
    let straight = model::corners_and_middle(side, Surd::int(1), Surd::ZERO);
    f.add("pins.owners.straight", owners(&straight))?;
    f.add("pins.owners.turned", owners(&model::five_turned()))?;
    f.add("pins.owners.at3", owners(&model::five_straight()))?;
    Ok(())
}

/// 03: a turn by a hair. The box, the corner tiles and the middle tile's
/// centre held; the middle tile turned a further exact small turn, either
/// way, and twice as far. Every held corner on the inner circle; its depth
/// inside the turned tile, the same at all four; the overlap at the
/// magnified corner, clipped exactly; the box of three, where the same turn
/// is free; ever smaller turns, never zero; the lens's choices, and that its
/// window holds the larger overlap; and the four-sided overlap past 30°.
fn hair(f: &mut Facts) -> Result<(), String> {
    let p = model::five_turned();
    let c = p.tiles[MIDDLE].centre;
    let d2 = |q: Point<Surd>| (q.0 - c.0) * (q.0 - c.0) + (q.1 - c.1) * (q.1 - c.1);
    let inner = d2(p.inner_corner(0));
    ensure((0..4).all(|k| d2(p.inner_corner(k)) == inner), || {
        "the held corners are not all the same distance from the centre".to_owned()
    })?;
    f.add("hair.inner2", inner)?;
    f.add("hair.inradius2", d2(p.tiles[MIDDLE].midpoints()[0]))?;
    let one = model::small_turn();
    let two = model::compose(one, one);
    let minus = (one.0, -one.1);
    f.add("hair.turn", format!("{},{}", one.0, one.1))?;
    let degrees = |t: Point<Surd>| t.1.to_f64().atan2(t.0.to_f64()).to_degrees();
    f.add("hair.degrees.1dp", format!("{:.1}", degrees(one)))?;
    f.add("hair.degrees2.1dp", format!("{:.1}", degrees(two)))?;
    let depth = model::held_corner_depth;
    for turn in [one, two, minus] {
        let q = model::hair_turned(turn);
        ensure(
            (0..4).all(|k| model::depth_inside(&q.tiles[MIDDLE], q.inner_corner(k)) == depth(turn)),
            || "the held corners do not all sink the same depth".to_owned(),
        )?;
    }
    let (d1, d2v) = (depth(one), depth(two));
    f.add("hair.depth", d1)?;
    f.add("hair.depth.4dp", d1.decimals(4))?;
    f.add("hair.depth2", d2v)?;
    f.add("hair.depth2.4dp", d2v.decimals(4))?;
    let ratio = d2v * d1.recip();
    f.add("hair.ratio", ratio)?;
    f.add("hair.ratio.2dp", ratio.decimals(2))?;
    let k = shown::LENS_CORNER;
    let bite = |turn: Point<Surd>| {
        let q = model::hair_turned(turn);
        model::overlap_polygon(&q.tiles[k], &q.tiles[MIDDLE])
    };
    let (b1, b2, bm) = (bite(one), bite(two), bite(minus));
    f.add("hair.vertices", b1.len())?;
    f.add("hair.area", model::polygon_area(&b1))?;
    f.add("hair.area2", model::polygon_area(&b2))?;
    f.add("hair.minus", format!("{}|{}", depth(minus), model::polygon_area(&bm)))?;
    f.add("hair.check", model::hair_turned(one).check_exact().summary())?;
    f.add("hair.check.minus", model::hair_turned(minus).check_exact().summary())?;
    let roomy: Vec<&str> = [one, minus]
        .iter()
        .map(|&turn| {
            let v = model::hair_turned_roomy(turn).check_exact();
            if v.fits() && v.touching.is_empty() {
                "free"
            } else {
                "blocked"
            }
        })
        .collect();
    f.add("hair.roomy", roomy.join("|"))?;
    let tiny: Vec<String> = shown::TINY
        .iter()
        .map(|&n| depth(model::half_angle_turn(n)).to_string())
        .collect();
    f.add("hair.tiny", tiny.join(","))?;
    let half = Surd::ratio(shown::LENS_HALF.0, shown::LENS_HALF.1);
    f.add("hair.lens.mag", shown::LENS_MAG)?;
    f.add("hair.lens.half", half)?;
    f.add("hair.lens.bar", Surd::ratio(shown::LENS_BAR.0, shown::LENS_BAR.1))?;
    let held = p.inner_corner(k);
    let near = |x: Surd, y: Surd| x - y <= half && y - x <= half;
    let fits = b2.iter().all(|v| near(v.0, held.0) && near(v.1, held.1));
    f.add("hair.lens.fits", if fits { "yes" } else { "no" })?;
    let beyond = model::hair_turned(model::half_angle_turn(shown::BEYOND));
    let quad = model::overlap_polygon(&beyond.tiles[0], &beyond.tiles[MIDDLE]);
    f.add("hair.beyond", format!("{}|{}", quad.len(), model::polygon_area(&quad)))?;
    Ok(())
}

/// A step's verdict as the facts print it.
fn step(verdict: Option<bool>) -> &'static str {
    match verdict {
        Some(true) => "yes",
        Some(false) => "no",
        None => "no room",
    }
}

/// Each tile's pins strictly inside it, tile by tile: `0|1|2|3|0,1,2,3`.
fn owners(p: &Packing<Surd>) -> String {
    let rows: Vec<String> = model::pin_owners(p)
        .iter()
        .map(|held| {
            if held.is_empty() {
                "none".to_owned()
            } else {
                held.iter().map(usize::to_string).collect::<Vec<_>>().join(",")
            }
        })
        .collect();
    rows.join("|")
}

#[cfg(test)]
mod tests {
    use explainer_kit::Script;

    use super::*;

    fn value(key: &str) -> String {
        let f = facts().unwrap();
        f.get(key).unwrap_or_else(|| panic!("no fact {key}")).to_owned()
    }

    fn line(id: &str) -> String {
        Script::parse(crate::SCRIPT).unwrap().text(id).unwrap().to_owned()
    }

    /// A small whole number as the narration says it.
    fn word(n: u32) -> &'static str {
        [
            "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
        ][n as usize]
    }

    /// A decimal as the narration says it: "2.71" is "two point seven one".
    fn spoken(decimal: &str) -> String {
        let (whole, part) = decimal.split_once('.').unwrap();
        let digit = |c: char| word(c.to_digit(10).unwrap());
        let mut out = word(whole.parse().unwrap()).to_owned();
        out.push_str(" point");
        for c in part.chars() {
            out.push(' ');
            out.push_str(digit(c));
        }
        out
    }

    fn int(key: &str) -> u32 {
        value(key).parse().unwrap()
    }

    #[test]
    fn the_facts_self_check() {
        facts().unwrap().self_check().unwrap();
    }

    /// What the voice says, on the numbers the video computes.
    #[test]
    fn the_narration_says_the_facts() {
        assert!(line("01_1").starts_with(&format!("{} ", capital(word(int("tiles"))))));
        let l2 = line("01_2");
        assert!(
            l2.starts_with(&format!("{} fit", capital(word(int("grid.full.holds"))))),
            "{l2}"
        );
        assert!(l2.contains(&format!("a box {} wide", word(int("grid.full")))), "{l2}");
        let l3 = line("01_3");
        assert!(l3.contains(&format!("{} wide", word(int("grid.side")))), "{l3}");
        assert!(
            l3.contains(&format!("room for {} more", word(int("grid.room")))),
            "{l3}"
        );
        // 01_8: one tile apart is the squared distance 1
        assert_eq!(value("turned.diagonal2"), "1");
        assert!(line("01_8").contains("one tile apart"));
        // 01_9: the gap is 1/√2 = √2/2, the box two plus it, and the rounding
        let l9 = line("01_9");
        assert_eq!(value("turned.gap"), "1/2*sqrt(2)");
        assert!(l9.contains("the gap is one over the square root of two"), "{l9}");
        assert_eq!(value("turned.side"), format!("2 + {}", value("turned.gap")));
        assert!(
            l9.contains(&format!("about {}", spoken(&value("turned.side.2dp")))),
            "{l9}"
        );
        // 01_7: each inner corner touches the middle of a side
        assert_eq!(value("turned.midpoints"), "4");
        assert_eq!(value("turned.touching"), "0-4,1-4,2-4,3-4");
        // 01_10: one step further overlaps every corner tile
        assert_eq!(value("planted.squeeze_further"), "overlap:0-4,1-4,2-4,3-4");
    }

    fn capital(w: &str) -> String {
        let mut c = w.chars();
        c.next()
            .map(|first| first.to_uppercase().chain(c).collect())
            .unwrap_or_default()
    }

    #[test]
    fn spoken_numbers_read_as_the_narration_reads_them() {
        assert_eq!(spoken("2.71"), "two point seven one");
        assert_eq!(spoken("0.71"), "zero point seven one");
        assert_eq!(capital("five"), "Five");
    }

    /// Every planted failure is in the facts, and none of them fits.
    #[test]
    fn every_planted_failure_is_a_fact_and_a_failure() {
        let f = facts().unwrap();
        for p in model::planted() {
            let v = f.get(&format!("planted.{}", p.key)).unwrap();
            assert_ne!(v, "fits", "{}", p.key);
        }
        assert_eq!(f.get("turned.check"), Some("fits"));
        assert_eq!(f.get("straight.check"), Some("fits"));
    }

    /// What chapter 02's voice says, on the numbers the model computes:
    /// the marks at a third and two thirds, three pieces each shorter than a
    /// tile, four pins, at most four straight tiles, three as the boundary
    /// where five fit, and no pin inside the turned tile.
    #[test]
    fn the_four_pins_say_the_facts() {
        let three = word(int("pins.threshold"));
        let unit = model::marks(Surd::int(1));
        assert_eq!(unit, [Surd::ratio(1, 3), Surd::ratio(2, 3)]);
        let l1 = line("02_1");
        assert!(l1.contains(&format!("a box {three} wide")), "{l1}");
        assert!(l1.contains("a third and two thirds"), "{l1}");
        let l2 = line("02_2");
        let pieces = word(u32::try_from(unit.len()).unwrap() + 1);
        assert!(l2.contains(&format!("{pieces} equal pieces")), "{l2}");
        assert!(l2.contains("shorter than one tile"), "{l2}");
        assert!(value("pins.piece.2dp").parse::<f64>().unwrap() < 1.0);
        assert_eq!(value("pins.lemma"), "yes");
        assert!(line("02_3").contains("strictly inside"));
        let pins = word(int("pins.count"));
        let l4 = line("02_4");
        assert!(l4.contains(&format!("these {pins} pins strictly inside")), "{l4}");
        assert_eq!(value("pins.straight_lemma"), "yes");
        let l6 = line("02_6");
        assert!(l6.starts_with(&format!("{} at most", capital(pins))), "{l6}");
        let five = word(int("tiles"));
        assert!(
            l6.contains(&format!("{five} straight tiles need a box at least {three} wide")),
            "{l6}"
        );
        let l7 = line("02_7");
        assert!(l7.starts_with(&format!("At exactly {three},")), "{l7}");
        assert!(l7.ends_with(&format!("{five} fit.")), "{l7}");
        assert_eq!(value("pins.lemma.at3"), "no");
        assert_eq!(value("straight.check"), "fits");
        assert_eq!(value("pins.owners.at3"), "none|none|none|none|none");
        assert!(line("02_8").contains("no pin is inside the turned tile"));
        assert!(value("pins.owners.turned").ends_with("|none"));
    }
}
