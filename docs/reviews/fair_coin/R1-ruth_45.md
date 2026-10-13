<!-- R1 (storyboard) review of fair_coin by ruth_45, verbatim as returned on 2026-10-12; storyboard at commit 2c90b12; absolute file paths made repository-relative. Dispositions: R1.md -->

# R1 review: fair_coin storyboard, by Ruth Adler (45)

I read the brief, the storyboard and the fact sheet as a working mathematician, and I ran `ruth_audit.md` over every line of draft words. The numbers are in very good shape. Every value I recomputed matches the oracle. The problems are in the words: one sentence names a recycling procedure that can't reach the numbers shown, several universal claims go false at an edge the video itself shows, and one term is defined wrongly.

## What I checked, and how

**Recomputed.** I used `python3 -I` with exact `Fraction`s unless noted. Scripts are in `scratchpad/ruth/`.
- **The square at 80%:** 16/25, 4/25, 4/25, 1/25. A pair decides 8/25 of the time.
- **Average tosses, 1/(p(1−p)):** 4 at 1/2, 25/6 at 3/5, 100/21 at 7/10, 25/4 at 4/5, 100/9 at 9/10, 10000/99 (101.01) at 99/100.
- **Long games:** 0.68¹⁰ = 2.114%, which is one game in 47.3. The chance that the longest of 10,000 games runs at least 30 pairs is about 13%. Three standard errors on 1,000 games are 47.4.
- **Fixed-length rules (04_6):** the counts of possible rules are 4, 12, 64, 700 and 17,424. I brute-forced every one at n+1 rational p for n ≤ 5, and none is fair for every coin. If draws are allowed, a two-toss rule is equal for every coin.
- **Two coins (05_2):** 2/5 against 1/10, so Alice wins 4/5.
- **The coin with memory (05_4):** after a head, the pairs HH, HT, TH, TT have chances 16/25, 4/25, 1/25, 4/25. So "one change of face against two" is a correct account: HT is a repeat (4/5) times a change (1/5), TH is two changes, and the ratio is 4.
  - The plain rule played to the end: Alice wins 8/13 after a head, 5/13 after a tail, and 1/2 from a long-run start.
  - The lopsided memory: 7/12 after a head, 5/12 after a tail, 5/9 from a long-run start.
- **05_6's cure (every toss that follows a head, paired up):** I solved the Markov chain exactly for three different memories. It is exactly 1/2 from either start when played to the end.
  - Under a deadline it leans. Starting after a head, heads-first wins 64.7%, 53.9% and 50.5% of the games decided within 8, 16 and 32 tosses, which reproduces fact-sheet finding 4. Starting after a tail it is 72.6%, 57.8% and 51.1%.
- **Same counts, same chance:** holds for every pattern up to n = 6 at p = 4/5. Row 4 is 0.16, 2.56, 15.36, 40.96, 40.96 percent. HHT, HTH and THH are 0.128 each.
- **A die from four tosses:** it works 96/625 of the time and costs 625/24 = 26.04 tosses a roll. The three-toss route costs 12/25 acceptance, then 12.5 tosses in all. The chi-squared of the faces 166…159 is 4.06 on 5 degrees of freedom.
- **The loaded die (5, 5, 10, 10, 20, 50 percent):**
  - Two rolls tie 63/200 of the time; the first is bigger 137/400.
  - Three rolls all differ 651/2000 of the time. Listing every outcome, all six orders are 217/4000 each.
  - For n = 4, listing every outcome gives 24 orders at 173/40000 each. All n distinct needs 38.5 rolls on average at n = 4, 249 at n = 5 and 3,333 at n = 6.
  - The 4-5-6 brick has volume 1/100. 120 of the 216 boxes have three different faces.
- **Entropy and the two climbing rules:**
  - h(0.8) = 0.721928. It is a fixed point of Peres's recursion to 1e-16 at five values of p.
  - Peres's recursion by depth: 0.1600, 0.2876, 0.3874, 0.4655, 0.5721 (depth 6), 0.6351 (depth 8), and 0.7202 at depth 22.
  - Reusing only the thrown-away pairs levels off at **0.1794**. Reusing only the record of which pairs matched levels off at 0.3927.
  - Elias by block size: 2 → 0.16, 64 → 0.6462, 4096 → 0.7200.
  - Elias on a single block of 100 tosses averages 66.93 fair tosses but gives **more than 72 in 24% of runs**.
- **Dijkstra:** (2³⁷−2)/37 = 3,714,566,310.
- **Seeded runs.** I replayed runs 1, 2, 4 and 5 with my own loop on NumPy's PCG64DXSM, seeds 195101, 195102, 195104 and 195105:
  - Run 1: 791 heads.
  - Run 2: 494 for Alice, 6,256 tosses, longest game 22 pairs.
  - Run 4: mean 6.2076 tosses, longest 30 pairs, histogram 3209…205.
  - Run 5: 807 for Alice.

  I also ran `oracle.py`: 265 facts, 43 cross-checks, 0 failed.

**Sources I checked.**
- **Von Neumann, p. 36 (page image `gb_PA36.png`):** the 11_1 and 11_3 quotations are verbatim. "State of sin" is on the same page. The title page says "Issued June 11, 1951", and the preface says UCLA, June 29, 30 and July 1, 1949.
- **Diaconis, Holmes and Montgomery:** the abstract says ".51". Page 218 says some students' spun coins came up "fewer than 10% heads". Page 231 says the mean was 0.508, rounded up to .51.
- **Bartoš:** 0.508, interval [0.506, 0.509], 350,757 flips. Each flip in their study started from the side the coin had just landed on.
- **Jun and Kocher, §4.3:** the card's words are verbatim, and the text says "Intel has enhanced…". Raw bit frequencies were about .5 ± 10⁻².
- **Gelman and Nolan:** the abstract matches. **Chance News:** 1878/5520 matches.
- **Peres:** I checked the definition (*) on p. 591 (it recycles **two** sequences) and Propositions 2 and 3 (pp. 593–594).
- **Elias, pp. 865–868:** the Samuelson process, with the best mapping "credited to John W. Pratt"; composition classes on p. 868; Theorems 1 and 2.
- **Others:** Stout and Warren p. 212 and Theorem 3; Hoeffding and Simons p. 341; Vadhan §6.5; Samuelson's abstract through OpenAlex; Dijkstra's EWD 1071.

**Unchecked.**
- I did not replay E-S3 or E-S6 to E-S9 myself; those are the oracle's word only. The same goes for Blum's E-F7 numbers.
- I did not see Samuelson's own pages (paywalled) or Blum's 1986 journal text.
- I did not read `src/model.rs`, and I did not check Juels et al.

## Audit checklist (draft words)

| item | result |
|---|---|
| Quantifiers explicit | **Fails** at 04_4, 10_2 (missing "on average") and at 05_4, 06_2, 06_6 (claims stated too broadly): R2, R4, R5, R12 |
| Always/never true at its edges | **Fails** at 06_2 and 06_6 (the coin with memory), 10_4 (72.0), 04_6's lower-third (rules that allow draws) and 04_1 (p = ½): R2, R4, R8, R12 |
| Every probability names what is random | Mostly passes. 05_5 (it is the starting face, under a particular way of tossing) and 10_2 (one run against the average) fail: R10, R2 |
| No probability put on a fixed unknown | Passes. The 05 note correctly calls Bartoš's interval Bayesian |
| Standard terms | **Fails**: 03_7 calls "same chance and doesn't affect" *independent* (R3) |
| Exact version kept somewhere | **Missing** for "ends with probability one" (R6), "on average" at 10_2 (R2) and Samuelson's scope (R13) |
| Historical claims | Quotes are verbatim and dates are right. "In his words" (R17), Blum's year (R13) and Intel's "lopsided" (R18) need fixes |
| Numbers equal the oracle, voice and screen rounded alike | All match the oracle. Voice and screen disagree at 04_5 (1 in 50 against 2.1%) and 10_2 (72 against 72.19): R12, R2 |
| Edge cases acknowledged | p = 0 and 1 are handled (03_7, 11_5). n > 6 at 08_6 is not (R15) |
| Notation consistent | p and 1 − p are used consistently. Alice's side of the diagonal flips between the coin square and the die grid (R14) |
| Nothing true but misleading | 05_5, 01_5's lower-third, 11_4 and 10_3: R10, R11, R18, R1 |

## Comprehension check

- **01:** "Call it" with an 80% coin is a bet on the coin, not a fair game. The question is whether that same coin, with nobody knowing its chance, can settle one bet exactly fairly. (As drafted, 01_4 gives the wrong reason why swapping calls fails: R7.)
- **02:** Toss twice and let heads-then-tails and tails-then-heads decide. Both blocks are 0.8 × 0.2 = 16%, so the game is even though the coin isn't, and the seeded thousand games come out 494 to 506.
- **03:** The deciding blocks are p(1−p) and (1−p)p, mirror images across the diagonal, so they are equal for every p and nobody needs to know p. That holds provided the two tosses of a pair have the same chance and don't influence each other.
- **04:** Only 2p(1−p) of pairs decide, so a game averages 1/(p(1−p)) tosses: 6.25 here, about 101 at 99%. There is no longest game, and no rule that always names a winner within a fixed number of tosses can be fair for every coin.
- **05:** The fairness *is* the mirror symmetry. Two different coins break it (40% against 10%), and so does a coin that remembers its last face (16% against 4% after a thrown-away two heads). Using only the tosses that follow a head restores it.
- **06:** For independent tosses of one coin, a pattern's chance depends only on how many heads it has. So the counts carry the unfairness, and given the counts every order is equally likely. Pascal's triangle counts those equally likely groups.
- **07:** Four tosses with exactly two heads give six equally likely patterns, which is a fair die from an unfair coin, at about 26 tosses a roll.
- **08:** For a loaded die, different rolls are equally likely in every order, two rolls either way round and three rolls in all six orders. Throwing out ties leaves a fair die, at about 9 rolls each here.
- **09:** The six orders of a 4, a 5 and a 6 are one 0.1 × 0.2 × 0.5 brick turned six ways inside the cube, so they have equal volume and therefore equal chance.
- **10:** The trick gets 16 fair tosses per hundred from this coin. No rule *averages* more than the entropy, about 72.2. Elias's bigger groups and Peres's recycling climb toward that ceiling without reaching it.
- **11:** Von Neumann gave the trick and its at-best 25% efficiency in 1949 (printed 1951), and Intel's 1999 generator used a corrector based on it. The closing square shows the reason again: the order is fair even when the counts are not.

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 4 | 3 | 4 | 4 | 3 |
| 02 | 5 | 5 | 5 | 4 | 4 |
| 03 | 5 | 3 | 4 | 4 | 5 |
| 04 | 4 | 3 | 4 | 4 | 4 |
| 05 | 4 | 3 | 5 | 3 | 5 |
| 06 | 4 | 3 | 4 | 4 | 4 |
| 07 | 5 | 4 | 4 | 4 | 4 |
| 08 | 4 | 4 | 5 | 4 | 4 |
| 09 | 4 | 5 | 3 | 5 | 5 |
| 10 | 3 | 2 | 4 | 3 | 3 |
| 11 | 4 | 4 | 4 | 4 | 4 |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| R1 | Ruth | 10_3 | blocker | "Pick up the pairs you threw away, and run the trick again on them" describes only half of Peres's procedure. Peres (p. 591, definition (*)) recycles two sequences: the value of each matching pair, *and* for every pair whether it matched. Recycling the thrown-away pairs alone levels off at 0.1794, about 18 per hundred, never 72. The bars drawn (28.7, 38.8 → 72) and the measured 69.6 come from the full procedure, which recycles both. As worded, a student who codes what they heard gets 18. | Change the words, same length: "Or keep two things the trick throws away. What each matching pair showed, and whether each pair matched. Run the trick again on both." Relabel the ladder "reusing what the trick ignored", not "reusing the discards". The picture can split the row into its two recycled rows; no extra time. |
| R2 | Ruth | 10_1, 10_2, 10_4 | major | **10_2 needs a qualifier.** "No rule beats it" is true only on average. Elias's rule on one block of 100 tosses of this coin averages 66.9 but gives **more than 72 in 24% of runs**. Separately, the ceiling is 72.19, not 72. Peres passes 72.0 at depth 22 (72.02) and Elias at block 4096, so 10_4's "never quite arrive" at seventy-two is false at the spoken precision. 10_1's "the best possible rule" presupposes a best rule, and none exists, because the bound is approached but never reached. | 10_1: "How many could any rule get, on average?" 10_2: "Just over seventy-two, on average. That ceiling is called the coin's entropy, and no rule beats it on average." Label the green line "72.2 (model)". 10_4: "Both climb toward the ceiling, and never quite reach it." Lower-third: "average fair tosses per 100, for any rule whose output is exactly fair: at most 72.19 (Elias 1972, Theorem 1)". |
| R3 | Ruth | 03_7 | major | "Tosses like that are called independent" gives the name *independent* to *same chance and no influence*. Independent means only the second. With this definition, 05_1's two coins, which are independent, would look like the trick's conditions failing on independence, which they don't. The lower-third's "precisely:" also overclaims: the conditions are enough, not necessary. Swapping a pair's two tosses leaving its chance unchanged is enough (fact-sheet finding 3; Peres's Proposition 1 covers exchangeable bits). | "The coin's chance is the same on both tosses of a pair. And one toss doesn't affect the next, which is what independent means." Lower-third: change "precisely:" to "enough:" and keep the rest. The exchangeable version goes in the description, as the fact sheet already words it. |
| R4 | Ruth | 06_2, 06_6 (06_7) | major | "Order never changes a chance" and "for any coin at all" are universal claims, and chapter 05 just showed a coin where order *does* change a chance: 16% against 4% after a thrown-away two heads. A careful viewer sees the contradiction one minute later. The fact sheet's "Simplification → exact" row quotes "the order carries no trace of the coin's chance", which is not the storyboard's sentence (06_7). | 06_2: "And when one toss doesn't affect the next, order never changes a chance." 06_6: "…every pattern is equally likely, however unfair the coin." Change the fact-sheet row to quote 06_7, "The order is perfectly fair", with the exact version "given the counts, every arrangement is equally likely, for independent tosses of one coin". |
| R5 | Ruth | 05_4 | major | "Anyone who saw the pair that was thrown away knows who's about to win" claims too much. After two heads, Alice wins the *next deciding pair* 4 times in 5 and the *game* 8 times in 13 (61.5%). That is a known lean, not a known winner, and the next pair may not decide at all. | "Anyone who saw the pair that was thrown away knows which way to bet." Description: give the 80%, 8/13 and 5/13 figures, and note that from an unknown long-run start the game is still even. That is exactly why this break is sneaky. |
| R6 | Ruth | 04_5 | major | Is 04_5 stated correctly? It is true but only half the statement. For every n the chance a game is still going after n pairs is positive, but the narration never says the game ends with probability one. "Perfectly fair" (01_1) and "exactly" (02_6) depend on that half. "No game is sure to end…" also puts the quantifier on games, when it belongs on toss numbers. | "The chance that a game never ends is zero. But no toss is late enough to be sure it's over." Lower-third: "ends with probability 1; still going after n pairs with chance 0.68ⁿ > 0". |
| R7 | Ruth | 01_4 | major | The logic is off. Giving Bob heads never makes "call it" fair, even when the coin is known; it only moves the edge to Bob. Alternating callers across bets is even over two bets with no knowledge at all, so "only helps if they know which side the coin favours" is wrong under either reading. The real reason the trick matters: one bet, and whoever knows the coin better picks the winning side. | "Swapping, so Bob calls heads, just hands him the edge. And if either of them knows the coin better, that one picks the winning side." Rewrite the "misconception defused" note to match: fair means fair whatever the coin, and whatever either player knows about it. |
| R8 | Ruth | 04_6 | minor | The lower-third "no rule with a fixed number of tosses is fair for every coin" is false without a missing condition. "Toss twice; HT Alice, TH Bob, otherwise nobody wins" gives Alice and Bob the same chance for every coin. The spoken argument assumes someone wins on ten heads. I confirmed E-X1's counts: 4, 12, 64, 700, 17,424, none fair. | Lower-third: "no rule that always names a winner within a fixed number of tosses is fair for every coin". Spoken: "always finish within ten tosses". |
| R9 | Ruth | 11_5 | minor | Is 11_5 stated correctly? "As long as it can land both ways" is the right condition on p, the open interval 0 < p < 1, and even the coin with memory has a fair coin inside it (Samuelson). But "ignoring what came up, and looking only at the order" contradicts itself: the order is the order of what came up. As a recipe it also holds only for independent tosses. | "You find it by setting aside how many heads came up, and looking only at the order." The vocabulary already has *counts*, so "setting the counts aside" also works. Independence is carried by R22's card. |
| R10 | Ruth | 05_5 | minor | "Real coins have a touch of this", where "this" is memory, misleads. Diaconis, Holmes and Montgomery measured dependence on the *starting* face. That becomes memory only if each toss starts the way the last one landed, which is the procedure Bartoš et al. actually used ("autocorrelated" in their text) and which Kerrich used before them. | "Real coins have a touch of this, when each toss starts the way the last one landed." |
| R11 | Ruth | 01_5 | minor | The lower-third "tossed and caught, a coin can't be biased" overstates Gelman and Nolan, who say it can't be *weighted* to be *substantially* more likely to land heads. 05_5 then shows a tossed-and-caught coin with a 51% same-side lean. The caption's 34% heads doesn't support "this unfair" (80%). DHM p. 218 does: some spun coins came up fewer than 10% heads. | Lower-third: "weighting can't make a tossed-and-caught coin favour a side: Gelman and Nolan, 2002". Optional caption line: "some spun coins: under 10% heads (Diaconis, Holmes and Montgomery, 2007, p. 218)". |
| R12 | Ruth | 04_1, 04_4, 04_5 | minor | 04_4's "would need four tosses" and "needs about a hundred and one" drop "on average" (brief rule 4), right where the chapter is defusing "it takes two tosses". 04_1's "most pairs" is false at p = ½, where it is exactly half; the tested sentence says "at least half". At 04_5 the voice says "one game in fifty" while the screen says 2.1% and the fact key says 1 in 47. | 04_4: "would average four tosses … averages about a hundred and one." 04_1: keep "With our coin" in front of "most pairs". 04_5: "About two games in a hundred are still going after ten pairs." |
| R13 | Ruth | 05_6, 11_5 | minor | 05_6's lower-third doesn't say what the cure covers: it is exact only for a coin that remembers just its last toss. It cites "Blum, 1986", but only the 1984 FOCS text was seen, and fact-sheet finding 1 found an inconsistency in that very text. Samuelson's own pages were not seen; Elias (p. 866) credits the best of Samuelson's rules to John W. Pratt. | Lower-third: "for a coin that remembers only its last toss: Samuelson, 1968. Using every toss takes more care: Blum, 1984". Do the same on the 11_5 receipts card. Put Pratt and the "not seen" note in the description. |
| R14 | Ruth | 08_3, 08_4 | minor | In the coin square, Alice's block (heads then tails) is above the diagonal. In the die grid, her blocks ("first roll bigger") are below it. The 08_4 morph between the two will flip her colour across the line, which undercuts "a coin is just a die with two faces". | 08_3: "Alice wins when the first roll is smaller, and Bob wins when it's bigger." Teal above the diagonal; colour roles "(heads then tails; first roll smaller)", with heads as the low face. |
| R15 | Ruth | 08_5, 08_6 | minor | "Smallest first, biggest first, and so on" names two orders with each phrase. "It keeps going" stops at six for a six-sided die, and the price climbs fast: with this die, all different needs 38.5 rolls on average at four, 249 at five, 3,333 at six. | 08_5: "Low, middle, high. High, low, middle. Six orders in all, equally likely." 08_6 lower-third: "n different rolls, up to six: n! equally likely orders". Add oracle keys if any of these costs is shown. |
| R16 | Ruth | 07_5 | minor | The lower-third's "three tosses that aren't all alike are a fair one in three" doesn't say what is chosen; it is the position of the odd one out (each 4/25, checked). The 07 note says Stout and Warren "proved there is no best recipe". Their Theorem 3 is about producing a fair *toss*: no tree algorithm is cheapest for every p at once. It is not about dice. | Lower-third: "the odd one out of three tosses that aren't all alike is a fair one in three…". Description: "for a fair toss, no recipe is cheapest for every coin at once (Stout and Warren 1984, Theorem 3)". |
| R17 | Ruth | 11_3 (11_1) | minor | "In his words": the printed text is Forsythe's summary, written in von Neumann's first person. The card credits Forsythe, but the voice attributes the exact words to von Neumann. | "In the words of the printed summary, at most twenty-five percent as efficient as ordinary coin tossing." |
| R18 | Ruth | 11_4 | minor | "Raw, lopsided bits": Jun and Kocher report raw frequencies of about .5 ± 10⁻² under extreme conditions, and Intel "enhanced" the corrector to deal with correlations. An 80/20 stream on screen would overstate the raw bias. | "ran its raw, slightly lopsided bits". Draw the stream nearly balanced, or label it "bias exaggerated". |
| R19 | Ruth | 03_4 (03_6) | minor | The 80% tile reads 509 one chapter after 02_8 showed 494 at 80%, with nothing saying it is a fresh run. The sweep never goes below 50%, yet 03_6 says nobody needs to know "which side it favours". | Tiles: "a fresh 1,000 games at each stop". Let the slider dip to 20% once; the readout is symmetric, so no new facts are needed. |
| R20 | Ruth | 02_4, 03_7 | minor | The square quietly assumes independence: the second cut runs straight across both columns. 02_4 credits the cut only to "the same coin". | Picture only. At 03_7, point card one at "second cut at the same mark as the first" and card two at "one straight cut across both columns". Then 05_2's moved cut is visibly card one failing, and 05_4's staggered cut is card two failing. |
| R21 | Ruth | 05_4 | delight | The staggered square is the best picture of conditional chance I have seen for this. One change of face against two is exactly 0.8 × 0.2 against 0.2 × 0.2. | Keep it, and write those two products in the blocks, as 02_6 does. |
| R22 | Ruth | 11_1 | delight | Von Neumann names the trick's condition himself, in the sentence just before the card's quote: "If independence of successive tosses is assumed, we can reconstruct a 50–50 chance out of even a badly biased coin by tossing twice." | Start the card one sentence earlier, keeping the printed dash in "50–50". That ties 03_7 to 11 and carries 11_5's missing condition without a spoken word. |
| R23 | Ruth | 05_5 | delight | DHM's own unrounded figure was 0.508 (p. 231: "rounded this up to the 0.51 quoted"), from 27 filmed tosses. Bartoš et al. measured 0.508 from 350,757. | On the card, in small type under the DHM line: "(their mean: 0.508)". |
| R24 | Ruth | 04_6 | delight | A sharper and very short form of the proof: a rule sure to finish by toss n can be exactly fair for at most n coins. Alice's chance minus ½ is a polynomial of degree at most n, and it isn't zero because it equals ±½ at p = 1. | Description only. |
| R25 | Ruth | 03_7 | delight | "The same chance on both tosses" is only needed *within* a pair. The coin may drift from one pair to the next and the trick stays exact, which matters for hardware sources. | Description only, one sentence. |
