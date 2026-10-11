<!-- R2 script review, 2026-10-11, from the reviewer's final answer (packet tools/review.py; script at 85fbff2) -->

# Maya, 14: R2 script review of *The Arithmetic Where 3 × 7 = 9*

**Summary.** I found no blockers. Every chapter gives me a sentence I could tell a friend. I found seven majors. Each one can be fixed with a few words or one picture change. Three of them are lines that sound wrong to me or contradict something said earlier:

- **M1:** 17_2 says an extra packet divides the risk by 256, but RaptorQ's packets are plain ex-or cards.
- **M2:** 15_3 says the byte cards come "last", but they were agreed before anything was sent.
- **M4:** in 10_3, the reason for the video's final line goes unexplained.

The others:

- **M3:** 06_5 doesn't parse when spoken.
- **M5:** the 13_4 picture swaps to an object nobody explains.
- **M6:** in 16_2–16_3 the numbers have no source I can hear.
- **M7:** 16_4 is a predict I can't make.

## What I checked and how

- **Recomputed by hand, then with a throwaway Python script in my scratchpad (nothing in the repo):**
  - Bits and ex-or: place values; 5 = 101, 7 = 111; 5⊕7 = 2 and 2⊕7 = 5.
  - Chapter 3's cards: 2, 6, 6, 3; card 1 ⊕ card 3 = 4; D = 2; A, B, C, D = 5, 3, 7, 2.
  - 7/8 · 6/8 · 4/8 = 0.328; 2¹⁰ = 1,024 and 2²⁰ ≈ a million.
  - Clocks: on the 256 clock, 2·128 → 0 and 2·131 = 262 → 6. On the seven-hour clock, 3·5 → 1, and rows 1–6 are reshuffles. The primes either side of 256 are 251 and 257.
  - Polynomials and folds:
    - (x+1)(x²+x+1) = x³+1 = 1001 = 9.
    - Under the rotation fold, 2·255 = 255 and 3·255 = 0.
    - 16+8+4+1 = 29, and 2·142 = 1.
    - 87·131 = 49, and 193 under AES's polynomial.
  - The byte clock:
    - Doubling goes 1…128, 29, 58, and gives 255 distinct values.
    - Clicks: 7 + 2 → 58, and 180 + 25 = 205 → 167.
    - Each partner pair's clicks add to 255.
  - Cost: 50,000³/3 = 4.17×10¹³ steps, which is 11.6 h.
  - The ring: cards 6, 4, 5, 14, 9, with E⊕2A = 6. The first four cards ex-or to 9, and 9⊕6 = 15 = 3A, so A = 5.
  - The parking toy (from `toy_examples.py`): the cards are C, A⊕C = 2, B⊕C⊕D = 6, A⊕B = 6, D⊕E = 14 and B⊕E = 15, and c = 15⊕14⊕6 = 7.
  - 6,316/256 ≈ 24.7.
- **Unchecked (I took these from the fact sheet):** 97,876; 2.64 loops, 23,500, five or six loops; 71/42/23/12%; "twenty spares"; the 98% stall and 116–176 missing pieces; one in 120 untouched; "ten to sixteen"; 70.7% and 0.646%; 6,316 / 24 / 1; the race counts; the deployments.
- **Timing (my own estimate at 150 words a minute, not the voice's real speed):** every chapter's speech fits under 85 s. Chapters 10, 11, 14, 16 and 17 already have 72–77 s of words before holds, so I give the word cost of each fix there.

## Comprehension check

| ch | what I'd tell a friend | sure? |
|---|---|---|
| 01 | A million phones each lose different packets, so asking again swamps the sender. A loop makes the unlucky ones wait five or six rounds, and random pieces take about 98,000 for 10,000. Yet two extra packets are enough, thanks to an arithmetic where 3 × 7 = 9. | solid |
| 02 | Computers write numbers in bits. Ex-or flips the first number's bits wherever the second has a 1, and doing it twice undoes it. That's how we'll mix pieces. | solid (but *why* mix comes only in 03, M19) |
| 03 | Each packet is a card, an equation saying which pieces it mixed. The phone solves the cards like algebra and only needs enough useful ones, in any order. That's a fountain code, and RaptorQ is the one we'll open up. | solid |
| 04 | With three pieces, the eight possible cards sit on a cube's corners. Each useful card doubles the lit corners, so with two cards in hand a random card is useless half the time. | solid |
| 05 | Exactly enough random ex-or cards fail 71% of the time, and almost all the danger is in the last few cards: 1 in 8, 1 in 4, 1 in 2. | solid |
| 06 | Spares only roughly halve the risk, so one in a million needs about 20 spares. A ring of five cards is stuck no matter what. So we want cards that can scale a piece, which needs multiplication you can undo. | shaky on *why scaling helps* (M3) |
| 07 | Wrapping at 256 breaks dividing (2 × 128 = 0, so 2A = 6 could be 3 or 131). Only odd or even matters, so it fails 71% just like ex-or. | solid |
| 08 | A prime-sized clock lets you divide, but 257 doesn't fit in a byte. We need a "field" with exactly 256 members. | OK ("why not 251?", M18) |
| 09 | Read a byte's bits as a polynomial where 1 + 1 = 0. Then 3 = x+1 and 7 = x²+x+1, and their product is x³+1 = 9. | solid |
| 10 | The overflowing bit has to fold back in. A plain rotation fails (3 × 255 = 0) because x⁸+1 splits. RaptorQ folds with a polynomial that can't split, so 2 × 128 = 29 and every byte has a partner. | shaky: I take "secretly treats as zero" on faith (M4) |
| 11 | Doubling from 1 visits all 255 nonzero bytes, so every byte is a number of clicks on a 255-hour clock. Multiplying is adding clicks, and that's how the standard does it. | solid |
| 12 | Sorted by clicks, the noisy times table turns into diagonal stripes. The byte clock is the whole table. | OK (what does a colour mean? M17) |
| 13 | With random byte amounts, a card at the last slot is useless 1 time in 256 instead of 1 in 2. In the race, ex-or needs one or two spares and byte cards almost none. | solid in words; the picture lost me (M5) |
| 14 | Byte cards that use every piece are too slow (half a day for 50,000 pieces). Sparse cards solve by peeling, but peeling gets stuck and leaves pieces missing. | solid |
| 15 | RaptorQ sends only ex-or packets but adds 10–16 free byte cards aimed at the dangerous last slots. That cuts failure from 71% to about 6 in 1,000. | wobbly: first or last? (M2) |
| 16 | When peeling is stuck, RaptorQ parks a piece as an unknown, keeps peeling, and solves for it at the end. A byte card breaks the ring because A and 2A don't cancel. | I get the idea but couldn't follow the numbers by ear (M6, M7) |
| 17 | A real decoder with exactly enough packets fails about 6 in 1,000, with one extra 24 times in a million, with two extra once. That meets the standard's promise, and the same byte field is in TV, RAID and QR codes. | solid, except "divides by 256" confused me (M1) |

## Rubric (1–5)

For me, "correctness" means the numbers I could check came out right and nothing sounded wrong. Ruth owns the real maths.

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 4 | 5 | 5 | 4 | 5 |
| 02 | 4 | 5 | 3 | 4 | 4 |
| 03 | 4 | 5 | 4 | 4 | 5 |
| 04 | 5 | 5 | 4 | 5 | 4 |
| 05 | 4 | 5 | 5 | 5 | 4 |
| 06 | 3 | 5 | 3 | 4 | 3 |
| 07 | 4 | 5 | 4 | 4 | 4 |
| 08 | 4 | 5 | 4 | 5 | 4 |
| 09 | 5 | 5 | 5 | 5 | 5 |
| 10 | 3 | 5 | 4 | 3 | 4 |
| 11 | 5 | 5 | 5 | 4 | 5 |
| 12 | 3 | 5 | 3 | 5 | 5 |
| 13 | 3 | 4 | 4 | 4 | 4 |
| 14 | 4 | 5 | 4 | 3 | 4 |
| 15 | 4 | 4 | 5 | 4 | 4 |
| 16 | 2 | 5 | 4 | 3 | 4 |
| 17 | 4 | 3 | 5 | 4 | 5 |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| M1 | maya_14 | 17_2 (with 13_5, 13_7, 15_1) | major | I was told three things. RaptorQ's packets are all plain ex-or cards (15_1). With ex-or, each extra card "only roughly halves" the risk (13_5). A bigger field "gains nothing" if the amounts are only zeros and ones (13_7). So at the 17_2 predict I'd halve 6,316 and guess about 3,000. Then the answer is "Twenty-four." At the payoff, that sounds like a mistake or magic. Also, "extra card" in 13_5 meant a *byte* card. | 17_2: "Thanks to the byte cards the phone already has, even a plain ex-or packet now divides the risk by about two fifty-six. So with one extra packet, how many failures?" (+~12 words; Ruth to confirm the reason. The fact sheet's ex-or-only rows really do just halve, 70.7 → 42.7 → 22.9%, B-F21.) 13_7: add "…if every card's amounts are only zeros and ones." |
| M2 | maya_14 | 15_3 (with 15_2, 03_5) | major | 15_2 says both sides agree on the byte cards "before sending", so in my head they're in the bucket before any packet arrives. Then 15_3 says they "fill the last ten to sixteen slots". First or last? This is the video's main trick, and right now it sounds like a cheat. | 15_3, before the byte-card sentence: "The bucket doesn't care about order, remember? So count the ex-or cards first." This calls back 03_5 (+~14 words; 15 has room, about 52 s of words). |
| M3 | maya_14 | 06_5, 06_6 | major | This line tells me why the next six chapters exist, and I can't follow it when it's spoken. "With only use it or don't, a card…" sounds like a broken sentence, and "it would almost never be": be what? The "two hundred fifty-six amounts" come before the idea of scaling. Nothing says 256 amounts means one byte, yet 06_6 assumes it ("multiplying a byte"). | Reorder, about the same length: "So far a card can only use a piece or leave it out. Then a card at the last slot is useless half the time, and the ring stays stuck. What if a card could scale a piece? Three times A, or a hundred times A? With any byte as the amount, two hundred fifty-six choices, and the right arithmetic, a card would almost never be useless." 06_6: "Then a byte times a byte must give a byte, and we must be able to undo it." |
| M4 | maya_14 | 10_2, 10_3 (17_7 rests on it) | major | "The rotation secretly treats x to the eighth plus one as zero" is where I'd rewind and then give up, because nothing says why. The video's last line ("a polynomial refuses to factor") rests on this sentence, so I'd end the video taking the punchline on faith. The 10_2 predict is also one I can't do in the pause. | 10_2: add a hint I can use: "Hint: three times is double, ex-or itself." Then I can find that doubling 11111111 just rotates it back to 255, and 255 ⊕ 255 = 0 (checked). 10_3: replace the "secretly" sentence with "The rotation turns x to the eighth into one, so x to the eighth plus one becomes one plus one, zero. And x to the eighth plus one splits into x plus one, which is three, times a long polynomial, which is two fifty-five." Keep "two fifty-six was two times one twenty-eight all over again." About +20 words in all, and 10 is near the cap, so check the timing. |
| M5 | maya_14 | 13_4 picture (with 13_2–13_3, 16_1) | major | 13_2 and 13_3 show the danger meter at the last slot, and the 13_4 words say "exactly one choice of amount lands in the lit part". But the picture swaps to a "ghost copy of the file" drifting through the cards, which nobody explains. I'd be looking for the lit part and see a ghost instead. Then 16_1 has another ghost ("c") that means something else. | Keep the same object: at 13_4, spread the last card's 256 possible amounts as 256 dots on the meter (or the cube). 255 land outside the teal lit part and 1 lands in it. If the ghost file stays, it needs its own spoken sentence, and 16_1's parked "c" shouldn't be drawn as the same kind of ghost. |
| M6 | maya_14 | 16_2, 16_3 (with 14_3) | major | Heard aloud, this is a pile of numbers from nowhere: "B is six ex-or two ex-or c. D is six ex-or six ex-or two, so c cancels." I never heard this toy's cards, so I can't tell where the two c's came from. In 16_3 I can't tell where fifteen, fourteen and six came from either. I checked the toy: three of its cards *are* chapter 3's cards (A⊕C = 2, B⊕C⊕D = 6, A⊕B = 6). The others are C alone, D⊕E = 14 and B⊕E = 15. That's a lovely callback, and nobody says it. | 16_2, first words: "These are our cards from the start, plus a new piece E." Picture: at 14_3 and 16_2, reuse chapter 3's card objects, and light the card each line comes from beside its result. 16_3: "One card is left over, B ex-or E is fifteen. It says c is fifteen ex-or fourteen ex-or six, which is seven." (+~20 words; 16 is near the cap, so the source cards can carry part of this on screen.) |
| M7 | maya_14 | 16_4, 16_5 | major | "Walk around the ring. What do you get?" I can't predict that, because the ring's numbers were never given (06_4 showed only letters). "Three A is fifteen" is something I just have to watch. It's a predict that makes me feel stupid. | Ask what 06_4 lets me predict: "Walk around the ring. Does A cancel this time?" (I'd say no: A meets two A.) 16_5: say where fifteen comes from: "The other four cards add up to nine, and nine ex-or six is fifteen." (Checked: 6⊕4⊕5⊕14 = 9, 9⊕6 = 15, 3·5 = 15.) |
| M8 | maya_14 | 15_2, 16_1, 17_1, 17_4 | minor | "Block" is spoken four times, but only a lower-third explains it. At 17_1 ("small blocks of ten pieces") I'd ask why we tested ten pieces when our picture has ten thousand. | 15_2: "Big files are cut into blocks, and each block is solved on its own." 17_1: add "and about the same on blocks of a hundred and a thousand" (B-F22: 0.63% at 101, 0.54% at 1,002). |
| M9 | maya_14 | 17_4 | minor | "At most one in a hundred, one in ten thousand, and one in a million": of what? The line never ties them to exactly enough, one extra and two extra. | "…one in a hundred with exactly enough packets, one in ten thousand with one extra, and one in a million with two, averaged over random packets…" |
| M10 | maya_14 | 17_7 | minor | The video opened with my million phones, and the words never say what happens to them. One in a million, with a million phones: I want to hear "about one". | After 17_7's first sentence: "Out of our million phones, about one waits for one more packet." (Ruth to confirm.) This is what's left of R1 M37. |
| M11 | maya_14 | 10_6 or 11_8, 13_7, 17_6 | minor | "Byte field" is first spoken at 17_6, as if I already knew it. 13_7 says "a bigger field", but bigger than what? Zero and one were never called a field. | When the arithmetic is finished (10_6 or 11_8): "Bytes with this multiplying form a field. Call it the byte field." 13_7: "A card whose amounts are only zeros and ones is still an ex-or card, even in the byte field. All the gain comes from the other two fifty-four amounts." |
| M12 | maya_14 | 10_3, 10_4 vs 17_7; 13_7; 14_6 | minor | One idea, two words: "split" (10_3, 10_4) becomes "refuses to factor" in the video's last sentence. Elsewhere it's "amounts", but 13_7 says "values". "Stuck" and "stall" both appear in 14_6. | I know "factor" from algebra class, so use it in 10 as well ("factors into…", "a polynomial that can't be factored. A prime polynomial."). 13_7: "amounts". 14_6: "most runs get stuck around ninety-eight percent". |
| M13 | maya_14 | 14_7 → 15_2 | minor | 14_7 says one piece in a hundred and twenty isn't in any card and no decoder can find it. RaptorQ's packets are sparse too, so I wait to hear how it saves those pieces. Nobody tells me. | 15_2: "…extra cards that tie the pieces together by fixed rules, so even a piece no packet mixed can be worked out." (Ruth to confirm.) |
| M14 | maya_14 | 05_4 | minor | "Ten pieces or ten million, the answer is about seventy-one percent." The answer to what? And why doesn't size matter? | "Ten pieces or ten million, only the last few slots matter, so the lottery fails about seventy-one percent of the time either way." |
| M15 | maya_14 | 03_5, 06_1 | minor | "It needs enough useful ones." How many is enough? Later lines lean on "exactly enough" (13_6, 14_6, 14_8, 17_1, 17_4) and on "spares" (06_1, 13_6), and neither word was defined. | 03_5: "It needs as many useful cards as there are pieces." 06_1: "Spares help. Spares are cards beyond exactly enough." |
| M16 | maya_14 | 11_6 | minor | "…and three is twenty-five." In a video where three times seven is nine, for a second I'd hear "three equals twenty-five". | "…and three is twenty-five clicks." |
| M17 | maya_14 | 12_1, 12_2 | minor | Nobody says what a square's colour means, and the predict needs that. | 12_1: "Each square's colour is its answer." Then I can predict stripes from "add clicks". |
| M18 | maya_14 | 08_3, 08_4 | minor | "The nearest prime is two fifty-seven." Then why not two fifty-one, which fits in a byte? And why exactly 256 members? | 08_4: "We need exactly two hundred fifty-six members, one for every byte, so no byte goes to waste." |
| M19 | maya_14 | 02_3 | minor | The sticker album just showed the doubles problem, and 02_3 jumps to mixing without saying how mixing helps. I only find out at 03_5, a minute later. | "Instead of sending pieces, we'll send mixes of pieces, so no phone waits for one particular packet." |
| M20 | maya_14 | 03_3, 16_3 | minor | 16_3's "It's elimination in a clever order" brings in a new word on the line that says when the whole method works. | 03_3: "Solving works like elimination in algebra class." Then 16_3 becomes a callback. |
| M21 | maya_14 | 14_1 | minor | Our picture has ten thousand pieces, and suddenly it's "fifty thousand pieces". | "For a big file of fifty thousand pieces…" (This keeps R1's number and only says why it changed.) |
| M22 | maya_14 | 15_2 picture | minor | The screen says "7 ex-or rules + 10 byte rules" while the words say "cards". Rules and cards look like two different things. | Labels: "7 free ex-or cards + 10 free byte cards + 10 packets"; big block: "241 free ex-or + 11 free byte cards". |
| d1 | maya_14 | 11_1 | delight | "What comes next?" after 128, and I *know* it's 29 from 10_5. I'd shout it at the screen. | Keep it exactly here. |
| d2 | maya_14 | 09_4–09_6 | delight | FOIL, the matching pairs vanish, x³+1 = 1001 = 9, and the title finally makes sense. With "pause and try it" I get there before the answer. | Keep the hold, and keep x² + x² (never 2x²). |
| d3 | maya_14 | 01_7–01_8 | delight | The sticker album: 98,000 for 10,000 is a great shock. The three counts side by side (98,000 / 23,500 / 10,002) make the promise concrete. | Keep. |
| d4 | maya_14 | 03_3, 03_6–03_7 | delight | The fifth card, B⊕C = 4, is the one I made myself in 03_3. I felt clever. | Keep, and let the bounce take a beat. |
| d5 | maya_14 | 10_5–10_6 | delight | 2 × 128 was zero in 07_3 and now it's 29, with the same bit row and the same gutter. I checked 2 × 142 = 1 myself with the new rule. | Keep, and let 10_6's fold go slowly enough to follow by hand. |
| d6 | maya_14 | 11_3, 11_7, 11_8, 12_4 | delight | The star untangles into a circle, the wrong clock and right clock rhyme, and stripes come out of noise. | Keep. 11_7: add "because their clicks add up to one full lap" so I know *why* partners are mirror images (1 + 254 = 255 for 2 and 142, checked). |
| d7 | maya_14 | 17_2–17_3 | delight | 6,316 ÷ 256 ≈ 25, and the answer is 24. I predicted it. | Keep (with M1's clause, so the rule fits ex-or packets). |
| d8 | maya_14 | 17_6 | delight | "Every QR code": the arithmetic is in my pocket. | Keep. |
| d9 | maya_14 | every chapter | delight | Every chapter has a ⟨predict⟩, and most are ones I can actually get: 04_4 half, 08_1 five, 13_3 one in 256, 14_4 nothing moves, 15_4 seventy-one percent again. | Keep, and fix the two I can't get (10_2 via M4, 16_4 via M7). |
