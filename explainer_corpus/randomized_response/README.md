# How to Learn a Secret About a Crowd Without Learning Each Person's Answer

Ask a class of twenty whether they ever copied homework and a yes is a
confession. Give everyone two private coins instead: heads on the first, tell
the truth; tails, let the second coin answer, heads for yes, tails for no.
On average the survey hears seven yes and thirteen no from a class in which
four copied, and gets back to four by taking away the second coin's yeses (a
quarter of the class) and doubling what is left. Yet no answer is harmless:
a yes is three times as likely from someone who copied, and asking the same
person again with fresh coins multiplies that. Counted over every student's
four coin outcomes, a yes means the giver copied with chance three in seven,
not three in four; a no, one in thirteen; before any answer, one in five.

**Status (2026-10-19, revision `v1-michael-20261019`):**
- Chapter 01 (`01_the_coin`, Beads `.8.1`) is committed at eb81fbc. Root
  verified it (Mail 3759): fmt, strict Clippy, the tests, the release build,
  facts against the oracle, glyph pages, and a silent 540p30 scratch draft of
  81.1 s with zero layout findings, with the cue-end and question/reveal
  frames viewed. It was also source-reviewed (Mail 3741; four low findings,
  fixed). That verification covers the source, native checks and visuals;
  root's Beads receipt for it is separate.
- Chapter 02 (`02_what_a_yes_says`, Beads `.8.2`) is accepted by root for its
  source, native and sampled-visual criteria only (Mail 3831):
  - native checks: fmt, strict release all-target Clippy, 37 tests, the
    build, and 105/105 facts exactly matching the oracle (26 checks);
  - a silent draft of 2,531 frames (84.37 s) with no layout findings;
  - frames viewed: all 11 cue ends, 7 glyph pages, the prediction and its
    reveal, and both corrected cards (two lines, legible, clear of the
    chance line's caption).

  It was independently source-reviewed (TanElm, Mail 3802; the two layout
  and domain repairs were then re-reviewed with no remaining findings, Mails
  3827 and 3829). This acceptance does not cover narration or the full video.
- Chapter 03 (`03_the_price_of_the_coin`, Beads `.8.3`) is accepted by root
  for its source and sampled silent visuals only (Mail 3980). Root closes
  the Bead after verifying this commit. The acceptance took two steps:
  - Root accepted draft06 provisionally (Mail 3932), then held it because
    its sample card was on screen too briefly to read. FuchsiaHawk measured
    the cards (Mail 3941): the sample card had 4.20 s for 143 characters,
    and the denser runs and clamp cards 5.13 s for 126 and 5.27 s for 117.
  - The repair (Mails 3944 and 3952) brings the sample card in at 03_9's
    "Sample twenty", where the fixed class's card goes. It also shortens
    the runs and clamp cards, and the clamp card now reads "allow negatives:
    …" so it cannot be misread. The repair adds no wait.

  The checks:
  - central17 (Mail 3916): fmt, strict release all-target Clippy, 44 tests,
    the build, 123 facts against the oracle (31 checks) and 105 glyph
    sources with no failures;
  - central18 (Mail 3932), after the row names moved inside the title-safe
    edge: fmt, strict Clippy, the layout test, the build, and draft06, a
    silent draft of 2,338 frames (77.93 s) with no layout findings;
  - central21 (Mail 3980), after the card repair: fmt, strict release
    all-target Clippy, the build, 5 targeted tests (the 39 unchanged ones
    filtered out) and 103 glyph sources; draft07, 2,338 frames (77.93 s)
    with no layout findings and unchanged cues;
  - frames viewed: on draft06, the 10 cue ends, the prediction and its
    reveal (the answer hidden until 35.2 s), the 10 glyph pages, the moved
    labels (22.5 s) and the closing card (77.0 s); on draft07, the frames at
    12, 50, 60, 64 and 76 s.

  FuchsiaHawk reviewed three things independently:
  - the source (Mail 3899), reconstructing the seeded draws and the exact
    distribution; one card defect was found and fixed;
  - the rendered images (Mail 3926);
  - draft07's cards (Mail 3965): the sample card is fully readable for
    15.90 s (was 4.20 s), the clamp card's wording is complete, and no label
    drops out.

  This acceptance does not cover narration, audio, continuous playback, 4K
  or the full video.
- Chapter 04 (`04_asking_again`, Beads `.8.4`) passed root's native batch
  and its actual-frame review (Mail 4053), for source and sampled silent
  visuals only. Root verified the commit, 8d6f615, and closed the Bead
  (Mail 4061).
  - Before the batch: FuchsiaHawk checked the mathematics independently
    (Mail 4026). Root asked for a clearer 04_8 sentence, "The majority of
    three answers is right …" (Mail 4028).
  - central28 stopped on two strict Clippy findings in the scene (Mail
    4036). The fixes moved a test-only constant into the tests and replaced
    a test's `vec!` with an array, with no allowance and no changed
    assertion.
  - central30 (Mail 4053): fmt, strict Clippy, 49 tests, 137 facts against
    the oracle, 131 glyph sources with no failures, the build and the
    render, with no input drift. Draft09 is a silent draft of 2,436 frames
    (81.20 s) with no layout findings.
  - Frames viewed: root saw 7 actual frames (13, 21, 29, 45, 60, 73 and 80
    s) and all 13 glyph pages.
  - EmeraldLeopard reviewed 15 frames and 10 fps regions independently
    (Mail 4049). It confirmed the 27-against-1 grids and the prediction left
    blank until 04_4, then the priors, the majority for both truths and the
    memoized columns. Its measured card times were 23.6, 19, 12.8 and 18 s,
    with no layout findings. Its one optional note, that the memo card
    states its conclusion while 04_9 is still explaining, was accepted for
    this slice without a change.

  This does not cover narration, audio, continuous playback, 4K or the full
  video.
- Chapter 05 (`05_the_dial`, Beads `.8.5`, the last in the storyboard's
  arc) passed root's native batch and actual-frame review, for its
  mathematics and sampled silent visuals only (Mail 4132). Root closes the
  Bead.
  - The first version's draft10 was withheld. TanElm's review (Mail 4095)
    found 05_6 and 05_7 using "wobble" for a ratio of per-person variances,
    against chapter 03's miss size.
  - The repair (Mail 4097) speaks the fixed class's misses instead, about 2
    rather than 4, and about 9. It adds the rule-following condition to the
    closing card and holds every card for its reading time.
  - central42: fmt, strict all-targets release Clippy, 55 tests, 190 facts
    against the oracle, 175 glyph sources with no failures, the build and
    the render, with all 20 input pins unchanged.
  - draft11: `05_the_dial.mp4`, silent, 960×540 at 30 fps, 2,425 frames
    (80.83 s), no layout findings, SHA-256
    f4e69356dc9da2acfe58dd8366ef0d05698b6c9a2f2c958c6a87fbd479d75d22. Root
    viewed 10 changed frames.
  - Independent reviews:
    - Mail 4119 checked the revised miss mathematics, its rounding, the
      assertions and the scope.
    - Mail 4128 checked the actual frames: the "Seven" and miss phrases
      are on time, no answer is leaked before its reveal, all five cards
      stay up long enough, and the endpoints are right.
    - A doubled-looking table root saw near 70 s was checked frame by
      frame and found not to be a defect. Optional alignment nits were not
      blockers.

  This does not cover narration, audio, continuous playback, 4K or the full
  video.

The owner rejected the first wave's narration on 2026-10-19 (AGENTS.md,
"Owner rejection and withdrawal"): `narration_units.json` and the pause plans
are text inputs only, and nothing here is to be synthesized with the
rejected method. No audio or full-video acceptance exists.

| File | What it holds |
|---|---|
| `src/model.rs` | the mathematics: exact fractions, the coin rule, the class on average, every student's outcomes and the shares they give, odds, the yes count's exact distribution and the estimate's mean with and without rounding negatives up, a class's coins from one seeded draw, sampling variance, repeated and memoized answers |
| `src/facts.rs` | every number the video shows or says (175 keys), three pinned to their sources; chapter 03's eight seeded classes (`runs`); chapter 05's dial settings (`DIAL`) |
| `oracle.py` | the same numbers by independent methods (the standard library, and NumPy to replay the seeded classes), 41 cross-checks |
| `src/chapters/the_coin.rs` | chapter 01's scene |
| `src/chapters/what_a_yes_says.rs` | chapter 02's scene |
| `src/chapters/the_price_of_the_coin.rs` | chapter 03's scene |
| `src/chapters/asking_again.rs` | chapter 04's scene |
| `src/chapters/the_dial.rs` | chapter 05's scene |
| `src/chapters/draw.rs` | the video's shapes, playback, line timing and think time |
| `docs/storyboard.md` | the words and pictures (the source of `script.tsv`), with each chapter's semantic pause plan |
| `narration_units.json` | the lines in coherent takes for a later synthesis (text only for now) |
| `docs/fact-sheet.md`, `docs/brief.md`, `docs/protected.tsv` | each number's source; the brief; the words every revision keeps |

```bash
uv run --no-project --with numpy python3 -I -B oracle.py # every number, and 41 cross-checks on stderr
python3 -I ../../tools/storyboard.py --check randomized_response
python3 -I ../../tools/lint_script.py script.tsv
./x randomized_response facts-check                      # the Rust facts against the oracle (builds)
```

## Sources

- **Dwork and Roth**, *The Algorithmic Foundations of Differential Privacy*,
  Foundations and Trends in Theoretical Computer Science 9 (2014). Read
  2026-10-19 from the authors' PDF (cis.upenn.edu/~aaroth/Papers/privacybook.pdf).
  §2.3 and §3.2 give this exact two-coin rule (their *tails* tells the truth;
  for a fair coin the swap changes nothing), the expected yes share
  (1/4)(1 − p) + (3/4)p = 1/4 + p/2 and the estimate "twice the fraction
  answering Yes minus 1/2". Claim 3.5: the rule is (ln 3, 0)-differentially
  private, Pr[Yes | Yes] = 3/4, Pr[Yes | No] = 1/4, and both answers' ratios
  are 3. Theorem 3.16: under composition "the epsilons and the deltas add up".
  §3.7: "Randomized Response is due to Warner".
- **Warner**, "Randomized response: a survey technique for eliminating evasive
  answer bias", *JASA* 60(309):63–69 (1965), as Dwork and Roth cite it.
  **Not read**: the paper is paywalled and its PubMed record did not load.
  Warner's own design is a spinner choosing between "I am in group A" and "I
  am not", with estimator [λ̂ − (1 − P)]/(2P − 1) and variance
  π(1 − π)/n + P(1 − P)/[n(2P − 1)²], **from secondary sources only** (the
  QuantEcon lecture "Randomized Response Surveys", read 2026-10-19, and a
  search summary of later papers). The two-coin rule gives exactly the answer
  chances of that spinner at P = 3/4, so its extra variance is 3/(4n)
  (`Channel::warner_extra_variance`); the video never calls the coin rule
  Warner's.
- **Erlingsson, Pihur and Korolova**, "RAPPOR: Randomized Aggregatable
  Privacy-Preserving Ordinal Response", CCS 2014 (arXiv 1407.6981). Read
  2026-10-19, §1: the one-coin and two-coin rules; "assuming absolute
  compliance with the randomization protocol (an assumption that may not hold
  for human subjects ...)"; memoization keeps ln(3)-differential privacy
  "even with an infinite number of responses" when the true value does not
  change or changes in an uncorrelated way; without it, 75 yes in 100 answers
  leave "No" a 1.39 × 10⁻²⁴ fraction of cases (`R-X3`, recomputed exactly as
  1/(3⁵⁰ + 1)).

## Closest treatments

None of these was watched as video; what each covers comes from text read on
2026-10-19.

- *Randomized Response Explained*, Security and Privacy Academy (YouTube
  `SD7EzSkBXug`): title and channel from YouTube's oEmbed metadata only. **Not
  watched; no transcript read; its content is unknown.**
- Pearce and Jiang, *How randomized response can help collect sensitive
  information responsibly* (Google PAIR explorable, 2020), through a text
  fetch: a plagiarism question, one coin (heads reports plagiarism), the
  estimate by doubling the non-plagiarism rate, simulated error spreads and
  the privacy/accuracy trade-off. No repeated answers or privacy budget. Its
  interactive pictures were not seen.
- University of Washington CSE 163, *Randomized Response* lesson: the same
  two-coin rule with heads telling the truth and a cheating question; a worked
  1/3 → 5/12 example, inverted in words; no variance, no repeated questions.
- QuantEcon, *Randomized Response Surveys*: Warner's spinner, the estimator
  and its two-part variance, an MSE comparison with untruthful direct
  answers; no repeated answers or composition.

What this video adds, if its later chapters are built: the rule's chances
counted from four coin outcomes on screen, both answers' exact ratios, the
exact scatter of one class of twenty, the variance split into sampling and
coins, and repeated-answer leakage counted over all 64 outcomes beside a
memoized answer. The question overlaps the common one (cheating, plagiarism);
no vacant niche is claimed.

## Scope limits

- The arithmetic assumes everyone follows the rule with fair, private coins.
  Whether real people answer more honestly under it is an empirical question
  this video does not answer and no participation or honesty effect is
  claimed.
- Chapter 01's counts are average counts (every coin outcome equally often),
  said and carded as such; a real class of twenty scatters, standard
  deviation √15 ≈ 3.87 people (`R-F7`).
- The class is fixed and every member answers: its scatter is the coins'
  alone, a variance of 3n/4 for the estimated count (15 for the class) or
  3/(4n) for the estimated share. Drawing n people independently, with
  replacement or from a population so large that it makes no difference,
  adds the ordinary sampling term: n·p(1 − p) for the count, p(1 − p)/n for
  the share (`R-F8`). Drawing all N = n people without replacement adds
  nothing, which is the fixed class again. The two stay apart.
- Chapter 03's eight classes are real draws, not a selection: one raw draw
  each from `PCG64DXSM(3)`. Seed 3 is the chapter's number, fixed before any
  draw was seen and the only seed tried. Two of the eight fall below zero,
  more than the exact 9%; eight are too few to show rates, and the chapter
  says so. Its chances come from the exact distribution over all 4²⁰
  configurations, not from those draws.
- Negative estimates are kept. The estimate is right on average only
  because it can go below zero; rounding negatives up to 0 would raise its
  average from 4 to 4.26 (`R-F14`).
- Chapter 02's chances (3/7 after a yes, 1/13 after a no) belong to this
  class, where one in five copied. The factor of three is the rule's alone;
  the chance it leads to depends on the class: 0 if nobody copied, 3/4 if half
  the class did (`R-F12`, `R-X5`). The likelihood ratio (3) and the chance
  (3/7) are kept apart on screen.
- "Three times, never more" is about what one answer says about its own
  giver under this rule. Fresh coins on each repeat multiply the worst ratio
  (3, 9, 27, …); a memoized answer does not, while the truth stays put.
- Chapter 04 counts one student asked three times about one secret that
  does not change, with independent fresh coins, or with one flip whose
  answer is resent (the memoized answer). Its 27/31 is this class's chance,
  where one in five copied (27/28 from an even start, 0 if nobody copied;
  `R-F15`). It makes no claim about repeated surveys in general, about other
  questions asked of the same person, or about what RAPPOR's full design
  guarantees.
- Chapter 05's dial changes only the chance t of telling the truth. The
  second coin stays fair, and the spinner of four slices just replaces the
  first coin, so it is not Warner's spinner. The voice speaks chapter 03's
  misses: how far the fixed class of 20's count usually strays, the square
  root of its variance. That is about 4 at 1/2, about 2 at 3/4 (1.97) and
  about 9 at 1/4 (8.66). The table shows the coins' variance per person,
  (1 − t²)/(4t²). The count's variance is 20 times it, the share's 1/20 of
  it, and a sample adds p(1 − p)/n (chapter 03). None of these is a chance
  that anyone copied, and no ratio of per-person variances is spoken as a
  miss. The edges are exact: at t = 1 no ratio bounds a yes; at t = 0
  there is no estimate. The closing promise is for one answer each, with
  everyone following the rule with fair, private coins.
- The class, seats and question are invented; no real survey data is used.
