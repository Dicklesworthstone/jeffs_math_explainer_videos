//! Chapter 06, padlocks: the older way to get any three of five. Ten
//! padlocks go on the vault's door, one for each pair of trustees, in two
//! rows of five (06_1). Each lock's keys go to the three trustees outside
//! its pair, lock 4·5's first (06_2), then the other nine, until every
//! trustee carries six and the trio 1, 2, 3 opens all ten (06_3). The
//! question is how many locks stop trustees 4 and 5 (06_4). The answer is
//! one, their own, whose keys are with 1, 2 and 3 (06_5). One lock can't do
//! that job for two pairs: a lock serving 1·2 and 2·3 has keys only with 4
//! and 5 and shuts out 1, 2, 3 together, and one serving 1·2 and 3·4 shuts
//! out 1, 2, 3, 4. So ten locks is the fewest, and a search over every plan
//! agrees. Shamir's bigger version follows, quoted verbatim (06_6), with
//! its price, 462 locks and 252 keys per scientist against our 10 and 6
//! (06_7). Shamir's own answer then replaces the locks: the dial comes back
//! to the door, the envelope from 04 returns and opens, and each trustee
//! holds one number again (06_8).
//!
//! Every lock, key and count is the model's (`Padlocks`, `fewest_locks`),
//! and the shares are `envelope::dealt`. The door and lamp are the vault's,
//! drawn as draw.rs draws them. Without its dial, the door stands for the
//! older way; 06_8 brings the dial back as a `Face`, the vault's
//! proportions. As in chapters 02 to 05, emphasis never restyles or scales
//! typeset text: a lock or a card frame is pointed at by an outline of its
//! own, and only shapes (lock bodies, discs) change color or swell.

use explainer_kit::animatic::wrap;
use explainer_kit::{Kit, Palette, fade_all, hex, tween, v};
use fmn::prelude::*;

use super::draw::{
    DIAL_R, DIAL_Y, DOOR_H, DOOR_O, DOOR_W, DOOR_X, DOOR_Y, LAMP_R, LAMP_Y, READ_CPS, Rail, bundle, card, disc,
    fade_in_all, fade_in_lagged, fail, frame_at, line_style, panel, read_by, recolor, recolorings, say, stroke, swap,
    swaps, think, wait_until,
};
use super::envelope::{ENV_H, ENV_W, ENV_X, ENV_Y, Envelope, PAD, dealt, frame_halos, outline, swelling};
use super::two_keys::{Face, glow, glowing, leaving, moving, text_halo};
use crate::facts::{SHAMIR_NEEDED, SHAMIR_SCIENTISTS};
use crate::model::{Dial, OPENING_TRIO, PAIR, Padlocks, THRESHOLD, TRUSTEES, combinations, fewest_locks};
use crate::palette;

pub struct ThePadlocks {
    pub kit: Kit,
}

// ----------------------------------------------------------------- words

/// Liu's problem as Shamir sets it out (p. 612, section 1), verbatim, and
/// the card's source line.
const QUOTE: &str = "Eleven scientists are working on a secret project. They wish to lock up the documents in a \
                     cabinet so that the cabinet can be opened if and only if six or more of the scientists are \
                     present.";
const QUOTE_SOURCE: &str = "Shamir, How to Share a Secret, Comm. ACM 22(11), 1979, p. 612, introducing \
                            Liu's problem (C. L. Liu, 1968)";
/// Shamir's next sentence, verbatim: the counts are his.
const ANSWER: &str = "It is not hard to show that the minimal solution uses 462 locks and 252 keys per scientist.";
const ANSWER_SOURCE: &str = "Shamir, p. 612";
/// Property (1) of his scheme (p. 613), verbatim.
const PIECE: &str = "The size of each piece does not exceed the size of the original data.";
const PIECE_SOURCE: &str = "Shamir 1979, p. 613, property (1)";
/// The comparison's column heads.
const LOCKS_HEAD: &str = "locks";
const KEYS_HEAD: &str = "keys each";
/// The minimality card's first line (its second is counted).
const OWN_LOCK: &str = "every pair needs a lock of its own,";

/// The two merged locks of 06_5: two pairs that overlap, and two that
/// don't.
const MERGES: [[[u32; 2]; 2]; 2] = [[[1, 2], [2, 3]], [[1, 2], [3, 4]]];
/// The words each merged lock's failure lands on.
const CAUSES: [&str; 2] = ["or it would also stop the three", "or four trustees"];

// ------------------------------------------------------------------ layout

/// The locks: two rows of five on the door, the step between them, each
/// body's size, its shackle's radius and legs, how far an open shackle
/// lifts, and the label under each body.
const LOCK_DX: f64 = 0.58;
const LOCK_ROWS: [f64; 2] = [1.6, 0.2];
const LOCK_W: f64 = 0.4;
const LOCK_H: f64 = 0.32;
const SHACKLE_R: f64 = 0.12;
const SHACKLE_LEG: f64 = 0.08;
const SHACKLE_LINE: f64 = 4.0;
const LIFT: f64 = 0.12;
const LOCK_LABEL_SIZE: f64 = 18.0;
const LOCK_LABEL_DY: f64 = 0.27;
/// A lock's outline: from just under its label to over its lifted shackle.
const CELL_TOP: f64 = LOCK_H / 2.0 + SHACKLE_LEG + SHACKLE_R + LIFT;
const CELL_BOTTOM: f64 = LOCK_LABEL_DY + 0.06;
const CELL_W: f64 = LOCK_W + 2.0 * PAD + 0.06;
/// A key: its half length, its bow, its tooth and its line; the six slots
/// in a trustee's card frame, three across and two down.
const KEY_HALF: f64 = 0.075;
const KEY_BOW_R: f64 = 0.032;
const KEY_TOOTH: f64 = 0.04;
const KEY_LINE: f64 = 3.0;
const SLOT_DX: [f64; 3] = [-0.19, 0.0, 0.19];
const SLOT_DY: [f64; 2] = [0.09, -0.09];
/// How long keys take to appear at their lock, and their flights' lag.
const KEY_IN: f64 = 0.3;
const KEY_LAG: f64 = 0.15;
/// The key count, under the rail.
const COUNT_Y: f64 = -3.0;
const COUNT_SIZE: f64 = 24.0;
/// The rule, top right.
const RULE_X: f64 = 3.6;
const RULE_Y: f64 = 3.3;
const RULE_SIZE: f64 = 28.0;
/// The think ring (06_4).
const THINK_X: f64 = 3.6;
const THINK_Y: f64 = 1.3;
/// The merged locks (06_5): each row's lock, its two labels either side
/// under it, and its two lines of words.
const MERGE_X: f64 = 1.15;
const MERGE_ROWS: [f64; 2] = [2.15, 0.45];
const MERGE_LABEL_DX: f64 = 0.27;
const MERGE_TEXT_X: f64 = 1.85;
const MERGE_TEXT_DY: f64 = 0.2;
const MERGE_SIZE: f64 = 24.0;
/// The minimality card's two lines.
const FEWEST_X: f64 = 3.6;
const FEWEST_Y: [f64; 2] = [-0.55, -0.95];
const FEWEST_SIZE: f64 = 24.0;
/// The quote cards: centred across at `QUOTE_X`, the big one's top.
const QUOTE_X: f64 = 3.6;
const QUOTE_TOP: f64 = 3.72;
const CARD_BACK: &str = "#1A2130";
const CARD_O: f64 = 0.94;
const CARD_TOP_PAD: f64 = 0.28;
const CARD_GAP: f64 = 0.14;
const CARD_BOTTOM_PAD: f64 = 0.24;
const CARD_SIDE_PAD: f64 = 0.3;
/// The comparison under the quote (06_7): its rows' step, the label column's
/// left edge and the two number columns' right edges, and its sizes.
const TABLE_DY: f64 = 0.5;
const TABLE_HEAD_DY: f64 = 0.32;
const TABLE_LABEL_X: f64 = 0.5;
const LOCKS_RIGHT: f64 = 4.4;
const KEYS_RIGHT: f64 = 6.2;
const HEAD_SIZE: f64 = 20.0;
const ROW_SIZE: f64 = 22.0;
const NUMBER_SIZE: f64 = 30.0;
const CHOOSE_SIZE: f64 = 18.0;
const CHOOSE_DY: f64 = 0.4;
const ANSWER_DY: f64 = 0.32;
/// How long each count-up takes.
const COUNT_RT: f64 = 1.2;
/// 06_8: the envelope returns this much lower than in 04, so its open flap
/// stays inside the frame; the flap's rise; the property card's gap under it.
const ENV_DROP: f64 = 0.8;
const FLAP_IN: f64 = 0.06;
const OPEN_RISE: f64 = 0.3;
const PIECE_GAP: f64 = 0.3;
/// A ring round the dial (06_8), outside its rim.
const DIAL_RING: f64 = 0.14;

/// A quote card's type: the words' size, line step and wrap, and the
/// source's.
struct QuoteStyle {
    size: f64,
    line: f64,
    chars: usize,
    source_size: f64,
    source_line: f64,
    source_chars: usize,
}

const BIG: QuoteStyle = QuoteStyle {
    size: 24.0,
    line: 0.37,
    chars: 40,
    source_size: 17.0,
    source_line: 0.27,
    source_chars: 58,
};

const SMALL: QuoteStyle = QuoteStyle {
    size: 19.0,
    line: 0.3,
    chars: 50,
    source_size: 16.0,
    source_line: 0.26,
    source_chars: 50,
};

/// More lines than any card here needs: a quote is never cut short.
const MAX_LINES: usize = 12;

// ------------------------------------------------------------ the numbers

/// A lock's name: the pair it stops, "1·2".
fn lock_name(group: &[u32]) -> String {
    group.iter().map(ToString::to_string).collect::<Vec<_>>().join("·")
}

/// "5", "4 and 5", "1, 2 and 3".
fn and_list(xs: &[u32]) -> String {
    let words: Vec<String> = xs.iter().map(ToString::to_string).collect();
    match words.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
    }
}

/// "1, 2, 3".
fn listed(xs: &[u32]) -> String {
    xs.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
}

/// The rule card.
fn rule(k: usize, n: usize) -> String {
    format!("any {k} of {n}")
}

/// The key count card.
fn key_count(each: usize, total: usize) -> String {
    format!("{each} keys each, {total} keys in all")
}

/// What one lock stopping both pairs `p` and `q` must do: give its keys only
/// to the trustees outside both, so everyone in either is shut out together.
/// (holders, shut out)
fn merged(p: &[u32], q: &[u32]) -> (Vec<u32>, Vec<u32>) {
    let shut: Vec<u32> = TRUSTEES
        .iter()
        .copied()
        .filter(|t| p.contains(t) || q.contains(t))
        .collect();
    let holders = TRUSTEES.iter().copied().filter(|t| !shut.contains(t)).collect();
    (holders, shut)
}

fn holders_line(holders: &[u32]) -> String {
    format!("keys only with {}", and_list(holders))
}

fn shut_line(shut: &[u32]) -> String {
    format!("{} together: shut out", listed(shut))
}

/// The minimality card's second line.
fn fewest_line(pairs: usize, fewest: usize) -> String {
    format!("so {pairs} pairs need at least {fewest} locks")
}

/// The comparison's row labels.
fn ours_label(k: usize, n: usize) -> String {
    format!("any {k} of {n} trustees")
}

fn theirs_label(k: usize, n: u32) -> String {
    format!("any {k} of {n} scientists")
}

/// "= C(11, 5)": how many groups of `k` there are among `n`.
fn choose(n: u32, k: usize) -> String {
    format!("= C({n}, {k})")
}

/// For every lock (by index) and every trustee holding its key: the slot
/// that key takes among the trustee's own keys, in lock order.
fn key_slots(locks: &Padlocks) -> Vec<(usize, u32, usize)> {
    let mut out = Vec::new();
    for (i, group) in locks.locks().iter().enumerate() {
        for &t in TRUSTEES.iter().filter(|t| !group.contains(t)) {
            let slot = locks.locks()[..i].iter().filter(|g| !g.contains(&t)).count();
            out.push((i, t, slot));
        }
    }
    out
}

// ---------------------------------------------------------------- devices

/// Lock `i`'s body centre: two rows of five on the door.
fn lock_at(i: usize) -> Vec3 {
    v(DOOR_X + LOCK_DX * ((i % 5) as f64 - 2.0), LOCK_ROWS[i / 5])
}

/// Slot `i` of trustee `t`'s six keys, in its card frame.
fn slot_at(t: u32, i: usize) -> Vec3 {
    let f = frame_at(t);
    v(f[0] + SLOT_DX[i % 3], f[1] + SLOT_DY[i / 3])
}

/// A padlock: a body that turns green when open, a shackle that lifts, and
/// the name of the pair it stops under it.
struct Lock {
    group: Vec<u32>,
    whole: Mob,
    body: Mob,
    shackle: Mob,
    /// Where the shut shackle's centre is, as built.
    shackle_home: Vec3,
    label: Option<Mob>,
    centre: Vec3,
}

impl Lock {
    /// A lock at `centre` stopping `group`, its label `name` (no label when
    /// `None`: a merged lock's labels come from the grid). Detached.
    fn build(kit: &Kit, stage: &mut Stage<'_>, group: &[u32], name: Option<&str>, centre: Vec3) -> fmn::Result<Self> {
        let body = panel(stage, Palette::dim(), 1.0, centre, LOCK_W, LOCK_H)?;
        let (x, top) = (centre[0], centre[1] + LOCK_H / 2.0);
        let mut points = vec![v(x - SHACKLE_R, top), v(x - SHACKLE_R, top + SHACKLE_LEG)];
        let n = 16;
        for i in 0..=n {
            let a = std::f64::consts::PI * (1.0 - f64::from(i) / f64::from(n));
            points.push(v(x + SHACKLE_R * a.cos(), top + SHACKLE_LEG + SHACKLE_R * a.sin()));
        }
        points.push(v(x + SHACKLE_R, top));
        let shackle = stroke(stage, points, Palette::dim(), SHACKLE_LINE);
        let shackle_home = stage.get_center(shackle);
        let mut members = vec![shackle, body];
        let label = match name {
            Some(s) => {
                let m = kit.text(
                    stage,
                    s,
                    LOCK_LABEL_SIZE,
                    Palette::ink(),
                    v(x, centre[1] - LOCK_LABEL_DY),
                )?;
                members.push(m);
                Some(m)
            }
            None => None,
        };
        let whole = bundle(stage, &members)?;
        Ok(Self {
            group: group.to_vec(),
            whole,
            body,
            shackle,
            shackle_home,
            label,
            centre,
        })
    }

    /// The shackle lifting (or dropping back) and the body turning green
    /// (or grey), prepared, as one move of run time 1 (a lagged group sets
    /// the real time).
    fn springing(&self, stage: &mut Stage<'_>, open: bool) -> fmn::Result<Box<dyn Animation>> {
        let (lift, color) = if open {
            (LIFT, palette::open())
        } else {
            (0.0, Palette::dim())
        };
        let to = v(self.shackle_home[0], self.shackle_home[1] + lift);
        let both = vec![
            moving(stage, self.shackle, to, 1.0)?,
            stage.prepare(recolor(self.body, color, 1.0)?)?,
        ];
        let g = AnimationGroup::new(stage.arena_mut(), both)?;
        stage.prepare(g)
    }

    /// An outline round the whole lock, label to lifted shackle, in `color`.
    /// Detached.
    fn halo(&self, stage: &mut Stage<'_>, color: Srgb) -> fmn::Result<Mob> {
        let (top, bottom) = (self.centre[1] + CELL_TOP, self.centre[1] - CELL_BOTTOM);
        outline(
            stage,
            v(self.centre[0], (top + bottom) / 2.0),
            CELL_W,
            top - bottom + 2.0 * PAD,
            color,
        )
    }
}

/// Locks springing open (or shut), one after another, the whole taking `rt`.
fn spring(stage: &mut Stage<'_>, locks: &[&Lock], open: bool, rt: f64, lag: f64) -> fmn::Result<()> {
    if locks.is_empty() {
        return Ok(());
    }
    let mut anims = Vec::with_capacity(locks.len());
    for l in locks {
        anims.push(l.springing(stage, open)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, lag)?.with_run_time(rt);
    stage.play(g)?;
    Ok(())
}

/// A key, in the share color, centred at `at`: a round bow and a blade with
/// a tooth. Detached.
fn key(stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let left = at[0] - KEY_HALF;
    let bow = disc(stage, palette::share(), v(left + KEY_BOW_R, at[1]), KEY_BOW_R);
    let blade = stroke(
        stage,
        vec![
            v(left + 2.0 * KEY_BOW_R, at[1]),
            v(at[0] + KEY_HALF, at[1]),
            v(at[0] + KEY_HALF, at[1] - KEY_TOOTH),
        ],
        palette::share(),
        KEY_LINE,
    );
    bundle(stage, &[bow, blade])
}

/// Keys appearing at their locks, then flying one after another to their
/// slots, the flights taking `rt`. Returns the keys.
fn deal_keys(stage: &mut Stage<'_>, flights: &[(Vec3, Vec3)], rt: f64) -> fmn::Result<Vec<Mob>> {
    let mut keys = Vec::with_capacity(flights.len());
    for &(from, _) in flights {
        keys.push(key(stage, from)?);
    }
    fade_in_all(stage, &keys, KEY_IN)?;
    let mut anims = Vec::with_capacity(keys.len());
    for (&k, &(_, to)) in keys.iter().zip(flights) {
        anims.push(moving(stage, k, to, 1.0)?);
    }
    let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), anims, KEY_LAG)?.with_run_time(rt);
    stage.play(g)?;
    Ok(keys)
}

/// A quote card: `words` in italic lines of at most `style.chars`
/// characters, the source in dim lines under them, on a dark panel, centred
/// across at `x` with its top at `top`. Returns the card, one bundle, and
/// its bottom edge. Detached.
fn quote_card(
    kit: &Kit,
    stage: &mut Stage<'_>,
    style: &QuoteStyle,
    words: &str,
    source: &str,
    x: f64,
    top: f64,
) -> fmn::Result<(Mob, f64)> {
    let lines = wrap(words, style.chars, MAX_LINES);
    let sources = wrap(source, style.source_chars, MAX_LINES);
    let h = card_height(style, lines.len(), sources.len());
    let mut texts = Vec::with_capacity(lines.len() + sources.len());
    let mut y = top - CARD_TOP_PAD - style.line / 2.0;
    for l in &lines {
        texts.push(kit.text_styled(stage, l, style.size, Palette::ink(), v(x, y), false, true)?);
        y -= style.line;
    }
    y += style.line / 2.0 - CARD_GAP - style.source_line / 2.0;
    for l in &sources {
        texts.push(kit.text(stage, l, style.source_size, Palette::dim(), v(x, y))?);
        y -= style.source_line;
    }
    let widest = texts.iter().map(|&m| stage.get_width(m)).fold(0.0, f64::max);
    let back = panel(
        stage,
        hex(CARD_BACK),
        CARD_O,
        v(x, top - h / 2.0),
        widest + 2.0 * CARD_SIDE_PAD,
        h,
    )?;
    let mut all = vec![back];
    all.extend(texts);
    Ok((bundle(stage, &all)?, top - h))
}

/// A quote card's height for `lines` lines of words and `sources` of source.
fn card_height(style: &QuoteStyle, lines: usize, sources: usize) -> f64 {
    CARD_TOP_PAD + style.line * lines as f64 + CARD_GAP + style.source_line * sources as f64 + CARD_BOTTOM_PAD
}

/// A ring of radius `r` round `at`, in `color`. Detached.
fn ring(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Mob {
    let c = Circle::new()
        .radius(r)
        .style(line_style(color, 3.0))
        .build()
        .moved_to(at);
    stage.arena_mut().add(c)
}

/// When a viewer still reading until `busy` has also read `text`, fully in
/// now: the cards of one argument are read one after another.
fn read_on(stage: &Stage<'_>, busy: f64, text: &str) -> f64 {
    read_by(stage, text).max(busy + text.chars().count() as f64 / READ_CPS)
}

impl SceneConstruct for ThePadlocks {
    fn name(&self) -> &str {
        "06_padlocks"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let dial = Dial::vault();
        // the plan, and the facts the pictures rest on, checked before
        // anything is drawn
        let padlocks = Padlocks::new(&TRUSTEES, THRESHOLD);
        let pairs = combinations(&TRUSTEES, THRESHOLD - 1);
        let each: Vec<usize> = TRUSTEES.iter().map(|&t| padlocks.keys(t)).collect();
        let keys_each = each[0];
        if each.iter().any(|&k| k != keys_each) {
            return Err(fail(format!("the trustees carry {each:?} keys, not the same number")));
        }
        if !padlocks.opens(&OPENING_TRIO) {
            return Err(fail(format!("trustees {OPENING_TRIO:?} do not open every lock")));
        }
        if padlocks.stuck_at(&PAIR) != 1 {
            return Err(fail(format!(
                "trustees {PAIR:?} are stopped by {} locks, not one",
                padlocks.stuck_at(&PAIR)
            )));
        }
        let fewest = fewest_locks(&TRUSTEES, THRESHOLD);
        if fewest != Some(pairs.len()) || padlocks.locks().len() != pairs.len() {
            return Err(fail(format!(
                "the fewest locks are {fewest:?}, the plan has {}, and there are {} pairs",
                padlocks.locks().len(),
                pairs.len()
            )));
        }
        for [p, q] in MERGES {
            let (_, shut) = merged(&p, &q);
            if shut.len() < THRESHOLD {
                return Err(fail(format!("a lock for {p:?} and {q:?} shuts out only {shut:?}")));
            }
        }
        let scientists: Vec<u32> = (1..=SHAMIR_SCIENTISTS).collect();
        let liu = Padlocks::new(&scientists, SHAMIR_NEEDED);
        let slots = key_slots(&padlocks);

        // 06_1: the older way: ten padlocks on the door, one for each pair
        let line = say(&kit, stage, "06_1")?;
        let door = panel(stage, palette::ghost(), DOOR_O, v(DOOR_X, DOOR_Y), DOOR_W, DOOR_H)?;
        let lamp = disc(stage, palette::ghost(), v(DOOR_X, LAMP_Y), LAMP_R);
        let rail = Rail::build(&kit, stage, &TRUSTEES)?;
        fade_in_all(stage, &[door, lamp], 0.5)?;
        rail.reveal(stage, 0.6)?;
        line.wait_for(stage, "any three of five")?;
        let rule_card = kit.text(
            stage,
            &rule(THRESHOLD, TRUSTEES.len()),
            RULE_SIZE,
            Palette::ink(),
            v(RULE_X, RULE_Y),
        )?;
        fade_in_all(stage, &[rule_card], 0.4)?;
        line.wait_for(stage, "Put ten padlocks on the door")?;
        let mut locks = Vec::with_capacity(padlocks.locks().len());
        for (i, group) in padlocks.locks().iter().enumerate() {
            locks.push(Lock::build(&kit, stage, group, Some(&lock_name(group)), lock_at(i))?);
        }
        let wholes: Vec<Mob> = locks.iter().map(|l| l.whole).collect();
        fade_in_lagged(stage, &wholes, 2.0, 0.3)?;
        line.wait_for(stage, "one for each pair of trustees")?;
        // the first lock and the pair it is named for
        let mut halos = vec![locks[0].halo(stage, Palette::ink())?];
        halos.extend(frame_halos(stage, &locks[0].group, palette::share())?);
        glow(stage, &halos, 1.0)?;
        line.wait_end(stage)?;

        // 06_2: each lock's keys go to everyone outside its pair; lock 4·5's
        // to 1, 2 and 3
        let line = say(&kit, stage, "06_2")?;
        let own = locks
            .iter()
            .position(|l| l.group == PAIR)
            .ok_or_else(|| fail(format!("no lock for {PAIR:?}")))?;
        line.wait_for(stage, "The lock for four and five")?;
        let own_halo = locks[own].halo(stage, palette::pair())?;
        let mut anims = swaps(stage, &[], &[own_halo], 0.4)?;
        anims.extend(recolorings(stage, &rail.tint(&PAIR, palette::pair()), 0.4)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, "has its keys with one, two and three")?;
        let flights: Vec<(Vec3, Vec3)> = slots
            .iter()
            .filter(|&&(i, ..)| i == own)
            .map(|&(i, t, s)| (locks[i].centre, slot_at(t, s)))
            .collect();
        let holders: Vec<u32> = slots.iter().filter(|&&(i, ..)| i == own).map(|&(_, t, _)| t).collect();
        let mut keys = deal_keys(stage, &flights, 1.0)?;
        let halos = frame_halos(stage, &holders, palette::share())?;
        glow(stage, &halos, 0.6)?;
        line.wait_end(stage)?;
        let mut anims = leaving(stage, &[own_halo], ORIGIN, 0.4)?;
        anims.extend(recolorings(stage, &rail.tint(&PAIR, palette::share()), 0.4)?);
        stage.play_prepared(anims)?;

        // 06_3: the other nine locks' keys; six each; any three open all ten
        let line = say(&kit, stage, "06_3")?;
        let flights: Vec<(Vec3, Vec3)> = slots
            .iter()
            .filter(|&&(i, ..)| i != own)
            .map(|&(i, t, s)| (locks[i].centre, slot_at(t, s)))
            .collect();
        keys.extend(deal_keys(stage, &flights, 2.2)?);
        line.wait_for(stage, "six keys")?;
        let count = kit.text(
            stage,
            &key_count(keys_each, keys.len()),
            COUNT_SIZE,
            Palette::ink(),
            v(DOOR_X, COUNT_Y),
        )?;
        fade_in_all(stage, &[count], 0.4)?;
        line.wait_for(stage, "Any three trustees together")?;
        let mut anims = recolorings(stage, &rail.tint(&OPENING_TRIO, palette::trio()), 0.4)?;
        let halos = frame_halos(stage, &OPENING_TRIO, palette::trio())?;
        anims.extend(glowing(stage, &halos, 0.6)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, "hold a key to every lock")?;
        let all: Vec<&Lock> = locks.iter().collect();
        spring(stage, &all, true, 1.5, 0.2)?;
        line.wait_for(stage, "because no pair contains all three of them")?;
        // locks 1·2 and 4·5, each with the first of the trio outside its
        // pair, who holds its key
        for i in [0, own] {
            let outside = OPENING_TRIO
                .iter()
                .copied()
                .find(|t| !locks[i].group.contains(t))
                .ok_or_else(|| fail(format!("the trio sits inside lock {:?}'s pair", locks[i].group)))?;
            let halos = vec![
                locks[i].halo(stage, palette::trio())?,
                frame_halos(stage, &[outside], palette::trio())?[0],
            ];
            glow(stage, &halos, 0.8)?;
        }
        line.wait_end(stage)?;

        // 06_4: how many locks stop trustees 4 and 5?
        let line = say(&kit, stage, "06_4")?;
        spring(stage, &all, false, 0.8, 0.1)?;
        let mut anims = recolorings(stage, &rail.tint(&OPENING_TRIO, palette::share()), 0.4)?;
        anims.extend(recolorings(stage, &rail.tint(&PAIR, palette::pair()), 0.4)?);
        let halos = frame_halos(stage, &PAIR, palette::pair())?;
        anims.extend(glowing(stage, &halos, 0.6)?);
        stage.play_prepared(anims)?;
        line.wait_end(stage)?;
        let question = think(&kit, stage, v(THINK_X, THINK_Y))?;

        // 06_5: just one, their own; no lock serves two pairs; so ten
        let line = say(&kit, stage, "06_5")?;
        // just one: as the "?" goes, the nine others spring open for 4 and
        // 5, and their own lock is outlined, in one move
        let theirs: Vec<&Lock> = locks
            .iter()
            .filter(|l| PAIR.iter().any(|t| !l.group.contains(t)))
            .collect();
        let mut opening = Vec::with_capacity(theirs.len());
        for l in &theirs {
            opening.push(l.springing(stage, true)?);
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), opening, 0.15)?.with_run_time(1.2);
        let mut anims = vec![stage.prepare(g)?];
        anims.extend(leaving(stage, &[question], ORIGIN, 0.3)?);
        let own_halo = locks[own].halo(stage, palette::pair())?;
        anims.extend(swaps(stage, &[], &[own_halo], 0.5)?);
        stage.play_prepared(anims)?;
        line.wait_for(stage, "their own")?;
        let halos = frame_halos(stage, &holders, palette::share())?;
        glow(stage, &halos, 0.8)?;
        line.wait_for(stage, "And no lock can do that job")?;
        spring(stage, &theirs, false, 0.6, 0.05)?;
        let mut anims = leaving(stage, &[own_halo], ORIGIN, 0.4)?;
        anims.extend(recolorings(stage, &rail.tint(&PAIR, palette::share()), 0.4)?);
        stage.play_prepared(anims)?;
        // one lock for two pairs: the overlapping 1·2 and 2·3, and the
        // separate 1·2 and 3·4. Both merged locks come in together as two
        // pairs are named, the two pairs' labels crowding onto each: copies
        // that lie exactly on the grid's labels, so they come in unseen,
        // move over
        let mut demo = Vec::new();
        let mut merged_locks = Vec::with_capacity(MERGES.len());
        let mut anims = Vec::new();
        for (r, [p, q]) in MERGES.iter().enumerate() {
            let at = v(MERGE_X, MERGE_ROWS[r]);
            let lock = Lock::build(&kit, stage, &[], None, at)?;
            anims.extend(swaps(stage, &[], &[lock.whole], 0.5)?);
            for (side, pair) in [-1.0, 1.0].into_iter().zip([p, q]) {
                let label = locks
                    .iter()
                    .find(|l| l.group == pair)
                    .and_then(|l| l.label)
                    .ok_or_else(|| fail(format!("no labelled lock for {pair:?}")))?;
                let from = stage.get_center(label);
                let copy = kit.text(stage, &lock_name(pair), LOCK_LABEL_SIZE, Palette::ink(), from)?;
                stage.add_to_scene(copy)?;
                let to = v(at[0] + side * MERGE_LABEL_DX, at[1] - LOCK_LABEL_DY);
                anims.push(moving(stage, copy, to, 0.9)?);
                demo.push(copy);
            }
            merged_locks.push(lock);
        }
        stage.play_prepared(anims)?;
        // each merged lock's failure on its own words, the rows read one
        // after the other
        let mut shut_out = Vec::new();
        let mut read = 0.0;
        for (r, ([p, q], lock)) in MERGES.iter().zip(&merged_locks).enumerate() {
            let at = v(MERGE_X, MERGE_ROWS[r]);
            line.wait_for(stage, CAUSES[r])?;
            let (holders, shut) = merged(p, q);
            let words = [
                kit.text_left(
                    stage,
                    &holders_line(&holders),
                    MERGE_SIZE,
                    Palette::ink(),
                    v(MERGE_TEXT_X, at[1] + MERGE_TEXT_DY),
                )?,
                kit.text_left(
                    stage,
                    &shut_line(&shut),
                    MERGE_SIZE,
                    palette::leak(),
                    v(MERGE_TEXT_X, at[1] - MERGE_TEXT_DY),
                )?,
            ];
            let mut anims = swaps(stage, &[], &words, 0.4)?;
            anims.extend(recolorings(stage, &rail.tint(&shut, palette::trio()), 0.4)?);
            let halos = frame_halos(stage, &shut, palette::leak())?;
            anims.extend(glowing(stage, &halos, 0.8)?);
            anims.push(swelling(stage, lock.body, 1.15, palette::leak(), 0.8)?);
            stage.play_prepared(anims)?;
            read = read_on(stage, read, &format!("{} {}", holders_line(&holders), shut_line(&shut)));
            demo.push(lock.whole);
            demo.extend(words);
            shut_out = shut;
        }
        line.wait_for(stage, "So ten locks is the fewest")?;
        let fewest_words = [
            OWN_LOCK.to_owned(),
            fewest_line(pairs.len(), fewest.unwrap_or(pairs.len())),
        ];
        let mut fewest_card = Vec::with_capacity(2);
        for (text, y) in fewest_words.iter().zip(FEWEST_Y) {
            fewest_card.push(kit.text(stage, text, FEWEST_SIZE, Palette::ink(), v(FEWEST_X, y))?);
        }
        let grid = outline(
            stage,
            v(DOOR_X, (LOCK_ROWS[0] + CELL_TOP + LOCK_ROWS[1] - CELL_BOTTOM) / 2.0),
            4.0 * LOCK_DX + CELL_W,
            LOCK_ROWS[0] + CELL_TOP - LOCK_ROWS[1] + CELL_BOTTOM + 2.0 * PAD,
            Palette::ink(),
        )?;
        let mut anims = recolorings(stage, &rail.tint(&shut_out, palette::share()), 0.4)?;
        anims.extend(swaps(stage, &[], &fewest_card, 0.5)?);
        anims.extend(glowing(stage, &[grid], 1.2)?);
        stage.play_prepared(anims)?;
        let read = read_on(stage, read, &fewest_words.join(" "));
        line.wait_end(stage)?;
        wait_until(stage, read)?;

        // 06_6: Shamir's bigger version, verbatim
        let line = say(&kit, stage, "06_6")?;
        let mut going = demo;
        going.extend(fewest_card);
        going.push(rule_card);
        let anims = leaving(stage, &going, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        // the card comes in as the paper is named, so it is read while the
        // line and the next are spoken
        let (quote, bottom) = quote_card(&kit, stage, &BIG, QUOTE, QUOTE_SOURCE, QUOTE_X, QUOTE_TOP)?;
        fade_in_all(stage, &[quote], 0.6)?;
        let mut read = read_by(stage, &format!("{QUOTE} {QUOTE_SOURCE}"));
        line.wait_for(stage, "Any six of eleven scientists")?;
        let quote_halo = text_halo(stage, quote, Palette::dim())?;
        glow(stage, &[quote_halo], 1.0)?;
        line.wait_end(stage)?;

        // 06_7: the price: 462 locks and 252 keys each, against 10 and 6
        let line = say(&kit, stage, "06_7")?;
        let head_y = bottom - TABLE_HEAD_DY;
        let (ours_y, theirs_y) = (head_y - TABLE_DY, head_y - 2.0 * TABLE_DY);
        let mut table = Vec::new();
        for (text, right) in [(LOCKS_HEAD, LOCKS_RIGHT), (KEYS_HEAD, KEYS_RIGHT)] {
            let m = kit.text(stage, text, HEAD_SIZE, Palette::dim(), v(right, head_y))?;
            stage.move_to(m, v(right, head_y), RIGHT);
            table.push(m);
        }
        for (text, y) in [
            (ours_label(THRESHOLD, TRUSTEES.len()), ours_y),
            (theirs_label(SHAMIR_NEEDED, SHAMIR_SCIENTISTS), theirs_y),
        ] {
            table.push(kit.text_left(stage, &text, ROW_SIZE, Palette::ink(), v(TABLE_LABEL_X, y))?);
        }
        for (n, right) in [(padlocks.locks().len(), LOCKS_RIGHT), (keys_each, KEYS_RIGHT)] {
            let m = kit.text(stage, &n.to_string(), NUMBER_SIZE, Palette::ink(), v(right, ours_y))?;
            stage.move_to(m, v(right, ours_y), RIGHT);
            table.push(m);
        }
        let choose_y = theirs_y - CHOOSE_DY;
        // Shamir's sentence, the counts' source, comes in with the table.
        // Reserve its reading time after the longer question card: a
        // viewer cannot read both passages at once.
        let (answer, _) = quote_card(
            &kit,
            stage,
            &SMALL,
            ANSWER,
            ANSWER_SOURCE,
            QUOTE_X,
            choose_y - ANSWER_DY,
        )?;
        let mut arriving = table.clone();
        arriving.push(answer);
        fade_in_all(stage, &arriving, 0.5)?;
        read = read_on(stage, read, &format!("{ANSWER} {ANSWER_SOURCE}"));
        let mut counts = Vec::with_capacity(2);
        for (phrase, n, right) in [
            ("four hundred and sixty-two locks", liu.locks().len(), LOCKS_RIGHT),
            ("two hundred and fifty-two keys", liu.keys(1), KEYS_RIGHT),
        ] {
            line.wait_for(stage, phrase)?;
            let tracker = stage.add_value_tracker(0.0);
            counts.push(kit.live_number(
                stage,
                tracker,
                0,
                false,
                NUMBER_SIZE,
                Palette::ink(),
                v(right, theirs_y),
                RIGHT,
            )?);
            stage.play(tween(tracker, n as f64, COUNT_RT)?)?;
        }
        // the counts are groups of five: C(11, 5) locks, C(10, 5) keys each
        let k = SHAMIR_NEEDED - 1;
        let mut groups = Vec::with_capacity(2);
        for (text, right) in [
            (choose(SHAMIR_SCIENTISTS, k), LOCKS_RIGHT),
            (choose(SHAMIR_SCIENTISTS - 1, k), KEYS_RIGHT),
        ] {
            let m = kit.text(stage, &text, CHOOSE_SIZE, Palette::dim(), v(right, choose_y))?;
            stage.move_to(m, v(right, choose_y), RIGHT);
            groups.push(m);
        }
        fade_in_all(stage, &groups, 0.5)?;
        table.extend(groups);
        line.wait_end(stage)?;
        // the cards stay until they have been read
        wait_until(stage, read)?;

        // 06_8: Shamir's own answer: no locks; one number each, from the dial
        let line = say(&kit, stage, "06_8")?;
        let mut going = vec![quote, answer];
        going.extend(table);
        going.extend(counts);
        let anims = leaving(stage, &going, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        line.wait_for(stage, "the dealer's method")?;
        let envelope = Envelope::build(&kit, stage)?;
        for &m in envelope.parts() {
            let c = stage.get_center(m);
            stage.move_to(m, v(c[0], c[1] - ENV_DROP), ORIGIN);
        }
        fade_in_all(stage, envelope.parts(), 0.5)?;
        line.wait_for(stage, "with no locks at all")?;
        let mut going = keys;
        going.push(count);
        let anims = leaving(stage, &going, ORIGIN, 0.5)?;
        stage.play_prepared(anims)?;
        let mut lagged = Vec::with_capacity(locks.len());
        for l in &locks {
            stage.clear_updaters(l.whole, true);
            let a = fade_out(stage.arena_mut(), l.whole, ORIGIN, 1.0)?;
            lagged.push(stage.prepare(a)?);
        }
        let g = AnimationGroup::with_lag_ratio(stage.arena_mut(), lagged, 0.2)?.with_run_time(1.0);
        stage.play(g)?;
        let face = Face::build(&kit, stage, dial, v(DOOR_X, DIAL_Y), DIAL_R, Palette::dim(), 0)?;
        face.show(stage, 0.5)?;
        line.wait_for(stage, "Each trustee carries one number")?;
        // the envelope opens: flap and seal and the "sealed" note go, the
        // flap stands open over it
        let &[_, flap, seal, _, note] = envelope.parts() else {
            return Err(fail(format!(
                "the envelope has {} parts, not five",
                envelope.parts().len()
            )));
        };
        let top = ENV_Y - ENV_DROP + ENV_H / 2.0 - FLAP_IN;
        let (left, right) = (ENV_X - ENV_W / 2.0 + FLAP_IN, ENV_X + ENV_W / 2.0 - FLAP_IN);
        let open_flap = stroke(
            stage,
            vec![v(left, top), v(ENV_X, top + OPEN_RISE), v(right, top)],
            Palette::dim(),
            2.0,
        );
        swap(stage, &[flap, seal, note], &[], 0.4)?;
        fade_in_all(stage, &[open_flap], 0.4)?;
        let mut cards = Vec::with_capacity(TRUSTEES.len());
        for (t, y) in dealt(dial, &TRUSTEES) {
            cards.push(card(&kit, stage, t, &y.to_string(), palette::share())?);
        }
        fade_in_lagged(stage, &cards, 1.6, 0.35)?;
        line.wait_for(stage, "from the same dial as the secret")?;
        let (piece, _) = quote_card(
            &kit,
            stage,
            &SMALL,
            PIECE,
            PIECE_SOURCE,
            ENV_X,
            ENV_Y - ENV_DROP - ENV_H / 2.0 - PIECE_GAP,
        )?;
        let mut anims = swaps(stage, &[], &[piece], 0.5)?;
        let mut halos = frame_halos(stage, &TRUSTEES, palette::share())?;
        halos.push(ring(stage, v(DOOR_X, DIAL_Y), DIAL_R + DIAL_RING, palette::secret()));
        anims.extend(glowing(stage, &halos, 1.0)?);
        stage.play_prepared(anims)?;
        let read = read_by(stage, &format!("{PIECE} {PIECE_SOURCE}"));
        line.wait_end(stage)?;
        wait_until(stage, read)?;

        fade_all(stage, 0.6)?;
        kit.finish(stage)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};

    use super::super::draw::{FRAME_H, FRAME_W, FRAME_Y, RAIL_Y, THINK_R, TRUSTEE_R};
    use super::*;
    use crate::facts;
    use crate::model::csv;

    /// The production narration: registering chapter 06 brings its lines
    /// into script.tsv.
    const SCRIPT: &str = crate::SCRIPT;

    /// Generous text sizes, as the other chapters' tests: 0.0055 units a
    /// point a character wide and 0.0065 a point tall.
    fn text_w(text: &str, size: f64) -> f64 {
        0.0055 * size * text.chars().count() as f64
    }

    fn text_h(size: f64) -> f64 {
        0.0065 * size
    }

    /// Every phrase the scene waits for is in its line of the script.
    #[test]
    fn every_phrase_the_scene_waits_for_is_in_its_line() {
        let script = explainer_kit::Script::parse(SCRIPT).unwrap();
        for (id, phrase) in [
            ("06_1", "any three of five"),
            ("06_1", "Put ten padlocks on the door"),
            ("06_1", "one for each pair of trustees"),
            ("06_2", "The lock for four and five"),
            ("06_2", "has its keys with one, two and three"),
            ("06_3", "six keys"),
            ("06_3", "Any three trustees together"),
            ("06_3", "hold a key to every lock"),
            ("06_3", "because no pair contains all three of them"),
            ("06_5", "their own"),
            ("06_5", "And no lock can do that job"),
            ("06_5", "or it would also stop the three"),
            ("06_5", "or four trustees"),
            ("06_5", "So ten locks is the fewest"),
            ("06_6", "Any six of eleven scientists"),
            ("06_7", "four hundred and sixty-two locks"),
            ("06_7", "two hundred and fifty-two keys"),
            ("06_8", "the dealer's method"),
            ("06_8", "with no locks at all"),
            ("06_8", "Each trustee carries one number"),
            ("06_8", "from the same dial as the secret"),
        ] {
            let text = script.text(id).unwrap_or_else(|| panic!("no line {id}"));
            assert!(text.contains(phrase), "{id} has no {phrase:?}: {text}");
        }
    }

    /// What the scene draws is the facts': ten locks named by the pairs,
    /// six keys each and thirty in all, lock 4·5's keys with 1, 2 and 3,
    /// the trio opening every lock, each pair stopped by its own lock alone,
    /// ten the fewest any plan needs, Shamir's 462 and 252 (his sentence's
    /// own numbers too), and the shares 8, 3, 1, 2, 6.
    #[test]
    fn the_pictures_are_the_facts() {
        let f = facts::facts().unwrap();
        let locks = Padlocks::new(&TRUSTEES, THRESHOLD);
        assert_eq!(f.get("S-F3.locks"), Some(locks.locks().len().to_string().as_str()));
        let names: Vec<String> = locks.locks().iter().map(|g| lock_name(g)).collect();
        assert_eq!(
            names,
            ["1·2", "1·3", "1·4", "1·5", "2·3", "2·4", "2·5", "3·4", "3·5", "4·5"]
        );
        assert_eq!(f.get("S-F3.keys_each"), Some("6"));
        let slots = key_slots(&locks);
        assert_eq!(f.get("S-F3.keys_total"), Some(slots.len().to_string().as_str()));
        assert_eq!(key_count(6, slots.len()), "6 keys each, 30 keys in all");
        let own = locks.locks().iter().position(|g| g == &PAIR).unwrap();
        let holders: Vec<u32> = slots.iter().filter(|s| s.0 == own).map(|s| s.1).collect();
        assert_eq!(f.get("S-F3.pair_lock.holders"), Some(csv(&holders).as_str()));
        assert_eq!(and_list(&holders), "1, 2 and 3");
        assert!(locks.opens(&OPENING_TRIO));
        assert_eq!(f.get("S-F3.locks_a_pair_cannot_open"), Some("1"));
        assert_eq!(locks.stuck_at(&PAIR), 1);
        assert_eq!(f.get("S-F3.fewest_locks"), Some("10"));
        assert_eq!(fewest_locks(&TRUSTEES, THRESHOLD), Some(10));
        assert_eq!(rule(THRESHOLD, TRUSTEES.len()), "any 3 of 5");
        assert_eq!(fewest_line(10, 10), "so 10 pairs need at least 10 locks");
        let scientists: Vec<u32> = (1..=SHAMIR_SCIENTISTS).collect();
        let liu = Padlocks::new(&scientists, SHAMIR_NEEDED);
        assert_eq!(f.get("S-F3.shamir.locks"), Some(liu.locks().len().to_string().as_str()));
        assert_eq!(f.get("S-F3.shamir.keys_each"), Some(liu.keys(1).to_string().as_str()));
        for key in ["S-X1.shamir.locks", "S-X1.shamir.keys"] {
            let n = f.get(key).unwrap();
            assert!(
                ANSWER.contains(&format!(" {n} ")),
                "{key} {n} is not in Shamir's sentence"
            );
        }
        assert_eq!(choose(SHAMIR_SCIENTISTS, SHAMIR_NEEDED - 1), "= C(11, 5)");
        assert_eq!(choose(SHAMIR_SCIENTISTS - 1, SHAMIR_NEEDED - 1), "= C(10, 5)");
        assert_eq!(combinations(&scientists, SHAMIR_NEEDED - 1).len(), liu.locks().len());
        assert_eq!(combinations(&scientists[1..], SHAMIR_NEEDED - 1).len(), liu.keys(1));
        assert_eq!(ours_label(THRESHOLD, TRUSTEES.len()), "any 3 of 5 trustees");
        assert_eq!(theirs_label(SHAMIR_NEEDED, SHAMIR_SCIENTISTS), "any 6 of 11 scientists");
        let shares: Vec<u32> = dealt(Dial::vault(), &TRUSTEES).iter().map(|&(_, y)| y).collect();
        assert_eq!(f.get("S-F6.shares"), Some(csv(&shares).as_str()));
        assert_eq!(f.get("S-F3.dealer.numbers_each"), Some("1"));
    }

    /// Every trustee's six slots fill exactly once, each key from a lock
    /// whose pair the holder is outside.
    #[test]
    fn the_keys_fill_every_slot_once() {
        let locks = Padlocks::new(&TRUSTEES, THRESHOLD);
        let slots = key_slots(&locks);
        for &t in &TRUSTEES {
            let mut mine: Vec<usize> = slots.iter().filter(|s| s.1 == t).map(|s| s.2).collect();
            mine.sort_unstable();
            assert_eq!(mine, (0..locks.keys(t)).collect::<Vec<_>>(), "trustee {t}");
            assert_eq!(mine.len(), SLOT_DX.len() * SLOT_DY.len());
        }
        for &(i, t, _) in &slots {
            assert!(!locks.locks()[i].contains(&t), "trustee {t} holds lock {i}'s key");
        }
        // the key for lock 4·5 is the last of each holder's six
        let own = locks.locks().iter().position(|g| g == &PAIR).unwrap();
        assert!(slots.iter().filter(|s| s.0 == own).all(|s| s.2 == 5));
    }

    /// A lock serving two different pairs keeps its keys from both, and so
    /// shuts out their three or four trustees together, a group that must
    /// open. True of all 45 choices of two pairs; the two pictured are the
    /// overlapping 1·2, 2·3 and the separate 1·2, 3·4. The same pair twice
    /// shuts out only two (the check is not vacuous).
    #[test]
    fn a_merged_lock_shuts_out_a_group_that_must_open() {
        let pairs = combinations(&TRUSTEES, THRESHOLD - 1);
        let mut sizes = std::collections::BTreeSet::new();
        let mut choices = 0;
        for (i, p) in pairs.iter().enumerate() {
            for q in &pairs[i + 1..] {
                let (holders, shut) = merged(p, q);
                assert!(holders.iter().all(|t| !p.contains(t) && !q.contains(t)));
                assert_eq!(holders.len() + shut.len(), TRUSTEES.len());
                assert!(shut.len() >= THRESHOLD, "{p:?} {q:?} shut out {shut:?}");
                // a trio inside the shut-out group holds no key to the lock
                let trio = &shut[..THRESHOLD];
                assert!(trio.iter().all(|t| !holders.contains(t)));
                sizes.insert(shut.len());
                choices += 1;
            }
        }
        assert_eq!(choices, 45);
        assert_eq!(sizes.into_iter().collect::<Vec<_>>(), vec![3, 4]);
        let f = facts::facts().unwrap();
        assert_eq!(f.get("S-F3.merged.unions"), Some("3,4"));
        assert_eq!(f.get("S-F3.merged.pairs"), Some(MERGES[0].len().to_string().as_str()));
        let pictured: Vec<(String, String)> = MERGES
            .iter()
            .map(|[p, q]| {
                let (h, s) = merged(p, q);
                (holders_line(&h), shut_line(&s))
            })
            .collect();
        assert_eq!(
            pictured,
            [
                (
                    "keys only with 4 and 5".to_owned(),
                    "1, 2, 3 together: shut out".to_owned()
                ),
                (
                    "keys only with 5".to_owned(),
                    "1, 2, 3, 4 together: shut out".to_owned()
                ),
            ]
        );
        let (_, same) = merged(&[1, 2], &[1, 2]);
        assert_eq!(same, vec![1, 2]);
        assert!(same.len() < THRESHOLD);
    }

    /// The quotes are whole and verbatim (no wrap cuts them), Shamir's
    /// sentences as the brief records them.
    #[test]
    fn the_quote_cards_are_whole() {
        for (style, text) in [(&BIG, QUOTE), (&BIG, QUOTE_SOURCE), (&SMALL, ANSWER), (&SMALL, PIECE)] {
            for chars in [style.chars, style.source_chars] {
                let lines = wrap(text, chars, MAX_LINES);
                assert!(lines.len() < MAX_LINES, "{text}");
                assert_eq!(lines.join(" "), text.split_whitespace().collect::<Vec<_>>().join(" "));
            }
        }
        assert!(QUOTE.starts_with("Eleven scientists are working on a secret project."));
        assert!(QUOTE.ends_with("if and only if six or more of the scientists are present."));
        assert!(!QUOTE.contains("  "), "a line break left a double space");
        assert!(!QUOTE_SOURCE.contains("  "));
        assert_eq!(
            ANSWER,
            "It is not hard to show that the minimal solution uses 462 locks and 252 keys per scientist."
        );
        assert_eq!(
            PIECE,
            "The size of each piece does not exceed the size of the original data."
        );
    }

    /// The scene opens the envelope by its parts' order, as envelope.rs
    /// builds them: body, flap, seal, name, note.
    #[test]
    fn the_envelope_parts_are_in_the_order_the_scene_opens() {
        assert!(include_str!("envelope.rs").contains("parts: vec![body, flap, seal, name, note]"));
    }

    /// The locks fit the door in two rows, apart, under the lamp; the keys
    /// fit their frames; the count sits under the rail; the merged locks
    /// and their words, the minimality card and the think ring keep to the
    /// right half and apart; the quote card, the comparison and the answer
    /// card stack inside the frame; and the envelope's open flap stays
    /// inside it.
    #[test]
    fn the_layout_keeps_its_pieces_apart() {
        const FRAMES_TOP: f64 = FRAME_Y + FRAME_H / 2.0;
        const DOOR_RIGHT: f64 = DOOR_X + DOOR_W / 2.0;
        const {
            assert!(
                DOOR_X - 2.0 * LOCK_DX - CELL_W / 2.0 > DOOR_X - DOOR_W / 2.0,
                "the locks on the door"
            );
            assert!(
                LOCK_DX > CELL_W - 0.1,
                "the locks' outlines clear of the next lock's body"
            );
            assert!(LOCK_DX - LOCK_W > 2.0 * PAD, "the locks apart");
            assert!(
                LOCK_ROWS[0] - CELL_BOTTOM > LOCK_ROWS[1] + CELL_TOP + 0.1,
                "the rows apart"
            );
            assert!(
                LOCK_ROWS[0] + CELL_TOP + PAD < DOOR_Y + DOOR_H / 2.0,
                "the top row on the door"
            );
            assert!(
                LOCK_ROWS[1] - CELL_BOTTOM - PAD > DOOR_Y - DOOR_H / 2.0,
                "the bottom row on the door"
            );
            assert!(LAMP_Y - LAMP_R > DOOR_Y + DOOR_H / 2.0, "the lamp over the door");
            assert!(
                SLOT_DX[2] + KEY_HALF < FRAME_W / 2.0 - 0.03,
                "the keys inside their frames across"
            );
            assert!(
                SLOT_DY[0] + KEY_BOW_R < FRAME_H / 2.0 - 0.03,
                "the keys inside their frames up"
            );
            assert!(
                SLOT_DY[1] - KEY_TOOTH > -FRAME_H / 2.0 + 0.03,
                "the keys inside their frames down"
            );
            assert!(SLOT_DY[0] - KEY_TOOTH > SLOT_DY[1] + KEY_BOW_R, "the key rows apart");
            assert!(SLOT_DX[1] - SLOT_DX[0] > 2.0 * KEY_HALF, "the key columns apart");
            assert!(COUNT_Y + 0.1 < RAIL_Y - TRUSTEE_R - 0.2, "the count under the rail");
            assert!(
                COUNT_Y - 0.1 > -FRAME_Y_RADIUS + 0.25,
                "the count inside the bottom edge"
            );
            assert!(
                MERGE_X - CELL_W / 2.0 > DOOR_RIGHT + 0.5,
                "the merged locks right of the door"
            );
            assert!(
                MERGE_ROWS[0] - CELL_BOTTOM > MERGE_ROWS[1] + CELL_TOP + 0.2,
                "the merged rows apart"
            );
            assert!(
                MERGE_X + MERGE_LABEL_DX + 0.2 < MERGE_TEXT_X,
                "the labels left of the words"
            );
            assert!(
                MERGE_ROWS[0] + CELL_TOP < RULE_Y - 0.25,
                "the first merged lock under the rule"
            );
            assert!(
                FEWEST_Y[0] + 0.1 < MERGE_ROWS[1] - CELL_BOTTOM - 0.2,
                "the minimality card under the merges"
            );
            assert!(THINK_Y + THINK_R < RULE_Y - 0.3, "the think ring under the rule");
            assert!(THINK_X - THINK_R > DOOR_RIGHT + 0.5, "the think ring right of the door");
            assert!(
                ENV_Y - ENV_DROP + ENV_H / 2.0 + OPEN_RISE < FRAME_Y_RADIUS - 0.2,
                "the open flap inside the frame"
            );
        }
        // the merge words and the minimality card keep inside the right edge
        let locks = Padlocks::new(&TRUSTEES, THRESHOLD);
        let fewest = fewest_locks(&TRUSTEES, THRESHOLD).unwrap();
        for [p, q] in MERGES {
            let (h, s) = merged(&p, &q);
            for text in [holders_line(&h), shut_line(&s)] {
                assert!(
                    MERGE_TEXT_X + text_w(&text, MERGE_SIZE) < FRAME_X_RADIUS - 0.25,
                    "{text}"
                );
            }
        }
        for text in [OWN_LOCK.to_owned(), fewest_line(locks.locks().len(), fewest)] {
            let w = text_w(&text, FEWEST_SIZE);
            assert!(FEWEST_X - w / 2.0 > DOOR_RIGHT + 0.3 && FEWEST_X + w / 2.0 < FRAME_X_RADIUS - 0.25);
        }
        let rule_w = text_w(&rule(THRESHOLD, TRUSTEES.len()), RULE_SIZE);
        assert!(RULE_X + rule_w / 2.0 < FRAME_X_RADIUS - 0.25);
        assert!(RULE_Y + text_h(RULE_SIZE) / 2.0 < FRAME_Y_RADIUS - 0.25);
        // the quote card, then the comparison, then the answer card
        let card_w = |style: &QuoteStyle, text: &str, source: &str| {
            let widest = |t: &str, chars: usize, size: f64| {
                wrap(t, chars, MAX_LINES)
                    .iter()
                    .map(|l| text_w(l, size))
                    .fold(0.0, f64::max)
            };
            widest(text, style.chars, style.size).max(widest(source, style.source_chars, style.source_size))
                + 2.0 * CARD_SIDE_PAD
        };
        let lines = |text: &str, chars: usize| wrap(text, chars, MAX_LINES).len();
        let w = card_w(&BIG, QUOTE, QUOTE_SOURCE);
        assert!(QUOTE_X - w / 2.0 > DOOR_RIGHT + 0.3, "quote left {}", QUOTE_X - w / 2.0);
        assert!(
            QUOTE_X + w / 2.0 < FRAME_X_RADIUS - 0.15,
            "quote right {}",
            QUOTE_X + w / 2.0
        );
        const { assert!(QUOTE_TOP < FRAME_Y_RADIUS - 0.15) };
        let bottom = QUOTE_TOP - card_height(&BIG, lines(QUOTE, BIG.chars), lines(QUOTE_SOURCE, BIG.source_chars));
        let head_y = bottom - TABLE_HEAD_DY;
        assert!(
            head_y + text_h(HEAD_SIZE) / 2.0 < bottom - 0.15,
            "the heads under the quote"
        );
        let theirs_y = head_y - 2.0 * TABLE_DY;
        let choose_y = theirs_y - CHOOSE_DY;
        assert!(choose_y + text_h(CHOOSE_SIZE) / 2.0 < theirs_y - text_h(NUMBER_SIZE) / 2.0 - 0.05);
        let answer_top = choose_y - ANSWER_DY;
        assert!(answer_top < choose_y - text_h(CHOOSE_SIZE) / 2.0 - 0.1);
        let answer_bottom = answer_top
            - card_height(
                &SMALL,
                lines(ANSWER, SMALL.chars),
                lines(ANSWER_SOURCE, SMALL.source_chars),
            );
        assert!(answer_bottom > -FRAME_Y_RADIUS + 0.25, "answer bottom {answer_bottom}");
        let w = card_w(&SMALL, ANSWER, ANSWER_SOURCE);
        assert!(QUOTE_X - w / 2.0 > DOOR_RIGHT + 0.3 && QUOTE_X + w / 2.0 < FRAME_X_RADIUS - 0.15);
        // the table's columns: labels, then locks, then keys, apart
        let label_right = TABLE_LABEL_X
            + text_w(&theirs_label(SHAMIR_NEEDED, SHAMIR_SCIENTISTS), ROW_SIZE)
                .max(text_w(&ours_label(THRESHOLD, TRUSTEES.len()), ROW_SIZE));
        let locks_left = LOCKS_RIGHT
            - [
                text_w("462", NUMBER_SIZE),
                text_w(LOCKS_HEAD, HEAD_SIZE),
                text_w(&choose(11, 5), CHOOSE_SIZE),
            ]
            .into_iter()
            .fold(0.0, f64::max);
        assert!(label_right + 0.2 < locks_left, "{label_right} {locks_left}");
        let keys_left = KEYS_RIGHT
            - [
                text_w("252", NUMBER_SIZE),
                text_w(KEYS_HEAD, HEAD_SIZE),
                text_w(&choose(10, 5), CHOOSE_SIZE),
            ]
            .into_iter()
            .fold(0.0, f64::max);
        assert!(LOCKS_RIGHT + 0.3 < keys_left);
        const { assert!(KEYS_RIGHT < FRAME_X_RADIUS - 0.25) };
        const { assert!(TABLE_LABEL_X > DOOR_RIGHT + 0.3) };
        // the property card under the lowered envelope, over the frames
        let piece_top = ENV_Y - ENV_DROP - ENV_H / 2.0 - PIECE_GAP;
        let piece_bottom = piece_top
            - card_height(
                &SMALL,
                lines(PIECE, SMALL.chars),
                lines(PIECE_SOURCE, SMALL.source_chars),
            );
        assert!(piece_bottom > FRAMES_TOP + 0.3, "{piece_bottom}");
        let w = card_w(&SMALL, PIECE, PIECE_SOURCE);
        assert!(ENV_X - w / 2.0 > DOOR_RIGHT + 0.3 && ENV_X + w / 2.0 < FRAME_X_RADIUS - 0.15);
        // the key count's words fit under the rail
        let count_w = text_w(&key_count(6, 30), COUNT_SIZE);
        assert!(DOOR_X - count_w / 2.0 > -FRAME_X_RADIUS + 0.25);
    }
}
