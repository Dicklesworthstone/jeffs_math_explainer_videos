# Fact sheet: Video H, "How to Find a Lost Submarine"

Bead `mx-e-a-h-6ix2.2` (gate G1). One row per number the video may show or
say, and one row per claim that rests on a source. Every value in the table
at the end was copied from the oracle's output by a script, not typed.

```
./x bayesian_search facts-check        # search facts == oracle.py, key by key
```

- **Result, 2026-10-14** (the build host), with the storyboard as revised
  after review R2: `facts-check bayesian_search: 320 video facts against 320
  oracle keys: PASS`. The oracle passes 62 of 62 of its own cross-checks.
  (Later the same day, with the description's computed numbers added as
  oracle-only F-T rows: `320 video facts against 351 oracle keys: PASS`, 97
  of 97 cross-checks; the rows are listed after the table at the end.)
  (After R1: 303 of 303; at G1: 263 of 263; with the first storyboard: 274
  of 274. R2 added fourteen keys: `F-H.feet_per_sea_mile.exact`,
  `F-H.once.per_cell.exact`, `F-R.grid.empty_cells`,
  `F-R.model.random.found_by.{20,81}`, `F-R.model.sweep.first20.points`,
  `F-R.model.F6.first_search`, `F-K.lanes.{min,max}.{1,2,3}` and
  `F-D.strip.feet`.)
- **Rust side:** `cargo test -p bayesian_search_explainer` on the build
  host: 152 tests pass, clippy `-D warnings` and `fmt --check` clean.
- **Rows:** F-M the four-cell sea; F-U the rules for where to search next;
  F-R the 1968 map and the model search over it; F-H the hook's arithmetic;
  F-K Koopman's formula and the seeded lanes; F-D the drifting cloud
  (model); F-S, F-P, F-A, F-C, F-N cited numbers and what follows from them
  (Scorpion, Palomares, Air France 447, the Coast Guard, the names); F-Q the
  quizzes.

## How the two sides get their numbers

| What | `src/model.rs` (the video) | `oracle.py` (independent) |
|---|---|---|
| the four-cell sea | exact fractions through the update formula: the searched cell keeps its misses, every cell is divided by the chance of an empty search | counting dots: the stories put dots in cells, a search puts out the dots it would have found, a cell's chance is its share of the dots still lit |
| "no order of searches does better" | every order of up to 8 searches (65,536 of them at 8), each through the update formula | the same orders by dot counting |
| the average number of searches | a float sum, with ties decided at one part in 10⁹ | exact fractions for every choice and every term |
| the 1968 map | `y1968::POINTS`, a full 16 × 20 table transcribed from the page image of Figure 2 | a second transcription, typed from zoomed crops of the same page as sparse rows; its sum is checked against the paper's 10,000 replications, and E5 + B7 against the paper's "23 percent" |
| the model search over that map (three plans, all on the 180 lettered cells) | a scan for the cell with the most chance left; a stable sort for "each cell once, likeliest first" | a sorted list of (chance left, place in reading order) |
| the proof in slices | each cell's slices from the closed form p d (1 − d)ᵏ, checked against the rule's run | each cell searched alone, over and over, on the dots; then the rule run on the dots, each search's dots compared with that cell's next slice |
| the code panel | the four lines as constants (`chapters::glyphs::CODE`), printed as facts | its own copy, checked to be the storyboard's text, then executed on the four-cell sea in exact fractions: the rule's order, and the maps 50, 12.5, 25, 12.5 and 60, 15, 10, 15 |
| the lanes over a cell | rows and columns marked; swept points = all points minus (unmarked rows × unmarked columns) | the same seed through NumPy's PCG64DXSM; a full 200 × 200 grid of booleans, counted |
| the drifting cloud | the kit's seeded stream, rejection from the square | NumPy on the same seed, the same rejection, standard-library `hypot` |
| dates | days from a civil date by arithmetic | `datetime.date` |
| cited numbers | constants in `facts::cited`, pinned to a literal with its source | the same literals typed a second time from the sources, with cross-checks where two of them must agree (1,026 miles at one knot = "about 43 days"; 153 miles at 24 a day = 6.4 days; the beacon sum 0.92, 0.72, 0.77) |

Rounding: fractions are rounded half up, exactly, on both sides (`Q::fixed`
and the oracle's `fixed`); the few float values (the formula, the cloud) are
printed with one or two places and none sits near a rounding edge.

## Findings (uncomfortable first)

1. **The map did not find the Scorpion by itself, and it was not there at
   the start.** The search began on 10 June; Craven and Andrews set the nine
   scenarios and their weights on 18 July; the map reached the ship on 12
   August (S1 p. 149: "it was only during the at-sea periods beginning with
   the third cruise that the a priori distribution had an opportunity to
   play a role"). On the third cruise the probabilities "did not play a
   substantial part"; on the fourth they "played an important role"; the
   wreck was found on the fifth, when effort went to F6 "because of
   proximity to the piece of metal and the fact that this was a region with
   high a priori probability" (p. 148). That piece of metal had been
   photographed between 27 June and 23 July, before any map existed. The
   video tells it this way and never says "Bayes found the Scorpion".
2. **The wreck was next to the top cell, not in it.** "Scorpion's location
   is in rectangle F6 within 260 yards of the edge of rectangle E5, the
   rectangle having distinctly the highest a priori probability" (p. 145).
   F6 held 85 of the 10,000 points (0.85%), 24th of the 172 printed cells;
   E5 held 1,250. Secondary accounts give 220 and 200 yards and move the
   distance from a cell's edge to a predicted point; the video uses the
   paper's sentence.
3. **At sea, the cell with the wreck looked almost fully searched.** The
   on-scene figures for how thoroughly F6 had been searched were 0.65 (10
   September), 0.94 (9 October) and 0.99 (28 October); the estimates made
   afterwards, with the sensors measured against the wreck as found, were
   0.47, 0.65 and 0.74 (Tables 1 and 3). The reason is in the paper: "the
   optimistic sensor (particularly sonar) sweep width estimates then
   available", "typically 800 feet and 400 feet for sonar and magnetometer"
   against 185 feet for the camera, 220 for the magnetometer and 0 for the
   sonar (Table 2). In the update, 0.94 leaves a cell 6% of its chance and
   0.65 leaves it 35%: almost six times as much. This is the video's
   closing lesson, and it is the paper's own.
4. **How the weights were set.** The order asks that this be verified
   before it is told. The primary source says that Craven and Andrews
   "postulated nine scenarios ... and assigned credences to them", and that
   a credence "incorporated (at least indirectly as interpreted by Craven
   and Andrews) the views and opinions of Navy operating personnel as well
   as the analysis of specialists in many scientific areas" (pp. 143-44).
   It does not list the scenarios or print the weights. The story that
   experts bet bottles of Chivas Regal is in *Blind Man's Bluff* and
   nowhere else that could be read (not in the paper, not in Craven's
   memoir, not in McGrayne). The video says two men gave nine stories
   weights and says nothing of bets.
5. **Palomares was not found by a probability map.** The official report
   chose the area that held the bomb "primarily" from sonar contacts given
   "high credibility because of the statements and plotted evidence from
   the Spanish fisherman" (S7 p. 87), and *Alvin* found it "in the
   vicinity of the place where Simo' Orts reported seeing a weapon descend"
   (p. 94). What began there, by the 1971 paper's account, is the method:
   "Craven had earlier originated this approach in the H-bomb search"
   (S1 p. 144), and the measures of search effectiveness "were used
   previously in the Mediterranean H-bomb search" (p. 141).
6. **AF 447: two maps, and no proof about the beacons.** The main map kept
   the beacon search with a detection chance of 0.77; "doubts about the
   beacons compelled us to produce the alternate posterior ... which assumes
   the beacons did not function", and the wreck "falls in a high probability
   area" of that one (S3 p. 78). No source gives a probability assigned to
   "both beacons failed", and none says the wreck was in the top cell. That
   the beacons failed is the authors' inference ("suggest that both beacons
   failed to actuate"); the BEA's test of the one recovered beacon found it
   "not functional" two years on and called the test "not conclusive" about
   2009. The authors were asked for the map, not the search plan (p. 72);
   the plan was the BEA's, a systematic search from the centre outward, and
   "It is P̃1234 that the BEA used to plan the successful 2011 search"
   (p. 75): the first map, the one that counted the silence. Both figures
   are dark near the centre. So the video draws the wreck's ring the same
   on both maps, and does not say the first map hid it (review R1, R10).
7. **The Aldridge search.** The Coast Guard's own releases never mention the
   planning software; its part is as reported by the *New York Times
   Magazine* from the command centre. The helicopter's last track was drawn
   by hand through the high-chance area after the software froze ("it
   wasn't Sarops-generated"), and Aldridge was not drifting freely: he had
   tied a cut-loose buoy to his partner's and was "sitting on the rope
   between his two buoys" when found (review R2: the draft said "tied
   himself", which is not the article's wording). The video says the first map was wrong
   because its inputs were, that new evidence moved it about thirty miles,
   and that a helicopter flying through the new area saw him; it does not
   say the software found him.
8. **In the small sea the rule's lead is modest, and on the Navy's map
   most of the lead is the map.** On average the rule takes 6.34 searches
   of the small sea, row by row 8.00, and "the likeliest cell first" 7.04
   (6.92 with the other tie-break); after 3, 5 and 6 searches row by row
   has found exactly as much as the rule. The theorem is "never ahead of
   it", and that is what is said. On the 1968 grid (model: the 180 lettered
   cells, every search sees two in three) twenty searches give, by the
   map's own reckoning, 0.8% row by row, 44.3% for a plan that has the map
   and never updates it, and 49.8% for the rule; at eighty-one searches
   32.9, 57.8 and 73.2. The first draft compared only the first and the
   last, raced the two to the wreck's own cell (a race a cruder plan wins),
   and said the percentages of "her" (review R1: D6, R5, R11).
9. **The paper's miles are sea miles.** It never says so; but 1,026 miles
   are "about 43 days of bottom time at the speed of 1 knot" (1,026 ÷ 24 =
   42.75) and 153 miles are "6.4 days at 24 miles per day". The hook's
   arithmetic therefore uses 6,076 feet to the mile, and the video says
   "miles".
10. **The student essay is a map of the topic, not a source.** Its checkable
    claims about the search are tabled below; one is false, three are
    imprecise or contradicted by the 1971 paper, two stand.
11. **Koopman.** The formula is first printed in his 1946 report, which he
    credits to a group ("Dr. G. E. Kimball, the pioneer in the theory of
    search"); the video says "Koopman and his colleagues". Sources disagree
    on when the report was declassified (1958 and 1966), so no year is
    given. No recording or note of how he said his name was found; the
    lexicon will carry the ordinary English reading and say it is unsourced.
12. **The rule is proved for a thing that stays put.** Chew's theorem (its
    number, 2.1, is unverified: S11) needs
    one object that does not move, independent searches with a fixed chance
    of seeing, and equal costs. For a drifting target the video says only
    that the map must be moved before each update, and makes no claim that
    the simple rule stays best (Brown 1980 treats that problem).

13. **The update never adds.** A cell with no dots stays at zero whatever
    is searched, so a first map must leave room for the truth; and each of
    the video's three stories was put right from outside the loop (sweep
    widths re-measured after the find, a second map, new evidence from the
    boat). The first storyboard's "it never needs the first map to be
    right" was false and is gone (review R1's one blocker).
14. **The 12-mile patch is the map's frame, not the Navy's fence** (review
    R2). The lettered grid of Figure 2 is 12 × 12.6 sea miles, and 1,100 of
    the 10,000 runs ended outside it. Nothing read says the Navy's search had
    been narrowed to it, and the secondary sources disagree on the size of
    the area the acoustic work left (see "Not said"). 02_5 therefore says
    "take a patch of sea about twelve miles on a side", and the screen calls
    it the grid of the Navy's later map.
15. **A plan with no map is not a plan that starts in the corner** (review
    R2, recomputed). Row by row from row 1, over the 180 lettered cells at
    two in three, finds 0.76% in twenty searches because its first twenty
    cells hold 114 of the 10,000 runs; the same plan from row 5 finds 22%,
    from row 12 0.2%, column by column from A 10.3%, and twenty cells
    chosen at random each once 6.6% on average (the 10th to the 90th
    percentile of 20,000 orders: 2.3 to 12.6). All of these are shares of all
    10,000 runs; over the 180 cells alone they would be larger by 1/0.89.
    The never-updated map (44.3%) and the rule (49.8%) are unchanged, and the
    rule's lead over no map is mostly the map.
16. **Which estimates were which in 1968** (review R2). The at-sea widths
    were typically 800 feet for sonar and 400 for the magnetometer; the
    afterwards estimates are camera 185, magnetometer 220 and sonar 0, the
    sonar's from the failure of one of the two sonars to detect the wreck
    ("crude sensor sweep width estimates against the actual target", p.
    151). The cell's coverage fell from 2.81 to 1.05 (a factor of 2.7)
    because every sensor and every pass is added; a strip four times
    narrower on its own would give 50.5%, not 65%.
17. **"The first map does not have to be right" is exact only with room for
    the truth** (review R2, four readers). For a thing that stays put, when
    every place that might hold it has some chance and every search can see
    (d > 0), the rule comes back to every such cell and finds it with chance
    1; the first map then sets only how long the search takes
    (F6, with 85 dots, is reached at search 30 of the model run; 40 of the
    180 cells have no dots and are never searched).

## Sources

`Seen` says what was read and how. Downloaded copies live outside the
repository and are never committed.

| id | Source | Seen | Used for |
|---|---|---|---|
| S1 | H. R. Richardson and L. D. Stone, "Operations Analysis During the Underwater Search for Scorpion", *Naval Research Logistics Quarterly* 18(2), June 1971, pp. 141-157 | in full, as page images (Internet Archive scan of the issue); Figure 2 and Tables 1 to 3 read from the images | the 1968 map, the update formula as printed (p. 146), dates, sweep widths, what the analysis did and did not do |
| S2 | B. O. Koopman, *Search and Screening*, OEG Report No. 56 (Washington, 1946) | in full (Internet Archive scan); pp. 28-29 and 35-41 from page images | the random search formula (eq. 40) and its three assumptions; "diminishing returns"; "by Bayes' formula" (p. 37); the plan that never needs redoing (p. 38) |
| S3 | L. D. Stone, C. M. Keller, T. M. Kratzke and J. P. Strumpfer, "Search for the Wreckage of Air France Flight AF 447", *Statistical Science* 29(1), 2014, pp. 69-80 | in full (arXiv 1405.4720 and the journal PDF) | the three weighted ingredients, the four empty searches, the detection chances, the two maps |
| S4 | Metron, "Search Analysis for the Location of the AF447 Underwater Wreckage", report to the BEA, 20 January 2011 | in full | detail behind S3 (0.83 for one sonar search where S3 says 0.90; 52 beacons, 5 failures) |
| S5 | BEA, final report on the accident to F-GZCP (July 2012), and "Sea Search Operations" (2012) | targeted passages | date, 216 + 12 aboard, the 40 NM circle, "about 6.5 NM", 3,900 m, "22 months after", the search phases |
| S6 | T. M. Kratzke, L. D. Stone and J. R. Frost, "Search and Rescue Optimal Planning System", FUSION 2010 | in full | CASP (1974), SAROPS ("operational since January, 2007"), particles, Bayes' rule on the weights |
| S7 | Defense Nuclear Agency, "Palomares Summary Report" (15 January 1975) | passages (OCR, with pp. 94 and 125 checked against page images) | the accident, the four weapons, the fisherman, *Alvin*, the dates |
| S8 | Naval History and Heritage Command, *DANFS*, "Scorpion VI (SSN-589)" (2018); Naval Nuclear Propulsion Program, report NT-24-1 (2024), pp. 7-8 | in full | 99, the dates of May and June 1968, "400 miles southwest of the Azores in more than 10,000 feet of water", sank 22 May, cause undetermined |
| S9 | US Coast Guard Addendum to the National SAR Supplement, COMDTINST M16130.2F (2013) | Table H-15 and sections 3.1.1, 3.7 | a helicopter's sweep width for a person in the water (0.1 NM); "approximately 90 percent of all cases involve ... no searching" |
| S10 | Paul Tough, "A Speck in the Sea", *The New York Times Magazine*, 2 January 2014; US Coast Guard releases of 24 July 2013 | in full (archived copies) | the Aldridge search |
| S11 | M. C. Chew, Jr., "A Sequential Search Procedure", *Annals of Mathematical Statistics* 38(2), 1967, pp. 494-502, DOI 10.1214/aoms/1177698965 | the abstract only (the Project Euclid record, citation check 2026-10-14); the paper's text NOT seen (the publisher and JSTOR refuse automated fetches, no open copy found), so **"Theorem 2.1" is unverified**: nobody has read the number off the page. An earlier version of this row said pp. 494-496 as page images; nothing here confirms the number from them | the theorem behind "no order does better": the abstract states the every-n result for one fixed object, independent looks with fixed overlook probabilities and unit cost, and says Blackwell showed that with costs the one-stage procedure minimises the expected searching cost |
| S12 | W. L. Black, "Discrete Sequential Search", *Information and Control* 8(2), 1965, pp. 159-162; J. B. Kadane, Cowles Foundation Discussion Paper 220 (1967) | in full | the rule with unequal costs; why a fixed budget with unequal costs is a different problem |
| S13 | J. R. Frost and L. D. Stone, *Review of Search Theory: Advances and Applications to Search and Rescue Decision Support*, USCG report CG-D-15-01 (2001) | chapters 1 to 4; text read in an archived copy of DTIC ADA397065 (127 pp.), with 4.5, 4.6.1.3 and Table 4-2 (citation check 2026-10-14) | Koopman's group in the war; the drift error as three tenths of the distance drifted, **a probable error** (the radius of the circle that holds 50 percent), later increased from one-eighth to 0.3 "but even this may have been optimistic" (4.6.1.3); the first search as a square of half-side 1.1 E, "the POC of a square with this inscribed radius is 0.64762" (Table 4-2, p. 4-18); the flat-topped posterior |
| S14 | T. Bayes, "An Essay towards solving a Problem in the Doctrine of Chances", *Philosophical Transactions* 53 (read 23 December 1763), pp. 370-418; S. E. Fienberg (2006); S. M. Stigler (1986); *Oxford DNB* | pp. 370-376 of the essay; the cited pages of the others | Bayes died in 1761; the essay was sent by Richard Price; the rule in the form used here is Laplace's (1774) |
| S15 | J. P. Craven, *The Silent War* (2001); S. Sontag and C. Drew, *Blind Man's Bluff* (1998); S. B. McGrayne, *The Theory That Would Not Die* (2011) | paragraphs or search fragments of library scans only | point of view; never a quotation and never a number |
| S16 | The owner's reference: *The Search for the USS Scorpion: A Modern Application of Bayesian Statistics* (tomrocksmaths.com, 2023), a student essay | in full | the map of the topic; its claims are tabled below |
| S17 | United States National Search and Rescue Supplement to the IAMSAR Manual, National SAR Committee, May 2000, glossary (the copy hosted by Civil Air Patrol) | the glossary text read; the IAMSAR Manual Vol. II itself NOT seen (not freely online); the 2018 edition's PDF refuses fetches (a search snippet of its glossary has the same 0.3 wording) | the confidence factor, "normally 0.3", used to determine drift error; the probable error ("the range on either side of the average ... such that the probability of being within that range is 50%"); the total probable error as a root sum of squares |

### Passages quoted or leaned on

- S1 p. 143: "a conference was held July 18 where Dr. J. P. Craven and Dr.
  F. A. Andrews postulated nine scenarios concerning the events attending
  the disaster and assigned credences to them."
- S1 pp. 144-45: "The result of 10,000 replications is shown in Figure 2."
- S1 p. 145: "It has now been determined that Scorpion's location is in
  rectangle F6 within 260 yards of the edge of rectangle E5, the rectangle
  having distinctly the highest a priori probability. Considering the size
  of the area covered by the a priori distribution, this is indeed a near
  miss."
- S1 p. 146, the update as printed: L′ⱼ = Lⱼ(1 − Eⱼ) ÷ (1 − SEP), "the
  probability that Scorpion is in rectangle j considering the
  (unsuccessful) search".
- S1 p. 148: "The LEP's computed on scene for F6 are, to a surprising
  degree, higher than the after-search estimates ... largely because of the
  optimistic sensor (particularly sonar) sweep width estimates then
  available to the analysts."
- S1 p. 149: "The distribution was then hand-carried at the earliest date
  to the scene of operations arriving August 12."
- S1 p. 141: the analysis "provided a quantitative framework for the
  planning and documentation of the search effort; of itself this
  substantially enhanced the continuity of the search in the face of
  considerable turnover of leadership".
- S2 p. 28: "These assumptions lead to the formula of random search
  p = 1 − e^(−WL/A)"; p. 35: it is "a proper estimate, on the conservative
  side, of the practical results achieved".
- S3 p. 76: "Experience in past searches has shown that detection estimates
  based on manufacturers' specifications and operator estimates tend to be
  optimistic. Thus, we put a maximum of 0.9 on estimates of sensor
  detection probabilities."
- S3 p. 71: "The probability distribution on which the search is based is
  therefore a subjective one."
- S6: "After an unsuccessful search, the posterior particle probabilities
  are computed using Bayes' rule."
- S11 p. 494: the rule "is shown to be strongly optimal in the sense that
  P[N > n | δ*] ≤ P[N > n | δ], for each n = 0, 1, 2, ..., and every search
  procedure δ."
- S8 (*DANFS*): "on 5 June, she was declared 'presumed lost' with all
  hands"; "A court of inquiry convened but did not determine the cause of
  her loss."

## The reference essay's claims

| Claim | Verdict |
|---|---|
| "After eight days passed, the Navy issued a statement declaring the crew lost at sea and presumed dead." | Imprecise. Overdue 27 May, "presumed lost" 5 June: nine days (S8). The video gives the dates. |
| "strapped to the Scorpion were two nuclear-tipped torpedoes", and the search was to find "these dangerous warheads" | The number is in a Navy report (NT-24-1: "carried two torpedoes with nuclear weapons"); "strapped to" is wrong, and no source read gives the warheads as the search's purpose. Left out. |
| "reducing the searchable area from a stretch of 3000 miles to just a few square miles" | Contradicted by the map itself: the lettered grid is 151 square miles and 1,100 of the 10,000 points fall outside it. Not said. |
| "starting the search off with rectangle F6" | False. F6 is where the wreck was found; E5 had the highest prior; the search began two months before the map arrived (S1). |
| "only a few hundred yards away from one of the initial high-probability regions marked on Craven's map" | True in substance: 260 yards from the edge of the single highest cell; the map was computed by Richardson from Craven's and Andrews's scenarios. |
| "Craven gathered and evaluated the opinions of various specialists regarding each possible scenario." | Supported, with the paper's hedge ("at least indirectly as interpreted by Craven and Andrews"). |

## Not said, and why

- That the Navy had narrowed its search to a 12-mile patch (finding 14).
- A cause of the *Scorpion*'s loss: undetermined (S8).
- A number of acoustic signals, and the size of the area they left: the
  secondary sources disagree (five trains, eight events, nineteen signals;
  "three or four square miles", "12-by-12-mile").
- The Chivas Regal bets (finding 4).
- "Bayes found the H-bomb" (finding 5) and any distance between the
  fisherman's point and the bomb (no source gives one).
- "The mathematicians proved the pingers were broken", and a probability of
  beacon failure (finding 6).
- "SAROPS found John Aldridge", and any lives-saved total credited to the
  software (finding 7; nine cases in ten involve no search, S9).
- A year for the declassification of Koopman's report (finding 11).
- A depth for the *Scorpion* in metres as if sourced: the sources say "more
  than 10,000 feet"; "about two miles" is from S1 ("as much as 2 miles
  below the surface").

## Terms (the jargon ledger)

| Term | First said at | As spoken | On screen |
|---|---|---|---|
| strip; sled | 02_5 | "the search gear rode a sled, towed just above the sea floor, and saw a strip about two hundred feet wide" | the strip drawn to scale; "sweep width (the width of the strip)" once, at 10_3 |
| map; cell | 03_1, 03_2 | "a map of how likely each place is"; "Four cells" | the grid; "prior: the map before a search" at 03_7 |
| story; weight | 03_2, 03_4 | "two stories of what might have happened to it"; "Experts weigh the two stories" | "scenarios" and "credence" once, at 04_2 |
| dot | 03_4 | "Each dot is one place the wreck could be." | "a dot: one equally likely way things could be" at 06_3; "particle" at 11_1 |
| chance | 03_5 | "What is the chance the wreck is in cell A?" | "probability" only in lower-thirds |
| the chance of seeing it | 05_2 | "a search there sees the wreck only one time in four" | "1 in 4"; "detection probability" once, at 05_6 |
| the chance of a find | 05_3, 05_6 | "How likely is one search of a cell to find the wreck?"; "the chance it is there, times the chance of seeing it" | the three slots under each cell |
| comes back empty; miss | 06_1, 06_4 | "the search comes back empty"; "because a search can miss" | "empty"; never swapped |
| Bayes' rule; update | 07_1, 07_5 | "That move has a name. Bayes' rule"; "Update." | "update: redrawing the map by Bayes' rule" at 07_2; "prior", "posterior" at 07_1 |
| slice | 08_2 | "a fixed slice of the first hundred dots" | the bars |
| fresh try | 08_3, 09_5 | "searches that are fresh tries of equal effort"; "The second is a fresh try" | "each search an independent try" in the description |
| row by row | 08_4 | "One plan has no map, and goes row by row." | never "the sweep" |
| track; pass | 09_2, 09_5 | "enough track to pass over a cell once"; "The first pass catches six dots of nine." | "coverage = strip × track ÷ area" at 10_4; the random search formula at 09_4 |
| drift | 10_2 | "The map drifts, and it spreads." | "leeway" once, at 10_1 |

## Simplification → exact

| Said | Exact | Where the exact form is |
|---|---|---|
| "the chance it is there, times the chance of seeing it" (05_6) | P(find in cell i) = pᵢ dᵢ, with dᵢ conditional on the wreck being in cell i | lower-third at 05_6, with the Coast Guard's POS = POC × POD |
| "Multiply each cell's chance by the chance that the search would come back empty if the wreck were there ... Then divide by what is left" (07_2) | pᵢ′ = pᵢ(1 − dᵢ) ÷ (1 − pᵢdᵢ) for the searched cell i; pⱼ′ = pⱼ ÷ (1 − pᵢdᵢ) for every other cell j: one denominator, the chance this search comes back empty | on screen at 07_2; as printed in 1971 at 13_1 |
| "Always search where the next search is most likely to find it ... no other order is ever ahead" (08_1, 08_3) | maximize pᵢdᵢ; with independent searches of fixed dᵢ, a target that does not move and equal costs, P[N > n] is least for every n (Chew 1967, Theorem 2.1, the number unverified: S11); with unequal costs, maximize pᵢdᵢ ÷ cᵢ for the least expected cost, the weaker result of Blackwell and Black 1965 | lower-third at 08_3; the description |
| "on average, sixty-three percent" (09_3) | 1 − e^(−WL/A) at WL = A, Koopman (1946), eq. (40), under his three assumptions; hundred-to-a-cell strips average 63.4 | the formula on the plot at 09_4 |
| "a strip only a tenth of a mile wide" (10_3) | sweep width 0.1 nautical mile: an effective width (the area under the lateral range curve), not a hard edge | lower-third at 10_3 |
| "The map drifts, and it spreads." (10_2) | each particle is moved by its own draw of wind and current, then reweighted; the cloud need not stay round | "model" on the cloud; the description |
| "By the map's own reckoning" (08_5) | the chance of a find within n searches, averaged over the prior map | the chart's label |
| "ninety-four went out" (13_2) | the factor 1 − E applied to that cell before the map is divided by 1 − SEP | the 1971 formula at 13_1; F6's three chances at 13_6 |
| "if the wreck were at any of those twenty, we would have it" (06_3) | a dot is a place together with this search's luck: of B's 30 equally likely ways, 20 are ones in which this search would see it; a wreck at any given place in B is seen two times in three. P(B is the answer | empty) = 10/80 either way | lower-third at 06_3 (review R2) |
| "the first map does not have to be right" (14_4) | for a stationary target, with every possible place given some chance and d > 0, the rule finds it with chance 1; the first map sets how long that takes. A place with no dots is never searched | caption and picture at 14_4; the description (review R2) |
| "take a patch of sea about twelve miles on a side" (02_5) | the lettered grid of Figure 2, 12 × 12.6 sea miles; 11% of the map's runs end outside it | label at 02_5 (review R2) |
| "what changed is what we know" (06_7) | the distribution is subjective: "The probability distribution on which the search is based is therefore a subjective one" (S3 p. 71) | lower-third at 06_7 |

## Always sentences and their edge tests

| Sentence (line) | Test | Range, and the edge |
|---|---|---|
| "Every cell we did not search grew by a quarter." (06_8) | `the_hand_example_goes_as_the_video_says`; `an_update_keeps_the_total_and_moves_chance_the_right_way` (`src/model.rs`) | the three unsearched cells, each × 100 ÷ 80; in general, after a search of a cell that had some chance and that can see (pᵢ > 0 and dᵢ > 0), every unsearched cell with some chance rises by the same factor 1 ÷ (1 − pᵢdᵢ); the factor is 1 if the searched cell had no chance or the search cannot see (R2: D19) |
| "no other order is ever ahead" (08_3) | `no_order_of_searches_beats_the_rule`; `the_rule_takes_the_biggest_slices_first`; `F-U.brute.rule_is_best`, `F-U.slices.rule_takes_biggest` | every order of 1 to 8 searches (65,536 at 8); the rule's first 12 searches against the 12 biggest slices; ties (row by row is level at 3, 5 and 6) are why the word is "ahead"; the three conditions are spoken |
| "a cell's slices only shrink" (08_2) | the same slices test; the oracle searches each cell alone eight times | strictly, for a chance of seeing strictly between 0 and 1 |
| "ten more passes with that sonar would have found nothing" (13_6) | `F-S.sweep.after` sonar 0 | holds on the paper's estimate of a sweep width of zero, which rests on the failure of one of the two sonars to detect her (Table 2, note a); "on that estimate" is spoken (review R2) |
| "Each pass finds less than the one before, and no number of passes makes it certain." (09_6) | `no_number_of_passes_makes_it_certain` | 1 to 20 passes at one in four, two in three and nine in ten; the edges are a search that cannot miss (certain in one) and passes that are not fresh tries (13_6) |
| "An empty search only tells you something if it could have found what you were looking for." (14_1) | `a_blind_search_changes_nothing_and_a_perfect_one_empties_its_cell` | a chance of seeing of zero leaves all four chances exactly as they were; a chance of one empties the cell |
| "A first map that leaves room for the truth." (14_4) | `a_cell_with_no_dots_never_gains_one`; `F-U.zero_stays_zero` | thirty empty searches of the two cells that have dots; the empty cells stay at exactly zero |
| every countdown is a quiz with the same seconds | `every_countdown_is_a_quiz_and_every_quiz_has_its_countdown` (`src/main.rs`); the oracle reads the storyboard too | all 5 |
| every number the script speaks | `the_script_s_spoken_numbers` (`src/facts.rs`) | 90 phrases, each with the fact it rests on |

## The numbers

Copied from `oracle.py`'s output by a script; `search facts` prints the same
keys and values.

### Every key

| key | value |
|---|---|
| `F-M.dots` | 100 |
| `F-M.story1.weight` | 60 |
| `F-M.story1.share` | 1/2, 1/3, 1/6, 0 |
| `F-M.story1.dots` | 30, 20, 10, 0 |
| `F-M.story2.weight` | 40 |
| `F-M.story2.share` | 1/4, 1/4, 1/4, 1/4 |
| `F-M.story2.dots` | 10, 10, 10, 10 |
| `F-M.prior.dots` | 40, 30, 20, 10 |
| `F-M.prior.percent` | 40, 30, 20, 10 |
| `F-M.seeing` | 1/4, 2/3, 2/3, 2/3 |
| `F-M.find.fraction` | 1/10, 1/5, 2/15, 1/15 |
| `F-M.find.percent` | 10.0, 20.0, 13.3, 6.7 |
| `F-M.likeliest.cell` | A |
| `F-M.likeliest.find.percent` | 10 |
| `F-M.first.cell` | B |
| `F-M.first.find.percent` | 20 |
| `F-M.first.over_likeliest` | 2 |
| `F-M.empty1.dots_out` | 20 |
| `F-M.empty1.dots_left` | 80 |
| `F-M.empty1.cell_dots_left` | 10 |
| `F-M.empty1.after.fraction` | 1/2, 1/8, 1/4, 1/8 |
| `F-M.empty1.after.percent` | 50.0, 12.5, 25.0, 12.5 |
| `F-M.empty1.rise` | 5/4 |
| `F-M.empty1.rise.decimal` | 1.25 |
| `F-M.second.find.fraction` | 1/8, 1/12, 1/6, 1/12 |
| `F-M.second.find.percent` | 12.5, 8.3, 16.7, 8.3 |
| `F-M.second.cell` | C |
| `F-M.empty2.after.percent` | 60, 15, 10, 15 |
| `F-M.empty2.found_by` | 1/3 |
| `F-M.third.find.percent` | 15.0, 10.0, 6.7, 10.0 |
| `F-M.third.cell` | A |
| `F-M.second.find.dots` | 10, 20/3, 40/3, 20/3 |
| `F-M.second.find.dots.rounded` | 10, 7, 13, 7 |
| `F-M.split.factor` | 3 |
| `F-M.split.lit` | 240 |
| `F-M.split.lit.cells` | 120, 30, 60, 30 |
| `F-M.split.second.dots_out` | 40 |
| `F-M.split.second.lit.cells` | 120, 30, 20, 30 |
| `F-M.third.dots_out` | 30 |
| `F-M.split.third.lit.cells` | 90, 30, 20, 30 |
| `F-M.fourth.cell` | A |
| `F-M.seen_within.1` | 2/3 |
| `F-M.seen_within.1.percent` | 66.7 |
| `F-M.seen_within.2` | 8/9 |
| `F-M.seen_within.2.percent` | 88.9 |
| `F-M.seen_within.3` | 26/27 |
| `F-M.seen_within.3.percent` | 96.3 |
| `F-U.best.order` | BCAABDACAAAB |
| `F-U.best.found_by.percent` | 20.0, 33.3, 43.3, 50.8, 57.5, 64.2, 69.8, 74.2, 78.5, 81.6, 84.0, 86.2 |
| `F-U.best.expected_searches` | 6.34 |
| `F-U.likeliest.order` | AABACAABDAAC |
| `F-U.likeliest.found_by.percent` | 10.0, 17.5, 37.5, 43.1, 56.5, 60.7, 63.8, 70.5, 77.2, 79.5, 81.3, 85.8 |
| `F-U.likeliest.expected_searches` | 7.04 |
| `F-U.likeliest_fresh.order` | ABAACAADBAAC |
| `F-U.likeliest_fresh.found_by.percent` | 10.0, 30.0, 37.5, 43.1, 56.5, 60.7, 63.8, 70.5, 77.2, 79.5, 81.3, 85.8 |
| `F-U.likeliest_fresh.expected_searches` | 6.92 |
| `F-U.sweep.order` | ABCDABCDABCD |
| `F-U.sweep.found_by.percent` | 10.0, 30.0, 43.3, 50.0, 57.5, 64.2, 68.6, 70.8, 76.5, 78.7, 80.2, 80.9 |
| `F-U.sweep.expected_searches` | 8.00 |
| `F-U.brute.searches` | 8 |
| `F-U.brute.orders` | 65536 |
| `F-U.brute.best` | 1/5, 1/3, 13/30, 61/120, 23/40, 77/120, 67/96, 1069/1440 |
| `F-U.brute.rule_is_best` | true |
| `F-U.slices.A` | 10.0, 7.5, 5.6, 4.2 |
| `F-U.slices.B` | 20.0, 6.7, 2.2, 0.7 |
| `F-U.slices.C` | 13.3, 4.4, 1.5, 0.5 |
| `F-U.slices.D` | 6.7, 2.2, 0.7, 0.2 |
| `F-U.best.dots_out` | 20, 40/3, 10, 15/2, 20/3, 20/3, 45/8, 40/9 |
| `F-U.best.dots_out.rounded` | 20.0, 13.3, 10.0, 7.5, 6.7, 6.7, 5.6, 4.4 |
| `F-U.slices.rule_takes_biggest` | true |
| `F-U.zero_stays_zero` | true |
| `F-R.replications` | 10000 |
| `F-R.total` | 10000 |
| `F-R.printed_cells` | 172 |
| `F-R.rows` | 13, 58, 124, 214, 380, 804, 3232, 653, 2648, 1090, 395, 261, 88, 29, 6, 5 |
| `F-R.columns` | 1, 416, 582, 302, 1304, 1543, 604, 1632, 613, 856, 633, 434, 339, 237, 190, 131, 86, 71, 22, 4 |
| `F-R.shading` | 80, 71, 19, 2 |
| `F-R.E5` | 1250 |
| `F-R.E5.percent` | 12.5 |
| `F-R.E5.one_in` | 8 |
| `F-R.B7` | 1096 |
| `F-R.top_two.percent` | 23 |
| `F-R.F6` | 85 |
| `F-R.F6.percent` | 0.85 |
| `F-R.F6.rank` | 24 |
| `F-R.E5.rank` | 1 |
| `F-R.E5_over_F6` | 14.7 |
| `F-R.grid.cells` | 180 |
| `F-R.cell.miles` | 1x0.84 |
| `F-R.grid.area` | 151.2 |
| `F-R.grid.miles` | 12x12.6 |
| `F-R.grid.points` | 8900 |
| `F-R.outside.points` | 1100 |
| `F-R.grid.empty_cells` | 40 |
| `F-R.model.seeing` | 2/3 |
| `F-R.model.best.first_cells` | E5, B7, C5, G5, E5, B7 |
| `F-R.model.best.found_by.20` | 49.8 |
| `F-R.model.best.found_by.81` | 73.2 |
| `F-R.model.once.found_by.20` | 44.3 |
| `F-R.model.once.found_by.81` | 57.8 |
| `F-R.model.sweep.found_by.20` | 0.8 |
| `F-R.model.sweep.found_by.81` | 32.9 |
| `F-R.model.search_miles` | 28 |
| `F-R.model.random.found_by.20` | 6.6 |
| `F-R.model.random.found_by.81` | 26.7 |
| `F-R.model.sweep.first20.points` | 114 |
| `F-R.model.F6.first_search` | 30 |
| `F-H.track.miles` | 1026 |
| `F-H.track.days_at_one_knot` | 43 |
| `F-H.strip.feet` | 200 |
| `F-H.feet_per_sea_mile` | 6076 |
| `F-H.feet_per_sea_mile.exact` | 6076.1 |
| `F-H.once.miles` | 4594 |
| `F-H.once.hundreds` | 46 |
| `F-H.once.days_at_one_knot` | 191 |
| `F-H.once.over_track` | 4.5 |
| `F-H.once.miles.camera` | 4966 |
| `F-H.once.miles.magnetometer` | 4176 |
| `F-H.once.miles.believed` | 1148 |
| `F-H.once.per_cell` | 26 |
| `F-H.once.per_cell.exact` | 25.5 |
| `F-H.once.passes_per_cell` | 26 |
| `F-H.track.cells` | 40 |
| `F-H.track.share.percent` | 22 |
| `F-K.formula.half` | 39.3 |
| `F-K.perfect.half` | 50 |
| `F-K.formula.1` | 63.2 |
| `F-K.perfect.1` | 100 |
| `F-K.formula.2` | 86.5 |
| `F-K.perfect.2` | 100 |
| `F-K.formula.3` | 95.0 |
| `F-K.perfect.3` | 100 |
| `F-K.formula.1.unseen` | 36.8 |
| `F-K.coverage.flat` | 1.10 |
| `F-K.coverage.canyon` | 0.29 |
| `F-K.lanes.seed` | 1946 |
| `F-K.lanes.side` | 200 |
| `F-K.lanes.share.percent` | 1 |
| `F-K.lanes.per_coverage` | 100 |
| `F-K.lanes.runs` | 1000 |
| `F-K.lanes.one_run.1` | 66.5 |
| `F-K.lanes.mean.1` | 63.4 |
| `F-K.lanes.min.1` | 57.1 |
| `F-K.lanes.max.1` | 69.9 |
| `F-K.lanes.exact.1` | 63.4 |
| `F-K.lanes.one_run.2` | 87.6 |
| `F-K.lanes.mean.2` | 86.7 |
| `F-K.lanes.min.2` | 81.9 |
| `F-K.lanes.max.2` | 91.3 |
| `F-K.lanes.exact.2` | 86.6 |
| `F-K.lanes.one_run.3` | 95.3 |
| `F-K.lanes.mean.3` | 95.1 |
| `F-K.lanes.min.3` | 92.5 |
| `F-K.lanes.max.3` | 97.4 |
| `F-K.lanes.exact.3` | 95.1 |
| `F-K.lanes.first` | v149, v190, h161, h182 |
| `F-D.seed` | 2013 |
| `F-D.dots` | 2000 |
| `F-D.start_radius` | 1 |
| `F-D.drift` | 1.5 |
| `F-D.error_share` | 0.3 |
| `F-D.track` | 180 |
| `F-D.strip` | 0.1 |
| `F-D.strip.metres` | 185 |
| `F-D.strip.feet` | 608 |
| `F-D.strip.altitude.feet` | 300 |
| `F-D.swept` | 18 |
| `F-D.h1.radius` | 1.45 |
| `F-D.h1.area` | 6.6 |
| `F-D.h1.coverage` | 2.73 |
| `F-D.h1.seeing` | 93 |
| `F-D.h1.farthest` | 1.43 |
| `F-D.h3.radius` | 2.35 |
| `F-D.h3.area` | 17.3 |
| `F-D.h3.coverage` | 1.04 |
| `F-D.h3.seeing` | 65 |
| `F-D.h3.farthest` | 2.32 |
| `F-D.h6.radius` | 3.70 |
| `F-D.h6.area` | 43.0 |
| `F-D.h6.coverage` | 0.42 |
| `F-D.h6.seeing` | 34 |
| `F-D.h6.farthest` | 3.65 |
| `F-D.h12.radius` | 6.40 |
| `F-D.h12.area` | 128.7 |
| `F-D.h12.coverage` | 0.14 |
| `F-D.h12.seeing` | 13 |
| `F-D.h12.farthest` | 6.33 |
| `F-D.flight.h3.best_side` | 3 |
| `F-D.flight.h3.inside` | 1689 |
| `F-D.flight.h3.seeing` | 86 |
| `F-D.flight.h3.find` | 73 |
| `F-D.flight.h3.find_by_side` | 15, 52, 73, 67, 51, 39, 31, 25 |
| `F-D.flight.h6.best_side` | 5 |
| `F-D.flight.h6.inside` | 1707 |
| `F-D.flight.h6.seeing` | 51 |
| `F-D.flight.h6.find` | 44 |
| `F-D.flight.h6.find_by_side` | 4, 16, 32, 43, 44, 38, 31, 25 |
| `F-D.double.area_factor` | 4 |
| `F-D.double.coverage_factor` | 1/4 |
| `F-S.crew` | 99 |
| `F-S.last_message` | 1968-05-21 |
| `F-S.sank` | 1968-05-22 |
| `F-S.due` | 1968-05-27 |
| `F-S.presumed_lost` | 1968-06-05 |
| `F-S.found` | 1968-10-28 |
| `F-S.days.sank_to_due` | 5 |
| `F-S.days.due_to_presumed` | 9 |
| `F-S.days.due_to_found` | 154 |
| `F-S.months.due_to_found` | 5 |
| `F-S.search_began` | 1968-06-10 |
| `F-S.scenarios_set` | 1968-07-18 |
| `F-S.scenarios` | 9 |
| `F-S.map_arrived` | 1968-08-12 |
| `F-S.days.search_to_map` | 63 |
| `F-S.months.search_to_map` | 2 |
| `F-S.days.map_to_found` | 77 |
| `F-S.cruises` | 5 |
| `F-S.yards` | 260 |
| `F-S.yards.metres` | 238 |
| `F-S.miles_from_azores` | 400 |
| `F-S.depth.feet` | 10000 |
| `F-S.depth.metres` | 3048 |
| `F-S.depth.miles` | 1.9 |
| `F-S.forecast.days` | 35-45 |
| `F-S.bottom_days` | 43 |
| `F-S.miles_to_reach_F6` | 153 |
| `F-S.update.page` | 146 |
| `F-S.sweep.at_sea` | 800, 400 |
| `F-S.sweep.after` | 185, 220, 0 |
| `F-S.sweep.magnetometer.ratio` | 1.8 |
| `F-S.F6.searched.at_sea` | 65, 94, 99 |
| `F-S.F6.searched.after` | 47, 65, 74 |
| `F-S.F6.left.at_sea.percent` | 6 |
| `F-S.F6.left.after.percent` | 35 |
| `F-S.F6.left.ratio` | 5.8 |
| `F-S.SEP.at_sea` | 40, 73, 75 |
| `F-S.SEP.after` | 29, 45, 48 |
| `F-S.F6.chance.at_sea` | 0.19 |
| `F-S.F6.chance.after` | 0.54 |
| `F-S.F6.coverage.at_sea` | 2.81 |
| `F-S.F6.coverage.after` | 1.05 |
| `F-P.date` | 1966-01-17 |
| `F-P.bombs` | 4 |
| `F-P.on_land` | 3 |
| `F-P.in_sea` | 1 |
| `F-P.airmen` | 7 of 11 |
| `F-P.found` | 1966-03-15 |
| `F-P.recovered` | 1966-04-07 |
| `F-P.days.to_found` | 57 |
| `F-P.days.to_recovered` | 80 |
| `F-P.years_before_scorpion` | 2 |
| `F-P.depth.feet` | 2550 |
| `F-P.depth.metres` | 777 |
| `F-A.lost` | 2009-06-01 |
| `F-A.found` | 2011-04-02 |
| `F-A.aboard` | 228 |
| `F-A.months` | 22 |
| `F-A.days` | 670 |
| `F-A.circle.nm` | 40 |
| `F-A.circle.km` | 74 |
| `F-A.circle.area_km2` | 17241 |
| `F-A.weights` | 35, 35, 30 |
| `F-A.weights.sum` | 100 |
| `F-A.points` | 75000 |
| `F-A.empty_searches` | 4 |
| `F-A.underwater_searches` | 3 |
| `F-A.underwater_searches.years` | 2009,2009,2010 |
| `F-A.beacon.independent` | 0.9216 |
| `F-A.beacon.independent.rounded` | 0.92 |
| `F-A.beacon.dependent` | 0.72 |
| `F-A.beacon.used.percent` | 77 |
| `F-A.beacon.left.percent` | 23 |
| `F-A.found.nm` | 6.5 |
| `F-A.found.km` | 12 |
| `F-A.found.share_of_radius` | 16 |
| `F-A.depth.metres` | 3900 |
| `F-C.casp` | 1974 |
| `F-C.sarops` | 2007 |
| `F-C.sarops.dots` | 2500, 5000, 10000 |
| `F-C.no_search.percent` | 90 |
| `F-C.aldridge.date` | 2013-07-24 |
| `F-C.aldridge.fell` | 03:30 |
| `F-C.aldridge.seen` | 14:58 |
| `F-C.aldridge.in_water` | 11h28 |
| `F-C.aldridge.in_water.hours` | 11.5 |
| `F-C.aldridge.rerun` | 08:30 |
| `F-C.aldridge.rerun.hours_after_fall` | 5 |
| `F-C.aldridge.guess` | 21:30-23:30 |
| `F-C.aldridge.guess.middle` | 22:30 |
| `F-C.aldridge.hours_off` | 5 |
| `F-C.aldridge.miles_south` | 30 |
| `F-C.aldridge.miles_offshore` | 40-43 |
| `F-N.koopman` | 1946 |
| `F-N.bayes.died` | 1761 |
| `F-N.bayes.read` | 1763 |
| `F-N.laplace` | 1774 |
| `F-N.chew` | 1967 |
| `F-N.paper` | 1971 |
| `F-N.paper.years_after` | 3 |
| `F-M.code.1` | `p[i] *= 1 - d[i]                       # the search of cell i came back empty` |
| `F-M.code.2` | `t = sum(p); p = [x / t for x in p]     # the map adds up to one again` |
| `F-M.code.3` | `find = [x * y for x, y in zip(p, d)]   # chance of a find, cell by cell` |
| `F-M.code.4` | `i = find.index(max(find))              # search there next` |
| `F-Q.count` | 5 |
| `F-Q.names` | stories, first_search, after_empty, next_search, two_searches |
| `F-Q.stories.line` | 03_5 |
| `F-Q.stories.seconds` | 6 |
| `F-Q.stories.answer` | 40 |
| `F-Q.first_search.line` | 05_4 |
| `F-Q.first_search.seconds` | 6 |
| `F-Q.first_search.answer` | B |
| `F-Q.after_empty.line` | 06_5 |
| `F-Q.after_empty.seconds` | 6 |
| `F-Q.after_empty.answer` | 12.5 |
| `F-Q.next_search.line` | 07_3 |
| `F-Q.next_search.seconds` | 6 |
| `F-Q.next_search.answer` | C |
| `F-Q.two_searches.line` | 09_5 |
| `F-Q.two_searches.seconds` | 6 |
| `F-Q.two_searches.answer` | 8 |

### Research rows (oracle only: the F-T family)

Numbers the published description states (`docs/description.md` names each
in a bracket) and the video neither shows nor says, so `search facts` does
not print them; `facts-check` allows an oracle that prints more. Each is
computed in `oracle.py` from first principles by the oracle's own functions
and cross-checked there (351 oracle keys, 97 cross-checks, 0 failed). Five
only write a fact the video prints in the decimals the description's code and
quotations use: `F-T.four_cell.prior.decimal`,
`F-T.air_france.weights.decimal`, `F-T.f6.searched.oct9.*` and
`F-T.code.t_first`.

| key | value | what it is |
|---|---|---|
| `F-T.four_cell.prior.decimal` | 0.40, 0.30, 0.20, 0.10 | the code's `p`, the map in decimals |
| `F-T.four_cell.find.mixed` | 10, 20, 13 1/3, 6 2/3 | the chances of a find in a hundred, as exact mixed numbers (F-M.find.percent rounds them) |
| `F-T.small_sea.level_with_row_by_row.searches` | 3, 5, 6 | the searches after which row by row has found exactly what the rule has, checked to 200 searches |
| `F-T.air_france.weights.decimal` | 0.35, 0.35, 0.30 | the three AF 447 weights in decimals |
| `F-T.code.t_first` | 0.8 | the code's first `t`, the dots left of the hundred |
| `F-T.f6.searched.oct9.on_scene` | 0.94 | F6 on 9 October, computed on scene (Table 1) |
| `F-T.f6.searched.oct9.afterwards` | 0.65 | F6 on 9 October, estimated afterwards (Table 3) |
| `F-T.f6.coverage.ratio` | 2.7 | the coverage fell by this factor (2.68 unrounded) |
| `F-T.f6.dots.give_or_take` | 9 | F6's 85 dots of 10,000 runs: the binomial spread (9.18) |
| `F-T.lanes.one_run_above_mean` | 3.1 | the shown run above the mean of the thousand, in points (3.05) |
| `F-T.lanes.sd.thousand_runs` | 1.75 | standard deviation of the thousand runs, in points |
| `F-T.lanes.one_run_sds_above.thousand_runs` | 1.7 | the shown run, in standard deviations of the thousand runs |
| `F-T.lanes.sd.exact` | 1.72 | the exact standard deviation of one run (exact fractions; listing every way to lay lanes on a small cell, and 400,000 simulated runs, agree) |
| `F-T.lanes.one_run_sds_above.exact` | 1.8 | the shown run, in the exact standard deviations |
| `F-T.model.over_grid.factor` | 1.12 | 10,000 ÷ 8,900, the runs inside the lettered grid |
| `F-T.model.sweep.over_grid.found_by.20` | 0.9 | row by row, twenty searches, over the 180 cells alone (percent) |
| `F-T.model.once.over_grid.found_by.20` | 49.8 | the map never updated, over the 180 cells alone |
| `F-T.model.best.over_grid.found_by.20` | 55.9 | the rule, over the 180 cells alone |
| `F-T.model.sweep_from_row5.found_by.20` | 22.3 | row by row from row 5, twenty searches (percent of all runs) |
| `F-T.drift.double_radius.hours` | 1, 3, 6, 12 | |
| `F-T.drift.double_radius.seeing` | 49, 23, 10, 3 | a circle twice as wide, the same flight: percent seen at those hours (no longer cited in the description) |
| `F-T.drift.coast_guard.total_probable_error` | 1.10, 1.68, 2.88, 5.49 | E in miles at 1, 3, 6 and 12 hours: the root sum of squares of the start's 1 mile and 0.3 × 1.5 mi/h × hours, both as probable errors (S13, S17) |
| `F-T.drift.coast_guard.first_search.half_side_in_E` | 1.1 | the first search's square, in units of E (S13 Table 4-2, typed from the source) |
| `F-T.drift.coast_guard.first_search.poc` | 0.64759 | the share of a circular normal spread (E = σ√(2 ln 2) = 1.17741σ) inside that square; S13 prints 0.64762, which matches E = 1.17745σ |
| `F-T.drift.coast_guard.first_search.poc_percent` | 65 | the same, in a hundred |
| `F-T.drift.coast_guard.first_search.seeing` | 95, 73, 36, 12 | percent seen inside that square by 180 miles × 0.1 mile of track at 1, 3, 6 and 12 hours, random search |
| `F-T.drift.coast_guard.circle.holds_in_100` | 95, 99 | the shares of the spread two comparison circles hold |
| `F-T.drift.coast_guard.circle_95.radius_in_E` | 2.08 | the radius, in E, of the circle that holds 95 in 100 |
| `F-T.drift.coast_guard.circle_95.seeing` | 67, 37, 15, 4 | percent seen inside it at those hours |
| `F-T.drift.coast_guard.circle_99.radius_in_E` | 2.58 | the radius, in E, of the circle that holds 99 in 100 |
| `F-T.drift.coast_guard.circle_99.seeing` | 51, 26, 10, 3 | percent seen inside it at those hours (the video's own disc: F-D.h*.seeing, 93, 65, 34, 13) |
