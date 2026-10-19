# Epsilon and Delta: Why Getting Closer Wasn't Enough

Every curve x^n on [0, 1] is unbroken, yet the shape they approach is flat
at zero with a single point at height one: a limit that jumps. A fixed
point's gap to that limit shrinks to zero (0.9 needs n = 7 to get within
1/2, 0.99 needs 69, 0.999 needs 693), but the moving point 2^(−1/n), where
each curve crosses one half, keeps a gap of exactly 1/2 for every n. The
video turns that difference, who chooses the point and when, into epsilon
and delta, then into constructive choices of delta (the Sprague and Davis
notes the owner supplied) and the open-set picture behind topology.

**Status (2026-10-19, revision `v1-michael-20261019`, task
`mx-video-program-program-87do.58.1`):** chapter 01
(`01_a_limit_that_jumps`, 82.9 s by the storyboard's estimate) is written
as code: its scene, facts, model and independent oracle, with its
storyboard, script and narration units. Nothing has been compiled,
rendered or narrated here; the corpus orchestrator runs the central build,
tests, draft render and frame review. A scratch draft must confirm the
chapter stays under 85 s. The continuation is outlined in
[`docs/storyboard.md`](docs/storyboard.md) §2, and the deltas its chapters
need are already computed and oracle-checked.

| File | What it holds |
|---|---|
| `src/model.rs` | the mathematics: the curves, the limit, the moving point, exact big-integer certificates for every printed decimal, the constructive deltas and the naive one that fails |
| `src/facts.rs` | every number shown or said, computed from the model, pinned where a source prints it |
| `oracle.py` | the same keys computed independently (Python decimal and fractions, binomial sums; standard library only) |
| `src/chapters/limit_that_jumps.rs` | chapter 01's scene |
| `src/chapters/draw.rs` | the plot, the colour roles, phrase timing, the think ring |
| `docs/storyboard.md` | the words and pictures (the source of `script.tsv`), colour roles, vocabulary, narration units and timing |
| `narration_units.json` | the script grouped into coherent narration units, bound to `script.tsv`'s bytes |

```bash
python3 -I -B oracle.py                          # 55 keys, cross-checks on stderr
python3 -I tools/storyboard.py --check epsilon_delta
python3 -I tools/lint_script.py explainer_corpus/epsilon_delta/script.tsv
./x epsilon_delta facts-check                    # the Rust facts against the oracle (builds)
```

## Sources

Read for this chapter (2026-10-19):

- **MIT 18.100B, Spring 2025, lecture 20** (T. H. Colding),
  [PDF](https://ocw.mit.edu/courses/18-100b-real-analysis-spring-2025/mit18_100b_s25_lec20.pdf):
  Example 1 gives x^n on [0, 1] converging pointwise to 0 on [0, 1) and 1 at
  1; Example 1A shows the convergence is not uniform by taking, from the
  intermediate value theorem, a point x_n in (0, 1) where the n-th function
  equals 1/2. Our 2^(−1/n) is that point in closed form. The pinned limit
  values cite it.
- **MIT 18.100B, lecture 21**, Theorem 1 (a uniform limit of continuous
  functions on an interval is continuous, by an ε/3 argument): read for
  chapter 09; not used in chapter 01.
- **University of Toronto MAT237 notes, §1.2**
  ([page](https://www.math.utoronto.ca/courses/mat237y1/20199/notes/Chapter1/S1.2.html)):
  a limit at a needs a to be a limit point of the domain S and uses
  0 < |x − a| < δ with x in S; continuity at a in S uses |x − a| < δ with x
  in S; Theorem 5 states the preimage form, and the notes add that f need
  not be invertible. Read for chapters 04 and 11.
- **Sprague (1960) and Davis (1961)**, American Mathematical Monthly: their
  bibliographic identity and displayed mathematics come from the brief,
  which read the owner's screenshots and JSTOR's issue records. They were
  not re-read in this pass. `model.rs` reproduces the brief's arithmetic
  independently: δ = √(4 + ε) − 2 = 0.0248456731 at ε = 1/10, C = 22 and 187.

Competing treatments inspected (page text only; no video was watched):
[3Blue1Brown's epsilon–delta lesson](https://www.3blue1brown.com/lessons/epsilon-delta/)
(2017) defines limits with input and output ranges, uses a rational function
with a hole and a jump that has no limit, and has none of x^n,
pointwise/uniform, constructive δ for polynomials, topology or named
history. Its linked Essence of Calculus video was not watched. The
[Axiom Academy written lesson](https://theaxiomacademy.org/learn/ra-uniform-convergence)
uses x^n on [0, 1] and an ε-tube around the limit, but no moving
half-height point. The [GeoGebra page](https://geogebra.org/m/nfueujqt)
shows x^n only at fixed points; its applets were not run. Found by search
and not inspected: MIT RES.18-006's lecture on uniform convergence, and
animation pages at Manitoba and NTNU. Novelty is not established; the full
competition review remains open.

## Scope limits kept in chapter 01

- No historical names or dates are spoken (a test enforces it) until the
  brief's page-level attribution check is done.
- The narration says "unbroken". The lower-third states continuity on
  [0, 1] exactly. At the jump, the screen says that x = 1 is the domain's
  right end, where x arrives only from the left.
- The plot shows eight curves, and the narration says a picture of a few
  curves cannot prove the limit. The argument spoken is the fixed ratio
  x < 1, and the exact decimals are certified by integer arithmetic.
- The half-height point is a sufficient witness, not the worst one: the
  gap's supremum over [0, 1] is 1 for every n, and is never reached.
- "Uniform continuity" is never said: every x^n is uniformly continuous on
  [0, 1]. Pointwise and uniform convergence are named in chapter 08.
- Narration is on hold. The owner rejected the first wave's narrated cuts on
  2026-10-19, and the rejected method must not carry into new synthesis. The
  units and semantic gaps in the storyboard are a proposal for the next
  qualified method; no take exists.
