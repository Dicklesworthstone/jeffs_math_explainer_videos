<!-- R2 script review, 2026-10-11, from the reviewer's final answer (packet tools/review.py; script at 85fbff2) -->

# R2 script review: raptorq_gf256 (Dev, 28)

There are no blockers. Three findings are major and would change the video:

1. **dev-01:** 14_7 says about one piece in a hundred and twenty is in no card and "no decoder can find it". Chapter 15 never says how RaptorQ, whose packets are also sparse ex-or cards, gets past that. My R1 tie-back (D4) was never added.
2. **dev-02:** the video prices the dense solve exactly but never gives one number for RaptorQ's cost. D5 is still deferred to an open bead, and narration locks the words.
3. **dev-03:** the 11_7 table-multiply lower-third doesn't compile with the natural Rust types. The obvious fix then gives wrong products, silently in release builds, for about half of all byte pairs.

## Comprehension check

| ch | What I'd tell a colleague |
|---|---|
| 01 | A 10,000-piece picture goes to a million phones that can't ask for resends. A carousel costs about 2.6 loops (≈23,500 packets caught) and 5–6 loops for the unluckiest. Random pieces cost ≈98,000. A fountain phone needs just K+2 = 10,002. |
| 02 | Ex-or flips the bits where the second number has ones, and doing it twice undoes it. |
| 03 | Each packet is a linear equation: a subset of pieces ex-ored together equals a value. Both sides derive the subset from the packet number with a shared recipe. The phone needs any K independent equations, in any order, and solves them by elimination. |
| 04 | With r useful cards you can make 2ʳ of the 2ᴷ possible cards. So a random card is useless with probability 2^(r−K). |
| 05 | K random ex-or cards are full rank only ≈29% of the time, for any K, because the last few slots fail with probability 1/8, 1/4, 1/2. |
| 06 | Each ex-or spare only about halves the risk (20 spares for 10⁻⁶), and some structures, like the ring of five, are dependent no matter what. So we want byte coefficients, which needs a byte multiply we can undo. |
| 07 | Mod 256, even multipliers destroy information (2·128 = 0). Random byte amounts are then only as good as their low bit, so failure is 71% again. |
| 08 | A prime modulus gives inverses (a field), but 257 doesn't fit in a byte. We need a field with exactly 256 elements. |
| 09 | Bytes become polynomials over GF(2). Addition is ex-or and multiplication is carry-less, so 3·7 = 9 with no overflow. |
| 10 | Reduce overflow with a polynomial that can't be factored: shift, and if the top bit falls off, ex-or in 0x1D (the CRC move). Rotation fails because x⁸+1 factors. AES's 0x11B gives different products, so both ends must agree. |
| 11 | Under 0x11D, powers of 2 reach all 255 nonzero bytes. So multiplying is adding logs mod 255 (EXP/LOG tables), and a byte's inverse is at 255 − log. |
| 12 | Sorted by log, the multiplication table is constant along anti-diagonals: stripes. |
| 13 | With random byte coefficients, the last slot fails 1 time in 256 and each spare divides the risk by 256. This only works if the coefficients use values other than 0 and 1. |
| 14 | A dense solve costs K³/3 (half a day at 50k pieces). Sparse cards peel cheaply but stall near 98%, and ≈1 piece in 120 is in no card at all. |
| 15 | RaptorQ packets stay sparse ex-or. Both sides add fixed equations with zero right-hand sides, and the 10–16 GF(256) ones close the last directions; swapping them for ex-or rows takes failure from 0.65% to 71%. *I had to supply what the ex-or free cards do myself (dev-01).* |
| 16 | Peel. When stuck, park a variable as a symbol, finish peeling, then solve the small dense system with the leftover and byte rows and back-substitute. It succeeds exactly when the whole system has full rank. *I can't say what it costs (dev-02).* |
| 17 | The real decoder at K = 10 fails 6,316 / 24 / 1 times per million at K, K+1, K+2, and the RFC requires at most 1/100, 1/10⁴, 1/10⁶. It's systematic. ATSC 3.0 names it, Raptor was chosen for MBMS in 2005, and 0x11D is in Linux RAID-6 and QR codes. |

No sentence is wrong or empty. 15 and 16 are the weak ones.

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 4 | 5 | 5 | 4 | 4 |
| 02 | 4 | 5 | 3 | 3 | 2 |
| 03 | 5 | 4 | 5 | 4 | 4 |
| 04 | 4 | 5 | 4 | 4 | 4 |
| 05 | 5 | 5 | 5 | 4 | 5 |
| 06 | 4 | 5 | 5 | 4 | 4 |
| 07 | 5 | 5 | 4 | 4 | 5 |
| 08 | 4 | 5 | 4 | 4 | 3 |
| 09 | 5 | 5 | 4 | 4 | 4 |
| 10 | 4 | 5 | 5 | 4 | 5 |
| 11 | 4 | 4 | 4 | 4 | 5 |
| 12 | 4 | 5 | 3 | 4 | 4 |
| 13 | 4 | 5 | 5 | 4 | 4 |
| 14 | 4 | 4 | 5 | 4 | 4 |
| 15 | 3 | 4 | 5 | 4 | 5 |
| 16 | 3 | 5 | 4 | 4 | 4 |
| 17 | 4 | 4 | 4 | 4 | 4 |

Chapter 02's pacing is slow for me, but that is Maya's call, so there is no finding for it.

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| dev-01 | Dev | 14_7 → 15_1–15_3 | major | 14_7 says about one piece in a hundred and twenty "isn't in any card. No decoder can find it." Then 15_1 says RaptorQ's packets are sparse ex-or cards too, and 15_3 says they fill the bucket "to just short of full". Nothing between those lines says why RaptorQ doesn't hit the same floor. The reason is the precode, the core idea of Raptor codes: every intermediate symbol sits in at least one free card, so a piece no packet touched can still be pinned down. As spoken, I leave thinking the ex-or free cards are filler and the byte cards do all the work. R1 D4 asked for this tie-back in chapter 15. The disposition fixed 14_7 only, and neither the storyboard nor the script contains it (I grepped for "touch"). | In 15_2, after "the phone gets them free", add: "Every piece sits in some of these free cards, so even a piece no packet touched can be pinned down." Check it before narration. In `rq6330.py` `precode_rows` (RFC 6330 §5.3.3.3), each of the first W − S symbols goes into three LDPC rows: b, b+a, b+2a mod S. Those rows are distinct because S is an odd prime and a < S (a ≤ 41 at K′ = 10,017 and ≤ 62 at 56,403). The S LDPC symbols get identity entries, and the HDPC rows are dense over the first K′+S columns. Add an oracle row for the LT-untouched intermediate symbols at K′ = 10,017 with N = K′ (my rough e^(−d̄N/W) estimate is about 85), set against S + H = 252. |
| dev-02 | Dev | 14_1 vs 13_1, 15_1, 16_1–16_3 | major | The dense solve gets an exact price (K³/3, "half a day"; I recomputed 41.67×10¹² and 11.57 h). RaptorQ only ever gets "cheaper" (13_1, 15_1) and "mostly peels" (16_1). R1 D5 was deferred to bead mx-e-b-qyy.10, which is still open. Once narration is generated, the video ships with no RaptorQ cost at all. Also, 14_2's "About half mix two" is the only density figure I hear, so I'd build RaptorQ packets that mix two pieces. Its repair packets actually mix about 7.15 intermediate symbols (B-F16). | **Now, from an existing oracle row:** 15_1 "Every one is a plain ex-or of about seven pieces" (B-F16 = 7.152). **Before narration:** time cberner `raptorq` 2.0.0 (the venv `rq_mc.py` already uses) decoding one 10,000 × 1 KB block from K+2 packets on one core. Record the inactivation count too, and pin both as RECORDED rows. Then add one sentence at 16_3: "On one laptop core, the whole ten-thousand-piece block decodes in about ⟨measured⟩." Don't voice D5's "1.6 million steps instead of 3.6×10¹¹". 169³/3 = 1.6 million is only the dense core. The back-substitution alone is up to L·u = 10,269 × 169 ≈ 1.7 million whole-piece (1 KB) operations, so a step count undersells the real work. Measure it instead. |
| dev-03 | Dev | 11_7 lower-third (for programmers) | major | `EXP[LOG[a] + LOG[b]]` doesn't compile if `a: u8`, because Rust indexes arrays by usize. The natural fix is a trap. Logs fit in a byte, so I'd declare `LOG: [u8; 256]`, which is what the crate does (`facts::logs() -> [u8; 256]`, and its own test widens to `u16` before adding). Then `LOG[a] + LOG[b]` overflows u8 for **32,131 of the 65,025** nonzero pairs (I counted them). Debug builds panic; release builds return a wrong product. That silent wrap at 256 is chapter 07's broken clock, in my own code. | Show the types: `static EXP: [u8; 510]; static LOG: [u16; 256];` … `EXP[(LOG[a as usize] + LOG[b as usize]) as usize]`. Add to the lower-third: "a u8 LOG wraps at 256: chapter 7's bug." The 510 is right: the largest index is 254 + 254 = 508. |
| dev-04 | Dev | 14_7 | minor | 14_7 follows the "two percent extra" sentence, so I take its 1 in 120 to be at 2% extra. It is the N = K value: e^−4.8166 is 1 in 123.5. At 2% extra it is 1 in 136, about 74 of the 10,000 pieces, which are part of the 116–176 missing. | "With exactly enough cards, about one piece in a hundred and twenty isn't in any card." Or keep the 2% frame: "About seventy of them aren't in any card." That needs an oracle key: 10,000·e^(−4.8166·1.02) = 73.5. |
| dev-05 | Dev | 16_2 | minor | "D is six ex-or six ex-or two, so c cancels." The c is already gone from what I hear, so the "so" doesn't follow, and D = 2 is never said. I had to rebuild it from card y3 (D = y3 ⊕ B ⊕ C). The values are right: A = 2⊕c, B = 6⊕2⊕c, D = 2, E = 12, and c = 15⊕14⊕6 = 7 is the only one of 256 values consistent with y6. | "D is six ex-or B ex-or c. B carries a c too, so the two c's cancel, and D is two." Screen: `D = 6 ⊕ (6 ⊕ 2 ⊕ c) ⊕ c = 2`. |
| dev-06 | Dev | 11_8 | minor | The rhyme sets the 256-hour clock against the 255-hour clock as if they were the same kind of clock. I'd come away with "so byte multiply is mod 255". Z/255 isn't a field either: 3·85 ≡ 0. The real difference is that one clock counts bytes and the other counts clicks. | "The wrong clock counted bytes, and multiplying on it broke. The right one counts clicks, two hundred fifty-five of them, and going around it is multiplying." Or keep the words and add a lower-third: "clicks wrap at 255; bytes never do (mod 255 isn't a field: 3 × 85 = 255)." |
| dev-07 | Dev | 17_6 | minor | Two real-system claims are wider than their sources. (a) "disk arrays that survive two dead drives": the source covers Linux md RAID-6 only (Anvin, `mktables.c`). Dual-parity schemes such as NetApp RAID-DP and EVENODD use ex-or only (Corbett et al., FAST '04; Blaum et al., IEEE TC 1995; I did not open these). (b) "the standard for next-generation TV broadcasts": A/331:2021 names RaptorQ, but ATSC 3.0 is one of several next-generation broadcast standards, and its Annex A.4.1 makes the FEC optional ("may be protected"). | "It's in ATSC 3.0, a standard for next-generation TV broadcasts … the same byte field protects Linux's disk arrays that survive two dead drives, and every QR code." Description: "ATSC A/331:2021 Annex A; the FEC is optional." |
| dev-08 | Dev | 03_9 | minor | "an internet standard since twenty eleven": in IETF terms "Internet Standard" is a maturity level with an STD number. RFC 6330 is a Proposed Standard (the RFC Editor record in B-S1). Anyone who looks it up will say it isn't one. | Keep the voice and put the exact status on the name card: "RFC 6330 · IETF Proposed Standard · August 2011". |
| dev-09 | Dev | 14_1 | minor | "At a billion a second" is the number a programmer argues with. GF(256) multiply-adds vectorize with SIMD shuffle tables, so "half a day" sounds like a straw man and I stop listening. The cubic growth is the real argument, and the matrix alone is 50,000² bytes = 2.5 GB. | "At a billion a second, half a day. Even ten times faster, over an hour." (41,667 s / 10 = 1.16 h, computed.) Lower-third: "the matrix alone is 2.5 GB; double the pieces, eight times the work." |
| dev-10 | Dev | 17_2–17_3 | minor | 17_2 teaches "÷256 per card". 6,316 / 256² ≈ 0.1, yet 17_3 reports one failure at K+2. "Too rare to measure precisely" is honest, but anyone who does the division will think the rule broke. It didn't: with a mean of 0.096, at least one failure happens 9% of the time. The exact 95% interval for 1 in 10⁶ is 2.53×10⁻⁸ to 5.57×10⁻⁶ (recomputed). | Lower-third at 17_3: "the ÷256 rule predicts about 0.1 here; one is within the noise (95%: 0.025 to 5.6 per million)." |
| dev-11 | Dev | 13_6 screen and lower-third | minor | The race shows "exact: 1.606" and the lower-third shows "1.6067…". They look like one number rounded wrong. They are 1.6057 (ten pieces) and 1.6067 (the limit); I recomputed both. | Show both to four places with labels: "ten pieces 1.6057 · many pieces 1.6067…". |
| dev-12 | Dev | 13_4 | minor | "exactly one choice of amount lands in the lit part": one choice of which amount? The 1/256 comes from fixing every amount but one and noting that scaling by a nonzero byte is a reshuffle. That argument is what I'd want to re-derive. | "Fix every amount but one. Of its two hundred fifty-six values, exactly one lands in the lit part." |
| dev-13 | Dev | 09_5, 11_2, 16_3; 17_7 names lower-third | minor | Three search terms I'd actually type are missing: "carry-less multiplication" (09_5 is exactly that), "primitive polynomial" (why 2 visits all 255 bytes under 0x11D but only 51 under 0x11B; I recomputed 30 irreducible and 16 primitive) and "Gaussian elimination" (16_3). Also, the AES "51 steps" lower-third sits in chapter 12's notes, but the claim it qualifies is spoken at 11_2. | Add the three terms to 17_7's names-to-search lower-third. Show the 51-steps lower-third at 11_2. |
| dev-14 | Dev | 07_6 and 14_8 lower-thirds | minor | Two programmer-facing claims rest on weak sources. "it's why hash mixers use odd multipliers" is backed only by Wikipedia (B-S8). Reed–Solomon "N ≤ 257 (q + 1)" has no fact-sheet row. Both are true as far as I know. | Add source rows. Candidates I have not verified: Knuth, TAOCP Vol. 3 §6.4 (multiplicative hashing), and MacWilliams & Sloane, ch. 10–11 (doubly extended Reed–Solomon, n = q + 1). |
| dev-15 | Dev | 15_4–15_5, 17_1–17_3 | delight | The ablation is the experiment I'd have run myself: same harness, byte rows swapped for ex-or rows, 0.646% → 70.66%. The real decoder's 6,316 → 24 is a ratio of 263, close to 256. The mechanism and the measurement agree. | Keep both. |
| dev-16 | Dev | 10_4–10_7 panel and CRC lower-third | delight | The panel does exactly what the voice says: shift, and if the top bit falls off, ex-or in 0x1D. The AES flip and the CRC aside make it code I've written before. | Keep as is. |
| dev-17 | Dev | 16_4–16_5 | delight | The ring walk lands back on 03's A, B, C, D (cards 6, 4, 5, 14, 6 → 3A = 15 → A = 5), and "adding amounts is ex-or too" is said out loud. | Keep. |

## What I checked and how

My script is `<scratch>/r2_dev_b/check.py` (run with `python3 -I`). It shares no code with the oracle.

- **Code panel (storyboard lines 224–228).**
  - It is byte-identical to the R1 panel, which R1 compiled with rustc. I did not compile it this round, per the rules.
  - Reading it as Rust:
    - In Rust, `&` binds tighter than `!=`, unlike C.
    - `u8 << 1` drops the top bit without a debug panic.
    - `if … else` is a valid operand of `^`.
    - `acc` infers `u8`.
  - Transliterated to Python with u8 masking, it equals the carry-less product mod 0x11D on all 65,536 pairs. With `0x1B` it equals the product mod 0x11B on all 65,536 pairs.
  - The comments hold: `mul(0x57,0x83) = 0x31` and `mul(2,142) = 1`. After the swap they are 0xC1 and `mul(2,141) = 1`. `mul(2,142)` becomes 7 under 0x1B, so the comment flip at 10_7 is necessary.
- **CRC aside.** Looping the panel's `xtime` as CRC-8/SAE-J1850 (init and xorout 0xFF) gives the check value 0x4B and reproduces all 7 attested codewords.
- **Spoken numbers.** I recomputed every number below, and each one matches the script.
  - **Ex-or and the cards:** 5⊕7 = 2 and back to 5; cards 2, 6, 6, 3; B⊕C = 4; D = 2.
  - **Probabilities:** 7/8·6/8·4/8 = 0.328. Failure is 0.711 / 0.422 / 0.230 / 0.120 for 0–3 spares, and 0.7109 at K = 10. Twenty spares give 9.54×10⁻⁷. The byte model gives 3.92×10⁻³, 1.53×10⁻⁵, 5.98×10⁻⁸.
  - **Hook:** K·H_K = 97,876.06. The carousel figures are 0.368 / 0.990 per phone and 0.368 / 0.990 for all phones by 5 and 6 loops, with a mean of 2.642 loops.
  - **Wrap-around:** mod 256, 2·128 = 0, 2·131 = 6 and 3·171 = 513; on the seven-hour clock, 3·5 = 1. The rotation fold gives 3·255 = 0 (0x101).
  - **Byte field:** FOIL gives 9. 3·7 = 9 under all 30 irreducible polynomials. 2·128 = 29, 2·142 = 1, 128·4 = 58, 150·3 = 167, 7·7 = 21.
  - **Logs:** log 3 = 25, log 150 = 180, α²⁰⁵ = 167. The mirror rule holds. The log-sorted table is constant along anti-diagonals.
  - **Larger counts:** spare means 1.6057 / 1.6067 / 0.0039. The race counts sum to 1,000 with mean 1.603. 50,000³/3 and 11.57 h. e^−4.8166 = 1 in 123.5. 116–176 pieces missing at 2% extra.
  - **Toy decodes:** the inactivation toy and the ring walk.
  - **Measured runs:** 6,316 / 24 / 1 and the interval.
  - **RAID-6:** 255 distinct weights under 0x11D, 51 under 0x11B.
- **Unchecked by me:**
  - The recorded Monte Carlo counts (B-F21, B-F22) and the seeded race draws. I checked only the race's sum and mean.
  - All primary sources: RFC 6330, ATSC A/331, SP-050246, ISO 18004, Anvin, FIPS 197.
  - The RAID-DP, EVENODD, Knuth and MacWilliams–Sloane citations in dev-07 and dev-14.
  - The precode-coverage claim in dev-01, which I checked by reading `rq6330.py`'s construction, not by running it (that needs the RFC text).
