<!--
Draft video description for Video H (written for review round R2, 2026-10-13).
Every statement below rests on a row of docs/fact-sheet.md or an oracle key;
the bracketed tags name them. It holds the exact versions of what the
narration simplifies (plan rule 11), the programmer notes that have no room
on screen, and what review round R1 sent "to the description" (R19, R25, D2,
D9 to D11, D13, D14). The release package adds the chapter timestamps and
the repository link, and trims this to what the upload page accepts, linking
this file for the rest.
-->

# How to Find a Lost Submarine

A nuclear submarine has gone missing and time is running out. How do you
look for something you cannot see, in an ocean you cannot search all of?
The answer is a map of where it could be, a rule for where to look next, and
one move that turns "we found nothing" into new knowledge. The video builds
that move by counting dots, then shows it at work in four real searches:
the hydrogen bomb lost at Palomares, the USS *Scorpion*, Air France 447 and
the Coast Guard's search for a lobsterman who fell overboard.

Five "your turn" quizzes with a countdown. Pause whenever you want more
time.

The *Scorpion* was lost on 22 May 1968 with all 99 men aboard. The Navy
searched from 27 May to 5 June as if they might still be saved; they could
not be. Nothing in the video says otherwise. [F-S.sank, F-S.crew, F-S.due, F-S.presumed_lost]

## Chapters

(Timestamps come with the finished cut.)

1. The question
2. Ninety-nine
3. A map of chances
4. Ten thousand dots
5. A search can miss
6. Nothing found
7. Bayes' rule
8. Where next
9. The second pass
10. Adrift
11. Twelve hours
12. The silence
13. How well they looked
14. Four maps

## The method, exactly

Everything in chapters 03 to 08 is one idea, counted with dots. Here it is
in symbols, with its edges.

- **The setting.** One lost thing, in one of N cells, that stays where it
  is. Cell i holds it with chance pᵢ (the pᵢ add to 1). A search of cell i,
  if the thing is there, finds it with chance dᵢ, the "chance of seeing it":
  a number between 0 and 1 that is *conditional on the thing being in the
  cell*. Each search is a fresh try: independent of the others, with the
  same dᵢ every time [F-U.*, F-M.seeing].
- **The chance of a find** from one search of cell i is pᵢdᵢ: the chance it
  is there times the chance of seeing it. In the video's four-cell sea
  (A, B, C, D hold 40, 30, 20 and 10 of 100 dots; d = 1/4, 2/3, 2/3, 2/3)
  that is 10, 20, 13⅓ and 6⅔ in a hundred, so the best first search is B,
  not the likeliest cell A [F-M.prior.dots, F-M.seeing, F-M.find.percent,
  F-T.four_cell.find.mixed].
- **The update after an empty search of cell i:**
  pᵢ′ = pᵢ(1 − dᵢ) ÷ (1 − pᵢdᵢ), and pⱼ′ = pⱼ ÷ (1 − pᵢdᵢ) for every other
  cell j. One denominator: 1 − pᵢdᵢ is the chance that this search comes
  back empty. This is Bayes' rule, in the form Richardson and Stone printed
  in 1971 (L′ⱼ = Lⱼ(1 − Eⱼ) ÷ (1 − SEP)). It is the dot-counting of chapter
  06: 100 dots, 20 go out, 80 are left, B has 10 of them [F-M.empty1.*, F-N.paper, F-M.dots].
- **Where it holds.** "Every cell we did not search rises by the same
  factor 1 ÷ (1 − pᵢdᵢ)" is true for every unsearched cell that has some
  chance, after a search of a cell that had some chance (pᵢ > 0) and that
  can see (dᵢ > 0); the factor is 1 otherwise. A search that cannot see
  (dᵢ = 0) changes nothing, and one that cannot miss (dᵢ = 1) empties its
  cell [tests `a_blind_search_changes_nothing…`].
- **The update never adds.** A cell that holds no dots keeps none, whatever
  is searched: its chance is multiplied by a number and stays zero. So the
  first map must leave room for the truth (14_4), and every story in the
  video was put right *from outside* the loop: widths measured again, a
  second map, new evidence from the boat [F-U.zero_stays_zero].
- **The rule.** Search the cell with the greatest pᵢdᵢ (after each update).
  For one thing that does not move, searches that are independent tries
  with a known, fixed dᵢ, and equal cost per search, this is optimal in the
  strongest sense: for every n, the chance of *still not having found it
  after n searches* is as small as any plan can make it (Chew 1967, "A
  Sequential Search Procedure", Theorem 2.1; Blackwell and Black 1965 had
  the weaker result, the least expected cost, for unequal costs too, by the
  rule pᵢdᵢ ÷ cᵢ). The video says the three conditions it
  can say in a breath (08_3, 09_5); the two it leaves to this page are that
  the chance of seeing is *known*, and that moving between cells costs
  nothing. A canyon is the classic place where the second fails to hold in
  practice: its shadow hides the same spot from every pass from the same
  side, so repeated passes from one side are not fresh tries. Change the
  angle or the sensor, not only the count (chapter 13's sonar is the same
  failure in its extreme form). [F-N.chew]
- **The proof in slices (08_2, 08_3).** The k-th search of cell i puts out
  pᵢdᵢ(1 − dᵢ)ᵏ⁻¹ of the *original* chance, whatever was searched in
  between, and these slices shrink as k grows. The chance of a find within n
  searches is the sum of the n slices taken. So the best n searches are the n
  largest slices, which are available in that order because each cell's
  slices only shrink; and that is what the rule takes, one search at a time
  [F-U.slices.*]. The oracle also tries every order of up to 8 searches
  (65,536 at 8) and finds none ahead of the rule [F-U.brute.*]. Ties are why
  the video says "ahead": row by row is level with the rule after 3, 5 and 6
  searches of the small sea [F-T.small_sea.level_with_row_by_row.searches].
- **Unequal costs.** With a cost cᵢ per search, rank by pᵢdᵢ ÷ cᵢ for the
  least expected cost to find (Black 1965); with a fixed budget and unequal
  costs the problem is different (Kadane 1967). Moving between cells is
  taken to cost nothing here.
- **A thing that moves.** The rule is proved for a thing that stays put. For
  a drifting target the map must be moved before each update, and the simple
  rule is no longer guaranteed to be the best (the problem is treated by
  Brown 1980). The video makes no claim that it is.

## For programmers

The four lines on screen (07_2 and 08_3) are the whole method:

```
p[i] *= 1 - d[i]                       # the search of cell i came back empty
t = sum(p); p = [x / t for x in p]     # the map adds up to one again
find = [x * y for x, y in zip(p, d)]   # chance of a find, cell by cell
i = find.index(max(find))              # search there next
```

As shown they are a loop body: they need `p` and `d` defined, `i` chosen
before the first update, and a stop. Runnable, on the four-cell sea (it
picks B, C, A, A, B, D, A, C, the first eight of `F-U.best.order`):

```
p = [0.40, 0.30, 0.20, 0.10]           # the map: the chance it is in each cell [F-T.four_cell.prior.decimal]
d = [1/4, 2/3, 2/3, 2/3]               # the chance a search of each cell sees it [F-M.seeing]
for _ in range(8):
    find = [x * y for x, y in zip(p, d)]   # chance of a find, cell by cell
    i = find.index(max(find))              # search there next
    print("ABCD"[i])
    p[i] *= 1 - d[i]                       # the search of cell i came back empty
    t = sum(p); p = [x / t for x in p]     # the map adds up to one again (t is 0.8 at the first) [F-T.code.t_first]
```

The normalising line is for display: it does not change which cell has the
greatest chance of a find, because every cell is divided by the same `t`
(so a heap of the next slice of each cell does the whole job, in O(log N) a
search). The rule is a k-way merge of N sorted streams, cell i's k-th slice
being pᵢdᵢ(1 − dᵢ)ᵏ⁻¹.

- **Dots are weights.** "A search puts out the dots it would have seen" and
  "a search multiplies each dot's weight by its chance of being missed" are
  the same update: 20 of B's 30 dots dark and 10 lit, or all 30 dimmed to a
  third, give the same map. The Coast Guard's planner does the second (a
  particle's weight is multiplied by the chance it was missed) [F-C.sarops, F-M.prior.dots, F-M.empty1.dots_out].
  Deleting each swept dot with probability dᵢ is also a valid way to
  sample the new map, but a wasteful one: it throws away sample size at
  every search, where weights keep every dot.
- **Why thousands of dots.** After an empty flight over the middle of a
  cloud the weights are uneven, and the effective number of dots (the
  inverse of the sum of squared normalised weights) drops. Real planners
  carry thousands of particles per scenario for that reason [F-C.sarops.dots:
  2,500, 5,000 or 10,000], and resample.
- **Un-normalised on purpose.** The Coast Guard's bookkeeping leaves the
  weights un-normalised: what is left is then the chance that every search
  so far would have come back empty, a number worth watching.
- **Where a dot is reweighted.** At the position it held when the searcher
  passed, not where it has drifted to since.
- **The 1968 map as a simulation.** Pick a scenario by its weight, draw its
  uncertain quantities (course, speed, position), run it, and add one to the
  cell where the run ends; do that 10,000 times [F-R.replications, F-S.sank, F-R.F6]. A cell
  with no dot is not impossible, only rarer than about one run in ten
  thousand, and the number of runs sets the map's resolution: F6's 85 dots
  are 85 give or take about 9 [F-R.F6, F-T.f6.dots.give_or_take]. A cell with
  no dots is never searched by the rule (the update never adds).
- **The drift cloud (10_2 to 10_4) is the video's own model**: 2,000 dots
  [F-D.dots, seed F-D.seed], a start known to within a mile [F-D.start_radius],
  a drift of 1.5 miles an hour [F-D.drift], and a drift error of 0.3 of the
  distance drifted (the Coast Guard's own planning figure for drift error, a
  probable error: the radius of the circle that holds half of the possible
  places; the National SAR Supplement calls it the confidence factor,
  "normally 0.3", and Frost and Stone 2001, §4.6.1.3, report it and call it
  "perhaps optimistic") [F-D.error_share, F-D.drift]. Each dot keeps *one*
  wind-and-current error for the whole drift, so the spread grows in
  proportion to time. Two things the video does not settle, and says
  "model" about. First, an error drawn afresh every hour would make the
  spread grow more slowly (like the square root of time) and the same
  flight at twelve hours would find noticeably more than 13 in 100
  [F-D.h12.seeing]; which of the two suits a person in the water is a
  modelling choice. Second, checked against the sources: 0.3 is a probable
  error, not a radius that holds every dot, and the video's dots, all inside
  that radius, are simpler than the Coast Guard's spread. The Coast Guard
  adds the errors as a root sum of squares (E) and sizes its first search as
  a square of half-side 1.1 E, which holds about 65 in 100 of the spread
  (Frost and Stone 2001, Table 4-2 and §4.5.4); with the video's constants
  that first square gives 95, 73, 36 and 12 in 100 at 1, 3, 6 and 12 hours,
  and a circle that holds 95 in 100 of the spread gives 67, 37, 15 and 4
  (the video's own disc gives 93, 65, 34 and 13 [F-D.h1.seeing,
  F-D.h3.seeing, F-D.h6.seeing, F-D.h12.seeing]). "Hours matter" stands.
  [F-T.drift.coast_guard.first_search.half_side_in_E,
  F-T.drift.coast_guard.first_search.poc_percent,
  F-T.drift.coast_guard.first_search.seeing,
  F-T.drift.coast_guard.circle.holds_in_100,
  F-T.drift.coast_guard.circle_95.seeing, F-T.drift.double_radius.hours] The
  video's own words are "in this model".
- **The flight.** The 93, 65, 34 and 13 in 100 are for 180 miles of track
  spread evenly over the whole circle [F-D.track] and the random-search
  formula below. A flight fitted to the cloud does better: at three hours the
  same 180 miles on a 3-mile square sees 86% of the dots inside it and finds
  the person 73 times in 100; at six hours, a 5-mile square, 44 [F-D.flight.*].
  The video makes no claim about the best plan for a moving target. [F-D.h1.seeing, F-D.h3.seeing, F-D.h6.seeing, F-D.h12.seeing]

## Search effectiveness, exactly

- **Koopman's random search formula** (*Search and Screening*, OEG Report 56,
  1946, eq. 40): the chance that a point is swept is 1 − e^(−WL/A) for W the
  width of the strip, L the length of the track and A the area, when each
  piece of track is laid down *without regard to what is already swept*. At
  WL = A that is 63.2%, at twice that 86.5%, at three times 95.0%
  [F-K.formula.*, F-K.perfect.1, F-N.koopman]. Koopman calls it "a proper estimate, on the conservative
  side, of the practical results achieved": a deliberately laid parallel
  pattern does better than random strips for the same track, though a real
  sensor, which misses some of what lies inside its own strip, still does
  not reach 100% at a coverage of one. So the video's "two in three" is a
  cautious floor for a flat cell, an illustration, and not a measurement of
  any real sensor. (Chapter 13's gap between 94 and 65 percent comes from
  the widths, which the paper then scores with this same formula.) [F-S.F6.searched.at_sea, F-S.F6.searched.after]
- **Sweep width is an effective width.** It is the area under the lateral
  range curve, not a hard edge: a searcher sees some of what is outside it
  and misses some of what is inside. The Coast Guard's 0.1 nautical mile (185
  m) for a person in the water from a helicopter at 300 feet [F-D.strip, F-D.strip.metres, F-D.strip.altitude.feet] is
  a value from its Table H-15, which is given for a particular object,
  height and visibility; other conditions give other widths.
- **The hundred strips (09_3)** are a simulation: 100 strips, each 1% of the
  cell, at random offsets, seed 1946; one run swept 66.5%, the mean of 1,000
  runs is 63.4%, the least of them swept 57.1% and the most 69.9%, and the
  exact mean is 63.4% [F-K.lanes.*]. The shown run is 3.1 points above the
  mean of the thousand runs: 1.7 standard deviations of the thousand runs
  (1.75 points), or 1.8 of the exact spread of one run (1.72 points)
  [F-T.lanes.one_run_above_mean, F-T.lanes.sd.thousand_runs,
  F-T.lanes.one_run_sds_above.thousand_runs, F-T.lanes.sd.exact,
  F-T.lanes.one_run_sds_above.exact]. That is why it is labelled "this run".
  The formula's 63.2 is the limit of ever thinner strips [F-K.formula.1].
- **The two-in-three search** (05_2) is the video's own illustration for a
  flat cell. Koopman's curve puts it at a coverage of 1.10, and the canyon's
  one in four at 0.29 [F-K.coverage.*].
- **The model search over the 1968 map (08_4, 08_5)** is the video's own:
  every search sees what is in its cell two times in three
  [F-R.model.seeing], searches are independent, and every plan searches the
  same 180 lettered cells [F-R.grid.cells]. The percentages are of *all
  10,000 runs*; 1,100 of them ended outside those cells, so no plan reaches
  them [F-R.total, F-R.outside.points]. Over the 180 cells alone, which hold
  8,900 of the runs [F-R.grid.points], the three twenty-search figures would
  be 0.9, 49.8 and 55.9 [F-T.model.sweep.over_grid.found_by.20,
  F-T.model.once.over_grid.found_by.20, F-T.model.best.over_grid.found_by.20],
  each larger by 10,000 ÷ 8,900, a factor of 1.12 [F-T.model.over_grid.factor].
  Row by row from row 1 starts at the map's emptiest edge: its first twenty
  cells hold 114 of the 10,000 [F-R.model.sweep.first20.points], so its
  0.8% [F-R.model.sweep.found_by.20] is a low value for a plan with no map.
  Twenty cells chosen at random, each once, average 6.6%
  [F-R.model.random.found_by.20] (from row 5 it would be 22%
  [F-T.model.sweep_from_row5.found_by.20]). The map, never updated, reaches
  44.3% [F-R.model.once.found_by.20] and the rule 49.8%
  [F-R.model.best.found_by.20], so most of the gap to row by row is having
  the map. A cell with no dots is never searched: 40 of the 180 hold none
  [F-R.grid.empty_cells], and the cell that held the wreck, F6, with 85 dots
  [F-R.F6], would be reached at the thirtieth search
  [F-R.model.F6.first_search] (a thinner cell would wait
  far longer: the rule goes to the likeliest cell each time). The first map
  "does not have to be right", but a wrong one costs time. [F-R.grid.cells, F-R.replications, F-R.F6, F-R.model.sweep.found_by.20, F-R.model.once.found_by.20, F-R.model.best.found_by.20]

## Four searches, and what is and is not known

- **Palomares, 1966** [F-P.*; S7]. A B-52 and a tanker collided on 17
  January 1966; four hydrogen bombs fell, three on land and one into the
  sea. The Navy's account chose the area that held the bomb "primarily" from
  sonar contacts "accorded high credibility because of the statements and
  plotted evidence from the Spanish fisherman"; the submersible *Alvin*
  found it on 15 March in the vicinity of where he had seen it descend. The
  probability work kept the books on how well each area had been searched;
  it did not find the bomb. The 1971 paper says Craven "had earlier
  originated this approach in the H-bomb search". [F-N.paper]
- **Scorpion, 1968** [S1; F-S.*]. Lost on 22 May, 400 miles southwest of the
  Azores in more than 10,000 feet of water (the Navy's own wording, DANFS
  and report NT-24-1; the 1971 paper says "as much as 2 miles" below the
  surface); cause undetermined and not speculated on. The search began on 10 June; Craven and Andrews set nine
  scenarios and their weights on 18 July; Richardson's map reached the ship on
  12 August (63 days in); the wreck was found on the fifth cruise, on 28
  October, in cell F6, "within 260 yards of the edge of rectangle E5, the
  rectangle having distinctly the highest a priori probability" [F-S.yards].
  F6 held 85 of the 10,000 points (0.85%, 24th of the 172 printed cells);
  E5 held 1,250. The sled went back to F6 "because of proximity to the piece
  of metal and the fact that this was a region with high a priori
  probability": the metal had been photographed between 27 June and 23 July,
  before any map existed. The video never says that Bayes' rule found the
  *Scorpion*. [F-N.paper, F-R.F6, F-R.F6.percent, F-R.printed_cells, F-R.E5, F-R.top_two.percent]
- **How the weights were set.** The paper says Craven and Andrews "assigned
  credences" that reflected, "at least indirectly", the views of operating
  personnel and specialists. It does not list the scenarios or print the
  weights. The story that experts bet bottles of whisky is in one
  journalistic account and nowhere else that could be read; the video does
  not tell it.
- **The reference essay** (tomrocksmaths.com, 2023) is a map of the topic,
  not a source: of its six checkable claims about the search one is false
  (that the search started with rectangle F6), three are imprecise or
  contradicted by the 1971 paper, and two stand. [F-N.paper]
- **Air France 447, 2009 to 2011** [S3, S4, S5]. Flight AF 447 from Rio to
  Paris went down on 1 June 2009 with 228 aboard [F-A.lost, F-A.aboard]. The
  wreck was found on 2 April 2011, about 6.5 nautical miles (12 km) from the
  last known position, in 3,900 m of water, 22 months on, after three
  underwater searches had found nothing [F-A.found, F-A.found.nm,
  F-A.found.km, F-A.depth.metres, F-A.months, F-A.underwater_searches]. The
  analysts' map was a weighting of three ingredients (0.35, 0.35 and 0.30,
  chosen by judgement, which the authors call "subjective")
  [F-A.weights, F-T.air_france.weights.decimal], as three stories: anywhere
  in the circle, near the last position, traced back from what was found
  afloat (the paper's D1, D2 and D3, in that order). It was updated for four
  empty searches, one on the surface and three under water
  [F-A.empty_searches, F-A.underwater_searches]. Doubts about the
  beacons led them to a second map that leaves the silence out; the wreck
  lay in "a high probability area" of that one. *Both* maps are dark near
  the centre of the circle, and the BEA planned the 2011 search from the
  first, a systematic search from the centre outward. That the beacons
  failed is the authors' inference. They write that the better way to treat
  the doubt "would have been to compute a joint distribution on beacon
  failure and wreck location", which is chapter 03's move (a weight on a
  story). [F-A.lost, F-A.found, F-A.aboard, F-A.found.nm, F-A.found.km, F-A.depth.metres, F-A.months, F-A.weights]
- **The Coast Guard and John Aldridge, 2013** [S6, S10]. CASP (Computer
  Assisted Search Planning) entered service in 1974 and used a Bayesian
  approach; SAROPS has been operational since January 2007. Of the
  Coast Guard's cases about nine in ten involve no search at all. The account
  of the Aldridge search is the *New York Times Magazine*'s, reported from the
  command centre; the Coast Guard's own releases do not mention the
  software. The first map's guess for when he fell was 9:30 to 11:30 p.m.
  (middle 10:30); he fell at about 3:30 a.m.; the map was re-run at about
  8:30 a.m. and he was seen at 2:58 p.m., 11 hours 28 minutes after he fell.
  The helicopter's last track was drawn by hand through the high-chance area
  after the software froze, and he had tied two lobster buoys together and
  was sitting on the rope between them when he was found, so he was not a
  free drifter and his twelve hours are not chapter 10's 13 in a hundred.
  The video does not say the software found him. [F-C.aldridge.in_water, F-D.h12.seeing, F-C.casp, F-C.sarops, F-C.aldridge.date, F-C.aldridge.guess.middle]

## Chapter 13, in the paper's own numbers

Richardson and Stone (1971), Tables 1 to 3, for F6 on 9 October: on scene,
using the sensor widths then believed (800 feet for sonar, 400 for the
magnetometer [F-S.sweep.at_sea]), the chance that F6 had already been
searched thoroughly was 0.94 [F-T.f6.searched.oct9.on_scene]; estimated
afterwards, with the sensors' widths set against the wreck as found (camera
185 feet, magnetometer 220, sonar 0 [F-S.sweep.after]), it was 0.65
[F-T.f6.searched.oct9.afterwards]. The update leaves a cell 6% of its chance
at 0.94 and 35% at 0.65: almost six times as much [F-S.F6.left.at_sea.percent,
F-S.F6.left.after.percent, F-S.F6.left.ratio]. (The sonar's width falls from
800 feet to 0 and the magnetometer's from 400 to 220; the cell's coverage
falls by a factor of 2.7, from 2.81 to 1.05, because every sensor and every
pass is added [F-T.f6.coverage.ratio, F-S.F6.coverage.at_sea,
F-S.F6.coverage.after].) F6's chance on the map was 0.85% before any search,
0.19% by the ship's numbers and 0.54% by the estimated ones [F-R.F6.percent,
F-S.F6.chance.at_sea, F-S.F6.chance.after]. The paper calls these "crude sensor sweep width estimates
against the actual target", for *Scorpion* as it was found, partly buried;
the sonar's zero rests on the failure of one of the two sonars to detect
her, so "ten more passes with that sonar would have found nothing" is
conditional on that estimate. At the 800 feet believed, one pass over the
whole lettered grid was 1,148 sea miles, and the sled had towed 1,026
[F-H.once.miles.believed, F-H.track.miles]. The order of the evidence does
not matter: the early passes were scored against a map that arrived late,
and the product of the factors is the same in any order. [F-S.F6.searched.at_sea, F-S.F6.searched.after, F-S.F6.left.at_sea.percent, F-S.F6.left.after.percent, F-S.F6.left.ratio, F-S.F6.coverage.at_sea, F-S.F6.coverage.after, F-S.sweep.at_sea, F-S.sweep.after, F-S.F6.chance.at_sea, F-S.F6.chance.after, F-R.F6.percent, F-N.paper, F-H.once.miles.believed, F-H.track.miles]

## What the video simplifies

| Said | Exact |
|---|---|
| "the chance it is there, times the chance of seeing it" (05_6) | pᵢdᵢ, with dᵢ conditional on the thing being in cell i; the Coast Guard's names are POS = POC × POD |
| "take a patch of sea about twelve miles on a side" (02_5) | the lettered grid of the Navy's later map, 12 × 12.6 sea miles (1,100 of the map's 10,000 dots fall outside it); the video does not say the Navy's search had been narrowed to it, because the sources disagree on the size of the area the acoustic work left |
| "a search sees a wreck two times in three" (05_2) | an illustration; the video derives the idea of an effective chance from Koopman's formula, and a real dᵢ depends on sensor, conditions and the object |
| "if the wreck were at any of those twenty, we would have it" (06_3) | the dots are equally likely ways things could be, each including whether this search would see it (a dot is a place together with this search's luck, drawn afresh for each search); no *place* in B is one where a wreck is certainly seen, a wreck at any place in B is seen two times in three; 20 of B's 30 dots are the ways in which it would have been [F-M.prior.dots, F-M.empty1.dots_out, F-R.grid.miles, F-R.outside.points, F-R.total, F-C.casp, F-C.sarops] |
| "the first map does not have to be right" (14_4) | for a thing that stays put, if every place that might hold it has some chance and every search can see (dᵢ > 0), the rule comes back to every such cell and finds it with chance 1; the first map sets only how long that takes. A place given no dots is never searched (the update never adds) |
| "by the map's own reckoning" (08_5) | the chance of a find within n searches, averaged over the prior map; not a statement about where the wreck was |
| "a strip a tenth of a mile wide" (10_3) | a sweep width: an effective width |
| "the map drifts and it spreads" (10_2) | each dot is moved by its own draw of wind and current, then reweighted; the cloud need not stay round |
| "since nineteen seventy-four" (11_1) | CASP was put into operation in 1974; SAROPS since 2007 |
| "the method that searched for them" (14_6) | the lineage the Coast Guard's own developers give (Kratzke, Stone and Frost 2010; Frost and Stone 2001): Richardson's and Stone's work on the *Scorpion* search to CASP to SAROPS |

## Words you will hear next in practice

The video keeps to a few names, once each. In the Coast Guard's own
vocabulary (the standard terms of search planning; the first three are the
names the fact sheet records at 05_6): *containment* (POC) is the chance the
thing is in the area you searched, the video's "chance it is there";
*detection* (POD) is "the chance of seeing it"; *probability of success*
(POS) is their product, "the chance of a find". Others you will meet:
*datum* (the place a search is built around; here the drift cloud's
centre), *track spacing* (the distance between lanes; a spacing equal to the
sweep width is a coverage of 1), *creeping line* (parallel lanes worked from
one edge; the "row by row" plan of chapter 08 is one), and *leeway* (the
push of the wind on what floats). "Coverage" is the ratio of track × sweep
width to area; the 09 readout shows the share swept, which is 63% at a
coverage of 1 for random strips. [F-K.formula.1]

## What the video does not say

That the Navy had narrowed its search to a patch of sea twelve miles on a
side, or the size of the area its acoustic work left, or a number of
acoustic signals (the sources disagree); the cause of the *Scorpion*'s
loss; the Chivas Regal bets; that Bayes found the H-bomb; that
mathematicians proved the beacons were broken, or a probability that they
failed; that the software found John Aldridge, or any lives-saved total
credited to it; a year when Koopman's report was declassified (sources
disagree); how he said his own name (nothing was found).

## Sources

Cited and never vendored. Full list, with the page numbers and what was and
was not seen, in `docs/fact-sheet.md`.

- H. R. Richardson and L. D. Stone, "Operations Analysis During the
  Underwater Search for Scorpion", *Naval Research Logistics Quarterly*
  18(2), 1971, pp. 141-157.
- B. O. Koopman, *Search and Screening*, OEG Report 56, 1946.
- L. D. Stone, C. M. Keller, T. M. Kratzke and J. P. Strumpfer, "Search for
  the Wreckage of Air France Flight AF 447", *Statistical Science* 29(1),
  2014, pp. 69-80; Metron, report to the BEA, 20 January 2011; BEA, final
  report on the accident to F-GZCP, July 2012, and "Sea Search Operations".
- T. M. Kratzke, L. D. Stone and J. R. Frost, "Search and Rescue Optimal
  Planning System", FUSION 2010; J. R. Frost and L. D. Stone, *Review of
  Search Theory*, USCG report CG-D-15-01, 2001; United States National
  Search and Rescue Supplement to the IAMSAR Manual, 2000 edition, glossary.
- M. C. Chew, Jr., "A Sequential Search Procedure", *Annals of Mathematical
  Statistics* 38(2), 1967, pp. 494-502 (the abstract is seen; the paper's
  text is not, so the number of the theorem, 2.1, is unverified); W. L.
  Black, "Discrete Sequential Search", *Information and Control* 8(2), 1965,
  pp. 159-162; J. B. Kadane, Cowles Foundation Discussion Paper 220, 1967.
- Defense Nuclear Agency, *Palomares Summary Report*, 1975.
- Naval History and Heritage Command, *DANFS*, "Scorpion VI (SSN-589)";
  Naval Nuclear Propulsion Program, report NT-24-1.
- US Coast Guard, Addendum to the National SAR Supplement, COMDTINST
  M16130.2F (2013), Table H-15.
- Paul Tough, "A Speck in the Sea", *The New York Times Magazine*, 2
  January 2014.
- T. Bayes, "An Essay towards solving a Problem in the Doctrine of Chances",
  *Philosophical Transactions* 53, 1763 (read after his death in 1761, sent
  by Richard Price); the rule in the form used here is Laplace's (1774).

## Names to search

Bayesian search theory; Koopman, *Search and Screening*; Richardson and
Stone 1971; Stone, *Theory of Optimal Search*; SAROPS. [F-N.paper]

Narrated by synthetic speech. Every number in this video is in the code, and
each was checked a second way: the repository link is on the receipts card
and in the description.
