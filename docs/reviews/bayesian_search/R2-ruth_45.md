# R2 script review: "How to Find a Lost Submarine" (Ruth Adler, 45, with the mathematician audit)

Saved verbatim from the text the reviewer returned (2026-10-13); the reviewer
did not write the file itself. An earlier attempt at this review was cut off
by a usage limit and left nothing; this is the second, complete run. Finding
ids R1 to R20. One note on the reviewer's own account: it reports reading "all
95 script lines"; `script.tsv` has 82 lines (it may have counted the table
rows of the packet). It also reports that a grep of `docs/` for a few words
surfaced lines of `docs/claims.md` (the P3 claim walk), which it used for
three items (R14, the Table H-15 row, the combined-sensor arithmetic).

I read the packet in order (stage rules, brief, script, storyboard, description draft, fact sheet), the persona prompt and `ruth_audit.md`. I judged pictures only from the storyboard's picture column, since the packet has no frames. Pacing and delight scores rest on the storyboard's seconds and picture notes.

Scratch work is in `<scratch>/ruth_r2/` (`check_sea.py`, `check_rest.py`, `check_07.py`). Nothing was written in the repository.

## Comprehension check

| Ch | One sentence to a friend |
|---|---|
| 01 | Someone asks how you could possibly find a lost submarine in time, and before any answer is spoken we watch a search find nothing and the map change anyway: finding nothing is information. |
| 02 | The question was real in 1968: the *Scorpion* did not come home, all 99 men were already lost, and even finding the wreck was hard because one careful pass over the patch would have meant several times everything the sled had towed. |
| 03 | You build a map of chances by weighing two stories sixty to forty and handing out a hundred dots by what each story says, so cell A gets forty, B thirty, C twenty, D ten. |
| 04 | The Navy built its map the same way, from nine weighted stories played out ten thousand times, and the wreck lay beside the map's top cell, not in it, so the map alone did not find her. |
| 05 | The likeliest cell is not the best place to search, because a search sees a wreck only some of the time: chance of a find is chance it is there times chance of seeing it, and B beats A two to one. |
| 06 | An empty search is evidence: it puts out the dots it would have caught, so B falls from thirty to twelve and a half and every cell not searched rises by a quarter, though nothing in the sea changed. |
| 07 | That move is Bayes' rule (multiply by the chance of coming back empty, divide by what is left), and search, update, choose again is the whole loop. |
| 08 | Searching where the next search is most likely to find it is the best order, because each search takes a fixed slice of the first hundred dots and the rule takes the biggest slices first, and on the Navy's map most of the gain is simply having a map. |
| 09 | A search sees only two in three because strips laid at random leave gaps (sixty-three percent at one full coverage), and a repeat pass finds less than the one before and never makes it certain. |
| 10 | A person in the water drifts, so the cloud of dots spreads and the same flight sees less each hour, which is why hours matter. |
| 11 | The Coast Guard drifts its dots and re-runs them when evidence shows the first story was wrong, and in the re-run map a helicopter found John Aldridge alive after almost twelve hours. |
| 12 | For Air France 447 the analysts mixed three weighted maps, folded in each empty search, and drew the map twice because the beacons' silence might have meant nothing. |
| 13 | In 1968 the ship's numbers called the wreck's own cell ninety-four percent searched because the sonar's width was overestimated, and measured against the wreck it was sixty-five, so an empty search counts only as much as your estimate of how well you looked. |
| 14 | An empty search tells you something only if it could have found the thing; each of the four was put right by re-measuring, a second map or new evidence, the 99 could not be saved, and the method now searches for people who can. |

## Rubric (1 to 5)

| Ch | clarity | correctness | motivation | pacing | delight | what holds the lowest score down |
|---|---|---|---|---|---|---|
| 01 | 5 | 5 | 5 | 4 | 4 | nothing found |
| 02 | 4 | 4 | 5 | 3 | 3 | seven quantities in four sentences (R11) |
| 03 | 5 | 4 | 4 | 4 | 4 | "each dot is one place" seeds R3 |
| 04 | 4 | 5 | 4 | 4 | 4 | |
| 05 | 5 | 5 | 5 | 4 | 4 | |
| 06 | 4 | 3 | 5 | 4 | 5 | 06_3 (R3) |
| 07 | 4 | 4 | 4 | 3 | 4 | dense recipe plus code panel; "÷ 80" as printed (R10) |
| 08 | 4 | 4 | 4 | 3 | 5 | R6, R8 |
| 09 | 4 | 5 | 4 | 4 | 4 | R15 is cosmetic |
| 10 | 3 | 3 | 5 | 3 | 4 | R1, R9; 10_4 carries five numbers |
| 11 | 5 | 4 | 5 | 4 | 5 | R14 (description only) |
| 12 | 4 | 4 | 4 | 4 | 4 | R13 |
| 13 | 3 | 3 | 5 | 3 | 5 | R2, R5, R7, R12 |
| 14 | 4 | 3 | 5 | 4 | 4 | R4 |

## Findings

Findings that would change the video are first. There is no blocker. Every fix below is a picture, lower-third or description change unless it names exact words.

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me | suggested fix |
|---|---|---|---|---|---|
| R1 | ruth_45 | 10_4 | major | "Twice the radius is four times the area, so each square mile gets a quarter of the track" is followed at once by 93, then 13. The "so" invites the viewer to apply the quarter to the 93, and that is false. In the model's own numbers (radius 1 + 0.45 h sea miles, 180 miles of track, 0.1 strip) hour 1 has radius 1.45, area 6.6, coverage 2.73 and hour 12 has radius 6.40, area 128.7, coverage 0.14. The radius grows x4.4, the area x19.5. A quarter of 2.73 gives 49%, which is the model at hour 4.2, where the radius has doubled. "Twice" is the illustration of a principle, and nothing on screen says the 12-hour circle is not twice the 1-hour one. | Picture only: put the model's own radii and areas on the two circles ("1 hour: radius 1.45, area 6.6 sq sea miles. 12 hours: radius 6.4, area 129. Radius x4.4, area x19, track per square mile 1/19"). Keep the "x 1/4" tag on the doubled pair as "for any doubling". The spoken line then stays true, because the numbers that follow are visibly a larger ratio. |
| R2 | ruth_45 | 13_6 (13_5) | major | "At that width, ten more passes with that sonar would have found nothing" is a counterfactual stated as certain. Its only support is a sweep width of zero (paper: "crude sensor sweep width estimates against the actual target") resting on one sonar pass that did not detect her (description and storyboard say so). One non-detection estimates a small width, not zero. "At that width" also has no antecedent for the sonar: the last width spoken in 13_5 is "about two hundred feet", at which more passes would find her. "Ten" has no fact key and is arbitrary under a width of zero. | Picture: in the hold at 13_6 keep the sonar's "0" lit with "sweep width 0 ft: the paper's estimate, from one pass that did not see her. Any number of passes x 0 = 0". Spoken, if a word may change: "And at that width" becomes "And on that estimate". The sentence then reads "And on that estimate, ten more passes with that sonar would have found nothing." |
| R3 | ruth_45 | 06_3 (06_2, 03_4) | major | 03_4 defines a dot as "one place the wreck could be". 06_2 and 06_3 then say twenty of B's thirty such places are ones where "we would have it". No place in B has that property: a wreck at any given place in B is seen two times in three. In a place-plus-luck simulation, P(seen given there) is 0.67 for every place and the posterior of B is 0.124, as it should be. The sentence is exact only if a dot is a place together with this search's luck (the description's simplification table says exactly that), or if the dots are the places the random strips of ch. 09 happened to cover. The lower-third "one equally likely way things could be" arrives in the hold after the sentence and says nothing about luck. A viewer who reads "place" literally concludes twenty particular places were cleared, and then sees the next pass catch two thirds of the remainder again. | Picture: at 06_2 and 06_3 the lower-third reads "a dot: a place and this search's luck. A fresh draw each search". At 06_4 label the ten still-lit B dots "wreck here, search missed it". If a word may move, 06_3 "were at any of those twenty" becomes "were any of those twenty ways". |
| R4 | ruth_45 | 14_4 | major | The brief (9a) says "it is never said that the method does not need the first map to be right". 14_4 opens "The first map does not have to be right." The next sentence rescues it only for a mathematician. Exactly: if every possible place has some dots and every search can see (d > 0), the rule returns to every such cell and finds a wreck that stays put with chance 1. The first map's quality sets only how long that takes. F6 held 0.85% and was the 24th cell, and the search took five months, while the video's premise is that time is short. The line also sits against the fourth sentence ("you need a new map when new evidence says a story was wrong"), and a viewer will ask which it is. | Picture, in the 14_4 hold under the "never adds" caption: "A wrong first map costs time, not the answer, if it gave the truth some chance. Zero stays zero." If re-narration is ever bought, "does not have to be right" becomes "does not have to be exact". |
| R5 | ruth_45 | 13_4, 13_5, 13_6 | minor | 800 ft down to 200 ft is a factor 4, but F6's coverage went 2.81 to 1.05 (factor 2.7). A fourfold narrower strip alone gives 50.5%, not 65. The 65 comes from sums over sensors and passes. The ship also credited the magnetometer with 400 ft (the paper's "typically 800 feet and 400 feet"), the measured widths are camera 185, magnetometer 220, sonar 0, and only part of the track was sonar. 13_4 draws only the sonar's 800 in red, and the storyboard note "a strip believed four times too wide becomes 94 percent" is not what happened. A viewer who divides gets another answer. | 13_4 shows both believed widths (sonar 800, magnetometer 400). 13_5 labels the curve marks "all sensors, all passes added: 2.8 to 1.05". Drop the "four times too wide" idea from the storyboard note. |
| R6 | ruth_45 | 08_5 | minor | "Less than one chance in a hundred" is true (0.76%, recomputed). It is the value of a row order that starts at the map's emptiest corner: the first twenty cells hold 114 of the 10,000 dots. A no-map plan has no preferred order, and its average over orders is 20 x (8,900/180)/10,000 x 2/3 = 6.6%, nine times the figure on screen. The conclusion survives (the never-updated map gives 44.3%), but the 0.8% flatters the contrast. | Chart: add a dashed mark "no map, any order, on average: 6.6%" and label the 0.8% "starting at the empty corner". The spoken line stays true. |
| R7 | ruth_45 | 13_1, 09_4, 10_4 | minor | Three symbols mean two things. L is the track length in Koopman's formula (09_4, 10_4: W, L, A) and the chance she is in the cell in the paper's update (13_1: L' = L(1-E) / (1-SEP)). A is the formula's area and also cell A, on the 09 plot that marks "1 in 4: the canyon". "Coverage" is a ratio that can reach 2.7 at 10_4 and a fraction swept (63%) at 09_3. The description repeats both letter clashes ("WL = A" and "L'j = Lj(...)"). | 13_1 and the description's update line use the video's own p and d, with the paper's letters named once in small type ("the paper's L and E"). The Koopman formula carries "width x length / area" in words, or writes the area as "area". Where "coverage" is a ratio, caption it "coverage (track x strip / area)". |
| R8 | ruth_45 | 08_3 (08_2) | minor | (a) The Chew lower-third "the least chance of still searching" garbles his P[N>n]: it is the chance of not yet having found it. (b) Three conditions are spoken. Two are unspoken: moving between cells costs nothing (only the description says so), and "fresh tries" leaves out "the same chance of seeing each time", without which slices need not shrink. (c) 08_2 answers "Yes." to an unconditional question twelve seconds before the conditions arrive. (d) "Ahead" has no stated measure except the axis. | Lower-third: "Chew 1967: for one thing that stays put, tries with the same chance of seeing each time, equal cost, nothing for travel: after every number of searches, the smallest chance of not having found it yet." The axis label becomes "ahead = more of the first hundred dots out, after the same number of searches". |
| R9 | ruth_45 | 10_3 | minor | "Spotting someone only inside a narrow strip" is a hard edge. A sweep width is the area under the lateral range curve: as many are seen outside it as are missed inside. The storyboard's own Ruth note says "an effective width, not a hard edge", and that reaches only the description. The lower-third says "sweep width (the width of the strip)". | Lower-third: "sweep width: an effective width, not a hard edge. As many seen outside it as missed inside". No spoken change. |
| R10 | ruth_45 | 07_2 | minor | The picture's "each product ÷ 80: 50, 12.5, 25, 12.5" is false as printed. 40 ÷ 80 is 0.5, and the x100 is hidden in the spoken "so the map adds up to a hundred again". A student checking the arithmetic finds a wrong line. | Label the column "÷ 80, as a percent", or print "x 100 ÷ 80" as 06_8 prints "100 dots ÷ 80 dots". |
| R11 | ruth_45 | 02_5, 02_6 | minor | (a) "The search had narrowed to a patch" is the map's lettered frame: 12 x 12.6 sea miles, with 1,100 of the 10,000 dots (11%) outside it. Nothing quoted says the Navy had fenced the search to it. (b) 4,594 uses 6,076.1 ft per sea mile. With the "6,076 feet each" printed on the same screen, 180 x 0.84 x 6,076 / 200 = 4,593.5, so a student gets 4,593. (c) The "about 3,000" two-sensor figure promised in the 02_6 annotation has no key among the fact sheet's 303. | (a) label at 02_5 "the map's lettered grid, 12 x 12.6 sea miles. 11% of its dots lie outside". (b) print "about 4,600". (c) add the two-sensor key to the oracle before the label is drawn. |
| R12 | ruth_45 | 13_4 (promise from 02) | minor | Storyboard ch. 02 says that at the 800 ft they believed, one pass looked affordable (1,148 sea miles, F-H.once.miles.believed) and "chapter 13 comes back to that". The rows of 13 do not. It is the best single number for why the ship's figures looked fine: the sled had towed 1,026, which is 89% of one pass at 800 ft. Without it, ch. 02's 4.5x is a hindsight number the Navy did not have, shown before the viewer learns that. | Picture at 13_4 next to "sonar 800 ft": "whole patch once at 800 ft: 1,148 sea miles. Towed: 1,026". |
| R13 | ruth_45 | 12_2, 12_3 | minor | 12_2 says "three underwater searches found nothing". 12_3 says the map was updated "for every empty search". The description says four empty searches, and the oracle's comment says the first of the paper's four is the surface search. A viewer who counts gets three while the description gets four. | Picture at 12_3: "4 empty searches: 1 on the surface, 3 under water", with the three year marks of 12_2 plus one mark for the surface search. |
| R14 | ruth_45 | description (S10); stale note in ch. 11 | minor | The description and fact-sheet finding 7 say "he had tied himself to lobster buoys". A grep for "buoy" through the repository surfaced a line of `docs/claims.md` quoting the Times as "sitting on the rope between his two buoys" and saying he tied a cut-loose buoy to his partner's. I could not read the article. If that line is right, "tied himself" is not the source's wording. The script's "clinging to two lobster buoys" agrees with it, but the storyboard's ch. 11 note still reads "clinging to a lobster buoy". | Description: replace with the article's own words ("tied two lobster buoys together and held on", if verified). Update the stale storyboard note. |
| R15 | ruth_45 | 09_3 | minor | The single run on screen ends at 66.5%, which is near the "two in three" that 09_4 reaches at 1.1 times the track. The viewer hears 63 and sees a run that already looks like two in three. In my 1,000 stdlib runs the sd was 1.7 points and the range 57.2 to 67.9, so 66.5 is a +1.8 sd run. The label "this run" helps, but the spread is not shown. | Show the band's min and max next to the mean ("1,000 runs: 57 to 68, mean 63"), or pick a run near the mean. |
| R16 | ruth_45 | 08_2, 08_3 | delight | The slices argument is the proof I would show students: a search takes a fixed slice of the original dots, a cell's slices only shrink, so the best n searches are the n biggest, which the rule takes in order. The brute-force line (65,536 orders) as a footnote to a proof, and "ahead" chosen because of ties, are exactly right. I recomputed it and it holds. | Keep. |
| R17 | ruth_45 | 06_7, 06_8 | delight | "Nobody searched it, and nothing in the sea has changed... What changed is what we know," then every unsearched cell grows by the same quarter. A frequentist-safe statement of what a posterior is, in a line a fourteen-year-old can follow. | Keep, and keep the Stone et al. quote under it. |
| R18 | ruth_45 | 13_1 | delight | The dot update turns, label by label, into the line the paper printed, with the same single denominator 1 - SEP that 07_2 named. The viewer has already done the computation by hand. | Keep (apply R7's letters). |
| R19 | ruth_45 | 09_3 | delight | A thousand runs of random strips drawn as a band, with one labelled "this run", teaches the difference between one outcome and an average better than any sentence. | Keep (add the spread, R15). |
| R20 | ruth_45 | 14_1 | delight | The opening shot replayed with a strip that sees nothing and no dot going out is the correct edge test of the chapter's own sentence, shown instead of stated. | Keep. |

## Answers to the producer's particular questions (a) to (f)

**(a) 08_3's optimality claim against Chew.** It is exactly right as written. The fact sheet's quote of Chew p. 494 is P[N > n | δ*] ≤ P[N > n | δ] for each n and every procedure δ. That is "no other order is ever ahead", where "ahead" means more of the first hundred dots out after the same number of searches. Adaptivity cannot matter, because every search before the find comes back empty.

I enumerated all 4^8 = 65,536 orders of the four-cell sea in exact fractions. None is ahead of the rule at any n from 1 to 8. Both tie-breaks (BCAABDAC and BCAADBAC) attain the best value at every n. Row by row is level with the rule at n = 3, 5 and 6 and behind elsewhere, so "ahead", not "better", is the correct word.

Each condition is necessary. I tested a counterexample for each:
- **Unequal cost** (C costs 4): at a budget of 3 the p·d rule has found 20%, and cheap-first B, A, D has found 36.7%.
- **Repeats that see nothing new:** after four searches the rule's order gives 43.3% against 50.0% for B, C, A, D.
- **A target that moves:** not tested here (qualitative); the description already declines the claim (Brown 1980).

Still unspoken are free travel and "same chance each time" (R8). The 1968 map has 11% of its dots outside the searched grid; the rule is then optimal among plans that search the grid.

**(b) 09_3 and 09_4, "on average sixty-three percent".** The random objects are the positions of the strips, independent and uniform. The quantity is the expected swept fraction, 1 - (1 - 1/100)^100 = 63.4% for the hundred strips shown, tending to 1 - e^-1 = 63.2% for thin strips. Both are "sixty-three". For a fixed point the same number is the chance it is covered, which is why it can be called "the chance of seeing it".

That requires the wreck's place to be independent of the strips, and the strips to be laid "without regard to what is already swept" (Koopman's assumption; the description says it, the narration has "if your strips land at random"). The two-in-three is therefore an average over where the strips fell. A searcher who knows the track would not update with a cell-level d, and "a fresh try" is the assumption that you ignore what you know. The Koopman lower-third "conservative side" covers the rest. The 1.10 and 0.29 recompute (66.7% and 25.2%). The single-run figure is R15.

**(c) 06_2 and 06_3, "if the wreck were at any of those twenty, we would have it".** See R3. It is exact for dots as (place, luck) pairs, and false for dots as places. The posterior is right either way (B = 10/80 = 1/8, recomputed, and checked by Monte Carlo), so only the sentence and its definition need the fix.

**(d) 13_2 and 13_6 against L' = L(1-E)/(1-SEP).** Everything reproduces:
- Coverage is -ln(0.06) = 2.813 and -ln(0.35) = 1.050.
- With the fact sheet's SEP (0.73, 0.45): 0.85% x 0.06 / 0.27 = 0.189% and 0.85% x 0.35 / 0.55 = 0.541%. These are the screen's 0.19% and 0.54%.
- "Ninety-four went out" is the numerator factor 1 - E. F6's share does not fall by 94%, because the whole map is then divided by 0.27, and the on-screen 0.85, 0.19, 0.54 shows that.
- "Sixty-five percent, not ninety-four" is exact (ratio 35/6 = 5.8).

What does not survive is "would have found nothing" and the missing antecedent (R2), and the 800-to-200 arithmetic (R5).

**(e) 10_4.** The model numbers recompute to 93.4, 64.6, 34.2 and 13.1, and the screen's coverages 2.7, 1.0, 0.4, 0.14 are 2.725, 1.037, 0.419, 0.140, so they are rounded consistently. "Twice the radius is four times the area" is true. The chain from it to 93 and 13 is the problem (R1).

The random-search formula is conservative for a planned parallel sweep. At 2.7 coverage a laid pattern would cover the whole circle, not 93%, and at 0.14 the two agree. The description's fitted-flight numbers (73% at hour 3) and its note that the random-search formula is conservative carry this. "Model" is on screen throughout.

**(f) 14_4, "does not have to be right".** See R4. It is true in the limit and false as advice for a search with a deadline.

## Audit log

Everything numeric was recomputed with `python3 -I`, standard library only (exact `fractions` where possible). I did not open the primary sources; every quotation, page number and attribution below is "checked against the fact sheet's text, not against the source" unless stated.

1. **Quantifiers explicit.**
   - I read all 95 script lines.
   - Always, every and never sentences found: 05_6 ("not always"), 06_8, 08_2, 08_3, 09_6, 14_1, 14_4.
   - Not explicit or not tested: 06_3 (R3), 13_6 (R2), 10_3 "only inside" (R9), 08_2 "Yes." (R8), and 07_5 "the whole method". The last is acceptable because 14_4 names what the loop does not do: the first map, the estimate of d, the new story.
2. **Always/never at the edges.**
   - 06_8: A, C, D each x 5/4 exactly. D has chance > 0 in this sea; with a cell of zero chance it would not rise (the description says so).
   - 08_2: slices 10, 7.5, 5.625 / 20, 6.67, 2.22 / 13.33, 4.44 / 6.67, 2.22 verified.
   - 08_3: see (a).
   - 09_6: 8/9 and 26/27 verified, and 27 dots give 18, 6, 2 with 1 left.
   - 14_1: d = 0 changes nothing, and so does p = 0 in the searched cell.
   - The fact sheet's edge table omits 06_3, 13_6 and 14_4's "does not have to be right".
3. **Probability spaces.**
   - 03_5, 03_6 and 06_x are degrees of belief, and 03_4 ("how much you believe") and 06_7 (subjective quote) say so.
   - 05_2 and 09_3 are chances over the search's own randomness: strips in 09, luck in 06.
   - 10_4 is the strips' placement with the person inside the circle.
   - 08_5 is the map's belief ("by the map's own reckoning"), checked. 13_2 and 13_6 are the ship's estimate of sweep widths.
4. **No probability on a fixed unknown.** None found in the narration beyond R3. 12_5's "77 in 100 it would have been heard" is a detection chance. The beacon sum 0.25 x 0.92 + 0.75 x 0.72 = 0.77 recomputed.
5. **Terms in their standard sense.**
   - Estimate versus estimator: the ship's numbers are "estimates", fine.
   - Sweep width is called a width, and its "effective" nature is R9.
   - "Coverage" and L, A, E are R7.
   - Field versus ring, FCFS and the like are not applicable here.
6. **Simplifications recorded.** The description's "What the video simplifies" table covers 05_6, 05_2, 06_3, 08_5, 10_3, 10_2, 11_1 and 14_6. The hook arithmetic of ch. 02 is exact only on screen (R11), and the 13_6 zero is in the description but not on screen (R2).
7. **Historical claims.**
   - Dates recomputed: 22 to 27 May is 5 days. 27 May to 5 June is 9. 10 June to 12 August is 63. 27 May to 28 October is 154 days (5 months). 12 August to 28 October is 77. Palomares 17 January to 15 March is 57 days. 1 June 2009 to 2 April 2011 is 670 days (22 months). 03:30 to 14:58 is 11 h 28 m. 22:30 to 03:30 is 5 h.
   - On-screen quotations (06_7, 07_1, 09_4, 12_5, 13_7, 04_6) match the fact sheet's typed passages character for character.
   - Unchecked against sources: Chew's Theorem 2.1 and the Blackwell attribution, Koopman eq. 40, the BEA phases and 216 + 12 aboard, Table H-15, CASP 1974 and SAROPS 2007, the NYT details (hatch, pumps, 3:30 and 2:58), Brown 1980, Kadane.
   - Depth: 02_3 quotes the Navy's "more than ten thousand feet" (NT-24-1 p. 7, per fact sheet). My unverified recollection is that other accounts say about 3,000 m (about 9,800 ft). The script is faithful to its stated source, so no finding.
8. **Numbers equal the oracle, voice equals screen.**
   - Four-cell sea: 40/30/20/10; 10, 20, 13⅓, 6⅔; 50, 12.5, 25, 12.5; 60, 15, 10, 15 (and 07_5's 240 → 200 → 170 dots); 20, 33, 43, 51, 58, 64, 70, 74 (57.5 rounded half up).
   - 1968 map: row and column sums equal the fact sheet's F-R.rows and F-R.columns. Total 10,000, 172 printed cells, E5 = 1/8 exactly, E5 + B7 = 23.46% (paper's "23 percent"), F6 = 85 with rank 24, E5/F6 = 14.7, 8,900 inside the grid. The model search gives 0.76, 44.35 and 49.77 at 20 searches and 32.9, 57.8 and 73.2 at 81. The first cells are E5, B7, C5, G5, E5, B7.
   - Hook: 4,593 or 4,594, 4,966, 4,176, 1,148, and 22% of the grid painted by 1,026 miles (R11).
   - Koopman: 63.21, 86.47, 95.02, 36.8 unswept.
   - Drift and F6 numbers: see (d) and (e).
   - Not reproduced: the seeded NumPy runs (66.5 single run, 63.4 band, the cloud's 2,000 dots, the fitted-flight 73%), because they need `PCG64DXSM`. I ran 1,000 stdlib bands instead (mean 63.39, sd 1.68).
   - The Figure 2 transcription was copied from `oracle.py`, so it is internally consistent, not independently read from the page image.
9. **Edge cases.** Ties (verified), d = 0 and d = 1 (description), zero-chance cell (14_4, description), E near 1 (coverage 4.6 at 0.99, not mentioned, fine). Ticket overflow and K versus K' are not applicable.
10. **Notation.** R7, plus the voice-only letters p and d (never spoken), which is consistent.
11. **True-but-misleading.** R1, R2, R4, R6, R8.
12. **Persona notes.** I checked every Maya, Dev and Ruth bullet in ch. 02 to 14.
    - Reached the narration, a picture, a lower-third or the description, except:
      - ch. 02's "chapter 13 comes back to that" (R12);
      - the stale ch. 11 note (R14);
      - Dev's "merge of four sorted lists" in ch. 08, which the description covers only through the slices argument, no reason given. This is immaterial.
    - Ruth's "effective width" reaches only the description (R9).
    - The brief's (9a) and 14_4 disagree in wording (R4).
    - The ch. 08 note says "most of that is simply having a map" was cut. The script keeps a short form, which is fine.

Incidental disclosure: a grep of `docs/` for "buoy", "thirty miles", "H-15" and "depth" surfaced lines of `docs/claims.md`. I used only:
- the Times buoy quotation, for R14;
- the Table H-15 row ("person in water 0.1 at 300 ft at every visibility"), which would make the description's "visibility" caveat unnecessary for that row;
- the combined-sensor arithmetic: 220 + 185 x (1 - 0.55) = 303.25 ft and 151.2 x 6,076 / 303.25 = 3,030 sea miles. That arithmetic reproduces, but the 0.55 factor comes from the paper's formula (1), which I have not seen, so "unchecked". The "about 3,000" has no fact key (R11).

I read no file under `docs/reviews/` other than the packet, and no other reviewer's output.
