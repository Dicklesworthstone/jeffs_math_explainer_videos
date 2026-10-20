# Epsilon and Delta: Why Getting Closer Wasn't Enough

Every curve x^n on [0, 1] is unbroken, yet the shape they approach is flat
at zero with a single point at height one: a limit that jumps. A fixed
point's gap to that limit shrinks to zero (0.9 needs n = 7 to get within
1/2, 0.99 needs 69, 0.999 needs 693), but the moving point 2^(−1/n), where
each curve crosses one half, keeps a gap of exactly 1/2 for every n. The
video turns that difference, who chooses the point and when, into epsilon
and delta, then into constructive choices of delta (the Sprague and Davis
notes the owner supplied) and the open-set picture behind topology.

**Status (2026-10-20, revision `v1-michael-20261019`):**

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
- **Chapter 03** (`03_what_the_old_objection_was_about`, task `.58.3`) is
  built. It opens on chapter 02's resting picture, rebuilt by chapter 02's
  own builders (`shrinking_region::resting`). It shows Berkeley's objection
  to a step: dividing by a nonzero increment, then setting it to zero. It
  shows that step on a modern example, the square at x = 1/2, then the
  limit, which never uses h = 0. All of the following is the orchestrator's
  and TanElm's evidence, not this lane's:
  - Source and history: TanElm (3994) fetched the primary PDF
    independently. It confirmed the quote verbatim on p. 6 with its two
    elisions, the page map, 1734, the paraphrases and the arithmetic, and
    found no defect.
  - Root's source review (3991) found the exact card shown too briefly and a
    long source card competing with the quote; both were repaired before
    native work (3998).
  - Central23 stopped on a missing `Timed` import (E0599), fixed in 4004.
  - Central24: fmt, strict all-target release Clippy, 52 native tests and
    the build passed; all 107 fact rows equal the oracle; 130 glyph sources
    rendered with 0 failures, and root viewed all 12 pages.
  - Root viewed draft06's frames:
    - the join with chapter 02's last frame (86 s);
    - the modern algebra;
    - the quotation at 36 and 43 s;
    - the prediction at 47 s;
    - the slope panel at 63 s;
    - the end at 77 and 91 s.
  - One finding from draft06: the Q(h) header stood 0.21 from the top,
    inside the 0.25 title-safe margin. It was lowered, with its guard now
    asserting that margin (4013), and passed in central25.
  - TanElm (4017) measured the 0/0 step, its fan and its note at full
    opacity for only about 1.3 s in draft07. They now carry into 03_6's
    first sentence (4025).
  - Central27: fmt, strict Clippy, the build and the 3 tests touching the
    changed scene passed (49 unchanged tests filtered), with no input drift
    and an empty layout report.
  - The model, facts and glyphs are unchanged since central24's full run.
  - TanElm (4033) measured the red 0/0 at full opacity from 55.667 to
    59.967 s (4.33 s) and the note from 55.767 s (4.23 s). They clear from
    60.0 to 60.4 s, before a clean panel and hops. Root's frames agree.

  The draft is `.x/second-wave-20261019/draft08/epsilon_delta`: chapter 03
  silent, 2778 frames, 92.60 s. This qualifies the mathematics, the sources
  and sampled frames only: not audio, 4K or continuous motion. The lane's
  own checks were source-level (oracle, storyboard, lint, units, rustfmt,
  ubs), and it inspected no image.
- **Chapter 04** (`04_the_challenge_game`, task `.58.4`) is built. It
  opens on chapter 03's last picture, rebuilt by chapter 03's own builders
  (`old_objection::final_picture`).
  - Chapter 03's scene itself changed only in that its three strings (the
    Q(h) formula, the limit, the exact card) are now built by functions both
    chapters share, with the same text.
  - Root's central35 (Mail 4087, root's evidence) passed on the first
    source:
    - fmt, strict Clippy, 60 native tests and the build;
    - 131 facts and 159 glyph sources;
    - a rebuilt chapter 03 byte-identical to the accepted one (SHA 07c0aa94);
    - a 93 s draft09, from which root viewed 12 frames and all 15 glyph
      pages and found no spatial defect.
  - EmeraldLeopard's source review (4086) found no math error, but withheld
    the chapter on an explanatory gap and three smaller points:
    - 04_4 concluded discontinuity from one losing window, right after 04_3
      showed that one losing window decides nothing;
    - 04_8 said every answer covers its window;
    - h = 0 in the continuity game was left implicit;
    - `window_failure`'s test picks left [0, 1] for large windows.
  - All four were repaired (4099); the scope limits below give the details.
    Two corrections followed:
    - `reach` is described as the symmetric safe radius, and t < reach as
      the supported range, not an impossibility (root, 4103 → 4105);
    - central40's strict Clippy flagged one constant assertion, now in a
      `const` block with the same inequality (4107 → 4110).
  - Root's central41 (Mail 4120, root's evidence) passed:
    - fmt, strict all-target release Clippy, 60 tests and 133 facts;
    - the glyphs;
    - the silent draft10, `.x/second-wave-20261019/draft10/epsilon_delta`:
      2791 frames, 93.03 s, SHA 66363662…, empty layout report.
  - Root inspected 12 final frames. EmeraldLeopard (4118) independently
    found no blocker in the new proof, the continuity rule, the h = 0 note,
    the prediction or the card timing. Optional typography notes remain and
    are not blockers.
  - This qualifies the mathematics, the sources and sampled frames only:
    not audio, 4K, continuous motion or the full video. The lane's own checks
    were source-level (oracle, storyboard, lint, units, rustfmt, ubs), and it
    inspected no image.
- **Chapter 05** (`05_why_written_proofs_hide_the_search`, task `.58.5`)
  is built. It opens on chapter 04's last picture, rebuilt by chapter 04's
  own builders (`challenge_game::end_picture`).
  - Chapter 04's scene changed only in that 04_7's domain note, band,
    tolerance, window and rows are now built by functions both chapters
    share, with the same text, sizes, places and order.
  - Two native batches stopped before it passed:
    - central45 on a layout guard that missed by one ulp in floating point
      (−0.55 − (−1.15) = 0.5999999999999999); the row moved to −1.16 with
      the guard unchanged (4144 → 4146);
    - central46 on a test-only helper flagged as dead code; it is now
      `#[cfg(test)]` (4149 → 4151).
  - Root's central47 (Mail 4153, root's evidence) passed:
    - fmt, strict all-target release Clippy, 66 tests and the build;
    - 142 facts and 178 glyph sources;
    - a rebuilt chapter 04 byte-identical to draft10 (SHA 66363662…);
    - the silent draft11 chapter 05, 2729 frames, 90.97 s (SHA a4cc9a59…),
      with an empty layout report. Root viewed 9 frames and found no
      collision.
  - TanElm's independent source review (4152) found the proof, the exact
    values, the oracle and the chapter 04 extraction correct. It found two
    things to fix, both now repaired in source (the scope limits below give
    the details):
    - the prediction was answered before it was asked (S1);
    - 05_8's narration left out the domain (M1).
  - The repair changes 05_6, 05_7 and 05_8's words and pictures only.
    TanElm's source follow-up (4156) found S1 and M1 resolved, the M2 note
    accurate, and no new defect.
  - Root's central48 (Mail 4157, root's evidence) passed:
    - fmt, strict all-target release Clippy, the 3 scene tests and the
      spoken-number, name-passage and storyboard/registry/script tests;
    - the build and render, with 17 input hashes stable and the model,
      facts, oracle and chapter 04 unchanged from central47;
    - the silent draft12 chapter 05, 2799 frames, 93.30 s (SHA b250e49e…),
      with an empty layout report.
  - TanElm's pixel review of draft12 (4159):
    - Nothing of the answer shows from 44.0 to 54.8 s: no outline, no widest
      row, no edge ring.
    - The outline begins at 54.80 s, after the "?" fades, and the row is
      full by 55.2 s. The edge ring appears at 56.23 s, on "where the curve
      leaves the band".
    - Proof line 2 relights at 74.70 s, on the domain phrase.
    - Chapter 04's last frame and chapter 05's first are visually identical.
    - One cosmetic collision: the think window's "pause for more time" ran
      about 5 px (at 540p) into the found window's left edge.
  - The think ring moved 0.25 left (`RING_AT` x −4.4 → −4.65; 4165 → 4168).
    Root's central49 (Mail 4171, root's evidence) passed fmt, strict Clippy,
    the 3 scene tests, the build and the render:
    - the final silent draft13 chapter 05, 2799 frames, 93.30 s (SHA
      5b05ee1f…), with an empty layout report and cues byte-identical to
      draft12;
    - root viewed frames at 49.1, 51.5, 53.8 and 57.0 s.
  - TanElm's delta check of draft13 (4178) passed:
    - the note is visible 49.73–54.10 s (132 frames), at least 10 px (at
      540p) short of the window's edge and at least 24 px from the y-axis;
    - outside the note's box, draft12 and draft13 differ only by codec
      noise.
  - FuchsiaHawk's review (4160) of the first-five assembly, assembled01
    (13781 frames, 459.37 s, with chapter 05 from draft12), found:
    - it is frame-identical to the five chapters;
    - the four joins are invisible;
    - there is no viewer-facing defect;
    - one bookkeeping nit: the global cues are written to 3 decimals
      (±0.33 ms). Root keeps those and writes a 6-decimal sidecar
      separately.
  - Draft11, draft12, draft13 and assembled01 are all kept. This qualifies
    the mathematics, the sources and sampled and measured frames only: not
    audio, 4K, continuous motion or the full video. The lane's own checks
    were source-level (oracle, storyboard, lint, units, rustfmt, ubs, and a
    Python emulation of the script-facing tests), and it inspected no
    image.
- **Chapter 06** (`06_sprague_recentring`, task `.58.6`) passed root's
  native verification in `narration_lab/production-20261020/second-wave-central/epsilon06-native-01`:
  fmt, strict release Clippy, 75 tests, 157 oracle values and 196 glyph
  sources. Rebuilt chapter 05 is byte-identical to draft13. Chapter 06 is
  2,805 frames, 93.5 s at 960×540p30, with empty layout lint. Root viewed
  11 actual frames and glyph pages 15–18; the prediction answer remains
  hidden until its reveal. Independent review is quota-blocked.
- **Chapter 07** (`07_the_polynomial_bound`, task `.58.7`) passed root's
  native verification in `narration_lab/production-20261020/second-wave-central/epsilon07-native-01`:
  fmt, strict release all-target Clippy, 78 tests, 169 oracle values,
  223 glyph sources with zero failures, and full decode. Its first scratch
  render is 2,417 frames, 80.566667 s at 960×540p30, with empty layout lint.
  Root viewed 12 actual frames spanning the algebra, prediction, reveal,
  general bound and ending, plus glyph pages 18–20. No independent visual
  or listening verdict is claimed.

- **Chapter 08** (`08_fixed_or_moving`, task `.58.8`) passed root's
  first native checks: 80 tests, 170 oracle values and 246 glyph sources.
  The first render exposed a title-safe margin warning. Lowering the title
  by 0.1 scene units cleared it; all cue times stayed byte-identical.
  Final `epsilon08-native-02` passes fmt, strict release all-target Clippy,
  two affected mathematical tests, build and full decode. It is 2,373
  frames, 79.1 s at 960×540p30. Root viewed 10 baseline and six final
  frames. UBS reports zero critical issues. Independent review remains
  quota-blocked. Both definitions include every later n; the counterexample
  takes n = N, using the same checked moving point as the opening.

**Limits.** Both silent scratch drafts run longer than the ~85 s chapter
target: the opening at 93.17 s, chapter 02 at 87.27 s. Root's decision is to
keep the prediction time and not speed anything up. Chapter 03's draft runs
92.60 s against a storyboard estimate of 83.9 s; root let it run over to
keep the mathematics readable (Mail 4001). Chapter 04's draft10 runs
93.03 s against a storyboard estimate of 84.7 s. Chapter 05's draft13 runs
93.30 s against a storyboard estimate of 84.5 s. Draft11, before the
prediction repair added seven words, ran 90.97 s against 82.3 s. Chapter
06's measured scratch duration is 93.5 s. Chapter 07 is 80.566667 s.
Narration pacing and the full audiovisual cut are not qualified. Whole-video,
audio and continuous-playback acceptance are not claimed for any chapter.
The continuation is outlined in [`docs/storyboard.md`](docs/storyboard.md)
§2, and the deltas its chapters need are already computed and
oracle-checked.

| File | What it holds |
|---|---|
| `src/model.rs` | the mathematics: the curves, the limit, the moving point, the strip and its width, exact big-integer certificates for every printed decimal, chapter 03's exact chord slopes (refused at h = 0) and their limit, chapter 04's exact witnesses (the pick that beats a wide window, the pick that beats every window for a misfilled hole, the window too narrow to hold a seen pick), chapter 05's window found by bounding a factor on the domain and the widest window's certified rounding, chapter 06's slope-only window and the first input where it loses, the constructive deltas and the naive one that fails |
| `src/facts.rs` | every number shown or said, computed from the model, pinned where a source prints it; chapter 03's page references and chapter 06's citation |
| `oracle.py` | the same keys computed independently (Python decimal and fractions, binomial sums; standard library only) |
| `src/chapters/limit_that_jumps.rs` | chapter 01's scene, and the builders chapter 02 rebuilds its last picture with |
| `src/chapters/shrinking_region.rs` | chapter 02's scene: the zoom window, the two strips, the narrowing; and its resting picture, rebuilt for chapter 03 |
| `src/chapters/old_objection.rs` | chapter 03's scene: the step on the square, 0/0 at h = 0, the slope panel, the tolerance and window; and its last picture, rebuilt for chapter 04 |
| `src/chapters/challenge_game.rs` | chapter 04's scene: the turns, the honest and the wide window, limit against continuity, the blind window, continuity of x² at the end of [0, 1]; and its last picture, rebuilt for chapter 05 |
| `src/chapters/hidden_search.rs` | chapter 05's scene: the finished proof, the search backward from the goal, the generous bound on the domain, the window found against the widest, the proof read forward; and its last picture, rebuilt for chapter 06 |
| `src/chapters/recentring.rs` | chapter 06's scene: the domain changed to every real number, x² recentred at 2, the slope-only window that loses, the square completed, Sprague's δ for every ε, the capped δ |
| `src/chapters/draw.rs` | the plot, the bright curve's morph, the colour roles, phrase timing, the think ring |
| `docs/storyboard.md` | the words and pictures (the source of `script.tsv`), colour roles, vocabulary, narration units and timing |
| `narration_units.json` | the script grouped into coherent narration units, bound to `script.tsv`'s bytes |

```bash
python3 -I -B oracle.py                          # 157 keys, cross-checks on stderr
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
  not be invertible. Read for chapters 04 and 11. Re-read for chapter 04 on
  2026-10-19 (the brief's link), in the page's own words:
  - lim f(x) = L means "∀ε > 0, ∃δ > 0 such that if x ∈ S and
    0 < |x − a| < δ, then |f(x) − L| < ε";
  - for this to make sense a must be a limit point of S: "∀δ > 0, ∃x ∈ S
    such that 0 < |x − a| < δ";
  - f is continuous at a ∈ S if "∀ε > 0, ∃δ > 0 such that if x ∈ S and
    |x − a| < δ, then |f(x) − f(a)| < ε";
  - at a limit point, continuity at a is the limit equalling f(a).

  Chapter 04's two definitions and its precisely card follow these. Its
  window at x = 1 keeps x ∈ [0, 1], and its strict inequalities are the
  page's. Chapter 05's proof is the continuity form with S = [0, 1] and
  a = 1/2. That chapter reads no new source and attributes nothing.
- **Sprague (1960) and Davis (1961)**, American Mathematical Monthly: their
  bibliographic identity and displayed mathematics come from the brief,
  which read the owner's screenshots and JSTOR's issue records. `model.rs`
  reproduces the brief's arithmetic independently: δ = √(4 + ε) − 2 =
  0.0248456731 at ε = 1/10, C = 22 and 187. Davis's note was not re-read
  in this pass.
- **Atherton H. Sprague, "A Note on δ and ε"**, read for chapter 06
  (2026-10-19) in two places:
  - the owner's screenshot of its page (the brief's `IMG_1439.PNG`, whose
    SHA-256 matched the brief's). The scan is cropped at its foot, below the
    start of the note's cubic example. It is a private file, and is not
    committed;
  - Crossref's record for its DOI,
    [10.2307/2308663](https://doi.org/10.2307/2308663): *The American
    Mathematical Monthly* 67(8), October 1960, p. 780.

  What the scan shows, and the chapter uses:
  - Sprague sets aside the usual route: write |x² − 4| = |x − 2|·|x + 2|
    and restrict x to values between 1 and 3, so that |x + 2| < 5. He says
    "this leaves me cold". That route's δ, min(1, ε/5), is the chapter's
    capped window.
  - "The whole trick is to expand x² − 4 in powers of x − 2": x² − 4 =
    (x − 2)² + 4(x − 2), so |x² − 4| ≤ |x − 2|² + 4|x − 2|.
  - "(completing the square)": |x − 2|² + 4|x − 2| + 4 < ε + 4, so
    |x − 2| + 2 < √(ε + 4), giving δ = √(ε + 4) − 2.
  - He states the limit form, 0 < |x − 2| < δ. The chapter proves
    continuity at 2, |h| < δ, which adds only h = 0, where the gap is 0.
  - The scan's later paragraphs (x³ at 2, the n-th root for xⁿ, 5x², a
    general polynomial) are not used. The general bound is chapter 07's.

  The chapter names Sprague once (06_9), for the method its scan shows.
  It quotes nothing on screen or in the narration, and it gives no claim
  about him beyond the note.

Read for chapter 03 (2026-10-19, the brief's links; page numbers are the
printed ones, with the PDF's 1-based page in parentheses):

- **George Berkeley, *The Analyst***, D. R. Wilkins's 2002 edition of the
  1734 text
  ([PDF](https://www.maths.tcd.ie/pub/HistMath/People/Berkeley/Analyst/Analyst.pdf)).
  - The note on the text, p. iii (3), says the edition is based on the 1734
    first editions, London and Dublin. That is the year said in 03_1.
  - §XIII begins at the foot of p. 5 (17) and runs onto p. 6 (18); §XIV is
    on p. 6. The argument he examines lets x become x + o, expands (x + o)^n,
    divides both increments by o to get 1 and nx^(n−1) + …, and then says
    "Let now the Increments vanish".
  - His objection, on p. 6, is the sentence 03_3 shows with two phrases
    elided: "the former Supposition that the Increments were something, or
    that there were Increments, is destroyed, and yet a Consequence of that
    Supposition, i. e. an Expression got by virtue thereof, is retained."
  - §XIV restates it. He has supposed "that o is something", then supposes
    "that o is nothing", yet begs "leave to retain nx^(n−1)": "a most
    inconsistent way of arguing".
  - §§XV–XVI, p. 7 (19), were read for context and are not quoted. §XVI
    notes that had the second supposition come "before the common Division
    by o, all had vanished at once", and calls the move "this Artifice of
    first dividing, and then changing your Supposition".
  - Page numbering: the task note gives PDF pages 16–17 for printed 5–6. In
    the copy read here, printed 5, 6 and 7 are PDF pages 17, 18 and 19, so
    the chapter cites printed pages only.
- **Richard Dedekind, preface to *Continuity and Irrational Numbers***,
  dated March 20, 1872, in W. W. Beman's 1901 authorised translation
  ([Project Gutenberg PDF](https://www.gutenberg.org/files/21016/21016-pdf.pdf)).
  - The passage is on p. 1 (6); the task note gives PDF page 5, which is the
    copyright page here.
  - In the autumn of 1858, lecturing on the differential calculus in
    Zürich, he "had recourse to geometric evidences" in proving "that every
    magnitude which grows continually, but not beyond all limits, must
    certainly approach a limiting value".
  - That is a question about completeness, separate from Berkeley's
    objection. Chapter 03 does not name him. Its last line says only that
    whether a limit exists at all is a separate question. The passage is
    recorded for chapter 12.

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

- No historical names or dates are spoken in chapters 01 and 02 (a test
  enforces it).
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

## Scope limits kept in chapter 03

- Names and dates:
  - One name and one year are spoken: George Berkeley and 1734, in 03_1,
    03_3 and 03_8.
  - A test holds that no other line says the name, and that each line that
    does has a storyboard row citing The Analyst §§XIII–XIV, pp. 5–6.
  - It also holds that no attribution not read at page level is said
    anywhere: Cauchy, Weierstrass, Bolzano, Newton, Leibniz, Dedekind,
    Heine, "invented" or "first to".
- What is said about the history:
  - The narration says Berkeley "questioned" and "objected" to a step. It
    does not say early calculus was wrong, and it calls the answer, one,
    right.
  - It does not say whose argument he examines, and it names no one else.
- The example is ours and modern: x² at x = 1/2, "in modern notation", said
  before any step is shown. The screen says the same. That Berkeley writes o
  for the increment and treats every power of x is stated here (Sources),
  not on screen.
- The citation on screen is concise. The column's heading gives the book,
  its year and "§§XIII–XIV, pp. 5–6", and the quote's own line gives
  "§XIII, p. 6 (Wilkins's 2002 edition)". An earlier draft also had a
  210-character source lower-third competing with the quote during 03_3 and
  03_4 (root's source review, Mail 3991); it was removed.
- Reading time:
  - The 130-character quote and its 59-character attribution stay up
    through 03_4's question, then cross-fade into the think ring. That is
    about 11.6 s fully shown, estimated from source; root viewed it at 36
    and 43 s.
  - The 0/0 step, its fan and its 43-character note stay fully shown at
    least 4 s from the completed reveal. They carry into 03_6's first
    sentence, which says the answer is reached without h ever being zero.
  - A const assert holds that 4 s to at least the note's characters at
    15 a second plus 1 s. TanElm measured 4.33 s for the 0/0 and 4.23 s for
    the note in draft08 (4033). Draft07 showed them for about 1.3 s (4017).
  - The precisely card (160 characters) comes in at "Any tolerance", as the
    window narrows, and stays to the chapter's end through 03_8: about
    16.8 s fully shown, estimated from source.
  - Root saw the card on two lines, clear of the "next:" line (4010). It
    still says that the slope is exactly 1 + h for every h ≠ 0, that every
    tolerance has its window, and that h = 0 gives 0/0, which the limit
    never uses.
  - The first source showed a 217-character card only on "the limit is one"
    and removed it as 03_8 began, about 1.9 s (Mail 3991).
- The model computes the chord's slope only for h ≠ 0. At h = 0 it refuses
  0/0, and the facts refuse to build if it ever accepts it. The tests and the
  oracle (fractions, independently) check that:
  - the slope is exactly 1 + h for every nonzero rational h tried, on both
    sides;
  - the window 0 < |h| < t keeps it strictly within t of 1;
  - at |h| = t it is exactly t away, so no wider window works.
- The tolerances:
  - 1/10 is said and 1/20 is shown. A test holds that each window is drawn
    wider than the hollow mark at h = 0, so it reads around it.
  - "Epsilon" and "delta" are not said yet; the narration says "tolerance".
- The fan of 03_5 is drawn at slopes from −1/2 to 3 to show that any line
  through one point fits 0/0. Those slopes are not printed.
- Not claimed:
  - that this definition settles whether limits exist (03_8 calls that a
    separate question);
  - any date for Cauchy or Weierstrass;
  - that anyone was the first to write it.

## Scope limits kept in chapter 04

- The order is the definition's: the challenger's tolerance, then the
  defender's window, then any pick in it. The narration says the window may
  depend on the tolerance, the point and the curve, never on the later h.
  The screen shows why: a defender who saw a nonzero pick could answer with
  a window too narrow to hold it (`blind_window`, |h|/2), and would win even
  against the misfilled graph.
- The rules on screen follow the game being played. The limit game's third
  rule is 0 < |h| < δ. From "Continuity puts zero in" on, it becomes
  |h| < δ. h = 0 is then a legal pick that no window can shut out, but it
  always passes, since |g(0) − g(0)| = 0 < ε; 04_6 shows this as a note
  beside the hidden pick.
- The limit is punctured and continuity is not. The limit's line says
  0 < |h| < δ against 1; continuity's says |h| < δ against g(0).
- Every pick played or tested is an input of the square's domain:
  1/2 + h ∈ [0, 1]. The model keeps |h| within the symmetric safe radius
  `reach(a)` = min(a, 1 − a), which is 1/2 here.
  - `window_failure`'s pick is min((ε + δ)/2, 1/2): the midpoint up to
    δ = 9/10, where it is exactly 1/2, and the domain's edge after that.
  - `fill_failure` caps its pick the same way.
  - `window_failure` supports only tolerances below the safe radius
    (t < 1/2), and refuses a larger one as out of its range. That is not a
    claim that no pick misses: at t = 1/2, the legal pick h = 1/2 is
    exactly 1/2 away, and so misses the strict tolerance. A test keeps both
    the refusal and that miss.

The arguments for every input are algebra:
- **The honest window.** For every h with 0 < |h| < δ = ε, the slope is
  1 + h, so |Q(h) − 1| = |h| < ε. This is the green line in 04_2. Filled
  with 1, g(0) = 1 adds |g(0) − 1| = 0, so the same window is continuous.
- **A wider window.** For ε < 1/2 (the safe radius) and δ > ε, the pick
  h = min((ε + δ)/2, 1/2) lies inside the window and the domain. If the
  midpoint is chosen, ε < h < δ. If the cap 1/2 is chosen, h = 1/2 > ε,
  and δ ≥ 1 − ε > 1/2. Either way |Q − 1| = h > ε.
- **The fill 1.5.** For every δ, the pick h = ½·min(δ, 2/5) has
  0 < h < δ and h ≤ 1/5, so g(h) = 1 + h ≤ 1.2, at least 3/10 below 1.5 and
  more than ε = 1/10. No window works. 04_4 now says so ("no window
  works"), and the screen shows the argument beside the window played:
  "any δ: h = ½ min(δ, 2/5)" and "|g(h) − 1.5| ≥ 3/10 > ε". The pick shown,
  h = 0.05, is the case δ = 1/10.
- **The end of the domain.** For 2^(−1/2) < x ≤ 1, squaring is increasing
  on [0, 1], so 1/2 < x² ≤ 1.

The checks are samples and demonstrations, not proofs:
- **The model's tests:**
  - every δ = k/100 from 11/100 to 3, plus 101/1000, 7/3 and 1000, with
    the exact edges 89/100, 9/10 and 91/100;
  - the fill 1.5 at nine windows from 1/1000 to 1000, its widest pick 1/5
    exactly, and the domain cap on a far fill;
  - grids of picks for the fill 1;
  - rationals across the window at x = 1.
- **The oracle:**
  - an independent grid search of in-domain picks finds a miss in every
    wider window tried (up to 1000) and in every window for the fill 1.5;
  - it then checks the model's witness formulas separately, which is a
    check of the same formula, not an independent search.
- The narration does not say "for every tolerance". The scene plays 1/10,
  and the model refuses a fill closer to 1 than the tolerance.
- Continuity at x = 1 keeps the domain's hypothesis: x ∈ [0, 1], so the
  window is 2^(−1/2) < x ≤ 1, open at its left end, where x² is exactly 1/2
  and so not within. δ = 1 − 2^(−1/2) ≈ 0.2929 is chapter 02's certified
  width at n = 2; the edge 0.7071 is chapter 01's certified crossing.
- No Sprague or Davis method appears (chapters 06 and 07): the end-of-domain
  bound is the monotonicity x² > 1/2 ⟺ x > 2^(−1/2) on [0, 1].
- Names and words:
  - No name or date is said.
  - "Epsilon" and "delta" are said once each, in 04_1, with their words;
    the screen writes ε and δ only in TeX.
  - 04_8 says "A winning answer covers every input in its window"; the
    chapter also shows answers that lose.
- Reading time, estimated from source at the scratch rate (root measures
  the render):
  - Each pick's readout stands about 1.4 s. The last ones stand longer: the
    honest round's through to 04_3, the miss into 04_4.
  - The reason |Q(h) − 1| = |h| < δ = ε stands about 7 s.
  - The precisely card (215 characters) stands from 04_4's second sentence
    to 04_6's second, about 27 s.
  - The δ rule stands about 8 s.
  - 04_7's rows stand about 10 s, to the end.
  - The chapter runs about 93 s silent by the same estimate, over the ~85 s
    target as the others do; root's draft10 measured 93.03 s. EmeraldLeopard
    found no blocker in the prediction or card timing on it (4118).

## Scope limits kept in chapter 05

- The domain is stated. The proof's second line says x ∈ [0, 1], and 05_4
  says "On the domain, zero to one". The bound |x + 1/2| ≤ 3/2 holds on
  that domain and is reached at x = 1, where a glow marks the axis.
- It is continuity at x = 1/2, as in chapter 04's continuity rule: the
  window |x − 1/2| < δ holds x = 1/2 itself, where the gap is 0 < ε.
- The argument for every input is algebra, shown as the proof's three
  lines:
  - for every ε > 0, take δ = (2/3)ε > 0;
  - for every x ∈ [0, 1] with |x − 1/2| < δ,
    |x² − 1/4| = |x − 1/2|·|x + 1/2| ≤ (3/2)|x − 1/2| < (3/2)δ = ε.
  - The ≤ is not strict, since the factor is exactly 3/2 at x = 1. The
    last step is strict for every such x, x = 1/2 included, since
    |x − 1/2| < δ and 3/2 > 0.
- The search and the proof are the same chain:
  - The search's rows read it from the goal: |x² − 1/4| < ε, the factoring,
    the bound on [0, 1], then (3/2)|x − 1/2| < ε when |x − 1/2| < (2/3)ε,
    then δ.
  - 05_8 lights the proof's lines in the definition's order: ε first, δ
    from ε (for this point and this curve), then every x in the domain and
    in the window.
  - The narration says both conditions on x, as the proof's second line
    does (TanElm, M1). The domain matters: for ε > 3/4 the window reaches
    past [0, 1], and there the bound fails. For example, at ε = 3, δ = 2,
    x = 2.4 gives a gap of 5.51 > 3.
- The window found is sufficient, not the widest:
  - At ε = 1/10 it is 1/15 ≈ 0.067. The widest symmetric window there is
    √(1/4 + 1/10) − 1/2 = √0.35 − 0.5 ≈ 0.0916, printed 0.092.
  - The right side binds: x² reaches 0.35 at x = √0.35 ≈ 0.5916. The left
    side alone would allow 1/2 − √0.15 ≈ 0.1127.
  - The widest is a measured, certified number, not derived on screen; the
    narration says "about".
    - The model certifies its 3-place rounding by exact squares at the
      half-unit ends.
    - The oracle computes it independently with 60-digit decimal square
      roots and confirms the right side is the tighter.
    - The facts refuse to build unless 1/15 is below the rounding's
      certified lower end, 0.0915.
  - The words are "widest" and "generous", never "best" or "optimal". 05_7
    says "A window only has to work for every input, not be the widest".
  - That the window found is narrower than the widest holds for every
    ε < 3/4, not always (TanElm, M2). At ε = 3/4 the window found equals the
    widest, δ = 1/2, the bound being attained at x = 1. For ε > 3/4 every
    window works on the domain. The chapter claims only the case it shows,
    ε = 1/10, and the narration never says "never".
- The model's tests and the oracle are checks at chosen points, not the
  proof:
  - the chain and the window at all 1001 points k/1000 of [0, 1];
  - x = 0.58, outside the window found, still passes;
  - 0.5915 passes and 0.5917 fails, exactly;
  - 0.4 passes on the left.
- The prediction is quantitative: 05_6 asks "Against one tenth, about how
  large could delta be, and still work?". The 4 s think window follows.
  - An earlier version asked "Is two thirds of epsilon the widest window
    that works?". TanElm found it answered before it was asked (4152, S1):
    05_4 had just called the step generous, and a yes/no question invited
    that word's answer.
  - With the question, at "Against one tenth", come the band's label
    ε = 1/10 and the window found there, δ = 1/15 ≈ 0.067, as an anchor. No
    widest outline, ring or number is on screen before 05_7.
  - The answer opens 05_7: at "About point zero nine two" the widest
    outline and the row "widest window: δ ≈ 0.092" appear. At "where the
    curve leaves the band" a ring marks that point, and at "One fifteenth
    is smaller" a glow marks the window found's row. A test holds that no
    line before 05_7 says the number, and that 05_7 opens with it.
  - The "only about 1" row of 05_4 invites a guess near one tenth. The
    curve allows a little less, since on the right the factor x + 1/2 is
    about 1.09.
- The plot draws the band and the window at ε = 1/10 from 05_2 on, while the
  narration keeps ε general until "Against one tenth". The band's label reads
  "within ε of 1/4" until then, and "ε = 1/10" after. Before the answer, the
  curve can be seen staying in the band a little past the window's edges.
  That is a picture to estimate from, not a number.
- No Sprague or Davis method appears (chapters 06 and 07):
  - no recentring and no exact solve for δ;
  - no first restriction δ ≤ 1 and no coefficient sum, min(1, ε/C).
  - The bound is the stated domain's, for this one curve and point. 05_9
    leaves "ways to make that search systematic" to the next chapters.
- Names and words:
  - No name or date is said; the names test covers every line.
  - "The challenger" carries over from chapter 04.
  - The search and the proof are named in the storyboard's vocabulary.
- Reading time, estimated from source at the scratch rate (root measures
  the render):
  - The finished proof stands lit about 6.6 s before it dims. It stays on
    screen, dim, and each line is lit again in 05_8, standing 15 to 22 s to
    the end.
  - The window found's row stands about 4.5 s before the question ends.
  - The think window leaves about 5.1 s from the question's end to the
    answer.
  - The search's goal row stands about 80 s.
  - The widest row stands about 38 s.
  - The "next:" line stands about 4 s.
  - The chapter runs about 93 s silent by the same estimate, over the ~85 s
    target as the others do. Draft13 measured 93.30 s; draft11, before the
    repair, measured 90.97 s against an estimate of 90.8 s.

## Scope limits kept in chapter 06

- The setting changes on screen before anything is computed. A number line
  shows chapter 05's domain [0, 1] and point 1/2, then every real number
  (both ends opened) and the point 2. The rows say "was: x ∈ [0, 1],
  a = 1/2" and "now: x ∈ ℝ, a = 2, a² = 4". The value is written at the
  point, a² = 4, since x ranges over every real number (root, Mail 4284).
  There is no interval from here on, so nothing bounds x + 2 for free.
- The plot is a magnified stretch near (2, 4), one tenth wide in x, and the
  caption says "magnified near x = 2". Its axes carry only 2 and 4 (facts).
  Near 2 the square looks straight. That is true of the picture, and it is
  the trap the prediction sets.
- The argument for every input is algebra, shown as rows:
  - h = x − 2, and x² − 4 = h² + 4h exactly;
  - |x² − 4| = |h² + 4h| ≤ |h|² + 4|h|, the triangle inequality, for h of
    either sign;
  - |h|² + 4|h| + 4 = (|h| + 2)² < 4 + ε exactly when
    |h| < δ = √(4 + ε) − 2, since t ↦ t² + 4t is increasing for t ≥ 0;
  - δ > 0 for every ε > 0, because √(4 + ε) > 2;
  - 0 ≤ |h| < δ ⇒ |x² − 4| ≤ |h|² + 4|h| < δ² + 4δ = ε. The last step is
    strict for every such h, h = 0 included (0 < ε), and either sign.
- The theorem has no bound on ε. The model's `sprague_delta_rounded` prints
  only δ < 1, so it refuses ε ≥ 5. That is a decimal-display limit, and
  its doc says so. The model's tests check ε = 1/1000, 1/10, 1, 5, 21 and
  1000 exactly, without it:
  - a grid point inside δ exists, so δ > 0;
  - the last grid point inside passes on both sides and at 0;
  - the next one out on the right fails;
  - at ε = 5 and 21, δ is exactly 1 and 3, and h = δ gives exactly ε, so
    the window must be open.
- The prediction asks whether the slope alone, δ = ε/4, works.
  - The answer, "No, but almost", is ringed on the plot only after the
    think window: h = 0.0249 is inside 1/40 = 0.025, and moves x² by
    0.10022001 > 1/10. A test holds that no line before 06_5 says either
    number.
  - Before the answer, the dashed edge at 2.025 and the point where the
    curve leaves the band, 2.02485, are 0.00015 apart. That is about
    2 pixels at 4K, so the picture cannot give it away.
  - It loses at every tolerance, not only one tenth: at the window's edge
    the change is ε + ε²/16 > ε. Equivalently √(4 + ε) − 2 < ε/4 always.
    The model's test finds a losing input inside ε/4 at all six tolerances
    above.
  - For h > 0 the bound is the change itself, so this is a real loss, not
    a loose bound.
- The capped window is the route Sprague's note sets aside:
  - |h| < 1 keeps 1 < x < 3, so |x + 2| < 5;
  - then |x² − 4| = |h|·|x + 2| ≤ 5|h| < ε when |h| < min(1, ε/5), with
    h = 0 giving 0;
  - at 1/10 it is 0.02, the continuation's fact `S.delta_simple`.
  - It is never wider than Sprague's δ, and equals it at ε = 5, where both
    are 1 (test and oracle).
  - The chapter says "both work". It does not call either one the best.
- At 1/10, Sprague's δ ≈ 0.0248456731 is certified to ten places. The
  narration says "about point zero two four eight", its four-place rounding
  (`C6.delta.4dp`).
- No Davis method appears (chapter 07): no coefficient sum and no general
  polynomial. The cap's 5 is derived as |4 + h| < 4 + 1, the factor route,
  and the facts check that it equals the continuation's simple δ. "Next, a
  bound for any polynomial" points ahead without naming it.
- Names: Sprague is said once, in 06_9, and a test ties that line to his
  note (the storyboard row cites "A Note on δ and ε"). The citation on
  screen gives the journal, volume, year and page (facts restated from
  Crossref). The narration quotes nothing. Before narration, his name
  needs a lexicon entry, like every name.
- Reading time, estimated from source at the scratch rate (root measures
  the render):
  - from the question's end to "No, but almost": about 5.1 s (hold and the
    4 s ring);
  - the slope-only row stands about 8 s before the answer;
  - the guarantee row stands about 30 s to the end;
  - the citation about 7 s;
  - "next:" about 3.4 s, for 33 characters.
  - The chapter runs about 93 s silent by the same estimate, over the
    ~85 s target as chapters 03 to 05 do. The storyboard estimate is
    84.5 s.

## Scope limits kept in chapter 07

The owner's scan `IMG_1440.PNG` (SHA-256
`a81690f93493f80b2de65382cd7d32ff55aa90913d0d4ef0cc105aec10c3da39`)
was read directly on 2026-10-20. Allan Davis's “A Further Note on δ and ε”,
Monthly 68 (1961), pp. 567–568, uses x³ + 5x² at 1, where its value is 6.
Recentring gives h³ + 8h² + 13h. The pictured bars at |h| = 1/2 are
computed illustrations; the inequalities supply the proof for every h.
For |h| < 1, the absolute change is at most (1 + 8 + 13)|h| = 22|h|.
Our sufficient choice δ = min(1, ε/22) uses the strict input condition
|h| < δ to obtain a strict final error below ε. It is not claimed to be
the largest possible window. The source instead phrases its restrictions
as strict inequalities for δ.

For a general finite polynomial expansion, C is the sum of the absolute
values of its nonconstant coefficients. When C > 0, δ = min(1, ε/C)
works by the same argument. When C = 0, the polynomial is constant and
any positive δ works; no division by zero occurs. The coefficient sum
and its bar appear only after the prediction window. No derivative or
infinite-series claim is made.

## Narration

The owner resumed these five videos on 2026-10-20. Use the established
Michael reference and whole-thought narration route with measured semantic
pauses. All twelve chapters have root-checked scratch visual renders.
Initial chapter 01 takes are being generated from the frozen 108-cue,
36-unit script in `second-wave-central/epsilon-narrated-01`. Complete
narrated review, independent verification and listening remain outstanding.

## Chapter 09 native review (2026-10-20)

The three-piece uniform-limit proof is implemented. The root native run in
`narration_lab/production-20261020/second-wave-central/epsilon09-native-01`
passed fmt, strict release all-target Clippy, 80 tests, 172 oracle values
and 269 glyph sources (zero failures). Its scratch render is 2,430 frames,
81 seconds at 960×540p30, with empty layout lint and a successful full
decode. Root viewed 11 actual frames spanning the hypotheses, three
budgets, hidden answer, full inequality and final choice order. UBS passed
with zero critical findings; the 212 warnings are triaged in the receipt.
The first failed Clippy log is retained. Independent and listening review
remain outstanding.

The proof follows [MIT lecture 21, Theorem 1](https://ocw.mit.edu/courses/18-100b-real-analysis-spring-2025/mit18_100b_s25_lec21.pdf).
The diagram is a schematic detour among four values; its edge lengths are
not measured errors. Each allocated bound is strictly epsilon/3. Uniform
convergence is sufficient, not necessary: continuous triangular spikes
max(0, 1−|nx−1|) on [0,1] converge pointwise to the continuous zero function
but retain supremum one, so they do not converge uniformly.

## Chapter 10 native review (2026-10-20)

`epsilon10-native-02` is 2,327 frames, 77.566667 seconds at 960×540p30.
The baseline passed 82 tests, 185 oracle values and 294 glyph sources.
Root inspected ten baseline frames and three final frames after aligning
the table columns. Final fmt, strict Clippy, affected tests, native render,
full decode and empty layout lint passed; cues were unchanged. Failures
and the first render are retained. The [sequential criterion](https://www.jirka.org/ra/html/sec_metcont.html)
is Proposition 7.5.2. Exact rational samples illustrate the countersequence
and square-root error ratios; the formulas, rather than a finite table,
prove the general statements. Independent and listening review remain open.


## Chapters 11–12 native review (2026-10-20)

Chapter 11 presents the preimage criterion in both directions, with
openness relative to the domain. The square example uses two disjoint
open intervals; the step function and square-root domain show why
endpoints and the domain matter. The root inspected nine baseline frames,
found overlapping plain-text endpoint numerals, and replaced those four
labels with native TeX. Three corrected frames show separate labels; the
cue file is byte-identical. The corrected scratch render is 2,481 frames,
82.7 seconds at 960×540p30.

Chapter 12 uses exact rational bisection for q²−2 on Q intersect [1,2].
The factor-four estimate proves continuity in that domain; irrationality
of sqrt(2) prevents a zero there. Nested closed intervals and real
completeness supply the separate existence step on [1,2] in R. This follows
[Jirka, Section 3.3](https://www.jirka.org/ra/html/sec_minmaxint.html),
Lemma 3.3.7 and Theorem 3.3.8, with the rational counterexample explained
explicitly. Fourteen actual frames were inspected, including all displayed
brackets, the hidden answer and the closing cards. The scratch render is
2,578 frames, 85.933333 seconds at 960×540p30.

The final native run passed fmt, strict release all-target Clippy, 85 tests,
208 exact oracle values, and 366 glyph sources with zero failures. Both
renders have empty layout lint and successful full decodes. Receipts in
`epsilon11-native-02` and `epsilon12-native-01` retain hashes, source pins,
inspection times and UBS triage. These are root technical and sampled
visual checks; independent review and listening remain pending.
