# The boundary trick behind Stokes' theorem

Status: owner-requested idea and working treatment, 2026-11-02. A [research narration and visual draft](stokes-theorem-script-storyboard-20261102-draft01.md) now exists; research and planning only. No approved production script, ElevenLabs recording, native render, independent mathematics/pedagogy review, continuous listening or publication is claimed. Related: `complex_analysis` (contour integrals and residues), `abstract_algebra_visual` (structure and maps), and `exponential_magic` (physical rate laws).

## Promise and audience

A curious calculus learner should finish able to see why local contributions cancel across shared boundaries, distinguish a two-dimensional surface in 3D from a three-dimensional volume, follow the actual box proof in arbitrary dimension, and recognize what additional ingredients extend that proof to smooth manifolds. An engineering learner should see why the same relationship connects a changing magnetic field to circulation around a loop. Avoid suggesting that a boundary reconstructs every interior detail: opposite local contributions may cancel.

Working title: **The Boundary Trick Behind Stokes' Theorem**. Opening challenge: adjacent moving tiles each have a circulating edge; predict which edges survive when we add their signed circulations. Keep the answer hidden until after the spoken question and an actual pause.

## Proposed visual arc

1. **One dimension: cancellation you can count.** Cut `[a,b]` into signed segments. Each interior endpoint occurs once positive and once negative; the remaining boundary is `b − a`. Show `∫ₐᵇ f′(x) dx = f(b) − f(a)`, including an arrow reversal. This is the fundamental theorem, not an analogy to it.
2. **Two dimensions: oriented edges.** Join two squares, then a grid: shared edges are traversed in opposite directions. Introduce a 1-form as a rule that assigns signed accumulation to an oriented small step, not merely a decorative arrow. Concrete example `ω = x dy` on `[0,1]²`: `dω = dx∧dy`; the positively oriented boundary gives 1 from the right edge and 0 from the other three, matching area 1. Show why swapping orientation changes signs.
3. **Surface in 3D: bending without losing the boundary.** Bend a disk into a cap while keeping its rim, tile the cap and cancel interior seams. A surface is *intrinsically 2D* even when embedded in 3D. With `F=(-y/2,x/2,0)`, `curl F=(0,0,1)`, and the unit circle oriented counterclockwise from above, both the rim circulation and the upward curl flux through a smooth paraboloid cap over the unit disk equal `π`. Show the normal and rim orientation linked by the right-hand rule, and compare a flat disk with a curved cap. Equality concerns the integral of curl for this common oriented boundary, not equality of arbitrary surface fluxes.
4. **A genuine 3D region: flux through a closed surface.** Use a volume made from cubes, with outward normals on shared faces canceling. For `F=(x,y,z)/3` on the unit ball, `div F=1`, and both the volume integral and outward surface flux are `4π/3`. This is the divergence theorem as a 3-form version of the same statement, not the same geometry as the 2D cap.
5. **A hole is a boundary too.** On the annulus `1≤r≤2`, use `α=(-y dx+x dy)/(x²+y²)`. It is smooth on the annulus and `dα=0` there. The outer boundary contributes `+2π` and the inner boundary, with induced clockwise orientation, `−2π`. A loop around the origin cannot be filled by a disk on which this particular form is smooth. The apparent contradiction is a missing hypothesis/boundary, not a failure of Stokes.
6. **Build the general language.** A `k`-form is integrated on an oriented `k`-dimensional object; its exterior derivative is a `(k+1)`-form. A compact oriented smooth `n`-manifold `M` with boundary and a smooth `(n−1)`-form `ω` satisfy `∫_M dω = ∫_{∂M} ω`, with the boundary orientation induced from `M`. Explain that intrinsic dimension is what counts. Noncompact cases need appropriate support/convergence hypotheses; nonorientable domains and singularities require different treatment. [MIT theorem and dimensional cases](https://math.mit.edu/~dyatlov/18.155/155-notes.pdf#page=237).
7. **The proof, first on an n-dimensional box.** With standard orientation, write
   `ω = Σᵢ (-1)^(i−1) fᵢ dx¹∧...∧omit(dxⁱ)∧...∧dxⁿ`.
   Alternating wedge products kill every term except the derivative in the missing coordinate, so `dω = (Σᵢ ∂ᵢfᵢ) dx¹∧...∧dxⁿ`. Integrate each term using Fubini and the ordinary one-variable fundamental theorem: upper-face contribution minus lower-face contribution. The induced outward-first boundary orientation gives exactly these signs. This is an actual proof for a box, under the stated smoothness and standard orientation, rather than a slogan about tiny tiles.
8. **From a box to manifolds, without hiding the hard step.** Explain the additional standard tools: orientation-preserving coordinate charts (interior or half-space near a boundary), pullback commuting with `d`, and a partition of unity. Multiply `ω` by chart-supported weights summing to one; apply the local coordinate calculation to each piece and sum. Linearity and `Σρᵢ=1` reassemble both integrals; the derivative-of-weight terms cancel because `d(Σρᵢ)=0`. Boundary orientations must agree across charts. Name these as required lemmas and offer a detailed appendix or companion, rather than portraying the tile picture alone as a complete proof for arbitrary manifolds. [Stanford proof, §10.2](https://math.stanford.edu/~eliash/Public/177-2015/52htext-2015.pdf#page=171); [Cornell notes on partitions and integration](https://pi.math.cornell.edu/~sjamaar/manifolds/manifold.pdf).
9. **Why engineers care.** For a fixed, stationary loop bounding a fixed surface, Faraday's law relates `∮ E·dl` to `−d/dt ∫ B·n dS`; Stokes turns the circulation into the surface integral of `curl E`, giving the local Maxwell–Faraday law under the usual smoothness assumptions and for arbitrary suitable surfaces. Changing the surface in time needs additional care and should not be silently folded into this fixed-loop derivation. Animate a changing field through a loop and the induced circulation. [OpenStax derivation and fixed-surface caveat](https://openstax.org/books/calculus-volume-3/pages/6-7-stokes-theorem).

## Visual and proof risks to resolve

- Show consistent orientation throughout; separately demonstrate that reversing `M` reverses the induced boundary orientation and both integrals.
- Use a real boundary chart and a partition-of-unity diagram for the general proof. Canceling a finite grid alone is not a full manifold proof.
- Distinguish a smooth manifold with boundary from a box with corners; state the piecewise-smooth/with-corners extension used for box examples.
- Put the singularity outside the annulus domain, retain the inner boundary, and never apply the disk theorem through the undefined origin.
- Do not imply `∂²=0` alone proves Stokes; explain its relationship to `d²=0` and to boundary orientation, with the needed compatibility assumptions.
- Show discrete oriented tile boundaries on the same visual objects as their continuous limits; check cancellation and rounding with an independent exact oracle before narration.
- The proof on a general smooth manifold invokes standard chart, change-of-variables and partition-of-unity lemmas. Either prove them in a companion or clearly identify them as inputs; do not call a brief sketch a fully self-contained proof.

## Bounded author checks

`python3 -I docs/video-program/research-20261016/evidence/stokes-examples-check-20261102.py` exited 0 (SHA-256 `7c50c9cda49664f4dcba69373b7dae4ebf01308d99a52b051471b9832a63e486`). Exact rational arithmetic checked 25 signed-square boundary cancellations and integral 1, the wedge-sign and affine-box identities for n=1 through 8, and sampled rational algebra for the paraboloid, circle and annulus. The unit-sphere and integral-of-a-constant geometry is cited/derived in the treatment, not independently proven by that script. These finite author checks are not a proof for arbitrary forms/manifolds, a scene test or independent mathematical review.

`stokes-boundary-chain-check-20261102.py` exited 0 (SHA-256 `d451784aecb40f9285ff7b07a7a4b2879c8469c08456774295bb04be80eb783a`): it checked boundary-of-boundary cancellation for 7,537 oriented cubical cells in ambient dimensions 2 through 8. This finite check does not prove the general statement, test `d²=0`, or constitute independent review.

## Existing-treatment leads; comparison not complete

- [Khan Academy, Stokes' theorem intuition](https://www.khanacademy.org/math/multivariable-calculus/greens-theorem-and-stokes-theorem/stokes-theorem/v/stokes-theorem-intuition): creator description says it connects curl on a surface with circulation at the boundary. The search result exposed an introductory transcript excerpt; no complete viewing or listening is claimed.
- [The Bright Side of Mathematics, Manifolds 52](https://thebrightsideofmathematics.com/courses/manifolds/mf52_info/): creator page identifies a generalized-Stokes theorem video and downloadable notes in a manifolds course. Its treatment has not been viewed end-to-end.
- [Generalised Stokes' Theorem, Differential forms #3](https://www.youtube.com/watch?v=6B9eWS1GCR0): search listing identifies a relevant differential-forms video; its watch page returned rate limiting during this pass. No viewing or content comparison is claimed.
- [Alan Parry, The Generalized Stokes Theorem](https://www.youtube.com/watch?v=8-kH8LPtjjo): the listing describes a unified differential-forms treatment; [his university profile](https://www.uvu.edu/directory/employee/?id=K0V1MkpxaEdpUENmVUd6dk1hT0NxZz09) confirms attribution. Four paused frames were inspected, showing a classroom whiteboard and dimension, fundamental-theorem and Green's-theorem material; see [bounded comparison](stokes-competitor-parry-sampled-20261102.md). No full viewing or listening, or claim about unobserved portions, is made.

Candidate distinction, still a hypothesis: one *continuous* signed-cancellation story from 1D through both surface and volume cases, an explicit box proof in arbitrary dimension, a truthful chart/partition-of-unity bridge, an annulus counterexample and a fixed-loop physics payoff. Check actual treatments before claiming novelty.

## Next work

- [ ] Independently check the dimension, signs, example calculations, regularity and chart-proof account.
- [ ] Review competing videos end-to-end, document exact overlaps and strengthen the distinctive treatment.
- [ ] Review and revise the research narration/storyboard draft; prepare an independent example oracle and native FrankenManim scene designs, protecting accessibility and mobile legibility.
- [ ] Use the owner's current ElevenLabs narration direction, preserving complete takes; actual listening, take selection and fine synchronization are required.
- [ ] Validate native renders, captions, mathematical teaching, motion, sound and final output; retain independent review as open until it actually occurs.
