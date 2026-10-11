<!-- R1 storyboard review, 2026-10-10, from the reviewer's final answer (packet tools/review.py; storyboards at e26ec36) -->

# R1 storyboard review: capture_recapture (Video A), as Dev, 28

I read the persona prompt and the whole R1 packet: brief, storyboard, fact sheet and the non-numeric sources. I also read the parts of `docs/research/capture_recapture/scripts/verify2.py` (§E, §F) that produce the chapter 07 and 09 numbers, and I recomputed the claims I would want to test. What I ran, and what I left unchecked, is at the end.

## Comprehension check

- **01:** One net dip times 100 can be off by about 30× (88,000 / 45,100 / 2,800 against a true 10,000) because fish crowd together and a net can't tell how much of the lake it covered. So instead we fill the unseen cell of a 2×2 table, 100 | 900 / 900 | ?, with about 8,100.
- **02:** The second net caught 100 of the 1,000 marked fish, so it effectively covered a tenth of the lake, and the lake is about 10,000. k/M ≈ n/N and k/n ≈ M/N are the same cross-multiplication read two ways.
- **03:** The fish don't have to spread evenly, only the marks do. After mixing, whether a fish is marked doesn't depend on where it is, so any net's marked fraction matches the lake's.
- **04:** In a 20-fish lake you can list all 4,845 possible netfuls and sort them by how many marked fish they hold. Dividing by 4,845 gives the hypergeometric distribution, C(M,k)·C(N−M,n−k)/C(N,n), which predicts the catch if you know the lake.
- **05:** Turn it around: fix the k = 3 we saw and compute P_N(K = 3) for every lake size N. That "score" (the likelihood) peaks at 33. It sums to about 16.7, not 1, so it is not a probability over lake sizes.
- **06:** L(N) > L(N−1) exactly when Mn − kN > 0, which equals ab − k·u. So the score rises until the dark cell reaches ab/k, and the best guess is ⌊Mn/k⌋, which is the proportion guess. At Feller's numbers, 9,999 and 10,000 tie.
- **07:** On a known 10,000-fish lake, k is a symmetric bell (SD 9), but Mn/k is skewed high because 1/k is convex. Small studies can get k = 0, and Chapman's (M+1)(n+1)/(k+1) − 1 avoids that at the cost of a small downward bias.
- **08:** For Feller's catch, the plausible lake sizes form a lopsided 95% range [8,400, 12,073], and it narrows mainly as you catch more marked fish. The sentence is right, but I could not tell a colleague how the range is built, because the voice only says "the careful way" (finding D1).
- **09:** Each failure is the second net's marked fraction drifting away from the lake's (bold fish, fish that learn to love or avoid the bait, lost marks, newcomers, no mixing). Deaths that ignore marks change nothing. That is my version of the rule; the draft's own rule is contradicted by 09_7 (finding D6).
- **10:** The same table estimated France's population from births (Laplace), hidden bugs from two reviewers' overlap, and Kosovo deaths from four lists. Each time the cell nobody saw is filled in from the cells we did see.

No chapter is a blocker by the packet's definition. Chapter 08 comes closest.

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 4 | 4 | 5 | 4 | 4 |
| 02 | 5 | 3 | 5 | 4 | 4 |
| 03 | 4 | 3 | 4 | 4 | 4 |
| 04 | 5 | 4 | 4 | 3 | 4 |
| 05 | 4 | 4 | 5 | 3 | 5 |
| 06 | 4 | 5 | 4 | 3 | 5 |
| 07 | 4 | 3 | 5 | 3 | 4 |
| 08 | 2 | 3 | 4 | 3 | 4 |
| 09 | 4 | 3 | 5 | 4 | 4 |
| 10 | 4 | 3 | 5 | 3 | 5 |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| D1 | Dev | 08_5 | major | "Build a range around each study's estimate, the careful way" is the hand-wave I'm allergic to. Chapters 04–06 gave me enough to code everything else, but not the interval. A lower-third saying "exact test inversion" is a name, not a mechanism. | Add one sentence and one picture: "For each lake size, ask whether a catch of 100 would land in that lake's middle 95%; keep the sizes where it would." Reuse 05's fixed-N slice with both 2.5% tails shaded at the endpoints. I checked: at 8,399 P(K ≤ 100) = 0.02494, at 8,400 it is 0.02503; at 12,073 P(K ≥ 100) = 0.02503, at 12,074 it is 0.02498. Bridge from 07: at N = 10,000 the accepted counts are exactly k = 83…118, which is 07's middle 95%. |
| D2 | Dev | 08_5 (picture) | major | "Twenty stacked ranges from the 07 simulation; one misses" doesn't come out of the seed. With `default_rng(20261009)` the first 20 runs are k = 97, 92, 98, 97, 110, 102, 102, 94, 95, 86, 98, 93, 98, 96, 94, 111, 103, 89, 105, 114, all inside 83–118, so none miss. The first miss is run 40 (k = 78). The first 100 runs have 3 misses; all 10,000 have 447 (4.47%). Exact coverage at N = 10,000 is 95.45%. Showing "one in twenty" means picking a window, which is the seed-shopping the hook forbids. | Show the first 100 ranges from the seed (3 miss), or the first 20 with "none missed this time; 447 of 10,000 did". Don't choose a window to get one miss. |
| D3 | Dev | 07_4 vs 08_6 vs Feller footnote | major | In 07 I hear "95% of estimates land between about eighty-five hundred and twelve thousand", which is true: P(83 ≤ K ≤ 118) = 95.45%, giving 8,475–12,048. In 08 I hear 8,400–12,100, and the footnote says Feller's 8,500–12,000 is a 93.6% range. Same spoken numbers, two different percentages, two minutes apart. I would file a bug. | 07_4: say "from about eighty-five hundred to just over twelve thousand" and keep 8,475 / 12,048 on screen. 08_6: add "Close to what we watched in 07, but a different question: there the lake was fixed and the catch moved; here the catch is fixed and the lake moves." Footnote: "Feller's 8,500–12,000, read as a range for the lake, covers ≈ 93.6%." |
| D4 | Dev | 07 Dev note (planning card) | major | "Expect about 10 fish caught twice for ±30%, about 100 for ±10%" are one-SD numbers. The exact 95% ranges I computed at N = 10,000 are: k = 10 (M = n = 316) gives 5,575–20,546, i.e. −44% / +106%; k = 100 gives −16% / +21%. Anyone who sizes a study from this card will undersample badly. | Card text: "95% range: about 10 caught twice → from half to double; about 100 → −16% to +21%; about 400 → about ±8%." Or label the current numbers "± one standard deviation". |
| D5 | Dev | 07_7 Dev note | major | The absorption identity in "my" note is wrong. (n+1)/(k+1)·C(M,k) ≠ C(M+1,k+1): at M = 10, n = 5, k = 3 it gives 180 against 330. The correct form is (M+1)/(k+1)·C(M,k) = C(M+1,k+1); the n+1 is absorbed separately by (N+1)/(n+1)·C(N,n) = C(N+1,n+1). The bias formula in the lower-third is correct: at N = 1,000, M = n = 50 the closed form equals the exact mean 935.2207. Also, the voice line "as if one imaginary fish had been caught both times" sells Chapman as exactly the ad hoc smoothing the note says it isn't. | Fix the identity before it becomes a lower-third. Put both absorptions in the description. Add a clause to the voice: "the ones aren't a fudge; they're what makes the average come out right." |
| D6 | Dev | 09_1 vs 09_7 | major | 09_1's rule says the estimate breaks "when the lake changes in between". Thirty seconds later a third of the lake dies and nothing happens. A rule with an unannounced exception is a bug report. | State the one rule as: "The second net's marked fraction has to match the lake's. Anything that breaks that match breaks the estimate." Then 09_7 follows from the rule ("death doesn't care about marks"), and 09_6 becomes "which lake you measured". |
| D7 | Dev | 09_8 | major | The picture replays 03's habitat dynamics "not flattening", but 6,250 / 25,000 come from a different model in `verify2.py` §F: marked fish confined to one half of the lake, and the second catch drawn 80% from one half and 20% from the other. Anyone who opens the receipts finds the numbers don't come from the simulation on screen. | Either run 03's simulation with no mixing time (second catch at the dock, then at the far shore) and pin those medians, or draw the two-halves model with a label "80% of the net from this half". |
| D8 | Dev | 09_3–09_7 | major | Each gauge number sounds like a property of bold fish, but each comes from one arbitrary parameter: 7,750 means "30% of fish 3× as catchable"; 7,140 / 14,490 mean marked fish 1.5× / ⅔× as catchable; 12,500 means 20% of marks lost; 12,120 means +2,000 newcomers; and 30% die in 09_7. Without the parameter the number is not testable. | Put the parameter on every gauge label. Say the 09_3 one aloud: "if three fish in ten are three times as easy to catch". In 09_6, mark 12,000 (the lake at the second catch) on the gauge, not only 10,000. |
| D9 | Dev | 07_5 | major | Read line by line, the panel doesn't run as shown: `rng`, `M`, `N` and `n` are undefined and numpy isn't imported. The voice says "always check a simulation against something you can compute exactly", but no exact line appears. The good news: with the setup added, it reproduces the fact sheet digit for digit (mean 10,080.002; 2.5% / 50% / 97.5% = 8,474.58 / 10,000 / 12,048.19). `hypergeometric(ngood, nbad, nsample)` is the right argument order (checked in numpy's docstring). | Add greyed setup lines: `import numpy as np`, `rng = np.random.default_rng(20261009)`, `M = n = 1_000; N = 10_000`. Add one exact line, e.g. `p = [comb(M,k)*comb(N-M,n-k)/comb(N,n) for k in range(n+1)]`. In 07_6, note that `M*n/k` at k = 0 gives `inf` and a RuntimeWarning in numpy (a ZeroDivisionError in plain Python). |
| D10 | Dev | 10_4 | major | "It counts bugs. Two reviewers…" sounds like a real result. `verify2.py` labels 30 / 24 / 12 as hypothetical. I'd want a source before quoting this in a design review. | Say "Say two reviewers…". Put Briand et al. 2000 (IEEE TSE 26(6)) in the description as the real-world evidence. Use Mills's seeded bugs as a lower-third: in 1970 he planted known bugs as "marked fish", and his p. 73 line maps lake, fish and marks onto system, errors and seeded errors. |
| D11 | Dev | 10_5 | major | Given an "honest range 47–94" followed by "easy bugs pull it down", what do I actually do with my team's numbers? The range assumes all bugs are equally easy to find, which is false for code. The objection everyone raises first ("one reviewer is senior") is answered only in a persona note. | Add: "Reviewers of different skill are fine; bugs of different difficulty are not. With two reviewers, read sixty as a floor." Cite Briand et al. 2000: with too few inspectors, "underestimation may be substantial". |
| D12 | Dev | 07_7 | major | "It never divides by zero" is true, but at k = 0 Chapman returns 51·51 − 1 = 2,600 for a 1,000-fish lake, which happens in 7.2% of these small studies. That output is a shrug, not a measurement. "Nearly exact once you expect a few" depends on the number: I get −32.5% bias at about 1 expected, −6.5% at 2.5, and −1.1% to −1.6% at about 4 (the lower-third's −1.1% is one end of that). | Voice: "once you expect four or more caught twice". Lower-third: "−1.1% to −1.6% at about 4". One clause on k = 0: "it still gives an answer, 2,600 here, but that's a hint the study was too small." |
| D13 | Dev | 05_6 (picture) | major | "Bars summed: 16.67" as a live sum over the sweep is wrong if the sweep stops at 60: N = 17…60 sums to 9.89. The tail is heavy (to N = 3,000 the sum is still 16.662). A dev who checks with `range(17, 61)` will decide the video is wrong. | Zoom the axis out (log scale) while the counter runs, or label it "all sizes, out to infinity (sizes past 60 add 6.8)". |
| D14 | Dev | 02_1 | major | The footnote card quotes the 3rd edition verbatim, and the fact sheet marks that text UNVERIFIED (archive.org lends it only). "Biologists had been doing it for years" understates it: Schnabel published in the *Monthly* in 1938 (verified). | Check the footnote against a borrowed 3rd edition before P3, or paraphrase it without quote marks. Say "for decades". |
| D15 | Dev | 08_2 + description | minor | The float64 note quotes the smallest normal (2.2×10⁻³⁰⁸). Underflow to zero actually happens below the smallest subnormal, 5×10⁻³²⁴. I checked: `float(Fraction)` and `exp(lgamma…)` both give 0.0 for 5.35×10⁻⁴³⁰. The scipy line "the opposite letters" is vague. | Lower-third on screen at 08_2: "below float64's smallest positive (5×10⁻³²⁴), so code works in logs." Description: numpy `hypergeometric(M, N−M, n)`; scipy `hypergeom.pmf(k, N, M, n)`. The positional slots match our letters but scipy names them (M, n, N), so keyword calls with our letters are silently wrong. (The scipy signature is unchecked here: scipy isn't installed.) |
| D16 | Dev | 08_7 | minor | "Slider on k; the range shrinks." What is held fixed? If M and n are fixed, moving k moves the estimate itself, and the range shrinks partly because N̂ shrinks. | Slide "expected caught twice" with the lake fixed at 10,000 and M = n = √(10,000·k), and show the range in percent. Checked: k = 100 gives −16% / +21%; k = 400 (M = n = 2,000) gives −7.5% / +8.5%, a 0.44× width, so "about halves". |
| D17 | Dev | 03_5 | minor | "The catch is a tiny copy of the lake" contradicts 01, where catches disagreed 31×. The catch copies the lake's marked fraction, not its density. | "…a tiny copy of the lake's mix." |
| D18 | Dev | 10_6 | minor | "About 4,400 victims were named on several lists" reads as the overlap; it is the union of 4,400 unique people. "A full count" overclaims what the Kosovo Memory Book is. | "4,400 different victims were named across four lists… a later name-by-name count found 9,790." |
| D19 | Dev | 10_7 | minor | The receipts card says "every number in this video was computed by code". The Laplace inputs, his odds and the Kosovo figures are RECORDED, not computed (fact-sheet legend). The first person who clicks the link will notice. | "Every number was computed by code or copied from a cited source: link." |
| D20 | Dev | 05_6 | minor | Feller's quote card uses "n" for the lake size, while our screen's n is the green second catch. | Render it as "…that n [our N, lake size] is greater than…". |
| D21 | Dev | 04_7 | minor | The bar labels 28.2 / 47.0 / 21.7 / 3.1 / 0.1 sum to 100.1 (checked). Someone will add them up. | Two decimals (28.17 / 46.96 / 21.67 / 3.10 / 0.10 = 100.00, checked), or a small "rounded" tag. |
| D22 | Dev | 06_4; 03_6, 06_5, 06_6, 09_3, 10_5 | minor | "Marked times caught, minus caught twice times…": "caught" and "caught twice" in one breath, and "overlap" drifts in as a third word for k. | "Marked times the second catch, minus caught-twice times the lake size." Pick either "caught twice" or "overlap" and use only that one. |
| D23 | Dev | 10_2–10_3 | minor | The storyboard still says "In selected towns he counted everyone", which fact-sheet item 5 already flagged. The 300,000:1 odds appear without saying they assume the hand-picked communes are typical. | Use the fact sheet's wording ("In thirty départements spread across France, officials counted everyone in chosen communes"). Add a clause to the lower-third: "(his calculation; it treats the chosen communes as typical)". |
| D24 | Dev | 03 notes, 03_3 | minor | The 03 note still says "stationary density ∝ e^(−U)", which the fact sheet measured as up to 14% off (the shore effect). The 03_3 counts (50 with 5 marked, 10 with 1) are unchecked against any simulation frame. | Adopt the fact sheet's option (c) and delete the e^(−U) sentence so it never becomes a label. Take the 03_3 patches from a real frame, or tag them "illustration". |
| D25 | Dev | 02_5 | delight | "The marked fish measured our net for us" answers 01_6's "a net can't tell how much it searched": k/M is the net's measured coverage. That's the mechanism, not a slogan. | Keep it, and tie it explicitly: "it caught one in ten marked fish, so it searched a tenth of the lake." |
| D26 | Dev | 06_3–06_8 | delight | The adjacent-ratio trick reduces to a sign test, sign(Mn − kN) = sign(ab − k·u), which gives ⌊Mn/k⌋ outright. I checked the ratios at 33/34 exactly (529/528, 288/289) and the exact 9,999 / 10,000 tie. Extra for programmers: in floats, lgamma breaks that tie (logL(9,999) − logL(10,000) = −7.3×10⁻¹²), so a float argmax returns 10,000 alone. | Keep it. Add a 06_8 lower-third: "Exact fractions tie; floating point picks 10,000 by rounding noise." Put the promised cancellation in the description. |
| D27 | Dev | 07_3 | delight | "Ten too few pushes harder than ten too many" (+1,111 vs −909, checked) is Jensen's inequality without the name, with numbers I can test. | Keep it as written. |

## Checks run (credit rules)

- `python3 -I /tmp/r1_dev_a/check.py` (Python 3.14.6, numpy 2.4.1; exact `math.comb` / `Fraction`, lgamma for the large cases; about 4 min). Recomputed:
  - C(20,4) and the five piles; the percent sums.
  - Toy argmax 33 and L(33) = 0.3178; the area sums (17–60, 17–3,000).
  - The ratio identity at 33 and 34; the Feller tie; E[K] and SD 9.0004.
  - 10⁶/90 and 10⁶/110; P(83 ≤ K ≤ 118); E[Mn/K | K > 0] = 10,082.50.
  - The code panel run fresh from the seed.
  - P(K = 0) = 7.198% (1 in 13.89); Chapman 935.22 and plain 1,241.24; Chapman bias at E[k] ≈ 1, 4.096 and 4.
  - 5.35×10⁻⁴³⁰ underflow; coin 7.96%.
  - The CI boundaries 8,399 / 8,400 / 12,073 / 12,074; coverage 95.45% at N = 10,000.
  - The exact CIs for k = 10, 100 and 400.
  - Reviewers: [47, 94], 60, 18, Chapman 58.6; Laplace 71,866.33 / 28.3528 / 28,352,845; 9,790 ∈ [9,002, 12,122].
- Two one-liners with `python3 -I`: misses among the seeded runs (0 / 3 / 447), and the float tie at 9,999 vs 10,000.
- One more one-liner: the absorption identity counterexample, and the Chapman closed form against the exact mean.
- **Read, not rerun:** `verify2.py` §F, the failure-mode models whose parameters I cite in D7 and D8.
- **Unchecked:**
  - scipy's signature (not installed; stated from its documentation).
  - The A-CH03 dynamics numbers and the A-F27 lake draw.
  - The failure-mode medians (7,752 and the rest).
  - The Kosovo and Laplace recorded figures (copied from sources, not computable).
  - The 3rd-edition text.
  - The per-chapter length estimates.
