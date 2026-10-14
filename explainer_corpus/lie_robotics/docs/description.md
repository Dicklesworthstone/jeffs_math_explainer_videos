# Why Robot Arms Think in Lie Groups: draft video description

<!-- Draft YouTube description (R2, 2026-10-14). Every number here is an oracle key
(oracle.py) or a fact-sheet row (fact-sheet.md); every source is a fact-sheet row. Chapter
timestamps come from the narrated cut (`./x lie_robotics chapters`). Sources not opened are
listed as not verified. -->

Tilt a book a quarter turn, then roll it a quarter turn, then do it in the other order: the
book faces a different way. Slide it right then away, or away then right: it lands on the same
spot. Positions add; orientations don't. This video follows what a seven-joint robot arm
(a KUKA LBR iiwa 14) has to do about that: turns, the one move that makes them add (do the
arithmetic on the flat piece that touches the curved space, then wrap the answer back on),
screws, a product of seven exponentials, the Jacobian, the directions an arm can lose, its
spare joint, and Newton's method on the group of poses in three steps. The last quiz asks
what is left over when two small turns are made in two orders, and the answer is the Lie
bracket.

## Chapters

(timestamps after the narrated cut)

1. The book
2. One turn
3. Three angles
4. Halfway
5. Spin
6. The platform
7. Screws
8. Seven screws
9. By hand
10. Velocities add
11. Lost directions
12. The spare joint
13. Steps
14. The steady path
15. Everywhere

Names to search: Lie group, Lie algebra, SE(3), SO(3), screw theory (twist, wrench), product of
exponentials, Chasles–Mozzi theorem, the matrix exponential and logarithm, Rodrigues' formula,
the Jacobian (space form, body form), singularities, redundant manipulators and the null space,
Newton–Raphson inverse kinematics, gimbal lock.

Pronunciation: Lie is said "Lee"; Chasles "Shahl"; Mozzi "Motsee"; Rodrigues as the French
surname; KUKA and iiwa as KUKA writes them.

## Precisely

The narration simplifies in places. Here is the exact version of each simplification, by the
beat where it is said (chapter_beat). Notation: R a 3 by 3 turn matrix, [ω] the matrix of
ω × (the cross product with a unit axis ω), T a pose, S a screw axis, q the joint angles.

- **01_1, 01_2.** The two quarter turns are made about lines fixed on the table: tilt about the
  line left to right (the far edge goes up), roll about the line pointing away (the cover goes
  to the right), the cover starting up. Then "tilt first" leaves the cover facing you and "roll
  first" leaves it facing the wall on your right. With the opposite sense of a turn, the cover
  faces another wall (the facing after tilting depends only on the tilt's sense, the facing
  after rolling only on the roll's); the two orders still differ for every choice of senses.
  About the book's own edges instead of the table's lines, the two outcomes are exchanged.
- **01_4.** "Orientations don't" is said of these two turns. Turns about one axis do swap, and
  so do two half turns about perpendicular axes (02_5). For all turns, two turns swap exactly
  when one is no turn, or both are about the same line, or both are half turns about
  perpendicular lines: every other pair, which is almost every pair, does not.
- **01_6.** "About a hundred and fifty years old": Sophus Lie began his theory of continuous
  groups in the winter of 1873–74. Writing an arm's pose as a product of exponentials is
  Brockett's (08_6, 1983); it rests on Lie's groups and on screw theory.
- **02_4.** Euler's theorem: every turn of space fixes an axis, and a product of turns is a
  turn. The turn may be the zero turn.
- **02_5.** A group is closed, associative, has an identity and has inverses. "Usually": for
  turns of space the pairs that swap form a set of measure zero (see 01_4).
- **03_4.** The thresholds (lamp above 70°, the reference dropped above 85°) are those of the
  command module computer's flight software (Comanche 055, routine GLOCKMON). The bar of the
  picture, the volume the three ring axes span, is this video's own measure: cos of the middle
  angle, 34% at 70° and 9% at 85°.
- **03_5.** Collins's line was a joke. The capcom answered "You were unreadable. Say again
  please." and Collins said "Disregard." A fourth ring makes one ring angle redundant, as the
  seventh joint of an arm does; the extra ring helps only if its angle is driven to use that
  freedom.
- **03_6.** A theorem: no continuous one-to-one map sends all orientations into three numbers,
  so any naming by three numbers is many-to-one or jumps somewhere. For nested rings, all
  twelve orders of three axes lose a direction at some middle angle (90° for the six orders
  yaw, pitch, roll type; 0° or 180° for the six that repeat an axis). Nine numbers (a matrix,
  with six rules) and four (a unit quaternion, with one rule) have no bad spot; a quaternion and
  its negative are the same orientation.
- **04_3, 04_4.** "Halfway" means the middle of the shorter way round. On a circle two headings
  have two midpoints: 0° is the middle of the 340° way round and 180° the middle of the 20° way.
  180° is also the direction of the sum of the two unit heading vectors, and the point nearest
  to both (10° from each). Two exactly opposite headings have two halfways. In code, wrap the
  difference with `atan2(sin d, cos d)`: in C, C++ and Java `%` and `fmod` keep the sign of the
  dividend, so `fmod(-340 + 180, 360) - 180` is -340, unchanged, where Python's `%` gives +20.
- **04_5.** "The space is curved": a circle bends away from its flat piece (the tangent line)
  and closes up; as a line it has no curvature of its own. The space of orientations is curved in
  the strict sense. For poses it means that steady motions are not straight lines in pose
  coordinates.
- **05_1, 05_3.** A spin is an angular velocity: a unit axis times a speed in radians a second.
  The exponential is `e^{[ω]t} = I + sin t [ω] + (1 − cos t)[ω]²` for a unit axis ω and an
  angle t in radians (with t = 0 it is the identity; the axis is only defined for a nonzero
  spin). The name is this move's, from the flat piece back to the curved space (for a spin it is
  the exponential of a matrix); it has nothing to do with growth. The logarithm goes back, with a
  branch of its own near a half turn.
- **05_2.** Adding two spins and holding for a second turns about both at once (127.28° about
  (1, 1, 0)/√2 for the two quarter turns), which is neither order's result. Near doing nothing
  arrows and orientations match one to one; far away they fold (a half turn has two arrows).
- **05_6.** The formula is the space form of Rodrigues' rotation formula; the paper (1840)
  works out how turns combine and says that their order matters. "Named after" is all that is
  claimed for where the formula sits in it.
- **05_7.** A Lie group is a smooth manifold whose multiplication and inverse are smooth. Its Lie
  algebra is the tangent space at the identity with a bracket (15_4).
- **06.** The platform's velocity is the platform's own (forward and sideways read in its frame),
  held constant: the pose after six seconds is the exponential of the 3 by 3 velocity matrix times
  six seconds. The numbers (half a meter a second, 30° a second) are this video's, not KUKA's:
  KUKA publishes 3.6 km/h forward, 2.0 km/h sideways and no turning rate. Wheel senses (front
  left, front right, rear left, rear right; Lynch and Park eq. 13.10): forward + + + +, sideways
  − + + −, spin − + − +. "Add six seconds of velocity onto the pose" is one big step on the
  numbers: (3, 0) and a heading of 180°. Stepping in small pieces and turning the velocity by the
  heading each time is the wrap done piece by piece: 0.01 s steps land within 5 mm of the exact
  (0, 1.91).
- **07_2, 07_3.** A twist's slide arrow, in the room's frame, is the velocity of the point at
  the room's origin: not the tip's and not the slide along the screw's axis. The tip's velocity is
  slide + spin × tip position.
- **07_5.** Chasles–Mozzi: every rigid displacement is a screw displacement. With no turn the
  screw is a pure slide and its axis is only a direction. At an exact half turn the axis point is
  half the part of the slide across the axis and only the sense of the axis is free; for a very
  small turn the axis lies far away.
- **08_1, 08_2.** Turning about a line through c: x goes to c + R(x − c).
- **08_6.** The formula is the space form (Lynch and Park eq. 4.14), `T(q) = e^{[S₁]q₁} ⋯ e^{[S₇]q₇} M`.
  "Hold each joint's twist for its joint angle": S is a unit twist, one radian a second, held
  for q seconds with q in radians. Brockett presented it at MTNS-83 (Beer Sheva); the
  proceedings are dated 1984.
- **09_1.** The gripper's tip is the flange: no gripper is in the model.
- **10_2, 10_3.** The Jacobian is the space Jacobian (eq. 5.11): column i is joint i's screw
  carried to the present pose, (ω, −ω × c) with ω the joint's unit axis and c any point on it,
  both as the arm stands now. Conventions differ between libraries: whether the spin rows or the
  slide rows come first, and whether the slide is the origin's velocity or the tip's own. Check
  your library's before comparing numbers.
- **11_1.** "Directions" is the rank of the Jacobian.
- **11_3.** At q = (0, 60°, 0, 0, 0, 60°, 0) the rank is 5. No joint moves the wrist centre along
  the line from shoulder to wrist; the gripper's tip, 0.126 m past the wrist centre, can still
  move along it while turning (1 m/s along it costs 9.2 rad/s of turning), so what is lost is a
  pure slide of the gripper. With the wrist straight the tip lies on the line and cannot.
- **11_4.** Singularity is a property of the joint angles, not of the gripper's pose: the same
  gripper pose can be reached by a singular and by a non-singular set of angles (a witness: the
  pose with the tip at (−0.526, 0, 0.780) m and R = [0 0 −1; 0 1 0; 1 0 0] is reached at rank 5
  by (0, 0, 0, 90°, 0, 0, 0) and at rank 6 by (−111.8°, 39.0°, 68.2°, 90°, −69.1°, 41.0°,
  110.9°), inside KUKA's limits).
  Singular settings are also found well inside the reach.
- **11_5.** 54 and 160 °/s are for the elbow, for a pure pull of the gripper at 0.1 m/s with the
  gripper's orientation held; they are the same however the other joints share the motion. The
  limit, 75 °/s, is KUKA's published speed for axis 4 at rated payload (14 kg), a data-sheet
  figure and not a law. The crossing is at a bend of 21.5°.
- **11_6.** In the L shape rank J = 6 with joints 5 and 7 collinear; without any one of joints 1
  to 3 it is 5, without joint 7 (or 5) still 6. Not in every pose with a straight wrist: at
  (0, 90°, 0, 90°, 90°, 0, 0) the rank is 5. This wrist's axes go z, y, z, so its bad spot is the
  middle joint at zero; the rings' z, y, x had theirs at a quarter turn.
- **12_3.** The usual kinematic model: a ball joint at the shoulder (3 hinges), a hinge at the
  elbow, three at the wrist. The palm on the desk is Lynch and Park's Exercise 2.2; their
  Exercise 2.3 notes the forearm is really a closed chain.
- **12_5.** What the freedom is used for in practice: getting round obstacles, staying off joint
  limits, and staying away from singular settings.
- **13_2.** The orientations (0°, 89°, 0°) and (90°, 89°, 90°), as ring angles in the drawn order
  (outer about the vertical, middle about the line pointing away, inner about the line left to
  right), are 1.414° apart. In the other nesting order the same angles are 179.99° apart.
- **13_3, 13_4.** The method is Lynch and Park §6.2.2 with the error read in the gripper's frame
  and the pseudoinverse of the body Jacobian: `error = log(inverse(T(q)) · T_goal)`,
  `q = q + pinv(J_body(q)) · error`, stop below 0.001 rad and 0.1 mm. J_body is the Jacobian of
  chapter 10 with each twist read in the gripper's frame. With the space form of both (error
  `log(T_goal · inverse(T(q)))` and `J(q)`) the iterates are the same; with one of each the loop
  does not converge. In the space form the slide part of the error is the room origin's remaining
  distance, not the tip's, so "0.1 mm" means the tip's distance only in the body form. Near a
  half turn the logarithm needs its own branch.
- **13_6.** Newton's method: from a guess, measure the miss, correct by it, repeat. "From here, three": from q = (0, 30°, 0, −60°, 0, 90°, 0) to the goal (0.400, 0.300,
  0.250) m with R = [0 −1 0; −1 0 0; 0 0 −1]. From other starts the count differs (5, 13, 10 and
  4 steps in four others tried) and the method can fail. For a chosen swing of the elbow this
  arm's geometry (a spherical shoulder, an elbow, a spherical wrist, no offsets) gives a closed
  form; the video makes no claim about it. Each step cuts the error by a bigger factor than the
  last (5.4, 22.6, 157 for the distance), while the absolute gain falls (411, 90, 4.1 mm).
- **14_2, 14_3.** The determinant of a blend `(1 − s)R₀ + sR₁` of two turns that differ by the
  angle a is 1 − 2s(1 − s)(1 − cos a); at s = ½ it is cos²(a/2). For a quarter turn it is 0.5. The
  blend is the "lerp" of matrices; renormalising it ("nlerp") repairs the shape and not the
  timing: a quarter of the way it has turned 18.4°, not 22.5°; the screw path's turn is the
  "slerp" one.
- **14_4, 14_5.** "Steady" means one constant twist. A path that keeps the tip on a straight line
  and the turn on its own screw (the decoupled path, Lynch and Park eqs. 9.7 and 9.8) is common
  in practice. At exactly a half turn the twist that joins two poses is not unique.
- **15_1.** Adding dt [ω] R to R is a forward Euler step on the nine numbers; it swells the body
  (volume 1.1052, 5.1% wider across the axis after 1,000 steps of 0.01 s at one radian a second).
  The exponential step does not. A unit quaternion renormalised each step also stays rigid. A
  humanoid is one robot here, the Unitree G1: 29 revolute joints, so 6 + 29 = 35 numbers of
  velocity and 3 + 4 + 29 = 36 of stored state (a position, a quaternion, the joints).
- **15_3, 15_4.** With T the tilt-then-roll turn and U the roll-then-tilt turn, both about lines
  fixed in the room (tilt about x, roll about y), the leftover is T U⁻¹. For turns of angles a
  and b it is a turn by ab + O(3) radians about an axis that tends to ŷ × x̂ = −ẑ: 0.0175° at 1°
  (0.7° off −z), 1.741° at 10° (7° off), 120° at 90°. The Lie bracket is the corresponding rule
  on the flat piece, the cross product: [y, x] = −z (x is tilt's line, y roll's, z up).

## Numbers to test with

The arm: KUKA LBR iiwa 14 R820, every joint turning about its own +z after the roll of its joint
origin. Joint origins along z (m): 0.1575, 0.2025, 0.2045, 0.2155, 0.1845, 0.2155, 0.081, 0.045.
Axes with the arm at zero, and their heights (m): 1 (0, 0, 1) at 0.1575; 2 (0, 1, 0) at 0.360;
3 (0, 0, 1) at 0.5645; 4 (0, −1, 0) at 0.780; 5 (0, 0, 1) at 0.9645; 6 (0, 1, 0) at 1.180;
7 (0, 0, 1) at 1.261; the flange at 1.306 m. Limits ±170°, ±120°, ±170°, ±120°, ±170°, ±120°,
±175°; speeds with rated payload 85, 85, 100, 75, 130, 135, 135 °/s.

| Pose | Joint angles (degrees) | Tip (m) | Rank |
|---|---|---|---|
| home | 0, 0, 0, 0, 0, 0, 0 | (0, 0, 1.306) | 3 |
| after the elbow's quarter turn (09_2) | 0, 0, 0, 90, 0, 0, 0 | (−0.526, 0, 0.780) | 5 |
| the L shape (09_3) | 0, 90, 0, 90, 0, 0, 0 | (0.420, 0, 0.886) | 6 |
| after joint 1 (09_5) | 90, 90, 0, 90, 0, 0, 0 | (0, 0.420, 0.886) | 6 |
| the quiz pose (11_2) | 0, 60, 0, 0, 0, 60, 0 | (0.819, 0, 0.707) | 5 |
| the L shape, joint 5 turned | 0, 90, 0, 90, 90, 0, 0 | | 5 |
| the Newton start (13_5) | 0, 30, 0, −60, 0, 90, 0 | (0.610, 0, 0.598) | |

In the L shape the levers (distance from the tip to each axis) are 0.420, 0.673, 0.526, 0.526,
0, 0.126, 0 m, and joint 2's column is (0, 1, 0, −0.36, 0, 0), spin first. The Newton run's tip
distances are 504.99, 93.93, 4.15, 0.03 mm and its orientation errors 90°, 8.64°, 0.724°, 0.0076°;
it ends at q = (24.89°, 38.57°, 15.58°, −104.86°, −15.56°, 38.65°, −40.55°), inside the limits.
The screw between the start and the goal turns 90° about the vertical through (0.355, 0.045) and
slides −0.348 m; the tip's straight line and the screw's path part by 0.0758 m at most.

## Code

Every number was computed by code and checked by a second program: `oracle.py` (independent of
the model that drew the scenes) and the video's own model. The loop of 13_4 is the whole method:

```
error = log(inverse(T(q)) * T_goal)       # a twist, read in the gripper's frame
q     = q + pinv(J_body(q)) * error       # the smallest joint step that makes it
repeat until error < 0.001 rad and 0.1 mm
```

The physics-engine remark of 15_1 refers to FrankenSim's `crates/fs-time/src/se3.rs`, function
`se3_exp_step` (`T_next = T * Exp(h * twist_body)`), github.com/Dicklesworthstone/frankensim.

## Sources

- K. M. Lynch and F. C. Park, *Modern Robotics: Mechanics, Planning, and Control* (Cambridge
  University Press, 2017; the authors' preprint, May 3, 2017, whose page numbers are used): Def.
  3.1 p. 70 and its footnote ("Lie" is pronounced "Lee"); Prop. 3.5 p. 71; Prop. 3.11, eq. (3.51),
  p. 84 (Rodrigues' formula); eqs. (3.70), (3.88), (3.90)–(3.92), pp. 98–106 (twists, the screw
  exponential and logarithm); §3.3.3.1 p. 105 (Chasles–Mozzi); eq. (4.14) p. 142 (the space form);
  §5.1.6 pp. 189–190 (redundancy); eqs. (5.9)–(5.11) pp. 179–180 (the space Jacobian); §6.2.2
  pp. 227–232 and Example 6.1 (numerical inverse kinematics, the tolerances); §9.2.1 pp. 326–328
  (eqs. 9.6 to 9.8); eq. (13.10) p. 519 (mecanum wheels); Exercises 2.2 and 2.3 p. 38.
- R. M. Murray, Z. Li and S. S. Sastry, *A Mathematical Introduction to Robotic Manipulation* (CRC
  Press, 1994), Theorem 2.11 p. 49 (Chasles), eq. (3.3) p. 87 (product of exponentials).
- R. W. Brockett, "Robotic manipulators and the product of exponentials formula", in
  *Mathematical Theory of Networks and Systems* (MTNS-83, Beer Sheva, 1983), Lecture Notes in
  Control and Information Sciences 58, Springer, 1984, pp. 120–129, doi:10.1007/BFb0031048.
- J. Solà, J. Deray and D. Atchuthan, "A micro Lie theory for state estimation in robotics",
  arXiv:1812.01537 (2021), §II.
- L. Euler, "Formulae generales pro translatione quacunque corporum rigidorum", E478, presented
  9 October 1775, *Novi Commentarii Academiae Scientiarum Petropolitanae* 20 (1776), pp. 189–207.
- O. Rodrigues, "Des lois géométriques qui régissent les déplacements d'un système solide dans
  l'espace…", *Journal de Mathématiques Pures et Appliquées* 5 (1840), pp. 380–440 (p. 391, on
  the order of rotations; p. 386, footnote, the attribution to Chasles).
- M. Chasles, "Note sur les propriétés générales du système de deux corps semblables entr'eux",
  *Bulletin des Sciences Mathématiques, Astronomiques, Physiques et Chimiques* 14 (1830),
  pp. 321–326.
- G. Mozzi, *Discorso matematico sopra il rotamento momentaneo dei corpi* (Naples, 1763); M.
  Ceccarelli, "Screw axis defined by Giulio Mozzi in 1763 and early studies on helicoidal motion",
  *Mechanism and Machine Theory* 35 (2000), pp. 761–770.
- S. Lie: MacTutor, "Sophus Lie": the winter of 1873–74.
- D. Hoag, MIT Instrumentation Laboratory report E-1344 (1963); the Apollo flight software
  Comanche 055 (routine GLOCKMON, listing p. 147); NASA, *Apollo 11 Technical Air-to-Ground Voice
  Transcription*, tape 67/11, p. 337 (04 09 00 30 and 04 09 00 40 to 04 09 00 46); E. M. Jones
  and P. Fjeld, "Gimbal Angles, Gimbal Lock, and a Fourth Gimbal for Christmas", Apollo Lunar
  Surface Journal (2000).
- KUKA, data sheet "LBR iiwa 14 R820" (0000-246-833, V1.2, 01.06.2022), p. 1: reach 820 mm,
  rated payload 14 kg, 7 axes, ranges and speeds; KUKA, "KMR iiwa" (revision 01.05.2024), p. 2: 3.6
  km/h longitudinal, 2.0 km/h lateral, Mecanum wheels.
- Joint origins: aica-technology/kuka-iiwa-description, `robots/iiwa14.xacro`, tag v0.1.0
  (GPL-3.0); nothing is copied. The humanoid: unitreerobotics/unitree_ros,
  `robots/g1_description/g1_29dof.urdf` (BSD-3-Clause), 29 revolute joints.
- Where the mathematics is used (15_1): `manif` (a Lie theory library for state estimation);
  Pinocchio (Carpentier et al., 2019, free-floating joints for mobile robots and humanoids);
  T. Lee, M. Leok and N. H. McClamroch, "Geometric tracking control of a quadrotor UAV on SE(3)",
  CDC 2010, pp. 5420–5425; GTSAM (`Pose3`, "3D Pose manifold SO(3) x R^3 and group SE(3)").

## Not verified

The hardback's page numbers; the texts of Brockett (1984), Ceccarelli (2000), Chasles (1830) and
Mozzi (1763), which were not opened; where a formula equal to Rodrigues' sits in the 1840 paper;
how KUKA pronounces "iiwa"; the rate at which KUKA's controller runs; the current KUKA
specification behind the login (the data sheet figures above are from the 2022 sheet and the
2015 specification, which agree). The 29 joints and the 35 and 36 are counts of the file and
arithmetic. The remarks on what library or engine authors do (conventions, quaternions) are not
claims about a particular library.
