<!-- R1 (storyboard) review of fair_coin by dev_28, verbatim as returned on 2026-10-12; storyboard at commit 2c90b12; absolute file paths made repository-relative. Dispositions: R1.md -->

# R1 storyboard review: fair_coin — Dev, 28

Chapter 10 has one blocker: the spoken recipe for reusing discards cannot produce the numbers shown beside it (D1). Four majors follow (D2–D5); every fix is a word swap or a lower-third, so none adds length.

## Comprehension check

- **01:** An 80%-heads coin makes "call it" an 80/20 bet, and swapping calls doesn't help because nobody knows which way or how far it leans; the coin stands in for any lopsided source.
- **02:** Toss twice and let only heads-tails or tails-heads decide: each is 0.8 × 0.2 = 16%, and the same coin that went 791 to 209 gives a 494 to 506 game.
- **03:** The two deciding pairs are the same two factors in the other order, p(1 − p) = (1 − p)p, so they tie for any p without anyone measuring p, provided both tosses have the same chance and don't affect each other.
- **04:** The price is rejection: only 32% of pairs decide, 6.25 tosses on average with no worst case, and no rule that must name a winner by a fixed toss can be fair for every coin.
- **05:** Break either assumption and the tie goes: two different coins give 40 against 10, and a coin that repeats itself makes the pair after a discard predictable four to one; pairing only tosses that follow a head repairs it.
- **06:** Reordering results never changes their chance, so sort n-toss patterns by head count: the count carries all the bias, and inside a count group every pattern is equally likely.
- **07:** Four tosses with exactly two heads are six equally likely patterns, so that is a fair die, at about 26 tosses a roll because only 15% of blocks qualify.
- **08:** Same move on a loaded die: two different rolls are equally likely in either order, and three all-different rolls come in six equally likely orders, a fair die from a loaded one.
- **09:** Three rolls are a point in a cube sliced by the die's chances; the six orders of 4, 5, 6 are one 0.1 × 0.2 × 0.5 brick turned six ways, so equal volume means equal chance.
- **10:** The trick yields 16 fair tosses per 100, the ceiling (entropy) is 72, and you climb toward it with bigger pattern groups or by rerunning the trick on the pairs you threw away.
  - I ran that last clause and it is wrong as a recipe: it stalls at 18. By the packet's rule a wrong sentence is a blocker for the chapter (D1).
- **11:** It is von Neumann's 1949 trick, he priced it himself (at best 25% as efficient as a fair coin), Intel built a corrector on it in 1999, and the takeaway is to ignore the values and read the order.

## Rubric

| chapter | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 5 | 4 | 5 | 4 | 3 |
| 02 | 5 | 5 | 5 | 5 | 5 |
| 03 | 5 | 5 | 4 | 4 | 4 |
| 04 | 4 | 4 | 4 | 4 | 4 |
| 05 | 4 | 4 | 3 | 3 | 4 |
| 06 | 5 | 5 | 5 | 4 | 5 |
| 07 | 4 | 5 | 4 | 4 | 4 |
| 08 | 4 | 3 | 5 | 4 | 5 |
| 09 | 4 | 5 | 3 | 4 | 3 |
| 10 | 3 | 2 | 4 | 4 | 3 |
| 11 | 4 | 4 | 4 | 4 | 4 |

Correctness scores cover what I recomputed (listed at the end). Cited history is unchecked.

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| D1 | dev_28 | 10_3, 10_4 | blocker | The words say "pick up the pairs you threw away, and run the trick again on them", then "both climb toward seventy-two". I implemented exactly that: it gives 17.9 per hundred at every depth (18.2 on 100,000 simulated tosses) and never moves. The on-screen 28.7, 38.8 and 69.6 come from Peres's real procedure, which recycles two leftovers: whether each pair matched, and what the matching pairs showed. `model.rs` and `oracle.py` both do it that way. | 10_3 words: replace the last sentence with "Or keep what the trick ignored, and run the trick again on that." (shorter). Relabel the second ladder "reusing both leftovers". Lower-third: "for programmers: two leftover streams. a XOR b for every pair, and a for every matching pair. Recurse on both (Peres, 1992)". |
| D2 | dev_28 | 08_3, 08_4, colour table | major | 08_3 gives Alice "first roll bigger" and colours the blocks under the diagonal teal. 08_4 then lifts "one teal block (a five then a six)", which is first roll smaller and sits above the diagonal (face 5 spans 0.3 to 0.5, face 6 spans 0.5 to 1.0). In the coin square teal is also above the diagonal, so the grid-to-coin morph in the hindsight beat would swap colours across the mirror line. | 08_3 words: "Alice wins when the second roll is bigger, and Bob wins when the first is." Picture: blocks above the diagonal teal. Colour table and the 08 Dev note: "second roll bigger" for Alice. 08_4 is then right as written. |
| D3 | dev_28 | 05_6 | major | "The trick is fair again" is true only if nobody sets a deadline, and I always have one (a buffer size, a timeout). Cut off after 8 tosses, heads-first wins 64.7% of decided games (53.9% at 16); I reproduced the fact sheet's finding 4 by enumeration. The plain rule on an independent coin stays at exactly 50.0% under the same cutoff, so this is a new failure the cure introduces: how long a game took leaks who won. The cost is also unstated: 12.5 tosses a decision against 10.0 for the plain rule on the same coin. | Swap the lower-third for "Samuelson, 1968. Fair when played to the end. Stop at a deadline and it leans. Blum, 1986, uses every toss". Add the yield to the counter caption: "200,000 tosses, 15,890 decisions". The 64.7% and the reason (heads-tails finishes in two tosses, tails-heads waits for the coin to return to heads) go in the description. |
| D4 | dev_28 | 04_5, 04_6, 04 Dev note | major | The lower-third "no rule with a fixed number of tosses is fair for every coin" is false as written: "toss twice, otherwise no result" is such a rule and is fair. The Rust comment has the missing words: "and then name a winner". In practice a capped loop that returns no result stays exactly 50/50 on decided games; a cap that falls back to a default winner gives 51.06% at ten pairs and 84% at one. Separately, the rejection-sampling aside in the notes is attached to no row, so nothing will draw it. | 04_6 lower-third: "no rule that must name a winner after a fixed number of tosses is fair for every coin". Attach the aside to 04_5: "for programmers: rejection sampling, a retry loop with no upper bound. If you cap it, the cap returns no result, never a default winner". |
| D5 | dev_28 | 05_4 | major | The first test I'd run on the coin with memory is a win count, and it passes: 49.97% heads-first over 200,059 decisions (fact sheet: exactly 1/2 in the long run). The failure is in the sequence: each result differs from the last 61.6% of the time (fact sheet 61.5%). That is the classic way a correlated source gets through a frequency test, and the storyboard never says the tally stays level. "Knows who's about to win" also overstates a four-to-one tip. | Lower-third at 05_4: "for programmers: count the wins and this coin still looks fair, 50 to 50. The lean is in the sequence: each result predicts the next". Words: "can bet on who's about to win". |
| D6 | dev_28 | 08_3, 08_5 | minor | "If the two rolls match, roll again" doesn't say both. Keeping the first roll gives Alice 61.1% (exact 1393/2280). 08_5 has no reject branch at all; sliding by one roll until three differ gives orders from 8.7% to 22.8% where fair is 16.7%. The coin chapters got this right ("toss two more", "toss four more"). | 08_3: "roll two more". 08_5: reuse 07_2's grey "roll three more" loop arrow on the picture. |
| D7 | dev_28 | 02_7 | minor | The bug I'd most likely write is a sliding window: keep the second toss of a discarded pair and compare it with the next. That returns the first toss, 80% heads (79.9% in 200,000 simulated games). The words guard against it, but the hazard is never named. | Lower-third at 02_7: "for programmers: fresh tosses for every pair. Reuse a toss from a thrown-away pair and you get the first toss back". |
| D8 | dev_28 | 07_5 | minor | "Three tosses that aren't all alike are a fair one in three" doesn't say which one in three, and there are exactly six such patterns sitting there looking like die faces. Mapped directly they come out 26.7% three times and 6.7% three times. The fair choice is the position of the odd toss (I checked: 1/3 each); which group you're in is 4/5 against 1/5. The 12.5 itself is right. | Lower-third: "for programmers: in three tosses that aren't all alike, the odd one out is equally likely to be first, second or third. That fair 1 in 3 plus one coin trick makes a die for 12.5 tosses". |
| D9 | dev_28 | 11_4 | minor | "Computers took him at his word" generalises from one 1999 white paper, and it reads like a pitch. It also leaves chapter 05 hanging: a noisy circuit is exactly the kind of source I'd expect to have memory. The fact sheet holds an honest cost that goes unused: one bit for every 6 raw samples, which sits right beside our coin's 6.25. | Words: "One chip maker took him at his word." Card: add "one bit out for every 6 raw samples" (`E-C4.intel.raw_samples_per_bit`). I believe newer generators condition with a cipher or hash instead, but I did not check; do not put that on screen without a source in the fact sheet. |
| D10 | dev_28 | 01_5, 05_2, 05_3 | minor | 01_5 promises the circuit and 11_4 pays it off, but 05's two breaks are toys (who tosses two different coins?). For me they map straight onto real sources: a chance that drifts between the two samples is "two coins", and sampling faster than the noise changes is the coin with memory. | Captions only, no words: 05_2 "the same as a source that drifts between the two tosses"; 05_3 "the same as a circuit sampled faster than its noise changes". Both are claims about real systems: add a source to the fact sheet or keep them in the description. |
| D11 | dev_28 | 10_1, 10_2, 10_3 | minor | "A hundred tosses… seventy-two" is a long-run rate. From exactly 100 tosses, Elias on one block averages 66.9, and I get a ceiling of 68.1 for any rule that is fair for every coin. The climb's cost is also unstated: block 64 holds 64 tosses before anything comes out and ranks among 1.8 × 10^18 patterns, and the 69.6 needed all 100,000 tosses in hand. | 10_2 lower-third: "per hundred tosses, over a long run. Name to search: entropy". 10_3: one caption under the ladders, "the price: memory and waiting". |
| D12 | dev_28 | 04_3, 04_4, 04_5, 04_6 | minor | Units wobble. The histogram has one bar per number of pairs but its mean marker reads "6.21 tosses", which would sit at 3.1 on that axis. 04_5 counts ten pairs and 04_6 ten tosses in the next breath. "A fair coin would need four tosses" drops "on average". | Label the histogram axis in tosses (2, 4, 6, …). 04_6: "twenty tosses" and twenty discs, matching 04_5's ten pairs. 04_4: "A fair coin still averages four." |
| D13 | dev_28 | 03_6 | minor | The best engineering point of the first act is only in the notes: nothing is calibrated. My instinct was "measure the bias over a thousand tosses and correct for it", which needs a stored constant that goes stale. | Lower-third at 03_6: "for programmers: no calibration step and no stored constant". |
| D14 | dev_28 | 03_4, 05_6, 04_3 | minor | Some seeded counts look like leans to a suspicious eye: 472 at the 90% stop, 7,847 against 8,043 right under "fair again", and the fourth histogram bar (926 where the model says 1,006). I checked: the first two are −1.77 and −1.55 standard errors, and the histogram's chi-squared is 12.25 on 8 degrees of freedom. All ordinary, but the picture gives me no way to know that. | Draw a faint two-standard-error band on the tiles and counters (469 to 531 for 1,000 games). No words. |
| D15 | dev_28 | 05_5 | minor | The same-side bias is memory only if each toss starts the way the last one landed; the picture shows the thumb but never makes that link. This is also the beat I'd trade if 05 runs long: six beats and four ideas in 75 s is the densest chapter. | Card: add "when each toss starts the way the last one landed". If 05 needs seconds, make 05_5 a card over 05_4's hold with no narration. |
| D16 | dev_28 | 09_1 | minor | I already believed the six orders from 08_4's flip; the cube gave me nothing to implement and 09_1 doesn't say why I'm looking. | 09_1 words: open with "The same flip, one dimension up." in place of "Here's the picture behind that." If length is needed, this is the chapter where I lose nothing. |
| D17 | dev_28 | 07_2, 08_6 | minor | Two asides I'd have liked, both lower priority than everything above: the die-from-a-coin loop is the same reject-and-retry that fixes `rand() % 6`, and n! orders is what a sort decides. | Only if a lower-third slot is free: 07_2 "the same reject and retry that fixes rand() % 6"; 08_6 append "the order is what a sort finds". Do not trade D1–D8 for these. |
| D18 | dev_28 | 02_8, 02_9 | delight | It runs before it explains: 791 to 209 above 494 to 506, same coin. I got 49.90% heads-first and a mean of 6.250 tosses over 200,000 games on my own generator. | Keep. |
| D19 | dev_28 | 04_6 | delight | A whole impossibility proof in two sentences that I could repeat at a whiteboard: all-heads has to belong to someone. | Keep the argument; fix only the lower-third (D4). |
| D20 | dev_28 | 05_4 | delight | "One change of face… two" is the mechanism, not a formula; I wrote the simulation from that sentence and got 3.99 to 1 after two heads. | Keep. |
| D21 | dev_28 | 06_7, 08_4 | delight | "The counts carry the unfairness. The order is perfectly fair." and "A coin is just a die with two faces." are the two lines I'd repeat to a colleague. | Keep both verbatim. |
| D22 | dev_28 | 11_5 | delight | The receipts card is not a pose: I re-ran `oracle.py` and every seeded count the storyboard quotes came out the same. | Keep the repository link on the card; add the seed (1951) beside it so a viewer can replay the runs. |

If only four fixes are taken: D1, D2, D3, D4.

## What I checked and how

- **My own scripts** (exact fractions plus Mersenne Twister simulations, independent of the oracle's code and generator): `dev_check.py` and `dev_check2.py` in the session's scratch directory (outside the repository).
- **Recomputed exactly, all matching the storyboard:**
  - The coin: 64/16/16/4; 32% decides; 6.25, 4, 11.11 and 101.01 tosses; 2.1% undecided after ten pairs (one in 47); two coins 40 against 10; sticky coin 16 against 4.
  - Patterns: 12.8%; row-four stretches; 15.36% and 26.04 tosses; 48% and 12.5 tosses.
  - Dice: ties 31.5%, 2.92 rolls a decision; each order 5.425%; all three differ 32.55% (the sheet rounds to 32.6), 9.22 rolls; 120 of 216 boxes; brick 1%.
  - Rates: entropy 0.7219; Peres 16.0, 28.8, 38.7, 46.5, 63.5; Elias 16.0, 28.2, 49.4, 64.6.
  - Samuelson under a deadline: 64.7% and 53.9% starting after a head (67.3% and 55.7% from a long-run start).
- **Simulated:**
  - Plain rule and Samuelson's rule on 2,000,000 sticky tosses. Samuelson came out 49.79% heads-first, with successive results differing 49.97% of the time.
  - Die from four tosses, 26.02 tosses a roll.
  - Peres on 100,000 tosses: 70.3 at full depth, against the video's 69.6 on its own seed.
  - Blum's first two results: 0.15, 0.149, 0.349, 0.352 announced at once; 0.25 each announced on return.
  - The wrong implementations in D1 and D6–D8.
- **Re-ran the project's oracle** from the scratch directory (`python3 -I -B`, NumPy 2.4.1): 265 facts, 43 cross-checks, 0 failed, and every seeded count in the storyboard matches its output. That shows the storyboard quotes the oracle; it is not an independent check of those counts. Independently I only checked they are statistically plausible (D14).
- **Read:** `explainer_corpus/fair_coin/oracle.py` in full, and lines 270–306 and 570–803 of `explainer_corpus/fair_coin/src/model.rs`.
- **Unchecked:**
  - Every cited literal: the von Neumann quote and page, Jun and Kocher, Gelman and Nolan, Diaconis and others, Bartoš and others, the penny counts, all years.
  - The E-X1 rule counts (I read the code and checked the 04_6 argument by hand, not the enumeration).
  - The Rust tests and `./x fair_coin facts-check`, which I did not run.
  - Whether Elias's rule is the best possible at exactly 100 tosses (D11 gives my bound, not a proof of optimality).
- No file in the repository was edited, created or deleted.
