# R2 review: maya_14 (verbatim; paths made relative)

**Verdict:** no blockers. Five majors, at 02_6, 05_3 (two), 07_3 and 10_4. Chapters 05 and 10 are where I would rewind or quit; 02, 03, 06 and 08 are where I felt clever.

## Comprehension check

- **01:** The coin lands heads four times in five, so "just call it" is fair only if the caller doesn't know which side is lucky, and Alice and Bob can't check that about each other. So they need a way that is fair whatever either of them knows.
- **02:** Toss twice. Heads-then-tails and tails-then-heads each cover 4 of the 25 small squares, so Alice takes one, Bob the other, and matches are tossed again. A thousand games came out 494 to 506 with a coin that is still 791 to 209.
- **03:** The two blocks are one rectangle flipped over the diagonal (4/5 × 1/5 the other way round), so they are equal for every coin. Nobody needs to know the coin, as long as it's the same coin both times and one toss doesn't affect the next.
- **04:** The grey is the price. Only 32 pairs in 100 decide, so a game takes six and a quarter tosses on average, more for a worse coin, and nobody can promise when it ends.
- **05:** If the second toss comes from a different coin, or the coin copies its last result, the blocks stop being mirror images and the game leans. For the copying coin, using only tosses that follow a head fixes it. (I can say what happened with the copying coin; I can't say what harm it does. See M2.)
- **06:** Three tosses make a cube, and the three "two heads, one tail" slabs are one slab turned three ways, so same counts means same chance. The unfairness only decides which group you land in.
- **07:** Four tosses with exactly two heads come in six equally likely patterns, so they pick a die face. That is a fair die from an unfair coin, at about 26 tosses a roll.
- **08:** With a loaded die, roll twice: Alice wins if the first roll is bigger, Bob if the second is. It's the same square cut six ways, every block has an equal twin across the diagonal, and the coin was the two-face case.
- **09:** Three different rolls come in six orders, and the six bricks are one brick turned six ways, so the order is a fair pick among six. That is a fair die from a loaded one, at about nine rolls each.
- **10:** The trick is von Neumann's, from 1949, and it wastes a lot: 16 fair tosses per 100 with our coin. No rule can average more than about 72, and cleverer rules climb toward that without reaching it.
- **11:** Computers run the same trick on noisy readings. Any unfair coin that can land both ways has a fair one inside, found by ignoring how many heads came up and looking only at the order.

## Rubric

"Correctness" is how right it looked and sounded to me, plus the sums I could do in my head. It is not a check of the maths.

| chapter | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 call it | 4 | 4 | 5 | 4 | 3 |
| 02 toss twice | 4 | 5 | 5 | 4 | 5 |
| 03 every coin | 5 | 5 | 5 | 4 | 5 |
| 04 the price | 4 | 4 | 4 | 4 | 3 |
| 05 where it breaks | 3 | 3 | 3 | 2 | 3 |
| 06 same counts | 4 | 4 | 4 | 3 | 5 |
| 07 a die from a coin | 4 | 3 | 5 | 4 | 5 |
| 08 loaded dice | 4 | 5 | 4 | 4 | 5 |
| 09 six orders | 3 | 4 | 3 | 4 | 4 |
| 10 what is left | 2 | 3 | 3 | 2 | 2 |
| 11 the fair coin inside | 4 | 4 | 4 | 4 | 4 |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| M1 | maya_14 | 02_6 | major | From about 52 to 56 s the big block says "16" and the two thin blocks say "4" then "16%". Two sixteens on one square mean different things: 16 small squares (64%) and 16 percent (4 small squares). The voice says "sixteen" for both within fifteen seconds. For a moment I thought the big block was 16%. | Write the counts with their unit ("16 of 25", "4 of 25", "1 of 25") and put the percent in the readout column where chapter 03 keeps it ("4 of 25 = 16%"). Or give all four blocks their percent at once (64, 16, 16, 4). |
| M2 | maya_14 | 05_3 | major | I can't say what is broken. "Four times as likely" sounds unfair, then "after two tails it's the other way round" sounds like it evens out, and the screen says 50.0% heads. The description has the real harm in one line ("the winners come out half and half, but each result predicts the next"). The voice never says it. | Say it after "the other way round", for example "So the wins still come out even, but anyone who saw the last pair knows who is ahead." Ruth should check the wording. Also say why "after two heads": "Say two heads were just thrown away." |
| M3 | maya_14 | 05_3 | major | From about 39 to 53 s I am following the hardest reasoning in the video while the screen holds nine separate things. The three-line "real coins" note (with 350,757 and 0.508 in it) comes on at 39.2 s, the same moment as the stepped square. My eyes went to the note and I missed "one change, two changes". | Hold the note until the pause after 05_3, or cut it to one line ("real tossed coins do this a little: about 51 in 100 land the way they started") with sources in the description. Bring "6,306 against 1,615" on only after "four times as likely". |
| M4 | maya_14 | 10_4 | major | "The two things it ignored": I can't name them and the voice doesn't. The labels that do ("which pairs matched", "what the matching pairs showed") are the smallest text on the busiest frame of the video: two charts, 13 bars, five dots with decimals, a triangle, a square with two trees, a caption and a lower-third, inside one 15 s sentence. This is where I felt stupid. | Say them: "It ignored two things: whether each pair matched, and what the matching pairs were. Each is a new unfair coin, so run the trick on those too." Finish and dim the left chart before the right one starts. Drop the dots or give every bar one. Reword "the price: memory and waiting" ("memory" meant the coin's memory in chapter 05). |
| M5 | maya_14 | 07_3 | major | The bar for three (136) is plainly the shortest and stops exactly on the bottom edge of the grey band (136 to 181). It looks as if it missed, while the voice says each face came up "about as often as a fair die's would". The label "a fair die: 136 to 181 (model)" doesn't say what the band is; 02_8's caption does. | Label the band as 02_8 does ("a fair die's counts land in here about 19 times in 20") and name the low bar aloud: "The three came up least, and still within what a fair die does." Same label at 09_5. Whether 136 counts as inside is Ruth's to say. |
| M6 | maya_14 | 01_1 | minor | "Neither trusts what the other knows about it" took me two listens, in the first ten seconds. | "…and each suspects the other knows the coin better." |
| M7 | maya_14 | 01_4 | minor | A question mark over each head under "which side does it favour?" reads as "neither of them knows". The voice has just said a call is fair when the caller has no idea, so the picture says the bet is already fair. | Aim each mark at the other player ("does he know?", "does she know?"), or use one caption: "what does the other one know?" |
| M8 | maya_14 | 01_5 | minor | The third card, "and, far more mildly, the noise inside a computer", is up from about 56.0 to 57.8 s: two lines, never spoken. I couldn't read it. | Bring it in with the die at about 54 s and hold all three until 01_6's card, or shorten it to "computer noise (a little)". |
| M9 | maya_14 | 02_2 | minor | The first pair and its label "a pair" last about a second. Then the screen is empty from about 7.2 to 8.6 s while the voice says "A pair can be four things", and a new HH tile fades in somewhere else. | Let the bracketed pair shrink into the first tile, with no empty frame. |
| M10 | maya_14 | 02_3 | minor | The first cut is a thin dark line on a dark square, and nothing marks fifths until the grid arrives in 02_4. I can't see "four fifths of the width". | Draw the cut bright while it drops, with five faint ticks along the bottom edge. |
| M11 | maya_14 | 02_7, 02_8 | minor | Nobody plays one game slowly in this chapter. The first eight games appear in about three seconds (66 to 69 s) while the voice is already reading totals. | Play game 1 alone under 02_7's last sentence (two heads, two heads, two heads, tails then heads: Bob), then the other seven fast. |
| M12 | maya_14 | 03_7 | minor | The lower-third says "probability p". The video has only said "chance", so I took it for a second thing. The label "enough:" didn't tell me what the note was. | "what it needs, exactly: tosses that don't affect each other, with the same chance p of heads (textbooks say probability)". |
| M13 | maya_14 | 04_3 | minor | The screen says "2 tosses ÷ 0.32 = 6.25", but the voice got there by "one in three, three pairs, six tosses". I didn't see why you divide 2 by 0.32. The answer also counts up from zero, so at 24.2 s it reads "2 tosses ÷ 0.32 = 0.03". | Follow the voice: "100 ÷ 32 = 3.125 pairs, × 2 = 6.25 tosses". Fade the answer in whole. |
| M14 | maya_14 | 04_6 | minor | The lower-third ("no rule that always names a winner within a fixed number of tosses…") is the hardest sentence in the chapter and nobody says it. The row of ticks under the axis has no label. | Plain words, or description only: "why no promise: a rule that must end by toss 10 has to hand ten heads in a row to somebody". Label the ticks "one tick, one game". |
| M15 | maya_14 | 05_4 | minor | "Each of those has the same chance": which chance? The second row is 12 blue out of 15 and nothing says so. | Say and show the number: "After a head it's heads four times in five, every time", with the caption "after a head: heads 4 in 5". |
| M16 | maya_14 | 06_6 | minor | The row for three tosses shows pieces I've never met (three rods, a tiny cube) with no discs or labels. "Pascal's triangle" is only a name, and the whole line is 7 s. | Put the three-disc patterns or "3 heads … 0 heads" on the boxes, as the four-toss row has. When the 6 glows, draw 3 + 3 = 6 from the row above, so the name comes with its rule. |
| M17 | maya_14 | 06_7, 07_4 | minor | At 50% the bars read 6, 25, 37, 25, 6. I added them: 99. The same bars at 80% read 41, 41, 15 here but 41.0, 41.0, 15.4 in 07_4. | Use one rounding in both chapters, to one decimal (6.3, 25.0, 37.5). |
| M18 | maya_14 | 06_7, 07_1 | minor | "Inside a group, every pattern is equally likely" has no picture. Then 07_1 shows "2.56%" six times from nowhere. | At that sentence, split the middle bar (15.4%) into six equal slices of 2.56%, one per strip. |
| M19 | maya_14 | 07_3 | minor | "the first 8 blocks of 4 tosses": "block" has meant a piece of the square since chapter 02. | Say "tries" (07_4 already says "15.4% of tries"). |
| M20 | maya_14 | 07_4, 07_5 | minor | 26 arrives without its sum, though chapter 04 taught me the sum. "There are cheaper rules" promises something and gives nothing. | Show "4 tosses ÷ 0.154 = 26.0". Cut the first sentence of 07_5 or name one cheaper rule. |
| M21 | maya_14 | 08_1 (first at 01_5) | minor | "Loaded" is never tied to "unfair". | "Now swap the coin for an unfair die, a loaded one." |
| M22 | maya_14 | 08_5 | minor | This is the best moment of the act, and the coin's square is held only from about 43.3 to 45.6 s before the grid opens again. Beside it the legend still says "the first roll is bigger" and "10%", next to blocks I know are 16%. | Hold 3 s or more. During the hold, swap the legend to "heads then tails / tails then heads, 16%" or dim the dice rows. |
| M23 | maya_14 | 09_1 | minor | "Three rolls do more." More what? I only learn at 09_4 that the goal is a fair die. | "Three rolls can make a fair die." |
| M24 | maya_14 | 09_4 | minor | The six cards are labelled 4 5 6, 4 6 5 and so on, so I wondered whether I must roll a four, a five and a six. Nobody says the order picks the face; I got it from the gold outline. | Label the cards low / middle / high, or show one more roll (1, 6, 3 lands on the same card as 4, 6, 5). Add "so the order picks a face". |
| M25 | maya_14 | 09_6 | minor | "Each step costs more", but the numbers that show it are only in the lower-third. "4 × 3 × 2 × 1 = 24" has no partner. | Main column: "3 rolls: 3 × 2 × 1 = 6 orders, about 9 rolls each. 4 rolls: 4 × 3 × 2 × 1 = 24 orders, about 39 rolls each." |
| M26 | maya_14 | 10_1, 10_2 | minor | One thing is said three ways in 15 s: "25 percent as efficient", "one fair toss for every four" (on screen about 2.5 s) and "sixteen in a hundred". Nobody says a game is one fair toss, so I didn't see that 16 is chapter 04's six and a quarter again. | Caption "a fair coin: 25 fair tosses per 100". Voice: "Each game is one fair toss. At six and a quarter tosses a game, a hundred tosses give about sixteen." |
| M27 | maya_14 | 10_3 | minor | The new word "entropy" is on screen only at the end of a lower-third marked "precisely:". | Put it on the line itself: "the ceiling (the coin's entropy): 72.2 per 100". |
| M28 | maya_14 | 11_1 | minor | The readings become ones and zeros and nobody says a one is like heads. | "Each reading is a one or a zero, like heads or tails." |
| M29 | maya_14 | 11_3 | minor | The last picture of the video is the receipts card alone. | Keep the square beside it to the last frame, as it is at 45 s. |
| M30 | maya_14 | 02_5, 02_6, 02_9 | delight | I counted four small squares and four before the voice said it. The two bars at the end, lopsided coin over level game, are the whole video in one picture. | Keep. |
| M31 | maya_14 | 03_2, 03_4 | delight | The flip landing exactly, then the two factors trading places. At 20% the green block is tall and the violet one wide, and it still says 16% = 16%. | Keep. |
| M32 | maya_14 | 05_2, 06_4 | delight | The red overhang shows the break without a word. The three slabs turning until they are the same slab at 12.8% made me guess the dice answer in chapter 09 before it came. | Keep. |
| M33 | maya_14 | 07_1, 07_2, 11_1, 11_2 | delight | I said "a die" before the voice did, and "pair them up and throw away the matches" at the chip. Both questions are ones I can answer. | Keep. |

**What I checked and how.** I read the persona prompt, the packet, the script, the storyboard, the brief, R1.md, the description draft, and the fact sheet's first 190 lines plus the table rows for numbers I saw on screen. I opened all 22 packet images. I grabbed 205 more frames at full draft size (52 sheets in `~/jmev_orch/fair_coin/r2_scratch/maya/`) to time what the sheets could not show: 01_5's third card, the empty frame in 02_2, 02_6's two sixteens, the 04_3 count-up, when 05_3's lower-third arrives, the 08_5 hold and the build of 10_4. In my head I did these sums:

- the small squares, 16, 4, 4 and 1 of 25;
- 9/10 × 1/10;
- 2 ÷ 0.32, 2 ÷ 0.5 and 2 ÷ 0.0198;
- 40 against 10;
- 4/5 × 1/5 and 1/5 × 1/5;
- 16/125 = 12.8%;
- 0.8² × 0.2² = 2.56%, six of them, and 4 ÷ 0.154;
- 1/2 × 1/5, and 1/10 × 1/5 × 1/2;
- 100 ÷ 6.25.

I also added the totals (791 + 209, 494 + 506, 807 + 193, 7,847 + 8,043, the six faces to 949, the six orders to 650, 1,021 + 1,020 + 959) and ran 11_2's pair filter by hand on the 32 readings; all came out as shown.

**What I did not check.** These are unchecked, not fine: 72.2 and the chapter 10 bars, the three fair-die bands, 6.21, the longest game of 60, 2.1%, the 200,000-toss counts, the costs in 09_6's lower-third, every source and date, and the rest of the fact sheet. The drafts are silent, so all timing against the voice is by the cue times. Text I called small was judged at 960×540. I wrote nothing in the repository.
