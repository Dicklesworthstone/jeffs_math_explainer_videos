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
whom copied homework, and one coin rule defined before any formula.
Chapters 01 to 04 are storyboarded and built, and chapter 05, the last, is
written as source; every chapter's numbers come from the same model, facts
and oracle (`src/model.rs`, `src/facts.rs`, `oracle.py`).

| Chapter | Spine | Status |
|---|---|---|
| `01_the_coin` | a direct question exposes whoever says yes; the coin rule (heads, the truth; tails, a second coin answers); the class routed through it on average: half tells the truth (two yes, eight no), the second coin answers the other ten (five and five); the survey sees seven yes and thirteen no; take away the second coin's yeses on average, a quarter of the class, and double what is left: four; but a yes is three times as likely from someone who copied | committed at eb81fbc (`.8.1`). Root verified it on 2026-10-19 (Mail 3759): fmt, strict Clippy, 30 tests, release build, facts against the oracle, glyph pages, and a silent 540p30 scratch draft of 81.1 s with zero layout findings, all 11 cue-end frames viewed and the prediction answer hidden through its hold. Source reviewed (Mail 3741, four low findings fixed). Not narrated; no audio or full-video acceptance |
| `02_what_a_yes_says` | every student with all four coin outcomes, eighty in all; both answers' ratios of three counted from them; the prediction "a student says yes: what's the chance they copied?"; twelve of the twenty-eight yes outcomes are a copier's, three in seven (on average three of chapter 01's seven yeses), not three in four; why: copiers have three times the yeses but are outnumbered four to one; a no, four of fifty-two, one in thirteen; the chance moves from one in five, the coin fixing only the factor of three; deniable but not meaningless, and the count's accuracy is a separate question | `.8.2`, accepted by root on 2026-10-19 for source, native and sampled-visual criteria only (Mail 3831): fmt, strict release Clippy, 37 tests, the build and 105/105 facts against the oracle; a silent draft of 2,531 frames (84.37 s) with no layout findings; the 11 cue-end frames, 7 glyph pages, prediction and reveal, and both corrected cards viewed. Independently reviewed (TanElm, Mail 3802; repairs re-reviewed with no remaining findings, Mails 3827 and 3829). Not narrated; no audio or full-video acceptance |
| `03_the_price_of_the_coin` | the same class asked eight times with fresh coins (a seeded generator): estimates from minus four to fourteen, one on four; then every one of the 4²⁰ equally likely ways the coins can land: the yes counts and the estimates pile up around seven and four; the prediction "how often exactly on four?": about one class in five, and about one in eleven below zero; below zero means fewer yeses than the second coin alone gives on average, and rounding up would push the average above four; every answer wobbles the same, so twenty doubled miss by about four people (√15); that is the fixed class: sampled from a big school, who gets picked adds p(1 − p)/n to the coins' 3/(4n), and at one in five the same precision takes almost six times the people (91/16; the coins' part alone 75/16) | `.8.3`, accepted by root on 2026-10-19 for source and sampled silent visuals only (Mail 3980); root closes it after verifying the commit. Draft06 was first accepted provisionally (Mail 3932), then held: its sample card had 4.20 s for 143 characters, and the runs and clamp cards about 5 s for 126 and 117 (FuchsiaHawk, Mail 3941). The repair (Mails 3944 and 3952) brings the sample card in at 03_9, shortens the other two, and has the clamp card read "allow negatives: …". Checks: central17 (Mail 3916): fmt, strict release Clippy, 44 tests, the build, 123 facts against the oracle (31 checks), 105 glyph sources. Draft06 (Mail 3932), after the title-safe row-name move: 2,338 frames (77.93 s), no layout findings; cue ends, prediction and reveal, glyph pages, labels and closing card viewed. Central21 (Mail 3980): fmt, strict Clippy, the build, 5 targeted tests, 103 glyph sources; draft07: 2,338 frames (77.93 s), no layout findings, unchanged cues; frames at 12, 50, 60, 64 and 76 s viewed. Independently reviewed by FuchsiaHawk: the source (Mail 3899; its card defect fixed), the rendered images (Mail 3926) and draft07's cards (Mail 3965: the sample card readable for 15.90 s). Not narrated; no audio, continuous-playback or full-video acceptance |
| `04_asking_again` | one student asked three times with fresh coins: all 64 coin outcomes for each truth; the prediction "how many times as likely are three yeses from someone who copied?": 27, counted (27 outcomes against 1), and each fresh answer can multiply it by three, three noes the other way; in the class, where one in five copied, 108 three-yes outcomes against 16, so twenty-seven in thirty-one against one yes's three in seven; a majority right twenty-seven times in thirty-two, whichever the truth; one flip whose answer is repeated (RAPPOR's memoized answer) keeps three yeses at three times, for one unchanged secret under this rule | `.8.4`, closed by root on 2026-10-19 at 8d6f615 for source, mathematics and sampled silent visuals only (Mails 4053 and 4061). Central30: fmt, strict Clippy, 49 tests, 137 facts against the oracle, 131 glyph sources, the build and the render; draft09, 2,436 frames (81.20 s), no layout findings; 7 actual frames and all 13 glyph pages viewed. Independently reviewed: FuchsiaHawk, the mathematics (Mail 4026); EmeraldLeopard, the actual frames and card times (Mail 4049). Before the batch the 04_8 sentence was clarified (Mail 4028) and two strict Clippy findings fixed (Mail 4036). Not narrated; no audio or full-video acceptance |
| `05_the_dial` | the first coin becomes a dial, the chance t of telling the truth, ours at one half: a spinner of four slices over eight equally likely outcomes; the prediction "at three quarters, how many times as likely is a yes from someone who copied?": seven, 7 outcomes of 8 against 1, (1 + t)/(1 − t) for both answers; the count gets sharper, our class of twenty missing by about two people, not four (the coins' variance per person (1 − t²)/(4t²), 20 times it for the count); at a quarter a yes is under twice as likely but the class misses by about nine; the edges, the direct question (no wobble, no bound) and a pure coin (nothing revealed, nothing to count); the promise for one answer each, everyone following the rule | `.8.5`, accepted by root on 2026-10-19 for mathematics and sampled silent visuals only (Mail 4132); root closes the Bead. The first version's draft10 was withheld: TanElm's review (Mail 4095) found 05_6 and 05_7 speaking "wobble" in the per-person variance's sense, against chapter 03's miss size. The repair (Mail 4097) speaks the class's misses, about 2, not 4, and about 9. Central42: fmt, strict Clippy, 55 tests, 190 facts against the oracle, 175 glyph sources, the build and the render, all 20 input pins unchanged; draft11, 2,425 frames (80.83 s), silent 960×540, no layout findings, 10 changed frames viewed. Independently reviewed: the revised mathematics (Mail 4119) and the actual frames, phrase timing, card dwell and endpoints (Mail 4128). Not narrated; no audio, continuous-playback, 4K or full-video acceptance |

## Vocabulary (one word per idea)

**class** (the twenty; never "sample" or "population" in chapter 01's
voice); **copied** (the secret, the true answer; "copied homework");
**coin** and **second coin** (fair and private; the first is "a coin");
**heads** and **tails**; **tell the truth** (what heads means); **answer**
(what someone says, yes or no); **the survey** (who sees only the answers);
**on average** (every coin outcome equally often; never "expected" in the
voice); **outcome** (one of the four equally likely ways the two coins land,
from chapter 02); **the chance** (that a particular student copied, given
what is known: "one in five" before an answer, "three in seven" after a yes;
never "probability" or "posterior" in the voice); **three times as likely**
(an answer's ratio between the two truths; never "likelihood ratio" in the
voice). "Randomized response", "likelihood ratio", "posterior" and
"differential privacy" are not said; 02_11's "names to search" card shows
them. "Odds" appears only on the 02_10 card. From chapter 03:
**estimate** (the survey's count, double the yeses less half the class, as
chapter 01's code card named it; never "estimator"); **scatter** and
**wobble** (how far real classes stray; "variance" and "standard
deviation" only on cards); **this fixed class** (everyone in it answers)
against **sample** (twenty picked at random from a big school). From
chapter 04: **fresh coins** (new coins for every answer, independent of the
last); **flips only once** and **repeats that answer** (the memoized
answer; "memoized" only on cards); **the majority** (the answer given at
least twice in three). From chapter 05: **the dial** (the chance of telling
the truth, t; the only other "chance" in the voice, and always said in
full); **spinner** and **slices** (the dial drawn: so many equal slices say
tell the truth, the rest hand the answer to the second coin); **miss by**
(as 03_8: how far the fixed class's estimate usually strays; the coins'
variance per person, the count's and the share's appear only on cards and
in the table, never as "wobble" with a number).

## Color roles

| Idea | Color |
|---|---|
| copied, the truth only we can see: a ring on a student | `hero` gold `#FFD866`, strokes only, never a fill |
| a yes answer: its column and the students in it | blue `#58C4DD` |
| a no answer: its column and the students in it | violet `#C792EA` |
| heads, the truth: its lane and label | green `#83C167` |
| the second coin's answers: its lane, its coin, the outcomes it decides | `dim`, a voice that carries no information |
| a student before answering, the question, and the first coin | `ink` |
| the places answers land in, and the second coin's yeses once taken away (01_9) | `faint` |
| students we infer but cannot see (01_10's doubled count) | a hollow outline in the yes blue |
| an outcome before its answer is drawn, and an outcome set aside while the other answer is counted (chapter 02) | `faint` |
| the class's share who copied, one in five, on chapter 02's chance line | `hero` gold, a marker's stroke |
| chapter 03's distribution: a bar per yes count | `dim` fill; the bar of exactly four `ink` once revealed; the bars below zero `red`, the warn role |
| the true count, four, on chapter 03's axis | `hero` gold, a line's stroke and its words |
| chapter 04's outcome squares: faint until counted; then blue for the pattern counted among yeses, violet among noes; the rest faint again | `faint`, the yes blue, the no violet |

## 01_the_coin: The coin (~82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Ask a class of twenty if they've ever copied homework. Anyone who did has a reason to say no. | The question "Have you ever copied homework?" at the top. Twenty students, small ink discs, come in one by one in four rows of five on the left (`model::CLASS_SIZE`, `CLASS_COLUMNS`) | lagged reveal of the class | |
| 01_2 | Only we can see the truth. Four of them did. | On "Four of them did" gold rings close around the four who copied (`model::COPIERS`), lagged; a small gold ring and "copied: only we can see this" as a key under the class | lagged highlight | |
| 01_3 | So everyone flips a coin where nobody can see. Heads, tell the truth. | A coin right of the class. On "Heads" a green lane rises from it to the upper row, "heads: the truth", with two empty places at its end under the column heads "yes" (blue) and "no" (violet) | lanes drawn from the rule | |
| 01_4 | Tails, flip a second coin, and let it answer. Heads for yes, tails for no. | A dim lane, "tails", falls from the coin to the second coin on the lower row ("second coin" under it) and runs on to the lower row's two places. On "Heads for yes" the word "heads" appears in the lower yes place and flashes with the "yes" head; on "tails for no", "tails" in the lower no place with the "no" head | lanes drawn from the rule, highlight | |
| 01_5 | On average, half the class tells the truth. Two of our four say yes. | The ten students whose coins land heads (every coin outcome equally often: `Class::average_coins`) flash and travel up the green lane: two ringed students into the upper yes place, eight others into its no place, each recoloured by its answer. A "precisely" lower-third: fair, private coins; these are the average counts; a real class strays from them | motion along lanes, lagged | |
| 01_6 | The other ten get the second coin. Five yes, five no, whatever the truth. | The other ten travel down the dim lane through the second coin: five into the lower yes place (one of them ringed), five into its no place (one ringed). The counts 2, 8, 5, 5 stand by their places, checked against the facts' tally | motion along lanes, counts | |
| 01_7 | Seven yes, thirteen no. That's all the survey sees. | "7" over the yes column and "13" over the no column. On "That's all the survey sees" the lanes, coins, their words, the places and their counts, the rings and the key fade: two columns of colored students and two numbers are left | highlight, fade to the survey's view | |
| 01_8 | ⟨predict⟩ From those two numbers, how could the survey get back to four? | On "two numbers" the two totals flash; a "?" with the think ring and "pause for more time" on the left, where the class sat, the countdown | predict pause | |
| 01_9 | Take away the second coin's yeses. On average that's five, a quarter of the class. Two are left. | The "?" goes and the lower yes place's five students dim to faint. On the left the arithmetic is written in TeX: on "On average that's five", 20/4 = 5; on "Two are left", 7 − 5 = 2, and the upper place's two students flash | TeX derivation, highlight | |
| 01_10 | Those two came from the half who told the truth. So double them. Four. | On "So double them", beside the two, two hollow blue outlines appear (the copiers among the other half, whom the second coin hid) and "× 2"; on "Four" the result 2 × 2 = 4 under the arithmetic and the four gold rings come back on the students who really copied: two in the upper yes place, one in each lower place. A "for programmers" lower-third: estimate = 2 * yes - n / 2, in real arithmetic; it can fall outside 0 to n (no yeses give −10, twenty give 30). The picture holds 1.5 s after the line for the card | TeX derivation, lower-third (code) | hold |
| 01_11 | No answer is a confession. But someone who copied says yes three times as often as someone who didn't. | The class clears. Two rows of the four coin outcomes, heads heads, heads tails, tails heads, tails tails, each cell coloured by the answer it gives: "copied" has three blue and one violet, "didn't" one blue and three violet; "3 of 4" and "1 of 4" beside them, then "× 3". A "precisely" lower-third: both answers' ratios. The picture holds 2 s after the line, then the chapter fades | grid built from the rule, highlight, lower-third | hold |

### Notes for 01

- The rule is on screen before any number is derived (01_3, 01_4); every
  later count is the rule run on the class (`Tally::of`), never typed.
- "On average" (01_5, 01_9) means every one of the four coin outcomes lands
  equally often within the copiers and within the others
  (`Class::average_coins`; a class whose groups are not multiples of four is
  refused). The 01_5 lower-third keeps the exact version; the scatter of a
  real class is chapter 03's (`R-F7`).
- Prediction (01_8): the viewer already knows the truth (the rings), so the
  question is the survey's method, not the number. The question is on screen
  by its last word; four seconds of silence with the countdown; the answer
  starts with 01_9.
- 01_10's "Four" is the estimate on the average counts; the rings returning
  show it matches the class's truth here because the coins landed on
  average. 01_11 keeps the kill criterion: a yes is not harmless, it is
  three times as likely from someone who copied (`R-F5`).
- Exact versions kept on screen: 01_5 (fair private coins; average counts; a
  real class strays), 01_10 (the estimate's code and its edges: `R-X2`,
  −10 at no yeses, 30 at twenty), 01_11 (a yes is 3 of 4 coin outcomes from
  someone who copied against 1 of 4, a no the reverse; 3 times, never more,
  `R-X1`).
- Budget: 158 words at 2.6 a second plus 0.4 s a line, thought gaps 0.82 s,
  the think window 5.2 s, a 2 s tail: about 81 s, before animation waits.

### Pauses for 01 (semantic timing)

Targets are the **total** silence the ear should hear at each sentence end
inside a line, including both sides' own quiet; inserted silence is the
target less the take's measured gap (narration skill §3), never added on
top. Between lines the kit's thought gaps apply (`LineGaps::Thought`),
except where a row says otherwise.

| where | kind | target |
|---|---|---|
| at 01_1 "homework. \| Anyone" | continuation | 0.55 s |
| at 01_2 "the truth. \| Four" | reveal (the rings close on "Four") | 0.65 s |
| at 01_3 "nobody can see. \| Heads" | rule begins | 0.5 s |
| at 01_3 → 01_4 "Heads, tell the truth. \| Tails" | contrast pair: keep the two halves of the rule parallel | the kit's continuation gap |
| at 01_4 "let it answer. \| Heads for yes" | continuation | 0.45 s |
| at 01_5 "tells the truth. \| Two of our four" | continuation, the picture moving | 0.55 s |
| at 01_6 "the other ten. \| Five yes" | continuation | 0.45 s |
| at 01_7 "thirteen no. \| That's all" | turn to the survey's view | 0.6 s |
| at 01_8 after "get back to four?" | **prediction hold**, not a question/answer pause: a plain falling wh-question read, then the four-second countdown (`think`); no added rise, no extra silence before the countdown | the kit's breath + the countdown |
| at 01_9 "coin's yeses. \| On average" and "the class. \| Two are left" | steps of one calculation | 0.5 s, 0.55 s |
| at 01_10 "told the truth. \| So double them" | continuation | 0.5 s |
| at 01_10 "double them. \| Four." | **reveal**: the answer lands alone | 0.9 s |
| at 01_10 → 01_11 | after the reveal, a new thought | the kit's beat gap; the class clears in it |
| at 01_11 "a confession. \| But someone" | contrast | 0.7 s |

The narration units (`narration_units.json`) keep each thought in one take:
01u1 is the setup and the rule (01_1 to 01_4), 01u2 the class routed and the
survey's question (01_5 to 01_8, ending on the question so the take ends
cleanly before the countdown), 01u3 the answer and the turn (01_9 to 01_11).
They are text inputs only: the owner rejected the first wave's narration on
2026-10-19 (AGENTS.md), and nothing here is to be synthesized with that
method until it is requalified.

## 02_what_a_yes_says: What a yes says (~82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Give every student all four coin outcomes, equally likely. Twenty students, eighty outcomes. | Chapter 01's twenty students as a row of discs across the top: the four who copied first, ringed in gold under "copied", then after a gap the sixteen others under "didn't copy". Under each disc a column of four faint squares appears, lagged row by row, one per coin outcome, rows labelled H H, H T, T H, T T on the left. On "Twenty students", 20 × 4 = 80 (TeX) on the right | grid built from the rule, lagged | |
| 02_2 | Someone who copied says yes in three of their four. Someone who didn't, in one. Three times as likely. | On "Someone who copied" each copier's squares take their answers' colours, the rule run on each outcome: three blue, one violet; "yes: 3 of 4" under the copiers. On "Someone who didn't", the others' squares: one blue, three violet each; "yes: 1 of 4" under them. On "Three times as likely", "× 3" at the right | grid built from the rule, highlight | |
| 02_3 | For a no, it's the reverse, three times again. | The labels become "no: 1 of 4" and "no: 3 of 4"; the violet squares flash, one per copier, three per other; the "× 3" flashes | highlight | |
| 02_4 | ⟨predict⟩ So a student says yes. What's the chance they copied? | The labels go back to the yes counts per student; a "?" with the think ring and "pause for more time" right of the grid, the countdown. No total of yes squares is on screen | predict pause | |
| 02_5 | Count the yes outcomes. The four who copied have twelve, the sixteen others sixteen. | The "?" goes and every violet square dims to faint. On "have twelve" the copiers' blue squares flash and "12" replaces their label; on "the sixteen others" the others' blue squares flash and "16" replaces theirs | highlight, counts | |
| 02_6 | Twelve of twenty-eight. On average, three of the seven yeses. Three in seven, not three in four. | Under the grid, 12/28 (TeX); on "On average", "= 3/7" and, beside it, "a quarter of the outcomes: 3 of chapter 1's 7 yes answers"; on "not three in four", on the right "3/4 is a copier's chance of saying yes". A "precisely" lower-third: 3/7 is the chance that someone who said yes copied, given this class; it would be 3/4 only if half the class had copied | TeX derivation, lower-third | |
| 02_7 | Each copier has three times the yeses, but the others outnumber them four to one. | The derivation and card stay; the copiers' column labels flash "3 each", the others' "1 each"; then the two groups of discs flash, "4" and "16" over them | highlight | |
| 02_8 | A no works the same way. Four of the fifty-two no outcomes are a copier's, one in thirteen. | The labels and derivation go; the blue squares dim and the violet ones light again. On "Four of the fifty-two" the copiers' violet squares flash with "4" under them and "48" under the others'; 4/52 = 1/13 (TeX) under the grid | highlight, TeX derivation | |
| 02_9 | A yes lifts the chance from one in five to three in seven. A no drops it to one in thirteen. | The derivation goes. A chance line from 0 to 1 under the grid: a gold mark at 1/5, "the class: 4 of 20"; on "three in seven" a blue mark at 3/7 and a blue arc from 1/5 to it; on "one in thirteen" a violet mark at 1/13 and a violet arc | number line, motion | |
| 02_10 | The coin fixes the three times. Where the chance ends up depends on the class. | The arcs flash. A "precisely" lower-third: an answer multiplies the odds by 3 for a yes, 1/3 for a no; here 1 : 4 becomes 3 : 4 or 1 : 12. If nobody copied, a yes would mean 0. (The chances 3/7 and 1/13 those odds give are the marks already on the line, and 02_6's card has the half-class 3/4; the card stays two lines at every wrap the band can fall back to, 92 down to 72) | lower-third, highlight | |
| 02_11 | An answer is deniable, but not meaningless. How well the survey counts is a separate question. | The marks stay. On "deniable" the 3/7 mark flashes (far from 1); on "not meaningless" the blue arc flashes (far from 1/5). On "How well the survey counts", a "names to search" lower-third: likelihood ratio, posterior probability, randomized response, differential privacy. The picture holds 2 s after the line | lower-third, highlight | hold |

### Notes for 02

- Every square is the rule run on one student and one outcome
  (`model::class_outcomes`); its colour is that answer. The counts drawn
  are checked against the facts (`R-F11`) before they are labelled.
- The likelihood ratio is shown before the posterior and kept apart from
  it: 02_2 and 02_3 compare one student's four outcomes per truth (3 of 4
  against 1 of 4, both answers), while 02_5 to 02_9 compare the whole class's
  outcomes per answer. The ratio is the coin's; the chance needs the class.
- Prediction (02_4): the tempting answer is 3/4, read straight off a copier's
  column. Nothing on screen totals the yes squares before 02_5, and 3/7 first
  appears with 02_6. The question is a plain falling wh-question, then the
  four-second countdown.
- The average-count reading (02_6, `R-F11.average.*`): the class's 80
  outcomes are 20 students times every outcome once, so a quarter of each
  count is chapter 01's class on average: 28/4 = 7 yes answers, 12/4 = 3 of
  them from copiers; 52/4 = 13 noes, 4/4 = 1 from a copier.
- Exact versions kept on screen: 02_6 (3/7 is the chance given this class; 3/4
  would be right only at half the class, `R-F12.posterior.yes_at_half`),
  02_10 (odds 1 : 4 → 3 : 4 or 1 : 12, their chances being the line's marks;
  the edge, nobody copying, `R-X5`). Both cards stay two lines at every
  wrap the lower-third can narrow to, 92 down to 72 characters
  (`the_cards_are_two_lines`).
- 02_11 keeps privacy and accuracy apart: the factor of three is what one
  answer can say about its own giver; how close the class count comes is
  the planned chapter 03 (`R-F7`, `R-F8`). Neither is said to imply the other.
- Budget: 163 words at 2.6 a second plus 0.4 s a line, thought gaps 0.82 s,
  the think window 5.2 s, a 2 s hold and tail: about 84 s before animation
  waits. Root's silent draft measured 84.37 s (2,531 frames, Mail 3831),
  0.63 s under the cap. Any line or wait added later must be paid for:
  first shorten the scene's own waits (02_1's lagged squares, 02_7's
  flashes), then fold 02_3 into 02_2; 02_7's "why" stays.

### Pauses for 02 (semantic timing)

Targets are total silence, as for chapter 01.

| where | kind | target |
|---|---|---|
| at 02_1 "equally likely. \| Twenty students" | continuation | 0.55 s |
| at 02_2 "three of their four. \| Someone who didn't" | contrast | 0.5 s |
| at 02_2 "in one. \| Three times as likely" | conclusion | 0.6 s |
| at 02_4 "says yes. \| What's the chance" | set-up, then the question | 0.45 s |
| at 02_4 after "they copied?" | **prediction hold**: a plain falling wh-question, then the countdown; no added rise | the kit's breath + the countdown |
| at 02_5 "the yes outcomes. \| The four who copied" | continuation | 0.5 s |
| at 02_6 "twenty-eight. \| On average" | continuation | 0.55 s |
| at 02_6 "seven yeses. \| Three in seven" | **reveal**, and its contrast "not three in four" kept in the same breath | 0.8 s |
| at 02_8 "the same way. \| Four of the" | continuation | 0.5 s |
| at 02_9 "three in seven. \| A no drops it" | contrast | 0.6 s |
| at 02_10 "the three times. \| Where the chance" | contrast | 0.6 s |
| at 02_11 "not meaningless. \| How well" | turn to a separate question | 0.8 s |

Units: 02u1 is the outcomes, both ratios and the question (02_1 to 02_4,
ending on the question); 02u2 the count, the answer and the no (02_5 to
02_8); 02u3 the chance line and the turn (02_9 to 02_11). Text inputs only,
as for chapter 01, and unqualified proposals until there is listening
evidence. Two choices remain open questions rather than settled
segmentation. One is the request boundary between the prediction question
(end of 02u1) and its answer (start of 02u2). The other is 02u2's length,
64 words (TanElm, Mail 3802). Neither is changed without auditory evidence.

## 03_the_price_of_the_coin: The price of the coin (~84 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Real coins don't land on average. Ask the class eight times with fresh coins. | On "Ask the class" eight rows appear one after another, each the same twenty students as small discs coloured by the answer their coins gave this time, the four who copied ringed in gold; each row's yes count at its right under "yes". The coins are real draws (`facts::runs`) | lagged rows built from the rule | |
| 03_2 | Their estimates run from minus four to fourteen. Only one lands on four. | Each row's estimate appears under "estimate" (−4, 0, 14, 10, 8, 4, −2, 6); on "minus four to fourteen" the two extremes flash; on "Only one lands on four" the one 4 flashes. A "precisely" lower-third: the first eight draws from PCG64DXSM, seed 3, none thrown away; too few to show the rates (short: it has about 5 s; Mail 3944) | highlight, lower-third | |
| 03_3 | Now count every way the coins can land. Four to the twentieth, over a trillion, all equally likely. | The rows and card go. A two-row axis appears across the lower half: yes counts 0 to 20 above, the estimates −10 to 30 below, each estimate under its yes count. On "Four to the twentieth" 4²⁰ = 1,099,511,627,776 (TeX) at the upper right, and the bars rise left to right, one per yes count, each as tall as its exact chance | histogram grown from the distribution, TeX | |
| 03_4 | The estimates pile up around four, the true count. | A gold line at estimate 4 (seven yeses), "4 copied" above it; the bars beside it flash | highlight | |
| 03_5 | ⟨predict⟩ How often does a class land exactly on four? | The 4²⁰ total goes; a "?" with the think ring and "pause for more time" at the right. No bar is labelled with a chance yet, and the axis has no chance scale | predict pause | |
| 03_6 | About one class in five. And about one in eleven goes below zero. | The "?" goes; on "one class in five" the bar at 4 turns ink and "exactly 4: about 1 class in 5 (20%)" appears at the upper left; on "below zero" the bars of estimates −10 to −2 turn red, "below 0 (under 5 yeses): about 1 in 11 (9%)". A "precisely" lower-third: over all 1,099,511,627,776 equally likely ways, exactly 4 about 20% of the time, below 0 about 9% | highlight, lower-third | |
| 03_7 | Below zero means fewer yeses than the second coin alone gives on average. Rounding those up to zero would push the average above four. | The red bars (yes counts under 5) flash. On "Rounding those up", "rounded up to 0, the average would be 4.26" at the upper left; the card becomes: allow negatives: 2 × yes − 10 averages 4; replace them with 0: the average is 4.26 (short: it has about 5 s; Mails 3944 and 3952) | highlight, lower-third | |
| 03_8 | Every answer wobbles the same, whoever gives it. Twenty of them, doubled, miss by about four people. | The notes go. At the right, in TeX: 3/16 × 20 = 15/4, then 15/4 × 4 = 15, then √15 ≈ 3.87; on "miss by about four people" a white bracket over the bars from 4 − 3.87 to 4 + 3.87, labelled √15 ≈ 3.87. A "precisely" lower-third: each answer's variance is 3/16 for anyone; 20 make 15/4; doubling the count multiplies variance by 4: 15. Standard deviation: about 3.87 (doubling multiplies the variance by 4, the derivation's ×4, not by 2; Mail 3899) | TeX derivation, lower-third | |
| 03_9 | That's for this fixed class. Sample twenty from a big school instead, and who gets picked adds its own scatter. | The bracket stays. On "Sample twenty" the derivation and the fixed class's card go. In their place comes the variance of the estimated share for independent draws: p(1 − p)/n + 3/(4n) (TeX), labelled above "variance of the estimated share" (03_8 was the count's scale, this is the share's), with "who gets picked, plus the coins" under it. A "precisely" lower-third, which stays to the end (the sample's assumptions, given time to read; Mail 3944): assumes independent draws, with replacement or from a large population, and truthful direct answers; at p = 1/5 that is 91/16, about 5.69 times | TeX, formula, lower-third | |
| 03_10 | If one in five copied, matching a direct question's precision takes almost six times as many people. | On "almost six times", 91/16 ≈ 5.69 (TeX) under the formula, "times as many people, for the same precision" under it; the card stays. The picture holds 2 s after the line | TeX | hold |

### Notes for 03

- The eight classes are real draws, not a choice: one raw 64-bit draw per
  class from `explainer_kit::stats::Data::new(3)` (NumPy's
  `PCG64DXSM(3).random_raw()`), student i's two coins in bits 2i and 2i + 1
  (`model::coins_from_bits`). Seed 3 is the chapter's number, fixed before
  any draw was looked at and the only seed tried. Two of the eight land
  below zero, against 9% over all outcomes: eight are too few to show
  rates, and the card says so. The oracle redraws them with NumPy.
- Every bar is the exact chance of its yes count over all 4²⁰ equally
  likely coin configurations (`model::yes_count_distribution`, already
  oracle-checked as `R-F7`); its estimate, 2 · yes − 10, is chapter 01's
  `model::undo`. The mean of the estimate is exactly 4 (`R-F14.estimate_mean`).
- Prediction (03_5): the axis has no chance scale and no bar is labelled
  before 03_6; the answer, about one class in five (20%), appears only after
  the countdown.
- Below zero, honestly (03_7): the estimate is negative exactly when the yes
  count is under 5, the second coin's yeses on average (`R-F4.forced_yes`).
  It is kept, not clamped: rounding the negatives up to 0 raises the
  average to 4.26 (`R-F14.clamped_mean_2dp`), so the clamped estimate is no
  longer right on average.
- The fixed class and a sample are kept apart (03_8 to 03_10). The class's
  scatter is the coins' alone: each answer's variance 3/16 whoever gives it
  (`R-F14.answer_variance`), the count's 15/4, the estimate's 15, standard
  deviation √15 ≈ 3.87. Twenty drawn independently from a big school (with
  replacement, or a population large enough not to matter) add p(1 − p)/n
  to the share's variance; drawing the whole class without replacement adds
  nothing. "Almost six times as many people" is the total, 91/16 ≈ 5.69,
  against direct answers assumed truthful; the coins' part alone is 75/16.
  No claim is made about practical privacy guarantees or real surveys'
  honesty.
- Budget: 152 words, 10 lines; chapter 02's 163 words and 11 lines
  measured 84.37 s. The bars' growth and the eight rows are the scene's
  longest waits; shorten those first if the draft runs over 85 s.

### Pauses for 03 (semantic timing)

| where | kind | target |
|---|---|---|
| at 03_1 "on average. \| Ask the class" | turn from the claim to the test | 0.6 s |
| at 03_2 "to fourteen. \| Only one" | continuation | 0.55 s |
| at 03_3 "can land. \| Four to the twentieth" | continuation | 0.5 s |
| at 03_5 after "exactly on four?" | **prediction hold**: a plain falling wh-question, then the countdown | the kit's breath + the countdown |
| at 03_6 "one class in five. \| And about one in eleven" | **reveal**, then a second fact | 0.7 s |
| at 03_7 "on average. \| Rounding those up" | continuation | 0.6 s |
| at 03_8 "whoever gives it. \| Twenty of them" | continuation | 0.55 s |
| at 03_9 "this fixed class. \| Sample twenty" | contrast | 0.7 s |

Units (unqualified proposals, text only): 03u1 the eight classes (03_1,
03_2); 03u2 every outcome and the question (03_3 to 03_5, ending on the
question); 03u3 the answer and below zero (03_6, 03_7); 03u4 the scatter and
the sample (03_8 to 03_10).

## 04_asking_again: Asking again (~82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Ask one student the same question three times, with fresh coins each time. | One student (an ink disc, no ring: we do not know their truth). On "three times" three answer places appear one by one to its right, "ask 1", "ask 2", "ask 3"; on "fresh coins" each gets its own two coins, the first ink, the second dim | lagged places, coins | |
| 04_2 | Four coin outcomes for each answer make sixty-four for all three, all equally likely. | The asks go. Two grids of 64 squares, "copied" (gold) on the left and "didn't copy" on the right, every square faint: one square per way the six coins can land, in 2 × 2 blocks of 4 × 4 (the first ask's coins pick the block, the second's the row, the third's the column). On "sixty-four", 4 × 4 × 4 = 64 (TeX) at the upper right. A "precisely" lower-third: each square is one way all 6 coins can land, 64 equally likely. Block: the first answer's coins; row: the second's; column: the third's | grid built from the rule, TeX, lower-third | |
| 04_3 | ⟨predict⟩ How many times as likely are three yeses from someone who copied? | A "?" with the think ring and "pause for more time" at the right. No square is coloured yet, and nothing counts the yeses | predict pause | |
| 04_4 | Twenty-seven. Someone who copied says yes three times in twenty-seven outcomes, anyone else in one. | The "?" goes. In the left grid the 27 squares whose three answers are all yes turn blue: three 3 × 3 blocks, 3 × 3 × 3. Under it "3 yes: 27 of 64". On "anyone else in one" the one such square on the right turns blue, "3 yes: 1 of 64", and 27/1 = 27 (TeX) at the right | highlight, TeX | |
| 04_5 | Each fresh answer can multiply it by three. Three noes do the same the other way. | 27 = 3 × 3 × 3 (TeX) under the ratio. On "Three noes" the all-no squares turn violet, 1 on the left and 27 on the right, "3 no: 1 of 64" and "3 no: 27 of 64" under them. The card becomes: 0 to 3 yeses are 1/27, 1/3, 3, 27 times as likely from someone who copied; the most after 1 to 5 fresh answers: 3, 9, 27, 81, 243 | highlight, TeX, lower-third | |
| 04_6 | In our class, one in five copied. Those four have a hundred and eight such outcomes, the sixteen others sixteen. | The violet squares and the right-hand TeX go. The grid names become "4 copied" and "16 didn't copy". On "a hundred and eight" the left grid's blue squares flash and 4 × 27 = 108 (TeX) appears under it; on "the sixteen others" the right grid's one blue square flashes and 16 × 1 = 16 | highlight, TeX | |
| 04_7 | So after three yeses, the chance they copied is twenty-seven in thirty-one. After one, three in seven. | On "twenty-seven in thirty-one", 108/(108 + 16) = 27/31 (TeX) at the right, and the card becomes: 27/31 is the chance in this class, where 4 of 20 copied; from an even 1/2 the same 3 yeses would leave 27/28, and if nobody copied, 0. On "After one", "after one yes: 3/7" under the fraction in dim | TeX fraction, lower-third | |
| 04_8 | The majority of three answers is right twenty-seven times in thirty-two, whichever the truth. | The products go and the grid names return to "copied" and "didn't copy". On the left the squares with at least two yeses turn blue, on the right those with at least two noes violet, the rest faint: "majority right: 54 of 64" under each. On "twenty-seven times", 54/64 = 27/32 (TeX) at the right; on "whichever the truth" both counts flash | highlight, TeX | |
| 04_9 | But suppose the student flips only once, and repeats that answer. | On "flips only once" the grids and the right-hand TeX go. In their place, for each truth, four columns, one per outcome of the one flip (H H, H T, T H, T T over them), each column three squares of the same answer: on the left three columns blue and one violet, on the right one blue and three violet. The card becomes: memoized: one flip for this secret, its answer resent each ask, so repeats add no evidence; only for one unchanged secret, under this rule | grid built from the rule, lower-third | |
| 04_10 | Then three yeses are three times as likely, the same as one. The repeats add nothing. | "3 yes: 3 of 4" and "3 yes: 1 of 4" under the columns; 3/1 = 3 (TeX) at the right; on "the same as one" it flashes, and "fresh coins: 27" appears under it in dim | TeX, highlight | |
| 04_11 | That holds only while the secret stays the same, under this rule. | The columns and the card stay; the grid names flash. The picture holds 2 s after the line | highlight | hold |

### Notes for 04

- One student asked three times with fresh, independent coins: 4³ = 64
  equally likely outcomes of the six coins (`R-F9.outcomes`), every square
  `model::answer` run on one truth and one outcome, tallied against
  `model::repeated_answers` before any count is drawn. By number of yeses,
  0 to 3, someone who copied has 1, 9, 27, 27 of them and anyone else 27,
  27, 9, 1 (`R-F9.by_yeses.*`); three yeses are 27 against 1, three noes 1
  against 27 (`R-F9.all_yes.*`, `R-F9.all_no.*`).
- Prediction (04_3): no square is coloured and no count of yeses is on
  screen until 04_4. Chapters 01 and 02 said "three times as likely" for
  one answer; the tempting answers are three (nothing changes) and nine
  (three added three times); the counted answer is 27 = 3³.
- "Can multiply it by three": the largest ratio over all answer patterns
  after 1 to 5 fresh answers is 3, 9, 27, 81, 243 (`R-F9.worst_ratio_by_asks`);
  mixed answers partly cancel (two yeses and a no: 3).
- The chance after three yeses is the class's, counted as chapter 02 counts
  it: every student with every outcome, the copiers' 4 × 27 = 108 three-yes
  outcomes against the others' 16 × 1 = 16, so 27/31 (`R-F15.all_yes.*`,
  `R-F9.posterior.all_yes`; the closed form checks it). It depends on the
  class: from an even prior 27/28, if nobody copied 0
  (`R-F15.posterior.*`). After one yes it was 3/7 (`R-F11.posterior.yes`).
- The majority of three answers is right for someone who copied in 54 of
  64 outcomes and for anyone else in 54 of 64, 27/32 either way
  (`R-F9.majority.outcomes`, `R-F9.majority_right`, and `.didnt`).
- A memoized answer (RAPPOR's permanent randomized response): the two
  coins are flipped once for this secret and the answer is sent again on
  every ask. Its outcomes are 4, not 64, and only all-yes or all-no
  happen; three yeses are 3 against 1, the same as one yes
  (`R-F10.memoized.*`), and in the class they leave 3/7, one yes's chance.
  The chapter claims this only for one secret that does not change, asked
  under this rule. It does not say how repeated surveys behave in general,
  or what RAPPOR's full design guarantees.
- Budget: 160 words, 11 lines; chapter 03's 152 words and 10 lines measured
  77.93 s silent. The 128 squares fade in lagged inside 04_2's line.

### Pauses for 04 (semantic timing)

| where | kind | target |
|---|---|---|
| at 04_1 "three times, \| with fresh coins" | continuation | 0.5 s |
| at 04_3 after "from someone who copied?" | **prediction hold**: a plain falling wh-question, then the countdown | the kit's breath + the countdown |
| at 04_4 "Twenty-seven. \| Someone who copied" | **reveal**, then its count | 0.7 s |
| at 04_6 "one in five copied. \| Those four" | continuation | 0.55 s |
| at 04_7 "thirty-one. \| After one" | contrast | 0.6 s |
| at 04_9 "But suppose" | turn from fresh coins to one flip | 0.6 s |
| at 04_10 "the same as one. \| The repeats" | continuation | 0.55 s |

Units (unqualified proposals, text only): 04u1 the asks and the question
(04_1 to 04_3, ending on the question); 04u2 the answer and the factor
(04_4, 04_5); 04u3 the class's chance and the majority (04_6 to 04_8); 04u4
one flip, and what it covers (04_9 to 04_11).

## 05_the_dial: The dial (~82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Make the first coin a dial, the chance of telling the truth. Ours sits at one half. | A number line across the top left, "the chance of telling the truth", ticks 0, 1/4, 1/2, 3/4, 1, a green marker at 1/2. On "Ours sits at one half": a spinner of four slices over a row of eight outcome squares (each slice over its two second-coin faces, H and T), two slices green (tell the truth) and two dim (the second coin); two rows of eight squares, "copied" and "didn't copy", coloured by their answers, "yes: 6 of 8" and "yes: 2 of 8". At the right a table: the dial, times as likely, coins' variance; the 1/2 row reads 3 and 3/4 | number line, grid built from the rule, table | |
| 05_2 | Set it to three quarters, a spinner of four slices, three saying tell the truth. | On "three quarters" the marker moves to 3/4, a third slice turns green, the squares go faint and their counts go; the table gains a 3/4 row with its ratio and variance blank. On "a spinner" a "precisely" lower-third: a spinner of 4 equal slices, 3 saying tell the truth and 1 handing the answer to the fair second coin: 4 × 2 = 8 equally likely outcomes | number line, lower-third | |
| 05_3 | ⟨predict⟩ At three quarters, how many times as likely is a yes from someone who copied? | A "?" with the think ring and "pause for more time" at the lower right. The squares are faint and the 3/4 row has no ratio | predict pause | |
| 05_4 | Seven. Of the eight outcomes, someone who copied says yes in seven, anyone else in one. | The "?" goes; the squares take their answers' colours, "yes: 7 of 8" and "yes: 1 of 8"; the 3/4 row's ratio reads 7. The card becomes: at 3/4 a yes is 7 of 8 outcomes for someone who copied, 1 of 8 for anyone else, and a no the reverse: 7 times either way, (1 + t)/(1 − t) | highlight, table, lower-third | |
| 05_5 | Turning the dial up makes every answer say more. | Under the table a general row: t, (1 + t)/(1 − t) (TeX); the ratios 3 and 7 flash | TeX formula, highlight | |
| 05_6 | But the count gets sharper. Our class of twenty would miss by about two people, not four. | On "miss by about two" the 3/4 row's variance reads 7/36 and the general row gains (1 − t²)/(4t²) (TeX). Under the outcome rows, "our class of 20 misses by" with two lines (TeX): 1/2: √(20 × 3/4) ≈ 3.87 and 3/4: √(20 × 7/36) ≈ 1.97; both flash. The card becomes: per person (1 − t²)/(4t²); class of 20: count variance 20 × it, miss about its square root, share variance it/20; a sample adds p(1 − p)/n | TeX formula, highlight, lower-third | |
| 05_7 | Down at a quarter, a yes is under twice as likely, but the class would miss by about nine. | The marker moves to 1/4, two green slices turn dim, the squares recolour, "yes: 5 of 8" and "yes: 3 of 8"; the table gains the 1/4 row, 5/3 and 15/4. On "about nine" a third line, 1/4: √(20 × 15/4) ≈ 8.66, and it and the 1/2 line flash | number line, table, highlight | |
| 05_8 | All the way up is the direct question, no wobble and no limit on what a yes reveals. | The marker moves to 1, every slice green; "yes: 8 of 8" and "yes: 0 of 8"; the 1 row reads "no bound" and 0. The card becomes: at 1 nobody who didn't copy says yes, so nothing bounds what a yes reveals; at 0 everyone says yes half the time, so no estimate exists | number line, table, lower-third | |
| 05_9 | All the way down, the second coin answers for everyone. Nothing is revealed, and nothing can be counted. | The marker moves to 0, every slice dim; "yes: 4 of 8" for both rows; the 0 row reads 1 and "no estimate" | number line, table | |
| 05_10 | In between, a survey counts a secret while every answer stays deniable, if each person answers once. | At the line's start the card becomes: if everyone follows the rule, with fair, private coins, once each; fresh coins on each repeat multiply the ratio (chapter 4). The marker returns to 1/2; the 1/4, 1/2 and 3/4 rows and the three misses flash. The picture holds 2 s after the line, and the card stays at least its reading time | highlight, lower-third | hold |

### Notes for 05

- The dial is `model::Channel` at t = k/4, the first coin replaced by a
  spinner of four equal slices, k of them saying tell the truth and the
  rest handing the answer to the fair second coin. Every setting has the
  same 4 × 2 = 8 equally likely outcomes (`R-F16.outcomes`). The squares
  are the rule run on each outcome, tallied against `Channel::p_said`
  before a count is drawn; the table's numbers are `Channel::ratio` and
  `Channel::coin_variance`. The spinner only replaces this rule's first
  coin. It is not Warner's spinner, which chooses between two questions
  (README, "Sources").
- Both answers carry the same ratio, (1 + t)/(1 − t): at 3/4 a yes is 7/8
  against 1/8 and a no 7/8 against 1/8 the other way (`R-F16.up.*`). The
  facts and the oracle check both answers and the closed form at every
  setting.
- Prediction (05_3): the 3/4 row has no ratio and the squares are faint
  until 05_4. The tempting answer is 3 (the slices' 3 to 1); counted, a
  copier says yes in 7 of the 8 outcomes and anyone else in 1.
- The voice speaks of misses, as 03_8 did ("miss by about four people"):
  how far the fixed class of 20's estimate usually strays, the square root
  of its count's variance. That variance is 20 times the coins' variance
  per person, (1 − t²)/(4t²) (`R-F16.*.coin_variance`). At 1/2 it is
  20 × 3/4 = 15, a miss of √15 ≈ 3.87, about four. At 3/4 it is
  20 × 7/36 = 35/9, a miss of 1.97, about two. At 1/4 it is 20 × 15/4 = 75,
  a miss of 8.66, about nine (`R-F16.*.class_variance`, `.miss_2dp`,
  `.miss_rounded`, the rounding exact). The estimated share's variance is
  the per-person one over 20 (3/80 at 1/2), and drawing a sample adds
  p(1 − p)/n to it (chapter 03). The per-person ratios 7/27 and 5 are
  variances, not misses, and are not spoken. None of these is a chance
  that anyone copied.
- Every card stays up at least its reading time, 15 characters a second
  from the moment it is fully in, over the lines after it; the last card
  holds past the 2 s if it needs to.
- The edges: at 1 the direct question, coin variance 0 and no ratio bound
  (a yes never comes from someone who didn't copy); at 0 a pure coin, ratio
  1 and no estimate (nobody tells the truth, so nothing is left to divide
  by) (`R-F16.direct.*`, `R-F16.pure.*`, `R-X1.*`).
- The ending restates the video's promise for one answer each. Its card
  names every condition: everyone follows the rule, with fair, private
  coins, once each. It also repeats chapter 04's warning about fresh coins
  on every ask. No claim is made about real surveys' honesty, about
  repeated surveys in general, or about RAPPOR's guarantees.
- Budget: 161 words, 10 lines; chapter 04's 160 words and 11 lines
  measured 81.20 s silent.

### Pauses for 05 (semantic timing)

| where | kind | target |
|---|---|---|
| at 05_1 "the truth. \| Ours sits" | continuation | 0.55 s |
| at 05_3 after "from someone who copied?" | **prediction hold**: a plain falling wh-question, then the countdown | the kit's breath + the countdown |
| at 05_4 "Seven. \| Of the eight" | **reveal**, then its count | 0.7 s |
| at 05_6 "sharper. \| Our class" | continuation | 0.55 s |
| at 05_7 "a quarter, \| a yes" | comma, continuation | 0.35 s |
| at 05_9 "for everyone. \| Nothing" | continuation | 0.55 s |
| at 05_10 "In between" | turn to the conclusion | 0.6 s |

Units (unqualified proposals, text only): 05u1 the dial and the question
(05_1 to 05_3, ending on the question); 05u2 the answer and the price
(05_4 to 05_6); 05u3 down and the edges (05_7 to 05_9); 05u4 the promise
(05_10).
