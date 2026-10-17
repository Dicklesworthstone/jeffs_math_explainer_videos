# Epsilon and Delta: Why “Getting Closer” Wasn’t Enough

Owner-requested candidate, 2026-10-17. Stable identity: `epsilon_delta`; Beads epic: `mx-video-program-program-87do.58`. This is a researched editorial brief, not a locked script or production approval. The catalog and its linked Beads epic own disposition and remaining work.

## Owner intake

The owner supplied two screenshots and proposed an episode connecting the difficulty of epsilon–delta, calculus foundations, counterintuitive failures, more systematic proofs, and the topological approach mentioned in a reply. This is explicit personal editorial interest, not measured audience demand.

- `IMG_1439.PNG`, SHA-256 `231f5e2ffc487781055a04b9bf607010b67b7d925ba28569c97dc5f14a44ad09`: visible extract of Atherton H. Sprague, “A Note on δ and ε,” and the poster’s account of finding it in an anthology.
- `IMG_1440.PNG`, SHA-256 `a81690f93493f80b2de65382cd7d32ff55aa90913d0d4ef0cc105aec10c3da39`: Allan Davis, “A Further Note on δ and ε,” plus the reply “The topological definition is even simpler.”

The attachments were read as source material. The first scan is cropped at its lower edge. No original post URL or unseen replies are inferred. The notes’ journal metadata was independently found in JSTOR’s issue records: Sprague, *American Mathematical Monthly* **67**(8), October 1960, p.780, [DOI](https://doi.org/10.2307/2308663); Davis, **68**(6), June–July 1961, pp.567–568, [DOI](https://doi.org/10.2307/2311156). The screenshots supply the displayed mathematical text; the issue listings verify bibliographic identity, not unseen article content.

## Editorial recommendation

**Make the viewer need the definition before asking them to manipulate it.** Open with perfectly continuous curves whose limit has a jump. The central question is: what does “as close as we want” promise, and who gets to choose each quantity first?

The payoff is three connected discoveries:

1. Epsilon–delta turns a picture into a guarantee that survives every sufficiently close input.
2. The intimidating algebra is often a search for a sufficient error bound, not a magic exact delta. The two notes make that search constructive.
3. Sequences and topology re-express the guarantee; they help with different tasks and do not erase its logical content.

Audience fit: visual-proof viewers get a genuine visual failure, students get a method they can use, and programmers get a precise dependency contract. Owner-reported success of visual proofs supports fit as an editorial judgment, not a causal performance forecast.

Avoid the title “The Definition That Fixed All of Calculus.” The foundations also require a number system with the right completeness properties and separate hypotheses for passing limits through operations. The story is better when precision reveals exactly what a theorem needs.

## Mathematical spine and animation

### 1. Continuous at every finite stage; discontinuous in the limit

For positive integers n, animate f_n(x)=x^n on **[0,1]**. At each fixed x<1 the height tends to zero; at x=1 it stays one. The pointwise limit f is zero on [0,1) and one at 1. The graph really has a discontinuity at its endpoint, using the domain’s relative topology.

First pin an ordinary x, then reveal the moving witness x_n=2^(-1/n). It approaches 1, but f_n(x_n)=1/2 while f(x_n)=0. The troublesome region shrinks without ceasing to exist. A screen-resolution warning is useful: an increasingly thin region is not an empty region. This is a modern teaching example, not a claim that these exact curves appeared in a particular historical dispute. [MIT’s lecture 20](https://ocw.mit.edu/courses/18-100b-real-analysis-spring-2025/mit18_100b_s25_lec20.pdf) gives this example and distinguishes pointwise and uniform convergence.

The quantifier contrast, revealed only after the picture:

- Pointwise: for every x and every ε>0, there is N (allowed to depend on x and ε) such that every n≥N has error below ε at that x.
- Uniform: for every ε>0, there is one N that works for every x and every n≥N.

The witness gives an error of 1/2 for every n, so uniform convergence fails. Do not call this a failure of “uniform continuity”: that is a different property. Indeed every individual x^n is uniformly continuous on [0,1].

### 2. The challenge comes first; the response must cover every input

For continuity at a point a in a metric domain D:

\[
\forall\varepsilon>0\;\exists\delta>0\;\forall x\in D:
\quad |x-a|<\delta\Rightarrow |f(x)-f(a)|<\varepsilon.
\]

Animate an output tolerance band, then an input interval whose **entire image** stays in it. A challenger chooses ε; the defender chooses δ; the challenger may then choose any x in that input interval. One lucky point, one favorable approach, or one successful tolerance is insufficient.

Delta may depend on ε, a and the function, but not on the subsequently chosen x. A limit at a uses 0<|x−a|<δ and an output target L, and assumes a is an accumulation point of the domain; continuity includes x=a and uses f(a). Keep the distinction visible without a digression about isolated points. [Toronto’s definitions](https://www.math.utoronto.ca/courses/mat237y1/20199/notes/Chapter1/S1.2.html) state these domain conditions.

Why this can feel difficult is a pedagogical hypothesis, not a diagnosed universal learning disorder: nested quantifiers, reverse-order discovery versus forward-order proof, simultaneous input/output distances, and bounds introduced without explaining their purpose. Animate the scratchwork first; then replay it as the short finished proof. Avoid implying that a smaller δ has to be optimal.

### 3. What Sprague’s note actually buys

For x² at a=2, write h=x−2. Then

\[
|x^2-4|=|h^2+4h|\le |h|^2+4|h|.
\]

Instead of guessing a restriction on x+2, solve the sufficient error-budget inequality t²+4t<ε for t≥0. It gives

\[
\delta=\sqrt{4+\varepsilon}-2>0.
\]

For |h|<δ, the bound is strictly below δ²+4δ=ε. That is a complete epsilon–delta argument. The note offers a systematic way to choose delta **inside the same definition**, not a replacement for it.

As a simpler comparison, |h|<1 gives |h|²+4|h|≤5|h|, so δ=min(1,ε/5) also works. At ε=0.1 the two choices are approximately 0.0248456731 and 0.02. Both succeed; optimizing delta is not the assignment. Show a valid, deliberately conservative input band surviving the challenge.

### 4. Davis turns this into a reusable local bound

The displayed cubic, centered at a=1, becomes

\[
x^3+5x^2-6=h^3+8h^2+13h,\qquad h=x-1.
\]

When |h|<1, its absolute value is at most 22|h|. Therefore δ=min(1,ε/22) works. Visually, three error contributions fit inside one shared budget; higher powers become easier to control in a unit neighborhood.

Generalization, independently derived from the same algebra: write P(a+h)−P(a)=Σ c_k h^k, for k=1,…,m. Set C=Σ|c_k|. If C>0, δ=min(1,ε/C) suffices. If C=0 the polynomial is constant, so any positive δ works. This is a finite polynomial expansion, requiring neither derivatives nor a Taylor-series convergence theorem.

Davis’s second example also checks: for P(x)=−3x⁴−8x+2 at a=−2, P(a)=−30 and the coefficients of h,h²,h³,h⁴ are 88,−72,24,−3; their absolute sum is 187. Keep this as a supplemental example rather than a second full on-screen computation.

### 5. Return to the opening and repair the theorem

Uniform convergence is sufficient for the limit of continuous functions to remain continuous. It is not necessary in every individual example. The repair can be pictured as a three-piece error bridge:

\[
|f(x)-f(a)|\le |f(x)-f_N(x)|+|f_N(x)-f_N(a)|+|f_N(a)-f(a)|.
\]

Choose one N making each outer error <ε/3 everywhere. Then continuity of that fixed f_N supplies δ for the middle error. The total is <ε. That order of choices is what the original picture lacked. This standard proof appears in [MIT lecture 21, Theorem 1](https://ocw.mit.edu/courses/18-100b-real-analysis-spring-2025/mit18_100b_s25_lec21.pdf); the three-piece error bridge is our proposed animation.

### 6. Topology: preserve neighborhoods without specifying their size

Start with ordinary intervals. A function f:X→Y is continuous exactly when the preimage of every open set in Y is open in X. For metric spaces this is equivalent to the epsilon–delta condition at every point. The reply is right that the formulation is shorter; whether it is simpler to learn depends on whether open sets and preimages already mean something to the viewer.

Original worked visual: for f(x)=x² on R, the output set (1,9) pulls back to (−3,−1)∪(1,3). It is open despite having two pieces. Preimage means “all inputs landing here,” not an inverse function. Contrast the step f(x)=0 for x<0 and 1 for x≥0: the open target set (1/2,3/2) pulls back to [0,∞), which is not open in R.

Earn the equivalence: an open target neighborhood contains a little ε-ball; epsilon–delta provides a δ-ball around each point of its preimage. Conversely, the preimage of the target ε-ball is an open neighborhood of a, so it contains a δ-ball. Open sets in a restricted domain are **relative to that domain**. The usual forward image of an open set need not be open under a continuous map. [Toronto’s Theorem 5](https://www.math.utoronto.ca/courses/mat237y1/20199/notes/Chapter1/S1.2.html) gives the preimage equivalence and notes that invertibility is unnecessary.

## Alternatives, with the cost of each made explicit

| Approach | What it helps you do | What it does not remove |
|---|---|---|
| Recenter and bound, as in the screenshots | Construct δ for polynomials instead of guessing it | Still an epsilon–delta proof |
| Limit laws and algebra of continuous functions | Reuse proved results; do not re-prove every polynomial from scratch | Their foundational proofs and domain restrictions |
| Sequential criterion | On metric spaces: x_n→a implies f(x_n)→f(a) for **every** sequence in the domain; find one bad sequence to refute continuity | One tested sequence cannot prove it; the converse fails in general topological spaces |
| Local Lipschitz bound / modulus of continuity | Turn desired output accuracy into an explicit sufficient input tolerance | Continuity need not be Lipschitz; √x at zero is continuous but has no finite local Lipschitz constant there |
| Open-set preimages | Handle continuity without choosing distances; make composition immediate | Learning topology; no numerical δ is automatically supplied |
| Modern infinitesimals | Express continuity as infinitesimally close inputs having infinitesimally close outputs | A suitable hyperreal extension and transfer framework; these are not nonzero infinitesimals inside R |

For sequences, a compact proof of the metric converse avoids unsupported assertion: if continuity fails at a, some ε₀>0 survives every δ. Choose x_n within 1/n of a whose output remains at least ε₀ from f(a). This contradicts preservation of every convergent sequence. For quantitative bounds, use √x only on [0,∞), with δ=ε² at zero. Do not use the mean value theorem to establish the foundations of differentiation without explaining the dependency.

For the optional infinitesimal coda, [Keisler, *Foundations of Infinitesimal Calculus*, Definition 3.7](https://people.math.wisc.edu/~hkeisler/foundations.pdf), gives the close-input/close-output formulation. In explicit extension notation, for a standard real a and standard real function f on a neighborhood of a: x≈a implies *f(x)≈f(a). A full construction belongs in another episode.

## Historical account: supported core and remaining attribution work

**Supported core.** In *The Analyst* (1734), §§XIII–XIV, Berkeley criticizes an argument that introduces a nonzero increment, divides by it, then treats it as vanished. Present this as his criticism of a justification, not proof that all early calculus was nonsense. The modern derivative of x² keeps h≠0 in ((a+h)²−a²)/h=2a+h and takes a limit; it never divides by zero. [Primary text, edited from the original editions](https://www.maths.tcd.ie/pub/HistMath/People/Berkeley/Analyst/Analyst.pdf).

Dedekind’s own 1872 preface recounts his 1858 concern about the foundations beneath limit arguments, including bounded increasing quantities having limits. This supports a second issue beyond manipulating increments: what justifies the continuum itself? [Authorized English translation, pp.1–2](https://www.gutenberg.org/files/21016/21016-pdf.pdf). Our boundary example: on Q∩[1,2], q²−2 is continuous and changes sign, but has no rational zero. Epsilon–delta continuity alone does not manufacture the missing real number.

**Attribution still to qualify before narration.** Bolzano, Cauchy, Weierstrass, Heine, Dedekind and others belong in the development, but avoid an unsupported “first inventor” date. Cauchy’s [1821 original is located at ETH](https://www.e-rara.ch/zut/content/structure/8445462?lang=en): chapter II concerns continuity; chapter VI concerns series. The attempted chapter-PDF retrieval failed in this pass. A page-level comparison with his 1853 sum-theorem formulation remains open before using it as a named historical failure. Do not silently equate Cauchy’s infinitesimal language with a modern pointwise statement or claim he personally used the x^n example. Exclude or qualify disputed attribution rather than making it dramatic by making it false.

Proposed historical narration: “Calculus was extraordinarily successful before its foundations reached their modern form. The hard part was making every hidden assumption visible—and discovering which assumptions the theorems actually needed.”

## Episode shape, competition and effort

Target **14–17 minutes**, twelve short chapters, each at most approximately 85 seconds: (1) continuous curves, broken limit; (2) shrinking region and fixed versus moving x; (3) Berkeley’s increment objection and modern limit; (4) challenge/response tolerance game; (5) why written proofs hide discovery; (6) Sprague’s recentering; (7) Davis’s general bound; (8) pointwise versus uniform choices; (9) three-piece proof repairing the opening; (10) sequences and quantitative bounds; (11) preimages and topology; (12) completeness, infinitesimal coda, return to the guarantee. If these ideas cannot breathe within that duration, move the infinitesimal/completeness expansion into supplementary material, retaining the factual caveat in the main narration.

The closest verified competitor is [3Blue1Brown’s epsilon–delta lesson](https://www.3blue1brown.com/lessons/epsilon-delta/) and its linked 2017 Essence of Calculus chapter. It already explains input/output bands visually. A generic “epsilon–delta made intuitive” treatment is therefore weak differentiation. Our proposed value is the failure → dependency order → constructive bounds → topology progression, tied to the owner’s two notes. Full viewing and broader coverage checks remain necessary; this limited search does not establish novelty.

Planning estimate, not measured production performance: one focused editorial/research day for attribution and full competitor review; another one to two days for a script and mathematical/storyboard review; animation medium-to-high effort despite simple curves, because quantifier dependencies and pullbacks must be staged clearly. No engine invention appears necessary from the concept alone. Existing chapter pacing and Michael voice gates still apply. Do not estimate final rendering or narration retries from this paper outline.

## Concrete next work and decision gates

- [x] Read both supplied images; separate screenshot content from bibliographic metadata.
- [x] Check Sprague’s bound and independently expand both Davis examples.
- [x] Establish the modern counterexample and a mathematically correct repair.
- [x] Derive a useful topology visual and distinguish methods from definitions.
- [x] Find a direct competing treatment; bound novelty claims.
- [ ] Read the closest complete competing videos and record which story beats are already covered.
- [ ] Finish page-level historical attribution before scripting named errors or invention dates; omit unsupported claims.
- [ ] Storyboard the moving x_n witness and the preimage transition before committing to the full episode.
- [ ] Have a fresh mathematical review check all domains, quantifier orders, strict inequalities, and the distinction between uniform convergence and uniform continuity.
- [ ] Record owner selection before activating production tasks; then use existing exact-cut and Michael acceptance gates.

**Kill/reframe criterion:** if the episode becomes a generic tolerance-band tutorial with a history montage, or topology only appears as an unexplained slogan, reframe before production. Keep it if a viewer can explain both why the moving witness defeats pointwise reasoning and why an input neighborhood inside a preimage is the same local guarantee as epsilon–delta.

## Evidence boundary and purpose

Consumer: Jeff choosing the slate and the eventual scriptwriter. Gate: source/proof/coverage qualification before this candidate becomes a production script. Provenance: the owner’s explicit 2026-10-17 request, including the two supplied notes and topology reply. Retire this brief as the controlling plan when replaced by a reviewed script/storyboard; retain it as source history. This is editorial/research work, not shipped animation or improved audio.

Creation check: humans read this brief; it is not runtime capability. The explicit owner request justifies the intake, not a speculative governance system. No integrity-control exception is invoked. Highest-priority work for this request is a useful story and correct mathematics; one brief and the existing catalog/epic are sufficient. Additional dashboards or a second task board would add less value.

Verification this pass: exact integer binomial expansion produced [6,13,8,1] at a=1 and [−30,88,−72,24,−3] at a=−2; corresponding absolute nonconstant coefficient sums were 22 and 187. A standard-library Python calculation checked the displayed decimal delta and representative x_n values. Those finite checks are arithmetic sanity checks, not proofs; the inequalities and exact formulas above supply the arguments. No independent reviewer, completed competition watch, full history audit, animation or audio review is claimed.
