# Fact sheet: Video J, "What Does It Mean to Raise e to a Matrix?"

Bead `mx-e-a-j-8yq7.2` (gate G1). One row per number the video may show or
say, and one row per claim that rests on a source. Every value in the tables
at the end was copied from the oracle's output by a script, not typed.

```
./x matrix_exponential facts-check        # expm facts == oracle.py, key by key
```

- **Result, 2026-10-13** (the build host; CPython 3.14, NumPy and SciPy
  through uv): `facts-check matrix_exponential: 233 video facts against 233
  oracle keys: PASS`. The oracle prints 233 keys, passes 92 of 92 of its own
  cross-checks, and compares 69 exact exponentials with SciPy's `expm`.
- **Rust side:** `./x matrix_exponential test`: 16 tests pass, clippy
  `-D warnings` and `fmt --check` clean.
- **Rows:** F-N the number exponential; F-H the hook; F-J the quarter turn;
  F-S does the series settle; F-T two tanks; F-G walks on a network; F-D
  spreading (heat, chance, a two-state quantum system); F-C computing it;
  F-O order matters; F-L the doorway to Lie theory; F-X the edges of the
  always sentences, each as "how many hold of how many checked"; F-B cited
  literals.

## How the two sides get their numbers

| What | `src/model.rs` (the video) | `oracle.py` (independent) |
|---|---|---|
| every matrix exponential | scaling and squaring in 64-bit floats: halve the matrix until its largest row sum is at most one half, sum 24 terms of the series, square back up | the exact power series in Python `Fraction`s, stopped when a geometric bound on the remainder is below 10⁻³⁰; closed forms (cosine and sine, Rodrigues' formula, the tanks' eigenvalues) where the matrix carries π or the time is long; and every exact result is then compared with SciPy's `expm` (Padé with scaling and squaring), a third route |
| the size of a matrix (the most it can stretch an arrow) | the square root of the largest eigenvalue of AᵀA, by Jacobi rotations | NumPy's largest singular value; the terms of e^{5J} also as the exact fractions 5ᵏ/k! |
| compounding and the number series | 64-bit floats | exact fractions; a million compoundings in 60-digit decimals |
| walks on the network | one link added at a time (integer counts) | every sequence of nodes listed and the ones that follow links kept; then compared with the powers of the adjacency matrix |
| the two-state quantum system | the exponential of the real 4 by 4 matrix that acts as −iH on (real parts, imaginary parts) | the amplitudes (1 ± e^{−2it})/2 from the eigenvalues 0 and 2; SciPy's complex `expm` as a check |
| the 32-bit runs | Rust `f32`, each operation rounded | NumPy `float32` scalars, the same operations in the same order (so the two agree bit for bit) |
| Padé approximants at x = 1 | integer coefficients (2m − j)!/(j!(m − j)!), reduced | the rational function evaluated in fractions; its first error term checked for m = 1 to 6 |
| the edges of the always sentences | 200 seeded matrix pairs (PCG64DXSM, seed 20261013; sizes 2, 3, 4; entries in [−3, 3)), judged with `model::expm` | the same 200 pairs from NumPy's PCG64DXSM, judged with SciPy's `expm` and NumPy's norms |
| cited numbers | constants in `facts::cited`; the ones the video also computes are pinned against its own arithmetic (e to 15 places against Euler's digits; e^A for Moler and Van Loan's matrix against their six printed places; its eigenvalues; 8 + 3 + 1 against the count of force carriers) | the same literals, typed from the sources; plus Moler and Van Loan's printed sixteenth term and their factorization, both checked exactly |

Printed precision: transcendental values at four to six decimals; a value
within 10⁻⁶ of a rounding tie at its printed precision is refused by both
sides, so the two routes cannot round differently (none is).

## Findings (uncomfortable first)

1. **The order's "e^{A+B} = e^A e^B only when A and B commute" is false as
   stated for a single pair.** Three full turns about the x axis, four about
   the y axis and their sum (five full turns about a tilted axis: 3, 4, 5)
   are all the identity, so e^{X+Y} = e^X e^Y, yet XY ≠ YX
   (`F-O.turns.*`). What is true: if AB = BA the rule holds
   (`F-X.commuting_rule_holds`, 200 of 200); and e^{t(A+B)} = e^{tA} e^{tB}
   for *every* time t exactly when AB = BA, because the gap for short times
   is ½t²(AB − BA) (`F-O.gap.*`, and the test
   `edge_the_gap_is_half_t_squared_times_the_commutator`). The script says
   "when they commute, it holds; when they don't, it can fail", with the
   exact statement on screen; it never says "only when".
2. **Moler and Van Loan's famous wrong answer cannot be reproduced on a
   modern machine, and ours differs from theirs.** Their series run (1978,
   pp. 807–808) used IBM 370 hexadecimal short arithmetic: 59 terms, two
   entries with the wrong sign. The same series in IEEE 32-bit arithmetic
   (`F-C.mvl.f32.*`, computed here and repeated operation for operation in
   NumPy) takes 62 terms and returns four numbers that are each wrong in the
   first digit, one with the wrong sign, the worst off by 1.94. The video
   shows its own run and labels it; their printed digits are quoted as
   theirs (`F-B.mvl.ibm.*`) and only in the description.
3. **They name no best method.** "Thus, we see perhaps three or four
   candidates for 'best' method" (1978, Conclusions); scaling and squaring
   is "the only generally competitive series method". The 2003 revisit
   reprints those conclusions and adds that such methods "have an even
   greater appeal". The script says scaling and squaring with Padé
   approximants is what MATLAB and SciPy use (verified in their
   documentation and in SciPy's source), not that the paper crowned it.
4. **Jacob Bernoulli and "1683".** The date in common circulation is
   MacTutor's and has no document behind it; the compound-interest question
   was printed in *Acta Eruditorum* in May 1690. If the script gives a year
   it is 1690.
5. **Euler did not write e^{ix} = cos x + i sin x.** In the *Introductio*
   (1748) the letter i is an infinitely large number (e^z = (1 + z/i)^i,
   §125) and the formula reads e^{v√−1} = cos v + √−1 sin v (§138). The
   script credits him with the series, the limit and the formula, and puts
   no modern notation in his mouth.
6. **Laguerre defined it; Peano used it.** Laguerre (1867) defines e^X for
   a matrix by the power series and already remarks that e^X e^Y = e^{X+Y}
   fails in general; he applies it to abelian functions, not to
   differential equations. Peano (1887, 1888) writes x = e^{Rt}a for
   x′ = Rx. One on-screen line; no "invented by".
7. **The Standard Model's group has fine print.** SU(3) × SU(2) × U(1) is
   the usual statement (Particle Data Group); the group that acts
   faithfully on the known matter is that product modulo a subgroup of
   Z₆ that experiments cannot yet pin down (Tong 2017; Baez and Huerta
   2010). The Lie algebra, and so the count 8 + 3 + 1 = 12, is the same in
   every case. The script speaks of "twelve independent tiny steps, one for
   each force carrier"; the fine print is in the description. The match is
   with the carriers as usually listed (8 gluons, W⁺, W⁻, Z, photon); the
   W's, Z and photon are mixtures of the SU(2) and U(1) fields, which the
   description says.
8. **The hub is not favoured.** For a hopper that leaves along each link at
   rate one (the rule −L), the long-run chance is one quarter at every node
   of the four-node network (`F-D.chance.from_hub.settles`), hub or tail.
   The familiar "proportional to the number of links" answer belongs to the
   discrete walk that takes one step per tick. The quiz says which hopper it
   means.
9. **Two Laplacians.** Heat on a network as e^{−tL} with L = D − A is
   Kondor and Lafferty's form (2002). Chung's *Spectral Graph Theory* uses
   the normalized Laplacian; it is not cited for this formula.
10. **Communicability counts walks, not paths**, with repeats allowed, each
    weighted by one over k factorial (Estrada and Hatano 2008). Their
    preprint prints eq. 2 as "= e^A" with no subscript on the right; the
    meaning is the (p, q) entry. Stanley's Theorem 1.1 is the source for
    "the entries of A^k count walks of length k".
11. **Tiny steps along a circle spiral outward.** (1 + iπ/4)⁴ lands at
    −2.32 + 1.20i, not at −1; a hundred steps reach −1.05 (`F-N.turn.steps.*`).
    The picture shows the overshoot shrinking; it does not pretend a few
    straight steps stay on the circle.
12. **θ is not available.** The engine at the workspace pin draws `\theta`
    as ϑ. The angle of a turn is the time t throughout; Video G agreed to
    the same letter.
13. **exp reaches every rotation but not one-to-one.** A full turn and no
    turn are the same motion (`F-L.full_turn_is_identity`). "Every rotation
    is the exponential of a skew matrix" holds because SO(n) is compact and
    connected (Hall 2015, Cor. 11.10, number from a secondary citation); it
    fails for groups such as SL(2, R), which the script does not mention.
14. **People.** Nicholas Higham died in January 2024. The research pass
    found one page reporting that Cleve Moler died in May 2026; it was not
    confirmed from a second source, and the script's wording does not
    depend on it either way (the paper is "Moler and Van Loan's", 1978).
15. **Every pronunciation is from a secondary source**, and four names
    (Moler, Van Loan, Hatano, Laguerre) have no written source at all. They
    are probed in the voice in P3; the owner's pronunciation overrides.

## Sources

Status: **P** primary text opened; **P\*** primary text in a non-final form
(preprint, transcription, translation, OCR of a reprint); **M** bibliographic
record only; **S** secondary only. Nothing here is committed to the
repository; the research notes and downloads live in the lane's scratch
folder.

| id | Source | Used for | Seen |
|---|---|---|---|
| S1 | C. Moler and C. Van Loan, "Nineteen Dubious Ways to Compute the Exponential of a Matrix", *SIAM Review* 20(4), 1978, 801–836. DOI 10.1137/1020098 | the title; Method 1's example (matrix, the correct e^A, the IBM result, K = 59, the sixteenth term); the verdicts in finding 3 | P (scan; p. 807 read as a page image) |
| S2 | the same authors, "…, Twenty-Five Years Later", *SIAM Review* 45(1), 2003, 3–49. DOI 10.1137/S00361445024180 | what changed by 2003 | P\* (pre-publication proof; no page numbers quoted) |
| S3 | N. J. Higham, "The Scaling and Squaring Method for the Matrix Exponential Revisited", *SIAM J. Matrix Anal. Appl.* 26(4), 2005, 1179–1193. DOI 10.1137/04061101X | Padé degrees 3, 5, 7, 9, 13; "the best choice of degree of Padé approximant turns out to be 13" | P |
| S4 | A. H. Al-Mohy and N. J. Higham, "A New Scaling and Squaring Algorithm for the Matrix Exponential", *SIAM J. Matrix Anal. Appl.* 31(3), 2009, 970–989. DOI 10.1137/09074721X | what SciPy's `expm` implements | P |
| S5 | SciPy manual, `scipy.linalg.expm`, and `scipy/linalg/src/_matfuncs_expm.c` | "implements the algorithm given in [Al-Mohy and Higham 2009]"; degrees {3, 5, 7, 9, 13} | P |
| S6 | MATLAB documentation, `expm` | cites Higham 2005 and Al-Mohy and Higham 2009 | P\* (read through a fetch tool, not the raw page) |
| S7 | E. Estrada and N. Hatano, "Communicability in complex networks", *Phys. Rev. E* 77, 036111 (2008). DOI 10.1103/PhysRevE.77.036111; arXiv:0707.0756 | the definition of communicability and its name | P\* (arXiv v1) |
| S8 | R. P. Stanley, *Algebraic Combinatorics* (Springer, 2013), Theorem 1.1 | the entries of A^k count walks of length k | P\* (author's posted version) |
| S9 | R. I. Kondor and J. Lafferty, "Diffusion Kernels on Graphs and Other Discrete Input Spaces", *Proc. ICML 2002*, 315–322 | heat on a network as the exponential of −L, L = D − A | P\* (author-posted PDF) |
| S10 | J. R. Norris, *Markov Chains* (Cambridge, 1997), §2.1 and Theorem 2.1.1 | P(t) = e^{tQ} for a finite chain | P\* (the theorem's number and role from the author's posted sections; its statement not seen) |
| S11 | B. Zwiebach, MIT 8.05 notes "Quantum Dynamics", eq. (2.45) | U(t) = exp(−iHt/ħ) for a time-independent H | P |
| S12 | L. Euler, *Introductio in analysin infinitorum*, vol. 1 (Lausanne, 1748), §§122, 123, 125, 138 | e to 23 places; the series; the limit; e^{v√−1} = cos v + √−1 sin v | P\* (I. Bruce's translation with the Latin) |
| S13 | Jacob Bernoulli, "Quaestiones nonnullae de usuris…", *Acta Eruditorum*, May 1690, 219–223 | the compound-interest question | P\* (OCR of the 1690 volume) |
| S14 | E. Laguerre, "Sur le calcul des systèmes linéaires", *J. École Polytechnique* cahier 42 (1867); *Œuvres* vol. 1 (1898), 221–267, p. 251 | e^X by the series; e^X e^Y ≠ e^{X+Y} in general | P\* (1898 reprint, page images) |
| S15 | G. Peano, "Intégration par séries des équations différentielles linéaires", *Math. Annalen* 32 (1888), 450–456 | x = e^{Rt}a for x′ = Rx | P\* (a transcription) |
| S16 | Sophus Lie: MacTutor and Wikipedia biographies; title pages of *Theorie der Transformationsgruppen* (Leipzig, 1888, 1890, 1893, with F. Engel) | 17 December 1842 – 18 February 1899, Norwegian; the treatise | S for the life; P\* for the title pages |
| S17 | B. C. Hall, *An Elementary Introduction to Groups and Representations*, arXiv:math-ph/0005032, and *Lie Groups, Lie Algebras, and Representations*, 2nd ed. (Springer, 2015), Cor. 11.10 | the first terms of the Baker–Campbell–Hausdorff formula, with signs; exp is onto for compact connected groups | P\* (the arXiv notes); S for the corollary's number |
| S18 | Particle Data Group, *Review of Particle Physics* (2026), §9 and §10 | SU(3) × SU(2) × U(1); eight gluons; W⁺, W⁻, Z and the photon | P |
| S19 | J. Baez and J. Huerta, "The Algebra of Grand Unified Theories", *Bull. AMS* 47 (2010), 483–552; D. Tong, "Line operators in the Standard Model", *JHEP* 07 (2017) 104 | the fine print of finding 7 | P\* (arXiv versions) |
| S20 | Campbell (1897, 1898), Baker (1902, 1905), Hausdorff (1906), Dynkin (1947); R. Achilles and A. Bonfiglioli, *Arch. Hist. Exact Sci.* 66 (2012), 295–358 | the names on the formula; that it has more parents (Schur, Poincaré, Pascal) | M and S (no original opened) |
| S21 | H. Padé, thesis, *Ann. sci. École Normale Supérieure* (3) 9 (1892), supplement 3–93 | the name; "made the first systematic study" (MacTutor), not "invented" | M |

Short quotations used on screen or in the description (each at most two
sentences, attributed): S1, "The computed approximation even has the wrong
sign in two components." and "Thus, we see perhaps three or four candidates
for 'best' method."; S14, "il est à remarquer qu'en général on n'aura pas
e^X · e^Y = e^{X+Y}".

## Simplifications and their exact versions

| The video says | Exactly |
|---|---|
| "its rate equals its size" | f′ = f, f(0) = 1; the only such function (if g′ = g too, then (g/f)′ = 0) |
| "e to the t A is where you end up" | x(t) = e^{tA}x(0) is the unique solution of x′ = Ax with that start |
| "the size of a matrix: the most it can stretch any arrow" | the operator 2-norm, max ‖Ax‖/‖x‖; any submultiplicative norm gives the same proof |
| "the series settles for every square matrix" | it converges absolutely, entry by entry, for every real or complex square matrix, since ‖A^k/k!‖ ≤ ‖A‖^k/k! and Σ‖A‖^k/k! = e^{‖A‖} |
| "when they commute, the rule holds; when they don't, it can fail" | AB = BA ⇒ e^{A+B} = e^A e^B; and e^{t(A+B)} = e^{tA}e^{tB} for all t ⇔ AB = BA; for short times the gap is ½t²[A, B] + O(t³) |
| "the first correction is half the commutator" | log(e^X e^Y) = X + Y + ½[X, Y] + (1/12)[X, [X, Y]] − (1/12)[Y, [X, Y]] + … (Baker–Campbell–Hausdorff; converges for small X, Y) |
| "the exponential of a skew matrix is a rotation, and every rotation comes from one" | exp maps so(n) onto SO(n); not one-to-one |
| "a Lie group is a smooth family of motions; its Lie algebra is its tiny steps" | a group that is also a smooth manifold with smooth multiplication and inverse; its Lie algebra is the tangent space at the identity with the bracket [X, Y] = XY − YX (for matrix groups) |
| "the forces of the Standard Model are built on SU(3) × SU(2) × U(1)" | a gauge theory with that Lie algebra; the global form of the group is the product modulo a subgroup of Z₆ not yet fixed by experiment |
| "a two-state quantum system" | ψ′ = −iHψ with ħ = 1 and H the Laplacian of two linked nodes; the chances are cos²t and sin²t |
| "equal chance at every node in the long run" | for the continuous-time walk with generator −L on a connected network; the discrete-time walk's stationary chances are proportional to the degrees |
| "Padé: a ratio of two polynomials that matches the series as far as it can" | the [m/m] approximant agrees with e^x through x^{2m}; error term (m!)²/((2m)!(2m+1)!) x^{2m+1} |

## The always sentences and their edge tests

| Sentence (as the script may say it) | Tested | Result |
|---|---|---|
| the series settles for every square matrix, and no term is larger than size(A)^k/k! | 200 seeded matrices, 41 terms each against the bound, the 120-term total against the exponential (`F-X.series_settles`; Rust test `edge_every_always_sentence_holds_on_every_case`) | 200 of 200 |
| the size of a product is at most the product of the sizes | 200 seeded pairs (`F-X.sizes_multiply`); the shear, where it is strict (2.414 < 2.618), and the nilpotent matrix, whose square is zero (`F-S.shear.*`, `F-S.nil.*`) | 200 of 200 |
| running for s and then for t is running for s + t | 200 seeded (A, s, t) (`F-X.flow_adds`) | 200 of 200 |
| if A and B commute, e^{A+B} = e^A e^B | 200 seeded commuting pairs (B a polynomial in A) (`F-X.commuting_rule_holds`) | 200 of 200 |
| if they do not commute it can fail | fails on all 200 seeded generic pairs (`F-X.generic_rule_fails`); but holds for the 3, 4, 5 turns (`F-O.turns.*`): "can", never "must" | 200 of 200 fail; 1 exception shown |
| the exponential of a skew matrix is a rotation | 200 seeded skew matrices of sizes 2, 3, 4: RᵀR = I and det R = 1 (`F-X.skew_gives_rotation`) | 200 of 200 |
| e to a matrix can always be undone (by e to minus it), and its determinant is e to the trace, never zero | 200 seeded matrices (`F-X.exponential_undone`) | 200 of 200 |
| heat on a network: the total never changes and nothing goes negative | both networks, 50 times each (`F-X.heat_conserves`) | 100 of 100 |
| the quantum pair's total chance stays one | 50 times (`F-X.quantum_total_one`) | 50 of 50 |
| the entries of A^k count the walks of length k | lengths 0 to 8 on the four-node network, by listing every walk (oracle cross-checks; Rust test `walk_counts_are_the_powers_of_the_adjacency_matrix`) | 9 of 9 lengths |

## Terms (the jargon ledger)

The line ids of the defining beats are filled in with the storyboard
(P2).

| Term (spoken) | Plain-language definition as spoken | On-screen label |
|---|---|---|
| rate | how fast something is changing right now | "rate"; f′ once, with "the derivative" in a lower-third |
| tiny step | what the rule does in a very short time | the teal arrow |
| matrix | a grid of numbers that turns one arrow into another | "matrix" |
| the state | the list of numbers that describes the system now | x(t) |
| the rule / the result | the matrix that says where each point is headed / where it ends up | A / e^{tA} |
| running total, settles | the sum so far; it stops changing in every digit you care to look at | "converges", said once |
| size (of a matrix) | the most it can stretch any arrow | "operator norm" in a lower-third |
| walk | a route along links, repeats allowed | "walk of length k" |
| commute | the order does not matter | AB = BA |
| the commutator | AB minus BA: what is left over when the order is swapped | [A, B] |
| skew | flip the grid across its diagonal and every sign changes | Aᵀ = −A |
| Lie group | a smooth family of motions | SO(3) |
| Lie algebra | its tiny steps | so(3) |

## The numbers

### F-N: the number exponential

`compound.nN`: one unit at 100% a year compounded N times, (1 + 1/N)^N. `series.kK`: 1 + 1 + 1/2! + … + 1/K!. `growth.*`: the quiz (5% a year for 20 years: rate times time is one, so the factor is e). `add.*`: e²·e³ = e⁵. `turn.t1`, `turn.pi`: e^{it} as (real part, imaginary part); `turn.steps.nN`: (1 + iπ/N)^N, N tiny steps of a half turn.

| key | value |
|---|---|
| `F-N.e` | 2.718282 |
| `F-N.e.digits15` | 2.718281828459045 |
| `F-N.compound.n1` | 2.00000 |
| `F-N.compound.n2` | 2.25000 |
| `F-N.compound.n4` | 2.44141 |
| `F-N.compound.n12` | 2.61304 |
| `F-N.compound.n365` | 2.71457 |
| `F-N.compound.n1000000` | 2.71828 |
| `F-N.series.k0` | 1.000000 |
| `F-N.series.k1` | 2.000000 |
| `F-N.series.k2` | 2.500000 |
| `F-N.series.k3` | 2.666667 |
| `F-N.series.k4` | 2.708333 |
| `F-N.series.k5` | 2.716667 |
| `F-N.series.k6` | 2.718056 |
| `F-N.series.k7` | 2.718254 |
| `F-N.series.k8` | 2.718279 |
| `F-N.series.k9` | 2.718282 |
| `F-N.series.k10` | 2.718282 |
| `F-N.factorials` | 1,1,2,6,24,120,720 |
| `F-N.growth.rate` | 0.05 |
| `F-N.growth.years` | 20 |
| `F-N.growth.factor` | 2.718282 |
| `F-N.growth.doubling_years` | 13.8629 |
| `F-N.add.e2` | 7.389056 |
| `F-N.add.e3` | 20.085537 |
| `F-N.add.product` | 148.413159 |
| `F-N.add.e5` | 148.413159 |
| `F-N.turn.t1` | 0.540302,0.841471 |
| `F-N.turn.pi` | -1.000000,0.000000 |
| `F-N.turn.steps.n4` | -2.3206,1.2037 |
| `F-N.turn.steps.n16` | -1.3524,0.0534 |
| `F-N.turn.steps.n100` | -1.0506,0.0011 |
| `F-N.turn.steps.n1000` | -1.0049,0.0000 |
| `F-N.turn.radian_degrees` | 57.2958 |

### F-H: the hook

`guess`: e raised to each entry. For the zero matrix the guess is four ones (determinant 0: it flattens the plane onto a line) where the answer must be the identity. `twice.*`: the guess applied twice against the guess for twice the matrix; the series passes the same test (`twice.true`).

| key | value |
|---|---|
| `F-H.zero.guess` | 1,1;1,1 |
| `F-H.zero.guess.det` | 0.000000 |
| `F-H.zero.true` | 1,0;0,1 |
| `F-H.J.guess` | 1.000000,0.367879;2.718282,1.000000 |
| `F-H.J.true` | 0.540302,-0.841471;0.841471,0.540302 |
| `F-H.twice.guess` | 2.000000,0.735759;5.436564,2.000000 |
| `F-H.twice.guess.doubled` | 1.000000,0.135335;7.389056,1.000000 |
| `F-H.twice.true` | true |

### F-J: the quarter turn J

Powers of J (rows separated by `;`), its period, and e^{tJ} at a quarter, a half and a full turn. `split.*`: the even terms of the series sum to cos 1 and the odd terms to sin 1.

| key | value |
|---|---|
| `F-J.power.k1` | 0,-1;1,0 |
| `F-J.power.k2` | -1,0;0,-1 |
| `F-J.power.k3` | 0,1;-1,0 |
| `F-J.power.k4` | 1,0;0,1 |
| `F-J.period` | 4 |
| `F-J.exp.quarter` | 0.000000,-1.000000;1.000000,0.000000 |
| `F-J.exp.half` | -1.000000,0.000000;0.000000,-1.000000 |
| `F-J.exp.full` | 1.000000,0.000000;0.000000,1.000000 |
| `F-J.split.even` | 0.540302 |
| `F-J.split.odd` | 0.841471 |

### F-S: does the series settle? (e to five quarter turns)

`terms`: the size of each term of the series for e^{5J}, k = 0 to 20 (exactly 5^k/k!). `total.kK`: where the running total sends the arrow (1, 0) after term K. `farthest.*`: the running total's greatest distance from the origin. `limit`: (cos 5, sin 5). `bound`: e⁵, the sum of all the sizes. `shear.*`, `nil.*`: the size of a product against the product of the sizes.

| key | value |
|---|---|
| `F-S.t` | 5 |
| `F-S.size` | 5.000000 |
| `F-S.terms` | 1.00000, 5.00000, 12.50000, 20.83333, 26.04167, 26.04167, 21.70139, 15.50099, 9.68812, 5.38229, 2.69114, 1.22325, 0.50969, 0.19603, 0.07001, 0.02334, 0.00729, 0.00214, 0.00060, 0.00016, 0.00004 |
| `F-S.peak` | 26.04167 |
| `F-S.peak.k` | 4,5 |
| `F-S.total.k0` | 1.0000,0.0000 |
| `F-S.total.k1` | 1.0000,5.0000 |
| `F-S.total.k2` | -11.5000,5.0000 |
| `F-S.total.k3` | -11.5000,-15.8333 |
| `F-S.total.k4` | 14.5417,-15.8333 |
| `F-S.total.k5` | 14.5417,10.2083 |
| `F-S.total.k6` | -7.1597,10.2083 |
| `F-S.total.k7` | -7.1597,-5.2927 |
| `F-S.total.k8` | 2.5284,-5.2927 |
| `F-S.total.k9` | 2.5284,0.0896 |
| `F-S.total.k10` | -0.1627,0.0896 |
| `F-S.total.k11` | -0.1627,-1.1336 |
| `F-S.total.k12` | 0.3469,-1.1336 |
| `F-S.total.k13` | 0.3469,-0.9376 |
| `F-S.total.k14` | 0.2769,-0.9376 |
| `F-S.total.k15` | 0.2769,-0.9609 |
| `F-S.total.k16` | 0.2842,-0.9609 |
| `F-S.total.k17` | 0.2842,-0.9588 |
| `F-S.total.k18` | 0.2836,-0.9588 |
| `F-S.total.k19` | 0.2836,-0.9589 |
| `F-S.total.k20` | 0.2837,-0.9589 |
| `F-S.farthest.k` | 4 |
| `F-S.farthest.radius` | 21.4978 |
| `F-S.limit` | 0.283662,-0.958924 |
| `F-S.bound` | 148.413159 |
| `F-S.first_below_thousandth.k` | 18 |
| `F-S.term.k20` | 0.0000392 |
| `F-S.shear.size` | 1.618034 |
| `F-S.shear.squared.size` | 2.414214 |
| `F-S.shear.size.squared` | 2.618034 |
| `F-S.nil.size` | 1.000000 |
| `F-S.nil.squared` | 0,0;0,0 |

### F-T: two tanks

100 litres each, 10 litres a minute each way, 100 g of salt in the first. `at.tT`: grams in each tank at minute T, e^{TA} applied to the start. `steps.nN`: the same ten minutes taken as N straight steps, (I + (10/N)A)^N. `within_a_gram.minutes`: when the first tank is within a gram of its final 50.

| key | value |
|---|---|
| `F-T.rule` | -0.1,0.1;0.1,-0.1 |
| `F-T.start` | 100,0 |
| `F-T.at.t1` | 90.9365,9.0635 |
| `F-T.at.t5` | 68.3940,31.6060 |
| `F-T.at.t10` | 56.7668,43.2332 |
| `F-T.at.t20` | 50.9158,49.0842 |
| `F-T.at.t60` | 50.0003,49.9997 |
| `F-T.steps.n1` | 0.0000,100.0000 |
| `F-T.steps.n2` | 50.0000,50.0000 |
| `F-T.steps.n5` | 53.8880,46.1120 |
| `F-T.steps.n10` | 55.3687,44.6313 |
| `F-T.steps.n100` | 56.6310,43.3690 |
| `F-T.steps.n1000` | 56.7532,43.2468 |
| `F-T.total` | 100.000000 |
| `F-T.settles` | 50.0000,50.0000 |
| `F-T.eigenvalues` | -0.200000,0.000000 |
| `F-T.within_a_gram.minutes` | 19.5601 |

### F-G: walks on a network

The four-node network (a triangle 1, 2, 3 with a tail 4). `walks.kK`: the number of walks of exactly K links between every pair (rows are start nodes). `exp`: e^A, the communicability of every pair. `exp.one_to_four.running`: node 1 to node 4, adding walks of length 0, 1, 2, … each divided by its factorial. `exp.trace`: the sum of the diagonal (the Estrada index).

| key | value |
|---|---|
| `F-G.nodes` | 4 |
| `F-G.links` | 1-2,1-3,2-3,3-4 |
| `F-G.degrees` | 2,2,3,1 |
| `F-G.walks.k1` | 0,1,1,0;1,0,1,0;1,1,0,1;0,0,1,0 |
| `F-G.walks.k2` | 2,1,1,1;1,2,1,1;1,1,3,0;1,1,0,1 |
| `F-G.walks.k3` | 2,3,4,1;3,2,4,1;4,4,2,3;1,1,3,0 |
| `F-G.walks.k4` | 7,6,6,4;6,7,6,4;6,6,11,2;4,4,2,3 |
| `F-G.exp` | 2.7830, 2.4151, 2.6215, 0.9155 ; 2.4151, 2.7830, 2.6215, 0.9155 ; 2.6215, 2.6215, 3.4921, 1.7059 ; 0.9155, 0.9155, 1.7059, 1.6611 |
| `F-G.exp.trace` | 10.7192 |
| `F-G.exp.one_to_four.running` | 0.0000, 0.0000, 0.5000, 0.6667, 0.8333, 0.8833, 0.9069, 0.9131, 0.9150 |
| `F-G.walks.one_to_four` | 0,0,1,1,4,6,17,31,76 |

### F-D: spreading (heat, chance, one letter i)

`L`: the Laplacian of the network. `heat.tT`: one unit of heat placed on node 4, after time T, at nodes 1 to 4 (a column of e^{−TL}). `chance.*`: the hopper's rule is −L; a row of e^{−tL} is where a hopper that started at the hub is likely to be. `pair.*`: two linked nodes; `pair.heat.tT` settles toward one half, `pair.quantum.tT` (e^{−iTL}, squared sizes) goes back and forth: all at the other node at time π/2 (`half`), all back at π (`full`).

| key | value |
|---|---|
| `F-D.L` | 2,-1,-1,0;-1,2,-1,0;-1,-1,3,-1;0,0,-1,1 |
| `F-D.heat.t0.25` | 0.0211,0.0211,0.1580,0.7999 |
| `F-D.heat.t0.5` | 0.0591,0.0591,0.2162,0.6656 |
| `F-D.heat.t1` | 0.1289,0.1289,0.2454,0.4968 |
| `F-D.heat.t2` | 0.2049,0.2049,0.2499,0.3403 |
| `F-D.heat.t5` | 0.2478,0.2478,0.2500,0.2545 |
| `F-D.heat.settles` | 0.2500,0.2500,0.2500,0.2500 |
| `F-D.chance.rows_sum_to_zero` | true |
| `F-D.chance.from_hub.t1` | 0.2454,0.2454,0.2637,0.2454 |
| `F-D.chance.from_hub.settles` | 0.2500,0.2500,0.2500,0.2500 |
| `F-D.pair.L` | 1,-1;-1,1 |
| `F-D.pair.is_the_tanks` | true |
| `F-D.pair.heat.t0.5` | 0.683940,0.316060 |
| `F-D.pair.quantum.t0.5` | 0.770151,0.229849 |
| `F-D.pair.heat.t1` | 0.567668,0.432332 |
| `F-D.pair.quantum.t1` | 0.291927,0.708073 |
| `F-D.pair.heat.t2` | 0.509158,0.490842 |
| `F-D.pair.quantum.t2` | 0.173178,0.826822 |
| `F-D.pair.quantum.quarter` | 0.500000,0.500000 |
| `F-D.pair.quantum.half` | 0.000000,1.000000 |
| `F-D.pair.quantum.full` | 1.000000,0.000000 |

### F-C: computing it

`mvl.*`: Moler and Van Loan's matrix. `exact`: e^A. `term.k16`, `term.k17`: the sixteenth and seventeenth terms of the series (millions, opposite signs). `f32.series`, `f32.terms`: the series summed in 32-bit arithmetic until a term changes nothing, and the number of that term. `f32.scaled.*`: the same 32 bits with the matrix halved eight times, eight terms, squared eight times. `pade.mM`: the [M/M] Padé approximant of e^x at x = 1. `diag.*`: the tanks' eigenvalues exponentiated. `nil.exp.t3`: a matrix that cannot be diagonalized, whose series ends by itself.

| key | value |
|---|---|
| `F-C.mvl.A` | -49,24;-64,31 |
| `F-C.mvl.eigenvalues` | -17,-1 |
| `F-C.mvl.exact` | -0.735759,0.551819;-1.471518,1.103638 |
| `F-C.mvl.term.k16` | 6977251.9, -3488626.0 ; 9303002.5, -4651501.3 |
| `F-C.mvl.term.k17` | -6977251.9, 3488626.0 ; -9303002.5, 4651501.3 |
| `F-C.mvl.term.largest` | 9303002.5 |
| `F-C.mvl.size` | 89.6324 |
| `F-C.mvl.f32.series` | -0.788594,1.314455;-0.126288,-0.839240 |
| `F-C.mvl.f32.terms` | 62 |
| `F-C.mvl.f32.worst_error` | 1.9429 |
| `F-C.mvl.f32.wrong_signs` | 1 |
| `F-C.mvl.f32.scaled` | -0.735775,0.551832;-1.471552,1.103665 |
| `F-C.mvl.f32.scaled.halvings` | 8 |
| `F-C.mvl.f32.scaled.divisor` | 256 |
| `F-C.mvl.f32.scaled.terms` | 8 |
| `F-C.mvl.f32.scaled.worst_error` | 0.000034 |
| `F-C.pade.m1` | 3/1 |
| `F-C.pade.m1.value` | 3.000000 |
| `F-C.pade.m2` | 19/7 |
| `F-C.pade.m2.value` | 2.714286 |
| `F-C.pade.m3` | 193/71 |
| `F-C.pade.m3.value` | 2.718310 |
| `F-C.pade.m4` | 2721/1001 |
| `F-C.pade.m4.value` | 2.718282 |
| `F-C.pade.m3.error` | 0.000028 |
| `F-C.series.k6.error` | 0.000226 |
| `F-C.pade.m3.times_closer` | 8.1 |
| `F-C.diag.tanks.exp_eigenvalues.t10` | 0.135335,1.000000 |
| `F-C.diag.tanks.rebuilt` | true |
| `F-C.nil.exp.t3` | 1,3;0,1 |

### F-O: order matters

A and B are the two shears. `gap.tT`: (e^{TA}e^{TB} − e^{T(A+B)}) divided by T²/2, which tends to the commutator. `commuting.*`: a pair that commutes. `turns.*`: three and four full turns about two axes: the rule holds without commuting.

| key | value |
|---|---|
| `F-O.A` | 0,1;0,0 |
| `F-O.B` | 0,0;1,0 |
| `F-O.expA` | 1,1;0,1 |
| `F-O.expB` | 1,0;1,1 |
| `F-O.expA_expB` | 2,1;1,1 |
| `F-O.expB_expA` | 1,1;1,2 |
| `F-O.exp_sum` | 1.543081,1.175201;1.175201,1.543081 |
| `F-O.AB` | 1,0;0,0 |
| `F-O.BA` | 0,0;0,1 |
| `F-O.commutator` | 1,0;0,-1 |
| `F-O.gap.t1` | 0.914,-0.350;-0.350,-1.086 |
| `F-O.gap.t0.1` | 0.999,-0.033;-0.033,-1.001 |
| `F-O.gap.t0.01` | 1.000,-0.003;-0.003,-1.000 |
| `F-O.commuting.commutator` | 0,0;0,0 |
| `F-O.commuting.rule_holds` | true |
| `F-O.turns.x_is_identity` | true |
| `F-O.turns.y_is_identity` | true |
| `F-O.turns.sum_is_identity` | true |
| `F-O.turns.sum_full_turns` | 5.000000 |
| `F-O.turns.commute` | false |

### F-L: the doorway to Lie theory

`bracket.*`: the three spins about the axes and their commutators. `loop.tT`: a small turn about x, then y, then back about x, then back about y, minus the identity, divided by T²: it tends to the spin about z. `spins.nN`: independent spins in N dimensions. `su3`, `su2`, `u1`: independent tiny steps of each group.

| key | value |
|---|---|
| `F-L.Lz` | 0,-1,0;1,0,0;0,0,0 |
| `F-L.bracket.xy` | 0,-1,0;1,0,0;0,0,0 |
| `F-L.bracket.xy_is_z` | true |
| `F-L.bracket.yz_is_x` | true |
| `F-L.bracket.zx_is_y` | true |
| `F-L.exp.Lz.t1` | 0.540302, -0.841471, 0.000000 ; 0.841471, 0.540302, 0.000000 ; 0.000000, 0.000000, 1.000000 |
| `F-L.loop.t0.1` | -0.005, -0.997, -0.050 ; 0.997, -0.005, -0.050 ; 0.050, 0.050, 0.000 |
| `F-L.loop.t0.01` | 0.000, -1.000, -0.005 ; 1.000, 0.000, -0.005 ; 0.005, 0.005, 0.000 |
| `F-L.spins.n2` | 1 |
| `F-L.spins.n3` | 3 |
| `F-L.spins.n4` | 6 |
| `F-L.spins.n5` | 10 |
| `F-L.su3` | 8 |
| `F-L.su2` | 3 |
| `F-L.u1` | 1 |
| `F-L.standard_model.steps` | 12 |
| `F-L.full_turn_is_identity` | true |

### F-X: the edges of the always sentences

How many of the seeded cases satisfy each sentence, of how many checked (see the table above).

| key | value |
|---|---|
| `F-X.seed` | 20261013 |
| `F-X.series_settles` | 200 of 200 |
| `F-X.sizes_multiply` | 200 of 200 |
| `F-X.flow_adds` | 200 of 200 |
| `F-X.commuting_rule_holds` | 200 of 200 |
| `F-X.generic_rule_fails` | 200 of 200 |
| `F-X.skew_gives_rotation` | 200 of 200 |
| `F-X.exponential_undone` | 200 of 200 |
| `F-X.heat_conserves` | 100 of 100 |
| `F-X.quantum_total_one` | 50 of 50 |

### F-B: cited literals

Typed from the sources (S-numbers above); the video shows them only as quotations.

| key | value |
|---|---|
| `F-B.mvl.year` | 1978 |
| `F-B.mvl.revisit_year` | 2003 |
| `F-B.mvl.ways` | 19 |
| `F-B.mvl.ibm.result` | -22.25880,-1.432766;-61.49931,-3.474280 |
| `F-B.mvl.ibm.terms` | 59 |
| `F-B.pade.degrees` | 3,5,7,9,13 |
| `F-B.higham.year` | 2005 |
| `F-B.al_mohy_higham.year` | 2009 |
| `F-B.estrada_hatano.year` | 2008 |
| `F-B.euler.year` | 1748 |
| `F-B.euler.e` | 2.71828182845904523536028 |
| `F-B.bernoulli.year` | 1690 |
| `F-B.laguerre.year` | 1867 |
| `F-B.peano.year` | 1888 |
| `F-B.lie.born` | 1842 |
| `F-B.lie.died` | 1899 |
| `F-B.bosons` | 8,2,1,1 |
