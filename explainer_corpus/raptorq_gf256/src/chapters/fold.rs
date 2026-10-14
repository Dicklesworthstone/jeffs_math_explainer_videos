//! Chapter 10, the fold: the rotation (10_1) and its failure, 3 × 255 = 0
//! (10_2–10_3: x⁸ + 1 = (x + 1)(x⁷ + … + 1)); a prime polynomial, said and
//! shown whole, x⁸ + x⁴ + x³ + x² + 1, treated as zero: x⁸ → x⁴ + x³ + x² + 1
//! (10_4); 2 × 128 folds to 29 (10_5, the 07_3 slide again, blooming into
//! the 0x1D chord); 2 × 142 = 1 and the byte field (10_6). The shift-and-add
//! code panel and AES's polynomial, 87 × 131 = 49 here and 193 there, are
//! 12_5's ([`aes_contrast`]).

use explainer_kit::props::{lower_third, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, children, fade_all, hex, play, v};
use fmn::animation::{indicate, replacement_transform};
use fmn::core::constants::{FRAME_X_RADIUS, FRAME_Y_RADIUS};
use fmn::library::SpanMapData;
use fmn::prelude::*;

use super::draw::{Role, card_fill, group_of};
use super::register::Register;
use crate::model::{AES_FOLD, FOLD, inv, mul};

/// x⁸ → 1: the rotation, as a fold byte.
const ROTATE: u8 = 0x01;
/// 10_1's byte (x⁷), 10_2's, 10_6's partner of 2, and 10_7's pair.
const X7: u8 = 0x80;
const ALL: u8 = 0xFF;
const TWO: u8 = 2;
const THREE: u8 = 3;
const LEFT_BYTE: u8 = 0x57;
const RIGHT_BYTE: u8 = 0x83;
const LISTEN_S: f64 = 3.0;

/// A byte in the code's hex.
fn hexb(b: u8) -> String {
    format!("{b:#04X}").replace("0X", "0x")
}

/// A degree-8 polynomial's nine bits in hex, lower-case 0x: 0x11D.
pub(crate) fn hexw(w: u16) -> String {
    format!("{w:#05X}").replace("0X", "0x")
}

/// The orange [`Kit::tex`] lines of the fold: "x^{8} \to …" from a fold byte.
pub(crate) fn fold_rule(fold: u8) -> String {
    let rhs: Vec<String> = (0..8u32)
        .rev()
        .filter(|&e| fold >> e & 1 == 1)
        .map(|e| match e {
            0 => "1".to_owned(),
            1 => "x".to_owned(),
            _ => format!("x^{{{e}}}"),
        })
        .collect();
    format!(r"x^{{8}} \to {}", rhs.join(" + "))
}

/// The full degree-8 polynomial of a fold byte: "x^{8} + x^{4} + …".
fn fold_poly(fold: u8) -> String {
    fold_rule(fold).replace(r" \to ", " + ")
}

// ------------------------------------------------------------ code panel

/// One segment of a code line, with an optional key to find it by.
pub(crate) type Seg = (String, Option<&'static str>);

/// 10_4–10_7's "for programmers" panel: the video's own shift-and-add
/// multiply, the fold byte and the comments' values computed from `fold`.
fn code_lines(fold: u8) -> Vec<Vec<Seg>> {
    let plain = |s: &str| -> Vec<Seg> { vec![(s.to_owned(), None)] };
    let partner = (1..=255u8)
        .find(|&b| mul(TWO, b, fold) == 1)
        .expect("a prime fold: 2 has a partner");
    vec![
        plain("fn xtime(a: u8) -> u8 {"),
        vec![
            ("    (a << 1) ^ if a & 0x80 != 0 { ".into(), None),
            (hexb(fold), Some("fold")),
            (" } else { 0 }".into(), None),
        ],
        plain("}"),
        plain("fn mul(mut a: u8, mut b: u8) -> u8 { // no tables"),
        plain("    let mut acc = 0;"),
        plain("    while b != 0 {"),
        vec![("        if b & 1 != 0 { acc ^= a }".into(), Some("pick"))],
        vec![("        a = xtime(a);".into(), Some("double"))],
        plain("        b >>= 1;"),
        plain("    }"),
        vec![
            (
                format!("    acc // mul({}, {}) == ", hexb(LEFT_BYTE), hexb(RIGHT_BYTE)),
                None,
            ),
            (hexb(mul(LEFT_BYTE, RIGHT_BYTE, fold)), Some("product")),
        ],
        vec![
            (format!("}}       // mul({TWO}, "), None),
            (partner.to_string(), Some("partner")),
            (") == 1".into(), None),
        ],
    ]
}

pub(crate) struct CodePanel {
    pub group: Mob,
    keyed: Vec<(&'static str, Mob)>,
    size: f64,
}

impl CodePanel {
    /// The panel with its top-right corner at `corner`, `size` pt.
    pub(crate) fn build(
        kit: &Kit,
        stage: &mut Stage<'_>,
        lines: &[Vec<Seg>],
        size: f64,
        corner: Vec3,
    ) -> fmn::Result<Self> {
        // one column's advance: twenty zeros minus ten zeros
        let ten = kit.mono(stage, &"0".repeat(10), size, Palette::ink(), ORIGIN)?;
        let twenty = kit.mono(stage, &"0".repeat(20), size, Palette::ink(), ORIGIN)?;
        let col = (stage.get_width(twenty) - stage.get_width(ten)) / 10.0;
        let line_h = size / 48.0 * 0.52;
        let cols = lines
            .iter()
            .map(|l| l.iter().map(|(s, _)| s.chars().count()).sum::<usize>())
            .max()
            .unwrap_or(0);
        let (w, h) = (col * cols as f64 + 0.6, 0.85 + line_h * lines.len() as f64);
        let centre = v(corner[0] - w / 2.0, corner[1] - h / 2.0);
        let back = Rectangle::new()
            .width(w)
            .height(h)
            .corner_radius(0.12)
            .style(
                Style::default()
                    .fill(hex("#1A2130"), 0.96)
                    .stroke(Palette::faint(), 1.5, 1.0),
            )
            .build()?
            .moved_to(centre);
        let back = stage.arena_mut().add(back);
        let left = centre[0] - w / 2.0 + 0.3;
        let top = centre[1] + h / 2.0 - 0.3;
        let header = kit.text_left(stage, "for programmers", 22.0, Palette::dim(), v(left, top))?;
        let (mut all, mut keyed) = (vec![back, header], Vec::new());
        for (i, line) in lines.iter().enumerate() {
            let y = top - 0.45 - line_h * i as f64;
            let mut at = 0usize;
            for (s, key) in line {
                let lead = s.chars().take_while(|&c| c == ' ').count();
                let body = s.trim_start();
                if !body.is_empty() {
                    let ink = if key.is_some() { Role::byte() } else { Palette::ink() };
                    let x = left + col * (at + lead) as f64;
                    let m = kit.mono(stage, body, size, ink, v(x, y))?;
                    stage.move_to(m, v(x, y), LEFT);
                    all.push(m);
                    if let Some(k) = key {
                        keyed.push((*k, m));
                    }
                }
                at += s.chars().count();
            }
        }
        let group = group_of(stage, &all)?;
        Ok(Self { group, keyed, size })
    }

    pub(crate) fn get(&self, key: &str) -> Mob {
        self.keyed
            .iter()
            .find(|(k, _)| *k == key)
            .map(|&(_, m)| m)
            .expect("a keyed segment")
    }

    /// Swap a keyed segment for new text in the same place (the same width:
    /// a monospaced face and equal lengths).
    fn swap(
        &mut self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        key: &'static str,
        text: &str,
        ink: Srgb,
    ) -> fmn::Result<Transform> {
        let old = self.get(key);
        let left = v(stage.get_left(old)[0], stage.get_center(old)[1]);
        let new = kit.mono(stage, text, self.size, ink, left)?;
        stage.move_to(new, left, LEFT);
        for k in &mut self.keyed {
            if k.0 == key {
                k.1 = new;
            }
        }
        Ok(replacement_transform(old, new))
    }
}

/// A lower-third whose text is math (the kit's band, a Tex line).
pub(crate) fn math_lower_third(kit: &Kit, stage: &mut Stage<'_>, label: &str, tex: &str) -> fmn::Result<Mob> {
    let h = 0.72;
    let w = 2.0 * FRAME_X_RADIUS - 0.8;
    let centre = v(0.0, -FRAME_Y_RADIUS + 0.2 + h / 2.0);
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(0.12)
        .style(
            Style::default()
                .fill(hex("#1A2130"), 0.94)
                .stroke(Palette::faint(), 1.5, 1.0),
        )
        .build()?
        .moved_to(centre);
    let back = stage.arena_mut().add(back);
    let left = -w / 2.0 + 0.25;
    let tag = kit.text_left(stage, label, 26.0, Palette::hero(), v(left, centre[1]))?;
    let x = left + stage.get_width(tag) + 0.25;
    let m = kit.tex(stage, tex, 34.0, Palette::ink(), &[], v(x, centre[1]))?;
    // a long line shrinks to the band rather than run off it
    let room = w / 2.0 - 0.25 - x;
    if stage.get_width(m) > room {
        stage.set_width(m, room, false);
    }
    stage.move_to(m, v(x, centre[1]), LEFT);
    group_of(stage, &[back, tag, m])
}

/// The glyphs of a Tex whose source spans lie inside `range`.
fn glyphs(stage: &Stage<'_>, tex: Mob, spans: &SpanMapData, range: std::ops::Range<usize>) -> Vec<Mob> {
    let kids = children(stage, tex);
    spans
        .entries
        .iter()
        .zip(kids)
        .filter(|(e, _)| e.start >= range.start && e.end <= range.end)
        .map(|(_, m)| m)
        .collect()
}

/// The centre of some glyphs' boxes.
fn centre_of(stage: &Stage<'_>, glyphs: &[Mob]) -> Vec3 {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for &g in glyphs {
        let (a, b) = (stage.get_corner(g, DL), stage.get_corner(g, UR));
        lo = [lo[0].min(a[0]), lo[1].min(a[1])];
        hi = [hi[0].max(b[0]), hi[1].max(b[1])];
    }
    v((lo[0] + hi[0]) / 2.0, lo[1])
}

pub(crate) fn fade_list(stage: &mut Stage<'_>, mobs: &[Mob], run_time: f64) -> fmn::Result<()> {
    let mut out: Vec<Box<dyn Animation>> = Vec::new();
    for &m in mobs {
        let a = fade_out(stage.arena_mut(), m, ORIGIN, 1.0)?;
        out.push(stage.prepare(a)?);
    }
    stage.play_prepared_with(
        out,
        PlayOverrides {
            run_time: Some(run_time),
            ..PlayOverrides::default()
        },
    )?;
    Ok(())
}

/// The powers of x over a register's slots, x⁷ … 1.
fn power_labels(kit: &Kit, stage: &mut Stage<'_>, reg: &Register) -> fmn::Result<Vec<Mob>> {
    (0..8u32)
        .map(|k| {
            let e = 7 - k;
            let s = match e {
                0 => "1".to_owned(),
                1 => "x".to_owned(),
                _ => format!("x^{{{e}}}"),
            };
            let at = v(reg.slot(k as usize)[0], reg.centre[1] + 0.85 * reg.cell);
            kit.tex(stage, &s, 36.0, Palette::dim(), &[], at)
        })
        .collect()
}

pub struct Fold {
    pub kit: Kit,
}

impl SceneConstruct for Fold {
    fn name(&self) -> &str {
        "10_the_fold"
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let cell = 0.85;

        // 10_1: x⁷ · x = x⁸ doesn't fit; the rotation drops it back at the bottom
        let mut reg = Register::build(&kit, stage, X7, cell, v(0.6, 0.1), Role::byte(), false)?;
        let powers = power_labels(&kit, stage, &reg)?;
        let g = reg.gutter_point();
        let x8 = kit.tex(
            stage,
            "x^{8}",
            36.0,
            Role::lost(),
            &[],
            v(g[0], reg.centre[1] + 0.85 * cell),
        )?;
        let past = kit.tex(stage, r"x^{7} \cdot x = x^{8}", 64.0, Palette::ink(), &[], v(0.6, 2.4))?;
        kit.say(stage, "10_1")?;
        reg.show(stage, 0.6)?;
        let labels = group_of(stage, &powers)?;
        play!(stage;
            fade_in(stage.arena_mut(), labels, ORIGIN, 1.0)?.rt(0.5),
            write(stage.arena(), past).rt(0.8),
        );
        play!(stage; fade_in(stage.arena_mut(), x8, ORIGIN, 1.0)?.rt(0.4));
        let fallen = reg.double(&kit, stage, 1.0)?;
        reg.rotate_in(stage, fallen, 1.3)?;
        assert_eq!(reg.value(), mul(TWO, X7, ROTATE), "the rotation turns x⁸ into 1");
        let rotation = kit.tex(
            stage,
            r"\text{rotation: } x^{8} \to 1",
            44.0,
            Palette::dim(),
            &[],
            v(0.6, -1.4),
        )?;
        play!(stage; fade_in(stage.arena_mut(), rotation, ORIGIN, 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 10_2: three times 255 under the rotation? (3a = 2a ⊕ a)
        let ask = kit.tex(
            stage,
            &format!(r"{THREE} \times {ALL} ="),
            64.0,
            Palette::ink(),
            &[],
            v(0.2, 2.4),
        )?;
        let hint_tex = kit.tex(
            stage,
            r"3 \cdot a = 2 \cdot a \oplus a",
            44.0,
            Palette::ink(),
            &[],
            v(4.2, -2.4),
        )?;
        let hint_w = stage.get_width(hint_tex) + 0.6;
        let hint_back = Rectangle::new()
            .width(hint_w)
            .height(0.8)
            .corner_radius(0.12)
            .style(Style::default().fill(card_fill(), 0.96).stroke(Role::card(), 3.0, 1.0))
            .build()?
            .moved_to(v(4.2, -2.4));
        let hint_back = stage.arena_mut().add(hint_back);
        let hint = group_of(stage, &[hint_back, hint_tex])?;
        kit.say(stage, "10_2")?;
        play!(stage;
            fmn::animation::replacement_transform(past, ask).rt(0.7),
            fade_out(stage.arena_mut(), rotation, ORIGIN, 1.0)?.rt(0.4),
        );
        reg.set_value(&kit, stage, ALL, 0.6)?;
        play!(stage; fade_in(stage.arena_mut(), hint, v(0.0, 0.15), 1.0)?.rt(0.5));
        let q_at = v(stage.get_right(ask)[0] + 0.5, 2.4);
        let q = predict_pause(&kit, stage, q_at)?;

        // 10_3: zero. 2 · 255 rotates to itself, and 255 ⊕ 255 = 0
        kit.say(stage, "10_3")?;
        play!(stage; fade_out(stage.arena_mut(), q, ORIGIN, 1.0)?.rt(0.3));
        let fallen = reg.double(&kit, stage, 0.8)?;
        reg.rotate_in(stage, fallen, 0.9)?;
        let doubled = mul(TWO, ALL, ROTATE);
        assert_eq!(reg.value(), doubled);
        let twice = kit.tex(
            stage,
            &format!(r"{TWO} \times {ALL} = {doubled}"),
            40.0,
            Palette::dim(),
            &[],
            v(5.4, 0.1),
        )?;
        play!(stage; fade_in(stage.arena_mut(), twice, ORIGIN, 1.0)?.rt(0.4));
        let below = Register::build(&kit, stage, ALL, cell, v(0.6, -1.3), Role::byte(), false)?;
        let xor = kit.tex(
            stage,
            r"\oplus",
            56.0,
            Role::ex_or(),
            &[],
            v(below.gutter_point()[0], -1.3),
        )?;
        below.show(stage, 0.4)?;
        play!(stage; fade_in(stage.arena_mut(), xor, ORIGIN, 1.0)?.rt(0.3));
        // the second row rises into the first: every bit meets itself
        let mut merge: Vec<Box<dyn Animation>> = Vec::new();
        for k in 0..8 {
            let a = fade_out(
                stage.arena_mut(),
                below.tiles[k],
                v(0.0, reg.centre[1] - below.centre[1]),
                1.0,
            )?;
            merge.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            merge,
            PlayOverrides {
                run_time: Some(0.7),
                ..PlayOverrides::default()
            },
        )?;
        let zero = doubled ^ ALL;
        reg.set_value(&kit, stage, zero, 0.4)?;
        assert_eq!(zero, mul(THREE, ALL, ROTATE), "3 · 255 = 0 under the rotation");
        let answer = kit.tex(
            stage,
            &format!(r"{THREE} \times {ALL} = {zero}"),
            64.0,
            Role::lost(),
            &[],
            v(0.6, 2.4),
        )?;
        play!(stage;
            replacement_transform(ask, answer).rt(0.6),
            fade_out(stage.arena_mut(), below.frame, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), xor, ORIGIN, 1.0)?.rt(0.5),
            fade_out(stage.arena_mut(), hint, ORIGIN, 1.0)?.rt(0.5),
        );
        // the algebra: x⁸ → 1, so x⁸ + 1 → 0, and x⁸ + 1 factors
        let mut row = vec![reg.frame, labels, x8, twice];
        row.extend(&reg.tiles);
        fade_list(stage, &row, 0.5)?;
        let to_one = kit.tex(stage, r"x^{8} \to 1", 60.0, Palette::ink(), &[], v(-3.4, 1.1))?;
        let to_zero = kit.tex(stage, r"x^{8} + 1 \to 0", 60.0, Palette::ink(), &[], v(2.4, 1.1))?;
        play!(stage; write(stage.arena(), to_one).rt(0.7));
        play!(stage; write(stage.arena(), to_zero).rt(0.7));
        let fac_src = r"(x + 1)(x^{7} + x^{6} + \cdots + x + 1) = x^{8} + 1";
        let (fac, fac_spans) = kit.tex_spans(stage, fac_src, 60.0, Palette::ink(), &[], v(0.0, -0.4))?;
        play!(stage; write(stage.arena(), fac).rt(1.2));
        let split = fac_src.find(")(").expect("two factors") + 1;
        let close = fac_src.find(") =").expect("the product") + 1;
        let first = glyphs(stage, fac, &fac_spans, 0..split);
        let second = glyphs(stage, fac, &fac_spans, split..close);
        let (a, b) = (centre_of(stage, &first), centre_of(stage, &second));
        let three = kit.tex(stage, &THREE.to_string(), 48.0, Role::lost(), &[], v(a[0], a[1] - 0.45))?;
        let all = kit.tex(stage, &ALL.to_string(), 48.0, Role::lost(), &[], v(b[0], b[1] - 0.45))?;
        play!(stage;
            fade_in(stage.arena_mut(), three, v(0.0, 0.15), 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), all, v(0.0, 0.15), 1.0)?.rt(0.5),
        );
        let again = kit.tex(stage, r"256 = 2 \times 128", 48.0, Role::wrap(), &[], v(0.0, -2.2))?;
        play!(stage; fade_in(stage.arena_mut(), again, ORIGIN, 1.0)?.rt(0.6));
        kit.hold(stage)?;

        // 10_4: a prime polynomial, said and shown whole (it stays up through
        // 10_6); treated as zero, its x⁸ becomes the fold rule (Maya R3
        // maya-02: the rule's right side alone factors). The code panel
        // waits for 12_5, where it gets a beat of its own (Dev R3 D1).
        fade_list(stage, &[answer, to_one, to_zero, fac, three, all, again], 0.5)?;
        let board = super::voice::board("10_the_fold")?;
        let prime = kit.text(
            stage,
            "a prime polynomial: it can't be factored",
            32.0,
            Role::fold(),
            v(0.0, 3.3),
        )?;
        let whole = u16::from(FOLD) | 0x100;
        let poly = kit.tex(stage, &fold_poly(FOLD), 60.0, Role::fold(), &[], v(-1.2, 2.45))?;
        let poly_x = stage.get_right(poly)[0] + 0.3;
        let poly_val = kit.tex(stage, &format!("= {whole}"), 44.0, Palette::dim(), &[], v(poly_x, 2.45))?;
        stage.move_to(poly_val, v(poly_x, 2.45), LEFT);
        let hex_at = v(stage.get_right(poly_val)[0] + 0.25, 2.42);
        let poly_hex = kit.mono(stage, &hexw(whole), 26.0, Palette::dim(), hex_at)?;
        stage.move_to(poly_hex, hex_at, LEFT);
        let rule = kit.tex(stage, &fold_rule(FOLD), 60.0, Role::fold(), &[], v(-1.2, 1.3))?;
        let rule_x = stage.get_right(rule)[0] + 0.3;
        let residue = kit.tex(stage, &format!("= {FOLD}"), 44.0, Role::fold(), &[], v(rule_x, 1.3))?;
        stage.move_to(residue, v(rule_x, 1.3), LEFT);
        let hex_at = v(stage.get_right(residue)[0] + 0.25, 1.27);
        let residue_hex = kit.mono(stage, &hexb(FOLD), 26.0, Role::fold(), hex_at)?;
        stage.move_to(residue_hex, hex_at, LEFT);
        let line = super::voice::Spoken::say(&kit, stage, &board, "10_4")?;
        play!(stage; fade_in(stage.arena_mut(), prime, ORIGIN, 1.0)?.rt(0.6));
        line.at(stage, 1)?;
        play!(stage; write(stage.arena(), poly).rt(1.4));
        play!(stage;
            fade_in(stage.arena_mut(), poly_val, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), poly_hex, ORIGIN, 1.0)?.rt(0.5),
        );
        line.at_phrase(stage, "Treat it as zero")?;
        play!(stage; write(stage.arena(), rule).rt(1.2));
        play!(stage;
            fade_in(stage.arena_mut(), residue, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), residue_hex, ORIGIN, 1.0)?.rt(0.5),
        );
        kit.hold(stage)?;

        // 10_5: 2 × 128 = x⁸; fold it: 16 + 8 + 4 + 1 = 29, not zero
        let mut reg = Register::build(&kit, stage, X7, cell, v(-0.9, -1.7), Role::byte(), true)?;
        let doubled_src = format!(r"{TWO} \times {X7} = x^{{8}}");
        let ask = kit.tex(stage, &doubled_src, 52.0, Palette::ink(), &[], v(-0.9, 0.2))?;
        kit.say(stage, "10_5")?;
        reg.show(stage, 0.5)?;
        play!(stage; write(stage.arena(), ask).rt(0.6));
        let fallen = reg.double(&kit, stage, 0.9)?;
        reg.fold_in(&kit, stage, fallen, FOLD, Role::fold(), 0.8)?;
        let folded = mul(TWO, X7, FOLD);
        assert_eq!(reg.value(), folded);
        let places: Vec<String> = (0..8u32)
            .rev()
            .filter(|&e| folded >> e & 1 == 1)
            .map(|e| (1u32 << e).to_string())
            .collect();
        // two rows, clear of the register and inside the safe area
        let parts = kit.tex(stage, &places.join(" + "), 44.0, Role::byte(), &[], v(4.6, -1.35))?;
        let total = kit.tex(stage, &format!("= {folded}"), 52.0, Role::byte(), &[], v(4.6, -2.0))?;
        let sum = group_of(stage, &[parts, total])?;
        let not_zero = kit.text(stage, "not zero", 32.0, Palette::green(), v(4.6, -2.6))?;
        play!(stage; write(stage.arena(), parts).rt(0.7));
        play!(stage; write(stage.arena(), total).rt(0.4));
        play!(stage; fade_in(stage.arena_mut(), not_zero, ORIGIN, 1.0)?.rt(0.4));
        // the listen window: the 07_3 slide again, blooming into 0x1D's chord
        let wav = super::sound::wav("10_5_fold", &super::sound::overflow(Some(FOLD), LISTEN_S))?;
        kit.sonify(stage, LISTEN_S, "10_5", &wav)?;
        play!(stage; indicate(stage.arena_mut(), rule, 1.08, Some([0.94, 0.68, 0.37]))?.rt(1.0));
        let lit: Vec<Mob> = (0..8).filter(|&k| reg.bits[k]).map(|k| reg.tiles[k]).collect();
        let mut pulse: Vec<Box<dyn Animation>> = Vec::new();
        for &t in &lit {
            let a = indicate(stage.arena_mut(), t, 1.2, Some([1.0, 0.85, 0.4]))?;
            pulse.push(stage.prepare(a)?);
        }
        stage.play_prepared_with(
            pulse,
            PlayOverrides {
                run_time: Some(1.4),
                ..PlayOverrides::default()
            },
        )?;
        stage.wait(0.6)?;
        let crc = lower_third(
            &kit,
            stage,
            "for programmers:",
            &format!(
                "written a CRC? Same move: shift, and if a bit falls off, ex-or in the polynomial. CRC-8/SAE-J1850 \
                 uses this very {}",
                hexb(FOLD)
            ),
        )?;
        play!(stage; fade_in(stage.arena_mut(), crc, v(0.0, 0.2), 1.0)?.rt(0.5));
        kit.hold(stage)?;

        // 10_6: every nonzero byte has a partner: 2 × 142 = 1; the byte field
        let partner = inv(TWO);
        let ask2 = kit.tex(
            stage,
            &format!(r"{TWO} \times {partner} ="),
            52.0,
            Palette::ink(),
            &[],
            v(-0.9, 0.2),
        )?;
        kit.say(stage, "10_6")?;
        play!(stage;
            fade_out(stage.arena_mut(), crc, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), sum, ORIGIN, 1.0)?.rt(0.4),
            fade_out(stage.arena_mut(), not_zero, ORIGIN, 1.0)?.rt(0.4),
            replacement_transform(ask, ask2).rt(0.6),
        );
        reg.set_value(&kit, stage, partner, 0.6)?;
        let fallen = reg.double(&kit, stage, 0.8)?;
        reg.fold_in(&kit, stage, fallen, FOLD, Role::fold(), 0.8)?;
        let product = mul(TWO, partner, FOLD);
        assert_eq!((reg.value(), product), (1, 1), "everything cancels but 1");
        let is_one = kit.tex(
            stage,
            &product.to_string(),
            52.0,
            Role::byte(),
            &[],
            v(stage.get_right(ask2)[0] + 0.35, 0.2),
        )?;
        play!(stage; write(stage.arena(), is_one).rt(0.4));
        let field = kit.text_styled(stage, "the byte field", 56.0, Role::byte(), v(4.7, 0.2), true, false)?;
        play!(stage; write(stage.arena(), field).rt(0.8));
        let name = math_lower_third(&kit, stage, "names to search:", r"\text{GF}(256) = \text{GF}(2^{8})")?;
        play!(stage; fade_in(stage.arena_mut(), name, v(0.0, 0.2), 1.0)?.rt(0.5));
        stage.wait(2.0)?;
        let why = precisely(
            &kit,
            stage,
            "the polynomial is irreducible over GF(2) (no factors with 0/1 coefficients), so no two nonzero bytes \
             multiply to 0; so each row of the times table has no repeats, so it contains 1",
        )?;
        play!(stage;
            fade_out(stage.arena_mut(), name, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), why, v(0.0, 0.2), 1.0)?.rt(0.5),
        );
        kit.hold(stage)?;

        // the byte field: a big reveal, a breath for the music (10_7, AES's
        // contrast, moved to 12_5 to keep this chapter under the gate); 1.3 s,
        // not the kit's 2.5, keeps it at 85 s with michael's reads (Codex B
        // review 2, P2: 86.15 s)
        kit.music_breath(stage, 1.3)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

/// 12_5 (was 10_7): first the shift-and-add code panel alone, centred and
/// large while the line says what it does (Dev R3 D1), its fold byte lit at
/// "the fold"; then, from the line's third sentence, it shrinks to the corner
/// beside RaptorQ's and AES's prime polynomials, 87 × 131 = 49 here and 193
/// there, and its 0x1D, 0x31 and 142 flip to 0x1B, 0xC1 and 141; the labels
/// lower-third. Call it once `line` (12_5) has started; it returns what it
/// put on screen.
pub(crate) fn aes_contrast(kit: &Kit, stage: &mut Stage<'_>, line: &super::voice::Spoken) -> fmn::Result<Vec<Mob>> {
    let ours = mul(LEFT_BYTE, RIGHT_BYTE, FOLD);
    let theirs = mul(LEFT_BYTE, RIGHT_BYTE, AES_FOLD);
    let col =
        |kit: &Kit, stage: &mut Stage<'_>, x: f64, name: &str, fold: u8, product: u8, ink: Srgb| -> fmn::Result<Mob> {
            let n = kit.text(stage, name, 36.0, ink, v(x, 2.75))?;
            let p = kit.tex(stage, &fold_poly(fold), 40.0, ink, &[], v(x, 1.95))?;
            let m = kit.tex(
                stage,
                &format!(r"{LEFT_BYTE} \times {RIGHT_BYTE} = {product}"),
                60.0,
                Palette::ink(),
                &[],
                v(x, 0.7),
            )?;
            let h = kit.mono(
                stage,
                &format!("{} · {} = {}", hexb(LEFT_BYTE), hexb(RIGHT_BYTE), hexb(product)),
                24.0,
                Palette::dim(),
                v(x, -0.05),
            )?;
            group_of(stage, &[n, p, m, h])
        };
    // built at its corner size and spot, shown centred at about 27 pt
    let mut panel = CodePanel::build(kit, stage, &code_lines(FOLD), 17.0, v(6.95, 3.6))?;
    let corner = stage.get_center(panel.group);
    let big = 27.0 / 17.0;
    stage.scale(panel.group, big);
    stage.move_to(panel.group, v(0.0, 0.2), ORIGIN);
    play!(stage; fade_in(stage.arena_mut(), panel.group, ORIGIN, 1.0)?.rt(0.6));
    let glow = Some([0.94, 0.68, 0.37]);
    line.at_phrase(stage, "Keep doubling with the fold")?;
    let (fold_seg, double) = (panel.get("fold"), panel.get("double"));
    play!(stage;
        indicate(stage.arena_mut(), fold_seg, 1.5, glow)?.rt(1.0),
        indicate(stage.arena_mut(), double, 1.15, glow)?.rt(1.0),
    );
    line.at_phrase(stage, "ex-or together")?;
    let pick = panel.get("pick");
    play!(stage; indicate(stage.arena_mut(), pick, 1.15, glow)?.rt(1.0));
    line.at(stage, 2)?;
    let back = panel
        .group
        .animate()
        .set_anim_args(AnimateArgs {
            run_time: Some(0.8),
            ..AnimateArgs::default()
        })?
        .scale(1.0 / big)?
        .move_to(corner, ORIGIN)?;
    play!(stage; back);
    let here = col(kit, stage, -4.6, "RaptorQ", FOLD, ours, Role::byte())?;
    let aes = col(kit, stage, -0.5, "AES", AES_FOLD, theirs, Palette::ink())?;
    play!(stage; fade_in(stage.arena_mut(), here, v(0.0, -0.15), 1.0)?.rt(0.6));
    play!(stage; fade_in(stage.arena_mut(), aes, v(0.0, -0.15), 1.0)?.rt(0.6));
    let aes_partner = (1..=255u8)
        .find(|&b| mul(TWO, b, AES_FOLD) == 1)
        .expect("AES's fold is prime");
    let s1 = panel.swap(kit, stage, "fold", &hexb(AES_FOLD), Role::fold())?;
    let s2 = panel.swap(kit, stage, "product", &hexb(theirs), Role::fold())?;
    let s3 = panel.swap(kit, stage, "partner", &aes_partner.to_string(), Role::fold())?;
    play!(stage; s1.rt(0.8), s2.rt(0.8), s3.rt(0.8));
    let labels = precisely(
        kit,
        stage,
        "every prime polynomial of degree 8 builds the same field with different names for its bytes: all fields \
         of 256 elements are alike; the polynomial is a choice of labels",
    )?;
    play!(stage; fade_in(stage.arena_mut(), labels, v(0.0, 0.2), 1.0)?.rt(0.5));
    // the swapped segments are new mobjects, outside the panel's group
    let mut shown = vec![panel.group, here, aes, labels];
    shown.extend(panel.keyed.iter().map(|&(_, m)| m));
    Ok(shown)
}

/// RaptorQ's prime polynomial, whole, in its role colour, with "can't be
/// factored" under it: what 17_7's "a polynomial refuses to factor" points
/// at (the swap rule x⁸ → x⁴ + x³ + x² + 1 alone factors: Maya R3 maya-02).
/// Centred at `at`, detached (fade it in). No halo: its stacked layers read
/// as rings (Codex B R11 P2).
pub fn prime_polynomial(kit: &Kit, stage: &mut Stage<'_>, at: Vec3) -> fmn::Result<Mob> {
    let poly = kit.tex(stage, &fold_poly(FOLD), 56.0, Role::fold(), &[], at)?;
    let below = stage.get_bottom(poly)[1] - 0.3;
    let note = kit.text(stage, "can't be factored", 28.0, Palette::dim(), v(at[0], below))?;
    group_of(stage, &[poly, note])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panels_code_is_the_models_multiply_for_both_folds() {
        // the panel prints the model's own shift-and-add with the fold byte
        // filled in; its comments are the products it would compute
        for fold in [FOLD, AES_FOLD] {
            let lines = code_lines(fold);
            let text: String = lines
                .iter()
                .map(|l| l.iter().map(|(s, _)| s.as_str()).collect::<String>() + "\n")
                .collect();
            assert!(text.contains(&format!("{{ {} }}", hexb(fold))), "{text}");
            let product = mul(LEFT_BYTE, RIGHT_BYTE, fold);
            assert!(text.contains(&format!("== {}", hexb(product))));
        }
        assert_eq!(hexb(FOLD), "0x1D");
        assert_eq!(hexb(mul(LEFT_BYTE, RIGHT_BYTE, FOLD)), "0x31");
        assert_eq!(hexb(mul(LEFT_BYTE, RIGHT_BYTE, AES_FOLD)), "0xC1");
        let text = |fold| {
            code_lines(fold)
                .concat()
                .into_iter()
                .map(|(s, _)| s)
                .collect::<String>()
        };
        assert!(text(FOLD).contains("mul(2, 142) == 1"));
        assert!(text(AES_FOLD).contains("mul(2, 141) == 1"), "AES's partner of 2");
    }

    #[test]
    fn the_rotation_has_zero_divisors_and_the_fold_does_not() {
        // 10_3: under x⁸ → 1, 3 · 255 = 0; x⁸ + 1 = (x + 1)(x⁷ + … + 1)
        assert_eq!(mul(THREE, ALL, ROTATE), 0);
        assert_eq!(mul(TWO, ALL, ROTATE), ALL, "11111111 rotates to itself");
        assert_eq!(mul(TWO, X7, ROTATE), 1);
        // under RaptorQ's fold no two nonzero bytes multiply to 0
        for a in 1..=255u8 {
            for b in 1..=255u8 {
                assert_ne!(mul(a, b, FOLD), 0, "{a} × {b}");
            }
        }
        // 10_5, 10_6
        assert_eq!(mul(TWO, X7, FOLD), 29);
        assert_eq!(mul(TWO, inv(TWO), FOLD), 1);
        assert_eq!(inv(TWO), 142);
        assert_eq!(fold_rule(FOLD), r"x^{8} \to x^{4} + x^{3} + x^{2} + 1");
        // x⁸ + 1 = (x + 1)(x⁷ + x⁶ + … + 1): the carry-less 3 · 255 is 0x101
        let product = crate::model::poly::foil(THREE, ALL)
            .iter()
            .fold(0u16, |acc, &e| acc ^ (1 << e));
        assert_eq!(product, 0x101);
        assert_eq!(fold_poly(AES_FOLD), r"x^{8} + x^{4} + x^{3} + x + 1");
    }
}
