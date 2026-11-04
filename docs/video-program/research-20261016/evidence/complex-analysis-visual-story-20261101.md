# A visual journey through complex analysis

Owner request, 2026-11-01. Working title: **The Beautiful Geometry of Complex Functions**. This expands the existing `complex_analysis` catalog identity and links the argument-principle candidate and the requested exponential/logarithm sister videos. This is a working script and picture plan, not a finished film, independent review, narration or publication.

## Promise and continuous visual grammar

Follow a point in an input plane and its image in an output plane; keep the same colors and coordinates in both. First a little cross reveals what a derivative does. Next a closed loop becomes a moving arrow, making zeros and poles visible as turns. Finally use two poles to evaluate a real integral. Show all three mathematical links rather than presenting pretty function art as a proof.

Use domain coloring as an additional view, not a substitute for the two planes: hue encodes the argument of the output; a separately explained brightness or contour convention encodes magnitude. Hue is undefined at output zero and near poles the function is undefined; explicitly mark those points. Supply a phase key, readable contours/labels and an alternative for viewers who cannot distinguish colors. A single hue wheel alone loses magnitude information and repeats at full turns. Explain whether a pattern belongs to the input plane or the output plane at every transition.

The user's “if a derivative exists, it dictates all the properties” is a useful intuition to refine: differentiability at **one point** does not imply holomorphicity nearby. If a function is complex differentiable throughout an open set, Cauchy's theorem yields local power series and all derivatives. On a connected domain, its values on a set with an accumulation point in that domain determine the function by the identity theorem. A first derivative at one point alone does not determine an entire function. A map is locally conformal at points with nonzero derivative; critical points require separate discussion.

One ElevenLabs request produced two lossless Oliver / Eleven v4 opening candidates, retained with their exact submitted text, settings and hashes in `narration_lab/third-wave-20261024/root/elevenlabs-complex-analysis-20261101/opening-takes-receipt01.json`. Neither has been listened to, selected or synchronized. Submitted text is not proof of audible speech; the rest of the narration is not yet made.

## Draft narration and scenes

### 1. A function is a machine for moving a plane

Picture: input/output coordinates side by side. Drag one tracked point; leave short trails. Progress through z², z³−1, 1/z and exp(z), then add sin(z), a rational function and a preview of the gamma function. Each has its own domain and clear legend. Never numerically draw through a pole or silently draw a selected log branch as global.

> “A real function takes a number on a line and returns another. A complex function takes a point in a plane and returns another point in a plane. That sounds like four dimensions to draw. So instead, let's keep two planes: one for where we start, one for where we land.
>
> “Square a complex number and its distance from zero is squared while its angle is doubled. A whole grid bends and folds. Take a reciprocal and the inside and outside trade places, except at zero, where the reciprocal is undefined. The exponential turns horizontal motion into scaling and vertical motion into rotation. With color we can see the output angle across an entire input plane at once. But first we must learn what the colors mean.”

For z², angle doubling is understood modulo a full turn and z=0 has no defined angle. Reciprocal maps radius r to 1/r and angle θ to −θ for z≠0; inside/outside of the unit circle swap. exp(x+iy)=e^x(cos y+i sin y). Use gamma only where a trusted numerical implementation is available and disclose its analytic continuation and poles; the gallery is a plan, not an implementation claim.

Candidate gallery after the basics: `z²−1` (two simple zeros), `(z−0.45)/(z−1.35)` (a simple zero and a simple pole), `1/(1+z⁴)` (the four poles needed later), `e^z` (periodicity), `sin z` (zeros and complex growth), `Γ(z)` (meromorphic continuation and poles) and `ζ(z)` (meromorphic continuation with a pole at 1). These are planned examples only: validate each implementation and visual domain independently. For zeta do not imply a color plot proves the Riemann hypothesis or that values from its defining series are valid outside its convergence region; for gamma do not draw through its poles. Include only as much of the gallery as the audience can actually read.

### 2. What a complex derivative actually demands

Picture: at z₀, send a small horizontal and vertical arrow into a map; in the derivative limit both arrows experience the *same complex multiplication*. Show a shrinking cross at an ordinary point of z². Repeat at z=0, where derivative is zero: the leading effect is quadratic and the crossing angle is doubled. Contrast conjugation, which reflects and is not complex differentiable anywhere.

> “A complex derivative asks for one number that describes the limiting change, no matter which direction we approach from. Multiplying by that one complex number scales and rotates every tiny arrow by the same amount. So wherever the derivative is nonzero, a sufficiently small picture keeps its angles.
>
> “There is a big difference between having that limit at one isolated point and having it throughout a neighborhood. The powerful theory begins when the derivative exists on an open region. A function with that property is called holomorphic there. At a point where the derivative vanishes, the first-order approximation does not preserve angles; for z squared at zero, the angle doubles instead.”

Counterexample to the single-point overclaim: `f(z)=|z|²`; `[f(h)-f(0)]/h=conj(h)→0`, so f′(0)=0, but f is not complex differentiable at nonzero points. Nonzero derivative is needed for local conformality; `f′=0` does not mean the function itself is necessarily singular.

### 3. Why that local rule creates global rigidity

Picture: move a small circle around an interior point. Show values on boundary feeding a center via Cauchy's formula, then coefficients appearing in the Taylor expansion. Neighboring disks overlap, forming an “analytic relay.”

> “Here is the surprise. If a function is holomorphic throughout a neighborhood of a closed disk, its values around the boundary circle determine the value at the center. More than that: the same boundary values determine every derivative there. That is Cauchy's integral formula.
>
> “The result is far stronger than ordinary smoothness. Near every point, the function agrees with a convergent power series. On a connected region, two holomorphic functions that agree on a set with an accumulation point inside the region must agree everywhere there. Local information can force the rest, provided the hypotheses hold.”

For a positively oriented circle and a function holomorphic on a neighborhood of its closed disk, `f(a)=1/(2πi) ∮ f(z)/(z−a) dz`; `f^(n)(a)=n!/(2πi) ∮ f(z)/(z−a)^(n+1) dz`. This is a stated theorem with intuition, not a complete proof. The identity theorem requires an accumulation point **in** the connected domain; values on a finite list do not suffice. Reference: [Cauchy integral formulas](https://complexanalysis.org/web/sec_cauchy-integral-formulas.html) and [NIST DLMF §1.10](https://dlmf.nist.gov/1.10).

### 4. A loop that counts invisible zeros and poles

Use a simple rational map throughout: `f(z)=(z−0.45)/(z−1.35)`. A counterclockwise input circle centered at zero of radius 0.8 encloses the zero but not the pole; its image winds +1 about zero. A radius 1.7 circle encloses both and its image winds 0. At radii 0.45 or 1.35 the contour hits a zero or pole, so the count is undefined and the animation must stop/flag it.

> “Walk once around a loop in the input plane, and follow the output point. How many times does its arrow turn around zero? For a counterclockwise simple loop, if the function is meromorphic on and inside it and the loop crosses neither a zero nor a pole, that winding number equals the number of enclosed zeros minus the number of enclosed poles, with multiplicity.
>
> “The reason is a residue calculation applied to the logarithmic derivative. Near a zero of order m, f prime divided by f has residue m. Near a pole of order n, its residue is minus n. A contour integral adds those contributions. It counts roots without first solving for their exact locations.”

Formula: `wind(f∘γ,0)=(1/2πi)∮γ f′(z)/f(z) dz=Z−P` for a positively oriented simple closed contour with a meromorphic f on a neighborhood of the contour and interior and no zeros or poles on the contour. Count multiplicities. Winding is of the **image** curve about zero; a residue of f is not itself generally a count of zeros/poles. See [MIT notes, argument principle](https://math.mit.edu/~helgason/18122.pdf), Theorem 18, and [argument principle](https://complexanalysis.org/web/sec_argument-rouche.html).

An author-built [color and winding prototype](complex-analysis-winding-prototype-20261101.html) renders this rational map, lets a point travel around a chosen input circle and plots its image. The output now rescales using the image circle's center and radius, making the scale change explicit; at either undefined contour boundary it withholds the mapped path. Desktop author checks covered 0.8, 1.34, 1.35 and 1.36; a separate calculation sampled 2,049 points for each of the 169 non-boundary slider radii and checked window bounds and winding. The path can look polygonal close to the pole, and expanding axes reduce local detail. Separate earlier author checks matched the integral residue sum. These are sampled exploratory results, not an independent proof, full accessibility review, or production footage.

### 5. A real integral from poles off the real line

Picture: the real graph `1/(1+x⁴)` lies below, the complex z-plane above. Complete a real-line segment with an upper semicircle; show the two enclosed roots and residues. Animate the arc's bound shrinking as R grows; keep the semicircle counterclockwise.

> “Here is a real question: what is the area under one over one plus x to the fourth, from minus infinity to infinity? Rather than finding a long real antiderivative, let x become a complex variable and close the path with a huge semicircle above the real axis.
>
> “There are two simple poles inside: the fourth roots of minus one in the upper half-plane. The residue at each root r is one over four r cubed. Their sum is minus i times the square root of two, divided by four. Multiply by two pi i, and we get pi over the square root of two. The arc tends to zero, so this is the original real integral.”

Details: `α=e^(iπ/4)`, `β=e^(3iπ/4)`, residues `1/(4α³)` and `1/(4β³)`; their sum `−i√2/4`, yielding `∫₋∞∞ dx/(1+x⁴)=π/√2`. For R>1, `|1+z⁴|≥R⁴−1` on the upper arc and its length is πR, giving bound `πR/(R⁴−1)→0`. The ordinary improper integral converges absolutely; do not describe a principal value as if it were the same convergence argument. The video must work the roots and signs visually rather than dropping the result as a magic trick.

### 6. Analytic continuation: one function beyond its original recipe

Picture: start from `1+z+z²+…` inside the unit disk and extend with `1/(1−z)` outside, marking z=1 as a pole. Label the original series as convergent only for |z|<1; plot the rational continuation away from 1. Use gamma as a second, sourced gallery example and explicitly mark its poles.

> “A power series can give us a function only inside the region where it converges. The geometric series gives one over one minus z inside the unit circle. The rational expression continues the same function beyond that circle, but not through its pole at one. Analytic continuation extends a function; it does not make the original divergent series converge.
>
> “Sometimes following continuations along different paths can lead to different branches. The logarithm's winding story is an example, and it gets its own sister video. Residues describe isolated singularities and power contour methods; they are related to continuation, but are not a universal mechanism for every continuation.”

Gamma integral defines Γ(z) initially for Re z>0; it extends meromorphically to the complex plane with simple poles at 0, −1, −2, …, by [NIST DLMF §5.2](https://dlmf.nist.gov/5.2). If visualizing poles, do not claim gamma is holomorphic *at* them. For log, continuation around zero may change branch; distinguish local continuation, paths, singularities and globally single-valued choices.

### Close

> “A complex derivative asks every direction to agree. That requirement makes local maps rigid, turns boundaries into sources of information, and lets us count and calculate by walking around what we cannot see. The colors are a map of those constraints. They become useful when we can read the geometry behind them.”

## Existing treatments and production limits

[Domain coloring: Visualizing Complex Functions](https://www.youtube.com/watch?v=MpxA1YIoEyA) and the [interactive domain-coloring/conformal-map overview](https://maxschottenfeld.github.io/complex-analysis/assets/visualizations/00-overview-domain-coloring-and-conformal-maps.html) already pair colored functions with visual mapping; visual color alone is not a novelty claim. Only the source descriptions and overview page were inspected, not continuous viewing/listening. The proposed through-line from a tracked point to a loop, an integral and continuation remains a hypothesis to assess against more complete treatments and fresh-reader feedback.

- [ ] Independently audit the residue arithmetic, arc estimate, counterexamples, Cauchy/identity conditions, gallery domains, branch choices and narration claims.
- [ ] Review competing treatments audiovisually and assess actual distinctness. No complete competitor viewing or listening is claimed.
- [ ] Build and inspect real visual scenes, phase/magnitude legend and accessible alternatives; verify numerical computation near poles/critical points, and use qualified libraries for special functions.
- [ ] Create ElevenLabs narration and retain complete takes/settings; actual continuous listening, speech-content review, pronunciation and fine sync remain open.
- [ ] Follow existing resource admission and exclusive guard for builds/renders; verify real visual and technical output before acceptance. No new production footage or publication is claimed here.
