<!-- Claim walk (gate G3) of script.tsv after R1 and the de-slopify pass, 2026-10-13. The first part is this lane's dispositions; below the rule is the walker's answer, verbatim (scratch paths shortened). Fixes are applied in the storyboard; see the commit that lands this file. -->

# Claim walk — Video G

## Dispositions

171 claims (every spoken sentence and every on-screen number or statement):
133 OK, 17 edges that hold, 5 edges that fail as worded, 7 mismatches, 5
unsourced, 4 misleading. No spoken number was wrong. Every non-OK row is
**fixed** in `docs/storyboard.md` (and the fact sheet), in the walker's order:

| # | line | disposition |
|---|---|---|
| 1 | 11_6 | fixed: "A six-joint arm, with two joints at the shoulder, would lose a direction here. This arm has a third, and it covers." Without any one of joints 1 to 3 the L shape has rank 5; without joint 7, 6 (`G-J.rank.ell.without_3`, `.without_7`) |
| 2 | 10_5 | fixed: the leg is labelled 52.6 |
| 3 | 03_6 | fixed: the lower third says three numbers cannot give every orientation one name without a jump; nine give one name each, four give two |
| 4 | 05_4 | fixed: "the line pointing between its three edge arrows" |
| 5 | 08_6 | fixed: "Hold each joint's twist for its joint angle, and you get its screw, an exponential"; the label says twist |
| 6 | 15_4 | fixed: the lower third says roll's arrow × tilt's arrow is −z, "[y, x] = −z" |
| 7 | 02_3 | fixed: the same angle with the line of each edge |
| 8 | 15_1 | fixed: "Mapping robots estimate them", sourced to GTSAM |
| 9 | 12_3 | fixed: Lynch and Park §5.1.6, p. 190, quoted in the fact sheet |
| 10 | 13_6 | fixed: the closed-formula claim is cut; finding 8 says it was not checked |
| 11 | 15_1 | fixed: arXiv:1003.2005's abstract quoted |
| 12 | 03_4 | fixed: Fjeld's sentence ("Then it waited for a realignment") quoted beside GLOCKMON's coarse align |
| 13 | 05_2 | fixed: "Add these two spins" |
| 14 | 03_4 | fixed: Apollo's is "the guidance unit"; "platform" is KUKA's alone |
| 15 | 05_1, 05_3, 05_6 | fixed: "a distance along the flat piece", "That move …", "The exponential has a formula"; "step" is Newton's alone |
| 16 | 04_4 | fixed: "bring the difference into −180…180" |
| 17 | 08_1, 10_4 | fixed: 08_1 names shoulder, elbow and wrist where axes meet; 10_4 labels "at the wrist" |
| 18 | 09_3 | fixed: "swings everything above it" |
| 19 | 07_5 | fixed: the edge tests now count distinct poses (no turn once): 264 of 264 screws, 66 of 66 blends |
| 20 | 09_1 | fixed: "12.6 from wrist to flange, where the drawn gripper's tip sits" |
| 21 | 15_5 | fixed: "every number we worked out was computed by code and checked by a second program; quoted numbers are cited" |

The stale quotes the walker listed in the fact sheet are updated to the
words now spoken.

---

# Claim walk (gate G3): Video G, "Why Robot Arms Think in Lie Groups"

Independent walk of every spoken sentence (`script.tsv`) and every on-screen
number or statement (`docs/storyboard.md`, picture column), 2026-10-13.

Files walked (md5): `script.tsv` d361447f…c212 · `docs/storyboard.md`
7d159fea…4e9d · `docs/fact-sheet.md` c0e5a768…cf81 · `oracle.py`
f45adbb3…c4ef (repository HEAD da00000, working tree as found; nothing in the
repository was edited, created or deleted, and cargo was not run).

## How the checks were made

My own scripts, none of which imports or reads `oracle.py`
(`<scratch>/scratch_claims/`, run with
`uv run --quiet --no-project --with numpy --with scipy python3 -I <script>`):

| Tag | Script | Route (different from the oracle's) |
|---|---|---|
| B | `walk_book.py` | scipy `Rotation` for every turn, angle and axis; 20,000 random pairs; cube group grown here; ring orders by determinant |
| F | `walk_floor_screw.py` | the platform by RK4 integration of its body velocity (no exponential); screws by Chasles's construction, rebuilt as slide to the axis, turn, slide back |
| A | `walk_arm.py` | the arm from the xacro file itself (parsed, through scipy's fixed-axis Euler angles) and again as a product of exponentials typed from KUKA's drawing (360, 420, 400, 126 mm): the two agree to 1e-15 at 200 random poses; a geometric Jacobian (axis × lever) checked against finite differences; SVD ranks; `numpy.linalg.pinv`; scipy `logm` |
| C | `walk_coda.py` | the leftover by scipy `Rotation`; the drift by 1,000 explicit steps |
| W | `walk_words.py`, `walk_words2.py` | script.tsv against the storyboard's words column; first spoken use of each term; fact-sheet quotes against script.tsv |
| X | `walk_extras.py` | distinct poses among the 312; CIEDE2000 of the palette; sums |
| S | the lane's local source copies, `<lane scratch>/sources/` | each quote found by me in the text named (file and line), or read from the page image |

Also: the oracle was run (225 keys, 11 of 11 cross-checks) and its output is
identical, key by key, to the fact sheet's "Every fact" table (225 of 225).
`script.tsv` equals the storyboard's words column on all 83 spoken lines. No
banned word ("rotation", "tangent space", "manifold", "end effector", "null
space", "commutator", "geodesic") is spoken; no digit, colon, dash or
parenthesis is in the spoken words.

Verdicts: **OK** · **EDGE HOLDS** · **EDGE FAILS** · **MISMATCH** ·
**UNSOURCED** · **MISLEADING**. "S:" is spoken, "P:" is on screen. Nothing is
left unchecked.

## The walk

| # | line | claim (verbatim words or on-screen text) | rests on (fact key or source) | how I checked | verdict | fix if not OK |
|---|---|---|---|---|---|---|
| 1 | 01_1 | S: "Tilt a book a quarter turn about one line on the table. Then roll it a quarter turn about another. Now swap the order. Roll first, then tilt. Does the book end up the same way?" P: card "same or different?", ring from 5 | `G-B.turns.same` 0, `G-Q.01_1` | B: tilt = quarter turn about the left-to-right line lifts the far edge (0,1,0) to (0,0,1); roll about the line pointing away carries the cover to the right. Quiz: one right choice (different); 01_2 states it | OK | |
| 2 | 01_2 | S: "No. Tilt first, and the cover ends up facing you." | `G-B.tilt_then_roll.cover` 0,−1,0 | B: cover goes to (0,−1,0), toward the viewer | OK | |
| 3 | 01_2 | S: "Roll first, and it faces the wall on your right." | `G-B.roll_then_tilt.cover` 1,0,0 | B: cover goes to (1,0,0) | OK | |
| 4 | 01_2 | P: lower third "turn about the book's own edges instead and the two answers trade places: still different" | `G-B.*.R` | B: own-edge tilt, roll equals room roll, tilt, and the reverse | OK | |
| 5 | 01_3 | P: title, (say "Lee") | Lynch and Park Def. 3.1 fn 2 | S: MR.txt l.3563 "pronounced “Lee”" | OK | |
| 6 | 01_4 | S: "Sliding is different. Slide the book right, then away, or away, then right, and it lands on the same spot." | `G-B.slides.same` 1 | B: vector sums equal | OK | |
| 7 | 01_4 | S: "So positions add, and orientations don't." P: "positions add" ticked, "orientations don't" crossed | `G-B.turns.same`; L&P Prop. 3.5 | B: the two orders differ; edge: two turns about one axis (40° and 70°) do add in either order; S: MR.txt l.3578 "generally not commutative" | EDGE HOLDS (as the general rule; the same-axis exception is given at 02_5) | |
| 8 | 01_5 | S: "This is a KUKA robot arm with seven joints." P: KUKA LBR iiwa 14 R820 | `G-A.joints`, `G-L.kuka.axes` | S: data sheet text l.63 "Number of axes 7"; A: the xacro has 7 revolute joints | OK | |
| 9 | 01_5 | S: "Its job is to put its gripper at a pose, a position and an orientation together." | ledger: pose | W: first spoken use of "pose" is this line | OK | |
| 10 | 01_6 | S: "So its software must do arithmetic with things that refuse to add." | 01_4 | follows from row 7 | OK | |
| 11 | 01_6 | S: "The way out is about a hundred and fifty years old," | `G-L.lie.year` 1873; MacTutor | S: mactutor_Lie.html l.246 "winter of 1873-74"; 2026 − 1873 = 153 | OK | |
| 12 | 01_6 | S: "and by the end you'll read it in one line." P: `T(q) = e^{[S_1]q_1} … e^{[S_7]q_7} M` | L&P eq. 4.14 | A: that product equals the chain at 104 poses. Note: 08_6 dates the line itself to 1983; "it" here is the idea | OK | |
| 13 | 02_1 | S: "Look where tilt then roll left the book. It could have got there in a single turn, about a single axis." | `G-B.tilt_then_roll.angle`, `.axis` | B: 120.0000° about (0.5774, 0.5774, −0.5774) | OK | |
| 14 | 02_1 | P: "A gold axis appears through the corner where the three arrows meet" | geometry | The one turn's axis passes through the point where the two table lines cross and through no other point of the table; about a parallel axis elsewhere the book reaches the same facing but another place | EDGE HOLDS (only if the two gold lines of 01_1 cross at that corner; "under each" book suggests its middle) | |
| 15 | 02_2 | S: "Two quarter turns went in. How big is that single turn?" P: ¼, ⅓, ½ turn | `G-Q.02_2` 120 | B: 120°; one right choice; 02_3 states it | OK | |
| 16 | 02_3 | S: "A third of a turn. Three of them bring the book home." P: 120°, 240°, 360° | `G-B.tilt_then_roll.angle` | B: R³ = I | OK | |
| 17 | 02_3 | S: "Its axis leans equally from the book's three edges." | `G-B.tilt_then_roll.axis` | B: 54.74° to the line of each edge | OK (of the edges as lines) | |
| 18 | 02_3 | P: "The axis is shown making the same angle with each edge arrow" | `.axis` 0.5774,0.5774,−0.5774 | B: to the arrows +x, +y, +z (the ones 02_6 and 05_5 need) the angles are 54.7°, 54.7°, 125.3°; no sense of the axis gives three equal angles. The line that does sit between those arrows is 05_4's | EDGE FAILS (picture) | Mark the 54.7° against each edge's line (edges extended through the corner), not against the three arrows |
| 19 | 02_4 | S: "That always works. Any turn followed by any turn is one turn about one axis." | `G-X1.one_turn` 192/192 | B: 20,000 random products all orthogonal, determinant one, an axis left in place; edges: zero then zero, a turn then its undoing, half then half on one axis (all the zero turn), half turns on perpendicular axes (a half turn), equal axes (110°) | EDGE HOLDS (a zero turn counts as a turn; it has no single axis) | |
| 20 | 02_4 | S: "Euler showed it in seventeen seventy-five." P: "Euler, 1775: however a sphere is turned about its centre, some diameter ends where it began" | `G-L.euler.year`; E478 | S: e478tr.txt l.324 "Quomodocunque sphaera circa centrum suum conuertatur, semper assignari potest diameter…"; Euler Archive page: 1775 / 1776, Novi Commentarii 20, pp. 189-207. Presented 1775, printed 1776; two turns about a centre are one displacement, so the spoken form follows | OK | |
| 21 | 02_5 | S: "And any turn can be undone." | inverse | B: R Rᵀ = I | EDGE HOLDS | |
| 22 | 02_5 | S: "A collection where you can combine any two, and undo any one, is called a group." P: "precisely: and combining is associative" | ledger: group | definition; associativity is on the card; W: first use | OK | |
| 23 | 02_5 | S: "In this one, order usually matters." | `G-X2.order_free_pairs` 120/576 | B: cube group grown here: 24 turns, 120 of 576 ordered pairs swap (order matters in 79%); 0 of 20,000 random pairs swap | OK | |
| 24 | 02_5 | P: lower third "about one axis, order never matters" | `G-X2.same_axis_swap` | B: 40° and 70° about a skew axis swap | EDGE HOLDS | |
| 25 | 02_6 | S: "To compute, track where the book's three edges point. Three arrows, three numbers each. Nine numbers in a grid, a matrix. One turn after another is one grid times another." | `G-R.numbers.matrix` 9 | B: columns of the grid are where +x, +y, +z land; the product roll · tilt is tilt then roll | OK | |
| 26 | 02_6 | P: the grid of `G-B.tilt_then_roll.R`; tag `R = roll · tilt`, later turn on the left | `G-B.tilt_then_roll.R` | B: rows (0,1,0), (0,0,−1), (−1,0,0) | OK | |
| 27 | 03_1 | S: "Nine numbers is a lot for one orientation. Three should do." | `G-S.numbers.*` | an orientation has three independent ways to change (9 − 6) | OK | |
| 28 | 03_1 | S: "Here is a machine that tries, a gimbal. Three rings, one inside the next, each turning about its own axis." P: outer vertical, middle away, inner left to right | `G-E.rings.order` z,y,x | B: R = Rz Ry Rx used throughout | OK | |
| 29 | 03_2 | S: "Turn the middle ring a quarter turn. Now the outer ring's axis and the inner ring's axis line up. Do those two rings do different things any more?" P: card "different, or the same?" | `G-E.left.90`, `G-Q.03_2` | B: inner axis at middle 90° is (0,0,−1), on the outer's line; one right choice (the same); 03_3 states it | OK | |
| 30 | 03_3 | S: "No. They turn the book about the same axis." | `G-E.lock.same` 0.0000 | B: ring angles (30,90,0) and (0,90,−30) give one matrix (difference 0); at middle 0 the same test gives 0.5 | OK | |
| 31 | 03_3 | S: "Nothing is stuck, but three rings now give only two ways to turn. This is gimbal lock." P: "ways to turn" 3, then 2 | `G-E.left.90` | B: rank of the three ring axes is 2 at 90°, 3 at 0°, 70°, 85° | OK | |
| 32 | 03_4 | S: "The Apollo spacecraft had this problem. Its guidance platform hung in three gimbals." | `G-L.apollo.gimbals`; Hoag E-1344 | S: alsj_e-1344.html "The IMU … in Apollo is a three degree of freedom gimballed structure" | OK | |
| 33 | 03_4 | S: "Past seventy degrees on the middle one, a warning lamp lit." | `G-L.apollo.lamp_deg` | S: Comanche055 T4RUPT l.568 "ABS(MGA) GREATER THAN 70 DEGREES AND LESS THAN OR EQUAL TO 85 DEGREES -- GIMBAL LOCK LAMP TURNED ON." | OK | |
| 34 | 03_4 | S: "Past eighty-five, the computer gave up," | `G-L.apollo.lost_deg` | S: l.569 "GREATER THAN 85 DEGREES -- ISS PUT IN COARSE ALIGN AND NO ATT LAMP TURNED ON." | OK | |
| 35 | 03_4 | S: "and the platform had to be realigned." | nothing quoted in the fact sheet | S: the lane's own download alsj_gimbals.html (Fjeld): "froze the IMU at 85 degrees, flashing an unfriendly 'no att' light. Then it waited for a realignment." True; the sheet does not carry it | UNSOURCED (in the sheet) | Add that sentence to the fact sheet's Apollo sources; words stay |
| 36 | 03_4 | P: bar 34% at 70°, 9% at 85°; lower third "…gimbal lock lamp above 70°, reference dropped above 85°. The bar is this video's own measure" | `G-E.left.70` 0.3420, `.85` 0.0872 | B: volume of the three ring axes 0.3420 and 0.0872 | OK | |
| 37 | 03_5 | S: "On Apollo eleven, Michael Collins radioed a request. How about sending me a fourth gimbal for Christmas." P: quote card, "105 hours into the flight" | NASA air-to-ground transcript | S: a11transcript_tec.txt l.14251 "04 09 00 30 CMP (COLUMBIA) How about sending me a fourth gimbal for Christmas."; 4 × 24 + 9 = 105 | OK | |
| 38 | 03_5 | S: "A fourth ring would have given one to spare." | Hoag; ALSJ page | 4 − 3 = 1; S: alsj_gimbals.html, Armstrong: "We had a four-gimbal platform on Gemini." | OK | |
| 39 | 03_6 | S: "The fault is in the three numbers. Name every orientation with only three, and somewhere there is a bad spot." | theorem in the fact sheet; `G-X6.ring_orders` 12/12 | B: all 12 ring orders have a middle angle with no volume (zyx ±90°, zyz 0°, 180°). Edge tried: the video's own spin arrows name every orientation with three numbers and lose no direction inside a half turn, but must jump at a half turn (two arrows there). So it survives if a "bad spot" is a lost direction or a forced jump: a naming with neither would be a covering of the orientations by a piece of three-number space, and their only coverings are themselves and the 3-sphere, neither of which fits | EDGE HOLDS | |
| 40 | 03_6 | S: "So software stores nine numbers, or four, and the extra ones buy a naming with no bad spot." | L&P fn 4, 5 | S: MR.txt l.1799 "subject to singularities", l.1804 "double covering" | OK | |
| 41 | 03_6 | P: "rotation matrix: 9 numbers − 6 rules = 3"; "unit quaternion: 4 numbers − 1 rule = 3" | `G-S.numbers.*` | arithmetic; six rules (R Rᵀ = I), one rule (unit length) | OK | |
| 42 | 03_6 | P: lower third "precisely: no continuous one-to-one naming of all orientations by three numbers exists" | theorem (invariance of domain) | True. But "one-to-one" is a test the unit quaternion, ticked in the same frame, also fails (B: q and −q are one orientation), so it does not state precisely what the words claim | MISLEADING | "precisely: every naming of all orientations by three numbers must somewhere jump or lose a direction; four numbers need not (each orientation then has two names, q and −q)" |
| 43 | 04_1 | S: "Angles hide a second trap. Take two headings for a robot. A hundred and seventy degrees to the left. And a hundred and seventy to the right, written minus a hundred and seventy." P: 170°, −170°, 20° apart | `G-H.a`, `G-H.b` | B: the two arrows are 20.0° apart | OK | |
| 44 | 04_2 | S: "What heading is halfway between them?" P: 0°, 90°, 180° | `G-Q.04_2` 180 | B: 180° is the middle of the short arc; 0° is also the same distance from both (170° each, the long way round). One right choice if "between" is the short way | EDGE HOLDS | |
| 45 | 04_3 | S: "Average the numbers and you get zero. Straight ahead." P: (170 + −170) ÷ 2 = 0 | `G-H.mean` | B: 0.0 | OK | |
| 46 | 04_3 | S: "But both arrows point almost straight back." | `G-H.a`, `.b` | B: each is 10° from straight back | OK | |
| 47 | 04_3 | S: "The true halfway is a hundred and eighty, the exact opposite." | `G-H.halfway`, `.off_by` | B: by unit vectors 180.0; 180° from the average | OK | |
| 48 | 04_4 | S: "The trouble is the ruler's seam. So peel the ruler off, and lay it flat against the circle, with its zero on the first heading. Now measure to the second. Twenty degrees. Take half. Ten degrees on from the first is a hundred and eighty." | `G-H.gap` 20 | B: −340 brought into −180…180 is +20; 170 + 10 = 180 | OK | |
| 49 | 04_4 | P: tag "wrap the difference into −180…180: −340 becomes +20" | `G-H.gap` | B: the number is right. But here "wrap" names the measuring step (onto the flat piece), the opposite of the "wrap the answer back on" the next beat defines | MISMATCH (one word, two things, on screen) | "bring the difference into −180…180: −340 becomes +20" |
| 50 | 04_5 | S: "That is the whole trick, and the rest of this video uses it again and again. The space is curved. Do the arithmetic on the flat piece that touches it. Then wrap the answer back on." P: "the flat piece: the tangent space" | ledger: the flat piece | W: "flat piece" returns in 05_1, 05_3, 05_7, 06_4, 10_2, 13_3, 14_4, 15_4, 15_5; first use here | OK | |
| 51 | 05_1 | S: "On the circle, a step along the flat piece was so many degrees. Read it as a speed, held for one second. In space, the flat piece is made of such spins. Each is one arrow along its axis, as long as its speed." P: "20° a second, for one second" | ledger: spin | definition; W: first use of "spin" | OK | |
| 52 | 05_2 | S: "Arrows are easy to stretch, to average and to add." | a vector space | definition | OK | |
| 53 | 05_2 | S: "Add two spins, and you get both turns at once, a third answer." P: 127° about the in-between axis | `G-B.both_at_once.*` | B: 127.2792° about (0.7071, 0.7071, 0), equal to neither order. Edge: two spins about one axis give the same answer as either order, not a third | EDGE FAILS (as a general sentence; true of the two spins on screen) | "Add these two spins, and you get both turns at once, a third answer." |
| 54 | 05_2 | P: lower third "near doing nothing, arrows and orientations match one to one; far away they fold: a half turn has two arrows" | exponential of so(3) | B: a half turn about u equals a half turn about −u | OK | |
| 55 | 05_3 | S: "Hold one spin steady for one second, and the book ends at some orientation. That step, from the flat piece back to the curved space, is the exponential." P: `e^{[ω]t}`, a unit axis ω, turned through t | ledger: the exponential | B: scipy's matrix exponential of the spin matrix equals the turn, 2,000 random | OK | |
| 56 | 05_4 | S: "Spin the book about the line leaning equally from its three edges, a third of a turn per second. After one second, where do its edges point?" P: three choices; "the other one" beside chapter 02's axis | `G-R.diag.*`, `G-Q.05_4` | B: four lines lean equally from the three edges. 02_3 used these words for (1,1,−1); this beat means (1,1,1). About 02's line the edges land on −z, x, −y, and 05_5 would be false. Only the picture's small label tells them apart. Quiz itself: one right choice | MISLEADING | "Spin the book about the other such line, the one between its three edge arrows, a third of a turn per second." |
| 57 | 05_5 | S: "Each edge lands where the next one was." | `G-R.diag.edges` | B: x to y, y to z, z to x | OK | |
| 58 | 05_5 | S: "That is where the book ended in the first quiz, with roll first." | `G-R.diag.is_roll_then_tilt` 1 | B: the two matrices are equal | OK | |
| 59 | 05_6 | S: "The step has a formula, named after Rodrigues." | L&P eq. 3.51 | S: MR.txt l.4185 "also known as Rodrigues’ formula for rotations" | OK | |
| 60 | 05_6 | S: "Any point splits into a part along the axis, which stays, and a part that swings around." P: `p' = along + cos t around + sin t (ω × around)`; `e^{[ω]t} = I + sin t [ω] + (1 − cos t)[ω]²` | `G-R.rod.*` | B: both formulas against scipy on 2,000 random axes, angles and points (6e-14); the point (1,0,0): along (⅓,⅓,⅓), around (⅔,−⅓,−⅓), lands on (0,1,0). Edge: a point on the axis has nothing to swing | EDGE HOLDS | |
| 61 | 05_6 | P: lower third "named for Olinde Rodrigues, who worked out in 1840 how turns combine" | `G-L.rodrigues.year` | S: rodrigues1840.txt l.453 "composition des rotations d'un système solide, l'ordre de ces rotations…"; page footers "Tome V. — Novembre 1840" (page image) | OK | |
| 62 | 05_7 | S: "A group of motions that is smooth like this is a Lie group, after the Norwegian mathematician Sophus Lie." P: "Sophus Lie (say "Lee"), 1842–1899" | Solà et al. §II-A; MacTutor | S: mactutor_Lie.html "Norwegian mathematician", born 17 December 1842, died 18 February 1899 | OK | |
| 63 | 05_7 | S: "Its flat piece is its Lie algebra." | Solà et al. §II-C | definition; W: first use | OK | |
| 64 | 06_1 | S: "Poses work the same way. Start on a floor, with a KUKA mobile platform. Its wheels carry slanted rollers, so it can drive forward, slide sideways and spin, all at once." P: KUKA KMR iiwa, Mecanum wheels | KUKA KMR iiwa brochure; L&P ch. 13 | S: kuka_kmriiwa_en.txt l.43-45 "Mecanum … move omnidirectionally"; MR.txt l.23857 | OK | |
| 65 | 06_1 | S: "Three numbers for its pose, three for its velocity." | planar pose | (x, y, heading); (forward, sideways, spin) | OK | |
| 66 | 06_2 | S: "Hold one velocity for six seconds. Forward at half a meter per second, spinning at thirty degrees per second. Where does the platform end up?" P: three maps | `G-P.v`, `.spin_deg`, `.t`, `G-Q.06_2` | F: one right choice (beside); 06_3 states it | OK | |
| 67 | 06_3 | S: "Beside where it started. Just under two meters to its left, facing backward. It drove a half circle." P: (0.00, 1.91) facing back; "a half circle 3 m long is 6 ÷ π = 1.91 m across" | `G-P.end` 0.0000,1.9099; `.heading` 180 | F: RK4 ends at (0.0000, 1.9099), heading 180.0°, path 3.0000 m; 6 ÷ π = 1.9099 | OK | |
| 68 | 06_4 | S: "Add six seconds of velocity onto the pose, and you predict three meters straight ahead. That is the flat piece's answer." P: 3.00 m to (3.00, 0.00) | `G-P.naive` | 0.5 × 6 = 3.0 | OK | |
| 69 | 06_4 | S: "Wrap it onto the curved space, and the same three meters bend around a fixed point." P: radius 0.95 m | `G-P.arc_length`, `.radius`, `.centre` | F: path length 3.0000; every point of the path 0.9549 from (0, 0.9549) to 1e-13 | OK | |
| 70 | 06_4 | P: slider through (3.00, 0), (0, 1.91), (0, 0); tag "0.5 ÷ 0.524 = 0.95 m" | `G-P.spin0.end`, `.spin60.end`, `.radius` | F: spin 0 ends (3, 0), spin 60°/s ends (0, 0); 30° = 0.5236 rad; 0.5 ÷ 0.524 = 0.9542 and 3 ÷ π = 0.9549 both show as 0.95 | OK | |
| 71 | 06_4 | P: lower third "on a floor, every steady motion is a turn about a point, or with no spin a straight slide. Our numbers, not KUKA's" | `G-X5.floor` 45/45 | F: 306 of 306 by integration, including rest, pure slide, pure spin, a spin of 0.001 rad/s | EDGE HOLDS | |
| 72 | 06_4 | P: "Wheel speeds: Lynch and Park eq. 13.10" | L&P eq. 13.10 | S: MR_p516-519.txt l.181, the four-mecanum-wheel model | OK | |
| 73 | 07_1 | S: "Now leave the floor. On a floor, a pose could change in three independent ways. Forward, sideways, and turning. In how many ways can a pose change in space?" P: 3, 6, 9 | `G-Q.07_1` 6 | one right choice; 07_2 states it | OK | |
| 74 | 07_2 | S: "Six. Three slides and three turns." | `G-S.numbers.pose` | 3 + 3 | OK | |
| 75 | 07_2 | S: "So a velocity in space is two arrows, a slide and a spin. Six numbers, which roboticists call a twist." | `G-S.numbers.twist`; L&P | S: MR.txt l.3045 "which together we call a spatial velocity or twist"; W: first use | OK | |
| 76 | 07_3 | S: "Hold a twist steady, and you get the motion of a screw. The body turns about an axis, and slides along that same axis." | L&P Prop. 3.25 | F: a held twist at three times has one axis line, with turn and slide in proportion to time. Edge: a twist with no spin is a pure slide, which 07_4 covers | EDGE HOLDS | |
| 77 | 07_3 | P: the four-word card; lower third "a twist's slide arrow is the velocity of the point at the origin; its sideways part moves the axis…" | L&P eq. 3.70 | F: slide arrow (0.2, 0.1, −0.4) against a slide along the axis of −0.313: different things, as the lower third says | OK | |
| 78 | 07_4 | S: "A door is a screw with no slide. A drawer is a screw with no turn. A bottle cap does both." | definition of a screw | by the definition of 07_3 | OK | |
| 79 | 07_5 | S: "Now pick up the book. Put it down anywhere, facing any way. One screw would have carried it there, every time." | `G-X3.one_screw`; Chasles–Mozzi | F: 5,000 of 5,000 random poses rebuilt from one screw (2e-15); edges: not moved, pure slide, pure turn about an offset axis, half turn with slide, a turn of one arcsecond, 179.999° | EDGE HOLDS (same facing gives a screw with no turn, as 07_4 allows) | |
| 80 | 07_5 | P: ghost a meter right, a meter away, half a meter up, a quarter turn; axis through (0, 1); "turn 90°, slide 0.5 m" | `G-S.toss.*` | F: 90.0°, axis (0,0,1), point (0, 1, 0), slide 0.5 | OK | |
| 81 | 07_5 | P: "a wall of small books tossed to other poses… 312 of 312" | `G-X3.one_screw` 312/312 | X: 13 axes × 6 angles × 4 slides = 312 tests, but 264 distinct poses: the 13 axes at a zero turn repeat the same 4 poses, and 13 of the 312 books are not moved at all | MISLEADING (low) | Show "264 of 264", or sweep the zero turn once; or caption them "312 tests" |
| 82 | 07_6 | S: "Michel Chasles published that in eighteen thirty." P: "Chasles 1830: every displacement of a rigid body is a screw displacement" | `G-L.chasles.year`; Rodrigues 1840 p. 386 fn | S: page image read by me: "Ce beau théorème de géométrie appartient, je crois, à M. Chasles, qui l'a publié dans le Bulletin universel des Sciences, tome XIV, année 1830."; Crossref's copy of Ceccarelli's references "…Paris, 1830, pp. 321–326". Second-hand: Chasles's note itself is unopened | OK | |
| 83 | 07_6 | S: "Giulio Mozzi had described such an axis in seventeen sixty-three." P: "Mozzi 1763 (for motion at an instant)" | `G-L.mozzi.year`; Ceccarelli 2000 (title); Minguzzi | S: minguzzi.txt l.175 and ref [2] "Screw axis defined by Giulio Mozzi in 1763 and early studies on helicoidal motion"; MR.txt l.5700. Second-hand: neither Mozzi's book nor Ceccarelli's text is opened, as the sheet says | OK | |
| 84 | 08_1 | S: "Back to the arm. Standing straight up, it is one point three meters tall." P: height mark 1.3 m | `G-A.height` 1.3060 | A: 1.306 from the xacro; 36 + 42 + 40 + 12.6 = 130.6 (to the flange) | OK | |
| 85 | 08_1 | S: "Each of its seven joints is a hinge, like the door. All it can do is turn about its own axis." | xacro | A: joints 1 to 7 are revolute, each about one axis | EDGE HOLDS | |
| 86 | 08_1 | P: seven axes "alternately vertical and level"; panel: the axes' heights and directions at zero, and M | `G-A.axis.*`, `.axis_height.*`, `.home.*` | A: z, y, z, −y, z, y, z at 0.1575, 0.36, 0.5645, 0.78, 0.9645, 1.18, 1.261; M = no turn at (0, 0, 1.306) | OK | |
| 87 | 08_2 | S: "A turn about an axis is a screw with no slide. So the arm is seven screws, stacked." P: tag "turning about a line through c: x goes to c + R(x − c)" | 07_3, 07_4 | by definition; the tag's formula is the one my recipe uses (row 91) | OK | |
| 88 | 08_3 | S: "Suppose only joint four turns, the elbow. Which parts of the arm move?" P: "everything", "only what is past the elbow", "only the forearm" | `G-Q.08_3` | A: joint 4 is at the elbow (0.78 m); one right choice; 08_4 states it | OK | |
| 89 | 08_4 | S: "Everything past the elbow, as one rigid piece. Nothing before it." | `G-K.elbow.moves` 5,6,7,tip | A: joints 5, 6, 7 and the tip move and keep their mutual distances; joints 1 to 4 stay | OK | |
| 90 | 08_4 | S: "Each joint applies its own screw to everything further out." | L&P §4.1 | A: for each of the seven joints at random poses, nothing inward moves and everything outward turns rigidly about that joint's axis; the first joint carries the whole arm, the last only the tip. S: MR.txt l.6961 | EDGE HOLDS | |
| 91 | 08_5 | S: "That gives a recipe for the arithmetic. Start with the gripper's pose when every joint is at zero. Apply the screw of the last joint, then the one before, down to the first." P: tag "the order of the sums, not of the motion" | L&P eq. 4.14 | A: the recipe equals the chain at 104 poses (random, all zero, every joint at its limit, only joint 7, only joint 1); the opposite order with the same axes does not | OK | |
| 92 | 08_5 | S: "That way, each axis is still where it stood at zero when its turn comes." | space form | A: every axis tracked through the recipe: at its turn each is at its zero place, 104 of 104, first and last joint included | EDGE HOLDS | |
| 93 | 08_6 | S: "Each screw, held for its joint angle, is an exponential." P: label "joint 1's screw, held for its angle" | L&P eq. 4.14 | The mathematics is right (row 91). The word is not: by the four-word card a screw is "how far", and what is held is a spin or a twist (05_3, 07_3, 14_4) | MISMATCH (one word, two things) | "Hold a joint's twist for its joint angle, and you get its screw. That is an exponential." Label: "joint 1's twist, held for its angle" |
| 94 | 08_6 | S: "So the gripper's pose is a product of seven exponentials, the line from the start." P: the line built right to left | eq. 4.14 | A: seven exponentials, then M, which the line shows | OK | |
| 95 | 08_6 | S: "Roger Brockett presented it in nineteen eighty-three." P: "the product of exponentials · Brockett, 1983" | `G-L.brockett.year`; L&P | S: MR.txt l.7849 "first presented by Brockett", l.358 "In 1983 I was invited to speak…"; Springer record: MTNS-83, Beer Sheva, June 20–24, 1983 (proceedings 1984, text unread) | OK | |
| 96 | 09_1 | S: "Try it by hand, with the real arm's lengths." P: 36, 42, 40, 12.6 cm | `G-A.shoulder_height`, `.upper_arm`, `.forearm`, `.wrist_to_tip` | A from the xacro: 0.36, 0.42, 0.40, 0.126 | OK | |
| 97 | 09_1 | P: label "12.6 from wrist to the gripper's tip" | storyboard note: the point is the flange | KUKA's 126 mm runs from the wrist to the mounting flange; a real gripper's tip is farther. Shown under "the real arm's lengths" | MISLEADING (low) | "12.6 from wrist to the flange, where the gripper bolts on" (or say so in the description and on the label) |
| 98 | 09_2 | S: "Bend two joints a quarter turn each, the shoulder and the elbow. The recipe says, elbow first. With the arm still straight up, the screw at the elbow swings the forearm over until it lies level." P: (0, 0, 130.6) to (−52.6, 0, 78.0) cm | `G-K.ell.after_elbow` | A: tip (−52.6, 0, 78.0); forearm points along (−1, 0, 0), level | OK | |
| 99 | 09_3 | S: "Then the screw at the shoulder swings everything, about the shoulder's axis, where it stood at zero." | space form | A: what is above the shoulder swings; the 36 cm below it does not | EDGE FAILS (low: "everything") | "…swings everything above it, about the shoulder's axis…" |
| 100 | 09_3 | S: "The upper arm comes down level, and the forearm points straight up." P: "42 forward and 36 + 40 + 12.6 up"; (42.0, 0, 88.6) cm | `G-K.ell.p` 0.4200,0,0.8860 | A: upper arm along (1,0,0), forearm along (0,0,1); tip (42.0, 0, 88.6); 36 + 40 + 12.6 = 88.6 | OK | |
| 101 | 09_4 | S: "Now joint one, at the base, turns a quarter turn about the vertical. Where does the gripper go?" P: "higher", "round to the side, same height", "nowhere" | `G-Q.09_4` | A: one right choice; 09_5 states it | OK | |
| 102 | 09_5 | S: "Round to the side, at the same height. It was forty-two centimeters in front of the base. Now it is forty-two centimeters to the side." P: (42.0, 0, 88.6) to (0, 42.0, 88.6) cm | `G-K.quarter.p` | A: (0, 42.0, 88.6). Said 42, shown 42.0: the value is exactly 42 | OK | |
| 103 | 09_6 | S: "The recipe never asked where the elbow's axis is now. Each screw is written down once, with the arm at zero." | space form | A: rows 91, 92 | OK | |
| 104 | 10_1 | S: "A robot's real question runs the other way. This is the pose I want. Which joint angles get me there? The answer starts with speeds." | L&P ch. 6 | no number; the method of chapter 13 does start from the Jacobian | OK | |
| 105 | 10_2 | S: "For speeds, each axis counts where it is right now. A joint turning alone swings the gripper around its axis. That is a twist. And twists live on the flat piece, so they add." | L&P eqs. 5.10, 5.11 | A: a Jacobian built from the axes where they are now equals finite differences of the chain (1.5e-10); the map is linear | OK | |
| 106 | 10_3 | S: "Seven joints, seven twists, six numbers each. Stand them side by side. A grid, six rows by seven columns, called the Jacobian. Joint speeds go in. The gripper's twist comes out." P: `V = J(q) q̇`; lower third | `G-J.shape` 6x7; `G-J.ell.col2` | A: shape (6, 7); column 2 at the L shape (0, 1, 0, −0.36, 0, 0) | OK | |
| 107 | 10_3 | P: tag "a twist's slide is the velocity of the point at the room's origin; the tip's velocity is slide + spin × tip position" | L&P eq. 3.70 | A: the lever from that formula equals the tip's distance from the axis | OK | |
| 108 | 10_4 | S: "The arm is in an L shape. Which joint moves the gripper's tip fastest, at the same turning speed? Joint one, two or seven?" | `G-Q.10_4` 2 | A: levers 0.42, 0.6731, 0; one right choice; 10_5 states it | OK | |
| 109 | 10_5 | S: "Joint two. What counts is how far the tip is from each axis." | lever = distance | A: tip speed per unit joint speed is the tip's distance from that joint's axis, all seven | OK | |
| 110 | 10_5 | S: "Sixty-seven centimeters from the shoulder's. Forty-two from the base's." P: 67, 42 cm | `G-J.ell.lever.2` 0.6731, `.1` 0.4200 | A: 0.6731 and 0.4200 | OK | |
| 111 | 10_5 | S: "And the tip sits right on joint seven's axis, so that joint only spins it." P: 0 cm | `G-J.ell.lever.7` 0.0000 | A: 0.0000 | OK | |
| 112 | 10_5 | P: "the right triangle with legs 42 and 53" beside "67" | `G-J.ell.lever.2` | A: the legs are 42 and 52.6 (40 + 12.6). √(42² + 52.6²) = 67.3; √(42² + 53²) = 67.6, which a viewer rounds to 68 | MISMATCH (rounding) | Label the leg 52.6 (the viewer has already seen 40 + 12.6) |
| 113 | 11_1 | S: "Six rows allow six independent directions for the gripper. Usually, seven joints cover all six." P: "directions: 6" | `G-J.rank.swing` 6 | A: rank 6 at the pose shown and at 2,000 of 2,000 random poses; W: first use of "direction" | OK | |
| 114 | 11_2 | S: "Here the arm's elbow is straight. Can the gripper still move in all six directions?" P: "all six", "one is lost" | `G-Q.11_2`; `G-J.rank.elbow_straight` | A: rank 5 at (0, 60°, 0, 0, 0, 60°, 0); one right choice; 11_3 states it | OK | |
| 115 | 11_3 | S: "No. Along the arm, every joint's push on the wrist is zero." | `G-J.elbow_straight.push_along_arm` 0.0000 | A: all seven pushes are 0 at this pose, and at most 1e-16 over 500 random poses with the elbow straight (joints 1 to 3 pass through the shoulder, joint 4 pushes across the arm, joints 5 to 7 pass through the wrist); 0.2897 at a bent pose | EDGE HOLDS | |
| 116 | 11_3 | S: "So the gripper can't slide further out, and it can't even begin to slide straight back." P: lower third "at first, the gripper can only swing sideways" | `G-J.rank.elbow_straight` | A: a slide along the arm is out of reach (least-squares miss 0.95 of 1); both sideways slides are reached exactly; bending the elbow 1° draws the wrist back 0.03 mm, 5° draws it 0.78 mm (second order) | OK | |
| 117 | 11_3 | S: "Five of the six directions are left." P: readout 5 | `G-J.rank.elbow_straight` 5 | A: singular values 2.11, 1.82, 1.35, 0.50, 0.36, 0 | OK | |
| 118 | 11_4 | S: "A pose like this is called a singularity. Close to one, a motion along the lost direction costs huge joint speeds." | L&P §5.3; `G-J.blowup.*` | A: row 120 | OK | |
| 119 | 11_4 | P: lower third "not only at full stretch: straight up with the elbow bent, this arm also has five; straight up, three" | `G-J.rank.elbow_only` 5, `.home` 3 | A: 5 and 3 | OK | |
| 120 | 11_5 | S: "Pull the gripper straight back at ten centimeters per second. With the elbow bent thirty degrees, it must turn at fifty-four degrees per second." | `G-J.blowup.30` 54.03 | A: 54.03 from the pseudoinverse of my Jacobian and from the triangle. "Must": the joint speeds that move nothing have no elbow part (1e-15), and the rate is the same with the wrist at 30° and with joints 1, 3, 5, 7 turned | OK | |
| 121 | 11_5 | S: "Bent ten degrees, a hundred and sixty." | `G-J.blowup.10` 160.44 | A: 160.44 | OK | |
| 122 | 11_5 | S: "Its limit is seventy-five." P: limit line at 75°/s | `G-J.blowup.limit`; KUKA data sheet | S: data sheet text l.99 "A4 75 °/s" (speed with rated payload) | OK | |
| 123 | 11_5 | P: crossing at a bend of 21.5°; "3°: 534°/s; 1°: 1,602°/s, four and a half turns a second" | `G-J.blowup.bend_at_limit`, `.3`, `.1`, `.1.turns_per_s` | A: 21.49°; 534.17; 1602.34; 4.45 turns a second | OK | |
| 124 | 11_6 | S: "You have met this before. The wrist is three joints whose axes meet at one point, a gimbal." | xacro | A: axes 5, 6, 7 all pass through the wrist point (distances 0) | OK | |
| 125 | 11_6 | S: "Straighten it, and two axes line up, like the rings." P: joints 5 and 7 fall into one line | `G-J.rank.ell` | A: with joint 6 at zero, axes 5 and 7 are one line | OK | |
| 126 | 11_6 | S: "A six-joint arm would lose a direction here. The seventh joint covers for it." P: "directions: 6" stays lit | `G-J.rank.ell` 6 | A: at the L shape the rank is 6; with joint 1, 2, 3, 4, 5, 6, 7 removed it is 5, 5, 5, 5, 6, 5, 6. Take away joint seven (the joint the video calls "joint seven", shown lining up with joint 5) and nothing is lost; the arm that loses a direction is the usual six-joint one, without joint 3 (rank 5 at 300 of 300 poses with a straight wrist). Joint 7 is the duplicate here, not the cover | EDGE FAILS | "A six-joint arm, with only two joints at its shoulder, would lose a direction here. This arm's third shoulder joint covers for it." Picture: light joint 3 |
| 127 | 12_1 | S: "Seven joints for six directions leaves one joint spare. What does it buy?" P: 7 − 6 = 1 | `G-J.spare` 1 | A: 7 − rank = 1 at the pose of 12_4 | OK | |
| 128 | 12_2 | S: "Try it on yourself. Lay your palm flat on a desk and keep your shoulder still. Can your elbow still move?" P: "it can", "it can't" | `G-Q.12_2`; L&P Ex. 2.2 | S: MR.txt l.2244-2274; one right choice; 12_3 states it | OK | |
| 129 | 12_3 | S: "It can. It swings on an arc while your hand stays put." | L&P Ex. 2.2 | S: the exercise sets up exactly this; A: the robot's version, row 131 | OK | |
| 130 | 12_3 | S: "Counted the way a roboticist counts, your arm has seven joints too." P: 3 + 1 + 3 = 7 | L&P Exercises 2.2, 2.3 | S: the exercises ask for the count and give no answer; no opened source in the sheet states seven. The count is the standard model and true to my knowledge. The lane's KUKA brochure copy has "The kinematic system of the LBR iiwa is based on the human arm." | UNSOURCED | Add one opened source that states 3 + 1 + 3 (and the KUKA sentence); words stay |
| 131 | 12_4 | S: "The robot does the same. Its gripper holds one pose, while its elbow swings on a circle twenty-nine centimeters in radius." P: "gripper moved: 0.0 cm"; 29 cm | `G-J.elbow.radius` 0.2897; `G-J.swing.null_twist` 0.0000 | A: 0.2897 as the elbow's distance from the shoulder-to-wrist line and as 0.42 × 0.40 ÷ 0.58; motion followed both ways: tip fixed to 1e-15, radius constant, 352° of the circle before a joint limit | OK | |
| 132 | 12_5 | S: "That is what the spare joint buys. The elbow can get out of the way while the gripper keeps working." P: lower third "the null space of the Jacobian: joint speeds whose twists add up to nothing" | `G-J.swing.null_twist` | A: the null vector's twist is 2e-16 | OK | |
| 133 | 13_1 | S: "Back to the question. Here is the pose I want. Which joint angles get me there?" P: gripper pointing down; goal a quarter turn round and lower | `G-I.start.*`, `G-I.goal.*` | A: start tip (0.61, 0, 0.5977), tool axis (0, 0, −1); goal at 0.25 m height, a quarter turn about the vertical | OK | |
| 134 | 13_2 | S: "Subtracting angles walks into both traps. These two orientations are one point four degrees apart. Yet two of their three ring angles differ by ninety." P: (0°, 89°, 0°), (90°, 89°, 90°); 90°, 0°, 90°; 1.4° | `G-E.near.*` | B: 1.4142° apart; scipy reads the two back as (0, 89, 0) and (90, 89, 90) | OK | |
| 135 | 13_3 | S: "So do it on the flat piece. Compare the pose the arm has with the pose it wants. One screw joins them. Read as a twist, that screw is the error." P: lower third "the logarithm: the twist that would reach the goal in one second" | L&P §6.2.2 | A: the error used is the logarithm of (pose)⁻¹ · goal | OK | |
| 136 | 13_4 | S: "Then run the Jacobian backwards, from that twist to joint speeds, and move the joints that way. Look again, and repeat." P: the three code lines; "0.001 rad and 0.1 mm" | `G-I.ik.tolerances`; L&P Ex. 6.1 | A: I ran exactly this loop | OK | |
| 137 | 13_5 | S: "The gripper starts half a meter from its goal, and a quarter turn off. How many steps until it is within a tenth of a millimeter?" P: 3, 30, 300 | `G-I.ik.dist_mm`, `.angle_deg`, `G-Q.13_5` | A: 504.99 mm and 90.0° at the start; first under 0.1 mm after 3 steps; one right choice; 13_6 states it | OK | |
| 138 | 13_6 | S: "From here, three." | `G-I.ik.steps` 3 | A: 3 (scipy logm, numpy pinv, my body Jacobian); final joint angles (24.89, 38.57, 15.58, −104.86, −15.56, 38.65, −40.55)°, inside the limits | OK | |
| 139 | 13_6 | S: "Half a meter off. Then nine centimeters. Four millimeters. Three hundredths of a millimeter." P: 50 cm and 90°; 9 cm and 8.6°; 4 mm and 0.7°; 0.03 mm and 0.01° | `G-I.ik.dist_mm` 504.99,93.93,4.15,0.03; `.angle_deg` | A: 504.99, 93.93, 4.15, 0.026 mm; 90.00, 8.64, 0.72, 0.008°. Said and shown rounded alike | OK | |
| 140 | 13_6 | S: "It is Newton's method, on a curved space." P: "each step gains more than the one before" | L&P §6.2.2 | S: MR.txt l.10938 "Newton-Raphson"; A: gains ×5.4, ×22.6, ×157 | OK | |
| 141 | 13_6 | P: tag "from a poor start it can take many more steps, or fail; real solvers cap, damp, respect joint limits and retry." | fact sheet finding 7 | A: of 60 random starts, 3 reached the goal inside the joint limits within 60 steps (5 to 9 steps) | OK | |
| 142 | 13_6 | P: tag "For this arm a closed formula also exists" | fact sheet finding 8, which cites nothing | True to my knowledge of arms built shoulder, elbow, wrist; the sheet opens no source for it | UNSOURCED | Cite an opened paper on closed-form inverse kinematics for this kind of seven-joint arm, or cut the sentence |
| 143 | 14_1 | S: "One more question. Between the start and the goal, what should the gripper do on the way?" | none | no factual claim | OK | |
| 144 | 14_2 | S: "Blend the two orientations, number by number. Half of one matrix plus half of the other. Is the result halfway there?" P: "halfway", "not even an orientation" | `G-Q.14_2` | A: determinant 0.5; one right choice; 14_3 states it | OK | |
| 145 | 14_3 | S: "It is not even an orientation. A gripper turned by that blend would be squashed to half its volume." P: volume 0.50 at halfway; 0.71 long | `G-I.blend.det` 0.5000 | A: 0.5000; volumes at 0, ¼, ½, ¾, 1 are 1, 0.625, 0.5, 0.625, 1; mean of two unit arrows a quarter turn apart 0.7071 | OK | |
| 146 | 14_3 | P: lower third "a blend of two different turns is never a turn" | `G-X4.blend` 78/78 | A: 0 of 20,000 random blends (weights strictly between 0 and 1) reach determinant 1; 1 − 2s(1 − s)(1 − cos a) holds to 1e-15; a half turn apart, half and half: 0 | EDGE HOLDS | |
| 147 | 14_3 | P: tag "a quarter of the way it has turned 18.4°, not 22.5°" | `G-I.blend.quarter_turned`, `G-I.screw.quarter_turned` | A: 18.4349 (nearest turn by singular values) and 22.5000 | OK | |
| 148 | 14_4 | S: "The steady path is the screw. Do the halving on the flat piece. Take the twist that joins the two poses, and hold it for half a second. Half the turn and half the slide," P: "a quarter turn and 34.8 cm down"; "turned 45°, volume 1.00" | `G-I.screw.*` | A: 90.0° about the vertical through (0.355, 0.045), slide −0.3477; halfway 45.0°, determinant 1 | OK | |
| 149 | 14_4 | S: "and every pose on the way is a true pose." | L&P eq. 9.6 | A: 41 fractions of the way, determinant and squareness to 3e-16 | EDGE HOLDS | |
| 150 | 14_5 | S: "The tip may go straight instead, because positions do add. Here the two paths differ by seven point six centimeters." P: "the largest gap between them marked: 7.6 cm"; lower third | `G-I.bow` 0.0758 | A: 0.0758 both at equal fractions and as the helix's distance from the line (halfway; 0.2589 × (1 − cos 45°)). It is the largest gap, which the picture says | OK | |
| 151 | 15_1 | S: "The same moves run far beyond this arm. Drones steer poses." | Lee, Leok, McClamroch, CDC 2010: "(record only)" | S: only a title record for the paper cited. The lane's download of the same authors' arXiv:1003.2005 abstract does say "A nonlinear tracking controller is developed on the special Euclidean group SE(3)" | UNSOURCED (as cited) | Quote that abstract in the fact sheet; words stay |
| 152 | 15_1 | S: "Mapping robots average them." | nothing in the sheet | S: Solà et al. and the manif and GTSAM READMEs speak of estimation, smoothing and mapping; none is quoted for averaging poses | UNSOURCED | "Mapping robots estimate them." (matches the quotes held), or add an opened source on averaging poses |
| 153 | 15_1 | S: "A physics engine can step each body by the exponential of a small twist, so the body stays rigid." P: volume 1.105, width 1.051 against 1.000; tag "1,000 steps of 0.01 s at one radian a second. FrankenSim … se3_exp_step" | `G-C.drift.*`; FrankenSim | C: adding the spin gives determinant 1.10517, width 1.05127 across the axis, 1.000 along it; exponential steps keep determinant 1 (1e-13); S: frankensim/crates/fs-time/src/se3.rs l.80 `pub fn se3_exp_step` | OK | |
| 154 | 15_1 | S: "And a humanoid's whole body floats as one free pose, plus twenty-nine joint angles." P: Unitree G1; 6 + 29 = 35; 3 + 4 + 29 = 36 | `G-C.g1.*`; Pinocchio | S: cmaes_explainer/vendor/g1_29dof.urdf parsed by me: 29 revolute joints, root link pelvis; pinocchio.txt l.132-135 | OK | |
| 155 | 15_2 | S: "Last one. Tilt the book one degree, then roll it one degree. Then the other order. Do the two results still differ?" P: "the same", "still different" | `G-Q.15_2` | C: the two matrices differ by 3e-4; one right choice; 15_3 states it | OK | |
| 156 | 15_3 | S: "They do, by less than two hundredths of a degree." P: 0.0175° at 1° | `G-B.leftover.1.angle` 0.0175 | C: 0.017453° | OK | |
| 157 | 15_3 | S: "And that leftover is itself a turn, almost exactly about the one axis neither turn used." P: 120° at quarter turns, 1.74° at 10° | `G-B.leftover.*` | C: axis (−0.0087, 0.0087, −0.9999), 0.71° off the vertical; 1.7409° at 10° (7.05° off); 120° about a diagonal at 90° | OK | |
| 158 | 15_4 | S: "For small turns, make both ten times smaller, and the leftover shrinks a hundredfold." | `G-B.leftover.10`, `.1` | C: 10° to 1°: ×99.75; 1° to 0.1°: ×99.997 (90° to 9°: ×85, so "for small turns" is needed and said) | OK | |
| 159 | 15_4 | S: "On the flat piece, that leftover follows a rule of its own, called the Lie bracket. It measures how much order matters." | ledger: the Lie bracket | C: leftover angle equals the product of the angles (0.017453° for 1° × 1°) | OK | |
| 160 | 15_4 | P: lower third "leftover ≈ angle × angle (in radians) about roll's arrow × tilt's arrow: [x, y] = z" | `G-B.leftover.1.axis` −0.0087,0.0087,−0.9999 | C: the leftover is about −z = y × x, which is roll's arrow × tilt's arrow, as the first half says. "[x, y] = z" is the other order (tilt × roll = +z) | MISMATCH (order and sign, on screen) | "…about roll's arrow × tilt's arrow: [y, x] = −z" (or the general rule "[a, b] = a × b") |
| 161 | 15_5 | S: "So the arm multiplies its motions, and does its adding on the flat piece. That is what it means to think in Lie groups." | the whole video | summary of rows 91 and 105 | OK | |
| 162 | 15_5 | P: receipts card "every number in this video was computed by code" | fact sheet G-L | The cited numbers are typed from sources, not computed: 1775, 1763, 1830, 1840, 1983, 70°, 85°, 105 hours, 1842–1899, 75°/s, the arm's lengths | EDGE FAILS (low) | "every computed number in this video came from code; every cited one is sourced in the description" |
| 163 | vocab | "platform": Apollo's guidance platform (03_4, twice) and KUKA's mobile platform (06_1, 06_2) | vocabulary list: "platform (the mobile one)" | W | MISMATCH (one word, two things) | 03_4: "Its inertial reference hung in three gimbals. … and the reference had to be realigned." (the lower third already says "reference") |
| 164 | vocab | "step": an amount on the flat piece (05_1), the way back to the curved space (05_3 "That step…", 05_6 "The step has a formula"), a Newton step (13_5) | vocabulary list: "step" | W | MISMATCH (one word, two things, inside chapter 05) | 05_3: "That move, from the flat piece back to the curved space, is the exponential." 05_6: "The exponential has a formula, named after Rodrigues." |
| 165 | vocab | "wrist": first spoken at 11_3 ("push on the wrist", a point), defined at 11_6 ("three joints whose axes meet at one point"); 10_4's picture names joint 7 "wrist" | ledger has no row | W | MISMATCH (low: used before defined; one label, two things) | 10_4: name joint 7 "last joint"; 11_3: mark the wrist point when the word is first said |
| 166 | vocab | "tilt", "roll": only the book's two turns (01_1, 01_2, 02_1, 05_5, 15_2); the rings are never called roll in words or on screen | vocabulary list | W | OK | |
| 167 | vocab | "tip": always the gripper's tip (10_4, 10_5, 14_5) | vocabulary list | W | OK | |
| 168 | vocab | "wrap … back on": spoken at 04_5 and 06_4 in one sense | vocabulary list | W (the on-screen exception is row 49) | OK | |
| 169 | vocab | "base": only the arm's base (09_4, 09_5, 10_5); the mobile one is always "platform" | vocabulary list | W | OK | |
| 170 | ledger | every ledger term is first spoken on its ledger line: pose 01_5, group 02_5, matrix and grid 02_6, gimbal 03_1, gimbal lock 03_3, flat piece 04_5, spin 05_1, exponential 05_3, Lie group and Lie algebra 05_7, twist 07_2, screw 07_3, Jacobian 10_3, direction 11_1, singularity 11_4, spare joint 12_1, leftover 15_3, Lie bracket 15_4 | jargon ledger | W. Note: "ways" (03_3, 07_1) becomes "directions" at 11_1, two words for one idea | OK | |
| 171 | quizzes | fifteen quizzes; each card lists choices of which exactly one is right, and the next beat states it | `G-Q.*` | rows 1, 15, 29, 44, 56, 66, 73, 88, 101, 108, 114, 128, 137, 144, 155; every countdown on the card equals its `G-Q.*.seconds` | OK | |

## Every non-OK row, most important first

1. **Row 126, 11_6, EDGE FAILS.** "A six-joint arm would lose a direction here. The seventh joint covers for it." With joint seven removed the L shape still has rank 6; it is the arm without joint 3 that drops to 5. Joint 7 is the axis that became a duplicate. Fix: "A six-joint arm, with only two joints at its shoulder, would lose a direction here. This arm's third shoulder joint covers for it."
2. **Row 112, 10_5 picture, MISMATCH.** Legs "42 and 53" give 67.6, shown beside "67". Fix: label the leg 52.6.
3. **Row 42, 03_6 lower third, MISLEADING.** "no continuous one-to-one naming … by three numbers" is true, but the ticked unit quaternion is not one-to-one either. Fix: "every naming of all orientations by three numbers must somewhere jump or lose a direction; four numbers need not (two names each, q and −q)".
4. **Row 56, 05_4, MISLEADING.** "the line leaning equally from its three edges" names four lines, and 02_3 used the words for a different one. Fix: "the other such line, the one between its three edge arrows".
5. **Row 93, 08_6, MISMATCH.** "Each screw, held for its joint angle" against the card (a screw is how far; a twist is held). Fix: "Hold a joint's twist for its joint angle, and you get its screw. That is an exponential."
6. **Row 160, 15_4 lower third, MISMATCH.** "about roll's arrow × tilt's arrow: [x, y] = z": the first half is −z, the second +z. Fix: "[y, x] = −z".
7. **Row 18, 02_3 picture, EDGE FAILS.** The axis cannot make "the same angle with each edge arrow": 54.7°, 54.7°, 125.3°. Fix: measure against the edges' lines.
8. **Row 152, 15_1, UNSOURCED.** "Mapping robots average them." Fix: "estimate", or a source.
9. **Row 130, 12_3, UNSOURCED.** "your arm has seven joints too": the cited exercises give no count. Fix: add a source that states it.
10. **Row 142, 13_6 tag, UNSOURCED.** "For this arm a closed formula also exists": no citation. Fix: cite one or cut.
11. **Row 151, 15_1, UNSOURCED.** "Drones steer poses": record only. Fix: quote the arXiv:1003.2005 abstract already downloaded.
12. **Row 35, 03_4, UNSOURCED (in the sheet).** "the platform had to be realigned": add Fjeld's sentence from the ALSJ page already downloaded.
13. **Row 53, 05_2, EDGE FAILS (low).** "Add two spins … a third answer" fails for spins about one axis. Fix: "Add these two spins".
14. **Row 163, MISMATCH.** "platform" is Apollo's at 03_4 and KUKA's at 06. Fix: "inertial reference" at 03_4.
15. **Row 164, MISMATCH.** "step" is an amount at 05_1 and the exponential at 05_3 and 05_6. Fix: "That move…"; "The exponential has a formula…".
16. **Row 49, 04_4 tag, MISMATCH.** "wrap the difference" is the opposite direction from "wrap the answer back on". Fix: "bring the difference into −180…180".
17. **Row 165, MISMATCH (low).** "wrist" is spoken at 11_3 before 11_6 defines it; 10_4 labels joint 7 "wrist".
18. **Row 99, 09_3, EDGE FAILS (low).** "swings everything". Fix: "everything above it".
19. **Row 81, 07_5 picture, MISLEADING (low).** "312 of 312": 264 distinct poses, 13 not moved.
20. **Row 97, 09_1 label, MISLEADING (low).** "to the gripper's tip" is KUKA's wrist-to-flange length.
21. **Row 162, 15_5 card, EDGE FAILS (low).** "every number … was computed by code": the cited ones were not.

## Passed, with a condition to keep

- Row 14 (02_1 picture): the single turn's axis must pass through the point where the two table lines cross.
- Row 44 (04_2): 0° is the same distance from both headings the long way round; "between" carries the quiz.
- Row 39 (03_6): "bad spot" must be allowed to mean a forced jump as well as a lost direction (the spin arrows' bad spot is a jump at the half turn).
- Row 12 and row 95: 01_6's "it" (about 150 years old) is the idea; 08_6's "it" (1983) is the line.

## Stale quotes in the fact sheet (the script is what is spoken)

| Sheet | Sheet says | script.tsv says |
|---|---|---|
| ledger, spare joint | "seven joints, six directions: one joint is spare" | 12_1 "Seven joints for six directions leaves one joint spare." |
| 01_4 | "Positions add. Orientations don't." | "So positions add, and orientations don't." |
| 03_6 | "Name every orientation with just three numbers, and somewhere there is a bad spot." | "Name every orientation with only three, and somewhere there is a bad spot." |
| 03_6 | "The spare ones are the price." | "the extra ones buy a naming with no bad spot." |
| 04_3 | "The honest halfway is a hundred and eighty." | "The true halfway is a hundred and eighty, the exact opposite." |
| 05_2 | "But adding two spins is not one turn then another." | "Add two spins, and you get both turns at once, a third answer." |
| 13_6 | "From this start, three." | "From here, three." |
| 15_1 | "so it stays rigid" | "so the body stays rigid" |
| finding 7 | the script says "for this start and this goal" | "From here, three." |
| finding 8 | the narration says "there is rarely a tidy formula" | not spoken; the tag at 13_6 says "For this arm a closed formula also exists" |
| finding 3 | labels the grid, speeds and paths "model" | lower third "Our numbers, not KUKA's" |
| finding 11 | 16.3 min | the chapter headings now sum to 985 s, 16.4 min |
| Rodrigues p. 386 quote | "…tome XIV" (1830) | the page reads "tome XIV, année 1830." (image checked); the sheet's caveat "to be checked against the page image" can be closed for this quote |
| row labels | "G-P the base" | the video's word for it is "platform"; "base" is the arm's |

Also confirmed though not on screen: the palette's nine roles are at least
ΔE2000 20 apart (my CIEDE2000: least 20.7, pink against violet; 20.8 orange
against gold); the storyboard's note figures 0.29 m/s, 0.58 m, 0.82 m, rank 4
stretched out, 0.7° and 7° off, 5.1% wider all recompute.

## Names (P3 probe, 2026-10-13, michael)

Probed with `probe_names.py run --voice michael` (three carriers, a control and a decoy each);
`lexicon.json` keeps each name's respelling and what the ASR heard. Respelled because the plain
spelling is misread: Lie as "Lee", Euler as "Oiler", Sophus as "Soofus", Chasles as "Shahl"
(plain: "Chassels"), Mozzi as "Motsee" (plain: "Mozzie"), KUKA as "Kuka" (the capitals read
unsteadily). Read as written, heard right: Apollo, Collins, Newton, Brockett, Jacobian, gimbal (hard
g; "JIM-bul" was heard as "Jim Bull") and Rodrigues ("rod-REE-gez" took a soft g). For the owner's
ear or the narration sheets: Rodrigues (05_6), Chasles and Mozzi (07_6), Brockett's stress (08_6),
KUKA (01_5, 06_1), Sophus (05_7, the English SOH-fus is the alternative) and gimbal's g.
