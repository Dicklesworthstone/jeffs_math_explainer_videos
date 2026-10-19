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
  checks and sampled visuals only. It added a scene, its facts (`R-F4` to
  `R-F6`, `R-X4`, `R-X5`) and their oracle rows. It is committed at d17decc,
  and root closed the task.
- **Chapter 03** (`03_round_after_round`, task `.18.3`) adds repeated
  averaging with real iterates, its facts (`R-F7`, `R-F8`, `R-X6`, `R-X7`,
  104 fact keys in all) and their oracle rows. Root's central17 batch
  (BlueSalmon, Mail 3918) passed fmt, strict clippy, 43 tests, the release build, the 104-fact comparison with the
  oracle (15 checks) and an empty layout lint; its silent scratch draft05
  ran 2489 frames, 82.97 s. An image review of draft05 (EmeraldLeopard,
  Mail 3927) found three cards on screen under 4 s, round 5's 307/512
  cited before its row, and the battery-held table left beside the
  battery-free counterfactual. Those were repaired in source (Mail 3954).
  Root's central20 (Mail 3971) then passed the build, fmt, strict clippy,
  the 5 targeted tests and the glyph probe (100 sources); draft06 ran 2489
  frames, 82.97 s, with an empty layout lint, its four cards up about 13.8,
  13.0, 18.9 and 8.3 s, the table cleared for the battery-free case and no
  prediction leak. Root viewed the revised cards and glyph pages 0, 3, 4
  and 5, and EmeraldLeopard (Mail 3964) independently verified the repair.
  That accepts source and sampled visuals only: no audio, music, 4K,
  continuous-playback or full-video acceptance. It is committed at
  39eed75, and root closed the task.
- **Chapter 04** (`04_only_one`, task `.18.4`) is source written in this
  lane: why the rule has only one solution, by the difference of two
  solutions with the same top and bottom and its peak (the maximum
  principle) on the drawn network, and the island where the argument
  fails; its facts (`R-F9`, `R-X8`, 121 fact keys in all) and their oracle
  rows (18 cross-checks). An independent math review (EmeraldLeopard, Mail
  4027) passed, and root narrowed two cards (Mail 4030: card B's route
  "here, to the top", card C "at most one solution") and bound the spoken
  zero chance to the no-path fact. Root's central29 passed fmt but failed
  strict clippy (`eq_op`, a value less itself in `facts::difference_ends`);
  that function now checks the circuit's ends against the battery and
  returns exact zero (Mail 4047). Root's central31 then passed fmt, strict
  clippy, 51 tests, the 121-fact comparison with the oracle, the glyph probe
  (123 sources, no failures), the build and a silent scratch render, draft09
  (2470 frames, 82.33 s); but its layout lint flagged peak tokens overlapping
  ("M" on "M", 27 %, at 32.6 s) and root's frames showed tokens on junction
  dots. Those were repaired in the scene (Mail 4058): one mark per far end,
  landing clear of its dot, and handovers that fade one M out before the
  next comes in. Root's central33 passed fmt, strict clippy, the six changed
  scene tests, the build and draft10 (2470 frames, 82.33 s, an empty layout
  lint); root viewed 11 frames and all 15 glyph pages, and TanElm's
  independent review of the actual frames (Mail 4069) found no concrete
  defect, its three cards up 14.63, 23.20 and 15.67 s. That accepts source,
  math and sampled silent visuals only: no audio, music, 4K,
  continuous-playback or full-video acceptance. Closing the task is root's.

The owner's 2026-10-19 rejection of the first-wave narration (`AGENTS.md`)
holds for this video too: `narration_units.json` is a script grouping, not
a synthesis instruction.

| File | What it holds |
|---|---|
| `src/model.rs` | exact fractions; a network of positive conductances; the voltages by Ohm's and Kirchhoff's laws (Gaussian elimination), the walker's chances by its absorbing chain (`(I − Q)⁻¹`), the balance's determinant, the paths a peak spreads along and the walk step by step, the edges (an island, a family of 729 networks) |
| `src/facts.rs` | every number the video shows or says, and Doyle and Snell's own worked example pinned |
| `oracle.py` | the same numbers computed independently: spanning-forest counts, Cramer's rule, permutation determinants; cross-checks on stderr |
| `src/chapters/the_same_number.rs` | chapter 01's scene |
| `src/chapters/why_an_average.rs` | chapter 02's scene: the current balance at the left junction wire by wire, derived to the average; the walker's first step over the same four wires; the general conductance-weighted rule |
| `src/chapters/round_after_round.rs` | chapter 03's scene: repeated averaging from a guess of zero, every middle junction at once from the previous round's values; a table of the real rounds and their gaps; never exactly, within a thousandth after five rounds; the ends averaged too as the edge |
| `src/chapters/only_one.rs` | chapter 04's scene: a supposed second solution beside the circuit, their difference (0 at both ends), its peak spreading along wires to the top, and the island of two junctions wired only to each other, where any equal pair keeps the rule and a walker's chance of reaching the top is 0 |
| `src/chapters/draw.rs` | the network picture (side-by-side wires, the battery, values beside wires, arrowheads of flow), playback, line timing and the countdown |
| `docs/storyboard.md` | the words and pictures (the source of `script.tsv`) |
| `narration_units.json` | the script's lines grouped into whole-thought units (three per chapter, each prediction question ending its take); written and checked by `narration_units.py` |
| `docs/fact-sheet.md`, `docs/brief.md`, `docs/protected.tsv` | every number's source, the brief, the words every revision keeps |

```bash
python3 -I -B oracle.py                               # every number; 18 cross-checks on stderr (numpy optional)
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
  to the top or the bottom, and positive conductances. Chapter 04 proves it
  on screen for the drawn network and states the hypothesis; the island
  (`R-X2`, `R-X8`) shows it failing; the family (`R-X3`, `R-X8.family`)
  checks it, a second solution existing exactly where a junction is cut
  off, and the agreement on all 729 small networks. The model refuses zero
  and negative conductances.
- Ideal components throughout (identical resistors, an ideal battery,
  steady currents). No physical claim beyond Ohm's law and Kirchhoff's
  current law.
- Chapter 01 is close to the 85 s cap by the scratch-voice estimate (about
  84 s). Chapter 02 is 173 words; its estimate is about 82 s, and root's
  silent scratch draft04 measured 81.6 s (2448 frames). Chapter 03 is 173
  words; its estimate is about 83 s, and root's silent scratch drafts
  measured 82.97 s (2489 frames) both before the card repair (draft05) and
  after it (draft06). Chapter 04 is 172 words; its estimate is about 82 s,
  and root's silent scratch drafts measured 82.33 s (2470 frames) both
  before the token repair (draft09) and after it (draft10). No chapter has
  been timed with a real voice.
- Chapter 03's repeated averaging updates every middle junction at once from
  the previous round (Jacobi's method), and says so; the one-at-a-time kind
  (Gauss-Seidel) is only a card's comparison. Its gap bound is exact for
  this network, because each middle junction has one wire to an unknown,
  weighted 1/4; it is not claimed for other networks. No finite round is
  claimed to reach the answer: every round's values have power-of-two
  denominators (40 rounds checked, and the card gives the reason for all).
  Each of its lower-thirds stays up at least its reading time, fifteen
  characters a second from when it is fully in, over the lines after it
  where need be.
- Chapter 02 derives the rule at the left junction only, using the right
  junction's known 2/5; why the two averages together have one solution is
  chapter 01's claim and chapter 04's argument. A current is written as
  its drop over the wires' common resistance, never as a bare number of
  amperes.
- Chapter 04 never draws a second solution as if it existed: the copy is
  "a second solution?" with "?" until the argument shows it is the first.
  No path of wires joins the island to an end, so its walker is never
  there after any number of steps (the 12 steps followed agree), and its
  chance of reaching the top first is 0; that 0 is one number, while the
  rule leaves the island's voltage any equal pair. The proof card says "at
  most one solution", what the maximum and minimum argument proves; the
  circuit's voltages are one.
- Not yet established: any narration or music for any chapter; a
  continuous review of a chapter's every frame (root sampled the frames
  named above); chapter 03's full test suite after the card repair
  (central20 ran the 5 targeted tests; central17 ran all 43 before it);
  chapter 04's full test suite after the token repair (central33 ran the
  six changed scene tests; central31 ran all 51 before it); the whole
  video.
