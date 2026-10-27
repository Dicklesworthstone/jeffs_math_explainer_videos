# Storyboard — How Much Can Five Serial Numbers Reveal?

The hand-edited source of this video's words and pictures.
`python3 -I tools/storyboard.py german_tank` checks it (gate G2) and writes
`script.tsv` from the words column. Every number below is a fact computed in
`src/facts.rs` and recomputed independently by `oracle.py`; fact group
`G-Fn` belongs to chapter `n`.

This file holds only the chapters whose native scenes exist in source:
chapters 01 to 13. Chapters 14 to 16 are written in full, in the same format,
in `docs/storyboard-draft.md`; a chapter moves here when its scene is
implemented, so `script.tsv`, the chapter registry and the Rust tests always
agree on what can render. The arc below describes the whole film.

Words are spoken exactly as written, one thought at a time: no colons,
semicolons, dashes or parentheses in them. `*(…)*` is an annotation, never
spoken. `⟨predict⟩` opens a prediction question; the beat ends with the
question and the picture holds (a visible four-second countdown, "pause for
more time") before the next beat answers it. Semantic pause intentions are in
`docs/pauses.tsv`.

## The arc

Owner direction 2026-10-24: intuition, motivation and setting before
machinery; no first-minute deadline. The film earns each idea from one
continuing example, five board-game boxes, whose objects keep their identity
and colors throughout.

**Act one, the question (01 to 03).** A company won't say how many copies of
its game it made, and five friends want to know whether their boxes are rare.
They turn the boxes over and read five numbers; the boxes move onto a line of
every possible number, each becoming its tick without losing its box. The
viewer finds the one certain thing, at least sixty, and meets the unknown
tail above it (01). Could the count be a thousand? A computed experiment
answers what it can: fair handfuls drawn from a thousand reach far up the
line, while handfuls from seventy crowd the stretch the friends' numbers
occupy, about twelve apart; the numbers can't prove the count, but their
spacing is a clue, and they can never say whether the handful was fair (02).
The gaps are then counted one at a time, and the top gap, the one nobody can
see, is guessed by the others: about seventy-one. The recipe is named with
its condition on screen: if the handful is fair, it is right on average (03).

**Act two, the proof (04 to 07).** The question and the assumption are made
exact and the recipe is written for any handful (04); a company of five boxes
shows every handful at once, and the ten estimates average exactly five (05);
the gaps can trade places, so the hidden top gap is the same size on average
as the seen ones (06); the same argument works for any count (07).

**Act three, what it can't promise (08 to 13).** Unbiased is a property of
the recipe: twenty repeat purchases scatter, all thirteen million handfuls
balance at the true count, lopsided, an estimate is rarely exact and misses
by about nine on average (08);
the range sixty to a hundred and seven and what ninety-five percent
confidence means (09); a rival recipe, doubling the average, unbiased and
sometimes absurd (10); the largest number carries everything a fair handful
says about the count (11); averaging the rival within groups that share a
largest number (12) and the proof that no unbiased recipe misses less, with
the honest footnote that a biased one can (13).

**Act four, breaking the assumptions (14 to 16).** The same five boxes from a
shop that only received the first seventy-one: the recipe counts the shop;
shelves of late boxes push it high, and the giant first gap gives it away
(14); an unknown first number, and equal chances per box that are still not
a fair handful (15). The name and its history in one line, the opening
picture once more, and a final transfer question: a sixth box (16).

| Chapter | Spine | Status |
|---|---|---|
| `01_five_boxes` | why the count matters; five numbered boxes; boxes become ticks; at least sixty; the unknown tail | words and pictures here; scene in `src/chapters/five_boxes.rs` |
| `02_a_thousand_boxes` | the fair handful defined by a bin; computed handfuls from a thousand and from seventy; spacing as the clue; fairness can't be seen | words and pictures here; scene in `src/chapters/a_thousand_boxes.rs` on the shared line of `src/chapters/friends_line.rs` |
| `03_counting_the_gaps` | the gaps counted one at a time; the top gap is one more gap; about seventy-one; the recipe and its condition; a promise about the recipe | words and pictures here; scene in `src/chapters/counting_the_gaps.rs` |
| `04_what_we_assume` | the count named and unknown; one handful, the one assumption; a recipe and its name; the gap recipe in the largest number alone; the edge handful 1 to 5; never below the largest | words and pictures here; scene in `src/chapters/what_we_assume.rs` on the shared line, with the halo, replacement and panel builders chapter 05 uses |
| `05_every_handful` | too many handfuls to look at; a company of five boxes, two seen; every pair and its exact estimate; the ten balance at five, the largest alone at four; what unbiased means, and that one company is not a proof | words and pictures here; scene in `src/chapters/every_handful.rs` |
| `06_gaps_trade_places` | the tiny table's unseen squares become three gaps; pairs and gap lists match exactly, each one in ten; a list read backwards; trading the gaps below and above gives another row; the three columns hold the same numbers and each averages one box; back to the friends, whose sixth gap is the hidden one | words and pictures here; scene in `src/chapters/gaps_trade_places.rs`, with the friends' line and its gaps that chapter 07 opens on |
| `07_right_on_average` | the same argument with the count a letter; each gap averages N minus five, over six; the guess, eleven, is one handful's average, and in the tiny company guess and top gap rarely agree but both average one; so the estimates average the count; one box doubles and subtracts one; any handful size | words and pictures here; scene in `src/chapters/right_on_average.rs` |
| `08_one_estimate` | the promise is about the recipe; a company of exactly seventy-one; twenty computed handfuls and their estimates; every handful counted by its largest number, sixty-seven stacks; how often exactly right, about one in twenty-nine; the balance and the lopsided sides; the average miss and the standard deviation; three ideas kept apart | words and pictures here; scene in `src/chapters/one_estimate.rs` |
| `09_how_far_off` | the count fixed and unknown; for each count, the chance a fair handful is all sixty or less; just over and just under five percent at a hundred and seven and eight; the range sixty to a hundred and seven; the twenty handfuls build their own ranges, nineteen cover seventy-one; what ninety-five percent confidence means and doesn't; the division behind each bar; ten boxes, sixty to seventy-nine | words and pictures here; scene in `src/chapters/how_far_off.rs` on chapter 08's draws |
| `10_a_rival_recipe` | another recipe from the middle number, twice the average minus one, unbiased too; seventy-three for the friends; a strange handful with sixty gives twenty-eight point six, below a box someone holds; about one handful in five falls below its own largest; every handful of the seventy-one-box company, counted by total; both balance at seventy-one, doubling wider by the average miss and by the standard deviation | words and pictures here; scene in `src/chapters/a_rival_recipe.rs` on chapter 08's bars |
| `11_the_largest_says_it_all` | told only the largest, sixty; pairs topped by five in companies of five, six and eight, the same four, each one in four; for sixty, every group of four equally likely at every count while the chance of sixty changes; the other four tell nothing, the largest is sufficient; the gaps below sixty always hold fifty-five; all of it needs a fair handful | words and pictures here; scene in `src/chapters/the_largest_says_it_all.rs` |
| `12_averaging_the_wobble` | the tiny company's ten pairs with the doubling recipe, unbiased but wobbling; sorted by their larger number, each pair kept whole; predict the biggest group's average; every group's average is the gap recipe's estimate, the same in any company; weighted by their chances the groups give five, equally weighted four and a quarter; misses as arrows, cancelling in one group and one-sided in another; average miss 1.4 against 1.2, squared 3 against 2.25; averaging never makes misses bigger | words and pictures here; scene in `src/chapters/averaging_the_wobble.rs` |
| `13_nothing_does_better` | unbiased must hold at every count, not just the true one; for pairs, each company forces the next value of a recipe of the larger number, two, three and a half, five, six and a half; the same step at any count and handful size, never without a new value; so only the gap recipe; any unbiased recipe, even three times the smaller minus one, averages into it; best unbiased, ties allowed; a slightly biased recipe misses less at seventy-one but not at five boxes | words and pictures here; scene in `src/chapters/nothing_does_better.rs` |
| `14_an_unfair_draw` | shop and company sampling pools; biased draws | implemented in `an_unfair_draw.rs` |
| `15_other_assumptions` | two hidden endpoint gaps and dependent deliveries | implemented in `other_assumptions.rs` |
| `16_five_boxes_again` | original-five callback, uncertainty and shop conditions, sixth-box prediction | implemented in `five_boxes_again.rs` |

## Vocabulary (one word per idea)

**box** (one copy of the game; never "unit", "item" or "tank" before chapter
16); **number** (the serial number printed on a box; "serial number" only in
the title and chapter 16); **count** (how many boxes were made, N; never
"population" or "total" in the voice); **the line** (every number the
company could have printed, drawn as the strip); **handful** (the boxes that
were seen, five unless said otherwise; never "sample" in the voice); **fair
handful** (five pulled blindly from a shaken bin of every box made: any five
exactly as likely as any other five; never "random sample"); **spacing** (how
far apart neighbouring numbers sit, counting from zero; chapter 02 only);
**largest number** (the maximum; never "max"); **gap** (the unseen numbers
below the smallest seen one, between two seen ones, or above the largest);
**top gap** (the gap above the largest, never seen); **recipe** (a rule that
turns any handful into an estimate; "estimator" is said once, in 04, as its
name); **estimate** (what a recipe says for one handful); **guess** (the
gap recipe's stand-in for the top gap, the average of the gaps it can see;
07); **gap recipe** (the
largest number plus the average gap); **doubling recipe** (twice the
average, minus one, from 10); **unbiased** (the estimates average to the true
count, whatever that count is); **average miss** (how far an estimate lands
from the count ignoring direction, averaged over handfuls; never the standard deviation); **range** (the counts for which a
handful like the friends' is not rare, from 09); **standard deviation** (named
in 08 as the second width, the square root of the average squared miss, and said again in 10 to compare the recipes); **sufficient** (said once, in 11, as
the name for "the largest number says it all"); **group** (from 12, the pairs
or handfuls that share a largest number, with their chance as weight; 11's
"groups of four" are the other four numbers' sets, said only there);
**squared miss** (an estimate's miss squared, averaged over handfuls, from
12 always beside the average miss and never instead of it); **staircase**
(13, the values the only unbiased recipe of the largest number is forced to
take).

## Color roles

| Idea | Color |
|---|---|
| a number someone saw, its box and its tick | gold `#F0AC5F` |
| a handful drawn by computer from a company whose count we chose (02, 08, 09) | gold outlines at half strength, never filled |
| an unseen number in a gap below the largest, the gap's count, a spacing arrow | blue `#58C4DD` |
| the top gap and its "?" | `hero` `#FFD866` (the "?", the dashed bracket and the countdown; in the tiny table of 06 and 07 also the top gap's outlined squares, its column head, its sizes and their average) |
| an estimate the gap recipe gives | green `#83C167` |
| what is certain (the lower bound, "at least 60") | teal `#5CD0B3`, text and strokes only |
| the condition "if the handful is fair" | `ink` text in a `dim` outline, the same badge every time it appears |
| the doubling recipe and its estimates (from 10), and its misses as arrows (12) | purple |
| a group's average, the gap recipe's estimate, its misses as arrows and the staircase's bars (12, 13) | green |
| a rival recipe that is neither (13: three times the smaller minus one, seven sixths of the largest minus one) | `ink` |
| an unfair draw's pool and its estimate (14, 15) | red |
| numbers not under discussion, the line and the boxes' outlines | `faint` and `dim` |
| a company's count fixed for an experiment (08, 09), a range of counts and a computed range that covers it (09) | `ink` strokes |
| how many handfuls (08) or how likely a handful this low (09), as bars | green bars for estimates (08); `dim` bars for chances (09), `faint` once rare |

## 01_five_boxes: Five boxes (~75 s)

Centre: one game box, a rounded rectangle with a lid band, drawn the same way
every time a box appears. Top band: the five friends' boxes. Middle: the
line, one faint dot for every number from 1 to 84, fading out on the right
(nobody knows where it ends). Right: the panel and the countdown. Every
number is `G-F1`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | A small company makes a board game, and it won't say how many copies it has made. | One game box (a rounded body with a lid band) left of centre, alone; a dim "how many?" beside it | single object, slow reveal | |
| 01_2 | Five friends who own it would like to know. If the company made a few dozen, their boxes are rare. If it made tens of thousands, they're ordinary. | The box rises into the middle of a top band and four more fade in beside it, five boxes in a row, none showing a number; "how many?" moves to the panel. Under the band a dim scale, "a few dozen" over "rare" at its left end and "tens of thousands" over "ordinary" at its right; a hero "?" marker moves to each end as it is named and drifts back, settling nowhere | lagged reveal, stakes | |
| 01_3 | Every box has a number on the bottom, counting up from one. The first box made is number one, the next is number two, and nothing is skipped. | The scale fades. On "counting up from one" the line draws from the left, one faint dot per number from 1 to 84, fading out over the strip's last quarter; "1" under the first dot and "2" over the second (staggered: they are 0.15 apart); on "nothing is skipped" a ripple runs along the first twelve dots | number line, lagged dots | |
| 01_4 | The friends turn their boxes over. Fourteen, twenty-seven, thirty-three, fifty-one and sixty. | Each box turns over (a squash and unsquash; its lid band goes) as its number is spoken and shows its number in gold; the picture holds so the five can be read | lagged reveal, inspection hold | hold |
| 01_5 | Put each box where its number belongs, on the line of every number the company could have printed. | One at a time, from the right (60 first), each box shrinks and slides down onto its own dot, keeping its gold number; the five dots turn gold and a gold tick grows from each dot to the bottom centre of its box. From here everything on the line is bound to three trackers through one map (the line's last number shown, the boxes' spread, the ticks' growth). On "every number the company could have printed" a ripple runs along the whole line | identity-preserving move | |
| 01_6 | ⟨predict⟩ Before any clever ideas, what can you say for certain about how many boxes the company made? | Nothing new appears; the countdown runs in the panel | prediction pause, think time | |
| 01_7 | It made at least sixty. Box sixty exists, so boxes one to sixty all do too. | A teal bracket under the line from 1 to 60; the panel writes "at least 60" in teal under "how many?"; on "Box sixty exists" box 60 flashes; on "boxes one to sixty" the dots under the bracket brighten one step | bracket, highlight | |
| 01_8 | Above sixty, nobody knows. The line might stop right after sixty, or run on for hundreds more. | The labels 1 and 2 go. A dashed hero bracket starts at 61 and runs off the faded end, a hero "?" under it near the right. On "or run on" the five boxes spread into a row from the line's left end, each tick still joining its dot to its box's bottom centre; then, on "hundreds more", the line zooms out to 1 … 1,000 under them: the 84 dots squeeze to the left, a thin faint line fills the rest of the strip, the dashed bracket grows to cover almost all of it and the teal bracket shrinks with its numbers. The boxes are apart in every state of both moves, not only at their ends | number line, zoom out | |
| 01_9 | Nothing these five numbers say can rule out a thousand boxes. But would five boxes from a thousand look like these? | Marks 1, 250, 500 and 750 under the zoomed line and "1,000?" in dim over its right end; on "look like these" the five boxes and their ticks flash once | highlight | |

### Notes for 01

- `G-F1.numbers` 14, 27, 33, 51, 60; `G-F1.first` 1; `G-F1.handful` 5;
  `G-F1.largest` 60 and `G-F1.at_least` 60 (the smallest count every number
  in the handful fits under); `G-F1.thousand` 1000.
- The line's labels come from the same facts as its dots: "1" is
  `G-F1.first` and "2" is `G-F1.second` (01_3), and the zoomed line's marks
  are `G-F1.zoom_marks` 1, 250, 500, 750 (01_9): the first number, then the
  quarters of the zoom's extent `G-F1.thousand`, whose own label is
  "1,000?". The scene refuses to build if a label or mark drifts from the
  numbers its line is drawn with.
- The numbers 14, 27, 33, 51, 60 were chosen so the five gaps (13, 12, 5,
  17, 8) differ visibly but average a whole number; nothing in the video
  claims they were drawn at random.
- The stakes (01_2) are the owner's "why does this matter": scarcity. "A few
  dozen" and "tens of thousands" are deliberately far from the answer so the
  scale leaks nothing; the hero marker never settles.
- Prediction (01_6): the tempting answers are "nothing" and "exactly sixty".
  The answer is the lower bound and only that; 01_8 makes the unknown tail
  explicit before any estimate exists.
- 01_9 ends on a question that is not a prediction beat: chapter 02 answers it
  with an experiment.
- No war, tank or history appears before chapter 16.
- Pause intentions (`docs/pauses.tsv`): after the stakes, after the numbers
  are read (the inspection hold), the prediction countdown after 01_6, a
  reveal pause after "It made at least sixty.", a thought pause after
  "Above sixty, nobody knows."

## 02_a_thousand_boxes: If there were a thousand (~85 s)

Chapter 01's line again, 1 to 84 with the five boxes on their dots and
ticks. It can zoom out to 1 to 1,000 and back, the boxes spreading into a
row before the line squeezes and coming home after it stretches. The bin, a
large rounded rectangle, in the middle. Rows of drawn handfuls stack above
the line, one row per handful, each number a gold ring at half strength; the
friends' boxes stay full gold. Every number is `G-F2`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | To find out, we need a picture of how the five were chosen. Imagine every box the company made in a huge bin, shaken well. | On "Imagine every box" the boxes and their ticks fade and the bin appears; every dot of the line, all at one strength now, pours into it and settles in a heap, each dot still a box; on "shaken well" the bin shakes | morph, identity-preserving move | |
| 02_2 | Five boxes come out blindly, and any five are exactly as likely as any other five. Call that a fair handful. | On "come out blindly" five dots rise out of the heap as gold rings (`G-F2.bin_handful`); on "as likely as any other five" they flash; on "Call that a fair handful" the panel writes "fair handful" and a "precisely" lower-third rises (card B) | lagged reveal, lower-third | |
| 02_3 | Real friends don't shop from a bin, so this is an assumption. Later we'll break it on purpose. | On "this is an assumption" the panel's words give way to the badge "if the handful is fair" in the panel's corner, where it stays; on "Later we'll break it" the five sink back and every dot pours back onto the line with its old fade; the rings and the bin go and the boxes and ticks return; card B stays until it can be read | badge | |
| 02_4 | ⟨predict⟩ If the company had made a thousand boxes, where would a fair handful's five numbers usually land? | The boxes spread into their row; on "a thousand boxes" the line's end slides to 1,000 under them, the 84 dots squeezing to the left; marks 1, 250, 500, 750 and 1,000 under the line; the countdown | prediction pause, think time, number line | |
| 02_5 | All over the line. Here are ten fair handfuls from a thousand, drawn by computer. Each of them reaches past six hundred. | The countdown goes; on "Here are ten" ten rows drop in one after another from the computed draws (`G-F2.big_handfuls`), each row's largest flashing as it lands; on "past six hundred" a dim upright mark at 600 rises through every row and all ten largest numbers flash | computed experiment, lagged reveal | |
| 02_6 | Five numbers that all stay at sixty or less, like the friends', turn up in fewer than one handful in a million. | On "sixty or less" a teal outline, strokes only, surrounds the stretch 1 to 60 across every row: no row fits inside it; on "fewer than one handful" the panel: "all 5 at most 60" over "from 1,000: fewer than one in a million" | highlight | |
| 02_7 | Now try a company of seventy. Its handfuls look much more like the friends'. Nearly half of its possible handfuls stay at sixty or below. Three of these ten do. | On "Now try a company" the rows, the 600 mark, the outline and the marks fade; the line comes back in to 1 … 84, the boxes coming home after it, and an upright end mark at 70 appears; on "Its handfuls look" ten rows from seventy drop in (`G-F2.small_handfuls`); on "Nearly half" the teal outline 1 to 60 returns and the panel adds "from 70: 45% of all handfuls"; on "Three of these ten" the three rows inside the outline flash | computed experiment, zoom in, highlight | |
| 02_8 | Look at the spacing. Numbers from a thousand sit over a hundred and fifty apart, on average. From seventy, about twelve apart, just like the friends' numbers. | The rows, the outline, the 70 mark and the panel's lines go. On "Numbers from a thousand" the line zooms out again, the panel heads "average spacing" and a blue readout "arrows' length ÷ 5" appears at the lower right; one row from a thousand drops in and blue spacing arrows run from zero to each of its numbers while the live readout counts their length over five, ending at 190.6; the panel adds "all handfuls from 1,000: 166.8". On "From seventy" that row goes, the line comes back in and a row from seventy runs the same way to 11.0; the panel adds "all handfuls from 70: 11.8". On "just like the friends'" arrows run under the friends' own five (14, 13, 6, 18, 9), the readout ending at 12.0, and the panel adds "the friends' handful: 12" | live readout, coordinated views, number line | |
| 02_9 | So the numbers can't prove the count, but their spacing is a clue. What they can never tell you is whether the handful was fair. | The drawn row, its arrows and the readout go; on "their spacing is a clue" the friends' arrows flash; on "whether the handful was fair" the badge flashes | highlight, badge | |

### Notes for 02

- The draws: SplitMix64 with seed `G-F2.seed` 20261024 plus one stream number
  per experiment (1 for a thousand, 2 for seventy, and one for the bin's
  handful), each box drawn uniformly by rejection, a repeat redrawn, the
  first ten handfuls shown in the order drawn and never chosen. `src/model.rs`
  and `oracle.py` implement the generator separately and must print the same
  rows. They are illustrations; every spoken claim is an exact fact:
  `G-F2.big_largest_over_600` true ("past six hundred" is `G-F2.over` 600,
  under `G-F2.big_lowest_largest` 750), `G-F2.big_all_at_most_60`
  227563/343762135425 ≈ 6.6 × 10⁻⁷ (`…_under_one_in_a_million` true),
  `G-F2.small_all_at_most_60.percent` 45 ("nearly half").
- "Three of these ten do" is `G-F2.small_fit_shown` 3: one draw's count, said
  after the exact share so the two are not confused. The scene refuses rows
  where none or all fit, or a count that differs from the words.
- Spacing: the average distance from each number to the one before it,
  counting from zero, is (N + 1)/6 for a fair handful of five: 1001/6 ≈ 166.8
  (`G-F2.big_spacing`, "over a hundred and fifty", `G-F2.spacing_over` 150)
  and 71/6 ≈ 11.8 (`G-F2.small_spacing`, "about twelve"); the friends' is
  60/5 = 12 (`G-F2.friends_spacing`). "Just like" compares those two rounded
  values, nothing more. The live readout is one row's arrows over five and
  ends on that row's own average (`G-F2.big_row_spacing` 190.6,
  `G-F2.small_row_spacing` 11.0); the panel's exact averages are labelled
  "all handfuls from" so a row's readout is never read as the average.
- Card B (02_2): "precisely: the count N is fixed and unknown; the handful is
  random: every set of 5 numbers from 1 to N is equally likely (a uniform
  draw without replacement). Equal chances for each box are not enough."
  Chapter 15 shows why.
- The bin is the owner's "concrete before notation": the fair handful is
  defined by a physical action before any formula, and its badge stays in the
  panel through chapter 03.
- "Real friends don't shop from a bin" states the assumption's status; it is
  not a claim about any real market. Chapter 14 breaks it with these same
  five boxes.
- The stretch 1 to 60 is teal, the certain role, so it is an outline and never
  a fill.
- Pauses: a hold after the bin has settled (02_1), the countdown after 02_4, a
  reveal pause after "All over the line.", a thought pause after "Look at the
  spacing."

## 03_counting_the_gaps: Counting what nobody saw (~67 s)

Back to the line from 1 to 84, the friends' boxes on their ticks, the badge
in the panel's corner. Gaps are counted dot by dot, each with its own live
count under the line. Every number is `G-F3`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | So let's use that clue. Look at the numbers nobody has seen. Below fourteen, there are thirteen. | On "Look at the numbers" the dots 1 to 13 light blue one at a time while a live count under them climbs 1, 2, … 13; on "there are thirteen" a blue arc closes over them and the count stays as its label | live count, one gap at a time | |
| 03_2 | Between fourteen and twenty-seven, twelve more. | On "Between fourteen" the dots 15 to 26 count up the same way, a little faster, to 12; on "twelve more" their arc closes | live count | |
| 03_3 | Then five, then seventeen, then eight. | On each number the next gap counts up, a little faster each, and closes: 5, 17, 8 | live count, lagged reveal | |
| 03_4 | These stretches are the gaps. Five gaps hold fifty-five unseen boxes. That's eleven per gap, on average. | On "These stretches are the gaps" the five arcs flash in turn; on "Five gaps hold" the five counts slide up into the panel and the signs appear between them, 13 + 12 + 5 + 17 + 8 = 55 with the 55 in blue; on "That's eleven per gap" the line 55 ÷ 5 = 11 under it, 55 and 11 in blue | equation, identity-preserving move | |
| 03_5 | There's one more gap, above sixty. It's the top gap, and nobody can see it. | On "one more gap" a dashed hero bracket draws in, dash by dash, from 61 off the faded end; on "It's the top gap" a hero "?" under it; on "nobody can see it" the "?" flashes | bracket, highlight | |
| 03_6 | In a fair handful, nothing makes the top gap special. It's just one more space between numbers, and it happens to be out of sight. | On "In a fair handful" the badge flashes. On "It's just one more space" an inset comes in at the upper left: "a fair handful from 70", chapter 02's first row from seventy (`G-F3.known_handful`) on its own short line ending at 70, every gap with its blue arc and count, the top gap's too (`G-F3.known_gaps`, `G-F3.known_top_gap`), and the top gap's hero dashed bracket; its six arcs flash in turn: because that count is known, its top gap can be seen. On "out of sight" the friends' dashed bracket and its "?" flash | inset, callback, lagged reveal | |
| 03_7 | ⟨predict⟩ If the top gap is about as big as the others, how many boxes did the company make? | The inset fades; the dashed bracket keeps its full, unknown length; the countdown | prediction pause, think time | |
| 03_8 | About seventy-one. Sixty, plus one average gap of eleven. | The countdown and the "?" give way to a green tick at 71 with "≈ 71" over it; on "Sixty, plus" box 60 flashes; on "plus one average gap" the dashed bracket's right end draws back to 71, then the dashes become a green bracket from 61 to 71 with a green 11 under it; on "of eleven" the panel writes 60 + 11 = 71, the 60 gold, the 11 blue and the 71 green | equation, highlight, tracker | |
| 03_9 | That's a recipe. Take the largest number and add the average gap. If the handful is fair, this recipe is right on average. | On "That's a recipe" the sums fade, the panel's top line becomes "estimate = largest + average gap" (estimate green, largest gold, average gap blue) and 60 + 11 = 71 moves up under it; on "Take the largest number" box 60 flashes; on "add the average gap" the green bracket flashes; on "If the handful is fair" "right on average" appears at the panel's foot and the corner badge slides beside it; on "right on average" both flash | equation card, badge, highlight | |
| 03_10 | Right on average is a promise about the recipe, not about seventy-one. We'll prove it, and then find out what it can't promise. | A "precisely" lower-third rises (card A); on "a promise about the recipe" the recipe flashes; on "not about seventy-one" 60 + 11 = 71, the green tick, "≈ 71" and the green bracket dim a step while the recipe, its claim and the badge stay bright; card A stays until it can be read | lower-third, card | |

### Notes for 03

- `G-F3.gaps` 13, 12, 5, 17, 8; `G-F3.unseen` 55; `G-F3.gap_count` 5;
  `G-F3.average_gap` 11; `G-F3.estimate` 71. The voice reads them one at a
  time: `G-F3.first_seen` 14 and `G-F3.first_gap` 13, `G-F3.second_seen` 27
  and `G-F3.second_gap` 12, then `G-F3.last_gaps` 5, 17, 8. "One more gap"
  (03_5, 03_6, 03_8) is `G-F3.top_gaps` 1, counted on the known handful,
  where the top gap can be seen.
- A gap's count and its lit dots round the same tracker the way the kit's
  live count does, so the number shown and the dots lit agree on every frame.
- The inset's row is the first drawn from seventy, (30, 33, 50, 53, 55):
  gaps 29, 2, 16, 2, 1 and top gap 15. It was not chosen; its top gap is
  simply visible because we fixed the count.
- 03_6 is the intuition chapters 05 to 07 prove ("the gaps can trade
  places"); its claim is about a fair handful only, and the badge is on
  screen when it is made.
- Card A (03_10): "precisely: boxes 1 to N with no skips; a fair handful of 5.
  Right on average: over all handfuls, the estimates average exactly N, for
  every N of at least 5. 71 is one estimate, not a count."
- Prediction (03_7): the tempting answers are "sixty" (the largest) and "about
  double the average, seventy-three" (which returns in chapter 10 as the
  rival recipe). The countdown hides the bracket's length and every estimate.
- Pauses: the counting holds (03_1 to 03_3 are visual beats with short
  words); a thought pause after "Five gaps hold fifty-five unseen boxes."; the
  countdown after 03_7; a reveal pause after "About seventy-one."

## 04_what_we_assume: Saying it exactly (~85 s)

The friends' line from 1 to 84 as chapter 03 drew it, the five boxes on their
gold ticks, no arcs and no estimate yet; the badge in the panel's corner.
Highlights are halos: a separate rounded outline comes and goes around what
the words point at, and the thing itself is untouched. No typeset text is
indicated or recoloured in place; a panel line that changes is replaced, the
old line fading as the new one comes in. Every number is `G-F1`, `G-F3` or
`G-F4`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Let's pin the question down. Call the true number of boxes the count. We don't know it. It might be sixty, seventy-one or five hundred. | On "Call the true number" the panel's top line reads "count = ?"; on "sixty", "seventy-one" and "five hundred" each value comes in, dim, in a row after it: 60, 71, 500 (`G-F1.largest`, `G-F3.estimate`, `G-F4.far_count`) | panel, reveal on the word | |
| 04_2 | The friends' five boxes are a single handful. We assume the bin from before, so every handful of five was equally likely to be theirs. | The three values fade; on "a single handful" a gold halo comes and goes round the five boxes; on "We assume the bin from before" a halo round the badge; on "every handful of five" six rows of gold rings land over the line one row at a time, chapter 02's handful from the bin first and five more from the same computed draw (`G-F4.ghost_handfuls`), every row drawn alike | halo, lagged reveal, callback | |
| 04_3 | A recipe is any rule that turns a handful into an estimate of the count. Statisticians call it an estimator. | The rows fade; on "turns a handful into an estimate" a picture builds left to right in the panel: "handful" in gold, an arrow, "recipe" in a dim frame, an arrow, "estimate" in green; on "Statisticians call it an estimator" "estimator" in dim under the frame | diagram, lagged reveal | |
| 04_4 | Ours, the gap recipe, adds the average gap to the largest number. Of the sixty boxes from one to sixty, five are the friends', so the gaps hold sixty minus five, fifty-five. | On "Ours, the gap recipe" the picture is replaced by "estimate = largest + average gap" (estimate green, largest gold, average gap blue); on "Of the sixty boxes" the teal certain bracket from 1 to 60 draws under the line; on "five are the friends'" a gold halo round the five boxes; on "so the gaps hold" the five blue gap arcs draw in turn and the panel writes "gaps: 60 − 5 = 55", gaps and 55 blue, 60 gold | equation, halo, lagged reveal | |
| 04_5 | With any five numbers, the gaps below the largest hold the largest minus five. So the estimate is the largest, plus the largest minus five, over five. For our friends, sixty plus fifty-five over five, seventy-one. | On "With any five numbers" that line is replaced by "gaps: largest − 5 = 60 − 5 = 55" and the five arcs ripple, their strokes only; on "So the estimate is" the recipe is replaced by "estimate = largest + (largest − 5) ÷ 5"; on "For our friends" "60 + 55 ÷ 5 = 71" builds piece by piece under them, 60 gold, 55 blue, 71 green; on "seventy-one" a green tick at 71 on the line with "71" over it; the panel holds to be read | formula, equation, identity-preserving replacement | hold |
| 04_6 | ⟨predict⟩ What would the recipe say if the five numbers were one, two, three, four and five? | On "one, two, three, four and five" the arcs, the certain bracket and the green tick fade; gold rings land on the dots 1 to 5 with "1, 2, 3, 4, 5" under them, the friends' boxes staying where they are; the countdown | prediction pause, think time | |
| 04_7 | Five. There are no gaps, so there's nothing to add. And since gaps are never negative, the recipe never guesses below the largest number it saw. | The countdown and the friends' two lines give way to the edge handful's: "gaps: largest − 5 = 5 − 5 = 0" (the 0 blue) and "5 + 0 ÷ 5 = 5" (the last 5 green), with a green tick at 5 on the line and "5" over it (`G-F4.edge_gaps`, `G-F4.edge_estimate`); on "There are no gaps" a blue halo round the sum's 0; on "never guesses below" "never below the largest" in teal under the line | equation, halo | |
| 04_8 | A single handful can't tell whether a recipe is good. We'll judge a recipe across every handful the company could have sold. | On "A single handful" a gold halo round the five rings; on "can't tell whether" a halo round "count = ?"; on "We'll judge a recipe" the panel's lines under "count = ?", the rings, their label, the tick and the teal words fade; on "every handful the company" the six rows of rings come back, nearly all at once | halo, lagged reveal, callback | |

### Notes for 04

- 04_1 names three counts the handful allows, each at least the largest:
  `G-F1.largest` 60, `G-F3.estimate` 71 and the round `G-F4.far_count` 500.
  None of them is claimed.
- The six rows are `G-F4.ghost_handfuls`, the first six handfuls of the
  computed stream behind chapter 02's bin, the first being
  `G-F2.bin_handful`. They were not chosen. Two of their numbers are a box
  apart (43 and 44), and a test keeps every ring clear of its neighbours.
- "With any five numbers, the gaps below the largest hold the largest minus
  five": the oracle checks it for every handful, k ≤ 5, N ≤ 14. "Never
  guesses below the largest" is checked for every handful, k ≤ 6, N ≤ 12,
  with equality exactly at 1 to k (oracle cross-checks). The scene's own test
  repeats it, with the doubling recipe on `G-F10.silly_handful` as a negative
  control that does go below.
- `G-F4.edge_handful` 1, 2, 3, 4, 5 has `G-F4.edge_gaps` all 0 and
  `G-F4.edge_estimate` 5. The panel shows the empty total without colouring
  either 5 of "5 − 5", because the two 5s play different roles.
- Chapter 02 defined the fair handful physically (card B). 04_2 restates it
  as the one assumption, with a halo on the badge.
- Prediction (04_6): a viewer may expect the recipe always to add something.
  With no gaps it adds nothing. The countdown hides every line of the answer.
- Pauses: a beat after "We don't know it." and after "a single handful."; a
  clause pause after "the largest, plus the largest minus five," so that
  "over five" groups correctly; a reveal pause after "Five.". The inspection
  hold after 04_5 is a scene hold.

## 05_every_handful: Every possible handful (~77 s)

A clear screen. The 71-box company's handfuls are counted, not shown. Then a
company of five boxes is drawn like the friends' boxes, two of them seen,
with its table of every pair. Every estimate is the exact fraction the model
computes, shown exactly (3.5, never rounded). Highlights are halos and
colour changes are replacements, as in chapter 04. Every number is `G-F5`,
or `G-F1.handful` for five.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | A company of seventy-one boxes has over thirteen million handfuls of five, far too many to look at. So let's shrink the company. Suppose it made just five boxes, and you see two. | "handfuls of 5 from 71" in dim with a big live count under it at 0; on "over thirteen million" the count runs up to 13,019,909 (`G-F5.handfuls`); on "So let's shrink the company" both fade; on "just five boxes" five small boxes numbered 1 to 5, drawn like the friends' boxes, land at the upper right; on "you see two" the labels 2 and 5 are replaced by gold ones | live count, lagged reveal, replacement | |
| 05_2 | There are ten possible pairs, from one and two up to four and five. In a fair draw, each pair is equally likely. | On "ten possible pairs" the column numbers 1 to 5 and then the table's ten rows land in turn, each row five small squares with the pair's two squares gold, in the order 1 2, 1 3, …, 4 5 (`G-F5.pair_list`); on "from one and two" a gold halo round the first row; on "up to four and five" round the last; on "each pair is equally likely" "each pair: 1 in 10" in dim under the table; the table holds to be read | table, lagged reveal, halo | hold |
| 05_3 | Take the pair two and five. Below five, the unseen boxes are one, three and four. That's three boxes, five minus two, in two gaps. So with a pair, the recipe adds half of the largest minus two. | On "Take the pair two and five" a gold halo round its row; on "the unseen boxes are" the small boxes' labels 1, 3 and 4 are replaced by blue ones; on "That's three boxes" blue arcs over the two gaps with their counts, 1 and 2 (`G-F5.example_high_gaps`), and "gaps: 5 − 2 = 3" at the right, gaps and 3 blue, 5 gold; on "So with a pair" "estimate = largest + (largest − 2) ÷ 2" under it | equation, halo, replacement | |
| 05_4 | For two and five, that's five plus three halves, six and a half. For one and two, there are no gaps, so the estimate is just two. | On "five plus three halves" "5 + 3 ÷ 2 = 6.5" builds piece by piece under the formula, 5 gold, 3 blue, 6.5 green; on "six and a half" the table's "estimate" column head and a green 6.5 in the pair's row; on "For one and two" a gold halo round the first row; on "just two" a green 2 in it | equation, table, halo | |
| 05_5 | Here are all ten estimates. Some are too low and some too high. ⟨predict⟩ What do the ten average? | The worked example fades: the small boxes, their labels, the arcs and the lines at the right; on "all ten estimates" the other eight estimates cascade down their rows, so the column reads 2, 3.5, 5, 6.5, 3.5, 5, 6.5, 5, 6.5, 6.5 (`G-F5.estimates_shown`); on "Some are too low" a dim vertical line labelled "count = 5" runs through a small plot beside the rows, and each row's estimate lands on it as a green dot; the countdown | table, cascade, prediction pause | |
| 05_6 | Exactly five, the true count. The largest number on its own averages only four. | The countdown gives way to a dim axis with a tick at each whole number from 1; the ten green dots fly from their rows and stack on it at their values; a green fulcrum under the stacks at 5 with "average = 5" (`G-F5.average`); on "The largest number on its own" a second axis below it, and each pair's larger number flies as a gold dot from its gold square, stacks, and its fulcrum comes in at 4 with "average = 4" (`G-F5.largest_average`) | balance, identity-preserving move | |
| 05_7 | A recipe is unbiased if its estimates average to the true count, whatever that count is. A single small company isn't a proof. So why does adding the average gap work? | On "A recipe is unbiased" "unbiased: average estimate = count" at the upper right, average estimate green; on "whatever that count is" "count 5: average 5" under it, average 5 green; on "isn't a proof" "every other count: ?" in dim; on "So why does adding" an ink halo round that open question | equation card, halo | |

### Notes for 05

- `G-F5.handfuls` 13,019,909 handfuls of five from `G-F5.company` 71. It is
  the same count as `G-F8.handfuls`: the oracle counts it as ordered draws of
  five over the orders of five, and checks that against chapter 08's brute
  force. The voice says the whole millions, `G-F5.millions` 13, and says
  there are more (`G-F5.handfuls_over_millions`).
- `G-F5.pairs` 10 in `G-F5.pair_list` order, from `G-F5.first_pair` 1, 2 to
  `G-F5.last_pair` 4, 5. `G-F5.estimates` are exact, shown as
  `G-F5.estimates_shown`; `G-F5.average` 5; `G-F5.largest_average` 4.
- The worked pairs. `G-F5.example_high_pair` is 2, 5: its unseen boxes are
  `G-F5.example_high_unseen` 1, 3, 4, which is
  `G-F5.example_high_unseen_count` 3 boxes in two gaps,
  `G-F5.example_high_gaps` 1 and 2. Its estimate is `G-F5.example_high`
  13/2, said "six and a half" (`G-F5.example_high_decimal` 6.5).
  `G-F5.example_low_pair` 1, 2 has no gaps, so `G-F5.example_low` is 2.
- The two boxes seen in 05_1 are the worked pair 2 and 5, so the small boxes
  carry from "you see two" through 05_4.
- The scene recomputes every estimate from its pair's gaps and refuses a
  table the facts contradict. As negative controls, the oracle checks that
  adding the unseen count undivided, or dividing it over three gaps, does not
  average five.
- The doubling recipe also averages five on this company
  (`G-F12.doubling_estimates`). So 05 claims only that the gap recipe
  averages five here, and nothing about other recipes; chapter 10 makes that
  comparison.
- Prediction (05_5): the tempting answers are "a bit under five", because
  the first rows look low, or "a bit over", because four of the ten are 6.5.
  The line at the count is drawn before the question so the viewer can weigh
  the dots. Nothing shows the average until the countdown ends.
- Pauses: a thought pause after "far too many to look at." while the count
  finishes; another after "Some are too low and some too high." while the
  dots land; a reveal pause after "Exactly five, the true count."; a beat
  after "A single small company isn't a proof." before the question that
  opens 06. The inspection hold after 05_2 is a scene hold.

## 06_gaps_trade_places: Gaps that can trade places (~136 s)

Chapter 05's tiny table, in the same place: the column numbers 1 to 5, the
ten rows of five small squares with each pair's two squares gold, and "each
pair: 1 in 10" under it; its estimates and the balance are gone. To its right
go three columns, one for each gap of a pair: below, between and above, the
top gap's head in hero. Highlights are halos and colour changes are
replacements, as in chapters 04 and 05; no typeset text is indicated,
recoloured or scaled in place, and nothing is drawn on top of a copy of
itself. Every number is `G-F5` or `G-F6`, or `G-F1` and `G-F3` on the friends'
line.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Here's why, starting with the ten pairs from five boxes. Each pair leaves three boxes unseen. Its two numbers cut them into three gaps, below the smaller, between the two, and above the larger. | The table comes in as chapter 05 built it; on "Each pair leaves" every row's three unseen squares are replaced by gap squares, filled blue in a gap below the larger number and a hero outline in the top gap; on "below the smaller", "between the two" and "above the larger" the three column heads come in, one on each: "below", "between" and, in hero, "above" | table, replacement | |
| 06_2 | One and two leave gaps of zero, zero and three. Two and five leave one, two and zero. Every pair has its own three gaps, and they always add up to three. | On "One and two leave" a gold halo round the first row; on "zero, zero and three" its gap sizes 0, 0 and 3 under the heads (`G-F6.low_gaps`); on "Two and five leave" a gold halo round its row; on "one, two and zero" its 1, 2 and 0 (`G-F6.high_gaps`); on "Every pair has" the other eight rows' sizes cascade down (`G-F6.gap_lists`), below and between in blue, above in hero; on "add up to three" "below + between + above = 3" at the upper right | table, cascade, halo | |
| 06_3 | A list can also be read backwards. One, two and zero says skip one box, take the next, skip two, take the next, and stop. That rebuilds two and five. | At the right, the list 1, 2, 0 in its three columns over a row of five empty squares numbered 1 to 5; on "skip one box" a blue halo round the 1 and the first square fills blue; on "take the next, skip" the second fills gold; on "skip two" a blue halo round the 2 and the third and fourth fill blue; on "take the next, and stop" the fifth fills gold; on "and stop" a hero halo round the 0, and nothing is added; on "That rebuilds" gold halos round the built row and round the table's row for 2 and 5, square for square the same | lagged reveal, halo, table | |
| 06_4 | So pairs and gap lists match up exactly. The ten lists are every way to split three unseen boxes into three gaps, each exactly once. And since each pair is one in ten, so is each list. | The built row goes; on "match up exactly" "pair" and "gap list" with a two-headed arrow between them under the sum; on "every way to split" "every split of 3 into 3 gaps, once each" under that, and an ink halo round the whole column of lists (`G-F6.one_pair_per_list`); on "each pair is one in ten" a halo round "each pair: 1 in 10"; on "so is each list" "each list: 1 in 10" under it (`G-F6.each_chance`); held to be read | equation card, halo | hold |
| 06_5 | Now let the gap below and the gap above trade places. For one and two, zero, zero and three becomes three, zero and zero. That's the list for four and five, another row of the table, just as likely. | The upper right clears; on "trade places" a gold halo round the first row, and its list is written again at the right after "1, 2:"; on "zero, zero and three becomes" the written 0 lifts and the 3 drops, they slide past each other and settle in each other's places, 3, 0, 0, each keeping its colour; on "That's the list for four and five" the traded list slides down to the last row's height, and halos come in round it and round the row for 4 and 5, its squares and its list (`G-F6.low_traded`, `G-F6.low_partner`); held to be compared | morph, identity-preserving move, halo | hold |
| 06_6 | Every row has a partner like that, and a row with equal gaps below and above is its own partner. So the below column and the above column hold the same ten numbers, in a different order. | The traded list goes; on "Every row has a partner" halos come and go round each row's list and its partner's, a pair at a time (`G-F6.partners`: rows 1 and 10, 2 and 9, 3 and 7, 5 and 8), then round the rows of 1, 5 and of 2, 4 alone; on "So the below column" "below: 0, 0, 0, 0, 1, 1, 1, 2, 2, 3" at the right in blue, the column top to bottom (`G-F6.below_values`); on "and the above column" "above: 0, 0, 0, 0, 1, 1, 1, 2, 2, 3" under it in hero (`G-F6.above_values`) | halo, equation | |
| 06_7 | Trading the gap between with the gap above works the same way. So all three columns hold the same ten numbers, and they must have the same average. | On "Trading the gap between" "between: 0, 0, 0, 0, 1, 1, 1, 2, 2, 3" under them in blue (`G-F6.between_values`); on "all three columns" an ink halo round the three lines | equation, halo | |
| 06_8 | Add up each column. Each holds ten boxes, over ten pairs, so each gap averages one box. Three unseen boxes, shared evenly by three gaps. That includes the top gap, above the larger number. | On "Add up each column" each column's sum, 10, under its last row, with "sum" beside them (`G-F6.column_sums`); on "so each gap averages" each column's average, 1, under its sum, the top gap's in hero, with "average = sum ÷ 10" beside them (`G-F6.position_averages`); on "Three unseen boxes" "3 unseen ÷ 3 gaps = 1 each" at the right; on "That includes the top gap" a hero halo round the above column's average; held to be read | table, equation, halo | hold |
| 06_9 | Now back to the friends' five boxes. Five numbers cut the unseen boxes into six gaps, but only five arcs are drawn. ⟨predict⟩ Where is the sixth gap? | The table and everything beside it fade; the friends' line from 1 to 84 comes back as chapter 04 drew it, the five boxes on their gold ticks, and the badge in the corner; on "into six gaps" the five blue arcs and their counts 13, 12, 5, 17 and 8 under the line (`G-F3.gaps`); on "only five arcs" a blue halo round the arcs; the countdown | number line, callback, prediction pause, think time | |
| 06_10 | Above sixty. It's the top gap, which nobody can see. In the small company, the top gap averaged one box, just like the gaps below it. | The countdown's "?" gives way to the dashed hero bracket, drawing in dash by dash from 61 off the faded end, with its "?" under it, as in 03_5; on "It's the top gap" a hero halo round that "?"; on "In the small company" "small company: each gap averaged 1" at the panel's top (`G-F6.top_average`); on "just like the gaps below it" a blue halo round the five arcs | bracket, callback, halo | |

### Notes for 06

- The table is chapter 05's, square for square (`G-F5.pair_list`): chapter
  05 ends by fading everything, so 06 builds the same squares in the same
  places and brings them back. `G-F6.gap_lists` gives each row's gaps, below,
  between and above, in row order; they add to `G-F6.unseen` 3 in every row
  and there are `G-F6.positions` 3 of them.
- The worked rows are chapter 05's: `G-F5.first_pair` 1, 2 with
  `G-F6.low_gaps` 0, 0, 3 and `G-F5.example_high_pair` 2, 5 with
  `G-F6.high_gaps` 1, 2, 0, which 06_3 reads backwards into the pair again.
- The matching (06_4): `G-F6.one_pair_per_list` checks that the ten lists
  are the ten splits of 3 into three parts, once each (stars and bars); each
  pair, and so each list, has chance `G-F6.each_chance` 1/10.
- The trade (06_5) is done on a copy written at the right, never on the
  first row, so the table keeps every row's true list; the copy lands beside
  the row for 4 and 5 (`G-F6.low_traded` 3, 0, 0, `G-F6.low_partner` 4, 5),
  never on top of its list. Its numbers keep the colours of the places they
  came from, so the viewer can see which value moved where.
- `G-F6.partners` lists each row's partner by trading below and above, rows
  counted from 1: 10, 9, 7, 4, 8, 6, 3, 5, 2, 1. The rows of 1, 5 and 2, 4
  are their own partners. The three columns' values in order,
  `G-F6.below_values`, `G-F6.between_values` and `G-F6.above_values`, are all
  0, 0, 0, 0, 1, 1, 1, 2, 2, 3; `G-F6.column_sums` 10, 10, 10;
  `G-F6.position_averages` 1, 1, 1; `G-F6.top_average` 1.
- Why the trade needs a fair draw: the oracle's negative control weights each
  pair by its larger number, and then the top gap averages 3/4 while the gap
  below averages 9/8. The badge is not on screen in the table; 06_4 says
  "each pair is one in ten", the condition the argument uses.
- 06_9 to 06_10 return to the friends' line: `G-F6.friends_positions` 6 gaps,
  `G-F6.friends_seen_gaps` 5 seen, the counts `G-F3.gaps`, and the hidden
  one above `G-F1.largest` 60.
- Prediction (06_9): 03_5 named the top gap, so this checks that the viewer
  still has the picture once the table has made the gaps into positions. A
  viewer who has lost the thread looks between the boxes for a missing arc.
  The countdown shows the five arcs and nothing above sixty; the bracket and
  its "?" come only with the answer.
- The draft's card C (06_3) is folded into chapter 07's card D, which states
  the general matching and averages.
- Pauses: a beat after "Each pair leaves three boxes unseen."; a thought
  pause after "That rebuilds two and five." while the halos compare the two
  rows; after "each exactly once." before the chance; a beat after "Add up
  each column." while the sums land; a reveal pause after "Above sixty.". The
  inspection holds after 06_4, 06_5 and 06_8 are scene holds.

## 07_right_on_average: Right on average, for any count (~166 s)

The friends' line as chapter 06 left it: the five boxes on their gold ticks,
the five blue arcs with their counts under the line, the dashed hero bracket
from 61 with its "?", and the badge in the corner. Numbers came first; here
the panel carries a letter for the first time, N for the count. The recipe's
stand-in for the top gap is called the guess. Highlights are halos and colour
changes are replacements. Every number is `G-F1`, `G-F3`, `G-F6` or `G-F7`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Now the same argument for any count. Call the count N. The friends' five numbers cut the N minus five unseen boxes into six gaps, five they can see and the top gap they can't. | On "Call the count N" the panel's top line reads "count = N"; on "unseen boxes into six gaps" "13 + 12 + 5 + 17 + 8 + top gap = N − 5" builds piece by piece under it, the counts blue and "top gap" hero; on "five they can see" a blue halo round the five counts under the line; on "the top gap they can't" a hero halo round the "?" | equation, halo | |
| 07_2 | The matching from the table still works. Every handful of five gives a list of six gaps adding up to N minus five, and every such list builds back its own handful. | On "The matching from the table" "handful" in gold, a two-headed arrow and "6 gaps adding to N − 5" in blue on the panel's third line; on "builds back its own handful" a gold halo round the five boxes | equation, halo | |
| 07_3 | Every handful is equally likely, so every list is too. Letting gaps trade places gives another list, just as likely, so all six gaps have the same average. Together they always hold N minus five boxes, so each gap averages N minus five, over six. | On "Every handful is equally likely" an ink halo round the badge; on "Together they always hold" a halo round the sum's "N − 5"; on "so each gap averages" "each gap averages (N − 5) ÷ 6" on the panel's last line, and "small company: 3 ÷ 3 = 1" in dim at the upper left (`G-F7.gap_average_formula`); held to be read | equation, halo, callback | hold |
| 07_4 | That includes the top gap, which the recipe can't see. So the recipe guesses it with the average of the five gaps it can see. For the friends, that's eleven. | The panel's two middle lines fade and "each gap averages (N − 5) ÷ 6" moves up under "count = N"; on "That includes the top gap" a hero halo round the "?"; on "the average of the five gaps" "guess = (13 + 12 + 5 + 17 + 8) ÷ 5 = 11" builds under it, guess and the counts blue; on "that's eleven" a blue arc at half strength over 61 to 71 with "11" over it, the guess, above the hidden top gap's bracket | equation, number line | |
| 07_5 | But eleven is one handful's average, and another handful would give another number. In the small company, each pair guesses its top gap with the average of its two gaps below. Row by row, the guess and the top gap agree only three times in ten. | On "But eleven is" a blue halo round the guess's 11; on "In the small company" an inset at the upper left builds row by row: the ten pairs in dim, then "guess", each pair's average of its gaps below (`G-F7.tiny_guesses_shown`: 0, 0.5, 1, 1.5, 0.5, 1, 1.5, 1, 1.5, 1.5), then "top gap", each pair's top gap (`G-F7.tiny_tops`: 3, 2, 1, 0, 2, 1, 0, 1, 0, 0); on "agree only three times" ink halos round the three columns where they match, the pairs 1 4, 2 4 and 3 4 (`G-F7.tiny_agree`); held to be inspected | table, lagged reveal, halo | hold |
| 07_6 | Yet over all ten pairs, the guesses average one box, and so do the top gaps. For the friends it's the same. Over all handfuls, each seen gap averages N minus five, over six, so the average of the five does too, exactly like the top gap. | On "the guesses average one box" "average 1" at the end of the guess row, and on "so do the top gaps" at the end of the top gap row (`G-F7.tiny_guess_average`, `G-F7.tiny_top_average`), with a halo round both; on "so the average of the five" "guess: averages (N − 5) ÷ 6 too" on the panel under the guess's sum; on "exactly like the top gap" a hero halo round the "?" and a blue one round the guess's arc; held to be read | equation, halo | hold |
| 07_7 | So for every handful, the count is the largest number plus the top gap, and the estimate is the largest number plus the guess. Over all handfuls, the guess and the top gap have the same average, so the estimates average the count. The recipe is unbiased, whatever the count. | The inset fades; on "the count is the largest number" the panel's lines give way to "count = largest + top gap", on "the estimate is the largest number" "estimate = largest + guess" under it, on "Over all handfuls" "estimate − count = guess − top gap", and on "have the same average" "on average: (N − 5) ÷ 6 − (N − 5) ÷ 6 = 0"; on "The recipe is unbiased" chapter 05's line "unbiased: average estimate = count" at the upper left, average estimate green, and an ink halo round it; held to be read | derivation, equation, halo | hold |
| 07_8 | ⟨predict⟩ What does the recipe do with a handful of just one box? | Everything but the badge goes; a fresh line from 1 to 84 with one gold box on its tick at 37 (`G-F7.one_box`), one blue arc over 1 to 36 with no count, and a dashed hero bracket from 38 off the faded end with its "?"; nothing shows the gap's size or an estimate; the countdown | number line, prediction pause, think time | |
| 07_9 | It doubles the number and subtracts one. One box leaves two gaps, and the gap below is the only clue to the gap above. Box thirty-seven has thirty-six boxes below it, so the recipe says seventy-three. | The countdown's "?" gives way to "estimate = largest + (largest − 1) ÷ 1" and "= twice the largest − 1" on the panel; on "One box leaves two gaps" a blue halo round the arc and a hero halo round the bracket; on "the only clue" a blue arc at half strength over 38 to 73, the guess, as long as the arc below; on "thirty-six boxes below it" "36" under the arc below (`G-F7.one_box_below`); on "the recipe says" "37 + 36 ÷ 1 = 73" builds piece by piece, 37 gold, 36 blue, 73 green, and a green tick at 73 with "73" over it (`G-F7.one_box_estimate`) | equation, number line | |
| 07_10 | Nothing in the proof needed five. With any handful size, the numbers cut the unseen boxes into one more gap than there are numbers, all with the same average. So the recipe always adds the largest minus the size, over the size, and it's right on average for every count, as long as the handful is fair. | The panel's lines fade; on "one more gap than there are numbers" "k numbers: k + 1 gaps"; on "all with the same average" "each gap averages (N − k) ÷ (k + 1)"; on "So the recipe always adds" "estimate = largest + (largest − k) ÷ k", with "k = 5: 60 + 55 ÷ 5 = 71" in dim under it; on "as long as the handful is fair" an ink halo round the badge; card D rises and stays until it can be read | formula, card, halo | hold |

### Notes for 07

- 07_1 to 07_3 are chapter 06's argument with the count a letter:
  `G-F6.friends_positions` 6 gaps, `G-F6.friends_seen_gaps` 5 seen, the
  counts `G-F3.gaps` 13, 12, 5, 17, 8, and `G-F7.gap_average_formula`
  (N − 5)/6. The oracle checks the matching and the equal averages at every
  position for k ≤ 4, N ≤ 11, and that trading any two positions maps the
  lists onto themselves.
- The guess (07_4) is the average of the seen gaps, for the friends
  `G-F3.average_gap` 11. 07_5 keeps one handful's average apart from the
  average over handfuls: in the tiny company, row by row, the guess
  (`G-F7.tiny_guesses`) and the top gap (`G-F7.tiny_tops`) agree in only
  `G-F7.tiny_agree` 3 rows of 10, yet both average 1
  (`G-F7.tiny_guess_average`, `G-F7.tiny_top_average`). In every row the
  estimate minus the count is the guess minus the top gap (oracle check), so
  05's estimates are the same table read another way.
- Unbiased for every count (07_7): `G-F7.general_unbiased` (k ≤ 5, N ≤ 21,
  by largest number) and the oracle's enumeration (k ≤ 6, N ≤ 22) and
  closed form (k = 5, N ≤ 400). The oracle also checks that the guess
  averages (N − k)/(k + 1) for k ≤ 4, N ≤ 11. Its negative control: guessing
  the top gap with the widest seen gap instead of their average averages
  above N.
- The one-box handful (07_8, 07_9): `G-F7.one_box_handful` 1,
  `G-F7.one_box` 37, `G-F7.one_box_below` 36, `G-F7.one_box_gaps` 2,
  `G-F7.one_box_estimate` 73; `G-F7.one_box_recipe` 2m − 1 and
  `G-F7.one_box_unbiased`. 37 is illustrative: no claim rests on it. It is the
  friends' average, so the doubling recipe of chapter 10 also gives the
  friends 73; the voice does not connect the two here.
- Prediction (07_8): the tempting answers are "thirty-seven", the only number
  seen, and "nothing, there are no gaps to average", from 04's edge handful.
  The countdown shows the one box, its arc with no count and the hidden top
  gap; nothing shows 36, 73 or the doubling before the answer.
- Card D (07_10): "for a fair handful of k from boxes 1 to N, the top gap and
  the guess (largest − k) ÷ k both average (N − k) ÷ (k + 1), so the estimates
  average exactly N, for every N of at least k." The same recipe,
  m(1 + 1/k) − 1, is Clark, Gonye and Miller (2021), p. 3 (`G-F3.estimate`'s
  pin); Goodman (1952) was not read (`docs/brief.md`, "Sources").
- Pauses: a clause pause after "each gap averages N minus five," so "over six"
  divides the whole difference; a thought pause after "For the friends,
  that's eleven." before the contrast; a beat after "so the estimates
  average the count."; a reveal pause after "It doubles the number and
  subtracts one.". The inspection holds after 07_3, 07_5, 07_6, 07_7 and
  07_10 are scene holds; the one after 07_5 lets the halos round the three
  agreeing pairs stand, so no pause row adds silence there.

## 08_one_estimate: One estimate is not the average (~136 s)

The friends' line from 1 to 84 as chapter 03 left it: the five boxes on
their gold ticks, their estimate as a green tick at 71 with its number, and
the badge. Then a company that made exactly 71 boxes, an upright ink mark at
71 over the line, and its handfuls: first twenty drawn by computer, each a
row of five gold rings at half strength (chapter 02's look) whose estimate
drops as a green dot onto a stack over the line; then all 13,019,909 of them
as 67 green bars over the same line, one per largest number, each standing
at that stack's estimate. The bars are built from how many handfuls have
each largest number, never by listing handfuls. Highlights are halos and
colour changes are replacements; no typeset text is indicated, recoloured or
scaled in place. Every number is `G-F1`, `G-F3`, `G-F5` or `G-F8`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Right on average is a promise about the recipe across all handfuls, not about our seventy-one. To see what it promises, imagine a company that made exactly seventy-one boxes, and friends who could buy five fresh boxes from it, again and again. | The line opens with the boxes on their ticks, the green tick at 71 with "71" over it and the badge; on "not about our seventy-one" a green halo round that "71"; on "imagine a company" the boxes, their ticks and the green tick fade, the five gold dots give way to plain ones, an upright ink mark rises at 71 and the panel's top line reads "count = 71" (`G-F8.count`); on "buy five fresh boxes" an ink halo round the badge | number line, badge, halo | |
| 08_2 | Here are twenty such handfuls, drawn by computer. | On "Here are twenty such handfuls" twenty rows drop in one after another at one height over the line (`G-F8.again_handfuls`), five gold rings at half strength each; as each row lands the row before it fades and its estimate drops as a green dot onto a stack over its place on the line (`G-F8.again_estimates`), no stack more than three high; the rows keep landing after the words end, a protected visual task | computed experiment, lagged reveal, number line | |
| 08_3 | Their estimates land between about forty-five and eighty-four point two, and three of them land exactly on seventy-one. Together they average about seventy-two. | On "Their estimates land between" green halos round the lowest dot and the highest, with "44.6" and "84.2" under the line (`G-F8.again_lowest`, `G-F8.again_highest`); on "three of them land exactly" a green halo round the stack on the mark at 71 (`G-F8.again_exact`); on "Together they average" a short green upright mark under the line at 72.32 with "average 72.32" under it (`G-F8.again_average`) | halo, number line | |
| 08_4 | Twenty is only twenty. This company has over thirteen million handfuls of five, and we can count them all, because the recipe looks only at the largest number. Just one handful tops out at five. Five handfuls top out at six. Over nine hundred thousand top out at seventy-one. So the handfuls fall into sixty-seven stacks by their largest number, and every handful in a stack gives the same estimate. | On "Twenty is only twenty" the dots, their numbers and the average mark fade; on "over thirteen million" "13,019,909 handfuls of 5" on the panel's second line (`G-F8.handfuls`); on "the recipe looks only at" "estimate = largest + (largest − 5) ÷ 5" on its third line; a table at the upper left under the badge, heads "largest", "handfuls" and "estimate", gains a row on each of the next three sentences: on "Just one handful tops out" 5, 1, 5 (`G-F8.topped_by_smallest`); on "Five handfuls top out" 6, 5, 6.2 (`G-F8.next_largest`, `G-F8.topped_by_next`); on "Over nine hundred thousand" 71, 916,895, 84.2 (`G-F8.topped_by_count`, `G-F8.highest_estimate`), and that stack's bar rises at 84.2, the tallest; on "sixty-seven stacks" the other bars grow left to right, each at its stack's estimate (`G-F8.stacks`), except the stack at exactly 71, whose place stays empty under a hero "?"; held to be inspected | histogram, table, lagged reveal | hold |
| 08_5 | ⟨predict⟩ Out of all those handfuls, how often do you think the estimate is exactly seventy-one, the true count? | The bars stand with the hero "?" over the empty place at 71; the countdown at the left, under the table; nothing shows that stack's size | histogram, prediction pause, think time | |
| 08_6 | About one handful in twenty-nine. Three in twenty was lucky. Being right on average doesn't mean being right often. | The countdown and the "?" give way to the bar at 71, rising in green like the others, and an ink halo round it; on "About one handful" "exactly 71: 1 in 29" on the panel's last line (`G-F8.exactly_right_one_in`); on "Three in twenty was lucky" "20 draws: 3 exactly 71" in dim under the table (`G-F8.again_draws`, `G-F8.again_exact`) | histogram, halo | |
| 08_7 | Yet all the bars together balance at seventy-one exactly, while our twenty draws averaged about seventy-two. The bars are lopsided, though. Fifty-eight percent of handfuls overestimate, by about eight boxes on average, and the estimates that fall short miss by twelve on average. | On "balance at seventy-one exactly" a green fulcrum comes in under the line at 71 with "balance point: 71" left of it (`G-F8.average`); on "our twenty draws averaged" "20 draws: average 72.32" in dim joins the table's foot; on "The bars are lopsided" ink halos round the bars right of 71 and round those left of it, together; on "Fifty-eight percent of handfuls" a dim bracket under the bars right of 71 with "58% too high, by about 8" under it (`G-F8.overestimate_percent`, `G-F8.overshoot_about`); on "the estimates that fall short" a dim bracket under the bars left of 71 with "too low: by 12" (`G-F8.undershoot`); held to be inspected | histogram, highlight, halo | hold |
| 08_8 | So how far off is a typical estimate? Ignoring the direction, an estimate misses by about nine boxes on average. Statisticians often quote a different width, the standard deviation. It squares each miss before averaging, then takes the square root, so big misses count for more, and here it comes to about eleven point seven. | The brackets and their words go; on "misses by about nine boxes" an ink span under the line from 71 − 9.2 to 71 + 9.2 with "average miss: 9.2" under it (`G-F8.average_miss_decimal`); on "the standard deviation" a dim span under that one from 71 − 11.7 to 71 + 11.7 with "standard deviation: 11.7" (`G-F8.sd_about`); on "It squares each miss" the panel's lines under "count = 71" give way to "average miss = average distance from 71" and "standard deviation = √(average squared miss)"; held to be read | histogram, formula, halo | hold |
| 08_9 | So keep these ideas apart. The recipe is unbiased, so its estimates balance at the true count. They spread out around it, by about nine boxes on average. And our seventy-one is a single estimate, from a single handful. | On "The recipe is unbiased" a green halo round the fulcrum and its words; on "They spread out around it" an ink halo round the two spans; on "And our seventy-one" "our 71: a single estimate" in green on the panel's last line; card E rises and stays until it can be read | histogram, halo, card | hold |

### Notes for 08

- The company is `G-F8.count` 71, the friends' own estimate (`G-F3.estimate`),
  so the mark at 71 is where both the true count and our estimate sit; the
  scene refuses a count that differs from the estimate. Its handfuls of five:
  `G-F8.handfuls` 13,019,909 (chapter 05's "over thirteen million",
  `G-F5.millions`).
- The twenty rows: stream 3 of chapter 02's generator, the first twenty drawn
  (`G-F8.again_draws`), estimates `G-F8.again_estimates`; lowest
  `G-F8.again_lowest` 44.6 ("about forty-five", `G-F8.again_lowest_about`),
  highest `G-F8.again_highest` 84.2 (also the highest any handful can give,
  `G-F8.highest_estimate`), exactly 71 in `G-F8.again_exact` 3 rows, average
  `G-F8.again_average` 1808/25 = 72.32 ("about seventy-two",
  `G-F8.again_average_about`). The voice claims nothing else about them; the
  exact statements come from every handful. "Three in twenty was lucky": the
  oracle checks that three or more exact estimates in twenty fair draws has
  chance under one in twenty.
- Counting every handful by its largest number: a handful topped by m has its
  other four below m, so there are C(m − 1, 4) of them, and the recipe gives
  each the same estimate, m + (m − 5) ÷ 5. Stacks run from m = 5 (one
  handful, `G-F8.topped_by_smallest`, estimate `G-F8.lowest_estimate` 5)
  through m = 6 (`G-F8.topped_by_next` 5, estimate 6.2) to m = 71
  (`G-F8.topped_by_count` 916,895, over `G-F8.topped_by_count_over` 900,000
  and under a million, estimate 84.2, the tallest bar): `G-F8.stacks` 67
  stacks, 1.2 apart, summing to every handful (oracle check, and brute force
  over all 13,019,909 handfuls). Only m = 60 gives exactly 71.
- Exact, from every handful: `G-F8.average` 71 (unbiased);
  `G-F8.exactly_right` 65018/1859987, about 1 in 28.6
  (`G-F8.exactly_right_one_in` 29); `G-F8.overestimate_share`
  1079771/1859987 (`G-F8.overestimate_percent` 58), overshooting by
  `G-F8.overshoot` 780216/98161 ≈ 7.95 on average (`G-F8.overshoot_about`
  8), while those below 71 fall short by exactly `G-F8.undershoot` 12 on
  average. The negative control: the median estimate is 74.6, not 71, so
  "balance" is the average and never the middle.
- The two widths: the average miss `G-F8.average_miss` 17164752/1859987 ≈
  9.23 (`G-F8.average_miss_decimal` 9.2, "about nine",
  `G-F8.average_miss_about`), the average distance from 71 ignoring
  direction; the standard deviation, the square root of `G-F8.variance`
  4752/35 = (71 − 5)(71 + 1) ÷ (5 × 7) (oracle check of the closed form),
  ≈ 11.66 (`G-F8.sd_about` 11.7, rounded exactly from the fraction). The
  negative control: the average miss squared is less than the variance, so
  the two widths differ for every spread-out recipe.
- Card E (08_9), built from the facts: "for N = 71 and handfuls of 5, all
  13,019,909 estimates average exactly N; they miss by 9.2 on average, and
  their standard deviation is √((N − 5)(N + 1) ÷ 35) ≈ 11.7."
- Prediction (08_5): the tempting answer is "most of the time", from 07's
  "right on average". The countdown shows every other stack with the place
  at 71 empty under its "?"; its size, 1 in 29 and the table's row for it
  come only with the answer. The video never says "the probability that the
  count is 71": the count is fixed in this model.
- Pauses: a thought pause after "not about our seventy-one."; the twenty rows
  landing after 08_2 are a protected visual hold between 08_2 and 08_3, not a
  pause row; a turn after "Twenty is only twenty."; a beat after "top out at
  seventy-one." while the tallest bar rises; a reveal pause after "About one
  handful in twenty-nine."; a beat after "The bars are lopsided, though."; a
  thought pause after the question "So how far off is a typical estimate?".
  The inspection holds after 08_4, 08_7, 08_8 and 08_9 are scene holds.

## 09_how_far_off: How far off could it be? (~142 s)

The friends' line again: their five boxes on gold ticks and the badge. Then
the line's end slides out to 130 (a layout value, never written or said)
with the boxes in their row at the left, and over it one dim bar per
possible count from 60 to 130: the exact chance
that a fair handful of five from that many boxes would all be 60 or less,
the handfuls of five from the first 60 boxes divided by all the handfuls of
five. The range is an ink bar under the line. Then the coverage experiment:
the count fixed at 71, an upright ink mark there, and chapter 08's twenty
computed handfuls, each building its own range by the same rule. The voice
never calls the 95% a chance about the count. Highlights are halos and
colour changes are replacements. Every number is `G-F1`, `G-F8` or `G-F9`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Back to the friends' own boxes. Their count is a fixed number nobody knows, at least sixty. How large could it be? Flip the question around. For each possible count, how often would a fair handful of five all be sixty or less? | The line opens with the boxes on their ticks and the badge; on "a fixed number nobody knows" "count: fixed, unknown" on the panel's top line; on "at least sixty" a teal halo round the box at 60 (`G-F1.at_least`); on "How large could it be" the boxes spread into their row, the line's faded end fills in and the end slides out to 130 under them; on "For each possible count" "chance all 5 are at most 60" on the panel's second line, and dim bars grow left to right, one per count from 60 to 130, each as tall as that chance | number line, zoom out, bars, lagged reveal | |
| 09_2 | If the count were seventy-one, about forty-two percent of the time. If it were a hundred, about seven percent. The bigger the count, the rarer a handful this low. | On "If the count were seventy-one" "71: 42%" under the line at 71 and an ink halo round the bar there (`G-F9.at_most_60.71.percent`); on "If it were a hundred" "100: 7%" under the line at 100 and a halo round its bar (`G-F9.second_count`, `G-F9.at_most_60.100.percent`); on "The bigger the count" an ink halo round the bars from 100 on | bars, halo | |
| 09_3 | At a hundred and seven boxes the chance is still just over five percent. At a hundred and eight it's just under, and it keeps falling from there. Counts that big would rarely give a handful like ours. | On "At a hundred and seven boxes" a dashed ink line across the bars at 5%, with "5%" just past the line's end (`G-F9.level`, `G-F9.level_percent`), an ink halo round the bar at 107 and "107 boxes: 5.1%" on the panel's third line (`G-F9.at_most_60.107.percent1`); on "At a hundred and eight" a halo round the bar at 108 and "108 boxes: 4.9%" on the last line (`G-F9.first_rare`, `G-F9.at_most_60.108.percent1`); on "it keeps falling" the bars from 108 on give way to faint ones | bars, highlight, halo | |
| 09_4 | So keep every count from sixty to a hundred and seven, the counts that give a handful like ours at least five times in a hundred. That's our range. | On "So keep every count" the two chances under the line go, and an ink span under the line from 60 to 107 with a tick at each end and "range: 60 to 107" under it (`G-F9.range`); on "at least five times in a hundred" an ink halo round the dashed line and its "5%"; held to be read | bars, number line, halo | hold |
| 09_5 | Is a range built this way any good? Test it where we know the answer. Fix the count at seventy-one, and let the twenty handfuls drawn by computer each build their own range the same way. | On "Test it where we know" the bars, the dashed line, the panel's chances, the range, the boxes and their ticks go, and the friends' gold dots turn plain; on "Fix the count at seventy-one" the panel's top line becomes "count = 71" (`G-F8.count`) and an upright ink mark rises at 71; on "each build their own range" twenty rows drop in one after another, bottom to top (`G-F8.again_handfuls`), each a small gold ring at half strength at its handful's largest number and a dim range from there to its top (`G-F9.again_ranges`); the rows keep landing after the words end, a protected visual task | computed experiment, lagged reveal, number line | |
| 09_6 | Nineteen of the twenty ranges cover seventy-one. The range that misses came from a handful topped by thirty-eight. Over every handful, ranges built this way cover seventy-one about ninety-six times in a hundred, and whatever the count, never fewer than ninety-five. | On "Nineteen of the twenty ranges cover" the nineteen ranges across the mark give way to ink ones (`G-F9.again_cover`); on "The range that misses" an ink halo round the dim range left of the mark, with "largest 38" before its ring (`G-F9.again_miss_largest`); on "Over every handful" "every handful: 96% cover 71" on the panel's second line (`G-F9.cover_71_percent`); on "never fewer than ninety-five" "any count: at least 95%" on its third line (`G-F9.worst_coverage_at_least_95`); held to be inspected | computed experiment, halo, replacement | hold |
| 09_7 | That's what ninety-five percent confidence means. The count stays fixed, and the range changes from handful to handful. It's a promise about the method, not a ninety-five percent chance that the count lies in our range. | On "ninety-five percent confidence" "95% confidence" on the panel's last line (`G-F9.confidence_percent`); on "The count stays fixed" an ink halo round the mark; on "the range changes" an ink halo round the twenty ranges; on "a promise about the method" an ink halo round "95% confidence"; the screen never writes a chance about the count | equation card, halo | |
| 09_8 | Behind each bar was a single division. Count the handfuls of five from the first sixty boxes, and divide by all the handfuls of five from the whole company. At a hundred and seven boxes, that's just over five percent. | The rows, the mark and the panel's lines go; the bars, the dashed line and the range come back as 09_4 left them; on "Count the handfuls of five" "chance = handfuls of 5 from 60" on the panel's top line and "÷ handfuls of 5 from N" under it; on "At a hundred and seven boxes" "N = 107: 5,461,512 ÷ 106,308,566 ≈ 5.1%" on its third line (`G-F9.from_largest`, `G-F9.from_range_top`), and an ink halo round the bar at 107; held to be read | formula, bars, halo | hold |
| 09_9 | ⟨predict⟩ If the friends had ten boxes instead, with the same largest number, would the range get wider or narrower? | The panel's lines give way to "10 boxes, largest still 60" (`G-F9.ten_boxes`); the countdown | prediction pause, think time, bars | |
| 09_10 | Narrower. Ten numbers all sixty or less would be rare from a big company, so the range shrinks to sixty through seventy-nine. | The countdown goes; on "Ten numbers all sixty or less" the bars give way to the chances for ten, the handfuls of ten from the first 60 boxes over all the handfuls of ten, falling much faster; on "the range shrinks" the range gives way to one from 60 to 79 with "range: 60 to 79" (`G-F9.ten_boxes_range`); card F rises and stays until it can be read | bars, replacement, card | hold |

### Notes for 09

- The chances: `G-F9.at_most_60.71` 780216/1859987 (≈ 0.42, "about forty-two
  percent"), `G-F9.at_most_60.100` 32509/448140 (≈ 0.073, "about seven"),
  `G-F9.at_most_60.107` 390108/7593469 ≈ 0.0514 (`…percent1` 5.1) and
  `G-F9.at_most_60.108` 32509/663507 ≈ 0.0490 (4.9), the first under
  `G-F9.level` 1/20; the oracle checks every count from 108 to 400 is rare
  and the chance keeps falling. 107 is `G-F9.range_top` (`G-F9.range` 60,
  107), and the scene draws each bar from C(60, 5) ÷ C(N, 5) and refuses a
  range whose top is not where the bars cross 1/20. A first guess from
  (60/N)⁵ ≈ 1/20 gives 109 (oracle negative control); the exact bound is
  107, and that is what the voice says.
- The formula (09_8) is shown only after the bars and the coverage
  experiment: `G-F9.from_largest` C(60, 5) = 5,461,512 and
  `G-F9.from_range_top` C(107, 5) = 106,308,566.
- The coverage experiment: the count is fixed at `G-F8.count` 71 and the
  twenty handfuls are chapter 08's (`G-F8.again_handfuls`), each range from
  its largest to `G-F9.again_ranges`; `G-F9.again_cover` 19 cover 71 and
  `G-F9.again_miss` 1 misses, the handful topped by `G-F9.again_miss_largest`
  38, range 38 to 67. A range covers 71 exactly when its largest number is
  40 or more (oracle check), so over every handful it covers 71 with chance
  `G-F9.cover_71` 1777736/1859987 ≈ 0.956 (`G-F9.cover_71_percent` 96).
- "Whatever the count, never fewer than ninety-five": a range misses N only
  when the chance of a handful topped that low is under 1/20 (the bars fall
  as N grows), so it misses with chance under 1/20 for every N. The oracle
  also computes the coverage for every count from 5 to 150; its smallest is
  95.002%, at 144 (`G-F9.worst_coverage_at_least_95`).
- Negative controls: the range from the largest number to the estimate
  covers 71 only about 62% of the time, so the estimate is not the range's
  top; under a flat prior on 60 to 200 or 60 to 1,000 boxes, the posterior
  chance that the count is at most 107 is 91.8% or 91.2%, so the 95% is no
  posterior and depends on nothing like a prior. The screen and the voice
  never write the 95% as a chance about the count.
- Ten boxes (09_9, 09_10): `G-F9.ten_boxes` 10 with the same largest 60
  gives `G-F9.ten_boxes_range` 60, 79 (`G-F9.ten_boxes_top`). The five extra
  boxes are never drawn on the line: no number of theirs is shown or said.
- Card F (09_10), matching the ten-box experiment now shown: "range: each N from 60 up where 10
  fair numbers from 1 to N are all at most 60 at least 5% of the time. For
  any fixed N, such ranges cover N at least 95% of the time; not a chance
  about N." Three lines at every width the lower third narrows to.
- At this scale the bars at 107 and 108 differ in height by under a
  hundredth of a scene unit, so the cutoff is carried by the panel's two
  lines and by the bars from 108 on turning faint, not by the bar heights.
- Prediction (09_9): the tempting answer is "wider, more boxes means more
  uncertainty". The countdown shows only the new handful size and the old
  range; the ten-box chances and 79 come with the answer.
- Pauses: a thought pause after "How large could it be?" while the line's end
  slides out; a beat after "it keeps falling from there." while the bars past
  108 fade; the twenty rows landing after 09_5 are a protected visual hold
  between 09_5 and 09_6; a beat after "Nineteen of the twenty ranges cover
  seventy-one."; a thought pause after "That's what ninety-five percent
  confidence means."; a reveal pause after "Narrower.". The inspection holds
  after 09_4, 09_6, 09_8 and 09_10 are scene holds.

## 10_a_rival_recipe: A rival recipe (~149 s)

The friends' line from 1 to 84 as chapter 08 opened it: the five boxes on
their gold ticks, the gap recipe's green tick at 71 with its number over the
line, and the badge. A second recipe is earned from the middle number and
drawn in purple (the colour table's role for the doubling recipe), its marks
under the line so they never meet the green ones over it. Then a strange
handful keeps the box at 60 and swaps the other four for gold rings at half
strength on 2, 3, 4 and 5, as chapter 04 showed its edge handful. Then the
71-box company of chapter 08, every handful: the line's end slides out to
140 (a layout value, never written or said), chapter 08's 67 green bars come
back narrower at that scale, and the doubling recipe's handfuls, counted by
the total of their five numbers from a sum-of-subsets table and never by
listing handfuls, rise as a purple outline over them, stroke only, three
totals to a step so each step is as wide as a green bar's. Highlights are
halos and colour changes are replacements; no typeset text is indicated,
recoloured or scaled in place. Every number is `G-F1`, `G-F3`, `G-F8` or
`G-F10`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Here's another recipe, built on a different idea. A company's numbers run from one up to the count, so they balance at the middle number. With seventy-one boxes the middle is thirty-six, and the count is twice the middle, minus one. | The line opens with the boxes on their ticks, the green tick at 71 with "71" over it and the badge; on "they balance at the middle number" "middle = (count + 1) ÷ 2" on the panel's top line; on "With seventy-one boxes" an ink span under the line from 1 to 71 and a small ink fulcrum under it at 36 with "36" under the fulcrum (`G-F8.count`, `G-F10.middle_71`); on "twice the middle, minus one" "count = 2 × middle − 1" on the panel's second line (`G-F10.from_middle`) | number line, equation | |
| 10_2 | A fair handful's numbers should spread around the middle too, so let the handful's average stand in for it. Twice the average, minus one. Call it the doubling recipe. | On "let the handful's average stand in" the span, the fulcrum and "36" go; on "Twice the average, minus one" the panel's second line gives way to "doubling recipe = 2 × average − 1", the name purple (`G-F10.recipe`); on "Call it the doubling recipe" a purple halo round that line | equation, halo | |
| 10_3 | Our five add up to a hundred and eighty-five, an average of thirty-seven. So doubling says seventy-three, just above the gap recipe's seventy-one. | On "add up to" "(14 + 27 + 33 + 51 + 60) ÷ 5 = 37" on the panel's third line (`G-F10.sum`, `G-F10.average`), and a short purple mark under the line at 37 with "37" under it; on "So doubling says seventy-three" "2 × 37 − 1 = 73" on the panel's last line, the 73 purple, and a purple tick under the line at 73 with "73" under it (`G-F10.doubling`), beside the green tick at 71 over the line | equation, number line | |
| 10_4 | It's unbiased too. In a fair handful each box is just as likely to carry any number from one to the count, so averaged over every handful, the handful's average is the middle number. That makes the doubling recipe average the count, whatever the count is. | On "In a fair handful" an ink halo round the badge; on "the handful's average is the middle number" an ink halo round the panel's top line; on "That makes the doubling recipe average the count" a purple halo round the panel's recipe line | halo, equation | |
| 10_5 | So which recipe should we trust? Try a strange handful. Keep the box at sixty, and make the other four two, three, four and five. ⟨predict⟩ What does doubling say for this handful? | On "Try a strange handful" the purple marks, their numbers and the panel's last two lines go; on "Keep the box at sixty" a gold halo round the box at 60; on "make the other four" the four smaller boxes, their numbers and their ticks fade and their gold dots turn faint; on "two, three, four and five" gold rings at half strength land on the dots 2, 3, 4 and 5 with "2, 3, 4, 5" under them (`G-F10.silly_others`); the countdown; nothing shows the new average | prediction pause, think time, number line | |
| 10_6 | Twenty-eight point six. That's fewer than the sixty boxes we know were made. The gap recipe says seventy-one for this handful, as it does for every handful topped by sixty. | The countdown goes; on "Twenty-eight point six" "(2 + 3 + 4 + 5 + 60) ÷ 5 = 14.8" and "2 × 14.8 − 1 = 28.6" on the panel's last two lines, the 28.6 purple (`G-F10.silly_sum`, `G-F10.silly_average_decimal`, `G-F10.silly_doubling_decimal`), and a purple tick under the line at 28.6 with "28.6" under it; on "fewer than the sixty boxes" a teal halo round the box at 60 (`G-F1.at_least`); on "The gap recipe says seventy-one" a green halo round the green tick at 71 and its number (`G-F10.silly_gap`) | equation, halo, number line | |
| 10_7 | That's no freak. In a company of seventy-one boxes, about one handful in five gets a doubling estimate below its own largest number. The gap recipe never does. | On "about one handful in five" the panel's last two lines give way to "71 boxes: 20.9% of handfuls" and "doubling below their own largest", in purple (`G-F10.below_largest_71_percent1`); on "The gap recipe never does" a green halo round the green tick at 71 | equation, halo | |
| 10_8 | So put both recipes side by side, over every handful from a company of exactly seventy-one boxes. The gap recipe's sixty-seven bars come back. The doubling recipe looks only at the total of the five numbers, so its handfuls can be counted by total, and its estimates spread from five all the way to a hundred and thirty-seven. | On "put both recipes side by side" the rings and their numbers, the 28.6 tick and its number, the box at 60 with its number and tick, the green tick and its 71 and the panel's lines go, the dot at 60 turns faint as the faded dots even out, and the line's end slides out to 140; on "exactly seventy-one boxes" "count = 71" comes in on the panel's top line and an upright ink mark rises at 71 (`G-F8.count`); on "The gap recipe's sixty-seven bars come back" chapter 08's green bars rise left to right, one per largest number, narrower at this scale (`G-F8.stacks`); on "counted by total" the purple outline rises over them, one step for every three totals, 111 steps, its tallest step at 71 (`G-F10.bars_71`, `G-F10.tallest_bar_71`, `G-F10.tallest_bar_71_at`); on "from five all the way to" "5" and "137" in purple under the line at the outline's two ends (`G-F10.doubling_range_71`) | histogram, number line, zoom out, lagged reveal | |
| 10_9 | Both balance at seventy-one. But the purple bars are lower and wider. Ignoring direction, doubling misses by fourteen point three boxes on average, the gap recipe by nine point two. The standard deviation, built from squared misses, says seventeen point eight against eleven point seven. By either measure, the gap recipe's estimates spread less. | On "Both balance at seventy-one" an ink fulcrum under the line at 71 with "balance point: 71" left of it (`G-F10.doubling_average_71`, `G-F8.average`); on "lower and wider" a purple halo round the outline and a green halo round the bars, together; on "Ignoring direction" a table on the panel under "count = 71", heads "gap" in green and "doubling" in purple, then the row "average miss", 9.2 and 14.3 (`G-F8.average_miss_decimal`, `G-F10.doubling_miss_71_decimal`); on "The standard deviation" the row "standard deviation", 11.7 and 17.8 (`G-F8.sd_about`, `G-F10.doubling_sd_about`); held to be read | histogram, table, halo | hold |
| 10_10 | So the gap recipe, which uses only the largest number, beats the recipe that uses all five. What are the other four worth, then? | On "uses only the largest number" a green halo round the bars; on "the recipe that uses all five" a purple halo round the outline; the table stays until it can be read | histogram, halo, table | hold |

### Notes for 10

- Earning the recipe: the numbers 1 to N average (N + 1) ÷ 2, the middle
  (`G-F10.middle_71` 36 for 71 boxes), so N = 2 × middle − 1
  (`G-F10.from_middle`, checked by the oracle for every count to 200). A
  fair handful's average stands in for the middle: `G-F10.recipe` "2 ×
  average − 1". The friends' five: `G-F10.sum` 185, `G-F10.average` 37,
  `G-F10.doubling` 73.
- "It's unbiased too": each box of a fair handful is equally likely to be any
  number, so over every handful the handful's average averages the middle
  (`G-F10.average_of_averages_71` 36 over all 13,019,909 handfuls of the
  71-box company) and the doubling recipe averages the count
  (`G-F10.doubling_unbiased`, by enumeration for handfuls of 1 to 5 and
  every count to 16, and `G-F10.doubling_average_71` 71).
- The strange handful (10_5, 10_6): `G-F10.silly_handful` 2, 3, 4, 5, 60;
  `G-F10.silly_sum` 74, average `G-F10.silly_average_decimal` 14.8,
  doubling `G-F10.silly_doubling` 143/5 = 28.6, below the 60 the handful
  itself proves exist (`G-F1.at_least`); the gap recipe says
  `G-F10.silly_gap` 71, as for every handful topped by 60. Edges (oracle):
  doubling is never below the largest for one box or a pair; for handfuls
  of three to six it is for some handful from every count above the size;
  at N = k every handful is 1 to N and doubling says N.
- "About one handful in five" (10_7): at 71 boxes, doubling lands below its
  handful's own largest number for `G-F10.below_largest_71` 2721678/13019909
  ≈ 20.9% of handfuls (`G-F10.below_largest_71_percent1`; 1 in 4.78, said
  as `G-F10.below_largest_71_one_in` 5), from the count of handfuls by
  largest number and total together; the gap recipe never does (04).
- The bars (10_8): the green ones are chapter 08's, C(m − 1, 4) handfuls at
  m + (m − 5) ÷ 5 for m = 5 to 71 (`G-F8.stacks` 67). The doubling recipe
  depends only on the total s, at 2s ÷ 5 − 1, so its handfuls are counted by
  total with a sum-of-subsets table; totals run 15 to 345, so estimates run
  `G-F10.doubling_range_71` 5 to 137. Three totals 3m − 1, 3m and 3m + 1 make
  one step of the outline, 1.2 wide and centred on the gap recipe's value for
  largest m, so both sets of bars count handfuls per 1.2 of estimate on one
  scale: `G-F10.bars_71` 111 steps (m = 5 to 115), the tallest
  `G-F10.tallest_bar_71` 339,265 handfuls at `G-F10.tallest_bar_71_at` 71,
  about 0.37 of the green tallest (916,895 at 84.2). The scene builds the
  steps from the table and refuses them unless they hold every handful
  once; its tests check the table against a list of every handful for
  small companies.
- The widths (10_9), exact over every handful of the 71-box company, the
  correction of the draft's "misses by about fourteen boxes on average":
  the average miss, the average distance from 71 ignoring direction, is
  `G-F10.doubling_miss_71` 934167456/65099545 ≈ 14.35 (shown as
  `G-F10.doubling_miss_71_decimal` 14.3) against 08's `G-F8.average_miss`
  ≈ 9.23 (9.2); the standard deviation, the square root of the average
  squared miss (for an unbiased recipe the spread about the count itself),
  is √(`G-F10.doubling_variance_71` 1584/5) ≈ 17.80 (`G-F10.doubling_sd_about`
  17.8) against √(4752/35) ≈ 11.66 (11.7). Doubling is wider by both widths;
  each recipe's average miss squared is under its variance, so the two
  widths are different numbers for both (oracle checks). Doubling's average
  squared miss is (N + 1)(N − 5) ÷ 15, `G-F10.variance_ratio` 7/3 of the
  gap recipe's (N − 5)(N + 1) ÷ 35 at every count (oracle: (k + 2)/3 by
  enumeration for handfuls of 1 to 5, equal only for one box or N = k). The
  voice never says "about fourteen": 14.35 is too close to the rounding
  edge to round both ways truthfully.
- "Beats" (10_10) means by both widths at 71; chapter 13 states the general
  result. No card: the table is the chapter's exact statement, held to be
  read.
- Prediction (10_5): the tempting answer is "somewhere near the gap
  recipe's 71", since both recipes are unbiased. The countdown shows the
  handful and the recipe only; the average 14.8, the 28.6 and its place on
  the line come with the answer.
- Pauses: a thought pause after "Twice the average, minus one." as the
  recipe lands; a beat after "an average of thirty-seven." while its purple
  mark lands; a thought pause after "It's unbiased too."; a question pause
  after "So which recipe should we trust?"; a reveal pause after
  "Twenty-eight point six."; a turn after "That's no freak."; a beat after
  "The gap recipe's sixty-seven bars come back." while the green bars rise;
  a beat after "Both balance at seventy-one." and after "But the purple bars
  are lower and wider." while the halos stand; a thought pause after "beats
  the recipe that uses all five." before the chapter's question. The
  zoom-out runs inside 10_8's first sentence, before "exactly seventy-one
  boxes" (a director pause is at most 1.4 s, the zoom 2.4 s with the dots
  evening out; the cue is a draft cue until the line is measured); the
  inspection holds after 10_9 and 10_10 are scene holds.

## 11_the_largest_says_it_all: The largest number says it all (~143 s)

The friends' line again, the boxes on their ticks and the badge. Then only
the box at 60 keeps its number, and the other four are any four below it.
Under the line, the tiny company's pairs as a triangle of dots, one dot per
pair, its column the pair's larger number and its height the smaller (2 to 5
across, 1 to 4 up); then the same for companies of 6 and 8 boxes beside it,
bottom rows level, so the column topped by 5 is four dots high in all three.
Then a table under the line for three counts, 60, 71 and 1,000. The pairs
are listed whole (small cases); nothing at the scale of 60 is listed. Every
number is `G-F1`, `G-F3`, `G-F5`, `G-F8`, `G-F10` or `G-F11`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | Back to the friends' own five boxes. Suppose all you were told is that the largest number is sixty. Then the other four could be any four different numbers below sixty. | The line opens with the boxes on their ticks and the badge; on "all you were told" the four smaller boxes, their numbers and their ticks fade and their gold dots turn faint, the box at 60 stays, and "largest: 60" comes in on the panel's top line (`G-F1.largest`); on "any four different numbers below sixty" a dim span under the line from 1 to 59 with "any 4 of 1 to 59" under it (`G-F10.others`) | number line, highlight | |
| 11_2 | Try it on pairs. In the tiny company of five boxes, four of its ten pairs have five as their larger number, and the smaller one is one, two, three or four. A fair draw makes those four pairs equally likely, so each is one time in four. | On "Try it on pairs" the span and its words go; under the line at the left, ten dim dots land in a triangle, one per pair of the tiny company, each in the column of its larger number and at the height of its smaller, with "5 boxes: 10 pairs" under it (`G-F5.boxes`, `G-F5.pairs`); on "four of its ten pairs" the four dots in the column of 5 give way to gold ones, with a gold "5" over that column and "topped by 5: 4 of 10" under the caption (`G-F11.pair_top`, `G-F11.pairs_topped`, `G-F11.pair_shares`); on "the smaller one is one" "1", "2", "3" and "4" in dim at the left of their rows (`G-F11.pair_smaller`); on "so each is one time in four" "each 1 in 4" on the caption's third line (`G-F11.pair_each`) | grid, lagged reveal, replacement | |
| 11_3 | Now let the company make six boxes, or eight. There are more pairs, fifteen and then twenty-eight, so a larger number of five gets rarer. But the pairs topped by five are still the same four, and each is still one time in four. | On "six boxes, or eight" two more triangles land beside the first, fifteen and twenty-eight dim dots on the same rows, with "6 boxes: 15 pairs" and "8 boxes: 28 pairs" under them (`G-F11.pair_more`, `G-F11.pair_more_pairs`); on "a larger number of five gets rarer" "topped by 5: 4 of 15" and "topped by 5: 4 of 28" under the two new captions (`G-F11.pair_shares`); on "the pairs topped by five" the two new columns of 5 give way to gold dots, with a gold "5" over each, and a gold halo round each of the three gold columns, together; on "each is still one time in four" "each 1 in 4" on the two new captions' third lines (`G-F11.pair_each`, `G-F11.pair_same_for_every_company`) | grid, lagged reveal, halo, coordinated views | |
| 11_4 | It's the same with sixty. Whatever the count, from sixty boxes up, a handful topped by sixty is sixty plus any four numbers below it. That's over four hundred and fifty thousand groups of four, all equally likely, for every count. Only the chance that sixty is the largest changes. With exactly sixty boxes it's one in twelve, with seventy-one about one in twenty-nine, and with a thousand about one in eighteen million. | On "It's the same with sixty" the triangles and their words go; on "over four hundred and fifty thousand groups" "455,126 groups of 4 below 60" on the panel's second line (`G-F11.others_sets`); on "Only the chance that sixty is the largest changes" a table under the line, heads "count", "largest is 60" and "each group of 4"; a row lands as each count is named: on "With exactly sixty boxes" 60, 1 in 12, 1 in 455,126; on "with seventy-one" 71, ≈ 1 in 29, 1 in 455,126; on "with a thousand" 1,000, ≈ 1 in 18 million, 1 in 455,126 (`G-F11.counts`, `G-F11.top_chance.60`, `G-F11.top_chance.71.one_in`, `G-F11.top_chance.1000.millions`, `G-F11.each_group.60`, `.71`, `.1000`); held to be inspected | table, lagged reveal | hold |
| 11_5 | ⟨predict⟩ So once you know the largest number, how much can the other four tell you about the count? | The table stands; the countdown | prediction pause, think time, table | |
| 11_6 | Nothing. Given the largest, their pattern is the same for every count, so it can't favour any count over another. Statisticians say the largest number is sufficient. It holds everything a fair handful can tell you about the count. | The countdown goes; on "their pattern is the same for every count" an ink halo round the table's last column; on "sufficient" "the largest number: sufficient" on the panel's third line; card G rises and stays until it can be read | table, halo, card | hold |
| 11_7 | That's why the gap recipe needs only the largest number. Whatever the other four are, the gaps below sixty hold fifty-five numbers, sixty minus five. Ours split them fairly evenly, and the strange handful's put nearly all of them in a single gap. | The table, card G and the panel's lower lines go; on "That's why the gap recipe" the four smaller boxes come back on their numbers with their ticks, their dots turn gold again, and the blue gap arcs and counts come in under the boxes (`G-F3.gaps`); on "the gaps below sixty hold" "13 + 12 + 5 + 17 + 8 = 55 = 60 − 5" on the panel's second line, the 55 blue (`G-F3.unseen`); on "the strange handful's put" "2, 3, 4, 5: 1 + 0 + 0 + 0 + 54 = 55" on its third line (`G-F11.silly_gaps`, `G-F11.silly_unseen`) | number line, equation, callback | |
| 11_8 | All of this needs the handful to be fair. If bigger numbers were likelier to be picked, the other four could shift with the count and carry a clue of their own. With a fair handful they say nothing more about the count, but they can still warn you when a draw doesn't look fair. | On "needs the handful to be fair" an ink halo round the badge; on "they can still warn you" an ink halo round the four smaller boxes | halo | |

### Notes for 11

- Given the largest number m, a fair handful of k from 1 to N is m and k − 1
  numbers below it, and every such group is equally likely: chance
  1/C(m − 1, k − 1) for every N ≥ m. For the friends, `G-F11.others_sets`
  455,126 = C(59, 4) groups ("over four hundred and fifty thousand",
  `G-F11.others_sets_over` 450,000), each `G-F11.each_group.60`,
  `.71` and `.1000` 1/455,126, computed at each count as one handful's
  chance over the chance the largest is 60 (oracle: the same for every
  count from 60 to 300).
- The pairs (11_2, 11_3): companies `G-F11.pair_companies` 5, 6 and 8,
  `G-F5.pairs` 10 and `G-F11.pair_more_pairs` 15 and 28; the pairs topped
  by `G-F11.pair_top` 5 are the same `G-F11.pairs_topped` 4 in every company
  (`G-F11.pair_same_for_every_company`), smaller numbers
  `G-F11.pair_smaller` 1 to 4, each `G-F11.pair_each` 1/4 given the larger;
  their share of all pairs falls, `G-F11.pair_shares` 2/5, 4/15 and 1/7.
  The scene lists these pairs whole and refuses a triangle whose column of 5
  differs between companies.
- The counts (11_4): the chance the largest is 60 is C(59, 4)/C(N, 5):
  `G-F11.top_chance.60` 1/12 at the edge N = 60 (where it is 5/60, oracle
  check), `G-F11.top_chance.71` 65018/1859987 ≈ 1 in 28.6
  (`G-F11.top_chance.71.one_in` 29, the same number as 08's exact
  estimate), `G-F11.top_chance.1000` ≈ 1 in 18,127,488
  (`G-F11.top_chance.1000.millions` 18); it falls at every count from 60 to
  300 (oracle). The oracle computes these as sequential draws, never as
  binomials.
- Card G (11_6), from the facts, the fairness first: "fair handful from
  any N ≥ 60, given largest 60: each group of 4 below 60 is the others with
  chance 1/455,126. Only the chance of largest 60 depends on N: the largest
  is sufficient for N." Three lines at every width the lower third narrows
  to, with no line break inside a formula; 186 characters, 12.4 s to read
  at fifteen a second.
- The callback (11_7): whatever the other four are, the gaps below 60 hold
  `G-F3.unseen` 55 = 60 − 5 numbers (oracle: every one of the 455,126
  groups), the friends' `G-F3.gaps` 13, 12, 5, 17 and 8 and the strange
  handful's `G-F11.silly_gaps` 1, 0, 0, 0 and 54 (`G-F11.silly_unseen` 55).
  So the gap recipe's estimate is a function of 60 alone.
- Fairness (11_8): the whole chapter rests on every handful being equally
  likely. The oracle's negative control draws each box in turn with chance
  proportional to its number: then the others' chances given the largest
  are unequal and change with the count (handfuls of 2 with largest 4, and
  of 3 with largest 6, counts up to three more), while fair draws of the
  same sizes give equal chances that never change. "Could shift" is said for
  that reason, not "would". The warning job is chapter 14's: a first gap of
  157 gives an unfair shelf away.
- Prediction (11_5): the tempting answer is "something", since the four
  numbers spread more or less widely. The countdown shows the table, whose
  last column is the evidence; the word "sufficient" and card G come with
  the answer.
- Pauses: a thought pause after "the largest number is sixty." while the
  four boxes fade; a turn after "Try it on pairs."; a beat after
  "Now let the company make six boxes, or eight." while the triangles land;
  a turn after "It's the same with sixty."; a thought pause after "Only the
  chance that sixty is the largest changes." before its numbers; a reveal
  pause after "Nothing."; a beat after "the largest number is sufficient.";
  a thought pause after "needs only the largest number." while the arcs
  return; a thought pause after "All of this needs the handful to be
  fair.". The inspection holds after 11_4 and 11_6 are scene holds.

## 12_averaging_the_wobble: Averaging away the wobble (~245 s)

The tiny table again (chapter 05's ten pairs from five boxes, one row each,
the seen two squares gold), smaller, on the left: each row's doubling
estimate in purple in a column beside its squares, and a small plot right
of it, a purple dot at the estimate. The rows then move, each one whole
(squares, estimate and dot together), into four groups by their larger
number, and every pair keeps its own row. A green column beside the purple
one takes each group's average, with a green dot on each row's plot under
the purple one. Later, an upright dim line at the count 5 through every
row's plot (drawn only after the rows have stopped moving), and each miss as
an arrow from it, purple over the row's centre, green under it. The panel
is on the right, the badge at the top right. Highlights are halos and colour
changes are replacements; no typeset text is indicated, recoloured or
scaled in place. Every number is `G-F5`, `G-F10`, `G-F11` or `G-F12`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Back to the tiny company of five boxes, where any two are as likely to be seen as any other two. Here are its ten pairs again, now with the doubling recipe's estimates. For a pair, doubling the average just gives the total, so the recipe adds the two numbers and takes away one. | The badge at the top right; on "Here are its ten pairs again" the column numbers 1 to 5 and the ten rows land one after another, the seen squares gold (`G-F5.pair_list`); on "the doubling recipe's estimates" the head "doubling" and each row's purple estimate, 2, 3, 4, 5, 4, 5, 6, 6, 7, 8, with its purple dot on the row's plot (`G-F12.doubling_estimates`); on "doubling the average just gives the total" "doubling: 2 × average − 1" on the panel's first line, "doubling" purple (`G-F10.recipe`); on "adds the two numbers" "for a pair: total − 1" under it (`G-F12.pair_doubling`) | table, lagged reveal, plot, equation | |
| 12_2 | They average five, so doubling is unbiased here too. But they wobble, from two all the way up to eight. | On "They average five" "average: 5" on the panel's third line (`G-F12.doubling_average`); on "from two all the way up to eight" a purple halo round the estimate 2 and another round the 8, together (`G-F12.doubling_range`) | equation, halo | |
| 12_3 | Where does the wobble come from? The last chapter showed that once you know the larger number, the smaller carries no news about the count. Doubling listens to it anyway, so the four pairs topped by five get four different estimates. | On "so the four pairs topped by five" an ink halo round each of the four rows topped by 5, with their estimates 5, 6, 7 and 8, together (`G-F11.pairs_topped`, `G-F12.ends_in_5`) | table, halo | |
| 12_4 | So sort the pairs by their larger number. Each pair keeps its squares and its estimate, and moves next to the pairs that share its larger number. That makes four groups, topped by two, three, four and five, with one, two, three and four pairs. | On "So sort the pairs" the panel's second and third lines go and the ten rows move, each whole, into group order, a little space between the groups; on "That makes four groups" "topped by 2", "topped by 3", "topped by 4" and "topped by 5" come in one after another right of the plots, each at its group's middle (`G-F12.groups`, `G-F12.group_tops`, `G-F12.group_sizes`) | table, morph, lagged reveal | |
| 12_5 | Look at the biggest group. Doubling says five, six, seven and eight for its pairs. ⟨predict⟩ What's their average? | On "Look at the biggest group" an ink halo round the four rows topped by 5; on "five, six, seven and eight" a purple halo round their four estimates (`G-F12.ends_in_5`); the countdown; nothing shows the average | prediction pause, think time, table | |
| 12_6 | Six and a half. That's what the gap recipe says for every pair topped by five. The other groups work the same way. Three and four average three and a half. Four, five and six average five. And the lone pair topped by two says two. | The countdown goes; on "Six and a half" the green head "averaged", 6.5 in green beside each of the four estimates topped by 5 with a green dot on each of their plots, and "(5 + 6 + 7 + 8) ÷ 4 = 6.5" on the panel's second line, the 6.5 green (`G-F12.ends_in_5_average_decimal`); on "every pair topped by five" a green halo round the four green values; on "Three and four average" 3.5 beside both pairs topped by 3, with their green dots, and "(3 + 4) ÷ 2 = 3.5" on the third line (`G-F12.ends_in_3`, `G-F12.ends_in_3_average`); on "Four, five and six average five" 5 beside the three topped by 4 and "(4 + 5 + 6) ÷ 3 = 5" on the fourth line (`G-F12.ends_in_4`, `G-F12.ends_in_4_average`); on "the lone pair topped by two" 2 beside it, with its green dot (`G-F12.ends_in_2`) | table, equation, halo, plot | |
| 12_7 | So every group's average is exactly the gap recipe's estimate. Averaging inside the groups turns the doubling recipe into the gap recipe. | On "the gap recipe's estimate" the panel's second to fourth lines go, "estimate = largest + (largest − 2) ÷ 2" comes in on the second, "estimate" green and "largest" gold (chapter 04's formula for a pair), and a green halo round the green column (`G-F12.group_averages_shown`, `G-F12.grouped_is_gap_recipe`); on "turns the doubling recipe into the gap recipe" "averaged doubling = gap recipe" on the third line, "doubling" purple and "gap recipe" green | formula, halo, table | |
| 12_8 | And a recipe can use these averages without knowing the count. With a fair draw, once the larger number is known, every smaller number below it is equally likely, whatever the count. So in a company of six boxes, or eight, the pairs topped by five still average six and a half. | On "With a fair draw" an ink halo round the badge; on "a company of six boxes, or eight" "6 boxes: 5, 6, 7, 8 average 6.5" and "8 boxes: 5, 6, 7, 8 average 6.5" on the panel's fourth and fifth lines (`G-F11.pair_more`, `G-F12.group_average_same_for_every_count`) | equation, halo | |
| 12_9 | Is the averaged recipe still unbiased? The groups aren't equally likely. Each pair is a chance in ten, so a group's chance is its number of pairs in ten, and the biggest group's is four in ten. | On "Is the averaged recipe still unbiased?" the panel's lines go; on "The groups aren't equally likely" an ink halo round the lone pair topped by 2 and another round the four topped by 5, together; on "a group's chance is its number of pairs in ten" "1 in 10", "2 in 10", "3 in 10" and "4 in 10" right of the group labels, each at its group's middle (`G-F5.pairs`, `G-F12.group_sizes`); on "four in ten" an ink halo round "4 in 10" (`G-F12.top_group_chance`) | table, halo | |
| 12_10 | Weight each group's average by its chance, and the ten pairs' average comes back, exactly five. Give the four groups equal weight instead, and you'd get four and a quarter, because the lone pair would count as much as the biggest group. | On "Weight each group's average by its chance" each chance line goes on to "× 2 = 0.2", "× 3.5 = 0.7", "× 5 = 1.5" and "× 6.5 = 2.6", the averages green; on "comes back, exactly five" "0.2 + 0.7 + 1.5 + 2.6 = 5" under the table on the right (`G-F12.weighted_average`); on "Give the four groups equal weight" "(2 + 3.5 + 5 + 6.5) ÷ 4 = 4.25" under it (`G-F12.equal_weight_average_shown`); on "the lone pair would count as much" an ink halo round the lone pair and another round the biggest group, together | equation, halo, table | |
| 12_11 | Now the misses. Draw each estimate's miss as an arrow from five, the true count. In the group topped by four, doubling's arrows are minus one, zero and plus one. They cancel, so the group's average misses by nothing at all. | On "Now the misses" the chance lines and sums go; on "an arrow from five" a dim upright line at 5 through every row's plot, "count = 5" over it, and a purple arrow from it to each purple dot (none where the estimate is 5), one after another (`G-F5.boxes`); on "the group topped by four" an ink halo round its three rows; on "minus one, zero and plus one" "−1, 0, +1" in purple right of the label "topped by 4" (`G-F12.cancel_misses`); on "misses by nothing at all" "(−1 + 0 + 1) ÷ 3 = 0" under it, the 0 green: the group's green dots sit on the line | plot, table, lagged reveal, halo, equation | |
| 12_12 | In the group topped by five, the arrows are zero, one, two and three, all on the same side. Nothing cancels, so the average misses by one and a half, the average length of the arrows. | On "the group topped by five" an ink halo round its four rows; on "zero, one, two and three" "0, +1, +2, +3" in purple right of "topped by 5" (`G-F12.same_side_misses`); on "misses by one and a half" a green arrow from the line to each green dot of the group, all 1.5 long, and "(0 + 1 + 2 + 3) ÷ 4 = 1.5" under the purple misses, the 1.5 green (`G-F12.same_side_average_miss`) | plot, halo, equation | |
| 12_13 | Add it up over all ten pairs. Ignoring direction, doubling misses by one point four boxes on average, and the gap recipe by one point two. | On "Add it up over all ten pairs" the miss lines go and the green arrows of the groups topped by 2 and 3 come in (the group topped by 4 has none); on "Ignoring direction" a table on the panel, heads "doubling" in purple and "gap recipe" in green; on "doubling misses by one point four" the row "average miss" and 1.4 under "doubling" (`G-F12.doubling_miss`); on "the gap recipe by one point two" 1.2 under "gap recipe" (`G-F12.grouped_miss`) | table, plot | |
| 12_14 | That's a rule, not luck. The arrow of an average is never longer than the average length of the arrows. It's shorter when they point both ways, and just as long when they all point the same way. So averaging inside the groups can never make the average miss bigger. | On "The arrow of an average" "an average's arrow is never longer" and "than the arrows' average length" on two lines low on the panel; on "It's shorter when they point both ways" a purple halo round the arrows of the group topped by 4; on "just as long when they all point the same way" a purple halo round the arrows of the group topped by 5 | equation, halo, plot | |
| 12_15 | Squared misses tell the same story with different numbers. Doubling's squared misses average three, the gap recipe's two and a quarter, so the standard deviation falls from about one point seven to exactly one point five. For squares, the drop, three quarters, is exactly the wobble inside the groups. | On "squared misses average three" the table's row "average squared miss", 3 and 2.25 (`G-F12.doubling_squared_miss`, `G-F12.grouped_squared_miss_shown`); on "so the standard deviation falls" the row "standard deviation", ≈ 1.7 and 1.5 (`G-F12.doubling_sd_about`, `G-F12.grouped_sd_decimal`); on "the drop, three quarters" "3 − 2.25 = 0.75: wobble inside groups" under the table (`G-F12.spread_inside`); held to be read | table, equation | hold |
| 12_16 | So averaging any recipe inside the groups of the largest number keeps its average and never makes its misses bigger, plain or squared. That's why the gap recipe beat the doubling recipe. | On "keeps its average and never makes its misses bigger" card H rises and stays until it can be read; the table stays | card, table | hold |

### Notes for 12

- The pairs and estimates: `G-F5.pair_list` in lexicographic order with
  `G-F12.doubling_estimates` 2, 3, 4, 5, 4, 5, 6, 6, 7, 8, which for a pair
  are the total less one (`G-F12.pair_doubling`, from `G-F10.recipe` 2 ×
  average − 1); they average `G-F12.doubling_average` 5 and run
  `G-F12.doubling_range` 2 to 8. The scene computes every estimate from its
  pair and refuses a table the facts contradict.
- The groups (12_4): `G-F12.groups` 4, topped by `G-F12.group_tops` 2 to 5,
  sizes `G-F12.group_sizes` 1 to 4 (a group of pairs topped by m has m − 1
  pairs). The rows move whole; nothing is redrawn, so each pair keeps its
  squares, estimate and dot through the regrouping. No stroke is on the
  screen while they move: the count line and the arrows come only in 12_11.
- The group averages (12_6): `G-F12.ends_in_2` 2; `G-F12.ends_in_3` 3, 4 →
  `G-F12.ends_in_3_average` 7/2; `G-F12.ends_in_4` 4, 5, 6 →
  `G-F12.ends_in_4_average` 5; `G-F12.ends_in_5` 5, 6, 7, 8 →
  `G-F12.ends_in_5_average` 13/2; together `G-F12.group_averages`, shown
  exactly as `G-F12.group_averages_shown` 2, 3.5, 5, 6.5, the gap recipe's
  m + (m − 2) ÷ 2 for each top m (`G-F12.grouped_is_gap_recipe`).
- "Without knowing the count" (12_8) is chapter 11's sufficiency: given the
  larger number, a fair draw makes every smaller one equally likely at every
  count, so each group's average is the same in every company from its top
  up (`G-F12.group_average_same_for_every_count`, checked in every company
  of pairs up to twelve boxes). The oracle's negative control: draws
  weighted by the numbers change a group's average with the count (pairs
  topped by 4, handfuls of three topped by 6).
- The weights (12_9, 12_10): each pair has chance 1/`G-F5.pairs`, so a
  group's chance is its size in ten, the biggest `G-F12.top_group_chance`
  2/5 ("four in ten"). Weighted by those chances the group averages give
  0.2 + 0.7 + 1.5 + 2.6 = `G-F12.weighted_average` 5, the count; equal
  weights give `G-F12.equal_weight_average` 17/4 (4.25). The oracle checks
  both for handfuls of 2 to 4 and every count to 12: weighted always gives
  the count, equal weights never do once the count exceeds the handful size
  (equal at N = k, where there is a single group).
- The arrows (12_11, 12_12): a miss is estimate − 5, so the group topped by
  `G-F12.cancel_group` 4 has `G-F12.cancel_misses` −1, 0, +1 (average 0,
  while their average length is 2/3), and the group topped by
  `G-F12.same_side_group` 5 has `G-F12.same_side_misses` 0, +1, +2, +3,
  average `G-F12.same_side_average_miss` 3/2, equal to their average length
  because none points the other way.
- The widths (12_13 to 12_15), exact over the ten pairs, two different
  widths kept apart: the average miss, ignoring direction, is
  `G-F12.doubling_miss` 1.4 for doubling and `G-F12.grouped_miss` 1.2 for the
  gap recipe; the average squared miss is `G-F12.doubling_squared_miss` 3
  against `G-F12.grouped_squared_miss` 9/4 (2.25), so the standard deviation
  (the square root of the average squared miss) is √3 ≈
  `G-F12.doubling_sd_about` 1.7 against exactly `G-F12.grouped_sd` 3/2
  (1.5). For squares the drop is exactly the average squared distance of
  doubling's estimates from their group's average, `G-F12.spread_inside`
  3/4; for plain misses there is no such identity, only the inequality.
- The rule (12_14) is the triangle inequality for averages: the length of
  an average of arrows is at most the average of their lengths, equal only
  when they all point one way (or are zero). Applied group by group with the
  groups' true chances it gives the average miss; the oracle checks it,
  plain and squared, for the doubling recipe and three times the smaller
  minus one, handfuls of 2 to 4 and every count to 12, and that plain equals
  only when every group's misses are one-sided.
- Card H (12_16), its fairness first: "fair handful, any recipe: averaged
  over each largest M, it becomes a recipe of M alone, free of N. At every
  N its average stays, and its average miss and squared miss never grow."
  Three lines at every width the lower third narrows to; 178 characters,
  11.9 s to read at fifteen a second, counted from its fade-in. It is the
  general statement after the ten pairs have shown it; "Rao–Blackwell" is
  the name, kept off the card.
- Prediction (12_5): the tempting answer is "about five, the count", since
  the recipe is unbiased. The countdown shows the group's four estimates;
  the 6.5 and the green column come with the answer.
- Pauses: a beat after "Here are its ten pairs again," while the rows land;
  a thought pause after "But they wobble," before the range; a question
  pause after "Where does the wobble come from?"; a turn after "So sort the
  pairs by their larger number." while the rows move; a reveal pause after
  "Six and a half."; a thought pause after "Averaging inside the groups
  turns the doubling recipe into the gap recipe."; a question pause after
  "Is the averaged recipe still unbiased?"; a beat after "comes back,
  exactly five."; a turn after "Now the misses."; a beat after "They
  cancel," and after "Nothing cancels,"; a thought pause after "That's a
  rule, not luck."; a beat after "the drop, three quarters," before what it
  is. The regroup in 12_4 runs inside its first sentence's turn; the
  inspection holds after 12_15 and 12_16 are scene holds.

## 13_nothing_does_better: Nothing unbiased does better (~257 s)

The panel on the right, the badge at the top left. On the left, a company's
pairs as a triangle of dots, as in chapter 11: a column per larger number,
2 to 5 across, gold over its foot, and a row per smaller number, 1 to 4 up,
dim left of the first column, each coming in with the column that brings its
row; a company of two boxes has a single dot, and each box more adds a
column on the right. Under each column a green bar, its height the value a recipe of
the larger number must give that column, the value over it: the staircase.
Then a rival recipe's values in ink beside the dots, and at the end a table
across the frame for the 71-box company. Highlights are halos and colour
changes are replacements; no typeset text is indicated, recoloured or
scaled in place. Every number is `G-F1`, `G-F5`, `G-F8`, `G-F12` or
`G-F13`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | Averaging turned the doubling recipe into the gap recipe, and shrank its misses. Could some cleverer unbiased recipe still beat the gap recipe? To find out, we need to know every recipe that's unbiased. | The badge; "averaged doubling = gap recipe" on the panel's first line, "doubling" purple and "gap recipe" green (chapter 12's line); on "every recipe that's unbiased" "every unbiased recipe: ?" on its second line | equation | |
| 13_2 | Unbiased has to hold at every count the company could have made, because we never know which count is true. A recipe that always says seventy-one is right on average if the count is seventy-one, and wrong on average at every other count. So it isn't unbiased. | On "Unbiased has to hold at every count" "unbiased: right on average at every count" on the panel's third line; on "A recipe that always says seventy-one" "always 71: right on average only at 71" on its fourth line (`G-F13.always`); on "So it isn't unbiased" an ink halo round that line | equation, halo | |
| 13_3 | Start with pairs, and recipes that look only at the larger number. The smallest company with a pair has two boxes, and its only pair is one and two. To average two, the recipe must say two when the larger number is two. | On "Start with pairs" the panel's lines go and "recipe of the larger number: ?" comes in on its first line; on "The smallest company with a pair" a single dot at column 2, row 1 lands with a gold "2" under its column and "2 boxes: 1 pair" under the bars' foot (`G-F13.smallest_company`, `G-F13.company_pairs`); on "To average two" "2 boxes: 1 × ? = 1 × 2" on the panel's second line, small; on "must say two when the larger number is two" it gives way to "2 boxes: 1 × 2 = 1 × 2", the solved 2 green, and a green bar rises under column 2 with "2" over it (`G-F13.forced_at_2`) | grid, equation, bars | |
| 13_4 | Three boxes make three pairs. The pair topped by two must still say two. The other two are topped by three, and all three estimates must average three, so those two must say three and a half. | On "Three boxes make three pairs" two dots land in a new column 3 with a gold "3" under it, and the caption gives way to "3 boxes: 3 pairs" (`G-F13.next_company`, `G-F13.pairs_from_3`); on "The pair topped by two must still say two" a green halo round the bar at 2; on "all three estimates must average three" "3 boxes: 1 × 2 + 2 × ? = 3 × 3" on the panel's third line (`G-F13.new_pairs.3`); on "those two must say three and a half" it gives way to the solved line, 3.5 green, and the bar under column 3 rises to 3.5 (`G-F13.forced_at_3_decimal`) | grid, equation, bars, halo | |
| 13_5 | Four boxes make six pairs, and three of them are topped by four. The other three already have their values. ⟨predict⟩ What must the recipe say for the new three? | On "Four boxes make six pairs" three dots land in a new column 4 with a gold "4", and the caption gives way to "4 boxes: 6 pairs" (`G-F13.predict_company`, `G-F13.pairs_from_4`, `G-F13.new_pairs.4`); on "The other three already have their values" a green halo round the bars at 2 and 3.5 and "4 boxes: 1 × 2 + 2 × 3.5 + 3 × ? = 6 × 4" on the panel's fourth line; the countdown, lower right of the panel; nothing shows the answer | prediction pause, think time, grid, equation | |
| 13_6 | Five. That's the only value that brings all six estimates to an average of four. Take the same step at five boxes, and its four new pairs are forced to six and a half. | The countdown goes; on "Five" the fourth line gives way to the solved line, 5 green, and the bar under column 4 rises to 5 (`G-F13.forced_at_4`); on "the only value" a green halo round that line; on "at five boxes" four dots land in a new column 5 with a gold "5", the caption gives way to "5 boxes: 10 pairs" and "5 boxes: 1 × 2 + 2 × 3.5 + 3 × 5 + 4 × ? = 10 × 5" comes in on the panel's fifth line (`G-F13.last_company`, `G-F13.new_pairs.5`); on "forced to six and a half" it gives way to the solved line, 6.5 green, and the bar under column 5 rises to 6.5 (`G-F13.forced_at_5_decimal`, `G-F13.forced_shown`) | grid, equation, bars | |
| 13_7 | The same step works at every count, for handfuls of any size. Going up by a box, the only new handfuls are the ones that contain the new top box, and it's always their largest number. They're never missing. Their share is the handful size over the count. So each step has exactly one unknown value, and the step fixes it. With five seen from exactly sixty boxes, that share is one in twelve, as we found before. | On "the only new handfuls" "new handfuls: the ones that contain N" on the panel under the company lines; on "it's always their largest number" "their largest number: always N" under it; on "Their share is the handful size over the count" "their share: k ÷ N, never 0"; on "exactly one unknown value" a green halo round the newest column's dots and bar; on "five seen from exactly sixty boxes" "5 seen, 60 boxes: 5 ÷ 60 = 1 in 12" (`G-F1.handful`, `G-F1.at_least`, `G-F13.share_at_60`, `G-F13.recurrence_proved`) | equation, halo, grid | |
| 13_8 | Each count fixes the next value, all the way up from the smallest company. So only a single recipe of the largest number is unbiased at every count, and it's the gap recipe. | On "Each count fixes the next value" a green halo round each bar in turn, left to right; on "it's the gap recipe" the four general lines go and "estimate = largest + (largest − k) ÷ k" comes in under the company lines, "estimate" green and "largest" gold (chapter 04's formula), and a green halo round the bars (`G-F13.forced_is_gap_recipe`) | bars, lagged reveal, formula, halo | |
| 13_9 | Now take any unbiased recipe at all, even a recipe that uses every number. For pairs, try three times the smaller number, minus one. Its estimates swing from two all the way to eleven, yet over the ten pairs they still average five. | On "Now take any unbiased recipe at all" the panel's lines go; on "three times the smaller number, minus one" "3 × smaller − 1" on the panel's first line, and beside each dot its value in ink, 2, 5, 8 or 11 by its row, one after another (`G-F13.smaller_recipe`, `G-F13.smaller_estimates`); on "from two all the way to eleven" an ink halo round the four 2s and another round the 11, together (`G-F13.smaller_range`); on "they still average five" "(2 + 2 + 2 + 2 + 5 + 5 + 5 + 8 + 8 + 11) ÷ 10 = 5" on the second line, small (`G-F13.smaller_average`) | grid, lagged reveal, halo, equation | |
| 13_10 | Average it inside the groups of the larger number. With a fair draw those averages don't depend on the count, so they make a recipe, and averaging keeps it unbiased at every count. So they have to be the staircase again, two, three and a half, five, and six and a half. | On "Average it inside the groups" "topped by 2: 2 ÷ 1 = 2", "topped by 3: (2 + 5) ÷ 2 = 3.5", "topped by 4: (2 + 5 + 8) ÷ 3 = 5" and "topped by 5: (2 + 5 + 8 + 11) ÷ 4 = 6.5" on the panel's third to sixth lines, one after another, the averages green (`G-F13.smaller_grouped`); on "With a fair draw" an ink halo round the badge; on "So they have to be the staircase again" a green halo round the bars | equation, lagged reveal, halo, bars | |
| 13_11 | The smaller-number recipe misses by two point four boxes on average, against the gap recipe's one point two. That's the whole argument. Averaging any unbiased recipe inside the groups gives the gap recipe, and never makes its misses bigger, plain or squared. So no unbiased recipe misses by less than the gap recipe. | On "The smaller-number recipe misses by two point four" the averaging lines go and "average miss: 3 × smaller − 1 2.4, gap recipe 1.2" on two lines, the 1.2 line green (`G-F13.smaller_miss_decimal`, `G-F12.grouped_miss`); on "Averaging any unbiased recipe" "any unbiased recipe" lower on the panel; on "inside the groups gives the gap recipe" a short down arrow under it with "average in the groups" beside it, and "the gap recipe" under the arrow in green; on "never makes its misses bigger" "misses never bigger, plain or squared" under that; on "So no unbiased recipe misses by less" a green halo round "the gap recipe" | equation, halo | |
| 13_12 | Best doesn't mean every rival does worse. With exactly as many boxes as the handful holds, there's just a single handful, so every unbiased recipe is exactly right. And a recipe that already uses only the largest number doesn't change when you average it. | On "Best doesn't mean every rival does worse" the panel's lower lines go; on "With exactly as many boxes as the handful holds" an ink halo round the column topped by 2 and its bar, and "2 boxes: every unbiased recipe says 2" low on the panel; on "doesn't change when you average it" "gap recipe, averaged: the gap recipe" under it | grid, halo, equation | |
| 13_13 | A recipe that accepts a little bias can sometimes miss by less. Take seven sixths of the largest number, minus one. For the friends' five boxes it says sixty-nine. Over every handful from seventy-one boxes it averages sixty-nine, two short of the count, yet its misses average about eight point seven boxes against the gap recipe's nine point two, and its squared misses are smaller too. | On "A recipe that accepts a little bias" everything but the badge goes; on "Take seven sixths of the largest number" a table across the frame, heads "gap recipe" in green and "7/6 × largest − 1" in ink (`G-F13.shrunk_recipe`); on "it says sixty-nine" the row "friends' five", 71 and 69 (`G-F3.estimate`, `G-F13.shrunk_friends`); on "it averages sixty-nine" the row "average, 71 boxes", 71 and 69 (`G-F8.average`, `G-F13.shrunk_average_71`); on "two short of the count" an ink halo round that 69 (`G-F13.shrunk_bias_71`); on "about eight point seven" the row "average miss", ≈ 9.2 and ≈ 8.7 (`G-F8.average_miss_decimal`, `G-F13.shrunk_miss_71_shown`); on "its squared misses are smaller too" the row "average squared miss", ≈ 135.8 and ≈ 132.3 (`G-F13.gap_squared_71_shown`, `G-F13.shrunk_squared_71_shown`) | table, halo | |
| 13_14 | But it doesn't win everywhere. With exactly five boxes, the gap recipe is always right and seven sixths says four and five sixths, and its squared misses are bigger at every count from five to ten. Among recipes that are right on average at every count, though, none misses by less than the gap recipe, as long as the handful is fair. | On "With exactly five boxes" the row "every handful, 5 boxes", 5 and ≈ 4.8 (`G-F13.edge_count`, `G-F13.gap_at_edge`, `G-F13.shrunk_at_edge_shown`); on "bigger at every count from five to ten" "7/6: squared misses bigger at counts 5 to 10" under the table (`G-F13.shrunk_loses_squared_at`); on "Among recipes that are right on average" card I rises; on "as long as the handful is fair" an ink halo round the badge; card I stays until it can be read | table, card, halo | hold |

### Notes for 13

- Unbiased at every count (13_2): a recipe is unbiased when its average
  over every fair handful equals the count for every count N ≥ k the company
  could have made, not just for the true one. "Always 71"
  (`G-F13.always` 71) averages 71 at every count, right only at N = 71
  (oracle: enumerated at counts 5 to 12 and at 71).
- The staircase for pairs (13_3 to 13_6), companies `G-F13.companies` 2 to
  5 with `G-F13.company_pairs` 1, 3, 6 and 10 pairs, of which
  `G-F13.new_pairs.3`, `.4` and `.5` 2, 3 and 4 are new (topped by the new
  box): 1 × g(2) = 1 × 2; 1 × 2 + 2 × g(3) = 3 × 3; 1 × 2 + 2 × 3.5 + 3 ×
  g(4) = 6 × 4; 1 × 2 + 2 × 3.5 + 3 × 5 + 4 × g(5) = 10 × 5. So
  `G-F13.forced` 2, 7/2, 5, 13/2 (shown `G-F13.forced_shown` 2, 3.5, 5,
  6.5), the gap recipe's values (`G-F13.forced_is_gap_recipe`, k = 1 to 5).
  The scene solves each line exactly and refuses a staircase the facts
  contradict.
- The general step (13_7), proved, not enumerated (`G-F13.recurrence_proved`):
  the handfuls of k from 1 to N are those from 1 to N − 1 and the
  C(N − 1, k − 1) that contain N, whose largest number is N. Unbiased at
  N − 1 and at N says C(N − 1, k − 1) g(N) = N C(N, k) − (N − 1) C(N − 1, k),
  one equation with one unknown, whose coefficient is never zero: the share
  of new handfuls is k ÷ N. With g(k) = k forced at N = k, induction fixes
  every value, and the identities k C(N, k) = N C(N − 1, k − 1) and
  k C(N − 1, k) = (N − k) C(N − 1, k − 1) turn the equation into
  g(N) = N + (N − k) ÷ k. The oracle checks those identities as polynomials
  in N for k = 1 to 12 and the last step as a polynomial in N and k; its
  negative control refuses a wrong final identity. With 5 seen from 60
  boxes the share is 5 ÷ 60 = `G-F13.share_at_60` 1/12, chapter 11's
  `G-F11.top_chance.60`.
- The rival (13_9 to 13_11): `G-F13.smaller_recipe` 3 × smaller − 1, the
  smaller number's own unbiased recipe for pairs (the smaller averages
  (N + 1) ÷ 3); over the tiny company's pairs `G-F13.smaller_estimates` 2,
  2, 2, 2, 5, 5, 5, 8, 8, 11, from `G-F13.smaller_range` 2 to 11, average
  `G-F13.smaller_average` 5; its group averages `G-F13.smaller_grouped` are
  the staircase; it misses by `G-F13.smaller_miss` 12/5 (2.4) on average,
  squared `G-F13.smaller_squared_miss` 9. The oracle checks it unbiased, its
  group averages the staircase, and its misses at least the gap recipe's,
  plain and squared, for handfuls of 2 to 5 and every count to 13.
- 13_11 is Lehmann–Scheffé (the name kept off the card): averaging an
  unbiased recipe inside the groups gives an unbiased recipe of the largest
  number (12's sufficiency), which can only be the gap recipe, and 12's rule
  says the misses never grow. "No unbiased recipe misses by less" is the
  claim; not that every rival misses by more.
- Ties (13_12): at N = k every handful is 1 to k, so every unbiased recipe
  says k and misses by nothing; a recipe of the largest number is its own
  group average. The oracle's negative controls: a staircase bumped at one
  value is biased at exactly the counts from that value up; and a recipe
  that sees only min(largest, 4), forced to 2, 7/2 and 5 by the counts 2 to
  4, has no new value to choose at five boxes and averages 22/5 there, the
  step with a zero coefficient.
- The biased recipe (13_13, 13_14): `G-F13.shrunk_recipe` 7/6 × largest − 1
  (`G-F13.shrunk_factor` 7/6) says `G-F13.shrunk_friends` 69 for the
  friends; at 71 boxes it averages `G-F13.shrunk_average_71` 69
  (`G-F13.shrunk_bias_71` −2; its bias is −(N + 1) ÷ 36 at every count,
  checked from 5 to 300), misses by `G-F13.shrunk_miss_71` ≈ 8.71
  (`G-F13.shrunk_miss_71_shown` 8.7) against `G-F8.average_miss` ≈ 9.23, and
  its average squared miss is `G-F13.shrunk_squared_71` 397/3 ≈ 132.3
  against 4752/35 ≈ 135.8 (`G-F13.gap_squared_71_shown`), all exact over
  every handful by the counts of handfuls by largest number. It does not
  dominate: at `G-F13.edge_count` 5 boxes it always says
  `G-F13.shrunk_at_edge` 29/6 (≈ 4.8) where the gap recipe says
  `G-F13.gap_at_edge` 5. For counts 5 to 300 the facts list where it does
  not miss less: squared at `G-F13.shrunk_loses_squared_at` 5 to 10, each
  strictly bigger (no ties, so the voice's "bigger" holds), and plain at
  `G-F13.shrunk_loses_plain_at` 5 and 11, bigger at 5 and an exact tie at
  11 (12/11 each), so the picture says nothing about plain losses.
  "Sometimes" is said for that reason.
- Card I (13_14), its fairness first: "fair handful: a recipe unbiased at
  every N ≥ k, averaged over each largest M, is the only unbiased recipe of
  M, M + (M − k) ÷ k. So none has a smaller average miss or squared miss at
  any N." Three lines at every width the lower third narrows to; 189
  characters, 12.6 s to read at fifteen a second. Its "≥" is a glyph to
  qualify (card G has the same sign).
- Prediction (13_5): the tempting answer is "four, the count", or "four and
  a half". The countdown shows the equation with its "?", the bars at 2 and
  3.5 and the three new dots; the 5 comes with the answer.
- Pauses: a question pause after "Could some cleverer unbiased recipe still
  beat the gap recipe?"; a thought pause after "because we never know which
  count is true."; a turn after "So it isn't unbiased."; a beat after "and
  its only pair is one and two."; a thought pause after "The pair topped by
  two must still say two."; a reveal pause after "Five."; a turn after "The
  same step works at every count, for handfuls of any size."; a thought
  pause after "They're never missing."; a beat after "and the step fixes
  it."; a thought pause after "all the way up from the smallest company.";
  a beat after "That's the whole argument."; a turn after "Best doesn't mean
  every rival does worse."; a beat after "two short of the count,"; a
  thought pause after "But it doesn't win everywhere.". The inspection
  hold after 13_14 is a scene hold.

## 14_an_unfair_draw: When the handful isn't fair (~80 s)

The line, extended to 200; the five friends' boxes on their ticks; red for
the pool a handful actually came from. The badge, for the first time,
crossed out.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | Everything so far rested on a fair handful. Now break it, with the same five boxes. Suppose all five friends shop at one store, and it only ever got the first seventy-one boxes. | The badge gets a red stroke through it; the line runs to 200; boxes 1 to 71 get a red outline labelled "one shop" around the friends' five | badge, highlight | |
| 14_2 | Say the company really made two hundred. The friends' numbers look just as innocent as before, and the recipe still says seventy-one. | A hero marker at 200; the five boxes, arcs and green 71 are exactly chapter 03's picture | callback, highlight | |
| 14_3 | Averaged over every handful that shop could sell, the recipe gives exactly seventy-one. It measured the shop, not the company, and it's short by a hundred and twenty-nine. | Exact weighted distribution of shop-handful estimates, with mean 71; a red bracket from 71 to 200 labelled "129 never on sale here" | lagged reveal, bars | |
| 14_4 | It can miss the other way too. Suppose only boxes from a hundred and fifty-one up are still on shelves. ⟨predict⟩ Will the recipe guess too low, or too high? | The red pool moves to 151–200; five boxes land on ticks at 158, 166, 173, 189, 200; the countdown | prediction pause, think time | |
| 14_5 | Too high. This handful's estimate is two hundred and thirty-nine. The gap below the smallest number holds a hundred and fifty-seven boxes, and the recipe treats it like any other gap. | A blue arc from 1 to 157 labelled "157" beside arcs of 7, 6, 15 and 10; the panel: 200 + 195/5 = 239; a dim "average over this shelf: 230" | equation, highlight | |
| 14_6 | That gap is the clue. In a fair draw from two hundred boxes, all five numbers above a hundred and fifty would happen less than once in a thousand. | The 157 arc flashes red; "fair draw: < 1 in 1,000" in the panel | highlight | |
| 14_7 | So the other four numbers earn their keep after all. Once the largest is known, they add nothing to our best unbiased recipe under the fair model. But they can warn us that the draw wasn't fair. An unlikely handful is a warning, not proof. | Callback to 11: the four smaller boxes brighten, the giant first gap among them | highlight, callback | |

### Notes for 14

- The shop: `G-F14.shop_boxes` 71, `G-F14.made` 200, `G-F14.shop_handful` the
  friends' own 14, 27, 33, 51, 60 (a fair handful from the shop), so
  `G-F14.shop_estimate` 71 and `G-F14.shop_average` 71 (every handful of
  1 to 71, the enumeration of chapter 08), `G-F14.shop_short_by` 129. The
  oracle's negative control requires this average to be 71, not 200, and a
  second shop of 40 boxes to average 40.
- The late shelf: `G-F14.late_estimate` 239, `G-F14.late_first_gap` 157,
  `G-F14.late_other_gaps` 7,6,15,10, `G-F14.late_average` 230 (all 2,118,760
  handfuls from the 50 boxes 151 to 200, enumerated);
  `G-F14.fair_all_above` 1081/1293699 ≈ 0.00084 (< 1/1000).
- Card J: "Precisely: five drawn fairly from boxes a+1 … a+P, the recipe averages
  P + 6a/5. It estimates the pool it was drawn from, shifted by how far the
  pool starts from 1."
- Using the friends' own boxes is the owner's "show the biased-shop failure
  using the same objects": nothing in the handful changes, only the bin it
  came from. Both shops are illustrative scenarios, not claims about how any
  product is sold.

- Root implementation: the same red pool outline moves from 1–71 to 151–200; the original friends stay unchanged for the shop comparison. The replacement handful changes only boxes, connectors and selected-dot roles. Bars weight every possible maximum by its exact number of handfuls. Prediction waits for the full invitation; card J has a complete reading hold. A rare event is diagnostic evidence, not proof that the sampling model is false.

## 15_other_assumptions: Other assumptions (~100 s)

The same observed five boxes, followed by an explicitly separate ten-box delivery experiment.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | Another assumption was that numbering starts at one. A company might begin at an unknown number instead. | Same five boxes remain on their number line; outside dots and the known first label disappear. | number line |  |
| 15_2 | Then the gap below the smallest is hidden too. Two gaps are unknown, one at each end, and the four inner gaps are the only clues. | Dashed unknown intervals at both ends; four computed inner arcs and counts. | highlight |  |
| 15_3 | Between fourteen and sixty, the inner gaps hold forty-two boxes, ten and a half per gap. And fourteen to sixty spans forty-seven numbers. | 12 + 5 + 17 + 8 = 42, 42 / 4 = 10.5; inclusive bracket of 47 numbers. | equation |  |
| 15_4 | ⟨predict⟩ Add one average inner gap at each end. What's the estimate now? | Add two endpoint brackets, each 10.5 on the same scale; countdown. | prediction pause, think time |  |
| 15_5 | Sixty-eight. It's the same gap idea with two hidden gaps instead of one, and in a fair draw it's unbiased too. That's a count of boxes, not the last serial number. | 47 + 2 × 10.5 = 68 boxes; exact formula card, conditional on fair draws without replacement and consecutive numbering. | equation card | hold |
| 15_6 | One last trap. Every box having the same chance isn't enough. Every handful of five must be equally likely. | New small experiment; explicit equal-handful rule. | equation card |  |
| 15_7 | Return to numbering from one. Suppose ten boxes were made. A coin flip chooses your delivery: boxes one to five, or boxes six to ten. | Ten fixed numbered boxes. The same selection outline fades out, jumps while hidden, then fades in around the second complete delivery; it returns the same way in 15_8. No intermediate mixed handful is selected. Each delivery probability 1/2; transition is one frame shorter at 15 fps, with later absolute cue starts unchanged. | controlled change |  |
| 15_8 | Every box has a one in two chance of reaching you. But those chances are tied together: you always receive an entire delivery, never a mixed handful. | All ten boxes display 1/2. Outline returns to first half; mixed handfuls cannot occur. | highlight | hold |
| 15_9 | Our original, starts-at-one recipe gives estimates five or eleven. Those average eight, not ten. Equal chances for individual boxes did not make the draw fair. | Both exact original-recipe calculations; (5 + 11) / 2 = 8, actual count 10. Final condition and deliberate reading hold. | equation, callback | hold |

### Notes for 15

- `G-F15.inner_unseen` 42, `G-F15.inner_average` 21/2 (10.5),
  `G-F15.span_numbers` 47, `G-F15.spread_estimate` 68,
  `G-F15.spread_unbiased` true (first numbers 1, 7 and 10,001; counts 5 to 15,
  enumerated); `G-F15.deliveries_each_box` 1/2, `G-F15.deliveries_average` 8,
  `G-F15.deliveries_made` 10.
- Card K retains the implemented assumptions, spread recipe and warning that
  it estimates the count, not the last serial number. Source: Clark, Gonye
  and Miller (2021), §3, especially §3.2, pp. 7–10 of the
  [author-hosted manuscript](https://web.williams.edu/Mathematics/sjmiller/public_html/math/papers/GTPv10.pdf).
  Root checked that section on 2026-10-26. F7 disposition: the bibliographic
  source line is intentionally in these notes and `brief.md`, not on the card;
  the displayed assumptions/formula/count warning and its reading hold stay intact.
- 15_7 is the negative control the oracle requires to fail: equal per-box
  inclusion without equally likely handfuls. It is why chapter 02 defined the
  fair handful by the bin, any five as likely as any other five, and not by
  each box's chance.


Root implementation clarification (2026-10-25): the counterexample returns explicitly to the original starts-at-one recipe. The unknown-start estimator counts boxes, not the last serial label. The final long draft cue is now three thought-sized cues; no fixed runtime target. Card scope includes fair draws without replacement, consecutive numbers and k > 1. Four added fact keys bind inner/hidden gap counts and delivery ranges/estimates; earlier facts remain exact.


## 16_five_boxes_again: Back to the five boxes (~125 s)

The same five observations return, with a wider scale to include107 and the sixth box90. The sixth observation arrives without replacing the original objects.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 16_1 | This puzzle is often called the German tank problem. In the Second World War, Allied analysts estimated German production from serial numbers on captured equipment. | Dim historical citation with full reading hold; narrow claim supported by the publisher abstract. | card | hold |
| 16_2 | The reasoning needs no war. It needs numbers that count up from one without skipping, and a fair handful. | The same original five boxes return on their ticks; fair-handful badge; wider view accommodates the later confidence endpoint. | number line, callback |  |
| 16_3 | So, the five friends. Box sixty proves there are at least sixty. The gaps say about seventy-one. | At-least60 bracket and label, then five computed gap arcs and71 estimate. | highlight, callback | hold |
| 16_4 | In a fair draw, that recipe is right on average at every possible count. Among recipes that are unbiased at every count, none has a smaller average squared miss. | Precisely scoped best-unbiased claim beside the same example. | equation card |  |
| 16_5 | But seventy-one is one estimate from one handful. Our ninety-five percent confidence range runs from sixty to a hundred and seven. The percentage describes repeated use of the procedure, not a probability for this particular count. | Confidence bracket60–107 on the same scale; explicit distinction between repeated procedure and probability for this count. | bars, highlight | hold |
| 16_6 | And if those five were drawn fairly from one shop's stock, numbered from one, the recipe estimates the shop's stock, not the whole company. | Static red shop outline1–71; badge yields to shop condition; same observed five boxes remain. | controlled change, callback | hold |
| 16_7 | ⟨predict⟩ Now return to a fair draw from the company. A sixth friend turns up with box ninety. What's the new estimate? | Sixth numbered box arrives at90; same five boxes stay; restored badge and prediction countdown. Do not reveal104 yet. | prediction pause, think time |  |
| 16_8 | Ninety, plus one average gap. Six gaps below ninety hold eighty-four boxes, fourteen per gap, so about a hundred and four. | Sixth arc29;84 unseen /6 gaps=14, then90+14=104 on persistent line. | equation, lagged reveal | hold |
| 16_9 | The largest number tells you where the count begins. The gaps tell you the rest, as long as the draw was fair. | Final six-box picture and condition card, with full reading hold. | hold, callback | hold |

### Notes for 16

- **16_1's source: the publisher's abstract, not the paper.** When this
  draft was written, the claim rested only on the 1947 paper's title and on
  secondary descriptions; the paper returned HTTP 403 and was not read
  (`docs/brief.md`, "Sources"). Root has since obtained, through a web
  search, the publisher's indexed abstract of Ruggles and Brodie (1947) at
  https://www.tandfonline.com/doi/abs/10.1080/01621459.1947.10501915. It
  states that in early 1943 the Economic Warfare Division of the American
  Embassy in London used markings and serial numbers on captured German
  equipment to estimate production and strength, first tyres and later
  tanks and other weapons. That supports 16_1's narrow sentence and nothing
  more. Opening the page directly still returns HTTP 403, so the full paper
  has still not been read. The card's page range 72–91 is from the
  secondary descriptions, not from the abstract.
- Wording (root's choice, 2026-10-24): the abstract names an embassy
  division, not a profession, so 16_1 says "Allied analysts" (it said
  "Allied statisticians" before). If 16_1 is cut, 16_2 still opens the
  chapter cleanly.
- No historical number (the often-quoted 169 / 1,000 / 122 table, or any
  production or strength figure) is voiced or shown. The abstract gives
  none, and the paper that prints that table cites a wiki for it.
- `G-F16.sixth` 90, `G-F16.average_gap` 14, `G-F16.estimate` 104.
- 16_5 repeats `G-F9.range` 60,107 and `G-F14.shop_average` 71.
- 16_6 is the final transfer question the owner asked for: the viewer can
  solve it with the picture alone.

Root implementation2026-10-25: best unbiased is explicitly mean squared error; the confidence and shop conditions are separate cues. Shop sampling is fair within consecutively numbered stock starting at1. Historical page range is omitted from the on-screen citation because it was secondary-source metadata. Three new computed facts bind six-gap count, unseen total and full gap list.

Root review repair2026-10-26: chapters14 and16 use the narration's terms “unlikely” and “average squared miss” on screen, including their precise cards. Chapter15 uses × and − in its formulas. Existing character-based reading holds account for the longer wording. The final shop outline reads `G-F14.shop_boxes` directly and checks its equality with this example's gap estimate; it does not derive the stock from the estimate.
