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
    /// W. Stromquist, "Packing 10 or 11 unit squares in a square",
    /// *Electronic Journal of Combinatorics* 10 (2003) #R8 (fact sheet S4):
    /// the ten-tile packing chapter 08 shows is its Figure 1.
    pub const STROMQUIST: u32 = 2003;
    /// The case page's side for five tiles, 2 + √2/2, in the facts' form;
    /// Friedman's Table 1 writes it 2 + 1/√2.
    pub const SIDE: &str = "2 + 1/2*sqrt(2)";
    /// The case page's decimal, 2.70710678118654752440084436210485, to five
    /// places.
    pub const SIDE_5DP: &str = "2.70711";
    /// "A 3×3 grid would need side 3" (the case page).
    pub const GRID: &str = "3";
    /// Friedman's four points for five tiles (DS7, Theorem 2), in his order
    /// and the facts' form: 1/√2 = 1/2*sqrt(2).
    pub const FOUR_POINTS: &str = "(1, 1) (1, 1 + 1/2*sqrt(2)) (1 + 1/2*sqrt(2), 1) (1 + 1/2*sqrt(2), 1 + 1/2*sqrt(2))";
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
    corner(&mut f)?;
    strip(&mut f)?;
    middle(&mut f)?;
    scale(&mut f)?;
    beyond(&mut f)?;
    f.add("source.gobel", cited::GOBEL)?;
    f.add("source.stromquist", cited::STROMQUIST)?;
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

/// 04: the corner lemma (Friedman, DS7, Lemma 1). The point it guarantees;
/// the shadow's range and the proof's bound at its two ends; the lemma
/// checked exactly and completely at every checked turn, with its least
/// margin at an eighth of a turn; its edges (a straight tile on a wall: on
/// the edge; clear of the walls or turned: inside); two failures with an
/// assumption dropped; and the box of 2.71: Friedman's four points, the
/// centre region, the corner blocks settled and what is left.
fn corner(f: &mut Facts) -> Result<(), String> {
    let point: Point<Surd> = model::corner_point();
    f.add("corner.point", format!("{},{}", point.0, point.1))?;
    let straight = (Surd::int(1), Surd::ZERO);
    let eighth = model::eighth_turn();
    f.add("corner.shadow.min", model::shadow_width(straight))?;
    f.add("corner.shadow.max", model::shadow_width(eighth))?;
    f.add("corner.shadow.max.2dp", model::shadow_width(eighth).decimals(2))?;
    f.add("corner.bound.max", model::corner_bound(model::shadow_width(straight)))?;
    f.add("corner.bound.eighth", model::corner_bound(model::shadow_width(eighth)))?;
    let turns = model::corner_turns();
    let holds = turns.iter().all(|&t| model::corner_lemma_margin(t) >= Surd::ZERO);
    ensure(holds, || "the corner lemma fails at a checked turn".to_owned())?;
    f.add("corner.lemma", "yes")?;
    f.add("corner.turns", turns.len())?;
    f.add("corner.margin.eighth", model::corner_lemma_margin(eighth))?;
    let h = Surd::ratio(1, 2);
    let edges: Vec<&str> = [
        (h, h),
        (h, Surd::ratio(4, 5)),
        (Surd::ratio(4, 5), h),
        (Surd::ratio(3, 4), Surd::ratio(3, 4)),
    ]
    .iter()
    .map(|&c| model::corner_holds(c, straight).word())
    .collect();
    f.add("corner.edges", edges.join("|"))?;
    let turned_inside = turns
        .iter()
        .filter(|&&t| model::shadow_width(t) > Surd::int(1))
        .all(|&t| {
            model::corner_extremes(t)
                .iter()
                .all(|&c| model::corner_holds(c, t) == model::Holds::Inside)
        });
    f.add(
        "corner.turned",
        if turned_inside { "inside" } else { "not always inside" },
    )?;
    let past = (Surd::ratio(8, 5), h);
    let in_quadrant = model::in_quadrant(&model::Tile::straight(past.0, past.1));
    f.add(
        "corner.fail.centre",
        format!(
            "{}|{}",
            model::corner_holds(past, straight).word(),
            quadrant_word(in_quadrant)
        ),
    )?;
    let crossing = model::Tile {
        centre: (h, h),
        cos: eighth.0,
        sin: eighth.1,
    };
    f.add(
        "corner.fail.wall",
        format!(
            "{}|{}",
            model::corner_holds((h, h), eighth).word(),
            quadrant_word(model::in_quadrant(&crossing))
        ),
    )?;
    let side = model::turned_side();
    let pts: Vec<String> = model::corner_points(side)
        .iter()
        .map(|(x, y)| format!("({x}, {y})"))
        .collect();
    f.pin(
        "corner.box.points",
        pts.join(" "),
        cited::FOUR_POINTS,
        "Friedman DS7, Theorem 2: P = {(1,1), (1,1+1/√2), (1+1/√2,1), (1+1/√2,1+1/√2)}",
    )?;
    let (lo, hi) = model::centre_region(side);
    let region = (hi - lo) * (hi - lo);
    let block = Surd::int(1) - lo;
    let covered = Surd::int(4) * block * block;
    f.add("corner.box.region", region)?;
    f.add("corner.box.block", block)?;
    f.add("corner.box.covered", covered)?;
    f.add("corner.box.left", region - covered)?;
    f.add("corner.box.middle", side - Surd::int(2))?;
    Ok(())
}

/// 05: the side strip (Friedman, DS7, Lemma 2) with the points a gap apart,
/// one tile up: the points, two of the box's four; the assumptions kept;
/// the lemma checked exactly and completely at every checked turn; the slip
/// height at the two ends of the shadow's range, least straight (not at an
/// eighth of a turn, as Friedman's sketch says), and the floor cut at both
/// ends; the edges, the alternatives and 04's stray tile; the dodges, each
/// with one assumption dropped; and the box of 2.71 with the strips settled.
fn strip(f: &mut Facts) -> Result<(), String> {
    let (gap, one, h) = (model::turned_gap(), Surd::int(1), Surd::ratio(1, 2));
    let side = model::turned_side();
    let pts = model::strip_points(gap, one);
    let four = model::corner_points(side);
    ensure(pts == [four[0], four[2]], || {
        "the strip's points are not two of the four".to_owned()
    })?;
    let shown: Vec<String> = pts.iter().map(|(x, y)| format!("({x}, {y})")).collect();
    f.add("strip.points", shown.join(" "))?;
    f.add("strip.sum", gap + Surd::int(2) * one)?;
    let straight = (one, Surd::ZERO);
    let allowed = model::strip_broken(gap, one, &model::on_floor(one, straight)).is_empty();
    f.add("strip.assumed", if allowed { "yes" } else { "no" })?;
    let turns = model::corner_turns();
    let margins: Vec<Option<Surd>> = turns.iter().map(|&t| model::strip_margin(t, gap, one)).collect();
    ensure(margins.iter().all(Option::is_some), || {
        "a checked turn admits no centre one tile up".to_owned()
    })?;
    let holds = margins.iter().flatten().all(|&m| m >= Surd::ZERO);
    ensure(holds, || "the side strip lemma fails at a checked turn".to_owned())?;
    // the proof's step 4, for every turn at once: the points stand no higher
    // than the least slip height
    ensure(model::least_slip_height(gap) >= one, || {
        "the points stand above the least slip height".to_owned()
    })?;
    f.add("strip.lemma", "yes")?;
    f.add("strip.turns", turns.len())?;
    let eighth = model::eighth_turn();
    f.add("strip.height.straight", model::slip_height(one, gap))?;
    let high = model::slip_height(Surd::sqrt2(), gap);
    f.add("strip.height.eighth", high)?;
    f.add("strip.height.eighth.2dp", high.decimals(2))?;
    f.add("strip.least", model::least_slip_end(gap))?;
    f.add("strip.cut.straight", model::floor_cut(straight, one))?;
    let cut = model::floor_cut(eighth, one);
    f.add("strip.cut.eighth", cut)?;
    f.add("strip.cut.eighth.2dp", cut.decimals(2))?;
    let margin = model::strip_margin(eighth, gap, one).ok_or("an eighth admits no centre one tile up")?;
    f.add("strip.margin.eighth", margin)?;
    // the points a quarter up: an eighth admits no centre, so its short
    // floor cut is no dodge (the lemma holds there vacuously)
    let quarter = Surd::ratio(1, 4);
    let vacuous = match model::strip_margin(eighth, gap, quarter) {
        None => "no centre",
        Some(_) => "a centre",
    };
    f.add(
        "strip.vacuous",
        format!("{vacuous}|{}", model::floor_cut(eighth, quarter)),
    )?;
    let held = |t: &model::Tile<Surd>| model::strip_holds(t, gap, one).map(model::Holds::word).join(",");
    let row = |turn: Point<Surd>, across: &[Surd]| {
        across
            .iter()
            .map(|&a| held(&model::on_floor(a, turn)))
            .collect::<Vec<_>>()
            .join("|")
    };
    f.add("strip.edges", row(straight, &[one, one + gap * h, one + gap]))?;
    f.add(
        "strip.alternatives",
        row(eighth, &[Surd::ratio(11, 10), one + gap * h, Surd::ratio(9, 10) + gap]),
    )?;
    let turned_inside = turns.iter().filter(|&&t| model::shadow_width(t) > one).all(|&t| {
        let low = model::shadow_width(t) * h;
        [(one, low), (one + gap, low), (one, one), (one + gap, one)]
            .iter()
            .all(|&centre| {
                let tile = model::Tile {
                    centre,
                    cos: t.0,
                    sin: t.1,
                };
                model::strip_holds(&tile, gap, one).contains(&model::Holds::Inside)
            })
    });
    f.add(
        "strip.turned",
        if turned_inside { "inside" } else { "not always inside" },
    )?;
    f.add("strip.one", held(&model::Tile::straight(Surd::ratio(8, 5), h)))?;
    for d in model::strip_dodges() {
        let broken = model::strip_broken(d.x, d.y, &d.tile);
        ensure(broken == [d.breaks], || {
            format!("the dodge {} breaks {broken:?}, not {}", d.key, d.breaks)
        })?;
        let words = model::strip_holds(&d.tile, d.x, d.y).map(model::Holds::word);
        ensure(words == ["outside"; 2], || format!("the dodge {} holds a point", d.key))?;
        f.add(
            &format!("strip.fail.{}", d.key),
            format!(
                "{}|{}|{}",
                words.join(","),
                quadrant_word(model::in_quadrant(&d.tile)),
                broken.join(",")
            ),
        )?;
    }
    let (lo, hi) = model::centre_region(side);
    let region = (hi - lo) * (hi - lo);
    let block = one - lo;
    let strip_area = gap * block;
    let covered = Surd::int(4) * (block * block + strip_area);
    ensure(region - covered == gap * gap, || {
        "the strips and corner blocks do not leave the middle square".to_owned()
    })?;
    f.add("strip.box.strip", strip_area)?;
    f.add("strip.box.covered", covered)?;
    f.add("strip.box.left", region - covered)?;
    Ok(())
}

/// 06: the middle (Friedman, DS7, Lemma 3). The middle square cut along its
/// diagonal into two triangles, by corner (indices into the four points),
/// their squared sides and the diagonal's; the lemma checked exactly at
/// every checked turn and centre; the box's centre on their shared edge; the
/// eighth-turned tile there holding all four points on its edges, the
/// straight one holding them inside; the two ways to miss every corner, each
/// with its assumption dropped; where five fit, each point on two tiles'
/// edges; and nothing of the centre region left.
fn middle(f: &mut Facts) -> Result<(), String> {
    let side = model::turned_side();
    let (one, half) = (Surd::int(1), Surd::ratio(1, 2));
    let pts = model::corner_points(side);
    let tris = model::middle_triangles(side);
    let words = |h: &[model::Holds]| h.iter().map(|h| h.word()).collect::<Vec<_>>().join(",");
    let squares = |tri: &[Point<Surd>; 3]| {
        model::sides2(tri)
            .iter()
            .map(Surd::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };
    let corners: Vec<String> = model::MIDDLE_TRIANGLES
        .iter()
        .map(|tri| tri.map(|i| i.to_string()).join(","))
        .collect();
    f.add("middle.triangles", corners.join("|"))?;
    let short = tris.iter().all(|tri| model::sides2(tri).iter().all(|&s| s <= one));
    ensure(short, || "a middle triangle has a side longer than a tile".to_owned())?;
    f.add("middle.sides2", tris.iter().map(squares).collect::<Vec<_>>().join("|"))?;
    let (dx, dy) = (pts[3].0 - pts[0].0, pts[3].1 - pts[0].1);
    f.add("middle.diagonal2", dx * dx + dy * dy)?;
    let mut checks = 0usize;
    for tri in &tris {
        for centre in model::triangle_centres(tri) {
            ensure(model::triangle_holds(tri, centre) != model::Holds::Outside, || {
                format!("{centre:?} is not in its triangle")
            })?;
            for turn in model::corner_turns() {
                let t = model::Tile {
                    centre,
                    cos: turn.0,
                    sin: turn.1,
                };
                let how = model::meets(&t, tri);
                ensure(matches!(how, model::Meets::Holds(_)), || {
                    format!("the tile at {centre:?} turned {turn:?} misses its triangle: {how:?}")
                })?;
                checks += 1;
            }
        }
    }
    f.add("middle.lemma", "yes")?;
    f.add("middle.checks", checks)?;
    let turned_tile = model::five_turned().tiles[MIDDLE];
    f.add("middle.edge.holds", words(&model::held(&turned_tile, &pts)))?;
    let straight = model::Tile::straight(side * half, side * half);
    f.add("middle.straight.holds", words(&model::held(&straight, &pts)))?;
    let on: Vec<&str> = tris
        .iter()
        .map(|tri| model::triangle_holds(tri, turned_tile.centre).word())
        .collect();
    f.add("middle.centre.on", on.join("|"))?;
    let (big, tile) = model::middle_too_large();
    let tri = model::MIDDLE_TRIANGLES[0].map(|i| big[i]);
    // what each way of missing forces: the pair past opposite sides is more
    // than a tile apart; every corner beside a diagonal is strictly beyond
    // it, and the centre on it
    if let model::Meets::Apart(i, j, _) = model::meets(&tile, &tri) {
        let (dx, dy) = (tri[i].0 - tri[j].0, tri[i].1 - tri[j].1);
        ensure(dx * dx + dy * dy > one, || {
            "two corners past opposite sides are within a tile".to_owned()
        })?;
    }
    let out = model::centre_outside();
    if let model::Meets::Beside(k) = model::meets(&out, &tris[0]) {
        let beyond = tris[0].iter().all(|&p| model::beyond(&out, p, k) > Surd::ZERO);
        ensure(beyond && model::beyond(&out, out.centre, k) == Surd::ZERO, || {
            "the corners beside a diagonal are not all beyond it".to_owned()
        })?;
    }
    f.add(
        "middle.fail.large",
        format!(
            "{}|{}|{}",
            meets_word(model::meets(&tile, &tri)),
            squares(&tri),
            words(&model::held(&tile, &big))
        ),
    )?;
    f.add(
        "middle.fail.out",
        format!(
            "{}|{}|{}|{}",
            meets_word(model::meets(&out, &tris[0])),
            model::triangle_holds(&tris[0], out.centre).word(),
            words(&model::held(&out, &tris[0])),
            model::Holds::of(model::depth_inside(&out, pts[1])).word()
        ),
    )?;
    let packing = model::five_turned();
    let covers: Vec<String> = pts
        .iter()
        .map(|&p| {
            packing
                .tiles
                .iter()
                .enumerate()
                .filter(|(_, t)| model::depth_inside(t, p) >= Surd::ZERO)
                .map(|(i, _)| i.to_string())
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect();
    f.add("middle.packing.covers", covers.join("|"))?;
    let inside: Vec<String> = pts
        .iter()
        .enumerate()
        .filter(|&(_, &p)| packing.tiles.iter().any(|t| model::depth_inside(t, p) > Surd::ZERO))
        .map(|(k, _)| k.to_string())
        .collect();
    f.add(
        "middle.packing.inside",
        if inside.is_empty() {
            "none".to_owned()
        } else {
            inside.join(",")
        },
    )?;
    let (lo, hi) = model::centre_region(side);
    let (gap, block) = (model::turned_gap(), one - lo);
    let settled = Surd::int(4) * block * block + Surd::int(4) * gap * block + gap * gap;
    f.add("middle.box.left", (hi - lo) * (hi - lo) - settled)?;
    Ok(())
}

/// 07: four points, five tiles. The growth shown, λ = 6/5, and the narrower
/// box it supposes, B/λ; the shown core (a real unit tile in the box B
/// wide) and what it and its growth hold; the point on its edge, how deep it
/// is once grown (λ − 1)/2, at λ = 1 and below 1; that the grown tile fits
/// the box B and its original the box B/λ; the count; and where five fit,
/// the shared point's two tiles touching, overlapping once both grow.
fn scale(f: &mut Facts) -> Result<(), String> {
    let side = model::turned_side();
    let (one, half) = (Surd::int(1), Surd::ratio(1, 2));
    let lambda = Surd::ratio(model::GROWTH.0, model::GROWTH.1);
    let pts = model::corner_points(side);
    let words = |h: &[model::Holds]| h.iter().map(|h| h.word()).collect::<Vec<_>>().join(",");
    let in_box = |c: &[Point<Surd>; 4], s: Surd| {
        let walls = Surd::ZERO..=s;
        quadrant_word(c.iter().all(|q| walls.contains(&q.0) && walls.contains(&q.1)))
    };
    f.add("scale.lambda", lambda)?;
    let small_side = side * lambda.recip();
    f.add("scale.small", small_side)?;
    f.add("scale.small.2dp", small_side.decimals(2))?;
    let core = model::scale_core();
    f.add("scale.core.held", words(&model::held(&core, &pts)))?;
    let grown_held: Vec<model::Holds> = pts
        .iter()
        .map(|&p| model::Holds::of(model::depth_sized(&core, lambda, p)))
        .collect();
    f.add("scale.grown.held", words(&grown_held))?;
    let point = pts[0];
    let edge = model::depth_inside(&core, point);
    ensure(edge == Surd::ZERO, || {
        "the shown core's point is not on its edge".to_owned()
    })?;
    let margin = model::depth_sized(&core, lambda, point);
    ensure(margin == edge + (lambda - one) * half && margin > Surd::ZERO, || {
        "the grown depth is not (λ − 1)/2".to_owned()
    })?;
    f.add("scale.margin", margin)?;
    f.add("scale.margin.2dp", margin.decimals(2))?;
    let at = |l: Surd| {
        let d = model::depth_sized(&core, l, point);
        format!("{d}|{}", model::Holds::of(d).word())
    };
    f.add("scale.margin.at1", at(one))?;
    f.add("scale.margin.below", at(Surd::ratio(9, 10)))?;
    f.add("scale.grown.walls", in_box(&core.corners_sized(lambda), side))?;
    let small = model::grown(&core, lambda.recip());
    f.add("scale.small.walls", in_box(&small.corners(), small_side))?;
    ensure(model::grown(&small, lambda) == core, || {
        "the growth does not undo".to_owned()
    })?;
    let tiles = usize::try_from(TILES).map_err(|e| e.to_string())?;
    ensure(tiles > pts.len(), || "there are as many points as tiles".to_owned())?;
    f.add("scale.count", format!("{tiles}|{}", pts.len()))?;
    let five = model::five_turned();
    let (a, b) = (five.tiles[0], five.tiles[MIDDLE]);
    let relation = |c: Surd| match model::contact(c, Surd::ZERO) {
        model::Contact::Apart => "apart",
        model::Contact::Touching => "touch",
        model::Contact::Overlapping => "overlap",
    };
    f.add(
        "scale.shared",
        format!(
            "{}|{}",
            relation(model::clearance(&a, &b)),
            relation(model::clearance_sized(&a, &b, lambda))
        ),
    )?;
    Ok(())
}

/// 08: beyond five. Stromquist's ten in the box 3 + 1/√2 and the eleven in
/// the box 1939/500, each checked exactly (a fit, nothing about smaller
/// boxes): their sides, counts, turned tiles, the eleven's turn and its angle
/// as shown, the ten's turned tile's four touches; and the planted
/// failures, each caught.
fn beyond(f: &mut Facts) -> Result<(), String> {
    let ten = model::ten();
    let v = ten.check_exact();
    ensure(v.fits() && ten.tiles.len() == 10, || {
        format!("the ten do not fit: {}", v.summary())
    })?;
    f.add("beyond.ten.side", ten.side)?;
    f.add("beyond.ten.side.2dp", ten.side.decimals(2))?;
    f.add("beyond.ten.check", v.summary())?;
    f.add("beyond.ten.count", ten.tiles.len())?;
    let turned = |p: &Packing<Surd>| p.tiles.iter().filter(|t| t.sin != Surd::ZERO).count();
    f.add("beyond.ten.turned", turned(&ten))?;
    let last = ten.tiles.len() - 1;
    f.add(
        "beyond.ten.touches",
        v.touching.iter().filter(|&&(i, j)| i == last || j == last).count(),
    )?;
    let eleven = model::eleven();
    let v = eleven.check_exact();
    ensure(v.fits() && eleven.tiles.len() == 11, || {
        format!("the eleven do not fit: {}", v.summary())
    })?;
    f.add("beyond.eleven.side", eleven.side)?;
    f.add("beyond.eleven.side.3dp", eleven.side.decimals(3))?;
    f.add("beyond.eleven.side.2dp", eleven.side.decimals(2))?;
    f.add("beyond.eleven.check", v.summary())?;
    f.add("beyond.eleven.count", eleven.tiles.len())?;
    f.add("beyond.eleven.turned", turned(&eleven))?;
    let t = eleven.tiles[eleven.tiles.len() - 1];
    f.add("beyond.eleven.turn", format!("{},{}", t.cos, t.sin))?;
    let degrees = t.sin.to_f64().atan2(t.cos.to_f64()).to_degrees();
    f.add("beyond.eleven.degrees.2dp", format!("{degrees:.2}"))?;
    f.add("beyond.eleven.degrees.0dp", format!("{degrees:.0}"))?;
    for (key, p) in model::beyond_planted() {
        let v = p.check_exact();
        ensure(!v.fits(), || format!("the planted failure {key} passes the check"))?;
        f.add(&format!("beyond.fail.{key}"), v.summary())?;
    }
    Ok(())
}

/// How a tile meets a triangle, as the facts print it: `holds`, or the way
/// it misses every corner with the sides involved.
fn meets_word(m: model::Meets) -> String {
    match m {
        model::Meets::Holds(_) => "holds".to_owned(),
        model::Meets::Apart(_, _, k) => format!("apart:{},{}", model::SIDES[k], model::SIDES[(k + 2) % 4]),
        model::Meets::Beside(k) => format!("beside:{},{}", model::SIDES[k], model::SIDES[(k + 1) % 4]),
    }
}

/// Whether a tile keeps inside the walls, as the facts print it.
fn quadrant_word(inside: bool) -> &'static str {
    if inside { "inside the walls" } else { "through a wall" }
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

    /// What chapter 03's voice says, on the numbers the model computes: the
    /// held corners half a tile out, on the inner circle; the lens's
    /// magnification; a doubled turn almost four times deeper, not four; the
    /// overlap both ways, never zero; the free turn in the box of three.
    #[test]
    fn the_turn_by_a_hair_says_the_facts() {
        assert_eq!(value("hair.inner2"), "1/4");
        assert_eq!(value("hair.inradius2"), value("hair.inner2"));
        assert!(line("03_2").contains("half a tile from that centre"));
        let mag = int("hair.lens.mag");
        let tens = [
            "", "ten", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
        ];
        let said = match mag % 10 {
            0 => tens[(mag / 10) as usize].to_owned(),
            ones => format!("{} {}", tens[(mag / 10) as usize], word(ones)),
        };
        let l5 = line("03_5");
        assert!(l5.contains(&format!("magnified {said} times")), "{l5}");
        let ratio: f64 = value("hair.ratio.2dp").parse().unwrap();
        assert!(ratio < 4.0 && (ratio.round() - 4.0).abs() < 1e-9, "{ratio}");
        assert!(l5.contains("almost four times deeper"), "{l5}");
        assert_eq!(value("hair.check"), "overlap:0-4,1-4,2-4,3-4");
        assert_eq!(value("hair.check.minus"), value("hair.check"));
        assert!(line("03_6").contains("never zero"));
        assert!(value("hair.tiny").split(',').all(|d| d.starts_with("1/")));
        assert_eq!(value("hair.roomy"), "free|free");
        assert!(line("03_7").contains("turned freely"));
        assert!(line("03_8").contains("only that this one turn is blocked"));
    }

    /// What chapter 04's voice says, on the facts: the point one tile
    /// across and one tile up, centres within one tile of both walls; the
    /// bound never past one half, the reach half a tile; only a straight tile
    /// on a wall touches; the point can escape without an assumption; the
    /// box of 2.71 and what is left.
    #[test]
    fn the_corner_lemma_says_the_facts() {
        assert_eq!(value("corner.point"), "1,1");
        let l2 = line("04_2");
        assert!(l2.contains("within one tile of both walls"), "{l2}");
        assert!(l2.contains("one tile across and one tile up"), "{l2}");
        assert_eq!(value("corner.lemma"), "yes");
        assert_eq!(value("corner.bound.max"), "1/2");
        assert!(line("04_5").contains("never passes one half"));
        assert_eq!(value("hair.inradius2"), "1/4");
        let l6 = line("04_6");
        assert!(l6.contains("Half a tile is how far a tile reaches"), "{l6}");
        assert!(
            l6.contains("Only a straight tile resting on a wall just touches it"),
            "{l6}"
        );
        assert_eq!(value("corner.edges"), "edge|edge|edge|inside");
        assert_eq!(value("corner.turned"), "inside");
        assert!(line("04_7").contains("the point can escape"));
        assert!(value("corner.fail.centre").starts_with("outside|inside the walls"));
        assert!(value("corner.fail.wall").starts_with("outside|through a wall"));
        let l8 = line("04_8");
        assert!(
            l8.contains(&format!("box of {}", spoken(&value("turned.side.2dp")))),
            "{l8}"
        );
        assert_eq!(value("corner.box.covered"), "1");
        assert_eq!(value("corner.box.left"), "1/2 + sqrt(2)");
    }

    /// What chapter 05's voice says, on the facts: past a corner block one
    /// point is not enough (04's stray tile), so two, one tile up; the lemma;
    /// the reach; least at the straight end, exactly one, and higher at an
    /// eighth; the points at one; three dropped assumptions, each a dodge;
    /// the strips settled and the middle left.
    #[test]
    fn near_a_side_says_the_facts() {
        assert_eq!(value("strip.one"), "outside,edge");
        let l1 = line("05_1");
        assert!(l1.contains("one point is not enough, so take two"), "{l1}");
        assert!(l1.contains("within a tile of the floor, covers one"), "{l1}");
        assert_eq!(value("strip.points"), "(1, 1) (1 + 1/2*sqrt(2), 1)");
        assert_eq!(value("strip.assumed"), "yes");
        assert_eq!(value("strip.lemma"), "yes");
        assert!(line("05_3").contains("at most half a tile above its centre"));
        assert!(line("05_4").contains("Lower is narrower"));
        let l5 = line("05_5");
        assert!(l5.contains("one shadow tall"), "{l5}");
        assert!(l5.contains("two right triangles"), "{l5}");
        let l6 = line("05_6");
        assert!(l6.contains("lowest at an end, exactly one when straight"), "{l6}");
        assert_eq!(value("strip.least"), "straight");
        assert_eq!(value("strip.height.straight"), "1");
        assert!(value("strip.height.eighth.2dp").parse::<f64>().unwrap() > 1.0);
        assert!(l6.contains("The points stand at one"), "{l6}");
        assert!(
            value("strip.points")
                .split(") (")
                .all(|p| p.trim_end_matches(')').ends_with(", 1"))
        );
        let l7 = line("05_7");
        assert!(
            l7.contains("Drop the floor, raise the points, or widen the gap"),
            "{l7}"
        );
        for key in ["floor", "high", "wide", "past"] {
            assert!(
                value(&format!("strip.fail.{key}")).starts_with("outside,outside|"),
                "{key}"
            );
        }
        assert!(line("05_8").contains("Only the middle is left"));
        assert_eq!(value("strip.box.left"), "1/2");
        assert_eq!(value("strip.box.covered"), "1 + sqrt(2)");
        assert_eq!(value("strip.vacuous"), "no centre|1/2");
    }

    /// What chapter 06's voice says, on the facts: the diagonal one tile
    /// long and every side at most one; the lemma; a long side parting two
    /// corners past opposite sides; the centre outside leaving the corners
    /// beside a diagonal; covering perhaps only on the edge; every centre
    /// settled; where five fit, each point on two tiles' edges and inside
    /// none.
    #[test]
    fn a_triangle_of_points_says_the_facts() {
        assert_eq!(value("middle.diagonal2"), "1");
        assert!(line("06_1").contains("its diagonal, one tile long"));
        assert!(value("middle.sides2").split(['|', ',']).all(|s| s == "1" || s == "1/2"));
        assert!(line("06_2").contains("sides are at most one"));
        assert_eq!(value("middle.lemma"), "yes");
        let large = value("middle.fail.large");
        assert!(large.starts_with("apart:left,right|9/16,9/16,9/8|"), "{large}");
        assert!(line("06_4").contains("over a tile apart"));
        assert!(line("06_4").contains("a side longer than one"));
        let out = value("middle.fail.out");
        assert!(out.starts_with("beside:"), "{out}");
        assert!(out.contains("|outside|outside,outside,outside|"), "{out}");
        let l5 = line("06_5");
        assert!(l5.contains("past two neighbouring sides only"), "{l5}");
        assert!(l5.contains("past one of the other two"), "{l5}");
        assert!(l5.contains("the triangle misses the centre"), "{l5}");
        assert_eq!(value("middle.edge.holds"), "edge,edge,edge,edge");
        assert!(line("06_6").contains("perhaps only on its edge"));
        assert_eq!(value("middle.box.left"), "0");
        let l7 = line("06_7");
        assert!(l7.contains("every tile in the box covers a point"), "{l7}");
        assert!(
            value("middle.packing.covers")
                .split('|')
                .all(|c| c.split(',').count() == 2)
        );
        assert_eq!(value("middle.packing.inside"), "none");
        assert!(l7.contains("each point sits on the edges of two tiles"), "{l7}");
    }

    /// What chapter 07's voice says, on the facts: our box B wide exactly,
    /// and a narrower one supposed; never a rounded cutoff, since B is about
    /// 2.70711, below 2.71; the shown tile in the supposed box and grown;
    /// the unit core covering a point on its edge; that point strictly inside
    /// the grown tile by (λ − 1)/2, on the edge at λ = 1 and lost below it;
    /// five tiles against four points, a shared point overlapping once
    /// grown; the turned packing fitting, its points shared on edges.
    #[test]
    fn four_points_five_tiles_says_the_facts() {
        assert_eq!(value("turned.side"), "2 + 1/2*sqrt(2)");
        assert_eq!(value("turned.gap"), "1/2*sqrt(2)");
        let l1 = line("07_1");
        assert!(
            l1.contains("exactly two plus one over the square root of two wide"),
            "{l1}"
        );
        assert!(l1.contains("Suppose five tiles fit a narrower one"), "{l1}");
        assert!(
            value("turned.side.5dp").parse::<f64>().unwrap() < 2.71,
            "rounding up would be wrong"
        );
        for id in ["07_1", "07_2", "07_4", "07_7"] {
            assert!(!line(id).contains(&spoken("2.71")), "{id} speaks a rounded cutoff");
        }
        assert!(line("07_2").contains("as wide as ours"));
        assert!(line("07_4").contains("It lies in our box"));
        assert!(value("scale.small.2dp").parse::<f64>().unwrap() < value("turned.side.5dp").parse::<f64>().unwrap());
        assert_eq!(value("scale.small.walls"), "inside the walls");
        assert_eq!(value("scale.grown.walls"), "inside the walls");
        assert!(line("07_2").contains("Each tile grows by the same factor"));
        assert!(line("07_3").contains("keep a unit tile with the same centre"));
        assert!(value("scale.core.held").split(',').any(|h| h == "edge"));
        assert!(line("07_4").contains("at worst on its edge"));
        let lambda: Vec<f64> = value("scale.lambda").split('/').map(|n| n.parse().unwrap()).collect();
        let margin: Vec<f64> = value("scale.margin").split('/').map(|n| n.parse().unwrap()).collect();
        assert!((margin[0] / margin[1] - (lambda[0] / lambda[1] - 1.0) / 2.0).abs() < 1e-12);
        assert!(line("07_5").contains("strictly inside it, by at least half the growth"));
        assert_eq!(value("scale.margin.at1"), "0|edge");
        assert!(value("scale.margin.below").ends_with("|outside"));
        let count = value("scale.count");
        let (tiles, points) = count.split_once('|').unwrap();
        let (tiles, points): (u32, u32) = (tiles.parse().unwrap(), points.parse().unwrap());
        assert!(tiles > points);
        let l6 = line("07_6");
        assert!(l6.starts_with(&format!("{} tiles", capital(word(tiles)))), "{l6}");
        assert!(
            l6.contains(&format!("{} points are not enough", capital(word(points)))),
            "{l6}"
        );
        assert_eq!(value("scale.shared"), "touch|overlap");
        assert!(l6.contains("no two can share one without overlapping"), "{l6}");
        assert_eq!(value("turned.check"), "fits");
        assert!(line("07_7").contains("the turned packing fits this one"));
        assert_eq!(value("middle.packing.inside"), "none");
        assert!(line("07_8").contains("shared by two tiles"));
    }

    /// What chapter 08's voice says, on the facts: ten in a box 3 + 1/√2,
    /// nine straight and one turned; eleven in a box about 3.88, five turned
    /// about 40° (39.97°, not 40); each checked exactly and fitting, with
    /// every planted failure caught; and the five's box exactly 2 + 1/√2.
    #[test]
    fn what_is_proved_says_the_facts() {
        assert_eq!(value("beyond.ten.side"), "3 + 1/2*sqrt(2)");
        let ten = int("beyond.ten.count");
        let l1 = line("08_1");
        let opening = format!(
            "{} tiles fit a box three plus one over the square root of two wide",
            capital(word(ten))
        );
        assert!(l1.starts_with(&opening), "{l1}");
        let turned = int("beyond.ten.turned");
        let nine_and_one = format!("{} straight and {} turned", word(ten - turned), word(turned));
        assert!(l1.contains(&nine_and_one), "{l1}");
        let l2 = line("08_2");
        assert_eq!(int("beyond.eleven.count"), 11);
        assert!(l2.starts_with("Eleven fit"), "{l2}");
        let side = format!("about {} wide", spoken(&value("beyond.eleven.side.2dp")));
        assert!(l2.contains(&side), "{l2}");
        let five = format!("{} of them turned", word(int("beyond.eleven.turned")));
        assert!(l2.contains(&five), "{l2}");
        assert_eq!(value("beyond.eleven.degrees.0dp"), "40");
        assert_ne!(value("beyond.eleven.degrees.2dp"), "40.00");
        assert!(l2.contains("about forty degrees"), "{l2}");
        assert_eq!(value("beyond.ten.check"), "fits");
        assert_eq!(value("beyond.eleven.check"), "fits");
        for key in ["ten_narrower", "eleven_narrower", "eleven_doubled"] {
            assert_ne!(value(&format!("beyond.fail.{key}")), "fits", "{key}");
        }
        let l3 = line("08_3");
        assert!(
            l3.contains("every tile is a true unit square, none overlap, and all are inside"),
            "{l3}"
        );
        assert!(l3.contains("big enough, nothing more"), "{l3}");
        assert_eq!(value("turned.side"), "2 + 1/2*sqrt(2)");
        assert!(line("08_5").contains("exactly two plus one over the square root of two"));
    }
}
