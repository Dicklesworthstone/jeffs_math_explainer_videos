# Storyboard — What Does It Mean to Raise e to a Matrix?

The hand-edited source of this video's words and pictures. `python3 -I
tools/storyboard.py matrix_exponential` checks it (gate G2) and writes
`script.tsv` from the words column. Facts: `docs/fact-sheet.md` and
`oracle.py` (every number below names its fact key); brief: `docs/brief.md`;
reviews and their dispositions: `docs/reviews/matrix_exponential/`.

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a question for the
viewer; the beat ends with the question and the picture holds before the
next beat answers it.

**The "your turn" device** (Video F's, kept the same across the corpus). A
quiz is a `⟨predict⟩` beat whose last column says `listen N s`: a violet
card with the problem, a ring that drains over N seconds with the seconds
left in its middle, the line "pause for more time" under it, the narration
silent and the music alone (the motif's rising steps, left unresolved until
the answer lands). The next beat works the answer in the same place on
screen. Eleven quizzes on a countdown; every one is a row of
`model::QUIZZES`, and its answer is the `F-Q.<line>.*` facts. Five further
prediction pauses hold for a moment without a ring.

**Lower-thirds** are tagged so a viewer knows which she may let go by:
"precisely" (the exact statement), "for programmers", "who and when". One
occupies the foot of the frame at a time.

## 2 The arc

One thread runs through all of it: **the exponential turns a rule for tiny
steps into where you end up.** A number in, growth out; i in, turning out; a
matrix in, the motion of a whole linked system out.

| Act | Chapters | Spine |
|---|---|---|
| I. The question | 01 e to a matrix, 02 the obvious guess | the question, answered at once (a rotation by one radian) with the dots already following their arrows; what a matrix is; the entry-by-entry guess fails on the matrix of zeros |
| II. What e to the x has meant all along | 03 growth, 04 the series, 05 one function, 06 turning | compound interest in tiny steps settles at e; the series earned piece by piece from "rate equals amount"; they are one function, and one dial of it handles every "rate is a share of the amount" problem; a rate of i pushes sideways: a circle, at speed one |
| III. Put a matrix in | 07 a matrix in, 08 does it settle | the series only needs multiplying, dividing by whole numbers and adding; the quarter turn J behaves like i, so the series for e^{tJ} is a spiral that lands on the circle; why the series settles for every square matrix |
| IV. What it means | 09 follow the arrows, 10 two tanks, 11 walks, 12 heat and chance | the matrix read as a rule for motion, and why the series is that motion (its rate is A times itself); salt between two tanks as a dot on the same plane; powers count walks and e^A adds them all; heat and chance by one matrix, on four nodes and on four hundred; one letter i and it never settles |
| V. Computing it | 13 how computers do it | the series is right and a poor algorithm; halve, sum a short series, square back; a ratio of polynomials; eigenvalues point to Video K |
| VI. The rule that breaks | 14 order matters, 15 the doorway, 16 where it leads | two rules at once is not one after the other unless they commute; a loop that does not close leaves the commutator; spins, their loop, a Lie algebra and its Lie group; robots (Video G) and the Standard Model's twelve force carriers; the opening card again, as the last quiz, answered in one sentence |

## 3 Color roles

One colour per idea for the whole video (plan §6.1). Role beats kind:
whatever sits in an exponent is teal, whatever the exponential returns is
gold. A highlight never uses a data colour.

| Idea | Color |
|---|---|
| the rule: a rate, a tiny step, the matrix in the exponent, its powers and terms, velocity arrows, spins | teal `#5CD0B3` |
| the result: e^{tA}, the running total, a path traced, where a point ends up, a rotation reached | `hero` gold `#FFD866` |
| the act of turning drawn as geometry: the fan of arrows, the dial of powers, an arc | blue `#54C4E5` |
| a substance in a container or on a node: money, salt, heat, chance | amber `#E8903B` |
| the second rule B, where order matters | rose `#E48BB5` |
| the guess that fails, a broken rule, digits lost | `warn` red `#FC6255` |
| a confirmed answer | `good` green `#83B943` |
| the "your turn" card and its countdown | violet `#9985E6` |

The seven hues are checked pairwise for contrast (ΔE2000 ≥ 20) by a test
before drafts, as Video C's customers and Video F's pieces were. (Blue, amber,
green and violet were nudged from `#58C4DD`, `#F0AC5F`, `#83C167` and
`#B189D6` when that test first ran and four pairs fell under 20; every pair,
ink and background included, is now at least 22: `docs/decisions.md`.)

## Vocabulary (one word per idea)

**matrix**: a grid of numbers that works as a machine (feed it an arrow, it
hands back an arrow). A matrix is used two ways, and the words keep them
apart: as a **move** (the arrow that comes out is where you land: the
quarter turn, a rotation, "the matrix that changes nothing") and as a
**rule** (the arrow that comes out is your velocity: where you are headed).
The exponential takes a rule and returns a move. **rate**: how fast an
amount is changing right now; **amount**: how much there is ("rate equals
amount"; never "size" for this); **share**: the fixed fraction r in "the
rate is a share of the amount". **tiny step**. **arrow** (never "vector"
aloud). **series**, **term** (introduced at 04_6; "piece" only before it);
**running total** and **settles** ("converges" is said once, at 08_1).
**size** of a matrix only: its biggest stretch. **exponential**, as a noun
for "e to the matrix", from 07_2 on. **quarter turn**, **half turn**;
**radian**, defined at 06_4 (an arc as long as the radius). **walk** (a
route along links that may double back), **node**, **link**. **spin**: a
skew matrix used as a rule (as in Video G); **rotation**: the move it leads
to. **commute**; **the commutator**; **Lie algebra** and **Lie group**, named
only after the spins and the rotations have been shown. **your turn** (every
quiz). Terms are numbered by k, and the screen shows k under the bars.
Letters said aloud: e, x, t, i, J, A, B, k, only where the screen shows
them. Never "simply", "just", "obviously", "it turns out", "magic".

## 4 Showcase (which feature carries which idea)

| Feature | Beats | The idea it carries |
|---|---|---|
| a dot field placed every frame at e^{tA} applied to each start, with arrows and trails | 01_2, 09_1 to 09_3, 10_2 to 10_6, 16_4 | the central picture: a rule for tiny steps, followed |
| live recomputation from a tracker (entries of e^{tA}, grams, heat, chance) | 01_2, 07_6, 10, 12 | the number on screen is the picture on screen |
| a row of TeX terms sliding one place onto itself | 04_6, 09_4 | the series is its own rate; with A, its rate is A times itself |
| a spiral of terms that swells and still lands on the circle; its steps stand up as bars | 07_5, 08_2, 08_4 | the series for e^{tJ}; big terms that cancel |
| a network of four hundred nodes recoloured from one tracker | 12_4 | what would be hopeless by hand |
| a small projected cube turned by the model | 15_1 to 15_3 | spins, and the loop that does not close |
| morphs that keep identity (network to tanks; machine to velocity field) | 12_5, 09_1 | the same object under a new reading |
| sound that teaches (for the score bead) | 08_4, 12_6, every countdown | the terms heard swelling and collapsing; two pitches that settle or trade places; the motif left unresolved until the answer |

No beat needs the camera route.

## 01_e_to_a_matrix

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | ⟨predict⟩ This is e, raised to the power of a matrix. A grid of four numbers, sitting where an exponent should be. What could that possibly equal? | e with the grid 0, −1; 1, 0 as its exponent, "= ?"; a small label under the e, "e = 2.718…"; behind it a plane of faint dots stirs | **the grid in an exponent**; dot field | |
| 01_2 | A rotation of the whole plane, by exactly one radian. By the end, you'll see why it couldn't be anything else. | The answer grid builds, 0.540, −0.841; 0.841, 0.540 (`F-H.J.true`), and shrinks to a corner as the plane takes the stage: every dot carries a small teal arrow at right angles to its position and travels a gold arc for one unit of time; an arc label "1 radian = 57.3°" (`F-V.01_2.degrees.shown`) | dot field placed from a tracker; trails | |
| 01_3 | *(title)* | **What Does It Mean to Raise e to a Matrix?** | title card | listen 3 s (the motif: one step, three times, each a little higher) |
| 01_4 | First, what is a matrix? A grid of numbers that works as a machine. Feed it an arrow, and it hands back a new arrow. Each row mixes the arrow's two numbers into one new number. | The grid 0, −1; 1, 0; the arrow (2, 1) goes in; row one gives 0 × 2 − 1 × 1 = −1, row two gives 1 × 2 + 0 × 1 = 2; the arrow (−1, 2) comes out (`F-M.machine.*`) | **the machine**: rows lighting in turn | |
| 01_5 | This one turns every arrow a quarter turn. We'll call it J. | A fan of eight arrows goes in and comes out turned a quarter turn, in blue; the grid takes the label "J: the quarter turn" | morph that keeps identity | hold |

- **Cold open** (plan rule 20): the question, then the answer with the
  central picture already running (the dots follow their arrows), then the
  title at about twenty seconds. The ring that this question deserves is at
  16_3, when the viewer can answer it.
- **Maya:** a matrix is given to her as a machine, with one arrow worked
  through its rows, before anything else is done with one.
- **Ruth:** e^J is the rotation by one radian (`F-H.J.true`).
- **Engine:** about 120 dots placed by one updater from one tracker; the
  grid is the shared grid prop (numbers and drawn brackets), which can sit
  in an exponent, tick and be pointed at.

## 02_the_obvious_guess

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | E is a number, about two point seven one eight. E squared is e times e. But e times itself, a matrix number of times? That means nothing. | e = 2.718…; e² = e · e; then e with the grid as its exponent = e · e · … · e with "how many times?" under the dots in red | TeX equation; highlight | |
| 02_2 | ⟨predict⟩ So try the obvious repair. Raise e to each entry. Test it on the easiest matrix of all, four zeros. E to the zero is one, so the guess gives four ones. Is that right? | Each entry of J gets its own little e (the guess, outlined in red: `F-H.J.guess`); then the matrix of zeros becomes four ones (`F-H.zero.guess`); the question as a card; for programmers: "this guess is np.exp(A); the real thing is scipy.linalg.expm(A)" | lagged reveal; prediction card | |
| 02_3 | No. E to the zero is one, and multiplying by one changes nothing. Its matrix twin has ones on the diagonal, zeros elsewhere, and leaves every arrow alone. | 1 × anything = the same thing; beside it the grid 1, 0; 0, 1 and a plane of dots that stays exactly where it is (`F-H.zero.true`), label "the matrix that changes nothing" | the machine; dot field | |
| 02_4 | Four ones squash the whole plane onto a single line. | The grid of four ones acts on the same dots: each lands at (x + y, x + y), on the diagonal line; the squash plays out, then a red cross lands on the guess | dot field placed from a tracker | |
| 02_5 | So the guess is dead. What has e to the x meant all along? | The guess fades; e^x stands alone, large, with a question mark | morph | hold |

- **The hook** (plan rule 2): the obvious repair fails on exact arithmetic,
  the simplest matrix there is; nothing is cherry-picked.
- **Why "should":** one is the number that changes nothing when you
  multiply; its matrix twin is the identity.
- **Misconception defused:** "a function of a matrix acts on each entry".
- **Ruth:** the guess also breaks "run twice equals run for twice as long"
  (`F-H.twice.*`); one failure is enough on screen.

## 03_growth

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Start with money. One dollar, at a hundred percent a year, becomes two. | An amber bar of height 1 grows to 2 in one step along a year-long axis (`F-N.compound.n1`) | bar on a tracker | |
| 03_2 | Split the year in two. Fifty percent, twice. The second step earns on the first, so you end with two twenty-five. | Two steps: 1, 1.5, 2.25; each step's gain a teal slice on top of the bar; beside it (1 + 1/2)² = 2.25 (`F-V.03_2.two_steps`) | staircase; TeX equation | |
| 03_3 | ⟨predict⟩ Your turn. Pause whenever you want longer. Four steps give two forty-four. Twelve give two sixty-one. Keep cutting the year finer. Does the total grow without limit? | The quiz card keeps all four totals in view: 2, 2.25, (1 + 1/4)⁴ = 2.44, (1 + 1/12)¹² = 2.61 (`F-V.03_3.*`); ring from 4 (`F-Q.03_3.*`) | **the quiz card**; countdown ring | listen 4 s (countdown) |
| 03_4 | No. It settles. A step every day gives two point seven one five. A million steps give two point seven one eight two eight. That number is e. | 365 steps: 2.715; 1,000,000 steps: 2.71828 (`F-V.03_4.*`); the staircase is now a smooth gold curve; who and when: "Jacob Bernoulli asked this in 1690" (`F-B.bernoulli.year`) | live number; morph | |
| 03_5 | Every step followed one rule. In a thousandth of a year, you gain a thousandth of what you have. So your gain per year, your rate, equals your amount. | On the curve a point slides, carrying a slope triangle whose run is exactly one year; its rise and the point's height are two verticals that stay the same length all the way along; on the main frame "f′ means: the rate of f"; precisely: "f′ = f, and f(0) = 1" | **steepness equals height**, two lines from one tracker | |
| 03_6 | Grow like that for a year, and a dollar becomes e. Two years, e times e. X years, e to the x. The one curve that starts at one, with rate equal to amount. | Marks on the curve at 1, 2 and x years: e, e · e, e^x; the curve takes its label; "rate = amount" in teal | highlight | hold |

- **Maya:** no calculus is assumed. "Rate" is a gain per year she can
  compute for a thousandth of a year, and then the steepness of a curve she
  can see. The quiz is hers: the jumps shrink (2, 2.25, 2.44, 2.61).
- **Why the curve is called e to the x:** 03_6, by years.
- **Ruth:** (1 + 1/n)^n is exact for each n shown. "The one curve" carries
  its start at one (without it, 2e^x and the zero line qualify too).
- **Later:** the bracket (1 + 1/n)^n on screen here is what 09_2 puts a
  matrix into.

## 04_the_series

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | There is a second way to reach that curve. Build it piece by piece, so its rate equals itself. | The target at the top, f′ = f; below it f(x) = with an empty row waiting | TeX equation | |
| 04_2 | Start with one, our first dollar. Its rate is zero, and we need one. So add x, which climbs at a rate of one. | The piece 1 as a flat line, "rate 0" in red; then + x, a line of steepness 1, and the mismatch clears | plots in small windows | |
| 04_3 | Now x needs a partner whose rate is x. That's half of x squared, the area under the line. An area grows as fast as its leading edge is tall. | The triangle under the line y = x, shaded teal, area x²/2; a thin strip at its right edge, as tall as the line there, slides outward as x grows | **area grows at the height of its edge** | |
| 04_4 | ⟨predict⟩ Your turn. Each new piece must have the one before it as its rate. Raise the power by one, and divide by the new power. What comes after half of x squared? | The quiz card: 1, x, x²/2, ?; the bottoms written 1, 1 × 2; a small arrow "its rate" from each piece back to the one before; ring from 5 (`F-Q.04_4.*`) | the quiz card | listen 5 s (countdown) |
| 04_5 | X cubed over six. Then x to the fourth over twenty-four. Each bottom number is the last one, times the next whole number. Those are the factorials. | x³/(2 × 3), then x⁴/(2 × 3 × 4); under the arrow back from x³/6, "3 · x²/6 = x²/2"; on the main frame 4! = 1 × 2 × 3 × 4 = 24 (`F-N.factorials`) | TeX equation; highlight | |
| 04_6 | An endless sum like this is called a series. Take its rate, and each term turns into the term before it. The whole row slides one place over, and nothing is lost. | First five terms only: take the rate and the last one has nothing to come from (a gap, in red); then the endless row e^x = 1 + x + x²/2! + x³/3! + …, which slides one place and lands on itself | **the row slides onto itself** (term-to-term pairs) | hold |

- **Earned, not stated** (plan rule 6): each piece is there to have the one
  before it as its rate, and the viewer derives the third.
- **Maya:** the one fact about rates she needs is the picture at 04_3.
- **Ruth:** the five-term sum fails and the endless one does not: that is
  the exact reason only the series is its own rate. Term-by-term rates are
  legitimate here (the series converges everywhere; chapter 08, and the
  description).
- **Thread:** this slide returns at 09_4 with a matrix in it.

## 05_one_function

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Try the series at x equals one. One, plus one, plus a half, plus a sixth. The running total climbs to two point seven one eight two eight. The bank's number. | The terms at x = 1; a running total ticks 2, 2.5, 2.667, 2.708 and on to 2.71828 at k = 9 (`F-V.05_1.running`, `F-N.series.k9`) beside the bank's 2.71828 | live number; count-up | |
| 05_2 | That's because both obey rate equals amount, from the same start, and only one curve can. | The staircase and the growing sum close in on one gold curve; precisely: "if g′ = g too, then g(x) · f(−x) never changes, so g = f" | morphs onto one curve | |
| 05_3 | Now loosen the rule. Let the rate be a fixed share of the amount, say five percent a year. The answer is the same curve, stretched. E to the zero point zero five t. | One curve with a dial marked r, set at 0.05; the rule y′ = 0.05 y above it, the result e^{0.05 t} beside the curve for a start of one; then the letters, with the start put back: y′ = r y, y = y(0) e^{rt}, "times whatever you start with" | **one curve on a dial**; TeX rule and result | |
| 05_4 | Turn the dial. Savings, bacteria, radioactive decay, a cooling cup of coffee. One function handles them all. | The dial sweeps: the curve steepens, flattens at r = 0 and flips to decay for negative r; one label swaps in place at four stops (savings, bacteria, decay, "coffee: its difference from the room"); precisely: "a differential equation: a rule for the rate" | live curve from a tracker | |
| 05_5 | ⟨predict⟩ Your turn. Five percent a year, growing all the time, for twenty years. How many times bigger? | The quiz card over the same picture: the dial at 0.05, a marker at 20 years; ring from 5 (`F-Q.05_5.*`) | the quiz card | listen 5 s (countdown) |
| 05_6 | Five percent times twenty is one. So it grows by e to the one. Two point seven one eight times. | 0.05 × 20 = 1; e¹ = 2.718 (`F-V.05_6.*`); the marker's height reads 2.718 | TeX equation; highlight | hold |

- **The owner's point**, in his words: the exponential "vastly simplifies
  things" and "solves so many differential equations". One curve on a dial
  is the whole family y′ = ry; numbers come first (five percent), then the
  letters.
- **Ruth:** the starting amount is kept on screen as y(0); "rate" is the
  gain per year and "share" is r, never the same word for both. Cooling is
  Newton's law for the difference from room temperature, and the label says
  so.
- **Dev:** "growing all the time" is continuous growth at rate 0.05, not 5%
  added once a year.

## 06_turning

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Now feed that function the number i. You may know it as the square root of minus one. Here, multiplying by i means one thing. Turn a quarter turn. | The number plane; the arrow to 1; times i turns it straight up; times i again and it points at −1; on the main frame i × i = −1, "two quarter turns" | **the quarter turn**, in blue | |
| 06_2 | So let your rate be i times your position. That's a push sideways, at right angles to where you stand, and as long as your distance from the centre. | A point at 1 with a teal velocity arrow straight up, the same length as the arrow from the centre; as the point is dragged about, the velocity arrow stays at right angles and matches its length | velocity arrow recomputed from the point | |
| 06_3 | ⟨predict⟩ Your turn. Follow that rule in tiny steps, each one a small push sideways. What path do you trace? | The quiz card over the plane: one step drawn, the rest to come; ring from 4 (`F-Q.06_3.*`) | the quiz card | listen 4 s (countdown) |
| 06_4 | A circle. Four big steps drift outward. A hundred hug it. Shrink the steps to nothing and you stay on it, moving at speed one. So after a time t, you've covered an arc of length t. That's an angle of t radians. | Four straight steps end at −2.32, 1.20; a hundred end at −1.05 (`F-V.06_4.*`); then the smooth circle, a gold point at arc length t; an arc as long as the radius is marked "1 radian"; who and when: "Euler, 1748: e^{v√−1} = cos v + √−1 sin v" | paths; live arc length | |
| 06_5 | That point is called e to the i t. Run it for a time pi, the length of a half circle, and you land on minus one. | The gold point takes the label e^{it} (the rule of 05_3 with i as the share); it travels the upper half circle, arc length π, and lands on −1 (`F-N.turn.pi`): e^{iπ} = −1 | TeX equation; highlight | hold |

- **Maya:** i is an action, a quarter turn; "i × i = −1" is two quarter
  turns she can see. A radian is defined on the picture: an arc as long as
  the radius. Half a circle of radius one is π long.
- **The reason for the cold open is planted here:** speed equals distance
  from the centre, so on the unit circle a time t is an arc of length t.
- **Honest about tiny steps:** four straight steps end well off the circle
  (fact sheet, finding 11).
- **Ruth:** Euler's credit sits on the general formula, in his notation
  (*Introductio*, §138), not on the case with π.
- **Notation:** the angle is the time t. No θ anywhere.

## 07_a_matrix_in

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Back to the matrix. The series asks three things of x. Multiply it by itself, divide by whole numbers, and add. A square matrix can do all three. | The series for e^x with three tags lighting in turn; under them three small demonstrations on the machine: A · A as "through the machine twice", A/2 as "the arrow that comes out, halved", A + B as "the two arrows that come out, added"; "square: as many rows as columns" | highlight; the machine | |
| 07_2 | So put a matrix where the x was. That is the definition of e to a matrix, also called its exponential. | Every x turns into A, and the leading 1 into the grid 1, 0; 0, 1, which then shrinks to the letter I: e^A = I + A + A²/2! + A³/3! + …; who and when: "Laguerre, 1867" (`F-B.laguerre.year`) | **x becomes A** (term-to-term pairs) | |
| 07_3 | ⟨predict⟩ Your turn. Try it on J, the quarter turn. The series needs J times J. A quarter turn, twice. What is J squared? | The quiz card: J² = ?, with the fan of arrows beside it; ring from 4 (`F-Q.07_3.*`) | the quiz card | listen 4 s (countdown) |
| 07_4 | A half turn. Every arrow reversed. That's minus the matrix that changes nothing. So J behaves exactly like i. | The fan turns twice and points backwards; J² = −I (`F-J.power.k2`); the powers J, −I, −J, I go round a dial (`F-J.power.*`, `F-J.period`) beside i, −1, −i, 1 | the machine; dial of powers | |
| 07_5 | Now run J for a time t. The terms alternate, one along the arrow, the next at right angles, and they spiral in to a point on the circle. | The terms of e^{tJ} applied to the arrow (1, 0), drawn head to tail at t = 1: 1 along, t up, t²/2 back, t³/6 down, …: a right-angle spiral that closes on the point at one radian | **the spiral of terms** | |
| 07_6 | Its across part is the cosine of t, its up part the sine. So e to the t J is the rotation by t radians. At t equals one, it's the matrix from the start. | The spiral's across moves add up to cos t and its up moves to sin t (`F-J.split.*` at t = 1); e^{tJ} = the grid cos t, −sin t; sin t, cos t; the dot plane turns as t runs to 1 and the entries read 0.540, −0.841, 0.841, 0.540 (`F-V.07_6.at_one`) | live entries from a tracker; dot field | hold |

- **The step the owner asked for:** "square matrices instead of real/complex
  numbers in the power series".
- **Maya:** the three operations are shown on the machine before the
  definition uses them; cosine and sine arrive as "across" and "up".
- **Numbers before letters:** the first exponential is worked on J, whose
  powers the viewer can do by turning arrows.
- **Ruth:** the definition is Laguerre's (1867); Peano's use of it (1888,
  first in Italian in 1887) is credited at 09_6. "Why may we add up endlessly?" is chapter 08's job.

## 08_does_it_settle

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | But an endless sum of matrices? For it to have an answer, the running total has to settle, in every entry. Mathematicians say the series converges. | The running-total grid for e^J with four live entries ticking as terms are added; they slow and stop on 0.540, −0.841, 0.841, 0.540 | live entries | |
| 08_2 | Take a worrying case. E to five J. It must land five radians round the circle, yet two of its terms are near twenty-six. | The spiral of 07_5 with t on a tracker, run from 1 to 5: it swells out past 21 (`F-V.08_2.swings_past`); its steps straighten and stand up as bars with k under them: 1, 5, 12.5, 20.8, 26.0, 26.0 (`F-V.08_2.bars`) | spiral on a tracker; **steps become bars** | |
| 08_3 | ⟨predict⟩ Your turn. Each new term multiplies by five again, and divides by the next whole number. What happens to the terms from here? | The bars so far, a live ratio "5 ÷ k" under the next gap; the quiz card, ring from 4 (`F-Q.08_3.*`) | the quiz card | listen 4 s (countdown) |
| 08_4 | They collapse. Once you're dividing by six, seven, eight, the factorial wins. By k equals twenty, about forty millionths. And the running total spirals home. | The ratio drops below 1 and the bars fall away; the last visible bar labelled with its value, k = 20: 0.0000392 (`F-S.term.k20`); an inset zooms fifteen times onto the unit circle, where the spiral lands at 0.284, −0.959 (`F-V.08_4.home`) | bars; inset zoom by group scaling | listen 4.8 s (the terms, a note each as loud as its bar) |
| 08_5 | It works for every square matrix. Call the most a matrix can stretch an arrow its size. The k-th term is at most the size to the k, over k factorial, and the factorial always wins. So what's still to come keeps shrinking, and the total settles. | A matrix turns the unit circle into an ellipse; its longest arrow is marked "size"; then outline bars size^k/k! as ceilings: 5J's bars fill theirs exactly ("this one touches its ceiling"), and a second matrix's bars sit visibly lower (`F-S.strict.*`); the ceilings add to e^{size} (for 5J: 148.4, `F-V.08_5.bound`); precisely: "the operator norm, ‖AB‖ ≤ ‖A‖ ‖B‖"; for programmers: "np.linalg.norm(A, 2); the proof: ‖A^k/k!‖ ≤ ‖A‖^k/k!, so the tail is smaller than the tail of e^{‖A‖}" | circle to ellipse; bars under ceilings | hold |

- **The owner's question:** "what it means for a matrix power series to
  converge", and the order's "show partial sums converging on a 2x2
  example". The example is chosen to worry: big terms that cancel.
- **Maya:** one short beat: the biggest stretch, and the factorial still
  wins. The norm's name and the proof are tagged strips.
- **Dev:** the proof sketch is the comparison test, said as "what is still
  to come keeps shrinking" (the tail), not only "sits underneath".
- **Ruth:** for 5J the bound is attained, and the picture says so; the
  second matrix shows it need not be. The claim is for real and complex
  square matrices of any size, in any submultiplicative norm.
- **For the score bead:** a listen window at 08_4 where each term is a note
  as loud as its bar.

## 09_follow_the_arrows

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | So e to a matrix exists. What does it mean? Read the matrix a new way. Feed it your position, and what comes back is your velocity. | The machine of 01_4 again, with J: for a dozen positions, each output arrow slides to sit on the tip of its input, shrinks, and turns teal: a field of velocity arrows; caption "what comes out is not where you land. It is where you are headed" | **the machine becomes a field** | |
| 09_2 | Take tiny steps along those arrows. Four. Sixteen. Shrink them to nothing, and each point traces a smooth path. | One dot: four straight steps, sixteen, then the smooth gold trail; small beside it, (I + tA/n)^n, the bank's bracket with a matrix in it | paths; TeX | |
| 09_3 | Follow the arrows for a time t, and each point is carried to its finish. The matrix that does the carrying is e to the t A. | All the dots flow for a time t with gold trails, the dot at the centre visibly still; then the rule's four entries slide on trackers from the turn to an inward spiral (the rule −0.3, −1; 1, −0.3: `F-M.spiral.*`) and the arrows and trails re-form; the entries of e^{tA} tick beside them | **follow the arrows**: dots placed at e^{tA} from a tracker; precomputed trails | hold |
| 09_4 | The series shows why. Take its rate. Every term slides one place over, and leaves one copy of A out front. So its rate is A, times itself. Velocity is the matrix applied to position. | The row e^{tA} = I + tA + t²A²/2! + …; "take the rate" slides every term one place, as at 04_6, and one A is left in front of the whole row; precisely: "x′ = Ax has the solution x(t) = e^{tA} x(0), and no other" | **the row slides, leaving A in front** | |
| 09_5 | ⟨predict⟩ Follow the arrows for a time s. Then for a time t more. What single matrix does the whole trip? | The flow run in two stages, s and then t, beside one run of s + t starting from the same dots | prediction card | |
| 09_6 | E to the A, run for s plus t. One rule, run twice, is one longer run. The matrix is the rule. Its exponential is where the rule takes you. | The two stagings end on the same dots; e^{tA} e^{sA} = e^{(s+t)A} with a green tick; then two cards, "A: the rule" in teal and "e^{tA}: where it takes you" in gold, an arrow between; who and when: "Peano, 1888: solving x′ = Ax this way" (`F-B.peano.year`) | highlight; card | silence before the last sentence, then hold |

- **The central picture** (brief): a rule for tiny steps, followed. It was
  running in the cold open, is built here from the machine, and returns in
  chapters 10 and 16.
- **The owner's question:** "what does it mean INTUITIVELY to take the
  exponential of it?"
- **Why the series is the flow** (the claim the title rests on): 09_4, the
  slide of 04_6 with one A left in front.
- **Two readings of a matrix, kept apart:** as a move at 01_4, as a rule
  here; e^{tA} is the move you get by following the rule A for a time t.
- **Ruth:** "each point is carried to its finish" (some do not move: the
  centre never does). e^{(s+t)A} = e^{sA}e^{tA} always holds because sA and
  tA commute: one rule, two durations. Chapter 14 is two rules.
- **Engine:** about 80 dots offset by half a step so none sits on the
  centre except one drawn on purpose; trails are precomputed polylines
  revealed in step with the dots.

## 10_two_tanks

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Now a real system. Two tanks, a hundred litres each, with pumps swapping ten litres a minute. Stir a hundred grams of salt into the left tank. | Left half: two tanks joined by two pipes, "10 L/min" each way; the left tank's water turns amber; readouts 100 g and 0 g (`F-T.start`) | **the tanks**; live readouts | |
| 10_2 | Salt leaves each tank at a rate of one tenth of what it holds, per minute, and arrives in the other. That is two equations, and their four numbers make one matrix. | The equations in words, "rate of left = −0.1 × left + 0.1 × right", then in letters; their four numbers lift into the grid (`F-T.rule`); right half: chapter 09's plane, "grams left" across and "grams right" up, a gold dot at (100, 0) and the rule's teal arrows, which all point along one diagonal toward the line of equal salt | TeX morph; the plane of 09 | |
| 10_3 | Where is the salt after ten minutes? Take one big step, and all hundred grams change tanks. That is absurd, because the right tank would have been sending salt back. | One step of ten minutes throws the dot along its arrow, clean through the balance line, to (0, 100) (`F-V.10_3.one_step`); the readouts jump; then the red mark | highlight | |
| 10_4 | Ten steps do better. A thousand, better still. It's the bank's trick again, and e to the ten A is where those steps are heading. Fifty-six point seven seven grams. | Ten steps as a teal polyline, then the gold trail stopping at 56.77; a small table: 10 steps, 55.37, off by 1.40 g; 1,000 steps, 56.75, off by 0.014 g; e^{10A}, 56.77 (`F-V.10_4.*`, `F-T.off_by.*`) | paths; table | |
| 10_5 | ⟨predict⟩ Your turn. Leave the pumps running for hours. Where does the salt end up? | The quiz card: the two tanks, "after a long time?"; ring from 4 (`F-Q.10_5.*`) | the quiz card | listen 4 s (countdown) |
| 10_6 | Fifty grams in each. And the exponential says how fast. Each tank is within a gram of fifty after about twenty minutes. | The dot comes to rest on the balance line, where the arrows have no length; the tanks level; a marker at 19.6 min, "within 1 g" (`F-V.10_6.*`); "total: 100 g, always" (`F-T.total`) | dot placed from a tracker | hold |

- **The order's "real example":** two coupled tanks; the system of linear
  equations and its matrix are on screen together (the owner: "each square
  matrix is a system of linear equations").
- **The central picture, with numbers on it:** the pair of readings is one
  dot on chapter 09's plane.
- **"Hopeless otherwise", first taste:** a thousand steps buy three digits
  (each ten times more steps buys one more); the exponential is exact at
  any time.
- **Ruth:** the rate is stated as a rate ("one tenth of what it holds, per
  minute"), not as "a tenth crosses every minute", which is the one-minute
  step the chapter refutes. The n-step numbers are exact
  (`50(1 + (1 − 2/n)^n)`). Each tank is assumed stirred.

## 11_walks

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | A matrix can also record who is linked to whom. Four nodes. Write a one where two nodes are linked, and a zero where they aren't. | Four nodes, a triangle 1, 2, 3 with a tail to 4 (`F-G.links`); the grid of ones and zeros fills in as each link lights; precisely: "the adjacency matrix" | **the network**; the grid | |
| 11_2 | A walk is a route along links, and it may double back. Multiply that matrix by itself, and each entry counts the two-step walks between a pair of nodes. | Row 1 and column 4 light; the four products under them, with the single 1 × 1 under node 3, "a middle node counts when both links exist"; the walk 1 to 3 to 4 travelled by a token (`F-G.walks.k2`) | grid; a token that travels | |
| 11_3 | ⟨predict⟩ Your turn. Start at node three. How many two-step walks bring you back to node three? | The quiz card: the network with node 3 lit; ring from 4 (`F-Q.11_3.*`) | the quiz card | listen 4 s (countdown) |
| 11_4 | Three. Out along any of its three links, and straight back. The matrix agrees. | A token goes out and back three times, each out leg and back leg as two shallow arcs either side of the link; the squared grid's entry for 3 and 3 lights: 3 | token; highlight | |
| 11_5 | The k-th power counts walks of k steps. So e to this matrix adds up every walk of every length, a walk of k steps divided by k factorial. There are infinitely many walks, and the exponential adds them all. | From node 1 to node 4, each walk drawn as a trace whose brightness is 1/k!: one of two steps, bright; one of three, dimmer; four of four, fainter (`F-G.walks.one_to_four`); what piles up at node 4 counts to 0.92 (`F-V.11_5.one_to_four`); for programmers: "1/k! is one choice of discount; it always settles, and it gives e^A" | traces weighted by 1/k!; count-up | |
| 11_6 | Network scientists call that number communicability. Nodes one and two are linked. So are one and three. But one and three score higher, because node three has more ways round. | e^A as a grid shaded by value (`F-G.exp`): 2.62 for nodes 1 and 3, 2.42 for nodes 1 and 2 (`F-V.11_6.*`); who and when: "Estrada and Hatano, 2008"; for programmers: "scipy.linalg.expm(A)[0, 2]" | shaded grid | hold |

- **The owner's question:** "and also the adjacency matrix of a graph".
- **Maya:** "walk" is defined, with doubling back, before the quiz needs
  it; the surprise (one and three beat one and two) is spoken.
- **Ruth:** walks, not paths (Stanley, Theorem 1.1); the oracle lists every
  walk up to length 8.

## 12_heat_and_chance

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Now let something flow. Put heat on node four. Each link carries heat from its hotter end to its cooler end, as fast as the difference between them. | Chapter 11's grid stays, and its empty diagonal fills with minus the number of links at each node as small arrows leave along them: the rule −L (`F-D.L`); node 4 lit amber, the others dark | **the grid gains its diagonal**; node discs | |
| 12_2 | That rule is a matrix, so the answer is its exponential. The heat spreads, and levels out. A quarter on each node. | The nodes' amber follows e^{−tL} as t runs to 5 (`F-D.heat.*`); four bars beside them level at 0.25 (`F-V.12_2.level`); "total: 1, always" | node discs and bars from a tracker | |
| 12_3 | Read the amber as heat, or as the chance of finding a random hopper. The matrix is the same. | The same bars, relabelled "chance"; a small hopper jumps along the links; precisely: "a hopper that leaves along each link at rate one (probability writes e^{tQ}); one that takes one hop per tick would favour the busy node, 3/8" (`F-D.chance.*`) | relabel in place | |
| 12_4 | On four nodes you could do this by hand. On four hundred, the exponential still answers in one stroke. | Pull back to a grid network of four hundred nodes (`F-D.big.*`); heat dropped on one node spreads as t runs, every node's amber one entry of a column of e^{−tL}; a counter runs up to the number of 40-step walks from one corner to the opposite one, 4,988,731,519,200 (`F-D.big.walks`) | **four hundred nodes recoloured from one tracker**; count-up | hold |
| 12_5 | ⟨predict⟩ The two tanks were this all along. Two nodes, one link, settling at half and half. Now put the letter i into the exponent. Does it still settle? | The network shrinks to two nodes that land on the two tanks, the link on the pipes (`F-D.pair.is_the_tanks`); a dot slides along a line toward zero as the two bars level (`F-D.pair.heat.*`); then an i appears in the exponent | morph that keeps identity; prediction card | |
| 12_6 | Never. Remember what i did to growth. It turned it into going round. So the chance sloshes across, and back, for ever. That is how a quantum state moves in time. | Left, e^{−tL}: the dot slides to zero, the bars level. Right, e^{−itL}: the same dot goes round a circle and the bars trade places, all across at π/2, all back at π (`F-D.pair.quantum.*`); precisely: "a two-state quantum system, ħ = 1: e^{−iHt} moves two complex numbers; each bar is the squared size of one" | bars from a tracker; contrast | listen 6.4 s (the same sweep again, heard: two pitches a panel, heat's level, the quantum pair's across and back) |

- **Three fields, one function:** heat (e^{−tL}), chance (e^{tQ}), quantum
  (e^{−iHt}), as the order lists them.
- **"Hopeless otherwise":** four hundred nodes, one evaluation; and a walk
  count no one could reach by hand.
- **Which hopper** is on screen where the claim is made: this one leaves
  along each link at rate one and levels at a quarter each; the one-hop-per-
  tick walk would favour the busy node (3/8). It is no longer a quiz.
- **Ruth:** L = D − A (Kondor and Lafferty's form). The quantum beat claims
  a two-state system with ħ = 1; the bars are squared sizes of complex
  amplitudes, and the strip says so.
- **Thread:** the i of chapter 06 turns settling into turning.

## 13_how_computers_do_it

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | How does a computer find e to a matrix? Surely it adds up the series. Try this one, from a paper by Moler and Van Loan. Nineteen Dubious Ways to Compute the Exponential of a Matrix. | The matrix −49, 24; −64, 31 (`F-C.mvl.A`); the paper's title as a card: Cleve Moler and Charles Van Loan, SIAM Review, 1978 (`F-B.mvl.year`, `F-B.mvl.ways`) | card | |
| 13_2 | ⟨predict⟩ Its terms swell past nine million, with alternating signs, like five J, only worse. Suppose the computer keeps about seven digits. Can anything after the decimal point survive? | Bars for the largest entry of each term, k under them, climbing to 9,303,002 at k = 16 and 17 (`F-C.mvl.term.*`); a labelled hairline far below for the true entries, shown as a grid of numbers: −0.736, 0.552, −1.472, 1.104 (`F-V.13_2.exact`); on one card "9,303,002" and "about 7 digits" | bars from data; prediction card | |
| 13_3 | No. Every entry comes out wrong, one with the wrong sign. The mathematics is fine. The digits that mattered were thrown away along the way. | Our 32-bit result, −0.789, 1.314, −0.126, −0.839 (`F-V.13_3.series`), in red beside the true one; for programmers: "this video's own run, 32-bit, term × A then ÷ k, stopped at k = 62; catastrophic cancellation. In 64-bit this matrix survives; doubled, it fails again" (`F-C.mvl.f64.*`) | contrast; highlight | |
| 13_4 | The cure is tiny steps again. Half the time, run twice, is the whole time. So halve the matrix eight times, until the series is short and tame. Then square the result, eight times. | e^A = (e^{A/2})², the rule of 09_6; A ÷ 256 (`F-C.mvl.f32.scaled.divisor`); a series through the eighth power; eight squarings back up; the same 32 bits now give −0.736, 0.552, −1.472, 1.104, right to three decimal places (`F-V.13_4.*`), green; "scaling and squaring"; for programmers: "not too many halvings: twenty leave two digits" (`F-C.mvl.f32.halvings.*`) | **halve, then square back**: lagged chain | |
| 13_5 | Real programs do exactly that, with a ratio of two polynomials, called a Padé approximant, for the short series. And when a matrix untangles into separate rates, that's the eigenvalue video. | At x = 1: the series through x⁶ gives 2.71806; a ratio of two cubics gives 193/71 = 2.71831, eight times closer (`F-V.13_5.*`); who and when: "MATLAB and SciPy: Higham 2005; Al-Mohy and Higham 2009"; then the tanks' two rates, 0 and −0.2 a minute, and a pointer card to the eigendecomposition video, in its letters: e^{tA} = V e^{tΛ} V⁻¹ (`F-C.diag.*`) | TeX equation; pointer card | hold |

- **The order:** "diagonalize when possible (one beat, point to K),
  otherwise scaling and squaring with Padé approximants; cite Moler and Van
  Loan".
- **Honest about whose wrong answer:** the red numbers are this video's own
  32-bit run, with its order of operations on the strip (other orders give
  other wrong answers). Moler and Van Loan's printed run is quoted in the
  description only.
- **Dev:** the strip says what he needs: 32-bit, the loop, cancellation,
  and that 64-bit only postpones it. The cure's reason is on screen:
  e^A = (e^{A/2})². More halvings are not better.
- **Ruth:** the paper names no best method; the script says what programs
  use, which is verified. Diagonalizing fails for matrices such as the
  shear of chapter 14 and is unreliable when the directions nearly line up
  (description).
- **Thread:** "tiny steps" a third time.

## 14_order_matters

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | For numbers, growing at two shares at once is the same as growing at one after the other. E to the sum is e to the first, times e to the second. | One amber bar grows through a stretch at share a ("the first") and then a stretch at share b ("the second"); beside it a bar grown under a + b together; they end level; e^{a+b} = e^a · e^b with a green tick | two runs against one | |
| 14_2 | Now two matrices, A and B, both shears. One pushes the top of a square to the right. The other pushes its right side upward. | A unit square; the rule A in teal, its arrows growing with height; the rule B in rose, its arrows growing to the right; their grids (`F-O.A`, `F-O.B`) | **the square, sheared**, driven by the model | |
| 14_3 | ⟨predict⟩ Your turn. Follow A, then B. Or B, then A. Does the square end up in the same place? | The quiz card over the two squares, each about to move; ring from 4 (`F-Q.14_3.*`) | the quiz card | listen 4 s (countdown) |
| 14_4 | No. They're mirror images. And both rules at once, a tiny step of each, over and over, lands between them. Three different results. | Three parallelograms form and slide onto one origin: A then B (the grid 1, 1; 1, 2), B then A (2, 1; 1, 1), and both at once (1.543, 1.175; 1.175, 1.543) between them (`F-O.expB_expA`, `F-O.expA_expB`, `F-V.14_4.both_at_once`); one grid at a time under the shape being named | three shapes driven by the model | silence before the last sentence |
| 14_5 | The rule broke because order matters. Multiply the two grids in either order, A B, and B A. When those agree, the matrices commute, and for those, the rule holds. | AB and BA as two different grids (`F-O.AB`, `F-O.BA`); "commute: AB = BA"; precisely: "e^{t(A+B)} = e^{tA} e^{tB} for every time t exactly when AB = BA. One pair can still obey it without commuting: 3 full turns about x, 4 about y, 5 about the 3-4-5 axis" (`F-O.turns.*`) | highlight; lower-third | |
| 14_6 | The failure can be measured. Go round a loop. First A, then B, then undo each. The square doesn't come back, and what's left over is set by A B minus B A. That is the commutator. | The square taken round the four moves for a time 0.5 each: it returns squeezed one way and stretched the other (`F-O.loop.*`); the leftover drawn as a new teal rule, a field of stretch-and-squeeze arrows; [A, B] = AB − BA = the grid 1, 0; 0, −1 (`F-O.commutator`), "leftover ≈ −t² [A, B] for short times"; who and when: "the Baker–Campbell–Hausdorff formula"; for programmers: "update A's effect, then B's, and this is your error per step" | the loop that does not close | hold |

- **The order:** "e^{A+B} = e^A e^B only when A and B commute; the
  commutator measures the failure (the first term of
  Baker–Campbell–Hausdorff)". **Corrected:** the script never says "only
  when". 14_5 says "for those, the rule holds", and the strip carries the
  exact statement and the 3-4-5 exception.
- **The rule that breaks is the right rule:** two shares at once against
  one after the other (14_1), which is what two matrices at once against
  one after the other fails. One rule run for two durations (09_6) is a
  different statement and never fails.
- **Convention on screen:** "A, then B" means A's motion first, so the
  matrix is e^B e^A (the later motion is written on the left). Each grid
  sits under the shape it makes. The loop's leftover is computed in the
  order it is played, and the sign on the strip is the one that comes out.
- **"Both at once"** is defined by the video's own idea: a tiny step of
  each, over and over (`F-O.alternate.*`).
- **Sibling:** Video G keeps the book quiz for rotations; this chapter uses
  shears.

## 15_the_doorway

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | That leftover matters most for rotations. Take the quarter turn J. Flip it across its diagonal, and every sign changes. Use such a matrix as a rule, and call it a spin. Follow a spin, and you get a rotation. Three dimensions have three, one for each pair of axes. | J; its entries swap across the diagonal and change sign (precisely: "skew: A^T = −A"); then one 3 by 3 skew grid at a time, lit while a small cube with one coloured face turns about that axis under e^{t·spin} (`F-L.exp.Lz.t1`); the three pairs of axes, xy, yz, zx, each a spin turning one axis toward the other; precisely: "every rotation comes from a spin; a full turn and no turn are the same rotation. Four axes would make six pairs" (`F-L.spins.*`, `F-L.full_turn_is_identity`) | projected cube turned by the model; grids | |
| 15_2 | ⟨predict⟩ Your turn. Tip the cube a little about x. Then about y. Then undo each one. Are you back where you began? | The quiz card: the cube and the four moves listed; ring from 4 (`F-Q.15_2.*`) | the quiz card | listen 4 s (countdown) |
| 15_3 | No. Repeat that loop, and the cube creeps round the third axis. The commutator of two spins is a third spin. | Seen from along z: the loop of four tips about the fixed x and y axes, twenty times; the cube creeps round by 45.7°, backwards about z (`F-L.creep.*`), a live readout of the leftover angle, a ghost marking the start; then the bracket of the x-spin and the y-spin = the z-spin (`F-L.bracket.*`), with the note "this loop runs it backwards" (`F-L.loop.is_minus_z_spin`) | cube turned by the model; ghost; live angle | |
| 15_4 | Add two spins, or take their commutator, and you still have a spin. A set closed like that is a Lie algebra. Follow one rotation by another, or undo one, and you still have a rotation. That is a Lie group. The exponential takes each spin to a rotation, and reaches every rotation. | Two panels. Left, "spins: add them, bracket them" with the three grids, labelled "a Lie algebra, so(3)". Right, "rotations: follow one by another, undo one" with the cube, labelled "a Lie group, SO(3)". A gold arrow marked exp from left to right | two panels; lagged reveal | hold |

- **Show, then name:** "spin", "Lie algebra" and "Lie group" are said only
  after the thing is on screen; the property that makes each one is in the
  sentence that names it.
- **Rule and move, again:** a spin is a rule, a rotation is a move.
- **The loop is pinned:** tips are about the fixed x and y axes, played in
  the order spoken; the facts multiply in that order (later motion on the
  left), and the creep's direction is whatever comes out. Twenty loops of
  0.2 radian tips, because one small loop leaves too little to see.
- **Ruth:** exp maps the spins onto the rotations (compact and connected),
  not one-to-one; both halves are on the strip. "Spins don't commute" is
  said of three dimensions only (in four, spins in separate planes do).
- **Sibling:** this is the formal version of Video G's last picture; G was
  told this loop is on a cube and theirs is an order swap on a book.

## 16_where_it_leads

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 16_1 | Robots use this. A robot arm's pose is a product of exponentials, one per joint. That's the companion video. | An arm of three joints on three trackers, each tag lighting as its joint turns, written Video G's way, e^{[S₁]q₁} e^{[S₂]q₂} e^{[S₃]q₃} M; a pointer card; who and when: "Sophus Lie, 1842–1899"; a strip of names: "also: spacecraft attitude, computer graphics, quantum gates" | pointer card; lagged reveal | |
| 16_2 | In particle physics, the forces of the Standard Model are built on three Lie groups, with eight, three, and one independent tiny steps. That makes twelve, and there are twelve particles that carry those forces. | SU(3) × SU(2) × U(1) with 8, 3 and 1 under them (`F-V.16_2.steps`); twelve tiles: 8 gluons under SU(3); one brace over "SU(2) × U(1): 3 + 1 = 4" holding W⁺, W⁻, Z and the photon (`F-B.bosons`); precisely: "the fine print is in the description" | grid of tiles; lagged reveal | |
| 16_3 | ⟨predict⟩ One last time, and now it's your turn. E, raised to the quarter turn J. What does it equal, and why? | The opening card returns unanswered, now as a quiz card; ring from 5 (`F-Q.16_3.*`) | the quiz card | listen 5 s (countdown) |
| 16_4 | A rotation by one radian. The rule J moves every point at a speed equal to its distance from the centre. So in one unit of time, each point travels an arc as long as its radius. One radian. | A teal ruler of tiny steps with a tick at 1 wraps onto the gold circle; the tick lands at an arc of length one and the dot plane turns under it; labels "a line of tiny steps: the Lie algebra" and "a circle of rotations: the Lie group"; the ruler keeps wrapping past a full turn | **the line wraps onto the circle**; dot field | |
| 16_5 | The matrix is a rule for tiny steps. E to the matrix is where that rule takes you. Its rate is the rule, applied to itself. | A strip of four small live panes, each a rule in teal and its result in gold: a share, growth; i, turning; a matrix, flow; a spin, rotation; then, after the hold, the receipts card, "every number in this video was computed by code; every dot was placed by e^{tA}", the repository link and names to search: matrix exponential, Baker–Campbell–Hausdorff, Lie algebra | recap strip; receipts card | hold |

- **Honestly scoped** (the order): one line, one picture. Twelve tiny steps
  are the dimensions of the three Lie algebras; twelve carriers as the
  Particle Data Group lists them, with the brace showing that the W's, the
  Z and the photon come from the three and the one together. The
  description carries the rest (a possible quotient by a subgroup of Z₆;
  nothing about gravity).
- **Hand-off:** 16_1 points to Video G in G's notation, with the home pose M.
- **The ending makes the opening inevitable** (plan rules 15 and 19): the
  opening card is the last quiz, and its answer is one sentence that the
  viewer has owned since 06_4. The picture under it is the exact bridge for
  this example: a line of tiny steps (multiples of J) wrapped onto a circle
  of rotations, many times over.
- **The last line is true of everything in the recap:** the rate of e^{tA}
  is A applied to it.
- **Receipts card** and names to search (plan rule 26).
