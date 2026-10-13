# Claim walk — Video H (bayesian_search)

Walked 2026-10-13 against `script.tsv` (82 lines), `docs/storyboard.md` (as revised after R1), `docs/fact-sheet.md`, the oracle's output, and the primary sources under `<scratch>/sources/`. My working files are in `<scratch>/walk_scratch/v2/`. I did not use the older files in `walk_scratch/` itself.

## How I checked

- **Oracle.** I ran `oracle.py` (303 keys) and compared it key by key with the fact sheet's last table. All 303 values agree; the only differences are the spaces after commas.
- **My own arithmetic.** I wrote `walk_scratch/v2/walk_math.py` (python3 -I, exact `Fraction`s; output in `walk_math_out.txt`). It recomputes the following:
  - the four-cell sea: the stories, the 100 dots, both updates, the split into 300 dots, and the third and fourth choices;
  - the slices, and every one of the 65,536 orders of eight searches, by brute force. No order is ever strictly ahead of the rule. Row by row ties with it after 3, 5 and 6 searches;
  - the nine-dot quiz;
  - the hook's miles, including the paper's sensor-group formula (1);
  - the dates and day counts;
  - the 1968 model search, from the research notes' transcription of Figure 2 (180 lettered cells, 8,900 points inside the grid);
  - Koopman's curve;
  - the drift model's coverage and percentages;
  - F6 on 9 October by L′ = L(1 − E)/(1 − SEP);
  - the AF 447 beacon sum.
- **Primary sources, read directly:**
  - **Richardson and Stone 1971**, from the PDF text layer. I checked Figure 2 (p. 145) and Tables 1 to 3 (pp. 148 and 151) against the rendered page images.
  - **The Navy record:** *DANFS* and NT-24-1.
  - **Palomares:** DNA 1975 (OCR text), pp. 87, 94 and 125.
  - **Air France 447:** Stone et al. 2014 (text, plus Figures 7 and 8 as page images); the BEA final report, *Sea Search Operations* and the director's note.
  - **The Coast Guard:** Kratzke, Stone and Frost 2010; the Addendum's Table H-15; the 1996 *Theory of Search*.
  - **Aldridge:** the NYT Magazine article and the Coast Guard's 2013 releases.
  - **Theory:** Koopman 1946 (OCR text, pp. 28, 35 and 37); Chew 1967, p. 494 (page image); the ODNB entry on Bayes; Frost and Stone 2001; the ORMS Today profile of Stone.

## 1. Summary

| Count | |
|---|---|
| Rows walked | **129**: 82 script lines and 47 on-screen items that make a claim |
| OK | **112** (68 script lines, 44 on-screen items) |
| Spoken or shown numbers | All recomputed: **0 arithmetic errors**. Every number matches its fact, and voice and screen round the same way, except one said/shown rounding mismatch (07_4) |
| Always/never sentences tested | **27**. 25 hold cleanly. 1 holds, but its wording invites a wrong reading (08_3). **1 fails as worded** (13_6) |
| Said/shown mismatches | **2**: 02_6 (the voice says "whole search", the screen says "broad-area search track"); 07_4 (the voice says "thirteen", the screen shows 13⅓ ringed dots before the dots are split in three) |
| History and attribution | **1 unsupported or likely wrong** (02_6's comparator). **10 imprecise** (02_4, 02_5, 04_3, 11_1, 11_5, 11_6, 12_3, 13_5, 13_6, 14_2). **1 misattributed quotation** (12_5 lower-third). **1 imprecise lower-third** (04_4) |
| Tone and binding promise | **1 problem** (14_4, "say honestly"). Nothing implies the crew could have been saved. Nothing speculates on the cause. Nothing credits the map with more than the 1971 paper does |
| True but misleading | **1** (08_4/08_5: the rule against row by row) |

The history is in good shape. The hard claims all check against the primary text:
- the 260 yards;
- 94 against 65 percent on 9 October;
- "had not seen her at all";
- the metal photographed that summer;
- found on 28 October;
- Palomares and the fisherman;
- AF 447's 228 dead, the 40 NM circle, the three searches, the 35/35/30 weights, the 31-day listening, the two maps, both dark near the centre, about a week, and the wreck in the passive-searched area;
- CASP in 1974;
- Aldridge: 3:30 a.m., the 9:30 to 11:30 p.m. first guess, about 30 miles, the hatch and the pumps, the helicopter, almost 12 hours.

The problems below are about comparators, attribution and wording.

## 2. Problems, most serious first

**1. 02_6: the comparison in the hook is against a figure the paper explicitly limits.**
- **What is wrong.** "That is more than four times what the sled covered in the whole search." The 1,026 miles is not the whole search. The paper says (p. 146): "1,026 miles of track were used for broad-area search (not including contact investigation)." And (p. 149): the 1,026 includes "the research and development effort up to detection and excluding the identification effort around M8/3".
- **Why it matters.** The M8/3 investigation alone ran from 14 to 26 August, runs 29 to 52 (p. 147). The claim survives only if all excluded towing came to under 4,594/4 − 1,026 ≈ 122 miles. Twelve days at the paper's own 24 miles a day gives up to about 288. As worded, the claim is unsupported and probably false.
- **Said against shown.** The screen label is right ("broad-area search track, June to October: 1,026"). The voice contradicts it. The storyboard's own note says "broad-area search up to detection", but its misconception line says "what the whole search towed".
- **Second, smaller point (02_5, "It saw a strip about two hundred feet wide").** This is one sensor's width. The paper combines sensors towed together with its formula (1). For the camera and magnetometer together, that gives W = 220 + 185 × (1 − 0.55) ≈ 303 ft. One pass would then be about 3,030 miles, about 3.0 × 1,026. Across sensors the multiple runs from 2.95 (both together) to 4.84 (camera alone).
- **Fix (keeps the length).** "One pass over the whole patch meant forty-six hundred miles of towing. That is more than four times all the sled's open searching before the find." If the sensor-group point is to be met too: "…meant three to five thousand miles of towing. That is three to five times all the sled's open searching before the find."
- **Also fix** the storyboard note under 02: "four and a half times what the whole search towed" → "…what the broad-area search towed".

**2. 14_4: "You have to say honestly how well you looked." The tone implies the 1968 analysts were dishonest.**
- **What is wrong.** This comes straight after 14_3 ("The strip was believed too wide"), so it reads as a charge that the men at sea were dishonest. The storyboard's own vocabulary bans "honest figure" for exactly this reason: "the men at sea were optimistic, not dishonest".
- **What the source says.** The paper blames "the optimistic sensor (particularly sonar) sweep width estimates then available to the analysts" (p. 148).
- **Fix.** "You have to judge how well you really looked." (or "You have to know truly how well you looked.")

**3. 13_6 (with 13_5): "the true figure", and "ten more passes with that sonar would have changed nothing". This overstates what the after-search numbers are.**
- **What the source says.** The paper (p. 151) calls them "crude sensor sweep width estimates against the actual target". It adds: "We emphasize that these estimates apply to *Scorpion* as it was actually found (i.e., partially buried)". The sonar's zero rests on one observation (Table 2, note a): "based on the failure of one of the two sonars to detect *Scorpion*. No data are available pertaining to the performance of the other sonar".
- **The edge fails as worded.** "Would have changed nothing" holds only if the sonar's width is exactly 0. It is also ambiguous: ten more passes *would* have changed the ship's numbers, pushing on-scene LEP in F6 from 0.94 toward 0.99 and F6 further down the map.
- **Fixes.**
  - 13_6: "So the better estimate for that cell was sixty-five percent, not ninety-four. And at that width, ten more passes with that sonar would have found nothing."
  - 13_5: "Estimated afterwards against the wreck itself, …"
  - Screen at 13_5: "measured afterwards" → "estimated afterwards".

**4. 08_4/08_5: true but misleading. The gap is mostly the map, not the rule.**
- **What is wrong.** "On the Navy's map the difference is large … One plan has no map, and goes row by row. The other follows the rule." The voice then contrasts 0.8 percent with 49.8 percent. My recomputation from the Figure 2 transcription, with every search seeing 2 in 3:
  - row by row, 20 searches: 0.76%;
  - each cell once, likeliest first (the map never updated): 44.35%;
  - the rule: 49.77% (at 81 searches: 32.85 / 57.76 / 73.19).
- **Why row by row looks so bad.** Row by row starts at the empty north edge: its first 20 cells hold 114 of 10,000 points. A no-map plan that picks 20 cells at random averages 6.6%.
- **Effect.** The voice credits "the rule" with a gap that is about nine-tenths the map. The screen's third curve shows this, but the line that said it was cut.
- **Fix.** 08_5: "Twenty by the rule, about one in two, and most of that is the map itself." Or 08_4's first sentence: "On the Navy's map, most of the gain is having the map."

**5. 12_5 lower-third: a paraphrase labelled as the authors' words.**
- **What is wrong.** On screen: "the fuller way, in the authors' words: one map, with a weight on whether the beacons worked". Those are not the authors' words.
- **What the source says.** Stone et al. 2014 (p. 78): "A better way to handle the doubts that we had about the beacons would have been to compute a joint distribution on beacon failure and wreck location."
- **Fix.** Quote it: "the fuller way, in the authors' words: 'a joint distribution on beacon failure and wreck location'". Or drop "in the authors' words".

**6. 07_4 (with 07_5): said is not shown, and the picture breaks the storyboard's whole-dot rule.**
- **What is wrong.** The voice says "a search of C thirteen". The picture rings "13⅓ in C, 6⅔ in B and in D". A third of a dot cannot be ringed, and the storyboard puts the split into three ("more dots, same map") at 07_5, after these rings. The split itself is never said.
- **Fix.** Split the dots at the start of 07_4, before the rings (40 → 120 and so on: 30 in A, 40 in C, 20 in B and in D). Say "…and a search of C thirteen and a third", or "…a search of C more than thirteen".

**7. 14_2: "Four searches, each against its map. Palomares, …" No source says the bomb was found against a probability map.**
- **What the sources say.**
  - DNA 1975 (p. 87): the area that held the bomb was chosen "primarily" from sonar contacts "accorded high credibility because of the statements and plotted evidence from the Spanish fisherman".
  - DNA 1975 (p. 94): the find was "in the vicinity of the place where Simo' Orts reported seeing a weapon descend".
  - McGrayne, as quoted in the research notes: the prior and the updates "were never combined in time to be of any use".
- **The picture.** The panel itself compares the find with the fisherman's point, not with a map.
- **Fix.** "Four searches, and where each one ended. Palomares, …"

**8. 04_3: "A mathematician, Henry Richardson". No source read calls him one.**
- **What the sources say.** RS71 says only "one of the authors" of Daniel H. Wagner, Associates. ORMS 2017: "Richardson was assigned the task of using these scenarios to produce a probability map". DNA 1975 (p. 125) lists an unnamed "civilian mathematician" in the Palomares Tactical Analysis Group.
- **Fix.** "An analyst, Henry Richardson, had a computer play those stories out ten thousand times." Or keep "mathematician" only if a source naming him as one is added to the fact sheet.

**9. 11_6: "clinging to a lobster buoy". When found, he was on two.**
- **What the source says.** NYT: "there was John Aldridge, sitting on the rope between his two buoys". He had tied a cut-loose buoy to Pete Spong's.
- **Fix.** "He was alive, clinging to two lobster buoys, after almost twelve hours in the water."

**10. 11_5: "That was work Aldridge would not have started until the boat was far offshore." The source says "usually".**
- **What the source says.** NYT: "Aldridge and Sosinski would usually wait to start filling the tanks until their boat reached the 40-fathom curve", "about 25 miles offshore".
- **Fix.** "That was work Aldridge usually left until the boat was far offshore."

**11. 12_3: "built a map from three smaller maps". The three are not smaller.**
- **What the source says.** Stone et al. (p. 72): "The distribution D1 is uniform within the 40 NM circle". D2 is a circular normal truncated at 40 NM. D3 is reverse drift. Each covers the circle.
- **Fix.** "Then a team of search analysts built a map from three separate maps."

**12. 02_4: "Nobody knew that yet." This is slightly too strong.**
- **What the sources say.** Bradley (*Congressional Record*, secondary): by 28 May the commander and "others in the service's command believed the *Scorpion* had been destroyed". Offley (secondary): by 30 May the acoustic analysis had fixed a 12-by-12-mile area. Certainty came only with the find.
- **Fix.** "Nobody could be sure of that yet."

**13. 11_1: "has had software that searches this way". The software plans searches; it does not search.**
- **What the source says.** Kratzke 2010: CASP "used a Bayesian approach implemented by a particle filter". NYT: the controller drew the last track by hand.
- **Fix.** "…has had software that plans searches this way since nineteen seventy-four."

**14. 04_4 lower-third: "10,000 runs of what two men believed, not 10,000 sinkings". This is reductive.**
- **What the source says.** The credences "incorporated (at least indirectly as interpreted by Craven and Andrews) the views and opinions of Navy operating personnel as well as the analysis of specialists in many scientific areas" (p. 144). Each run also drew course, speed and position (pp. 144-45).
- **Fix.** "10,000 runs of nine weighted stories, not 10,000 sinkings".

**15. 08_3: "no other order is ever ahead". The edge holds, but "ahead" is by the map's own count.**
- **What holds.** Chew 1967 (p. 494) proves it for a fixed object, fixed overlook probabilities 0 < α < 1, independent looks and unit costs. My brute force over all 65,536 orders agrees.
- **The gap.** A fourth condition goes unspoken: the chance of seeing must be *known* (chapter 13 is the counter-case). And "ahead" means ahead in the map's own chance. For a particular wreck, another order can find it sooner: in 1968, a plan aimed at F6 would have.
- **Why it is low.** 08_5 says "By the map's own reckoning" two lines later.
- **Optional fix.** "…no other order is ever ahead, by the map's own count."

**16. Storyboard notes and the fact sheet. These are not spoken, but they feed the description.**
- **Chapter 13 note:** "a strip believed four times too wide becomes 94 percent". The tables imply coverage of 2.81 against 1.05, a ratio of 2.7, not 4. A four-times coverage on 1.05 would give 98.5%, not 94%. Fix: "a strip believed far too wide (coverage 2.8 against 1.05)…".
- **Chapter 13 note:** "it made no difference: the order of the evidence does not matter". That is true of the computed posterior. But the paper adds (p. 149): "This does not mean that time lost by non-optimal procedures at the beginning of the search can be recovered fully". And (p. 152): "Unfortunately, the a priori distribution was not available on scene until August 12." The description must not imply that the late map cost nothing.
- **Fact sheet "Terms" table:** its "As spoken" column is stale ("Experts weigh the two stories", "a map of how likely each place is", "a strip only a tenth of a mile wide"). The fact sheet's "Simplification → exact" table quotes 10_3 as "a strip only a tenth of a mile wide"; the script now says "It is a tenth of a mile wide."

**17. Minor, optional.**
- **12_2: "In nearly two years, three underwater searches found nothing."** The three ran from June 2009 to May 2010; 22 months is the time to the find (BEA: "found on 2 April 2011, 22 months after disappearing"). Optional: "In the year that followed, three underwater searches found nothing." The screen keeps "22 months".
- **13_1: "Its map reached the search ship in August."** The paper says it was "hand-carried at the earliest date to the scene of operations arriving August 12". Mizar was between cruises 2 (ended 6 August) and 3 (began 14 August). Optional: "Its map reached the search in August".

## 3. The full table

Verdicts:
- **OK**: the source or the arithmetic says it.
- **OK (model)**: the video's own stated assumption, recomputed.
- **IMPRECISE**: close; the difference is stated.
- **UNSUPPORTED**: no source text found.
- **WRONG**: contradicted by the source.
- **MISMATCH**: said and shown disagree.
- **EDGE HOLDS / EDGE FAILS**: an always or never sentence, tested at its edges.
- **MISLEADING**: true but misleading.
- **TONE**: breaks the tone rules or the binding promise.

Page numbers refer to the source named in the row.

### Script lines

| line | claim | verdict | evidence |
|---|---|---|---|
| 01_1 | Premise: a sub missing, men trapped, time running out | OK (premise) | Set in `dim` as a premise. 02_1 to 02_3 say what was true of *Scorpion* within about 15 s. Does not imply her crew could have been saved |
| 01_2 | A search that finds nothing has still told you something | EDGE HOLDS | The search shown can see: most of the cell's dots go out. The general edge (a blind search, d = 0) is 14_1's |
| 02_1 | May 1968; USS *Scorpion*, nuclear sub, heading home across the Atlantic, 99 aboard | OK | *DANFS*: "complement 99"; "came about and headed west for home"; "On 21 May, she indicated her position to be about 50 miles south of the Azores." |
| 02_2 | Due at Norfolk on the 27th, did not arrive; nine days of search | OK | *DANFS*: "Six days later [after 21 May], *Scorpion* was reported overdue at Norfolk … on 5 June, she was declared 'presumed lost'". 27 May to 5 June = 9 days (my dates) |
| 02_3 | Sank five days before due, in more than 10,000 ft; all 99 lost | OK | NT-24-1 p. 7: "The submarine SCORPION sank on 22 May 1968, 400 miles southwest of the Azores in more than 10,000 feet of water." 22 to 27 May = 5 days. *DANFS*: "presumed lost with all hands" |
| 02_4 | Nobody knew that yet; where do you look? | IMPRECISE | Problem 12. Command "believed" she was destroyed by 28 May (Bradley, secondary). Certainty came only with the find |
| 02_5 | Narrowed to a patch about 12 miles on a side; sled towed just above the floor; strip about 200 ft | IMPRECISE | Grid 12 rows × 1 mile by 15 columns × 0.84 mile = 12 × 12.6 (RS71 p. 143). Sled: "a towed platform (called the sled)", with "bottom time" (pp. 142, 149). 200 ft is one sensor's after-search width (Table 2: 185 and 220). The paper's formula (1) for camera and magnetometer together gives ≈303 ft (problem 1) |
| 02_6 | One pass = 4,600 miles; more than 4 × what the sled covered in the whole search | UNSUPPORTED; MISMATCH with the screen | 4,594 miles at 200 ft (mine), so "forty-six hundred" is right. But 1,026 is broad-area search only, "not including contact investigation" (p. 146), "up to detection and excluding the identification effort around M8/3" (p. 149). Problem 1 |
| 02_7 | Where would you look first? | OK | Question |
| 03_1 | Natural answer: the likeliest place; you need a map | OK | Framing |
| 03_2 | Made-up sea, four cells, two stories | OK (model) | |
| 03_3 | Story 1: A most likely, then B, then C; story 2: anywhere | OK (model) | Shares ½, ⅓, ⅙, 0 and ¼ each |
| 03_4 | Weights 60/40; 100 dots, 60 and 40 | OK (model) | |
| 03_5 | Story 1 puts half in A; story 2 a quarter | OK (model) | |
| 03_6 | 30 + 10 = 40 dots, 40% | OK | Exact: 60 × ½ + 40 × ¼ = 40 |
| 03_7 | Map 40, 30, 20, 10 | OK | Exact: [40, 30, 20, 10], sum 100 |
| 04_1 | Map made the same way; Craven, a Navy scientist, first tried the idea two years earlier, at Palomares | OK | RS71 p. 144: "Craven had earlier originated this approach in the H-bomb search." DNA 1975 p. 81: "Doctor John P. Craven, Chief Scientist, Deep Submergence Systems Project". 1966 → 1968 |
| 04_2 | Bomb found near where a fisherman watched it fall; July 1968, Craven and Andrews set nine stories, each weighted | OK | DNA 1975 p. 94: found "in the vicinity of the place where Simo' Orts reported seeing a weapon descend". RS71 p. 143: "a conference was held July 18 where Dr. J. P. Craven and Dr. F. A. Andrews postulated nine scenarios … and assigned credences to them." |
| 04_3 | A mathematician, Henry Richardson, had a computer play the stories 10,000 times | IMPRECISE | Monte Carlo and 10,000: RS71 pp. 144-45 ("The result of 10,000 replications"). Richardson made the map: ORMS 2017. "Mathematician" is unsourced (problem 8) |
| 04_4 | He counted the dots per cell; the printed map | OK | RS71 p. 145: "the output of the replication is a 'one' added to the appropriate search cell". Figure 2 checked on the page image |
| 04_5 | One cell stands out at 1 in 8; E5 | OK | Figure 2: E5 = 1,250 (page image). 1,250/10,000 = ⅛. RS71 p. 145: "distinctly the highest a priori probability" |
| 04_6 | Wreck in a neighbouring cell, within 260 yards of E5's edge | OK | RS71 p. 145: "*Scorpion*'s location is in rectangle F6 within 260 yards of the edge of rectangle E5". Asterisk in F6, upper left, beside E5 (page image) |
| 04_7 | The map alone did not find her; the search took five months | OK | 27 May to 28 October = 154 days. RS71 p. 148: the find came "because of proximity to the piece of metal" as well as the prior |
| 05_1 | 40% says search A first; a search is not a sure thing | OK (model) | |
| 05_2 | Canyon A seen 1 in 4; flat cells 2 in 3 | OK (model) | |
| 05_3 | How likely is one search to find it? | OK | Question |
| 05_4 | Quiz: A or B | OK (model) | |
| 05_5 | B: ¼ × 40 = 10, ⅔ × 30 = 20, twice | OK | Exact: 10, 20, 13⅓, 6⅔; B/A = 2 |
| 05_6 | Find = chance there × chance of seeing; best cell not always the likeliest | OK; EDGE HOLDS | USCG 1996 p. 2-4: "POS = POC x POD". "Not always" is shown by B beating A |
| 06_1 | Search B, empty | OK (model) | |
| 06_2 | 30 in B, ⅔ caught: 20 | OK | Exact |
| 06_3 | Those 20 go out | OK (model) | |
| 06_4 | 80 left; 10 still in B | OK | Exact |
| 06_5 | Quiz: B now? | OK | |
| 06_6 | 1 in 8, 12.5%, down from 30 | OK | 10/80 = ⅛ |
| 06_7 | A unsearched, nothing changed in the sea, now 40/80 = ½; what changed is what we know | OK | Exact. Lower-third quotation verified (row S14) |
| 06_8 | Every unsearched cell grew by a quarter: 40→50, 20→25, 10→12.5 | OK; EDGE HOLDS | Factor 100/80 = 5/4 for every unsearched cell (all nonzero) |
| 07_1 | Bayes' rule; Thomas Bayes, English minister, died 1761 | OK | ODNB: "Bayes, Thomas (1701?–1761), mathematician and Presbyterian minister"; "He died on 7 April 1761"; probably born in Hertfordshire |
| 07_2 | Multiply by the chance of an empty search (⅓ for B, 1 elsewhere), divide by 80 | OK | 40 + 10 + 20 + 10 = 80. Gives 50, 12.5, 25, 12.5. The "1 for every other cell" edge holds: no false alarms |
| 07_3 | A has half the chance; A is the canyon; where next? | OK | |
| 07_4 | C; in dots nothing changed for A or C; A 10, C "thirteen"; only B fell | MISMATCH; EDGE HOLDS | C's catch is 40/3 = 13⅓ in original dots (mine), and the screen shows 13⅓. Fractional rings come before the split at 07_5 (problem 6). "Only B has fallen" holds: A 10, C 13⅓, D 6⅔ unchanged; B 20 → 6⅔ |
| 07_5 | Search, empty, update, choose again; that loop is the whole method | OK | After C empties: 60, 15, 10, 15. Third choice A, 30 split dots out. Fourth choice still A (13.2 against 11.8). "Whole method" is rhetoric; 14_4 adds the first map and how well you looked |
| 08_1 | Always search where the next search is most likely to find it? | OK | Question. Screen order BCAABDAC and curve 20, 33, 43, 51, 58, 64, 70, 74, all reproduced |
| 08_2 | Each search of a cell puts out a fixed slice of the first 100 dots, whatever came between; a cell's slices only shrink | EDGE HOLDS | Slice k = p·d·(1 − d)^(k−1), independent of other cells. Strictly shrinking for 0 < d < 1 (weakly for d = 1). Conditions spoken at 08_3 |
| 08_3 | Best 8 = 8 biggest slices, the rule takes them biggest first; under three conditions no order is ever ahead | EDGE HOLDS (wording risk) | Brute force: 0 of 65,536 orders strictly ahead at any n ≤ 8. Ties: row by row at n = 3, 5, 6. Chew p. 494: "strongly optimal … P[N > n \| δ*] ≤ P[N > n \| δ]" for c_i = 1, 0 < α_i < 1, independent outcomes. Problem 15 |
| 08_4 | On the Navy's map the difference is large; row by row against the rule | MISLEADING | Problem 4: the map alone gets 44.3%; random no-map cells average 6.6%; row by row starts in the emptiest rows |
| 08_5 | By the map's reckoning: 20 row by row < 1 in 100; 20 by the rule ≈ 1 in 2 | OK | Mine: 0.76% (0.85% if normalized to the 8,900 in-grid points) and 49.77%. First cells E5, B7, C5, G5, E5, B7. One search at d = ⅔ = 28.0 miles at 200 ft |
| 09_1 | WWII, hunting submarines, Koopman and colleagues asked how much area a searcher covers | OK | Morse's foreword to OEG 56: work "during the period 1942-1945 by members of the Anti-Submarine Warfare Operations Research Group". Frost and Stone 2001 p. 2-1: "Bernard Koopman joined ASWORG in 1943 … Koopman, James Dobbie, and a few others" |
| 09_2 | Perfect strips side by side cover all of a cell with one cell's worth of track | OK; EDGE HOLDS | Definite-range strips tiling the cell |
| 09_3 | Strips at random overlap and leave gaps; on average 63% | OK | Koopman p. 28, eq. (40): "the formula of random search p = 1 − e^(−WL/A)" with "different (not too near) portions placed independently". 1 − e^(−1) = 63.2%; 100 one-percent strips give 1 − 0.99^100 = 63.4% |
| 09_4 | Koopman's formula gives the curve; a little more track → 2 in 3 | OK | −ln(⅓) = 1.099 ("1.1"); −ln(¾) = 0.288 ("0.29") |
| 09_5 | Two passes over a cell; first catches 6 of 9; second, a fresh try, ⅔ of what is left | OK | |
| 09_6 | 8 of 9; 1 left; third pass leaves 1 in 27; each pass finds less; no number of passes makes it certain | OK; EDGE HOLDS | 9 × (1 − (⅓)²) = 8; (⅓)³ = 1/27. Holds for 0 < d < 1 with fresh tries, both spoken. The edges, d = 1 and passes that are not fresh tries, are named in the notes |
| 10_1 | A wreck stays put; a person drifts with wind and current, neither known exactly | OK | Kratzke 2010 sec. 2.5: "two forces acting on a drifting particle; currents and winds" |
| 10_2 | Each dot moves with its own guess; the map drifts and spreads | OK (model) | Kratzke 2010: "each particle represents a possible path" |
| 10_3 | From a helicopter, the Coast Guard counts on a strip a tenth of a mile wide for a person in the water | OK | Table H-15, "Uncorrected Visual Sweep Width – Helicopters for Altitudes 300-500 Feet": Person in Water 0.1 at 300 ft at every visibility. It is an effective width, not a hard edge; the fact sheet says so |
| 10_4 | Twice the radius = 4 × area = ¼ the track per square mile; model 93 in 100 at 1 h, 13 in 100 at 12 h | OK (model) | Radii 1.45, 2.35, 3.70, 6.40 → coverage 2.73, 1.04, 0.42, 0.14 → 93.4, 64.6, 34.2, 13.1% (mine) |
| 11_1 | Coast Guard has had software that searches this way since 1974; thousands of dots; Bayes after every empty search | IMPRECISE (minor) | Kratzke 2010: "In 1974 the U.S. Coast Guard put into operation … CASP … which used a Bayesian approach implemented by a particle filter"; "2,500, 5,000, or 10,000 particles per scenario". The software plans searches; it does not search (problem 13) |
| 11_2 | July 2013, lobsterman John Aldridge fell off his boat at night, about 40 miles off Long Island; nobody saw | OK | NYT: "alone on the deck of a lobster boat in the middle of the night, 40 miles off the tip of Long Island"; "about 3:30 a.m. on July 24"; Sosinski "asleep on a bunk below". (The NYT later places the fall "a few miles south of the 40-fathom curve", about 25 miles out; it is not consistent with itself) |
| 11_3 | Searchers guessed the evening before; the first map almost 30 miles too far north | OK | NYT: "between 9:30 p.m. and 11:30 p.m."; "Almost 30 miles south of where the Jayhawk crew was carefully searching for him" |
| 11_4 | The map was wrong because its story was wrong | OK | NYT: "no clear idea when — and therefore where — he fell overboard" |
| 11_5 | Partner found an open hatch and pumps running; work not started until far offshore; he fell hours later | IMPRECISE (minor) | NYT: "One hatch cover was upside down on the deck"; "The pumps were on"; they "would usually wait to start filling the tanks until … the 40-fathom curve" (problem 10) |
| 11_6 | Map re-run, dots moved south; that afternoon a helicopter flying through them spotted him; alive, clinging to a lobster buoy, almost 12 hours | IMPRECISE (minor) | NYT: "At 8:30 a.m. … most of the dark-red search particles had migrated south of the 40-fathom curve". Hand-drawn track "straight through the main search area", at 2:46 p.m. plus 12 minutes. "sitting on the rope between his two buoys" (problem 9). USCG Dec. 2013: "recovered approximately 12 hours later"; 3:30 to 2:58 = 11 h 28 m |
| 12_1 | June 2009, AF 447 Rio to Paris went down in the Atlantic; all 228 killed | OK | BEA final 1.1: "Twelve crew members … and 216 passengers were on board". Table 1.2: Fatal 12 + 216 |
| 12_2 | Wreck had to lie inside a 40 NM circle; in nearly two years three underwater searches found nothing | OK (minor note) | Stone et al. 2014: "the wreckage had to lie within 40 NM of the plane's last known position". Phases 1 to 3 (BEA) all failed. Timing note in problem 17 |
| 12_3 | Analysts built a map from three smaller maps, weighted 35/35/30; updated for every empty search | IMPRECISE (minor) | p. 72: "p1 = p2 = 0.35 and p3 = 0.3"; p. 75: "computed in four steps with each step accounting for an increment of unsuccessful search". "Smaller" (problem 11) |
| 12_4 | For a month ships listened for the black boxes' beacons and heard nothing | OK | p. 69: "The search began on June 10, 2009 and lasted 31 days … the beacons were not detected." |
| 12_5 | If beacons worked, the silence counts a great deal; if broken, not at all; map drawn twice | OK | p. 78: "doubts about the beacons compelled us to produce the alternate posterior shown in Figure 8, which assumes the beacons did not function". The main map used 0.77 |
| 12_6 | Both maps weighted near the centre; resumed there, found after about a week; in a high-chance part of the second map, in water the listening ships had crossed | OK | Figures 7 and 8 (page images): darkest near the centre. BEA director: "a strong possibility for discovery of the wreckage near the centre of the Circle. It was in this area that it was in fact discovered after one week of exploration". p. 78: "falls in a high probability area". p. 80: "located in an area thoroughly covered by the passive search" |
| 13_1 | Navy faced the same question in 1968; map reached the search ship in August, two months in; every pass since June scored against it | OK (minor note) | RS71 p. 149: "hand-carried at the earliest date to the scene of operations arriving August 12"; "not available on-scene for the first two months". Table 1: on-scene SEP for runs 1-63. "Search ship" (problem 17) |
| 13_2 | By October the ship's numbers said one cell 94% searched; 94 of every 100 dots out; it was the wreck's cell | OK | Table 1 (page image): on-scene LEP in F6 0.65 (Sept. 10), 0.94 (Oct. 9), 0.99 (Oct. 28) |
| 13_3 | The wreck was there and the sled had been over it | OK | Implied by LEP 0.94; p. 145: "large amounts placed in neighboring cells, including F6" |
| 13_4 | At sea they believed the sonar saw an 800 ft strip | OK | p. 151: "The latter estimates were typically 800 feet and 400 feet for sonar and magnetometer" |
| 13_5 | Measured afterwards against the wreck, the sonar that passed her had not seen her at all; camera and magnetometer about 200 ft | IMPRECISE | Table 2 (page image): camera 185, magnetometer 220, sonar 0, "based on the failure of one of the two sonars to detect *Scorpion*". p. 151: "crude sensor sweep width estimates" (problem 3) |
| 13_6 | The true figure was 65%, not 94; ten more passes with that sonar would have changed nothing | IMPRECISE; EDGE FAILS AS WORDED | Table 3 (page image): after-search LEP in F6 0.65 on Oct. 9. "Crude" estimates; the sonar zero rests on one sonar's failure (problem 3) |
| 13_7 | Found at the end of October; sled went back to where metal was photographed that summer, beside the map's top cell | OK | p. 142: "*Scorpion* was located on October 28". p. 147: "A shiny, bent piece of metal was photographed, near the border of F5 and F6", between June 27 and July 23. p. 148: effort went to F6 "because of proximity to the piece of metal and the fact that this was a region with high a priori probability". *DANFS*: "at the end of October" |
| 14_1 | An empty search only tells you something if it could have found the thing | EDGE HOLDS | With d = 0 the update is ×1 everywhere: the map is unchanged |
| 14_2 | Four searches, each against its map; Palomares near the fisherman; *Scorpion* beside the top cell; AF 447 near the centre; Aldridge inside the dots once the story was put right | IMPRECISE | The *Scorpion*, AF 447 and Aldridge parts are OK, as above. Palomares had no map that the find is measured against (problem 7) |
| 14_3 | Strip believed too wide; the silence might have meant nothing; the story five hours off | OK | 800 against 185/220/0. Stone et al.: "suggest that both beacons failed". NYT: "did not fall in the water at 10:30 p.m.; he fell in at 3:30 a.m." |
| 14_4 | The first map need not be right but must leave room; say honestly how well you looked; new map on new evidence | TONE; EDGES HOLD | Zero stays zero; with room left and true seeing chances the loop eventually reaches the truth. "Honestly" (problem 2) |
| 14_5 | The 99 could not be saved | OK | Sank 22 May in more than 10,000 ft (NT-24-1); search began 27 May |
| 14_6 | The method that searched for them now searches for people who can | OK | ORMS 2017: Richardson "convinced the U.S. Coast Guard to contract with Wagner Associates to develop a search planning system (Computer Assisted Search Program)"; "If a search failed, the system computed the posterior probability map". Kratzke 2010: CASP 1974 → SAROPS |

### On-screen items that make claims

| item | claim | verdict | evidence |
|---|---|---|---|
| S1 02_1 | "USS Scorpion (SSN-589), May 1968" | OK | *DANFS* |
| S2 02_2 | 27 May "due"; nine marks to 5 June; "aircraft and dozens of ships" | OK | *DANFS*: "dozens of ships and submarines" |
| S3 02_3 | Back to 22 May; "99 men" | OK | NT-24-1 p. 7 |
| S4 02_4 | "about 400 miles southwest of the Azores" | OK | NT-24-1; *DANFS*; RS71 p. 142 |
| S5 02_5 | 12 × 15 grid; "camera 185 ft, magnetometer 220 ft"; "one cell: 26 sea miles of track (model: perfect strips)" | OK | Table 2. 0.84 mi × 6,076/200 = 25.5 → 26 |
| S6 02_6 | "all 180 cells: 4,594 sea miles"; "broad-area search track, June to October: 1,026"; 40 cells painted | OK | 4,593.6 (mine). 1,026/25.5 = 40.2. The label agrees with p. 146; the voice does not (problem 1) |
| S7 04_1 | "Palomares, Spain, 17 January 1966" | OK | DNA 1975 p. 15 |
| S8 04_2 | "found by the submersible Alvin, 15 March 1966"; "18 July 1968: nine stories … Not printed in the 1971 paper" | OK | DNA 1975 p. 94: "on 15 March ALVIN successfully backed down the furrow"; RS71 p. 143 |
| S9 04_3 | "each run drew its own course, speed and position" | OK | RS71 p. 144 |
| S10 04_4 | "Figure 2. Each cell about a mile across"; lower-third "10,000 runs of what two men believed, not 10,000 sinkings" | IMPRECISE (lower-third) | Cells 1 × 0.84 mile (p. 143). Lower-third: problem 14 |
| S11 04_5 | "1,250 of 10,000: 1 in 8"; B7 nearly as hot | OK | Figure 2: B7 = 1,096; "E5 and B7, have 23 percent" (p. 145) |
| S12 04_6 | "85 of 10,000: fewer than 1 in 100"; quoted "within 260 yards of the edge of rectangle E5" | OK | Figure 2 page image; p. 145 verbatim. F6 is 24th of 172 (23 cells hold more than 85) |
| S13 05_6 | Lower-third: "probability of success = containment × detection" | OK | USCG 1996 [2-3]: "POS = POCxPOD" |
| S14 06_7 | Quote: "The probability distribution on which the search is based is therefore a subjective one." | OK | Stone et al. 2014 p. 71, verbatim |
| S15 06_8 | "× 1¼ (100 dots ÷ 80 dots)" | OK | |
| S16 07_1 | "Thomas Bayes, died 1761 … Laplace, 1774" | OK | ODNB. Stigler 1986 p. 359, via the research notes: Laplace's principle is "equivalent to Bayes's theorem with all causes being a priori equally likely" |
| S17 07_2 | Table 1, ⅓, 1, 1; sum 80 "the chance this search comes back empty"; p(1 − d) ÷ (1 − pd) | OK | Exact |
| S18 07_4 | Rings: 10 in A, 13⅓ in C, 6⅔ in B and D | MISMATCH | Fractional rings before the split; the voice says "thirteen" (problem 6) |
| S19 07_5 | 60, 15, 10, 15; third search A, 30 out | OK | Exact |
| S20 08_1 | Order BCAABDAC; curve 20 … 74 | OK | Reproduced |
| S21 08_2 | Slices A 10, 7½, 5.6; B 20, 6⅔, 2.2; C 13⅓, 4.4; D 6⅔, 2.2 | OK | Reproduced |
| S22 08_3 | "65,536, checked one by one"; "M. C. Chew, 1967: after any number of searches, the least chance of still searching" | OK | Brute force; Chew p. 494 (page image) |
| S23 08_4 | E5, B7, C5, G5; "one search, about 28 sea miles of towing" | OK (model) | Reproduced; 1.099 × 25.5 = 28.0 |
| S24 08_5 | Row by row 0.8%, map never updated 44.3%, rule 49.8% | OK (model) | 0.76 / 44.35 / 49.77 |
| S25 09_1 | "B. O. Koopman, Search and Screening, 1946" | OK | OEG 56 title page (research notes); OCR text in hand |
| S26 09_3-4 | Mean 63%; "2 in 3 … at 1.1"; "1 in 4 … at 0.29"; Koopman: "a proper estimate, on the conservative side" | OK | Koopman p. 35 verbatim: "(1) is a proper estimate, on the conservative side, of the practical results achieved" |
| S27 09_6 | "1 of 9"; 27 dots: 18, 6, 2, 1 left | OK | Exact |
| S28 10_3 | "0.1 sea mile, 185 m: person in the water, helicopter at 300 feet. US Coast Guard, Table H-15" | OK | Table H-15 (text layer). 0.1 × 1,852 = 185 m |
| S29 10_4 | Coverage 2.7, 1.0, 0.4, 0.14 → 93, 65, 34, 13% | OK (model) | Reproduced |
| S30 11_1 | "CASP, 1974. SAROPS, since 2007: thousands of particles for each scenario" | OK | Kratzke 2010: "operational since January, 2007" |
| S31 11_2 | "24 July 2013" | OK | USCG release |
| S32 11_3 | "first guess: 9:30 to 11:30 p.m." | OK | NYT |
| S33 11_5 | Fall moves from 10:30 p.m. toward 3:30 a.m. | OK | NYT |
| S34 11_6 | "seen about 2:58 p.m.: 11 hours 28 minutes" | OK | NYT: 2:46 p.m. plus "Twelve minutes later" |
| S35 12_1 | "1 June 2009. 228 people" | OK | BEA final |
| S36 12_2 | "40 nautical miles (74 km)"; searches 2009, 2009, 2010; "22 months" | OK | BEA final: "22 months after disappearing". 40 × 1.852 = 74.1 |
| S37 12_3 | Discs "anywhere in the circle", "near the last position", "traced back from what was found afloat"; 35, 35, 30; "75,000 particles; each dot here stands for 25" | OK | p. 72-73: D1 uniform, D2 circular normal around the LKP, D3 reverse-drifted bodies. "N = 75,000" |
| S38 12_5 | "77 in 100 it would have been heard"; lower-third "in the authors' words: one map, with a weight on whether the beacons worked" | WRONG (lower-third) | 0.25 × 0.9216 + 0.75 × 0.72 = 0.7704. The lower-third is a paraphrase presented as the authors' words (problem 5) |
| S39 12_6 | "about 6.5 nautical miles (12 km) from the last position"; "The investigators planned from the first map" | OK | BEA final: "about 6.5 NM on the radial 019". Stone et al. p. 75: "It is P̃1234 that the BEA used to plan the successful 2011 search." |
| S40 13_1 | 10 June; 12 August; L′ = L(1 − E) ÷ (1 − SEP); "p. 146" | OK | RS71 pp. 142, 146, 149 |
| S41 13_2 | F6: 94 of 100 go out | OK | Table 1 |
| S42 13_4 | "believed at sea: sonar 800 ft" | OK | p. 151 |
| S43 13_5 | "measured afterwards: camera 185 ft, magnetometer 220 ft", sonar 0; "believed: 2.8 times over", "measured: 1.05" | OK (wording note) | −ln(0.06) = 2.81, −ln(0.35) = 1.05. "Measured" → "estimated" (problem 3) |
| S44 13_6 | F6 "before any search 0.85%. By the ship's numbers 0.19%. By the measured ones 0.54%" | OK | 0.85 × 0.06/0.27 = 0.189; 0.85 × 0.35/0.55 = 0.541 |
| S45 13_7 | "photographed in the summer; not found again"; "28 October 1968"; quoted lower-third | OK | p. 147: "cursory investigations which were not successful in regaining or rephotographing the contacts". p. 148: lower-third verbatim |
| S46 14_2 | Panel labels "widths re-measured after the find", "a second map", "re-run with new evidence" | OK | As above |
| S47 14_4 | "the update multiplies and never adds: a place with no dots can never gain one" | OK | p × (1 − d) of 0 is 0 |

### Always/never sentences

| line | sentence | edge test | result |
|---|---|---|---|
| 01_2 | "It has still told you something" | The search shown can see | holds |
| 02_3 | "All ninety-nine men were lost" | *DANFS* | holds |
| 03_3 | "could be anywhere" | Uniform ¼ | holds |
| 05_6 | "not always the likeliest" | B beats A | holds |
| 06_8 | "Every cell we did not search grew by a quarter" | All three unsearched cells nonzero, × 5/4 | holds |
| 07_2 | "one for every other cell" | No false alarms | holds |
| 07_4 | "Only B has fallen" | Catches in original dots: A 10, C 13⅓, D 6⅔ unchanged | holds |
| 07_5 | "the whole method" | Rhetoric; 14_4 completes it | holds as rhetoric |
| 08_1 | "Always search where …" | Posed as a question, answered at 08_3 | holds |
| 08_2 | "a fixed slice … does not matter what was searched in between" | Stationary target, independent looks, fixed d | holds |
| 08_2 | "a cell's slices only shrink" | 0 < d < 1 strict; d = 1 weak | holds |
| 08_3 | "best eight searches are the eight biggest slices" | Prefix property, verified | holds |
| 08_3 | "no other order is ever ahead" | Chew; brute force over 65,536 | holds, but "ahead" means by the map, and seeing chances must be known (problem 15) |
| 09_2 | "would cover all of it" | Perfect tiling | holds |
| 09_6 | "Each pass finds less than the one before" | 6, 2, ⅔ … | holds |
| 09_6 | "no number of passes makes it certain" | d < 1, fresh tries | holds |
| 10_1 | "A wreck stays where it is" | The video's wrecks | holds |
| 11_1 | "after every empty search" | Kratzke 2010 sec. 2.4 | holds |
| 11_2 | "Nobody saw him go" | NYT | holds |
| 12_1 | "All two hundred and twenty-eight … killed" | BEA | holds |
| 12_5 | "If they were broken, not at all" | Metron 2011 p. 34: "the ULB search would have no chance" | holds |
| 12_6 | "Both maps put their weight near the centre" | Figures 7 and 8 | holds |
| 13_5 | "had not seen her at all" | Table 2, one sonar | holds |
| 13_6 | "ten more passes … would have changed nothing" | Needs W = 0 exactly, which is a crude one-sonar estimate; also the ship's numbers would have changed | **fails as worded** (problem 3) |
| 14_1 | "only tells you something if it could have found" | d = 0 gives no change | holds |
| 14_4 | "does not have to be right … has to leave room" | Zero stays zero; each story was put right from outside, which the fourth sentence covers | holds |
| 14_5 | "could not be saved" | NT-24-1 | holds |
