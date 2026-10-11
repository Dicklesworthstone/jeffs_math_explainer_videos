//! Every number Video B shows or says, computed here and checked two ways:
//! `pin` against a literal from the fact sheet, and `./x raptorq_gf256
//! facts-check` against `oracle.py` (same keys). Chapters read their numbers
//! from here; a typed number is a defect (doctrine 2).

use explainer_kit::Facts;

/// RaptorQ's fold polynomial x⁸ + x⁴ + x³ + x² + 1 (0x11D), low byte.
pub const FOLD: u8 = 0x1D;
/// AES's polynomial x⁸ + x⁴ + x³ + x + 1 (0x11B), low byte.
pub const AES_FOLD: u8 = 0x1B;

/// Multiply by x (the byte 2): shift, and fold the bit that falls off.
pub fn xtime(a: u8, fold: u8) -> u8 {
    (a << 1) ^ if a & 0x80 != 0 { fold } else { 0 }
}

/// Byte multiplication, shift-and-add (the video's code panel).
pub fn mul(mut a: u8, mut b: u8, fold: u8) -> u8 {
    let mut acc = 0;
    while b != 0 {
        if b & 1 != 0 {
            acc ^= a;
        }
        a = xtime(a, fold);
        b >>= 1;
    }
    acc
}

/// Discrete logs base 2 under RaptorQ's fold: `log[a]` for every nonzero byte.
pub fn logs() -> [u8; 256] {
    let mut log = [0u8; 256];
    let mut a = 1u8;
    for step in 0..255u16 {
        log[a as usize] = step as u8;
        a = xtime(a, FOLD);
    }
    log
}

/// The multiplicative order of `a` (the smallest k ≥ 1 with aᵏ = 1).
pub fn order(a: u8, fold: u8) -> u32 {
    let mut x = a;
    let mut k = 1;
    while x != 1 {
        x = mul(x, a, fold);
        k += 1;
        assert!(k <= 255, "{a} is zero or not invertible");
    }
    k
}

pub fn facts() -> Result<Facts, String> {
    let mut f = Facts::new();
    // 03: the cards, with A, B, C, D = 5, 3, 7, 2
    let (a, b, c, d) = (5u8, 3u8, 7u8, 2u8);
    f.pin("B-F3.A_xor_C", a ^ c, "2", "fact sheet B-F3")?;
    f.pin("B-F3.B_xor_C_xor_D", b ^ c ^ d, "6", "fact sheet B-F3")?;
    f.pin("B-F3.A_xor_B", a ^ b, "6", "fact sheet B-F3")?;
    f.pin("B-F3.A_xor_B_xor_C_xor_D", a ^ b ^ c ^ d, "3", "fact sheet B-F3")?;
    f.pin("B-F3.B_xor_C", b ^ c, "4", "fact sheet B-F3")?;
    f.pin("B-F3.card1_xor_card3", (a ^ c) ^ (a ^ b), "4", "fact sheet B-F3")?;
    f.pin(
        "B-F3.D_from_card2_and_BxC",
        (b ^ c ^ d) ^ (b ^ c),
        "2",
        "fact sheet B-F3",
    )?;
    // 04–05: the lottery at three pieces, exact in binary fractions
    f.pin(
        "B-F4.success_K3",
        7.0 / 8.0 * (6.0 / 8.0) * (4.0 / 8.0),
        "0.328125",
        "fact sheet B-F4",
    )?;
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
    let e = 12u8;
    let ring = [a ^ b, b ^ c, c ^ d, d ^ e, e ^ mul(2, a, FOLD)];
    f.pin(
        "B-F9.card_values",
        ring.iter().map(u8::to_string).collect::<Vec<_>>().join(","),
        "6,4,5,14,6",
        "fact sheet B-F9",
    )?;
    let s = ring.iter().fold(0, |x, y| x ^ y);
    f.pin("B-F9.y5_xor_S_equals_3A", s, "15", "fact sheet B-F9")?;
    f.pin(
        "B-F9.A_solved_15_div_3",
        mul(s, 244, FOLD),
        "5",
        "fact sheet B-F9 (3⁻¹ = 244)",
    )?;
    Ok(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Carry-less multiply, then reduce by the full degree-8 polynomial: an
    /// independent route to the same products.
    fn clmul_reduce(a: u8, b: u8, poly: u16) -> u8 {
        let mut p = 0u16;
        for i in 0..8 {
            if b >> i & 1 != 0 {
                p ^= u16::from(a) << i;
            }
        }
        for bit in (8..16).rev() {
            if p >> bit & 1 != 0 {
                p ^= poly << (bit - 8);
            }
        }
        p as u8
    }

    #[test]
    fn the_panel_multiply_agrees_with_carryless_reduction_on_every_pair() {
        for a in 0..=255u8 {
            for b in 0..=255u8 {
                assert_eq!(mul(a, b, FOLD), clmul_reduce(a, b, 0x11D), "{a} * {b}");
                assert_eq!(mul(a, b, AES_FOLD), clmul_reduce(a, b, 0x11B), "{a} * {b} (AES)");
            }
        }
    }

    #[test]
    fn multiplying_is_adding_clicks_and_every_byte_has_a_partner() {
        let log = logs();
        let mut exp = [0u8; 255];
        for a in 1..=255u8 {
            exp[log[a as usize] as usize] = a;
        }
        for a in 1..=255u8 {
            for b in 1..=255u8 {
                let clicks = (u16::from(log[a as usize]) + u16::from(log[b as usize])) % 255;
                assert_eq!(mul(a, b, FOLD), exp[clicks as usize]);
            }
            // the partner sits at the mirror image: log a⁻¹ = 255 − log a
            let partner = exp[((255 - u16::from(log[a as usize])) % 255) as usize];
            assert_eq!(mul(a, partner, FOLD), 1);
        }
    }
}
