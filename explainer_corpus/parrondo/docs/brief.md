# Brief — Two Losing Games Can Win — Here Is the Catch

Video identity `parrondo` (catalog epic `mx-video-program-program-87do.15`,
task `.15.1`). Draft of 2026-10-24 by WildBass. Defaults chosen by this lane
are marked *(default)*.

## Promise

Two coin games each lose money on their own, yet choosing between them with
a fair coin before every round wins. The viewer computes all three long-run
averages, sees where the gain comes from (the share of rounds spent at each
remainder of the capital divided by three moves when the games are mixed),
and leaves knowing exactly when switching fails.

Audience: the corpus's three viewers (plan §1.2). Maya needs the remainder
and the "crowd" defined before any chain or weight; Dev needs the mechanism
and a check he could code; Ruth needs the exact model, the long-run sense of
"wins", and no universal claim about switching.

## The model (exact, from the primary source)

Harmer & Abbott (1999), verified against the one-page Nature text:

- **Game A**: one coin, win probability `p = 1/2 − ε`.
- **Game B**: if the capital is a multiple of `M`, a coin with
  `p₂ = 1/10 − ε`; otherwise a coin with `p₃ = 3/4 − ε`.
- `M = 3`, `ε = 0.005`; every round wins or loses one unit.
- Their figure averages 50,000 simulated trials over 100 games for A, B,
  AABB repeated, and random switching.

Assumptions this video states on screen *(default where marked)*: the capital
is any whole number, negative values included, and the game never stops (no
ruin); "divides by three" means remainder 0, so −3 counts and −1 leaves 2
*(default; the paper does not discuss negative capital)*; the "mix" is a fair
coin choosing A or B independently before every round, the random mixture
with γ = 1/2 of Ethier & Lee; "wins" means the long-run average gain per
round is positive (a strong law of large numbers holds for these games,
Ethier & Lee 2009). Units: one dollar per round, so averages read as cents a
round *(default; the papers use abstract capital)*.

## The continuing example

One triangle with corners 0, 1, 2 (the capital's remainder) carries every
game. The **skeleton** coins without the handicap (1/2, 1/10, 3/4) give exact
small numbers a viewer can check:

- B's crowd of **26** players settles at **10, 4, 12**; one round moves every
  player and leaves the counts unchanged (1 + 9 arrive at 0, 1 + 3 at 1,
  9 + 3 at 2); 1 + 3 + 9 = **13** winners, exactly half. 26 is the smallest
  crowd whose expected flows are all whole players.
- B's weights are 5/13, 2/13, 6/13: the bad coin gets 38.5 % of the rounds,
  not a third. With equal thirds B would win 8/15 = 53.3 %.
- The mix's coins are 3/10 and 5/8; its weights 245/709, 180/709, 284/709;
  it wins 727/1418, an average of 18/709 ≈ 2.54 cents a round (Ethier & Lee
  give 18/709). Its A rounds win exactly half; its B rounds win 745/1418 =
  52.5 % because corner 0 holds only 34.6 %.

The published handicap (ε = 1/200) then gives the opening's numbers: A −1.00,
B −0.87 (exactly −73443/8446300 dollars), mix +1.57 (exactly 223/14200)
cents a round.

## Primary sources

1. G. P. Harmer and D. Abbott, "Losing strategies can win by Parrondo's
   paradox", *Nature* 402, 864 (23/30 December 1999), doi:10.1038/47220. Read
   in full (one page). Supports: the three coins and M = 3, ε = 0.005; "called
   Parrondo's paradox, after its discoverer, Juan Parrondo"; the flashing
   brownian ratchet used "to explain the game by analogy"; AABB periodic and
   random switching both winning in their simulation.
   **Two errors in that source, not to repeat:** the figure caption says
   "coin 3 has a poor probability of winning" (coin 2, the 1/10 − ε coin, is
   the poor one), and "playing games A and B in any sequence leads to a win",
   which is false: strict alternation AB loses for every small ε > 0 (below,
   and Ethier & Lee).
2. S. N. Ethier and J. Lee, "Limit theorems for Parrondo's paradox",
   *Electronic Journal of Probability* 14 (2009), paper 62, 1827–1862,
   doi:10.1214/EJP.v14-684 (arXiv:0902.2368). Read through the capital-
   dependent sections (via the ar5iv rendering). Supports and pins: the strong
   law and central limit theorem for these games; μ_B(0) = 0 and the
   stationary distribution formula that gives (5/13, 2/13, 6/13) at ρ = 1/3;
   μ for the γ = 1/2 random mixture at ε = 0, 18/709; periodic patterns at
   ε = 0: [1,1] = AB gives 0 for every ρ and −(228/169)ε + O(ε²); [2,2] = AABB
   4/163; [1,2] = ABB 2416/35601; [2,1] = AAB 32/1609. The oracle reproduces
   every one of these exactly, per round played.
3. G. P. Harmer and D. Abbott, "Parrondo's paradox", *Statistical Science*
   14(2), 206–213 (1999), doi:10.1214/ss/1009212247. Located, not read; cited
   only as further reading.

Not used as evidence: Wikipedia, Scientific American, Science News, Futility
Closet and blog posts found by search. One secondary claim deliberately left
out of the narration: the year Parrondo devised the games (often given as
1996) was not found in a primary source this lane read.

Related but different, not to be conflated: Ethier & Lee, "Parrondo's
paradox via redistribution of wealth", *EJP* 17 (2012), is about many
players exchanging capital. This video's "redistribution" is the shift of one
player's long-run share of rounds among the three remainders.

## Bounded competition check (2026-10-24)

Method: four web searches; no video was watched, transcribed or opened. What
was found are search listings and articles, not viewings.

- YouTube itself: no direct video result came back. One third-party AI
  summary page describes a YouTube explainer titled "Parrondo's Paradox: How
  Two Losing Games Win" (roulette framing, Markov chains, a "penguin slides"
  analogy); its channel, length and views are unknown to this lane.
- Written explainers found: Wikipedia, Scientific American, Futility Closet,
  3 Quarks Daily, R-bloggers (a simulation post), brainwagon, a Wolfram
  Demonstration, an arXiv tutorial on the Markov-chain analysis
  (arXiv:2304.05876), and a CSUN note that reports the 34.6 % share at
  remainder 0 under the mix.
- What those listings suggest the common treatment does: state the rules,
  simulate, and gesture at a ratchet or at "spending more time in good
  states". What this video adds, as a hypothesis rather than a measured gap:
  a crowd of 26 that checks fairness in whole numbers; the single line "B
  wins 75 % minus 65 points per unit of weight at zero" that turns the reveal
  into one dial; the cycle-product test as the rigorous form of the triangle;
  and explicit negative controls (ordinary coins, strict alternation, the
  wrong share of A, finite-horizon luck).

Channel evidence (MOR_CASE, SELECTION) supports only an editorial hypothesis:
the fair-coin video's 1.3 % click-through warns that a coin hook must reveal
its mechanism quickly. Nothing here predicts views.

## Opening and arc (second draft, after the owner's 2026-10-24 direction)

No first-minute derivation deadline: the film earns each formula after the
setting. The opening (chapter 01, the one chapter with a native scene) asks
the puzzle, gives one player seven dollars and game A's bent coin, shows a few
seeded rounds that climb, lets the viewer predict the cent a round, then
shows the same player's 100,000 rounds sinking about a thousand dollars.
Chapters 02 to 18 are planned, words and pictures, in
`docs/storyboard-draft.md` ("The arc"): B's rule and the remainder taught on
that player; B looks like a winner and loses; the mix looks hopeless and
wins; the bad-coin exposure as the clue; the triangle; 26 copies of the
player as a crowd with conserved flows before any weight is named (round-
number coins, labelled); the mix shifting the crowd; the published coins
restored; net flow and the two products; switching schedules that fail;
what "wins" promises over finite play; a closing switching rule that loses.
The draft runs about 21 minutes by the storyboard tool's estimate
(unmeasured).

## Vocabulary (one word per idea)

**game A**, **game B**, **the mix** (a fair coin picks A or B before every
round; never "strategy" in the voice); **round** (one toss, one dollar won or
lost); **bad coin**, **good coin**; **your money** / **your dollars** (the
capital); **divides by three**; **remainder** (0, 1, 2) and **corner** (the
triangle's three corners named by remainders); **forward** (a win, one corner
round) and **back** (a loss); **crowd** (many independent players, split
exactly by the coins' chances: the expected numbers); **settle**; **weight**
(a corner's long-run share of rounds; "stationary distribution" only on
screen as a name to search); **average per round** (never "drift" or
"expected value" in the voice); **net flow** (forward minus back across one
edge per round); **forward product**, **backward product**; **the half
point** (ε, half a percentage point off every coin; never "half a cent");
**handicap**.

## Protected facts

The published coins, the three long-run averages, B's fairness without the
handicap in whole numbers, the mix's shift of weight off corner 0, strict
alternation losing, and the sign rule are pinned by `src/facts.rs`
(`the_headline_numbers`, with the source pins), by the spoken-number table
in `src/main.rs`, and recomputed another way by `oracle.py`.

## Risks

- Pronunciation (for the lexicon, not resolved here): Parrondo
  (Spanish /paˈrondo/, "pah-RON-doh"), Juan ("hwahn"), Harmer, Abbott,
  "thirteenths", "nine over one hundred sixty", "sixty-two and a half",
  "fifty-two point eight", letter names "game A"/"game B" (A must not reduce
  to the article). Ethier is shown on screen only (pronunciation unverified).
- Mathematical: "wins" must stay "long-run average per round"; the sign rule's
  direction is stated with its formula on screen; "the whole gain comes from
  the weights" is exact only in the decomposition by game (A rounds, B
  rounds), which the script uses.
- Editorial: no betting-system framing, no financial or investing analogy, no
  biology or economics applications, no unverified history.
