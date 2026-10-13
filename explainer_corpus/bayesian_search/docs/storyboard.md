# Storyboard — How to Find a Lost Submarine

The hand-edited source of this video's words and pictures. `python3 -I
tools/storyboard.py bayesian_search` checks it (gate G2) and writes
`script.tsv` from the words column. Facts: `docs/fact-sheet.md` and
`oracle.py` (every number below names its fact key); brief: `docs/brief.md`;
reviews and their dispositions: `docs/reviews/bayesian_search/` (this is the
storyboard as revised after review R1).

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a question for the
viewer; the beat ends with the question and the picture holds before the
next beat answers it.

**Two kinds of question, two looks.**

- **Your turn** (a quiz): a `⟨predict⟩` beat whose last column says
  `listen N s (countdown)`. The stage stays: a violet frame comes on around
  the live picture, a ring at top right drains evenly over N seconds with
  the seconds left in its middle, and the question as spoken sets in the
  caption band with "pause for more time". The narration is silent and the
  music is alone, one soft mallet note a second. Whatever the viewer needs
  to count is on the stage, never hidden by a card. At zero the answer lands
  in the picture, the frame lifts, and the next beat works it where it
  stands. Five quizzes, all on the made-up sea or on Koopman's cell: no
  ring ever runs over a real person or a real loss. Every quiz is a row of
  `model::QUIZZES`, and its numbers are the `F-Q.<name>.*` facts.
- **A question to carry** (01_1, 02_7, 04_5, 08_1, 10_2, 11_4, 12_4, 13_3,
  14_3): the question as a line in the caption band with no ring, held about
  two seconds. The owner's subject line is the first of them.

**The dots.** The video's one picture. A dot is one equally likely way
things could be: one place the lost thing could be. The map is the cloud of
them, and a cell's chance is its share of the dots still lit. One grammar,
the same in every chapter:

- a **teal ring** on a dot means "this search would see this one";
- a dot **goes out** only when a search has come back empty (it turns
  `faint` and stays where it is, so the viewer can still count how many
  there were); lit always means "still possible", and no dot is ever "lit
  up" by being seen;
- every cell on every map carries a **blue heat** equal to its share of the
  lit dots, recomputed live: when one cell's dots go out it cools and every
  other cell deepens in the same instant, with their dots untouched;
- no dot relights, except once, at 13_6, where that is the lesson;
- no dot appears, dims or goes out on a sentence about a crew or a
  passenger. The 99 and the 228 are a number in ink on a cleared stage.

Nothing on screen is ever a number that the dots do not show. Where the
dots would not divide evenly (the second search of the small sea puts out
thirteen and a third of a hundred), every dot first splits in three, with
the line "more dots, same map" on screen (07_5).

**The made-up sea.** Four cells in a two-by-two block, A and B above C and
D, tagged "a made-up sea" whenever it is on screen, with a hundred dots:
40, 30, 20, 10 (`F-M.prior.dots`). Its lost ship is "it" and "the wreck",
never "she": it is not the *Scorpion*, and its two stories say nothing about
her. A is drawn as a canyon (four smooth contour lines in `dim`). Under each
cell three numbers keep fixed slots from 05 to 08, stacked as a small sum:
blue, the chance it is there; teal, the chance of seeing it; gold, the
chance of a find.

**The 1968 map.** Richardson and Stone's Figure 2 redrawn as it was printed:
the lattice, the column letters A to O and row numbers 1 to 12, the printed
numbers (`model::y1968::POINTS`), each cell's heat from its number. It is
one shot seen four times: the cold open (01_2), built from ten thousand dots
(04), under a model search (08), and with what really happened (13). The
source line "Richardson and Stone, 1971, Figure 2" is on screen whenever its
numbers are.

**Model numbers.** The word "model" is on screen with every number that
comes from the video's own assumptions (02_5, 02_6, 08_4, 08_5, 10_2 to
10_4), with the assumption beside it.

## 2 The arc

| Act | Chapters | Spine |
|---|---|---|
| I. The question | 01 the question, 02 ninety-nine | the owner's question; the picture that will answer it; the truth about the *Scorpion*; why you cannot look everywhere |
| II. The map | 03 a map of chances, 04 ten thousand dots | two weighted stories make a map of a hundred dots; where the idea began (Palomares); the Navy's map made the same way, and the wreck beside its top cell |
| III. The method | 05 a search can miss, 06 nothing found, 07 Bayes' rule, 08 where next | the likeliest cell is not the best first search; an empty search by hand, in dots; the rule named and the loop; why no order does better, and a model search over the 1968 map |
| IV. Effort and time | 09 the second pass, 10 adrift | Koopman: why a search sees two times in three, and why a second pass buys less; a target that drifts, and why hours matter |
| V. Four searches | 11 twelve hours, 12 the silence, 13 how well they looked, 14 four maps | the Coast Guard and John Aldridge; Air France 447; what really happened in 1968; four maps side by side, what the method asks, and the 99 |

The payoff is previewed twice before any method is taught: the cold open's
wordless move on the real map (01_2) and the real map with the wreck beside
its top cell (04_6). The video ends on the rigorous form of the opening
picture: the update as the 1971 paper printed it (13_1), and the cold open's
shot replayed with every part named (14_4).

**Length.** The G2 estimate below is 15.8 minutes, of which ten seconds
are question holds on the five countdown beats that the real quiz does not
spend (the ring is the pause). That is over the order's 11 to 15: review
R1's corrections added about a minute (the truth about the first map, the
proof in place of an assertion, Palomares told where its idea is used),
and on the three finished cuts in this voice the real narration ran 3 to 5
percent longer than the same estimate. If the owner wants it inside 15,
the cuts, in order: chapter 08's model search over the Navy's map (08_4 and
08_5, 30 seconds); the first quiz as a plain sum (03_5, 10 seconds); the
hook's arithmetic folded into one sentence (02_5 and 02_6, 10 seconds).

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses a
data colour.

| Idea | Color |
|---|---|
| the map: dots still lit, and a cell's heat | blue `#58C4DD` |
| the search: the searcher, its track and strip, the ring on a dot it would see, the chance of seeing | deep teal `#17806D` |
| the chance of a find, and the cell the rule chooses; nothing else (not the likeliest cell, not a title) | `hero` gold `#FFD866` |
| dots a search has put out | slate `#566074` |
| the true place of a wreck, once the viewer is told it | `ink` white, a small hollow ring |
| a last known position (not the true place) | a small open diamond in `dim` |
| a living person found (11_6 and 14_6 only) | `good` green `#83C167` |
| an estimate that turned out wrong (11_3, 13_4, 14_3) | `warn` red `#FC6255` |
| the "your turn" frame and its countdown | violet `#B189D6` |

Every pair of these, with the background, is at least 24 apart (CIEDE2000),
pinned by a test in the crate before any draft; the two hexes changed after
review R1 (the teal and the slate) are what bring the closest pairs up from
16 to 24. The made-up sea's cells are drawn on the bare background, not on
card fill. Red is only ever an estimate that turned out wrong: never danger
and never the dead.

## Vocabulary (one word per idea)

**map**; **cell** (never square, box, sector or zone); **dot**; **story**
and its **weight**; **chance** (never "probability" in the narration, which
is on screen where the formal names are given, and never "odds");
**the chance it is there**, **the chance of seeing it**, **the chance of
a find**; **search** (one pass over one cell); **comes back empty** (the
search found nothing) and **miss** (it was there and was not seen): never
swapped; **update** (on screen at 07_2, spoken at 07_5); **track**, **strip**, **pass**;
**row by row** (the plan with no map; never "the sweep", which would
collide with "sweep width"); **drift**. The *Scorpion* is "she", as the
Navy writes; the made-up sea's ship is "it" and "the wreck". "Miles" at sea
are sea miles and the screen says so once ("sea miles, 6,076 feet each");
depths are in feet. Never "could have been saved", "miracle", "genius",
"magic", "honest figure" (the men at sea were optimistic, not dishonest).
The formal names appear once each, in a lower-third: scenario, credence,
prior, posterior, detection probability, Bayes' rule, sweep width, coverage,
the random search formula, particle, leeway.

**Lower-thirds** arrive in the hold after the motion they annotate, one at
a time, about a dozen words; anything longer is in the description
(`docs/description.md`, written with the script).

## The code panel (for programmers)

Pinned here so the script review can read it and the oracle can run it
(`F-M.code.*`). The first two lines appear at 07_2, the last two at 08_3.

```
p[i] *= 1 - d[i]                       # the search of cell i came back empty
t = sum(p); p = [x / t for x in p]     # the map adds up to one again
find = [x * y for x, y in zip(p, d)]   # chance of a find, cell by cell
i = find.index(max(find))              # search there next
```

## Sound plan (for the score bead)

Nothing here is auditioned yet; the drafts of P5 are silent but for the
scratch voice.

- **Two silences.** After "All ninety-nine men were lost" (02_3) and after
  "could not be saved" (14_5) nothing sounds for two and a half seconds: no
  music breath (a breath would lift the bed), no cue. The scenes wait
  without a breath, and the score bead keeps the bed out across them.
- **The search sound** has one meaning, a search of the map that came back
  empty, and is heard only on that move: 01_2, 06_1, 07_5, the first search
  of 08_4, 11_1, 12_5, 14_4. Each sits in its own gap. Trials, strips laid
  at random and model runs are silent. An empty search gets no answering
  note; at 11_6, when the track reaches the man, one plucked note answers,
  the only one before the ending.
- **Countdowns:** one soft pitched mallet note a second, never a tick; the
  last notes do not close on the tonic, so the resolution is not spent.
- **The motif** is left open at the title and resolves once, at 14_6.
- **Not in this video:** sonar pings, alarms, heartbeat or clock ticks, a
  held bass note under 02_3, 12_1, 13 or 14_5.

## 01_the_question (24 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | ⟨predict⟩ A nuclear sub has gone missing. Men are trapped, and time is running out. How can we possibly find them in time? | A plain dark stage. The three sentences set themselves large, one line each, as they are said: the first two in `dim`, the question in ink | a card: the owner's question | |
| 01_2 | Start with a strange idea. A search that finds nothing has still told you something. | After the first sentence the voice stops for two and a half seconds: the 1968 lattice draws, then its dots, thick near the middle; a teal strip crosses the brightest cell with the search sound; most of that cell's dots go out, and in the same instant every other cell's blue deepens. The second sentence lands on the changed map. No labels, no numbers | the dots; a grid; cell heat recounted live from the dots; the search | hold (2.5 s with the voice silent, between the two sentences) |
| 01_3 | *(title)* | **How to Find a Lost Submarine**, over the map, which dims | title card | listen 3 s (the motif, unanswered) |

- **Cold open** (plan rule 20): the owner's subject line in his words, then
  the payoff as a paradox with the picture that will explain it, shown
  before it is described, then the title, at about 20 seconds.
- **Honesty:** the question is asked before any ship is named, as a premise
  (in `dim`); chapter 02's first line says the Navy "had to ask" it, and
  its third says what the answer was for the *Scorpion*.
- **Not the men:** the dots are the real map's thousands, on its lattice,
  so they cannot be read as 99 people.

## 02_ninety_nine (83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | In May of nineteen sixty-eight the United States Navy had to ask that question for real. The U S S Scorpion, a nuclear submarine, was heading home across the Atlantic with ninety-nine men aboard. | The Atlantic as a dark field with two labelled points, the Azores and Norfolk, and a thin track between them; a small ink point moves along it. "USS Scorpion (SSN-589), May 1968" (`F-S.crew`) | a number line of dates along the bottom; lagged reveal | |
| 02_2 | She was due at Norfolk, Virginia, on the twenty-seventh, and did not arrive. For nine days the Navy searched the ocean, as if her crew might still be saved. | The date line: 27 May, "due" (`F-S.due`); the point is no longer on the track. Nine small search marks fill along the date line at the pace of the sentence, to 5 June (`F-S.days.due_to_presumed`). "aircraft and dozens of ships". No running numeral | lagged reveal | |
| 02_3 | They could not be. She had gone down five days before she was due, in more than ten thousand feet of water. All ninety-nine men were lost. | The date line runs back to 22 May (`F-S.sank`, `F-S.days.sank_to_due`). The search marks fade. Then the stage clears to a small white ring and "99 men", and nothing else | a card for the 99, in ink | hold (a silence: the scene waits 2.5 s with no music breath) |
| 02_4 | Nobody knew that yet. And even to find the wreck, the Navy faced the first question of any search. Where do you look? | The ring goes out (nobody knew). A patch of sea southwest of the Azores is outlined: "about 400 miles southwest of the Azores" (`F-S.miles_from_azores`) | highlight | |
| 02_5 | Not everywhere. The search had narrowed to a patch of sea about twelve miles on a side. But the search gear rode a sled, towed just above the sea floor, and saw a strip about two hundred feet wide. | The patch becomes the 1968 grid, empty, 12 rows by 15 columns (`F-R.grid.miles`). An inset of one cell: the strip to scale, and the twenty-six passes side by side that fill it (`F-H.once.passes_per_cell`). "strip as measured afterwards: camera 185 ft, magnetometer 220 ft" (`F-S.sweep.after`, `F-H.strip.feet`); "one cell: 26 sea miles of track (model: perfect strips)" (`F-H.once.per_cell`) | a grid; the strip drawn to scale; lagged reveal | |
| 02_6 | One pass over the whole patch meant forty-six hundred miles of towing, more than four times what the sled managed in the whole search. | The strip runs fast over the whole grid, painting cell by cell under a mile counter, and stops at 1,026 with 40 of the 180 cells painted (`F-H.track.miles`, `F-H.track.cells`). "all 180 cells: 4,594 sea miles (6,076 feet each)" (`F-H.once.miles`, `F-H.once.over_track`); "broad-area search track, June to October: 1,026" | a grid recoloured in place; live counter of miles | |
| 02_7 | ⟨predict⟩ So where would you look first? | The grid, a quarter painted, held | a question in the caption band | |

- **The truth, early** (the order): 02_3 says it in the third line of the
  first real chapter, and nothing sounds after it. "As if her crew might
  still be saved" describes the search of 27 May to 5 June and nothing
  else.
- **The obvious approach fails on the real numbers** (plan rule 2): the
  strip width is the after-search estimate for the two sensors that worked
  and says so on screen; the 1,026 miles is the paper's figure for
  broad-area search up to detection. At the time, with the 800-foot strip
  they believed in, one pass looked affordable (1,148 miles,
  `F-H.once.miles.believed`): chapter 13 comes back to that.
- **Misconception defused:** "they just had to search the area carefully".
  Carefully, once, is four and a half times what the whole search towed.
- **Ruth:** nothing is said about how the area was narrowed; the secondary
  sources disagree, and the paper says only that others had done it. The
  paper's miles are sea miles (fact sheet, finding 9) and the screen says
  so; the depth is the Navy's own "more than 10,000 feet".

## 03_a_map_of_chances (78 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | The natural answer is, wherever it most likely is. So you need a map of how likely each place is. | The Navy's grid leaves, and the stage stands empty for a beat | | |
| 03_2 | Start small, with a made-up sea. Four cells, a lost ship, and two stories of what might have happened to it. | Four cells draw themselves, two by two, labelled A, B, C, D, tagged "a made-up sea". Two small story cards above, each a tiny four-cell map, captioned "kept to its route" and "strayed" | a grid; lagged reveal | |
| 03_3 | In the first story it kept to its route, so the wreck is most likely in cell A, maybe in B, less likely in C. In the second it strayed, and the wreck could be anywhere. | Card one's tiny map shades: a half, a third, a sixth, nothing (`F-M.story1.share`); then card two's, evenly: a quarter each (`F-M.story2.share`) | bars | |
| 03_4 | Experts weigh the two stories. Say sixty to forty. Now picture a hundred dots, sixty from the first story and forty from the second. Each dot is one place the wreck could be. | The weights 60 and 40 land on the cards (`F-M.story1.weight`). Sixty dots leave card one and settle: 30, 20, 10 (`F-M.story1.dots`). Forty leave card two, but hang above the cells, not yet settled | the dots; a count-up per cell | |
| 03_5 | ⟨predict⟩ The first story puts half of its dots in cell A. The second puts a quarter of its dots there. What is the chance the wreck is in cell A? | The violet frame and the ring. On the stage: thirty dots in A, and forty hanging above the sea | **the quiz frame**; countdown ring on a tracker | listen 6 s (countdown: the music alone, one mallet note a second) |
| 03_6 | Half of sixty is thirty. A quarter of forty is ten. Forty dots in a hundred. Forty percent. | The forty hanging dots fall, ten to a cell (`F-M.story2.dots`). In A, 30 + 10 = 40 (`F-Q.stories.answer`) | count-up | |
| 03_7 | Count every cell that way and you have the map. Forty, thirty, twenty, ten. | The cards fade. Each cell shows its dots, its percent and its blue heat: 40, 30, 20, 10 (`F-M.prior.percent`). Lower-third: "prior: the map before a search" | live numbers recounted from the dots | hold |

- **A made-up sea, said and shown** (review R1): the Navy's grid leaves the
  stage first, the tag stays on, the ship is "it", and the two stories are
  about a route, not about the *Scorpion*. The nine real scenarios are not
  known (04_2) and nothing here suggests what they were.
- **Show, then name:** "scenario" and "prior" are not said; each is on
  screen once (04_2, 03_7).
- **Misconception defused:** "a map of chances needs data about where ships
  sink". It needs stories and weights; the weights are judgments, and the
  video says "say sixty to forty".
- **Ruth:** the chance of A is the weighted sum 0.6 × ½ + 0.4 × ¼ (the law
  of total probability), done as dots. "Most likely in cell A" is a chance
  of exactly one half within the first story.
- **Maya:** every number here is a count of dots she can see, and the quiz
  leaves them on the stage.

## 04_ten_thousand_dots (79 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | The Navy's map was made the same way. John Craven, a Navy scientist, had first tried the idea two years earlier, searching for a hydrogen bomb lost in the sea off Palomares, in Spain. | A coast, a point offshore, "Palomares, Spain, 17 January 1966" (`F-P.date`). A few story cards, blank, with weights as outlines | lagged reveal | |
| 04_2 | That bomb was found near where a fisherman watched it fall, but the idea stayed. In July of sixty-eight, Craven and a colleague named Andrews set out nine stories of how the Scorpion might have been lost, and gave each a weight. | A small mark where the fisherman pointed, and the white ring beside it: "found by the submersible Alvin, 15 March 1966" (`F-P.found`). Then nine blank story cards build one at a time, each with a weight drawn as an outline: "18 July 1968: nine stories (scenarios), each with a weight (the paper's word: credence). Not printed in the 1971 paper" (`F-S.scenarios`, `F-S.scenarios_set`) | lagged reveal; lower-third: scenario | |
| 04_3 | A mathematician, Henry Richardson, had a computer play those stories out ten thousand times. Ten thousand dots. | The 1968 lattice, empty. Dots rain onto it, a counter running to 10,000 (`F-R.replications`); they pile where the map will be high. Caption: "each run drew its own course, speed and position" | the dots in flight, moved by placement; live counter; a grid | |
| 04_4 | Then he counted the dots in every cell. This is the map they printed. | The dots in each cell gather into its printed number, cell by cell in a ripple, and the cell takes its heat (`F-R.printed_cells`). "Richardson and Stone, 1971, Figure 2. Each cell about a mile across" (`F-R.cell.miles`). Lower-third, in the hold: "10,000 runs of what two men believed, not 10,000 sinkings" | a grid of numbers built from primitives; ripple | |
| 04_5 | ⟨predict⟩ One cell stands out, with one dot in every eight. They called it E five. Was she there? | E5 takes an ink outline: "1,250 of 10,000: 1 in 8" (`F-R.E5`, `F-R.E5.one_in`). The next highest, B7, is nearly as hot | highlight | |
| 04_6 | Almost. The wreck lay in a neighbouring cell, within two hundred and sixty yards of the edge of E five. | An inset of E5 and F6, four times the size: a white ring in the corner of F6 that touches E5, as the paper's asterisk sits. "85 of 10,000: fewer than 1 in 100" (`F-R.F6`, `F-R.F6.percent`). "within 260 yards of the edge of rectangle E5" (`F-S.yards`), quoted | highlight; the true place | |
| 04_7 | But the map alone did not find her, and the search took five months. To see why, we need a method for using a map like this. | The map dims; the made-up sea returns beside it | | |

- **Palomares, told where its idea is used** (review R1): two sentences,
  with what the map did not do said at once. The method began there
  (the 1971 paper: "Craven had earlier originated this approach in the
  H-bomb search"); the bomb was found by the submersible *Alvin* "in the
  vicinity of the place" where the fisherman saw it fall (the official
  report).
- **Payoff previewed** (plan rule 3): the real map and the wreck beside its
  top cell, before any method. 04_7 says at once that the map alone did not
  do it; chapter 13 says what did.
- **Misconception defused:** "the map said exactly where she was". She was
  in a cell holding 85 of the 10,000 dots, the 24th highest of 172
  (`F-R.F6.rank`), next to the one holding 1,250; both counts are on screen
  as "1 in 8" and "fewer than 1 in 100".
- **Ruth:** who did what is from the 1971 paper (pp. 143-45, 149) and, for
  which author made the map, the INFORMS profile of Stone. Nothing is said
  about how the weights were arrived at beyond "gave each a weight" (fact
  sheet, finding 4). The dots are runs of what two men believed: the
  lower-third says so, against reading 1,250 as a frequency of sinkings.
- **Dev** (description): pick a scenario by its weight, draw the uncertain
  quantities, add one to the cell where the run ends; a cell with no dots
  is not impossible, only rarer than one run in ten thousand.

## 05_a_search_can_miss (70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Back to the made-up sea. Forty percent says search cell A first. But a search is not a sure thing. | The made-up sea with its hundred dots and its blue numbers: 40, 30, 20, 10 | the dots | |
| 05_2 | Cell A is a canyon. A search there sees the wreck only one time in four. In the flat cells, a search sees it two times in three. | Contours appear in A. In the order of the sentences: a strip crosses a spare dot in A four times, and a teal ring catches it once; then a spare dot in a flat cell three times, ringed twice. The teal numbers take their slots: "1 in 4", "2 in 3" (`F-M.seeing`) | lagged reveal; the search | |
| 05_3 | So ask a better question. How likely is one search of a cell to find the wreck? | Under each cell the third slot opens, empty and gold: "chance of a find" | | |
| 05_4 | ⟨predict⟩ Forty percent, seen one time in four. Thirty percent, seen two times in three. Which cell gives the better chance of a find, A or B? | The violet frame and the ring; the sea, its dots and the blue and teal numbers stay | **the quiz frame**; countdown ring | listen 6 s (countdown) |
| 05_5 | Cell B. A quarter of forty is ten. Two thirds of thirty is twenty. Twice the chance. | In A, 10 of the 40 dots take a teal ring; in B, 20 of the 30. The gold numbers land: 10%, 20%, then 13⅓% and 6⅔% for C and D (`F-M.find.fraction`, `F-M.first.over_likeliest`). B's border turns gold | live numbers from the dots; highlight | |
| 05_6 | The chance of a find is the chance it is there, times the chance of seeing it. So the best cell to search is not always the likeliest. | The three slots read across as one line: chance it is there × chance of seeing it = chance of a find. Lower-third, in the hold: "detection probability. The Coast Guard's names: probability of success = containment × detection" | TeX equation in words, then p × d under it; lower-third | hold |

- **The second obvious approach fails** (plan rule 2): "look where it most
  likely is" loses to the second-likeliest cell, two to one
  (`F-M.likeliest.find.percent`, `F-M.first.find.percent`).
- **Misconception defused:** "a search either covers a place or it does
  not". It sees what is there some of the time; chapter 09 says why.
- **Ruth:** the product needs "seeing" to be conditional on being there,
  which is how 05_2 defines it. The canyon and its one in four are the
  video's illustration. C and D are shown as thirds, not rounded.

## 06_nothing_found (75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | So we search cell B. And the search comes back empty. | The teal strip crosses B with the search sound, in a gap between the two sentences. Nothing answers. "empty" | the search | |
| 06_2 | That is not nothing. Look at the dots. Thirty are in B, and this search would have caught two out of every three. Twenty dots. | All thirty of B's dots flicker, and two in every three, scattered through the cell, settle with a teal ring: twenty (`F-M.empty1.dots_out`) | highlight; count-up | |
| 06_3 | If the wreck were at any of those twenty, we would have it. We do not. So they go out. | The twenty go out: they turn slate and stay in place. B's blue cools, and A, C and D deepen in the same instant. The percents under the cells give way to counts of lit dots. Lower-third, in the hold: "a dot: one equally likely way things could be" | the dots; cell heat | |
| 06_4 | Eighty dots are left, and the wreck is at one of them. Ten are still in B, because a search can miss. | The lit dots drop into one row under the sea, grouped A, B, C, D, against a fixed ruler from 0 to 100: the row stops at 80 (`F-M.empty1.dots_left`); B's ten pulse (`F-M.empty1.cell_dots_left`) | the dots lined up against a number line; live counter | |
| 06_5 | ⟨predict⟩ Ten dots out of eighty. What is the chance it is in cell B now? | The violet frame and the ring; the row of eighty, with B's ten marked, stays | **the quiz frame**; countdown ring | listen 6 s (countdown) |
| 06_6 | One in eight. Twelve and a half percent, down from thirty. | The row stretches to fill the ruler, every group by the same amount; B's ten now span 12.5 (`F-Q.after_empty.answer`) | morph; live number | |
| 06_7 | Now look at cell A. Nobody searched it, and nothing in the sea has changed. But forty dots out of eighty is one half. What changed is what we know. | On the stretched row A ends at 50; in the sea A's dots have not moved. Lower-third, in the hold: "'The probability distribution on which the search is based is therefore a subjective one.' Stone and others, 2014" | live number recounted from the dots; lower-third | |
| 06_8 | Every cell we did not search grew by a quarter. Forty to fifty. Twenty to twenty-five. Ten to twelve and a half. | The dots return to the sea; the percents come back under the cells: 50, 12.5, 25, 12.5 (`F-M.empty1.after.percent`). "× 1¼ (100 dots ÷ 80 dots)" (`F-M.empty1.rise`) | TeX fraction | hold |

- **The centre of the video:** Bayes' rule done as counting, before it has a
  name. The stretch of the row against the ruler is "scale the map so it
  adds up to a hundred again", seen as one gesture.
- **The chances are knowledge** (review R1): 06_7 says it in the narration,
  with the AF 447 authors' sentence under it.
- **Which twenty** (review R1): the dots that go out are scattered and
  "which ones does not matter", so nobody concludes that ten particular
  places are blind. That each search is a fresh try is said aloud where it
  first bites (08_3, 09_5).
- **Misconception defused:** "an empty search of B tells you about B and
  nothing else". A did not change and A's chance did. And "B is ruled out":
  ten dots are still lit there.
- **Ruth:** conditioning on the event "the search of B came back empty":
  100 equally weighted possibilities, 20 excluded, 80 left.

## 07_bayes_rule (84 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | That move has a name. Bayes' rule, after Thomas Bayes, an English minister who died in seventeen sixty-one. | The made-up sea on the left. "Bayes' rule" in ink; under it "Thomas Bayes, died 1761. As a rule for equally likely cases, which is what the dots are: Laplace, 1774" (`F-N.bayes.died`, `F-N.laplace`). Lower-third, in the hold: "prior: the map before a search. Posterior: the map after it" | lower-third | |
| 07_2 | As a recipe. Multiply each cell's chance by the chance that the search would come back empty if the wreck were there. One third for cell B, and one for every other cell. Then divide by what is left, eighty, so the map adds up to a hundred again. | A grid of separate cells, one row for each cell of the sea: the percents fly in from the map (40, 30, 20, 10), then the column "× chance of an empty search": 1, ⅓, 1, 1 (small, under the ones: "a search of B cannot find a wreck that is somewhere else"), the products 40, 10, 20, 10, their sum 80 in teal, labelled "the chance this search comes back empty", and each product ÷ 80: 50, 12.5, 25, 12.5. In the hold the letters land on the numbers of B's row: p (1 − d) ÷ (1 − p d), and under it "every other cell: its own chance ÷ that same 1 − p d"; "update: redrawing the map by Bayes' rule". The first two lines of the code panel slide in, tagged "for programmers" | a table built from primitives, a column a sentence; TeX derivation from numbers to letters; code panel | hold |
| 07_3 | ⟨predict⟩ Here is the new map. Cell A has half the chance now. But A is the canyon. Where do you search next? | The violet frame and the ring. The sea with its lit dots (40, 10, 20, 10 of 80), its blue and teal numbers, and the first round's gold numbers greyed: 10, 20, 13⅓, 6⅔ | **the quiz frame**; countdown ring | listen 6 s (countdown) |
| 07_4 | Cell C. In dots, nothing has changed for A or C. A search of A would still catch ten, and a search of C thirteen. Only B has fallen. | Rings on the dots each search would catch: 10 in A, 13⅓ in C, 6⅔ in B and in D (`F-M.second.find.dots`); C's border turns gold (`F-Q.next_search.answer`) | the dots; highlight | |
| 07_5 | Search. Come back empty. Update. Choose again. That loop is the whole method. | Every dot splits in three, "more dots, same map" (`F-M.split.lit`). The sea sits inside a ring of the four words and does each as it is said: the strip crosses C with the search sound; nothing answers; forty of C's sixty dots go out and the heat shifts, 60, 15, 10, 15 (`F-M.empty2.after.percent`); the gold border jumps to A. Then three seconds with the voice silent while the loop turns once more by itself: A searched, empty, thirty dots out (`F-M.third.dots_out`), and the gold stays on A | the dots; cell heat; the loop as one figure | hold (3 s with the voice silent: the loop turns once more) |

- **Show, then name:** the name arrives after the viewer has done the rule
  by hand (06); 07_2 then says it as a recipe, numbers before letters.
- **One denominator, named** (review R1): the 80 is "the chance this search
  comes back empty", and both lines of the letters divide by that same
  quantity, as the 1971 paper's formula does (13_1).
- **Whole dots** (review R1): the second search would put out thirteen and
  a third of a hundred dots, so the dots split in three first.
- **Misconception defused:** "now A has half the chance, so surely A". In
  dots a search of A still catches ten and a search of C thirteen.
- **Ruth:** Bayes solved a special case; Laplace's 1774 principle is the
  rule for equally likely causes, which is exactly what the dots are (fact
  sheet, S14). "The cell you searched drops, every other cell rises" holds
  for cells with some chance and a search that can see; the fact sheet
  gives the range.
- **Dev:** the two lines at 07_2 are the update; the choice is at 08_3.

## 08_where_next (73 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | ⟨predict⟩ Always search where the next search is most likely to find it. Is that really the best you can do? | The loop keeps turning by itself: B, C, A, A, B, D, A, C. Each search's dots drop as a bar onto a running stack beside the sea; the stack's top traces a curve: 20, 33, 43, 51, 58, 64, 70, 74 (`F-U.best.order`, `F-U.best.found_by.percent`). Axis: "dots put out so far, of the first hundred" | a curve built step by step from the dots; axes | |
| 08_2 | Yes, and you can see why. Each search of a cell puts out a fixed slice of the first hundred dots, whatever was searched in between. And a cell's slices only shrink. | The stack comes apart into four columns, one a cell, each a row of shrinking bars: A 10, 7½, 5.6; B 20, 6⅔, 2.2; C 13⅓, 4.4; D 6⅔, 2.2 (`F-U.slices.*`) | bars sorted in place | |
| 08_3 | So the best eight searches are just the eight biggest slices, and the rule takes them biggest first. For a wreck that stays put, and searches that are fresh tries of equal effort, no other order is ever ahead. | The eight tallest bars light gold and fly back, tallest first, into the stack: the same curve. Small: "every order of eight searches, 4 × 4 × 4 × 4 × 4 × 4 × 4 × 4 = 65,536, checked one by one" (`F-U.brute.orders`). Lower-third, in the hold: "M. C. Chew, 1967: after any number of searches, the least chance of still searching." The last two lines of the code panel | morph that keeps identity; lower-third; code panel | |
| 08_4 | On the Navy's map the difference is large. Suppose every search sees a wreck in its cell two times in three. One plan has no map, and goes row by row. The other follows the rule. | Left: chapter 02's shot, the strip on row 1 of the empty grid. Right: the 1968 map with its numbers opened back into dots; the strip crosses E5 with the search sound, two thirds of its dots go out and the rest of the map deepens; then B7, C5, G5 (`F-R.model.best.first_cells`). "model: the lettered grid; every search sees 2 in 3; one search, about 28 sea miles of towing" (`F-R.model.seeing`, `F-R.model.search_miles`) | two grids recoloured in place; the dots; live counters of searches | |
| 08_5 | By the map's own reckoning, twenty searches row by row have less than one chance in a hundred of a find. Twenty by the rule, about one in two. | Between the maps one chart on 08_1's axes, three curves growing to twenty searches: row by row 0.8%, "the map, never updated: each cell once, likeliest first" 44.3%, the rule 49.8% (`F-R.model.sweep.found_by.20`, `F-R.model.once.found_by.20`, `F-R.model.best.found_by.20`). "chance of a find by now, according to the map" | a curve per plan; live numbers | hold |

- **The proof, not a census** (review R1): the picture is the argument. A
  search of a cell removes a fixed share of the original dots, p × d, then
  p × d × (1 − d), and so on, whatever happens between; within a cell the
  shares shrink; so the best n searches are the n largest shares, and the
  rule takes them in order. That every order of eight was also tried is a
  small line and a test (`F-U.brute.rule_is_best`).
- **Always sentence and its edge:** "no other order is ever ahead", with its
  three conditions spoken: the wreck stays put, each search is a fresh try
  (independent, with a known chance of seeing), and searches cost the same
  effort. Tested by every order of 1 to 8 searches and by the slices
  (`the_rule_takes_the_biggest_slices_first`). Ties are why the word is
  "ahead": row by row is level with the rule after 3, 5 and 6 searches of
  the made-up sea.
- **Honest size of the claim** (review R1): on the 1968 grid, a plan that
  has the map and never updates it already reaches 44.3% in twenty
  searches; the loop adds five and a half points there and more later (57.8
  against 73.2 at eighty-one, `F-R.model.*.found_by.81`). The screen's third curve says so (the narration's "most of that is simply
  having a map" was cut for length). All three plans search the same 180
  lettered cells. With equal seeing everywhere, the rule here is "the
  likeliest cell after each update"; the screen says so.
- **By the map's own reckoning** (review R1): the percents are the map's
  belief, averaged over where it thinks she might be. They are not said of
  "her": the viewer knows where she was.
- **Model, not history:** nothing here is what the Navy did; chapter 13 is.
- **Dev:** the rule is a merge of four sorted lists; with unequal costs it
  ranks by chance of a find per unit of cost, and moving between cells is
  taken to cost nothing (description).

## 09_the_second_pass (84 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Why would a search see a wreck only two times in three? In the Second World War, hunting submarines, Bernard Koopman and his colleagues asked how much of an area a searcher really covers. | One flat cell of the made-up sea fills the stage, B's thirty dots in it. Lower-third: "B. O. Koopman, Search and Screening, 1946" (`F-N.koopman`) | lower-third | |
| 09_2 | With perfect strips, laid side by side, enough track to pass over a cell once would cover all of it. | A hundred strips, each one percent of the cell wide, fill it edge to edge, none overlapping; a readout runs to 100% (`F-K.perfect.1`) | live readout recounted from the swept area | |
| 09_3 | But if your strips land at random, some overlap and some leave gaps. The same track then covers, on average, sixty-three percent. | The cell again: the same hundred strips fall at seeded random offsets, across and down (`F-K.lanes.seed`); gaps stay dark; the readout stops at this run's value, labelled "this run" (`F-K.lanes.one_run.1`). It travels into a plot as the end of its own curve; a band of a thousand other runs fills, and their mean is labelled 63% as it is said (`F-K.lanes.mean.1`) | live readout; a curve; a band for many runs | |
| 09_4 | Koopman's formula gives that number, and the rest of the curve. A little more track, and you are at two in three. That is one search of a flat cell. | The formula's curve is drawn over the mean. Two marks on it: "2 in 3: a flat cell" at 1.1 times the track, "1 in 4: the canyon" at 0.29 (`F-K.coverage.flat`, `F-K.coverage.canyon`). The formula as TeX on the plot. Lower-third, in the hold: "the random search formula. Koopman: 'a proper estimate, on the conservative side'" | formula; curve; lower-third | |
| 09_5 | ⟨predict⟩ Run that search twice over one cell. The first pass catches six dots of nine. The second is a fresh try, and catches two in three of what is left. How many of the nine are caught? | The violet frame and the ring. Nine dots in a cell: six ringed and gone out, three still lit | **the quiz frame**; countdown ring | listen 6 s (countdown) |
| 09_6 | Eight. One dot in nine is still there, and a third pass leaves one in twenty-seven. Each pass finds less than the one before, and no number of passes makes it certain. | Two of the three take a ring and go out; one dot is still lit: "1 of 9" (`F-M.seen_within.2`). Then 27 dots and three passes: 18, 6, 2, and one still lit (`F-M.seen_within.3`). The three points land on the chapter's curve | the dots; count-up | hold |

- **One fact, not two** (review R1): the chapter asks why two in three and
  answers it: sixty-three percent at exactly one coverage, two in three at
  a little more (1.1). The eight in nine of the quiz is the same curve at
  twice that, and the dots show why: each fresh pass finds the same share
  of what is left.
- **Earned formula** (plan rule 6): the formula is shown only as the curve
  through numbers the viewer has just watched accumulate. On the plot, in
  small type: "e: a fixed number, about 2.718" and "(1 − 1/100)¹⁰⁰ = 36.6%
  unswept", which is where the e comes from.
- **Said equals shown:** "sixty-three" is the formula's 63.2 and the mean of
  the thousand runs (63.4); the one run on screen shows its own number,
  66.5, labelled "this run".
- **Always sentences and their edges:** "Each pass finds less than the one
  before" and "no number of passes makes it certain" hold for a search that
  can miss and whose passes are fresh tries (`no_number_of_passes_makes_it_
  certain`); the quiz says "a fresh try" aloud. Chapter 13 shows what
  happens when passes are not: a sensor that cannot see the target sees
  nothing on every pass.
- **Misconception defused:** "search an area twice and you have surely seen
  everything in it".
- **Ruth:** the strips on screen are Koopman's own derivation (p. 28): n
  pieces of track, each placed independently. "If your strips land at
  random" is the condition; the formula is the limit of thin strips.
- **Dev:** the dots that go out carry a teal ring first, the same grammar
  as chapter 06.

## 10_adrift (68 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | A wreck stays where it is. A person in the water does not. Wind and current carry them, and nobody knows either one exactly. | Open water. A small open diamond, "last known position"; arrows for wind and current, each with a small fan of uncertainty. Lower-third, in the hold: "leeway: the push of the wind on what floats" | arrows; lower-third | |
| 10_2 | ⟨predict⟩ So every dot moves with its own guess at the wind and the current. The map drifts, and it spreads. What does that do to a search? | The diamond becomes a cloud of 2,000 dots (`F-D.dots`, seed `F-D.seed`) that drifts across the stage and widens; a circle follows its edge; a time line along the bottom with a moving mark: 1, 3, 6, 12 hours; "model" | the dots as a swarm, moved by placement; a number line of hours; a circle that grows | |
| 10_3 | And a person is hard to see. From a helicopter, the Coast Guard counts on spotting someone in the water within a strip only a tenth of a mile wide. | The cloud holds still. A track across it, its strip drawn to scale: a hairline. "sweep width (the width of the strip) 0.1 sea mile, 185 m: person in the water, helicopter at 300 feet. US Coast Guard, Table H-15" (`F-D.strip`, `F-D.strip.metres`) | the search; lower-third: sweep width | |
| 10_4 | Twice the radius is four times the area, so each square mile gets a quarter of the track. In this model, a flight that would find the person ninety-three times in a hundred after one hour finds them thirteen times in a hundred after twelve. That is why hours matter. | The cloud and the diamond leave. Two outline circles, one twice as wide, each in its square, the same zigzag track in each: four of the small squares tile the large one, "coverage = strip × track ÷ area: × ¼" (`F-D.double.coverage_factor`). Then chapter 09's curve. Four markers slide down it as the hours pass: coverage 2.7, 1.0, 0.4, 0.14 give 93%, 65%, 34%, 13% (`F-D.h1.coverage` to `F-D.h12.seeing`), each with its small cloud above. "model: the same 180 sea miles of track spread evenly over the circle" | a curve; live markers; "model" | hold |

- **The owner's "time is running out", made exact.** The cloud's radius
  grows with the distance drifted, so its area grows as the square, and the
  same track covers less of it; chapter 09's curve turns that into a
  chance.
- **No clock on a person** (review R1): time is a line with a moving mark,
  never a ticking face; no countdown runs in this chapter, and the doubled
  radius is drawn on two outline circles after the cloud has left the
  stage.
- **Model numbers** (all marked "model", in sea miles): a start known to
  within a mile, a drift of a mile and a half an hour, a drift error of
  three tenths of the distance drifted (the Coast Guard's long-standing
  rule of thumb, `F-D.error_share`), each dot keeping one error for the
  whole drift, and one flight of 180 miles of track spread evenly over the
  circle. Only the strip width is the Coast Guard's. A flight fitted to
  the cloud does better (73% at hour 3, `F-D.flight.h3.find`); the
  description says so.
- **Misconception defused:** "a later search just has farther to go". It
  has more water to cover with the same track.
- **Ruth:** nothing here claims the rule of chapter 08 is still the best
  plan for a moving target (description: Brown 1980). The strip is an
  effective width, not a hard edge.

## 11_twelve_hours (73 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | The United States Coast Guard has had software that searches this way since nineteen seventy-four. It moves thousands of dots, each a possible drifting person or boat, and updates them by Bayes' rule after every empty search. | The drifting cloud from chapter 10 with a search rectangle laid over part of it and the search sound: of the dots inside, the share the search would have seen go out; the rest of the cloud deepens. Lower-third, in the hold: "CASP, 1974. SAROPS, since 2007: thousands of particles for each scenario" (`F-C.casp`, `F-C.sarops`) | the dots recoloured in place; lower-third: particle | |
| 11_2 | In July of twenty thirteen, a lobsterman named John Aldridge fell off his boat at night, about forty miles off Long Island. Nobody saw him go. | A coast line at the top (Long Island, Montauk marked) and open water below; a boat's track heading south; a hollow white point left behind on it. An hour line down the side with a moving mark. "24 July 2013" (`F-C.aldridge.date`) | a number line of hours | |
| 11_3 | The searchers had to guess when he fell, and guessed the evening before. So the first map put its dots almost thirty miles too far north. | The hour line: "first guess: 9:30 to 11:30 p.m." in red (`F-C.aldridge.guess`). A cloud of dots near the coast, far from the white point (`F-C.aldridge.miles_south`) | the dots; the wrong estimate in red | |
| 11_4 | ⟨predict⟩ The map was wrong because its story was wrong. What could fix it? | The cloud and the distant white point, held | a question in the caption band | |
| 11_5 | Evidence. On deck, his partner found an open hatch and pumps running, work Aldridge would not have started until far offshore. He had fallen hours later. | Two small marks on the boat's track far to the south. The mark on the hour line begins to slide down, from 10:30 p.m. toward 3:30 a.m. (`F-C.aldridge.fell`, `F-C.aldridge.hours_off`); the red window stays behind, struck out | one tracker: the time of the fall | |
| 11_6 | The Coast Guard ran the map again, and the dots moved south. That afternoon a helicopter flying through them spotted him, alive, clinging to a lobster buoy, after almost twelve hours in the water. | As the time of the fall slides, the point where he left the boat slides down the track and the cloud, recomputed from it, travels south and settles around the white point. A track crosses it; one plucked note; the point turns green. "seen about 2:58 p.m.: 11 hours 28 minutes" (`F-C.aldridge.seen`, `F-C.aldridge.in_water`) | the dots as a swarm; a find | hold |

- **A living person, found:** the half of the owner's question that the
  *Scorpion* could not answer. The only green in the
  video until its last picture.
- **Source and its limits** (fact sheet, finding 7): the account is the New
  York Times Magazine's, reported from the command centre; the Coast
  Guard's releases do not mention the software. The last track was drawn by
  hand through the high-chance area after the software froze, so the script
  says "a helicopter flying through them" and credits no program with the
  find. He had swum to lobster buoys and held on, which the narration says
  ("clinging to a lobster buoy"): he was not a free drifter, and his twelve
  hours are not chapter 10's thirteen in a hundred.
- **Misconception defused:** "the computer knows where he is". It knows
  what it was told; a wrong time of fall moved the whole map thirty miles,
  and no empty search could have moved it back (14_4).
- **Ruth:** "has had software that searches this way since 1974": CASP was
  put into operation that year and "used a Bayesian approach" (S6). The
  Coast Guard's bookkeeping leaves the map un-normalized on purpose.
- **Dev** (description): an empty search multiplies a dot's weight by its
  chance of being missed; weights grow uneven, which is why there are
  thousands of dots.

## 12_the_silence (78 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | June, two thousand nine. Air France flight four four seven, from Rio to Paris, went down in the Atlantic. All two hundred and twenty-eight people aboard were killed. | A flight track; the last reported position as a small open diamond. Then a cleared stage: "1 June 2009. 228 people" in ink (`F-A.aboard`, `F-A.lost`), held on the last sentence | a card for the 228 | |
| 12_2 | The wreck had to lie inside a circle forty nautical miles in radius. In nearly two years, three underwater searches found nothing. | The circle around the diamond, "40 nautical miles (74 km)" (`F-A.circle.nm`, `F-A.circle.km`). Three searched areas shade in, 2009, 2009, 2010 (`F-A.underwater_searches`); "22 months" (`F-A.months`) | lagged reveal | |
| 12_3 | Then a team of search analysts built a map from three smaller maps, weighted thirty-five, thirty-five and thirty, the way we weighted two stories, and updated it for every empty search. | Three small discs, labelled by method: "anywhere in the circle", "near the last position", "traced back from what was found afloat", with weights 35, 35, 30 (`F-A.weights`). They cross-fade into one cloud of 3,000 dots: "75,000 particles; each dot here stands for 25" (`F-A.points`). The searched areas put out their share. "weights chosen by judgment" | the dots; bars | |
| 12_4 | ⟨predict⟩ One search needed a decision. For a month, ships had listened for the beacons on the plane's black boxes, and heard nothing. How much should that silence count? | The listening tracks across the circle. "heard nothing" | a question in the caption band | hold (two seconds with nothing sounding) |
| 12_5 | If the beacons worked, a great deal. If they were broken, not at all. So the analysts drew the map twice, once counting the silence, and once leaving it out. | Two discs side by side, the same size and orientation. Left: "the silence counts: 77 in 100 it would have been heard" (`F-A.beacon.used.percent`), dots out along the listening tracks. Right: "the silence left out", those dots still lit. Lower-third, in the hold: "the fuller way, in the authors' words: one map, with a weight on whether the beacons worked" | two dot clouds recoloured in place; lower-third | |
| 12_6 | Both maps put their weight near the centre of the circle. The search resumed there, and found the wreck after about a week, in a high-chance part of the second map, and in water the listening ships had already crossed. | A hollow white ring north-northeast of the diamond, at the same place on both discs, "about 6.5 nautical miles (12 km) from the last position" (`F-A.found.nm`, `F-A.found.km`); last, a listening track is drawn across it. "April 2011. The investigators planned from the first map" | the true place | hold |

- **Source:** Stone, Keller, Kratzke and Strumpfer (2014) and the BEA's
  final report. The 228 are said plainly, as the 99 were.
- **What the source supports, and no more** (review R1): the investigators
  planned the 2011 search from the first map (p. 75), which already allowed
  for dead beacons; both maps are dark near the centre; the paper marks the
  wreck on the second map only and calls that place "a high probability
  area". So the ring is drawn the same on both discs, and the script does
  not say the first map hid the wreck or that the silence was "trusted too
  far". What the authors blame for the lost year is the 2010 search, which
  was planned without a map of this kind.
- **Not said** (fact sheet, finding 6): that the wreck was in the top cell;
  that the analysts planned the search; that the beacons are known to have
  failed.
- **Misconception defused:** "they heard nothing there, so it is not
  there": "B is ruled out" from chapter 06, at full scale.
- **Ruth:** the authors' own better way (a joint distribution over the
  beacons' state and the wreck's place) is on screen in their words; it is
  chapter 03's move, a weight on a story.
- **Engine:** 75,000 dots cannot be drawn; 3,000 are, and say so.

## 13_how_well_they_looked (82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | The Navy had faced the same question in nineteen sixty-eight. Its map reached the search ship in August, two months into the search, and every pass of the sled since June was scored against it. | The 1968 map. A date line: 10 June, search begins; 12 August, the map arrives (`F-S.search_began`, `F-S.map_arrived`, `F-S.months.search_to_map`). In the hold, 07_3's line turns part by part into the one the paper printed: L′ = L (1 − E) ÷ (1 − SEP), with word labels carried across: "L: the chance she is in the cell. E: how thoroughly it has been searched. SEP: the chance the search so far would have found her". "Richardson and Stone, 1971, p. 146" | a grid; TeX morph that keeps its labels; a number line of dates | |
| 13_2 | By October, the ship's numbers said one cell had been searched ninety-four percent thoroughly. Of every hundred dots there, ninety-four went out. It was the cell with the wreck in it. | F6 enlarged beside the map, "F6, per 100 dots"; 94 go out (`F-S.F6.searched.at_sea`, `F-S.F6.left.at_sea.percent`). The white ring appears among the six still lit | the dots; the true place | |
| 13_3 | ⟨predict⟩ The wreck was there, and the sled had been over that water. What went wrong? | F6, held, six dots lit | a question in the caption band | |
| 13_4 | The estimate of how well they were looking. At sea, they believed the sonar could see a strip eight hundred feet wide. | A strip to scale across a cell, "believed at sea: sonar 800 ft" in red (`F-S.sweep.at_sea`) | bars drawn to scale; the wrong estimate in red | |
| 13_5 | Measured afterwards against the wreck itself, the sonar that passed her had not seen her at all. The camera and the magnetic detector saw about two hundred feet. | Under it, to the same scale: "measured afterwards: camera 185 ft, magnetometer 220 ft"; for the sonar a baseline tick and the numeral 0, no bar (`F-S.sweep.after`). Chapter 09's curve with two marks: "believed: 2.8 times over", "measured: 1.05" (`F-S.F6.coverage.at_sea`, `F-S.F6.coverage.after`) | bars; a curve | |
| 13_6 | So the true figure for that cell was sixty-five percent, not ninety-four. And ten more passes with that sonar would have changed nothing. | In F6, dots relight until 35 are lit (`F-S.F6.searched.after`, `F-S.F6.left.after.percent`). Under the cell, its chance on the map: "before any search 0.85%. By the ship's numbers 0.19%. By the measured ones 0.54%" (`F-R.F6.percent`, `F-S.F6.chance.at_sea`, `F-S.F6.chance.after`) | the dots relighting, the only time in the video | |
| 13_7 | She was found at the end of October, when the sled went back to where a scrap of metal had been photographed that summer, beside the map's top cell. | A small mark on the border of F5 and F6, "photographed in the summer; not found again". The ring in F6 stays white. "28 October 1968" (`F-S.found`). Lower-third, in the hold: effort went to F6 "because of proximity to the piece of metal and the fact that this was a region with high a priori [prior] probability" | the true place; lower-third | hold |

- **The uncomfortable part, told** (fact sheet, findings 1 and 3): the map
  came two months late; the on-scene numbers had nearly written off the
  right cell; a piece of metal and the prior together brought the sled
  back. The numbers are the paper's Tables 1 to 3, for 9 October. Even
  scored with the measured widths, F6 had fallen from 0.85% to 0.54%; the
  ship's numbers put it at 0.19%.
- **The same question, not the same mistake** (review R1): 13_1 links back
  to "how much should an empty search count?", which is what chapter 12
  asked; nothing says the AF 447 analysts erred.
- **The opening picture, in its rigorous form** (plan rule 15): the update
  the viewer did with dots turns into the line the paper printed, label by
  label. The early passes were scored against a late map, and it made no
  difference: the order of the evidence does not matter (description).
- **One mechanism** (review R1): the paper scored each cell with Koopman's
  formula, so a strip believed four times too wide becomes 94 percent
  through chapter 09's curve.
- **Fresh tries, and when they are not:** "ten more passes with that sonar
  would have changed nothing" is the edge of chapter 09's "each pass finds
  less": a sensor that cannot see the target finds nothing on every pass.
  The paper's zero rests on one of the two sonars passing without a
  detection; the narration says "the sonar that passed her".
- **Misconception defused:** "more searching always sharpens the map".
- **Words:** "the true figure", not "the honest figure". "Magnetic
  detector" is spoken and "magnetometer" is on screen.

## 14_four_maps (85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | An empty search only tells you something if it could have found what you were looking for. | Before the words, three seconds of the opening shot with a strip that sees nothing: it crosses a cell, no dot goes out, no cell changes. Then the line alone, in ink | the dots; a card | hold |
| 14_2 | Four searches, each against its map. Palomares, found near where a fisherman watched it fall. Scorpion, beside the map's top cell. Air France, near the centre both maps pointed to. John Aldridge, inside the dots, once the story was put right. | Four panels in a row, each the last frame of its story, built left to right as they are named: the coast, the fisherman's mark and the ring beside it; E5 and the ring in F6, "widths re-measured after the find"; the two discs and the ring, "a second map"; the cloud moved south and the green point, "re-run with new evidence" | a table of four pictures built from primitives; lagged reveal; highlight | |
| 14_3 | ⟨predict⟩ A strip believed too wide. A silence that might have meant nothing. A story five hours off. So what does the method ask of you? | The three flaws light in red inside their pictures: the 800-foot strip, the listening tracks, the first cloud up the coast (`F-C.aldridge.hours_off`) | highlight; a question in the caption band | |
| 14_4 | Not a first map that is right. A first map that leaves room for the truth. An honest account of how well you looked. And a new map, when new evidence says a story was wrong. | The panels fold away. The cold open's shot again, frame for frame, each part named as its clause is said: the cloud, "what you believe"; the strip, "how well you looked"; the strip crosses with the search sound, the dots go out, the map deepens. In the hold two lines set under it: the update, and in gold "search where the chance of a find is greatest". Small: "the update multiplies and never adds: a place with no dots can never gain one" (`F-U.zero_stays_zero`) | the dots; formula | hold |
| 14_5 | The ninety-nine men of the Scorpion could not be saved. | The stage clears by the ordinary fade before the sentence. Chapter 02's card returns exactly: the white ring and "99 men", the same place and size | a card for the 99 | hold (a silence: the scene waits 2.5 s with no music breath) |
| 14_6 | The method that searched for them now searches for people who can. | The ring and its label stay. Beside them, chapter 11's last frame: the cloud, the track, the green point | the dots | hold |
| 14_7 | *(receipts)* | "Every number in this video is in the code, and each was checked a second way." The repository link; "narrated by synthetic speech"; names to search | receipts card | listen 4 s (the motif's last chord) |

- **What the method asks** (review R1, the round's one blocker): the first
  draft said the method "never needs the first map to be right". That is
  false at its edge: the update multiplies and never adds, so a place given
  no dots can never gain one (`a_cell_with_no_dots_never_gains_one`), and
  each story here was put right from outside the loop (widths re-measured,
  a second map, new evidence), as the labels under the panels say. What is
  true is on screen and spoken: the first map has to leave room for the
  truth. The made-up sea's second story ("could be anywhere") and AF 447's
  35 percent on "anywhere in the circle" are that room.
- **One clean summary** (the order): four maps side by side, each with where
  the thing was found against it, and what put the map right.
- **The 99, with respect** (review R1): no dot dims as they are named; the
  stage clears first and chapter 02's card returns unchanged. The sentence
  about the living comes after a silence, and the music resolves there,
  once, and not on the dead.
- **Names to search** (plan rule 26), on the receipts card: Bayesian search
  theory; Koopman, "Search and Screening"; Richardson and Stone 1971; Stone,
  "Theory of Optimal Search"; SAROPS.
- **Ruth:** "the method that searched for them" is the lineage the Coast
  Guard's own developers give (S6, S13). "A story five hours off" is the
  first guess's middle, 10:30 p.m., against 3:30 a.m. "A silence that might
  have meant nothing" claims nothing about what the beacons did.
