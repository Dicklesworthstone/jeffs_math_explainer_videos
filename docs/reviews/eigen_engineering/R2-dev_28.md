# R2 review of the script by Dev, 28 (verbatim)

# Review R2 (script): eigen_engineering, reviewer Dev, 28

I found no blockers. Seven findings would change the video, and they come first in the table (D1 to D7). Every number the script speaks reproduced from independent code.

## What I checked and how

- **Truss.** I rebuilt the Pratt truss in numpy from the 05_4 code panel alone, then solved, eigendecomposed and truncated it. Every packet number reproduced. The code panel is correct and sufficient.
  - 97 of 441 entries nonzero, and K⁻¹ is 441 of 441.
  - Eigenvalues 3.12 to 1,226 MN/m, ratio 393, and 3 zero eigenvalues when unsupported.
  - Sag 17.96 mm (span/1,336) and all 21 member forces (top chord −450, bottom +400, middle post 0).
  - Single load at x = 8 m: 4.66 mm down, 1.33 mm at the roller, 64° between move and push.
  - Sag percentages 99.6, 102.0, 101.8, 99.1, 98.9, 99.1, 100.0, and force errors 82, 44, 8, 0 kN.
- **Vibration.** I checked the 4 t deck with half masses at the ends and 3.4 t of lumped steel.
  - Modes 5.91, 11.78, 14.86 and 20.93 Hz, and 4.32 Hz with 8 t decks.
  - Mode 2 is 69% horizontal kinetic energy.
  - Overlap with K's own patterns for modes 1 to 3 is 0.98 or better, so "shift a little" (07_1) is true.
- **Rooms and hall.** I used a brute-force triple loop plus an integer-sqrt count.
  - Bathroom: the nine tones, 60 to 150 Hz.
  - Hall: 29, 2,246 and 1,964,040 modes (estimate 1,964,120), Schroeder 20.2 Hz, mode width 1.16 Hz, Sabine 1,579 m².
  - Rooms: cube 38 modes below 200 Hz with 36 sharing, narrow room 48 and study 67 with none shared, study crossover 200.2 Hz.
  - Tube: 40 slices give a 0.024% error, and a pushed slice comes back as (−1, 2, −1) at 35.3°.
- **Economy and factory.** Exact rational arithmetic and enumeration reproduced the rest.
  - Economy: eigenvalues 0.7 and 0.2, rounds 24, 16.8, 11.76, plan (80, 80) and (75, 50), shares 18 and 4, hungry 1.1 and −240, and a split table meeting (0, 1) with (0, 1.25).
  - Factory: corners and profits, and 1,361 plans.
  - Carts: 30° maximum turn, and 129 days from 1 July to 7 November 1940.
- **Attic.** I got 232 cells and 29.76, 56.05, 59.91 Hz only after guessing a ceiling flat for the first 2.5 m. The packet never says that.
- **FrankenSim.** I read the owner's repo read-only. The `let s = 317; // 100_489 unknowns` line, `rectangular_cavity_modes`, and `const E: f64 = 200e9` all exist at the repo's HEAD (dc8f32c28, not the cited e76c94213).
- **Unchecked (cannot open the sources).**
  - Dates, 70 mm, 156 and 10 people, 42 mph, 12 cycles a minute.
  - Beranek's 1.9 s and 18,750 m³, the 44 sectors and the 1973 prize wording.
  - Damper counts, A36 yield, and the Strouhal "close to 1 Hz".
  - Every quotation, including Dallard, Billah and Scanlan, Helmholtz, Hilbert and Sabine.

## Comprehension check

| Ch | One sentence to a friend |
|---|---|
| 01 | A London footbridge swayed under plain footsteps although its designers had checked how it would bounce, so a bridge has ways of moving of its own that a strength check does not ask about. |
| 02 | Nudging one of two spring-linked carts forces you to push both, and "which push holds which move" is a little table you multiply row by row. |
| 03 | A few directions of motion come back as pushes in the same direction, only scaled (eigenvectors, eigenvalues 1 and 3). |
| 04 | Write the push as a mix of those patterns, divide each part by its number, add, and two tangled equations become two divisions. |
| 05 | A truss is springs written into a 21×21 mostly-empty table, one load moves every joint, and five loads sag it 18 mm with 450 kN in the top chord. |
| 06 | The table has 21 own patterns up to about 400 times stiffer, soft ones dominate, so one gets the sag within half a percent but the member forces need many more. |
| 07 | With mass added the patterns become modes at 5.9, 11.8 and 14.9 Hz, heavier decks are slower, and any small motion is a mix of modes. |
| 08 | Neither bridge was too weak: each had a mode and something pushing along with it that fed it energy (negative damping), not wind puffing in time. |
| 09 | Air in a tube is the same carts-and-springs table, its own patterns are standing waves, and the lowest tone of a 2.5 m bathroom is 343 ÷ 5 = 68.6 Hz. |
| 10 | A hall-sized box has nearly two million modes below 1 kHz, each blurred about 1 Hz wide, so above about 20 Hz they merge and single modes stop mattering. |
| 11 | Big halls are handled with statistics, but in small rooms single modes rule and a cube stacks three on one tone, and odd shapes get cut into cells and eigen-solved. |
| 12 | An economy is a table of what making things uses up, and making an even mix uses up seven tenths of itself, so 0.7 is that pattern's eigenvalue. |
| 13 | Production rounds add up to a finite total only when the largest eigenvalue is below one, and at 1.1 they grow without end. |
| 14 | Symmetric tables always have enough own patterns, and the idea stops at quarter turns, bent steel and linear programming, because the London engineers had the modes but not what would push. |

## Rubric (1 to 5)

| Ch | Clarity | Correctness | Motivation | Pacing | Delight |
|---|---|---|---|---|---|
| 01 | 4 | 4 | 5 | 4 | 4 |
| 02 | 5 | 5 | 4 | 5 | 4 |
| 03 | 4 | 5 | 4 | 4 | 5 |
| 04 | 4 | 5 | 5 | 4 | 4 |
| 05 | 4 | 4 | 4 | 3 | 3 |
| 06 | 3 | 3 | 4 | 3 | 4 |
| 07 | 3 | 4 | 4 | 4 | 4 |
| 08 | 3 | 3 | 5 | 4 | 4 |
| 09 | 4 | 5 | 4 | 3 | 5 |
| 10 | 4 | 5 | 5 | 4 | 4 |
| 11 | 3 | 3 | 4 | 3 | 3 |
| 12 | 4 | 5 | 4 | 3 | 4 |
| 13 | 4 | 3 | 5 | 4 | 4 |
| 14 | 3 | 4 | 4 | 3 | 4 |

## Findings

| id | persona | chapter/line | severity | what confused or misled me | suggested fix |
|---|---|---|---|---|---|
| D1 | Dev | 14_1, 01_2, 05_7, 06_5 | major | The thesis is "one calculation serves a bridge". The caveat I most need, that nobody eigendecomposes K for one static load, is only a lower-third at 05_7, under a voice line about eighteen millimetres. After 04 and 06_5 I would leave thinking the split is how solvers work. Chapters 11 and 14_5 then say halls and scarce resources are not it either. | Say it aloud once, at 06_5 or 06_7: "That is how to see the answer, not how a program finds it. For one load a program solves the table directly. The patterns pay off when things move." Change 14_1 to "why does one idea serve…", which matches the title. |
| D2 | Dev | 05_3 vs 01_1, 01_6, 08_5 | major | "Sideways" means along the span in 05_3 (the "1.3 mm sideways at the roller"), and across the deck for the footbridge. The truss is flat and cannot sway or twist (a caption at 07_6 only), so none of its three modes is the kind that hurt London or Tacoma. Yet 08_7 says "each had a mode". | 05_3: "move along the span, or up and down". Add one spoken sentence at 07_6 or 08_1: "Our truss is flat. It cannot sway or twist, and those are modes too." |
| D3 | Dev | 13_4, 14_3 | major | I cannot get 18 and 4 with the dot-product recipe from 04 and the 06_5 formula. Dot products give 20 and 5.4, which rebuild (36.2, 9.2), not (30, 10), because (1, 1) and (3, −2) are not at right angles. The voice only says "It is a mix of both own patterns". The warning lives in the lower-thirds at 04_7 and 14_3. | 13_4: "These two patterns are not at right angles, so the shares come from solving two equations. Eighteen and four." |
| D4 | Dev | 13_7, 13_3 | major | The failure I would actually ship is `x = d/(1 − λ)`, which silently returns −240 of each for the hungry economy. It is in a lower-third, while the voice says only "the total never arrives". Separately, 13_3 "keeps three tenths of what it makes" is true of the even mix only: I recomputed that (30, 10) keeps 35% and (10, 30) keeps 25%. | Voice: "Put it in the formula anyway and you get minus two hundred forty of each. So check the number first." 13_3: "Of an even mix, the economy keeps three tenths." |
| D5 | Dev | 06_6, 06_7 | major | "Good enough for how far it sags" reads as general, but it is one smooth, symmetric load read at midspan. With the 05_5 lone load, one pattern gives 76% of the drop under it. It gives 42% at the first interior joint and 86% for a lone midspan load, and ten patterns are needed to get within 2.5%. The readout itself wanders about ±2% (102.0 at two patterns). | 06_6: "For this load, the softest pattern alone…". Add: "One sharp load needs many more." |
| D6 | Dev | 07_2, 07_5 | major | I could not reproduce 5.9 Hz from what is said or shown. The model has 4 t at the five middle deck joints, 2 t at each end joint, and 3.4 t of steel shared half to each end of every member. "Four tonne deck at every deck joint" read literally gives 5.85, 11.36 and 14.54 Hz. With no steel mass, M is singular at the five top joints, and a token mass gives 6.3, 12.6 and 15.7. Mode 2 slides along the roller, so it feels the end mass. | A lower-third giving the mass table. Voice: "four tonnes at each middle deck joint". |
| D7 | Dev | 07_1, 07_4, 07_5 | major | The chapter is "every mode a note", but I never hear what sets the note. The quiz answer "slower, four point three" has its reason (not 5.9 ÷ √2, the steel did not get heavier) only as a tag. The solver for Kφ = ω²Mφ is not named. | Spoken in 07_5: "Frequency goes as the square root of stiffness over mass, so more mass is slower. Not exactly root two slower, because the steel did not get heavier." Lower-third: solve M^(-1/2) K M^(-1/2) with eigh, which is what I did. |
| D8 | Dev | 08_7 | minor | "Negative damping" is named, but "damping" is never said aloud, so the name has nothing under it. | Add "the opposite of the friction that lets a swing die away". |
| D9 | Dev | 08_5 | minor | "To keep their balance" has no recorded passage in the fact sheet. The Dallard quote there says the force grows with amplitude, not why. It is a real standard explanation, but it needs its quote. | Add the source line to the fact sheet, or drop the clause. |
| D10 | Dev | 05_8 | minor | "Under a third of what makes its steel yield" is the wrong yardstick for a squeezed 4 m member, because buckling comes first. That is only in a tag. The spoken line invites reading it as a 3× margin. | "…well under what the steel could carry before yielding, though a squeezed member buckles sooner." |
| D11 | Dev | 01_4, 14_6 | minor | To a developer "code" reads as source code, and "what no code had asked" is ambiguous. "Said nothing of" is stronger than the sourced "is being updated to cover the phenomenon". "Had known how it would move" is stronger than "modes within 10%", and it clashes with 14_7's "how does it want to move". | "The British standard they worked to did not cover…", "no standard had asked", and "known how it wanted to move". |
| D12 | Dev | 01_6 | minor | "This one swings between half a swing and one swing a second" gives one mode a range. The range is four modes (0.48, 0.80, 0.95, 1.03 Hz), which is only explained on screen at 08_5. "Half a swing a second" is also clumsy. | "Its sideways modes run from one swing every two seconds to one a second." |
| D13 | Dev | 05_4 code panel | minor | The logic is right, but the notation snags. `a` and `b` are joints in `2a` and positions in `(b - a)`, `L` is undefined, indices are 0-based without saying so, and which three rows are struck is only in the picture. The panel sits on a hold of unknown length. | `d = pos[b] − pos[a]; L = |d|; (c, s) = d / L`. Say "rows of joint j are 2j, 2j+1 from zero". Hold the panel at least 6 s. |
| D14 | Dev | 05_5, 05_6 picture | minor | "Second deck joint" is ambiguous. 4.7 mm and 1.3 mm belong to x = 8 m, the third joint counting the pin. At x = 4 m I get 2.6 mm and 1.0 mm. | "second interior deck joint, 8 m from the pin". |
| D15 | Dev | 11_1 | minor | "Halls are designed with statistics" drops the "and rays" of the brief. "Which began with" has an unclear antecedent. | "Halls are designed with averages and rays. The averages began with Wallace Sabine." |
| D16 | Dev | 11_3, 11_4 | minor | "Piles its tones up most" holds only for exact coincidence. I recomputed: in a window of the room's own mode width (4.4 Hz), cube, narrow and study all have 6 modes. Below 100 Hz all three have 3. A tester who adds blur gets a tie. | Reword the question: "Which has several modes on exactly the same tone?", or add "in a perfect box". |
| D17 | Dev | 11_5 | minor | There are four small gaps. The cost of cells is unsaid (about 6 cells per wavelength, so roughly 10⁸ cells for this hall at 1 kHz, unchecked as a standard). The attic is a 2D side view whose "wall adds nothing, skip eigenvalue 0" table differs from 09_2's "same table". The 232 cells need the flat first 2.5 m. FrankenSim's casebook certifies modes in windows of a 317×317 grid, not "the lowest modes of a cell grid". | Add the cost line and "pressure form of the same table" on screen. State the ceiling rule. Say "finds certified windows of low modes". |
| D18 | Dev | 09_3 card | minor | The German card ("…eines der Eigentöne des elastischen Körpers…, sei dieser nun eine Saite, eine Membran oder eine Luftmasse") is not among the passages the fact sheet records as seen (S8 records "Ein Luftraum… hat gewisse Eigentöne"). I could not verify it. | Record page and verbatim text in the fact sheet, or use the S8 sentence. |
| D19 | Dev | 05_7 | minor | "Less than a finger's width" for 17.96 mm is unreliable, because fingers run about 16 to 20 mm. | "about one part in thirteen hundred of the span". |
| D20 | Dev | 12_1, 11_3 | minor | 12_1 has 46 words with four fractions in one breath. 11_3 has 43 words and two new ideas before a 5 s countdown. | Split 12_1 into two beats, and move "A room booms…" into 11_4. |
| D21 | Dev | 14_7 | minor | "How strong is it." and the next two end in periods, so a one-sentence-per-call TTS will read them as statements. | Use "?", or confirm intonation in the listen pass. |
| D22 | Dev | 06_1, 06_7, 08_3 | minor | "The move … is held by a push" and "what is inside it" are murky. At 08_3 "the wind's push followed the motion" says what, not why. | "the push that causes it", "the forces in the members", and one clause: the twist changes the angle the wind meets the deck (check against Billah and Scanlan). |
| D23 | Dev | 04_4 | delight | "The right cart, which nobody pushed, one third" is the moment tangled became legible. | Keep. |
| D24 | Dev | 09_2 | delight | The same table in the same screen place, only longer, is the best developer moment in the video. | Keep. |
| D25 | Dev | 12_2, 12_3 | delight | The quiz is built so the answer is an eigenvector, which is a neat trick. | Keep. |
| D26 | Dev | 06_7 | delight | Stating the 82 kN error aloud is what separates this from a sales pitch. | Keep. |
