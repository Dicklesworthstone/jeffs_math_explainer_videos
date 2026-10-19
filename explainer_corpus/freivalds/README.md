# Check a Giant Matrix Product Without Doing the Product Again

A narrated explainer of Freivalds' check (catalog identity `freivalds`,
revision `v1-michael-20261019`). Someone hands you C and says it is A times
B. Redoing the product costs n³ multiplications. Instead, toss a coin for
each column, add up the heads columns of B and multiply by A, add up the
heads columns of C, and compare. A right answer always passes. A wrong
answer passes at most half the tosses, and fresh tosses multiply that down.

**Status.** First slice only: the exact model (`src/model.rs`), every
number (`src/facts.rs`), an independent oracle (`oracle.py`), chapter 01's
scene (`src/chapters/check_with_coins.rs`) and its words and pictures
(`docs/storyboard.md`, generated `script.tsv`, and `narration_units.json`, an
unqualified and unfrozen grouping proposal). Chapters 02 to 04 are named in the storyboard's arc
and not written yet. Nothing here has been compiled, rendered, narrated or
listened to by the lane that wrote it; root's native batch does that. No
narration is requested: after the owner's 2026-10-19 rejection of the
first-wave narration, the coherent route is under independent review.

## What the video claims, exactly

- For n × n matrices of whole numbers (the proof works in any ring), let r
  be a column of n independent fair coins, 1 for heads and 0 for tails. The
  check passes when A(Br) = Cr. B·r and C·r only add columns; A(Br) takes n²
  multiplications.
- If C = AB, every toss passes: A(Br) = (AB)r exactly.
- If C ≠ AB, at most half of the 2^n tosses pass. Proof: pick a column j
  where C is wrong, and pair each toss with the one that differs only in
  coin j. Some row of (AB − C)r changes by a nonzero amount between the two,
  so at most one of each pair passes. That count is a chance only because
  every toss is equally likely: fair coins, independent of each other, and
  tossed after C is fixed (independent of C). With a coin that lands heads
  one time in ten, the example's slip would get past nine tosses in ten.
- k tosses, every coin independent across tosses too, all pass a wrong C
  with chance at most 2^−k. The video's ladder is this bound
  (`facts::theorem_bound`): the pairing proof's one half, for every n. The
  finite sweeps below do not derive it. They check it at three by three
  and two by two (never exceeded, no coin pair passing twice, an equality
  case at each size), and the video refuses to render if a check fails. The example
  meets it exactly, as a separate fact (`F-F4.example_meets_bound`): its one
  slip passes exactly the tosses with tails over its column (4 of 8, and 16
  of the 64 pairs of tosses).
- Edges the facts compute and the oracle recomputes (F-X rows): every
  nonzero mistake with entries −1, 0, 1 at three by three (19,682) and with
  entries −2 to 2 at two by two (624) passes at least one toss (all tails)
  and at most half; 156 and 72 of them pass exactly half; 78 at three by
  three pass a count that is not a power of two (three of eight, which is
  where integers differ from arithmetic mod 2); the same toss used twice
  gains nothing; every toss can be beaten by a wrong C written by someone
  who saw it first.
- **Not claimed:** anything about floating point. In IEEE doubles (`F-X4`,
  `model::ROUNDED_*`), a right answer fails one toss of four and a wrong one
  passes all four, so the exact bound does not transfer; a tolerance makes a
  different test. Also not claimed: that multiplication needs n³ steps (the
  01_2 card says faster methods exist and none is known to need only about
  n² steps), or anything about derandomizing the check.

## Sources, checked 2026-10-19

- R. Freivalds, "Probabilistic Machines Can Use Less Running Time",
  *Information Processing 77* (IFIP Congress 77), North-Holland, pp. 839–842.
  The citation is from the Wikipedia article below and is the one textbooks
  use. **The paper itself was not read** (no readable copy found); one
  lecture note cites a 1979 paper instead. The video names no date for it
  until it is read.
- UIUC CS 473, Fall 2019, lecture 12, "Freivalds Trick and Schwartz-Zippel
  Lemma" (courses.grainger.illinois.edu/cs473/fa2019/lectures/lecture12.pdf),
  read in full for this section: r uniform in {0,1}^n; the lemma for vectors
  over any field or ring, Pr[xᵀr = yᵀr | x ≠ y] ≤ 1/2, with the pairing proof
  this video draws; YES with probability 1 when AB = C; k repetitions,
  error ≤ 1/2^k in O(kn²).
- Wikipedia, "Freivalds' algorithm": the citation above, the d_ij·r_j + y
  proof, and a 2 × 2 worked example (A = (2 3 / 3 4), B = (1 0 / 1 2),
  C = (6 5 / 8 7)) that lists all four 0/1 columns, two of which pass.
- Wikipedia, "Computational complexity of matrix multiplication": the
  schoolbook n³ multiplications; best known exponent about 2.3712 (2026);
  whether ω = 2 is open; the fastest methods are not used in practice.
- The slate's link (Berkeley CS 271 notes, `~sinclair/cs271/n1.pdf`)
  returned HTTP 404 and was not read.

## Closest competing treatments (inspected, not watched)

- Wikipedia's 2 × 2 example above, the closest worked treatment. This video
  differs on purpose: three by three, one slip you cannot see, all eight
  tosses checked on screen, the pairing seen in the tosses themselves, the
  worst case met exactly, and the floating-point boundary made concrete.
- "Freivalds' Algorithm: a randomized algorithm to verify matrix
  multiplication || Monte Carlo", Cyber Enlightener (YouTube yh73ChFj5ik,
  2024-03-29, 40 min 14 s, about 2,000 views) and "Power of randomized
  algorithm: Freivalds' algorithm", Math Geeks (LdJzuNSkHCQ, 2022-11-23,
  8 min 59 s, about 1,550 views). Only their titles, channels, dates,
  lengths, view counts and descriptions were read; their caption tracks
  returned no text. **Neither has been watched.**
- Four searches found no animated explainer; that is search evidence, not
  proof that none exists. A reviewer should watch both videos before the
  brief freezes.

## Pronunciation before any synthesis

Chapter 01 says the letter names A, B and C as matrix names; probe them in
their sentences (A must not be read as the article). "Freivalds" (Latvian;
Rūsiņš Mārtiņš Freivalds, 1942–2016) is not spoken yet and needs its own
lookup and probe (`lexicon.json`).

## Checks

Cheap, no compilation (run in this lane):

```bash
python3 -I -B oracle.py                                   # from this folder: the facts, and 11 cross-checks
python3 -I tools/storyboard.py --check freivalds          # from the repo root: G2, script.tsv current
python3 -I tools/lint_script.py explainer_corpus/freivalds/script.tsv --chapters 01_check_with_coins
```

Native, root's batch: `./x freivalds facts-check`, the crate's fmt, clippy
and tests, the glyph probe, and a 540p scratch-voice draft with the layout
lint and contact and cue sheets read.
