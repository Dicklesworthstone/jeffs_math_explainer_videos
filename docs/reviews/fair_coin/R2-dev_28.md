# R2 review: dev_28 (verbatim; paths made relative)

# R2 review: dev_28 (Dev, 28)

## Comprehension check

1. **01 call it.** One toss can't settle a bet with a coin that lands heads four times in five, because whoever holds heads wins about 79% and neither player can verify what the other knows about the coin.
2. **02 toss twice.** Toss twice: heads-then-tails wins for Alice, tails-then-heads for Bob, a match means two fresh tosses. On the 5×5 grid each deciding block is 4 squares of 25, and a seeded thousand games came out 494 to 506 from a coin that came out 791 to 209.
3. **03 every coin.** The two deciding blocks are one rectangle mirrored across the diagonal, p(1−p) against (1−p)p, so they are equal for every p and nobody has to measure p. The only two assumptions are the same chance on both tosses and no influence between them.
4. **04 the price.** It is rejection sampling: a pair decides 32% of the time, so a game averages 2 ÷ 0.32 = 6.25 tosses (4 at best, about 101 at 99% heads). The tail is unbounded, and no rule with a fixed length can be fair for every coin.
5. **05 where it breaks.** Break either assumption and the mirror breaks: a fair second coin gives 40% against 10%, and a coin that repeats its last result gives 16% against 4% after two heads while still counting 50% heads. For one-step memory the fix is to feed the trick only the tosses that directly follow a head.
6. **06 same counts.** For independent same-chance tosses a pattern's chance depends only on how many heads it has, so patterns group by count into Pascal's rows. The unfairness decides which group you land in and never the order inside it.
7. **07 a die from a coin.** The middle group of four tosses has six equally likely patterns, so map them to faces and reject any block without exactly two heads. That is a fair die from an unfair coin at about 26 tosses a roll.
8. **08 loaded dice.** Same trick on a loaded die: roll twice, re-roll a match, first-bigger wins for Alice and second-bigger for Bob, because every block above the diagonal has an equal twin below it. The coin is the two-face case.
9. **09 six orders.** Three rolls that all differ arrive in one of six orders, and the six bricks are one 1/10 × 1/5 × 1/2 brick turned six ways. So the order of ranks is a fair six-way choice at about 9 rolls each, or 24 orders from four rolls at about 39.
10. **10 what is left.** Von Neumann's trick gets 16 fair tosses per 100 from this coin (25 at best), and the ceiling for any exactly fair rule is the entropy, 72.2 per 100. Bigger blocks (Elias) or recursing on the two things the trick ignored (Peres) climb toward it, paying in memory and waiting.
11. **11 the fair coin inside.** Hardware noise bits have the same problem, and Intel's 1999 generator used a corrector based on this pairing. The fairness is in the order once you set the counts aside.

## Rubric

| chapter | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 4 | 4 | 4 | 4 | 3 |
| 02 | 5 | 4 | 5 | 4 | 5 |
| 03 | 5 | 5 | 4 | 4 | 5 |
| 04 | 4 | 5 | 4 | 3 | 3 |
| 05 | 3 | 4 | 5 | 2 | 4 |
| 06 | 4 | 4 | 4 | 4 | 4 |
| 07 | 5 | 4 | 4 | 4 | 4 |
| 08 | 5 | 5 | 4 | 4 | 5 |
| 09 | 3 | 3 | 4 | 4 | 3 |
| 10 | 4 | 5 | 4 | 3 | 4 |
| 11 | 5 | 4 | 4 | 4 | 4 |

Correctness here means the picture agrees with the line and with the numbers I recomputed. Sources are unchecked.

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| D1 | dev_28 | 09_4 to 09_6, cards under the bars | major | The six order cards keep the example's digits ("4 5 6" … "6 5 4") when they become the labels of the counted bars. The bar over "4 5 6" reads 119, but that is every low-middle-high triple. My replay of run 8 gives the same 650 and the same six counts, and only 126 of the 650 are a four, a five and a six. Read literally, the screen says the 650 came from 4-5-6 trios, which the lower-third's "33 times in 100" contradicts (4-5-6 in some order is 6 in 100). It also hides what I need to code it: the face comes from the order of ranks, for any three different rolls. | From 09_4's "Low, high, middle" on, drop the digits and keep the three little bars, or write low/middle/high. Caption the bars "any three different rolls, by the order they came in". |
| D2 | dev_28 | 05_4 | major | The words and the title "two tosses that each follow a head" are right, and the drop-down row is drawn correctly. But the lower row is never paired and nothing marks a skipped toss. After a chapter of thinking in pairs, the version I would write first is "wait for a head, take the next two tosses". On the video's own run 6 that gives 6,394 against 1,657, the same break 05_3 just showed (6,306 against 1,615). The per-toss filter then pairing gives the draft's 7,847 against 8,043. | Bracket the lower row into pairs with the green and violet underlines of 02_8's eight games. Grey out one skipped disc (the sixth toss, a tail after a tail) with "not used: it follows a tail". If a slot is free: "for programmers: filter toss by toss, then pair what is left". |
| D3 | dev_28 | 02_8, 07_3, 09_5 (and 04_3) | minor | The live counters are tweens toward the final values, not tallies of what has landed. At 07_3 (26.0 s) all six read 1 while the strip above shows exactly one four and one three, and they then rise in lockstep (19 18 15 18 19 18, each 11.3% of its final). At 02_8 the drafts show 270–277 and 360–369; the run's true tallies at those game counts are 274–273 and 363–366, so the on-screen race has Bob ahead where Alice was. 01_3 is a true tally (212–58 after 270 tosses). 04_3 briefly reads "2 tosses ÷ 0.32 = 4.08". | Drive these counters from the run's prefix as 01_3 does, or show bars without numerals until they stop. Snap 04_3's result in rather than counting up. |
| D4 | dev_28 | 02_7 and 04_4 lower-thirds | minor | The two for-programmers asides that prevent real bugs are up for 4.6 s (61.0–65.7 s, 20 words) and 5.3 s (26.9–32.2 s, 24 words) at scratch timing, while a legend builds or a histogram grows. Most other lower-thirds get 8 s or more. | Hold 02_7's through the first eight games (to about 70 s, before the "1,000 games" caption needs the strip). Bring 04_4's in at 04_3's "Exactly, six and a quarter", since 04_5's slider needs the same strip. |
| D5 | dev_28 | 02_8 | minor | Nothing says a dot is a pair. The caption says 1,000 games, the square holds 3,128 dots and two thirds sit in the grey. | Caption: "1,000 games, 3,128 pairs: 2,128 thrown away" (`E-S2.trick.pairs`, `.thrown_away`; my replay agrees). It also pre-pays 04_1's 68%. |
| D6 | dev_28 | 07_3, 09_5 | minor | "a fair die: 136 to 181 (model)" does not say what the band is, and face three (136) sits exactly on its lower edge while the voice says "about as often as a fair die's would". 02_8's caption does say "about 19 times in 20". By my simulation a fair die puts at least one of six faces outside such a band in about 23% of runs. | Label both bands the way 02_8 does: "each face: 136 to 181, about 19 times in 20 (model)". |
| D7 | dev_28 | 05_3 | minor | Between 46 and 53 s the screen holds a two-line legend, a title, two readouts with products, the model tag, the toss row, three caption lines and a three-line real-coins lower-third of about forty words. Meanwhile the square flips to "after two tails" and the voice is in the script's longest line (23.5 s). 05_4 starts 0.6 s after it ends. | Cut the lower-third to its first clause and source (the description already has the rest), or bring it in after the flip settles and add a 3 s hold before 05_4. |
| D8 | dev_28 | 04_6 | minor | The marks under the histogram's axis have no legend, and the encoding changes at 18 tosses from bar height to marks that merge into solid blocks. The model's "2.1% still going" has no counted partner. | Label the row "one mark per game: 446 games of 18 tosses or more" (`E-S4.price.pairs_9_plus`). Beside the 2.1%, add "counted: 195 of 10,000" (my replay of run 4). |
| D9 | dev_28 | 03_4 | minor | The 99% stop lasts about 0.6 s (37.6–38.2 s). "1.0% = 1.0%" over two hairlines is the frame that kills "it only works for nearly fair coins", and I could not read it at speed. The 20% stop gets over 2 s. | Park 1.5 to 2 s at 99%, taking the time from the 20% stop. |
| D10 | dev_28 | 10_2 | minor | The fifteen fair tosses pulled from the strip are 10 heads and 5 tails (my replay agrees). It is honest data, but it is the chapter's only picture of fair output and it looks lopsided. This is separate from queued item 3, which is about the count. | Add to the dim caption: "the whole run of 100,000: 16,114 fair tosses, 8,047 heads" (`E-S9.squeeze.von_neumann.*`, replayed). |
| D11 | dev_28 | 11_2 | minor | The corrector pairs raw readings with no reminder that this needs readings that don't affect each other, which is chapter 05's break and the first six words of the 10_1 quote. That is the one thing I would get wrong in practice. Separately, fact-sheet finding 6 says Intel's corrector keeps the second reading of a differing pair, while the picture under the Intel card keeps the first (10 → 1). I have not checked the Intel paper. | Small tag under the corrector: "needs readings that don't affect each other (chapter 5)". Either flip the picture's convention or leave the card clearly as a quote about the idea. |
| D12 | dev_28 | 06_7 | minor | At the 50% start the bars read 6%, 25%, 37%, 25%, 6%, which sums to 99. Six sixteenths is 37.5%. | Show one decimal at this stop, or round half up. |
| D13 | dev_28 | 10_4, right panel | minor | Bars for rounds 1, 2, 3, 4, 6, 8 are equally spaced, so the curve's bend is partly the axis. "all" has a counted dot (69.6) with no bar or model value. | Space by round or put a break mark between 4 and 6. Label the last column "counted only". |
| D14 | dev_28 | description draft, receipts card (11_3) | minor | "Seeds 195101 to 195109" with "heads when the draw is below 0.8" replays runs 1, 2, 4, 7 and 9. For runs 5, 6 and 8 I had to guess or read the fact sheet. Run 6 matches only with "heads when the draw is below 0.8 after a head, 0.2 after a tail" and the first toss counted as following a head; pairing one step later gives 15,981 games, not 15,890. | Add three recipe lines to the description: the fair second coin (below 0.5), the die (first face whose running total of twentieths exceeds the draw), and the memory coin as above. |
| D15 | dev_28 | 02_2 | minor | The screen is empty from 6.9 to 8.8 s, under "A pair can be four things." | Keep 02_1's bracketed pair up until the first tile lands. |
| D16 | dev_28 | 09_2 | minor | The six order labels float around the cube with no leader to their bricks ("6 5 4" sits outside the left wall). At 540p I cannot tell which label is which brick. | A thin leader from each label to its brick, or place each label on its brick's visible face. |
| D17 | dev_28 | 03_7, then 05_2 and 05_3 | delight | The two preconditions are pinned to the geometry like assertions: crossing point on the diagonal for "same chance", a straight cut for "no influence". Chapter 05 then fails each one in the same picture. "One change against two changes" is the mechanism, not a slogan. | Keep. |
| D18 | dev_28 | 07_2, 07_3, 11_2 | delight | The pattern-to-face mapping is on screen and the data obeys it: in the first eight blocks THHT becomes a four and HTTH a three, which my replay reproduces. At 11_2 I paired the 32 readings by hand and got the screen's 1 1 0 0 0 1 1 0. | Keep. |
| D19 | dev_28 | receipts card; 02_7 aside | delight | From the card's seeds and the one-line recipe I reproduced every counted number I tried (list below). 02_7's aside also checks out: a sliding window that reuses a thrown-away toss gave Alice 79.97% in my simulation. | Keep; see D14 for the three missing recipe lines. |

**What I checked, and how.** I read the persona prompt, packet, script, storyboard, fact sheet, description, brief and R1, and opened all 22 sheets. I also pulled about 150 further frames at native 960×540 into `~/jmev_orch/fair_coin/r2_scratch/dev/`.

- **Model numbers:** I recomputed every model number I cite in my own script (`dev_check_r2.py`, same folder), and all match the screen. That covers the square, 6.25, 2.1%, the 469–531, 136–181 and 90–127 bands, 12.8%, the four-toss row, 26.0, the die's 33 and 10 in 100 with 9 and 39 rolls, 72.2, and both ladders.
- **Seeded counts:** using NumPy PCG64DXSM I reproduced 791/209; 494/506 with 3,128 pairs and the eight drawn games; 6.21 and 60 with the histogram; 807; 949 with all six faces; the die's six counts; 1,021/1,020/959; 650 with all six orders; 77 and 15; 16,114/8,047; Peres 28,734, 38,754 and 69,624; and run 6's 6,306/1,615, 1,681/6,442 and 15,890.
- **Description claims:** 64.7% under a deadline, the 18 and 39 plateaus and the 12.5-toss die all hold.
- **Timings:** lower-third durations and the empty screen come from a luma scan of the drafts.
- **Nothing to report** on the pictures and numbers of chapters 03 (beyond D9) and 08.

**Not checked.**

- Sound (the drafts are silent), 4K legibility and colour distances.
- Any source text: no web access, so every citation and real-world claim is unchecked, including Intel, Gelman and Nolan, Diaconis and Bartoš.
- Blum's 15/15/35/35, the fixed-length-rule enumeration, and Elias's seeded dot (49.4; I recomputed only its model rate).
- The repository's code and tests, which I did not run.
- Motion between the frames I sampled.

**One slip to report.** One helper call picked up another reviewer's same-named script in the shared session scratchpad and created an empty folder in `r2_scratch/maya/`. I removed that empty folder and moved my helper into my own folder. Nothing in the repository was touched.
