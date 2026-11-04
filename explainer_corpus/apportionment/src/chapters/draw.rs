//! The film's look, shared by its chapters (storyboard, "The look").
//!
//! A town is a row: its name at the left, then a seat grid (one cell per
//! seat, [`CELL`] wide), a thin share band along the bottom of the grid (the
//! whole part in the town's color, the tail in hero yellow), and seat tokens
//! in the cells. Seat `k` (0-based) sits in the cell from `seat_x(k)` to
//! `seat_x(k + 1)`, so the right edge of a town's last token is at
//! `seat_x(seats)`, and the quota box for the share rounded down and up
//! spans `seat_x(floor)` to `seat_x(ceil)`: a town keeps the quota promise
//! exactly when its last token ends inside its box.
//!
//! Live pictures are driven by ONE tracker each: the house size (shares,
//! bands, a dial: each dial has its own, so one hand can move while the
//! others wait) or the yardstick (quotients). A tracker's value is the
//! exact quantity the narration names; every label at a resting value is
//! formatted from `crate::model`, never typed.
//!
//! Builders return detached mobjects (in the arena, not yet in the scene),
//! like `Kit::text`: bring them in with an animation. Engine rules: one
//! flat fill per mobject, no outline on filled shapes, short strokes.
//! Playback, line timing and the think ring follow german_tank's accepted
//! helpers.

use explainer_kit::narration::{frames_covering, wait_frames};
use explainer_kit::{Kit, Palette, Script, Timed, rect, v};
use fmn::prelude::*;

/// A scene error carrying `message` (a missing phrase, a broken invariant).
pub fn fail(message: String) -> fmn::Error {
    fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig(Box::leak(
        message.into_boxed_str(),
    )))
}

/// A town's color: Ash blue, Birch gold, Cedar green; the four- and
/// five-town maps continue with purple and teal.
pub fn town_color(i: usize) -> Srgb {
    match i {
        0 => Palette::blue(),
        1 => Palette::gold(),
        2 => Palette::green(),
        3 => Palette::purple(),
        _ => Palette::teal(),
    }
}

// ------------------------------------------------------------------ rows

/// The rows' centres, Ash at the top.
pub const ROW_Y: [f64; 3] = [1.5, 0.0, -1.5];
/// The names' left edge, the grid's left edge (seat 0) and one cell.
pub const NAME_X: f64 = -6.5;
pub const GRID_X: f64 = -4.5;
pub const CELL: f64 = 0.42;
/// A grid cell's height; the share band's centre below the row centre and
/// its height; a token's side and its centre above the row centre.
pub const ROW_H: f64 = 0.86;
pub const BAND_DY: f64 = -0.3;
pub const BAND_H: f64 = 0.14;
pub const TOKEN: f64 = 0.3;
pub const TOKEN_DY: f64 = 0.07;
/// Readouts' left edge: right of a 13-cell grid.
pub const READ_X: f64 = 1.15;
pub const NAME_SIZE: f64 = 26.0;
pub const READ_SIZE: f64 = 24.0;

/// The left edge of cell `k` (also: the right edge of `k` seats).
pub fn seat_x(k: f64) -> f64 {
    GRID_X + k * CELL
}

/// A flat fill and no outline.
pub fn flat(c: Srgb, o: f64) -> Style {
    Style::default().fill(c, o).stroke(c, 0.0, 0.0)
}

/// A stroke and no fill, at opacity `o`.
pub fn line_style(c: Srgb, w: f64, o: f64) -> Style {
    Style::default().fill(c, 0.0).stroke(c, w, o)
}

/// A stroke through `points`, detached.
pub fn stroke(stage: &mut Stage<'_>, points: Vec<Vec3>, style: Style) -> Mob {
    stage.arena_mut().add(polyline(points).style(style).build())
}

/// A town's name, left aligned at the row's height, and, unless `label` is
/// empty, a second line under it (its population).
pub fn name(kit: &Kit, stage: &mut Stage<'_>, i: usize, y: f64, label: &str) -> fmn::Result<Vec<Mob>> {
    if label.is_empty() {
        return Ok(vec![kit.text_left(
            stage,
            crate::model::NAMES[i],
            NAME_SIZE,
            town_color(i),
            v(NAME_X, y),
        )?]);
    }
    let n = kit.text_left(
        stage,
        crate::model::NAMES[i],
        NAME_SIZE,
        town_color(i),
        v(NAME_X, y + 0.14),
    )?;
    let p = kit.text_left(stage, label, 18.0, Palette::dim(), v(NAME_X, y - 0.24))?;
    Ok(vec![n, p])
}

/// The faint outlines of `cells` grid cells at row height `y`, detached.
pub fn grid(stage: &mut Stage<'_>, y: f64, cells: usize) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for k in 0..cells {
        let r = Rectangle::new()
            .width(CELL)
            .height(ROW_H)
            .style(line_style(Palette::faint(), 1.5, 0.6))
            .build()?
            .moved_to(v(seat_x(k as f64 + 0.5), y));
        out.push(stage.arena_mut().add(r));
    }
    Ok(out)
}

/// The two pieces of a share band for quotient `q`: (whole, tail) as
/// (left x, width) pairs.
pub fn band_spans(q: f64) -> [(f64, f64); 2] {
    let whole = (q + 1e-9).floor();
    let tail = (q - whole).max(0.0);
    [
        (seat_x(0.0), (whole * CELL).max(0.004)),
        (seat_x(whole), (tail * CELL).max(0.004)),
    ]
}

/// A static share band for quotient `q` in row `y`: [whole, tail], detached.
pub fn band(stage: &mut Stage<'_>, y: f64, q: f64, color: Srgb) -> fmn::Result<[Mob; 2]> {
    let [(x0, w0), (x1, w1)] = band_spans(q);
    let bottom = y + BAND_DY - BAND_H / 2.0;
    let whole = rect(v(x0, bottom), w0, BAND_H, flat(color, 0.9))?;
    let tail = rect(v(x1, bottom), w1, BAND_H, flat(Palette::hero(), 0.9))?;
    Ok([stage.arena_mut().add(whole), stage.arena_mut().add(tail)])
}

/// A live share band whose quotient is `q(tracker value)`, redrawn when the
/// tracker moves; added to the scene now. [whole, tail].
pub fn live_band(
    stage: &mut Stage<'_>,
    y: f64,
    color: Srgb,
    tracker: Mob,
    q: impl Fn(f64) -> f64 + Copy + 'static,
) -> fmn::Result<[Mob; 2]> {
    let bottom = y + BAND_DY - BAND_H / 2.0;
    let mut out: Vec<Mob> = Vec::new();
    for (piece, fill) in [(0_usize, color), (1, Palette::hero())] {
        let m = stage.always_redraw(move |s| {
            let value = q(s.tracker_value(tracker).unwrap_or(0.0));
            let (x, w) = band_spans(value)[piece];
            let r = Rectangle::new()
                .width(w)
                .height(BAND_H)
                .style(flat(fill, 0.9))
                .build()
                .expect("positive extents");
            let m = s.add(r);
            s.move_to(m, v(x, bottom + BAND_H / 2.0), LEFT);
            m
        });
        stage.add_to_scene(m)?;
        out.push(m);
    }
    Ok([out[0], out[1]])
}

/// A seat token centred in cell `k` of row `y`, detached.
pub fn token(stage: &mut Stage<'_>, y: f64, k: usize, color: Srgb) -> fmn::Result<Mob> {
    token_at(stage, v(seat_x(k as f64 + 0.5), y + TOKEN_DY), color)
}

/// A seat token centred at `at`, detached.
pub fn token_at(stage: &mut Stage<'_>, at: Vec3, color: Srgb) -> fmn::Result<Mob> {
    let r = Rectangle::new()
        .width(TOKEN)
        .height(TOKEN)
        .corner_radius(0.06)
        .style(flat(color, 0.95))
        .build()?
        .moved_to(at);
    Ok(stage.arena_mut().add(r))
}

/// Tokens for cells `from..to` of row `y`, detached.
pub fn tokens(stage: &mut Stage<'_>, y: f64, from: usize, to: usize, color: Srgb) -> fmn::Result<Vec<Mob>> {
    (from..to).map(|k| token(stage, y, k, color)).collect()
}

/// Where a token in cell `k` of row `y` sits.
pub fn token_spot(y: f64, k: usize) -> Vec3 {
    v(seat_x(k as f64 + 0.5), y + TOKEN_DY)
}

/// The quota box of row `y`: from `lo` seats to `hi` seats (share rounded
/// down and up), a little taller than the row, detached.
pub fn quota_box(stage: &mut Stage<'_>, y: f64, lo: u128, hi: u128, color: Srgb) -> fmn::Result<Mob> {
    let (x0, x1) = (seat_x(lo as f64), seat_x(hi as f64));
    let r = Rectangle::new()
        .width(x1 - x0 + 0.14)
        .height(ROW_H + 0.12)
        .corner_radius(0.05)
        .style(line_style(color, 3.0, 0.95))
        .build()?
        .moved_to(v((x0 + x1) / 2.0, y));
    Ok(stage.arena_mut().add(r))
}

/// A readout at the right of row `y`, detached.
pub fn readout(kit: &Kit, stage: &mut Stage<'_>, s: &str, y: f64, color: Srgb) -> fmn::Result<Mob> {
    kit.text_left(stage, s, READ_SIZE, color, v(READ_X, y))
}

/// A check mark (a short stroke), detached.
pub fn check(stage: &mut Stage<'_>, at: Vec3, size: f64, color: Srgb) -> Mob {
    let p = |x: f64, y: f64| v(at[0] + x * size, at[1] + y * size);
    stroke(
        stage,
        vec![p(-0.5, 0.0), p(-0.15, -0.38), p(0.55, 0.45)],
        line_style(color, 6.0, 1.0),
    )
}

/// A cross (two short strokes), detached.
pub fn cross(stage: &mut Stage<'_>, at: Vec3, size: f64, color: Srgb) -> [Mob; 2] {
    let p = |x: f64, y: f64| v(at[0] + x * size, at[1] + y * size);
    [
        stroke(stage, vec![p(-0.4, -0.4), p(0.4, 0.4)], line_style(color, 6.0, 1.0)),
        stroke(stage, vec![p(-0.4, 0.4), p(0.4, -0.4)], line_style(color, 6.0, 1.0)),
    ]
}

/// The standard three rows at house size `house` (Hamilton chapters): names
/// with populations, grids of `cells`, static share bands. Detached.
pub struct Rows {
    pub names: Vec<Mob>,
    pub grids: Vec<Mob>,
    pub bands: Vec<[Mob; 2]>,
}

impl Rows {
    pub fn build(kit: &Kit, stage: &mut Stage<'_>, pops: &[u64; 3], house: u64, cells: usize) -> fmn::Result<Self> {
        let mut rows = Rows {
            names: Vec::new(),
            grids: Vec::new(),
            bands: Vec::new(),
        };
        for i in 0..3 {
            let label = format!("{} people", crate::model::grouped(u128::from(pops[i])));
            rows.names.extend(name(kit, stage, i, ROW_Y[i], &label)?);
            rows.grids.extend(grid(stage, ROW_Y[i], cells)?);
            let q = crate::model::share(pops, i, house).to_f64();
            rows.bands.push(band(stage, ROW_Y[i], q, town_color(i))?);
        }
        Ok(rows)
    }

    pub fn all(&self) -> Vec<Mob> {
        let mut out = self.names.clone();
        out.extend(&self.grids);
        for b in &self.bands {
            out.extend(b);
        }
        out
    }
}

/// Share readouts ("1.54") for the three towns at `house`, detached.
pub fn share_readouts(kit: &Kit, stage: &mut Stage<'_>, pops: &[u64; 3], house: u64) -> fmn::Result<Vec<Mob>> {
    (0..3)
        .map(|i| {
            let s = crate::model::decimal(crate::model::share(pops, i, house), 2);
            readout(kit, stage, &s, ROW_Y[i], town_color(i))
        })
        .collect()
}

// ------------------------------------------------------------------ dials

/// A dial's radius, its hand's length and its arc's width.
pub const DIAL_R: f64 = 0.82;
pub const HAND: f64 = 0.74;
pub const ARC_W: f64 = 9.0;

/// The point on a dial at `at` a share `turns` of a full turn clockwise
/// from the top, at radius `r`.
pub fn dial_point(at: Vec3, turns: f64, r: f64) -> Vec3 {
    let a = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * turns;
    v(at[0] + r * a.cos(), at[1] + r * a.sin())
}

/// The arc from the top of a dial clockwise through `turns` of a turn.
pub fn arc_points(at: Vec3, from: f64, turns: f64, r: f64) -> Vec<Vec3> {
    let t = turns.clamp(0.002, 1.0);
    let n = (96.0 * t).ceil() as usize + 2;
    (0..n)
        .map(|j| dial_point(at, from + t * j as f64 / (n - 1) as f64, r))
        .collect()
}

/// A town's dial at `at`: the share `house × pop / total` as a hand whose
/// angle is its tail (one turn = one seat), the swept tail as a thick arc
/// in hero yellow, and the whole part as a live number under the dial.
/// Driven by the `house` tracker, whose value now is `h0`. Returns the
/// mobjects to fade in (ring, top tick, arc, hand, anchor); the live whole
/// part is already on stage.
#[allow(clippy::too_many_arguments)]
pub fn dial(
    kit: &Kit,
    stage: &mut Stage<'_>,
    at: Vec3,
    pop: u64,
    total: u64,
    house: Mob,
    h0: f64,
    color: Srgb,
) -> fmn::Result<Vec<Mob>> {
    let speed = pop as f64 / total as f64;
    let tail = move |h: f64| {
        let q = h * speed;
        (q - (q + 1e-9).floor()).max(0.0)
    };
    let ring = Circle::new()
        .radius(DIAL_R)
        .style(line_style(color, 3.0, 0.8))
        .build()
        .moved_to(at);
    let ring = stage.arena_mut().add(ring);
    let tick = stroke(
        stage,
        vec![dial_point(at, 0.0, DIAL_R - 0.1), dial_point(at, 0.0, DIAL_R + 0.12)],
        line_style(Palette::ink(), 3.0, 0.9),
    );
    let arc = stroke(
        stage,
        arc_points(at, 0.0, tail(h0), DIAL_R),
        line_style(Palette::hero(), ARC_W, 0.85),
    );
    let hand = stroke(
        stage,
        vec![at, dial_point(at, tail(h0), HAND)],
        line_style(color, 6.0, 1.0),
    );
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let h = s.tracker_value(house).unwrap_or(0.0);
            if last == Some(h) {
                return;
            }
            let t = tail(h);
            let _ = s.set_points(hand, polyline(vec![at, dial_point(at, t, HAND)]).build().points());
            let _ = s.set_points(arc, polyline(arc_points(at, 0.0, t, DIAL_R)).build().points());
            last = Some(h);
        },
        true,
    )?;
    kit.live_value(
        stage,
        move |s| (s.tracker_value(house).unwrap_or(0.0) * speed + 1e-9).floor(),
        0,
        false,
        30.0,
        color,
        v(at[0], at[1] - DIAL_R - 0.42),
        ORIGIN,
    )?;
    Ok(vec![ring, tick, arc, hand, anchor])
}

/// A faint wedge ahead of a hand: from `from` turns through `step` turns.
pub fn step_arc(stage: &mut Stage<'_>, at: Vec3, from: f64, step: f64, color: Srgb) -> Mob {
    stroke(
        stage,
        arc_points(at, from, step, DIAL_R + 0.16),
        line_style(color, 5.0, 0.55),
    )
}

// Historical three-dial centres (Ash, Birch, Cedar), now unused.
// Retired shared dial layout; each independent dial now owns its position.
// pub const DIALS: [[f64; 2]; 3] = [[2.3, 0.3], [4.35, 0.3], [6.4, 0.3]];

// -------------------------------------------------------------- signposts

/// The signpost list on the right: card centres, size and spacing.
pub const LIST_X: f64 = 4.45;
pub const LIST_TOP: f64 = 3.4;
pub const LIST_STEP: f64 = 0.338;
pub const CARD_W: f64 = 4.2;
pub const CARD_H: f64 = 0.29;
pub const CARD_SIZE: f64 = 17.0;

/// The height of the card ranked `rank` (1 at the top).
pub fn card_y(rank: usize) -> f64 {
    LIST_TOP - (rank as f64 - 1.0) * LIST_STEP
}

/// A signpost card: a faint band in the town's color, the rank, "Birch
/// seat 4" and the yardstick at which that seat opens, detached.
pub fn card(
    kit: &Kit,
    stage: &mut Stage<'_>,
    rank: usize,
    town: usize,
    seat: u64,
    value: &str,
    y: f64,
) -> fmn::Result<Vec<Mob>> {
    let mut body = card_body(kit, stage, town, seat, value, v(LIST_X, y), 0.5)?;
    let r = kit.text(
        stage,
        &rank.to_string(),
        CARD_SIZE - 2.0,
        Palette::dim(),
        v(LIST_X - CARD_W / 2.0 + 0.22, y),
    )?;
    body.insert(1, r);
    Ok(body)
}

/// A card without a rank, centred at `at`, its label `label_dx` in from the
/// left edge: [background, "Birch seat 4", value], detached.
pub fn card_body(
    kit: &Kit,
    stage: &mut Stage<'_>,
    town: usize,
    seat: u64,
    value: &str,
    at: Vec3,
    label_dx: f64,
) -> fmn::Result<Vec<Mob>> {
    let bg = Rectangle::new()
        .width(CARD_W)
        .height(CARD_H)
        .corner_radius(0.05)
        .style(flat(town_color(town), 0.2))
        .build()?
        .moved_to(at);
    let bg = stage.arena_mut().add(bg);
    let label = format!("{} seat {seat}", crate::model::NAMES[town]);
    let n = kit.text_left(
        stage,
        &label,
        CARD_SIZE,
        town_color(town),
        v(at[0] - CARD_W / 2.0 + label_dx, at[1]),
    )?;
    let val = value_text(kit, stage, value, at)?;
    Ok(vec![bg, n, val])
}

/// A card's value, right-aligned inside a card centred at `at`, detached.
pub fn value_text(kit: &Kit, stage: &mut Stage<'_>, value: &str, at: Vec3) -> fmn::Result<Mob> {
    let val = kit.text(stage, value, CARD_SIZE, Palette::ink(), at)?;
    stage.move_to(val, v(at[0] + CARD_W / 2.0 - 0.12, at[1]), RIGHT);
    Ok(val)
}

/// The first `count` Jefferson signposts of `pops` as cards in rank order.
pub fn signpost_cards(kit: &Kit, stage: &mut Stage<'_>, pops: &[u64], count: usize) -> fmn::Result<Vec<Vec<Mob>>> {
    rule_cards(kit, stage, crate::model::Rule::Jefferson, pops, count)
}

/// The first `count` signposts of a rule as cards in rank order. Hill's
/// signposts are squared in `model`, so Hill has no cards.
pub fn rule_cards(
    kit: &Kit,
    stage: &mut Stage<'_>,
    rule: crate::model::Rule,
    pops: &[u64],
    count: usize,
) -> fmn::Result<Vec<Vec<Mob>>> {
    if rule == crate::model::Rule::Hill {
        return Err(fail("Hill signposts are squared; they have no cards".into()));
    }
    let list = crate::model::signposts(rule, pops, count);
    let mut out = Vec::new();
    for (j, (town, seat, value)) in list.iter().enumerate() {
        out.push(card(
            kit,
            stage,
            j + 1,
            *town,
            *seat,
            &crate::model::people(*value),
            card_y(j + 1),
        )?);
    }
    Ok(out)
}

/// A card's identity while the list moves: (town, seat number).
pub type CardKey = (usize, u64);

/// A card in a list that moves: its identity, the value it shows and its
/// parts [background, label, value]. Its rank is not part of it: ranks stay
/// printed in their own column ([`rank_column`]) while cards move.
pub struct ListCard {
    pub key: CardKey,
    pub value: String,
    pub mobs: Vec<Mob>,
}

/// The rank numbers' column, between the lane and the list.
pub const RANK_X: f64 = LIST_X - CARD_W / 2.0 - 0.2;
/// Where a card whose rank changes travels: a card's width and a gap left of
/// the list, past the rank column, so it never crosses a card in the list.
pub const LANE_X: f64 = LIST_X - CARD_W - 0.45;
/// A listed card's label inset (the rank is outside the card).
const LIST_LABEL_DX: f64 = 0.2;

/// The fixed rank numbers 1 to `count` beside the list, detached.
pub fn rank_column(kit: &Kit, stage: &mut Stage<'_>, count: usize) -> fmn::Result<Vec<Mob>> {
    (1..=count)
        .map(|r| {
            kit.text(
                stage,
                &r.to_string(),
                CARD_SIZE - 2.0,
                Palette::dim(),
                v(RANK_X, card_y(r)),
            )
        })
        .collect()
}

/// The first `count` signposts of a rule as keyed cards in rank order,
/// detached, without ranks. Hill's signposts are squared: no cards.
pub fn keyed_cards(
    kit: &Kit,
    stage: &mut Stage<'_>,
    rule: crate::model::Rule,
    pops: &[u64],
    count: usize,
) -> fmn::Result<Vec<ListCard>> {
    if rule == crate::model::Rule::Hill {
        return Err(fail("Hill signposts are squared; they have no cards".into()));
    }
    let mut out = Vec::new();
    for (j, (town, seat, value)) in crate::model::signposts(rule, pops, count).iter().enumerate() {
        let value = crate::model::people(*value);
        let mobs = card_body(
            kit,
            stage,
            *town,
            *seat,
            &value,
            v(LIST_X, card_y(j + 1)),
            LIST_LABEL_DX,
        )?;
        out.push(ListCard {
            key: (*town, *seat),
            value,
            mobs,
        });
    }
    Ok(out)
}

/// Where a listed card's parts go when the card is centred at (x, y): the
/// background by its centre, the label by its left edge, the value by its
/// right edge.
fn card_spots(mobs: &[Mob], x: f64, y: f64) -> Vec<(Mob, Vec3, Vec3)> {
    vec![
        (mobs[0], v(x, y), ORIGIN),
        (mobs[1], v(x - CARD_W / 2.0 + LIST_LABEL_DX, y), LEFT),
        (mobs[2], v(x + CARD_W / 2.0 - 0.12, y), RIGHT),
    ]
}

/// A card-sized outline at (x, y) in a town's color, detached: where a
/// moving card left from, or where it will land.
fn ghost(stage: &mut Stage<'_>, town: usize, x: f64, y: f64, opacity: f64) -> fmn::Result<Mob> {
    let g = Rectangle::new()
        .width(CARD_W + 0.06)
        .height(CARD_H + 0.04)
        .corner_radius(0.05)
        .style(line_style(town_color(town), 2.0, opacity))
        .build()?
        .moved_to(v(x, y));
    Ok(stage.arena_mut().add(g))
}

/// Move a list of keyed cards from `shown` to `next`, the same rule's list
/// after only town `mover`'s population changed. Ranks stay in their column.
///
/// 1. The mover's cards change their values in place (the other towns'
///    values cannot change; that is checked).
/// 2. Each mover card whose rank changes gets a ghost outline where it is and
///    another where it will land, slides out into the lane, travels along it
///    and slides back in. The other towns' cards shift up or down inside the
///    list at the same time. Within each group the order never changes (also
///    checked), so no card ever passes through another.
/// 3. Cards pushed past the end fade out below the list; cards arriving from
///    below fade in under it and travel up their own way.
///
/// Returns the cards now on screen, in `next`'s order.
pub fn slide_cards(
    kit: &Kit,
    stage: &mut Stage<'_>,
    shown: Vec<ListCard>,
    next: Vec<ListCard>,
    mover: usize,
    rt: f64,
) -> fmn::Result<Vec<ListCard>> {
    let count = shown.len();
    let below = card_y(count + 1);
    let rank_in = |list: &[ListCard], key: CardKey| list.iter().position(|c| c.key == key).map(|r| r + 1);

    // 1. values change in place
    let mut morphs = Vec::new();
    for (a, card) in shown.iter().enumerate() {
        let Some(b) = rank_in(&next, card.key) else { continue };
        let new = &next[b - 1].value;
        if *new != card.value {
            if card.key.0 != mover {
                return Err(fail(format!(
                    "slide_cards: {:?} changed value but its town did not change",
                    card.key
                )));
            }
            let target = value_text(kit, stage, new, v(LIST_X, card_y(a + 1)))?;
            morphs.push((card.mobs[2], target));
        }
    }
    morph_all(stage, &morphs, 0.6)?;

    // 2. who travels where: (mobs, town, from y, to y, in the lane)
    let mut trips: Vec<(Vec<Mob>, usize, f64, f64, bool)> = Vec::new();
    let mut leaving = Vec::new();
    let mut order_lane: Vec<(usize, usize)> = Vec::new();
    let mut order_list: Vec<(usize, usize)> = Vec::new();
    for (a, card) in shown.iter().enumerate() {
        let a = a + 1;
        let b = rank_in(&next, card.key);
        let lane = card.key.0 == mover && b != Some(a);
        let to = b.map_or(below, card_y);
        if b.is_none() {
            leaving.extend(card.mobs.iter().copied());
        }
        let end = b.unwrap_or(count + 1);
        if lane {
            order_lane.push((a, end))
        } else {
            order_list.push((a, end))
        }
        trips.push((card.mobs.clone(), card.key.0, card_y(a), to, lane));
    }
    let mut arriving = Vec::new();
    for (b, card) in next.iter().enumerate() {
        if rank_in(&shown, card.key).is_some() {
            continue;
        }
        let lane = card.key.0 == mover;
        let x = if lane { LANE_X } else { LIST_X };
        for (m, at, edge) in card_spots(&card.mobs, x, below) {
            stage.move_to(m, at, edge);
        }
        arriving.extend(card.mobs.iter().copied());
        if lane {
            order_lane.push((count + 1, b + 1))
        } else {
            order_list.push((count + 1, b + 1))
        }
        trips.push((card.mobs.clone(), card.key.0, below, card_y(b + 1), lane));
    }
    for group in [&mut order_lane, &mut order_list] {
        group.sort_unstable();
        if group.windows(2).any(|w| w[0].1 > w[1].1) {
            return Err(fail("slide_cards: two cards in one group would cross".into()));
        }
    }

    // ghosts: where each lane card leaves from and where it will land
    let mut ghosts = Vec::new();
    for (_, town, from, to, lane) in &trips {
        if *lane {
            if *from > below + 1e-9 {
                ghosts.push(ghost(stage, *town, LIST_X, *from, 0.55)?);
            }
            if *to > below + 1e-9 {
                ghosts.push(ghost(stage, *town, LIST_X, *to, 0.9)?);
            }
        }
    }
    fade_in_all(stage, &ghosts, 0.3)?;

    // out to the lane (arrivals appear there, under the list)
    let mut out_moves = Vec::new();
    for (mobs, _, from, _, lane) in &trips {
        if *lane && !arriving.contains(&mobs[0]) {
            out_moves.extend(card_spots(mobs, LANE_X, *from));
        }
    }
    move_edges(stage, &out_moves, 0.5)?;
    fade_in_all(stage, &arriving, 0.3)?;

    // along the lane and inside the list, together
    let mut travel = Vec::new();
    for (mobs, _, _, to, lane) in &trips {
        travel.extend(card_spots(mobs, if *lane { LANE_X } else { LIST_X }, *to));
    }
    move_edges(stage, &travel, rt)?;

    // back into the list; leavers fade out below it; ghosts go
    let mut in_moves = Vec::new();
    for (mobs, _, _, to, lane) in &trips {
        if *lane && *to > below + 1e-9 {
            in_moves.extend(card_spots(mobs, LIST_X, *to));
        }
    }
    move_edges(stage, &in_moves, 0.5)?;
    fade_out_all(stage, &[ghosts, leaving].concat(), 0.4)?;

    let mut out = Vec::new();
    for card in next {
        match shown.iter().find(|c| c.key == card.key) {
            Some(old) => out.push(ListCard {
                key: card.key,
                value: card.value,
                mobs: old.mobs.clone(),
            }),
            None => out.push(card),
        }
    }
    Ok(out)
}

/// Move each mobject so the point its `edge` names lands on its target,
/// together, over `rt` seconds.
pub fn move_edges(stage: &mut Stage<'_>, moves: &[(Mob, Vec3, Vec3)], rt: f64) -> fmn::Result<()> {
    if moves.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(m, to, edge) in moves {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .move_to(to, edge)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// The cutoff under card `rank`: everything above it is a seat. Detached.
pub fn cutoff(stage: &mut Stage<'_>, rank: usize) -> Mob {
    let y = cutoff_y(rank);
    stroke(
        stage,
        vec![v(LIST_X - CARD_W / 2.0 - 0.15, y), v(LIST_X + CARD_W / 2.0 + 0.15, y)],
        line_style(Palette::hero(), 4.0, 1.0),
    )
}

pub fn cutoff_y(rank: usize) -> f64 {
    card_y(rank) - LIST_STEP / 2.0
}

// -------------------------------------------------------------- scorecard

/// The promises a rule can keep, as the scorecard's columns.
pub const PROMISES: [&str; 3] = ["quota promise", "no take backs", "population promise"];
/// The scorecard's column centres and the rule names' left edge.
pub const SCORE_X: [f64; 3] = [-0.6, 2.4, 5.4];
pub const SCORE_LABEL_X: f64 = -6.3;
pub const SCORE_STEP: f64 = 0.72;

/// The scorecard's heading row (the first `cols` promises) at height `y`.
pub fn score_head(kit: &Kit, stage: &mut Stage<'_>, cols: usize, y: f64) -> fmn::Result<Vec<Mob>> {
    let mut out = Vec::new();
    for c in 0..cols {
        out.push(kit.text(stage, PROMISES[c], 22.0, Palette::dim(), v(SCORE_X[c], y))?);
    }
    out.push(stroke(
        stage,
        vec![v(SCORE_LABEL_X, y - 0.32), v(SCORE_X[cols - 1] + 1.4, y - 0.32)],
        line_style(Palette::faint(), 2.0, 0.8),
    ));
    Ok(out)
}

/// One scorecard row: the rule's name and, per column, a check (kept
/// always), a cross (can be broken) or nothing, with an optional small note
/// under the mark ("never with 3 towns"). Detached.
pub fn score_row(
    kit: &Kit,
    stage: &mut Stage<'_>,
    rule: &str,
    marks: &[(Option<bool>, &str)],
    y: f64,
) -> fmn::Result<Vec<Mob>> {
    let mut out = vec![kit.text_left(stage, rule, 26.0, Palette::ink(), v(SCORE_LABEL_X, y))?];
    for (c, &(mark, note)) in marks.iter().enumerate() {
        let at = v(SCORE_X[c], y);
        match mark {
            Some(true) => out.push(check(stage, at, 0.42, Palette::green())),
            Some(false) => out.extend(cross(stage, at, 0.36, Palette::red())),
            None => {}
        }
        if !note.is_empty() {
            out.push(kit.text(stage, note, 15.0, Palette::dim(), v(SCORE_X[c] + 0.95, y - 0.05))?);
        }
    }
    Ok(out)
}

// --------------------------------------------------------------- playback

/// Bring several detached mobjects in together with a fade, `rt` seconds.
pub fn fade_in_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Bring detached mobjects in one after another, the whole taking `rt`.
pub fn fade_in_lagged(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64, lag: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// Fade several mobjects out together, `rt` seconds.
pub fn fade_out_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        stage.clear_updaters(m, true);
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Flash several mobjects (the highlight role), together.
pub fn flash_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fmn::animation::indicate(stage.arena_mut(), m, 1.25, None)?;
        anims.push(stage.prepare(a.rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Move each mobject to its point, together, over `rt` seconds.
pub fn move_all(stage: &mut Stage<'_>, moves: &[(Mob, Vec3)], rt: f64) -> fmn::Result<()> {
    if moves.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(m, to) in moves {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .move_to(to, ORIGIN)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Turn each mobject to its color and opacity, together, over `rt` seconds.
pub fn refill(stage: &mut Stage<'_>, mobs: &[(Mob, Srgb, f64)], rt: f64) -> fmn::Result<()> {
    if mobs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(m, color, opacity) in mobs {
        let a = m
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(rt),
                ..AnimateArgs::default()
            })?
            .set_color_with_opacity(color, opacity)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Morph each mobject into its (detached) target, together.
pub fn morph_all(stage: &mut Stage<'_>, pairs: &[(Mob, Mob)], rt: f64) -> fmn::Result<()> {
    if pairs.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(m, target) in pairs {
        anims.push(stage.prepare(Transform::new(m, target).rt(rt))?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Animate a tracker to `value` over `rt` seconds at an even pace.
pub fn run_tracker(stage: &mut Stage<'_>, tracker: Mob, value: f64, rt: f64) -> fmn::Result<()> {
    let a = tracker
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(rt),
            rate_func: Some(fmn::core::rate::linear),
            ..AnimateArgs::default()
        })?
        .set_value(value)?;
    stage.play(a)?;
    Ok(())
}

/// Animate a tracker to `value` over `rt` seconds, easing in and out.
pub fn ease_tracker(stage: &mut Stage<'_>, tracker: Mob, value: f64, rt: f64) -> fmn::Result<()> {
    stage.play(explainer_kit::tween(tracker, value, rt)?)?;
    Ok(())
}

/// Animate several trackers to their values together over `rt` seconds, at
/// an even pace (`linear`) or easing in and out.
pub fn run_trackers(stage: &mut Stage<'_>, moves: &[(Mob, f64)], rt: f64, linear: bool) -> fmn::Result<()> {
    if moves.is_empty() {
        return Ok(());
    }
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    for &(tracker, value) in moves {
        let mut args = AnimateArgs {
            run_time: Some(rt),
            ..AnimateArgs::default()
        };
        if linear {
            args.rate_func = Some(fmn::core::rate::linear);
        }
        let a = tracker.animate().set_anim_args(args)?.set_value(value)?;
        anims.push(stage.prepare(a)?);
    }
    stage.play_prepared(anims)?;
    Ok(())
}

/// Wait `seconds` (a beat of stillness after a reveal).
pub fn pause(stage: &mut Stage<'_>, seconds: f64) -> fmn::Result<()> {
    let fps = stage.scene().time().fps();
    wait_frames(stage, frames_covering(seconds, fps))
}

/// Wait (if need be) until scene time `t`.
pub fn wait_until(stage: &mut Stage<'_>, t: f64) -> fmn::Result<()> {
    let now = stage.scene().time().to_f64();
    if t > now {
        let fps = stage.scene().time().fps();
        wait_frames(stage, frames_covering(t - now, fps))?;
    }
    Ok(())
}

/// Let the current line finish, then fade everything on stage.
pub fn wipe(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.hold(stage)?;
    explainer_kit::fade_all(stage, 0.6)
}

// ------------------------------------------------------------ line timing

/// Words a second for a line with no narrator (visual-only renders).
const SILENT_WPS: f64 = 2.6;

/// A line being spoken: when it began, how long it runs and where its
/// sentences start, so a picture can land on the phrase it illustrates.
/// Without measured word marks a phrase's time is interpolated inside its
/// sentence: an approximate draft cue, not fine sync. Fine sync to the
/// ElevenLabs Oliver takes is root's, after the takes exist.
pub struct Spoken {
    pub id: String,
    pub start: f64,
    pub len: f64,
    text: String,
    sentences: Vec<(usize, f64)>,
    phrase_marks: std::collections::HashMap<usize, f64>,
}

/// The character each sentence of `text` starts at: after a `.`, `?` or
/// `!` and its space, as the narrator splits a line.
fn sentence_offsets(text: &str) -> Vec<usize> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = vec![0];
    for (i, pair) in chars.windows(2).enumerate() {
        if matches!(pair[0], '.' | '?' | '!') && pair[1] == ' ' && i + 2 < chars.len() {
            out.push(i + 2);
        }
    }
    out
}

/// Line `id`'s sentences with their starts: the narrator's measured onsets,
/// or each sentence's share of `len` by characters. A narrator that splits
/// the line differently is an error, never a guess.
fn timed_sentences(id: &str, text: &str, len: f64, measured: &[f64]) -> fmn::Result<Vec<(usize, f64)>> {
    let offsets = sentence_offsets(text);
    if measured.is_empty() {
        let all = text.chars().count().max(1) as f64;
        return Ok(offsets.into_iter().map(|at| (at, len * at as f64 / all)).collect());
    }
    if measured.len() != offsets.len() {
        return Err(fail(format!(
            "line {id} has {} sentences, but the narrator timed {}",
            offsets.len(),
            measured.len()
        )));
    }
    Ok(offsets.into_iter().zip(measured.iter().copied()).collect())
}

/// Speak line `id` now (after the previous line), as `Kit::say`.
pub fn say(kit: &Kit, stage: &mut Stage<'_>, id: &str) -> fmn::Result<Spoken> {
    kit.say(stage, id)?;
    let script = Script::parse(crate::SCRIPT).map_err(fail)?;
    let text = script
        .text(id)
        .ok_or_else(|| fail(format!("narration line {id} is not in script.tsv")))?
        .to_owned();
    let words = text.split_whitespace().count() as f64;
    let len = kit
        .narrator
        .as_ref()
        .and_then(|n| n.length(id))
        .unwrap_or(words / SILENT_WPS + 0.4);
    let sentences = timed_sentences(id, &text, len, &kit.sentence_starts(id))?;
    Ok(Spoken {
        id: id.to_owned(),
        start: stage.scene().time().to_f64(),
        len,
        text,
        sentences,
        phrase_marks: kit.narrator.as_ref().map(|n| n.phrase_marks(id)).unwrap_or_default(),
    })
}

impl Spoken {
    /// When `phrase` begins: a timing mark if one exists, else interpolated
    /// inside its sentence.
    pub fn at(&self, phrase: &str) -> fmn::Result<f64> {
        let byte = self
            .text
            .find(phrase)
            .ok_or_else(|| fail(format!("line {} has no phrase {phrase:?}", self.id)))?;
        if let Some(&seconds) = self.phrase_marks.get(&byte) {
            return Ok(self.start + seconds);
        }
        let i = self.text[..byte].chars().count();
        let all = self.text.chars().count().max(1);
        let j = self.sentences.iter().rposition(|&(at, _)| at <= i).unwrap_or(0);
        let (from, t0) = self.sentences[j];
        let (to, t1) = self.sentences.get(j + 1).copied().unwrap_or((all, self.len));
        let share = (i - from) as f64 / (to - from).max(1) as f64;
        Ok(self.start + t0 + (t1 - t0) * share)
    }

    /// Wait (if need be) until `phrase` begins.
    pub fn wait_for(&self, stage: &mut Stage<'_>, phrase: &str) -> fmn::Result<()> {
        let t = self.at(phrase)?;
        wait_until(stage, t)
    }
}

// --------------------------------------------------------------- think time

/// The prediction pause after a ⟨predict⟩ question (storyboard: listen 4 s).
pub const THINK_S: f64 = 4.0;
pub const THINK_R: f64 = 0.55;
pub const PAUSE_NOTE: &str = "pause for more time";
/// Where the think ring sits unless a chapter needs it elsewhere.
pub const THINK_AT: [f64; 2] = [6.1, -2.9];

/// Let the question finish, mark the moment for the score, and show a
/// large "?" in a gold ring that drains over [`THINK_S`], with "pause for
/// more time" under it. Returns the "?", which the answer clears.
pub fn think(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    kit.hold(stage)?;
    kit.mark(stage, "predict");
    let q = kit.text(stage, "?", 70.0, Palette::hero(), at)?;
    let note = kit.text(
        stage,
        PAUSE_NOTE,
        16.0,
        Palette::dim(),
        v(at[0], at[1] - THINK_R - 0.28),
    )?;
    let arc = move |f: f64| -> Vec<Vec3> {
        let f = f.clamp(0.002, 1.0);
        let n = (96.0 * f).ceil() as usize + 2;
        (0..n)
            .map(|i| {
                let a = std::f64::consts::FRAC_PI_2 - std::f64::consts::TAU * f * i as f64 / (n - 1) as f64;
                v(at[0] + THINK_R * a.cos(), at[1] + THINK_R * a.sin())
            })
            .collect()
    };
    let left = stage.add_value_tracker(1.0);
    let ring = stage
        .arena_mut()
        .add(polyline(arc(1.0)).style(line_style(Palette::hero(), 4.0, 1.0)).build());
    let anchor = stage.arena_mut().add(VMobject::new());
    let mut last = None;
    stage.add_updater(
        anchor,
        move |s, _| {
            let f = s.tracker_value(left).unwrap_or(0.0);
            if last == Some(f) {
                return;
            }
            let _ = s.set_points(ring, polyline(arc(f)).build().points());
            last = Some(f);
        },
        true,
    )?;
    fade_in_all(stage, &[anchor, ring, q, note], 0.2)?;
    run_tracker(stage, left, 0.0, THINK_S)?;
    fade_out_all(stage, &[anchor, ring, note], 0.2)?;
    Ok(q)
}

/// The think ring at its usual place.
pub fn think_here(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<Mob> {
    think(kit, stage, v(THINK_AT[0], THINK_AT[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_row_fits_left_of_the_readouts() {
        // Cedar never holds more than 12 seats in the film's rows
        assert!(seat_x(13.0) < READ_X);
        const { assert!(NAME_X + 1.6 < GRID_X) };
    }

    #[test]
    fn bands_split_whole_and_tail() {
        let [(x0, w0), (x1, w1)] = band_spans(1.5);
        assert!((x0 - GRID_X).abs() < 1e-12 && (w0 - CELL).abs() < 1e-12);
        assert!((x1 - seat_x(1.0)).abs() < 1e-12 && (w1 - 0.5 * CELL).abs() < 1e-12);
        // an exact whole share has no tail (Cedar at 15 seats)
        let [_, (_, w)] = band_spans(10.0);
        assert!(w < 0.01);
    }

    #[test]
    fn nineteen_cards_fit_the_frame() {
        assert!(card_y(1) + CARD_H / 2.0 < 4.0);
        assert!(card_y(19) - CARD_H / 2.0 > -4.0);
        const { assert!(LIST_X + CARD_W / 2.0 < 7.1) };
    }

    #[test]
    fn sentences_split_like_the_narrator() {
        assert_eq!(sentence_offsets("One. Two? Three."), vec![0, 5, 10]);
    }
}
