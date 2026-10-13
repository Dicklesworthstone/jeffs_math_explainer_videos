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
silent and the music alone (one soft note a second). The next beat works the
answer in the same place on screen. The owner asked for quizzes with pauses
before the answers, so eleven of the sixteen prediction pauses are quizzes
on a countdown. Every quiz is a row of `model::QUIZZES`, and its answer is
the `F-Q.<line>.*` facts.

## 2 The arc

One thread runs through all of it: **the exponential turns a rule for tiny
steps into where you end up.** A number in, growth out; i in, turning out; a
matrix in, the motion of a whole linked system out.

| Act | Chapters | Spine |
|---|---|---|
| I. The question | 01 e to a matrix, 02 the obvious guess | a quiz before anything is taught, answered at once (a rotation); what a matrix is; the entry-by-entry guess fails on the matrix of zeros |
| II. What e to the x has meant all along | 03 growth, 04 the series, 05 one function, 06 turning | compound interest in tiny steps settles at e; the series earned piece by piece from "its rate equals its size"; they are one function, and it solves every "rate proportional to amount" equation; a rate of i pushes sideways: a circle |
| III. Put a matrix in | 07 a matrix in, 08 does it settle | the series only needs adding, multiplying and dividing by whole numbers; the quarter turn J behaves like i, so e^{tJ} is a rotation; why the series settles for every square matrix |
| IV. What it means | 09 follow the arrows, 10 two tanks, 11 walks, 12 heat and chance | the matrix is a rule for motion and its exponential is the result; salt between two tanks; powers count walks and e^A adds them all; heat and chance spread by the same matrix; one letter i and it never settles |
| V. Computing it | 13 how computers do it | the series is right and a poor algorithm; halve, sum a short series, square back; a ratio of polynomials; eigenvalues point to Video K |
| VI. The rule that breaks | 14 order matters, 15 the doorway, 16 where it leads | e^{A+B} = e^A e^B needs A and B to commute; the leftover is the commutator; skew matrices, spins, a Lie algebra and its Lie group; robots (Video G) and the Standard Model's twelve force carriers; back to the opening card |

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses
a data colour.

| Idea | Color |
|---|---|
| the rule: a rate, a tiny step, the matrix in the exponent, its powers, velocity arrows | teal `#5CD0B3` |
| the result: e^{tA}, the running total, a path traced, where a point ends up | `hero` gold `#FFD866` |
| turning: i, the quarter turn J, skew matrices, rotations | blue `#58C4DD` |
| the stuff that flows: money, salt, heat, chance | amber `#F0AC5F` |
| the second rule B, where order matters | rose `#E48BB5` |
| the guess that fails, a broken rule, digits lost | `warn` red `#FC6255` |
| a confirmed answer | `good` green `#83C167` |
| the "your turn" card and its countdown | violet `#B189D6` |

The seven hues are checked pairwise for contrast (ΔE2000 ≥ 20) by a test
before drafts, as Video C's customers and Video F's pieces were.

## Vocabulary (one word per idea)

**rate** (how fast something is changing right now; never "derivative"
aloud); **tiny step**; **the rule** (the matrix, read as what says where
each point is headed) and **the result** (where it ends up); **arrow** (never
"vector" aloud); **matrix** (a grid of numbers that works as a rule: feed it
an arrow, it hands back a new arrow); **running total** and **settles** (the
word "converges" is said once, at 08_1, where it is defined); **size** of a
matrix (the most it can stretch any arrow); **quarter turn**, **half turn**;
**walk**, **node**, **link**; **spin** (a steady turning about one axis, as
in Video G); **order matters**, then **commute**; **the commutator**; **Lie
algebra** and **Lie group**, named only after the spins and the rotations
have been shown; **your turn** (every quiz after the cold open). Letters said
aloud: e, x, t, i, J, A, B, k only where the screen shows them. Never
"simply", "just", "obviously", "it turns out", "magic".

## 01_e_to_a_matrix (~50 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | ⟨predict⟩ This is e, raised to the power of a matrix. A grid of four numbers, sitting where an exponent should be. What could that possibly equal? | The quiz card, cold: e with the grid 0, −1; 1, 0 as its exponent, "= ?", the ring draining from 5, "pause for more time" (`F-Q.01_1.*`) | **the quiz card**; countdown ring on a tracker | listen 5 s (the countdown: the music alone, one soft note a second) |
| 01_2 | It equals this. A rotation of the whole plane, by exactly one radian. That's about fifty-seven degrees. By the end, you'll see why it couldn't be anything else. | The answer grid builds where the ring was: 0.540, −0.841; 0.841, 0.540 (`F-H.J.true`); behind it a plane of faint dots turns through an arc labelled "1 radian ≈ 57.3°" (`F-N.turn.radian_degrees`) | TeX equation; a dot field moved by placement | |
| 01_3 | *(title)* | **What Does It Mean to Raise e to a Matrix?** | title card | listen 3 s (the motif: one step, three times, each a little higher) |
| 01_4 | First, what is a matrix? A grid of numbers that works as a rule. Feed it an arrow, and it hands back a new arrow. | A grid of four numbers on the left; an arrow goes in, and a new arrow comes out; then a fan of eight arrows, all carried to their new places at once | **the rule machine**; morph that keeps identity | |
| 01_5 | This particular grid turns every arrow a quarter turn. We'll call it J, and it comes back later. | The fan turns a quarter turn, in blue; the grid takes the label "J: the quarter turn" | highlight | hold |

- **Cold open** (plan rule 20): a puzzle the viewer cannot answer, answered
  in three short sentences with the plane turning, then the title, then the
  honest failure of the obvious guess (chapter 02).
- **Maya:** a matrix is given to her in two sentences and one picture, as a
  rule that moves arrows, before anything is done with one. "Radian" comes
  with its size in degrees.
- **Ruth:** e^J is the rotation by one radian (`F-H.J.true`; the oracle's
  exact series against cos 1 and sin 1).
- **Engine:** the dot field is about 120 dots moved by placement from a
  tracker; arrows are fixed mobjects rotated in place.

## 02_the_obvious_guess (~55 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | E squared is e times e. But e, multiplied by itself a matrix number of times? That means nothing. | e² = e · e; then e with the grid as its exponent = e · e · … · e, with "how many times?" under the dots in red | TeX equation; highlight | |
| 02_2 | So try the obvious repair. Raise e to each entry, one at a time. | Each entry of J gets its own little e: the guess 1, 0.368; 2.718, 1 (`F-H.J.guess`), outlined in red as "the guess" | lagged reveal | |
| 02_3 | ⟨predict⟩ Test the guess on the easiest matrix of all, four zeros. E to the zero is one, so the guess gives four ones. Is that right? | The matrix of zeros; the guess turns it into four ones (`F-H.zero.guess`); the question as a card | prediction card | hold |
| 02_4 | No. E to the zero should change nothing. And the matrix that changes nothing has ones on its diagonal, and zeros everywhere else. | The identity matrix beside a plane of dots that stays exactly where it is (`F-H.zero.true`); label "changes nothing" | the rule machine | |
| 02_5 | Four ones do something violent. They squash the whole plane onto a single line. | The matrix of four ones acts on the same plane of dots: every dot slides onto the diagonal line; a red cross on the guess; small "determinant 0" (`F-H.zero.guess.det`) | dot field moved by placement; highlight | |
| 02_6 | So the guess is dead. To find the real meaning, we have to ask what e to the x has meant all along. | The guess fades; e^x stands alone, large, with a question mark | morph | hold |

- **The hook** (plan rule 2): the obvious repair fails on exact arithmetic,
  the simplest matrix there is; nothing is cherry-picked.
- **Misconception defused:** "a function of a matrix acts on each entry".
- **Ruth:** the guess also breaks "run twice equals run for twice as long"
  (`F-H.twice.*`: its square has 2 on the diagonal where the guess for 2J
  has 1); one failure is enough on screen, the second is in the fact sheet.
- **Dev:** this is the difference between `np.exp(A)` and
  `scipy.linalg.expm(A)`; a "for programmers" lower-third says so at 02_2.

## 03_growth (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Start with money. One dollar, in a bank that pays a hundred percent a year. After one year, two dollars. | An amber bar of height 1 grows to 2 in one step along a year-long axis; "100% a year" (`F-N.compound.n1`) | bar on a tracker | |
| 03_2 | Now split the year in two. Fifty percent, twice. The second step earns on the first, so you end with two twenty-five. | Two steps: 1, then 1.5, then 2.25 (`F-N.compound.n2`); each step's gain drawn as a teal slice on top of the bar | staircase; lagged reveal | |
| 03_3 | ⟨predict⟩ Your turn, and pause the video whenever you want longer. Four steps give two forty-four. Twelve give two sixty-one. Keep cutting the year finer. Does the total grow without limit? | The staircase with 4 steps, then 12; totals 2.44 and 2.61 (`F-N.compound.n4`, `.n12`); the quiz card, ring from 5 (`F-Q.03_3.*`) | the quiz card; staircase | listen 5 s (countdown) |
| 03_4 | No. It settles. A step every day gives two point seven one five. A million steps give two point seven one eight two eight. That number is called e. | n = 365: 2.715; n = 1,000,000: 2.71828 (`F-N.compound.n365`, `.n1000000`); the staircase is now a smooth curve; "e = 2.71828…" (`F-N.e`); lower-third "Jacob Bernoulli asked this question in 1690" (`F-B.bernoulli.year`) | live number; morph | |
| 03_5 | Every step followed one rule. Grow by a share of what you have. Shrink the steps to nothing, and the rule becomes this. Your rate of growth equals your size. | On the smooth curve a point slides, carrying a slope triangle; two live readouts, "height" and "steepness", stay equal all the way along; lower-third "precisely: f′ = f, with f(0) = 1" | **the rule, made smooth**; live readouts recomputed from the curve | |
| 03_6 | That's the first face of e to the x. The one curve whose steepness, everywhere, equals its height. | The curve takes its label e^x; under it "rate = size" in teal | highlight | hold |

- **Maya:** no calculus is assumed. "Rate" is the steepness of a curve she
  can see, and it arrives as the limit of steps she has just counted.
- **Misconception defused:** "compounding more often grows without limit"
  (03_3). It is the first quiz, and the first place she is told she may
  pause.
- **Ruth:** (1 + 1/n)^n is exact for each n shown (fractions in the oracle;
  a million compoundings in 60-digit decimals). The limit exists and is e;
  the picture does not prove it, and the script does not say it does.
- **"Huh":** Bernoulli's question is from 1690 (the often-quoted 1683 has
  no document behind it; fact sheet, finding 4).

## 04_the_series (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Here's a second way to reach that curve. Build it piece by piece, so that its rate equals itself. | The target written small at the top, "rate of f = f"; below it f(x) = with an empty row waiting | TeX equation | |
| 04_2 | Start with one. Its rate is zero, and we need one. So add x, which climbs at a rate of one. | The piece 1 as a flat line, "rate 0" in red; then + x, the line with steepness 1, and the mismatch clears | plots in small windows; highlight | |
| 04_3 | Now x needs a partner whose rate is x. That's half of x squared, the area under the line. An area grows as fast as its leading edge is tall. | The triangle under the line y = x, shaded teal, its area labelled x²/2; a thin strip at its right edge, as tall as the line there, slides outward as x grows | **area grows at the height of its edge**; live shading | |
| 04_4 | ⟨predict⟩ To match that one, add x cubed over six. Every new piece supplies the rate of the piece before it. So what comes after x cubed over six? | 1 + x + x²/2 + x³/6 + ?; a small arrow labelled "rate of" from each piece back to the one before; the quiz card, ring from 5 (`F-Q.04_4.*`) | the quiz card; lagged reveal | listen 5 s (countdown) |
| 04_5 | X to the fourth, over twenty-four. Each bottom number is the one before, times the next whole number. One, two, six, twenty-four. The factorials. | x⁴/24 lands; under the bottoms, 2 × 3 = 6 and 6 × 4 = 24 (`F-N.factorials`); lower-third "k! = 1 × 2 × … × k" | TeX equation; highlight | |
| 04_6 | Carry on for ever, and the sum is its own rate. Each piece turns into the piece before it, so the whole row slides one place over, and nothing is lost. | The row e^x = 1 + x + x²/2! + x³/3! + …; "take the rate" slides every piece one place to the left and the row lands on itself | **TransformMatchingTex**: the row onto itself | hold |

- **Earned, not stated** (plan rule 6): the series is built to satisfy the
  rule of chapter 03, one piece at a time, and the viewer predicts a piece.
- **Maya:** the only fact about rates she needs is in the picture at 04_3:
  an area grows as fast as its leading edge is tall.
- **Ruth:** term-by-term differentiation of a power series is legitimate
  inside its radius of convergence, which here is infinite; the description
  says so. The picture is the fundamental theorem of calculus, not a proof
  of convergence (that is chapter 08).

## 05_one_function (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Try it at x equals one. One, plus one, plus a half, plus a sixth. The running total climbs to two point seven one eight. The same number the bank gave us. | The pieces at x = 1; a running total ticks 2, 2.5, 2.667, 2.708, 2.717, 2.718 (`F-N.series.k1` to `.k6`); beside it the bank's 2.71828 from chapter 03 | live number; count-up | |
| 05_2 | That's no coincidence. The tiny steps and the series obey the same rule, rate equals size, from the same start. Only one curve can. | The staircase of chapter 03 and the growing sum of pieces both close in on one curve; lower-third "precisely: if two functions both have rate = size and start at 1, their ratio never changes"; lower-third "Euler, 1748: the series and the limit" (`F-B.euler.year`) | morphs onto one curve | |
| 05_3 | And this is why e to the x simplifies so much. Whenever a rate is proportional to an amount, the answer is e, to the rate times the time. | The rule y′ = r y on the left; its result y(t) = y(0) e^{rt} on the right; lower-third "a differential equation: a rule for the rate" | TeX derivation: rule → result | |
| 05_4 | Savings, bacteria, radioactive decay, a cooling cup of coffee. One function handles them all. | Four small plots appear in turn, each a stretched or flipped copy of the same curve, labelled with its sign of r: growing, growing, decaying, decaying | lagged reveal; plots | |
| 05_5 | ⟨predict⟩ Your turn. Something grows at five percent a year, continuously, for twenty years. How many times bigger does it get? | The quiz card: "5% a year, for 20 years", ring from 6 (`F-Q.05_5.*`) | the quiz card | listen 6 s (countdown) |
| 05_6 | Five percent times twenty is one. So it grows by e to the one. Two point seven one eight times. | 0.05 × 20 = 1; e¹ = 2.718 (`F-N.growth.*`) | TeX equation; highlight | |
| 05_7 | Last, grow for a while, then a while longer, and the factors multiply. E to the a plus b equals e to the a, times e to the b. Remember that rule. It's going to break. | A time axis cut into a stretch a and a stretch b; e^{a+b} = e^a · e^b in a gold box, which stays in a corner | highlight; box | hold |

- **The owner's point**, in his words: the exponential "vastly simplifies
  things" and "solves so many differential equations". 05_3 is the whole
  family y′ = ry, numbers first at 05_5.
- **Ruth:** uniqueness is on screen with its one-line reason. y′ = ry has
  the solutions y(0)e^{rt} and no others. "Cooling" is Newton's law for the
  difference from room temperature; the plot's label says "difference from
  the room".
- **Dev:** continuous growth at 5% a year means rate 0.05 per year, not 5%
  compounded once a year; the card says "continuously".
- **Thread:** 05_7's boxed rule is the one that breaks at 14_2.

## 06_turning (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Now feed that function something strange. The number i. Forget square roots for a moment. Multiplying by i means one thing. Turn a quarter turn. | The number plane; the arrow to 1; times i turns it to point straight up; times i again, and it points at −1; lower-third "so i × i = −1" | **the quarter turn**, in blue | |
| 06_2 | So make your rate i times your position. That's a push sideways. Always at right angles to where you stand. | A point at 1 with a teal velocity arrow pointing straight up; as the point moves, the arrow stays at right angles to its position | velocity arrow recomputed from the point | |
| 06_3 | ⟨predict⟩ Follow that rule in tiny steps, each one a small push sideways. What path do you trace? | Four straight steps, then sixteen, each at right angles to the arrow from the centre (`F-N.turn.steps.n4`, `.n16`); the question as a card | prediction card; path growing | hold |
| 06_4 | A circle. Four big steps drift outward. A hundred hug the circle. In the limit you stay on it exactly, and after a time t, you have turned through an angle t. | The four-step path ends far outside; the hundred-step path ends at −1.05 (`F-N.turn.steps.n100`); then the smooth circle, a gold point at angle t, labelled e^{it} | paths; live angle | |
| 06_5 | Run it for a time pi, half a turn, and you land on minus one. E to the i pi equals minus one. Growth, pushed sideways, is rotation. | The gold point travels the upper half circle and lands on −1 (`F-N.turn.pi`); e^{iπ} = −1; lower-third "Euler, 1748" | TeX equation; highlight | hold |

- **Maya:** i is given to her as an action, a quarter turn, which is all the
  video ever uses. "i × i = −1" is then something she can see: two quarter
  turns.
- **The picture is honest about tiny steps:** four straight steps end at
  −2.32 + 1.20i, well off the circle; the overshoot shrinks as the steps do
  (fact sheet, finding 11).
- **Ruth:** Euler's own formula reads e^{v√−1} = cos v + √−1 sin v
  (*Introductio*, §138); the lower-third names him and the year, and the
  description gives his notation.
- **Notation:** the angle is the time t. No θ anywhere (the engine draws it
  as ϑ; Video G uses t as well).

## 07_a_matrix_in (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Back to the matrix. Look at what the series asks of x. Multiply it by itself. Divide by whole numbers. Add. | The series for e^x; three tags light in turn over it: "multiply by itself", "divide by a whole number", "add" | highlight | |
| 07_2 | A square matrix can do all three. So put a matrix where the x was. That is the definition of e to a matrix. | Every x turns into A, and the leading 1 into I, "the matrix that changes nothing": e^A = I + A + A²/2! + A³/3! + …; lower-third "Laguerre, 1867; Peano, 1888" (`F-B.laguerre.year`, `F-B.peano.year`) | **TransformMatchingTex**: x → A | |
| 07_3 | ⟨predict⟩ Your turn. Try it on J, the quarter turn. The series needs J times J, which means, do a quarter turn, twice. What is J squared? | The quiz card: J² = ?, with the little fan of arrows beside it, ring from 5 (`F-Q.07_3.*`) | the quiz card | listen 5 s (countdown) |
| 07_4 | A half turn. Every arrow reversed. That's minus the matrix that changes nothing. So J behaves exactly like i. | The fan turns twice and points backwards; J² = −I (`F-J.power.k2`); the powers J, −I, −J, I go round a small dial (`F-J.power.*`, `F-J.period`) beside i, −1, −i, 1 | the rule machine; dial | |
| 07_5 | So the series sorts itself into two piles. The even powers are plain numbers. The odd powers are numbers times J. | The terms of e^{tJ} drop into two columns: I × (1 − t²/2! + t⁴/4! − …) and J × (t − t³/3! + …) | lagged regrouping of TeX pieces | |
| 07_6 | Those two piles are the cosine and the sine. So e to the t J is the rotation by the angle t. And at t equals one, it's the matrix from the very start. | The piles become cos t and sin t (at t = 1: `F-J.split.even`, `.odd`); e^{tJ} = the grid cos t, −sin t; sin t, cos t; the dot plane turns as t runs to 1 and the entries read 0.540, −0.841, 0.841, 0.540 | live entries from a tracker; dot field | hold |

- **The step the owner asked for:** "directly introducing how that leads to
  the matrix exponential function using square matrices instead of
  real/complex numbers in the power series".
- **Numbers before letters:** the first matrix exponential is worked on J,
  whose powers the viewer can do by turning arrows.
- **Ruth:** the definition is Laguerre's (1867), who already remarks that
  e^X e^Y = e^{X+Y} fails in general; Peano (1888) is the one who solves
  x′ = Rx with it. "Why may we do this?" is chapter 08's job, and 07_2 does
  not claim the series converges.
- **Maya:** cosine and sine appear as the two coordinates of a point on the
  circle from chapter 06, drawn beside the grid.

## 08_does_it_settle (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | But an endless sum of matrices? What would it mean for that to have an answer? That the running total settles, in every entry. Mathematicians say the series converges. | A running-total matrix for e^J with four live entries ticking as terms are added; they slow and stop on 0.540, −0.841, 0.841, 0.540 | live entries; count-up | |
| 08_2 | Here's a worrying case. E to five times J. Its terms get big. Two of them have entries near twenty-six. | A = 5J; bars for the size of each term rise: 1, 5, 12.5, 20.8, 26.0, 26.0 (`F-S.terms`, `F-S.peak`); beside them the running total's arrow swings out past 21 (`F-S.total.k0` to `.k5`, `F-S.farthest.*`) | bars growing from data; path | |
| 08_3 | ⟨predict⟩ Your turn. Each new term multiplies by five again, and divides by the next whole number. What happens to the terms from here? | The bars so far, "× 5, then ÷ k" over the next gap; the quiz card, ring from 5 (`F-Q.08_3.*`) | the quiz card | listen 5 s (countdown) |
| 08_4 | They collapse. Once you're dividing by six, seven, eight, the factorial wins. By the twentieth term, about forty millionths. And the running total spirals home, to a point on the circle. | The bars fall away to 0.0000392 at k = 20 (`F-S.term.k20`); the path of running totals spirals in to (0.284, −0.959), on the unit circle (`F-S.total.*`, `F-S.limit`) | bars; spiral path | |
| 08_5 | And that works for every square matrix. Measure a matrix by its size, the most it can stretch any arrow. Multiply two matrices, and their sizes at most multiply. | A matrix turns the unit circle into an ellipse; its longest arrow is the size; a second matrix stretches again; lower-third "the operator norm: ‖AB‖ ≤ ‖A‖ ‖B‖"; small example, a shear of size 1.618 whose square has size 2.414, under 2.618 (`F-S.shear.*`) | circle to ellipse; highlight | |
| 08_6 | So the k-th term is never bigger than the size to the k, over k factorial. Those numbers add up to e to the size, which is finite. The matrix series sits underneath, so it settles. Every time. | The term bars sit under taller outline bars size^k/k!; the outlines add up to e^{size} (for 5J: 148.4, `F-S.bound`); "for programmers: ‖A^k/k!‖ ≤ ‖A‖^k/k!, and those sum to e^{‖A‖}"; small "checked on 200 random matrices" (`F-X.series_settles`) | bars under bars; lower-third | hold |

- **The owner's question:** "what it means for a matrix power series to
  converge", and the order's "show partial sums converging on a 2x2
  example". The example is chosen to worry: big terms that cancel.
- **Dev:** the proof sketch is the "for programmers" lower-third; it is the
  comparison test.
- **Ruth:** "settles in every entry" is convergence in any norm (all norms
  on matrices are equivalent); the bound holds for every submultiplicative
  norm; the claim is for real and complex square matrices of any size. For
  5J the bound is attained (the size of the k-th term is exactly 5^k/k!),
  and the shear shows it need not be.
- **Thread:** big terms that must cancel are the villain of chapter 13.

## 09_follow_the_arrows (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | So e to a matrix exists. But what does it mean? Go back to tiny steps. | (1 + x/n)^n, and under it (I + A/n)^n, the same shape with a matrix in it | TeX equation | |
| 09_2 | Read the matrix as a rule for motion. Wherever you are, apply the matrix to your position, and that's your velocity. Where you're headed, right now. | A plane of dots; each grows a small teal arrow, the matrix applied to its position (first J: arrows at right angles; then a second rule whose arrows spiral inward) | **follow the arrows**: velocity field recomputed from positions | |
| 09_3 | Now follow the arrows for a time t. Every point lands somewhere new. The matrix that carries each start to its finish is e to the t A. | The dots flow along their arrows for a time t, leaving gold trails; faint rings mark where they began; a gold arrow from one start to its finish, labelled e^{tA} | dots moved by placement from a tracker; trails | |
| 09_4 | ⟨predict⟩ Follow them for a time s. Then for a time t more. What single matrix does the whole trip? | The flow run in two stages, s and then t, with a pause between; the question as a card | prediction card | hold |
| 09_5 | E to the s plus t, times A. Run, and run again, is one longer run. The rule of exponents, kept. | e^{tA} e^{sA} = e^{(s+t)A}; lower-third "precisely: x′ = Ax has the solution x(t) = e^{tA} x(0)"; small "checked on 200 random cases" (`F-X.flow_adds`) | TeX equation; lower-third | |
| 09_6 | The matrix is the rule for tiny steps. Its exponential is the result. | Two cards side by side: "A: the rule" in teal, "e^{tA}: the result" in gold, an arrow from one to the other | card | hold |

- **The central picture** (brief): a rule for tiny steps, followed. It
  returns in chapters 10, 12 and 15.
- **The owner's question:** "what does it mean INTUITIVELY to take the
  exponential of it?" Answer: follow the rule for a time t.
- **Ruth:** e^{(s+t)A} = e^{sA}e^{tA} always holds, because sA and tA
  commute. That is why this chapter's rule survives and chapter 14's does
  not.
- **Engine:** about 80 dots placed each frame at e^{tA} applied to their
  start, with t on a tracker; the arrows are fixed mobjects rotated and
  scaled in place.

## 10_two_tanks (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | A real one. Two tanks, a hundred litres each, with pumps swapping ten litres a minute. Stir a hundred grams of salt into the left tank. | Two tanks joined by two pipes with arrows, "10 L/min" each way; the left tank's water turns amber; readouts 100 g and 0 g (`F-T.start`) | **the tanks**; live readouts | |
| 10_2 | Every minute, a tenth of each tank's salt crosses over. Two equations, one for each tank. Together they are one matrix. | The two rate equations, x₁′ = −0.1 x₁ + 0.1 x₂ and x₂′ = 0.1 x₁ − 0.1 x₂; their four numbers lift out into the grid A (`F-T.rule`) | TeX morph: equations → matrix | |
| 10_3 | Where is the salt after ten minutes? Take it as one big step, and the rule says all hundred grams change tanks. Absurd. | One step of ten minutes: the readouts jump to 0 g and 100 g (`F-T.steps.n1`), flagged in red | highlight | |
| 10_4 | Ten steps say about fifty-five grams stay. A thousand steps say fifty-six point seven five. E to the ten A gives the answer in one stroke. Fifty-six point seven seven. | A small table: 10 steps, 55.37; 1,000 steps, 56.75; e^{10A}, 56.77 (`F-T.steps.n10`, `.n1000`, `F-T.at.t10`); the tanks' levels follow | table; live readouts | |
| 10_5 | ⟨predict⟩ Your turn. Leave the pumps running for hours. Where does the salt end up? | The quiz card: the two tanks, "after a long time?", ring from 5 (`F-Q.10_5.*`) | the quiz card | listen 5 s (countdown) |
| 10_6 | Fifty grams in each. And the exponential says how fast. Within a gram of even, after about twenty minutes. | Two curves, the grams in each tank against minutes, close on 50 (`F-T.at.*`, `F-T.settles`); a marker at 19.6 min, "within 1 g" (`F-T.within_a_gram.minutes`); "total: 100 g, always" (`F-T.total`) | curves from the model; highlight | hold |

- **The order's "real example":** two coupled tanks. The system of linear
  equations and its matrix are on screen together (the owner: "each square
  matrix is a system of linear equations").
- **"Hopeless otherwise":** a thousand matrix steps for five digits, against
  one evaluation that is exact at any time.
- **Ruth:** the one-step answer is Euler's method with a step far too
  large; the n-step numbers are exact (`50(1 + (1 − 2/n)^n)`, checked in the
  oracle). The mixing model assumes each tank is stirred.
- **Maya:** the quiz needs no formula: the salt evens out.

## 11_walks (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | A matrix can also be a map. Four nodes, some of them linked. Write a one where two nodes are linked, and a zero where they aren't. | A network of four nodes, a triangle 1, 2, 3 with a tail to 4 (`F-G.links`); its grid of ones and zeros fills in as each link glows; lower-third "the adjacency matrix" | **the network**; grid built from primitives | |
| 11_2 | Multiply that matrix by itself, and each entry counts something. The number of two-step walks between a pair of nodes. | The squared grid (`F-G.walks.k2`); its entry for nodes 1 and 4 is 1, and the one walk, 1 to 3 to 4, is traced on the network | grid; path traced | |
| 11_3 | ⟨predict⟩ Your turn. Start at node three. How many two-step walks bring you back to node three? | The quiz card: the network with node 3 glowing, ring from 5 (`F-Q.11_3.*`) | the quiz card | listen 5 s (countdown) |
| 11_4 | Three. Out along any of its three links, and straight back. The matrix agrees. | Three out-and-back walks traced one after another; the squared grid's entry for 3 and 3 lights: 3 | paths traced; highlight | |
| 11_5 | The k-th power counts walks of k steps. So e to this matrix adds up every walk of every length, and divides a walk of k steps by k factorial. Long walks count for less. Infinitely many walks, one function. | For nodes 1 and 4, the counts 0, 0, 1, 1, 4, 6, 17 … (`F-G.walks.one_to_four`) each divided by its factorial; the running total climbs 0.5, 0.667, 0.833, … to 0.92 (`F-G.exp.one_to_four.running`, `F-G.exp`); lower-third "a walk may revisit nodes and links" | count-up; lagged reveal | |
| 11_6 | Network scientists call that number communicability. How well two nodes are connected, counting every route between them, and not only the shortest. | e^A as a grid shaded by value (`F-G.exp`): 2.62 for nodes 1 and 3, 0.92 for nodes 1 and 4; lower-third "Estrada and Hatano, 2008" (`F-B.estrada_hatano.year`) | shaded grid | hold |

- **The owner's question:** "and also the adjacency matrix of a graph".
- **Ruth:** walks, not paths, repeats allowed (Stanley, Theorem 1.1). The
  oracle lists every walk up to length 8 and compares with the powers.
- **Dev:** `scipy.linalg.expm(A)[0, 3]` for a graph's adjacency matrix; the
  diagonal is Estrada's subgraph centrality. (Description.)
- **"Huh":** nodes 1 and 3 communicate more than nodes 1 and 2 (2.62
  against 2.42), though both pairs are directly linked: node 3 has more
  routes.

## 12_heat_and_chance (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Now let something flow along the links. Put heat on node four. Each link carries heat from its hotter end to its cooler end. | The network; node 4 glows amber, the others dark; small arrows along the links from hot to cool; the rule as a grid, −L (`F-D.L`) | the network; node glow | |
| 12_2 | That rule is a matrix too, so the answer is its exponential. Watch the heat spread, and level out. A quarter on each node. | The nodes' glow follows e^{−tL} as t runs 0.25, 0.5, 1, 2, 5 (`F-D.heat.*`); four amber bars beside them level at 0.25; "total: 1, always" | node colours and bars from a tracker | |
| 12_3 | ⟨predict⟩ Your turn. Same network, new meaning. A hopper jumps along the links at random. After a long time, which node is it most likely to be on? | A small hopper jumping from node to node; the amber now reads "chance"; the quiz card, ring from 6 (`F-Q.12_3.*`) | the quiz card; hopper | listen 6 s (countdown) |
| 12_4 | None of them. A quarter each. The busy node in the middle gets more visitors, and loses them just as fast. | The same four bars, now "chance of finding the hopper", settle at 0.25 each (`F-D.chance.from_hub.*`); lower-third "a hopper that leaves along each link at rate one; in probability this is written e^{tQ}" | bars; lower-third | |
| 12_5 | And the two tanks were this all along. Two nodes, one link, settling at half and half. | The network shrinks to two nodes and a link, and the two tanks fade in over them (`F-D.pair.is_the_tanks`, `F-D.pair.heat.*`) | morph that keeps identity | |
| 12_6 | Now put the letter i into the exponent. The same two nodes never settle. The chance sloshes across, and back, for ever. That is how quantum physics moves in time. | Two rows: e^{−tL}, two bars settling at ½ and ½; e^{−itL}, two bars swapping, all of it across at time π/2 and all back at π (`F-D.pair.quantum.*`); lower-third "a two-state quantum system, with ħ = 1: e^{−iHt}" | bars from a tracker; contrast | hold |

- **Three fields, one function:** heat (e^{−tL}), chance (e^{tQ}), quantum
  (e^{−iHt}), as the order lists them.
- **Misconception defused:** "the hopper ends up mostly at the busy node"
  (12_3). For this hopper the long-run chance is equal; the degree-weighted
  answer belongs to the walk that takes one step per tick (fact sheet,
  finding 8), and the lower-third says which hopper this is.
- **Ruth:** L = D − A (Kondor and Lafferty's form). The quantum beat claims
  only a two-state system with ħ = 1; the chances are cos²t and sin²t, and
  the total stays one (`F-X.quantum_total_one`).
- **Thread:** the i of chapter 06 turns settling into turning.

## 13_how_computers_do_it (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | So how does a computer find e to a matrix? Surely it adds up the series. | The series with a small "compute" tag and a cursor | highlight | |
| 13_2 | Try this matrix, from a famous paper. Nineteen Dubious Ways to Compute the Exponential of a Matrix. | The matrix −49, 24; −64, 31 (`F-C.mvl.A`); the paper's title as a card: Moler and Van Loan, SIAM Review, 1978 (`F-B.mvl.year`, `F-B.mvl.ways`) | card; quote | |
| 13_3 | Its terms swell past nine million, with alternating signs. They have to cancel down to four numbers smaller than two. | Bars for the largest entry of each term climb to 9,303,002 at terms 16 and 17 (`F-C.mvl.term.*`); far below, the true e^A: −0.736, 0.552, −1.472, 1.104 (`F-C.mvl.exact`) | bars from data; contrast of scale | |
| 13_4 | ⟨predict⟩ A computer that keeps about seven digits adds up that series, until the total stops changing. How good is its answer? | "32-bit numbers: about 7 digits"; the series running; the question as a card | prediction card | hold |
| 13_5 | Wrong in every entry. One even has the wrong sign. The mathematics is fine. The digits that mattered were thrown away long before the end. | Our 32-bit result, −0.789, 1.314, −0.126, −0.839 (`F-C.mvl.f32.series`), in red beside the true one; "after 62 terms" (`F-C.mvl.f32.terms`); lower-third "this video's own run, in 32-bit arithmetic" | contrast; highlight | |
| 13_6 | The cure is the oldest idea in this video. Tiny steps. Halve the matrix eight times, so its series is short and tame. Then square the result, eight times. | A ÷ 256 (`F-C.mvl.f32.scaled.divisor`); a short series of 8 terms; then eight squarings back up; the same 32 bits now give −0.736, 0.552, −1.472, 1.104, the true answer to three decimal places (`F-V.13_6.*`, `F-C.mvl.f32.scaled`), green; label "scaling and squaring" | **halve, then square back**: lagged chain | |
| 13_7 | Real libraries do exactly that, with a sharper stand-in for the short series. A ratio of two polynomials, called a Padé approximant. | At x = 1: the series through x⁶ gives 2.71806; the ratio of two cubics gives 193/71 = 2.71831, eight times closer to e (`F-C.pade.m3.*`, `F-C.series.k6.error`); lower-third "MATLAB and SciPy: Higham 2005; Al-Mohy and Higham 2009" (`F-B.higham.year`, `F-B.al_mohy_higham.year`) | TeX equation; lower-third | |
| 13_8 | And when a matrix untangles into separate rates, you exponentiate each rate alone. That's the eigenvalue story, which has a video of its own. | The tanks' matrix untangled into two rates, 0 and −0.2 a minute; e^{0·t} and e^{−0.2t} (`F-C.diag.*`); a pointer card to the eigendecomposition video | pointer card | hold |

- **The order:** "how e^A is computed in practice: diagonalize when possible
  (one beat, point to K), otherwise scaling and squaring with Padé
  approximants; cite Moler and Van Loan".
- **Honest about whose wrong answer:** the red numbers are this video's own
  32-bit run (62 terms, one wrong sign). Moler and Van Loan's printed run
  was on an IBM 370 (59 terms, two wrong signs); it is quoted in the
  description, not shown as ours (fact sheet, finding 2).
- **Ruth:** the paper names no best method ("perhaps three or four
  candidates"); the script says what libraries use, which is verified.
  Diagonalizing fails for matrices such as the shear of chapter 14, whose
  series stops by itself (`F-C.nil.exp.t3`); the description says so.
- **Dev:** this chapter is his: catastrophic cancellation, `expm` versus a
  hand-rolled series, degree-13 Padé in double precision.
- **Thread:** "tiny steps" a third time: (e^{A/256})^{256}.

## 14_order_matters (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | Remember the rule we said would break? E to the a plus b equals e to the a, times e to the b. | The gold box from 05_7 comes out of its corner to the centre | box | |
| 14_2 | ⟨predict⟩ Your turn. Swap the numbers for two matrices, A and B. Does the rule still hold? | The quiz card: e^{A+B} = e^A e^B ?, ring from 5 (`F-Q.14_2.*`) | the quiz card | listen 5 s (countdown) |
| 14_3 | Take two shears. A pushes the top of a square to the right. B pushes its right side upward. | A unit square; the rule A in teal slides its top edge right; the rule B in rose slides its right edge up; their grids beside them (`F-O.A`, `F-O.B`) | **the square, sheared**; morph | |
| 14_4 | Follow A, then B. Now B, then A. Now both rules at once. Three different shapes. | Three squares side by side become three different parallelograms, each with its grid: 1, 1; 1, 2 for A then B; 2, 1; 1, 1 for B then A; 1.543, 1.175; 1.175, 1.543 for both at once (`F-O.expB_expA`, `F-O.expA_expB`, `F-O.exp_sum`) | three morphs; grids | |
| 14_5 | The rule broke because order matters. A then B is not B then A. When the order makes no difference, we say two matrices commute, and for those, the rule holds. | AB and BA as two different grids (`F-O.AB`, `F-O.BA`); "commute: AB = BA"; lower-third "precisely: e^{t(A+B)} = e^{tA} e^{tB} for every time t exactly when AB = BA" | highlight; lower-third | |
| 14_6 | And the failure can be measured. Take one order away from the other. A B, minus B A. That leftover is called the commutator, and it is the first correction to the broken rule. | [A, B] = AB − BA = the grid 1, 0; 0, −1 (`F-O.commutator`); under it e^{tA} e^{tB} = e^{t(A+B) + ½t²[A, B] + …}; lower-third "the Baker–Campbell–Hausdorff formula" | TeX equation; lower-third | hold |

- **The order:** "e^{A+B} = e^A e^B only when A and B commute; the
  commutator measures the failure (the first term of
  Baker–Campbell–Hausdorff)". **Corrected:** the script never says "only
  when". Commuting is enough; for all times it is also necessary; a single
  pair can obey the rule without commuting (three, four and five full turns:
  `F-O.turns.*`). 14_5 says "for those, the rule holds", and the exact
  statement is on screen.
- **Convention on screen:** "A, then B" means A's motion first, so the
  matrix is e^B e^A (the later motion is written on the left). Each grid is
  shown under the shape it makes, and the caption names the order in words.
- **Ruth:** the gap, divided by t²/2, tends to the commutator (`F-O.gap.*`).
- **Sibling:** Video G keeps the book quiz for "rotations don't commute";
  this chapter uses shears, so the two do not repeat each other.

## 15_the_doorway (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | That leftover is a doorway. Go back to rotations. The quarter turn J has a special shape. Flip it across its diagonal, and every sign changes. | J; its entries swap across the diagonal and change sign; lower-third "skew: Aᵀ = −A" | highlight; morph | |
| 15_2 | Every matrix of that shape exponentiates to a rotation, and every rotation comes from one. In three dimensions there are three basic ones. A spin about each axis. | Three 3 by 3 skew grids, one per axis; a wireframe cube turns about x, then y, then z as e^{t·spin} runs (`F-L.exp.Lz.t1`, `F-X.skew_gives_rotation`); lower-third "a full turn and no turn are the same rotation" (`F-L.full_turn_is_identity`) | wireframe redrawn from the model; grids | |
| 15_3 | ⟨predict⟩ Your turn. Three dimensions, three independent spins. How many are there in four dimensions? | The quiz card: "3 dimensions: 3 spins. 4 dimensions: ?", ring from 6 (`F-Q.15_3.*`) | the quiz card | listen 6 s (countdown) |
| 15_4 | Six. A spin doesn't really happen around an axis. It happens in a plane, and four axes make six pairs. | Three axes and their three pairs, each pair a small turning plane; a fourth axis joins and three more pairs light: 6 (`F-L.spins.*`) | lagged reveal; count-up | |
| 15_5 | And spins don't commute. Tip a little about x, a little about y, then undo each one. You are not back where you began. What's left over is a small spin about z. The commutator of two spins is a third spin. | The cube: a small turn about x, about y, back about x, back about y; it ends slightly turned about z, shown by a ghost of where it started; [x-spin, y-spin] = z-spin (`F-L.bracket.*`, `F-L.loop.*`) | wireframe; ghost; TeX equation | |
| 15_6 | The spins, with that commutator, form a Lie algebra. The rotations they generate form a Lie group. And the exponential is the bridge from one to the other. | Left, a flat sheet of teal arrows, "tiny steps: a Lie algebra, so(3)"; right, a curved surface in blue, "motions: a Lie group, SO(3)"; a straight line on the sheet is carried by exp onto a curve on the surface; lower-third "Sophus Lie, 1842–1899" (`F-B.lie.*`) | **the bridge**: line onto curve | hold |

- **Show, then name:** "Lie algebra" and "Lie group" are said only after
  the spins, their commutator and the rotations are on screen.
- **Ruth:** exp maps skew matrices onto the rotations (compact and
  connected), not one-to-one; both halves are on screen. A Lie algebra is a
  vector space closed under the bracket; "the spins with that commutator"
  is so(3). The general definition (a smooth manifold that is a group) is in
  the description.
- **Maya:** the four-dimension quiz is a counting question she can do:
  pairs of axes.
- **Sibling:** this is the formal version of Video G's last picture; G
  writes a turn as e^{[ω]t}, shown once at 16_1.

## 16_where_it_leads (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 16_1 | Lie groups are the mathematics of smooth motion. A robot arm's pose is a product of exponentials, one for every joint. That's the companion video, on Lie groups and robotics. | A sphere turning and a dial going round; then an arm drawn as a chain of joints, each tagged with its own exponential, written Video G's way, e^{[ω]t}; a pointer card to the Lie groups and robotics video | pointer card; lagged reveal | |
| 16_2 | ⟨predict⟩ And in particle physics, the forces of the Standard Model are built on three Lie groups. Their independent tiny steps number eight, three, and one. Twelve in all. What might those twelve be? | SU(3) × SU(2) × U(1) with 8, 3 and 1 under them (`F-L.su3`, `F-L.su2`, `F-L.u1`); 8 + 3 + 1 = 12; the question as a card | prediction card; count-up | hold |
| 16_3 | The particles that carry the forces. Eight gluons. The W plus, the W minus, and the Z. And the photon. | Twelve tiles fill in: 8 gluons, W⁺, W⁻, Z, the photon (`F-B.bosons`, `F-L.standard_model.steps`); lower-third "the fine print is in the description" | grid of tiles; lagged reveal | |
| 16_4 | So that's what it means to raise e to a matrix. The matrix is a rule for tiny steps. E to the matrix is where that rule takes you. | A strip of four pairs, rule to result: a rate, growth; i, turning; a matrix, flow; a spin, rotation | recap strip | |
| 16_5 | Growth, turning, heat, chance, rotation. One function, following one rule. Its rate is itself. | The opening quiz card returns with its answer filled in and the plane turning one radian; then the receipts card, "every number in this video was computed by code", with the repository link and names to search: matrix exponential, Baker–Campbell–Hausdorff, Lie algebra | the quiz card, answered; receipts card | hold |

- **Honestly scoped** (the order): one line, one picture. "Built on three
  Lie groups" is the gauge group SU(3) × SU(2) × U(1); the twelve tiny
  steps are the dimensions of the three Lie algebras; the twelve carriers
  are as the Particle Data Group lists them. The description carries the
  fine print (the W's, Z and photon are mixtures; the group may be a
  quotient by a subgroup of Z₆; nothing is said about gravity).
- **Hand-off:** 16_1 points to Video G in G's own notation.
- **End on the rigorous form of the opening picture** (plan rule 15): the
  opening card, now answerable in one sentence, and the bridge of 15_6.
- **Receipts card** and names to search (plan rule 26).
