> 2026-10-09. All numbers in this memo were verified by the scripts in `scripts/` (see `scripts/README.md`).

# RaptorQ's finite-field math: research memo for a 3B1B-style explainer

Three labels run through this memo:
- **[V]** marks a verified fact. Each one comes from a primary source or from my own computation; the method is given.
- **[O]** marks the owner's framing, from his article, his X post and the two AI chats he linked.
- **[P]** marks my creative proposals.

My verification scripts are in `docs/research/raptorq_gf256/scripts/`.

---

## 0. What I could and could not retrieve

| Source | Status |
|---|---|
| Article (jeffreyemanuel.com/writing/raptorq) | **Full text retrieved** with curl and an HTML-to-text converter, since WebFetch only returned a summary. The interactive widgets (toy decode, ripple viz, precode viz) render client-side as "Loading visualization", so I did not get them. |
| X post 2063674211558904278 | **Full text retrieved** via `api.fxtwitter.com`. The attached photo is a phone screenshot of `asupersync/src/raptorq/gf256.rs` (header comment: "GF(256) … irreducible polynomial x^8 + x^4 + x^3 + x^2 + 1 (0x1D…)", log/exp tables, AVX2/NEON kernel dispatch). |
| Claude share (claude.ai/share/9e196534…) | Direct fetch, the snapshot API and WebFetch were all blocked by Cloudflare or got an empty shell. **Full text retrieved** through the `r.jina.ai` reader proxy, except one widget ("Interactive visual hidden when shared"). |
| ChatGPT share (chatgpt.com/share/6a24a163…) | **Full final answer retrieved** by parsing the page's embedded payload, plus the first terse answer and some reasoning snippets. |
| RFC 6330 | Full text. Quotes are below. |
| Qualcomm "RaptorQ Technical Overview" (2010) | Full PDF text. |
| Shokrollahi, "Raptor Codes", IEEE Trans. IT 52(6), 2006 | Full PDF text. |
| Shokrollahi & Luby monograph (FnT Comm. Inf. Theory 6(3–4), 2011) | **Bibliographic record only** (paywalled). I did not read it. Its claims are covered here through the RFC, the Qualcomm overview and my own simulations. |
| MacKay, "Fountain codes" (2005) and Luby, "LT codes" (FOCS 2002) | Full text blocked (403 / paywall). Claims attributed to them are cross-checked against secondary sources or my own math. |

---

## 1. [O] The owner's framing

**Article: "The Black Magic of Liquid Data"**
- The core image is a receiver that is "basically just a bucket" for a "spray" of packets.
- TCP is a conversation, and conversations break down at Mars latency or with a million receivers. The article calls this "feedback implosion".
- "Packets Are Equations". The worked example uses A=5, B=3, C=7, D=2 with packets A⊕C=2, B⊕C⊕D=6, A⊕B=6, A⊕B⊕C⊕D=3.
- Three promises: rateless, systematic, near-MDS.
- "The Coupon Collector's Tax": with d=5, about e^-5 ≈ 0.7% of symbols are never touched.
- LT codes, the ideal and robust soliton, the "ripple", the 2-core.
- "Don't make the fountain code perfect": let peeling get to about 97% and let a precode clean up the rest.
- The three constraint layers, a toy decode, peeling as Sudoku, and inactivation in four phases.
- Engineering tricks: systematic encoding, one-integer ESI, padding to K', degree table with mean about 4.8, permanent inactivation of about √K symbols.
- "Why +2 Packets Changes Everything", using ∏(1−q^-j).
- A history timeline and a Shamir secret-sharing aside.

**X post.** He calls RaptorQ "a free lunch that shouldn't exist". He asked two AI models (GPT-5.5 and Opus 4.8) "What's the relationship between RaptorQ and the Galois field GF(2^8) with x^8+x^4+x^3+x^2+1?" and then asked them to unpack the WHY. He prefers the Claude answer as "more human-centric".

**Claude answer.** Its spine is a "negotiated settlement" between cheap GF(2) and well-conditioned GF(256). Its key points:
- The random-matrix product: 0.2887 vs 0.9961.
- Irreducible vs primitive: 30 vs 16 polynomials.
- Under AES's 0x11B, 2 has order 51.
- OCT_EXP[8] = 29 as a "fingerprint" of the polynomial.
- Log/antilog as a "discrete-log slide rule".
- The three-band matrix A, the MT·Γ construction of the HDPC rows, and "each dense row ≈ 8 bits of dependency-killing power".
- **I verified every numeric claim in it (see §2).**

**GPT answer.** It is more mechanical and covers the same ground:
- Why a field is needed (no inverse of 2 mod 256).
- Bytes as polynomials, the 0x80·0x02 = 0x1D "128 → 29 moment", and the xtime code.
- A worked 0x57·0x83 = 0x31.
- Symbols as byte vectors, and AES's 0x1B vs RaptorQ's 0x1D.
- **I verified all of its numbers as well.**

---

## 2. [V] Verified facts

### 2.1 RFC 6330 essentials (https://www.rfc-editor.org/rfc/rfc6330, August 2011)

**The field.** §5.7.1: "octets can be thought of as elements of a finite field, i.e., the finite field GF(256)".

§5.7.2 defines u + v = u ^ v, with subtraction the same. Multiplication is:

    u * v = 0 if either is 0, else OCT_EXP[OCT_LOG[u] + OCT_LOG[v]]

Division is OCT_EXP[OCT_LOG[u] − OCT_LOG[v] + 255]. The inverse is OCT_EXP[255 − OCT_LOG[u]]. "alpha is the octet with the integer representation 2", with alpha^^i = OCT_EXP[i].

§5.7.3: OCT_EXP has 510 entries and starts 1, 2, 4, 8, 16, 32, 64, 128, 29, 58, 116, 232, 205, …

- **The RFC never names the polynomial.** A grep for "irreducible", "polynomial" and "primitive" finds nothing. It defines the field only through its tables.
- I verified that the tables are exactly the powers of x modulo x^8+x^4+x^3+x^2+1 (0x11D). All 510 OCT_EXP entries and all 255 OCT_LOG entries match.
- I also checked table-based multiplication against polynomial multiplication for all 65,536 pairs. They agree.

**Parameters.** §5.3.3.3 gives L = K'+S+H and P = L−W:
- S is the number of LDPC symbols. It is prime, and these are GF(2) rows.
- H is the number of HDPC symbols.
- W is the number of LT symbols. It is prime.
- P is the number of PI (permanently inactivated) symbols.
- K'_max = 56403. ESIs are 24-bit. Table 2 lists 477 values of K'.

From Table 2, with my arithmetic:

| K' | S | H | W | L | P | (S+H)/K' |
|---|---|---|---|---|---|---|
| 10 | 7 | 10 | 17 | 27 | 10 | 170% |
| 101 | 17 | 10 | 113 | 128 | 15 | 26.7% |
| 1002 | 59 | 10 | 1021 | 1071 | 50 | 6.9% |
| 10017 | 241 | 11 | 10111 | 10269 | 158 | 2.5% |
| 56403 | 907 | 16 | 56951 | 57326 | 375 | 1.6% |

**H only ranges from 10 to 16 across all 477 values.** A block of 56,403 symbols carries just 16 GF(256) constraint rows.

**HDPC rows.** §5.3.3.3: G_HDPC = MT·GAMMA, with GAMMA[i,j] = alpha^^(i−j) for i ≥ j (lower-triangular Toeplitz, unit diagonal).

For every column except the last, MT[i,j] = 1 when i = Rand[j+1,6,H] or i = (Rand[j+1,6,H] + Rand[j+1,7,H−1] + 1) % H. Those two positions always differ, so each such column has exactly two 1s. The last column is alpha^^i.

**Matrix A.** §5.3.3.4.2, Figure 5:
- First S rows: G_LDPC,1 | I_S | G_LDPC,2
- Next H rows: G_HDPC | I_H
- Last K' rows: G_ENC

**Decoding.** §5.4.2.1: "C can be decoded if and only if the rank of A is L."

"The order in which Gaussian elimination is performed … has no bearing on whether or not the decoding is successful." So inactivation decoding is exactly as strong as full Gaussian elimination; the ordering only affects speed.

**Degree table.** §5.3.5.2, Table 1. My analysis: for d = 2…29 it is **exactly the ideal soliton 1/(d(d−1))**, quantized to 2^-20. Specifically:
- P(2) = 524288/2^20 = 1/2 exactly.
- P(1) = 5243/2^20 ≈ 0.5%.
- All mass for d ≥ 30 sits at 30 (0.0295 = 1/29 − P(1)).
- Mean degree is 4.817.
- The extra PI degree d1 is 2 or 3, so a repair symbol XORs about 7.15 intermediate symbols on average.

**Compliance requirement.** §5.8, quoted: "If the decoder receives K' encoding symbols … with corresponding ESIs chosen independently and uniformly at random … then on average the decoder will fail to recover the entire source block at most 1 out of 100 times." The same requirement continues with K'+1 → 1 in 10,000 and K'+2 → 1 in 1,000,000.

**Erratum.** The only verified erratum (5548) concerns the maximum transfer length. It is irrelevant to this video.

### 2.2 Measured RaptorQ failure rates and an ablation

**Implementation and cross-check.** I implemented the RFC 6330 construction in Python: Rand, Deg, Tuple, Enc, the LDPC rows, MT·Γ, and A. My encoder produced **byte-identical repair packets** to the independent `cberner/raptorq` crate (PyPI 2.0.0) for K' = 10, 26 and 101: 60/60, 76/76 and 151/151 packets. The systematic A was full rank for all of the first 120 K' values (up to 1002).

**Decode failure.** I used cberner's decoder, received random subsets drawn mostly from repair symbols, and set K = K'.

| K | at K | at K+1 | at K+2 |
|---|---|---|---|
| 10 (10⁶ trials) | **0.632%** ± 0.008% | **2.4×10⁻⁵** (24 failures) | 1×10⁻⁶ (1 failure; too few to estimate) |
| 101 (2×10⁵) | 0.631% | 2×10⁻⁵ (4 failures) | not run |
| 1002 (2×10⁴) | 0.54% | 0 in 20,000 | not run |

My own rank-based failure at K' = 10 was 0.646% (10⁵ trials), which agrees with cberner's decoder.

**Ablation (my experiment, new as far as I know).** I kept RaptorQ's matrix and replaced its 10 GF(256) HDPC rows.
- **With 10 dense random GF(2) rows:** failure at K' was **70.7%** (K'=10) and **70.2%** (K'=101). At K'+1 it was 42.7% / 41.6%; at K'+2, 22.9% / 24.4%. These are essentially the random-binary-matrix values (71.1%, 42.2%, 23.0%).
- **With dense random GF(256) rows:** 0.685%, which matches the structured MT·Γ result within noise.

So **ten rows of byte arithmetic take the failure rate from about 71% to about 0.6%**, and the RFC's structured rows behave like random GF(256) rows.

### 2.3 Random-matrix rank math

I computed these exactly and checked them by Monte Carlo at k = 20, 20k trials. MC: GF(2) 0.712 / 0.429 / 0.223 / 0.119; GF(256) 0.00395.

- A random k×k matrix over GF(q) is invertible with probability ∏_{i=1..k}(1−q^-i).
  - q=2: k=1 gives 0.5, k=2 gives 0.375, k=3 gives 0.328. The limit is **0.288788…**, essentially reached by k ≈ 20.
  - q=256: k=1 gives 0.99609, and the limit is **0.996078…**.
- With k unknowns and k+h random equations, P(rank k) = ∏_{i=h+1..k+h}(1−q^-i). In the k→∞ limit, failure is:

| h | q=2 | q=256 |
|---|---|---|
| 0 | 0.7112 | 3.92×10⁻³ |
| 1 | 0.4224 | 1.53×10⁻⁵ |
| 2 | 0.2299 | 5.98×10⁻⁸ |
| 3 | 0.1199 | 2.34×10⁻¹⁰ |

- **Successive failure ratios:** about 1.7 to 2 for q=2, and 255.996 / 256.00 for q=256.
- **Bounds.** The union bound gives failure ≤ q^-h/(q−1). Liva–Paolini–Chiani (IEEE Comm. Lett. 2010, DOI 10.1109/LCOMM.2010.02.092080, as quoted in arXiv 1909.08545) give q^-(h+1) ≤ P_fail < q^-h/(q−1), which is tight. Qualcomm's overview uses the 1−1/256^(h+1) figure.
- **Expected extra equations** beyond k is Σ_{i≥1} 1/(q^i−1). That is **1.6067** for q=2 (the Erdős–Borwein constant) and **0.00394** for q=256.
- **Intuition, exactly stated.** When you already hold i independent equations, they span q^i of the q^k possible coefficient vectors. A uniformly random new one is wasted with probability q^(i−k). For the last needed equation (i = k−1) that is 1/2 over GF(2) and 1/256 over GF(256).
- **Important nuance.** A 0/1 matrix has the same rank over GF(2) and over GF(256), because rank does not change under field extension. I checked this with a 5-cycle: rank 4 over both fields. **The gain comes from coefficients outside {0,1}, not from "working in GF(256)".**

### 2.4 Field facts

All of these were computed exhaustively.

- **Z/256 is not a field.** 2·128 ≡ 0, 2 has no inverse, and 127 nonzero elements are non-units (the evens). Row a of the Z/256 table has 256/gcd(a,256) distinct values: row 2 has 128, row 128 has 2.
- **GF(256).** Every nonzero row of the multiplication table is a permutation.
- **Polynomial counts.** There are 30 irreducible degree-8 polynomials over GF(2): (2^8−2^4)/8 = 240/8. Of these, 16 are primitive: φ(255)/8 = 128/8. Both 0x11D and 0x11B are irreducible. **Only 0x11D is primitive.** Under 0x11B, 2 has order 51 and 3 has order 255.
- **Element orders** in GF(256)*: orders 1, 3, 5, 15, 17, 51, 85, 255 occur 1, 2, 4, 8, 16, 32, 64, 128 times. They are all powers of 2 because 255 = 3·5·17 is a product of Fermat primes, so every φ(d) is a power of 2. There are 128 primitive elements.
- **Subfields.** GF(16) = {0,1,10,11,68,69,78,79,146,147,152,153,214,215,220,221}. GF(4) = {0,1,214,215}.
- **Worked examples** (RaptorQ field, 0x11D):
  - 2·128 = **29** (Z/256: 0; AES: 27).
  - 3·7 = **9**: carry-less 011×111 = 111⊕1110 = 1001, no reduction needed.
  - 7·7 = 21: (x²+x+1)² = x⁴+x²+1. This is the "freshman's dream", which I checked for all pairs.
  - 128·3 = **157**: x⁸+x⁷ → x⁷+x⁴+x³+x²+1.
  - 150·3 = **167**: 442 carry-less, XOR 0x11D; log route 180+25 = 205, OCT_EXP[205] = 167.
  - 0x57·0x83 = **0x31 (49)**. The carry-less product is x¹³+x¹¹+x⁹+x⁸+x⁶+x⁵+x⁴+x³+1. XOR x⁵p gives x¹¹+x⁷+x⁶+x⁴+x³+1. XOR x³p gives x⁵+x⁴+1. **Under AES's 0x11B the same product is 0xC1**, the FIPS-197 example.
  - 0x53·0xCA = 1 under AES but **0x8F** under RaptorQ.
  - Inverses: 2⁻¹ = **142** (2·142 = 284 = 0x11C, XOR 0x11D = 1). 3⁻¹ = 244.
  - 3 = α^25, 29 = α^8, 128 = α^7.
- **Log-ordered table.** If rows and columns are ordered by discrete log, entry (i,j) = α^((i+j) mod 255). The table becomes a perfect Hankel/circulant stripe pattern (checked).
- **xtime.** (a<<1) ^ (0x1D if a&0x80) equals multiplication by 2 for all 256 bytes.
- **Same polynomial elsewhere.** 0x11D is also the Linux RAID-6 field (Anvin, "The mathematics of RAID-6": "{02}^8 = {1d}", https://www.kernel.org/pub/linux/kernel/people/hpa/raid6.latex).

### 2.5 LT, peeling and the coupon collector

- **Random copies.** If each packet is a uniformly random copy of one of K originals, you need K·H_K packets on average to see them all. For K = 10,000 that is **97,876 (9.79×)**.
- **Never-touched symbols.** The article's e^-5 = 0.674% gives about 67 of 10,000 symbols. With RFC degrees (mean 4.82) and N = K, it is e^-4.82 ≈ 0.81%.
- **Pure peeling with the RFC degree table** (my sim, K = 10,000, 5 runs each):
  - N = K: collapses (8–42% recovered).
  - N = 1.02K: about 98.2–98.8% (one run collapsed at 16%).
  - N = 1.05K: about 98.7–99.0%.
  - N = 1.5K: still about 0.02–0.1% of symbols never touched.
- **Shokrollahi 2006, Prop. 1:** a reliable LT decoder needs Ω(k log k) edges. Quote: "It is absolutely remarkable that it is possible to construct a weight distribution that matches this lower bound via a fast decoder. Such distributions were exhibited by Luby."
- **Raptor (Shokrollahi 2006 abstract):** overhead k(1+ε), O(log(1/ε)) per symbol, O(k log(1/ε)) to decode. Precoding was also found independently by Maymounkov.
- **Qualcomm overview:** about √K PI symbols. The dense phase is M×M with M ∝ √K, so M³ bit operations are "dwarfed" by M² ≈ K symbol operations, and the total is linear time.
- **Peeling toy** (5 unknowns A=5, B=3, C=7, D=2, E=12; packets y1=C=7, y2=A⊕C=2, y3=B⊕C⊕D=6, y4=A⊕B=6, y5=D⊕E=14, y6=B⊕E=15):
  - Peel order: C, A, B, D, E. The ripple size goes 1, 1, 1, 2, 2.
  - Drop y1 and nothing peels.
  - Inactivate C as an unknown c. Then A = y2⊕c, B = y4⊕y2⊕c, D = y3⊕y4⊕y2 (c cancels!), E = y5⊕D.
  - y6 gives c = y6⊕y5⊕y3 = 15⊕14⊕6 = **7**. Back-substitution then gives A=5, B=3, D=2, E=12.
- **The cycle.** The equations A⊕B, B⊕C, C⊕D, D⊕E, E⊕A have **rank 4 over GF(2)**, so the system is genuinely unsolvable, not merely stuck.
  - Replace the last equation with E + 2·A and the rank becomes 5.
  - With values: y = 6, 4, 5, 14, 6. S = y1+y2+y3+y4 = 9. Then y5+S = 15 = (1+2)·A = 3A, so A = 15/3 = **5**.
  - In general a cycle of equations a_i x_i + b_i x_{i+1} is singular exactly when ∏(b_i/a_i) = 1. With random nonzero GF(256) coefficients that has probability 1/255 (MC 0.0038).

### 2.6 Context, deployments and history

- **Feedback implosion.** RFC 3453: ARQ "has limitations, including the feedback implosion problem because many receivers are transmitting back to the sender…". With 10⁶ receivers at 1% loss, P(a packet reaches everyone) = e^-10050 ≈ 0. Each packet is missed by about 10,000 receivers.
- **3GPP MBMS.** SA#28, June 2005 (SP-050246): "SA4 has agreed to select Raptor codes as the FEC code for MBMS file delivery and streaming in Release 6." This is R10 (RFC 5053), chosen over a Reed–Solomon proposal. **I could not confirm that RaptorQ was later adopted into TS 26.346.**
- **ATSC 3.0.** A/331 (2021) cites RFC 6330 for the ROUTE AL-FEC ("FEC Scheme Raptor(Q) (RFC6330)"; "The FEC building block is used according to RFC 6330").
- **DVB-IPTV.** AL-FEC with a Raptor enhancement layer (RFC 6683). DVB-H IPDC also used Raptor (secondary source).
- **Digital Fountain** was acquired by Qualcomm in February 2009.
- **"Raptor" = "rapid tornado"** (Shokrollahi's 2011 KTH talk, cited by Wikipedia).
- **MDS limit.** A linear [n,k] MDS code over GF(q) with k ≥ 2 has n ≤ q+k−1 (standard bound). Reed–Solomon over GF(256) is therefore capped at about 256–257 symbols per codeword. **No fixed-field linear code can be both rateless and MDS.**

---

## 3. [P] Narrative arc

The video is 9 chapters, about 12 minutes. Its spine is that **each fix exposes the next problem**: copies, then XOR equations, then GF(2) rank loss, then a byte field, then the cost of density, then the sparse+dense hybrid.

**Ch1. A million phones (~75 s). Hook.**
- *Idea:* Broadcast one file to a million phones that each lose different packets. Asking for retransmissions drowns the sender. Sending random copies makes you wait for the rarest one.
- *Picture:* A dot-field of phones, each with its own red loss pattern. NACK arrows flood the tower. Then a coupon-collector counter crawls toward the last missing piece.
- *Numbers:* Each packet is missed by about 10,000 phones at 1% loss. 10,000 pieces need 97,876 random copies.
- *Misconceptions:* Loss here means *erasure*, not corruption: packets either arrive intact (checked by checksums) or not at all.
- *Jargon:* Say "lost packet" and avoid "erasure channel" until the end. Use "feedback implosion", not "ACK storm" (that term means something else).
- *Personas:* The teen gets the stakes. The developer gets why TCP doesn't scale to broadcast. The mathematician gets an honest coupon-collector setup.

**Ch2. Packets are equations (~80 s).**
- *Idea:* Send combinations instead of pieces. XOR is addition with no carries, and XOR undoes itself. Every packet is one equation, arrival order is irrelevant, and you need k *independent* equations.
- *Picture:* Four bytes as bit columns. Packets light up the subset of columns they combine.
- *Example:* The article's A=5, B=3, C=7, D=2 → 2, 6, 6, 3. These four packets have rank 4 (verified).
- *Misconception:* "Any four packets work." Set up the cliffhanger: what if a new equation is redundant?
- *Personas:* The developer gets XOR as the hero. The mathematician sees the vector space GF(2)^k.

**Ch3. The XOR lottery (~85 s).**
- *Idea:* Each new random equation is wasted if it lands in the span of the old ones. Over GF(2), the last needed equation is a coin flip. Exactly-k success is 28.9%, and you need 1.607 extra packets on average.
- *Picture:* A 3D cube of GF(2)³ (8 points). Spans light up as a point, then a line, then a plane. Random arrows "miss" into the lit region.
- *Numbers:* 0.5, 0.375, 0.328, then the limit 0.2888. Failure at K, K+1, K+2: 71%, 42%, 23%.
- *Misconception:* Bad luck rather than structure. Show the 5-cycle, whose equations sum to zero over GF(2).
- *Personas:* The teen gets "the target you must avoid". The mathematician gets ∏(1−q^-i).
- *Close on a question:* what if each coefficient had 256 choices instead of 2?

**Ch4. Why not just "mod 256"? (~75 s)**
- *Idea:* Elimination needs division, and mod 256 has zero divisors. 2·128 = 0, so 2 has no inverse. Mod a prime works, but 257 doesn't fit in a byte and 251 wastes values.
- *Picture:* A Z/256 multiplication heat map. Even rows collapse into banded "shadows", and row 128 shows only two colors.
- *Jargon:* Define "field" as "a number system where you can always divide by anything except zero". Use "ring" only for Z/256.
- *Personas:* The teen sees why primes matter. The developer sees why `u8` multiplication is useless here.

**Ch5. Bytes are polynomials (~85 s).**
- *Idea:* Read the bits as coefficients. Addition is XOR. Multiplication is carry-less, with overflow folded back using x⁸ = x⁴+x³+x²+1. An irreducible polynomial is a "prime polynomial", which is why there are no zero divisors.
- *Picture:* The bits of 128 morph into x⁷. Multiplying by x shoves the 1 off the edge, and it lands as 00011101 = 29.
- *Examples:* 3·7 = 9; 2·128 = 29 (vs 0 in Z/256); 0x57·0x83 = 0x31.
- *Misconception:* "GF(256) is mod 256." It is not.
- *Personas:* The developer gets a 3-line xtime. The mathematician gets GF(2)[x]/(p), with all fields of order 256 isomorphic: the polynomial picks coordinates, and that matters for interoperability.

**Ch6. A slide rule for bytes (~80 s).**
- *Idea:* The powers of 2 visit all 255 nonzero bytes, so every byte has a logarithm. Multiplication becomes adding logs mod 255, which is where the RFC's OCT_EXP/OCT_LOG tables come from.
- *Contrast:* Under AES's polynomial, doubling closes after 51 steps. Return to the lottery: over GF(256), exactly-k succeeds 99.61% of the time, and each extra packet cuts failure about 256×.
- *Picture:* A 255-gon clock. Then the heat-map reorder (set piece below).
- *Example:* 150·3: logs 180+25 = 205 → 167.
- *Misconception:* Primitivity is needed for a field. Irreducibility is enough; primitivity just makes 2 the generator, which is convenient.

**Ch7. The price of density (~85 s).**
- *Idea:* Dense GF(256) elimination is O(K³), about 1.25×10¹⁴ operations for K = 50,000. Sparse XOR packets plus "peeling" are fast, but the coupon collector returns: about 0.8% of symbols are never touched, and peeling stalls around 98–99%.
- *Picture:* A bipartite graph cascade with the ripple glowing, then a stall.
- *Example:* The 5-unknown peel (C, A, B, D, E) and the drop-y1 stall.
- *Misconception:* Peeling is not Gaussian elimination. It can fail when GE would succeed.

**Ch8. Precode plus a pinch of GF(256) (~85 s).**
- *Idea:* Solve for slightly more "intermediate" symbols bound by cheap XOR checks (LDPC) plus 10–16 dense GF(256) rows (HDPC). Inactivation means peel, park the stuck unknowns, solve a small dense core, then back-substitute.
- *Picture:* Matrix A with a gold HDPC band. Then the parking animation, with c resolving to 7.
- *Cycle twist:* A single coefficient of 2 makes the loop solvable (15/3 = 5).
- *Ablation:* 0.65% vs 71%.
- *Personas:* The developer gets "99.9% XOR, a sliver of table lookups". The mathematician gets the holonomy view of cycles.

**Ch9. The payoff (~70 s).**
- *Measured:* 0.63% failure at K, about 2.4×10⁻⁵ at K+1, about 10⁻⁶ at K+2. The RFC requires ≤ 1%, ≤ 0.01% and ≤ 0.0001%.
- *Systematic:* Clean channels need no decode.
- *Deployed:* ATSC 3.0, and Raptor in 3GPP MBMS.
- *Closing line:* A polynomial that refuses to factor is why two extra packets buy a million-to-one.

---

## 4. [P] Set pieces

1. **The heat-map reveal.** Show the 256×256 GF(256) table in natural order, which looks like white noise with every row a permutation. Then sort both axes by discrete log. Rows slide into place and the table snaps into perfect anti-diagonal stripes (entry = α^(i+j)). Wrap the result onto a torus, since logs are taken mod 255.
2. **Z/256 vs GF(256) side by side.** Collapsing bands on one side, a Latin-square shimmer on the other.
3. **The doubling clock.** 255 points on a circle. Under 0x11D the doubling path traces a single 255-step star. Under 0x11B it closes into a 51-step star, and switching to 3 completes the full circle.
4. **Bits → polynomial → fold.** The x⁸ overflow term "falls" and lands as the 0x1D bit pattern.
5. **The rank race.** Two matrices fill with random rows, and a rank bar ticks up. A Monte Carlo histogram shows "extra packets needed": GF(2) mean 1.607, GF(256) mean 0.004.
6. **The packet rain** into a bucket, with a fill gauge in units of rank.
7. **The peeling cascade** on a Tanner graph, with the ripple and the stall. A ghosted "inactive" node lets the cascade resume.
8. **The cycle holonomy.** Walk around the pentagon. Over GF(2) you return unchanged (no information). With one twist you return "rotated", and the loop pins the unknowns.
9. **Zoom on matrix A.** Zoom from 27×27 (K'=10) out to 10,269 columns (K'=10,017). Binary speckle, with a single 11-row gold thread for the HDPC rows.
10. **The ablation counter.** Two million-trial tallies run side by side: GF(256) HDPC at about 0.65% and GF(2) HDPC at about 71%.

---

## 5. [V] Ledger of on-screen numbers

Unless noted, numbers were verified by the Python scripts listed above.

| Value | How verified |
|---|---|
| 0x11D = x⁸+x⁴+x³+x²+1; OCT_EXP[0..9] = 1,2,4,…,128,29,58 | RFC tables vs computed powers (exact match) |
| 2·128 = 29 (Z/256: 0; AES: 27); 3·7 = 9; 128·3 = 157; 150·3 = 167; 0x57·0x83 = 0x31 (AES 0xC1); 2⁻¹ = 142 | Exhaustive multiply plus table route |
| 30 irreducible, 16 primitive; ord(2) under 0x11B = 51 | Enumeration |
| 0.288788, 0.996078; failure table in §2.3; 1.6067 / 0.00394 | Exact products; MC at k=20 |
| 4.817 mean degree; P(2) = ½; ≈7.15 symbols per repair | RFC Table 1 |
| H = 10–16; P = 158 at K' = 10017 | RFC Table 2 |
| 97,876 copies for K = 10,000; e^-5 = 0.674% | Exact |
| Peeling stall ≈ 98–99% at 2–5% overhead | My simulation (K = 10⁴, 5 runs; one collapse) |
| RaptorQ 0.632% / 2.4×10⁻⁵ / ~10⁻⁶ | cberner/raptorq MC, 10⁶ trials, K = 10 |
| Ablation 0.65% vs 70.7% | My RFC implementation, cross-validated byte-exact |
| Toy values (c = 7; 15/3 = 5) | Script |

---

## 6. Pitfalls a mathematician or coding theorist would flag

**Problems in the owner's materials:**

1. **"Collect any K… recover the original"** (article subtitle). This is false as stated. The RFC guarantees ≤ 1% failure *on average over uniformly random ESIs*; it does not cover every subset or adversarial loss. Say "almost any K; about 99.4% of the time measured".
2. **"Total overhead under 5%."** Intermediate symbols are never transmitted. The bandwidth overhead is the reception overhead, about +0 to +2 symbols. The precode expansion costs computation, and it is about 2.5% only near K ≈ 10⁴ (6.9% at K ≈ 1000).
3. **The article's 5-cycle example** is rank-deficient over GF(2). Inactivation cannot solve it; it needs another equation or a non-binary coefficient. That makes it the ideal place to introduce GF(256).
4. **"≈√K permanently inactivated; 100×100 at K = 10,000".** The RFC gives P = 158 (≈1.6√K'). Inactivations made during decoding add to that.
5. **GF(2) at K+2 is "~78%"** in the article. The correct limit is 77.0%.
6. **"Basis of much of the robustness in modern cell networks"** (X post). This overstates it. Raptor/RaptorQ is application-layer FEC for broadcast/multicast (MBMS, ATSC 3.0, DVB). Cellular link robustness mainly comes from physical-layer codes and HARQ.

**General pitfalls:**

7. **MDS vs near-MDS.** Reed–Solomon is MDS. Rateless MDS over a fixed field is impossible (n ≤ q+k−1). Near-MDS is a probabilistic statement.
8. **"GF(256) helps" needs care.** Rank is field-extension invariant, so the benefit comes only from coefficients outside {0,1}, in the HDPC rows.
9. **∏(1−q^-i) is an idealization.** RaptorQ's matrix is structured and sparse. Its measured 0.63% is about 1.6× the random GF(256) value. Present the formula as intuition and the measurements as fact.
10. **Field vs ring language.** Z/256 is a ring; GF(2⁸) ≅ GF(2)[x]/(p) is a field. The choice of polynomial is a representation choice that matters for wire compatibility. All fields of order 256 are isomorphic.
11. **Irreducible ≠ primitive.** Irreducible is required for a field. Primitive only makes x a generator, which is convenient.
12. **Peeling vs Gaussian elimination.** Peeling can fail where GE succeeds. RFC inactivation decoding succeeds iff rank(A) = L.
13. **Reed–Solomon costs.** O(n log n) FFT decoders exist. RS's real limits are its fixed rate and the block length capped by field size.
14. **K vs K'.** The guarantees are stated at K'. Padding symbols are known zeros and are never sent.
15. **"Random" is pseudorandom and deterministic** (V0–V3 tables). The ESI alone tells both sides which equation a packet carries.
16. **The RFC degree table is a truncated ideal soliton,** not the robust soliton. Robustness comes from the precode and inactivation.
17. **Log of 0 is undefined.** Zero is handled separately (RFC §5.7.2).
18. **The RFC promises only 100× per extra symbol.** The roughly 256× seen in random dense GF(256) matrices is the model; I measured about 263× from K to K+1 at K' = 10.

---

## 7. Sources

- Article: https://www.jeffreyemanuel.com/writing/raptorq
- X post: https://x.com/doodlestein/status/2063674211558904278 (retrieved via https://api.fxtwitter.com/doodlestein/status/2063674211558904278)
- Claude share: https://claude.ai/share/9e196534-ae1f-4f90-8b98-39f2c5fd2045 (via r.jina.ai)
- ChatGPT share: https://chatgpt.com/share/6a24a163-caf4-83ea-88cd-2a656dbb3f80
- RFC 6330: https://www.rfc-editor.org/rfc/rfc6330 · errata: https://errata.rfc-editor.org/rfc6330
- RFC 3453: https://www.rfc-editor.org/rfc/rfc3453
- RFC 6683 (DVB-IPTV AL-FEC): https://www.rfc-editor.org/rfc/rfc6683
- Qualcomm, RaptorQ Technical Overview (2010): https://qualcomm.com/content/dam/qcomm-martech/dm-assets/documents/RaptorQ_Technical_Overview.pdf
- Shokrollahi 2006: https://people.ece.ubc.ca/janm/Papers_RG/Shokrollahi_IT_June06.pdf
- Shokrollahi & Luby monograph: https://nowpublishers.com/article/Details/CIT-060
- Liva–Paolini–Chiani bounds, as quoted in https://arxiv.org/pdf/1909.08545 · Lázaro et al. 2018: https://arxiv.org/abs/1809.01515
- 3GPP SP-050246: https://www.3gpp.org/FTP/tsg_sa/TSG_SA/TSGS_28/Docs/PDF/SP-050246.pdf
- ATSC A/331:2021: https://www.atsc.org/wp-content/uploads/2021/01/A331-2021-Signaling-Delivery-Sync-FEC.pdf
- Qualcomm acquires Digital Fountain: https://www.streamingmediablog.com/2009/02/qualcomm-acquires-digital-fountain.html
- Raptor code (Wikipedia; "rapid tornado" etymology): https://en.wikipedia.org/wiki/Raptor_code
- RAID-6 field: https://www.kernel.org/pub/linux/kernel/people/hpa/raid6.latex
- cberner/raptorq (used for cross-validation): https://github.com/cberner/raptorq
