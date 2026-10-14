# R3 review: capture_recapture, narrated draft (Dev, 28)

Reviewer: dev_28 (working programmer, rusty on probability). Stage: R3, narrated 540p draft, read
through the 15 cue sheets (every tile opened, cropped and upscaled where text was small), the script
and the description in the packet, plus the scene code and the oracle where a sheet only shows the
first 1.5 s of a line.

## What I checked, and how

- **Ran the chapter 9 code panel verbatim** (Python 3, numpy 2.4.1, not the 2.5.4 the description
  names). It prints `10088.586031204479 10082.498763370464`, the exact digits the description
  quotes. The run gave k mean 99.93 and SD 8.96, standard error of the mean estimate 9.21 (gap 0.66
  SE), and the 2.5/97.5 percentiles 8,474.58 / 12,048.19. These match 09_3, 09_5, 09_6 and the
  on-screen output line.
- **Recomputed exactly in Python** (big-int `comb`, `lgamma` in log space): ch 4 piles
  1,365/2,275/1,050/150/5 of 4,845 (28/47/22/3/<1%); 05_4 scores 7.79% (20), 31.78% (33), 15.90%
  (60), peak 33; 05_7 partial sum 17–60 = 9.89, full sum → 16.67; 07_2 ratios 529/528 and 576/578;
  08_5 ratios 80,982,001/80,981,901, 81,000,000/81,000,000, 81,018,001/81,018,101; 10_2 P(K=0) =
  0.0720 (1 in 13.9); 10_3 Chapman mean 935.22 (−6.48%), plain mean given K>0 1,241.24, Chapman at
  k=0 = 2,600; 11_2 P = 10^−429.27 = 5.35×10^−430; 11_4 4.43%; 11_5 7.96%; 12_3 range 8,400–12,073
  and 12_2 "experiment 1" (k=101) 8,326–11,940; Feller's 8,500/12,000 tails 3.47%/2.89%; 14_5 range
  47–94; 13_9 6,250 / 25,000 (from 160 and 40 caught twice); 14_2 Laplace ratio 28.3528.
- **Re-simulated two failure models** with my own seeds: bold fish (3 in 10 at 3×, weighted
  without-replacement draws), median 7,752 (13_3 says ~7,750). Bugs of mixed difficulty (30 easy at
  0.8/0.7, 30 hard at 0.2/0.1): median 41.2, range wholly below 60 in 78% of runs, never above (the
  14_6 card says 77%).
- **Checked the floating-point claims**: Python `math.lgamma` gives logL(10,000) − logL(9,999) =
  +7.28×10⁻¹², and a float ratio scan gives exactly 1.0 at 10,000 (08_5 note). `comb(10_000, 1_000)`
  has 1,410 digits (relevant to porting the panel).
- **Read scene code for timing** where a 1.5 s tile can't show the whole line: how_sure (11_2),
  small (10_2–10_3), failures (13_x relabel order), balance (08_4 cards), comparison (07_4 strikes),
  ring_toss (12_5 slider), cheating (09_6 panel and its printed line), the_dead (15_1 overlay).
- **Unchecked**: the scene's own simulation draws (01 dips 45,100/88,000/2,800; 03_6 corner counts;
  13 bait/newcomers medians at the scene's seeds), the Laplace and Kosovo source figures (Ruth's
  lane), and legibility at final resolution (cue-sheet tiles are 480×270). The three dense
  chapter-15 sheets the packet lists were not in my set, so I did not review them.

## Comprehension check

| ch | what I'd tell a colleague |
|---|---|
| 01 | Mark 1,000 fish, recatch 1,000, find 100 marked, and you can estimate the ~8,100 fish no net touched. Scaling up one patch fails because the net knows its share of the water, not its share of the fish (88,000 vs 2,800 from two dips). |
| 02 | The marked fish measure the net: 100 of 1,000 marked came back, so the net saw ~1/10 of the fish, and N ≈ M·n/k = 1,000·1,000/100 = 10,000 (the "soup guess"). |
| 03 | Fish can stay clumped and the estimate still works, as long as the *marks* mix: once being marked is unrelated to position, any net sees the lake's ~1-in-10 marked share, and small nets just wobble more. |
| 04 | With a known lake, count every netful (C(20,4) = 4,845) and sort by how many marked fish it holds. That is the hypergeometric distribution P(k given N), but in the field you know k, not N. |
| 05 | Flip it: for the observed k = 3, compute P(k given N) for every N. That is the likelihood. It peaks at 33 (= the soup guess) and is a score, not a probability (it sums to 16⅔). |
| 06 | It's one table of P over (N, k): a fixed-N row sums to 1 (what will this lake give), a fixed-k column is the likelihood (which lake gave this catch) and doesn't. |
| 07 | Divide consecutive likelihoods and almost everything cancels: the score still rises while M·n − k·N > 0, which in table cells is a·b − k·u > 0. |
| 08 | The score peaks where the 2×2 table balances (k·u = a·b, independence), i.e. at ⌊M·n/k⌋. The best-score size is the soup guess, with an exact tie at 9,999/10,000. |
| 09 | Simulate 10,000 experiments on a known 10,000-fish lake: k ≈ 100 ± 9, M·n/k is skewed and runs ~0.8% high, 95% land in 8,475–12,048, and the sim's mean matches the exact one (always check a sim against an exact number). |
| 10 | With few expected recaptures, k = 0 happens (7.2%) and M·n/k blows up. Chapman's (M+1)(n+1)/(k+1) − 1 never divides by zero and is within 2% on average once you expect ≥ 4 caught twice. |
| 11 | Turned around: tiny lakes are impossible (5×10⁻⁴³⁰), huge ones implausible, and even the best size gives exactly 100 only 4.4% of the time, which is normal (like 50 heads in 100 flips). |
| 12 | The "careful range" is a test inversion: keep every N whose middle 95% of catches contains our k. For Feller's catch that's 8,400–12,073; it covers the true N ≥ 95% of the time whatever N is, and its width goes like 1/√k. |
| 13 | One rule: the second catch must hold the marking-day marked share. Bold or bait-loving fish push the estimate low; bait-shy fish, lost marks and newcomers push it high; deaths don't bias it (it counts the marking-day lake); unmixed marks make it depend on where you fish. |
| 14 | Same table counts France (births = marks, ~28.4 M) and bugs (two reviewers: 30, 24, 12 shared → ~60 total, ~18 hiding, 47–94), but bugs that are easy for everyone drag the estimate low. |
| 15 | Four overlapping victim lists in Kosovo gave ~10,400 (9,002–12,122), and a later name-by-name count of 9,790 landed inside. The common pattern is a table with one cell nobody can see. |

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 hook | 4 | 5 | 5 | 4 | 4 |
| 02 the marks measure the net | 4 | 5 | 4 | 4 | 3 |
| 03 why the marks spread | 4 | 5 | 4 | 3 | 4 |
| 04 netfuls | 5 | 5 | 4 | 4 | 4 |
| 05 turning it around | 4 | 5 | 4 | 4 | 4 |
| 06 the landscape | 3 | 5 | 3 | 4 | 3 |
| 07 Feller's comparison | 3 | 5 | 3 | 3 | 3 |
| 08 the table balances | 4 | 5 | 4 | 4 | 4 |
| 09 check it by cheating | 5 | 5 | 5 | 4 | 5 |
| 10 small experiments | 4 | 4 | 5 | 3 | 4 |
| 11 how sure | 4 | 5 | 4 | 3 | 4 |
| 12 the ring toss | 4 | 5 | 5 | 4 | 5 |
| 13 one rule for every failure | 4 | 5 | 5 | 4 | 4 |
| 14 the dark cell everywhere | 4 | 4 | 5 | 4 | 5 |
| 15 counting the dead | 4 | 4 | 4 | 4 | 4 |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| D1 | dev_28 | 10_3 (and 12_2) | major | The voice sells Chapman as "It never divides by zero." The screen (in dim 24 px) shows what that means in practice: with none caught twice it answers 51 × 51 ÷ 1 − 1 = **2,600**, for a lake of 1,000. That isn't a fix. It's a silent sentinel. If I ship `(M+1)*(n+1)/(k+1) - 1` on the strength of this line, my code returns a confident finite number exactly when the data has no upper bound. The description knows ("a sign the experiment was too small"), and chapter 12's range handles it honestly (open above when k = 0), but neither the voice nor the screen connects the two. | 10_3 voice, after "It never divides by zero": "With none caught twice it still answers, two thousand six hundred: a number, not an answer." Set the at-zero line in ink, not dim. In 12_2 add "…whatever that size is, even with none caught twice: then the range has no top." That way an implementer learns to report k = 0 as "too small", never as 2,600. |
| D2 | dev_28 | 13_1–13_9, 15_1 | major | Chapter 13 tells me *how* each failure bends the estimate, which is great, but never says I can't *see* it in my data. With two catches, some lake size always balances the table, so the counts alone can't reveal bold fish or bait-shy fish. Then 15_1 shows four lists and says "from the names shared between lists", which hides that using four lists (log-linear models, in the lower-third only) is exactly the remedy for the dependence chapter 13 and 14_6 warn about. As an implementer I come away knowing the failure modes and no way to detect or reduce them. | One sentence closing 13_9: "Two catches can't tell you which rule broke: some lake size always balances the table. A third catch, or one made a different way, lets you check." In 15_1: "From how the four lists overlap (with four, they could check which lists leaned on each other), researchers estimated…". |
| D3 | dev_28 | 02_7, description "Names to search" | major | The estimator is never given its standard name. "Soup guess" works in the video, but I can't google it, find it in a stats library, or say it to a colleague. "Lincoln–Petersen" (or "Petersen estimator") appears nowhere: not in the voice, not on screen, not in the description's search list. The fact sheet already says "Petersen's formula" is safe to use. | A lower-third at 02_7 under the formula: "in textbooks: the Lincoln–Petersen estimator". Add "Lincoln–Petersen estimator" and "closed population" to the description's Names to search. Optionally one voice clause, "Call it the soup guess; textbooks call it Petersen's." |
| D4 | dev_28 | 11_2 | major | The picture runs ahead of the words, and the one programmer hook in this chapter is the casualty. From the scene code: "chance ≈ 5 × 10⁻⁴³⁰" is written and the 430-zero strip scrolls about 2.6–6.6 s into the line, fading by about 7.2 s. The line runs about 14.5 s, and "The chance of that is…" is its third sentence, starting around 9 s. So the zeros are gone before the voice mentions zeros, and while they scroll I'm hearing about 900 unmarked fish. The float64 label ("a computer's everyday numbers (float64) round to zero past here") rides the moving reel and is still for only 1.0 s. (Inferred from the code and cue times; the 1.5 s tile shows only the marker.) | `line.at(stage, "The chance of that")` before writing the chance and starting the reel. Hold the reel at the float64 cut for ≥ 2.5 s, or detach the label so it stays put after the strip moves on. Optional voice clause: "…too small for a computer's everyday numbers, so code works in logarithms." |
| D5 | dev_28 | 09_6 | minor | The panel is correct (I ran it). But `comb(M, j) * comb(N - M, n - j) / comb(N, n)` only works because Python ints are unbounded: `comb(10_000, 1_000)` has 1,410 digits. Port that line to Rust, JS or C++ with doubles and you get inf/NaN. Separately, the yellow run-box's top edge (buff 0.05) cuts through the bottom of `marked = np.arange(N) < M`, so the `<` is half hidden on the tile. | Add a comment line to the panel and the description: `# exact: Python ints are unbounded; elsewhere sum exp(lgamma(...))`. Give the highlight boxes buff ≈ 0.12, or open the line spacing, so no border sits on code. |
| D6 | dev_28 | 10_2 | minor | Chapter 9 just handed me `est = M * n / k`. Chapter 10 shows k = 0 as a fraction over 0 with a dot flying off, but never says what *that code* does: numpy gives `inf` with a RuntimeWarning, and `est.mean()` becomes `inf`. A programmer who reruns the panel with M = n = 50 and N = 1,000 hits this immediately. The description has it; the video doesn't. | A small "for programmers" tag at 10_2 beside the 50×50/0: "the chapter 9 code: est = inf (RuntimeWarning), so est.mean() = inf". |
| D7 | dev_28 | 07_1–07_2 | minor | Algebra-heavy and under-motivated for me. Why do I care that the ratio cancels? The real payoff, that this is how code walks the curve (one multiply per step, logs for big lakes), sits only in the 07 "precisely" box. 07_2 also packs four products and two comparisons into one sentence. | Promote the payoff into 07_1's voice: "…almost everything cancels. It's also how code computes the curve: one multiplication per step." Keep 07_2's numbers on screen as a big "529 > 528 ↑ / 576 < 578 ↓" so the ear doesn't have to hold them. |
| D8 | dev_28 | 06_1–06_4 | minor | On every sampled frame the 3D landscape is hard to parse. At 06_3 the "lake of 33" slice is seen obliquely behind a translucent plane, and the row's bars aren't readable. The HUD text carries the idea, but the picture adds little over chapter 5. For a programmer, the cleanest version is a 2D array. | Add a HUD caption at 06_3/06_4: "one table P[N][k]: each row (fixed N) sums to 1 over k; our column k = 3 summed over N gives 16.67". Or inset the sliced row as a flat 11-bar chart with "Σ = 1.000". |
| D9 | dev_28 | 08_5 ("precisely" note) | minor | "an lgamma log-likelihood picks 10,000 by rounding" is a claim about a library, stated as if it held everywhere. I checked: Python's `math.lgamma` gives +7.28×10⁻¹² in favor of 10,000, and a float ratio scan ties exactly at 1.0. Another lgamma (libm, scipy, R) could break the tie the other way or not at all. The description words it correctly ("may break the tie either way"); the on-screen note doesn't. | On screen: "exact fractions tie; Python's math.lgamma happens to pick 10,000, and another lgamma may pick 9,999". |
| D10 | dev_28 | 13_7 (pattern in all 13_x) | minor | At 13_7 the title changes to "deaths" (relabel, ~0.9 s in) before the reset swaps the readout (~1.4–1.8 s). The cue tile shows "deaths … ≈ 12,120", with the needle back at 10,000 and the readout not yet updated. Paused there, it reads as "deaths push the estimate to 12,120", the opposite of 13_8. | In `Bench`, reset (needle and readout) before relabel, or run both in one play, so a title never sits over the previous case's number. |
| D11 | dev_28 | 08_4 | minor | The "lake size 10,000" live readout (y ≈ 1.05) is half covered by the incoming "top score" card (top edge ≈ 1.05) until the cards merge, about 1–2 s. The tile shows "10,000" cut in half behind the card. | Fade the lake-size readout before the cards come in, or drop the two cards by ~0.4. |
| D12 | dev_28 | 14_6 | minor | "Reviewers of different skill cause no trouble" is true for *overall* skill (each reviewer has a fixed chance per bug), but I'd over-read it. Real reviewers specialize (security vs. perf). When each finds bugs the other misses, the caught-twice cell shrinks and the estimate runs *high*, the trap-shy case of 13_4. That's oversimplified in a way that matters for the exact use case the video pitches to me. | "Reviewers of different skill cause no trouble. Reviewers who hunt different kinds of bug push it up, and bugs that are easy for everyone pull it down, like the bold fish." |
| D13 | dev_28 | 14_4, 15_1 | minor | Nobody says that "caught twice" requires *matching*. For bugs it's deduplicating two reports of one defect; for Kosovo it's matching names across lists. A missed match is a lost mark (13_5, estimate up), and a false match does the reverse. That's the most likely failure in my world (crash dedup, bug trackers), and the video already has the vocabulary for it. | One clause in 14_4: "…without comparing notes, and as long as you can tell when two reports are the same bug: a missed match is a mark washed off." |
| D14 | dev_28 | 12_1–12_3, description "Code" | minor | Chapter 12 gives the careful range in words and names it ("exact test inversion" note, "95% confidence interval" card), which is implementable, but it's the hardest piece to write and the only code offered is chapter 9's. A ~10-line snippet would let me reproduce 8,400–12,073 directly (I did, in about that many lines). | Add a second snippet to the description: log-space pmf via `lgamma`, the two one-sided tails, and a scan over N that keeps sizes where both tails exceed 2.5%. It should print `8400 12073`. |
| D15 | dev_28 | 09_6 | delight | Best moment for me. Runnable code, the seed on screen, the output printed under it, and "apart by 0.66 of the simulation's own wobble (standard error 9.2)". I ran it verbatim and got the exact digits, `10088.586031204479 10082.498763370464`, on numpy 2.4.1 as well as the stated 2.5.4. "Always check a simulation against something you can compute exactly" is the lesson I'd put on a team wiki. | Keep. Optionally add "(also 2.4.1)" to the description's numpy note. |
| D16 | dev_28 | 04_3, 04_7 | delight | This is how I'd count it in code: 20·19·18·17 = 116,280 ordered picks, ÷ 4! orders = 4,845 = "20 choose 4". Then the 10 × 105 grid for the two-marked pile, "like shirts and pants". It ends with the general formula on screen. I could write `pmf(N, M, n, k)` from this chapter alone. | Keep. |
| D17 | dev_28 | 08_5 | delight | The exact tie at 9,999/10,000 with the integer ratios on screen (80,982,001/80,981,901 > 1, 81,000,000/81,000,000 = 1, 81,018,001/81,018,101 < 1, all recomputed), plus the honesty about floating point, is exactly the edge case I'd hit in a unit test. | Keep (with D9's wording fix). |
| D18 | dev_28 | 12_2, 12_5 | delight | The ring toss pins down what "95%" promises (coverage ≥ 95% at every true N, 95.45% at this one), and the slider gives me a planning rule I can use: "rule of thumb 1 ÷ √(caught twice)", width × ~0.5 per 4×, "one size: 10,000" when you catch the whole lake. The "experiment 1: 101 caught twice → 8,326–11,940" readout is right (recomputed). | Keep. |
| D19 | dev_28 | 13_2–13_9, 14_6 | delight | Every failure is a seeded model with its parameters and "median of 400 runs" on screen, so I can reproduce it. I did for two: bold fish median 7,752 vs the voice's ~7,750, and mixed-difficulty bugs median 41.2 with the range below 60 in 78% of runs vs the card's 77%. Death not biasing it ("counts the lake on marking day") is the kind of non-obvious result that sticks. | Keep. |

## Verdict

Ship-worthy for a programmer after four fixes. The mechanism (counting netfuls → likelihood → MLE =
M·n/k → test-inversion range) is precise enough to implement, and every number I recomputed holds.
Chapter 9's panel runs as shown. The four majors are about practice, not math: say that Chapman's
k = 0 answer is meaningless (D1), say that two-catch failures are invisible in the counts and what
helps (D2), give the estimator its textbook name (D3), and put 11_2's zeros and float64 label on the
words that describe them (D4).
