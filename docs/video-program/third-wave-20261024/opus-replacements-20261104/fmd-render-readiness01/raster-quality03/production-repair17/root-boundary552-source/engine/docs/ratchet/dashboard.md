# The fmd-math coverage ratchet

The headline metric of the no-LaTeX pivot (§11.5): what fraction of the
real 3b1b formula corpus typesets natively. The denominator is **frozen**
(G0-4: `9269` distinct strings, `17711` occurrences, corpus hash
`a8325e49e0ce78fcc735533952740e9adeaaa5cb10f9c13d73aaa3ba4bf883fc`, rules_version 1); the numbers may only rise.

**Computed against franken_markdown `126a5304e8d9`.**

| Plane | Occurrence-weighted | Unique-string |
|---|---|---|
| **Parse** | 99.994 % | 99.989 % |
| **Parse + typeset returned Ok** | 99.994 % | 99.989 % |
| **Layout within 5% of real TeX (oracle)** | 25.589 % | 3.010 % |

"Typeset returned Ok" means fmd-math produced a layout without an error. It does not show that the layout is right. The corpus TeX box oracle (fm-tex-layout-oracle-bkbc) lays out 585 strings, sampled by construct class and occurrence (6847 occurrences), and compares each box with real TeX's: 279 are within 5% and 421 within 10%, covering 4532 occurrences within 5%. The oracle row counts only those verified strings against the whole corpus; unsampled strings count as unverified.

## Pending constructs (parse plane)

| Construct | Occurrences blocked | Tracked at |
|---|---|---|
| `\dx` | 1 | franken_manim fm-j5t |

## Pending at layout (parse succeeds)

| Construct | Occurrences blocked | Tracked at |
|---|---|---|

## Trend (by franken_markdown rev)

| Rev | Parse occ. % | Parse uniq. % | Layout occ. % | Layout uniq. % |
|---|---|---|---|---|
| `5310d87a9db3` | 99.577 | 99.266 | 98.916 | 98.188 |
| `4e5066c62818` | 99.577 | 99.266 | 99.379 | 98.921 |
| `76fcbd264d67` | 99.881 | 99.795 | 99.684 | 99.450 |
| `5db49f54f0cf` | 99.944 | 99.892 | 99.746 | 99.547 |
| `4743f78d3e57` | 99.960 | 99.924 | 99.763 | 99.579 |
| `b30516ea9522` | 99.977 | 99.957 | 99.780 | 99.612 |
| `0e727c336281` | 99.989 | 99.978 | 99.791 | 99.633 |
| `82588865c453` | 99.994 | 99.989 | 99.797 | 99.644 |
| `e911be2ad4ff` | 99.994 | 99.989 | 99.994 | 99.989 |
| `68fe29b8af0d` | 99.994 | 99.989 | 99.994 | 99.989 |
| `4c8caa256e30` | 99.994 | 99.989 | 99.994 | 99.989 |
| `f059cac6b861` | 99.994 | 99.989 | 99.994 | 99.989 |
| `bfb3221c53df` | 99.994 | 99.989 | 99.994 | 99.989 |
| `0a0281278edc` | 99.994 | 99.989 | 99.994 | 99.989 |
| `a3578a0f1bd7` | 99.994 | 99.989 | 99.994 | 99.989 |
| `a8aab0db646f` | 99.994 | 99.989 | 99.994 | 99.989 |
| `d664c138440f` | 99.994 | 99.989 | 99.994 | 99.989 |
| `e4e8e2ee1b8e` | 99.994 | 99.989 | 99.994 | 99.989 |
| `798de11bb51c` | 99.994 | 99.989 | 99.994 | 99.989 |
| `bd00a2471566` | 99.994 | 99.989 | 99.994 | 99.989 |
| `3ed1d3320069` | 99.994 | 99.989 | 99.994 | 99.989 |
| `126a5304e8d9` | 99.994 | 99.989 | 99.994 | 99.989 |

## How this is enforced

- Coverage is a pure function of (frozen corpus, fmd-math pin), so the
numbers can only move when `SUITE.lock`'s `franken_markdown` row moves.
An always-on CI test requires `baseline.tsv` to name the current pin:
**a pin bump without a ratchet re-run fails CI.**
- On corpus-bearing machines the ratchet test recomputes all four
counts and fails on any decrease; `RATCHET_UPDATE=1` blesses a
deliberate advance (with this dashboard regenerated in the same
commit).
- Every non-tier-1 construct must fail with its precise, named,
tier-tagged error — audited construct-by-construct against the G0-4
table in always-on CI. Nothing ever fails silently.

## The escalation path (R1, the G2 checkpoint)

If coverage misses a gate's criteria, the response is a **public
amendment with a construct-sprint plan** — never a silent slip: the
gap is named construct-by-construct above, each with its tracked
bead, and the gate review adjudicates the sprint scope in the open.

## Licensing (§15.3)

The corpus strings are 3b1b-authored course material and stay in the
private fixture; this dashboard publishes **numbers, construct names,
and hashes only**. Anyone with the pinned trees can reproduce the
denominator byte-for-byte via `scripts/harvest_tex_corpus.py`.
