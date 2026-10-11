# Storyboard — How to Count Fish You've Never Seen

The hand-edited source of this video's words and pictures (from R1, 2026-10-11,
revised at R2 the same day; it started as plan §9.4, revision 2). `python3 -I
tools/storyboard.py capture_recapture` checks it (gate G2) and writes
`script.tsv` from the words column. Facts: `docs/fact-sheet.md` and
`oracle.py`; brief: `docs/brief.md`; the draft video description, where every
note marked "description" lands: `docs/description.md`; R1 and R2 reviews and
their dispositions: `docs/reviews/capture_recapture/`.

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a prediction
question; the beat ends with the question and the picture holds 2 s before the
next beat answers it.

## 2 The arc

| Act | Chapters | What the viewer gets |
|---|---|---|
| I. The trick | 01 cold open + hook, 02 the marks measure the net, 03 why the marks spread evenly | the table, the answer, and why it is legitimate |
| II. Making it exact | 04 netfuls, 05 turning the question around, 06 one landscape, two cuts, 07 Feller's comparison, 08 the table balances | counting, likelihood, and Feller's ratio, which *is* the table's balance |
| III. Honesty | 09 check it by cheating, 10 small experiments, 11 how sure, 12 the ring toss, 13 one rule for every failure | the estimate's scatter first (lake known), then the inversion (lake unknown), then failure modes |
| IV. Everywhere | 14 France and the bugs, 15 Kosovo and the coda | Laplace, code review, Kosovo; the coda |

The 09/11 order is deliberate: the viewer first *watches* estimates scatter
around a lake whose size we know, then turns the question around (the same move
as 05) to ask which sizes could have produced our catch.

**The hero picture is a mosaic** (R1 creative A-1): the four-cell table drawn
with areas in proportion to the counts. Columns: first catch (marked, gold,
width 1,000) and not marked (width N − 1,000); rows: in the second catch
(green, top) and not. The two catches are independent exactly when the divider
runs straight across both columns (k/M = (n − k)/(N − M) ⇔ kN = Mn), so "the
table balances" is a line going straight. With green on top, the unmarked
column's divider sits *below* the marked column's while its green share is the
bigger one (Mn − kN > 0), and rises toward it as N grows (R2 Ruth R18). The dark cell, the fish nobody
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
**experiment** (one run of mark-and-recatch; never "study"), **the mystery lake**
(chapter 05's lake of unknown size, 10 / 10 / 3; R2), **the careful range** (the
exact 95% range; never "honest range"; R2), **marking day** (when the first catch
is marked and let go; the lake Mn/k aims at; R2). Never say "n" and "N" in one
sentence; letters live on screen under word labels.

## 01_hook: The fish nobody caught (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | A thousand fish are caught, marked, and let go. A while later, a second net catches a thousand. Of those, a hundred are marked. | Top-down lake with fixed landmarks (reeds in the north-west, a dark hole with contour lines in the south-east, a small dock on the south shore): 1,000 drawn fish; the first catch glows gold; the second catch gets green rings; the caught-twice fish are gold bodies in green rings. Key: "each fish drawn = 10 fish" | particle swarm, glow |  |
| 01_2 | Now look at all the fish no net ever touched. | Everything else fades to dim silhouettes; the fish fly by placement into a four-cell table labelled in words: columns "marked (first catch)" / "unmarked", rows "in the second catch" / "not" (R2 Maya M10). Cells 100 caught twice, 900 first catch only, 900 second catch only, and a dark "?" | morph swarm → table |  |
| 01_3 | Our best guess for this dark cell is eight thousand one hundred. And we can say how sure we are. | "≈ 8,100" fills the dark cell; the table grows into the mosaic, areas in proportion, the dark cell ballooning to most of the frame, full of dim silhouettes | live number, morph |  |
| 01_4 | *(title)* | **How to Count Fish You've Never Seen** | title card | listen 3 s (the motif, unresolved) |
| 01_5 | Why not count one patch and scale up? This net covers one hundredth of the lake. Four hundred fifty-one fish, times a hundred, is forty-five thousand one hundred. *(the deep-end spot of the seeded lake, A-F27)* | A net drops on the deep end; a 10×10 grid with one square lit; the arithmetic. The counter counts the square's fish in the 10,000-fish simulated lake (451), not the drawn glyphs (one per ten, so about a tenth as many); the key "each fish drawn = 10 fish" stays on screen (R2 Ruth R19) | net sweep, live count |  |
| 01_6 | ⟨predict⟩ Try two other spots. Same answer? | The net hovers over the weeds and over open water | prediction card |  |
| 01_7 | Not even close. In the weeds, eighty-eight thousand. In open water, only two thousand eight hundred. | Three estimates on a plain number line (2,800 / 45,100 / 88,000, 31× apart) | number line |  |
| 01_8 | The net knows it covered a hundredth of the water. It can't know what share of the fish that was. | A density heat map fades in over the landmarks; a box "how many fish in all? (coming up)"; lower-third (R2 Ruth R11, Dev dev-09): "precisely: averaging many random dips does work (all 100 squares here average exactly 10,000), but only 22 of the 100 single dips land within 2× of it. One net can't tell what share of the fish it reached." | heat map, `indicate` |  |

- **Personas.** Maya: the table is labelled in words (M2) and the dips are
  spoken (M4). Dev: sampling bias in one picture. Ruth: the net knows its area,
  not its share of the fish (R-12; 01_8 voice); quadrat sampling with many
  random quadrats can work, one net cannot measure its own coverage, which marks
  provide (R2: the 01_8 lower-third and the description, 01_8).
- **Facts:** the seeded clustered lake (A-F27) gives the dips; spots are fixed
  and sensible, not seed-shopped. The 01_8 lower-third's numbers are
  `A-F27.all_cells.mean_estimate` and `.share_of_dips_within_2x_of_truth`. The
  cold open (01_1–01_3) is about 20 s (plan rule 20; creative A-8).

## 02_the_marks_measure_the_net (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | In nineteen fifty, the probabilist William Feller put this puzzle in his famous textbook. Others had been using it for years, and his first edition never mentions them. | Quote card, verified first-edition text: "Suppose that 1000 fish caught in a lake are marked by red spots…" (Feller 1950, p. 37) | Text card |  |
| 02_2 | Feller marked his fish with red spots. Ours glow, so you can see them. | A red spot on one fish morphs into the gold glow | morph |  |
| 02_3 | Read the catch one way first. Of the thousand marked fish, the second net caught a hundred. One in ten. | Gold bar: "1,000 marked", a hundred of them lit "caught twice" | proportion bar prop |  |
| 02_4 | ⟨predict⟩ So guess that the second net caught about one in ten of all the fish. It caught a thousand. A thousand is one tenth of how many? | Green tray "1,000 in the second catch"; card "1,000 = one tenth of ?" | prediction card |  |
| 02_5 | Ten thousand. The marked fish have measured our net for us. It searched about a tenth of the fish. | The green catch is copied ten times; the copies stack into a 10,000-fish bar, the last 8,100 dimming to silhouettes; the dark cell = 10,000 − 1,900 = 8,100 | lagged reveal, table callback |  |
| 02_6 | Read the same fact the other way. One fish in ten in the second catch is marked, so about one in ten in the lake is. We marked a thousand, so the lake holds about ten times that. | Two pictures side by side: the gold bar (100 of 1,000 marked caught again) and the green tray (100 of 1,000 caught are marked); then the 1,000 marked fish copied ten times, as 02_5 copied the catch | proportion bar prop |  |
| 02_7 | Marked times second catch, over caught twice. A thousand times a thousand, over a hundred. Ten thousand. Call it the soup guess. | Numbers 1,000 × 1,000 / 100 = 10,000; then the letters under word labels: N̂ = M·n/k | TransformMatchingTex |  |
| 02_8 | Why should that work? How sure can we be? And when does it fail? | Three question cards; each returns as an act title (03_1, 09_1, 13_1) | statement cards | hold |

- **Personas.** Maya: every thousand carries its colour and word (M5); the rule
  is said in words and numbers before 05 uses it (M6); "soup guess" is named
  here and used everywhere after (m7). Dev: the two readings are the same
  cross-multiplication; "it searched about a tenth of the fish" ties 02_5 back
  to 01_7 (D25). Ruth: 10,000 is a best guess (02_4 "guess", R2 R7), and the
  careful argument is promised, not asserted.
- **Exact version (lower-third at 02_6, and the description, 02_6):**
  "precisely: E[K] = nM/N for every catch size, by linearity". The third-edition
  footnote 13 ("used in the first edition without knowledge…") stays off screen
  and out of the description until checked in a library copy (R-05, D14; fact
  sheet marks it UNVERIFIED); so 02_1 says "his first edition" (R2 R13, dev-07).

## 03_why_the_marks_spread (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Stir soup, and one spoonful tells you how salty the whole pot is. Our second catch is the spoonful. | Act card "Why should that work?" (2 s); then the lake as the pot: a ladle-sized net dips once | act card, statement card |  |
| 03_2 | We built this lake, so we can peek. We marked the first thousand near the dock, and let them go there. | Label "only we can see this" (it carries the old "The fisher can't", cut at R2 for length); god's-eye view; the 100 gold fish cluster at the dock | label |  |
| 03_3 | ⟨predict⟩ Watch them swim. Will the marks stay bunched up near the dock? | The lake, tinted in 3×3 regions by the share of fish that are marked: hot gold at the dock, dark elsewhere | prediction card |  |
| 03_4 | The fish stay crowded where they like to be. But the marks spread out evenly among them. | One lake panel, time-lapse: the fish stay clumped in the weeds and the deep end; the regions' gold tint evens out from 66% at the dock to about 10% everywhere (readout in the caption band); faint contour streamlines (short opaque pieces) carry the circling | **habitat-potential dynamics from 10,000 simulated fish; tinted regions recoloured in place** |  |
| 03_5 | Wherever the second net goes, about one fish in ten it catches is marked. The catch is a small copy of the lake's mix. | Real catches from the simulation's t = 400 frame (R2 Ruth R4, Dev dev-06): nets of a thousand fish, the story's second catch, come up at the weeds, open water and the deep end (the region centres NW, C, SE) holding 105, 94 and 103 marked; a strip under them lists all nine region centres, 90 to 105 marked (9.0% to 10.5%). Each net's fish are drawn one per ten, so a tray shows about a hundred fish and about ten gold; the label carries the true count. Caption "listen: about one bell for every ten notes" (R1 Maya m21) | repeated catches, counters | listen 5 s (each drawn fish of a tray is a kalimba note, each marked one a vibraphone note, in catch order) |
| 03_6 | The fullest corner has twenty-eight hundred eleven fish, two hundred eighty-two marked, and the emptiest has two hundred four, twenty-five marked. Both sit near one in every ten, and the small one wobbles more. | Two insets of whole 3×3 regions at t = 400, every fish drawn 1:1 and labelled "every fish in this corner": the most crowded region (SE, the deep end: 2,811 fish, 282 marked, 10.0%) and the least crowded (NE: 204 fish, 25 marked, 12.3%); picked by fish count alone, never by marks | zoom insets, counters |  |
| 03_7 | Mixing is what makes the table fair. Being marked becomes unrelated to where a fish swims. In both columns, the second net takes the same one in ten, so the line across the table runs straight. | The mosaic returns, columns labelled "marked (first catch)" and "unmarked"; each column's green share counts up (100 of 1,000; 900 of 9,000) and the divider runs straight across both (R2 Maya M4) | callback, counters |  |
| 03_8 | That's what independent means here. The second net doesn't care which fish were caught before, as long as every fish is as easy to catch. | Label "marks mixed fairly = independent" under the straight divider; the lower-third's "(chapter 13)" points ahead to equal catchability | label | hold |

- **Precisely (lower-third at 03_8, and the description, 03_8):** "after
  mixing, being marked is independent of position; equal catchability is a
  separate assumption (chapter 13)" (R-04). The 03_8 voice now names the
  assumption too (R2 R10).
- **Ruth:** uniform density is not required. Dynamics: overdamped drift −∇U plus
  a circulation along U's contours plus noise; the lake panel's density tint is
  labelled "simulated fish density", never "e^(−U)" (fact sheet option (c);
  R-26, D24; R2 R21: 03_4 is one panel). Visible fish follow smoothed paths
  ("time-lapse"; E-A3). The model and the 14% drift from e^(−U) go to the
  description (03_4).
- **Sound:** 03_4 widens a vibraphone shimmer from centre-south to full width as
  the marks spread; the 03_5 window lets the proportion be heard (creative A-4;
  score bead mx-e-a-8y5.13).
- **Facts (oracle keys `A-CH03.portable.*`; the scene runs the same mixing in
  `model.rs`, bit for bit):** every number in 03_4–03_6 is read from the
  portable run's frames (SplitMix64 noise, so the scene and the oracle draw the
  same fish; the numpy runs `A-CH03.habitat.*` stay as the research record) by a
  rule that never looks at the marks. 03_4: the dock region starts at 65.9%
  marked, every region is 8.6–12.3% at t = 400. 03_5: a net of 1,000 (the
  story's n) at each of the nine region centres catches the 1,000 nearest fish;
  they hold 90, 93, 103, 100, 94, 100, 105, 100, 99 marked (SW … NE). 03_6: the
  most crowded region (SE, 2,811 fish, 282 marked) and the least crowded (NE,
  204, 25); 12.3% is within the binomial wobble of a 204-fish region (SD 2.1
  points), which the voice says. The R1 insets (side-0.04 squares with 50/5 and 10/1, `A-CH03.inset.*`)
  were selected to match the voice and the R1 trays were drawn; both are
  retired (R2 Ruth R4, Dev dev-06), and their keys stay in the oracle as the
  record. Nets of 10 and 20 hold 0–3 and 0–5 marked (`A-CH03.trays.net10/20`):
  small catches wobble, which is chapter 09's lesson.

## 04_netfuls: Every possible netful (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | To see exactly why, shrink the lake. Twenty fish, five of them marked. And our net holds four. | Toy lake: 20 fish, 5 gold, morphing out of the big lake | morph from big lake |  |
| 04_2 | Random means every group of four is equally likely. How many groups are there? | Netfuls flash | lagged reveal |  |
| 04_3 | Twenty choices for the first fish, nineteen for the second, then eighteen, then seventeen. But the same four fish come out in twenty-four orders, four times three times two times one, and that's still one netful. So divide by twenty-four. Four thousand eight hundred forty-five. | Four fish shuffling through their 24 orders; `20·19·18·17 = 116,280`, `/ 24 = 4,845`; then "twenty choose four" | count-up |  |
| 04_4 | ⟨predict⟩ Sort them by how many marked fish they hold. Which pile is biggest? | Five empty bins labelled 0 to 4 marked | prediction card |  |
| 04_5 | *(no narration: the pour)* | All 4,845 netful glyphs pour into the five bins (spike S-4); live counts climb to 1,365 / 2,275 / 1,050 / 150 / 5 | **4,845 glyphs** | listen 4 s (a soft granular rain; each bin has its own note) |
| 04_6 | One marked fish. One in four, same as the lake. The soup again. | The 1 bin glows | histogram prop |  |
| 04_7 | Take the pile with two marked fish. Ten ways to pick two of the five marked ones. For each, a hundred and five ways to pick two of the fifteen unmarked ones, like shirts and pants. One thousand fifty. | The ten marked pairs; 20 − 5 = 15 unmarked; 15·14/2 = 105; `C(5,2)·C(15,2) = 10·105 = 1,050` | TransformMatchingTex |  |
| 04_8 | Divide every pile by all four thousand eight hundred forty-five, and you get the chances. About twenty-eight percent, forty-seven, twenty-two, three, and under one. | Bars as whole percents 28 / 47 / 22 / 3 / <1 | histogram prop |  |
| 04_9 | Mathematicians call it the hypergeometric distribution. If we know the lake, we know the chances of every catch. But we only know the catch. | The rule in words first: chance = (ways to pick the marked ones) × (ways to pick the unmarked ones) ÷ (all netfuls), with the letters as small labels; arrow "lake → catch" with a question mark | statement card | hold |

- **Precisely (lower-thirds; all three also in the description, 04_8–04_9):**
  "P_N(K = k) = C(M,k)·C(N−M, n−k) / C(N,n)"
  (built one substitution per TransformMatchingTex, E-X4); "exact: 91/323,
  455/969, 70/323, 10/323, 1/969" (R-21, D21, m10); "Σₖ C(5,k)C(15,4−k) =
  C(20,4) (Vandermonde)".
- **Glyph:** `\binom` on screen; "C(M, k)" read aloud "M choose k". The
  hypergeometric name is spoken once (m22).

## 05_turning_it_around (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | A new lake, and we don't know its size. Call it the mystery lake. We marked ten, caught ten, and three were marked. | Fogged lake; the second net comes up | data card | listen 3 s (three vibraphone rings among seven kalimba plinks: k = 3 heard) |
| 05_2 | ⟨predict⟩ Let's try the soup guess first. Ten times ten, over three? | "≈ ?" | prediction card |  |
| 05_3 | About thirty-three. Now check it properly. What's the smallest the lake could be? We've already seen seventeen different fish. | "≈ 33"; 17 marked on an axis | axis |  |
| 05_4 | For each possible lake size, ask how likely it is to see exactly three marked. A lake of twenty, about eight percent. Thirty-three, about thirty-two percent. Sixty, about sixteen. | Three still frames: the histogram for N = 20, 33, 60, the k = 3 bar lit and its height printed | stills first |  |
| 05_5 | Now sweep through every size. | N tracker sweeps 17 → 60; the histogram morphs; the k = 3 bar drops a dot onto a curve | **tracker-driven histogram + traced curve** | listen 4 s (a soft tone whose filter is brightest at the top) |
| 05_6 | The curve peaks at thirty-three, just as the soup guess said. It's called the likelihood. Think of it as a score for each lake size. How likely our catch would be, if the lake were that size. | Peak 33, score ≈ 31.8% (the same percent as 05_4's still, R2 Ruth R21); name card "score of a lake size = how likely our catch would be if the lake were that size" | statement card |  |
| 05_7 | Add up every height, out to lakes as big as you like, and you get about sixteen and two thirds. Real chances would add up to one, a hundred percent. The lake has one size, so these aren't chances that it's a certain size. They're scores for comparing guesses. | A running sum while the axis compresses out to N = 2,000: "17 to 60: 9.9 … all sizes: 16.67", then "16.67 = 1,667%" beside a "100%" mark for real chances (R2 Maya M2). Feller's 1950 card comes in only after the last word, during the hold (R2 Maya M14): "It is, therefore, meaningless to ask for the probability that n [Feller's n: our lake size] is greater than, say, 6000." | live sum | hold |

- **Precisely (lower-third at 05_7, and the description, 05_7):** "Σ_N L(N) =
  Mn/(k(k−1)) = 50/3 when k ≥ 2; infinite when k ≤ 1" (proved, A-F7; R-06,
  D13). The "1,667%" is `A-F7.toy_area_pct_rounded`.
- **Ruth (→ the description, 06_4; R2 R3):** every fixed-k slice rises left of
  the crest and falls right of it (Feller's ratio); fixed-N slices peak near
  it, at ⌊(n+1)(M+1)/(N+2)⌋, tied with one less when that quotient is whole
  (R-27; checked on every N ≤ 40, `A-CH06.row_mode_rule.*`).
- **Key line (R1 X-3):** the music drops out 0.5 s before "The lake has one
  size" in 05_7.
- **Maya:** "score" is the spoken word from here on; the two slices are named
  aloud (M9); "about sixteen and two thirds", never "about seventeen", which
  is also the smallest lake (m2); "a hundred percent" ties the sum to the
  percents of 05_4 (R2 M2); "the mystery lake" names this lake for 07 (R2 M11).
- **Facts to pin (new oracle keys):** L at N = 20, 33, 60 for (10, 10, 3):
  7.8% / 31.8% / 15.9%; the partial sum 17–60 = 9.89.

## 06_the_landscape: Same mountains, two questions (~40 s; the whole chapter on the camera route)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Here's every lake size and every possible count at once. | 05's histograms slide back into rows and rise into the lit bar landscape P_N(K = k), 44 sizes × 11 counts (S-5's field); a still first, with axes labelled in words ("lake size", "caught twice", as 09_2's histogram; R2 Maya M16) fixed in frame | **lit 3D bars + `CameraRig`** |  |
| 06_2 | ⟨predict⟩ Cut across at one lake size. And what do you get? | The camera swings to face a fixed-size slice (a translucent plane, drawn last) | slicing plane, orbit < π |  |
| 06_3 | The chances from counting each netful. They add to one. | The slice's bars light; "Σ = 1" | slice |  |
| 06_4 | Cut along at our three, and you get the score curve. Its heights, out to every lake size, add to sixteen and two thirds. Same mountains, two questions. What catch will this lake give, and which lake gave our catch? | The camera swings 90° to face the k = 3 slice side-on and ends exactly on 05_6's 2D curve; the cut back to the 2D route matches frame for frame; the crest k = Mn/N is drawn last as a ridge over the bar tops; any Σ label on the slice says "all sizes" (the 17–60 slice on screen sums to 9.89; R2 Ruth R14). On the last question each slice gets its question as a label: "what catch will this lake give?" on the fixed-size slice, "which lake gave our catch?" on the score curve (R2 Maya M15) | camera move, matched cut | hold |

- **Engine:** the whole chapter is about 40 s on the camera route (≈ 6 min of
  rendering at S-5's 6.1 fps), so it needs no mid-chapter route join; it opens
  on 05_7's last frame (the histograms sliding into rows) and ends exactly on
  05's 2D score curve; labels are fixed in frame (needs a kit HUD helper, E-X3);
  the crest is a depth-tested ridge over the bar tops (E-A5). Where both
  question labels sit while the camera ends on the 2D curve is a scene-building
  choice for the chapter build (bead mx-e-a-8y5.9).

## 07_feller_s_comparison: Is the score still rising? (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Feller found the top score without drawing anything. Compare each lake size with the one just below it. Divide one score by the other, and almost everything cancels. That leftover fraction is also how code can walk the curve, one size at a time. | Two neighbouring bars from 05's score curve with their ratio between them; the two scores' long counting formulas cancel in quick strike-throughs (R2 Dev dev-04); then "each score = the one before × this fraction" under what's left (R3 Dev D7, Maya maya-04) | statement card |  |
| 07_2 | In the mystery lake, the step from thirty-two to thirty-three leaves twenty-three times twenty-three, over thirty-three times sixteen. That's five hundred twenty-nine against five hundred twenty-eight. The top one wins, so the score goes up. From thirty-three to thirty-four, it's five hundred seventy-six against five hundred seventy-eight, so it goes down. | `529 = 23 × 23` over `528 = 33 × 16`, each factor labelled in table words: 23 "unmarked fish", 23 "fish the second net missed", 33 "lake size", 16 "dark cell" (R2 Maya M3, Dev dev-04); the larger side lights; then `576 = 24 × 24` over `578 = 34 × 17`, labelled the same way | live arithmetic |  |
| 07_3 | ⟨predict⟩ So take the top minus the bottom, written with letters. Multiply it out. And what survives? | `(N−M)(N−n) − N(N−M−n+k)` on one line, with 07_2's word labels kept under the letters, so the letters are a rewrite of what was just seen (R2 Maya M3) | prediction card |  |
| 07_4 | Marked times second catch, minus caught twice times lake size. In the mystery lake, ten times ten, minus three times thirty-three. One. While that's positive, the score is still rising. | Expanded on one line; the terms that cancel strike through in pairs (drawn strike lines): N², N·n, N·M; `Mn − kN` survives; then `10 × 10 − 3 × 33 = 1` (= 529 − 528) | **term-by-term cancellation** |  |
| 07_5 | Now write it with the table's four cells. It becomes first catch only times second catch only, minus caught twice times the dark cell. | `Mn − kN = ab − k·u`; each letter flies to its mosaic cell (`parts_by_tex` + replacement transforms, E-A7) | TransformMatchingTex onto the table |  |
| 07_6 | In the mystery lake, seven times seven, minus three times sixteen. One, same as before. Back to Feller's lake. | 49 − 48 = 1 = 100 − 99; the toy table swaps for Feller's | live arithmetic |  |

## 08_the_table_balances (~55 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | ⟨predict⟩ So start with a small dark cell and let it grow. Where does the climb stop? | The mosaic with a sliver of a dark cell; the two second-catch dividers, left (marked column) and right (unmarked column) | prediction card |  |
| 08_2 | In each column, the green part is what the second net caught. While the green share in the unmarked column is bigger than in the marked column, the score keeps climbing. | The mosaic sweeps N up from 1,900: the dark cell rises out of nothing; green is on top, so the right divider starts below the left one and rises toward it (R2 Ruth R18) | **mosaic sweep** |  |
| 08_3 | A hundred times eight thousand one hundred is eight hundred ten thousand. So is nine hundred times nine hundred. The shares match, the line is straight, and the table balances. | The dark-cell counter ticks to 8,100 as the line goes straight | **counter** | listen 5 s (the motif resolves to 1̂ over the opening pad) |
| 08_4 | A straight line is what two independent catches look like on average. So the size with the top score and the soup guess agree, rounded down to a whole fish, like thirty-three from thirty-three and a third. | Cards merge; the straight divider glows; "33⅓ → 33" beside it | statement card |  |
| 08_5 | At Feller's numbers the line goes exactly straight at ten thousand. So the step up from nine thousand nine hundred ninety-nine changes nothing, and the two tie for the top score. Feller just says ten thousand. | The straight mosaic; the two neighbouring ratios in a lower-third | live fraction | hold |

- **Precisely (lower-thirds; all also in the description, 07_1–08_5):** 07_1
  "L(N)/L(N−1) = (N−M)(N−n) / (N(N−M−n+k)), for N > M + n − k; code walks the
  curve the same way, one multiply per step (in logs for big lakes, see 11_2)"
  (R-18; R2 Dev dev-04); 08_4 "k·u = a·b: the table's odds ratio is 1, the
  2×2 picture of independence; observed tables wobble around it (chapter 09)"
  (R-29; R2 Ruth R8); 08_5 "MLE = ⌊Mn/k⌋; a tie with Mn/k − 1
  when Mn/k is whole and k < min(M, n); none when k = 0; exact fractions tie,
  an lgamma log-likelihood picks 10,000 by rounding" (D26; A-F9.lgamma_prefers;
  a float ratio scan ties exactly, A-F9.float_ratio_L10000_over_L9999); neighbouring ratios
  80,982,001/80,981,901 and 81,018,001/81,018,101 (R-33).
- **Ruth:** Mn − kN = (M−k)(n−k) − k·u is exact algebra (+100, 0, −100 at
  9,999 / 10,000 / 10,001; toy 49 − 48 = 1 → u = 16 → N = 33). With green on
  top, the mosaic's right divider is below the left exactly when Mn − kN > 0
  (creative A-1, checked by algebra; R2 Ruth R18 corrected "above"). The 07_2
  factors are `A-F6.N33.*` / `A-F6.N34.*` (checked against the exact ratio).
- **Music:** the leitmotif resolves here, not at 02's 10,000 (unearned there);
  the 08_3 window is the video's main result (§7.5: 4–8 s); the music thins
  0.5 s before 08_4, a key line (X-3).
- **Cut (R1):** both seesaws (creative A-2); the mosaic carries the balance.

## 09_check_it_by_cheating: Ten thousand lakes (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Let's check this by cheating. Build a lake of exactly ten thousand fish, and run Feller's experiment on it ten thousand times. | Act card "How sure can we be?" (2 s); a run counter "experiment 1 … 10,000"; a 100 × 100 grid of tiny round lakes, each lighting with its estimate (10,000 flat cells, E-A8; legend "each dot = one whole experiment") | **10,000 flat cells recoloured in place** | listen 5 s (twelve trials as separate plinks at the pitch of their caught-twice count, then a wash; the peak's note grows out of it) |
| 09_2 | ⟨predict⟩ Where do the counts of fish caught twice pile up? | Empty histogram axes, "fish caught twice" | prediction card |  |
| 09_3 | They pile up in a bell around a hundred. Most land within about nine of it. | Histogram of the panel's 10,000 runs' k (09_6), labelled "SD 9.0" (exact 9.00; these runs 8.96) | histogram prop |  |
| 09_4 | But the guess divides by that count, and division bends things. Ninety caught twice gives about eleven thousand one hundred. A hundred and ten gives about nine thousand one hundred. Ten too few pushes harder than ten too many. | The k-bell's bars travel along the hyperbola N = Mn/k (05's crest curve) and stretch into a right-skewed histogram | **bars along a curve** |  |
| 09_5 | Ninety-five percent of the guesses land from about eighty-five hundred to just over twelve thousand. Their average is about ten thousand and ninety, a little high. | Quantiles 8,475 / 10,000 / 12,048; mean ≈ 10,090 | live stats |  |
| 09_6 | *(for programmers)* Two lines of code run all the experiments. The last line checks their average against the exact one, built by the same rule as each netful. They agree, within the simulation's own wobble. Always check a simulation against something you can compute exactly. | Code panel (CM Typewriter), copyable in the description: greyed setup `import numpy as np` / `from math import comb` / `rng = np.random.default_rng(20261009)` / `M = n = 1_000; N = 10_000` / `marked = np.arange(N) < M`; then the two lines that net the fish, `k = np.array([marked[rng.choice(N, n, replace=False)].sum() for _ in range(10_000)])` / `est = M * n / k`; then the exact line `p = [comb(M, j) * comb(N - M, n - j) / comb(N, n) for j in range(n + 1)]` and the check `print(est.mean(), sum(M * n / j * p[j] for j in range(1, n + 1)))`, with its output line verbatim under it (`A-CH09.panel.print_line`: 10088.586… 10082.498…) (R2 Dev dev-01, Ruth R20, Maya M17) | code panel slides in |  |

- **Facts:** from R2 the panel nets fish (`rng.choice` of the lake's fish
  without replacement) instead of sampling the hypergeometric law with
  `rng.hypergeometric`, so the check against the exact law tests chapter 04's
  claim instead of numpy's sampler (Dev dev-01). Everything chapter 09 and the
  12_2 ring toss show comes from this panel's own 10,000 runs, keys
  `A-CH09.panel.*` and `A-CH12.ring_toss_panel.*`: SD of k 8.96, mean 10,088.6
  (± 9.2), quantiles 8,474.58 / 10,000 / 12,048.19 (equal to the exact ones),
  no run with k = 0; the exact conditional mean is 10,082.5 (A-F16), 0.66
  standard errors away ("they agree"). The 09_1 listen window's twelve plinks are
  the first twelve runs, `A-CH09.panel.first12_k`. The R1 panel's stream
  (`A-F19.sim_*`, `A-CH12.ring_toss.*`: mean 10,080.002, SD 8.91, misses 3 and
  447) is the replay of the research script and is no longer shown.

## 10_small_experiments: Dividing by zero (~45 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | ⟨predict⟩ With a smaller experiment it gets worse. In a lake of a thousand fish, mark fifty and catch fifty. How often does the second catch hold no marked fish at all? | A small lake; the second net comes up | prediction card |  |
| 10_2 | About one time in fourteen. Then the soup guess divides by zero. | P(k = 0) = 7.2%; an estimate flies off the edge | red |  |
| 10_3 | Chapman's fix is to add one to each count, and subtract one at the end, as if one imaginary fish had been caught both times. It never divides by zero. With none caught twice it still answers two thousand six hundred, a number, not an answer. | A ghost fish (dashed gold outline) drops into the caught-twice cell as M and n tick up by one; `(M+1)(n+1)/(k+1) − 1`; at none caught twice, 51 × 51 ÷ 1 − 1 = 2,600 in ink with "a number, not an answer: chapter 12's range has no top" (R3 Dev D1) | TransformMatchingTex |  |
| 10_4 | Here we expect only two and a half caught twice. Over the experiments that catch any, the soup guess runs about twenty-four percent high, and Chapman's runs about six and a half percent low. Once you expect four or more, its average lands within two percent of the truth, though any one guess still scatters widely. | Two needles on one gauge around 1,000: the plain soup guess at +24% (experiments that caught any) in red, Chapman at −6.5%; a label "expected caught twice here: 2.5" by the gauge; the within-2% wedge labelled as the average's band (R2 Ruth R17, Maya M18, Dev dev-03; R3 Ruth R3-ruth-01: the bias is under 2%, not a typical error; Maya maya-13: split from 10_3) | gauge | hold |

- **Precisely (lower-third at 10_3, and the description, 10_3):** "E[N̂_C] =
  N − (N+1)·C(N−M, n+1)/C(N+1, n+1): exactly N when M + n ≥ N, and never low by
  more than N·e^(−Mn/N), under 2% once you expect 4 caught twice" (R-08,
  proved by Ruth: each factor (N−M−j)/(N+1−j) ≤ 1 − M/N; brute-forced on
  583,220 triples). At k = 0 Chapman still answers (2,600 here), a hint the
  experiment was too small (D12). The voice's "two and a half" is
  `A-F21.expected_k` and "within two percent" is
  `A-F18.bound_pct_at_expected_k4` (1.83%).
- **Dev (the description, 10_3):** the "+1" is two absorption identities:
  (M+1)/(k+1)·C(M,k) = C(M+1,k+1) and (N+1)/(n+1)·C(N,n) = C(N+1,n+1) (D5
  corrected the plan's wrong form). Planning card: "typical error (1 SD): about
  ±32% at 10 caught twice, ±10% at 100 (1 ÷ √(caught twice), a little less
  when the catches are a big share of the lake); a 95% range is about twice
  that, lopsided upward when few are caught twice" (R-24, D4; R2 "±30%"
  corrected to 1/√10 = 31.6%).
  In numpy `M*n/k` at k = 0 is `inf` with a RuntimeWarning (D9;
  `A-CH09.numpy_*`). The planning card's 1/√k is `A-CH12.planning.*`.

## 11_how_sure: Turning it around again (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | So far the lake was known, and we watched the guesses scatter. Now turn it around. We don't know the lake. Which sizes could plausibly have given us our hundred caught twice? | Score curve for Feller's numbers (relative to its top) | Axes + `c2p`, FunctionGraph |  |
| 11_2 | Only nineteen hundred fish? Then the second net scooped up all nine hundred unmarked fish and only a hundred of the thousand marked ones. The chance of that is about five in a one followed by four hundred thirty zeros. | Zeros scroll across the screen for 3 s; a dashed marker at the 324th zero, "a computer's everyday numbers (float64) round to zero past here" (smallest positive ≈ 4.9 × 10⁻³²⁴; R2 Maya M19, the word float64 kept for Dev's dev-15) | scrolling digits |  |
| 11_3 | A million fish? Then we'd expect to catch about one marked fish, not a hundred. | Marker far right | number line |  |
| 11_4 | ⟨predict⟩ Even at the best size, catching exactly a hundred had only about a four percent chance. Is that a problem? | 4.43% at 10,000 | prediction card |  |
| 11_5 | Flip a hundred coins. Exactly fifty heads comes up only about eight percent of the time, yet it's the most likely count. | Coin inset: P(50 of 100) = 7.96% | inset histogram |  |

## 12_the_ring_toss: A range that keeps its promise (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Now the careful range. For each lake size, ask whether a hundred caught twice would land in that lake's middle ninety-five percent. Keep the sizes where it would. | 05's fixed-size slices, both 2.5% tails shaded, at the two edges | interval construction |  |
| 12_2 | Build that range for every experiment. In the long run, it rings the true size at least ninety-five times in a hundred, whatever that size is. Like a ring toss. | A peg at 10,000; the ranges of chapter 09's first 100 experiments (the 09_6 panel's runs, in run order) drop as rings in a lagged cascade; 98 clack on, 2 miss in red (runs 13 and 71); counter "all 10,000: 464 missed (455 expected)"; then the name **confidence interval**; then, under the name card as the rest fades, dim: "with none caught twice, the range has no top" (R3 Dev D1) | ring toss | listen 3 s (wooden clacks) |
| 12_3 | Statisticians call it a ninety-five percent confidence interval. For Feller's catch it runs from about eighty-four hundred to about twelve thousand one hundred. Close to the range we watched when the lake was known, but a different question. There the lake was fixed and the catch moved. Here the catch is fixed, and we try every lake size. | The name card from 12_2 and its line stay up, alone, while the name is said, and goes at "For Feller's catch" (R3 Maya-14); [8,400, 12,073] shaded; 09's range (8,475–12,048) stacked under it, both labelled | shading |  |
| 12_4 | ⟨predict⟩ What makes the range tight? | The range on its own, with three knobs: marked, second catch, caught twice | prediction card |  |
| 12_5 | Mostly, more fish caught twice, which means marking more and catching more. Four times as many roughly halves the range. Catching a bigger share of the lake helps more and more, and when every fish is caught twice, the range shrinks to a single size. | Slider on "expected caught twice" with the lake fixed at 10,000 and M = n = √(10,000·k); the 95% range in percent, −16% / +21% at 100, with "typical error ±9%" beside it and under that "rule of thumb: 1 ÷ √(caught twice) = 10%" (at 25: ±19% against 20%; at 400: ±4% against 5%; R2 Dev dev-11); the bar shows the dot's own range from the first frame (R3 Ruth-06); with every fish caught twice the range closes to one tick at 10,000 | tracker-driven interval | hold |

- **Precisely (lower-thirds; all also in the description, 11_2–12_5):** 11_2
  "below float64's smallest positive (5 × 10⁻³²⁴), so code works in logarithms"
  (D15); 12_2 "exact test inversion; coverage ≥ 95% at every lake size; 95.45%
  at this one" (R-22); 12_3 "Feller's own 8,500–12,000 matches one-sided levels
  3.5% and 2.9%: confidence ≈ 93.6%" (R-25).
- **Bayesian line (→ the description, 12_3; R-30, R2 Ruth R3):** "Want a
  probability for the lake size? You need a starting belief." With a flat prior
  on N ≥ 1,900: 8,536 to 12,186, median 10,121 (A-F26): almost the same
  numbers, a different question. Not spoken: chapter 12 is the video's
  answer to "how sure", and a second framework in the voice would blur 12_2's
  coverage reading.
- **Ruth (→ the description, 12_5; R-32, R2 R3):** the relative error ≈
  √((1 − M/N)(1 − n/N)/k) is the symmetric delta method; the exact SD of Mn/K
  (given K > 0) is 924. Marking 2,000 and catching 500 is the same experiment as
  marking 500 and catching 2,000 (Feller eq. 6.2): SD 894 both ways.
- **Facts (oracle keys `A-CH12.ring_toss_panel.*`, `A-CH12.ring_toss.*`,
  `A-CH12.coverage.*`, `A-CH12.slider.*`):** from R2 the rings are the 09_6
  panel's own experiments: misses among the first 100 (2: runs 13 and 71) and
  among all 10,000 (464, against 454.7 expected at the exact coverage 95.45%);
  never pick a window to show a miss (D2, R-22). "Whatever that size is": the
  coverage is at least 95% at every lake size by construction, and the oracle
  checks it on 27,901 lake sizes from 1,000 to 1,000,000 (minimum 95.007%, at
  10,967; R2 Ruth R9). The 12_5 slider's exact ranges are 8,274 / 3,673 / 1,597
  wide at k = 25 / 100 / 400 (each fourfold step × 0.44, then × 0.43: the 1/√k
  halving plus the growing share), and [10,000, 10,000] once both catches are
  the whole lake; typical errors 19 / 9 / 4% against the rule of thumb's
  20 / 10 / 5%.

## 13_one_rule_for_every_failure (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | Every failure breaks one rule. On average, the second catch must hold the same share of marked fish that the lake held on marking day. | Act card "When does it fail?" (2 s); the true lake's mosaic with its divider straight; the guess's needle on a gauge labelled "the soup guess, Mn/k (truth on marking day: 10,000)" (R3 Maya-12); the lower-third below | gauge prop |  |
| 13_2 | ⟨predict⟩ Some fish are bold and easy to catch, both times. Will the guess be too big, or too small? | Bold fish glow; quiz card | prediction card |  |
| 13_3 | Too small. Bold fish land in the caught-twice cell more than their share, so the dark cell shrinks. If three fish in ten are three times as easy to catch, the guess comes out around seventy-seven fifty. | The mosaic's divider bends; the estimate's mosaic straightens it by shrinking the dark cell; needle to ≈ 7,750; label "model: 3 in 10 fish 3× as catchable, both times; median of 400 runs" | bent mosaic |  |
| 13_4 | Fish that learn to love the bait do the same. Fish that learn to avoid it push the guess up. | Down to ≈ 7,140, up to ≈ 14,490; labels "model: marked fish 1.5× as catchable" / "⅔×" | gauge |  |
| 13_5 | If marks wash off, marked fish look unmarked. That pushes it up too. | ≈ 12,500; label "model: 20% of marks lost" | gauge |  |
| 13_6 | Two thousand newcomers dilute the marks, so you count the lake at the second catch. | ≈ 12,120 beside a second gauge mark "lake at the second catch: 12,000"; label "model: 2,000 unmarked fish join between the catches" | gauge |  |
| 13_7 | ⟨predict⟩ What if three fish out of ten die between the catches? | The lake thins | prediction card |  |
| 13_8 | The guess doesn't move. Death doesn't care about marks, so the share stays one in ten. But it counts the lake on marking day, not the seven thousand left. | Needle stays at 10,000 (the marking-day mark); a second gauge mark "lake at the second catch: 7,000", mirroring 13_6's "12,000", lights as the voice says "seven thousand" (R2 Ruth R1, Maya M1, Dev dev-02); label "model: each fish dies with chance 3 in 10"; the music drops out for 1.5 s after the beat | huh beat |  |
| 13_9 | If the marks never mix, the answer depends on where you fish. Near the dock you'd say sixty-two fifty, on the far side twenty-five thousand. | Chapter 03's lake panel, now *not* evening out (hot at the dock); label "two-halves model: all marks in one half; second catch 80% from one half"; ≈ 6,250 / ≈ 25,000; Feller's caveat on a card (silent): catches made in one spot within a short time; a closing line under the panel: "two catches alone can't show which rule broke; more lists let you check some of it (chapter 15)" (R3 Dev D2) | callback | hold |

- **Precisely (lower-third at 13_1, and the description, 13_1; R2 blocker,
  Ruth R1's wording):** "precisely: Mn/k = M ÷ (marked share of the second
  catch). It aims at a lake of L fish (Mn/E[k] = L) exactly when that share is,
  on average, M/L. Even then it runs a little high (chapter 09)." The R1
  wording ("biased exactly when that share differs from the marked share of
  the lake you mean") was false for deaths, where the shares match the
  thinned lake yet Mn/k aims at the lake on marking day, and in the ideal case,
  where the shares match yet Mn/K runs high (mean 10,082.5 given K > 0).
- **The two targets, said aloud (R2):** with M the number marked, the share
  rule makes Mn/k aim at the lake on marking day (13_1); deaths keep the share
  at M/10,000, so the needle stays at 10,000 while about 7,000 fish are left
  (13_8, `A-CH13.deaths_30pct.*`); newcomers drop it to M/12,000, the lake at
  the second catch (13_6, `A-CH13.newcomers_2000.*`); lost marks drop the
  visible share to M/12,500, so the needle reads 12,500
  (`A-CH13.marks_lost_20pct.*`). Ruth's R2 delight D5:
  newcomers count the lake at the second catch, deaths count it on marking day.
- **Ruth:** each case is a seeded simulation (true lake 10,000, M = n = 1,000;
  A-F22); every gauge names its model (R-07, D8), and the full model
  parameters go to the description (13_3–13_9). The 13_9 numbers come from the
  two-halves model, drawn as such (R-10, D7); "the dock" is where chapter 03
  released the marks (R2 Maya M9). Schnabel's 1938 paper in the *American
  Mathematical Monthly* goes to the description's sources (fact-sheet row 7
  verifies the citation, not its content, so the description cites it only
  for "others had been using it", 02_1).

## 14_the_dark_cell_everywhere: France and the bugs (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | Around eighteen hundred, the French mathematician Laplace used the same table to count France. Births were written down, so in our table, each newborn is a marked fish. | The mosaic stays at screen centre for the whole chapter; its fish glyphs morph into newborn glyphs; France's outline as a side inset (public-domain outline, E-A10) | template morph |  |
| 14_2 | At his request, officials counted everyone in chosen towns across France. Just over two million people, with about seventy-two thousand births a year. That's one birth for every twenty-eight and a bit people. Taking France's births as a million a year, that's about twenty-eight point four million people. | The mosaic relabelled: births = marked, people counted = second catch, births among them = caught twice | template morph |  |
| 14_3 | *(silent: the odds card)* | Laplace, *Essai philosophique* (tr. Truscott & Emory 1902, p. 67): "a bet of almost 300000 against 1" that he was within half a million; "his calculation; it treats the chosen towns as typical" | odds card | hold |
| 14_4 | *(for programmers)* ⟨predict⟩ It counts bugs as well. Say two reviewers read the same code without comparing notes. One finds thirty bugs, the other twenty-four, and twelve are the same. How many are still hiding? | The glyphs morph into bug glyphs; two lists and a Venn (boolean ops) as an inset; no "for programmers" tag on screen here, the easiest prediction in the video (R1 Maya m26); lower-third (R2 Ruth R15, Dev dev-08): H. D. Mills (1970; IBM report, 1972), *Software Productivity* p. 73: "'lake' is synonymous with 'system'…"; his marked fish were bugs planted on purpose | prediction card |  |
| 14_5 | Thirty times twenty-four, over twelve, is sixty. They found forty-two, so about eighteen are still hiding. But only twelve were caught twice, and that's not many. If every bug is equally easy to find, the careful range for all the bugs runs from forty-seven to ninety-four. | Range bar 47–94 on the mosaic's estimate, labelled "if every bug is equally easy to find" and "careful range for all the bugs: 47 to 94, so 5 to 52 still hiding" (R3 Maya-03, Ruth-12) | range bar |  |
| 14_6 | Reviewers of different skill cause no trouble. Bugs of different difficulty do. Bugs that are easy for both reviewers swell the caught-twice cell and pull the guess down, like the bold fish. So expect sixty, and the range, to run low. | Callback to 13_3's bent mosaic; a small model card: "model: 30 easy bugs, 30 hard; the range missed the true 60 in 77% of runs, always below it" (`A-CH14.reviewers.difficulty.*`); under it, dim, with the "no": "specialist reviewers: guess runs high" (R3 Dev D12, Ruth-11) | callback |  |

## 15_counting_the_dead: Kosovo, and the coda (~40 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | And it has counted the dead. Between March and June of nineteen ninety-nine, in the war in Kosovo, four lists named about four thousand four hundred different victims between them. Together they make a table of sixteen cells, one for each mix of lists, and one cell holds the people on no list. From how the lists overlap, researchers estimated about ten thousand four hundred Kosovar Albanians killed in those months. With four lists, they could also allow for lists that lean on each other, which two catches can never do. Years later, a name-by-name count for the same months found nine thousand seven hundred ninety, inside their range. | Sober typography only, each picture on its words (R3 Ruth-10, Dev D2, Maya-16): the title "Kosovo, 1999" alone; the named bar on a line of people at "four lists named"; the four-list table, sixteen cells, one dark, at "make a table", and "on no list" at its words; the estimate and the range 9,002–12,122 at "researchers estimated"; at "With four lists", "4 lists: the model can let lists lean on each other", then dim "two catches: it has to assume they don't"; the later count at "a name-by-name count"; lower-third (R2 Dev dev-10): "four lists; log-linear models (multiple systems estimation): Ball et al. 2002, report to the ICTY; the later count compared by Spagat 2014"; the music fades to silence over 1.2 s in the gap before this beat, never mid-sentence | quiet card, **silence** under the numbers | hold |
| 15_2 | ⟨predict⟩ France, the bugs, Kosovo. What do they share? | The three tables side by side, each with one dark cell | prediction card |  |
| 15_3 | Each is a table with one cell nobody can see, filled in by the cells we can. | The mosaic sinks into the lake surface and the swarm re-forms (01_2 in reverse); receipts card "Every number in this video was computed by code or copied from a cited source: link" | coda; the motif alone on felt piano, the pad only under the last shot | listen 3 s |

- **Ruth (→ the description, 14_2–15_1; R2 R3, R16):** Laplace's sample:
  thirty départements spread evenly over France, and within them the communes
  with the most reliable records, counted by the government at his request
  (2,037,615 people; 215,599 births in 3 years = 71,866⅓ a year; ratio
  28.352845; 28,352,845 for 1,000,000 births, which Laplace *assumed*); not a
  census (R-03, R-16). The odds rest on his model, in which the chosen communes
  stand in for France (R-14). Dropped from the description: Cochran 1978 and
  Pearson 1928 (nobody has read them for this video; no fact-sheet row).
  Kosovo used multiple systems estimation with log-linear models on four
  sources, not a two-list formula; Ball et al. estimate Kosovar Albanians
  killed, 20 March to 22 June 1999 (pp. 3–4); Spagat compares the estimate with
  the Kosovo Memory Book's violent deaths for 20 March to 20 June and notes it
  may run "a bit too high" from false reports (2014, p. 9) (R-02, R-15, R2
  R16; fact-sheet row 12, re-read for R2). Mills (IBM report, 1972; his
  collected papers date the article 1970) seeded bugs deliberately: painted
  fish (R-31; R2 R15: the 14_4 lower-third).
- **Dev (→ the description, 14_4–14_6):** two reviewers of different skill are
  the M_t model, which the formula survives when they read independently
  (A-CH14.reviewers.skill: median 60 for skills 0.5 / 0.4; the careful range
  holds 60 in 97% of runs); bugs that differ in difficulty for everyone (M_h)
  bias it low (A-CH14.reviewers.difficulty: median 41.25, and the careful
  range sits wholly below 60 in 77% of runs, never above; D11; R2 dev-05;
  Briand et al. 2000, IEEE TSE 26(6), "underestimation may be substantial"
  when the inspectors are too few). The 30 / 24 / 12 example is hypothetical
  (D10: "say"). Exact 95% range [47, 94] by test inversion; Chapman 58.6 ± 8.4.
  Dev's Eick et al. (ICSE 1992) is left out: Dev recalled it, nobody has read
  it (R2 dev-08).
- **Dropped:** the German tank problem as a contrast (R-28): out of scope in
  the brief, and it would need a ledger row citing Ruggles & Brodie (1947).
