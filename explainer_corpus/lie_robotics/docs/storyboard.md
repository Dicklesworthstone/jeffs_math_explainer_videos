# Storyboard — Why Robot Arms Think in Lie Groups

The hand-edited source of this video's words and pictures. `python3 -I
tools/storyboard.py lie_robotics` checks it (gate G2) and writes `script.tsv`
from the words column. Facts: `docs/fact-sheet.md` and `oracle.py` (every
number below names its fact key); brief: `docs/brief.md`; reviews and their
dispositions: `docs/reviews/lie_robotics/` (this is the storyboard as
revised after round R1).

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a question for the
viewer; the beat ends with the question and the picture holds before the
next beat answers it. The seconds in each chapter heading are the gate's
own estimate (`tools/storyboard.py`).

**The quiz device.** The owner asked for "quizzes with pauses before the
answers", so every chapter has one. A quiz is a `⟨predict⟩` beat whose last
column says `listen N s`: a violet card with the question and its choices,
set beside the picture and never over it; a ring that drains over N seconds
with the seconds left in its middle; the line "pause for more time" under
it; the narration silent and the music alone, holding its motif open until
the answer lands. The next beat gives the answer in the same place on
screen, and the picture shows why. A readout that would give the answer
away stays hidden until the countdown ends. Every quiz is a row of
`model::QUIZZES`; its answer is computed (`G-Q.<line>.answer`), never typed.

**Three-dimensional pictures.** The book, the gimbal, the arm and the
humanoid are drawn by the video's own projection (`chapters/draw.rs`: one
fixed view from the front right and above, about 32° round and 22° up; each
3D world one redrawn group, faces culled and parts sorted back to front,
farther parts dimmer by colour) on franken_manim's retained route, so
matrices, readouts and quiz cards share the frame with them and the layout
lint reads every chapter. Axes: x to the right, y away from the viewer, z
up. A close-up is the same projection with a larger scale.

**The screw, drawn one way.** Wherever a screw appears (a joint's axis, the
book's move in 07, the error in 13, the path in 14) it is the same object:
a gold axis that runs out of frame at both ends, a curled arrow for the turn
and a straight one for the slide, and when the motion is shown, its thread
drawn faintly past both ends of the part actually travelled. Chapter 14's
shot is chapter 01's last frame again.

**The badge.** A small circle with an orange line touching it (chapter 04's
picture) sits in a corner whenever the one move is being made: measure on
the flat piece, then wrap the answer back on. It appears in 05, 06, 10, 13
and 14.

**Four words, one card.** Shown at 07_3 and brought back small at 10_2 and
13_3: a turn is how far, turning only; a spin is how fast, turning only; a
screw is how far, turning and sliding; a twist is how fast, turning and
sliding.

## 2 The arc

One picture carries the video: **the screw** (a turn about an axis with a
slide along it). A spin held for a second is a screw with no slide; a
platform on a floor turns about a point; every motion in space is one screw;
every joint is one; the gripper's velocity is a sum of them; the error to a
goal is one; the steady path is one. Under it sits one move, made in chapter
04 and repeated to the end: the space is curved, so do the arithmetic on the
flat piece that touches it, then wrap the answer back on.

| Act | Chapters | Spine |
|---|---|---|
| I. Orientations refuse to add | 01 the book, 02 one turn, 03 three angles, 04 halfway | two quarter turns in two orders; turns form a group; three angles have a bad spot (Apollo); averaging angles points the wrong way; the flat piece |
| II. The flat piece and the way back | 05 spin, 06 the platform, 07 screws | a spin held one second (the exponential, Rodrigues); a platform's arc on a floor; in space every motion is a screw (Chasles, Mozzi) |
| III. The arm | 08 seven screws, 09 by hand | each joint is a screw; the product of exponentials (Brockett); two joints by hand with the arm's real lengths, then a third |
| IV. Moving | 10 velocities add, 11 lost directions, 12 the spare joint | the robot's real question is asked; the Jacobian; where the arm loses a direction and what that costs; the elbow's free circle |
| V. Getting there | 13 steps, 14 the steady path | Newton's method on the group, three steps from this start; the screw between two poses against a blend of matrices |
| VI. Everywhere | 15 everywhere | drones, maps, a physics engine's step, a humanoid's floating body; the book again: how much order matters is itself a turn |

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses
a data colour. `hero`, `warn`, `good` and the quiz card's violet are the
corpus's own.

| Idea | Color |
|---|---|
| orientation and turns (the book's cover, a ring, a turn's arc) | blue `#4A8DF8` |
| position, a path travelled | teal `#3FE0E0` |
| the flat piece: every velocity arrow (a spin curls, a slide is straight), twists, the Jacobian's columns | orange `#D48F3F` |
| a screw's axis, and the joint in question | `hero` gold `#FFD866` |
| the goal pose | pink `#EC4E97` |
| the arm's links and joints at rest | steel `#CFD7E5` on `dim` |
| a lost direction, a bad spot, a wrong path (always dashed or crossed) | `warn` red `#FC6255` |
| a confirmed answer (ticks and equals signs only) | `good` green `#83C167` |
| the quiz card and its countdown | violet `#B189D6` |

Every pair of the nine is at least ΔE2000 20 apart (least 20.7, pink
against violet); `palette::tests` computes it.

## Vocabulary (one word per idea)

**turn** (a rotation, as an action or its result; never "rotation" except in
the on-screen name "rotation matrix"); **quarter turn**, **third of a turn**;
**tilt** and **roll** (the book's two turns); **slide** (a translation);
**orientation** (which way a thing faces), **position** (where it is),
**pose** (both together); **doing nothing** (the identity) and **undo** (the
inverse); **group**; **matrix** or **grid** (of numbers); **ring** and
**gimbal**; **heading**; **the flat piece** (the tangent space; the formal
name appears once, on screen) and **wrap the answer back on**; **spin** (a
steady rate of turning, an arrow) and **arrow** (a vector); **the
exponential**; **twist** (six numbers: a spin arrow and a slide arrow);
**screw** (a turn about an axis with a slide along it); **joint**, **joint
angle**, **gripper** and the gripper's **tip**, **shoulder**, **elbow**,
**wrist**, the arm's **base**; **platform** (the mobile one); **Jacobian**;
**direction** (a way the gripper can move); **singularity**; **spare
joint**; **step**; **goal**; **blend** (number-by-number averaging);
**leftover** (what separates the two orders). Never "rotation vector",
"tangent space", "manifold", "end effector", "null space", "commutator" or
"geodesic" aloud: each is a lower third.

## 01_the_book (~64 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | ⟨predict⟩ Tilt a book a quarter turn about one line on the table. Then roll it a quarter turn about another. Now swap the order. Roll first, then tilt. Does the book end up the same way? | Two copies of a book on a table, side by side, drawn in three dimensions (blue cover up, spine on the left). Two gold lines are painted on the table under each: one left to right, one pointing away. Left copy: tilt (far edge up, about the first line), then roll (to the right, about the second). Right copy stays still, greyed. Beside them, not over them, the violet quiz card: "same or different?" with the ring draining from 5 (`G-Q.01_1.*`) | **the book**; **the quiz card**; 3D turns from trackers; countdown ring on the scene clock | listen 5 s (the countdown: the music alone) |
| 01_2 | No. Tilt first, and the cover ends up facing you. Roll first, and it faces the wall on your right. | The right copy now does roll, then tilt. Both books hold their final orientations; an arrow out of each cover: toward the viewer, and to the right. Lower third: "turn about the book's own edges instead and the two answers trade places: still different" (`G-B.tilt_then_roll.cover`, `G-B.roll_then_tilt.cover`) | 3D turns; highlight; lower third | |
| 01_3 | *(title)* | **Why Robot Arms Think in Lie Groups**, and small under it: (say "Lee") | title card | listen 3 s (the motif) |
| 01_4 | Sliding is nothing like this. Slide the book right, then away, or away, then right. Same spot. Positions add. Orientations don't. | Both books flat again; each slides right and away in opposite orders along teal paths, landing on the same teal dot. Two lines build: "positions add" (ticked), "orientations don't" (crossed) (`G-B.slides.same`, `G-B.turns.same`) | morph; highlight | |
| 01_5 | This is a KUKA robot arm with seven joints. Its job is to put its gripper at a pose, a position and an orientation together. | The arm (lower third: KUKA LBR iiwa 14 R820) reaches toward the book on its table; the goal pose glows pink: a dot for position, three short axes for orientation. Labels: position (teal), orientation (blue), "pose = both" | **the arm** (projected 3D, one redrawn group); lagged reveal | |
| 01_6 | So its software must do arithmetic with things that refuse to add. The way out is about a hundred and fifty years old, and by the end you'll read it in one line. | The arm's gripper corkscrews down onto the book along a gold screw axis (chapter 14's shot, in outline). Above it the line `T(q) = e^{[S_1]q_1} e^{[S_2]q_2} \cdots e^{[S_7]q_7} M` writes itself in dim ink (`G-L.lie.year`) | TeX formula; moving arm; **the screw** | |

- **Cold open** (plan rule 20): a puzzle the viewer can try with a real
  book, answered at once; the title lands straight after the answer; then
  the payoff previewed as the line the video will earn.
- **Misconception defused:** "turning is like moving: do the parts in any
  order". Shown false on a real object before any symbol appears.
- **Ruth:** the two turns are made about lines fixed on the table, so the
  later turn multiplies on the left; the matrices are `G-B.*.R`. About the
  book's own edges the two outcomes are exchanged, which the lower third
  says. "Positions add" is commutativity of vector addition; "orientations
  don't" is said of these two turns, never as "never".
- **Dev:** "about a hundred and fifty years old": Lie began his theory of
  continuous groups in the winter of 1873–74 (MacTutor).

## 02_one_turn (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Look where tilt then roll left the book. It could have got there in a single turn, about a single axis. | One book, with three equal arrows along its edges. Its start (ghost, flat) and its end (cover toward the viewer, on its side). A gold axis appears through the corner where the three arrows meet, and the book swings from ghost to end in one turn about it | 3D turn; ghost; **the screw**'s axis | |
| 02_2 | ⟨predict⟩ Two quarter turns went in. How big is that single turn? | The quiz card with three choices: ¼ turn, ⅓ turn, ½ turn; the ring drains from 5 (`G-Q.02_2.*`) | quiz card; countdown | listen 5 s |
| 02_3 | A third of a turn. Three of them bring the book home. Its axis leans equally from the book's three edges. | The single turn plays three times, a live angle readout climbing 120°, 240°, 360°: the book is home. The axis is shown making the same angle with each edge arrow (`G-B.tilt_then_roll.angle`, `.axis`) | live number on a tracker; 3D turn | |
| 02_4 | That always works. Any turn followed by any turn is one turn about one axis. Euler showed it in seventeen seventy-five. | Three more pairs of turns flash by small, each collapsing to one gold axis and one arc. Lower third: "Euler, 1775: however a sphere is turned about its centre, some diameter ends where it began" (`G-X1.one_turn`, `G-L.euler.year`) | lagged reveal; highlight; lower third | |
| 02_5 | And any turn can be undone. A collection where you can combine any two, and undo any one, is called a group. In this one, order usually matters. | The book turns and turns back. The word **group**, with "precisely: and combining is associative" small under it. Then two panels: tilt then roll against roll then tilt, a crossed "="; two turns about one axis in both orders, a green "=". Lower third: "about one axis, order never matters" (`G-X2.*`) | card; lower third | |
| 02_6 | To compute, track where the book's three edges point. Three arrows, three numbers each. Nine numbers in a grid, a matrix. One turn after another is one grid times another. | The three edge arrows; as the book turns their coordinates fill the columns of a 3 by 3 grid. Then the grids for tilt and roll combine into the grid of `G-B.tilt_then_roll.R` (the result only, no working). "For programmers" tag: `R = roll · tilt`: turns about the room's lines, so the later turn goes on the left; the columns are where the edges point | **matrix** built from primitives; lagged reveal; "for programmers" tag | |

- **Misconception defused:** "two quarter turns make a half turn". They make
  a third of a turn about an axis neither turn used.
- **Ruth:** "any turn followed by any turn is one turn": Euler's theorem
  (E478, presented 9 October 1775, printed 1776), on screen in his own
  terms; edge-tested on 192 products, including products that are half
  turns and products that are no turn. Associativity is on the card of
  02_5 and not spoken. "Usually": of the 576 ordered pairs of a cube's 24
  turns, 120 do not care about order (`G-X2.order_free_pairs`, in the fact
  sheet), and two half turns about perpendicular axes also swap.
- **Dev:** the tag states the order of the product and the two conventions
  that make it right.

## 03_three_angles (~77 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Nine numbers is a lot for one orientation. Three should do. Here is a machine that tries, a gimbal. Three rings, one inside the next, each turning about its own axis. | The book inside three nested blue rings: the outer about the vertical, the middle about the line pointing away, the inner about the left to right line (`G-E.rings.order`); three live angle readouts. Each ring wiggles in turn and the book answers with three visibly different motions | **the gimbal** (three projected rings, cut into arcs for depth); live numbers; 3D turns | |
| 03_2 | ⟨predict⟩ Turn the middle ring a quarter turn. Now the outer ring's axis and the inner ring's axis line up. Do those two rings still do different things? | The middle ring turns to 90°; the outer and inner axes swing into one line (dashed red). Quiz card: "different, or the same?", ring from 4. The "ways to turn" readout is hidden (`G-Q.03_2.*`) | quiz card; countdown; highlight | listen 4 s |
| 03_3 | No. They turn the book about the very same axis. Nothing is stuck, but three rings now give only two ways to turn. This is gimbal lock. | Outer ring wiggles, then inner: the book makes the same motion both times. The readout "ways to turn" appears at 3 and drops to 2; the missing axis is drawn dashed red (`G-E.lock.same`, `G-E.left.90`) | live number; highlight | |
| 03_4 | Apollo flew with it. Its guidance platform hung in three gimbals. Past seventy degrees on the middle one, a warning lamp lit. Past eighty-five, the computer dropped its sense of direction. | The same gimbal; its middle ring sweeps up while a bar beside it, "of the third way to turn, this much is left", shrinks: 34% at 70° (lamp), 9% at 85° (dashed red). Lower third: "Apollo guidance computer: gimbal lock lamp above 70°, reference dropped above 85°. The bar is this video's own measure" (`G-L.apollo.*`, `G-E.left.70`, `G-E.left.85`) | live bar on a tracker; lower third | |
| 03_5 | On Apollo eleven, Michael Collins radioed a request. How about sending me a fourth gimbal for Christmas. With a fourth ring, there is always one to spare. | A quote card: "How about sending me a fourth gimbal for Christmas." Michael Collins, Apollo 11, 105 hours into the flight. Then a fourth ring fades in round the gimbal | quote card | hold |
| 03_6 | It wasn't bad engineering. Name every orientation with just three numbers, and somewhere there is a bad spot. So software stores nine numbers, or four. The spare ones are the price. | Three boxes: "three angles: 3 numbers, a bad spot" (dashed red); "rotation matrix: 9 numbers − 6 rules = 3"; "unit quaternion: 4 numbers − 1 rule = 3" (both ticked). Lower third: "precisely: no continuous one-to-one naming of all orientations by three numbers exists" (`G-S.numbers.*`, `G-X6.ring_orders`) | table built from primitives; lower third | |

- **Misconception defused:** "gimbal lock means the rings jam". Nothing
  jams; two rings start doing the same job, so one way of turning has no
  ring left.
- **Ruth:** the rings are yaw, pitch, roll with `R = Rz Ry Rx`, drawn in
  that order; at a quarter turn of the middle ring only roll minus yaw
  matters (`G-E.lock.same` is the largest difference between the two
  matrices: zero). The bar is the volume spanned by the three ring axes,
  cos of the middle angle, and is labelled as the video's own. The
  thresholds are the command module computer's (Comanche 055, GLOCKMON:
  lamp above 70°, coarse align and NO ATT above 85°); the gimbals carried
  the platform that held the reference. The quote is verbatim from NASA's
  air-to-ground transcript, 04 09 00 30. 03_6 is a theorem, stated exactly
  in the lower third and proved in the fact sheet; all twelve orders of
  three rings lose a direction somewhere (`G-X6.ring_orders`).
- **Dev:** the same loss appears in an arm's wrist in chapter 11. A unit
  quaternion and its negative are the same orientation.

## 04_halfway (~68 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Angles hide a second trap. Take two headings for a robot. A hundred and seventy degrees to the left. And a hundred and seventy to the right, written minus a hundred and seventy. | A circle seen from above with a small robot at its centre; "ahead" at 0°. Two blue heading arrows, at 170° and −170°, both pointing nearly straight back, 20° apart (`G-H.a`, `G-H.b`) | circle and arrows | |
| 04_2 | ⟨predict⟩ What heading is halfway between them? | Quiz card: "halfway between 170° and −170°?" Choices 0°, 90°, 180°. Ring from 5 (`G-Q.04_2.*`) | quiz card; countdown | listen 5 s |
| 04_3 | Average the numbers and you get zero. Straight ahead. But both arrows point almost straight back. The honest halfway is a hundred and eighty, the exact opposite. | The sum (170 + −170) ÷ 2 = 0 writes out; a dashed red arrow appears at 0°. Then a green tick at 180°, between the two blue arrows. Under the circle a ruler from −180 to 180 bends up and wraps onto it, its two ends meeting at the back in a dashed red seam; the small robot, told to go from one heading to the other by the numbers, spins the long way round (`G-H.mean`, `G-H.halfway`, `G-H.off_by`) | TeX equation; highlight; morph that keeps identity | |
| 04_4 | The trouble is the ruler's seam. So peel the ruler off, and lay it flat against the circle, with its zero on the first heading. Now measure to the second. Twenty degrees. Take half. Ten degrees on from the first is a hundred and eighty. | The same ruler peels off the circle, turns orange and lies flat against it, touching at the 170° point with its zero there; its seam is now on the far side. The step to the other heading reads 20°; half of it, 10°, is wrapped back onto the circle, landing on the tick. "For programmers" tag: "wrap the difference into −180…180: −340 becomes +20" (`G-H.gap`) | larger scale of the same picture; number line; morph; "for programmers" tag | |
| 04_5 | That is the whole trick, and the rest of this video is that trick again and again. The space is curved. Do the arithmetic on the flat piece that touches it. Then wrap the answer back on. | The circle and its orange line, centred, then shrinking to a corner: **the badge**. Three words build: curved, flat, wrap back. Lower third: "the flat piece: the tangent space" | card; lower third; **the badge** | hold |

- **Misconception defused:** "the average of two angles is the angle in
  between". Only when the two numbers are on the same side of the seam.
- **Ruth:** "halfway" means the midpoint of the shorter arc; it is unique
  here because the headings are 20° apart, not 180°. `G-H.off_by` is the
  angle between the averaged heading and the honest one. On a circle the
  flat piece's arithmetic is exact for any step under a half turn; for
  turns in space it is exact along one axis and close for small steps,
  which is why 13 repeats it.
- **Dev:** the tag is the one line of code; the bug is the robot that spins
  the long way round.

## 05_spin (~84 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | On the circle, a step along the flat piece was so many degrees. Read it as a speed, held for one second. In space, the flat piece is made of such spins, each one arrow, along its axis, as long as its speed. | The badge's circle: the 20° step relabelled "20° a second, for one second". It morphs to the book, still, with an orange arrow through it and a curl for the sense of the turning; lengthening the arrow speeds the turning | morph that keeps identity; 3D book; arrow on a tracker | |
| 05_2 | Arrows are easy to stretch, to average and to add. But adding two spins is not one turn then another. It is both at once. | The tilt's arrow and the roll's arrow add tip to tail into one arrow between them. Held for a second, the book lands at a third orientation: 127° about the in-between axis, beside the two books of chapter 01, matching neither. Lower third: "near doing nothing, arrows and orientations match one to one; far away they fold: a half turn has two arrows" (`G-B.both_at_once.*`) | arrows adding; 3D turn; lower third | |
| 05_3 | Hold one spin steady for one second, and the book ends at some orientation. That step, from the flat piece back to the curved space, is the exponential. | The arrow stays fixed while the book turns about it; a one-second clock ring fills; the book stops. The badge in the corner draws its "wrap back" arrow as the label "the exponential" appears. Lower third: `e^{[\omega]t}`: a unit axis ω, turned through t | 3D turn; clock on a tracker; TeX; **the badge** | |
| 05_4 | ⟨predict⟩ Spin the book about the line leaning equally from its three edges, a third of a turn per second. After one second, where do its edges point? | The orange arrow along the line between the three edge arrows (a ghost of chapter 02's axis beside it, labelled "the other one"). Quiz card: "back where they began", "each where the next one was", "each reversed". Ring from 5 (`G-Q.05_4.*`) | quiz card; countdown | listen 5 s |
| 05_5 | Each edge lands where the next one was. That is where the book ended in the first quiz, with roll first. | The book spins for the second; each edge arrow lands on its neighbour's ghost. The book from 01_2 (roll, then tilt) appears beside it in the same orientation; a green "=" (`G-R.diag.edges`, `G-R.diag.is_roll_then_tilt`) | 3D turn; highlight | |
| 05_6 | The step has a formula, named after Rodrigues. Any point splits into a part along the axis, which stays, and a part that swings around. | One corner of the book: its part along the axis (teal, fixed) and its part around (blue, sweeping a circle), with `p' = \text{along} + \cos t\,\text{around} + \sin t\,(\omega \times \text{around})`. Then, small, under a "for programmers" tag: `e^{[\omega]t} = I + \sin t\,[\omega] + (1 - \cos t)\,[\omega]^2`, "ω: a unit axis; t: the angle in radians; [ω]p means ω × p". Lower third: "named for Olinde Rodrigues, who worked out in 1840 how turns combine" (`G-R.rod.*`, `G-L.rodrigues.year`) | TeX from the picture; "for programmers" tag; lower third | |
| 05_7 | A group of motions that is smooth like this is a Lie group, after the Norwegian mathematician Sophus Lie. Its flat piece is its Lie algebra. | The badge grows back to the centre: its circle labelled **Lie group** (blue), its orange line **Lie algebra**, the wrap-back arrow "the exponential". A name card: "Sophus Lie (say "Lee"), 1842–1899" | card; names; **the badge** | |

- **Misconception defused:** "a spin's three numbers are three angles", and
  "to combine two turns, add their arrows". A spin is one arrow, and adding
  two arrows turns about both at once: a third answer.
- **Ruth:** "the exponential" is the matrix exponential of the spin's skew
  matrix, held for one second; in the formula ω is a unit axis and t the
  angle in radians (Lynch and Park eq. 3.51). The arrows are a flat space,
  but the step back is not one to one: a half turn has two arrows, and the
  lower third of 05_2 says so. "Smooth" stands for "a differentiable
  manifold with smooth operations"; "group" was defined at 02_5.
- **Dev:** three numbers in, nine out; the logarithm goes back.

## 06_the_platform (~55 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Poses work the same way. Start on a floor, with a KUKA mobile platform. Its wheels carry slanted rollers, so it can drive forward, slide sideways and spin, all at once. Three numbers for its pose, three for its velocity. | Top view of the platform (lower third: KUKA KMR iiwa, Mecanum wheels): a rounded rectangle, four wheels hatched at 45°. Three orange arrows at its centre: forward and sideways straight, spin curled. Then two readouts: pose (x, y, heading) and velocity (forward, sideways, spin) | **the platform** (top view); lagged reveal; readouts | |
| 06_2 | ⟨predict⟩ Hold one velocity for six seconds. Forward at half a meter per second, spinning at thirty degrees per second. Where does the platform end up? | The platform at the origin of a floor grid in meters, its velocity shown. Quiz card with three small maps: "three meters ahead", "beside where it started", "back where it started". Ring from 6 (`G-Q.06_2.*`, `G-P.v`, `G-P.spin_deg`, `G-P.t`) | quiz card; countdown; grid | listen 6 s |
| 06_3 | Beside where it started. Just under two meters to its left, facing backward. It drove a half circle. | The platform drives its arc, leaving a teal trail, a live pose readout running; it stops at (0.00, 1.91) facing back. On screen: "a half circle 3 m long is 6 ÷ π = 1.91 m across" (`G-P.end`, `G-P.heading`) | live readout from the moving platform; traced path | |
| 06_4 | Add six seconds of velocity onto the pose, and you predict three meters straight ahead. That is the flat piece's answer. Wrap it onto the curved space, and the same three meters bend around a fixed point. | The straight 3.00 m line to (3.00, 0.00) in solid orange; the badge lights. The line bends, at constant length, into the arc about a gold centre dot, radius 0.95 m. Then one slider on the spin, from 0 to 60° a second: the path runs through the quiz's three choices, (3.00, 0), (0, 1.91) and (0, 0). "For programmers" tag: "radius = speed ÷ spin, the spin in radians per second: 0.5 ÷ 0.524 = 0.95 m. Wheel speeds: Lynch and Park eq. 13.10". Lower third: "on a floor, every steady motion is a turn about a point, or with no spin a straight slide. Our numbers, not KUKA's" (`G-P.naive`, `G-P.arc_length`, `G-P.radius`, `G-P.centre`, `G-P.spin0.end`, `G-P.spin60.end`, `G-X5.floor`, `G-P.wheels.*`) | morph at constant length; tracker; live number; "for programmers" tag; **the badge** | |

- **Misconception defused:** "velocity times time is where you end up".
  It is where you end up on the flat piece; the pose is that answer
  wrapped round a centre.
- **Ruth:** the velocity is the platform's own (forward and sideways are
  read in its frame), held constant; the pose reached is the exponential
  of the 3 by 3 velocity matrix times six seconds. "Every steady motion is
  a turn about a point or a slide" is tested on 45 velocities and is on
  screen, not spoken. The platform moves at half of KUKA's published top
  speed (3.6 km/h); KUKA publishes no turning rate, and the lower third
  says the numbers are the video's.
- **Dev:** how to compute the arc; the wheel formula's source (the wheels'
  senses for forward, sideways and spin are computed, `G-P.wheels.*`, and
  go in the description).

## 07_screws (~62 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | ⟨predict⟩ Now leave the floor. On a floor, a pose could change in three independent ways. Forward, sideways, and turning. In how many ways can a pose change in space? | Chapter 06's floor tilts into the fixed three-quarter view, the platform on it, its three orange arrows still drawn. The book rises off the floor. Quiz card: 3, 6, 9. Ring from 4 (`G-Q.07_1.*`) | affine morph of the floor group; quiz card; countdown | listen 4 s |
| 07_2 | Six. Three slides and three turns. So a velocity in space is two arrows, a slide and a spin. Six numbers, which roboticists call a twist. | The book in space with a straight orange arrow and a curled one; six numbers in two triples under the word **twist** (`G-S.numbers.pose`, `G-S.numbers.twist`) | 3D book; readout | |
| 07_3 | Hold a twist steady, and you get the motion of a screw. The body turns about an axis, and slides along that same axis. | On the tilted floor, the gold centre dot of 06 stands up into a vertical axis; the platform's arc lifts off the floor into a helix as a slide along the axis is added. **Four words, one card** appears: turn, spin, screw, twist. Lower third: "a twist's slide arrow is the velocity of the point at the origin; its sideways part moves the axis, as the platform's centre moved" | **the screw** (axis, helix); morph from 06; card; lower third | |
| 07_4 | A door is a screw with no slide. A drawer is a screw with no turn. A bottle cap does both. | Three small scenes, each with its gold axis: a door swinging, a drawer sliding, a cap rising as it turns | lagged reveal | |
| 07_5 | Here is the surprise. Pick up the book. Put it down anywhere, facing any way. One screw would have carried it there. Every time. | The book at rest, then a ghost of it a meter right, a meter away, half a meter up and a quarter turn round. A gold vertical axis appears off to the side, through the point (0, 1); the book replays the move as one corkscrew about it. Readout: turn 90°, slide 0.5 m. Then a wall of small books tossed to other poses, each gaining its own gold axis: "312 of 312" (`G-S.toss.*`, `G-X3.one_screw`) | 3D screw; ghost; live readout; lagged reveal of many | |
| 07_6 | Michel Chasles published that in eighteen thirty. Giulio Mozzi had described such an axis in seventeen sixty-three. | Lower third: "Mozzi 1763 (for motion at an instant) · Chasles 1830: every displacement of a rigid body is a screw displacement". The axis stays | lower third | |

- **Misconception defused:** "a motion in space is a turn plus a slide in
  some direction, and those are separate things". The slide can always be
  taken along the turn's own axis, once the axis is put in the right place.
- **Ruth:** six is the number of independent ways a pose can change; that
  three numbers cannot name every orientation was 03_6. Chasles's theorem
  with its edge: a pure slide is a screw with no turn (the axis is then
  only a direction), tested on 312 poses with turns from none to 150°. The
  1830 date is from Rodrigues's own footnote of 1840 and Ceccarelli's
  reference list; Mozzi's 1763 book is on the motion at an instant, and
  the words say "such an axis", not "the theorem". A twist's slide arrow is
  not the slide along the axis; the lower third of 07_3 says what it is.
- **Dev:** the four-word card fixes the vocabulary used from here on.

## 08_seven_screws (~73 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Back to the arm. Standing straight up, it is one point three meters tall. Each of its seven joints is a hinge, like the door. All it can do is turn about its own axis. | The arm, bent, with seven gold collars, one at each joint; it straightens to zero, standing up, with a height mark 1.306 m, and the seven axis lines appear, numbered 1 to 7, alternately vertical and level. "For programmers" panel: the model as data: the seven axes' heights and directions at zero, and the gripper's pose M at zero (`G-A.height`, `G-A.axis.*`, `G-A.axis_height.*`, `G-A.home.*`) | **the arm**; lagged reveal; labels; code panel with the "for programmers" tag | |
| 08_2 | A turn about an axis is a screw with no slide. So the arm is seven screws, stacked. | Each axis gains the curled arrow of chapter 07's screw, in gold, one after another. Tag: "turning about a line through c: x goes to c + R(x − c)" | lagged reveal; highlight; **the screw** | |
| 08_3 | ⟨predict⟩ Suppose only joint four turns, the elbow. Which parts of the arm move? | The elbow's axis glows gold. Quiz card: "everything", "only what is past the elbow", "only the forearm". Ring from 4 (`G-Q.08_3.*`) | quiz card; countdown | listen 4 s |
| 08_4 | Everything past the elbow, as one rigid piece. Nothing before it. Each joint applies its screw to everything further out. | The elbow turns a quarter turn: forearm, wrist and gripper swing together; the lower arm stays grey and still (`G-K.elbow.moves`) | moving arm; highlight | |
| 08_5 | That gives a recipe for the arithmetic. Start with the gripper's pose when every joint is at zero. Apply the last joint's screw, then the one before, down to the first. That way, each axis is still at its zero place when its turn comes. | The arm at home with its gripper pose marked M. Screws light from the wrist down to the base, one at a time; each time, the part of the arm above that joint swings, about an axis that has not yet moved. Tag: "the order of the sums, not of the motion" | moving arm; lagged reveal | |
| 08_6 | Each screw, held for its joint angle, is an exponential. So the gripper's pose is a product of seven exponentials, the line from the start. Roger Brockett presented it in nineteen eighty-three. | The line builds right to left as the screws lit: `M`, then `e^{[S_7]q_7}`, …, then `e^{[S_1]q_1}`, giving `T(q) = e^{[S_1]q_1} e^{[S_2]q_2} \cdots e^{[S_7]q_7} M`. Plain labels under its parts: "the gripper's pose", "joint 1's screw, held for its angle", "…", "the pose at zero". Lower third: "the product of exponentials · Brockett, 1983" (`G-L.brockett.year`) | TeX built term by term, selected by span; labels | hold |

- **Misconception defused:** "you must track every link's frame to find the
  gripper", and "the recipe's order is the order the arm moves in". Seven
  axes at zero and one home pose are enough, and the order is the order of
  the arithmetic.
- **Ruth:** the formula is the space form (Lynch and Park eq. 4.14): the
  screws `S_i` are the joint axes with the arm at zero, in the room's
  frame. Brockett spoke at MTNS in Beer Sheva in 1983; the proceedings are
  dated 1984 and their text was not read.
- **Dev:** the panel and the tag make each exponential computable: an axis
  direction, a point on it, an angle.

## 09_by_hand (~61 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Try it by hand, with the real arm's lengths. | The arm at home with four lengths marked in centimeters: 36 to the shoulder, 42 upper arm, 40 forearm, 12.6 from wrist to the gripper's tip. The tip readout is headed "forward, sideways, up" (`G-A.shoulder_height`, `G-A.upper_arm`, `G-A.forearm`, `G-A.wrist_to_tip`) | labels; braces | |
| 09_2 | Bend two joints a quarter turn each, the shoulder and the elbow. The recipe says, elbow first. With the arm still straight up, the elbow's screw swings the forearm over until it lies level. | `e^{[S_2]\,90°}\, e^{[S_4]\,90°}\, M` on screen; the right factor glows. The forearm swings about the elbow's gold axis; the tip readout goes from (0, 0, 130.6) to (−52.6, 0, 78.0) cm (`G-K.ell.after_elbow`) | moving arm; live readout; TeX highlight by span | |
| 09_3 | Then the shoulder's screw swings everything, about the shoulder's axis, just where it stood at zero. The upper arm comes down level, and the forearm points straight up. | The left factor glows; the whole arm above the shoulder swings a quarter turn about the shoulder's axis. The sum writes out, 42 forward and 36 + 40 + 12.6 up, and a moment later the readout lands on (42.0, 0, 88.6) cm (`G-K.ell.p`) | moving arm; live readout; sum written out | |
| 09_4 | ⟨predict⟩ Now joint one, at the base, turns a quarter turn about the vertical. Where does the gripper go? | Joint one's vertical axis glows. Quiz card: "higher", "round to the side, same height", "nowhere". Ring from 4 (`G-Q.09_4.*`) | quiz card; countdown | listen 4 s |
| 09_5 | Round to the side, at the same height. It was forty-two centimeters in front of the base. Now it is forty-two centimeters to the side. | The arm swings a quarter circle about the vertical; the tip readout runs from (42.0, 0, 88.6) to (0, 42.0, 88.6) cm; a third factor joins the line on the left (`G-K.quarter.p`) | moving arm; live readout | |
| 09_6 | Notice what the recipe never asked. Where the elbow's axis is now. Each screw is written down once, with the arm at zero. | The elbow's axis where it is now (grey) against where the formula used it (gold, at home, dashed). A crossed-out alternative beside it: "shoulder first: then you must find where the elbow's axis went" | highlight | |

- **Misconception defused:** "the first joint acts first". The joint
  nearest the gripper acts on the home pose first; the base's screw acts
  last and carries everything.
- **Ruth:** the worked pose is joints 2 and 4 at 90°, the others at zero;
  its tip is pinned by hand (42 forward; 88.6 up) and by the oracle's walk
  along the chain. Joint 4's axis points along minus y at zero, which is
  why the forearm first swings backward. The "gripper's tip" is the
  description's flange: no gripper is in the model, and the drawn fingers
  end at that point.
- **Dev:** the product is taken in the order of the joints; the closing
  beat is the line to repeat to a colleague.

## 10_velocities_add (~66 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Here is the question a robot really faces. This is the pose I want. Which joint angles get me there? The answer starts with speeds. | The arm in its L shape; the pink goal pose lights at the book, then dims and stays through chapters 10 to 12. Small curled arrows at two joints start to turn | the arm; goal marker | |
| 10_2 | For speeds, each axis counts where it is right now. A joint turning alone swings the gripper around its axis. That is a twist. And twists live on the flat piece, so they add. | Shoulder alone: an orange twist arrow at the gripper. Base joint alone: another. Both turning: the two arrows add tip to tail into one. The badge lights; the four-word card returns small | arrows adding; moving arm; **the badge** | |
| 10_3 | Seven joints, seven twists, six numbers each. Stand them side by side. A grid, six rows by seven columns, called the Jacobian. Joint speeds go in. The gripper's twist comes out. | Seven orange columns rise beside the arm, one per joint, each tied to its joint by a thin line; they close up into a 6 by 7 grid. `V = J(q)\,\dot q`. "For programmers" tag: "a twist's slide is the velocity of the point at the room's origin; the tip's velocity is slide + spin × tip position". Lower third: "the Jacobian: the map from joint speeds to the gripper's twist, at the pose the arm is in" (`G-J.shape`, `G-J.ell.col2`) | **matrix** from columns; TeX; "for programmers" tag; lower third | |
| 10_4 | ⟨predict⟩ The arm is in an L shape. Which joint moves the gripper's tip fastest, at the same turning speed? Joint one, two or seven? | Joints 1, 2 and 7 glow in turn, each named on the arm (base, shoulder, wrist). Quiz card with the three choices. Ring from 5 (`G-Q.10_4.*`) | quiz card; countdown | listen 5 s |
| 10_5 | Joint two. What counts is how far the tip is from each axis. Sixty-seven centimeters from the shoulder's. Forty-two from the base's. And the tip sits right on joint seven's axis, so that joint only spins it. | Three gold lever lines from each axis to the tip with their lengths: 67.3, 42.0 and 0.0 cm; the right triangle with legs 42 and 52.6 under the first; three velocity arrows at the tip in proportion (`G-J.ell.lever.*`) | braces; arrows in proportion | |

- **Misconception defused:** "a joint nearer the gripper moves it more".
  The lever is the distance from the joint's axis, wherever the joint is.
- **Ruth:** the Jacobian is the space Jacobian (Lynch and Park eq. 5.11):
  column i is joint i's screw carried to the present pose; it is a linear
  map between two flat pieces (joint speeds and twists), which the lower
  third says. The "speed of the tip" is the linear velocity of the tip
  point, lever times joint speed; the tag says how to read it off a twist.
- **Dev:** six rows (three of spin, three of slide), seven columns; the tag
  prevents reading the slide rows as the tip's velocity.

## 11_lost_directions (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | Six rows allow six independent directions for the gripper. Usually, seven joints cover all six. | The arm bent, with a readout "directions: 6" and six small orange arrows at the gripper (three straight, three curled) (`G-J.rank.swing`) | readout; arrows | |
| 11_2 | ⟨predict⟩ Here the arm's elbow is straight. Can the gripper still move in all six directions? | The elbow straightens: upper arm and forearm in one line, shoulder and wrist still bent. The readout is hidden. Quiz card: "all six", "one is lost". Ring from 4 (`G-Q.11_2.*`) | quiz card; countdown | listen 4 s |
| 11_3 | No. Along the arm, every joint's push on the wrist is zero. So the gripper can't slide further out, and it can't even begin to slide straight back. Five of the six directions are left. | The readout returns at 5. Each joint wiggles in turn and its push on the wrist is drawn: none has any part along the arm; one straight arrow along the arm goes dashed red. Lower third: "at first, the gripper can only swing sideways" (`G-J.rank.elbow_straight`, `G-J.elbow_straight.push_along_arm`) | readout from a nominal pose; highlight; lower third | |
| 11_4 | A pose like this is called a singularity. Close to one, a motion along the lost direction costs huge joint speeds. | The word **singularity**. The elbow bends a little; a speed gauge beside it. Lower third: "not only at full stretch: straight up with the elbow bent, this arm also has five; straight up, three" (`G-J.rank.elbow_only`, `G-J.elbow_only.reach`, `G-J.rank.home`) | card; gauge; lower third | |
| 11_5 | Pull the gripper straight back at ten centimeters per second. With the elbow bent thirty degrees, it must turn at fifty-four degrees per second. Bent ten degrees, a hundred and sixty. Its limit is seventy-five. | The elbow's speed against its bend as a curve on an axis cut off at 200°/s, with a dashed red limit line at 75°/s; the crossing marked at a bend of 21.5°. Above the cut the two last values as numbers: 3°: 534°/s; 1°: 1,602°/s, "four and a half turns a second". Tag: "from the geometry alone" (`G-J.blowup.*`) | curve on a clipped axis; limit line; highlight | |
| 11_6 | You have met this before. The wrist is three joints whose axes meet at one point, a gimbal. Straighten it, and two axes line up, like the rings. A six-joint arm would lose a direction here. The seventh joint covers for it. | Close on the wrist (the same projection, larger): its three gold axes; the middle joint straightens and joints 5 and 7 fall into one line; the three axes morph into chapter 03's three rings. "directions: 6" stays lit beside the arm (`G-J.rank.ell`) | close-up by scale; morph of gold axes into the gimbal | |

- **Misconception defused:** "a singularity is where the arm cannot reach".
  Singular poses are also found well inside the reach (the lower third of
  11_4 names one), and what collapses is the set of directions the
  gripper can move in.
- **Ruth:** "directions" is the rank of the Jacobian: 6 at the bent pose, 5
  with the elbow straight (the one lost direction is a slide along the
  arm: no joint moves the wrist along it, `push_along_arm` = 0, against
  0.29 m/s at the bent pose), 5 standing straight up with only the elbow
  bent (0.58 m from shoulder to wrist, inside the 0.82 m reach), 4
  stretched straight out with the wrist straight too, 3 standing straight
  up. Six rows allow at most six. The elbow's speed comes from the triangle
  shoulder, elbow, wrist and is the same for any way of sharing the motion
  among the other joints. With joint 6 at zero two wrist axes are in line;
  in the L shape the arm keeps all six directions, though not in every
  pose with a straight wrist.
- **Dev:** the limit is KUKA's published speed for axis 4 at rated payload;
  the crossing at 21.5° is computed.

## 12_the_spare_joint (~48 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Seven joints. Six directions. One joint is spare. What does it buy? | The tally 7 − 6 = 1 beside the arm (`G-J.spare`) | TeX | |
| 12_2 | ⟨predict⟩ Try it on yourself. Lay your palm flat on a desk and keep your shoulder still. Can your elbow still move? | An outline of a human arm, palm on a desk, shoulder pinned. Quiz card: "it can", "it can't". Ring from 6 (`G-Q.12_2.*`) | quiz card; countdown | listen 6 s |
| 12_3 | It can. It swings on an arc while your hand stays put. Counted the way a roboticist counts, your arm has seven joints too. | The outline's elbow swings on an arc; the count 3 + 1 + 3 = 7 at shoulder, elbow, wrist. The outline then morphs into the robot arm in the same pose | traced arc; labels; morph that keeps identity | |
| 12_4 | The robot does the same. Its gripper holds one pose, while its elbow swings on a circle twenty-nine centimeters in radius. | The arm in the L shape with its wrist bent; the elbow travels its gold circle about the line from shoulder to wrist, as far as the joints allow; a readout "gripper moved: 0.0 cm" stays at zero; the radius 29.0 cm (`G-J.elbow.radius`, `G-J.swing.null_twist`) | the arm moved along computed joint angles; live readout; traced path | |
| 12_5 | That is what the spare joint buys. The elbow can get out of the way while the gripper keeps working. | An obstacle slides in and the elbow swings out of its way. The seven orange arrows at the gripper cancel to nothing. Lower third: "the null space of the Jacobian: joint speeds whose twists add up to nothing" | arrows cancelling; lower third | |

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

## 13_steps (~66 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | Back to the question. Here is the pose I want. Which joint angles get me there? | The arm over the table, gripper pointing down; the pink goal brightens at the book, a quarter turn round and lower (`G-I.start.*`, `G-I.goal.*`) | the arm; goal marker | |
| 13_2 | Subtracting angles walks into both traps. These two orientations are one point four degrees apart. Yet two of their three ring angles differ by ninety. | Two books, almost identical, each in chapter 03's rings. Under them their ring angles: (0°, 89°, 0°) and (90°, 89°, 90°), the differences 90°, 0°, 90° in dashed red; between the books a small orange arrow: 1.41° (`G-E.near.*`) | 3D books; highlight | |
| 13_3 | So do it on the flat piece. Compare the pose the arm has with the pose it wants. One screw joins them. Read as a twist, that screw is the error. | The badge moves to the centre and runs once: a gold screw axis appears between the gripper and the goal; laid on the orange line it reads as six numbers. Lower third: "the logarithm: the twist that would reach the goal in one second" | **the screw**; **the badge**; readout; lower third | |
| 13_4 | Then run the Jacobian backwards, from that twist to joint speeds, and move the joints that way. Look again, and repeat. | The badge's wrap-back arrow fires and the arm takes one step. A code panel under "for programmers": `error = log(inverse(T(q)) · T_goal)` "a twist, read in the gripper's frame"; `q = q + pinv(J_body(q)) · error` "the smallest joint step that makes it"; `repeat until error < 0.001 rad and 0.1 mm`. Lower third: "many joint speeds make that twist; the smallest is taken" | code panel with the "for programmers" tag; lower third | |
| 13_5 | ⟨predict⟩ The gripper starts half a meter from its goal, and a quarter turn off. How many steps until it is within a tenth of a millimeter? | Quiz card with three choices: 3, 30, 300. Ring from 5 (`G-Q.13_5.*`) | quiz card; countdown | listen 5 s |
| 13_6 | From this start, three. Half a meter off. Then nine centimeters. Then four millimeters. Then three hundredths of a millimeter. It is Newton's method, on a curved space. | The arm takes its three steps; before each hop the remaining error is redrawn as a smaller gold screw. A table fills, rows "start, after 1, after 2, after 3": 505.0 mm and 90.00°; 93.9 mm and 8.64°; 4.2 mm and 0.72°; 0.03 mm and 0.01°; "each step gains more than the one before". Tag: "from a poor start it can take many more steps, or fail; real solvers cap, damp, respect joint limits and retry. For this arm a closed formula also exists" (`G-I.ik.*`) | the arm on computed joint angles; **the screw** shrinking; table | |

- **Misconception defused:** "the computer just solves for the angles", and
  "the error is a difference of angles". It walks, each step is the
  flat-piece trick of chapter 04, and the error is one screw.
- **Ruth:** the method is Lynch and Park §6.2.2 with the error read in the
  gripper's own frame and the pseudoinverse of the body Jacobian (the
  smallest of the infinitely many joint steps); the tolerances are their
  Example 6.1's. "From this start, three": Newton's method needs a start
  that is close enough, and the tag says what happens otherwise. The two
  orientations of 13_2 sit one degree from gimbal lock.
- **Dev:** the loop as code with its frame named; the stopping rule; what
  real solvers add.

## 14_the_steady_path (~57 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | One more question. Between the start and the goal, what should the gripper do on the way? | The start pose and the goal pose, the arm faint behind them | the arm; markers | |
| 14_2 | ⟨predict⟩ Blend the two orientations, number by number. Half of one matrix plus half of the other. Is the result halfway there? | Two 3 by 3 grids and their average between them. Quiz card: "halfway", "not even an orientation". Ring from 4 (`G-Q.14_2.*`) | matrices; quiz card; countdown | listen 4 s |
| 14_3 | It is not even an orientation. A gripper turned by that blend would be squashed to half its volume. | The badge's circle: two arrows a quarter turn apart average to an arrow that stops short of the circle, 0.71 long. Then a small gripper drawn through the blended matrix at steps 0, ¼, ½, ¾, 1: it shrinks in the middle and recovers; readout: volume 0.50 at halfway. Lower third: "a blend of two different turns is never a turn". "For programmers" tag: "re-normalising repairs the shape, not the timing: a quarter of the way it has turned 18.4°, not 22.5°" (`G-I.blend.det`, `G-X4.blend`, `G-I.blend.quarter_turned`, `G-I.screw.quarter_turned`) | **the badge**; 3D shape through a matrix; live number; lower third | |
| 14_4 | The honest path is the screw. Do the halving on the flat piece. Take the twist that joins the two poses, and hold it for half a second. Half the turn and half the slide, and every pose on the way is a true pose. | Chapter 01's last frame, now in full: the gripper corkscrews from start to goal about the gold vertical axis, a quarter turn and 34.8 cm down, the thread drawn faintly past both ends. At halfway the readouts: turned 45°, volume 1.00 (`G-I.screw.*`) | **the screw**; live readouts; traced path; **the badge** | hold |
| 14_5 | The tip may go straight instead, because positions do add. Here the two paths differ by seven point six centimeters. | The straight line from start to goal in teal beside the screw's helix; the largest gap between them marked: 7.6 cm. Lower third: "'steady' here means one constant twist. Splitting the job, a straight line for the tip and the turn on its own, with the turning still on its own screw, is common in practice" (`G-I.bow`) | two paths; brace; lower third | |

- **Misconception defused:** "any mix of two valid things is valid".
  Orientations are not closed under averaging; the mix leaves the space.
- **Ruth:** the determinant of a blend of two turns that differ by an angle
  a, with weights 1 − s and s, is 1 − 2s(1 − s)(1 − cos a): one only when
  the turns are equal or a weight is zero (tested on 78 turns at s = ½,
  where it is cos²(a/2)). The screw path is Lynch and Park eq. 9.6; the
  split path is their eqs. 9.7 and 9.8. Which path is "straight" depends on
  what is held constant, so the chapter says "steady".
- **Dev:** why not just re-normalise; what motion planners do, and the size
  of the difference here.

## 15_everywhere (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | The same moves run far beyond this arm. Drones steer poses. Mapping robots average them. A physics engine steps each body by the exponential of a small twist, so it stays rigid. And a humanoid's whole body floats as one free pose, plus twenty-nine joint angles. | Two small icons (a quadrotor, a map with pose markers). Then two spinning books side by side under "add the spin to the nine numbers" and "the exponential": the left one swells; readouts end at volume 1.105, width 1.051 against 1.000. Tag: "our test: 1,000 steps of 0.01 s at one radian a second. FrankenSim steps bodies by the exponential (fs-time, se3_exp_step)". Then a stick figure of the Unitree G1 (lower third with its name) with a pose marker at its pelvis and dots at its joints; "for programmers": velocity 6 + 29 = 35 numbers, stored state 3 + 4 + 29 = 36. Lower third, names to search: "Lie group · SE(3) · screw theory · product of exponentials" (`G-C.*`) | **the humanoid** (projected 3D skeleton); two live readouts; tally; lower third | |
| 15_2 | ⟨predict⟩ Last one. Tilt the book one degree, then roll it one degree. Then the other order. Do the two results still differ? | The opening's two books and their gold lines on the table. Quiz card: "the same", "still different". Ring from 4 (`G-Q.15_2.*`) | quiz card; countdown | listen 4 s |
| 15_3 | They do, by less than two hundredths of a degree. And that leftover is itself a turn, almost exactly about the one axis neither turn used. | One sweep: both turns shrink together from 90° down to 1°; between the two books the leftover is drawn as a wedge about its own gold axis, its angle live: 120° at quarter turns, 1.74° at 10°, 0.0175° at 1°; the axis stands up from a diagonal toward the vertical (`G-B.leftover.*`, `G-B.between.*`) | one tracker; live number; 3D wedge | |
| 15_4 | For small turns, make both ten times smaller, and the leftover shrinks a hundredfold. On the flat piece, that leftover follows a rule of its own, called the Lie bracket. It measures how much order matters. | The three readings stay as a short table. The badge: the two spin arrows on the orange line, and a third arrow at right angles to both. Lower third: "leftover ≈ angle × angle (in radians) about roll's arrow × tilt's arrow: [x, y] = z" (`G-B.leftover.*`) | table from primitives; **the badge**; lower third | |
| 15_5 | So the arm does not add orientations. It multiplies motions, and does its adding on the flat piece. That is what it means to think in Lie groups. | The opening's two books above the arm reaching its goal; the seven-factor line under them with its plain labels. Then the receipts card: "every number in this video was computed by code" with the repository link | card; receipts | hold |

- **Misconception defused:** "order matters, full stop". It matters by an
  amount, the amount is a turn, and for small turns it is second order.
- **Ruth:** the leftover is the group commutator of the two turns; the Lie
  bracket is the rule on the flat piece (for spins, the cross product) that
  it follows to second order: its angle tends to the product of the angles
  in radians and its axis to the third axis (0.7° off at 1°, 7° at 10°),
  which is why the words say "almost exactly" and "for small turns". The
  G1's velocity has 35 numbers, while a pose stored as a position and a
  quaternion makes 36 numbers of state. The swelling is the determinant
  after 1,000 forward Euler steps of 0.01 s at 1 rad/s, (1 + 10⁻⁴)¹⁰⁰⁰;
  across the axis the body is 5.1% wider, along it unchanged.
- **Dev:** the engine named is the owner's; the tag cites the function. The
  36 against 35 is what a robotics library's `nq` and `nv` are.
