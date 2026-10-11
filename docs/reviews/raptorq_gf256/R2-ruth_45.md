<!-- R2 script review, 2026-10-11, from the reviewer's final answer (packet tools/review.py; script at 85fbff2) -->

# R2 review: Video B, *The Arithmetic Where 3 × 7 = 9* (Ruth Adler, ruth_45)

**No blockers. There are four majors (R1–R4).** Every number I recomputed matches the oracle at the shown precision, and voice and screen round it the same way. The problems are three statements that are wrong or unsupported as worded, and exact versions that have nowhere to live yet.

## What I checked and how

- **Oracle.** I ran `oracle.py` as instructed. Output is in `<scratch>/r2_ruth_b/oracle.tsv`.
  - 397 keys, 4.56 s.
  - All 37 `_ok` keys print 1, and all 11 `provenance_check` keys print `match`.
  - I edited nothing in the repo; `git status` is the same as at the start.
- **My own code.** I wrote `<scratch>/r2_ruth_b/checks.py` and ran it with `python3 -I -B`; output is in `checks.tsv`. It reads nothing from the repo. It recomputed:
  - **Field arithmetic.** Shift-and-add GF(256) under 0x11D and 0x11B. Every product on screen: 3·7, 2·128, 128·4, 150·3, 2·142, 7·7, 3·5, 2·5, 0x57·0x83 (49 and 193), and mul(2, 141) and mul(2, 142) under 0x11B.
  - **Polynomials and logs.** 30 irreducible and 16 primitive degree-8 polynomials, and 3·7 = 9 under all 30. Under 0x11B, ord 2 = 51 and ord 3 = 255. Logs 25, 180, 7, 2 and 198; α²⁰⁵ = 167, α⁹ = 58, α¹⁴¹ = 21. The mirror rule holds for all 255 nonzero bytes. The element-order counts are 1, 2, …, 128. The rotation fold rotates all 256 bytes, and 3·255 ≡ 0 under it.
  - **Z/256.** 2A ≡ 6 has exactly the solutions {3, 131}. Odd multipliers are bijections and even ones are not. Exhaustive checks give 3/8 for 2×2 matrices mod 16 and 21/64 for 3×3 matrices mod 4, both equal to ∏(1−2⁻ⁱ).
  - **Rank probabilities, exact `Fraction`.**
    - Ex-or: 21/64 at three pieces; failure 0.711212, 0.422424, 0.229898, 0.119884 and 0.061209 with 0–4 spares, so the step ratios are 1.684, 1.837, 1.918 and 1.959. 0.710930 at K = 10. h = 20 is the first to reach 10⁻⁶. 229,898 phones stuck.
    - Average spares: 1.6057 at K = 10 and 1.60670 in the limit (ex-or); 0.0039369 for byte cards.
    - Byte cards: failure 3.921509×10⁻³, 1.531863×10⁻⁵ and 5.983839×10⁻⁸, against the union bound. Ratios 255.996 and 255.99998.
    - The shape model at K′ = 10 comes to 0.48939% (R5).
- **Hook.**
  - Coupon collector: K·H_K = 97,876.06; the last piece alone takes 10,000 draws and the last ten 29,289.7.
  - Carousel, per phone: done by 2 loops 0.36786, by 3 loops 0.99005; mean 2.64219 loops.
  - Carousel, all 10⁶ phones: done by 5 loops 0.36788, by 6 loops 0.99005.
  - Packets until done: E[sent] = 23,729.95 by my own exact sum; E[caught] = 23,492.65.
- **Density.**
  - K³/3 = 4.1667×10¹³, which is 11.574 h at 10⁹ steps per second.
  - RFC 6330 Table 1, typed from memory, matches the oracle's transcription exactly: mean degree 4.81664, P(2) = 1/2.
  - Untouched pieces: 1 in 123.5 at N = K, 1 in 136 at +2%, 1 in 157 at +5%.
- **Peeling, my own simulation.** 20 runs per overhead, K = 10,000, RFC degrees, a different RNG.
  - At N = K no run completes; 13 of 20 stop below 50% (range 3–72%).
  - At +2%, 3 of 20 collapse; the rest stop at 97.6–99.0%, leaving 101–241 pieces missing.
  - At +5%, every run reaches 98.5–99.2%.
  - This is consistent with every spoken peeling claim. If anything, "usually gets stuck" is weak: with about 81 untouched pieces it essentially always does.
- **Toys.**
  - Ring ranks over GF(256): the ex-or ring has rank 4, and any 4 of its 5 cards have rank 4; with E⊕2A the rank is 5. Card values 6, 4, 5, 14, 9 and 6; S = 9 and 3A = 15 ⇒ A = 5.
  - The all-2 ring has rank 4 (R3).
  - Cards 2, 6, 6, 3, 4, 14, 15; c = 15⊕14⊕6 = 7.
  - Rank of 0/1 matrices is the same over GF(2) and GF(256) (300 random 6×6).
  - "Exactly one amount lands in the lit part" holds on 200 random hyperplanes at K = 3.
- **Statistics.** Clopper–Pearson by bisection on the exact binomial: 0.616–0.647% for 6,316; 1.54–3.57×10⁻⁵ for 24; 2.53×10⁻⁸–5.57×10⁻⁶ for 1. 6,316/256 = 24.7. 646/10⁵ against 6,316/10⁶ gives z = 0.54.
- **Odds and ends.** 2²⁴ = 16,777,216. The primes nearest 256 are 251 and 257. Table 2 sums and P (27, 10,269, 57,326; P = 10 and 158). S and W are prime for K′ = 10, 10,017 and 56,403.
- **Unchecked:**
  - The recorded Monte Carlo counts B-F21 and B-F22: I saw only the provenance re-read.
  - The exact seeded values of B-F18.
  - The Table 2 entries themselves, beyond the sums and primality above.
  - Every primary source (RFC 6330, A/331, ISO 18004, FIPS 197, Anvin, the Erdős scan, SP-050246, RFC 3453/8681): I relied on the fact sheet's quotes.
  - All durations and reading times.

## Claim-walk fixes: did they land?

- **The 6 MISMATCH fixes all landed.**
  - 14_6 says "over a hundred".
  - 01_8 shows ≈ 23,500, and the key equals my exact sum.
  - The 10_7 code panel flips to `mul(2, 141)`.
  - 13_6 shows 1.606.
  - 14_1 says forty-two trillion.
  - 10_4 puts 285 under the whole polynomial.
- **The 4 EDGE claims that failed as spoken were all fixed** (01_5, 06_5, 08_2, 14_8).
- **Loose claims.** 13_5 and 17_5 are fixed. Two are not:
  - 11_7 "every byte's partner" is unchanged (R10).
  - 15_3 is unchanged, and its picture now misleads (R16).
- **Claim-walk notes not applied:** 14_7's N (R6) and 16_4's "join" (R11).
- **The 5 claims with no support now have source rows** (B-S1–B-S5, and the 13_6 race is pinned). The newly worded 03_9, 11_7 and 16_1 are right, apart from R15's card nit.

## Comprehension check

| ch | the sentence I'd say to a friend |
|---|---|
| 01 | Resends swamp the sender when a million phones each randomly lose 1%. A carousel leaves the unluckiest phone waiting 5–6 loops, and random raw pieces need about 98,000 catches for 10,000 pieces, yet a phone with two packets beyond the piece count almost always decodes; the video explains that gap. |
| 02 | Bytes are bit rows, and ex-or flips bits and undoes itself (5⊕7 = 2, 2⊕7 = 5). |
| 03 | A packet is an ex-or equation over pieces; decoding is elimination, and only the number of useful (independent) cards matters, not which ones or their order. RaptorQ is the code under study. |
| 04 | With three pieces the 8 possible cards are a cube's corners; each useful card doubles the lit set, so with two in hand a coin-flip card is useless half the time. |
| 05 | Multiplying the chances that each next card is useful gives 7/8·6/8·4/8; exactly enough random ex-or cards fail 71% at any size, almost all of it in the last few slots. |
| 06 | Each spare only roughly halves the risk (about 20 for one in a million), and a ring of five cards is stuck by structure, so we want byte amounts, which needs a byte multiplication we can undo. |
| 07 | Mod 256, even amounts destroy information (2·128 = 0), and whether the cards pin everything down depends only on the last bits, so wrap-around fails exactly like ex-or. |
| 08 | A prime clock gives every nonzero number a partner, so you can divide; no prime fits in a byte, and a 256-member field is what we need. |
| 09 | Bytes become 0/1 polynomials where 1 + 1 = 0, adding is ex-or, and FOIL gives (x+1)(x²+x+1) = x³+1, so 3·7 = 9. |
| 10 | Products must be folded back into a byte. The rotation fold has zero divisors (3·255 = 0) because x⁸+1 factors, while RaptorQ's irreducible fold x⁸ → x⁴+x³+x²+1 gives a field (2·128 = 29, 2·142 = 1); AES's polynomial labels the same field differently. |
| 11 | Under 0x11D, ×2 visits all 255 nonzero bytes, so multiplying is adding click counts (logs) mod 255, and partners are mirror images. |
| 12 | Sorted by clicks, the times table is constant along anti-diagonals, because a product depends only on the click sum. |
| 13 | With uniform byte amounts a last-slot card is useless 1 time in 256, so each spare buys about a factor of 256 instead of about 2; 0/1 amounts gain nothing from the bigger field. |
| 14 | Dense solving costs about K³/3 steps, so fast codes go sparse and peel. Peeling stalls (and about 1 piece in 124 is in no card), and Reed–Solomon is ideal but capped near 256 packets. |
| 15 | RaptorQ keeps packets ex-or and adds agreed-in-advance equations, with 10–16 byte ones that fill exactly the dangerous last slots; replacing them with ex-or rows takes failure from 0.65% to 71%. *(I got where the free cards' answers come from only from the lower-third; R1.)* |
| 16 | When peeling stalls, park a piece as an unknown, keep peeling and solve for it at the end: elimination in a clever order, exact iff the cards pin everything down. One byte card unlocks the ring (3A = 15 ⇒ A = 5). |
| 17 | A real decoder at ten pieces failed 6,316, 24 and 1 times in 10⁶ with 0, 1 and 2 extra packets. That is consistent with the standard's average-case 10⁻², 10⁻⁴ and 10⁻⁶ (over random packet IDs), and the same field runs ATSC 3.0, Linux RAID-6 and QR codes. |

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight | main reason |
|---|---|---|---|---|---|---|
| 01 | 4 | 4 | 5 | 4 | 3 | R2 |
| 02 | 4 | 5 | 4 | 3 | 3 | primer is slow for me (Maya's call) |
| 03 | 4 | 4 | 5 | 4 | 4 | R15 |
| 04 | 5 | 5 | 4 | 4 | 4 | |
| 05 | 5 | 5 | 5 | 4 | 4 | |
| 06 | 4 | 4 | 5 | 4 | 4 | R14 |
| 07 | 4 | 5 | 4 | 4 | 5 | D4 |
| 08 | 4 | 5 | 4 | 4 | 3 | |
| 09 | 4 | 4 | 4 | 4 | 5 | R13 |
| 10 | 4 | 5 | 5 | 4 | 5 | |
| 11 | 5 | 4 | 4 | 4 | 5 | R10 |
| 12 | 4 | 5 | 4 | 3 | 4 | R19 |
| 13 | 4 | 4 | 5 | 4 | 5 | R7, R14; D1 |
| 14 | 4 | 4 | 4 | 4 | 3 | R6, R17 |
| 15 | 3 | 3 | 5 | 4 | 4 | R1, R16 |
| 16 | 4 | 3 | 4 | 4 | 5 | R3, R11, R12; D3 |
| 17 | 4 | 4 | 4 | 4 | 4 | R2, R8, R9; D2 |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| R1 | ruth_45 | 15_2, 15_3 | major | **What is said:** "both sides agree on extra cards that tie the pieces together … the phone gets them free." **Why it's wrong:** a card's answer depends on the file, and K arbitrary pieces cannot satisfy S + H independent fixed equations. As spoken, these free cards cannot exist. The scheme works only because the sender adds S + H extra unknowns, and every free card's answer is zero (RFC 6330 §5.3.3.4.2; the 15_2 lower-third). **Knock-on:** 15_3's bucket then has more slots than pieces (the 15_2 grid is 27 × 27 for 10 pieces), which the narration never explains. **History:** R1's D3 and RB-26 fixes put the exact version only in a lower-third; the narration contradicts it rather than simplifying it. | **15_2:** "Before sending, the sender adds a few extra pieces, chosen so that some fixed rules hold. Each rule is a card whose answer is zero, so it's never sent, and the phone gets it free. Only ten to sixteen of these rules are byte cards, whatever the block size. In big blocks, hundreds of others are cheap ex-ors." **15_3 picture:** the bucket gains a slot per extra piece (27 at ten pieces). Keep the lower-third. |
| R2 | ruth_45 | 01_2, 17_4, 17_7 | major | **The promise:** "almost every phone that catches just two packets more than it needs" is made under 01_1's model: each packet lost independently at 1%, from a stream that sends the originals first (17_5). **The backing:** RFC 6330 §5.8 averages over IDs drawn uniformly from the whole range, and our runs drew from the first 20,000 IDs. Such a phone holds about 9,900 originals and 102 repair packets, a third distribution that neither covers. At K = 10,000 the padding to K′ = 10,017 also adds 17 known rows at fixed IDs. **So:** the video attaches a probability to a space nobody computed it in. | **Minimum:** add to the description: "The standard's figures, and our runs, are for packets with randomly chosen IDs. A phone that loses random packets from a stream that sends the originals first receives a different mix, which neither covers." **Better (Dev):** run the existing raptorq 2.0.0 harness at K = 1,002, originals first, i.i.d. 1% loss, h = 0, 2×10⁴ trials. Compare with the recorded 0.54%, and pin the result as a B-F22 row. |
| R3 | ruth_45 | 16_5 lower-third (my own R1 RB-D4 wording) | major | **What it says:** "A loop of two-piece cards is stuck exactly when its amounts multiply to 1 going around." **Why it's wrong:** read literally, it is false. The ring 2A+2B, 2B+2C, …, 2E+2A has rank 4 over GF(256) (computed), so it is stuck, yet its ten amounts multiply to 2¹⁰ = 116. **The true rule** is about ratios, and it is what the oracle key `B-F9.cycle_rule_stuck_iff_prod_b_over_a_is_1_ok` tests. **Also:** "loop" is reserved for the carousel. | "A ring of cards a·X + b·Y is stuck exactly when the ratios b/a multiply to 1 going around; with ex-or every ratio is 1." |
| R4 | ruth_45 | notes at 01 (RB-07), 15 (D9/RB-30), 17 (RB-29/D16); plan §5.1 rule 11 | major | **The gap:** several exact versions, and R2's, are routed to "the description", but no description draft exists; `publish/youtube.md` is made only at publish time. The fact sheet also has no *simplification → exact* ledger, which rule 11 requires. **Effect:** the R2 rule ("every persona note reached …") cannot be checked for these notes. As of now they have reached nothing. | **Storyboard:** add a "Description (draft)" section with the exact sentences, so the R3 packet carries it: the loss model (RB-07), the decoder with K, trials and ID pool (RB-29), R2's sentence, and R5 if it is kept. **Fact sheet:** add a "Simplifications → exact" table, one row per simplification: beat id, exact text, and home (lower-third or description). |
| R5 | ruth_45 | 15 note (D9/RB-30) | minor | The shape model "≈ 0.49% (0.098% + 0.392%)" is meant for the description but has no oracle key. It is a closed form, not Monte Carlo. I recomputed 1−∏ᵢ₌₁₁²⁷(1−2⁻ⁱ) = 0.09762% and 1−∏ᵢ₌₁¹⁰(1−256⁻ⁱ) = 0.39215%, combined 0.48939%, so the figure is right. The receipts card promises that every number was computed by code. | Add `B-F21.shape_model_K10_fail` (with the two factors as subkeys) before the description uses it, or leave it out. |
| R6 | ruth_45 | 14_7 | minor | "About one piece in a hundred and twenty isn't in any card" holds at N = K (1 in 123.5). It follows "even with two percent extra", where it is 1 in 136. The sentence doesn't say which N. | "With exactly enough cards, about one piece in a hundred and twenty isn't in any card." |
| R7 | ruth_45 | 13_6 screen and lower-third | minor | "exact: 1.606" (ten pieces) and "… = 1.6067…" (the limit) appear together, both as "the average", and the second is unlabelled. | Lower-third: "with many pieces: 1 + 1/3 + 1/7 + 1/15 + … = 1.6067…; with ten pieces the sum stops at 1/1023: 1.6057. Erdős proved the full sum irrational in 1948." |
| R8 | ruth_45 | 17_3 | minor | "Too rare to measure precisely" is correct, but its exact version lives only in a storyboard note. One failure in 10⁶ gives a 95% Clopper–Pearson interval of 2.53×10⁻⁸ to 5.57×10⁻⁶ (recomputed), which cannot confirm 10⁻⁶. | Beside the measured "1": "95% interval: 2.5×10⁻⁸ – 5.6×10⁻⁶". Also put it in the description. |
| R9 | ruth_45 | 15_5; 17_1 caption; 05_2, 06_2–06_3 | minor | **Rule 25 gaps:** 15_5's counters have no trial counts (10⁴ and 10⁵). The 17_1 caption's 0.63% and 0.54% have none either (2×10⁵ and 2×10⁴, both partial runs). The ex-or ladders lack the word "model", though 13_5 has it. **Two harnesses:** viewers get two "six in a thousand" figures, 0.646% at 15_5 and 0.632% at 17_1, from different harnesses. They agree (z = 0.54), but nothing explains the difference. | **15_5:** "10 pieces · 100,000 / 10,000 trials · our RFC 6330 implementation". **17_1 caption:** add "(200,000 / 20,000 trials)". **05_2, 06:** add "model". |
| R10 | ruth_45 | 11_7 | minor | "Every byte's partner sits at its mirror image": zero has no partner. 10_6 and 17_7 correctly say "nonzero". | "every nonzero byte's partner …" |
| R11 | ruth_45 | 16_4 | minor | "Let one byte card join it" adds a sixth card, but the picture and B-F9 replace E⊕A with E⊕2A. | "Let a byte card take the fifth card's place, E plus two times A is six." |
| R12 | ruth_45 | 16_1 | minor | "Carry it along like x in algebra": since 09_2, x has been the polynomial variable, i.e. the byte 2. Here the same symbol means an unknown. | "carry it along like an unknown in algebra" |
| R13 | ruth_45 | 09_1 | minor | "Something like a prime, with two hundred fifty-six members" gives the members to the prime. The modulus has no members; the number system does. | "We need a clock that works like a prime one, but with two hundred fifty-six hours." |
| R14 | ruth_45 | 06_5, 13_6 | minor | To a probabilist "almost never" means probability 0. Here it means 1/256 and 0.39%. | 06_5: "it would hardly ever be" (keeps 13_4's number a surprise). 13_6: "Byte cards rarely need even one." |
| R15 | ruth_45 | 03_9 name card | minor | In IETF usage "Internet Standard" is the top maturity level. RFC 6330 is a Proposed Standard (B-S1). The spoken phrase is fine colloquially; the card should be exact. | Card: "RFC 6330 · IETF Proposed Standard · August 2011". |
| R16 | ruth_45 | 15_3 picture | minor | "A thin gold layer seals the top": at the ten-piece block of 15_2 and 15_5, the gold is 10 of 27 slots (37%). It is thin only in big blocks (11 of 10,269). This is RB-01 coming back through the picture. | Label the bucket's block size: either the big block drawn schematically, or 10 of 27 slots gold at ten pieces. |
| R17 | ruth_45 | 14 notes (Ruth) | minor | "The standard's degree table is the ideal soliton 1/(d(d−1)), d = 2…29, quantized to 2⁻²⁰, mean 4.817" has no home and no drop reason, which breaks the R2 rule. I verified it: mean 4.81664; max relative deviation 5.0×10⁻⁴. | Lower-third on the 14_2 histogram: "RFC 6330's degrees: P(d) ≈ 1/(d(d−1)) for d = 2…29 (the ideal soliton); mean 4.82". Or drop it with a reason in R2.md. |
| R18 | ruth_45 | 13_4, 13_7, 14_8 | minor | **The gap:** three always/never claims have no test named in the fact sheet. **13_4** "exactly one choice of amount lands in the lit part": the claim walk ran K = 2 and I ran K = 3, but there is no key. **13_7:** one example matrix, not the general fact. **14_8** "any set of exactly enough packets": MDS, cited from the memo only. | Add `_ok` keys: one-amount-lit, exhaustive at K = 2; rank of 0/1 matrices equal over GF(2) and GF(256) on random matrices. Add a sourced MDS row. |
| R19 | ruth_45 | 12 notes | minor | Four lower-thirds (RB-19, RB-20, D12, RB-D5, about 85 words) hang under a 40 s chapter with no beat ids; R1.md places them at 11. An exact version that flashes for two seconds is not "findable". | Give each lower-third a beat id and at least (words ÷ 3) s on screen. Mirror every lower-third's text in the description. |
| R20 | ruth_45 | 17_7 names-to-search | minor | Nobody is credited. I'd like the history: Luby's LT codes (FOCS 2002), Shokrollahi's Raptor codes (IEEE Trans. Inf. Theory, 2006), and RFC 6330's authors. | Add them to the names-to-search lower-third after checking each against the RFC header and the papers. *I have not checked these.* |
| D1 | ruth_45 | 13_4 | delight | "Scaling by a nonzero byte reshuffles all 256 values, so exactly one choice of amount lands in the lit part" is an actual proof in one sentence. I checked it on 200 random hyperplanes at K = 3. | Keep it. Optional precise lower-third: "fix the other amounts; for a piece whose own card isn't lit, exactly one of 256 amounts is". |
| D2 | ruth_45 | 17_2–17_3 | delight | The model is tested on screen: 6,316 / 256 = 24.7 is predicted, and 24 is measured. | Keep the prediction pause. |
| D3 | ruth_45 | 16_5 | delight | A ⊕ 2A = 3A because 1 ⊕ 2 = 3; the same A–D as chapter 03; "three times five is fifteen here too, but three times seven was nine". | Keep it word for word. |
| D4 | ruth_45 | 07_6 lower-third | delight | Wrap-around fails *exactly* as often as ex-or, at every K. I re-confirmed it exhaustively: 3/8 for 2×2 mod 16 and 21/64 for 3×3 mod 4. | Keep it. |
| D5 | ruth_45 | 13_5 | delight | One byte spare drops eight rungs of the same ladder (log₂ 256 = 8): two arithmetics on one scale. | Keep it. |

## Mathematician audit (`ruth_audit.md`)

| item | result |
|---|---|
| Quantifiers explicit | R6, R10, R14 (and R2). Otherwise pass: "on average" at 01_8 and 13_6, "almost all" at 05_3, "most runs" at 14_6 (17 of 20 in my runs), "averaged over random packets" at 17_4. |
| Always/never tested at the edges, test named in the fact sheet | R18. Pass, each tested: 02_6, 07_5, 08_2, 10_6, 11_2, 11_7 (but R10), 12_4, 15_2 (H ∈ [10, 16] over 477 K′), 16_3, 17_4. |
| Every probability names its space | R2. Pass: 04_4 (coin flips), 13_x (uniform amounts, zero card included), 17_4 (random IDs), 01_1 (random loss), 17_1 (decoder, K, trials). |
| No probability on a fixed unknown; CI as coverage | Pass. The fact sheet's B-F23 reads the 95% upper bound correctly. On-screen home: R8. |
| Terms in their standard sense | Pass: field with its laws (08_4); "prime polynomial" means irreducible (correct in GF(2)[x]); irreducible kept apart from primitive (12 lower-third, AES's hand is 3). Note: "ring" names the five-card cycle and is never said in the algebraic sense, so there is no clash in the words; but see R3's "loop". |
| Every simplification has its exact version recorded | R1, R4, R8, R17, R19. |
| Historical claims verified | Pass on 2011, 2005, 1948, AES 0x11B, QR 0x11D (2006 edition only), Linux RAID-6, CRC-8/SAE-J1850, fountain etymology (all from fact-sheet quotes; I did not reopen them). R15; R20 is a wish, not a failure. |
| Numbers equal the oracle at shown precision, voice = screen | Pass for every number in "What I checked". R7 is a labelling issue, not a value error. |
| Edge cases | R10 (zero), R6 (which N). Pass: log 0 (zero at the centre, zero check in code), K vs K′ (17_4 lower-third), the 2²⁴ cap (14_8). |
| Notation consistent | R12 (x), R3 ("loop"). Pass: byte 3 vs count 3 (RB-03 lower-third); 2x² never written (RB-04). |
| Nothing true-but-misleading | R1, R2, R3. |
| Every persona note reached script, card, lower-third or description | R4 (all description-routed notes), R8 (17's interval), R17 (14's soliton note). Every other note checked landed: 02, 03, 05, 06, 07/08, 09, 10, 13, 16; 15's D9 and 16's D5 are deferred to bead mx-e-b-qyy.10 with a reason. |

My scratch files are in `<scratch>/r2_ruth_b/`: `oracle.tsv`, `checks.py` and `checks.tsv`.
