# R2 mathematician audit: the three generation passes (verbatim)

*Run of the modes-of-reasoning discipline on the script and storyboard: three independent passes, each a bounded operator with its own question, a checked null allowed. Each result starts with its status line. Their claims were audited by other readers: `R2-claim-audit.md`. Saved by the producer from the reviewer's returned text, with the reviewer's scratch folder written as `<scratch>` and one private path written as `<cmaes_explainer>`; nothing else was edited.*

---

## Pass P01: statements about turns, headings, exp and log, history (deductive, with boundary witnesses)

RESULT_STATUS: NO_MATERIAL_INCREMENT

No spoken line in `script.tsv` is false at its stated scope or edges. I also found nothing that would mislead a smart tenth-grader or make a roboticist wince, and every number recomputed matches. One lower third (11_3) is literally false for the gripper tip at the pictured pose; it costs nothing to fix and is listed below the bar.

## WHAT I CHECKED

**Sources opened.** I read `script.tsv`, `docs/storyboard.md` (all 15 chapters, including lower thirds, cards and tags), `docs/fact-sheet.md`, `docs/brief.md` and `docs/protected.tsv`.

I did not read `docs/reviews/**`, `claims.md`, `decisions.md`, `oracle.py` or `src/model.rs`.

**My own model.** It uses numpy only (2.4.1, `python3 -I`; no scipy). The iiwa 14 chain is built from the joint origins and rpy values read directly from `<cmaes_explainer>/vendor/iiwa14.xacro` (lines 48 to 279), not from the oracle. I then compared my outputs with the fact sheet's values.

**Scripts and where they live.** They are in `<scratch>` as `check1.py`, `check2.py`, `check3.py`, `check4.py`, `check5.py`, `check6.py`, `check7.py` and `check8.py`.

### The quiz answers, recomputed from scratch

| Line | What I ran | Result |
|---|---|---|
| 01_2 | tilt = quarter turn about x, roll = quarter turn about y, room-fixed lines | Tilt then roll gives cover to (0,−1,0), toward the viewer. Roll then tilt gives cover to (1,0,0), the right wall. The two matrices differ. About the book's own edges the two outcomes trade places (checked). |
| 02_3 | angle and axis of the tilt-then-roll product | 120.000000° about (1,1,−1)/√3. It makes 54.7356° with each of the three edge lines. Three repeats give the identity. |
| 04_3 | headings 170° and −170° | Arithmetic mean is 0°. Circular mean (angle of the unit-vector sum) is 180°. Shorter-arc midpoint is 180°. Distances from 180° are 10° and 10°; from 0° they are 170° and 170°. |
| 05_5 | spin 120°/s about (1,1,1)/√3 for one second | The result is [[0,0,1],[1,0,0],[0,1,0]]. Edge x goes to y, y to z, z to x. It equals the roll-then-tilt matrix exactly and not the tilt-then-roll one. |
| 06_3 | platform at 0.5 m/s forward, 30°/s spin, 6 s | Closed form and a 600,000-step integration agree: (0, 1.90986) m, heading 180°. Radius 0.95493 m, arc 3.0 m, and 6/π = 1.90986. |
| 09_5 | PoE from the xacro, equal to the walked chain to 6e-16 over 50 random q | Elbow quarter turn: tip (−0.526, 0, 0.780). Plus shoulder: (0.420, 0, 0.886). Plus base quarter turn: (0, 0.420, 0.886). |
| 10_5 | distance from tip to each axis line at the L shape | Joint 1: 0.4200. Joint 2: 0.6731. Joint 7: 0.0000. Joints 3 and 4 are 0.5260, so joint 2 is the largest of all seven. |
| 15_3 | leftover of Ry·Rx against Rx·Ry | 0.017453° at 1°, 1.7409° at 10°, 120° at 90°. The ratio 10° to 1° is 99.75. The axis is 0.707°, 7.05° and 54.7° off z. |

### Other mathematics checked

- **Euler and the product of two turns (02_4).** 13,689 products, including identity and half-turn products, all gave an orthogonal matrix with det 1 and a fixed axis.
- **Commuting turns ("usually", 02_5).** 120 of 576 cube-turn pairs commute. Random pairs from SO(3): 0 of 20,000 commute. Same-axis pairs and perpendicular half-turn pairs do commute.
- **Both spins at once (05_2).** exp(sum) is 127.279° about (1,1,0)/√2 and equals neither order. A small-step simulation matches.
- **Gimbal (03_2, 03_3).** The ring-axis volume is cos(middle): 0.3420 at 70°, 0.0872 at 85°, 0 at 90°, with rank 2 at 90°. All 12 axis orders have a singular middle angle (90° for the six Tait-Bryan orders, 0° or 180° for the six proper-Euler orders).
- **Ring angles (13_2).** (0,89,0) against (90,89,90) in z,y,x order are 1.4142° apart. With the rings in the other order they would be 179.99° apart, so the picture must keep z,y,x.
- **03_6 counting.** The constraint map RᵀR has rank 6 at the identity, leaving 3 free directions. q and −q give the same turn.
- **Chasles with edges (07_5).** 2,060 poses including pure slides, a 1e-6° turn, 179.999° and exactly 180° all rebuild from axis, angle, point and slide. The 07_5 example gives axis z through (0,1) with slide 0.5.
- **Arm facts.**
  - Axes at zero are z, y, z, −y, z, y, z, and the home tip is (0,0,1.306).
  - Joints 1–3 meet at one point and joints 5–7 meet at one point for random q.
  - Only joint 4 turning leaves frames 1–3 fixed and moves links 4–7 and the flange.
- **Ranks.**

  | Pose | Rank |
  |---|---|
  | home | 3 |
  | stretched, wrist straight | 4 |
  | elbow straight, wrist bent (0,60,0,0,0,60,0) | 5 |
  | up with only the elbow bent 90° | 5 |
  | L shape | 6 |
  | L with joint 5 at 90° | 5 |
  | L without any one of joints 1, 2 or 3 | 5 |
  | L without joint 5 or 7 | 6 |

- **11_3 push along the arm.** At the elbow-straight pose all 7 joints push the wrist centre by 0.000000 along the shoulder–wrist line. The pure slide along it has least-squares residual 0.994 (not reachable). Sideways slides have residual 0.
- **11_5 elbow rates.** From my own triangle formula and from pinv on the 6×7 Jacobian, at both signs of q4:

  | Bend | Elbow rate |
  |---|---|
  | 30° | 54.03°/s |
  | 10° | 160.44°/s |
  | 3° | 534.17°/s |
  | 1° | 1602.34°/s |

  The 75°/s limit is reached at a bend of 21.49°. The null vector has zero q4 component, so the elbow rate is the same however the other joints share the motion.
- **12_4 elbow circle.** The radius is 0.28966 m at the L shape (S–W distance 0.58). Following the self-motion at q6 = 60°, the gripper pose drifts by 4.7e-16 and the radius stays 0.28966.
- **12_4 scene constraint.** At the straight-wrist L shape (q6 = 0) the 1-D null space is joints 5 and 7 counter-rotating, not the elbow swing. The storyboard's "L shape with its wrist bent" is therefore necessary, and the scene must keep it.
- **13 Newton run.** My own Newton run on SE(3), with body-frame log error and pinv of the body Jacobian, from (0,30,0,−60,0,90,0) to the fact sheet's goal gives 504.99, 93.93, 4.154 and 0.026 mm after 0 to 3 steps. The angles are 90°, 8.64°, 0.72° and 0.0076°. The result is within joint limits.
- **Newton from other starts.** Other starts took 5, 13, 10 and 4 steps, so "From here, three" is correctly start-specific.
- **14 blend.** det = 1 − 2s(1−s)(1−cos a) held on 5,000 random pairs, and none were orthogonal. At a = 90°, s = ½ the volume is 0.50. At a half turn apart, s = ½ gives rank 1. The nearest turn at s = ¼ has turned 18.435°, against 22.5° on the screw.
- **14 screw path.** The screw is 90° about z through (0.355, 0.045) with slide −0.3477. The largest gap to the straight line is 0.07584 m. At halfway the turn is 45° and det 1.
- **15 drift and counts.**
  - 1,000 adding steps give det 1.10517 and width 1.0513; exp steps give 1.0000.
  - The G1 URDF has 29 revolute joints with root link `pelvis`, so 6 + 29 = 35 and 3 + 4 + 29 = 36.

### External facts (this run's web search)

- **KUKA data sheet.** It lists axis speeds 85, 85, 100, 75, 130, 135, 135 °/s and reach 820 mm, so the "limit is seventy-five" holds.
- **Collins quote.** The Apollo Lunar Surface Journal places it at 105:00:30, consistent with 04 09 00 30.
- **Euler E478.** Presented 9 October 1775, printed 1776 (Novi Commentarii 20).
- **Mozzi 1763.** The treatise is listed in secondary sources (Wikipedia, arXiv). One source notes the sources do not all agree his result equals Chasles's theorem, which matches the script's hedge ("described such an axis").

### Not checked by me

- Brockett 1983: my search did not locate the paper.
- Comanche 055's 70° and 85° thresholds: I did not reopen the listing.
- Rodrigues 1840, Chasles 1830, Lie's dates (1842–1899, winter 1873–74): taken from the fact sheet and my background knowledge only.
- KMR iiwa Mecanum wheels and KUKA's pronunciation.

### Quantified words, tested at their edges

All of these hold, or are exactly bounded on screen:

- 02_4 "always" and "any turn followed by any turn".
- 02_5 "usually", "any", "undo any one".
- 03_6 "every orientation… a bad spot".
- 04_3 "the true halfway".
- 05_2 "a third answer".
- 05_6 "named after Rodrigues" and "any point".
- 07_5 "every time".
- 11_1 "usually".
- 14_4 "every pose on the way".
- 14_3 lower third "never a turn".

## CANDIDATE CLAIMS

### P01-01 (marginal, below the bar; optional)

- **Claim type:** deductive, with a witness.
- **Statement:** the storyboard's lower third at row 11_3: "at first, the gripper can only swing sideways".
- **Spoken lines.** The spoken 11_3 is exact, because "slide" means translation with no turn in the video's vocabulary: "Along the arm, every joint's push on the wrist is zero. So the gripper can't slide further out, and it can't even begin to slide straight back."
- **Why the lower third is imprecise.** At the pictured pose (elbow straight, shoulder and wrist bent, q = (0,60°,0,0,0,60°,0)), the tip is 0.126 m from the wrist centre at 60° to the arm line. Only the wrist centre is barred from moving along the arm. The tip is not.
- **Evidence** (`check8.py`; tip (0.8193, 0, 0.707) matches the fact sheet's G-J.elbow_straight.p):
  - Joint 6 alone at unit rate gives tip speed 0.1260 m/s, of which 0.1091 is along the arm line.
  - Joint 4 gives 0.1091 along the arm out of a tip speed of 0.4757.
  - Joint 2 gives −0.1091 out of 0.8897.
  - Over random joint-speed vectors, the largest |tip velocity along arm| / |tip velocity| is 0.997.
  - With the wrist straight the ratio is 1e-14, so the sentence is true there.
- **Inference.** What is lost is a push of the wrist along the arm. Equivalently, a pure slide of the gripper along it is lost, and rank is 5. The tip can still move along the arm line while turning.
- **Assumptions.** The pictured pose is the fact sheet's quiz pose with q6 = 60°, and "gripper" means the tip or flange as in 09_1.
- **Competing-reading check.** If "swing" means turning about the shoulder, the lower third is loosely true, but joint 2 still gives 12% of its tip speed along the arm. If the drawn wrist is straight, it is true. I could not rule out either.
- **Smallest fix.** Change the lower third to "at first, the wrist can only swing sideways", or "no joint can push the wrist along the arm". It lands on the lower third, with no re-narration and no protected phrase touched.
- **Materiality.** Low. A roboticist or Ruth might note it.
- **Uncertainty.** How a viewer reads "gripper" and "swing" in the lower third.

## NULL OR BLOCKED RESULT

Examined and found true at their stated scope, or true with the exact version on screen. None of these needs a change.

- **Turn and spin senses are shown, not spoken.** The answers "facing you", "the wall on your right" (01_2) and "to its left" (06_3) depend on tilt = far edge up, roll = cover toward the right, and a counterclockwise spin. The 01_1 and 06_2 pictures show these. The "different" answers are sense-independent. A viewer repeating the experiment with the opposite sense would see a different wall, so the picture must make the senses legible.
- **04_3 "the true halfway".** It means the midpoint of the shorter arc, which is also the circular mean. It is unique unless the headings are exactly opposite. The wrap rule is on the 04_4 tag, and the brief states the convention.
- **03_6 "nine numbers, or four… no bad spot".**
  - Four numbers name each orientation twice (q, −q), and the lower third says so.
  - "Software stores" is non-exclusive, since KUKA's own interface uses A, B, C angles.
  - An optional hedge would be "serious software", but the lower third carries the exact version.
- **11_6 "This arm has a third, and it covers."**
  - It is true at the L shape. It fails at (0,90,0,90,90,0,0), rank 5.
  - "Here" bounds it, and fact-sheet finding 9 records the exception.
- **13_3 and 14_4 "the twist that joins the two poses".**
  - It is unique unless the relative turn is exactly a half turn, where the logarithm has two equal-length answers.
  - The example is 90°.
- **12_2 "keep your shoulder still".** It could be read as locking the shoulder joint's angles, which would make the answer "no". The picture pins the shoulder point, and natural use gives "it can". Not recommended for change.
- **08_5 tag "the order of the sums".** The formula is a product. It reads as "arithmetic" in British usage, and the spoken line is exact.
- **04_5 "The space is curved".** It is extrinsically true of the circle. SO(3) has positive curvature, and the audience reading is fine.
- **06_4 "Add six seconds of velocity onto the pose".** It coincides with the tangent-vector answer at the identity start pose. That is the stated case.
- **02_3 versus 05_4 axes.** The 02_3 axis (1,1,−1)/√3 for tilt-then-roll and the 05_4 axis (1,1,1)/√3 for roll-then-tilt are consistent with 05_5. The 05_4 picture labels the other axis "the other one".

No protected phrase is contradicted. I found no false statement in script lines 01_1 to 15_5 against the quantities I could derive.


---

## Pass P02: what a working roboticist would object to (practitioner test)

RESULT_STATUS: MATERIAL_INCREMENT

Narrow increment. A roboticist would accept every spoken robotics sentence in the assigned lines at the level it is spoken. I found one low-materiality on-screen imprecision (the lower third of 11_3), one optional tag note (11_5), and one fact-sheet inconsistency that is never spoken.

## WHAT I CHECKED

**Sources opened.** I read COMMON.md, INTEGRITY.md, the assignment, MOR_CASE.md, script.tsv (all 83 lines), storyboard.md (all chapters, including lower thirds and tags), fact-sheet.md, brief.md and protected.tsv.

**Sources not opened.** I did not open docs/reviews, claims.md, decisions.md, oracle.py or src/model.rs.

**Lines covered.** All 83 script lines were read. Robotics content was examined in detail in 01_5, 06_1 to 06_4, 07_2 to 07_5, 08_1 to 08_6, 09_1 to 09_6, 10_1 to 10_5, 11_1 to 11_6, 12_1 to 12_5, 13_1 to 13_6, 14_1 to 14_5 and 15_1.

**My own model.** Code is in `<scratch>/`: `iiwa.py` and `check1.py` to `check8.py`, all run with `python3 -I`.

- **Chain from the xacro.** The eight link offsets and the rpy rolls come from `<cmaes_explainer>/vendor/iiwa14.xacro`, lines 48 to 279. They match what I know of the iiwa_stack file.
- **Independent DH chain.** A second chain uses only d = 0.36, 0.42, 0.40, 0.126 with alternating ±90° twists and no rpy.
- **Agreement.** At 6 random configurations the two chains differ by at most 1.8e-15 with all joint-sign multipliers +1. This confirms the axis structure and signs, not the lengths.
- **Jacobian, rank, null space, Newton IK.** All are my own code, not oracle.py.

### Computed results against the fact sheet

| Claim (line) | Built myself | Fact sheet |
|---|---|---|
| 7 revolute joints, alternately vertical and level axes (01_5, 08_1) | axes (0,0,1), (0,1,0), (0,0,1), (0,-1,0), (0,0,1), (0,1,0), (0,0,1) | G-A.axis.* agrees |
| Height straight up, "one point three" (08_1) | FK z at q=0 is 1.306 m; rotation = identity | G-A.height 1.3060 |
| Link lengths 36 / 42 / 40 / 12.6 cm (09_1) | axis heights 0.1575, 0.36, 0.5645, 0.78, 0.9645, 1.18, 1.261 | agrees |
| Wrist axes 5, 6, 7 meet at one point; shoulder axes 1, 2, 3 meet at one point (11_6) | point distances ≤ 4e-17 | not tabulated, consistent |
| By hand: tip after elbow 90 / +shoulder 90 / +joint 1 90 (09_2 to 09_5) | (-0.526, 0, 0.78) / (0.42, 0, 0.886) / (0, 0.42, 0.886) | G-K.ell.after_elbow, G-K.ell.p, G-K.quarter.p agree |
| Levers in the L shape 67 / 42 / 0 cm (10_5) | 0.6731, 0.4200, 0.0000 (joints 2, 1, 7); also 0.526, 0.526, 0, 0.126 for joints 3 to 6 | G-J.ell.lever.* agrees |
| Jacobian 6 by 7; rank 6 in the L shape (10_3, 11_6) | 6x7, rank 6, smallest singular value 0.23 | G-J.rank.ell 6 |
| 11_6 without joint 3 / without joint 7 | rank 5 / rank 6; also without joint 1 5, joint 2 5, joint 4 5, joint 5 6, joint 6 5 | G-J.rank.ell.without_3 and without_7 agree |
| 11_6 six-joint arm: angular rank of joints 1, 2, 4, 5, 6, 7 in the L shape | 2 of 3 | consistent with "loses a direction here" |
| Ranks: straight up 3; stretched out horizontally 4; elbow straight 5; straight up with elbow bent 5; L shape with joint 5 a quarter turn 5 | 3, 4, 5, 5, 5 | G-J.rank.* agrees |
| 11_3, push along the arm for every joint at elbow-straight | 1e-17 or smaller for all 7 joints (wrist centre) | G-J.elbow_straight.push_along_arm 0.0000 |
| 11_5, elbow speed for a 0.1 m/s pure pull at bends 30, 10, 3, 1 deg | 54.03, 160.44, 534.17, 1602.34 deg/s; unchanged when a null-space vector is added; matches v·d/(a·b·sin ε) | G-J.blowup.* agrees |
| 75 deg/s met at a bend of 21.5° | 21.49°; 1602 deg/s = 4.45 turns/s | agrees |
| Elbow circle 29 cm (12_4) | height of triangle 0.42/0.40/0.58 is 0.2897; integrating the null motion keeps the elbow at 0.28966 to 0.28967 from the shoulder-wrist line while the gripper pose drifts by at most 4e-5 (integration error). Null space is 1-dimensional. | G-J.elbow.radius 0.2897 |
| Newton from q=(0,30,0,-60,0,90,0) (13_5, 13_6) | position error 504.99, 93.93, 4.15, 0.03 mm; angle 90, 8.6404, 0.7239, 0.0076 deg; converged by the 0.001 rad / 0.1 mm rule in 3 steps; final q and all iterates within ±170, 120, 170, 120, 170, 120, 175 | G-I.ik.* agrees |
| 13_2: (0,89,0) vs (90,89,90) | 1.4142° for ZYX; 179.99° if ordered XYZ | G-E.near.apart 1.4142 |
| 14_3: blend of a quarter turn | det 0.5000; nearest turn at s=¼ is 18.4349° | G-I.blend.det, G-I.blend.quarter_turned agree |
| 14_4, 14_5: screw between start and goal | angle 90°, axis z, slide -0.3477 m, midpoint turned 45° with det 1.000000, tip gap to the straight line 0.0758 m | G-I.screw.*, G-I.bow agree |
| 06_2, 06_3: platform | small-step integration ends at (0.0000, 1.9099), heading 180°, radius 0.9549, path length 3.000 | G-P.* agrees |
| 15_1: additive steps | det after 1000 steps of 0.01 s at 1 rad/s is 1.1052, widths 1.0513 / 1.0513 / 1; exponential steps keep det 1.000000 | G-C.drift.* agrees |
| 15_3: one-degree leftover | 0.0175°, axis z-component 0.99992; 10° gives 1.7409°; 90° gives 120° | G-B.leftover.* agrees |
| 15_1: humanoid counts | 29 revolute joints in g1_29dof.urdf (12 leg, 3 waist, 14 arm) plus one floating joint; 6+29 = 35, 3+4+29 = 36 | G-C.g1.* agrees |
| Book facts (01_2, 02_3, 05_2, 05_5) | tilt then roll 120° with cover to (0,-1,0); roll then tilt cover to (1,0,0); both-at-once 127.28° about (1,1,0)/√2; diagonal 120° spin equals roll-then-tilt | agrees |

**Unchecked here (no network).**
- KUKA's published A4 speed of 75 deg/s and the other speeds 85, 85, 100, 130, 135, 135. I recall this as the LBR iiwa 14 R820 table and it agrees with the fact sheet, but I did not re-open the data sheet.
- KUKA's 820 mm reach and the KMR iiwa 3.6 km/h and Mecanum wheels.
- The Lynch and Park equation numbers, page numbers and the §5.1.6 quotation.
- The joint ranges 170, 120, 170, 120, 170, 120, 175. The owner's `app/lib/armInverseKinematics.ts` line 32 lists them (not independent), and they match my recollection. They are never spoken.

## CANDIDATE CLAIMS

### P02-01 (observation) Lower third of 11_3 says the gripper can only swing sideways

- **Statement.** Storyboard row 11_3 (storyboard.md line 352), lower third: "at first, the gripper can only swing sideways".
- **Spoken context.** 11_3: "No. Along the arm, every joint's push on the wrist is zero. So the gripper can't slide further out, and it can't even begin to slide straight back. Five of the six directions are left." The spoken words say "wrist" for the push and "slide" for the lost motion. The lower third drops both qualifiers.
- **Why it misleads.** What is zero is the wrist centre's velocity along the arm line. The gripper's tip (the flange origin) can still move along that line, but only while turning. Only the pure slide, with no turning, is lost.
- **Evidence (check5.py, check2.py).**
  - Pose q = (0, 60°, 0, 0, 0, 60°, 0), the fact sheet's elbow-straight pose. Wrist centre (0.7101, 0, 0.77), flange origin (0.8193, 0, 0.707).
  - Flange-origin velocity along the shoulder-to-wrist line per rad/s of joints 1 to 7 is [0, -0.1091, 0, +0.1091, 0, -0.1091, 0]. Joint 6 alone gives the tip (-0.063, 0, -0.1091) with angular velocity (0,1,0).
  - The wrist-centre velocity along the line is below 1e-16 for every joint.
  - At A6's 135 deg/s (2.36 rad/s) the tip could move outward at about 0.26 m/s while turning.
  - The set of reachable twists is a 5-dimensional hyperplane: v_tip·u = -0.109 ω_y. A pure pull (ω = 0) lies outside it (least-squares residual 0.099 for a 0.1 m/s pure slide).
  - fact-sheet.md line 187 already states it correctly: "the gripper cannot translate along it without turning".
- **Inference.** The lower third is true of the wrist and false of the tip. A viewer reading it literally concludes that at a straight elbow the gripper cannot move outward at all.
- **Assumptions.** The quiz pose is the fact sheet's q (the storyboard only says "shoulder and wrist still bent"). The effect is pose dependent: with the wrist straight, the tip lies on the arm line and joint 6 moves it only sideways.
- **Competing reading.** "Gripper" in the lower third could be loose language for the wrist point, which the picture draws. I checked the same beat's words: they use "wrist" for the push and "gripper" for the slide, so a viewer maps the lower third to the gripper. The competing reading is possible but not the likely one.
- **Smallest fix (lower third only, no re-narration).** Change "gripper" to "wrist": "at first, the wrist can only swing sideways". Optional second clause: "the lost direction: a slide along the arm, with no turning".
- **Materiality.** Low. The spoken words are correct under the video's own vocabulary of "slide" and "turn", and the fix costs nothing. It does not touch a protected phrase.
- **Uncertainty.** Low on the fact. Moderate on whether viewers would be misled.

### P02-02 (observation, optional) The 54 / 160 deg/s figures at 11_5 assume the gripper does not turn

- **Statement.** 11_5: "Pull the gripper straight back at ten centimeters per second. With the elbow bent thirty degrees, it must turn at fifty-four degrees per second. Bent ten degrees, a hundred and sixty." The tag is "from the geometry alone".
- **Why it is incomplete.** The figures are exact for a pure slide, and the elbow's rate is the same however the other joints share the motion (verified with null-space additions).
- **Evidence (check8.py).** If the gripper may tilt, the tip can go back along the line at 0.1 m/s with joint 6 alone at the elbow at rest. At a 30° bend that is 0.820 rad/s (47.0 deg/s); at 10° it is 50.1 deg/s, against joint 6's 135 deg/s.
- **Inference.** "Pull straight back" is the standard Cartesian linear move with orientation held, and the video's own "slide" vocabulary says the same. A roboticist would read it that way, so this is not a falsehood.
- **Fix.** Optional tag change: "from the geometry alone, gripper not turning". No re-narration.
- **Materiality.** Very low. Drop it if the lower-third change in P02-01 is not made.

### P02-03 (observation, documentation only) Closed-form IK is stated two ways in the fact sheet and brief

- **Statements.**
  - fact-sheet.md Finding 8 (line 104): closed-form IK "was not checked from a source… the video makes no claim".
  - fact-sheet.md line 191, row 13_6: "A closed-form solution exists for this arm."
  - brief.md "What the script must get right" (10): "this arm does have a closed-form solution".
- **Problem.** The first says unchecked; the other two assert it. For a 7-joint arm the usual closed form is per value of the one free redundancy parameter (the elbow angle on the 29 cm circle), not a single solution. That is my recollection and I did not open a source for it.
- **Spoken content.** Nothing spoken or on screen asserts closed-form IK, so no re-narration is involved.
- **Fix.** Make the fact-sheet row and brief agree with Finding 8, or add "for a given elbow position".
- **Materiality.** Documentation only.

## NULL RESULT FOR THE REST OF THE PIVOTAL SUBQUESTION

I judged each of the following as accepted by Lynch and Park, Murray/Li/Sastry and KUKA's published data at the level the video speaks. None were demoted to "fine" without a recomputation or an explicit "unchecked".

- **Arm facts.**
  - 01_5 and 08_1: seven joints, hinge joints, 1.306 m tall, "turn about its own axis".
  - 09_1: lengths 36 / 42 / 40 / 12.6 cm.
  - 11_5: "Its limit is seventy-five" (unchecked against the data sheet, see above).
- **Mecanum platform, 06_1 to 06_4.** Slanted rollers, three numbers for pose and for velocity, half circle, 1.91 m left and facing backward, three metres bending about a fixed point. Wheel speed numbers are unchecked.
- **Product of exponentials, 08_5 and 08_6.** Apply the last joint's screw first, so each proximal axis is still where it stood at zero. This is the space form (verified by the by-hand poses).
- **Jacobian, 10_2 and 10_3.**
  - Columns are the joint twists at the current pose.
  - "Joint speeds go in. The gripper's twist comes out" holds with the 10_3 tag.
  - The 6 by 7 shape and "Usually, seven joints cover all six" (generic rank 6).
  - "Run the Jacobian backwards" (13_4) with the pinv code and "the smallest" tag.
- **Singularities.**
  - Elbow straight: rank 5, the lost direction is a pure slide along the arm.
  - "Close to one… huge joint speeds" holds.
  - 11_6 wrist: axes 5, 6, 7 meet at a point, and straightening aligns 5 and 7.
  - "A six-joint arm, with two joints at the shoulder, would lose a direction here. This arm has a third, and it covers" is scoped to "here". The rank-5 counterexample pose (joint 5 a quarter turn) is documented in the fact sheet and not contradicted by the words.
- **Redundancy, 12_1 to 12_5.** "Seven joints for six directions", the seven-joint arm "the way a roboticist counts", the 29 cm circle with the gripper pose held, and the null-space lower third.
- **Newton, 13_5 and 13_6.** "From here, three", and "Newton's method, on a curved space" holds with the tag about poor starts. All iterates stay inside the joint limits.
- **15_1 remarks.**
  - "A physics engine **can** step each body by the exponential of a small twist" is true as a possibility. Many engines integrate a quaternion and renormalise, which also stays rigid.
  - "Drones steer poses" and "mapping robots estimate them" are accepted.
  - "a humanoid's… twenty-nine joint angles" matches the 29-joint G1 file, which has a floating base joint. The lower third names the G1.

## CONSIDERED AND NOT RAISED

- **11_5, "it must turn".** The antecedent "it" is read by a listener as the elbow (the last noun, and the picture is "the elbow's speed"). Low risk.
- **13_4 and 13_6, "move the joints that way… the arm takes its three steps".** The words describe an iteration, which is also valid as closed-loop resolved-rate control. The picture shows iterates as hops. This is a production-engineering caveat I judged out of scope for this video.
- **Elbow-first swing direction at 09_2.** The forearm swings backward because axis 4 is -y; the words do not say a direction, and negative q4 bends forward as on a real iiwa. No issue.
- **11_4, "straight up, three".** This pose is itself a full stretch, so "not only at full stretch" reads loosely, but the ranks are correct.


---

## Pass P03: true but misleading to a curious 14-year-old (misconception probes)

RESULT_STATUS: MATERIAL_INCREMENT

The increment is small. The spoken script has no false statement, and I found no high-risk misconception at the 14-year-old level. I did find one on-screen overstatement that can be fixed with no re-narration, plus three optional low-materiality wording points. None touches a protected phrase.

## WHAT I CHECKED

**Read (all 83 lines, 01_1 to 15_5):**
- `script.tsv`, `docs/storyboard.md` (every picture, lower third, tag and note), `docs/fact-sheet.md`, `docs/brief.md`, `docs/protected.tsv`
- `tools/review_prompts/maya_14.md`, `MOR_CASE.md`
- I did not read `docs/reviews/**`, `claims.md` or `decisions.md`, to stay independent.

**Recomputed with my own code**, `python3 -I` with numpy, in `<scratch>/` (`check.py` to `check6.py`). I built the arm's forward kinematics from the vendored xacro `<cmaes_explainer>/vendor/iiwa14.xacro`, not from `oracle.py` or `model.rs`.

| Check | My result | Matches video |
|---|---|---|
| Home flange, axis heights | flange 1.306 m; axes alternate vertical/level; heights 0.1575, 0.36, 0.5645, 0.78, 0.9645, 1.18, 1.261 | yes |
| 09 poses | (0.42, 0, 0.886) and (0, 0.42, 0.886); after elbow only (−0.526, 0, 0.78) | yes |
| 10_5 levers, L shape | 0.42, 0.673, 0 | yes |
| Ranks | home 3; arm out with wrist straight 4; elbow straight 5; elbow only bent 5; L shape 6; L with joint 5 at 90° is 5; L shape without any of joints 1 to 3 is 5; without joint 7 is 6 | yes |
| 11_3 push on the wrist along the arm | zero for all seven joints | yes |
| 11_5 elbow rate | 54.03, 160.44, 534.17, 1602.34 deg/s at 30, 10, 3, 1 deg bend | yes |
| 11_5 limit crossing | 75 deg/s met at a bend of 21.49 deg | yes |
| 11_5 independence | same value for min-norm and a null-space-shifted solution | yes |
| 12_4 circle radius | 0.2897 m at shoulder-to-wrist 0.58 m | yes |
| 13_6 Newton from `(0,30,0,-60,0,90,0)` | error 504.99, 93.93, 4.15, 0.03 mm; orientation 90, 8.64, 0.72, 0.0076 deg; solution inside limits | yes |
| 14 screw | 90 deg about vertical through (0.355, 0.045), slide −0.3477 m; midway turned 45 deg | yes |
| 14_5 gap | helix-to-line gap 0.0758 m | yes |
| 14_3 blend | determinant 0.5 at a quarter turn; equals cos²(a/2) at 30, 60, 90, 120, 150, 179 deg | yes |
| 14_3 re-normalised | 18.435 deg at a quarter of the way | yes |
| 13_2 | 1.414 deg apart for ZYX ring angles (0,89,0) and (90,89,90) | yes (see note below) |
| 15_3, 15_4 leftover | 120 deg at 90; 1.7409 deg at 10 (axis −0.087, 0.087, −0.992); 0.01745 deg at 1 deg | yes |
| 02, 05 | tilt then roll is 120 deg about (1,1,−1)/√3; 120 deg about (1,1,1)/√3 cycles the edges and equals roll then tilt; both spins at once give 127.28 deg | yes |
| 06 platform | end (0, 1.9099), heading 180 deg, radius 0.9549 m | yes |
| 15_1 drift | 1000 add-steps give determinant 1.1052, width 1.0513 | yes |

- Note on 13_2: the 1.4 deg figure holds only for the ring order drawn. The same angles in X-Y-Z order are 179.99 deg apart, and in Z-Y-Z order 121.15 deg. The storyboard draws Z-Y-X, so it is consistent.
- Two numbers I derived beyond the video: the screw path is 0.5351 m long against 0.5050 m for the line (3.0 cm longer). A shoulder-to-wrist distance of 0.5, 0.7, 0.8 m gives an elbow circle of 0.325, 0.213, 0.09 m.

**Unchecked by me** (no network; taken from the fact sheet):
- KUKA data-sheet limits and speeds, the KMR iiwa Mecanum wheels, the Comanche 055 thresholds, the Collins transcript time, the dates for Lie, Euler, Mozzi, Chasles and Brockett.

## CANDIDATE CLAIMS

**P03-01** (observation, then recommendation; the one I recommend doing)
- **Statement:** 11_3 lower third (storyboard row 11_3): "at first, the gripper can only swing sideways". Spoken 11_3: "Five of the six directions are left."
- **Why misleading:**
  - "only" says all the gripper can do is swing sideways, while the line counts five of six directions remaining.
  - The lost direction is one slide of the wrist along the arm.
- **Evidence:** at `q=(0,60,0,0,0,60,0)` rank is 5 (`check.py`, `check3.py`).
  - A pure slide of the flange along the arm line cannot be reached (residual 0.994).
  - Two pure slides across it can (0.0).
  - Pure spins about two of three axes can (0.0). The third has residual 0.108, because the lost twist mixes the along-arm slide with a bend in the plane.
- **Inference:** the lower third is right for "the wrist's push" and wrong for "the gripper can only".
- **Assumptions:** the lower third is read with the line, and "gripper" is read literally.
- **Competing reading:** "swing" means the first-order sideways response of the wrist. The beat's picture draws only pushes on the wrist, so nothing on screen corrects "only".
- **Smallest fix:** lower third only, no re-narration: "at first, the wrist can move across the arm, never along it".
- **Materiality:** low, zero cost. **Uncertainty:** low on the facts, moderate on how many viewers read the lower third.

**P03-02** (optional; a roboticist's wince, and a word used for two things)
- **Statement:** 11_4: "A pose like this is called a singularity."
- **Why misleading:**
  - The video defines pose as "a position and an orientation together" (01_5) and uses it that way in 10_1, 12_4 ("Its gripper holds one pose"), 13_1, 13_3 and 14_4.
  - 11_4 is the only spoken use of "pose" for the arm's whole joint setting.
  - Singularity is a property of the joint setting, not of the gripper's pose.
- **Evidence:** `check4.py`, `check5.py`.
  - The pose reached at `q=(0,0,0,60,0,0,0)` (the 11_4 lower third's "straight up with the elbow bent") has rank 5.
  - The same gripper pose, p=(−0.456, 0, 1.043) with the same orientation, is reached inside KUKA's limits by joint settings of rank 6.
  - One example: `q=(−20.6, −55.2, 159.4, 60, 20.9, 58.2, −159.1)` deg, smallest singular value 0.16.
- **Possible wrong takeaway:** a singularity is a gripper pose to avoid. That sits oddly beside 12_4's one pose with a whole circle of joint settings.
- **Competing reading:** the arm is on screen, so "a pose like this" reads naturally as the arm's shape. The consequence for Maya is small.
- **Smallest fix:** script 11_4, one word: "An arm shape like this is called a singularity." The fact sheet and brief wording "singular poses" can become "singular joint settings" at no cost.
- **Materiality:** low to medium for Ruth and roboticists, low for Maya. The owner may reasonably decline. **Uncertainty:** moderate.

**P03-03** (HYPOTHESIS; optional)
- **Statement:** 06_4: "Wrap it onto the curved space, and the same three meters bend around a fixed point."
- **Why possibly misleading:**
  - "space" means the room in 05_1 ("In space"), 07_1 and 07_2.
  - It means the abstract set of orientations or poses in 04_5, 05_3, 06_4 and 13_6.
  - In 06_4 a flat, gridded floor has just been shown (06_2, 06_3). The curved space of poses (x, y, heading) is not drawn. What bends on screen is the path.
- **Possible wrong takeaway:** the floor is curved, or the path bends because the surface is curved.
- **Competing reading:** the picture (a line bending at constant length about a gold centre dot) and "bend around a fixed point" point to the path, so the likelier effect is that "curved space" means nothing rather than something false.
- **Smallest fix:** lower third, no re-narration, appended to the existing one: "the curved space here is the space of poses (where, which way), not the floor".
- **Materiality:** low. **Uncertainty:** high; I did not test with viewers.

**P03-04** (HYPOTHESIS; optional; a "the" for a "a")
- **Statement:** 10_1: "The answer starts with speeds."
- **Why possibly misleading:**
  - It is one method, differential inverse kinematics with Newton steps, presented as the answer.
  - The fact sheet row for 13_6 notes that a closed-form solution exists for this arm, but nothing spoken or on screen says other solvers exist, apart from the 13_6 tag "real solvers cap, damp…".
  - The fact sheet's own finding 8 says closed form was not source-checked, so the claim stays on the safe side.
- **Smallest fix:** script 10_1, one word: "One answer starts with speeds."
- **Materiality:** low; the 13_6 tag partly covers it. **Uncertainty:** moderate.

## NULL OR BLOCKED RESULT

Examined and not claimed (what I looked at, and why I dropped each):
- **02_4 "any turn followed by any turn is one turn about one axis":** turns about axes that do not meet give a screw. The lower third in the same beat says "about its centre", and 07_3 and 07_5 give the general case.
- **11_6 "This arm has a third, and it covers":**
  - The risk is that a viewer generalises to every straight wrist. My model gives rank 5 at `(0,90,0,90,90,0,0)`, matching fact-sheet finding 9.
  - The words are scoped by "here", and "directions: 6" stays lit beside the arm. An optional "and here it covers" is not needed.
- **11_3 "can't slide further out… can't even begin to slide straight back":** it could be counted as two lost directions. The single red arrow and "Five of the six" settle it in the beat.
- **12_4 "twenty-nine centimeters" (protected):** the radius is pose-specific. The picture says "L shape", so it is low risk.
- **14_3 "half its volume" (protected):** it is specific to a quarter turn apart. The picture reads "volume 0.50 at halfway" and the lower third states the general rule.
- **14_5 "differ by seven point six centimeters" (protected):**
  - A viewer could read it as one path being 7.6 cm longer, but the lengths differ by 3.0 cm.
  - The beat's picture marks the largest gap, so I dropped it.
- **Slide arrow of a twist vs slide along the screw's axis (07_2, 07_3, 10_3):** storyboard, fact sheet and brief all know it, and the 07_3 lower third and 10_3 tag state it. It stays a dense lower third, but I have no new fix.
- **08_5 order of the sums vs order of motion:** the on-screen tag "the order of the sums, not of the motion" covers it.
- **02_6 "one grid times another":** the viewer may not know the product. That is vocabulary difficulty, not a false belief.
- **12_1 "one joint spare":** no specific joint is spare, and 12_5 shows the use.
- **15_1 "step each body by the exponential, so the body stays rigid":** the picture labels the comparison as "our test".
- **Others with no false belief found:**
  - 03_6 "software stores nine numbers, or four" is not an "only".
  - 01_4 "positions add" is protected and said of these turns.
  - 05_2 "a third answer" is honest.
  - 04_5 "The space is curved" is anchored by the circle picture.

## PER-CHAPTER TAKEAWAY (what a 14-year-old carries away, and whether it is the intended one)

- **01:** Turning a book in two orders ends differently while sliding does not, so a robot's software must cope with things that do not add. Intended. Null; the 01_2 lower third covers the body-fixed vs table-fixed trap.
- **02:** Any two turns make one turn about one axis (here a third of a turn), turns can be undone so they form a group where order usually matters, and a turn is a grid of nine numbers. Intended. Null.
- **03:** Three angles for orientation have a bad spot (two rings do the same job, so a way to turn is lost), which Apollo hit, and software uses nine or four numbers to avoid it. Intended. Null.
- **04:** Averaging angle numbers can point backward at the seam, so measure the short difference on a flat ruler, halve it, and wrap back. Intended. Null.
- **05:** A spin is one arrow, holding it a second is the exponential, adding arrows gives a third answer, and smooth motion groups are Lie groups with the flat piece as the algebra. Intended. Null.
- **06:** A platform at steady velocity travels a circle, so velocity times time is not where it ends up. Intended. See P03-03.
- **07:** A pose changes in six ways, a steady twist is a screw motion, and any rigid move is one screw. Intended. Null.
- **08:** Each joint is a screw, and the pose is a product of seven exponentials taken last joint first from the zero pose. Intended. Null.
- **09:** Elbow, shoulder, then base put the tip 42 cm forward, then 42 cm to the side, with axes taken at zero. Intended. Null.
- **10:** The Jacobian turns joint speeds into the gripper's twist, and the tip's speed depends on distance from each axis. Intended. See P03-04.
- **11:** A straight elbow loses a direction (a singularity), speeds blow up near it, and the wrist is a gimbal that the third shoulder joint covers here. Intended. See P03-01 and P03-02.
- **12:** The spare joint lets the elbow swing on a circle while the gripper holds still. Intended. Null.
- **13:** Take the error as a twist, run the Jacobian backwards, step and repeat; three steps from this start. Intended. Null.
- **14:** Blending matrices breaks orientation, the steady path is the screw, and a straight tip path is also used. Intended. Null.
- **15:** These moves recur in drones, mapping, simulation and humanoids, and the order's leftover is a turn measured by the Lie bracket. Intended. Null.

**Recommendation:** P03-01 is cheap (lower third only) and worth doing. P03-02 to P03-04 are the owner's call. None touches a protected phrase.

