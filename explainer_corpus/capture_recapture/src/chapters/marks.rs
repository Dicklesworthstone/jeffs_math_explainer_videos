//! Chapter 02, the marks measure the net: Feller's puzzle (02_1), red spots
//! become the gold glow (02_2), one fact read two ways with the catches laid
//! out in trays, one fish drawn per ten (02_3–02_6), the soup guess in
//! numbers and then in letters (02_7), and the three questions the video
//! answers (02_8).

use explainer_kit::animatic::HOLD_S;
use explainer_kit::props::{Swarm, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, thousands, v};
use fmn::animation::replacement_transform;
use fmn::prelude::*;

use super::draw::{Glyph, Role, fish, marked_fish, ring, spotted_fish};
use super::lake::{PER_GLYPH, Shade, Tray, fade_in_all, key};
use crate::facts::FELLER;

/// A card behind text: the corpus's raised fill, a faint edge.
fn card(w: f64, h: f64, at: Vec3) -> fmn::Result<VMobject> {
    Ok(Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(
            Style::default()
                .fill(hex("#1A2130"), 0.94)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(at))
}

/// The full-size tray of a hundred drawn fish (a thousand fish): twenty
/// across, five down, so its first two columns are one tenth of it.
const COLS: usize = 20;
const ROWS: usize = 5;
const PITCH: (f64, f64) = (0.34, 0.27);

fn tray(c: Vec3, scale: f64) -> Tray {
    Tray {
        c,
        cols: COLS,
        rows: ROWS,
        pitch: (PITCH.0 * scale, PITCH.1 * scale),
    }
}

/// Slots in the tray's first `tenth` columns: the "one in ten" block.
fn first_columns(n: usize, cols: usize) -> Vec<usize> {
    (0..n).filter(|i| i % COLS < cols).collect()
}

/// A glyph shrunk for a smaller tray (its paints unchanged).
fn shrunk(g: Glyph, f: f64) -> Glyph {
    Glyph {
        mob: g.mob.scaled_about(f, ORIGIN),
        paints: g.paints,
    }
}

/// One tray's fish, built detached: `gold` slots get the marked glyph, the
/// rest the plain one, and `ringed` slots a green ring each.
struct TrayFish {
    plain: Swarm,
    gold: Swarm,
    rings: Swarm,
    card: Mob,
}

impl TrayFish {
    fn build(
        stage: &mut Stage<'_>,
        t: Tray,
        n: usize,
        gold: &[usize],
        ringed: &[usize],
        scale: f64,
        edge: Srgb,
    ) -> fmn::Result<Self> {
        let poses = t.poses(n);
        let pick = |ids: &[usize]| ids.iter().map(|&i| poses[i]).collect::<Vec<_>>();
        let plain_ids: Vec<usize> = (0..n).filter(|i| !gold.contains(i)).collect();
        let plain = Swarm::build(
            stage,
            &shrunk(fish(Role::unmarked()), scale).mob,
            &pick(&plain_ids),
            None,
        )?;
        let gold = Swarm::build(stage, &shrunk(marked_fish(), scale).mob, &pick(gold), None)?;
        let rings = Swarm::build(stage, &shrunk(ring(), scale).mob, &pick(ringed), None)?;
        let card = stage.arena_mut().add(t.card(edge)?);
        Ok(Self {
            plain,
            gold,
            rings,
            card,
        })
    }

    /// The card, then every fish, as one list to fade in.
    fn parts(&self) -> [Mob; 4] {
        [self.card, self.plain.group, self.gold.group, self.rings.group]
    }
}

pub struct Marks {
    pub kit: Kit,
}

impl SceneConstruct for Marks {
    fn name(&self) -> &str {
        "02_the_marks_measure_the_net"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let (big_m, big_n, k) = FELLER;
        let lake = big_m * big_n / k;
        let seen = big_m + big_n - k;
        // drawn fish: one per ten
        let (dm, dn, dk) = (
            big_m as usize / PER_GLYPH,
            big_n as usize / PER_GLYPH,
            k as usize / PER_GLYPH,
        );
        // the caught-twice share of a tray is its first columns
        let tenth_cols = COLS * dk / dm;

        // 02_1: Feller's puzzle, in his first edition's words
        let quote = kit.text_styled(
            stage,
            "“Suppose that 1000 fish caught in a lake are marked by red spots…”",
            40.0,
            Palette::ink(),
            v(0.0, 0.45),
            false,
            true,
        )?;
        let cite = kit.text(
            stage,
            "William Feller, An Introduction to Probability Theory and Its Applications (1950), p. 37",
            22.0,
            Palette::dim(),
            v(0.0, -0.45),
        )?;
        let w = stage.get_width(quote).max(stage.get_width(cite)) + 0.8;
        let back = stage.arena_mut().add(card(w, 2.0, ORIGIN)?);
        kit.say(stage, "02_1")?;
        play!(stage;
            fade_in(stage.arena_mut(), back, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), quote, v(0.0, -0.15), 1.0)?.rt(1.2),
            fade_in(stage.arena_mut(), cite, ORIGIN, 1.0)?.rt(1.2),
        );
        kit.hold(stage)?;

        // 02_2: a red spot becomes the gold glow
        let red = stage.arena_mut().add(spotted_fish());
        stage.scale(red, 14.0);
        stage.move_to(red, v(0.0, -0.1), ORIGIN);
        let gold = stage.arena_mut().add(marked_fish().mob);
        stage.scale(gold, 14.0);
        stage.move_to(gold, v(0.0, -0.1), ORIGIN);
        let feller_word = kit.text(stage, "Feller: a red spot", 30.0, Palette::red(), v(0.0, -2.6))?;
        let ours_word = kit.text(stage, "ours: a glow", 30.0, Role::marked(), v(0.0, -2.6))?;
        kit.say(stage, "02_2")?;
        play!(stage;
            fade_out(stage.arena_mut(), quote, v(0.0, 1.0), 1.0)?.rt(0.8),
            fade_out(stage.arena_mut(), cite, v(0.0, 1.0), 1.0)?.rt(0.8),
            fade_out(stage.arena_mut(), back, v(0.0, 1.0), 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), red, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), feller_word, ORIGIN, 1.0)?.rt(0.8),
        );
        stage.wait(0.6)?;
        play!(stage;
            replacement_transform(red, gold).rt(1.4),
            replacement_transform(feller_word, ours_word).rt(1.4),
        );
        kit.hold(stage)?;

        // 02_3: one reading. Of the thousand marked, the second net caught a
        // hundred: one in ten (the first two columns of the marked tray)
        let the_key = key(&kit, stage)?;
        let top = tray(v(0.0, 1.75), 1.0);
        let twice = first_columns(dm, tenth_cols);
        let marked_tray = TrayFish::build(
            stage,
            top,
            dm,
            &(0..dm).collect::<Vec<_>>(),
            &twice,
            1.0,
            Role::marked(),
        )?;
        let ring_shade = Shade::new(stage, &marked_tray.rings.members, &ring().paints, 0.0)?;
        let marked_label = kit.text_left(
            stage,
            &format!("{} marked", thousands(big_m)),
            30.0,
            Role::marked(),
            v(top.left(), top.top() + 0.3),
        )?;
        let block_x = (top.slot(0)[0] + top.slot(tenth_cols - 1)[0]) / 2.0;
        let twice_label = kit.text(
            stage,
            &format!("{} caught twice", thousands(k)),
            26.0,
            Role::second(),
            v(block_x + 0.75, top.c[1] - (ROWS as f64 + 0.6) * PITCH.1 / 2.0 - 0.28),
        )?;
        let one_in = kit.text(
            stage,
            &format!("1 in {}", big_m / k),
            34.0,
            Palette::ink(),
            v(5.55, top.c[1]),
        )?;
        kit.say(stage, "02_3")?;
        play!(stage;
            fade_out(stage.arena_mut(), gold, v(0.0, 0.5), 0.2)?.rt(0.8),
            fade_out(stage.arena_mut(), ours_word, ORIGIN, 1.0)?.rt(0.6),
        );
        fade_in_all(
            stage,
            &[marked_tray.card, marked_tray.gold.group, marked_label, the_key],
            0.8,
        )?;
        // the rings go on top of the gold halos
        stage.add_to_scene(marked_tray.rings.group)?;
        stage.play(ring_shade.to(1.0, 0.8)?)?;
        fade_in_all(stage, &[twice_label, one_in], 0.6)?;
        kit.hold(stage)?;

        // 02_4: so the second net caught about one fish in ten. A thousand is
        // a tenth of how many? (The second catch: every fish ringed, its
        // first two columns the marked ones.)
        let low = tray(v(0.0, -1.05), 1.0);
        let caught_tray = TrayFish::build(
            stage,
            low,
            dn,
            &first_columns(dn, tenth_cols),
            &(0..dn).collect::<Vec<_>>(),
            1.0,
            Role::second(),
        )?;
        let second_label = kit.text_left(
            stage,
            &format!("{} in the second catch", thousands(big_n)),
            30.0,
            Role::second(),
            v(low.left(), low.top() + 0.3),
        )?;
        let ask = kit.text(
            stage,
            &format!("{} = one tenth of ?", thousands(big_n)),
            40.0,
            Palette::ink(),
            v(0.0, -2.75),
        )?;
        kit.say(stage, "02_4")?;
        let mut second = caught_tray.parts().to_vec();
        second.push(second_label);
        fade_in_all(stage, &second, 0.8)?;
        play!(stage; write(stage.arena(), ask).rt(1.0));
        let q = predict_pause(&kit, stage, v(5.4, -2.75))?;

        // 02_5: ten thousand. The second catch copied ten times stacks into
        // the lake; of its fish we have seen 1,900, and the rest dim
        kit.say(stage, "02_5")?;
        let mut away = vec![q, ask, second_label, twice_label, one_in, marked_label];
        away.extend(marked_tray.parts());
        let mut fades: Vec<Box<dyn Animation>> = Vec::new();
        for m in away {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?.rt(0.5);
            fades.push(stage.prepare(a)?);
        }
        stage.play_prepared(fades)?;
        // ten small trays, two across and five down; the real catch is the
        // first, the other nine are copies of it
        let mini = 0.42;
        let mini_at = |j: usize| v(-1.65 + 3.3 * (j % 2) as f64, 1.55 - 0.78 * (j / 2) as f64);
        let mut moves: Vec<Box<dyn Animation>> = Vec::new();
        for g in caught_tray.parts() {
            let a = g
                .animate()
                .set_anim_args(AnimateArgs {
                    run_time: Some(1.0),
                    ..AnimateArgs::default()
                })?
                .scale_about(mini, Some(low.c), None)?
                .shift([mini_at(0)[0] - low.c[0], mini_at(0)[1] - low.c[1], 0.0])?;
            moves.push(stage.prepare(a)?);
        }
        stage.play_prepared(moves)?;
        // the nine copies are the rest of the lake (lake_trays: every slot
        // has a fate, and the totals are pinned in the tests)
        let trays_of_lake = lake_trays(dn, tenth_cols, (lake / big_n) as usize);
        let copies: Vec<TrayFish> = trays_of_lake
            .iter()
            .enumerate()
            .skip(1)
            .map(|(j, t)| {
                TrayFish::build(
                    stage,
                    tray(mini_at(j), mini),
                    dn,
                    &t.gold,
                    &t.ringed_slots(dn),
                    mini,
                    Role::second(),
                )
            })
            .collect::<fmn::Result<_>>()?;
        let total = kit.text(
            stage,
            &format!("{} × {} = {} fish", lake / big_n, thousands(big_n), thousands(lake)),
            32.0,
            Palette::ink(),
            v(0.0, 2.45),
        )?;
        for c in &copies {
            let mut v: Vec<Box<dyn Animation>> = Vec::new();
            for g in c.parts() {
                let a = fade_in(stage.arena_mut(), g, v_shift(), 1.0)?.rt(0.35);
                v.push(stage.prepare(a)?);
            }
            stage.play_prepared(v)?;
        }
        fade_in_all(stage, &[total], 0.5)?;
        // none of the copies was in the second catch (their rings go); their
        // gold tenth is the rest of the first catch, 900 fish; everything
        // else in them, 8,100 fish, nobody caught
        let never = kit.text(
            stage,
            &format!(
                "{} − {} = {} never caught",
                thousands(lake),
                thousands(seen),
                thousands(lake - seen)
            ),
            30.0,
            Role::dark(),
            v(0.0, -2.75),
        )?;
        let mut rings_out = Vec::new();
        let mut unseen = Vec::new();
        for c in &copies {
            rings_out.extend(c.rings.members.iter().copied());
            unseen.extend(c.plain.members.iter().copied());
        }
        let copy_rings = Shade::new(stage, &rings_out, &ring().paints, 1.0)?;
        let dim = Shade::new(stage, &unseen, &fish(Role::unmarked()).paints, 1.0)?;
        stage.play(copy_rings.to(0.0, 0.8)?)?;
        play!(stage; dim.to(0.22, 1.2)?, fade_in(stage.arena_mut(), never, ORIGIN, 1.0)?.rt(1.2));
        kit.hold(stage)?;
        stage.wait(1.0)?;

        // 02_6: the other reading: one in ten of the second catch is marked,
        // so about one in ten of the lake; the marked fish copied ten times
        fade_all(stage, 0.8)?;
        let small = 0.6;
        let left = tray(v(-3.45, 2.0), small);
        let right = tray(v(3.45, 2.0), small);
        let all_m: Vec<usize> = (0..dm).collect();
        let marked_side = TrayFish::build(stage, left, dm, &all_m, &twice, small, Role::marked())?;
        let catch_side = TrayFish::build(
            stage,
            right,
            dn,
            &first_columns(dn, tenth_cols),
            &(0..dn).collect::<Vec<_>>(),
            small,
            Role::second(),
        )?;
        let left_words = kit.text(
            stage,
            &format!("{} of {} marked were caught again", thousands(k), thousands(big_m)),
            24.0,
            Role::marked(),
            v(left.c[0], left.top() + 0.3),
        )?;
        let right_words = kit.text(
            stage,
            &format!("{} of {} caught are marked", thousands(k), thousands(big_n)),
            24.0,
            Role::second(),
            v(right.c[0], right.top() + 0.3),
        )?;
        let tiny = 0.3;
        let tiny_at = |j: usize| v(-5.6 + 2.8 * (j % 5) as f64, 0.2 - 0.75 * (j / 5) as f64);
        // the lake ten times the marked thousand: the marked tray and nine
        // trays like it of unmarked fish, so the gold stays one in ten
        let lake_gold = marked_lake(dm, (lake / big_m) as usize);
        let marked_copies: Vec<TrayFish> = lake_gold
            .iter()
            .enumerate()
            .map(|(j, gold)| {
                let edge = if gold.is_empty() {
                    Role::unmarked()
                } else {
                    Role::marked()
                };
                TrayFish::build(stage, tray(tiny_at(j), tiny), dm, gold, &[], tiny, edge)
            })
            .collect::<fmn::Result<_>>()?;
        let lake_words = kit.text(
            stage,
            &format!(
                "{} marked × {} = {} in the lake",
                thousands(big_m),
                lake / big_m,
                thousands(lake)
            ),
            30.0,
            Palette::ink(),
            v(0.0, -1.85),
        )?;
        kit.say(stage, "02_6")?;
        let mut l = marked_side.parts().to_vec();
        l.push(left_words);
        fade_in_all(stage, &l, 0.8)?;
        let mut r = catch_side.parts().to_vec();
        r.push(right_words);
        fade_in_all(stage, &r, 0.8)?;
        let note = precisely(
            &kit,
            stage,
            "E[K] = nM/N for every size of second catch, by linearity: each of the n fish caught is marked \
             with chance M/N.",
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.8));
        for c in &marked_copies {
            let mut v: Vec<Box<dyn Animation>> = Vec::new();
            for g in [c.card, c.gold.group, c.plain.group] {
                let a = fade_in(stage.arena_mut(), g, v_shift(), 1.0)?.rt(0.3);
                v.push(stage.prepare(a)?);
            }
            stage.play_prepared(v)?;
        }
        fade_in_all(stage, &[lake_words], 0.6)?;
        kit.hold(stage)?;

        // 02_7: the soup guess, numbers first, then letters under words
        fade_all(stage, 0.8)?;
        let numbers = kit.tex(
            stage,
            &format!(
                r"\frac{{{} \times {}}}{{{}}} = {}",
                thousands(big_m).replace(',', "{,}"),
                thousands(big_n).replace(',', "{,}"),
                thousands(k),
                thousands(lake).replace(',', "{,}")
            ),
            64.0,
            Palette::ink(),
            &[],
            v(0.0, 1.3),
        )?;
        let letters = kit.tex(
            stage,
            r"\hat{N} = \frac{M\,n}{k}",
            72.0,
            Palette::ink(),
            &[
                ("M", Role::marked()),
                ("n", Role::second()),
                ("k", Role::marked()),
                ("N", Role::dark()),
            ],
            v(0.0, -0.9),
        )?;
        let legend = [
            ("M  marked", Role::marked(), -4.0),
            ("n  second catch", Role::second(), 0.0),
            ("k  caught twice", Role::marked(), 4.0),
        ];
        let mut words = Vec::new();
        for (s, c, x) in legend {
            words.push(kit.text(stage, s, 28.0, c, v(x, -2.55))?);
        }
        let name = kit.text(stage, "the soup guess", 36.0, Role::dark(), v(0.0, 3.0))?;
        kit.say(stage, "02_7")?;
        play!(stage; write(stage.arena(), numbers).rt(1.6));
        stage.wait(0.6)?;
        play!(stage; write(stage.arena(), letters).rt(1.4));
        for w in words {
            play!(stage; fade_in(stage.arena_mut(), w, v(0.0, 0.15), 1.0)?.rt(0.4));
        }
        play!(stage; fade_in(stage.arena_mut(), name, ORIGIN, 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 02_8: the three questions; each returns as an act title
        fade_all(stage, 0.8)?;
        kit.say(stage, "02_8")?;
        for (i, s) in ["Why should that work?", "How sure can we be?", "When does it fail?"]
            .into_iter()
            .enumerate()
        {
            let x = -4.6 + 4.6 * i as f64;
            let t = kit.text(stage, s, 32.0, Palette::ink(), v(x, 0.0))?;
            let b = stage.arena_mut().add(card(4.3, 1.6, v(x, 0.0))?);
            play!(stage;
                fade_in(stage.arena_mut(), b, v(0.0, -0.2), 1.0)?.rt(0.6),
                fade_in(stage.arena_mut(), t, v(0.0, -0.2), 1.0)?.rt(0.6),
            );
        }
        kit.music_breath(stage, HOLD_S)?;
        kit.hold(stage)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// One tray of 02_5's lake: which slots hold marked fish, and whether its
/// fish were in the second catch (ringed).
#[derive(Clone, Debug, PartialEq)]
struct LakeTray {
    gold: Vec<usize>,
    ringed: bool,
}

impl LakeTray {
    fn ringed_slots(&self, n: usize) -> Vec<usize> {
        if self.ringed { (0..n).collect() } else { Vec::new() }
    }
}

/// 02_5's lake as `copies` trays of `n` drawn fish: the first is the second
/// catch (ringed; its first `tenth_cols` columns the caught twice), the
/// others the rest of the lake (no rings; their first columns the rest of
/// the first catch, the other slots fish nobody caught).
fn lake_trays(n: usize, tenth_cols: usize, copies: usize) -> Vec<LakeTray> {
    (0..copies)
        .map(|j| LakeTray {
            gold: first_columns(n, tenth_cols),
            ringed: j == 0,
        })
        .collect()
}

/// 02_6's lake: `copies` trays of `n` drawn fish, the first all marked (the
/// first catch), the rest unmarked; the gold slots of each.
fn marked_lake(n: usize, copies: usize) -> Vec<Vec<usize>> {
    (0..copies)
        .map(|j| if j == 0 { (0..n).collect() } else { Vec::new() })
        .collect()
}

/// The small drop a copied tray makes as it appears.
fn v_shift() -> Vec3 {
    v(0.0, -0.12)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lake_of_02_5_holds_every_fish_once() {
        // drawn glyphs: 1,000 in all; 100 marked, 900 not; 190 seen, 810 never
        let (big_m, big_n, k) = FELLER;
        let (dm, dn, dk) = (
            big_m as usize / PER_GLYPH,
            big_n as usize / PER_GLYPH,
            k as usize / PER_GLYPH,
        );
        let lake = (big_m * big_n / k) as usize / PER_GLYPH;
        let trays = lake_trays(dn, COLS * dk / dm, lake / dn);
        let all = trays.len() * dn;
        let gold: usize = trays.iter().map(|t| t.gold.len()).sum();
        let seen: usize = trays.iter().map(|t| if t.ringed { dn } else { t.gold.len() }).sum();
        assert_eq!((all, gold, all - gold, seen, all - seen), (1_000, 100, 900, 190, 810));
        // in the tens of fish the labels say: 10,000 − 1,900 = 8,100
        assert_eq!((seen * PER_GLYPH, (all - seen) * PER_GLYPH), (1_900, 8_100));
    }

    #[test]
    fn the_lake_of_02_6_is_one_tenth_marked() {
        let (big_m, big_n, k) = FELLER;
        let dm = big_m as usize / PER_GLYPH;
        let trays = marked_lake(dm, (big_m * big_n / k / big_m) as usize);
        let gold: usize = trays.iter().map(Vec::len).sum();
        assert_eq!(gold, big_m as usize / PER_GLYPH);
        assert_eq!(trays.len() * dm, (big_m * big_n / k) as usize / PER_GLYPH);
    }

    #[test]
    fn a_trays_first_two_columns_are_one_tenth_of_it() {
        let (m, k) = (FELLER.0 as usize / PER_GLYPH, FELLER.2 as usize / PER_GLYPH);
        let cols = COLS * k / m;
        assert_eq!(first_columns(m, cols).len(), k);
        assert_eq!(cols * ROWS, k);
    }
}
