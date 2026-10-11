//! Chapter 15, where the danger lives: RaptorQ's packets are plain ex-or
//! cards of about seven pieces (15_1); an extra piece P = A ⊕ B ⊕ C makes
//! the card A ⊕ B ⊕ C ⊕ P zero for any file, a free card (15_7); the sender
//! adds extra pieces so that fixed free cards all have the result zero,
//! drawn as the ten-piece grid, 27 × 27 (15_2); the bucket fills with ex-or
//! cards up to its last few slots, and the byte cards close them (15_3);
//! with ex-or there instead (15_4), the lottery again: the recorded
//! ablation, 70.7% against 0.646% (15_5).

use explainer_kit::props::{RowStrips, precisely, predict_pause};
use explainer_kit::{Kit, Palette, Timed, fade_all, hex, play, thousands, v};
use explainer_score::Stereo;
use fmn::prelude::*;

use super::draw::{OPLUS, Role, badge, flat, group_of, line_style};
use super::lottery::{Draw, rank_chord};
use super::props::{BucketProp, DangerMeter};
use super::sound;
use super::voice::{Spoken, board};
use crate::decoding::{
    ABLATION_BYTES, ABLATION_EX_OR, BIG, Block, RaceRun, TEN, ldpc_row, packet_sizes, pieces_per_packet, race,
    schematic_packets,
};
use crate::model::{PIECES, p_full, useless_chance};

const KEY: &str = "15_where_the_danger_lives";
/// The seed of 15_1's packets, and where 15_2's schematic search starts.
const SIZES_SEED: u32 = 1501;
const GRID_SEED_FROM: u32 = 1502;

/// An empty cell of the grid: a shade above the background.
fn empty() -> Srgb {
    hex("#161B24")
}

/// 15_7's extra piece on three pieces: P = A ⊕ B ⊕ C, with chapter 3's
/// values.
pub fn extra_piece() -> u8 {
    PIECES[0] ^ PIECES[1] ^ PIECES[2]
}

/// The shape model at ten pieces with exactly enough packets (fact sheet
/// B-F21, the description's "15 (model)"): uniform 0/1 rows for the S + K′
/// ex-or rows and uniform byte rows for the H byte rows, in Ruth's product
/// form 1 − Π_{i=H+1..L} (1 − 2⁻ⁱ) · Π_{i=1..H} (1 − 256⁻ⁱ).
pub fn shape_model_fail() -> f64 {
    let b = TEN;
    let ex_or: f64 = (b.h + 1..=b.l()).map(|i| 1.0 - 0.5f64.powi(i as i32)).product();
    let bytes: f64 = (1..=b.h).map(|i| 1.0 - 256f64.powi(-(i as i32))).product();
    1.0 - ex_or * bytes
}

/// 15_5's printed shape-model rate, in percent to two digits ("0.49").
pub fn shape_model_shown() -> String {
    super::byte_lottery::sig_digits(100.0 * shape_model_fail(), 2)
}

/// The extra piece's badge: a circle in the hero role with "P", `r` its size.
fn extra_badge(kit: &Kit, stage: &mut Stage<'_>, r: f64, at: Vec3) -> fmn::Result<Mob> {
    let ring = Circle::new()
        .radius(r * 0.95)
        .arc_center(at)
        .style(
            Style::default()
                .fill(Palette::hero(), 0.10)
                .stroke(Palette::hero(), (r * 9.0).max(1.5), 1.0),
        )
        .build();
    let ring = stage.arena_mut().add(ring);
    let letter = kit.text(stage, "P", r * 95.0, Palette::hero(), at)?;
    group_of(stage, &[ring, letter])
}

/// 15_7's free card, `h` tall and centred at `at`: A ⊕ B ⊕ C ⊕ P = 0 in
/// chapter 3's card style, the extra piece's badge last.
fn free_card(kit: &Kit, stage: &mut Stage<'_>, h: f64, at: Vec3) -> fmn::Result<Mob> {
    let (r, slot, gap) = (h * 0.27, h * 0.95, h * 0.55);
    let eq = kit.tex(stage, "= 0", h * 52.0, Palette::ink(), &[], at)?;
    let eq_w = stage.get_width(eq);
    let w = h * 0.5 + 4.0 * slot + 3.0 * gap + h * 0.3 + eq_w;
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(h * 0.16)
        .style(
            Style::default()
                .fill(super::draw::card_fill(), 0.96)
                .stroke(Role::ex_or(), (h * 6.0).max(2.0), 1.0),
        )
        .build()?
        .moved_to(at);
    let mut all = vec![stage.arena_mut().add(back)];
    let mut x = at[0] - w / 2.0 + h * 0.25;
    for k in 0..4 {
        if k > 0 {
            all.push(kit.tex(stage, OPLUS, h * 52.0, Role::ex_or(), &[], v(x + gap / 2.0, at[1]))?);
            x += gap;
        }
        let c = v(x + slot / 2.0, at[1]);
        all.push(if k < 3 {
            badge(kit, stage, k, r, c)?
        } else {
            extra_badge(kit, stage, r, c)?
        });
        x += slot;
    }
    stage.move_to(eq, v(x + h * 0.3, at[1]), LEFT);
    all.push(eq);
    group_of(stage, &all)
}

/// A packet as an ex-or card of `n` unnamed pieces (small squares joined by
/// ⊕), its edge in the ex-or role, `h` tall, its left edge's middle at `left`.
fn chip_card(kit: &Kit, stage: &mut Stage<'_>, n: usize, h: f64, left: Vec3) -> fmn::Result<Mob> {
    let (chip, gap, pad) = (h * 0.42, h * 0.55, h * 0.3);
    let w = 2.0 * pad + n as f64 * chip + (n as f64 - 1.0) * gap;
    let back = Rectangle::new()
        .width(w)
        .height(h)
        .corner_radius(h * 0.16)
        .style(
            Style::default()
                .fill(super::draw::card_fill(), 0.96)
                .stroke(Role::ex_or(), (h * 6.0).max(2.0), 1.0),
        )
        .build()?
        .moved_to(v(left[0] + w / 2.0, left[1]));
    let mut all = vec![stage.arena_mut().add(back)];
    let mut x = left[0] + pad + chip / 2.0;
    for k in 0..n {
        if k > 0 {
            all.push(kit.tex(
                stage,
                OPLUS,
                h * 46.0,
                Role::ex_or(),
                &[],
                v(x - chip / 2.0 - gap / 2.0, left[1]),
            )?);
        }
        let sq = Rectangle::new()
            .width(chip)
            .height(chip)
            .corner_radius(chip * 0.15)
            .style(
                Style::default()
                    .fill(Palette::ink(), 0.12)
                    .stroke(Palette::ink(), 2.0, 0.9),
            )
            .build()?
            .moved_to(v(x, left[1]));
        all.push(stage.arena_mut().add(sq));
        x += chip + gap;
    }
    group_of(stage, &all)
}

/// The rows of the ten-piece grid (15_2): the S free ex-or (LDPC) rows,
/// exact; the H free byte (HDPC) rows, dense over the first K′ + S columns
/// and 1 on their own (drawn dense: schematic); the K′ packet rows from
/// [`schematic_packets`]. Each cell: 0 empty, 1 ex-or, 2 byte.
pub fn grid_cells(block: Block, packets: &[Vec<usize>]) -> Vec<Vec<u8>> {
    let l = block.l();
    let mut rows = Vec::with_capacity(l);
    for i in 0..block.s {
        let mut r = vec![0u8; l];
        for c in ldpc_row(block, i) {
            r[c] = 1;
        }
        rows.push(r);
    }
    for i in 0..block.h {
        let mut r = vec![0u8; l];
        for c in r.iter_mut().take(block.k + block.s) {
            *c = 2;
        }
        r[block.k + block.s + i] = 2;
        rows.push(r);
    }
    for p in packets {
        let mut r = vec![0u8; l];
        for &c in p {
            r[c] = 1;
        }
        rows.push(r);
    }
    rows
}

/// The rank chord's draws for the last `n` slots of a race run: whether each
/// card was useful, and the level it arrived at.
fn draws(run: &RaceRun, n: usize) -> Vec<Draw> {
    let mut held = 0;
    run.useful
        .iter()
        .take(n)
        .map(|&useful| {
            let d = Draw { useful, held };
            held += u32::from(useful);
            d
        })
        .collect()
}

pub struct Danger {
    pub kit: Kit,
}

impl SceneConstruct for Danger {
    fn name(&self) -> &str {
        KEY
    }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let board = board(KEY)?;
        self.cheap(&kit, stage, &board)?;
        self.free_card(&kit, stage, &board)?;
        self.grid(&kit, stage, &board)?;
        // the 15_5 listen window is the chapter's breath: no second hold
        self.bucket(&kit, stage, &board)?;
        fade_all(stage, 1.0)?;
        kit.finish(stage)
    }
}

impl Danger {
    /// 15_1: plain ex-or packets of about seven pieces.
    fn cheap(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let sizes = packet_sizes(SIZES_SEED, 5);
        let line = Spoken::say(kit, stage, board, "15_1")?;
        let mut cards = Vec::new();
        for (i, &n) in sizes.iter().enumerate() {
            cards.push(chip_card(kit, stage, n, 0.68, v(-6.7, 2.55 - 1.2 * i as f64))?);
        }
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &c in &cards {
            let a = fade_in(stage.arena_mut(), c, v(-0.4, 0.0), 1.0)?;
            anims.push(stage.prepare(a.rt(0.7))?);
        }
        let group = explainer_kit::lagged(stage, anims, 0.2)?;
        stage.play(group)?;
        line.at(stage, 1)?;
        let none = kit.text(stage, "no byte amounts", 40.0, Role::ex_or(), v(4.4, 1.1))?;
        play!(stage; fade_in(stage.arena_mut(), none, ORIGIN, 1.0)?.rt(0.5));
        line.at(stage, 2)?;
        let avg = kit.tex(
            stage,
            &format!(r"\approx {}\ \text{{pieces per packet}}", pieces_per_packet().round()),
            52.0,
            Palette::ink(),
            &[],
            v(4.4, -0.1),
        )?;
        let note = kit.text(stage, "(average)", 30.0, Palette::dim(), v(4.4, -0.8))?;
        play!(stage;
            fade_in(stage.arena_mut(), avg, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), note, ORIGIN, 1.0)?.rt(0.6),
        );
        Ok(())
    }

    /// 15_2: the ten-piece grid, its free cards' results zero.
    fn grid(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let b = TEN;
        let (_, packets, out) = schematic_packets(b, b.k, GRID_SEED_FROM);
        let cells = grid_cells(b, &packets);
        let paint = |v: u8| match v {
            1 => Role::ex_or(),
            2 => Role::byte(),
            _ => empty(),
        };
        let colors: Vec<Vec<Srgb>> = cells.iter().map(|r| r.iter().map(|&c| paint(c)).collect()).collect();
        let cell = 0.175;
        let top_left = v(-2.3, 2.4);
        let at = |r: f64, c: f64| v(top_left[0] + cell * c, top_left[1] - cell * r);
        let line = Spoken::say(kit, stage, board, "15_2")?;
        fade_all(stage, 0.6)?;
        let strips = RowStrips::build(stage, top_left, (cell, cell), &colors)?;
        // the column and row groups, named
        let col_heads = [
            (format!("{} pieces", b.k), 0.0, b.k as f64, Palette::ink()),
            (
                format!("{} extra pieces", b.extra()),
                b.k as f64,
                b.l() as f64,
                Palette::ink(),
            ),
        ];
        let mut labels = Vec::new();
        for (s, c0, c1, color) in &col_heads {
            let p = at(0.0, (c0 + c1) / 2.0);
            labels.push(kit.text(stage, s, 26.0, *color, v(p[0], p[1] + 0.3))?);
        }
        // every column is an unknown: the originals are cards over them too
        let all = at(0.0, b.l() as f64 / 2.0);
        labels.push(kit.text(
            stage,
            &format!("{} unknowns", b.l()),
            30.0,
            Palette::ink(),
            v(all[0], all[1] + 0.72),
        )?);
        let row_heads = [
            (format!("{} free ex-or cards", b.s), 0, b.s, Role::ex_or()),
            (format!("{} free byte cards", b.h), b.s, b.s + b.h, Role::byte()),
            (format!("{} packets", b.k), b.s + b.h, b.l(), Palette::ink()),
        ];
        for (s, r0, r1, color) in &row_heads {
            let p = at((*r0 + *r1) as f64 / 2.0, 0.0);
            let m = kit.text(stage, s, 26.0, *color, ORIGIN)?;
            stage.move_to(m, v(p[0] - 0.25, p[1]), RIGHT);
            labels.push(m);
        }
        let labels = group_of(stage, &labels)?;
        play!(stage;
            fade_in(stage.arena_mut(), strips.group, ORIGIN, 1.0)?.rt(1.0),
            fade_in(stage.arena_mut(), labels, ORIGIN, 1.0)?.rt(1.0),
        );
        // so that many fixed cards are zero like this
        let mut zeros = Vec::new();
        for r in 0..b.s + b.h {
            let p = at(r as f64 + 0.5, b.l() as f64);
            zeros.push(kit.tex(stage, "= 0", 22.0, Palette::dim(), &[], v(p[0] + 0.35, p[1]))?);
        }
        let zeros = group_of(stage, &zeros)?;
        play!(stage; fade_in(stage.arena_mut(), zeros, v(-0.2, 0.0), 1.0)?.rt(0.8).lag(0.05));
        let never = kit.text_left(stage, "free cards, never sent:", 28.0, Palette::dim(), v(3.25, 1.35))?;
        let know = kit.text_left(stage, "both sides know them", 28.0, Palette::dim(), v(3.25, 0.95))?;
        let never = group_of(stage, &[never, know])?;
        play!(stage; fade_in(stage.arena_mut(), never, ORIGIN, 1.0)?.rt(0.5));
        // even one that no packet mixed sits in an equation
        line.at(stage, 2)?;
        // a piece no packet mixed: its column is empty in every packet row...
        let col_x = at(0.0, out as f64)[0];
        let packet_top = at((b.s + b.h) as f64, 0.0)[1];
        let packet_bottom = at(b.l() as f64, 0.0)[1];
        let red = Rectangle::new()
            .width(cell + 0.06)
            .height(packet_top - packet_bottom + 0.06)
            .style(line_style(Role::lost(), 4.0))
            .build()?
            .moved_to(v(col_x + cell / 2.0, (packet_top + packet_bottom) / 2.0));
        let red = stage.arena_mut().add(red);
        play!(stage; show_creation(red).rt(0.6));
        // ...and it sits in a free ex-or card: those cells light
        let mut lit = Vec::new();
        for i in 0..b.s {
            if ldpc_row(b, i).contains(&out) {
                let p = at(i as f64 + 0.5, out as f64 + 0.5);
                lit.push(
                    Rectangle::new()
                        .width(cell + 0.08)
                        .height(cell + 0.08)
                        .style(Style::default().fill(Role::lit(), 0.35).stroke(Role::lit(), 4.0, 1.0))
                        .build()?
                        .moved_to(p),
                );
            }
        }
        let lit = stage.arena_mut().add(v_group(lit));
        play!(stage; fade_in(stage.arena_mut(), lit, ORIGIN, 1.5)?.rt(0.6));
        let tag = kit.text_left(stage, "packet rows: a random draw", 24.0, Palette::dim(), v(3.25, -1.0))?;
        let tag2 = kit.text_left(stage, "in the standard's shape", 24.0, Palette::dim(), v(3.25, -1.38))?;
        let note = precisely(
            kit,
            stage,
            "RaptorQ adds S + H extra unknowns and S + H fixed equations with result zero; packets and originals \
             alike are ex-or cards over all of them, so the sender solves once and sends the K originals first \
             (the K′ − K padding pieces are known zeros, never sent)",
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), tag2, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), note, v(0.0, 0.2), 1.0)?.rt(0.6),
        );
        Ok(())
    }

    /// 15_7: a free card on three pieces, checkable with chapter 3's values.
    fn free_card(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let line = Spoken::say(kit, stage, board, "15_7")?;
        fade_all(stage, 0.6)?;
        let p = extra_piece();
        let [a, b, c, ..] = PIECES;
        // the pieces A, B and C, with their values
        let r = 0.36;
        let mut row = Vec::new();
        for (i, x) in [-5.6, -3.2, -0.8].into_iter().enumerate() {
            let m = badge(kit, stage, i, r, v(x, 2.4))?;
            let val = kit.text_left(
                stage,
                &format!("= {}", PIECES[i]),
                40.0,
                Palette::ink(),
                v(x + 0.5, 2.4),
            )?;
            row.push(group_of(stage, &[m, val])?);
        }
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in &row {
            let a = fade_in(stage.arena_mut(), m, v(0.0, -0.2), 1.0)?;
            anims.push(stage.prepare(a.rt(0.5))?);
        }
        let shown = explainer_kit::lagged(stage, anims, 0.15)?;
        stage.play(shown)?;
        // with three pieces, add P = A ⊕ B ⊕ C
        line.at(stage, 1)?;
        let pb = extra_badge(kit, stage, r, v(2.2, 2.4))?;
        let pv = kit.text_left(stage, &format!("= {p}"), 40.0, Palette::hero(), v(2.7, 2.4))?;
        let extra = kit.text(stage, "an extra piece", 26.0, Palette::dim(), v(2.6, 1.8))?;
        let def = kit.tex(
            stage,
            &format!(r"P = A \oplus B \oplus C = {a} \oplus {b} \oplus {c} = {p}"),
            44.0,
            Palette::ink(),
            &[],
            v(0.0, 0.9),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), pb, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), pv, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), extra, ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), def, v(0.0, 0.15), 1.0)?.rt(0.7),
        );
        // then A ⊕ B ⊕ C ⊕ P is always zero
        line.at(stage, 2)?;
        let card = free_card(kit, stage, 0.9, v(0.0, -0.75))?;
        let check = kit.tex(
            stage,
            &format!(r"{a} \oplus {b} \oplus {c} \oplus {p} = {}", a ^ b ^ c ^ p),
            36.0,
            Palette::dim(),
            &[],
            v(0.0, -1.75),
        )?;
        play!(stage; fade_in(stage.arena_mut(), card, v(0.0, 0.2), 1.0)?.rt(0.7));
        play!(stage; fade_in(stage.arena_mut(), check, ORIGIN, 1.0)?.rt(0.5));
        // both sides know that card, so it's never sent
        line.at(stage, 3)?;
        let never = kit.text(
            stage,
            "both sides know this card, so it is never sent",
            30.0,
            Palette::dim(),
            v(0.0, -2.55),
        )?;
        play!(stage; fade_in(stage.arena_mut(), never, ORIGIN, 1.0)?.rt(0.5));
        // it's a free card
        line.at(stage, 4)?;
        let tag = kit.text(stage, "a free card", 40.0, Palette::hero(), v(0.0, 0.05))?;
        play!(stage;
            fade_out(stage.arena_mut(), def, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), tag, ORIGIN, 1.0)?.rt(0.5),
            fmn::animation::indicate(stage.arena_mut(), card, 1.08, None)?.rt(0.8),
        );
        Ok(())
    }

    /// 15_3–15_5: the bucket fills with ex-or cards to its last few slots,
    /// the byte cards close it; the ablation counters.
    fn bucket(
        &self,
        kit: &Kit,
        stage: &mut Stage<'_>,
        board: &explainer_kit::storyboard::ChapterBoard,
    ) -> fmn::Result<()> {
        let big = BIG;
        let ten = TEN;
        let line = Spoken::say(kit, stage, board, "15_3")?;
        fade_all(stage, 0.6)?;
        // the big block's bucket, a schematic: a break, then the last slots
        let bucket = BucketProp::build(stage, 1, v(-2.4, -0.75), 2.6, 4.3)?;
        let meter = DangerMeter::on(kit, stage, &bucket, 0.0)?;
        let label = kit.text(
            stage,
            &format!("our picture's block: {} slots", thousands(big.l() as u64)),
            26.0,
            Palette::ink(),
            v(-2.4, -3.25),
        )?;
        let inset = BucketProp::build(stage, ten.l(), v(3.2, -0.75), 1.5, 4.3)?;
        let inset_label = kit.text(
            stage,
            &format!("ten pieces: {} slots", ten.l()),
            26.0,
            Palette::ink(),
            v(3.2, -3.25),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), bucket.group, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), meter.group, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), label, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), inset.group, ORIGIN, 1.0)?.rt(0.6),
            fade_in(stage.arena_mut(), inset_label, ORIGIN, 1.0)?.rt(0.6),
        );
        stage.add_to_scene(meter.fill)?;
        line.at(stage, 2)?;
        // ex-or cards first: they fill all but the last H slots
        let gold_from = 0.82;
        let blue = bucket.level(stage, 0.0, gold_from - 0.04, Role::ex_or())?;
        let brk = Line::new(
            v(
                -2.4 - 1.15,
                -0.75 - 4.3 / 2.0 + 0.14 + (4.3 - 0.28) * (gold_from - 0.02),
            ),
            v(
                -2.4 + 1.15,
                -0.75 - 4.3 / 2.0 + 0.14 + (4.3 - 0.28) * (gold_from - 0.02),
            ),
        )
        .style(
            Style::default()
                .fill(Palette::dim(), 0.0)
                .stroke(Palette::dim(), 2.0, 0.8),
        )
        .build()?;
        let brk = stage.arena_mut().add(brk);
        let mut inset_blue = Vec::new();
        for i in 0..ten.ex_or_slots() {
            inset_blue.push(inset.water(stage, i, Role::ex_or())?);
        }
        let inset_blue = group_of(stage, &inset_blue)?;
        let blue_word = kit.text(
            stage,
            &format!("{} ex-or cards", thousands(big.ex_or_slots() as u64)),
            26.0,
            Palette::ink(),
            v(-2.4, -1.3),
        )?;
        let grow = fmn::animation::grow_from_edge(stage.arena_mut(), blue, DOWN, None)?;
        play!(stage;
            grow.rt(1.4),
            fade_in(stage.arena_mut(), inset_blue, v(0.0, -0.2), 1.0)?.rt(1.4).lag(0.05),
            fade_in(stage.arena_mut(), brk, ORIGIN, 1.0)?.rt(1.4),
            fade_in(stage.arena_mut(), blue_word, ORIGIN, 1.0)?.rt(1.4),
        );
        line.at(stage, 3)?;
        // the byte cards close the last H slots, where the danger lives
        let gold = bucket.level(stage, gold_from, 1.0, Role::byte())?;
        let mut inset_gold = Vec::new();
        for i in ten.ex_or_slots()..ten.l() {
            inset_gold.push(inset.water(stage, i, Role::byte())?);
        }
        let inset_gold_g = group_of(stage, &inset_gold)?;
        let gold_word = kit.text_left(
            stage,
            &format!("{} byte cards", big.h),
            28.0,
            Role::byte(),
            v(-0.85, 1.15),
        )?;
        let inset_words = [
            kit.text_left(
                stage,
                &format!("{} ex-or", ten.ex_or_slots()),
                28.0,
                Role::ex_or(),
                v(4.15, -1.6),
            )?,
            kit.text_left(stage, &format!("{} byte", ten.h), 28.0, Role::byte(), v(4.15, 0.85))?,
        ];
        let byte_chance = 1.0 - p_full(256, 1, 0);
        let reading = meter.readout(kit, stage, &format!("1/{}", (1.0 / byte_chance).round()))?;
        play!(stage;
            fade_in(stage.arena_mut(), gold, v(0.0, -0.15), 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), inset_gold_g, v(0.0, -0.15), 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), gold_word, ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), inset_words[0], ORIGIN, 1.0)?.rt(0.8),
            fade_in(stage.arena_mut(), inset_words[1], ORIGIN, 1.0)?.rt(0.8),
            meter.to(byte_chance, 1.0)?,
            fade_in(stage.arena_mut(), reading, ORIGIN, 1.0)?.rt(0.6),
        );

        // 15_4: what if those last cards were plain ex-or cards?
        let ex_or_chance = useless_chance(ten.l() as u32 - 1, ten.l() as u32);
        let half = meter.readout(kit, stage, &format!("1/{}", (1.0 / ex_or_chance).round()))?;
        kit.say(stage, "15_4")?;
        let mut anims: Vec<Box<dyn Animation>> = Vec::new();
        for &m in inset_gold.iter().chain([&gold]) {
            let a = m.animate().set_color(Role::ex_or())?;
            anims.push(stage.prepare(a)?);
        }
        // the labels say what the last slots now hold
        let ex_or_word = kit.text_left(stage, &format!("{} ex-or cards", big.h), 28.0, Role::ex_or(), ORIGIN)?;
        let at = stage.get_left(gold_word);
        stage.move_to(ex_or_word, at, LEFT);
        let a = fmn::animation::replacement_transform(gold_word, ex_or_word);
        anims.push(stage.prepare(a)?);
        let ten_ex_or = kit.text_left(stage, &format!("{} ex-or", ten.h), 28.0, Role::ex_or(), ORIGIN)?;
        let at = stage.get_left(inset_words[1]);
        stage.move_to(ten_ex_or, at, LEFT);
        let a = fmn::animation::replacement_transform(inset_words[1], ten_ex_or);
        anims.push(stage.prepare(a)?);
        let a = fmn::animation::replacement_transform(reading, half);
        anims.push(stage.prepare(a)?);
        let a = meter.to(ex_or_chance, 1.0)?;
        anims.push(stage.prepare(a)?);
        stage.play_prepared_with(
            anims,
            PlayOverrides {
                run_time: Some(1.0),
                ..PlayOverrides::default()
            },
        )?;
        // the "?" leaves with the wipe at 15_5
        predict_pause(kit, stage, v(0.6, 0.3))?;

        // 15_5: the recorded ablation, ten pieces
        let line = Spoken::say(kit, stage, board, "15_5")?;
        stage.clear_updaters(meter.fill, true);
        fade_all(stage, 0.6)?;
        let sides = [
            ("plain ex-or last cards", Role::ex_or(), -3.5, ABLATION_EX_OR),
            ("byte last cards", Role::byte(), 3.5, ABLATION_BYTES),
        ];
        let mut heads = Vec::new();
        for (title, color, x, _) in &sides {
            heads.push(kit.text(stage, title, 36.0, *color, v(*x, 3.3))?);
        }
        // why six in a thousand is a little worse than one in 256 (R3 Dev D3)
        let caption = precisely(
            kit,
            stage,
            &format!(
                "{} pieces, {} and {} trials, our RFC 6330 implementation. Worse than 1 in {} because RaptorQ's \
                 ex-or cards are sparse and fixed: even random rows of its shape fail {}%",
                ten.k,
                thousands(u64::from(ABLATION_EX_OR.1)),
                thousands(u64::from(ABLATION_BYTES.1)),
                (1.0 / (1.0 - p_full(256, 1, 0))).round(),
                shape_model_shown(),
            ),
        )?;
        play!(stage;
            fade_in(stage.arena_mut(), heads[0], ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), heads[1], ORIGIN, 1.0)?.rt(0.5),
            fade_in(stage.arena_mut(), caption, v(0.0, 0.2), 1.0)?.rt(0.5),
        );
        let t = stage.add_value_tracker(0.0);
        let mut live = Vec::new();
        let mut counts = Vec::new();
        let bar_w = 4.4;
        for (_, color, x, (fails, trials)) in &sides {
            let share = f64::from(*fails) / f64::from(*trials);
            let places = if share < 0.01 { 3 } else { 1 };
            let pct = kit.live_value(
                stage,
                move |s| 100.0 * share * s.tracker_value(t).unwrap_or(0.0),
                places,
                false,
                84.0,
                *color,
                v(*x + 0.6, 1.75),
                RIGHT,
            )?;
            let sign = kit.text_left(stage, "% fail", 44.0, *color, v(*x + 0.75, 1.7))?;
            let of = kit.text(
                stage,
                &format!(
                    "{} of {} failed",
                    thousands(u64::from(*fails)),
                    thousands(u64::from(*trials))
                ),
                30.0,
                Palette::ink(),
                v(*x, 0.95),
            )?;
            // the failed share as a bar, red on the whole
            let left = x - bar_w / 2.0;
            let whole = explainer_kit::rect(v(left, 0.25), bar_w, 0.3, flat(Palette::faint(), 0.6))?;
            let whole = stage.arena_mut().add(whole);
            let bar = explainer_kit::live_bar(stage, t, v(left, 0.4), bar_w * share, 0.3, Role::lost())?;
            play!(stage;
                fade_in(stage.arena_mut(), sign, ORIGIN, 1.0)?.rt(0.4),
                fade_in(stage.arena_mut(), whole, ORIGIN, 1.0)?.rt(0.4),
            );
            live.extend([pct, bar]);
            counts.push(of);
        }
        line.at(stage, 1)?;
        stage.play(explainer_kit::tween(t, 1.0, 1.6)?)?;
        for m in live {
            stage.clear_updaters(m, true);
        }
        // the counts arrive with the percentages they make, never before
        play!(stage;
            fade_in(stage.arena_mut(), counts[0], ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), counts[1], ORIGIN, 1.0)?.rt(0.4),
        );
        // the listen window: one run of each kind fills its last slots in time
        // with its rank chord: the ex-or run thuds and stops a slot short, the
        // byte run completes
        let listen = board
            .beats
            .iter()
            .find(|b| b.id == "15_5")
            .and_then(explainer_kit::storyboard::Beat::listen_s)
            .unwrap_or(3.0);
        let runs = chord_runs(ten.k);
        let times = chord_times(ten.k, listen);
        let mut buckets = Vec::new();
        for (_, _, x, _) in &sides {
            let b = BucketProp::build(stage, ten.k, v(*x, -1.55), 1.1, 2.3)?;
            buckets.push(b);
        }
        play!(stage;
            fade_in(stage.arena_mut(), buckets[0].group, ORIGIN, 1.0)?.rt(0.4),
            fade_in(stage.arena_mut(), buckets[1].group, ORIGIN, 1.0)?.rt(0.4),
        );
        let wav = sound::wav("15_5_rank_chords", &last_slots_chord(ten.k, listen))?;
        kit.sonify(stage, listen, "15_5", &wav)?;
        let start = stage.scene().time().to_f64();
        let mut events: Vec<(f64, usize, Draw)> = Vec::new();
        for (side, (draws, at)) in runs.iter().zip(&times).enumerate() {
            for (d, &t0) in draws.iter().zip(at) {
                events.push((t0, side, *d));
            }
        }
        events.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (t0, side, d) in events {
            let now = stage.scene().time().to_f64() - start;
            if t0 > now + 1e-3 {
                stage.wait(t0 - now)?;
            }
            let color = if side == 0 { Role::ex_or() } else { Role::byte() };
            if d.useful {
                let w = buckets[side].water(stage, d.held as usize, color)?;
                play!(stage; fade_in(stage.arena_mut(), w, v(0.0, -0.1), 1.0)?.rt(0.08));
            } else {
                // a useless card: a red flash on the rim
                let rim = buckets[side].rim();
                let flash = explainer_kit::rect(
                    v(buckets[side].centre[0] - 0.55, rim - 0.02),
                    1.1,
                    0.08,
                    flat(Role::lost(), 0.95),
                )?;
                let flash = stage.arena_mut().add(flash);
                play!(stage; fade_in(stage.arena_mut(), flash, ORIGIN, 1.0)?.rt(0.06));
                play!(stage; fade_out(stage.arena_mut(), flash, ORIGIN, 1.0)?.rt(0.06));
            }
        }
        let left = listen - (stage.scene().time().to_f64() - start);
        if left > 0.0 {
            stage.wait(left)?;
        }
        Ok(())
    }
}

/// The two runs 15_5 plays: the race's first ex-or run that leaves a slot
/// empty with exactly enough cards, and its first byte run with every card
/// useful; each as rank-chord draws over `pieces` cards.
fn chord_runs(pieces: usize) -> [Vec<Draw>; 2] {
    let race = race();
    let ex_or = race
        .ex_or
        .iter()
        .find(|r| r.useful.iter().take(pieces).any(|u| !u))
        .expect("most ex-or runs need a spare");
    let byte = race
        .byte
        .iter()
        .find(|r| r.useful.iter().take(pieces).all(|&u| u))
        .expect("most byte runs need none");
    [draws(ex_or, pieces), draws(byte, pieces)]
}

/// When each card of the two runs sounds, seconds into a window of
/// `seconds`: the ex-or run in the first half of the time before the
/// score's 0.8 s fade, the byte run in the second (`rank_chord` puts card k
/// at 0.15 + k·step).
fn chord_times(pieces: usize, seconds: f64) -> [Vec<f64>; 2] {
    let half = (seconds - 0.8) / 2.0;
    let step = half / pieces as f64;
    [
        (0..pieces).map(|k| 0.15 + step * k as f64).collect(),
        (0..pieces).map(|k| half + 0.15 + step * 0.9 * k as f64).collect(),
    ]
}

/// 15_5's sound: the rank chord of each run of [`chord_runs`] at the
/// times of [`chord_times`]: thuds and an unfinished chord, then a complete one.
fn last_slots_chord(pieces: usize, seconds: f64) -> Stereo {
    let [ex_or, byte] = chord_runs(pieces);
    let half = (seconds - 0.8) / 2.0;
    let step = half / pieces as f64;
    let mut out = rank_chord(&ex_or, step, seconds);
    let second = rank_chord(&byte, step * 0.9, seconds - half);
    out.mix_in(&second, (half * explainer_score::FS) as usize, 1.0);
    sound::finish(out, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Narration truth for 15 on the data the chapter draws.
    #[test]
    fn the_voice_of_15_matches_its_data() {
        // 15_1: "about seven pieces on average"
        assert_eq!(pieces_per_packet().round(), 7.0);
        // 15_2: every piece, old or new, is in some free card (LDPC rows here;
        // the HDPC rows are dense besides)
        let (_, packets, out) = schematic_packets(TEN, TEN.k, GRID_SEED_FROM);
        let cells = grid_cells(TEN, &packets);
        assert_eq!(cells.len(), TEN.l());
        assert!(cells[TEN.s + TEN.h..].iter().all(|r| r[out] == 0), "no packet mixes it");
        for c in 0..TEN.l() {
            assert!(
                cells[..TEN.s + TEN.h].iter().any(|r| r[c] != 0),
                "column {c} in a free card"
            );
        }
        // 15_7: P = A ⊕ B ⊕ C makes A ⊕ B ⊕ C ⊕ P "always zero": for any
        // values, not just chapter 3's
        let p = extra_piece();
        assert_eq!(PIECES[0] ^ PIECES[1] ^ PIECES[2] ^ p, 0);
        assert!((0..=255u8).all(|a| (0..=255u8).all(|b| a ^ b ^ (a ^ b) == 0)));
        // 15_3: the ex-or cards fill all but the last ten to sixteen slots
        // (H's range is pinned in facts)
        for b in [TEN, BIG, crate::decoding::LARGEST] {
            assert!(
                (crate::decoding::H_RANGE.0..=crate::decoding::H_RANGE.1).contains(&b.h),
                "{}",
                b.k
            );
            assert_eq!(b.l() - b.ex_or_slots(), b.h);
        }
        // 15_5: "About seventy-one percent failure"; "about six in a thousand"
        let x = f64::from(ABLATION_EX_OR.0) / f64::from(ABLATION_EX_OR.1);
        assert_eq!((x * 100.0).round(), 71.0);
        let b = f64::from(ABLATION_BYTES.0) / f64::from(ABLATION_BYTES.1);
        assert_eq!((b * 1000.0).round(), 6.0);
        // 15_5's lower-third: the shape model sits between 1 in 256 and the measurement
        let model = shape_model_fail();
        assert!(1.0 - p_full(256, 1, 0) < model && model < b, "{model}");
        assert_eq!(shape_model_shown(), "0.49");
        let s = last_slots_chord(TEN.k, 3.0);
        assert!(s.is_finite() && s.peak() > 0.0);
    }
}
