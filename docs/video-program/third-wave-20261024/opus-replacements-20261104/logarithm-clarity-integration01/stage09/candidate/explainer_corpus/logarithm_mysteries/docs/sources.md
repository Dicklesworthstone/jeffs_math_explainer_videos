# logarithm_mysteries: sources behind the on-screen claims

Bead `mx-video-program-program-87do.63`. RubyRaven, 2026-11-04, after root's
cross-review (Mail 803). This file covers what each sourced screen line rests on,
what was actually read, and the gaps that remain.

**Reading level, stated plainly:**

- **Read in full text:** the Higham chapter and the Gallier paper (sections 1 and 2).
- **Not read:** Culver's paper and Higham's 2008 book. Their theorem numbers appear
  here only as other authors cite them.

## 1. Higham, "Functions of Matrices" (Handbook of Linear Algebra chapter)

N. J. Higham, *Functions of Matrices*, MIMS EPrint 2005.21, Manchester Institute for
Mathematical Sciences, 2006. This is the chapter written for the Handbook of Linear
Algebra.

- Record: <https://eprints.maths.manchester.ac.uk/1067/>
- Read: the PDF on that record, section 4 "Matrix Logarithm".

Section 4, quoted from the text:

> Let A ∈ Cn×n. Any X such that e^X = A is a logarithm of A.
>
> 1. If A has no eigenvalues on R− then there is a unique logarithm X of A all of whose
>    eigenvalues lie in the strip { z : −π < Im(z) < π }. This is the principal
>    logarithm of A, and is written X = log A. If A is real then log A is real.
>
> 3. A ∈ Rn×n has a real logarithm if and only if A is nonsingular and A has an even
>    number of Jordan blocks of each size for every negative eigenvalue.
>
> 5. If A ∈ Cn×n is nonsingular then det(A) = exp(tr(log A)), where log A is any
>    logarithm of A.

Where the film uses it:

| cue | screen claim | rests on |
|---|---|---|
| 10_4 | the principal matrix logarithm: no eigenvalue on the closed negative axis ⇒ one log with eigenvalue angles in (−π, π), real when A is real | Fact 1 (and Gallier Thm 3.11(a) for the real case, below) |
| 11_20 | "A real matrix has a real logarithm exactly when it is invertible and, for every negative eigenvalue, its Jordan blocks of each size occur an even number of times." | Fact 3, word for word in substance |
| 11_7 | 2×2: a direction sent to a negative multiple gives a real log only if A = −cI, c > 0 | Fact 3, specialised (see the derivation below) |
| 11_8 | singular matrices have no logarithm: det e^X = e^(tr X) ≠ 0 | Fact 5's identity, read for X; the series argument is standard |
| 12_21 | "eigenvalue −1, so no principal log (Higham §4 Fact 1)" | Fact 1's definition, plus one step (below) |

**Fact 1's R−.** At the square-root fact, the text writes R−₀ and glosses it as
"the closed negative real axis". At the logarithm fact, the extracted text writes R−.
Zero cannot matter there, because a logarithm needs A nonsingular (Fact 5). The film says
"no eigenvalue on the negative real axis" for invertible A, which is the same condition.

**The 12_21 step.** Fact 1 defines the principal logarithm as the logarithm whose
eigenvalues all lie in the open strip. If A has eigenvalue −1, every logarithm X has an
eigenvalue μ with e^μ = −1. So Im μ is an odd multiple of π, and μ is never inside the
open strip. Hence no logarithm meets Fact 1's definition.

The film shows the same thing directly: the two slowest spins, ±π, tie on the window's
open edge. `the_two_slowest_spins_tie_so_no_principal_choice_exists` checks it.

**The 2×2 rule from Fact 3.** Take a real 2×2 matrix A with a negative eigenvalue λ.

| A's eigenvalues and blocks | count for λ | real log? |
|---|---|---|
| one negative, one positive | one 1×1 block | odd: no |
| two distinct negatives | one 1×1 block each | odd: no |
| a repeated λ in a single 2×2 Jordan block | one 2×2 block | odd: no |
| a repeated λ in two 1×1 blocks, A = λI = −cI | two 1×1 blocks | even: yes |

The yes case is reached by the spin, as 11_7 shows: e^{πJ} = −I, scaled by ln c.

## 2. Gallier, "Logarithms and Square Roots of Real Matrices"

J. Gallier, *Logarithms and Square Roots of Real Matrices: Existence, Uniqueness, and
Applications in Medical Imaging*, arXiv:0805.0245v2 [math.GM], 9 Nov 2013.

- <https://arxiv.org/abs/0805.0245>
- Read: the arXiv PDF, theorem statements, proofs' outlines and remarks.

Quoted:

> **Theorem 3.3** Every n × n invertible complex matrix, A, has a logarithm, X. To find
> such a logarithm, we can proceed as follows: (1) Compute a Jordan form, A = PJP⁻¹ …

> **Theorem 3.4** Let A be a real n × n matrix and let … J_{r1}(α1), …, J_{rm}(αm) be its
> list of Jordan blocks. Then, A has a real logarithm iff A is invertible and if, for
> every ri and every real eigenvalue αi < 0, the number, mi, of Jordan blocks identical
> to J_{ri}(αi) is even.

> **Definition 3.7** Let S(n) denote the set of all real matrices whose eigenvalues,
> λ + iµ, lie in the horizontal strip determined by the condition −π < µ < π.
>
> **Theorem 3.11 (a)** If A is any real invertible n×n matrix and A has no negative
> eigenvalues, then A has a unique real logarithm, X, with X ∈ S(n).

Gallier's remarks bear on the citations:

- He finds Theorem 3.4 first in Culver (1966, his ref. [12]), whose proofs "rely heavily
  on results from Gantmacher". He states it also appears in Horn and Johnson, *Topics in
  Matrix Analysis*, as Theorem 6.4.15, with the proof left as an exercise. He gives his
  own proof.
- He says Theorem 3.11 "also appears in Higham's book [18] as Theorem 1.31", and first
  appeared, without proof, as Lemma A2 of Kenney and Laub.

Where the film uses it:

| cue | screen line | rests on |
|---|---|---|
| 10_4 | precisely line ends "(Higham; Gallier Thm 3.11)" | Thm 3.11(a), the real case of the principal log |
| 11_20 | "Culver, Proc. AMS 17 (1966) 1146–1151, as stated by Higham (Handbook of Linear Algebra) and Gallier (Thm 3.4)" | Thm 3.4 and Higham Fact 3 agree; that is what the line claims |
| 11_8 | "Gallier, arXiv 0805.0245, Theorem 3.3, proved via the Jordan form" | Thm 3.3 |

Round 1 cited "Higham Thm 1.27" for 11_8 and "Higham Thm 1.31" for 10_4. Neither book
theorem was read. 11_8 now cites Gallier Thm 3.3, which was read. 10_4 cites Higham's
EPrint and Gallier Thm 3.11. "Thm 1.31" survives only as Gallier's cross-reference,
reported above.

## 3. Culver (1966): cited, not read

W. J. Culver, "On the existence and uniqueness of the real logarithm of a matrix",
Proc. Amer. Math. Soc. 17 (1966) 1146–1151. The bibliographic data comes from Gallier's
reference [12]. The DOI 10.1090/S0002-9939-1966-0202740-6 was carried from round 1's
secondary sources and has not been resolved here.

Both download attempts returned the AMS "Verifying your connection" page, not the paper.
So the screen line names Culver only "as stated by" Higham and Gallier, which is exactly
what was checked.

**Gap:** a reader with AMS access should confirm that the theorem in the paper matches
Fact 3 / Thm 3.4. Gallier's remark that Culver also characterises *unique* real logarithms
(a stronger condition, all eigenvalues real and positive) is not used on screen.

## 4. Claims shown by derivation and tested, not cited

| cue | claim | where checked |
|---|---|---|
| 11_4 | X commutes with e^X: X·X^k = X^(k+1) = X^k·X, term by term | `x_and_its_own_exponential_end_both_routes_together` (X·X^k = X^k·X for k = 0…11, and both routes end together on u) |
| 11_5 | (XA − AX)u = (b(a₂ − a₁)u₂, c(a₁ − a₂)u₁); with A = diag(−½, −2) each part is ±1.5 × an off-diagonal entry × a part of u | `each_part_of_the_gap_is_an_off_diagonal_entry_times_the_stretch_difference` (down to off-diagonal scale 10⁻⁶); `the_ends_meet_once_the_off_diagonal_entries_are_zero` |
| 11_6 | a diagonal real X stretches each axis by e^(tp) > 0, so no axis reverses | `anything_commuting_with_the_flip_pair_is_diagonal`, `a_diagonal_flow_never_flips_an_axis` |
| 11_1, 09_1 | ½(I + mirror) has determinant 0: the average of the endpoints is singular | `the_plain_average_of_identity_and_mirror_is_flat`; fact `matrix.mirror_average_det` (oracle) |
| 12_4–12_21 | the spins about the rod that end at the half turn are ±½, ±1½, … laps per unit time; no other axis works; the slowest two tie | `a_half_turn_is_reached_by_odd_half_turns_either_way_about_the_rod`, `the_shown_rates_all_end_in_the_half_turn`, `a_spin_about_any_other_axis_never_lands_on_the_half_turn`, `the_two_slowest_spins_tie_so_no_principal_choice_exists` |
| 06_6 | the principal candidate's gap is 0 up to −1 and −1 just after | `the_principal_candidate_jumps_a_lap_just_after_minus_one_and_only_there`; fact `loop.principal_gap` |
| 07_2 | Log i = πi/2, Log(−i) = −πi/2 | fact `branch.log_i_pi`; `log_of_i_and_minus_i_are_a_quarter_turn_either_way` |
| 08_3 | for z at 3π/4, Log(z²) = 2 Log z + 2πi·k with k = −1; generally k ∈ {−1, 0, 1} | `the_square_drops_exactly_one_lap_so_k_is_minus_one`; the range follows from Im Log ∈ (−π, π] |
| 11_2 | det e^(tX) = e^(t tr X) never reaches 0 (Liouville) | facts `matrix.flow_det_t1`, `matrix.flow_det_t3`, `matrix.flow_trace`; `signed_area_along_a_flow_is_e_to_trace_times_t_at_the_shown_times`; standard, not cited to a text |

## 5. Carried from round 1, not re-read in this round

- **DLMF §4.2** (<https://dlmf.nist.gov/4.2>) for chapter 07's domain statements:
  Log holomorphic off (−∞, 0]; with Arg = π on the negative axis, Im Log fills (−π, π].
- **ISO 16** (A4 = 440 Hz) for chapter 00.
