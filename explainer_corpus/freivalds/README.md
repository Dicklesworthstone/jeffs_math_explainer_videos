# Check a Giant Matrix Product Without Doing the Product Again

A narrated explainer of Freivalds' check (catalog identity `freivalds`,
revision `v1-michael-20261019`). Someone hands you C and says it is A times
B. Redoing the product costs n³ multiplications. Instead, toss a coin for
each column, add up the heads columns of B and multiply by A, add up the
heads columns of C, and compare. A right answer always passes. A wrong
answer passes at most half the tosses, and fresh tosses multiply that down.

**Status.** The exact model (`src/model.rs`), every number
(`src/facts.rs`), an independent oracle (`oracle.py`), and the scenes of
chapters 01 (`src/chapters/check_with_coins.rs`), 02
(`src/chapters/never_more_than_half.rs`) and 03
(`src/chapters/toss_again.rs`), with their words and pictures
(`docs/storyboard.md`, the generated `script.tsv`, and `narration_units.json`,
an unqualified and unfrozen grouping proposal). Chapter 04 is named in the
storyboard's arc and not written yet.

- **Chapter 01:** source and silent visual checkpoint accepted by root at
  `94988fd`, on root's evidence, not this lane's. Root's central06 ran fmt,
  strict release clippy, all 32 tests, the release build, and the 92
  native/oracle facts with 17 oracle checks. Root read all six glyph pages
  and the changed and prediction frames of draft02 (2,524 frames, empty
  layout lint), and central07 reran the changed SPOKEN test.
- **Chapter 02:** source, native checks and sampled silent visuals accepted
  by root for `.23.2`, on root's evidence, not this lane's.
  - Root read the final changed source, all eight glyph pages, nine cue-end
    frames and twelve pairing and prediction frames.
  - Root's central14 passed fmt, strict release all-target clippy, 42 tests
    and the build. All 150 native and oracle fact rows are equal, the 20
    oracle checks pass, and the 81 glyph sources show zero failures.
  - Draft04's chapter 02 is 2,483 frames (82.766667 s), silent, 960 × 540 at
    30 fps, with an empty layout lint. Its SHA-256 is
    `8dd755a7edaa22d0c5a66bfb209c73f181c078898734cb17e8d751b34d5bd8fd`.
  - EmeraldLeopard's independent review of the actual images (Mail 3894)
    found no blocking defect. Root accepted its three cosmetic notes with no
    change.
  - Not accepted: audio, continuous playback, or the full video.
  - After the `.23.3` wording change to 02_3 ("by the same amount"),
    root's central22 re-rendered it: 2,529 frames (84.30 s), SHA-256
    `121366125aded0259b0cc8666a8d8db3815f6f8adcacec2a796b7459d8a72d0b`.
- **Chapter 03:** source, native checks and sampled visuals accepted by
  root for `.23.3`, on root's evidence, not this lane's.
  - Root's central22 passed fmt, strict release all-target clippy, 48
    native tests and the release build. All 177 native and oracle fact
    rows are equal, and the 108 glyph sources show zero failures.
  - Chapter 03 is 2,371 frames (79.033333 s), SHA-256
    `2911f66ee1d759fa2cd56d4ac636862c664a63eda97f997dc6d30f8ba84d9d9d`.
    Root read the actual cue, reveal and card frames, and the new glyph
    pages.
  - FuchsiaHawk (Mail 3997) independently verified the final source, the
    decoded pixels and how long the cards stay up.
  - Not accepted: audio, continuous motion, the full video, or
    publication.

The lane that writes the code never compiles, renders or narrates it. No
narration is requested: after the owner's 2026-10-19 rejection of the
first-wave narration, the coherent route is under independent review, and
no audio of this video has been generated or accepted.

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
- Chapter 02 checks these on every toss of every shape:
  - C·r − A·(B·r) = (C − AB)·r exactly, so the check sees only the mistake,
    and misses exactly when the mistake's heads columns sum to zero in
    every row (F-F6).
  - The edges of its always-sentences (F-X5): two of one sign in a row hide
    only from tails on both their coins (18 sign and size cases); one too
    big and one too small by the same amount hide exactly when their coins
    agree (6); one wrong entry per row and column hides only from all tails
    (384 patterns).
  - Being wrong in every column is not enough for that last one: the
    1, 1, −1 row is wrong in every column and hides from three tosses.
  - On the actual tosses (F-F8), coin 1's four pairs each move row 2's sum
    by exactly the wrong entry (+1), and no pair holds more than one hidden
    toss. That "one" is the proof's constant (`facts::MISSES_PER_PAIR`,
    pinned as F-F8.misses_per_pair_bound): the example is checked against
    it, never used to find it, and the scene refuses to draw if it exceeds it.
  - The proof needs a column where C is wrong (F-X5.right_column): paired
    by coin 1, over a column the slip is right in, the slip's row sum never
    moves and two of the four pairs hide twice (2, 2, 0, 0). The model's
    test checks the same for every three by three mistake with a right
    column.
- Chapter 03 keeps the example and the theorem apart, and shows each half
  of "fresh" failing:
  - Two fresh tosses: 16 of the 64 ordered ways both miss the slip, the
    misses of the first toss times those of the second (F-F5.two, F-F9).
    That quarter is this slip's, exactly. For any fixed wrong C the bound is
    the theorem's per-toss bound squared, at most a quarter
    (F-F9.two.bound), and 1/2^k for k tosses on the card.
  - One toss reused for both checks is the grid's diagonal: 4 of 8, still
    half, above the bound for two fresh tosses
    (F-F9.same_toss.above_two_bound).
  - A toss seen before C is written can always be beaten. The oracle finds
    a wrong C for each of the eight by search (F-X3.known_toss.fooled), and
    the chapter shows `model::blind_spot`'s: one wrong entry under a tails
    coin, or under all heads 1 and −1 in one row. Each is passed by its own
    toss in the check itself (`facts::blind_spots`).
  - The edges in size: at two by two every toss can be beaten too
    (F-X3.n2.fooled, by search in the oracle); at one by one a heads coin
    cannot (F-X3.n1.heads_fooled), so the card says "from two by two up".
  - Each of those wrong Cs is still a fixed wrong C, and fresh tosses catch
    it 4 of 8 times (F-X3.blind_spot.fresh_catches).
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
python3 -I -B oracle.py                                   # from this folder: the facts, and 22 cross-checks
python3 -I tools/storyboard.py --check freivalds          # from the repo root: G2, script.tsv current
python3 -I tools/lint_script.py explainer_corpus/freivalds/script.tsv --chapters 01_check_with_coins,02_never_more_than_half,03_toss_again
```

The oracle ran 11 cross-checks in the first slice and 17 at `94988fd`. The
three chapter 02 checks brought it to 20 at `ca4f677`, and the two chapter
03 checks bring it to 22.

Native, root's batch: `./x freivalds facts-check`, the crate's fmt, clippy
and tests, the glyph probe, and a 540p scratch-voice draft with the layout
lint and contact and cue sheets read.
