# Proposed next slate

Status: editorial recommendation under review; **not yet a completed cross-model duel or owner-selected production slate**. Claude's fresh local instance still encountered its weekly quota on the first request. Codex's independent thirty concepts are frozen in `WIZARD_IDEAS_CODEX.md`; second-family generation, cross-scores and reveal remain pending. The ordering below is the lead's practical production judgment, not predicted view counts.

## Recommended small batch

| Order | Working title | Why it fits | What makes this treatment worth making | Gate before committing |
|---|---|---|---|---|
| 1, pilot | **Two Bad Echoes Can Make One Perfect Measurement** | Concrete surprise, tiny exact example, serious payoff; suggested in the X replies. | Derive Golay complementary-pair cancellation and the recursion on screen, then show the precise common-channel assumption and how motion breaks it. | Full competition viewing, proof storyboard and an intelligible stationary echo model. “Perfect” refers only to the ideal modeled measurement. |
| 2 | **Why Turning a Square Can Make the Whole Box Smaller** | Jeff explicitly liked square packing; visual-proof audience fit. | Keep five physical tiles on screen and explain exact construction versus local rigidity versus global optimality. Recent work can be a carefully qualified coda. | Reconstruct the chosen lower-bound proof. If it cannot be taught honestly, retitle as construction-versus-proof or choose another theorem. |
| 3 | **Why P-Values Confuse Even Doctors** (working title) | Explicit owner direction: doctors survey, medical diagnosis, and why the reversal fools smart people. | Reconstruct the survey controversy compassionately; use natural-frequency medical examples to separate conditional directions, then derive a study p-value. | Verify the survey wording and population; diagnostic rates are not p-values. Find a distinct complete explanation without crowding it with optional stopping. |
| 4, reserve until coverage review | **How to Learn a Crowd's Secret Without Learning Each Person's Answer** | Fish-like aggregate inference with an appealing paradox. | Derive randomized response, its uncertainty cost and the repeated-answer limitation; the mathematical privacy promise is explicit. | Existing accessible explanations already exist. Variance/privacy composition must supply a material distinct payoff. |
| 5, reserve until coverage review | **Two Shadows Can Hide Different Worlds** | Visual puzzle grows into a real inverse problem. | Show identical row/column sums, derive switching ambiguity, and distinguish feasible reconstruction from unique reconstruction. | Complete competing-video review and an honest small tomography theorem; do not imply arbitrary imaging is binary line sums. |

The first batch should initially mean **one completed pilot plus two qualified briefs**, not five simultaneous full films. Research and small visual tests can run in parallel while the voice comparison is unresolved. This protects the expensive work from inheriting a narration or explanation defect.

## Concrete story attacks

### Golay: let the viewer discover cancellation

Open with two nearby targets and a long coded pulse that creates misleading peaks. Slide four signed blocks against a copy, showing every product contributing to a peak. Neither first correlation is clean. Add the second and watch nonzero-lag terms cancel. Keep that exact example through the recursion, measurement equation and failure case. A brief sound demonstration can reinforce the mechanism, but the film must work with sound off.

Independent small calculations reproduced the nonnegative-lag sum `[8,0,0,0]` for `(1,1,1,-1)` and `(1,1,-1,1)`; the receipt is `.mor/run-20261016T200000Z/repro/toy-math-check.json`. That is a toy arithmetic check, not the completed film's proof audit. [Chi, Pezeshki and Calderbank's chapter](https://users.ece.cmu.edu/~yuejiec/papers/Golaybkch_final.pdf) defines the cancellation and explains its sensitivity to Doppler. The practical limitation belongs in the central argument, not a buried disclaimer.

### Packing: explain the awkward tile, not just a record gallery

Let the audience try a grid. Rotate the middle square: its own bounding box grows, yet the joint arrangement can shrink because empty regions interlock. Derive exact contacts. Then switch the question from “does this fit?” to “could anything smaller fit?” Visually distinguish a collision oracle, an optimizer and a proof that excludes all alternatives.

The [maintainer's five-square case](https://jlevy.github.io/squares/cases/5.html) provides the exact construction and proof references; the [tutorial](https://jlevy.github.io/squares/tutorial.html) separates proof and verification status. We have not independently certified all its current frontier results. Jeff's expressed interest in recent square-packing work does not automatically endorse replacing it with a historical five-square episode; selection should make that scope explicit.

### P-values: a question that changes direction

The owner requests a medical opening. Use the 2007 Windish/Huot/Green survey of internal-medicine residents in Connecticut, then the later criticism of its answer key. Do not present it as a representative survey of all US doctors or claim a majority selected a particular distractor. The drama is that even the question and key deserve scrutiny. Explain why the desired question—how credible a hypothesis is after seeing evidence—is so easily substituted for the one a p-value calculates.

Use synthetic patient counts to show the change of denominator, explicitly distinguishing sensitivity, false-positive rate and positive predictive value from a study p-value. Then build the study null world, count outcomes at least as extreme, and connect back to effect size, uncertainty and a nonmedical test. Exact sources, arithmetic and story beats are in [the medical opening brief](evidence/pvalues-medical-opening.md).

### Optional stopping: a possible sequel, not the required medical core

Start with the same fair-coin tree used to define the tail probability. Highlight every path captured when significance is checked at 20, 40, …, 200 tosses; captured paths cannot be counted twice. The independently recomputed one-sided exact model rejects about 3.84% of fair-coin experiments at one look at 200, versus 13.91% when stopping at the first of those ten rejecting looks. These values are specific to the stated discrete test, not universal p-hacking rates.

Use the same paths for a capital process that multiplies by 1.5 on heads and 0.5 on tails. Explain why its fair-coin conditional expectation stays unchanged and why crossing 20 has probability at most 1/20. This gives the viewer a constructive repair. The [ASA statement](https://www.amstat.org/asa/files/pdfs/p-valuestatement.pdf) supports interpretation boundaries; [Howard and colleagues](https://arxiv.org/abs/1810.08240) provide the broader research bridge. The full story still needs a production proof review.

## Strong alternatives worth keeping

- **Good–Turing missing mass:** infer the probability that the next observation is new from singleton counts. A strong fish sequel if it clearly separates unseen probability mass from unseen population size. [Good's original paper](https://academic.oup.com/biomet/article-abstract/40/3-4/237/195998).
- **Electrical networks and random walks:** a voltage solves the same boundary-value problem as a hitting probability. It offers a satisfying complete bridge rather than another generic linear-algebra tour. [Doyle and Snell](https://math.dartmouth.edu/~doyle/docs/walkspdf/walks.pdf).
- **Rotor-router walks:** deterministic local arrows reproduce some effects of randomness; choose a precise small theorem instead of showing attractive patterns. [Propp's paper](https://arxiv.org/abs/1007.2389).
- **Rigidity and the pebble game:** why enough bars can still leave a structure floppy; explicitly qualify generic planar position. This differs from the existing vibration/eigenvalue film. [Jacobs and Hendrickson](https://people.engr.tamu.edu/ajiang/PebbleGame.pdf).
- **Freivalds verification:** check a matrix product much more cheaply than recomputing it, with an exact probability argument. Keep exact arithmetic separate from floating-point stability. [Sinclair's lecture](https://people.eecs.berkeley.edu/~sinclair/cs271/n1.pdf).

These are research footholds, not five additional commitments or claims of sparse YouTube coverage.

## Ideas to park, and why

Generic Fourier/FFT, Monty Hall, Benford, Nash and broad complex-analysis introductions need a sharper gap before selection. Group testing looked promising but the bounded research found a [close animated treatment](https://vitchakorn.com/project/group-testing/) already covering pooling optimization and splitting; it was downgraded for a concrete reason. Hopf/spinors and the recent AI mathematical solves risk a large prerequisite burden before an earned payoff. The submarine suggestion was already made into a video. Agent workflow/how-I-build belongs on Jeff's other channel.

Do not mistake this for rejecting those subjects forever. A distinct theorem, example or application can reopen a candidate. Preserve the original suggestion, owner reaction, comparison links and rejection reason in its record.

## What the audience evidence changes

The recurring voice criticism is not simply a preference against a British narrator: one viewer explicitly likes it, while others describe changes of identity/accent and between-line discontinuity. Preserve the intended Michael voice and test stability. Correct word transcription is necessary but insufficient. The separate “incoherent” and “not intuitive” comments require an argument/comprehension audit alongside audio work. [Source-linked audience capture](evidence/audience.md).

The four reported winners suggest a useful prior: start from a concrete object, expose a surprising failure of intuition, and derive an idea the viewer can reuse. Views and likes alone do not establish that causal story, especially with different promotion and upload ages. No claim that this proposed ordering maximizes expected views has been established.
