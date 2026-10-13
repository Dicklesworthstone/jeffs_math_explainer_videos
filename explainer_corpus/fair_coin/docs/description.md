<!--
Draft video description for Video E (P3, 2026-10-12). Every statement below
rests on a row of docs/fact-sheet.md or an oracle key; the bracketed tags
name them. It holds the exact versions of what the narration simplifies
(plan rule 11), the programmer notes that have no room on screen, and what
review round R1 sent "to the description". The release package adds the
chapter timestamps and the repository link, and trims this to what the upload
page accepts, linking this file for the rest.
-->

# How to Make a Fair Coin Out of an Unfair One

Toss an unfair coin twice. If the two results match, throw them away and toss
two fresh ones. If they differ, heads-then-tails wins for one player and
tails-then-heads for the other. The game is exactly fair whatever the coin's
chance of heads, and nobody has to know that chance. This video shows why,
what it costs, how it breaks, and how the same idea makes a fair die out of a
loaded one.

## Chapters

(Timestamps come with the finished cut.)

1. Call it
2. Toss twice
3. Every coin
4. The price
5. Where it breaks
6. Same counts
7. A die from a coin
8. Loaded dice
9. Six orders
10. What is left
11. The fair coin inside

## Precisely

What the narration says in plain words, stated exactly.

**The coin.** The 80% coin is a model. A coin that is flipped and caught
cannot be made to favour a side by weighting it (Gelman and Nolan 2002); a
coin that is spun, or allowed to bounce, can. In one instructor's classroom
records, 1960s pennies spun on a table landed heads 1,878 times in 5,520
[E-C3.pennies]; Diaconis, Holmes and Montgomery report students whose spun
coins "came up fewer than 10% heads" (2007, p. 218). The often-repeated
figure of 80% tails for a spun penny has no data behind it that we could
find, and the video does not use it.

**The trick (chapters 2 and 3).** For two independent tosses with the same
probability p of heads, P(heads then tails) = p(1 − p) = P(tails then heads),
for every p in [0, 1] [E-F1, E-F4]. The game needs 0 < p < 1: a coin that
cannot land both ways never produces a differing pair. Independence with one
shared p is enough, not necessary: all the trick uses is that swapping the
two tosses of a pair leaves the pair's probability unchanged. The chance may
even drift from one pair to the next, as long as it is the same within each
pair.

**The price (chapter 4).** A pair differs with probability 2p(1 − p), so the
number of pairs is geometric and the expected number of tosses is
1/(p(1 − p)): 6.25 at p = 4/5, 4 at p = 1/2, about 101 at p = 0.99 [E-F3,
E-F4]. The game ends with probability one, but for every n it is still going
after n pairs with probability (1 − 2p(1 − p))ⁿ > 0; at p = 4/5 that is 2.1%
after ten pairs [E-F3]. No rule that always names a winner within a fixed
number n of tosses is fair for every coin: at p near 1 the all-heads pattern
has nearly all the probability, and it belongs to one player. (More sharply:
the first player's chance minus one half is a non-zero polynomial of degree
at most n in p, so such a rule can be exactly fair for at most n coins.) A
rule that may end with no result is a different thing: "toss twice, and if
they match nobody wins" is fair for every coin. Checked for every rule on up
to five tosses [E-X1].

**Two coins (chapter 5).** With chances p₁ then p₂, the two differing pairs
have probabilities p₁(1 − p₂) and (1 − p₁)p₂, equal exactly when p₁ = p₂.
With a fair second coin the first coin's chance comes straight back: 0.40
against 0.10, four to one [E-F5].

**A coin with memory (chapter 5).** The model is a two-state Markov chain
that repeats its last result with probability 4/5; its long-run share of
heads is exactly one half. After two heads, heads-then-tails has probability
4/5 × 1/5 and tails-then-heads 1/5 × 1/5 [E-F6]. Played to the end from
there, the plain trick gives the heads-first player 8/13, and 5/13 after two
tails; from a long-run start it is exactly one half, and so is the first pair
of a long run. That is why a tally of wins does not show this break: the
winners come out half and half, but each result predicts the next.

Real tossed coins have a little of this. A flipped coin lands the same way up
as it started about 51% of the time (Diaconis, Holmes and Montgomery 2007,
whose own unrounded figure was 0.508); 350,757 recorded flips gave 0.508,
with a 95% credible interval of 0.506 to 0.509 (Bartoš and others, 2025).
That is a memory only when each toss starts the way the last one landed.

**The cure.** For a coin that remembers only its last toss, the tosses that
come straight after a head are independent with one shared chance, so the
trick applied to those alone is exactly fair when played to the end
(Samuelson 1968, as described by Elias 1972, who credits the best form of it
to John W. Pratt; Samuelson's own pages were not seen for this video). Under
a deadline it is not fair: starting straight after a head, of the games
decided within 8 tosses of this coin the heads-first player wins 64.7%,
because heads-then-tails can finish in two tosses while tails-then-heads must
wait for the coin to return to heads.

**Same counts (chapter 6).** For independent tosses of one coin, a pattern's
probability depends only on how many heads it has: p^k (1 − p)^(n−k). So,
given the counts, every arrangement is equally likely. This fails for a coin
with memory, which is why the narration fences it. The groups are what Elias
calls composition classes; their sizes are the binomial coefficients, the
rows of Pascal's triangle.

**Dice (chapters 8 and 9).** For independent rolls of one die, the n! orders
of n different values are equally likely, whatever the die's chances. With
the video's die (5, 5, 10, 10, 20 and 50 percent) two rolls differ 68.5% of
the time, 2.9 rolls for each fair decision; three rolls all differ 32.6% of
the time, 9.2 rolls for each roll of a fair die; four all differ 10.4% of the
time [E-F9]. The face is read from the order alone, never from which numbers
came up.

**What is left (chapter 10).** The trick's long-run yield is p(1 − p) fair
tosses per toss: 0.16 at p = 4/5 and 0.25 for a fair coin, the "at most 25
percent" of the 1951 summary. The ceiling is the binary entropy
h(p) = −p log₂ p − (1 − p) log₂(1 − p), 0.7219 at p = 4/5: no rule whose
output is exactly fair can average more per toss over a long run (Elias
1972, Theorem 1). A single run of 100 tosses can give more than 72; the
average cannot. Elias's rule on blocks of N tosses and Peres's iterated
procedure both approach the ceiling and never reach it [E-F10].

## For programmers

- **Fresh tosses for every pair.** A sliding window that keeps the second
  toss of a thrown-away pair and compares it with the next one returns plain
  heads-or-tails: the unfair coin, unchanged.
- **It is rejection sampling:** a retry loop with no upper bound. If you cap
  the loop, the cap must return "no result", never a default winner.
- **Nothing is calibrated.** There is no estimate of p and no stored constant
  to go stale.
- **A cheaper die.** In three tosses that are not all alike, the odd one out
  is equally likely to be first, second or third. That fair one-in-three and
  one coin trick make a die for 12.5 tosses on average at p = 4/5, against
  26 for "four tosses with exactly two heads" [E-F8]. Do not map the six
  not-all-alike patterns straight onto six faces: three of them are four
  times as likely as the other three.
- **Coins with memory, using every toss (Blum).** Treat "after a head" and
  "after a tail" as two coins and run the trick on each. A result must be
  announced only when the coin is next in the state that produced it;
  announcing it at once is wrong. On a coin that repeats itself 80% of the
  time, the first two results announced at once come out 15, 15, 35 and 35
  percent where 25 each is fair; announced on return they are 25 each
  [E-F7].
- **Peres's procedure, as code.** From pairs (a, b): output a when a ≠ b.
  Then recurse on two leftover streams: a XOR b for every pair, and a for
  every pair with a = b. Reusing only the second stream stops near 18 fair
  tosses per hundred at p = 4/5; only the first, near 39; both, they climb
  to 72.2. On 100,000 simulated tosses, run until nothing was left, it gave
  69.6 per hundred [E-S9]. The price of climbing is memory and waiting: a
  block of 64 tosses must be held before anything comes out. And decide the
  number of tosses to read before you start: Peres stresses (Remark 3) that
  stopping his procedure, or Elias's, the moment it has produced the number
  of fair tosses you wanted does not give fair tosses.

## Sources

- John von Neumann, "Various Techniques Used in Connection With Random
  Digits" (summary written by George E. Forsythe), in *Monte Carlo Method*,
  National Bureau of Standards Applied Mathematics Series 12, 1951, pp.
  36–38: the proceedings of a symposium held at UCLA on June 29, 30 and July
  1, 1949. The quoted passage is on p. 36. The same page holds "Any one who
  considers arithmetical methods of producing random digits is, of course, in
  a state of sin."
- Wassily Hoeffding and Gordon Simons, "Unbiased Coin Tossing with a Biased
  Coin", *Annals of Mathematical Statistics* 41(2), 1970.
- Paul A. Samuelson, "Constructing an Unbiased Random Sequence", *Journal of
  the American Statistical Association* 63(324), 1968.
- Peter Elias, "The Efficient Construction of an Unbiased Random Sequence",
  *Annals of Mathematical Statistics* 43(3), 1972.
- Quentin F. Stout and Bette Warren, "Tree Algorithms for Unbiased Coin
  Tossing with a Biased Coin", *Annals of Probability* 12(1), 1984: they
  prove Hoeffding and Simons's conjecture that there is no optimal algorithm
  for making a fair toss from a coin of unknown chance.
- Manuel Blum, "Independent Unbiased Coin Flips From a Correlated Biased
  Source: A Finite State Markov Chain", 25th FOCS, 1984; *Combinatorica*
  6(2), 1986.
- Edsger W. Dijkstra, "Making a fair roulette from a possibly biased coin",
  *Information Processing Letters* 36(4), 1990 (EWD 1071): 37 equally likely
  results from 37 tosses.
- Yuval Peres, "Iterating von Neumann's Procedure for Extracting Random
  Bits", *Annals of Statistics* 20(1), 1992.
- Ari Juels, Markus Jakobsson, Elizabeth Shriver and Bruce K. Hillyer, "How
  to turn loaded dice into fair coins", *IEEE Transactions on Information
  Theory* 46(3), 2000.
- Andrew Gelman and Deborah Nolan, "You Can Load a Die, But You Can't Bias a
  Coin", *The American Statistician* 56(4), 2002.
- Persi Diaconis, Susan Holmes and Richard Montgomery, "Dynamical Bias in
  the Coin Toss", *SIAM Review* 49(2), 2007.
- František Bartoš and others, "Fair Coins Tend to Land on the Same Side
  They Started: Evidence from 350,757 Flips", *Journal of the American
  Statistical Association* 120(552), 2025.
- Benjamin Jun and Paul Kocher, "The Intel Random Number Generator",
  Cryptography Research white paper, 1999: "a hardware corrector based on a
  concept proposed by John von Neumann", producing "an average of one bit for
  every 6 raw binary samples". The paper says Intel enhanced the corrector,
  and that even under extreme conditions the raw bit frequencies were within
  about a hundredth of one half.
- Salil Vadhan, *Pseudorandomness*, 2012: "The study of randomness extraction
  goes back to von Neumann."

## Every number

Every count and chance in the video is computed by the video's own code and
checked against an independent script (`./x fair_coin facts-check`); quoted
numbers (dates, the 0.508, the 350,757 flips) are cited above. The seeded
runs can be replayed with NumPy's `Generator(PCG64DXSM(seed))`, seeds 195101
to 195109 for runs 1 to 9 (1951 is the year the summary was printed): a toss
is heads when the draw is below 0.8.
