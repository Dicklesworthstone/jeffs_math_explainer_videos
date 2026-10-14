<!--
Draft video description for Video J (2026-10-14). Every statement below rests
on a row of docs/fact-sheet.md or an oracle key; the bracketed tags name them.
It holds the exact versions of what the narration simplifies (plan rule 11),
the programmer notes that have no room on screen, and what review round R1
sent "to the description". The release package adds the chapter timestamps and
the repository link, and trims this to what the upload page accepts, linking
this file for the rest.
-->

# What Does It Mean to Raise e to a Matrix?

Put a grid of four numbers `[0, −1; 1, 0]` in the exponent of `e`, and what
comes out is a rotation of the plane by one radian (`[cos 1, −sin 1; sin 1,
cos 1] ≈ [0.540, −0.841; 0.841, 0.540]`) [F-H.J.true]. One picture carries
the whole video: **the exponential turns a rule for tiny steps into where you
end up.** Put a number in, and growth comes out; put `i` in, and turning comes
out; put a matrix in, and the motion of a whole linked system comes out.

## Chapters

(Timestamps come with the finished cut.)

1. e to a matrix
2. The obvious guess
3. Growth
4. The series
5. One function
6. Turning
7. A matrix in
8. Does it settle
9. Follow the arrows
10. Two tanks
11. Walks
12. Heat and chance
13. How computers do it
14. Order matters
15. The doorway
16. Where it leads

## Precisely

What the narration says in plain words, stated exactly.

**Two uses of a matrix (chapters 1 and 9).** A matrix `A` is a linear map: feed
it an arrow `x`, and it returns an arrow `Ax`. The video keeps two uses of `A`
apart: as a **move** (`x ↦ Mx` is where you land) and as a **rule** (`dx/dt =
Ax` gives your velocity at `x`). The matrix exponential takes a rule `A` and a
time `t` and returns the move `e^{tA}` that carries every starting point `x(0)`
to `x(t) = e^{tA} x(0)` [F-X.flow_adds].

**The obvious guess fails (chapter 2).** Exponentiating entry by entry sends
the zero matrix `[0, 0; 0, 0]` to `[1, 1; 1, 1]`, which is not the identity
`I = [1, 0; 0, 1]` ("the matrix that changes nothing"), and sends `J = [0, −1;
1, 0]` to `[1, 1/e; e, 1]`, whose determinant is `0` rather than `1` [F-H.J.guess,
F-H.zero.guess].

**Two definitions, one function (chapters 3 to 5).** For any square matrix `A`,
the limit of tiny steps and the power series agree:

`lim_{n → ∞} (I + A/n)^n = ∑_{k=0}^∞ A^k / k! = e^A` [F-T.steps.n1000, F-T.at.t10].

With `A` replaced by `tA`, differentiating term by term gives `(d/dt) e^{tA} =
A e^{tA} = e^{tA} A`, with `e^{0·A} = I`. That is the unique matrix solution of
`dX/dt = A X`, `X(0) = I`.

**Why the series always settles (chapter 8).** Write `‖A‖` for the operator
2-norm (the largest singular value, called "the size of `A`" in the video).
Submultiplicativity `‖AB‖ ≤ ‖A‖ ‖B‖` gives `‖A^k / k!‖ ≤ ‖A‖^k / k!` [F-X.sizes_multiply,
F-S.bound]. Once `k > ‖A‖`, the bound's ratio `‖A‖ / k` is strictly below
`1`, so the series converges absolutely for every square matrix `A` [F-X.series_settles].
Before `k` passes `‖A‖`, the terms can swell: for `5J`, term `k = 5` has size
`5^5 / 5! = 3125/120 ≈ 26.04` before the factorial wins and the partial sums
settle on the unit circle [F-S.terms, F-S.peak, F-Y.five_j.peak_term]. [F-N.factorials]

**Two tanks and coupled systems (chapter 10).** With a flow transfer rate of
`0.1` per minute each way between two tanks, the amounts `x(t)` follow
`dx/dt = A x` with `A = [−0.1, 0.1; 0.1, −0.1]` [F-T.rule]. Starting from `(100, 0)`
grams, `x(t) = e^{tA} (100, 0) = (50 + 50 e^{−0.2t}, 50 − 50 e^{−0.2t})`:
`90.9` and `9.1` grams at `t = 1`, `56.8` and `43.2` grams at `t = 10`, settling
toward `(50, 50)` [F-T.at.t1, F-T.at.t10, F-T.settles]. [F-T.start, F-T.eigenvalues]

**Walks on a network (chapter 11).** For an adjacency matrix `A` with `0` and
`1` entries, the `(i, j)` entry of `A^k` counts walks of length `k` from node `j`
to node `i` (or `i` to `j` when `A` is symmetric) [F-G.walks.k1..k4]. In `e^A =
∑_{k=0}^∞ A^k / k!`, every walk of length `k` is weighted by `1/k!`
(Estrada and Higham 2010) [F-G.exp].

**Heat, continuous-time random walks, and the Schrödinger equation (chapter 12).**
For a graph with Laplacian `L = D − A`, heat flow `du/dt = −L u` has solution
`u(t) = e^{−tL} u(0)`, conserving total heat and settling to the uniform
average on any connected graph [F-D.L, F-D.heat.settles]. For a random hopper,
two models have different long-run behaviour on a graph whose nodes have
different degrees (such as the video's 4-node kite, degrees 2, 2, 3, 1): [F-G.nodes, F-G.degrees]
- Under the symmetric continuous-time Laplacian rate matrix `−L` (jumping
  across each link at unit rate), the stationary distribution is uniform: `1/4`
  on every node [F-D.chance.from_hub.settles, F-Y.kite.stationary_continuous].
- For a discrete-time hopper that picks a uniformly random link at each tick
  (or a continuous-time hopper that leaves every node at unit total rate and
  chooses a random neighbour), the stationary probability of a node is
  proportional to its degree: `2/8, 2/8, 3/8, 1/8` (`0.250, 0.250, 0.375, 0.125`)
  [F-D.chance.per_tick, F-Y.kite.stationary_per_tick].
Putting `−i` in front of `L` turns `du/dt = −L u` into the discrete Schrödinger
equation `dψ/dt = −i L ψ`, whose solution `ψ(t) = e^{−itL} ψ(0)` is unitary
(`‖ψ(t)‖ = ‖ψ(0)‖` for all `t`) and oscillates without settling: on a 2-node
link, the probabilities at the two nodes are `cos² t` and `sin² t`
[F-D.pair.quantum.t1, F-X.quantum_total_one]. [F-G.degrees]

**When does `e^{A+B} = e^A e^B` hold? (chapter 14).** If `AB = BA`, then the
binomial expansion holds for `(A + B)^k` and `e^{A+B} = e^A e^B`
[F-O.commuting.rule_holds, F-X.commuting_rule_holds]. The converse is true for *all times*
(`e^{t(A+B)} = e^{tA} e^{tB}` for every real `t` implies `AB = BA`, as follows
from differentiating twice at `t = 0`), but **not** for a single pair `(A, B)`
at `t = 1`. A clean 3-by-3 counterexample uses the 3-4-5 right triangle: let
`X` be the skew-symmetric generator of `3` full turns (`6π` radians) about the
x-axis and `Y` the generator of `4` full turns (`8π` radians) about the y-axis.
Then `XY ≠ YX`, yet `X + Y` generates `√(3² + 4²) = 5` full turns (`10π`
radians) about the tilted axis `(3/5, 4/5, 0)`, so `e^X = I`, `e^Y = I`, and
`e^{X+Y} = I = e^X e^Y` [F-O.turns.*].

**The commutator loop (chapters 14 and 15).** Following `A` for time `t`, then
`B` for time `t`, then `−A` for time `t`, then `−B` for time `t` multiplies in
the order played (later motion on the left):

`e^{−tB} e^{−tA} e^{tB} e^{tA} = I − t²(AB − BA) + O(t³)` [F-O.loop.is_minus_commutator,
F-L.loop.is_minus_z_spin].

(Conventions that write the product left-to-right or start with `−A` get `+t²[A, B]`.)
Alternating tiny steps `(e^{(t/n)B} e^{(t/n)A})^n` kills the `t²` error and
converges to `e^{t(A+B)}` as `n → ∞` (the Lie–Trotter product formula)
[F-O.alternate.n10, F-O.alternate.n1000].

**The doorway to Lie theory and the Standard Model (chapters 15 and 16).** The
skew-symmetric matrices (`Aᵀ = −A`) form the Lie algebra `so(n)`, closed under
the commutator `[A, B] = AB − BA`; the exponential maps `so(n)` onto the
connected rotation group `SO(n)` [F-L.bracket.*, F-X.skew_gives_rotation]. In physics,
unitary phase transformations come from skew-Hermitian matrices (`X* = −X`, or
`iH` with `H` Hermitian). The Lie algebra of the Standard Model gauge group is
`su(3) ⊕ su(2) ⊕ u(1)`, which has `(3² − 1) + (2² − 1) + 1 = 8 + 3 + 1 = 12`
independent directions — corresponding to the 8 gluons, the 3 weak gauge fields
(`W¹, W², W³`, which mix with the `u(1)` hypercharge field `B` after electroweak
symmetry breaking to form `W⁺, W⁻, Z⁰` and the photon `γ`) [F-L.su3, F-L.su2,
F-L.u1, F-L.standard_model.steps]. Globally, the Standard Model gauge group is
`(SU(3) × SU(2) × U(1)) / Γ`, where the discrete central quotient `Γ` is a
subgroup of Z6 (`ℤ₆`, depending on which global group is taken as fundamental;
Baez and Huerta 2010), all sharing the same 12-dimensional Lie algebra.

## For programmers

- **Why summing the power series in floating point fails (chapter 13).** On
  Moler and Van Loan's test matrix `A = [−49, 24; −64, 31]` (eigenvalues `−1`
  and `−17`), the exact exponential is `e^A ≈ [−0.735759, 0.551819; −1.471518,
  1.103638]` [F-C.mvl.A, F-C.mvl.eigenvalues, F-C.mvl.exact]. Along the series,
  `A^k / k!` grows to `9,303,002.5` in magnitude at `k = 16` and `k = 17` before
  cancelling down below `1.5` [F-C.mvl.term.largest]. In IEEE 754 32-bit float
  (`f32`, 24-bit binary significand, about 7 decimal digits), our loop `term =
  (term * A) / k; total = total + term` stops changing at `k = 62` and returns
  `[−0.788594, 1.314455; −0.126288, −0.839240]`: no entry is right to two
  digits, and one has the wrong sign. Other equally natural orders of the same
  operations give other wrong answers [F-C.mvl.f32.*].
- **Why our 32-bit numbers differ from Moler and Van Loan's 1978 paper.** Moler
  and Van Loan (1978, p. 808; 2003, p. 8) ran their experiment on an **IBM 370**
  in short hexadecimal floating-point arithmetic (base 16, 6 hex digits = 24
  bits with wobbling precision), stopping at `K = 59` terms with `[−22.25880,
  −1.432766; −61.49931, −3.474280]`: two entries with the wrong sign, the two
  in its right-hand column [F-B.mvl.ibm.result]. Because IBM 370 hex rounding is
  not IEEE 754 binary rounding, the video shows the actual IEEE `f32` run (`k =
  62`, `[−0.788594, 1.314455; −0.126288, −0.839240]`, one entry with the wrong
  sign) and cites the IBM 370 numbers here. Even in IEEE 64-bit (`f64`), summing the
  raw series for `2A` loses almost all accuracy (`0` of `4` entries right to two
  significant digits) [F-C.mvl.f64.doubled.entries_right_to_two_digits]. [F-B.mvl.year, F-B.mvl.revisit_year, F-B.mvl.ibm.terms, F-C.mvl.f32.series, F-C.mvl.f32.terms]
- **Scaling and squaring with a Padé approximant.** Halving `A` eight times
  (`A / 256`, size `0.35 < 1`) keeps the series terms tiny; summing 8 terms
  and squaring 8 times (`e^A = (e^{A/2^s})^{2^s}`) recovers `[−0.735775,
  0.551832; −1.471552, 1.103665]` in `f32` (worst error `0.000034`), though
  over-halving (e.g. `s = 20`) loses accuracy again as `I + A/2^s` rounds away
   digits of `A` before the 20 squarings amplify the rounding error
  [F-C.mvl.f32.scaled.*, F-C.mvl.f32.halvings.*, F-Y.mvl.size_over_divisor]. Production libraries
  (`scipy.linalg.expm`, MATLAB `expm`, Julia `exp`) pair scaling and squaring
  with a `[m/m]` diagonal Padé approximant `p_m(X) / q_m(X)` (`m ∈ {3, 5, 7, 9,
  13}` chosen from the matrix norm and `‖A^{2k}‖^{1/(2k)}` bounds; Higham 2005,
  Al-Mohy and Higham 2009): at `x = 1`, `[3/3]` Padé (`193/71 ≈ 2.718310`) is
  `8.1` times closer to `e` than the degree-6 Taylor polynomial (`1957/720 ≈
  2.718056`) for the same highest power [F-C.pade.*]. [F-B.pade.degrees, F-B.higham.year, F-B.al_mohy_higham.year, F-N.series.k6, F-N.factorials, F-Y.taylor6.at_one]
- **Do not compute `e^{tA} v` by forming `e^{tA}` when `A` is large and sparse.**
  On the 400-node grid (`20 × 20`), `L` is sparse (`1,920` nonzeros) while
  `e^{−tL}` is completely dense (`160,000` positive entries). For a single
  vector `e^{tA} v`, use Krylov subspace methods or Chebyshev/Taylor action
  (`scipy.sparse.linalg.expm_multiply`, Al-Mohy and Higham 2011). When the graph
  is a Cartesian grid, `L = L_x ⊗ I + I ⊗ L_y` commutes, so `e^{−tL} = e^{−tL_x}
  ⊗ e^{−tL_y}` and two `20 × 20` exponentials do the job of one `400 × 400`. [F-D.big.nodes, F-Y.big.side, F-Y.big.laplacian_nonzeros, F-Y.big.exp_positive_entries]

## Sources

- Jacob Bernoulli, *Ars Conjectandi*, 1713 (continuous compounding and the
  limit of `(1 + 1/n)^n`).
- Leonhard Euler, *Introductio in analysin infinitorum*, Vol. 1, 1748, Chapter
  VII (`e = 2.71828182845904523536028…` and its series, p. 90; Euler's formula
  connecting `e^{ix}` to `cos x + i sin x`, §138, p. 104).
- Arthur Cayley, "A Memoir on the Theory of Matrices", *Philosophical
  Transactions of the Royal Society of London* 148, 1858, pp. 17–37.
- Sophus Lie and Friedrich Engels, *Theorie der Transformationsgruppen*, 3
  vols., Teubner, 1888–1893.
- Henri Padé, "Sur la représentation approchée d'une fonction par des fractions
  rationnelles", *Annales scientifiques de l'École Normale Supérieure* (3) 9,
  1892, pp. 3–93.
- Cleve Moler and Charles Van Loan, "Nineteen Dubious Ways to Compute the
  Exponential of a Matrix", *SIAM Review* 20(4), 1978, pp. 801–836; updated as
  "Nineteen Dubious Ways to Compute the Exponential of a Matrix, Twenty-Five
  Years Later", *SIAM Review* 45(1), 2003, pp. 3–49.
- Nicholas J. Higham, "The Scaling and Squaring Method for the Matrix
  Exponential Revisited", *SIAM Journal on Matrix Analysis and Applications*
  26(4), 2005, pp. 1179–1193.
- Awad H. Al-Mohy and Nicholas J. Higham, "A New Scaling and Squaring Algorithm
  for the Matrix Exponential", *SIAM Journal on Matrix Analysis and
  Applications* 31(3), 2009, pp. 970–989.
- Ernesto Estrada and Desmond J. Higham, "Network Properties Revealed through
  Matrix Functions", *SIAM Review* 52(4), 2010, pp. 696–714.
- John C. Baez and John Huerta, "The Algebra of Grand Unified Theories",
  *Bulletin of the American Mathematical Society* 47(3), 2010, pp. 483–552.

## Every number

Every matrix, series term, norm, walk count, heat value, and floating-point
trace in the video is computed by the video's own Rust model (`src/model.rs`)
and checked against an independent rational/SciPy oracle (`./x
matrix_exponential facts-check`, `oracle.py`). The 200 random edge
matrices for the universal claims (`F-X.*`) are drawn from NumPy's
`Generator(PCG64DXSM(20261013))` alongside the named degenerate cases (zero,
nilpotent, defective shear, non-normal).
