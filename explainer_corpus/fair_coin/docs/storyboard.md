# Storyboard — How to Make a Fair Coin Out of an Unfair One

The hand-edited source of this video's words and pictures. `python3 -I
tools/storyboard.py fair_coin` checks it (gate G2) and writes `script.tsv`
from the words column. Facts: `docs/fact-sheet.md` and `oracle.py` (every
number below names its fact key); brief: `docs/brief.md`; reviews and their
dispositions: `docs/reviews/fair_coin/`.

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a prediction
question; the beat ends with the question and the picture holds before the
next beat answers it.

## 2 The arc

The owner asked for two things, the two-toss trick and what it grows into for
unfair dice; they carry the video. Memory, the entropy limit and the hardware
note are one picture or one beat each.

| Act | Chapters | Spine |
|---|---|---|
| I. The trick | 01 call it, 02 toss twice, 03 every coin | an unfair coin and an unfair bet; two tosses, four blocks of a square, two of them always equal; why: the same two numbers multiplied in either order, the mirror image, for every coin |
| II. Its price and its limits | 04 the price, 05 where it breaks | most pairs are thrown away; six and a quarter tosses on average, no longest game; two different coins and a coin with a memory break the mirror |
| III. What it grows into | 06 same counts, 07 a die from a coin, 08 loaded dice, 09 six bricks | the same results in a different order always have the same chance; Pascal's rows as groups of equally likely patterns; six patterns make a die; a loaded die compared with itself, and the orders of different rolls |
| IV. Coda | 10 what is left, 11 rigorously unbiased | how much fairness the coin holds and how to get nearer to it; von Neumann's paragraph; the opening square, now with its reason |

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses a
data colour.

| Idea | Color |
|---|---|
| heads | blue `#58C4DD` |
| tails | amber `#F0AC5F` |
| Alice, and the block or order that wins for her (heads then tails; first roll bigger) | teal `#5CD0B3` |
| Bob, and his block or order (tails then heads; second roll bigger) | violet `#B189D6` |
| thrown away (a matching pair, a tie, a block that is not used) | `faint` grey |
| the mirror line, and "same counts, same chance" wherever it appears | `hero` gold `#FFD866` |
| a trick that has broken | `warn` red `#FC6255` |
| the ceiling (the entropy) | `good` green `#83C167` |
| a measured count beside its model number | `dim` ink with the run's size written out; the model number says "model" (plan rule 25) |

The six hues are checked pairwise for contrast (ΔE2000 ≥ 20) by a test before
drafts, as Video C's customers were.

## Vocabulary (one word per idea)

**unfair coin** (never "biased", "loaded", "weighted" or "bent" for the
coin); **chance** (never "probability" or "odds"; the 03_7 lower-third names
"probability" once); **toss** for the coin and **roll** for a die (never
"flip" or "throw"); **pair** (two tosses in a row); **heads then tails** and
**tails then heads** (the two pairs that decide; always in full); **two
heads**, **two tails** (the pairs that match); **throw away** (a matching
pair); **block** (a region of the square); **pattern** (one particular run of
results); **counts** (how many heads a pattern has) and **order**; **group**
(the patterns with the same counts); **fair** (each outcome equally likely);
**loaded die**; **fair tosses** (what comes out; never "bits"); **entropy**
(named once, at 10_2). The players are **Alice** and **Bob**.

## 01_call_it (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | This coin isn't fair. It lands heads four times in five. Alice and Bob have to settle a bet with it, and neither of them knows how unfair it is. There's a way to do it that is perfectly fair, using nothing but this coin. | One coin, tossed again and again: each result drops into a row as a disc, heads blue, tails amber, heads plainly winning; Alice (teal) and Bob (violet) wait either side of the table; lower-third "a model coin: heads 80%" (`E-F1.heads_percent`) | lagged disc row; count-up tally |  |
| 01_2 | *(title)* | **How to Make a Fair Coin Out of an Unfair One** | title card | listen 3 s (the motif: two notes, then the same two swapped) |
| 01_3 | The obvious way is to call it. Alice says heads, Bob gets tails, one toss decides. In a thousand tosses, heads comes up seven hundred ninety-one times. Alice wins about four bets in five. | The thousand seeded tosses stream into two bars, heads and tails; the counters stop at 791 and 209 (`E-S1.call_it.heads`, `.tails`); caption "1,000 tosses" | live bars and counters; bars |  |
| 01_4 | Then let Bob call heads. But that only helps if they know which side the coin favours. With a strange coin, they don't. | Both faces of the coin are covered by question marks; the two bars swap sides and swap back: it could be either | morph; highlight |  |
| 01_5 | A real coin, tossed and caught, can't be made this unfair. A coin spun on a table can. So can a loaded die, or the noisy circuit inside a computer. Our coin stands for all of them. | Three small pictures in a row: a coin spinning on its edge (caption "1,878 heads in 5,520 spins of 1960s pennies: classroom records", `E-C3.pennies.*`), a loaded die, a noisy circuit trace; lower-third "tossed and caught, a coin can't be biased: Gelman and Nolan, 2002" | lagged reveal |  |
| 01_6 | ⟨predict⟩ So here's the puzzle. One unfair coin, and nobody knows how unfair. How would you get a fair decision out of it? | The coin alone at centre, Alice and Bob either side; the question as a card | prediction card | hold |

- **Misconception defused:** "just give the other player heads" (01_4): that
  needs the coin's chance, which nobody has.
- **Ruth:** the coin is a model and says so (01_1 lower-third, 01_5): a
  flipped-and-caught coin cannot be biased by weighting (Gelman and Nolan
  2002, abstract). The penny count is classroom data reported in Chance News
  11.02, labelled as that; the unsourced "80% tails" figure is not used
  (fact sheet, finding 2).
- **Dev:** the circuit in 01_5 is paid off at 11_4 (Intel's generator).

## 02_toss_twice (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Toss the coin twice, and look at the two results as a pair. | Two discs land side by side and a bracket joins them: one pair | bracket |  |
| 02_2 | A pair can be four things. Two heads. Heads then tails. Tails then heads. Two tails. | Four pair tiles appear one by one, each two discs: HH, HT, TH, TT | lagged reveal |  |
| 02_3 | Draw their chances as areas. The first toss splits a square from left to right. Heads gets four fifths of the width. | A unit square; a vertical cut at 0.8; the bottom edge carries a blue stretch "first toss: heads" and a short amber stretch "tails" | **the unit square**; grow |  |
| 02_4 | The second toss splits it from bottom to top, at the same mark, because it's the same coin. | A horizontal cut at 0.8; the left edge carries "second toss: heads" (blue, the lower four fifths) and "tails"; four blocks, each taking its pair tile | the unit square; the tiles morph into the blocks |  |
| 02_5 | Two heads is the big block, sixty-four percent. Two tails is the small one, four percent. ⟨predict⟩ What about heads then tails, against tails then heads? | The big block reads 64%, the small one 4% (`E-F1.square.hh`, `.tt`); the two thin blocks pulse with question marks | live area readouts; highlight |  |
| 02_6 | Sixteen percent each. Exactly equal. | Both thin blocks read 16% (`E-F1.square.ht`, `.th`); beside them "0.8 × 0.2" and "0.2 × 0.8" | highlight |  |
| 02_7 | So play it like this. Alice calls heads then tails. Bob calls tails then heads. If the pair matches, throw it away and toss two more. | The heads-then-tails block turns teal and takes Alice's name; the tails-then-heads block turns violet and takes Bob's; the two matching blocks fade to grey with a loop arrow "toss again" | highlight; morph |  |
| 02_8 | A thousand games with our coin. Alice wins four hundred ninety-four. Bob wins five hundred six. | Beside the square, games play as rows of pair tiles (the run's first games: grey pairs, then a teal or violet one, `E-S2.trick.first_games`); then the rows race and two counters tick to 494 and 506 (`E-S2.trick.heads_first`, `.tails_first`); caption "1,000 games, the same unfair coin" | live counters; lagged rows; a strip of 1,000 ticks | listen 4 s (each game is one of two notes, Alice's or Bob's: an even patter) |
| 02_9 | The coin is as unfair as ever. The game isn't. | The coin's own bars (791 to 209) above the game's bars (494 to 506) | bars | hold |

- **The payoff is previewed here** (plan rule 3): the method works on honest
  seeded data before any "why".
- **Ruth:** "exactly equal" is `E-F1.square.ht.exact` = `.th.exact` = 4/25;
  494 to 506 is one seeded run of 1,000 (seed in `E-S0.seed`), well inside
  chance (three standard errors are 47).
- **Engine:** the square is four fixed rectangles moved and stretched by
  placement, never redrawn point by point; the area readouts are small live
  numbers.

## 03_every_coin (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Why exactly equal? Heads then tails is four fifths times one fifth. Tails then heads is one fifth times four fifths. | Over each thin block its product in TeX: 4/5 × 1/5 and 1/5 × 4/5 | TeX fraction products |  |
| 03_2 | The same two numbers, multiplied in the other order. And multiplying doesn't care about order. | The two products swap their factors and settle as one equation; on the square a gold diagonal appears, the teal block lifts, flips across it and lands exactly on the violet one | **TransformMatchingTex**; the mirror flip; glow |  |
| 03_3 | ⟨predict⟩ Make the coin worse. Ninety percent heads. Are the two blocks still equal? | A slider "chance of heads" under the square moves from 80% to 90%; both cuts follow it; the readouts blank to question marks | slider on a value tracker; prediction card |  |
| 03_4 | Yes. Slide the cut wherever you like. One block is always the mirror image of the other. | The slider sweeps from 50% to 99% and back; both cuts move together, the two thin blocks stay mirror images across the gold diagonal, their live readouts always equal (`E-F4.*.ht`); at each stop a small tile shows Alice's wins in 1,000 seeded games there: 493, 502, 490, 509, 472, 513 (`E-S3.*.heads_first`) | **live recomputation from a tracker**; lagged tiles |  |
| 03_5 | In letters, call the chance of heads p. Heads then tails is p times one minus p. Tails then heads is one minus p times p. Equal for every p. | The numbers in the equation morph into letters: p(1 − p) = (1 − p)p; the square's edges take the labels p and 1 − p | TransformMatchingTex: numbers → letters |  |
| 03_6 | So nobody needs to know how unfair the coin is, or even which side it favours. | The slider's number is covered by a question mark; the two blocks stay equal; Alice and Bob beside them | highlight |  |
| 03_7 | The trick does lean on two things. The coin's chance is the same on both tosses, and one toss doesn't affect the next. Tosses like that are called independent. | Two cards: "same chance on both tosses", "one toss doesn't affect the next"; lower-third "precisely: independent tosses with the same probability p of heads, for any p strictly between 0 and 1" | statement cards | hold |

- **The owner's sentence** "since the trials are independent, the
  probabilities commute" is 03_2 (the swap) and 03_7 (what it needs).
- **Misconception defused:** "it works because the coin is close to fair" →
  the 99% stop (03_4).
- **Ruth:** "every p" is tested at both ends
  (`edge_heads_tails_equals_tails_heads_for_every_chance`): the areas are
  equal at 0 and 1 too, but then no pair ever decides; the lower-third's
  "strictly between" is the exact statement. What the mirror needs is
  slightly less than 03_7 says (fact sheet, finding 3, and "Simplification →
  exact"); 05 shows both ways it fails.
- **Dev:** no estimate of p is ever computed; the procedure is the same for
  every coin (Elias 1972 calls this "without making use of the process
  parameters").

## 04_the_price (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | The trick has a price. Most pairs get thrown away. With our coin, a pair decides only thirty-two times in a hundred. | The square at 80%: the two grey blocks merge into one bar reading 68%, the teal and violet into one reading 32% (`E-F2.thrown_away_percent`, `.decides_percent`) | morph blocks → proportion bar |  |
| 04_2 | ⟨predict⟩ So how many tosses does a game take, on average? | The question as a card; a single game plays slowly: pair, pair, pair, decided | prediction card |  |
| 04_3 | Six and a quarter. Here are ten thousand games. They average six point two one. | A histogram grows from the 10,000 seeded games, one bar per number of pairs needed (`E-S4.price.pairs_1_to_8`); a marker at the mean, "6.21 tosses, 10,000 games" (`E-S4.price.mean_tosses`); under it "model: 1 ÷ (0.8 × 0.2) = 6.25" (`E-F3.expected_tosses`) | **histogram growing from data**; count-up |  |
| 04_4 | A fair coin would need four tosses. A coin that's ninety-nine percent heads needs about a hundred and one. | The model line becomes 1 ÷ (p(1 − p)); a curve of it against the chance of heads, lowest at one half; dots at 4, 6.25 and 101 (`E-F4.p50.expected_tosses.exact`, `E-F4.p99.expected_tosses_rounded`), the last far up a broken axis | curve on axes; TeX |  |
| 04_5 | And no game is sure to end by any given toss. About one game in fifty is still going after ten pairs. The longest of our ten thousand ran thirty pairs. Sixty tosses. | The histogram's long tail stretches right; "after 10 pairs: 2.1% still going (model)" (`E-F3.undecided_after.10_pairs_percent`); the longest game drawn in full, thirty pairs (`E-S4.price.longest_pairs`) | histogram tail; lagged row |  |
| 04_6 | Could a cleverer rule always stop after ten tosses? No. Picture a coin that almost never lands tails. Ten heads in a row is then almost certain, so whoever wins on ten heads almost always wins. | A row of ten blue discs; the slider pushed to its far end; one player's bar fills almost completely, red; lower-third "no rule with a fixed number of tosses is fair for every coin" (`E-X1.*`: every rule on up to 5 tosses checked) | highlight; live bar | hold |

- **Misconception defused:** "it takes two tosses" (04_2) and "it must end by
  some point" (04_5).
- **Ruth:** the number of pairs is geometric; "sure to end" is with
  probability one, and 04_5 says what is true instead of "always ends".
  04_6's argument is the whole proof: at p near 0 or 1 one pattern has nearly
  all the chance. The model number carries the word "model", the measured one
  its 10,000 (plan rule 25).
- **Dev (lower-third, for programmers):** this is rejection sampling: loop
  until accepted, with no bound on the loop.

## 05_where_it_breaks (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | ⟨predict⟩ Now break it. Toss our unfair coin first, and a fair coin second. Is heads then tails still level with tails then heads? | Two coins: the unfair one (80%) and a second, fair one; the square's top cut gets its own slider | prediction card |  |
| 05_2 | No. One cut moved and the other didn't, so the blocks aren't mirror images any more. Forty percent against ten. Alice wins four games in five, as if they'd never bothered. | The horizontal cut slides from 0.8 to 0.5 while the vertical one stays; the teal block grows to 40%, the violet shrinks to 10% (`E-F5.two_coins.ht`, `.th`); the gold diagonal no longer maps one onto the other and turns red; 1,000 seeded games: 807 for Alice (`E-S5.two_coins.heads_first`) | the unit square, one cut on its own tracker; live counters | listen 3 s (the same two notes: now four in five are Alice's) |
| 05_3 | A sneakier break is a coin with a memory. Say it repeats its last result four times in five. | The single coin again, with a small arrow from each result back to the next toss: "repeats 80%" (`E-F6.sticky.repeat_percent`); lower-third "a model" | arrow; label |  |
| 05_4 | After two heads, heads then tails takes one change of face. Tails then heads takes two. So it's four times as likely. Anyone who saw the pair that was thrown away knows who's about to win. | The square for the pair after two heads: the first toss still cuts at 0.8, but the two columns are now cut at different heights, 0.8 in the heads column and 0.2 in the tails column; teal reads 16%, violet 4% (`E-F6.sticky.after_heads.ht`, `.th`, `.ratio`); measured in 200,000 tosses: 6,306 against 1,615 (`E-S6.sticky.after_hh.*`) | **the staggered square**; live readouts |  |
| 05_5 | Real coins have a touch of this. A tossed coin comes up the way it started about fifty-one times in a hundred. | A coin leaves a thumb heads up and lands heads up; card "about .51: Diaconis, Holmes and Montgomery, 2007"; beneath it "350,757 tosses: 0.508 (Bartoš and others, 2025)" (`E-C3.*`) | quote card |  |
| 05_6 | The cure is to compare like with like. Pair up only the tosses that come straight after a head, and the trick is fair again. | A long row of sticky tosses; every disc that follows a blue one drops to a second row; that row is cut into pairs and played: 7,847 against 8,043 (`E-S6.sticky.after_heads_only.*`); lower-third "Samuelson, 1968. Using every toss takes more care: Blum, 1986" | lagged filter of a row; live counters | hold |

- **One picture for memory:** the staggered square (05_4). The exact
  statements, including that the *first* pair of a long run is still fair,
  are in the fact sheet (finding 3, `E-F6.*`) and go in the description.
- **Ruth:** "four times as likely" is exact for this model (0.16 against
  0.04). 05_5 is "the same way it started", not "heads"; 0.508 is the
  paper's estimate, with a 95% credible interval (Bayesian) of 0.506 to
  0.509. 05_6 is fair when played to the end; under a deadline it leans (fact
  sheet, finding 4), and the video sets none.
- **Dev:** Blum's point, for the description: treat "after a head" and "after
  a tail" as two coins and you may use every toss, but each result must be
  announced only when the coin is next in that state; announcing at once is
  wrong (`E-F7.blum.*`: 15, 15, 35, 35 percent for the first two results
  where 25 each is fair).

## 06_same_counts (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Go back to why the trick works. Heads then tails and tails then heads are made of the same results, one head and one tail. Only the order is different. | The two deciding pairs as disc pairs, one above the other; a gold link joins each blue disc to its twin and each amber to its twin | morph that keeps identity; glow |  |
| 06_2 | And order never changes a chance. ⟨predict⟩ Take three tosses. Two heads then a tail. Heads, tails, heads. A tail then two heads. Which is likeliest? | Three rows of three discs: HHT, HTH, THH; question marks where their chances will go | lagged reveal; prediction card |  |
| 06_3 | None of them. Each is point eight, times point eight, times point two, in some order. Same counts, same chance. | Under each row its product: 0.8 × 0.8 × 0.2, 0.8 × 0.2 × 0.8, 0.2 × 0.8 × 0.8; the factors slide into the same order and all three read 12.8% (`E-F8.three_two.each_percent`); the words "same counts, same chance" in gold | TransformMatchingTex; glow |  |
| 06_4 | So sort every pattern by how many heads it has. For two tosses, the groups hold one, two and one. The middle group is our trick. | The four pairs of two tosses slide into three groups by count of heads: [HH] [HT, TH] [TT]; a count under each: 1, 2, 1 (`E-F8.pascal.row2`); the middle group glows teal and violet | **patterns reorganize into groups**; count-up |  |
| 06_5 | For three tosses, one, three, three, one. For four, one, four, six, four, one. | The eight patterns of three tosses, then the sixteen of four, sort into groups below; only the counts stay as numbers, row under row (`E-F8.pascal.row3`, `.row4`): Pascal's triangle, named in a lower-third | lagged group reveals; a grid of counts |  |
| 06_6 | The coin's unfairness decides which group you land in. Our coin usually lands in a group with lots of heads. But inside a group, every pattern is equally likely, for any coin at all. | Row four as one long bar cut by the 80% coin into five stretches (`E-F8.row4.heads0_percent` … `heads4_percent`: 0.16, 2.56, 15.36, 40.96, 40.96 percent); each stretch is cut again into equal slices, 1, 4, 6, 4 and 1 of them; the slider moves and the stretches change while the slices inside each stay equal | proportion bar on a tracker; live recomputation |  |
| 06_7 | The counts carry the unfairness. The order is perfectly fair. | A card with the two sentences, the second in gold | statement card | hold |

- **This is the idea the dice need** (the owner's "generalize"): order is fair
  given the counts. 06_7 is the act's one-sentence recap (plan rule 14).
- **Misconception defused:** "heads-heavy patterns that end in tails are
  rarer than ones that start with it" (06_2).
- **Ruth:** exchangeability of independent, identically distributed tosses;
  tested for every pattern of up to six tosses
  (`edge_same_counts_same_chance`). The groups are Elias's "composition
  classes" (1972, p. 868).
- **Engine:** at most 16 strips of four discs on stage; groups are moved by
  placement.

## 07_a_die_from_a_coin (~55 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | ⟨predict⟩ Look at the middle of row four. Six patterns, all equally likely. What else has six equally likely outcomes? | The group of six lifts out of row four: HHTT, HTHT, HTTH, THHT, THTH, TTHH (`E-F8.four_two.list`), each reading 2.56% (`E-F8.four_two.each`) | highlight; lagged reveal |  |
| 07_2 | A die. Toss the coin four times. If exactly two are heads, the pattern picks the face. If not, toss four more. | Each of the six patterns morphs into a die face, one to six; the other ten patterns of four tosses fade to grey with the loop arrow "toss again" | **morph: patterns → die faces** |  |
| 07_3 | Twenty-four thousand tosses of our unfair coin made nine hundred forty-nine rolls of a fair die. Each face came up about one time in six. | The seeded tosses stream past in fours; accepted ones drop onto six bars that grow to 166, 161, 136, 160, 167, 159 (`E-S7.coin_die.faces`); a line at one sixth of 949; caption "24,000 tosses, 949 rolls" (`E-S7.coin_die.tosses`, `.dice`) | histogram growing from data; count-up |  |
| 07_4 | It isn't cheap. A coin that loves heads rarely gives exactly two in four. It works about fifteen tries in a hundred, so a roll costs twenty-six tosses on average. | The row-four bar again with the six-pattern stretch lit: 15.4% (`E-F8.four_two.works_percent`); "model: 26.0 tosses a roll; this run: 25.3" (`E-F8.four_two.expected_tosses`, `E-S7.coin_die.tosses_per_die`) | proportion bar; live numbers |  |
| 07_5 | There are cheaper recipes. But the idea is what matters. Any group of patterns with the same counts is a fair choice among them. | Pascal's triangle returns with every entry glowing in turn: each is a fair choice among that many; lower-third "for programmers: three tosses that aren't all alike are a fair one in three; with one coin trick that makes a die for 12.5 tosses" (`E-F8.three_plus_coin.expected_tosses`) | glow sweep over the grid | hold |

- **The owner's example** (C(4, 2) = 6: "a fair six-sided die made from a
  bent coin"), with its honest cost.
- **Ruth:** the die's fairness over 949 rolls is a chi-squared check in the
  tests, not an eyeball; "about one time in six" spans 136 to 167. Stout and
  Warren (1984) proved there is no best recipe (description).
- **Dev:** the lower-third's cheaper route is computed
  (`E-F8.three.works_percent` = 48, then 3/0.48 + 6.25).

## 08_loaded_dice (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Now swap the coin for a loaded die. This one shows a six half the time. | A die; six bars for its faces at 5, 5, 10, 10, 20 and 50 percent (`E-F9.loaded.percents`), pointed at, not read; 6,000 seeded rolls fill them: 318, 283, 579, 607, 1,237, 2,976 (`E-S8.loaded.faces`) | bars growing from data |  |
| 08_2 | ⟨predict⟩ Can Alice and Bob get a fair decision from it? | Alice and Bob either side of the die; the question as a card | prediction card |  |
| 08_3 | Roll it twice. If the two rolls match, roll again. If not, Alice wins when the first roll is bigger, and Bob wins when the second is. | The unit square again, cut by the die's chances along both sides: thirty-six blocks; the blocks under the diagonal turn teal, those above it violet, the six on it grey; the gold diagonal returns | **the square, cut six ways: a grid of 36 blocks**; lagged colouring |  |
| 08_4 | It's the coin trick again. Two different results are as likely one way round as the other. A coin is just a die with two faces. | One teal block (a five then a six) lifts, flips across the diagonal onto its violet twin (a six then a five), and fits; then the whole lower triangle flips onto the upper; 3,000 seeded pairs: 1,021 for Alice, 1,020 for Bob, 959 ties (`E-S8.two_rolls.*`); the six-way grid morphs back into the coin's two-way square and out again | the mirror flip; grid morph; live counters |  |
| 08_5 | Three rolls do more. If all three are different, they came in one of six orders. Smallest first, biggest first, and so on. Six orders, equally likely. That's a fair die, made from a loaded one. | Three dice land: two, six, five; a small chart ranks them low, high, middle; six order cards (the six ways to arrange low, middle, high) each map to a face; 6,000 seeded rolls in threes made 650 fair rolls: 119, 108, 103, 105, 112, 103 (`E-S8.three_rolls.dice`, `.orders`) | lagged cards; bars growing from data |  |
| 08_6 | And it keeps going. Four different rolls can come in twenty-four orders. Five, in a hundred and twenty. | The order cards multiply: 2, 6, 24, 120 (`E-F9.orders.*`); lower-third "n different rolls: n! equally likely orders. With this die, three rolls all differ 33 times in 100: 9.2 rolls for each fair one (model)" (`E-F9.three_rolls.works_percent`, `.expected_rolls`) | count-up | hold |

- **The owner's "unfair dice".** 08_4 is the hindsight moment (plan rule 19):
  heads then tails against tails then heads is "which of two different
  results came first", and a coin is a two-faced die.
- **Misconception defused:** "a loaded die can only make loaded decisions"
  (08_2).
- **Ruth:** the orders of *different* values; ties are thrown away, and the
  chance of each order is the same symmetric sum over trios
  (`edge_every_order_of_different_rolls_is_equally_likely`, by full listing
  for three dice). 1,021 to 1,020 is one run, not a law.
- **Dev:** with this die two rolls decide 68.5% of the time, 2.9 rolls a
  decision (`E-F9.two_rolls.*`); Juels, Jakobsson, Shriver and Hillyer
  (2000) is the paper for getting the most out of dice.

## 09_six_bricks (~30 s; the whole chapter on the camera route)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Here's the picture behind that. Three rolls pick a point in a cube, and the die's chances slice it along every side. | A lit unit cube, slowly turning; three sets of slicing planes at the die's running totals, the same six slabs along each edge; the faint grid of 216 boxes (`E-F9.three_rolls.boxes`) | **3D cube with a lit orbit (camera)**; lagged slicing |  |
| 09_2 | ⟨predict⟩ A four, a five and a six, in each possible order, land in these six bricks. How do the bricks compare? | Six bricks light up inside the cube, three teal-to-violet hues apart; each carries its order (4 5 6, 4 6 5, …); the camera orbits | glow; orbit (camera) |  |
| 09_3 | They're one brick, turned six ways. A tenth, by a fifth, by a half. Same volume, so the same chance. | The six bricks slide out of the cube and turn until they stand side by side, identical; one is labelled 10% × 20% × 50% = 1% (`E-F9.brick.sides_percent`, `.volume_percent`); caption "every trio of faces makes six equal bricks: 120 of the 216 boxes" (`E-F9.three_rolls.boxes_all_different`) | **bricks fly out and align (camera)** | hold |

- **Why 3D earns its place:** the square's mirror image, one dimension up;
  six congruent bricks are the 3! orders. A 2D fallback (six equal bars) is
  ready if the camera route costs too much at 4K.
- **Engine:** drawn as slab outlines on the cube's faces and six solid
  prisms, not 216 boxes; segments of an orbit under half a turn each.

## 10_what_is_left (~40 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | One loose end. The two toss trick throws most of the coin away. A hundred tosses of our coin give about sixteen fair ones. ⟨predict⟩ How many could the best possible rule get? | A strip of a hundred discs, about eighty blue; sixteen fair tosses drop out of it (`E-F10.von_neumann.per_hundred`); an empty scale from 0 to 100 beside it | count-up; number line |  |
| 10_2 | Seventy-two. That ceiling is called the coin's entropy, and no rule beats it. | A green line at 72 on the scale (`E-F10.entropy.per_hundred`); the trick's 16 far below it; lower-third "name to search: entropy" | highlight; number line |  |
| 10_3 | You get closer by wasting less. Use the bigger groups in deeper rows of the triangle. Or pick up the pairs you threw away, and run the trick again on them. | Two ladders of bars climb toward the green line: bigger groups (blocks of 4, 8, 16, 32, 64 tosses: `E-F10.elias.*`) and reusing the discards (once, twice, three times, …: `E-F10.peres.*`); dots for 100,000 seeded tosses sit on the bars: 16.1, 28.7, 38.8, and 69.6 when nothing is left to reuse (`E-S9.squeeze.*`); lower-third "Elias, 1972; Peres, 1992" | bars growing toward a limit; lagged reveals |  |
| 10_4 | Both climb toward seventy-two, and never quite arrive. | The bars' tops trace two curves bending under the green line | curve | hold |

- **One picture, not a proof** (the order): the ceiling and two ways to climb.
- **Ruth:** the entropy h(0.8) = 0.7219 bits per toss is an upper bound on
  the long-run rate of any such rule (Elias 1972, Theorem 1) and the limit
  of Peres's and Elias's (Peres, Proposition 3; Elias, Theorem 2); "never
  quite arrive" is tested (`edge_rates_climb_to_the_entropy_and_never_pass_it`).
  "A hundred tosses give about sixteen" is the long-run rate times 100.
- **Dev:** rates are output per input toss; the measured 69.6 comes from
  Peres's procedure run until no pairs remain on 100,000 tosses.

## 11_rigorously_unbiased (~50 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | The two toss trick is John von Neumann's. He gave it in a talk in nineteen forty-nine, in a single paragraph. | Quote card, verbatim: "If we get heads-heads or tails-tails, we reject the tosses and try again. If we get heads-tails (or tails-heads), we accept the result as heads (or tails). The resulting process is rigorously unbiased…"; source line "John von Neumann, Various Techniques Used in Connection With Random Digits (summary written by George E. Forsythe), 1951, p. 36" (`E-C1.*`) | quote card |  |
| 11_2 | ⟨predict⟩ He named the price too. Suppose the coin was fair all along. How much of it does the trick waste? | The square at 50%: four equal blocks, two grey | the unit square; prediction card |  |
| 11_3 | Three tosses in four. In his words, at most twenty-five percent as efficient as ordinary coin tossing. | The quote's ending appears: "…although the amended process is at most 25 percent as efficient as ordinary coin-tossing." (`E-C1.von_neumann.efficiency_percent`); beside it the model: one fair toss for every four (`E-F10.fair_coin.von_neumann.percent`) | quote card; highlight |  |
| 11_4 | Computers took him at his word. In nineteen ninety-nine, Intel's hardware random number generator ran its raw, lopsided bits through a corrector built on this trick. | The noisy circuit of 01_5; a stream of lopsided 0s and 1s enters in pairs, matching pairs vanish, a balanced stream leaves; card "a hardware corrector based on a concept proposed by John von Neumann: Jun and Kocher, The Intel Random Number Generator, 1999" (`E-C4.intel.year`) | pair filter on a stream; quote card |  |
| 11_5 | So an unfair coin still has a fair one inside it, as long as it can land both ways. You find it by ignoring what came up, and looking only at the order. | The opening square at 80% returns, the gold mirror line across it, the teal and violet blocks pulsing; then the receipts card: "every number in this video was computed by code", the repository link, and names to search: von Neumann's trick; Hoeffding and Simons, 1970; Elias, 1972; Blum, 1986; Dijkstra's fair roulette, 1990; Peres, 1992; "How to turn loaded dice into fair coins", 2000 | the unit square; receipts card | listen 4 s (the motif, resolved) |

- **Ends on the rigorous form of the opening picture** (plan rule 15): the
  same square, now with the reason drawn on it.
- **Ruth:** the printed text is Forsythe's summary of the talk (symposium at
  UCLA, June 29 to July 1, 1949; volume issued June 11, 1951); the quote is
  verbatim from p. 36, hyphens as printed. "As long as it can land both
  ways" is the edge of 11_5's "an unfair coin". The Intel card says "based
  on": the paper says Intel "enhanced" the corrector, and it does not name
  the chip, so neither do we. NIST's final SP 800-90B does not list the
  method; nothing here says it is approved anywhere.
- **Delight, on screen only:** the same page of the 1951 volume holds "Any
  one who considers arithmetical methods of producing random digits is, of
  course, in a state of sin." It may sit under the 11_1 card in small type.
- **Corpus threads (receipts card):** Hoeffding (corpus entry 1) analysed
  this trick in 1970; Dijkstra (who posed Video C's problem) built a fair
  37-way roulette from a possibly unfair coin in 1990.
