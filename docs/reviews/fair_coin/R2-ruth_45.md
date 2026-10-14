# R2 review: ruth_45 (verbatim; paths made relative)

# R2 review, fair_coin: Professor Ruth Adler (ruth_45)

No blockers. Three majors (R1 to R3), all fixed by a label or a caption. Every number at rest on the sheets agrees with the fact sheet except one rounding (R4).

## Comprehension check

| ch | What I would tell a friend |
|---|---|
| 01 | Calling one toss of a coin that lands heads four times in five hands the bet to whoever gets the favoured side. Neither player can check what the other knows, so they need a procedure that is fair whatever anyone knows. |
| 02 | Toss twice. Heads-then-tails and tails-then-heads each take 4 of the 25 small squares, so Alice takes one, Bob the other, and matching pairs are re-tossed. A thousand seeded games split 494 to 506 with the coin as lopsided as ever. |
| 03 | The two deciding blocks are mirror images across the diagonal, p by (1 − p) and (1 − p) by p, so they are equal for every p and nobody needs to know p. It needs the same chance on both tosses and no influence of one toss on the next. |
| 04 | The price is the thrown-away pairs. A pair decides 32% of the time, so a game averages 6.25 tosses (4 at best, about 101 for a 99% coin), and no toss is late enough to be sure it is over. |
| 05 | Break either assumption and the mirror breaks. A fair second coin gives 40% against 10%. A coin that copies its last result tilts the next pair 16% to 4%, one way or the other depending on what was just thrown away. Using only tosses that follow a head gives back a plain unfair coin. |
| 06 | For independent tosses of one coin a pattern's chance depends only on how many heads it has. Patterns fall into groups of 1, 2, 1; 1, 3, 3, 1; 1, 4, 6, 4, 1, and the unfairness lives only in which group you land in. |
| 07 | The six patterns with two heads in four tosses are equally likely, so they are a fair die. It costs about 26 tosses a roll with this coin, because that group comes up 15.4% of the time. |
| 08 | With a loaded die, roll twice and let "first bigger" against "second bigger" decide. Every block above the diagonal has an equal twin below it, and a coin is the two-faced case. |
| 09 | Three different rolls can arrive in six equally likely orders (for 4, 5, 6: six bricks of 1/10 × 1/5 × 1/2). The order is a fair six-way choice: 650 fair rolls from 6,000 loaded ones, dearer with four. |
| 10 | Von Neumann's 1949 talk, in Forsythe's printed summary, gave the trick and its price. Our coin yields 16 fair tosses per hundred on average, and no exactly fair rule can average more than 72.2 (the entropy). Deeper rows and reusing both leftovers approach that and never reach it. |
| 11 | Hardware noise is cleaned the same way (Intel, 1999). You find the fair coin by ignoring how many heads came up and reading only the order. |

## Rubric

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 4 | 4 | 5 | 4 | 3 |
| 02 | 5 | 5 | 5 | 4 | 5 |
| 03 | 5 | 5 | 4 | 4 | 5 |
| 04 | 4 | 4 | 4 | 4 | 3 |
| 05 | 3 | 3 | 4 | 2 | 4 |
| 06 | 4 | 4 | 5 | 4 | 5 |
| 07 | 4 | 4 | 4 | 4 | 4 |
| 08 | 5 | 4 | 4 | 4 | 5 |
| 09 | 3 | 3 | 4 | 4 | 4 |
| 10 | 4 | 4 | 4 | 3 | 4 |
| 11 | 4 | 5 | 4 | 3 | 4 |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me | suggested fix |
|---|---|---|---|---|---|
| R1 | ruth_45 | 09_4, 09_5 (cards from 35.8 s, bars 50.6 to 57.8 s) | major | The six order cards carry the digits "4 5 6 … 6 5 4" and stay under the 09_5 bars. The bars do not count fours, fives and sixes. My replay of run 8 gives 119, 108, 103, 105, 112, 103 for every trio of three different rolls, sorted by order pattern. The literal sequence 4, 5, 6 has chance 1/100, about 20 of the 2,000 trios, not 119. The label says something other than what was counted and teaches that the rule needs a four, a five and a six. The storyboard's own note ("the face comes only from the order, never from which numbers came up") is in the description and contradicted on screen. | When the bricks become cards, drop the digits and keep the three-bar rank chart (words "low, high, middle" if a label is needed). Make 09_4's second example another trio: 6, 1, 3 for "high, low, middle", and show 2, 6, 3 landing on the same card as 4, 6, 5. Caption 09_5 "2,000 threes of rolls; 650 with three different rolls, sorted by their order". |
| R2 | ruth_45 | 05_3 (28.7 to 52.2 s) | major | The chapter says "break", shows 16% ≠ 4%, then both mirrored counts (6,306 against 1,615; 1,681 against 6,442). Add them and the wins are 7,987 to 8,057: even. That is right for this coin: Alice's chance from a long-run start is exactly 1/2 (`E-F6.sticky.plain_rule.long_run.exact`). But nothing on screen or in the voice says in what sense the game broke. Two wrong takeaways are on offer: "Alice now wins more" (false for this coin) and "it cancels, so nothing broke". What is lost is fairness given what has been seen: after a thrown-away pair of heads Alice wins 8 games in 13 (recomputed, with 5/13 after tails). At R1 (my R5) the figures went to the description only; the new picture now raises the question on screen. | Give the answer the lower-third slot now held by the unspoken real-coins note, which is already in the description: "the wins still split half and half overall; after a thrown-away pair of heads Alice is ahead 8 to 5 (model)". Or two spoken sentences after "the other way round": "Overall the wins still come out even. But anyone who saw the last pair knows which way to bet." |
| R3 | ruth_45 | 07_3 (band from about 31 s), 09_5 | major | "a fair die: 136 to 181 (model)" and "a fair die: 90 to 127 (model)" are ranges with no quantifier, drawn as one band across all six bars. Each is a two-standard-error band for one face taken alone (exact binomial: 95.5% for both). 02_8's caption says "about 19 times in 20"; these two do not. As drawn, the claim is that a fair die's six bars sit in the band. A fair die puts at least one bar outside about 23% of the time (400,000 simulated dice: all six inside 77%). Face 3's 136 sits on the edge: one fewer and the picture would have seemed to refute a fair die. The band's definition is not in the description either. | Captions: "a fair die: each face 136 to 181, about 19 times in 20 (model)", and the same at 09_5. Description: one line defining the three bands (two standard errors about the fair count; one count inside about 19 times in 20; all six faces inside about 3 times in 4). |
| R4 | ruth_45 | 06_7 (68.3 to 70.5 s, the cue-sheet frame) | minor | At "chance of heads 50%" the bars read 6%, 25%, 37%, 25%, 6%, which add to 99. The middle is 6/16 = 37.5% exactly, and the fact sheet has it as `E-F8.four_two.fair_coin.works_percent` = 37.5. Every other label in the chapter rounds to nearest. The same row is also printed at two precisions: 41% and 15% at 06_7, 41.0% and 15.4% at 07_4. | Print 37.5%, or label the fair stop in sixteenths (1/16, 4/16, 6/16, 4/16, 1/16), which shows the bars are the group sizes there. Use one precision for the row in both chapters. |
| R5 | ruth_45 | 05_3, 05_4 (49.0 to 50.5 s; 58.0 to 60.0 s) | minor | Live readouts run through the two morphs of the stepped square. Under the header "the pair after two tails" they read "11% ≠ 9%" then "7% ≠ 13%", with the two product labels printed over each other at 49.3 s. Under "two tosses that each follow a head" they read "6% ≠ 20%" then "16% ≠ 16%", while the voice says each has the same chance. The in-between squares are no coin in the story. | Blank the two readouts and the sign during each morph and switch the header when the square lands (R1's E24 already plans blanking). Never draw ≠ between two equal printed numbers. |
| R6 | ruth_45 | 04_3 (24.2 to 25.0 s), 04_5 (32 to 43 s), 10_1 | minor | The line under the square counts up through an equals sign: "2 tosses ÷ 0.32 = 4.08" is on screen as the voice says "six and a quarter". During the 04_5 sweep the printed divisor does not give the printed quotient ("2 tosses ÷ 0.41 = 4.84"; 2 ÷ 0.41 is 4.88), and at 37.2 s the slider reads 80% beside "0.33 = 6.15". The line also drops "on average", the chapter's subject. So does 10_1's "a fair coin: one fair toss for every four". | Write 6.25 in one step. Show the equation only at the stops (0.50 = 4.00; 0.0198 = 101.01) or blank it while the slider moves. "= 6.25 tosses a game, on average (model)"; "on average, one fair toss for every four". |
| R7 | ruth_45 | 02_8, 07_3, 09_5 (counters) | minor | The counters beside the landing data are ramps of the final counts, not tallies of what has landed. At 70.0 s and 70.8 s the score reads 103 to 106 and 360 to 369; in my replay of run 2 neither is ever the score. 07_3's 105, 102, 86, 101, 106, 100 at 27.5 s is 0.632 × the finals and never a true tally of run 7. 09_5's mid-count frames fit the same ramp (not replayed step by step). 01_3 does it properly: 212 heads and 58 tails after 270 tosses is the run's real count. A ramp holds the split at 49.4% from start to finish, hiding the wander that makes the band believable. | Tally what has landed, as 01_3 does (the 02_8 score then wanders inside its band), or show no numerals until the count is complete. |
| R8 | ruth_45 | 10_2 (strip, 22 to 28 s) | minor | The "15 fair tosses out of these 100" are 10 heads and 5 tails (replayed: TTHHHHHTHHHHTTH). Honest, and ordinary for a fair coin (10 to 5 or worse about 3 times in 10). But nine chapters have trained the eye that mostly blue means unfair. | Under the strip add the run's own tally from the fact sheet: "the whole run: 16,114 fair tosses, 8,047 heads" (`E-S9.squeeze.von_neumann.*`). |
| R9 | ruth_45 | 01_4 | minor | "A call is fair only if the caller has no idea which side the coin favours" is a probability over the caller's ignorance, not over the toss. For the coin on the table and the call actually made, the caller wins 4 in 5 or 1 in 5; it averages to one half only for a call unrelated to the coin's lean. The trick is fair in the stronger sense: for the fixed coin, over its tosses alone. That contrast is the video's point, and its exact version is recorded nowhere (no row in "Simplification → exact", nothing under "Precisely"). | Description: "A blind call is an even bet only averaged over the caller's choice: a caller as likely to name either side wins ½ · 4/5 + ½ · 1/5 = 1/2. For a given coin and a given call the chance is 4/5 or 1/5. The trick is fair for the coin as it is." Add the row to the fact sheet's table. |
| R10 | ruth_45 | 04_6 (44 to 53 s) | minor | The ticks under the axis have no label, and from 18 to 32 tosses they are solid blocks. Those eight slots hold 150, 101, 63, 42, 26, 19, 16 and 13 games (replay), so "one tick a game" is visible only from 34 on. The bars stop at 16, which reads as if almost no game ran longer. | Label the row "446 games of 18 tosses or more, one tick each" (`E-S4.price.pairs_9_plus`). Either stack the ticks upward or keep bars through 32 and start ticks at 34. |
| R11 | ruth_45 | 08_5 (43.5 to 46.5 s) | minor | While the grid is the coin's square the die's readouts stay: "six then five 10%, five then six 10%" beside blocks that are now 16%, and "959 matches" of 3,000 under a square that is 68% grey. | Fade the two 10% lines and the count caption while the picture is the coin's; bring them back with the six-way grid. |
| R12 | ruth_45 | 10_4 (right panel) | minor | The x-axis runs 1, 2, 3, 4, 6, 8, all at equal spacing. Rounds 5 and 7 are skipped without a mark (52.6 and 60.8 by my recursion), so the climb from 47 to 64 looks twice as steep as it is. | Draw all of rounds 1 to 8 (two new keys), or put break marks between 4 and 6 and between 6 and 8. |
| R13 | ruth_45 | 05_1 (5.5 to 12.8 s) | minor | For the whole question the vertical slider reads "second toss 80%" beside the legend "second toss: a fair coin, heads 50%". | Hide the second slider's number until the cut moves at "No.", or tag the square "so far: both cuts at 80%". |
| R14 | ruth_45 | 11_3 (receipts); description "Every number" | minor | The receipts card is up about 5 s (44.3 to 49.3 s) for some 60 words. The description says the runs "can be replayed … a toss is heads when the draw is below 0.8". Replaying them I needed three rules it does not give: run 5's second coin at 0.5; run 6's 0.8 after a head and 0.2 after a tail, started as if after a head; run 8's die, the first face whose running total exceeds the draw. There is also no run 3 in the video. | Hold the card 10 s or leave it as the end frame. Add the three rules and "run 3 is not used" to the description. |
| R15 | ruth_45 | 03_4; 01_5 | minor | Two holds are shorter than what they show (pacing is Maya's call). The park at 99% lasts about 0.7 s (37.6 to 38.3 s), and it is the frame that answers "it only works because the coin is nearly fair". 01_5's third card ("and, far more mildly, the noise inside a computer") is up about 2 s and never spoken. | Park 2 s at 99% with "1.0% = 1.0%" readable; bring the third card in with the second. |
| R16 | ruth_45 | 03_4 (42.5 s), 05_2 (20 s) | delight | The flip replayed at 20%, with the coin favouring tails, and then the red overhang where the flipped block no longer fits. The proof and its failure are the same gesture. | Keep both. |
| R17 | ruth_45 | 02_8, 08_5 | delight | The eight games drawn beside the square are the run's first eight, pair for pair (checked against my replay), and the dots are the draws themselves. The six-way grid closing to the coin's square at 44.5 s makes 08_5's line true in one move. | Keep. |
| R18 | ruth_45 | 06_7, 07_4 | delight | "41%, 41%" is not a bug: four heads and three heads tie exactly when p = 4(1 − p), which is our coin. A careful viewer will suspect a rendering error. | One line in the description. |

## Numbers on the sheets and how each was checked

FS = matched to a fact-sheet key. RC = recomputed with exact fractions. RP = my own replay of the seeded run (NumPy 2.4.1, PCG64DXSM, seeds 195101 to 195109).

| ch | numbers | how |
|---|---|---|
| 01 | 80% tag; 791 and 209; 19 and 1 after 20 tosses; 212 and 58 after 270; level bar split at 49.4% | FS; FS + RP; RP; RP; measured on a frame |
| 02 | 16, 4, 4, 1 of 25 and 16%; 494 and 506; first eight games; band 469 to 531 "19 in 20" | FS + RC; FS + RP; FS + RP; FS + RC (exact binomial 95.4%). Mid-count 103/106 and 360/369: see R7. |
| 03 | 4/5 and 1/5 labels and both products; 16%; 9% at 90%; 1.0% at 99%; 25% at 50%; 16% at 20%; passing values 23%, 18%, 7%, 3%, 24% | FS + RC; the passing values RC, consistent with an unrounded slider |
| 04 | 68% and 32%; 6.25; 6.21; histogram bars; tail ticks; 60; 2.1%; 4.00; 98%, 2.0%, 0.0198, 101.01 | FS + RC; FS + RC; FS + RP (6.2076); measured on a frame against RP counts, within a pixel; RP; FS + RP; FS + RC (0.68¹⁰); FS + RC |
| 05 | 40% and 10%; 807 and 193; 50.0% heads; 16% and 4% with their products, and the mirror; 6,306, 1,615, 1,681, 6,442; 15,890, 7,847, 8,043; the 26 discs and the 15 dropped ones | FS + RC; FS + RP (193 by subtraction); FS + RP; FS + RC; FS + RP; FS + RP; RP |
| 05 | 51 in 100; 350,757; 0.508; years 2007, 2025, 1968, 1984 | FS only (cited rows) |
| 06 | three products and 12.8%; rows 1 2 1, 1 3 3 1, 1 4 6 4 1; the 4, 6 and 4 patterns in the boxes; bars at 56% and 80% | FS + RC; FS + RC; read at full size; RC and FS. Bars at 50%: R4. |
| 07 | six patterns, their order and faces; 2.56%; first eight blocks giving faces 4 and 3; 949; 166, 161, 136, 160, 167, 159; band 136 to 181; 41.0, 41.0, 15.4, 2.6, 0.2; 26.0; 25.3 | FS; FS + RC; RP; FS + RP; FS + RP; FS + RC (135.2 to 181.1); FS + RC; FS + RC (26.04); FS + RC |
| 08 | 5, 5, 10, 10, 20, 50; 318 … 2,976; 10% twice; 1,021, 1,020, 959; stretch widths and axis order | FS; FS + RP; RC; FS + RP; measured on a frame |
| 09 | 1/10 × 1/5 × 1/2 = 1/100; 119 … 103 and 650; band 90 to 127; 24; "33 in 100, about 9; 10 in 100, about 39"; brick labels against positions | FS + RC; FS + RP; FS + RC (89.3 to 127.3); RC; FS + RC (32.55%, 9.22, 10.38%, 38.5); reasoned on one frame, consistent |
| 10 | quote and source line; 1951, p. 36, the 1949 dates; one in four; 77 heads, 15 fair tosses, every disc of the strip; 16 per 100; 72.2 | word for word against the fact sheet's passage; FS; FS + RC; FS + RP; FS + RC; FS + RC (h(0.8) = 0.72193) |
| 10 | Elias 16, 28, 37, 49, 58, 65; Peres 16, 29, 39, 47, 57, 64; dots 16.1, 28.7, 38.8, 69.6, 49.4 | FS + RC from class sizes; FS + RC by Peres's recursion; FS + RP with my own Peres, and Elias in the oracle's rank order |
| 11 | 32 readings giving 1 1 0 0 0 1 1 0; 16% and 10%; seeds; years on the receipts; repository address; 1999 and the quoted fragment | paired by hand (illustrative, no key); FS + RC; FS, and I used them; FS; matches the checkout's remote, not fetched; FS only |

Checked with nothing to report: the description's 8/13 and 5/13, 64.7% and 53.9% under a deadline, 12.5 tosses for the cheaper die, 18 and 39 for the half-procedures, 32.6%, 9.2 and 10.4% for the dice (all recomputed). Run 4's far tail is heavier than usual (three games of 56 or more tosses where 0.3 are expected), but the replay confirms it and the line says "of our ten thousand", so nothing changes.

## What I checked and did not

I opened all 22 sheets, read the script, storyboard, description, brief, fact sheet, R1 dispositions and the audit list, and grabbed 164 further frames at full draft size. I recomputed the model numbers with exact fractions and replayed runs 1, 2, 4, 5, 6, 7, 8 and 9 in my own Python (`python3 -I`). I walked the mathematician audit line by line; its failures are R1 to R3, R6 and R9.

Unchecked:
- Every cited source itself (von Neumann and Forsythe, Gelman and Nolan, Diaconis and others, Bartoš, Jun and Kocher, Peres's Remark 3). I compared screen and script with the fact sheet's passages only.
- "Many computers get them from electrical noise" (11_1); I found no fact-sheet row for "many".
- Blum's 15, 15, 35, 35 in the description.
- Dot and cell counts in the 02_8 square and the 01_3 and 07_3 grids.
- Bar heights at 07_3, 08_1, 09_5 and 10_4 beyond the spot measurements above.
- The colour distances.
- The drafts in motion, and sound. I saw frames only.

The four queued items were not re-reviewed.

Everything I wrote is in `~/jmev_orch/fair_coin/r2_scratch/ruth/`: `recompute.py`, `recompute2.py`, `recompute3.py`, `grab.sh` and the frame sheets. Nothing in the repository was touched.
