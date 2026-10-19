# Epsilon and Delta: Why Getting Closer Wasn't Enough

Every curve x^n on [0, 1] is unbroken, yet the shape they approach is flat
at zero with a single point at height one: a limit that jumps. A fixed
point's gap to that limit shrinks to zero (0.9 needs n = 7 to get within
1/2, 0.99 needs 69, 0.999 needs 693), but the moving point 2^(−1/n), where
each curve crosses one half, keeps a gap of exactly 1/2 for every n. The
video turns that difference, who chooses the point and when, into epsilon
and delta, then into constructive choices of delta (the Sprague and Davis
notes the owner supplied) and the open-set picture behind topology.

**Status (2026-10-19, revision `v1-michael-20261019`):**

- **Chapter 01** (`01_a_limit_that_jumps`, task
  `mx-video-program-program-87do.58.1`) landed in commit 520dff7. The
  orchestrator verified it, not this lane (Mail 3766): strict Clippy, 33
  native tests, all 55 facts against the oracle, a rebuilt and rendered
  opening, and its own viewing of all 10 cue-end frames, all 6 glyph pages
  and the prediction and reveal frames at 32.9, 37.9 and 39.2 s. The
  question holds without the answer; the reveal follows it. The lane's own
  checks were source-level only (oracle, storyboard, lint, rustfmt, ubs);
  it inspected no rendered image.
- **Chapter 02** (`02_the_shrinking_region`, task `.58.2`) is built. Its
  right-panel pieces are rebuilt from chapter 01's own builders. The
  orchestrator's batch central13 ran on handoff 3868 (Mail 3871); all of
  the following is root's evidence, not this lane's:
  - fmt, strict release all-target Clippy, 42 tests and the build passed;
  - all 78 native fact rows equal the independent oracle, whose 58
    cross-checks pass;
  - 97 glyph sources rendered with 0 failures, and root viewed all 9 pages;
  - root viewed all 10 opening cue-end frames, all 9 chapter 02 cue-end
    frames, and 12 prediction, zoom, reset and card frames at 37, 39, 42,
    45, 47, 50, 54.3, 58.2, 60.5, 62.3, 65 and 67.5 s. Root reported correct
    endpoints, an intact prediction interval, the window label going 10 →
    100 → 1000 through the reset, and the short card on one line, visible
    from 62.3 through 67.5 s. The layout lint report was empty.

  The drafts are in `.x/second-wave-20261019/draft04/epsilon_delta`
  (silent, 960×540, 30 fps):
  - the opening: 2795 frames, 93.17 s, byte-identical to draft01 with
    identical cues;
  - chapter 02: 2618 frames, 87.27 s.

  **Marker repair.** TanElm's image review of draft04 (3881) found one
  overlap the central13 check had not caught: from 33.5 to 42 s the ring
  marking the main plot's sub-pixel strip touched the axis numeral 1 and
  the hollow limit mark. A short arrow pointing at the strip from the right
  replaced it. The orchestrator's batch central15 (Mail 3910), root's
  evidence again:
  - fmt, strict release all-target Clippy and the release build passed;
  - the draft05 chapter 02 is 2618 frames, 87.27 s (silent, 960×540,
    30 fps), its layout report empty, its cues byte-identical to draft04;
  - root viewed six frames and an enlarged crop at 37 s, and TanElm
    independently confirmed the collision is resolved (3908).

  A transient text crossfade at 48 s is unchanged; it is an accepted
  caveat. The independent reviews are TanElm's (3822, 3855, 3881, 3908).
  The lane's own checks were source-level only, and it inspected no image.

**Limits.** Both silent scratch drafts run longer than the ~85 s chapter
target: the opening at 93.17 s, chapter 02 at 87.27 s. Root's decision is to
keep the prediction time and not speed anything up. Narration pacing and the
full audiovisual cut are not qualified. Whole-video, audio and
continuous-playback acceptance are not claimed for either chapter. The
continuation is outlined in [`docs/storyboard.md`](docs/storyboard.md) §2,
and the deltas its chapters need are already computed and oracle-checked.

| File | What it holds |
|---|---|
| `src/model.rs` | the mathematics: the curves, the limit, the moving point, the strip and its width, exact big-integer certificates for every printed decimal, the constructive deltas and the naive one that fails |
| `src/facts.rs` | every number shown or said, computed from the model, pinned where a source prints it |
| `oracle.py` | the same keys computed independently (Python decimal and fractions, binomial sums; standard library only) |
| `src/chapters/limit_that_jumps.rs` | chapter 01's scene, and the builders chapter 02 rebuilds its last picture with |
| `src/chapters/shrinking_region.rs` | chapter 02's scene: the zoom window, the two strips, the narrowing |
| `src/chapters/draw.rs` | the plot, the bright curve's morph, the colour roles, phrase timing, the think ring |
| `docs/storyboard.md` | the words and pictures (the source of `script.tsv`), colour roles, vocabulary, narration units and timing |
| `narration_units.json` | the script grouped into coherent narration units, bound to `script.tsv`'s bytes |

```bash
python3 -I -B oracle.py                          # 78 keys, cross-checks on stderr
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

## Scope limits kept in chapter 02

- Two strips are kept apart by their ends, with exact tests at n = 1, where
  the crossing 1/2 is a fraction. The curve is above 1/2 on x_n < x ≤ 1,
  which holds x = 1. The gap is at least 1/2 on x_n ≤ x < 1, which holds
  the moving point and not x = 1, where the curve and the limit are both 1.
  The narration's strip from 02_3 on is the gap strip.
- Nothing is drawn wider than it says. The zoom window is exactly as wide as
  the plot, so its labelled ×10, ×100 and ×1000 are the true sideways
  stretches (an earlier draft's 5.6-unit window made them ×(7/6)m; review
  3822). Its heights are drawn at 1/3 of the plot's, and the label says
  "heights ×1/3". The facts refuse to build if a label disagrees with the
  geometry, and a test checks the same. The magnifier box on the plot is the
  window at its true width, about 1.3 pixels of the 4K picture at ×1000.
  Below a pixel, a short arrow from the right points at where the strip
  is, not how wide; a ring there touched the axis numeral and the hollow
  mark (review 3881).
- 02_7 resets only the window, and says so before it raises anything
  ("Reset the window to x to the tenth"); the main plot stays at x¹⁰⁰⁰.
  The window leaves n = 1000 for n = 10 on those words, goes to 100 at
  "Raise the power tenfold", and to 1000 at "then again".
- The pixel claim is resolution-bound and stated as such: the plot is 1296
  pixels wide in the 3840-pixel final picture, so the strip is narrower
  than one of them from n = 898. A smaller screen reaches that sooner. The
  narration says only "thinner than a pixel".
- The width is spoken as shrinking "like point six nine over n". The card
  is kept short enough to read in 02_7's last sentence: "w(n) = 1 − 2^(−1/n)
  > 0; n·w(n) → ln 2 ≈ 0.6931. This is a limit, not a finite equality." The
  two-sided bounds behind it, ln 2/n − (ln 2)²/(2n²) < w(n) < ln 2/n for
  every n, are tested in `model.rs` and `oracle.py` and stated here; they
  are not shown on screen.
- The window shares 0.670, 0.691 and 0.693 are shown as numbers, and the
  narration says "almost the same", not "the same".

## Narration

- Narration is on hold. The owner rejected the first wave's narrated cuts on
  2026-10-19, and the rejected method must not carry into new synthesis. The
  units and semantic gaps in the storyboard are a proposal for the next
  qualified method; no take exists.
