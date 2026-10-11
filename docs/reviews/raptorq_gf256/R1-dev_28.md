<!-- R1 storyboard review, 2026-10-10, from the reviewer's final answer (packet tools/review.py; storyboards at e26ec36) -->

# R1 review: Video B, "The Arithmetic Where 3 × 7 = 9" (Dev, 28)

The storyboard as generated contains a broken code panel. That is a blocker, and the cause is a bug in the storyboard builder. Eight findings are major; most are mechanisms or costs that sit in the notes but are never voiced.

## What I checked and how

- **Code panel (06):**
  - I wrapped the storyboard's 3-line panel in `fn main` and compiled it with `rustc 1.100.0-nightly --edition 2021`. It gives 8 errors: `xtime`, `a` and `b` are not found, plus one type mismatch.
  - The plan's full 6-line block (plan lines 1748–1755) compiles with `-O` and no warnings. Both claims in its comment hold: `mul(0x57,0x83)==0x31` and `mul(2,142)==1`.
  - I compared it on all 65,536 pairs against my own carry-less multiply followed by reduction by 0x11D. There were 0 mismatches.
  - A 510-entry exp/log table version also has 0 mismatches against it.
  - Under 0x11B, 0x57·0x83 = 0xC1.
  - Files are in `/tmp/r1_dev_b/` (`panel_plan.rs`, `panel_storyboard.rs`).
- **Why the panel breaks:** I read `docs/storyboards/build.py`. It keeps a note line only if it starts with `"- "` or two spaces. So the code fences, `fn xtime…`, `fn mul…` and `}` are all dropped. The bakery storyboard loses its whole SeqCst panel the same way: its 10_4 says "(below)" and nothing follows.
- **Numbers I recomputed (Python 3.14.6, `checks.py`, `checks2.py`):**
  - Chapter 03: 21/64 = 0.328; limit 0.288788.
  - Failure with 0–3 spares: 71.12 / 42.24 / 22.99 / 11.99%. 20 spares are needed for 10⁻⁶.
  - At K = 10: 0.7109. The GF(256) model at K, K+1, K+2: 3.9215×10⁻³ / 1.5319×10⁻⁵ / 5.984×10⁻⁸.
  - K·H_K = 97,876.06. The carousel figures are 0.368 / 0.990 per phone, and 0.368 / 0.990 for all 10⁶ phones by 5 and 6 loops.
  - Cards 2, 6, 6, 3. B⊕C = 4. 5⊕7 = 2.
  - Pentagon cards 6, 4, 5, 14, 6; S = 9; 3A = 15; 3⁻¹ = 244; A = 5.
  - 2·128 = 29, 128·4 = 58, 150·3 = 167. Logs: 3 → 25, 150 → 180; α²⁰⁵ = 167.
  - Under 0x11B, 2 has order 51 and 3 has order 255. There are 30 irreducible and 16 primitive degree-8 polynomials.
  - Inactivation toy (from `toy_examples.py`): c = 15⊕14⊕6 = 7.
  - e^−4.8166 = 0.809%. 50,000³/3 = 4.17×10¹³.
  - Z/16, all 65,536 2×2 matrices: invertible exactly when the determinant is odd (fraction 0.375).
- **CRC-8:** the reveng CRC catalogue lists CRC-8/SAE-J1850 with polynomial 0x1D. Looping the panel's `xtime` over "123456789" gives its catalogued check value, 0x4B.
- **Unchecked by me (taken from the fact sheet):**
  - The recorded Monte Carlo counts (B-F21, B-F22).
  - Erdős 1948, RAID-6/Linux, MBMS 2005 and the QR polynomial.
  - **ATSC 3.0 is unchecked by anyone.**
  - My "model" numbers in D9 and D14 are my own and have not been through the oracle.

## Comprehension check

| ch | What I'd tell a colleague |
|---|---|
| 01 | Broadcasting to a million phones that can't ask for resends is hard. A loop makes the unluckiest phones wait 5–6 loops, random pieces are worse (about 98k for 10k pieces), yet a phone holding K+2 packets almost always rebuilds the file. |
| 02 | Each packet is the XOR of some subset of pieces, and its number seeds a PRNG that names the subset. So each packet is one linear equation over bits. The phone does Gaussian elimination and only needs K independent equations, in any order. |
| 03 | K uniformly random XOR rows are full rank only about 29% of the time. Each useful row doubles the span, so the chance the next row is useless is 2^(rank−K). Nearly all the risk is in the last few rows, whatever K is. |
| 04 | Each spare packet only halves the failure rate (about 20 spares for 10⁻⁶). Some structures, such as a 5-cycle of pairs, are dependent no matter what. So we want coefficients beyond 0/1, which needs a byte multiply we can undo. |
| 05 | Wrapping at 256 fails: even numbers lose the top bit (2·128 = 0), so invertibility depends only on parity and you're back at 71%. You need a 256-element field, and plain "mod" can't give you one. |
| 06 | Treat a byte as a GF(2) polynomial and multiply carry-less. Reduce with an irreducible polynomial (0x11D): shift left, and if a bit falls off, XOR in 0x1D. Now every nonzero byte has an inverse. 3·7 = 9 because no reduction happens. |
| 07 | Under 0x11D, doubling visits all 255 nonzero bytes, so multiplication becomes adding logs mod 255, which is the table method. AES's polynomial also gives a field, but there 2 cycles after 51 steps. |
| 08 | Random byte coefficients miss any fixed bad direction with probability 1/256. Failure at K is about 0.39% and drops 256× per extra card, but only if the coefficients actually use values other than 0/1. |
| 09 | A dense solve is K³/3 (about 4×10¹³ at 50k pieces), so real codes keep cards sparse and solve by peeling. Peeling is cheap but fragile: lose the starting card and nothing moves; with a few % extra it still stalls at 98–99%. |
| 10 | RaptorQ packets stay XOR. Both sides add fixed cheap XOR clues plus 10–16 byte-weighted clues that close the last open directions, taking failure from 71% to about 0.6%. Decoding is peel, park a few stubborn pieces, then a small dense solve. I filled in "clues are free equations" myself (D3), and I can't say what decoding costs (D5). |
| 11 | Measured on 10-piece blocks: 6,316 / 24 / 1 failures per million at K, K+1, K+2, within RFC 6330's 1/100, 1/10⁴, 1/10⁶ promise. It is systematic. It is used in TV broadcast; its older sibling, Raptor, in 3GPP MBMS; and the same field is in RAID-6. |

None of these sentences is empty or wrong, so no chapter-level blocker. Chapter 10 is the shakiest.

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 4 | 4 | 5 | 4 | 4 |
| 02 | 4 | 4 | 4 | 3 | 3 |
| 03 | 5 | 5 | 5 | 4 | 5 |
| 04 | 4 | 5 | 4 | 4 | 3 |
| 05 | 4 | 5 | 4 | 4 | 4 |
| 06 | 4 | 2 | 4 | 3 | 4 |
| 07 | 5 | 5 | 3 | 4 | 5 |
| 08 | 4 | 4 | 4 | 4 | 3 |
| 09 | 4 | 3 | 4 | 4 | 4 |
| 10 | 3 | 3 | 4 | 3 | 4 |
| 11 | 4 | 4 | 4 | 4 | 3 |

- 02 pacing is 3 for me because of the binary primer. Maya wins that one, so no finding.
- 06 correctness is 2 because of the code panel as generated. The voice itself is correct.

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me | suggested fix |
|---|---|---|---|---|---|
| D1 | Dev | 06, code panel note; `docs/storyboards/build.py` | blocker | **The storyboard's panel is three orphan lines.** There is no `fn mul(mut a: u8, mut b: u8) -> u8`, no closing `}` and no `fn xtime`. Wrapped in `fn main`, rustc gives 8 errors. Worse, `xtime` is the only line where 0x1D appears in code, which is the whole point of 06, and the brief promises "the Rust `xtime` / `mul` code". The plan's block is correct (compiled; 65,536/65,536 agree with an independent reference). The cause is build.py's note filter, which drops fences and column-0 lines. The same bug deletes bakery 10_4's entire SeqCst panel. | Make build.py copy fenced blocks verbatim (track ``` state and append every line until the closing fence), then re-run. Add a G2 check: every "code panel" note must contain a fenced block, and Rust blocks must pass `rustc --edition 2021` inside a wrapper. Fact sheet B-F11 says "code panel (05)"; it should say 06. |
| D2 | Dev | 10_7 vs 10_1 | major | **10_7 breaks the rule 10_1 just set.** 10_1 says RaptorQ packets never carry byte amounts, then 10_7 fixes the pentagon with a *card* "E plus two A". That is the network-coding design from 08 (byte amounts in every card), not RaptorQ. Building from this, I'd put byte coefficients in packets. The arithmetic itself is right (I checked 6, 4, 5, 14, 6 → S = 9 → 15 = 3·5 → A = 15·244 = 5). Also "A plus two A is three A" sounds like integer addition. Next I'd assume A + 3A = 4A, but it's 2A. | Make the fifth equation a gold byte *clue*, not a warm-white packet: "Now let one byte clue join the loop: E plus two A is six." Add one clause: "adding amounts is ex-or too: one plus two is three." |
| D3 | Dev | 10_2 | major | **"Bonus pieces… never sent: free clues" — I can't see how a never-sent piece helps the phone.** To implement it I need two facts. Each fixed rule is an extra equation the phone adds with a known right-hand side (zero; the RFC's D vector starts with S+H zeros). And the XOR packets mix the bonus pieces in like any other piece. Separately, "blocks" appears here for the first time with no definition. | 10_2: "Each rule is an equation the phone gets for free, because it already knows the answer: zero. Packets mix the bonus pieces in like any other piece." Add a lower-third: "big files are cut into blocks of up to 56,403 pieces; each block decodes on its own and needs its own extra packets." |
| D4 | Dev | 09_5 → 10_2 | major | **The stall at 98–99% is shown but never explained, and chapter 10 never says what fixes it.** The mechanism is only in the Ruth note: at N = K, e^−4.82 ≈ 0.81% of pieces (1 in 124, checked) appear in no card at all, so no decoder can recover them. That is the clearest reason the bonus pieces exist. Also "One run in five collapsed" comes from n = 5 but is said like a rate, and K = 10,000 is never stated. | 09_5: "About one piece in a hundred and twenty isn't in any card at all. No decoder can find it." Outline those pieces red. Reword the collapse: "In one of our five runs it collapsed at sixteen percent." Caption: "10,000 pieces, RFC degree table, 5 seeded runs". In 10_2, tie it back: "the cheap ex-or clues reach the pieces no packet touched" (check against RFC 6330 §5.3.3.3 first). |
| D5 | Dev | 09_1, 10_5–10_6 | major | **Act III says "dense is slow, RaptorQ spends it wisely", but RaptorQ's cost is never given.** 09_1 gives the dense cost (4.17×10¹³, checked), though "steps" has no unit. Chapter 10 never says what RaptorQ's solve costs, so I can't tell whether it is 10× or 10⁵× cheaper. Honest costs are part of my contract. | 09_1: add "at a billion steps a second, about half a day" (11.6 h). 10_6 lower-third: "the dense solve covers only the parked pieces: at 10,017 pieces, at least 158 + 11 of them, about 1.6 million steps instead of 3.6×10¹¹." Replace the 169 floor with a measured inactivation count and a measured decode time (cberner crate, 10,000 × 1 KB) as oracle rows. |
| D6 | Dev | 06 code panel (no line id) | major | **The panel has no line id and no screen time.** Chapter 06 is estimated at 83 s against an 85 s cap, and I need 8–10 s to read 5 lines of Rust. As planned, it either breaks the cap or flashes past. | Pin the panel in a corner from 06_6 through 06_8 (about 20 s). Light `0x1D` orange when 06_7 folds 2·128 into 29, and light the `mul(2, 142) == 1` comment at 06_8. At 06_9, swap `0x1D` → `0x1B` and show 0x31 flip to 0xC1 (both verified). This adds no time. |
| D7 | Dev | 09_6 + its lower-third | major | **Three problems in one aside.** (1) "Reed–Solomon codes are perfect": "perfect code" is a technical term (Hamming, Golay codes). RS codes are MDS, which is a different property. (2) "With byte-sized symbols it tops out near 256": the cap is on the number of packets for a byte-sized *field*. In erasure-coding libraries, RS shards can be as large as you like. (3) The lower-third `n ≤ q + k − 1` is, as far as I know, a true but loose MDS bound, and it is not what caps RS near 256. With q = 256 and k = 10,000 it allows 10,255, so the screen contradicts the voice. | Voice: "Reed–Solomon codes are optimal: any K of N packets work. But you pick N up front, and with byte arithmetic N tops out at about 256 packets." Lower-third: "Reed–Solomon over GF(256): N ≤ 255 (257 extended); more needs a bigger field or separate blocks." |
| D8 | Dev | 01_4 (vs 01_2, 02_2) | major | **01_4 makes piece = packet ("thousands of small pieces called packets").** But 01_2 ("two more packets than the picture has pieces") and 02_2 ("Don't send pieces. Send combinations") only make sense if they differ. This is the source-symbol versus encoding-symbol distinction, and it breaks the brief's one-word-per-idea rule. I first read 01_2 as "receive K+2 of the original pieces". | "A file is cut into thousands of small pieces, and each one travels in its own packet." |
| D9 | Dev | 10_3 | major | **"The ex-or equations… leave a few directions open; the byte clues close them" is the takeaway, but with no numbers.** So I can't answer "why 10–16, not 2?". My own model, not oracle-checked: K′+S XOR rows that stop H short of full fail to be independent about 2^−H of the time (H = 10: 0.098%). H random byte rows finishing the last H directions fail about 1/255 (0.392%). Total ≈ 0.49%, against 0.646% measured (10_4) and 0.63% (11_1). If those last H rows were XOR, you get the square lottery back: 71.1%. | Put the model through the oracle. If it holds, 10_3: "Ten slots short of full, the ex-or part fails about one time in a thousand. The last ten slots, where ex-or fails seven times in ten, go to the byte clues, which miss one time in two fifty-six." Label it **model**, like 08_4. |
| D10 | Dev | 02_7 (and 02_3) | minor | **"Feed it to the same random-number generator" is how I'd get a real implementation wrong.** I'd seed my language's PRNG with the packet number, and my encoder would disagree with a phone written in another language. RaptorQ's generator is a fixed recipe in RFC 6330 (its own Rand function and tables). The phone also needs K and the piece size, which travel separately. Also, 02_3 says a packet lists its pieces and 02_7 says it carries a seed; the switch isn't marked. | "…both sides feed it to the same fixed recipe, written into the standard, so every phone gets the same answer." Lower-third: "the packet number (RFC 6330's ESI) is the seed; the block size travels in a header." |
| D11 | Dev | 07_5; 07 Ruth note | minor | **The only code shown is the slow bit-serial multiply** (up to 8 iterations per byte). The 07 note says "the code checks zero first", but there is no table code anywhere. Tables are how I'd write it and how the RFC does it (OCT_EXP/OCT_LOG). I verified the two-line version against the panel on all 65,536 pairs. | 07_5 lower-third, monospace: `if a == 0 \|\| b == 0 { 0 } else { EXP[LOG[a] + LOG[b]] } // 510-entry EXP: no % 255`. Footnote: "real decoders use tables or SIMD shuffles." |
| D12 | Dev | 07_8; 07 and 11 notes | minor | **"Lets two visit everyone" — so what?** The practical answer is RAID-6: Q = Σ2ⁱ·Dᵢ needs every 2ⁱ distinct, and under 0x11B only 51 are. The fact sheet verified this (Anvin's paper, `mktables.c`), but the storyboard still says "verify before narrating" and gives it no line. The QR note under 11 is stale the same way. | Lower-third at 07_8's hold: "Linux RAID-6 gives drive i the weight 2ⁱ; with AES's polynomial only 51 of those weights differ." Remove both stale "verify" tags. |
| D13 | Dev | 05_2, 05_5 | minor | **"Only the last bit of each amount mattered" is a result without its mechanism.** 05_2's "three A's means dividing" also makes me think 3 is the problem, but 3 is invertible mod 256. The mechanism: odd multipliers have inverses mod 256 (3 × 171 = 513 ≡ 1), even ones don't. I brute-forced 2×2 matrices mod 16: invertible exactly when the determinant is odd. | 05_5: "Odd amounts can be undone, even ones can't, so the lottery only sees each amount's last bit: heads or tails, like ex-or." Optional programmers' lower-third: "3 × 171 = 513 = 2·256 + 1; it's why multiplying by an odd constant in a hash mixer is reversible." |
| D14 | Dev | 01_6 vs 01_5 | minor | **The comparison mixes an average with a worst case.** 97,876 (checked) is the average with no loss; 01_5 quoted the unluckiest phone. At the 1% loss just stated, the sender transmits about 98,865 on average. The unluckiest of a million needs about 234,000 received (my Gumbel approximation, unchecked). | Say "on average, about ninety-eight thousand received". Or show "average 97,876 · unluckiest ≈ 230,000 (model)" after adding an oracle row. |
| D15 | Dev | 08_5 | minor | **The race doesn't say K, and its means (1.6, 0.004) are theory.** In 1,000 trials the byte histogram would have about 4 runs needing a spare, so a live sim shows something like 0.002 or 0.006, not 0.004. | Show K and the seed. Display the run's own counts ("3 of 1,000 needed one spare"), with the theory value labelled "model". |
| D16 | Dev | 11_1 | minor | **"We ran the real thing" — which thing?** It's cberner's `raptorq` 2.0.0, drawing from a fixed pool of 20,000 packets rather than uniform ESIs as §5.8 specifies (fact-sheet caveat). "Ten pieces" invites "toy size". | Caption: "raptorq 2.0.0 (Rust); also 0.63% at 101 pieces and 0.54% at 1,002". Put the pool caveat in the Ruth note. |
| D17 | Dev | 11_5 | minor | **"The standard for next-generation TV broadcasts" has no source.** The fact sheet says ATSC 3.0 (A/331) was not re-checked, and it is narrated. | Re-check A/331 (clause and edition) before narration lock. Card text: "ATSC 3.0 (A/331)", "3GPP MBMS, 2005 (Raptor)", "Linux RAID-6". |
| D18 | Dev | 11_6 vs 11_2 | minor | **11_6 overstates what 11_2 measured.** 11_2 honestly says K+2 is "too rare to measure precisely" (1 failure; 95% interval up to 5.6×10⁻⁶). Then 11_6 says "two extra packets buy a million-to-one" as if measured. And nobody tells me what the unlucky phone does, which is the cheap fountain failure mode. | 11_6: "Two extra packets buy the standard's one in a million, and the rare unlucky phone just waits for one more." |
| D19 | Dev | 09_3 | minor | **"Remove that piece everywhere" is half the implementation.** In code it means XOR its value into every card that uses it and drop it from their lists. You also need a queue of cards with one unknown left. The lower-third gives only the first data structure. | Lower-third: "per piece, a list of the cards that use it; plus a queue of cards with one unknown left (the ripple)." |
| D20 | Dev | 06_9 / 06 panel; 11_5 | delight | **The AES-vs-RaptorQ side-by-side (0x31 vs 0xC1, verified) is great.** One more I verified: the panel's `xtime` is literally the inner step of CRC-8/SAE-J1850 (polynomial 0x1D in the reveng catalogue; looping `xtime` over "123456789" reproduces its check value 0x4B). That would make me say "I've written this before". The fact sheet also verified that QR codes use 0x11D. | Lower-third next to the panel: "Written a CRC? Same move: shift, and if a bit falls off, ex-or in the polynomial. CRC-8/SAE-J1850 uses this very 0x1D." Add "every QR code" to the 11_5 card. |
| D21 | Dev | 03_6 | delight | **The danger meter is the clearest picture of the mechanism.** The chance of a useless card is exactly the lit fraction, 2^(rank−K), so I can see why the risk sits at the end and why K doesn't matter. I'd explain it to a colleague with this picture. | Keep it. Label the axis "lit corners ÷ all corners = chance a card is useless". |
| D22 | Dev | 05_5 | delight | I didn't expect wrap-around bytes to fail exactly as often as XOR, and the race shows it rather than asserting it. | Keep it, with D13's one line. |
| D23 | Dev | 02 lower-third; 11_2 | delight | "Lost, not corrupted… one that slips through would poison the decode" and "too rare to measure precisely" are the honest failure modes I came for. | Keep both word for word. |
