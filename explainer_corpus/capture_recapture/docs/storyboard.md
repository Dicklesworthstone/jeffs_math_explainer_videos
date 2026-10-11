# Storyboard — How to Count Fish You've Never Seen

The hand-edited source of this video's words and pictures (from R1, 2026-10-11;
it started as plan §9.4, revision 2). `python3 -I tools/storyboard.py
capture_recapture` checks it (gate G2) and writes `script.tsv` from the words
column. Facts: `docs/fact-sheet.md` and `oracle.py`; brief: `docs/brief.md`;
R1 reviews and their dispositions: `docs/reviews/capture_recapture/`.

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a prediction
question; the beat ends with the question and the picture holds 2 s before the
next beat answers it.

## 2 The arc

| Act | Chapters | What the viewer gets |
|---|---|---|
| I. The trick | 01 cold open + hook, 02 the marks measure the net, 03 why the marks spread evenly | the table, the answer, and why it is legitimate |
| II. Making it exact | 04 netfuls, 05 turning the question around, 06 the table balances | counting, likelihood, and Feller's ratio, which *is* the table's balance |
| III. Honesty | 07 check it by cheating, 08 how sure, 09 one rule for every failure | the estimate's scatter first (lake known), then the inversion (lake unknown), then failure modes |
| IV. Everywhere | 10 the dark cell everywhere | Laplace, code review, Kosovo; the coda |

The 07/08 order is deliberate: the viewer first *watches* estimates scatter
around a lake whose size we know, then turns the question around (the same move
as 05) to ask which sizes could have produced our catch.

**The hero picture is a mosaic** (R1 creative A-1): the four-cell table drawn
with areas in proportion to the counts. Columns: first catch (marked, gold,
width 1,000) and not marked (width N − 1,000); rows: in the second catch
(green, top) and not. The two catches are independent exactly when the divider
runs straight across both columns (k/M = (n − k)/(N − M) ⇔ kN = Mn), so "the
table balances" is a line going straight. The dark cell, the fish nobody
caught, is most of the area (8,100 of 10,000), and the picture says so.

**One drawn fish stands for ten** (spike S-2 and engine review E-A1): the lake
shows 1,000 fish, 100 of them gold (the first catch); the second catch is 100
fish in green rings, 10 of them gold (caught twice). A key on screen says "each
fish drawn = 10 fish". The swimming is a time-lapse of a 10,000-fish simulation
(plain Rust state); the maps are computed from all 10,000 (E-A2, E-A3).

## 3 Color roles

| Idea | Color | Where |
|---|---|---|
| marked (the first catch), "caught twice" | gold `#FFD866` (hero) | fish glow, mosaic column, t2c |
| unmarked fish | blue `#58C4DD` | fish |
| the second catch | green `#83C167` | net ring, mosaic row |
| the dark cell, the lake size, the estimate | teal `#5CD0B3` on near-black | mosaic, slider, needle |
| likelihood ("score") | purple `#9A72AC` | curves, landscape slice |
| probability over k | blue-grey `#8A93A3` bars | histograms |
| bias / failure | red `#FC6255` (never the marks) | gauges, warnings |

## Vocabulary (one word per idea)

**marked** (the first catch; "Feller marked his fish with red spots; ours glow"),
**first catch**, **second catch**, **caught twice** (never "overlap" or
"recaptured" aloud), **first catch only**, **second catch only**, **the dark
cell** (no net touched them), **lake size**, **the soup guess** (marked times
second catch over caught twice), **score** (likelihood, after it is named once),
**experiment** (one run of mark-and-recatch; never "study"). Never say "n" and
"N" in one sentence; letters live on screen under word labels.

## 01_hook: The fish nobody caught (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | A thousand fish are caught, marked, and let go. One week later, a second net catches a thousand. Of those, a hundred are marked. | Top-down lake with fixed landmarks (reeds in the north-west, a dark hole with contour lines in the south-east, a small dock on the south shore): 1,000 drawn fish; the first catch glows gold; the second catch gets green rings; the caught-twice fish are gold bodies in green rings. Key: "each fish drawn = 10 fish" | particle swarm, glow |  |
| 01_2 | Now look at the fish no net ever touched. | Everything else fades to dim silhouettes; the fish fly by placement into a four-cell table labelled in words: columns "first catch" / "not caught first", rows "in the second catch" / "not". Cells 100 caught twice, 900 first catch only, 900 second catch only, and a dark "?" | morph swarm → table |  |
| 01_3 | There are about eight thousand one hundred of them. And we can say how sure we are. | "≈ 8,100" fills the dark cell; the table grows into the mosaic, areas in proportion, the dark cell ballooning to most of the frame, full of dim silhouettes | live number, morph |  |
| 01_4 | *(title)* | **How to Count Fish You've Never Seen** | title card | listen 3 s (the motif, unresolved) |
| 01_5 | Why not count one patch and scale up? This net covers one hundredth of the lake. Four hundred fifty-one fish, times a hundred, is forty-five thousand one hundred. *(the deep-end spot of the seeded lake, A-F27)* | A net drops on the deep end; a 10×10 grid with one square lit; the arithmetic | net sweep, live count |  |
| 01_6 | ⟨predict⟩ Try two other spots. Same answer? | The net hovers over the weeds and over open water | prediction card |  |
| 01_7 | Not even close. In the weeds, eighty-eight thousand. In open water, two thousand eight hundred. | Three estimates on a plain number line (2,800 / 45,100 / 88,000, 31× apart) | number line |  |
| 01_8 | The net knows it covered a hundredth of the water. It can't know what share of the fish that was. | A density heat map fades in over the landmarks; a box "how many fish in all? (coming up)" | heat map, `indicate` |  |

- **Personas.** Maya: the table is labelled in words (M2) and the dips are
  spoken (M4). Dev: sampling bias in one picture. Ruth: the net knows its area,
  not its share of the fish (R-12); quadrat sampling with many random quadrats
  can work, one net cannot measure its own coverage, which marks provide.
- **Facts:** the seeded clustered lake (A-F27) gives the dips; spots are fixed
  and sensible, not seed-shopped. The cold open (01_1–01_3) is about 20 s
  (plan rule 20; creative A-8).

## 02_the_marks_measure_the_net (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | In nineteen fifty, the probabilist William Feller put this puzzle in his famous textbook. Biologists had been doing it for decades. His book never mentions them. | Quote card, verified first-edition text: "Suppose that 1000 fish caught in a lake are marked by red spots…" (Feller 1950, p. 37) | Text card |  |
| 02_2 | Feller marked his fish with red spots. Ours glow, so you can see them. | A red spot on one fish morphs into the gold glow | morph |  |
| 02_3 | Here's the first way to read it. Of the thousand marked fish, the second net caught a hundred. One in ten. | Gold bar: "1,000 marked", a hundred of them lit "caught twice" | proportion bar prop |  |
| 02_4 | ⟨predict⟩ So the second net probably caught about one in ten of all the fish. It caught a thousand. A thousand is one tenth of how many? | Green tray "1,000 in the second catch"; card "1,000 = one tenth of ?" | prediction card |  |
| 02_5 | Ten thousand. The marked fish measured our net for us. It searched about a tenth of the fish. | The green catch is copied ten times; the copies stack into a 10,000-fish bar, the last 8,100 dimming to silhouettes; the dark cell = 10,000 − 1,900 = 8,100 | lagged reveal, table callback |  |
| 02_6 | Read the same fact the other way. One fish in ten in the second catch is marked, so about one in ten in the lake is. | Two pictures side by side: the gold bar (100 of 1,000 marked caught again) and the green tray (100 of 1,000 caught are marked) | proportion bar prop |  |
| 02_7 | Marked times second catch, over caught twice. A thousand times a thousand, over a hundred. Ten thousand. Call it the soup guess. | Numbers 1,000 × 1,000 / 100 = 10,000; then the letters under word labels: N̂ = M·n/k | TransformMatchingTex |  |
| 02_8 | Why should that work? How sure can we be? And when does it fail? | Three question cards; each returns as an act title (03_1, 07_1, 09_1) | statement cards | hold |

- **Personas.** Maya: every thousand carries its colour and word (M5); the rule
  is said in words and numbers before 05 uses it (M6); "soup guess" is named
  here and used everywhere after (m7). Dev: the two readings are the same
  cross-multiplication; "it searched about a tenth of the fish" ties 02_5 back
  to 01_7 (D25). Ruth: 10,000 is a best guess, and the careful argument is
  promised, not asserted.
- **Exact version (lower-third at 02_6):** "precisely: E[K] = nM/N for every
  catch size, by linearity". The third-edition footnote 13 ("used in the first
  edition without knowledge…") stays off screen until checked in a library copy
  (R-05, D14; fact sheet marks it UNVERIFIED).

## 03_why_the_marks_spread (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Think about tasting soup. Stir it, and one spoonful tells you how salty the whole pot is. | Act card "Why should that work?" (2 s); then the lake as the pot: a ladle-sized net dips once | act card, statement card |  |
| 03_2 | We built this lake, so we can peek. The fisher can't. We marked the first thousand near the dock, and let them go there. | Label "only we can see this"; god's-eye view; the 100 gold fish cluster at the dock | label |  |
| 03_3 | ⟨predict⟩ Watch them swim. Will the marks stay bunched up near the dock? | The lake, tinted in 3×3 regions by the share of fish that are marked: hot gold at the dock, dark elsewhere | prediction card |  |
| 03_4 | The fish stay crowded where they like to be. But the marks spread out evenly among them. | One lake panel, time-lapse "≈ two weeks in 12 s": the fish stay clumped in the weeds and the deep end; the regions' gold tint evens out from 66% at the dock to about 10% everywhere (readout in the caption band); faint contour streamlines (short opaque pieces) carry the circling | **habitat-potential dynamics from 10,000 simulated fish; tinted regions recoloured in place** |  |
| 03_5 | Wherever the second net goes, about one fish in ten it catches is marked. The catch is a tiny copy of the lake's mix. | Nets of 10 and of 20 fish come up in different places; each tray's marked share sits near 10% | repeated small catches | listen 5 s (one vibraphone note among every ten kalimba notes, the same beat for both nets) |
| 03_6 | Zoom in. A crowded patch has fifty fish, five of them marked. A sparse one has ten, with one marked. Both are one in ten. | Two 1:1 insets from the simulation's own fish, labelled "every fish" | zoom insets |  |
| 03_7 | That's what makes the table fair. Mixing makes being marked unrelated to where a fish swims. So the second net catches marked fish in their fair share. | The mosaic returns; its divider runs straight across both columns | callback |  |
| 03_8 | That's what independent means here. The second net doesn't care which fish were caught before. | Label "marks mixed fairly = independent" under the straight divider | label | hold |

- **Precisely (lower-third at 03_8):** "after mixing, being marked is
  independent of position; equal catchability is a separate assumption (chapter
  9)" (R-04).
- **Ruth:** uniform density is not required. Dynamics: overdamped drift −∇U plus
  a circulation along U's contours plus noise; the left of the picture is
  labelled "simulated fish density", never "e^(−U)" (fact sheet option (c);
  R-26, D24). Visible fish follow smoothed paths ("time-lapse"; E-A3).
- **Sound:** 03_4 widens a vibraphone shimmer from centre-south to full width as
  the marks spread; the 03_5 window lets the proportion be heard (creative A-4).
- **Facts to pin (new oracle keys):** the 03_6 patch counts from a real
  simulation frame, or the insets are tagged "illustration".

## 04_netfuls: Every possible netful (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | To see exactly why, shrink the lake. Twenty fish, five of them marked. Our net holds four. | Toy lake: 20 fish, 5 gold, morphing out of the big lake | morph from big lake |  |
| 04_2 | Random means every group of four is equally likely. How many groups are there? | Netfuls flash | lagged reveal |  |
| 04_3 | Twenty choices for the first fish, nineteen for the second, then eighteen, then seventeen. But the same four fish come out in twenty-four orders, four times three times two times one, and that's still one netful. So divide by twenty-four. Four thousand eight hundred forty-five. | Four fish shuffling through their 24 orders; `20·19·18·17 = 116,280`, `/ 24 = 4,845`; then "twenty choose four" | count-up |  |
| 04_4 | ⟨predict⟩ Sort them by how many marked fish they hold. Which pile is biggest? | Five empty bins labelled 0 to 4 marked | prediction card |  |
| 04_5 | *(no narration: the pour)* | All 4,845 netful glyphs pour into the five bins (spike S-4); live counts climb to 1,365 / 2,275 / 1,050 / 150 / 5 | **4,845 glyphs** | listen 4 s (a soft granular rain; each bin has its own note) |
| 04_6 | One marked fish. One in four, same as the lake. The soup again. | The 1 bin glows | histogram prop |  |
| 04_7 | Take the pile with two marked fish. Ten ways to pick two of the five marked ones. For each, a hundred and five ways to pick two of the fifteen unmarked ones, like shirts and pants. One thousand fifty. | The ten marked pairs; 20 − 5 = 15 unmarked; 15·14/2 = 105; `C(5,2)·C(15,2) = 10·105 = 1,050` | TransformMatchingTex |  |
| 04_8 | Divide every pile by all four thousand eight hundred forty-five, and you get the chances. About twenty-eight percent, forty-seven, twenty-two, three, and under one. | Bars as whole percents 28 / 47 / 22 / 3 / <1 | histogram prop |  |
| 04_9 | Mathematicians call it the hypergeometric distribution. If we know the lake, we know the chances of every catch. But we only know the catch. | The rule in words first: chance = (ways to pick the marked ones) × (ways to pick the unmarked ones) ÷ (all netfuls), with the letters as small labels; arrow "lake → catch" with a question mark | statement card | hold |

- **Precisely (lower-thirds):** "P_N(K = k) = C(M,k)·C(N−M, n−k) / C(N,n)"
  (built one substitution per TransformMatchingTex, E-X4); "exact: 91/323,
  455/969, 70/323, 10/323, 1/969" (R-21, D21, m10); "Σₖ C(5,k)C(15,4−k) =
  C(20,4) (Vandermonde)".
- **Glyph:** `\binom` on screen; "C(M, k)" read aloud "M choose k". The
  hypergeometric name is spoken once (m22).

## 05_turning_it_around (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | A new lake, and we don't know its size. We marked ten, caught ten, and three were marked. | Fogged lake; the second net comes up | data card | listen 3 s (three vibraphone rings among seven kalimba plinks: k = 3 heard) |
| 05_2 | ⟨predict⟩ The soup guess first. Ten times ten, over three? | "≈ ?" | prediction card |  |
| 05_3 | About thirty-three. Now check it properly. What's the smallest the lake could be? We've already seen seventeen different fish. | "≈ 33"; 17 marked on an axis | axis |  |
| 05_4 | For each possible lake size, ask how likely it is to see exactly three marked. A lake of twenty, about eight percent. Thirty-three, about thirty-two percent. Sixty, about sixteen. | Three still frames: the histogram for N = 20, 33, 60, the k = 3 bar lit and its height printed | stills first |  |
| 05_5 | Now sweep through every size. | N tracker sweeps 17 → 60; the histogram morphs; the k = 3 bar drops a dot onto a curve | **tracker-driven histogram + traced curve** | listen 4 s (a soft tone whose filter is brightest at the top) |
| 05_6 | The curve peaks at thirty-three, just as the soup guess said. It's called the likelihood. Think of it as a score for each lake size. How likely our catch would be, if the lake were that size. | Peak 33, L ≈ 0.318; name card "score of a lake size = how likely our catch would be if the lake were that size" | statement card |  |
| 05_7 | Add up every height, out to lakes as big as you like, and you get about sixteen and two thirds, not one. The lake has one size. These aren't chances that it's a certain size. They're scores for comparing guesses. | A running sum while the axis compresses out to N = 2,000: "17 to 60: 9.9 … all sizes: 16.67"; Feller's 1950 card: "It is, therefore, meaningless to ask for the probability that n [Feller's n: our lake size] is greater than, say, 6000." | live sum | hold |

- **Precisely (lower-third at 05_7):** "Σ_N L(N) = Mn/(k(k−1)) = 50/3 when
  k ≥ 2; infinite when k ≤ 1" (proved, A-F7; R-06, D13).
- **Ruth:** every fixed-k slice rises left of the crest and falls right of it
  (Feller's ratio); fixed-N slices peak near it, at ⌊(n+1)(M+1)/(N+2)⌋, tied
  with one less when that quotient is whole (R-27).
- **Maya:** "score" is the spoken word from here on; the two slices are named
  aloud (M9); "about sixteen and two thirds", never "about seventeen", which
  is also the smallest lake (m2).
- **Facts to pin (new oracle keys):** L at N = 20, 33, 60 for (10, 10, 3):
  7.8% / 31.8% / 15.9%; the partial sum 17–60 = 9.89.

## 06_the_landscape: Same mountains, two questions (~40 s; the whole chapter on the camera route)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Here's every lake size and every possible count at once. | 05's histograms slide back into rows and rise into the lit bar landscape P_N(K = k), 44 sizes × 11 counts (S-5's field); a still first, with axes labelled in words ("lake size", "marked fish caught") fixed in frame | **lit 3D bars + `CameraRig`** |  |
| 06_2 | ⟨predict⟩ Cut across at one lake size. What do you get? | The camera swings to face a fixed-size slice (a translucent plane, drawn last) | slicing plane, orbit < π |  |
| 06_3 | The chances from the netfuls. They add to one. | The slice's bars light; "Σ = 1" | slice |  |
| 06_4 | Cut along at our three, and you get the score curve. Its heights add to sixteen and two thirds. Same mountains, two questions. | The camera swings 90° to face the k = 3 slice side-on and ends exactly on 05_6's 2D curve; the cut back to the 2D route matches frame for frame; the crest k = Mn/N is drawn last as a ridge over the bar tops | camera move, matched cut | hold |

- **Engine:** the whole chapter is about 40 s on the camera route (≈ 6 min of
  rendering at S-5's 6.1 fps), so it needs no mid-chapter route join; it opens
  on 05_7's last frame (the histograms sliding into rows) and ends exactly on
  05's 2D score curve; labels are fixed in frame (needs a kit HUD helper, E-X3);
  the crest is a depth-tested ridge over the bar tops (E-A5).

## 07_feller_s_comparison: Is the score still rising? (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Feller found the top score without drawing anything. Compare each lake size with the one just below it. | Two neighbouring bars from 05's score curve with a ">" between them | statement card |  |
| 07_2 | In our small lake, the step from thirty-two to thirty-three comes down to two numbers, five twenty-nine against five twenty-eight. The top one wins, so the score goes up. From thirty-three to thirty-four, it's five seventy-six against five seventy-eight, so it goes down. | The two fractions with live numbers; the larger side lights | live arithmetic |  |
| 07_3 | ⟨predict⟩ So take the top minus the bottom, written with letters. Multiply it out. What survives? | `(N−M)(N−n) − N(N−M−n+k)` on one line | prediction card |  |
| 07_4 | Marked times second catch, minus caught twice times lake size. While that's positive, the score is still rising. | Expanded on one line; the terms that cancel strike through in pairs (drawn strike lines): N², N·n, N·M; `Mn − kN` survives | **term-by-term cancellation** |  |
| 07_5 | Now write it with the table's four cells. It becomes first catch only times second catch only, minus caught twice times the dark cell. | `Mn − kN = ab − k·u`; each letter flies to its mosaic cell (`parts_by_tex` + replacement transforms, E-A7) | TransformMatchingTex onto the table |  |
| 07_6 | In the small lake, seven times seven, minus three times sixteen. One, same as before. Back to Feller's lake. | 49 − 48 = 1 = 100 − 99; the toy table swaps for Feller's | live arithmetic |  |

## 08_the_table_balances (~55 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | ⟨predict⟩ So start with a small dark cell and let it grow. Where does the climb stop? | The mosaic with a sliver of a dark cell; the two second-catch dividers, left (marked column) and right (dark column) | prediction card |  |
| 08_2 | In each column, the green part is what the second net caught. While the unmarked side's green share is bigger, the score keeps climbing. | The mosaic sweeps N up from 1,900: the dark cell rises out of nothing and the right divider falls toward the left one | **mosaic sweep** |  |
| 08_3 | A hundred times eight thousand one hundred is eight hundred ten thousand. So is nine hundred times nine hundred. The shares match, the line is straight, and the table balances. | The dark-cell counter ticks to 8,100 as the line goes straight | **counter** | listen 5 s (the motif resolves to 1̂ over the opening pad) |
| 08_4 | A straight line is exactly what two independent catches look like. So the careful argument and the soup guess give the same answer, rounded down to a whole fish. | Cards merge; the straight divider glows | statement card |  |
| 08_5 | At Feller's numbers the line is perfectly straight at ten thousand, so lakes of nine thousand nine hundred ninety-nine and ten thousand tie for the top score. Feller just says ten thousand. | The straight mosaic; the two neighbouring ratios in a lower-third | live fraction | hold |

- **Precisely (lower-thirds):** 07_1 "L(N)/L(N−1) = (N−M)(N−n) / (N(N−M−n+k)),
  for N > M + n − k" (R-18); 08_4 "k·u = a·b: the table's odds ratio is 1, the
  2×2 picture of independence" (R-29); 08_5 "MLE = ⌊Mn/k⌋; a tie with Mn/k − 1
  when Mn/k is whole and k < min(M, n); none when k = 0; exact fractions tie,
  floating point picks 10,000 by rounding" (D26); neighbouring ratios
  80,982,001/80,981,901 and 81,018,001/81,018,101 (R-33).
- **Ruth:** Mn − kN = (M−k)(n−k) − k·u is exact algebra (+100, 0, −100 at
  9,999 / 10,000 / 10,001; toy 49 − 48 = 1 → u = 16 → N = 33). The mosaic's
  right divider is above the left exactly when Mn − kN > 0 (creative A-1,
  checked by algebra).
- **Music:** the leitmotif resolves here, not at 02's 10,000 (unearned there);
  the 08_3 window is the video's main result (§7.5: 4–8 s); the music thins
  0.5 s before 08_4, a key line (X-3).
- **Cut (R1):** both seesaws (creative A-2); the mosaic carries the balance.

## 09_check_it_by_cheating: Ten thousand lakes (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Let's check this by cheating. Build a lake of exactly ten thousand fish, and run Feller's experiment on it ten thousand times. | Act card "How sure can we be?" (2 s); a run counter "experiment 1 … 10,000"; a 100 × 100 grid of tiny round lakes, each lighting with its estimate (10,000 flat cells, E-A8; legend "each dot = one whole experiment") | **10,000 flat cells recoloured in place** | listen 5 s (twelve trials as separate plinks at the pitch of their caught-twice count, then a wash; the peak's note grows out of it) |
| 09_2 | ⟨predict⟩ Where do the counts of fish caught twice pile up? | Empty histogram axes, "fish caught twice" | prediction card |  |
| 09_3 | In a bell around a hundred. Most land within about nine of it. | Histogram of k (mean 100, SD 9.0) | histogram prop |  |
| 09_4 | But the estimate divides by that count, and dividing bends things. Ninety caught twice gives about eleven thousand one hundred. A hundred and ten gives about nine thousand one hundred. Ten too few pushes harder than ten too many. | The k-bell's bars travel along the hyperbola N = Mn/k (05's crest curve) and stretch into a right-skewed histogram | **bars along a curve** |  |
| 09_5 | Ninety-five percent of the estimates land from about eighty-five hundred to just over twelve thousand. Their average is about ten thousand and eighty, a little high. | Quantiles 8,475 / 10,000 / 12,048; mean ≈ 10,080 | live stats |  |
| 09_6 | *(for programmers)* Two lines of code. Always check a simulation against something you can compute exactly. | Code panel (CM Typewriter): greyed setup `import numpy as np` / `rng = np.random.default_rng(20261009)` / `M = n = 1_000; N = 10_000`; then `k = rng.hypergeometric(M, N - M, n, size=10_000)` / `est = M * n / k`; then the exact line `p = [comb(M, k) * comb(N - M, n - k) / comb(N, n) for k in range(n + 1)]` | code panel slides in |  |

- **Facts:** the seeded simulation (seed `20261009`) reproduces the fact sheet
  digit for digit (Dev ran the panel: mean 10,080.002; 8,474.58 / 10,000 /
  12,048.19); exact conditional mean 10,082.5.

## 10_small_experiments: Dividing by zero (~45 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | ⟨predict⟩ With a smaller experiment it gets worse. In a lake of a thousand, mark fifty and catch fifty. How often does the second catch hold no marked fish at all? | A small lake; the second net comes up | prediction card |  |
| 10_2 | About one time in fourteen. Then the soup guess divides by zero. | P(k = 0) = 7.2%; an estimate flies off the edge | red |  |
| 10_3 | Douglas Chapman's fix is to add one to each count, and subtract one at the end, as if one imaginary fish had been caught both times. The ones aren't a fudge. They're what makes the average come out right. It never divides by zero, and once you expect four or more caught twice, it's nearly exact on average. | A ghost fish (dashed gold outline) drops into the caught-twice cell as M and n tick up by one; `(M+1)(n+1)/(k+1) − 1`; two needles on one gauge around 1,000: the plain soup guess at +24% (experiments that caught any) in red, Chapman at −6.5% | TransformMatchingTex | hold |

- **Precisely (lower-third at 10_3):** "E[N̂_C] = N − (N+1)·C(N−M, n+1)/C(N+1,
  n+1): exactly N when M + n ≥ N, and never low by more than N·e^(−Mn/N),
  under 2% once you expect 4 caught twice" (R-08, proved by Ruth: each factor
  (N−M−j)/(N+1−j) ≤ 1 − M/N; brute-forced on 583,220 triples). At k = 0
  Chapman still answers (2,600 here), a hint the experiment was too small (D12).
- **Dev (description):** the "+1" is two absorption identities:
  (M+1)/(k+1)·C(M,k) = C(M+1,k+1) and (N+1)/(n+1)·C(N,n) = C(N+1,n+1) (D5
  corrected the plan's wrong form). Planning card: "typical error (1 SD): ±30%
  at 10 caught twice, ±10% at 100; a 95% range is about twice that" (R-24, D4).
  In numpy `M*n/k` at k = 0 is `inf` with a RuntimeWarning (D9).

## 11_how_sure: Turning it around again (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | So far the lake was known, and we watched the estimates scatter. Now turn it around. We don't know the lake. Which sizes could plausibly have given us our hundred caught twice? | Score curve for Feller's numbers (relative to its top) | Axes + `c2p`, FunctionGraph |  |
| 11_2 | Only nineteen hundred fish? Then the second net scooped up all nine hundred unmarked fish and only a hundred of the thousand marked ones. The chance of that is about five in a one followed by four hundred thirty zeros. | Zeros scroll across the screen for 3 s; a dashed marker at the 308th zero, "a float64 gives up here" | scrolling digits |  |
| 11_3 | A million fish? Then we'd expect to catch about one marked fish, not a hundred. | Marker far right | number line |  |
| 11_4 | ⟨predict⟩ Even at the best size, catching exactly a hundred had only about a four percent chance. Is that a problem? | 4.43% at 10,000 | prediction card |  |
| 11_5 | Flip a hundred coins. Exactly fifty heads happens only about eight percent of the time, yet it's the most likely count. | Coin inset: P(50 of 100) = 7.96% | inset histogram |  |

## 12_the_ring_toss: A range that keeps its promise (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Here's the careful range. For each lake size, ask whether a catch of a hundred would land in that lake's middle ninety-five percent. Keep the sizes where it would. | 05's fixed-size slices, both 2.5% tails shaded, at the two edges | interval construction |  |
| 12_2 | Build that range for each experiment, and it rings the true size at least ninety-five times in a hundred, whatever the true size is. Like a ring toss. | A peg at 10,000; the first 100 seeded experiments' ranges drop as rings in a lagged cascade; 97 clack on, 3 miss in red; counter "all 10,000: 447 missed"; then the name **confidence interval** | ring toss | listen 3 s (wooden clacks) |
| 12_3 | For Feller's catch it runs from about eighty-four hundred to about twelve thousand one hundred. Close to the range we watched when the lake was known, but a different question. There the lake was fixed and the catch moved. Here the catch is fixed and the lake moves. | [8,400, 12,073] shaded; 07's range (8,475–12,048) stacked under it, both labelled | shading |  |
| 12_4 | ⟨predict⟩ What makes the range tight? | The range on its own, with three knobs: marked, second catch, caught twice | prediction card |  |
| 12_5 | Mostly, more fish caught twice. Four times as many halves the range. Catching a big share of the lake helps a little, and catching all of it leaves no doubt. | Slider on "expected caught twice" with the lake fixed at 10,000 and M = n = √(10,000·k); the range in percent: ±9% at 100 | tracker-driven interval | hold |

- **Precisely (lower-thirds):** 11_2 "below float64's smallest positive
  (5 × 10⁻³²⁴), so code works in logarithms" (D15); 12_2 "exact test inversion;
  coverage ≥ 95% at every lake size; 95.45% at this one" (R-22); 12_3 "Feller's
  own 8,500–12,000 matches one-sided levels 3.5% and 2.9%: confidence ≈ 93.6%"
  (R-25, description).
- **Optional (if time):** "Want a probability for the lake size? You need a
  starting belief. With a flat one, eight thousand five hundred to twelve
  thousand two hundred: almost the same numbers, a different question." (A-F26;
  R-30 asks to keep it; description at least, naming the prior: flat on
  N ≥ 1,900.)
- **Ruth:** the relative error ≈ √((1 − M/N)(1 − n/N)/k) is the symmetric
  delta method; the exact SD of Mn/K (given K > 0) is 924. Marking 2,000 and
  catching 500 is the same experiment as marking 500 and catching 2,000
  (Feller eq. 6.2): SD 894 both ways (one line, R-32).
- **Facts to pin (new oracle keys):** misses among the first 100 seeded
  experiments (3) and among all 10,000 (447); never pick a window to show a
  miss (D2, R-22).

## 13_one_rule_for_every_failure (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | Every failure breaks one rule. In the second net, marked and unmarked fish must be equally likely to be caught. Then the catch is a fair copy. | Act card "When does it fail?" (2 s); the true lake's mosaic with its divider straight; the estimate's needle on a gauge labelled "estimated lake size (truth: 10,000)" | gauge prop |  |
| 13_2 | ⟨predict⟩ Some fish are bold and easy to catch, both times. Will the estimate come out too big, or too small? | Bold fish glow; quiz card | prediction card |  |
| 13_3 | Too small. Bold fish get caught twice more than their share, so the dark cell shrinks. In our lake, where three fish in ten are three times as easy to catch, the estimate comes out around seventy-seven fifty. | The mosaic's divider bends; the estimate's mosaic straightens it by shrinking the dark cell; needle to ≈ 7,750; label "3 in 10 fish 3× as catchable, both times; median of 400 runs" | bent mosaic |  |
| 13_4 | Fish that learn to love the bait do the same. Fish that learn to avoid it push the estimate up. | Down to ≈ 7,140, up to ≈ 14,490; labels "marked fish 1.5× as catchable" / "⅔×" | gauge |  |
| 13_5 | Marks that wash off make marked fish look unmarked. That pushes it up too. | ≈ 12,500; label "20% of marks lost" | gauge |  |
| 13_6 | Two thousand newcomers between the catches dilute the marks. Then you've counted the lake as it is at the second catch. | ≈ 12,120 beside a second gauge mark "lake at the second catch: 12,000" | gauge |  |
| 13_7 | ⟨predict⟩ What if three fish in ten die between the catches? | The lake thins | prediction card |  |
| 13_8 | The estimate doesn't move. Death doesn't care about marks, so the rule still holds. | Needle stays at 10,000; the music drops out for 1.5 s | huh beat |  |
| 13_9 | And if the marks never mix, everything depends on where you fish. Fish near the release site, and you'd say sixty-two fifty. Fish the far side, and you'd say twenty-five thousand. | Chapter 03's lake panel, now *not* evening out (hot at the dock); label "two-halves model: all marks in one half; second catch 80% from one half"; ≈ 6,250 / ≈ 25,000; Feller's caveat on a card (silent): catches made in one spot within a short time | callback | hold |

- **Precisely (lower-third at 13_1):** "Mn/k estimates M ÷ (marked share of the
  second catch); it is biased exactly when that share differs from the marked
  share of the lake you mean" (R-01, the R1 blocker, fixed here).
- **Ruth:** each case is a seeded simulation (true lake 10,000, M = n = 1,000;
  A-F22); every gauge names its model (R-07, D8). The 13_9 numbers come from
  the two-halves model, drawn as such (R-10, D7). Schnabel's 1938 multi-census
  estimator in the *American Mathematical Monthly* (one line, optional).

## 14_the_dark_cell_everywhere: France and the bugs (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | Around eighteen hundred, the French mathematician Laplace used the same table to count France. Births were written down, so in our table, each newborn is a marked fish. | The mosaic stays at screen centre for the whole chapter; its fish glyphs morph into newborn glyphs; France's outline as a side inset (public-domain outline, E-A10) | template morph |  |
| 14_2 | At his request, officials counted everyone in chosen towns across France. About two million people, with about seventy-two thousand births a year. That's one birth for every twenty-eight and a bit people. Taking France's births as a million a year, that's about twenty-eight point four million people. | The mosaic relabelled: births = marked, people counted = second catch, births among them = caught twice | template morph |  |
| 14_3 | *(silent: the odds card)* | Laplace, *Essai philosophique* (tr. Truscott & Emory 1902, p. 67): "a bet of almost 300000 against 1" that he was within half a million; "his calculation; it treats the chosen towns as typical" | odds card | hold |
| 14_4 | *(for programmers)* ⟨predict⟩ It counts bugs too. Say two reviewers read the same code without comparing notes. One finds thirty bugs, the other twenty-four, and twelve are the same. How many are still hiding? | The glyphs morph into bug glyphs; two lists and a Venn (boolean ops) as an inset; lower-third: Mills, 1970: "'lake' is synonymous with 'system'…" (p. 73) | prediction card |  |
| 14_5 | Thirty times twenty-four, over twelve, is sixty. They found forty-two, so about eighteen. But twelve is a small number caught twice. An honest range is forty-seven to ninety-four bugs in all. | Range bar 47–94 on the mosaic's estimate | range bar |  |
| 14_6 | Reviewers of different skill are fine. Bugs of different difficulty are not. Easy bugs swell the caught-twice cell and pull the estimate down, like the bold fish. So read sixty as a floor. | Callback to 13_3's bent mosaic | callback |  |

## 15_counting_the_dead: Kosovo, and the coda (~40 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | And it has counted the dead. In the war in Kosovo, in nineteen ninety-nine, several lists named about four thousand four hundred different victims between them. The names shared between lists estimated about ten thousand four hundred killed. Years later, a name-by-name count found nine thousand seven hundred ninety, inside the estimate's range. | Sober typography only; the glyphs become small grey name marks; a four-list table with sixteen cells, one dark; the range 9,002–12,122; the music fades to silence over 1.2 s in the gap before this beat, never mid-sentence | quiet card, **silence** under the numbers | hold |
| 15_2 | ⟨predict⟩ France, the bugs, Kosovo. What do they share? | The three tables side by side, each with one dark cell | prediction card |  |
| 15_3 | Each is a table with one cell nobody can see, filled in by the cells we can. | The mosaic sinks into the lake surface and the swarm re-forms (01_2 in reverse); receipts card "Every number in this video was computed by code or copied from a cited source: link" | coda; the motif alone on felt piano, the pad only under the last shot | listen 3 s |

- **Ruth:** Laplace's sample: thirty départements spread evenly over France,
  and within them the communes with the most reliable records, counted by the
  government at his request (2,037,615 people; 215,599 births in 3 years =
  71,866⅓ a year; ratio 28.352845; 28,352,845 for 1,000,000 births, which
  Laplace *assumed*); not a census (R-03, R-16). The odds assume the chosen
  communes behave like a random sample (R-14; Cochran 1978 and Pearson 1928
  analyse them: check before citing). Kosovo used multiple-systems estimation
  with log-linear models on four lists, not a two-list formula; the period is
  March–June 1999 (Ball et al. 2002; Spagat 2014; R-02, R-15). Mills (IBM
  report, 1972; his collected papers date the article 1970) seeded bugs
  deliberately: painted fish (R-31).
- **Dev:** two reviewers of different skill are the M_t model, which the
  formula survives; bugs that differ in difficulty for everyone (M_h) bias it
  low (D11; Briand et al. 2000, IEEE TSE 26(6), "underestimation may be
  substantial"). The 30 / 24 / 12 example is hypothetical (D10: "say").
  Exact 95% range [47, 94] by test inversion; Chapman 58.6 ± 8.4.
- **Optional (cut first if long):** the German tank problem as a contrast
  needs a ledger row citing Ruggles & Brodie (1947) before use (R-28).
