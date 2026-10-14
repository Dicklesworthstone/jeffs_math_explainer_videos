# R2 review of the script by the domain-engineer lens (verbatim)

# Engineer-lens review of R2 (script, before narration): "The One Idea Behind Bridges, Concert Halls and Economies"

**Result: no blockers and no majors.** I found eight minor objections: five wording or attribution points, two missing qualifiers and one scope tag. Every computed number I could rebuild matched the packet. Nothing in the spoken lines is wrong at the level the three practitioners would reject the video.

## Findings

| id | field | line | severity | What the practitioner would object to (standard statement) | Suggested fix |
|---|---|---|---|---|---|
| EN-1 | acoustics | 11_5 (spoken) | minor | "Real rooms are rarely boxes" is false as a generalisation. Most living rooms, studios, control rooms, classrooms and bathrooms are rectangular, and the rigid-wall box formula is the working tool for them. The video's own examples (the bathroom, the 4.8 x 4 x 2.6 m room, the cube) are box rooms. Boston itself is a shoebox. What fails is rigid walls, furniture and sloped or splayed surfaces. | "And some rooms are not boxes, an attic with a sloping ceiling for one, and then there is no formula." |
| EN-2 | acoustics | 11_5 picture and "29.8, 56.1, 59.9 Hz" | minor | The 232-cell model is a 2-D side view, a room extruded endlessly sideways. A 2-D section yields only the n_z = 0 family of modes. A real attic of width W adds modes at c/2W, so a 7 m wide attic would have its lowest mode at 24.5 Hz, below the 29.8 Hz shown. | Add an on-screen tag: "side view only: the width adds its own modes". |
| EN-3 | acoustics | 11_4 lower-third | minor | "(Beranek)" trails "three of the most admired concert halls are shoeboxes: the trouble is sides that match, not boxes". Beranek is the source for the shoebox halls. "Sides that match" is a small-room result (Walker, Morse). In a hall above its Schroeder frequency, mode coincidence is not the concern. | Attach "(Beranek)" to the first clause only. Tag the second clause "(small rooms: Walker, BBC 1993)". |
| EN-4 | economics | 13_7 spoken "every demand can be met"; its lower-third; description "An economy" paragraph | minor | In Leontief's open model, ρ(C) < 1 (Hawkins-Simon) says only that intermediate inputs do not swallow the output. Labour, land and capital sit outside the table and are unconstrained. "Every demand can be met" is therefore conditional on primary factors. A lay viewer may read it as a statement about a real economy feeding itself, and primary factors are the binding constraint in practice, which is exactly the linear program of 14_5. No exact version exists anywhere (the lower-third "precisely" and the description are silent on primary inputs). | Add to the lower-third and description: "...every demand can be met, given enough workers, land and machines (they are outside the table)". Optionally speak it: "can be met, if the workers are there". |
| EN-5 | economics | 01_2 (with the "two bars rising in step" picture) | minor | The video's economy is Leontief's static open model. It has no time and no motion, and "rounds" are layers of indirect inputs, not periods. The eigenvector there is a balanced mix (proportions), not a "way of moving". "Moving" and growth belong to the dynamic or von Neumann model, which the brief puts out of scope. | "A bridge has ways of moving that are all its own. So does the air in a room. And an economy has a mix of its own." Draw the bars as a proportion that reproduces itself. Make sure the 13_1 picture or lower-third says a round is "one more layer of inputs". |
| EN-6 | structural | 08_7, 14_6 | minor | The narration never defines "damping", yet its negative is named as the answer. The 37 viscous dampers on 14_6 are unexplained on screen. Standard statement (Dallard): damping removes energy from vibration. The pedestrian force acts as negative damping, and the retrofit works by adding positive damping above the crowd's contribution. | One clause before "negative damping": "Damping is what drains a swing. This pushed energy in instead, and engineers call that negative damping." Tag the dampers at 14_6: "the fix was more damping, not a stronger bridge". |
| EN-7 | structural | 14_6 (spoken) | minor | "Its engineers had known how it would move" over-claims. What the design model had right was the frequencies, within 10% (Dallard). The sway amplitude and the critical-crowd onset were not known. Said flat, it reads as "they knew it would sway", which fights the opening. The exact version is on screen. | "Its engineers had known its modes." Or keep it and rely on the on-screen "sideways modes within 10%". |
| EN-8 | structural | 01_6 (spoken) | minor | "A walker's weight shifts from side to side about once a second" is ambiguous. The standard fact is that the lateral force fundamental is half the step rate, one full left-right-left cycle per second at 2 steps a second. "Side to side once a second" can be heard as one crossing a second, which is 0.5 Hz. The factor of 2 matters, because the argument is that 1 Hz sits at the top of the bridge's 0.5 to 1.0 Hz band. | "A walker sways left, right and back about once a second." |

Optional addition to "Names to search" (no defect): "modal stiffness" (04_3 calls K's eigenvalue "stiffness") and "room modes" or "eigentones" (the acoustics terms for 10_1 and 11_x).

## Checked and found fine

Rebuilt in numpy (scripts in the reviewer's own scratch folder: `truss.py`, `truss2.py`, `vib.py`, `rooms.py`, `attic.py`, `econ.py`).

**Structural**
- **Truss statics:** 12 joints, 21 members, 3 held, 21 free moves; K has 97 of 441 entries nonzero and its inverse 441 of 441.
- **Stiffness and sag:** the eigenvalues run 3.121 to 1,226.2 MN/m (ratio 392.9, "nearly four hundred"). Unsupported K has exactly 3 zero eigenvalues. Five 100 kN loads give 17.96 mm at midspan (span/1,336).
- **Forces and stress:** the member forces match the packet (top chord −450, bottom +400, middle post 0, end posts −353.6, diagonals 212.1). Top-chord stress is 75 MPa, which is 30% of 250.
- **One load:** 100 kN at the second interior deck joint gives 4.66 mm down and 1.33 mm at the roller; the move-to-push angle is 64.4°. All 21 free moves are nonzero, so "every joint but the pinned one" holds.
- **Truncation:** softest pattern alone gives 99.64% of the sag, then 102.0, 101.8, 99.1, 98.9, 99.1 (10), 100.0 (15). Worst member force error is 82.1, 44, 8 and 0 kN for 1, 5, 15 and 21 patterns. Hand check of the Pratt method of sections agrees with the force list.
- **Modes:** with 4 t at interior deck joints and 2 t at the ends, plus 3,421 kg of steel, f = 5.91, 11.78, 14.86 and 20.93 Hz, ratios 1 : 1.99 : 2.51 : 3.54. At 8 t it is 4.32 Hz.
- **"Shift a little" (07_1):** MAC between K's pattern i and mode i is 1.00, 0.99 and 0.985 for the first three, so the phrase is fair.
- **Mode 2:** 69% of its kinetic energy is longitudinal ("mostly slides the deck along its length") and mode 3 is 84% vertical ("S").

**Acoustics**
- **Hall counts:** the box 45 x 23 x 18 m has 29 modes at or below 20 Hz, 2,246 at or below 100 Hz and 1,964,040 at or below 1 kHz (Weyl estimate 1,964,120). Lowest mode is 3.81 Hz.
- **Hall figures:** Schroeder frequency is 20.2 Hz, mode width 1.16 Hz, density 5,802 modes per Hz, about 6,700 modes per width. Sabine absorption for T = 1.9 s is 1,579 m².
- **Bathroom:** the nine tones from 60 to 150 Hz match, and 343/5 = 68.6 Hz.
- **Small rooms:** the 4.8 x 4 x 2.6 m room has 67 modes below 200 Hz with none sharing a tone, and its Schroeder frequency is 200 Hz. The narrow room has 48 and none shared. The cube has 38, with 36 sharing, three at 57.2 Hz.
- **Windows:** the mode-in-window maxima are 6, 6, 6 for the cube against 2, 4, 6 for the other two, so "ties at the room's own width" is correctly disclosed.
- **Attic:** the 232-cell profile (flat 2.25 m, then sloping to 1.25 m) reproduces 29.76, 56.05 and 59.91 Hz exactly, and the 288-cell box gives 28.56 Hz.
- **Schroeder criterion:** the "three or more modes per half-power width" criterion derives to about 2,000 sqrt(T/V).

**Economics**
- **Table and eigenvalues:** eigenvalues are 0.7 and 0.2, with eigenvectors (1,1) and (3,−2). Ten of each uses up 7 of each.
- **Rounds and plans:** the rounds run 24, 16.8, 11.76, 8.232; 13 rounds reach 99%; the plan is (80, 80). The order (30, 10) is 18 and 4 of the patterns, plan (75, 50), rounds (15,11), (9.3,8.5), (6.27,6.11).
- **Hungry table:** the hungry table has eigenvalue 1.1 and the formula gives −240.
- **Logic:** the "ρ < 1 for every demand; ≥ 1 some demand fails; every demand fails if irreducible" logic is correct (left Perron vector argument).
- **Factory:** the corners are (0,0), (0,40), (40,20), (50,0) with profits 0, 280, 460, 400. There are 1,361 whole-number plans, best (40,20) at 460, and (0,60) breaks the aluminium limit.

**Simplified, with the exact version on screen or in the description (not defects)**
- "Within half a percent" (06_6): the readout shows the overshoot and settling.
- "Under a third of what makes its steel yield" (05_8): the buckling tag is on screen.
- "Halls are designed with statistics" (11_1): rays and geometrical acoustics are on screen and named.
- "Four tonne deck at every deck joint" (07_2): ends carry half, said in the description.
- "Stiffness" for K's eigenvalue (04_3): it is the modal or principal stiffness, and 07_1's lower-third gives ω².
- The code gap (01_4): a 2000 New Civil Engineer report quotes Dallard saying "BS5400 and other bridge codes are all about vertical input from pedestrians at 2Hz".

## Unchecked

- **Dallard et al. 2001 passages:** the quotes and tables (Table 3 modes, 10%, 1.3 Hz, 80,000 to 100,000, 2,000, 37 dampers, 4 and 26 pairs) are typed from the packet. I could not open the paper. The "BS 5400 Appendix C" detail has no web source.
- **Tacoma figures:** 12 cycles a minute at the end, 14 at onset, vortex shedding "close to 1 Hz", 42 mph and the board's wording are taken from the packet.
- **Millennium Bridge mechanism:** "walkers pushed harder to keep their balance" (08_5) is not in the packet's quoted passages. It is consistent with the balance-control literature, but I did not confirm the citation.
- **Boston Symphony Hall:** 18,750 m³, 1.9 s occupied, and the shoebox phrase from Beranek.
- **Sabine's role:** whether Symphony Hall was "worked out this way before it was built".
- **Helmholtz on-screen quote:** the 09_3 text is not among the packet's verbatim source passages.
- **Leontief's sector count:** 44 sectors is supported for the 1919 and 1929 tables (Kohli, Monthly Labor Review), but I could not confirm the count for the 1936 article. The on-screen "(1936, 1941)" is not verified for 1936.
- **Damper counts and dates:** the 54 versus 52 TMD counts and the 22 February 2002 reopening beyond the packet's sources.
- **Wording that cannot be reproduced:** the "less than a finger's width" comparison (18 mm is borderline) and the Strouhal estimate of about 1 Hz at 42 mph (order of magnitude only).
