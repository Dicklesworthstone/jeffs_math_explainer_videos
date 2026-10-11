# R3 (narrated draft) review: raptorq_gf256, by ruth_45

Reviewer: Prof. Ruth Adler, 45, mathematician (persona `tools/review_prompts/ruth_45.md`; `ruth_audit.md` applied).

**Verdict.** I found no blockers and no majors. The mathematics is sound: field axioms, Z/256 vs GF(2⁸),
irreducibility, the 0x11D fold, discrete logs, GF(2) vs GF(256) ranks, and RFC 6330 §5.8's average-case requirement are
all stated correctly, and every number I recomputed matches the oracle. There are 13 minor findings. Two of them change
spoken lines (15_2, 15_3). The rest are on-screen labels, one terminology gap (irreducible/primitive), and two notes for
the description.

## How I checked

- **Picture.** I opened all 17 cue sheets with their legends, chapter by chapter, against the script. Each tile is
  480×270, so I also pulled 960×540 frames at several points in each line from
  `renders/raptorq_gf256_narrated_watch.mp4`. I aligned it to the sheets by image matching, and the residuals were low
  for chapters 01–09 and 11–16. I also swept the whole file at 2 fps to find every distinct lower-third panel (43
  found). **Caveat:** the watch file (14:42) is older than the sheets (14:56). Its chapter 17 is still the storyboard
  placeholder, and chapter 10's timing differs. So for chapter 17 I had only the seven sheet tiles. I read the strings
  shown in 17_4–17_7 from `src/chapters/two_packets.rs` (read only). Those are the 17_4 lower-third "RFC 6330 §5.8:
  averaged over randomly chosen packet IDs, for each padded block size K′", the 17_6 cards and credits, the 17_7 names
  list and the receipts card. I did not see them in a frame.
- **Recomputed** with `python3 -I`, standard library only, using exact `Fraction`s where it mattered:
  - **Byte-field products:**
    - 3·7 = 9, 2·128 = 29, 128·4 = 58, 150·3 = 167, 2·142 = 1, 3·5 = 15 under 0x11D.
    - 87·131 = 49 under 0x11D and 193 under 0x11B; 2·141 = 1 under 0x11B.
    - The carry-less product clmul(3, 255) = 0x101.
  - **Logs and orders:** ord 2 = 255 under 0x11D and 51 under 0x11B. log 3 = 25, log 150 = 180, log 128 = 7,
    log 7 = 198. α²⁰⁵ = 167, α¹⁴¹ = 21, α⁹ = 58. OCT_EXP[8..12] = 29, 58, 116, 232, 205.
  - **Z/256:** 2·131 ≡ 6 and 3·171 = 513.
  - **Ex-or lottery:** 21/64 at K = 3. Failure 0.710930 at K = 10. In the limit with 0, 1, 2, 3 spares: 0.711212,
    0.422424, 0.229898, 0.119884. The ratios between spares are 1.684, 1.837, 1.918, 1.959, 1.979. Failure is
    9.537×10⁻⁷ at 20 spares and 1.907×10⁻⁶ at 19.
  - **Byte lottery:** 3.921509×10⁻³, 1.531863×10⁻⁵, 5.983839×10⁻⁸; the union bound 1/255 = 0.39216%.
  - **Mean spares:** 1.6057 at K = 10, 1.60670 in the limit, 0.0039369 for byte cards.
  - **Hook arithmetic:** K·H_K = 97,876.06. The carousel probabilities are 0.368 and 0.990 per phone, and 0.3679 and
    0.9900 for all 10⁶ phones.
  - **Dense cost and coverage:** K³/3 = 4.1667×10¹³, which is 11.57 h at 10⁹ steps per second. 73.42 pieces are in
    no card at 2% extra.
  - **Chapter 17:** 6,316/24 = 263. 6,316/256² = 0.0964, and P(≥ 1) = 9.19%.
  - **Running example:** the ring cards 6, 4, 5, 14, 6 ex-or to 15, and 15/3 = 5. The ring has rank 4, and rank 5 with
    E ⊕ 2A. Chapter 14's stuck system, without the C card, still has rank 5. 13_1's byte card evaluates to 244 on
    A…E = 5, 3, 7, 2, 12.
  - **mod 7:** the times table, and 3⁻¹ = 5.
  - **Code panels:** checked by hand. Rust parses `a & 0x80 != 0` as `(a & 0x80) != 0`, unlike C. The panels index
    EXP[0..508] with u16 logs. There are 32,131 pairs with LOG[a] + LOG[b] ≥ 256, counted from i + j over [0, 254]².
- **Unchecked** (keyed or recorded in the oracle, not rerun by me):
  - The recorded decoder and ablation counts: 6,316/24/1; 646/10⁵; 7,066/10⁴; 1,262/2×10⁵; 108/2×10⁴.
  - The 07_6 race: 1,404 and 1,388 of 2,000 (B-F25.race07).
  - The 13_6 race counts. I checked only the exact means.
  - The 14_6 seeded peeling percentages.
  - The 15_2 "85 LT symbols". I checked only the order of magnitude: 10,111·e^(−4.77) ≈ 86.
  - The external sources (RFC, ATSC, ISO/IEC 18004, FIPS 197, Linux). I relied on the fact sheet's quotes.
- **Audit (`ruth_audit.md`):**
  - Quantifiers: pass, except 15_2 and 15_3 (R3-01, R3-02).
  - Each probability says what is random: pass. "Model" labels are on 05, 06 and 13. 17_4 says "averaged over randomly
    chosen packet IDs".
  - Field vs ring: pass (07, 08_4).
  - Irreducible vs primitive: not named on screen (R3-03).
  - Estimator vs estimate: 13_6 is exemplary, and 01_8 is borderline (R3-04).
  - Numbers vs oracle at the shown precision: every one I checked passes. Voice and screen round alike, for example
    70.66% → "seventy-one" and "70.7%", and 73.4 → "about seventy" and "≈ 70".
  - Edge cases: K vs K′ (R3-07), log of 0 (11_7 "0: none"), zero divisors (10_3).
  - The exact versions are in the description, except 03_8's 2²⁴ cap, which lost its home when 14_8 was cut (R3-08).

## Comprehension check

| ch | what I would tell a friend |
|---|---|
| 01 | A million phones each losing ~1% of packets makes asking again (feedback implosion), looping (most phones finish in loop 2 or 3, the unluckiest of a million in 5 or 6) and random pieces (~97,876 draws by coupon collecting) all poor, while a phone of a fountain code is almost always done at 10,002 packets, and the claimed secret is an arithmetic where 3 × 7 = 9. |
| 02 | Numbers are bit strings, and ex-or flips the bits of one number where the other has a 1 (5 ⊕ 7 = 2); flipping twice undoes it (2 ⊕ 7 = 5). |
| 03 | Each packet is a linear equation over GF(2) in the pieces, decoding is elimination, and the phone needs as many linearly independent equations as there are pieces, in any order; RaptorQ (RFC 6330, Proposed Standard, 2011) is such a fountain code. |
| 04 | With three pieces the possible cards are the 8 vertices of a cube (GF(2)³), j independent cards span 2ʲ of them, and with two in hand a uniformly random card is useless with probability 4/8. |
| 05 | K uniform 0/1 cards are all independent with probability ∏(1 − 2⁻ⁱ) (21/64 at K = 3, 0.2888 in the limit), so the "ex-or lottery" fails ~71% at every size, almost all of it in the last few slots. |
| 06 | Each spare roughly halves the failure (2 spares: 23%, 20 for one in a million), structured sets like the 5-card ring have rank 4 whatever happens, and both problems call for 256 possible amounts with arithmetic that can be undone. |
| 07 | Wrapping at 256 fails: 2·128 = 0, even multipliers lose information, and a matrix over Z/256 is invertible iff it is invertible mod 2, so random byte amounts fail exactly as often as ex-or. |
| 08 | Mod a prime every nonzero number has an inverse (mod 7, 3⁻¹ = 5), so Z/p is a field, but 257 does not fit in a byte; we need a field of exactly 256 elements, which exists because 256 is a prime power. |
| 09 | Read a byte as a polynomial over GF(2): adding is ex-or, multiplying is carry-less, and (x + 1)(x² + x + 1) = x³ + 1, so 3 × 7 = 9. |
| 10 | Products past x⁷ must be folded: the reducible x⁸ + 1 (a rotation) gives zero divisors (3·255 = 0), while RaptorQ's irreducible x⁸ + x⁴ + x³ + x² + 1 (0x11D) gives a field, with 2·128 = 29 and 2·142 = 1. |
| 11 | Under 0x11D the byte 2 has order 255, so each nonzero byte is a power of 2 and its exponent (the click count, a discrete log) turns multiplying into adding mod 255; a byte's inverse sits at minus its clicks. |
| 12 | RFC 6330 multiplies with log/antilog tables; sorted by log, the times table is constant along anti-diagonals; AES's different irreducible polynomial gives an isomorphic field with different labels, so both ends must agree. |
| 13 | With uniform GF(256) amounts a card at the last slot is useless with probability exactly 1/256 (vs 1/2), so each spare divides failure by ~256 (0.39%, 1.5×10⁻⁵, 6×10⁻⁸), while 0/1 cards gain nothing from the bigger field. |
| 14 | Dense byte cards cost ~K³/3 steps (half a day at 50,000 pieces), sparse cards decode by peeling, but peeling stalls, and even with 2% extra ~73 of 10,000 pieces are in no card at all. |
| 15 | RaptorQ sends only sparse ex-or packets over the pieces plus S + H extra unknowns tied by S ex-or and H = 10–16 byte equations with right side zero; the ex-or rows almost always fill all but the last H slots, the byte rows fill those at ≈1/256 risk each, and swapping them for ex-or rows takes failure at ten pieces from 0.646% to 70.66%. |
| 16 | Inactivation decoding peels until stuck, carries a stuck unknown symbolically, and solves the parked ones at the end (Gaussian elimination in a clever order, success iff rank L); one byte amount turns the ring's A ⊕ A = 0 into 3A = 15, so A = 5. |
| 17 | A real decoder at ten pieces failed 6,316, 24 and 1 times per million with 0, 1, 2 extra packets, inside RFC 6330 §5.8's average-case ceilings 10⁻², 10⁻⁴, 10⁻⁶ (the last too rare to confirm); the code is systematic, optional FEC in ATSC 3.0, and its byte field is Linux RAID-6's and QR codes'. |

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight | note |
|---|---|---|---|---|---|---|
| 01 | 4 | 4 | 5 | 4 | 4 | 01_8 counter (R3-04); hook's packet mix unrun (R3-09) |
| 02 | 5 | 4 | 4 | 3 | 3 | slow for me (Maya's call); 02_6 label (R3-11) |
| 03 | 5 | 4 | 5 | 4 | 4 | "endless" lost its 2²⁴ (R3-08) |
| 04 | 5 | 5 | 4 | 4 | 4 | |
| 05 | 5 | 5 | 5 | 4 | 4 | |
| 06 | 4 | 5 | 5 | 4 | 4 | |
| 07 | 4 | 5 | 4 | 4 | 4 | |
| 08 | 5 | 5 | 5 | 4 | 3 | |
| 09 | 5 | 5 | 5 | 4 | 5 | |
| 10 | 4 | 4 | 5 | 4 | 4 | "can't be factored" over what? (R3-03) |
| 11 | 5 | 4 | 4 | 4 | 5 | "log₂ 150 = 180" (R3-10) |
| 12 | 4 | 5 | 4 | 4 | 5 | |
| 13 | 5 | 5 | 5 | 4 | 5 | |
| 14 | 4 | 4 | 5 | 4 | 4 | 14_6 label (R3-05) |
| 15 | 3 | 3 | 5 | 3 | 4 | R3-01, R3-02, R3-06, R3-07; densest chapter |
| 16 | 4 | 5 | 4 | 4 | 5 | |
| 17 | 4 | 4 | 5 | 4 | 4 | names list (R3-03); R3-09 |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| R3-01 | ruth_45 | 15_2 | minor | "And every piece is in some free card, **so** even a piece that no packet mixed **can still be worked out**." Being in some equation is necessary, not sufficient. Coming right after 14_7's "No decoder can find those", "can still be worked out" will be heard as "will be". Coverage only makes the piece's column of the matrix nonzero. Recovery needs rank L (16_3's own lower-third), and that fails 0.646% of the time at ten pieces (15_5). The description's verb is the right one: the free cards "reach" those pieces. | Say: "And every piece is in some free card, so no piece is out of reach: even one that no packet mixed sits in an equation that can pin it down." |
| R3-02 | ruth_45 | 15_3 | minor | "Count the ex-or cards first, and they fill it up to the last ten to sixteen slots, before the danger starts." The count is exact: L − H = K′ + S ex-or rows. The rank is not. In the uniform model the ex-or rows fall short of L − H with probability 0.0976% at ten pieces (B-F21 shape model). RaptorQ's sparse, structured rows fall short more often: the shape model gives 0.489%, the measurement 0.646%. As spoken, the model's typical case is stated as a certainty. | Insert "almost always": "…and they almost always fill it up to the last ten to sixteen slots…". Description 15_3: add "in the uniform model the ex-or rows fall short with probability 0.098% at ten pieces; RaptorQ's sparse rows do so somewhat more often." |
| R3-03 | ruth_45 | 10_4, 10_6 and 11_2 lower-thirds, 17_7 names | minor | **The gap:** irreducible vs primitive never reaches the screen. The fact sheet's Simplifications row for 10_4 says the exact version ("irreducible over GF(2); 0x11D is also primitive, 0x11B is not") lives in the 10 and 11_2 lower-thirds. As rendered, they say only "the polynomial can't be factored, so no two nonzero bytes multiply to 0…" and "under AES's polynomial, times two comes back to 1 after 51 steps (its hand is 3)". **"Can't be factored"** also needs its ring: over GF(256) this polynomial splits into eight linear factors (x − α^(2^i)). **The names list:** 17_7 offers "primitive polynomial" but not "irreducible polynomial". A viewer who searches will find the stronger property under a name that sounds like the video's "prime polynomial". | **10_6 lower-third:** "irreducible over GF(2) (no factors with 0/1 coefficients), so no two nonzero bytes multiply to 0; …". **11_2 lower-third:** "0x11D is primitive: 2 reaches all 255. AES's 0x11B is irreducible but not primitive: times two returns after 51 steps (its hand is 3); every finite field has such a clock." **17_7 names:** add "irreducible polynomial" (and "discrete logarithm", R3-10). Otherwise, correct the fact sheet's "home" column. |
| R3-04 | ruth_45 | 01_8 (screen) | minor | The counter is titled "random pieces drawn". While it runs (40,390 … 51,637 …) it reads as one run of the sticker album. It stops at exactly 97,876 with the album full, and only then does "on average" appear. The code animates the expected curve, not a run, so the label should say so from the first frame. A single run lands on its mean with probability ≈ 0, since its SD is ≈ K·π/√6 ≈ 12,800. Compare 13_6, which shows a seeded run (1.603) beside the exact mean (1.6057): that is the pattern to follow. | Show "on average" from the moment the counter starts ("random pieces drawn, on average"). Or let a seeded run stop at its own count and put "on average: 97,876" beside it. |
| R3-05 | ruth_45 | 14_6 (screen) | minor | "116 to 176 pieces still missing" stands over all five 2%-extra bars, including the run that collapsed at 15.8%, which has about 8,400 pieces missing. The label is false for that bar. The narration is careful ("most runs"); the label should be too. | "4 of 5 runs: 116 to 176 pieces still missing; one stalled at 15.8%". |
| R3-06 | ruth_45 | 15_5 (screen) | minor | The percentage counts up while the "x of N failed" line is already final. The cue tile shows "0.0% fail / 7,066 of 10,000 failed", and 4 s later the screen reads "51.2% fail / 7,066 of 10,000 failed" (the byte side reads "0.468% / 646 of 100,000"). For two seconds the screen contradicts itself. 07_6 counts both together and is fine. | Count the numerator with the percentage, or fade in "x of N failed" when the counter lands. |
| R3-07 | ruth_45 | 15_2 lower-third; description 15_2 | minor | "…the sender solves for them all so the first K′ packets are the originals." There are K originals. ISIs K … K′ − 1 are padding pieces: known zeros, never sent. At ten pieces K = K′, so the picture is right, but the "precisely" box is not exact at 10,000 pieces (K′ = 10,017). | "…so the first K packets are the originals (the K′ − K padding pieces are known zeros and never sent)". |
| R3-08 | ruth_45 | 03_8 (voice and screen "fountain codes: endless fresh mixes") | minor | At R1 (my RB-25), "endless" got its exact version: RaptorQ's 24-bit ESI, 2²⁴ = 16,777,216 per block. That lived in the 14_8 lower-third. 14_8 has left the video, so the cap now appears only in the description's 14_8 entry, under a beat the viewer never sees. | Extend the 03_8 lower-third with "(RaptorQ: up to 2²⁴ ≈ 16.8 million per block)", or move the 2²⁴ sentence into the description's 03_8 entry. |
| R3-09 | ruth_45 | 01_2, 17_4, 17_7 (carried from R2 R2) | minor | The minimum is met: the description's "What the promise averages over" is honest. But the headline claim still rests on a packet mix nobody has computed: originals first, i.i.d. 1% loss, then repair. The fact sheet still lists that run as not done (deferred to mx-e-b-qyy.10). I expect it to hold, but I mark it "unchecked", not "fine". | Before final, run the existing `rq_mc.py` harness at K = 1,002 with the hook's loss model (on the build host). If that cannot happen, add "the hook's originals-first stream: not measured" to the 17_4 lower-third. |
| R3-10 | ruth_45 | 11_6 (screen "log₂ 150 = 180") | minor | log₂ normally means the real logarithm (log₂ 150 ≈ 7.23). Here it is a discrete logarithm in the byte field. The narration's "That click count is called a logarithm" is fine for Maya, but the formula on screen collides with what every viewer learned at school. | Write "log 150 = 180 (base 2, in the byte field)", or "clicks(150) = 180". Add "discrete logarithm" to 17_7's names to search. |
| R3-11 | ruth_45 | 02_6 (screen) | minor | For about 1.5 s after "Zero, one, zero. Two." the row is still labelled "5" while it shows 0 1 0 (cue tile 02_6). It is relabelled 2 by ~54 s. A paused frame shows "5 = 010". | Relabel 5 → 2 on the same beat as the last flip. |
| R3-12 | ruth_45 | 07_6 "for programmers" lower-third | minor | "…which is why hash" runs into the panel's right border at 960×540, so the last word touches or crosses the frame of the box. | Rewrap, or shorten to "…can be undone; that's why hash mixers multiply by odd numbers". |
| R3-13 | ruth_45 | description 14_8 | minor | "Over GF(256), N is at most 257 (q + 1, with the point at infinity); checked for K = 3 on 3,000 random triples of the 257 columns." This is right for Reed–Solomon codes. But the "checked for K = 3" clause invites reading 257 as a bound for any code where every K of N packets suffice, and at K = 3 that is false. Over GF(2⁸) the conic {(1, t, t²)} ∪ {(0, 0, 1)}, plus its nucleus (0, 1, 0), is a hyperoval: 258 columns, any 3 independent. By hand: two conic points with the nucleus give det = (a − b)² ≠ 0, and one conic point with (0,0,1) and (0,1,0) gives ±1. The 3,000 triples test that the 257 columns work, not that 258 cannot. | "A Reed–Solomon code over GF(256) has at most 257 packets (q + 1); the 3,000 triples check that all 257 work. (At K = 3 a non-RS code does one better: a hyperoval gives 258.)" That would make a nice footnote for mathematicians. |
| R3-14 | ruth_45 | 16_5 | delight | "3 × 5 = 15 here too, but 3 × 7 = 9" lands exactly when a viewer would wonder whether byte arithmetic is secretly ordinary arithmetic. The ring lower-third now states the true ratio rule. I re-checked it: rank 4 for the plain ring, rank 5 with E ⊕ 2A, and 6 ⊕ 4 ⊕ 5 ⊕ 14 ⊕ 6 = 15 = 3·5. | Keep. |
| R3-15 | ruth_45 | 12_4 lower-third | delight | The element orders 1, 3, …, 255 occur φ(d) = 1, 2, 4, …, 128 times, and 255 = (2+1)(2²+1)(2⁴+1) explains why: 255 is a product of Fermat primes, so every φ(d) is a power of 2. Together the counts spell 11111111₂. That is the plan's old false claim turned into a true and better one. | Keep. |
| R3-16 | ruth_45 | 13_6 | delight | A seeded race (mean 1.603) beside the exact sum 1.6057 at ten pieces, the limit 1.6067…, and Erdős's 1948 irrationality proof. This is how to put a sample next to its expectation. | Keep, and use it as the model for R3-04. |
| R3-17 | ruth_45 | 03, 13_1, 14, 16 | delight | One running example, consistent everywhere: A, B, C, D, E = 5, 3, 7, 2, 12 satisfy every card in 03, 14 and 16. Even 13_1's random-looking byte card works out (215A ⊕ 109B ⊕ 128C ⊕ 211D ⊕ 83E = 244; I recomputed it). The stuck Sudoku of 14_5 has rank 5, so parking c in 16_2 genuinely solves it. | Keep. |
| R3-18 | ruth_45 | 07_6, 10 (for programmers), 09_3 | delight | The asides for programmers are true and earn their place: 3·171 = 513 = 2·256 + 1 and odd multipliers; CRC-8/SAE-J1850's 0x1D; and the callback 5 ⊕ 7 = 2 rewritten as (x² + 1) + (x² + x + 1) = x. | Keep. |
