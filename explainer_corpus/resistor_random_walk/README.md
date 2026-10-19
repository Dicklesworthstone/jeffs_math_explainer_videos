# A Resistor Network Is Secretly Answering a Probability Question

Hold the top of a small network of identical wires at one volt and the
bottom at zero. The left junction settles at 3/5 of a volt. Now forget the
battery: a walker at the left junction picks one of its four wires at random
and walks until it reaches the top or the bottom. Its chance of reaching the
top first is 3/5 too. Both numbers obey one averaging rule with the same top
and bottom, and that rule has only one solution.

**Status (revision `v1-michael-20261019`):** this lane writes source only;
it compiles, tests, renders and narrates nothing, and the central lane runs
those. No audio or music exists, and none is accepted.

- **Chapter 01** (`01_the_same_number`, task `.18.1`) is at commit 38e52c0.
  Root's central05 batch (BlueSalmon, Mail 3763) passed fmt, strict release
  clippy, 32 tests, the release build, the 47-fact oracle comparison, the
  glyph probe and a silent scratch render of the opening. Root then viewed
  its final and prediction frames and the three glyph pages (Mail 3772).
  That accepts source and the visual opening only. The task's receipt block
  stays open, and closing it is root's.
- **Chapter 02** (`02_why_an_average`, task `.18.2`) was corrected after
  review 3798. Root's central10 batch (BlueSalmon, Mail 3832) passed fmt,
  strict release all-target clippy, 38 tests, the release build and the
  72/72 fact comparison with the independent oracle (12 checks, numpy
  included). Its silent scratch render draft04 ran 2448 frames, 81.6 s, with
  an empty layout lint. Root viewed the corrected 02_1, 02_3, 02_7 and 02_8
  frames and all eight glyph pages (59 sources), and an independent Opus
  review (3827) accepted the corrected source. That accepts source, native
  checks and sampled visuals only. It adds a scene, its facts (`R-F4` to
  `R-F6`, `R-X4`, `R-X5`, 72 fact keys in all) and their oracle rows. The
  task's commit and closure are root's.

The owner's 2026-10-19 rejection of the first-wave narration (`AGENTS.md`)
holds for this video too: `narration_units.json` is a script grouping, not
a synthesis instruction.

| File | What it holds |
|---|---|
| `src/model.rs` | exact fractions; a network of positive conductances; the voltages by Ohm's and Kirchhoff's laws (Gaussian elimination), the walker's chances by its absorbing chain (`(I − Q)⁻¹`), the balance's determinant, the edges (an island, a family of 729 networks) |
| `src/facts.rs` | every number the video shows or says, and Doyle and Snell's own worked example pinned |
| `oracle.py` | the same numbers computed independently: spanning-forest counts, Cramer's rule, permutation determinants; cross-checks on stderr |
| `src/chapters/the_same_number.rs` | chapter 01's scene |
| `src/chapters/why_an_average.rs` | chapter 02's scene: the current balance at the left junction wire by wire, derived to the average; the walker's first step over the same four wires; the general conductance-weighted rule |
| `src/chapters/draw.rs` | the network picture (side-by-side wires, the battery, values beside wires, arrowheads of flow), playback, line timing and the countdown |
| `docs/storyboard.md` | the words and pictures (the source of `script.tsv`) |
| `narration_units.json` | the script's lines grouped into whole-thought units (three per chapter); written and checked by `narration_units.py` |
| `docs/fact-sheet.md`, `docs/brief.md`, `docs/protected.tsv` | every number's source, the brief, the words every revision keeps |

```bash
python3 -I -B oracle.py                               # every number; 12 cross-checks on stderr (numpy optional)
python3 -I ../../tools/storyboard.py --check resistor_random_walk
python3 -I ../../tools/lint_script.py script.tsv
python3 -I -B narration_units.py --check
./x resistor_random_walk facts-check                  # the Rust facts against the oracle (builds; central lane)
```

## Sources

Doyle and Snell, *Random Walks and Electric Networks* (version 3.02 on arXiv,
math/0001057; derived from the 1984 MAA Carus monograph). Text extracted
from the arXiv PDF on 2026-10-19 UTC; the document is not committed. What
it says, as used here:

- §1.3.1: on a connected network with conductances Cxy, the random walk has
  Pxy = Cxy / Cx, Cx the sum of the conductances at x. Its worked example
  (Ca = 2, Cb = 3, Cc = 4, Cd = 5) is `model::doyle_snell`, pinned in
  `R-P1.steps_from_c`.
- §1.3.2: Ohm's law ixy = (vx − vy)Cxy and Kirchhoff's current law at
  every x other than a and b give vx = Σ Pxy vy; the probability hx of
  reaching a before b is harmonic with the same boundary values; "Hence
  v = h". Verbatim: "When a unit voltage is applied between a and b, making
  va = 1 and vb = 0, the voltage vx at any point x represents the
  probability that a walker starting from x will return to a before
  reaching b." The worked values vc = 7/16, vd = 3/8 and icd = 1/8 are
  pinned in `R-P1`.
- §1.1.5: the maximum principle (a harmonic function takes its maximum and
  minimum on the boundary) and the uniqueness principle (the difference of
  two harmonic functions with the same boundary values is harmonic and zero
  on the boundary, so zero). Its line of N equal resistors, f(x) = x/N, is
  pinned in `R-P2`. The 01_7 lower-third is this argument.
- §1.2.6: on an absorbing chain a harmonic function's interior values are
  fixed by its boundary values, f inside = B f on the boundary, with
  B = NR and N = (I − Q)⁻¹; `Network::walk` computes exactly this.
- p. 43: resistors in parallel may be replaced by one whose conductance is
  the sum of theirs; the video's doubled links are drawn as two identical
  wires on this basis.
- §1.3.3: the walk-based reading of current begins from "a naive view of
  the process of electrical conduction" and notes "realism is not what
  we're after". The video's walker is likewise a model of the equations; it
  makes no claim about electron paths. (Exercise 1.3.6 poses, as a
  question, whether a related button-swapping model "suggest[s] a
  hypothesis about the behavior of conduction electrons in metals"; the
  video does not take that up.) On screen (01_8), "a model of the
  equations, not of electrons" is the video's own statement, and the only
  citation, "voltage equals chance: Doyle and Snell, section 1.3.2", names
  the source of the theorem, not of that caveat.

Not read for this slice: Kakutani (1944) on Brownian motion and the
Dirichlet problem, Kirchhoff (1847) on spanning trees, Lyons and Peres's
*Probability on Trees and Networks*. The oracle's spanning-forest formula is
not taken on trust from them: the oracle checks it against Cramer's rule on
300 random networks of rational conductances.

## Competing treatments

Four web searches on 2026-10-19 UTC (three standard, one extended):
"random walks electric networks voltage hitting probability video
explanation"; "youtube "random walks" "electric networks" OR "electrical
networks" voltage probability animation"; "site:youtube.com random walk
electrical network resistors probability"; "Pólya random walk recurrence
electrical network resistance video SoME youtube". They returned written
treatments, known here only by their titles and the search tool's
summaries (none was opened apart from the arXiv book): lecture slides by
Avrim Blum (TTIC toolkit course), a 2008 Cornell talk based on Doyle and
Snell, Penn and Stanford student slides on
Pólya's theorem by networks, a Szeged seminar abstract, a Columbia talk
abstract and olympiad notes. The only video result was a
university lecture titled "Discrete-time Markov chains - Recurrence and
transience: 2D random walk" (youtube.com/watch?v=K42Q5dUQr-c), which the
search summary describes as proving recurrence from multistep transition
probabilities, not through networks. Nothing was watched, and YouTube's own
search was not used. Whether a close animated treatment of voltage as a
hitting probability exists is unknown. Doyle and Snell state the
correspondence and prove it by the uniqueness principle; this video's
intended difference is the one picture: the same values
sliding into the same junction in both pictures, on a network small enough
to check by hand.

## Scope and remaining limits

- The claim "only one solution" needs every middle junction joined by wires
  to the top or the bottom, and positive conductances. The island (`R-X2`)
  shows it failing; the family (`R-X3`) checks it and the agreement on all
  729 small networks. The model refuses zero and negative conductances.
- Ideal components throughout (identical resistors, an ideal battery,
  steady currents). No physical claim beyond Ohm's law and Kirchhoff's
  current law.
- Chapter 01 is close to the 85 s cap by the scratch-voice estimate (about
  84 s). Chapter 02 is 173 words; its estimate is about 82 s, and root's
  silent scratch draft04 measured 81.6 s (2448 frames). Neither chapter has
  been timed with a real voice.
- Chapter 02 derives the rule at the left junction only, using the right
  junction's known 2/5; why the two averages together have one solution is
  chapter 01's (and a later chapter's) argument. A current is written as
  its drop over the wires' common resistance, never as a bare number of
  amperes.
- Not yet established: any narration or music for either chapter; a
  continuous review of a chapter's every frame (root sampled the frames
  named above); the whole video.
