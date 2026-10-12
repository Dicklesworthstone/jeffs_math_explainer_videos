//! Chapter 08, prime clocks: on a seven-hour clock, three times what is one
//! (08_1)? Five: the hand hops three hours five times and lands on 1, and
//! every row of the seven-hour times table but zero's is a reshuffle, so
//! every hour but 0 has a partner (08_2). Zero sneaks in at 256 = 2 × 128,
//! and the nearest prime, 257, needs a ninth bit (08_3). A field, with 256
//! members: catch the falling bit (08_4).

use explainer_kit::props::{precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, play, stroke_style, tween, v};
use fmn::animation::indicate;
use fmn::prelude::*;

use super::clock::Clock;
use super::draw::{Role, group_of};
use super::register::{Register, tile};
use crate::model::clocks::{bits_needed, mul_mod, partner, primes_around};

/// The prime clock's hours, and 08_1's question: 3 × ? = 1.
const HOURS: u8 = 7;
const THREE: u8 = 3;
/// 08_3: a byte's clock and its factors.
const BYTE_HOURS: u32 = 256;
const TWO: u32 = 2;

/// The prime clock's colour: it works, so the "good" role.
fn prime() -> Srgb {
    Palette::green()
}

const TABLE_CELL: f64 = 0.62;
const TABLE_X: f64 = 3.6;
const TABLE_Y: f64 = -0.35;

/// The centre of the seven-hour table's cell (row, col); row/col −1 are the
/// headers.
fn cell_at(row: i32, col: i32) -> Vec3 {
    let n = f64::from(HOURS);
    v(
        TABLE_X + (f64::from(col) - (n - 1.0) / 2.0) * TABLE_CELL,
        TABLE_Y - (f64::from(row) - (n - 1.0) / 2.0) * TABLE_CELL,
    )
}

pub struct PrimeClocks {
    pub kit: Kit,
}

impl SceneConstruct for PrimeClocks {
    fn name(&self) -> &str {
        "08_prime_clocks"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let hours = u32::from(HOURS);
        let five = partner(u32::from(THREE), hours).expect("7 is prime: 3 has a partner");

        // 08_1: a prime number of hours; on seven, three times what is one?
        let clock = Clock::build(
            &kit,
            stage,
            usize::from(HOURS),
            v(-3.6, -0.2),
            2.3,
            Palette::ink(),
            &(0..usize::from(HOURS)).collect::<Vec<_>>(),
            34.0,
        )?;
        let title = kit.text(
            stage,
            &format!("{HOURS} hours: {HOURS} is prime"),
            38.0,
            prime(),
            v(3.2, 2.6),
        )?;
        // "what times three is one?": the unknown first, ? × 3 = 1
        let lhs = kit.tex(
            stage,
            &format!(r"\times {THREE}"),
            72.0,
            Palette::ink(),
            &[],
            v(3.3, 0.6),
        )?;
        let rhs = kit.tex(stage, "= 1", 72.0, Palette::ink(), &[], v(4.75, 0.6))?;
        kit.say(stage, "08_1")?;
        play!(stage;
            show_creation(clock.ring).rt(1.0),
            fade_in(stage.arena_mut(), clock.ticks, ORIGIN, 1.0)?.rt(1.0),
            fade_in(stage.arena_mut(), title, ORIGIN, 1.0)?.rt(0.8),
        );
        let mut labels: Vec<Box<dyn Animation>> = Vec::new();
        for &(_, l) in &clock.labels {
            let a = fade_in(stage.arena_mut(), l, ORIGIN, 1.0)?;
            labels.push(stage.prepare(a)?);
        }
        let labels = explainer_kit::lagged(stage, labels, 0.15)?;
        play!(stage; labels.rt(0.8));
        play!(stage;
            write(stage.arena(), lhs).rt(0.6),
            write(stage.arena(), rhs).rt(0.6),
        );
        let q = predict_pause(&kit, stage, v(2.05, 0.6))?;

        // 08_2: five. The hand hops three hours five times and lands on one
        let at = stage.add_value_tracker(0.0);
        let hand = clock.hand(stage, at, prime());
        kit.say(stage, "08_2")?;
        stage.add_to_scene(hand)?;
        let ans = kit.tex(stage, &five.to_string(), 72.0, prime(), &[], v(2.05, 0.6))?;
        play!(stage;
            fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3),
            fade_in(stage.arena_mut(), ans, ORIGIN, 1.0)?.rt(0.3),
        );
        let mut hops = Vec::new();
        for k in 1..=five {
            play!(stage; tween(at, f64::from(k * u32::from(THREE)), 0.42)?);
            let n = kit.text(
                stage,
                &k.to_string(),
                24.0,
                prime(),
                clock.point(f64::from(k * u32::from(THREE)), 0.72),
            )?;
            play!(stage; fade_in(stage.arena_mut(), n, ORIGIN, 1.0)?.rt(0.15));
            hops.push(n);
        }
        let product = u32::from(THREE) * five;
        assert_eq!(mul_mod(u32::from(THREE), five, hours), 1);
        let one = clock.labels[1].1;
        let full = kit.tex(
            stage,
            &format!(r"{five} \times {THREE} = {product} \equiv 1"),
            48.0,
            Palette::ink(),
            &[],
            v(3.3, -0.55),
        )?;
        play!(stage;
            indicate(stage.arena_mut(), one, 1.5, Some([0.51, 0.76, 0.40]))?.rt(0.8),
            write(stage.arena(), full).rt(0.8),
        );
        // the table: every row but zero's is a reshuffle
        stage.clear_updaters(hand, true);
        let mut gone: Vec<Mob> = hops.clone();
        gone.extend([title, lhs, rhs, ans, full, hand]);
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &gone {
            let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
            anims.push(stage.prepare(a)?);
        }
        let shrink = clock.group.animate().scale(0.78)?.move_to(v(-4.4, -0.35), ORIGIN)?;
        anims.push(stage.prepare(shrink)?);
        stage.play_prepared_with(
            anims,
            PlayOverrides {
                run_time: Some(0.7),
                ..PlayOverrides::default()
            },
        )?;
        let times = kit.tex(stage, r"\times", 34.0, Palette::dim(), &[], cell_at(-1, -1))?;
        let mut heads = vec![times];
        for k in 0..i32::from(HOURS) {
            heads.push(kit.text(stage, &k.to_string(), 30.0, Palette::dim(), cell_at(-1, k))?);
            heads.push(kit.text(stage, &k.to_string(), 30.0, Palette::dim(), cell_at(k, -1))?);
        }
        let heads = group_of(stage, &heads)?;
        let mut rows = Vec::new();
        let mut ones = Vec::new();
        for r in 0..hours {
            let mut cells = Vec::new();
            for c in 0..hours {
                let p = mul_mod(r, c, hours);
                let ink = if r == 0 { Palette::faint() } else { Palette::ink() };
                let t = kit.text(stage, &p.to_string(), 30.0, ink, cell_at(r as i32, c as i32))?;
                cells.push(t);
                if p == 1 {
                    ones.push(cell_at(r as i32, c as i32));
                }
            }
            rows.push(group_of(stage, &cells)?);
        }
        play!(stage; fade_in(stage.arena_mut(), heads, ORIGIN, 1.0)?.rt(0.5));
        play!(stage; fade_in(stage.arena_mut(), rows[0], ORIGIN, 1.0)?.rt(0.3));
        for &row in &rows[1..] {
            let band = Rectangle::new()
                .width(TABLE_CELL * f64::from(HOURS))
                .height(TABLE_CELL * 0.9)
                .corner_radius(0.08)
                .style(stroke_style(prime(), 2.5))
                .build()?
                .moved_to(v(TABLE_X, stage.get_center(row)[1]));
            let band = stage.arena_mut().add(band);
            play!(stage;
                fade_in(stage.arena_mut(), row, v(0.2, 0.0), 1.0)?.rt(0.35),
                show_creation(band).rt(0.35),
            );
            play!(stage; fade_out(stage.arena_mut(), band, ORIGIN, 1.0)?.rt(0.2));
        }
        let mut rings = Vec::new();
        for &p in &ones {
            let ring = Circle::new()
                .radius(TABLE_CELL * 0.42)
                .arc_center(p)
                .style(stroke_style(prime(), 3.0))
                .build();
            rings.push(stage.arena_mut().add(ring));
        }
        let rings = group_of(stage, &rings)?;
        play!(stage; show_creation(rings).rt(0.9));
        let partners = kit.text(stage, "every hour but 0 has a partner", 30.0, prime(), v(TABLE_X, -3.0))?;
        play!(stage; fade_in(stage.arena_mut(), partners, ORIGIN, 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 08_3: zero sneaks in at 256 = 2 × 128; 257 is prime but needs a
        // ninth bit
        let half = BYTE_HOURS / TWO;
        let (_, next_prime) = primes_around(BYTE_HOURS);
        fade_all(stage, 0.6)?;
        let factored = kit.tex(
            stage,
            &format!(r"{BYTE_HOURS} = {TWO} \times {half}"),
            60.0,
            Palette::ink(),
            &[],
            v(-3.3, 2.4),
        )?;
        let zero = kit.tex(
            stage,
            &format!(
                r"{TWO} \times {half} \equiv {} \pmod{{{BYTE_HOURS}}}",
                mul_mod(TWO, half, BYTE_HOURS)
            ),
            52.0,
            Role::wrap(),
            &[],
            v(3.3, 2.4),
        )?;
        let prime_note = kit.tex(
            stage,
            &format!(r"{next_prime} \text{{ is prime: hours }} 0, 1, \ldots, {BYTE_HOURS}"),
            52.0,
            prime(),
            &[],
            v(0.0, 0.95),
        )?;
        let reg = Register::build(&kit, stage, 0, 0.8, v(0.9, -1.25), Palette::ink(), true)?;
        let ninth = tile(&kit, stage, true, 0.8, Role::lost(), reg.gutter_point())?;
        let spill = kit.text(
            stage,
            &format!("{BYTE_HOURS} needs {} bits", bits_needed(BYTE_HOURS)),
            30.0,
            Role::lost(),
            v(-3.6, -2.7),
        )?;
        kit.say(stage, "08_3")?;
        play!(stage; write(stage.arena(), factored).rt(0.9));
        play!(stage; fade_in(stage.arena_mut(), zero, ORIGIN, 1.0)?.rt(0.7));
        stage.wait(1.0)?;
        play!(stage; write(stage.arena(), prime_note).rt(1.0));
        reg.show(stage, 0.6)?;
        play!(stage;
            fade_in(stage.arena_mut(), ninth, v(1.2, 0.3), 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), spill, ORIGIN, 1.0)?.rt(0.8),
        );
        play!(stage; indicate(stage.arena_mut(), ninth, 1.25, Some([0.99, 0.38, 0.33]))?.rt(0.8));
        kit.hold(stage)?;

        // 08_4: show, then name: the four operations, division by anything
        // but 0, the usual rules; "a field"; 256 members, one per byte
        kit.say(stage, "08_4")?;
        play!(stage;
            fade_out(stage.arena_mut(), factored, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), zero, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), prime_note, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), spill, ORIGIN, 1.0)?.rt(0.5),
        );
        let mut ops = Vec::new();
        for (i, op) in ["+", "-", r"\times", r"\div"].into_iter().enumerate() {
            let m = kit.tex(stage, op, 72.0, Palette::ink(), &[], v(0.2 + 1.2 * i as f64, 2.55))?;
            play!(stage; fade_in(stage.arena_mut(), m, v(0.0, -0.2), 1.0)?.rt(0.35));
            ops.push(m);
        }
        let but = kit.text(stage, "by anything but 0", 24.0, Palette::dim(), v(3.8, 1.95))?;
        play!(stage; fade_in(stage.arena_mut(), but, ORIGIN, 1.0)?.rt(0.4));
        let rules = kit.tex(
            stage,
            r"a + b = b + a \qquad a(b + c) = ab + ac \qquad \ldots",
            40.0,
            Palette::dim(),
            &[],
            v(1.9, 1.5),
        )?;
        play!(stage; write(stage.arena(), rules).rt(1.2));
        let field = kit.text_styled(stage, "a field", 72.0, Role::byte(), v(-4.3, 2.45), true, false)?;
        play!(stage; write(stage.arena(), field).rt(0.8));
        stage.wait(0.6)?;
        let members = kit.text(
            stage,
            &format!("{BYTE_HOURS} members, one for every byte"),
            32.0,
            Role::byte(),
            // Leave room below the caption for the caught ninth bit.
            v(-1.2, 0.9),
        )?;
        let side = (BYTE_HOURS as f64).sqrt() as usize;
        let dots: Vec<VMobject> = (0..BYTE_HOURS as usize)
            .map(|k| {
                Dot::new()
                    .point(v(
                        5.6 + 0.088 * ((k % side) as f64 - 7.5),
                        0.35 - 0.088 * ((k / side) as f64 - 7.5),
                    ))
                    .radius(0.028)
                    .color(Role::byte())
                    .build()
            })
            .collect();
        assert_eq!(dots.len(), side * side, "256 is a square");
        let dots = stage.arena_mut().add(v_group(dots));
        play!(stage;
            fade_in(stage.arena_mut(), members, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), dots, ORIGIN, 1.0)?.rt(1.2).lag(0.02),
        );
        stage.wait(0.8)?;
        // catch it: the spilled bit lifts out, turns to the fold's colour, and
        // waits above the register's low bits (chapter 10 folds it in)
        let catch = ninth
            .animate()
            .set_anim_args(AnimateArgs {
                run_time: Some(1.6),
                path_arc: Some(-1.2),
                ..AnimateArgs::default()
            })?
            .move_to(v(reg.slot(4)[0], reg.centre[1] + 1.3), ORIGIN)?
            .set_color(Role::fold())?;
        stage.play(catch)?;
        let note = precisely(
            &kit,
            stage,
            &format!(
                "finite fields exist exactly for sizes that are a power of a prime: {HOURS} is one, and so is {BYTE_HOURS} \
                 = {}",
                vec![TWO.to_string(); bits_needed(BYTE_HOURS - 1) as usize].join(" × ")
            ),
        )?;
        play!(stage; fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6));
        kit.hold(stage)?;
        kit.music_breath(stage, explainer_kit::animatic::HOLD_S)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}
