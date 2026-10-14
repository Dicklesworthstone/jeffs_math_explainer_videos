# R2 review, ruth_45, with the mathematician audit (verbatim)

*Saved by the producer from the reviewer's returned text, with the reviewer's scratch folder written as `<scratch>` and one private path written as `<cmaes_explainer>`; nothing else was edited.*

---

# Review R2 (script), lie_robotics — ruth_45 (Professor Ruth Adler), with the mathematician audit

## What I read, ran and compared

- **Read in full:** the persona prompt, `ruth_audit.md`, all 1,343 lines of `R2-packet.md` (brief, script, storyboard, fact sheet, every fact row), and `explainer_corpus/lie_robotics/docs/protected.tsv`.
- **Mechanical checks, by my own `python3 -I` code, in scratch only:**
  - The packet's 83 script lines equal the storyboard words. The only unmatched id is 01_3, the title.
  - No spoken line has a colon, semicolon, dash, parenthesis or digit.
  - All 42 protected phrases are present in `script.tsv`.
- **Recomputation:** my code is built from the brief's link offsets and the standard formulas, with exact fractions or float checks as noted.
  - The axis directions at zero are the fact sheet's table. I chose the xacro roll pattern by brute force so that it reproduces that table, so the directions are not independent of the sheet.
  - The heights are my own sums of the brief's offsets.
- **Scope of "checked":** a number I did not recompute is marked unchecked. The recomputation table and the unchecked list follow the audit.

## Comprehension check (one sentence per chapter)

| Ch | What I would tell a friend |
|---|---|
| 01 the book | Two quarter turns of a book ended up facing a different way when I swapped their order, while two slides land in the same place, and a robot arm has to do arithmetic on exactly the thing that does not commute. |
| 02 one turn | Any two turns in a row are one turn about one axis (tilt then roll is a third of a turn about a diagonal, so three of them bring the book home), turns form a group where order usually matters, and a computer tracks them as 3-by-3 grids. |
| 03 three angles | Naming an orientation by three ring angles always has a bad spot, gimbal lock, where two rings do the same job and one way of turning is lost (Apollo hit it), so software stores nine numbers or four. |
| 04 halfway | Averaging heading numbers can point the opposite way, the fix is to lay a flat ruler against the circle, halve there and wrap back, and that one move is what the video keeps doing. |
| 05 spin | A spin is an arrow, holding it for a second and wrapping back is the exponential, adding two arrows turns about both at once (a third answer, not either order), and such a group is a Lie group with the arrows as its Lie algebra. |
| 06 platform | A platform driven forward and spinning at constant rates does not land 3 m ahead but goes round a half circle of radius 0.95 m, so velocity times time is the flat answer and the pose is that answer wrapped about a point. |
| 07 screws | A pose can change in six independent ways (a twist), holding a twist steady is a screw, and every rigid displacement is one screw (Chasles 1830, Mozzi 1763). |
| 08 seven screws | Each joint is a hinge, so a screw, and applying the last joint's screw first to the home pose gives the gripper's pose as a product of seven exponentials (Brockett 1983). |
| 09 by hand | Elbow first, then shoulder, puts the tip at 42 cm forward and 88.6 cm up, and joint one swings it round to the side, with each screw written down once for the arm at zero. |
| 10 velocities | Each joint's twist is carried to where the arm is now, twists add, the 6-by-7 Jacobian maps joint speeds to the gripper's twist, and what counts is the tip's distance from each axis. |
| 11 lost directions | Where the Jacobian drops rank (a singularity) a direction is lost, here the wrist's slide along the arm, and near it the elbow needs huge speeds (54, 160, 534, 1,602 °/s against a limit of 75). |
| 12 spare joint | Seven joints for six directions leaves one spare, so the elbow can swing on a 29 cm circle while the gripper holds one pose, like your own elbow with your palm on a desk. |
| 13 steps | The error to a goal is one screw read as a twist, so run the Jacobian backwards and repeat, which from this start takes three steps (505, 94, 4, 0.03 mm). |
| 14 steady path | A number-by-number blend of two orientations is not an orientation (half the volume at halfway), whereas holding the joining twist for half a second gives a true pose at every time. |
| 15 everywhere | The same moves run drones, mapping, physics engines and humanoids, and the leftover of two small turns in two orders is itself a turn of size angle times angle, which is what the Lie bracket measures. |

No sentence is wrong or empty, so there is no blocker.

## Rubric (1–5)

| Ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 4 | 3 | 5 | 4 | 4 |
| 02 | 4 | 5 | 4 | 4 | 5 |
| 03 | 4 | 4 | 5 | 3 | 5 |
| 04 | 5 | 4 | 5 | 4 | 4 |
| 05 | 4 | 5 | 4 | 3 | 5 |
| 06 | 5 | 5 | 5 | 4 | 4 |
| 07 | 4 | 5 | 4 | 4 | 5 |
| 08 | 4 | 5 | 4 | 4 | 3 |
| 09 | 5 | 5 | 4 | 4 | 4 |
| 10 | 4 | 5 | 4 | 4 | 4 |
| 11 | 3 | 4 | 5 | 3 | 5 |
| 12 | 5 | 5 | 4 | 4 | 4 |
| 13 | 4 | 4 | 5 | 4 | 4 |
| 14 | 4 | 5 | 4 | 4 | 5 |
| 15 | 4 | 5 | 4 | 3 | 5 |

## Findings

Findings that would change the video come first. "Words" means a re-spoken line, "picture" means the picture, a card or a lower third, and "paper" means the fact sheet, brief or description only.

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me | suggested fix |
|---|---|---|---|---|---|
| R-01 | ruth_45 | 04_3, 04_4 | major | "Halfway" between 170° and −170° has two answers on a circle. 0° is the midpoint of the 340° way round and 180° is the midpoint of the 20° way. The video's "true halfway" is the shorter arc's, and "unique unless the headings are opposite" is in the fact sheet only. Nothing on screen says so. "True" tells a viewer the other midpoint is false, when it is the right answer to a different question. The tag "bring the difference into −180…180" implies the convention without stating it. | **Picture, not words.** Lower third on 04_3 or 04_4: "halfway = the middle of the shorter way round (20°). 0° is the middle of the longer way (340°). Exactly opposite headings have two." "The true halfway is a hundred and eighty" is protected and stays. |
| R-02 | ruth_45 | 01_1, 01_2 | major | The words fix neither sense of the quarter turns, and "cover ends up facing you" and "faces the wall on your right" hold for one sense each. I computed all four sense combinations. Tilt-first faces ±y depending only on the tilt sense, and roll-first faces ±x depending only on the roll sense. The quiz answer ("different") is true for all four, so the lesson survives. But this is the one experiment the video asks viewers to try with a real book, and about half of them will fail a literal check of the stated facings in the first minute. | **Picture.** Chalk marks or lower third on 01_1: "far edge up, then roll to the right". Add to 01_2: "turn the other way and the facings change, the two orders still differ". If words must move, re-speak only 01_1 with "far edge up" and "to the right" added, keeping the protected "about one line on the table". |
| R-03 | ruth_45 | 11_6 | major | "A six-joint arm … would lose a direction here. This arm has a third, and it covers." The exact version (a straight wrist does cost this arm a direction in other poses) lives in the fact sheet only. I reproduced rank 5 at q = (0, 90°, 0, 90°, 90°, 0, 0). "Here" is attached to the first sentence, so the second reads as a general property of the seven-joint arm. This is the audit's "true but misleading" case. | **Lower third** on 11_6: "in this pose. Turn joint 5 a quarter and this arm also has five". Optionally add "here" to the end of the line. The protected clause "with two joints at the shoulder, would lose a direction here" is untouched. |
| R-04 | ruth_45 | whole script, no `description.md` | major | Everything the storyboard sends to "the description" is recorded nowhere. The items:<br>(a) page-level citations for Euler E478, Rodrigues JMPA 5, Chasles Bulletin 14, Mozzi 1763 via Ceccarelli, and Brockett MTNS-83 (text not read);<br>(b) the Collins transcript reference (tape 67/11 p. 337, 04 09 00 30);<br>(c) the wheel senses `G-P.wheels.*` (06 Dev);<br>(d) the 15_1 where-used sources (manif, Pinocchio, GTSAM, Lee–Leok–McClamroch, `se3_exp_step`);<br>(e) the "not verified" list (Brockett and Mozzi texts, Rodrigues's formula location, "iiwa" pronunciation);<br>(f) the Apollo thresholds are the command module computer's (Comanche 055);<br>(g) Lie's 1873–74 for "a hundred and fifty years";<br>(h) the L&P Exercise 2.2 and 2.3 credit for the palm on the desk and the model caveat;<br>(i) licences and "model" labelling. | **Paper.** Write `description.md` from the fact sheet's Sources and Not-verified sections. No words change. |
| R-05 | ruth_45 | 13_2 | minor | "Subtracting angles walks into both traps." The pair shown, (0°, 89°, 0°) and (90°, 89°, 90°), is only the gimbal-lock trap (1.4142° apart, two ring angles 90° apart). I computed that pair. No seam is involved, so the second trap (ch. 04) is claimed but not shown. | **Picture:** add a second row, "heading 179° − (−179°) = 358°, not 2°". Alternatively **words:** "the traps". |
| R-06 | ruth_45 | 11_5 | minor | "With the elbow bent thirty degrees, it must turn at fifty-four…". The nearest noun is the elbow, but the sentence before is about the gripper, which must not turn. "Its limit is seventy-five" has no unit and no source on screen. It is KUKA's A4 speed at rated payload, a published figure, not a hard law. | **Words, one word:** "the elbow must turn". **Lower third** on the limit line: "KUKA's published speed for joint 4 at rated payload". |
| R-07 | ruth_45 | 13_6 screen table | minor | "Each step gains more than the one before." The absolute gains are 411, 90 and 4.1 mm, so it is false as read. It is true as a factor (5.4, 22.6, 157). | **Picture:** "each step shrinks the error by a bigger factor than the last". |
| R-08 | ruth_45 | 03_6 lower third, 08–13, 15_4 | minor | Notation collisions:<br>(a) q is the joint-angle vector (T(q), V = J(q) q̇) and the unit quaternion in 03_6 ("q and −q");<br>(b) x, y, z are the three basis spins in 15_4's `[y, x] = −z` but position coordinates in the ch. 06 readout. | **Picture.** 03_6: "a quaternion and its negative". 15_4: write the bracket as roll's arrow × tilt's arrow = third arrow, or ω_x, ω_y, ω_z. |
| R-09 | ruth_45 | fact sheet Terms vs 15_4 | minor | The ledger says `[x, y] = z`, the storyboard says `[y, x] = −z`. The sheet never defines "leftover" as T·U⁻¹, where T is tilt-then-roll and U is roll-then-tilt. I checked that this order gives axis ≈ −z and matches `G-B.between.axis` at 90°. The other order flips the sign. | **Paper:** define the leftover's order and make the ledger match. |
| R-10 | ruth_45 | brief (10), fact sheet 13_6 row vs Finding 8 | minor | The brief and the 13_6 row assert "this arm does have a closed-form solution". Finding 8 says that was not checked from a source and the video makes no claim. For a seven-joint arm "a closed form" means a one-parameter family (the elbow angle), not "the" solution. The narration is clean. | **Paper:** delete the clause from the brief and the row, or cite and state it as a family. |
| R-11 | ruth_45 | 07_5, 14_4 | minor | Edge-test coverage:<br>(a) `G-X3` (264) uses turns of 30° to 150° and no half turn, though 180° is where the axis and logarithm are delicate;<br>(b) "every pose on the way is a true pose" (14_4) has no row in the always/never table. | **Paper:** add 180° and near-0 to `G-X3` and add a row for 14_4. In my own runs half turn, 170°, identity, 2,000 random products and the midpoint det all passed, so nothing is wrong. |
| R-12 | ruth_45 | 02_5 | minor | "Usually" is supported by 120/576 commuting pairs in the 24-element cube group, about 21%. For all turns the truth is "almost every pair": the commuting pairs are exactly same axis (including identity) or two perpendicular half turns. The word is right and the evidence cited is a subgroup. | **Paper:** one sentence in the row. |
| R-13 | ruth_45 | 04_5 | minor | "The space is curved." A circle is intrinsically flat, and a ruler wraps onto it without stretching. What fails is that it closes up, a topological fact (the seam). In SO(3) and SE(3) curvature is real. The exact statement "exact on a circle for steps under a half turn, first order in space" appears only in the fact sheet. | **Picture or description.** Small lower third: "a circle is curved only in how it sits in the plane. The seam is that it closes up". No spoken change. |
| R-14 | ruth_45 | 05_7, 01_6 | minor | "Smooth like this" is not defined on screen (a smooth manifold with smooth multiplication and inverse). "Its flat piece is its Lie algebra" omits the bracket, which only arrives at 15_4. Lie's 1873–74, behind "about a hundred and fifty years old", is nowhere on screen. | **Picture:** the 05_7 name card says "Lie group: a smooth surface whose multiplying and undoing are smooth. Lie algebra: its flat piece, with the bracket (15_4). Lie began this in 1873–74". |
| R-15 | ruth_45 | 15_1 | minor | "A humanoid's whole body … twenty-nine joint angles" reads as general, but it is one robot (the G1 with 29 joints). "Drones steer poses" and "Mapping robots estimate them" have sources in the fact sheet only. | **Words, optional:** "this humanoid's" (keeping protected "twenty-nine joint angles"). **Picture:** the named lower third already planned. **Paper:** sources go in R-04. |
| R-16 | ruth_45 | 01_4 | minor | The screen's crossed "orientations don't" is unqualified. The persona note says "said of these two turns", but the narration is general, and the same-axis exception arrives a chapter later. | **Picture:** small text under the crossed line, "these two turns. Turns about one axis do swap". Words are protected. |
| R-17 | ruth_45 | 03_5 | minor | "A fourth ring would have given one to spare." It is true as redundancy, but a fourth ring is not immunity by itself, since the rings must be driven out of coplanar axes. There is no row in the simplification table. | **Paper:** add a row. No change to the video. |
| R-18 | ruth_45 | 03_6 | minor | "A naming with no bad spot" is exact for nine numbers. For four it is two names per orientation, which the lower third says. The lower third is the longest in the video, so it needs a hold long enough to read. | **Picture:** hold, or split the lower third. |
| R-19 | ruth_45 | 08_6 | minor | "Hold each joint's twist for its joint angle" means the unit-speed screw axis held for as many seconds as the angle in radians. The word "twist" is used for any velocity at 07_2. | **Paper or picture:** a half-line "(a unit twist, one radian a second)". No words change. |
| R-20 | ruth_45 | 05_4, 05_5 | delight | A spin about the diagonal at a third of a turn per second sends each edge to the next, and that is exactly the matrix of "roll then tilt". I verified R = [[0,0,1],[1,0,0],[0,1,0]] equals the product. One arrow reproduces a product of two turns, and the callback to the first quiz lands. | Keep. |
| R-21 | ruth_45 | 15_3, 15_4 | delight | The leftover of two one-degree turns is 0.0175°, which is 1° squared in radians. The ratio from 10° to 1° is 99.75, the axis is 0.7° off −z, and y × x = −z. This is a clean, true reason "the Lie bracket measures how much order matters". | Keep. |
| R-22 | ruth_45 | 11_5, 11_6 | delight | The elbow's speed goes like 1/sin(bend) and meets the 75 °/s limit at 21.49°. The wrist's "like the rings" link is exactly right, since gimbal lock is a rank loss of ring angles to orientation, the same notion as the arm's singularity. | Keep. |
| R-23 | ruth_45 | 03_5 and ch. 12 | delight | Collins's "fourth gimbal" and the arm's spare joint are one idea, redundancy, and the script lets the viewer meet it twice. | Keep. |

## Mathematician audit (`ruth_audit.md`), item by item

1. **Quantifiers explicit.**
   - I ran every line and found no unquantified claim beyond R-01 (halfway), R-02 (senses) and R-16.
   - The two "usually" lines (02_5, 11_1) are measure-one statements and true. 11_1 holds for generic joint angles, which I confirmed at the L shape and found rank 6.
2. **Always and never sentences tested at their edges, test named.**
   - Tested and named: 02_4, 02_5, 03_6, 06_4's lower third, 07_5, 11_3, 12_4, 14_3.
   - Gaps: R-11 (180° in G-X3, and 14_4 has no row) and R-12 (the evidence level of "usually").
3. **Every probability states its space.** No probability appears in the script or fact sheet, so there is nothing to check.
4. **No probability on a fixed unknown, no likelihood as probability.** No frequentist or Bayesian language appears. Nothing to report.
5. **Terms in their standard sense.**
   - Checked: turn, group, ring (gimbal ring, never the algebraic ring), Lie group, Lie algebra, twist, screw, Jacobian, singularity, rank as "directions", exponential, pseudoinverse.
   - "Screw" is used for the finite displacement, as on the four-word card. "Twist" at 08_6 means the unit twist (R-19).
   - Singularity is used only for the Jacobian rank drop, and gimbal lock is linked to it correctly at 11_6.
6. **Every simplification has its exact version recorded.**
   - Recorded: 02_4, 03_6, 05_2, 06_4, 07_5, 10_3, 14_3, 14_5.
   - Not on screen or in the description: R-01, R-03, R-13, R-14, R-17.
7. **Every historical claim verified (dates, attributions, quotes).**
   - Consistent with the sheet's sources: Euler 1775 (printed 1776), Rodrigues 1840, Chasles 1830, Mozzi 1763, Brockett 1983, Lie 1842–1899, and the Collins quote verbatim from the transcript.
   - The sheet records that the Brockett, Mozzi and Chasles texts were not read, and that the Rodrigues and Chasles page text was read from a scan text layer.
   - The wording is suitably hedged ("presented", "described such an axis", "named after"). I did not verify any citation text myself.
   - The recorded provenance is missing on screen and in the description (R-04, R-14).
8. **Every number equals the oracle at the shown precision; voice and screen round the same way.** All checked numbers agree. 13_6 was the only borderline rounding: 0.0264 mm is "three hundredths" and 504.99 mm is "50 cm", both consistently on voice and screen.
9. **Edge cases acknowledged.**
   - Handled:
     - half turn has two arrows (05_2 lower third);
     - pure slide (07_4);
     - zero turn (fact sheet);
     - a rank-5 straight wrist (fact sheet only, R-03);
     - Newton from a poor start (13_6 tag);
     - blend at a = 180° gives det 0.
   - Not shown: opposite headings (R-01).
   - 06_1's "three numbers for its pose" uses a heading angle that has the same seam as ch. 04. That is fine as a count of freedoms and I have no finding.
10. **Notation consistent.** R-08 and R-09. Otherwise t (angle), T (pose), M, S, J, V and q̇ are used consistently, with L&P's bracket and spin first.
11. **Nothing true-but-misleading.**
    - Hedged, and I found no further misleading lines: "From here, three", "would lose a direction here", "usually, seven joints cover all six", "counted the way a roboticist counts".
    - Misleading as written: R-02 (facings), R-03 (covers), and R-05 ("both traps").
12. **Every persona note reached the video.**
    - **Reached:** the lines-fixed-on-the-table note, the Euler lower third, associativity, the bar's "own measure", Apollo as the computer's thresholds, the exponential's definition, "model" labelling, the twist's slide arrow, tip = flange (09_1), the Jacobian's slide-rows tag, the singularity inside the reach, the Newton tag, "steady" on 14_5, and the swelling readout.
    - **Not reached (or reached only in the fact sheet):** shorter arc (R-01); "said of these two turns" (R-16); the 576/120 evidence (R-12); smoothness (R-14); the limit's source (R-06); the 04_5 exactness statement (R-13); the Collins transcript reference, wheel senses, 15_1 sources and Exercise 2.2 credit (R-04).

## What I recomputed (`python3 -I`, own code)

| Quantity | Result | vs fact sheet |
|---|---|---|
| Book: tilt-then-roll and roll-then-tilt R, angle 120° and axes (1,1,−1)/√3 and (1,1,1)/√3; covers (0,−1,0) and (1,0,0) | as listed | match |
| Own-edge turning exchanges the two outcomes; all four sense combinations | 1 of 4 gives the stated facings | match; R-02 |
| Both spins at once: 127.2792° about (1,1,0)/√2 | as listed | match |
| Diagonal spin 120°/s for 1 s equals roll-then-tilt; each edge to the next | R = [[0,0,1],[1,0,0],[0,1,0]] | match |
| Leftover at 1°, 10°, 90°: 0.0175°, 1.7409°, 120°; axes; ratio 99.75; 0.707° and 7.05° off −z | as listed | match |
| Cube group: 24 turns, 120/576 commuting ordered pairs; half turns about perpendicular axes commute | as listed | match |
| 2,000 random products of two turns: orthogonal, det 1, eigenvalue 1 | all pass | consistent with Euler |
| Gimbal ZYX: lock difference 0; (30,90,0) and (0,90,−30) same; volume cos 70°, cos 85°; pair (0,89,0) and (90,89,90) | 0; 0.342; 0.0872; 1.4142° | match |
| All 12 ring orders lose volume | 90°/270° (Tait–Bryan), 0°/180° (proper Euler) | match |
| Headings: mean 0, gap 20, halfway 180 | as listed | match |
| Platform: radius 0.9549, end (0, 1.9099), heading 180°, 6/π, arc 3, spins 0/30/60 | as listed | match |
| Toss: 90°, axis z, point (0,1), slide 0.5; half turn, 170°, identity rebuilt | as listed | match |
| Arm from offsets plus axis table: heights, total 1.306, tips for every posed row | as listed | match |
| Ranks: home 3, stretch 4, elbow-straight 5, elbow-only 5, L 6, L wrist turned 5; without joint k at L: 5,5,5,5,6,5,6 | as listed | match, and one more: dropping any of joints 1 to 3 gives 5 |
| Lost direction at elbow-straight: left null vector equals (p_w × u, u) | parallel | confirms "slide along the arm" |
| Levers at L: 0.42, 0.6731, 0.526, 0.526, 0, 0.126, 0 | as listed | match |
| Elbow circle 0.2897 (triangle 0.42, 0.40, 0.58); push along arm 0.2897 | as listed | match |
| Blow-up 30/10/3/1°: 54.03, 160.44, 534.17, 1,602.34 °/s; bend at limit 21.49°; 4.45 turns/s | as listed | match |
| Newton from (0,30,0,−60,0,90,0) to the goal: 504.99, 93.93, 4.15, 0.03 mm; 90, 8.64, 0.724, 0.0076°; tolerances met after 3 steps; final q within limits | as listed | match |
| Screw between poses: 90° about z, slide −0.3477, point (0.355, 0.045), midpoint 45° and det 1, max gap 0.0758 | as listed | match |
| Blend det: ½, ¼ and 1,000 random matched formula; nearest turn at ¼ = 18.4349°; arrow 0.7071 | as listed | match |
| Drift: (1+10⁻⁴)¹⁰⁰⁰ = 1.1052, width 1.0513, simulated 1,000 steps | as listed | match |

## Unchecked, not claimed as fine

- The counts 192/192, 264/264, 66/66 and 45/45: I tested the underlying statements on my own cases, not their grids.
- All KUKA and KMR data-sheet values (limits, speeds, reach, payload, 3.6 km/h).
- The joint rpy values in the xacro, and Unitree's 29 joints (the 35 and 36 counts are arithmetic only).
- The Comanche 055 listing quote and page, NASA's transcript and its page and time stamp, MacTutor's 1873–74 quote, Euler's Latin, the Rodrigues and Chasles page text, and the L&P equation and page numbers.
- Brockett's record details, and the pronunciations.
