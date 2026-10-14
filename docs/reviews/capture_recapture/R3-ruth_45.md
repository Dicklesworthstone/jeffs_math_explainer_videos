# R3 review: capture_recapture, narrated draft. Ruth (ruth_45)

Reviewer: Prof. Ruth Adler, 45, mathematician who reads like a statistician. I judged correctness
first. Pacing and vocabulary belong to Maya, and which mechanisms make the cut belongs to Dev.

**Verdict.** Every number the voice speaks is right, and I recomputed each one. The probability
language is careful: the likelihood is never called a probability of N, the range is described as
a procedure's coverage, and every "always" carries its qualifier or has its exact version in
Precisely. Two things would change the video. 10_3 says "within two percent on average", which
reads as an accuracy claim, and the claim is false. The 13_7 frame puts the newcomers' 12,120
under the "deaths" title. The rest are small fixes, mostly in the description. I would show this
to my students once the two majors are fixed.

## What I checked, and how

- **Sheets.** I opened all 15 cue sheets and enlarged every tile, and matched each tile to its
  legend: tile i is line i, and any trailing tiles repeat the chapter's last frame. The packet also
  lists three chapter-15 "dense" sheets. They were not in my set, so I did not review them.
  Anything that appears on screen only after the 1.5 s frame of a line is **unchecked**, because
  no tile shows it. That covers:
  - the 08_5 lgamma lower-third;
  - the 09_3 SD label;
  - the 11_2 float64 marker;
  - the 12_3 range label;
  - 13_6's "lake at the second catch";
  - 05_7's "all sizes: 16.67";
  - the silent 14_3 Laplace odds card.
- **Recomputation.** I ran `python3 -I` with exact `fractions` and `math.comb` against every spoken
  number from chapter 4 to chapter 14. All of them match:
  - **Chapter 4:** 4,845 and 116,280 ÷ 24; the piles 1,365 / 2,275 / 1,050 / 150 / 5; the
    percents 28.2 / 47.0 / 21.7 / 3.1 / 0.1.
  - **Chapter 5:** L(20) = 7.8%, L(33) = 31.8%, L(60) = 15.9%; the peak at 33; the area 16.6667,
    with the 17–60 slice summing to 9.89.
  - **Chapter 7:** the ratios 529/528 and 576/578.
  - **Chapter 8:** the exact 9,999 / 10,000 tie.
  - **Chapter 9:** SD(K) = 9.0005; P(91 ≤ K ≤ 109) = 70.9%; P(83 ≤ K ≤ 118) = 95.45%;
    E[Mn/K | K > 0] = 10,082.50; the quantiles 8,474.58 / 12,048.19; 11,111 against 9,091.
  - **Chapter 10:** P(K = 0) = 0.0720 (1 in 13.9); Chapman's mean 935.22 (−6.48%), and its
    closed form N − (N+1)·C(N−M, n+1)/C(N+1, n+1) equals the direct sum exactly; 2,600 at k = 0.
  - **Chapter 11:** 5.3515 × 10⁻⁴³⁰; 4.43%; 7.96%.
  - **Chapter 12:** the range [8,400, 12,073] by test inversion; the slider ranges [6,936, 15,210]
    and [9,254, 10,851]; Feller's tails 3.47% and 2.89%; the k = 0 range starting at 272,086;
    P(K = 0) at a lake of a million = 36.75%.
  - **Chapter 14:** Laplace's 215,599 births, 71,866⅓ a year, ratio 28.352845, total 28,352,845;
    the bugs' 60, 42 found and range [47, 94].
- **Oracle.** I ran `oracle.py` read-only (numpy 2.5.4) and compared its keys with the screen
  numbers: `A-CH03.portable.*`, `A-CH12.slider.*`, `A-CH12.ring_toss_panel.run1_k_ci95` (101 →
  8,326–11,940, matching 12_2's label) and `A-F22.median_rounded_10.*`. It printed 607 keys,
  while the fact sheet's header still says 547. That staleness is in the sheet, not the video.
- **Code panel.** I ran the description's code panel exactly as printed. It prints
  `10088.586031204479 10082.498763370464`, as stated.
- **Spagat (2014).** I read pp. 7–9 myself from the PDF text. Three things are confirmed there:
  "The KMBD number for this period is 9,790, placing it right in the middle of the BBSDA range";
  "a bit too high"; and the 20 March–20 June window "restrict[ed] … to violent deaths". That
  passage states no restriction by ethnicity.
- **Not re-read by me:** Ball et al. (2002) (I relied on the fact sheet's quotes), Chapman (1951),
  Wittes (1972), and Feller's 3rd edition.

## Comprehension check

1. **Hook.** One net can't know what share of the fish it reached: three spots in the same
   10,000-fish lake give 2,800 to 88,000. Marking and recatching can, and they put the dark cell at
   about 8,100 with an honest error bar.
2. **The marks measure the net.** 100 of the 1,000 marks came back, so the second net took about a
   tenth of the fish (equivalently, a tenth of the lake is marked). So N ≈ Mn/k = 10,000, the
   "soup guess".
3. **Why the marks spread.** Mixing makes being marked independent of where a fish lives, so a net
   cast anywhere carries the lake's one-in-ten share, with more wobble in small regions. Equal
   catchability is a separate assumption.
4. **Netfuls.** Sort all C(20, 4) = 4,845 equally likely netfuls by how many marked fish they hold
   and you get the hypergeometric law: knowing the lake fixes the chance of every catch.
5. **Turning it around.** The likelihood scores each lake size by how probable our catch would be.
   It peaks at 33 (the soup guess, rounded down), but it sums to 16⅔, so it is not a probability
   distribution over lake sizes.
6. **The landscape.** In one table of P_N(K = k), a slice at fixed N is a probability law that
   sums to 1, and a slice at fixed k is the likelihood. Same numbers, two questions.
7. **Feller's comparison.** In L(N)/L(N−1) almost everything cancels: the score rises exactly while
   Mn − kN > 0, and in the table's cells that is ab − k·u.
8. **The table balances.** The score climbs until k·u = a·b (odds ratio 1, which is what
   independent catches look like on average). So the best size is ⌊Mn/k⌋, and at Feller's numbers
   9,999 and 10,000 tie exactly.
9. **Check it by cheating.** In a known lake of 10,000, k forms a bell around 100 with SD 9.
   Dividing by k skews the estimates high (mean about 10,083; middle 95% from 8,475 to 12,048).
   Check a simulation against the exact law.
10. **Small experiments.** With 2.5 expected recaptures, k = 0 happens 1 time in 14 and Mn/k blows
    up. Chapman's +1/−1 never divides by zero, and its bias is under 2% once you expect 4 or more.
11. **How sure.** The likelihood across sizes rules out 1,900 and a million. A 4% chance for the
    observed count, even at the best size, is normal, like 8% for exactly 50 heads.
12. **The ring toss.** Keeping every lake size whose middle 95% contains our 100 gives 8,400 to
    12,073. The procedure covers the true size at least 95% of the time whatever N is, and its
    width shrinks mainly with the number caught twice.
13. **One rule for every failure.** Each failure shifts the expected marked share of the second
    catch:
    - bold or bait-loving fish push the estimate down;
    - bait-shy fish and lost marks push it up;
    - newcomers move the target to the later lake;
    - deaths change nothing but the meaning;
    - unmixed marks make the answer depend on where you fish.
14. **The dark cell everywhere.** The same table estimated France (births as marks, about 28.4
    million) and estimates hidden bugs (60, range 47–94). The estimate runs low when some bugs are
    easy for both reviewers.
15. **Counting the dead.** Multiple-systems estimation on four lists of Kosovo victims gave about
    10,400 killed, and a later name-by-name count (9,790) fell inside its range. Every example is a
    table with one cell nobody can see.

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight | note |
|---|---|---|---|---|---|---|
| 01 hook | 4 | 5 | 5 | 4 | 4 | the 31× dips are the right opening; small label/voice lag at 01_7 |
| 02 marks measure the net | 5 | 5 | 4 | 4 | 4 | the two readings of one fact are excellent |
| 03 why the marks spread | 4 | 4 | 4 | 4 | 4 | 03_4's "t = 0" readout; the Precisely line quotes the wrong run |
| 04 netfuls | 5 | 5 | 4 | 4 | 3 | |
| 05 turning it around | 5 | 5 | 5 | 4 | 5 | likelihood ≠ probability, done properly |
| 06 the landscape | 3 | 5 | 4 | 4 | 4 | the 3D landscape is hard to read at draft size (Maya's call) |
| 07 Feller's comparison | 4 | 5 | 4 | 3 | 4 | dense algebra; correct |
| 08 the table balances | 4 | 4 | 4 | 4 | 5 | "top score 10,000" mixes up max and argmax |
| 09 check it by cheating | 5 | 5 | 5 | 4 | 4 | the code runs as printed |
| 10 small experiments | 4 | **3** | 4 | 3 | 5 | "within two percent on average" |
| 11 how sure | 5 | 5 | 5 | 4 | 4 | |
| 12 the ring toss | 4 | 4 | 5 | 4 | 5 | the whole-lake edge; slider/bar mismatch on its first frame |
| 13 one rule for every failure | 4 | **3** | 5 | 4 | 4 | stale 12,120 under "deaths" |
| 14 dark cell everywhere | 4 | 4 | 5 | 4 | 5 | the Precisely notation clash; the direction of the bug bias |
| 15 counting the dead | 4 | 4 | 5 | 4 | 4 | respectful and sober; the voice drops the estimate's scope |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| R3-ruth-01 | Ruth | 10 / 10_3 | major | "Once you expect four or more, it's within two percent on average" naturally reads as "a typical Chapman estimate is within 2% of the truth". That is false. Only the **bias** is under 2%, and the bound N·e^(−Mn/N) = 1.83% is correct. The error itself is large. I computed the exact law: at N = 1,000, M = n = 64 (E[k] = 4.1), the bias is −1.10%, the mean absolute error is 35%, and the SD is 54%. At N = 10,000, M = n = 200, they are −1.62%, 37% and 57%. This is the bias-versus-spread distinction that chapters 9–12 teach, stated backwards. | Voice: "Once you expect four or more, its average lands within two percent of the truth, though any one guess still scatters widely." Precisely 10_3: add "That bounds the bias; at 4 caught twice one estimate's typical error is still about ±50% (chapter 12's planning rule)." |
| R3-ruth-02 | Ruth | 13 / 13_7 (picture) | major | At 13_7 the title reads "deaths" and the needle sits at 10,000, but the big readout says **≈ 12,120**. That is the newcomers' median (`A-F22.median_rounded_10.newcomers_2000`) left over from 13_6. The voice is asking "What if three fish out of ten die?", so the screen shows a wrong answer, and 13_8 then contradicts it ("doesn't move"). Every other scenario resets to ≈ 10,000 on its first frame; this one does not. | Reset the readout with the title: show "≈ 10,000" or a neutral "?" during 13_7, as 13_4, 13_5, 13_6 and 13_9 do. Then let 13_8's reveal keep the needle and the readout at 10,000. |
| R3-ruth-03 | Ruth | 03 / description Precisely 03_4, 03_5 | minor | The exact version quotes the numpy run, but the scene draws the portable run. Precisely says shares go "to 9.3–10.7% at t = 400" with counts "about 12.8 times apart", while the portable run has 8.6–12.3% and 2,811 / 204 = 13.8× (`A-CH03.portable.t400.*`). The very next bullet (03_6) cites NE at 12.3%, which lies outside "9.3–10.7%", so the description contradicts itself. The "nets of 10 or 20 fish hold 0–3 and 0–5" come from the numpy frame (`A-CH03.trays.*`), not from the frame on screen. | Rewrite 03_4 from the portable keys: "65.9% at the start (dock region) to 8.6–12.3% at t = 400, all within about 1.1 binomial SDs; region counts stay about 12.8× apart at t = 0 and 13.8× at t = 400." In 03_5, either re-pin the 10- and 20-fish nets on the portable t = 400 frame or drop them. Also check the "14% drift" sentence against the portable run. |
| R3-ruth-04 | Ruth | 03 / 03_4 (picture) | minor | The clock reads "t = 0", but the dock-side regions read SW 18%, S 56%, SE 15%. The oracle's t = 0 is 14.2 / 65.9 / 11.5% (`A-CH03.portable.t0.marked_percent_1dp`), and the storyboard promises "66% at the dock". Either the clock floors an already-running time, or the shares on screen differ from the oracle's. Either way a viewer who compares the screen with "65.9% at the start" sees a mismatch. | Hold the t = 0 state for about a second so 66% is visible before the clock starts, or show t to one decimal. Then confirm that the scene's step-0 region shares equal the t0 key. |
| R3-ruth-05 | Ruth | 12 / 12_5 (voice) | minor | "Catching the whole lake shrinks the range to a single size" holds only when **every fish is caught twice** ([10,000, 10,000], since a lake of 10,001 gives that catch with chance 1/10,001). If only the second net happens to take the whole lake (M = 1,000, n = 10,000, so k = 1,000), my test inversion gives [10,000, 10,035]: nothing in that catch rules out an extra fish. The slider's M = n makes the intended meaning clear on screen, but the sentence alone does not. | Voice: "…and when every fish is caught twice, the range shrinks to a single size." This also matches the description's own wording ("With the whole lake caught twice"). |
| R3-ruth-06 | Ruth | 12 / 12_5 (picture) | minor | On 12_5's first frame the "expected caught twice" dot sits at 25, but the range bar runs −16% / +21%, which is the 100-recapture range (`A-CH12.slider.k100`). At 25 the range is −31% / +52% (`.k25`). The bar and the dot disagree. | Drive the bar from the dot from the first frame, or start the dot at 100. |
| R3-ruth-07 | Ruth | 08 / 08_4 (picture) | minor | The box labelled "top score" shows "10,000". The top score is the value 0.0443 (4.43%). 10,000 is the lake size that attains it, and so does 9,999 (08_5). The voice gets it right ("the size with the top score"), but the box conflates max with argmax. | Relabel the box "best size (top score)" or "top score at 10,000". |
| R3-ruth-08 | Ruth | 05–08, 11 / 05_6 on; description Sources | minor | "Score" for the likelihood clashes with standard usage, where the score is d log L/dN and is **zero** at the top. A student who later meets the "score test" will be confused by "top score". The Sources line "(for the history behind 'score': Feller, p. 38, credits R. A. Fisher)" deepens the clash: Fisher's own "score" is the derivative. | Keep the word in the voice; 05_6 already says "It's called the likelihood". Add to Precisely: "05_6. 'Score' is our informal word for the likelihood L(N). In statistics 'the score' means d log L/dN, which is zero at the top." Change the Sources parenthetical to "for the history of maximum likelihood". |
| R3-ruth-09 | Ruth | 14 / description Precisely 14_6 | minor | "E[a]·E[b]/E[k] = N" reuses a and b for the reviewers' totals (30 and 24), but the Precisely header defines a = M − k and b = n − k. Under the declared notation the identity is false: in the 0.5 / 0.4 model, E[a]·E[b]/E[k] = 18 = E[u], not 60. (I checked this.) | Write "E[A]·E[B]/E[k] = N, where A and B are the two reviewers' totals (30 and 24 here)". Optionally add the neat cell form "equivalently E[a]·E[b]/E[k] = E[u]". |
| R3-ruth-10 | Ruth | 15 / 15_1 | minor | The scope of the estimate is dropped. Ball et al.'s 10,356 is "Kosovar Albanians killed" between 20 March and 22 June 1999. The voice says only "about ten thousand four hundred killed". Spagat compares it with the Kosovo Memory Book's 9,790 violent deaths for 20 March–20 June, and the passage I read restricts the KMB count to violent deaths only, with no ethnic restriction. So the comparison is not exactly like for like, and that matters in this chapter above all. Tone and picture are respectful (abstract list cells, no imagery of the dead). | Voice: "…researchers estimated about ten thousand four hundred Kosovar Albanians killed in those three months. Years later, a name-by-name count of the same weeks found 9,790, inside the estimate's range." Precisely 15_1: add "Spagat's KMB figure is all violent deaths in the period; he states no ethnic restriction, so the comparison is his." |
| R3-ruth-11 | Ruth | 14 / 14_6 | minor | "So expect sixty, and the range, to run low" holds when the same bugs are easy for **both** reviewers, which is what the model and Briand show. If the reviewers specialise (each finds different kinds of bug), k shrinks and the estimate runs **high**. Both the voice and Precisely state the low direction as the general rule. | Voice: "Bugs that are easy for both reviewers swell the caught-twice cell…". Precisely 14_6: add "Reviewers who specialise in different kinds of bugs do the opposite: k shrinks and the estimate runs high." |
| R3-ruth-12 | Ruth | 14 / 14_5 | minor | The voice gives the range straight after "about eighteen are still hiding", so "forty-seven to ninety-four" can be heard as the number still hiding. It is the total; the screen's "47 to 94 bugs" resolves this for viewers who look. | Voice: "…the careful range for all the bugs runs from forty-seven to ninety-four." |
| R3-ruth-13 | Ruth | description / Names to search | minor | The estimator's standard name, the Lincoln–Petersen estimator, appears nowhere. A viewer who wants the literature won't find it under "capture–recapture" alone. Not naming Petersen is right given Goudie & Goudie's (2007) doubt. | Add "the Lincoln–Petersen estimator (a name whose history is disputed; see Goudie and Goudie 2007)" to Names to search. |
| R3-ruth-14 | Ruth | 01, 02 / 01_7, 02_5 (picture) | minor | Two numbers are spoken before they are shown. At 01_7 the voice says "In the weeds, eighty-eight thousand" while the net is leaving the weeds and the weeds carry no label (the deep end's 45,100 is back, though). At 02_5 the voice opens "Ten thousand" over a nearly empty frame. | Land each label on its spoken word: 88,000 on the weeds' box during "In the weeds", and 10,000 on screen at 02_5's first word. |
| R3-ruth-15 | Ruth | 05 / 05_7 | delight | The likelihood "adds up to sixteen and two thirds", so it can't be a probability of the lake size. Feller's own "meaningless to ask for the probability that n is greater than, say, 6000" sits alongside it, and the closed form Mn/(k(k−1)), infinite for k ≤ 1, is in Precisely. This is the cleanest likelihood-versus-probability treatment I have seen at this level. | Keep as is. |
| R3-ruth-16 | Ruth | 08 / 08_5 | delight | The exact tie: 81,000,000 / 81,000,000 on screen, flanked by 80,982,001/80,981,901 and 81,018,001/81,018,101, then "Feller just says ten thousand". The lgamma tie-break note in Precisely is the kind of honesty I want students to see. | Keep as is. |
| R3-ruth-17 | Ruth | 10 / 10_3 | delight | Chapman's estimator as "one imaginary fish caught both times" is the correct interpretation (M+1, n+1, k+1, then N+1 − 1), and Precisely gives the two absorption identities behind it. I didn't know that reading. | Keep as is (with R3-ruth-01's wording fix). |
| R3-ruth-18 | Ruth | 12 / 12_2–12_3 | delight | "It rings the true size at least ninety-five times in a hundred, whatever that size is" states coverage as a property of the procedure. "There the lake was fixed and the catch moved. Here the catch is fixed" names the frequentist inversion exactly. The flat-prior posterior in Precisely ("almost the same numbers, a different question") is a lovely footnote. | Keep as is. |
| R3-ruth-19 | Ruth | 13 / 13_1, 13_8 | delight | The share rule ("the same share of marked fish that the lake held on marking day") turns deaths from a puzzle into an insight: the needle doesn't move, but it measures the marking-day lake, not the 7,000 left. | Keep as is (once R3-ruth-02's readout is fixed). |
| R3-ruth-20 | Ruth | 14 / 14_4 (description) | delight | Mills's "'lake' is synonymous with 'system'… 'marked fish' with 'calibration errors'", with both dates given (1970 article, 1972 IBM report), is a historical footnote I didn't know, and it is sourced to the page. | Keep as is. |
