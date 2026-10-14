# R2 script review: SAR planner lens (bayesian_search, "How to Find a Lost Submarine")

Saved verbatim from the text the reviewer returned (2026-10-13); the reviewer
did not write the file itself. The lens is the order's "which an engineer would
object to", read here as a practising search planner at a rescue coordination
centre (the persona prompt is described in `R2.md`). Finding ids S1 to S22.

I read the persona prompt and the whole packet (brief, script, storyboard, description, fact sheet) and no other file. I wrote scratch scripts outside the repository and edited nothing in it.

## Comprehension check, with a per-chapter verdict

| ch | one sentence on what a watchstander would take away | verdict |
|---|---|---|
| 01 | A search that finds nothing still reshapes the map, shown on the 1968 grid. | ok (imagined search is unlabelled, S16) |
| 02 | In 1968 the Navy did not know the 99 were already lost, and one careful pass over about 150 square miles with a 200-foot strip was out of reach. | caveat: the 200 ft is the after-the-fact width (S6) |
| 03 | Weigh two stories, turn them into a hundred dots, count per cell, and you have a map: 40, 30, 20, 10. | ok |
| 04 | The Navy made its map the same way (nine weighted stories, 10,000 runs), and the wreck lay next to the top cell, so a map alone is not an answer. | ok |
| 05 | The best cell to search is chance there times chance of seeing, which can be a less likely but easier cell. | ok (this is POS = POC × POD) |
| 06 | An empty search removes the dots it would have seen, and unsearched cells rise by the same factor though nothing about them changed. | ok |
| 07 | That move is Bayes' rule, and the loop is search, come back empty, update, choose again. | caveat: "the whole method" (S8) |
| 08 | Taking the biggest remaining slice each time is provably the best order for a wreck that stays put, with fresh equal-cost tries. | caveat: strawman baseline, travel cost, "fresh try" (S7, S8, S9) |
| 09 | A search covers only part of what it passes over, 63% at one coverage, and each further pass finds less. | would correct: the cause given for "two in three" (S1) |
| 10 | A drifting target spreads, the same track covers less each hour, so a single flight finds the person less and less. | would correct: sweep width as a hard edge, and an unchecked drift-error model (S2, S4, S10, S11) |
| 11 | The Coast Guard's map was wrong because it was told the wrong time, a clue moved it, and a helicopter found him. | caveat: who decided and what the software did (S3) |
| 12 | Air France 447 analysts weighted three ingredients and drew two maps because they doubted the silence, and the wreck was where both pointed. | caveat: red on the silence, unchecked pairing (S12, S13) |
| 13 | On scene the right cell looked 94% searched because the believed strip was about four times too wide, and measured afterwards it was 65%. | caveat: the sonar zero and the hindsight width (S5, S6) |
| 14 | An empty search tells you something only if it could have seen the thing, the first map must leave room for the truth, and the method now searches for people who can be saved. | ok (S18 is a one-line addition) |

## Findings

Findings that would change the video come first.

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| S1 | SAR planner | 09_1, 09_3, 09_4 | major | The chapter asks "why would a search see a wreck only two times in three?" and answers with strips that "land at random". In practice the shortfall at coverage one has two causes the video leaves out: real sensors miss inside their own strip (the lateral range curve), and a deliberately laid parallel pattern is not random. A trainee leaves thinking the chance of seeing is a penalty for sloppy track laying, which is also not what happened to the sled, which was towed on deliberate lines. The honest phrase "conservative side" is in a lower-third for a few seconds and fully only in the description. My own check, with an inverse-cube sensor and parallel lanes, gives 0.79 at coverage 1.0 and 0.83 at 1.1, against 0.63 and 0.67 for random strips. At coverage 0.14 the two agree (0.139 against 0.131). It is an illustration of mine, not a source. | Picture only. At 09_4 relabel the curve "strips laid at random: the floor you can count on". Add a second, dimmer mark "same track in parallel lanes: higher, still not 100%, because sensors miss inside the strip". Keep the numbers out unless the oracle gets a key for them. Put the same two sentences in the description beside the Koopman bullet. |
| S2 | SAR planner | 10_3 | major | "The Coast Guard counts on spotting someone in the water only inside a narrow strip" with the lower-third "sweep width (the width of the strip)" is the hard-edge reading of sweep width. Sweep width is the area under the lateral range curve: a searcher sees some of what lies outside it and misses some of what lies inside, and the two balance. The right words ("effective width, not a hard edge", and that the 0.1 belongs to one object, height and visibility) are only in the description. A trainee who learned this at 10_3 will draw hard edges on a chart. | Lower-third: replace "(the width of the strip)" with "an effective width: what it sees outside the strip balances what it misses inside". Name the visibility the 0.1 comes from in Table H-15 (unchecked; not in the packet). Show "sweep width" at its first use, the strip at 02_5, not at 10_3. If one spoken phrase may change: 10_3 "only inside a narrow strip" becomes "as if it saw only a narrow strip". |
| S3 | SAR planner | 11_1, 11_4 to 11_6 | major | The chapter shows software that "moves thousands of dots and updates them", then a clue, a re-run, a helicopter and a green point. The picture reads "the map led the helicopter to the man". What actually happened (fact sheet finding 7) is that the program froze and the last track was drawn by hand through the high-chance area, and people read the hatch and pumps as a clue to the time. That is the lesson I teach: the tool holds the dots; people supply the time, the clue, the legs and the eyes. The narration is careful ("a helicopter flying through those dots"), but the picture is not, and the hand-drawn track is only in notes. | Lower-third in the 11_6 hold: "The program froze. The last track was drawn by hand through the high-chance area (Tough, NYT Magazine, 2014)". Add at 11_5, only if S10 names who read the pumps as a clue, a caption "a planner read the pumps as the time he fell". Narration unchanged. |
| S4 | SAR planner | 10_2 to 10_4 (and F-D.* in the oracle) | major, unchecked | The cloud's edge is a 1-mile start plus 0.3 times the distance drifted, with every dot inside. The packet calls 0.3 "the Coast Guard's long-standing rule of thumb" and never says what kind of number it is. As I recall the manual method, drift error is a probable error, a 50% radius that is then multiplied by a safety factor to contain the object, not the edge of a disc that holds everything. I could not check S13 from the packet. If I am right, the containing circle is about twice the radius and four times the area, and 93, 65, 34, 13 become about 49, 23, 10 and 3 (I computed it as coverage divided by 4). The trend and "hours matter" survive; the magnitudes do not. The narration's "every dot moves with its own guess" and the model's "one error kept for the whole drift" are also two different things. | Check S13 (Frost and Stone 2001) for what 0.3 is. If it is a probable error or a standard deviation, redo F-D.* with the containing radius. If the author keeps the disc, relabel on screen: "model: every dot within a mile plus three tenths of the drift; the Coast Guard's own search circles are drawn larger". Narration already says "in this model". |
| S5 | SAR planner | 13_5, 13_6 | minor | "Ten more passes with that sonar would have found nothing" rests on a sweep width of zero, which the paper estimated from one sonar pass that did not detect her and which it calls "crude". "At that width" signs it, but a watchstander hears certainty. The calibration against the real target is the best thing in the video, so its limits should be as visible as its result. | Put the paper's own words on screen at 13_5: "from one sonar pass that did not detect her; the paper: 'crude sensor sweep width estimates against the actual target'". If a word may change: 13_6 "would have found nothing" becomes "would very likely have found nothing". |
| S6 | SAR planner | 02_5, 02_6 against 13_4 | minor | The hook uses the width measured after the find (200 ft), so one pass is 4,594 sea miles and "several times what the sled did". At the 800 ft they believed, one pass is 1,148 miles, about what the sled had towed (1,026). The first chapter says you cannot search everywhere, and chapter 13 says the cell looked 94% done. The viewer must reconcile them alone, because the 1,148 appears only in a storyboard note. A watchstander will see that the Navy, by its own numbers, had nearly searched everywhere. | Caption at 13_4 beside the red 800 ft strip: "at 800 ft, one pass over the whole patch is 1,148 sea miles; the sled had towed 1,026" (F-H.once.miles.believed is already in the oracle). No narration change. |
| S7 | SAR planner | 08_4, 08_5 | minor | "Row by row, no map" gets 0.8% for twenty searches because it starts in the corner where the map is coldest. Without a map any order is about equal. By its own flat reckoning twenty of 180 cells at two in three is 20/180 × 2/3 = 7.4% (6.6% with the 89% of the dots inside the grid). So the large gap of 0.8 against 49.8 is partly an unlucky corner. "Most of that is the map itself" and the 44.3% curve are the honest part. Trainees will also map "row by row" onto a creeping line. | Add a fourth marker on 08_5's chart, "no map, any order: about 7", and label the row-by-row curve "from the corner where the map is coldest". Both numbers would need oracle keys. |
| S8 | SAR planner | 07_5, 08_3 | minor | "That loop is the whole method" claims too much. The loop needs three things it cannot supply: the stories, the weights, and a chance of seeing for each sensor. The rule's edge conditions leave out that moving between cells is taken to cost nothing (in the description only), and "fresh tries" is not the word we use (independent). | Lower-third at 08_3, after Chew's line: "and moving between cells costs nothing; a fresh try is independent of the last". One word if allowed: 07_5 "the whole method" becomes "the heart of the method". |
| S9 | SAR planner | 05_2, 08_3, 09_5, 13_6 | minor | The made-up canyon is the classic case where repeated passes are not fresh tries: a shadow hides the same spot from every pass from the same side. The spoken edge "fresh tries" is correct, but the example invites the opposite reading, that you can keep going at the canyon cell at one in four each time. Chapter 13 shows the failure only for the sonar. | Lower-third at 08_3 or 09_6: "A canyon hides the same place from every pass from the same side. Change the angle or the sensor, not only the count (see 13_6)". |
| S10 | SAR planner | 10_4 | minor | "That is why hours matter" answers the owner's "time is running out" with only the growth of the area. How long a person survives, which depends on water temperature and is what makes the cold open's "men are trapped" bite, is neither said nor modelled. A trainee may conflate the two, or conclude that a 13% flight at twelve hours means the search is hopeless, when real searches go on for days and the chances accumulate. | On-screen line in the 10_4 hold: "Not in this model: how long a person survives. That clock runs too." No numbers. |
| S11 | SAR planner | 10_4 label | minor | The note says "180 sea miles of track spread evenly over the circle", which suggests parallel lanes, but the numbers come from Koopman's random formula, the cautious one. With lanes the 3-hour figure would be nearer 80 than 65 under my inverse-cube check. At 12 hours the layout hardly matters (13 against 14), so the headline is robust to layout and sensitive to S4. | Change the note to "model: the random-search formula (cautious); lanes laid in parallel do somewhat better at 1 to 3 hours, hardly at all at 12". |
| S12 | SAR planner | 14_3 (colour), 12_4 | minor | The listening tracks light in `warn` red, and red means "an estimate that turned out wrong". The AF 447 sources do not say the first map was wrong: the BEA planned the 2011 search from it and the wreck is dark on both maps. The narration says "might have meant nothing", the picture says "wrong". | Draw the listening tracks as a `dim` outline tagged "doubted", not red. |
| S13 | SAR planner | 12_3 | minor, unchecked | The picture pairs 35, 35, 30 with "anywhere in the circle", "near the last position" and "traced back from what was found afloat". The fact sheet has the weights (F-A.weights) but no key for which ingredient got which, and 14_4's note presumes 35% for "anywhere in the circle". I cannot confirm the pairing from the packet. | Confirm against S3 and add a fact key for the pairing, or show the three weights without labels paired to them. |
| S14 | SAR planner | 11_6 | minor | The script says "two lobster buoys", the storyboard note says "a lobster buoy", and the fact sheet says "lobster buoys" with no count key. One of the three is wrong or unsourced. | Check S10 and make the three agree. If only "buoys" is sourced, the narration word "two" would have to go; otherwise add a key. |
| S15 | SAR planner | 09_2 to 09_4, 10_4, 13_5, 10_1 | minor | "Coverage" is used for two things: the percent of area swept (the 09_2 readout and "covers sixty-three percent") and the ratio W·L/A (10_4, 13_5, where 63% sits at coverage 1.0). In our vocabulary coverage factor is the ratio. Track spacing, creeping line, datum and containment are not named, so the video does not connect to the words a watchstander will hear next. | Label the 09 readout "swept", and keep "coverage" for the ratio. In the 09_2 caption: "track spacing equal to sweep width is coverage 1". Tag the row-by-row plan "(a creeping line from one corner)". Label the cloud's centre "datum" at 10_2. All captions or lower-thirds. |
| S16 | SAR planner | 01_2 | minor | The cold open shows a strip crossing the real 1968 grid's brightest cell with most of its dots going out, deliberately with no labels. A watchstander can take it for what the Navy did (the essay's false claim, "started the search with F6", is the same slip). | A dim "an imagined search" tag appears in the 01_2 hold, after the strip, before the second sentence. |
| S17 | SAR planner | 02_3 | minor, unchecked | "More than ten thousand feet" is the Navy's own wording (S8). I recall popular accounts giving about 9,800 ft (3,000 m), and I could not check either from the packet. A viewer who looks it up may think the video is wrong. | Caption "the Navy's figure: more than 10,000 feet" on the 02_3 card, and a line in the description. |
| S18 | SAR planner | 14_4 | minor | "The first map does not have to be right. It has to leave room for the truth" is now true and narrowly worded. The risk is the other reading, a licence for a careless first map. The Aldridge case is the cost: a guessed time moved the map 30 miles. | One line in the 14_4 hold: "Wrong inputs cost hours: a guessed time put the first map 30 miles north." (F-C.aldridge.miles_south) |
| S19 | SAR planner | 13_2 to 13_6 | delight | The chapter I would show: the analysts print their own optimistic sweep widths, measure them against the real target, and the 94 becomes 65. The wreck's white ring appearing among the six lit dots is exact. It teaches that a chance of seeing is measured, not wished for. | Keep as drawn. |
| S20 | SAR planner | 06_7, 14_1 | delight | "What changed is what we know" and "An empty search only tells you something if it could have found what you were looking for" are sentences I would put on the watch-floor wall. The first has the authors' subjective-distribution quote under it. | Keep; both are already in the narration. |
| S21 | SAR planner | 11_3, 11_4, 12_5, 12_6 | delight | The first map wrong because its inputs were, and two maps drawn for a doubted input instead of choosing one, is the nearest the video comes to real planning. The AF 447 language ("high-chance part of the second map") does not overreach. | Keep. |
| S22 | SAR planner | 02_3, 14_5 | delight | Two and a half seconds of silence after the 99, with no dot dimming and the card returning unchanged, is respectful to the crew and to the people who searched. | Keep. |

## What I ran and what I did not check

Everything below was run with `python3 -I`, standard library only.

- **Dates:**
  - 27 May to 5 June is 9 days.
  - 22 May to 27 May is 5 days.
  - 27 May to 28 October is 154 days (5.06 months).
  - 10 June to 12 August is 63 days.
  - 1 June 2009 to 2 April 2011 is 670 days (22 months).
  - 03:30 to 14:58 is 11 h 28 min; 22:30 to 03:30 is 5 h.
  - All agree with the script.
- **Hook arithmetic:** the grid is 151.2 square miles and 180 cells.
  - One pass is 4,594 sea miles at 200 ft, 4,966 at 185, 4,176 at 220 and 1,148 at 800.
  - There are 25.5, so 26, passes per cell.
  - 1,026 miles cover 40.2 cells.
  - 4,594 ÷ 1,026 is 4.48.
  - One search at coverage 1.10 is 28 miles.
- **Four-cell sea:**
  - Every number in 03 to 07 reproduces: 40/30/20/10, find chances 10/20/13⅓/6⅔, the update to 50/12.5/25/12.5, ×1.25, then 60/15/10/15, then A next.
  - The rule's curve 20, 33, 43, 51, 58, 64, 70, 74 and the slices match 08_2.
  - I brute-forced all 4⁸ = 65,536 orders: the rule is best at every n up to 8.
  - Row by row is level with the rule at n = 3, 5, 6.
  - Expected searches are 6.34, 7.04 and 8.00.
- **Koopman:**
  - 1−e^(−C) gives 39.3, 63.2, 86.5, 95.0.
  - Flat 2 in 3 needs coverage 1.10 and the canyon 1 in 4 needs 0.29.
  - The exact mean of 100 one-percent strips is 63.4 (and 86.6, 95.1).
  - The nine-dot quiz gives 6, 8 and 1 in 9, and 1 in 27 after three passes.
- **Drift model:** radii 1.45, 2.35, 3.70, 6.40; areas 6.6, 17.3, 43.0, 128.7; coverage 2.73, 1.04, 0.42, 0.14 give 93.4, 64.6, 34.2, 13.1%.
- **Chapter 13:** 1−e^(−2.81) = 0.940 and 1−e^(−1.05) = 0.650. The ratio left is 5.8. L′ comes out 0.189% and 0.541% from 0.85% with SEP 0.73 and 0.45.
- **AF 447:** 40 NM is 74.08 km and 17,241 km². 6.5 NM is 12.0 km, 16% of the radius. 75,000 ÷ 3,000 is 25.
- **1968 map consistency:**
  - Rows and columns each sum to 10,000.
  - E5 = 1 in 8; E5 + B7 = 23.5%.
  - E5, B7 and F6 fit the column and row totals only if lettered column A is the 4th table column and row 1 the 3rd table row.
  - The lettered grid then holds 8,900 with an overlap of 7.
  - The model plans' bounds hold: row by row at 20 searches is at most 2.25% (claimed 0.8); at 81 it is at most 36.0% (claimed 32.9); once-each at 81 is at most 59.3% (claimed 57.8).

**Unchecked, because the packet does not carry the source or the table:**
- The model-plan percentages 0.8, 44.3, 49.8, 32.9, 57.8 and 73.2 as such (only their bounds).
- F6 rank 24 of 172.
- The seeded run 66.5% (needs NumPy's PCG64DXSM) and the 2,000-dot cloud.
- Every quotation from the 1971 paper, Koopman p. 35, Stone et al. and Tough.
- Table H-15's value and its conditions.
- The Aldridge hatch and pumps, the 9:30 to 11:30 guess, the 08:30 re-run and the 14:58 sighting.
- The month of listening in AF 447 and the "about a week".
- CASP 1974 and SAROPS 2007.
- The Scorpion depth (S17).
- The 0.3 factor's definition (S4).
- The parallel-sweep figures in S1 and S11 are my own model, not a source, and should not go on screen without an oracle key.

## Final judgement

I would show it to new watchstanders. The core arithmetic is exact, the update is the same one we use (POC′ = POC(1−POD)/(1−POS)), the Scorpion history keeps its edges, and the honest chapters (13, 11 and 12) teach what I most want new people to take in: a chance of seeing is measured, a first map is only as good as its inputs, and nothing found is not the same as nothing there. The caveat I would say first is that the "two in three" and every detection figure in the video are illustrations, the random-laying figure being a cautious floor and not what a deliberately laid pattern with a real sensor gives. Sweep width is an effective width, not an edge (S1, S2). The second caveat is that the dots are the tool's half: people supply the scenarios, the evidence, the legs and the eyes, and in the Aldridge case drew the last track by hand (S3). One item should be checked before release, the drift-error model behind 93, 65, 34, 13 (S4). Everything else I found is a caption, a lower-third or a description line, and none of the fixes needs a new narration take except at most one word each in 07_5 and 10_3.
