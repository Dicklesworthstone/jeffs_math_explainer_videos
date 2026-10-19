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
whom copied homework, and one coin rule defined before any formula. Only
chapter 01 is storyboarded and built; the rest is the plan its facts and
model already compute (`src/model.rs`, `src/facts.rs`, `oracle.py`).

| Chapter | Spine | Status |
|---|---|---|
| `01_the_coin` | a direct question exposes whoever says yes; the coin rule (heads, the truth; tails, a second coin answers); the class routed through it on average: half tells the truth (two yes, eight no), the second coin answers the other ten (five and five); the survey sees seven yes and thirteen no; take away the second coin's yeses on average, a quarter of the class, and double what is left: four; but a yes is three times as likely from someone who copied | source written 2026-10-19 (EmeraldTern, `.8.1`); not compiled, rendered, narrated or reviewed |
| `02_what_a_yes_says` *(planned)* | both answers' ratios, three for a yes and three for a no, counted from the four coin outcomes; among the seven yeses, on average three copied (3/7 against the class's 1/5), among the thirteen noes one (1/13); deniable is not meaningless | facts and model only |
| `03_the_price_of_the_coin` *(planned)* | the yes count of a real class scatters: its exact distribution over the 4²⁰ coin outcomes; the estimate's standard deviation √15 ≈ 3.87 people, exactly four only about one class in five, negative about one in eleven; for independent draws, with replacement or from a large population, the coins add 3/(4n) to the share's sampling variance p(1 − p)/n: at one fifth the added part alone is 75/16 times the direct question's variance and the total 91/16 times; drawing the whole class without replacement leaves only the coins' part | facts and model only |
| `04_asking_again` *(planned)* | one person asked three times with fresh coins: all 64 coin outcomes counted, three yeses 27 times likelier from someone who copied (posterior 27/31); a majority vote right 27 times in 32; each fresh answer multiplies the worst ratio by three; repeating one memoized answer keeps it at three (RAPPOR's permanent randomized response) | facts and model only |
| `05_the_dial` *(planned)* | a truth chance t instead of a fair first coin: ratio (1 + t)/(1 − t), coin variance (1 − t²)/(4t²); the edges, the direct question (no bound) and a pure coin (no estimate) | model only |

## Vocabulary (one word per idea)

**class** (the twenty; never "sample" or "population" in chapter 01's
voice); **copied** (the secret, the true answer; "copied homework");
**coin** and **second coin** (fair and private; the first is "a coin");
**heads** and **tails**; **tell the truth** (what heads means); **answer**
(what someone says, yes or no); **the survey** (who sees only the answers);
**on average** (every coin outcome equally often; never "expected" in the
voice). "Randomized response", "likelihood ratio" and "differential privacy"
are not said in chapter 01.

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
