# R2 review of the script by Maya, 14 (verbatim)

# Review R2 (script) by Maya, 14: "The One Idea Behind Bridges, Concert Halls and Economies"

**Verdict:** I would finish this video, and I found no blocker. I could say one honest sentence for all 14 chapters, though the sentences for 6, 11 and 13 are partly guesses. The nine majors share one cause. In the 17.6 to 15 minute cut, the sentence the next line needs sometimes moved from the voice to a lower-third. When I'm watching the picture I can't also read a lower-third. The worst places are 06_7, 11_1 and 13_4. Where I'd quit is 14_3, just before the ending. Where I'd rewind: 02_5, 04_3, 09_3, 13_4. Where I'd feel clever: 02_6, 05_5, 06_3, 13_5.

## What I checked, and how

- **Read:** my persona prompt and the whole packet (all 1,720 lines: instructions, brief, script, storyboard, description, fact sheet). I did not read any other review file.
- **Recomputed with my own scratch program (outside the repo, NumPy):** every number below reproduced the script and storyboard exactly.
  - Carts: pushes (2,−1), (1,1), (3,−3); eigenvalues 1 and 3; largest turn 30°; moves (2/3,1/3) and (1/3,2/3).
  - Truss: 21 members, 21 free moves, 97 of 441 not zero, eigenvalues 3.12 to 1,226 MN/m (factor 393).
    - Sag 17.96 mm (span/1,336); lone load 4.66 mm down, 1.33 mm at the roller, all 21 move, 64° turn.
    - Member forces −450, +400, −353.6, +212.1 and 0 kN.
    - Truncated sag 99.6, 102.0, 101.8, 99.1, 98.9, 99.1 and 100.0 percent; worst force errors 82, 44, 8 and 0 kN.
    - Modes 5.91, 11.78 and 14.86 Hz, and 4.32 Hz with the 8 tonne deck.
  - Rooms:
    - Bathroom: nine tones from 68.6 to 146.0 Hz.
    - Cube: 38 modes below 200 Hz with 36 sharing, lowest 57.2 Hz three times over.
    - 4.8 × 4 × 2.6 room: 67 modes below 200 Hz, none shared.
    - Narrow room: 48 below 200 Hz, none shared.
  - Hall:
    - Volume 18,630 m³ and lowest mode 3.8 Hz.
    - Counts 29, 2,246 and 1,964,040 at 20, 100 and 1,000 Hz.
    - Schroeder frequency 20.2 Hz; blur width 1.16 Hz; 1,579 m² of absorption.
  - Economy: eigenvalues 0.7 and 0.2; rounds 24, 16.8, 11.76; total 80; lopsided order 18 + 4 giving 75 and 50; rounds 15/11, 9.3/8.5, 6.27/6.11; hungry table 1.1 giving 26.4, 29.04 and −240.
  - Factory: 1,361 plans, best (40, 20) at 460.
- **Unchecked:**
  - The attic's 232 cells and its 29.8, 56.1 and 59.9 Hz (the cell layout is not in the packet).
  - Every cited number and quotation: dates, 70 mm, 156 and 10, 42 mph, 12 a minute, 1.9 s, 18,750 m³, 44 sectors, the years. That is Ruth's.
  - That walkers pushed harder "to keep their balance" (08_5). It is not in the fact sheet's quoted passages.
  - The pictures themselves; I only had the storyboard's description of them.
  - Real timing. I counted 1,959 spoken words, about 12.6 minutes at an assumed 2.6 words a second; michael's real cadence and the holds are unchecked.
  - Pronunciations.
- **Gaps in line ids:** 01_3 and 14_8 are cards in the storyboard. 08_6, 11_2, 13_2, 13_6 and 14_2 appear nowhere, so they read as cut ids. No viewer effect.

## Comprehension check (one sentence to a friend)

| Ch | What I'd say |
|---|---|
| 01 | A new London footbridge that had been checked still swayed so much it was closed in two days, because a bridge has ways of moving of its own that footsteps can find, and so do air and economies. |
| 02 | Two carts on springs: move one and you must push both, so the rule from moves to pushes is written as a table (a matrix), and moving both together gives the same pattern back. |
| 03 | Of all the ways to move the carts, only two (together, against) come back along themselves, just 1 or 3 times bigger; those are the own patterns, eigenvectors, with eigenvalues 1 and 3. |
| 04 | To find where a push sends the carts, split the push into the two patterns, divide each by its stiffness and add; tangled equations become two small divisions. |
| 05 | A steel truss is 21 moves in a mostly empty 21 by 21 table, yet one load moves every joint, and five loads sag it 18 mm, which is fine. |
| 06 | The truss has 21 own patterns; the soft ones dominate, so the softest alone gives the sag, but I could not say why the forces need the stiff ones too. |
| 07 | Add mass and the patterns become modes, each swinging at its own number of times a second (5.9, 11.8, 14.9 hertz), and a heavier deck makes the lowest one slower. |
| 08 | Tacoma and London each had a mode that something kept pushing along with, so neither was about strength; engineers call that negative damping, which I can't explain. |
| 09 | Air in a tube is carts and springs again, each own pattern has a tone, and a 2.5 m bathroom's lowest tone is 68.6 hertz, which I can test by humming. |
| 10 | A hall-sized box has nearly two million modes below 1,000 hertz, each blurred about a hertz, so above about 20 hertz they merge and the modes stop being the story. |
| 11 | Halls are designed by averages (I couldn't say which), modes rule small rooms where the cube is worst, and odd shapes get cut into cells and given to a computer. |
| 12 | An economy where making things uses things up is a table too: make equal amounts and it uses up 70 percent, so 0.7 is an eigenvalue. |
| 13 | To deliver 24 of each you make 80 of each (24 ÷ 0.3), a lopsided order swings onto the even mix (I couldn't redo that one), and an economy can feed itself when the largest eigenvalue is below one. |
| 14 | One calculation serves bridge, room and economy because in its own patterns a table only multiplies; it stops for quarter turns, bent steel and factory-profit problems; and the footbridge's engineers knew how it moved but not what would push along with it. |

## Rubric (1 to 5)

| Ch | Clarity | Correctness | Motivation | Pacing | Delight | Note |
|---|---|---|---|---|---|---|
| 01 | 4 | 4 | 5 | 4 | 4 | "code" and "Its" wobble (M-11, M-12) |
| 02 | 4 | 5 | 4 | 4 | 4 | |
| 03 | 5 | 5 | 4 | 4 | 5 | |
| 04 | 3 | 5 | 5 | 3 | 4 | payoff has no picture (M-01) |
| 05 | 3 | 4 | 3 | 3 | 4 | member, pin, yield (M-15, M-16) |
| 06 | 3 | 5 | 4 | 3 | 3 | 06_7 (M-02) |
| 07 | 3 | 5 | 3 | 4 | 4 | mode versus own pattern (M-21) |
| 08 | 3 | 4 | 5 | 4 | 4 | "negative damping" (M-03) |
| 09 | 3 | 5 | 4 | 3 | 5 | 09_3 overloaded (M-04) |
| 10 | 3 | 5 | 4 | 4 | 5 | "half-waves" (M-05) |
| 11 | 3 | 5 | 3 | 3 | 4 | "statistics" (M-06) |
| 12 | 4 | 5 | 4 | 3 | 4 | |
| 13 | 3 | 4 | 5 | 3 | 5 | 13_4 (M-07) |
| 14 | 3 | 4 | 5 | 3 | 5 | 14_3, 14_5 (M-08, M-09) |

## Findings

Majors first, then minors in chapter order, then delights.

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me | suggested fix |
|---|---|---|---|---|---|
| M-01 | Maya | 04_7 (and 04_1) | major | The chapter's whole promise is "two tangled equations have become two divisions". I know simultaneous equations from school, so this should be my moment. But at 04_1 the two equations are "small", and at 04_7 the stage is cleared to a sentence card. "Tangled" has nothing to point at. | Show `2a − b = 1` and `−a + 2b = 0` large for a beat at 04_1. At 04_7 put them beside the two one-line divisions (`½ ÷ 1`, `½ ÷ 3`), tangled on the left and separate on the right. |
| M-02 | Maya | 06_7 (and 05_8) | major | "Not for what is inside it. A force puts the stiffness back in, so the stiff patterns count." I couldn't tell what "it" is or what "back in" means. The reason lives only in the 05_8 and 06_7 lower-thirds. In the voice I'd nod and lose why forces need many patterns. | Say it plainly, keeping 82 kN. For example: "Not for the forces inside the members. A member's force is its stretch times its stiffness, so the stiff patterns, which the division shrank, count again. With one pattern the worst force is off by eighty-two kilonewtons, when the biggest is four hundred fifty." Speak the "stretch times stiffness" rule in 05_8, not only on screen. |
| M-03 | Maya | 08_7 (and 14_6) | major | "Engineers call that negative damping." Nobody told me what damping is, so the label has no meaning. At 14_6 the bridge "reopened" and I never hear how it was fixed, so negative damping never gets its opposite. | Gloss it once: "A swing dies down because something takes its energy. Engineers call that damping. Here the push gave energy instead, so negative damping." In 14_6: "It reopened in two thousand two, with dampers added to take the swing's energy out." (Source: 37 viscous dampers, K-B2.dampers_viscous.) |
| M-04 | Maya | 09_3 | major | This is the densest line in the video. It has a jagged push, a smooth hump that comes back, sloshing, and a tone per hump. Over it run a German quote card, a "35°" readout and a formula lower-third. I would read none of that and lose the line, and the German means nothing to me. | Split 09_3 in two beats (the jagged push, then the hump that comes back). Move the Helmholtz card to a silent beat after 09_5, or to the description. Put the formula lower-third in the description. |
| M-05 | Maya | 10_1 | major | "A mode counts half-waves along the length, the width and the height." In 09_3 I learned "humps" (one hump, two, three). Now there is a new word I don't know for the same thing. "Mode" and "tone" are also in the same two sentences, and I can't tell which is which. | "A mode counts humps along the length, the width and the height. One formula gives the tone of every mode of a box with hard walls." Keep "humps" throughout. |
| M-06 | Maya | 11_1 | major | 10_x showed me the modes don't decide a hall. The replacement is "Halls are designed with statistics, which began with Wallace Sabine", and I never learn which statistics. The plain rule ("the bigger the room, the longer a sound lingers…") is on screen only. | Speak it: "Halls are designed with averages. The bigger the room, the longer a sound lingers, and the more its walls and people soak up, the shorter. That began with Wallace Sabine." |
| M-07 | Maya | 13_4 | major | "Each round shrinks each part by its own eigenvalue", then the screen shows "18 ÷ 0.3 = 60 and 4 ÷ 0.8 = 5". I know 0.3 from 13_3. I don't know where 0.8 comes from, and 18, 4 and the answer 75 and 50 are never said. If I stop to read, the voice is gone. | Add the missing step in the voice: "The first part keeps three tenths, the second keeps eight tenths. Eighteen over three tenths is sixty. Four over eight tenths is five. So seventy-five of food and fifty of tools." If that is too long, keep only the swing in the voice and cut the totals from the screen. |
| M-08 | Maya | 14_3 | major | Three theorem-sized claims in about 15 seconds, right before the finale. "Symmetric" is glossed only in a lower-third, "enough" and "splits" are loose, and "Having nothing negative makes the pattern with the largest eigenvalue one you can make" took me three reads. This is where I'd stop paying attention. | Cut to what the finale needs: "Tables like the bridge's and the room's always have enough. The economy's has enough too, because its two eigenvalues are different." Leave the nothing-negative sentence in the 12_5 lower-third and the description. If it stays: "Because the economy's table has no negative numbers, the pattern of its largest eigenvalue has none either, so you can really make it." |
| M-09 | Maya | 14_5 | major | The factory picture (three limit lines, a green region, "$460: linear programming") arrives with no set-up, in a sentence about three other things. "Eigenproblems" is a word I have never heard. "A quarter turn" is a quarter turn of what? | One sentence for the factory: "A factory with limited metal and hours wants the most profit; that is solved at a corner of a shape, and no eigenvector finds it." One clause for the turn: "A rule that turns every arrow a quarter turn has no own pattern you can draw, since nothing comes back along itself." Replace "eigenproblems" with "something this idea can solve". |
| M-10 | Maya | 01_1 | minor | "About seven centimetres" sounded small to me, and nothing in the voice says why that closed a bridge. | I don't see a sourced sentence on what walkers felt in the fact sheet, so ask Ruth whether one exists. Otherwise let the picture show the walkers stumbling and gripping rails, and keep the crowd as a number. |
| M-11 | Maya | 01_4 and 14_6 | minor | Two things. (a) "The British code they worked to" and "What no code had asked": I first heard computer code, and a code doesn't ask. (b) 01_4 says they "had checked how it would bounce", but 14_6 says they "had known how it would move". My first thought was "which is it?". The word "Its" in 01_4 comes after the title card, and I'd lost what it meant. | "The footbridge's designers had worked out how it would bounce and how it would sway. The British rulebook they worked to said nothing of walkers pushing a bridge sideways." In 14_6: "What no rulebook had asked…". |
| M-12 | Maya | 01_6 | minor | The line ends on "a walker's weight shifts … about once a second", and the link to "that footsteps can find" is left to two drifting counters. I got there late. | Add a short sentence that keeps "not locked" true, such as "About the same rate." |
| M-13 | Maya | 02_5 | minor | The table 2, −1, −1, 2 appears in one sentence. I can run the rows, but not why those four numbers. | "The first column is what we just found. The second is the same for the right cart." |
| M-14 | Maya | 04_3 | minor | "Against has stiffness three, so its half shrinks to a sixth": I guessed why we divide. | "It takes three newtons to move it one centimetre, so a push of a half moves it a sixth." |
| M-15 | Maya | 04_5 | minor | 5 seconds is not enough to redo the split with a sign flip. I'd only get it by seeing the mirror, and the answer line says "turned round" afterwards. | 8 seconds, or reword to invite the mirror: "Push the right cart instead. It's the left cart's story, turned round." (no hint on the card). |
| M-16 | Maya | 05_2, 05_3 | minor | "Each member is a spring": nobody said what a member is (only the screen does). "The supports hold three": I can't check 24 − 3 without knowing the pin holds two ways and the roller one. | "Each member, each steel piece of the frame, is a spring." "The pin holds its joint both ways, the roller only up and down, so the supports hold three." |
| M-17 | Maya | 05_7, 05_8 | minor | "Less than a finger's width" for 18 mm sounded wrong: my finger is about 15 mm. "What makes its steel yield" made me think of a yield sign. | "About a finger's width." "Under a third of what would bend the steel for good." |
| M-18 | Maya | 06_1 | minor | "The move one load caused is held by a push at a single joint, a different pattern" is tangled. | "Take the move one load caused: every joint moves. The push that holds it is one arrow at one joint. Not the same pattern." |
| M-19 | Maya | 06_5 | minor | For the carts I found the halves by eye. For 21 patterns who finds the shares? (`q·f` is on screen and never named.) | "A computer finds the shares; the sum is the same as for the carts." |
| M-20 | Maya | 06_6 | minor | The readout goes 99.6, 102.0, 101.8, 99.1, 98.9. I'd read that as more patterns making it worse, and nobody says it wobbles. | "Add more and it wobbles around the true sag, then settles." |
| M-21 | Maya | 07_1, 09_3, 10_1, 10_5, 11_1 | minor | Names drift. "Own patterns shift … into modes" never says a mode is the engineers' name for one. 09_3 goes back to "own patterns" and 10_1 to "mode". Then tone, note and pitch all appear. Two words for one thing make me think they are two things. | Say once at 07_1: "Engineers call these own patterns modes." After that use "mode" for patterns and "tone" for what you hear, and drop "pitch" and "note" where "tone" works. |
| M-22 | Maya | 07_5 | minor | The on-screen tag "not 5.9 ÷ √2 = 4.2" brings in a √2 nobody has said; it reads as noise. | Move it to the description, or say "not half as fast, because the steel did not get heavier". |
| M-23 | Maya | 07_6 | minor | Why "the lowest few", and why "feed" (here) and "push along with" (08_7)? One idea, two phrases. | "The lowest few, the softest, which are the ones footsteps and wind can reach." Use "push along with" throughout, or gloss "feed" once. |
| M-24 | Maya | 08_3 | minor | "The wind's push followed the motion, and on every swing it gave more than it took back": "followed" sounded like "came after", and the bookkeeping is new. | "The wind pushed the way the deck was already turning, so every swing came out a little bigger." |
| M-25 | Maya | 08_5 | minor | "To keep their balance" gave me a reason I couldn't find in the fact sheet's quotes, so I can't say it is sourced. | Ruth to confirm a source for the reason, or drop the clause. |
| M-26 | Maya | 09_4 | minor | The speed of sound (343) is only on the card. I don't know it, and 6 seconds is short. | Say it: "Sound travels about three hundred and forty-three metres a second." |
| M-27 | Maya | 09_5 | minor | "Near a hundred and seven hertz": the bathroom's 1.6 m width was never mentioned, so 107 comes from nowhere. | "It is one point six metres wide, which gives a hundred and seven." |
| M-28 | Maya | 10_5, 10_6 | minor | "Dies away, so no mode is one exact pitch": the "so" didn't follow for me. "The crossover" shows up as a new noun in 10_6. | "A sound that fades is never one exact pitch." "Above that frequency the modes merge. That crossover is called the Schroeder frequency." |
| M-29 | Maya | 12_5 | minor | "A negative amount of one good": "good" sounded like good-or-bad, and I wondered "so what?". | "Needs a negative amount of food or of tools, which no one can make." |
| M-30 | Maya | 13_3 | minor | "The economy keeps three tenths of what it makes" is true only for equal amounts. I would try it on the lopsided order next, get (100, 33), and be confused. | "When it makes equal amounts of each, the economy keeps three tenths." |
| D-01 | Maya | 01_1 | delight | "And it had been checked." That made me want to know. | Keep. |
| D-02 | Maya | 02_4 | delight | "You moved one cart, and had to push two." I got the problem in one breath. | Keep. |
| D-03 | Maya | 03_4 | delight | The sweep, with "two lines only" and then the names. I felt I had found it myself. | Keep. |
| D-04 | Maya | 05_5, 05_6 | delight | A mostly empty table followed by "will only that joint move?" sets a trap that works; "tied only to its neighbours, and still they all move" lands. | Keep. |
| D-05 | Maya | 09_5 | delight | Humming in a bathroom to test it. I'd do this tonight. | Keep. |
| D-06 | Maya | 10_4 | delight | "Nearly two million." Three words, and the picture does the rest. | Keep. |
| D-07 | Maya | 13_3, 13_5 | delight | 24 ÷ 0.3 = 80 I could do in my head, and "1.1 of each for every one it makes" let me predict. | Keep. |
| D-08 | Maya | 14_7 | delight | The three questions close the loop with 01_5. | Keep. |
