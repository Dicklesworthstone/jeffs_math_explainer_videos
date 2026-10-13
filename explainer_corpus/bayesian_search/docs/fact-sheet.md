# Fact sheet: Video H, "How to Find a Lost Submarine"

Bead `mx-e-a-h-6ix2.2` (gate G1). One row per number the video may show or
say, and one row per claim that rests on a source. Every value in the table
at the end was copied from the oracle's output by a script, not typed.

```
./x bayesian_search facts-check        # search facts == oracle.py, key by key
```

- **Result, 2026-10-13** (the build host; CPython 3.14, NumPy via uv), with
  the storyboard: `facts-check bayesian_search: 274 video facts against 274
  oracle keys: PASS`. The oracle passes 38 of 38 of its own cross-checks.
  (At G1, before the storyboard: 263 of 263.)
- **Rust side:** `./x bayesian_search test`: 19 tests pass, clippy
  `-D warnings` and `fmt --check` clean.
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
| the model search over that map | a scan for the cell with the most chance left | a sorted list of (chance left, place in reading order) |
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
   the plan was the BEA's, a systematic search from the centre outward.
7. **The Aldridge search.** The Coast Guard's own releases never mention the
   planning software; its part is as reported by the *New York Times
   Magazine* from the command centre. The helicopter's last track was drawn
   by hand through the high-chance area after the software froze ("it
   wasn't Sarops-generated"), and Aldridge was not drifting freely: he had
   tied himself to lobster buoys. The video says the first map was wrong
   because its inputs were, that new evidence moved it about thirty miles,
   and that a helicopter flying through the new area saw him; it does not
   say the software found him.
8. **In the small sea the rule's lead is modest, and the script says no
   more.** On average the rule takes 6.34 searches, a plain sweep 8.00, and
   "the likeliest cell first" 7.04 (6.92 with the other tie-break); after 3,
   5 and 6 searches the sweep has found exactly as much as the rule. The
   theorem is "never behind", and that is what is said. The large gap is on
   the 1968 map, where the chance is spread over 172 cells: after 20 model
   searches the rule has found the wreck with chance 52.7% and a row-by-row
   sweep with 0.8%.
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
12. **The rule is proved for a thing that stays put.** Chew's theorem needs
    one object that does not move, independent searches with a fixed chance
    of seeing, and equal costs. For a drifting target the video says only
    that the map must be moved before each update, and makes no claim that
    the simple rule stays best (Brown 1980 treats that problem).

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
| S11 | M. C. Chew, Jr., "A Sequential Search Procedure", *Annals of Mathematical Statistics* 38(2), 1967, pp. 494-502 | pp. 494-496 as page images | the theorem behind "no order does better" |
| S12 | W. L. Black, "Discrete Sequential Search", *Information and Control* 8, 1965, pp. 159-162; J. B. Kadane, Cowles Foundation Discussion Paper 220 (1967) | in full | the rule with unequal costs; why a fixed budget with unequal costs is a different problem |
| S13 | J. R. Frost and L. D. Stone, *Review of Search Theory: Advances and Applications to Search and Rescue Decision Support*, USCG report CG-D-15-01 (2001) | chapters 1 to 4 | Koopman's group in the war; the drift error as three tenths of the distance drifted; the flat-topped posterior |
| S14 | T. Bayes, "An Essay towards solving a Problem in the Doctrine of Chances", *Philosophical Transactions* 53 (read 23 December 1763), pp. 370-418; S. E. Fienberg (2006); S. M. Stigler (1986); *Oxford DNB* | pp. 370-376 of the essay; the cited pages of the others | Bayes died in 1761; the essay was sent by Richard Price; the rule in the form used here is Laplace's (1774) |
| S15 | J. P. Craven, *The Silent War* (2001); S. Sontag and C. Drew, *Blind Man's Bluff* (1998); S. B. McGrayne, *The Theory That Would Not Die* (2011) | paragraphs or search fragments of library scans only | point of view; never a quotation and never a number |
| S16 | The owner's reference: *The Search for the USS Scorpion: A Modern Application of Bayesian Statistics* (tomrocksmaths.com, 2023), a student essay | in full | the map of the topic; its claims are tabled below |

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
| strip | 02_5 | "the sled that carried the search gear saw a strip only two hundred feet wide" | the strip drawn to scale; "sweep width" once, at 10_3 |
| map; cell | 03_1, 03_2 | "a map of how likely each place is"; "a sea of four cells" | the grid; "prior" once, at 03_8 |
| story; weight | 03_2, 03_5 | "two stories of what might have happened"; "People who know submarines weigh the stories" | "scenario" and "credence" once, at 04_1 |
| dot | 03_5 | "Each dot is one place she could be." | "replication" at 04_2 and "particle" at 11_1, once each |
| chance | 03_6 | "What is the chance she is in cell A?" | "probability" only in lower-thirds |
| the chance of seeing her | 05_2 | "a search there sees the wreck only one time in four" | "seen 1 in 4"; "detection probability" once, at 05_6 |
| the chance of a find | 05_3, 05_6 | "How likely is one search of a cell to find her?"; "the chance she is there, times the chance of seeing her" | p × d with word labels |
| comes back empty; miss | 06_1, 06_4 | "the search comes back empty"; "because a search can miss" | "empty"; never swapped |
| update; Bayes' rule | 07_1, 07_6 | "That move has a name. Bayes' rule"; "Update." | "prior" and "posterior" at 07_1 |
| lanes; track; pass | 09_3, 09_4 | "enough track to pass over a cell once"; "Each pass finds less than the one before." | "coverage" and the random search formula at 09_4 |
| drift | 10_2 | "The map drifts, and it spreads." | "leeway" once, at 10_1 |

## Simplification → exact

| Said | Exact | Where the exact form is |
|---|---|---|
| "the chance she is there, times the chance of seeing her" (05_6) | P(find in cell i) = pᵢ dᵢ, with dᵢ conditional on the wreck being in cell i | lower-third at 05_6, with the Coast Guard's POS = POC × POD |
| "The cell you searched drops. Every other cell rises." (07_3) | pᵢ′ = pᵢ(1 − dᵢ) ÷ (1 − pᵢdᵢ); pⱼ′ = pⱼ ÷ (1 − pᵢdᵢ) | on screen at 07_3; as printed in 1971 at 13_1 |
| "Always search where the next search is most likely to find her." (08_1) | maximize pᵢdᵢ; with equal costs P[N > n] is least for every n (Chew 1967, Theorem 2.1); with unequal costs, maximize pᵢdᵢ ÷ cᵢ for the least expected cost (Blackwell; Black 1965) | lower-third at 08_2; the description |
| "on average the same track covers sixty-three percent" (09_3) | 1 − e^(−WL/A) at WL = A, Koopman (1946), eq. (40), under his three assumptions; hundred-to-a-cell lanes average 63.4 | lower-third at 09_4 |
| "a strip only a tenth of a mile wide" (10_3) | sweep width 0.1 nautical mile: an effective width (the area under the lateral range curve), not a hard edge | lower-third at 10_3 |
| "The map drifts, and it spreads." (10_2) | each particle is moved by its own draw of wind and current, then reweighted; the cloud need not stay round | "model" on the cloud; the description |
| "ninety-four went out" (13_2) | the factor 1 − E applied to that cell before the map is divided by 1 − SEP | the 1971 formula at 13_1 |
| "say what you believe" (14_7) | the distribution is subjective: "The probability distribution on which the search is based is therefore a subjective one" (S3 p. 71) | the description |

## Always sentences and their edge tests

| Sentence (line) | Test | Range, and the edge |
|---|---|---|
| "Every cell we did not search went up, by the same factor." (06_8) | `an_update_keeps_the_total_and_moves_chance_the_right_way` (`src/model.rs`); the oracle's "every unsearched cell rises by the same factor" | fifteen updates in a row over all four cells; the factor is 1 ÷ (1 − pᵢdᵢ), which is 1 only for a search that cannot see |
| "The cell you searched drops. Every other cell rises." (07_3) | the same test | the searched cell falls strictly while its chance of being seen is above zero |
| "No other order of searches is ever ahead." (08_2) | `no_order_of_searches_beats_the_rule`; `F-U.brute.rule_is_best` | every order of 1 to 8 searches (65,536 at 8); the three other rules over 12 searches; ties (the sweep is level at 3, 5 and 6) are why the word is "ahead" |
| "Each pass finds less than the one before." (09_4) | `lanes_sweep_what_the_formula_says_on_average`; `F-K.formula.*` | the formula's gains 63.2, 23.3, 8.5; a single seeded run can break it lane by lane, so the sentence sits on the formula's curve, not the run's |
| "No number of passes makes it certain." (09_6) | `no_number_of_passes_makes_it_certain` | 1 to 20 passes at one in four, two in three and nine in ten; the edge is a search that cannot miss, which is certain in one |
| "An empty search only tells you something if it could have found what you were looking for." (13_6) | `a_blind_search_changes_nothing_and_a_perfect_one_empties_its_cell` | a chance of seeing of zero leaves all four chances exactly as they were; a chance of one empties the cell |
| every countdown is a quiz with the same seconds | `every_countdown_is_a_quiz_and_every_quiz_has_its_countdown` (`src/main.rs`); the oracle reads the storyboard too | all 6 |
| every number the script speaks | `the_script_s_spoken_numbers` (`src/facts.rs`) | 85 phrases, each with the fact it rests on |

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
| `F-R.model.seeing` | 2/3 |
| `F-R.model.best.first_cells` | E5, B7, C5, G5, E5, B7 |
| `F-R.model.best.F6_search` | 35 |
| `F-R.model.sweep.F6_search` | 81 |
| `F-R.model.best.found_by.10` | 38.1 |
| `F-R.model.sweep.found_by.10` | 0.0 |
| `F-R.model.best.found_by.20` | 52.7 |
| `F-R.model.sweep.found_by.20` | 0.8 |
| `F-R.model.best.found_by.35` | 64.0 |
| `F-R.model.sweep.found_by.35` | 2.1 |
| `F-R.model.best.found_by.81` | 79.8 |
| `F-R.model.sweep.found_by.81` | 32.9 |
| `F-H.track.miles` | 1026 |
| `F-H.track.days_at_one_knot` | 43 |
| `F-H.strip.feet` | 200 |
| `F-H.feet_per_sea_mile` | 6076 |
| `F-H.once.miles` | 4594 |
| `F-H.once.hundreds` | 46 |
| `F-H.once.days_at_one_knot` | 191 |
| `F-H.once.over_track` | 4.5 |
| `F-H.once.miles.camera` | 4966 |
| `F-H.once.miles.magnetometer` | 4176 |
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
| `F-K.lanes.exact.1` | 63.4 |
| `F-K.lanes.one_run.2` | 87.6 |
| `F-K.lanes.mean.2` | 86.7 |
| `F-K.lanes.exact.2` | 86.6 |
| `F-K.lanes.one_run.3` | 95.3 |
| `F-K.lanes.mean.3` | 95.1 |
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
| `F-S.sweep.at_sea` | 800, 400 |
| `F-S.sweep.after` | 185, 220, 0 |
| `F-S.sweep.magnetometer.ratio` | 1.8 |
| `F-S.F6.searched.at_sea` | 65, 94, 99 |
| `F-S.F6.searched.after` | 47, 65, 74 |
| `F-S.F6.left.at_sea.percent` | 6 |
| `F-S.F6.left.after.percent` | 35 |
| `F-S.F6.left.ratio` | 5.8 |
| `F-P.date` | 1966-01-17 |
| `F-P.bombs` | 4 |
| `F-P.on_land` | 3 |
| `F-P.in_sea` | 1 |
| `F-P.airmen` | 7 of 11 |
| `F-P.found` | 1966-03-15 |
| `F-P.recovered` | 1966-04-07 |
| `F-P.days.to_found` | 57 |
| `F-P.days.to_recovered` | 80 |
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
| `F-Q.count` | 6 |
| `F-Q.names` | stories, first_search, after_empty, next_search, two_searches, radius_doubles |
| `F-Q.stories.line` | 03_6 |
| `F-Q.stories.seconds` | 6 |
| `F-Q.stories.answer` | 40 |
| `F-Q.first_search.line` | 05_4 |
| `F-Q.first_search.seconds` | 6 |
| `F-Q.first_search.answer` | B |
| `F-Q.after_empty.line` | 06_5 |
| `F-Q.after_empty.seconds` | 6 |
| `F-Q.after_empty.answer` | 12.5 |
| `F-Q.next_search.line` | 07_4 |
| `F-Q.next_search.seconds` | 6 |
| `F-Q.next_search.answer` | C |
| `F-Q.two_searches.line` | 09_5 |
| `F-Q.two_searches.seconds` | 5 |
| `F-Q.two_searches.answer` | 8/9 |
| `F-Q.radius_doubles.line` | 10_4 |
| `F-Q.radius_doubles.seconds` | 5 |
| `F-Q.radius_doubles.answer` | 1/4 |
