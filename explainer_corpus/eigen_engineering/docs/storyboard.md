# Storyboard — The One Idea Behind Bridges, Concert Halls and Economies

The hand-edited source of this video's words and pictures. `python3 -I
tools/storyboard.py eigen_engineering` checks it (gate G2) and writes
`script.tsv` from the words column. Facts: `docs/fact-sheet.md` and
`oracle.py` (every number below names its fact key); brief: `docs/brief.md`;
reviews and their dispositions: `docs/reviews/eigen_engineering/`.

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a question for the
viewer; the beat ends with the question and the picture holds before the
next beat answers it.

**The "your turn" device** (Video F's, shared with Video J). A quiz is a
`⟨predict⟩` beat whose last column says `listen N s (countdown)`: a violet
card with the problem, a ring that drains over N seconds with the seconds
left in its middle, the line "pause for more time" under it, the narration
silent and the music alone (one soft note a second). The next beat works the
answer in the same place on screen. Every quiz is a row of `model::QUIZZES`,
and its numbers are the `K-Q.<line>.*` facts. The owner asked for quizzes
with pauses in every video: seven countdowns, each a sum or a choice Maya
can make in her head, none closer than a minute to the next, and eight
plain prediction pauses (plan rule 10). Each countdown is checked on the
draft against its cue times (the question readable by the last word, the
answer only after the ring ends, no hint during it).

**The one picture** (R1 C1). Two devices recur in every act: the arrow test
(a move in blue, the push that holds it in orange, an angle readout, gold
when they line up: the carts in 03, the truss in 06_1, the air in 09_3, the
economy in 12_5 and 13, the quarter turn in 14_5), and the ladder of own
patterns, one rung per pattern, where the split happens (two rungs in 04,
21 in 06, the economy's two in 13_4, Λ in 14_1).

## 2 The arc

One idea, shown small, then found in three places, then bounded.

| Act | Chapters | Spine |
|---|---|---|
| I. The idea, on two carts | 01 the bridge that swayed, 02 two carts, 03 own patterns, 04 the split | a bridge that was checked and still swayed; move one cart and both need a push; two directions in which the pushes come back along the moves; in those patterns two tangled equations become two divisions |
| II. The bridge | 05 a steel truss, 06 twenty-one patterns, 07 every mode a note, 08 two bridges | members as springs, written into one table, and one load moving every joint; the arrow test on the truss and its 21 own patterns, the softest landing within half a percent of the sag while the forces need many; with mass, the modes, each a note; Tacoma and London told as the sources tell them |
| III. The hall | 09 the air in a room, 10 two million modes, 11 where modes rule | air as carts and springs, a round trip, and tones you can test in a bathroom; a hall with two million modes below 1 kHz that blur together above 20 Hz; so halls are designed with statistics, and modes rule small rooms and shapes with no formula, cut into cells |
| IV. The economy | 12 an economy, 13 can it feed itself | a table of what making things uses up; its rounds on the arrow plane, a lopsided order swinging onto the even mix, so the largest eigenvalue decides; the factory problem of the owner's third image appears once, in 14_5, as what it is: linear programming |
| V. One idea | 14 one idea | the split run forwards is K = QΛQᵀ; why symmetric tables split cleanly and what nothing negative buys; where the idea stops; the footbridge again, with the three questions its engineers had to answer |

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses a
data colour.

| Idea | Color |
|---|---|
| what goes into the rule (a move of carts or joints; a mix of goods) | blue `#58C4DD` |
| what the rule gives back (the pushes; what making things uses up) | orange `#F2994A` |
| an own pattern: the two line up (an eigenvector, a mode) | `hero` gold `#FFD866` |
| squeezed (a member in compression; air pressed together) | magenta `#E0408F` |
| stretched (a member in tension; air thinned out) | deep teal `#26867C` |
| the "your turn" card and its countdown | violet `#B189D6` (Videos F and J use the same) |
| holds, allowed, can feed itself | `good` green `#83C167` |
| fails, not allowed | `warn` red `#FC6255` |

Every pair of these, and each against `ink`, is at least ΔE2000 20 apart
(least 20.9, orange against gold), by a test. The first choice of hues
(`#F0AC5F`, `#EA87B3` and `#5CD0B3`) was not: the first sat 14.9 from gold,
the second 16.7 from the quiz violet and the third 17.3 from green. The
violet is a card's fill and never a line or a word beside `dim` text, from
which it is only 15.4.

## Vocabulary (one word per idea)

**the rule** (what a table of numbers does; "matrix" is said once, as its
name); **arrow** (a list of numbers, drawn); **move** and **push** (never
"displacement", "force vector"); **pattern**, **own pattern**, then its two
names, **eigenvector** (mathematics, from 03_6) and **mode** (engineering,
from 07_1, once mass is counted; then the bridges and the rooms);
**eigenvalue**, and for a structure **stiffness**
(**soft**, **stiff**); **mix** (never "linear combination"; "superposition"
once, in a lower-third); **joint** and **member** (never "node", "bar",
"element"); **squeezed** and **stretched**; **hertz** for the number of swings
a second, **tone** for a room's mode as heard ("standing wave" only on
screen); **round** and **uses up** for the
economy; **your turn** (every quiz). Never "simply", "just", "magic" or
"secret".

## 01_the_bridge_that_swayed (~57 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | London, June two thousand. Under nothing but footsteps, a new footbridge sways sideways by about seven centimetres. Two days later it is closed. And it had been checked. | Frame one: the deck from above, already swinging side to side under walking dots (their paths carried with it, not in step); "10 June 2000"; a ruler reading 7 cm (`K-B2.sway_cm`); "closed 12 June" stamps across it (`K-B2.closed`); lower-third "80,000 to 100,000 people on the first day" (`K-B2.first_day`) | particles on a path; morph on a tracker | |
| 01_2 | A bridge has ways of moving that are all its own. So does the air in a room, and so does an economy. One calculation finds them all. | Three small pictures side by side, each moving in one pattern drawn in gold: a truss sagging and springing back, the air in a box sloshing, two bars of an economy rising in step | lagged reveal; morph | |
| 01_3 | *(title)* | **The One Idea Behind Bridges, Concert Halls and Economies**, set on two lines | title card | listen 3 s (the motif: the overtones of one note) |
| 01_4 | Its designers had checked how it would bounce. The British code they worked to said nothing of walkers pushing a bridge sideways. | The deck again; two small icons: up-and-down bounce with a green tick, side-to-side sway with a question mark | highlight |  |
| 01_5 | ⟨predict⟩ So what does a bridge have, besides strength, that footsteps can find? | The question as a card over the still deck | prediction card | hold |
| 01_6 | A way of moving all its own. This one swings sideways between half a swing and one swing a second. And a walker's weight shifts from side to side about once a second. | The deck's swing drawn as a gold curve with a counter "about 0.5 to 1 swing a second"; under it a walker seen from behind, weight shifting left and right, a counter "about 1 a second"; the two counters drift in and out of phase, never locked (`K-B2.lateral_hz`, `K-B2.walker_lateral_hz`) | two live counters, not locked | |

- **Cold open** (plan rule 20; R1 C4): the swaying deck is the first frame,
  the paradox ("it had been checked") lands by about 10 s, the payoff
  preview by about 20 s, then the title. Gold means "its own" from the
  first minute.
- **Misconception defused:** "a bridge that moves must be too weak". The
  engineers' paper calls it a loading effect "which had not been anticipated
  during design"; nothing in it is about strength.
- **Ruth:** only what the sources say (fact sheet §Sources): the dates, the
  crowd, 70 mm, the British Standard "being updated to cover the
  phenomenon". 01_6 gives the measured range (0.48 to 1.03 Hz) and does not
  set two rhythms in step (R1 R2): the counters drift.
- **Maya:** no number she has to hold; the question is the hook.

## 02_two_carts (~74 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Two carts between two walls, joined by three springs. Each spring pushes back one newton, the weight of an apple, for every centimetre it is stretched or squeezed. | Two carts between two walls, three springs drawn as zigzags; a ruler under the track; "1 newton per centimetre" beside a spring; a small apple icon | **the carts**; grow; lagged reveal |  |
| 02_2 | Slide the left cart one centimetre to the right, and hold the right cart still. | The left cart slides 1 cm (a blue arrow marks the move); the left spring stretches, thins and tints teal; the middle one squeezes, thickens and tints magenta; two hands appear | morph on a tracker; **moves in blue**; **stretched teal, squeezed magenta** | |
| 02_3 | Two springs fight the left cart, so that hand pushes with two newtons. The middle spring shoves the right cart, so the other hand pulls back with one, a push of minus one. | Orange arrows on the carts: 2 to the right on the left cart, 1 to the left on the right cart; the list "moves 1, 0" beside "pushes 2, −1" (`K-A1.push.left`) | **pushes in orange**; live labels |  |
| 02_4 | You moved one cart, and had to push two. Each part is tied to its neighbours. | The two lists side by side, the 0 in the moves and the −1 under it in the pushes ringed; behind the carts, faint, a truss and a grid of air cells, every piece joined to the next | highlight; lagged reveal | |
| 02_5 | So there is a rule that turns the two moves into the two pushes. Write it as a table, called a matrix. Along each row, multiply each number by its move, and add. | The table 2, −1 over −1, 2 between the two lists, the carts still above, the word "matrix" under it, small, as it is named; the top row lights as "2 × 1 + (−1) × 0 = 2", then the bottom row as "(−1) × 1 + 2 × 0 = −1" (`K-A1.table`) | table built from primitives; highlight |  |
| 02_6 | ⟨predict⟩ Your turn. Pause if you want more time. Slide both carts one centimetre to the right. What are the two pushes? | The quiz card: the carts drawn on it, the table, "moves 1, 1", "pushes ?, ?", the ring draining from 5, "pause for more time" (`K-Q.02_6.*`) | **the quiz card**; countdown ring on a tracker | listen 5 s (countdown) |
| 02_7 | One and one. The middle spring is not stretched at all. The pushes come back in the same pattern as the moves. | Both carts slide together, the middle spring unchanged and white; "2 × 1 + (−1) × 1 = 1" twice; pushes 1, 1 land beside moves 1, 1 and the two lists pulse together in gold (`K-A1.push.together`) | morph; **gold for a pattern that comes back** | |

- **A situation before an abstraction:** springs you can count before any
  table; the rule is read as two sums before it is called a matrix, and the
  recipe (along a row, multiply, add) is spoken (R1 M1).
- **The honest failure in miniature** (02_4).
- **Ruth:** the rule is "the pushes that hold these moves", f = K u, in that
  direction every time; the reverse question is chapter 04.
- **Engine:** two rectangles moved by placement; three springs as fixed
  polylines written by one guarded updater (R1 E17).

## 03_own_patterns (~67 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | ⟨predict⟩ Now slide them one centimetre toward each other. What pushes does that take? | The carts close in, the middle spring squeezed hard and deep magenta; "moves 1, −1", "pushes ?, ?" over the live carts | prediction card | hold |
| 03_2 | Three and minus three. The middle spring is squeezed from both ends, so it counts double. The same pattern, three times the size. | Pushes 3, −3 beside moves 1, −1; gold pulse (`K-A1.push.against`) | morph; highlight | |
| 03_3 | Draw the moves as one arrow, the left cart's move across and the right cart's up. Each direction is one way to move the carts. Draw the pushes the same way. | A plane under the carts. First the three pairs already found, each with its cart picture: 1, 0 with 2, −1, not lined up; 1, 1 with 1, 1 and 1, −1 with 3, −3, lined up. The carts stay on stage above, driven by the blue arrow, with faint drop lines to its two numbers | **the arrow plane**; the carts driven by the arrow |  |
| 03_4 | Swing the move round, and the push runs ahead of it or falls behind. They line up along two lines only. Both carts together, and the carts against each other. | The blue arrow sweeps round, the orange arrow recomputed from it every frame and drawn thinner, inside the blue; an angle readout (rounded) (`K-A2.worst_turn_deg`); the sweep slows on the two lines, each a wide soft gold band to the frame's edges; the orange arrow slides out from under the blue on "together" so both are seen (`K-A2.lined_up_deg`, `K-A2.directions`) | live recomputation from a moving object; highlight | |
| 03_5 | These are the system's own patterns. Move the carts in one of them, and the pushes it takes come in that same pattern. Only the size changes. | On each gold line, the blue arrow and its orange answer: the same length along "together", three times as long along "against"; lower-third "largest turn: sin = (3 − 1) ÷ (3 + 1), 30°" | morph | |
| 03_6 | An own pattern is called an eigenvector, and its size change an eigenvalue. Eigen is German for own. For the carts, the eigenvalues are one and three. | The names land on the picture: "eigenvector" on a gold line, "eigenvalue" beside the 1 and the 3; a small card "eigen (German): own"; lower-third `K q = \lambda q` (`K-A2.eigenvalues`) | TeX equation; card | hold |

- **Show, then name** (plan rule 7): a pause, a sweep, then the word. "Mode"
  waits for chapter 07 (R1 M10, R6).
- **One picture carries the idea:** the arrow that comes back along itself.
  It returns on the truss (06_1), the air (09_3), the economy (12_5, 13_4)
  and the limits (14_5).
- **Ruth:** an eigenvector is a direction: the gold marks are lines through
  the origin, and the sweep meets each twice. The largest turn is exactly
  30°.
- **Dev:** the readout is the angle between u and K u, not the stretch.

## 04_the_split (~73 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | ⟨predict⟩ In real life you know the push and want the moves. Push the left cart with one newton and leave the right one free. Where do the carts end up? | The carts at rest; one orange arrow of 1 on the left cart; "pushes 1, 0" and "moves ?, ?"; under them, small, the two equations `2a - b = 1`, `-a + 2b = 0` | prediction card; highlight | hold |
| 04_2 | That push is a mix of the two patterns. Half of together, plus half of against. | In the arrow plane, the orange arrow 1, 0 splits along the two gold lines into two shorter arrows; beside it `1, 0 = \tfrac12 (1, 1) + \tfrac12 (1, -1)` (`K-A3.mix`) | morph; TeX equation | |
| 04_3 | Each pattern answers by itself, and for springs its eigenvalue is called stiffness. Together has stiffness one, so its half stays a half. Against has stiffness three, so its half shrinks to a sixth. | Two rungs of a ladder beside the plane, one per pattern, "÷ 1" and "÷ 3"; `\tfrac12 \div 1 = \tfrac12` and `\tfrac12 \div 3 = \tfrac16`; the two orange pieces turn into blue ones, the second a third as long (`K-A3.shares`) | **the ladder of own patterns**; TeX derivation; morph | |
| 04_4 | Add the answers. The left cart moves two thirds of a centimetre, and the right cart, which nobody pushed, one third. | The two blue pieces add tip to tail into one arrow; `\tfrac12 + \tfrac16 = \tfrac23`, `\tfrac12 - \tfrac16 = \tfrac13`; the carts move there; then, on screen only, the check: the table turns 2/3, 1/3 back into 1, 0 (`K-A3.moves`) | morph; TeX | |
| 04_5 | ⟨predict⟩ Your turn. Push the right cart instead. Where do they end up? | The quiz card: the carts with the orange arrow on the right cart, "moves ?, ?", the ring from 5 (`K-Q.04_5.*`) | the quiz card | listen 5 s (countdown) |
| 04_6 | One third and two thirds. The same answer, turned round. | The carts settle at 1/3, 2/3; the arrow-plane picture mirrored across the "together" line (`K-A3.moves_right`) | morph | |
| 04_7 | Two tangled equations have become two divisions. Seen in its own patterns, a system like this falls apart into pieces that ignore each other. | A cleared stage, the sentence as a card; lower-third "names to search: eigendecomposition, diagonalization, superposition; for programmers: the split is this simple because these patterns are at right angles" | card | hold |

- **Earn the formula** (plan rule 6): the split is done once with halves and
  sixths; the letters wait for the truss (06_5).
- **Close the act with one sentence** (04_7, rule 14); a breath of silence
  before it (R1 C20).
- **Ruth:** the mix uses the patterns as drawn, (1, 1) and (1, −1); ½ and ½
  are the coefficients, 1 and 3 the eigenvalues. Exact fractions
  (`K-A3.*`), checked against solving the two equations directly.

## 05_a_steel_truss (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Now a steel truss bridge, a frame of triangles twenty-four metres long. One end is pinned, the other sits on a roller. | The truss draws itself joint by joint and member by member, the posts and diagonals thinner than the chords; legend "chords and end posts 6,000 mm²; posts and diagonals 3,000 mm²" (`K-T1.chord_mm2`, `.web_mm2`); a pin and a roller; dimension lines 24 m and 4 m (`K-T1.span_m`, `.depth_m`); tag "a model" | **the truss**; lagged reveal |  |
| 05_2 | Each member is a spring. This bottom one takes three hundred kilonewtons, the weight of about thirty tonnes, to stretch it one millimetre. | One bottom member glows and stretches a hair; the numbers first, `200\,\text{kN/mm}^2 \times 6000\,\text{mm}^2 \div 4000\,\text{mm} = 300\,\text{kN per mm}`, then `k = EA/L` under them with words: how stiff steel is, times how thick, over how long (`K-T1.e_gpa`, `K-T1.spring_kn_per_mm`, `K-T1.spring_tonnes`) | TeX: numbers, then letters | |
| 05_3 | Each joint can move sideways and up or down. Twelve joints make twenty-four moves, and the supports hold three, so twenty-one are left. | 12 × 2 − 3 = 21 on the truss alone, and 21 small blue arrows on the joints (`K-T1.joints`, `K-T1.free`) | TeX; lagged reveal | |
| 05_4 | So the rule is a table again, twenty-one by twenty-one, and most of it stays empty. | Beside the truss a 21 by 21 lattice; one member lights and its cells fill (a lower-third says why: each member writes its spring only into its own two joints' rows), then the rest in a rush, brighter for stiffer; a mirror line along the diagonal last (`K-T2.entries`, `K-T2.nonzero`, `K-T2.symmetric`); on the hold, a code panel, for programmers: `for each member a to b: (c, s) = (b - a) / L; g = (-c, -s, c, s); rows r = (2a, 2a+1, 2b, 2b+1); K[r_i][r_j] += (EA/L) g_i g_j; then strike out the 3 held rows and columns`; lower-third "forget the supports and the table has three own patterns of stiffness zero: slide, lift, turn" (`K-X3.unsupported_zero`) | **the stiffness table**; cells recoloured in place; code panel | hold |
| 05_5 | ⟨predict⟩ Hang a hundred kilonewtons on one joint. Will only that joint move, its neighbours, or every joint? | One orange arrow down on the second deck joint, "100 kN, about 10 tonnes"; three choices on a card (`K-T3.load_tonnes`) | prediction card | hold |
| 05_6 | Every joint but the pinned one. Each is tied only to its neighbours, and still they all move. | The truss deforms, drawn 200 times larger than life (tag "moves × 200"); every joint's blue arrow grows except at the pin, which is held and has none (tagged "held"); "4.7 mm" at the loaded joint, "1.3 mm" at the roller; the lattice flips to the table's inverse: "the table: 97 of 441 not zero; its inverse: 441 of 441" (`K-T3.moving`, `.drop_mm`, `.slide_mm`, `K-T2.inverse_nonzero`) | morph on a tracker; live labels |  |
| 05_7 | Load all five deck joints. The middle sags eighteen millimetres, less than a finger's width. | Five orange arrows of 100 kN; `K u = f` over the lattice, tagged "21 equations, solved together"; the truss sags smoothly; "17.96 mm" at midspan (`K-T4.sag_mm_2dp`); lower-third, for programmers: "real programs factor K once, or iterate, and reuse it for every load; nobody computes its eigenvectors for this" | TeX; morph; lower-third |  |
| 05_8 | The top is squeezed hardest, a force of four hundred fifty kilonewtons, under a third of what makes its steel yield. | A lower-third, on screen only: "a member's force is how far its ends moved apart, times its stiffness". Members take colour and weight: magenta and thick along the top (−450), teal along the bottom (up to +400), the middle post thin and grey (0); a gauge "75 of 250 N per mm², yield"; tag "a squeezed member is also checked for buckling, another eigenvalue problem" (`K-T5.top_kn`, `.bottom_kn`, `.middle_post_kn`, `.stress_mpa`, `K-T5.stress_percent_of_yield`) | **squeezed magenta, stretched teal**; lagged colouring; live bar | |

- **Tiny example, then scale up** (plan rule 5): the count of moves is
  spoken (R1 M6), and the table is built a spring at a time.
- **Numbers before letters** (05_2), in units that cancel (R1 M8, R25).
- **Ruth:** every number has two independent sources (the stiffness method
  in `model.rs`; joint equilibrium and the unit-load method in the oracle).
  The gauge is labelled yield, and the buckling tag says what it does not
  check (R1 R10, D15).
- **Dev:** the code panel is the direct stiffness method in full (R1 D2);
  the inverse's 441 of 441 is why programs factor and never invert (D26).

## 06_twenty_one_patterns (~72 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Test the truss as we tested the carts. The move one load caused is held by a push at a single joint, a different pattern. | The full-size truss: blue arrows at every joint for the move of 05_6; the push that holds it, one orange arrow at one joint; the angle readout from chapter 03 in the corner, "64°" (`K-P3.lone_turn_deg`) | **the arrow test on the truss**; live readout |  |
| 06_2 | But some moves come back as themselves. This table has twenty-one own patterns, its eigenvectors, each with a stiffness of its own. | The move morphs into pattern 1: an orange arrow grows at every joint along its blue one, the readout reads 0°, the truss turns gold (`K-P3.softest_turn_deg`); then the wall: all 21 patterns as small gold trusses, one on each rung of a ladder spaced by rank, each bar's length the log of its stiffness (`K-P1.count`) | **the ladder of own patterns**, 21 rungs; lagged reveal | |
| 06_3 | ⟨predict⟩ Your turn. Which is the softest? The whole span sagging as one, or neighbouring joints moving opposite ways? | The quiz card: the bottom and top rungs' trusses lifted out of the wall, the ring from 5 (`K-Q.06_3.*`) | the quiz card | listen 5 s (countdown) |
| 06_4 | The single sag. Moving together stretches little, and moving against stretches a lot. The stiffest pattern is nearly four hundred times stiffer. | The two patterns move side by side with their rungs: 3.12 and 1,226 kN per mm, "× 393"; the two cart pictures from chapter 03 beside them, small (`K-P1.softest_mn_per_m`, `.stiffest_mn_per_m`, `.ratio`) | morph; live numbers | |
| 06_5 | Now split the load into the patterns, as with the carts. Each share is divided by its stiffness, so here the soft ones win. | The five orange arrows break into 21 orange bars on the rungs; each is divided by its stiffness and turns blue; the top of the ladder all but vanishes; the letters arrive from the carts' halves and sixths: `u = \sum_i \frac{q_i \cdot f}{\lambda_i \, q_i \cdot q_i} q_i` | **TransformMatchingTex**: the carts' fractions → letters; bars | |
| 06_6 | The softest pattern alone lands within half a percent of the sag at the middle. Good enough for how far it sags. | The truss rebuilt from pattern 1 laid over the true shape; a readout steps as patterns are added: 99.6, 102.0, 101.8, 99.1, 98.9, then 99.1 at ten and 100.0 at fifteen (`K-P2.sag_percent.*`) | morph; live number | |
| 06_7 | Not for what is inside it. A force puts the stiffness back in, so the stiff patterns count, and with one pattern the worst force is off by eighty-two kilonewtons. | A lower-third, on screen only: "a force is a stretch times a stiffness, which puts back what the division took out"; and a closing card, "a few patterns tell you how a structure moves, not everything inside it". A second readout: "worst member force off by 82 kN" with 1 pattern, 44 with 5, 8 with 15, 0 with all 21 (`K-P2.force_error_kn.*`); then the wall of 21 beside a ladder of a million, its bottom few rungs lit; lower-third "names to search: modal truncation, static correction; a library call finds all 21: numpy.linalg.eigh" | count-up; live numbers; lagged reveal | |

- **The hero picture returns on the thing itself** (R1 C1): the same angle
  readout, the same gold, on the truss.
- **State costs plainly** (06_7): the sag settles, the forces do not, and
  why is said (R1 D5). "Within half a percent" replaces "99.6% of the sag"
  (R1 R5): the readout shows it overshoot and settle.
- **Ruth:** the formula is scale-free (R1 R4, D1); these are K's own
  patterns, and "mode" waits for mass (07_1).

## 07_every_mode_a_note (~59 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | That was a bridge standing still. Now give every joint its mass, and the own patterns shift a little into the structure's modes, each swinging at a frequency of its own. | Masses as discs on the joints, larger along the deck; K's pattern 1 ghosted under the first mode; on stage `K\varphi = \omega^2 M\varphi` and under it `\varphi_i^T M \varphi_j = 0`; lower-third "precisely: the eigenvalue is ω², hertz = ω ÷ 2π; modes are at right angles once mass is counted" | TeX equation; lagged reveal |  |
| 07_2 | With a four tonne deck at every deck joint, the lowest mode swings five point nine times a second. Five point nine hertz. | Mode 1 swings, drawn large; tag "slow motion × 6"; a counter of swings and a clock; "5.9 Hz"; tag "plus the steel's own 3.4 tonnes" (`K-V1.deck_kg`, `K-V1.hz.1`, `K-V1.steel_tonnes`) | morph on a clock; live counter | |
| 07_3 | The next two come at eleven point eight and fourteen point nine hertz. | Modes 2 and 3 swing beside mode 1 at their true relative rates, slowed by the same factor, labelled on screen "mostly slides the deck along its length" and "mostly bends it into an S" (`K-V1.hz.2`, `.hz.3`, `K-V1.shape.2`, `.3`); then the listen window: the three modes as three tones in their true ratios, 1 : 1.99 : 2.51, one at a time, then together (`K-V1.ratio.2`, `.ratio.3`); tag "sound: the same ratios, faster" | three morphs at three rates | listen 4 s (the three modes as tones) |
| 07_4 | ⟨predict⟩ Your turn. Make the deck twice as heavy. Does the lowest mode get faster, slower, or stay the same? | The quiz card: the truss with its deck discs doubling, three choices, the ring from 4 (`K-Q.07_4.*`) | the quiz card | listen 4 s (countdown) |
| 07_5 | Slower, four point three hertz. | Mode 1 beside its heavier twin, visibly slower; "5.9 Hz" and "4.3 Hz"; tag "the same springs, more mass to move; not 5.9 ÷ √2 = 4.2: the steel did not get heavier" (`K-V2.hz.1`, `K-V2.naive_hz`) | two morphs; live numbers | |
| 07_6 | Any small motion of the bridge is a mix of its modes, each keeping its own time. So an engineer finds the lowest few, and asks what could feed them. | A messy shake of the truss resolves into the three clean modes stacked under it; caption "this model moves only in its own plane"; lower-third "names to search: modal analysis; the Lanczos algorithm finds the lowest few modes of a huge table without the rest" | morph; lower-third | |

- **Why this chapter:** where engineers use the decomposition every day, and
  the set-up for both bridges. "Mode" is introduced here (R1 M10, R6, D4).
- **Sound that teaches** (R1 C6): the listen window plays the true ratios.
- **Ruth:** the generalized problem is stated as it is; the model has no
  damping; mode 2 is mostly along the deck because the right support is a
  roller.

## 08_two_bridges (~57 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Tacoma Narrows, November nineteen forty. A suspension bridge four months old twists itself apart in a wind of about forty miles an hour. | A long deck seen end on, twisting about its centre line; "7 November 1940"; "opened 1 July 1940"; "42 mph, measured that morning" (`K-B1.failed`, `K-B1.opened`, `K-B1.wind_mph`, `K-B1.wind_round_mph`) | morph on a clock | |
| 08_2 | You might guess the wind puffed in time with the bridge. It did not. Its puffs would come about once a second, and the bridge twisted once every five. | Two pulses side by side, plainly out of step: "the wind's puffs: about 1 a second (estimate)" and "the twist: 1 every 5 seconds"; lower-third "Billah and Scanlan, American Journal of Physics, 1991" (`K-B1.vortex_hz`, `K-B1.twist_period_s`) | two live counters out of step | |
| 08_3 | What grew was one twisting mode. As the deck twisted, the wind's push followed the motion, and on every swing it gave more than it took back. | The twisting deck; wind arrows largest as the deck passes level, pointing the way it is already turning; the twist grows; quote card, small: "well designed and built to resist safely all static forces, including wind, usually considered in the design of similar structures" (the board of engineers, 1941); lower-third "names to search: aeroelastic flutter" | morph; quote card | |
| 08_4 | ⟨predict⟩ So what was feeding London's footbridge? | The footbridge from chapter 01, still; the question as a card | prediction card | hold |
| 08_5 | Its sideways modes sit in the range where walkers push. And the more the deck moved, the harder walkers pushed, to keep their balance. | The plan view's three spans, each lit and swinging at its own measured rate (centre 0.48 and 0.95, south 0.80, north 1.03 Hz), the walker's push beside them; the deck's sway and the walkers' arrows grow together (`K-B2.lateral_hz`, `K-B2.walker_lateral_hz`); lower-third "Dallard et al.: possible on any bridge with a sideways frequency below about 1.3 Hz, given enough walkers; names to search: synchronous lateral excitation"; a second lower-third, Arup's test: "the north span was stable with 156 people; with just 10 more the movement increased suddenly" (`K-B2.risk_below_hz`, `K-B2.test_steady`, `K-B2.test_added`) | number line; morph | |
| 08_7 | Neither bridge's trouble was strength. Each had a mode, and something that pushed along with it. Engineers call that negative damping. | The two bridges side by side, each with its one mode drawn in gold | highlight | |

- **Honest examples of modes:** both stories from their primary sources.
  The feeding is shown as a push that follows the motion (R1 R3), never as
  a push in time with the bridge; neither "resonance" nor "synchrony" is
  spoken as the cause (R1 M27, D17).
- **Ruth:** 12 a minute is the final motion (Farquharson, through Billah and
  Scanlan); the board has 14 at its onset. "About forty" for 42 measured
  that morning (R1 R23). The board's sentence is quoted whole.

## 09_the_air_in_a_room (~68 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | The same idea, with nothing solid in it. Cut the air in a tube into thin slices, each with mass, and squeezed air pushes back like a spring. | A long tube; its air cut into slices; one slice pushed, its neighbours crowd (magenta) on one side and thin out (teal) on the other and push it back | morph; **squeezed magenta, stretched teal** |  |
| 09_2 | So a tube of air is a row of carts and springs. | Eight slices turn into eight carts with springs; under them the table, 2 down the diagonal and −1 on each side, in the same place on screen as in 02_5, tagged "the same table as before, only longer"; it grows to 40 by 40 as flat cells (`K-R1.slices`) | morph that keeps identity; grid | |
| 09_3 | Push one slice out of place, and the push comes back jagged. One smooth hump comes back as itself. These own patterns slosh in place, one hump, two, three, each with its own tone. | The arrow test on the air: a blue curve of the slices' moves and an orange curve of the pushes; a single slice gives a spike with two dips, "35°" (`K-R1.lump_turn_deg`); one hump gives back one hump, "0°", gold; then two and three humps; small: "40 slices give the lowest tone to within 0.03%" (`K-R1.error_bound_percent`); a quote card over the moving waves: "…eines der Eigentöne des elastischen Körpers…, sei dieser nun eine Saite, eine Membran oder eine Luftmasse" (Helmholtz, 1863: own tones of a string, a membrane or a mass of air), with "Eigenwerte: Hilbert, 1904, 41 years later" (`K-S.helmholtz_year`, `K-S.hilbert_year`, `K-S.eigen_gap_years`); lower-third, for programmers: "hertz = c ÷ (2π × slice width) × √eigenvalue" | **the arrow test on the air**; three morphs |  |
| 09_4 | ⟨predict⟩ One swing is a round trip, there and back at the speed of sound. Your turn. A bathroom is two and a half metres long, so a round trip is five metres. How many round trips a second? | First one magenta pulse of pressed air runs down a tube, bounces off the wall and comes back (`K-R0.sound_m_per_s`); then the quiz card: a small room, "343 m a second", "2 × 2.5 = 5 m", the ring from 6 (`K-Q.09_4.*`) | particles on a path; the quiz card | listen 6 s (countdown) |
| 09_5 | About seventy. Sixty-eight point six hertz, too low for most voices. But hum a slow slide upward in a tiled bathroom, and near a hundred and seven hertz the room hums back. | "343 ÷ 5 = 68.6 Hz"; a figure humming in the room, a pitch line sliding from 60 to 150 Hz, the room's outline glowing gold at every tone it crosses, all nine: 68.6, 71.5, 99.1, 107.2, 127.3, 128.8, 137.2, 142.9 and 146.0 Hz; the five that fit one side are labelled with that side (68.6 and 137.2 the 2.5 m length, 71.5 and 142.9 the 2.4 m height, 107.2 the 1.6 m width), the other four, which need two or three sides, fainter and unlabelled, so the line never reads as the room's whole list (`K-R2.first_hz`, `.second_hz`, `K-R2.voice_hz`; every tone to 150 Hz is a fact still to add, `K-R2.tones_to_150_hz`, pinned in `facts.rs` and `oracle.py`) | number line; glow | listen 3 s (a hum sliding through the room's tones) |

- **Continuity:** the air is the carts (09_2), and the arrow test is the
  carts' (09_3; R1 C1).
- **The rule is earned** (R1 C22, D22): the round trip comes before the
  quiz, and the quiz's numbers are on the card (R1 M12).
- **A "huh"** (the card over 09_3's waves, away from the quiz, R1 M29): the same word, 41 years apart;
  nothing about one borrowing from the other.
- **Maya:** the home test is now in reach of a voice (R1 R12, D23).

## 10_two_million_modes (~56 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | A room has three directions, so a mode counts half-waves along the length, the width and the height. One formula gives every tone of a box with hard walls. | A plan view and a side view of a box, side by side; three counters nx, ny, nz each changing the view that can see it; the pressure maps magenta through the background to teal, swapping each half swing, brightest at walls and corners, labelled "where the air moves least, at the walls, it is squeezed most"; the formula `\text{tone} = \frac{c}{2}\sqrt{(n_x/L_x)^2 + (n_y/L_y)^2 + (n_z/L_z)^2}`, with "length, width, height" set under it; lower-third "FrankenSim: rectangular_cavity_modes" | TeX equation; **heat maps of a mode**, recoloured in place |  |
| 10_2 | Now a concert hall, a box about the volume of Boston's Symphony Hall, with its lowest mode far below hearing. | The box grows until the bathroom is a speck beside it; "45 × 23 × 18 m = 18,630 m³ (a model); Boston Symphony Hall: 18,750 m³"; "lowest mode: 343 ÷ (2 × 45) = 3.8 Hz" (`K-H1.volume_m3`, `K-H1.boston_m3`, `K-H1.lowest_hz`) | morph; live number |  |
| 10_3 | ⟨predict⟩ How many modes do you think it has below a thousand hertz? | The question as a card over the hall; an empty frequency line | prediction card | hold |
| 10_4 | Nearly two million. | Each mode a dot at its three counts on axes scaled in hertz; a quarter circle of radius "frequency" sweeps out over the floor's dots, each one it passes turning gold, while a counter runs to 2,246 at 100 Hz and 1,964,040 at 1,000 Hz, the dots fusing into a fill (`K-H2.count.100`, `K-H2.count.1000`); lower-third "the count grows as the cube of the frequency, and depends on the volume, hardly on the shape" | count-up; dots in a growing region |  |
| 10_5 | Sound here takes about two seconds to die away, so no mode is one exact pitch. Each is blurred about a hertz wide. | A single mode as a peak on a frequency line, with a width; a decay curve beside it, "1.9 s"; "about 1.2 Hz wide" (`K-H1.t60_s`, `K-H2.mode_width_hz`) | curve; live number |  |
| 10_6 | Where three or more modes crowd into one blur, they merge. The crossover is called the Schroeder frequency, and for this hall it is about twenty hertz, the bottom edge of hearing. | The frequency line fills with gold peaks from the true modes, sparse then dense, and the gold drains to grey as they merge into a rough band; a marker "20 Hz" (`K-H3.schroeder_hz`); lower-third "Schroeder frequency ≈ 2000 × √(decay time ÷ volume)" | curve that grows; highlight | listen 3 s (the same hum in the hall: an even tail, no answer) |

- **Scale up honestly:** the same formula, a bathroom and a hall, and the
  count that changes the question (R1 C17: why so many).
- **Misconception defused:** "a concert hall's sound is its modes".
- **Ruth:** modes are counted exactly in whole numbers and against the
  textbook estimate; "three or more in one blur" is the 1962 criterion
  (R1 R13, D9); 20 Hz at a mid-frequency decay time.
- **Engine:** two flat views, not the camera route (R1 C8 against E28; see
  decisions).

## 11_where_modes_rule (~53 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | So almost every note you can hear in a hall lies above the crossover. Halls are designed with statistics, which began with Wallace Sabine. | Rays bouncing round the hall's outline; the outline filling with seats and people, the decay curve shortening as they arrive; Sabine's rule in words, on screen only: "the bigger the room, the longer a sound lingers, and the more it soaks up, the shorter"; and with the hall's numbers: "1.9 s = 0.161 × 18,630 ÷ 1,579 m² of absorption" (`K-H4.sabine_m2`); "Boston Symphony Hall, 1900: worked out this way before it was built" (`K-S.sabine.opened`); lower-third "names to search: Sabine equation, geometrical acoustics, image sources" | particles on paths; TeX |  |
| 11_3 | ⟨predict⟩ Single modes rule in small rooms, where the crossover climbs to two hundred hertz. A room booms when several modes land on one tone. Your turn. Which piles its tones up most, a long narrow room, an ordinary one, or a perfect cube? | The small room's frequency line: 67 separate gold peaks up to a marker at 200 Hz, "67 modes below 200 Hz"; "4.8 × 4 × 2.6 m; dies away in 0.5 s (a model)" (`K-R4.sides_m`, `K-R4.schroeder_hz`, `K-R4.modes_below`); then the quiz card: three small rooms in outline, the ring from 5 (`K-Q.11_3.*`) | curve; count-up; the quiz card | listen 5 s (countdown) |
| 11_4 | The cube. Its three sides match, so three modes share its lowest tone. | Three number lines of the rooms' tones below 200 Hz: the narrow room's and the ordinary room's ticks all apart, the cube's stacked in threes; "343 ÷ 6 = 57.2 Hz × 3" (`K-R3.first_hz`, `K-R3.first_count`, `K-R5.sharing_200.*`); lower-third "three of the most admired concert halls are shoeboxes: the trouble is sides that match, not boxes (Beranek)" | number lines; highlight | |
| 11_5 | And real rooms are rarely boxes, so there is no formula. Cut the air into cells, build the table, and ask for its eigenvectors. The same calculation as the bridge. | An attic studio's side view, 6 m long, its ceiling sloping to 1.25 m, cut into 232 cells; the table builds as in 05_4; mode 1 moves, the map swapping magenta and teal, about 300 motes crowding and thinning with it, then the next two at their true ratio; "29.8, 56.1, 59.9 Hz" (`K-N1.cells`, `K-N1.hz.*`); lower-third, for programmers: "one unknown per cell; each pair of touching cells adds 1 to both diagonal entries and −1 between them; a wall adds nothing; skip the first eigenvalue, 0"; lower-third "FrankenSim's solver finds the lowest modes of a 100,489-cell grid" | **heat map on a room that is not a box**; motes; cells recoloured in place | hold |

- **State limits plainly:** the hall act ends on where the eigen-idea stops
  being the tool, and where it comes back: small rooms and shapes with no
  formula, in the audible bass (R1 R17, M17).
- **Sabine** is given his rule, not the rays (R1 R16, M31).
- **The rooms** have no sides in simple ratios, so only the cube stacks
  (R1 R9, D7).

## 12_an_economy (~59 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | The last system has no springs. Farms make food, workshops make tools, and making things uses things up. A unit of food takes four tenths of food and two tenths of tools. A unit of tools takes three tenths of food and five tenths of tools. | Two icons, a field and a workshop, tagged "a model", with arrows from each to the other and to itself; the arrows take their numbers and fold into a table, 0.4, 0.3 over 0.2, 0.5, the columns headed "to make 1 food" and "to make 1 tools", the rows "food used up" and "tools used up" (`K-E1.table`) | lagged reveal; table built from primitives; morph |  |
| 12_2 | ⟨predict⟩ This rule works like the carts', along each row. Your turn. Make ten units of each. How much of each is used up? | A blue list "made" goes into the table and an orange list "used up" comes out; the quiz card laid out exactly as 02_6's: the table, "made 10, 10", "used up ?, ?", the ring from 6 (`K-Q.12_2.*`) | the quiz card | listen 6 s (countdown) |
| 12_3 | Seven of each. The same pattern comes back, seven tenths the size. | "0.4 × 10 + 0.3 × 10 = 7" and "0.2 × 10 + 0.5 × 10 = 7"; the two lists pulse together in gold, as in 02_7 (`K-E1.uses_even`) | TeX; highlight | |
| 12_4 | So equal amounts of each is an eigenvector, with eigenvalue seven tenths. Making that mix, the economy uses up seventy percent of what it makes, and keeps thirty. | Two bars, food and tools, each with a blue outline, 70 in orange and 30 in green (`K-E2.largest`, `K-E2.kept_percent`) | bars; highlight | |
| 12_5 | Swing the arrow as before. The other own pattern, with eigenvalue two tenths, needs a negative amount of one good. | The arrow plane of chapter 03, food across and tools up: the sweep; a gold line along the even mix marked 0.7, and a dashed gold line marked 0.2 running into a quarter hatched red where one good is negative (`K-E2.mix`, `K-E2.other`); lower-third, on screen only: "Perron 1907, Frobenius 1912: a table with no negative numbers always has an own pattern with none, and it carries the largest eigenvalue; that eigenvector ranked web pages (Brin and Page 1998)" | the arrow plane; live recomputation | |

- **The one picture returns** (12_5): the same sweep, on a table that is not
  symmetric.
- **"Mix" means only a sum of patterns** (R1 M5); rows and columns are
  labelled (R1 M14); 70% is said of the even mix only (R1 R18, D25).
- **Ruth:** Perron and Frobenius for a nonnegative matrix; strictly positive
  and unique when irreducible, as this one is. Checked on 1,000 seeded
  positive tables and 1,004 with zeros (`K-X6.*`).

## 13_can_it_feed_itself (~62 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | People want twenty-four of food and twenty-four of tools. Making twenty-four of each uses up sixteen point eight of each, so make those too, and so on, round after round. | A household icon, "wants 24, 24"; the line "made = used up + delivered", then `x = Cx + d`, on screen only; then on the arrow plane of 12_5: the demand arrow goes in, the rule gives back an orange arrow "used up", which slides to the tip, turns blue "so make those as well" and goes in again; a chain of arrows each seven tenths as long; small stacks as a readout: 24, 16.8, 11.76 (`K-E3.rounds`) | TeX; **the rounds on the arrow plane**; morph |  |
| 13_3 | The economy keeps three tenths of what it makes. So to keep twenty-four, make twenty-four divided by three tenths. Eighty of each. | The chain walks to a gold cross at 80, 80; "24 ÷ 0.3 = 80"; "check: make 80, use up 56, 24 left" (`K-E3.total`); lower-third "names to search: Leontief inverse, Neumann series" | TeX; live number |  |
| 13_4 | Now a lopsided order, thirty of food and ten of tools. It is a mix of both own patterns, and each round shrinks each part by its own eigenvalue. So the rounds swing onto the even mix. | A chain from 30, 10: its links start askew and swing onto the gold line, 15, 11, then 9.3, 8.5, then 6.27, 6.11 (`K-E5.rounds`); "30, 10 = 18 of 1, 1 + 4 of 3, −2"; "18 ÷ 0.3 = 60 and 4 ÷ 0.8 = 5"; the cross at 75, 50 (`K-E5.shares`, `K-E5.totals_by_pattern`, `K-E5.total`) | the split, again; morph |  |
| 13_5 | ⟨predict⟩ A hungrier economy uses up one point one of each for every one it makes. What happens to the rounds? | A block and, beside it, a block 1.1 times as tall, a question mark for the next | prediction card | hold |
| 13_7 | They grow, and the total never arrives. So the test is one number. With the largest eigenvalue below one, every demand can be met. At one or more, some demand cannot. | The hungry chain, each link longer, runs off the frame in red: 24, 26.4, 29.04 (`K-E4.rounds`), tagged "that economy cannot feed itself"; lower-third, for programmers: "the formula still answers, 24 ÷ (1 − 1.1) = −240 of each; check the eigenvalue, or that the plan has no negative entry" (`K-E4.formula_says`); then a card, "largest eigenvalue < 1: every demand can be met", the 0.7 in green and the 1.1 in red on a line with 1 marked; lower-third "precisely: for a table with no negative entries; below one, every demand; at one or more, some demand fails, and every demand when each industry feeds every other (Hawkins and Simon, 1949)" (`K-X7.*`); then a 44 by 44 grid as lines, faint, beside the 2 by 2: "Wassily Leontief built such tables for the American economy, 44 sectors wide (1936, 1941); Nobel prize in economics, 1973, for the development of the input-output method and for its application to important economic problems" (`K-S.leontief.sectors`, `K-S.leontief.prize_year`) | morph; card; number line; grid | hold |

- **The split, used** (R1 C9, R8, D8, M15): a demand that is not an own
  pattern shows why the largest eigenvalue decides.
- **Make the surprise inevitable** (plan rule 19): the viewer adds the
  rounds by hand (24 ÷ 0.3, R1 M16) before the eigenvalue is named as the
  test.
- **Ruth:** the precise statement is the lower-third (R1 R20).

## 14_one_idea (~74 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | So why does one calculation serve a bridge, a room and an economy? Each is a rule in a table. And in its own patterns, when it has enough of them, a table only multiplies. | Chapter 04 replayed forwards on the carts' plane: the move 1, 0 splits along the gold lines into ½ and ½; each share is multiplied by its rung, 1 and 3; the pieces add tip to tail to 2, −1, the pushes of 02_3; the letters arrive under the steps: `K = Q \Lambda Q^T`, tag "columns of length 1" | TransformMatchingTex: the halves → the letters; the ladder |  |
| 14_3 | Tables like the bridge's and the room's are symmetric, so they always have enough. The economy's is not, but its eigenvalues differ, so it still splits. Having nothing negative makes the pattern with the largest eigenvalue one you can make. | The carts' table with its mirror line beside the economy's, where 0.3 and 0.2 do not match; the carts' two gold lines at right angles; lower-third, on screen only: "symmetric, the same on both sides of the diagonal: the spectral theorem says such a table always has a full set of own patterns, at right angles (for motion, right angles counted with mass)" (`K-X1.*`); then the economy's two lines, not at right angles, the dashed one into the red quarter; `C = V \Lambda V^{-1}` | TeX; highlight |  |
| 14_4 | ⟨predict⟩ And where does the idea stop? | The question as a card | prediction card | hold |
| 14_5 | A quarter turn has no own pattern you can draw. Steel bent too far stops answering in proportion. And some questions about scarce resources are not eigenproblems at all. | Full frame: the arrow plane with a quarter turn, the orange arrow at right angles to the blue all the way round, the readout stuck at 90°; small, a shear with one gold line, tagged "nothing negative, and still only one" (`K-X2.quarter_turn`, `K-X2.shear`); then 05_8's gauge running past 250 and the member's straight push-against-move line bending over; then the factory of the owner's third image as one picture: three limit lines, the allowed region in green, the line of equal profit sliding out to the corner 40, 20, "$460: linear programming" (`K-L2.best`, `K-L2.best_profit`); lower-third "where it holds, the same split gives powers of a table and its exponential: `C^n = V \Lambda^n V^{-1}`; see the companion video on the matrix exponential" | the arrow plane; live recomputation; curve; plot |  |
| 14_6 | London's footbridge reopened in two thousand two. Its engineers had known how it would move. What no code had asked was what would push along with it. | The cold open's view, people walking, the deck still; its sideways pattern a thin gold curve lying along it, the dampers small pistons; "reopened 22 February 2002"; "design model: sideways modes within 10% of measured (Dallard et al. 2001)"; "37 viscous dampers and 4 pairs of sideways tuned mass dampers" (`K-B2.reopened`, `K-B2.predicted_within_percent`, `K-B2.dampers_viscous`, `K-B2.tmd_lateral_pairs`) | particles on a path; highlight |  |
| 14_7 | So there are three questions. How strong is it. How does it want to move. And what will push along with it. | The gold curve breathes once; silence before the last sentence | highlight | hold |
| 14_8 | *(receipts)* | "Every computed number in this video was checked by a second program, written separately. Every cited number has its source in the description." with the repository's address | card | listen 3 s (the resolution) |

- **End on the rigorous form of the opening picture** (plan rule 15; R1 C2):
  the split of chapter 04 run forwards becomes K = QΛQᵀ, and the bridge of
  the cold open returns with the questions its engineers had to answer.
- **Ruth:** the ending says what the sources say (R1 R1): the modes were
  predicted; the load that fed them was not. 14_3 says what Perron and
  Frobenius give and what the split needs (R1 R7).
- **Receipts** (plan rule 26), worded for what was checked (R1 D11).

## Showcase

- [x] TeX derivations: the split with halves and sixths, then letters (04,
  06_5, 14_1); k = EA/L from numbers (05_2).
- [x] value trackers driving live numbers: the angle between move and push
  (03, 06_1, 09_3); the sag and force readouts as patterns are added (06);
  mode counts (10).
- [x] live recomputation from a moving object: the orange arrow is K times
  the blue one, every frame (03, 12, 15).
- [x] morphs that keep identity: slices of air into carts (09_2); the
  truss's deformed shapes from the solved moves (05 to 07); the rounds on
  the arrow plane (13).
- [x] data windows: the ladder of own patterns (04, 06, 15), frequency lines
  (10, 11), the factory's region (14_5).
- [x] tables and grids built from primitives, recoloured in place: the 21
  by 21 stiffness table (05), heat maps of modes (10, 11).
- [x] sound that teaches, in listen windows: the truss's modes as tones
  (07_3), the bathroom's tones (09_5), the hall's even tail (10_6), every
  countdown. (The small room's answers, once 11_2's window, went in P3:
  decisions D-43.)
- [ ] the camera route: not used (decisions D-05).
