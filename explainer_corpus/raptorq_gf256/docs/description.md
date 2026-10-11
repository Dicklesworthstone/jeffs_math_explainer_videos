# The Arithmetic Where 3 × 7 = 9: description (draft)

Draft of the YouTube description, written at R2 (2026-10-11). It becomes `publish/youtube.md` at release
(bead mx-e-b-qyy.16), where the chapter timestamps come from the cue logs. Every number below is an oracle key or a
fact-sheet row (`fact-sheet.md` beside this file); the "Simplifications → exact" table there lists each entry's
backing. YouTube limits a description's length, so at publish the full "Precisely" list may stay in this file in the
repository, with the YouTube text keeping the summary, the chapters, the note on what the promise averages over, the
sources and a link here.

---

RaptorQ, the fountain code in an internet standard and in ATSC 3.0 broadcasting, sends packets that are plain ex-ors
of pieces of a file, and a random set of exactly enough ex-or equations fails to pin the file down about 71% of the
time (model), almost all of it in the last few equations. RaptorQ closes those last few with 10 to 16 equations over
a 256-element byte field, where bytes multiply as polynomials folded by one that cannot be factored, which is why
3 × 7 = 9.

## Chapters

(timestamps at publish)

1. A million phones
2. Bits and ex-or
3. Packets are equations
4. The cube
5. The ex-or lottery
6. Spares and rings
7. Why not wrap around at 256?
8. Prime clocks
9. Bytes are polynomials
10. The fold
11. The byte clock
12. The times table
13. The byte lottery
14. The price of density
15. Where the danger lives
16. Parking a piece
17. Two packets

## What the promise averages over

The standard's figures, and our runs, are for packets chosen at random: RFC 6330 §5.8 averages over packet IDs drawn
independently and uniformly from the whole ID range, for each padded block size K′ of its Table 2, and our decoder
runs drew from a fixed pool of the K originals and 20,000 repair packets. A phone that loses random packets from a
stream that sends the originals first receives a different mix of packets, which neither covers.

## Precisely

The video simplifies; here is the exact version of each simplification, by beat (chapter_line). "Model" marks a
computed probability under stated assumptions; measured numbers carry their block size and trial count.

**01_1** Model: each phone loses each packet independently with probability 1%. With a million phones, each packet
then misses 10,000 of them on average (standard deviation 99.5).

**01_2, 17_4, 17_7** RFC 6330 §5.8: a compliant decoder fails at most 1 time in 100, in 10,000 and in 1,000,000 when
it receives K′, K′ + 1 and K′ + 2 packets, on average over uniformly random packet IDs. See "What the promise averages
over" above.

**01_6** A phone is done by the end of loop 2 with probability 0.368 and by loop 3 with 0.990, so 99.005% finish in
loop 2 or 3; after loop 3, 0.995% still wait. All 10⁶ phones are done by loop 5 with probability 0.3679 and by loop 6 with 0.9900. The
mean is 2.642 loops per phone.

**01_8** All three counts are packets caught, on average: random pieces K·H_K = 97,876.06 (98,864.7 sent at 1% loss);
an average carousel phone 23,492.65 (23,729.95 sent; it finishes partway through its last loop); a fountain phone with
two extra 10,002 (10,103.0 sent). Of the 97,876, the last piece alone takes 10,000 draws and the last ten take 29,289.7.

**02_6** The ex-or of several bits is 1 exactly when an odd number of them are 1.

**03_5** "Useful" means linearly independent of the cards already held; the bucket's filled slots are the rank.

**03_8** The deck is a fixed recipe written into the standard: a packet's ID (the ESI) fixes its internal index, and
RFC 6330's Tuple[K′, X] fixes which pieces it mixes; the block size travels in a header.

**03_9** RaptorQ is RFC 6330, an IETF Proposed Standard published in August 2011.

**05_2** K uniform random 0/1 cards are all useful with probability ∏(1 − 2⁻ⁱ), i = 1…K: 21/64 = 0.328 at K = 3, and
0.288788 in the limit, so the lottery fails 71.0930% at ten pieces and 71.1212% in the limit (model).

**05_3, 05_4** The chance of failing outside the last three slots is 11.99%: the rest of the 71% is in those three.

**06_2** With 0, 1, 2, 3 spares the lottery fails 71.1%, 42.2%, 23.0%, 12.0% (model): the first spare divides the risk
by 1.684, later ones by more, tending to 2 (1.959 from the third spare to the fourth). With a million phones, two
spares leave 229,898 stuck.

**06_3** The smallest number of spares with failure at most one in a million is 20 (9.54×10⁻⁷; with 19 it is
1.907×10⁻⁶).

**06_4** The ring's five cards have rank 4 over GF(2), and still 4 over GF(256) for the same 0/1 cards.

**06_5** With uniform amounts in a field, a card at the last slot is useless with probability 1/256 (13_4); with
wrap-around amounts it is still 1/2 (07).

**07_6** A K × K matrix over Z/256 is invertible exactly when its determinant is odd, exactly when it is invertible mod
2; uniform entries reduce to uniform bits, so success is ∏(1 − 2⁻ⁱ) again (checked at K = 8: 0.2853 in 40,000 samples
against 0.2899). For programmers: 3 × 171 = 513 = 2·256 + 1, so multiplying by an odd constant is reversible, which is
why multiplicative hashes use odd multipliers (Linux's `hash.h`: "This hash multiplies the input by a large odd number
and takes the high bits").

**08_4** Finite fields exist exactly for prime-power sizes.

**09_5** In the FOIL, x² + x² and x + x vanish because 1 + 1 = 0 (never "2x²"). 3 × 7 = 9 under every one of the 30
degree-8 fold polynomials, because the product has degree 3 and no fold happens.

**10_1, 10_3** The rotation is multiplication mod x⁸ + 1 = (x + 1)⁸, which factors, so 3 · 255 = 0 there: the
carry-less product of 3 and 255 is x⁸ + 1 itself.

**10_4, 10_6** "A prime polynomial" means irreducible over GF(2). If p cannot be factored, no two nonzero bytes
multiply to 0, so each row of the times table has no repeats, so it contains 1: every nonzero byte has a partner.
GF(2⁸) = GF(2)[x]/(p) for any irreducible p of degree 8; all fields of order 256 are isomorphic, and the polynomial is
a choice of labels. The byte field is GF(256), also written GF(2⁸). RaptorQ's p is x⁸ + x⁴ + x³ + x² + 1 (285, 0x11D).
For programmers: CRC-8/SAE-J1850 uses this very 0x1D (its 7 attested codewords reproduce with the panel's `xtime`).

**11_2** Under 0x11D, 2 reaches all 255 nonzero bytes (0x11D is primitive: 16 of the 30 irreducible degree-8
polynomials are). Every finite field has such a clock. Under AES's polynomial, times two comes back to 1 after 51
steps; its hand is 3.

**11_4** For programmers: Linux RAID-6 gives drive i the weight 2ⁱ in this field; with AES's polynomial only 51 of
those weights would differ.

**11_5, 11_6** 7 × 7: 198 + 198 = 396, which is 141 clicks after one lap, and α¹⁴¹ = 21; by hand
(x² + x + 1)² = x⁴ + x² + 1 = 21. 150 × 3: 180 + 25 = 205 clicks, α²⁰⁵ = 167.

**11_7** A byte's partner sits at minus its clicks, mod 255 (255 − its clicks, with a full turn counting as 0): 127
mirror pairs, and 1 is its own partner; 0 has none.

**12_0** RFC 6330 §5.7.2 defines multiplication through two tables, OCT_EXP and OCT_LOG; implementations may compute it
other ways. The code is below.

**12_4** The log-sorted times table is constant along anti-diagonals (a Hankel matrix). Element orders 1, 3, 5, 15, 17,
51, 85, 255 occur 1, 2, 4, …, 128 times; each count is a power of 2 because 255 = (2 + 1)(2² + 1)(2⁴ + 1), and
together they make 11111111₂ = 255.

**12_5** AES (FIPS 197) folds by x⁸ + x⁴ + x³ + x + 1 (283, 0x11B): 87 · 131 is 49 under RaptorQ's polynomial and 193
under AES's, and AES's partner of 2 is 141, not 142.

**13_1** Cards with random amounts from a finite field are random linear network coding (Ho et al., 2006).

**13_4** Fix the other amounts; for a piece whose own card isn't lit, exactly one of its 256 amounts puts the card in
the lit part (checked exhaustively for two pieces).

**13_5** Model, uniform byte amounts: failure with h spares is below the union bound 256⁻ʰ/255; exactly it is
3.9215×10⁻³, 1.5319×10⁻⁵ and 5.984×10⁻⁸ at h = 0, 1, 2 (the bound at h = 0 is 3.9216×10⁻³).

**13_6** The race: CPython's `random.Random(1306)`, 1,000 races a side, uniform cards including the zero card. Ex-or
spares 0, 1, 2, … occurred 297, 303, 171, 94, 65, 39, 17, 4, 7, 0, 2, 0, 0, 0, 0, 1 times (mean 1.603); byte spares 0
and 1 occurred 995 and 5 times (mean 0.005). Exact means at ten pieces: 1 + 1/3 + 1/7 + … + 1/1023 = 1.6057 and
0.0039. With many pieces the ex-or sum is 1.6067…, the Erdős–Borwein constant, which Erdős proved irrational in 1948.

**13_7** A matrix of 0/1 amounts has the same rank over GF(2) and over GF(256).

**14_1** Dense Gaussian elimination takes about K³/3 multiply-adds: 4.17×10¹³ at K = 50,000, 11.57 hours at 10⁹ a
second. For programmers: even ten times faster it is still 1.16 hours; the matrix alone is 2.5 GB; doubling the pieces
multiplies the work by 8.

**14_2** RFC 6330's degree table: P(d) ≈ 1/(d(d − 1)) for d = 2…29 (the ideal soliton, within a relative 5.0×10⁻⁴),
so a card mixes two pieces with probability exactly 1/2; mean degree 4.82.

**14_3** For programmers: per piece, a list of the cards that use it, plus a queue of cards with one unknown left (the
ripple).

**14_6** Five seeded runs (`random.Random(42)`) with 10,000 pieces and the standard's degree table: with exactly enough
cards, peeling recovered 8.25–42.1%; at 2% extra, four runs stopped at 98.24–98.84% (116–176 pieces missing) and one
collapsed at 15.8%; at 5% extra, 98.65–99.03%. Peeling is not elimination: it can stop where elimination would succeed.

**14_7** At 2% extra (10,200 cards), a piece is in no card with probability (1 − 4.8166/10,000)^10,200 = 0.734%: 73.4 of
the 10,000 on average (one in 136). With exactly enough cards it is one in 124.

**14_8** For programmers (this aside left the video to keep chapter 14 under 85 s): Reed–Solomon codes are the ideal.
Any set of exactly enough packets rebuilds the picture, but the sender must fix the packet count in advance, and with
byte arithmetic it tops out at 257. A fountain never runs dry. Precisely: Reed–Solomon codes are MDS: any K of the N
packets rebuild the data. Over GF(256), N is at most 257 (q + 1, with the point at infinity); checked for K = 3 on
3,000 random triples of the 257 columns. RaptorQ's packet IDs are 24 bits: 16,777,216 per block.

**15_1** "About seven pieces": a packet is the ex-or of 7.152 intermediate symbols on average, Table 1's mean degree
4.82 plus 2.34 of the permanently inactivated symbols (modelling the standard's Rand[X, 3, 2] as uniform). The extra
pieces of 15_2 are counted.

**15_2** RaptorQ adds S + H extra unknowns tied to the pieces by S + H fixed equations whose right side is zero (RFC
6330 §5.3.3.3: they "evaluate to zero"); packets are ex-or cards over all of them, and the sender solves for them all
so the first K′ packets are the originals. Every one of the L = K′ + S + H unknowns appears in some free card, for every
one of the standard's 477 block sizes. Model: with exactly K′ = 10,017 packets, about 85 of the 10,111 LT symbols are in
no packet; the 252 free cards still reach them.

**15_6** Big files are cut into blocks of up to 56,403 pieces; each block decodes on its own. H, the number of byte
rows, is between 10 and 16 for all 477 block sizes; at 10,017 pieces there are S = 241 ex-or rows and H = 11 byte rows.

**15_3** The ex-or rows always stop exactly H short of the unknowns: L − H = K′ + S, so 17 of 27 slots at ten pieces
and 10,258 of 10,269 at 10,017. "One time in 256" is the model for uniform byte rows; the standard's byte rows are
structured (at ten pieces, uniform random byte rows failed 137 / 20,000 = 0.685% and the standard's 0.646%).

**15 (model)** Random rows in RaptorQ's own shape at ten pieces (17 uniform 0/1 rows and 10 uniform byte rows over 27
unknowns) fail 0.489% = 1 − (1 − 0.0976%)(1 − 0.392%); the gap to the measured 0.63% is the price of sparse,
structured ex-or rows.

**15_5** Our own RFC 6330 implementation of the equations, ten pieces, packet IDs uniform over 2²⁴: 646 failures in
100,000 trials with the standard's byte rows (0.646%) and 7,066 in 10,000 with those rows replaced by ex-or rows
(70.66%; 70.2% at 101 pieces). 17_1's decoder measured 0.632% in another harness; the two agree (z = 0.55).

**16_1** RFC 6330 §5.4.2.4: "The number of rows i of the submatrix U_upper is generally much larger than the number of
columns u of U_upper", so in big blocks most steps are peels; at ten pieces, 10 of the 27 unknowns are parked from
the start. RaptorQ parks 158 pieces on purpose at 10,017 pieces (search: inactivation decoding). The sender also runs
one solve, to send the originals unchanged first.

**16_3** Decoding succeeds if and only if the cards have rank L, in any elimination order (RFC 6330 §5.4.2.1).

**16_5** Here 3 is the byte x + 1, not a count: A ⊕ A ⊕ A is just A. A ring of cards a·X + b·Y is stuck exactly when
the ratios b/a multiply to 1 going around; with ex-or every ratio is 1.

**17_1** raptorq 2.0.0 (Rust, by cberner), K = 10, 10⁶ trials for each of K, K + 1 and K + 2 packets, drawn from a
fixed pool of the 10 originals and 20,000 repair packets: 6,316, 24 and 1 failures. Also 1,262 in 200,000 (0.631%) at
101 pieces and 108 in 20,000 (0.54%) at 1,002 pieces (both runs partial).

**17_2** Why a plain ex-or packet divides the risk by about 256: counting the ex-or cards first (15_3), one more ex-or
packet almost always takes one more slot, so one byte card has nothing left to fill, and a spare byte card divides
the risk by about 256 (13_5). In the shape model at ten pieces, the extra ex-or row is useful with probability 0.998,
that case is 65% of the failures left, and the model's factor is 208; the real decoder measured 6,316 / 24 = 263.

**17_3** One failure in 10⁶ has a 95% Clopper–Pearson interval of 2.53×10⁻⁸ to 5.57×10⁻⁶ (24 in 10⁶: 1.54×10⁻⁵ to
3.57×10⁻⁵; 6,316: 0.616% to 0.647%), so it cannot confirm one in a million. Dividing by 256 twice predicts about 0.096
failures in 10⁶ here, and a count with that mean is at least 1 about 9.2% of the time.

**17_6** ATSC A/331:2021, the ATSC 3.0 delivery standard, names RFC 6330 for its FEC, which is optional ("may be
protected with FEC"). 3GPP chose Raptor (RFC 5053), RaptorQ's predecessor, for MBMS in 2005; RaptorQ is not in TS
26.346 (V19.3.0). Linux's md RAID-6 uses this field (Anvin). QR codes use 285 = 0x11D (ISO/IEC 18004:2006 §6.5.2; later
editions were not checked).

**17_7** Names to search: fountain code, LT code, Raptor, RaptorQ, RFC 6330, inactivation decoding, GF(256),
carry-less multiplication, primitive polynomial, Gaussian elimination. LT codes: Luby (2002); Raptor codes: Shokrollahi
and Luby; RFC 6330 (2011): Luby, Shokrollahi, Watson, Stockhammer and Minder.

## Sources

- M. Luby, A. Shokrollahi, M. Watson, T. Stockhammer, L. Minder, "RaptorQ Forward Error Correction Scheme for Object
  Delivery", RFC 6330, IETF, August 2011 (Proposed Standard): https://www.rfc-editor.org/rfc/rfc6330 (§3.2, §4.4.2,
  §5.2, §5.3, §5.4.2, §5.6, §5.7, §5.8, §9.2). Its informative references, which we did not open: M. Luby, "LT codes",
  FOCS 2002, pp. 271–280; A. Shokrollahi and M. Luby, "Raptor Codes", Foundations and Trends in Communications and
  Information Theory 6(3–4), 2011.
- RFC 5053 (Raptor), RFC 3453 (feedback implosion), RFC 8681 and RFC 8406 (random linear codes): rfc-editor.org.
- ATSC A/331:2021, "Signaling, Delivery, Synchronization, and Error Protection" (§1, normative reference [28],
  Annex A.3.5.2, A.4.1, A.4.2.4).
- 3GPP SP-050246 (TSG SA #28, June 2005), the choice of Raptor for MBMS; TS 26.346 V19.3.0; TR 26.947 V19.0.0.
- J. W. Byers, M. Luby, M. Mitzenmacher, A. Rege, "A Digital Fountain Approach to Reliable Distribution of Bulk Data",
  ACM SIGCOMM '98, §III.
- T. Ho, M. Médard, R. Koetter, D. R. Karger, M. Effros, J. Shi, B. Leong, "A Random Linear Network Coding Approach to
  Multicast", IEEE Trans. Inf. Theory 52(10), 2006.
- H. P. Anvin, "The mathematics of RAID-6"; Linux `lib/raid/raid6/mktables.c`; Linux `include/linux/hash.h`.
- ISO/IEC 18004:2006 (QR Code 2005), §6.5.2.
- FIPS 197-upd1, Advanced Encryption Standard, §4.2.
- P. Erdős, "On arithmetical properties of Lambert series", J. Indian Math. Soc. 12 (1948), 63–66.
- The reveng CRC catalogue, entry CRC-8/SAE-J1850.
- Wikipedia, "Finite field" (prime-power sizes; a secondary source).

## Code

Every number in this video was computed by code, and each is checked against an independent oracle:
https://github.com/Dicklesworthstone/jeffs_math_explainer_videos (the video: `explainer_corpus/raptorq_gf256/`; its
oracle: `oracle.py` there; the research scripts: `docs/research/raptorq_gf256/`).

The code panel from chapter 10, RaptorQ's byte multiply with no tables (with `0x1B` in place of `0x1D` it is AES's,
where `mul(2, 141) == 1`):

```rust
fn xtime(a: u8) -> u8 { (a << 1) ^ if a & 0x80 != 0 { 0x1D } else { 0 } }
fn mul(mut a: u8, mut b: u8) -> u8 {      // shift-and-add, no tables
    let mut acc = 0;
    while b != 0 { if b & 1 != 0 { acc ^= a } a = xtime(a); b >>= 1 }
    acc                                   // mul(0x57, 0x83) == 0x31, mul(2, 142) == 1
}
```

The table multiply from chapter 11, as the standard writes it down. The log table needs 16-bit entries: with 8-bit
logs, LOG[a] + LOG[b] wraps at 256 for 32,131 of the 65,025 nonzero pairs, chapter 7's broken clock again (a panic in
a debug build, a wrong product in a release build). EXP has 510 entries because the largest index is 254 + 254 = 508,
so no `% 255` is needed.

```rust
static EXP: [u8; 510] = [/* 2^0, 2^1, ..., 2^509 */]; // largest index 254 + 254 = 508: no % 255
static LOG: [u16; 256] = [/* LOG[0] is never read */]; // u16: a u8 sum wraps at 256 (chapter 7's bug)
fn mul(a: u8, b: u8) -> u8 {                            // the tables are elided here
    if a == 0 || b == 0 { 0 } else { EXP[(LOG[a as usize] + LOG[b as usize]) as usize] }
}
```
