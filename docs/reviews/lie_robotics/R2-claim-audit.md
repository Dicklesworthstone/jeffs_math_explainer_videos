# R2 claim audit (verbatim)

*The claims of the six readers that could change the video, as 31 atomic statements (A-01 to C-11), each given to a reader who had not written it and who reached its status from their own derivation or code. Statuses: VERIFIED, PARTIALLY_SUPPORTED, HYPOTHESIS, REJECTED, NON_MATERIAL. Saved by the producer from the reviewer's returned text, with the reviewer's scratch folder written as `<scratch>` and one private path written as `<cmaes_explainer>`; nothing else was edited.*

---

## Audit A: precision of statements about turns, headings and the arm's rank (claims A-01 to A-10)

# Audit of CLAIMS_A (A-01 to A-10), Video G

## What I ran

All code is mine and sits in `<scratch>/`, run with `python3 -I` and numpy only.

- **`kin.py`** parses `iiwa14.xacro` directly (URDF rpy = Rz·Ry·Rx, each joint then turned about its own z).
  - It reproduces the fact sheet's axes and heights: heights 0.1575, 0.36, 0.5645, 0.78, 0.9645, 1.18, 1.261, flange 1.306.
  - It reproduces the tips: L shape (0.42, 0, 0.886) and the quiz pose (0.8193, 0, 0.7070).
- **Scripts per claim:**
  - `a01.py`: Jacobians, ranks and pushes.
  - `a03.py`: the four turn-sense combinations.
  - `a04.py`: my own Newton solver.
  - `a05.py`, `a05b.py`: ranks and random straight-wrist poses.
  - `a06.py`: ring-angle pair.
  - `a08.py`: leftover axes.
  - `a09.py`: commuting-pair counts.
  - `a10.py`, `a10b.py`: screw and path edge cases.
- For definitions only, I read `oracle.py`, `model.rs` and the chapter files in `src/chapters/`. I did not read `docs/reviews/**`.

---

## A-01 (11_3 lower third "the gripper can only swing sideways")

STATUS: VERIFIED

WHY:
- At q = (0,60,0,0,0,60,0) the rank of J is 5 (singular values 2.11, 1.82, 1.35, 0.50, 0.36, 0).
- The left null vector is a pure force along the shoulder-to-wrist line through the wrist centre.
- The wrist centre is the point where axes 5, 6 and 7 meet, 0.82 m from the shoulder. Its distance to each of those axes is 0.
- The wrist-centre velocity Jacobian has rank 2, and its column space is exactly the plane perpendicular to the arm line u = (0.866, 0, 0.5). The component along u is 0 for all seven joints.
- A pure slide of the whole gripper along u cannot be reached (residual 0.95). Pure slides along either perpendicular direction can (residual about 1e-16).
- The tip (flange origin) is r = 0.126 m past the wrist centre, 60° off the line.
  - Joints 2, 4 and 6 each push the tip along u by 0.109 m/s per rad/s (0.126·sin 60°).
  - The tip's position Jacobian has rank 3.
  - A tip velocity of 1 m/s along u is reachable, at the cost of 9.16 rad/s of gripper turning.
- With the wrist straight (q6 = 0) the tip lies on the line (distance 1e-16) and its push along u is 0 (rank 4 there).
- So the spoken "can't slide ... can't begin to slide straight back" is exact, and the fact-sheet row 11_3 ("cannot translate along it without turning") is exact. Only the lower third, taken about the tip, is imprecise.

FIX: Storyboard row 11_3 lower third (and the `lost_directions.rs` string) becomes:
"at first, the wrist can only swing sideways"

FIX CHECK:
- True: the wrist centre's velocity space is the plane across the arm, rank 2.
- Not spoken, and no protected phrase touched.
- No new double meaning: "the wrist" in 11_3 ("push on the wrist") and 11_6 ("three joints whose axes meet at one point") already names that point.

---

## A-02 (04_3 "true halfway" and the convention)

STATUS: PARTIALLY_SUPPORTED

WHY:
- The two midpoints of 170 and −170 are 0 (middle of the 340° way) and 180 (middle of the 20° way).
- The claim that the convention is "on screen nowhere" is overstated.
  - Picture: the tick sits between the two arrows, and the step reads 20°.
  - Code panel as built in `halfway.rs`: "d = wrap(b - a) / # into -180 ... 180 / # so the step is short". The storyboard tag says "bring the difference into −180…180: −340 becomes +20".
  - What is missing is a plain-words statement.
- "True" is not just a convention.
  - The mean of the two unit heading vectors points to 180.
  - 180 is 10° from each heading, while 0 is 170° from each. So 0 is the point farthest from both.
- The objection is therefore weaker than claimed: 0 answers a question nobody asked. A plain-words rule still serves "Ruth" and brief item 5.

CORRECTED STATEMENT: The convention (shorter way round) is on screen only in code form and in the picture, not in plain words. "True" is well founded: 180 is the vector mean and the nearest point to both headings.

FIX: Storyboard 04_4 "for programmers" tag becomes "bring the difference into −180…180, the shorter way round: −340 becomes +20". Optional matching change in `halfway.rs`: "# so the step is the shorter way round". Spoken words and protected phrases are unchanged.

FIX CHECK:
- True: b − a = −340 becomes +20, which is the shorter way.
- A tag, so not spoken.
- Protected "The true halfway is a hundred and eighty" is untouched.
- No new double meaning.

---

## A-03 (01_2 facings versus the senses of the quarter turns)

STATUS: VERIFIED

WHY:
- Setup: x right, y away, z up; cover starts up; turns about room lines (later turn on the left); tilt = Rx(±90), roll = Ry(±90).

| Tilt | Roll | Tilt first, cover faces | Roll first, cover faces |
|---|---|---|---|
| far edge up | to the right | toward you | right wall |
| far edge up | to the left | toward you | left wall |
| far edge down | to the right | far wall | right wall |
| far edge down | to the left | far wall | left wall |

- "Facing you" depends only on the tilt sense, and the right wall only on the roll sense.
- "Different" holds in all four combinations (Rx·Ry ≠ Ry·Rx).
- The existing lower third is exact: about the book's own edges, tilt-then-roll faces the right wall and roll-then-tilt faces you, so the answers trade places.
- The spoken 01_1 gives no sense. The storyboard picture row does ("far edge up", "to the right"), but only as animation. The cold open invites trying it on a real book, where either sense is natural.

FIX: Storyboard 01_2 lower third replaced by:
"tilt: far edge up. roll: cover to the right. Turn either the other way, or about the book's own edges, and a cover faces another wall: still different"

Optional picture addition at 01_1: a small curled arrow on each gold table line showing its sense.

FIX CHECK:
- True: in every alternative (tilt reversed, roll reversed, own edges) at least one cover faces a different wall, and the answer is still different.
- Protected "Tilt first, and the cover ends up facing you" and "about one line on the table" are unchanged.
- No new double meaning.

---

## A-04 (13_6 "each step gains more than the one before")

STATUS: VERIFIED

WHY:
- My own Newton solver (body Jacobian, pseudoinverse, error log(T⁻¹T_goal)) reproduces tip-to-goal 504.99, 93.93, 4.15, 0.0264 mm and error spin 90, 8.640, 0.724, 0.0076°. Final q matches the fact sheet's.
- Absolute gains are 411.1, 89.8, 4.1 mm and 81.4°, 7.9°, 0.72°, so they decrease.
- Factors are 5.38, 22.6, 157 for distance and 10.4, 11.9, 95.6 for angle, so they increase.
- Digits gained are 0.73, 1.35, 2.20 for distance and 1.02, 1.08, 1.98 for angle, so they increase.
- Read plainly, the sentence is false. The chapter test (`steps.rs` 606 to 608) asserts the ratio increases, so the intent was the factor.

CORRECTED STATEMENT: Each step shrinks the error by a bigger factor than the one before, while the absolute gain falls.

FIX: Storyboard 13_6 table text (and `EACH_GAINS` in `steps.rs`) becomes:
"each step divides the error by a bigger number than the one before"
("cuts the error by a bigger factor" is equally true.)

FIX CHECK:
- True for distance and angle, and for the rounded table values (5.6, 22.5, 133 and 10.5, 12.3, 70).
- On-screen text, no protected phrase.
- "Number" avoids "factor", which the video also uses for the seven-factor line.

---

## A-05 (11_6 "this arm has a third, and it covers")

STATUS: NON_MATERIAL (the numbers are VERIFIED; the proposed lower third is not needed)

WHY:
- At (0,90,0,90,0,0,0) the rank is 6 with the tip at (0.42, 0, 0.886).
  - Without joint 1, 2 or 3, or without joint 4 or 6, the rank is 5.
  - Without joint 5 or 7 it is 6.
  - So "without any one of joints 1 to 3 is 5" and "without joint 7 is 6" are both true. Joint 5 is equally the duplicate.
- At (0,90,0,90,90,0,0) the rank is 5, as fact-sheet finding 9 says. Along (0,90,0,90,q5,0,0) the sixth singular value is 0.2321 at q5 = 0, 0.0036 at 89° and 0 at 90°.
- With q6 = 0 and the other joints random within the limits (20,000 poses), the rank is 6 in 19,991. The 9 exceptions were all within 0.5° of q4 = 0, where the elbow is straight.
- A six-joint arm (one of joints 1 to 3 removed) with a straight wrist: 0 of 15,000 poses had rank 6.
- So "covers" is the generic truth for a straight wrist, and the spoken "here" scopes it to the pose shown.
- Exceptions are thin sets, such as the quarter-turned joint 5 in the L shape. They are already recorded in fact-sheet finding 9, row 11_6 and the storyboard "Ruth" note.
- A viewer who generalises is mostly right, so no on-screen fix is needed.

Side finding: fact-sheet finding 9 (line 115) still quotes the narration as "the seventh joint covers for it". The script says "This arm has a third, and it covers". Update that quote.

FIX: NONE on screen. If the owner wants Ruth-level exactness, the optional 11_6 lower third is:
"in this pose. With joint 5 turned a quarter turn, this arm has five directions too"

FIX CHECK:
- True: (0,90,0,90,90,0,0) has rank 5.
- Not spoken, and the protected clause "with two joints at the shoulder, would lose a direction here" is untouched.

---

## A-06 (13_2 "both traps" and the 1.4° figure)

STATUS: VERIFIED

WHY:
- For the drawn nesting (outer Z, middle Y, inner X) the angle between (0,89,0) and (90,89,90) is 1.4142°. The ring angles differ by 90, 0 and 90.
- With the opposite nesting the same triples are 179.99° apart, so the drawn order matters. Fact sheet finding 12 already handles this.
- The pitch scan shows 14.13° at 80°, 7.07° at 85°, 1.414° at 89° and 0.141° at 89.9°. At exactly 90° the two are identical.
- The pair shows only the gimbal trap. No ring angle wraps through ±180.
- `steps.rs` draws only this pair, so the seam trap (04) is not shown although the line says "both".
- The seam example numbers check: 170 and −170 differ by 340 as numbers and are 20° apart. 179 and −179 differ by 358 and are 2° apart.
- The spoken sentence is true as a general statement, so no re-speaking is needed.

FIX: Add to the 13_2 picture, at "walks into both traps", a small callback line using chapter 04's own pair:
"headings 170° and −170°: the numbers differ by 340°, the headings by 20°"
Keep the books and ring angles as they are.

FIX CHECK:
- True.
- On-screen text.
- No protected phrase in 13_2.
- Reuses chapter 04's numbers, so no new double meaning.

---

## A-07 (04_5 "The space is curved")

STATUS: NON_MATERIAL

WHY:
- True: a circle with arc-length metric has zero intrinsic curvature. The picture shows extrinsic bending (the circle bends away from its tangent line), which is exactly what "the flat piece that touches it" needs.
- SO(3) with the bi-invariant metric is genuinely curved: a geodesic sphere of radius r has area 16π·sin²(r/2), so K = 1/4. The general sentence, about the spaces the video then uses, is true.
- A caution on any "exact version" in the description:
  - For poses, SE(2) with the natural metric dx² + dy² + dθ² is intrinsically flat, and SE(3) has no bi-invariant metric.
  - "Curved" for poses really means that steady motions are not straight lines in pose coordinates.
  - An exact description sentence should keep to orientations.
- Neither the spoken line nor the picture misleads at the video's scope.

FIX: NONE. Optionally, one description sentence: "'Curved' here means the space bends away from its flat piece (a circle from its tangent line). The space of orientations is also curved in the strict sense."

FIX CHECK: The optional sentence is true, adds no spoken words, and touches no protected phrase.

---

## A-08 (15_4 sign and ledger versus fact sheet)

STATUS: VERIFIED

WHY:
- With T = tilt then roll = Ry·Rx and U = roll then tilt = Rx·Ry (room lines, later turn on the left):

| Turn size | Quantity | Angle | Axis |
|---|---|---|---|
| 1° | T·U⁻¹ | 0.017453° | (−0.0087, 0.0087, −0.9999) |
| 10° | T·U⁻¹ | 1.7409° | (−0.0868, 0.0868, −0.9924) |
| 90° | T·U⁻¹ | 120° | (−0.5774, 0.5774, −0.5774) |

- These equal the fact sheet's `G-B.leftover.*` and the oracle's `tw @ wt.T`. `model::leftover` is the same.
- The axis is 0.707° off −z at 1° and 7.05° off at 10°.
- 10° against 1° gives a ratio of 99.7, the hundredfold.
- ‖T·U⁻¹ − Rz(−ab)‖ is 2.7e-6 at 1° (squared angle 3.05e-4), against 6.1e-4 for Rz(+ab). This matches the bracket of roll's arrow with tilt's arrow: ŷ × x̂ = −ẑ.
- So the lower third ("about roll's arrow × tilt's arrow, which is −z: [y, x] = −z") is right for T·U⁻¹.
- The reverse direction (U·T⁻¹, or T⁻¹·U) gives +z. The body-frame form U⁻¹·T gives −z at small turns but a different axis, (0.577, −0.577, −0.577), at 90°.
- The fact sheet never defines the direction. Its ledger "On screen" cell and row 15_3/15_4 say "[x, y] = z", while the storyboard and `everywhere.rs` say "[y, x] = −z". Both are true as brackets, but they differ in order and are inconsistent as quotes of the screen.

FIX: Fact sheet only.
1. In the G-B legend or row 15_3/15_4, add: "leftover = T·U⁻¹, the turn that carries the roll-then-tilt book's orientation onto the tilt-then-roll book's, about lines fixed in the room (oracle `tw @ wt.T`, `model::leftover`); for angles a (tilt, about x) and b (roll, about y) it tends to a turn by ab about ŷ × x̂ = −ẑ; read the other way round it is +z."
2. Ledger "On screen" cell: "[y, x] = −z", and in row 15_3/15_4 write "[y, x] = −z (equivalently [x, y] = z for the leftover read the other way)".

Optional: the 15_4 lower third gives x, y, z only by position; "(x: tilt's line, y: roll's, z: up)" would pin them.

FIX CHECK: True per the runs. Doc-only, so no voice rule applies and no protected phrase is touched.

---

## A-09 (usually, and the 120 of 576 pairs)

STATUS: VERIFIED

WHY:
- The cube group has 24 turns and 120 of 576 ordered pairs commute (20.8%). My breakdown is 47 with an identity, 49 on a shared axis line, and 24 perpendicular half-turn pairs.
- A criterion "one is no turn, or same axis line, or both half turns about perpendicular lines" matches the exact set with 0 mismatches. I also tested:
  - the 60-turn icosahedral group: 300 of 3600 (8.3%), 0 mismatches;
  - the 12-turn tetrahedral group: 48 of 144, 0 mismatches;
  - the 12-turn dihedral D6 group: 72 of 144, 0 mismatches.
- Hand cases: half turns about x and y commute. A half turn about x and a quarter turn about y do not. Half turns about x and about (1,1,0) do not.
- For all turns the commuting set has measure zero. Among 200,000 random pairs, a fraction of 5e-6 had |AB − BA| < 1e-3. The typical commutator turn is 120° (5th and 95th percentiles 36° and 175°).
- So "usually" is right and conservative, and the exact set is as proposed.

FIX: Optional one sentence in the fact-sheet row 02_5: "For all turns, A and B swap exactly when one is no turn, or both turn about the same line, or both are half turns about perpendicular lines; every other pair, almost all, does not. The cube's 120 of 576 is a subgroup count of the same rule (the 60 turns of an icosahedron give 300 of 3600)."

FIX CHECK: True per the runs. Doc-only, and the spoken "order usually matters" and the lower third are untouched.

---

## A-10 (edge tests: G-X3 and 14_4)

STATUS: VERIFIED (the statements hold at every edge tried; whether to record them is only a documentation matter)

WHY:
- **Screw decomposition** at 200 random (R, t) per turn size, each reconstructed and checked for uniqueness:

| Turn | Reconstruct error | Axis distance from origin (|t| ≈ 1) |
|---|---|---|
| none (pure slide) | 0 | axis is only a direction |
| 1e-6 rad | 5e-10 | up to 3e6 m |
| 0.001° | 4e-11 | 2e5 m |
| 0.1° | 3e-13 | 1.7e3 m |
| 1° | 4e-14 | 180 m |
| 30° to 150° | ≤ 2e-15 | a few m |
| 179.9° | 2e-15 | a few m |
| 180° | 2e-15 | a few m |

  The small-turn error is conditioning (about eps/θ), not a failure of the theorem.
- **Half turn:**
  - The axis point is t⊥/2, and the slide h·n is unchanged under n → −n with h → −h.
  - Only the sense of the axis is free; line, point and slide are unique.
  - Points on the axis move by exactly h·n, and no other points move parallel to it.
- **Near zero:** the axis still exists, at distance |t⊥| / (2 sin(θ/2)).
- **14_4:** X(s) = exp(s·log(T_end·T_start⁻¹))·T_start was tested for relative turns of none, 1e-6 rad, 0.1°, 90°, 179.9°, 180° and 270°, at 21 values of s each.
  - Orthogonality error is ≤ 2e-15, det − 1 is ≤ 2.4e-15, the bottom row is exact, and the endpoint is hit to ≤ 2.2e-15.
  - At exactly 180° the twist is not unique. The two choices (±axis) both give true poses but different halfway poses (they differ by 1.49 in the matrix entries, for the case run). The video's example is 90°, so "the twist" is right there.
- The video's own `model::log6` ("Not for an exact half turn") and `angle_axis` (returns a zero axis at exactly π), like the oracle's `angle_axis` ("not for an exact half turn"), exclude exactly this case. That is why G-X3 stops at 150°. Adding 180° as a computed key would need a half-turn branch in the model and oracle, for example the axis from (R + I)/2.

FIX: Fact sheet only, as text, with no new key.
1. In the always/never table, row 07_5, append: "Not in the tested set: an exact half turn (model and oracle logarithms exclude it) and turns under 30°. The theorem holds there: at a half turn the axis point is t⊥/2 and only the sense of the axis is free; for a small turn the axis lies far away, at |t⊥| / (2 sin(turn/2))."
2. Add a row 14_4, "every pose on the way is a true pose": "exp(s·log(T_end·T_start⁻¹))·T_start is orthogonal with determinant 1 for all s, including relative turns of none, near zero, near 180° and exactly 180° (there the twist is one of two equal ones, and the video's example is a quarter turn)."

FIX CHECK:
- True per the runs.
- Doc-only: no spoken words change, and the protected "One screw would have carried it there, every time" is untouched.
- The numbers for 264 poses are not altered.

---

## Summary

| Claim | Status |
|---|---|
| A-01 | VERIFIED: change "gripper" to "wrist" in the 11_3 lower third |
| A-02 | PARTIALLY_SUPPORTED: add "the shorter way round" to the 04_4 tag |
| A-03 | VERIFIED: extend the 01_2 lower third with the senses and "another wall" |
| A-04 | VERIFIED: reword to "divides the error by a bigger number" |
| A-05 | NON_MATERIAL: numbers verified; fix finding 9's stale quote only |
| A-06 | VERIFIED: add a 170°/−170° callback line to 13_2 |
| A-07 | NON_MATERIAL: leave as is |
| A-08 | VERIFIED: define leftover = T·U⁻¹ and align the ledger to [y, x] = −z (fact sheet) |
| A-09 | VERIFIED: add one fact-sheet sentence (optional) |
| A-10 | VERIFIED: add two fact-sheet text lines; no new computed key |

No spoken word needs to change for any claim, and no protected phrase is touched by any fix.


---

## Audit B: robotics statements, code panels, sources (claims B-01 to B-10)

# Audit AUD-B: claims B-01 to B-10

All checks were my own code: `arm.py` (an independent arm model built from the xacro's joint origins and rpy), `b01.py`, `b01b.py`, `b01c.py`, `b02.py`, `b03.py`, `b04.py`, `b04b.py`, `b04c.py`, `b05.py`, `b06.py` and `b07.py`, all in the AUD-B scratch directory under `<scratch>/`. I used numpy only.

My model matches the fact sheet where I could compare it:
- Axis heights at zero: 0.1575, 0.36, 0.5645, 0.78, 0.9645, 1.18, 1.261 m.
- Product of exponentials against the URDF chain walk: agree to 7e-16.
- Both Jacobians against finite differences: agree to 4e-10.
- `G-J.ell.col2` and the lever lengths: reproduced.

I did not read `docs/reviews/**`. I read the chapter sources only for the literal on-screen strings of 10_3, 11_4/11_5, 13_4 and 15_1.

Web items:
- **Apollo transcript:** read through a summarising fetch of NASA's technical air-to-ground transcript HTML.
- **KUKA data sheet:** I could not open KUKA's PDF. I confirmed 75 °/s only through search results and third-party listings.
- **Brockett's chapter:** the Springer record redirected to a login.

| Claim | Status |
|---|---|
| B-01 | VERIFIED (one of its two proposed fixes is worse than the other) |
| B-02 | PARTIALLY_SUPPORTED |
| B-03 | PARTIALLY_SUPPORTED |
| B-04 | PARTIALLY_SUPPORTED |
| B-05 | VERIFIED |
| B-06 | PARTIALLY_SUPPORTED |
| B-07 | REJECTED |
| B-08 | PARTIALLY_SUPPORTED |
| B-09 | PARTIALLY_SUPPORTED |
| B-10 | NON_MATERIAL |

---

## B-01 Body Jacobian never defined; frames mixed

**STATUS:** VERIFIED. The proposed fix "define J_body" is right. The alternative "use the space form" is worse.

**WHY:** Start q = (0,30,0,−60,0,90,0)°, tip (0.610, 0, 0.5977), goal p = (0.4, 0.3, 0.25) with R as in the fact sheet. I ran four loops with `q += pinv(J) e`, stopping at 0.001 rad and 0.1 mm.

- **(a) body error log(T⁻¹ T_goal) with J_body:** converges in 3 steps.
  - Tip distance: 504.99, 93.93, 4.154, 0.026 mm. Orientation error: 90°, 8.64°, 0.724°, 0.0076°.
  - Final q = (24.89, 38.57, 15.58, −104.86, −15.56, 38.65, −40.55)°, matching `G-I.ik.*`.
- **(b) space error log(T_goal T⁻¹) with J(q):** also 3 steps. The iterates are identical to (a) to 3e-15 rad.
  - Reason: J_s = Ad_T J_b and e_s = Ad_T e_b, so J dq = e has the same solution set. At full row rank the minimum-norm solution is frame-independent.
  - At a rank-5 pose (elbow straight) the two steps differ by 2.7e-3 rad, because the least-squares norm is frame-dependent.
- **(c) the mixtures:** body error with ch 10's J, and space error with J_body. Neither converges.
  - From the Newton start, the tip is still 0.91 m and 0.58 m away after 60 steps, and not converged after 500.
  - From 60 random starts each within ±1°, ±5°, ±10° and ±30° of the solution, 0 of 60 converge. (a) and (b) converge in 60 of 60 at every spread, in a median of 1 to 3 steps.
- **On screen:** the only Jacobian is 10_3's space form, and `J_body` appears only in the 13_4 panel (`steps.rs` line 410). The panel's comment names the frame of `error` but not of `J_body`.
- **Stopping rule in the space form:**
  - It is not the tip's distance, because the slide part is the velocity of the point at the room's origin.
  - |v_s| against the tip distance is 661 mm against 505 mm at the start, and 5.54 mm against 4.15 mm at step 2.
  - A tip exactly on the goal but 0.0009 rad off gives |v_s| = 0.42 mm, so the "0.1 mm" test would not stop.
  - In the body form |v_b| is the tip distance to second order (535 against 505 mm at the start, then 93.98 against 93.93, 4.154 against 4.154, 0.026 against 0.026).
  - So "within a tenth of a millimeter" in 13_5 is only true of the body form.

**CORRECTED STATEMENT:** The panel uses the body Jacobian, which is never defined on screen. A viewer who pairs ch 10's J with the body error gets a loop that does not converge. The space form (space error with J) is equivalent in iterates but its "0.1 mm" no longer means the tip's distance, so define J_body rather than switch.

**FIX:** Add one comment line to the 13_4 code panel, above the `q = q + …` line.

`# J_body: chapter 10's grid, each twist read in the gripper's frame`

**FIX CHECK:** True: J_b = Ad_{T⁻¹} J_s, which I verified by finite differences. It is on-screen text, so the voice rules do not apply. It touches no protected phrase and adds no new word with two meanings, since "gripper's frame" is already in the panel. The layout lint (panel height) needs a re-run.

---

## B-02 No rule for building a Jacobian column

**STATUS:** PARTIALLY_SUPPORTED. The gap is real and the proposed line is correct, but the gap is modest.

**WHY:** On screen, the 10_3 tag says only how to read a twist: slide is the velocity of the point at the origin, and tip velocity is slide + spin × tip. No beat gives the six-vector of a column. 08_2's tag gives the action x → c + R(x − c), which makes the exponential computable but not the 6-vector.

I tested the proposed rule on 200 random poses × 7 joints. The columns of `jac_space(q)` equal (ω, −ω × c), with ω = the present axis direction and c = any point on the present axis (I shifted c randomly along the axis). The maximum error is 2e-15.

At the L shape, column 2 = (0, 1, 0, −0.36, 0, 0), which equals `G-J.ell.col2` with spin first. The tag's own identity (tip velocity = slide + spin × tip) holds, and the levers 0.42, 0.6731, 0.526, 0.526, 0, 0.126, 0 match the fact sheet.

One precision: ω must be a unit vector, otherwise the column is scaled. The video's notation for an axis is ω (05_3, 05_6), not w.

**CORRECTED STATEMENT:** Same claim, with "unit" and ω.

**FIX:** Add a third line to the 10_3 "for programmers" tag.

`column i = (ω, −ω × c): ω a unit arrow along joint i's axis, c any point on it, both where the arm is now`

**FIX CHECK:** True (above), spin first, no spoken change, no protected phrase, and ω and c match 05_6 and 08_2. `velocities_add.rs` has a test that the note ends inside the margin, so the third line must re-pass it.

---

## B-03 "it" and "Its limit" in 11_5

**STATUS:** PARTIALLY_SUPPORTED. The missing owner and source of the limit is real. The pronoun is a mild ambiguity that the picture resolves.

**WHY (exactness):**
- Under the reading "pure slide back along the shoulder-to-wrist line, gripper orientation held", the elbow rate is exactly 54.03 °/s at a 30° bend and 160.44 °/s at 10°. At 3° it is 534.17 and at 1° it is 1602.34, all matching `G-J.blowup.*`.
- The J⁺ solutions give the same |q̇₄| for six random other-joint settings per bend. The Jacobian's null vector has zero elbow component, so the number does not depend on how the other joints share the motion.
- If the gripper may turn, the numbers are not required. With q̇₄ = 0 the tip still goes back at 0.100 m/s, using 135 °/s of gripper spin at 30° and 56 °/s at 10°.
- The video's own vocabulary supports the exact reading: 11_3 calls the lost direction a slide "straight back", with no spin.
- The crossing at 21.5° gives 74.97 °/s.

**WHY (pronoun):** "it" has two candidates, the gripper (object of the previous sentence) and the elbow (inside the fronted clause). The picture settles it: the y-axis title is "its speed, °/s" under "the elbow's bend, degrees", the gauge says "the elbow's speed to pull the tip back at 10 cm/s", and the arm's elbow tweens to 30° at "With the elbow bent thirty degrees".

**WHY (limit):**
- The red label is just "limit 75". The owner and source appear only in the Dev note ("KUKA's published speed for axis 4 at rated payload"). That is a data-sheet figure, not a law.
- I confirmed A4 = 75 °/s "speed with rated payload" (A1–A7: 85, 85, 100, 75, 130, 135, 135) from a KUKA-derived data sheet listing and robots.com.
- Rated payload is 14 kg and reach is 820 mm.
- I did not open KUKA's PDF. The search summary said the 01.06.2022 sheet might be the CR (cleanroom) variant's. The fact sheet says the 2015 specification agrees; I did not open that either.

**CORRECTED STATEMENT:** The numbers are exact for a pure slide with the gripper's orientation held. The limit line carries no owner or source on screen.

**FIX:** Picture only, with no re-speak.
1. Change the limit label (both the gauge's and the plot's) from `limit 75` to `KUKA's limit, axis 4 (elbow), at rated payload: 75`.
2. Extend the "from the geometry alone" tag to `from the geometry alone: gripper held at one angle, tip pulled straight back`.

If the owner wants the sentence itself exact, the smallest spoken change is "it must turn" → "the elbow must turn". "Its limit is seventy-five" stays untouched, and the pronoun then has a clear owner.

**FIX CHECK:** The label text is true (75 °/s, A4, rated payload per the sheet; 14 kg per the fact sheet). The spoken alternative has no punctuation or numbers and no protected phrase changed. It adds no double meaning, since "turn" is already used for joints (10_2, 10_4).

---

## B-04 "A pose like this is called a singularity"

**STATUS:** PARTIALLY_SUPPORTED. The technical point is verified. The damage is small and a lower-third fix suffices.

**WHY:** Singularity is rank J(q) < 6, a property of q. The video defines "pose" as the gripper's position and orientation (01_5), uses it so in 12_4, 13_1, 13_3 and 14_4, and 12_4's whole point is that one gripper pose has a circle of joint settings.

**Witness (b04c.py):**
- **Same gripper pose in both rows:** p = (−0.526, 0, 0.780) m, R = [[0,0,−1],[0,1,0],[1,0,0]], with the pose difference at 2e-16. This is the elbow-only family of `G-J.rank.elbow_only`, with the elbow bent 90°.
- **q_a = (0, 0, 0, 90, 0, 0, 0)°:** singular values 2.080, 1.794, 1.414, 0.363, 0.222, **0**, so rank 5. The elbow point is (0, 0, 0.78).
- **q_b = (−111.8, 39.0, 68.2, 90, −69.1, 41.0, 110.9)°:** singular values 2.057, 1.705, 1.308, 0.384, 0.185, **0.151**, so rank 6, and within KUKA's limits. The elbow point is (−0.098, −0.245, 0.686).

A nuance in the claim's favour: for the spoken quiz pose (elbow straight, wrist centre at the full 0.82 m reach) every joint setting reaching that gripper pose is singular. I found 20, all with |q₄| ≤ 2e-5 and smallest singular value ≤ 3e-6. So "a pose like this" is true of the spoken example, but not of the lower third's second example, "straight up with the elbow bent".

"Singular pose" is common loose usage. A roboticist would say "configuration". The word does carry two meanings in this video: the gripper's pose (defined) and the arm's whole posture (11_4).

**CORRECTED STATEMENT:** "Singularity" belongs to the joint angles. A gripper pose can be reached by both singular and non-singular joint angles (witness above), except at full reach, where all are singular.

**FIX:** Lower third of 11_4, prepend "a property of the joint angles:".

`a property of the joint angles: not only at full stretch. Straight up with the elbow bent, this arm also has five; straight up, three`

If the spoken sentence must be exact, the smallest wording is "A set of joint angles like this is called a singularity."

**FIX CHECK:** True, since rank depends on q only. The lower third is on-screen text, with no voice-rule issue and no protected phrase. "Joint angles" is already vocabulary, and the lower third now does not use "pose" for the arm's setting. The spoken alternative has no colon, dash or number and still reads with 11_4's second sentence. I recommend not re-speaking.

---

## B-05 Closed-form IK: fact sheet against brief

**STATUS:** VERIFIED.

**WHY (by reading):**
- Fact-sheet finding 8 (lines 104–107): closed form "was not checked from a source… the video makes no claim."
- The fact sheet's 13_6 Simplification row (line 191) ends "A closed-form solution exists for this arm."
- Brief item (10) says "this arm does have a closed-form solution", while the brief's "Out of scope" row lists closed-form IK.
- The script and storyboard make no claim.

**What is true (b05.py):**
- Joints 1, 2 and 3 axes pass through one point, to 1e-16 m, over 300 random q. Joints 5, 6 and 7 axes pass through one point likewise.
- |shoulder–elbow| is constant at 0.420000 and |elbow–wrist| at 0.400000.
- The self-motion dimension at random settings is 1.
- So the arm is spherical–revolute–spherical with no offsets. A closed form exists per chosen value of one redundancy parameter, the elbow's swing (swivel or arm angle), with a few discrete branches.
- Search results point to Shimizu et al. 2008 for SRS arms and Kuhlemann et al. 2016 for the iiwa. I did not open either paper.

**CORRECTED STATEMENT:** For a chosen elbow swing (one free number) this arm has a closed-form solution, by its geometry: spherical shoulder, elbow, spherical wrist, no offsets. The structure was checked numerically; no paper was opened here; the video makes no claim.

**FIX:** Fact sheet and brief only, with no viewer-facing change.
1. Row 13_6, replace the last sentence with the corrected statement.
2. Brief item (10), replace "this arm does have a closed-form solution" with "per chosen elbow swing, a closed form exists; the video does not say so".
3. Finding 8, record the numeric check and the unopened sources.

**FIX CHECK:** True, and it agrees across the three places.

---

## B-06 15_1 additive step and the FrankenSim tag

**STATUS:** PARTIALLY_SUPPORTED. The numbers are right. The words are true. Both proposed additions are optional, and the "cannot look up" half is wrong.

**WHY (numbers, b06.py):** 1000 steps of R += 0.01[ω]R at 1 rad/s give det 1.1052 ((1+10⁻⁴)¹⁰⁰⁰ = 1.10517) and singular values 1.0513, 1.0513 and 1.0000. The exponential gives det 1.000000, with orthogonality error 3e-14.

**WHY (words and picture):**
- The words say "can". A real engine is cited: `fs-time/src/se3.rs`'s `se3_exp_step` is documented as `T_next = T * Exp(h * twist_body)`.
- The picture's failure label "add the spin to the nine numbers" is loose. The precise name is "forward Euler on the matrix", which is only in the Ruth note.
- A unit quaternion renormalised each step also stays rigid. In my run |q| = 1 by construction and the angle error was 8.3e-5 rad (0.005°) after 10 s, so "only first-order in timing" is not accurate for a constant spin.
- "No real engine ships the additive step" is a HYPOTHESIS. I did not verify any engine, and the video does not say one does.
- Nothing in the words says the exponential is the only way.

**WHY (FrankenSim lookup):** `gh repo view Dicklesworthstone/frankensim` shows PUBLIC, and the function exists as quoted. The system can be looked up by name. The weak point is that the description needs to carry the repo and path.

**CORRECTED STATEMENT:** The picture's naive method is forward Euler on the nine numbers, and it is a test the video ran. The cited engine is public.

**FIX:** Tag text.

`our test: 1,000 steps of 0.01 s at one radian a second. Left: R ← R + 0.01 [ω] R, forward Euler on the nine numbers. Right: the exponential step. FrankenSim (github.com/Dicklesworthstone/frankensim, fs-time, se3_exp_step) steps bodies the second way.`

Put the same repo and path in the description. I would not add the quaternion note.

**FIX CHECK:** True. It is on-screen only, with no protected phrase and no double meaning.

---

## B-07 06_4 "you predict three meters straight ahead"

**STATUS:** REJECTED.

**WHY (b07.py):** The exact end is (0, 1.9099).

| Method | dt | End point | Error |
|---|---|---|---|
| Heading-aware Euler (x += v cosθ dt, θ += ω dt) | 0.1 | (0.0500, 1.9094) | 50 mm |
| Heading-aware Euler | 0.01 | (0.0050, 1.9099) | 5.0 mm |
| Heading-aware Euler | 0.001 | (0.0005, 1.9099) | 0.5 mm |
| Additive step in the platform's own components (pose += velocity·dt) | any dt | (3.000, 0.000), heading 180° | 3.56 m |

The claim's "within 5 mm at dt 0.01" is correct. But its integrator turns the velocity by the current heading each step. That is the wrap done in small pieces, which is the video's thesis (15_1), not a straw man.

The sentence is about adding the platform's velocity numbers onto the pose numbers. That gives (3, 0, 180°) at every step size. The sentence is exact. The "one big step" is already in it: "Add six seconds of velocity".

**CORRECTED STATEMENT:** None.

**FIX:** NONE.

**FIX CHECK:** Not applicable. A tag would add noise.

---

## B-08 08_6 "Hold each joint's twist for its joint angle"

**STATUS:** PARTIALLY_SUPPORTED, low materiality.

**WHY:** e^{[S]q} is the screw only for the unit twist (spin a unit axis, so one radian a second) held q seconds, with q in radians. 07_2 defines "twist" as any six-number velocity, and no beat says "one radian a second" for a joint's twist. 09_2 writes the angle as 90° in the exponent, so a viewer can read "hold for 90 degrees" as a unit mismatch.

Under the video's own definitions the spoken line is acceptable. 10_2 and 10_3 already treat "each joint's twist" as the joint's own twist, and 05_1 says a speed is held for a time. It is loose, not false.

**CORRECTED STATEMENT:** Exact for the joint's unit twist held for q seconds with q in radians.

**FIX:** Picture only. Change the label under the formula from `joint 1's twist, held for its angle` to:

`joint 1's twist at one radian a second, held for q₁ seconds (q₁ in radians)`

**FIX CHECK:** True. No spoken change and no protected phrase. "Radian a second" matches the 15_1 tag.

---

## B-09 Two dates for "it" (01_6 and 08_6)

**STATUS:** PARTIALLY_SUPPORTED, low materiality.

**WHY:**
- **Dates, Lie:**
  - Lie began his theory of continuous groups in the winter of 1873–74. I got this from a search summary of Hawkins via Wikipedia, and I did not retrieve the MacTutor page.
  - 2026 − 1873 = 153, so "about a hundred and fifty" is right.
  - Hawkins also traces roots back to 1869–73.
  - The fact sheet quotes Brockett's foreword as "150-year-old ideas of… Klein and Lie".
- **Dates, Brockett:**
  - The MTNS-83 talk in Beer Sheva is dated 1983 in the fact sheet's quote of Brockett's foreword. I could not open the foreword, and the Springer record redirected to a login.
  - The proceedings are 1984, and Wikipedia and a Carleton paper date the formula "1984".
  - "Presented… nineteen eighty-three" is the right word and year, and it is protected.
- **Overlap:** 01_6 says "the way out… you'll read it in one line". 08_6 then says "Roger Brockett presented it" right after "the line from the start". A listener can hear two ages for the same "it". Nothing on screen ties 1873 to anything. The 150 appears only in the Dev note, and the 05_7 card shows only "1842–1899".

**CORRECTED STATEMENT:** Both sentences are true. Lie's theory (1873–74) is the way out. Brockett's 1983 talk is the line for arms. The screen should say so.

**FIX:** Two on-screen additions, no spoken change.
1. 08_6 lower third: `the product of exponentials · Brockett, 1983 · built on Lie's groups, begun 1873–74`.
2. 05_7 name card: `Sophus Lie (say "Lee"), 1842–1899 · began his theory of continuous groups in 1873–74`.

**FIX CHECK:** True. "Roger Brockett presented it in nineteen eighty-three" and "after the Norwegian mathematician Sophus Lie" are untouched. No new double meaning.

---

## B-10 03_5 "radioed a request"

**STATUS:** NON_MATERIAL.

**WHY:**
- **Transcript** (NASA technical air-to-ground transcript, day 4, hour 09; read through a summarising fetch):
  - 04 08 59 35 CC: "Columbia, Houston. We noticed you are maneuvering very close to gimbal lock. I suggest you move back away. Over."
  - 04 08 59 43 CMP: "Yes. I am going around it, doing this CMC AUTO maneuvers to the PAD values of roll 270, pitch 101, yaw 45."
  - 04 09 00 30 CMP: "How about sending me a fourth gimbal for Christmas."
  - 04 09 00 40 CC: "Columbia, Houston. You were unreadable. Say again please."
  - 04 09 00 46 CMP: "Disregard."
- This is 105:00:30 GET, which matches the fact sheet and the card's "105 hours into the flight".
- "Request" is the grammatical form of the line, and "Christmas" is the joke, so it does not read as a work order. The following sentence is the narrator's own gloss.
- The reply and "Disregard" are true and add colour. They are not needed to correct anything.

**CORRECTED STATEMENT:** None.

**FIX:** NONE. Optional only: add a second line to the 03_5 quote card.

`Houston: "You were unreadable. Say again please." Collins: "Disregard."`

**FIX CHECK:** True per the transcript. The quote and the name "Michael Collins" are untouched.


---

## Audit C: clarity fixes (claims C-01 to C-11)

# Audit of CLAIMS_C (AUD-C): clarity fixes, C-01 to C-11

**Scope.** I read the storyboard rows, `script.tsv`, `brief.md`, `fact-sheet.md` and `protected.tsv`. I did not read `docs/reviews/**`, `docs/claims.md` or `docs/decisions.md`.

**What I ran.** My own code is in `<scratch>/`, run with `python3 -I`.
- **`kin.py`:** forward kinematics parsed straight from `iiwa14.xacro`. It reproduces the joint heights 0.1575, 0.36, 0.5645, 0.78, 0.9645, 1.18 and 1.261 m, the flange at 1.306 m, and the axes (z, y, z, −y, z, y, z).
- **`c03.py`:** product of exponentials against the chain, and the space Jacobian against the "axis where it is now" columns.
- **`c04.py`:** Newton's method from the video's start to its goal.
- **`c06.py`:** the blend, with determinant and singular values.
- **`c07.py`:** the leftover of one-degree turns.
- **`c11.py`:** which axes meet, and the rank with each joint removed.
- **Word counts** over `script.tsv`.

For what is actually drawn I also read the scene code in `src/chapters/` (`screws.rs`, `velocities_add.rs`, `lost_directions.rs`, `spin.rs`, `everywhere.rs`, `the_book.rs`, `the_steady_path.rs`, `one_turn.rs`, `seven_screws.rs`). Where the build differs from the storyboard row I say so (C-05, C-07, C-11).

**Summary.**

| Claim | Status |
|---|---|
| C-01 | NON_MATERIAL |
| C-02 | PARTIALLY_SUPPORTED |
| C-03 | PARTIALLY_SUPPORTED |
| C-04 | PARTIALLY_SUPPORTED |
| C-05 | PARTIALLY_SUPPORTED |
| C-06 | VERIFIED |
| C-07 | PARTIALLY_SUPPORTED (15_3 half REJECTED) |
| C-08 | VERIFIED |
| C-09 | NON_MATERIAL |
| C-10 | PARTIALLY_SUPPORTED |
| C-11 | PARTIALLY_SUPPORTED |

---

## C-01 "axis" never defined; lower third "axis: the line a turn spins about"

**STATUS:** NON_MATERIAL

**WHY:**
- **Count:** the count is right. "axis" appears 22 times and "axes" twice, 24 in all, from 02_1 on.
- **No spoken definition:** it is never defined in words. But at 02_1 the gold line fades in at "a single turn" and the book swings about it exactly as "about a single axis" is spoken (`one_turn.rs`).
- **Link to 01_1:** the line is gold, the same colour as the two table lines of 01_1 (palette: "a screw's axis" is hero gold).
- **Later uses:** every later use is spoken over a drawn line: rings (03), joint axes (08), levers (10_5), the wrist (11_6).
- **School background:** a 14-year-old already knows "axis of rotation" from school.
- **Result:** I can construct no wrong or missing idea that picture and words together would leave.
- **Defect in the proposed text:** "spins about" collides with the video's defined word *spin* (05_1, a speed arrow). The four-word card at 07_3 says a turn is how far and a spin is how fast, so "a turn spins" mixes them.

**FIX:** NONE needed. If a label is wanted anyway, write the word "axis" on the gold line when it first fades in (02_1 picture). That adds no sentence.

**FIX CHECK:** a bare label is true and adds no spoken word. A sentence version, if forced, would be "axis: the line the turn is about", which echoes 01_1's "about one line". It uses neither "spin" nor "spins".

---

## C-02 05_1 carries four new ideas; proposed rewrite

**STATUS:** PARTIALLY_SUPPORTED

**WHY:**
- **Dense, not misleading.** Every clause is true.
  - 04_4 measured 20°.
  - Reading it as 20° a second held one second is the exponential's own definition (05_3).
  - The flat piece of the turns of space is the arrows of spin.
  - An arrow along the axis with length equal to the speed is the standard picture.
- **The picture already carries the concrete number.** The build shows "20° a second, for one second" at "Read it as a speed" (`spin.rs`).
- **One real gap:** no on-screen word "spin" exists at 05_1, although "such spins" is spoken. The four-word card does not arrive until 07_3.
- **The rewrite is true on its numbers.** 04_4 gives twenty degrees, and "a spin is one arrow along its axis, as long as its speed" is correct.
- **It is not a safe replacement, for two reasons.**
  - It drops "the flat piece is made of such spins". That clause is the only spoken link between "flat piece" and "arrows in space". 05_3 ("from the flat piece back to the curved space"), 05_7 ("Its flat piece is its Lie algebra") and 10_2 ("twists live on the flat piece") all lean on it.
  - It says "three dimensions", but the video's word for this is "space" everywhere (05_1, 07_1, 07_2).
- **Spoken change not warranted.** The original is not false or misleading, so the constraint against re-speaking applies.

**CORRECTED STATEMENT:** 05_1 is dense but true. The only gap is an on-screen name for "spin" at its first use.

**FIX:**
- **Words:** leave 05_1 as is.
- **Picture (05_1, at "Each is one arrow"):** put the word "spin" on the arrow. Add one caption under it: "spin: how fast it turns. The arrow's length is the speed."

**FIX CHECK:** true. It adds no spoken words and uses the video's own term. "turns" is the video's ordinary verb. It does not use "spins about", so there is no double meaning with 07_3's card.

---

## C-03 10_2 "each axis counts where it is right now" after 09_6

**STATUS:** PARTIALLY_SUPPORTED

**WHY:**
- **The sentence is true.**
  - Lynch–Park eq. 5.11 columns agree with "each joint's screw at its present axis" to 3e-16 (`c03.py`).
  - The product of exponentials agrees with the chain to 2e-16.
- **The axes really are elsewhere in the video's L shape** (joints at 0, 90, 0, 90, 0, 0, 0):

| Joint | Where its axis is in the L shape |
|---|---|
| 1 | unchanged |
| 2 | unchanged |
| 3 | turned 90° |
| 4 | moved 0.594 m |
| 5 | moved 0.420 m |
| 6 | moved 0.594 m |
| 7 | moved 0.420 m |

- **The contradiction is real only if "For speeds," is dropped.** That prefix does mark the switch from 09_6's "never asked where the elbow's axis is now".
- **The picture does not help.** 10_2 shows joints 2 and 1 (shoulder, base). Those are exactly the two axes that have not moved in this pose, so "now" equals "at zero" in the one demonstration shown.
- **The proposed new sentence is true.** "For speeds the arm has already moved, so each axis counts where it is right now" holds in the L shape.
  - The "so" suggests that history is the reason. The reason is that a speed is a statement about this instant. At the zero pose the rule is the same, and "has already moved" is then false.
  - It also re-speaks a sentence that is not false.

**FIX:** picture only. When "each axis counts where it is right now" is spoken (10_2), replay 09_6's pair for the elbow. Draw the elbow's axis where it is now (the 0.59 m move above), solid gold. Draw its zero-pose position as a faint dashed ghost, with a small tag "for speeds: this one".

**FIX CHECK:** true. No spoken change, so no protected phrase is touched. It reuses 09_6's own picture with the roles swapped, so it adds no new term.

---

## C-04 10 to 12 do not say why speeds come first; add "because the robot gets there in small steps"

**STATUS:** PARTIALLY_SUPPORTED

**WHY:**
- **The gap is real but small.**
  - The pink goal stays through 10 to 12 (dimmed).
  - 10_2 lights the badge ("twists live on the flat piece, so they add").
  - 13_1 opens with "Back to the question".
  - The promise at 10_1 and its payoff at 13_4 are therefore signposted. Only the reason is missing, for about 3 minutes.
- **"Small steps" is false of this video's own run.** My Newton run matches `G-I.ik.*` (`c04.py`):

| Step | Position error | Orientation error | Joint move |
|---|---|---|---|
| start | 504.99 mm | 90° | |
| 1 | 93.93 mm (81% of the distance gone) | 8.64° | norm 80° (joint 7 about 46°, joint 6 about 44°, joint 4 about 33°) |
| 2 | 4.15 mm | 0.72° | norm 17° |
| 3 | 0.026 mm | 0.0076° | norm 1.5° |

  - Steps 1 and 2 are large; only step 3 is small.
- **What is true:** each Newton step is "a joint speed held for one second". The error twist is "the twist that would reach the goal in one second" (13_3's own lower third), and `q = q + pinv(J)·error`.

**CORRECTED STATEMENT:** the missing reason is "each step toward the goal is a speed held for one second", not "small steps".

**FIX:** picture. Keep the dimmed pink goal through 10 to 12 and give it a standing label: "the goal: reached in steps, each a speed held for one second". No corner tag is needed (the corner holds the badge). No spoken change.

**FIX CHECK:** true on the numbers above. It echoes 05_1 and 05_3's "held for one second". It contains no word with two meanings and no protected phrase.

---

## C-05 07_3 lower third "point at the origin" is too much

**STATUS:** PARTIALLY_SUPPORTED

**WHY:**
- **The note's content is true.**
  - The twist's slide arrow is the velocity of the point at the room's origin (fact sheet 10_2 row).
  - Its part perpendicular to the spin sets the axis's offset. With unit spin ω and slide v, the offset is ω × v.
  - For the platform, v/ω = 0.955 m (`G-P.radius`).
- **Nothing later needs the second clause.**
  - 10_3's tag states the origin fact in full, and that is the one that matters for 10_4 and 10_5.
  - "Its sideways part moves the axis" is used nowhere later (nothing in 08 to 15 reads a twist's slide arrow back into an axis position).
- **The note is dense:** 24 words, about 6 s on screen. Two of its words are traps:
  - "origin" is first defined at 10_3 as "the room's origin" (the 07 note says just "the origin").
  - "sideways" is chapter 06's word for the platform's lateral velocity (forward, sideways, spin). Here it means "perpendicular to the spin arrow".
- **The "lands as the card appears" timing is a storyboard-row artifact.** In the build (`screws.rs`) the note fades in at "Six numbers" in 07_2 and is removed at "The body turns about an axis" in 07_3. The four-word card comes later, after the helix. They never share the screen.
- **"Give the card tiny pictures"** answers nothing I could find. It is optional, so leave it.

**CORRECTED STATEMENT:** the note is too dense and uses two double-meaning words. It is not co-located with the card in the build.

**FIX:**
- **Placement:** keep it with the twist's six numbers (07_2), as built.
- **Register:** re-set it as a "for programmers" tag.
- **Text:** shorten it to "a twist's slide arrow is the velocity of the point at the room's origin" (the same words as 10_3's tag).
- **Storyboard:** move the row's lower-third text from 07_3 to 07_2 so it matches the build.
- **Cross-references:** update the fact sheet's finding 13 and the storyboard's Ruth bullet for chapter 07, which both cite the 07_3 lower third.

**FIX CHECK:** true. It is spoken-silent. It drops "sideways" and "as the platform's centre moved" and uses "room's origin" consistently with 10_3. The picture (the gold centre dot standing up into the axis) still carries the platform link.

---

## C-06 14_2 quiz: add a middle choice "not halfway, but an orientation"

**STATUS:** VERIFIED

**WHY:**
- **Numbers.** From the video's two orientations (90° apart about z):

| Quantity | Value |
|---|---|
| Blend determinant at s = ½ | 0.5000 (the formula 1 − 2s(1−s)(1−cos a) agrees) |
| Singular values of the half blend | 1, 0.7071, 0.7071, so not orthogonal |
| Nearest turn at s = ¼ (renormalised) | 18.43°, not 22.5° |
| Nearest turn at s = ½ (renormalised) | exactly 45° |

- **The middle choice is false in either reading.**
  - The raw blend is not an orientation.
  - The renormalised blend at ½ is exactly halfway, so "not halfway" is false for it too.
  - It is therefore a clean distractor.
- **It is a belief a viewer can actually hold.** It is the direct transfer from chapter 04, where averaging gave a valid heading in the wrong place. A viewer who has just learned that "the average is a valid answer but wrong" would pick it.
- **Two choices are a form giveaway.** Plain "halfway" against the odd "not even an orientation" invites picking by form.
- **14_3 still follows.** "It is not even an orientation" now reads as the answer to the third choice, with the middle one contrasted by the word "even".
- **Cost.** The card edit is chapter code only (`ASK` in `the_steady_path.rs`); `model::QUIZZES` holds only the countdown.

**FIX:** 14_2 quiz card, three choices: "halfway" / "an orientation, but not halfway" / "not even an orientation". (The claim's order and wording also work.) The countdown can stay at 4 s; 08_3's card has about as many words. If extended to 5 s, `G-Q.14_2.seconds` and its oracle key must change in step.

**FIX CHECK:** all three are true or false as intended. No spoken word changes. It introduces no new term, and "halfway" keeps one meaning.

---

## C-07 15_3 "that leftover" before it is named; 15_4 "a rule of its own" says no rule

**STATUS:** PARTIALLY_SUPPORTED (the 15_3 half is REJECTED)

**WHY (15_3):**
- **The leftover is named on screen before it is spoken.** The row says the wedge's "angle live". The build fades in the readouts "turns:" and "leftover:" at the start of 15_3, before "They do" (`everywhere.rs`, lines 440 to 455).
- **The anaphor resolves.** "that leftover" refers back to "by less than two hundredths of a degree", which the readout shows next to the word "leftover".
- **A re-speak adds nothing.** "Call that the leftover" does not help; it re-speaks a sentence that carries two protected phrases.
- **Verdict on this half:** REJECTED.

**WHY (15_4):**
- **The existing lower third already states the rule, in symbols.**
  - It reads "leftover ≈ a turn by angle × angle (in radians) about roll's arrow × tilt's arrow, which is −z: the Lie bracket [y, x] = −z".
  - I recomputed it (`c07.py`): the leftover of 1° turns is 0.0175° about (−0.0087, 0.0087, −0.9999), which is 0.71° off −z.
  - At 10° it is 1.7409° (product of angles 1.7453°), 7.05° off.
  - The 10-to-1 ratio is 99.75, about a hundredfold.
  - y × x = −z.
  - Turns about one axis leave 0.
- **The bracket badge already draws a third arrow at right angles.**
- **The remaining gap:** the rule is in cross-product notation, "×". The video has shown "×" only in tags (05_6, 10_3), so Maya cannot read it.
- **The proposed plain line is true** at this level: the bracket is the cross product, perpendicular to both arrows (zero for parallel arrows).
- **Defect in the wording:** "make a third" reuses "third" from 05_2's protected "a third answer", which was the sum arrow lying between the two. The 15_4 arrow is perpendicular, not a sum.

**CORRECTED STATEMENT:** "that leftover" is already named on screen; the only gap is a plain-language statement of the rule in 15_4's lower third.

**FIX:** 15_4 lower third. Put a plain lead before the symbols: "the leftover turns about the arrow at right angles to both spin arrows. The Lie bracket: [y, x] = −z". Keep "by angle × angle" for the programmers' line if wanted. Leave 15_3 and the spoken 15_4 untouched.

**FIX CHECK:** true and spoken-silent. It uses no "third", no colon issue (on-screen only), and no new term. The protected phrases "by less than two hundredths of a degree", "almost exactly about the one axis neither turn used" and "called the Lie bracket" are untouched.

---

## C-08 03_6 lower third: "q and −q" collides with q as joint angles

**STATUS:** VERIFIED (low severity)

**WHY:**
- **q already means the joint-angle vector from 01_6.** The cold open's line is `T(q) = e^{[S_1]q_1} … M`. It means the same in 08_6, 10_3 (`V = J(q) q̇`) and 13_4.
- **The 03_6 lower third (`three_angles.rs`) uses q for a quaternion,** so one letter has two meanings. The brief's notation row reserves "joint angles q".
- **Harm:** it is a 3 to 4 s read between two uses. Harm is small but real; the cost of removing it is zero.
- **The replacement is true.** A unit quaternion and its negative are the same orientation (four give two names). The word "quaternion" is already in the same frame ("unit quaternion: 4 numbers − 1 rule = 3").

**FIX:** 03_6 lower third, "nine give one name each, four give two (a quaternion and its negative), with no jump".

**FIX CHECK:** true and spoken-silent. It adds no new term and no collision. "negative" could be misread as a negative number, which is minor.

---

## C-09 01_4: crossed "orientations don't" unqualified

**STATUS:** NON_MATERIAL

**WHY:**
- **The cross is a red ✗ glyph beside the text, not a strike-through** (`mark()` in `the_book.rs`). It reads as "fails to add", which is consistent with the speech.
- **It is up for only about 3 s.** The line leaves at 01_5.
- **The generic sentence is protected** ("positions add, and orientations don't") and is true as a statement about orientations. The video itself corrects the "never" reading about 75 s later at 02_5: "order usually matters", with the green "=" panel for one axis.
- **The proposed caption's second half is harmful.**
  - "turns about one axis do swap" uses "axis" before 02_1.
  - It pre-empts 02_5's reveal.
  - It dilutes the cold open's contrast just as it lands.

**CORRECTED STATEMENT:** the gap is real but short-lived and healed by the video's own design.

**FIX:** NONE needed. If the owner wants the on-screen line to carry the scope the storyboard's Ruth note promises ("said of these two turns, never 'never'"), append only "(these two turns)" to the crossed line. Do not add the same-axis clause.

**FIX CHECK:** "(these two turns)" is true, uses no new term, and leaves the protected sentence unchanged.

---

## C-10 05_7: smooth, the Lie algebra said once, the bracket arrives at 15_4

**STATUS:** PARTIALLY_SUPPORTED

**WHY:**
- **Smooth.**
  - The sentence says "smooth like this", and "like this" points at the picture of continuously changing turns.
  - A 14-year-old's "no jumps or corners" is close enough for the claim.
- **The Lie algebra is said once.** It is "its flat piece", which matches the source's definition (the tangent space at the identity). The bracket that completes it arrives only at 15_4.
- **The picture can carry a short gloss, but not the proposed card.**
  - The beat is about 7 s; the proposed card is about 35 words.
  - "a smooth surface" is wrong in two ways. It is not the video's term, which is "curved space" (04_5). A surface is 2-dimensional, while SO(3) is 3 numbers (03_6) and the pose group has 6.
  - "multiplying and undoing" departs from 02_5's "combine … undo".
  - "(15_4)" is an internal beat id and means nothing to a viewer.
- **The date is true** and ties back to 01_6's "about a hundred and fifty years old". MacTutor says Lie began his theory of continuous groups in the winter of 1873–74, and 2026 − 1873 = 153.

**FIX:** 05_7 labels and name card. Short only.
- **Lie group (blue):** "a smooth curved space of motions".
- **Lie algebra (orange):** "its flat piece, plus a rule for two spins (the bracket, at the end)".
- **Name card:** add "began this in 1873–74".

**FIX CHECK:** true. It uses the video's own words ("curved space", "flat piece", "spins"), has no internal id, and changes no spoken word.

---

## C-11 shoulder/base naming across 08_1, 10_4, 11_6

**STATUS:** PARTIALLY_SUPPORTED

**WHY:**
- **The model supports three joints at the shoulder.**
  - Axes 1, 2 and 3 meet at (0, 0, 0.36) in every sampled pose (residual ≤ 4e-16).
  - Axes 5, 6 and 7 meet at the wrist point.
  - Axis 4 meets neither (residual about 0.3).
  - So "the wrist is three joints whose axes meet at one point" (11_6) and 12_3's "3 + 1 + 3" are right.
- **"Two joints at the shoulder" is also right.** At the L shape the rank is 6 with all seven. Removing joint 1, 2 or 3 gives rank 5 (singular values (2.00, 1.55, 1.05, 0.36, 0.23, 0) without joint 1, and so on). Removing joint 7 or 5 leaves 6.
- **The words mean different things in different places.**
  - "the shoulder" is joint 2 alone in 09_2, 09_3, 10_5 and 08_1's label "shoulder (joint 2)".
  - "the shoulder" is the group of joints 1, 2 and 3 in 11_6.
  - Joint 1 is "the base" in 09_4, 10_4 and 10_5.
  - In the build (`lost_directions.rs`), at "two joints at the shoulder" the collars of joints 1 and 2 light with "two at the shoulder", and joint 3 lights at "a third". So joint 1 is "the base" at 10_4 and then "at the shoulder" at 11_6.
  - "the wrist" is joint 6 at 08_1 and joint 7 at 10_4.
- **Half the proposed fix is already done.** `velocities_add.rs` labels 10_4's joints "1 · the base", "2 · the shoulder", "7 · the wrist".
- **The other half conflicts.** With a group label "shoulder: three joints" at 08_1 and 11_6, "joint 2 (shoulder)" at 10_4 makes "shoulder" mean both the single joint and the group.
- **The spoken words cannot move.** 10_5's "from the shoulder's" and 11_6's "with two joints at the shoulder" are protected.

**CORRECTED STATEMENT:** the script uses "the shoulder" both for joint 2 and for the joints 1, 2, 3 group. The video says the group's true size correctly, but the labels leave the viewer to reconcile it.

**FIX:** labels only; no spoken word changes.
- **08_1:** name the three meeting points by the joints that meet there: "shoulder · joints 1 2 3", "elbow · joint 4", "wrist · joints 5 6 7". Draw axes 1, 2, 3 crossing at the shoulder dot.
- **11_6:** under "two joints at the shoulder" show "joints 1 and 2". At "a third" show "joint 3: the third". The build already has "two at the shoulder" and "joint 3".
- **Rule for 08 to 12:** a name on screen always carries its joint number(s).
- **10_4:** leave as built.

**FIX CHECK:** true on the model (axes 1–3 meet; rank 5 without any one of 1–3; 12_3's 3 + 1 + 3). Protected phrases are untouched. The numbers on every label remove the single-joint versus group ambiguity.

