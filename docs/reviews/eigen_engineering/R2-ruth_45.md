# R2 review of the script by Ruth Adler, 45, with the mathematician audit (verbatim)

# R2 review of the script, "The One Idea Behind Bridges, Concert Halls and Economies" (Ruth Adler, 45, with the mathematician audit)

## What I checked, and how

I read the whole packet: brief, script (what is heard), storyboard picture column, description draft and fact sheet. I read neither the R1 reviews nor any other R2 review. I did not open `oracle.py` or `facts.rs`, and I did not hear or see any render.

**Recomputed independently** (python3 -I, numpy through uv; scratch files in `.../scratchpad/ruth45/`: `truss.py`, `rooms.py`, `attic.py`, `hump.py`):

- **Truss, rebuilt from the member list** (Pratt, 12 joints, 21 members, E = 200 GPa).
  - Structure: 97 of 441 entries non-zero. Eigenvalues 3.12 to 1,226.22 MN/m, ratio 393, all 21 distinct (smallest gap 8.9 MN/m). Inverse has 441 of 441 non-zero. Unsupported, exactly 3 zero eigenvalues.
  - Statics: lone 100 kN at joint 2 gives 4.66 mm down and 1.33 mm slide at the roller, 64.4° between move and push, and the smallest of the 21 free moves is 0.22 mm (so all 21 move). Five 100 kN loads give a 17.96 mm sag (span/1,336) and member forces 450, 400, 353.6, 212.1, 0 kN, with 75 N/mm² = 30% of 250.
  - Truncation: the readout 99.6, 102.0, 101.8, 99.1, 98.9, 99.1 (ten), 100.0 (fifteen). Worst force error 82, 44, 8, 0 kN. Largest move error 1.2 mm.
  - Vibration: 4 t deck, steel 3,421 kg. Modes 5.91, 11.78, 14.86, 20.93 Hz, ratios 1.99 and 2.51. With 8 t decks 4.32 Hz against 4.18 naive. Mode 2 is 69% horizontal. φᵀφ is not diagonal, φᵀMφ = I.
- **Carts and economy** (exact fractions): everything in chapters 02 to 04 and 12 to 13.
  - Carts: sin θ = 1/2 gives 30°.
  - Economy: eigenvalues 0.7 and 0.2 with patterns (1,1) and (3,−2). Rounds 24, 16.8, 11.76, with a total of 80. (30,10) = 18(1,1) + 4(3,−2), rounds (15,11), (9.3,8.5), (6.27,6.11), plan (75,50). Hungry table eigenvalues 1.1 and 0.2.
  - Scaling test: 28 of 41 scaled tables feed themselves, and at 10/7 the largest eigenvalue is exactly 1.
  - Counterexample: [[1.1,0],[0.3,0.2]] meets (0,1) with (0,1.25).
- **Rooms and hall:**
  - Bathroom: nine tones 68.6 to 146.0 Hz, as listed.
  - Hall: 29, 2,246 and 1,964,040 modes at ≤ 20, 100 and 1,000 Hz. Estimate 1,964,120. 5,802 modes per hertz. 99.4% of 18,750 m³. Lowest mode 3.8 Hz. Schroeder 20.2 Hz. Width 1.16 Hz. Sabine area 1,579 m².
  - Small rooms below 200 Hz: study 67 modes, none shared. Narrow 48, none shared. Cube 38, with 36 on shared tones.
  - Tube: 40 slices, 0.0245% in frequency, 35.26° lump turn.
- **Dates:** 129 days, 2 days, 41 years.
- **Attic:** I reconstructed the 232-cell shape (8 flat columns of 12 cells, then a ramp to 5 cells over 24 columns) with the table "touching cells add 1 and −1". It reproduces 29.76, 56.05 and 59.91 Hz, and the full 24 × 12 box gives 28.56 Hz.

**Unchecked by me** (cited numbers and texts I cannot see):
- Dallard pp. 20 to 33 and Table 3, the 70 mm, 156 and 10, 42 mph, 12 a minute and 1 Hz (Billah and Scanlan), 1.9 s, 18,750 m³, 44 sectors, Nobel wording.
- The seeded 1,000 and 1,004 table counts (K-X1, K-X6).
- FrankenSim paths. The German quotations (I can read German but have no scan).
- Audio, rendered pictures, and score. Pacing is judged from word counts only (1,959 words, about 12.5 min at 2.6 words a second before holds).

---

## Comprehension check (one sentence per chapter, as to a friend)

| Ch | What just happened and why it mattered |
|---|---|
| 01 | A brand-new London footbridge swayed under ordinary walkers and was shut in two days even though it had been checked, so the question is what a bridge has besides strength that footsteps can find: ways of moving of its own. |
| 02 | Two carts on springs: move one and you must push both, and the push-from-move recipe is a little table of numbers (a matrix). |
| 03 | Swing the move around and the push lines up with it in only two directions (together, against). Those are the system's own patterns, with sizes 1 and 3: eigenvectors and eigenvalues. |
| 04 | To go from push to move, split the push into the own patterns, divide each by its stiffness, and add back: (2/3, 1/3) cm. Two tangled equations become two divisions. |
| 05 | A 24 m steel truss is a 21 by 21 mostly empty table. One load moves almost every joint; the full deck load sags it 18 mm and squeezes the top chord to 30% of yield. |
| 06 | The truss table has 21 own patterns. The softest, a plain sag, nearly gives the sag alone (99.6%) but not the forces (off by 82 kN), because a force puts the stiffness back in. |
| 07 | With mass the patterns become modes (5.9, 11.8, 14.9 Hz). A heavier deck makes the lowest slower (4.3 Hz), and any small motion is a mix of modes. |
| 08 | Tacoma's twisting mode and London's sideways mode were each fed by something that responded to the motion (wind, walkers), not by a lack of strength: "negative damping". |
| 09 | Air in a tube is the carts again. A smooth hump comes back as itself, each hump is a tone, and a 2.5 m bathroom answers a hum near 68.6 and 107 Hz. |
| 10 | A box the size of Boston Symphony Hall has nearly two million modes below 1 kHz, each blurred about a hertz, so above about 20 Hz they overlap: the mode picture stops being the useful one for a hall. |
| 11 | Halls are designed with statistics (Sabine). Modes still rule small rooms, where a cube piles three modes on one tone, and odd shapes are cut into cells for the same eigen-calculation. |
| 12 | An economy is a table of what making things uses up. Equal amounts come back at 0.7 of the size, which is the largest eigenvalue, and the other pattern (0.2) needs a negative amount. |
| 13 | To deliver 24 of each you make 80 of each (24 ÷ 0.3). A lopsided order's rounds swing onto the even mix, and with the largest eigenvalue at 1.1 the rounds grow and some demands cannot be met. |
| 14 | One calculation serves all three because in its own patterns a table only multiplies (symmetric tables always have enough). It stops at quarter turns, bent steel and linear programs. London's engineers knew how it would move but not what would push along with it. |

## Rubric (1 to 5; pacing from word counts only)

| Ch | Clarity | Correctness | Motivation | Pacing | Delight |
|---|---|---|---|---|---|
| 01 | 4 | 4 | 5 | 4 | 4 |
| 02 | 5 | 5 | 4 | 4 | 3 |
| 03 | 5 | 5 | 4 | 4 | 4 |
| 04 | 5 | 5 | 4 | 4 | 4 |
| 05 | 4 | 5 | 4 | 4 | 3 |
| 06 | 4 | 4 | 4 | 3 | 4 |
| 07 | 4 | 3 | 4 | 4 | 3 |
| 08 | 3 | 3 | 5 | 4 | 4 |
| 09 | 4 | 3 | 4 | 3 | 5 |
| 10 | 4 | 5 | 5 | 4 | 4 |
| 11 | 4 | 3 | 4 | 4 | 4 |
| 12 | 5 | 5 | 4 | 4 | 4 |
| 13 | 3 | 4 | 5 | 3 | 5 |
| 14 | 4 | 4 | 5 | 3 | 4 |

The correctness 3s are 07 (07_1), 08 (08_2, 08_5, 08_7), 09 (09_3 card) and 11 (11_5 and 11_1). The two pacing 3s that matter are 13_7 (six ideas in one beat plus a card, a lower-third and a grid) and 06_7 (24-word sentence).

---

## Findings

Order: what changes the narration, then the pictures and cards, then description and fact sheet.

| id | persona | chapter/line | severity | what misled me or is wrong | suggested fix |
|---|---|---|---|---|---|
| R-01 | ruth | 13_5 | major | "uses up one point one of each for every one it makes" is ambiguous in speech: "one point one" next to "every one". The natural parse (every unit made uses 1.1 of each good) is a table [[1.1,1.1],[1.1,1.1]] with eigenvalue 2.2. What is true is that making one of each uses 1.1 of each, i.e. the even mix returns at 1.1 times its size. The hungry table itself is never shown. | Mirror 12_2: "A hungrier economy. Make ten of each, and eleven of each is used up. What happens to the rounds?" |
| R-02 | ruth (+maya) | 08_2 vs 08_7, 14_6, 14_7 | major | The video rejects "the wind puffed in time with the bridge" (08_2: "It did not") and then closes on "something that pushed along with it" / "what would push along with it". To a listener "along with" is "in time with". The accepted mechanism is a push that depends on the motion, not an outside beat. This is the video's thesis phrase. | 08_7: "Each had a mode, and something that fed on its motion. The more it moved, the harder it was pushed. Engineers call that negative damping." 14_6 and 14_7: "what would push harder as it moved". |
| R-03 | ruth | 08_5 | major | "the harder walkers pushed, to keep their balance" gives a motive. The fact sheet's S1 passage supports only "the lateral force imparted by individuals increases" with amplitude. Brief item 8 says the wording must hold under both accounts. I believe the balance-control mechanism is in the literature (Macdonald; Belykh 2021's model), but no passage is recorded, so I leave it unchecked. | Say "the harder walkers pushed sideways" (what Dallard says), or add a sourced balance passage to S1 and keep the clause. |
| R-04 | ruth | 07_1 | major | "the own patterns shift a little into the structure's modes" is a universal. Recomputed with this mass distribution: the overlap (squared cosine) of mode j with K's pattern j is 1.00, 0.99, 0.98, 0.86, 0.85, 0.79 for j = 1 to 6, then ≤ 0.2 from mode 7 on. The best match for some modes is only 0.25. True for the lowest few, false in general, and there is no fact key for it. | "the lowest own patterns shift a little into the structure's modes". Add a fact key for the overlaps of modes 1 to 3. |
| R-05 | ruth | 11_5 (and description "Small rooms and the cells") | major | The attic's 232 cells are a two-dimensional slice (24 × 12 cells of 0.25 m). I reproduced the three numbers on a 2D table, and the 288-cell box gives 28.56 against 28.58, also 2D. "has its first modes at 29.8, 56.1, 59.9 Hz" is false for a real 3D room: a depth of 4 m alone adds a 42.9 Hz mode between the first two. "Side view" hints at this, but no line says it. | State it on screen and in the description: "a side view, as if the room were a long tunnel of this shape; a real room's depth adds modes". Replace "first modes" by "lowest modes of this slice". |
| R-06 | ruth | 09_3 card (Helmholtz) | major | The on-screen German "…eines der Eigentöne des elastischen Körpers…, sei dieser nun eine Saite, eine Membran oder eine Luftmasse" is not the passage the fact sheet says was read (S8, p. 147: "Ein Luftraum, welcher, wie der der Violine, … hat gewisse Eigentöne"). It carries two ellipses, no page, and a tidy triad that the recorded passage does not contain. The audit requires verbatim quotes with source and page. It becomes a blocker for the card if the page cannot be produced. | Produce page and scan for this text, or replace it with the recorded p. 147 sentence (a violin's enclosed air has its own tones) and its translation. Record the exact words and page in S8. |
| R-07 | ruth | 08_2 (also 08_1, description Tacoma) | major | "Its puffs would come about once a second, and the bridge twisted once every five. It did not." The whole denial rests on Billah and Scanlan's 1 Hz and 12 c/m. The fact sheet says the copy kept holds only the abstract and the quoted pages were not re-read at R1. I cannot check this either. A flat "It did not" should not go to narration on an unchecked pair of numbers. | Re-read p. 120 of the scan and record the verbatim lines and pages before narration. If it fails, say "It is not what the sources find". |
| R-08 | ruth | 11_1 | minor | "Halls are designed with statistics, which began with Wallace Sabine." The antecedent of "which" is "statistics", so it literally says statistics began with Sabine. | "Halls are designed with statistics. That began with Wallace Sabine." |
| R-09 | ruth | 11_1 picture ("Boston Symphony Hall, 1900: worked out this way before it was built") | minor | S10 records only that Sabine's 1900 paper uses "the new Boston Music Hall" as its example (p. 60) and that the hall opened 15 Oct 1900. The fact sheet's own "Not claimed" list refuses "designed by a formula". "Before it was built" is not recorded. | Show "Sabine advised the architects of Boston Symphony Hall, opened 1900", or add the source row. |
| R-10 | ruth | 01_4 | minor | "The British code … said nothing of walkers pushing a bridge sideways." The sources give "is being updated to cover the phenomenon" (Arup), i.e. "did not cover". "Said nothing" is a universal over the code's text. | "The British code they worked to did not cover walkers pushing a bridge sideways." |
| R-11 | ruth | 14_6 | minor | "What no code had asked": "no code" is every code in the world. The sourced fact is about BS 5400 (and the Arup page). | "What the code they used had not asked". |
| R-12 | ruth | 01_6 | minor | "This one swings sideways between half a swing and one swing a second." One pattern has one frequency. The sources give four sideways modes (0.48, 0.80, 0.95, 1.03 Hz). "A swing" is also left undefined against "weight shifts from side to side about once a second" (one cycle, i.e. two steps). | "Its sideways patterns swing between half a swing and one swing a second." Then define a swing as one full there-and-back. |
| R-13 | ruth | 09_3 | minor | "One smooth hump comes back as itself." Only the half-sine hump does. Recomputed on 40 slices: sine 0°, a parabola hump 22.5°, a Gaussian hump 54.6°. | "One smooth hump, half a wave, comes back as itself." |
| R-14 | ruth | 08_3 | minor | "on every swing it gave more than it took back." This "every" has no edge test. At a steady amplitude, input per cycle equals loss. True while the motion grows. | "so the swings grew: the wind gave more than the bridge could shed". Name the source passage in the edge table. |
| R-15 | ruth | 12_4, 13_4, 04_2, 07_6 | minor | "mix" means a sum of patterns (04_2, 07_6, 13_4 first use) and also a basket of goods (12_4 "making that mix", 13_4 "the even mix"). One sentence in 13_4 uses both. The brief's own rule says mix is a sum of patterns only. "Swing" is used for a cycle (01_6, 07_2), for rotating the arrow (03_4, 12_5) and for converging (13_4). | 13_4: "So the rounds swing onto equal amounts of each." Keep "mix" for sums of patterns. |
| R-16 | ruth | 01_2 | minor | "One calculation finds them all": the "all" has no row in the edge table, and 14_5 gives the quarter turn as a counter-case. | Add a fact-sheet row ("all own patterns of a table with enough of them") or "finds them" with the 14_1 qualifier. |
| R-17 | ruth | 13_7 lower-third, card | minor | (a) The Hawkins and Simon credit sits after the irreducible clause, so it reads as crediting the whole statement. (b) "each industry feeds every other" is the direct reading; irreducible means directly or indirectly. (c) "44 sectors wide (1936, 1941)": the 44 is sourced to the 1941 book through Kohli, and the 1936 text was not seen. (d) The spoken "every demand" means demands with no negative entry, not stated aloud. | (a) Place the credit after "below one". (b) "directly or indirectly feeds every other". (c) "(1941 book, for 1919 and 1929)". (d) Add "non-negative" on the card. |
| R-18 | ruth | 10_6 lower-third, description | minor | "2000 × √(decay time ÷ volume)": 2000 is dimensional (seconds, cubic metres). Without the units the formula is wrong in any other unit system. | "≈ 2000 × √(decay time in seconds ÷ volume in cubic metres)". |
| R-19 | ruth | 14_6 picture | minor | "37 viscous dampers and 4 pairs of sideways tuned mass dampers" for the reopened bridge. S1 says "are to be installed" (a 2001 plan), and the fact sheet itself notes that Building gives 54 tuned mass dampers while S1's plan is 60. The pair count is not needed for the story. | Drop the pair count, or write "planned 2001". |
| R-20 | ruth | 11_4 lower-third | minor | "(Beranek)" after "the trouble is sides that match, not boxes" attributes the conclusion to him. S13 supports only "the shoebox shape of the famous halls". "Sides that match" is Walker and Morse, and Walker's "simple integer ratios" is wider than "match". | "Three of the most admired halls are shoeboxes (Beranek). The trouble is sides that match or are in simple ratios (Walker)." |
| R-21 | ruth | 09_3 picture ("Eigenwerte: Hilbert, 1904, 41 years later") | minor | "41 years later" insinuates descent. The brief and description say nothing is claimed. | "the same word, 41 years apart", as the storyboard note itself says. |
| R-22 | ruth | 07_2 | minor | "a four tonne deck at every deck joint". The model gives the pin and roller joints half (2 t) each, and the deck joints in 05_7 are the five interior ones. Only the description says so. | "at each of the five deck joints", or put "end joints 2 t" in a tag. |
| R-23 | ruth | 09_2 vs 11_5 | minor | "the same table as before, only longer" (diagonal 2 at both ends, displacement of slices) against 11_5's table (1 at ends, wall adds nothing, eigenvalue 0 skipped: pressure in cells). Both are right but they are not the same table. | In the 11_5 lower-third say "cells hold pressure here, which is why walls add nothing and a zero appears". |
| R-24 | ruth | 06_6 | minor | "within half a percent" is the coincidence of two terms: patterns 2 and 4 contribute +2.35% and −2.74%. The readout shows 102.0 next. The narration can be taken as "one mode gives half a percent". | "the softest pattern alone lands within half a percent here". The description already has the exact version. |
| R-25 | ruth | notation (05_4 vs 10_1; description) | minor | `c` is the unit-vector cosine in the 05_4 code panel and the speed of sound in 10_1. The description uses V (volume, and the eigenvector table in `C = VΛV⁻¹`), A (area, absorption), N (member force, mode count), f (push, frequency), L (member length, room side, edge length), ρ and θ, against the brief's own rules. | Rename in the code panel (`(cx, sy)`). In the description use T60 = 0.161 V/A with "absorption" a, and N(f) as "count". Drop ρ and θ. |
| R-26 | ruth | fact sheet | minor | Stale cross-references. Finding 10 says "09_4 states the two dates" (the card is at 09_3). Finding 9 quotes a script line "worked that way on" that is no longer in the script. The factory is "his second image" in finding 2 and the brief's economy row, but "the owner's third image" in 14_5 and the description (where the truss is the third). The published description should not say "the owner's third image" at all. | Correct the ids and numbering; remove "the owner's third image" from the description. |
| R-27 | ruth | fact sheet S1, S6 | minor | No pages for the load-bearing quotations ("not anticipated during design", "within 10%", "below about 1.3 Hz", "as the amplitude … increases", the board's sentence). The S1 copy lacks pp. 22 to 23, where the lateral discussion probably sits. The 08_3 card is a clause, not the board's "whole" sentence as the storyboard note says. | Add pages, or mark which were read in other copies. Say "extract" on the card. |
| R-28 | ruth | 05_7 | minor | "less than a finger's width" for 17.96 mm. Index finger breadth is about 16 to 20 mm (unchecked). It is a coin flip whether the claim is true. | "about a finger's width", or cut. |
| R-29 | ruth | 07_5 picture | minor | "5.9 ÷ √2 = 4.2" against "4.3 Hz": at one decimal the two differ by 0.1, and the point is 4.32 against 4.18. | Show two decimals: 4.32 against 4.18. |
| R-30 | ruth | 07_6 lower-third, description | minor | "the Lanczos algorithm finds the lowest few modes": plain Lanczos finds extreme eigenvalues; for the lowest few of a stiffness table you use shift-and-invert (as FrankenSim's does). | "Lanczos, with shift-and-invert, finds the lowest few". |
| R-31 | ruth | title | minor | "Concert Halls" in the title, while chapters 10 and 11 say a hall above about 20 Hz is statistics, not modes. The video is honest about it, but the title promises the idea rules there. | Owner's call. Optional: "Bridges, Rooms and Economies". |
| D-1 | ruth | 13_4 | delight | The lopsided order's rounds swinging onto the (1,1) line is a visible Perron vector, and the arithmetic (18 ÷ 0.3 = 60, 4 ÷ 0.8 = 5, plan (75, 50)) checks exactly. | Keep. |
| D-2 | ruth | 03_5 lower-third | delight | The worst turn being exactly 30° from (3 − 1) ÷ (3 + 1) is a lovely little identity (verified). | Keep. |
| D-3 | ruth | 05_6 | delight | "97 of 441 not zero; inverse 441 of 441" shows why programs factor and never invert (verified). | Keep. |
| D-4 | ruth | 11_4, 07_5 | delight | The cube's lowest tone (57.17 Hz) times three (36 of 38 modes below 200 Hz shared). The 4.32 against 4.18 "the steel did not get heavier". Both verified. | Keep. |

---

## Mathematician audit, item by item

**1. Quantifiers explicit.**
- Fails: 01_2 ("them all"), 01_4 ("said nothing"), 07_1 ("the own patterns"), 08_3 ("every swing"), 09_3 ("one smooth hump"), 11_1 ("statistics, which began"), 13_5 ("for every one"), 14_6 ("no code").
- 13_7 and 14_3 are correct, with the domain (no negative entries) carried by the lower-third and by 14_3's "nothing negative".
- Checked and passing: 03_4 (two lines, 45° and 135° mod 180°), 05_6 (21 of 21 non-zero), 06_2 (21 distinct stiffnesses), 14_3 (spectral theorem), 14_5.

**2. Always/never sentences tested at the edges, test named in the fact sheet.**
- Listed and passing: 02_4, 03_4, 04_7, 05_6, 06_5, 07_6, 10_5, 11_1, 11_4, 13_7, 14_3, 14_5.
- Not in the edge table: 01_2, 06_2 (I checked: 21 distinct, minimum gap 8.9 MN/m), 07_1 (modal frequencies distinct, minimum gap 2.85 Hz; and R-04), 08_3, 09_3, 10_1 ("every tone of a box"; verified against my own enumeration).

**3. Every probability states its space.** Nothing to report. The script contains no probability, chance or random. "On average" appears only in the description, quoting Belykh et al. The seeded tables are tests, not claims.

**4. No probability of a fixed unknown, likelihood misnamed, confidence interval.** Nothing to report: no likelihood or confidence language anywhere.

**5. Terms used in the standard sense.**
- Fails: 12_4 and 13_4 ("mix" for a sum of patterns and for goods), 01_6 and 13_4 ("swing" twice), 11_1 (statistics).
- Passing: "irreducible" and "primitive" (the description says irreducible, correctly); "largest eigenvalue" (the Perron root, which for non-negative tables is the spectral radius); "symmetric"; "stiffness" for an eigenvalue of K (and ω² for the generalized problem, said in the lower-third); "table" for matrix.
- Not defined aloud: "negative damping" (08_7) and "yield".

**6. Every simplification has its exact version recorded.**
- Missing: 11_5 (slice is 2D, R-05), 07_1 (only the lowest few patterns shift little, R-04), 10_6 (units, R-18), 09_3 (which hump, R-13), 08_5 (balance, R-03), 14_6 (planned damper counts, R-19).
- Recorded and fine: 05_8 (buckling tag on screen and description), 06_6, 09_5 (all nine tones), 13_7 (precise lower-third), 14_3, 14_5.

**7. Historical claims verified (dates, attributions, quotes with source and page).**
- Dates I could compute: 129 days, two days, 41 years. The calendar dates themselves match the sources recorded (unchecked by me at the source).
- Fails: 09_3 card (Helmholtz quote, R-06), 11_1 picture ("before it was built", R-09), 08_5 ("to keep their balance", R-03), 01_4 and 14_6 (code claims, R-10 and R-11), 13_7 card (1936, R-17), 14_6 (planned dampers, R-19), 08_2 (Billah and Scanlan not re-read, R-07), pages missing for S1 and S6 (R-27).
- Passing: Hilbert quotation and pages (p. 51, p. 64 recorded); Nobel citation text matches the 1973 citation as quoted; Arup 156 and 10; Dallard 1.3 Hz.

**8. Numbers equal the oracle; voice and screen round the same way.**
- All recomputed numbers match to the shown precision (see list at the top). The rounding pairs are consistent: 17.96 mm and "eighteen", 5.91 and "five point nine", 393 and "nearly four hundred", 42 mph and "about forty", 99.6% and "within half a percent", 1.19 Hz and "about a hertz", 20.2 Hz and "about twenty", 1,964,040 and "nearly two million".
- Only R-29 (4.3 against 4.2) is thin at one decimal.
- Unchecked: the cited numbers and the seeded counts listed at the top. The 14_8 receipts claim ("every computed number … checked by a second program") is consistent with what I recomputed; I did not run their oracle.

**9. Edge cases acknowledged where they matter.**
- Unsupported truss, three zero eigenvalues: 05_4 lower-third.
- Zero eigenvalue skipped: 11_5.
- Repeated eigenvalues: cube, K-X1.
- Reducible tables: 13_7 and the description.
- Exactly 1: 13_7 "one or more".
- "Not all zero" indices: description only.
- Fails: the spoken demand is never said to be non-negative (R-17d).

**10. Notation consistent.** Fails: `c` (05_4 against 10_1) and the description's V, A, N, f, L, ρ, θ (R-25). Everything else on screen matches the brief's table.

**11. Nothing true-but-misleading.**
- Fails: 11_5 (2D, R-05), 13_5 (R-01), 08_2 against 08_7 and 14_6 (R-02), 06_6 (luck, R-24), 01_2 (R-16).
- Checked and acceptable: 05_8 ("under a third of yield" with the buckling tag on screen), 09_5 ("near 107 Hz the room hums back", with all nine tones in the picture), 11_3 and 11_4 ("share its lowest tone" is exact; the sheet records the ties), 14_6 ("had known how it would move", sourced to the within-10% quotation).

**12. Persona notes reached the narration, a card, a lower-third or the description.**
- Every Ruth, Dev and Maya bullet visible in the storyboard I could trace does reach one of those.
- Notes:
  - 07's "no damping" and 03's "an eigenvector is a direction" reach only the description, which the rule allows.
  - 08's "the board's sentence is quoted whole" is not true of the card (a clause, R-27).
  - I cannot see `decisions.md`, so I cannot judge persona notes dropped with a reason.

**Checked, nothing to report:**
- The script has no colons, semicolons, dashes or parentheses, and none of "simply", "just", "magic", "secret", "resonance", "synchrony" or "fell into step". The word "vector" occurs only in "eigenvector".
- The cart arithmetic, the economy chapters and the hall counts are all correct as stated.
- 12_4 "seventy percent … keeps thirty" is qualified by "that mix".
- 13_7 "some demand cannot" is true for every non-negative table with largest eigenvalue ≥ 1 (proof by the left Perron vector; verified on the split and equal-to-one cases).
