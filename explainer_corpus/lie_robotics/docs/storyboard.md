# Storyboard — Why Robot Arms Think in Lie Groups

The hand-edited source of this video's words and pictures. `python3 -I
tools/storyboard.py lie_robotics` checks it (gate G2) and writes `script.tsv`
from the words column. Facts: `docs/fact-sheet.md` and `oracle.py` (every
number below names its fact key); brief: `docs/brief.md`; reviews and their
dispositions: `docs/reviews/lie_robotics/`.

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a question for the
viewer; the beat ends with the question and the picture holds before the
next beat answers it.

**The quiz device.** The owner asked for "quizzes with pauses before the
answers", so every chapter has one. A quiz is a `⟨predict⟩` beat whose last
column says `listen N s`: a violet card with the question and its choices, a
ring that drains over N seconds with the seconds left in its middle, the line
"pause for more time" under it, the narration silent and the music alone. The
next beat gives the answer in the same place on screen, and the picture shows
why. Every quiz is a row of `model::QUIZZES`; its answer is computed
(`G-Q.<line>.answer`), never typed.

**Three-dimensional pictures.** The book, the gimbal, the arm and the
humanoid are drawn by the video's own projection (`chapters/draw.rs`: one
fixed three-quarter view, faces and links sorted back to front, farther parts
dimmer) on franken_manim's retained route, so matrices, readouts and quiz
cards share the frame with them and the layout lint reads every chapter.
Axes on screen: x to the right, y away from the viewer, z up.

## 2 The arc

One picture carries the video: **the screw** (a turn about an axis with a
slide along it). A spin held for a second is a screw with no slide; a base
on a floor turns about a point; every motion in space is one screw; every
joint is one; the gripper's velocity is a sum of them; the error to a goal
is one; the straight path is one. Under it sits one move, made in chapter
04 and repeated to the end: the space is curved, so do the arithmetic on the
flat piece that touches it, then roll the answer back on.

| Act | Chapters | Spine |
|---|---|---|
| I. Orientations refuse to add | 01 the book, 02 one turn, 03 three angles, 04 halfway | two quarter turns in two orders; turns form a group; three angles have a bad spot (Apollo); averaging angles points the wrong way; the flat piece |
| II. The flat piece and the way back | 05 spin, 06 the base, 07 screws | a spin held one second (the exponential, Rodrigues); a base's arc on a floor; in space every motion is a screw (Chasles, Mozzi); six numbers |
| III. The arm | 08 seven screws, 09 by hand | each joint is a screw; the product of exponentials (Brockett); two joints by hand with the arm's real lengths, then a third |
| IV. Moving | 10 velocities add, 11 lost directions, 12 the spare joint | the Jacobian; where the arm loses a direction and what that costs; the elbow's free circle |
| V. Getting there | 13 steps, 14 the straight path | Newton's method on the group, three steps; the screw between two poses against a blend of matrices |
| VI. Everywhere | 15 everywhere | a humanoid's floating body, drones, maps, a physics engine's step; the book again: how much order matters is itself a turn |

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses
a data colour.

| Idea | Color |
|---|---|
| orientation and turns (the book's cover, a ring, a turn's arc) | blue `#58C4DD` |
| position and slides | teal `#5CD0B3` |
| the flat piece: spin arrows, twists, the Jacobian's columns | orange `#F28E52` |
| a screw's axis, and the joint in question | `hero` gold `#FFD866` |
| the goal pose | pink `#EA87B3` |
| the arm's links and joints at rest | steel `#C9D1DD` on `dim` |
| a lost direction, a bad spot, a wrong path | `warn` red `#FC6255` |
| a confirmed answer | `good` green `#83C167` |
| the quiz card and its countdown | violet `#B189D6` |

The seven hues are checked pairwise for contrast (ΔE2000 ≥ 20) by a test
before drafts.

## Vocabulary (one word per idea)

**turn** (a rotation, as an action or its result; never "rotation" except in
the on-screen name "rotation matrix"); **quarter turn**, **third of a turn**;
**tip** and **roll** (the book's two turns); **slide** (a translation);
**orientation** (which way a thing faces), **position** (where it is),
**pose** (both together); **doing nothing** (the identity) and **undo** (the
inverse); **group**; **matrix** (a grid of numbers); **ring** and **gimbal**;
**heading**; **the flat piece** (the tangent space; the formal name appears
once, on screen); **spin** (a steady rate of turning, an arrow) and
**arrow** (a vector); **the exponential**; **twist** (six numbers: a spin
arrow and a slide arrow); **screw** (a turn about an axis with a slide along
it); **joint**, **joint angle**, **gripper**, **tip** of the gripper as "the
gripper's tip" only where "tip" the turn cannot be meant, **shoulder**,
**elbow**, **wrist**, **base** (the mobile platform, and the arm's foot);
**Jacobian**; **direction** (a way the gripper can move); **singularity**;
**spare joint**; **step**; **goal**; **blend** (number-by-number averaging).
Never "rotation vector", "tangent space", "manifold", "end effector",
"null space" or "geodesic" aloud: each is a lower third.

## 01_the_book (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | ⟨predict⟩ Put a book flat on a table. Tip its far edge up, a quarter turn. Then roll it a quarter turn to the right. Now start over, and swap the order. Roll first, then tip. Does the book end up the same way? | Two copies of a book on a table, side by side, drawn in three dimensions (blue cover up, spine on the left). Left copy: tip, then roll, each turn's axis drawn through the book as it happens. Right copy stays still, greyed, under a violet quiz card: "same or different?" with the ring draining from 5 (`G-Q.01_1.*`) | **the book**; **the quiz card**; 3D turns by placement; countdown ring on a tracker | listen 5 s (the countdown: the music alone) |
| 01_2 | No. Tip first, and the cover ends up facing you. Roll first, and it faces the wall on your right. | The right copy now does roll, then tip. Both books hold their final orientations; an arrow out of each cover: toward the viewer, and to the right (`G-B.tip_then_roll.cover`, `G-B.roll_then_tip.cover`) | 3D turns; highlight | |
| 01_3 | Sliding is nothing like this. Slide the book right, then away, or away, then right, and it lands on the same spot. Positions add. Orientations don't. | Both books flat again; each slides right and away in opposite orders along teal arrows, landing on the same teal dot. Two lines build: "positions add" (teal, ticked), "orientations don't" (blue, crossed) (`G-B.slides.same`, `G-B.turns.same`) | morph; highlight | |
| 01_4 | *(title)* | **Why Robot Arms Think in Lie Groups** | title card | listen 3 s (the motif) |
| 01_5 | This is a KUKA robot arm with seven joints. Its job is to put its gripper at a pose, a position and an orientation together. | The arm (lower third: KUKA LBR iiwa 14 R820) reaches toward a mug on a table; the goal pose glows pink: a dot for position, three short axes for orientation. Labels: position (teal), orientation (blue), "pose = both" | **the arm** (projected 3D, links back to front); lagged reveal | |
| 01_6 | So its software must do arithmetic with things that refuse to add. The way out is about a hundred and fifty years old, and by the end you'll read it in one line. | The arm's gripper corkscrews down onto the mug along a gold screw axis (the path of chapter 14, in outline). Above it the line `T(q) = e^{[S_1]q_1} e^{[S_2]q_2} \cdots e^{[S_7]q_7} M` writes itself in dim ink (`G-L.lie.year`) | TeX formula; moving arm | |

- **Cold open** (plan rule 20): a puzzle the viewer can try with a real
  book, answered at once, then the title, then the payoff previewed as the
  line the video will earn.
- **Misconception defused:** "turning is like moving: do the parts in any
  order". Shown false on a real object before any symbol appears.
- **Ruth:** the two turns are made about axes fixed in the room, so the later
  turn multiplies on the left; the matrices are `G-B.*.R`. "Positions add"
  is commutativity of vector addition; "orientations don't" is said of
  these two turns, never as "never".
- **Dev:** "about a hundred and fifty years old": Lie began his theory of
  continuous groups in the winter of 1873–74 (MacTutor).

## 02_one_turn (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Look where tip then roll left the book. It could have got there in a single turn, about a single axis. | One book. Its start (ghost, flat) and its end (cover toward the viewer, on its side). A gold axis appears through the book's corner and the book swings from ghost to end in one turn about it | 3D turn; ghost | |
| 02_2 | ⟨predict⟩ Two quarter turns went in. How big is that single turn? | The quiz card with three choices: ¼ turn, ⅓ turn, ½ turn; the ring drains from 5 (`G-Q.02_2.*`) | quiz card; countdown | listen 5 s |
| 02_3 | A third of a turn. A hundred and twenty degrees, about the diagonal through the book's corner. | The single turn replays with a live angle readout climbing to 120°; the axis labelled as the corner diagonal (`G-B.tip_then_roll.angle`, `.axis`) | live number on a tracker | |
| 02_4 | That always works. Any turn followed by any turn is one turn about one axis. Euler showed it in seventeen seventy-five. | Three more pairs of turns flash by small, each collapsing to one gold axis and one arc. Lower third: "Euler, 1775: every motion of a body about a fixed point is a turn about some axis" (`G-X1.one_turn`, `G-L.euler.year`) | lagged reveal; highlight | |
| 02_5 | And any turn can be undone. A collection where you can combine any two, and undo any one, is called a group. | The book turns and turns back; a still book labelled "doing nothing". Three ticks: combine, undo, do nothing. The word **group**. Then a small pair of panels: tip then roll against roll then tip, a red "≠"; two turns about one axis in both orders, a green "=". Lower third: "order usually matters: of the 576 ordered pairs of a cube's 24 turns, 120 do not care" (`G-X2.*`) | card; ticks; lower third | |
| 02_6 | To compute, track where the book's three edges point. Three arrows, three numbers each. Nine numbers in a grid, a matrix. One turn after another is one matrix times another. | Three arrows along the book's edges; as the book turns their coordinates fill the columns of a 3 by 3 grid. Then the two grids for tip and roll multiply into the grid of `G-B.tip_then_roll.R`. "For programmers" tag: `R = roll · tip` | **matrix** built from primitives; lagged reveal; "for programmers" tag | |

- **Misconception defused:** "two quarter turns make a half turn". They make
  a third of a turn about an axis neither turn used.
- **Ruth:** "any turn followed by any turn is one turn": Euler's theorem
  (E478, presented 9 October 1775, printed 1776); edge-tested on 192
  products, including products that are half turns and products that are
  no turn. Associativity is the fourth group axiom; it is on the card of
  02_5 ("precisely: and combining is associative") and not spoken. The
  narration never says turns "never" swap: the lower third of 02_5 gives
  the exact count for a cube's turns, and `G-X2.half_turns_swap` records
  that two half turns about perpendicular axes also swap.
- **Dev:** the "for programmers" tag states the order of the product.

## 03_three_angles (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Nine numbers is a lot for one orientation. Three should do. Here is a machine that tries, a gimbal. Three rings, one inside the next, each turning about its own axis. | The book inside three nested blue rings (outer about the vertical, middle about the left to right line, inner about the book's own long axis), three live angle readouts. Each ring wiggles in turn and the book answers with three visibly different motions | **the gimbal** (three projected rings); live numbers; 3D turns | |
| 03_2 | ⟨predict⟩ Turn the middle ring a quarter turn. Now the outer ring's axis and the inner ring's axis line up. Do those two rings still do different things? | The middle ring turns to 90°; the outer and inner axes swing into one line (they flash red). Quiz card: "different, or the same?", ring from 4 (`G-Q.03_2.*`) | quiz card; countdown; highlight | listen 4 s |
| 03_3 | No. They turn the book about the very same axis. Three rings, but only two ways to turn. This is gimbal lock. | Outer ring wiggles, then inner: the book makes the same motion both times. A readout "ways to turn: 3" drops to 2; the missing axis is drawn dashed red (`G-E.lock.same`, `G-E.left.90`) | live number; highlight | |
| 03_4 | It happened on the way to the Moon. Apollo steered by three gimbals. Past seventy degrees on the middle one, a warning lamp lit. Past eighty-five, the computer gave up its reference. | A dial for the middle ring's angle sweeping up; a bar "third direction left" shrinking as it goes: 34% at 70° (lamp), 9% at 85° (red). Lower third: "Apollo guidance computer: gimbal lock lamp above 70°, reference dropped above 85°" (`G-L.apollo.*`, `G-E.left.70`, `G-E.left.85`) | live bar on a tracker; lower third | |
| 03_5 | On Apollo eleven, Michael Collins radioed a request. How about sending me a fourth gimbal for Christmas. | A quote card: "How about sending me a fourth gimbal for Christmas." Michael Collins, Apollo 11, 105 hours into the flight | quote card | hold |
| 03_6 | It wasn't bad engineering. Any way of naming every orientation with just three numbers has a bad spot somewhere. | Three other three-number schemes as small icons (other ring orders), each with its own red spot. Lower third: "precisely: no three numbers chart all orientations smoothly" | lagged reveal; lower third | |

- **Misconception defused:** "gimbal lock means the rings jam". Nothing
  jams; two rings start doing the same job, so one way of turning has no
  ring left.
- **Ruth:** the rings are yaw, pitch, roll with `R = Rz Ry Rx`; at a quarter
  turn of the middle ring only roll minus yaw matters (`G-E.lock.same` is
  the largest difference between the two matrices: zero). "Third direction
  left" is the volume spanned by the three ring axes, cos of the middle
  angle. The thresholds are the command module computer's own (Comanche
  055, GLOCKMON: lamp above 70°, coarse align and NO ATT above 85°). The
  quote is verbatim from NASA's air-to-ground transcript, 04 09 00 30. The
  last line's exact form (no three-number chart covers SO(3) without a
  singularity) is in the fact sheet with its sources.
- **Dev:** the same loss appears in an arm's wrist in chapter 11.

## 04_halfway (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Angles hide a second trap. Two compass headings. One is a hundred and seventy degrees. The other is minus a hundred and seventy. | A circle seen from above with a small robot at its centre; "ahead" at 0°. Two blue heading arrows, at 170° and −170°, both pointing nearly straight back, 20° apart (`G-H.a`, `G-H.b`) | circle and arrows | |
| 04_2 | ⟨predict⟩ What heading is halfway between them? | Quiz card: "halfway between 170° and −170°?" Choices 0°, 90°, 180°. Ring from 5 (`G-Q.04_2.*`) | quiz card; countdown | listen 5 s |
| 04_3 | Average the numbers and you get zero. Straight ahead. But both arrows point almost straight back. The honest halfway is a hundred and eighty, the exact opposite. | The sum (170 + −170) ÷ 2 = 0 writes out; a red arrow appears at 0°, pointing ahead. Then a green arrow at 180°, between the two blue ones. Under the circle a number line from −180 to 180 bends up and wraps onto it, its two ends meeting at the back in a red seam (`G-H.mean`, `G-H.halfway`, `G-H.off_by`) | TeX equation; highlight; morph that keeps identity | |
| 04_4 | But zoom in on one point of the circle. Up close, it looks straight. So measure from one heading to the other as a turn. Twenty degrees. Take half. Ten degrees on from the first is a hundred and eighty. | Zoom on the 170° point: an orange straight line touches the circle there, with tick marks. The step to the other heading is laid along it: 20°; half of it, 10°, is rolled back onto the circle, landing on the green arrow (`G-H.gap`) | scaled group as a zoom; number line; morph | |
| 04_5 | That is the whole trick, and the rest of this video is that trick again and again. The space is curved. Do the arithmetic on the flat piece that touches it. Then roll the answer back on. | The circle and its orange line, centred. Three words build: curved, flat, roll back. Lower third: "the flat piece: the tangent space" | card; lower third | hold |

- **Misconception defused:** "the average of two angles is the angle in
  between". Only when the two numbers are on the same side of the seam.
- **Ruth:** "halfway" means the midpoint of the shorter arc; it is unique
  here because the headings are 20° apart, not 180°. `G-H.off_by` is the
  angle between the averaged heading and the honest one.
- **Dev:** this is the bug behind a robot that spins the long way round.

## 05_spin (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | For orientations, the flat piece is made of spins. Start from doing nothing, and begin to turn. Say which axis, and how fast. That is one arrow, along the axis, as long as the speed. | The book, still. An orange arrow grows through it; a curled arrow shows the sense of the turning. Lengthening the arrow speeds the turning | 3D book; arrow on a tracker | |
| 05_2 | Arrows are easy. They add, they stretch, they average, with no bad spots. | Two orange arrows add tip to tail; one doubles; two average to one in between. Beside them, the arrow's three numbers | lagged reveal | |
| 05_3 | Hold one spin steady for one second, and the book ends at some orientation. That step, from a spin to an orientation, is called the exponential. | The arrow stays fixed while the book turns about it; a one-second clock ring fills; the book stops. A curved arrow from the orange arrow to the turned book, labelled "exponential". Lower third: `e^{[\omega]}` | 3D turn; clock on a tracker; TeX | |
| 05_4 | ⟨predict⟩ Spin the book about the diagonal through its corner, at a third of a turn per second. After one second, where do its three edges point? | The orange arrow along the corner diagonal; the book's three edge arrows drawn. Quiz card: "back where they began", "each where the next one was", "each reversed". Ring from 5 (`G-Q.05_4.*`) | quiz card; countdown | listen 5 s |
| 05_5 | Each edge lands where the next one was. That is where the book ended in the first quiz, with roll first. | The book spins for the second; each edge arrow lands on its neighbour's ghost. The book from 01_2 (roll, then tip) appears beside it in the same orientation; a green "=" (`G-R.diag.edges`, `G-R.diag.is_roll_then_tip`) | 3D turn; highlight | |
| 05_6 | There is a formula for this step. Split any point into the part along the axis, which stays, and the part going around, which swings on a circle. It is called Rodrigues' formula. | One corner of the book: its part along the axis (teal, fixed) and its part around (blue, sweeping a circle). The formula builds from those pieces: `e^{[\omega]t} = I + \sin t\,[\omega] + (1 - \cos t)\,[\omega]^2`. Lower third: "named for Olinde Rodrigues, who worked out in 1840 how turns combine" (`G-R.rod.*`, `G-L.rodrigues.year`) | TeX derivation from the picture; highlight | |
| 05_7 | A smooth space of motions like this, where you can combine and undo, is a Lie group, after the Norwegian mathematician Sophus Lie. Its flat piece is its Lie algebra. | The curved space as a surface with the orange flat piece touching it at "doing nothing"; the exponential arrow from one to the other. Two names: **Lie group** (blue), **Lie algebra** (orange). A name card: "Sophus Lie, 1842–1899" | card; names | |

- **Misconception defused:** "a spin's three numbers are three angles".
  They are one arrow; nothing is done in sequence, so there is no order and
  no bad spot at doing nothing.
- **Ruth:** "the exponential" is the matrix exponential of the spin's skew
  matrix, held for one second; the formula is Lynch and Park eq. 3.51 with
  a unit axis and t in radians. A half turn has two arrows (plus and minus);
  the fact sheet records it. "Smooth, where you can combine and undo" is
  the plain form of "a group that is a differentiable manifold with smooth
  operations". "No bad spots" is said of the arrows as a flat space; the
  exponential itself folds over at a half turn.
- **Dev:** three numbers in, nine out; the logarithm goes back.

## 06_the_base (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Poses work the same way. Start on a floor. This is a KUKA mobile base. Its wheels carry slanted rollers, so it can drive forward, slide sideways and spin, all at once. Its pose is three numbers. So is its velocity. | Top view of the base (lower third: KUKA KMR iiwa, Mecanum wheels): a rounded rectangle, four wheels hatched at 45°. Three arrows at its centre: forward, sideways (teal), spin (orange, curled). Then two readouts: pose (x, y, heading) and velocity (forward, sideways, spin). A "for programmers" panel slides in from the right: the four wheel-speed lines and the wheels' senses for forward (+ + + +), sideways (− + + −) and spin (− + − +); "Lynch and Park, eq. 13.10" (`G-P.wheels.*`) | **the base** (top view); lagged reveal; readouts; code panel with the "for programmers" tag | |
| 06_2 | ⟨predict⟩ Hold one velocity for six seconds. Forward at half a meter per second, spinning at thirty degrees per second. Where does the base end up? | The base at the origin of a floor grid in meters, its velocity shown. Quiz card with three small maps: "three meters ahead", "beside where it started", "back where it started". Ring from 5 (`G-Q.06_2.*`, `G-P.v`, `G-P.spin_deg`, `G-P.t`) | quiz card; countdown; grid | listen 5 s |
| 06_3 | Beside where it started. Just under two meters to its left, facing backward. It drove a half circle. | The base drives its arc, leaving a teal trail, a live pose readout running; it stops at (0.00, 1.91) facing back (`G-P.end`, `G-P.heading`) | live readout from the moving base; traced path | |
| 06_4 | Add six seconds of velocity onto the pose, and you predict three meters straight ahead. The exponential gets it right. A steady velocity carries the base around a fixed point. | A dashed red straight path to (3.00, 0.00) with a ghost base there, crossed. The true arc's centre appears as a gold dot with a radius line, 0.95 m (`G-P.naive`, `G-P.radius`, `G-P.centre`) | highlight; live number | |
| 06_5 | On a floor, that is every steady motion there is. A turn about some point. Or, with no spin, a straight slide. | Four more velocity triples play as small paths: circles about four different gold points, and one straight line labelled "spin 0". Tag: "model" (`G-X5.floor`) | lagged reveal; many small paths | |

- **Misconception defused:** "velocity times time is where you end up".
  True for the three numbers of a slide; false once the spin is not zero.
- **Ruth:** the velocity is the base's own (forward and sideways are read in
  its frame), held constant; the pose reached is the exponential of the
  3 by 3 velocity matrix times six seconds. "Every steady motion is a turn
  about a point or a slide" is tested on 45 velocities. The base in the
  quiz moves at half of KUKA's published top speed (3.6 km/h); KUKA
  publishes no turning rate, and the grid, the speeds and the paths are
  labelled "model".
- **Dev:** the wheel formula with its source; the senses are computed.

## 07_screws (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Now leave the floor. In space, a velocity is two arrows. A spin, and a slide. Six numbers, which roboticists call a twist. | The book in space with an orange spin arrow and a teal slide arrow; six numbers in two triples under the word **twist** (`G-S.numbers.twist`) | 3D book; readout | |
| 07_2 | Hold a twist steady, and you get the motion of a screw. The body turns about an axis, and slides along that same axis. | The book corkscrews along a gold axis, leaving a helix; the axis carries a curled arrow and a straight one | **the screw** (axis, helix); traced path | |
| 07_3 | A door is a screw with no slide. A drawer is a screw with no turn. A bottle cap does both. | Three small scenes, each with its gold axis: a door swinging, a drawer sliding, a cap rising as it turns | lagged reveal | |
| 07_4 | Here is the surprise. Pick up the book. Put it down anywhere, facing any way. One screw would have carried it there. Every time. | The book at rest, then a ghost of it a meter right, a meter away, half a meter up and a quarter turn round. A gold vertical axis appears off to the side, through the point (0, 1); the book replays the move as one corkscrew about it. Readout: turn 90°, slide 0.5 m (`G-S.toss.*`, `G-X3.one_screw`) | 3D screw; ghost; live readout | |
| 07_5 | Michel Chasles published that in eighteen thirty. Giulio Mozzi had found the axis in seventeen sixty-three. | Lower third: "Mozzi 1763 · Chasles 1830: every motion of a rigid body is a screw motion". The axis stays | lower third | |
| 07_6 | ⟨predict⟩ So how many numbers does a pose in space need? | Quiz card: "numbers in a pose?" Choices 3, 6, 9. Ring from 4 (`G-Q.07_6.*`) | quiz card; countdown | listen 4 s |
| 07_7 | Six. Three for position, three for orientation. Yet software stores nine numbers for an orientation, or four. The spare ones are the price of having no bad spots. | A tally: position 3 + orientation 3 = 6. Then three boxes: "three angles: 3 numbers, bad spots" (red), "rotation matrix: 9 numbers, 6 rules" and "unit quaternion: 4 numbers, 1 rule" (both green) (`G-S.numbers.*`) | table built from primitives; highlight | |

- **Misconception defused:** "a motion in space is a turn plus a slide in
  some direction, and those are separate things". The slide can always be
  taken along the turn's own axis, once the axis is put in the right place.
- **Ruth:** Chasles's theorem with its edge: a pure slide is a screw with
  no turn (the axis is then only a direction), tested on 312 poses with
  turns from none to 150°. Mozzi's priority follows Ceccarelli (2000); the
  dates are of publication. A quaternion and its negative name the same
  orientation; a matrix's nine numbers obey six rules.
- **Dev:** why pose code carries redundant numbers.

## 08_seven_screws (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Back to the arm. Standing straight up, it is one point three meters tall. Each of its seven joints is a hinge. A hinge does one thing. It turns about its own axis. | The arm at home, straight up, with a height mark 1.306 m. Seven axis lines appear bottom to top, numbered 1 to 7, alternately vertical and level (`G-A.height`, `G-A.axis.*`, `G-A.axis_height.*`) | **the arm**; lagged reveal; labels | |
| 08_2 | A turn about an axis is a screw with no slide. So the arm is seven screws, stacked. | Each axis gains the curled arrow of chapter 07's screw, in gold, one after another | lagged reveal; highlight | |
| 08_3 | ⟨predict⟩ Suppose only joint four turns, the elbow. Which parts of the arm move? | The elbow's axis glows gold. Quiz card: "everything", "only what is past the elbow", "only the forearm". Ring from 4 (`G-Q.08_3.*`) | quiz card; countdown | listen 4 s |
| 08_4 | Everything past the elbow, as one rigid piece. Nothing before it. Each joint applies its screw to everything further out. | The elbow turns a quarter turn: forearm, wrist and gripper swing together; the lower arm stays grey and still (`G-K.elbow.moves`) | moving arm; highlight | |
| 08_5 | That gives a recipe. Start with the gripper's pose when every joint is at zero. Apply the last joint's screw, then the one before, down to the first. | The arm at home with its gripper pose marked M. Screws light from the wrist down to the base, one at a time; each time, the part of the arm above that joint swings | moving arm; lagged reveal | |
| 08_6 | Each screw, held for its joint angle, is an exponential. So the gripper's pose is a product of seven exponentials, the line from the start. Roger Brockett presented it in nineteen eighty-three. | The line builds right to left as the screws lit: `M`, then `e^{[S_7]q_7}`, …, then `e^{[S_1]q_1}`, giving `T(q) = e^{[S_1]q_1} e^{[S_2]q_2} \cdots e^{[S_7]q_7} M`. Lower third: "the product of exponentials · Brockett, 1983" (`G-L.brockett.year`) | TeX built term by term | |

- **Misconception defused:** "you must track every link's frame to find the
  gripper". Seven axes at zero and one home pose are enough.
- **Ruth:** the formula is the space form (Lynch and Park eq. 4.14): the
  screws `S_i` are the joint axes with the arm at zero, in the room's
  frame. Brockett spoke at MTNS in Beer Sheva in 1983; the proceedings are
  dated 1984.
- **Dev:** `q` is the list of joint angles, in the order of the joints.

## 09_by_hand (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Try it by hand, with the real arm's lengths. | The arm at home with four lengths marked, in meters: 0.36 to the shoulder, 0.42 upper arm, 0.40 forearm, 0.126 from wrist to the gripper's tip (`G-A.shoulder_height`, `G-A.upper_arm`, `G-A.forearm`, `G-A.wrist_to_tip`) | labels; braces | |
| 09_2 | Bend two joints a quarter turn each, the shoulder and the elbow. The recipe says, elbow first. With the arm still straight up, the elbow's screw swings the forearm over until it lies level. | `e^{[S_2]\,90°}\, e^{[S_4]\,90°}\, M` on screen; the right factor glows. The forearm swings about the elbow's gold axis; the gripper's tip readout goes from (0, 0, 1.306) to (−0.526, 0, 0.780) (`G-K.ell.after_elbow`) | moving arm; live readout; TeX highlight | |
| 09_3 | Then the shoulder's screw swings everything, about the shoulder's axis, just where it stood at zero. The upper arm comes down level, and the forearm points straight up. | The left factor glows; the whole arm above the shoulder swings a quarter turn about the shoulder's axis. The tip lands at (0.420, 0, 0.886): 0.42 forward, and 0.36 + 0.40 + 0.126 up (`G-K.ell.p`) | moving arm; live readout; sum written out | |
| 09_4 | Notice what the recipe never asked. Where the elbow's axis is now. Each screw is written down once, with the arm at zero. | The elbow's axis where it is now (grey) against where the formula uses it (gold, at home, dashed) | highlight | |
| 09_5 | ⟨predict⟩ Now joint one, at the base, turns a quarter turn about the vertical. Where does the gripper go? | Joint one's vertical axis glows. Quiz card: "higher", "round to the side, same height", "nowhere". Ring from 4 (`G-Q.09_5.*`) | quiz card; countdown | listen 4 s |
| 09_6 | Round to the side, at the same height. It was forty-two centimeters in front of the base. Now it is forty-two centimeters to the side. | The arm swings a quarter circle about the vertical; the tip readout runs from (0.420, 0, 0.886) to (0, 0.420, 0.886); a third factor joins the line on the left (`G-K.quarter.p`) | moving arm; live readout | |

- **Misconception defused:** "the first joint acts first". The joint
  nearest the gripper acts on the home pose first; the base's screw acts
  last and carries everything.
- **Ruth:** the worked pose is joints 2 and 4 at 90°, the others at zero;
  its tip is pinned by hand (0.42 forward; 0.886 up) and by the oracle's
  walk along the chain. Joint 4's axis points along minus y at zero, which
  is why the forearm first swings backward.
- **Dev:** the product is taken in the order of the joints.

## 10_velocities_add (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Now speeds. If the joints are turning, how is the gripper moving? | The arm in its L shape; small curled arrows at two joints start to turn | the arm | |
| 10_2 | Each joint, turning alone, moves the gripper along that joint's screw. That is a twist. And twists live on the flat piece, so they add. | Shoulder alone: an orange twist arrow at the gripper. Base joint alone: another. Both turning: the two arrows add tip to tail into one | arrows adding; moving arm | |
| 10_3 | Seven joints, seven twists, six numbers each. Stand them side by side. A grid, six rows by seven columns, called the Jacobian. Joint speeds go in. The gripper's twist comes out. | Seven orange columns rise beside the arm, one per joint, each tied to its joint by a thin line; they close up into a 6 by 7 grid. `V = J(q)\,\dot q` (`G-J.shape`, `G-J.ell.col2`) | **matrix** from columns; TeX | |
| 10_4 | ⟨predict⟩ The arm is in an L shape. Which joint moves the gripper's tip fastest, at the same turning speed? Joint one, two or seven? | Joints 1, 2 and 7 glow in turn, each named on the arm (base, shoulder, wrist). Quiz card with the three choices. Ring from 5 (`G-Q.10_4.*`) | quiz card; countdown | listen 5 s |
| 10_5 | Joint two. What counts is how far the tip is from each axis. Sixty-seven centimeters from the shoulder's. Forty-two from the base's. And the tip sits right on joint seven's axis, so that joint only spins it. | Three gold lever lines from each axis to the tip with their lengths: 0.67 m, 0.42 m, 0.00 m; three velocity arrows at the tip in proportion (`G-J.ell.lever.*`) | braces; arrows in proportion | |
| 10_6 | And the columns change as the arm moves, because its axes move with it. | The arm moves through three poses; the grid's entries run as live numbers. Lower third: "the Jacobian: the arm's flat piece at the pose it is in" | live numbers recomputed from the pose | |

- **Misconception defused:** "a joint nearer the gripper moves it more".
  The lever is the distance from the joint's axis, wherever the joint is.
- **Ruth:** the Jacobian is the space Jacobian (Lynch and Park eq. 5.11):
  column i is joint i's screw carried to the present pose. The "speed of
  the tip" is the linear velocity of the tip point, lever times joint
  speed.
- **Dev:** six rows (three of spin, three of slide), seven columns.

## 11_lost_directions (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | Six rows means six independent directions for the gripper to move in. Usually, seven joints cover all six. | The arm bent, with a readout "directions: 6" and six small arrows at the gripper (three slides, three spins) (`G-J.rank.swing`) | readout; arrows | |
| 11_2 | ⟨predict⟩ Here the arm is stretched straight out. Can the gripper still move in all six directions? | The arm stretches level. Quiz card: "all six", "some are lost". Ring from 4 (`G-Q.11_2.*`) | quiz card; countdown | listen 4 s |
| 11_3 | No. It can't move further out, and it can't even begin to pull straight back. Along the arm, every joint's push is zero. Four of the six directions are left. | The readout drops to 4. Each joint wiggles in turn and its orange arrow at the gripper is shown: none has any part along the arm; the red dashed arrow along the arm stays empty (`G-J.rank.stretch`, `G-J.stretch.along_arm_missing`) | live readout from the pose; highlight | |
| 11_4 | A pose like this is called a singularity. Close to one, small motions of the gripper cost huge joint speeds. | The word **singularity**. The elbow bends a little; a speed gauge beside it | card; gauge | |
| 11_5 | Pull the gripper straight back at ten centimeters per second. With the elbow bent thirty degrees, it must turn at fifty-four degrees per second. Bent ten degrees, a hundred and sixty. Its limit is seventy-five. | A bar for the elbow's speed against its bend, stepping through 30°, 10°, 3°, 1° (54, 160, 534 and 1,602 degrees per second); a red limit line at 75°/s which the bar crosses between the first two. Tag: "model: geometry only" (`G-J.blowup.*`) | live bar; limit line; highlight | |
| 11_6 | You have met this before. The wrist is three joints whose axes meet at one point. A gimbal. Straighten it, and two of its axes line up, exactly like the rings. | Close on the wrist: its three axes; the middle joint straightens and joints 5 and 7 fall into one red line. Chapter 03's gimbal fades in over it, its two rings in the same line | 3D close-up; morph to the gimbal | |

- **Misconception defused:** "a singularity is where the arm cannot reach".
  The stretched arm is inside its reach; it is the directions it can move
  in that collapse.
- **Ruth:** "directions" is the rank of the Jacobian: 6 at the bent pose, 4
  stretched straight out (elbow, shoulder and wrist all aligned), 5 with
  only the elbow straight, 3 standing straight up. The elbow's speed comes
  from the triangle shoulder, elbow, wrist and is the same for any way of
  sharing the motion among the other joints. The wrist with joint 6 at zero
  has two axes in line; the whole arm, with its spare joint, still reaches
  all six directions there (`G-J.rank.ell`), which is the next chapter.
- **Dev:** the limit is KUKA's published speed for axis 4 at rated payload.

## 12_the_spare_joint (~55 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Seven joints. Six numbers in a pose. One joint is spare. What does it buy? | The tally 7 − 6 = 1 beside the arm (`G-J.spare`) | TeX | |
| 12_2 | ⟨predict⟩ Try it on yourself. Lay your palm flat on a desk and keep your shoulder still. Can your elbow still move? | An outline of a human arm, palm on a desk, shoulder pinned. Quiz card: "it can", "it can't". Ring from 5 (`G-Q.12_2.*`) | quiz card; countdown | listen 5 s |
| 12_3 | It can. It swings on an arc while your hand stays put. Counted the way a roboticist counts, your arm has seven joints too. | The outline's elbow swings on an arc; the count 3 + 1 + 3 = 7 at shoulder, elbow, wrist | traced arc; labels | |
| 12_4 | The robot does the same. Its gripper holds one pose, while its elbow swings on a circle twenty-nine centimeters in radius. | The arm in the L shape with its wrist bent; the elbow travels its gold circle about the line from shoulder to wrist, as far as the joints allow; a readout "gripper moved: 0.000 m" stays at zero; the radius 0.29 m (`G-J.elbow.radius`, `G-J.swing.null_twist`) | the arm moved along computed joint angles; live readout; traced path | |
| 12_5 | In the Jacobian, that is a set of joint speeds whose twists add up to nothing. | The seven orange arrows at the gripper cancel to nothing. An obstacle slides in and the elbow swings out of its way. Lower third: "the null space of the Jacobian: used to dodge obstacles and to stay clear of singularities" | arrows cancelling; lower third | |

- **Misconception defused:** "one pose, one set of joint angles". For this
  arm one pose has a whole circle of them.
- **Ruth:** the palm on the desk is Lynch and Park's Exercise 2.2; "counted
  the way a roboticist counts" marks the seven-joint arm (three at the
  shoulder, one at the elbow, three at the wrist) as a model (their
  Exercise 2.3 notes the forearm is really a closed chain). The radius is
  the height of the triangle with sides 0.42, 0.40 and the distance from
  shoulder to wrist at that pose. The swing stops where a joint reaches its
  limit; no extent is claimed.
- **Dev:** what the freedom is used for in practice.

## 13_steps (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | Now the question a robot really faces. Here is the pose I want. Which joint angles get me there? | The arm over a table, gripper pointing down; a mug; the goal pose in pink at the mug, a quarter turn round and lower (`G-I.start.*`, `G-I.goal.*`) | the arm; goal marker | |
| 13_2 | There is rarely a tidy formula, so the arm takes steps. First, compare the pose it has with the pose it wants. One screw joins them. Read as a twist, that screw is the error. | A gold screw axis appears between the gripper and the goal; the six numbers of the twist read out. Lower third: "the logarithm: the twist that would reach the goal in one second" | 3D screw; readout; lower third | |
| 13_3 | Second, ask the Jacobian which joint speeds make that twist, and move the joints that way. Third, look again, and repeat. | A loop of four boxes: pose, error twist, Jacobian, joint step. A code panel under "for programmers" with the three lines of the loop | flow diagram; code panel | |
| 13_4 | ⟨predict⟩ The gripper starts half a meter from its goal, and a quarter turn off. How many steps until it is within a tenth of a millimeter? | Quiz card with three choices: 3, 30, 300. Ring from 5 (`G-Q.13_4.*`) | quiz card; countdown | listen 5 s |
| 13_5 | Three. Half a meter off. Then nine centimeters. Then four millimeters. Then three hundredths of a millimeter. It is Newton's method, running on a curved space. | The arm takes its three steps; a table fills: step, distance left, angle left: 505.0 mm and 90.00°; 93.9 mm and 8.64°; 4.2 mm and 0.72°; 0.03 mm and 0.01°. Bars shrink on a logarithmic scale. Lower third: "Newton's method; stops at 0.001 rad and 0.1 mm, as in Lynch and Park's example" (`G-I.ik.*`) | the arm on computed joint angles; table; bars | |
| 13_6 | Why not just subtract angles? Because of the two traps. These two orientations are one point four degrees apart. Yet two of their three angles differ by ninety. | Two books, almost identical. Under them their three ring angles: (0°, 89°, 0°) and (90°, 89°, 90°), the differences 90°, 0°, 90° in red; between the books a small orange arrow: 1.41° (`G-E.near.*`) | 3D books; highlight | |

- **Misconception defused:** "the computer just solves for the angles".
  It walks, and each step is the flat-piece trick of chapter 04.
- **Ruth:** the method is Lynch and Park §6.2.2 with the error read in the
  gripper's own frame and the pseudoinverse of the body Jacobian; the
  tolerances are their Example 6.1's. Three steps is for this start and
  this goal; Newton's method needs a start that is close enough, and the
  fact sheet says so. For this arm a closed formula does exist (it is a
  shoulder, elbow, wrist design), which is why the line says "rarely". The
  two orientations of 13_6 sit one degree from gimbal lock.
- **Dev:** the loop as code; the stopping rule.

## 14_the_straight_path (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | One more question. Between the start and the goal, what should the gripper do on the way? | The start pose and the goal pose, the arm faint behind them | the arm; markers | |
| 14_2 | ⟨predict⟩ Blend the two orientations, number by number. Half of one matrix plus half of the other. Is the result halfway there? | Two 3 by 3 grids and their average between them. Quiz card: "halfway", "not even an orientation". Ring from 4 (`G-Q.14_2.*`) | matrices; quiz card; countdown | listen 4 s |
| 14_3 | It is not even an orientation. A gripper turned by that blend would be squashed to half its volume. A blend of two different turns is never a turn. | A small gripper drawn through the blended matrix at steps 0, ¼, ½, ¾, 1: it shrinks in the middle and recovers. Readout: volume 0.50 at halfway (`G-I.blend.det`, `G-X4.blend`) | 3D shape through a matrix; live number | |
| 14_4 | The honest path is the screw. Take the twist that joins the two poses, and hold it for half a second. Half the turn and half the slide, and every pose on the way is a true pose. | The gripper corkscrews from start to goal about the gold vertical axis: a quarter turn and 0.35 m down. At halfway the readouts: turned 45°, volume 1.00 (`G-I.screw.*`) | 3D screw; live readouts; traced path | |
| 14_5 | Real robots often let the tip travel a straight line instead. Here the screw bows away from that line by seven point six centimeters. | The straight line from start to goal in teal beside the screw's helix; the largest gap between them marked: 7.6 cm. Tag: "for programmers: split the path, a straight line for the tip and the turn on its own" (`G-I.bow`) | two paths; brace | |

- **Misconception defused:** "any mix of two valid things is valid".
  Orientations are not closed under averaging; the mix leaves the space.
- **Ruth:** the determinant of the blend of two turns that differ by an
  angle a is cos²(a/2): one only when the turns are equal (tested on 78
  turns), which is what "never" rests on. The screw path is Lynch and Park
  eq. 9.6; the split path is their eqs. 9.7 and 9.8.
- **Dev:** what motion planners do, and the size of the difference here.

## 15_everywhere (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | The same moves run far beyond this arm. Drones steer poses. Mapping robots average them. A humanoid's whole body floats, so its state is one free pose, plus the angle of every joint. For this one, twenty-nine. | Two small scenes (a quadrotor tilting, a map with pose markers pulled into line), then a stick figure of the Unitree G1 (lower third with its name) with a pose marker at its pelvis and dots at its joints; the tally 6 + 29 = 35 numbers of velocity. Lower third, names to search: "Lie group · SE(3) · screw theory · product of exponentials" (`G-C.g1.*`) | **the humanoid** (projected 3D skeleton); tally; lower third | |
| 15_2 | And a physics engine steps every body forward by a small twist. How it steps matters. Add the spin onto the nine numbers a thousand times, and the body has swollen by ten percent. With the exponential, it stays rigid. | Two spinning books side by side under "add" and "exponential"; the left one visibly grows; live volume readouts ending at 1.105 and 1.000. Tag: "model: 1,000 steps of 0.01 s". Lower third: "FrankenSim steps bodies by the exponential (fs-time, se3_exp_step)" (`G-C.drift.*`) | two live readouts; 3D books | |
| 15_3 | ⟨predict⟩ Last one. Tip the book one degree, then roll it one degree. Then the other order. Do the two results still differ? | The opening's two books, turning by one degree each way. Quiz card: "the same", "still different". Ring from 4 (`G-Q.15_3.*`) | quiz card; countdown | listen 4 s |
| 15_4 | They do, by less than two hundredths of a degree. And that leftover is itself a turn, about the one axis neither turn used. | The two results overlaid and magnified; the leftover turn's axis appears vertical; readout 0.0175° (`G-B.leftover.1.*`) | scaled group as a zoom; live number | |
| 15_5 | For quarter turns the leftover was a third of a turn. For small turns it shrinks like the product of the two angles. That is the Lie bracket. It measures how much order matters. | A three-row table: 1° and 1° leaves 0.0175°; 10° and 10° leaves 1.74°; 90° and 90° leaves 120°. Lower third: "the Lie bracket" (`G-B.leftover.*`) | table from primitives; highlight | |
| 15_6 | So the arm does not add orientations. It multiplies motions, and does its adding on the flat piece. That is what it means to think in Lie groups. | The opening's two books above the arm reaching its goal; the seven-factor line under them. Then the receipts card: "every number in this video was computed by code" with the repository link | card; receipts | hold |

- **Misconception defused:** "order matters, full stop". It matters by an
  amount, the amount is a turn, and for small turns it is second order.
- **Ruth:** the G1's state is a pose and 29 joint angles; its velocity has
  35 numbers, while a pose stored as a position and a quaternion makes 36
  numbers of state. The swelling is the determinant after 1,000 forward
  Euler steps of 0.01 s at 1 rad/s, (1 + 10⁻⁴)¹⁰⁰⁰; the exponential's stays
  1 to rounding. The leftover is the group commutator of the two turns; its
  angle tends to the product of the angles (in radians) and its axis to
  the third axis.
- **Dev:** the engine named is the owner's; the lower third cites the
  function.
