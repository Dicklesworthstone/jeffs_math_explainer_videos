# R2 review, dev_28 (verbatim)

*Saved by the producer from the reviewer's returned text, with the reviewer's scratch folder written as `<scratch>` and one private path written as `<cmaes_explainer>`; nothing else was edited.*

---

# Review R2 (script), lie_robotics, persona Dev, 28 (dev_28)

I read the persona prompt, the whole packet (brief, script, storyboard, fact sheet, every fact row) and `explainer_corpus/lie_robotics/docs/protected.tsv`. I read nothing else under `docs/reviews/`. I edited nothing in the repository. My scratch scripts are in `<scratch>`.

**Verdict as Dev.** I can explain this to a colleague. I can write a toy for chapters 1 to 9 and 12 (Rodrigues, rotate-about-a-line, product of exponentials, swing circle). I cannot write chapter 10's Jacobian columns or chapter 13's loop from what is on screen, and the one code panel in the video mixes two frames in a way that does not converge when I run it (D-01). Every number I recomputed matched. Nothing is a blocker. Four majors would change the video, all through tags, panels, lower thirds or the missing description, and none needs a re-spoken line.

I have no finding against a protected phrase. Where a finding sits next to one I say so.

## Comprehension check

| ch | the sentence I would say to a friend |
|---|---|
| 01 | Two quarter turns of a book land differently depending on order while two slides do not, and a robot arm has to put its gripper at a position plus an orientation, so its software must compute with things that refuse to add. |
| 02 | Any two turns compose to one turn about one axis (Euler), turns can be undone so they form a group, and in code a turn is a 3 by 3 grid whose product is the composition, with a specific order. |
| 03 | Three nested angles cannot name every orientation without a bad spot: at a quarter turn of the middle ring two rings do the same job, so you lose a direction (Apollo's 70° and 85° thresholds), which is why software keeps nine numbers or four. |
| 04 | Averaging headings breaks at the ±180° seam (0 instead of 180), so measure the gap on a flat line touching the circle at the first heading, halve it there, and wrap back: log, arithmetic, exp. |
| 05 | A spin is an arrow (axis times speed) that adds like a vector, holding it for a second and mapping it to a rotation is the exponential (Rodrigues is the code), and a smooth group like this is a Lie group whose flat piece is its Lie algebra. |
| 06 | Adding velocity times time to a pose says 3 m ahead, the exponential says a half circle of radius v over spin, 1.91 m to the left and facing back. |
| 07 | A pose changes in six independent ways, a velocity in space is a twist (spin and slide), holding one gives a screw, and every rigid displacement is one screw. |
| 08 | Each joint is a screw about its own axis, written down once with the arm at zero, and the gripper's pose is the product of seven exponentials applied to the zero pose, last joint first. |
| 09 | By hand with real lengths: elbow a quarter turn then shoulder a quarter turn puts the tip at (42, 0, 88.6) cm, joint one then swings it to (0, 42, 88.6), and no screw ever asks where an axis went. |
| 10 | Velocities live on the flat piece so they add: each joint gives a twist column, the columns stack into a 6 by 7 Jacobian from joint speeds to the gripper's twist, and the lever from the axis decides which joint moves the tip most (joint 2, 67 cm). |
| 11 | When the Jacobian drops a direction (elbow straight, 5 of 6) moving that way costs huge joint speeds (54°/s at 30° bend, 160°/s at 10°, past the 75°/s limit at 21.5°), and this arm's spare joint covers the wrist's alignment. |
| 12 | Seven joints for six directions leaves a one-parameter family of joint angles for the same gripper pose: the elbow swings on a 29 cm circle while the gripper does not move. |
| 13 | Inverse kinematics is Newton on the group: compute the screw from the current pose to the goal as an error twist, pseudo-invert the Jacobian, step, repeat, and from this start it takes 3 steps (505, 94, 4, 0.03 mm). |
| 14 | Averaging two rotation matrices leaves the rotations (determinant 0.5), re-normalising fixes the shape but not the timing, and the screw path is a true pose all the way and differs from the tip's straight line by 7.6 cm. |
| 15 | The same exp-and-log arithmetic runs drones, mapping and physics engines and the humanoid's floating base, and the leftover of two small turns is itself a second-order turn about the third axis, whose rule on the flat piece is the Lie bracket. |

## Rubric (1 to 5)

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 5 | 4 | 5 | 5 | 4 |
| 02 | 4 | 5 | 4 | 4 | 3 |
| 03 | 4 | 5 | 5 | 3 | 5 |
| 04 | 4 | 5 | 5 | 4 | 4 |
| 05 | 3 | 5 | 4 | 3 | 3 |
| 06 | 4 | 5 | 4 | 4 | 4 |
| 07 | 4 | 4 | 4 | 4 | 4 |
| 08 | 4 | 5 | 5 | 3 | 4 |
| 09 | 5 | 5 | 4 | 4 | 4 |
| 10 | 3 | 4 | 4 | 3 | 3 |
| 11 | 4 | 5 | 5 | 3 | 5 |
| 12 | 5 | 5 | 5 | 5 | 5 |
| 13 | 3 | 4 | 5 | 3 | 4 |
| 14 | 4 | 5 | 5 | 4 | 4 |
| 15 | 3 | 4 | 4 | 3 | 4 |

The 4s for correctness mark chapters with a finding below. No number is wrong.

## Findings

Findings that would change the video come first. "Words" says whether a spoken line must change. Unless stated, no re-speak is needed.

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me | suggested fix |
|---|---|---|---|---|---|
| D-01 | dev_28 | 13_4 panel (with 13_3, 10_3) | major | The panel steps with `pinv(J_body(q))`, but the only Jacobian I was ever shown is chapter 10's `J(q)` (space form, slide equals the velocity of the room's origin). "Error read in the gripper's frame" plus chapter 10's J is what I would type, and it does not work. I rebuilt the arm from the packet's axes and heights and ran exactly that mix from the 13_6 start. It wandered for 12 iterations with the tip 0.34 to 1.13 m from the goal. With matching frames I get 505, 94, 4.15, 0.03 mm in 3 steps, as the table says. Nothing on screen says there are two Jacobians. | Panel and tag only, words unchanged (13_3 and 13_4 name no frame). Cheapest is the space form so chapter 10's grid is the one used: `error = log(T_goal · inverse(T(q)))` with the tag "a twist, read in the room's frame", then `q = q + pinv(J(q)) · error`. I ran it and the iterates are identical to the body form (same min-norm step of the same equation). If the owner keeps the body form, J_body needs a defining line, which needs the adjoint, which the brief puts out of scope. Ruth should confirm the equivalence against Lynch and Park §6.2.2. |
| D-02 | dev_28 | whole video, "the description" | major | `docs/description.md` does not exist. The storyboard routes these to it, so today they are recorded nowhere a viewer can reach: the wheel senses `G-P.wheels.*` (06 Dev note), "the tip is the flange" (09 Ruth note), the 15_1 sources (manif, Pinocchio, Lee et al., GTSAM). I would also want these there: the Apollo sources behind 03_4 (Comanche 055 listing p. 147 for 70° and 85°, and the "had to be realigned" half, which in the fact sheet rests on an ALSJ page about the lunar module), KUKA's data sheet behind 75°/s and 820 mm, the Lynch and Park equation numbers, the xacro licence note, and a source for 14_5's "common in practice" (the packet's only source is the textbook's decoupled path). | Write the description before narration is final. No script change. List the sources by the lines that cite them (03_4, 03_5, 06_4, 11_5, 14_5, 15_1). |
| D-03 | dev_28 | 11_3, 13_6, 14_5 | major | The video gives me outputs but not inputs, so I cannot test its three headline claims. Not on screen: the Newton start q = (0, 30, 0, −60, 0, 90, 0)° and goal (0.400, 0.300, 0.250) m with R = [0,−1,0; −1,0,0; 0,0,−1] (needed for "3 steps" and the 7.6 cm), and the elbow-straight pose q = (0, 60, 0, 0, 0, 60, 0)° (needed for "5 of 6"). The L shape is derivable from 09, so that one is fine. | Put start q and the goal pose in the 13_6 "from a poor start..." tag, and the elbow-straight q beside the "directions" readout at 11_3. Picture only. The description can repeat them. |
| D-04 | dev_28 | 10_3, 13_3 | major | Two operations I need are named but not given. First, chapter 10 says each column is the joint's twist "where the axis is right now" but never how to build the six numbers. Only the semantics tag is there, and the adjoint is out of scope. Second, 13_3's "the logarithm" is a lower third with no formula and no pointer, and the whole of chapter 13 rests on it. Without them I cannot implement chapters 10 and 13, which is the contract. | At 10_3 add one tag line that needs no adjoint: "column i = (ω, −ω × c), where ω is joint i's axis direction and c any point on it, both read off the arm's present pose". I checked it: joint 2 of the L shape gives (0, 1, 0, −0.36, 0, 0), matching `G-J.ell.col2`. At 13_3 extend the lower third with "Lynch and Park eq. 3.91 and 3.92". |
| D-05 | dev_28 | 13_2 | minor | "Subtracting angles walks into both traps" is shown for one. (0, 89, 0) against (90, 89, 90) with differences 90, 0, 90 is the near-lock trap of chapter 03. No seam (wrap) trap is on screen. I recomputed the 1.414° and it holds. | Picture: add a second small pair for the seam, for example headings 179° and −179° read as "differ by 358°, are 2° apart". If the picture cannot change, the words need "walks into the traps" in place of "both traps" (one re-speak). |
| D-06 | dev_28 | 05_6, 06_4, 13_3 tags | minor | Three formulas I am handed divide by something that goes to zero. Rodrigues with a unit axis: zero spin gives axis = w / norm(w) = NaN, which I got in numpy. The platform's `radius = speed ÷ spin`, whose own slider visits spin 0. The pose logarithm's 1 / sin θ at θ near 0 and near π. | Tag text only. 05_6: "t near 0: use I + t[ω]". 06_4: "spin 0: a straight line, no radius". 13_3: "near 0 and near a half turn the formula needs its own branch". |
| D-07 | dev_28 | 04_4 tag | minor | "Bring the difference into −180…180" is the classic bug site. In C, C++ and Java `%` and `fmod` keep the sign: I ran fmod(−340 + 180, 360) − 180 and got −340, unchanged, where Python's `%` gives 20. Exactly 180° apart there are two halfways. Ruth's note says "unique here" but the tag does not say what happens otherwise. | Tag text. Add `atan2(sin d, cos d)` as the one-liner that works in every language, and "exactly opposite: two halfways, pick one". |
| D-08 | dev_28 | 15_1 | minor | The demonstration swells a body by adding `dt·[ω]R` to the nine numbers. I ran it and got 1.1052 and 1.0513, so the numbers are right. But no engine I know of ships that. The shipped fix is a quaternion plus renormalise, which also stays rigid and is only first-order in timing (chapter 14's own "repairs the shape, not the timing"). "So the body stays rigid" reads as if only the exponential does that. | Tag text. Name the failure "a forward Euler step on the matrix" and add "a quaternion renormalised each step also stays rigid; the exponential is exact for a constant spin". Words stay: they say "can step ... so the body stays rigid", which is true. |
| D-09 | dev_28 | 15_1 tag | minor | "FrankenSim steps bodies by the exponential (fs-time, se3_exp_step)" is a claim about a system I cannot look up. Quoted as module header, but a viewer has no repository. | Put a link in the description (D-02). If it is not public by release, name a public engine instead. From memory, Bullet and MuJoCo both integrate orientation by an axis-angle exponential, but I did not verify that here and Ruth should before it is shown. |
| D-10 | dev_28 | 06_4 | minor | "Add six seconds of velocity onto the pose" is a one-big-step straw man to a programmer. I ran forward Euler on the platform with heading updated each step: dt 0.1 gives (0.05, 1.9094), dt 0.01 gives (0.005, 1.9099), against the exact (0, 1.9099). I would try that in two minutes and conclude the video exaggerates. | Tag text. Add "in tiny steps x += v cos θ dt gets within a centimeter (0.01 s steps land within 5 mm); the exponential gets it exactly in one step of any size". |
| D-11 | dev_28 | 04_4, 05_3 | minor | Two storyboard Dev notes did not land. 05's "three numbers in, nine out; the logarithm goes back" appears in no picture of chapter 05, and the logarithm is not named until 13_3. Chapter 04 does the measuring step without calling it by its name. | Lower thirds only. 04_4: "measuring the gap: the logarithm". 05_3: "exp: 3 numbers in, a 3 by 3 grid out; the logarithm goes back". |
| D-12 | dev_28 | 14_3, 14_4 | minor | The programmer's names are missing. The blend is a lerp, the repaired blend is nlerp (hence the 18.4° against 22.5° timing gap in the tag), and the screw path's rotation part is slerp. The 18.4° and 22.5° I recomputed. | Lower thirds, no re-speak: "blend: lerp", "re-normalised: nlerp", "the screw path: slerp for the turn (its slide and turn together: ScLERP)". |
| D-13 | dev_28 | 10_3 tag | minor | A toy validated against a library will disagree. This video lists spin first and gives the slide as the room origin's velocity. From memory (not checked here), Pinocchio and KDL list linear first and give the tip's own velocity. The tag fixes the semantics but not the mismatch. | Add to the tag "libraries often list the slide first and give the tip's own velocity, v + ω × p". |
| D-14 | dev_28 | 11_1, 11_3, 11_4 | minor | "Directions" is rank, and "rank" is said nowhere (words or lower third). To test I want the number. At the elbow-straight pose the singular values (this video's units) are 2.11, 1.82, 1.35, 0.50, 0.36, 0.000: one is zero, the next is 0.36. | Lower third at 11_1: "directions = the rank of the Jacobian". Optionally a "smallest singular value" gauge on the 11_4 speed gauge. Words unchanged. |
| D-15 | dev_28 | 11_6 | minor | "Straighten it, and two axes line up, like the rings." Chapter 03's rings (z, y, x) lock with the middle ring a quarter turn. The wrist (z, y, z) locks at the middle joint zero. Same disease, different bad angle. A viewer who remembers 90° will be briefly lost. `G-X6.bad_angle` has both. | Lower third at 11_6: "this wrist's axes go z, y, z, so its bad spot is the middle joint at zero; the rings' z, y, x had theirs at a quarter turn". Words stay (the sentence about two joints at the shoulder is protected). |
| D-16 | dev_28 | 11_5 | minor | The 75°/s line is KUKA's rated-payload speed for axis 4 (the storyboard's own Dev note), but the picture does not say so. A reader will take it for a physical wall. | Label the limit line "KUKA data sheet, at rated payload". "From the geometry alone" already covers the rest. |
| D-17 | dev_28 | 01_6, 08_6 | minor | "About a hundred and fifty years old" (Lie, 1873) then "Roger Brockett presented it in nineteen eighty-three" read as two dates for the same "it". The fact sheet is consistent (the ideas are Lie's, the arm's line is Brockett's) but the viewer is not told. | Lower third at 08_6: "the ideas: Lie, 1870s; writing an arm this way: Brockett, 1983". Both spoken sentences stay (the second is protected). |
| D-18 | dev_28 | 03_5 | minor | "Radioed a request" turns a joke into a work order. The fact sheet records the capcom answering "You were unreadable. Say again please." and Collins saying "Disregard." That exchange is the better beat for a programmer. | Quote card: add the capcom's reply and "Disregard" under the quote. Optionally "radioed a joke" for "radioed a request" (one re-speak). "Michael Collins" and the quote are protected and unchanged. |
| D-19 | dev_28 | 01_1 | minor | Someone at a desk with a real book will tilt it one way, and the cover will face away instead of "facing you". The answer "different" survives, "facing you" does not. The sense of each quarter turn is only in the picture. | Lower third at 01_1: "tilt: far edge up; roll: toward the right". Words stay ("about one line on the table" is protected and untouched). |
| D-20 | dev_28 | 05_6, 10_3, 13_4, 11_2 | minor | Pacing for someone reading a formula or code. 05_6 has a spoken sentence, a TeX formula, a tag and a lower third in one beat with no hold. 13_4's panel must be read in about 10 s and then the quiz card replaces the attention. 11_2 gives 4 s for a rank question I cannot reason about from the picture. | Add a hold on 05_6. Keep 13_4's panel visible beside 13_5's quiz card (the card is beside the picture anyway). Take 11_2's countdown from 4 to 6 s (a timing parameter, not narration). |
| D-21 | dev_28 | fact sheet (not the video) | minor | The sheet contradicts itself. Finding 8 says closed-form IK for this arm "was not checked from a source" and was removed from 13_6's tag. But the "Simplification" row for 13_6 and the brief's item (10) state "a closed-form solution exists for this arm". The narration says neither, so the video is unaffected. | Make the three places agree. I believe it is true (spherical shoulder and wrist, one redundancy parameter, the arm-angle family; Shimizu et al., T-RO 2008, recalled from memory and not verified here), so cite it, or soften the two sentences. |
| D-22 | dev_28 | 12_5 | minor, delight | The storyboard's Dev note asks "what the freedom is used for in practice". Only the obstacle is shown. Also the fourth ring of 03_5 ("one to spare") pays off in ch 12 but is never called back. A programmer would love that link. | Lower third at 12_5: "used for obstacles, joint limits and staying away from singularities". Flash the four-ring gimbal for a beat at 12_1 beside "7 − 6 = 1". Picture only. |
| D-23 | dev_28 | length (brief row, fact sheet 11) | minor | My vote on cuts, as the packet asks Dev to decide which mechanisms and costs stay. Keep 03_4: it is the only real-system cost of the lock and carries the 70° and 85° numbers I would test against. Keep 14_5: it is the "what do planners do" I asked for. If time must go, take 03_5 before 03_4. The cost of that cut is D-22's callback. | Keep 03_4, 11_5 and 14_5. |
| D-24 | dev_28 | 04_3 | delight | The robot told to go from one heading to the other by the numbers and spinning the long way round is the bug in miniature. | Keep. |
| D-25 | dev_28 | 08_1 panel | delight | The model as data (axis directions, heights, the zero pose M) is exactly what I need to write the loop. With the 08_2 tag (c + R(x − c)) every exponential is computable. | Keep, and make sure the panel stays up long enough to copy. |
| D-26 | dev_28 | 11_5 | delight | The blow-up curve with a limit line crossing at 21.5° and "four and a half turns a second" at 1°. Every number held up: 54.03, 160.44, 534.17, 1,602.34, 21.49, 4.45. | Keep. |
| D-27 | dev_28 | 13_2 | delight | Orientations 1.4° apart with ring angles 90° off is the bug report I would have filed. | Keep (see D-05 for the second trap). |
| D-28 | dev_28 | 14_3 tag | delight | "Re-normalising repairs the shape, not the timing: 18.4°, not 22.5°" is the nlerp caveat every game programmer needs and almost nobody says. | Keep. |

## Line-by-line audit of the "for programmers" tags and the panel

| line | what it says | does it do what the voice says |
|---|---|---|
| 02_6 | `R = roll · tilt`, columns are where the edges point | Yes. Verified: Ry·Rx reproduces tilt-then-roll about the room's lines. The spoken line "one grid times another" is order-free, so the tag carries it. |
| 04_4 | wrap the difference into −180…180 | Right idea, see D-07. |
| 05_6 | `e^{[ω]t} = I + sin t [ω] + (1 − cos t)[ω]²`, `[ω]p = ω × p` | Correct for a unit ω. Divide-by-zero in the axis, see D-06. |
| 06_4 | `radius = speed ÷ spin`, 0.5 ÷ 0.524 = 0.95 m | Correct. Spin zero, see D-06. |
| 08_1 and 08_2 | axes and heights, M at zero, `c + R(x − c)` | Correct and sufficient for a toy. |
| 10_3 | `V = J(q) q̇`, tip velocity = slide + spin × tip position | Correct (my levers match `G-J.ell.lever.*`). Columns not buildable, see D-04. |
| 13_4 | the three-line loop | Mixes frames, see D-01. `error < 0.001 rad and 0.1 mm` compares a 6-vector with two scalars: say the spin part's norm and the slide part's norm. The slide part of the log is not exactly the tip's remaining distance, but at the last step they agree (0.026 mm against 0.03 mm). |
| 14_3 | repaired blend at a quarter of the way has turned 18.4° | Verified (`atan2(s sin a, 1 − s + s cos a)`). |
| 15_1 | 1,000 steps of 0.01 s at 1 rad/s, 1.105 and 1.051; velocity 35, state 36 | Verified by running the additive step. 35 and 36 are the usual `nv` and `nq` of a free-flyer plus 29 joints (my recollection, unchecked). The tag could say "nq and nv". |

## What I checked and how

I recomputed with NumPy, in the scratch directory, independent of the oracle:

- The book:
  - both products and their covers: (0, −1, 0) and (1, 0, 0);
  - 120° about (1, 1, −1)/√3 and about (1, 1, 1)/√3;
  - 127.28° about (1, 1, 0)/√2 for the sum of the spins;
  - the leftover at 1°, 10° and 90° (0.0175, 1.7409, 120.0), and the axis (−0.0087, 0.0087, −0.9999), which fixes the convention behind the lower third's `[y, x] = −z`.
- The platform: end (0, 1.9099), heading 180°, radius 0.955, 6 ÷ π.
- Headings: the gap of 20° and the halfway of 180°.
- Gimbal: the 13_2 pair 1.414° apart, the lock matrices' difference of zero, and cos 70° and cos 85° (0.342, 0.087).
- Blend: determinant 0.5, 18.43°, and the swell numbers 1.1052 and 1.0513 (by actually running the additive step).
- The arm, built from `G-A` axes and heights:
  - home 1.306 m, after the elbow (−0.526, 0, 0.78), the L shape (0.42, 0, 0.886), joint one (0, 0.42, 0.886);
  - the seven levers (0.42, 0.673, 0.526, 0.526, 0, 0.126, 0) and joint 2's column;
  - ranks home 3, straight out 4, elbow only 5, elbow straight 5, L shape 6, 5 without joint 3, 6 without joint 7, wrist turned 5.
- The elbow's rates and limit crossing: 54.03, 160.44, 534.17, 1602.34, 21.49°, 4.45 turns a second; and the swing radius 0.2897.
- Newton from `G-I.start.q` to `G-I.goal`: 504.99, 93.93, 4.15, 0.03 mm and 90, 8.64, 0.72, 0.0076°, in both body and space form, and the divergent mismatch for D-01.
- The screw path: 90°, slide −0.3477, point (0.355, 0.045), halfway det 1.0 and turned 45°, gap 7.58 cm.

**Unchecked.** I cannot see the pictures, so every claim about what a lower third or card shows rests on the storyboard text.

- All history and sources: Euler 1775, Chasles 1830, Mozzi 1763, Lie 1873, Brockett 1983, Rodrigues 1840, the Apollo thresholds and the Collins transcript, and the KUKA data sheet figures.
- Lynch and Park equation numbers and page numbers.
- Anything I said about Pinocchio, KDL, MoveIt, Bullet, MuJoCo, KUKA's LIN motion, or Shimizu et al., which is from memory and marked above.
- The statement that real planners "often" use a straight tip line (the textbook presents it; "common in practice" has no source in the packet).
- The Unitree G1 counts (29 joints, 35 and 36) beyond the packet's own statement.
- Pronunciations.

**Storyboard Dev notes without a home** (beyond D-11): 11's "limit is KUKA's published speed at rated payload" is not in the picture column (D-16). 15's "nq and nv" is not named in the tag (audit table above). 06's wheel senses go to the description, which does not exist (D-02).
