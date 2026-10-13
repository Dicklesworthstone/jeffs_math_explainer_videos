# Storyboard — How to Find a Lost Submarine

The hand-edited source of this video's words and pictures. `python3 -I
tools/storyboard.py bayesian_search` checks it (gate G2) and writes
`script.tsv` from the words column. Facts: `docs/fact-sheet.md` and
`oracle.py` (every number below names its fact key); brief: `docs/brief.md`;
reviews and their dispositions: `docs/reviews/bayesian_search/`.

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a question for the
viewer; the beat ends with the question and the picture holds before the
next beat answers it.

**Two kinds of question, two looks.**

- **Your turn** (a quiz): a `⟨predict⟩` beat whose last column says
  `listen N s (countdown)`. A violet card with the question; a ring that
  drains over N seconds with the seconds left in its middle; the line "pause
  for more time" under it; the narration silent and the music alone, one
  mallet note a second. At zero the answer lands, large, where the ring was.
  The next beat then works it on the same picture. Every quiz is a row of
  `model::QUIZZES`, and its numbers are the `F-Q.<name>.*` facts.
- **A question to carry** (01_1, 02_6, 04_4, 08_1, 11_4, 12_4, 13_3, 14_6):
  the question as a card with no ring, held about two seconds. The owner's
  subject line is the first of them.

**The dots.** The video's one picture. A dot is one place the lost thing
could be; the map is the cloud of them; a cell's chance is its share of the
dots still lit. A search is a teal strip crossing a cell; the dots it would
have found go out (they turn `faint` and stay where they are, so the viewer
can still see how many there were); the percentages of the cells are
recounted live from the lit dots. Nothing on screen is ever a number that
the dots do not show.

**The small sea.** Four cells in a two-by-two block, A and B above C and D,
with a hundred dots: 40, 30, 20, 10 (`F-M.prior.dots`). A is drawn as a
canyon (contour lines; the search strip thins as it crosses) and is the same
in every chapter that uses the small sea (03, 05, 06, 07, 08, 09).

**The 1968 map.** Richardson and Stone's Figure 2 redrawn as it was printed:
the lattice, the column letters A to O and row numbers 1 to 12, every one of
the 172 printed numbers (`model::y1968::POINTS`), each cell's heat from its
number. It appears three times: built from ten thousand dots (04), under a
model search (08), and with what really happened (13). The source line
"Richardson and Stone, 1971, Figure 2" is on screen whenever it is.

**Model numbers.** The word "model" is on screen with every number that
comes from the video's own assumptions (02_5, 08_3 to 08_5, 10_2 to 10_6).

## 2 The arc

| Act | Chapters | Spine |
|---|---|---|
| I. The question | 01 the question, 02 ninety-nine | the owner's question; the picture that will answer it; the truth about the *Scorpion*; why you cannot look everywhere |
| II. The map | 03 a map of chances, 04 ten thousand dots | two weighted stories make a map of a hundred dots; the Navy's map made the same way, and the wreck one cell from its top cell |
| III. The method | 05 a search can miss, 06 nothing found, 07 Bayes' rule, 08 where next | the likeliest cell is not the best first search; an empty search by hand, in dots; the rule named; the loop, the theorem that no order does better, and a model search over the 1968 map |
| IV. Effort and time | 09 the second pass, 10 adrift | Koopman: why a search sees two times in three, and why a second pass buys less; a target that drifts, and why hours matter |
| V. Four searches | 11 twelve hours, 12 the silence, 13 how well they looked, 14 four maps | the Coast Guard and John Aldridge; Air France 447; what really happened in 1968; one table, the lesson the three stories share, and the 99 |

The payoff is previewed twice before any method is taught: the cold open's
silent picture (01_2) and the real map with the wreck beside its top cell
(04_5). The video ends on the rigorous form of the opening picture: the
update as the 1971 paper printed it (13_1) and the rule with its theorem
(14_7).

**Length.** The G2 estimate below is 15.0 minutes, the top of the order's
range. On the three finished cuts in this voice the real narration ran 3
to 5 percent longer than the same estimate, so the cut may come out near
15 and a half minutes; chapter 08's model search (08_3 to 08_5, about 33
seconds) is the first cut if the owner wants it shorter.

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses a
data colour.

| Idea | Color |
|---|---|
| the map: dots still lit, and a cell's heat | blue `#58C4DD` |
| the search: the searcher, its track and strip, the chance of seeing | teal `#5CD0B3` |
| the chance of a find, and the cell the rule chooses | `hero` gold `#FFD866` |
| dots a search has put out | `faint` `#3B4352` |
| the true place (wreck, bomb, person), shown only once it is known | `ink` white, a small ring |
| a find | `good` green `#83C167` |
| an estimate that turned out wrong (11_3, 13_2, 13_4, 14_6) | `warn` red `#FC6255` |
| the "your turn" card and its countdown | violet `#B189D6` |

Pairwise contrast (CIEDE2000, with the background) is pinned by a test in
the crate before any draft.

## Vocabulary (one word per idea)

**map**; **cell** (never square, box, sector or zone); **dot**; **story**
and its **weight**; **chance** (never "probability" in the narration, which
is on screen where the formal names are given, and never "odds");
**the chance she is there**, **the chance of seeing her**, **the chance of
a find**; **search** (one pass over one cell); **comes back empty** (the
search found nothing) and **miss** (it was there and was not seen): never
swapped; **update**; **track**, **strip**, **pass**; **drift**. A ship is
"she" where the Navy's own usage is being followed (the *Scorpion*); the
generic lost thing in the small sea is "the wreck" or "she". Never
"could have been saved", "miracle", "genius", "magic". The formal names
appear once each, in a lower-third: scenario, prior, posterior, detection
probability, Bayes' rule, sweep width, coverage, the random search formula,
particle, leeway.

## Sound plan (for the score bead)

Nothing here is auditioned yet; the drafts of P5 are silent but for the
scratch voice.

- **Countdowns:** one mallet note a second, the last three always the same,
  landing on the tonic as the answer appears (as in Video F).
- **02_3:** after "All ninety-nine men were lost", two and a half seconds of
  the bed alone, no melody.
- **A search:** one soft swept sound as a strip crosses a cell; a dot going
  out makes no sound.
- **14_8 and 14_9:** the motif resolves here and nowhere earlier.

## 01_the_question (21 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | ⟨predict⟩ A nuclear sub has gone missing. Men are trapped, and time is running out. How can we possibly find them in time? | Dark water. The three sentences set themselves in ink, large, one line each, as they are said | a card: the owner's question | |
| 01_2 | Start with a strange idea. A search that finds nothing has still told you something. | The words lift away. A faint grid of cells over the whole frame, and a cloud of blue dots on it, thick in one place. A teal strip crosses one cell: most of that cell's dots go out. Every other cell's share bar rises a little. No labels, no numbers | the dots; a grid; live bars recounted from the dots; highlight on the swept cell | |
| 01_3 | *(title)* | **How to Find a Lost Submarine**, over the dots, which dim | title card | listen 3 s (the motif, unanswered) |

- **Cold open** (plan rule 20): the owner's subject line in his words, then
  the payoff as a paradox (finding nothing is information) with the picture
  that will explain it, then the title. About 15 seconds before the title.
- **Honesty:** the question is asked before any ship is named; the next
  chapter's third line answers it for the *Scorpion*.

## 02_ninety_nine (67 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | This happened. In May of nineteen sixty-eight the U S S Scorpion, a nuclear submarine, was heading home across the Atlantic with ninety-nine men aboard. | The Atlantic as a dark field with two labelled points, the Azores and Norfolk, and a thin track between them; a small ink point moves along it. "USS Scorpion (SSN-589), May 1968" and "99 men" (`F-S.crew`) | a number line of dates along the bottom; lagged reveal | |
| 02_2 | She was due at Norfolk, Virginia, on the twenty-seventh, and did not arrive. For nine days the Navy searched the ocean, as if her crew might still be saved. | The date line: 27 May, "due" (`F-S.due`); the point is no longer on the track. Small teal search marks spread along the whole track for nine days, to 5 June (`F-S.days.due_to_presumed`). "aircraft and dozens of ships" | count-up of days; lagged reveal | |
| 02_3 | They could not be. She had gone down five days before she was due, in water about two miles deep. All ninety-nine men were lost. | The date line runs back to 22 May (`F-S.sank`, `F-S.days.sank_to_due`): a small white ring where she lies, "more than 10,000 feet" (`F-S.depth.feet`). The search marks fade. Then only the ring and "99" | a card for the 99, in ink, nothing else on screen | hold |
| 02_4 | Nobody knew that yet. And even to find the wreck, the Navy faced the first question of any search. Where do you look? | The ring goes out (nobody knew). A patch of sea southwest of the Azores is outlined: "about 400 miles southwest of the Azores" (`F-S.miles_from_azores`) | highlight | |
| 02_5 | Not everywhere. Sound recordings had narrowed the search to a patch of sea twelve miles on a side. But the sled that carried the search gear saw a strip only two hundred feet wide. One pass over the whole patch meant forty-six hundred miles of towing. By the end of October it had managed a thousand. | The patch fills the frame as the 1968 grid, empty, 12 rows by 15 columns (`F-R.grid.miles`). A teal strip one hair wide crawls along the top row. Counter "to pass over all of it once: 4,594 miles (model: a 200-foot strip, 180 cells of 1 × 0.84 miles)" (`F-H.once.miles`, `F-H.strip.feet`, `F-R.grid.cells`). Then a second counter, "towed, June to October: 1,026 miles" (`F-H.track.miles`), and a bar of each (`F-H.once.over_track`) | a grid; live counters of miles; the strip drawn to scale; bars | |
| 02_6 | ⟨predict⟩ So where would you look first? | The empty grid, held | a question card | |

- **The truth, early** (the order): 02_3 says it in the third line of the
  first real chapter, and the score holds on the bed alone after it.
  "As if her crew might still be saved" describes the search of 27 May to
  5 June and nothing else.
- **The obvious approach fails on the real numbers** (plan rule 2): the
  strip width is the after-search estimate for the two sensors that worked
  (185 and 220 feet, `F-S.sweep.after`); the 1,026 miles is the paper's own
  figure for the whole search up to detection, 10 June to 28 October.
- **Misconception defused:** "they just had to search the area carefully".
  Carefully, once, is four and a half times what five months allowed.
- **Ruth:** "sound recordings" is all that is said of the acoustic work; the
  secondary sources disagree on how many signals and how small an area, and
  the grid's own size (12 by 12.6 miles) is what the picture uses.
  "About two miles deep" is 10,000 feet (`F-S.depth.miles`: 1.9).

## 03_a_map_of_chances (75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | The natural answer is, wherever she most likely is. So you need a map of how likely each place is. | The big grid shrinks away to the left. Four cells, two by two, labelled A, B, C, D | a grid; morph | |
| 03_2 | Start small. A sea of four cells, and two stories of what might have happened. | Two story cards above the cells: "she held her course" and "something turned her" | lagged reveal | |
| 03_3 | In the first story she held her course. Then she is probably in cell A, maybe in B, less likely in C. | Card one's shares appear as bars over the cells: a half, a third, a sixth, nothing (`F-M.story1.share`) | bars | |
| 03_4 | In the second, something turned her, and she could be anywhere. | Card two's shares: a quarter each (`F-M.story2.share`) | bars | |
| 03_5 | People who know submarines weigh the stories. Say sixty to forty. Now picture a hundred dots, sixty from the first story and forty from the second. Each dot is one place she could be. | The weights 60 and 40 land on the cards (`F-M.story1.weight`). Sixty dots leave card one and settle: 30, 20, 10 (`F-M.story1.dots`). Forty leave card two, but hang above the cells, not yet settled | the dots; a count-up per cell | |
| 03_6 | ⟨predict⟩ The first story puts half of its dots in cell A. The second puts a quarter of its dots there. What is the chance she is in cell A? | The quiz card: "half of 60, plus a quarter of 40, out of 100 = ?" | **the quiz card**; countdown ring on a tracker | listen 6 s (countdown: the music alone, one mallet note a second) |
| 03_7 | Half of sixty is thirty. A quarter of forty is ten. Forty dots in a hundred. Forty percent. | Card two's forty dots settle, ten to a cell (`F-M.story2.dots`). In A, 30 + 10 = 40 (`F-Q.stories.answer`) | count-up | |
| 03_8 | Count every cell that way and you have the map. Forty, thirty, twenty, ten. | The cards fade. Each cell shows its dots and its percent: 40, 30, 20, 10 (`F-M.prior.percent`) | live numbers recounted from the dots | hold |

- **Show, then name:** "scenario" and "prior" are not said. A lower-third at
  03_8: "the prior map: what is believed before any search". "Scenario"
  appears once, at 04_1, on screen.
- **Misconception defused:** "a map of chances needs data about where
  submarines sink". It needs stories and honest weights; the weights are
  judgments, and the video says "say sixty to forty".
- **Ruth:** the chance of A is the weighted sum 0.6 × ½ + 0.4 × ¼
  (the law of total probability), done as dots. The two stories and their
  weights are the video's own illustration and are said to be ("Say").
- **Maya:** every number here is a count of dots she can see.

## 04_ten_thousand_dots (56 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | The Navy's map was made the same way. In July, John Craven, who led the search's technical advisers, and a colleague named Andrews set out nine stories of how the Scorpion might have been lost, and gave each a weight. | Nine blank story cards in a row, each with a weight bar drawn as an outline (the paper does not print them). "18 July 1968: nine scenarios, each with a credence" (`F-S.scenarios`, `F-S.scenarios_set`) | lagged reveal; lower-third: scenario | |
| 04_2 | A mathematician, Henry Richardson, had a computer play those stories out ten thousand times. Ten thousand dots. | The 1968 lattice, empty. Dots rain onto it, a counter running to 10,000 (`F-R.replications`); they pile where the map will be high | the dots; live counter; a grid | |
| 04_3 | Then he counted the dots in every cell. This is the map they printed. | The dots in each cell collapse into its printed number, cell by cell in a ripple, and the cell takes its heat. All 172 numbers (`F-R.printed_cells`). "Richardson and Stone, 1971, Figure 2. Divide by 10,000. Each cell about a mile across" (`F-R.cell.miles`) | a grid of numbers built from primitives; ripple | |
| 04_4 | ⟨predict⟩ One cell stands out, with one dot in every eight. They called it E five. Was she there? | E5 glows gold: 1,250 (`F-R.E5`, `F-R.E5.one_in`). The next highest, B7, is marked faintly: 1,096 | highlight | |
| 04_5 | Almost. The wreck lay in a neighbouring cell, two hundred and sixty yards from the edge of E five. | A white ring in the corner of F6 that touches E5, as the paper's asterisk sits: F6 = 85 (`F-R.F6`). "within 260 yards of the edge of rectangle E5" (`F-S.yards`), quoted | highlight; the true place | |
| 04_6 | But the map alone did not find her. It took five months, and a method for using a map like this. | The map dims; the small sea returns beside it | morph | hold |

- **Payoff previewed** (plan rule 3): the real map and the wreck beside its
  top cell, before any method. 04_6 says at once that the map alone did not
  do it; chapter 13 says what did.
- **Misconception defused:** "the map said exactly where she was". She was
  in a cell holding 85 of the 10,000 dots, the 24th highest of 172
  (`F-R.F6.rank`), next to the one holding 1,250.
- **Ruth:** who did what is from the 1971 paper (pp. 143-45, 149) and, for
  which author made the map, the INFORMS profile of Stone; the nine
  scenarios are not listed anywhere that could be read, so the cards are
  blank. Nothing is said about how the weights were arrived at beyond "gave
  each a weight" (fact sheet, finding 4). Each run drew its own course,
  speed and position; the lower-third says so.
- **Dev** (lower-third at 04_2, labelled "for programmers"): a Monte Carlo
  simulation, 10,000 replications; pick a scenario by its weight, draw the
  uncertain quantities, add one to the cell where the run ends.

## 05_a_search_can_miss (68 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Back to the small sea. Forty percent says search cell A first. But a search is not a sure thing. | The small sea with its hundred dots; A's 40% pulses | the dots | |
| 05_2 | Cell A is a canyon. A search there sees the wreck only one time in four. In the flat cells, a search sees it two times in three. | Contours appear in A. A teal strip tries each kind of cell on a spare dot: in A, one pass in four lights it; in a flat cell, two in three. Under each cell: "seen 1 in 4", "seen 2 in 3" (`F-M.seeing`) | lagged reveal; highlight | |
| 05_3 | So ask a better question. How likely is one search of a cell to find her? For that, she has to be there, and the search has to see her. | Under each cell a second, empty readout in gold: "chance of a find". Two bars for A: the 40 dots, and the quarter of them a search would see, in teal | bars | |
| 05_4 | ⟨predict⟩ Forty percent, seen one time in four. Thirty percent, seen two times in three. Which cell gives the better chance of a find, A or B? | The quiz card, with the two cells drawn small | **the quiz card**; countdown ring | listen 6 s (countdown) |
| 05_5 | Cell B. A quarter of forty is ten. Two thirds of thirty is twenty. Twice the chance. | In A, 10 of the 40 dots ring teal; in B, 20 of the 30. The gold readouts: 10%, 20%, and then 13% and 7% for C and D (`F-M.find.percent`, `F-M.first.over_likeliest`) | live numbers from the dots; highlight | |
| 05_6 | The chance of a find is the chance she is there, times the chance of seeing her. So the best cell to search is not always the likeliest. | The line builds over the cells: chance of a find = chance she is there × chance of seeing her. B's border turns gold | TeX equation in words, then p × d under it; lower-third | hold |

- **The second obvious approach fails** (plan rule 2): "look where she most
  likely is" loses to the second-likeliest cell, two to one
  (`F-M.likeliest.find.percent`, `F-M.first.find.percent`).
- **Lower-third at 05_6**, precisely: "detection probability. The Coast
  Guard's name for this product: probability of success = probability of
  containment × probability of detection."
- **Misconception defused:** "a search either covers a place or it does
  not". It sees what is there some of the time; chapter 09 says why.
- **Ruth:** the product needs "seeing" to be conditional on being there,
  which is how 05_2 defines it ("a search there sees the wreck one time in
  four"). The canyon and its one in four are the video's illustration.

## 06_nothing_found (67 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | So we search cell B. And the search comes back empty. | The teal strip crosses B. Nothing lights. "empty" | the search | |
| 06_2 | That is not nothing. Look at the dots. Thirty are in B, and this search would have found two out of three of them. Twenty dots. | Twenty of B's thirty dots ring teal (`F-M.empty1.dots_out`) | highlight; count-up | |
| 06_3 | If she were at any of those twenty, we would have her. We do not. So they go out. | The twenty go out: they turn faint and stay in place | the dots | |
| 06_4 | Eighty dots are left, and she is at one of them. Ten are still in B, because a search can miss. | A counter of lit dots runs from 100 down to 80 (`F-M.empty1.dots_left`); B's ten lit dots pulse (`F-M.empty1.cell_dots_left`) | live counter | |
| 06_5 | ⟨predict⟩ Ten dots out of eighty. What is the chance she is in cell B now? | The quiz card: "10 of 80 = ?" | **the quiz card**; countdown ring | listen 6 s (countdown) |
| 06_6 | One in eight. Twelve and a half percent, down from thirty. | B's readout falls from 30% to 12.5% (`F-Q.after_empty.answer`) | live number | |
| 06_7 | Now look at cell A. Nobody searched it, and it has the same forty dots. But forty out of eighty is one half. Searching B made A more likely. | A's dots do not move; its readout climbs from 40% to 50%. C goes to 25%, D to 12.5% (`F-M.empty1.after.percent`) | live numbers recounted from the dots | |
| 06_8 | Every cell we did not search went up, by the same factor. | "× 100 ÷ 80" beside A, C and D (`F-M.empty1.rise`) | TeX fraction | hold |

- **The centre of the video:** Bayes' rule done as counting, before it has a
  name. Nothing is multiplied by a formula; dots go out and the rest are
  counted.
- **Misconception defused:** "an empty search of B tells you about B and
  nothing else". A did not change and A's chance did.
- **Second misconception:** "B is ruled out". Ten dots are still lit there.
- **Ruth:** this is conditioning on the event "the search of B came back
  empty": 100 equally weighted possibilities, 20 excluded, 80 left.

## 07_bayes_rule (62 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | That move has a name. Bayes' rule, after Thomas Bayes, an English minister who died in seventeen sixty-one. | The small sea moves left. "Bayes' rule" in gold; under it "Thomas Bayes, died 1761. In this form: Laplace, 1774" (`F-N.bayes.died`, `F-N.laplace`) | lower-third: Bayes' rule, prior, posterior | |
| 07_2 | In words. Take each cell's chance. Multiply by the chance that the search would come back empty if she were there. Then scale the whole map so it adds up to a hundred again. | A table with a row for each cell: 40 × 1, 30 × ⅓, 20 × 1, 10 × 1; the products 40, 10, 20, 10; their sum 80; then each over 80: 50, 12.5, 25, 12.5 | TeX derivation: TransformMatchingTex from the words to the numbers to the percents; a table | |
| 07_3 | The cell you searched drops. Every other cell rises. And the map is ready for the next search. | The percents slide back onto the cells. The formula in letters under the table: p′ = p (1 − d) ÷ (1 − p d) for the searched cell, p ÷ (1 − p d) for the rest | formula; highlight | |
| 07_4 | ⟨predict⟩ Here is the new map. Cell A has half the chance now. But A is the canyon. Where do you search next? | The quiz card over the new map: 50, 12.5, 25, 12.5, with "seen 1 in 4" under A and "seen 2 in 3" under the others | **the quiz card**; countdown ring | listen 6 s (countdown) |
| 07_5 | Cell C. A half, seen one time in four, is twelve and a half percent. A quarter, seen two times in three, is nearly seventeen. | The gold readouts: 12.5%, 8.3%, 16.7%, 8.3% (`F-M.second.find.percent`); C's border turns gold (`F-Q.next_search.answer`) | live numbers; highlight | |
| 07_6 | Search. Come back empty. Update. Choose again. That loop is the whole method. | The loop as four words in a ring, the strip crossing C, C's dots going out, the map recounting: 60, 15, 10, 15 (`F-M.empty2.after.percent`). A code panel slides in from the right, tagged "for programmers": three lines, weights times the chance of an empty search, sum, divide | the dots; code panel | hold |

- **Show, then name:** the name arrives after the viewer has done the rule
  by hand (06) and again as a table (07_2).
- **Lower-third at 07_1**, precisely: "prior: the map before the search.
  Posterior: the map after it." Laplace is on screen and not spoken.
- **Misconception defused:** "now A has half the chance, so surely A". A
  half seen one time in four is less than a quarter seen two times in
  three.
- **Ruth:** Bayes solved a special case; the discrete rule with unequal
  priors is Laplace's (fact sheet, S14). The narration says only "after".
- **Dev:** the code panel is the whole algorithm; the Coast Guard's planner
  does the same to each simulated particle's weight (chapter 11).

## 08_where_next (54 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | ⟨predict⟩ Always search where the next search is most likely to find her. Is that really the best you can do? | The small sea on the left with the rule in gold above it. On the right a curve grows, one step a search: the chance she has been found by now, 20, 33, 43, 51, 58, 64 percent, under the searches B, C, A, A, B, D (`F-U.best.order`, `F-U.best.found_by.percent`) | a curve built step by step; axes | |
| 08_2 | Yes. For a wreck that stays put, and searches that cost the same, it is a theorem, proved in nineteen sixty-seven. No other order of searches is ever ahead. | A fan of faint curves fills in under the gold one, a counter running to 65,536: every order of eight searches (`F-U.brute.orders`); none crosses above it (`F-U.brute.rule_is_best`). Lower-third: "M. C. Chew, 1967: under this rule the chance of still searching after n searches is the least possible, for every n" (`F-N.chew`) | count-up; many curves as one mobject; lower-third | |
| 08_3 | On the Navy's map, the difference is large. Suppose every search sees a wreck in its cell two times in three. One plan sweeps the grid row by row. The other follows the rule. | The 1968 map twice, side by side, "model: every search sees 2 in 3" (`F-R.model.seeing`). Left: a strip going along row 1. Right: the strip jumping to E5, B7, C5, G5 (`F-R.model.best.first_cells`), each searched cell cooling | two grids recoloured in place; live counters of searches | |
| 08_4 | After twenty searches, the sweep has had less than one chance in a hundred of finding her. The rule, better than one in two. | Under each map a bar: 0.8% and 52.7% (`F-R.model.sweep.found_by.20`, `F-R.model.best.found_by.20`), growing live as the searches run to 20 | live bars | |
| 08_5 | The rule reaches the wreck's own cell on its thirty-fifth search. The sweep, on its eighty-first. | Both run on: the right-hand strip reaches F6 at search 35, the left-hand one at 81 (`F-R.model.best.F6_search`, `F-R.model.sweep.F6_search`). The white ring shows in F6 on both | live counters; the true place | hold |

- **Honest size of the claim** (fact sheet, finding 8): on the small sea the
  rule averages 6.34 searches and the plain sweep 8.00
  (`F-U.*.expected_searches`), and the two are level after 3, 5 and 6
  searches. The script says "is ever ahead", not "is always behind", and
  moves to the real map for the large difference.
- **Always sentence and its edge:** "No other order of searches is ever
  ahead" is `F-U.brute.rule_is_best` (every order of 1 to 8 searches, tried
  one by one) and Chew's Theorem 2.1; the theorem's conditions are spoken
  ("a wreck that stays put, and searches that cost the same").
- **Model, not history:** both plans over the 1968 map are the video's own
  model and say so on screen. The paper's own estimate of the same thing is
  quoted in a lower-third at 08_5: "the best plan would have reached that
  cell after 153 miles of track" (`F-S.miles_to_reach_F6`, p. 146).
- **Dev:** a greedy algorithm that is provably optimal; with unequal costs
  it ranks by chance of a find per unit cost, and with a hard budget and
  unequal costs it becomes a knapsack problem (description).

## 09_the_second_pass (75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Why would a search see a wreck only two times in three? The answer comes from the Second World War. | One flat cell of the small sea fills the stage, a single dot in it | morph | |
| 09_2 | Hunting submarines, a group of American scientists, Bernard Koopman among them, asked how much of an area a searcher really covers. | "B. O. Koopman, Search and Screening, 1946" (`F-N.koopman`) | lower-third | |
| 09_3 | With perfect lanes, side by side, enough track to pass over a cell once would cover all of it. But at sea your lanes land where they land. Some overlap, some leave gaps, and on average the same track covers sixty-three percent. | Left: lanes one percent wide fill a cell edge to edge, none overlapping; its readout runs to 100% (`F-K.perfect.1`). Right: the same hundred lanes fall at seeded random offsets, across and down (`F-K.lanes.seed`); gaps stay dark; its readout stops at this run's value (`F-K.lanes.one_run.1`, "this run"). Below, a plot: this run's curve, a faint bundle of other runs, their mean at 63% (`F-K.lanes.mean.1`, 1,000 runs) | live readouts recounted from the swept area; a curve; many curves as one mobject | |
| 09_4 | Koopman's formula gives that number, and the next ones. Twice the track, eighty-six percent. Three times, ninety-five. Each pass finds less than the one before. | The lanes keep falling to three times the track; the formula's curve is drawn over the mean: 63%, 86%, 95% marked on it (`F-K.formula.1`, `.2`, `.3`). Lower-third: "1 − e^(−WL/A): the random search formula. W strip width, L track length, A area" | formula; curve; lower-third | |
| 09_5 | ⟨predict⟩ So take a search that sees two times in three, and run it twice over the same cell. What is the chance that at least one pass sees her? | The quiz card: "2 in 3, twice = ?" | **the quiz card**; countdown ring | listen 5 s (countdown) |
| 09_6 | Eight in nine. The first pass leaves a third unseen, and the second finds two thirds of that. No number of passes makes it certain. | Nine dots in a cell: the first pass lights six, the second lights two of the three left; one stays dark (`F-M.seen_within.2`). Then 27 dots and a third pass: 26 of 27 (`F-M.seen_within.3`) | the dots; count-up | hold |

- **Earned formula** (plan rule 6): the formula is shown only as the curve
  through numbers the viewer has just watched accumulate; the letter e is on
  screen and never spoken.
- **Said equals shown:** "sixty-three" is the formula's 63.2 and the mean of
  the thousand runs (`F-K.lanes.mean.1`); the one run on screen shows its
  own number, which differs, and is labelled "this run".
- **Always sentence and its edge:** "No number of passes makes it certain"
  is 1 − (1 − d)ⁿ < 1 for every n when d < 1 (a test in `model.rs`).
- **Misconception defused:** "search an area twice and you have surely
  seen everything in it". Two passes of a two-in-three search leave one
  chance in nine.
- **Ruth:** Koopman's three assumptions are in the lower-third's second
  line ("the target equally likely anywhere in the area; the track laid in
  pieces placed independently; seen within the strip, never beyond"), and
  his own judgment of the formula: "a proper estimate, on the conservative
  side". The lanes on screen are that model exactly: each lane is a strip
  placed independently of the others. The formula is the limit of thin
  lanes; hundred-to-a-cell lanes average 63.4 (`F-K.lanes.exact.1`).
- **Dev:** one search of a flat cell is coverage 1.1 by this formula, and
  of the canyon 0.29 (`F-K.coverage.flat`, `.canyon`): the strip is about a
  quarter as wide among the rocks.

## 10_adrift (64 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | A wreck stays where it is. A person in the water does not. Wind and current carry them, and nobody knows either one exactly. | Open water. One white point; arrows for wind and current, each with a small fan of uncertainty | arrows | |
| 10_2 | So every dot moves with its own guess at the wind and the current. The map drifts, and it spreads. | The point becomes a cloud of 2,000 dots (`F-D.dots`, seed `F-D.seed`) that drifts across the stage and widens; a circle follows its edge; an hour clock runs 1, 3, 6, 12; "model" | the dots as a swarm, moved by placement; live clock; a circle that grows | |
| 10_3 | And a person is hard to see. From a helicopter, the Coast Guard counts on spotting someone in the water within a strip only a tenth of a mile wide. | A helicopter track across the cloud, its strip drawn to scale: a hairline. "sweep width 0.1 nautical mile: person in the water, helicopter at 300 feet (US Coast Guard)" (`F-D.strip`) | the search; lower-third: sweep width | |
| 10_4 | ⟨predict⟩ Let the cloud's radius double, and fly the same length of track. What happens to the coverage? | The quiz card: a circle and one twice as wide, the same zigzag track in each | **the quiz card**; countdown ring | listen 5 s (countdown) |
| 10_5 | It falls to a quarter. Twice the radius is four times the area. | Four of the small circles tile into the large one; "coverage × ¼" (`F-Q.radius_doubles.answer`) | morph | |
| 10_6 | In this model, a flight that would find the person ninety-three times in a hundred after one hour finds them thirteen times in a hundred after twelve. That is why hours matter. | The cloud at 1, 3, 6 and 12 hours in a row, the same track over each; under them 93%, 65%, 34%, 13% (`F-D.h1.seeing` to `F-D.h12.seeing`) and a curve through them | bars; a curve; "model" | hold |

- **The owner's "time is running out", made exact.** The cloud's radius
  grows with the distance drifted (the Coast Guard's long-standing rule of
  thumb puts the drift error at three tenths of it, `F-D.error_share`), so
  its area grows as the square, and the same track covers less of it.
- **Model numbers** (all marked "model"): a start known to within a mile, a
  drift of a mile and a half an hour, one flight of 180 miles of track
  (`F-D.*`). Only the strip width is the Coast Guard's.
- **Misconception defused:** "a later search just has farther to go". It
  has more water to cover with the same track.
- **Ruth:** nothing here claims the rule of chapter 08 is still the best
  plan for a moving target; the description says so and cites Brown (1980).
  "A tenth of a mile" is a tenth of a nautical mile, 185 metres
  (`F-D.strip.metres`), and is an effective width, not a hard edge.
- **Dev:** the dots are a particle filter: move each particle, then
  reweight.

## 11_twelve_hours (66 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | The United States Coast Guard has searched this way since nineteen seventy-four. Its software moves thousands of dots, each a possible drifting person or boat, and updates them by Bayes' rule after every empty search. | The drifting cloud from chapter 10 with a search rectangle laid over part of it; the dots inside dim; the rest brighten. "CASP, 1974. SAROPS, since 2007: 2,500 to 10,000 particles for each scenario" (`F-C.casp`, `F-C.sarops`, `F-C.sarops.dots`) | the dots recoloured in place; lower-third: particle | |
| 11_2 | In July of twenty thirteen, a lobsterman named John Aldridge fell off his boat at night, about forty miles off Long Island. Nobody saw him go. | A coast line at the top (Long Island, Montauk marked) and open water below; a boat's track heading south; a white point left behind on it. "24 July 2013" (`F-C.aldridge.date`) | a number line of hours down the side | |
| 11_3 | The searchers had to guess when he fell, and guessed the evening before. So the first map put its dots almost thirty miles too far north. | The hour line: "first guess: 9:30 to 11:30 p.m." in red (`F-C.aldridge.guess`). A cloud of dots near the coast, far from the white point (`F-C.aldridge.miles_south`) | the dots; the wrong estimate in red | |
| 11_4 | ⟨predict⟩ The map was wrong because its story was wrong. What could fix it? | The cloud and the distant white point, held | a question card | |
| 11_5 | Evidence. On deck, his partner found an open hatch and pumps running, work Aldridge would not have started until far offshore. He had fallen hours later. | Two small marks on the boat's track far to the south. The hour line: the red window is struck out; "about 3:30 a.m." in ink (`F-C.aldridge.fell`, `F-C.aldridge.hours_off`) | highlight | |
| 11_6 | The Coast Guard ran the map again, and the dots moved south. That afternoon a helicopter flying through them spotted him, alive, after almost twelve hours in the water. | The cloud is redrawn far to the south, around the white point. A track crosses it; the point turns green. "seen about 2:58 p.m.: 11 hours 28 minutes" (`F-C.aldridge.seen`, `F-C.aldridge.in_water`) | the dots; a find | hold |

- **A living person, found:** the half of the owner's question that the
  *Scorpion* could not answer.
- **Source and its limits** (fact sheet, finding 7): the account is the New
  York Times Magazine's, reported from the command centre; the Coast
  Guard's releases do not mention the software. The last track was drawn by
  hand through the high-chance area after the software froze, so the script
  says "a helicopter flying through them" and credits no program with the
  find. Aldridge had tied himself to lobster buoys and was not drifting
  freely; the description says so.
- **Misconception defused:** "the computer knows where he is". It knows
  what it was told; a wrong time of fall moved the whole map thirty miles.
- **Ruth:** the Coast Guard's bookkeeping leaves the map un-normalized on
  purpose; "updates them by Bayes' rule" is the developers' own phrase
  (S6). CASP, the 1974 system, "used a Bayesian approach" in the same
  authors' words.

## 12_the_silence (71 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | June, two thousand nine. Air France flight four four seven, from Rio to Paris, was lost over the Atlantic with two hundred and twenty-eight people aboard. | A flight track; the last reported position as a white cross. "1 June 2009. 228 aboard" (`F-A.aboard`, `F-A.lost`) | a card for the 228, held a moment | |
| 12_2 | The wreck had to lie inside a circle forty nautical miles in radius. Ships and sonar searched it for nearly two years, and did not find it. | The circle around the cross, "40 nautical miles (74 km)" (`F-A.circle.nm`, `F-A.circle.km`). Searched areas shade in, phase by phase, 2009 and 2010; a month counter runs to 22 (`F-A.months`) | lagged reveal; live counter | |
| 12_3 | Then a team of search analysts built a map from three ingredients, weighted thirty-five, thirty-five and thirty, and folded in every empty search. | Three small maps (an even disc; a mound at the centre; a patch from drifting bodies traced backwards) with weights 35, 35, 30 (`F-A.weights`) merge into one cloud of dots, "75,000 particles" (`F-A.points`). The searched areas dim their dots | the dots; morph; bars | |
| 12_4 | ⟨predict⟩ One search needed a decision. For a month, ships had listened for the beacons on the flight recorders, and heard nothing. How much should that silence count? | The listening tracks across the circle. "heard nothing" | a question card | |
| 12_5 | If the beacons worked, a great deal. If they were broken, not at all. So the analysts drew the map twice, once counting the silence, and once leaving it out. | Two maps side by side. Left: "the silence counts: 77 in 100 it would have been heard" (`F-A.beacon.used.percent`), dots dim along the listening tracks. Right: "the silence left out", those dots still lit | two dot clouds recoloured in place | |
| 12_6 | The search resumed near the centre of the circle and found the wreck within a week. It lay in a high-chance part of the second map, in water the listening ships had already crossed. | A white ring north-northeast of the cross, "about 6.5 nautical miles (12 km) from the last position" (`F-A.found.nm`, `F-A.found.km`), bright on the right-hand map and dim on the left; a listening track runs over it. "April 2011" | the true place; a find | hold |

- **Source:** Stone, Keller, Kratzke and Strumpfer (2014) and the BEA's
  final report. On screen at 12_3: "weights chosen by judgment
  ('subjective weights')". At 12_5: "no sensor was credited with more than
  9 in 10" (`F-A.beacon.dependent`, the paper's cap).
- **What is not said** (fact sheet, finding 6): that the wreck was in the
  top cell; that the analysts planned the search (the investigators did,
  starting at the centre); that the beacons are known to have failed. The
  picture shows two maps as the paper's Figures 7 and 8 do, not copies of
  them.
- **Misconception defused:** "they heard nothing there, so it is not
  there". The same mistake as "B is ruled out" in chapter 06, at full
  scale.
- **Ruth:** the authors say a joint distribution over the beacons' state
  and the wreck's place would have been the better way; the description
  quotes them.

## 13_how_well_they_looked (77 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | The same thing had happened in nineteen sixty-eight. The Navy's map reached the search ship in August, two months into the search. From then on, every pass of the sled was scored, cell by cell. | The 1968 map. A date line: 10 June, search begins; 12 August, the map arrives (`F-S.search_began`, `F-S.map_arrived`, `F-S.months.search_to_map`). The update as the paper printed it, under the map: L′ = L (1 − E) ÷ (1 − SEP), "Richardson and Stone, 1971, p. 146" | a grid; formula; a number line of dates | |
| 13_2 | By October, the ship's numbers said one cell had been searched ninety-four percent thoroughly. Of every hundred dots there, ninety-four went out. It was the cell with the wreck in it. | F6 enlarged beside the map with a hundred dots; 94 go out (`F-S.F6.searched.at_sea`, `F-S.F6.left.at_sea.percent`). The white ring appears among the six still lit | the dots; highlight | |
| 13_3 | ⟨predict⟩ The wreck was there, and the sled had been over that water. What went wrong? | F6, held, six dots lit | a question card | |
| 13_4 | The estimate of how well they were looking. At sea, the sonar was credited with a strip eight hundred feet wide. Measured afterwards against the wreck itself, it got zero, and the camera and the magnetometer about two hundred feet each. The honest figure for that cell was sixty-five percent. | Three strips to scale: "assumed at sea: sonar 800 ft, magnetometer 400 ft" in red; "measured afterwards: camera 185 ft, magnetometer 220 ft, sonar 0" (`F-S.sweep.at_sea`, `F-S.sweep.after`). In F6, dots relight until 35 are lit (`F-S.F6.searched.after`, `F-S.F6.left.after.percent`) | bars drawn to scale; the dots; the wrong estimate in red | |
| 13_5 | She was found on the twenty-eighth of October, when the sled went back to a scrap of metal photographed that summer, in a part of the map that had been likely from the start. | A small mark on the border of F5 and F6, "photographed in the summer". The ring in F6 turns green. "28 October 1968" (`F-S.found`). The paper's sentence in a lower-third: effort went to F6 "because of proximity to the piece of metal and the fact that this was a region with high a priori probability" | a find; lower-third | |
| 13_6 | An empty search only tells you something if it could have found what you were looking for. | The line alone, in ink | a card | hold |

- **The uncomfortable part, told** (fact sheet, findings 1 and 3): the map
  came two months late; the on-scene numbers had nearly written off the
  right cell; a piece of metal and the prior together brought the sled
  back. The numbers are the paper's Tables 1 to 3, for 9 October.
- **The opening picture, in its rigorous form** (plan rule 15): the update
  the viewer did with dots is on screen as printed in 1971.
- **Always sentence and its edge:** 13_6 is the update with a chance of
  seeing of zero: a search that could not have seen the target leaves every
  chance as it was (a test in `model.rs`).
- **Misconception defused:** "more searching always sharpens the map".
  Searching with an inflated estimate of how well you see sharpens it in
  the wrong place.
- **Ruth:** "ninety-four went out" is the factor 1 − E before the map is
  rescaled; the paper's E for F6 on 9 October was 0.94 at sea and 0.65
  afterwards. "The same thing" refers to an empty search trusted too far,
  not to a beacon.

## 14_four_maps (75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | Four searches, each against its map. | A table with four rows: Palomares 1966, Scorpion 1968, Air France 447, John Aldridge 2013; columns "the map" and "found" | a table built from primitives; lagged reveal | |
| 14_2 | Palomares, nineteen sixty-six. A hydrogen bomb, lost in the sea off Spain. Craven first weighed stories there. The bomb itself was found where a fisherman had watched it fall. | Row one fills: "weighted scenarios and search scores, first used"; "found by the submersible Alvin, 15 March 1966, where a fisherman saw it fall" (`F-P.found`, `F-P.days.to_found`) | table | |
| 14_3 | Scorpion. Beside the map's top cell, in a cell the numbers had nearly written off. | Row two, with a thumbnail of E5 and F6 | table; highlight | |
| 14_4 | Air France. In a high-chance part of the second map, the one that set the silence aside. | Row three, with a thumbnail of the two discs | table | |
| 14_5 | John Aldridge. Inside the dots, once the story was put right. | Row four, with a thumbnail of the cloud moved south | table | |
| 14_6 | ⟨predict⟩ Three of those maps had something wrong in them at first. A strip too wide. A silence trusted too far. A story five hours off. So why does the method work? | The three flaws light in red in their rows (`F-C.aldridge.hours_off`) | highlight; a question card | |
| 14_7 | Because it never needs the first map to be right. It needs you to say what you believe, to say honestly how well you looked, and to let every empty search move the dots. | The table folds away. The cold open's picture returns, now labelled: the dots, the strip, the update p′ = p (1 − d) ÷ (1 − p d), and the rule "search where p × d is greatest" | the dots; formula | |
| 14_8 | The ninety-nine men of the Scorpion could not be saved. The method that searched for them now searches for people who can. | The dots dim to a single ring and "99"; then the drifting cloud with a green point in it | the dots; a card | hold |
| 14_9 | *(receipts)* | "Every number in this video was computed by code, and checked against a second calculation." The repository link; "narrated by synthetic speech"; the sources' names | receipts card | listen 4 s (the motif, resolved) |

- **One clean summary** (the order): where each was found against its map.
  Palomares is told as where the method began, not as a find it made (fact
  sheet, finding 5).
- **The 99, with respect:** the last spoken line names them and claims
  nothing for them.
- **Names to search** (plan rule 26), on the receipts card: Bayesian search
  theory; Koopman, "Search and Screening"; Richardson and Stone 1971; Stone,
  "Theory of Optimal Search"; SAROPS.
- **Ruth:** "the method that searched for them" is the lineage the Coast
  Guard's own developers give (S6, S13): the Scorpion analysis, then CASP in
  1974, then SAROPS. "A story five hours off" is the first guess's middle,
  10:30 p.m., against 3:30 a.m.
