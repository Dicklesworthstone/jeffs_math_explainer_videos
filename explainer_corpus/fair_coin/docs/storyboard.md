# Storyboard — How to Make a Fair Coin Out of an Unfair One

The hand-edited source of this video's words and pictures (revised at R1,
2026-10-12). `python3 -I tools/storyboard.py fair_coin` checks it (gate G2)
and writes `script.tsv` from the words column. Facts: `docs/fact-sheet.md`
and `oracle.py` (every number below names its fact key); brief:
`docs/brief.md`; reviews and their dispositions: `docs/reviews/fair_coin/`
(ids such as M2, D1, R4, E8, C3 below are R1 findings).

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a prediction
question; the beat ends with the question and the picture holds before the
next beat answers it.

## 2 The arc

The owner asked for two things, the two-toss trick and what it grows into for
unfair dice; they carry the video. Memory, the entropy ceiling and the
hardware note are one picture or one beat each.

| Act | Chapters | Spine |
|---|---|---|
| I. The trick | 01 call it, 02 toss twice, 03 every coin | an unfair coin that one player knows better than the other; two tosses, four blocks of a square, two of them always equal; why: one block is the other flipped across the diagonal, for every coin |
| II. Its price and its limits | 04 the price, 05 where it breaks | the grey is the price: six and a quarter tosses on average, and nobody can promise when a game stops; a second coin moves one cut, a memory puts a step in the other |
| III. What it grows into | 06 same counts, 07 a die from a coin, 08 loaded dice, 09 six orders | a third toss turns the square into a cube whose equal pieces are one slab turned; patterns sorted by counts are Pascal's rows; six patterns make a die; a loaded die on the same square, and three rolls in the same cube |
| IV. Coda | 10 what is left, 11 the fair coin inside | von Neumann's paragraph and its price; how much fairness the coin holds; the opening square, now with its reason, and the dice beside it |

One picture runs through all four acts (C3): the unit square, cut by the
chances. It gains a third side for three tosses (06) and for three rolls
(09), and it is cut six ways for a die (08). Pascal's triangle is what is
left of it when there are too many tosses to draw.

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses a
data colour. The four hues of this video and the corpus's gold, red, ink,
dim and faint are pairwise at least ΔE2000 20 apart (the least is 21.6; a
test in `chapters/draw.rs` holds them to it; C10).

| Idea | Color |
|---|---|
| heads; for a die, a bigger roll's stretch is drawn nearer the corner, as heads is | blue `#58C4DD` |
| tails | pink `#EA87B3` |
| Alice, and the block that wins for her (heads then tails; first roll bigger): above the diagonal, always | green `#8BD96F` |
| Bob, and his block (tails then heads; second roll bigger): below the diagonal, always | violet `#994DE5` |
| thrown away (a matching pair, a tie); lifted to `dim` while it is the subject (04) | `faint` grey |
| the mirror line, and "same counts, same chance" wherever it appears; strokes, glows and words only, never a fill | `hero` gold `#FFD866` |
| the part of a block that no longer fits its mirror image | `warn` red `#FC6255` |
| the ceiling (the entropy) | `ink`, as a ruled line |
| a measured count beside its model number | `dim` ink with the run's size written out; the model number says "model" (plan rule 25) |

## Vocabulary (one word per idea)

**unfair coin** (never "biased", "loaded", "weighted" or "bent" for the
coin); **chance** (never "probability" or "odds"; the 03_7 lower-third names
"probability" once); **toss** for the coin and **roll** for a die (never
"flip" or "throw"; "flip" is kept for turning a block over the diagonal);
**pair** (two tosses in a row); **heads then tails** and **tails then heads**
(the two pairs that decide; always in full); **two heads**, **two tails**;
**match** (the two results of a pair are the same, on screen too; never
"tie"); **throw away** (a matching pair); **block** (a region of the
square), **slab** and **brick** (pieces of the cube); **result** (what one
toss or roll shows; never "outcome"); **pattern** (the results of several
tosses, in order; introduced at 06_2); **counts** (how many heads and how
many tails a pattern has; defined at 06_4) and **order**; **group** (the
patterns with the same counts; the numbers 1, 2, 1 are "group sizes" on
screen); **the trick** (the two-toss game only) and **rule** (any other
method); **fair** (each result equally likely); **loaded die**; **fair
tosses** (what comes out; never "bits"); **entropy** (named once, at 10_3);
**independent** (named once, at 03_7, for "one toss doesn't affect the
next"). On screen, "model" marks a number worked out from the chances; the
01_1 tag says what it means. The players are **Alice** and **Bob**.

Numbers: the coin's chance is said in fifths ("four times in five", "four
fifths"); the size of a block is a share of the square, in percent (M15).

## 01_call_it (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | This coin isn't fair. It lands heads four times in five. Alice and Bob need it to settle a bet, and each suspects the other knows the coin better. They can still do it, perfectly fairly, using nothing but this coin. | One coin, tossed again and again: the first results drop one by one as discs, heads blue, tails pink, then the rest of the run's thousand tosses pour into a grid of ticks, and the bar under them settles leaning far toward blue with its count, "1,000 tosses: 791 heads" (`E-S1.call_it.*`; the whole run, so the count shown is the count said); Alice (green) and Bob (violet) either side, each eyeing the other. Under the last sentence a second bar grows beneath the first: level, green and violet, captioned only "the same coin" (C1; it is 02_9's bar, `E-S2.trick.*`). A small tag by the coin: "a model coin (one we made up): heads 80%" (`E-F1.heads_percent`) | lagged disc row; live bar; the payoff as a paradox |  |
| 01_2 | *(title)* | **How to Make a Fair Coin Out of an Unfair One** | title card | listen 3 s (the motif: two notes, then the same two swapped) |
| 01_3 | The obvious way is to call it. One toss, heads or tails. But whoever gets heads wins about four bets in five. | One toss between them, called; then the thousand tosses' bar again with its two counts, 791 and 209 (`E-S1.call_it.heads`, `.tails`), the heads end marked as the caller's wins; caption "1,000 tosses" | live bars and counters; a grid of ticks |  |
| 01_4 | So a call is fair only if the caller has no idea which side the coin favours. And neither of them can check what the other knows. | The coin between them, a question mark over each player's head: what does the other one know? The bar's winning end flickers between their two colours | highlight; morph |  |
| 01_5 | An ordinary coin, tossed and caught, can't be weighted to do this. But a spun coin can lean well to one side, and a loaded die as far as you like. | Three small pictures in a row: a coin spinning on its edge, a loaded die, a jittery trace captioned "and, far more mildly, the noise inside a computer"; lower-third "weighting can't make a tossed-and-caught coin favour a side by much (Gelman and Nolan, 2002). Our coin stands for the others" | lagged reveal |  |
| 01_6 | ⟨predict⟩ So how could they make the bet fair, whatever either of them knows about the coin? | The coin alone at centre, Alice and Bob either side; the question as a card | prediction card | hold |

- **R1:** the cold open now shows the payoff as a paradox (C1); 01_5 is
  three icons (C8, M20, R11); the penny counts and "some spun coins came up
  fewer than 10% heads" (Diaconis, Holmes and Montgomery 2007, p. 218) go in
  the description.
- **P3 (claim walk), the hook's logic:** two earlier versions did not follow.
  "Neither knows the coin" makes a blind call an even bet already (M10, R7);
  "Alice knows and Bob doesn't" is cured by letting Bob call, since a call
  made in ignorance wins exactly half the time. What is true, and what the
  trick answers: whoever gets the favoured side wins four bets in five, so a
  call is fair only if the caller knows nothing about the coin, and neither
  player can check what the other knows. The trick is fair whatever either
  of them knows (03_6).
- **P3:** 01_5 no longer says computer noise can be "this unfair": Jun and
  Kocher report raw frequencies within about a hundredth of one half. "An
  ordinary coin" and "weighted" keep the claim to what Gelman and Nolan show
  (a two-headed coin is not their subject), and the lower-third keeps their
  "substantially" as "by much".
- **Misconception defused:** "let the one who doesn't know the coin call"
  (01_4): fair only if you can trust that he doesn't.
- **Ruth:** "fair" here means fair whatever the coin is and whatever either
  player knows about it. The coin is a model and says so (01_1 tag).
- **Dev:** the jittery trace of 01_5 is paid off at 11_1 and 11_2.

## 02_toss_twice (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Toss the coin twice, and look at the two results as a pair. | Two discs land side by side and a bracket joins them: one pair | bracket |  |
| 02_2 | A pair can be four things. Two heads, two tails, heads then tails, or tails then heads. | Four pair tiles appear one by one, each two discs: the two matching pairs, then the two that differ | lagged reveal |  |
| 02_3 | Picture every way the two tosses could go as a square. The first toss cuts it in two, with four fifths of the width for heads. | A unit square; a cut drops at four fifths of the way across; under the bottom edge a blue stretch "first toss: heads" and a short pink one "tails" | **the unit square**; grow |  |
| 02_4 | The second toss cuts it again, four fifths of the way up, at the same mark, because it's the same coin. | A second cut runs straight across at four fifths of the way up; the left edge takes "second toss: heads" (blue) and "tails"; a faint five by five grid of small squares shows through; each of the four blocks takes its pair tile | the unit square; the tiles settle onto the blocks |  |
| 02_5 | A block's share of the square is its pair's chance. Count the small squares. Two heads has sixteen of the twenty-five, and two tails has one. ⟨predict⟩ What about heads then tails, against tails then heads? | The big block counts up to 16, the small corner to 1 (`E-F1.square.hh.exact`, `.tt.exact`: 16/25 and 1/25); the two thin blocks pulse with question marks; all four blocks still neutral (C25) | count-up; highlight |  |
| 02_6 | Four each. Exactly equal. Four in twenty-five is sixteen percent. | Both thin blocks count to 4 (`E-F1.square.ht.exact`, `.th.exact`: 4/25); each then reads 16% (`E-F1.square.ht_percent`, `.th_percent`) | count-up; highlight |  |
| 02_7 | Alice calls heads then tails. Bob calls tails then heads. If the pair matches, throw it away and toss two fresh ones. | The heads-then-tails block turns green and takes Alice's name; the tails-then-heads block turns violet and takes Bob's; the two matching blocks fade to grey with a loop arrow "toss two fresh ones"; lower-third "for programmers: fresh tosses for every pair. Reuse a toss from a thrown-away pair and you get plain heads-or-tails back" (D7) | highlight; morph |  |
| 02_8 | A thousand games with our coin. Alice wins four hundred ninety-four. Bob wins five hundred six. | The run's first eight games play beside the square at reading speed (`E-S2.trick.first_games`), each pair landing as a dot in its own block (a toss is heads when its draw falls left of, or below, the cut: E25); then all 3,128 pairs of the run land (`E-S2.trick.pairs`): 2,128 in the grey (`E-S2.trick.thrown_away`), 494 in the green, 506 in the violet (`E-S2.trick.heads_first`, `.tails_first`); two counters tick; a faint band on the game's bar, "a perfectly fair game rarely comes out 500 each: 469 to 531, about 19 times in 20 (model)" (`E-F2.fair_band.games1000`; M8, D14); caption "1,000 games, the same unfair coin" | **every pair as a dot in its block**; live counters | listen 3 s (a thrown-away pair is a soft tick, a deciding pair the motif's two notes in the order they fell, rising for Alice and falling for Bob; then the race, the two as an even patter) |
| 02_9 | The coin is as unfair as ever. The game isn't. | The coin's own bar (791 to 209) above the game's bar (494 to 506): the two bars of 01_1 | bars | hold |

- **R1:** why a share of the square is a chance is now said and countable
  (M2): the square is every way two tosses can go, and at four fifths the
  blocks are 16, 4, 4 and 1 small squares of 25. The thousand games are drawn
  in the square itself (E25, C13, C3). The first eight games are all shown
  (C18).
- **The payoff is previewed here** on honest seeded data, before any "why".
- **Ruth:** "exactly equal" is 4/25 against 4/25; 494 to 506 is one seeded
  run of 1,000, inside the band shown. The square's straight second cut
  already assumes one toss does not affect the next; 03_7 says so (R20).
- **Engine:** one `Square` prop for the whole video: four flat unit polygons
  stretched by placement about their own corners from three trackers (the
  first cut, and the second cut in each column), readouts in a fixed column
  beside it (E4, E10). Dots appear by opacity and never move.

## 03_every_coin (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Why exactly equal? Heads then tails is four fifths wide and one fifth tall. Tails then heads is one fifth wide and four fifths tall. | The green block's width and height take the labels 4/5 and 1/5; the violet block's take 1/5 and 4/5 | labels on the blocks |  |
| 03_2 | Flip one across the diagonal and it lands on the other. The same two numbers, multiplied in the other order. And multiplying doesn't care about order. | A gold diagonal appears; a copy of the green block lifts, flips across it with its two labels riding along, and lands exactly on the violet one; only then do the two products settle beside the square as one equation, 4/5 × 1/5 = 1/5 × 4/5 (C2, E13) | **the mirror flip**; whole factors trade places (E5) | listen 3 s (the motif's two notes on the lift, the same two swapped on the landing) |
| 03_3 | ⟨predict⟩ Make the coin more unfair. Heads nine times in ten. Are the two blocks still equal? | A slider "chance of heads" under the square moves from 80% to 90% (its readout is in percent, like the 01_1 tag); both cuts follow it; the five by five grid fades; the readouts become question marks | slider on a value tracker; prediction card |  |
| 03_4 | Yes. Nine percent each. Slide the cuts anywhere, even past the middle so the coin favours tails. The two blocks stay mirror images. | Both blocks read 9% (`E-F4.p90.ht_percent`); the slider then sweeps to 99%, where the two blocks are hairlines and still mirror images, back down through 50% to 20%, and home to 80%; a dot where the two cuts cross rides the gold diagonal the whole way (C12); the two live readouts are always equal (`E-F4.*.ht`) | **live recomputation from a tracker** |  |
| 03_5 | In letters, call the chance of heads p, so tails gets the rest, one minus p. Heads then tails is p times the rest. Tails then heads is the rest times p. Equal for every p. | The square's edges take the labels p and 1 − p; the equation's numbers become letters, factor by factor: p(1 − p) = (1 − p)p | numbers → letters, one factor at a time (E5) |  |
| 03_6 | So nobody needs to know how unfair the coin is, or which side it favours. And knowing wouldn't help either of them. | The slider drifts on its own with its number covered; both readouts are question marks joined by a gold equals sign that never flickers (C19); lower-third "for programmers: nothing to measure first, and no stored constant to go stale" (D13) | highlight; tracker |  |
| 03_7 | The trick does lean on two things. The coin's chance is the same on both tosses. And one toss doesn't affect the next, which is what independent means. | Two cards, each pointing at the square: "the same chance on both tosses" at the second cut sitting at the same mark as the first; "one toss doesn't affect the next" at the second cut running straight across both columns (R20); lower-third "enough: independent tosses with the same probability p of heads, for any p strictly between 0 and 1" | statement cards tied to the picture | hold |

- **The owner's sentence** "since the trials are independent, the
  probabilities commute" is 03_2 (the flip is the swap) and 03_7.
- **R1:** the flip is the multiplication (C2); the answer to the prediction
  is said (M18); the sweep goes below one half (R19, M18) and parks at 99%
  (C13); the result tiles are gone (C13, M8); "one minus p times p" is no
  longer spoken (M16); "independent" names only the second condition (R3,
  M17) and the lower-third says "enough", not "precisely".
- **Misconception defused:** "it works because the coin is close to fair" →
  the hairlines at 99%.
- **Ruth:** the areas are equal at 0 and 1 too, but then no pair ever
  decides; "strictly between" is the exact statement
  (`edge_heads_tails_equals_tails_heads_for_every_chance`). The two cards are
  exactly what 05 breaks, one each.

## 04_the_price (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | The trick has a price. With our coin, most pairs get thrown away. A pair decides only thirty-two times in a hundred. | The square at 80%; the two grey blocks lift to a lighter grey and read 68% together, the green and violet 32% (`E-F2.thrown_away_percent`, `.decides_percent`) | the unit square; highlight |  |
| 04_2 | ⟨predict⟩ So how many tosses does a game take, on average? | The question as a card; one game plays slowly: pair, pair, pair, decided | prediction card |  |
| 04_3 | A pair decides about one time in three, so about three pairs. That's about six tosses. Exactly, six and a quarter. | Under the square: "2 tosses ÷ 0.32 = 6.25 tosses a game (model)" (`E-F3.expected_tosses`) | TeX; count-up |  |
| 04_4 | Ten thousand games agree. They average six point two one tosses. | A histogram grows from the 10,000 seeded games, one bar for each number of tosses, 2, 4, 6 and on (`E-S4.price.pairs_1_to_8`); a marker at the mean, "counted in 10,000 games: 6.21" (`E-S4.price.mean_tosses`); lower-third "for programmers: rejection sampling, a retry loop with no upper bound. If you cap it, the cap returns no result, never a default winner" (D4) | **histogram growing from data** |  |
| 04_5 | A fair coin still averages four tosses. A coin that lands heads ninety-nine times in a hundred averages about a hundred and one. | The slider sweeps: at 50% half the square is coloured and the line under it reads 4; at 99% the colour is two hairlines and it reads 101 (`E-F4.p50.expected_tosses.exact`, `E-F4.p99.expected_tosses_rounded`); then back to 80% | the unit square on its tracker; live number |  |
| 04_6 | And nobody can promise when a game will stop. The longest of our ten thousand ran sixty tosses. | The histogram's tail as a row of single ticks under the axis, one for each game of 18 tosses or more (`E-S4.price.pairs_9_plus`), the last at 60 (`E-S4.price.longest_tosses`); "after 20 tosses: 2.1% still going (model)" (`E-F3.undecided_after.10_pairs_percent`); lower-third "no rule that always names a winner within a fixed number of tosses is fair for every coin: it would have to hand an unbroken run of heads to somebody" (`E-X1.*`) | ticks under an axis | hold |

- **R1:** the square carries the price (C7): the grey is what is paid. The
  answer is built from 04_1's number (M6); everything is in tosses (M7, D12,
  E15); the curve is gone (C7) and the tail is ticks, since a bar for the
  longest game would be a quarter of a pixel (E7).
- **Ruth:** "on average" is said each time (R12). A game ends with
  probability one, and for every n it is still going after n pairs with
  chance 0.68ⁿ > 0 (lower-third or description; R6). The lower-third's rule
  must name a winner: "toss twice, otherwise no result" is fair for every
  coin and names none (R8, D4).
- **Dev (lower-third at 04_4, for programmers):** rejection sampling: a retry
  loop with no upper bound. If you cap it, the cap returns no result, never a
  default winner (D4).

## 05_where_it_breaks (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | ⟨predict⟩ Now break it. Toss our unfair coin first, and a fair coin second. Is heads then tails still equal to tails then heads? | Two coins: the unfair one and a second, fair one; the square's second cut gets a slider of its own | prediction card |  |
| 05_2 | No. One cut moved and the other didn't, so the blocks aren't mirror images any more. Forty percent against ten. Alice wins four games in five, as if they'd never bothered. | The second cut slides from four fifths down to one half while the first stays; green grows to 40%, violet shrinks to 10% (`E-F5.two_coins.ht_percent`, `.th_percent`); the dot where the cuts cross leaves the gold diagonal; the flip of 03_2 replays and the green copy overhangs the violet block, the overhang red (C12); caption "counted in 1,000 games: 807 for Alice" (`E-S5.two_coins.heads_first`) | the unit square, one cut on its own tracker; the mirror flip | listen 3 s (the even patter of 02_8, then the same two notes four to one) |
| 05_3 | A sneakier break is a coin with a memory. It looks fair, but it copies its last result four times in five. | A different-looking coin, tagged "a model: half heads in the long run, repeats its last result 80% of the time" (`E-F6.sticky.long_run_heads.exact`, `E-F6.sticky.repeat_percent`); a row of its tosses, runs of one colour; caption "counted in 200,000 tosses: 50.0% heads" (`E-S6.sticky.heads_percent`); lower-third, here and gone before the next line, "real tossed coins do this a little: about 51 in 100 land the way they started (Diaconis, Holmes and Montgomery, 2007)" | a row of runs |  |
| 05_4 | Say a pair of heads was just thrown away. Now heads then tails needs one change, and tails then heads needs two. So heads then tails is four times as likely, and after two tails it's the other way round. Whoever saw the last pair knows who is ahead. | One thing at a time (R2: M3, D7, C5): large in the right column, the two ways on from a head as disc rows, head, head, tail (one change) and head, tail, head (two changes), each change ticking as it is said; then the square for the pair after two heads: the first toss cuts at four fifths, but the two columns are now cut at different heights, a step in the second cut; green reads 16% with "4/5 × 1/5" in it, violet 4% with "1/5 × 1/5" (`E-F6.sticky.after_heads.*`); caption "counted in 200,000 tosses: after two heads 6,306 against 1,615; after two tails 1,681 against 6,442" (`E-S6.sticky.after_hh.*`, `.after_tt.*`), the square flipping to its mirror image for the second, its readouts blank while it moves (R2: R5); under the last sentence, "the wins still split half and half overall; after a thrown-away pair of heads Alice is ahead 8 to 5 (model)" (`E-F6.sticky.plain_rule.*`; R2: R2, M2) | **the square with a step in its cut**; live readouts |  |
| 05_5 | The cure is to use only the tosses that come straight after a head. Each of those lands heads four times in five, whatever came before it. So they behave like a plain unfair coin, and the trick handles that. | A row of this coin's tosses; every disc that follows a blue one drops to a second row, which is the plain unfair coin of chapter 02, tagged "after a head: heads 4 in 5"; a disc that follows a pink one greys out, "not used: it follows a tail"; the second row is then bracketed into pairs as chapter 02's games were (R2: D2, M15); on the square the step slides level and the gold diagonal's dot returns to the line; caption "200,000 tosses, 15,890 games: 7,847 against 8,043" (`E-S6.sticky.after_heads_only.*`); lower-third "for a coin that remembers only its last toss: Samuelson, 1968. Fair when played to the end. Using every toss takes more care: Blum, 1984" (`E-C2.blum.focs_year`) | lagged filter of a row; the unit square | hold |

- **R1:** memory is one picture, the step (C8, C23), and two beats. The coin
  is visibly a different coin that looks fair by count (M4, D5); the reason
  the cure works is said (M5); the real-coin note is a lower-third with its
  condition (R10, D15, R23) and is not spoken; "knows who's about to win" is
  gone (R5); the cure's limits are on screen (R13, D3).
- **Ruth:** 16% against 4% is exact for this model. Played to the end from
  any start, Alice's chance under the plain rule is 8/13 after a head and
  5/13 after a tail (`E-F6.sticky.plain_rule.*`), and exactly one half from a
  long-run start: that is why a tally of wins does not show the break
  (description). The cure is exactly fair when played to the end
  (`edge_after_only_rule_is_fair_for_every_memory`) and leans under a
  deadline (fact sheet, finding 4).
- **Dev (description):** Blum's point: with "after a head" and "after a
  tail" as two coins you may use every toss, but each result must be
  announced only when the coin is next in that state (`E-F7.blum.*`).

## 06_same_counts (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Back to the trick. Heads then tails and tails then heads hold the same results, one head and one tail, in a different order. | The square at 80% again; the disc pairs lift out of the green and violet blocks; a gold link joins each blue disc to its twin and each pink to its twin | the unit square; morph that keeps identity |  |
| 06_2 | Try three tosses. Two tosses needed a square, so three need a cube. | The square tips back and gains a third side, cut at four fifths like the others: a cube in eight pieces (drawn by projection on the fast route; E1, C3) | **the square grows into a cube** |  |
| 06_3 | ⟨predict⟩ Two heads then a tail. Heads, tails, heads. A tail then two heads. Which of these three patterns is likeliest? | Three slabs light up in the cube, one as each pattern is named, each carrying its pattern as three discs; the word "pattern" under the first | lagged reveal; prediction card |  |
| 06_4 | None of them. Each is four fifths, times four fifths, times one fifth, in some order. The same slab, turned three ways. For tosses like ours, with the same chance every time and no memory, order never changes a chance. | The three slabs slide out of the cube and turn until they stand side by side, identical; each reads 12.8% (`E-F8.three.heads2.each_percent`) | **slabs slide out and align**; glow |  |
| 06_5 | So sort every pattern by its counts, how many heads and how many tails. Same counts, same chance. Two tosses give groups of one, two and one, and the middle group is our trick. | The square's four blocks slide into three groups by counts, each group labelled with its counts (two heads; one head and one tail; two tails); the words "same counts, same chance" in gold; group sizes 1, 2, 1 under them (`E-F8.pascal.row2`); the middle group is the green and the violet block | **patterns reorganize into groups**; count-up |  |
| 06_6 | Three tosses give one, three, three, one. Four tosses give one, four, six, four, one. That's Pascal's triangle. | The cube's eight pieces sort the same way: 1, 3, 3, 1 (`E-F8.pascal.row3`); four tosses have no picture, so their sixteen patterns arrive as disc strips already in groups, 1, 4, 6, 4, 1 (`E-F8.pascal.row4`); the three rows of group sizes stand as a triangle, the 6 glowing, and hold | lagged group reveals; a grid of group sizes | listen 2 s (the finished triangle) |
| 06_7 | The coin's unfairness only decides which group you land in. Inside a group, every pattern is equally likely. The counts carry the unfairness. The order is perfectly fair. | The row for four tosses as five boxes side by side, its patterns as equal tiles inside each; over each box a bar for the chance of landing there (`E-F8.row4.heads0_percent` … `heads4_percent`); a slider moves from 50% to 80% and the bars pour toward the heads end while the tiles inside each box stay equal; under the last two sentences they stand as a card, the second in gold | bars on a tracker; live recomputation | hold |

- **R1:** the square never leaves (C3): three tosses are the same picture
  with a third side, and "same counts, same chance" is first seen as one
  slab turned three ways. 06_4's claim is fenced to tosses with one chance
  and no memory, since 05 has just shown a coin where order changes a chance
  (R4, M12, the orchestrator); the question comes before the claim. "Counts"
  is defined at 06_5, and the gold words wait for it (M13, claim walk);
  Pascal's triangle is said aloud and held (M22, C14); the bar of 06_7 is
  five boxes, because two of its five stretches were under a pixel wide (E9,
  C14).
- **P3:** the prediction has a beat of its own (06_3), so the three patterns
  are three short sentences a voice can read and the line still ends with its
  question.
- **This is the idea the dice need:** order is fair given the counts. 06_7 ends on
  the act's recap (plan rule 14).
- **Ruth:** for independent tosses of one coin a pattern's chance depends only
  on its counts (`edge_same_counts_same_chance_without_memory`, every pattern
  of up to six tosses; the same test shows it failing for the coin with
  memory). The groups are Elias's "composition classes" (1972, p. 868).
- **Engine:** the cube is eight boxes drawn as projected polygons, opaque,
  with one tracker for the tilt; it is the prop chapter 09 uses.

## 07_a_die_from_a_coin (~50 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | ⟨predict⟩ Look at the middle of the row for four tosses. Six patterns, all equally likely. What else has six equally likely results? | The middle box of 06_6 lifts out: HHTT, HTHT, HTTH, THHT, THTH, TTHH (`E-F8.four_two.list`), and stays up for the whole pause (M32) | highlight; lagged reveal |  |
| 07_2 | A die. Toss the coin four times. If exactly two are heads, the pattern picks the face. If not, toss four fresh ones. | Each of the six patterns morphs into a die face, one to six; the other four boxes fade to grey with the loop arrow "toss four fresh ones" | **morph: patterns → die faces** |  |
| 07_3 | Twenty-four thousand tosses of our unfair coin made nine hundred forty-nine rolls of a fair die. Each face came up about as often as a fair die's would. | The run's first eight blocks of four play at reading speed, the accepted ones dropping as die faces; then a grid of 6,000 cells lights, grey for a block thrown away and gold-edged for a roll, while six bars rise to 166, 161, 136, 160, 167, 159 (`E-S7.coin_die.faces`); a faint band across the bars, "a fair die: 136 to 181 (model)" (`E-S7.coin_die.fair_band`); caption "24,000 tosses, 949 rolls" (`E-S7.coin_die.tosses`, `.dice`) | bars growing from data; a grid recoloured in place (E6) |  |
| 07_4 | It isn't cheap. A coin this unfair rarely gives exactly two heads in four. A roll costs about twenty-six tosses, on average. | The five boxes of 06_6 with their bars at 80%; the middle one lit: "15.4% of tries" (`E-F8.four_two.works_percent`); "model: 26.0 tosses a roll. This run: 25.3" (`E-F8.four_two.expected_tosses`, `E-S7.coin_die.tosses_per_die`) | bars; live numbers |  |
| 07_5 | Any group of patterns with the same counts is a fair choice among them. | Pascal's triangle returns and each entry glows in turn: a fair choice among that many | glow sweep over the grid | hold |

- **The owner's example** (C(4, 2) = 6: "a fair six-sided die made from a
  bent coin"), with its honest cost.
- **R1:** one cost is said, the one that checks against the screen (M23); the
  cheaper rule is in the description, stated so that it cannot be coded
  wrongly: "in three tosses that aren't all alike, the odd one out is equally
  likely to be first, second or third; that fair one-in-three and one coin
  trick make a die for 12.5 tosses" (`E-F8.three_plus_coin.expected_tosses`;
  C21, D8, R16). The 24,000 tosses are a grid and six bars (E6).
- **Ruth:** the die's fairness over 949 rolls is a chi-squared check in the
  tests; the band is two standard errors about 949/6.

## 08_loaded_dice (~45 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Now swap the coin for an unfair die. This one is loaded to show a six half the time. | A die; six bars for its faces, 5, 5, 10, 10, 20 and 50 percent (`E-F9.loaded.percents`), pointed at, not read; caption "counted in 6,000 rolls: 318, 283, 579, 607, 1,237, 2,976" (`E-S8.loaded.faces`) | bars |  |
| 08_2 | ⟨predict⟩ Alice and Bob want to settle their bet with it. How would you do it? | Alice and Bob either side of the die; the question as a card | prediction card |  |
| 08_3 | Roll it twice. If the two rolls match, roll two fresh ones. If not, Alice wins when the first roll is bigger, and Bob wins when the second is. | The square again, cut six ways along each side by the die's chances, the six's wide stretch nearest the corner, as heads was: thirty-six blocks; those above the diagonal turn green, those below it violet, the six on it grey; the gold diagonal returns | **the square, cut six ways: 36 blocks**; lagged colouring |  |
| 08_4 | It's the coin trick again. A six then a five is exactly as likely as a five then a six, and across the diagonal every block has an equal twin. | One green block (a six then a five) lifts, flips across the diagonal and fits its violet twin (a five then a six); then the whole green triangle flips onto the violet one, as one sheet; still caption "counted in 3,000 pairs of rolls: 1,021 for Alice, 1,020 for Bob, 959 matches" (`E-S8.two_rolls.*`) | **the mirror flip**, fifteen blocks at once (E13) |  |
| 08_5 | And a coin is just a die with two faces. | The die's cuts slide together until only one is left on each side: the thirty-six blocks are the coin's four, green and violet where they were; hold; then out again; the gold words "the same two results, either order: the same chance" | **the six-way grid becomes the coin's square** | hold |

- **The owner's "unfair dice", first half.** 08_5 is the hindsight moment
  (plan rule 19), on its own beat with a hold (C5).
- **R1:** Alice's green is above the diagonal in both squares, because a
  bigger roll, like heads, is drawn nearer the corner (C6, D2, R14, M1, E8,
  E17); the flipped block is the right one. "Roll two fresh ones" (D6).
  Three rolls move to chapter 09 (C4, M3).
- **Ruth:** the two orders of two *different* rolls are equally likely;
  matches are thrown away. 1,021 to 1,020 is one run, not a law. With this
  die two rolls decide 68.5% of the time, 2.9 rolls a decision
  (`E-F9.two_rolls.*`; description).

## 09_six_orders (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Three rolls can make a fair die. Three need the cube again, cut by the die's chances along every edge. | The six-way square of 08 tips back and gains its third side (E26): the cube of 06, now cut six ways each way; only the cuts on its three back faces are drawn (E3) | **the square grows into a cube**, again |  |
| 09_2 | ⟨predict⟩ Say the rolls are a four, a five and a six, in some order. Each order lands in one of these six bricks. How do the bricks compare? | Six bricks light up inside the cube, one neutral fill with gold edges, told apart by position (C15); the cube turns a little and stops; then each brick takes its order as a label (4 5 6, 4 6 5, …) | glow; a tracker turns the cube |  |
| 09_3 | They're one brick, turned six ways. A tenth, by a fifth, by a half. The same three rolls have the same chance in any order. | The six bricks slide out and turn until they stand in a row, identical; one is labelled 1/10 × 1/5 × 1/2 = 1/100 (`E-F9.brick.*`); the gold words "the same rolls, any order: the same chance" | **bricks slide out and align** | hold |
| 09_4 | So when three rolls all differ, the order they came in is a fair choice among six, and it picks a face. Low, high, middle is one order. High, low, middle is another. If any two match, roll three fresh ones. | Three dice land: four, six, five, and a small chart ranks them low, high, middle; then a different trio, six, one, three: high, low, middle; six order cards, the six ways to arrange low, middle and high, drawn as rank charts with no digits (the face comes only from the order, never from which numbers came up; R2: D1, R1, M24), and each brick in the row takes one and becomes a die face | lagged cards; morph |  |
| 09_5 | Six thousand rolls of the loaded die made six hundred fifty rolls of a fair one. | Six bars rise to 119, 108, 103, 105, 112, 103 (`E-S8.three_rolls.orders`); a faint band, "a fair die: 90 to 127 (model)" (`E-S8.three_rolls.fair_band`); caption "6,000 rolls, 650 fair ones" (`E-S8.loaded.rolls`, `E-S8.three_rolls.dice`) | bars growing from data |  |
| 09_6 | It keeps going, up to six rolls. Four different rolls can come in twenty-four orders. But rolls that all differ get rarer, so each step costs more. | "4 × 3 × 2 × 1 = 24" (`E-F9.orders.4`); lower-third "with this die: three rolls all differ 33 times in 100, about 9 rolls for each fair one; four all differ 10 times in 100, about 39 rolls (model)" (`E-F9.three_rolls.works_per_hundred`, `.expected_rolls_rounded`, `E-F9.four_rolls.*`) | count-up | hold |

- **The owner's "unfair dice", second half**, with a chapter of its own (C4,
  M3): the cube asks the question, the bricks answer it, and only then do
  the order cards and the measured rolls arrive. One trio (four, five, six)
  throughout.
- **Why 3D earns its place:** the square's mirror image, one dimension up;
  six congruent bricks are the 3! orders, as three congruent slabs were the
  three patterns of 06.
- **Ruth:** the orders of *different* values; the chance of each order is the
  same symmetric sum over trios
  (`edge_every_order_of_different_rolls_is_equally_likely`). The face comes
  only from the order, never from which numbers came up. A six-sided die
  stops at six different rolls, and the cost climbs steeply (R15): shown for
  three and four.
- **Engine:** bricks are projected polygons in a parallel projection, so six
  congruent bricks look identical; labels are plain text placed while the
  cube is still (E1, E2, E12). No camera route.

## 10_what_is_left (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | The trick comes from the mathematician John von Neumann, in a talk in nineteen forty-nine. The printed summary calls it rigorously unbiased, which is to say exactly fair. It also names the price. At most twenty-five percent as efficient as ordinary coin tossing. | Quote card, verbatim, revealed clause by clause: "If independence of successive tosses is assumed, we can reconstruct a 50–50 chance out of even a badly biased coin by tossing twice. If we get heads-heads or tails-tails, we reject the tosses and try again. If we get heads-tails (or tails-heads), we accept the result as heads (or tails). The resulting process is rigorously unbiased, although the amended process is at most 25 percent as efficient as ordinary coin-tossing."; "heads-tails" green, "tails-heads" violet, "rigorously unbiased" gold; two-line source "John von Neumann, Various Techniques Used in Connection With Random Digits (summary written by George E. Forsythe). Monte Carlo Method, 1951, p. 36. From a symposium held June 29 to July 1, 1949" (`E-C1.*`); beside the last clause the square at 50%: "a fair coin: one fair toss for every four" (`E-F10.fair_coin.von_neumann.percent`) | quote card that never truncates (E11) |  |
| 10_2 | ⟨predict⟩ Our coin gives less. These hundred tosses made fifteen fair ones, and the average is sixteen. Could a better rule get all hundred? | A strip of the first hundred tosses of run 9, 77 of them heads; its fifteen fair tosses drop out of it, captioned with their own count (`E-S9.squeeze.first_hundred.*`) and the whole run's (`E-S9.squeeze.von_neumann.*`); beside it a scale from 0 to 100 carrying the average, labelled as the model: "the trick, on average: 16 per 100 (model)" (`E-F10.von_neumann.per_hundred`) | count-up; number line |  |
| 10_3 | No. Guess heads and you're right four times in five, so each toss of this coin holds less than a fair one. The most any rule can average is just over seventy-two. That ceiling is called the coin's entropy. | A ruled line at 72.2 on the scale (`E-F10.entropy.per_hundred_1dp`), the trick's 16 far below it; lower-third "average fair tosses per 100, over a long run: at most 72.2 (model). Name to search: entropy" | number line; highlight |  |
| 10_4 | You get closer by wasting less. Use the bigger groups in deeper rows of the triangle. Or run the trick again on the two things it ignored, which pairs matched and what the matching pairs showed. Both climb toward the ceiling, and neither ever reaches it. | Two panels sharing the scale. Left: the triangle grows rows downward as bars climb for the rows for 4, 8, 16, 32 and 64 tosses (`E-F10.elias.*.per_hundred`). Right: the square gives up two new coins, each labelled as it drops out: "which pairs matched" (the grey part against the coloured part) and "what the matching pairs showed" (the two grey blocks); the trick runs on each, and on what those leave, round after round; bars climb for one, two, three rounds and on (`E-F10.peres.*.per_hundred`); dots for 100,000 seeded tosses sit on the bars: 16.1, 28.7, 38.8, and 69.6 when nothing is left to reuse (`E-S9.squeeze.*`); the bars' tops bend under the ruled line; caption "the price: memory and waiting"; lower-third "deeper rows: Elias, 1972. Reusing both leftovers: Peres, 1992" | bars growing toward a limit; the unit square splits off two new coins; the grid of group sizes | hold |

- **R1 blocker fixed (D1, R1):** the old words named only the thrown-away
  pairs; that alone levels off near 18 per hundred. Peres reuses two things,
  and 10_4 now says the trick is run again and again on the two things it ignored, while the picture labels both ("which pairs matched", "what the matching pairs showed")
  (one more round gives only 29 per hundred; claim walk); the picture shows
  each as a new coin cut from the square (C16).
- **P3 (claim walk):** "however far you go, they stop just short of it" is
  true of every row and every number of rounds; only a rule whose rows grow
  without end reaches the ceiling, in the limit (Elias, Theorem 2). The bars
  shown rise along doublings; Elias's rate does not rise from every row to
  the next (the row for 3 tosses gives less than the row for 2).
- **R1:** von Neumann's paragraph and its price open the coda, so the price
  is not reopened at the very end (C9); the card starts one sentence earlier,
  where the text names its own condition (R22), and the voice says "the
  printed summary" (R17); the prediction is one a viewer can answer, and the
  ceiling gets a reason (M9); "on average", "just over seventy-two" and
  "never quite reach" are the true forms (R2, D11).
- **One picture, not a proof** (the order).
- **Ruth:** h(0.8) = 0.7219 bits per toss bounds the long-run average rate
  of any rule whose output is exactly fair (Elias 1972, Theorem 1) and is
  the limit of Peres's and Elias's rules (Peres, Proposition 3; Elias,
  Theorem 2): `edge_rates_climb_to_the_entropy_and_never_pass_it`. A single
  run of 100 tosses can give more than 72; the average cannot.
- **Dev (description):** the two leftover streams are a XOR b for every
  pair, and a for every matching pair; recurse on both.

## 11_the_fair_coin_inside (~45 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | ⟨predict⟩ Computers need fair tosses too, for passwords and secret codes. Many get them from electrical noise, and raw noise comes out a little unfair. What would you do with it? | The jittery trace of 01_5 becomes a stream of ones and zeros with slightly more ones (labelled "unfairness exaggerated") | stream of digits; prediction card |  |
| 11_2 | Pair the readings up, and throw away the pairs that match. In nineteen ninety-nine, one chip maker, Intel, built a corrector based on this trick into its random number generator. | The stream enters in pairs; matching pairs vanish and a balanced stream leaves; card "a hardware corrector based on a concept proposed by John von Neumann. Jun and Kocher, The Intel Random Number Generator, 1999" (`E-C4.intel.year`) | pair filter on a stream; quote card |  |
| 11_3 | So an unfair coin like ours still has a fair one inside it, as long as it can land both ways. You find it by setting aside how many heads came up, and looking only at the order. | The opening square at 80% with its gold mirror line, green and violet pulsing; under the second sentence it opens once into the six-way grid and closes again (C9); the motif resolves over the held square; then the receipts card: "every count and chance in this video was computed by code; quoted numbers are cited" with the repository link and "seeds 195101 to 195109, NumPy PCG64DXSM" (`E-S0.seeds`; D22), and names to search on two lines: von Neumann's trick; Samuelson, 1968; Hoeffding and Simons, 1970; Elias, 1972; Blum, 1984; Dijkstra's fair roulette, 1990; Peres, 1992; "How to turn loaded dice into fair coins", 2000 | the unit square; receipts card | listen 4 s (the motif, resolved) |

- **Ends on the rigorous form of the opening picture** (plan rule 15), with
  the dice beside it (C9).
- **R1:** the hardware beat is now one a fourteen-year-old can answer (M11);
  "one chip maker", "based on" and "a little lopsided" are what the source
  supports (D9, R18): Jun and Kocher report raw frequencies near one half and
  say Intel "enhanced" the corrector. The last line no longer contradicts
  itself (R9).
- **Ruth:** "as long as it can land both ways" is the edge of 11_3's claim.
  NIST's final SP 800-90B does not list the method; nothing here says it is
  approved anywhere.
- **Corpus threads (receipts card):** Hoeffding (corpus entry 1) analysed
  this trick in 1970; Dijkstra (who posed Video C's problem) built a fair
  37-way roulette from a possibly unfair coin in 1990.
