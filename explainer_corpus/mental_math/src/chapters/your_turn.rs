//! Chapter 03, your turn: two quizzes on the trick, each worked in the same
//! place and ending on the answer already on the card: twenty-five squared,
//! with the move replayed small on its square (03_1, 03_2), and one hundred
//! five squared, past a hundred, where the number in front is ten (03_3,
//! 03_4); the sweep, every number in front from one to ten on one picture
//! whose strip and small square never change (03_5); the same line in
//! letters (03_6); and the honest limit, 135 squared, whose card goes to
//! wait in the top right corner for chapter 05 (03_7).

use explainer_kit::props::lower_third;
use explainer_kit::{Kit, Palette, Timed, play, thousands, tween, v};
use fmn::library::{SpanMapData, SpanMapEntry, TransformMatchingTex};
use fmn::prelude::*;

use super::draw::{
    Board, COLUMN_ROOM, COLUMN_X, MoveFigure, NOTE_PT, NumberLine1, QuizCard, QuizLook, SIDE_PT, Side, WAITING_AT,
    corner, cut_style, eq, fade_all_but, fade_in_all, fade_out_all, faded_line, fail, fit_width, flat, fly, glyphs,
    hundreds, line_style, middle_of, num, pause, quiz_at, rgb, say, strip, swap, waiting_card, your_turn,
};
use crate::facts::{facts, shown};
use crate::model::{Ask, EndsInFive, OneMove, ends_in_five, one_move};

pub struct YourTurn {
    pub kit: Kit,
}

// the chapter's fixed words (numbers are built from facts)
const IN_FRONT: &str = "the number in front";
/// 03_6: the line in letters (the identity the sweep has shown), and the
/// lower-third's words before the last number the program checked.
const LETTERS: &str = r"(10n+5)^2 = 100 \times n \times (n+1) + 25";
const CHECKED: &str = "the picture works at any size; a program also checked every one up to ";
/// The parts of the line in letters: what is new in it (it matches nothing
/// in the line of numbers), the hundreds, and the letter.
const LETTERS_NEW: [&str; 2] = ["10n+", "(n+1)"];
const LETTERS_HUNDREDS: &str = r"100 \times n \times (n+1)";
const LETTER: &str = "n";
/// The fact that says how far the program checked.
const CHECKED_KEY: &str = "F-X1.ends_in_five.largest";
const CHECKED_HOLDS: &str = "F-X1.ends_in_five.holds";
const CHECKED_ALL: &str = "F-X1.ends_in_five.checked";

/// The working under a card, and the numbers flown into its answer.
const WORK_PT: f64 = 64.0;
/// 03_1 and 03_2: the card at the left, the working under it, and the move
/// replayed small on a square of twenty-five at the right.
const FIRST_AT: Vec3 = [-3.5, 0.75, 0.0];
const FIRST_SCALE: f64 = 0.75;
const FIRST_WORK_Y: f64 = -2.0;
const SMALL_AT: Vec3 = [1.1, -2.85, 0.0];
const SMALL_UNIT: f64 = 0.15;
/// The replayed turn is quicker than the one move's own two seconds.
const SMALL_TURN_S: f64 = 1.3;
/// 03_5, the sweep: a board on which 110 wide by 105 tall fits the left of
/// the frame; the slider and the two readouts in the column.
const SWEEP_AT: Vec3 = [-6.6, -3.2, 0.0];
const SWEEP_UNIT: f64 = 0.062;
const SLIDER_Y: f64 = 2.2;
const SLIDER_HALF: f64 = 2.5;
const PRODUCT_Y: f64 = 0.55;
const SQUARE_Y: f64 = -0.85;
const SQUARE_PT: f64 = 72.0;
/// The first stops of the sweep (the squares already worked) go slower.
const SLOW_STOPS: usize = 3;
/// 03_6: the line of numbers and the line in letters.
const FORMULA_PT: f64 = 80.0;
/// The number split into its tens and its five, under the formula.
const SPLIT_PT: f64 = 52.0;
const FORMULA_Y: f64 = 0.7;
const FORMULA_ROOM: f64 = 12.6;
/// 03_7: the waiting card, large in the middle before it goes to its
/// corner.
const BIG: f64 = 2.4;
const BIG_AT: Vec3 = [0.0, 0.5, 0.0];

/// The move on `side` with a strip `deep`, or a scene error.
fn the_move_of(side: u64, deep: u64) -> fmn::Result<OneMove> {
    one_move(side, deep).ok_or_else(|| fail(format!("no strip {deep} deep comes off a square of {side}")))
}

/// The square a quiz asks for, worked by the trick.
fn squared_at(id: &str) -> fmn::Result<EndsInFive> {
    match quiz_at(id)?.ask {
        Ask::Square(x) => ends_in_five(x).ok_or_else(|| fail(format!("{x} does not end in five"))),
        other => Err(fail(format!("quiz {id} asks {other:?}, not a square"))),
    }
}

/// Recolour the glyphs of `tex` whose indices are in `which`.
fn paint(stage: &mut Stage<'_>, tex: Mob, which: impl IntoIterator<Item = usize>, color: Srgb) {
    let all = glyphs(stage, tex);
    for i in which {
        if let Some(&g) = all.get(i) {
            stage.set_color(g, color, None, true);
        }
    }
}

/// A line under `mobs`, at height `y`, `extra` longer at each end. Detached.
fn underline(stage: &mut Stage<'_>, mobs: &[Mob], y: f64, extra: f64, color: Srgb) -> Mob {
    let l = mobs.iter().map(|&m| stage.get_left(m)[0]).fold(f64::INFINITY, f64::min);
    let r = mobs
        .iter()
        .map(|&m| stage.get_right(m)[0])
        .fold(f64::NEG_INFINITY, f64::max);
    stage.arena_mut().add(
        polyline(vec![v(l - extra, y), v(r + extra, y)])
            .style(line_style(color, 5.0))
            .build(),
    )
}

/// The lowest point of `mobs`.
fn bottom_of(stage: &Stage<'_>, mobs: &[Mob]) -> f64 {
    mobs.iter()
        .map(|&m| stage.get_bottom(m)[1])
        .fold(f64::INFINITY, f64::min)
}

/// The working's two numbers, the hundreds and the 25, slide up into the
/// answer already on `card`, which takes their colours as they arrive (as
/// at 01_2). `product` ends in the hundreds; `then` is the 25.
fn land(kit: &Kit, stage: &mut Stage<'_>, card: &QuizCard, e: &EndsInFive, product: Mob, then: Mob) -> fmn::Result<()> {
    let digits = glyphs(stage, card.answer);
    let split = num(e.hundreds).len();
    let product_glyphs = glyphs(stage, product);
    if digits.len() <= split || product_glyphs.len() < split {
        return Err(fail(format!("{} has no digits after its hundreds", e.square)));
    }
    let (lead_at, tail_at) = (middle_of(stage, &digits[..split])?, middle_of(stage, &digits[split..])?);
    let from = middle_of(stage, &product_glyphs[product_glyphs.len() - split..])?;
    let lead = kit.tex(stage, &num(e.hundreds), WORK_PT, hundreds(), &[], from)?;
    stage.add_to_scene(lead)?;
    let k = card.look.pt() / WORK_PT;
    play!(stage;
        fly(lead, lead_at, k, 0.8)?,
        fly(then, tail_at, k, 0.8)?,
    );
    for &g in &digits[..split] {
        stage.set_color(g, hundreds(), None, true);
    }
    for &g in &digits[split..] {
        stage.set_color(g, corner(), None, true);
    }
    stage.remove_from_scene(lead);
    stage.remove_from_scene(then);
    Ok(())
}

/// "n × (n + 1) = hundreds" under a card, the hundreds in their colour, and
/// the 25 after it in the corner's, both detached.
fn working(
    kit: &Kit,
    stage: &mut Stage<'_>,
    e: &EndsInFive,
    corner_area: u64,
    at: Vec3,
    gap: f64,
) -> fmn::Result<[Mob; 2]> {
    let src = format!(r"{} \times {} = {}", e.n, e.n + 1, e.hundreds);
    let product = eq(kit, stage, &src, WORK_PT, at)?;
    let n = glyphs(stage, product).len();
    paint(stage, product, n - num(e.hundreds).len()..n, hundreds());
    let right = stage.get_right(product)[0];
    let then = kit.tex(stage, &num(corner_area), WORK_PT, corner(), &[], v(right + gap, at[1]))?;
    stage.move_to(then, v(right + gap, at[1]), LEFT);
    Ok([product, then])
}

/// The glyph indices of a span map that lie inside the first occurrence of
/// `needle` in its source.
fn within(spans: &SpanMapData, needle: &str) -> fmn::Result<Vec<usize>> {
    let start = spans
        .source
        .find(needle)
        .ok_or_else(|| fail(format!("{needle:?} is not in {:?}", spans.source)))?;
    let end = start + needle.len();
    Ok(spans
        .entries
        .iter()
        .enumerate()
        .filter(|(_, e)| e.start >= start && e.end <= end)
        .map(|(i, _)| i)
        .collect())
}

/// The glyph indices whose source is exactly `key`.
fn keyed(spans: &SpanMapData, key: &str) -> Vec<usize> {
    spans
        .entries
        .iter()
        .enumerate()
        .filter(|(_, e)| spans.source.get(e.start..e.end) == Some(key))
        .map(|(i, _)| i)
        .collect()
}

/// The same span map, with every glyph inside the first occurrence of each
/// of `fresh` given a key of its own. `TransformMatchingTex` pairs equal
/// source slices in order, so a glyph that is new in the target (the 1 of
/// 10n) would otherwise take the place of an old one (the 1 of 100); a
/// fresh glyph matches nothing and fades in.
fn fresh_keys(spans: &SpanMapData, fresh: &[&str]) -> fmn::Result<SpanMapData> {
    let mut new = std::collections::HashSet::new();
    for f in fresh {
        new.extend(within(spans, f)?);
    }
    let mut source = String::new();
    let mut entries = Vec::new();
    for (i, e) in spans.entries.iter().enumerate() {
        let key = spans
            .source
            .get(e.start..e.end)
            .ok_or_else(|| fail(format!("span {i} of {:?} is not a source slice", spans.source)))?;
        let key = if new.contains(&i) {
            format!("#{i}")
        } else {
            key.to_owned()
        };
        let start = source.len();
        source.push_str(&key);
        entries.push(SpanMapEntry {
            start,
            end: source.len(),
            kind: e.kind,
        });
    }
    Ok(SpanMapData {
        source: source.into(),
        entries,
    })
}

/// A path of separate straight segments (one mobject, many lines): each
/// segment is one quadratic, and a handle on the last anchor starts the
/// next one.
fn segments(lines: &[(Vec3, Vec3)]) -> Vec<Vec3> {
    let mut points: Vec<Vec3> = Vec::new();
    for &(a, b) in lines {
        if let Some(&last) = points.last() {
            points.push(last);
        }
        points.push(a);
        points.push(v((a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0));
        points.push(b);
    }
    points
}

/// The moved-strip picture for the number in front `n` (03_5), in board
/// units: a strip `deep` deep has come off the top of a square of side
/// 10n + deep and stands against its right side.
#[derive(Clone, Copy, Debug)]
struct Swept {
    side: f64,
    deep: f64,
}

impl Swept {
    fn at(n: f64, deep: f64) -> Self {
        Self {
            side: 10.0 * n + deep,
            deep,
        }
    }

    fn tall(&self) -> f64 {
        self.side - self.deep
    }

    fn wide(&self) -> f64 {
        self.side + self.deep
    }

    /// The body, the standing strip below the small square, the small
    /// square.
    fn pieces(&self) -> [[f64; 4]; 3] {
        let (s, d, t) = (self.side, self.deep, self.tall());
        [[0.0, 0.0, s, t], [s, 0.0, d, t], [s, t, d, d]]
    }

    /// The cuts between the fills: the seam beside the standing strip and
    /// the line under the small square.
    fn cuts(&self) -> [([f64; 2], [f64; 2]); 2] {
        let (s, d, t) = (self.side, self.deep, self.tall());
        [([s, 0.0], [s, t]), ([s, t], [s + d, t])]
    }

    /// The rectangle ruled into hundreds: a line every ten units each way,
    /// and its border.
    fn rules(&self) -> Vec<([f64; 2], [f64; 2])> {
        let (w, t) = (self.wide(), self.tall());
        let mut out = Vec::new();
        let mut x = 0.0;
        while x < w - 1e-6 {
            out.push(([x, 0.0], [x, t]));
            x += 10.0;
        }
        out.push(([w, 0.0], [w, t]));
        let mut y = 0.0;
        while y < t - 1e-6 {
            out.push(([0.0, y], [w, y]));
            y += 10.0;
        }
        out.push(([0.0, t], [w, t]));
        out
    }
}

/// The sweep (03_5): the moved-strip picture for every number in front,
/// drawn from one tracker by one updater that rewrites a handful of
/// points: the body in the hundreds' colour, the strip five deep in the
/// strips', the small square in the corner's, the rectangle ruled into its
/// hundreds; the strip's depth and the small square's area ride along with
/// them; a slider shows the number in front.
struct Sweep {
    front: Mob,
    standing: Mob,
    small: Mob,
    deep_label: Mob,
    small_label: Mob,
    /// Everything, back to front, for the fades.
    all: Vec<Mob>,
}

impl Sweep {
    fn build(kit: &Kit, stage: &mut Stage<'_>, deep: u64, small_area: u64, last: u64) -> fmn::Result<Self> {
        let board = Board::new(SWEEP_AT, SWEEP_UNIT);
        let deep_f = deep as f64;
        let start = Swept::at(1.0, deep_f);
        let [body_r, standing_r, small_r] = start.pieces();
        let body = board.rect(stage, body_r, flat(hundreds(), 1.0));
        let standing = board.rect(stage, standing_r, flat(strip(), 1.0));
        let small = board.rect(stage, small_r, flat(corner(), 1.0));
        let line_of = |stage: &mut Stage<'_>, style: Style| {
            stage.arena_mut().add(
                polyline(vec![board.p(0.0, 0.0), board.p(1.0, 0.0)])
                    .style(style)
                    .build(),
            )
        };
        let cuts = line_of(stage, cut_style());
        let rules = line_of(stage, faded_line(Palette::ink(), 2.5, 0.85));
        let deep_label = kit.text(stage, &num(deep), NOTE_PT, strip(), ORIGIN)?;
        let small_label = kit.text(stage, &num(small_area), NOTE_PT, corner(), ORIGIN)?;
        // the slider: the number in front, 1 to the last
        let slider = NumberLine1 {
            x0: COLUMN_X - SLIDER_HALF,
            x1: COLUMN_X + SLIDER_HALF,
            y: SLIDER_Y,
            lo: 1.0,
            hi: last as f64,
        };
        let mut track = vec![slider.axis(stage)];
        for n in 1..=last {
            track.push(slider.tick(stage, n as f64, 0.2, Palette::dim()));
            track.push(slider.label(kit, stage, &num(n), n as f64, 26.0, Palette::dim(), false)?);
        }
        let title = kit.text(stage, IN_FRONT, 30.0, hundreds(), v(COLUMN_X, SLIDER_Y + 0.6))?;
        let knob = slider.dot(stage, 1.0, 0.13, hundreds());
        let front = stage.add_value_tracker(1.0);
        let anchor = stage.arena_mut().add(VMobject::new());
        let mut seen = f64::NAN;
        stage.add_updater(
            anchor,
            move |s, _| {
                let n = s.tracker_value(front).unwrap_or(1.0);
                if n == seen {
                    return;
                }
                let shape = Swept::at(n, deep_f);
                for (m, r) in [body, standing, small].into_iter().zip(shape.pieces()) {
                    let quad = Polygon::new(board.quad(r)).build();
                    let _ = s.set_points(m, quad.points());
                }
                let to_scene = |lines: &[([f64; 2], [f64; 2])]| -> Vec<(Vec3, Vec3)> {
                    lines
                        .iter()
                        .map(|&(a, b)| (board.p(a[0], a[1]), board.p(b[0], b[1])))
                        .collect()
                };
                let _ = s.set_points(cuts, &segments(&to_scene(&shape.cuts())));
                let _ = s.set_points(rules, &segments(&to_scene(&shape.rules())));
                let [_, standing_r, small_r] = shape.pieces();
                let under = board.p(standing_r[0] + standing_r[2] / 2.0, 0.0);
                s.move_to(deep_label, v(under[0], under[1] - 0.15), UP);
                let beside = board.p(small_r[0] + small_r[2], small_r[1] + small_r[3] / 2.0);
                s.move_to(small_label, v(beside[0] + 0.15, beside[1]), LEFT);
                s.move_to(knob, slider.p(n), ORIGIN);
                seen = n;
            },
            true,
        )?;
        let mut all = vec![anchor, body, standing, small, cuts, rules, deep_label, small_label];
        all.extend(track);
        all.extend([title, knob]);
        Ok(Self {
            front,
            standing,
            small,
            deep_label,
            small_label,
            all,
        })
    }
}

impl SceneConstruct for YourTurn {
    fn name(&self) -> &str {
        "03_your_turn"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (blue, orange, ink) = (hundreds(), corner(), Palette::ink());

        // 03_1: your turn: twenty-five squared, the card at the left
        let first = squared_at("03_1")?;
        let first_card = your_turn(&kit, stage, "03_1", QuizLook::at(FIRST_AT, FIRST_SCALE))?;

        // 03_2: two times three is six, then twenty-five, worked under the
        // card; beside it, small, the move replayed on a square of 25: 20
        // tall, 30 wide and a small square
        let line = say(&kit, stage, "03_2")?;
        let m = the_move_of(first.number, first.number % 10)?;
        let figure = MoveFigure::build(stage, Board::new(SMALL_AT, SMALL_UNIT), m, blue, strip(), 0.0)?;
        let work_at = v(FIRST_AT[0] - 0.9, FIRST_WORK_Y);
        let [product, then] = working(&kit, stage, &first, m.corner(), work_at, 0.7)?;
        let mut ins = figure.square();
        ins.extend([figure.cut, product]);
        fade_in_all(stage, &ins, 0.5)?;
        stage.remove_from_scene(figure.cut);
        stage
            .scene_mut()
            .bring_to_front(&[figure.strip.fill, figure.strip.edge])?;
        play!(stage; tween(figure.strip.t, 1.0, SMALL_TURN_S)?);
        stage.add_to_scene(figure.seam)?;
        line.wait_sentence(stage, 1)?;
        let board = figure.board;
        let rectangle = figure.rectangle_rect();
        let small_labels = [
            board.label(&kit, stage, &num(m.tall()), rectangle, Side::Left, SIDE_PT, ink)?,
            board.label(&kit, stage, &num(m.wide()), rectangle, Side::Below, SIDE_PT, ink)?,
            board.label(
                &kit,
                stage,
                &num(m.corner()),
                figure.corner_rect(),
                Side::Right,
                NOTE_PT,
                orange,
            )?,
        ];
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for mob in [figure.corner, figure.corner_cut, then].into_iter().chain(small_labels) {
            let a = fade_in(stage.arena_mut(), mob, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a.rt(0.4))?);
        }
        stage.play_prepared(anims)?;
        line.wait_sentence(stage, 2)?;
        land(&kit, stage, &first_card, &first, product, then)?;

        // 03_5: the sweep: the number in front runs 1 to 10; at each stop the
        // rectangle is that many tens tall and one more wide, ruled into its
        // hundreds, with the count of hundreds and the square beside it; the
        // strip five deep and the small square never change
        let line = say(&kit, stage, "03_5")?;
        let fives: Vec<EndsInFive> = (1..=shown::SWEEP)
            .map(|n| ends_in_five(10 * n + 5).ok_or_else(|| fail(format!("{} does not end in five", 10 * n + 5))))
            .collect::<fmn::Result<_>>()?;
        let deep = fives[0].number % 10;
        let small_area = the_move_of(fives[0].number, deep)?.corner();
        let sweep = Sweep::build(&kit, stage, deep, small_area, shown::SWEEP)?;
        let mut readouts = Vec::new();
        for e in &fives {
            let hundreds_len = num(e.hundreds).len();
            let product = eq(
                &kit,
                stage,
                &format!(r"{} \times {} = {}", e.n, e.n + 1, e.hundreds),
                WORK_PT,
                v(COLUMN_X, PRODUCT_Y),
            )?;
            let n = glyphs(stage, product).len();
            paint(stage, product, n - hundreds_len..n, blue);
            let square = eq(
                &kit,
                stage,
                &format!("{}^2 = {}", e.number, e.square),
                SQUARE_PT,
                v(COLUMN_X, SQUARE_Y),
            )?;
            fit_width(stage, square, COLUMN_ROOM);
            let n = glyphs(stage, square).len();
            let tail = num(e.square).len() - hundreds_len;
            paint(stage, square, n - tail - hundreds_len..n - tail, blue);
            paint(stage, square, n - tail..n, orange);
            readouts.push([product, square]);
        }
        // what 03_2 left: its card and working, the small replay and its labels
        let mut gone = first_card.mobs();
        gone.push(product);
        gone.extend(figure.square());
        gone.extend([figure.seam, figure.corner, figure.corner_cut]);
        gone.extend(small_labels);
        fade_out_all(stage, &gone, 0.4)?;
        let mut ins = sweep.all.clone();
        ins.extend(readouts[0]);
        fade_in_all(stage, &ins, 0.5)?;
        // at each step the numbers of the next stop come first, and the
        // picture grows to them
        for k in 1..fives.len() {
            let (rt, rest) = if k < SLOW_STOPS { (0.6, 0.4) } else { (0.45, 0.15) };
            let mut anims: Vec<Box<dyn Animation>> = vec![stage.prepare(tween(sweep.front, fives[k].n as f64, rt)?)?];
            for &m in &readouts[k - 1] {
                let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(0.2))?);
            }
            for &m in &readouts[k] {
                let a = fade_in(stage.arena_mut(), m, ORIGIN, 1.0)?;
                anims.push(stage.prepare(a.rt(0.2))?);
            }
            stage.play_prepared(anims)?;
            pause(stage, rest)?;
        }
        line.wait_for(stage, "the strip is five deep")?;
        play!(stage;
            fmn::animation::indicate(stage.arena_mut(), sweep.standing, 1.1, Some(rgb(strip())))?.rt(0.8),
            fmn::animation::indicate(stage.arena_mut(), sweep.deep_label, 1.5, Some(rgb(strip())))?.rt(0.8),
        );
        line.wait_for(stage, "the small square")?;
        play!(stage;
            fmn::animation::indicate(stage.arena_mut(), sweep.small, 1.3, Some(rgb(orange)))?.rt(0.8),
            fmn::animation::indicate(stage.arena_mut(), sweep.small_label, 1.5, Some(rgb(orange)))?.rt(0.8),
        );

        // 03_6: in letters: the numbers of 35² = 100 × 3 × 4 + 25 turn into
        // (10n + 5)² = 100 × n × (n + 1) + 25
        let line = say(&kit, stage, "03_6")?;
        let five = ends_in_five(shown::FIVE).ok_or_else(|| fail(format!("{} does not end in five", shown::FIVE)))?;
        let m = the_move_of(five.number, five.number % 10)?;
        let hundred = m.rectangle() / five.hundreds;
        let front = num(five.n);
        let numbers_src = format!(
            r"{}^2 = {hundred} \times {front} \times {} + {}",
            five.number,
            five.n + 1,
            m.corner()
        );
        let at = v(0.0, FORMULA_Y);
        let (numbers, numbers_spans) = kit.tex_spans(stage, &numbers_src, FORMULA_PT, ink, &[], at)?;
        let (letters, letters_spans) = kit.tex_spans(stage, LETTERS, FORMULA_PT, ink, &[], at)?;
        // both lines at one size, the wider one fitting the frame
        let widest = stage.get_width(letters).max(stage.get_width(numbers));
        if widest > FORMULA_ROOM {
            for t in [numbers, letters] {
                stage.scale(t, FORMULA_ROOM / widest);
                stage.move_to(t, at, ORIGIN);
            }
        }
        // the number in front in the hundreds' colour, the 25 in the corner's
        let corner_len = num(m.corner()).len();
        let numbers_n = numbers_spans.entries.len();
        let fronts_at = [
            within(&numbers_spans, &format!("{}^", five.number))?
                .into_iter()
                .take(front.len())
                .collect::<Vec<_>>(),
            within(&numbers_spans, &format!(r"\times {front} \times"))?
                .into_iter()
                .skip(1)
                .take(front.len())
                .collect(),
        ];
        paint(stage, numbers, fronts_at.concat(), blue);
        paint(stage, numbers, numbers_n - corner_len..numbers_n, orange);
        let letters_n = letters_spans.entries.len();
        let letter_at = keyed(&letters_spans, LETTER);
        paint(stage, letters, letter_at.iter().copied(), blue);
        paint(stage, letters, letters_n - corner_len..letters_n, orange);
        let mut gone = sweep.all.clone();
        gone.extend(readouts.last().into_iter().flatten());
        swap(stage, &gone, &[numbers], 0.6)?;
        // where 10n + 5 comes from, under the numbers before they turn into
        // letters (R2, M7)
        let ten = (five.number - five.number % 10) / five.n;
        let split_src = format!(r"{} = {ten} \times {front} + {}", five.number, five.number % 10);
        let split_at = v(0.0, stage.get_bottom(numbers)[1] - 0.7);
        let split = kit.tex(stage, &split_src, SPLIT_PT, ink, &[(&front, blue)], split_at)?;
        play!(stage; fade_in(stage.arena_mut(), split, v(0.0, 0.1), 1.0)?.rt(0.5));
        line.wait_for(stage, "front n")?;
        let morph = TransformMatchingTex::new(
            stage.arena_mut(),
            numbers,
            letters,
            &numbers_spans,
            &fresh_keys(&letters_spans, &LETTERS_NEW)?,
        )?;
        play!(stage;
            morph.rt(1.5),
            fade_out(stage.arena_mut(), split, ORIGIN, 1.0)?.rt(0.6),
        );
        let all = glyphs(stage, letters);
        let pick = |idx: &[usize]| idx.iter().filter_map(|&i| all.get(i).copied()).collect::<Vec<Mob>>();
        let first_n = pick(&letter_at[..1.min(letter_at.len())]);
        if first_n.is_empty() {
            return Err(fail(format!("{LETTERS} has no {LETTER}")));
        }
        let low = bottom_of(stage, &all);
        let n_at = middle_of(stage, &first_n)?;
        let label = kit.text(stage, IN_FRONT, 34.0, blue, v(n_at[0], low - 0.5))?;
        play!(stage; fade_in(stage.arena_mut(), label, v(0.0, 0.15), 1.0)?.rt(0.4));
        line.wait_for(stage, "the next number")?;
        let times = pick(&within(&letters_spans, LETTERS_HUNDREDS)?);
        let times_line = underline(stage, &times, low - 0.14, 0.04, blue);
        play!(stage; fmn::animation::grow_from_edge(stage.arena_mut(), times_line, LEFT, None)?.rt(0.6));
        line.wait_sentence(stage, 2)?;
        let last = pick(&(letters_n - corner_len..letters_n).collect::<Vec<_>>());
        let corner_line = underline(stage, &last, low - 0.14, 0.04, orange);
        play!(stage; fmn::animation::grow_from_edge(stage.arena_mut(), corner_line, LEFT, None)?.rt(0.5));
        // once it has settled: the reason, and the check
        let f = facts().map_err(fail)?;
        let read = |key: &str| -> fmn::Result<u64> {
            f.get(key)
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| fail(format!("fact {key} is not a whole number")))
        };
        let (holds, checked) = (read(CHECKED_HOLDS)?, read(CHECKED_ALL)?);
        if holds != checked {
            return Err(fail(format!("the trick held for {holds} of {checked} numbers")));
        }
        let note = lower_third(&kit, stage, "", &format!("{CHECKED}{}", thousands(read(CHECKED_KEY)?)))?;
        pause(stage, 0.4)?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, -0.1), 1.0)?.rt(0.5));

        // 03_7: only as easy as that one multiplication: 135² needs 13 × 14,
        // and its card goes to wait in the top right corner
        let line = say(&kit, stage, "03_7")?;
        line.wait_for(stage, "that one multiplication")?;
        play!(stage; fmn::animation::indicate(stage.arena_mut(), times_line, 1.15, Some(rgb(blue)))?.rt(0.8));
        line.wait_sentence(stage, 1)?;
        let [back, math] = waiting_card(&kit, stage, shown::FIVE_HARD)?;
        stage.scale_many_about_point(&[back, math], BIG, WAITING_AT);
        stage.shift_many(&[back, math], v(BIG_AT[0] - WAITING_AT[0], BIG_AT[1] - WAITING_AT[1]));
        // the line in letters goes whole: none of its glyphs was animated
        // on its own, so they are all still in it
        swap(
            stage,
            &[letters, label, times_line, corner_line, note],
            &[back, math],
            0.6,
        )?;
        line.wait_for(stage, "thirteen times fourteen")?;
        // the product it needs: the glyphs after "135²:" and before "= ?"
        let hard =
            ends_in_five(shown::FIVE_HARD).ok_or_else(|| fail(format!("{} does not end in five", shown::FIVE_HARD)))?;
        let card_glyphs = glyphs(stage, math);
        let from = num(hard.number).len() + 2;
        let to = card_glyphs.len().saturating_sub(2);
        if to <= from {
            return Err(fail(format!("the waiting card for {} has no product", hard.number)));
        }
        let needs = &card_glyphs[from..to];
        let y = bottom_of(stage, &card_glyphs) - 0.16;
        let needs_line = underline(stage, needs, y, 0.06, blue);
        play!(stage; fmn::animation::grow_from_edge(stage.arena_mut(), needs_line, LEFT, None)?.rt(0.5));
        line.wait_sentence(stage, 2)?;
        play!(stage;
            fly(back, WAITING_AT, 1.0 / BIG, 1.0)?,
            fly(math, WAITING_AT, 1.0 / BIG, 1.0)?,
            fade_out(stage.arena_mut(), needs_line, ORIGIN, 1.0)?.rt(0.4),
        );
        // the card ends exactly as chapters 04 and 05 draw it
        stage.remove_from_scene(back);
        stage.remove_from_scene(math);
        let waiting = waiting_card(&kit, stage, shown::FIVE_HARD)?;
        for m in waiting {
            stage.add_to_scene(m)?;
        }
        kit.music_breath(stage, super::draw::END_HOLD_S)?;
        // the card stays: chapters 04 and 05 start with it in its corner
        fade_all_but(stage, &waiting, 0.8)?;
        kit.finish(stage)
    }
}
