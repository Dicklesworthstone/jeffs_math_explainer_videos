# Exponential D and E: native chapters, staged for root

**Author and disclosure.** LavenderPike wrote the words, models, oracles, scenes and every check here. The maths and the
teaching sequence have had no independent review. The brief itself still calls its sequence an author derivation awaiting
review. Nothing here has been built with Cargo, linked, run as a scene, rendered or heard.

**ElevenLabs Oliver only.** No takes exist for these 23 cues. Nothing here claims a recording, a listening pass or fine sync.

## What root applies

From inside `abc-teaching-stage01/native-crate`:

```
patch -p1 --fuzz=0 -i <this folder>/patches/integration.patch
```

The patch was replayed forward and reverse at fuzz 0 against the stage's bytes (`checks/bytes_check.py`). It touches nine
files and no others:

| File | Change |
|---|---|
| `src/limit_model.rs` | new: `candidate/limit01/limit_model.rs` |
| `src/solution_model.rs` | new: `candidate/only01/solution_model.rs` |
| `src/chapters/tiny_multipliers.rs` | new, two chapters: `candidate/limit01/tiny_multipliers.rs` |
| `src/chapters/only_solution.rs` | new: `candidate/only01/only_solution.rs` |
| `src/main.rs` | `mod limit_model;` and `mod solution_model;` |
| `src/chapters/mod.rs` | header line; `mod only_solution;` and `mod tiny_multipliers;`; three `Chapter` rows |
| `src/facts.rs` | `crate::limit_model::append_facts` and `crate::solution_model::append_facts`, before `self_check` |
| `oracle.py` | limit oracle, then uniqueness oracle, appended |
| `script.tsv` | both slices appended |

**Root's `build01.py` must change once this lands:** its `len(actual)==883` becomes **1079** (883 + 176 + 20), and the bin
name and receipt path are root's to choose. The registry goes from three chapters to six.

`intensity` in the three new rows (0.5, 0.5, 0.45) is my default, by comparison with the A-C rows. Change it freely.

## Cue ids: three chapters, not two

| Chapter key | Cues | Struct | Scratch replay at 0.8 / 1.0 / 1.25 |
|---|---|---|---|
| `04_tiny_multipliers` | 04_1 to 04_6 | `tiny_multipliers::TinyMultipliers` | 131 / 162 / 201 s |
| `05_whole_sum` | 05_1 to 05_8 | `tiny_multipliers::WholeSum` | 178 / 219 / 271 s |
| `06_only_solution` | 06_1 to 06_9 | `only_solution::OnlySolution` | 186 / 229 / 283 s |

D was built as one chapter and is split on purpose. As one it replays at 471 s at factor 1.25, and at 381 s even at
factor 1.0. `MAX_OUTPUT_BYTES` in `crates/explainer_kit/src/encode.rs` is 256 GiB, which is about 368 s of 4K60 at 1.5 bytes
a pixel (my arithmetic, from the skill's own 64 GiB for about 92 s). AGENTS rule 8 keeps engine limits as a real constraint
while lifting the editorial cap. Cue prefixes must equal chapter-key prefixes (`cli.rs:574`, `narration.rs:446`), so the
split changes ids. No takes exist, so this costs nothing now. It would not be free later.

The split is also where the teaching turns: 04 ends with the exact coefficient earned, and 05 opens "Hold k still, and let
n grow." F to I should take 07_ onward. These prefixes do not collide with the stage, which holds 01_ to 03_ only.

**Prediction holds for root's direction map:** `05_1→05_2` and `06_3→06_4`, beside A-C's `01_1→01_2`, `02_1→02_2` and
`03_2→03_3`. Each is authored in the scene as reveal, `hold`, `mark("predict")`, `music_breath(4.0)`, clear, next `say`.

## Words

Exact words are in `candidate/limit01/script-slice.tsv` and `candidate/only01/script-slice.tsv`: 23 cues, 1,501 words, no digits. All 23 are new for Oliver, and no take reuse is assumed.
Longest by scratch estimate: 06_8, 05_3 and 05_8, at 33 to 36 s. Root owns any take-length decision. The skill
lint passes with 0 errors. Its remaining warnings say these cues have five to eight sentences, and each of those sentences
already has its own picture hook.

The brief's prose is kept where it was good, and extended where the picture needed more. Additions a reviewer should weigh:
the four-step recurring example in 04_1, the counting in 04_4 and 04_5, the warning sum in 05_2, the visible aim and first
failed cut in 05_4, the edge table in 05_7, the time swap in 05_8, the failing polynomials in 06_3, the ledger in 06_6 and the
gap in 06_7.

## Pictures, cue by cue

| Cue | What the picture does |
|---|---|
| 04_1 | The rate chapter's four-step drawing. One step taken apart: height, width, rise, `h+h/4=5h/4`. The strip's four cells light as a dot hops. `(5/4)^4` and its printed value. |
| 04_2 | The strip becomes one unit of time; `y'=xy`; cells take `1+x/n`; `(1+x/n)^n`; rate-one table to e as cells refine and old drawings grey. |
| 04_3 | Series graph, live endpoint at 16 cells swept out to 3/2 and across to −3/2; read-outs; "Near is still not a proof"; the question; the power. |
| 04_4 | Sixteen small strips sorted into Pascal's row; x² column framed; six ways, a quarter times a quarter; series coefficient beneath for comparison. |
| 04_5 | Letters left, worked numbers right. Two cells picked, then the same two in the other order: one set counted twice, so `k!`. |
| 04_6 | Shared-out factors, then the chain; six sixteenths through the chain; `k>n` has the factor `1−n/n=0`. |
| 05_1 | 101 columns fill as n climbs, near column read beside far column; question; think gap. |
| 05_2 | "Not enough by itself." n terms of 1/n on the same axis, column three shrinking, total tower staying at one, weight moving outward. |
| 05_3 | Chain range and the separate `k>n` case; bars rise inside green ceilings; n climbs, lids do not move; the warning sum has no such lids (`1+1/2+1/3+…`). |
| 05_4 | Distance at most the gaps; aim one hundredth; first cut at 4, inset shows its tail is too big. n has not moved. |
| 05_5 | Cut moves to 8; only now does n move, head closing over three rows; the order flashes; total below the aim. |
| 05_6 | Live sweep to −3/2; odd bars point down, under mirrored ceilings; same bound, both true distances printed. |
| 05_7 | −3 cell by cell: −2, 1/4, 0, then the climb to E(−3). Red and green brackets mark where the decay reading holds. Algebra never asked for that. |
| 05_8 | The limit on the graph; `x=1` as one special case; time swap; walks both ways; second debt turns green. |
| 06_1 | E, and a gold "?" curve from the same start; proved and not-yet cards; third debt. |
| 06_2 | Product rule for any differentiable y, chain rule's sign shown separately. The rule is off screen for this whole derivation. |
| 06_3 | Degree 2 and 4 beside their products; exact slopes; `E(−x)≠0`; the rule comes back; question; think gap. |
| 06_4 | Level line; equal arrows up and down; cancel; constant; value one at zero. |
| 06_5 | Multiply by E; grid fact supplied; `y=E`; the "?" curve lands on E; `iff`. |
| 06_6 | One-way ledger, built in the order spoken. The result joins last. Struck out: uniqueness theorem, logarithm. |
| 06_7 | Domain with a gap: E near zero, 3E across it; level at 1 and 3; the two hypotheses as cards. |
| 06_8 | `y'=ky`, its test, `y(t)=y(0)E(kt)`; k = 1/2, 0, −1/2 from 2, and the zero start; units card; model question left open. |
| 06_9 | Three boxes into `E(x)=e^x`; the three fitting facts; all three debts in chapter 02's exact words. |

## The brief, line by line

| Brief | Where |
|---|---|
| D picture: fixed width, n cells, `1+x/n` | 04_2 |
| Endpoint on the series graph | 04_3 |
| Binomial-row comparison, columns approaching one | 04_4 to 04_6, 05_1 |
| "not enough by itself" | 05_2, with a counterexample the brief does not have |
| `|term| ≤ |x|^k/k!` | 05_3, tests at the edges in `checks/math_exact.py` |
| Shared tail first, then the head | 05_4, 05_5, enforced by a scene test that n does not move before the cut |
| Exact statement, `k>n` coefficients zero | 04_6, 05_3 |
| Negative x | 05_6 |
| No logarithm, no monotonicity | 05_7 (spoken and printed); checked non-monotone at −3 |
| Decay needs `1+x/n>0` | 05_7 |
| E: `(y E(−x))′=0`, `E(x)E(−x)=1` | 06_2 to 06_5 |
| No general uniqueness theorem | 06_6 |
| Interval containing zero | 06_4, 06_7 |
| `y′=ky`, `y(t)=y(0)E(kt)` | 06_8 |
| Units and model assumptions belong to applications | 06_8 states the units and hands the model question on |
| Three constructions, one object | 06_9 |

## Assumptions and edges, as tried

Words corrected because their first draft failed at an edge:

- "Each correction factor lies between zero and one" is **false** past `k=n+1`, where a factor goes negative. 05_3 now says
  until k passes n, and after that the chain is zero. The chain stays in [0,1] for all k; `math_exact.py` tries this.
- "Degree four breaks the rule less" fails at `x²>12`. 06_3 says "On these axes", and the panel stops at |x| = 3/2.
- "Bigger inputs need a later cut" overclaims. 05_8 says "this argument asks for".

Stated assumptions: y has a derivative at every point of one interval containing zero. "Zero slope on an interval means
constant" is taken as known and named on the ledger as an added tool; it is the mean value theorem, which the film does not
prove. `E'=E` and `E(x)E(−x)=1` come from chapters 02 and 03. `checks/source_checks.py` prints the only A-C lines with the word
"only": 01_5 and 02_6, and both state the debt as open.

Not covered, by choice: one-sided derivatives at the ends of a closed interval. 06_7's card says "a derivative at every
point". A viewer who presses on an endpoint gets no answer from this film.

## Checks and their results

`python3 -I -B checks/run_checks.py SCRATCH` from the repo root. Each output is stored beside its script.

| Check | Result |
|---|---|
| `make_integration` | builds the integrated crate in scratch and writes the patch |
| `bytes_check` | stage bytes match; patch forward and reverse, fuzz 0; `git apply --check` too; both frozen bundles match |
| `native_facts` | real `facts()` against real concatenated `oracle.py`: 1079 rows, `oracle_diff` empty, dict-equal; one tamper caught; all 41 model tests pass |
| `scene_typecheck` | the integrated crate's real `main.rs`, normal and test cfg, `rustc` and `clippy-driver -D warnings`, at the a399d62 metadata |
| `scene_controls` | 8 planted defects, 8 failures, including removing each wiring line |
| `math_exact` | exact-fraction derivations; the edges above; no staged code imported |
| `timing_replay` | real `sentences` and `estimated_starts`; every literal hook replayed; no hook over 0.25 s late at 0.8, 1.0 or 1.25; both answers wait for their gaps |
| `source_checks` | 15 D and 8 E scene tests run for real; 5 planted faults caught; rustfmt; lint; no digits; each phrase occurs once in its cue |
| `text_fit` | estimated extents, none off frame. 67 dynamic calls could not be resolved statically and fall to the layout lint. One overlap printed, 04_3's two cards, a deliberate swap |

Oracle stdout sha256 `38df7e3cf40dd73fcba6e2929c52fd9faa8107379c95c775967fe9159e09e32f`. Native facts sha256 `4efc0ed34de023e8ded21794cecc9ecd168adb68b0f9f211caa7dce3d9e13762`.

**Limits of those checks.** Type-checked is not compiled: no codegen, link or run of any scene. Scene tests test source order,
not behaviour. Scratch timing is not Oliver's. Phrase hooks replay the character-share fallback. `math_exact` is by the same
author as the chapters. Text widths are estimates for TeX. No pixel exists.

## Genuinely unresolved

1. **Independent review of the teaching sequence.** The brief asks for it, and neither I nor these checks supply it. The ledger's
   non-circularity deserves a reader who did not write it.
2. **Does the "?" curve read as a real rival?** It is drawn to stand for the question, and the model test proves it breaks the
   rule, so the picture claims no real solution. 06_5 lands it on E. If that looks like a correction rather than a proof, a
   label or a different picture is needed. Only a look at the draft can say.
3. **101-bar morphs.** 05_1 and 05_2 morph about 100 bars per step. Runtime is unmeasured.
4. **Glyph probe.** `\binom`, `\iff`, `\lim_{n\to\infty}`, `\neq`, `\sum_k`, tall `\left|…\right|` around `\binom`, and `≤`, `×` through
   Text's fallback chain all need the probe. The macros exist in fmd-math at 126a530, and that says nothing about how they look.
5. **`live_number` at negative values.** 05_6 passes `sign: false`, on the assumption that a negative still prints its minus. The sweep
   also passes zero, so −0.0 is possible. Unverified.
6. **Formula digits.** Numbers from models come through the facts. Typed digits in TeX constants (`5/4`, `6/16`, `2!=2`, `1\cdot 1=1`, axis
   ticks on 04_1 and the graph) are checked by `math_exact` or are ticks, but are not fact rows. Rule 2 strictly read would want the
   worked ones moved; I left them as algebra.
7. **`indicate` on the 06_6 arrows.** They are filled shapes, so the Mail 2038 outline problem should not apply, and the pulse's colour
   return has not been seen.
8. **Layout lint, contact and cue sheets, picture review, Oliver takes, listening, fine sync:** all open, all root's lane.

## Bytes

Built on root's stage. `checks/bytes_check.py` re-verifies these rows.

| File | sha256 |
|---|---|
| `patches/integration.patch` | `1be8145f6ad9ccebafe1cdb79a91ba2a44d24d32409d10b2705e3e06691babae` |
| `candidate/limit01/limit_model.rs` | `ec8a857fac17d8ead60cbdf1334a4ff2b22f12feebf02feb321d1341fc99eda7` |
| `candidate/limit01/tiny_multipliers.rs` | `9f584a39bc9c4e0d8a595183c154b49f7d69c188cb3429f7d558857c8936fd57` |
| `candidate/limit01/oracle.py` | `9428e8416b94690354fbdd7958c5028dafa108faab1f44496ec892f6b90de0ed` |
| `candidate/limit01/script-slice.tsv` | `3d37a8319d1cb6f1de44cdf9ebdae79ee32802af9116091c291858ed10ed3915` |
| `candidate/only01/solution_model.rs` | `111838d7bf4f3b9f0053d002294ba8db07452eb166da0eb0ad505a6bd89e15b8` |
| `candidate/only01/only_solution.rs` | `faf42a21e6f7eda62ef7e2674756f318551f320a245dd4015d8cc4d79e739041` |
| `candidate/only01/oracle.py` | `8fede8d2fc789b15ef5ea8efd658c152f33500b77f50e8b2a4cd796de789e4f1` |
| `candidate/only01/script-slice.tsv` | `8300bd1b800e871af77b774cc05e4daea627cda18c2591be43feedbc2ed4157e` |
