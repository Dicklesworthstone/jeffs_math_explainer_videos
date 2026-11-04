# Two sister videos: exponential and logarithm

Owner request, 2026-11-01. These are working scripts and picture plans for two requested videos, not finished videos. The first is introductory; the second begins as its inverse and continues into complex numbers and matrices. Current narration provider is ElevenLabs; no particular voice is imposed by this brief. The existing channel video *What Does It Mean to Raise e to a Matrix?* is a related work, not a replacement target. No independent mathematical/pedagogical review, continuous listening, fine-sync acceptance, or publication is claimed.

## Shared picture and promise

Use one horizontal ruler as an “addition machine” and one output scale as a “multiplication machine.” Moving a fixed distance on the first multiplies the second by a fixed factor. In video one, run forward from time/distance to a multiplier. In video two, reverse it: given a multiplier, find the distance. In the complex plane the output ruler becomes a circle and different turns can end at the same point. This is a proposed visual bridge, not a demonstrated novelty or audience result.

## Video one: The Magic of the Exponential

**Promise:** start with familiar repeated growth, show why the same function solves a rate law, and make a real piece of calculating hardware readable. Then show where the same mechanism reappears, without implying every phenomenon is exponential under all conditions.

### Opening: a machine that multiplies by sliding

Picture: slide rule C and D scales, cropped so the markings are readable. Set 1 on C above 2 on D; look below 3 on C to read 6 on D. Hold before the reveal. Then turn the slide into a pair of abstract rulers.

> “Before pocket calculators, engineers could multiply by moving two rulers. Line up a one with a two. Find a three on the moving scale, and beneath it is a six. Why does sliding distances add them, yet the numbers multiply? The answer leads to a function that describes growth, decay, charging circuits, heat and even rotations. The exponential is not a bag of unrelated miracles. One idea sits underneath them.”

The arithmetic is exact as a mathematical example. A real slide rule gives a reading limited by its physical scale and does not place the decimal point automatically: 2 × 3 and 20 × 30 use the same scale alignment. The instrument primarily uses logarithmic scales; exponential and logarithm functions form an inverse pair. The Smithsonian documents the C/D operation and separate decimal placement ([linear slide rules](https://postalmuseum.si.edu/object-group/slide-rules/linear-slide-rules)). No implication that e alone or base-e markings were necessary: common logarithms work too.

The [slide-rule visual prototype](exponential-slide-rule-prototype-20261101.html) lets the viewer align the scales and reveal the product. Desktop author checks covered the initially hidden answer, `2 × 3 = 6`, `3 × 3 = 9`, and legible labels after a cursor-overlap fix. Its factor range is deliberately limited to 1–3; it is not finished footage, an independent design review, or a complete physical-slide-rule simulator.

One ElevenLabs UI request produced two opening candidates, both preserved with source text, settings and hashes in `narration_lab/third-wave-20261024/root/elevenlabs-exponential-20261101/opening-takes-receipt01.json`. They use Oliver / Eleven v4 provisionally; neither has been listened to or selected, and no pronunciation or sync acceptance is claimed. No logarithm narration has been generated.

### 1. The property behind the machine

Picture: two identical jumps, then unequal jumps; synchronized segments join on the input line while multipliers combine on output. Start with 2^x as a recognizable example before normalizing to e.

> “If a quantity doubles for each unit of time, then three units multiply it by two, three times: eight. Two more units multiply it by four. Together the five units multiply by thirty-two. That is the rule: the multiplier for one interval followed by another is the product of their separate multipliers. In symbols, f of x plus y equals f of x times f of y.
>
> “That rule is what turns adding lengths into multiplying numbers. A logarithmic ruler puts a number at a position proportional to its logarithm. A physical slide adds positions, so its markings multiply. Reversing the idea, a logarithm converts multiplication back into addition.”

Clarify assumptions: for a nonzero continuous real-valued function on all real inputs satisfying the equation, f(0)=1, f is positive, and f(x)=e^(cx) for a real constant c. Without continuity or another regularity condition, do not make this classification claim. A constant function 1 corresponds to c=0. This property characterizes a family; the additional normalization f′(0)=1 selects e^x.

### 2. Why e and why its own derivative?

Picture: compare a tangent to 2^x and a tangent to e^x at x=0, then translate the same local picture across the input ruler. Avoid declaring all exponential bases self-derivatives.

> “Every positive exponential base has the addition-to-multiplication rule. What makes the base e special? Measure x in the natural units where the curve starts at one and its initial slope is also one. Then the same little step has the same proportional effect wherever we stand.
>
> “For a differentiable function with the multiplication rule, compare f at x plus a tiny h with f at x. The first is f of x times f of h. So the change near x is f of x times the change near zero. Divide by h and let h shrink: f prime of x equals f of x times f prime of zero. Choose the one with initial slope one, and its derivative is itself.”

On screen: `[f(x+h)-f(x)]/h = f(x)[f(h)-1]/h`, then `f′(x)=f(x) f′(0)`. The number e is the value at 1 of that normalized function, approximately 2.71828. A constant-rate model is needed: if `y′=k y` and `y(0)=y₀`, then `y=y₀e^(kt)`; the *fractional* rate is k only where y is nonzero, and k must be constant for this form.

### 3. Three doors into the same function

Picture: three panels show small-step multiplication, a running rate law, and accumulating Taylor terms. The three paths meet at a single curve; label hypotheses rather than claiming three unexplained formulas are obviously equal.

> “There are several ways to build this function. One starts with repeated small multipliers: for a fixed number x, take one plus x divided by n and multiply that by itself n times. As n grows, the result approaches e to the x.
>
> “A second asks for the curve whose height and slope match everywhere, starting at height one. A third adds a series: one, plus x, plus x squared over two factorial, plus x cubed over three factorial, and so on.
>
> “They meet because they describe the same growth law. The series can be differentiated term by term and gives back itself with starting value one. Uniqueness of that initial-value problem identifies it with the rate-law curve. The small-step limit reaches the same curve. These are three constructions of one object, not three unrelated coincidences.”

On screen: `exp(x)=limₙ→∞(1+x/n)^n=Σₖ₌₀∞x^k/k!`, and `y′=y, y(0)=1`. For negative fixed x, use n sufficiently large that the base is positive when using logarithms to justify the limit. Describe `e = lim (1+1/n)^n` as a special case; don't confuse it with the general-x formula. For the series-to-ODE and limit-to-ODE equivalence provide a source-bound proof in production; the brief is not that independent proof.

**Production derivation, with the dependency order made explicit:** Define `E(x)=Σₖ₌₀∞x^k/k!`. The ratio test gives infinite radius of convergence, so termwise differentiation gives `E′=E` and `E(0)=1`; [NIST DLMF 4.2.19](https://dlmf.nist.gov/4.2.E19) gives the exponential series. The derivative of `E(x)E(−x)` is zero, so the product is 1. Thus E never vanishes; since `E(0)=1` and E is continuous, it is positive for real x. For any other differentiable solution `y′=y, y(0)=1`, the derivative of `y/E` is zero, so `y=E`. This establishes uniqueness without assuming the addition law.

The addition law now follows from uniqueness: for fixed x, `t↦E(x+t)/E(x)` solves the same initial-value problem as `E(t)`. E is strictly increasing since `E′=E>0`; for positive x its series gives `E(x)≥1+x`, while `E(−x)=1/E(x)`, so its range is all positive reals. Define the real logarithm as the inverse of E. Its derivative at 1 is 1, so for fixed real x and sufficiently large n, `n ln(1+x/n)=x·[ln(1+x/n)/(x/n)]→x` when x is nonzero; x=0 is immediate. Continuity of E then gives `(1+x/n)^n→E(x)`. [DLMF's logarithm series](https://dlmf.nist.gov/4.6.E1) also gives the local first-order behavior. This route uses the logarithm after constructing E; it does not treat logarithm as an independent premise and then use the limit to define E. These are author-derived links to standard facts, not independent review; the visual should show a small step rather than rush through this proof.

### 4. The same clock in different clothes

Picture: four mini-panels share one rate-law strip, then each shows its own assumptions.

> “The same pattern turns up whenever the instantaneous rate is proportional to what remains or what has accumulated. A radioactive sample with a fixed decay constant has an exponentially declining expected number of undecayed nuclei. An ideal capacitor charging through a resistor approaches its final voltage with an exponential gap. Under Newton's cooling model, the temperature difference from a constant-temperature room decays exponentially. A first-order chemical reaction follows an exponential concentration under its model assumptions. Money under a constant continuously compounded rate grows by the same equation; real interest rates and cash flows need not be constant.
>
> “And exponential growth is not a promise of growth forever. In a population, limited resources or changing behavior can make the simple model fail.”

Equations for the visual, with explicit model assumptions: `N(t)=N₀e^(−λt)` (expectation for independent radioactive survival with constant hazard), `V_C(t)=V_final(1−e^(−t/RC))` (ideal fixed R,C and DC step from zero), `T(t)−T_room=(T₀−T_room)e^(−kt)` (Newton cooling model, constant room temperature and k>0), `[A](t)=[A]₀e^(−kt)` (first-order kinetics), `B(t)=B₀e^(rt)` (constant continuously compounded rate, no added cash flows). Sources: OpenStax on [radioactive decay](https://openstax.org/books/university-physics-volume-3/pages/10-3-radioactive-decay), [RC charging](https://openstax.org/books/college-physics/pages/21-6-dc-circuits-containing-resistors-and-capacitors), [cooling](https://openstax.org/books/calculus-volume-2/pages/4-3-separable-equations), [first-order rate laws](https://openstax.org/books/chemistry/pages/12-4-integrated-rate-laws) and [continuous compounding](https://openstax.org/books/intermediate-algebra-2e/pages/10-2-evaluate-and-graph-exponential-functions). Verify parameter units before production; no causal or universal claims implied.

### 5. The rotation that looks like growth in a new direction

Picture: e^(it) as a unit-circle point with a tangent arrow, then show its derivative is i times the position: multiplication by i rotates the vector a quarter turn.

> “Put an imaginary unit in the exponent, and the picture changes. The exponential traces a circle: cosine t plus i sine t. Multiplying by i turns an arrow through a right angle, so the same derivative rule now says the motion points along the circle rather than straight outward. Change in proportion to the current state can describe rotation as well as growth.”

`e^(it)=cos t+i sin t` follows from grouping even/odd terms in the convergent complex power series, not from treating imaginary quantities as real heights. Briefly point to the existing matrix-exponential video as the next layer, not as an assumed prerequisite.

### Ending and handoff

> “One rule links the ruler, the rate law, and the curve: add inputs, multiply outputs. The exponential is the forward machine. But what if we know the output and ask for the input? That is the logarithm. On the real number line it works beautifully. On a circle, the same destination can hide more than one journey.”

## Video two: The Mysteries of the Logarithm

**Promise:** explain the inverse honestly on positive real numbers, then show why a turn counter is needed for complex inputs and why matrices add a different kind of constraint. This may be more advanced than the first video: define each new object visually, and treat matrices as a clearly marked final extension.

### Opening: undo a movement when you cannot see the laps

Picture: one ramp and a circular track. Two runners stop at the same point after one and two extra laps; their paths cannot be distinguished from their endpoint alone.

> “How long did the money grow to become twice as large? A logarithm finds the missing input of an exponential. But what if the input also records turns, and several different numbers lead to the same point? Then undoing the exponential needs a choice. And when numbers become matrices, even the question ‘can it be undone?’ gets subtler.”

### 1. Undoing growth on the positive real line

> “The real exponential rises continuously from values near zero to arbitrarily large values. It never reaches zero, and it never visits a negative output. So every positive real number has exactly one real input: its natural logarithm. The logarithm of one is zero. The logarithm of e is one. The logarithm of zero is not a real number, nor is the real logarithm of a negative number.
>
> “Since exponentials turn adding inputs into multiplying outputs, their real inverse turns multiplication into addition: log of a times b is log a plus log b, when a and b are positive. That is the idea built into the slide rule.”

`ln:(0,∞)→R`; `ln(ab)=ln a+ln b` for positive real a,b; `d ln x/dx=1/x`. The input is a dimensionless ratio in physical applications; a logarithm of a unit-bearing quantity requires a reference scale.

### 2. What changes when the input can turn?

Picture: draw an input plane (horizontal real part, vertical imaginary part) beside an output plane. Trace a vertical interval of height 2π mapping once around the unit circle. Emphasize that input 0 and input 2πi are different, yet both map to output 1.

> “For complex numbers, the exponential can rotate as well as scale. Add two pi i to the input and the output makes one complete extra turn, returning to the same point. So the exponential of zero and the exponential of two pi i are both one. There is no single global inverse that returns every original input.
>
> “For a nonzero complex number with radius r and one angle theta, every logarithm has real part log r and imaginary part theta plus any whole number of complete turns. Zero is still excluded: the complex exponential never outputs zero.”

`{ln|z| + i(arg z + 2πk): k∈Z}`, `z≠0`. Use a set of possible values before choosing a branch; `arg z` denotes one chosen angle at that point, not a globally continuous angle.

### 3. A cut is bookkeeping, not a physical tear

Picture: unwrap the circle to a helix; follow one loop around zero. A marker advances by 2π in imaginary height even though its projected endpoint returns to the start. Then remove the nonpositive real axis and show the remaining slit plane mapping to an open strip.

> “To make one continuous choice, we restrict where we allow ourselves to travel. A common choice removes the nonpositive real axis and uses angles strictly between minus pi and pi. On that slit plane, the principal logarithm is a single analytic function. The cut marks where our chosen angle would jump; it is a choice of domain, not a crack in space.
>
> “Why do we need a restriction? Follow a continuously unwrapped angle around a full loop. It comes back two pi higher even though the point is where it started. A single-valued continuous choice cannot assign both values to that same point.”

`Log z=ln|z|+i Arg z`, `Arg∈(−π,π)` on `C\(−∞,0]`. A separate convention can assign a value on the negative axis using a half-open angle interval, but then it is discontinuous there; do not call that extension analytic on the cut. Distinguish `exp(Log z)=z` on the domain from `Log(exp w)=w` only on the selected horizontal strip.

### 4. When a familiar law stops being literal

> “Even after choosing a branch, log of a product need not equal the sum of the chosen logs. If two angles are each three quarters of pi, their sum is one and a half pi. The product lands at the same point as minus half pi. On the principal branch the two answers differ by one full turn: two pi i. The real rule did not become nonsense; its domain and the meaning of the logarithm changed.”

For `z=w=e^(3πi/4)` on the slit plane, `Log z+Log w=3πi/2`, while `Log(zw)=−πi/2`; all three values are in the principal domain. Their difference is `2πi`. Avoid suggesting the branch formula works for every possible pair or that all imaginary logs are rotations alone when modulus changes.

### 5. One step into higher dimensions: matrices

Picture: a square grid becomes a transformed grid; first show a full rotation, then a mirror flip. Explicitly distinguish a matrix from a complex number and matrix logarithm from a logarithm of a vector or a coordinate-wise logarithm.

> “For a square matrix A, a matrix logarithm means a matrix X whose matrix exponential is A. These can be non-unique even among real matrices: doing nothing and making a full turn both return a two-dimensional grid to its starting position.
>
> “Some transformations cannot be reached by exponentiating any real matrix. A mirror reflection has negative determinant, while the determinant of a real matrix exponential is always positive. But if we allow complex matrices, that same reflection can have a complex logarithm. So the answer depends on what kind of logarithm we ask for.”

Precise example: `J=[[0,−1],[1,0]]`; `exp(0)=exp(2πJ)=I`. `R=diag(−1,1)` has no real matrix logarithm since `det(exp X)=exp(tr X)>0` for real X, but `diag(iπ,0)` is a complex logarithm of R. The broader real-log existence criterion is subtler than determinant alone; a positive determinant is necessary, not sufficient. Link to the existing matrix-exponential video, and label this as one higher-dimensional generalization, not the only one. See [Higham, matrix logarithms](https://nhigham.com/2020/11/17/what-is-the-matrix-logarithm/) and [Gallier, real matrix logarithms](https://arxiv.org/abs/0805.0245).

### Close

> “The real logarithm reverses growth. In the complex plane it must also account for turns. With matrices it asks which transformations a continuous exponential motion can reach. The mystery is not that the inverse failed. It is that we discovered what information the forward map can forget.”

## Evidence and comparison

- [NIST DLMF §4.2](https://dlmf.nist.gov/4.2) covers real and complex exponential/logarithm definitions and branches; [§4.7](https://dlmf.nist.gov/4.7) covers derivatives. [OpenStax calculus](https://openstax.org/books/calculus-volume-1/pages/3-9-derivatives-of-exponential-and-logarithmic-functions) derives the exponential derivative; confirm a complete three-definition equivalence proof independently before final script lock.
- [Smithsonian slide-rule history and example](https://postalmuseum.si.edu/object-group/slide-rules/linear-slide-rules) gives 2×3, decimal-place limits and historical chronology. Logarithmic scales are not dependent on the natural base.
- [3Blue1Brown, What's so special about Euler's number e?](https://www.youtube.com/watch?v=m2MIpDrF7Es) is directly relevant established competition. Only its listing/title and description have been inspected here; no continuous video or transcript review is claimed. The paired slide-rule-to-complex-unwrapping treatment is a proposed framing, not proof of distinction.
- [ETH complex-analysis notes](https://people.math.ethz.ch/~sszirnyi/Complex_Analysis_Notes.pdf) discuss logarithm branches and the absence of a global holomorphic branch on the punctured plane. The winding sketch above still needs independent review.

## Open work

- [ ] Audit the mathematical claims and edge cases independently, including equivalence of constructions, branch choice, matrix counterexamples and domain-specific models.
- [ ] Read/watch representative existing treatments and record the actual scope and differentiation; test with a fresh reader.
- [ ] Build visual scenes, a readable C/D slide rule, and the complex-plane unwrapping, with numeric facts derived and checked against separate oracles.
- [ ] Write coherent ElevenLabs narration units; preserve source text, voice/model/settings and every take. Inspect pronunciation, listen continuously and check sync; do not infer acceptance from ASR or metadata.
- [ ] Run guarded builds and renders only when the existing free-space and other resource admission rules permit. Inspect actual output, captions and visuals, and seek independent acceptance.
- [ ] Keep both videos separate in the catalog and Beads, linked to each other and the existing matrix-exponential video. Publication is not claimed.

Author numerical spot checks on 2026-11-01 confirmed the 2×3 logarithmic alignment, the stated complex-log branch counterexample, full-turn rotation and the complex reflection logarithm, and compared 25-term exponential sums to a library at x = −2, 0, 1, 2. These do not establish the general theorems or replace independent review.

## Substantive replacement: build the three doors instead of naming them

2026-11-04 author expansion for bead `mx-video-program-program-87do.62`.
This replaces the abbreviated equivalence passage in the proposed full film.
It is original draft narration and a native scene specification, not generated
speech, compiled scenes or independent mathematical/pedagogical acceptance.
The introductory ruler and later applications still need comparable expansion.
The historical HTML prototype is not a production route. Use native
FrankenManim trackers, curves, TeX and transformations throughout.

### A. A rate law that forces its own curve

**Picture:** A growing blue bar and a tangent arrow share one value tracker.
At successive checkpoints the arrow length equals the bar height. Beside it,
show a constant-height arrow driving a straight line, so “constant rate” and
“constant proportional rate” are visibly different. Use dimensionless time;
restore physical units with a separate constant later.

> Start at height one. Here is the rule for drawing the curve: at every moment,
> its slope must equal its current height. At height one, climb at rate one.
> At height two, climb twice as fast. A straight line cannot do this. Its slope
> stays fixed while its height changes. Our curve must keep steepening.
>
> We can try to draw it with short straight steps. Read the current height,
> use that height as the slope, move a little to the right, and read again.
> The straight step is an approximation: during the step, the true height
> would already have changed. Make the steps shorter and the approximation
> improves. But before trusting the picture, let us build an exact solution.

### B. Why the factorials appear

**Picture:** Begin with `1 + ax + bx² + cx³ + …`. Match coefficients in two
aligned rows for the function and its derivative. Animate each derivative
coefficient downward into its matching power column. The coefficient is
revealed only after the viewer can predict it. Simultaneously add successive
polynomial curves on one fixed scale; retain the previous curve dimly.

> Suppose we try a polynomial, starting at one. Its linear coefficient must
> be one, because the initial slope must be one. Now differentiate the whole
> expression. The derivative of b times x squared is two b times x. To match
> the x term we already have, two b must equal one. So b is one half.
>
> The next derivative contributes three c times x squared. It must match
> one half times x squared. So c is one sixth. Each new coefficient is the
> previous coefficient divided by the next whole number. That produces one,
> one, one half, one sixth, one twenty-fourth, and so on: the reciprocals of
> the factorials.
>
> No finite polynomial will finish the job. Differentiating removes its
> highest power. But the infinite series has another term ready to supply
> every missing power. Call its sum E of x for now. We have not assumed it
> is an exponential; we are finding out what this construction can do.

**Mathematical obligation on screen:** For fixed `|x|≤R`, consecutive absolute
terms have ratio at most `R/(k+1)`, eventually at most one half. Thus the tail
is controlled by a geometric series uniformly on each bounded interval. The
derivative series has the same convergence property. This justifies
term-by-term differentiation and gives `E′=E`, `E(0)=1`; it is not merely a
formal manipulation of dots. A zoomable tail bound should accompany the
polynomial overlay, rather than presenting agreement of a few pixels as proof.

### C. Why adding inputs multiplies outputs

**Picture:** Put the terms of `E(x)` along one edge of a grid and those of
`E(y)` along the other. The cells represent their products. Color successive
diagonals of constant total degree. Enlarge the degree-two diagonal before
generalizing: `x²/2 + xy + y²/2 = (x+y)²/2`.

> Multiply two of these series. Every term from the first meets every term
> from the second. Instead of reading the grid by rows, collect terms with
> the same total degree. The constant is one. The first-degree terms give
> x plus y. The second-degree terms give x squared over two, plus x y, plus
> y squared over two. That is x plus y, squared, divided by two.
>
> The same pattern continues. The binomial coefficients are exactly the
> factors needed to turn each diagonal into x plus y to that degree,
> divided by the corresponding factorial. All the diagonals together give
> E of x plus y. We have derived the multiplication law from the series.
>
> In particular, E of x times E of minus x is E of zero, which is one.
> So E never vanishes. And for a real input, E of x is E of half x squared.
> It is positive. It also has positive derivative, so it increases. These
> are consequences of the construction, not separate guesses about a graph.

**Proof detail:** Absolute convergence justifies rearranging the double sum.
For degree `m`, use `Σ x^k y^(m−k)/(k!(m−k)!)=(x+y)^m/m!`.
Name `e=E(1)`. Integer powers follow from the multiplication law; positive
roots give rational powers, and continuity extends the identity to real
inputs. This is the point where the notation `E(x)=e^x` is earned.

### D. Why repeated tiny multipliers reach the same function

**Picture:** Give one unit of time a fixed width. Split it into `n` cells;
each cell multiplies height by `1+x/n`. Show the endpoint on the same graph
as the series. Then replace the endpoint coincidence with a binomial-row
comparison: its coefficient factors approach one, one column at a time.

> Return to the short straight steps. Over one unit of time, a fractional
> rate x gives each of n steps the multiplier one plus x over n. After all
> the steps, we have that multiplier raised to the n-th power. For a rate
> of one, this is the familiar compound-growth construction of e.
>
> Why should that limit be our series? Expand the finite product. Its
> coefficient of x to the k is one over k factorial, multiplied by a short
> chain of correction factors: one, one minus one over n, one minus two
> over n, and so on. For any fixed k, every correction factor approaches
> one as n grows. The coefficient approaches exactly the coefficient in
> our series.
>
> Matching each fixed coefficient is not enough by itself: there are also
> more and more terms. Here is what makes the argument safe. In absolute
> value, each finite-product term is no larger than the corresponding
> exponential-series term. We already know that series has a small tail.
> First make the shared tail small; then make the finitely many remaining
> coefficients close. That proves the whole expression converges.

**Exact statement:** For integer `n>0` and `k≤n`,
`binom(n,k)(x/n)^k = x^k/k! ∏_{j=0}^{k−1}(1−j/n)`.
All product factors lie in `[0,1]`; set the finite expansion's coefficients
to zero for `k>n`. The shared summable majorant is `|x|^k/k!`. This argument
works for each real x, including negative x, without assuming a logarithm
limit or relying on monotonicity of the compounding sequence. For a physical
decay-step interpretation, require n large enough that `1+x/n>0`.

### E. Close the differential-equation door without a circle

> We constructed a function whose derivative equals itself. Could a different
> function obey the same rule and start at the same height? Take any proposed
> solution and multiply it by E of minus x. Differentiate that product.
> One contribution is the proposed function times E of minus x; the other
> is its negative. They cancel. The product is constant, and at zero it is
> one. So the proposed solution must equal E of x everywhere on its interval.
>
> The power series, the small-multiplier limit and the rate law are three
> constructions of the same object. Its powers, derivatives and growth rule
> fit together because they express the same structure from different sides.

**Exact proof:** If `y′=y`, `y(0)=1` on an interval containing zero, then
`(y(x)E(−x))′=0`; use `E(x)E(−x)=1`. No appeal to an unproved general
uniqueness theorem is needed. For `y′=ky`, replace x with kt, giving
`y(t)=y(0)E(kt)` for constant k. Dimensional consistency and the assumptions
behind each physical model must be taught in the application chapters.

Reference refresh: [NIST DLMF §4.2(iii)](https://dlmf.nist.gov/4.2.iii)
provides the standard exponential definitions. The coefficient-grid,
shared-tail and integrating-factor teaching sequence above is an author
derivation, still awaiting independent review. The previously cited
Smithsonian page returned a request-rejected page on this refresh; no new
historical claim is inferred from that failed retrieval.

- [ ] Integrate this developed sequence with a full ruler demonstration,
  applications derived from models, complex rotations and the logarithm bridge.
- [ ] Have the derivation and teaching sequence independently reviewed.
- [ ] Implement and inspect the coefficient grid, polynomial convergence,
  controlled tail and compounding scenes in native FrankenManim.
- [ ] Generate ElevenLabs only after the expanded script is locked; preserve
  existing takes, keep listening/fine-sync gaps explicit, and qualify a new cut.

### F. Turn the multiplication law into a calculating machine

2026-11-04 continuation, original author narration. Preserve A–E above; this
adds the missing bridge and applications, not a replacement short survey.

**Native scene:** Keep the same exponential curve from A–E. A horizontal
input tracker drives its height. Place a second horizontal rail below it;
transfer heights 1, 2, 3, 4, 6, 10 to the rail at their inverse positions.
Rescale the input distance so one complete rail has length one and carries
1 through 10. All tick positions come from `log10(value)`, not hand spacing.
Expose the common scale factor `1/ln(10)` only after the ruler is understood.
Keep blue for input distance and gold for multiplicative output.

> We have proved that adding inputs multiplies outputs. Can we use that
> backwards, to make multiplication into something our hands can do?
> Put the number one at the beginning of a ruler. Put two where the
> exponential reaches two, and three where it reaches three. Continue in
> the same way. The labels get closer together because the curve grows
> faster as we move along it.
>
> The distance from one to two is the input that produces two. The distance
> from one to three is the input that produces three. Add those distances.
> Our multiplication law says the result is exactly the input that produces
> six. We have turned multiplication into adding lengths.
>
> Copy the ruler onto a sliding strip. Move its one above the fixed ruler's
> two. Now its three sits above six. No multiplication table was hidden in
> the movement. The shape of the scale did the calculation.

**Prediction:** Freeze the aligned 2×3 demonstration, then reset. Ask where
sliding 1 to 2.5 will place 4. Hold before moving the cursor. Answer lands at
the endpoint 10; use that endpoint to teach a finite ruler's limitation.
Next show 4×3 with the sliding **10**, not 1, aligned to fixed 4: sliding 3
lands at fixed 1.2. Keep a separate decimal-place estimate `4×3>10` and
restore the factor ten to obtain 12. Do not animate a nonexistent off-screen
12 tick and pretend that is the actual instrument's reading.

> A real rule is finite. For four times three, the ordinary alignment would
> run off the end. Use the other end of the sliding scale instead. It now
> reads one point two. We moved by one whole decade, so the answer is
> twelve, not one point two. The rule supplies the significant digits;
> you still have to keep track of the size of the answer.
>
> Reverse the movement and you divide. Longer, more finely marked scales
> let you distinguish closer answers, but a physical mark has width. This
> is approximate computation. Before electronic calculators, people could
> perform these operations by aligning scales instead of doing all the
> written arithmetic. The logarithm, our name for the inverse input, is
> what turns the exponential law into this instrument.

**Evidence:** MIT's [slide-rule demonstration](https://web.mit.edu/2.972/www/reports/slide_rule/slide_rule.html)
was opened on 2026-11-04 and shows C/D alignment and decimal-place handling.
It contains an obvious erroneous sentence about the logarithm of zero;
do not copy that sentence. The correct origin is `log(1)=0`, and zero has
no real logarithm. Smithsonian search results describe its historical
collection, but the direct spotlight fetch returned 404 on this refresh;
that failure supplies no fresh chronology. No quantified time-saving claim
is made. The narration and geometric derivation here are original.

### G. Why the same curve turns up in different laboratories

**Native scene:** Return to the blue height bar and rate arrow. Show a
population of particles and a tiny time window beside it. During that window
show the same *fraction* of the remaining population leaving, not the same
fixed number. Reuse the actual initial particle identifiers; removed particles
stay removed. Label the smooth curve as amount/mean, not an exact random
particle trajectory. Parameter `k` has units inverse time.

> Suppose each remaining particle has the same constant chance per unit
> time of disappearing. More remaining particles means more expected
> disappearances. Fewer remaining particles means fewer. The rate of
> decrease is proportional to what remains. That gives the same rule as
> before, with a minus sign: the derivative of the amount is minus k times
> the amount.
>
> Its solution is the starting amount times the exponential of minus k t.
> Equal time intervals remove equal fractions, not equal amounts. If one
> interval leaves half, the next leaves half of that half. The amount
> becomes a quarter. That is why a constant-rate decay model has a fixed
> half-life, whatever amount you start with.
>
> First-order chemical reactions use this same mathematical model when
> their rate is proportional to the concentration of the reactant. That
> assumption matters. A reaction whose rate depends on the product of two
> changing concentrations does not automatically follow this simple curve.

**Derivation card:** `N'=−kN`, `N=N0 exp(−kt)`,
`T_half=ln(2)/k`. Independently animate 8→4→2 at equal half-life intervals,
then 80→40→20 with the same timeline. Do not equate deterministic eight-dot
halving with a measured radioactive sample. Use it as the labelled model.

**Native circuit scene:** Transform the abstract amount bar into a capacitor
charge indicator without changing its value. Draw a resistor and a closed
wire, then derive voltage-to-current and charge-to-voltage before the ODE.
The discharge-current arrow points away from the positive capacitor plate.

> An electrical circuit gives us the same curve for a different reason.
> A charged capacitor pushes current through a resistor. The current is
> voltage divided by resistance. The capacitor's charge is capacitance
> times voltage. As current leaves, charge decreases, so capacitance times
> the voltage's derivative is minus voltage divided by resistance.
>
> Divide by capacitance. The rate of change of voltage is minus the voltage
> divided by R C. Once again, the rate is proportional to the amount left.
> The product R C is a time. After that much time, the voltage has fallen
> to one over e of its starting value. It has not reached zero.

**Native graph:** Horizontal axis `t/(RC)`, vertical `V/V0`; at 1 time
constant show `e^(−1)≈0.368`, at 2 show `e^(−2)≈0.135`. Leave a small
but visible tail and label ideal-model asymptote. A second panel changes R
while maintaining C and shows time-scale stretching, not a changed normalized
curve. Reference: [MIT OCW electric-power systems chapter 1](https://ocw.mit.edu/courses/6-061-introduction-to-electric-power-systems-spring-2011/d79822ef5fbdb21b1c44e1b3b8282453_MIT6_061S11_ch1.pdf)
is a source lead surfaced in search; its full derivation is not yet reviewed.

### H. Growth, discounting, and the limits of the model

> Interest works the same way when the added amount is proportional to the
> balance. At a fixed continuous rate r, a balance B obeys B prime equals
> r times B, giving B zero times e to the r t. The inverse question asks
> what starting amount grows into a specified future amount. Reverse the
> multiplier: multiply that future amount by e to the minus r t.
>
> These are model equations, not promises about investments. Fees,
> withdrawals, changing rates and risk change the problem. If a rate varies
> with time and nothing else adds or removes money, we accumulate the rate
> over time in the exponent. Deposits introduce an additional term in the
> differential equation. The exponential is powerful because it solves
> a precise rule, not because every real process must obey that rule.
>
> The same warning applies to a growing population. Constant proportional
> growth can describe an early regime. Limited resources change the rate.
> When the rate depends on how full the environment has become, a different
> curve can emerge. Recognizing the exponential means recognizing the
> assumptions that produce it, and noticing when those assumptions stop.

**Native comparison:** Plot constant-rate `B0 exp(rt)` and a separate
piecewise-rate model with an announced change; never morph one curve into the
other while retaining a false constant-r label. Show the accumulated area
under the rate graph controlling the multiplier. This is a mathematical toy,
not financial guidance or an empirical forecast.

### I. Turn instead of grow

> So far our slope has pointed along a real number line. A complex number
> can also turn a direction. Multiplying by i makes a quarter-turn.
> What happens if a moving point's velocity is i times its position?
>
> Its velocity is perpendicular to its radius. So, instead of moving farther
> out, it goes around. Starting at one, it travels on the unit circle.
> The same exponential series works for a complex input: plug in i t.
> Its real terms collect into cosine, and its imaginary terms into sine.
> We have reached e to the i t equals cosine t plus i sine t.
>
> Adding inputs still multiplies outputs. On the circle, that means adding
> angles composes rotations. Give the exponent a real part as well, and it
> scales while it turns. Growth, decay and oscillation are now three faces
> of one construction.

**Native proof:** Keep position arrow and tangent arrow attached to one
angle tracker. Draw the quarter-turn relation before starting motion.
Derive `d|z|²/dt=2 Re(conj(z) iz)=0` to justify constant radius; speed is
one for the unit-radius initial condition. Pair the even and odd series terms
with sign changes, not a sudden unrelated Euler-identity title card. Then
compare `exp((a+ib)t)` for a negative, zero, and positive a, with the same b.
The zero case stays circular; the other cases spiral with radius `exp(at)`.
Close by asking which information a full turn loses, handing the question to
the separately developed logarithm film. [NIST DLMF definitions](https://dlmf.nist.gov/4.2)
was refreshed; term regrouping requires absolute convergence as in B–C.

**Remaining obligations for F–I:** Independent mathematical/editorial review;
check primary chemistry/model sources; verify the circuit source in full;
implement native scenes and oracle-derived numeric labels; lock coherent
ElevenLabs units; inspect rendered motion/frames, complete recordings,
pronunciations, listening, fine sync and independent audiovisual acceptance.
This author draft is not a completed film or a verified production scene.
