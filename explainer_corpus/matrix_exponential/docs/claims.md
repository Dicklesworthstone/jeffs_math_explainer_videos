# Claim walk — What Does It Mean to Raise e to a Matrix?

Gate G3 (bead mx-e-a-j-8yq7.5). Every line of `script.tsv` that asserts
something, with what backs it: a fact key (`./x matrix_exponential
facts-check`, 354 of 354 against the oracle), a Rust test, or a source in
`docs/fact-sheet.md`. Lines that only ask, name or point are not listed.
Walked against the script as de-slopified on 2026-10-13; the pre-pass copy is
`docs/script.pre-deslop.tsv`.

Rules checked on every line: a spoken number equals the number on screen at
that beat and its fact, rounded the same way; every step the screen shows is
said in the same order; no superlative or comparison without a fact; no
"always" or "every" without an edge test or a proof named in the fact sheet.

## The walk

| line | what it asserts | backed by |
|---|---|---|
| 01_2 | e to the quarter turn is a rotation of the plane by exactly one radian | `F-H.J.true` (cos 1, −sin 1; sin 1, cos 1 to six places; the oracle's exact series against the closed form); the reason is 16_4 |
| 01_4 | a matrix turns an arrow into an arrow, each row mixing the arrow's two numbers | `F-M.machine.*`: (2, 1) goes to (−1, 2) |
| 01_5 | this one turns every arrow a quarter turn | `F-J.power.k1`; test `the_quarter_turn_pushes_at_right_angles` (same length, zero dot product, for every arrow tried including zero) |
| 02_1 | e is about 2.718 | `F-V.02_1.e` |
| 02_2 | e to the zero is one, so the entry-by-entry guess turns four zeros into four ones | `F-H.zero.guess` |
| 02_3 | multiplying by one changes nothing; its matrix twin has ones on the diagonal and leaves every arrow alone | `F-H.zero.true` (the series for the zero matrix is the identity) |
| 02_4 | four ones squash the whole plane onto a single line | `F-H.zero.guess.det` = 0; each arrow (x, y) lands at (x + y, x + y) |
| 03_1, 03_2 | one dollar at 100% becomes 2; in two steps of 50%, 2.25 | `F-N.compound.n1`, `F-V.03_2.two_steps` |
| 03_3 | four steps give 2.44, twelve give 2.61 | `F-V.03_3.*` |
| 03_4 | it settles; a step every day gives 2.715; a million give 2.71828; that number is e | `F-V.03_4.daily`, `.million`, `.e`; the limit is e (standard; Euler, *Introductio* §125, source S12) |
| 03_5 | in a thousandth of a year you gain a thousandth of what you have, so the gain per year equals the amount | the step rule at n = 1000; in the limit f′ = f (strip) |
| 03_6 | growing so for x years gives e to the x: the one curve that starts at one with rate equal to amount | uniqueness: fact sheet, "Simplifications", first row |
| 04_2 | 1 has rate 0; x climbs at rate 1 | elementary |
| 04_3 | half of x squared is the area under the line, and an area grows as fast as its leading edge is tall | the fundamental theorem of calculus, as the picture |
| 04_5 | x cubed over six, then x to the fourth over twenty-four; the bottoms are the factorials | `F-Q.04_4.answer` (x^3/6), `F-N.factorials` |
| 04_6 | take the rate of the series and each term turns into the term before it; nothing is lost | termwise differentiation of an everywhere-convergent power series (chapter 08; description) |
| 05_1 | at x = 1 the running total climbs to 2.71828, the bank's number | `F-V.05_1.running`, `F-V.05_1.settles` (k = 9) against `F-V.03_4.million` |
| 05_2 | both obey rate equals amount from the same start, and only one curve can | uniqueness, as 03_6 |
| 05_3 | if the rate is a fixed share of the amount, 5% a year, the answer is e to the 0.05 t (for a start of one) | y′ = ry ⇒ y = y(0)e^{rt}; `F-V.05_3.share` |
| 05_4 | savings, bacteria, radioactive decay and a cooling cup are settings of that share | each is y′ = ry (cooling: for the difference from the room, which the label says) |
| 05_6 | 5% times 20 is one, so it grows by e, 2.718 times | `F-V.05_6.*`, `F-Q.05_5.answer` |
| 06_1 | multiplying by i is a quarter turn | multiplication by i in the plane; `F-J.power.*` for its matrix |
| 06_2 | a rate of i times the position is a push at right angles, as long as the distance from the centre | test `the_quarter_turn_pushes_at_right_angles` |
| 06_4 | four big steps drift outward, a hundred hug the circle; in the limit you stay on it at speed one, so a time t is an arc of length t, t radians | `F-V.06_4.four_steps` (−2.32, 1.20), `.hundred_steps` (−1.05, 0.00); `F-Q.06_3.answer` (radius 1.000 at 62 times); speed = length of the push = 1 on the unit circle |
| 06_5 | the point is e to the i t; after a time π it is at −1 | `F-N.turn.pi` (−1, 0); the name: 05_3's rule with i as the share |
| 07_1 | the series asks of x only multiplying by itself, dividing by whole numbers and adding, and a square matrix can do all three | the definition; square so that it can multiply itself |
| 07_2 | that is the definition of e to a matrix | Laguerre 1867 (source S14) |
| 07_4 | J squared is a half turn, minus the identity; J behaves like i | `F-J.power.k2`, `F-J.period` (4), `F-Q.07_3.answer` |
| 07_5 | the terms alternate, one along the arrow, the next at right angles, and spiral in to a point on the circle | the terms of e^{tJ}(1, 0) are t^k/k! times successive quarter turns; `F-J.split.*` |
| 07_6 | the across part is cos t and the up part sin t; e to the tJ is the rotation by t radians; at t = 1 it is the opening matrix | `F-J.split.even`, `.odd`; `F-V.07_6.at_one`; test `the_exponential_of_the_quarter_turn_is_the_rotation` (seven times from 0 to 40) |
| 08_1 | to have an answer the running total must settle in every entry: the series converges | the definition (convergence in any matrix norm is entrywise convergence) |
| 08_2 | e to 5J must land five radians round the circle, yet two of its terms are near twenty-six | `F-S.limit`, `F-V.08_2.bars`, `F-V.08_2.near`, `F-S.peak.k` (k = 4 and 5) |
| 08_4 | once dividing by 6, 7, 8 the terms collapse; by k = 20, about forty millionths; the total spirals home | `F-Q.08_3.answer` (every term after k = 5 is smaller than the last, to k = 60), `F-V.08_4.term20.tens_of_millionths`, `F-V.08_4.home` |
| 08_5 | for every square matrix: the k-th term is at most size^k/k!, the factorial always wins, the remainder keeps shrinking and the total settles | the comparison with the series for e^{size}; `F-X.series_settles` (200 of 200), `F-X.named.series_settles` (13 of 13, exact), `F-S.strict.*`, `F-V.08_5.bound` |
| 09_1 | a matrix can be read as position in, velocity out | x′ = Ax |
| 09_2 | shrinking the tiny steps to nothing gives each point a smooth path | (I + tA/n)^n → e^{tA}; test `scaling_and_squaring_agrees_with_the_raw_series_and_with_tiny_steps` (error falls like 1/n) |
| 09_3 | following the arrows for a time t carries each point to its finish, and the matrix that does it is e^{tA} | x(t) = e^{tA}x(0) is the solution of x′ = Ax (Peano 1888, source S15); `F-M.spiral.*` for the second rule shown |
| 09_4 | the rate of the series is A times itself | termwise: d/dt (t^k A^k/k!) = A · t^{k−1}A^{k−1}/(k−1)! |
| 09_6 | running for s and then t is running for s + t | `F-X.flow_adds` (200 of 200), `F-X.named.flow_adds` (13 of 13) |
| 10_1, 10_2 | two 100-litre tanks, 10 litres a minute each way: salt leaves each at a rate of a tenth of what it holds per minute | `F-T.rule` |
| 10_3 | one step of ten minutes moves all hundred grams across | `F-V.10_3.one_step` (0, 100) |
| 10_4 | ten steps do better, a thousand better still; e to the 10A gives 56.77 g | `F-V.10_4.ten_steps` (55.37), `.thousand_steps` (56.75), `.exact` (56.77), `F-T.off_by.*` |
| 10_6 | fifty grams in each; each tank within a gram of fifty after about twenty minutes | `F-Q.10_5.answer`, `F-V.10_6.minutes` (19.6), `F-T.total` |
| 11_2 | each entry of the matrix times itself counts the two-step walks between a pair of nodes | `F-G.walks.k2`; test `walk_counts_are_the_powers_of_the_adjacency_matrix` |
| 11_4 | three two-step walks from node three back to itself | `F-Q.11_3.answer` |
| 11_5 | the k-th power counts walks of k steps; e to the matrix adds every walk, a walk of k steps divided by k!; infinitely many walks | Stanley, Theorem 1.1 (source S8); `F-G.exp.one_to_four.running`, `F-V.11_5.one_to_four` (0.92) |
| 11_6 | that number is called communicability; nodes one and three score higher than one and two, though both pairs are linked | Estrada and Hatano 2008 (source S7); `F-V.11_6.one_to_three` (2.62), `.one_to_two` (2.42); narration-truth test |
| 12_1 | each link carries heat from its hotter to its cooler end as fast as the difference | u′ = −Lu, `F-D.L` |
| 12_2 | the heat spreads and levels out, a quarter on each node | `F-D.heat.*`, `F-V.12_2.level`; `F-X.heat_conserves` (100 of 100) |
| 12_3 | the same matrix gives the chance of finding a random hopper | P(t) = e^{tQ} with Q = −L (Norris, source S10); the strip names the hopper; `F-D.chance.*` |
| 12_4 | on four hundred nodes the exponential still answers | `F-D.big.*` (400 nodes, 760 links; the whole Laplacian exponentiated by SciPy agrees with the video's route) |
| 12_5 | the two tanks are two nodes and one link, settling at half and half | `F-D.pair.is_the_tanks`, `F-D.pair.heat.*` |
| 12_6 | with i in the exponent it never settles: the chance goes across and back for ever; that is how a quantum state moves in time | `F-D.pair.quantum.*` (cos² t, sin² t; all across at π/2, back at π), `F-X.quantum_total_one`; U(t) = e^{−iHt} (source S11); scoped on the strip to a two-state system with ħ = 1 |
| 13_1 | the paper is Moler and Van Loan's "Nineteen Dubious Ways to Compute the Exponential of a Matrix" | source S1; `F-B.mvl.*` |
| 13_2 | its terms swell past nine million with alternating signs; a 32-bit computer keeps about seven digits | `F-V.13_2.millions`, `F-C.mvl.term.k16`, `.k17` (opposite signs); 24 binary digits |
| 13_3 | every entry comes out wrong, one with the wrong sign | `F-V.13_3.entries_right_to_two_digits` (0), `F-C.mvl.f32.wrong_signs` (1); test `the_thirty_two_bit_series_fails_and_scaling_and_squaring_does_not` |
| 13_4 | half the time run twice is the whole time; halve eight times, sum a short series, square eight times | e^A = (e^{A/2})²; `F-C.mvl.f32.scaled.*`, `F-V.13_4.right_to_three_places` |
| 13_5 | real programs do that with a ratio of two polynomials, a Padé approximant | SciPy and MATLAB documentation (sources S3 to S6); `F-V.13_5.*` |
| 14_1 | for numbers, two shares at once is one after the other: e to the sum is e to the first times e to the second | `F-N.add.*` |
| 14_4 | A then B and B then A are mirror images; both at once lands between them; three different results | `F-O.expB_expA` (1, 1; 1, 2), `F-O.expA_expB` (2, 1; 1, 1), `F-V.14_4.both_at_once`; `F-O.alternate.*` for "a tiny step of each"; `F-Q.14_3.answer` |
| 14_5 | the rule broke because order matters; when AB = BA the rule holds | `F-O.AB`, `F-O.BA`; `F-X.commuting_rule_holds` (200 of 200), `F-X.named.commuting_rule_holds` (6 of 6); the script never says "only when" (fact sheet, finding 1) |
| 14_6 | round the loop A, B, undo each, the square does not come back, and what is left over is set by AB − BA | `F-O.loop.*` (leftover/t² → −[A, B]); test `the_loops_leave_minus_the_commutator_in_the_order_played` |
| 15_1 | flip J across its diagonal and every sign changes; follow such a rule and you get a rotation; three dimensions have three, one for each pair of axes | `F-X.skew_gives_rotation` (200 of 200), `F-X.named.skew_gives_rotation` (5 of 5); `F-L.spins.n3` |
| 15_3 | repeat the loop and the cube creeps round the third axis; the commutator of two spins is a third spin | `F-Q.15_2.answer` (off by 2.28°), `F-L.creep.*` (45.7° after twenty loops, about an axis 8.1° off z), `F-L.bracket.*` |
| 15_4 | spins are closed under adding and under the commutator (a Lie algebra); rotations under following and undoing (a Lie group); the exponential takes each spin to a rotation and reaches every rotation | so(3) and SO(3); onto: Hall 2015, Cor. 11.10 (source S17) |
| 16_1 | a robot arm's pose is a product of exponentials, one per joint | Lynch and Park's product of exponentials, times the home pose M on screen (Video G) |
| 16_2 | the Standard Model's forces are built on three Lie groups with 8, 3 and 1 independent tiny steps; twelve particles carry those forces | `F-V.16_2.steps`, `F-L.standard_model.steps` pinned to `F-B.bosons` (Particle Data Group, source S18); fine print in the description |
| 16_4 | the rule J moves every point at a speed equal to its distance from the centre, so in one unit of time each point travels an arc as long as its radius: one radian | 06_2's identity; the definition of the radian; `F-Q.16_3.answer` |
| 16_5 | the rate of e to the matrix is the rule applied to itself | 09_4 |

## Always and every

"Every" and "always" occur at 01_5 (every arrow), 02_3 (every arrow), 08_5
(every square matrix; the factorial always wins), 16_4 (every point). Each
has its row above and, where it is a claim about all matrices, a row in the
fact sheet's edge table (seeded and named cases). "Never" occurs at 12_6: the
two-node quantum system does not settle (its chances are cos² t and sin² t
for all t).

## Letters

The script says single letters aloud (e, x, t, i, J, A, B, k, s). No sentence
opens with the letter A (the voice would read the article); A and B are
introduced as "two matrices, A and B", and the shares of 14_1 are "the
first" and "the second", not a and b. The letter-heavy sentences are checked in the narration sheets (P6).

## Names (P3 probe, 2026-10-13, michael)

Verified: Lie as "Lee" (transcript and spectrogram), Van Loan, Padé, commutator,
communicability; Euler, Baker, Hausdorff and Estrada pass but are on screen
only. Partial, for the owner's ear or the narration sheets: Moler ("Mohler",
heard as Mola, MOH-luh in this non-rhotic voice) at 13_1, and eigenvalue at
13_5 (fails only at a sentence start, where the script never puts it).
Campbell is on screen only and unverifiable by ASR. Records in `lexicon.json`.

## Changed by the de-slopify pass

31 of 87 lines changed. Removed on sight: one-word verdicts used as punch
lines ("Meaningless.", "Absurd.", "No coincidence."), slogan cadences ("Two
equations. Together, one matrix."; "Infinitely many walks, one function.";
"One function, one rule."; "Every time."), two vague metaphors ("a doorway",
"the bridge between them", "the mathematics of smooth motion", each replaced
by what it stood for), "do something violent", and an unnamed "famous paper"
(now "a paper by Moler and Van Loan"). "That is how quantum physics moves in
time" became "how a quantum state moves in time". Then the voice rules again:
no colons, semicolons, dashes or parentheses; numbers as words; lint PASS.
