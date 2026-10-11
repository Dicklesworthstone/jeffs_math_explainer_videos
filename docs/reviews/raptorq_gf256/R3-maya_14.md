# R3 review: raptorq_gf256 narrated draft, by Maya (14)

Reviewer: maya_14 (`tools/review_prompts/maya_14.md`). Stage: R3, narrated 540p draft, "The Arithmetic Where 3 x 7 = 9".

## What I checked and how

- I read the whole script and the description in the R3 packet.
- I opened all 17 cue sheets and read each one next to its legend and its script lines. Each tile is the frame 1.5 s into its
  line (`cue_sheets.py`, default `--at 1.5`). Tiles after the last legend entry are padding that repeats the last frame. So I
  judged what is on screen as each line starts. Anything that only appears later in a line is **unchecked**.
- I worked out these numbers myself, with a throwaway `python3 -c` (it wrote no files): 5 ex-or 7 = 2 and 2 ex-or 7 = 5;
  2 ex-or 6 = 4 and 6 ex-or 4 = 2; the chapter-3 pieces A=5, B=3, C=7, D=2 (with E=12 they match every card in 14, 16 and
  the ring); 7/8 x 6/8 x 4/8 = 21/64 = 0.33; 2 x 131 = 262, which wraps to 6; 5 x 3 = 15 = 1 on the 7-clock; (x+1)(x^2+x+1)
  = x^3+1 = 9; the rotation gives 3 x 255 = 0; 2 x 128 = 29 and 2 x 142 = 1 under RaptorQ's fold; 128 x 4 = 58; 150 is 180
  clicks, 3 is 25 clicks, and 205 clicks is 167; 87 x 131 = 49 (RaptorQ) and 193 (AES); the ring gives 3A = 15, so A = 5,
  and c = 7; 10,000 x H(10,000) = 97,876, about 98,000; 50,000^3/3 = 4.2 x 10^13 steps, 11.6 h; 6,316/256 = 24.7.
- **Unchecked** (taken as given): 71 / 42 / 23 percent, "twenty spares", 1 in 256, peeling stopping at 98% and the 70
  pieces in no card, 10,269 slots, the decoder counts 6,316 / 24 / 1, the "two or three loops".

## Comprehension check (what I'd tell a friend)

- **01 A million phones:** Sending one picture to a million phones that each drop 1 packet in 100 is hard. Asking again
  swamps the sender, looping makes you wait, and random pieces take about 98,000 tries to collect 10,000. A weird
  arithmetic where 3 x 7 = 9 somehow lets a phone finish with just two spare packets.
- **02 Bits and ex-or:** Numbers are bits, and ex-or flips the first number's bits wherever the second has a 1 (5 ex-or 7
  = 2). Doing it twice undoes it, and that's how we'll mix pieces.
- **03 Packets are equations:** A packet is an equation like "A ex-or C = 2", and the phone solves them like elimination
  in algebra. It needs as many useful cards as there are pieces, and a card you could build from the others is useless.
- **04 The cube:** Every possible card is a corner of a cube, and each useful card doubles the corners you can make. With
  two of three cards in hand, a random new card is useless half the time.
- **05 The ex-or lottery:** If you catch exactly enough random ex-or cards, it fails 71% of the time. Almost all the risk is
  in the last few slots, so it's 71% for ten pieces or ten million.
- **06 Spares and rings:** Each spare card only about halves the failure, so 1 in a million takes 20 spares. Some
  patterns, like the ring, are stuck anyway. So we want cards that multiply pieces by any byte, and that needs a
  multiplication you can undo.
- **07 Why not wrap around at 256:** On a 256-clock, 2 x 128 = 0, so you can't divide by even numbers, and random amounts
  fail 71% just like ex-or. (I can repeat the 71%, but I can't say *why* "only the last bit matters". See maya-03.)
- **08 Prime clocks:** On a clock with a prime number of hours, every number except 0 has a partner that multiplies it
  back to 1, so you can divide. 256 isn't prime and 257 doesn't fit in a byte, so we need some other "field" with 256
  members.
- **09 Bytes are polynomials:** Write bytes as polynomials where 1 + 1 = 0, and 3 x 7 = (x+1)(x^2+x+1) = x^3 + 1, which is
  1001 = 9.
- **10 The fold:** When a product overflows, you fold the fallen x^8 back in. Folding it as a rotation breaks the same way
  the clock did (3 x 255 = 0). RaptorQ folds with a polynomial that can't be factored, so every byte gets a partner
  (2 x 142 = 1) and bytes become a field. (Honest note: until 12_5 I thought the polynomial that can't be factored was
  x^4+x^3+x^2+1. See maya-02.)
- **11 The byte clock:** Doubling over and over visits all 255 nonzero bytes. So each byte is a number of clicks on a
  255-hour clock, and multiplying means adding clicks (128 x 4: 7 + 2 = 9 clicks = 58).
- **12 The times table:** Sorted by clicks, the noisy times table turns into stripes. The standard multiplies with two
  lookups, and both ends must use the same polynomial or they get different answers.
- **13 The byte lottery:** If cards use random byte amounts, the last card is useless only 1 time in 256, not 1 in 2, so
  byte cards almost never need a spare.
- **14 The price of density:** Cards that mix every piece take half a day to solve. Sparse cards solve like Sudoku, but
  they get stuck, and some pieces aren't in any card at all.
- **15 Where the danger lives:** RaptorQ sends cheap ex-or packets, and both sides also know some free cards. The 10 to 16
  byte cards among them land exactly on the last risky slots, which takes failure from 71% to about 6 in 1000. (I could
  say *that*, but not *how* the extra pieces and free cards work. See maya-01.)
- **16 Parking a piece:** When peeling gets stuck, the decoder treats one piece as an unknown, keeps going, and solves for
  it at the end. A byte card unsticks the ring, because A and 2A don't cancel.
- **17 Two packets:** A real decoder failed 6 times in 1000 with exactly enough packets, 24 in a million with one extra
  and 1 in a million with two, since each extra divides the risk by about 256. That's the opening promise, and it all
  comes from 3 x 7 = 9.

No chapter left me with a wrong or empty sentence, so I have no blockers. Chapter 10 only came right once 12_5's screen
corrected me. In chapter 15 I could say what happens but not how.

## Rubric (1-5)

| ch | clarity | correctness | motivation | pacing | delight | note |
|---|---|---|---|---|---|---|
| 01 | 4 | 5 | 5 | 4 | 4 | strong hook; 01_8 counter lags the voice |
| 02 | 5 | 5 | 3 | 4 | 3 | clean; nobody says a piece is a number |
| 03 | 4 | 5 | 4 | 4 | 4 | "deck" muddles "cards" |
| 04 | 4 | 5 | 4 | 4 | 4 | tally shows 3 of 8 next to "50%" |
| 05 | 4 | 5 | 4 | 4 | 3 | the danger meter works; labels tiny |
| 06 | 4 | 4 | 5 | 3 | 3 | 06_5 is a 23 s line; 06_3 label garbled while it moves |
| 07 | 2 | 4 | 4 | 3 | 3 | the "last bit" leap; determinant box |
| 08 | 4 | 5 | 4 | 4 | 4 | "257 doesn't fit in a byte" is clever |
| 09 | 4 | 5 | 3 | 4 | 5 | I did the FOIL myself |
| 10 | 2 | 4 | 4 | 2 | 3 | which polynomial is prime?; 10_3 too dense |
| 11 | 4 | 5 | 4 | 4 | 5 | 29 predict, star untangles |
| 12 | 4 | 5 | 3 | 4 | 5 | stripes reveal; "orders" box |
| 13 | 3 | 4 | 4 | 3 | 3 | 1-in-256 reason too abstract |
| 14 | 4 | 4 | 5 | 4 | 4 | Sudoku and "Nothing moves" work; 50,000 vs 10,000 |
| 15 | 2 | 4 | 4 | 2 | 2 | extra pieces / free cards lose me; blocks interrupt |
| 16 | 4 | 5 | 4 | 3 | 5 | callbacks to ch 3 cards and the ring |
| 17 | 4 | 4 | 5 | 4 | 4 | 6,316 / 256 is a predict I can do; stats boxes |

Correctness here means it sounded right to me and the small numbers I worked out matched (see the list above). It is
not Ruth's check.

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| maya-01 | maya_14 | 15_2, 15_6, 15_3 | major | I lost it at 15_2. "The sender adds some extra pieces, picked so that certain fixed cards all have the result zero." Extra pieces of what? Why zero? The picture is a 27-column grid labelled "10 pieces / 17 extra pieces / 7 free ex-or cards / 10 free byte cards / 10 packets", so there are more made-up pieces than real ones, and nobody says why. Then 15_6 brings in "blocks" before I've taken in free cards. Then 15_3's bucket says "the big block: 10,269 slots", but I thought our picture was 10,000 pieces. I can repeat the ending (71% down to 6 in 1000), but not how it works. | Before 15_2's general sentence, give a three-piece version I can check with ex-or: "Take A, B and C, and add an extra piece P = A ex-or B ex-or C. Now the card A ex-or B ex-or C ex-or P is always zero. Both sides know it, so it's never sent. That's a free card." Draw that card in chapter 3's card style, then grow it into the grid. Move 15_6 (blocks) after 15_5 or into the description. Label the big bucket "our picture: 10,269 slots (pieces plus extra pieces)". |
| maya-02 | maya_14 | 10_4, 10_5 (echoed in 17_7) | major | "We need a polynomial that can't be factored... RaptorQ swaps in x to the fourth plus x cubed plus x squared plus one." The only polynomial I hear and see whole is x^4+x^3+x^2+1, so I took that to be the prime polynomial. It isn't. It's what goes in place of x^8, and on its own it *does* factor: put in x = 1 and you get 1+1+1+1 = 0, so x+1 divides it (it's (x+1)(x^3+x+1)). I only saw the real one, x^8+x^4+x^3+x^2+1, on 12_5's screen two chapters later. So when 17_7 says "a polynomial refuses to factor", my head points at the wrong one. A sharp viewer will post this in the comments. | Say it the same way 10_3 says "x to the eighth plus one is zero": "RaptorQ's prime polynomial is x to the eighth plus x to the fourth plus x cubed plus x squared plus one. Treat it as zero, so whenever an x to the eighth falls off, swap in x to the fourth plus x cubed plus x squared plus one." Keep "x^8 + x^4 + x^3 + x^2 + 1: can't be factored" on screen through 10_6, above the existing "x^8 -> ..." line. |
| maya-03 | maya_14 | 07_5, 07_6 | major | "Even amounts lose information... Odd ones never do. So with random byte amounts, only the last bit matters." Nobody tells me the last bit is what makes a number odd, so "only the last bit matters" comes from nowhere, and "that's the coin flip from ex-or again" is a jump I can't make. The two square grids in 07_6 ("ex-or cards", "wrap-around cards") are never explained, so I just have to believe the 71%. | Add one sentence before the jump: "A number is odd exactly when its last bit is one." Then animate the purple wrap-around grid so each amount keeps only its last bit (odd becomes 1, even becomes 0), until it looks just like the cyan ex-or grid. Say: "Keep only the last bits, and you have an ex-or card. The wrap-around cards can be solved exactly when that ex-or card can." |
| maya-04 | maya_14 | 07_6, 12_4, 17_2, 17_3, 17_4 | major | Small "precisely:" and "measured:" boxes sit at the bottom while I'm meant to watch the main picture, and they use words I don't know. 07_6: "can be undone exactly when its determinant is odd". 12_4: "orders 1, 3, 5, 15, 17, 51, 85, 255 occur 1, 2, 4, ... times". 17_2 and 17_3: "raptorq 2.0.0 (Rust); 95% interval 0.616-0.647%". 17_4: "95% interval: 0.025 to 5.6 per million" and "÷256 twice predicts ≈ 0.1 failures here". 07_6 is the worst, because I'm already shaky on the last bit, and a box about determinants is exactly where I feel this video isn't for me. | Move these to the description's "Precisely" list, where they already are, or to one "for the curious" card at the very end. If one has to stay, write it in the video's own words. 07_6: "precisely: wrap-around cards can be solved exactly when their last bits alone can". 17_4: "two extra: too rare to measure exactly". No "determinant", "orders" or "interval" on screen. |
| maya-05 | maya_14 | 13_2, 13_3, 13_4 | minor | 13_2 says "Half the corners were lit", but the screen shows a bucket, not the cube. At 13_3 the screen already says "the 2 values of one ex-or amount: 0 useful, 1 useless" before anyone says what's being held fixed, so it reads as "using a piece makes a card useless". Then 13_4's reason ("Fix every amount but one... reshuffles all two fifty-six values... lands in the lit part") is too abstract for me. I just take 1 in 256 on trust. | Bring back 04_5's half-lit cube for 13_2. For 13_4, use two pieces I can check: "You hold the card A ex-or B. A new card is 9A ex-or kB. Only k = 9 makes it nine times the card you already hold, so 1 choice in 256 is useless. With ex-or, A ex-or kB is useless only for k = 1: 1 choice in 2." Show the 0-and-1 pair after that sentence, not before. |
| maya-06 | maya_14 | 10_3 | minor | Four steps in one breath: the rotation makes x^8 into one, so x^8 + 1 is zero, it factors into x + 1 times "a long polynomial", and that's 3 times 255. I got "same problem as 2 x 128", but not why x^8 + 1 matters. | Write the chain on screen, one line per clause: "x^8 + 1 -> 0", "x^8 + 1 = (x + 1)(x^7 + x^6 + ... + 1) = 3 x 255", "so 3 x 255 = 0, just like 2 x 128 = 0". Say what the long polynomial is: "x to the seventh plus ... plus one, all ones, two fifty-five." |
| maya-07 | maya_14 | 03_8 (against 03_1) | minor | "Both sides own the same shuffled deck." The cards are the packets, so I pictured the deck as the pile of packets. Also, 03_1 says each packet says which pieces it mixed, but 03_8 says the packet's *number* tells. While the deck is being described, the screen shows the fountain. | Use a word that isn't "card": "Both sides have the same recipe book, so a packet only carries its number, and the phone looks up which pieces it mixed." Show "packet #37 -> A ex-or C". |
| maya-08 | maya_14 | 04_5 | minor | Next to "4 lit of 8: useless 50% of the time", the tally shows 3 useless dots and 5 useful ones. I'd ask: so is it a half, or three eighths? | Throw enough random cards that the tally sits near half (say 40 throws, about 20 and 20). Or label the dots "8 random tries" and say "in the long run, half". |
| maya-09 | maya_14 | 06_3 | minor | While the marker slides down the scale, its label is letters drawn on top of letters ("20 spares... 1 in 1,000..."). I can't read it. | Hide the label while the marker moves. Write "20 spares: about 1 in a million" once it stops. |
| maya-10 | maya_14 | 14_1 | minor | Our picture was 10,000 pieces, and suddenly it's "a file of fifty thousand pieces". I wondered if I'd missed a change. | Say "a bigger file, like a short video of fifty thousand pieces". Or start from our picture: "Even our picture takes about five minutes per phone. Fifty thousand pieces takes half a day." (10,000^3/3 at a billion a second is 5.6 min; I worked it out.) |
| maya-11 | maya_14 | 16_5 | minor | "Three A is fifteen, and A is five" sounds like ordinary dividing, and "one ex-or two is three" also matches ordinary 1 + 2. Right after 3 x 7 = 9, I'd wonder whether I'm allowed to divide the normal way now. At 1.5 s the frame shows "14 ⊕ 5 ⊕ 4 ⊕ 6 ⊕ A ⊕ 2A = 6", with numbers on both sides, and I'd stare at it. | Add "and here three times five really is fifteen, because nothing cancels", so I know it's checked, not ordinary division. Show it in one clean line: "A ⊕ 2A = 6 ⊕ 4 ⊕ 5 ⊕ 14 ⊕ 6 = 15". |
| maya-12 | maya_14 | 06_5, 08_3, 10_5, 12_5, 04_1 | minor | Symbols I don't know on screen: "amount ∈ {0, 1}" (06_5), "2 x 128 ≡ 0 (mod 256)" (08_3), "0x1D" (10_5), "0x57 · 0x83 = 0x31" (12_5), and "∅" (04_1, though the voice says "nothing", so I figured that one out). | Write "amount: 0 or 1" and "2 x 128 -> 0 on the 256 clock". Keep hex inside the programmers' panel only. "∅" can stay if it's pointed at as the voice says "nothing at all". |
| maya-13 | maya_14 | 04_4, 05_3, 05_4, 06_1, 11_5, 13_2, 15_3 | minor | Labels that carry the meaning are tiny: "danger: chance a new card is useless", "slots already filled", the cube's "lands on a lit corner (useless)", the scale numbers 1/10 ... 1/1,000,000, and 11_5's clicks-to-bytes table, where I'd want to check that 9 clicks is 58. On a phone they'd be about a millimetre tall. | Double the size of labels that carry meaning. Make 11_5's table big, and light up the 7, 2 and 9 columns as they're spoken. |
| maya-14 | maya_14 | 10_4, 10_5, 10_6, 12_0, 12_5 | minor | The "for programmers" code box sits right beside the fold during 10_5 and 10_6, the beats where I most need to watch the bits. In 12_0, "a u8 sum wraps for 32,131 of 65,025 pairs" floats outside the box. (The label "for programmers" is good, because it tells me I can skip it.) | Bring the panel in after 10_6 finishes, and put the u8 caption inside the panel. |
| maya-15 | maya_14 | 01_8, 15_5 | minor | The counters lag the voice. In 01_8 the voice says "about ninety-eight thousand" while the screen says "random pieces drawn 40,390". In 15_5 a big "0.0% fail" sits next to "7,066 of 10,000 failed". I only see 1.5 s into each line, so where they end up is unchecked. | Let 01_8's counter reach about 98,000 before the voice says the number, or as it says it. That also shows "the last few take forever". In 15_5, count the fraction and the percentage up together, so they never disagree. |
| maya-16 | maya_14 | 13_7 | minor | The ring comes back with a faint "as ex-or cards: 4 of 5 useful", but the voice talks about "the other two fifty-four amounts" and never mentions the ring, so I didn't know why it was there. | Say it: "So our ring of five ex-or cards is still stuck, even in the byte field." Make the caption readable. |
| maya-17 | maya_14 | 14_2, 14_3 | minor | The card boxes from 14_2 turn into dots and lines in 14_3, and nobody says what the dots and lines are. | Add one phrase: "Each dot is a card, and each line goes to a piece it uses." Or morph each card box into its dot. |
| maya-18 | maya_14 | 02_3 | minor | We jump from pieces of a sunset to ex-or on 5 and 7, and nobody says a piece is a number. I had to guess. | Add: "A piece of the picture is just numbers, so mixing pieces means mixing numbers." |
| maya-19 | maya_14 | 11_7 | minor | Why is the partner at the mirror image? I had to work it out. | Add: "because their clicks add up to one full turn, which lands back on one." |
| maya-20 | maya_14 | 09_4, 09_5, 09_6 | delight | "Multiply them out... What do you get?" I could FOIL it myself, got x^3 + 1 = 9 before the voice did, and felt like I'd broken the title's riddle. Best moment of the video. | Keep it. Keep the pause long enough to do the multiplication (it's about 15 s now, which is fine). |
| maya-21 | maya_14 | 11_1, 11_2 | delight | "One twenty-eight. What comes next?" I remembered 29 from 10_5 and felt clever. Then the star untangling into a circle (11_3) looks great. | Keep it. |
| maya-22 | maya_14 | 16_2, 16_5 | delight | "Three of these are our cards from the start." A comes out as 5 again, the same A as in chapter 3, and the ring from chapter 6 finally gets solved. It feels like the video kept its promises. | Keep it. Light up the matching chapter-3 cards as 16_2 names them. |
| maya-23 | maya_14 | 12_2, 12_4 | delight | Noise to perfect diagonal stripes. I guessed "some pattern" and it was prettier than I expected. | Keep it. |
| maya-24 | maya_14 | 03_6, 03_7 | delight | "Here's a fifth card. Is it useful?" I spotted that it's cards one and three put together before the answer. Good first predict. | Keep it. |
| maya-25 | maya_14 | 14_4, 14_5, 14_7 | delight | "Lose the card that starts it. What happens?" "Nothing moves." Short and satisfying. And the sunset coming back with red circles on the missing pieces is the best continuity in the video. | Keep it. |
| maya-26 | maya_14 | 17_2, 17_3 | delight | 6,316 divided by 256 is about 25, and the answer was 24. A predict I can do in my head. | Keep it. Hold the pause a beat longer, so there's time to divide. |
| maya-27 | maya_14 | 01_2 | delight | "A kind of arithmetic where three times seven is nine." That's why I'd keep watching. | Keep it. If "3 x 7 = 9" isn't already on screen later in 01_2 (unchecked at 1.5 s), put it there. |
