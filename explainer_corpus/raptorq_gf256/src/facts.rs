//! Every number Video B shows or says, computed here and checked two ways:
//! `pin` against a literal from the fact sheet, and `./x raptorq_gf256
//! facts-check` against `oracle.py` (same keys). Chapters read their numbers
//! from here; a typed number is a defect (doctrine 2). The arithmetic and the
//! data live in `model.rs`.

use explainer_kit::Facts;

use crate::decoding::{
    ABLATION_BYTES, ABLATION_EX_OR, ATSC_A331_YEAR, BIG, CARD_SEED, DENSE_PIECES, ERDOS_YEAR, H_RANGE, LARGEST,
    LT_CODES_YEAR, MBMS_RAPTOR_YEAR, MEASURED_K10, MEASURED_LARGER, PEEL_EXTRA_PERCENT, PEEL_PIECES, PeelRun,
    RACE_PIECES, RACE_SEED, RACES, RFC_6330_YEAR, RFC_ONE_IN, STEPS_PER_SECOND, TEN, TOY_CARDS, degree_mass,
    dense_steps, dense_time, exact_mean_spares, expected_untouched, histogram, mean_degree, mean_spares,
    one_amount_lights_exhaustive, peel_runs, peel_toy, pieces_per_packet, race, race_means_within_4se,
    random_byte_card,
};

use crate::model::{
    AES_FOLD, CARDS, DECK_PACKETS, FIELD_SEED, FIFTH_CARD, FOLD, GLYPHS, HOOK_PIECES, LOSS_ONE_IN, PHONES, PIECE_NAMES,
    PIECES, RING, album_mean, album_mean_floor, carousel_mean_caught, carousel_mean_sent, deck_mix, field_done_by_loop,
    logs, mean_loops, mul, order, p_done_by, p_done_in_loop_2_or_3, p_fail, p_fail_limit, p_fail_limit_times, p_full,
    p_full_limit, p_waiting_after, rank_gf2, rank_gf256, result, spares_for, unluckiest_of_a_million, useless_chance,
};

/// A double as Python's `repr` prints it (oracle.py's format): the shortest
/// digits that read back to the same double, in fixed notation from 10⁻⁴ up
/// to 10¹⁶ and as d.ddde±XX outside it.
pub fn py(x: f64) -> String {
    if x == 0.0 {
        return "0.0".to_owned();
    }
    let sci = format!("{x:e}");
    let (mantissa, exp) = sci.split_once('e').expect("{:e} always has an exponent");
    let exp: i32 = exp.parse().expect("an integer exponent");
    let sign = if x < 0.0 { "-" } else { "" };
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let n = digits.len() as i32;
    if (-4..16).contains(&exp) {
        if exp < 0 {
            format!("{sign}0.{}{digits}", "0".repeat((-exp - 1) as usize))
        } else if n <= exp + 1 {
            format!("{sign}{digits}{}.0", "0".repeat((exp + 1 - n) as usize))
        } else {
            let (int, frac) = digits.split_at((exp + 1) as usize);
            format!("{sign}{int}.{frac}")
        }
    } else {
        let (lead, rest) = digits.split_at(1);
        let m = if rest.is_empty() {
            lead.to_owned()
        } else {
            format!("{lead}.{rest}")
        };
        format!("{sign}{m}e{}{:02}", if exp < 0 { '-' } else { '+' }, exp.abs())
    }
}

/// A list as the oracle prints it.
fn list<T: ToString>(items: impl IntoIterator<Item = T>) -> String {
    items.into_iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    hook(&mut f)?;
    // 02: bits and ex-or (B-X4)
    f.pin(
        "B-X4.place_values_8_bits",
        list((0..8).map(|i| 1u32 << i)),
        "1,2,4,8,16,32,64,128",
        "fact sheet B-X4",
    )?;
    f.pin("B-X4.bits_of_5", format!("{:b}", 5), "101", "fact sheet B-X4")?;
    f.pin("B-X4.bits_of_7", format!("{:b}", 7), "111", "fact sheet B-X4")?;
    f.pin("B-X4.byte_max_2^8_minus_1", u8::MAX, "255", "fact sheet B-X4")?;
    f.pin("B-X4.xor_5_7", 5 ^ 7, "2", "fact sheet B-X4")?;
    f.pin(
        "B-X4.bits_of_xor_5_7",
        format!("{:03b}", 5 ^ 7),
        "010",
        "fact sheet B-X4",
    )?;
    f.pin("B-X4.xor_2_7", 2 ^ 7, "5", "fact sheet B-X4")?;
    // 03: the cards, with A, B, C, D = 5, 3, 7, 2
    let (a, b, c, d) = (PIECES[0], PIECES[1], PIECES[2], PIECES[3]);
    f.pin("B-F3.A_xor_C", result(CARDS[0]), "2", "fact sheet B-F3")?;
    f.pin("B-F3.B_xor_C_xor_D", result(CARDS[1]), "6", "fact sheet B-F3")?;
    f.pin("B-F3.A_xor_B", result(CARDS[2]), "6", "fact sheet B-F3")?;
    f.pin("B-F3.A_xor_B_xor_C_xor_D", result(CARDS[3]), "3", "fact sheet B-F3")?;
    f.pin("B-F3.rank_gf2_4_cards", rank_gf2(&CARDS), "4", "fact sheet B-F3")?;
    f.pin("B-F3.B_xor_C", result(FIFTH_CARD), "4", "fact sheet B-F3")?;
    f.pin(
        "B-F3.card1_xor_card3",
        result(CARDS[0]) ^ result(CARDS[2]),
        "4",
        "fact sheet B-F3",
    )?;
    let mut five = CARDS.to_vec();
    five.push(FIFTH_CARD);
    f.pin(
        "B-F3.rank_gf2_with_5th_card_BxC",
        rank_gf2(&five),
        "4",
        "fact sheet B-F3",
    )?;
    f.pin(
        "B-F3.D_from_card2_and_BxC",
        result(CARDS[1]) ^ result(FIFTH_CARD),
        "2",
        "fact sheet B-F3",
    )?;
    // 03_8: the toy deck (a stand-in for RFC 6330's recipe, out of scope)
    f.add("B-S4.toy_deck.packets", list(DECK_PACKETS))?;
    f.add("B-S4.toy_deck.masks", list(DECK_PACKETS.map(deck_mix)))?;
    // 03_9: the name card
    f.pin("B-S1.sourced.rfc6330_number", 6330, "6330", "RFC 6330 header")?;
    f.pin(
        "B-S1.sourced.rfc6330_year",
        2011,
        "2011",
        "RFC 6330 header, August 2011",
    )?;
    // 04–05: the lottery at three pieces, exact in binary fractions
    for (k, want) in [(1, "0.5"), (2, "0.375"), (3, "0.328125")] {
        f.pin(
            &format!("B-F4.success_K{k}"),
            py(p_full(2, k, 0)),
            want,
            "fact sheet B-F4",
        )?;
    }
    f.pin(
        "B-F4.success_K3_check_7_8x6_8x4_8",
        py(7.0 / 8.0 * (6.0 / 8.0) * (4.0 / 8.0)),
        "0.328125",
        "fact sheet B-F4",
    )?;
    f.pin("B-F4.fail_K3", py(p_fail(2, 3, 0)), "0.671875", "1 − 21/64")?;
    f.pin(
        "B-F4.success_limit",
        py(p_full_limit(2, 0)),
        "0.2887880950866024",
        "fact sheet B-F4",
    )?;
    f.pin(
        "B-F4.useless_prob_3pieces_2cards_held",
        py(useless_chance(2, 3)),
        "0.5",
        "fact sheet B-F4",
    )?;
    // 05–06: the many-piece lottery and its spares
    for (h, want) in [
        (0, "0.7112119049133976"),
        (1, "0.42242380982679517"),
        (2, "0.22989841310239353"),
        (3, "0.11988390068844976"),
    ] {
        f.pin(
            &format!("B-F5.fail_gf2_h{h}"),
            py(p_fail_limit(2, h)),
            want,
            "fact sheet B-F5",
        )?;
    }
    f.pin(
        "B-F5.fail_gf2_h0_K10",
        py(p_fail(2, 10, 0)),
        "0.710929701580251",
        "fact sheet B-F5",
    )?;
    f.add(
        "B-F5.danger_meter_K10",
        list((0..10).map(|held| py(useless_chance(held, 10)))),
    )?;
    f.pin(
        "B-F28.min_spares_for_fail_le_1e-6",
        spares_for(1_000_000),
        "20",
        "fact sheet B-F28",
    )?;
    f.pin(
        "B-F28.fail_gf2_h20",
        py(p_fail_limit(2, 20)),
        "9.536740132413907e-07",
        "fact sheet B-F28",
    )?;
    f.pin(
        "B-F28.phones_stuck_of_1e6_at_h2",
        py(p_fail_limit_times(2, 2, 1_000_000)),
        "229898.41310239353",
        "fact sheet B-F28",
    )?;
    f.pin("B-X4.halvings_2^10", 1u32 << 10, "1024", "fact sheet B-X4")?;
    f.pin("B-X4.halvings_2^20", 1u32 << 20, "1048576", "fact sheet B-X4")?;
    // 06: the ring of five, and the byte's 256 amounts
    f.pin("B-F9.rank_gf2", rank_gf2(&RING), "4", "fact sheet B-F9")?;
    f.pin("B-X4.byte_values_2^8", 1u32 << 8, "256", "2⁸")?;
    // 07: wrap-around at 256
    f.pin("B-F10.z256_2x128", 2u16 * 128 % 256, "0", "fact sheet B-F10")?;
    f.pin("B-F10.z256_2x131", 2u16 * 131 % 256, "6", "fact sheet B-F10")?;
    // 09–10: the byte field (RaptorQ's fold) and AES's
    for (key, x, y, want) in [
        ("B-F11.mul_3_7", 3, 7, "9"),
        ("B-F11.mul_2_128", 2, 128, "29"),
        ("B-F11.mul_128_4", 128, 4, "58"),
        ("B-F11.mul_150_3", 150, 3, "167"),
        ("B-F11.mul_7_7", 7, 7, "21"),
        ("B-F11.mul_2_142", 2, 142, "1"),
        ("B-F11.mul_3_244", 3, 244, "1"),
        ("B-F11.mul_3_5", 3, 5, "15"),
    ] {
        f.pin(key, mul(x, y, FOLD), want, "fact sheet B-F11")?;
    }
    f.pin(
        "B-F11.mul_0x57_0x83_0x11D",
        mul(0x57, 0x83, FOLD),
        "49",
        "fact sheet B-F11 (0x31)",
    )?;
    f.pin(
        "B-F11.mul_0x57_0x83_0x11B_AES",
        mul(0x57, 0x83, AES_FOLD),
        "193",
        "fact sheet B-F11 (0xC1)",
    )?;
    // 10_7: the panel's comment flips with the fold, AES's partner of 2 is 141
    f.pin(
        "B-F11.mul_2_141_0x11B_AES",
        mul(2, 141, AES_FOLD),
        "1",
        "fact sheet B-F11 (claim walk)",
    )?;
    // 11: the byte clock
    let log = logs();
    for (key, x, want) in [
        ("B-F13.log_3", 3usize, "25"),
        ("B-F13.log_29", 29, "8"),
        ("B-F13.log_128", 128, "7"),
        ("B-F13.log_150", 150, "180"),
    ] {
        f.pin(key, log[x], want, "fact sheet B-F13")?;
    }
    f.pin(
        "B-F13.log150_plus_log3",
        u16::from(log[150]) + u16::from(log[3]),
        "205",
        "fact sheet B-F13",
    )?;
    f.pin("B-F12.ord2_under_0x11D", order(2, FOLD), "255", "fact sheet B-F12")?;
    f.pin("B-F12.ord2_under_0x11B", order(2, AES_FOLD), "51", "fact sheet B-F12")?;
    f.pin("B-F12.ord3_under_0x11B", order(3, AES_FOLD), "255", "fact sheet B-F12")?;
    // 06, 16: the ring of five and its byte card
    let e = PIECES[4];
    let ring = [a ^ b, b ^ c, c ^ d, d ^ e, e ^ mul(2, a, FOLD)];
    f.pin("B-F9.card_values", list(ring), "6,4,5,14,6", "fact sheet B-F9")?;
    let s = ring.iter().fold(0, |x, y| x ^ y);
    f.pin("B-F9.y5_xor_S_equals_3A", s, "15", "fact sheet B-F9")?;
    f.pin(
        "B-F9.A_solved_15_div_3",
        mul(s, 244, FOLD),
        "5",
        "fact sheet B-F9 (3⁻¹ = 244)",
    )?;
    density(&mut f)?;
    arithmetic(&mut f)?;
    byte_lottery(&mut f)?;
    danger(&mut f)?;
    parking(&mut f)?;
    two_packets(&mut f)?;
    Ok(f)
}

/// 01: a million phones, the carousel and the sticker album.
fn hook(f: &mut Facts) -> Result<(), String> {
    f.pin("B-F1.K", HOOK_PIECES, "10000", "fact sheet B-F1")?;
    f.pin(
        "B-F1.K_times_H_K",
        py(album_mean(HOOK_PIECES)),
        "97876.06036044382",
        "fact sheet B-F1",
    )?;
    f.pin(
        "B-F1.K_times_H_K_floor",
        album_mean_floor(HOOK_PIECES),
        "97876",
        "fact sheet B-F1",
    )?;
    f.add("B-F2.phones", PHONES)?;
    f.add("B-F2.loss_one_in", LOSS_ONE_IN)?;
    f.pin(
        "B-F2.expected_misses_per_packet",
        PHONES / u64::from(LOSS_ONE_IN),
        "10000",
        "fact sheet B-F2",
    )?;
    for (m, want) in [(2, "0.36786104643292994"), (3, "0.9900498287989156")] {
        f.pin(
            &format!("B-F24.P_phone_done_by_{m}_loops"),
            py(p_done_by(m, 1)),
            want,
            "fact sheet B-F24",
        )?;
    }
    for (m, want) in [(5, "0.3678794411530483"), (6, "0.9900498337491631")] {
        f.pin(
            &format!("B-F24.P_all_1e6_phones_done_by_{m}_loops"),
            py(p_done_by(m, PHONES)),
            want,
            "fact sheet B-F24",
        )?;
    }
    f.pin(
        "B-F24.mean_loops_per_phone",
        py(mean_loops()),
        "2.642190129869331",
        "fact sheet B-F24",
    )?;
    f.add("B-F24.P_phone_done_in_loop_2_or_3", py(p_done_in_loop_2_or_3()))?;
    f.add("B-F24.P_phone_still_waiting_after_3_loops", py(p_waiting_after(3)))?;
    f.pin(
        "B-F24.carousel_mean_packets_sent_until_done",
        py(carousel_mean_sent()),
        "23729.95434759208",
        "fact sheet B-F24 (23,729.95)",
    )?;
    f.pin(
        "B-F24.carousel_mean_packets_caught_until_done",
        py(carousel_mean_caught()),
        "23492.65480411616",
        "fact sheet B-F24 (23,492.65)",
    )?;
    f.pin(
        "B-F24.fountain_packets_caught_K_plus_2",
        HOOK_PIECES + 2,
        "10002",
        "fact sheet B-F24",
    )?;
    // the drawn field: a seeded sample of 1,000 phones, and the unluckiest of a million
    f.add("B-F24.field.seed", FIELD_SEED)?;
    f.add("B-F24.field.glyphs", GLYPHS)?;
    f.add("B-F24.field.phones_per_glyph", PHONES / GLYPHS as u64)?;
    f.add("B-F24.field.glyphs_done_by_loop_1_to_6", list(field_done_by_loop()))?;
    f.add("B-F24.field.unluckiest_of_1e6_loops", unluckiest_of_a_million())?;
    Ok(())
}

/// A key more than one chapter registers (the same value from the same
/// code): added the first time; a later registration must agree.
fn once(f: &mut Facts, key: &str, value: impl ToString) -> Result<(), String> {
    let value = value.to_string();
    match f.get(key) {
        Some(v) if v == value => Ok(()),
        Some(v) => Err(format!("{key}: {value} here, {v} where first registered")),
        None => f.add(key, value).map(|_| ()),
    }
}

/// 13: the byte card, the last slot at one in 256, the ladder's ÷ 256, the
/// seeded race on ten pieces (`decoding::race`, the oracle's draws), and the
/// 0/1 ring in either field.
fn byte_lottery(f: &mut Facts) -> Result<(), String> {
    let amounts = random_byte_card();
    f.add("B-CH13.byte_card.seed", CARD_SEED)?;
    f.add("B-CH13.byte_card.amounts_A_to_E", list(amounts))?;
    let value = amounts.iter().zip(PIECES).fold(0, |acc, (&a, p)| acc ^ mul(a, p, FOLD));
    f.add("B-CH13.byte_card.result", value)?;
    f.pin(
        "B-F7.success_K1",
        py(p_full(256, 1, 0)),
        "0.99609375",
        "fact sheet B-F7 (255/256)",
    )?;
    let (lines, ok) = one_amount_lights_exhaustive();
    f.add("B-F7.last_slot_lit_lines_checked_K2", lines)?;
    f.add("B-F7.last_slot_exactly_one_amount_lit_exhaustive_K2_ok", u8::from(ok))?;
    for h in 0..3 {
        f.add(&format!("B-F8.fail_gf256_h{h}"), py(p_fail_limit(256, h)))?;
    }
    f.add("B-F27.h0.union_bound", py(1.0 / 255.0))?;
    let race = race();
    f.pin("B-F6.race_K10_seeded.seed", RACE_SEED, "1306", "fact sheet B-F6")?;
    f.add("B-F6.race_K10_seeded.races_per_side", RACES)?;
    f.pin(
        "B-F6.race_K10_seeded.gf2.spares_histogram_0_up",
        list(histogram(&race.ex_or)),
        "297,303,171,94,65,39,17,4,7,0,2,0,0,0,0,1",
        "fact sheet B-F6 (race)",
    )?;
    f.add("B-F6.race_K10_seeded.gf2.mean_spares", py(mean_spares(&race.ex_or)))?;
    f.pin(
        "B-F6.race_K10_seeded.gf256.spares_histogram_0_up",
        list(histogram(&race.byte)),
        "995,5",
        "fact sheet B-F6 (race)",
    )?;
    f.add("B-F6.race_K10_seeded.gf256.mean_spares", py(mean_spares(&race.byte)))?;
    f.add(
        "B-F6.race_K10_seeded.means_within_4se_of_exact_ok",
        u8::from(race_means_within_4se(&race)),
    )?;
    let k = RACE_PIECES as u32;
    f.add("B-F6.mean_spares_gf2_K10", py(exact_mean_spares(1, k)))?;
    f.add("B-F6.mean_spares_gf256_K10", py(exact_mean_spares(8, k)))?;
    f.add("B-F6.mean_spares_gf2", py(exact_mean_spares(1, 100)))?;
    f.add("B-F6.last_term_denominator_K10", (1u32 << k) - 1)?;
    f.pin(
        "B-F6.sourced.erdos_irrationality_year",
        ERDOS_YEAR,
        "1948",
        "Erdős, J. Indian Math. Soc. 12 (1948)",
    )?;
    let rows: Vec<Vec<u8>> = RING
        .iter()
        .map(|&c| (0..5).map(|p| u8::from(c >> p & 1 == 1)).collect())
        .collect();
    f.add("B-F9.rank_gf256_same_01_matrix", rank_gf256(&rows))?;
    f.add("B-F9.amounts_other_than_0_and_1", (1u32 << 8) - 2)?;
    Ok(())
}

/// 15: RaptorQ's shape (Table 2's rows for ten pieces, the big block and the
/// largest; H's range), a packet's pieces, and the recorded ablation.
fn danger(f: &mut Facts) -> Result<(), String> {
    f.pin(
        "B-F16.mean_symbols_per_repair",
        py(pieces_per_packet()),
        "7.152475357055664",
        "fact sheet B-F16 (7.152)",
    )?;
    for (k, b) in [(10, TEN), (10_017, BIG)] {
        for (key, v) in [("S", b.s), ("H", b.h), ("W", b.w), ("L", b.l()), ("P", b.p())] {
            once(f, &format!("B-F19.K{k}.{key}"), v)?;
        }
        f.add(&format!("B-X7.K{k}.extra_pieces_S_plus_H"), b.extra())?;
        f.add(&format!("B-X7.K{k}.ex_or_cards_L_minus_H"), b.ex_or_slots())?;
    }
    f.pin("B-F19.H_min", H_RANGE.0, "10", "RFC 6330 Table 2 (fact sheet B-F19)")?;
    f.pin("B-F19.H_max", H_RANGE.1, "16", "RFC 6330 Table 2 (fact sheet B-F19)")?;
    once(f, "B-F19.Kprime_max", LARGEST.k)?;
    for (key, (fails, trials)) in [("K10_gf2_hdpc_h0", ABLATION_EX_OR), ("K10_rfc_hdpc_h0", ABLATION_BYTES)] {
        f.add(&format!("B-F21.recorded.{key}.fails"), fails)?;
        f.add(&format!("B-F21.recorded.{key}.trials"), trials)?;
        f.add(
            &format!("B-F21.recorded.{key}.rate"),
            py(f64::from(fails) / f64::from(trials)),
        )?;
    }
    Ok(())
}

/// 17: the measured decoder (RECORDED, fact sheet B-F22) with its exact
/// intervals and the ÷ 256-twice prediction, the standard's promise
/// (SOURCED, B-F23), and the years on the cards and credits.
fn two_packets(f: &mut Facts) -> Result<(), String> {
    for (h, (&(fails, trials), want)) in MEASURED_K10.iter().zip(["6316", "24", "1"]).enumerate() {
        f.pin(
            &format!("B-F22.recorded.K10.h{h}.failures"),
            fails,
            want,
            "fact sheet B-F22 (original_rq_mc_K10_1e6trials.txt)",
        )?;
        f.add(&format!("B-F22.recorded.K10.h{h}.trials"), trials)?;
        f.add(
            &format!("B-F22.recorded.K10.h{h}.rate"),
            py(f64::from(fails) / f64::from(trials)),
        )?;
    }
    for (k, fails, trials) in MEASURED_LARGER {
        f.add(&format!("B-F22.recorded.K{k}.h0.failures"), fails)?;
        f.add(&format!("B-F22.recorded.K{k}.h0.trials"), trials)?;
        f.add(
            &format!("B-F22.recorded.K{k}.h0.rate"),
            py(f64::from(fails) / f64::from(trials)),
        )?;
    }
    let predicted = f64::from(MEASURED_K10[0].0) / 65_536.0;
    f.add("B-F22.K10.h2_failures_predicted_by_div256_twice", py(predicted))?;
    f.add(
        "B-F22.K10.h2_P_at_least_one_if_poisson_mean_predicted",
        py(-(-predicted).exp_m1()),
    )?;
    // what the chapter prints from them (the exact intervals agree with the
    // oracle's to about nine digits, not bit for bit, so the printed
    // roundings are what is checked)
    let shown = crate::chapters::two_packets::shown();
    f.add("B-CH17.shown.K10.h0.cp95_percent", shown.h0_interval_percent.join("–"))?;
    f.add(
        "B-CH17.shown.K10.h2.cp95_per_million",
        shown.h2_interval_per_million.join("–"),
    )?;
    for ((k, _, _), percent) in MEASURED_LARGER.iter().zip(shown.larger_percent) {
        f.add(&format!("B-CH17.shown.K{k}.h0.percent"), percent)?;
    }
    f.add("B-CH17.shown.K10.h2_predicted", shown.h2_predicted)?;
    f.add(
        "B-CH17.shown.K10.h2_P_at_least_one_percent",
        shown.h2_at_least_one_percent,
    )?;
    for (h, (&one_in, want)) in RFC_ONE_IN.iter().zip(["0.01", "0.0001", "1e-06"]).enumerate() {
        f.pin(
            &format!("B-F23.sourced.rfc6330_s5.8_max_fail_Kprime_plus_{h}"),
            py(1.0 / f64::from(one_in)),
            want,
            "RFC 6330 §5.8 (fact sheet B-F23)",
        )?;
    }
    once(f, "B-S1.sourced.rfc6330_year", RFC_6330_YEAR)?;
    f.add("B-CH17.sourced.lt_codes_year", LT_CODES_YEAR)?;
    f.add("B-CH17.sourced.mbms_raptor_year", MBMS_RAPTOR_YEAR)?;
    f.add("B-CH17.sourced.atsc_a331_year", ATSC_A331_YEAR)?;
    Ok(())
}

/// 16: the parked piece (the toy's peel with C parked, `decoding::peel_toy`),
/// the big block's parameters, and the ring with a byte card.
fn parking(f: &mut Facts) -> Result<(), String> {
    let cards: Vec<usize> = (1..TOY_CARDS.len()).collect();
    f.add(
        "B-F20.y_values_y2_to_y6",
        list(cards.iter().map(|&j| result(TOY_CARDS[j]))),
    )?;
    let parked = 2;
    let (steps, leftover) = peel_toy(&cards, Some(parked));
    // the leftover card B ⊕ E with D ⊕ E and B ⊕ C ⊕ D: all but C cancel
    let c = [5usize, 4, 2].iter().fold(0, |acc, &j| acc ^ result(TOY_CARDS[j]));
    let cancel = [5usize, 4, 2].iter().fold(0, |acc, &j| acc ^ TOY_CARDS[j]);
    if cancel != 1 << parked || leftover != [5] {
        return Err(format!("16_3: the cards ex-or to {cancel:#b}, leftover {leftover:?}"));
    }
    f.pin("B-F20.c_equals_y6_xor_y5_xor_y3", c, "7", "fact sheet B-F20")?;
    // every c consistent with the leftover card, B ⊕ E = 15, given the peel
    let value = |p: usize, cc: u8| {
        let s = steps.iter().find(|s| s.piece == p).expect("peeled");
        s.value.constant ^ if s.value.with_c { cc } else { 0 }
    };
    let fits: Vec<u8> = (0..=255u8)
        .filter(|&cc| value(1, cc) ^ value(4, cc) == result(TOY_CARDS[5]))
        .collect();
    f.add("B-F20.unique_c_consistent_with_y6", list(&fits))?;
    f.add(
        "B-F20.back_substituted_A_B_D_E",
        list([0, 1, 3, 4].map(|p| value(p, c))),
    )?;
    for (key, v) in [("S", BIG.s), ("H", BIG.h), ("W", BIG.w), ("L", BIG.l()), ("P", BIG.p())] {
        once(f, &format!("B-F19.K10017.{key}"), v)?;
    }
    once(f, "B-F9.one_xor_two", 1 ^ 2)?;
    once(f, "B-F9.three_times_five", mul(3, 5, FOLD))?;
    once(f, "B-F11.mul_3_7", mul(3, 7, FOLD))?;
    Ok(())
}

/// 14: dense solving, RFC 6330's degrees, the toy's peel, the scene's own
/// seeded peeling runs and the pieces no card mixes (`decoding.rs`; the
/// chapter draws these values).
fn density(f: &mut Facts) -> Result<(), String> {
    let k = DENSE_PIECES;
    f.add("B-F15.K_cubed", k * k * k)?;
    f.add("B-F15.K_cubed_over_3", py(dense_steps(k)))?;
    f.add("B-F15.K_cubed_over_3_in_trillions", py((k * k * k) as f64 / 3e12))?;
    f.add(
        "B-F15.hours_at_1e9_steps_per_s",
        py(dense_time(k, STEPS_PER_SECOND, 3600)),
    )?;
    f.add(
        "B-F15.days_at_1e9_steps_per_s",
        py(dense_time(k, STEPS_PER_SECOND, 86_400)),
    )?;
    f.add("B-F15.matrix_bytes_K50000", k * k)?;
    f.add(
        "B-F15.hours_at_1e10_steps_per_s",
        py(dense_time(k, STEPS_PER_SECOND * 10, 3600)),
    )?;
    f.add("B-F15.work_ratio_when_pieces_double", (2 * k).pow(3) / k.pow(3))?;
    f.pin("B-F16.P_deg2", py(degree_mass(2)), "0.5", "RFC 6330 Table 1")?;
    f.pin(
        "B-F16.mean_degree",
        py(mean_degree()),
        "4.816641807556152",
        "fact sheet B-F16",
    )?;
    // the toy's peel (14_3), and without its first card (14_5)
    let all: Vec<usize> = (0..TOY_CARDS.len()).collect();
    let (steps, _) = peel_toy(&all, None);
    f.pin(
        "B-F20.peel_order",
        list(steps.iter().map(|s| PIECE_NAMES[s.piece])),
        "C,A,B,D,E",
        "fact sheet B-F20",
    )?;
    f.add("B-F20.ripple_sizes", list(steps.iter().map(|s| s.ripple)))?;
    let all_right = steps.len() == PIECES.len()
        && steps
            .iter()
            .all(|s| !s.value.with_c && s.value.constant == PIECES[s.piece]);
    f.add("B-F20.peel_recovers_all_correctly_ok", u8::from(all_right))?;
    let (stuck, _) = peel_toy(&all[1..], None);
    f.pin("B-F20.recovered_without_y1", stuck.len(), "0", "fact sheet B-F20")?;
    // the seeded runs (14_6): the oracle's lt_peel_sim, replayed
    let runs = peel_runs();
    for (g, &pct) in PEEL_EXTRA_PERCENT.iter().enumerate() {
        let tag = format!("B-F18.eps{pct:03}");
        let share = |r: &PeelRun| r.recovered() as f64 / PEEL_PIECES as f64;
        f.add(
            &format!("{tag}.recovered_fractions_5runs"),
            list(runs[g].iter().map(|r| py(share(r)))),
        )?;
        let ok: Vec<&PeelRun> = runs[g].iter().filter(|r| 2 * r.recovered() >= PEEL_PIECES).collect();
        f.add(&format!("{tag}.n_collapsed_below_50pct"), runs[g].len() - ok.len())?;
        let shares: Vec<f64> = runs[g].iter().map(share).collect();
        f.add(
            &format!("{tag}.min_recovered"),
            py(shares.iter().copied().fold(f64::INFINITY, f64::min)),
        )?;
        f.add(
            &format!("{tag}.max_recovered"),
            py(shares.iter().copied().fold(0.0, f64::max)),
        )?;
        if !ok.is_empty() && ok.len() < runs[g].len() {
            let low = ok.iter().map(|r| share(r)).fold(f64::INFINITY, f64::min);
            f.add(&format!("{tag}.min_recovered_non_collapsed"), py(low))?;
        }
        if pct != 0 {
            let missing: Vec<usize> = ok.iter().map(|r| PEEL_PIECES - r.recovered()).collect();
            f.add(
                &format!("{tag}.missing_pieces_non_collapsed_min"),
                missing.iter().min().copied().unwrap_or(0),
            )?;
            f.add(
                &format!("{tag}.missing_pieces_non_collapsed_max"),
                missing.iter().max().copied().unwrap_or(0),
            )?;
        }
    }
    f.add("B-F17.expected_untouched_K10000_N10200", py(expected_untouched()))?;
    Ok(())
}

/// 07–11 (lane B-2): wrap-around and the race, the prime clocks, bytes as
/// polynomials, the fold, the byte clock. Every key here is a number those
/// chapters draw or say beyond the ones pinned above.
fn arithmetic(f: &mut Facts) -> Result<(), String> {
    use crate::model::clocks::{bits_needed, mul_mod, partner, primes_around};
    use crate::model::poly::{exponents, foil, survivors};
    use crate::model::wrap::{self, RACE_PIECES, RACE_ROUNDS, RACE_SEED, Race};
    use crate::model::{exps, inv};
    // 07: 3 and 131 both double to 6; 3's partner wrapped at 256
    f.pin("B-F10.z256_2x3", wrap::mul(2, 3), "6", "fact sheet B-F10")?;
    f.pin("B-F10.int_2x131", 2u32 * 131, "262", "fact sheet B-F10 (claim walk)")?;
    let inv3 = wrap::inverse_odd(3);
    f.pin("B-F10.z256_inverse_of_3", inv3, "171", "fact sheet B-F10 (claim walk)")?;
    let big = 3u32 * u32::from(inv3);
    f.pin("B-F10.int_3x171", big, "513", "fact sheet B-F10 (claim walk)")?;
    f.pin(
        "B-F10.int_3x171_divmod_256",
        list([big / 256, big % 256]),
        "2,1",
        "fact sheet B-F10",
    )?;
    // 07_6: the race, seeded (oracle: numpy's PCG64DXSM(706))
    let race = Race::draw();
    let (xf, wf) = (race.xor_fails(), race.wrap_fails());
    f.add("B-F25.race07.seed", RACE_SEED)?;
    f.add("B-F25.race07.pieces", RACE_PIECES)?;
    f.add("B-F25.race07.rounds_per_side", RACE_ROUNDS)?;
    f.add("B-F25.race07.xor_failures", xf.iter().filter(|&&x| x).count())?;
    f.add("B-F25.race07.wrap_failures", wf.iter().filter(|&&x| x).count())?;
    f.add(
        "B-F25.race07.first_rounds_xor_fail",
        list(xf[..4].iter().map(|&x| u8::from(x))),
    )?;
    f.add(
        "B-F25.race07.first_rounds_wrap_fail",
        list(wf[..4].iter().map(|&x| u8::from(x))),
    )?;
    f.add("B-F25.race07.wrap_round1_amounts", list(race.wrap[0].iter().flatten()))?;
    // 08: the seven-hour clock and why 256 is not prime
    f.pin(
        "B-X4.z7_inverse_of_3",
        partner(3, 7).unwrap_or(0),
        "5",
        "fact sheet B-X4",
    )?;
    f.add("B-X4.z7_times_table", list((0..49).map(|i| mul_mod(i / 7, i % 7, 7))))?;
    f.pin("B-X4.int_3x5", 3u32 * 5, "15", "fact sheet B-X4")?;
    f.add("B-X4.int_3x5_divmod_7", list([15 / 7, 15 % 7]))?;
    f.pin(
        "B-X4.z7_nonzero_rows_are_permutations_ok",
        u8::from((1..7).all(|a| {
            (0..7)
                .map(|b| mul_mod(a, b, 7))
                .collect::<std::collections::HashSet<_>>()
                .len()
                == 7
        })),
        "1",
        "fact sheet B-X4",
    )?;
    f.pin(
        "B-X4.z7_zero_has_no_partner_ok",
        u8::from(partner(0, 7).is_none()),
        "1",
        "fact sheet B-X4",
    )?;
    f.pin("B-X4.int_2x128", 2u32 * 128, "256", "fact sheet B-X4")?;
    let (below, above) = primes_around(256);
    f.pin(
        "B-X4.nearest_primes_below_and_above_256",
        list([below, above]),
        "251,257",
        "fact sheet B-X4",
    )?;
    f.pin("B-X4.bits_needed_for_256", bits_needed(256), "9", "fact sheet B-X4")?;
    // 09: bytes as polynomials; 3 × 7 by FOIL
    f.pin("B-X4.x_powers_of_3", list(exponents(3)), "1,0", "fact sheet B-X4")?;
    f.pin("B-X4.x_powers_of_7", list(exponents(7)), "2,1,0", "fact sheet B-X4")?;
    f.pin("B-X4.x_powers_of_9", list(exponents(9)), "3,0", "fact sheet B-X4")?;
    f.add("B-X4.bits_of_9", format!("{:04b}", survivors(&foil(3, 7))))?;
    f.add("B-F11.foil_3x7_terms", list(foil(3, 7)))?;
    f.pin(
        "B-F11.foil_3x7_carryless",
        survivors(&foil(3, 7)),
        "9",
        "fact sheet B-F11",
    )?;
    // 10: the rotation fold (x⁸ → 1) and RaptorQ's
    const ROTATE: u8 = 0x01;
    f.pin(
        "B-F26.mul2_mod_x8_plus_1_of_128",
        mul(2, 128, ROTATE),
        "1",
        "fact sheet B-F26",
    )?;
    f.pin(
        "B-F26.mul2_mod_x8_plus_1_of_255",
        mul(2, 255, ROTATE),
        "255",
        "fact sheet B-F26",
    )?;
    // 10_3: 2 · 255 rotates to 255, ex-or 255 itself
    f.pin("B-F26.xor_255_255", mul(2, 255, ROTATE) ^ 255, "0", "fact sheet B-F26")?;
    f.pin(
        "B-F26.mul_3_255_mod_x8_plus_1",
        mul(3, 255, ROTATE),
        "0",
        "fact sheet B-F26",
    )?;
    let clmul = |a: u8, b: u8| foil(a, b).iter().fold(0u16, |acc, &e| acc ^ (1 << e));
    f.pin(
        "B-F26.carryless_3_255",
        clmul(3, 255),
        "257",
        "fact sheet B-F26 (x⁸ + 1)",
    )?;
    f.pin("B-F11.fold_x8_residue", FOLD, "29", "fact sheet B-F11")?;
    f.add("B-F11.fold_polynomial_0x11D_decimal", u16::from(FOLD) | 0x100)?;
    f.pin(
        "B-X5.sourced.crc8_sae_j1850_poly",
        FOLD,
        "29",
        "fact sheet B-X5 (reveng catalogue)",
    )?;
    f.pin(
        "B-F11.clmul_2_128_is_x8",
        clmul(2, 128),
        "256",
        "fact sheet B-F11 (claim walk)",
    )?;
    f.pin("B-F11.inv_2", inv(2), "142", "fact sheet B-F11")?;
    f.pin(
        "B-F11.inv_2_under_0x11B",
        (1..=255u8).find(|&b| mul(2, b, AES_FOLD) == 1).unwrap_or(0),
        "141",
        "fact sheet B-F11 (claim walk)",
    )?;
    // 11: the byte clock, the ruler, the slide rule, the lookup tables
    let log = logs();
    let exp = exps();
    f.pin(
        "B-F14.oct_exp_0_to_12_computed",
        list(&exp[..13]),
        "1,2,4,8,16,32,64,128,29,58,116,232,205",
        "fact sheet B-F14",
    )?;
    f.pin("B-F13.log_4", log[4], "2", "fact sheet B-F13")?;
    f.pin("B-F13.log_58", log[58], "9", "fact sheet B-F13")?;
    f.pin("B-F13.exp_7_plus_2", exp[7 + 2], "58", "fact sheet B-F13")?;
    f.pin("B-F13.exp_205", exp[205], "167", "fact sheet B-F13")?;
    // 11_6 (R3, Dev D4): 150 × 150 goes past the top of the clock
    let l150 = u16::from(log[150]);
    f.pin("B-F13.log150_plus_log150", l150 + l150, "360", "fact sheet B-F13 (R3)")?;
    f.pin(
        "B-F13.log150_plus_log150_mod_255",
        (l150 + l150) % 255,
        "105",
        "fact sheet B-F13 (R3)",
    )?;
    f.pin("B-F13.exp_105", exp[105], "26", "fact sheet B-F13 (R3)")?;
    f.pin("B-F13.mul_150_150", mul(150, 150, FOLD), "26", "fact sheet B-F13 (R3)")?;
    f.pin(
        "B-F13.int_2^3_x_2^4",
        (1u32 << 3) * (1u32 << 4),
        "128",
        "fact sheet B-F13 (claim walk)",
    )?;
    let l7 = u16::from(log[7]);
    f.pin("B-F13.log_7", l7, "198", "fact sheet B-F13 (claim walk)")?;
    f.pin("B-F13.log7_plus_log7", l7 + l7, "396", "fact sheet B-F13 (claim walk)")?;
    f.pin(
        "B-F13.log7_plus_log7_mod_255",
        (l7 + l7) % 255,
        "141",
        "fact sheet B-F13 (claim walk)",
    )?;
    f.pin("B-F13.exp_141", exp[141], "21", "fact sheet B-F13 (claim walk)")?;
    f.pin(
        "B-F11.clmul_7_7_no_fold",
        clmul(7, 7),
        "21",
        "fact sheet B-F11 (claim walk)",
    )?;
    // 11_7's rule, log a⁻¹ ≡ −log a (mod 255), on every nonzero byte, 1 included
    f.pin(
        "B-F13.log_inverse_is_255_minus_log_all255_ok",
        u8::from((1..=255u8).all(|a| u16::from(log[inv(a) as usize]) == (255 - u16::from(log[a as usize])) % 255)),
        "1",
        "fact sheet B-F13 (claim walk)",
    )?;
    let mirror = (1..=255u8).filter(|&a| inv(a) != a).count() / 2;
    f.pin(
        "B-F13.mirror_pairs_a_ne_inverse",
        mirror,
        "127",
        "fact sheet B-F13 (claim walk)",
    )?;
    f.pin(
        "B-F13.bytes_that_are_their_own_inverse",
        list((1..=255u8).filter(|&a| inv(a) == a)),
        "1",
        "fact sheet B-F13 (claim walk)",
    )?;
    let (wraps, pairs) = wrap::u8_log_sum_wraps(&log);
    f.pin("B-F13.nonzero_pairs", pairs, "65025", "fact sheet B-F13 (R2)")?;
    f.pin(
        "B-F13.nonzero_pairs_log_sum_ge_256_u8_overflow",
        wraps,
        "32131",
        "fact sheet B-F13 (R2)",
    )?;
    let max_sum = (1..256)
        .flat_map(|a| (1..256).map(move |b| (a, b)))
        .map(|(a, b)| u16::from(log[a]) + u16::from(log[b]))
        .max();
    f.pin(
        "B-F13.max_log_sum_nonzero_pairs",
        max_sum.unwrap_or(0),
        "508",
        "fact sheet B-F13 (R2)",
    )?;
    f.pin(
        "B-X1.raid6_distinct_weights_2^i_i0_254_under_0x11B",
        (0..255)
            .map(|i| (0..i).fold(1u8, |x, _| mul(x, 2, AES_FOLD)))
            .collect::<std::collections::HashSet<_>>()
            .len(),
        "51",
        "fact sheet B-X1",
    )?;
    // 12_4's lower-third: the element orders and how many bytes have each
    let mut orders: Vec<(u32, usize)> = Vec::new();
    for x in 1..=255u8 {
        let o = order(x, FOLD);
        match orders.iter_mut().find(|(k, _)| *k == o) {
            Some((_, c)) => *c += 1,
            None => orders.push((o, 1)),
        }
    }
    orders.sort_unstable();
    f.pin(
        "B-F12.element_orders_0x11D",
        list(orders.iter().map(|&(o, _)| o)),
        "1,3,5,15,17,51,85,255",
        "fact sheet B-F12",
    )?;
    f.pin(
        "B-F12.element_order_counts_0x11D",
        list(orders.iter().map(|&(_, c)| c)),
        "1,2,4,8,16,32,64,128",
        "fact sheet B-F12",
    )?;
    f.pin(
        "B-F12.product_(2+1)(2^2+1)(2^4+1)",
        (2 + 1) * (4 + 1) * (16 + 1),
        "255",
        "fact sheet B-F12",
    )?;
    f.add(
        "B-F12.smallest_generator_0x11B",
        (2..=255u8).find(|&g| order(g, AES_FOLD) == 255).unwrap_or(0),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn py_prints_doubles_as_python_repr_does() {
        // repr() in CPython 3.14, for each case
        for (x, want) in [
            (0.0, "0.0"),
            (0.5, "0.5"),
            (2.0, "2.0"),
            (97_876.060_360_443_82, "97876.06036044382"),
            (0.0001, "0.0001"),
            (0.000_015_318_626_537_905_112, "1.5318626537905112e-05"),
            (1e-5, "1e-05"),
            (9.536_740_132_413_907e-7, "9.536740132413907e-07"),
            (2.248_774_849_816_482e-44, "2.248774849816482e-44"),
            (1e15, "1000000000000000.0"),
            (1e16, "1e+16"),
            (1.5e300, "1.5e+300"),
            (-0.25, "-0.25"),
            (0.0009765625, "0.0009765625"),
        ] {
            assert_eq!(py(x), want, "{x:e}");
        }
    }

    #[test]
    fn every_registered_number_passes_its_pin() {
        let f = facts().unwrap();
        f.self_check().unwrap();
        assert_eq!(f.get("B-F5.danger_meter_K10").unwrap().split(',').count(), 10);
    }
}
