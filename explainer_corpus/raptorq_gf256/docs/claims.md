<!-- Claim walk (gate G3) of script.tsv at 49ef758, 2026-10-11: the walker's final answer, verbatim (scratch paths shortened to <scratch>/). Fixes are applied in the storyboard; see the commit that lands this file. -->

# Claim walk — Video B (script at HEAD)

The script is at HEAD `49ef758`, and I edited no repo file.

## Summary

I found **172 claims** over 103 spoken lines, plus the numbers on screen at those beats:

| Status | Count |
|---|---|
| **MISMATCH** | 6 |
| EDGE | 39 (4 fail as spoken) |
| UNSOURCED | 28 (only 5 lack support) |
| unchecked | 3 |
| OK | 96 |

**MISMATCH (fix before narration or render):**
1. **14_6:** "it's two hundred pieces missing" is wrong. The B-F18 stalls leave 116–176 pieces missing at 2% extra and 97–135 at 5% extra. Say "over a hundred".
2. **01_8, screen:** "an average carousel phone ≈ 26,000" counts whole loops. The expected packets until a phone is done is **23,730 sent**, about 23,490 caught. I got that from an exact sum, and a 40,000-phone Monte Carlo gave 23,706. It is not yet in the oracle (Maya D1).
3. **10_7, code panel:** when `0x1D` swaps to `0x1B`, the comment `mul(2, 142) == 1` stays on screen and becomes false (it gives 7). AES's inverse of 2 is 141.
4. **13_6, screen:** "exact: 1.607" is the many-piece limit. On the ten-piece race the exact mean is **1.606** (1.6057).
5. **14_1:** the voice says "about forty trillion" but the screen shows "≈ 4.2×10¹³". The two round differently; the value is 4.17×10¹³.
6. **10_4, screen:** "285 / 0x11D" sits under `x⁸ → x⁴+x³+x²+1`. The right side of that arrow is 29 (0x1D); 285 is the whole polynomial.

**EDGE claims that fail as spoken:**
- **01_5:** "Broadcast TV can't ask at all". ATSC 3.0 has an optional return channel (A/323).
- **06_5:** "it would almost never be". That is true only in a field; with wrap-around arithmetic, the next chapter's, it is still 1/2.
- **08_2:** "every number has a partner". Zero has none.
- **14_8:** "Any ten thousand packets rebuild the picture", followed by byte Reed–Solomon "tops out around 256". The two sentences contradict each other.

**EDGE claims that hold but are loose:**
- **13_5:** "only halves it". The first spare divides by 1.68.
- **15_3:** "just short of full". At 10 pieces that is 17 of 27 slots.
- **17_5:** "go first". The standard does not require it.
- **11_7:** "every byte's partner". Zero is excluded.
- **03_8** "endless" and **14_8** "never runs dry". Each block is capped at 2²⁴ packets.

**UNSOURCED with no support yet** (5 of the 28):
- **03_9:** "the best fountain code today". RFC 6330 only claims it beats Raptor (RFC 5053).
- **11_7:** "how a phone multiplies fast". This is a claim about implementations.
- **16_1:** "RaptorQ's decoder mostly peels". D5 is still open, and at 10 pieces P = 10 of L = 27 symbols are parked on purpose.
- **13_6:** the race's seed and counts are not pinned.
- **03_8:** where the name "fountain" comes from.

The other 23 UNSOURCED are true:
- 15 are trivial arithmetic I recomputed.
- 4 are nontrivial values I recomputed that need an oracle key: 01_6 stragglers, 01_8 "last few take forever", 10_1 rotation, 14_1 half a day.
- 4 hold per a primary source I read but have no fact-sheet row: ATSC A/331 citing RFC 6330, RFC 8681 for random byte codes, CRC-8/SAE-J1850 using 0x1D, and the ESI telling the phone which pieces a packet mixed.

**unchecked:** "requests swamp the sender" (the memo cites RFC 3453; I didn't open it), "hash mixers use odd multipliers", and "finite fields exist exactly for prime-power sizes".

**What I ran:**
- **Oracle:** `oracle.py`, 297 keys, into `<scratch>/oracle.tsv`.
- **Recomputations:** `<scratch>/checks.py` with `python3 -I -B` (output in `checks.tsv`). It covers:
  - exhaustive byte checks: XOR, Z/256 bijections, inverses, the mirror rule, rank over GF(2) vs GF(256), the cycle rule;
  - finite-K spare averages, the danger-meter tails, the carousel sum, the B-F18 missing-piece counts, and every products/logs value on screen.
- **Carousel check:** `<scratch>/carousel_mc.py`, a Monte Carlo cross-check.
- **Spelling of numbers:** a grep of the script's words finds no digits, symbols, colons, semicolons, dashes or parentheses (only the generated `#` header). Number style mixes "two fifty-six" (15 times) with "two hundred fifty-six" (4 times); that is not an error.
- **facts.rs:** all 31 pinned literals equal the oracle (scripted comparison). I did not build the binary or run `facts-check`. The file's header says it computes every number the video shows or says, but most spoken numbers are not in it yet, for example 97,876, 71/42/23%, 6,316/24/1, 1.607 and 4.17×10¹³.

**Sources I fetched** (kept in their own folders under `<scratch>/`):
- [RFC 6330](https://www.rfc-editor.org/rfc/rfc6330.txt) (sha256 matches the fact sheet's copy)
- [ATSC A/331:2021](https://www.atsc.org/wp-content/uploads/2021/01/A331-2021-Signaling-Delivery-Sync-FEC.pdf)
- [ATSC A/323 listing](https://www.atsc.org/atsc-documents/a-3232018-dedicated-return-channel-for-atsc-3-0) (search result only; I did not open the standard)
- [reveng CRC catalogue](https://reveng.sourceforge.io/crc-catalogue/1-15.htm)
- [FIPS 197-upd1](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.197-upd1.pdf)
- [RFC 8681](https://www.rfc-editor.org/rfc/rfc8681.txt)

## Claims

| line | claim | source | status | note / fix |
|---|---|---|---|---|
| 01_1 | "a million phones" | B-F2 (n = 10⁶), B-F24 (10⁶ phones) | OK | Screen: ~1,000 glyphs × 1,000 each ✓ |
| 01_1 | "loses about one packet in a hundred" | model input p = 1% (B-F2, B-F24) | OK | A model assumption, given in the description (RB-07) |
| 01_2 | "almost every phone that catches just two packets more than it needs rebuilds the picture exactly" | B-F23 = RFC 6330 §5.8 (≤ 10⁻⁶ at K′+2); I re-read §5.8 in rfc6330.txt (sha256 87f6da89…21ace, same as the fact sheet) | EDGE | Holds as the standard's requirement (averaged over random ESIs). Measured only at 10 pieces (1 in 10⁶; CP95 up to 5.6×10⁻⁶, B-F23 cp95 flag 0), never at 10,000 |
| 01_2 | "three times seven is nine" | B-F11.mul_3_7 = 9 | OK | Screen chalk `3 × 7 = 9` ✓ |
| 01_4 | "cut into ten thousand pieces" | B-F1.K = 10000 | OK | Screen 10,000 ✓ |
| 01_5 | "each packet goes missing for about ten thousand of them" | B-F2.expected_misses_per_packet = 10000 (SD 99.5) | OK | — |
| 01_5 | "the requests swamp the sender" | memo §2.6 (quotes RFC 3453, "feedback implosion") | unchecked | Not a fact-sheet row; I did not open RFC 3453 |
| 01_5 | "Broadcast TV can't ask at all" | none | EDGE | Fails as stated for ATSC 3.0: the ATSC document list (web search result; I did not open it) shows A/323:2018, "Dedicated Return Channel for ATSC 3.0", which is optional. Fix: "an antenna can't ask at all" or "broadcast TV mostly can't ask" |
| 01_6 | "Most phones need two or three loops" | B-F24: per phone, done by 1 loop 2.2×10⁻⁴⁴, by 2 0.368, by 3 0.990 | OK | 99.0% finish in loop 2 or 3 (recomputed) |
| 01_6 | "The unluckiest of a million need five or six" | B-F24: all 10⁶ done by 4 loops 3.7×10⁻⁴⁴, by 5 0.368, by 6 0.990 | OK | — |
| 01_6 | (screen) "after loop 3 a few red stragglers" | none; recomputed from the B-F24 formula | UNSOURCED | Recomputed: 0.995% of phones (≈ 9,950, about 10 of the 1,000 glyphs) are still waiting. Consistent with "a few" |
| 01_7 | "Sending random pieces is worse" | B-F1 (97,876) vs B-F24 (mean 2.64 loops) | OK | — |
| 01_7 | "all ten thousand" | B-F1.K | OK | — |
| 01_8 | "On average, about ninety-eight thousand" | B-F1.K_times_H_K = 97,876.06 | OK | Screen count-up 97,876 and "random ≈ 98,000" ✓ |
| 01_8 | "The last few take forever, because you keep getting doubles" | none; recomputed | UNSOURCED | Recomputed: the last piece alone takes 10,000 draws on average, and the last ten take 29,290 (30% of 97,876). Holds |
| 01_8 | (screen) "an average carousel phone ≈ 26,000" | no key (storyboard says pin it first, Maya D1) | **MISMATCH** | 2.64 loops × 10,000 counts whole loops, but a phone finishes partway through its last loop. Exact expected packets until done is **23,730 sent** (exact sum; Monte Carlo 23,706 over 40,000 phones), ≈ 23,490 caught. Show "≈ 24,000" and pin the key. Also use one unit for all three counts: 97,876 and 10,002 are packets caught |
| 01_8 | (screen) "two extra packets 10,002" | K + 2 | OK | — |
| 02_1 | "One, two, four, eight" | none (place values) | UNSOURCED | Trivial and recomputed. Screen ✓ |
| 02_2 | "Five is four plus one, so it's one, zero, one" | none | UNSOURCED | Trivial: bin(5) = 101 |
| 02_2 | "A byte … holds zero to two fifty-five" | none | UNSOURCED | Trivial: 2⁸ − 1 = 255. Screen 11111111 = 255 ✓ |
| 02_5 | "Five is one, zero, one. Seven is one, one, one" | none | UNSOURCED | Trivial |
| 02_6 | "Zero, one, zero. Two." | none | UNSOURCED | Recomputed 5 ⊕ 7 = 2. Screen ✓ |
| 02_6 | "flip twice, and you're back where you started" | none | EDGE | Holds: (a ⊕ b) ⊕ b = a for all 65,536 byte pairs (ran) |
| 02_6 | "Two ex-or seven is five again" | none | UNSOURCED | Recomputed 2 ⊕ 7 = 5. Screen ✓ |
| 03_1 | (screen) cards A⊕C = 2, B⊕C⊕D = 6, A⊕B = 6, A⊕B⊕C⊕D = 3 | B-F3 | OK | — |
| 03_3 | "Ex-or card one with card three, and A cancels. B ex-or C is four" | B-F3.card1_xor_card3 = 4, B_xor_C = 4 | OK | — |
| 03_4 | "D is two" | B-F3.D_from_card2_and_BxC = 2 | OK | Screen fill-in 5, 3, 7, 2 = B-F3 inputs; rank 4 makes them unique ✓ |
| 03_5 | "It doesn't care which cards arrive, or in what order" | B-F3 rank | EDGE | Holds: rank 4 in all 24 orders of the four cards (ran); a span never depends on order |
| 03_5 | (screen) bucket of four slots | B-F3 (4 pieces) | OK | — |
| 03_6 | "B ex-or C is four" | B-F3.B_xor_C = 4 | OK | — |
| 03_7 | "You could make it yourself from cards one and three" | B-F3.rank_gf2_with_5th_card_BxC = 4; card1 ⊕ card3 = 4 | OK | — |
| 03_8 | "A sender can make endless fresh mixes" | RFC 6330 abstract and §1 ("as many encoding symbols as needed"); §3.2 ESI is 24-bit (read) | EDGE | Holds in practice. Strictly, the cap is 2²⁴ = 16,777,216 per block (the 14_8 lower-third says so). Not a fact-sheet row |
| 03_8 | "so these are called fountain codes" | RFC 6330 abstract ("RaptorQ is also a fountain code") | UNSOURCED | The term is confirmed. The fountain etymology (the "digital fountain" of Byers, Luby, Mitzenmacher and Rege, 1998) is not sourced in the repo |
| 03_8 | "a packet's number tells the phone which pieces it mixed" | RFC 6330 §5.3.5.4 Tuple[K′, X]; §5.3.1 ISI = ESI (+ K′ − K for repair), read; memo line 349 | UNSOURCED | Holds once K′ is known. The "pieces" are intermediate symbols, which the 15_2 lower-third explains. Add a row |
| 03_8 | (screen lower-third) "the ESI is the seed; the block size travels in a header" | RFC 6330 §5.3.1, §3.3 (read) | EDGE | Holds as a simplification: the seed is the ISI plus J(K′), and the OTI carries F and T, from which K follows |
| 03_9 | "The best fountain code today is called RaptorQ" | none | UNSOURCED | RFC 6330 claims only that it beats Raptor (RFC 5053): "superior flexibility … better coding efficiency than Raptor codes". Nothing backs "best". Fix: "the fountain code in today's standards" or cite a source |
| 04_1 | "there are eight possible cards" | B-F4 (K = 3, eighths) | OK | 2³ = 8 |
| 04_3 | "you can make two things … Add B, and you can make four" | B-F4; recomputed span sizes 2, 4, 8 | OK | Screen 1 → 2 → 4 ✓ |
| 04_3 | "Each useful card doubles what you can make" | none | EDGE | Holds: checked every ordered triple of independent vectors in GF(2)⁴ (ran) |
| 04_5 | "Four lit corners out of eight. Half." | B-F4.useless_prob_3pieces_2cards_held = 0.5 | OK | — |
| 05_2 | "Seven eighths, times six eighths, times four eighths" | B-F4.success_K3_check = 0.328125 | OK | — |
| 05_2 | "About one time in three, so it fails two times in three" | B-F4 (0.328, failure 0.672) | OK | Screen 0.328 → 0.672 ✓ |
| 05_2 | "With many pieces it fails seventy-one percent" | B-F5.fail_gf2_h0 = 0.71121 | OK | Screen 0.7112 ✓ |
| 05_3 | "Almost all the danger is in the last few slots" | B-F4 limit; recomputed | EDGE | Holds: risk outside the last 3 / 5 / 8 slots is 12.0% / 3.1% / 0.39% (ran; matches the Ruth note) |
| 05_4 | "Three slots from the end … one time in eight. Then one in four. At the last slot, one in two" | B-F4 factors 7/8, 6/8, 4/8 | OK | Screen 1/8, 1/4, 1/2 ✓ |
| 05_4 | "Ten pieces or ten million, the answer is about seventy-one percent" | B-F5.fail_gf2_h0_K10 = 0.71093; limit 0.71121 | EDGE | Holds: both round to 71% (it rises with K; at K = 5 it is 70.2%) |
| 06_2 | "Nearly one time in four" | B-F5.fail_gf2_h2 = 0.2299 | OK | 1 in 4.35 |
| 06_2 | "from seventy-one to forty-two percent" | B-F5 h0 = 0.7112, h1 = 0.4224 | OK | — |
| 06_2 | "each one after that roughly halves it" | B-F5; recomputed ratios | EDGE | Holds: the factors are 1.84, 1.92, 1.96, 1.98 (ran) |
| 06_2 | "almost a quarter of a million stuck" | B-F28.phones_stuck_of_1e6_at_h2 = 229,898 | OK | Screen 71 / 42 / 23 / 12% ✓ |
| 06_3 | "Reaching one in a million takes about twenty spares" | B-F28.min_spares_for_fail_le_1e-6 = 20 | OK | Screen: twenty rungs ✓ |
| 06_3 | "Ten halvings make about a thousand. Twenty make about a million" | none | UNSOURCED | Trivial: 1,024 and 1,048,576 |
| 06_4 | "Every letter appears twice, so together they cancel" | B-F9 ring | EDGE | Holds: column counts 2, 2, 2, 2, 2 (ran) |
| 06_4 | "The five are worth only four" | B-F9.rank_gf2 = 4 | OK | Any four of the five have rank 4 (ran; Ruth note) |
| 06_5 | "a card at the last slot is useless half the time, and the ring stays stuck" | B-F4; the ring's span is 16 of 32 (ran) | OK | — |
| 06_5 | "With two hundred fifty-six amounts to choose from, it would almost never be" | B-F7.success_K1 = 255/256 | EDGE | Holds only in a field. With wrap-around amounts, the next chapter's arithmetic, it is still useless half the time (B-F25). Fix: "it could almost never be, if the arithmetic works" |
| 07_1 | "Bytes could wrap around at two fifty-six … call that mod" | definition | OK | Fine as setup, though ×3 can be undone mod 256 (3 · 171 ≡ 1). The failure is the even amounts |
| 07_3 | "Zero." | B-F10.z256_2x128 = 0 | OK | Screen `2·128 ≡ 0` ✓ |
| 07_4 | "It could be three, or one thirty-one" | B-F10.z256_2x3 = 6, z256_2x131 = 6 | OK | Exhaustive: 2A ≡ 6 has exactly the solutions {3, 131} (ran). Screen 262 → 6 ✓ |
| 07_5 | "Even amounts lose information … Odd ones never do" | B-F10.row_a_has_256_over_gcd_values_all_a_ok | EDGE | Holds: x ↦ ax mod 256 is a bijection exactly when a is odd, all 256 a (ran) |
| 07_5 | "only the last bit matters" | B-F25 (invertible ⇔ det odd ⇔ invertible mod 2; certificate 4,000/4,000) | EDGE | Holds for whether the cards pin every piece down. It does not hold for partial information (2A = 6 still gives A mod 128). 200,000 random 2×2 cases agree (ran) |
| 07_6 | "fails seventy-one percent of the time, just like ex-or" | B-F25 + B-F5 (MC K = 8: 0.2853 vs 0.2899) | OK | Screen: both counters → 71% ✓ |
| 07_6 | (screen lower-third) "3 × 171 = 513 = 2·256 + 1" | none | UNSOURCED | Recomputed ✓ (trivial) |
| 07_6 | (screen lower-third) "it's why hash mixers use odd multipliers" | none | unchecked | Not checked; needs a citation (e.g. multiplicative hashing) |
| 08_2 | "Five." | none | UNSOURCED | Recomputed: 3 · 5 = 15 ≡ 1 (mod 7) |
| 08_2 | "Every row of its times table, except zero's, is a reshuffle of all seven values" | none | EDGE | Holds: rows 1–6 mod 7 are all permutations (ran) |
| 08_2 | "So every number has a partner that multiplies it back to one" | none | EDGE | Fails at 0, which has no partner (ran). Fix: "every number but zero" |
| 08_3 | "two fifty-six is two times one twenty-eight" | B-F10 | OK | Screen ✓ |
| 08_3 | "The nearest prime is two fifty-seven" | none | EDGE | Holds: the primes nearby are 251 and 257, so 257 is nearest (ran) |
| 08_3 | "an hour two fifty-six, which doesn't fit in a byte" | none | UNSOURCED | Trivial: 256 needs 9 bits. Screen: the ninth bit spills ✓ |
| 08_4 | "a field … with exactly two hundred fifty-six members" | B-F12 (such fields exist: 30 irreducible octics) | OK | — |
| 08_4 | (screen lower-third) "finite fields exist exactly for prime-power sizes" | none | unchecked | A standard theorem; I did not verify it or find a row |
| 09_1 | "Numbers can't do it" | B-F10 (Z/256 has 127 nonzero non-units) | EDGE | Holds: 256 is not prime, so no Z/n of size 256 is a field |
| 09_1 | "Polynomials can" | B-F12.0x11D_irreducible = 1 | OK | — |
| 09_2 | "Three was two plus one, so now it's x plus one" | none | UNSOURCED | Trivial. Screen 10000000 → x⁷, 00000011 → x + 1 ✓ |
| 09_3 | "adding is ex-or again" | RFC 6330 §5.7.2 "u + v = u ^ v" (read) | EDGE | Holds by definition |
| 09_4 | "seven is x squared plus x plus one" | none | UNSOURCED | Trivial |
| 09_5 | "Six terms, and two pairs match … x cubed plus one … one, zero, zero, one. Nine." | B-F11.foil_3x7_carryless = 9 | OK | Screen FOIL ✓ |
| 09_6 | "Three times seven is nine" | B-F11.mul_3_7 = 9 | OK | Holds under all 30 irreducible folds (ran), as the brief says |
| 10_1 | "drops the fallen bit back in at the bottom. A rotation" | B-F26 (the fold mod x⁸ + 1) | UNSOURCED | Recomputed: ×2 mod x⁸ + 1 rotates left for all 256 bytes. No key states the rotation |
| 10_3 | "Zero." | B-F26.mul_3_255_mod_x8_plus_1 = 0 | OK | Screen `3·255 = 0` ✓ |
| 10_3 | "x to the eighth plus one splits into x plus one, times a long polynomial. That's three times two fifty-five" | B-F26.carryless_3_255 = 257 (0x101) | OK | — |
| 10_4 | "RaptorQ's rule … swap in x to the fourth plus x cubed plus x squared plus one" | B-F14 (all 510 OCT_EXP = powers of x mod 0x11D) | OK | RFC 6330 §5.7 never names the polynomial and defines the field only by its tables. The script's wording is still accurate |
| 10_4 | "a polynomial that can't split. A prime polynomial" | B-F12.0x11D_irreducible = 1 | OK | — |
| 10_4 | (screen) `x⁸ → x⁴ + x³ + x² + 1` with "285 in decimal; 0x11D beneath" | B-X2 (285 = 0x11D) | **MISMATCH** | 285 is the whole polynomial x⁸ + x⁴ + x³ + x² + 1. The arrow's right side is 29 (0x1D), the number 10_5 says aloud. Put 285 / 0x11D under the full polynomial, or 29 / 0x1D under the right side |
| 10_5 | "Two times one twenty-eight is x to the eighth" | none | UNSOURCED | Trivial: x · x⁷ |
| 10_5 | "sixteen plus eight plus four plus one. Twenty-nine. Not zero" | B-F11.mul_2_128 = 29; fold_x8_residue = 29 | OK | Screen 00011101 ✓; panel 0x1D ✓ |
| 10_6 | "every nonzero byte now has a partner" | B-F11.nonzero_rows_are_permutations_ok | EDGE | Holds: all 255 nonzero bytes have inverses (ran) |
| 10_6 | "Two times one forty-two is exactly one" | B-F11.mul_2_142 = 1; inv_2 = 142 | OK | Panel `mul(2, 142) == 1` ✓ (B-F11.plan_rust_mul_2_142) |
| 10_7 | "A famous encryption standard picked a different prime polynomial" | FIPS 197-upd1 §4.2 eq. 4.3, m(x) = x⁸+x⁴+x³+x+1 (fetched and read); B-F12.0x11B_irreducible = 1 | OK | The fact sheet has no FIPS 197 quote: add one. My multiplier reproduces FIPS's worked examples ({57}•{13} = {fe}) |
| 10_7 | "the same two bytes can multiply to different answers" | B-F11 0x57·0x83 = 49 (0x11D) vs 193 (0x11B) | OK | Screen 87 · 131 = 49 vs 193, hex 0x31 vs 0xC1 ✓ |
| 10_7 | (screen, code panel) "0x1D swaps to 0x1B and 0x31 flips to 0xC1" | B-F11.inv_2_under_0x11B = 141; ran the panel code with 0x1B | **MISMATCH** | The comment `mul(2, 142) == 1` stays on screen and becomes false: under 0x1B, mul(2, 142) = 7. Flip it to `mul(2, 141) == 1` as well, or dim it |
| 10_7 | (screen lower-third) "CRC-8/SAE-J1850 uses this very 0x1D" | reveng CRC catalogue (fetched): poly=0x1d init=0xff xorout=0xff check=0x4b | UNSOURCED | Holds; I reproduced check 0x4b. Add a fact-sheet row |
| 11_1 | "One, two, four, eight, sixteen, thirty-two, sixty-four, one twenty-eight" | B-F14.oct_exp_0_to_12 | OK | — |
| 11_2 | "Twenty-nine" | B-F14 (OCT_EXP[8] = 29) | OK | — |
| 11_2 | "you visit every nonzero byte exactly once before coming back. Two hundred fifty-five steps" | B-F12.ord2_under_0x11D = 255; B-F14.distinct_powers = 255 | EDGE | Holds (ran) |
| 11_4 | "A clock with two hundred fifty-five hours" | B-F12 | OK | — |
| 11_5 | "two cubed times two to the fourth is two to the seventh" | none | UNSOURCED | Trivial: 8 · 16 = 128, also in GF(256) (ran) |
| 11_5 | "One twenty-eight times four. Seven clicks plus two is nine clicks. Fifty-eight" | B-F13 log_128 = 7, log_4 = 2, exp_7_plus_2 = 58; B-F11.mul_128_4 = 58 | OK | Screen 2⁷·2² = 2⁹ = 58 ✓ |
| 11_6 | "One fifty is a hundred eighty clicks, and three is twenty-five … two hundred five clicks, which is one sixty-seven" | B-F13 log_150 = 180, log_3 = 25, sum 205, exp_205 = 167; B-F11.mul_150_3 = 167 | OK | Screen: the ring rotates 25 ✓ |
| 11_7 | "It's how a phone multiplies fast. Look up two click counts, add them, and look up the answer" | RFC 6330 §5.7.2 defines × by OCT_EXP/OCT_LOG (B-F14) | UNSOURCED | The standard defines multiplication this way, but "how a phone multiplies fast" is a claim about implementations, and nothing backs it. Bulk kernels often use SIMD nibble tables (memo line 19: the owner's gf256.rs has log/exp tables plus AVX2/NEON kernels). Fix: "It's how the standard writes it down" |
| 11_7 | "every byte's partner sits at its mirror image across the top" | facts.rs test; recomputed | EDGE | Holds for all 255 bytes on the clock (log a⁻¹ = 255 − log a, ran). 0 has no partner (it sits at the centre), and 1 is its own mirror. Screen "127 mirror pairs" ✓ (127 pairs plus the fixed point 1) |
| 11_8 | "two hundred fifty-six hours … two hundred fifty-five" | B-F10, B-F12 | OK | — |
| 12_1 | "It looks like noise" | qualitative | OK | "Looks", not "is", as the storyboard's engine note requires |
| 12_4 | "perfect diagonal stripes. Each stripe is one total of clicks" | B-F13.log_ordered_table_is_hankel_ok (all 255²) | EDGE | Holds |
| 12_4 | (screen lower-thirds) AES ord 2 = 51 and hand 3; orders 1…255 occur 1, 2, …, 128 times; 255 = (2+1)(2²+1)(2⁴+1); RAID-6 51 weights; "7 × 7: 198 + 198 = 396 → 141 → 21" | B-F12, B-X1; log 7 = 198 recomputed (no key) | OK | All recomputed ✓. log 7 = 198 needs a key before it goes on screen |
| 13_1 | "This design really exists" | none in the repo; RFC 8681 (Jan 2020, Standards Track): random linear codes over GF(2⁸), full density (fetched) | UNSOURCED | Holds. Add a row (RFC 8681, or Ho et al. 2006 for RLNC) |
| 13_2 | "Half the corners were lit … useless half the time" | B-F4 | OK | — |
| 13_4 | "Only one time in two fifty-six" | B-F7.success_K1 = 255/256 | OK | Brute force at K = 2: 256 of 65,536 cards are useless (ran) |
| 13_4 | "Scaling by a nonzero byte reshuffles all two fifty-six values" | B-F11.nonzero_rows_are_permutations_ok | OK | — |
| 13_4 | "exactly one choice of amount lands in the lit part" | none | EDGE | Holds for any one piece's amount with the others fixed: exactly one a for each b at K = 2 (ran) |
| 13_5 | "every extra card divides the risk by two fifty-six" | B-F8 | EDGE | Holds approximately: ratios 255.996 and 255.99998 (ran). Screen ladder 0.39% / 0.0015% / 0.000006% ✓; lower-third B-F27 ✓ |
| 13_5 | "With ex-or, each extra card only halves it" | B-F5.ratio_fail_h0_over_h1 = 1.684 | EDGE | The first spare divides by 1.68, then 1.84, 1.92, 1.96. Say "roughly halves", as 06_2 does |
| 13_6 | "Ex-or needs one or two spares on average" | B-F6.mean_spares_gf2 = 1.6067 (K → ∞) | OK | At 10 pieces it is 1.6057 (Σⱼ≤₁₀ 1/(2ʲ−1), ran) |
| 13_6 | "Byte cards almost never need even one" | B-F8.h0; recomputed at K = 10 | OK | P(any spare needed) = 0.392% |
| 13_6 | (screen) "exact: 1.607 / 0.0039" on the ten-piece race | B-F6 (K → ∞ limit) | **MISMATCH** | At ten pieces the exact mean is **1.606** (1.6057); 1.607 is the many-piece limit. 0.0039 ✓. Show 1.606, or label 1.607 as "many pieces" |
| 13_6 | (screen) histograms from "this run's own counts", "a thousand times over" | none (D15, RB-23 open) | UNSOURCED | Pin the seed and counts in the oracle |
| 13_6 | (screen lower-third) "1 + 1/3 + 1/7 + … = 1.6067…; Erdős proved it irrational in 1948" | B-F6; fact sheet open item 1 (VERIFIED) | OK | — |
| 13_7 | "A bigger field gains nothing if the amounts are still only zeros and ones" | B-F9.rank_gf256_same_01_matrix = 4 | EDGE | Holds: rank does not change under field extension; 2,000 random 6×6 0/1 matrices give equal ranks (ran) |
| 13_7 | "the other two fifty-four values" | none | UNSOURCED | Trivial: 256 − 2 |
| 14_1 | "about the number of pieces, cubed, over three" | B-F15 (ledger ≈ K³/3) | OK | — |
| 14_1 | "For fifty thousand pieces, that's about forty trillion steps" | B-F15.K_cubed_over_3 = 4.1667×10¹³ | **MISMATCH** | Voice and screen round differently: the voice says forty (one figure, down), the screen "≈ 4.2×10¹³" (two figures, up). Say "about forty-two trillion" or show "≈ 4×10¹³" |
| 14_1 | "At a billion a second, half a day" | none; recomputed | UNSOURCED | 41,667 s = 11.57 h ✓; screen "≈ 11.6 h" ✓. Add a key |
| 14_2 | "About half mix two" | B-F16.P_deg2 = 1/2 exactly | OK | In RaptorQ each card also takes 2–3 PI symbols (B-F16 E[d1] = 2.34); fine for "fast codes" |
| 14_2 | "a few mix many" | B-F16.P_deg30_tail = 2.95% | OK | — |
| 14_3 | (screen) peel order C, A, B, D, E | B-F20.peel_order | OK | — |
| 14_5 | "Nothing moves" | B-F20.recovered_without_y1 = 0 | OK | — |
| 14_6 | "With exactly enough sparse cards, peeling usually gets stuck early" | B-F18.eps000 (5/5 runs stall at 8.25–42.1%) | OK | — |
| 14_6 | "Even with a few percent extra, it stalls around ninety-eight percent" | B-F18 eps002 98.24–98.84% (4 of 5); eps005 98.65–99.03% | OK | Really 98–99% |
| 14_6 | "it's two hundred pieces missing" | B-F18 × 10,000 | **MISMATCH** | The measured stalls leave **116–176** pieces missing at 2% extra and **97–135** at 5% extra, never 200. Say "over a hundred pieces missing" |
| 14_6 | (screen caption) "one run with 2% extra stuck at 16%" | B-F18.eps002.min_recovered = 0.1578 | OK | — |
| 14_7 | "About one piece in a hundred and twenty isn't in any card at all" | B-F17: e^−4.817 = 0.809% → 1 in 123.5 | OK | Right at two figures, but only for exactly enough cards. The sentence follows the "few percent extra" line, and with 2% / 5% extra it is 1 in 136 / 1 in 157 (ran) |
| 14_7 | "No decoder can find it" | none | EDGE | Holds for a plain sparse code: no equation contains that piece |
| 14_8 | "Reed-Solomon codes are ideal. Any ten thousand packets rebuild the picture" | memo §2.6 (RS is MDS) | EDGE | Contradicts the next sentence: over bytes RS has N ≤ 257, so it can't hold 10,000 pieces in one block, and "any ten thousand" needs a larger field. Fix: "With big enough arithmetic, any ten thousand…" |
| 14_8 | "with byte arithmetic that tops out around two hundred fifty-six" | memo §2.6 (n ≤ q + 1 = 257) | OK | Lower-third "N ≤ 257 (q + 1)" ✓. The memo is the source, not a fact-sheet row |
| 14_8 | "A fountain never runs dry" | RFC 6330 §3.2: ESI is a 24-bit unsigned integer (read) | EDGE | The cap is 2²⁴ per block. Lower-third "2²⁴ ≈ 16.8 million" ✓ (16,777,216); add the row |
| 15_1 | "Its packets never use byte amounts. Every one is a plain ex-or card" | RFC 6330 §5.2 ("each such repair symbol is the exclusive-or of a small number of intermediate symbols") and §5.3.5.3 Enc (read); research rq6330.py enc_indices, byte-exact vs cberner | EDGE | Holds. The fact sheet does not quote it: add |
| 15_2 | "They're never sent, so the phone gets them free" | RFC 6330 §5.3.3.4.2: D starts with S + H zero symbols (read) | OK | Quote not yet in the fact sheet |
| 15_2 | "Only ten to sixteen of them are byte cards, whatever the block size" | B-F19.H_min = 10, H_max = 16 over all 477 K′ | EDGE | Holds for every K′ in Table 2 |
| 15_2 | "In big blocks, hundreds of others are cheap ex-ors" | B-F19 S = 241 at 10,017 and 907 at 56,403 | OK | — |
| 15_2 | (screen) "7 ex-or rules + 10 byte rules + 10 packets", 27 × 27; "241 ex-or + 11 byte rules" | B-F19.K10 (S 7, H 10, L 27); B-F19.K10017 (S 241, H 11) | OK | — |
| 15_3 | "The ex-or cards fill the bucket to just short of full, before the danger starts" | B-F19 (L − H = K′ + S) | EDGE | Always H short. But at K′ = 10, the block shown in 15_2 and measured in 17, that is 17 of 27 slots (63%). It holds in the meter's terms: 10 slots from the end a random ex-or card is useless 1 time in 1,024 |
| 15_3 | "The byte cards fill the last ten to sixteen slots, where a card is useless only one time in two fifty-six" | B-F19 H; B-F7 (model) | OK | A model statement: the real HDPC rows are structured (random byte rows 0.685% vs RFC rows 0.646%, B-F21) |
| 15_5 | "About seventy-one percent failure" | B-F21.recorded.K10_gf2_hdpc_h0 = 70.66% (K′ = 101: 70.2%) | OK | Screen 70.7% ✓, rounded the same way |
| 15_5 | "With the byte cards, about six in a thousand" | B-F21.recorded.K10_rfc_hdpc_h0 = 0.646% | OK | Screen 0.646% ✓ |
| 16_1 | "RaptorQ's decoder mostly peels" | none (D5 open: no measured inactivation count) | UNSOURCED | At K′ = 10 the decoder parks P = 10 of L = 27 symbols on purpose (37%, B-F19). At 10,017 it parks 158 of 10,269 (1.5%) plus unmeasured dynamic inactivations. Measure first, or say "in big blocks" |
| 16_1 | "When it's stuck, it parks one stubborn piece" | RFC 6330 §5.4.2.2 (read) | EDGE | A simplification. The RFC moves r − 1 columns of a minimum-r row and prefers r = 2 (one piece); it also parks the PI symbols from the start. The lower-third covers that |
| 16_2 | "A is two ex-or c. B is six ex-or two ex-or c" | B-F20.y_values (y2 = 2, y4 = 6) | OK | — |
| 16_2 | "D is six ex-or six ex-or two, so c cancels. And E follows from D" | B-F20 (y3 = 6, y5 = 14); D = 2 (ran) | OK | Screen E = 14 ⊕ D ✓ |
| 16_3 | "c is fifteen ex-or fourteen ex-or six, which is seven" | B-F20.c_equals_y6_xor_y5_xor_y3 = 7; the only consistent value of 256 | OK | Screen back-substitution 5, 3, 2, 12 = B-F20 ✓ |
| 16_3 | "it works exactly when the cards pin down every piece" | RFC 6330 §5.4.2.1: "C can be decoded if and only if the rank of A is L" (read) | EDGE | Holds. Quote it in the fact sheet (the lower-third cites it) |
| 16_4 | "E plus two times A is six" | B-F9.card_values (fifth = 6) | OK | Words say the byte card "joins"; the picture and B-F9 replace the fifth card. Both solve, since the old fifth card was redundant. Consider "takes the fifth card's place" |
| 16_5 | "one ex-or two is three" | B-F9.one_xor_two = 3 | OK | — |
| 16_5 | "Three A is fifteen, so A is five" | B-F9.y5_xor_S_equals_3A = 15, A_solved = 5; B-F11.mul_3_5 = 15 | OK | Screen ✓ |
| 16_5 | "The same A, B, C, and D as at the start" | B-F9 and B-F3 both use 5, 3, 7, 2 | OK | — |
| 16_5 | (screen lower-thirds) "parks 158 pieces on purpose at 10,017"; "a loop … is stuck exactly when its amounts multiply to 1 going around" | B-F19.K10017.P = 158; RFC §5.4 (PI "permanently inactivated", read); cycle rule recomputed | OK | The cycle rule matched in 3,000 random 5-cycles over GF(256) (ran) |
| 17_1 | "a million times, on small blocks of ten pieces" | B-F22.recorded.K10.h0.trials = 10⁶ | OK | — |
| 17_1 | "it failed six thousand three hundred sixteen times. About six in a thousand" | B-F22.recorded.K10.h0.failures = 6316 (provenance match) | OK | Caption "0.63% at 101 pieces, 0.54% at 1,002" = B-F22 ✓ |
| 17_2 | "Each extra card divides the risk by about two fifty-six" | B-F8 (model 256.0); B-F22.K10.ratio_h0_over_h1 = 263 | OK | — |
| 17_3 | "Twenty-four. Two extra, just once" | B-F22 h1 = 24, h2 = 1 | OK | Screen 6,316 / 24 / 1 ✓ |
| 17_3 | "too rare to measure precisely" | B-F22 CP95 2.5×10⁻⁸ – 5.6×10⁻⁶ | OK | — |
| 17_4 | "at most one in a hundred, one in ten thousand, and one in a million, averaged over random packets, for every block size it supports" | B-F23 = RFC 6330 §5.8 (re-read) | OK | "Every block size" means every padded K′ in Table 2, which the lower-third states |
| 17_5 | "The originals go first, unchanged" | RFC 6330 §1, systematic: "the original unmodified source symbols … can be sent" (read) | EDGE | "Unchanged" holds. "Go first" is the sender's choice; RFC 6330 does not require it |
| 17_5 | "a phone with a clean connection does no decoding at all" | RFC 6330 §1 (systematic); memo line 287 | EDGE | Holds: receiving all K source symbols is the block |
| 17_6 | "It's in the standard for next-generation TV broadcasts" | fact sheet: NOT re-checked (D17). I fetched ATSC A/331:2021 (19 Jan 2021): §2.1 normative ref [28] RFC 6330; Annex A.4.2.4 "The FEC building block is used according to RFC 6330" | UNSOURCED | Holds. Add the quote and close D17 |
| 17_6 | "Its older sibling was chosen for mobile broadcast in two thousand five" | fact sheet open item 4 (SP-050246, June 2005: Raptor, RFC 5053) | OK | Card "3GPP MBMS, 2005 (Raptor)" ✓ |
| 17_6 | "the same byte field protects disk arrays that survive two dead drives" | B-X1 + Anvin quote (0x11D, g = {02}) | OK | Card "Linux RAID-6" ✓ |
| 17_6 | "and every QR code" | B-X2 + ISO/IEC 18004:2006 §6.5.2 quote | EDGE | Holds for the 2006 edition read. The current 2024 edition is unchecked (fact sheet's limit) |
| 17_7 | "Two extra packets buy the standard's one in a million" | B-F23 | OK | It is the standard's requirement. Our 10⁶ trials cannot confirm 10⁻⁶ (B-F23.recorded_K10_cp95_upper_le_requirement = 1,1,0); the script does not claim they do |
| 17_7 | "every nonzero byte has a partner to divide by" | B-F11.nonzero_rows_are_permutations_ok | EDGE | Holds (ran) |
| 17_7 | "a polynomial refuses to factor" | B-F12.0x11D_irreducible = 1 | OK | — |
| 17_7 | "Three times seven is nine" | B-F11.mul_3_7 = 9 | OK | — |

All scratch work is in `<scratch>/`: `oracle.tsv`, `checks.py`, `checks.tsv`, `carousel_mc.py` and `rows.tsv`, with the downloaded sources in the `dl_*` folders.
