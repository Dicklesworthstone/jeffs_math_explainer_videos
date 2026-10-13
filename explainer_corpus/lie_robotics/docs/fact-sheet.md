# Fact sheet: Video G, "Why Robot Arms Think in Lie Groups"

Bead `mx-e-a-suh.2` (gate G1). One row per number the video may show or
say, and one row per claim that rests on a source. Every value in the tables
at the end was copied from the oracle's output by a script, not typed.

```
./x lie_robotics facts-check        # lie facts == oracle.py, key by key
```

- **Result, 2026-10-13** (the build host; CPython 3.14, NumPy 2.5.4):
  `facts-check lie_robotics: 227 video facts against 227 oracle keys: PASS`.
  The oracle prints 227 keys and passes 11 of 11 of its own cross-checks, 14
  of 14 with SciPy 1.18.1 present, 18 of 18 with SciPy and the owner's
  source files present (`CMAES_EXPLAINER_DIR`).
- **Rust side:** `./x lie_robotics test`: 21 tests pass, clippy
  `-D warnings` and `fmt --check` clean.
- **Rows:** G-A the arm; G-B the book; G-E three angles; G-H halfway; G-R
  spins; G-P the base; G-S screws; G-K the arm's poses; G-J velocities; G-I
  steps and paths; G-C the coda; G-X the edges of the always sentences, each
  as "how many hold of how many checked"; G-Q the quizzes; G-L cited
  literals.

## How the two sides get their numbers

| What | `src/model.rs` (the video) | `oracle.py` (independent) |
|---|---|---|
| a turn about an axis | Rodrigues' formula, I + sin a K + (1 − cos a) K² | the three elementary matrices typed out; any other axis by a generic scaled Taylor series for the matrix exponential, which knows nothing about turns |
| the angle and axis of a turn | the trace and the antisymmetric part | a generic matrix logarithm (inverse scaling and squaring with Denman–Beavers square roots, then the series of log(I + X)) |
| a twist held for a second, and back | the screw exponential and logarithm in closed form (Lynch and Park eqs. 3.88, 3.91, 3.92) | the same generic exponential and logarithm on 4 by 4 matrices |
| the screw of a pose | from the logarithm: axis, the point `w × v`, the slide `(w · v) a` | from the generic logarithm; rebuilt from typed-out pieces (slide to the axis, turn, slide back) with no exponential |
| the arm's pose | the product of exponentials (eq. 4.14), the screws read off once at zero | the URDF chain walked joint by joint: origin, roll pitch yaw, then the joint's turn about its own z |
| the Jacobian | analytic: each joint's screw carried to the present pose (eq. 5.11) | central differences of the chain |
| how many directions (rank) | Gram–Schmidt on the columns | NumPy's singular values |
| each joint's push on the wrist along the arm | the joint's axis crossed with the lever to the wrist | each joint moved a little, the wrist watched |
| the tip's lever about each joint | from the Jacobian's column | the distance from the tip to the joint's axis line |
| the elbow's speed near full stretch | the smallest joint speeds that give a pure pull, Jᵀ(JJᵀ)⁻¹ | the triangle shoulder, elbow, wrist: the law of cosines differentiated |
| the elbow's circle | the elbow's distance from the line shoulder to wrist | Heron's formula for the triangle's height |
| joint speeds that move nothing | the 6 by 6 minors of the Jacobian | the last right singular vector |
| Newton's steps | its own elimination for the pseudoinverse, analytic body Jacobian, closed-form logarithm | `numpy.linalg.pinv`, finite-difference Jacobian, generic logarithm |
| the base on the floor | closed form (sines and cosines about the centre) | the generic exponential of the 3 by 3 velocity matrix, and 60,000 small steps |
| wheel senses | Lynch and Park eq. 13.10 | the rolling condition wheel by wheel (eqs. 13.4 to 13.6) |
| the third direction left at a ring angle | cos of the angle | the volume spanned by the three ring axes (a determinant) |
| the twelve orders of three rings | Rodrigues turns, its own determinant | typed-out matrices, NumPy's determinant |
| a blend repaired to the nearest turn | closed form, atan2(s sin a, 1 − s + s cos a) | the polar part from a singular value decomposition |
| a cube's 24 turns | grown from the two quarter turns as whole-number matrices | the same closure, from the typed-out matrices |
| cited numbers | constants in `facts::cited` and `model::LIMITS_DEG`, `SPEED_DEG_S`, pinned where the video's own arithmetic reaches them (KUKA's 820 mm reach is the upper arm plus the forearm) | the same literals, typed from the sources |

Every decimal is printed by one rule on both sides (fixed places, never a
negative zero). Where a value sits near a rounding boundary the number of
places was chosen so that neither side can round the other way (the Newton
distances are printed to 0.01 mm for that reason).

## Findings (uncomfortable first)

1. **`vendor/iiwa14_kinematics.json` does not describe the arm.** It turns
   every ±π/2 roll of the xacro's joint origins into 0, which makes all
   seven joint axes parallel. The video uses the xacro only, and the
   oracle re-reads the xacro when the owner's checkout is present.
2. **The xacro is GPL-3.0.** It is byte-identical (SHA-256
   `242ae014…d7dcc2`) to `robots/iiwa14.xacro` in
   aica-technology/kuka-iiwa-description, tag v0.1.0; that repository names
   no copyright holder. The same joint-origin strings are in
   epfl-lasa/iiwa_ros (GPL-3.0-or-later) and IFL-CAMP/iiwa_stack (BSD
   2-Clause, © Virga and Esposito, TUM). Nothing from any of them is copied
   here: the model types eight lengths and seven sets of three angles, with
   the path cited.
3. **The base's numbers in the owner's project are not KUKA's current
   ones.** `kmrGeometry.ts` cites KUKA revision `904d9560…` but its
   1.19 × 0.72 × 0.70 m, 375 kg and 175 kg are the 2016 brochure's; the
   cited revision says 1,080 × 630 × 700 mm, 390 kg, 170 kg. The wheelbase
   and wheel size there are that project's own assumptions (KUKA publishes
   a 250 mm wheel, the file uses 150 mm). The limits "1.5 m/s" and
   "1.0 rad/s" in `mecanumKinematics.ts` appear in no KUKA revision opened;
   KUKA publishes 3.6 km/h forward, 2.0 km/h sideways and no turning rate.
   The video shows no chassis dimension, runs the base at 0.5 m/s (half the
   published top speed) and labels the grid, speeds and paths "model".
4. **The wheel formula's citation in the owner's file cannot be found.**
   "Killpack, A Brief Overview of the Omnidirectional WMR (Mecanum Wheel),
   Carnegie Mellon University, 2012" turned up in no search. The formula
   itself is right: it equals Lynch and Park eq. (13.10) term for term, and
   the video cites that.
5. **Where Rodrigues' formula sits in Rodrigues's own paper was not
   found.** The 1840 paper works out how turns combine (§8, pp. 390–391)
   and says outright that their order matters; a formula that can be
   matched line by line with the modern one was not located. The lower
   third says "named for Olinde Rodrigues", not "Rodrigues, 1840". MacTutor
   prints the wrong journal (it is Liouville's *Journal de Mathématiques
   Pures et Appliquées* 5, 380–440).
6. **Lynch and Park never say "gimbal lock".** The phrase and the numbers
   are NASA's. The page often cited for the Collins quote ("Gimbal Angles,
   Gimbal Lock, and a Fourth Gimbal for Christmas") is by Eric M. Jones and
   Paul Fjeld (2000), not David Hoag; Hoag's memo (MIT Instrumentation
   Laboratory E-1344, 1963) is quoted on it.
7. **Newton's method is not always this well behaved.** From the start pose
   the chosen goal takes 3 steps. From a poorer start the same method took
   up to 22 steps and left the joint limits in the lane's exploration; the
   script says "for this start and this goal" in the fact sheet's terms and
   the narration claims only the run shown. A first version of the oracle
   used a logarithm that dropped the slide along the axis (an eigenvalue
   method on a matrix that cannot be diagonalized) and needed 7 to 22
   steps; the fix is the generic logarithm described above, checked against
   SciPy.
8. **Whether this arm has a closed-form inverse kinematics was not checked
   from a source.** Designs with a shoulder, elbow and spherical wrist are
   believed to have one, but no paper on it was opened, so the video makes
   no claim about it (the claim walk removed it from 13_6's tag).
9. **A straight wrist does not always cost this arm a direction, and
   sometimes does.** With joint 6 at zero, joints 5 and 7 share an axis
   (the gimbal of chapter 03). In the L shape the Jacobian still has rank
   6 (`G-J.rank.ell`): the spare joint covers. With joint 5 a quarter turn
   round, (0, 90°, 0, 90°, 90°, 0, 0), it has rank 5
   (`G-J.rank.ell_wrist_turned`; Ruth found this in R1). The narration says
   only that "a six-joint arm would lose a direction here" and that "the
   seventh joint covers for it", of the pose on screen.
10. **The Unitree file the owner's project vendors is marked Deprecated
    upstream**, and that project's pages cite a different file
    (`g1_29dof_mode_11.urdf`). Both have 29 revolute joints; the video uses
    only that count and the root link's name.
11. **The length is over the default.** 16.5 min by the G2 estimate
    against "12–16 min" (19.4 at the first draft; 16.1 before R1, whose
    fixes added words). Fifteen countdowns are 1.2 min of it. The corpus's
    three published `michael` cuts ran 4 to 5% longer than their G2
    estimates (15.2 → 15:43, 17.3 → 18:12, 20.0 → 21:04), so the real cut
    will be near 17 min. The next cuts, if the owner wants the default
    met, are the Apollo beats (03_4, 03_5) and 14_5.
12. **R1 caught three things the first sheet had wrong or missing.** The
    gimbal was drawn with its rings in another order than the model's
    (for the drawn order the two orientations of 13_2 are 180° apart, not
    1.4°; the picture now follows `G-E.rings.order`). The quiz pose of
    chapter 11 was the fully stretched arm, which loses two directions
    while the words explained one; it is now the pose with only the elbow
    straight (rank 5, one lost direction, a slide along the arm). And the
    first palette's five closest pairs were under the ΔE2000 20 the brief
    claimed (16.0 at worst); the palette was replaced and a test computes
    it.
13. **A twist's slide arrow is easy to misread.** In the room's frame it
    is the velocity of the point at the room's origin, not of the tip and
    not the slide along the screw's axis. Read naively, the Jacobian of
    10_3 would answer the quiz of 10_4 backwards (Dev, R1). The tag at
    10_3 and the lower third at 07_3 say what it is; the levers of 10_5
    are computed as the tip's own speed.

## Terms (the jargon ledger)

| Term | First said | As spoken | On screen |
|---|---|---|---|
| pose | 01_5 | "a position and an orientation together" | pose = both |
| group | 02_5 | "a collection where you can combine any two, and undo any one" | group; "precisely: and combining is associative" |
| matrix, grid | 02_6 | "nine numbers in a grid, a matrix" | the 3 by 3 grid; rotation matrix (03_6) |
| gimbal, gimbal lock | 03_1, 03_3 | "three rings, one inside the next"; "three rings now give only two ways to turn" | the rings; "ways to turn: 2" |
| the flat piece | 04_5 | "the flat piece that touches it" | "the flat piece: the tangent space"; the badge |
| spin | 05_1 | "a speed, held for one second … one arrow, along its axis, as long as its speed" | the orange arrow |
| the exponential | 05_3 | "that step, from the flat piece back to the curved space" | `e^{[ω]t}` |
| Lie group, Lie algebra | 05_7 | "a group of motions that is smooth like this"; "its flat piece" | the two names on the badge |
| twist | 07_2 | "two arrows, a slide and a spin: six numbers" | twist; the four-word card |
| screw | 07_3 | "turns about an axis, and slides along that same axis" | the gold axis and helix; the four-word card |
| Jacobian | 10_3 | "a grid, six rows by seven columns" | `V = J(q) q̇`; "the map from joint speeds to the gripper's twist" |
| direction | 11_1 | "six independent directions for the gripper" | "directions: 6" |
| singularity | 11_4 | "a pose like this" (a direction is lost) | singularity |
| spare joint | 12_1 | "Seven joints for six directions leaves one joint spare." | 7 − 6 = 1; "the null space of the Jacobian" |
| leftover | 15_3 | "that leftover is itself a turn" | the wedge between the two books |
| the Lie bracket | 15_4 | "a rule of its own" that the leftover follows on the flat piece | "[x, y] = z" |

Used on screen only: rotation matrix, unit quaternion, tangent space, null
space, the logarithm, product of exponentials, SE(3), commutator.

## Simplification → exact

| Line | Said | Exact |
|---|---|---|
| 01_4 | "So positions add, and orientations don't." | Translations form the commutative group R³; the turns of space, SO(3), do not commute in general (Lynch and Park Prop. 3.5). These two turns do not: `G-B.turns.same` = 0. About the book's own edges instead of the table's lines the two outcomes are exchanged. |
| 02_4 | "Any turn followed by any turn is one turn about one axis." | Euler's theorem: every element of SO(3) fixes an axis; SO(3) is closed under products. The turn may be the zero turn. |
| 02_5 | "combine any two, and undo any one"; "order usually matters" | A group: closed, associative, with an identity and inverses. Of the 576 ordered pairs of a cube's 24 turns, 120 commute (24 times the 5 conjugacy classes). |
| 03_6 | "Name every orientation with only three, and somewhere there is a bad spot." | Theorem: no continuous one-to-one map sends all of SO(3) into R³. SO(3) is a compact 3-manifold without boundary, so by invariance of domain its image would be an open and compact subset of R³, which does not exist. Hence any three-number naming is many-to-one or discontinuous somewhere. For angles of nested rings the failure is a lost direction: all twelve orders have one (`G-X6.ring_orders`). Lynch and Park, footnote 4, p. 28: three-parameter representations "are subject to singularities". |
| 03_6 | "the extra ones buy a naming with no bad spot" | Nine numbers with six constraints (R Rᵀ = I, det = 1), or four with one (unit length; q and −q are the same orientation: a double cover). Both are free of singularities; neither is a set of independent coordinates. |
| 04_3 | "The true halfway is a hundred and eighty." | The midpoint of the shorter arc between the two headings; unique because they are not opposite. |
| 04_5 | "Do the arithmetic on the flat piece that touches it." | On a circle the tangent line's arithmetic is exact for any step under a half turn. In space it is exact along a single axis and first-order otherwise, which is why chapter 13 iterates. |
| 05_2 | "Add these two spins, and you get both turns at once, a third answer." | so(3) is a vector space. exp(a + b) is the turn about both at once: for the two quarter turns, 127.28° about (1, 1, 0)/√2 (`G-B.both_at_once`), neither exp(a) exp(b) nor exp(b) exp(a). The exponential is onto but folds over: turns of π about opposite axes agree. |
| 05_3 | "from the flat piece back to the curved space" | exp: so(3) → SO(3), `e^{[ω]t}` with ω a unit axis and t the angle in radians. |
| 05_7 | "A group of motions that is smooth like this" | A Lie group: a smooth manifold with smooth group operations (Solà et al., §II-A). Its Lie algebra is the tangent space at the identity. |
| 06_4 | "the same three meters bend around a fixed point" (lower third: every steady motion is a turn about a point, or a slide) | Every one-parameter subgroup of SE(2) is a rotation about a fixed point or a translation; the path length is speed times time either way (`G-P.arc_length`). Tested on 45 velocities (`G-X5.floor`). |
| 07_2 | "Three slides and three turns." | SE(3) is six-dimensional: a pose can change in six independent ways. (Naming every pose needs more than six numbers, by 03_6.) |
| 07_5 | "One screw would have carried it there, every time." | The Chasles–Mozzi theorem: every rigid displacement is a screw displacement; with no turn the screw is a pure slide and its axis is only a direction. Tested on 264 different poses (`G-X3.one_screw`). |
| 08_6 | "a product of seven exponentials" | The space form of the product of exponentials, `T(q) = e^{[S1]q1} ⋯ e^{[S7]q7} M` (Lynch and Park eq. 4.14). |
| 10_2 | "twists live on the flat piece, so they add" | The spatial twist is linear in the joint velocities: `V = J(q) q̇` (eq. 5.10). Its slide part is the velocity of the point at the room's origin. |
| 11_3 | "Five of the six directions are left." | rank J = 5 at q = (0, 60°, 0, 0, 0, 60°, 0). No joint moves the wrist along the line from shoulder to wrist (`G-J.elbow_straight.push_along_arm` = 0), so the gripper cannot translate along it without turning. |
| 11_5 | "it must turn at fifty-four degrees per second" | The elbow's rate for the wrist to approach the shoulder at 0.1 m/s with the gripper not turning: `v d / (a b sin ε)` with a = 0.42, b = 0.40, d the shoulder to wrist distance and ε the bend. Kinematics only. It meets the 75°/s limit at ε = 21.5°. |
| 11_6 | "A six-joint arm, with two joints at the shoulder, would lose a direction here. This arm has a third, and it covers." | In the L shape rank J = 6 with joints 5 and 7 collinear; without any one of joints 1 to 3 it is 5 (`G-J.rank.ell.without_3`), without joint 7 still 6 (`G-J.rank.ell.without_7`). Not in every pose with a straight wrist: `G-J.rank.ell_wrist_turned` = 5. |
| 12_3 | "your arm has seven joints too" | The usual kinematic model: a ball joint at the shoulder (3), a hinge at the elbow (1), three at the wrist. Lynch and Park, Exercises 2.2 and 2.3. |
| 13_6 | "From here, three." | Newton–Raphson on SE(3) (Lynch and Park §6.2.2) from q = (0, 30°, 0, −60°, 0, 90°, 0) to the stated goal, stopping at 0.001 rad and 10⁻⁴ m. From other starts the count differs and the method can fail. A closed-form solution exists for this arm. |
| 13_2 | "Yet two of their three ring angles differ by ninety." | ZYX angles (0, 89°, 0) and (90°, 89°, 90°): the orientations are 1.414° apart. |
| 14_3 | "squashed to half its volume" (lower third: a blend of two different turns is never a turn) | det((1 − s) R₀ + s R₁) = 1 − 2s(1 − s)(1 − cos a) for turns that differ by the angle a; less than 1 whenever 0 < s < 1 and a is not a multiple of 360°. At s = ½ it is cos²(a/2) (`G-X4.blend`, 66 different turns). The nearest turn to the blend at s = ¼ has turned 18.43°, not 22.5°. |
| 14_4 | "every pose on the way is a true pose" | X(s) = exp(s log(X_end X_start⁻¹)) X_start lies in SE(3) for every s (Lynch and Park eq. 9.6, there in the body form). |
| 15_1 | "so the body stays rigid" | A product of exponentials of twists is in SE(3). Adding dt [ω] R to R instead gives det = (1 + dt²) per step at one radian a second: 1.1052 after 1,000 steps of 0.01 s, 5.1% wider across the axis. |
| 15_3, 15_4 | "that leftover is itself a turn, almost exactly about the one axis neither turn used"; "shrinks a hundredfold" | The leftover is the group commutator of the two turns. For turns by a and b about perpendicular axes it is a turn by ab + O(3) radians about an axis that tends to the third one (0.7° off at 1°, 7° at 10°). The Lie bracket is the corresponding rule on so(3): `[x, y] = z`, the cross product. |

## The always and never sentences, and their edges

| Line | Sentence | Edge test | Result |
|---|---|---|---|
| 02_4 | "That always works. Any turn followed by any turn is one turn about one axis." | products of turns about 13 axes by 0° to 150°, including products that are half turns and products that are no turn: orthogonal, determinant one, an axis left in place | `G-X1.one_turn` 192/192 |
| 02_5 | "order usually matters" (lower third: about one axis, order never matters) | every ordered pair of a cube's 24 turns; two turns about one axis; two half turns about perpendicular axes | `G-X2.order_free_pairs` 120/576; `G-X2.same_axis_swap` 1; `G-X2.half_turns_swap` 1 |
| 03_6 | "somewhere there is a bad spot" | a theorem (see above); and for rings, all twelve orders: the middle angle at which the three axes span no volume | `G-X6.ring_orders` 12/12; `G-X6.bad_angle.zyx` 90; `G-X6.bad_angle.zyz` 0,180 |
| 06_4 | "every steady motion is a turn about a point, or a slide" (on screen) | 45 velocities (forward and sideways −0.5, 0, 0.5 m/s; spin −60° to 60° a second), five times each: on a circle about the centre, or on a line when the spin is zero | `G-X5.floor` 45/45 |
| 07_5 | "One screw would have carried it there, every time." | 264 different poses: no turn, and turns of 30° to 150° about 13 axes, each with four slides including none; a pure slide reads as a screw with no turn | `G-X3.one_screw` 264/264; `G-X3.pure_slide.turn` 0.0000 |
| 11_3 | "every joint's push on the wrist is zero" (along the arm) | each joint's push on the wrist along the line from shoulder to wrist, at the pose with the elbow straight; and, so that the test can fail, at a bent pose | `G-J.elbow_straight.push_along_arm` 0.0000; `G-J.swing.push_along_arm` 0.2897 |
| 12_4 | "its gripper holds one pose" | the twist produced by the joint speeds that swing the elbow; along the whole swing the pose holds to 10⁻⁹ and the radius is constant (test `the_spare_joint_s_speeds_move_nothing`) | `G-J.swing.null_twist` 0.0000 |
| 14_3 | "a blend of two different turns is never a turn" (on screen) | 66 different turns blended half and half with doing nothing: the determinant is cos²(a/2), and below one exactly when the angle is not zero | `G-X4.blend` 66/66 |

## Sources

Nothing below is copied into this repository beyond the short quotations
shown; page numbers for Lynch and Park are those of the authors' preprint
dated May 3, 2017 (the December 2019 preprint has the same equation
numbers and pages shifted by two).

**The robots.**

- KUKA LBR iiwa 14 R820, joint origins: the owner's
  `cmaes_explainer/vendor/iiwa14.xacro`, lines 48, 81, 114, 147, 180, 213,
  246 and 279 (`<origin xyz rpy>` of joints 1 to 8), each revolute joint
  with `<axis xyz="0 0 1"/>`. Upstream: aica-technology/kuka-iiwa-description,
  `robots/iiwa14.xacro`, commit `4745d7ae` (tag v0.1.0), GPL-3.0.
- KUKA, data sheet "LBR iiwa 14 R820", 0000-246-833 / V1.2 / 01.06.2022 / en,
  p. 1: "Maximum reach 820 mm"; "Rated payload 14 kg"; "Number of axes 7";
  motion range A1 to A7 ±170°, ±120°, ±170°, ±120°, ±170°, ±120°, ±175°;
  speed with rated payload 85, 85, 100, 75, 130, 135, 135 °/s. Read from an
  Internet Archive capture of kuka.com (the live site now serves it behind
  a login). The 2015 specification ("Spez LBR iiwa V5") agrees and its
  working-envelope drawing shows 360, 420, 400 and 126 mm, which are the
  xacro's offsets added in pairs.
- KUKA, "KMR iiwa. Always spot-on safely.", revision dated 01.05.2024, p. 2:
  "Velocity in longitudinal direction max. 3.6 km/h"; "Velocity in lateral
  direction max. 2.0 km/h"; Mecanum wheels.
- The platform's wheel speeds: Lynch and Park eq. (13.10), p. 519 (wheels
  numbered round the chassis there; the model lists them front left, front
  right, rear left, rear right); and p. 519:
  "to move in the direction +ŷb, wheels 1 and 3 drive backward and wheels 2
  and 4 drive forward at the same speed". The owner's
  `app/lib/mecanumKinematics.ts` implements the same four lines.
- Unitree G1: the owner's `cmaes_explainer/vendor/g1_29dof.urdf`, 29
  revolute joints (12 in the legs, 3 at the waist, 14 in the arms), root
  link `pelvis`. Upstream: unitreerobotics/unitree_ros,
  `robots/g1_description/g1_29dof.urdf`, BSD-3-Clause, © 2016–2022 HangZhou
  YuShu Technology (Unitree Robotics).
- FrankenSim (the owner's), `crates/fs-time/src/se3.rs`, module header:
  "Exp-map lane: canonical `fs-ga` `Se3` states updated through explicit
  body/right or space/left group operations"; function `se3_exp_step`.

**Lynch and Park, *Modern Robotics: Mechanics, Planning, and Control***
(Cambridge University Press, 2017; the authors' preprint).

- Def. 3.1, p. 70: "The special orthogonal group SO(3), also known as the
  group of rotation matrices…"; footnote 2: "(where “Lie” is pronounced
  “Lee”)".
- Prop. 3.5, p. 71: multiplication of rotation matrices is "generally not
  commutative".
- Prop. 3.11, eq. (3.51), p. 84: Rodrigues' formula; "Equation (3.51) is
  also known as Rodrigues’ formula for rotations."
- Def. 3.13, eq. (3.62), p. 89: SE(3). Twists with the angular part first:
  eq. (3.70), p. 98. Screw axis: Def. 3.24, p. 104. The screw exponential:
  Prop. 3.25, eq. (3.88), p. 105; the logarithm: eqs. (3.90)–(3.92), p. 106.
- §3.3.3.1, p. 105: "the Chasles–Mozzi theorem states that every rigid-body
  displacement can be expressed as a displacement along a fixed screw axis
  S in space." §3.7, p. 115: "Mozzi and Chasles, who independently
  discovered…".
- §4.1, p. 140: "The key concept behind the PoE formula is to regard each
  joint as applying a screw motion to all the outward links." Eq. (4.14),
  p. 142: the space form. §4.5, p. 160: "the Product of Exponentials (PoE)
  formula first presented by Brockett". Example 4.7, pp. 150–152: a
  seven-joint arm (the Barrett WAM) whose axes at zero alternate as the
  iiwa's do; "The zero configuration is singular".
- Def. 5.1, eqs. (5.9)–(5.11), pp. 179–180: the space Jacobian. §5.3,
  p. 191: singularities. §5.1.6, pp. 189–190: redundancy and "internal
  motions".
- §6.2.2, pp. 227–232: numerical inverse kinematics; Example 6.1: the
  tolerances 0.001 rad and 10⁻⁴ m.
- §9.2.1, pp. 326–328: the screw path, eq. (9.6); the decoupled path, eqs.
  (9.7) and (9.8).
- §2.3.2 and footnotes 4 and 5, pp. 27–28: singularities of a
  representation; unit quaternions as "a double covering of the set of
  orientations". Appendix B.1: at 90° of the middle angle "there exists a
  one-parameter family of solutions".
- Exercises 2.2 and 2.3, p. 38: the palm on the table; the arm as a serial
  chain is a model. §5.1.6, p. 190: "if you consider your arm from your
  shoulder to your palm as a seven-joint open chain, when you place your
  palm at a fixed configuration in space (e.g., on the surface of a table),
  you still have one internal degree of freedom corresponding to the
  position of your elbow."
- Brockett's foreword, pp. ix–x: "In 1983 I was invited to speak at the
  triennial Mathematical Theory of Networks and Systems Conference in Beer
  Sheva, Israel"; "the practical utility of the 150-year-old ideas of the
  geometers Klein and Lie."

**Other mathematics.**

- Solà, Deray and Atchuthan, "A micro Lie theory for state estimation in
  robotics", arXiv:1812.01537v9 (2021), §II-A, p. 2: "a Lie group G is a
  smooth manifold whose elements satisfy the group axioms."; §II-C: "The
  tangent space at the identity … is called the Lie algebra"; Example 4,
  p. 5: "This expression is the well known Rodrigues rotation formula."
  Their twist puts the translation first; this video follows Lynch and
  Park (spin first).
- Murray, Li and Sastry, *A Mathematical Introduction to Robotic
  Manipulation* (CRC Press, 1994; the authors' PDF), p. 49: "Theorem 2.11
  (Chasles). Every rigid body motion can be realized by a rotation about an
  axis combined with a translation parallel to that axis." Product of
  exponentials: eq. (3.3), p. 87. Their twist puts the linear part first.
- R. W. Brockett, "Robotic manipulators and the product of exponentials
  formula", in P. A. Fuhrmann (ed.), *Mathematical Theory of Networks and
  Systems* (MTNS-83, Beer Sheva, June 20–24, 1983), Lecture Notes in
  Control and Information Sciences 58, Springer, 1984, pp. 120–129, DOI
  10.1007/BFb0031048. Record verified; the text is paywalled and was not
  read.

**History.**

- Euler, "Formulae generales pro translatione quacunque corporum
  rigidorum", E478, presented to the St Petersburg Academy on 9 October
  1775, printed in *Novi Commentarii* 20 (1776), pp. 189–207. The theorem
  as read in Sten's transcription: "Quomodocunque sphaera circa centrum
  suum conuertatur, semper assignari potest diameter, cuius directio in
  situ translato conueniat cum situ initiali".
- Rodrigues, "Des lois géométriques qui régissent les déplacements d'un
  système solide dans l'espace…", *Journal de Mathématiques Pures et
  Appliquées* 5 (1840), 380–440. P. 391: "l'ordre de ces rotations ne peut
  être modifié sans altérer la position de l'axe résultant". P. 386,
  footnote: "Ce beau théorème de géométrie appartient, je crois, à M.
  Chasles, qui l'a publié dans le Bulletin universel des Sciences, tome
  XIV" (1830). Both from the scan's text layer; to be checked against the
  page image before either is shown on screen.
- Chasles, "Note sur les propriétés générales du système de deux corps
  semblables entr'eux", *Bulletin des Sciences Mathématiques, Astronomiques,
  Physiques et Chimiques* 14 (1830), 321–326 (record from Ceccarelli's
  reference list and from Rodrigues's footnote; the note itself was not
  opened).
- Mozzi, *Discorso matematico sopra il rotamento momentaneo dei corpi*
  (Naples, 1763), through M. Ceccarelli, "Screw axis defined by Giulio
  Mozzi in 1763 and early studies on helicoidal motion", *Mechanism and
  Machine Theory* 35 (2000), 761–770 (record and title verified; text not
  opened), and E. Minguzzi, arXiv:1201.4497, p. 2: "the celebrated
  Chasles’s theorem which was actually first obtained by Giulio Mozzi".
- Sophus Lie, 1842–1899, Norwegian (MacTutor): "It was during the winter of
  1873 - 74 that Lie began to develop systematically what became his theory
  of continuous transformation groups, later called Lie groups".
- Apollo. Three gimbals: D. Hoag, MIT Instrumentation Laboratory E-1344
  (1963): "in Apollo is a three degree of freedom gimballed structure".
  Thresholds: the command module's flight software, Comanche 055, routine
  GLOCKMON (listing p. 147): "ABS(MGA) GREATER THAN 70 DEGREES AND LESS THAN
  OR EQUAL TO 85 DEGREES -- GIMBAL LOCK LAMP TURNED ON." and "ABS(MGA)
  GREATER THAN 85 DEGREES -- ISS PUT IN COARSE ALIGN AND NO ATT LAMP TURNED
  ON." And for the lunar module, Paul Fjeld on the ALSJ gimbals page: "the
  computer flashed a 'gimbal lock' warning at 70 degrees and froze the IMU
  at 85 degrees, flashing an unfriendly 'no att' light. Then it waited for
  a realignment." The quote: NASA, *Apollo 11 Technical Air-to-Ground Voice
  Transcription*, tape 67/11, p. 337: "04 09 00 30 CMP (COLUMBIA) How about
  sending me a fourth gimbal for Christmas." (105 hours and 30 seconds of
  ground elapsed time; Michael Collins; the capsule communicator answered
  "You were unreadable. Say again please." and Collins said "Disregard.")
- Where the mathematics is used (for the lower third of 15_1 and the
  description): `manif` ("a Lie theory library for state-estimation
  targeted at robotics applications"); Pinocchio (Carpentier et al., 2019:
  "Planar and free-floating joints are meant to be employed as the basis of
  kinematic tree of mobile robots (humanoids, …)"); Lee, Leok and
  McClamroch, "Geometric tracking control of a quadrotor UAV on SE(3)",
  CDC 2010, pp. 5420–5425 (record only), and their companion paper
  arXiv:1003.2005, abstract: "A mathematical model of the quadrotor UAV
  rigid body dynamics, defined on the configuration space SE(3), is
  introduced as a basis for the analysis"; GTSAM ("smoothing and mapping
  (SAM) in robotics and vision"), whose `Pose3.h` reads "3D Pose manifold
  SO(3) x R^3 and group SE(3)" (for 15_1's "Mapping robots estimate
  them").

**Not verified** (and so not said): the hardback's page numbers; the text
of Brockett 1984, of Ceccarelli 2000, of Chasles's 1830 note and of
Mozzi's book; where a formula equal to eq. (3.51) sits in Rodrigues 1840;
how KUKA itself pronounces "iiwa" (the word stays on screen); the
pronunciation of "Rodrigues" for this person beyond the French surname
(to be settled in P3 with the owner's ear as the last word); the rate at
which KUKA's controller runs; the current KUKA specification behind the
login.

## Every fact

Copied from `oracle.py`'s output by a script (key, value). `lie facts`
prints the same lines.


G-A: the arm (KUKA LBR iiwa 14 R820). Lengths in metres from the xacro's joint origins; axes and heights with every joint at zero; limits and speeds from KUKA's data sheet.

| Key | Value |
|---|---|
| `G-A.joints` | 7 |
| `G-A.offsets` | 0.1575,0.2025,0.2045,0.2155,0.1845,0.2155,0.0810,0.0450 |
| `G-A.shoulder_height` | 0.3600 |
| `G-A.upper_arm` | 0.4200 |
| `G-A.forearm` | 0.4000 |
| `G-A.wrist_to_tip` | 0.1260 |
| `G-A.height` | 1.3060 |
| `G-A.reach` | 0.8200 |
| `G-A.home.p` | 0.0000,0.0000,1.3060 |
| `G-A.home.R` | 1,0,0;0,1,0;0,0,1 |
| `G-A.axis.1` | 0,0,1 |
| `G-A.axis_height.1` | 0.1575 |
| `G-A.axis.2` | 0,1,0 |
| `G-A.axis_height.2` | 0.3600 |
| `G-A.axis.3` | 0,0,1 |
| `G-A.axis_height.3` | 0.5645 |
| `G-A.axis.4` | 0,-1,0 |
| `G-A.axis_height.4` | 0.7800 |
| `G-A.axis.5` | 0,0,1 |
| `G-A.axis_height.5` | 0.9645 |
| `G-A.axis.6` | 0,1,0 |
| `G-A.axis_height.6` | 1.1800 |
| `G-A.axis.7` | 0,0,1 |
| `G-A.axis_height.7` | 1.2610 |
| `G-A.limits_deg` | 170,120,170,120,170,120,175 |
| `G-A.speed_deg_s` | 85,85,100,75,130,135,135 |

G-B: the book. Tilt is a quarter turn about x, roll a quarter turn about y, about the room's axes (two lines on the table). `both_at_once` is the turn made by holding the two spins together for one second. `leftover.N` is the turn between the two orders' results for turns of N degrees; `product` is the product of the two angles, in degrees.

| Key | Value |
|---|---|
| `G-B.tilt_then_roll.R` | 0,1,0;0,0,-1;-1,0,0 |
| `G-B.roll_then_tilt.R` | 0,0,1;1,0,0;0,1,0 |
| `G-B.tilt_then_roll.cover` | 0,-1,0 |
| `G-B.roll_then_tilt.cover` | 1,0,0 |
| `G-B.tilt_then_roll.angle` | 120.0000 |
| `G-B.tilt_then_roll.axis` | 0.5774,0.5774,-0.5774 |
| `G-B.roll_then_tilt.angle` | 120.0000 |
| `G-B.roll_then_tilt.axis` | 0.5774,0.5774,0.5774 |
| `G-B.between.angle` | 120.0000 |
| `G-B.between.axis` | -0.5774,0.5774,-0.5774 |
| `G-B.slides.same` | 1 |
| `G-B.turns.same` | 0 |
| `G-B.both_at_once.angle` | 127.2792 |
| `G-B.both_at_once.axis` | 0.7071,0.7071,0.0000 |
| `G-B.leftover.1.angle` | 0.0175 |
| `G-B.leftover.1.axis` | -0.0087,0.0087,-0.9999 |
| `G-B.leftover.1.product` | 0.0175 |
| `G-B.leftover.10.angle` | 1.7409 |
| `G-B.leftover.10.axis` | -0.0868,0.0868,-0.9924 |
| `G-B.leftover.10.product` | 1.7453 |
| `G-B.leftover.90.angle` | 120.0000 |
| `G-B.leftover.90.axis` | -0.5774,0.5774,-0.5774 |
| `G-B.leftover.90.product` | 141.3717 |

G-E: three angles. `lock.same` is the largest difference between the matrices of ring angles (30°, 90°, 0°) and (0°, 90°, −30°); `left.P` the volume the three ring axes span at a middle angle of P degrees.

| Key | Value |
|---|---|
| `G-E.rings.order` | z,y,x |
| `G-E.lock.same` | 0.0000 |
| `G-E.free.differ` | 0.5000 |
| `G-E.left.0` | 1.0000 |
| `G-E.left.70` | 0.3420 |
| `G-E.left.85` | 0.0872 |
| `G-E.left.90` | 0.0000 |
| `G-E.near.apart` | 1.4142 |
| `G-E.near.triples` | 0,89,0;90,89,90 |
| `G-E.near.differ` | 90,0,90 |

G-H: halfway between two headings (degrees).

| Key | Value |
|---|---|
| `G-H.a` | 170 |
| `G-H.b` | -170 |
| `G-H.mean` | 0.0000 |
| `G-H.gap` | 20.0000 |
| `G-H.halfway` | 180.0000 |
| `G-H.off_by` | 180.0000 |

G-R: spins. The spin of chapter 05, and the two parts of the point (1, 0, 0) that Rodrigues' formula splits.

| Key | Value |
|---|---|
| `G-R.diag.rate_deg` | 120 |
| `G-R.diag.R` | 0,0,1;1,0,0;0,1,0 |
| `G-R.diag.arrow` | 1.2092,1.2092,1.2092 |
| `G-R.diag.is_roll_then_tilt` | 1 |
| `G-R.diag.edges` | 0,1,0;0,0,1;1,0,0 |
| `G-R.rod.along` | 0.3333,0.3333,0.3333 |
| `G-R.rod.around` | 0.6667,-0.3333,-0.3333 |
| `G-R.rod.result` | 0.0000,1.0000,0.0000 |
| `G-R.quarter_z.R` | 0,-1,0;1,0,0;0,0,1 |
| `G-R.numbers.arrow` | 3 |
| `G-R.numbers.matrix` | 9 |

G-P: the platform on the floor (the video's numbers). Metres, seconds, degrees; wheel senses in the order front left, front right, rear left, rear right.

| Key | Value |
|---|---|
| `G-P.v` | 0.5 |
| `G-P.spin_deg` | 30 |
| `G-P.t` | 6 |
| `G-P.end` | 0.0000,1.9099 |
| `G-P.heading` | 180.0000 |
| `G-P.naive` | 3.0000,0.0000 |
| `G-P.radius` | 0.9549 |
| `G-P.centre` | 0.0000,0.9549 |
| `G-P.arc_length` | 3.0000 |
| `G-P.full_circle_t` | 12 |
| `G-P.full.end` | 0.0000,0.0000 |
| `G-P.spin0.end` | 3.0000,0.0000 |
| `G-P.spin60.end` | 0.0000,0.0000 |
| `G-P.wheels.forward` | +,+,+,+ |
| `G-P.wheels.sideways` | -,+,+,- |
| `G-P.wheels.spin` | -,+,-,+ |

G-S: screws. The book tossed a quarter turn and (1, 1, 0.5) m, as one screw; how many numbers each description holds.

| Key | Value |
|---|---|
| `G-S.toss.turn_deg` | 90.0000 |
| `G-S.toss.axis` | 0.0000,0.0000,1.0000 |
| `G-S.toss.point` | 0.0000,1.0000,0.0000 |
| `G-S.toss.slide` | 0.5000 |
| `G-S.toss.rebuilt` | 0.0000 |
| `G-S.numbers.pose` | 6 |
| `G-S.numbers.twist` | 6 |
| `G-S.numbers.turn_matrix` | 9 |
| `G-S.numbers.pose_matrix` | 16 |
| `G-S.numbers.quaternion` | 4 |

G-K: the arm's poses (metres; joint angles in degrees).

| Key | Value |
|---|---|
| `G-K.ell.q` | 0,90,0,90,0,0,0 |
| `G-K.ell.p` | 0.4200,0.0000,0.8860 |
| `G-K.ell.R` | 1,0,0;0,1,0;0,0,1 |
| `G-K.ell.after_elbow` | -0.5260,0.0000,0.7800 |
| `G-K.quarter.p` | 0.0000,0.4200,0.8860 |
| `G-K.elbow.moves` | 5,6,7,tip |

G-J: velocities. Levers in metres; ranks of the Jacobian (and of the L shape's with joint 3 or joint 7 taken away); each joint's largest push on the wrist along the arm (m/s per rad/s); the elbow's speed in degrees per second for a pull of 0.1 m/s at each bend, and the bend at which it meets its limit.

| Key | Value |
|---|---|
| `G-J.shape` | 6x7 |
| `G-J.ell.lever.1` | 0.4200 |
| `G-J.ell.lever.2` | 0.6731 |
| `G-J.ell.lever.3` | 0.5260 |
| `G-J.ell.lever.4` | 0.5260 |
| `G-J.ell.lever.5` | 0.0000 |
| `G-J.ell.lever.6` | 0.1260 |
| `G-J.ell.lever.7` | 0.0000 |
| `G-J.ell.col2` | 0.0000,1.0000,0.0000,-0.3600,0.0000,0.0000 |
| `G-J.rank.home` | 3 |
| `G-J.rank.stretch` | 4 |
| `G-J.rank.elbow_straight` | 5 |
| `G-J.rank.elbow_only` | 5 |
| `G-J.rank.ell` | 6 |
| `G-J.rank.ell_wrist_turned` | 5 |
| `G-J.rank.swing` | 6 |
| `G-J.rank.ell.without_3` | 5 |
| `G-J.rank.ell.without_7` | 6 |
| `G-J.elbow_straight.p` | 0.8193,0.0000,0.7070 |
| `G-J.elbow_only.reach` | 0.5800 |
| `G-J.elbow_straight.push_along_arm` | 0.0000 |
| `G-J.swing.push_along_arm` | 0.2897 |
| `G-J.blowup.pull` | 0.1 |
| `G-J.blowup.30` | 54.03 |
| `G-J.blowup.10` | 160.44 |
| `G-J.blowup.3` | 534.17 |
| `G-J.blowup.1` | 1602.34 |
| `G-J.blowup.limit` | 75 |
| `G-J.blowup.bend_at_limit` | 21.5 |
| `G-J.blowup.1.turns_per_s` | 4.45 |
| `G-J.elbow.radius` | 0.2897 |
| `G-J.spare` | 1 |
| `G-J.swing.null_twist` | 0.0000 |
| `G-J.swing.p` | 0.5291,0.0000,0.8230 |

G-I: steps and paths. Newton's run from the start pose to the goal; the screw between them; the blend's determinant at halfway and how far its nearest turn has turned a quarter of the way; the largest gap between the screw path and the straight line.

| Key | Value |
|---|---|
| `G-I.start.q` | 0,30,0,-60,0,90,0 |
| `G-I.start.p` | 0.6100,0.0000,0.5977 |
| `G-I.start.R` | -1,0,0;0,1,0;0,0,-1 |
| `G-I.goal.p` | 0.4000,0.3000,0.2500 |
| `G-I.goal.R` | 0,-1,0;-1,0,0;0,0,-1 |
| `G-I.ik.steps` | 3 |
| `G-I.ik.dist_mm` | 504.99,93.93,4.15,0.03 |
| `G-I.ik.angle_deg` | 90.0000,8.6404,0.7239,0.0076 |
| `G-I.ik.q_deg` | 24.89,38.57,15.58,-104.86,-15.56,38.65,-40.55 |
| `G-I.ik.within_limits` | 1 |
| `G-I.ik.tolerances` | 0.001,0.0001 |
| `G-I.screw.angle` | 90.0000 |
| `G-I.screw.axis` | 0.0000,0.0000,1.0000 |
| `G-I.screw.point` | 0.3550,0.0450 |
| `G-I.screw.slide` | -0.3477 |
| `G-I.blend.det` | 0.5000 |
| `G-I.blend.quarter_turned` | 18.4349 |
| `G-I.bow` | 0.0758 |
| `G-I.screw.mid.det` | 1.0000 |
| `G-I.screw.mid.turned` | 45.0000 |
| `G-I.screw.quarter_turned` | 22.5000 |

G-C: the coda. The humanoid's counts; the determinant after 1,000 steps taken two ways, and the first body's width across its axis.

| Key | Value |
|---|---|
| `G-C.g1.joints` | 29 |
| `G-C.g1.velocity` | 35 |
| `G-C.g1.stored` | 36 |
| `G-C.drift.steps` | 1000 |
| `G-C.drift.dt` | 0.01 |
| `G-C.drift.add` | 1.1052 |
| `G-C.drift.exp` | 1.0000 |
| `G-C.drift.width` | 1.0513 |

G-X: the edges of the always sentences (held / checked).

| Key | Value |
|---|---|
| `G-X1.one_turn` | 192/192 |
| `G-X2.cube_turns` | 24 |
| `G-X2.order_free_pairs` | 120/576 |
| `G-X2.same_axis_swap` | 1 |
| `G-X2.half_turns_swap` | 1 |
| `G-X3.one_screw` | 264/264 |
| `G-X3.pure_slide.turn` | 0.0000 |
| `G-X4.blend` | 66/66 |
| `G-X5.floor` | 45/45 |
| `G-X6.ring_orders` | 12/12 |
| `G-X6.bad_angle.zyx` | 90 |
| `G-X6.bad_angle.zyz` | 0,180 |

G-Q: the quizzes: the countdown in seconds and the computed answer.

| Key | Value |
|---|---|
| `G-Q.01_1.seconds` | 5 |
| `G-Q.01_1.answer` | different |
| `G-Q.02_2.seconds` | 5 |
| `G-Q.02_2.answer` | 120 |
| `G-Q.03_2.seconds` | 4 |
| `G-Q.03_2.answer` | same |
| `G-Q.04_2.seconds` | 5 |
| `G-Q.04_2.answer` | 180 |
| `G-Q.05_4.seconds` | 5 |
| `G-Q.05_4.answer` | next |
| `G-Q.06_2.seconds` | 6 |
| `G-Q.06_2.answer` | beside |
| `G-Q.07_1.seconds` | 4 |
| `G-Q.07_1.answer` | 6 |
| `G-Q.08_3.seconds` | 4 |
| `G-Q.08_3.answer` | past |
| `G-Q.09_4.seconds` | 4 |
| `G-Q.09_4.answer` | side |
| `G-Q.10_4.seconds` | 5 |
| `G-Q.10_4.answer` | 2 |
| `G-Q.11_2.seconds` | 4 |
| `G-Q.11_2.answer` | lost |
| `G-Q.12_2.seconds` | 6 |
| `G-Q.12_2.answer` | moves |
| `G-Q.13_5.seconds` | 5 |
| `G-Q.13_5.answer` | 3 |
| `G-Q.14_2.seconds` | 4 |
| `G-Q.14_2.answer` | not |
| `G-Q.15_2.seconds` | 4 |
| `G-Q.15_2.answer` | differ |

G-L: cited literals.

| Key | Value |
|---|---|
| `G-L.kuka.reach_mm` | 820 |
| `G-L.kuka.payload_kg` | 14 |
| `G-L.kuka.axes` | 7 |
| `G-L.kmr.speed_kmh` | 3.6 |
| `G-L.kmr.speed_m_s` | 1.0 |
| `G-L.apollo.lamp_deg` | 70 |
| `G-L.apollo.lost_deg` | 85 |
| `G-L.apollo.gimbals` | 3 |
| `G-L.euler.year` | 1775 |
| `G-L.mozzi.year` | 1763 |
| `G-L.chasles.year` | 1830 |
| `G-L.rodrigues.year` | 1840 |
| `G-L.lie.year` | 1873 |
| `G-L.brockett.year` | 1983 |
