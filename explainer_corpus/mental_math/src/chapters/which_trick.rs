//! Chapter 12, which trick: the mixed round begins. The six small pictures
//! of 01_7 come back as tiles along the top, and from here the viewer picks
//! the trick before computing: a twenty percent tip (12_1), worked from a
//! tenth, doubled, while the bill's tile lights and doubles its tenth
//! (12_2); forty-nine times fifty-one (12_3), one either side of fifty,
//! while the middle's tile lights and closes its two bows (12_4). Each
//! working ends on the answer already on the card.

use explainer_kit::{Kit, Palette, Timed, play, tween, v};
use fmn::prelude::*;

use super::draw::{
    NumberLine1, QuizCard, QuizLook, Tile, TileKind, WHICH_TRICK, bow, dollars, eq, fade_in_lagged, fail, flat, fly,
    glyphs, group, middle_of, num, say, tiles_rest_and_clear, tiles_top, your_turn,
};
use crate::model::{Ask, blocks, money, pair};

pub struct WhichTrick {
    pub kit: Kit,
}

/// The quiz cards sit under the tiles at the left; the working stands
/// beside them, left-aligned at `WORK_X`, one line under another.
const CARD_AT: Vec3 = [-3.35, -0.55, 0.0];
const CARD_SCALE: f64 = 0.85;
const WORK_X: f64 = 0.8;
const WORK_TOP: f64 = 0.45;
const WORK_STEP: f64 = 1.2;
const WORK_PT: f64 = 64.0;
/// How far the tiles that do not fit go down, and how near the middle (in
/// the tile's hundredths) its two numbers close.
const DARK: f64 = 0.3;
const CLOSED: f64 = 15.0;

/// What each part of a tile was built with (fill alpha, stroke alpha), so
/// the tile can be lit, put out and put back. `Tile::light` sets both
/// alphas of every part, which would fill the middle's bows and the fifty's
/// outline: here a line keeps no fill and a fill no stroke.
fn alphas(stage: &Stage<'_>, tile: &Tile) -> Vec<(f64, f64)> {
    tile.parts
        .iter()
        .map(|&m| {
            (
                stage.get_fill_opacity(m).unwrap_or(0.0),
                stage.get_stroke_opacity(m).unwrap_or(0.0),
            )
        })
        .collect()
}

/// A tile's parts at `k` of their own alphas and its frame edged in
/// `edge`, as animations to play together.
fn shine(
    stage: &mut Stage<'_>,
    tile: &Tile,
    base: &[(f64, f64)],
    k: f64,
    edge: Srgb,
    rt: f64,
) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let args = || AnimateArgs {
        run_time: Some(rt),
        ..AnimateArgs::default()
    };
    let mut anims: Vec<Box<dyn Animation>> = Vec::new();
    let frame = tile
        .frame
        .animate()
        .set_anim_args(args())?
        .set_stroke(Some(edge), None, Some(1.0))?;
    anims.push(stage.prepare(frame)?);
    for (&m, &(fill, stroke)) in tile.parts.iter().zip(base) {
        let a = m
            .animate()
            .set_anim_args(args())?
            .set_fill(None, Some(fill * k))?
            .set_stroke(None, None, Some(stroke * k))?;
        anims.push(stage.prepare(a)?);
    }
    Ok(anims)
}

/// Light the tile at `lit` and put the others out (or, with no tile lit,
/// put every tile back as it was built).
fn light(
    stage: &mut Stage<'_>,
    tiles: &[Tile],
    bases: &[Vec<(f64, f64)>],
    lit: Option<usize>,
    rt: f64,
) -> fmn::Result<Vec<Box<dyn Animation>>> {
    let mut anims = Vec::new();
    for (i, (tile, base)) in tiles.iter().zip(bases).enumerate() {
        let (k, edge) = match lit {
            Some(j) if j == i => (1.0, Palette::ink()),
            Some(_) => (DARK, Palette::faint()),
            None => (1.0, Palette::faint()),
        };
        anims.extend(shine(stage, tile, base, k, edge, rt)?);
    }
    Ok(anims)
}

/// The tile that pictures the trick of `quiz`.
fn tile_for(tiles: &[Tile], card: &QuizCard) -> fmn::Result<usize> {
    let kind = TileKind::of(card.quiz.trick);
    tiles
        .iter()
        .position(|t| t.kind == kind)
        .ok_or_else(|| fail(format!("no tile pictures {:?}", card.quiz.trick)))
}

/// A copy of the working's last number flies into the answer already on the
/// card and lands on it. `line` is the working, `last` the number's TeX.
fn land(kit: &Kit, stage: &mut Stage<'_>, card: &QuizCard, line: Mob, last: &str) -> fmn::Result<()> {
    let probe = eq(kit, stage, last, WORK_PT, ORIGIN)?;
    let k = glyphs(stage, probe).len();
    let all = glyphs(stage, line);
    if all.len() < k {
        return Err(fail(format!("the working has no {last} at its end")));
    }
    let copy = eq(kit, stage, last, WORK_PT, middle_of(stage, &all[all.len() - k..])?)?;
    stage.add_to_scene(copy)?;
    let to = stage.get_center(card.answer);
    let scale = stage.get_width(card.answer) / stage.get_width(copy);
    play!(stage; fly(copy, to, scale, 0.8)?);
    stage.remove_from_scene(copy);
    Ok(())
}

impl SceneConstruct for WhichTrick {
    fn name(&self) -> &str {
        "12_which_trick"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let ink = Palette::ink();
        let look = QuizLook {
            at: CARD_AT,
            label: Some(WHICH_TRICK),
            pause_line: true,
            lands: true,
            scale: CARD_SCALE,
        };
        let work_y = |row: f64| WORK_TOP - WORK_STEP * row;

        // 12_1: the six tiles arrive along the top during the first
        // sentence; the card after it: a twenty percent tip on $45
        let tiles = tiles_top(stage)?;
        let bases: Vec<Vec<(f64, f64)>> = tiles.iter().map(|t| alphas(stage, t)).collect();
        let tile_mobs: Vec<Mob> = tiles
            .iter()
            .map(|t| group(stage, &t.mobs()))
            .collect::<fmn::Result<_>>()?;
        let line = say(&kit, stage, "12_1")?;
        fade_in_lagged(stage, &tile_mobs, 2.4, 0.4)?;
        let tip = QuizCard::build(&kit, stage, "12_1", look)?;
        line.wait_sentence(stage, 1)?;
        tip.show(stage, 0.5)?;
        tip.count(&kit, stage)?;

        // 12_2: the bill's tile lights; a tenth, $4.50, doubled: $9.00
        let line = say(&kit, stage, "12_2")?;
        let cents = match tip.quiz.ask {
            Ask::PercentOfMoney(_, cents) => cents,
            other => return Err(fail(format!("12_1 asks {other:?}, not a percent of money"))),
        };
        let b = blocks(cents).ok_or_else(|| fail(format!("a tenth of {cents} cents is not whole cents")))?;
        let worked = tip.quiz.worked().map_err(fail)?;
        if worked.steps != [money(b.ten)] || worked.answer != money(b.twenty) {
            return Err(fail(format!("12_1 is worked as {worked:?}, not a tenth doubled")));
        }
        let doubled = b.twenty / b.ten;
        let bill = tile_for(&tiles, &tip)?;
        let anims = light(stage, &tiles, &bases, Some(bill), 0.5)?;
        stage.play_prepared(anims)?;
        // the working is typeset whole for its layout, then shown in two
        // pieces, each where it sits in the whole
        let full_src = format!(r"{} \times {doubled} = {}", dollars(b.ten), dollars(b.twenty));
        let full = eq(&kit, stage, &full_src, WORK_PT, ORIGIN)?;
        stage.move_to(full, v(WORK_X, work_y(0.5)), LEFT);
        let all = glyphs(stage, full);
        let tenth_src = dollars(b.ten);
        let tenth = eq(&kit, stage, &tenth_src, WORK_PT, ORIGIN)?;
        let split = glyphs(stage, tenth).len();
        if all.len() <= split {
            return Err(fail(format!("{full_src} has nothing after its tenth")));
        }
        let at = middle_of(stage, &all[..split])?;
        stage.move_to(tenth, at, ORIGIN);
        // a leading {} keeps the times sign a binary operator, spaced as in
        // the whole line
        let rest_src = format!(r"{{}} \times {doubled} = {}", dollars(b.twenty));
        let rest = eq(&kit, stage, &rest_src, WORK_PT, ORIGIN)?;
        let at = middle_of(stage, &all[split..])?;
        stage.move_to(rest, at, ORIGIN);
        line.wait_sentence(stage, 1)?;
        play!(stage; fade_in(stage.arena_mut(), tenth, v(0.0, -0.12), 1.0)?.rt(0.4));
        // doubled: inside the tile a second tenth slides out of the first
        line.wait_sentence(stage, 2)?;
        let tile = &tiles[bill];
        let (one, cuts) = match (tile.kind, tile.parts.as_slice()) {
            (TileKind::Bill, [_, one, cuts @ ..]) => (*one, cuts.to_vec()),
            _ => return Err(fail("the bill's tile has no tenth".into())),
        };
        let (l, r) = (stage.get_left(one)[0], stage.get_right(one)[0]);
        let (lo, hi) = (stage.get_bottom(one)[1], stage.get_top(one)[1]);
        let tenth_alpha = bases[bill].get(1).map_or(1.0, |&(fill, _)| fill);
        let twin = stage.arena_mut().add(
            Polygon::new([v(l, lo), v(r, lo), v(r, hi), v(l, hi)])
                .style(flat(ink, tenth_alpha))
                .build(),
        );
        stage.add_to_scene(twin)?;
        stage.scene_mut().bring_to_front(&cuts)?;
        let slot = v((l + r) / 2.0 + (r - l), (lo + hi) / 2.0);
        play!(stage;
            fly(twin, slot, 1.0, 0.5)?,
            fade_in(stage.arena_mut(), rest, v(-0.2, 0.0), 1.0)?.rt(0.5),
        );
        land(&kit, stage, &tip, rest, &dollars(b.twenty))?;

        // 12_3: forty-nine times fifty-one; the tiles back as they were
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        let mut gone = tip.mobs();
        gone.extend([tenth, rest, twin]);
        for m in gone {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        anims.extend(light(stage, &tiles, &bases, None, 0.5)?);
        stage.play_prepared(anims)?;
        let product = your_turn(&kit, stage, "12_3", look)?;

        // 12_4: the middle's tile lights and closes its two bows; one either
        // side of fifty: 50² − 1² = 2500 − 1 = 2499
        let line = say(&kit, stage, "12_4")?;
        let (x, y) = match product.quiz.ask {
            Ask::Times(x, y) => (x, y),
            other => return Err(fail(format!("12_3 asks {other:?}, not a product"))),
        };
        let p = pair(x, y).ok_or_else(|| fail(format!("{x} and {y} have no whole middle")))?;
        let worked = product.quiz.worked().map_err(fail)?;
        if worked.answer != num(p.product) {
            return Err(fail(format!("12_3 is worked as {worked:?}, not {}", p.product)));
        }
        let middle = tile_for(&tiles, &product)?;
        let tile = &tiles[middle];
        let [axis, _, left_bow, right_bow, left_dot, right_dot] = match (tile.kind, tile.parts.as_slice()) {
            (TileKind::Middle, &[a, t, lb, rb, ld, rd]) => [a, t, lb, rb, ld, rd],
            _ => return Err(fail("the middle's tile is not a line with two bows".into())),
        };
        // the tile's own number line, read off its axis: 0 to 100 across
        let ruler = NumberLine1 {
            x0: stage.get_left(axis)[0],
            x1: stage.get_right(axis)[0],
            y: stage.get_center(axis)[1],
            lo: 0.0,
            hi: 100.0,
        };
        let value = |stage: &Stage<'_>, m: Mob| {
            ruler.lo + (stage.get_center(m)[0] - ruler.x0) / (ruler.x1 - ruler.x0) * (ruler.hi - ruler.lo)
        };
        let (a0, b0) = (value(stage, left_dot), value(stage, right_dot));
        let mid = (a0 + b0) / 2.0;
        let lift0 = stage.get_height(left_bow);
        let close = stage.add_value_tracker(0.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut last = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let c = s.tracker_value(close).unwrap_or(0.0);
                if c == last {
                    return;
                }
                let a = a0 + (mid - CLOSED - a0) * c;
                let b = b0 + (mid + CLOSED - b0) * c;
                let lift = lift0 * (mid - a) / (mid - a0);
                let arcs = [
                    (left_bow, ruler.p(a), ruler.p(mid)),
                    (right_bow, ruler.p(mid), ruler.p(b)),
                ];
                for (m, from, to) in arcs {
                    let shape = polyline(bow(from, to, lift)).build();
                    let _ = s.set_points(m, shape.points());
                }
                s.move_to(left_dot, ruler.p(a), ORIGIN);
                s.move_to(right_dot, ruler.p(b), ORIGIN);
                last = c;
            },
            true,
        )?;
        stage.add_to_scene(anchor)?;
        let rows = [
            format!("{}^2 - {}^2", p.middle, p.distance),
            format!("= {} - {}", p.middle_squared, p.distance_squared),
            format!("= {}", p.product),
        ];
        let mut working = Vec::new();
        for (i, src) in rows.iter().enumerate() {
            let m = eq(&kit, stage, src, WORK_PT, ORIGIN)?;
            stage.move_to(m, v(WORK_X, work_y(i as f64)), LEFT);
            working.push(m);
        }
        let mut anims = light(stage, &tiles, &bases, Some(middle), 0.5)?;
        let a = fade_in(stage.arena_mut(), working[0], v(0.0, -0.12), 1.0)?;
        anims.push(stage.prepare(a.rt(0.5))?);
        stage.play_prepared(anims)?;
        play!(stage; tween(close, 1.0, 0.8)?);
        line.wait_sentence(stage, 1)?;
        play!(stage; fade_in(stage.arena_mut(), working[1], v(0.0, -0.12), 1.0)?.rt(0.4));
        line.wait_sentence(stage, 2)?;
        play!(stage; fade_in(stage.arena_mut(), working[2], v(0.0, -0.12), 1.0)?.rt(0.4));
        land(&kit, stage, &product, working[2], &num(p.product))?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        // the tiles stay, all at rest, for chapter 13's quiz
        tiles_rest_and_clear(stage, &tiles, &tile_mobs, 0.8)?;
        kit.finish(stage)
    }
}
