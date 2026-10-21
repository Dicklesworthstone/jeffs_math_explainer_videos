# Storyboard — How to Learn a Secret About a Crowd Without Learning Each Person's Answer

The hand-edited source of this video's words and pictures.
`python3 -I tools/storyboard.py randomized_response` checks it (gate G2) and
writes `script.tsv` from the words column. Every number below is a fact in
`docs/fact-sheet.md`, computed in `src/facts.rs` and recomputed by `oracle.py`.

Words are spoken exactly as written, one thought at a time: no colons,
semicolons, dashes or parentheses in them. `*(…)*` is an annotation, never
spoken. `⟨predict⟩` opens a prediction question; the beat ends with the
question and the picture holds (a visible four-second countdown, "pause for
more time") before the next beat answers it.

## The arc

One invented class runs through the whole video: twenty students, four of
whom copied homework, and one coin rule defined before any formula. The
2026-10-20 owner direction set complete episodes at 10 to 15 minutes; the
five-chapter cut (6:27) became this thirteen-chapter arc, accepted for source
implementation by root (grant 2246) from EmeraldTern's candidate r3. Every
chapter's numbers come from the same model, facts and oracle (`src/model.rs`,
`src/facts.rs`, `oracle.py`). This arc's runtime is not measured; the
headings' seconds are planning budgets, and only native cue timing measures
it.

| Chapter | Spine | Status |
|---|---|---|
| `01_asking_straight_out` | two failed surveys: honest answers count but every yes is a confession; a coin alone hides every answer and counts nothing; whether people answer honestly is a separate question; the rule sits in between | new (grant 2246), uncompiled |
| `02_one_students_coins` | the rule with fair, independent, private coins; one copier through all four outcomes, then (prediction) one who didn't; three times as likely, never more, for one answer under this rule, whatever the survey believed; why the coins must stay private | new beats, with the old 01_3, 01_4, 01_11 and 02_3 beats; uncompiled |
| `03_the_class_on_average` | the class routed on average, seven yes and thirteen no, back to four (the old 01_5 to 01_10, with PinkDuck F1's regroup and step-down) | the old chapter 01's second half, words unchanged; uncompiled split |
| `04_any_class` | average yeses k/2 + n/4, run backwards; one survey's count gives an estimate, right on average and able to miss; (prediction) a class of forty | new, uncompiled |
| `05_what_a_yes_says` | the old chapter 02 without 02_3 (moved to 02) and with 02_2 shortened: three in seven, not three in four | words and pictures as committed apart from those two lines; uncompiled renumbering |
| `06_real_coins_scatter` | the old 03_1 to 03_7: eight real classes, all 4²⁰ outcomes, one in five exactly right, negatives kept | words unchanged; uncompiled split |
| `07_how_far_one_survey_misses` | why every answer wobbles the same; "miss by about four" is the standard deviation, typical not a limit; (prediction) three hundred students miss by more people but a smaller share | new, with the old 03_8; uncompiled |
| `08_the_price_of_the_coin` | the old 03_9 and 03_10 in the share's units: nine against twenty-one percentage points; (prediction) a hundred and fourteen people | new, with the old 03_9 and 03_10; uncompiled |
| `09_when_the_rule_is_not_followed` | everything assumed compliance; (prediction) if the four always say no the estimate averages minus two; bias, not scatter; a negative is a hint, not proof; RAPPOR's own caveat | new, uncompiled |
| `10_asking_again` | the old 04_1 to 04_8: twenty-seven against one, twenty-seven in thirty-one, the majority twenty-seven in thirty-two | words unchanged; uncompiled split |
| `11_one_flip` | a hundred fresh answers, every assumption said; 3⁵⁰ and twenty-three zeros from even odds; (prediction) one flip repeated stays at three; what RAPPOR does | new, with the old 04_9 to 04_11 (answer withheld through the new prediction); uncompiled |
| `12_the_dial` | the old chapter 05, the 05_8 line linking back to chapter 01 | words as committed but 12_8; uncompiled renumbering |
| `13_running_one` | the whole survey for programmers; its typical miss stated in advance, typical not a limit; (prediction) what each step protects; back to the opening | new, uncompiled |

The committed review history of the five-chapter arc (its root batches,
drafts and independent reviews) is in README.md and in git; it covers that
arc only, not this one.

## Vocabulary (one word per idea)

**class** (the twenty; never "sample" or "population" for it); **copied**
(the secret, the true answer; "copied homework"); **coin** and **second coin**
(fair, independent and private; the first is "a fair coin"); **heads** and
**tails**; **tell the truth** (what heads means); **answer** (what someone
says, yes or no); **the survey** (who sees only the answers); **on average**
(every coin outcome equally often; never "expected" in the voice);
**outcome** (one of the four equally likely ways two coins land); **the
chance** (that a particular student copied, given what is known, for a
student picked at random from this class unless the line says otherwise:
"one in five" before an answer, "three in seven" after a yes; never
"probability" or "posterior" in the voice); **three times as likely** (an
answer's ratio between the two truths; never "likelihood ratio" in the
voice); **estimate** (the survey's count from one set of answers: double the
yeses less half the class; never "estimator"); **right on average** (the
estimate's average over every way the coins can land is the truth; never
"unbiased" in the voice); **scatter** and **wobble** (how far real classes
stray); **typical miss**, **miss by** (the standard deviation: the square
root of the average squared miss; "variance" and "standard deviation" only
on cards; a typical size, never a limit); **this fixed class** (everyone in
it answers) against **sample** (twenty picked at random from a big school);
**share** (the fraction who copied, in percent or "one in five"; kept apart
from the count); **fresh coins** (new coins for every answer, independent of
the last); **flips only once** and **repeats that answer** (the memoized
answer; "memoized" only on cards); **the majority**; **the dial** (the chance
of telling the truth, t); **spinner** and **slices**; **estimate variance**
(on screen only). "Randomized response", "likelihood ratio", "posterior",
"differential privacy" and "RAPPOR" are not said; cards show them.

## Color roles

| Idea | Color |
|---|---|
| copied, the truth only we can see: a ring on a student | `hero` gold `#FFD866`, strokes only, never a fill |
| a yes answer: its column and the students in it | blue `#58C4DD` |
| a no answer: its column and the students in it | violet `#C792EA` |
| heads, the truth: its lane and label | green `#83C167` |
| the second coin's answers: its lane, its coin, the outcomes it decides | `dim`, a voice that carries no information |
| a student before answering, the question, and the first coin | `ink` |
| the places answers land in, and the second coin's yeses once taken away (03_5) | `faint` |
| students we infer but cannot see (03_6's doubled count) | a hollow outline in the yes blue |
| an outcome before its answer is drawn | `faint` |
| chapter 01's and 04's chart: the direct question's line `ink`, a coin alone's `dim`, the rule's `green` once derived; the class's point `hero` | as named |
| chapter 06's to 09's distribution: a bar per yes count | `dim` fill; the bar of exactly four `ink` once revealed; the bars below zero `red`, the warn role; chapter 09's broken rule `violet` |
| the true count, four, on the distribution's axis | `hero` gold, a line's stroke and its words |

## 01_asking_straight_out: Asking straight out (~74 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Ask a class of twenty if they've ever copied homework. Anyone who did has a reason to say no. | The question "Have you ever copied homework?" at the top. Twenty students, small ink discs, come in one by one in four rows of five on the left (`model::CLASS_SIZE`, `CLASS_COLUMNS`). On the right an empty chart fades in: "how many copied", 0 to 20, along the bottom; "yes answers, on average", 0 to 20, up the side | lagged reveal of the class, axis | |
| 01_2 | Only we can see the truth. Four of them did. | On "Four of them did" gold rings close around the four who copied (`model::COPIERS`), lagged; a small gold ring and "copied: only we can see this" as a key under the class | lagged highlight | |
| 01_3 | Suppose they all answer honestly anyway. Then every yes is a confession. | Every student takes the colour of a truthful answer: the four ringed students blue, the sixteen others violet. On "every yes" the key becomes "copied: the survey sees this too" and the rings flash; on the chart the direct question's line, yes = k, draws from 0 to 20 in ink, "asked straight out", with a gold point at the class's 4. A "precisely" lower-third: if everyone answers truthfully, the survey learns each person's answer to this one question | recolour, line on the chart, lower-third | |
| 01_4 | Whether people would answer honestly is a separate question. Ours is what each answer gives away. | On "a separate question" a dim line, "will people answer honestly? a separate question", fades in at the upper left; on "Ours is" an ink line under it, "this video's question: what does each answer give away?" | words | |
| 01_5 | So try the opposite. Everyone just flips a coin. Heads yes, tails no. | The card goes once read. The students return to ink, the key to "copied: only we can see this". On "Heads yes" each student takes the colour of their coin's answer on average: half of each group yes (2 of the 4 who copied, 8 of the 16 others), half no; "one coin each, on average: 10 yes" under the class | recolour, words | |
| 01_6 | ⟨predict⟩ Now the survey counts the yeses. What do they tell it about how many copied? | On "counts the yeses" the ten yes students flash; the "?" with the think ring and "pause for more time" between the class and the chart. The chart still has only the direct line; nothing is drawn at 10 | predict pause | |
| 01_7 | Nothing. Four copiers or none, the coin gives ten yeses on average. | On "Nothing" a flat dim line at 10 draws across every k, "coin only"; on "Four copiers or none" its points at 4 and at 0 flash. The two lines stand for a moment (a named 1 s hold). A "precisely" lower-third: a coin alone gives yes with chance 1/2 for everyone, so on average n/2 = 10 yeses, whatever k is | line on the chart, highlight, lower-third | |
| 01_8 | So honest answers give the count, but every yes exposes the person who gave it. A private coin alone reveals nothing about whether anyone copied, and counts nothing. | On "give the count" the direct line flashes; on "exposes the person" the rings flash; on "A private coin alone" the flat line flashes | highlight | |
| 01_9 | What we want sits in between. A coin the survey never sees, following a rule it knows. | The card goes once read. A dashed green line draws between the two lines, from 5 at k = 0 to 15 at k = 20, labelled only "?" | dashed line on the chart | |
| 01_10 | The survey can still estimate the count, and no single answer is a confession. | On "estimate the count" a gold point at the class's 4 on the dashed line (7 yeses) appears and flashes; on "no single answer" the students' rings turn faint. The picture holds 1 s after the line, then the chapter fades | highlight | hold |

### Notes for 01

- The chart's three lines are `model::average_yes` for the direct question
  (truth 1), a coin alone (truth 0) and the rule (truth 1/2), each person's
  chance of a yes added up (`R-F17`); the coin alone's line is the same for
  every k (the facts refuse otherwise).
- "On average" for one coin (01_5): half of each group says yes, checked
  against `average_yes` before it is drawn.
- Privacy claims stay tied to the mechanism: an honest yes reveals that
  person's answer to this one question; a private coin deciding alone, being
  independent of the secret, reveals nothing about it (`R-F17`, review
  RR-DRAFT-REVIEW issue 1, root's r2 finding).
- 01_4 says no claim about honesty is made; none is.
- Prediction (01_6): no point or line at 10 until 01_7.

### Pauses for 01 (semantic timing)

| where | kind | target |
|---|---|---|
| at 01_1 "homework. \| Anyone" | continuation | 0.55 s |
| at 01_2 "the truth. \| Four" | reveal (the rings close on "Four") | 0.65 s |
| at 01_3 "honestly anyway. \| Then every yes" | continuation | 0.5 s |
| at 01_4 "separate question. \| Ours is" | turn to the video's question | 0.6 s |
| at 01_6 after "how many copied?" | **prediction hold**: a plain falling wh-question, then the countdown | the kit's breath + the countdown |
| at 01_7 "Nothing. \| Four copiers" | **reveal**, then its reason | 0.7 s |
| at 01_8 "gave it. \| A private coin" | contrast | 0.6 s |
| at 01_9 "in between. \| A coin" | continuation | 0.5 s |

Units (text only; synthesis only under a requalified route): 01u1 01_1 to
01_4; 01u2 01_5 and 01_6 (ending on the question); 01u3 01_7 to 01_10.

## 02_one_students_coins: One student's coins (~72 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Here is the rule. Everyone flips a fair coin where nobody can see. Heads, tell the truth. | The question, the class with its rings and the key, as at the end of 01_2, fade in. A coin right of the class. On "Heads" a green lane rises from it to the upper row, "heads: the truth", with two empty places at its end under the column heads "yes" (blue) and "no" (violet) | lanes drawn from the rule | |
| 02_2 | Tails, flip a second fair coin, independent of the first, and let it answer. Heads for yes, tails for no. | A dim lane, "tails", falls from the coin to the second coin on the lower row ("second coin" under it) and runs on to the lower row's two places. On "Heads for yes" the word "heads" appears in the lower yes place and flashes with the "yes" head; on "tails for no", "tails" in the lower no place with the "no" head. A "precisely" lower-third, the rule's hypothesis: fair coins, independent across people and across asks, flipped where nobody else can see them | lanes drawn from the rule, highlight, lower-third | |
| 02_3 | Follow a student who copied. Heads first, either way, they tell the truth, yes. Tails then heads, the second coin says yes. Tails then tails, no. | Seat 2 (ringed) flashes; the class, the lanes and the places fade. The grid of the four coin outcomes comes in: "the two coins:" and H H, H T, T H, T T over four columns, the row name "copied". On "Heads first" the first two cells fill blue, "yes", with a green "truth" over both columns; on "Tails then heads" the third fills blue, "yes", with a dim "second coin" over it; on "Tails then tails" the fourth violet, "no", "second coin" over it | grid built from the rule | |
| 02_4 | ⟨predict⟩ Three yeses in four. Now someone who didn't copy. How many of the four give a yes? | "3 of 4" beside the copied row. The row "didn't" comes in under it, its four cells faint, with no answer words. The "?" with the think ring and "pause for more time" at the right. Nothing colours the didn't row | predict pause | |
| 02_5 | One. Heads first gives the truth, no. Only tails then heads says yes. | The didn't row's cells take their answers: H H and H T violet "no", T H blue "yes", T T violet "no"; "1 of 4" beside the row | grid built from the rule | |
| 02_6 | No answer is a confession. But someone who copied says yes three times as often as someone who didn't. | As 01_11 was: on "But someone who copied" the copied row's yes cells flash; on "as someone who didn't" the didn't row's; "× 3" at the right. A "precisely" lower-third: both answers' ratios (01_11's card, shortened to stay two lines: "a yes: 3 of the 4 coin outcomes if you copied, 1 of 4 if you didn't: 3 times as likely, never more. A no is 3 times as likely if you didn't") | highlight, lower-third | |
| 02_7 | For a no, it's the reverse. So under this rule, no answer is more than three times as likely from one truth as the other, whatever the survey believed. | The 02_3 beat, moved here: the shares become "no: 1 of 4" and "no: 3 of 4"; the no cells flash; the "× 3" flashes. On "whatever the survey believed" it flashes again; the ratio card stays | highlight | |
| 02_8 | That needs private coins. Seeing heads first, a survey would know the yes was true. | Once the ratio card has been read, the card becomes: for one answer under this rule, whatever was believed before; it needs coins nobody else sees. On "Seeing heads first" gold rings close around the copied row's two heads-first cells and "seen heads: a confession" appears under the grid. The picture holds 1 s, then the chapter fades | highlight, lower-third | hold |

### Notes for 02

- The single student's cells are `model::answer` on one truth and one coin
  outcome, the old 01_11 grid built cell by cell; the route words are
  `model::route`.
- Prediction (02_4): the didn't row stays faint and wordless until 02_5
  (the old 01_11 drew both rows at once; this answer-withholding change is
  new in grant 2246).
- "Whatever the survey believed" is about the ratio, which the coins alone
  fix (RAPPOR §1, "irrespective of any attacker's prior knowledge", `R-X8`);
  no chance is claimed for every prior. The chances in chapters 05 and 10
  are for a student picked at random from this class, where one in five
  copied; chapter 11's is from even odds, and says so.
- The hypothesis card (02_2) carries the independence across people and
  across asks (root's clarification 1).

### Pauses for 02 (semantic timing)

| where | kind | target |
|---|---|---|
| at 02_1 "the rule. \| Everyone" and "can see. \| Heads" | rule begins | 0.5 s, 0.5 s |
| at 02_1 → 02_2 "Heads, tell the truth. \| Tails" | contrast pair | the kit's continuation gap |
| at 02_2 "let it answer. \| Heads for yes" | continuation | 0.45 s |
| at 02_3 "who copied. \| Heads first" | continuation | 0.5 s |
| at 02_4 after "give a yes?" | **prediction hold** | the kit's breath + the countdown |
| at 02_5 "One. \| Heads first" | **reveal**, then its count | 0.7 s |
| at 02_6 "a confession. \| But someone" | contrast | 0.7 s |
| at 02_8 "private coins. \| Seeing" | continuation | 0.55 s |

Units: 02u1 02_1 to 02_4 (ending on the question); 02u2 02_5 to 02_8.

## 03_the_class_on_average: The class on average (~41 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | On average, half the class tells the truth. Two of our four say yes. | The class, its rings and key, the question, the coin, both lanes, their words and the four places fade in as at the end of 02_2 (a deliberate fade from the previous chapter). The ten students whose coins land heads (every coin outcome equally often: `Class::average_coins`) flash and travel up the green lane: two ringed students into the upper yes place, eight others into its no place, each recoloured by its answer. A "precisely" lower-third: fair coins, flipped in private; these are the average counts; a real class strays from them | motion along lanes, lagged, lower-third | |
| 03_2 | The other ten get the second coin. Five yes, five no, whatever the truth. | The other ten travel down the dim lane through the second coin: five into the lower yes place (one of them ringed), five into its no place (one ringed). The counts 2, 8, 5, 5 stand by their places, checked against the facts' tally | motion along lanes, counts | |
| 03_3 | Seven yes, thirteen no. That's all the survey sees. | "7" over the yes column and "13" over the no column. On "That's all the survey sees" the lanes, coins, their words, the places and their counts, the rings and the key fade while the second coin's students rise into the free slots after the truthful students: one compact yes cluster and one compact no cluster, with no gap exposing the former routes, and their two totals remain | highlight, fade to the survey's view | |
| 03_4 | ⟨predict⟩ From those two numbers, how could the survey get back to four? | On "two numbers" the two totals flash; a "?" with the think ring and "pause for more time" on the left, where the class sat, the countdown | predict pause | |
| 03_5 | Take away the second coin's yeses. On average that's five, a quarter of the class. Two are left. | The "?" goes and the five students from the second coin dim to faint, then step into the row below the two lit yeses, clearing the slots beside them for doubling. On the left the arithmetic is written in TeX: on "On average that's five", 20/4 = 5; on "Two are left", 7 − 5 = 2, and the two lit yes students flash | TeX derivation, highlight | |
| 03_6 | Those two came from the half who told the truth. So double them. Four. | On "So double them", beside the two, two hollow blue outlines appear (the copiers among the other half, whom the second coin hid) and "× 2"; on "Four" the result 2 × 2 = 4 under the arithmetic and the four gold rings come back on the students who really copied: two among the lit yeses, one among the faint yeses, and one in the regrouped no cluster. A "for programmers" lower-third: estimate = 2 * yes - n / 2, in real arithmetic; it can fall outside 0 to n (no yeses give −10, twenty give 30). The picture holds 1.5 s after the line and until the card is read, then the chapter fades | TeX derivation, lower-third (code) | hold |

### Notes for 03

- The words are the old 01_5 to 01_10, unchanged; the pictures are the old
  ones with PinkDuck F1's regroup (03_3) and step-down (03_5), committed at
  252aaac. The rule's picture is rebuilt at the chapter's start, the state
  02_2 ended in, and faded in.
- "On average" means every one of the four coin outcomes lands equally often
  within the copiers and within the others (`Class::average_coins`); the
  card keeps the exact version. The scatter of a real class is chapter 06's.
- Prediction (03_4): the viewer knows the truth (the rings), so the
  question is the survey's method.

### Pauses for 03 (semantic timing)

| where | kind | target |
|---|---|---|
| at 03_1 "tells the truth. \| Two of our four" | continuation, the picture moving | 0.55 s |
| at 03_2 "the second coin. \| Five yes" | continuation | 0.45 s |
| at 03_3 "thirteen no. \| That's all" | turn to the survey's view | 0.6 s |
| at 03_4 after "get back to four?" | **prediction hold** | the kit's breath + the countdown |
| at 03_5 "coin's yeses. \| On average" and "the class. \| Two are left" | steps of one calculation | 0.5 s, 0.55 s |
| at 03_6 "told the truth. \| So double them" | continuation | 0.5 s |
| at 03_6 "double them. \| Four." | **reveal**: the answer lands alone | 0.9 s |

Units: 03u1 03_1 to 03_4 (ending on the question); 03u2 03_5 and 03_6.

## 04_any_class: Any class (~74 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | A real survey has no rings to check. So take a class of n, where k copied. | Chapter 01's chart returns, its direct and coin-only lines faint; the axes now read "k who copied, 0 to n" and "yes answers, on average" | axis, lines on the chart | |
| 04_2 | On average, half the k copiers tell the truth and say yes, and the second coin says yes for a quarter of the class. | At k = 4 a stacked bar grows from the axis: a green part, k/2 = 2, "k/2" beside it; on "the second coin" a dim part on top, n/4 = 5, "n/4" | bars on the chart | |
| 04_3 | So the yeses average half of k plus a quarter of n. For us, two plus five, seven. | On the left, TeX: y = k/2 + n/4, "y: yes answers on average" under it in dim; the rule's line draws, green, from 5 at k = 0 to 15 at n; on "two plus five", 2 + 5 = 7 under the formula and a gold point at (4, 7). A "precisely" lower-third: on average yes = k/2 + n/4 (Dwork and Roth, section 2.3: share 1/4 + p/2), so k = 2 × (yes − n/4) | TeX formula, line on the chart, lower-third | |
| 04_4 | Run that backwards. Take away a quarter of the class, double what's left, and the average gives back k exactly. | An arrow runs from 7 on the yes axis across to the line and down to k = 4 (a named 1 s hold for it); TeX k = 2 × (y − n/4) | TeX formula, arrow on the chart | |
| 04_5 | But a survey hears a single count, not the average. The same steps turn it into an estimate of k, right on average, and able to miss. | Once the card is read it goes. On "a single count" a tick at 9 on the yes axis, chapter 06's fifth seeded class, "one real class: 9 yes, estimate 8"; its arrow lands at k = 8 beside the gold 4. On "right on average" the exact distribution of yes counts lies faint along the yes axis as thin bars. A "precisely" lower-third: estimate = 2 × yes − n/2; over every coin outcome it averages k, for every k; one survey's estimate can miss | arrow on the chart, bars, lower-third | |
| 04_6 | ⟨predict⟩ A class of forty says yes eighteen times. What does the survey estimate? | The real class's tick and words go; "n = 40, y = 18" (TeX) and "estimate = ?" on the left; the think ring | predict pause | |
| 04_7 | Sixteen. A quarter of forty is ten. Eighteen take away ten is eight, and double eight is sixteen. An estimate, not a count. | TeX, line by line with the words: 40/4 = 10; 18 − 10 = 8; 2 × 8 = 16; then "an estimate, not a count" in dim | TeX derivation | |
| 04_8 | The doubling undoes the first coin. The quarter is the second coin's yeses. | The class of forty goes; under k = 2 × (y − n/4), "2 ×: undoes the first coin" in green, then "n/4: the second coin's yeses" in dim. The picture holds 1 s, then the chapter fades | words, highlight | hold |

### Notes for 04

- The average yeses k/2 + n/4 hold for every k (`model::average_yes`,
  `R-F17`, pinned to Dwork and Roth §2.3 by `R-F2.yes_share`); running them
  backwards returns k exactly only for the average.
- One survey's count gives an estimate, right on average for every number of
  copiers (`R-F18.unbiased_every_k`, the old chapter 03's test), able to
  miss: the shown class is chapter 06's fifth seeded draw, 9 yeses, an
  estimate of 8 (`R-F18.shown.*`, review issue 2).
- Prediction (04_6): nothing for the class of forty is computed on screen
  before 04_7 (`R-F18`: 10, 8, 16).

### Pauses for 04 (semantic timing)

| where | kind | target |
|---|---|---|
| at 04_1 "to check. \| So take" | turn to the general class | 0.6 s |
| at 04_3 "a quarter of n. \| For us" | continuation | 0.5 s |
| at 04_5 "not the average. \| The same steps" | contrast | 0.6 s |
| at 04_6 after "the survey estimate?" | **prediction hold** | the kit's breath + the countdown |
| at 04_7 "Sixteen. \| A quarter" | **reveal**, then its steps | 0.7 s |
| at 04_7 "is sixteen. \| An estimate" | conclusion | 0.6 s |

Units: 04u1 04_1 to 04_4; 04u2 04_5 and 04_6 (ending on the question); 04u3
04_7 and 04_8.

## 05_what_a_yes_says: What a yes says (~74 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Give every student all four coin outcomes, equally likely. Twenty students, eighty outcomes. | Chapter 01's twenty students as a row of discs across the top: the four who copied first, ringed in gold under "copied", then after a gap the sixteen others under "didn't copy". Under each disc a column of four faint squares appears, lagged row by row, one per coin outcome, rows labelled H H, H T, T H, T T on the left. On "Twenty students", 20 × 4 = 80 (TeX) on the right | grid built from the rule, lagged | |
| 05_2 | Each copier has three yes outcomes, everyone else one. | The copiers' squares take their answers' colours, the rule run on each outcome: three blue, one violet; "yes: 3 of 4" under the copiers. On "everyone else" the others' squares: one blue, three violet each; "yes: 1 of 4" under them; then "× 3" at the right | grid built from the rule, highlight | |
| 05_3 | ⟨predict⟩ So a student says yes. What's the chance they copied? | The 80 total goes; a "?" with the think ring and "pause for more time" right of the grid, the countdown. No total of yes squares is on screen | predict pause | |
| 05_4 | Count the yes outcomes. The four who copied have twelve, the sixteen others sixteen. | The "?" goes and every violet square dims to faint. On "have twelve" the copiers' blue squares flash and "12" replaces their label; on "the sixteen others" the others' blue squares flash and "16" replaces theirs | highlight, counts | |
| 05_5 | Twelve of twenty-eight. On average, three of the seven yeses. Three in seven, not three in four. | Under the grid, 12/28 (TeX), standing where the full line's fraction will be; on "On average" the line "12/28 = 3/7" and, beside it, "a quarter of the outcomes: 3 of chapter 3's 7 yes answers" fade in over it (PinkDuck F3); on "not three in four", on the right "3/4 is a copier's chance of saying yes". A "precisely" lower-third: 3/7 is the chance that someone who said yes copied, given this class; it would be 3/4 only if half the class had copied | TeX derivation, lower-third | |
| 05_6 | Each copier has three times the yeses, but the others outnumber them four to one. | The derivation and card stay; the copiers' column labels flash "3 each", the others' "1 each"; then the two groups of discs flash, "4" and "16" over them | highlight | |
| 05_7 | A no works the same way. Four of the fifty-two no outcomes are a copier's, one in thirteen. | The labels and derivation go; the blue squares dim and the violet ones light again. On "Four of the fifty-two" the copiers' violet squares flash with "4" under them and "48" under the others'; 4/52 = 1/13 (TeX) under the grid | highlight, TeX derivation | |
| 05_8 | A yes lifts the chance from one in five to three in seven. A no drops it to one in thirteen. | The derivation goes. A chance line from 0 to 1 under the grid: a gold mark at 1/5, "the class: 4 of 20"; on "three in seven" a blue mark at 3/7 and a blue arc from 1/5 to it; on "one in thirteen" a violet mark at 1/13 and a violet arc | number line, motion | |
| 05_9 | The coin fixes the three times. Where the chance ends up depends on the class. | The arcs flash. A "precisely" lower-third: an answer multiplies the odds by 3 for a yes, 1/3 for a no; here 1 : 4 becomes 3 : 4 or 1 : 12. If nobody copied, a yes would mean 0 | lower-third, highlight | |
| 05_10 | An answer is deniable, but not meaningless. How well the survey counts is a separate question. | The marks stay. On "deniable" the 3/7 mark flashes (far from 1); on "not meaningless" the blue arc flashes (far from 1/5). On "How well the survey counts", a "names to search" lower-third: likelihood ratio, posterior probability, randomized response, differential privacy. The picture holds 2 s after the line | lower-third, highlight | hold |

### Notes for 05

- The old chapter 02, words unchanged but 05_2 (shortened: chapter 02 has
  already counted the ratio) and without 02_3 (moved to 02_7). Every square
  is `model::answer` on one student and one outcome, checked against
  `R-F11` before it is labelled.
- The chances 3/7 and 1/13 are for a student picked at random from this
  class, where 4 of 20 copied (the 05_5 card); the factor of three is the
  coins' (05_9). Prediction (05_3): the tempting answer is 3/4.

### Pauses for 05 (semantic timing)

| where | kind | target |
|---|---|---|
| at 05_1 "equally likely. \| Twenty students" | continuation | 0.55 s |
| at 05_3 "says yes. \| What's the chance" | set-up, then the question | 0.45 s |
| at 05_3 after "they copied?" | **prediction hold** | the kit's breath + the countdown |
| at 05_4 "the yes outcomes. \| The four who copied" | continuation | 0.5 s |
| at 05_5 "twenty-eight. \| On average" | continuation | 0.55 s |
| at 05_5 "seven yeses. \| Three in seven" | **reveal**, and its contrast kept in the same breath | 0.8 s |
| at 05_7 "the same way. \| Four of the" | continuation | 0.5 s |
| at 05_8 "three in seven. \| A no drops it" | contrast | 0.6 s |
| at 05_9 "the three times. \| Where the chance" | contrast | 0.6 s |
| at 05_10 "not meaningless. \| How well" | turn to a separate question | 0.8 s |

Units: 05u1 05_1 to 05_3 (ending on the question); 05u2 05_4 to 05_7; 05u3
05_8 to 05_10.

## 06_real_coins_scatter: Real coins scatter (~57 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Real coins don't land on average. Ask the class eight times with fresh coins. | On "Ask the class" eight rows appear one after another, each the same twenty students as small discs coloured by the answer their coins gave this time, the four who copied ringed in gold; each row's yes count at its right under "yes". The coins are real draws (`facts::runs`) | lagged rows built from the rule | |
| 06_2 | Their estimates run from minus four to fourteen. Only one lands on four. | Each row's estimate appears under "estimate" (−4, 0, 14, 10, 8, 4, −2, 6); on "minus four to fourteen" the two extremes flash; on "Only one lands on four" the one 4 flashes. A "precisely" lower-third: the first eight draws from PCG64DXSM, seed 3, none thrown away; too few to show the rates | highlight, lower-third | |
| 06_3 | Now count every way the coins can land. Four to the twentieth, over a trillion, all equally likely. | The rows and card go. A two-row axis appears across the lower half: yes counts 0 to 20 above, the estimates −10 to 30 below, each estimate under its yes count. On "Four to the twentieth" 4²⁰ = 1,099,511,627,776 (TeX) at the upper right, and the bars rise left to right, one per yes count, each as tall as its exact chance | histogram grown from the distribution, TeX | |
| 06_4 | The estimates pile up around four, the true count. | A gold line at estimate 4 (seven yeses), "4 copied" above it; the bars beside it flash | highlight | |
| 06_5 | ⟨predict⟩ How often does a class land exactly on four? | The 4²⁰ total goes; a "?" with the think ring and "pause for more time" at the right. No bar is labelled with a chance yet, and the axis has no chance scale | predict pause | |
| 06_6 | About one class in five. And about one in eleven goes below zero. | The "?" goes; on "one class in five" the bar at 4 turns ink and "exactly 4: about 1 class in 5 (20%)" appears at the upper left; on "below zero" the bars of estimates −10 to −2 turn red, "below 0 (under 5 yeses): about 1 in 11 (9%)". A "precisely" lower-third: over all 1,099,511,627,776 equally likely ways, exactly 4 about 20% of the time, below 0 about 9% | highlight, lower-third | |
| 06_7 | Below zero means fewer yeses than the second coin alone gives on average. Rounding those up to zero would push the average above four. | The red bars (yes counts under 5) flash. On "Rounding those up", "rounded up to 0, the average would be 4.26" at the upper left; the card becomes: allow negatives: 2 × yes − 10 averages 4; replace them with 0: the average is 4.26. The picture holds until the card is read, then the chapter fades | highlight, lower-third | hold |

### Notes for 06

- The old 03_1 to 03_7, words and pictures unchanged (seed 3, the first
  eight draws, none thrown away; the exact distribution over 4²⁰
  configurations, `R-F7`, `R-F13`, `R-F14`).
- Prediction (06_5): no chance is labelled before 06_6.

### Pauses for 06 (semantic timing)

| where | kind | target |
|---|---|---|
| at 06_1 "on average. \| Ask the class" | turn from the claim to the test | 0.6 s |
| at 06_2 "to fourteen. \| Only one" | continuation | 0.55 s |
| at 06_3 "can land. \| Four to the twentieth" | continuation | 0.5 s |
| at 06_5 after "exactly on four?" | **prediction hold** | the kit's breath + the countdown |
| at 06_6 "one class in five. \| And about one in eleven" | **reveal**, then a second fact | 0.7 s |
| at 06_7 "on average. \| Rounding those up" | continuation | 0.6 s |

Units: 06u1 06_1 and 06_2; 06u2 06_3 to 06_5 (ending on the question); 06u3
06_6 and 06_7.

## 07_how_far_one_survey_misses: How far one survey misses (~69 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Why so much scatter? A copier says yes three times in four, anyone else once in four. | Chapter 06's axis, bars (dim again) and gold line at 4 fade in, the state 06 ended in. At the upper right two small rows of four outcome squares: "copied" three blue and one violet, "didn't" one blue and three violet | histogram, grid built from the rule | |
| 07_2 | Those are mirror images, just as hard to predict. Every answer wobbles the same, whoever gives it. | "3/16" under each row. The old 03_8 derivation begins at the right: 3/16 × 20 = 15/4 (TeX). A "precisely" lower-third (03_8's): each answer's variance is 3/16 for anyone; 20 make 15/4; doubling the count multiplies variance by 4: 15. Standard deviation: about 3.87 | TeX derivation, lower-third | |
| 07_3 | Twenty of them, doubled, miss by about four people. | As 03_8: 15/4 × 4 = 15, then √15 ≈ 3.87; on "miss by about four people" a white bracket over the bars from 4 − 3.87 to 4 + 3.87, labelled ±√15 ≈ ±3.87 (PinkDuck F4) | TeX derivation, bracket | |
| 07_4 | That's a typical miss. Square each class's miss, average over every way the coins land, and take the square root, here of fifteen. | The two small rows go. Under the derivation, TeX: √(average of (estimate − 4)²) = √15 ≈ 3.87. Once the 03_8 card has been read, the card becomes: typical miss: the standard deviation, the square root of the average squared miss over every equally likely outcome: about 3.87 | TeX formula, lower-third | |
| 07_5 | It's typical, not a limit. More than two classes in five miss by four or more. | The bars of estimates 0 or less and 8 or more turn ink, "miss by 4 or more: 44% of outcomes" at the upper left. Once read, the card becomes: not a bound: 44% of outcomes miss by 4 or more, about 1 in 15 by 8 or more | highlight, lower-third | |
| 07_6 | ⟨predict⟩ Now ask three hundred students, same coins. Does the estimate miss by more people than in our class, or fewer? | The derivation and the note go. At the upper right an empty panel, "300 students, same coins", a dim line for its estimates; the think ring under it. No bracket and no number | predict pause | |
| 07_7 | More. About fifteen, the square root of three quarters of three hundred. But that's one in twenty of them. Our miss of four was about one in five. | On the panel's line a gold mark at 60 (one in five of 300), "60 copied", and a bracket of ±15 around it; TeX √(3/4 × 300) = √225 = 15. On "one in twenty" two share bars: "20 students: 3.87, 19%" and "300 students: 15, 5%". They stand for a moment (a named 1 s hold) | number line, bracket, bars | |
| 07_8 | In people, the miss grows. As a share, it shrinks. | "in people: 3.87 → 15" under the brackets flashes, then "as a share: 19% → 5%". Once read, the card becomes: fixed class of n, everyone answers: the typical miss is the square root of 3n/4, whatever k; 20: 3.87 (19% of 20); 300: 15 (5% of 300). The picture holds until it is read, then the chapter fades | highlight, lower-third | hold |

### Notes for 07

- "Miss by about four people" is the standard deviation, √15 ≈ 3.87, the
  root of the average squared miss over all 4²⁰ equally likely outcomes
  (`R-F7`); a typical size, not a bound: 239955963659/549755813888 (44%) of
  outcomes miss by 4 or more, 73349641357/1099511627776 (about one in 15)
  by 8 or more (`R-F19`; root convolved them independently). Estimates are
  even numbers, so missing by more than √15 is missing by 4 or more.
- Three hundred students: the estimate's variance is n times one person's
  3/4 whoever copied (chapter 13's check, `R-F23`), 225, a miss of 15; the
  mark at 60 is one in five of 300 (`R-F19.big.*`). No 300-person histogram
  is drawn (root's clarification 2).
- Count against share (review issue 4): in people the miss grows, 3.87 to
  15; as a share it shrinks, 19% to 5%.

### Pauses for 07 (semantic timing)

| where | kind | target |
|---|---|---|
| at 07_1 "so much scatter? \| A copier" | question to its answer | 0.6 s |
| at 07_2 "to predict. \| Every answer" | continuation | 0.55 s |
| at 07_4 "a typical miss. \| Square each" | definition | 0.55 s |
| at 07_5 "not a limit. \| More than" | continuation | 0.55 s |
| at 07_6 after "or fewer?" | **prediction hold** | the kit's breath + the countdown |
| at 07_7 "More. \| About fifteen" | **reveal** | 0.7 s |
| at 07_7 "three hundred. \| But that's" | contrast | 0.6 s |

Units: 07u1 07_1 to 07_3; 07u2 07_4 to 07_6 (ending on the question); 07u3
07_7 and 07_8.

## 08_the_price_of_the_coin: The price of the coin (~44 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | That's for this fixed class. Sample twenty from a big school instead, and who gets picked adds its own scatter. | Chapter 07's 20-student axis, bars, gold line and bracket fade in. On "Sample twenty" the variance of the estimated share for independent draws comes in at the right: p(1 − p)/n + 3/(4n) (TeX), "variance of the estimated share" above it, "who gets picked, plus the coins" under it. A "precisely" lower-third, which stays to the end: assumes independent draws, with replacement or from a large population, and truthful direct answers; at p = 1/5 that is 91/16, about 5.69 times | TeX, formula, lower-third | |
| 08_2 | Say one in five in the school copied. Twenty direct, truthful answers would miss that twenty percent by about nine percentage points. Twenty coin answers, by about twenty-one. | The 20-student bars, line and bracket go. A share axis on the left, −40% to 80%, a gold tick at 20%. On "nine percentage points" a narrow ink bracket, "20 direct answers: about 8.94 points"; on "twenty-one" a wide dim bracket under it, "20 coin answers: about 21.33 points". They stand for a moment (a named 1 s hold) | number line, brackets | |
| 08_3 | ⟨predict⟩ How many coin answers would it take to miss by only nine points? | "coin answers: 20" under the wide bracket; the think ring. No number beside it | predict pause | |
| 08_4 | About a hundred and fourteen. If one in five copied, matching a direct question's precision takes almost six times as many people. That's the price of the coin. | The wide bracket narrows to the narrow one's width, "114 coin answers: about 8.93 points", "coin answers: 114"; TeX 0.91/m = 0.16/20 gives m = 113.75. On "almost six times", as 03_10: 91/16 ≈ 5.69 (TeX) under the formula, "times as many people, for the same precision" under it; the card stays. The picture holds 2 s after the line | number line, TeX | hold |

### Notes for 08

- The old 03_9 and 03_10, made explicit in the share's units (review
  issue 4): independent draws at p = 1/5 with truthful direct answers; the
  share's standard deviation √(0.16/20) ≈ 8.94 points directly, √(0.91/20)
  ≈ 21.33 with the coins; the fewest coin answers that match, 114 (455/4
  exactly; `R-F20`), 91/16 ≈ 5.69 times twenty (`R-F8`). The count's misses
  (1.79 and 4.27) are not spoken, so count and share never mix.
- The sample card's wording is the earlier reading-time repair (F5), kept.

### Pauses for 08 (semantic timing)

| where | kind | target |
|---|---|---|
| at 08_1 "this fixed class. \| Sample twenty" | contrast | 0.7 s |
| at 08_2 "in the school copied. \| Twenty direct" | continuation | 0.5 s |
| at 08_2 "percentage points. \| Twenty coin answers" | contrast | 0.6 s |
| at 08_3 after "only nine points?" | **prediction hold** | the kit's breath + the countdown |
| at 08_4 "a hundred and fourteen. \| If one in five" | **reveal**, then its reading | 0.7 s |

Units: 08u1 08_1 to 08_3 (ending on the question); 08u2 08_4.

## 09_when_the_rule_is_not_followed: When the rule isn't followed (~54 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Everything so far assumed that everyone lets the coins decide. | The class as one row of twenty small discs at the top, the four who copied ringed; "assumed: everyone lets the coins decide" over it. Chapter 06's axis and its dim bars, with the gold line at 4, below | histogram, words | |
| 09_2 | ⟨predict⟩ Suppose the four who copied always say no, whatever their coins. On average, what does the survey estimate? | The four ringed discs turn violet, "always no" under them; the think ring at the right. Nothing is tallied | predict pause | |
| 09_3 | Minus two. The sixteen others give four yeses on average. Double four is eight, take away ten, minus two. | The sixteen others take their answers on average, one in four yes: 4 blue, 12 violet. TeX 2 × 4 − 10 = −2 at the right; a violet line at −2 on the axis, beside the gold 4 | recolour, TeX, highlight | |
| 09_4 | The truth is four. That isn't scatter. Here the average itself is pulled down, and asking more people won't fix it. | On "pulled down" the honest bars fade to faint while the broken rule's bars rise in violet, centred on −2 (its exact distribution). On "asking more people" a small chart at the upper right: "estimated share, on average" against "people asked" (20, 100, 300), three violet points at −10% under a gold line at 20%. A "precisely" lower-third: assumed: every copier says no and everyone else follows the rule; yes averages (n − k)/4 and the estimate −k/2; as a share, −p/2 for any n | histogram, chart, lower-third | |
| 09_5 | A negative estimate is a hint, not proof. Our class, following the rule, goes below zero about one time in eleven. | The honest bars return; those below zero turn red, "rule followed: below 0 about 1 in 11" | highlight | |
| 09_6 | The designers of a system for collecting such answers call absolute compliance an assumption that may not hold for human subjects. | Once read, the card becomes: RAPPOR (Erlingsson, Pihur and Korolova, CCS 2014), section 1: absolute compliance is 'an assumption that may not hold for human subjects'. The picture holds until it is read, then the chapter fades | lower-third | hold |

### Notes for 09

- One noncompliance pattern, stated as such: every copier says no whatever
  the coins, everyone else follows the rule (`model::yes_count_distribution_copiers_say_no`,
  `R-F21`). The average estimate is −k/2, here −2, a bias of −6; the share
  averages −p/2 = −1/10 whatever the number asked (checked at 20, 100 and
  300 by the oracle). Other patterns push either way or cancel (root's
  r2 audit: a copier always saying no and a non-copier always saying yes
  cancel in the average count), so no line says compliance is necessary:
  the calculation assumes it (review issue 3, root's r2 findings).
- The quotation is RAPPOR §1 verbatim (`R-X6`); the voice names no system.

### Pauses for 09 (semantic timing)

| where | kind | target |
|---|---|---|
| at 09_2 "whatever their coins. \| On average" | set-up, then the question | 0.5 s |
| at 09_2 after "the survey estimate?" | **prediction hold** | the kit's breath + the countdown |
| at 09_3 "Minus two. \| The sixteen" | **reveal**, then its steps | 0.7 s |
| at 09_4 "is four. \| That isn't" and "scatter. \| Here" | contrast | 0.55 s, 0.55 s |
| at 09_5 "not proof. \| Our class" | continuation | 0.55 s |

Units: 09u1 09_1 and 09_2 (ending on the question); 09u2 09_3 to 09_6.

## 10_asking_again: Asking again (~57 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Ask one student the same question three times, with fresh coins each time. | One student (an ink disc, no ring: we do not know their truth). On "three times" three answer places appear one by one to its right, "ask 1", "ask 2", "ask 3"; on "fresh coins" each gets its own two coins, the first ink, the second dim | lagged places, coins | |
| 10_2 | Four coin outcomes for each answer make sixty-four for all three, all equally likely. | The asks go. Two grids of 64 squares, "copied" (gold) on the left and "didn't copy" on the right, every square faint: one square per way the six coins can land, in 2 × 2 blocks of 4 × 4. On "sixty-four", 4 × 4 × 4 = 64 (TeX) at the upper right. A "precisely" lower-third: each square is one way all 6 coins can land, 64 equally likely. Block: the first answer's coins; row: the second's; column: the third's | grid built from the rule, TeX, lower-third | |
| 10_3 | ⟨predict⟩ How many times as likely are three yeses from someone who copied? | A "?" with the think ring and "pause for more time" at the right. No square is coloured yet, and nothing counts the yeses | predict pause | |
| 10_4 | Twenty-seven. Someone who copied says yes three times in twenty-seven outcomes, anyone else in one. | The "?" goes. In the left grid the 27 squares whose three answers are all yes turn blue. Under it "3 yes: 27 of 64". On "anyone else in one" the one such square on the right turns blue, "3 yes: 1 of 64", and 27/1 = 27 (TeX) at the right | highlight, TeX | |
| 10_5 | Each fresh answer can multiply it by three. Three noes do the same the other way. | 27 = 3 × 3 × 3 (TeX) under the ratio. On "Three noes" the all-no squares turn violet, 1 on the left and 27 on the right. The card becomes: 0 to 3 yeses are 1/27, 1/3, 3, 27 times as likely from someone who copied; the most after 1 to 5 fresh answers: 3, 9, 27, 81, 243 | highlight, TeX, lower-third | |
| 10_6 | In our class, one in five copied. Those four have a hundred and eight such outcomes, the sixteen others sixteen. | The grid names become "4 copied" and "16 didn't copy". On "a hundred and eight" the left grid's blue squares flash and 4 × 27 = 108 (TeX); on "the sixteen others" 16 × 1 = 16 | highlight, TeX | |
| 10_7 | So after three yeses, the chance they copied is twenty-seven in thirty-one. After one, three in seven. | On "twenty-seven in thirty-one", 108/(108 + 16) = 27/31 (TeX), and the card becomes: 27/31 is the chance in this class, where 4 of 20 copied; from an even 1/2 the same 3 yeses would leave 27/28, and if nobody copied, 0. On "After one", "after one yes: 3/7" in dim | TeX fraction, lower-third | |
| 10_8 | The majority of three answers is right twenty-seven times in thirty-two, whichever the truth. | On the left the squares with at least two yeses turn blue, on the right those with at least two noes violet: "majority right: 54 of 64" under each; 54/64 = 27/32 (TeX). The picture holds 1.5 s, then the chapter fades | highlight, TeX | hold |

### Notes for 10

- The old 04_1 to 04_8, words and pictures unchanged; the chapter now ends
  after the majority (the memoized answer is chapter 11's). The chance 27/31
  is for a student picked at random from this class, where one in five
  copied (the 10_7 card gives the even-prior and nobody-copied edges).

### Pauses for 10 (semantic timing)

| where | kind | target |
|---|---|---|
| at 10_1 "three times, \| with fresh coins" | continuation | 0.5 s |
| at 10_3 after "from someone who copied?" | **prediction hold** | the kit's breath + the countdown |
| at 10_4 "Twenty-seven. \| Someone who copied" | **reveal**, then its count | 0.7 s |
| at 10_6 "one in five copied. \| Those four" | continuation | 0.55 s |
| at 10_7 "thirty-one. \| After one" | contrast | 0.6 s |

Units: 10u1 10_1 to 10_3 (ending on the question); 10u2 10_4 and 10_5; 10u3
10_6 to 10_8.

## 11_one_flip: One flip (~68 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | Now picture a program asking the same student every day, with fresh coins, about a secret that never changes. | One student; a strip of 100 day squares fills, lagged, faint; "the same secret, fresh coins every day" over it | lagged grid | |
| 11_2 | After a hundred answers, seventy-five yes and twenty-five no. Each yes multiplies the evidence by three and each no divides it, leaving fifty threes multiplied together. | The squares take their answers, sorted: 75 blue, 25 violet, "75 yes, 25 no". TeX at the right: 3⁷⁵ / 3²⁵ = 3⁵⁰, then 3⁵⁰ = 717,897,987,691,852,588,770,249, "24 digits". A "precisely" lower-third: fresh coins each day, one secret that never changes, the rule followed, and even odds at the start: every yes multiplies the odds by 3, every no divides them by 3 | TeX, grid, lower-third | |
| 11_3 | From even odds, and with the rule followed, the chance they didn't copy now starts with twenty-three zeros after the decimal point. | TeX: 1/(3⁵⁰ + 1) ≈ 1.39 × 10⁻²⁴ (RAPPOR's figure); under it the decimal written out, 0.00000000000000000000000139, its zeros bracketed "23 zeros". It stands for a moment (a named 1.5 s hold) | TeX, words | |
| 11_4 | ⟨predict⟩ But suppose the student flips only once, and repeats that answer. How many times as likely are three yeses now? | The strip and TeX go. For each truth, four columns, one per outcome of the one flip (H H, H T, T H, T T over them), each column three squares, all **faint**, under "copied" and "didn't copy"; the think ring. No column is coloured and the memoized card is held back | grid built from the rule, predict pause | |
| 11_5 | Then three yeses are three times as likely, the same as one. The repeats add nothing. | The columns take their answers: on the left three blue and one violet, on the right one blue and three violet; "3 yes: 3 of 4" and "3 yes: 1 of 4"; 3/1 = 3 (TeX); on "the same as one" it flashes, and "fresh coins: 27" in dim. The memo card (04_9's, unchanged): memoized: one flip for this secret, its answer re-sent each ask, so repeats add no evidence; only for one unchanged secret, under this rule | TeX, highlight, lower-third | |
| 11_6 | That holds only while the secret stays the same, under this rule. | The grid names flash | highlight | |
| 11_7 | A design published in twenty fourteen and tried in Google's Chrome browser builds on this. It remembers a randomized answer, then randomizes every report of it again. | The columns go. Two boxes and an arrow: "a value → randomized once, remembered" then "every report → randomized again". Once the memo card is read, the card becomes: RAPPOR: a randomized answer memoized per value, then randomized again on each report; many bits per report, not this one-bit rule. The picture holds until it is read, then the chapter fades | words, lower-third | hold |

### Notes for 11

- A hundred fresh answers (RAPPOR §1.3's example, `R-X3`, `R-F22`): each
  yes multiplies the odds by 3, each no divides them by 3, whatever the
  order; 75 yes and 25 no leave 3⁵⁰, a 24-digit number; from even odds the
  chance of no copying is 1/(3⁵⁰ + 1) ≈ 1.39 × 10⁻²⁴, 23 zeros after the
  point. Every assumption is said: a secret that never changes, fresh
  coins, the rule followed, even odds (review issue 5).
- Prediction (11_4): the old 04_9 drew coloured columns and its card said
  "repeats add no evidence"; both now wait for 11_5 (the second
  answer-withholding change of grant 2246).
- RAPPOR (11_7): a memoized Permanent randomized response per value, then an
  Instantaneous randomized response on every report, over Bloom-filter bits;
  limited deployment in Google's Chrome browser (§1.1, §1.3, §2, `R-X7`).
  It builds on memoization; it is not this one-bit rule (review issue 6).

### Pauses for 11 (semantic timing)

| where | kind | target |
|---|---|---|
| at 11_2 "twenty-five no. \| Each yes" | continuation | 0.55 s |
| at 11_4 "that answer. \| How many" | set-up, then the question | 0.5 s |
| at 11_4 after "three yeses now?" | **prediction hold** | the kit's breath + the countdown |
| at 11_5 "the same as one. \| The repeats" | continuation | 0.55 s |
| at 11_7 "builds on this. \| It remembers" | continuation | 0.55 s |

Units: 11u1 11_1 to 11_3; 11u2 11_4 (the question alone); 11u3 11_5 to
11_7.

## 12_the_dial: The dial (~73 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Make the first coin a dial, the chance of telling the truth. Ours sits at one half. | A number line across the top left, "the chance of telling the truth", ticks 0, 1/4, 1/2, 3/4, 1, a green marker at 1/2. On "Ours sits at one half": a spinner of four slices over a row of eight outcome squares, two slices green and two dim; two rows of eight squares, "copied" and "didn't copy", coloured by their answers, "yes: 6 of 8" and "yes: 2 of 8". At the right a table: the dial, times as likely, estimate variance; the 1/2 row reads 3 and 3/4 | number line, grid built from the rule, table | |
| 12_2 | Set it to three quarters, a spinner of four slices, three saying tell the truth. | On "three quarters" the marker moves to 3/4, a third slice turns green, the squares go faint and their counts go; the table gains a 3/4 row with its ratio and variance blank. On "a spinner" a "precisely" lower-third: a spinner of 4 equal slices, 3 saying tell the truth and 1 handing the answer to the fair second coin: 4 × 2 = 8 equally likely outcomes | number line, lower-third | |
| 12_3 | ⟨predict⟩ At three quarters, how many times as likely is a yes from someone who copied? | A "?" with the think ring and "pause for more time" at the lower right. The squares are faint and the 3/4 row has no ratio | predict pause | |
| 12_4 | Seven. Of the eight outcomes, someone who copied says yes in seven, anyone else in one. | The "?" goes; the squares take their answers' colours, "yes: 7 of 8" and "yes: 1 of 8"; the 3/4 row's ratio reads 7. The card becomes: at 3/4 a yes is 7 of 8 outcomes for someone who copied, 1 of 8 for anyone else, and a no the reverse: 7 times either way, (1 + t)/(1 − t) | highlight, table, lower-third | |
| 12_5 | Turning the dial up makes every answer say more. | Under the table a general row: t, (1 + t)/(1 − t) (TeX); the ratios 3 and 7 flash | TeX formula, highlight | |
| 12_6 | But the count gets sharper. Our class of twenty would miss by about two people, not four. | On "miss by about two" the 3/4 row's variance reads 7/36 and the general row gains (1 − t²)/(4t²) (TeX). Under the outcome rows, "our class of 20 misses by" with two lines (TeX): 1/2: √(20 × 3/4) ≈ 3.87 and 3/4: √(20 × 7/36) ≈ 1.97; both flash. The card becomes: estimate variance per person (1 − t²)/(4t²), 4 × an answer's 3/16 = 3/4 at 1/2; class of 20: 20 × it; share it/20; a sample adds p(1 − p)/n | TeX formula, highlight, lower-third | |
| 12_7 | Down at a quarter, a yes is under twice as likely, but the class would miss by about nine. | The marker moves to 1/4, two green slices turn dim, the squares recolour, "yes: 5 of 8" and "yes: 3 of 8"; the table gains the 1/4 row, 5/3 and 15/4. On "about nine" a third line, 1/4: √(20 × 15/4) ≈ 8.66, and it and the 1/2 line flash | number line, table, highlight | |
| 12_8 | All the way up is the direct question we started with, no wobble and no limit on what a yes reveals. | The marker moves to 1, immediately followed by the spinner, answer rows and table: every slice green; "yes: 8 of 8" and "yes: 0 of 8"; the 1 row reads "no bound" and 0. The preceding card keeps its full reading hold, then fades out before the next card fades in (PinkDuck F2). The card becomes: at 1 nobody who didn't copy says yes, so nothing bounds what a yes reveals; at 0 everyone says yes half the time, so no estimate exists | number line, table, lower-third | |
| 12_9 | All the way down, the second coin answers for everyone. Nothing is revealed, and nothing can be counted. | The marker moves to 0, every slice dim; "yes: 4 of 8" for both rows; the 0 row reads 1 and "no estimate" | number line, table | |
| 12_10 | In between, a survey counts a secret while every answer stays deniable, if each person answers once. | At the line's start the card becomes: if everyone follows the rule, with fair, private coins, once each; fresh coins on each repeat multiply the ratio (chapter 10). The marker returns to 1/2; the 1/4, 1/2 and 3/4 rows and the three misses flash. The picture holds 2 s after the line, and the card stays at least its reading time | highlight, lower-third | hold |

### Notes for 12

- The old chapter 05, pictures and cards unchanged but one reference: the
  12_10 card's "(chapter 4)" names asking again by its new number, "(chapter
  10)". 12_8's words add "we started with", the link back to chapter 01.
  "Miss by" is chapter 07's typical miss.
- The misses: about 2, 4 and 9 for the fixed class of 20 at 3/4, 1/2 and
  1/4 (`R-F16`); none is a chance that anyone copied.

### Pauses for 12 (semantic timing)

| where | kind | target |
|---|---|---|
| at 12_1 "the truth. \| Ours sits" | continuation | 0.55 s |
| at 12_3 after "from someone who copied?" | **prediction hold** | the kit's breath + the countdown |
| at 12_4 "Seven. \| Of the eight" | **reveal**, then its count | 0.7 s |
| at 12_6 "sharper. \| Our class" | continuation | 0.55 s |
| at 12_7 "a quarter, \| a yes" | comma, continuation | 0.35 s |
| at 12_9 "for everyone. \| Nothing" | continuation | 0.55 s |
| at 12_10 "In between" | turn to the conclusion | 0.6 s |

Units: 12u1 12_1 to 12_3 (ending on the question); 12u2 12_4 to 12_6; 12u3
12_7 to 12_9; 12u4 12_10.

## 13_running_one: Running one (~62 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | For programmers, here is the whole survey. Publish the rule and the recipe before anyone answers. | At the upper left, numbered steps in turn: "1. publish the rule and the recipe". A "for programmers" lower-third: two private fair coins each; answer = the truth if coin 1 is heads, else coin 2's face; estimate = 2 × yes − n/2, negatives kept | words, lower-third (code) | |
| 13_2 | Each person flips the coins privately and sends only yes or no, once. | "2. each person: two private fair coins, one yes or no, once" | words | |
| 13_3 | Take away a quarter of the class from the yeses, double what's left, and keep negative results. | "3. estimate = 2 × (yes − n/4), negatives kept" | words | |
| 13_4 | Its typical miss, the square root of three quarters of the class, doesn't depend on who copied, so it can be stated in advance. | "4. report it with its typical miss, typical, not a limit"; at the right TeX √(3n/4) and a small table, "n = 20: 3.87", "n = 300: 15", with "the same whoever copied" under it. Once read, the card becomes: fixed class, rule followed, once each: typical miss = the square root of 3n/4, any k, not a limit; sampling adds n × p(1 − p) to the variance | words, TeX, table, lower-third | |
| 13_5 | ⟨predict⟩ What do the private coins protect, and what does keeping the negatives protect? | Steps 2 and 3 flash; a "?" with the think ring at the right; no guard is shown | predict pause | |
| 13_6 | The private coins limit what each answer reveals. Keeping the negatives keeps the average right. Answering once keeps the three times from multiplying. | At the right, one at a time with the words: "private coins: each answer at most 3 times as likely either way", "negatives kept: right on average", "once each: no multiplying". They stand together for a moment (a named 1 s hold). Once read, the card becomes: private coins: at most 3 times as likely either way; negatives kept: right on average; one answer each: no multiplying | words, highlight, lower-third | |
| 13_7 | We began with a yes that was a confession. Now no answer is, and the count can still be estimated. | The steps and guards go. The question "Have you ever copied homework?" and the class come back, the students in the rule's on-average answers, no rings; under them, TeX 4 ± 3.87 and "the estimate and its typical miss: typical, not a limit". The picture holds 2 s and until the card is read | words, TeX | hold |

### Notes for 13

- The procedure states each condition once: private, fair, independent
  coins; one answer per person for this secret; the published recipe;
  negatives kept (`R-F14`); the typical miss √(3n/4), the same for every
  number of copiers in a fixed class (`R-F23`), so it can be stated before
  anyone answers; a sample adds its own term (`R-F8`).
- The ± display carries "typical, not a limit" (root's clarification 2):
  the recipe's ± is a standard deviation, not a confidence interval.
- Whether the coins make people more honest is not claimed either way
  (01_4).

### Pauses for 13 (semantic timing)

| where | kind | target |
|---|---|---|
| at 13_1 "the whole survey. \| Publish" | continuation | 0.55 s |
| at 13_4 "who copied, \| so it can" | consequence | 0.45 s |
| at 13_5 after "the negatives protect?" | **prediction hold** | the kit's breath + the countdown |
| at 13_6 "each answer reveals. \| Keeping" and "the average right. \| Answering" | list | 0.55 s, 0.55 s |
| at 13_7 "a confession. \| Now no answer is" | the turn back | 0.7 s |

Units: 13u1 13_1 to 13_5 (ending on the question); 13u2 13_6 and 13_7.
