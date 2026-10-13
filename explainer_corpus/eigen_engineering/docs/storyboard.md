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
with pauses in every video; here they are the prediction pauses (plan rule
10), ten of them, each one a sum or a choice Maya can make in her head.

## 2 The arc

One idea, shown small, then found in three places, then bounded.

| Act | Chapters | Spine |
|---|---|---|
| I. The idea, on two carts | 01 the bridge that swayed, 02 two carts, 03 own patterns, 04 the split | a bridge that was checked and still swayed; push one cart and both answer; two directions in which the pushes come back along the moves; in those patterns two tangled equations become two divisions |
| II. The bridge | 05 a steel truss, 06 twenty-one patterns, 07 every mode a note, 08 two bridges | members as springs, written into one table, and one load moving every joint; the table's own patterns from soft to stiff, one of them carrying 99.6% of the sag; with mass, each pattern is a note; Tacoma and London told as the sources tell them |
| III. The hall | 09 the air in a room, 10 two million modes, 11 where modes rule | air as carts and springs, and a tone you can test in a bathroom; a hall with two million modes below 1 kHz, which blur together above 20 Hz; so halls are designed with statistics, modes rule small rooms, and a room that is not a box is cut into cells |
| IV. The economy | 12 an economy, 13 can it feed itself, 14 a different tool | a table of what making things uses up; its largest eigenvalue decides whether the rounds add up; the factory problem that is not an eigenproblem at all |
| V. One idea | 15 one idea | why symmetric tables split cleanly, what saves the economy's, the three places the idea stops, and the bridge again, asked the right question |

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
names, **eigenvector** (mathematics) and **mode** (engineering; used for the
bridge and the room); **eigenvalue**, and for a structure **stiffness**
(**soft**, **stiff**); **mix** (never "linear combination"; "superposition"
once, in a lower-third); **joint** and **member** (never "node", "bar",
"element"); **squeezed** and **stretched**; **tone** and **note** for a mode
as heard, **hertz** for swings a second; **round** and **uses up** for the
economy; **your turn** (every quiz). Never "simply", "just", "magic" or
"secret".

## 01_the_bridge_that_swayed (~58 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | London, the tenth of June, two thousand. A new footbridge opens across the Thames, and tens of thousands of people walk over it. | A slender deck across a river, seen from above, small dots walking along it; the date "10 June 2000"; caption "80,000 to 100,000 people on the first day" (`K-B2.first_day`) | particles on a path; lagged reveal | |
| 01_2 | And it begins to sway. Sideways, about seven centimetres, under nothing but footsteps. | The deck swings from side to side, drawn larger than life; a ruler beside it reads 70 mm (`K-B2.sway_mm`) | morph on a tracker; live number | |
| 01_3 | Two days later it was closed. Its designers had checked how it would bounce under walkers. The design codes had not asked how it would sway. | "closed 12 June" stamps across the deck (`K-B2.closed`); two small icons: up-and-down bounce with a tick, side-to-side sway with a question mark | highlight | |
| 01_4 | ⟨predict⟩ So what does a bridge have, besides strength, that footsteps can find? | The question as a card over the still bridge | prediction card | hold |
| 01_5 | A way of moving that is all its own. It can swing sideways about once a second, and a walker's weight shifts from side to side about once a second. | The deck swinging steadily, a counter beside it at "about 1 swing a second"; under it a walker seen from behind, weight shifting left, right, a counter at "about 1 a second" (`K-B2.lateral_hz`, `K-B2.walker_lateral_hz`) | two live counters in step | |
| 01_6 | Every structure has patterns like that. So does the air in a room, and so does an economy. One calculation finds them all. | Three small pictures side by side, each moving in one pattern: a truss sagging and springing back, the air in a box sloshing as a standing wave, two bars of an economy rising in step | lagged reveal; morph | |
| 01_7 | *(title)* | **The One Idea Behind Bridges, Concert Halls and Economies** | title card | listen 3 s (the motif: the overtones of one note) |

- **Cold open** (plan rule 20): a paradox, a bridge that was checked and
  swayed anyway, then the payoff previewed as three pictures, then the title.
  The honest failure of the obvious approach is the designers' own: the
  checks the codes asked for were done.
- **Misconception defused:** "a bridge that moves must be too weak". The
  engineers' paper calls it a loading effect "which had not been anticipated
  during design"; nothing in it is about strength.
- **Ruth:** only what the sources say (fact sheet, §Sources): the dates, the
  crowd, 70 mm, the codes "being updated to cover the phenomenon". "About
  once a second" covers the measured sideways modes at 0.80, 0.95 and
  1.03 Hz (the lowest, 0.48 Hz, is told in chapter 08). Nothing is said yet
  about why the sway grew; chapter 08 does that in words that stay true under
  both published accounts.
- **Maya:** no number she has to hold; the question is the hook.

## 02_two_carts (~76 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Start small. Two carts on a track between two walls, joined by three springs. Each spring pushes back with one newton for every centimetre it is stretched or squeezed. | Two carts between two walls, three springs drawn as zigzags; a ruler under the track; "1 newton per centimetre" beside a spring | **the carts**; grow; lagged reveal | |
| 02_2 | Slide the left cart one centimetre to the right, and hold the right cart still. | The left cart slides 1 cm (a blue arrow marks the move); the left spring stretches and thins, the middle one squeezes and thickens; two hands appear | morph on a tracker; **moves in blue** | |
| 02_3 | Two springs now fight the left cart, so that hand pushes with two newtons. The middle spring also shoves the right cart, so the other hand holds it back with one. | Amber arrows on the carts: 2 to the right on the left cart, 1 to the left on the right cart; the list "moves 1, 0" beside "pushes 2, −1" (`K-A1.push.left`) | **pushes in orange**; live labels | |
| 02_4 | You moved one cart, and had to push two. In every structure, each part is tied to its neighbours, so no part can be worked out alone. | The two lists side by side, the 0 in the moves and the −1 under it in the pushes ringed; behind the carts, faint, a truss and a grid of air cells, every piece joined to the next | highlight; lagged reveal | |
| 02_5 | Here is the rule for these carts, as a table. Each row turns the two moves into one push. A table like this is called a matrix. | The table 2, −1 over −1, 2 between the two lists; the top row lights as "2 × 1 − 1 × 0 = 2", then the bottom row as "−1 × 1 + 2 × 0 = −1"; the word "matrix" under it, small (`K-A1.table`) | table built from primitives; highlight | |
| 02_6 | ⟨predict⟩ Your turn. Pause if you want more time. Slide both carts one centimetre to the right. What are the two pushes? | The quiz card: the table, "moves 1, 1", "pushes ?, ?", the ring draining from 5, "pause for more time" (`K-Q.02_6.*`) | **the quiz card**; countdown ring on a tracker | listen 5 s (countdown) |
| 02_7 | One and one. The middle spring is not stretched at all. The pushes come back in the same pattern as the moves. | Both carts slide together, the middle spring unchanged; "2 × 1 − 1 × 1 = 1" twice; pushes 1, 1 land beside moves 1, 1 and the two lists pulse together in gold (`K-A1.push.together`) | morph; **gold for a pattern that comes back** | hold |

- **A situation before an abstraction:** springs you can count before any
  table; the table is read as two sums before it is called a matrix.
- **The honest failure in miniature** (02_4): the obvious way to think, one
  part at a time, fails on the smallest structure there is.
- **Misconception defused:** "only the cart I touch needs a push".
- **Maya:** she has never met a matrix. Here it is two sums she can do, and
  the word arrives after she has used it. The first quiz is two additions.
- **Ruth:** the rule is "the pushes that hold these moves", f = K u, stated
  in that direction every time; the inverse question comes in chapter 04.
- **Engine:** two rectangles moved by placement; three springs redrawn from
  the carts' positions (small `always_redraw` polylines).

## 03_own_patterns (~61 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | ⟨predict⟩ Now slide them one centimetre toward each other. What are the pushes? | The quiz card: the table, "moves 1, −1", "pushes ?, ?", the ring from 5 (`K-Q.03_1.*`) | the quiz card | listen 5 s (countdown) |
| 03_2 | Three and minus three. The middle spring is squeezed from both ends, so it counts double. The same pattern as the moves, three times the size. | The carts close in, the middle spring squeezed hard; pushes 3, −3 beside moves 1, −1; gold pulse (`K-A1.push.against`) | morph; highlight | |
| 03_3 | Draw the two moves as one arrow, and the two pushes as another. Swing the first arrow round, and the second runs ahead of it or falls behind. | A plane: left cart's number across, right cart's up. A blue arrow for the moves and an orange arrow for the pushes, from the same point; the blue one sweeps round a circle and the orange one is recomputed from it every frame; a readout shows the angle between them | **the arrow plane**; live recomputation from a moving object | |
| 03_4 | In two directions only, the arrows line up. Both carts together, and the carts against each other. | The sweep slows at 45° and at 135°: the arrows lie along each other and a gold line marks each direction; small cart pictures sit at the ends of the lines; readout "0°"; caption "everywhere else the push turns away, by up to 30°" (`K-A2.directions`, `K-A2.worst_turn_deg`) | highlight; live number | |
| 03_5 | These are the system's own patterns. Move it in one of them, and it pushes back in that same pattern. Only the size changes. | The two gold lines stay; on each, the blue arrow and its orange answer: the same length along "together", three times as long along "against" | morph | |
| 03_6 | An own pattern is called an eigenvector, and its size change an eigenvalue. Eigen is German for own. For a structure, engineers say mode. | The names land on the picture: "eigenvector" on a gold line, "eigenvalue" beside the 1 and the 3, "mode 1" and "mode 2" under the cart pictures; a small card "eigen (German): own"; lower-third: K q = λ q (`K-A2.eigenvalues`) | TeX equation; card; highlight | hold |

- **Show, then name** (plan rule 7): two quizzes and a sweep before the word
  eigenvector; "mode" is given at once as the engineer's name for the same
  thing, because the viewer will meet both.
- **One picture carries the idea:** the arrow that comes back along itself.
  It returns for the economy in chapter 12 and for the limits in chapter 15.
- **Misconception defused:** "every arrow is turned by a rule" and its
  opposite, "an eigenvector is an arrow that does not change" (it changes
  size; here by 1 and by 3).
- **Ruth:** an eigenvector is a direction: the gold marks are lines through
  the origin, not arrows, and the sweep meets each twice (45° and 225°,
  135° and 315°). The largest turn is exactly 30° (sin = (3 − 1)/(3 + 1)),
  computed by the sweep and pinned.
- **Dev:** the readout is the angle between u and K u, not the stretch
  ‖K u‖/‖u‖, which would peak along "against" and say nothing of direction.

## 04_the_split (~53 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | ⟨predict⟩ Why do engineers care? Take the question the other way round. Push the left cart with one newton, and leave the right cart free. Where do the two carts end up? | The carts at rest; one orange arrow of 1 on the left cart; "pushes 1, 0" and "moves ?, ?" | prediction card; highlight | hold |
| 04_2 | That push is a mix of the two patterns. Half of together, plus half of against. | In the arrow plane, the orange arrow 1, 0 splits along the two gold lines into two shorter arrows; beside it "1, 0 = ½ of 1, 1 + ½ of 1, −1" (`K-A3.mix`) | morph; TeX equation | |
| 04_3 | And each pattern answers by itself. Together has stiffness one, so its half stays a half. Against has stiffness three, so its half shrinks to a sixth. | Each gold line gets its own little division: "½ ÷ 1 = ½" and "½ ÷ 3 = ⅙"; the two orange pieces turn into blue ones, the second a third as long (`K-A3.shares`) | TeX derivation; morph | |
| 04_4 | Add the answers. The left cart moves two thirds of a centimetre, and the right cart, which nobody pushed, one third. | The two blue pieces add tip to tail into one arrow, 2/3, 1/3; above, the carts move to those places (`K-A3.moves`) | morph; live labels | |
| 04_5 | Two tangled equations have become two divisions. Seen in its own patterns, a system falls apart into pieces that ignore each other. | A cleared stage, the sentence as a card: "In its own patterns, a system falls apart into pieces that ignore each other."; lower-third: "names to search: eigendecomposition, superposition" | card | hold |

- **Earn the formula** (plan rule 6): the split is done once with halves and
  sixths; the letters wait for the truss (06_4).
- **Close the act with one sentence** (04_5, rule 14).
- **Misconception defused:** "to find where one cart goes you must solve
  for both at once". In the own patterns each piece is one division.
- **Ruth:** the mix uses the patterns (1, 1) and (1, −1) as drawn, not unit
  vectors: ½ and ½ are the coefficients, 1 and 3 the eigenvalues, ½ and ⅙
  the answer's coefficients; all exact fractions (`K-A3.*`), and the answer
  is checked against solving the two equations directly.
- **Dev:** this is diagonalization: change basis, divide, change back.

## 05_a_steel_truss (~76 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Now a real structure. A steel truss bridge, twenty-four metres long and four deep, with twelve joints and twenty-one members. | The truss draws itself joint by joint and member by member: a pin under the left end, a roller under the right; dimension lines 24 m and 4 m; counters reach 12 and 21 (`K-T1.span_m`, `.depth_m`, `.joints`, `.members`); small tag "a model" | **the truss**; lagged reveal; count-up | |
| 05_2 | Each member is a spring. This bottom one takes three hundred kilonewtons to stretch it one millimetre. | One bottom member glows and stretches a hair; beside it the numbers first: "200 GPa × 6,000 mm² ÷ 4 m = 300 kN per mm", then the letters under them, k = EA/L (`K-T1.e_gpa`, `.chord_mm2`, `.spring_kn_per_mm`) | TeX: numbers, then letters; highlight | |
| 05_3 | Its joints can make twenty-one separate moves, so the rule is a table again, twenty-one by twenty-one. Each member writes its spring into the rows and columns of its own two joints. | "12 joints × 2 directions − 3 held = 21" and 21 small blue arrows on the joints (`K-T1.free`); beside the truss a 21 by 21 grid of cells; members light one at a time and each fills its few cells, darker for stiffer; counters "441 entries" and "not zero" (`K-T2.entries`, `K-T2.nonzero`); a mirror line along the diagonal, tag "symmetric" (`K-T2.symmetric`); lower-third, for programmers: "for each member: K[its two joints] += k · g gᵀ, g = (−c, −s, c, s)" | **the stiffness table**; grid recoloured in place; lagged reveal | |
| 05_4 | ⟨predict⟩ Your turn. Hang a hundred kilonewtons, about ten tonnes, on one joint. How many of the twenty-one moves are not zero? | The quiz card: the truss, one orange arrow down on the second deck joint, "100 kN", the ring from 4 (`K-Q.05_4.*`, `K-T3.load_tonnes`) | the quiz card | listen 4 s (countdown) |
| 05_5 | All twenty-one. That joint drops four point seven millimetres, and the far support slides one point three. Everything is tied to everything. | The truss deforms, drawn 200 times larger than life (tag "moves × 200"); every joint's blue arrow grows; labels "4.7 mm" at the loaded joint and "1.3 mm" at the roller; counter "21 of 21 move" (`K-T3.moving`, `.drop_mm`, `.slide_mm`) | morph on a tracker; live labels | |
| 05_6 | Now load all five deck joints. A computer solves the twenty-one equations together, and the middle sags eighteen millimetres. | Five orange arrows of 100 kN; K u = f written over the grid; the truss sags smoothly; label "18.0 mm" at midspan; small: "span ÷ 1,336"; lower-third, for programmers: "for one load, real programs factor K and solve; they never compute its eigenvectors" (`K-T4.sag_mm`, `K-T4.span_over_sag`) | TeX; morph; live number; lower-third | |
| 05_7 | Each member's force follows from how far its ends moved apart. The top is squeezed hardest, four hundred fifty kilonewtons, and the middle post carries nothing at all. | Members take colour and weight: magenta and thick along the top (−450), teal along the bottom (up to +400), the middle post thin and grey (0); a legend "squeezed / stretched"; a bar gauge "75 of 250 N per mm²: steel's limit" with a green tick (`K-T5.top_kn`, `.bottom_kn`, `.middle_post_kn`, `.stress_mpa`, `K-T1.yield_mpa`) | **squeezed magenta, stretched teal**; lagged colouring; live bar | hold |

- **Tiny example, then scale up** (plan rule 5): two carts, then 21 moves.
  The table is built the same way, a spring at a time.
- **Numbers before letters** (05_2): the member's stiffness is computed with
  its numbers, and k = EA/L arrives under them.
- **The honest failure at full size** (05_4, 05_5): the tangle of chapter 02
  on a real structure, with real numbers.
- **Ruth:** every number here has two independent sources: the stiffness
  method in `model.rs`, and in the oracle joint equilibrium for the forces
  and the unit-load method for the moves, neither of which builds K. The
  middle post's zero is exact (its top joint carries no load and meets no
  other member with an upright part). The truss, its steel (E = 200 GPa,
  the US round figure) and its sections are a model, tagged so on screen.
- **Dev:** the first lower-third is the assembly loop of the direct
  stiffness method, and the grid shows why real stiffness tables are stored
  sparse. The second keeps the video honest about practice: statics is a
  factorization, not an eigendecomposition. Modes earn their keep in
  chapter 07.
- **Engine:** the grid is 441 fixed squares recoloured in place; the truss
  is 21 lines redrawn from the solved moves times a tracker.

## 06_twenty_one_patterns (~69 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | But this table has own patterns too. Twenty-one of them, each a way the whole truss can move, each with a stiffness of its own. | The truss shrinks to the left; on the right a ladder of 21 short bars, one per pattern, soft at the bottom, on a scale of stiffness; three patterns play in turn as small moving trusses (`K-P1.softest_mn_per_m`, `.stiffest_mn_per_m`) | bars on a data window; morph | |
| 06_2 | ⟨predict⟩ Your turn. Which is the softest? The whole span sagging as one, or neighbouring joints moving opposite ways? | The quiz card: two small trusses, one sagging in a single curve, one with joints alternating left and right along the deck, the ring from 5 (`K-Q.06_2.*`) | the quiz card | listen 5 s (countdown) |
| 06_3 | The single sag. It is the carts again. Moving together stretches little, and moving against stretches a lot. The stiffest pattern is nearly four hundred times stiffer. | The two patterns move side by side with their bars: 3.12 and 1,226 MN per m; "× 393" between them; the two cart pictures from chapter 03 beside them, small (`K-P1.softest_mn_per_m`, `.stiffest_mn_per_m`, `.ratio`) | morph; live numbers | |
| 06_4 | Now split the load into the patterns, as with the carts. Each share is divided by its stiffness, so the soft ones win. | The five orange arrows break into 21 small orange bars on the ladder (how much of each pattern the load holds); each is divided by its stiffness and turns blue; the top of the ladder all but vanishes; the letters arrive: u = Σ (qᵢ · f / λᵢ) qᵢ | **TransformMatchingTex**: the carts' halves and sixths → letters; bars | |
| 06_5 | The softest pattern alone gives ninety-nine point six percent of the sag at the middle. | The sagging truss is rebuilt from pattern 1 only, laid over the true shape: they all but coincide; readout "99.6% of the sag" (`K-P2.sag_percent.1`) | morph; live number | |
| 06_6 | The member forces are another matter. With that one pattern, the worst is off by eighty-two kilonewtons. With five, still by forty-four. | A second readout under the first: "worst member force off by 82 kN" with 1 pattern, "44 kN" with 5, "8 kN" with 15, "0" with all 21, as patterns are added one by one (`K-P2.force_error_kn.1`, `.5`, `.15`, `.21`) | count-up; live numbers | |
| 06_7 | That is why engineers prize modes. A structure with a million moves often behaves like a handful of them. | The ladder of 21 bars beside a ladder of a million, only its bottom few lit | lagged reveal | hold |

- **Continuity** (plan rule 13): "it is the carts again" (06_3) and the same
  split (06_4); the formula's letters are the carts' halves and sixths.
- **State costs plainly** (06_6, rule 16): the sag settles with one pattern
  and the member forces do not; both are computed.
- **Misconception defused:** "each added pattern brings the answer closer":
  the sag overshoots to 102% with two patterns before settling
  (`K-P2.sag_percent.2`), shown on the readout as patterns are added.
- **Ruth:** the patterns here are K's own eigenvectors, at right angles in
  the plain sense, eigenvalues in newtons a metre; the sum over all 21 is
  exact (`K-X4.rebuilt`). "Often" in 06_7 is an engineer's experience, not
  a theorem, and the beat before it shows where it fails.
- **Dev:** this is modal truncation. Displacements converge like 1/λ;
  stresses need the stiff modes or a static correction.

## 07_every_mode_a_note (~66 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Standing still is not where modes earn their keep. Motion is. Give every joint its mass and let go. Each own pattern swings by itself, at a frequency of its own. | Masses appear as discs on the joints, larger along the deck; the equation K φ = ω² M φ beside the truss; lower-third: "precisely: the eigenvalue is ω², and two modes are at right angles once mass is counted, φᵢᵀ M φⱼ = 0" | TeX equation; lagged reveal | |
| 07_2 | With a four tonne deck at every joint, the lowest mode swings five point nine times a second. Five point nine hertz. | Mode 1 swings, drawn large; a counter of swings and a clock; "5.9 Hz" (`K-V1.deck_kg`, `K-V1.hz.1`) | morph on a tracker; live counter | |
| 07_3 | The next two come at eleven point eight and fourteen point nine hertz. One slides the deck along its length, the other bends it into an S. | Modes 2 and 3 swing beside mode 1, each at its own true relative rate, with its number (`K-V1.hz.2`, `K-V1.hz.3`, `K-V1.shape.2`, `.3`) | three morphs at three rates | |
| 07_4 | ⟨predict⟩ Your turn. Make the deck twice as heavy. Does the lowest note go up, go down, or stay? | The quiz card: the truss with its deck discs doubling, three choices, the ring from 4 (`K-Q.07_4.*`) | the quiz card | listen 4 s (countdown) |
| 07_5 | Down, to four point three hertz. The same springs have more mass to move. | Mode 1 swinging beside its heavier twin, visibly slower; "5.9 Hz" and "4.3 Hz" (`K-V2.hz.1`) | two morphs; live numbers | |
| 07_6 | Any motion of the bridge is a mix of its modes, each keeping its own time. So an engineer finds the lowest few, and asks whether anything will push at that rhythm. | A messy shake of the truss resolves into the three clean modes stacked under it; lower-third: "names to search: modal analysis, the Lanczos algorithm; FrankenSim solves K φ = λ M φ in crates/fs-modal" | morph; lower-third | hold |

- **Why this chapter:** this is where engineers use the decomposition every
  day, and it sets up both bridges.
- **Misconception defused:** "a heavier bridge is a stiffer bridge".
- **Ruth:** the generalized problem is stated as it is (the lower-third of
  07_1); the model has no damping, and "is a mix" holds for the undamped
  linear model. Mode 2 is mostly along the deck because the right support
  is a roller (`K-V1.shape.2`).
- **Dev:** Lanczos by name, and the owner's own solver by path.
- **Sound:** a listen-window candidate for the score bead: the three modes
  as three tones in their true ratios, one at a time, then together.

## 08_two_bridges (~68 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Tacoma Narrows, the seventh of November, nineteen forty. A suspension bridge four months old twists itself apart in a wind of forty-two miles an hour. | A long deck seen end on, twisting about its centre line; the date; "opened 1 July 1940"; "wind 42 mph" (`K-B1.failed`, `K-B1.opened`, `K-B1.wind_mph`) | morph on a tracker | |
| 08_2 | For decades, textbooks called it resonance, the wind puffing in time with the bridge. But those puffs would come about once a second, and the bridge twisted once every five. | Two pulses side by side: "the wind's puffs: about 1 a second" and "the twist: 12 a minute", plainly out of step; lower-third: "Billah and Scanlan, American Journal of Physics, 1991" (`K-B1.vortex_hz`, `K-B1.twist_per_min`, `K-B1.twist_period_s`) | two live counters out of step | |
| 08_3 | What grew was one twisting mode. As the deck twisted, the wind's push changed with the twist and fed it. Engineers call that flutter. | The twisting deck with wind arrows whose size follows the angle of twist; the twist grows; the word "flutter"; a quote card, small: "well designed and built to resist safely all static forces" (the board of engineers, 1941) | morph; quote card | |
| 08_4 | ⟨predict⟩ So what was feeding London's footbridge? | The footbridge from chapter 01, still; the question as a card | prediction card | hold |
| 08_5 | It has sideways modes near half a swing a second, and near one. A walker pushes sideways once a second. And the more the deck moved, the harder walkers pushed to keep their balance. | A frequency line with four gold marks at 0.48, 0.80, 0.95 and 1.03 Hz and the walker's push at 1; the deck's sway and the walkers' arrows grow together (`K-B2.lateral_hz`, `K-B2.walker_lateral_hz`) | number line; morph | |
| 08_6 | In tests, one span stayed steady under a hundred fifty-six walkers. Ten more, and it began to swing. | A counter of walkers climbs to 156 with the deck still; +10, and the sway takes off (`K-B2.test_steady`, `K-B2.test_added`) | count-up; morph | |
| 08_7 | Neither bridge failed for lack of strength. Each had a mode, and something to feed it. | The two bridges side by side, each with its one mode drawn in gold | highlight | hold |

- **Honest examples of modes** (the order): both stories are told from their
  primary sources and say less than the legends do.
- **Misconceptions defused:** "Tacoma was resonance" (08_2), and "the
  walkers fell into step and that made the bridge sway" (08_5 says only that
  the push grew with the motion, which both the 2005 and the 2021 papers
  hold; neither "synchrony" nor "resonance" is spoken).
- **Ruth:** 12 a minute is the frequency observed in the final motion
  (Farquharson, through Billah and Scanlan); the board's report has 14 at
  its onset. Billah and Scanlan rule out vortex-shedding resonance and call
  it a self-excited flutter in one torsional degree of freedom; "the wind's
  push changed with the twist" is that, in plain words. "For decades": their
  paper surveys the textbooks in 1991. The board's sentence is quoted
  verbatim. The 156 and the 10 are Arup's own published test.
- **Dev:** a feedback loop with gain above one: the lower-third names to
  search are "aeroelastic flutter" and "synchronous lateral excitation".

## 09_the_air_in_a_room (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | The same idea, with nothing solid in it. Cut the air in a closed tube into thin slices. Each slice has mass, and squeezed air pushes back like a spring. | A long tube; its air cut into slices; one slice pushed, its neighbours crowd (magenta) on one side and thin out (teal) on the other and push it back | morph; **squeezed magenta, stretched teal** | |
| 09_2 | So a tube of air is a row of carts and springs. The same table as before, only longer. | The slices turn into a row of carts and springs; under them the table: 2 down the diagonal, −1 on each side, growing to 40 by 40 (`K-R1.slices`); lower-third: "FrankenSim tests its eigen-solver on this chain, crates/fs-modal/src/lib.rs" | morph that keeps identity; grid | |
| 09_3 | Its own patterns are standing waves. The air sloshes from end to end, still at the walls. One hump, then two, then three, and each has a tone of its own. | The first three patterns of the chain, one above another: slices swinging left and right, the pattern's outline a gold curve with one, two, three humps; small: "40 slices give the lowest tone to within 0.02%" (`K-R1.error_percent`) | three morphs; highlight | |
| 09_4 | Helmholtz called them Eigentöne, own tones, in eighteen sixty-three. That is forty-one years before David Hilbert wrote Eigenwert, eigenvalue. | A quote card: "…hat gewisse Eigentöne" (Helmholtz, 1863, on the air inside a violin), and under it "Eigenwerte" (Hilbert, 1904); "41 years" between them (`K-S.helmholtz_year`, `K-S.hilbert_year`, `K-S.eigen_gap_years`) | quote card | |
| 09_5 | ⟨predict⟩ Your turn. The lowest tone is the speed of sound divided by twice the length. Sound travels three hundred forty-three metres a second, and a bathroom is two and a half metres long. What is its lowest tone? | The quiz card: "lowest tone = speed of sound ÷ (2 × length)", a small room, "343 m a second", "2.5 m", the ring from 6 (`K-Q.09_5.*`) | the quiz card | listen 6 s (countdown) |
| 09_6 | Three hundred forty-three divided by five. Sixty-eight point six hertz, a low hum. Hum a slow slide in a tiled bathroom, and at certain notes the room hums back. | "343 ÷ 5 = 68.6 Hz"; a figure humming in a small room, a pitch line sliding up, the room's outline glowing gold as the line crosses 68.6 and 71.5 (`K-R2.first_hz`, `K-R2.second_hz`) | TeX; number line; glow | hold |

- **Continuity:** the air is the carts (09_2); the viewer already owns the
  table.
- **A "huh"** (09_4, rule 17): the word eigen was used for a room's tones
  before it was used for an equation. Only that is claimed: the same word,
  41 years apart, and nothing about one borrowing from the other.
- **Ruth:** slices that cannot move at the walls are the displacement form
  of the hard-wall problem; the pressure form has its humps at the walls
  (chapter 10). Hilbert's 1904 word is for integral equations, and the
  fact sheet says so.
- **Maya:** the quiz is one division, and she can test the answer at home.
- **Sound:** a listen-window candidate: 68.6 Hz, then 71.5 Hz, at true
  pitch.

## 10_two_million_modes (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | A room has three directions, so each mode counts humps along the length, the width and the height. For a box with hard walls, one formula gives every tone. | A box in outline; three counters nx, ny, nz step and the pressure pattern on its floor changes with them, strongest at the walls and corners; the formula with the words length, width, height under Lx, Ly, Lz; lower-third: "FrankenSim: rectangular_cavity_modes, crates/fs-couple/src/vibroacoustic.rs" | TeX equation; **heat map of a mode**, grid recoloured in place | |
| 10_2 | Now a concert hall. A box forty-five metres by twenty-three by eighteen, about the volume of Boston's Symphony Hall. Its lowest mode is three point eight hertz, far below hearing. | The box grows until the bathroom is a speck beside it; "18,630 m³ (a model); Boston Symphony Hall: 18,750 m³"; "3.8 Hz" (`K-H1.volume_m3`, `K-H1.boston_m3`, `K-H1.lowest_hz`) | morph; live number | |
| 10_3 | ⟨predict⟩ How many modes do you think it has below a thousand hertz? | The question as a card over the hall; a frequency line from 0 to 1,000 Hz, empty | prediction card | hold |
| 10_4 | Nearly two million. And already two thousand two hundred forty-six below a hundred hertz. | A histogram of modes per band rises steeply as a frequency marker slides right; a counter runs to 2,246 at 100 Hz and on to 1,964,040 at 1,000 Hz (`K-H2.count.100`, `K-H2.count.1000`) | histogram that grows; count-up | |
| 10_5 | Nobody will find two million modes one at a time, and nobody needs to. Sound in this hall takes about two seconds to die away, which blurs every mode a little. | A single mode as a peak on a frequency line, with a width; a decay curve beside it, "1.9 s" (`K-H1.t60_s`) | curve; live number | |
| 10_6 | Where modes crowd closer than their blur, they merge. The crossover is called the Schroeder frequency. For this hall it is twenty hertz, the bottom edge of hearing. | The frequency line fills with peaks left to right, sparse then dense, until they merge into a smooth wash; a marker "20 Hz" where they merge; "hearing begins about here" (`K-H3.schroeder_hz`) | curve that grows; highlight | hold |

- **Scale up honestly:** the same formula, a bathroom and a hall, and the
  count that changes the question.
- **Misconception defused:** "a concert hall's sound is its modes". Above
  20 Hz they cannot be told apart, and that is the whole audible range.
- **Ruth:** modes are counted exactly, in whole numbers (`rooms.rs`), and the
  count is compared with the textbook estimate (`K-H2.estimate.1000`). The
  model box has Boston's volume to within 1% (`K-H1.volume_percent`); the
  real hall is not a plain box, and the tag says "a model". The Schroeder
  frequency is 2000·√(T/V) (Schroeder and Kuttruff 1962); it is a
  crossover, not a wall, and modes still exist above it. "The bottom edge
  of hearing" is the conventional 20 Hz.
- **Dev:** the count grows as frequency cubed; that is the wall.

## 11_where_modes_rule (~68 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | So every note you can hear in a concert hall lies above the crossover. Halls are designed with statistics and rays of sound. Wallace Sabine worked that way on Boston's Symphony Hall, which opened in nineteen hundred. | Rays bouncing round the hall's outline; Sabine's rule with the hall's numbers: "1.9 s = 0.161 × 18,630 ÷ 1,579 m² of absorption" (`K-H4.sabine_m2`, `K-S.sabine.opened`); lower-third: "names to search: Sabine equation, ray tracing, statistical energy analysis" | particles on paths; TeX | |
| 11_2 | Single modes rule in small rooms. For a room five metres by four, the crossover climbs to two hundred hertz, and below it sixty-five modes carry all the bass. | The small room's frequency line: 65 separate peaks up to a marker at 200 Hz (`K-R4.schroeder_hz`, `K-R4.modes_below`) | curve; count-up | |
| 11_3 | ⟨predict⟩ Your turn. Which small room is worst for music? A long narrow one, an ordinary one, or a perfect cube? | The quiz card: three small rooms in outline, the ring from 5 (`K-Q.11_3.*`) | the quiz card | listen 5 s (countdown) |
| 11_4 | The cube. Its three sides match, so three modes share one note. In a three metre cube that is fifty-seven point two hertz, and it booms. | Three number lines of the rooms' lowest tones as tick marks: the cube's ticks land on top of each other in threes, "57.2 Hz × 3"; lower-third: "three of the most admired concert halls are shoeboxes: the trouble is matching sides, not boxes (Beranek)" (`K-R3.first_hz`, `K-R3.first_count`) | number lines; highlight | |
| 11_5 | And real rooms are not boxes, so there is no formula. You cut the air into cells, build the table, and ask for its eigenvectors. The same calculation as the bridge. | The side view of a hall with a lowered stage ceiling and raked seats, cut into 306 cells; its first three modes as heat maps with their frequencies; lower-third: "FrankenSim's solver is tested at 100,489 unknowns, crates/fs-modal/tests/modal_casebook.rs" (`K-N1.cells`, `K-N1.hz.1`, `.2`, `.3`) | **heat map on a shape that is not a box**; grid recoloured in place | hold |

- **State limits plainly** (plan rule 16): the hall act ends on where the
  eigen-idea stops being the tool, and where it comes back.
- **Misconception defused:** "a box is a bad room". Vienna's Musikverein,
  Amsterdam's Concertgebouw and Boston's Symphony Hall are boxes (Beranek);
  the lower-third of 11_4 says what the trouble really is.
- **Ruth:** Sabine's 0.161 is the modern metric constant (his own was
  0.164). He advised on the hall and did not design it; the script says
  "worked that way on". The cell method is checked first on the plain box,
  where the formula is known (`K-N1.box.error_percent`). "Worst" in the
  quiz is about the spread of the lowest modes, the standard criterion
  (Bolt 1946; Walker, BBC 1993).
- **Dev:** names to search in the lower-third of 11_5: "finite element
  method".

## 12_an_economy (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | The last system has no springs at all. An economy with two industries. Farms make food, and workshops make tools. | Two icons, a field and a workshop, with arrows from each to the other and to itself; tag "a model" | lagged reveal | |
| 12_2 | But making things uses things up. A unit of food takes four tenths of a unit of food, as seed, and two tenths of tools. A unit of tools takes three tenths of food and five tenths of tools. | The arrows take their numbers, then fold into a table: 0.4, 0.3 over 0.2, 0.5, the columns headed "to make 1 food" and "to make 1 tools" (`K-E1.table`) | table built from primitives; morph | |
| 12_3 | ⟨predict⟩ So here is another rule in a table. Your turn. Make ten units of each. How much of each is used up? | A blue list "made" goes into the table and an orange list "used up" comes out; then the quiz card: the table, "made 10, 10", "used up ?, ?", the ring from 6 (`K-Q.12_3.*`) | the quiz card | listen 6 s (countdown) |
| 12_4 | Seven of each. Four plus three, and two plus five. The same mix comes back, seven tenths the size. | "0.4 × 10 + 0.3 × 10 = 7" and "0.2 × 10 + 0.5 × 10 = 7"; the two lists pulse together in gold (`K-E1.uses_even`) | TeX; highlight | |
| 12_5 | So the even mix is an eigenvector of this table, with eigenvalue seven tenths. The economy uses up seventy percent of what it makes, and keeps thirty. | Two bars, food and tools, each split 70 to 30, the 30 in green (`K-E2.largest`, `K-E2.kept_percent`) | bars; highlight | |
| 12_6 | Swing the arrow as before. The only other direction that lines up needs a negative amount of tools. A table with no negative numbers always has an own pattern with none, and it carries the largest eigenvalue. | The arrow plane of chapter 03, now food across and tools up: the sweep, a gold line along the even mix, and a second line, greyed, running into the quarter where tools are negative; lower-third: "Perron 1907, Frobenius 1912; the same largest eigenvector ranks web pages (Brin and Page 1998)" (`K-E2.mix`, `K-E2.other`) | the arrow plane; live recomputation | hold |

- **The one picture returns** (12_6): the same sweep as the carts, on a
  table that is not symmetric.
- **Misconception defused:** "eigenvectors are about vibration". Here
  nothing moves; a rule is applied to a list.
- **Ruth:** the table is not symmetric and its two eigenvectors, (1, 1) and
  (3, −2), are not at right angles. The spoken sentence is Perron and
  Frobenius for a nonnegative matrix: its largest eigenvalue has an
  eigenvector with no negative entry (strictly positive, and unique up to
  scale, when the matrix is irreducible, as this one is). Checked on 1,000
  seeded positive tables (`K-X6.*`).
- **Maya:** the quiz is two small sums.

## 13_can_it_feed_itself (~69 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | People want twenty-four units of food and twenty-four of tools to live on. How much has to be made? | A household icon, "wants 24, 24"; an empty stack beside each industry | lagged reveal | |
| 13_2 | Making twenty-four of each uses up sixteen point eight of each. So make those as well, and that uses up seven tenths again. | Round by round, a block lands on each stack: 24, then 16.8, then 11.76, each seven tenths as tall as the one under it (`K-E3.rounds`) | bars that grow; count-up | |
| 13_3 | Every round is seven tenths of the one before, so the rounds add up to a finite total. Eighty of each. | The stacks close in on a dashed line at 80; "24 ÷ (1 − 0.7) = 80"; lower-third: "x = (I − C)⁻¹ d" (`K-E3.total`) | TeX; live number | |
| 13_4 | ⟨predict⟩ Your turn. A hungrier economy uses up one point one units for every unit it makes. What happens to the rounds? | The quiz card: a block and, beside it, a block 1.1 times as tall, a question mark for the next, the ring from 5 (`K-Q.13_4.*`) | the quiz card | listen 5 s (countdown) |
| 13_5 | They grow, and the total never arrives. That economy cannot feed itself, whatever it plans. | The hungry economy's stack: 24, 26.4, 29.04, climbing off the top of the frame, in red (`K-E4.rounds`) | bars that grow | |
| 13_6 | So the test is one number. With the largest eigenvalue below one, every demand can be met. At one or more, the economy cannot keep up. | A card: "largest eigenvalue < 1: every demand can be met", the 0.7 in green and the 1.1 in red on a line with 1 marked; lower-third: "precisely: for a table with no negative entries (the Hawkins–Simon condition, 1949)" (`K-X7.*`) | card; number line | |
| 13_7 | Wassily Leontief built such tables for the American economy, forty-four sectors wide, and won the Nobel prize in economics in nineteen seventy-three. | A 44 by 44 grid, faint, beside the 2 by 2; "Leontief, 1936 and 1941"; the prize citation, verbatim, small: "for the development of the input-output method and for its application to important economic problems" (`K-S.leontief.sectors`, `K-S.leontief.prize_year`) | grid; quote card | hold |

- **Make the surprise inevitable** (plan rule 19): the viewer adds the
  rounds by hand before the eigenvalue is named as the test.
- **Misconception defused:** "an economy that makes more of each thing than
  that thing's own making uses is fine": the test is the eigenvalue of the
  whole table, which counts what the inputs' inputs use up.
- **Ruth:** "every demand" is every demand with no negative entry, and
  "cannot keep up" is exact for this table, which has no zero entry (for a
  table that splits into separate parts, some demands may still be met; the
  lower-third and the fact sheet carry it). The equivalence is checked on a
  family of scaled tables against the inverse and against the
  Hawkins–Simon minors (`K-X7.*`). "Nobel prize in economics" is the common
  name; the prize's full name is in the description.
- **Dev:** the rounds are the Neumann series I + C + C² + …; its sum is the
  Leontief inverse.

## 14_a_different_tool (~45 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | Here is a question about scarce resources that looks alike, and is not. A factory makes bicycles and scooters from limited labour, aluminium and assembly time. A bicycle earns eight dollars, a scooter seven. | A small table: what a bicycle and a scooter each use of labour, aluminium and assembly, and what there is: 2, 1, 100; 1, 2, 80; 1, 1, 60; profits 8 and 7 (`K-L1.limits`, `K-L1.profit`) | table built from primitives | |
| 14_2 | ⟨predict⟩ The plans that break no limit fill a shape with four corners. Your turn. Which corner earns the most? | A plane, bicycles across and scooters up; three lines drawn one by one; the allowed region fills in green; four corner dots with their numbers; the point 0, 60 marked in red, "breaks the aluminium limit"; then the quiz card with the corners 50, 0 and 40, 20 and 0, 40, the ring from 6 (`K-L2.corners`, `K-L2.not_a_corner_allowed`, `K-Q.14_2.*`) | **plot**: lines, a filled region; the quiz card | listen 6 s (countdown) |
| 14_3 | Forty bicycles and twenty scooters, for four hundred sixty dollars. There is no own pattern here, and no eigenvalue. The best plan is found at a corner. That is linear programming, a different tool. | Each corner takes its profit: 400, 460, 280; a line of equal profit slides outward and last touches the region at 40, 20; a dot walks the edge from 0, 0 to 50, 0 to 40, 20; lower-third: "the simplex method, George Dantzig, 1947; FrankenSim's truss optimizer (crates/fs-truss) is a linear program too" (`K-L2.profits`, `K-L2.best`) | live numbers; a moving line; lower-third | hold |

- **Why this chapter:** the owner's third image is this problem. Its answer
  (40, 20) and 460 are right; it is not an eigenproblem, and the video says
  so rather than pretending (the order). The image's corner (0, 60) breaks
  the aluminium limit and is shown as what it is.
- **Misconception defused:** "any problem with a matrix in it is an
  eigenvalue problem".
- **Ruth:** "found at a corner": when a best plan exists and the region has
  a corner, some corner is best (Bertsimas and Tsitsiklis, in the fact
  sheet); here the best corner is also the best of all the allowed
  whole-number plans, by trying each (`K-X5.*`).
- **Dev:** the lower-third ties it to the owner's own code: `fs-truss` finds
  the lightest truss by a linear program, not by modes.

## 15_one_idea (~78 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | So why does one calculation serve a bridge, a room and an economy? Each is a rule in a table. And in its own patterns, a table does nothing but multiply. | The three tables of the video in a row; the first opens into three steps on a small grid of arrows: turn to the own patterns, stretch each by its eigenvalue, turn back; under it K = Q Λ Qᵀ | TeX equation; morph | |
| 15_2 | The bridge's table and the room's are symmetric, and a symmetric table always has a full set of own patterns, at right angles to one another. So any motion splits cleanly into modes. | The carts' two gold lines at right angles; then the truss's 21 patterns as a fan of small pictures; lower-third: "the spectral theorem" (`K-X1.*`) | highlight; lagged reveal | |
| 15_3 | The economy's table is not symmetric. What saves it is that nothing in it is negative. | The economy's two lines, not at right angles, one of them greyed; under it C = V Λ V⁻¹ | TeX; highlight | |
| 15_4 | ⟨predict⟩ And where does the idea stop? | The question as a card | prediction card | hold |
| 15_5 | Some tables have too few own patterns. A shear has one. A quarter turn has none you can draw. A hall has too many modes to tell apart. And steel bent too far stops answering in proportion. | Two arrow planes side by side: a shear, where the sweep finds one gold line, and a quarter turn, where the orange arrow stays at right angles all the way round (`K-X2.shear`, `K-X2.quarter_turn`); then the hall's merged wash of modes; then a push-against-move graph, straight at first and then bending over | the arrow plane; live recomputation; curve | |
| 15_6 | Where it holds, the same split gives powers of a table, and even its exponential. That is another video. | Cᵏ = V Λᵏ V⁻¹; a pointer card: "the matrix exponential: see the companion video" | TeX; card | |
| 15_7 | London's footbridge reopened in two thousand two, with dampers to calm its sideways modes. Its engineers had learned a second question. Not only how strong is it. How does it want to move? | The footbridge of chapter 01, people walking, the deck still; "reopened 22 February 2002; 37 viscous dampers and more than 50 tuned mass dampers" (`K-B2.reopened`, `K-B2.dampers_viscous`) | particles on a path; highlight | |
| 15_8 | *(receipts)* | "Every number in this video was computed by code and checked by a second program." with the repository's address | card | listen 3 s (the resolution) |

- **End on the rigorous form of the opening picture** (plan rule 15): the
  arrow that comes back along itself becomes K = QΛQᵀ, and the bridge of the
  cold open returns with the right question.
- **State limits plainly:** three places the idea stops, each shown.
- **Ruth:** "always has a full set at right angles" is the spectral theorem
  for real symmetric matrices, checked on 1,000 seeded symmetric tables
  (`K-X1.*`). The room's and the vibrating bridge's problem is the
  generalized one, where "at right angles" counts mass (07_1's lower-third).
  A shear has one eigen-direction and a quarter turn none in the real plane
  (its eigenvalues are complex; said in the description).
- **Dev:** powers and the exponential by the same split; Video J
  (matrix_exponential) takes it from there.
- **Receipts** (plan rule 26).

## Showcase

- [x] TeX derivations: the split with halves and sixths, then letters (04,
  06_4); k = EA/L from numbers (05_2).
- [x] value trackers driving live numbers: the angle between move and push
  (03); the sag and force readouts as patterns are added (06); mode counts
  (10).
- [x] live recomputation from a moving object: the orange arrow is K times
  the blue one, every frame (03, 12, 15).
- [x] morphs that keep identity: slices of air into carts (09_2); the
  truss's deformed shapes from the solved moves (05 to 07).
- [x] data windows: the ladder of stiffnesses (06), frequency lines (10,
  11), the factory's region (14).
- [x] tables and grids built from primitives, recoloured in place: the 21
  by 21 stiffness table (05), heat maps of room modes (10, 11).
- [x] sound that teaches, in listen windows: every countdown; candidates in
  07 and 09 for the score bead.
- [ ] the camera route: not used. Every picture here is flat, and a 3D room
  would cost a slower route for no idea the floor plan does not carry
  (plan §6.3).
